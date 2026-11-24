import { act, fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    planetLongitudes: vi.fn(async () => ({
      status: "ok",
      data: [
        { body: "Sun", lon: 256.5, retrograde: false },
        { body: "Saturn", lon: 10.25, retrograde: true },
        { body: "Mars", lon: null, retrograde: false },
      ],
    })),
  },
}));

import { api } from "../api";
import { HighlightProvider } from "../highlight/store";
import { setLanguage } from "../i18n";
import { tr } from "../i18n/tr";
import { PANEL_KINDS, type Panel } from "../layout/model";
import { ChartPanel } from "./ChartPanel";
import { GrahaTablePanel } from "./GrahaTablePanel";
import { panelDefinition, PANEL_REGISTRY } from "./registry";
import { SkyProvider } from "./sky";
import { PanelStatusContext } from "./states";
import { UnavailablePanel } from "./UnavailablePanel";

const chartPanel = (settings: Record<string, unknown> = { varga: "D1", style: "north" }): Panel => ({
  id: "c",
  kind: "chart",
  x: 0,
  y: 0,
  w: 6,
  h: 12,
  settings,
});
const tablePanel: Panel = { id: "t", kind: "graha_table", x: 6, y: 0, w: 6, h: 8, settings: {} };

function Providers({ children }: { children: ReactNode }) {
  return (
    <HighlightProvider>
      <SkyProvider now={() => 0}>{children}</SkyProvider>
    </HighlightProvider>
  );
}

beforeEach(() => vi.mocked(api.planetLongitudes).mockClear());
afterEach(() => vi.useRealTimers());

test("registry covers the seven kinds; only chart and graha_table are implemented here", () => {
  expect(Object.keys(PANEL_REGISTRY).sort()).toEqual([...PANEL_KINDS].sort());
  expect(PANEL_REGISTRY.chart.component).toBe(ChartPanel);
  expect(PANEL_REGISTRY.graha_table.component).toBe(GrahaTablePanel);
  for (const kind of ["dasha_tree", "dasha_strip", "transit_table", "notes", "birth_info"] as const) {
    expect(PANEL_REGISTRY[kind].component).toBe(UnavailablePanel);
  }
  expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
  const keys = new Set(Object.keys(tr.panel).map((k) => `panel.${k}`));
  for (const kind of [...PANEL_KINDS, "yoga_list"]) expect(keys.has(panelDefinition(kind).titleKey)).toBe(true);
  expect(PANEL_REGISTRY.chart.subtitle!(chartPanel({ varga: "D9" }))).toBe("D9");
});

