import { render, screen, within } from "@testing-library/react";
import { i18n } from "../i18n";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { buildPrintDocument } from "./model";
import { defaultPrintOptions, type PrintOptions } from "./options";
import { PrintDocumentView } from "./PrintDocumentView";
import shellCss from "../shell/shell.css?raw";
import css from "./print.css?raw";

const t = i18n.getFixedT("tr");
const renderDoc = (opened = fixture().opened, options: PrintOptions = defaultPrintOptions()) =>
  render(<PrintDocumentView doc={buildPrintDocument(t, opened, options)} />);
const sheet = (name: string) => screen.getByRole("region", { name });

test("four sheets in spec order, each on paper tokens with the client, the chart and its page number", () => {
  const { container } = renderDoc();
  const pages = [...container.querySelectorAll("section.print-page")];
  expect(pages.map((p) => p.getAttribute("data-page"))).toEqual(["cover", "charts", "dasha", "grahas"]);
  for (const p of pages) expect(p).toHaveClass("paper");
  expect(pages.map((p) => p.querySelector(".print-running")!.textContent)).toEqual([
    "Örnek Danışan · Doğum haritası1 / 4",
    "Örnek Danışan · Doğum haritası2 / 4",
    "Örnek Danışan · Doğum haritası3 / 4",
    "Örnek Danışan · Doğum haritası4 / 4",
  ]);
  expect(screen.getAllByRole("heading", { level: 2 }).map((h) => h.textContent)).toEqual(["Örnek Danışan", "Haritalar: D1 ve D9", "Vimshottari dasa", "Graha tablosu"]);
});

test("page numbers count only the chosen pages", () => {
  const { container } = renderDoc(fixture().opened, { pages: ["charts", "grahas"], style: "north" });
  expect([...container.querySelectorAll(".print-running .num")].map((n) => n.textContent)).toEqual(["1 / 2", "2 / 2"]);
});

test("cover lists birth data and settings; the warnings section appears only when there are warnings", () => {
  renderDoc();
  const cover = sheet("Örnek Danışan");
  expect(within(cover).getByText("New Delhi, Delhi, India")).toBeInTheDocument();
  expect(within(cover).getByText("Lahiri 23°51′")).toBeInTheDocument();
  expect(within(cover).getByText("Jyotish Desk 0.1.0 · Swiss Ephemeris 2.10.03")).toBeInTheDocument();
  expect(within(cover).queryByRole("heading", { name: "Uyarılar" })).toBeNull();
});

test("cover warnings are printed as words", () => {
  renderDoc(unknownTimeOpened());
  const cover = sheet("Örnek Danışan");
  expect(within(cover).getByRole("heading", { name: "Uyarılar" })).toBeInTheDocument();
  expect(within(cover).getByRole("listitem")).toHaveTextContent("Saat bilinmiyor: Doğum saati bilinmiyor; öğlen 12:00 alındı.");
});

test("charts sheet draws D1 and D9 as static images in the chosen style, with the mark legend", () => {
  renderDoc(fixture().opened, { pages: ["charts"], style: "south" });
  const charts = sheet("Haritalar: D1 ve D9");
  const images = within(charts).getAllByRole("img");
  expect(images.map((i) => i.getAttribute("aria-label"))).toEqual(["D1 haritası", "D9 haritası"]);
  for (const svg of images) expect(svg).toHaveAttribute("data-style", "south");
  expect(charts.querySelectorAll('[role="button"], [tabindex]')).toHaveLength(0);
  // Retro düz "R": Satürn'de var, hep geri giden düğümlerde yok.
  const d1 = charts.querySelector('[data-varga="D1"]')!;
  expect(d1.querySelector('[data-graha="Saturn"]')).toHaveTextContent(/^SaR$/);
  expect(d1.querySelector('[data-graha="Rahu"]')).toHaveTextContent(/^Ra$/);
  expect(d1.querySelector('[data-graha="Ketu"]')).toHaveTextContent(/^Ke$/);
  expect(d1.querySelector('[data-graha="Mercury"]')).toHaveTextContent(/^Mec$/);
  expect(d1.querySelectorAll(".chart-cell--lagna")).toHaveLength(1);
  expect(within(charts).getByText("R: retro · c: yanık · altı çizili: vargottama")).toBeInTheDocument();
});

