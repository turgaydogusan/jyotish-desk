import { RETRO_MARK } from "./chart/types";
import "./print.css";

/** Baskı denemesinin satırı: burç Rust'tan gelir (`Chart.grahas[].sign`), burada türetilmez. */
export interface SpikeRow {
  body: string;
  /** 0 = Mesha … 11 = Meena; `null` bilinmiyor. */
  sign: number | null;
  lon: number | null;
  /** Retro işareti basılır mı (`showsRetro`: Rahu/Ketu'da hiç). */
  showRetro: boolean;
}

// Güney Hint düzeni: 4×4 ızgaranın dış halkası, Meena sol üstten saat yönünde.
// [satır, sütun] → burç index (0 = Mesha).
const SOUTH_CELLS: Array<[number, number, number]> = [
  [0, 0, 11], [0, 1, 0], [0, 2, 1], [0, 3, 2],
  [1, 3, 3], [2, 3, 4], [3, 3, 5], [3, 2, 6],
  [3, 1, 7], [3, 0, 8], [2, 0, 9], [1, 0, 10],
];

export function PrintSpike({ rows }: { rows: SpikeRow[] }) {
  const bySign = new Map<number, string[]>();
  for (const r of rows) {
    if (r.sign === null) continue;
    bySign.set(r.sign, [...(bySign.get(r.sign) ?? []), r.body]);
  }
  const cell = 45;
  return (
    <div>
      <section className="print-page">
        <h2>Doğum haritası — ğüşıöç İ</h2>
        <p lang="hi">राशि</p>
        <svg
          data-testid="spike-chart"
          viewBox="0 0 180 180"
          width="160mm"
          height="160mm"
          xmlns="http://www.w3.org/2000/svg"
        >
          {SOUTH_CELLS.map(([r, c, sign]) => (
            <g key={sign} data-testid={`sign-${sign}`}>
              <rect x={c * cell} y={r * cell} width={cell} height={cell} fill="none" stroke="#000" strokeWidth={0.5} />
              <text x={c * cell + 3} y={r * cell + 10} fontSize={6}>
                {(bySign.get(sign) ?? []).join(" ")}
              </text>
            </g>
          ))}
        </svg>
      </section>
      <section className="print-page">
        <h2>Graha tablosu</h2>
        <table className="spike-table">
          <thead>
            <tr><th>Graha</th><th>Boylam</th><th>R</th></tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.body}>
                <td>{r.body}</td>
                <td>{r.lon === null ? "—" : r.lon.toFixed(4)}</td>
                <td>{r.showRetro ? RETRO_MARK : ""}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </div>
  );
}
