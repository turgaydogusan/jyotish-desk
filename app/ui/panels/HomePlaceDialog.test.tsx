import { act, fireEvent, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { setAppPrefs: vi.fn(), searchPlaces: vi.fn() } }));

import { api } from "../api";
import type { AppPrefs, HomePlace } from "../bindings";
import { SEARCH_DEBOUNCE_MS } from "../birth/PlaceSearch";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { usePrefs } from "../settings/prefs";
import { HomePlaceDialog, tzLabel } from "./HomePlaceDialog";

const FX = fixture();
const DELHI: HomePlace = FX.muhurta.home_place;
const withHome = (home: HomePlace | null): AppPrefs => ({ ...FX.prefs.prefs, home_place: home });

let prefs: AppPrefs | null;
function Probe() {
  prefs = usePrefs().prefs;
  return null;
}

/** Seçili yeri söyleyen satır (yer arama kutusunun da bir durum satırı vardır). */
const current = () => document.querySelector(".home-place-current");

function setup(initial: AppPrefs | null = withHome(null)) {
  const onClose = vi.fn();
  renderWithStores(
    <>
      <HomePlaceDialog opened={FX.opened} onClose={onClose} />
      <Probe />
    </>,
    { prefs: initial },
  );
  return { onClose };
}

beforeEach(() => {
  vi.mocked(api.setAppPrefs).mockReset();
  vi.mocked(api.searchPlaces).mockReset();
  vi.mocked(api.setAppPrefs).mockResolvedValue({ status: "ok", data: null });
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: FX.places });
});

test("with no place set the dialog offers the birth place of the open chart and saves it", async () => {
  const { onClose } = setup();
  expect(screen.getByRole("dialog", { name: "Bulunulan yer" })).toBeInTheDocument();
  expect(current()).toHaveAttribute("role", "status");
  expect(current()).toHaveTextContent("Bulunulan yer seçilmedi.");
  expect(screen.getByText(/Konum servisi kullanılmaz/)).toBeInTheDocument();
  // Seçilmemişken kaldırma eylemi yoktur; odak önerilen eylemdedir.
  expect(screen.queryByRole("button", { name: "Yeri kaldır" })).toBeNull();
  const use = screen.getByRole("button", { name: "Açık haritanın doğum yerini kullan (New Delhi, Delhi, India)" });
  expect(use).toHaveFocus();
  fireEvent.click(use);
  await waitFor(() => expect(prefs?.home_place).toEqual(DELHI));
  // Diğer tercihler aynen yazılır; yer doğum verisindeki saat dilimiyle saklanır.
  expect(api.setAppPrefs).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.setAppPrefs).mock.calls[0][0]).toEqual(withHome(DELHI));
  expect(current()).toHaveTextContent("Seçili yer: New Delhi, Delhi, India · Asia/Kolkata");
  expect(screen.getByRole("button", { name: "Yeri kaldır" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Kapat" }));
  expect(onClose).toHaveBeenCalledTimes(1);
});

test("a place picked from the atlas is saved with its coordinates and its IANA zone", async () => {
  vi.useFakeTimers();
  try {
    setup();
    const input = screen.getByRole("combobox", { name: "Yer ara" });
    fireEvent.change(input, { target: { value: "delhi" } });
    await act(async () => vi.advanceTimersByTime(SEARCH_DEBOUNCE_MS));
    expect(api.searchPlaces).toHaveBeenCalledWith("delhi", 8);
    fireEvent.keyDown(input, { key: "Enter" });
    await act(async () => vi.advanceTimersByTime(0));
    const hit = FX.places[0];
    expect(vi.mocked(api.setAppPrefs).mock.calls[0][0].home_place).toEqual({
      place: { name: "New Delhi", country: "India", admin1: "Delhi", lat: hit.lat, lon: hit.lon, geoname_id: hit.geoname_id },
      tz: { Iana: "Asia/Kolkata" },
    });
    expect(input).toHaveValue("");
    expect(current()).toHaveTextContent("Seçili yer: New Delhi, Delhi, India · Asia/Kolkata");
  } finally {
    vi.useRealTimers();
  }
});

test("removing the place clears it and moves the focus to the next action instead of dropping it", async () => {
  setup(withHome(DELHI));
  const remove = screen.getByRole("button", { name: "Yeri kaldır" });
  remove.focus();
  fireEvent.click(remove);
  await waitFor(() => expect(prefs?.home_place).toBeNull());
  expect(vi.mocked(api.setAppPrefs).mock.calls[0][0]).toEqual(withHome(null));
  expect(remove).not.toBeInTheDocument();
  expect(screen.getByRole("button", { name: /^Açık haritanın doğum yerini kullan/ })).toHaveFocus();
  expect(document.activeElement).not.toBe(document.body);
  expect(current()).toHaveTextContent("Bulunulan yer seçilmedi.");
});

test("a failed save is reported in the dialog and the place does not change", async () => {
  vi.mocked(api.setAppPrefs).mockResolvedValueOnce({ status: "error", error: { code: "invalid_input", params: { field: "home_place" } } });
  setup(withHome(DELHI));
  fireEvent.click(screen.getByRole("button", { name: "Yeri kaldır" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Geçersiz değer: home_place.");
  expect(prefs?.home_place).toEqual(DELHI);
  expect(screen.getByRole("button", { name: "Yeri kaldır" })).toBeInTheDocument();
  // Sonraki başarılı kayıt hatayı kaldırır.
  fireEvent.click(screen.getByRole("button", { name: "Yeri kaldır" }));
  await waitFor(() => expect(prefs?.home_place).toBeNull());
  expect(screen.queryByRole("alert")).toBeNull();
});

test("without readable preferences the place cannot be changed, and Esc closes the dialog", () => {
  const { onClose } = setup(null);
  expect(screen.getByRole("note")).toHaveTextContent("Tercihler okunamadığı için bulunulan yer değiştirilemiyor.");
  expect(screen.queryByRole("combobox")).toBeNull();
  expect(screen.queryByRole("button", { name: /doğum yerini kullan/ })).toBeNull();
  fireEvent.keyDown(document, { key: "Escape" });
  expect(onClose).toHaveBeenCalledTimes(1);
  expect(api.setAppPrefs).not.toHaveBeenCalled();
});

test("the zone is written as an IANA id, a signed offset or local mean time", () => {
  const at = (tz: HomePlace["tz"]) => tzLabel({ ...DELHI, tz }, "yerel ortalama saat");
  expect(at({ Iana: "Asia/Kolkata" })).toBe("Asia/Kolkata");
  expect(at({ Fixed: 19800 })).toBe("UTC+05:30");
  expect(at({ Fixed: -12600 })).toBe("UTC−03:30");
  expect(at({ Fixed: 0 })).toBe("UTC+00:00");
  expect(at("Lmt")).toBe("yerel ortalama saat");
});
