import { fireEvent, screen, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computePanchangExtras: vi.fn() } }));

import { api, type Result } from "../api";
import type { PanchangExtras } from "../bindings";
import { setLanguage } from "../i18n";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { PanchangPanel } from "./PanchangPanel";

const panel: Panel = { id: "pa", kind: "panchang", x: 0, y: 0, w: 4, h: 9, settings: {} };
const ok = (extras: PanchangExtras = fixture().natal_extras.panchang_extras): Result<PanchangExtras> => ({ status: "ok", data: extras });
const extrasTable = () => screen.findByRole("table", { name: "Panchang ekleri" });
const cells = (key: string) => [...document.querySelector(`tr[data-extra="${key}"]`)!.children].map((c) => c.textContent);
const highlighted = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.extra);

beforeEach(() => {
  vi.mocked(api.computePanchangExtras).mockReset();
  vi.mocked(api.computePanchangExtras).mockImplementation(async () => ok());
});

test("five extra facts under the panchang: yogi point, yogi, co-yogi, avayogi and the dagdha signs", async () => {
  renderWithStores(<PanchangPanel panel={panel} />);
  const table = await extrasTable();
  const opened = fixture().opened;
  expect(api.computePanchangExtras).toHaveBeenCalledTimes(1);
  expect(api.computePanchangExtras).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null });
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Öğe", "Değer"]);
  // Delhi 2000-01-01: nitya yoga Dhriti (8) → yogi nakshatrası Swati (15), lordu Rahu; nokta
  // Terazi'de (lordu Venüs); altıncı nakshatra Purva Ashadha (20), lordu Venüs; Krishna
  // Ekadashi: Yay ve Balık.
  expect(cells("yogi_point")).toEqual(["Yogi noktası", "Ter 9°19′ · Swati · pada 1"]);
  expect(cells("yogi")).toEqual(["Yogi", "Rahu"]);
  expect(cells("co_yogi")).toEqual(["Eş yogi", "Venüs"]);
  expect(cells("avayogi")).toEqual(["Avayogi", "Venüs · Purva Ashadha"]);
  expect(cells("dagdha")).toEqual(["Dagdha rasi", "Yay, Bal"]);
  expect(screen.getByText("Doğrulanmamış tablo: dagdha rasi tablosu basılı kaynakla henüz karşılaştırılmadı.")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: panchang_extras.v1")).toBeInTheDocument();
  expect(document.querySelector('[title="yaklaşık"]')).toBeNull();
  // Beş uzvun tablosu yerindedir.
  expect(screen.getByRole("table", { name: /^Doğum anının panchangı/ })).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderWithStores(<PanchangPanel panel={panel} />, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computePanchangExtras).not.toHaveBeenCalled();
});

test("unknown birth time: the request says so and every extra value is marked approximate", async () => {
  const extras = fixture().natal_extras.panchang_extras;
  extras.approximate = true;
  vi.mocked(api.computePanchangExtras).mockImplementation(async () => ok(extras));
  renderWithStores(<PanchangPanel panel={panel} />, { opened: unknownTimeOpened() });
  await extrasTable();
  expect(vi.mocked(api.computePanchangExtras).mock.calls[0][0].time_known).toBe(false);
  for (const key of ["yogi_point", "yogi", "co_yogi", "avayogi", "dagdha"]) {
    expect(document.querySelector(`tr[data-extra="${key}"] [title="yaklaşık"]`), key).not.toBeNull();
  }
  expect(cells("yogi")[1]).toBe("≈yaklaşık Rahu");
});

test("full or new moon: no dagdha sign, said in words", async () => {
  const extras = fixture().natal_extras.panchang_extras;
  extras.tithi = 15;
  extras.dagdha_rasis = [];
  vi.mocked(api.computePanchangExtras).mockImplementation(async () => ok(extras));
  renderWithStores(<PanchangPanel panel={panel} />);
  await extrasTable();
  expect(cells("dagdha")).toEqual(["Dagdha rasi", "yok (dolunay ve yeni ayda dagdha rasi olmaz)"]);
});

test("shared highlight: a graha marks the rows it lords, a sign the rows that name it", async () => {
  renderWithStores(<PanchangPanel panel={panel} />);
  const table = await extrasTable();
  expect(highlighted()).toEqual([]);
  // Venüs eş yogi ve avayogidir.
  fireEvent.click(within(table).getAllByRole("button", { name: "Venüs" })[0]);
  expect(highlighted()).toEqual(["co_yogi", "avayogi"]);
  fireEvent.click(within(table).getByRole("button", { name: "Rahu" }));
  expect(highlighted()).toEqual(["yogi"]);
  fireEvent.click(within(table).getByRole("button", { name: "Ter" }));
  expect(highlighted()).toEqual(["yogi_point"]);
  fireEvent.click(within(table).getByRole("button", { name: "Bal" }));
  expect(highlighted()).toEqual(["dagdha"]);
  expect(within(table).getByRole("button", { name: "Bal" })).toHaveAttribute("aria-pressed", "true");
});

test("an error leaves the five limbs in place and is retried from the panel", async () => {
  vi.mocked(api.computePanchangExtras).mockImplementationOnce(async () => ({ status: "error", error: { code: "engine_stopped", params: {} } }));
  renderWithStores(<PanchangPanel panel={panel} />);
  const alert = await screen.findByRole("alert");
  expect(screen.getByRole("table", { name: /^Doğum anının panchangı/ })).toBeInTheDocument();
  expect(screen.queryByRole("table", { name: "Panchang ekleri" })).toBeNull();
  fireEvent.click(within(alert).getByRole("button", { name: "Yeniden dene" }));
  await extrasTable();
  expect(api.computePanchangExtras).toHaveBeenCalledTimes(2);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("labels in three languages; classical names stay; French uses the typographic apostrophe", async () => {
  await setLanguage("en");
  const first = renderWithStores(<PanchangPanel panel={panel} />);
  await screen.findByRole("table", { name: "Panchang additions" });
  expect(cells("yogi_point")).toEqual(["Yogi point", "Li 9°19′ · Swati · pada 1"]);
  expect(cells("co_yogi")).toEqual(["Duplicate yogi", "Venus"]);
  expect(cells("avayogi")).toEqual(["Avayogi", "Venus · Purva Ashadha"]);
  first.unmount();
  await setLanguage("fr");
  renderWithStores(<PanchangPanel panel={panel} />);
  await screen.findByRole("table", { name: "Compléments du panchang" });
  expect(cells("yogi")).toEqual(["Yogi", "Rahu"]);
  expect(cells("dagdha")[0]).toBe("Dagdha rasi");
  for (const text of [...Object.values(fr.panchangExtras), ...Object.values(fr.navatara)]) expect(text).not.toContain("'");
  expect(Object.keys(en.panchangExtras)).toEqual(Object.keys(fr.panchangExtras));
});
