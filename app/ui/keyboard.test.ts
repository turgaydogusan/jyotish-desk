import { isEditableTarget } from "./keyboard";

test("inputs, textareas, selects and contenteditable are editable targets", () => {
  const input = document.createElement("input");
  const textarea = document.createElement("textarea");
  const select = document.createElement("select");
  const editable = document.createElement("div");
  editable.setAttribute("contenteditable", "true"); // jsdom contentEditable özelliğini uygulamaz
  const inner = document.createElement("span");
  editable.appendChild(inner);
  document.body.append(editable);
  expect(isEditableTarget(input)).toBe(true);
  expect(isEditableTarget(textarea)).toBe(true);
  expect(isEditableTarget(select)).toBe(true);
  expect(isEditableTarget(editable)).toBe(true);
  expect(isEditableTarget(inner)).toBe(true);
  editable.remove();
});

test("buttons, plain elements, window and null are not editable", () => {
  expect(isEditableTarget(document.createElement("button"))).toBe(false);
  expect(isEditableTarget(document.createElement("div"))).toBe(false);
  expect(isEditableTarget(window)).toBe(false);
  expect(isEditableTarget(null)).toBe(false);
});
