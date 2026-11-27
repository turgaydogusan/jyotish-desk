import { act, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn(), transitStep: vi.fn() } }));

import { api, type Result } from "../api";
import type { TransitStepRequest, TransitStepView, TransitView } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useSession, type Session } from "../session/store";
import { useTransitControl, type Transit } from "./store";
import { TransitStepper } from "./TransitStepper";
import { DEFAULT_PLAY_SPEED, PLAY_INTERVAL_MS, PLAY_SPEEDS } from "./useTransitPlayback";

const J0 = fixture().transit.snapshot.jd_ut!;
const stepped = (r: TransitStepRequest): Result<TransitStepView> => ({ status: "ok", data: { jd_ut: r.jd_ut! + r.count, utc: "" } });
const viewAt = (at: unknown): Result<TransitView> => {
  const view = fixture().transit;
  if (typeof at === "object" && at !== null && "Jd" in at) view.snapshot.jd_ut = (at as { Jd: number }).Jd;
  return { status: "ok", data: view };
};

let transit: Transit;
let session: Session;
function Probe() {
  transit = useTransitControl();
  session = useSession();
  return <TransitStepper />;
}
const group = () => screen.getByRole("group", { name: "Transit anını adımla" });
const play = () => within(group()).getByRole("button", { name: "Oynat" });
const pause = () => within(group()).getByRole("button", { name: "Duraklat" });
/** Zamanı ilerletir ve bekleyen yanıtları işler. */
const advance = (ms: number) => act(async () => void (await vi.advanceTimersByTimeAsync(ms)));
const steps = () => vi.mocked(api.transitStep).mock.calls.length;
const NORMAL = PLAY_INTERVAL_MS.normal;

/** `matchMedia` taklidi: "hareketi azalt" sorgusunun yanıtı ve değişim bildirimi. */
function mockReducedMotion(matches: boolean) {
  const listeners = new Set<() => void>();
  const query = {
    matches,
    addEventListener: (_: string, f: () => void) => listeners.add(f),
    removeEventListener: (_: string, f: () => void) => listeners.delete(f),
  };
  vi.stubGlobal("matchMedia", (q: string) => (q === "(prefers-reduced-motion: reduce)" ? query : { matches: false, addEventListener: () => {}, removeEventListener: () => {} }));
  return {
    set(next: boolean) {
      query.matches = next;
      for (const f of listeners) f();
    },
  };
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(api.transitsAt).mockReset();
  vi.mocked(api.transitsAt).mockImplementation(async (at) => viewAt(at));
  vi.mocked(api.transitStep).mockReset();
  vi.mocked(api.transitStep).mockImplementation(async (r) => stepped(r));
});
afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

async function renderReady() {
  const utils = renderWithStores(<Probe />);
  await advance(0);
  expect(play()).not.toHaveAttribute("aria-disabled");
  return utils;
}

test("play steps forward once per interval, each step after the previous moment has settled; pause stops", async () => {
  await renderReady();
  expect([DEFAULT_PLAY_SPEED, NORMAL]).toEqual(["normal", 1000]);
  fireEvent.click(play());
  // Düğmenin adı durumu söyler.
  expect(pause()).toBeInTheDocument();
  await advance(NORMAL - 1);
  expect(steps()).toBe(0);
  await advance(1);
  expect(steps()).toBe(1);
  expect(vi.mocked(api.transitStep).mock.calls[0][0]).toMatchObject({ jd_ut: J0, unit: "Day", count: 1 });
  expect(transit.at).toEqual({ Jd: J0 + 1 });
  await advance(NORMAL);
  await advance(NORMAL);
  expect(steps()).toBe(3);
  expect(transit.at).toEqual({ Jd: J0 + 3 });
  fireEvent.click(pause());
  await advance(10 * NORMAL);
  expect(steps()).toBe(3);
  expect(play()).toBeInTheDocument();
});

test("no step is planned while the transits of the last moment are still being read", async () => {
  await renderReady();
  let resolve!: (r: Result<TransitView>) => void;
  fireEvent.click(play());
  // İlk adımdan sonraki transit yanıtı bekletilir: motor meşgul.
  vi.mocked(api.transitsAt).mockImplementationOnce(() => new Promise((r) => (resolve = r)));
  await advance(NORMAL);
  expect(steps()).toBe(1);
  await advance(20 * NORMAL);
  expect(steps()).toBe(1);
  // Yanıt gelince bir aralık sonra sıradaki adım atılır.
  await act(async () => resolve(viewAt({ Jd: J0 + 1 })));
  await advance(NORMAL - 1);
  expect(steps()).toBe(1);
  await advance(1);
  expect(steps()).toBe(2);
});

