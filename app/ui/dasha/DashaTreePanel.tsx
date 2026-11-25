import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import type { DashaPeriod, Graha } from "../bindings";
import { timeUnknown } from "../panels/ChartPanel";
import { PanelError, usePanelLoadPhase } from "../panels/states";
import type { PanelProps } from "../panels/types";
import { useSession } from "../session/store";
import { useViewStore } from "../view/store";
import "./dasha.css";
import { childrenOf, formatJdDate, isOpen, MAX_DASHA_LEVEL, PRELOADED_LEVEL } from "./tree";
import { pathKey, useLazyDasha, type LazyEntry } from "./useLazyDasha";

interface Visible {
  period: DashaPeriod;
  expandable: boolean;
  open: boolean;
}

/**
 * Dasha ağacı (spec §3.3, Vimshottari). Tek açık yol: bir düğümü açmak kardeşlerini kapatır;
 * yol haritanın `ViewState.dasha_expanded_path`'idir ve haritayla kaydedilir. 1–3. seviye
 * haritayla gelir, 4–5. seviye istenerek yüklenir. Klavye: WAI-ARIA ağaç deseni.
 */
export function DashaTreePanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { model, setDashaPath } = useViewStore();
  const open = model.dashaPath;
  const chartId = opened?.chart_record.id ?? 0;
  const seed = opened?.chart.dasha.seed ?? { birth_jd_ut: null, moon_sidereal_lon: null, year_days: null };
  const lazy = useLazyDasha(chartId, seed, opened ? open : []);
  usePanelLoadPhase(opened !== null && lazy.loading);
  const [focusKey, setFocusKey] = useState<string | null>(null);
  const items = useRef(new Map<string, HTMLLIElement>());
  const treeRef = useRef<HTMLUListElement>(null);

  // Odak klavyeyle taşındıysa yeni öğeye geçer (fare ve ilk çizim odağı çalmaz).
  useEffect(() => {
    if (focusKey === null || !treeRef.current?.contains(document.activeElement)) return;
    items.current.get(focusKey)?.focus();
  }, [focusKey]);

  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  const { periods } = opened.chart.dasha;
  const offset = opened.chart_record.resolved.offset_secs;

  // Yüklenen liste de `childrenOf` süzgecinden geçer: yalnızca bu yolun bir alt seviyesi
  // kabul edilir (yanlış bir yanıt ağacı sonsuz döngüye sokamaz).
  const childList = (path: readonly Graha[]): DashaPeriod[] | LazyEntry | undefined => {
    if (path.length < PRELOADED_LEVEL) return childrenOf(periods, path);
    const entry = lazy.entries[pathKey(path)];
    return entry?.status === "ok" ? { status: "ok", periods: childrenOf(entry.periods, path) } : entry;
  };

  /** Yüklenmiş alt dönemler; yükleniyorsa ya da hatalıysa `null`. */
  const loaded = (path: readonly Graha[]): readonly DashaPeriod[] | null => {
    const list = childList(path);
    if (Array.isArray(list)) return list;
    return list?.status === "ok" ? list.periods : null;
  };

  // Görünür öğeler, ekrandaki sırayla (klavye gezintisi için).
  const visible: Visible[] = [];
  const walk = (path: readonly Graha[]) => {
    for (const p of loaded(path) ?? []) {
      const expandable = p.has_children && p.level < MAX_DASHA_LEVEL;
      const o = expandable && isOpen(open, p.lords);
      visible.push({ period: p, expandable, open: o });
      if (o) walk(p.lords);
    }
  };
  walk([]);
  // Sekme durağı her zaman görünür bir öğedir: odaklı öğe ağaçtan çıktıysa (başka harita, kapanan
  // yol, silinen tembel seviye) görünür en yakın atası, o da yoksa ilk öğe.
  const visibleKeys = new Set(visible.map((v) => pathKey(v.period.lords)));
  const focusPath = focusKey === null ? [] : focusKey.split(",");
  while (focusPath.length > 0 && !visibleKeys.has(focusPath.join(","))) focusPath.pop();
  const current = focusPath.length > 0 ? focusPath.join(",") : visible[0] ? pathKey(visible[0].period.lords) : null;

  const toggle = (p: DashaPeriod) => setDashaPath(isOpen(open, p.lords) ? p.lords.slice(0, -1) : p.lords);

  const onKeyDown = (e: KeyboardEvent) => {
    // Ağaç tuşları yalnızca öğelerindir; öğenin içindeki düğme ("Yeniden dene") kendi tuşlarını alır.
    if ((e.target as Element).getAttribute("role") !== "treeitem") return;
    const i = visible.findIndex((v) => pathKey(v.period.lords) === current);
    if (i < 0) return;
    const v = visible[i];
    const go = (j: number) => visible[j] && setFocusKey(pathKey(visible[j].period.lords));
    switch (e.key) {
      case "ArrowDown":
        go(i + 1);
        break;
      case "ArrowUp":
        go(i - 1);
        break;
      case "Home":
        go(0);
        break;
      case "End":
        go(visible.length - 1);
        break;
      case "ArrowRight":
        if (v.expandable && !v.open) toggle(v.period);
        else if (v.open && visible[i + 1]?.period.level === v.period.level + 1) go(i + 1);
        break;
      case "ArrowLeft":
        if (v.open) toggle(v.period);
        else if (v.period.level > 1) setFocusKey(pathKey(v.period.lords.slice(0, -1)));
        break;
      case "Enter":
      case " ":
        if (v.expandable) toggle(v.period);
        break;
      default:
        return;
    }
    e.preventDefault();
  };

  const label = (p: DashaPeriod) =>
    `${p.lords.map((g) => t(`grahaName.${g}`)).join("–")}, ${formatJdDate(p.start_jd_ut, offset)} – ${formatJdDate(p.end_jd_ut, offset)}`;

  const renderLevel = (path: readonly Graha[]) => {
    const list = childList(path);
    if (list === undefined)
      return (
        <li role="none" className="dasha-pending">
          {t("panel.loading")}
        </li>
      );
    if (!Array.isArray(list)) {
      if (list.status === "error")
        return (
          <li role="none">
            <PanelError error={list.error} onRetry={() => lazy.retry(path)} />
          </li>
        );
      return renderItems(list.periods);
    }
    return renderItems(list);
  };

  const renderItems = (list: readonly DashaPeriod[]) =>
    list.map((p, i) => {
      const key = pathKey(p.lords);
      const expandable = p.has_children && p.level < MAX_DASHA_LEVEL;
      const o = expandable && isOpen(open, p.lords);
      return (
        <li
          key={key}
          ref={(el) => {
            if (el) items.current.set(key, el);
            else items.current.delete(key);
          }}
          role="treeitem"
          aria-level={p.level}
          aria-setsize={list.length}
          aria-posinset={i + 1}
          aria-expanded={expandable ? o : undefined}
          aria-label={label(p)}
          tabIndex={key === current ? 0 : -1}
          className="dasha-item"
          onFocus={(e) => e.target === e.currentTarget && setFocusKey(key)}
        >
          {/* Tıklama satırdadır: iç içe öğelerin (ve hata düğmesinin) tıklaması üst düğümü kapatmaz. */}
          <span
            className="dasha-row"
            style={{ paddingInlineStart: `${(p.level - 1) * 14}px` }}
            onClick={() => {
              setFocusKey(key);
              if (expandable) toggle(p);
            }}
          >
            <span className="dasha-twisty" aria-hidden="true">
              {expandable ? (o ? "▾" : "▸") : ""}
            </span>
            <span className="dasha-lord">{t(`grahaName.${p.lords[p.lords.length - 1]}`)}</span>
            <span className="dasha-dates">
              {formatJdDate(p.start_jd_ut, offset)} – {formatJdDate(p.end_jd_ut, offset)}
            </span>
          </span>
          {o && <ul role="group">{renderLevel(p.lords)}</ul>}
        </li>
      );
    });

  return (
    <div className="dasha-panel">
      {timeUnknown(opened) && (
        <p className="panel-notice" role="note">
          {t("dasha.approx")}
        </p>
      )}
      <ul ref={treeRef} role="tree" aria-label={t("panel.dasha_tree")} className="dasha-tree" onKeyDown={onKeyDown}>
        {renderLevel([])}
      </ul>
    </div>
  );
}
