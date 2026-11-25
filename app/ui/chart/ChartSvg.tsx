// Kuzey Hint, Güney Hint ve daire haritası. Yalnızca belirteç sınıfları kullanılır;
// gezegenlere renk verilmez. Retro "R", yanma "c", vargottama altı çizili (spec §2.3).
// svg `role="group"`dur: `img` alt öğeleri sunumsal yapar ve ev/gezegen düğmelerini
// ekran okuyucudan gizlerdi (spec §6).
import { useMemo, type KeyboardEvent } from "react";
import { sameTarget, type HighlightTarget } from "../highlight/target";
import "./chart.css";
import { chartCells, layoutLabels, pathOf, VIEWBOX, type ChartCell, type LabelLayout } from "./geometry";
import { RETRO_MARK, showsRetro, type ChartData, type ChartLabels, type ChartPlanet, type ChartStyle } from "./types";

export interface ChartSvgProps {
  data: ChartData;
  style: ChartStyle;
  labels: ChartLabels;
  highlight?: HighlightTarget | null;
  /** Yalnızca `aria-pressed` için: sabitlenen öğe. Varsayılan `highlight`. */
  pressed?: HighlightTarget | null;
  onHover?: (target: HighlightTarget | null) => void;
  onPin?: (target: HighlightTarget) => void;
  /**
   * `false`: basılı çizim (Plan 12). Evler ve gezegenler düğme değildir, sekme durağı olmaz;
   * svg tek bir resimdir (`role="img"`) ve ayrıntıyı yanındaki tablo taşır. Varsayılan `true`.
   */
  interactive?: boolean;
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
  return `${graha[p.id]}${showsRetro(p.id, p.retrograde) ? RETRO_MARK : ""}${p.combust ? "c" : ""}`;
}

/** Transit etiketi: parantez biçimsel işarettir (renk değil, C9). */
function transitText(p: ChartPlanet, graha: ChartLabels["graha"]): string {
  return `(${graha[p.id]}${showsRetro(p.id, p.retrograde) ? RETRO_MARK : ""})`;
}

/** Erişilebilir ad sözcüklerden kurulur, glif okunmaz: "Ma, Sc 19°, retro, vargottama". */
function planetName(p: ChartPlanet, labels: ChartLabels): string {
  const parts = [`${labels.graha[p.id]}, ${labels.signs[p.sign]} ${Math.floor(p.degInSign)}°`];
  if (showsRetro(p.id, p.retrograde)) parts.push(labels.retro);
  if (p.combust) parts.push(labels.combust);
  if (p.vargottama) parts.push(labels.vargottama);
  return parts.join(", ");
}

export function ChartSvg({ data, style, labels, highlight = null, pressed = highlight, onHover, onPin, interactive = true }: ChartSvgProps) {
  // Saat bilinmiyorsa lagna yoktur: 1. ev Koç çizilir, lagna işareti ve dolgusu çizilmez,
  // evler yalnızca burç adıyla anılır (ev numarası anlamsızdır).
  const hasLagna = data.ascSign !== null;
  const asc = data.ascSign ?? 0;
  const cells = useMemo(() => chartCells(style, asc), [style, asc]);
  // Yerleşim yalnızca kısaltmalara ve lagna işaretine bağlıdır; `labels` nesnesi her
  // çizimde yeniden kurulsa da hesap tekrarlanmaz.
  const { graha, lagna } = labels;
  const textKey = JSON.stringify([graha, lagna]);
  const placed = useMemo(() => {
    const group = (list: readonly ChartPlanet[]) => {
      const bySign = new Map<number, ChartPlanet[]>();
      for (const p of list) {
        if (!drawable(p)) continue;
        bySign.set(p.sign, [...(bySign.get(p.sign) ?? []), p]);
      }
      return bySign;
    };
    const natal = group(data.planets);
    const transit = group(data.transits ?? []);
    return cells.map((cell) => {
      const planets = natal.get(cell.sign) ?? [];
      const transits = transit.get(cell.sign) ?? [];
      const texts = [...planets.map((p) => labelText(p, graha)), ...transits.map((p) => transitText(p, graha))];
      const layout: LabelLayout = layoutLabels(cell, texts, hasLagna && cell.lagnaAnchor ? lagna : "");
      return { cell, planets, transits, layout };
    });
    // textKey, graha ve lagna'yı kapsar.
  }, [cells, data.planets, data.transits, textKey, hasLagna]);

  const activate = (target: HighlightTarget) => (e: KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onPin?.(target);
    }
  };

  /** Düğme davranışı: yalnızca etkileşimli çizimde; basılı çizimde hiçbir öznitelik eklenmez. */
  const button = (target: HighlightTarget, isPressed: boolean, name: string) =>
    interactive
      ? {
          role: "button",
          tabIndex: 0,
          "aria-pressed": isPressed,
          "aria-label": name,
          onMouseEnter: () => onHover?.(target),
          onMouseLeave: () => onHover?.(null),
          onFocus: () => onHover?.(target),
          onBlur: () => onHover?.(null),
          onClick: () => onPin?.(target),
          onKeyDown: activate(target),
        }
      : {};

  return (
    <svg
      className="chart"
      data-style={style}
      viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`}
      role={interactive ? "group" : "img"}
      aria-label={labels.title}
      xmlns="http://www.w3.org/2000/svg"
    >
      {cells.map((cell) => {
        const house: HighlightTarget = { type: "house", house: cell.house };
        const on = cellHighlighted(cell, highlight);
        const isPressed = cellHighlighted(cell, pressed);
        const classes = ["chart-cell"];
        if (hasLagna && cell.house === 1) classes.push("chart-cell--lagna");
        if (on) classes.push("chart-cell--highlight");
        return (
          <g
            key={cell.house}
            className={`chart-house${on ? " chart-house--highlight" : ""}`}
            data-house={cell.house}
            data-sign={cell.sign}
            {...button(house, isPressed, hasLagna ? labels.house(cell.house, labels.signs[cell.sign]) : labels.signs[cell.sign])}
          >
            <path className={classes.join(" ")} d={pathOf(cell.points)} />
            {cell.signAnchor && (
              <text className="chart-sign" x={cell.signAnchor.x} y={cell.signAnchor.y} textAnchor="middle" dominantBaseline="central">
                {cell.sign + 1}
              </text>
            )}
            {hasLagna && cell.lagnaAnchor && (
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
          const isPressed = sameTarget(pressed, target);
          const { x, y, width, height } = layout.boxes[i];
          return (
            <g
              key={p.id}
              className={`chart-planet${on ? " chart-planet--highlight" : ""}`}
              data-graha={p.id}
              data-house={cell.house}
              data-x={x}
              data-y={y}
              {...button(target, isPressed, planetName(p, labels))}
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
                {showsRetro(p.id, p.retrograde) && <tspan>{RETRO_MARK}</tspan>}
                {p.combust && <tspan>c</tspan>}
              </text>
            </g>
          );
        }),
      )}
      {placed.map(({ cell, planets, transits, layout }) =>
        transits.map((p, i) => {
          const { x, y } = layout.boxes[planets.length + i];
          return (
            <text
              key={`transit-${p.id}`}
              className="chart-transit"
              data-transit={p.id}
              data-house={cell.house}
              x={x}
              y={y}
              textAnchor="middle"
              dominantBaseline="central"
              style={{ fontSize: `${layout.fontSize}px` }}
              aria-hidden="true"
            >
              {transitText(p, labels.graha)}
            </text>
          );
        }),
      )}
    </svg>
  );
}
