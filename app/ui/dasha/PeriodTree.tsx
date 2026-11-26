// Dönem ağacı (WAI-ARIA ağaç deseni): Vimshottari ve chara dasha ağaçlarının ortak çizimi ve
// klavyesi. Düğümleri panel kurar (dönemler Rust'tan gelir; burada hesap yoktur); bu bileşen
// yalnızca çizer, odağı taşır ve açma/kapama isteğini düğümün `toggle`'ına iletir.
//
// Anahtar virgülle ayrılmış yoldur: üst düğümün anahtarı son parça atılınca çıkar
// (`"Rahu,Jupiter"` → `"Rahu"`; `"2:Aries,Taurus"` → `"2:Aries"`).
import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError } from "../bindings";
import { PanelError } from "../panels/states";

export interface PeriodNode {
  key: string;
  /** 1 tabanlı seviye (`aria-level`). */
  level: number;
  /** Satırda görünen ad: dönemin son lordu ya da burcu. */
  name: string;
  /** Erişilebilir ad: kökten yol ve tarihler. */
  label: string;
  dates: string;
  expandable: boolean;
  open: boolean;
  /** Açık düğümün alt seviyesi. */
  kids?: PeriodLevel;
  toggle(): void;
}

export type PeriodLevel = { status: "ok"; nodes: PeriodNode[] } | { status: "loading" } | { status: "error"; error: CommandError; retry(): void };

const parentKey = (key: string) => key.split(",").slice(0, -1).join(",");

export function PeriodTree({ label, roots }: { label: string; roots: PeriodLevel }) {
  const { t } = useTranslation();
  const [focusKey, setFocusKey] = useState<string | null>(null);
  const items = useRef(new Map<string, HTMLLIElement>());
  const treeRef = useRef<HTMLUListElement>(null);

  // Odak klavyeyle taşındıysa yeni öğeye geçer (fare ve ilk çizim odağı çalmaz).
  useEffect(() => {
    if (focusKey === null || !treeRef.current?.contains(document.activeElement)) return;
    items.current.get(focusKey)?.focus();
  }, [focusKey]);

  // Görünür öğeler, ekrandaki sırayla (klavye gezintisi için).
  const visible: PeriodNode[] = [];
  const walk = (level: PeriodLevel | undefined) => {
    if (level?.status !== "ok") return;
    for (const node of level.nodes) {
      visible.push(node);
      if (node.open) walk(node.kids);
    }
  };
  walk(roots);
  // Sekme durağı her zaman görünür bir öğedir: odaklı öğe ağaçtan çıktıysa (başka harita, kapanan
  // yol, silinen tembel seviye) görünür en yakın atası, o da yoksa ilk öğe.
  const visibleKeys = new Set(visible.map((v) => v.key));
  let near = focusKey ?? "";
  while (near !== "" && !visibleKeys.has(near)) near = parentKey(near);
  const current = near !== "" ? near : (visible[0]?.key ?? null);

  const onKeyDown = (e: KeyboardEvent) => {
    // Ağaç tuşları yalnızca öğelerindir; öğenin içindeki düğme ("Yeniden dene") kendi tuşlarını alır.
    if ((e.target as Element).getAttribute("role") !== "treeitem") return;
    const i = visible.findIndex((v) => v.key === current);
    if (i < 0) return;
    const v = visible[i];
    const go = (j: number) => visible[j] && setFocusKey(visible[j].key);
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
        if (v.expandable && !v.open) v.toggle();
        else if (v.open && visible[i + 1]?.level === v.level + 1) go(i + 1);
        break;
      case "ArrowLeft":
        if (v.open) v.toggle();
        else if (v.level > 1) setFocusKey(parentKey(v.key));
        break;
      case "Enter":
      case " ":
        if (v.expandable) v.toggle();
        break;
      default:
        return;
    }
    e.preventDefault();
  };

  const renderLevel = (level: PeriodLevel) => {
    if (level.status === "loading")
      return (
        <li role="none" className="dasha-pending">
          {t("panel.loading")}
        </li>
      );
    if (level.status === "error")
      return (
        <li role="none">
          <PanelError error={level.error} onRetry={level.retry} />
        </li>
      );
    return level.nodes.map((node, i) => (
      <li
        key={node.key}
        ref={(el) => {
          if (el) items.current.set(node.key, el);
          else items.current.delete(node.key);
        }}
        role="treeitem"
        aria-level={node.level}
        aria-setsize={level.nodes.length}
        aria-posinset={i + 1}
        aria-expanded={node.expandable ? node.open : undefined}
        aria-label={node.label}
        tabIndex={node.key === current ? 0 : -1}
        className="dasha-item"
        onFocus={(e) => e.target === e.currentTarget && setFocusKey(node.key)}
      >
        {/* Tıklama satırdadır: iç içe öğelerin (ve hata düğmesinin) tıklaması üst düğümü kapatmaz. */}
        <span
          className="dasha-row"
          style={{ paddingInlineStart: `${(node.level - 1) * 14}px` }}
          onClick={() => {
            setFocusKey(node.key);
            if (node.expandable) node.toggle();
          }}
        >
          <span className="dasha-twisty" aria-hidden="true">
            {node.expandable ? (node.open ? "▾" : "▸") : ""}
          </span>
          <span className="dasha-lord">{node.name}</span>
          <span className="dasha-dates">{node.dates}</span>
        </span>
        {node.open && node.kids && <ul role="group">{renderLevel(node.kids)}</ul>}
      </li>
    ));
  };

  return (
    <ul ref={treeRef} role="tree" aria-label={label} className="dasha-tree" onKeyDown={onKeyDown}>
      {renderLevel(roots)}
    </ul>
  );
}
