import { fireEvent, render, screen } from "@testing-library/react";
import { HighlightProvider, useHighlight } from "./store";
import { sameTarget } from "./target";

function Probe() {
  const h = useHighlight();
  return (
    <div>
      <span data-testid="active">{JSON.stringify(h.active)}</span>
      <span data-testid="pinned">{JSON.stringify(h.pinned)}</span>
      <button onClick={() => h.hover({ type: "graha", id: "Sun" })}>hover sun</button>
      <button onClick={() => h.hover(null)}>leave</button>
      <button onClick={() => h.togglePin({ type: "house", house: 4 })}>pin house 4</button>
    </div>
  );
}

const text = (id: string) => screen.getByTestId(id).textContent;

test("sameTarget compares by kind and value", () => {
  expect(sameTarget({ type: "graha", id: "Sun" }, { type: "graha", id: "Sun" })).toBe(true);
  expect(sameTarget({ type: "graha", id: "Sun" }, { type: "graha", id: "Moon" })).toBe(false);
  expect(sameTarget({ type: "house", house: 1 }, { type: "sign", sign: 1 })).toBe(false);
  expect(sameTarget(null, null)).toBe(true);
  expect(sameTarget(null, { type: "sign", sign: 0 })).toBe(false);
});

test("hover wins over pin while it lasts; pin toggles; Esc clears both", () => {
  render(<HighlightProvider><Probe /></HighlightProvider>);
  expect(text("active")).toBe("null");
  fireEvent.click(screen.getByText("pin house 4"));
  expect(text("active")).toBe('{"type":"house","house":4}');
  fireEvent.click(screen.getByText("hover sun"));
  expect(text("active")).toBe('{"type":"graha","id":"Sun"}');
  fireEvent.click(screen.getByText("leave"));
  expect(text("active")).toBe('{"type":"house","house":4}');
  fireEvent.click(screen.getByText("pin house 4"));
  expect(text("pinned")).toBe("null");
  fireEvent.click(screen.getByText("pin house 4"));
  fireEvent.click(screen.getByText("hover sun"));
  fireEvent.keyDown(window, { key: "Escape" });
  expect(text("active")).toBe("null");
  expect(text("pinned")).toBe("null");
});
