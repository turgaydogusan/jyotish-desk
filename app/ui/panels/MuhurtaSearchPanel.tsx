// Muhurta araması paneli (ileri modüller spec'i §10): süzgeç formu, "Ara" ve sonuç listesi.
// Süzgeci astrolog kurar; hazır etkinlik şablonu, "uygun" ya da "uygunsuz" hükmü ve açıklama
// metni yoktur. Koşullar panelin ayarıdır (düzenle birlikte saklanır) ve adlandırılıp
// kaydedilebilir; kayıt yer, tarih ve kişi taşımaz. Pencereler `muhurta_search`'ten gelir;
// burada hesap yoktur (C4). Satırdaki başlangıç, transit tarihini o ana götürür.
import { useId, useLayoutEffect, useRef, useState, type FormEvent, type RefObject } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError, MuhurtaCriteria, MuhurtaWindowView, OpenedClient, SavedMuhurtaFilter } from "../bindings";
import { signIndex, SIGNS } from "../chart/adapter";
import { errorMessage } from "../i18n/errors";
import type { Panel } from "../layout/model";
import { MAX_MIN_MINUTES, MUHURTA_FLAGS, muhurtaPlace, natalMoon, parseCriteria, toggleMember, VARAS } from "../modules/muhurta";
import { useModule } from "../modules/useModule";
import { useMuhurtaSearch, type MuhurtaSearch } from "../modules/useMuhurtaSearch";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useTransitControl } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { Approx } from "./Approx";
import { formatJdClock, formatJdDate, formatJdSpan } from "./format";
import { nakshatraName, nityaYogaName, tithiName, tithiShort } from "./labels";
import { PlaceControls, PlaceMissing, usePanelPlace } from "./MuhurtaPlace";
import "./panels.css";
import { PanelError, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Listede çizilen en çok pencere; fazlası sayılır ve süzgecin daraltılması önerilir. */
export const MAX_SHOWN_WINDOWS = 500;
const TITHIS = Array.from({ length: 30 }, (_, i) => i + 1);
const NAKSHATRAS = Array.from({ length: 27 }, (_, i) => i + 1);

export function MuhurtaSearchPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Search panel={panel} opened={opened} />;
}

/** Onay kutusu kümesi: görünür başlık ve her değer için etiketli bir kutu. */
function CheckGroup<T extends string | number>({
  legend,
  options,
  selected,
  onToggle,
  fold,
}: {
  legend: string;
  options: readonly { value: T; label: string }[];
  selected: readonly T[];
  onToggle(value: T): void;
  /** Uzun kümeler açılır kapanır bir bölümde durur; başlık seçili sayıyı söyler. */
  fold?: string;
}) {
  const boxes = (
    <fieldset className="muhurta-checks">
      <legend className={fold ? "visually-hidden" : undefined}>{legend}</legend>
      {options.map((o) => (
        <label key={o.value}>
          <input type="checkbox" checked={selected.includes(o.value)} onChange={() => onToggle(o.value)} />
          {o.label}
        </label>
      ))}
    </fieldset>
  );
  if (!fold) return boxes;
  return (
    <details className="muhurta-fold">
      <summary>{fold}</summary>
      {boxes}
    </details>
  );
}

