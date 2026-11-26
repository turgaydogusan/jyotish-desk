// Dasa şeridi (UI spec §3.3, §3.4). Transit anının dasa yolu Rust'tan gelir (Vimshottari:
// `transits_at`, Plan 11 D10; chara dasha: `chara_dasha_at`); arayüz tarih karşılaştırmaz (C4).
// Seçilen seviyede anın üst döneminin alt dönemleri sıralanır; bir döneme tıklamak transit
// anını o dönemin başlangıcına taşır. `system` ayarı Vimshottari ile chara dasha arasında seçer
// (ileri modüller spec'i §6).
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { DashaPeriod } from "../bindings";
import { PanelError, usePanelLoadPhase } from "../panels/states";
import type { PanelProps } from "../panels/types";
import { useSession } from "../session/store";
import { useTransit } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { CharaDashaStrip } from "./CharaDashaStrip";
import "./dasha.css";
import { dashaSystem, PARENT_DEPTH, STRIP_LEVELS, stripLevel, SystemSelect, type StripLevel } from "./system";
import { childrenOf, formatJdDate, samePath } from "./tree";

export function DashaStripPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  const system = dashaSystem(panel.settings);
  const level = stripLevel(panel.settings);
  return (
    <div className="dasha-strip-panel">
      <div className="dasha-strip-toolbar">
        <SystemSelect value={system} onChange={(next) => setPanelSettings(panel.id, { system: next })} />
        <label htmlFor={`${id}-level`}>{t("dasha.strip.level")}</label>
        <select id={`${id}-level`} value={level} onChange={(e) => setPanelSettings(panel.id, { level: stripLevel({ level: e.target.value }) })}>
          {STRIP_LEVELS.map((l) => (
            <option key={l} value={l}>
              {t(`dasha.strip.levels.${l}`)}
            </option>
          ))}
        </select>
      </div>
      {system === "chara" ? <CharaDashaStrip opened={opened} level={level} /> : <Strip level={level} />}
    </div>
  );
}

function Strip({ level }: { level: StripLevel }) {
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
  if (transit.error) return <PanelError error={transit.error} onRetry={transit.retry} />;
  if (transit.view && periods.length === 0) return <p className="panel-message">{t("dasha.strip.outside")}</p>;
  if (periods.length === 0) return null;
  return (
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
  );
}
