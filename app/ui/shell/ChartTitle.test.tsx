import { fireEvent, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { ChartTitle } from "./ChartTitle";

test("no difference badge when the chart uses the default settings", () => {
  renderWithStores(<ChartTitle onSettings={() => {}} />);
  expect(screen.queryByRole("button", { name: /Ayar farkı/ })).toBeNull();
});

test("the difference badge names the differing fields and opens the settings", () => {
  const opened = fixture().opened;
  opened.calc_settings = { ...opened.calc_settings, node: "True", ayanamsa: "Raman" };
  const onSettings = vi.fn();
  renderWithStores(<ChartTitle onSettings={onSettings} />, { opened });
  const badge = screen.getByRole("button", { name: /Ayar farkı/ });
  expect(badge).toHaveAttribute("title", "Bu haritanın ayarı varsayılandan farklı: Ayanamsa, Düğüm");
  expect(badge).toHaveTextContent("≠");
  fireEvent.click(badge);
  expect(onSettings).toHaveBeenCalledTimes(1);
});

test("a differing advanced setting is named in the badge", () => {
  const opened = fixture().opened;
  opened.calc_settings = { ...opened.calc_settings, advanced: { karaka_scheme: "Eight", jaimini_co_lords: false } };
  renderWithStores(<ChartTitle onSettings={() => {}} />, { opened });
  expect(screen.getByRole("button", { name: /Ayar farkı/ })).toHaveAttribute("title", "Bu haritanın ayarı varsayılandan farklı: İleri modül ayarları");
});

test("a number prashna carries its number beside the chart label; a time prashna and other charts do not", () => {
  const { prashna } = fixture();
  const first = renderWithStores(<ChartTitle onSettings={() => {}} />, { opened: prashna.opened_number });
  const badge = screen.getByText("sayı: 54");
  expect(badge).toHaveAttribute("title", "Sayı prashnası: lagna sayıdan gelir (navamsa ortası); grahalar anın grahalarıdır.");
  // Açıklama ekran okuyucuya da okunur.
  expect(badge).toHaveTextContent("sayı: 54: Sayı prashnası: lagna sayıdan gelir (navamsa ortası); grahalar anın grahalarıdır.");
  expect(screen.getByText("Prashna 2026-11-25 15:30")).toBeInTheDocument();
  first.unmount();
  const second = renderWithStores(<ChartTitle onSettings={() => {}} />, { opened: prashna.opened });
  expect(screen.queryByText(/^sayı:/)).toBeNull();
  second.unmount();
  renderWithStores(<ChartTitle onSettings={() => {}} />);
  expect(screen.queryByText(/^sayı:/)).toBeNull();
});
