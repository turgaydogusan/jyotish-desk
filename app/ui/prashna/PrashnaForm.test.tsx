import { act, fireEvent, screen, within } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("../api", () => ({ api: { searchPlaces: vi.fn(), createPrashna: vi.fn(), setAppPrefs: vi.fn() } }));

import { api } from "../api";
import type { AppPrefs, OpenedClient } from "../bindings";
import { SEARCH_DEBOUNCE_MS } from "../birth/PlaceSearch";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import { useSession } from "../session/store";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { PrashnaForm } from "./PrashnaForm";

function OpenedLabel() {
  const { opened } = useSession();
  return <output aria-label="açık">{opened ? `${opened.client.name} / ${opened.chart_record.label}` : "-"}</output>;
}

const onClose = vi.fn();
const onCreated = vi.fn();
const home = () => fixture().muhurta.home_place;
const withHome = (): AppPrefs => ({ ...fixture().prefs.prefs, home_place: home() });

function renderForm({ opened = fixture().opened, prefs = withHome() }: { opened?: OpenedClient | null; prefs?: AppPrefs | null } = {}) {
  return renderWithStores(
    <>
      <OpenedLabel />
      <PrashnaForm onClose={onClose} onCreated={onCreated} />
    </>,
    { opened, prefs },
  );
}

const dialog = () => screen.getByRole("dialog", { name: "Yeni prashna" });
const field = (name: string | RegExp) => within(dialog()).getByLabelText(name);
const submit = () => fireEvent.click(within(dialog()).getByRole("button", { name: "Prashna kur" }));
const wait = (ms = 0) => act(async () => vi.advanceTimersByTime(ms));

beforeEach(() => {
  vi.useFakeTimers();
  onClose.mockReset();
  onCreated.mockReset();
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: fixture().places });
  vi.mocked(api.createPrashna).mockResolvedValue({ status: "ok", data: fixture().prashna.opened });
});
afterEach(() => vi.useRealTimers());

test("defaults: now, the current place, no number, no focus house, the open client; casting opens the returned chart", async () => {
  renderForm();
  expect(document.activeElement).toBe(field("Şimdi"));
  expect(field("Şimdi")).toBeChecked();
  expect(within(dialog()).queryByLabelText("Tarih")).toBeNull();
  expect(field("Yer")).toHaveDisplayValue("Bulunulan yer");
  expect(dialog().querySelector(".prashna-place")).toHaveTextContent("New Delhi, Delhi, India · Asia/Kolkata");
  expect(field("Sayı (isteğe bağlı)")).toHaveValue("");
  expect(field("Odak evi")).toHaveDisplayValue("Seçilmedi");
  expect(field("“Örnek Danışan” danışanına ekle")).toBeChecked();
  submit();
  await wait();
  expect(api.createPrashna).toHaveBeenCalledWith({ client_id: 1, at: "Now", place: home(), number: null, question: "", focus_house: null });
  expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Örnek Danışan / Prashna 2026-11-25 15:30");
  expect(onCreated).toHaveBeenCalledTimes(1);
  expect(onClose).toHaveBeenCalledTimes(1);
});

test("a typed moment, a number, a question and a focus house are sent as typed", async () => {
  vi.mocked(api.createPrashna).mockResolvedValue({ status: "ok", data: fixture().prashna.opened_number });
  renderForm();
  fireEvent.click(field("Başka bir an"));
  fireEvent.change(field("Tarih"), { target: { value: "2026-11-27" } });
  fireEvent.change(field("Saat"), { target: { value: "14:05" } });
  expect(field("Tarih")).toHaveAccessibleDescription("Sorunun yerinin duvar saati.");
  fireEvent.change(field("Sayı (isteğe bağlı)"), { target: { value: "54" } });
  fireEvent.change(field("Soru"), { target: { value: "Deneme sorusu" } });
  expect(field("Soru")).toHaveAccessibleDescription(/^13 \/ 2000 karakter\./);
  fireEvent.change(field("Odak evi"), { target: { value: "7" } });
  fireEvent.click(field("“Örnek Danışan” danışanına ekle"));
  submit();
  await wait();
  expect(api.createPrashna).toHaveBeenCalledWith({
    client_id: null,
    at: { Local: { date: "2026-11-27", time: "14:05" } },
    place: home(),
    number: 54,
    question: "Deneme sorusu",
    focus_house: 7,
  });
  expect(onClose).toHaveBeenCalledTimes(1);
});

