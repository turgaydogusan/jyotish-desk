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
  expect(key({ key: "?", shiftKey: true })).toEqual({ id: "help" });
});

test("ignored: Ctrl/Alt, digit 0, repeat, composition, editable fields and open modals", () => {
  expect(key({ key: "1", code: "Digit1", ctrlKey: true })).toBeNull();
  expect(key({ key: "n", metaKey: true, altKey: true })).toBeNull();
  expect(key({ key: "0", code: "Digit0", metaKey: true })).toBeNull();
  expect(key({ key: "n", metaKey: true, repeat: true })).toBeNull();
  expect(key({ key: "?", isComposing: true })).toBeNull();
  expect(key({ key: "?" }, document.createElement("input"))).toBeNull();
  const modal = document.createElement("div");
  modal.setAttribute("aria-modal", "true");
  document.body.append(modal);
  expect(key({ key: "n", metaKey: true })).toBeNull();
  modal.remove();
  expect(key({ key: "n", metaKey: true })).toEqual({ id: "newChart" });
});

test("every listed shortcut has a description", () => {
  for (const s of SHORTCUT_LIST) expect(Object.keys(tr.shortcuts)).toContain(s.id);
  expect(Object.keys(tr.shortcuts).filter((k) => k !== "title" && k !== "open" && k !== "close" && k !== "keys" && k !== "action")).toEqual(
    SHORTCUT_LIST.map((s) => s.id),
  );
});