function Search({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  const place = usePanelPlace(panel, opened);
  const criteria = parseCriteria(panel.settings.criteria);
  const search = useMuhurtaSearch(String(opened.chart_record.id));
  const running = search.status === "running";
  const phase = usePanelLoadPhase(running);
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const searchButton = useRef<HTMLButtonElement>(null);
  const setCriteria = (patch: Partial<MuhurtaCriteria>) => setPanelSettings(panel.id, { criteria: { ...criteria, ...patch } });
  const unknown = timeUnknown(opened);
  const ready = place.place !== null;

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (place.place === null || running) return;
    search.start(
      { from_date: from === "" ? null : from, to_date: to === "" ? null : to, place: muhurtaPlace(place.place), criteria, natal: natalMoon(opened) },
      opened.chart.meta.settings,
    );
  };

  const count = (n: number) => t(n === 0 ? "muhurtaSearch.any" : "muhurtaSearch.selected", { n });
  return (
    <div className="module-panel muhurta-search">
      <form className="muhurta-form" aria-label={t("muhurtaSearch.form")} onSubmit={submit}>
        <SavedFilters criteria={criteria} onLoad={(c) => setPanelSettings(panel.id, { criteria: c })} />
        <div className="module-toolbar">
          <PlaceControls panel={panel} opened={opened} at={place} />
        </div>
        <div className="module-toolbar">
          <label htmlFor={`${id}-from`}>{t("muhurtaSearch.from")}</label>
          <input id={`${id}-from`} type="date" value={from} aria-describedby={`${id}-range`} onChange={(e) => setFrom(e.target.value)} />
          <label htmlFor={`${id}-to`}>{t("muhurtaSearch.to")}</label>
          <input id={`${id}-to`} type="date" value={to} aria-describedby={`${id}-range`} onChange={(e) => setTo(e.target.value)} />
          <span id={`${id}-range`} className="chart-toolbar-hint">
            {t("muhurtaSearch.rangeHint")}
          </span>
        </div>
        <p className="chart-toolbar-hint">{t("muhurtaSearch.emptySetHint")}</p>
        <CheckGroup
          legend={t("muhurtaSearch.weekdays")}
          options={VARAS.map((v) => ({ value: v, label: t(`panchang.weekday.${v}`) }))}
          selected={criteria.weekdays}
          onToggle={(v) => setCriteria({ weekdays: toggleMember(criteria.weekdays, v, VARAS) })}
        />
        <CheckGroup
          legend={t("muhurtaSearch.tithis")}
          fold={`${t("muhurtaSearch.tithis")} · ${count(criteria.tithis.length)}`}
          options={TITHIS.map((n) => ({ value: n, label: `${tithiShort(n)} ${tithiName(n)}` }))}
          selected={criteria.tithis}
          onToggle={(n) => setCriteria({ tithis: toggleMember(criteria.tithis, n, TITHIS) })}
        />
        <CheckGroup
          legend={t("muhurtaSearch.nakshatras")}
          fold={`${t("muhurtaSearch.nakshatras")} · ${count(criteria.nakshatras.length)}`}
          options={NAKSHATRAS.map((n) => ({ value: n, label: nakshatraName(n) }))}
          selected={criteria.nakshatras}
          onToggle={(n) => setCriteria({ nakshatras: toggleMember(criteria.nakshatras, n, NAKSHATRAS) })}
        />
        <CheckGroup
          legend={t("muhurtaSearch.lagnaSigns")}
          fold={`${t("muhurtaSearch.lagnaSigns")} · ${count(criteria.lagna_signs.length)}`}
          options={SIGNS.map((s) => ({ value: s, label: t(`sign.${signIndex(s)}`) }))}
          selected={criteria.lagna_signs}
          onToggle={(s) => setCriteria({ lagna_signs: toggleMember(criteria.lagna_signs, s, SIGNS) })}
        />
        <CheckGroup
          legend={t("muhurtaSearch.exclude")}
          options={MUHURTA_FLAGS.map((f) => ({ value: f, label: t(`muhurta.flag.${f}`) }))}
          selected={criteria.exclude}
          onToggle={(f) => setCriteria({ exclude: toggleMember(criteria.exclude, f, MUHURTA_FLAGS) })}
        />
        <div className="module-toolbar">
          <label>
            <input type="checkbox" checked={criteria.daytime_only} onChange={(e) => setCriteria({ daytime_only: e.target.checked })} />
            {t("muhurtaSearch.daytimeOnly")}
          </label>
          <label htmlFor={`${id}-min`}>{t("muhurtaSearch.minMinutes")}</label>
          <input
            id={`${id}-min`}
            type="number"
            min={0}
            max={MAX_MIN_MINUTES}
            step={1}
            value={criteria.min_minutes}
            onChange={(e) => {
              const n = Math.floor(Number(e.target.value));
              setCriteria({ min_minutes: Number.isFinite(n) ? Math.min(Math.max(n, 0), MAX_MIN_MINUTES) : 0 });
            }}
          />
          <label>
            <input type="checkbox" checked={criteria.require_tara} onChange={(e) => setCriteria({ require_tara: e.target.checked })} />
            <Approx on={unknown} label={t("muhurtaSearch.natalApprox")}>
              {t("muhurtaSearch.requireTara")}
            </Approx>
          </label>
          <label>
            <input type="checkbox" checked={criteria.require_chandra} onChange={(e) => setCriteria({ require_chandra: e.target.checked })} />
            <Approx on={unknown} label={t("muhurtaSearch.natalApprox")}>
              {t("muhurtaSearch.requireChandra")}
            </Approx>
          </label>
        </div>
        <div className="module-toolbar">
          {/* Kullanılamazken yerel `disabled` değildir: odaklanır ve nedeni görünür bir satırdır. */}
          <button ref={searchButton} type="submit" aria-disabled={!ready || running ? true : undefined} aria-describedby={ready ? undefined : `${id}-why`}>
            {t("muhurtaSearch.run")}
          </button>
          {running && phase === "skeleton" && <StopButton onStop={search.stop} fallback={searchButton} />}
          <span className="muhurta-progress" role="status">
            {running && phase === "skeleton" && search.chunks > 0 ? t("muhurtaSearch.progress", { scanned: search.scanned, chunks: search.chunks }) : ""}
          </span>
        </div>
        {!ready && (
          <div id={`${id}-why`}>
            <PlaceMissing choice={place.choice} />
          </div>
        )}
      </form>
      {search.error && <PanelError error={search.error} onRetry={() => searchButton.current?.click()} />}
      <Results search={search} />
    </div>
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
      {t("muhurtaSearch.stop")}
    </button>
  );
}

