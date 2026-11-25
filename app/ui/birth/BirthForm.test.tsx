import { act, fireEvent, screen, within } from "@testing-library/react";
import { useEffect } from "react";
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: { searchPlaces: vi.fn(), previewMoment: vi.fn(), createClient: vi.fn(), updateChart: vi.fn() },
}));

import { api } from "../api";
import { useSession } from "../session/store";
import { fixture } from "../test/fixture";
import { NoticeBar, useNotices, type Notice } from "../shell/notices";
import { renderWithStores } from "../test/render";
import { BirthForm, PREVIEW_DEBOUNCE_MS } from "./BirthForm";
import { SEARCH_DEBOUNCE_MS } from "./PlaceSearch";

function OpenedName() {
  const { opened } = useSession();
  return <output aria-label="açık">{opened?.client.name ?? "-"}</output>;
}

const onClose = vi.fn();
function renderForm(mode: "new" | "edit", opened = mode === "edit" ? fixture().opened : null) {
  return renderWithStores(
    <>
      <OpenedName />
      <BirthForm mode={mode} onClose={onClose} />
    </>,
    { opened },
  );
}

const dialog = () => screen.getByRole("dialog");
const input = (name: string | RegExp) => within(dialog()).getByLabelText(name);
const wait = (ms: number) => act(async () => vi.advanceTimersByTime(ms));

async function fillDelhi() {
  fireEvent.change(input("Danışan adı"), { target: { value: "Ayşe" } });
  fireEvent.change(input("Tarih"), { target: { value: "2000-01-01" } });
  fireEvent.change(input("Saat"), { target: { value: "17:30" } });
  fireEvent.change(input("Doğum yeri"), { target: { value: "new delhi" } });
  await wait(SEARCH_DEBOUNCE_MS);
  fireEvent.keyDown(input("Doğum yeri"), { key: "Enter" });
  await wait(PREVIEW_DEBOUNCE_MS);
}

beforeEach(() => {
  vi.useFakeTimers();
  onClose.mockReset();
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: fixture().places });
  vi.mocked(api.previewMoment).mockResolvedValue({ status: "ok", data: fixture().preview });
  vi.mocked(api.createClient).mockResolvedValue({ status: "ok", data: fixture().opened });
  vi.mocked(api.updateChart).mockResolvedValue({ status: "ok", data: fixture().opened });
});
afterEach(() => vi.useRealTimers());

test("new chart: atlas place, live preview, save opens the returned chart", async () => {
  renderForm("new");
  expect(dialog()).toHaveAccessibleName("Yeni harita");
  expect(document.activeElement).toBe(input("Danışan adı"));
  await fillDelhi();
  expect(input("Bölge adı")).toHaveValue("Asia/Kolkata");
  expect(input("Enlem")).toHaveValue("28.62137");
  expect(within(dialog()).getByRole("region", { name: "Önizleme" })).toHaveTextContent("UTC 2000-01-01 12:00:00 · IST +05:30");
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(api.createClient).toHaveBeenCalledWith({
    name: "Ayşe",
    gender: null,
    chart: {
      label: "",
      kind: "Natal",
      birth: {
        date: "2000-01-01",
        time: "17:30",
        time_certainty: "Exact",
        place: { name: "New Delhi", country: "India", admin1: "Delhi", lat: 28.62137, lon: 77.2148, geoname_id: 1261481 },
        tz: { Iana: "Asia/Kolkata" },
        fold: null,
        notes: "",
      },
    },
  });
  expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Örnek Danışan");
  expect(onClose).toHaveBeenCalledOnce();
});

test("format errors show on blur and submit, clear on edit, and nothing is sent", async () => {
  renderForm("new");
  fireEvent.blur(input("Danışan adı"));
  expect(input("Danışan adı")).toHaveAttribute("aria-invalid", "true");
  expect(input("Danışan adı")).toHaveAccessibleDescription("Bu alan gerekli.");
  fireEvent.change(input("Danışan adı"), { target: { value: "A" } });
  expect(input("Danışan adı")).not.toHaveAttribute("aria-invalid");
  fireEvent.change(input("Enlem"), { target: { value: "kuzey" } });
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(input("Tarih")).toHaveAttribute("aria-invalid", "true");
  expect(input("Enlem")).toHaveAccessibleDescription("Ondalık sayı girin (ör. 28.61). Ondalık derece; kuzey ve doğu pozitif.");
  expect(api.createClient).not.toHaveBeenCalled();
  expect(api.previewMoment).not.toHaveBeenCalled();
});

