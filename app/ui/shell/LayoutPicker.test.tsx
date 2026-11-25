import { readFileSync } from "node:fs";
import { fireEvent, render, screen, within } from "@testing-library/react";
import { BUILTIN_LAYOUT_IDS, MAX_LAYOUTS } from "../layout/model";
import { HighlightProvider, useHighlight } from "../highlight/store";
import { initialViewModel } from "../view/model";
import { memoryPersistence } from "../view/persistence";
import { useLayoutStore, ViewStateProvider } from "../view/store";
import { LayoutPicker } from "./LayoutPicker";

function ActiveId() {
  return <output data-testid="active">{useLayoutStore().active.id}</output>;
}

function Pin() {
  const h = useHighlight();
  return (
    <>
      <button type="button" onClick={() => h.togglePin({ type: "sign", sign: 0 })}>
        pin
      </button>
      <output data-testid="pinned">{h.pinned ? "pinned" : "none"}</output>
    </>
  );
}

function setup() {
  render(
    <ViewStateProvider initial={initialViewModel(null)} persistence={memoryPersistence()}>
      <HighlightProvider>
        <LayoutPicker />
        <ActiveId />
        <Pin />
        <button type="button">outside</button>
      </HighlightProvider>
    </ViewStateProvider>,
  );
}

const picker = () => screen.getByRole("combobox", { name: "Düzen" });
const menu = () => fireEvent.click(screen.getByRole("button", { name: "Düzen işlemleri" }));
const item = (name: string) => screen.getByRole("menuitem", { name });
const trigger = () => screen.getByRole("button", { name: "Düzen işlemleri" });
const options = () => [...picker().querySelectorAll("option")].map((o) => o.textContent);
const saveAsNamed = (value: string) => {
  menu();
  fireEvent.click(item("Farklı kaydet…"));
  fireEvent.change(screen.getByRole("textbox", { name: "Düzen adı" }), { target: { value } });
  fireEvent.click(screen.getByRole("button", { name: "Kaydet" }));
};

test("lists builtin layouts by translated name and selects on change", () => {
  setup();
  expect([...picker().querySelectorAll("option")].map((o) => o.textContent)).toEqual(["Görüşme", "Araştırma", "Transit"]);
  fireEvent.change(picker(), { target: { value: BUILTIN_LAYOUT_IDS.transit } });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
});

test("builtin layouts can be reset but not renamed or deleted", () => {
  setup();
  menu();
  expect(item("Yeniden adlandır…")).toHaveAttribute("aria-disabled", "true");
  expect(item("Yeniden adlandır…")).toHaveAccessibleDescription("Hazır düzenler yeniden adlandırılamaz ve silinemez.");
  expect(item("Sil…")).toHaveAttribute("aria-disabled", "true");
  fireEvent.click(item("Sil…"));
  expect(screen.queryByText(/düzeni silinsin mi\?/)).toBeNull();
  expect(screen.getByRole("menu")).toBeInTheDocument();
  expect(item("Varsayılana döndür")).toBeEnabled();
  fireEvent.keyDown(screen.getByRole("menu"), { key: "Escape" });
  expect(screen.queryByRole("menu")).toBeNull();
});

test("save as asks for a name inline, then selects the copy", () => {
  setup();
  menu();
  fireEvent.click(item("Farklı kaydet…"));
  const input = screen.getByRole("textbox", { name: "Düzen adı" });
  expect(input).toHaveValue("Görüşme kopyası");
  fireEvent.change(input, { target: { value: "  " } });
  expect(screen.getByRole("button", { name: "Kaydet" })).toBeDisabled();
  fireEvent.change(input, { target: { value: "Sabah" } });
  fireEvent.click(screen.getByRole("button", { name: "Kaydet" }));
  expect(picker()).toHaveDisplayValue("Sabah");
  expect(screen.getByTestId("active").textContent).toMatch(/^user:/);
});

