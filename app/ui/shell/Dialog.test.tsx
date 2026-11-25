import { fireEvent, render, screen } from "@testing-library/react";
import { useState, type ReactNode } from "react";
import { vi } from "vitest";
import { Dialog } from "./Dialog";

const onClose = vi.fn();
beforeEach(() => onClose.mockReset());

function open(children: ReactNode) {
  render(
    <Dialog title="Deneme" onClose={onClose}>
      {children}
    </Dialog>,
  );
  return screen.getByRole("dialog", { name: "Deneme" });
}
const button = (name: string) => screen.getByRole("button", { name });
/** jsdom Tab'da odağı taşımaz: tuzak `preventDefault` ile ölçülür (`fireEvent` o zaman `false` döner). */
const tab = (target: Element, shiftKey = false) => !fireEvent.keyDown(target, { key: "Tab", shiftKey });

const three = (
  <>
    <button>bir</button>
    <button>iki</button>
    <button>üç</button>
  </>
);

test("Tab wraps from the last element to the first and is left alone in between", () => {
  open(three);
  expect(document.activeElement).toBe(button("bir"));
  expect(tab(button("bir"))).toBe(false);
  expect(document.activeElement).toBe(button("bir"));
  button("iki").focus();
  expect(tab(button("iki"))).toBe(false);
  expect(tab(button("iki"), true)).toBe(false);
  button("üç").focus();
  expect(tab(button("üç"))).toBe(true);
  expect(document.activeElement).toBe(button("bir"));
});

test("Shift+Tab wraps from the first element to the last", () => {
  open(three);
  expect(tab(button("bir"), true)).toBe(true);
  expect(document.activeElement).toBe(button("üç"));
  expect(tab(button("üç"), true)).toBe(false);
});

test("a disabled autofocus target is skipped and disabled elements are not trap edges", () => {
  open(
    <>
      <input aria-label="ad" data-autofocus disabled />
      <button>iki</button>
      <button>üç</button>
      <button disabled>dört</button>
    </>,
  );
  expect(document.activeElement).toBe(button("iki"));
  expect(tab(button("iki"), true)).toBe(true);
  expect(document.activeElement).toBe(button("üç"));
  expect(tab(button("üç"))).toBe(true);
  expect(document.activeElement).toBe(button("iki"));
});

test("an enabled autofocus target wins over the first element", () => {
  open(
    <>
      <button>bir</button>
      <button data-autofocus>iki</button>
    </>,
  );
  expect(document.activeElement).toBe(button("iki"));
});

test("with focus on the dialog itself Tab enters at the first element and Shift+Tab at the last", () => {
  const dialog = open(three);
  dialog.focus();
  expect(tab(dialog, true)).toBe(true);
  expect(document.activeElement).toBe(button("üç"));
  dialog.focus();
  expect(tab(dialog)).toBe(true);
  expect(document.activeElement).toBe(button("bir"));
});

test("a dialog without focusable content keeps focus on itself", () => {
  const dialog = open(<p>metin</p>);
  expect(document.activeElement).toBe(dialog);
  expect(tab(dialog)).toBe(true);
  expect(tab(dialog, true)).toBe(true);
  expect(document.activeElement).toBe(dialog);
});

test("after focus leaves the dialog Tab brings it back and Esc still closes", () => {
  open(three);
  (document.activeElement as HTMLElement).blur();
  expect(document.activeElement).toBe(document.body);
  expect(tab(document.body)).toBe(true);
  expect(document.activeElement).toBe(button("bir"));
  (document.activeElement as HTMLElement).blur();
  expect(tab(document.body, true)).toBe(true);
  expect(document.activeElement).toBe(button("üç"));
  (document.activeElement as HTMLElement).blur();
  fireEvent.keyDown(document.body, { key: "Escape" });
  expect(onClose).toHaveBeenCalledOnce();
});

test("a backdrop press does not take focus; a press inside the dialog is left alone", () => {
  const dialog = open(three);
  expect(fireEvent.mouseDown(dialog.parentElement!)).toBe(false);
  expect(fireEvent.mouseDown(dialog)).toBe(true);
  expect(fireEvent.mouseDown(button("iki"))).toBe(true);
  expect(onClose).not.toHaveBeenCalled();
});

test("Esc closes without reaching window listeners; composition and inner handlers keep it", () => {
  const onWindowKey = vi.fn();
  window.addEventListener("keydown", onWindowKey);
  open(
    <>
      <button>bir</button>
      <input aria-label="liste" onKeyDown={(e) => e.key === "Escape" && e.stopPropagation()} />
    </>,
  );
  fireEvent.keyDown(button("bir"), { key: "Escape", isComposing: true });
  fireEvent.keyDown(button("bir"), { key: "Escape", keyCode: 229 });
  fireEvent.keyDown(screen.getByRole("textbox", { name: "liste" }), { key: "Escape" });
  expect(onClose).not.toHaveBeenCalled();
  onWindowKey.mockReset();
  expect(fireEvent.keyDown(button("bir"), { key: "Escape" })).toBe(false);
  expect(onClose).toHaveBeenCalledOnce();
  expect(onWindowKey).not.toHaveBeenCalled();
  window.removeEventListener("keydown", onWindowKey);
});

test("closing returns focus to the opener and removes the key listener", () => {
  function Host() {
    const [shown, setShown] = useState(false);
    return (
      <>
        <button onClick={() => setShown(true)}>aç</button>
        {shown && (
          <Dialog title="Deneme" onClose={() => (onClose(), setShown(false))}>
            <button>bir</button>
          </Dialog>
        )}
      </>
    );
  }
  render(<Host />);
  button("aç").focus();
  fireEvent.click(button("aç"));
  expect(document.activeElement).toBe(button("bir"));
  fireEvent.keyDown(button("bir"), { key: "Escape" });
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(document.activeElement).toBe(button("aç"));
  fireEvent.keyDown(document.body, { key: "Escape" });
  expect(onClose).toHaveBeenCalledOnce();
});

test("only the topmost dialog handles Esc and Tab", () => {
  const outer = vi.fn();
  const inner = vi.fn();
  const { rerender } = render(
    <Dialog title="Dış" onClose={outer}>
      <button>dış</button>
    </Dialog>,
  );
  // İç pencere sonra açılır (gerçek akış: onay penceresi formun üstünde).
  rerender(
    <Dialog title="Dış" onClose={outer}>
      <button>dış</button>
      <Dialog title="İç" onClose={inner}>
        <button>iç</button>
      </Dialog>
    </Dialog>,
  );
  expect(document.activeElement).toBe(button("iç"));
  expect(tab(button("iç"))).toBe(true);
  expect(document.activeElement).toBe(button("iç"));
  fireEvent.keyDown(button("iç"), { key: "Escape" });
  expect(inner).toHaveBeenCalledOnce();
  expect(outer).not.toHaveBeenCalled();
});
