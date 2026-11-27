import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn(), transitStep: vi.fn() } }));

import { api, type Result } from "../api";
import type { TransitStepRequest, TransitStepView, TransitView } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { browserTimeZone, useTransitControl, type Transit } from "./store";
import { TransitStepper } from "./TransitStepper";
import { TransitTablePanel } from "./TransitTablePanel";
import { DEFAULT_STEP_UNIT, STEP_UNITS, stepBase } from "./useTransitStepper";

const NOW = fixture().transit;
const J0 = NOW.snapshot.jd_ut!;
/** Taklit adım: birim ne olursa olsun adım başına bir gün (aritmetik Rust'tadır). */
const stepped = (r: TransitStepRequest): Result<TransitStepView> => ({ status: "ok", data: { jd_ut: r.jd_ut! + r.count, utc: "" } });
/** Taklit transit: `{ Jd }` anının yanıtı o anı taşır. */
const viewAt = (at: unknown): Result<TransitView> => {
  const view = fixture().transit;
  if (typeof at === "object" && at !== null && "Jd" in at) view.snapshot.jd_ut = (at as { Jd: number }).Jd;
  return { status: "ok", data: view };
};

let transit: Transit;
function Probe() {
  transit = useTransitControl();
  return <TransitStepper />;
}
const group = () => screen.getByRole("group", { name: "Transit anını adımla" });
const back = () => within(group()).getByRole("button", { name: "Bir adım geri" });
const forward = () => within(group()).getByRole("button", { name: "Bir adım ileri" });
const settle = () => act(async () => {});
const requests = () => vi.mocked(api.transitStep).mock.calls.map((c) => c[0]);

beforeEach(() => {
  vi.mocked(api.transitsAt).mockReset();
  vi.mocked(api.transitsAt).mockImplementation(async (at) => viewAt(at));
  vi.mocked(api.transitStep).mockReset();
  vi.mocked(api.transitStep).mockImplementation(async (r) => stepped(r));
});

test("a step asks Rust with the shown moment, the unit and the browser's zone, then moves the transit moment", async () => {
  renderWithStores(<Probe />);
  await waitFor(() => expect(forward()).not.toHaveAttribute("aria-disabled"));
  expect(screen.getByRole("combobox", { name: "Adım birimi" })).toHaveDisplayValue("Gün");
  expect(within(screen.getByRole("combobox", { name: "Adım birimi" })).getAllByRole("option").map((o) => o.textContent)).toEqual(["Saat", "Gün", "Hafta", "Ay", "Yıl"]);
  fireEvent.click(forward());
  await settle();
  // An "şimdi"dir: çıkış, gösterilen yanıtın anıdır.
  expect(requests()).toEqual([{ jd_ut: J0, unit: "Day", count: 1, tz: browserTimeZone() }]);
  expect(transit.at).toEqual({ Jd: J0 + 1 });
  expect(vi.mocked(api.transitsAt).mock.calls.at(-1)![0]).toEqual({ Jd: J0 + 1 });
});

test("back steps by minus one and the unit select chooses the unit Rust steps by", async () => {
  renderWithStores(<Probe />);
  await waitFor(() => expect(back()).not.toHaveAttribute("aria-disabled"));
  fireEvent.change(screen.getByRole("combobox", { name: "Adım birimi" }), { target: { value: "Month" } });
  fireEvent.click(back());
  await settle();
  expect(requests()).toEqual([{ jd_ut: J0, unit: "Month", count: -1, tz: browserTimeZone() }]);
  expect(transit.at).toEqual({ Jd: J0 - 1 });
});

test("the next step starts from the stepped moment even before its transits arrive", async () => {
  renderWithStores(<Probe />);
  await waitFor(() => expect(forward()).not.toHaveAttribute("aria-disabled"));
  // Bundan sonraki transit yanıtları gelmez: çıkış anı deponun `{ Jd }` anından okunur.
  vi.mocked(api.transitsAt).mockImplementation(() => new Promise(() => {}));
  fireEvent.click(forward());
  await settle();
  fireEvent.click(forward());
  await settle();
  fireEvent.click(back());
  await settle();
  expect(requests().map((r) => [r.jd_ut, r.count])).toEqual([
    [J0, 1],
    [J0 + 1, 1],
    [J0 + 2, -1],
  ]);
  expect(transit.at).toEqual({ Jd: J0 + 1 });
  expect(forward()).not.toHaveAttribute("aria-disabled");
});

test("before the first transits arrive the buttons stay focusable and do nothing", async () => {
  vi.mocked(api.transitsAt).mockImplementation(() => new Promise(() => {}));
  renderWithStores(<Probe />);
  await settle();
  for (const button of [back(), forward()]) {
    expect(button).toHaveAttribute("aria-disabled", "true");
    expect(button).not.toBeDisabled();
    fireEvent.click(button);
  }
  await settle();
  expect(api.transitStep).not.toHaveBeenCalled();
});

