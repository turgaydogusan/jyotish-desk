// Olay araması paneli (ileri modüller spec'i §12.2): bir tarih aralığındaki gök olayları. Ayarlar
// (olay türleri, grahalar, yer) panelin ayarıdır ve düzenle birlikte saklanır; tarih aralığı
// saklanmaz. Olaylar `event_search`'ten gelir; burada hesap yoktur (C4). Saatler seçilen yerin
// dilimindedir (varsayılan: açık haritanın yeri). Satırdaki an, transit tarihini o ana götürür.
// Olay bir olgudur: "iyi" ya da "kötü" hükmü ve açıklama metni yoktur. Retro düz `R`'dir.
import { useEffect, useId, useLayoutEffect, useRef, useState, type FormEvent, type RefObject } from "react";
import { useTranslation } from "react-i18next";
import type { Graha, OpenedClient, SkyEventKind, SkyEventView } from "../bindings";
import { signIndex } from "../chart/adapter";
import { GRAHA_IDS } from "../chart/types";
import type { Panel } from "../layout/model";
import { chartInput } from "../modules/input";
import { muhurtaPlace } from "../modules/muhurta";
import { useEventSearch, type EventSearch } from "../modules/useEventSearch";
import { CheckSet } from "../search/CheckSet";
import { EVENT_KINDS, MAX_EVENTS, toggle } from "../search/model";
import "../search/search.css";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useTransitControl } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { formatJdClock, formatJdDate } from "./format";
import { nakshatraName } from "./labels";
import { PlaceControls, PlaceMissing, usePanelPlace } from "./MuhurtaPlace";
import "./panels.css";
import { PanelError, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Listede çizilen en çok olay; fazlası sayılır ve seçimin daraltılması önerilir. */
export const MAX_SHOWN_EVENTS = 500;
/** Yeni panelin seçimi: yavaş grahaların burç girişleri ve dönüşleri. */
export const DEFAULT_EVENT_KINDS: readonly SkyEventKind[] = ["SignIngress", "StationRetrograde", "StationDirect"];
export const DEFAULT_EVENT_GRAHAS: readonly Graha[] = ["Jupiter", "Saturn", "Rahu"];
const NATAL_KINDS: readonly SkyEventKind[] = ["TransitToNatal", "HouseFromNatalMoon"];
const GRAHA_FREE: readonly SkyEventKind[] = ["NewMoon", "FullMoon", "HouseFromNatalMoon"];

/** Panel ayarındaki kümenin toleranslı okuması: tanınmayan değerler atılır; anahtar yoksa varsayılan. */
export function parseSet<T>(raw: unknown, allowed: readonly T[], fallback: readonly T[]): T[] {
  if (!Array.isArray(raw)) return [...fallback];
  return allowed.filter((a) => raw.includes(a));
}

export function EventSearchPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Search panel={panel} opened={opened} />;
}

