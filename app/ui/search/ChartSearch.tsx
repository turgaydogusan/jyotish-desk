// Harita araması yüzeyi (ileri modüller spec'i §12 "Arayüz"): sağdan kayan, tam yükseklikte bir
// pencere. Pano paneli değildir: bir haritaya değil danışan veritabanına aittir. Koşul satırları,
// kapsam, kayıtlı aramalar, "Ara" / "Durdur" ve geldikçe listelenen sonuçlar. Sonuç satırı
// haritayı açar; "Sonuçları etiketle" onay penceresinden sonra toplu yazar.
//
// Arayüz hesap yapmaz (C4): koşulları Rust değerlendirir; buradaki denetim yalnızca biçimdir.
// Sorgu ve sonuçlar (adlar, sayılar, harita kimlikleri) loglanmaz.
import { useEffect, useId, useLayoutEffect, useRef, useState, type FormEvent, type RefObject } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { ChartQuery, CommandError, QueryCondition, QueryScope, SavedSearch } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { useModule } from "../modules/useModule";
import { ConfirmDialog } from "../shell/ConfirmDialog";
import { Dialog } from "../shell/Dialog";
import { ConditionRow } from "./ConditionRow";
import { EXAMPLE_CONDITION, hitSummary, MAX_CONDITIONS, MAX_SAVED_SEARCHES, MAX_SCOPE_TAGS, defaultCondition, parseTags, queryIssue, queryKey, SCOPE_KINDS } from "./model";
import "./search.css";
import { useChartSearch, type ChartSearch as Search } from "./useChartSearch";

/** Listede çizilen en çok eşleşme; fazlası sayılır ve koşulların daraltılması önerilir. */
export const MAX_SHOWN_HITS = 500;

export interface ChartSearchProps {
  /** Koşullar kabukta durur: yüzey kapanıp açılınca form yerindedir. */
  query: ChartQuery;
  onQueryChange(query: ChartQuery): void;
  onClose(): void;
  /** Sonuç satırı: haritayı açar (kabuk yüzeyi kapatır). */
  onOpenChart(clientId: number, chartId: number): void;
  /** Toplu etiketlemeden sonra: liste görünümleri ve (etiketlenenler arasındaysa) açık haritanın etiketleri yenilenir. */
  onTagged(chartIds: number[]): void;
}

