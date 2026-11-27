import { fireEvent, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";

// Panelin ikinci tablosu (panchang ekleri) bir komuttan gelir; bu dosya beş uzvu sınar, ekleri
// `PanchangExtras.test.tsx` sınar.
vi.mock("../api", () => ({
  api: { computePanchangExtras: vi.fn(async () => ({ status: "ok", data: (await import("../test/fixture")).fixture().natal_extras.panchang_extras })) },
}));

import { useHighlight } from "../highlight/store";
import { setLanguage } from "../i18n";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { nityaYogaName, NITYA_YOGA_NAMES, tithiInPaksha, tithiName, VARA_NAMES } from "./labels";
import { PanchangPanel } from "./PanchangPanel";

const panel: Panel = { id: "pa", kind: "panchang", x: 0, y: 0, w: 4, h: 9, settings: {} };

/** Ortak vurguyu dışarıdan süren düğmeler (başka bir panelin yerine). */
function Pins() {
  const h = useHighlight();
  return (
    <>
      <button onClick={() => h.togglePin({ type: "graha", id: "Moon" })}>pin moon</button>
      <button onClick={() => h.togglePin({ type: "graha", id: "Sun" })}>pin sun</button>
      <button onClick={() => h.togglePin({ type: "graha", id: "Mars" })}>pin mars</button>
    </>
  );
}

const cells = (key: string) => [...document.querySelector(`tr[data-row="${key}"]`)!.children].map((c) => c.textContent);
const highlighted = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.row);

afterEach(() => setLanguage("tr"));

test("five limbs with their end times and sunrise and sunset, in the birth place's time", () => {
  renderWithStores(<PanchangPanel panel={panel} />);
  // Delhi 2000-01-01 17:30 IST (+05:30); değerler tests/golden/new-delhi-2000-01-01.json ile aynı.
  expect(screen.getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Öğe", "Değer", "Bitiş"]);
  expect(cells("tithi")).toEqual(["Tithi", "Krishna 11 · Ekadashi", "2000-01-02 13:19"]);
  expect(cells("vara")).toEqual(["Vara", "Shanivara · Cumartesi", ""]);
  expect(cells("nakshatra")).toEqual(["Nakshatra", "Swati · pada 4", "2000-01-01 18:33"]);
  expect(cells("yoga")).toEqual(["Nitya yoga", "Dhriti", "2000-01-02 13:14"]);
  expect(cells("karana")).toEqual(["Karana", "Bava", "2000-01-02 00:09"]);
  expect(cells("sunrise")).toEqual(["Gün doğumu", "2000-01-01 07:17", ""]);
  expect(cells("sunset")).toEqual(["Gün batımı", "2000-01-01 17:30", ""]);
  expect(screen.getByRole("table", { name: "Doğum anının panchangı; saatler doğum yerinin saatiyle (IST +05:30)" })).toBeInTheDocument();
  expect(document.querySelector('[title="yaklaşık"]')).toBeNull();
  expect(screen.queryByText(/Gün doğumu yok/)).toBeNull();
});

test("without an open chart the panel says so", () => {
  renderWithStores(<PanchangPanel panel={panel} />, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
});

test("unknown birth time: a notice, and the time-bound values are marked approximate", () => {
  renderWithStores(<PanchangPanel panel={panel} />, { opened: unknownTimeOpened() });
  expect(screen.getByText("Doğum saati bilinmiyor: değerler öğlen 12:00 içindir ve yaklaşıktır (≈).")).toBeInTheDocument();
  for (const key of ["tithi", "vara", "nakshatra", "yoga", "karana"]) {
    const mark = document.querySelector(`tr[data-row="${key}"] [title="yaklaşık"]`);
    expect(mark, key).not.toBeNull();
    expect(mark).toHaveAttribute("aria-hidden", "true");
  }
  expect(cells("nakshatra")[1]).toBe("≈yaklaşık Swati · pada 4");
  // Gün doğumu ve batımı o günündür; saate bağlı değildir.
  for (const key of ["sunrise", "sunset"]) expect(document.querySelector(`tr[data-row="${key}"] [title="yaklaşık"]`), key).toBeNull();
});

test("no sunrise at the birth place: dashes and the existing warning text", () => {
  const opened = fixture().opened;
  opened.chart.panchang.sunrise_jd_ut = null;
  opened.chart.panchang.sunset_jd_ut = null;
  renderWithStores(<PanchangPanel panel={panel} />, { opened });
  expect(cells("sunrise")).toEqual(["Gün doğumu", "—", ""]);
  expect(cells("sunset")).toEqual(["Gün batımı", "—", ""]);
  expect(screen.getByText("Gün doğumu yok")).toBeInTheDocument();
  expect(screen.getByText(/Doğum yerinde gün doğumu ya da batımı yok\./)).toBeInTheDocument();
  // Beş uzuv yine gösterilir.
  expect(cells("tithi")[1]).toBe("Krishna 11 · Ekadashi");
});

test("shared highlight: the Moon marks its rows, the Sun its rows, another graha none", () => {
  renderWithStores(
    <>
      <Pins />
      <PanchangPanel panel={panel} />
    </>,
  );
  expect(highlighted()).toEqual([]);
  fireEvent.click(screen.getByRole("button", { name: "pin moon" }));
  expect(highlighted()).toEqual(["tithi", "nakshatra", "yoga", "karana"]);
  fireEvent.click(screen.getByRole("button", { name: "pin sun" }));
  expect(highlighted()).toEqual(["tithi", "yoga", "karana", "sunrise", "sunset"]);
  fireEvent.click(screen.getByRole("button", { name: "pin mars" }));
  expect(highlighted()).toEqual([]);
});

test("classical names stay the same in every language; only the weekday gloss and headers change", async () => {
  await setLanguage("en");
  const { unmount } = renderWithStores(<PanchangPanel panel={panel} />);
  expect(cells("tithi")).toEqual(["Tithi", "Krishna 11 · Ekadashi", "2000-01-02 13:19"]);
  expect(cells("vara")).toEqual(["Vara", "Shanivara · Saturday", ""]);
  expect(cells("sunrise")[0]).toBe("Sunrise");
  unmount();
  await setLanguage("fr");
  renderWithStores(<PanchangPanel panel={panel} />);
  expect(cells("vara")).toEqual(["Vara", "Shanivara · samedi", ""]);
  expect(cells("yoga")).toEqual(["Nitya yoga", "Dhriti", "2000-01-02 13:14"]);
});

test("tithi, yoga and vara names", () => {
  expect([1, 14, 15, 16, 29, 30].map(tithiInPaksha)).toEqual([1, 14, 15, 1, 14, 15]);
  expect([1, 11, 14, 15, 16, 26, 30].map(tithiName)).toEqual(["Pratipada", "Ekadashi", "Chaturdashi", "Purnima", "Pratipada", "Ekadashi", "Amavasya"]);
  expect([0, 31, 1.5].map(tithiName)).toEqual(["0", "31", "1.5"]);
  expect(NITYA_YOGA_NAMES).toHaveLength(27);
  expect(new Set(NITYA_YOGA_NAMES).size).toBe(27);
  expect([1, 8, 27, 28].map(nityaYogaName)).toEqual(["Vishkambha", "Dhriti", "Vaidhriti", "28"]);
  expect(Object.keys(VARA_NAMES)).toEqual(["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]);
});
