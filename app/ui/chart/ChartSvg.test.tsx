import { fireEvent, render, screen, within } from "@testing-library/react";
import chartCss from "./chart.css?raw";
import { ChartSvg } from "./ChartSvg";
import { RETRO_MARK, type ChartData, type ChartLabels, type ChartStyle } from "./types";

const labels: ChartLabels = {
  title: "Chart",
  graha: { Sun: "Su", Moon: "Mo", Mars: "Ma", Mercury: "Me", Jupiter: "Ju", Venus: "Ve", Saturn: "Sa", Rahu: "Ra", Ketu: "Ke" },
  signs: ["Ar", "Ta", "Ge", "Cn", "Le", "Vi", "Li", "Sc", "Sg", "Cp", "Aq", "Pi"],
  lagna: "As",
  house: (house, sign) => `House ${house}, ${sign}`,
  retro: "retrograde",
  combust: "combust",
  vargottama: "vargottama",
};

const data: ChartData = {
  ascSign: 4,
  planets: [
    { id: "Sun", sign: 4, degInSign: 12.5, retrograde: false },
    { id: "Moon", sign: 4, degInSign: 3, retrograde: false, vargottama: true },
    { id: "Mercury", sign: 4, degInSign: 20, retrograde: true, combust: true },
    { id: "Saturn", sign: 10, degInSign: 1, retrograde: true },
  ],
};

const STYLES: ChartStyle[] = ["north", "south", "circle"];

test.each(STYLES)("%s renders a named group with 12 houses whose children stay accessible", (style) => {
  const { container } = render(<ChartSvg data={data} style={style} labels={labels} />);
  expect(screen.queryByRole("img")).toBeNull();
  const chart = screen.getByRole("group", { name: "Chart" });
  expect(within(chart).getAllByRole("button", { name: /^House / })).toHaveLength(12);
  expect(within(chart).getByRole("button", { name: "Su, Le 12°" })).toBeInTheDocument();
  expect(container.querySelectorAll("path.chart-cell")).toHaveLength(12);
  expect(container.querySelector("svg")!.dataset.style).toBe(style);
});

test.each(STYLES)("%s: planets are placed in the house of their sign; lagna house is filled", (style) => {
  const { container } = render(<ChartSvg data={data} style={style} labels={labels} />);
  const houseOf = (id: string) => container.querySelector(`[data-graha="${id}"]`)!.getAttribute("data-house");
  expect(houseOf("Sun")).toBe("1");
  expect(houseOf("Saturn")).toBe("7");
  const lagna = container.querySelectorAll("path.chart-cell--lagna");
  expect(lagna).toHaveLength(1);
  expect(lagna[0].closest("g")!.getAttribute("aria-label")).toBe("House 1, Le");
});

test("north shows sign numbers; south and circle show the lagna marker", () => {
  const north = render(<ChartSvg data={data} style="north" labels={labels} />);
  const house1 = north.container.querySelector('g[data-house="1"]')!;
  expect(house1.querySelector("text.chart-sign")).toHaveTextContent("5");
  expect(north.queryByText("As")).toBeNull();
  north.unmount();
  for (const style of ["south", "circle"] as const) {
    const r = render(<ChartSvg data={data} style={style} labels={labels} />);
    expect(r.getAllByText("As")).toHaveLength(1);
    r.unmount();
  }
});

test("circle renders the sign number in all 12 houses", () => {
  const { container } = render(<ChartSvg data={data} style="circle" labels={labels} />);
  for (let h = 1; h <= 12; h++) {
    const g = container.querySelector(`g.chart-house[data-house="${h}"]`)!;
    const sign = Number(g.getAttribute("data-sign"));
    const texts = [...g.querySelectorAll("text.chart-sign")].map((t) => t.textContent);
    expect(texts, `house ${h}`).toContain(String(sign + 1));
  }
});