/** Kayıtlı süzgeçler: seç, adlandırıp kaydet, sil. Liste yeniden yazılırken yerinde durur. */
function SavedFilters({ criteria, onLoad }: { criteria: MuhurtaCriteria; onLoad(criteria: MuhurtaCriteria): void }) {
  const { t, i18n } = useTranslation();
  const id = useId();
  // Liste bir kez okunur (`useModule` geliştirme kipindeki ikinci effect koşusunda yeniden
  // istemez); kayıttan sonra Rust'ın döndürdüğü kanonik liste gösterilir.
  const loaded = useModule("muhurta-filters", () => api.loadMuhurtaFilters());
  const [written, setWritten] = useState<SavedMuhurtaFilter[] | null>(null);
  const filters = written ?? loaded.data ?? [];
  const [selected, setSelected] = useState("");
  const [name, setName] = useState("");
  const [failed, setFailed] = useState<CommandError | null>(null);
  const error = failed ?? loaded.error;
  const [busy, setBusy] = useState(false);
  const select = useRef<HTMLSelectElement>(null);

  const same = (a: string, b: string) => a.trim().toLocaleLowerCase(i18n.language) === b.trim().toLocaleLowerCase(i18n.language);
  const write = async (next: SavedMuhurtaFilter[], select_: string) => {
    setBusy(true);
    const r = await api.saveMuhurtaFilters(next);
    setBusy(false);
    if (r.status === "error") {
      setFailed(r.error);
      return false;
    }
    setFailed(null);
    setWritten(r.data);
    setSelected(r.data.find((f) => same(f.name, select_))?.name ?? "");
    return true;
  };
  const save = () => {
    const trimmed = name.trim();
    if (trimmed === "" || busy) return;
    const entry = { name: trimmed, criteria };
    const at = filters.findIndex((f) => same(f.name, trimmed));
    void write(at < 0 ? [...filters, entry] : filters.map((f, i) => (i === at ? entry : f)), trimmed);
  };
  const remove = async () => {
    if (selected === "" || busy) return;
    const done = await write(
      filters.filter((f) => f.name !== selected),
      "",
    );
    // "Sil" seçim boşalınca kullanılamaz olur: odak seçiciye taşınır.
    if (done) {
      setName("");
      select.current?.focus();
    }
  };

  return (
    <div className="module-toolbar muhurta-saved">
      <label htmlFor={`${id}-saved`}>{t("muhurtaSearch.saved")}</label>
      <select
        ref={select}
        id={`${id}-saved`}
        value={selected}
        onChange={(e) => {
          const filter = filters.find((f) => f.name === e.target.value);
          setSelected(filter?.name ?? "");
          setName(filter?.name ?? "");
          if (filter) onLoad(parseCriteria(filter.criteria));
        }}
      >
        <option value="">{t("muhurtaSearch.savedNone")}</option>
        {filters.map((f) => (
          <option key={f.name} value={f.name}>
            {f.name}
          </option>
        ))}
      </select>
      <label htmlFor={`${id}-name`}>{t("muhurtaSearch.name")}</label>
      <input id={`${id}-name`} type="text" value={name} maxLength={200} onChange={(e) => setName(e.target.value)} />
      <button type="button" aria-disabled={name.trim() === "" || busy ? true : undefined} onClick={save}>
        {t("muhurtaSearch.save")}
      </button>
      <button type="button" aria-disabled={selected === "" || busy ? true : undefined} onClick={() => void remove()}>
        {t("muhurtaSearch.remove")}
      </button>
      {error && (
        <span className="panel-error" role="alert">
          {errorMessage(t, i18n, error)}
        </span>
      )}
    </div>
  );
}

