// Doğum bilgisi paneli (spec §3.3): açık haritanın saklanan doğum verisi, Rust'ın çözdüğü an
// ve uyarılar. Uyarılar ad ve açıklamasıyla düz metindir (rozetin `title`'ı klavyeyle okunmaz).
// Komut çağırmaz; değerler `session.opened`'dan gelir.
import type { TFunction } from "i18next";
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { BirthData, OpenedClient, TzChoice } from "../bindings";
import { formatFixedOffset } from "../birth/model";
import { useSession } from "../session/store";
import "./panels.css";
import type { PanelProps } from "./types";

function tzText(t: TFunction, tz: TzChoice): string {
  if (tz === "Lmt") return t("birth.tzLmt");
  if ("Iana" in tz && tz.Iana !== undefined) return tz.Iana;
  return formatFixedOffset(tz.Fixed ?? 0);
}

/** "17:30:00" → "17:30"; sıfırdan farklı saniye ve kesir korunur. */
function timeText(t: TFunction, b: BirthData): string {
  if (b.time_certainty === "Unknown" || b.time === null) return t("birth.certaintyUnknown");
  return b.time.replace(/^(\d{2}:\d{2}):00(\.0+)?$/, "$1");
}

function placeText(t: TFunction, b: BirthData): string {
  const parts = [b.place.name, b.place.admin1, b.place.country].filter((s) => s.trim() !== "");
  return parts.length > 0 ? parts.join(", ") : t("birthInfo.manualPlace");
}

function coordText(b: BirthData): string {
  const { lat, lon } = b.place;
  return lat === null || lon === null ? "—" : `${lat.toFixed(4)}, ${lon.toFixed(4)}`;
}

export function BirthInfoPanel(_props: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <BirthInfo opened={opened} />;
}

function BirthInfo({ opened }: { opened: OpenedClient }) {
  const { t } = useTranslation();
  const id = useId();
  const b = opened.chart_record.birth;
  const r = opened.chart_record.resolved;
  const warnings: string[] = [...r.warnings, ...opened.chart.meta.warnings];
  const rows: [string, string][] = [
    [t("birthInfo.date"), b.date],
    [t("birthInfo.time"), timeText(t, b)],
    [t("birthInfo.certainty"), t(`birth.certainty${b.time_certainty}`)],
    [t("birthInfo.place"), placeText(t, b)],
    [t("birthInfo.coords"), coordText(b)],
    [t("birthInfo.tz"), tzText(t, b.tz)],
    [t("birthInfo.utc"), r.utc.replace("T", " ").replace("Z", " UTC")],
    [t("birthInfo.offset"), `${r.abbrev} ${formatFixedOffset(r.offset_secs)}`.trim()],
    [t("birthInfo.tzdb"), r.tzdb_version],
    [t("gender.label"), opened.client.gender ? t(`gender.${opened.client.gender}`) : t("gender.none")],
    [t("birthInfo.kind"), t(`chart.kind.${opened.chart_record.kind}`)],
  ];
  if (b.notes.trim() !== "") rows.push([t("birthInfo.source"), b.notes]);
  return (
    <section className="birth-info" aria-label={t("panel.birth_info")}>
      <dl className="birth-info-list">
        {rows.map(([term, value]) => (
          <div key={term} className="birth-info-row">
            <dt>{term}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
      <h3 id={`${id}-warnings`} className="birth-info-heading">
        {t("birthInfo.warnings")}
      </h3>
      {warnings.length === 0 ? (
        <p className="panel-message">{t("birthInfo.noWarnings")}</p>
      ) : (
        <ul className="birth-info-warnings" aria-labelledby={`${id}-warnings`}>
          {warnings.map((w) => (
            <li key={w}>
              <strong>{t(`badge.${w}`)}</strong>: {t(`badgeHint.${w}`)}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
