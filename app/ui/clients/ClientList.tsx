// Kenar çubuğunun liste yarısı (UI spec §4): arama, etiket süzgeci ve son açılanlar. Arama
// `search_clients` (ad, etiket, not), süzgeç `list_clients_by_tag`, boş sorgu `list_clients`
// ile okunur; sıralama Rust'tadır (son açılan önce). Danışan seçilince son haritası açılır.
// Liste tek sekme durağıdır; oklarla gezilir.
import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import { api, type Result } from "../api";
import type { Client, CommandError, SearchHit, TagCount } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { useSession } from "../session/store";
import { useNotices } from "../shell/notices";

export const CLIENT_SEARCH_DEBOUNCE_MS = 150;
/** Kenar çubuğunda gösterilen en çok danışan (Rust `MAX_LIST` 500). */
export const SIDEBAR_LIMIT = 200;
/** Kenar çubuğu arama kutusu: silmeden sonra odak buraya döner (OpenClient). */
export const CLIENT_SEARCH_ID = "client-search";

type Row = { client: Client; hit?: SearchHit };

/** Etiketleri dilin alfabesiyle sıralar (Rust yalnızca ASCII büyük/küçük harf katlar). */
export function sortTags(tags: TagCount[], language: string): TagCount[] {
  const collator = new Intl.Collator(language, { sensitivity: "base" });
  return [...tags].sort((a, b) => collator.compare(a.name, b.name));
}

export function ClientList() {
  const { t, i18n } = useTranslation();
  const session = useSession();
  const { push } = useNotices();
  const id = useId();
  const [query, setQuery] = useState("");
  const [tag, setTag] = useState("");
  const [tags, setTags] = useState<TagCount[]>([]);
  const [rows, setRows] = useState<Row[] | null>(null);
  const [error, setError] = useState<CommandError | null>(null);
  const [retry, setRetry] = useState(0);
  const seq = useRef(0);
  // Liste tek sekme durağıdır: ↑/↓/Home/End satırlarda gezer (roving tabindex). Durak danışana
  // bağlıdır, sıraya değil: liste yeniden okununca gezilen satırda kalır.
  const [focusId, setFocusId] = useState<number | null>(null);
  const rowsRef = useRef<HTMLUListElement>(null);

  useEffect(() => {
    void api.listTags().then((r) => r.status === "ok" && setTags(r.data));
  }, [session.revision]);

  useEffect(() => {
    const mine = ++seq.current;
    const q = query.trim();
    const run = async () => {
      let r: Result<Row[]>;
      if (q !== "") {
        const hits = await api.searchClients(q, SIDEBAR_LIMIT);
        r = hits.status === "ok" ? { status: "ok", data: hits.data.map((hit) => ({ client: hit.client, hit })) } : hits;
      } else {
        const list = tag !== "" ? await api.listClientsByTag(tag, SIDEBAR_LIMIT) : await api.listClients(SIDEBAR_LIMIT);
        r = list.status === "ok" ? { status: "ok", data: list.data.map((client) => ({ client })) } : list;
      }
      if (mine !== seq.current) return;
      if (r.status === "ok") {
        setRows(r.data);
        setError(null);
      } else setError(r.error);
    };
    // Yalnızca yazarken beklenir; süzgeç, yenileme ve ilk okuma hemen.
    const timer = setTimeout(() => void run(), q !== "" ? CLIENT_SEARCH_DEBOUNCE_MS : 0);
    return () => clearTimeout(timer);
  }, [query, tag, session.revision, retry]);

  const choose = async (client: Client) => {
    const e = await session.openClient(client.id);
    if (e) push({ id: "client", type: "error", messageKey: "notice.openFailed", error: e });
  };

  const heading = query.trim() !== "" ? t("clients.results") : tag !== "" ? t("clients.tagged", { tag }) : t("clients.recent");
  const openId = session.opened?.client.id;

  // Açık danışan değişince sekme durağı ona döner. Gezilen danışan listede yoksa durak açık
  // danışandadır, o da yoksa ilk satırdadır.
  useEffect(() => setFocusId(null), [openId]);
  const indexOf = (clientId: number | null | undefined) => (rows ? rows.findIndex((r) => r.client.id === clientId) : -1);
  const focusIndex = Math.max(indexOf(focusId) >= 0 ? indexOf(focusId) : indexOf(openId), 0);

  const onRowsKey = (e: KeyboardEvent<HTMLUListElement>) => {
    if (!rows || rows.length === 0) return;
    // Değiştirici tuşlu oklar (⌘↑, ⌥↓, ⇧Home…) sisteme ve tarayıcıya bırakılır.
    if (e.metaKey || e.ctrlKey || e.altKey || e.shiftKey) return;
    const last = rows.length - 1;
    const next =
      e.key === "ArrowDown" ? Math.min(focusIndex + 1, last) : e.key === "ArrowUp" ? Math.max(focusIndex - 1, 0) : e.key === "Home" ? 0 : e.key === "End" ? last : null;
    if (next === null) return;
    e.preventDefault();
    setFocusId(rows[next].client.id);
    rowsRef.current?.querySelectorAll<HTMLButtonElement>("button.client-row")[next]?.focus();
  };

  return (
    <section className="client-list" aria-labelledby={`${id}-heading`}>
      <div className="sidebar-field">
        <label htmlFor={CLIENT_SEARCH_ID}>{t("clients.search")}</label>
        <input
          id={CLIENT_SEARCH_ID}
          type="search"
          value={query}
          aria-describedby={`${id}-q-hint`}
          onChange={(e) => {
            setQuery(e.target.value);
            if (e.target.value.trim() !== "") setTag("");
          }}
        />
        <p id={`${id}-q-hint`} className="sidebar-hint">
          {t("clients.searchHint")}
        </p>
      </div>
      <div className="sidebar-field">
        <label htmlFor={`${id}-tag`}>{t("clients.tag")}</label>
        <select
          id={`${id}-tag`}
          value={tag}
          onChange={(e) => {
            setTag(e.target.value);
            setQuery("");
          }}
        >
          <option value="">{t("clients.allTags")}</option>
          {sortTags(tags, i18n.language).map((tc) => (
            <option key={tc.name} value={tc.name}>
              {t("clients.tagOption", { tag: tc.name, n: tc.charts })}
            </option>
          ))}
        </select>
      </div>
      <h2 id={`${id}-heading`} className="sidebar-heading">
        {heading}
      </h2>
      {error && (
        <div className="sidebar-error" role="alert">
          <p>{t("clients.loadFailed", { message: errorMessage(t, i18n, error) })}</p>
          <button type="button" onClick={() => setRetry((n) => n + 1)}>
            {t("clients.retry")}
          </button>
        </div>
      )}
      {rows && rows.length === 0 && !error && <p className="sidebar-empty">{query.trim() !== "" || tag !== "" ? t("clients.noResults") : t("clients.none")}</p>}
      {rows && rows.length > 0 && (
        <ul ref={rowsRef} className="client-rows" onKeyDown={onRowsKey}>
          {rows.map(({ client, hit }, i) => (
            <li key={client.id}>
              <button
                type="button"
                className="client-row"
                tabIndex={i === focusIndex ? 0 : -1}
                aria-current={client.id === openId ? "true" : undefined}
                onFocus={() => setFocusId(client.id)}
                onClick={() => void choose(client)}
              >
                <span className="client-name">{client.name}</span>
                {hit?.matched === "Tag" && <span className="client-meta">{t("clients.matchedTag")}</span>}
                {hit?.matched === "Note" && hit.snippet && <span className="client-meta">{t("clients.matchedNote", { snippet: hit.snippet })}</span>}
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
