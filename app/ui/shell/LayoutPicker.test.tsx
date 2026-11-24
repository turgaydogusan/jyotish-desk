import { fireEvent, render, screen } from "@testing-library/react";
import { BUILTIN_LAYOUT_IDS } from "../layout/model";
import { memoryPersistence } from "../layout/persistence";
import { LayoutProvider, useLayoutStore } from "../layout/store";
import { LayoutPicker } from "./LayoutPicker";

function ActiveId() {
  return <output data-testid="active">{useLayoutStore().active.id}</output>;
}

function setup() {
  render(
    <LayoutProvider persistence={memoryPersistence()}>
      <LayoutPicker />
      <ActiveId />
    </LayoutProvider>,
  );
}

const picker = () => screen.getByRole("combobox", { name: "Düzen" });
const menu = () => fireEvent.click(screen.getByRole("button", { name: "Düzen işlemleri" }));
const item = (name: string) => screen.getByRole("menuitem", { name });

test("lists builtin layouts by translated name and selects on change", () => {
  setup();
  expect([...picker().querySelectorAll("option")].map((o) => o.textContent)).toEqual(["Görüşme", "Araştırma", "Transit"]);
  fireEvent.change(picker(), { target: { value: BUILTIN_LAYOUT_IDS.transit } });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
});

test("builtin layouts can be reset but not renamed or deleted", () => {
  setup();
  menu();
  expect(item("Yeniden adlandır…")).toBeDisabled();
  expect(item("Sil…")).toBeDisabled();
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