test("a DST gap is shown on the time field with a one-click fix", async () => {
  vi.mocked(api.previewMoment).mockResolvedValueOnce({ status: "error", error: fixture().gap_error });
  renderForm("new");
  await fillDelhi();
  expect(input("Saat")).toHaveAttribute("aria-invalid", "true");
  expect(input("Saat")).toHaveAccessibleDescription(/2014-03-31T03:30:00 yerel saatte yok/);
  fireEvent.click(within(dialog()).getByRole("button", { name: "04:30 kullan" }));
  expect(input("Saat")).toHaveValue("04:30");
  expect(input("Tarih")).toHaveValue("2014-03-31");
});

test("an ambiguous time asks which occurrence and sends the choice", async () => {
  vi.mocked(api.previewMoment).mockResolvedValueOnce({
    status: "error",
    error: { code: "ambiguous_local_time", params: { local: "2014-10-26T03:30:00", earlier_offset_secs: "10800", later_offset_secs: "7200" } },
  });
  renderForm("new");
  await fillDelhi();
  const group = within(dialog()).getByRole("group", { name: "Bu saat iki kez yaşandı" });
  fireEvent.click(within(group).getByRole("radio", { name: "İkinci kez (geçişten sonra)" }));
  await wait(PREVIEW_DEBOUNCE_MS);
  expect(vi.mocked(api.previewMoment).mock.lastCall![0].fold).toBe("Later");
});

test("edit mode starts from the stored chart and saves with update_chart", async () => {
  renderForm("edit");
  expect(dialog()).toHaveAccessibleName("Haritayı düzenle");
  expect(input("Danışan adı")).toBeDisabled();
  // Ad alanı kilitli: odak yine pencereye girer, Esc tıklamadan çalışır.
  expect(dialog()).toContainElement(document.activeElement as HTMLElement);
  expect(document.activeElement).not.toBe(input("Danışan adı"));
  fireEvent.keyDown(document.activeElement!, { key: "Escape" });
  expect(onClose).toHaveBeenCalledOnce();
  onClose.mockReset();
  expect(input("Saat")).toHaveValue("17:30");
  fireEvent.change(input("Harita etiketi"), { target: { value: "Düzeltilmiş" } });
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  const { chart_record } = fixture().opened;
  expect(api.updateChart).toHaveBeenCalledWith(chart_record.id, { label: "Düzeltilmiş", kind: "Natal", birth: expect.objectContaining({ date: "2000-01-01", time: "17:30" }) });
  expect(api.createClient).not.toHaveBeenCalled();
});

test("saving replaces boot and store notices with the warnings of the returned chart", async () => {
  const stale: Notice[] = [
    { id: "boot", type: "error", messageKey: "notice.restoreFailed", error: { code: "store_sqlite", params: {} } },
    { id: "store:TzdbChanged", type: "warning", messageKey: "storeWarning.TzdbChanged", params: { stored: "2025a", current: "2025b" } },
    { id: "save", type: "error", messageKey: "notice.saveFailed", error: { code: "store_sqlite", params: {} } },
  ];
  function Seed() {
    const { push } = useNotices();
    useEffect(() => stale.forEach(push), [push]);
    return null;
  }
  const returned = fixture().opened;
  returned.warnings = ["LastChartMissing"];
  vi.mocked(api.updateChart).mockResolvedValueOnce({ status: "ok", data: returned });
  renderWithStores(
    <>
      <Seed />
      <NoticeBar />
      <BirthForm mode="edit" onClose={onClose} />
    </>,
    { opened: fixture().opened },
  );
  const bar = () => screen.getByRole("region", { name: "Bildirimler" });
  expect(within(bar()).getAllByRole("status")).toHaveLength(1);
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(onClose).toHaveBeenCalledOnce();
  expect(within(bar()).getByRole("status")).toHaveTextContent("Son açılan harita silinmiş");
  expect(bar()).not.toHaveTextContent("Saat dilimi verisi değişti");
  expect(bar()).not.toHaveTextContent("Son oturum açılamadı");
  // Kayıt hatası bildirimi otomatik kayda aittir; form onu kaldırmaz.
  expect(within(bar()).getAllByRole("alert")).toHaveLength(1);
});

test("save errors land on their field or above the buttons; Esc closes", async () => {
  vi.mocked(api.createClient).mockResolvedValueOnce({ status: "error", error: { code: "invalid_input", params: { field: "birth.date" } } });
  renderForm("new");
  await fillDelhi();
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(input("Tarih")).toHaveAccessibleDescription("Geçersiz değer: birth.date.");
  vi.mocked(api.createClient).mockResolvedValueOnce({ status: "error", error: { code: "store_sqlite", params: { detail: "disk" } } });
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(within(dialog()).getByRole("alert")).toHaveTextContent("Kaydedilemedi: Danışan veritabanı okunamadı ya da yazılamadı (disk).");
  expect(onClose).not.toHaveBeenCalled();
  fireEvent.keyDown(input("Tarih"), { key: "Escape" });
  expect(onClose).toHaveBeenCalledOnce();
});
