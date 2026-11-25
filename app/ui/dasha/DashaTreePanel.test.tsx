import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("../api", () => ({ api: { dashaChildren: vi.fn() } }));

import { api } from "../api";
import type { Graha, OpenedClient } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { useSession } from "../session/store";
import { renderWithStores } from "../test/render";
import { DashaTreePanel } from "./DashaTreePanel";

const panel: Panel = { id: "d", kind: "dasha_tree", x: 0, y: 0, w: 4, h: 8, settings: {} };

function renderTree(path: Graha[] = [], opened = fixture().opened) {
  opened.view_state.dasha_expanded_path = path;
  return renderWithStores(<DashaTreePanel panel={panel} />, { opened });
}

const tree = () => screen.getByRole("tree", { name: "Dasa ağacı" });
const item = (name: RegExp) => within(tree()).getByRole("treeitem", { name });
const click = (name: RegExp) => fireEvent.click(item(name).querySelector(".dasha-row")!);
const level = (n: number) => within(tree()).queryAllByRole("treeitem").filter((i) => i.getAttribute("aria-level") === String(n));

beforeEach(() => {
  vi.mocked(api.dashaChildren).mockReset();
  vi.mocked(api.dashaChildren).mockImplementation(async () => ({ status: "ok", data: fixture().dasha_children.periods }));
});
afterEach(() => vi.useRealTimers());

test("lists the nine mahadashas with dates in the birth's offset", () => {
  renderTree();
  const mahas = level(1);
  expect(mahas).toHaveLength(9);
  expect(mahas[0]).toHaveAccessibleName("Rahu, 1982-09-19 – 2000-09-18");
  expect(mahas[0]).toHaveAttribute("aria-expanded", "false");
  expect(mahas[0]).toHaveAttribute("aria-posinset", "1");
  expect(mahas[0]).toHaveAttribute("aria-setsize", "9");
  expect(mahas[0]).toHaveAttribute("tabindex", "0");
  expect(mahas[1]).toHaveAttribute("tabindex", "-1");
});

test("one open path: opening a sibling closes the other, and the path is saved with the chart", async () => {
  const { persistence } = renderTree();
  click(/^Jüpiter, /);
  expect(level(2)).toHaveLength(9);
  click(/^Jüpiter–Satürn, /);
  expect(level(3)).toHaveLength(9);
  click(/^Satürn, /);
  expect(level(2).map((i) => i.getAttribute("aria-label")!.split(",")[0])[0]).toBe("Satürn–Satürn");
  expect(level(3)).toHaveLength(0);
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.dasha_expanded_path).toEqual(["Saturn"]));
  expect(persistence.saved.at(-1)!.chartId).toBe(fixture().opened.chart_record.id);
  click(/^Satürn, /);
  expect(level(2)).toHaveLength(0);
});

test("keyboard follows the tree pattern", () => {
  renderTree();
  const first = level(1)[0];
  act(() => first.focus());
  fireEvent.keyDown(first, { key: "ArrowDown" });
  expect(document.activeElement).toHaveAccessibleName(/^Jüpiter, /);
  fireEvent.keyDown(document.activeElement!, { key: "ArrowRight" });
  expect(item(/^Jüpiter, /)).toHaveAttribute("aria-expanded", "true");
  fireEvent.keyDown(document.activeElement!, { key: "ArrowRight" });
  expect(document.activeElement).toHaveAccessibleName(/^Jüpiter–Jüpiter, /);
  fireEvent.keyDown(document.activeElement!, { key: "ArrowLeft" });
  expect(document.activeElement).toHaveAccessibleName(/^Jüpiter, /);
  fireEvent.keyDown(document.activeElement!, { key: "ArrowLeft" });
  expect(item(/^Jüpiter, /)).toHaveAttribute("aria-expanded", "false");
  fireEvent.keyDown(document.activeElement!, { key: "End" });
  expect(document.activeElement).toHaveAccessibleName(/^Mars, /);
  fireEvent.keyDown(document.activeElement!, { key: "Enter" });
  expect(item(/^Mars, /)).toHaveAttribute("aria-expanded", "true");
  fireEvent.keyDown(document.activeElement!, { key: "Home" });
  expect(document.activeElement).toHaveAccessibleName(/^Rahu, /);
});

