// Efemeris akışı (Plan 26). Ayrı dosyadır: `desk.spec.ts` eşzamanlı planlarla paylaşılır; "e2e"
// projesi adı `desk.spec.ts` ile biten her dosyayı koşar.
import { expect, test } from "@playwright/test";
import { FIXTURE, ipcCalls, mockIpc } from "./ipc";

type EphemerisCall = { month: { year: number; month: number } | null; place: { lat: number; lon: number; tz: unknown }; settings: { schema_version: number } };

test("ephemeris: a palette panel with a month of daily positions, month steps from Rust, marks in words and a day that moves the transit date", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const calls = async () => (await ipcCalls(page, "ephemeris_month")) as EphemerisCall[];
  // Tablo panel açılmadan istenmez (ileri modüller spec'i K1).
  expect(await calls()).toHaveLength(0);

  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("efemeris");
  await palette.getByRole("option", { name: /Efemeris panelini aç\/kapat/ }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  const panel = page.getByRole("region", { name: "Efemeris" });
  await expect(panel).toBeFocused();
  await expect(panel).toBeInViewport();

  // Bulunulan yer seçilmemiştir: tablo istenmez, panel nedenini söyler.
  await expect(panel.getByRole("note")).toContainText("Bulunulan yer seçilmedi");
  expect(await calls()).toHaveLength(0);
  await panel.getByRole("combobox", { name: "Yer" }).selectOption("chart");

  const month = FIXTURE.transit_tools.ephemeris.month;
  const table = panel.getByRole("table", { name: /^Kasım 2026: her günün başında/ });
  await expect(table.locator("tbody tr")).toHaveCount(month.days.length);
  const sent = await calls();
  expect(sent).toHaveLength(1);
  // Ay boştur ("bu ay" Rust'ta okunur); yer haritanın doğum yeridir.
  expect(sent[0].month).toBeNull();
  expect(sent[0].place).toMatchObject({ lat: 28.6139, lon: 77.209, tz: { Iana: "Asia/Kolkata" } });
  expect(sent[0].settings.schema_version).toBe(2);
  // On sütun: gün ve dokuz graha. 1 Kasım 2026 Pazar'dır.
  await expect(table.locator("thead th")).toHaveCount(10);
  await expect(table.locator('tr[data-date="2026-11-01"] th')).toHaveText("01 Paz");
  // 16 Kasım: Güneş o gün Akrep'e girer; işaret simge ve sözdür.
  const sun = table.locator('tr[data-date="2026-11-16"] td[data-graha="Sun"]');
  await expect(sun.locator('[data-mark="ingress"]')).toContainText("→ Akr");
  await expect(sun.locator('[data-mark="ingress"]')).toContainText("o gün Akr burcuna girer");
  await expect(table.locator('tr[data-date="2026-11-13"] td[data-graha="Mercury"] [data-mark="station"]')).toContainText("o gün düze döner");

  // Nakshatra kutusu yeni istek başlatmaz; hücrelere nakshatra ve pada eklenir.
  await expect(panel.locator(".ephemeris-nakshatra")).toHaveCount(0);
  await panel.getByRole("checkbox", { name: "Nakshatra" }).check();
  await expect(table.locator('tr[data-date="2026-11-01"] td[data-graha="Sun"] .ephemeris-nakshatra')).toHaveText("Swati 3");
  expect(await calls()).toHaveLength(1);

  // Sonraki ay: istek, Rust'ın döndürdüğü `next` ayını taşır; odak düğmede durur.
  const next = panel.getByRole("button", { name: "Sonraki ay" });
  await next.focus();
  await page.keyboard.press("Enter");
  await expect(panel.getByRole("table", { name: /^Aralık 2026:/ })).toBeVisible();
  expect((await calls()).at(-1)!.month).toEqual({ year: 2026, month: 12 });
  await expect(next).toBeFocused();

  // Gün, transit tarihini o günün başına götürür.
  const before = (await ipcCalls(page, "transits_at")).length;
  await panel.getByRole("table").locator('tr[data-date="2026-11-06"] th').getByRole("button").click();
  await expect.poll(async () => (await ipcCalls(page, "transits_at")).length).toBe(before + 1);
  const moved = (await ipcCalls(page, "transits_at")).at(-1) as { at: { Jd: number } };
  expect(moved.at).toEqual({ Jd: month.days[5].row.jd_ut });
});
