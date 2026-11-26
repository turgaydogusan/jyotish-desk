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

test("the karaka scheme and the co-lord rule are edited in the advanced group and saved with the chart", async () => {
  vi.mocked(api.saveCalcSettings).mockResolvedValue(okChart());
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />);
  const group = screen.getByRole("group", { name: "İleri modül ayarları" });
  const scheme = screen.getByLabelText("Chara karaka şeması");
  const coLords = screen.getByRole("checkbox", { name: "Jaimini eş lordları (Akrep: Mars ve Ketu; Kova: Satürn ve Rahu)" });
  expect(group).toContainElement(scheme);
  expect(group).toContainElement(coLords);
  expect(scheme).toHaveDisplayValue("Sekiz karaka (Rahu dahil)");
  expect(coLords).toBeChecked();
  fireEvent.change(scheme, { target: { value: "Seven" } });
  await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(1));
  expect(vi.mocked(api.saveCalcSettings).mock.calls[0][1].advanced).toEqual({ karaka_scheme: "Seven", jaimini_co_lords: true });
  expect(scheme).toHaveDisplayValue("Yedi karaka (Rahu hariç)");
  fireEvent.click(coLords);
  await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(2));
  expect(vi.mocked(api.saveCalcSettings).mock.calls[1][1].advanced).toEqual({ karaka_scheme: "Seven", jaimini_co_lords: false });
  // Grup varsayılandan ayrıldığını metinle söyler (renk tek başına değil).
  expect(group).toHaveTextContent("varsayılandan farklı");
  // Diğer alanlar değişmez.
  expect(vi.mocked(api.saveCalcSettings).mock.calls[1][1].node).toBe(fixture().opened.calc_settings.node);
});

test("an advanced difference is marked on its group and reset restores both fields", async () => {
  vi.mocked(api.saveCalcSettings).mockResolvedValue(okChart());
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />, {
    opened: openedWith({ advanced: { karaka_scheme: "Seven", jaimini_co_lords: false } }),
  });
  expect(screen.getAllByText("varsayılandan farklı")).toHaveLength(1);
  expect(screen.getByRole("group", { name: "İleri modül ayarları varsayılandan farklı" })).toBeInTheDocument();
  expect(screen.getByLabelText("Chara karaka şeması")).toHaveDisplayValue("Yedi karaka (Rahu hariç)");
  fireEvent.click(screen.getByRole("button", { name: "Varsayılana döndür" }));
  await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(1));
  expect(vi.mocked(api.saveCalcSettings).mock.calls[0][1].advanced).toEqual({ karaka_scheme: "Eight", jaimini_co_lords: true });
  await waitFor(() => expect(screen.queryByText("varsayılandan farklı")).toBeNull());
  expect(screen.getByRole("checkbox", { name: /Jaimini eş lordları/ })).toBeChecked();
});

test("without an open chart the advanced fields edit the defaults for new charts", async () => {
  vi.mocked(api.setAppPrefs).mockResolvedValue({ status: "ok", data: null });
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />, { opened: null });
  fireEvent.click(screen.getByRole("checkbox", { name: /Jaimini eş lordları/ }));
  await waitFor(() => expect(api.setAppPrefs).toHaveBeenCalledTimes(1));
  expect(vi.mocked(api.setAppPrefs).mock.calls[0][0].calc_settings.advanced).toEqual({ karaka_scheme: "Eight", jaimini_co_lords: false });
  expect(api.saveCalcSettings).not.toHaveBeenCalled();
  expect(screen.queryByText("varsayılandan farklı")).toBeNull();
});