test("chart panel draws the current sky with the no-lagna notice, fetching once for jd(now)", async () => {
  render(
    <Providers>
      <ChartPanel panel={chartPanel()} />
      <GrahaTablePanel panel={tablePanel} />
    </Providers>,
  );
  expect(await screen.findByRole("note")).toHaveTextContent("Lagna henüz hesaplanmıyor; şimdiki gökyüzü, 1. ev Koç");
  expect(screen.getByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Gü, Yay 16°" })).toBeInTheDocument();
  expect(api.planetLongitudes).toHaveBeenCalledTimes(1);
  expect(api.planetLongitudes).toHaveBeenCalledWith(2440587.5);
});

test("chart panel honours the style setting and falls back to north", async () => {
  const { container, unmount } = render(
    <Providers>
      <ChartPanel panel={chartPanel({ varga: "D1", style: "south" })} />
    </Providers>,
  );
  await screen.findByRole("group");
  expect(container.querySelector("svg")!.dataset.style).toBe("south");
  unmount();
  const r = render(
    <Providers>
      <ChartPanel panel={chartPanel({ style: "spiral" })} />
    </Providers>,
  );
  await r.findByRole("group");
  expect(r.container.querySelector("svg")!.dataset.style).toBe("north");
});

test("vargas other than D1 show an explicit later-milestone message", () => {
  render(
    <Providers>
      <ChartPanel panel={chartPanel({ varga: "D9" })} />
    </Providers>,
  );
  expect(screen.getByText("D9 haritası sonraki bir sürümde gelecek.")).toBeInTheDocument();
  expect(screen.queryByRole("group")).toBeNull();
});

test("graha table lists body, sign, degree with minutes and the retro marker", async () => {
  render(
    <Providers>
      <GrahaTablePanel panel={tablePanel} />
    </Providers>,
  );
  const sun = (await screen.findByRole("button", { name: "Güneş" })).closest("tr")!;
  expect(sun).toHaveTextContent("Yay");
  expect(sun).toHaveTextContent("16°30′");
  const saturn = screen.getByRole("button", { name: "Satürn" }).closest("tr")!;
  expect(saturn).toHaveTextContent("Koç");
  expect(saturn).toHaveTextContent("10°15′");
  expect(saturn).toHaveTextContent("℞");
  const mars = screen.getByRole("button", { name: "Mars" }).closest("tr")!;
  expect(mars.querySelectorAll("td")[0]).toHaveTextContent("—");
});

test("hovering a graha in the table highlights it in the chart, click pins, Esc clears", async () => {
  const { container } = render(
    <Providers>
      <ChartPanel panel={chartPanel()} />
      <GrahaTablePanel panel={tablePanel} />
    </Providers>,
  );
  const row = (await screen.findByRole("button", { name: "Satürn" })).closest("tr")!;
  const chartSaturn = () => container.querySelector('svg [data-graha="Saturn"]')!;
  fireEvent.mouseEnter(row);
  expect(chartSaturn()).toHaveClass("chart-planet--highlight");
  expect(row).toHaveClass("is-highlighted");
  fireEvent.mouseLeave(row);
  expect(chartSaturn()).not.toHaveClass("chart-planet--highlight");

  fireEvent.click(chartSaturn());
  expect(row).toHaveClass("is-highlighted");
  expect(screen.getByRole("button", { name: "Satürn" })).toHaveAttribute("aria-pressed", "true");
  fireEvent.keyDown(window, { key: "Escape" });
  expect(row).not.toHaveClass("is-highlighted");
});

test("an empty result shows the empty state", async () => {
  vi.mocked(api.planetLongitudes).mockResolvedValueOnce({ status: "ok", data: [] });
  render(
    <Providers>
      <GrahaTablePanel panel={tablePanel} />
    </Providers>,
  );
  expect(await screen.findByText("Gösterilecek graha yok.")).toBeInTheDocument();
});

test("errors are shown in the panel and retry fetches again", async () => {
  vi.mocked(api.planetLongitudes).mockResolvedValueOnce({ status: "error", error: { code: "engine_stopped", params: {} } });
  render(
    <Providers>
      <GrahaTablePanel panel={tablePanel} />
    </Providers>,
  );
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap motoru durdu.");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  expect(await screen.findByRole("button", { name: "Güneş" })).toBeInTheDocument();
  expect(api.planetLongitudes).toHaveBeenCalledTimes(2);
});

test("a slow load shows a skeleton after one second", async () => {
  vi.useFakeTimers();
  vi.mocked(api.planetLongitudes).mockReturnValueOnce(new Promise(() => {}));
  render(
    <Providers>
      <ChartPanel panel={chartPanel()} />
    </Providers>,
  );
  expect(screen.queryByLabelText("Yükleniyor")).toBeNull();
  act(() => vi.advanceTimersByTime(1000));
  expect(screen.getByLabelText("Yükleniyor")).toHaveAttribute("aria-busy", "true");
});

test("unavailable and unknown panels explain themselves without an error", () => {
  render(<UnavailablePanel panel={{ ...tablePanel, kind: "dasha_tree" }} />);
  expect(screen.getByText("Bu panel sonraki bir sürümde gelecek.")).toBeInTheDocument();
  render(<UnavailablePanel panel={{ ...tablePanel, kind: "yoga_list" }} />);
  expect(screen.getByText("“yoga_list” paneli bu sürümde tanınmıyor. Düzen olduğu gibi korunuyor.")).toBeInTheDocument();
  expect(screen.queryByRole("alert")).toBeNull();
});

test("labels follow the selected language", async () => {
  await setLanguage("en");
  render(
    <Providers>
      <ChartPanel panel={chartPanel()} />
      <GrahaTablePanel panel={tablePanel} />
    </Providers>,
  );
  expect(await screen.findByRole("button", { name: "Su, Sg 16°" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Sun" }).closest("tr")).toHaveTextContent("Sg");
});

test("chart aria-pressed follows the pin only: focus hovers, Enter pins, Enter again unpins", async () => {
  render(
    <Providers>
      <ChartPanel panel={chartPanel()} />
    </Providers>,
  );
  const planet = await screen.findByRole("button", { name: "Gü, Yay 16°" });
  act(() => planet.focus());
  expect(planet).toHaveAttribute("aria-pressed", "false");
  fireEvent.keyDown(planet, { key: "Enter" });
  expect(planet).toHaveAttribute("aria-pressed", "true");
  fireEvent.keyDown(planet, { key: "Enter" });
  expect(planet).toHaveAttribute("aria-pressed", "false");
});

test("a non-D1 chart reports no loading phase to the panel frame", () => {
  vi.useFakeTimers();
  vi.mocked(api.planetLongitudes).mockReturnValueOnce(new Promise(() => {}));
  const reported: string[] = [];
  render(
    <PanelStatusContext.Provider value={(p) => reported.push(p)}>
      <Providers>
        <ChartPanel panel={chartPanel({ varga: "D9" })} />
      </Providers>
    </PanelStatusContext.Provider>,
  );
  act(() => vi.advanceTimersByTime(2000));
  expect(reported.every((p) => p === "none")).toBe(true);
});

test("unmounting SkyProvider before the load resolves neither sets state nor warns", async () => {
  let resolve!: (v: Awaited<ReturnType<typeof api.planetLongitudes>>) => void;
  vi.mocked(api.planetLongitudes).mockReturnValueOnce(new Promise((r) => (resolve = r)));
  const spy = vi.spyOn(console, "error").mockImplementation(() => {});
  const { unmount } = render(
    <SkyProvider now={() => 0}>
      <span />
    </SkyProvider>,
  );
  unmount();
  await act(async () => {
    resolve({ status: "ok", data: [] });
  });
  expect(spy).not.toHaveBeenCalled();
  spy.mockRestore();
});
