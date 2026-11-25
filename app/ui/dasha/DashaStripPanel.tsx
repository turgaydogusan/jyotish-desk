// Dasa şeridi (UI spec §3.3, §3.4). Transit anının dasa yolu Rust'tan gelir (`transits_at`,
// Plan 11 D10); arayüz tarih karşılaştırmaz (C4). Seçilen seviyede anın üst döneminin alt
// dönemleri sıralanır; bir döneme tıklamak transit anını o dönemin başlangıcına taşır.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { DashaPeriod } from "../bindings";
import { PanelError, usePanelLoadPhase } from "../panels/states";
import type { PanelProps } from "../panels/types";
import { useSession } from "../session/store";
import { useTransit } from "../transit/store";
import { useLayoutStore } from "../view/store";
import "./dasha.css";
import { childrenOf, formatJdDate, samePath } from "./tree";

export const STRIP_LEVELS = ["maha", "antar", "pratyantar"] as const;
export type StripLevel = (typeof STRIP_LEVELS)[number];

export function stripLevel(settings: Record<string, unknown>): StripLevel {
  return (STRIP_LEVELS as readonly unknown[]).includes(settings.level) ? (settings.level as StripLevel) : "antar";
}

/** Seviyenin üst yol uzunluğu: maha için kök (0), antar için 1, pratyantar için 2. */
const PARENT_DEPTH: Record<StripLevel, number> = { maha: 0, antar: 1, pratyantar: 2 };

export function DashaStripPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Strip level={stripLevel(panel.settings)} id={id} onLevel={(level) => setPanelSettings(panel.id, { level })} />;
}

function Strip({ level, id, onLevel }: { level: StripLevel; id: string; onLevel(level: StripLevel): void }) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const transit = useTransit();
  usePanelLoadPhase(transit.loading && transit.view === null);
  if (!opened) return null;
  const depth = PARENT_DEPTH[level];
  const path = transit.view?.dasha_path ?? null;
  const offset = opened.chart_record.resolved.offset_secs;
  const periods: DashaPeriod[] = path && path.length > depth ? childrenOf(opened.chart.dasha.periods, path.slice(0, depth)) : [];
  const currentPath = path?.slice(0, depth + 1) ?? [];

  const name = (p: DashaPeriod) => p.lords.map((g) => t(`grahaName.${g}`)).join("–");
  return (
    <div className="dasha-strip-panel">
      <div className="dasha-strip-toolbar">
        <label htmlFor={`${id}-level`}>{t("dasha.strip.level")}</label>
        <select id={`${id}-level`} value={level} onChange={(e) => onLevel(e.target.value as StripLevel)}>
          {STRIP_LEVELS.map((l) => (
            <option key={l} value={l}>
              {t(`dasha.strip.levels.${l}`)}
            </option>
          ))}
        </select>
      </div>
      {transit.error ? (
        <PanelError error={transit.error} onRetry={transit.retry} />
      ) : transit.view && periods.length === 0 ? (
        <p className="panel-message">{t("dasha.strip.outside")}</p>
      ) : (
        periods.length > 0 && (
          <ol className="dasha-strip" aria-label={t("dasha.strip.label", { level: t(`dasha.strip.levels.${level}`) })}>
            {periods.map((p) => {
              const current = samePath(p.lords, currentPath);
              return (
                <li key={p.lords.join(",")}>
                  <button
                    type="button"
                    className={`dasha-strip-item${current ? " dasha-strip-item--current" : ""}`}
                    data-lord={p.lords[p.lords.length - 1]}
                    aria-current={current ? "true" : undefined}
                    onClick={() => transit.setAt({ Jd: p.start_jd_ut })}
                  >
                    <span className="dasha-strip-name">{name(p)}</span>
                    <span className="dasha-strip-date">{formatJdDate(p.start_jd_ut, offset)}</span>
                    {current && <span className="dasha-strip-now">{t("dasha.strip.current")}</span>}
                  </button>
                </li>
              );
            })}
          </ol>
        )
      )}
    </div>
  );
}