test("user layouts can be renamed and deleted after a named confirmation", () => {
  setup();
  menu();
  fireEvent.click(item("Farklı kaydet…"));
  fireEvent.change(screen.getByRole("textbox", { name: "Düzen adı" }), { target: { value: "Sabah" } });
  fireEvent.click(screen.getByRole("button", { name: "Kaydet" }));

  menu();
  fireEvent.click(item("Yeniden adlandır…"));
  fireEvent.change(screen.getByRole("textbox", { name: "Düzen adı" }), { target: { value: "Akşam" } });
  fireEvent.click(screen.getByRole("button", { name: "Kaydet" }));
  expect(picker()).toHaveDisplayValue("Akşam");

  menu();
  fireEvent.click(item("Sil…"));
  expect(screen.getByText("“Akşam” düzeni silinsin mi? Bu işlem geri alınamaz.")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Vazgeç" }));
  expect(picker()).toHaveDisplayValue("Akşam");

  menu();
  fireEvent.click(item("Sil…"));
  fireEvent.click(screen.getByRole("button", { name: "“Akşam” düzenini sil" }));
  expect(picker()).toHaveDisplayValue("Görüşme");
  expect([...picker().querySelectorAll("option")]).toHaveLength(3);
});

test("duplicate adds a copy after the source without switching", () => {
  setup();
  menu();
  fireEvent.click(item("Kopyala"));
  expect([...picker().querySelectorAll("option")].map((o) => o.textContent)).toEqual([
    "Görüşme",
    "Görüşme kopyası",
    "Araştırma",
    "Transit",
  ]);
  expect(picker()).toHaveDisplayValue("Görüşme");
});

test("save as and duplicate are disabled at the layout limit", () => {
  setup();
  for (let i = 3; i < MAX_LAYOUTS; i++) {
    menu();
    fireEvent.click(item("Kopyala"));
  }
  expect(options()).toHaveLength(MAX_LAYOUTS);
  menu();
  expect(item("Farklı kaydet…")).toHaveAttribute("aria-disabled", "true");
  expect(item("Kopyala")).toHaveAttribute("aria-disabled", "true");
  expect(item("Kopyala")).toHaveAccessibleDescription(`En çok ${MAX_LAYOUTS} düzen kaydedilebilir.`);
  fireEvent.click(item("Kopyala"));
  expect(options()).toHaveLength(MAX_LAYOUTS);
});

test("a disabled item's reason is visible in the menu and disabled items are dimmed", () => {
  setup();
  menu();
  const reason = within(screen.getByRole("menu")).getByText("Hazır düzenler yeniden adlandırılamaz ve silinemez.");
  expect(reason).not.toHaveClass("visually-hidden");
  expect(reason.closest("li")).toHaveAttribute("role", "none");
  expect(within(screen.getByRole("menu")).queryByText(/En çok/)).toBeNull();
  const css = readFileSync("ui/shell/shell.css", "utf8");
  expect(css).toMatch(/\.layout-menu \[aria-disabled="true"\]\s*\{[^}]*opacity:\s*0\.6;[^}]*cursor:\s*default;/);
});

test("reset on a user layout is aria-disabled with a visible reason, reachable by arrows and does nothing", () => {
  setup();
  saveAsNamed("Sabah");
  menu();
  const reset = item("Varsayılana döndür");
  expect(reset).toBeEnabled();
  expect(reset).toHaveAttribute("aria-disabled", "true");
  expect(reset).toHaveAccessibleDescription("Yalnızca hazır düzenler varsayılana döndürülür.");
  expect(within(screen.getByRole("menu")).getByText("Yalnızca hazır düzenler varsayılana döndürülür.")).not.toHaveClass("visually-hidden");
  fireEvent.keyDown(screen.getByRole("menu"), { key: "End" });
  expect(reset).toHaveFocus();
  fireEvent.click(reset);
  expect(screen.getByRole("menu")).toBeInTheDocument();
});

