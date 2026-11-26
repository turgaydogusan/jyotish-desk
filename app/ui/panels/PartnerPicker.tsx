// Uyum panelinin eş seçicisi (ileri modüller spec'i §9 "Arayüz"): palet tarzı danışan araması,
// ardından (danışanın birden çok haritası varsa) harita seçimi. WAI-ARIA combobox + listbox;
// ↑/↓ etkin öğeyi taşır, Enter seçer, Esc kapatır. Danışanlar Rust'ta aranır (`search_clients`;
// boş sorguda son açılanlar, `list_clients`), haritalar `list_charts` ile okunur. Açık haritanın
// kendisi eş olamaz. Seçim yalnızca kimlik taşır; arama metni ve sonuçlar loglanmaz.
import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { ChartRecord, Client, CommandError } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { Dialog } from "../shell/Dialog";
import "../palette/palette.css";

export const PICKER_SEARCH_DEBOUNCE_MS = 120;
export const PICKER_CLIENTS = 8;

interface Answer {
  q: string;
  clients: Client[];
  error: CommandError | null;
}

export interface PartnerPickerProps {
  /** Açık haritanın kimliği: listede sunulmaz. */
  openChartId: number;
  onClose(): void;
  /** Seçilen harita ve danışanı. Pencere önce kapanır, sonra çağrılır. */
  onPick(chart: ChartRecord, client: Client): void;
}