function Search({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  // Saatler varsayılan olarak açık haritanın yerinin dilimindedir.
  const placed: Panel = { ...panel, settings: { place: "chart", ...panel.settings } };
  const place = usePanelPlace(placed, opened);
  const kinds = parseSet(panel.settings.kinds, EVENT_KINDS, DEFAULT_EVENT_KINDS);
  const grahas = parseSet(panel.settings.grahas, GRAHA_IDS, DEFAULT_EVENT_GRAHAS);
  const search = useEventSearch(String(opened.chart_record.id));
  const running = search.status === "running";
  const phase = usePanelLoadPhase(running);
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const runButton = useRef<HTMLButtonElement>(null);

  const usesGrahas = kinds.some((k) => !GRAHA_FREE.includes(k));
  const natal = kinds.some((k) => NATAL_KINDS.includes(k));
  const why = kinds.length === 0 ? "kinds" : usesGrahas && grahas.length === 0 ? "grahas" : kinds.includes("Conjunction") && grahas.length < 2 ? "pair" : null;
  const ready = place.place !== null && why === null;
  // Liste, başlatıldığı girdiye aittir: girdi arama sürerken değişirse arama durdurulur.
  const key = JSON.stringify([place.place === null ? null : muhurtaPlace(place.place), from, to, kinds, grahas]);
  const [searchedKey, setSearchedKey] = useState<string | null>(null);
  const stale = search.status !== "idle" && searchedKey !== null && searchedKey !== key;
  const { stop } = search;
  useEffect(() => {
    if (running && stale) stop();
  }, [running, stale, stop]);

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (place.place === null || why !== null || running) return;
    setSearchedKey(key);
    search.start({
      from_date: from === "" ? null : from,
      to_date: to === "" ? null : to,
      place: muhurtaPlace(place.place),
      kinds,
      grahas,
      settings: opened.chart.meta.settings,
      // Doğum verisi yalnızca açık harita isteyen türler seçiliyse gönderilir.
      natal: natal ? chartInput(opened) : null,
    });
  };

  return (
    <div className="module-panel event-search">
      <form className="muhurta-form" aria-label={t("eventSearch.form")} onSubmit={submit}>
        <div className="module-toolbar">
          <PlaceControls panel={placed} opened={opened} at={place} />
        </div>
        <div className="module-toolbar">
          <label htmlFor={`${id}-from`}>{t("eventSearch.from")}</label>
          <input id={`${id}-from`} type="date" value={from} aria-describedby={`${id}-range`} onChange={(e) => setFrom(e.target.value)} />
          <label htmlFor={`${id}-to`}>{t("eventSearch.to")}</label>
          <input id={`${id}-to`} type="date" value={to} aria-describedby={`${id}-range`} onChange={(e) => setTo(e.target.value)} />
          <span id={`${id}-range`} className="chart-toolbar-hint">
            {t("eventSearch.rangeHint")}
          </span>
        </div>
        <CheckSet
          legend={t("eventSearch.kinds")}
          options={EVENT_KINDS.map((k) => ({ value: k, label: t(`eventSearch.kind.${k}`) }))}
          selected={kinds}
          onToggle={(k) => setPanelSettings(panel.id, { kinds: toggle(kinds, k, EVENT_KINDS) })}
        />
        <CheckSet
          legend={t("eventSearch.grahas")}
          options={GRAHA_IDS.map((g) => ({ value: g, label: t(`grahaName.${g}`) }))}
          selected={grahas}
          onToggle={(g) => setPanelSettings(panel.id, { grahas: toggle(grahas, g, GRAHA_IDS) })}
        />
        {natal && timeUnknown(opened) && (
          <p className="panel-notice" role="note">
            {t("eventSearch.natalApprox")}
          </p>
        )}
        <div className="module-toolbar">
          {/* Kullanılamazken yerel `disabled` değildir: odaklanır ve nedeni görünür bir satırdır. */}
          <button ref={runButton} type="submit" aria-disabled={!ready || running ? true : undefined} aria-describedby={ready ? undefined : `${id}-why`}>
            {t("eventSearch.run")}
          </button>
          {running && <StopButton onStop={search.stop} fallback={runButton} />}
          <span className="muhurta-progress" role="status">
            {running && phase !== "none" ? t("eventSearch.searching") : ""}
          </span>
        </div>
        {!ready && <div id={`${id}-why`}>{why !== null ? <p className="chart-toolbar-hint">{t(`eventSearch.why.${why}`)}</p> : <PlaceMissing choice={place.choice} />}</div>}
      </form>
      {search.error && (
        <PanelError
          error={search.error}
          onRetry={() => {
            // Hata kutusu yeni arama başlayınca gider: odak önce "Ara"ya taşınır.
            runButton.current?.focus();
            runButton.current?.click();
          }}
        />
      )}
      <Results search={search} stale={stale} />
    </div>
  );
}

/** "Durdur": arama bitince DOM'dan gider; odak ondaysa "Ara" düğmesine taşınır. */
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
      {t("eventSearch.stop")}
    </button>
  );
}

