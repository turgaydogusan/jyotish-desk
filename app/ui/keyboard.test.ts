import { isTextEntry } from "./keyboard";

const input = (type: string) => Object.assign(document.createElement("input"), { type });

test("text inputs, textareas and contenteditable are text entry", () => {
  const textarea = document.createElement("textarea");
  const editable = document.createElement("div");
  editable.setAttribute("contenteditable", "true"); // jsdom contentEditable özelliğini uygulamaz
  const inner = document.createElement("span");
  editable.appendChild(inner);
  document.body.append(editable);
  for (const type of ["text", "search", "number", "date", "time", "email"]) expect(isTextEntry(input(type)), type).toBe(true);
  expect(isTextEntry(textarea)).toBe(true);
  expect(isTextEntry(editable)).toBe(true);
  expect(isTextEntry(inner)).toBe(true);
  editable.remove();
});

test("selects, checkboxes, radios, buttons, plain elements, window and null are not text entry", () => {
  expect(isTextEntry(document.createElement("select"))).toBe(false);
  for (const type of ["checkbox", "radio", "button", "submit", "range"]) expect(isTextEntry(input(type)), type).toBe(false);
  expect(isTextEntry(document.createElement("button"))).toBe(false);
  expect(isTextEntry(document.createElement("div"))).toBe(false);
  expect(isTextEntry(window)).toBe(false);
  expect(isTextEntry(null)).toBe(false);
});
