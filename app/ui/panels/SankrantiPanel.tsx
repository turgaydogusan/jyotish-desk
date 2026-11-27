// Sankranti paneli (yol haritası Plan 26, §4.8): seçilen yılın güneş girişleri ve her giriş anının
// tithisi, varası ve Ay nakshatrası. Liste `sankranti_list`'ten gelir ve panel açıkken istenir
// (K1); burada hesap yoktur (C4). Girişler haritanın zodyak kipi ne olursa olsun sidereal
// burçlaradır; harita tropikal burç kullanıyorsa panel bunu sözle söyler. Yıl saklanmaz: boş
// yıl "bu yıl"dır ve Rust'ta okunur. Saatler seçilen yerin dilimindedir (varsayılan: bulunulan
// yer); satırdaki an transit tarihini o ana götürür. Liste olgudur: hüküm ve açıklama yoktur.
import { useId, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError, OpenedClient, SankrantiListView } from "../bindings";
import { signIndex } from "../chart/adapter";
import type { Panel } from "../layout/model";
import { useSankrantiList } from "../modules/useSankranti";
import { useSession } from "../session/store";
import { useTransitControl } from "../transit/store";
import { BalaScroll } from "./bala";
import { formatJdMinute } from "./format";
import { nakshatraName, tithiInPaksha, tithiName, VARA_NAMES } from "./labels";
import { PlaceControls, PlaceMissing, usePanelPlace } from "./MuhurtaPlace";
import "./panels.css";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Rust `TOOLS_YEAR_RANGE`: yıl kutusunun sınırları. */
export const YEAR_MIN = 1;
export const YEAR_MAX = 2999;

/** Yıl kutusunun metni → yıl; tam sayı değilse ya da aralık dışındaysa `null`. */
export function parseYear(text: string): number | null {
  if (!/^\d{1,4}$/.test(text.trim())) return null;
  const year = Number(text.trim());
  return year >= YEAR_MIN && year <= YEAR_MAX ? year : null;
}

export function SankrantiPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Body panel={panel} opened={opened} />;
}

/** `invalid_transit_input`'un alan adını arayüz dilinde yazar; öteki hatalar olduğu gibi durur. */
function fieldError(error: CommandError, name: (field: string) => string): CommandError {
  const field = error.params.field;
  if (error.code !== "invalid_transit_input" || typeof field !== "string") return error;
  return { ...error, params: { ...error.params, field: name(field) } };
}

function Body({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t, i18n } = useTranslation();
  const id = useId();
  const place = usePanelPlace(panel, opened);
  // `null`: bu yıl (Rust okur). Yıl oturumluktur; düzenle birlikte saklanmaz.
  const [year, setYear] = useState<number | null>(null);
  const list = useSankrantiList(opened, place.place, year, panel.id);
  const view = list.shown;
  const phase = usePanelLoadPhase(list.loading);
  const [text, setText] = useState("");
  const [editing, setEditing] = useState(false);
  // Gösterilen yıl değişince kutu onu gösterir; kullanıcı yazarken değişmez. Eşitleme çizim
  // sırasında yapılır (`TransitDate` ile aynı kalıp).
  const target = editing || view === null ? null : String(view.year);
  const [synced, setSynced] = useState(target);
  if (target !== synced) {
    setSynced(target);
    if (target !== null) setText(target);
  }
  const typed = parseYear(text);
  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (typed === null) return;
    setEditing(false);
    setYear(typed);
  };
  const shownError = list.error === null ? null : fieldError(list.error, (f) => (i18n.exists(`sankranti.field.${f}`) ? t(`sankranti.field.${f}`) : f));

  return (
    <div className="module-panel">
      <form className="muhurta-form" aria-label={t("sankranti.form")} onSubmit={submit}>
        <div className="module-toolbar">
          <PlaceControls panel={panel} opened={opened} at={place} />
        </div>
        <div className="module-toolbar">
          <label htmlFor={`${id}-year`}>{t("sankranti.year")}</label>
          <input
            id={`${id}-year`}
            type="number"
            inputMode="numeric"
            min={YEAR_MIN}
            max={YEAR_MAX}
            step={1}
            value={text}
            aria-describedby={`${id}-hint`}
            aria-invalid={editing && typed === null ? true : undefined}
            onChange={(e) => {
              setEditing(true);
              setText(e.target.value);
            }}
          />
          <button type="submit">{t("sankranti.show")}</button>
          <button
            type="button"
            aria-pressed={year === null}
            onClick={() => {
              setEditing(false);
              setYear(null);
            }}
          >
            {t("sankranti.thisYear")}
          </button>
          <span id={`${id}-hint`} className="chart-toolbar-hint">
            {t("sankranti.yearHint", { min: YEAR_MIN, max: YEAR_MAX })}
          </span>
        </div>
      </form>
      {opened.chart.meta.settings.zodiac !== "Sidereal" && (
        <p className="panel-notice" role="note">
          {t("sankranti.siderealNote")}
        </p>
      )}
      {place.place === null ? (
        <PlaceMissing choice={place.choice} />
      ) : shownError ? (
        <PanelError error={shownError} onRetry={list.retry} />
      ) : view === null ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : (
        <List view={view} />
      )}
    </div>
  );
}

function List({ view }: { view: SankrantiListView }) {
  const { t } = useTranslation();
  const transit = useTransitControl();
  const captionId = useId();
  return (
    <>
      {view.warnings.includes("LowPrecision") && (
        <p className="panel-notice" role="note">
          <strong>{t("badge.LowPrecision")}</strong>: {t("badgeHint.LowPrecision")}
        </p>
      )}
      {view.warnings.includes("SunriseUnavailable") && (
        <p className="panel-notice" role="note">
          <strong>{t("badge.SunriseUnavailable")}</strong>: {t("sankranti.noSunrise")}
        </p>
      )}
      <BalaScroll labelledBy={captionId}>
        <table className="module-table">
          <caption id={captionId}>{t("sankranti.caption", { year: view.year, n: view.rows.length })}</caption>
          <thead>
            <tr>
              <th scope="col">{t("sankranti.col.when")}</th>
              <th scope="col">{t("sankranti.col.sign")}</th>
              <th scope="col">{t("sankranti.col.tithi")}</th>
              <th scope="col">{t("sankranti.col.vara")}</th>
              <th scope="col">{t("sankranti.col.nakshatra")}</th>
            </tr>
          </thead>
          <tbody>
            {view.rows.map(({ sankranti: s, offset_secs }) => {
              const jd = s.jd_ut;
              const when = formatJdMinute(jd, offset_secs);
              return (
                <tr key={s.sign} data-sign={s.sign}>
                  <th scope="row">
                    {jd === null ? (
                      when
                    ) : (
                      <button type="button" className="link-button" aria-label={t("sankranti.goTo", { when })} onClick={() => transit.setAt({ Jd: jd })}>
                        {when}
                      </button>
                    )}
                  </th>
                  <td>{t(`sign.${signIndex(s.sign)}`)}</td>
                  <td>
                    {s.paksha} {tithiInPaksha(s.tithi)} · {tithiName(s.tithi)}
                  </td>
                  <td>
                    {VARA_NAMES[s.vara]} · {t(`panchang.weekday.${s.vara}`)}
                  </td>
                  <td>
                    {nakshatraName(s.nakshatra)} · {t("panchang.pada", { pada: s.pada })}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </BalaScroll>
      <p className="panel-message">{t("sankranti.method", { method: view.method })}</p>
    </>
  );
}
