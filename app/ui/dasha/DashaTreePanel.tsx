import { useTranslation } from "react-i18next";
import type { DashaPeriod, Graha, OpenedClient } from "../bindings";
import { timeUnknown } from "../session/birth";
import { usePanelLoadPhase } from "../panels/states";
import type { PanelProps } from "../panels/types";
import { useSession } from "../session/store";
import { useLayoutStore, useViewStore } from "../view/store";
import { CharaDashaTree } from "./CharaDashaTree";
import "./dasha.css";
import { PeriodTree, type PeriodLevel } from "./PeriodTree";
import { dashaSystem, SystemSelect } from "./system";
import { childrenOf, formatJdDate, isOpen, MAX_DASHA_LEVEL, PRELOADED_LEVEL } from "./tree";
import { pathKey, useLazyDasha, type LazyEntry } from "./useLazyDasha";

/**
 * Dasha ağacı (spec §3.3). `system` ayarı Vimshottari ile chara dasha arasında seçer (ileri
 * modüller spec'i §6). Tek açık yol: bir düğümü açmak kardeşlerini kapatır; yol haritanın
 * `ViewState`'indedir (sistem başına ayrı) ve haritayla kaydedilir. Klavye: WAI-ARIA ağaç deseni
 * (`PeriodTree`).
 */
export function DashaTreePanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { setPanelSettings } = useLayoutStore();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  const system = dashaSystem(panel.settings);
  return (
    <div className="dasha-panel">
      <div className="module-toolbar">
        <SystemSelect value={system} onChange={(next) => setPanelSettings(panel.id, { system: next })} />
      </div>
      {system === "chara" ? <CharaDashaTree opened={opened} /> : <VimshottariTree opened={opened} />}
    </div>
  );
}

/**
 * Vimshottari ağacı: 1–3. seviye haritayla gelir, 4–5. seviye istenerek yüklenir. Açık yol
 * `ViewState.dasha_expanded_path`'tir.
 */
function VimshottariTree({ opened }: { opened: OpenedClient }) {
  const { t } = useTranslation();
  const { model, setDashaPath } = useViewStore();
  const open = model.dashaPath;
  const lazy = useLazyDasha(opened.chart_record.id, opened.chart.dasha.seed, open);
  usePanelLoadPhase(lazy.loading);
  const { periods } = opened.chart.dasha;
  const offset = opened.chart_record.resolved.offset_secs;

  // Yüklenen liste de `childrenOf` süzgecinden geçer: yalnızca bu yolun bir alt seviyesi
  // kabul edilir (yanlış bir yanıt ağacı sonsuz döngüye sokamaz). Boş yanıt hatadır (M6-6):
  // her dönemin dokuz alt dönemi vardır, boş liste sessizce boş dal olarak gösterilmez.
  const childList = (path: readonly Graha[]): DashaPeriod[] | LazyEntry | undefined => {
    if (path.length < PRELOADED_LEVEL) return childrenOf(periods, path);
    const entry = lazy.entries[pathKey(path)];
    if (entry?.status !== "ok") return entry;
    const kids = childrenOf(entry.periods, path);
    return kids.length > 0 ? { status: "ok", periods: kids } : { status: "error", error: { code: "dasha_empty", params: {} } };
  };

  const dates = (p: DashaPeriod) => `${formatJdDate(p.start_jd_ut, offset)} – ${formatJdDate(p.end_jd_ut, offset)}`;
  const toggle = (p: DashaPeriod) => setDashaPath(isOpen(open, p.lords) ? p.lords.slice(0, -1) : p.lords);

  const level = (path: readonly Graha[]): PeriodLevel => {
    const list = childList(path);
    if (list === undefined) return { status: "loading" };
    if (!Array.isArray(list) && list.status === "error") return { status: "error", error: list.error, retry: () => lazy.retry(path) };
    const loaded = Array.isArray(list) ? list : list.periods;
    return {
      status: "ok",
      nodes: loaded.map((p) => {
        const expandable = p.has_children && p.level < MAX_DASHA_LEVEL;
        const o = expandable && isOpen(open, p.lords);
        return {
          key: pathKey(p.lords),
          level: p.level,
          name: t(`grahaName.${p.lords[p.lords.length - 1]}`),
          label: `${p.lords.map((g) => t(`grahaName.${g}`)).join("–")}, ${dates(p)}`,
          dates: dates(p),
          expandable,
          open: o,
          kids: o ? level(p.lords) : undefined,
          toggle: () => toggle(p),
        };
      }),
    };
  };

  return (
    <>
      {timeUnknown(opened) && (
        <p className="panel-notice" role="note">
          {t("dasha.approx")}
        </p>
      )}
      <PeriodTree label={t("panel.dasha_tree")} roots={level([])} />
    </>
  );
}
