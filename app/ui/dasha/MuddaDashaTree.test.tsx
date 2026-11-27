import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeVarsha: vi.fn(), saveVarshaChart: vi.fn(), dashaChildren: vi.fn(), computeCharaDasha: vi.fn(), charaDashaChildren: vi.fn() } }));

import { api, type Result } from "../api";
import type { VarshaView } from "../bindings";
import { i18n } from "../i18n";
import type { Panel } from "../layout/model";
import { PANEL_REGISTRY } from "../panels/registry";
import { VarshaPanel } from "../panels/VarshaPanel";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { resetVarshaAge } from "../varsha/yearStore";
import { useLayoutStore } from "../view/store";
import { DashaTreePanel } from "./DashaTreePanel";
import { TREE_SYSTEMS, treeSystem } from "./muddaSystem";
import { dashaSystem } from "./system";

const FX = fixture();
const ok = (view: VarshaView = fixture().varsha.current): Result<VarshaView> => ({ status: "ok", data: view });
const failure = { status: "error" as const, error: { code: "calc", params: { detail: "x" } } };
const DELHI = { lat: 28.6139, lon: 77.209, tz: { Iana: "Asia/Kolkata" } };
const LAYOUT = (settings: Record<string, unknown>, withVarsha = false) => {
  const panels: Panel[] = [{ id: "dt", kind: "dasha_tree", x: 0, y: 0, w: 4, h: 12, settings }];
  if (withVarsha) panels.push({ id: "vp", kind: "varsha", x: 4, y: 0, w: 7, h: 24, settings: {} });
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels }] };
};

