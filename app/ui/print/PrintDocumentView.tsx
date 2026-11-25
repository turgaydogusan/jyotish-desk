// Baskı belgesini A4 sayfalar olarak dizer (UI spec §7). Her `.print-page` bir kâğıttır:
// kenar boşluğu sayfanın iç dolgusudur (print.css, Plan 12 D3) ve içerik tek kâğıda sığacak
// biçimde tasarlanmıştır. Ekranda aynı öğeler önizlemedir. Metinler `PrintDocument`'ten gelir.
import type { ReactNode } from "react";
import { ChartSvg } from "../chart/ChartSvg";
import type { ChartsPage, CoverPage, DashaPage, GrahasPage, PrintDocument, PrintPage, PrintRow } from "./model";
import "./print.css";

function Rows({ rows }: { rows: readonly PrintRow[] }) {
  return (
    <dl className="print-rows">
      {rows.map(([term, value]) => (
        <div key={term} className="print-row">
          <dt>{term}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}

function Cover({ page }: { page: CoverPage }) {
  return (
    <>
      <Rows rows={page.birth} />
      <h3 className="print-subheading">{page.settingsHeading}</h3>
      <Rows rows={page.settings} />
      {page.warnings.length > 0 && (
        <>
          <h3 className="print-subheading">{page.warningsHeading}</h3>
          <ul className="print-warnings">
            {page.warnings.map((w) => (
              <li key={w.name}>
                <strong>{w.name}</strong>: {w.hint}
              </li>
            ))}
          </ul>
        </>
      )}
      <p className="print-credit">{page.credit}</p>
    </>
  );
}

function Charts({ page }: { page: ChartsPage }) {
  return (
    <>
      {page.note && <p className="print-note">{page.note}</p>}
      {page.charts.map((c) => (
        <figure key={c.varga} className="print-chart" data-varga={c.varga}>
          <figcaption>{c.caption}</figcaption>
          {c.data ? <ChartSvg data={c.data} style={page.style} labels={c.labels} interactive={false} /> : <p className="print-note">{c.missing}</p>}
        </figure>
      ))}
      <p className="print-legend">{page.legend}</p>
    </>
  );
}

function Dasha({ page }: { page: DashaPage }) {
  return (
    <>
      {page.note && <p className="print-note">{page.note}</p>}
      <div className="print-dasha">
        {page.blocks.map((b) => (
          <table key={b.lord} className="print-table print-dasha-block">
            <caption>
              <strong>{b.lord}</strong> {b.start} – {b.end}
            </caption>
            <thead>
              <tr>
                <th scope="col">{page.columns.lord}</th>
                <th scope="col">{page.columns.start}</th>
                <th scope="col">{page.columns.end}</th>
              </tr>
            </thead>
            <tbody>
              {b.rows.map((r) => (
                <tr key={r.lord}>
                  <th scope="row">{r.lord}</th>
                  <td>{r.start}</td>
                  <td>{r.end}</td>
                </tr>
              ))}
            </tbody>
          </table>
        ))}
      </div>
    </>
  );
}

function Grahas({ page }: { page: GrahasPage }) {
  return (
    <>
      {page.note && <p className="print-note">{page.note}</p>}
      <table className="print-table print-grahas">
        <thead>
          <tr>
            <th scope="col">{page.rowHeader}</th>
            {page.columns.map((c) => (
              <th key={c.id} scope="col" className={c.numeric ? "num" : undefined}>
                {c.label}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {page.rows.map((r) => (
            <tr key={r.key} data-row={r.key}>
              <th scope="row">{r.name}</th>
              {r.cells.map((text, i) => (
                <td key={page.columns[i].id} className={page.columns[i].numeric ? "num" : undefined}>
                  {text}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </>
  );
}

function body(page: PrintPage): ReactNode {
  switch (page.id) {
    case "cover":
      return <Cover page={page} />;
    case "charts":
      return <Charts page={page} />;
    case "dasha":
      return <Dasha page={page} />;
    case "grahas":
      return <Grahas page={page} />;
  }
}

export function PrintDocumentView({ doc }: { doc: PrintDocument }) {
  return (
    <div className="print-document">
      {doc.pages.map((page, i) => (
        <section key={page.id} className="print-page paper" data-page={page.id} aria-labelledby={`print-page-${page.id}`}>
          <header className="print-running">
            <span>
              {doc.client} · {doc.chart}
            </span>
            <span className="num">
              {i + 1} / {doc.pages.length}
            </span>
          </header>
          <h2 id={`print-page-${page.id}`} className="print-heading">
            {page.heading}
          </h2>
          {body(page)}
        </section>
      ))}
    </div>
  );
}
