import { matchShortcut, SHORTCUT_LIST } from "./shortcuts";
import { tr } from "./i18n/tr";

const key = (init: KeyboardEventInit, target: EventTarget = document.body) => {
  const e = new KeyboardEvent("keydown", init);
  Object.defineProperty(e, "target", { value: target });
  return matchShortcut(e);
};

test("global commands", () => {
  expect(key({ key: "3", code: "Digit3", metaKey: true })).toEqual({ id: "selectLayout", index: 2 });
  expect(key({ key: "\"", code: "Digit3", metaKey: true })).toEqual({ id: "selectLayout", index: 2 });
  expect(key({ key: "N", metaKey: true, shiftKey: true })).toEqual({ id: "toggleNotes" });
  expect(key({ key: "n", metaKey: true })).toEqual({ id: "newChart" });
  expect(key({ key: "e", metaKey: true })).toEqual({ id: "editChart" });
  expect(key({ key: "k", metaKey: true })).toEqual({ id: "palette" });
  expect(key({ key: "\\", code: "Backslash", metaKey: true })).toEqual({ id: "sidebar" });
  expect(key({ key: ",", code: "Backslash", metaKey: true })).toEqual({ id: "sidebar" });
  expect(key({ key: "?", shiftKey: true })).toEqual({ id: "help" });
});

test("ignored: Ctrl/Alt, digit 0, repeat, composition, editable fields and open modals", () => {
  expect(key({ key: "1", code: "Digit1", ctrlKey: true })).toBeNull();
  expect(key({ key: "n", metaKey: true, altKey: true })).toBeNull();
  expect(key({ key: "0", code: "Digit0", metaKey: true })).toBeNull();
  expect(key({ key: "n", metaKey: true, repeat: true })).toBeNull();
  expect(key({ key: "?", isComposing: true })).toBeNull();
  expect(key({ key: "k", metaKey: true, keyCode: 229 })).toBeNull();
  expect(key({ key: "?" }, document.createElement("input"))).toBeNull();
  const modal = document.createElement("div");
  modal.setAttribute("aria-modal", "true");
  document.body.append(modal);
  expect(key({ key: "n", metaKey: true })).toBeNull();
  modal.remove();
  expect(key({ key: "n", metaKey: true })).toEqual({ id: "newChart" });
});

test("Cmd shortcuts and ? work from a select, checkbox or radio, and Cmd shortcuts also work from text entry, where only ? is ignored", () => {
  const select = document.createElement("select");
  const checkbox = Object.assign(document.createElement("input"), { type: "checkbox" });
  const radio = Object.assign(document.createElement("input"), { type: "radio" });
  for (const target of [select, checkbox, radio]) {
    expect(key({ key: "2", code: "Digit2", metaKey: true }, target)).toEqual({ id: "selectLayout", index: 1 });
    expect(key({ key: "k", metaKey: true }, target)).toEqual({ id: "palette" });
    expect(key({ key: "?", shiftKey: true }, target)).toEqual({ id: "help" });
  }
  for (const target of [document.createElement("input"), document.createElement("textarea")]) {
    expect(key({ key: "k", metaKey: true }, target)).toEqual({ id: "palette" });
    expect(key({ key: "N", metaKey: true, shiftKey: true }, target)).toEqual({ id: "toggleNotes" });
    expect(key({ key: "?", shiftKey: true }, target)).toBeNull();
  }
});

test("every listed shortcut has a description", () => {
  for (const s of SHORTCUT_LIST) expect(Object.keys(tr.shortcuts)).toContain(s.id);
  expect(Object.keys(tr.shortcuts).filter((k) => k !== "title" && k !== "open" && k !== "close" && k !== "keys" && k !== "action")).toEqual(
    SHORTCUT_LIST.map((s) => s.id),
  );
});

test("⌘, opens the settings, also from a text field", () => {
  const input = document.createElement("input");
  document.body.append(input);
  const e = new KeyboardEvent("keydown", { key: ",", code: "Comma", metaKey: true, bubbles: true });
  Object.defineProperty(e, "target", { value: input });
  expect(matchShortcut(e)).toEqual({ id: "settings" });
  expect(SHORTCUT_LIST.some((s) => s.id === "settings" && s.keys === "⌘,")).toBe(true);
  input.remove();
});
