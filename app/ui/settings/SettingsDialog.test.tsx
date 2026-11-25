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

test("disabled default actions name their reason", () => {
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />);
  for (const name of ["Varsayılana döndür", "Bunu varsayılan yap"]) {
    const button = screen.getByRole("button", { name });
    expect(button).toBeDisabled();
    expect(button).toHaveAccessibleDescription("Ayarlar varsayılanla aynı.");
  }
});

test("without readable preferences the default actions say why they are off", () => {
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />, { prefs: null });
  for (const name of ["Varsayılana döndür", "Bunu varsayılan yap"]) {
    const button = screen.getByRole("button", { name });
    expect(button).toBeDisabled();
    expect(button).toHaveAccessibleDescription("Tercihler okunamadı; varsayılanlar düzenlenemiyor.");
  }
});

test("a custom ayanamsa cannot become the default and the button says so", () => {
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />, {
    opened: openedWith({ ayanamsa: { Custom: { t0_jd: 2451545, deg: 23.85 } } }),
  });
  const make = screen.getByRole("button", { name: "Bunu varsayılan yap" });
  expect(make).toBeDisabled();
  expect(make).toHaveAccessibleDescription("Özel ayanamsa varsayılan yapılamaz.");
  const reset = screen.getByRole("button", { name: "Varsayılana döndür" });
  expect(reset).toBeEnabled();
  expect(reset).not.toHaveAccessibleDescription();
});

test("focus moves to close when the pressed default action turns itself off", async () => {
  vi.mocked(api.setAppPrefs).mockResolvedValue({ status: "ok", data: null });
  vi.mocked(api.saveCalcSettings).mockResolvedValue(okChart());
  const first = renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />, { opened: openedWith({ node: "True" }) });
  const make = screen.getByRole("button", { name: "Bunu varsayılan yap" });
  make.focus();
  fireEvent.click(make);
  await waitFor(() => expect(make).toBeDisabled());
  expect(screen.getByRole("button", { name: "Kapat" })).toHaveFocus();
  first.unmount();

  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />, { opened: openedWith({ node: "True" }) });
  const reset = screen.getByRole("button", { name: "Varsayılana döndür" });
  reset.focus();
  fireEvent.click(reset);
  await waitFor(() => expect(reset).toBeDisabled());
  expect(screen.getByRole("button", { name: "Kapat" })).toHaveFocus();
});

test("a make-default that fails after the dialog closed is reported in the notice bar", async () => {
  let fail!: () => void;
  vi.mocked(api.setAppPrefs).mockReturnValue(
    new Promise((r) => {
      fail = () => r({ status: "error", error: { code: "store_sqlite", params: { detail: "disk" } } });
    }),
  );
  function Host() {
    const [open, setOpen] = useState(true);
    return (
      <>
        {open && <SettingsDialog onClose={() => setOpen(false)} debounceMs={0} />}
        <NoticeBar />
      </>
    );
  }
  renderWithStores(<Host />, { opened: openedWith({ node: "True" }) });
  fireEvent.click(screen.getByRole("button", { name: "Bunu varsayılan yap" }));
  await waitFor(() => expect(api.setAppPrefs).toHaveBeenCalledTimes(1));
  fireEvent.click(screen.getByRole("button", { name: "Kapat" }));
  await act(async () => fail());
  await waitFor(() => expect(screen.getByRole("alert")).toHaveTextContent("Ayarlar kaydedilemedi"));
});

test("the made-default status survives a chart save that lands afterwards", async () => {
  let land!: () => void;
  vi.mocked(api.saveCalcSettings).mockReturnValue(
    new Promise((r) => {
      land = () => r(okChart());
    }),
  );
  vi.mocked(api.setAppPrefs).mockResolvedValue({ status: "ok", data: null });
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={10_000} />);
  fireEvent.change(screen.getByLabelText("Düğüm"), { target: { value: "True" } });
  fireEvent.click(screen.getByRole("button", { name: "Bunu varsayılan yap" }));
  await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("Yeni haritaların varsayılanı olarak kaydedildi."));
  expect(api.saveCalcSettings).toHaveBeenCalledTimes(1);
  await act(async () => land());
  expect(screen.getByRole("status")).toHaveTextContent("Yeni haritaların varsayılanı olarak kaydedildi.");
});

test("a pending edit is written when the page hides or unloads", async () => {
  vi.mocked(api.saveCalcSettings).mockResolvedValue(okChart());
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={10_000} />);
  fireEvent.change(screen.getByLabelText("Düğüm"), { target: { value: "True" } });
  expect(api.saveCalcSettings).not.toHaveBeenCalled();
  fireEvent(window, new Event("pagehide"));
  await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(1));

  fireEvent.change(screen.getByLabelText("Düğüm"), { target: { value: "Mean" } });
  const state = vi.spyOn(document, "visibilityState", "get").mockReturnValue("hidden");
  fireEvent(document, new Event("visibilitychange"));
  state.mockRestore();
  await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(2));
  expect(vi.mocked(api.saveCalcSettings).mock.calls[1][1].node).toBe("Mean");
});