/** Panelleri düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanels() {
  const { active } = useLayoutStore();
  return (
    <>
      <section aria-label="tree">
        <DashaTreePanel panel={active.panels[0]} />
      </section>
      {active.panels[1] && (
        <section aria-label="varsha">
          <VarshaPanel panel={active.panels[1]} />
        </section>
      )}
    </>
  );
}

function renderTree(settings: Record<string, unknown> = { system: "mudda" }, { opened = fixture().opened, withVarsha = false } = {}) {
  return renderWithStores(<LivePanels />, { layout: LAYOUT(settings, withVarsha), opened });
}

const tree = () => screen.findByRole("tree", { name: "Mudda dasa" });
const items = (root: HTMLElement, level: number) => within(root).queryAllByRole("treeitem").filter((i) => i.getAttribute("aria-level") === String(level));
const click = (item: HTMLElement) => fireEvent.click(item.querySelector(".dasha-row")!);
const savedSettings = (persistence: ReturnType<typeof renderTree>["persistence"]) =>
  persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings;

beforeEach(() => {
  resetVarshaAge();
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.computeVarsha).mockImplementation(async (_input, age) => ok(age === 27 ? fixture().varsha.next : fixture().varsha.current));
});

test("the tree's system list offers the mudda dasha and the choice is saved with the layout", async () => {
  const { persistence } = renderTree({});
  const select = screen.getByRole("combobox", { name: "Sistem" });
  expect(within(select).getAllByRole("option").map((o) => o.textContent)).toEqual(["Vimshottari", "Chara dasha", "Mudda dasa"]);
  expect(select).toHaveValue("vimshottari");
  // Mudda dasa seçilmeden rapor istenmez (K1).
  expect(api.computeVarsha).not.toHaveBeenCalled();
  fireEvent.change(select, { target: { value: "mudda" } });
  await tree();
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ system: "mudda" }));
  expect(api.computeVarsha).toHaveBeenCalledTimes(1);
  // Sistem adı panelin erişilebilir adına eklenir; dasa şeridi mudda dasayı tanımaz.
  expect(PANEL_REGISTRY.dasha_tree.nameDetail?.({ id: "dt", kind: "dasha_tree", x: 0, y: 0, w: 4, h: 12, settings: { system: "mudda" } }, i18n.t)).toBe("Mudda dasa");
  expect(TREE_SYSTEMS).toEqual(["vimshottari", "chara", "mudda"]);
  expect(treeSystem({ system: "chara" })).toBe("chara");
  expect(treeSystem({ system: "other" })).toBe("vimshottari");
  expect(dashaSystem({ system: "mudda" })).toBe("vimshottari");
});

test("lists the nine periods of the running year from Rust, with the first-lord line and the unverified note", async () => {
  renderTree();
  const root = await tree();
  const opened = fixture().opened;
  // Yaş gönderilmez (süren yılı Rust bulur); dönemler yere bağlı değildir: doğum yeri gider.
  expect(api.computeVarsha).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null }, null, DELHI);
  const top = items(root, 1);
  // Delhi fikstürü: Swati (15), tamamlanan yıl 26 → artan 3: Mars. Sıra Mars'tan sürer.
  expect(top.map((i) => i.querySelector(".dasha-lord")?.textContent)).toEqual(["Mars", "Rahu", "Jüpiter", "Satürn", "Merkür", "Ketu", "Venüs", "Güneş", "Ay"]);
  expect(top[0]).toHaveAccessibleName("Mars, 2026-01-01 09:28 – 2026-01-22 16:50");
  expect(top[1]).toHaveAccessibleName("Rahu, 2026-01-22 16:50 – 2026-03-18 11:46");
  expect(top[8].querySelector(".dasha-dates")?.textContent).toMatch(/ – 2027-01-01 15:41$/);
  expect(top[0]).toHaveAttribute("aria-expanded", "false");
  expect(items(root, 2)).toHaveLength(0);
  expect(screen.getByText("27. yıl (tamamlanan yıl 26): 2026-01-01 09:28 – 2027-01-01 15:41 (UTC+05:30)")).toBeInTheDocument();
  expect(screen.getByText("İlk lord Mars: doğum nakshatrası 15 (Swati), tamamlanan yıl 26, artan 3.")).toBeInTheDocument();
  expect(screen.getByText("Doğrulanmamış tablo: mudda dasanın ilk lord kuralı ve dönem payları basılı kaynakla karşılaştırılmayı bekliyor.")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: varsha.sidereal_return_v1")).toBeInTheDocument();
  expect(screen.queryByText(/Doğum saati bilinmiyor/)).toBeNull();
  expect(screen.getByRole("spinbutton", { name: "Yaş (tamamlanan yıl)" })).toHaveValue(26);
});

test("a period opens its nine sub-periods; one path is open at a time; the keyboard works", async () => {
  renderTree();
  const root = await tree();
  click(items(root, 1)[0]);
  let kids = items(root, 2);
  expect(kids).toHaveLength(9);
  // Alt dönemler üst dönemin lorduyla başlar ve üst dönemin bitişinde biter.
  expect(kids[0]).toHaveAccessibleName("Mars–Mars, 2026-01-01 09:28 – 2026-01-02 15:18");
  expect(kids[8]).toHaveAccessibleName("Mars–Ay, 2026-01-20 22:13 – 2026-01-22 16:50");
  expect(kids.every((k) => !k.hasAttribute("aria-expanded"))).toBe(true);
  expect(items(root, 1)[0]).toHaveAttribute("aria-expanded", "true");
  // Başka bir dönemi açmak öncekini kapatır.
  click(items(root, 1)[1]);
  kids = items(root, 2);
  expect(kids).toHaveLength(9);
  expect(kids[0]).toHaveAccessibleName("Rahu–Rahu, 2026-01-22 16:50 – 2026-01-30 22:04");
  expect(items(root, 1)[0]).toHaveAttribute("aria-expanded", "false");
  // Klavye: sol ok kapatır, sağ ok açar.
  const rahu = items(root, 1)[1];
  rahu.focus();
  fireEvent.keyDown(rahu, { key: "ArrowLeft" });
  expect(items(root, 2)).toHaveLength(0);
  fireEvent.keyDown(rahu, { key: "ArrowRight" });
  expect(items(root, 2)).toHaveLength(9);
  // Alt dönemler istekle yüklenmez: rapor bir kez istenmiştir.
  expect(api.computeVarsha).toHaveBeenCalledTimes(1);
});

test("the year is shared with the Varshaphala panel: stepping it in one moves both", async () => {
  renderTree({ system: "mudda" }, { withVarsha: true });
  await tree();
  const panel = screen.getByRole("region", { name: "varsha" });
  await within(panel).findByRole("table", { name: "Varshaphala: yılın olguları" });
  // Aynı girdi: iki panel aynı raporu ayrı ayrı ister (doğum yeri, yaşsız).
  expect(vi.mocked(api.computeVarsha).mock.calls.map((c) => c[1])).toEqual([null, null]);
  fireEvent.click(within(panel).getByRole("button", { name: "Sonraki yıl" }));
  // 27 yaşı: 15 + 27 − 2 = 40, artan 4: Rahu.
  await waitFor(() => expect(items(screen.getByRole("tree", { name: "Mudda dasa" }), 1)[0]).toHaveAccessibleName("Rahu, 2027-01-01 15:41 – 2027-02-25 10:35"));
  expect(screen.getByText("İlk lord Rahu: doğum nakshatrası 15 (Swati), tamamlanan yıl 27, artan 4.")).toBeInTheDocument();
  for (const input of screen.getAllByRole("spinbutton", { name: "Yaş (tamamlanan yıl)" })) expect(input).toHaveValue(27);
  // Ağaçtaki "Bu yıl" iki paneli de süren yıla döndürür.
  fireEvent.click(within(screen.getByRole("region", { name: "tree" })).getByRole("button", { name: "Bu yıl" }));
  await waitFor(() => expect(within(panel).getByRole("spinbutton", { name: "Yaş (tamamlanan yıl)" })).toHaveValue(26));
  for (const button of screen.getAllByRole("button", { name: "Bu yıl" })) expect(button).toHaveAttribute("aria-pressed", "true");
});

test("unknown birth time: the periods are listed and marked approximate", async () => {
  vi.mocked(api.computeVarsha).mockResolvedValue(ok(fixture().varsha.time_unknown));
  renderTree({ system: "mudda" }, { opened: unknownTimeOpened() });
  const root = await tree();
  expect(vi.mocked(api.computeVarsha).mock.calls[0][0].time_known).toBe(false);
  expect(items(root, 1)).toHaveLength(9);
  expect(screen.getByText("Doğum saati bilinmiyor: dönüş anı, dönem tarihleri ve ilk lord yaklaşıktır (≈).")).toBeInTheDocument();
});

test("an error is shown with the year controls, and retry asks again", async () => {
  vi.mocked(api.computeVarsha).mockResolvedValueOnce(failure);
  renderTree();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.queryByRole("tree")).toBeNull();
  expect(screen.getByRole("button", { name: "Bu yıl" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await tree();
  expect(api.computeVarsha).toHaveBeenCalledTimes(2);
});

test("loading: nothing under a second, then a skeleton, then the tree", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<VarshaView>) => void;
    vi.mocked(api.computeVarsha).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderTree();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getByRole("tree", { name: "Mudda dasa" })).toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
});

test("the fixture's mudda order is Rust's and the nine top periods follow it cyclically", () => {
  const { mudda } = FX.varsha.current.report;
  const order = FX.varsha.mudda_order;
  expect(order).toEqual(["Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury", "Ketu", "Venus"]);
  const top = mudda.periods.filter((p) => p.level === 1).map((p) => p.lords[0]);
  const first = order.indexOf(mudda.first_lord);
  expect(top).toEqual(order.map((_, k) => order[(first + k) % 9]));
  expect(mudda.periods).toHaveLength(90);
});