export function PartnerPicker({ openChartId, onClose, onPick }: PartnerPickerProps) {
  const { t, i18n } = useTranslation();
  const id = useId();
  const [query, setQuery] = useState("");
  const [answer, setAnswer] = useState<Answer | null>(null);
  const [active, setActive] = useState(0);
  // Danışan seçildikten sonra: seçilebilir haritaları (açık harita dışında) ya da okuma hatası.
  const [step, setStep] = useState<{ client: Client; charts: ChartRecord[]; error: CommandError | null } | null>(null);
  const seq = useRef(0);
  const listRef = useRef<HTMLUListElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const q = query.trim();
  useEffect(() => {
    const mine = ++seq.current;
    const timer = setTimeout(
      () => {
        const call = q === "" ? api.listClients(PICKER_CLIENTS) : api.searchClients(q, PICKER_CLIENTS).then((r) => (r.status === "ok" ? { status: "ok" as const, data: r.data.map((h) => h.client) } : r));
        void call.then((r) => {
          if (mine !== seq.current) return;
          setAnswer(r.status === "ok" ? { q, clients: r.data, error: null } : { q, clients: [], error: r.error });
        });
      },
      q === "" ? 0 : PICKER_SEARCH_DEBOUNCE_MS,
    );
    return () => clearTimeout(timer);
  }, [q]);
  // Yanıt yalnızca yazılı metne aitse kullanılır: beklerken eski danışanlar listede durmaz.
  const settled = answer !== null && answer.q === q;
  const clients = settled ? answer.clients : [];
  const searchError = settled ? answer.error : null;

  useEffect(() => setActive(0), [query, step]);
  // Adım değişince odak gövdeye düşmez: ikinci adımda harita listesine, dönüşte arama kutusuna gelir.
  const stepped = useRef(false);
  useEffect(() => {
    if (step !== null) {
      stepped.current = true;
      listRef.current?.focus();
    } else if (stepped.current) inputRef.current?.focus();
  }, [step]);

  const pick = (chart: ChartRecord, client: Client) => {
    onClose();
    onPick(chart, client);
  };

  const chooseClient = (client: Client) => {
    const mine = ++seq.current;
    void api.listCharts(client.id).then((r) => {
      if (mine !== seq.current) return;
      if (r.status === "error") {
        setStep({ client, charts: [], error: r.error });
        return;
      }
      const charts = r.data.filter((c) => c.id !== openChartId);
      // Tek seçenek varsa ikinci adım gösterilmez.
      if (charts.length === 1) pick(charts[0], client);
      else setStep({ client, charts, error: null });
    });
  };

  const back = () => {
    seq.current += 1;
    setStep(null);
  };
  const count = step ? step.charts.length : clients.length;
  const current = Math.min(active, count - 1);
  useEffect(() => {
    listRef.current?.children[current]?.scrollIntoView?.({ block: "nearest" });
  }, [current, count]);

  const choose = (index: number) => {
    if (step) {
      const chart = step.charts[index];
      if (chart) pick(chart, step.client);
    } else {
      const client = clients[index];
      if (client) chooseClient(client);
    }
  };

  const onKeyDown = (e: KeyboardEvent<HTMLElement>) => {
    // IME bileşimini bitiren Enter seçim yapmaz (WebKit bunu 229 koduyla da bildirir).
    if (e.nativeEvent.isComposing || e.keyCode === 229) return;
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      if (count === 0) return;
      setActive((current + (e.key === "ArrowDown" ? 1 : -1) + count) % count);
    } else if (e.key === "Enter") {
      e.preventDefault();
      choose(current);
    }
  };

  const optionId = (i: number) => `${id}-opt-${i}`;
  const chartLabel = (c: ChartRecord) => c.label || t(`chart.kind.${c.kind}`);
  return (
    <Dialog title={t("compat.pickTitle")} onClose={onClose}>
      <div className="palette">
        {step === null ? (
          <>
            <label htmlFor={`${id}-q`} className="visually-hidden">
              {t("compat.pickSearch")}
            </label>
            <input
              ref={inputRef}
              id={`${id}-q`}
              data-autofocus
              role="combobox"
              aria-expanded="true"
              aria-controls={`${id}-list`}
              aria-autocomplete="list"
              aria-activedescendant={count > 0 ? optionId(current) : undefined}
              aria-describedby={`${id}-hint`}
              placeholder={t("compat.pickPlaceholder")}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={onKeyDown}
            />
            <p id={`${id}-hint`} className="palette-hint">
              {t("compat.pickHint")}
            </p>
            <ul ref={listRef} id={`${id}-list`} role="listbox" aria-label={t("compat.pickClients")} className="palette-list">
              {clients.map((c, i) => (
                <li
                  key={c.id}
                  id={optionId(i)}
                  role="option"
                  aria-selected={i === current}
                  className={i === current ? "is-active" : undefined}
                  onMouseDown={(e) => e.preventDefault()}
                  onClick={() => choose(i)}
                >
                  <span className="palette-label">{c.name}</span>
                </li>
              ))}
            </ul>
            {searchError && (
              <p className="palette-error" role="alert">
                {t("clients.loadFailed", { message: errorMessage(t, i18n, searchError) })}
              </p>
            )}
            {settled && !searchError && clients.length === 0 && <p className="palette-empty">{t("compat.pickNoClient")}</p>}
          </>
        ) : (
          <>
            <p id={`${id}-hint`} className="palette-hint">
              {t("compat.pickCharts", { name: step.client.name })}
            </p>
            {/* Liste odak alır: ok tuşları ve Enter burada da çalışır. */}
            <ul
              ref={listRef}
              id={`${id}-list`}
              role="listbox"
              tabIndex={0}
              aria-label={t("compat.pickCharts", { name: step.client.name })}
              aria-activedescendant={count > 0 ? optionId(current) : undefined}
              className="palette-list"
              onKeyDown={onKeyDown}
            >
              {step.charts.map((c, i) => (
                <li key={c.id} id={optionId(i)} role="option" aria-selected={i === current} className={i === current ? "is-active" : undefined} onClick={() => choose(i)}>
                  <span className="palette-label">{chartLabel(c)}</span>
                  <span className="palette-detail">{c.birth.date}</span>
                </li>
              ))}
            </ul>
            {step.error && (
              <p className="palette-error" role="alert">
                {t("clients.loadFailed", { message: errorMessage(t, i18n, step.error) })}
              </p>
            )}
            {!step.error && step.charts.length === 0 && <p className="palette-empty">{t("compat.pickNoOtherChart")}</p>}
            <div>
              <button type="button" onClick={back}>
                {t("compat.pickBack")}
              </button>
            </div>
          </>
        )}
        <p className="visually-hidden" role="status">
          {t("compat.pickCount", { n: count })}
        </p>
      </div>
    </Dialog>
  );
}
