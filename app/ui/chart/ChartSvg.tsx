// Kuzey Hint, Güney Hint ve daire haritası. Yalnızca belirteç sınıfları kullanılır;
// gezegenlere renk verilmez. Retro "℞", yanma "c", vargottama altı çizili (spec §2.3).
import type { KeyboardEvent } from "react";
import { sameTarget, type HighlightTarget } from "../highlight/target";
import "./chart.css";
import { chartCells, pathOf, stackPositions, VIEWBOX, type ChartCell } from "./geometry";
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

export function ChartSvg({ data, style, labels, highlight = null, onHover, onPin }: ChartSvgProps) {
  const asc = data.ascSign ?? 0;
  const cells = chartCells(style, asc);
  const bySign = new Map<number, ChartPlanet[]>();
  for (const p of data.planets) bySign.set(p.sign, [...(bySign.get(p.sign) ?? []), p]);

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
      role="img"
      aria-label={labels.title}
      xmlns="http://www.w3.org/2000/svg"
    >
      {cells.map((cell) => {
        const house: HighlightTarget = { type: "house", house: cell.house };
        const classes = ["chart-cell"];
        if (cell.house === 1) classes.push("chart-cell--lagna");
        if (cellHighlighted(cell, highlight)) classes.push("chart-cell--highlight");
        return (
          <g key={cell.house} data-house={cell.house} data-sign={cell.sign}>
            <path
              className={classes.join(" ")}
              d={pathOf(cell.points)}
              aria-label={labels.house(cell.house, labels.signs[cell.sign])}
              onMouseEnter={() => onHover?.(house)}
              onMouseLeave={() => onHover?.(null)}
              onClick={() => onPin?.(house)}
            />
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
      {cells.map((cell) => {
        const planets = bySign.get(cell.sign) ?? [];
        const spots = stackPositions(cell.planetAnchor, planets.length, cell.stack);
        return planets.map((p, i) => {
          const target: HighlightTarget = { type: "graha", id: p.id };
          const on = sameTarget(highlight, target);
          const abbr = labels.graha[p.id];
          const marks = `${p.retrograde ? " ℞" : ""}${p.combust ? " c" : ""}`;
          const { x, y } = spots[i];
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
              aria-label={`${abbr}${marks} ${Math.floor(p.degInSign)}°`}
              onMouseEnter={() => onHover?.(target)}
              onMouseLeave={() => onHover?.(null)}
              onFocus={() => onHover?.(target)}
              onBlur={() => onHover?.(null)}
              onClick={() => onPin?.(target)}
              onKeyDown={activate(target)}
            >
              <rect className="chart-planet-bg" x={x - 4} y={y - 2.6} width={8} height={5.2} rx={1} />
              <text
                className="chart-text"
                x={x}
                y={y}
                textAnchor="middle"
                dominantBaseline="central"
                textDecoration={p.vargottama ? "underline" : undefined}
              >
                <tspan>{abbr}</tspan>
                {p.retrograde && <tspan dx={0.4}>℞</tspan>}
                {p.combust && <tspan dx={0.4}>c</tspan>}
              </text>
            </g>
          );
        });
      })}
    </svg>
  );
}