test("nodes are always retrograde, so Rahu and Ketu carry no retro mark", () => {
  const nodes: ChartData = {
    ...data,
    planets: [
      { id: "Rahu", sign: 2, degInSign: 5, retrograde: true },
      { id: "Ketu", sign: 8, degInSign: 5, retrograde: true },
    ],
  };
  render(<ChartSvg data={nodes} style="north" labels={labels} />);
  expect(screen.getByRole("button", { name: "Ra, Ge 5°" }).querySelector("text")).toHaveTextContent(/^Ra$/);
  expect(screen.getByRole("button", { name: "Ke, Sg 5°" }).querySelector("text")).toHaveTextContent(/^Ke$/);
});

test("markers: retro R, combust c, vargottama underline — no colour coding", () => {
  render(<ChartSvg data={data} style="north" labels={labels} />);
  const mercury = screen.getByRole("button", { name: "Me, Le 20°, retrograde, combust" });
  expect(mercury).toHaveTextContent("MeRc");
  const moon = screen.getByRole("button", { name: "Mo, Le 3°, vargottama" });
  expect(moon.querySelector("text")).toHaveAttribute("text-decoration", "underline");
  expect(screen.getByRole("button", { name: "Su, Le 12°" }).querySelector("text")).not.toHaveAttribute("text-decoration");
  for (const b of screen.getAllByRole("button")) expect(b.getAttribute("aria-label")).not.toMatch(new RegExp(`\\p{Ll}${RETRO_MARK}`, "u"));
  for (const el of document.querySelectorAll("svg *")) {
    expect(el.getAttribute("fill") ?? "").not.toMatch(/#|rgb/);
    expect(el.getAttribute("stroke") ?? "").not.toMatch(/#|rgb/);
  }
});

test("several planets in one house get distinct label positions", () => {
  const { container } = render(<ChartSvg data={data} style="north" labels={labels} />);
  const spots = [...container.querySelectorAll('[data-graha][data-house="1"]')].map(
    (g) => `${g.getAttribute("data-x")},${g.getAttribute("data-y")}`,
  );
  expect(spots).toHaveLength(3);
  expect(new Set(spots).size).toBe(3);
});

test("null ascendant draws house 1 as Aries", () => {
  const { container } = render(<ChartSvg data={{ ...data, ascSign: null }} style="north" labels={labels} />);
  expect(container.querySelector('g[data-house="1"]')!.getAttribute("data-sign")).toBe("0");
  expect(container.querySelector('[data-graha="Saturn"]')!.getAttribute("data-house")).toBe("11");
});

test("hover, focus, click and keyboard report highlight targets", () => {
  const onHover = vi.fn();
  const onPin = vi.fn();
  const { container } = render(<ChartSvg data={data} style="north" labels={labels} onHover={onHover} onPin={onPin} />);
  const sun = screen.getByRole("button", { name: "Su, Le 12°" });
  fireEvent.mouseEnter(sun);
  expect(onHover).toHaveBeenLastCalledWith({ type: "graha", id: "Sun" });
  fireEvent.mouseLeave(sun);
  expect(onHover).toHaveBeenLastCalledWith(null);
  fireEvent.click(sun);
  expect(onPin).toHaveBeenLastCalledWith({ type: "graha", id: "Sun" });
  fireEvent.keyDown(sun, { key: "Enter" });
  expect(onPin).toHaveBeenCalledTimes(2);
  fireEvent.mouseEnter(container.querySelector('g[data-house="3"] path')!);
  expect(onHover).toHaveBeenLastCalledWith({ type: "house", house: 3 });
});

test("houses are keyboard-operable buttons that mirror hover on focus", () => {
  const onHover = vi.fn();
  const onPin = vi.fn();
  render(<ChartSvg data={data} style="north" labels={labels} onHover={onHover} onPin={onPin} />);
  const h3 = screen.getByRole("button", { name: "House 3, Li" });
  expect(h3).toHaveAttribute("tabindex", "0");
  expect(h3).toHaveAttribute("aria-pressed", "false");
  fireEvent.focus(h3);
  expect(onHover).toHaveBeenLastCalledWith({ type: "house", house: 3 });
  fireEvent.blur(h3);
  expect(onHover).toHaveBeenLastCalledWith(null);
  fireEvent.keyDown(h3, { key: "Enter" });
  expect(onPin).toHaveBeenLastCalledWith({ type: "house", house: 3 });
  fireEvent.keyDown(h3, { key: " " });
  expect(onPin).toHaveBeenCalledTimes(2);
  fireEvent.click(h3);
  expect(onPin).toHaveBeenCalledTimes(3);
  fireEvent.keyDown(h3, { key: "a" });
  expect(onPin).toHaveBeenCalledTimes(3);
});

test("house aria-pressed reflects the pinned house", () => {
  render(<ChartSvg data={data} style="south" labels={labels} highlight={{ type: "house", house: 3 }} />);
  expect(screen.getByRole("button", { name: "House 3, Li" })).toHaveAttribute("aria-pressed", "true");
  expect(screen.getByRole("button", { name: "House 4, Sc" })).toHaveAttribute("aria-pressed", "false");
});

test("invalid planets (non-finite degree, sign out of range) are skipped", () => {
  const bad: ChartData = {
    ascSign: 0,
    planets: [
      { id: "Sun", sign: 2, degInSign: Number.NaN, retrograde: false },
      { id: "Moon", sign: 12, degInSign: 3, retrograde: false },
      { id: "Mars", sign: -1, degInSign: 3, retrograde: false },
      { id: "Venus", sign: 1.5, degInSign: 3, retrograde: false },
      { id: "Jupiter", sign: 3, degInSign: Number.POSITIVE_INFINITY, retrograde: false },
      { id: "Saturn", sign: 5, degInSign: 7, retrograde: false },
    ],
  };
  for (const style of STYLES) {
    const { container, unmount } = render(<ChartSvg data={bad} style={style} labels={labels} />);
    expect([...container.querySelectorAll("[data-graha]")].map((g) => g.getAttribute("data-graha"))).toEqual(["Saturn"]);
    expect(container.textContent).not.toMatch(/NaN|Infinity/);
    unmount();
  }
});

test("asc outside 0…11 is normalised", () => {
  const { container } = render(<ChartSvg data={{ ...data, ascSign: 16 }} style="north" labels={labels} />);
  expect(container.querySelector('g[data-house="1"]')!.getAttribute("data-sign")).toBe("4");
  expect(container.querySelector('[data-graha="Sun"]')!.getAttribute("data-house")).toBe("1");
});

test("five marked planets in one house: label backgrounds are as wide as the estimate and never overlap", () => {
  const crowd: ChartData = {
    ascSign: 0,
    planets: (["Venus", "Mercury", "Mars", "Jupiter", "Saturn"] as const).map((id) => ({
      id,
      sign: 2,
      degInSign: 10,
      retrograde: true,
      combust: true,
    })),
  };
  for (const style of STYLES) {
    const { container, unmount } = render(<ChartSvg data={crowd} style={style} labels={labels} />);
    const rects = [...container.querySelectorAll('[data-graha] rect.chart-planet-bg')].map((r) => ({
      x: Number(r.getAttribute("x")),
      y: Number(r.getAttribute("y")),
      w: Number(r.getAttribute("width")),
      h: Number(r.getAttribute("height")),
    }));
    expect(rects).toHaveLength(5);
    for (const r of rects) expect(r.w).toBeGreaterThan(8);
    for (let i = 0; i < 5; i++)
      for (let j = i + 1; j < 5; j++) {
        const a = rects[i], b = rects[j];
        const sep = a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y;
        expect(sep, `${style} ${i}×${j}`).toBe(true);
      }
    unmount();
  }
});

test("highlighted graha and house get highlight classes", () => {
  const { container, rerender } = render(
    <ChartSvg data={data} style="north" labels={labels} highlight={{ type: "graha", id: "Saturn" }} />,
  );
  expect(container.querySelector('[data-graha="Saturn"]')).toHaveClass("chart-planet--highlight");
  expect(screen.getByRole("button", { name: "Sa, Aq 1°, retrograde" })).toHaveAttribute("aria-pressed", "true");
  rerender(<ChartSvg data={data} style="north" labels={labels} highlight={{ type: "sign", sign: 10 }} />);
  expect(container.querySelector('g[data-house="7"] path')).toHaveClass("chart-cell--highlight");
  expect(container.querySelector('g[data-house="7"]')).toHaveClass("chart-house--highlight");
  // Lagna evi zaten --select zeminli: vurgu orada da ağırlık ve kalın çizgiyle ayrışır.
  rerender(<ChartSvg data={data} style="north" labels={labels} highlight={{ type: "house", house: 1 }} />);
  expect(container.querySelector('g[data-house="1"]')).toHaveClass("chart-house--highlight");
  expect(container.querySelector('g[data-house="1"] path')).toHaveClass("chart-cell--lagna", "chart-cell--highlight");
});

const css = (chartCss as string).replace(/\/\*[\s\S]*?\*\//g, "").replace(/\s+/g, " ");
function rule(selector: string): string {
  const esc = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const m = css.match(new RegExp(`(?:^|})\\s*${esc}\\s*\\{([^}]*)\\}`));
  return m ? m[1] : "";
}

test("focus rings are a true 2px --ink ring for planets and houses", () => {
  for (const sel of [".chart-planet:focus-visible .chart-planet-bg", ".chart-house:focus-visible .chart-cell"]) {
    const r = rule(sel);
    expect(r, sel).toMatch(/stroke: var\(--ink\)/);
    expect(r, sel).toMatch(/stroke-width: 2px/);
    expect(r, sel).toMatch(/vector-effect: non-scaling-stroke/);
  }
});

test("chart text uses tokens, --ink-strong signs and tabular numerals; highlight is not colour-only", () => {
  expect(rule(".chart-sign")).toMatch(/fill: var\(--ink-strong\)/);
  expect(css).toMatch(/\.chart-text, \.chart-sign \{[^}]*font-variant-numeric: tabular-nums/);
  expect(rule(".chart-house--highlight .chart-sign")).toMatch(/font-weight: var\(--fw-medium\)/);
  expect(rule(".chart-cell--lagna.chart-cell--highlight")).toMatch(/stroke-width/);
  expect(rule(".chart-planet--highlight .chart-planet-bg")).toMatch(/stroke: var\(--ink\)/);
});

test.each(STYLES)("%s without a lagna draws no lagna marker or fill and names houses by sign", (style) => {
  const { container } = render(<ChartSvg data={{ ...data, ascSign: null }} style={style} labels={labels} />);
  expect(container.querySelector(".chart-cell--lagna")).toBeNull();
  expect([...container.querySelectorAll("text.chart-sign")].map((t) => t.textContent)).not.toContain("As");
  expect(screen.queryByRole("button", { name: /^House / })).toBeNull();
  expect(container.querySelector('[data-house="1"]')).toHaveAttribute("data-sign", "0");
  expect(container.querySelector('[data-house="1"]')).toHaveAttribute("aria-label", "Ar");
});

test("transit labels share the house layout, are parenthesised and not interactive", () => {
  const data: ChartData = {
    ascSign: 0,
    planets: [{ id: "Sun", sign: 0, degInSign: 10, retrograde: false }],
    transits: [
      { id: "Saturn", sign: 0, degInSign: 3, retrograde: true },
      { id: "Moon", sign: 5, degInSign: 20, retrograde: false },
    ],
  };
  const { container } = render(<ChartSvg data={data} style="north" labels={labels} />);
  const transits = [...container.querySelectorAll(".chart-transit")];
  expect(transits.map((t) => t.textContent)).toEqual([`(${labels.graha.Saturn}R)`, `(${labels.graha.Moon})`]);
  for (const t of transits) {
    expect(t.closest("[role=button]")).toBeNull();
    expect(t).toHaveAttribute("aria-hidden", "true");
  }
  // Natal ve transit etiketi aynı hücrede çakışmaz.
  const sun = container.querySelector('[data-graha="Sun"]')!;
  const sat = container.querySelector('[data-transit="Saturn"]')!;
  expect(`${sun.getAttribute("data-x")},${sun.getAttribute("data-y")}`).not.toBe(`${sat.getAttribute("x")},${sat.getAttribute("y")}`);
});

test("without transits the drawing has no transit labels", () => {
  const { container } = render(<ChartSvg data={{ ascSign: 0, planets: [] }} style="south" labels={labels} />);
  expect(container.querySelector(".chart-transit")).toBeNull();
});
