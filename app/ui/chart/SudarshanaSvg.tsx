// Sudarshana chakra çizimi: üç iç içe halka (içten dışa lagna, Ay, Güneş), her halkada on iki
// ev. Yalnızca belirteç sınıfları kullanılır; renk anlam taşımaz (C9). Her halkanın 1. evi
// dolguludur; lagna halkasının 1. evinde lagna işareti yazar (Ay halkasının 1. evinde Ay'ın,
// Güneş halkasının 1. evinde Güneş'in kendisi durur). Çizim tek bir resimdir (`role="img"`): 36 hücre ve 27 etiket
// sekme durağı olmaz; klavye ve ekran okuyucu karşılığı panelin tablosudur. Fare ile üzerine
// gelmek ve tıklamak ortak vurguyu sürer (UI spec §3.4). Halkası olmayan (`houses: null`: doğum
// saati bilinmiyor) halka çizilmez.
import { useMemo } from "react";
import type { HighlightTarget } from "../highlight/target";
import "./chart.css";
import { layoutLabels, pathOf, VIEWBOX } from "./geometry";
import { SUDARSHANA_RADII, SUDARSHANA_SIGN_FONT, sudarshanaCell } from "./sudarshana";
import type { GrahaId } from "./types";

export interface SudarshanaRingData {
  /** On iki ev, 1. evden başlayarak; `null`: halka çizilmez. */
  houses: { house: number; sign: number; grahas: GrahaId[] }[] | null;
}

export interface SudarshanaLabels {
  /** Erişilebilir ad (svg `role="img"` + `aria-label`). */
  title: string;
  /** Graha kısaltmaları. */
  graha: Record<GrahaId, string>;
  /** Lagna halkasının 1. evindeki işaret. */
  lagna: string;
}

export interface SudarshanaSvgProps {
  /** Üç halka, içten dışa. */
  rings: readonly SudarshanaRingData[];
  labels: SudarshanaLabels;
  highlight?: HighlightTarget | null;
  onHover?: (target: HighlightTarget | null) => void;
  onPin?: (target: HighlightTarget) => void;
}

export function SudarshanaSvg({ rings, labels, highlight = null, onHover, onPin }: SudarshanaSvgProps) {
  const { graha, lagna } = labels;
  const textKey = JSON.stringify([graha, lagna]);
  const placed = useMemo(
    () =>
      rings.flatMap((ring, k) =>
        (ring.houses ?? []).map((h) => {
          const cell = sudarshanaCell(k, h.house);
          // Lagna işareti yalnızca iç halkanın 1. evindedir ve grahalardan sonra yerleşir.
          const mark = k === 0 && h.house === 1 ? lagna : null;
          const texts = [...h.grahas.map((g) => graha[g]), ...(mark !== null ? [mark] : [])];
          return { ring: k, house: h.house, sign: h.sign, grahas: h.grahas, mark, cell, layout: layoutLabels(cell.labelCell, texts) };
        }),
      ),
    // textKey, graha ve lagna'yı kapsar.
    [rings, textKey],
  );
  // Ev vurgusu D1 evidir: lagna halkasının o evindeki burç (Rust'tan gelen veri; hesap değil).
  const activeSign = highlight?.type === "sign" ? highlight.sign : highlight?.type === "house" ? (rings[0]?.houses?.[highlight.house - 1]?.sign ?? null) : null;
  const activeGraha = highlight?.type === "graha" ? highlight.id : null;
  const pointer = (target: HighlightTarget) => ({
    onMouseEnter: () => onHover?.(target),
    onMouseLeave: () => onHover?.(null),
    onClick: () => onPin?.(target),
  });

  return (
    <svg className="chart sudarshana" viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`} role="img" aria-label={labels.title} xmlns="http://www.w3.org/2000/svg">
      <circle className="sudarshana-hub" cx={VIEWBOX / 2} cy={VIEWBOX / 2} r={SUDARSHANA_RADII[0]} aria-hidden="true" />
      {placed.map(({ ring, house, sign, cell }) => {
        const on = activeSign === sign;
        const classes = ["chart-cell"];
        if (house === 1) classes.push("chart-cell--lagna");
        if (on) classes.push("chart-cell--highlight");
        return (
          <g
            key={`${ring}-${house}`}
            className={`chart-house${on ? " chart-house--highlight" : ""}`}
            data-ring={ring}
            data-house={house}
            data-sign={sign}
            aria-hidden="true"
            {...pointer({ type: "sign", sign })}
          >
            <path className={classes.join(" ")} d={pathOf(cell.points)} />
            <text
              className="sudarshana-sign"
              x={cell.signAnchor.x}
              y={cell.signAnchor.y}
              textAnchor="middle"
              dominantBaseline="central"
              style={{ fontSize: `${SUDARSHANA_SIGN_FONT}px` }}
            >
              {sign + 1}
            </text>
          </g>
        );
      })}
      {placed.map(({ ring, house, grahas, layout }) =>
        grahas.map((g, i) => {
          const { x, y, width, height } = layout.boxes[i];
          return (
            <g
              key={`${ring}-${g}`}
              className={`chart-planet${activeGraha === g ? " chart-planet--highlight" : ""}`}
              data-ring={ring}
              data-graha={g}
              data-house={house}
              aria-hidden="true"
              {...pointer({ type: "graha", id: g })}
            >
              <rect className="chart-planet-bg" x={x - width / 2} y={y - height / 2} width={width} height={height} rx={layout.fontSize * 0.25} />
              <text className="chart-text" x={x} y={y} textAnchor="middle" dominantBaseline="central" style={{ fontSize: `${layout.fontSize}px` }}>
                {labels.graha[g]}
              </text>
            </g>
          );
        }),
      )}
      {placed.map(({ ring, house, grahas, mark, layout }) => {
        if (mark === null) return null;
        const { x, y } = layout.boxes[grahas.length];
        return (
          <text
            key={`mark-${ring}`}
            className="chart-mark"
            data-mark={mark}
            data-ring={ring}
            data-house={house}
            x={x}
            y={y}
            textAnchor="middle"
            dominantBaseline="central"
            style={{ fontSize: `${layout.fontSize}px` }}
            aria-hidden="true"
          >
            {mark}
          </text>
        );
      })}
    </svg>
  );
}
