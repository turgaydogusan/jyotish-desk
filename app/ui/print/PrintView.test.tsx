import { fireEvent, screen, within } from "@testing-library/react";
import { useState } from "react";
import { vi } from "vitest";
import { renderWithStores } from "../test/render";
import { PRINT_STORAGE_KEY } from "./options";
import { PrintView } from "./PrintView";

const renderView = (onPrint = vi.fn(), error: Parameters<typeof PrintView>[0]["error"] = null) => ({
  onPrint,
  ...renderWithStores(<PrintView error={error} onPrint={onPrint} />),
});
const sheets = (container: HTMLElement) => [...container.querySelectorAll("section.print-page")].map((s) => s.getAttribute("data-page"));
const toolbar = () => screen.getByRole("group", { name: "Baskı seçenekleri" });

test("the preview shows the four sheets; the toolbar is screen-only and its button asks for the print dialog", () => {
  const { container, onPrint } = renderView();
  expect(sheets(container)).toEqual(["cover", "charts", "dasha", "grahas"]);
  expect(toolbar()).toHaveClass("no-print");
  expect(toolbar().querySelector(".print-page")).toBeNull();
  const print = within(toolbar()).getByRole("button", { name: "Yazdır…" });
  expect(print).toHaveFocus();
  expect(onPrint).not.toHaveBeenCalled();
  fireEvent.click(print);
  expect(onPrint).toHaveBeenCalledOnce();
  expect(within(toolbar()).getByText(/PDF olarak kaydet/)).toBeInTheDocument();
});

test("unchecking a page removes its sheet and renumbers; the choice is stored", () => {
  const { container } = renderView();
  fireEvent.click(within(toolbar()).getByRole("checkbox", { name: "Vimshottari dasa" }));
  expect(sheets(container)).toEqual(["cover", "charts", "grahas"]);
  expect([...container.querySelectorAll(".print-running .num")].map((n) => n.textContent)).toEqual(["1 / 3", "2 / 3", "3 / 3"]);
  expect(JSON.parse(localStorage.getItem(PRINT_STORAGE_KEY)!)).toEqual({ pages: ["cover", "charts", "grahas"], style: "north" });
});

test("the last page cannot be unchecked and says why", () => {
  localStorage.setItem(PRINT_STORAGE_KEY, JSON.stringify({ pages: ["grahas"], style: "north" }));
  const { container } = renderView();
  const only = within(toolbar()).getByRole("checkbox", { name: "Graha tablosu" });
  expect(only).toHaveAttribute("aria-disabled", "true");
  expect(only).toHaveAccessibleDescription("En az bir sayfa seçili olmalıdır.");
  fireEvent.click(only);
  expect(only).toBeChecked();
  expect(sheets(container)).toEqual(["grahas"]);
  fireEvent.click(within(toolbar()).getByRole("checkbox", { name: "Kapak" }));
  expect(sheets(container)).toEqual(["cover", "grahas"]);
  expect(only).not.toHaveAttribute("aria-disabled");
  expect(screen.queryByText("En az bir sayfa seçili olmalıdır.")).toBeNull();
});

test("the chart style applies to both printed charts and is restored from storage", () => {
  localStorage.setItem(PRINT_STORAGE_KEY, JSON.stringify({ pages: ["charts"], style: "circle" }));
  const { container } = renderView();
  const style = within(toolbar()).getByRole("combobox", { name: "Harita biçimi" });
  expect(style).toHaveValue("circle");
  fireEvent.change(style, { target: { value: "south" } });
  expect([...container.querySelectorAll("svg.chart")].map((s) => s.getAttribute("data-style"))).toEqual(["south", "south"]);
  expect(JSON.parse(localStorage.getItem(PRINT_STORAGE_KEY)!).style).toBe("south");
});

test("the document title names the client and chart while the preview is open", () => {
  document.title = "Jyotish Desk";
  const { unmount } = renderView();
  expect(document.title).toBe("Örnek Danışan – Doğum haritası");
  unmount();
  expect(document.title).toBe("Jyotish Desk");
});

test("a print error is shown in the toolbar with a retry that prints again", () => {
  const { onPrint } = renderView(vi.fn(), { code: "print_failed", params: {} });
  expect(within(toolbar()).getByRole("alert")).toHaveTextContent("Yazdırma penceresi açılamadı.");
  fireEvent.click(within(toolbar()).getByRole("button", { name: "Yeniden dene" }));
  expect(onPrint).toHaveBeenCalledOnce();
});

test("retry after a print error keeps focus off the body when the error box goes away", () => {
  function Harness() {
    const [error, setError] = useState<Parameters<typeof PrintView>[0]["error"]>({ code: "print_failed", params: {} });
    return <PrintView error={error} onPrint={() => setError(null)} />;
  }
  renderWithStores(<Harness />);
  const retry = within(toolbar()).getByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  expect(within(toolbar()).queryByRole("alert")).toBeNull();
  expect(within(toolbar()).getByRole("button", { name: "Yazdır…" })).toHaveFocus();
});

test("without an open chart the preview renders nothing", () => {
  const { container } = renderWithStores(<PrintView error={null} onPrint={() => {}} />, { opened: null });
  expect(container.querySelector(".print-view")).toBeNull();
});