export function ChartSearch({ query, onQueryChange, onClose, onOpenChart, onTagged }: ChartSearchProps) {
  const { t } = useTranslation();
  const id = useId();
  const search = useChartSearch();
  const running = search.status === "running";
  const addButton = useRef<HTMLButtonElement>(null);
  const runButton = useRef<HTMLButtonElement>(null);
  const [tagsText, setTagsText] = useState(() => query.scope.tags.join(", "));

  const setConditions = (all_of: QueryCondition[]) => onQueryChange({ ...query, all_of });
  const setScope = (patch: Partial<QueryScope>) => onQueryChange({ ...query, scope: { ...query.scope, ...patch } });
  const issue = queryIssue(query);
  const empty = query.all_of.length === 0;
  const why = empty ? "noConditions" : issue !== null ? "incomplete" : null;
  const full = query.all_of.length >= MAX_CONDITIONS;
  // Liste, başlatıldığı sorguya aittir: form sonradan değişirse arama durdurulur ve bu söylenir.
  const key = queryKey(query);
  const stale = search.query !== null && queryKey(search.query) !== key;
  const { stop } = search;
  useEffect(() => {
    if (running && stale) stop();
  }, [running, stale, stop]);

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (why !== null || running) return;
    search.start(query);
  };
  const remove = (index: number) => {
    // Silinen satırın düğmesi DOM'dan gider: odak "Koşul ekle"ye taşınır.
    addButton.current?.focus();
    setConditions(query.all_of.filter((_, i) => i !== index));
  };

  return (
    <Dialog title={t("search.title")} variant="sheet" onClose={onClose}>
      <form className="search-form" aria-label={t("search.form")} onSubmit={submit}>
        <SavedSearches
          query={query}
          saveWhy={why}
          onLoad={(q) => {
            setTagsText(q.scope.tags.join(", "));
            onQueryChange(q);
          }}
        />
        <p className="chart-toolbar-hint">{t("search.allOfHint")}</p>
        <p className="chart-toolbar-hint">{t("search.housesNote")}</p>
        {query.all_of.map((c, i) => (
          <ConditionRow key={i} index={i} value={c} onChange={(next) => setConditions(query.all_of.map((x, j) => (j === i ? next : x)))} onRemove={() => remove(i)} />
        ))}
        {empty && <p className="panel-message">{t("search.emptyHint")}</p>}
        <div className="search-toolbar">
          <button
            ref={addButton}
            type="button"
            data-autofocus
            aria-disabled={full ? true : undefined}
            aria-describedby={full ? `${id}-full` : undefined}
            onClick={() => {
              if (!full) setConditions([...query.all_of, empty ? EXAMPLE_CONDITION : { cond: defaultCondition("GrahaInSign"), negate: false }]);
            }}
          >
            {t(empty ? "search.addExample" : "search.add")}
          </button>
          {full && (
            <span id={`${id}-full`} className="chart-toolbar-hint">
              {t("search.full", { max: MAX_CONDITIONS })}
            </span>
          )}
        </div>
        <fieldset className="search-scope">
          <legend>{t("search.scope.title")}</legend>
          <span className="search-field">
            <label htmlFor={`${id}-tags`}>{t("search.scope.tags")}</label>
            <input
              id={`${id}-tags`}
              type="text"
              value={tagsText}
              aria-describedby={`${id}-tags-hint`}
              onChange={(e) => {
                setTagsText(e.target.value);
                setScope({ tags: parseTags(e.target.value) });
              }}
            />
            <span id={`${id}-tags-hint`} className="chart-toolbar-hint">
              {t("search.scope.tagsHint", { max: MAX_SCOPE_TAGS })}
            </span>
          </span>
          <span className="search-field">
            <label htmlFor={`${id}-kind`}>{t("search.scope.kind")}</label>
            <select id={`${id}-kind`} value={query.scope.kind ?? ""} onChange={(e) => setScope({ kind: SCOPE_KINDS.find((k) => k === e.target.value) ?? null })}>
              <option value="">{t("search.scope.kindAny")}</option>
              {SCOPE_KINDS.map((k) => (
                <option key={k} value={k}>
                  {t(`search.scope.${k}`)}
                </option>
              ))}
            </select>
          </span>
          <label>
            <input type="checkbox" checked={query.scope.exclude_prashna} onChange={(e) => setScope({ exclude_prashna: e.target.checked })} />
            {t("search.scope.excludePrashna")}
          </label>
        </fieldset>
        <div className="search-toolbar">
          {/* Kullanılamazken yerel `disabled` değildir: odaklanır ve nedeni görünür bir satırdır. */}
          <button ref={runButton} type="submit" aria-disabled={why !== null || running ? true : undefined} aria-describedby={why !== null ? `${id}-why` : undefined}>
            {t("search.run")}
          </button>
          {running && <StopButton onStop={search.stop} fallback={runButton} />}
          {why !== null && (
            <span id={`${id}-why`} className="chart-toolbar-hint">
              {why === "incomplete" && issue !== null ? t("search.why.incomplete", { n: issue.row + 1 }) : t("search.why.noConditions")}
            </span>
          )}
        </div>
      </form>
      <Results search={search} stale={stale} onRetry={() => {
          // "Yeniden dene" yeni arama başlayınca DOM'dan gider: odak önce "Ara"ya taşınır.
          runButton.current?.focus();
          runButton.current?.click();
        }} onOpenChart={onOpenChart} onTagged={onTagged} />
      <div className="dialog-actions">
        <button type="button" onClick={onClose}>
          {t("search.close")}
        </button>
      </div>
    </Dialog>
  );
}

