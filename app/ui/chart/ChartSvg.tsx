// Kuzey Hint, Güney Hint ve daire haritası. Yalnızca belirteç sınıfları kullanılır;
// gezegenlere renk verilmez. Retro "℞", yanma "c", vargottama altı çizili (spec §2.3).
// svg `role="group"`dur: `img` alt öğeleri sunumsal yapar ve ev/gezegen düğmelerini
// ekran okuyucudan gizlerdi (spec §6).
import { useMemo, type KeyboardEvent } from "react";
import { sameTarget, type HighlightTarget } from "../highlight/target";
import "./chart.css";
import { chartCells, layoutLabels, pathOf, VIEWBOX, type ChartCell, type LabelLayout } from "./geometry";
import type { ChartData, ChartLabels, ChartPlanet, ChartStyle } from "./types";

export interface ChartSvgProps {
  data: ChartData;
  style: ChartStyle;
  labels: ChartLabels;
  highlight?: HighlightTarget | null;
  onHover?: (target: HighlightTarget | null) => void;
  onPin?: (target: HighlightTarget) => void;
}

function cellHighlighted(cell: ChartCell, h: HighlightTarget | null): boolean {
  if (h === null) return false;
  return (h.type === "house" && h.house === cell.house) || (h.type === "sign" && h.sign === cell.sign);
}

/** Çizilebilir gezegen: burç 0…11 tamsayı, derece sonlu. Diğerleri açıkça atlanır. */
function drawable(p: ChartPlanet): boolean {
  return Number.isInteger(p.sign) && p.sign >= 0 && p.sign <= 11 && Number.isFinite(p.degInSign);
}

function labelText(p: ChartPlanet, graha: ChartLabels["graha"]): string {
  return `${graha[p.id]}${p.retrograde ? "℞" : ""}${p.combust ? "c" : ""}`;
}

/** Erişilebilir ad sözcüklerden kurulur, glif okunmaz: "Ma, Sc 19°, retro, vargottama". */
function planetName(p: ChartPlanet, labels: ChartLabels): string {
  const parts = [`${labels.graha[p.id]}, ${labels.signs[p.sign]} ${Math.floor(p.degInSign)}°`];
  if (p.retrograde) parts.push(labels.retro);
  if (p.combust) parts.push(labels.combust);
  if (p.vargottama) parts.push(labels.vargottama);
  return parts.join(", ");
}

export function ChartSvg({ data, style, labels, highlight = null, onHover, onPin }: ChartSvgProps) {
  const asc = data.ascSign ?? 0;
  const cells = useMemo(() => chartCells(style, asc), [style, asc]);
  // Yerleşim yalnızca kısaltmalara ve lagna işaretine bağlıdır; `labels` nesnesi her
  // çizimde yeniden kurulsa da hesap tekrarlanmaz.
  const { graha, lagna } = labels;
  const textKey = JSON.stringify([graha, lagna]);
  const placed = useMemo(() => {
    const bySign = new Map<number, ChartPlanet[]>();
    for (const p of data.planets) {
      if (!drawable(p)) continue;
      bySign.set(p.sign, [...(bySign.get(p.sign) ?? []), p]);
    }
    return cells.map((cell) => {
      const planets = bySign.get(cell.sign) ?? [];
      const texts = planets.map((p) => labelText(p, graha));
      const layout: LabelLayout = layoutLabels(cell, texts, cell.lagnaAnchor ? lagna : "");
      return { cell, planets, layout };
    });
    // textKey, graha ve lagna'yı kapsar.
  }, [cells, data.planets, textKey]);

  const activate = (target: HighlightTarget) => (e: KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onPin?.(target);
    }
  };

  return (
    <svg
      className="chart"
      data-style={style}
      viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`}
      role="group"
      aria-label={labels.title}
      xmlns="http://www.w3.org/2000/svg"
    >
      {cells.map((cell) => {
        const house: HighlightTarget = { type: "house", house: cell.house };
        const on = cellHighlighted(cell, highlight);
        const classes = ["chart-cell"];
        if (cell.house === 1) classes.push("chart-cell--lagna");
        if (on) classes.push("chart-cell--highlight");
        return (
          <g
            key={cell.house}
            className={`chart-house${on ? " chart-house--highlight" : ""}`}
            data-house={cell.house}
            data-sign={cell.sign}
            role="button"
            tabIndex={0}
            aria-pressed={on}
            aria-label={labels.house(cell.house, labels.signs[cell.sign])}
            onMouseEnter={() => onHover?.(house)}
            onMouseLeave={() => onHover?.(null)}
            onFocus={() => onHover?.(house)}
            onBlur={() => onHover?.(null)}
            onClick={() => onPin?.(house)}
            onKeyDown={activate(house)}
          >
            <path className={classes.join(" ")} d={pathOf(cell.points)} />
            {cell.signAnchor && (
              <text className="chart-sign" x={cell.signAnchor.x} y={cell.signAnchor.y} textAnchor="middle" dominantBaseline="central">
                {cell.sign + 1}
              </text>
            )}
            {cell.lagnaAnchor && (
              <text className="chart-sign" x={cell.lagnaAnchor.x} y={cell.lagnaAnchor.y} textAnchor="middle" dominantBaseline="central">
                {labels.lagna}
              </text>
            )}
          </g>
        );
      })}
      {placed.map(({ cell, planets, layout }) =>
        planets.map((p, i) => {
          const target: HighlightTarget = { type: "graha", id: p.id };
          const on = sameTarget(highlight, target);
          const { x, y, width, height } = layout.boxes[i];
          return (
            <g
              key={p.id}
              className={`chart-planet${on ? " chart-planet--highlight" : ""}`}
              data-graha={p.id}
              data-house={cell.house}
              data-x={x}
              data-y={y}
              role="button"
              tabIndex={0}
              aria-pressed={on}
              aria-label={planetName(p, labels)}
              onMouseEnter={() => onHover?.(target)}
              onMouseLeave={() => onHover?.(null)}
              onFocus={() => onHover?.(target)}
              onBlur={() => onHover?.(null)}
              onClick={() => onPin?.(target)}
              onKeyDown={activate(target)}
            >
              <rect
                className="chart-planet-bg"
                x={x - width / 2}
                y={y - height / 2}
                width={width}
                height={height}
                rx={layout.fontSize * 0.25}
              />
              <text
                className="chart-text"
                x={x}
                y={y}
                textAnchor="middle"
                dominantBaseline="central"
                style={{ fontSize: `${layout.fontSize}px` }}
                textDecoration={p.vargottama ? "underline" : undefined}
              >
                <tspan>{labels.graha[p.id]}</tspan>
                {p.retrograde && <tspan>℞</tspan>}
                {p.combust && <tspan>c</tspan>}
              </text>
            </g>
          );
        }),
      )}
    </svg>
  );
}
