# Plan 04 — Arayüz Temeli: Tema, Pano Izgarası, Düzenler ve Harita Çizimi

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Demo `App.tsx`'i; tema belirteçleri, 12 sütunlu pano ızgarası, kaydedilebilir düzenler, üç stilde SVG harita, graha tablosu, ortak vurgu ve üç dilli i18n içeren arayüz temeliyle değiştirmek. Yalnızca bugün var olan üç komut kullanılır, yeni Rust kodu yazılmaz.

**Architecture:** Arayüz `app/ui/` altında katmanlara ayrılır:
- **Saf çekirdek (React yok):**
  - `theme/theme.ts`: tema tercihi.
  - `layout/model.ts`: düzen modeli ve toleranslı ayrıştırma.
  - `layout/grid.ts`: ızgara motoru (çakışmasız taşıma ve boyutlama, yeniden akış).
  - `layout/reducer.ts`: düzen eylemleri.
  - `chart/geometry.ts`: harita geometrisi.
  - `chart/adapter.ts`: `PlanetRow` → `ChartData`.
- **Durum (React bağlamları):**
  - `LayoutProvider`: düzenler; 500 ms debounce ile `LayoutPersistence`'a kaydeder.
  - `HighlightProvider`: oturumluk ortak vurgu.
  - `SkyProvider`: şimdiki gökyüzü; tek `planetLongitudes` çağrısı.
- **Görünüm:**
  - `chart/ChartSvg`: saf SVG harita.
  - `panels/*`: panel kaydı ve paneller.
  - `dashboard/*`: CSS grid pano; sürükleme, boyutlama, klavye.
  - `shell/*`: üst çubuk, düzen seçici, tema ve dil menüsü, baskı denemesi.

Arayüz hesap yapmaz. Tek istisna gösterim eşlemesidir: burç = `floor(lon / 30)` ve burç içi derece.