function Results({ search }: { search: MuhurtaSearch }) {
  const { t } = useTranslation();
  const transit = useTransitControl();
  const captionId = useId();
  if (search.status === "idle") return <p className="panel-message">{t("muhurtaSearch.idle")}</p>;
  const { windows, range } = search;
  const shown = windows.slice(0, MAX_SHOWN_WINDOWS);
  const units = { day: t("muhurtaSearch.unit.day"), hour: t("muhurtaSearch.unit.hour"), minute: t("muhurtaSearch.unit.minute") };
  const moment = (jd: number | null, offset: number) => `${formatJdDate(jd, offset)} ${formatJdClock(jd, offset, jd)}`;
  const row = (w: MuhurtaWindowView, n: number) => {
    const s = w.snapshot;
    const start = moment(w.start_jd_ut, w.start_offset_secs);
    return (
      <tr key={`${n}:${w.start_jd_ut}`} data-window={n}>
        <th scope="row">
          <button
            type="button"
            className="link-button"
            aria-label={t("muhurtaSearch.goTo", { when: start })}
            onClick={() => {
              if (w.start_jd_ut !== null) transit.setAt({ Jd: w.start_jd_ut });
            }}
          >
            {start}
          </button>
        </th>
        <td>{moment(w.end_jd_ut, w.end_offset_secs)}</td>
        <td className="num">{formatJdSpan(w.start_jd_ut, w.end_jd_ut, units)}</td>
        <td>{t(`panchang.weekday.${s.vara}`)}</td>
        <td>{`${tithiShort(s.tithi)} ${tithiName(s.tithi)}`}</td>
        <td>{`${nakshatraName(s.nakshatra)} ${s.pada}`}</td>
        <td>{nityaYogaName(s.yoga)}</td>
        <td>{s.karana_name}</td>
        <td>{t(`sign.${signIndex(s.lagna_sign)}`)}</td>
      </tr>
    );
  };
  return (
    <>
      <p className="panel-message" data-summary>
        {range
          ? t(`muhurtaSearch.summary.${search.status}`, {
              n: windows.length,
              from: formatJdDate(range.from_jd_ut, range.from_offset_secs),
              // Aralığın sonu son günün bitişidir: bir saniye öncesi o günün tarihini verir.
              to: formatJdDate(range.to_jd_ut === null ? null : range.to_jd_ut - 1 / 86_400, range.to_offset_secs),
            })
          : t("muhurtaSearch.searching")}
      </p>
      {search.warnings.includes("SunriseUnavailable") && (
        <p className="panel-notice muhurta-warning" role="note">
          {t("muhurtaSearch.noSunrise")}
        </p>
      )}
      {search.warnings.includes("LowPrecision") && (
        <p className="panel-notice" role="note">
          <strong>{t("badge.LowPrecision")}</strong>: {t("badgeHint.LowPrecision")}
        </p>
      )}
      {shown.length > 0 && (
        // Satırların çoğu hücresinde odaklanabilir denetim yoktur: kaydırma kabı klavye odağı alır
        // ve adını tablonun başlığından okur.
        <div className="bala-scroll muhurta-results" role="group" tabIndex={0} aria-labelledby={captionId}>
          <table className="module-table muhurta-table">
            <caption id={captionId}>{t("muhurtaSearch.results")}</caption>
            <thead>
              <tr>
                <th scope="col">{t("muhurtaSearch.col.start")}</th>
                <th scope="col">{t("muhurtaSearch.col.end")}</th>
                <th scope="col" className="num">
                  {t("muhurtaSearch.col.length")}
                </th>
                <th scope="col">{t("panchang.vara")}</th>
                <th scope="col">{t("panchang.tithi")}</th>
                <th scope="col">{t("panchang.nakshatra")}</th>
                <th scope="col">{t("panchang.yoga")}</th>
                <th scope="col">{t("panchang.karana")}</th>
                <th scope="col">{t("muhurtaSearch.col.lagna")}</th>
              </tr>
            </thead>
            <tbody>{shown.map(row)}</tbody>
          </table>
        </div>
      )}
      {windows.length > shown.length && <p className="panel-notice">{t("muhurtaSearch.capped", { shown: shown.length, n: windows.length })}</p>}
      <p className="panel-message">{t("muhurtaSearch.note")}</p>
      {search.method && <p className="panel-message">{t("muhurta.method", { method: search.method })}</p>}
    </>
  );
}
