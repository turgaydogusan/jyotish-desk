// Olay aramasının nakshatra lordu ve hazır seçimi (Plan 26). Ayrı dosyadır:
// `EventSearchPanel.test.tsx` eşzamanlı planlarla paylaşılır.
import { act, fireEvent, screen, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { eventSearch: vi.fn(), eventSearchCancel: vi.fn(), searchPlaces: vi.fn(), setAppPrefs: vi.fn() } }));

import { api, type Result } from "../api";
import type { EventSearchView, SkyEventView } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { EventSearchPanel } from "./EventSearchPanel";

const FX = fixture();
const VIEW = FX.search.events;
const ok = (view: EventSearchView): Result<EventSearchView> => ({ status: "ok", data: view });
/** Fikstürün ilk olayının anı ve ofsetiyle, verilen türde ve nakshatrada bir satır. */
const row = (kind: SkyEventView["event"]["kind"], grahas: SkyEventView["event"]["grahas"], nakshatra: number | null): SkyEventView => ({
  ...VIEW.events[0],
  event: { ...VIEW.events[0].event, kind, grahas, sign: null, nakshatra, house: null },
});

function LivePanel() {
  const { active } = useLayoutStore();
  return <EventSearchPanel panel={active.panels[0]} />;
}
function renderPanel(settings: Record<string, unknown> = {}) {
  const panel: Panel = { id: "es", kind: "event_search", x: 0, y: 0, w: 7, h: 18, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<LivePanel />, { layout, opened: fixture().opened, prefs: FX.prefs.prefs });
}
const run = () => screen.getByRole("button", { name: "Ara" });
const settle = () => act(async () => {});
const where = () => within(screen.getByRole("table", { name: "Olaylar" })).getAllByRole("row").slice(1).map((r) => r.querySelectorAll("td")[1].textContent);

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.eventSearchCancel).mockResolvedValue({ status: "ok", data: null });
});

test("a nakshatra ingress row names the lord of the nakshatra entered; other rows do not", async () => {
  // 4 Rohini: dokuzlu sıranın (4 − 1) mod 9 = 3. adı, Ay. 19 Mula: 18 mod 9 = 0, Ketu.
  // 27 Revati: 26 mod 9 = 8, Merkür. Yeni ay satırı nakshatra taşır ama giriş değildir.
  const events = [row("NakshatraIngress", ["Moon"], 4), row("NakshatraIngress", ["Moon"], 19), row("NakshatraIngress", ["Saturn"], 27), row("NewMoon", ["Sun", "Moon"], 18)];
  vi.mocked(api.eventSearch).mockResolvedValue(ok({ ...VIEW, events }));
  renderPanel();
  fireEvent.click(run());
  await settle();
  expect(where()).toEqual(["Rohini · lordu Ay", "Mula · lordu Ketu", "Revati · lordu Merkür", "Jyeshtha"]);
  // Sütun eklenmez: lord, var olan "Burç · nakshatra" hücresindedir.
  expect(within(screen.getByRole("table", { name: "Olaylar" })).getAllByRole("columnheader")).toHaveLength(3);
});

test("the fixture's rows are unchanged: none of them is a nakshatra ingress", async () => {
  vi.mocked(api.eventSearch).mockResolvedValue(ok(VIEW));
  renderPanel();
  fireEvent.click(run());
  await settle();
  expect(VIEW.events.some((e) => e.event.kind === "NakshatraIngress")).toBe(false);
  for (const text of where()) expect(text).not.toContain("lordu");
});

test("the preset selects the Moon's nakshatra ingresses and does not start a search", async () => {
  vi.mocked(api.eventSearch).mockResolvedValue(ok(VIEW));
  renderPanel();
  fireEvent.click(screen.getByRole("button", { name: "Hazır seçim: Ay'ın nakshatra girişleri" }));
  await settle();
  expect(api.eventSearch).not.toHaveBeenCalled();
  const form = screen.getByRole("form", { name: "Olay araması ayarları" });
  const checked = (group: string) =>
    within(within(form).getByRole("group", { name: group }))
      .getAllByRole("checkbox", { checked: true })
      .map((c) => c.parentElement?.textContent);
  expect(checked("Olay türleri")).toEqual(["Nakshatra girişi"]);
  expect(checked("Grahalar")).toEqual(["Ay"]);
  fireEvent.click(run());
  await settle();
  expect(vi.mocked(api.eventSearch).mock.calls.at(-1)![0]).toMatchObject({ kinds: ["NakshatraIngress"], grahas: ["Moon"] });
});

test("the lord texts exist in the three languages", () => {
  expect([tr.eventSearch.lordOf, en.eventSearch.lordOf, fr.eventSearch.lordOf]).toEqual(["lordu {{graha}}", "lord {{graha}}", "maître {{graha}}"]);
  expect([tr.transit.nakshatraLord, en.transit.nakshatraLord, fr.transit.nakshatraLord]).toEqual(["Nakshatra lordu", "Nakshatra lord", "Maître du nakshatra"]);
  for (const m of [tr, en, fr]) expect(m.table.nakshatraLord).toBe(m.transit.nakshatraLord);
});
