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
const tabStops = () => within(tree()).getAllByRole("treeitem").filter((i) => i.getAttribute("tabindex") === "0");

/** Ağacı çizer ve oturumun `open`'ını verir (başka harita açmak ya da `update_chart` dönüşü için). */
function renderWithOpen(path: Graha[]) {
  let open: (o: OpenedClient) => void = () => {};
  function Grab() {
    open = useSession().open;
    return null;
  }
  const opened = fixture().opened;
  opened.view_state.dasha_expanded_path = path;
  renderWithStores(
    <>
      <Grab />
      <DashaTreePanel panel={panel} />
    </>,
    { opened },
  );
  return (o: OpenedClient) => act(() => open(o));
}

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
  const open = renderWithOpen(["Rahu", "Rahu", "Rahu"]);
  await waitFor(() => expect(level(4)).toHaveLength(9));
  // `update_chart`: aynı harita kimliği, yeni doğum anı (yeni tohum). Yeni yanıt bekletilir.
  vi.mocked(api.dashaChildren).mockImplementation(() => new Promise(() => {}));
  const edited = fixture().opened;
  edited.view_state.dasha_expanded_path = ["Rahu", "Rahu", "Rahu"];
  edited.chart.dasha.seed.birth_jd_ut = edited.chart.dasha.seed.birth_jd_ut! + 0.5;
  open(edited);
  expect(level(4)).toHaveLength(0);
  expect(api.dashaChildren).toHaveBeenCalledTimes(2);
  expect(api.dashaChildren).toHaveBeenLastCalledWith(edited.chart.dasha.seed, ["Rahu", "Rahu", "Rahu"]);
});

test("a late response for the old birth data is dropped", async () => {
  type Reply = Awaited<ReturnType<typeof api.dashaChildren>>;
  let resolveOld: (r: Reply) => void = () => {};
  vi.mocked(api.dashaChildren).mockImplementation(() => new Promise(() => {}));
  vi.mocked(api.dashaChildren).mockImplementationOnce(() => new Promise<Reply>((r) => (resolveOld = r)));
  const open = renderWithOpen(["Rahu", "Rahu", "Rahu"]);
  expect(api.dashaChildren).toHaveBeenCalledTimes(1);
  const edited = fixture().opened;
  edited.view_state.dasha_expanded_path = ["Rahu", "Rahu", "Rahu"];
  edited.chart.dasha.seed.birth_jd_ut = edited.chart.dasha.seed.birth_jd_ut! + 0.5;
  open(edited);
  expect(api.dashaChildren).toHaveBeenCalledTimes(2);
  await act(async () => resolveOld({ status: "ok", data: fixture().dasha_children.periods }));
  expect(level(4)).toHaveLength(0);
  expect(within(tree()).getByText("Yükleniyor")).toBeInTheDocument();
});

test("the tab stop survives a focused item leaving the tree", () => {
  const open = renderWithOpen(["Jupiter", "Saturn"]);
  act(() => item(/^Jüpiter–Satürn–Merkür, /).focus());
  expect(tabStops()).toEqual([item(/^Jüpiter–Satürn–Merkür, /)]);
  // Başka harita, kapalı yol: odaklı 3. seviye öğe artık yok; durak görünür en yakın ataya geçer.
  const other = fixture().opened;
  other.chart_record.id += 1;
  other.view_state.dasha_expanded_path = [];
  open(other);
  expect(level(2)).toHaveLength(0);
  expect(tabStops()).toEqual([item(/^Jüpiter, /)]);
  // Durak klavyeyle kullanılabilir: oklar oradan devam eder.
  act(() => tabStops()[0].focus());
  fireEvent.keyDown(document.activeElement!, { key: "ArrowDown" });
  expect(document.activeElement).toHaveAccessibleName(/^Satürn, /);
  expect(tabStops()).toEqual([item(/^Satürn, /)]);
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

test("the retry button keeps its own keys: the tree neither collapses nor moves focus", async () => {
  vi.mocked(api.dashaChildren).mockResolvedValueOnce({ status: "error", error: { code: "invalid_dasha_input", params: { field: "path" } } });
  renderTree(["Rahu", "Rahu", "Rahu"]);
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  act(() => retry.focus());
  // `fireEvent` varsayılan engellenmediyse `true` döner: Enter ve boşluk düğmeyi etkinleştirebilir.
  expect(fireEvent.keyDown(retry, { key: "Enter" })).toBe(true);
  expect(fireEvent.keyDown(retry, { key: " " })).toBe(true);
  expect(item(/^Rahu–Rahu–Rahu, /)).toHaveAttribute("aria-expanded", "true");
  for (const key of ["ArrowDown", "ArrowUp", "ArrowLeft", "ArrowRight", "Home", "End"]) expect(fireEvent.keyDown(retry, { key })).toBe(true);
  expect(document.activeElement).toBe(retry);
  expect(item(/^Rahu–Rahu–Rahu, /)).toHaveAttribute("aria-expanded", "true");
  // jsdom tuştan tıklama üretmez; tarayıcıda Enter'ın varsayılan eylemi budur.
  fireEvent.click(retry);
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

test("an empty lazily loaded level is an error with a retry, not a silent empty list (M6-6)", async () => {
  vi.mocked(api.dashaChildren).mockResolvedValueOnce({ status: "ok", data: [] });
  const opened = fixture().opened;
  opened.view_state.dasha_expanded_path = fixture().dasha_children.path;
  renderWithStores(<DashaTreePanel panel={panel} />, { opened });
  expect(await screen.findByText("Bu dönemin alt dönemleri alınamadı.")).toBeInTheDocument();
  vi.mocked(api.dashaChildren).mockResolvedValueOnce({ status: "ok", data: fixture().dasha_children.periods });
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(screen.queryByText("Bu dönemin alt dönemleri alınamadı.")).toBeNull());
});