test("with an unknown birth time the charts carry the note and no lagna cell", () => {
  renderDoc(unknownTimeOpened(), { pages: ["charts"], style: "north" });
  const charts = sheet("Haritalar: D1 ve D9");
  expect(within(charts).getByText(/Doğum saati bilinmiyor: lagna ve evler gösterilmiyor/)).toBeInTheDocument();
  expect(charts.querySelectorAll(".chart-cell--lagna")).toHaveLength(0);
});

test("dasha sheet: nine captioned tables of nine antardashas", () => {
  renderDoc(fixture().opened, { pages: ["dasha"], style: "north" });
  const tables = within(sheet("Vimshottari dasa")).getAllByRole("table");
  expect(tables).toHaveLength(9);
  expect(tables[1].querySelector("caption")).toHaveTextContent("Jüpiter 2000-09-18 – 2016-09-18");
  const rows = within(tables[1]).getAllByRole("row");
  expect(rows).toHaveLength(10);
  expect(rows[0]).toHaveTextContent("AntardasaBaşlangıçBitiş");
  expect(rows[1]).toHaveTextContent("Jüpiter2000-09-182002-11-07");
});

test("graha sheet: a row per graha under the lagna, R only where it is shown", () => {
  renderDoc(fixture().opened, { pages: ["grahas"], style: "north" });
  const table = within(sheet("Graha tablosu")).getByRole("table");
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Graha", "Burç", "Derece", "Boylam", "Enlem", "Hız (°/gün)", "Retro", "Nakshatra", "Pada", "Yanık", "Durum", "Ev"]);
  expect(within(table).getAllByRole("rowheader").map((h) => h.textContent)).toEqual(["Lagna", "Güneş", "Ay", "Mars", "Merkür", "Jüpiter", "Venüs", "Satürn", "Rahu", "Ketu"]);
  const retro = (key: string) => table.querySelector(`[data-row="${key}"]`)!.querySelectorAll("td")[5];
  expect(retro("Saturn")).toHaveTextContent(/^R$/);
  expect(retro("Rahu")).toHaveTextContent(/^$/);
  expect(retro("Ketu")).toHaveTextContent(/^$/);
});

test("print CSS: zero page margin with padded sheets, one sheet per page, chrome hidden, tokens only", () => {
  expect(css).toMatch(/@page\s*{[^}]*size:\s*A4;[^}]*margin:\s*0;/);
  expect(css).toMatch(/\.print-page\s*{[^}]*padding:\s*15mm;/);
  // Kâğıtta adlı yazı tipi: sistem fontu PDF metin katmanında noktalı "i"yi yitirir (D6).
  expect(css).toMatch(/\.print-page\s*{[^}]*font-family:\s*"Helvetica Neue", Helvetica, Arial, sans-serif;/);
  expect(css).not.toMatch(/-apple-system|system-ui|var\(--font-ui\)/);
  const print = css.slice(css.indexOf("@media print"));
  expect(print).toMatch(/\.no-print\s*{[^}]*display:\s*none\s*!important;/);
  expect(print).toMatch(/\.print-page\s*{[^}]*break-after:\s*page;[^}]*break-inside:\s*avoid;/);
  expect(print).toMatch(/\.print-page:last-child\s*{[^}]*break-after:\s*auto;/);
  expect(css).not.toMatch(/#[0-9a-fA-F]{3,8}\b/);
  // Kabuk baskıda blok akışına döner; kural shell.css'in sonundadır (ekran kurallarından sonra).
  const shellPrint = shellCss.slice(shellCss.lastIndexOf("@media print"));
  expect(shellPrint).toMatch(/\.shell,\s*\.shell-body,\s*\.shell-main\s*{[^}]*display:\s*block;[^}]*padding:\s*0;/);
  expect(shellCss.indexOf(".shell-main {")).toBeLessThan(shellCss.lastIndexOf("@media print"));
  expect(css).toMatch(/\.paper \.chart-cell--lagna\s*{[^}]*stroke-width:\s*1\.2;/);
});