test("without a current place the form asks for one and does not fall back to the chart's place", async () => {
  renderForm({ prefs: fixture().prefs.prefs });
  expect(dialog().querySelector(".prashna-place")).toHaveTextContent("Bulunulan yer seçilmemiş.");
  submit();
  await wait();
  expect(api.createPrashna).not.toHaveBeenCalled();
  expect(field("Yer")).toHaveAttribute("aria-invalid", "true");
  expect(field("Yer")).toHaveAccessibleDescription(/Bir yer seçin\./);
  expect(document.activeElement).toBe(field("Yer"));
  expect(within(dialog()).getByText("Düzeltilmesi gereken 1 alan var.")).toBeInTheDocument();
  // Açık haritanın yeri bir seçimdir; seçilince hata hemen kalkar.
  fireEvent.change(field("Yer"), { target: { value: "chart" } });
  expect(field("Yer")).not.toHaveAttribute("aria-invalid");
  expect(dialog().querySelector(".prashna-place")).toHaveTextContent("New Delhi, Delhi, India · Asia/Kolkata");
  submit();
  await wait();
  const { place, tz } = fixture().opened.chart_record.birth;
  expect(vi.mocked(api.createPrashna).mock.calls[0][0].place).toEqual({ place, tz });
});

test("another place is searched in the atlas; with no open client the prashna goes to the built-in client", async () => {
  renderForm({ opened: null, prefs: fixture().prefs.prefs });
  // Açık harita yokken "açık haritanın yeri" seçeneği ve danışan kutusu yoktur.
  expect(within(field("Yer")).getAllByRole("option").map((o) => o.textContent)).toEqual(["Bulunulan yer", "Başka bir yer ara"]);
  expect(within(dialog()).queryByRole("checkbox")).toBeNull();
  expect(within(dialog()).getByText(/Açık danışan yok: prashna “Prashna” adlı yerleşik danışanın altında toplanır\./)).toBeInTheDocument();
  fireEvent.change(field("Yer"), { target: { value: "search" } });
  expect(dialog().querySelector(".prashna-place")).toHaveTextContent("Henüz yer seçilmedi.");
  fireEvent.change(field("Yer ara"), { target: { value: "new delhi" } });
  await wait(SEARCH_DEBOUNCE_MS);
  fireEvent.keyDown(field("Yer ara"), { key: "Enter" });
  const hit = fixture().places[0];
  expect(dialog().querySelector(".prashna-place")).toHaveTextContent(`${hit.name}`);
  submit();
  await wait();
  const sent = vi.mocked(api.createPrashna).mock.calls[0][0];
  expect(sent.client_id).toBeNull();
  expect(sent.place).toEqual({ place: { name: hit.name, country: hit.country, admin1: hit.admin1, lat: hit.lat, lon: hit.lon, geoname_id: hit.geoname_id }, tz: { Iana: hit.timezone } });
});

test("format errors stay in the form, focus the first invalid field and clear when the field is edited", async () => {
  renderForm();
  fireEvent.click(field("Başka bir an"));
  fireEvent.change(field("Sayı (isteğe bağlı)"), { target: { value: "109" } });
  submit();
  await wait();
  expect(api.createPrashna).not.toHaveBeenCalled();
  expect(field("Tarih")).toHaveAttribute("aria-invalid", "true");
  expect(document.activeElement).toBe(field("Tarih"));
  expect(field("Sayı (isteğe bağlı)")).toHaveAccessibleDescription(/Sayı 1 ile 108 arasında bir tamsayı olmalı\./);
  expect(within(dialog()).getByText("Düzeltilmesi gereken 2 alan var.")).toBeInTheDocument();
  fireEvent.change(field("Sayı (isteğe bağlı)"), { target: { value: "108" } });
  expect(field("Sayı (isteğe bağlı)")).not.toHaveAttribute("aria-invalid");
  fireEvent.click(field("Şimdi"));
  expect(within(dialog()).queryByText(/Düzeltilmesi gereken/)).toBeNull();
});

