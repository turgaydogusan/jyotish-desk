import { leaveNotes, leaveSidebar, MAIN_ID } from "./focus";

afterEach(() => (document.body.innerHTML = ""));

test("leaving the notes panel moves focus to main; focus elsewhere is left alone", () => {
  document.body.innerHTML = `<main id="${MAIN_ID}" tabindex="-1"><div class="notes-panel"><textarea></textarea></div><button>diğer</button></main>`;
  const box = document.querySelector("textarea")!;
  box.focus();
  leaveNotes();
  expect(document.activeElement).toBe(document.getElementById(MAIN_ID));
  const other = document.querySelector("button")!;
  other.focus();
  leaveNotes();
  expect(document.activeElement).toBe(other);
});

test("leaving the sidebar moves focus to its toggle", () => {
  document.body.innerHTML = `<button aria-controls="client-sidebar">aç</button><aside id="client-sidebar"><input /></aside>`;
  document.querySelector("input")!.focus();
  leaveSidebar();
  expect(document.activeElement).toBe(document.querySelector("button"));
});