describe("actions menu keyboard", () => {
  test("opening from the keyboard focuses the first item and wires aria attributes", () => {
    setup();
    expect(trigger()).toHaveAttribute("aria-expanded", "false");
    trigger().focus();
    fireEvent.keyDown(trigger(), { key: "ArrowDown" });
    const m = screen.getByRole("menu");
    expect(trigger()).toHaveAttribute("aria-expanded", "true");
    expect(trigger()).toHaveAttribute("aria-controls", m.id);
    expect(item("Farklı kaydet…")).toHaveFocus();
  });

  test("arrows, Home and End rove over every item; disabled ones say why", () => {
    setup();
    menu();
    expect(item("Farklı kaydet…")).toHaveFocus();
    // Hazır düzende "Yeniden adlandır…" ve "Sil…" devre dışıdır ama odaklanır (menü deseni).
    fireEvent.keyDown(screen.getByRole("menu"), { key: "ArrowDown" });
    expect(item("Yeniden adlandır…")).toHaveFocus();
    fireEvent.keyDown(screen.getByRole("menu"), { key: "ArrowDown" });
    expect(item("Kopyala")).toHaveFocus();
    fireEvent.keyDown(screen.getByRole("menu"), { key: "End" });
    expect(item("Varsayılana döndür")).toHaveFocus();
    fireEvent.keyDown(screen.getByRole("menu"), { key: "ArrowDown" });
    expect(item("Farklı kaydet…")).toHaveFocus();
    fireEvent.keyDown(screen.getByRole("menu"), { key: "ArrowUp" });
    expect(item("Varsayılana döndür")).toHaveFocus();
    fireEvent.keyDown(screen.getByRole("menu"), { key: "Home" });
    expect(item("Farklı kaydet…")).toHaveFocus();
  });

  test("Esc closes the menu, returns focus to the trigger and keeps the chart pin", () => {
    setup();
    fireEvent.click(screen.getByRole("button", { name: "pin" }));
    expect(screen.getByTestId("pinned")).toHaveTextContent("pinned");
    menu();
    fireEvent.keyDown(item("Farklı kaydet…"), { key: "Escape" });
    expect(screen.queryByRole("menu")).toBeNull();
    expect(trigger()).toHaveFocus();
    expect(screen.getByTestId("pinned")).toHaveTextContent("pinned");
  });

  test("Tab and an outside click close the menu", () => {
    setup();
    menu();
    fireEvent.keyDown(item("Farklı kaydet…"), { key: "Tab" });
    expect(screen.queryByRole("menu")).toBeNull();
    menu();
    fireEvent.pointerDown(screen.getByRole("button", { name: "outside" }));
    expect(screen.queryByRole("menu")).toBeNull();
  });
});

describe("focus return to the trigger", () => {
  test("after saving and after cancelling the name form", () => {
    setup();
    saveAsNamed("Sabah");
    expect(trigger()).toHaveFocus();
    menu();
    fireEvent.click(item("Yeniden adlandır…"));
    fireEvent.click(screen.getByRole("button", { name: "Vazgeç" }));
    expect(trigger()).toHaveFocus();
  });

  test("Esc in the name form cancels without clearing the pin", () => {
    setup();
    fireEvent.click(screen.getByRole("button", { name: "pin" }));
    menu();
    fireEvent.click(item("Farklı kaydet…"));
    fireEvent.keyDown(screen.getByRole("textbox", { name: "Düzen adı" }), { key: "Escape" });
    expect(screen.queryByRole("textbox", { name: "Düzen adı" })).toBeNull();
    expect(trigger()).toHaveFocus();
    expect(screen.getByTestId("pinned")).toHaveTextContent("pinned");
    expect(options()).toHaveLength(3);
  });

  test("after confirming or cancelling delete, and Esc cancels the confirmation", () => {
    setup();
    fireEvent.click(screen.getByRole("button", { name: "pin" }));
    saveAsNamed("Sabah");
    menu();
    fireEvent.click(item("Sil…"));
    fireEvent.click(screen.getByRole("button", { name: "Vazgeç" }));
    expect(trigger()).toHaveFocus();
    menu();
    fireEvent.click(item("Sil…"));
    fireEvent.keyDown(screen.getByRole("button", { name: "Vazgeç" }), { key: "Escape" });
    expect(screen.queryByRole("group", { name: "Sil…" })).toBeNull();
    expect(trigger()).toHaveFocus();
    expect(screen.getByTestId("pinned")).toHaveTextContent("pinned");
    expect(picker()).toHaveDisplayValue("Sabah");
    menu();
    fireEvent.click(item("Sil…"));
    fireEvent.click(screen.getByRole("button", { name: "“Sabah” düzenini sil" }));
    expect(trigger()).toHaveFocus();
  });

  test("after duplicate and reset", () => {
    setup();
    menu();
    fireEvent.click(item("Kopyala"));
    expect(trigger()).toHaveFocus();
    menu();
    fireEvent.click(item("Varsayılana döndür"));
    expect(trigger()).toHaveFocus();
  });
});

