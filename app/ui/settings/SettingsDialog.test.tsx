import { act, fireEvent, screen, waitFor } from "@testing-library/react";
import { useState } from "react";
import { afterEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    saveCalcSettings: vi.fn(),
    setAppPrefs: vi.fn(),
  },
}));

import { api } from "../api";
import type { CalcSettings, OpenedClient } from "../bindings";
import { useSession } from "../session/store";
import { NoticeBar } from "../shell/notices";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { SettingsDialog } from "./SettingsDialog";

afterEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
});

function Probe() {
  const { opened } = useSession();
  // `<output>` örtük `role="status"` taşır ve pencerenin durum satırıyla karışır; `<span>` kullanılır.
  return <span data-testid="node">{opened?.calc_settings.node}</span>;
}

function openedWith(patch: Partial<CalcSettings>): OpenedClient {
  const o = fixture().opened;
  o.calc_settings = { ...o.calc_settings, ...patch };
  return o;
}

const okChart = () => ({ status: "ok" as const, data: fixture().opened.chart });

test("an edit is saved after the debounce and the open chart takes the new settings", async () => {
  vi.mocked(api.saveCalcSettings).mockResolvedValue(okChart());
  renderWithStores(
    <>
      <SettingsDialog onClose={() => {}} debounceMs={0} />
      <Probe />
    </>,
  );
  fireEvent.change(screen.getByLabelText("Düğüm"), { target: { value: "True" } });
  await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(1));
  const [chartId, settings] = vi.mocked(api.saveCalcSettings).mock.calls[0];
  expect(chartId).toBe(fixture().opened.chart_record.id);
  expect(settings.node).toBe("True");
  await waitFor(() => expect(screen.getByTestId("node")).toHaveTextContent("True"));
  expect(screen.getByRole("status")).toHaveTextContent("Kaydedildi.");
});

test("rapid edits are written once and the last one wins", async () => {
  vi.mocked(api.saveCalcSettings).mockResolvedValue(okChart());
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={50} />);
  fireEvent.change(screen.getByLabelText("Ev sistemi"), { target: { value: "Placidus" } });
  fireEvent.change(screen.getByLabelText("Ev sistemi"), { target: { value: "Sripati" } });
  await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(1));
  expect(vi.mocked(api.saveCalcSettings).mock.calls[0][1].house_system).toBe("Sripati");
});

test("fields that differ from the default are marked and reset to default saves the default", async () => {
  vi.mocked(api.saveCalcSettings).mockResolvedValue(okChart());
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />, { opened: openedWith({ node: "True" }) });
  expect(screen.getAllByText("varsayılandan farklı")).toHaveLength(1);
  fireEvent.click(screen.getByRole("button", { name: "Varsayılana döndür" }));
  await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(1));
  expect(vi.mocked(api.saveCalcSettings).mock.calls[0][1]).toEqual(fixture().prefs.prefs.calc_settings);
  await waitFor(() => expect(screen.queryByText("varsayılandan farklı")).toBeNull());
});

test("make default writes the app prefs with the current language", async () => {
  vi.mocked(api.setAppPrefs).mockResolvedValue({ status: "ok", data: null });
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />, { opened: openedWith({ node: "True" }) });
  fireEvent.click(screen.getByRole("button", { name: "Bunu varsayılan yap" }));
  await waitFor(() => expect(api.setAppPrefs).toHaveBeenCalledTimes(1));
  const prefs = vi.mocked(api.setAppPrefs).mock.calls[0][0];
  expect(prefs.locale).toBe("tr");
  expect(prefs.calc_settings.node).toBe("True");
  await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("Yeni haritaların varsayılanı olarak kaydedildi."));
  expect(screen.queryByText("varsayılandan farklı")).toBeNull();
});

test("an invalid orb is flagged and not saved; a custom ayanamsa is shown but locked", async () => {
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />, {
    opened: openedWith({ ayanamsa: { Custom: { t0_jd: 2451545, deg: 23.85 } } }),
  });
  const ayanamsa = screen.getByLabelText("Ayanamsa");
  expect(ayanamsa).toBeDisabled();
  expect(ayanamsa).toHaveAccessibleDescription("İçe aktarılmış özel ayanamsa burada değiştirilemez.");
  const moon = screen.getByLabelText("Ay");
  fireEvent.change(moon, { target: { value: "abc" } });
  expect(moon).toHaveAttribute("aria-invalid", "true");
  expect(moon).toHaveAccessibleDescription("0 ile 30 arasında bir sayı girin.");
  await act(() => new Promise((r) => setTimeout(r, 20)));
  expect(api.saveCalcSettings).not.toHaveBeenCalled();
});

test("a save that fails after the dialog closed is reported in the notice bar", async () => {
  vi.mocked(api.saveCalcSettings).mockResolvedValue({ status: "error", error: { code: "store_sqlite", params: { detail: "disk" } } });
  function Host() {
    const [open, setOpen] = useState(true);
    return (
      <>
        {open && <SettingsDialog onClose={() => setOpen(false)} debounceMs={10_000} />}
        <NoticeBar />
      </>
    );
  }
  renderWithStores(<Host />);
  fireEvent.change(screen.getByLabelText("Düğüm"), { target: { value: "True" } });
  fireEvent.click(screen.getByRole("button", { name: "Kapat" }));
  await waitFor(() => expect(screen.getByRole("alert")).toHaveTextContent("Ayarlar kaydedilemedi"));
  expect(api.saveCalcSettings).toHaveBeenCalledTimes(1);
});
