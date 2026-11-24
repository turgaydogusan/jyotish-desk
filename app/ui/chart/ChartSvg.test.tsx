import { fireEvent, render, screen } from "@testing-library/react";
import { ChartSvg } from "./ChartSvg";
import type { ChartData, ChartLabels, ChartStyle } from "./types";

const labels: ChartLabels = {
  title: "Chart",
  graha: { Sun: "Su", Moon: "Mo", Mars: "Ma", Mercury: "Me", Jupiter: "Ju", Venus: "Ve", Saturn: "Sa", Rahu: "Ra", Ketu: "Ke" },
  signs: ["Ar", "Ta", "Ge", "Cn", "Le", "Vi", "Li", "Sc", "Sg", "Cp", "Aq", "Pi"],
  lagna: "As",
  house: (house, sign) => `House ${house}, ${sign}`,
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

test.each(STYLES)("%s renders an accessible image with 12 houses", (style) => {
  const { container } = render(<ChartSvg data={data} style={style} labels={labels} />);
  expect(screen.getByRole("img", { name: "Chart" })).toBeInTheDocument();
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
  expect(lagna[0].getAttribute("aria-label")).toBe("House 1, Le");
});

test("north shows sign numbers; south and circle show the lagna marker instead", () => {
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

test("markers: retro ℞, combust c, vargottama underline — no colour coding", () => {
  render(<ChartSvg data={data} style="north" labels={labels} />);
  const mercury = screen.getByRole("button", { name: "Me ℞ c 20°" });
  expect(mercury).toHaveTextContent("Me℞c");
  const moon = screen.getByRole("button", { name: "Mo 3°" });
  expect(moon.querySelector("text")).toHaveAttribute("text-decoration", "underline");
  expect(screen.getByRole("button", { name: "Su 12°" }).querySelector("text")).not.toHaveAttribute("text-decoration");
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
  const sun = screen.getByRole("button", { name: "Su 12°" });
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

test("highlighted graha and house get highlight classes", () => {
  const { container, rerender } = render(
    <ChartSvg data={data} style="north" labels={labels} highlight={{ type: "graha", id: "Saturn" }} />,
  );
  expect(container.querySelector('[data-graha="Saturn"]')).toHaveClass("chart-planet--highlight");
  expect(screen.getByRole("button", { name: "Sa ℞ 1°" })).toHaveAttribute("aria-pressed", "true");
  rerender(<ChartSvg data={data} style="north" labels={labels} highlight={{ type: "sign", sign: 10 }} />);
  expect(container.querySelector('g[data-house="7"] path')).toHaveClass("chart-cell--highlight");
});
