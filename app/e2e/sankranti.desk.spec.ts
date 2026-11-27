// Sankranti akışı (Plan 26). Ayrı dosyadır: `desk.spec.ts` eşzamanlı planlarla paylaşılır; "e2e"
// projesi adı `desk.spec.ts` ile biten her dosyayı koşar.
import { expect, test } from "@playwright/test";
import { FIXTURE, ipcCalls, mockIpc } from "./ipc";

type SankrantiCall = { year: number | null; place: { lat: number; lon: number; tz: unknown }; settings: { schema_version: number } };

test("sankranti: a palette panel that lists the year's twelve ingresses in the place's time and moves the transit date", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const calls = async () => (await ipcCalls(page, "sankranti_list")) as SankrantiCall[];
  // Liste panel açılmadan istenmez (ileri modüller spec'i K1).
  expect(await calls()).toHaveLength(0);

  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("sankranti");
  await palette.getByRole("option", { name: /Sankranti panelini aç\/kapat/ }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  const panel = page.getByRole("region", { name: "Sankranti" });
  await expect(panel).toBeFocused();
  await expect(panel).toBeInViewport();

  // Bulunulan yer seçilmemiştir: liste istenmez, panel nedenini söyler.
  await expect(panel.getByRole("note")).toContainText("Bulunulan yer seçilmedi");
  expect(await calls()).toHaveLength(0);
  await panel.getByRole("combobox", { name: "Yer" }).selectOption("chart");

  const list = FIXTURE.transit_tools.sankranti.list;
  const table = panel.getByRole("table", { name: `Sankranti listesi, ${list.year}: 12 giriş` });
  await expect(table.locator("tbody tr")).toHaveCount(12);
  const sent = await calls();
  expect(sent).toHaveLength(1);
  // Yıl boştur ("bu yıl" Rust'ta okunur); yer haritanın doğum yeridir.
  expect(sent[0].year).toBeNull();
  expect(sent[0].place).toMatchObject({ lat: 28.6139, lon: 77.209, tz: { Iana: "Asia/Kolkata" } });
  expect(sent[0].settings.schema_version).toBe(2);
  // Yılın ilk girişi Oğlak'tır; saat yerin dilimindedir (+05:30).
  await expect(table.locator("tbody tr").first().locator("th, td")).toHaveText(["2026-01-14 15:07", "Oğl", "Krishna 11 · Ekadashi", "Budhavara · Çarşamba", "Anuradha · pada 3"]);
  await expect(panel.getByRole("spinbutton", { name: "Yıl" })).toHaveValue("2026");

  // Başka bir yıl: istek yılı taşır.
  await panel.getByRole("spinbutton", { name: "Yıl" }).fill("2031");
  await panel.getByRole("button", { name: "Yılı göster" }).click();
  await expect(panel.getByRole("table", { name: "Sankranti listesi, 2031: 12 giriş" })).toBeVisible();
  expect((await calls()).at(-1)!.year).toBe(2031);

  // Satırdaki an transit tarihini o ana götürür; liste ve odak yerinde durur.
  const before = (await ipcCalls(page, "transits_at")).length;
  const first = panel.getByRole("table").locator("tbody tr").first().getByRole("button");
  await first.focus();
  await page.keyboard.press("Enter");
  await expect.poll(async () => (await ipcCalls(page, "transits_at")).length).toBe(before + 1);
  const moved = (await ipcCalls(page, "transits_at")).at(-1) as { at: { Jd: number } };
  expect(moved.at).toEqual({ Jd: list.rows[0].sankranti.jd_ut });
  await expect(panel.getByRole("table").locator("tbody tr")).toHaveCount(12);
  await expect(first).toBeFocused();
});
