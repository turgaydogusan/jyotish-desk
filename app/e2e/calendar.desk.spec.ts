// Transit takvimi akışları (yol haritası Plan 29). Ayrı dosyadır: `desk.spec.ts` paralel planlarla
// paylaşılır; "e2e" projesi adı `desk.spec.ts` ile biten her dosyayı koşar.
import { expect, test, type Page } from "@playwright/test";
import { FIXTURE, ipcCalls, mockIpc } from "./ipc";

type CalendarCall = { request: { month: { year: number; month: number } | null; place: unknown; filter: { kinds: string[]; moon: boolean }; natal: { jd_ut: number; time_known: boolean } | null } };
type Saved = { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: Record<string, unknown> }[] }[] } } };

const NOVEMBER = "Kasım 2026 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları";
const DECEMBER = "Aralık 2026 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları";

async function open(page: Page, options: Parameters<typeof mockIpc>[1] = { session: true }) {
  await mockIpc(page, options);
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
}

async function toggle(page: Page) {
  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("takvim");
  await palette.getByRole("option", { name: /Transit takvimi panelini aç\/kapat/ }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
}

const calls = async (page: Page) => (await ipcCalls(page, "month_calendar")) as CalendarCall[];
const savedPanels = async (page: Page) => {
  const layout = ((await ipcCalls(page, "save_view_state")).at(-1) as Saved | undefined)?.viewState.panel_layout;
  return layout?.layouts.find((l) => l.id === layout.active_layout_id)?.panels;
};

test("transit calendar: a palette panel; the place, the month grid, days by keyboard, the day detail and the day panchang", async ({ page }) => {
  await open(page);
  // Takvim panel açılmadan istenmez (ileri modüller spec'i K1).
  expect(await calls(page)).toHaveLength(0);
  await toggle(page);
  const panel = page.getByRole("region", { name: "Transit takvimi" });
  await expect(panel).toBeFocused();
  await expect(panel).toBeInViewport();
  // Bulunulan yer seçilmemiş: panel yer ister, ay hesaplanmaz.
  await expect(panel.getByRole("note")).toContainText("Bulunulan yer seçilmedi.");
  expect(await calls(page)).toHaveLength(0);

  // Bulunulan yer penceresi: açık haritanın doğum yeri. Klavyeyle açılır (WebKit tıklanan
  // düğmeye odak vermez): pencere kapanınca odak düğmeye döner.
  await panel.getByRole("button", { name: "Bulunulan yeri seç" }).focus();
  await page.keyboard.press("Enter");
  const dialog = page.getByRole("dialog", { name: "Bulunulan yer" });
  await dialog.getByRole("button", { name: /^Açık haritanın doğum yerini kullan/ }).click();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toHaveCount(0);

  // Ay ızgarası: bir istek; ay verilmez ("bugün" Rust'ta okunur), yerin adı ve doğum verisi gitmez.
  const grid = panel.getByRole("grid", { name: NOVEMBER });
  await expect(grid.locator("td[data-date]")).toHaveCount(30);
  const first = await calls(page);
  expect(first).toHaveLength(1);
  expect(first[0].request).toMatchObject({ month: null, place: FIXTURE.muhurta.place, filter: FIXTURE.calendar.request.filter, natal: null });
  await expect(grid.getByRole("columnheader")).toHaveText(["PztPazartesi", "SalSalı", "ÇarÇarşamba", "PerPerşembe", "CumCuma", "CmtCumartesi", "PazPazar"]);
  await expect(grid.locator("tbody tr")).toHaveCount(6);
  // Bugünün hücresi: gün doğumundaki tithi ve Ay nakshatrası, günün olayı saatiyle.
  const today = grid.locator('td[data-date="2026-11-25"]');
  await expect(today).toHaveClass(/is-today/);
  await expect(today.locator(".calendar-day-head")).toHaveText("25bugün");
  await expect(today.locator(".calendar-tithi")).toContainText("K1 Pratipada");
  await expect(today.locator(".calendar-nakshatra")).toHaveText("Rohini");
  await expect(today.locator("li")).toHaveText(["13:41 Me → VishakhaMerkür: nakshatra girişi · Vishakha"]);
  await expect(grid.locator('td[data-date="2026-11-09"] li')).toContainText(["12:32 Yeni ay"]);
  await expect(grid.locator('td[data-date="2026-11-16"] li')).toContainText(["19:43 Gü → Akr"]);
  await expect(panel.getByText(/^Yöntem: calendar\.month_v1\./)).toBeVisible();

  // Ayrıntı seçim yokken bugünündür.
  const detail = panel.locator("section.calendar-detail");
  await expect(detail).toHaveAttribute("data-detail", "2026-11-25");
  await expect(detail.getByText("Gün doğumu 06:55 · Krishna 1 · Pratipada · Rohini")).toBeVisible();

  // Klavye: ızgara tek sekme durağıdır; oklar günler arasında gezer, Enter günü seçer.
  await today.focus();
  await page.keyboard.press("ArrowLeft");
  await expect(grid.locator('td[data-date="2026-11-24"]')).toBeFocused();
  await page.keyboard.press("ArrowUp");
  await expect(grid.locator('td[data-date="2026-11-17"]')).toBeFocused();
  await page.keyboard.press("ArrowLeft");
  await page.keyboard.press("Enter");
  await expect(detail).toHaveAttribute("data-detail", "2026-11-16");
  await expect(detail.getByText("16 Kasım 2026 Pazartesi", { exact: true })).toBeVisible();
  await expect(detail.getByRole("table").locator("tbody tr").first().locator("th, td")).toHaveText(["19:43", "Güneş: burç girişi", "Akr"]);
  await expect(grid.locator('td[aria-selected="true"]')).toHaveAttribute("data-date", "2026-11-16");
  await expect(grid.locator('td[tabindex="0"]')).toHaveCount(1);

  // Page Down: sonraki ay (Rust'ın verdiği ay istenir) ve odak aynı günde durur.
  await page.keyboard.press("PageDown");
  const december = panel.getByRole("grid", { name: DECEMBER });
  await expect(december.locator("td[data-date]")).toHaveCount(31);
  await expect(december.locator('td[data-date="2026-12-16"]')).toBeFocused();
  expect((await calls(page)).at(-1)?.request.month).toEqual({ year: 2026, month: 12 });
  await expect(december.locator("td.is-today")).toHaveCount(0);
  await expect(panel.getByRole("combobox", { name: "Ay" })).toHaveValue("12");
  // Başka bir ayda seçim yoktur: ayrıntı gün ister.
  await expect(panel.getByText("Ayrıntısını görmek için bir gün seçin.")).toBeVisible();
  await page.keyboard.press("PageUp");
  await expect(grid.locator('td[data-date="2026-11-16"]')).toBeFocused();

  // Olayın saati transit tarihini o ana götürür; "Gün panchangını aç" paneli ekler.
  await detail.getByRole("button", { name: "19:43: transit tarihini bu ana götür" }).click();
  await detail.getByRole("button", { name: "Gün panchangını aç" }).click();
  const day = page.getByRole("region", { name: "Gün panchangı" });
  await expect(day).toBeVisible();
  const dayCalls = (await ipcCalls(page, "day_panchang")) as { at: unknown }[];
  expect(dayCalls.at(-1)?.at).toEqual({ Jd: FIXTURE.calendar.month.days[15].panchang_jd_ut });
  await expect.poll(async () => (await savedPanels(page))?.map((p) => p.kind).filter((k) => k === "calendar" || k === "muhurta_day")).toEqual(["calendar", "muhurta_day"]);

  // Aynı komut paneli kaldırır ve odak gövdeye düşmez.
  await toggle(page);
  await expect(panel).toHaveCount(0);
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
});

test("transit calendar filter: natal arrivals and the Moon's events are asked only when chosen and are saved with the layout", async ({ page }) => {
  await open(page);
  await toggle(page);
  const panel = page.getByRole("region", { name: "Transit takvimi" });
  // Yer: haritanın doğum yeri (panelin ayarı).
  await panel.getByRole("combobox", { name: "Yer" }).selectOption("chart");
  const grid = panel.getByRole("grid", { name: NOVEMBER });
  await expect(grid.locator('td[data-date="2026-11-24"] li')).toHaveCount(1);
  expect((await calls(page))[0].request.natal).toBeNull();

  // Süzgeç kapalı bir bölümdedir; özet seçili tür sayısını söyler.
  await panel.locator("summary", { hasText: "Olay türleri: 6 / 7 seçili" }).click();
  await panel.getByRole("checkbox", { name: "Doğum boylamına geliş", exact: true }).check();
  await expect(panel.locator("summary", { hasText: "Olay türleri: 7 / 7 seçili" })).toBeVisible();
  await panel.getByRole("checkbox", { name: /^Ay'ın olayları da/ }).check();
  // Doğum verisi ancak tür seçilince gider.
  await expect.poll(async () => (await calls(page)).at(-1)?.request.filter).toEqual({ kinds: FIXTURE.calendar.ids.kinds, moon: true });
  expect((await calls(page)).at(-1)?.request.natal).toMatchObject({ jd_ut: 2451545, time_known: true });
  // Hücre en çok üç olay yazar ve gerisini sayar; ayrıntı hepsini listeler.
  const full = grid.locator('td[data-date="2026-11-24"]');
  await expect(full.locator("li")).toHaveCount(4);
  await expect(full.locator("li.calendar-more")).toHaveText("+2 olay");
  await full.click();
  const detail = panel.locator("section.calendar-detail");
  await expect(detail.getByRole("table").locator("tbody tr")).toHaveCount(5);
  await expect(detail.getByRole("table").locator('tbody tr[data-event="TransitToNatal"] td').first()).toHaveText("Güneş, doğum Venüs boylamında");
  await expect.poll(async () => (await savedPanels(page))?.find((p) => p.kind === "calendar")?.settings).toEqual({ place: "chart", kinds: FIXTURE.calendar.ids.kinds, moon: true });

  // İngilizcede hafta Pazar başlar: aynı ay beş satırdır.
  await page.evaluate(() => localStorage.setItem("jyotish-desk.language.v1", "en"));
  await page.reload();
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Command palette" });
  await palette.getByRole("combobox").fill("calendar");
  await palette.getByRole("option", { name: /Show or hide the Transit calendar panel/ }).click();
  const english = page.getByRole("region", { name: "Transit calendar" });
  await english.getByRole("combobox", { name: "Place" }).selectOption("chart");
  const englishGrid = english.getByRole("grid", { name: /^Calendar for November 2026/ });
  await expect(englishGrid.locator("tbody tr")).toHaveCount(5);
  await expect(englishGrid.getByRole("columnheader").first()).toHaveText("SunSunday");
});

test("transit calendar with an unknown birth time: natal arrivals are marked approximate", async ({ page }) => {
  await open(page, { session: true, unknownTime: true });
  await toggle(page);
  const panel = page.getByRole("region", { name: "Transit takvimi" });
  await panel.getByRole("combobox", { name: "Yer" }).selectOption("chart");
  await expect(panel.getByRole("grid", { name: NOVEMBER })).toBeVisible();
  await expect(panel.getByText(/^Doğum saati bilinmiyor/)).toHaveCount(0);
  await panel.locator("summary", { hasText: /^Olay türleri/ }).click();
  await panel.getByRole("checkbox", { name: "Doğum boylamına geliş", exact: true }).check();
  await expect(panel.getByText("Doğum saati bilinmiyor: doğum boylamları öğlen 12:00 içindir; doğum Ay'ına gelişler yaklaşıktır.")).toBeVisible();
  await expect.poll(async () => (await calls(page)).at(-1)?.request.natal?.time_known).toBe(false);
});
