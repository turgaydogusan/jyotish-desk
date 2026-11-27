// Transit adımı ve oynatma akışları (Plan 26). Ayrı dosyadır: `desk.spec.ts` eşzamanlı planlarla
// paylaşılır; "e2e" projesi adı `desk.spec.ts` ile biten her dosyayı koşar.
import { expect, test, type Page } from "@playwright/test";
import { FIXTURE, ipcCalls, mockIpc } from "./ipc";

type StepCall = { request: { jd_ut: number; unit: string; count: number; tz: string } };

// Gösterilen anın yazımı tarayıcının dilimine bağlıdır: dilim sabitlenir.
test.use({ timezoneId: "Europe/Istanbul" });

/**
 * `transits_at` taklidini canlandırır: `{ Jd }` anının yanıtı o anı taşır (konumlar fikstürdeki
 * gibidir; arayüz hesap yapmaz). `mockIpc`'ten sonra çağrılır.
 */
async function liveTransits(page: Page) {
  await page.addInitScript(() => {
    type Invoke = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
    const internals = (window as unknown as { __TAURI_INTERNALS__: { invoke: Invoke } }).__TAURI_INTERNALS__;
    const inner = internals.invoke;
    internals.invoke = async (cmd, args = {}) => {
      const reply = await inner(cmd, args);
      const at = args.at;
      if (cmd === "transits_at" && typeof at === "object" && at !== null && "Jd" in at) {
        const jd = (at as { Jd: number }).Jd;
        const view = reply as { utc: string; snapshot: { jd_ut: number } };
        view.snapshot.jd_ut = jd;
        view.utc = new Date(Math.round((jd - 2440587.5) * 86_400_000)).toISOString();
      }
      return reply;
    };
  });
}

async function openTransitTable(page: Page) {
  await mockIpc(page, { session: true });
  await liveTransits(page);
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit3");
  const table = page.getByRole("region", { name: "Transit tablosu" });
  await expect(table.getByRole("table")).toBeVisible();
  return table;
}

const steps = async (page: Page) => (await ipcCalls(page, "transit_step")) as StepCall[];
const J0: number = FIXTURE.transit.snapshot.jd_ut;

test("transit steps: the step buttons move the moment by the chosen unit; play advances it until paused", async ({ page }) => {
  const table = await openTransitTable(page);
  const stepper = table.getByRole("group", { name: "Transit anını adımla" });
  const now = table.getByRole("button", { name: "Şimdi" });
  await expect(now).toHaveAttribute("aria-pressed", "true");
  expect(await steps(page)).toHaveLength(0);

  // Bir gün ileri: adım Rust'a gösterilen an, birim ve tarayıcının dilimiyle sorulur; varılan an
  // transit tarihi olur (fikstürün anı 2026-11-25 10:00 UT; İstanbul'da 13:00).
  const forward = stepper.getByRole("button", { name: "Bir adım ileri" });
  await forward.focus();
  await page.keyboard.press("Enter");
  await expect(table.getByRole("status")).toHaveText("Gösterilen an: 26 Kas 2026 13:00");
  expect((await steps(page))[0].request).toEqual({ jd_ut: J0, unit: "Day", count: 1, tz: "Europe/Istanbul" });
  await expect.poll(async () => ((await ipcCalls(page, "transits_at")) as { at: unknown }[]).at(-1)?.at).toEqual({ Jd: J0 + 1 });
  await expect(now).toHaveAttribute("aria-pressed", "false");
  // Odak düğmede durur: art arda adım klavyeyle atılır.
  await expect(forward).toBeFocused();
  await expect(table.getByLabel("Tarih")).toHaveValue("2026-11-26");

  // Bir hafta geri: çıkış anı bir önceki adımın vardığı andır.
  await stepper.getByRole("combobox", { name: "Adım birimi" }).selectOption("Week");
  await stepper.getByRole("button", { name: "Bir adım geri" }).click();
  await expect(table.getByRole("status")).toHaveText("Gösterilen an: 19 Kas 2026 13:00");
  expect((await steps(page))[1].request).toMatchObject({ jd_ut: J0 + 1, unit: "Week", count: -1 });

  // Oynat: an, duraklatılana dek aralıklarla ilerler.
  await stepper.getByRole("combobox", { name: "Adım birimi" }).selectOption("Day");
  await stepper.getByRole("combobox", { name: "Hız" }).selectOption("fast");
  await stepper.getByRole("button", { name: "Oynat" }).click();
  const pause = stepper.getByRole("button", { name: "Duraklat" });
  await expect(pause).toBeVisible();
  await expect.poll(async () => (await steps(page)).length, { timeout: 10_000 }).toBeGreaterThanOrEqual(5);
  await pause.click();
  await expect(stepper.getByRole("button", { name: "Oynat" })).toBeVisible();
  const played = await steps(page);
  // Her oynatma adımı bir öncekinin vardığı andan çıkar: adım atlanmaz, yinelenmez.
  for (let i = 3; i < played.length; i++) expect(played[i].request).toMatchObject({ jd_ut: played[i - 1].request.jd_ut + 1, unit: "Day", count: 1 });
  // Duraklatınca durur.
  await page.waitForTimeout(1_200);
  expect(await steps(page)).toHaveLength(played.length);
  // Tablo oynatma boyunca yerindedir.
  await expect(table.getByRole("table").locator("tbody tr")).toHaveCount(9);
});

test.describe("reduced motion", () => {
  test.use({ reducedMotion: "reduce" });

  test("transit steps: with reduced motion there is no play button, only steps", async ({ page }) => {
    const table = await openTransitTable(page);
    const stepper = table.getByRole("group", { name: "Transit anını adımla" });
    await expect(stepper.getByRole("note")).toContainText("oynatma sunulmuyor");
    await expect(stepper.getByRole("button", { name: "Oynat" })).toHaveCount(0);
    await expect(stepper.getByRole("combobox", { name: "Hız" })).toHaveCount(0);
    await stepper.getByRole("button", { name: "Bir adım ileri" }).click();
    await expect(table.getByRole("status")).toHaveText("Gösterilen an: 26 Kas 2026 13:00");
    expect(await steps(page)).toHaveLength(1);
  });
});