describe("inline name validation", () => {
  test("empty names show a linked error and disable Save; Enter shows it too", () => {
    setup();
    menu();
    fireEvent.click(item("Farklı kaydet…"));
    const input = screen.getByRole("textbox", { name: "Düzen adı" });
    expect(input).not.toHaveAttribute("aria-invalid", "true");
    fireEvent.change(input, { target: { value: "   " } });
    expect(screen.getByRole("button", { name: "Kaydet" })).toBeDisabled();
    expect(input).toHaveAttribute("aria-invalid", "true");
    expect(input).toHaveAccessibleDescription("Bir ad yazın.");
    fireEvent.submit(input.closest("form")!);
    expect(screen.getByText("Bir ad yazın.").closest("[aria-live]")).toBeInTheDocument();
    expect(options()).toHaveLength(3);
  });

  test("names used by another layout are rejected case-insensitively", () => {
    setup();
    menu();
    fireEvent.click(item("Farklı kaydet…"));
    const input = screen.getByRole("textbox", { name: "Düzen adı" });
    fireEvent.change(input, { target: { value: " araştırma " } });
    expect(screen.getByRole("button", { name: "Kaydet" })).toBeDisabled();
    expect(input).toHaveAccessibleDescription("Bu ad başka bir düzende kullanılıyor.");
    fireEvent.submit(input.closest("form")!);
    expect(options()).toHaveLength(3);
  });

  test("renaming may keep the layout's own name", () => {
    setup();
    saveAsNamed("Sabah");
    menu();
    fireEvent.click(item("Yeniden adlandır…"));
    const input = screen.getByRole("textbox", { name: "Düzen adı" });
    fireEvent.change(input, { target: { value: "SABAH" } });
    expect(screen.getByRole("button", { name: "Kaydet" })).toBeEnabled();
    expect(input).not.toHaveAttribute("aria-invalid", "true");
    fireEvent.change(input, { target: { value: "Transit" } });
    expect(screen.getByRole("button", { name: "Kaydet" })).toBeDisabled();
  });

  test("duplicate and save-as default names are unique", () => {
    setup();
    menu();
    fireEvent.click(item("Kopyala"));
    menu();
    fireEvent.click(item("Kopyala"));
    expect(options()).toEqual(["Görüşme", "Görüşme kopyası 2", "Görüşme kopyası", "Araştırma", "Transit"]);
    menu();
    fireEvent.click(item("Farklı kaydet…"));
    expect(screen.getByRole("textbox", { name: "Düzen adı" })).toHaveValue("Görüşme kopyası 3");
  });
});

describe("form target is pinned to the layout it was opened for", () => {
  const twoUserLayouts = () => {
    setup();
    saveAsNamed("A");
    saveAsNamed("B");
    expect(options()).toEqual(["Görüşme", "Araştırma", "Transit", "A", "B"]);
  };
  const switchAway = () => fireEvent.keyDown(window, { key: "4", code: "Digit4", metaKey: true });

  test("rename: switching layouts while the form is open never renames another layout", () => {
    twoUserLayouts();
    menu();
    fireEvent.click(item("Yeniden adlandır…"));
    switchAway();
    expect(picker()).toHaveDisplayValue("A");
    const input = screen.queryByRole("textbox", { name: "Düzen adı" });
    if (input) {
      fireEvent.change(input, { target: { value: "Yeni" } });
      fireEvent.click(screen.getByRole("button", { name: "Kaydet" }));
    }
    expect(options()).toEqual(["Görüşme", "Araştırma", "Transit", "A", "B"]);
  });

  test("delete: switching layouts while the confirmation is open never deletes another layout", () => {
    twoUserLayouts();
    menu();
    fireEvent.click(item("Sil…"));
    switchAway();
    for (const b of screen.queryAllByRole("button", { name: /düzenini sil/ })) fireEvent.click(b);
    expect(options()).toEqual(["Görüşme", "Araştırma", "Transit", "A", "B"]);
  });

  test("the open form closes when the layout select changes", () => {
    twoUserLayouts();
    menu();
    fireEvent.click(item("Yeniden adlandır…"));
    fireEvent.change(picker(), { target: { value: BUILTIN_LAYOUT_IDS.transit } });
    expect(screen.queryByRole("textbox", { name: "Düzen adı" })).not.toBeInTheDocument();
  });
});