test("a time error from Rust while the moment is now is shown by the moment choice and takes focus", async () => {
  vi.mocked(api.createPrashna).mockResolvedValueOnce({ status: "error", error: { code: "invalid_prashna_input", params: { field: "at" } } });
  renderForm();
  submit();
  await wait();
  expect(onClose).not.toHaveBeenCalled();
  expect(within(dialog()).getByText(/Bu an bu yerde çözülemedi\./)).toBeInTheDocument();
  expect(field("Şimdi")).toHaveAttribute("aria-invalid", "true");
  expect(field("Şimdi")).toHaveAccessibleDescription(/Bu an bu yerde çözülemedi\./);
  expect(document.activeElement).toBe(field("Şimdi"));
  fireEvent.click(field("Başka bir an"));
  expect(within(dialog()).queryByText(/Bu an bu yerde çözülemedi\./)).toBeNull();
});

test("a field error from Rust is shown on its field with a static message; other errors are a form alert", async () => {
  vi.mocked(api.createPrashna).mockResolvedValueOnce({ status: "error", error: { code: "invalid_prashna_input", params: { field: "at" } } });
  renderForm();
  fireEvent.click(field("Başka bir an"));
  fireEvent.change(field("Tarih"), { target: { value: "2026-03-29" } });
  fireEvent.change(field("Saat"), { target: { value: "02:30" } });
  submit();
  await wait();
  expect(onClose).not.toHaveBeenCalled();
  expect(field("Saat")).toHaveAttribute("aria-invalid", "true");
  expect(field("Saat")).toHaveAccessibleDescription(/Bu an bu yerde çözülemedi\./);
  expect(document.activeElement).toBe(field("Tarih"));
  // Açık harita değişmedi.
  expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Örnek Danışan /");
  // Alan düzeltilince hata kalkar; başka bir hata formun uyarısıdır.
  fireEvent.change(field("Saat"), { target: { value: "03:30" } });
  expect(field("Saat")).not.toHaveAttribute("aria-invalid");
  vi.mocked(api.createPrashna).mockResolvedValueOnce({ status: "error", error: { code: "not_found", params: { what: "client", id: "1" } } });
  submit();
  await wait();
  expect(within(dialog()).getByRole("alert")).toHaveTextContent(/^Prashna kurulamadı: /);
  expect(onClose).not.toHaveBeenCalled();
});

test("the cast button is locked while the chart is being cast: one command for two presses", async () => {
  let finish: (v: Awaited<ReturnType<typeof api.createPrashna>>) => void = () => {};
  vi.mocked(api.createPrashna).mockReturnValue(new Promise((resolve) => (finish = resolve)));
  renderForm();
  submit();
  await wait();
  submit();
  await wait();
  expect(api.createPrashna).toHaveBeenCalledTimes(1);
  await act(async () => finish({ status: "ok", data: fixture().prashna.opened }));
  expect(onClose).toHaveBeenCalledTimes(1);
});

test("Escape asks before discarding a changed draft; Cancel closes without asking", async () => {
  renderForm();
  fireEvent.keyDown(document, { key: "Escape" });
  expect(onClose).toHaveBeenCalledTimes(1);
  onClose.mockReset();
  fireEvent.change(field("Soru"), { target: { value: "Deneme" } });
  fireEvent.keyDown(document, { key: "Escape" });
  expect(onClose).not.toHaveBeenCalled();
  const confirm = screen.getByRole("dialog", { name: "Değişiklikler atılsın mı?" });
  fireEvent.keyDown(document, { key: "Escape" });
  expect(confirm).not.toBeInTheDocument();
  expect(field("Soru")).toHaveValue("Deneme");
  fireEvent.click(within(dialog()).getByRole("button", { name: "Vazgeç" }));
  expect(onClose).toHaveBeenCalledTimes(1);
  expect(api.createPrashna).not.toHaveBeenCalled();
});

test("the form texts exist in three languages and the classical word is the same in each", () => {
  for (const lang of [tr, en, fr]) {
    expect(lang.prashna.form.title).toContain("prashna");
    expect(lang.shortcuts.newPrashna).toBe(lang.prashna.form.title);
    expect(lang.palette.command.newPrashna).toBe(lang.prashna.form.title);
    expect(Object.keys(lang.prashna.form.error).sort()).toEqual(["at", "atMissing", "focus_house", "number", "place", "placeMissing", "question"]);
  }
  // Fransızca metinler tipografik kesme işareti kullanır.
  expect(JSON.stringify(fr.prashna)).not.toContain("'");
});
