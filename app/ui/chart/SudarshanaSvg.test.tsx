import { fireEvent, render, screen } from "@testing-library/react";
import { SudarshanaSvg, type SudarshanaLabels, type SudarshanaRingData } from "./SudarshanaSvg";
import type { GrahaId } from "./types";

const labels: SudarshanaLabels = {
  title: "Sudarshana",
  graha: { Sun: "Su", Moon: "Mo", Mars: "Ma", Mercury: "Me", Jupiter: "Ju", Venus: "Ve", Saturn: "Sa", Rahu: "Ra", Ketu: "Ke" },
  lagna: "As",
};

/** Burç → grahalar; halka `first` burcundan başlar. */
const PLACED: Record<number, GrahaId[]> = { 0: ["Sun", "Mercury"], 1: ["Venus"], 3: ["Moon"], 4: ["Rahu"], 7: ["Mars"], 8: ["Jupiter"], 10: ["Saturn", "Ketu"] };
const ring = (first: number): SudarshanaRingData => ({
  houses: Array.from({ length: 12 }, (_, i) => ({ house: i + 1, sign: (first + i) % 12, grahas: PLACED[(first + i) % 12] ?? [] })),
});
/** Lagna Akrep (7), Ay Yengeç (3), Güneş Koç (0). */
const RINGS = [ring(7), ring(3), ring(0)];

test("draws three rings of twelve cells as one image with nothing to tab to", () => {
  const { container } = render(<SudarshanaSvg rings={RINGS} labels={labels} />);
  const svg = screen.getByRole("img", { name: "Sudarshana" });
  expect(svg).toHaveClass("chart", "sudarshana");
  expect(container.querySelectorAll("path.chart-cell")).toHaveLength(36);
  expect(container.querySelectorAll("[tabindex], [role='button']")).toHaveLength(0);
  expect([...svg.children].every((c) => c.getAttribute("aria-hidden") === "true")).toBe(true);
  // Her halkanın 1. evi dolguludur ve halkanın başlangıç burcunu taşır.
  const firsts = [...container.querySelectorAll('g[data-house="1"]:has(> .chart-cell--lagna)')];
  expect(firsts.map((g) => [g.getAttribute("data-ring"), g.getAttribute("data-sign")])).toEqual([
    ["0", "7"],
    ["1", "3"],
    ["2", "0"],
  ]);
  expect(container.querySelectorAll(".chart-cell--lagna")).toHaveLength(3);
  // Burç numarası 1…12 (Koç 1): lagna halkasının 1. evi Akrep (8).
  expect(container.querySelector('g[data-ring="0"][data-house="1"] .sudarshana-sign')).toHaveTextContent("8");
  expect(container.querySelector('g[data-ring="2"][data-house="12"] .sudarshana-sign')).toHaveTextContent("12");
});

test("every graha is written once in each ring, in the house counted from that ring's first sign", () => {
  const { container } = render(<SudarshanaSvg rings={RINGS} labels={labels} />);
  expect(container.querySelectorAll(".chart-planet")).toHaveLength(27);
  const house = (ringNo: number, graha: string) => container.querySelector(`.chart-planet[data-ring="${ringNo}"][data-graha="${graha}"]`)!.getAttribute("data-house");
  // Güneş Koç'ta: Akrep'ten 6., Yengeç'ten 10., Koç'tan 1. ev.
  expect([house(0, "Sun"), house(1, "Sun"), house(2, "Sun")]).toEqual(["6", "10", "1"]);
  // Ay Yengeç'te: 9., 1. ve 4. ev.
  expect([house(0, "Moon"), house(1, "Moon"), house(2, "Moon")]).toEqual(["9", "1", "4"]);
  expect(container.querySelector('.chart-planet[data-ring="1"][data-graha="Moon"] text')).toHaveTextContent("Mo");
  // Lagna işareti yalnızca iç halkanın 1. evindedir.
  const marks = container.querySelectorAll(".chart-mark");
  expect(marks).toHaveLength(1);
  expect(marks[0]).toHaveTextContent("As");
  expect([marks[0].getAttribute("data-ring"), marks[0].getAttribute("data-house")]).toEqual(["0", "1"]);
});

test("a ring without houses is not drawn", () => {
  const { container } = render(<SudarshanaSvg rings={[{ houses: null }, RINGS[1], RINGS[2]]} labels={labels} />);
  expect(container.querySelectorAll("path.chart-cell")).toHaveLength(24);
  expect(container.querySelector('[data-ring="0"]')).toBeNull();
  expect(container.querySelector(".chart-mark")).toBeNull();
  expect(container.querySelectorAll(".chart-planet")).toHaveLength(18);
  expect(container.querySelector(".sudarshana-hub")).not.toBeNull();
});

test("a sign, a house or a graha highlight marks its three cells or labels", () => {
  const { container, rerender } = render(<SudarshanaSvg rings={RINGS} labels={labels} highlight={{ type: "sign", sign: 10 }} />);
  const lit = () => [...container.querySelectorAll(".chart-cell--highlight")].map((p) => p.parentElement!).map((g) => [g.getAttribute("data-ring"), g.getAttribute("data-house")]);
  // Kova: Akrep'ten 4., Yengeç'ten 8., Koç'tan 11. ev.
  expect(lit()).toEqual([
    ["0", "4"],
    ["1", "8"],
    ["2", "11"],
  ]);
  // Ev vurgusu D1 evidir: lagna halkasının 4. evi Kova'dır; aynı üç hücre.
  rerender(<SudarshanaSvg rings={RINGS} labels={labels} highlight={{ type: "house", house: 4 }} />);
  expect(lit()).toEqual([
    ["0", "4"],
    ["1", "8"],
    ["2", "11"],
  ]);
  // Lagna halkası yoksa ev vurgusunun karşılığı yoktur.
  rerender(<SudarshanaSvg rings={[{ houses: null }, RINGS[1], RINGS[2]]} labels={labels} highlight={{ type: "house", house: 4 }} />);
  expect(lit()).toEqual([]);
  rerender(<SudarshanaSvg rings={RINGS} labels={labels} highlight={{ type: "graha", id: "Mars" }} />);
  expect(lit()).toEqual([]);
  expect([...container.querySelectorAll(".chart-planet--highlight")].map((g) => g.getAttribute("data-graha"))).toEqual(["Mars", "Mars", "Mars"]);
});

test("the mouse drives the shared highlight: hover, leave and click", () => {
  const onHover = vi.fn();
  const onPin = vi.fn();
  const { container } = render(<SudarshanaSvg rings={RINGS} labels={labels} onHover={onHover} onPin={onPin} />);
  const cell = container.querySelector('g[data-ring="1"][data-house="2"]')!;
  fireEvent.mouseEnter(cell);
  expect(onHover).toHaveBeenLastCalledWith({ type: "sign", sign: 4 });
  fireEvent.mouseLeave(cell);
  expect(onHover).toHaveBeenLastCalledWith(null);
  fireEvent.click(cell);
  expect(onPin).toHaveBeenLastCalledWith({ type: "sign", sign: 4 });
  const venus = container.querySelector('.chart-planet[data-ring="2"][data-graha="Venus"]')!;
  fireEvent.mouseEnter(venus);
  expect(onHover).toHaveBeenLastCalledWith({ type: "graha", id: "Venus" });
  fireEvent.click(venus);
  expect(onPin).toHaveBeenLastCalledWith({ type: "graha", id: "Venus" });
});