test("a click while a step is being asked is ignored", async () => {
  renderWithStores(<Probe />);
  await waitFor(() => expect(forward()).not.toHaveAttribute("aria-disabled"));
  let resolve!: (r: Result<TransitStepView>) => void;
  vi.mocked(api.transitStep).mockImplementationOnce(() => new Promise((r) => (resolve = r)));
  fireEvent.click(forward());
  fireEvent.click(forward());
  fireEvent.click(back());
  await settle();
  expect(api.transitStep).toHaveBeenCalledTimes(1);
  await act(async () => resolve({ status: "ok", data: { jd_ut: J0 + 1, utc: "" } }));
  expect(transit.at).toEqual({ Jd: J0 + 1 });
  // İstek bitti: yeni adım atılır.
  fireEvent.click(forward());
  await settle();
  expect(api.transitStep).toHaveBeenCalledTimes(2);
});

test("a reply that arrives after the moment was changed another way is dropped", async () => {
  renderWithStores(<Probe />);
  await waitFor(() => expect(forward()).not.toHaveAttribute("aria-disabled"));
  let resolve!: (r: Result<TransitStepView>) => void;
  vi.mocked(api.transitStep).mockImplementationOnce(() => new Promise((r) => (resolve = r)));
  fireEvent.click(forward());
  await settle();
  // Kullanıcı bu arada başka bir an seçti (dasa şeridi, olay satırı).
  act(() => transit.setAt({ Jd: 2_451_545 }));
  await act(async () => resolve({ status: "ok", data: { jd_ut: J0 + 1, utc: "" } }));
  expect(transit.at).toEqual({ Jd: 2_451_545 });
});

test("a reply that arrives after the stepper is gone is dropped without an update", async () => {
  const { unmount } = renderWithStores(<Probe />);
  await waitFor(() => expect(forward()).not.toHaveAttribute("aria-disabled"));
  let resolve!: (r: Result<TransitStepView>) => void;
  vi.mocked(api.transitStep).mockImplementationOnce(() => new Promise((r) => (resolve = r)));
  fireEvent.click(forward());
  await settle();
  const calls = vi.mocked(api.transitsAt).mock.calls.length;
  unmount();
  await act(async () => resolve({ status: "ok", data: { jd_ut: J0 + 1, utc: "" } }));
  expect(vi.mocked(api.transitsAt).mock.calls).toHaveLength(calls);
});

test("an input error names the field in the interface language and the step can be retried", async () => {
  renderWithStores(<Probe />);
  await waitFor(() => expect(forward()).not.toHaveAttribute("aria-disabled"));
  vi.mocked(api.transitStep).mockResolvedValueOnce({ status: "error", error: { code: "invalid_transit_input", params: { field: "range" } } });
  fireEvent.click(back());
  expect(await screen.findByRole("alert")).toHaveTextContent("Transit isteği geçersiz: tarih aralığı (1–2999 yılları).");
  expect(transit.at).toBe("Now");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(transit.at).toEqual({ Jd: J0 - 1 }));
  expect(screen.queryByRole("alert")).toBeNull();
  expect(requests().map((r) => r.count)).toEqual([-1, -1]);
});

test("the step base is the chosen Julian day, or the shown moment once it has arrived", () => {
  const view = fixture().transit;
  expect(stepBase({ at: { Jd: 5 }, view: null, loading: true })).toBe(5);
  expect(stepBase({ at: "Now", view, loading: false })).toBe(J0);
  expect(stepBase({ at: "Now", view, loading: true })).toBeNull();
  expect(stepBase({ at: { Local: { date: "2026-12-01", time: "09:30", tz: "UTC" } }, view: null, loading: false })).toBeNull();
  expect([DEFAULT_STEP_UNIT, STEP_UNITS]).toEqual(["Day", ["Hour", "Day", "Week", "Month", "Year"]]);
});

test("the transit table panel carries the stepper under the date boxes and steps the table", async () => {
  const panel: Panel = { id: "tt", kind: "transit_table", x: 0, y: 0, w: 6, h: 8, settings: {} };
  renderWithStores(<TransitTablePanel panel={panel} />);
  await screen.findByRole("table");
  const form = screen.getByRole("form", { name: "Transit anı" });
  expect(form.compareDocumentPosition(group()) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  expect(group().compareDocumentPosition(screen.getByRole("table")) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  fireEvent.click(forward());
  await waitFor(() => expect(vi.mocked(api.transitsAt).mock.calls.at(-1)![0]).toEqual({ Jd: J0 + 1 }));
  expect(screen.getByRole("button", { name: "Şimdi" })).toHaveAttribute("aria-pressed", "false");
  expect(screen.getByRole("table")).toBeInTheDocument();
});

test("the step texts exist in the three languages", () => {
  for (const m of [tr, en, fr]) {
    expect(Object.keys(m.transitStep.units)).toEqual([...STEP_UNITS]);
    expect(Object.keys(m.transitStep.field).sort()).toEqual(["count", "jd_ut", "range", "tz"]);
  }
  expect([tr.transitStep.forward, en.transitStep.forward, fr.transitStep.forward]).toEqual(["Bir adım ileri", "One step forward", "Un pas en avant"]);
  expect(JSON.stringify(fr.transitStep)).not.toContain("'");
});