/**
 * "Durdur": arama bitince ya da durunca DOM'dan gider. Odak ondaysa gövdeye düşmesin diye
 * "Ara" düğmesine taşınır (layout effect'in temizliği düğüm kaldırılmadan önce koşar).
 */
function StopButton({ onStop, fallback }: { onStop(): void; fallback: RefObject<HTMLButtonElement | null> }) {
  const { t } = useTranslation();
  const ref = useRef<HTMLButtonElement>(null);
  useLayoutEffect(() => {
    const button = ref.current;
    return () => {
      if (button !== null && document.activeElement === button) fallback.current?.focus({ preventScroll: true });
    };
  }, [fallback]);
  return (
    <button ref={ref} type="button" onClick={onStop}>
      {t("search.stop")}
    </button>
  );
}

/** Kayıtlı aramalar: seç (koşulları forma yükler), adlandırıp sakla, sil. Yalnızca koşullar saklanır. */
function SavedSearches({ query, saveWhy: queryWhy, onLoad }: { query: ChartQuery; saveWhy: string | null; onLoad(query: ChartQuery): void }) {
  const { t, i18n } = useTranslation();
  const id = useId();
  const loaded = useModule("saved-searches", () => api.listSavedSearches());
  // Kayıttan ve silmeden sonra liste yerinde güncellenir (yeniden okunmaz: seçici DOM'da durur).
  const [written, setWritten] = useState<SavedSearch[] | null>(null);
  const stored = written ?? loaded.data;
  const known = stored !== null;
  const list = stored ?? [];
  const [selected, setSelected] = useState<number | null>(null);
  const [name, setName] = useState("");
  const [failed, setFailed] = useState<CommandError | null>(null);
  const [busy, setBusy] = useState(false);
  const select = useRef<HTMLSelectElement>(null);
  const error = failed ?? loaded.error;

  // Store'un karşılaştırmasıyla aynı (`to_lowercase`): arayüz dilinden bağımsız.
  const same = (a: string, b: string) => a.trim().toLowerCase() === b.trim().toLowerCase();
  const trimmed = name.trim();
  const existing = list.find((s) => same(s.name, trimmed));
  const unread = known ? null : loaded.error ? "unread" : "loading";
  const saveWhy = unread ?? (queryWhy !== null ? "queryIncomplete" : trimmed === "" ? "nameNeeded" : !existing && list.length >= MAX_SAVED_SEARCHES ? "capReached" : null);
  const removeWhy = unread ?? (selected === null ? "selectToRemove" : null);

  const save = async () => {
    if (saveWhy !== null || busy) return;
    setBusy(true);
    const r = await api.saveSearch({ name: trimmed, query });
    setBusy(false);
    if (r.status === "error") return setFailed(r.error);
    setFailed(null);
    const saved = r.data;
    setWritten([...list.filter((s) => s.id !== saved.id), saved].sort((a, b) => a.name.localeCompare(b.name)));
    setSelected(saved.id);
    setName(saved.name);
  };
  const remove = async () => {
    if (removeWhy !== null || selected === null || busy) return;
    setBusy(true);
    const r = await api.deleteSavedSearch(selected);
    setBusy(false);
    // Başka bir yerden silinmiş kayıt da listeden düşer.
    if (r.status === "error" && r.error.code !== "not_found") return setFailed(r.error);
    setFailed(null);
    setWritten(list.filter((s) => s.id !== selected));
    setSelected(null);
    setName("");
    // "Sil" seçim boşalınca kullanılamaz olur: odak seçiciye taşınır.
    select.current?.focus();
  };

  return (
    <div className="search-toolbar search-saved">
      <span className="search-field">
        <label htmlFor={`${id}-saved`}>{t("search.saved.label")}</label>
        <select
          ref={select}
          id={`${id}-saved`}
          value={selected ?? ""}
          onChange={(e) => {
            const found = list.find((s) => String(s.id) === e.target.value);
            setSelected(found?.id ?? null);
            setName(found?.name ?? "");
            if (found) onLoad(found.query);
          }}
        >
          <option value="">{t("search.saved.none")}</option>
          {list.map((s) => (
            <option key={s.id} value={s.id}>
              {s.name}
            </option>
          ))}
        </select>
      </span>
      <span className="search-field">
        <label htmlFor={`${id}-name`}>{t("search.saved.name")}</label>
        <input
          id={`${id}-name`}
          type="text"
          value={name}
          maxLength={200}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => {
            // Enter bu alanda aramayı saklar; çevredeki arama formunu göndermez.
            if (e.key !== "Enter") return;
            e.preventDefault();
            void save();
          }}
        />
      </span>
      <button type="button" aria-disabled={saveWhy !== null || busy ? true : undefined} aria-describedby={saveWhy !== null ? `${id}-save-why` : undefined} onClick={() => void save()}>
        {t("search.saved.save")}
      </button>
      <button
        type="button"
        aria-disabled={removeWhy !== null || busy ? true : undefined}
        aria-describedby={removeWhy === null ? undefined : removeWhy === saveWhy ? `${id}-save-why` : `${id}-remove-why`}
        onClick={() => void remove()}
      >
        {t("search.saved.remove")}
      </button>
      {error && (
        <span className="panel-error" role="alert">
          {errorMessage(t, i18n, error)}
        </span>
      )}
      {!known && loaded.error && (
        <button
          type="button"
          onClick={() => {
            // Düğme yeniden okuma başlayınca kalkar: odak seçiciye taşınır.
            select.current?.focus();
            loaded.retry();
          }}
        >
          {t("search.saved.reload")}
        </button>
      )}
      {saveWhy !== null && (
        <span id={`${id}-save-why`} className="chart-toolbar-hint">
          {t(`search.saved.${saveWhy}`, { max: MAX_SAVED_SEARCHES })}
        </span>
      )}
      {removeWhy !== null && removeWhy !== saveWhy && (
        <span id={`${id}-remove-why`} className="chart-toolbar-hint">
          {t(`search.saved.${removeWhy}`)}
        </span>
      )}
    </div>
  );
}