function Results({ search, stale }: { search: EventSearch; stale: boolean }) {
  const { t } = useTranslation();
  const transit = useTransitControl();
  const captionId = useId();
  if (search.status === "idle") return <p className="panel-message">{t("eventSearch.idle")}</p>;
  if (search.status === "stopped") return <p className="panel-message">{t("eventSearch.stopped")}</p>;
  const view = search.view;
  if (view === null) return null;
  const shown = view.events.slice(0, MAX_SHOWN_EVENTS);
  const name = (g: Graha) => t(`grahaName.${g}`);
  const label = ({ event: e }: SkyEventView): string => {
    const [a, b] = e.grahas;
    switch (e.kind) {
      case "Conjunction":
        return t("eventSearch.event.Conjunction", { a: name(a), b: name(b) });
      case "TransitToNatal":
        return a === b ? t("eventSearch.event.return", { graha: name(a) }) : t("eventSearch.event.TransitToNatal", { a: name(a), b: name(b) });
      case "HouseFromNatalMoon":
        return t("eventSearch.event.HouseFromNatalMoon", { graha: name(a), house: e.house ?? "—" });
      case "NewMoon":
      case "FullMoon":
        return t(`eventSearch.event.${e.kind}`);
      default:
        return t(`eventSearch.event.${e.kind}`, { graha: name(a) });
    }
  };
  const where = ({ event: e }: SkyEventView) => [e.sign === null ? null : t(`sign.${signIndex(e.sign)}`), e.nakshatra === null ? null : nakshatraName(e.nakshatra)].filter((x) => x !== null).join(" · ");
  const day = (jd: number | null, offset: number) => formatJdDate(jd, offset);
  return (
    <>
      <p className="panel-message" data-summary>
        {t(view.events.length === 0 ? "eventSearch.none" : "eventSearch.summary", {
          n: view.events.length,
          from: day(view.from_jd_ut, view.from_offset_secs),
          // Aralığın sonu son günün bitişidir: bir saniye öncesi o günün tarihini verir.
          to: day(view.to_jd_ut === null ? null : view.to_jd_ut - 1 / 86_400, view.to_offset_secs),
        })}
      </p>
      {stale && (
        <p className="panel-notice" role="note">
          {t("eventSearch.stale")}
        </p>
      )}
      {view.truncated && (
        <p className="panel-notice" role="note">
          {t("eventSearch.truncated", { max: MAX_EVENTS, until: day(view.scanned_to_jd_ut, view.to_offset_secs) })}
        </p>
      )}
      {shown.length > 0 && (
        <div className="bala-scroll muhurta-results" role="group" tabIndex={0} aria-labelledby={captionId}>
          <table className="module-table">
            <caption id={captionId}>{t("eventSearch.results")}</caption>
            <thead>
              <tr>
                <th scope="col">{t("eventSearch.col.when")}</th>
                <th scope="col">{t("eventSearch.col.event")}</th>
                <th scope="col">{t("eventSearch.col.where")}</th>
              </tr>
            </thead>
            <tbody>
              {shown.map((row, n) => {
                const jd = row.event.jd_ut;
                const when = `${day(jd, row.offset_secs)} ${formatJdClock(jd, row.offset_secs, jd)}`;
                return (
                  <tr key={`${n}:${jd}`} data-event={row.event.kind}>
                    <th scope="row">
                      {jd === null ? (
                        when
                      ) : (
                        <button type="button" className="link-button" aria-label={t("eventSearch.goTo", { when })} onClick={() => transit.setAt({ Jd: jd })}>
                          {when}
                        </button>
                      )}
                    </th>
                    <td>{label(row)}</td>
                    <td>{where(row)}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
      {view.events.length > shown.length && <p className="panel-notice">{t("eventSearch.capped", { shown: shown.length, n: view.events.length })}</p>}
      <p className="panel-message">{t("eventSearch.method", { method: view.method })}</p>
    </>
  );
}
