import { act, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { PanelError, PanelStatusContext, usePanelLoadPhase, type LoadPhase } from "./states";

function Phase({ loading }: { loading: boolean }) {
  return <span data-testid="phase">{usePanelLoadPhase(loading)}</span>;
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

test("no indicator under 100 ms, inline until 1 s, skeleton after", () => {
  const reported: LoadPhase[] = [];
  const { rerender } = render(
    <PanelStatusContext.Provider value={(p) => reported.push(p)}>
      <Phase loading />
    </PanelStatusContext.Provider>,
  );
  const phase = () => screen.getByTestId("phase").textContent;
  act(() => vi.advanceTimersByTime(99));
  expect(phase()).toBe("none");
  act(() => vi.advanceTimersByTime(1));
  expect(phase()).toBe("inline");
  act(() => vi.advanceTimersByTime(899));
  expect(phase()).toBe("inline");
  act(() => vi.advanceTimersByTime(1));
  expect(phase()).toBe("skeleton");
  rerender(
    <PanelStatusContext.Provider value={(p) => reported.push(p)}>
      <Phase loading={false} />
    </PanelStatusContext.Provider>,
  );
  expect(phase()).toBe("none");
  expect(reported).toEqual(["none", "inline", "skeleton", "none"]);
});

test("a fast load never shows an indicator", () => {
  const { rerender } = render(<Phase loading />);
  act(() => vi.advanceTimersByTime(50));
  rerender(<Phase loading={false} />);
  act(() => vi.advanceTimersByTime(2000));
  expect(screen.getByTestId("phase")).toHaveTextContent("none");
});

test("PanelError translates {code, params} and offers a retry", () => {
  const onRetry = vi.fn();
  render(<PanelError error={{ code: "data_files_missing", params: { path: "/eph" } }} onRetry={onRetry} />);
  expect(screen.getByRole("alert")).toHaveTextContent("Ephemeris dosyaları bulunamadı: /eph");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  expect(onRetry).toHaveBeenCalledOnce();
});

test("PanelError falls back to a generic message for unknown codes", () => {
  render(<PanelError error={{ code: "weird", params: {} }} onRetry={() => {}} />);
  expect(screen.getByRole("alert")).toHaveTextContent("Beklenmeyen bir sorun oluştu (weird).");
});

test("PanelError cannot let backend params override i18next options", () => {
  render(<PanelError error={{ code: "data_files_missing", params: { path: "/eph", lng: "en" } }} onRetry={() => {}} />);
  expect(screen.getByRole("alert")).toHaveTextContent("Ephemeris dosyaları bulunamadı: /eph");
});

/** Yeniden deneme hata kutusunu kaldıran panel: "Yeniden dene" düğmesi DOM'dan çıkar. */
function Retrying({ framed }: { framed: boolean }) {
  const [failed, setFailed] = useState(true);
  const body = failed ? <PanelError error={{ code: "weird", params: {} }} onRetry={() => setFailed(false)} /> : <p>loaded</p>;
  return (
    <main id="shell-main" tabIndex={-1}>
      {framed ? (
        <section className="panel" tabIndex={0} aria-label="Bakışlar">
          {body}
        </section>
      ) : (
        body
      )}
    </main>
  );
}

test("when retry removes the focused button, focus moves to the panel, not to the body", () => {
  render(<Retrying framed />);
  const retry = screen.getByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  expect(screen.getByText("loaded")).toBeInTheDocument();
  expect(document.activeElement).not.toBe(document.body);
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Bakışlar" }));
});

test("outside a panel frame the focus falls back to the main region", () => {
  render(<Retrying framed={false} />);
  const retry = screen.getByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  expect(document.activeElement).toBe(screen.getByRole("main"));
});

test("an error box removed while focus is elsewhere leaves the focus alone", () => {
  render(
    <>
      <button type="button">other</button>
      <Retrying framed />
    </>,
  );
  const other = screen.getByRole("button", { name: "other" });
  other.focus();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  expect(document.activeElement).toBe(other);
});
