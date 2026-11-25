import { expect, test } from "@playwright/test";
import { ipcCalls, mockIpc } from "./ipc";

test("birth entry: empty state, atlas place by keyboard, preview, save opens the chart", async ({ page }) => {
  await mockIpc(page, { session: false });
  await page.goto("/");
  await page.getByRole("region", { name: "Açık harita yok" }).getByRole("button", { name: "Yeni harita" }).click();
  const form = page.getByRole("dialog", { name: "Yeni harita" });
  await expect(form.getByLabel("Danışan adı", { exact: true })).toBeFocused();
  await form.getByLabel("Danışan adı", { exact: true }).fill("Örnek Danışan");
  await form.getByLabel("Tarih", { exact: true }).fill("2000-01-01");
  await form.getByLabel("Saat", { exact: true }).fill("17:30");
  const place = form.getByRole("combobox", { name: "Doğum yeri" });
  await place.fill("new delhi");
  await expect(form.getByRole("listbox", { name: "Bulunan yerler" }).getByRole("option").first()).toContainText("New Delhi");
  await place.press("Enter");
  await expect(form.getByLabel("Bölge adı", { exact: true })).toHaveValue("Asia/Kolkata");
  await expect(form.getByRole("region", { name: "Önizleme" })).toContainText("UTC 2000-01-01 12:00:00 · IST +05:30");
  await form.getByRole("button", { name: "Haritayı kaydet" }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await expect(page.getByRole("group", { name: "D1 haritası" })).toBeVisible();
  const created = (await ipcCalls(page, "create_client")) as { input: { chart: { birth: { tz: unknown } } } }[];
  expect(created[0].input.chart.birth.tz).toEqual({ Iana: "Asia/Kolkata" });
});

test("layout switch with Cmd+2 and a keyboard panel move that is saved with the chart", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit2");
  await expect(page.getByRole("combobox", { name: "Düzen" }).locator("option:checked")).toHaveText("Araştırma");
  await expect(page.getByRole("region", { name: "Graha tablosu" })).toBeVisible();
  const table = page.getByRole("region", { name: "Graha tablosu" });
  const before = Number(await table.getAttribute("data-y"));
  await table.focus();
  await page.keyboard.press("Alt+ArrowDown");
  await expect(table).toHaveAttribute("data-y", String(before + 1));
  await expect.poll(async () => (await ipcCalls(page, "save_view_state")).length).toBeGreaterThan(0);
  const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { chartId: number };
  expect(saved.chartId).toBe(1);
});

test("hovering a graha row highlights it in the chart; the style selector redraws", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit2");
  const row = page.getByRole("region", { name: "Graha tablosu" }).locator('tr[data-graha="Saturn"]');
  await row.hover();
  await expect(page.locator('svg [data-graha="Saturn"]').first()).toHaveClass(/chart-planet--highlight/);
  const chart = page.getByRole("region", { name: "Harita D1", exact: true });
  await chart.getByRole("combobox", { name: "Biçim" }).selectOption("south");
  await expect(chart.locator("svg.chart")).toHaveAttribute("data-style", "south");
});

test("dasha tree by keyboard loads level 4 on demand", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit2");
  const tree = page.getByRole("tree", { name: "Dasa ağacı" });
  await expect(tree).toBeVisible();
  await tree.getByRole("treeitem").first().focus();
  // Sağ ok: kapalıysa açar, açıksa ilk alt döneme iner (1 → 2 → 3. seviye).
  for (const key of ["ArrowRight", "ArrowRight", "ArrowRight", "ArrowRight", "ArrowRight"]) await page.keyboard.press(key);
  await expect(page.locator(":focus")).toHaveAttribute("aria-level", "3");
  await expect(tree.getByRole("treeitem", { name: /^Rahu–Rahu–Rahu–Rahu, / })).toBeVisible();
  await page.keyboard.press("ArrowRight");
  await expect(page.locator(":focus")).toHaveAttribute("aria-level", "4");
  expect(await ipcCalls(page, "dasha_children")).toHaveLength(1);
});

test("spec §8: Cmd+K, a name and Enter open that client's chart with no form or confirmation", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+KeyK");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await expect(palette.getByRole("combobox")).toBeFocused();
  await page.keyboard.type("ayşe");
  await expect(palette.getByRole("option").first()).toContainText("Ayşe Yılmaz");
  await page.keyboard.press("Enter");
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(page.getByRole("heading", { level: 1, name: "Ayşe Yılmaz" })).toBeVisible();
  await expect(page.getByRole("group", { name: "D1 haritası" })).toBeVisible();
  expect(await ipcCalls(page, "open_client")).toEqual([{ clientId: 2, chartId: null }]);
});

test("the sidebar opens a recent client, a new note is saved as it is typed, and Cmd+\\ from the note hides the sidebar", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  const sidebar = page.getByRole("complementary", { name: "Danışanlar" });
  await sidebar.getByRole("button", { name: "Ayşe Yılmaz" }).click();
  await expect(page.getByRole("heading", { level: 1, name: "Ayşe Yılmaz" })).toBeVisible();
  await expect(sidebar.getByRole("button", { name: "Ayşe Yılmaz" })).toHaveAttribute("aria-current", "true");
  const notes = page.getByRole("region", { name: "Notlar" });
  await notes.getByRole("button", { name: "Yeni not" }).click();
  const box = notes.getByRole("textbox");
  await expect(box).toBeFocused();
  await page.keyboard.type("Satürn dönüşü konuşuldu");
  await page.keyboard.press("Tab");
  await expect.poll(async () => ((await ipcCalls(page, "update_note")).at(-1) as { body?: string } | undefined)?.body).toBe("Satürn dönüşü konuşuldu");
  // ⌘ kısayolları not kutusunun içinden de çalışır (Plan 10 incelemesi MF1).
  await box.focus();
  await page.keyboard.press("Meta+Backslash");
  await expect(sidebar).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Danışanlar" })).toHaveAttribute("aria-expanded", "false");
});
