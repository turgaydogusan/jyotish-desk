import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    appInfo: vi.fn(),
    checkForUpdate: vi.fn(),
    installUpdate: vi.fn(),
  },
}));

import { api } from "../api";
import { UPDATE_STORAGE_KEY } from "../update/pref";
import { AboutDialog, SOURCE_URL } from "./AboutDialog";

const info = (updaterConfigured: boolean) => ({ status: "ok" as const, data: { version: "0.9.0", updaterConfigured } });
const found = { version: "0.9.1", notes: "- Düzeltme: baskı\n- Yeni: etiket", published: "2026-12-01" };

afterEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
});

async function open(updaterConfigured = true) {
  vi.mocked(api.appInfo).mockResolvedValue(info(updaterConfigured));
  render(<AboutDialog onClose={() => {}} />);
  const dialog = screen.getByRole("dialog", { name: "Jyotish Desk hakkında" });
  await within(dialog).findByText("Sürüm 0.9.0");
  return within(dialog);
}

test("shows the version, the licence, the source address and the data credits", async () => {
  const about = await open();
  expect(about.getByText(/AGPL-3\.0-or-later/)).toBeInTheDocument();
  expect(about.getByText(SOURCE_URL)).toBeInTheDocument();
  expect(SOURCE_URL).toBe("https://github.com/turgaydogusan/jyotish-desk");
  expect(about.getByText(/Swiss Ephemeris/)).toBeInTheDocument();
  expect(about.getByText(/GeoNames.*CC BY 4\.0/)).toBeInTheDocument();
  // Pencere tarayıcı açmaz: adres bağlantı değil, seçilebilir metindir.
  expect(about.queryByRole("link")).toBeNull();
});

test("opening the window asks nothing of the network and the startup check is off", async () => {
  const about = await open();
  expect(api.checkForUpdate).not.toHaveBeenCalled();
  const auto = about.getByRole("checkbox", { name: "Açılışta güncellemeleri denetle" });
  expect(auto).not.toBeChecked();
  expect(auto).toHaveAccessibleDescription(/danışan verisi, kimlik ya da kullanım bilgisi yoktur/);
  fireEvent.click(auto);
  expect(auto).toBeChecked();
  expect(localStorage.getItem(UPDATE_STORAGE_KEY)).toBe("auto");
  expect(api.checkForUpdate).not.toHaveBeenCalled();
  fireEvent.click(auto);
  expect(localStorage.getItem(UPDATE_STORAGE_KEY)).toBe("off");
});

test("a preference that cannot be stored stays off and says so", async () => {
  const about = await open();
  const setItem = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
    throw new Error("full");
  });
  const auto = about.getByRole("checkbox", { name: "Açılışta güncellemeleri denetle" });
  fireEvent.click(auto);
  setItem.mockRestore();
  expect(auto).not.toBeChecked();
  expect(about.getByRole("alert")).toHaveTextContent("Tercih saklanamadı; açılışta denetim yapılmayacak.");
});

test("a build without a signing key explains itself and offers no check", async () => {
  const about = await open(false);
  expect(about.getByText("Bu derlemede güncelleme denetimi yok (imza anahtarı gömülmemiş).")).toBeInTheDocument();
  expect(about.queryByRole("button", { name: "Şimdi denetle" })).toBeNull();
  expect(about.queryByRole("checkbox")).toBeNull();
});

test("check now reports an up-to-date app", async () => {
  vi.mocked(api.checkForUpdate).mockResolvedValue({ status: "ok", data: null });
  const about = await open();
  fireEvent.click(about.getByRole("button", { name: "Şimdi denetle" }));
  expect(await about.findByText("Kurulu sürüm güncel.")).toHaveAttribute("role", "status");
  expect(about.queryByRole("button", { name: /indir ve kur/ })).toBeNull();
});

test("a found version shows its notes and is installed only after a second, version-bound consent", async () => {
  vi.mocked(api.checkForUpdate).mockResolvedValue({ status: "ok", data: found });
  let finish!: (r: { status: "ok"; data: null }) => void;
  vi.mocked(api.installUpdate).mockReturnValue(new Promise((resolve) => (finish = resolve)));
  const about = await open();
  const check = about.getByRole("button", { name: "Şimdi denetle" });
  fireEvent.click(check);
  expect(await about.findByText("Yeni sürüm: 0.9.1 (2026-12-01).")).toBeInTheDocument();
  expect(about.getByLabelText("0.9.1 sürüm notları")).toHaveTextContent("- Düzeltme: baskı - Yeni: etiket");
  expect(api.installUpdate).not.toHaveBeenCalled();

  const install = about.getByRole("button", { name: "0.9.1 sürümünü indir ve kur" });
  fireEvent.click(install);
  expect(api.installUpdate).toHaveBeenCalledWith("0.9.1");
  expect(await about.findByText("0.9.1 indiriliyor ve imzası doğrulanıyor…")).toBeInTheDocument();
  // Sürerken düğmeler kilitlidir (yerel `disabled` değil) ve ikinci tıklama yeni istek göndermez.
  expect(install).toHaveAttribute("aria-disabled", "true");
  expect(check).toHaveAttribute("aria-disabled", "true");
  fireEvent.click(install);
  fireEvent.click(check);
  expect(api.installUpdate).toHaveBeenCalledTimes(1);
  expect(api.checkForUpdate).toHaveBeenCalledTimes(1);

  finish({ status: "ok", data: null });
  expect(await about.findByText("0.9.1 kuruldu. Yeni sürüm, uygulamayı kapatıp yeniden açınca çalışır.")).toBeInTheDocument();
  expect(about.queryByRole("button", { name: /indir ve kur/ })).toBeNull();
  expect(document.activeElement).toBe(about.getByRole("button", { name: "Kapat" }));
});

test("a failed check and a failed install are shown in the UI language and can be retried", async () => {
  vi.mocked(api.checkForUpdate).mockResolvedValueOnce({ status: "error", error: { code: "update_check_failed", params: { reason: "network" } } });
  const about = await open();
  fireEvent.click(about.getByRole("button", { name: "Şimdi denetle" }));
  expect(await about.findByRole("alert")).toHaveTextContent("Güncelleme denetlenemedi (ağa ulaşılamadı).");

  vi.mocked(api.checkForUpdate).mockResolvedValueOnce({ status: "ok", data: { ...found, published: null } });
  vi.mocked(api.installUpdate).mockResolvedValueOnce({ status: "error", error: { code: "update_install_failed", params: { reason: "signature" } } });
  fireEvent.click(about.getByRole("button", { name: "Şimdi denetle" }));
  expect(await about.findByText("Yeni sürüm: 0.9.1.")).toBeInTheDocument();
  expect(about.queryByRole("alert")).toBeNull();
  fireEvent.click(about.getByRole("button", { name: "0.9.1 sürümünü indir ve kur" }));
  expect(await about.findByRole("alert")).toHaveTextContent("Güncelleme kurulamadı (imza doğrulanamadı). Kurulu sürüm değişmedi.");
  // Kur düğmesi durur: kullanıcı yeniden deneyebilir.
  await waitFor(() => expect(about.getByRole("button", { name: "0.9.1 sürümünü indir ve kur" })).not.toHaveAttribute("aria-disabled"));
});