test("levels 4 and 5 load on demand and are cached per chart", async () => {
  const { seed } = fixture().opened.chart.dasha;
  renderTree(["Rahu", "Rahu", "Rahu"]);
  expect(await within(tree()).findAllByRole("treeitem", { name: /^Rahu–Rahu–Rahu–/ })).toHaveLength(9);
  expect(api.dashaChildren).toHaveBeenCalledTimes(1);
  expect(api.dashaChildren).toHaveBeenCalledWith(seed, ["Rahu", "Rahu", "Rahu"]);
  expect(level(4)).toHaveLength(9);
  click(/^Rahu–Rahu–Rahu, /);
  click(/^Rahu–Rahu–Rahu, /);
  expect(level(4)).toHaveLength(9);
  expect(api.dashaChildren).toHaveBeenCalledTimes(1);
});

test("editing the birth data drops lazily loaded levels", async () => {
  let open: (o: OpenedClient) => void = () => {};
  function Grab() {
    open = useSession().open;
    return null;
  }
  const opened = fixture().opened;
  opened.view_state.dasha_expanded_path = ["Rahu", "Rahu", "Rahu"];
  renderWithStores(
    <>
      <Grab />
      <DashaTreePanel panel={panel} />
    </>,
    { opened },
  );
  await waitFor(() => expect(level(4)).toHaveLength(9));
  // `update_chart`: aynı harita kimliği, yeni doğum anı (yeni tohum). Yeni yanıt bekletilir.
  vi.mocked(api.dashaChildren).mockImplementation(() => new Promise(() => {}));
  const edited = fixture().opened;
  edited.view_state.dasha_expanded_path = ["Rahu", "Rahu", "Rahu"];
  edited.chart.dasha.seed.birth_jd_ut = edited.chart.dasha.seed.birth_jd_ut! + 0.5;
  act(() => open(edited));
  expect(level(4)).toHaveLength(0);
  expect(api.dashaChildren).toHaveBeenCalledTimes(2);
  expect(api.dashaChildren).toHaveBeenLastCalledWith(edited.chart.dasha.seed, ["Rahu", "Rahu", "Rahu"]);
});

test("keyboard reaches the lazily loaded level", async () => {
  renderTree(["Rahu", "Rahu", "Rahu"]);
  await waitFor(() => expect(level(4)).toHaveLength(9));
  const l3 = item(/^Rahu–Rahu–Rahu, /);
  act(() => l3.focus());
  fireEvent.keyDown(l3, { key: "ArrowRight" });
  expect(document.activeElement).toHaveAttribute("aria-level", "4");
  fireEvent.keyDown(document.activeElement!, { key: "ArrowDown" });
  expect(document.activeElement).toHaveAccessibleName(/^Rahu–Rahu–Rahu–Jüpiter, /);
});

test("a response that is not one level below the requested path is ignored", async () => {
  // Her yola aynı 4. seviye listesini dönen bozuk arka uç: 5. seviye isteği boş görünür.
  renderTree(["Rahu", "Rahu", "Rahu", "Rahu"]);
  await waitFor(() => expect(api.dashaChildren).toHaveBeenCalledTimes(2));
  await waitFor(() => expect(level(4)).toHaveLength(9));
  expect(level(5)).toHaveLength(0);
});

test("a failed level-4 load shows an in-tree error with retry", async () => {
  vi.mocked(api.dashaChildren).mockResolvedValueOnce({ status: "error", error: { code: "invalid_dasha_input", params: { field: "path" } } });
  renderTree(["Rahu", "Rahu", "Rahu"]);
  expect(await screen.findByRole("alert")).toHaveTextContent("Dasa isteği geçersiz: path.");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(level(4)).toHaveLength(9));
  expect(api.dashaChildren).toHaveBeenCalledTimes(2);
});

test("a restored path opens immediately; unknown time adds an approximate note", () => {
  renderTree(["Jupiter", "Saturn"], unknownTimeOpened());
  expect(item(/^Jüpiter–Satürn, /)).toHaveAttribute("aria-expanded", "true");
  expect(level(3)).toHaveLength(9);
  expect(screen.getByRole("note")).toHaveTextContent("dasa tarihleri yaklaşıktır (≈)");
});

test("without an open chart the panel says so", () => {
  renderWithStores(<DashaTreePanel panel={panel} />, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
});