test("the speed select sets the wait between steps and the unit select the size of a step", async () => {
  await renderReady();
  const speed = within(group()).getByRole("combobox", { name: "Hız" });
  expect(within(speed).getAllByRole("option").map((o) => o.textContent)).toEqual(["Yavaş", "Orta", "Hızlı"]);
  expect([PLAY_SPEEDS, PLAY_INTERVAL_MS]).toEqual([["slow", "normal", "fast"], { slow: 2000, normal: 1000, fast: 400 }]);
  fireEvent.change(speed, { target: { value: "fast" } });
  fireEvent.change(within(group()).getByRole("combobox", { name: "Adım birimi" }), { target: { value: "Week" } });
  fireEvent.click(play());
  await advance(400);
  expect(steps()).toBe(1);
  expect(vi.mocked(api.transitStep).mock.calls[0][0].unit).toBe("Week");
  await advance(400);
  expect(steps()).toBe(2);
  fireEvent.change(speed, { target: { value: "slow" } });
  await advance(1999);
  expect(steps()).toBe(2);
  await advance(1);
  expect(steps()).toBe(3);
});

test("a step error stops playback and is shown", async () => {
  await renderReady();
  fireEvent.click(play());
  await advance(NORMAL);
  vi.mocked(api.transitStep).mockResolvedValueOnce({ status: "error", error: { code: "invalid_transit_input", params: { field: "range" } } });
  await advance(NORMAL);
  expect(steps()).toBe(2);
  expect(screen.getByRole("alert")).toHaveTextContent("Transit isteği geçersiz: tarih aralığı (1–2999 yılları).");
  expect(play()).toBeInTheDocument();
  await advance(10 * NORMAL);
  expect(steps()).toBe(2);
});

test("a transit error stops playback", async () => {
  await renderReady();
  fireEvent.click(play());
  vi.mocked(api.transitsAt).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  await advance(NORMAL);
  expect(steps()).toBe(1);
  expect(play()).toBeInTheDocument();
  await advance(10 * NORMAL);
  expect(steps()).toBe(1);
});

test("the timer stops when the stepper goes away", async () => {
  const { unmount } = await renderReady();
  fireEvent.click(play());
  await advance(NORMAL);
  expect(steps()).toBe(1);
  unmount();
  await advance(20 * NORMAL);
  expect(steps()).toBe(1);
});

test("opening another chart stops playback", async () => {
  await renderReady();
  fireEvent.click(play());
  await advance(NORMAL);
  expect(steps()).toBe(1);
  const other = fixture().opened;
  other.chart_record.id = 77;
  act(() => session.open(other));
  await advance(0);
  expect(play()).toBeInTheDocument();
  // Yeni haritada an "şimdi"ye döner ve oynatma kendiliğinden başlamaz.
  expect(transit.at).toBe("Now");
  await advance(20 * NORMAL);
  expect(steps()).toBe(1);
});

test("a moment chosen by hand during playback is the new starting point", async () => {
  await renderReady();
  fireEvent.click(play());
  await advance(NORMAL);
  expect(transit.at).toEqual({ Jd: J0 + 1 });
  act(() => transit.setAt({ Jd: 2_451_545 }));
  // Önce seçilen anın transitleri gelir (an yerleşir); aralık ondan sonra sayılır.
  await advance(0);
  expect(steps()).toBe(1);
  await advance(NORMAL);
  expect(steps()).toBe(2);
  expect(vi.mocked(api.transitStep).mock.calls.at(-1)![0].jd_ut).toBe(2_451_545);
  expect(transit.at).toEqual({ Jd: 2_451_546 });
});

test("with reduced motion playback is not offered and the step buttons still work", async () => {
  mockReducedMotion(true);
  renderWithStores(<Probe />);
  await advance(0);
  expect(within(group()).queryByRole("button", { name: "Oynat" })).toBeNull();
  expect(within(group()).queryByRole("combobox", { name: "Hız" })).toBeNull();
  expect(within(group()).getByRole("note")).toHaveTextContent("Sistemde “hareketi azalt” açık: oynatma sunulmuyor. Adım düğmeleri çalışır.");
  fireEvent.click(within(group()).getByRole("button", { name: "Bir adım ileri" }));
  await advance(0);
  expect(transit.at).toEqual({ Jd: J0 + 1 });
  await advance(20 * NORMAL);
  expect(steps()).toBe(1);
});

test("turning reduced motion on during playback stops it", async () => {
  const media = mockReducedMotion(false);
  await renderReady();
  fireEvent.click(play());
  await advance(NORMAL);
  expect(steps()).toBe(1);
  act(() => media.set(true));
  expect(within(group()).queryByRole("button", { name: "Duraklat" })).toBeNull();
  await advance(20 * NORMAL);
  expect(steps()).toBe(1);
  // Ayar kapanınca oynatma kendiliğinden başlamaz.
  act(() => media.set(false));
  expect(play()).toBeInTheDocument();
  await advance(20 * NORMAL);
  expect(steps()).toBe(1);
});

test("the playback texts exist in the three languages", () => {
  for (const m of [tr, en, fr]) expect(Object.keys(m.transitStep.speeds)).toEqual([...PLAY_SPEEDS]);
  expect([tr.transitStep.play, en.transitStep.play, fr.transitStep.play]).toEqual(["Oynat", "Play", "Lire"]);
  expect([tr.transitStep.pause, en.transitStep.pause, fr.transitStep.pause]).toEqual(["Duraklat", "Pause", "Mettre en pause"]);
  expect(JSON.stringify(fr.transitStep)).not.toContain("'");
  expect(fr.transitStep.reducedMotion).toContain("n’est pas proposée");
});