**Tech Stack:** React 19, Vite 8, TypeScript 7, Vitest 5 (jsdom 31, `@testing-library/react` 16, `@testing-library/jest-dom`), pnpm 11.8.0. Yeni bağımlılık: `i18next` 27 ve `react-i18next` 18 (Task 6'da `pnpm add` ile, yürütme anındaki son kararlı sürüm).

**Spec:**
- `docs/superpowers/specs/2026-11-24-ui-design.md`: tamamı (bağlayıcı).
- `docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md`: §4 (arayüz hesap yapmaz, `ViewState`, 500 ms debounce), §7 (bozuk görünüm durumu açılışı engellemez, hatalar `{code, params}`), §11 (M5 arayüz iskeleti).

## Global Constraints

- Lisans: AGPL-3.0-or-later (`app/package.json` → `"license": "AGPL-3.0-or-later"`).
- Kod adı `jyotish-desk`. "Kala", "Kāla", "Kaala", "Cala" ve türevleri hiçbir yerde kullanılmaz.
- `app/ui/bindings.ts` üretilmiş dosyadır, elle düzenlenmez. Tüm backend çağrıları `app/ui/api.ts` üzerinden yapılır.
- Bugün yalnızca üç komut vardır: `engineStatus`, `planetLongitudes(jdUt)` ve `printWindow`. Lagna, ev, varga, dasha, transit ve store komutları M2–M4'te gelir. Bu plan yeni Rust kodu yazmaz.
- Arayüz astrolojik hesap yapmaz (v1 spec §4). İzin verilen tek gösterim eşlemesi: `sign = floor(lon / 30)`, `degInSign = lon − 30·sign` ve evin burcu `(lagna + ev − 1) mod 12` (bütün burç). Lagna yokken 1. ev Koç kabul edilir ve bu açıkça yazılır.
- Renkler yalnızca spec §2.1'deki belirteçlerle verilir (`var(--…)`). Bileşenlerde ve CSS'te hex yazılmaz; hex yalnızca `app/ui/theme/tokens.css`'te bulunur. Bunu `theme/no-hex.test.ts` denetler.
- İki yazı ağırlığı vardır: 400 ve 500. Boyutlar 12, 13, 15 ve 18 px'tir. Rakamlarda `tabular-nums` kullanılır.
- Renk tek başına anlam taşımaz:
  - Retro `℞`, yanma `c`, vargottama altı çizili.
  - Vurgu, zemine ek olarak yazı ağırlığı 500 ile gösterilir.
  - Lagna, Kuzey Hint'te konumla, Güney Hint'te ve dairede metin işaretiyle de belirtilir.
- Kontrast WCAG AA'dır: metin belirteçleri her yüzeyde en az 4.5:1 (testle denetlenir).
- Odak halkası 2 px `--ink`. Tüm işlemler klavyeyle yapılabilir.
- `i18next` ve `react-i18next` dışında yeni bağımlılık eklenmez. Sürükleme ve ızgara elle yazılır.
- CI (`core-macos`) `pnpm --dir app install --frozen-lockfile`, `pnpm --dir app test` ve `pnpm --dir app build` çalıştırır. Her görev sonunda ikisi de yeşildir. Bağımlılık eklenince `pnpm-lock.yaml` da commit edilir.
- Kullanıcıya görünen her metin i18n'den gelir (tr varsayılan, en, fr). Testler Türkçe metinle doğrular.
- Commit mesajları İngilizcedir ve boş bir satırdan sonra `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` ile biter (aşağıdaki komutlarda ikinci `-m`).

---

## Dosya Yapısı (bu plan)

```
app/
  package.json                      # + i18next, react-i18next (Task 6)
  pnpm-lock.yaml                    # Task 6
  vite.config.ts                    # test.css: ?raw CSS içe aktarımları (Task 1)
  ui/
    main.tsx                        # tokens.css ve i18n içe aktarımı
    App.tsx                         # sağlayıcılar + Shell (Task 8; Task 6'da yalnızca import)
    App.test.tsx                    # Task 6'da küçük düzeltme, Task 8'de yeniden yazılır
    setupTests.ts                   # i18n başlatma, testler arası sıfırlama (Task 6)
    time.ts, time.test.ts           # unixMsToJdUt (App.tsx'ten taşındı, Task 6)
    theme/
      tokens.css                    # renk, yazı, odak belirteçleri; baskıda açık tema
      theme.ts                      # ThemePref, resolveTheme, applyTheme, load/save
      theme.test.ts, tokens.test.ts, no-hex.test.ts
    layout/
      model.ts                      # Panel, Layout, LayoutState, hazır düzenler, parse/serialize
      grid.ts                       # clamp, move, resize, settle, compact, reflow, kırılımlar
      reducer.ts                    # LayoutAction, layoutReducer, newId, normalizeState
      persistence.ts                # LayoutPersistence, localStorage (geçici), bellek
      store.tsx                     # LayoutProvider, useLayoutStore, debounce, Cmd+1…9
      model.test.ts, grid.test.ts, reducer.test.ts, store.test.tsx
    chart/
      types.ts                      # GrahaId, ChartData, ChartStyle, ChartLabels
      adapter.ts                    # signAndDegree, chartDataFromPlanetRows
      geometry.ts                   # northCells, southCells, circleCells, stackPositions
      ChartSvg.tsx, chart.css
      adapter.test.ts, geometry.test.ts, ChartSvg.test.tsx
    highlight/
      target.ts                     # HighlightTarget, sameTarget (Task 5)
      store.tsx                     # HighlightProvider, useHighlight, Esc (Task 6)
      store.test.tsx
    i18n/
      tr.ts, en.ts, fr.ts           # kaynaklar; Messages tipi tr'den türetilir
      index.ts                      # i18n örneği, setLanguage, loadLanguage
      i18n.test.ts
    panels/
      types.ts                      # PanelProps, PanelDefinition
      registry.ts                   # PANEL_REGISTRY, panelDefinition
      sky.tsx                       # SkyProvider, useSky
      states.tsx                    # usePanelLoadPhase, PanelSkeleton, PanelError, PanelStatusContext
      labels.ts                     # i18n → ChartLabels
      format.ts                     # formatDegree
      ChartPanel.tsx, GrahaTablePanel.tsx, UnavailablePanel.tsx, panels.css
      format.test.ts, states.test.tsx, panels.test.tsx
    dashboard/
      metrics.ts                    # gridMetrics, pxToGridDelta, arrowDelta
      PanelFrame.tsx                # başlık, satır içi gösterge, gövde, boyut tutamacı
      Dashboard.tsx, dashboard.css
      metrics.test.ts, Dashboard.test.tsx
    shell/
      Shell.tsx                     # tema uygulama, Cmd+Shift+N, ölçüm
      TopBar.tsx                    # başlık, düzen seçici, tema, dil, baskı denemesi
      LayoutPicker.tsx              # seçici + işlemler menüsü, satır içi ad ve silme onayı
      PrintView.tsx                 # PrintSpike + printWindow
      useElementWidth.ts, shell.css
      LayoutPicker.test.tsx
```

## Bu planda verilen kararlar

Spec'in açık bıraktığı noktalar aşağıdaki gibi kapatıldı. Her biri ilgili dosyada yorum olarak da yazılıdır.

- **Kalıcılık geçicidir.** `ViewState.panel_layout`'u yazan store komutu M4'te gelir. O zamana kadar düzen `localStorage`'da `jyotish-desk.layout.v1` anahtarıyla tutulur. Biçim spec §3.2'deki biçimdir, artı `schema_version`. M4'te yalnızca `LayoutPersistence`'ın yeni bir uygulaması yazılır.
- **Kırılım noktaları:** pano alanı ≥ 1000 px ise 12 sütun, ≥ 640 px ise 8 sütun, daha darsa 4 sütun.
  - 8 ve 4 sütundaki yeniden akış yalnızca gösterimdir: sürükleme, boyutlama ve klavyeyle düzenleme kapalıdır, kaydedilen 12 sütunluk düzen değişmez.
- **İtme kuralı:** taşınan ya da boyutlanan panel bırakıldığı yerde kalır. Çakıştığı paneller okuma sırasıyla (y, sonra x) aşağı itilir.
  - Kullanıcının bıraktığı boşluklar korunur; otomatik yukarı sıkıştırma yapılmaz.
  - Bu kural klavyeyle tek satırlık taşımayı mümkün kılar. Sıkıştırma olsaydı panel geri zıplardı.
- **Notlar paneli (`Cmd+Shift+N`):** düzende yoksa `x=8, y=0, w=4, h=8` konumuna sabitlenir ve çakışan paneller aşağı itilir. Varsa kaldırılır; diğer paneller yerinde kalır.
- **Hazır düzenler:**
  - Adları sabittir ve i18n'den gösterilir (`layout.interview` …). Yeniden adlandırılamaz ve silinemez, panelleri değiştirilebilir, "Varsayılana döndür" ile sıfırlanır.
  - Transit düzenindeki "tarih kaydırıcı" ayrı bir panel türü değildir. Transit tablosunun `date` ayarıdır (`null` = şimdi).
- **Harita paneli:** varga komutları olmadığı için yalnızca D1 çizilir.
  - D9, D10 gibi vargalar "sonraki sürümde" bilgisini gösterir. Transit halkası ayarı saklanır ama henüz çizilmez.
  - Lagna yoktur: 1. ev Koç kabul edilir ve panelde "Lagna henüz hesaplanmıyor; şimdiki gökyüzü, 1. ev Koç" notu çıkar.
- **Ad girişi ve silme onayı** satır içinde açılır (`window.prompt` kullanılmaz; WKWebView'da güvenilmez). Silme düğmesi düzenin adını taşır.
- **Motor sürümü** (demo'daki "Swiss Ephemeris 2.10.03" satırı) kabuktan kaldırılır. Panellerde yer almaz.

---

### Task 1: Tema belirteçleri ve tema tercihi

**Files:**
- Create: `app/ui/theme/tokens.css`, `app/ui/theme/theme.ts`
- Create/Test: `app/ui/theme/tokens.test.ts`, `app/ui/theme/theme.test.ts`, `app/ui/theme/no-hex.test.ts`
- Modify: `app/vite.config.ts` (Vitest'in `?raw` CSS içe aktarımlarını boşaltmaması için), `app/ui/main.tsx` (`tokens.css` içe aktarımı)

**Interfaces:**
- Produces:
  - CSS değişkenleri: `--bg --panel --line --ink --ink-strong --select --text --text-2 --warn --danger`, `--font-ui`, `--fw-regular` (400), `--fw-medium` (500), `--fs-table` (12px), `--fs-body` (13px), `--fs-panel-title` (15px), `--fs-client` (18px). Koyu tema `:root[data-theme="dark"]`.
  - `type ThemePref = "light" | "dark" | "system"`, `type ResolvedTheme = "light" | "dark"`
  - `THEME_PREFS: readonly ThemePref[]`, `THEME_STORAGE_KEY = "jyotish-desk.theme.v1"`, `DARK_QUERY`
  - `interface DarkMediaQuery { matches; addEventListener("change", l); removeEventListener("change", l) }`
  - `resolveTheme(pref: ThemePref, prefersDark: boolean): ResolvedTheme`
  - `applyTheme(pref: ThemePref, root?: HTMLElement, media?: DarkMediaQuery | null): () => void`: `root.dataset.theme` yazar. `"system"` iken medya sorgusunu dinler; dönen fonksiyon dinleyiciyi kaldırır.
  - `isThemePref(v: unknown): v is ThemePref`, `loadThemePref(storage?): ThemePref` (varsayılan `"system"`), `saveThemePref(pref, storage?): void`

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/theme/tokens.test.ts`:

```ts
import css from "./tokens.css?raw";

// Spec §2.1 tablosu, birebir.
const LIGHT: Record<string, string> = {
  "--bg": "#FAF9F5",
  "--panel": "#FFFFFF",
  "--line": "#D3D1C7",
  "--ink": "#534AB7",
  "--ink-strong": "#3C3489",
  "--select": "#EEEDFE",
  "--text": "#2C2C2A",
  "--text-2": "#5F5E5A",
  "--warn": "#BA7517",
  "--danger": "#A32D2D",
};
const DARK: Record<string, string> = {
  "--bg": "#1B1830",
  "--panel": "#26215C",
  "--line": "#3C3489",
  "--ink": "#AFA9EC",
  "--ink-strong": "#CECBF6",
  "--select": "#3C3489",
  "--text": "#EEEDFE",
  "--text-2": "#AFA9EC",
  "--warn": "#EF9F27",
  "--danger": "#F09595",
};

/** `marker`dan sonraki ilk `{ … }` bloğundaki `--ad: #hex` bildirimleri. */
function colorTokens(marker: string): Record<string, string> {
  const start = css.indexOf(marker);
  expect(start, marker).toBeGreaterThanOrEqual(0);
  const open = css.indexOf("{", start);
  const body = css.slice(open + 1, css.indexOf("}", open));
  const out: Record<string, string> = {};
  for (const m of body.matchAll(/(--[a-z0-9-]+)\s*:\s*(#[0-9A-Fa-f]{6})\s*;/g)) out[m[1]] = m[2];
  return out;
}

function luminance(hex: string): number {
  const [r, g, b] = [1, 3, 5]
    .map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)
    .map((c) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

test("light tokens match the spec table", () => {
  expect(colorTokens(":root {")).toEqual(LIGHT);
});

test("dark tokens match the spec table", () => {
  expect(colorTokens(':root[data-theme="dark"] {')).toEqual(DARK);
});

test("print forces the light palette with only ink and text", () => {
  const print = colorTokens("@media print");
  const allowed = new Set([LIGHT["--ink"], LIGHT["--text"], "#FFFFFF"]);
  expect(Object.keys(print).sort()).toEqual(Object.keys(LIGHT).sort());
  for (const [name, value] of Object.entries(print)) expect(allowed.has(value), name).toBe(true);
  expect(print["--ink"]).toBe(LIGHT["--ink"]);
  expect(print["--text"]).toBe(LIGHT["--text"]);
});

test("typography: system font, two weights, four sizes, tabular numbers, 2px ink focus ring", () => {
  expect(css).toContain("-apple-system");
  expect(css).toMatch(/--fw-regular:\s*400;/);
  expect(css).toMatch(/--fw-medium:\s*500;/);
  for (const px of [12, 13, 15, 18]) expect(css).toContain(`${px}px;`);
  expect(css).toContain("font-variant-numeric: tabular-nums;");
  expect(css).toMatch(/:focus-visible\s*{\s*outline:\s*2px solid var\(--ink\);/);
  const weights = [...css.matchAll(/font-weight:\s*([^;]+);/g)].map((m) => m[1].trim());
  for (const w of weights) expect(["var(--fw-regular)", "var(--fw-medium)"]).toContain(w);
});

test.each([
  ["light", LIGHT],
  ["dark", DARK],
])("%s text tokens reach WCAG AA (4.5:1) on every surface", (_, t) => {
  for (const fg of ["--text", "--text-2", "--ink-strong", "--ink", "--danger"]) {
    for (const bg of ["--bg", "--panel", "--select"]) {
      expect(contrast(t[fg], t[bg]), `${fg} on ${bg}`).toBeGreaterThanOrEqual(4.5);
    }
  }
  // Uyarı rengi metin değil rozet zeminidir (rozet metni --text); grafik öğe eşiği 3:1.
  for (const bg of ["--bg", "--panel"]) {
    expect(contrast(t["--warn"], t[bg]), `--warn on ${bg}`).toBeGreaterThanOrEqual(3);
  }
});
```

`app/ui/theme/theme.test.ts`:

```ts
import {
  applyTheme,
  loadThemePref,
  resolveTheme,
  saveThemePref,
  THEME_STORAGE_KEY,
  type DarkMediaQuery,
} from "./theme";

function fakeMedia(matches: boolean) {
  const listeners = new Set<() => void>();
  const media = {
    matches,
    addEventListener: (_: "change", l: () => void) => listeners.add(l),
    removeEventListener: (_: "change", l: () => void) => listeners.delete(l),
  };
  return {
    media: media as DarkMediaQuery,
    listeners,
    set(v: boolean) {
      media.matches = v;
      listeners.forEach((l) => l());
    },
  };
}

test("resolveTheme honours explicit choices and follows the system otherwise", () => {
  expect(resolveTheme("light", true)).toBe("light");
  expect(resolveTheme("dark", false)).toBe("dark");
  expect(resolveTheme("system", true)).toBe("dark");
  expect(resolveTheme("system", false)).toBe("light");
});

test("applyTheme writes data-theme on the root element", () => {
  const root = document.createElement("html");
  const { media } = fakeMedia(true);
  applyTheme("light", root, media);
  expect(root.dataset.theme).toBe("light");
  applyTheme("dark", root, media);
  expect(root.dataset.theme).toBe("dark");
});

test("system preference tracks prefers-color-scheme changes until disposed", () => {
  const root = document.createElement("html");
  const fake = fakeMedia(false);
  const dispose = applyTheme("system", root, fake.media);
  expect(root.dataset.theme).toBe("light");
  fake.set(true);
  expect(root.dataset.theme).toBe("dark");
  dispose();
  expect(fake.listeners.size).toBe(0);
  fake.set(false);
  expect(root.dataset.theme).toBe("dark");
});

test("explicit preferences do not subscribe to the media query", () => {
  const fake = fakeMedia(false);
  applyTheme("dark", document.createElement("html"), fake.media);
  expect(fake.listeners.size).toBe(0);
});

test("without matchMedia, system resolves to light", () => {
  const root = document.createElement("html");
  applyTheme("system", root, null);
  expect(root.dataset.theme).toBe("light");
});

test("theme preference round-trips through storage and defaults to system", () => {
  localStorage.clear();
  expect(loadThemePref()).toBe("system");
  saveThemePref("dark");
  expect(localStorage.getItem(THEME_STORAGE_KEY)).toBe("dark");
  expect(loadThemePref()).toBe("dark");
  localStorage.setItem(THEME_STORAGE_KEY, "sepia");
  expect(loadThemePref()).toBe("system");
});
```

`app/ui/theme/no-hex.test.ts` (sonraki görevlerde eklenen her `.ts`, `.tsx` ve `.css` dosyasını da kapsar):

```ts
// Renkler yalnızca tokens.css'te hex olarak bulunur (spec §2.1). Bu test sonradan eklenen
// her bileşeni de kapsar.
const sources = import.meta.glob("/ui/**/*.{ts,tsx,css}", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

// tokens.css belirteçlerin kaynağı; bindings.ts üretilmiş; PrintSpike ve print.css
// M7'de baskı düzenleriyle değişecek spike dosyaları; testler beklenen değer olarak hex içerir.
const EXEMPT = [/^\/ui\/theme\/tokens\.css$/, /^\/ui\/bindings\.ts$/, /^\/ui\/PrintSpike\.tsx$/, /^\/ui\/print\.css$/, /\.test\.tsx?$/];

test("no hex colors outside theme tokens", () => {
  const files = Object.keys(sources).filter((f) => !EXEMPT.some((re) => re.test(f)));
  expect(files.length).toBeGreaterThan(0);
  const offenders = files.filter((f) => /#[0-9a-fA-F]{3,8}\b/.test(sources[f]));
  expect(offenders).toEqual([]);
});
```

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

Komut: `cd app && pnpm vitest run ui/theme`
Beklenen: FAIL. `tokens.test.ts` ve `theme.test.ts` için `Failed to resolve import "./tokens.css?raw"` / `"./theme"`. `no-hex.test.ts` geçer (henüz hex içeren bileşen yok).

- [ ] **Step 3: Belirteçleri ve tema modülünü yaz**

`app/ui/theme/tokens.css`:

```css
/* Renk, yazı ve odak belirteçleri (docs/superpowers/specs/2026-11-24-ui-design.md §2).
   Hex değerleri yalnızca bu dosyada bulunur; bileşenler var(--…) kullanır. */

:root {
  color-scheme: light;
  --bg: #FAF9F5;
  --panel: #FFFFFF;
  --line: #D3D1C7;
  --ink: #534AB7;
  --ink-strong: #3C3489;
  --select: #EEEDFE;
  --text: #2C2C2A;
  --text-2: #5F5E5A;
  --warn: #BA7517;
  --danger: #A32D2D;

  --font-ui: -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif;
  --fw-regular: 400;
  --fw-medium: 500;
  --fs-table: 12px;
  --fs-body: 13px;
  --fs-panel-title: 15px;
  --fs-client: 18px;
}

:root[data-theme="dark"] {
  color-scheme: dark;
  --bg: #1B1830;
  --panel: #26215C;
  --line: #3C3489;
  --ink: #AFA9EC;
  --ink-strong: #CECBF6;
  --select: #3C3489;
  --text: #EEEDFE;
  --text-2: #AFA9EC;
  --warn: #EF9F27;
  --danger: #F09595;
}

/* Baskı her zaman açık temadır ve yalnızca --ink ile --text kullanır (spec §2.1, §7). */
@media print {
  :root,
  :root[data-theme="dark"] {
    color-scheme: light;
    --bg: #FFFFFF;
    --panel: #FFFFFF;
    --line: #534AB7;
    --ink: #534AB7;
    --ink-strong: #534AB7;
    --select: #FFFFFF;
    --text: #2C2C2A;
    --text-2: #2C2C2A;
    --warn: #2C2C2A;
    --danger: #2C2C2A;
  }
}

html {
  font-family: var(--font-ui);
  font-size: var(--fs-body);
  font-weight: var(--fw-regular);
  color: var(--text);
  background: var(--bg);
}

body {
  margin: 0;
}

h1,
h2,
h3,
th,
strong {
  font-weight: var(--fw-medium);
}

table,
time,
.num {
  font-variant-numeric: tabular-nums;
}

table {
  font-size: var(--fs-table);
}

:focus-visible {
  outline: 2px solid var(--ink);
  outline-offset: 2px;
}
```

`app/ui/theme/theme.ts`:

```ts
// Tema tercihi: "system" işletim sisteminin açık/koyu ayarını izler (spec §2.1).
export type ThemePref = "light" | "dark" | "system";
export type ResolvedTheme = "light" | "dark";

export const THEME_PREFS: readonly ThemePref[] = ["light", "dark", "system"];
export const THEME_STORAGE_KEY = "jyotish-desk.theme.v1";
export const DARK_QUERY = "(prefers-color-scheme: dark)";

/** `MediaQueryList`'in kullandığımız kısmı; testlerde sahtesi verilir. */
export interface DarkMediaQuery {
  readonly matches: boolean;
  addEventListener(type: "change", listener: () => void): void;
  removeEventListener(type: "change", listener: () => void): void;
}

export function resolveTheme(pref: ThemePref, prefersDark: boolean): ResolvedTheme {
  if (pref === "system") return prefersDark ? "dark" : "light";
  return pref;
}

function defaultMedia(): DarkMediaQuery | null {
  return typeof window.matchMedia === "function" ? window.matchMedia(DARK_QUERY) : null;
}

/**
 * `root.dataset.theme` değerini yazar. `pref === "system"` iken sistem ayarı değiştikçe
 * günceller. Dönen fonksiyon dinleyiciyi kaldırır.
 */
export function applyTheme(
  pref: ThemePref,
  root: HTMLElement = document.documentElement,
  media: DarkMediaQuery | null = defaultMedia(),
): () => void {
  const update = () => {
    root.dataset.theme = resolveTheme(pref, media?.matches ?? false);
  };
  update();
  if (pref !== "system" || media === null) return () => {};
  media.addEventListener("change", update);
  return () => media.removeEventListener("change", update);
}

export function isThemePref(v: unknown): v is ThemePref {
  return typeof v === "string" && (THEME_PREFS as readonly string[]).includes(v);
}

export function loadThemePref(storage: Storage = window.localStorage): ThemePref {
  try {
    const v = storage.getItem(THEME_STORAGE_KEY);
    return isThemePref(v) ? v : "system";
  } catch {
    return "system";
  }
}

export function saveThemePref(pref: ThemePref, storage: Storage = window.localStorage): void {
  try {
    storage.setItem(THEME_STORAGE_KEY, pref);
  } catch (e) {
    console.warn("theme preference not saved", e);
  }
}
```

`app/vite.config.ts` (tamamı):

```ts
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// Tauri dev sunucusu sabit portta bekler; port doluysa hata verilir.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { outDir: "dist", target: "safari16" },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["ui/setupTests.ts"],
    include: ["ui/**/*.test.{ts,tsx}"],
    // Vitest CSS içe aktarımlarını varsayılan olarak boşaltır; `?raw` ile okunan CSS
    // (tema ve renk testleri) ham metin olarak gelsin diye bu içe aktarımlar dahil edilir.
    css: { include: [/\.css\?raw$/] },
  },
});
```

> Vitest CSS dosyalarını varsayılan olarak boş modüle çevirir; `?raw` sorgusu da boş metin döndürür. `css.include` deseni sorgu dahil kimlikle eşleşir. Desen yalnızca `?raw` içe aktarımlarını kapsar, bileşenlerin CSS içe aktarımları yine boş kalır.

`app/ui/main.tsx` (tamamı):

```tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./theme/tokens.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
```

- [ ] **Step 4: Testleri ve derlemeyi çalıştır**

Komut: `cd app && pnpm vitest run ui/theme && pnpm test && pnpm build`
Beklenen:
- `ui/theme`: 13 test geçer.
- `pnpm test`: `Tests  23 passed (23)`.
- `pnpm build`: `tsc` hatasız, `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/theme app/vite.config.ts app/ui/main.tsx
git commit -m "feat(ui): theme tokens, light/dark/system preference, no-hex and contrast guards" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Düzen modeli ve toleranslı ayrıştırma

**Files:**
- Create: `app/ui/layout/model.ts`
- Create/Test: `app/ui/layout/model.test.ts`

**Interfaces:**
- Produces:
  - `PANEL_KINDS = ["chart","graha_table","dasha_tree","dasha_strip","transit_table","notes","birth_info"] as const`, `type KnownPanelKind`
  - `type PanelSettings = Record<string, unknown>`
  - `interface Panel { id: string; kind: string; x: number; y: number; w: number; h: number; settings: PanelSettings }`. `kind` bilinmeyen değer de olabilir.
  - `interface Layout { id: string; name: string; builtin: boolean; panels: Panel[] }`
  - `interface LayoutState { schema_version: number; active_layout_id: string; layouts: Layout[] }`
  - `LAYOUT_SCHEMA_VERSION = 1`
  - `BUILTIN_LAYOUT_IDS = { interview: "builtin:interview", research: "builtin:research", transit: "builtin:transit" }`, `type BuiltinLayoutId`
  - `isKnownPanelKind(kind: string): kind is KnownPanelKind`, `isBuiltinLayoutId(id: string): id is BuiltinLayoutId`
  - `builtinLayout(id: string): Layout | undefined` (taze kopya), `builtinLayouts(): Layout[]`, `defaultLayoutState(): LayoutState`
  - `parseLayoutState(raw: unknown): LayoutState`, `serializeLayoutState(state: LayoutState): string`
- Hazır düzenler (12 sütun, satır = 32 px):

| Düzen | Paneller (`id` kind varga: x,y,w,h) |
| --- | --- |
| `builtin:interview` "Görüşme" | `d1` chart D1: 0,0,6,12 · `d9` chart D9: 6,0,6,12 · `dasha` dasha_strip: 0,12,8,6 · `notes` notes: 8,12,4,6 |
| `builtin:research` "Araştırma" | `d1` chart D1: 0,0,4,10 · `d9` chart D9: 4,0,4,10 · `d10` chart D10: 8,0,4,10 · `grahas` graha_table: 0,10,6,9 · `dasha` dasha_tree: 6,10,6,9 |
| `builtin:transit` "Transit" | `d1` chart D1 (transit_ring): 0,0,7,12 · `transits` transit_table: 7,0,5,12 · `dasha` dasha_strip: 0,12,12,4 |

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/layout/model.test.ts`:

```ts
import {
  BUILTIN_LAYOUT_IDS,
  builtinLayout,
  defaultLayoutState,
  isKnownPanelKind,
  LAYOUT_SCHEMA_VERSION,
  PANEL_KINDS,
  parseLayoutState,
  serializeLayoutState,
  type Layout,
  type Panel,
} from "./model";

const ids = (layouts: Layout[]) => layouts.map((l) => l.id);
const BUILTINS = [BUILTIN_LAYOUT_IDS.interview, BUILTIN_LAYOUT_IDS.research, BUILTIN_LAYOUT_IDS.transit];

function overlaps(a: Panel, b: Panel) {
  return a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
}

test("catalog has the seven v1 panel kinds", () => {
  expect(PANEL_KINDS).toEqual([
    "chart",
    "graha_table",
    "dasha_tree",
    "dasha_strip",
    "transit_table",
    "notes",
    "birth_info",
  ]);
  expect(isKnownPanelKind("chart")).toBe(true);
  expect(isKnownPanelKind("yoga_list")).toBe(false);
});

test("default state has the three builtin layouts with interview active", () => {
  const s = defaultLayoutState();
  expect(s.schema_version).toBe(LAYOUT_SCHEMA_VERSION);
  expect(s.active_layout_id).toBe(BUILTIN_LAYOUT_IDS.interview);
  expect(ids(s.layouts)).toEqual(BUILTINS);
  expect(s.layouts.map((l) => l.name)).toEqual(["Görüşme", "Araştırma", "Transit"]);
  expect(s.layouts.every((l) => l.builtin)).toBe(true);
});

test("builtin presets follow the spec text and fit the 12-column grid without overlap", () => {
  const kinds = (id: string) => builtinLayout(id)!.panels.map((p) => `${p.kind}:${p.settings.varga ?? ""}`);
  expect(kinds(BUILTIN_LAYOUT_IDS.interview)).toEqual(["chart:D1", "chart:D9", "dasha_strip:", "notes:"]);
  expect(kinds(BUILTIN_LAYOUT_IDS.research)).toEqual([
    "chart:D1",
    "chart:D9",
    "chart:D10",
    "graha_table:",
    "dasha_tree:",
  ]);
  expect(kinds(BUILTIN_LAYOUT_IDS.transit)).toEqual(["chart:D1", "transit_table:", "dasha_strip:"]);
  expect(builtinLayout(BUILTIN_LAYOUT_IDS.transit)!.panels[0].settings.transit_ring).toBe(true);

  for (const id of BUILTINS) {
    const panels = builtinLayout(id)!.panels;
    for (const p of panels) {
      expect(p.x + p.w, `${id}/${p.id}`).toBeLessThanOrEqual(12);
      const [minW, minH] = p.kind === "chart" ? [4, 6] : [3, 4];
      expect(p.w).toBeGreaterThanOrEqual(minW);
      expect(p.h).toBeGreaterThanOrEqual(minH);
    }
    for (const a of panels) for (const b of panels) if (a !== b) expect(overlaps(a, b), `${a.id}/${b.id}`).toBe(false);
  }
});

test("builtinLayout returns fresh copies", () => {
  const a = builtinLayout(BUILTIN_LAYOUT_IDS.interview)!;
  a.panels[0].x = 5;
  expect(builtinLayout(BUILTIN_LAYOUT_IDS.interview)!.panels[0].x).toBe(0);
  expect(builtinLayout("user:1")).toBeUndefined();
});

test.each([null, undefined, 42, "x", [], { layouts: "nope" }])("garbage %j parses to the default state", (raw) => {
  expect(parseLayoutState(raw)).toEqual(defaultLayoutState());
});

test("serialize and parse round-trip", () => {
  const s = defaultLayoutState();
  s.layouts.push({
    id: "user:a",
    name: "Benim",
    builtin: false,
    panels: [{ id: "p1", kind: "notes", x: 0, y: 0, w: 4, h: 4, settings: { a: 1 } }],
  });
  s.active_layout_id = "user:a";
  const text = serializeLayoutState(s);
  expect(JSON.parse(text).schema_version).toBe(1);
  expect(parseLayoutState(JSON.parse(text))).toEqual(s);
});

test("unknown fields are ignored, invalid panels dropped, unknown kinds kept", () => {
  const raw = {
    schema_version: 1,
    extra: true,
    active_layout_id: "user:a",
    layouts: [
      {
        id: "user:a",
        name: "  A  ",
        builtin: false,
        color: "red",
        panels: [
          { id: "ok", kind: "chart", x: 0, y: 0, w: 4, h: 6, settings: { style: "south" }, junk: 1 },
          { id: "future", kind: "yoga_list", x: 4, y: 0, w: 3, h: 4 },
          { id: "neg", kind: "notes", x: -1, y: 0, w: 3, h: 4, settings: {} },
          { id: "frac", kind: "notes", x: 0, y: 0.5, w: 3, h: 4, settings: {} },
          { id: "zero", kind: "notes", x: 0, y: 0, w: 0, h: 4, settings: {} },
          { id: "", kind: "notes", x: 0, y: 0, w: 3, h: 4, settings: {} },
          { id: "nokind", x: 0, y: 0, w: 3, h: 4, settings: {} },
          { id: "ok", kind: "notes", x: 0, y: 20, w: 3, h: 4, settings: {} },
          "garbage",
        ],
      },
    ],
  };
  const s = parseLayoutState(raw);
  const a = s.layouts.find((l) => l.id === "user:a")!;
  expect(a.name).toBe("A");
  expect(a).not.toHaveProperty("color");
  expect(a.panels).toEqual([
    { id: "ok", kind: "chart", x: 0, y: 0, w: 4, h: 6, settings: { style: "south" } },
    { id: "future", kind: "yoga_list", x: 4, y: 0, w: 3, h: 4, settings: {} },
  ]);
  expect(s.active_layout_id).toBe("user:a");
});

test("missing builtins are re-added in canonical order before user layouts", () => {
  const s = parseLayoutState({
    active_layout_id: "builtin:research",
    layouts: [
      { id: "user:x", name: "X", builtin: false, panels: [] },
      { id: "builtin:research", name: "Renamed", builtin: true, panels: [] },
    ],
  });
  expect(ids(s.layouts)).toEqual([...BUILTINS, "user:x"]);
  const research = s.layouts[1];
  expect(research.name).toBe("Araştırma");
  expect(research.builtin).toBe(true);
  expect(research.panels).toEqual([]);
  expect(s.active_layout_id).toBe("builtin:research");
});

test("a layout claiming builtin with a non-builtin id becomes a user layout", () => {
  const s = parseLayoutState({ layouts: [{ id: "user:y", name: "Y", builtin: true, panels: [] }] });
  expect(s.layouts.find((l) => l.id === "user:y")!.builtin).toBe(false);
});

test("bad or missing active id falls back to the first builtin", () => {
  expect(parseLayoutState({ active_layout_id: "gone", layouts: [] }).active_layout_id).toBe(BUILTIN_LAYOUT_IDS.interview);
  expect(parseLayoutState({ active_layout_id: 7, layouts: [] }).active_layout_id).toBe(BUILTIN_LAYOUT_IDS.interview);
});

test("duplicate layout ids keep the first occurrence", () => {
  const s = parseLayoutState({
    layouts: [
      { id: "user:d", name: "First", builtin: false, panels: [] },
      { id: "user:d", name: "Second", builtin: false, panels: [] },
    ],
  });
  expect(s.layouts.filter((l) => l.id === "user:d").map((l) => l.name)).toEqual(["First"]);
});

test("a newer schema version is read best-effort", () => {
  const s = parseLayoutState({ schema_version: 99, active_layout_id: "builtin:transit", layouts: [] });
  expect(s.schema_version).toBe(LAYOUT_SCHEMA_VERSION);
  expect(s.active_layout_id).toBe("builtin:transit");
});
```

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

Komut: `cd app && pnpm vitest run ui/layout/model.test.ts`
Beklenen: FAIL, `Failed to resolve import "./model"`.

- [ ] **Step 3: Modeli yaz**

`app/ui/layout/model.ts`:

```ts
// Pano düzeni veri modeli (spec §3.2). Kalıcı biçim `ViewState.panel_layout` ile aynıdır;
// M4'teki store komutu gelene kadar localStorage'da tutulur (bkz. persistence.ts).

/** v1 panel kataloğu (spec §3.3). */
export const PANEL_KINDS = [
  "chart",
  "graha_table",
  "dasha_tree",
  "dasha_strip",
  "transit_table",
  "notes",
  "birth_info",
] as const;
export type KnownPanelKind = (typeof PANEL_KINDS)[number];

export type PanelSettings = Record<string, unknown>;

/**
 * `kind` bilinmeyen bir değer de olabilir (ör. daha yeni bir sürümün kaydettiği panel).
 * Bu paneller atılmaz; pano onları "tanınmıyor" yedeğiyle çizer ve düzen korunur.
 */
export interface Panel {
  id: string;
  kind: string;
  x: number;
  y: number;
  w: number;
  h: number;
  settings: PanelSettings;
}

export interface Layout {
  id: string;
  name: string;
  builtin: boolean;
  panels: Panel[];
}

export interface LayoutState {
  schema_version: number;
  active_layout_id: string;
  layouts: Layout[];
}

export const LAYOUT_SCHEMA_VERSION = 1;

export const BUILTIN_LAYOUT_IDS = {
  interview: "builtin:interview",
  research: "builtin:research",
  transit: "builtin:transit",
} as const;
export type BuiltinLayoutId = (typeof BUILTIN_LAYOUT_IDS)[keyof typeof BUILTIN_LAYOUT_IDS];

const BUILTIN_ORDER: readonly BuiltinLayoutId[] = [
  BUILTIN_LAYOUT_IDS.interview,
  BUILTIN_LAYOUT_IDS.research,
  BUILTIN_LAYOUT_IDS.transit,
];

export function isKnownPanelKind(kind: string): kind is KnownPanelKind {
  return (PANEL_KINDS as readonly string[]).includes(kind);
}

export function isBuiltinLayoutId(id: string): id is BuiltinLayoutId {
  return (BUILTIN_ORDER as readonly string[]).includes(id);
}

function chart(id: string, varga: string, x: number, y: number, w: number, h: number, transitRing = false): Panel {
  return { id, kind: "chart", x, y, w, h, settings: { varga, style: "north", transit_ring: transitRing } };
}

function panel(id: string, kind: KnownPanelKind, x: number, y: number, w: number, h: number, settings: PanelSettings = {}): Panel {
  return { id, kind, x, y, w, h, settings };
}

// 12 sütun, 32 px satır. Tarih kaydırıcı ayrı bir panel türü değildir; transit tablosunun
// `date` ayarıdır (null = şimdi).
const BUILTIN_FACTORIES: Record<BuiltinLayoutId, () => Layout> = {
  [BUILTIN_LAYOUT_IDS.interview]: () => ({
    id: BUILTIN_LAYOUT_IDS.interview,
    name: "Görüşme",
    builtin: true,
    panels: [
      chart("d1", "D1", 0, 0, 6, 12),
      chart("d9", "D9", 6, 0, 6, 12),
      panel("dasha", "dasha_strip", 0, 12, 8, 6, { level: "antar" }),
      panel("notes", "notes", 8, 12, 4, 6),
    ],
  }),
  [BUILTIN_LAYOUT_IDS.research]: () => ({
    id: BUILTIN_LAYOUT_IDS.research,
    name: "Araştırma",
    builtin: true,
    panels: [
      chart("d1", "D1", 0, 0, 4, 10),
      chart("d9", "D9", 4, 0, 4, 10),
      chart("d10", "D10", 8, 0, 4, 10),
      panel("grahas", "graha_table", 0, 10, 6, 9, { columns: ["lon", "nakshatra", "pada", "speed", "retro"] }),
      panel("dasha", "dasha_tree", 6, 10, 6, 9, { system: "vimshottari", open: [] }),
    ],
  }),
  [BUILTIN_LAYOUT_IDS.transit]: () => ({
    id: BUILTIN_LAYOUT_IDS.transit,
    name: "Transit",
    builtin: true,
    panels: [
      chart("d1", "D1", 0, 0, 7, 12, true),
      panel("transits", "transit_table", 7, 0, 5, 12, { date: null }),
      panel("dasha", "dasha_strip", 0, 12, 12, 4, { level: "antar" }),
    ],
  }),
};

/** Hazır düzenin taze kopyası; bilinmeyen kimlikte `undefined`. */
export function builtinLayout(id: string): Layout | undefined {
  return isBuiltinLayoutId(id) ? BUILTIN_FACTORIES[id]() : undefined;
}

export function builtinLayouts(): Layout[] {
  return BUILTIN_ORDER.map((id) => BUILTIN_FACTORIES[id]());
}

export function defaultLayoutState(): LayoutState {
  return {
    schema_version: LAYOUT_SCHEMA_VERSION,
    active_layout_id: BUILTIN_LAYOUT_IDS.interview,
    layouts: builtinLayouts(),
  };
}

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

function isNonEmptyString(v: unknown): v is string {
  return typeof v === "string" && v.trim().length > 0;
}

function isInt(v: unknown, min: number): v is number {
  return typeof v === "number" && Number.isInteger(v) && v >= min;
}

function parsePanel(raw: unknown): Panel | null {
  if (!isRecord(raw)) return null;
  const { id, kind, x, y, w, h, settings } = raw;
  if (!isNonEmptyString(id) || !isNonEmptyString(kind)) return null;
  if (!isInt(x, 0) || !isInt(y, 0) || !isInt(w, 1) || !isInt(h, 1)) return null;
  return { id, kind, x, y, w, h, settings: isRecord(settings) ? { ...settings } : {} };
}

function parsePanels(raw: unknown): Panel[] {
  if (!Array.isArray(raw)) return [];
  const seen = new Set<string>();
  const out: Panel[] = [];
  for (const item of raw) {
    const p = parsePanel(item);
    if (p === null || seen.has(p.id)) continue;
    seen.add(p.id);
    out.push(p);
  }
  return out;
}

function parseLayout(raw: unknown): Layout | null {
  if (!isRecord(raw)) return null;
  const { id, name, panels } = raw;
  if (!isNonEmptyString(id) || !isNonEmptyString(name)) return null;
  const fresh = builtinLayout(id);
  if (fresh) {
    // Hazır düzenin adı sabittir; panelleri kullanıcı değiştirmiş olabilir.
    return { ...fresh, panels: Array.isArray(panels) ? parsePanels(panels) : fresh.panels };
  }
  return { id, name: name.trim(), builtin: false, panels: parsePanels(panels) };
}

/**
 * Toleranslı ayrıştırma (v1 spec §7: bozuk görünüm durumu açılışı asla engellemez):
 * - bilinmeyen alanlar yok sayılır, geçersiz panel ve düzenler atılır,
 * - bilinmeyen panel türleri korunur,
 * - eksik hazır düzenler yeniden eklenir; hazır düzenler her zaman başta ve sabit sıradadır,
 * - geçersiz `active_layout_id` ilk hazır düzene döner.
 * `schema_version` yoksa 1 kabul edilir. Daha yeni bir sürüm de aynı kurallarla okunur.
 */
export function parseLayoutState(raw: unknown): LayoutState {
  if (!isRecord(raw)) return defaultLayoutState();
  const parsed = Array.isArray(raw.layouts) ? raw.layouts.map(parseLayout) : [];
  const byId = new Map<string, Layout>();
  for (const l of parsed) if (l !== null && !byId.has(l.id)) byId.set(l.id, l);

  const builtins = BUILTIN_ORDER.map((id) => byId.get(id) ?? BUILTIN_FACTORIES[id]());
  const users = [...byId.values()].filter((l) => !l.builtin);
  const layouts = [...builtins, ...users];

  const active = raw.active_layout_id;
  const activeId =
    typeof active === "string" && layouts.some((l) => l.id === active) ? active : BUILTIN_LAYOUT_IDS.interview;
  return { schema_version: LAYOUT_SCHEMA_VERSION, active_layout_id: activeId, layouts };
}

export function serializeLayoutState(state: LayoutState): string {
  return JSON.stringify({
    schema_version: LAYOUT_SCHEMA_VERSION,
    active_layout_id: state.active_layout_id,
    layouts: state.layouts,
  });
}
```

- [ ] **Step 4: Testleri ve derlemeyi çalıştır**

Komut: `cd app && pnpm vitest run ui/layout/model.test.ts && pnpm test && pnpm build`
Beklenen:
- `model.test.ts`: 17 test geçer.
- `pnpm test`: `Tests  40 passed (40)`.
- `pnpm build`: `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/layout/model.ts app/ui/layout/model.test.ts
git commit -m "feat(ui): layout model with builtin presets and tolerant versioned parsing" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Izgara motoru

**Files:**
- Create: `app/ui/layout/grid.ts`
- Create/Test: `app/ui/layout/grid.test.ts`

**Interfaces:**
- Consumes: `Panel`, `builtinLayouts()` (Task 2).
- Produces:
  - `GRID_COLUMNS = 12`, `ROW_HEIGHT_PX = 32`, `type Columns = 12 | 8 | 4`, `BREAKPOINTS`
  - `interface Rect { x; y; w; h }`
  - `columnsForWidth(px: number): Columns`: ≥ 1000 → 12, ≥ 640 → 8, aksi 4.
  - `minSize(kind: string): { w: number; h: number }`: `chart` 4×6, diğerleri 3×4.
  - `collides(a: Rect, b: Rect): boolean`, `hasOverlap(panels: readonly Rect[]): boolean`
  - `clampPanel(p: Panel, columns?: number): Panel`
  - `settle(panels: readonly Panel[], pinnedId?: string | null): Panel[]`: çakışmaları aşağı iterek çözer, sırayı korur.
  - `normalizePanels(panels, columns?): Panel[]`
  - `movePanel(panels, id, x, y, columns?): Panel[]`, `resizePanel(panels, id, w, h, columns?): Panel[]`
  - `compact(panels): Panel[]`
  - `reflow(panels, columns: Columns): Panel[]`
  - Tüm fonksiyonlar saftır; giriş dizisini değiştirmez, dönüş dizisi girişle aynı sıradadır.

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/layout/grid.test.ts` (son test 5 tohumla × 3 hazır düzen × 300 rastgele işlemde değişmezleri denetler):

```ts
import {
  clampPanel,
  collides,
  columnsForWidth,
  compact,
  GRID_COLUMNS,
  hasOverlap,
  minSize,
  movePanel,
  normalizePanels,
  reflow,
  resizePanel,
  ROW_HEIGHT_PX,
  settle,
} from "./grid";
import { builtinLayouts, type Panel } from "./model";

function p(id: string, x: number, y: number, w: number, h: number, kind = "notes"): Panel {
  return { id, kind, x, y, w, h, settings: {} };
}
const at = (panels: Panel[], id: string) => {
  const q = panels.find((r) => r.id === id)!;
  return [q.x, q.y, q.w, q.h];
};

/** Değişmezler: çakışma yok, ızgara içinde, en küçük boyut, tamsayı. */
function expectValid(panels: Panel[], columns = GRID_COLUMNS) {
  expect(hasOverlap(panels)).toBe(false);
  for (const q of panels) {
    const m = minSize(q.kind);
    expect(q.x).toBeGreaterThanOrEqual(0);
    expect(q.y).toBeGreaterThanOrEqual(0);
    expect(q.x + q.w).toBeLessThanOrEqual(columns);
    expect(q.w).toBeGreaterThanOrEqual(Math.min(m.w, columns));
    expect(q.h).toBeGreaterThanOrEqual(m.h);
    for (const v of [q.x, q.y, q.w, q.h]) expect(Number.isInteger(v)).toBe(true);
  }
}

test("constants follow the spec", () => {
  expect(GRID_COLUMNS).toBe(12);
  expect(ROW_HEIGHT_PX).toBe(32);
  expect(minSize("chart")).toEqual({ w: 4, h: 6 });
  expect(minSize("notes")).toEqual({ w: 3, h: 4 });
  expect(minSize("something_new")).toEqual({ w: 3, h: 4 });
});

test("columnsForWidth breakpoints", () => {
  expect(columnsForWidth(1600)).toBe(12);
  expect(columnsForWidth(1000)).toBe(12);
  expect(columnsForWidth(999)).toBe(8);
  expect(columnsForWidth(640)).toBe(8);
  expect(columnsForWidth(639)).toBe(4);
  expect(columnsForWidth(0)).toBe(4);
});

test("collides treats touching edges as free", () => {
  expect(collides(p("a", 0, 0, 3, 4), p("b", 3, 0, 3, 4))).toBe(false);
  expect(collides(p("a", 0, 0, 3, 4), p("b", 0, 4, 3, 4))).toBe(false);
  expect(collides(p("a", 0, 0, 3, 4), p("b", 2, 3, 3, 4))).toBe(true);
});

test("clampPanel enforces minimum size, grid bounds and integers", () => {
  expect(at([clampPanel(p("a", -2, -1, 1, 1))], "a")).toEqual([0, 0, 3, 4]);
  expect(at([clampPanel(p("c", 0, 0, 2, 2, "chart"))], "c")).toEqual([0, 0, 4, 6]);
  expect(at([clampPanel(p("a", 11, 0, 5, 4))], "a")).toEqual([7, 0, 5, 4]);
  expect(at([clampPanel(p("a", 0, 0, 30, 4))], "a")).toEqual([0, 0, 12, 4]);
  expect(at([clampPanel(p("a", 1.4, 2.6, 3.5, 4.4))], "a")).toEqual([1, 3, 4, 4]);
});

test("moving onto a panel pushes it down; the moved panel stays put", () => {
  const panels = [p("a", 0, 0, 6, 4), p("b", 6, 0, 6, 4)];
  const out = movePanel(panels, "b", 2, 2);
  expect(at(out, "b")).toEqual([2, 2, 6, 4]);
  expect(at(out, "a")).toEqual([0, 6, 6, 4]);
  expectValid(out);
});

test("pushing cascades through a stack", () => {
  const panels = [p("a", 0, 0, 4, 4), p("b", 0, 4, 4, 4), p("c", 0, 8, 4, 4), p("x", 6, 0, 4, 4)];
  const out = movePanel(panels, "x", 0, 2);
  expect(at(out, "x")).toEqual([0, 2, 4, 4]);
  expect(at(out, "a")).toEqual([0, 6, 4, 4]);
  expect(at(out, "b")).toEqual([0, 10, 4, 4]);
  expect(at(out, "c")).toEqual([0, 14, 4, 4]);
  expectValid(out);
});

test("panels that do not collide keep their positions (gaps are preserved)", () => {
  const panels = [p("a", 0, 0, 4, 4), p("b", 8, 10, 4, 4)];
  const out = movePanel(panels, "a", 0, 3);
  expect(at(out, "a")).toEqual([0, 3, 4, 4]);
  expect(at(out, "b")).toEqual([8, 10, 4, 4]);
});

test("move clamps to the grid and keeps array order", () => {
  const panels = [p("a", 0, 0, 4, 4), p("b", 4, 0, 4, 4)];
  const out = movePanel(panels, "b", 20, -3);
  expect(out.map((q) => q.id)).toEqual(["a", "b"]);
  expect(at(out, "b")).toEqual([8, 0, 4, 4]);
});

test("moving an unknown id returns an equal copy", () => {
  const panels = [p("a", 0, 0, 4, 4)];
  expect(movePanel(panels, "zz", 1, 1)).toEqual(panels);
});

test("resize grows into neighbours and pushes them down", () => {
  const panels = [p("a", 0, 0, 4, 4), p("b", 4, 0, 4, 4), p("c", 0, 4, 8, 4)];
  const out = resizePanel(panels, "a", 6, 6);
  expect(at(out, "a")).toEqual([0, 0, 6, 6]);
  expect(at(out, "b")).toEqual([4, 6, 4, 4]);
  expect(at(out, "c")).toEqual([0, 10, 8, 4]);
  expectValid(out);
});

test("resize respects minimum size and the right edge", () => {
  const panels = [p("c", 8, 0, 4, 6, "chart"), p("n", 0, 0, 3, 4)];
  expect(at(resizePanel(panels, "c", 1, 1), "c")).toEqual([8, 0, 4, 6]);
  expect(at(resizePanel(panels, "n", 40, 4), "n")).toEqual([0, 0, 12, 4]);
  expect(at(resizePanel(panels, "c", 9, 6), "c")).toEqual([8, 0, 4, 6]);
});

test("settle without a pin resolves overlapping input in reading order", () => {
  const out = settle([p("a", 0, 0, 4, 4), p("b", 0, 1, 4, 4), p("c", 2, 2, 4, 4)]);
  expect(at(out, "a")).toEqual([0, 0, 4, 4]);
  expect(at(out, "b")).toEqual([0, 4, 4, 4]);
  expect(at(out, "c")).toEqual([2, 8, 4, 4]);
});

test("normalizePanels clamps and de-overlaps stored panels", () => {
  const out = normalizePanels([p("c", 10, 0, 2, 2, "chart"), p("n", 9, 0, 3, 4)]);
  expect(at(out, "c")).toEqual([8, 0, 4, 6]);
  expect(at(out, "n")).toEqual([9, 6, 3, 4]);
  expectValid(out);
});

test("compact floats panels up without passing each other", () => {
  const out = compact([p("a", 0, 5, 4, 4), p("b", 0, 20, 4, 4), p("c", 6, 3, 4, 4)]);
  expect(at(out, "a")).toEqual([0, 0, 4, 4]);
  expect(at(out, "b")).toEqual([0, 4, 4, 4]);
  expect(at(out, "c")).toEqual([6, 0, 4, 4]);
});

test("reflow to 8 columns scales widths and keeps reading order", () => {
  const interview = builtinLayouts()[0].panels;
  const out = reflow(interview, 8);
  expectValid(out, 8);
  expect(out.map((q) => [q.id, q.x, q.y, q.w])).toEqual([
    ["d1", 0, 0, 4],
    ["d9", 4, 0, 4],
    ["dasha", 0, 12, 5],
    ["notes", 5, 12, 3],
  ]);
});

test("reflow to 4 columns stacks every panel full width in reading order", () => {
  const research = builtinLayouts()[1].panels;
  const out = reflow(research, 4);
  expectValid(out, 4);
  const sorted = [...out].sort((a, b) => a.y - b.y);
  expect(sorted.map((q) => q.id)).toEqual(["d1", "d9", "d10", "grahas", "dasha"]);
  expect(out.every((q) => q.x === 0 && q.w === 4)).toBe(true);
  expect(out.map((q) => q.h)).toEqual(research.map((q) => q.h));
});

test("reflow at 12 columns is a normalisation, not a re-layout", () => {
  const research = builtinLayouts()[1].panels;
  expect(reflow(research, 12)).toEqual(research);
});

/** Deterministik sözde rastgele (mulberry32). */
function rng(seed: number) {
  return () => {
    seed |= 0;
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

test.each([1, 2, 3, 4, 5])("no overlap across 300 random moves and resizes (seed %i)", (seed) => {
  const rand = rng(seed);
  const int = (lo: number, hi: number) => lo + Math.floor(rand() * (hi - lo + 1));
  for (const layout of builtinLayouts()) {
    let panels = layout.panels;
    for (let i = 0; i < 300; i++) {
      const target = panels[int(0, panels.length - 1)];
      panels =
        rand() < 0.5
          ? movePanel(panels, target.id, int(-2, 13), int(-2, 30))
          : resizePanel(panels, target.id, int(0, 14), int(0, 14));
      expectValid(panels);
    }
    for (const cols of [8, 4] as const) expectValid(reflow(panels, cols), cols);
  }
});
```

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

Komut: `cd app && pnpm vitest run ui/layout/grid.test.ts`
Beklenen: FAIL, `Failed to resolve import "./grid"`.

- [ ] **Step 3: Izgara motorunu yaz**

`app/ui/layout/grid.ts`:

```ts
// Pano ızgarası (spec §3.1): saf fonksiyonlar, React yok. Paneller hiçbir zaman üst üste
// binmez; taşınan ya da boyutlanan panel yerinde kalır, çakıştığı paneller aşağı itilir.
// Kullanıcının bıraktığı boşluklar korunur (otomatik yukarı sıkıştırma yalnızca dar
// pencerede yeniden akışta yapılır).
import type { Panel } from "./model";

export const GRID_COLUMNS = 12;
export const ROW_HEIGHT_PX = 32;
export type Columns = 12 | 8 | 4;

/**
 * Kırılım noktaları (pano alanının piksel genişliği):
 * - ≥ 1000 px → 12 sütun (en küçük 3 sütunluk panel ≈ 240 px),
 * - ≥ 640 px → 8 sütun,
 * - daha dar → 4 sütun (her panel tam genişlik, alt alta).
 */
export const BREAKPOINTS: ReadonlyArray<{ minWidth: number; columns: Columns }> = [
  { minWidth: 1000, columns: 12 },
  { minWidth: 640, columns: 8 },
  { minWidth: 0, columns: 4 },
];

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export function columnsForWidth(px: number): Columns {
  for (const b of BREAKPOINTS) if (px >= b.minWidth) return b.columns;
  return 4;
}

/** En küçük boyut: harita 4 × 6, diğerleri 3 × 4 (spec §3.1). */
export function minSize(kind: string): { w: number; h: number } {
  return kind === "chart" ? { w: 4, h: 6 } : { w: 3, h: 4 };
}

export function collides(a: Rect, b: Rect): boolean {
  return a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
}

export function hasOverlap(panels: readonly Rect[]): boolean {
  for (let i = 0; i < panels.length; i++)
    for (let j = i + 1; j < panels.length; j++) if (collides(panels[i], panels[j])) return true;
  return false;
}

/** Tamsayıya yuvarlar, en küçük boyutu ve ızgara sınırlarını uygular. */
export function clampPanel(p: Panel, columns: number = GRID_COLUMNS): Panel {
  const m = minSize(p.kind);
  const w = Math.max(Math.min(m.w, columns), Math.min(columns, Math.round(p.w)));
  const h = Math.max(m.h, Math.round(p.h));
  const x = Math.max(0, Math.min(columns - w, Math.round(p.x)));
  const y = Math.max(0, Math.round(p.y));
  return { ...p, x, y, w, h };
}

function byReadingOrder(a: Rect, b: Rect): number {
  return a.y - b.y || a.x - b.x;
}

/**
 * Çakışmaları aşağı iterek çözer. `pinnedId` verilirse o panel yerinden oynamaz.
 * Diğerleri okuma sırasıyla (y, sonra x) yerleştirilir; çakışan panel, çakıştığı
 * panellerin altına iner. Dönüş dizisi girişle aynı sıradadır.
 */
export function settle(panels: readonly Panel[], pinnedId: string | null = null): Panel[] {
  const pinned = pinnedId === null ? undefined : panels.find((p) => p.id === pinnedId);
  const placed: Panel[] = pinned ? [pinned] : [];
  const rest = panels.filter((p) => p !== pinned).sort(byReadingOrder);
  const out = new Map<string, Panel>(pinned ? [[pinned.id, pinned]] : []);
  for (const p of rest) {
    let q = p;
    for (;;) {
      const hits = placed.filter((r) => collides(q, r));
      if (hits.length === 0) break;
      q = { ...q, y: Math.max(...hits.map((r) => r.y + r.h)) };
    }
    placed.push(q);
    out.set(q.id, q);
  }
  return panels.map((p) => out.get(p.id)!);
}

/** Yüklenen ya da dışarıdan gelen panelleri geçerli hale getirir: sınırla, sonra çakışmaları çöz. */
export function normalizePanels(panels: readonly Panel[], columns: number = GRID_COLUMNS): Panel[] {
  return settle(panels.map((p) => clampPanel(p, columns)));
}

export function movePanel(panels: readonly Panel[], id: string, x: number, y: number, columns: number = GRID_COLUMNS): Panel[] {
  const p = panels.find((q) => q.id === id);
  if (!p) return [...panels];
  const moved = clampPanel({ ...p, x, y }, columns);
  return settle(panels.map((q) => (q.id === id ? moved : q)), id);
}

export function resizePanel(panels: readonly Panel[], id: string, w: number, h: number, columns: number = GRID_COLUMNS): Panel[] {
  const p = panels.find((q) => q.id === id);
  if (!p) return [...panels];
  // Sağ kenar ızgaradan taşmasın: genişlik önce kalan sütunlarla sınırlanır.
  const resized = clampPanel({ ...p, w: Math.min(w, columns - p.x), h }, columns);
  return settle(panels.map((q) => (q.id === id ? resized : q)), id);
}

/** Her paneli olabildiğince yukarı taşır (sırayı bozmadan, üstündekinin içinden geçmeden). */
export function compact(panels: readonly Panel[]): Panel[] {
  const placed: Panel[] = [];
  const out = new Map<string, Panel>();
  for (const p of [...panels].sort(byReadingOrder)) {
    let q = p;
    while (placed.some((r) => collides(q, r))) q = { ...q, y: q.y + 1 };
    while (q.y > 0 && !placed.some((r) => collides({ ...q, y: q.y - 1 }, r))) q = { ...q, y: q.y - 1 };
    placed.push(q);
    out.set(q.id, q);
  }
  return panels.map((p) => out.get(p.id)!);
}

/**
 * Dar pencere için yeniden akış (spec §3.1): okuma sırası korunur, genişlikler
 * sütun sayısına ölçeklenir ve paneller satır satır, soldan sağa dizilir.
 * 4 sütunda her panel tam genişliktir (alt alta). Sonuç yalnızca gösterim içindir;
 * kaydedilen düzen 12 sütunluk düzendir.
 */
export function reflow(panels: readonly Panel[], columns: Columns): Panel[] {
  if (columns === GRID_COLUMNS) return normalizePanels(panels, columns);
  const ordered = [...panels].sort(byReadingOrder);
  const out = new Map<string, Panel>();
  let x = 0;
  let rowY = 0;
  let rowBottom = 0;
  for (const p of ordered) {
    const scaled = columns === 4 ? 4 : Math.round((p.w * columns) / GRID_COLUMNS);
    const c = clampPanel({ ...p, w: scaled, x: 0, y: 0 }, columns);
    if (x + c.w > columns) {
      x = 0;
      rowY = rowBottom;
    }
    const placed = { ...c, x, y: rowY };
    out.set(p.id, placed);
    x += c.w;
    rowBottom = Math.max(rowBottom, rowY + c.h);
  }
  return panels.map((p) => out.get(p.id)!);
}
```

- [ ] **Step 4: Testleri ve derlemeyi çalıştır**

Komut: `cd app && pnpm vitest run ui/layout/grid.test.ts && pnpm test && pnpm build`
Beklenen:
- `grid.test.ts`: 22 test geçer.
- `pnpm test`: `Tests  62 passed (62)`.
- `pnpm build`: `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/layout/grid.ts app/ui/layout/grid.test.ts
git commit -m "feat(ui): hand-written 12-column grid engine with push-down collisions and reflow" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Düzen deposu, kalıcılık ve `Cmd+1…9`

**Files:**
- Create: `app/ui/layout/persistence.ts`, `app/ui/layout/reducer.ts`, `app/ui/layout/store.tsx`
- Create/Test: `app/ui/layout/reducer.test.ts`, `app/ui/layout/store.test.tsx`

**Interfaces:**
- Consumes: `LayoutState`, `Layout`, `Panel`, `builtinLayout`, `parseLayoutState`, `serializeLayoutState` (Task 2); `movePanel`, `normalizePanels` (Task 3).
- Produces:
  - `persistence.ts`:
    - `interface LayoutPersistence { load(): unknown | null; save(state: LayoutState): void }`
    - `LAYOUT_STORAGE_KEY = "jyotish-desk.layout.v1"`
    - `localStoragePersistence(storage?: Storage): LayoutPersistence`
    - `memoryPersistence(initial?: unknown): LayoutPersistence & { saved: LayoutState[] }`
  - `reducer.ts`:
    - `type LayoutAction`, varyantlar:
      - `{type:"select"; id}`
      - `{type:"selectIndex"; index}`
      - `{type:"saveAs"; id; name}`
      - `{type:"rename"; id; name}`
      - `{type:"duplicate"; sourceId; id; name}`
      - `{type:"delete"; id}`
      - `{type:"resetBuiltin"; id}`
      - `{type:"updatePanels"; panels}`
      - `{type:"toggleNotes"; panelId}`
    - `layoutReducer(state, action): LayoutState`
    - `activeLayout(state): Layout`
    - `normalizeState(state): LayoutState`
    - `newId(prefix: string): string`
    - `NOTES_PANEL = { x: 8, y: 0, w: 4, h: 8 }`
  - `store.tsx`:
    - `SAVE_DEBOUNCE_MS = 500`
    - `<LayoutProvider persistence debounceMs?>`
    - `useLayoutStore(): LayoutStore`, alanlar: `state`, `active`, `dispatch`, `select(id)`, `saveAs(name)`, `rename(id, name)`, `duplicate(sourceId, name)`, `remove(id)`, `resetBuiltin(id)`, `updatePanels(panels)`, `toggleNotes()`.
- Davranış:
  - İlk durum kaydedilmez. Her değişiklikten 500 ms sonra tek kayıt yapılır.
  - `pagehide` bekleyen kaydı hemen yazar.
  - `Cmd+1…9` (Shift, Alt ve Ctrl olmadan) sıradaki düzeni seçer.

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/layout/reducer.test.ts`:

```ts
import { hasOverlap } from "./grid";
import { BUILTIN_LAYOUT_IDS, builtinLayout, defaultLayoutState, type LayoutState } from "./model";
import { activeLayout, layoutReducer, newId, NOTES_PANEL, normalizeState } from "./reducer";

const { interview, research, transit } = BUILTIN_LAYOUT_IDS;

function withUser(): LayoutState {
  return layoutReducer(defaultLayoutState(), { type: "saveAs", id: "user:1", name: "Benim" });
}

test("select switches the active layout and ignores unknown ids", () => {
  const s = layoutReducer(defaultLayoutState(), { type: "select", id: research });
  expect(s.active_layout_id).toBe(research);
  expect(layoutReducer(s, { type: "select", id: "nope" })).toBe(s);
});

test("selectIndex follows layout order and ignores out-of-range", () => {
  const s = withUser();
  expect(layoutReducer(s, { type: "selectIndex", index: 2 }).active_layout_id).toBe(transit);
  expect(layoutReducer(s, { type: "selectIndex", index: 3 }).active_layout_id).toBe("user:1");
  expect(layoutReducer(s, { type: "selectIndex", index: 8 })).toBe(s);
});

test("saveAs copies the active layout, appends it and makes it active", () => {
  const s = withUser();
  const copy = s.layouts.at(-1)!;
  expect(copy).toMatchObject({ id: "user:1", name: "Benim", builtin: false });
  expect(copy.panels).toEqual(builtinLayout(interview)!.panels);
  expect(s.active_layout_id).toBe("user:1");
  // Derin kopya: kopyanın ayarı değişince hazır düzen etkilenmez.
  copy.panels[0].settings.style = "south";
  expect(s.layouts[0].panels[0].settings.style).toBe("north");
});

test("saveAs rejects blank names and duplicate ids", () => {
  const s = defaultLayoutState();
  expect(layoutReducer(s, { type: "saveAs", id: "user:1", name: "   " })).toBe(s);
  const u = withUser();
  expect(layoutReducer(u, { type: "saveAs", id: "user:1", name: "Again" })).toBe(u);
});

test("rename changes user layouts only", () => {
  const s = withUser();
  expect(layoutReducer(s, { type: "rename", id: "user:1", name: " Yeni " }).layouts.at(-1)!.name).toBe("Yeni");
  expect(layoutReducer(s, { type: "rename", id: interview, name: "X" })).toBe(s);
  expect(layoutReducer(s, { type: "rename", id: "user:1", name: "" })).toBe(s);
});

test("duplicate inserts a copy right after its source without changing the active layout", () => {
  const s = layoutReducer(defaultLayoutState(), { type: "duplicate", sourceId: research, id: "user:2", name: "Araştırma kopyası" });
  expect(s.layouts.map((l) => l.id)).toEqual([interview, research, "user:2", transit]);
  expect(s.layouts[2].builtin).toBe(false);
  expect(s.layouts[2].panels).toEqual(builtinLayout(research)!.panels);
  expect(s.active_layout_id).toBe(interview);
});

test("builtins cannot be deleted; deleting the active user layout falls back to the first", () => {
  const s = withUser();
  expect(layoutReducer(s, { type: "delete", id: interview })).toBe(s);
  const d = layoutReducer(s, { type: "delete", id: "user:1" });
  expect(d.layouts.map((l) => l.id)).toEqual([interview, research, transit]);
  expect(d.active_layout_id).toBe(interview);
});

test("resetBuiltin restores the preset panels", () => {
  let s = defaultLayoutState();
  s = layoutReducer(s, { type: "updatePanels", panels: [] });
  expect(activeLayout(s).panels).toEqual([]);
  s = layoutReducer(s, { type: "resetBuiltin", id: interview });
  expect(activeLayout(s).panels).toEqual(builtinLayout(interview)!.panels);
  expect(layoutReducer(s, { type: "resetBuiltin", id: "user:1" })).toBe(s);
});

test("updatePanels normalises the active layout's panels", () => {
  const s = layoutReducer(defaultLayoutState(), {
    type: "updatePanels",
    panels: [
      { id: "a", kind: "chart", x: 0, y: 0, w: 1, h: 1, settings: {} },
      { id: "b", kind: "notes", x: 0, y: 0, w: 3, h: 4, settings: {} },
    ],
  });
  const panels = activeLayout(s).panels;
  expect(panels[0]).toMatchObject({ x: 0, y: 0, w: 4, h: 6 });
  expect(hasOverlap(panels)).toBe(false);
});

test("toggleNotes removes notes when present and adds a 4-column panel on the right when absent", () => {
  let s = defaultLayoutState();
  s = layoutReducer(s, { type: "toggleNotes", panelId: "n1" });
  expect(activeLayout(s).panels.some((p) => p.kind === "notes")).toBe(false);
  s = layoutReducer(s, { type: "toggleNotes", panelId: "n2" });
  const panels = activeLayout(s).panels;
  expect(panels.find((p) => p.id === "n2")).toMatchObject({ kind: "notes", ...NOTES_PANEL });
  expect(hasOverlap(panels)).toBe(false);
  // Sağ üstteki D9 haritası notların altına itildi.
  expect(panels.find((p) => p.id === "d9")!.y).toBe(NOTES_PANEL.h);
});

test("normalizeState fixes stored geometry", () => {
  const s = defaultLayoutState();
  s.layouts[0].panels = [
    { id: "a", kind: "notes", x: 0, y: 0, w: 3, h: 4, settings: {} },
    { id: "b", kind: "notes", x: 1, y: 1, w: 3, h: 4, settings: {} },
  ];
  expect(hasOverlap(normalizeState(s).layouts[0].panels)).toBe(false);
});

test("newId yields distinct prefixed ids", () => {
  const a = newId("user");
  const b = newId("user");
  expect(a).toMatch(/^user:/);
  expect(a).not.toBe(b);
});
```

`app/ui/layout/store.test.tsx`:

```tsx
import { act, fireEvent, render, screen } from "@testing-library/react";
import { BUILTIN_LAYOUT_IDS, serializeLayoutState, type LayoutState } from "./model";
import { LAYOUT_STORAGE_KEY, localStoragePersistence, memoryPersistence } from "./persistence";
import { LayoutProvider, SAVE_DEBOUNCE_MS, useLayoutStore } from "./store";

function Probe() {
  const store = useLayoutStore();
  return (
    <div>
      <span data-testid="active">{store.active.id}</span>
      <span data-testid="count">{store.state.layouts.length}</span>
      <button onClick={() => store.saveAs("Benim")}>saveAs</button>
      <button onClick={() => store.select(BUILTIN_LAYOUT_IDS.research)}>research</button>
    </div>
  );
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

test("loads persisted state through the tolerant parser", () => {
  const p = memoryPersistence({ active_layout_id: BUILTIN_LAYOUT_IDS.transit, layouts: [] });
  render(<LayoutProvider persistence={p}><Probe /></LayoutProvider>);
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  expect(screen.getByTestId("count")).toHaveTextContent("3");
});

test("does not save on mount; saves once 500 ms after the last change", () => {
  const p = memoryPersistence();
  render(<LayoutProvider persistence={p}><Probe /></LayoutProvider>);
  act(() => vi.advanceTimersByTime(2000));
  expect(p.saved).toHaveLength(0);

  fireEvent.click(screen.getByText("research"));
  act(() => vi.advanceTimersByTime(300));
  fireEvent.click(screen.getByText("saveAs"));
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS - 1));
  expect(p.saved).toHaveLength(0);
  act(() => vi.advanceTimersByTime(1));
  expect(p.saved).toHaveLength(1);
  expect(p.saved[0].layouts).toHaveLength(4);
  expect(p.saved[0].active_layout_id).toMatch(/^user:/);
});

test("a pending save is flushed on pagehide", () => {
  const p = memoryPersistence();
  render(<LayoutProvider persistence={p}><Probe /></LayoutProvider>);
  fireEvent.click(screen.getByText("research"));
  fireEvent(window, new Event("pagehide"));
  expect(p.saved).toHaveLength(1);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS));
  expect(p.saved).toHaveLength(1);
});

test("Cmd+1…9 selects layouts by order", () => {
  render(<LayoutProvider persistence={memoryPersistence()}><Probe /></LayoutProvider>);
  fireEvent.keyDown(window, { key: "3", metaKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "2", metaKey: true, shiftKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "1" });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "1", metaKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.interview);
});

test("useLayoutStore outside the provider throws", () => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  expect(() => render(<Probe />)).toThrow(/LayoutProvider/);
});

test("localStorage persistence round-trips and survives corrupt data", () => {
  vi.spyOn(console, "warn").mockImplementation(() => {});
  localStorage.clear();
  const p = localStoragePersistence();
  expect(p.load()).toBeNull();
  const state: LayoutState = { schema_version: 1, active_layout_id: BUILTIN_LAYOUT_IDS.research, layouts: [] };
  p.save(state);
  expect(localStorage.getItem(LAYOUT_STORAGE_KEY)).toBe(serializeLayoutState(state));
  expect(p.load()).toEqual(state);
  localStorage.setItem(LAYOUT_STORAGE_KEY, "{not json");
  expect(p.load()).toBeNull();
});
```

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

Komut: `cd app && pnpm vitest run ui/layout/reducer.test.ts ui/layout/store.test.tsx`
Beklenen: FAIL, `Failed to resolve import "./reducer"` ve `"./persistence"`.

- [ ] **Step 3: Kalıcılık, reducer ve depoyu yaz**

`app/ui/layout/persistence.ts`:

```ts
// Düzen kalıcılığı. Geçici çözüm: M4'te `ViewState.panel_layout`'u okuyup yazan store
// komutu gelene kadar düzen localStorage'da tutulur. O komut geldiğinde yalnızca bu
// arayüzün yeni bir uygulaması yazılır; store ve pano değişmez.
import { serializeLayoutState, type LayoutState } from "./model";

export interface LayoutPersistence {
  /** Kayıtlı ham değer (ayrıştırılmamış) ya da kayıt yoksa `null`. */
  load(): unknown | null;
  save(state: LayoutState): void;
}

export const LAYOUT_STORAGE_KEY = "jyotish-desk.layout.v1";

export function localStoragePersistence(storage: Storage = window.localStorage): LayoutPersistence {
  return {
    load() {
      try {
        const text = storage.getItem(LAYOUT_STORAGE_KEY);
        return text === null ? null : (JSON.parse(text) as unknown);
      } catch (e) {
        // Bozuk JSON açılışı engellemez; varsayılan düzen kullanılır (v1 spec §7).
        console.warn("stored layout unreadable; using defaults", e);
        return null;
      }
    },
    save(state) {
      try {
        storage.setItem(LAYOUT_STORAGE_KEY, serializeLayoutState(state));
      } catch (e) {
        console.warn("layout not saved", e);
      }
    },
  };
}

/** Bellek içi kalıcılık: testler ve önizleme için. `saved` her kaydı sırayla tutar. */
export function memoryPersistence(initial: unknown = null): LayoutPersistence & { saved: LayoutState[] } {
  const saved: LayoutState[] = [];
  return {
    saved,
    load: () => (saved.length > 0 ? JSON.parse(serializeLayoutState(saved[saved.length - 1])) : initial),
    save: (state) => {
      saved.push(state);
    },
  };
}
```

`app/ui/layout/reducer.ts`:

```ts
// Düzen durumu için saf reducer. Kimlikler eylemle gelir; reducer rastgelelik üretmez.
import { movePanel, normalizePanels } from "./grid";
import { builtinLayout, type Layout, type LayoutState, type Panel } from "./model";

export type LayoutAction =
  | { type: "select"; id: string }
  | { type: "selectIndex"; index: number }
  | { type: "saveAs"; id: string; name: string }
  | { type: "rename"; id: string; name: string }
  | { type: "duplicate"; sourceId: string; id: string; name: string }
  | { type: "delete"; id: string }
  | { type: "resetBuiltin"; id: string }
  | { type: "updatePanels"; panels: Panel[] }
  | { type: "toggleNotes"; panelId: string };

/** Notlar paneli düzende yoksa sağa 4 sütun genişliğinde eklenir (spec §3.2). */
export const NOTES_PANEL = { x: 8, y: 0, w: 4, h: 8 } as const;

let seq = 0;
/** Oturum içinde benzersiz kimlik (ör. "user:m3k9x2a0"). */
export function newId(prefix: string): string {
  seq += 1;
  return `${prefix}:${Date.now().toString(36)}${seq.toString(36)}`;
}

export function activeLayout(state: LayoutState): Layout {
  return state.layouts.find((l) => l.id === state.active_layout_id) ?? state.layouts[0];
}

/** Yüklenen durumdaki her düzenin panellerini geçerli hale getirir. */
export function normalizeState(state: LayoutState): LayoutState {
  return { ...state, layouts: state.layouts.map((l) => ({ ...l, panels: normalizePanels(l.panels) })) };
}

function copyPanels(panels: Panel[]): Panel[] {
  return panels.map((p) => ({ ...p, settings: structuredClone(p.settings) }));
}

function mapLayout(state: LayoutState, id: string, f: (l: Layout) => Layout): LayoutState {
  return { ...state, layouts: state.layouts.map((l) => (l.id === id ? f(l) : l)) };
}

export function layoutReducer(state: LayoutState, action: LayoutAction): LayoutState {
  switch (action.type) {
    case "select":
      return state.layouts.some((l) => l.id === action.id) ? { ...state, active_layout_id: action.id } : state;
    case "selectIndex": {
      const target = state.layouts[action.index];
      return target ? { ...state, active_layout_id: target.id } : state;
    }
    case "saveAs": {
      const name = action.name.trim();
      if (name === "" || state.layouts.some((l) => l.id === action.id)) return state;
      const copy: Layout = { id: action.id, name, builtin: false, panels: copyPanels(activeLayout(state).panels) };
      return { ...state, active_layout_id: copy.id, layouts: [...state.layouts, copy] };
    }
    case "rename": {
      const name = action.name.trim();
      const target = state.layouts.find((l) => l.id === action.id);
      if (name === "" || !target || target.builtin) return state;
      return mapLayout(state, action.id, (l) => ({ ...l, name }));
    }
    case "duplicate": {
      const name = action.name.trim();
      const index = state.layouts.findIndex((l) => l.id === action.sourceId);
      if (name === "" || index < 0 || state.layouts.some((l) => l.id === action.id)) return state;
      const copy: Layout = { id: action.id, name, builtin: false, panels: copyPanels(state.layouts[index].panels) };
      const layouts = [...state.layouts];
      layouts.splice(index + 1, 0, copy);
      return { ...state, layouts };
    }
    case "delete": {
      const target = state.layouts.find((l) => l.id === action.id);
      if (!target || target.builtin) return state;
      const layouts = state.layouts.filter((l) => l.id !== action.id);
      const active = state.active_layout_id === action.id ? layouts[0].id : state.active_layout_id;
      return { ...state, layouts, active_layout_id: active };
    }
    case "resetBuiltin": {
      const fresh = builtinLayout(action.id);
      return fresh ? mapLayout(state, action.id, () => fresh) : state;
    }
    case "updatePanels":
      return mapLayout(state, state.active_layout_id, (l) => ({ ...l, panels: normalizePanels(action.panels) }));
    case "toggleNotes":
      return mapLayout(state, state.active_layout_id, (l) => {
        if (l.panels.some((p) => p.kind === "notes")) {
          // Kaldırırken diğer paneller yerinde kalır (kullanıcının boşlukları korunur).
          return { ...l, panels: l.panels.filter((p) => p.kind !== "notes") };
        }
        const notes: Panel = { id: action.panelId, kind: "notes", ...NOTES_PANEL, settings: {} };
        // Notlar sağ üste sabitlenir; çakışan paneller aşağı itilir.
        return { ...l, panels: movePanel([...l.panels, notes], notes.id, NOTES_PANEL.x, NOTES_PANEL.y) };
      });
  }
}
```

`app/ui/layout/store.tsx`:

```tsx
// Düzen durumu için React bağlamı: reducer, 500 ms debounce ile kayıt (v1 spec §4) ve
// Cmd+1…9 ile sıradaki düzene geçiş (spec §3.2).
import { createContext, useContext, useEffect, useMemo, useReducer, useRef, type Dispatch, type ReactNode } from "react";
import { parseLayoutState, type Layout, type LayoutState, type Panel } from "./model";
import type { LayoutPersistence } from "./persistence";
import { activeLayout, layoutReducer, newId, normalizeState, type LayoutAction } from "./reducer";

export const SAVE_DEBOUNCE_MS = 500;

export interface LayoutStore {
  state: LayoutState;
  active: Layout;
  dispatch: Dispatch<LayoutAction>;
  select(id: string): void;
  saveAs(name: string): void;
  rename(id: string, name: string): void;
  duplicate(sourceId: string, name: string): void;
  remove(id: string): void;
  resetBuiltin(id: string): void;
  updatePanels(panels: Panel[]): void;
  toggleNotes(): void;
}

const LayoutContext = createContext<LayoutStore | null>(null);

export function LayoutProvider({
  persistence,
  children,
  debounceMs = SAVE_DEBOUNCE_MS,
}: {
  persistence: LayoutPersistence;
  children: ReactNode;
  debounceMs?: number;
}) {
  const [state, dispatch] = useReducer(layoutReducer, persistence, (p) => normalizeState(parseLayoutState(p.load())));

  // İlk durum zaten kayıtlıdır; yalnızca değişiklikler kaydedilir.
  const initial = useRef(state);
  const pending = useRef<LayoutState | null>(null);
  useEffect(() => {
    if (state === initial.current) return;
    pending.current = state;
    const timer = setTimeout(() => {
      if (pending.current !== state) return; // pagehide ile zaten yazıldı
      pending.current = null;
      persistence.save(state);
    }, debounceMs);
    return () => clearTimeout(timer);
  }, [state, persistence, debounceMs]);

  // Pencere kapanırken bekleyen kayıt hemen yazılır.
  useEffect(() => {
    const flush = () => {
      if (pending.current) {
        persistence.save(pending.current);
        pending.current = null;
      }
    };
    window.addEventListener("pagehide", flush);
    return () => {
      window.removeEventListener("pagehide", flush);
      flush();
    };
  }, [persistence]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!e.metaKey || e.shiftKey || e.altKey || e.ctrlKey || !/^[1-9]$/.test(e.key)) return;
      e.preventDefault();
      dispatch({ type: "selectIndex", index: Number(e.key) - 1 });
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const store = useMemo<LayoutStore>(
    () => ({
      state,
      active: activeLayout(state),
      dispatch,
      select: (id) => dispatch({ type: "select", id }),
      saveAs: (name) => dispatch({ type: "saveAs", id: newId("user"), name }),
      rename: (id, name) => dispatch({ type: "rename", id, name }),
      duplicate: (sourceId, name) => dispatch({ type: "duplicate", sourceId, id: newId("user"), name }),
      remove: (id) => dispatch({ type: "delete", id }),
      resetBuiltin: (id) => dispatch({ type: "resetBuiltin", id }),
      updatePanels: (panels) => dispatch({ type: "updatePanels", panels }),
      toggleNotes: () => dispatch({ type: "toggleNotes", panelId: newId("notes") }),
    }),
    [state],
  );
  return <LayoutContext.Provider value={store}>{children}</LayoutContext.Provider>;
}

export function useLayoutStore(): LayoutStore {
  const store = useContext(LayoutContext);
  if (store === null) throw new Error("useLayoutStore must be used inside <LayoutProvider>");
  return store;
}
```

- [ ] **Step 4: Testleri ve derlemeyi çalıştır**

Komut: `cd app && pnpm vitest run ui/layout && pnpm test && pnpm build`
Beklenen:
- `ui/layout`: 57 test geçer.
- `pnpm test`: `Tests  80 passed (80)`.
- `pnpm build`: `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/layout/persistence.ts app/ui/layout/reducer.ts app/ui/layout/store.tsx app/ui/layout/reducer.test.ts app/ui/layout/store.test.tsx
git commit -m "feat(ui): layout store with debounced persistence (interim localStorage) and Cmd+1-9" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Harita çizimi: Kuzey Hint, Güney Hint, daire

**Files:**
- Create: `app/ui/chart/types.ts`, `app/ui/chart/adapter.ts`, `app/ui/chart/geometry.ts`, `app/ui/chart/ChartSvg.tsx`, `app/ui/chart/chart.css`, `app/ui/highlight/target.ts`
- Create/Test: `app/ui/chart/adapter.test.ts`, `app/ui/chart/geometry.test.ts`, `app/ui/chart/ChartSvg.test.tsx`

**Interfaces:**
- Consumes: `PlanetRow` (`app/ui/bindings.ts`: `{ body: string; lon: number | null; retrograde: boolean }`). Backend gövde adları (`app/src-tauri/src/commands.rs`, `GRAHAS` ve Ketu): `"Sun" "Moon" "Mars" "Mercury" "Jupiter" "Venus" "Saturn" "Rahu" "Ketu"`.
- Produces:
  - `types.ts`:
    - `GRAHA_IDS` (yukarıdaki dokuz ad), `type GrahaId`, `isGrahaId(v: string): v is GrahaId`
    - `interface ChartPlanet { id: GrahaId; sign: number; degInSign: number; retrograde: boolean; combust?: boolean; vargottama?: boolean }`
    - `interface ChartData { ascSign: number | null; planets: ChartPlanet[] }`
    - `type ChartStyle = "north" | "south" | "circle"`, `CHART_STYLES`, `isChartStyle(v: unknown): v is ChartStyle`
    - `interface ChartLabels { title: string; graha: Record<GrahaId, string>; signs: readonly string[]; lagna: string; house: (house: number, sign: string) => string }`
  - `adapter.ts`: `signAndDegree(lon: number): { sign: number; degInSign: number }`, `chartDataFromPlanetRows(rows: readonly PlanetRow[]): ChartData` (`ascSign: null`; `lon === null` ve bilinmeyen gövdeler atlanır).
  - `geometry.ts`:
    - `interface Point { x; y }`, `interface StackRule { axis: "x"|"y"; step; perLine; lineGap }`
    - `interface ChartCell { house; sign; points: Point[]; planetAnchor: Point; signAnchor: Point | null; lagnaAnchor: Point | null; stack: StackRule }`
    - `VIEWBOX = 100`
    - `houseSign(asc, house)`, `signHouse(asc, sign)`
    - `northCells(asc)`, `southCells(asc)`, `circleCells(asc)`, `chartCells(style, asc): ChartCell[]` (ev sırasıyla 1…12)
    - `SOUTH_SIGN_CELLS`
    - `stackPositions(anchor, n, rule): Point[]`, `pathOf(points): string`, `pointInPolygon(p, poly): boolean`
  - `ChartSvg.tsx`: `<ChartSvg data style labels highlight? onHover? onPin? />`. `role="img"` ve `aria-label=labels.title`.
    - Her gezegen `role="button"`, `tabIndex=0`, `aria-pressed` ve `aria-label="<kısaltma>[ ℞][ c] <derece>°"` taşır. `data-graha`, `data-house`, `data-x`, `data-y` öznitelikleri vardır.
    - Ev yolları `path.chart-cell`'dir. Lagna evi `chart-cell--lagna`, vurgulanan ev ya da burç `chart-cell--highlight`, vurgulanan gezegen `chart-planet--highlight` sınıfını alır.
  - `highlight/target.ts`: `type HighlightTarget = {type:"graha"; id: GrahaId} | {type:"house"; house: number} | {type:"sign"; sign: number}`, `sameTarget(a, b): boolean`.
- Geometri kuralları (100 × 100 kutu):
  - **Kuzey Hint:** dış kare, iki köşegen ve kenar ortalarını birleştiren eşkenar dörtgen.
    - 1. ev üstteki baklavadır; evler saat yönünün tersine ilerler (2. ev sol üst üçgen).
    - Her evde burç numarası (1–12) yazılır.
  - **Güney Hint:** 4 × 4 ızgaranın dış halkası. Burçlar sabittir: Meena sol üstte, saat yönünde. Lagna burcunun hücresinde `labels.lagna` işareti bulunur.
  - **Daire:** 1. ev solda (doğu). Evler saat yönünün tersine 30°'lik halka dilimleridir. 1. evde `labels.lagna` işareti bulunur.
  - Aynı evdeki gezegenler `stackPositions` ile dizilir. Üst ve alt üçgenler yatay dizer, diğerleri dikey. Bir satıra sığmayanlar yeni satıra geçer.
  - `ascSign === null` iken 1. ev Koç (0) çizilir. Notu çağıran gösterir.

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/chart/adapter.test.ts`:

```ts
import { chartDataFromPlanetRows, signAndDegree } from "./adapter";

test("sign is floor(lon / 30) and degInSign the remainder", () => {
  expect(signAndDegree(0)).toEqual({ sign: 0, degInSign: 0 });
  expect(signAndDegree(256.5)).toEqual({ sign: 8, degInSign: 16.5 });
  expect(signAndDegree(359.75)).toEqual({ sign: 11, degInSign: 29.75 });
  expect(signAndDegree(360)).toEqual({ sign: 0, degInSign: 0 });
  expect(signAndDegree(-30)).toEqual({ sign: 11, degInSign: 0 });
});

test("rows map to chart planets; null longitudes and unknown bodies are skipped", () => {
  const data = chartDataFromPlanetRows([
    { body: "Sun", lon: 256.5, retrograde: false },
    { body: "Saturn", lon: 10.25, retrograde: true },
    { body: "Mars", lon: null, retrograde: false },
    { body: "Pluto", lon: 100, retrograde: false },
    { body: "Rahu", lon: 100, retrograde: true },
    { body: "Ketu", lon: 280, retrograde: true },
  ]);
  expect(data.ascSign).toBeNull();
  expect(data.planets).toEqual([
    { id: "Sun", sign: 8, degInSign: 16.5, retrograde: false },
    { id: "Saturn", sign: 0, degInSign: 10.25, retrograde: true },
    { id: "Rahu", sign: 3, degInSign: 10, retrograde: true },
    { id: "Ketu", sign: 9, degInSign: 10, retrograde: true },
  ]);
});
```

`app/ui/chart/geometry.test.ts`:

```ts
import {
  chartCells,
  circleCells,
  houseSign,
  northCells,
  pointInPolygon,
  signHouse,
  southCells,
  stackPositions,
  type Point,
} from "./geometry";
import type { ChartStyle } from "./types";

const STYLES: ChartStyle[] = ["north", "south", "circle"];

function polygonArea(points: Point[]): number {
  let a = 0;
  for (let i = 0, j = points.length - 1; i < points.length; j = i++) a += (points[j].x + points[i].x) * (points[j].y - points[i].y);
  return Math.abs(a / 2);
}

test("house ↔ sign mapping is whole-sign from the ascendant", () => {
  expect(houseSign(0, 1)).toBe(0);
  expect(houseSign(4, 1)).toBe(4);
  expect(houseSign(10, 5)).toBe(2);
  for (let asc = 0; asc < 12; asc++) for (let h = 1; h <= 12; h++) expect(signHouse(asc, houseSign(asc, h))).toBe(h);
});

test.each(STYLES)("%s: 12 houses, each sign exactly once, house 1 holds the ascendant", (style) => {
  for (const asc of [0, 4, 11]) {
    const cells = chartCells(style, asc);
    expect(cells.map((c) => c.house)).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    expect(new Set(cells.map((c) => c.sign)).size).toBe(12);
    expect(cells[0].sign).toBe(asc);
    for (const c of cells) expect(c.sign).toBe(houseSign(asc, c.house));
  }
});

test.each(STYLES)("%s: every anchor lies inside its own cell", (style) => {
  for (const c of chartCells(style, 3)) {
    expect(pointInPolygon(c.planetAnchor, c.points), `house ${c.house} planet`).toBe(true);
    if (c.signAnchor) expect(pointInPolygon(c.signAnchor, c.points), `house ${c.house} sign`).toBe(true);
    if (c.lagnaAnchor) expect(pointInPolygon(c.lagnaAnchor, c.points), `house ${c.house} lagna`).toBe(true);
  }
});

test("north: house 1 is the top diamond and houses run counter-clockwise", () => {
  const cells = northCells(0);
  const centroid = (h: number) => {
    const pts = cells[h - 1].points;
    return { x: pts.reduce((s, p) => s + p.x, 0) / pts.length, y: pts.reduce((s, p) => s + p.y, 0) / pts.length };
  };
  expect(centroid(1)).toEqual({ x: 50, y: 25 });
  expect(centroid(4)).toEqual({ x: 25, y: 50 });
  expect(centroid(7)).toEqual({ x: 50, y: 75 });
  expect(centroid(10)).toEqual({ x: 75, y: 50 });
  expect(centroid(2).x).toBeLessThan(50); // sola doğru
  expect(centroid(12).x).toBeGreaterThan(50);
  // Dört baklava ve sekiz üçgen kareyi tam kaplar.
  const total = cells.reduce((s, c) => s + polygonArea(c.points), 0);
  expect(total).toBeCloseTo(100 * 100, 6);
  expect(cells.filter((c) => c.points.length === 4)).toHaveLength(4);
  expect(cells.every((c) => c.signAnchor !== null && c.lagnaAnchor === null)).toBe(true);
});

test("south: signs are fixed with Pisces top-left; the lagna marker follows the ascendant", () => {
  const cells = southCells(4);
  const pisces = cells.find((c) => c.sign === 11)!;
  expect(pisces.points[0]).toEqual({ x: 0, y: 0 });
  const aries = cells.find((c) => c.sign === 0)!;
  expect(aries.points[0]).toEqual({ x: 25, y: 0 });
  const leo = cells.find((c) => c.sign === 4)!;
  expect(leo.house).toBe(1);
  expect(leo.lagnaAnchor).not.toBeNull();
  expect(cells.filter((c) => c.lagnaAnchor !== null)).toHaveLength(1);
  // Sabit burç yerleri lagnaya göre değişmez.
  expect(southCells(0).find((c) => c.sign === 4)!.points).toEqual(leo.points);
});

test("circle: house 1 sits on the left and houses run counter-clockwise", () => {
  const cells = circleCells(0);
  expect(cells[0].planetAnchor.x).toBeLessThan(50);
  expect(cells[0].planetAnchor.y).toBeCloseTo(50, 6);
  // 2. ev ekranda sol altta (y aşağı büyür).
  expect(cells[1].planetAnchor.y).toBeGreaterThan(50);
  expect(cells[3].planetAnchor.y).toBeGreaterThan(cells[1].planetAnchor.y);
});

test("stackPositions centres on the anchor and never puts two labels on the same spot", () => {
  const rule = { axis: "y" as const, step: 5, perLine: 3, lineGap: 9 };
  expect(stackPositions({ x: 10, y: 10 }, 1, rule)).toEqual([{ x: 10, y: 10 }]);
  expect(stackPositions({ x: 10, y: 10 }, 2, rule)).toEqual([
    { x: 10, y: 7.5 },
    { x: 10, y: 12.5 },
  ]);
  const nine = stackPositions({ x: 50, y: 50 }, 9, rule);
  const keys = new Set(nine.map((p) => `${p.x},${p.y}`));
  expect(keys.size).toBe(9);
  for (let i = 0; i < nine.length; i++)
    for (let j = i + 1; j < nine.length; j++) {
      const d = Math.hypot(nine[i].x - nine[j].x, nine[i].y - nine[j].y);
      expect(d).toBeGreaterThanOrEqual(5);
    }
});

test.each(STYLES)("%s: up to three planets in a house stay inside it", (style) => {
  for (const c of chartCells(style, 0)) {
    for (const p of stackPositions(c.planetAnchor, 3, c.stack)) {
      expect(pointInPolygon(p, c.points), `house ${c.house}`).toBe(true);
    }
  }
});
```

`app/ui/chart/ChartSvg.test.tsx`:

```tsx
import { fireEvent, render, screen } from "@testing-library/react";
import { ChartSvg } from "./ChartSvg";
import type { ChartData, ChartLabels, ChartStyle } from "./types";

const labels: ChartLabels = {
  title: "Chart",
  graha: { Sun: "Su", Moon: "Mo", Mars: "Ma", Mercury: "Me", Jupiter: "Ju", Venus: "Ve", Saturn: "Sa", Rahu: "Ra", Ketu: "Ke" },
  signs: ["Ar", "Ta", "Ge", "Cn", "Le", "Vi", "Li", "Sc", "Sg", "Cp", "Aq", "Pi"],
  lagna: "As",
  house: (house, sign) => `House ${house}, ${sign}`,
};

const data: ChartData = {
  ascSign: 4,
  planets: [
    { id: "Sun", sign: 4, degInSign: 12.5, retrograde: false },
    { id: "Moon", sign: 4, degInSign: 3, retrograde: false, vargottama: true },
    { id: "Mercury", sign: 4, degInSign: 20, retrograde: true, combust: true },
    { id: "Saturn", sign: 10, degInSign: 1, retrograde: true },
  ],
};

const STYLES: ChartStyle[] = ["north", "south", "circle"];

test.each(STYLES)("%s renders an accessible image with 12 houses", (style) => {
  const { container } = render(<ChartSvg data={data} style={style} labels={labels} />);
  expect(screen.getByRole("img", { name: "Chart" })).toBeInTheDocument();
  expect(container.querySelectorAll("path.chart-cell")).toHaveLength(12);
  expect(container.querySelector("svg")!.dataset.style).toBe(style);
});

test.each(STYLES)("%s: planets are placed in the house of their sign; lagna house is filled", (style) => {
  const { container } = render(<ChartSvg data={data} style={style} labels={labels} />);
  const houseOf = (id: string) => container.querySelector(`[data-graha="${id}"]`)!.getAttribute("data-house");
  expect(houseOf("Sun")).toBe("1");
  expect(houseOf("Saturn")).toBe("7");
  const lagna = container.querySelectorAll("path.chart-cell--lagna");
  expect(lagna).toHaveLength(1);
  expect(lagna[0].getAttribute("aria-label")).toBe("House 1, Le");
});

test("north shows sign numbers; south and circle show the lagna marker instead", () => {
  const north = render(<ChartSvg data={data} style="north" labels={labels} />);
  const house1 = north.container.querySelector('g[data-house="1"]')!;
  expect(house1.querySelector("text.chart-sign")).toHaveTextContent("5");
  expect(north.queryByText("As")).toBeNull();
  north.unmount();
  for (const style of ["south", "circle"] as const) {
    const r = render(<ChartSvg data={data} style={style} labels={labels} />);
    expect(r.getAllByText("As")).toHaveLength(1);
    r.unmount();
  }
});

test("markers: retro ℞, combust c, vargottama underline — no colour coding", () => {
  render(<ChartSvg data={data} style="north" labels={labels} />);
  const mercury = screen.getByRole("button", { name: "Me ℞ c 20°" });
  expect(mercury).toHaveTextContent("Me℞c");
  const moon = screen.getByRole("button", { name: "Mo 3°" });
  expect(moon.querySelector("text")).toHaveAttribute("text-decoration", "underline");
  expect(screen.getByRole("button", { name: "Su 12°" }).querySelector("text")).not.toHaveAttribute("text-decoration");
  for (const el of document.querySelectorAll("svg *")) {
    expect(el.getAttribute("fill") ?? "").not.toMatch(/#|rgb/);
    expect(el.getAttribute("stroke") ?? "").not.toMatch(/#|rgb/);
  }
});

test("several planets in one house get distinct label positions", () => {
  const { container } = render(<ChartSvg data={data} style="north" labels={labels} />);
  const spots = [...container.querySelectorAll('[data-graha][data-house="1"]')].map(
    (g) => `${g.getAttribute("data-x")},${g.getAttribute("data-y")}`,
  );
  expect(spots).toHaveLength(3);
  expect(new Set(spots).size).toBe(3);
});

test("null ascendant draws house 1 as Aries", () => {
  const { container } = render(<ChartSvg data={{ ...data, ascSign: null }} style="north" labels={labels} />);
  expect(container.querySelector('g[data-house="1"]')!.getAttribute("data-sign")).toBe("0");
  expect(container.querySelector('[data-graha="Saturn"]')!.getAttribute("data-house")).toBe("11");
});

test("hover, focus, click and keyboard report highlight targets", () => {
  const onHover = vi.fn();
  const onPin = vi.fn();
  const { container } = render(<ChartSvg data={data} style="north" labels={labels} onHover={onHover} onPin={onPin} />);
  const sun = screen.getByRole("button", { name: "Su 12°" });
  fireEvent.mouseEnter(sun);
  expect(onHover).toHaveBeenLastCalledWith({ type: "graha", id: "Sun" });
  fireEvent.mouseLeave(sun);
  expect(onHover).toHaveBeenLastCalledWith(null);
  fireEvent.click(sun);
  expect(onPin).toHaveBeenLastCalledWith({ type: "graha", id: "Sun" });
  fireEvent.keyDown(sun, { key: "Enter" });
  expect(onPin).toHaveBeenCalledTimes(2);
  fireEvent.mouseEnter(container.querySelector('g[data-house="3"] path')!);
  expect(onHover).toHaveBeenLastCalledWith({ type: "house", house: 3 });
});

test("highlighted graha and house get highlight classes", () => {
  const { container, rerender } = render(
    <ChartSvg data={data} style="north" labels={labels} highlight={{ type: "graha", id: "Saturn" }} />,
  );
  expect(container.querySelector('[data-graha="Saturn"]')).toHaveClass("chart-planet--highlight");
  expect(screen.getByRole("button", { name: "Sa ℞ 1°" })).toHaveAttribute("aria-pressed", "true");
  rerender(<ChartSvg data={data} style="north" labels={labels} highlight={{ type: "sign", sign: 10 }} />);
  expect(container.querySelector('g[data-house="7"] path')).toHaveClass("chart-cell--highlight");
});
```

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

Komut: `cd app && pnpm vitest run ui/chart`
Beklenen: FAIL, `Failed to resolve import "./adapter"`, `"./geometry"`, `"./ChartSvg"`.

- [ ] **Step 3: Tipleri, dönüştürücüyü ve vurgu hedefini yaz**

`app/ui/chart/types.ts`:

```ts
// Harita çiziminin girdisi. Değerler Rust'tan hazır gelir; arayüz yalnızca çizer.

/** Backend'in `PlanetRow.body` değerleriyle birebir aynı (app/src-tauri/src/commands.rs). */
export const GRAHA_IDS = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"] as const;
export type GrahaId = (typeof GRAHA_IDS)[number];

export function isGrahaId(v: string): v is GrahaId {
  return (GRAHA_IDS as readonly string[]).includes(v);
}

export interface ChartPlanet {
  id: GrahaId;
  /** 0 = Mesha (Koç) … 11 = Meena (Balık). */
  sign: number;
  /** Burç içi derece, [0, 30). */
  degInSign: number;
  retrograde: boolean;
  combust?: boolean;
  vargottama?: boolean;
}

export interface ChartData {
  /** Lagna burcu; henüz hesaplanmıyorsa `null` (1. ev Koç kabul edilir, çağıran uyarı gösterir). */
  ascSign: number | null;
  planets: ChartPlanet[];
}

export type ChartStyle = "north" | "south" | "circle";
export const CHART_STYLES: readonly ChartStyle[] = ["north", "south", "circle"];

export function isChartStyle(v: unknown): v is ChartStyle {
  return typeof v === "string" && (CHART_STYLES as readonly string[]).includes(v);
}

/** Çizimde kullanılan metinler; çağıran i18n'den doldurur. */
export interface ChartLabels {
  /** Erişilebilir ad (svg `aria-label`). */
  title: string;
  /** Graha kısaltmaları. */
  graha: Record<GrahaId, string>;
  /** 12 burç kısaltması, 0 = Mesha. */
  signs: readonly string[];
  /** Güney Hint ve daire stilinde lagna hücresindeki işaret. */
  lagna: string;
  /** Ev hücresinin erişilebilir adı. */
  house: (house: number, sign: string) => string;
}
```

`app/ui/chart/adapter.ts`:

```ts
// PlanetRow → ChartData. Yalnızca gösterim eşlemesi yapılır (burç = floor(lon / 30),
// burç içi derece); astrolojik hesap değildir.
import type { PlanetRow } from "../bindings";
import { isGrahaId, type ChartData } from "./types";

export function signAndDegree(lon: number): { sign: number; degInSign: number } {
  const norm = ((lon % 360) + 360) % 360;
  const sign = Math.floor(norm / 30);
  return { sign, degInSign: norm - sign * 30 };
}

/** Lagna henüz backend'den gelmediği için `ascSign` her zaman `null`. */
export function chartDataFromPlanetRows(rows: readonly PlanetRow[]): ChartData {
  const planets: ChartData["planets"] = [];
  for (const r of rows) {
    if (r.lon === null || !isGrahaId(r.body)) continue;
    planets.push({ id: r.body, ...signAndDegree(r.lon), retrograde: r.retrograde });
  }
  return { ascSign: null, planets };
}
```

`app/ui/highlight/target.ts`:

```ts
// Paneller arası vurgu hedefi (spec §3.4).
import type { GrahaId } from "../chart/types";

export type HighlightTarget =
  | { type: "graha"; id: GrahaId }
  | { type: "house"; house: number }
  | { type: "sign"; sign: number };

export function sameTarget(a: HighlightTarget | null, b: HighlightTarget | null): boolean {
  if (a === null || b === null) return a === b;
  if (a.type === "graha" && b.type === "graha") return a.id === b.id;
  if (a.type === "house" && b.type === "house") return a.house === b.house;
  if (a.type === "sign" && b.type === "sign") return a.sign === b.sign;
  return false;
}
```

- [ ] **Step 4: Geometriyi yaz**

`app/ui/chart/geometry.ts`:

```ts
// Üç harita stili için saf geometri. Koordinatlar 100 × 100 kutudadır (svg viewBox).
// Evler bütün burç (whole sign) kabulüyle burçlara eşlenir: ev h → (lagna + h − 1) mod 12.
import type { ChartStyle } from "./types";

export interface Point {
  x: number;
  y: number;
}

export interface StackRule {
  /** Birden çok gezegen bu eksen boyunca dizilir. */
  axis: "x" | "y";
  step: number;
  perLine: number;
  lineGap: number;
}

export interface ChartCell {
  house: number; // 1…12
  sign: number; // 0…11
  points: Point[];
  planetAnchor: Point;
  /** Burç numarasının yeri (yalnızca Kuzey Hint). */
  signAnchor: Point | null;
  /** Lagna işaretinin yeri (Güney Hint ve daire, yalnızca 1. ev). */
  lagnaAnchor: Point | null;
  stack: StackRule;
}

export const VIEWBOX = 100;

export function houseSign(asc: number, house: number): number {
  return (asc + house - 1) % 12;
}

export function signHouse(asc: number, sign: number): number {
  return ((sign - asc + 12) % 12) + 1;
}

const pt = (x: number, y: number): Point => ({ x, y });

// Kuzey Hint: dış kare, iki köşegen ve kenar ortalarını birleştiren eşkenar dörtgen.
// 1. ev üstteki baklava; evler saat yönünün tersine ilerler.
const TL = pt(0, 0), TR = pt(100, 0), BR = pt(100, 100), BL = pt(0, 100);
const T = pt(50, 0), R = pt(100, 50), B = pt(50, 100), L = pt(0, 50), C = pt(50, 50);
const PTL = pt(25, 25), PTR = pt(75, 25), PBR = pt(75, 75), PBL = pt(25, 75);

// Baklavalar ve yan üçgenler dikey, üst/alt üçgenler yatay dizer.
const VERTICAL: StackRule = { axis: "y", step: 5.5, perLine: 4, lineGap: 9 };
const HORIZONTAL: StackRule = { axis: "x", step: 7.5, perLine: 4, lineGap: 5.5 };

const NORTH: ReadonlyArray<{ points: Point[]; planet: Point; sign: Point; stack: StackRule }> = [
  { points: [T, PTR, C, PTL], planet: pt(50, 22), sign: pt(50, 42), stack: VERTICAL }, // 1
  { points: [TL, T, PTL], planet: pt(25, 7), sign: pt(25, 19), stack: HORIZONTAL }, // 2
  { points: [TL, PTL, L], planet: pt(7, 25), sign: pt(19, 25), stack: VERTICAL }, // 3
  { points: [L, PTL, C, PBL], planet: pt(22, 50), sign: pt(42, 50), stack: VERTICAL }, // 4
  { points: [L, PBL, BL], planet: pt(7, 75), sign: pt(19, 75), stack: VERTICAL }, // 5
  { points: [BL, PBL, B], planet: pt(25, 93), sign: pt(25, 81), stack: HORIZONTAL }, // 6
  { points: [B, PBL, C, PBR], planet: pt(50, 78), sign: pt(50, 58), stack: VERTICAL }, // 7
  { points: [B, PBR, BR], planet: pt(75, 93), sign: pt(75, 81), stack: HORIZONTAL }, // 8
  { points: [BR, PBR, R], planet: pt(93, 75), sign: pt(81, 75), stack: VERTICAL }, // 9
  { points: [R, PBR, C, PTR], planet: pt(78, 50), sign: pt(58, 50), stack: VERTICAL }, // 10
  { points: [R, PTR, TR], planet: pt(93, 25), sign: pt(81, 25), stack: VERTICAL }, // 11
  { points: [TR, PTR, T], planet: pt(75, 7), sign: pt(75, 19), stack: HORIZONTAL }, // 12
];

export function northCells(asc: number): ChartCell[] {
  return NORTH.map((h, i) => ({
    house: i + 1,
    sign: houseSign(asc, i + 1),
    points: h.points,
    planetAnchor: h.planet,
    signAnchor: h.sign,
    lagnaAnchor: null,
    stack: h.stack,
  }));
}

/** Güney Hint: burçlar sabit, 4 × 4 ızgaranın dış halkası, Meena sol üstte, saat yönünde. */
export const SOUTH_SIGN_CELLS: ReadonlyArray<{ sign: number; row: number; col: number }> = [
  { sign: 11, row: 0, col: 0 },
  { sign: 0, row: 0, col: 1 },
  { sign: 1, row: 0, col: 2 },
  { sign: 2, row: 0, col: 3 },
  { sign: 3, row: 1, col: 3 },
  { sign: 4, row: 2, col: 3 },
  { sign: 5, row: 3, col: 3 },
  { sign: 6, row: 3, col: 2 },
  { sign: 7, row: 3, col: 1 },
  { sign: 8, row: 3, col: 0 },
  { sign: 9, row: 2, col: 0 },
  { sign: 10, row: 1, col: 0 },
];

const SOUTH_STACK: StackRule = { axis: "y", step: 5, perLine: 3, lineGap: 9 };

export function southCells(asc: number): ChartCell[] {
  const cells = SOUTH_SIGN_CELLS.map(({ sign, row, col }) => {
    const x = col * 25;
    const y = row * 25;
    const house = signHouse(asc, sign);
    return {
      house,
      sign,
      points: [pt(x, y), pt(x + 25, y), pt(x + 25, y + 25), pt(x, y + 25)],
      planetAnchor: pt(x + 12.5, y + 14),
      signAnchor: null,
      lagnaAnchor: house === 1 ? pt(x + 4, y + 6) : null,
      stack: SOUTH_STACK,
    };
  });
  return cells.sort((a, b) => a.house - b.house);
}

const CIRCLE_R_OUTER = 48;
const CIRCLE_R_INNER = 18;
const CIRCLE_STACK: StackRule = { axis: "y", step: 5, perLine: 3, lineGap: 7 };

function polar(r: number, deg: number): Point {
  const rad = (deg * Math.PI) / 180;
  // Matematiksel açı (saat yönünün tersi, y yukarı) → ekran (y aşağı).
  return pt(50 + r * Math.cos(rad), 50 - r * Math.sin(rad));
}

/** Daire: 1. ev solda (doğu ufku), evler saat yönünün tersine; her ev 30°'lik halka dilimi. */
export function circleCells(asc: number): ChartCell[] {
  const cells: ChartCell[] = [];
  for (let house = 1; house <= 12; house++) {
    const center = 180 + (house - 1) * 30;
    const start = center - 15;
    const outer: Point[] = [];
    const inner: Point[] = [];
    for (let i = 0; i <= 8; i++) {
      outer.push(polar(CIRCLE_R_OUTER, start + (30 * i) / 8));
      inner.push(polar(CIRCLE_R_INNER, start + 30 - (30 * i) / 8));
    }
    cells.push({
      house,
      sign: houseSign(asc, house),
      points: [...outer, ...inner],
      planetAnchor: polar((CIRCLE_R_OUTER + CIRCLE_R_INNER) / 2 + 2, center),
      signAnchor: null,
      lagnaAnchor: house === 1 ? polar(CIRCLE_R_INNER + 4, center) : null,
      stack: CIRCLE_STACK,
    });
  }
  return cells;
}

export function chartCells(style: ChartStyle, asc: number): ChartCell[] {
  if (style === "south") return southCells(asc);
  if (style === "circle") return circleCells(asc);
  return northCells(asc);
}

/** Aynı evdeki n gezegenin etiket konumları: kural eksenine dizilir, sığmazsa yeni satır. */
export function stackPositions(anchor: Point, n: number, rule: StackRule): Point[] {
  const lines = Math.ceil(n / rule.perLine);
  const out: Point[] = [];
  for (let i = 0; i < n; i++) {
    const line = Math.floor(i / rule.perLine);
    const idx = i % rule.perLine;
    const inLine = Math.min(rule.perLine, n - line * rule.perLine);
    const along = (idx - (inLine - 1) / 2) * rule.step;
    const across = (line - (lines - 1) / 2) * rule.lineGap;
    out.push(rule.axis === "y" ? pt(anchor.x + across, anchor.y + along) : pt(anchor.x + along, anchor.y + across));
  }
  return out;
}

export function pathOf(points: readonly Point[]): string {
  return points.map((p, i) => `${i === 0 ? "M" : "L"}${round(p.x)} ${round(p.y)}`).join(" ") + " Z";
}

function round(v: number): number {
  return Math.round(v * 100) / 100;
}

/** Işın atma ile nokta-çokgen testi (testlerde ve isabet denetiminde kullanılır). */
export function pointInPolygon(p: Point, poly: readonly Point[]): boolean {
  let inside = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const a = poly[i];
    const b = poly[j];
    if (a.y > p.y !== b.y > p.y && p.x < ((b.x - a.x) * (p.y - a.y)) / (b.y - a.y) + a.x) inside = !inside;
  }
  return inside;
}
```

- [ ] **Step 5: SVG bileşenini ve stilini yaz**

`app/ui/chart/chart.css`:

```css
.chart {
  display: block;
  width: 100%;
  height: 100%;
}
.chart-cell {
  fill: var(--panel);
  stroke: var(--ink);
  stroke-width: 0.4;
}
.chart-cell--lagna {
  fill: var(--select);
}
.chart-cell--highlight {
  fill: var(--select);
  stroke-width: 1;
}
.chart-text {
  fill: var(--ink-strong);
  font-size: 4.2px;
  font-weight: var(--fw-regular);
}
.chart-sign {
  fill: var(--ink);
  font-size: 3.4px;
}
.chart-planet {
  cursor: default;
}
.chart-planet:focus {
  outline: none;
}
.chart-planet:focus-visible .chart-text,
.chart-planet--highlight .chart-text {
  font-weight: var(--fw-medium);
}
.chart-planet--highlight .chart-planet-bg {
  fill: var(--select);
}
.chart-planet-bg {
  fill: transparent;
}
.chart-planet:focus-visible .chart-planet-bg {
  stroke: var(--ink);
  stroke-width: 0.5;
}
```

`app/ui/chart/ChartSvg.tsx`:

```tsx
// Kuzey Hint, Güney Hint ve daire haritası. Yalnızca belirteç sınıfları kullanılır;
// gezegenlere renk verilmez. Retro "℞", yanma "c", vargottama altı çizili (spec §2.3).
import type { KeyboardEvent } from "react";
import { sameTarget, type HighlightTarget } from "../highlight/target";
import "./chart.css";
import { chartCells, pathOf, stackPositions, VIEWBOX, type ChartCell } from "./geometry";
import type { ChartData, ChartLabels, ChartPlanet, ChartStyle } from "./types";

export interface ChartSvgProps {
  data: ChartData;
  style: ChartStyle;
  labels: ChartLabels;
  highlight?: HighlightTarget | null;
  onHover?: (target: HighlightTarget | null) => void;
  onPin?: (target: HighlightTarget) => void;
}

function cellHighlighted(cell: ChartCell, h: HighlightTarget | null): boolean {
  if (h === null) return false;
  return (h.type === "house" && h.house === cell.house) || (h.type === "sign" && h.sign === cell.sign);
}

export function ChartSvg({ data, style, labels, highlight = null, onHover, onPin }: ChartSvgProps) {
  const asc = data.ascSign ?? 0;
  const cells = chartCells(style, asc);
  const bySign = new Map<number, ChartPlanet[]>();
  for (const p of data.planets) bySign.set(p.sign, [...(bySign.get(p.sign) ?? []), p]);

  const activate = (target: HighlightTarget) => (e: KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onPin?.(target);
    }
  };

  return (
    <svg
      className="chart"
      data-style={style}
      viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`}
      role="img"
      aria-label={labels.title}
      xmlns="http://www.w3.org/2000/svg"
    >
      {cells.map((cell) => {
        const house: HighlightTarget = { type: "house", house: cell.house };
        const classes = ["chart-cell"];
        if (cell.house === 1) classes.push("chart-cell--lagna");
        if (cellHighlighted(cell, highlight)) classes.push("chart-cell--highlight");
        return (
          <g key={cell.house} data-house={cell.house} data-sign={cell.sign}>
            <path
              className={classes.join(" ")}
              d={pathOf(cell.points)}
              aria-label={labels.house(cell.house, labels.signs[cell.sign])}
              onMouseEnter={() => onHover?.(house)}
              onMouseLeave={() => onHover?.(null)}
              onClick={() => onPin?.(house)}
            />
            {cell.signAnchor && (
              <text className="chart-sign" x={cell.signAnchor.x} y={cell.signAnchor.y} textAnchor="middle" dominantBaseline="central">
                {cell.sign + 1}
              </text>
            )}
            {cell.lagnaAnchor && (
              <text className="chart-sign" x={cell.lagnaAnchor.x} y={cell.lagnaAnchor.y} textAnchor="middle" dominantBaseline="central">
                {labels.lagna}
              </text>
            )}
          </g>
        );
      })}
      {cells.map((cell) => {
        const planets = bySign.get(cell.sign) ?? [];
        const spots = stackPositions(cell.planetAnchor, planets.length, cell.stack);
        return planets.map((p, i) => {
          const target: HighlightTarget = { type: "graha", id: p.id };
          const on = sameTarget(highlight, target);
          const abbr = labels.graha[p.id];
          const marks = `${p.retrograde ? " ℞" : ""}${p.combust ? " c" : ""}`;
          const { x, y } = spots[i];
          return (
            <g
              key={p.id}
              className={`chart-planet${on ? " chart-planet--highlight" : ""}`}
              data-graha={p.id}
              data-house={cell.house}
              data-x={x}
              data-y={y}
              role="button"
              tabIndex={0}
              aria-pressed={on}
              aria-label={`${abbr}${marks} ${Math.floor(p.degInSign)}°`}
              onMouseEnter={() => onHover?.(target)}
              onMouseLeave={() => onHover?.(null)}
              onFocus={() => onHover?.(target)}
              onBlur={() => onHover?.(null)}
              onClick={() => onPin?.(target)}
              onKeyDown={activate(target)}
            >
              <rect className="chart-planet-bg" x={x - 4} y={y - 2.6} width={8} height={5.2} rx={1} />
              <text
                className="chart-text"
                x={x}
                y={y}
                textAnchor="middle"
                dominantBaseline="central"
                textDecoration={p.vargottama ? "underline" : undefined}
              >
                <tspan>{abbr}</tspan>
                {p.retrograde && <tspan dx={0.4}>℞</tspan>}
                {p.combust && <tspan dx={0.4}>c</tspan>}
              </text>
            </g>
          );
        });
      })}
    </svg>
  );
}
```

- [ ] **Step 6: Testleri ve derlemeyi çalıştır**

Komut: `cd app && pnpm vitest run ui/chart && pnpm test && pnpm build`
Beklenen:
- `ui/chart`: 28 test geçer.
- `pnpm test`: `Tests  108 passed (108)`. `no-hex.test.ts` yeni dosyaları da tarar.
- `pnpm build`: `✓ built`.

- [ ] **Step 7: Commit**

```bash
git add app/ui/chart app/ui/highlight/target.ts
git commit -m "feat(ui): North/South Indian and circle chart geometry with token-only SVG rendering" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: i18n, ortak vurgu, panel kaydı ve paneller

**Files:**
- Modify: `app/package.json`, `app/pnpm-lock.yaml` (`pnpm add`)
- Create: `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`, `app/ui/i18n/index.ts`
- Create: `app/ui/highlight/store.tsx`, `app/ui/time.ts`
- Create: `app/ui/panels/types.ts`, `app/ui/panels/registry.ts`, `app/ui/panels/sky.tsx`, `app/ui/panels/states.tsx`, `app/ui/panels/labels.ts`, `app/ui/panels/format.ts`
- Create: `app/ui/panels/ChartPanel.tsx`, `app/ui/panels/GrahaTablePanel.tsx`, `app/ui/panels/UnavailablePanel.tsx`, `app/ui/panels/panels.css`
- Modify: `app/ui/setupTests.ts`, `app/ui/App.tsx` (yalnızca `unixMsToJdUt` taşınır), `app/ui/App.test.tsx` (taşınan test çıkar)
- Create/Test: `app/ui/i18n/i18n.test.ts`, `app/ui/highlight/store.test.tsx`, `app/ui/time.test.ts`, `app/ui/panels/format.test.ts`, `app/ui/panels/states.test.tsx`, `app/ui/panels/panels.test.tsx`

**Interfaces:**
- Consumes:
  - `api.planetLongitudes(jdUt)` → `Result<PlanetRow[]>` (`app/ui/api.ts`), `CommandError { code: string; params: Record<string, string> }`.
  - `Panel`, `isKnownPanelKind`, `KnownPanelKind` (Task 2).
  - `ChartSvg`, `chartDataFromPlanetRows`, `signAndDegree`, `isChartStyle`, `isGrahaId`, `GRAHA_IDS`, `ChartLabels` (Task 5).
  - `HighlightTarget`, `sameTarget` (Task 5).
- Produces:
  - `i18n/index.ts`:
    - `i18n` (i18next örneği; eşzamanlı başlar, varsayılan `tr`)
    - `LANGUAGES = ["tr","en","fr"]`, `type Language`, `DEFAULT_LANGUAGE`, `LANGUAGE_STORAGE_KEY = "jyotish-desk.language.v1"`
    - `isLanguage`, `loadLanguage(storage?)`, `setLanguage(lng, storage?): Promise<void>` (kaydeder, `<html lang>`'i günceller), `currentLanguage()`
  - `i18n/tr.ts`: `tr` ve `type Messages`. `en: Messages` ve `fr: Messages` derleme anında aynı anahtarları taşır.
  - Anahtarlar (bu planın tüm metinleri):
    - `app.title`
    - `graha.<GrahaId>` (kısaltma), `grahaName.<GrahaId>` (tam ad), `sign.0…11` (burç kısaltması)
    - `chart.{title, lagna, house, noLagna, vargaUnavailable}`
    - `panel.{chart, graha_table, dasha_tree, dasha_strip, transit_table, notes, birth_info, unknown, loading, unavailable, unknownKind, retry, moveHint}`
    - `table.{graha, sign, degree, retro, empty}`
    - `errors.{ipc, calc, engine_stopped, data_files_missing, invalid_path, env_override, already_open, job_panicked, print_failed, unknown}`: backend hata kodlarının tamamı (`commands.rs`) ve `api.ts`'in `ipc` kodu.
    - `layout.{picker, interview, research, transit, actions, saveAs, rename, duplicate, delete, reset, name, save, cancel, copyName, confirmDelete, deleteNamed}`
    - `theme.{label, light, dark, system}`, `language.{label, tr, en, fr}`, `shell.{printSpike, backToDesk, print}`
  - Graha kısaltmaları (Sun…Ketu sırasıyla):
    - tr: `Gü Ay Ma Me Jü Ve Sa Ra Ke`
    - en: `Su Mo Ma Me Ju Ve Sa Ra Ke`
    - fr: `So Lu Ma Me Ju Ve Sa Ra Ke`
  - `highlight/store.tsx`:
    - `<HighlightProvider>`
    - `useHighlight(): Highlight`, alanlar: `active` (üzerine gelinen, yoksa sabitlenen), `pinned`, `hover(t | null)`, `togglePin(t)`, `clear()`
    - `Esc` ikisini de temizler. Durum oturumluktur, düzen durumunda tutulmaz.
  - `time.ts`: `unixMsToJdUt(ms: number): number` (App.tsx'ten taşındı).
  - `panels/types.ts`: `interface PanelProps { panel: Panel }`, `interface PanelDefinition { titleKey: string; subtitle?: (panel: Panel) => string | null; component: ComponentType<PanelProps> }`.
  - `panels/registry.ts`: `PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition>`, `panelDefinition(kind: string): PanelDefinition` (bilinmeyen tür → `panel.unknown` + `UnavailablePanel`).
  - `panels/sky.tsx`:
    - `type SkyState = {status:"loading"} | {status:"ok"; rows: PlanetRow[]} | {status:"error"; error: CommandError}`
    - `<SkyProvider now?>`, `useSky(): { state: SkyState; retry(): void }`
  - `panels/states.tsx`:
    - `type LoadPhase = "none" | "inline" | "skeleton"`, `INLINE_AFTER_MS = 100`, `SKELETON_AFTER_MS = 1000`
    - `PanelStatusContext` (panel çerçevesi başlıktaki göstergeyi bununla öğrenir), `usePanelLoadPhase(loading): LoadPhase`
    - `<PanelSkeleton />`, `<PanelError error onRetry />`
  - `panels/format.ts`: `formatDegree(deg: number): string` (`16.5` → `"16°30′"`).
  - `panels/labels.ts`: `grahaAbbreviations(t)`, `signAbbreviations(t)`, `chartLabels(t, varga): ChartLabels`.
  - Paneller:
    - `ChartPanel` ve `chartVarga(settings): string`.
    - `GrahaTablePanel`: sütunlar graha, burç, derece ve retro.
    - `UnavailablePanel`: bilinen tür → `panel.unavailable`, bilinmeyen tür → `panel.unknownKind`. Hata değildir, `role="alert"` taşımaz.

- [ ] **Step 1: Bağımlılıkları ekle**

Komut: `cd app && pnpm add i18next react-i18next`
Beklenen:
- `package.json` `dependencies` içinde `"i18next": "^27.x"` ve `"react-i18next": "^18.x"` (yürütme anındaki son kararlı sürüm).
- `pnpm-lock.yaml` güncellenir.
- Başka paket eklenmez (`git diff --stat app/package.json`: yalnızca iki satır).

> Kurulan i18next'in `init` seçeneklerinde eşzamanlı başlatma `initAsync: false`'tur (i18next ≥ 24). `node_modules/i18next/typescript/options.d.ts` içinde `initAsync` adını doğrula.

- [ ] **Step 2: Başarısız testleri yaz**

`app/ui/i18n/i18n.test.ts`:

```ts
import { en } from "./en";
import { fr } from "./fr";
import { currentLanguage, i18n, LANGUAGE_STORAGE_KEY, loadLanguage, setLanguage } from "./index";
import { tr } from "./tr";

type Tree = { [k: string]: string | Tree };

function flatten(tree: Tree, prefix = ""): Record<string, string> {
  const out: Record<string, string> = {};
  for (const [k, v] of Object.entries(tree)) {
    const key = prefix ? `${prefix}.${k}` : k;
    if (typeof v === "string") out[key] = v;
    else Object.assign(out, flatten(v, key));
  }
  return out;
}

const placeholders = (s: string) => [...s.matchAll(/{{\s*(\w+)\s*}}/g)].map((m) => m[1]).sort();
const ORDER = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"] as const;

test.each([
  ["en", en],
  ["fr", fr],
])("%s has exactly the Turkish keys, non-empty values and the same placeholders", (_, messages) => {
  const base = flatten(tr);
  const other = flatten(messages);
  expect(Object.keys(other).sort()).toEqual(Object.keys(base).sort());
  for (const [key, value] of Object.entries(other)) {
    expect(value.trim(), key).not.toBe("");
    expect(placeholders(value), key).toEqual(placeholders(base[key]));
  }
});

test("graha abbreviation sets per language", () => {
  expect(ORDER.map((g) => tr.graha[g])).toEqual(["Gü", "Ay", "Ma", "Me", "Jü", "Ve", "Sa", "Ra", "Ke"]);
  expect(ORDER.map((g) => en.graha[g])).toEqual(["Su", "Mo", "Ma", "Me", "Ju", "Ve", "Sa", "Ra", "Ke"]);
  expect(ORDER.map((g) => fr.graha[g])).toEqual(["So", "Lu", "Ma", "Me", "Ju", "Ve", "Sa", "Ra", "Ke"]);
});

test("each language has 12 distinct sign abbreviations", () => {
  for (const m of [tr, en, fr]) {
    const signs = Array.from({ length: 12 }, (_, i) => m.sign[String(i) as keyof typeof m.sign]);
    expect(new Set(signs).size).toBe(12);
  }
});

test("Turkish is the default and initialisation is synchronous", () => {
  expect(currentLanguage()).toBe("tr");
  expect(i18n.t("chart.noLagna")).toBe("Lagna henüz hesaplanmıyor; şimdiki gökyüzü, 1. ev Koç");
  expect(i18n.t("errors.calc", { detail: "x<y" })).toBe("Hesap tamamlanamadı (x<y).");
});

test("setLanguage switches strings, persists the choice and sets <html lang>", async () => {
  await setLanguage("fr");
  expect(i18n.t("graha.Sun")).toBe("So");
  expect(localStorage.getItem(LANGUAGE_STORAGE_KEY)).toBe("fr");
  expect(document.documentElement.lang).toBe("fr");
  expect(loadLanguage()).toBe("fr");
  localStorage.setItem(LANGUAGE_STORAGE_KEY, "de");
  expect(loadLanguage()).toBe("tr");
});
```

`app/ui/highlight/store.test.tsx`:

```tsx
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
```

`app/ui/time.test.ts`:

```ts
import { unixMsToJdUt } from "./time";

test("unixMsToJdUt maps the Unix epoch to JD 2440587.5", () => {
  expect(unixMsToJdUt(0)).toBe(2440587.5);
  expect(unixMsToJdUt(86_400_000)).toBe(2440588.5);
});
```

`app/ui/panels/format.test.ts`:

```ts
import { formatDegree } from "./format";

test("formatDegree writes degrees and zero-padded arc minutes, rounding down", () => {
  expect(formatDegree(0)).toBe("0°00′");
  expect(formatDegree(16.5)).toBe("16°30′");
  expect(formatDegree(10.25)).toBe("10°15′");
  expect(formatDegree(29.9999)).toBe("29°59′");
  expect(formatDegree(3.0166666666666666)).toBe("3°01′");
});
```

`app/ui/panels/states.test.tsx`:

```tsx
import { act, fireEvent, render, screen } from "@testing-library/react";
import { PanelError, PanelStatusContext, usePanelLoadPhase, type LoadPhase } from "./states";

function Phase({ loading }: { loading: boolean }) {
  return <span data-testid="phase">{usePanelLoadPhase(loading)}</span>;
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

test("no indicator under 100 ms, inline until 1 s, skeleton after", () => {
  const reported: LoadPhase[] = [];
  const { rerender } = render(
    <PanelStatusContext.Provider value={(p) => reported.push(p)}>
      <Phase loading />
    </PanelStatusContext.Provider>,
  );
  const phase = () => screen.getByTestId("phase").textContent;
  act(() => vi.advanceTimersByTime(99));
  expect(phase()).toBe("none");
  act(() => vi.advanceTimersByTime(1));
  expect(phase()).toBe("inline");
  act(() => vi.advanceTimersByTime(899));
  expect(phase()).toBe("inline");
  act(() => vi.advanceTimersByTime(1));
  expect(phase()).toBe("skeleton");
  rerender(
    <PanelStatusContext.Provider value={(p) => reported.push(p)}>
      <Phase loading={false} />
    </PanelStatusContext.Provider>,
  );
  expect(phase()).toBe("none");
  expect(reported).toEqual(["none", "inline", "skeleton", "none"]);
});

test("a fast load never shows an indicator", () => {
  const { rerender } = render(<Phase loading />);
  act(() => vi.advanceTimersByTime(50));
  rerender(<Phase loading={false} />);
  act(() => vi.advanceTimersByTime(2000));
  expect(screen.getByTestId("phase")).toHaveTextContent("none");
});

test("PanelError translates {code, params} and offers a retry", () => {
  const onRetry = vi.fn();
  render(<PanelError error={{ code: "data_files_missing", params: { path: "/eph" } }} onRetry={onRetry} />);
  expect(screen.getByRole("alert")).toHaveTextContent("Ephemeris dosyaları bulunamadı: /eph");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  expect(onRetry).toHaveBeenCalledOnce();
});

test("PanelError falls back to a generic message for unknown codes", () => {
  render(<PanelError error={{ code: "weird", params: {} }} onRetry={() => {}} />);
  expect(screen.getByRole("alert")).toHaveTextContent("Beklenmeyen bir sorun oluştu (weird).");
});
```

`app/ui/panels/panels.test.tsx`:

```tsx
import { act, fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    planetLongitudes: vi.fn(async () => ({
      status: "ok",
      data: [
        { body: "Sun", lon: 256.5, retrograde: false },
        { body: "Saturn", lon: 10.25, retrograde: true },
        { body: "Mars", lon: null, retrograde: false },
      ],
    })),
  },
}));

import { api } from "../api";
import { HighlightProvider } from "../highlight/store";
import { setLanguage } from "../i18n";
import { tr } from "../i18n/tr";
import { PANEL_KINDS, type Panel } from "../layout/model";
import { ChartPanel } from "./ChartPanel";
import { GrahaTablePanel } from "./GrahaTablePanel";
import { panelDefinition, PANEL_REGISTRY } from "./registry";
import { SkyProvider } from "./sky";
import { UnavailablePanel } from "./UnavailablePanel";

const chartPanel = (settings: Record<string, unknown> = { varga: "D1", style: "north" }): Panel => ({
  id: "c",
  kind: "chart",
  x: 0,
  y: 0,
  w: 6,
  h: 12,
  settings,
});
const tablePanel: Panel = { id: "t", kind: "graha_table", x: 6, y: 0, w: 6, h: 8, settings: {} };

function Providers({ children }: { children: ReactNode }) {
  return (
    <HighlightProvider>
      <SkyProvider now={() => 0}>{children}</SkyProvider>
    </HighlightProvider>
  );
}

beforeEach(() => vi.mocked(api.planetLongitudes).mockClear());

test("registry covers the seven kinds; only chart and graha_table are implemented here", () => {
  expect(Object.keys(PANEL_REGISTRY).sort()).toEqual([...PANEL_KINDS].sort());
  expect(PANEL_REGISTRY.chart.component).toBe(ChartPanel);
  expect(PANEL_REGISTRY.graha_table.component).toBe(GrahaTablePanel);
  for (const kind of ["dasha_tree", "dasha_strip", "transit_table", "notes", "birth_info"] as const) {
    expect(PANEL_REGISTRY[kind].component).toBe(UnavailablePanel);
  }
  expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
  const keys = new Set(Object.keys(tr.panel).map((k) => `panel.${k}`));
  for (const kind of [...PANEL_KINDS, "yoga_list"]) expect(keys.has(panelDefinition(kind).titleKey)).toBe(true);
  expect(PANEL_REGISTRY.chart.subtitle!(chartPanel({ varga: "D9" }))).toBe("D9");
});

test("chart panel draws the current sky with the no-lagna notice, fetching once for jd(now)", async () => {
  render(
    <Providers>
      <ChartPanel panel={chartPanel()} />
      <GrahaTablePanel panel={tablePanel} />
    </Providers>,
  );
  expect(await screen.findByRole("note")).toHaveTextContent("Lagna henüz hesaplanmıyor; şimdiki gökyüzü, 1. ev Koç");
  expect(screen.getByRole("img", { name: "D1 haritası" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Gü 16°" })).toBeInTheDocument();
  expect(api.planetLongitudes).toHaveBeenCalledTimes(1);
  expect(api.planetLongitudes).toHaveBeenCalledWith(2440587.5);
});

test("chart panel honours the style setting and falls back to north", async () => {
  const { container, unmount } = render(
    <Providers>
      <ChartPanel panel={chartPanel({ varga: "D1", style: "south" })} />
    </Providers>,
  );
  await screen.findByRole("img");
  expect(container.querySelector("svg")!.dataset.style).toBe("south");
  unmount();
  const r = render(
    <Providers>
      <ChartPanel panel={chartPanel({ style: "spiral" })} />
    </Providers>,
  );
  await r.findByRole("img");
  expect(r.container.querySelector("svg")!.dataset.style).toBe("north");
});

test("vargas other than D1 show an explicit later-milestone message", () => {
  render(
    <Providers>
      <ChartPanel panel={chartPanel({ varga: "D9" })} />
    </Providers>,
  );
  expect(screen.getByText("D9 haritası sonraki bir sürümde gelecek.")).toBeInTheDocument();
  expect(screen.queryByRole("img")).toBeNull();
});

test("graha table lists body, sign, degree with minutes and the retro marker", async () => {
  render(
    <Providers>
      <GrahaTablePanel panel={tablePanel} />
    </Providers>,
  );
  const sun = (await screen.findByRole("button", { name: "Güneş" })).closest("tr")!;
  expect(sun).toHaveTextContent("Yay");
  expect(sun).toHaveTextContent("16°30′");
  const saturn = screen.getByRole("button", { name: "Satürn" }).closest("tr")!;
  expect(saturn).toHaveTextContent("Koç");
  expect(saturn).toHaveTextContent("10°15′");
  expect(saturn).toHaveTextContent("℞");
  const mars = screen.getByRole("button", { name: "Mars" }).closest("tr")!;
  expect(mars.querySelectorAll("td")[0]).toHaveTextContent("—");
});

test("hovering a graha in the table highlights it in the chart, click pins, Esc clears", async () => {
  const { container } = render(
    <Providers>
      <ChartPanel panel={chartPanel()} />
      <GrahaTablePanel panel={tablePanel} />
    </Providers>,
  );
  const row = (await screen.findByRole("button", { name: "Satürn" })).closest("tr")!;
  const chartSaturn = () => container.querySelector('svg [data-graha="Saturn"]')!;
  fireEvent.mouseEnter(row);
  expect(chartSaturn()).toHaveClass("chart-planet--highlight");
  expect(row).toHaveClass("is-highlighted");
  fireEvent.mouseLeave(row);
  expect(chartSaturn()).not.toHaveClass("chart-planet--highlight");

  fireEvent.click(chartSaturn());
  expect(row).toHaveClass("is-highlighted");
  expect(screen.getByRole("button", { name: "Satürn" })).toHaveAttribute("aria-pressed", "true");
  fireEvent.keyDown(window, { key: "Escape" });
  expect(row).not.toHaveClass("is-highlighted");
});

test("an empty result shows the empty state", async () => {
  vi.mocked(api.planetLongitudes).mockResolvedValueOnce({ status: "ok", data: [] });
  render(
    <Providers>
      <GrahaTablePanel panel={tablePanel} />
    </Providers>,
  );
  expect(await screen.findByText("Gösterilecek graha yok.")).toBeInTheDocument();
});

test("errors are shown in the panel and retry fetches again", async () => {
  vi.mocked(api.planetLongitudes).mockResolvedValueOnce({ status: "error", error: { code: "engine_stopped", params: {} } });
  render(
    <Providers>
      <GrahaTablePanel panel={tablePanel} />
    </Providers>,
  );
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap motoru durdu.");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  expect(await screen.findByRole("button", { name: "Güneş" })).toBeInTheDocument();
  expect(api.planetLongitudes).toHaveBeenCalledTimes(2);
});

test("a slow load shows a skeleton after one second", async () => {
  vi.useFakeTimers();
  vi.mocked(api.planetLongitudes).mockReturnValueOnce(new Promise(() => {}));
  render(
    <Providers>
      <ChartPanel panel={chartPanel()} />
    </Providers>,
  );
  expect(screen.queryByLabelText("Yükleniyor")).toBeNull();
  act(() => vi.advanceTimersByTime(1000));
  expect(screen.getByLabelText("Yükleniyor")).toHaveAttribute("aria-busy", "true");
  vi.useRealTimers();
});

test("unavailable and unknown panels explain themselves without an error", () => {
  render(<UnavailablePanel panel={{ ...tablePanel, kind: "dasha_tree" }} />);
  expect(screen.getByText("Bu panel sonraki bir sürümde gelecek.")).toBeInTheDocument();
  render(<UnavailablePanel panel={{ ...tablePanel, kind: "yoga_list" }} />);
  expect(screen.getByText("“yoga_list” paneli bu sürümde tanınmıyor. Düzen olduğu gibi korunuyor.")).toBeInTheDocument();
  expect(screen.queryByRole("alert")).toBeNull();
});

test("labels follow the selected language", async () => {
  await setLanguage("en");
  render(
    <Providers>
      <ChartPanel panel={chartPanel()} />
      <GrahaTablePanel panel={tablePanel} />
    </Providers>,
  );
  expect(await screen.findByRole("button", { name: "Su 16°" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Sun" }).closest("tr")).toHaveTextContent("Sg");
});
```

- [ ] **Step 3: Testlerin başarısız olduğunu doğrula**

Komut: `cd app && pnpm vitest run ui/i18n ui/highlight ui/time.test.ts ui/panels`
Beklenen: FAIL, `Failed to resolve import "./en"`, `"./store"`, `"./time"`, `"./format"`, `"./states"`, `"../highlight/store"`.

- [ ] **Step 4: i18n kaynaklarını ve kurulumu yaz**

`app/ui/i18n/tr.ts`:

```ts
// Türkçe (varsayılan dil). Anahtar yapısı en.ts ve fr.ts için tiptir (`Messages`).
export const tr = {
  app: { title: "Jyotish Desk" },
  graha: { Sun: "Gü", Moon: "Ay", Mars: "Ma", Mercury: "Me", Jupiter: "Jü", Venus: "Ve", Saturn: "Sa", Rahu: "Ra", Ketu: "Ke" },
  grahaName: {
    Sun: "Güneş",
    Moon: "Ay",
    Mars: "Mars",
    Mercury: "Merkür",
    Jupiter: "Jüpiter",
    Venus: "Venüs",
    Saturn: "Satürn",
    Rahu: "Rahu",
    Ketu: "Ketu",
  },
  sign: {
    "0": "Koç",
    "1": "Boğ",
    "2": "İki",
    "3": "Yen",
    "4": "Asl",
    "5": "Baş",
    "6": "Ter",
    "7": "Akr",
    "8": "Yay",
    "9": "Oğl",
    "10": "Kov",
    "11": "Bal",
  },
  chart: {
    title: "{{varga}} haritası",
    lagna: "Lg",
    house: "{{house}}. ev, {{sign}}",
    noLagna: "Lagna henüz hesaplanmıyor; şimdiki gökyüzü, 1. ev Koç",
    vargaUnavailable: "{{varga}} haritası sonraki bir sürümde gelecek.",
  },
  panel: {
    chart: "Harita",
    graha_table: "Graha tablosu",
    dasha_tree: "Dasa ağacı",
    dasha_strip: "Dasa şeridi",
    transit_table: "Transit tablosu",
    notes: "Notlar",
    birth_info: "Doğum bilgisi",
    unknown: "Bilinmeyen panel",
    loading: "Yükleniyor",
    unavailable: "Bu panel sonraki bir sürümde gelecek.",
    unknownKind: "“{{kind}}” paneli bu sürümde tanınmıyor. Düzen olduğu gibi korunuyor.",
    retry: "Yeniden dene",
    moveHint: "Taşımak için Alt+ok, boyutlandırmak için Alt+Shift+ok",
  },
  table: { graha: "Graha", sign: "Burç", degree: "Derece", retro: "Retro", empty: "Gösterilecek graha yok." },
  errors: {
    ipc: "Uygulama çekirdeğine ulaşılamadı.",
    calc: "Hesap tamamlanamadı ({{detail}}).",
    engine_stopped: "Hesap motoru durdu. Sorun sürerse uygulamayı yeniden başlatın.",
    data_files_missing: "Ephemeris dosyaları bulunamadı: {{path}}",
    invalid_path: "Ephemeris klasörünün yolu kullanılamıyor.",
    env_override: "Ephemeris yolu bir ortam değişkeniyle değiştirilmiş; bu desteklenmiyor.",
    already_open: "Hesap motoru zaten açık.",
    job_panicked: "Hesap beklenmedik biçimde durdu.",
    print_failed: "Yazdırma penceresi açılamadı.",
    unknown: "Beklenmeyen bir sorun oluştu ({{code}}).",
  },
  layout: {
    picker: "Düzen",
    interview: "Görüşme",
    research: "Araştırma",
    transit: "Transit",
    actions: "Düzen işlemleri",
    saveAs: "Farklı kaydet…",
    rename: "Yeniden adlandır…",
    duplicate: "Kopyala",
    delete: "Sil…",
    reset: "Varsayılana döndür",
    name: "Düzen adı",
    save: "Kaydet",
    cancel: "Vazgeç",
    copyName: "{{name}} kopyası",
    confirmDelete: "“{{name}}” düzeni silinsin mi? Bu işlem geri alınamaz.",
    deleteNamed: "“{{name}}” düzenini sil",
  },
  theme: { label: "Tema", light: "Açık", dark: "Koyu", system: "Sistem" },
  language: { label: "Dil", tr: "Türkçe", en: "English", fr: "Français" },
  shell: { printSpike: "Baskı denemesi", backToDesk: "Panoya dön", print: "Yazdır…" },
};

type Strings<T> = { [K in keyof T]: T[K] extends string ? string : Strings<T[K]> };
export type Messages = Strings<typeof tr>;
```

`app/ui/i18n/en.ts`:

```ts
import type { Messages } from "./tr";

export const en: Messages = {
  app: { title: "Jyotish Desk" },
  graha: { Sun: "Su", Moon: "Mo", Mars: "Ma", Mercury: "Me", Jupiter: "Ju", Venus: "Ve", Saturn: "Sa", Rahu: "Ra", Ketu: "Ke" },
  grahaName: {
    Sun: "Sun",
    Moon: "Moon",
    Mars: "Mars",
    Mercury: "Mercury",
    Jupiter: "Jupiter",
    Venus: "Venus",
    Saturn: "Saturn",
    Rahu: "Rahu",
    Ketu: "Ketu",
  },
  sign: {
    "0": "Ar",
    "1": "Ta",
    "2": "Ge",
    "3": "Cn",
    "4": "Le",
    "5": "Vi",
    "6": "Li",
    "7": "Sc",
    "8": "Sg",
    "9": "Cp",
    "10": "Aq",
    "11": "Pi",
  },
  chart: {
    title: "{{varga}} chart",
    lagna: "As",
    house: "House {{house}}, {{sign}}",
    noLagna: "Lagna is not computed yet; showing the current sky with house 1 = Aries",
    vargaUnavailable: "The {{varga}} chart arrives in a later version.",
  },
  panel: {
    chart: "Chart",
    graha_table: "Graha table",
    dasha_tree: "Dasha tree",
    dasha_strip: "Dasha strip",
    transit_table: "Transit table",
    notes: "Notes",
    birth_info: "Birth details",
    unknown: "Unknown panel",
    loading: "Loading",
    unavailable: "This panel arrives in a later version.",
    unknownKind: "This version does not recognise the “{{kind}}” panel. The layout is kept as it is.",
    retry: "Try again",
    moveHint: "Alt+arrow moves, Alt+Shift+arrow resizes",
  },
  table: { graha: "Graha", sign: "Sign", degree: "Degree", retro: "Retro", empty: "No grahas to show." },
  errors: {
    ipc: "Could not reach the application core.",
    calc: "The calculation could not finish ({{detail}}).",
    engine_stopped: "The calculation engine stopped. If this persists, restart the application.",
    data_files_missing: "Ephemeris files were not found: {{path}}",
    invalid_path: "The ephemeris folder path cannot be used.",
    env_override: "The ephemeris path was changed by an environment variable; this is not supported.",
    already_open: "The calculation engine is already open.",
    job_panicked: "The calculation stopped unexpectedly.",
    print_failed: "The print dialog could not be opened.",
    unknown: "Something unexpected happened ({{code}}).",
  },
  layout: {
    picker: "Layout",
    interview: "Consultation",
    research: "Research",
    transit: "Transit",
    actions: "Layout actions",
    saveAs: "Save as…",
    rename: "Rename…",
    duplicate: "Duplicate",
    delete: "Delete…",
    reset: "Reset to default",
    name: "Layout name",
    save: "Save",
    cancel: "Cancel",
    copyName: "{{name}} copy",
    confirmDelete: "Delete the “{{name}}” layout? This cannot be undone.",
    deleteNamed: "Delete “{{name}}”",
  },
  theme: { label: "Theme", light: "Light", dark: "Dark", system: "System" },
  language: { label: "Language", tr: "Türkçe", en: "English", fr: "Français" },
  shell: { printSpike: "Print test", backToDesk: "Back to desk", print: "Print…" },
};
```

`app/ui/i18n/fr.ts`:

```ts
import type { Messages } from "./tr";

export const fr: Messages = {
  app: { title: "Jyotish Desk" },
  graha: { Sun: "So", Moon: "Lu", Mars: "Ma", Mercury: "Me", Jupiter: "Ju", Venus: "Ve", Saturn: "Sa", Rahu: "Ra", Ketu: "Ke" },
  grahaName: {
    Sun: "Soleil",
    Moon: "Lune",
    Mars: "Mars",
    Mercury: "Mercure",
    Jupiter: "Jupiter",
    Venus: "Vénus",
    Saturn: "Saturne",
    Rahu: "Rahu",
    Ketu: "Ketu",
  },
  sign: {
    "0": "Bél",
    "1": "Tau",
    "2": "Gém",
    "3": "Can",
    "4": "Lio",
    "5": "Vie",
    "6": "Bal",
    "7": "Sco",
    "8": "Sag",
    "9": "Cap",
    "10": "Ver",
    "11": "Poi",
  },
  chart: {
    title: "Carte {{varga}}",
    lagna: "As",
    house: "Maison {{house}}, {{sign}}",
    noLagna: "Le lagna n’est pas encore calculé ; ciel actuel, maison 1 = Bélier",
    vargaUnavailable: "La carte {{varga}} arrivera dans une version ultérieure.",
  },
  panel: {
    chart: "Carte",
    graha_table: "Tableau des grahas",
    dasha_tree: "Arbre des dashas",
    dasha_strip: "Bande des dashas",
    transit_table: "Tableau des transits",
    notes: "Notes",
    birth_info: "Données de naissance",
    unknown: "Panneau inconnu",
    loading: "Chargement",
    unavailable: "Ce panneau arrivera dans une version ultérieure.",
    unknownKind: "Cette version ne reconnaît pas le panneau « {{kind}} ». La disposition est conservée telle quelle.",
    retry: "Réessayer",
    moveHint: "Alt+flèche déplace, Alt+Maj+flèche redimensionne",
  },
  table: { graha: "Graha", sign: "Signe", degree: "Degré", retro: "Rétro", empty: "Aucun graha à afficher." },
  errors: {
    ipc: "Impossible de joindre le cœur de l’application.",
    calc: "Le calcul n’a pas pu aboutir ({{detail}}).",
    engine_stopped: "Le moteur de calcul s’est arrêté. Si le problème persiste, redémarrez l’application.",
    data_files_missing: "Fichiers d’éphémérides introuvables : {{path}}",
    invalid_path: "Le chemin du dossier des éphémérides est inutilisable.",
    env_override: "Le chemin des éphémérides a été modifié par une variable d’environnement ; ce n’est pas pris en charge.",
    already_open: "Le moteur de calcul est déjà ouvert.",
    job_panicked: "Le calcul s’est arrêté de façon inattendue.",
    print_failed: "La fenêtre d’impression n’a pas pu s’ouvrir.",
    unknown: "Un problème inattendu est survenu ({{code}}).",
  },
  layout: {
    picker: "Disposition",
    interview: "Consultation",
    research: "Recherche",
    transit: "Transits",
    actions: "Actions sur la disposition",
    saveAs: "Enregistrer sous…",
    rename: "Renommer…",
    duplicate: "Dupliquer",
    delete: "Supprimer…",
    reset: "Rétablir par défaut",
    name: "Nom de la disposition",
    save: "Enregistrer",
    cancel: "Annuler",
    copyName: "Copie de {{name}}",
    confirmDelete: "Supprimer la disposition « {{name}} » ? Cette action est irréversible.",
    deleteNamed: "Supprimer « {{name}} »",
  },
  theme: { label: "Thème", light: "Clair", dark: "Sombre", system: "Système" },
  language: { label: "Langue", tr: "Türkçe", en: "English", fr: "Français" },
  shell: { printSpike: "Essai d’impression", backToDesk: "Retour au tableau", print: "Imprimer…" },
};
```

`app/ui/i18n/index.ts`:

```ts
// i18next kurulumu: Türkçe varsayılan, İngilizce ve Fransızca (spec §6). Kaynaklar
// pakete gömülüdür; başlatma eşzamanlıdır, ilk çizimde anahtar görünmez.
import i18next from "i18next";
import { initReactI18next } from "react-i18next";
import { en } from "./en";
import { fr } from "./fr";
import { tr } from "./tr";

export const LANGUAGES = ["tr", "en", "fr"] as const;
export type Language = (typeof LANGUAGES)[number];
export const LANGUAGE_STORAGE_KEY = "jyotish-desk.language.v1";
export const DEFAULT_LANGUAGE: Language = "tr";

export function isLanguage(v: unknown): v is Language {
  return typeof v === "string" && (LANGUAGES as readonly string[]).includes(v);
}

export function loadLanguage(storage: Storage = window.localStorage): Language {
  try {
    const v = storage.getItem(LANGUAGE_STORAGE_KEY);
    return isLanguage(v) ? v : DEFAULT_LANGUAGE;
  } catch {
    return DEFAULT_LANGUAGE;
  }
}

export const i18n = i18next.createInstance();

void i18n.use(initReactI18next).init({
  resources: { tr: { translation: tr }, en: { translation: en }, fr: { translation: fr } },
  lng: loadLanguage(),
  fallbackLng: DEFAULT_LANGUAGE,
  supportedLngs: [...LANGUAGES],
  interpolation: { escapeValue: false }, // React zaten kaçışlar
  initAsync: false,
});
document.documentElement.lang = i18n.language;

export async function setLanguage(lng: Language, storage: Storage = window.localStorage): Promise<void> {
  try {
    storage.setItem(LANGUAGE_STORAGE_KEY, lng);
  } catch (e) {
    console.warn("language preference not saved", e);
  }
  document.documentElement.lang = lng;
  await i18n.changeLanguage(lng);
}

export function currentLanguage(): Language {
  return isLanguage(i18n.language) ? i18n.language : DEFAULT_LANGUAGE;
}
```

`app/ui/setupTests.ts` (tamamı; her test Türkçe ve boş depolamayla başlar):

```ts
import "@testing-library/jest-dom/vitest";
import { i18n } from "./i18n";

// Her test Türkçe ve boş depolamayla başlar.
afterEach(async () => {
  localStorage.clear();
  await i18n.changeLanguage("tr");
});
```

- [ ] **Step 5: Ortak vurgu deposunu yaz**

`app/ui/highlight/store.tsx`:

```tsx
// Ortak vurgu durumu (spec §3.4): oturumluk, düzen durumunun dışında. Üzerine gelme geçici
// vurgudur; tıklama sabitler, aynı öğeye yeniden tıklama ya da Esc kaldırır.
import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { sameTarget, type HighlightTarget } from "./target";

export interface Highlight {
  /** Çizilecek vurgu: üzerine gelinen öğe, yoksa sabitlenen. */
  active: HighlightTarget | null;
  pinned: HighlightTarget | null;
  hover(target: HighlightTarget | null): void;
  togglePin(target: HighlightTarget): void;
  clear(): void;
}

const HighlightContext = createContext<Highlight | null>(null);

export function HighlightProvider({ children }: { children: ReactNode }) {
  const [hovered, setHovered] = useState<HighlightTarget | null>(null);
  const [pinned, setPinned] = useState<HighlightTarget | null>(null);

  const clear = useCallback(() => {
    setHovered(null);
    setPinned(null);
  }, []);
  const togglePin = useCallback((t: HighlightTarget) => setPinned((p) => (sameTarget(p, t) ? null : t)), []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") clear();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [clear]);

  const value = useMemo<Highlight>(
    () => ({ active: hovered ?? pinned, pinned, hover: setHovered, togglePin, clear }),
    [hovered, pinned, togglePin, clear],
  );
  return <HighlightContext.Provider value={value}>{children}</HighlightContext.Provider>;
}

export function useHighlight(): Highlight {
  const h = useContext(HighlightContext);
  if (h === null) throw new Error("useHighlight must be used inside <HighlightProvider>");
  return h;
}
```

- [ ] **Step 6: `unixMsToJdUt`'yi `time.ts`'e taşı**

`app/ui/time.ts`:

```ts
/** Unix milisaniyesini UT Jülyen gününe çevirir (1970-01-01T00:00Z = JD 2440587.5). */
export function unixMsToJdUt(ms: number): number {
  return ms / 86_400_000 + 2440587.5;
}
```

`app/ui/App.tsx`'te fonksiyon tanımını sil ve içe aktar. Dosyanın baş kısmı şöyle olur; `App` bileşeni değişmez:

```tsx
import { useEffect, useState } from "react";
import { api } from "./api";
import type { CommandError, PlanetRow } from "./bindings";
import { PrintSpike } from "./PrintSpike";
import { unixMsToJdUt } from "./time";

export function App() {
```

`app/ui/App.test.tsx`'te `import { App, unixMsToJdUt } from "./App";` satırını `import { App } from "./App";` yap ve `"unixMsToJdUt maps the Unix epoch to JD 2440587.5"` testini sil. Test artık `time.test.ts`'tedir.

- [ ] **Step 7: Panel altyapısını yaz**

`app/ui/panels/types.ts`:

```ts
import type { ComponentType } from "react";
import type { Panel } from "../layout/model";

export interface PanelProps {
  panel: Panel;
}

export interface PanelDefinition {
  /** Panel başlığının i18n anahtarı. */
  titleKey: string;
  /** Başlığa eklenen kısa bilgi (ör. harita için varga). */
  subtitle?: (panel: Panel) => string | null;
  component: ComponentType<PanelProps>;
}
```

`app/ui/panels/format.ts`:

```ts
/** Burç içi dereceyi derece ve yay dakikasıyla yazar: 16.5 → "16°30′" (dakika aşağı yuvarlanır). */
export function formatDegree(deg: number): string {
  const totalMinutes = Math.floor(deg * 60 + 1e-9);
  const d = Math.floor(totalMinutes / 60);
  const m = totalMinutes % 60;
  return `${d}°${String(m).padStart(2, "0")}′`;
}
```

`app/ui/panels/sky.tsx`:

```tsx
// Şimdiki gökyüzü: lagna ve doğum haritası komutları gelene kadar harita ve graha tablosu
// bu veriyi gösterir. Tek çağrı yapılır, iki panel aynı sonucu paylaşır.
import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { api } from "../api";
import type { CommandError, PlanetRow } from "../bindings";
import { unixMsToJdUt } from "../time";

export type SkyState =
  | { status: "loading" }
  | { status: "ok"; rows: PlanetRow[] }
  | { status: "error"; error: CommandError };

export interface Sky {
  state: SkyState;
  retry(): void;
}

const SkyContext = createContext<Sky | null>(null);

export function SkyProvider({ children, now = Date.now }: { children: ReactNode; now?: () => number }) {
  const nowRef = useRef(now);
  const [attempt, setAttempt] = useState(0);
  const [state, setState] = useState<SkyState>({ status: "loading" });

  useEffect(() => {
    let cancelled = false;
    setState({ status: "loading" });
    void api.planetLongitudes(unixMsToJdUt(nowRef.current())).then((r) => {
      if (cancelled) return;
      setState(r.status === "ok" ? { status: "ok", rows: r.data } : { status: "error", error: r.error });
    });
    return () => {
      cancelled = true;
    };
  }, [attempt]);

  const retry = useCallback(() => setAttempt((a) => a + 1), []);
  const value = useMemo(() => ({ state, retry }), [state, retry]);
  return <SkyContext.Provider value={value}>{children}</SkyContext.Provider>;
}

export function useSky(): Sky {
  const sky = useContext(SkyContext);
  if (sky === null) throw new Error("useSky must be used inside <SkyProvider>");
  return sky;
}
```

`app/ui/panels/states.tsx`:

```tsx
// Panel durumları (spec §5): 100 ms altında gösterge yok, 100 ms – 1 s arası başlıkta satır
// içi gösterge, 1 s üstünde panel iskeleti. Hatalar panel içinde, kurtarma eylemiyle.
import { createContext, useContext, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError } from "../bindings";

export type LoadPhase = "none" | "inline" | "skeleton";
export const INLINE_AFTER_MS = 100;
export const SKELETON_AFTER_MS = 1000;

/** Panel çerçevesi (pano) bu bağlamla başlıktaki satır içi göstergeyi öğrenir. */
export const PanelStatusContext = createContext<(phase: LoadPhase) => void>(() => {});

export function usePanelLoadPhase(loading: boolean): LoadPhase {
  const [phase, setPhase] = useState<LoadPhase>("none");
  useEffect(() => {
    if (!loading) {
      setPhase("none");
      return;
    }
    const inline = setTimeout(() => setPhase("inline"), INLINE_AFTER_MS);
    const skeleton = setTimeout(() => setPhase("skeleton"), SKELETON_AFTER_MS);
    return () => {
      clearTimeout(inline);
      clearTimeout(skeleton);
    };
  }, [loading]);
  const effective: LoadPhase = loading ? phase : "none";
  const report = useContext(PanelStatusContext);
  useEffect(() => {
    report(effective);
  }, [effective, report]);
  return effective;
}

export function PanelSkeleton() {
  const { t } = useTranslation();
  return (
    <div className="panel-skeleton" aria-busy="true" aria-label={t("panel.loading")}>
      <div className="panel-skeleton-bar" />
      <div className="panel-skeleton-bar" />
      <div className="panel-skeleton-bar" />
    </div>
  );
}

export function PanelError({ error, onRetry }: { error: CommandError; onRetry: () => void }) {
  const { t, i18n } = useTranslation();
  const key = `errors.${error.code}`;
  const message = i18n.exists(key) ? t(key, error.params) : t("errors.unknown", { code: error.code });
  return (
    <div className="panel-error" role="alert">
      <p>{message}</p>
      <button type="button" onClick={onRetry}>
        {t("panel.retry")}
      </button>
    </div>
  );
}
```

`app/ui/panels/labels.ts`:

```ts
import type { TFunction } from "i18next";
import { GRAHA_IDS, type ChartLabels, type GrahaId } from "../chart/types";

export function grahaAbbreviations(t: TFunction): Record<GrahaId, string> {
  return Object.fromEntries(GRAHA_IDS.map((id) => [id, t(`graha.${id}`)])) as Record<GrahaId, string>;
}

export function signAbbreviations(t: TFunction): string[] {
  return Array.from({ length: 12 }, (_, i) => t(`sign.${i}`));
}

export function chartLabels(t: TFunction, varga: string): ChartLabels {
  return {
    title: t("chart.title", { varga }),
    graha: grahaAbbreviations(t),
    signs: signAbbreviations(t),
    lagna: t("chart.lagna"),
    house: (house, sign) => t("chart.house", { house, sign }),
  };
}
```

`app/ui/panels/panels.css`:

```css
.panel-message,
.panel-notice {
  margin: 0;
  color: var(--text-2);
}
.panel-notice {
  padding: 4px 0;
}
.chart-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 4px;
}
.chart-panel-figure {
  flex: 1;
  min-height: 0;
}
.graha-table {
  width: 100%;
  border-collapse: collapse;
}
.graha-table th,
.graha-table td {
  text-align: left;
  padding: 2px 6px;
  border-bottom: 1px solid var(--line);
}
.graha-table td.num {
  text-align: right;
}
.graha-table tr.is-highlighted th,
.graha-table tr.is-highlighted td {
  background: var(--select);
  color: var(--ink-strong);
  font-weight: var(--fw-medium);
}
.link-button {
  all: unset;
  cursor: default;
}
.link-button:focus-visible {
  outline: 2px solid var(--ink);
  outline-offset: 2px;
}
.panel-skeleton {
  display: grid;
  gap: 8px;
  padding: 8px 0;
}
.panel-skeleton-bar {
  height: 12px;
  background: var(--select);
  border-radius: 2px;
}
.panel-error {
  color: var(--danger);
}
.panel-error p {
  margin: 0 0 8px;
}
```

- [ ] **Step 8: Panelleri ve kaydı yaz**

`app/ui/panels/ChartPanel.tsx`:

```tsx
import { useTranslation } from "react-i18next";
import { chartDataFromPlanetRows } from "../chart/adapter";
import { ChartSvg } from "../chart/ChartSvg";
import { isChartStyle } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { chartLabels } from "./labels";
import "./panels.css";
import { useSky } from "./sky";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export function chartVarga(settings: Record<string, unknown>): string {
  return typeof settings.varga === "string" ? settings.varga : "D1";
}

/**
 * Harita paneli. Lagna ve varga komutları M2–M3'te gelir; şimdilik yalnızca D1 çizilir ve
 * şimdiki gökyüzü gösterilir (1. ev Koç). Diğer vargalar açık bir bilgi metni gösterir.
 */
export function ChartPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const sky = useSky();
  const highlight = useHighlight();
  const phase = usePanelLoadPhase(sky.state.status === "loading");
  const varga = chartVarga(panel.settings);
  const style = isChartStyle(panel.settings.style) ? panel.settings.style : "north";

  if (varga !== "D1") return <p className="panel-message">{t("chart.vargaUnavailable", { varga })}</p>;
  if (sky.state.status === "error") return <PanelError error={sky.state.error} onRetry={sky.retry} />;
  if (sky.state.status === "loading") return phase === "skeleton" ? <PanelSkeleton /> : null;

  const data = chartDataFromPlanetRows(sky.state.rows);
  return (
    <div className="chart-panel">
      {data.ascSign === null && (
        <p className="panel-notice" role="note">
          {t("chart.noLagna")}
        </p>
      )}
      <div className="chart-panel-figure">
        <ChartSvg
          data={data}
          style={style}
          labels={chartLabels(t, varga)}
          highlight={highlight.active}
          onHover={highlight.hover}
          onPin={highlight.togglePin}
        />
      </div>
    </div>
  );
}
```

`app/ui/panels/GrahaTablePanel.tsx`:

```tsx
import { useTranslation } from "react-i18next";
import { signAndDegree } from "../chart/adapter";
import { isGrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { sameTarget, type HighlightTarget } from "../highlight/target";
import { formatDegree } from "./format";
import "./panels.css";
import { useSky } from "./sky";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Graha tablosu: graha, burç, burç içi derece, retro. Sütun seçimi (`settings.columns`) sonraki planda. */
export function GrahaTablePanel(_: PanelProps) {
  const { t } = useTranslation();
  const sky = useSky();
  const highlight = useHighlight();
  const phase = usePanelLoadPhase(sky.state.status === "loading");

  if (sky.state.status === "error") return <PanelError error={sky.state.error} onRetry={sky.retry} />;
  if (sky.state.status === "loading") return phase === "skeleton" ? <PanelSkeleton /> : null;
  if (sky.state.rows.length === 0) return <p className="panel-message">{t("table.empty")}</p>;

  return (
    <table className="graha-table">
      <thead>
        <tr>
          <th scope="col">{t("table.graha")}</th>
          <th scope="col">{t("table.sign")}</th>
          <th scope="col">{t("table.degree")}</th>
          <th scope="col">{t("table.retro")}</th>
        </tr>
      </thead>
      <tbody>
        {sky.state.rows.map((row) => {
          const pos = row.lon === null ? null : signAndDegree(row.lon);
          const target: HighlightTarget | null = isGrahaId(row.body) ? { type: "graha", id: row.body } : null;
          const on = target !== null && sameTarget(highlight.active, target);
          const pinned = target !== null && sameTarget(highlight.pinned, target);
          return (
            <tr
              key={row.body}
              data-graha={row.body}
              className={on ? "is-highlighted" : undefined}
              onMouseEnter={target ? () => highlight.hover(target) : undefined}
              onMouseLeave={target ? () => highlight.hover(null) : undefined}
            >
              <th scope="row">
                {target ? (
                  <button
                    type="button"
                    className="link-button"
                    aria-pressed={pinned}
                    onClick={() => highlight.togglePin(target)}
                    onFocus={() => highlight.hover(target)}
                    onBlur={() => highlight.hover(null)}
                  >
                    {t(`grahaName.${row.body}`)}
                  </button>
                ) : (
                  row.body
                )}
              </th>
              <td>{pos ? t(`sign.${pos.sign}`) : "—"}</td>
              <td className="num">{pos ? formatDegree(pos.degInSign) : "—"}</td>
              <td>{row.retrograde ? "℞" : ""}</td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
```

`app/ui/panels/UnavailablePanel.tsx`:

```tsx
import { useTranslation } from "react-i18next";
import { isKnownPanelKind } from "../layout/model";
import "./panels.css";
import type { PanelProps } from "./types";

/**
 * Henüz uygulanmamış panel türleri ve tanınmayan türler için. Hata değildir: panel
 * düzende yerini korur ve sonraki bir sürümde içeriğiyle gelir.
 */
export function UnavailablePanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  return (
    <p className="panel-message">
      {isKnownPanelKind(panel.kind) ? t("panel.unavailable") : t("panel.unknownKind", { kind: panel.kind })}
    </p>
  );
}
```

`app/ui/panels/registry.ts`:

```ts
// Panel türü → bileşen (spec §3.3). Yeni panel türü yalnızca buraya eklenir.
import { isKnownPanelKind, type KnownPanelKind } from "../layout/model";
import { ChartPanel, chartVarga } from "./ChartPanel";
import { GrahaTablePanel } from "./GrahaTablePanel";
import type { PanelDefinition } from "./types";
import { UnavailablePanel } from "./UnavailablePanel";

export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
  chart: { titleKey: "panel.chart", subtitle: (p) => chartVarga(p.settings), component: ChartPanel },
  graha_table: { titleKey: "panel.graha_table", component: GrahaTablePanel },
  dasha_tree: { titleKey: "panel.dasha_tree", component: UnavailablePanel },
  dasha_strip: { titleKey: "panel.dasha_strip", component: UnavailablePanel },
  transit_table: { titleKey: "panel.transit_table", component: UnavailablePanel },
  notes: { titleKey: "panel.notes", component: UnavailablePanel },
  birth_info: { titleKey: "panel.birth_info", component: UnavailablePanel },
};

const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };

export function panelDefinition(kind: string): PanelDefinition {
  return isKnownPanelKind(kind) ? PANEL_REGISTRY[kind] : UNKNOWN;
}
```

- [ ] **Step 9: Testleri ve derlemeyi çalıştır**

Komut: `cd app && pnpm vitest run ui/i18n ui/highlight ui/time.test.ts ui/panels && pnpm test && pnpm build`
Beklenen:
- İlk komut: 25 test geçer (i18n 6, highlight 2, time 1, format 1, states 4, panels 11).
- `pnpm test`: `Tests  132 passed (132)`. `App.test.tsx`'te 3 test kalır.
- `pnpm build`: `✓ built`.

- [ ] **Step 10: Commit**

```bash
git add app/package.json app/pnpm-lock.yaml app/ui/i18n app/ui/highlight/store.tsx app/ui/highlight/store.test.tsx app/ui/time.ts app/ui/time.test.ts app/ui/panels app/ui/setupTests.ts app/ui/App.tsx app/ui/App.test.tsx
git commit -m "feat(ui): i18n (tr/en/fr), shared highlight, panel registry with chart and graha table" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Pano: CSS grid, sürükleme, boyutlama, klavye

**Files:**
- Create: `app/ui/dashboard/metrics.ts`, `app/ui/dashboard/PanelFrame.tsx`, `app/ui/dashboard/Dashboard.tsx`, `app/ui/dashboard/dashboard.css`
- Create/Test: `app/ui/dashboard/metrics.test.ts`, `app/ui/dashboard/Dashboard.test.tsx`

**Interfaces:**
- Consumes:
  - `useLayoutStore()` → `active`, `updatePanels` (Task 4).
  - `columnsForWidth`, `movePanel`, `resizePanel`, `reflow`, `GRID_COLUMNS`, `ROW_HEIGHT_PX`, `Columns` (Task 3).
  - `panelDefinition`, `PanelStatusContext`, `LoadPhase` (Task 6).
- Produces:
  - `metrics.ts`:
    - `GRID_GAP_PX = 8`
    - `interface GridMetrics { columns; colWidth; rowHeight; gap }`
    - `gridMetrics(width, columns): GridMetrics`
    - `pxToGridDelta(dxPx, dyPx, m): { dx; dy }`
    - `arrowDelta(key): { dx; dy } | null`
  - `PanelFrame.tsx`:
    - `type DragMode = "move" | "resize"`
    - `<PanelFrame panel editable onDragStart onKeyboardEdit />`
    - `section`: `tabIndex=0`, `aria-label` = başlık (+ varga), `data-panel-id`, `data-x`, `data-y`, `data-w`, `data-h`. Stil `gridColumn` ve `gridRow`.
    - Başlık (`data-testid="panel-header-<id>"`) sürükleme tutamacıdır. Sağ alt köşedeki `data-testid="panel-resize-<id>"` boyut tutamacıdır.
    - 100 ms – 1 s arasında başlıkta `role="status"` göstergesi çıkar.
  - `Dashboard.tsx`: `<Dashboard width={number} />`. Genişliği kabuk ölçer; testler sabit genişlik verir.
- Davranış:
  - Sürüklerken önizleme gösterilir. `pointerup` ile `updatePanels` çağrılır, `pointercancel` vazgeçer.
  - Dinleyiciler `window` üzerindedir (pointer capture kullanılmaz).
  - Klavye yalnızca panelin kendisi odaktayken çalışır (`e.target === e.currentTarget`): `Alt+ok` taşır, `Alt+Shift+ok` boyutlandırır (spec §6).
  - 8 ve 4 sütunda düzenleme kapalıdır ve boyut tutamacı çizilmez.

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/dashboard/metrics.test.ts`:

```ts
import { arrowDelta, GRID_GAP_PX, gridMetrics, pxToGridDelta } from "./metrics";

test("column width accounts for gaps", () => {
  const m = gridMetrics(1200 + 11 * GRID_GAP_PX, 12);
  expect(m.colWidth).toBe(100);
  expect(m.rowHeight).toBe(32);
  expect(gridMetrics(0, 4).colWidth).toBe(0);
});

test("pixel deltas snap to the nearest cell", () => {
  const m = gridMetrics(1200 + 11 * GRID_GAP_PX, 12); // sütun adımı 108, satır adımı 40
  expect(pxToGridDelta(0, 0, m)).toEqual({ dx: 0, dy: 0 });
  expect(pxToGridDelta(53, 19, m)).toEqual({ dx: 0, dy: 0 });
  expect(pxToGridDelta(55, 21, m)).toEqual({ dx: 1, dy: 1 });
  expect(pxToGridDelta(-216, 120, m)).toEqual({ dx: -2, dy: 3 });
  expect(pxToGridDelta(500, 0, gridMetrics(0, 12))).toEqual({ dx: 0, dy: 0 });
});

test("arrow keys map to unit steps", () => {
  expect(arrowDelta("ArrowLeft")).toEqual({ dx: -1, dy: 0 });
  expect(arrowDelta("ArrowDown")).toEqual({ dx: 0, dy: 1 });
  expect(arrowDelta("a")).toBeNull();
});
```

`app/ui/dashboard/Dashboard.test.tsx`:

```tsx
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    planetLongitudes: vi.fn(async () => ({
      status: "ok",
      data: [{ body: "Sun", lon: 256.5, retrograde: false }],
    })),
  },
}));

import { api } from "../api";
import { HighlightProvider } from "../highlight/store";
import { hasOverlap } from "../layout/grid";
import type { LayoutState, Panel } from "../layout/model";
import { memoryPersistence } from "../layout/persistence";
import { LayoutProvider } from "../layout/store";
import { SkyProvider } from "../panels/sky";
import { Dashboard } from "./Dashboard";
import { GRID_GAP_PX } from "./metrics";

// 12 sütun, sütun adımı 108 px (100 + 8 aralık), satır adımı 40 px.
const WIDTH = 1200 + 11 * GRID_GAP_PX;

function state(panels: Panel[]): LayoutState {
  return {
    schema_version: 1,
    active_layout_id: "user:t",
    layouts: [{ id: "user:t", name: "T", builtin: false, panels }],
  };
}

const PANELS: Panel[] = [
  { id: "a", kind: "graha_table", x: 0, y: 0, w: 4, h: 4, settings: {} },
  { id: "b", kind: "notes", x: 4, y: 0, w: 4, h: 4, settings: {} },
  { id: "c", kind: "yoga_list", x: 0, y: 4, w: 4, h: 4, settings: {} },
];

function setup(width = WIDTH, panels = PANELS) {
  const persistence = memoryPersistence(state(panels));
  const utils = render(
    <LayoutProvider persistence={persistence} debounceMs={0}>
      <HighlightProvider>
        <SkyProvider now={() => 0}>
          <Dashboard width={width} />
        </SkyProvider>
      </HighlightProvider>
    </LayoutProvider>,
  );
  const frame = (id: string) => utils.container.querySelector<HTMLElement>(`[data-panel-id="${id}"]`)!;
  const box = (id: string) => {
    const f = frame(id);
    return [f.dataset.x, f.dataset.y, f.dataset.w, f.dataset.h].map(Number);
  };
  const all = () =>
    [...utils.container.querySelectorAll<HTMLElement>("[data-panel-id]")].map((f) => ({
      x: Number(f.dataset.x),
      y: Number(f.dataset.y),
      w: Number(f.dataset.w),
      h: Number(f.dataset.h),
    }));
  return { ...utils, persistence, frame, box, all };
}

test("renders every panel of the active layout on a 12-column grid", async () => {
  const { container, frame } = setup();
  expect(container.querySelector(".dashboard")).toHaveAttribute("data-columns", "12");
  expect(frame("a").style.gridColumn).toBe("1 / span 4");
  expect(frame("b").style.gridRow).toBe("1 / span 4");
  expect(screen.getByRole("region", { name: "Graha tablosu" })).toBeInTheDocument();
  expect(await screen.findByRole("button", { name: "Güneş" })).toBeInTheDocument();
  expect(screen.getByRole("region", { name: "Notlar" })).toHaveTextContent("Bu panel sonraki bir sürümde gelecek.");
  expect(screen.getByRole("region", { name: "Bilinmeyen panel" })).toHaveTextContent("“yoga_list” paneli");
});

test("Alt+Arrow moves the focused panel; Alt+Shift+Arrow resizes; neighbours are pushed down", async () => {
  const { frame, box, all } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.keyDown(frame("b"), { key: "ArrowLeft", altKey: true });
  expect(box("b")).toEqual([3, 0, 4, 4]);
  expect(box("a")).toEqual([0, 4, 4, 4]);
  expect(box("c")).toEqual([0, 8, 4, 4]);
  fireEvent.keyDown(frame("b"), { key: "ArrowDown", altKey: true, shiftKey: true });
  expect(box("b")).toEqual([3, 0, 4, 5]);
  expect(hasOverlap(all())).toBe(false);
  // Alt olmadan ok tuşları ve panel içindeki öğeler düzeni değiştirmez.
  fireEvent.keyDown(frame("b"), { key: "ArrowLeft" });
  fireEvent.keyDown(screen.getByRole("button", { name: "Güneş" }), { key: "ArrowRight", altKey: true });
  expect(box("b")).toEqual([3, 0, 4, 5]);
});

test("dragging the header moves the panel by whole cells and commits on pointer up", async () => {
  const { box, persistence, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.pointerDown(getByTestId("panel-header-a"), { clientX: 10, clientY: 10, button: 0 });
  fireEvent.pointerMove(window, { clientX: 10 + 108 * 8 + 30, clientY: 10 + 40 * 2 - 10 });
  expect(box("a")).toEqual([8, 2, 4, 4]); // önizleme
  fireEvent.pointerUp(window);
  expect(box("a")).toEqual([8, 2, 4, 4]);
  await waitFor(() => expect(persistence.saved).toHaveLength(1));
  const saved = persistence.saved[0].layouts.find((l) => l.id === "user:t")!;
  expect(saved.panels.find((p) => p.id === "a")).toMatchObject({ x: 8, y: 2 });
});

test("pointer cancel discards the drag", async () => {
  const { box, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.pointerDown(getByTestId("panel-header-a"), { clientX: 0, clientY: 0, button: 0 });
  fireEvent.pointerMove(window, { clientX: 108 * 5, clientY: 0 });
  fireEvent.pointerCancel(window);
  expect(box("a")).toEqual([0, 0, 4, 4]);
});

test("the bottom-right handle resizes and respects the minimum size", async () => {
  const { box, getByTestId } = setup();
  await screen.findByRole("button", { name: "Güneş" });
  fireEvent.pointerDown(getByTestId("panel-resize-b"), { clientX: 0, clientY: 0, button: 0 });
  fireEvent.pointerMove(window, { clientX: 108 * 2, clientY: 40 * 3 });
  fireEvent.pointerUp(window);
  expect(box("b")).toEqual([4, 0, 6, 7]);
  fireEvent.pointerDown(getByTestId("panel-resize-b"), { clientX: 0, clientY: 0, button: 0 });
  fireEvent.pointerMove(window, { clientX: -108 * 10, clientY: -40 * 10 });
  fireEvent.pointerUp(window);
  expect(box("b")).toEqual([4, 0, 3, 4]);
});

test("narrow windows reflow to 8 and 4 columns read-only", async () => {
  const { container, box, frame, all } = setup(700);
  await screen.findByRole("button", { name: "Güneş" });
  expect(container.querySelector(".dashboard")).toHaveAttribute("data-columns", "8");
  expect(container.querySelector('[data-testid^="panel-resize-"]')).toBeNull();
  expect(hasOverlap(all())).toBe(false);
  const before = box("b");
  fireEvent.keyDown(frame("b"), { key: "ArrowLeft", altKey: true });
  expect(box("b")).toEqual(before);
  const narrow = setup(320);
  expect(narrow.container.querySelectorAll('.dashboard[data-columns="4"]')).toHaveLength(1);
});

test("the header shows an inline loading indicator between 100 ms and 1 s", async () => {
  vi.useFakeTimers();
  let resolve: (v: Awaited<ReturnType<typeof api.planetLongitudes>>) => void = () => {};
  vi.mocked(api.planetLongitudes).mockReturnValueOnce(new Promise((r) => (resolve = r)));
  setup(WIDTH, [PANELS[0]]);
  expect(screen.queryByRole("status")).toBeNull();
  act(() => vi.advanceTimersByTime(150));
  expect(screen.getByRole("status")).toHaveTextContent("Yükleniyor");
  await act(async () => resolve({ status: "ok", data: [] }));
  expect(screen.queryByRole("status")).toBeNull();
  vi.useRealTimers();
});
```

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

Komut: `cd app && pnpm vitest run ui/dashboard`
Beklenen: FAIL, `Failed to resolve import "./metrics"` ve `"./Dashboard"`.

- [ ] **Step 3: Ölçüleri yaz**

`app/ui/dashboard/metrics.ts`:

```ts
// Piksel ↔ ızgara dönüşümü. Pano CSS grid'i `GRID_GAP_PX` aralıkla çizer.
import { ROW_HEIGHT_PX, type Columns } from "../layout/grid";

export const GRID_GAP_PX = 8;

export interface GridMetrics {
  columns: Columns;
  colWidth: number;
  rowHeight: number;
  gap: number;
}

export function gridMetrics(width: number, columns: Columns): GridMetrics {
  const colWidth = Math.max(0, (width - (columns - 1) * GRID_GAP_PX) / columns);
  return { columns, colWidth, rowHeight: ROW_HEIGHT_PX, gap: GRID_GAP_PX };
}

/** İşaretçinin piksel yer değiştirmesini en yakın ızgara adımına yuvarlar. */
export function pxToGridDelta(dxPx: number, dyPx: number, m: GridMetrics): { dx: number; dy: number } {
  const colStep = m.colWidth + m.gap;
  const rowStep = m.rowHeight + m.gap;
  return {
    dx: m.colWidth > 0 ? Math.round(dxPx / colStep) : 0,
    dy: Math.round(dyPx / rowStep),
  };
}

/** Alt+ok / Alt+Shift+ok için yön. */
export function arrowDelta(key: string): { dx: number; dy: number } | null {
  switch (key) {
    case "ArrowLeft":
      return { dx: -1, dy: 0 };
    case "ArrowRight":
      return { dx: 1, dy: 0 };
    case "ArrowUp":
      return { dx: 0, dy: -1 };
    case "ArrowDown":
      return { dx: 0, dy: 1 };
    default:
      return null;
  }
}
```

- [ ] **Step 4: Panel çerçevesini, panoyu ve stili yaz**

`app/ui/dashboard/dashboard.css`:

```css
.dashboard {
  display: grid;
}
.panel {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 4px;
  position: relative;
}
.panel:focus-visible {
  outline: 2px solid var(--ink);
  outline-offset: 2px;
}
.panel-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  border-bottom: 1px solid var(--line);
  font-size: var(--fs-panel-title);
  font-weight: var(--fw-medium);
  color: var(--text);
  user-select: none;
  touch-action: none;
}
.panel[data-editable="true"] .panel-header {
  cursor: grab;
}
.panel-subtitle {
  color: var(--text-2);
  font-weight: var(--fw-regular);
}
.panel-status {
  margin-left: auto;
  color: var(--text-2);
  font-size: var(--fs-table);
  font-weight: var(--fw-regular);
}
.panel-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 8px;
}
.panel-resize {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 12px;
  height: 12px;
  cursor: nwse-resize;
  touch-action: none;
  border-right: 2px solid var(--line);
  border-bottom: 2px solid var(--line);
}
```

`app/ui/dashboard/PanelFrame.tsx`:

```tsx
import { useState, type KeyboardEvent, type PointerEvent } from "react";
import { useTranslation } from "react-i18next";
import type { Panel } from "../layout/model";
import { panelDefinition } from "../panels/registry";
import { PanelStatusContext, type LoadPhase } from "../panels/states";
import { arrowDelta } from "./metrics";

export type DragMode = "move" | "resize";

export interface PanelFrameProps {
  panel: Panel;
  editable: boolean;
  onDragStart(e: PointerEvent, panel: Panel, mode: DragMode): void;
  onKeyboardEdit(panel: Panel, mode: DragMode, dx: number, dy: number): void;
}

/** Panel çerçevesi: başlık (sürükleme tutamacı), satır içi yükleme göstergesi, gövde, boyut tutamacı. */
export function PanelFrame({ panel, editable, onDragStart, onKeyboardEdit }: PanelFrameProps) {
  const { t } = useTranslation();
  const [phase, setPhase] = useState<LoadPhase>("none");
  const def = panelDefinition(panel.kind);
  const Body = def.component;
  const title = t(def.titleKey);
  const subtitle = def.subtitle?.(panel) ?? null;

  const onKeyDown = (e: KeyboardEvent<HTMLElement>) => {
    // Yalnızca panelin kendisi odaktayken; panel içindeki düğme ve alanlar etkilenmez.
    if (e.target !== e.currentTarget || !e.altKey || !editable) return;
    const step = arrowDelta(e.key);
    if (!step) return;
    e.preventDefault();
    onKeyboardEdit(panel, e.shiftKey ? "resize" : "move", step.dx, step.dy);
  };

  return (
    <section
      className="panel"
      data-panel-id={panel.id}
      data-kind={panel.kind}
      data-x={panel.x}
      data-y={panel.y}
      data-w={panel.w}
      data-h={panel.h}
      data-editable={editable}
      tabIndex={0}
      aria-label={subtitle ? `${title} ${subtitle}` : title}
      aria-keyshortcuts={editable ? "Alt+ArrowUp Alt+ArrowDown Alt+ArrowLeft Alt+ArrowRight" : undefined}
      title={editable ? t("panel.moveHint") : undefined}
      style={{ gridColumn: `${panel.x + 1} / span ${panel.w}`, gridRow: `${panel.y + 1} / span ${panel.h}` }}
      onKeyDown={onKeyDown}
    >
      <header className="panel-header" data-testid={`panel-header-${panel.id}`} onPointerDown={(e) => onDragStart(e, panel, "move")}>
        <span>{title}</span>
        {subtitle && <span className="panel-subtitle">{subtitle}</span>}
        {phase === "inline" && (
          <span className="panel-status" role="status">
            {t("panel.loading")}…
          </span>
        )}
      </header>
      <div className="panel-body">
        <PanelStatusContext.Provider value={setPhase}>
          <Body panel={panel} />
        </PanelStatusContext.Provider>
      </div>
      {editable && (
        <div
          className="panel-resize"
          data-testid={`panel-resize-${panel.id}`}
          aria-hidden="true"
          onPointerDown={(e) => onDragStart(e, panel, "resize")}
        />
      )}
    </section>
  );
}
```

`app/ui/dashboard/Dashboard.tsx`:

```tsx
// Pano: etkin düzenin panellerini CSS grid ile çizer. 12 sütunda paneller başlıktan
// sürüklenir, sağ alt köşeden boyutlanır, klavyeyle Alt+ok / Alt+Shift+ok ile düzenlenir.
// Daha dar pencerede (8 ya da 4 sütun) yeniden akış yalnızca gösterimdir; düzenleme kapalıdır
// ve kaydedilen 12 sütunluk düzen değişmez.
import { useEffect, useRef, useState, type PointerEvent as ReactPointerEvent } from "react";
import { columnsForWidth, movePanel, reflow, resizePanel, GRID_COLUMNS } from "../layout/grid";
import type { Panel } from "../layout/model";
import { useLayoutStore } from "../layout/store";
import "./dashboard.css";
import { gridMetrics, pxToGridDelta } from "./metrics";
import { PanelFrame, type DragMode } from "./PanelFrame";

interface DragState {
  id: string;
  mode: DragMode;
  startX: number;
  startY: number;
  origin: Panel;
  base: Panel[];
}

export function Dashboard({ width }: { width: number }) {
  const { active, updatePanels } = useLayoutStore();
  const columns = columnsForWidth(width);
  const editable = columns === GRID_COLUMNS;
  const metrics = gridMetrics(width, columns);

  const [preview, setPreview] = useState<Panel[] | null>(null);
  const previewRef = useRef<Panel[] | null>(null);
  const drag = useRef<DragState | null>(null);
  const detach = useRef<(() => void) | null>(null);

  useEffect(() => () => detach.current?.(), []);

  const showPreview = (panels: Panel[] | null) => {
    previewRef.current = panels;
    setPreview(panels);
  };

  const onDragStart = (e: ReactPointerEvent, panel: Panel, mode: DragMode) => {
    if (!editable || e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    drag.current = { id: panel.id, mode, startX: e.clientX, startY: e.clientY, origin: panel, base: active.panels };

    const onMove = (ev: PointerEvent) => {
      const d = drag.current;
      if (!d) return;
      const { dx, dy } = pxToGridDelta(ev.clientX - d.startX, ev.clientY - d.startY, metrics);
      showPreview(
        d.mode === "move"
          ? movePanel(d.base, d.id, d.origin.x + dx, d.origin.y + dy)
          : resizePanel(d.base, d.id, d.origin.w + dx, d.origin.h + dy),
      );
    };
    const finish = (commit: boolean) => () => {
      const result = previewRef.current;
      detach.current?.();
      drag.current = null;
      showPreview(null);
      if (commit && result) updatePanels(result);
    };
    const onUp = finish(true);
    const onCancel = finish(false);
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    window.addEventListener("pointercancel", onCancel);
    detach.current = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      window.removeEventListener("pointercancel", onCancel);
      detach.current = null;
    };
  };

  const onKeyboardEdit = (panel: Panel, mode: DragMode, dx: number, dy: number) => {
    updatePanels(
      mode === "move"
        ? movePanel(active.panels, panel.id, panel.x + dx, panel.y + dy)
        : resizePanel(active.panels, panel.id, panel.w + dx, panel.h + dy),
    );
  };

  const panels = editable ? (preview ?? active.panels) : reflow(active.panels, columns);
  return (
    <div
      className="dashboard"
      data-columns={columns}
      style={{
        gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
        gridAutoRows: `${metrics.rowHeight}px`,
        gap: `${metrics.gap}px`,
      }}
    >
      {panels.map((p) => (
        <PanelFrame key={p.id} panel={p} editable={editable} onDragStart={onDragStart} onKeyboardEdit={onKeyboardEdit} />
      ))}
    </div>
  );
}
```

- [ ] **Step 5: Testleri ve derlemeyi çalıştır**

Komut: `cd app && pnpm vitest run ui/dashboard && pnpm test && pnpm build`
Beklenen:
- `ui/dashboard`: 10 test geçer.
- `pnpm test`: `Tests  142 passed (142)`.
- `pnpm build`: `✓ built`.

- [ ] **Step 6: Commit**

```bash
git add app/ui/dashboard
git commit -m "feat(ui): dashboard grid with pointer drag/resize and Alt+Arrow keyboard editing" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: Kabuk: üst çubuk, düzen seçici, tema ve dil menüsü, `Cmd+Shift+N`

**Files:**
- Create: `app/ui/shell/Shell.tsx`, `app/ui/shell/TopBar.tsx`, `app/ui/shell/LayoutPicker.tsx`, `app/ui/shell/PrintView.tsx`, `app/ui/shell/useElementWidth.ts`, `app/ui/shell/shell.css`
- Modify: `app/ui/App.tsx` (tamamen yeniden), `app/ui/main.tsx`
- Modify/Test: `app/ui/App.test.tsx` (tamamen yeniden)
- Create/Test: `app/ui/shell/LayoutPicker.test.tsx`
- `app/ui/PrintSpike.tsx` ve `PrintSpike.test.tsx` değişmez. Baskı denemesi üst çubuktaki düğmeyle açılır.

**Interfaces:**
- Consumes:
  - `LayoutProvider`, `useLayoutStore`, `localStoragePersistence`, `memoryPersistence`, `LayoutPersistence` (Task 4).
  - `Dashboard` (Task 7).
  - `HighlightProvider`, `SkyProvider`, `useSky`, `PanelError` (Task 6).
  - `applyTheme`, `loadThemePref`, `saveThemePref`, `isThemePref`, `THEME_PREFS`, `ThemePref` (Task 1).
  - `setLanguage`, `isLanguage`, `LANGUAGES`, `DEFAULT_LANGUAGE` (Task 6).
  - `api.printWindow()`, `PrintSpike` (mevcut).
- Produces:
  - `App({ persistence?: LayoutPersistence })`: sağlayıcı sırası `LayoutProvider > HighlightProvider > SkyProvider > Shell`.
  - `Shell`:
    - Tema tercihini yükler, uygular ve kaydeder.
    - `Cmd+Shift+N` → `toggleNotes()`.
    - Pano genişliğini `useElementWidth` ile ölçer; ölçüm yoksa (jsdom) `window.innerWidth` kullanılır.
  - `TopBar`:
    - Solda başlık, ortada `LayoutPicker`.
    - Sağda `Tema` seçimi (Açık / Koyu / Sistem), `Dil` seçimi (Türkçe / English / Français) ve baskı denemesi düğmesi.
    - Sınıfı `no-print`'tir.
  - `LayoutPicker` ve `layoutLabel(t, layout)`: hazır düzen adı i18n'den gelir.
    - Menü (`role="menu"`): Farklı kaydet…, Yeniden adlandır…, Kopyala, Sil…, Varsayılana döndür.
    - Hazır düzende yeniden adlandırma ve silme devre dışıdır; kullanıcı düzeninde sıfırlama devre dışıdır.
    - Ad girişi satır içi formdur. Silme onayı düzenin adını taşıyan düğmeyle yapılır.
  - `PrintView`: `PrintSpike` + "Yazdır…" düğmesi. `printWindow` hatası `PanelError` ile gösterilir.

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/App.test.tsx` (tamamen değiştir):

```tsx
import { fireEvent, render, screen, within } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("./api", () => ({
  api: {
    engineStatus: vi.fn(async () => ({ status: "ok", data: { seVersion: "2.10.03" } })),
    planetLongitudes: vi.fn(async () => ({
      status: "ok",
      data: [
        { body: "Sun", lon: 256.5, retrograde: false },
        { body: "Saturn", lon: 10.25, retrograde: true },
      ],
    })),
    printWindow: vi.fn(async () => ({ status: "ok", data: null })),
  },
}));

import { api } from "./api";
import { App } from "./App";
import { memoryPersistence } from "./layout/persistence";
import { THEME_STORAGE_KEY } from "./theme/theme";

function renderApp() {
  return render(<App persistence={memoryPersistence()} />);
}

test("opens on the interview layout with the current-sky chart", async () => {
  renderApp();
  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Görüşme");
  expect(await screen.findByRole("img", { name: "D1 haritası" })).toBeInTheDocument();
  expect(screen.getByRole("note")).toHaveTextContent("Lagna henüz hesaplanmıyor; şimdiki gökyüzü, 1. ev Koç");
  expect(screen.getByText("D9 haritası sonraki bir sürümde gelecek.")).toBeInTheDocument();
  expect(screen.getByRole("region", { name: "Notlar" })).toBeInTheDocument();
  expect(api.planetLongitudes).toHaveBeenCalled();
});

test("Cmd+2 switches to the research layout with the graha table", async () => {
  renderApp();
  fireEvent.keyDown(window, { key: "2", metaKey: true });
  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Araştırma");
  const table = await screen.findByRole("region", { name: "Graha tablosu" });
  expect(within(table).getByRole("button", { name: "Satürn" }).closest("tr")).toHaveTextContent("℞");
});

test("Cmd+Shift+N toggles the notes panel in the active layout", () => {
  renderApp();
  expect(screen.getByRole("region", { name: "Notlar" })).toBeInTheDocument();
  fireEvent.keyDown(window, { key: "N", metaKey: true, shiftKey: true });
  expect(screen.queryByRole("region", { name: "Notlar" })).toBeNull();
  fireEvent.keyDown(window, { key: "N", metaKey: true, shiftKey: true });
  const notes = screen.getByRole("region", { name: "Notlar" });
  expect(notes).toHaveAttribute("data-x", "8");
  expect(notes).toHaveAttribute("data-w", "4");
});

test("theme menu sets data-theme and remembers the choice", () => {
  renderApp();
  const select = screen.getByRole("combobox", { name: "Tema" });
  expect(select).toHaveDisplayValue("Sistem");
  fireEvent.change(select, { target: { value: "dark" } });
  expect(document.documentElement.dataset.theme).toBe("dark");
  expect(localStorage.getItem(THEME_STORAGE_KEY)).toBe("dark");
  fireEvent.change(select, { target: { value: "light" } });
  expect(document.documentElement.dataset.theme).toBe("light");
});

test("language menu switches every string", async () => {
  renderApp();
  fireEvent.change(screen.getByRole("combobox", { name: "Dil" }), { target: { value: "en" } });
  expect(await screen.findByRole("combobox", { name: "Layout" })).toHaveDisplayValue("Consultation");
  expect(screen.getByRole("img", { name: "D1 chart" })).toBeInTheDocument();
  expect(screen.getByRole("combobox", { name: "Theme" })).toBeInTheDocument();
});

test("the print test stays reachable and calls printWindow", async () => {
  renderApp();
  fireEvent.click(screen.getByRole("button", { name: "Baskı denemesi" }));
  expect(await screen.findByText(/Doğum haritası — ğüşıöç İ/)).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yazdır…" }));
  expect(api.printWindow).toHaveBeenCalledOnce();
  fireEvent.click(screen.getByRole("button", { name: "Panoya dön" }));
  expect(await screen.findByRole("img", { name: "D1 haritası" })).toBeInTheDocument();
});

test("a print failure is shown with a retry", async () => {
  vi.mocked(api.printWindow).mockResolvedValueOnce({ status: "error", error: { code: "print_failed", params: {} } });
  renderApp();
  fireEvent.click(screen.getByRole("button", { name: "Baskı denemesi" }));
  fireEvent.click(screen.getByRole("button", { name: "Yazdır…" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Yazdırma penceresi açılamadı.");
});

test("a failed sky request shows an in-panel error instead of a blank desk", async () => {
  vi.mocked(api.planetLongitudes).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderApp();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
});
```

`app/ui/shell/LayoutPicker.test.tsx`:

```tsx
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
```

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

Komut: `cd app && pnpm vitest run ui/App.test.tsx ui/shell`
Beklenen: FAIL.
- `LayoutPicker.test.tsx`: `Failed to resolve import "./LayoutPicker"`.
- `App.test.tsx`: eski demo `App` düzen seçicisi çizmediği için `Unable to find an accessible element with the role "combobox" and name "Düzen"`.

- [ ] **Step 3: Ölçüm kancasını ve stili yaz**

`app/ui/shell/useElementWidth.ts`:

```ts
import { useLayoutEffect, useState, type RefObject } from "react";

/** Öğenin piksel genişliği; ölçülemezse (ör. jsdom) pencere genişliği. */
export function useElementWidth(ref: RefObject<HTMLElement | null>): number {
  const [width, setWidth] = useState(() => window.innerWidth);
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const measure = () => setWidth(el.getBoundingClientRect().width || window.innerWidth);
    measure();
    if (typeof ResizeObserver === "function") {
      const observer = new ResizeObserver(measure);
      observer.observe(el);
      return () => observer.disconnect();
    }
    window.addEventListener("resize", measure);
    return () => window.removeEventListener("resize", measure);
  }, [ref]);
  return width;
}
```

`app/ui/shell/shell.css`:

```css
.shell {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}
.topbar {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 6px 12px;
  background: var(--panel);
  border-bottom: 1px solid var(--line);
  color: var(--text);
}
.topbar-title {
  margin: 0;
  font-size: var(--fs-client);
  font-weight: var(--fw-medium);
}
.topbar-center {
  flex: 1;
  display: flex;
  justify-content: center;
}
.topbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
}
.topbar label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-2);
}
.layout-picker {
  display: flex;
  align-items: center;
  gap: 8px;
  position: relative;
}
.layout-menu {
  position: absolute;
  top: 100%;
  right: 0;
  z-index: 10;
  margin: 4px 0 0;
  padding: 4px;
  list-style: none;
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 4px;
  min-width: 200px;
}
.layout-menu button {
  width: 100%;
  text-align: left;
}
.layout-inline {
  display: flex;
  align-items: center;
  gap: 6px;
}
.shell-main {
  flex: 1;
  padding: 8px;
  background: var(--bg);
}
.print-view-actions {
  margin-bottom: 8px;
}
```

- [ ] **Step 4: Düzen seçiciyi ve üst çubuğu yaz**

`app/ui/shell/LayoutPicker.tsx`:

```tsx
// Düzen seçici ve işlemler menüsü: farklı kaydet, yeniden adlandır, kopyala, sil, sıfırla.
// Ad girişi ve silme onayı satır içinde açılır (WKWebView'da window.prompt'a güvenilmez).
import { useState, type FormEvent, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";
import type { Layout } from "../layout/model";
import { useLayoutStore } from "../layout/store";

export function layoutLabel(t: TFunction, layout: Layout): string {
  return layout.builtin ? t(`layout.${layout.id.slice("builtin:".length)}`) : layout.name;
}

type Mode = null | "saveAs" | "rename" | "confirmDelete";

export function LayoutPicker() {
  const { t } = useTranslation();
  const store = useLayoutStore();
  const active = store.active;
  const [menuOpen, setMenuOpen] = useState(false);
  const [mode, setMode] = useState<Mode>(null);
  const [name, setName] = useState("");
  const activeLabel = layoutLabel(t, active);

  const open = (next: Mode, initialName = "") => {
    setMenuOpen(false);
    setName(initialName);
    setMode(next);
  };
  const close = () => setMode(null);

  const submitName = (e: FormEvent) => {
    e.preventDefault();
    if (name.trim() === "") return;
    if (mode === "saveAs") store.saveAs(name);
    if (mode === "rename") store.rename(active.id, name);
    close();
  };

  const onMenuKey = (e: KeyboardEvent) => {
    if (e.key === "Escape") setMenuOpen(false);
  };

  return (
    <div className="layout-picker">
      <label>
        {t("layout.picker")}
        <select value={active.id} onChange={(e) => store.select(e.target.value)}>
          {store.state.layouts.map((l) => (
            <option key={l.id} value={l.id}>
              {layoutLabel(t, l)}
            </option>
          ))}
        </select>
      </label>
      <button type="button" aria-haspopup="menu" aria-expanded={menuOpen} onClick={() => setMenuOpen((o) => !o)}>
        {t("layout.actions")}
      </button>
      {menuOpen && (
        <ul className="layout-menu" role="menu" aria-label={t("layout.actions")} onKeyDown={onMenuKey}>
          <li role="none">
            <button type="button" role="menuitem" onClick={() => open("saveAs", t("layout.copyName", { name: activeLabel }))}>
              {t("layout.saveAs")}
            </button>
          </li>
          <li role="none">
            <button type="button" role="menuitem" disabled={active.builtin} onClick={() => open("rename", active.name)}>
              {t("layout.rename")}
            </button>
          </li>
          <li role="none">
            <button
              type="button"
              role="menuitem"
              onClick={() => {
                store.duplicate(active.id, t("layout.copyName", { name: activeLabel }));
                setMenuOpen(false);
              }}
            >
              {t("layout.duplicate")}
            </button>
          </li>
          <li role="none">
            <button type="button" role="menuitem" disabled={active.builtin} onClick={() => open("confirmDelete")}>
              {t("layout.delete")}
            </button>
          </li>
          <li role="none">
            <button
              type="button"
              role="menuitem"
              disabled={!active.builtin}
              onClick={() => {
                store.resetBuiltin(active.id);
                setMenuOpen(false);
              }}
            >
              {t("layout.reset")}
            </button>
          </li>
        </ul>
      )}
      {(mode === "saveAs" || mode === "rename") && (
        <form className="layout-inline" onSubmit={submitName}>
          <label>
            {t("layout.name")}
            <input autoFocus value={name} onChange={(e) => setName(e.target.value)} />
          </label>
          <button type="submit" disabled={name.trim() === ""}>
            {t("layout.save")}
          </button>
          <button type="button" onClick={close}>
            {t("layout.cancel")}
          </button>
        </form>
      )}
      {mode === "confirmDelete" && (
        <div className="layout-inline" role="group" aria-label={t("layout.delete")}>
          <span>{t("layout.confirmDelete", { name: activeLabel })}</span>
          <button
            type="button"
            onClick={() => {
              store.remove(active.id);
              close();
            }}
          >
            {t("layout.deleteNamed", { name: activeLabel })}
          </button>
          <button type="button" autoFocus onClick={close}>
            {t("layout.cancel")}
          </button>
        </div>
      )}
    </div>
  );
}
```

`app/ui/shell/TopBar.tsx`:

```tsx
import { useTranslation } from "react-i18next";
import { isLanguage, LANGUAGES, setLanguage, DEFAULT_LANGUAGE } from "../i18n";
import { isThemePref, THEME_PREFS, type ThemePref } from "../theme/theme";
import { LayoutPicker } from "./LayoutPicker";

export interface TopBarProps {
  themePref: ThemePref;
  onThemeChange(pref: ThemePref): void;
  printView: boolean;
  onTogglePrint(): void;
}

/** Üst çubuk (spec §4): solda başlık, ortada düzen seçici, sağda tema, dil ve baskı. */
export function TopBar({ themePref, onThemeChange, printView, onTogglePrint }: TopBarProps) {
  const { t, i18n } = useTranslation();
  const language = isLanguage(i18n.language) ? i18n.language : DEFAULT_LANGUAGE;
  return (
    <header className="topbar no-print">
      <h1 className="topbar-title">{t("app.title")}</h1>
      <div className="topbar-center">{!printView && <LayoutPicker />}</div>
      <div className="topbar-right">
        <label>
          {t("theme.label")}
          <select value={themePref} onChange={(e) => isThemePref(e.target.value) && onThemeChange(e.target.value)}>
            {THEME_PREFS.map((p) => (
              <option key={p} value={p}>
                {t(`theme.${p}`)}
              </option>
            ))}
          </select>
        </label>
        <label>
          {t("language.label")}
          <select value={language} onChange={(e) => isLanguage(e.target.value) && void setLanguage(e.target.value)}>
            {LANGUAGES.map((l) => (
              <option key={l} value={l}>
                {t(`language.${l}`)}
              </option>
            ))}
          </select>
        </label>
        <button type="button" onClick={onTogglePrint}>
          {printView ? t("shell.backToDesk") : t("shell.printSpike")}
        </button>
      </div>
    </header>
  );
}
```

- [ ] **Step 5: Baskı görünümünü, kabuğu ve uygulama kökünü yaz**

`app/ui/shell/PrintView.tsx`:

```tsx
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError } from "../bindings";
import { useSky } from "../panels/sky";
import { PanelError } from "../panels/states";
import { PrintSpike } from "../PrintSpike";

/** M0 baskı denemesi; M7'deki A4 baskı düzenleri gelene kadar menüden erişilir. */
export function PrintView() {
  const { t } = useTranslation();
  const sky = useSky();
  const [error, setError] = useState<CommandError | null>(null);
  const print = async () => {
    setError(null);
    const r = await api.printWindow();
    if (r.status === "error") setError(r.error);
  };
  return (
    <div>
      <div className="print-view-actions no-print">
        <button type="button" onClick={() => void print()}>
          {t("shell.print")}
        </button>
        {error && <PanelError error={error} onRetry={() => void print()} />}
      </div>
      <PrintSpike rows={sky.state.status === "ok" ? sky.state.rows : []} />
    </div>
  );
}
```

`app/ui/shell/Shell.tsx`:

```tsx
import { useEffect, useRef, useState } from "react";
import { Dashboard } from "../dashboard/Dashboard";
import { useLayoutStore } from "../layout/store";
import { applyTheme, loadThemePref, saveThemePref, type ThemePref } from "../theme/theme";
import { PrintView } from "./PrintView";
import "./shell.css";
import { TopBar } from "./TopBar";
import { useElementWidth } from "./useElementWidth";

export function Shell() {
  const { toggleNotes } = useLayoutStore();
  const [themePref, setThemePref] = useState<ThemePref>(() => loadThemePref());
  const [printView, setPrintView] = useState(false);
  const measureRef = useRef<HTMLDivElement>(null);
  const width = useElementWidth(measureRef);

  useEffect(() => {
    saveThemePref(themePref);
    return applyTheme(themePref);
  }, [themePref]);

  // Cmd+Shift+N: notlar panelini etkin düzende aç/kapat (spec §3.2).
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.metaKey && e.shiftKey && !e.altKey && !e.ctrlKey && e.key.toLowerCase() === "n") {
        e.preventDefault();
        toggleNotes();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [toggleNotes]);

  return (
    <div className="shell">
      <TopBar
        themePref={themePref}
        onThemeChange={setThemePref}
        printView={printView}
        onTogglePrint={() => setPrintView((v) => !v)}
      />
      <main className="shell-main">
        <div ref={measureRef}>{printView ? <PrintView /> : <Dashboard width={width} />}</div>
      </main>
    </div>
  );
}
```

`app/ui/App.tsx` (tamamen değiştir):

```tsx
import { useState } from "react";
import { HighlightProvider } from "./highlight/store";
import { localStoragePersistence, type LayoutPersistence } from "./layout/persistence";
import { LayoutProvider } from "./layout/store";
import { SkyProvider } from "./panels/sky";
import { Shell } from "./shell/Shell";

/** Uygulama kökü. `persistence` testler için verilebilir; varsayılan localStorage'dır. */
export function App({ persistence }: { persistence?: LayoutPersistence }) {
  const [store] = useState(() => persistence ?? localStoragePersistence());
  return (
    <LayoutProvider persistence={store}>
      <HighlightProvider>
        <SkyProvider>
          <Shell />
        </SkyProvider>
      </HighlightProvider>
    </LayoutProvider>
  );
}
```

`app/ui/main.tsx` (tamamı):

```tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./i18n";
import "./theme/tokens.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
```

- [ ] **Step 6: Testleri ve derlemeyi çalıştır**

Komut: `cd app && pnpm vitest run ui/App.test.tsx ui/shell ui/PrintSpike.test.tsx && pnpm test && pnpm build`
Beklenen:
- İlk komut: 15 test geçer (App 8, LayoutPicker 5, PrintSpike 2).
- `pnpm test`: `Tests  152 passed (152)`.
- `pnpm build`: `✓ built`.

- [ ] **Step 7: Elle duman testi (Tauri penceresi)**

Komut: `cd app && pnpm tauri dev`
Beklenen:
- Pencere "Görüşme" düzeniyle açılır. D1 Kuzey Hint haritası şimdiki gökyüzünü ve "Lagna henüz hesaplanmıyor…" notunu gösterir. D9, dasa şeridi ve notlar panelleri "sonraki sürüm" metnini gösterir.
- `Cmd+2` Araştırma düzenine geçer ve graha tablosu dolar.
- Tablodaki bir grahanın üzerine gelince haritada da vurgulanır. `Esc` vurguyu kaldırır.
- Bir panel başlığından sürüklenince ızgaraya oturur ve diğer paneller aşağı itilir. Sağ alt köşeden boyutlanır.
- Panel odaktayken `Alt+ok` taşır, `Alt+Shift+ok` boyutlandırır.
- `Cmd+Shift+N` notlar panelini kapatır ve açar.
- Tema menüsünde "Koyu" seçilince renkler gece paletine geçer. Dil menüsünde "English" seçilince tüm metinler İngilizceye döner.
- Uygulama kapatılıp açılınca düzen, tema ve dil korunur.
- "Baskı denemesi" → "Yazdır…" macOS yazdırma penceresini açar.

- [ ] **Step 8: Commit**

```bash
git add app/ui/shell app/ui/App.tsx app/ui/App.test.tsx app/ui/main.tsx
git commit -m "feat(ui): shell with layout picker, theme and language menus, Cmd+Shift+N notes toggle" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Bu planın tamamlanma ölçütü

- `cd app && pnpm test` geçer: 152 test.
- `cd app && pnpm build` hatasız geçer: `tsc --noEmit` ve `vite build`.
- `core-macos` CI işi yeşil (`pnpm install --frozen-lockfile` dahil).
- `app/ui/bindings.ts` değişmedi. Rust tarafında değişiklik yok.
- Hex renk yalnızca `app/ui/theme/tokens.css`'te bulunur (`no-hex.test.ts`).
- Task 8 Step 7'deki duman testi elle yapıldı.

## Bu planın kapsamadıkları (sonraki planlar)

- **Danışan ve harita akışı (M6):** sol kenar çubuğu (danışan listesi, arama, etiketler, `Cmd+\`), `Cmd+K` komut paleti, doğum verisi düzenleme paneli (atlas araması, DST seçimleri), ilk açılıştaki "Yeni harita" boş durumu ve üst çubuktaki danışan adı ile harita etiketi. Hepsi store ve `open_client` komutlarına bağlıdır.
- **`ViewState` ile kalıcılık (M4):** `LayoutPersistence`'ın store komutuyla çalışan uygulaması. `localStorage`'daki düzen o zaman bir kez taşınır.
- **Panel içerikleri:** `dasha_tree`, `dasha_strip`, `transit_table` (tarih kaydırıcı dahil), `notes` ve `birth_info`. Her biri kendi backend komutuyla gelir (M3, M4, M6). Dasa seçiminin transit tablosunu ve halkasını o tarihe götürmesi (spec §3.4) de bunlarla birlikte yapılır.
- **Gerçek harita verisi (M2, M3):**
  - Lagna ve evler gelince `ascSign` dolar ve "Lagna henüz hesaplanmıyor" notu kendiliğinden kalkar.
  - D9, D10 ve diğer vargalar. Transit halkasının çizimi.
  - `combust` ve `vargottama` işaretleri backend verisiyle dolar; çizimleri bu planda hazırdır.
- **Panel ayarları arayüzü:** harita panelinde stil (kuzey / güney / daire) ve varga seçimi, graha tablosunda sütun seçimi (`settings.columns`; nakshatra, pada, hız, avastha), isteğe bağlı "doğal yararlı/zararlı tonu". Ayarlar modelde vardır ve harita paneli `settings.style`'ı okur; ayarı değiştiren denetimler sonraki planda gelir.
- **Uyarı rozetleri** (`PreTzdbReliability`, `LmtFromLongitude`, `TimeUnknownNoon`): doğum bilgisi panelinde ve harita başlığında gösterilir; `Chart.meta.warnings` ile gelir.
- **Nakshatra kısaltmaları:** i18n setine nakshatra verisiyle birlikte eklenir.
- **Baskı (M7):** A4 baskı düzenleri (kapak, D1+D9, dasa, graha tablosu). `PrintSpike` o zaman kaldırılır; `no-hex.test.ts`'teki istisnaları da onunla birlikte kalkar.
- **Görsel testler:** üç harita stili için açık ve koyu modda SVG anlık görüntüleri.
- **Uçtan uca testler (playwright):** düzen değiştirme, panel taşıma ve vurgunun paneller arasında yayılması. Playwright kurulunca eklenir.