function Results({ search, stale, onRetry, onOpenChart, onTagged }: { search: Search; stale: boolean; onRetry(): void; onOpenChart(clientId: number, chartId: number): void; onTagged(chartIds: number[]): void }) {
  const { t, i18n } = useTranslation();
  const listId = useId();
  const { status, hits, scanned, total } = search;
  const sent = search.query;
  if (status === "idle" || sent === null) return <p className="panel-message">{t("search.idle")}</p>;
  const shown = hits.slice(0, MAX_SHOWN_HITS);
  const counts = { n: hits.length, scanned, total: total ?? 0 };
  // İlk sayfadan önce toplam bilinmez: sayı uydurulmaz, yalnızca "Aranıyor…" yazılır.
  const summary = total === null ? (status === "running" ? t("search.summary.starting") : status === "stopped" ? t("search.summary.stoppedEarly") : null) : t(`search.summary.${status}`, counts);
  const finished = status === "done" && !stale;
  return (
    <section className="search-results" aria-labelledby={listId}>
      <h3 id={listId}>{t("search.results")}</h3>
      {/* İlerleme ve sonuç sayısı tek bir canlı bölgede: her sayfada yeniden okunur. */}
      <p className="panel-message" role="status" data-summary>
        {summary ?? ""}
      </p>
      {status === "running" && total !== null && <progress className="search-progress" max={Math.max(total, 1)} value={scanned} aria-label={t("search.progress")} />}
      {search.skippedTimeUnknown > 0 && <p className="panel-notice">{t("search.skippedTimeUnknown", { n: search.skippedTimeUnknown })}</p>}
      {search.failed > 0 && <p className="panel-notice">{t("search.failed", { n: search.failed })}</p>}
      {stale && (
        <p className="panel-notice" role="note">
          {t("search.stale")}
        </p>
      )}
      {search.error && (
        <p className="panel-error" role="alert">
          {errorMessage(t, i18n, search.error)}{" "}
          <button type="button" onClick={onRetry}>
            {t("search.retry")}
          </button>
        </p>
      )}
      {status === "done" && hits.length === 0 && <p className="panel-message">{t(scanned === 0 ? "search.noCharts" : "search.noHits", { scanned })}</p>}
      {/* Etiketleme listeden önce durur: klavyeyle ona varmak için yüzlerce sonuç düğmesinden geçilmez. */}
      {hits.length > 0 && <TagResults chartIds={hits.map((h) => h.chart_id)} finished={finished} onTagged={onTagged} />}
      {shown.length > 0 && (
        <ul className="search-hits">
          {shown.map((h) => {
            const chart = h.chart_label === "" ? t("search.unlabeled") : h.chart_label;
            const factsId = `${listId}-facts-${h.chart_id}`;
            return (
              <li key={h.chart_id}>
                <button type="button" className="search-hit" aria-label={t("search.open", { client: h.client_name, chart })} aria-describedby={factsId} onClick={() => onOpenChart(h.client_id, h.chart_id)}>
                  <span className="search-hit-name">{h.client_name}</span>
                  <span className="search-hit-chart">{chart}</span>
                  <span className="search-hit-facts" id={factsId}>{hitSummary(t, sent, h.matched)}</span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
      {hits.length > shown.length && <p className="panel-notice">{t("search.capped", { shown: shown.length, n: hits.length })}</p>}
    </section>
  );
}

/** "Sonuçları etiketle": etiket adı, sonra etiketi ve etkilenecek harita sayısını gösteren onay. */
function TagResults({ chartIds, finished, onTagged }: { chartIds: number[]; finished: boolean; onTagged(chartIds: number[]): void }) {
  const { t, i18n } = useTranslation();
  const id = useId();
  const [tag, setTag] = useState("");
  const [confirming, setConfirming] = useState(false);
  const [busy, setBusy] = useState(false);
  const [done, setDone] = useState<number | null>(null);
  const [error, setError] = useState<CommandError | null>(null);
  const trimmed = tag.trim();
  const why = !finished ? "notFinished" : trimmed === "" ? "tagNeeded" : trimmed.includes(",") ? "oneTag" : null;
  const apply = async () => {
    setConfirming(false);
    setBusy(true);
    const r = await api.tagCharts({ chart_ids: chartIds, tag: trimmed });
    setBusy(false);
    if (r.status === "error") {
      setDone(null);
      return setError(r.error);
    }
    setError(null);
    setDone(r.data);
    onTagged(chartIds);
  };
  return (
    <div className="search-toolbar search-tag">
      <span className="search-field">
        <label htmlFor={`${id}-tag`}>{t("search.tag.label")}</label>
        <input
          id={`${id}-tag`}
          type="text"
          value={tag}
          maxLength={64}
          onChange={(e) => {
            setTag(e.target.value);
            setDone(null);
          }}
        />
      </span>
      <button
        type="button"
        aria-disabled={why !== null || busy ? true : undefined}
        aria-describedby={why !== null ? `${id}-why` : undefined}
        onClick={() => {
          if (why === null && !busy) setConfirming(true);
        }}
      >
        {t("search.tag.apply")}
      </button>
      {why !== null && (
        <span id={`${id}-why`} className="chart-toolbar-hint">
          {t(`search.tag.${why}`)}
        </span>
      )}
      <span role="status">{done === null ? "" : t("search.tag.done", { n: done, tag: trimmed })}</span>
      {error && (
        <span className="panel-error" role="alert">
          {errorMessage(t, i18n, error)}
        </span>
      )}
      {confirming && (
        <ConfirmDialog
          title={t("search.tag.confirmTitle")}
          message={t("search.tag.confirm", { tag: trimmed, n: chartIds.length })}
          confirmLabel={t("search.tag.confirmAction", { n: chartIds.length })}
          onConfirm={() => void apply()}
          onCancel={() => setConfirming(false)}
        />
      )}
    </div>
  );
}
