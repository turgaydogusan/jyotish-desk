# Plan 12 — M7: baskı ve PDF düzenleri

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** M7'yi kapatmak: açık harita `⌘P` ile bir baskı önizlemesine açılır; önizleme dört A4 düzenden oluşur (kapak, D1+D9, Vimshottari dasa, graha tablosu), sayfalar ve harita biçimi seçilir, "Yazdır…" macOS yazdırma penceresini açar ve PDF o pencerenin "PDF olarak kaydet" seçeneğiyle üretilir. M0'dan beri duran baskı denemesi (`PrintSpike`) kaldırılır; baskı yolu bir sonda betiğiyle A4 ve Letter kâğıtta otomatik denetlenir.

**Architecture:**
- **Rust değişmez.** Var olan `print_window` komutu (wry `printOperation(with:)`) kullanılır; yeni IPC komutu, yetenek izni, bağımlılık ve şema değişikliği yoktur (D1, C2′, C3).
- **Belge modeli:** `print/model.ts` açık haritadan (`OpenedClient`) sayfa sayfa düz veri üretir (`PrintDocument`); bütün metinler orada çözülür. `print/PrintDocumentView.tsx` yalnızca dizer (D8). Hesap yoktur (C4).
- **Kâğıt:** her `.print-page` bir kâğıttır; kenar boşluğu sayfanın 15 mm iç dolgusudur ve `@page` kenarı 0'dır (D3). `.paper` sınıfı baskı paletini ekrandaki önizlemeye de verir (D7). Kâğıtta yazı tipi adıyla verilir (D6).
- **Kabuk:** `Shell` yazdırma çağrısını ve hatasını tutar; `⌘P` tek kısayol dinleyicisinden geçer (C7); önizlemede pano, kenar çubuğu ve düzen kısayolları yoktur (D9).
- **Testler:** Vitest (model, sayfalar, önizleme, kabuk), Playwright e2e (baskı ortamı, kâğıda sığma), yerel görsel temeller (dört kâğıt) ve gerçek yazdırma yolunu PDF üzerinden denetleyen `scripts/print-probe.sh` (D2).

**Tech Stack:** React 19, Vite 8, TypeScript 7 (`strict`), Vitest 5 (jsdom), i18next 27, Playwright 1.65.1 (WebKit), Tauri 2.12.7 / wry 0.57.3 (değişmez). Sonda: Swift (Xcode komut satırı araçları; AppKit, WebKit, PDFKit — sistem çatıları). **Yeni bağımlılık yok** (Rust ve npm).

**Spec:** `docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md` §3 ("Print ve PDF"), §4 ("PDF"), §10 (isim kuralları), §11 (M7), §12 (baskı düzenleri soyut tutulur); `docs/superpowers/specs/2026-11-24-ui-design.md` §2.1 (baskı her zaman açık tema, yalnızca `--ink` ve `--text`), §2.3 (retro düz `R`, Rahu ve Ketu'da yok; yanma `c`; vargottama altı çizili), §7 (kapak, D1+D9, dasa, graha tablosu).

**Ön koşul:** Taban commit `ecf7ed5` (Plan 11 ve son inceleme düzeltmeleri `main`'de). Plan 11 Task 16 adım 5'in elle denetim kapısı ilerleme notlarında "geçti" olarak kayıtlı olmalıdır; değilse Task 1 başlamaz.

## Kararlar

Muhafazakâr seçenek seçildi; her karar bir satır gerekçeyle. D2, D3, D6 ve D10 bu plan yazılırken ölçülmüştür (macOS 27.2, wry 0.57.3'ün yazdırma çağrısının aynısı; ayrıntı Task 7'deki sonda ve bulgu belgesi).

- **D1 — Yazdırma yolu var olan `print_window` komutudur; PDF, macOS yazdırma penceresinin "PDF olarak kaydet" seçeneğidir. Ayrı bir "PDF dışa aktar" komutu, Rust tarafı kayıt penceresi ve Typst yoktur.** Gerekçe: v1 spec §4 bu yolu tarif eder; D2 yolun çalıştığını gösterdi; ayrı bir PDF komutu ya `objc` çağrıları ya da yeni bağımlılık ister ve C2′ altında ikinci bir dosya penceresi açar.
- **D2 — M0 baskı denemesinin açık soruları bir sonda ile kapanır: `scripts/print-probe.sh` önizlemeyi uygulamanın kullandığı çağrıyla pencere göstermeden PDF'e yazdırır ve sayfa sayısını, kâğıt boyutunu, her sayfanın metnini ve iş başlığını denetler (A4 ve Letter; saati bilinen ve bilinmeyen harita). Sonda yereldir, CI'a eklenmez.** Gerekçe: eski denetim listesi elle doldurulmayı bekliyordu; sonda aynı soruları tekrarlanabilir biçimde yanıtlar ve plan yazılırken iki gerçek hatayı buldu (D3 ölçeği, D6 yazı tipi). CI koşucusunda penceresiz WebKit yazdırması doğrulanmamıştır; elle denetim kapısı (Task 8) gerçek uygulamayı ayrıca dener.
- **D3 — Kenar boşluğu kâğıdın değil sayfanın işidir: `@page { size: A4; margin: 0 }`, her `.print-page` 15 mm iç dolgulu ve `break-after: page`; her düzen tek kâğıda sığacak biçimde tasarlanır (taşan akış yoktur).** Gerekçe: wry yazdırma işinde `NSPrintInfo` kenarlarını 0 yapar (`wkwebview/mod.rs`, `print_with_options`); sayfa kutusu kenarlarına güvenmek kâğıt kenarına yapışan içerik verir. Ölçüm: WebKit içeriği yaklaşık 1,067 kat büyütür; kâğıdın CSS karşılığı A4'te 744 × 1053 px, Letter'da 765 × 990 px'tir (15 mm dolgu kâğıtta 16 mm olur). Düzenler bu bütçeye göre boyutlanır ve e2e testi aynı sınırı denetler.
- **D4 — Dört düzen: kapak (doğum bilgisi, hesap ayarları, uyarılar, sürümler), haritalar (D1 ve D9 alt alta, işaret açıklamasıyla), Vimshottari dasa (dokuz mahadasa, her biri dokuz antardasa; 3 × 3), graha tablosu (lagna satırı ve on bir sütunun hepsi). Notlar, transit, ashtakavarga ve panchang basılmaz.** Gerekçe: UI spec §7'nin listesi budur; notlar hassas veridir (v1 spec §5) ve kâğıda varsayılan olarak çıkmamalıdır; transit anı oturumluktur.
- **D5 — Baskı seçenekleri (hangi sayfalar, harita biçimi) `localStorage`'da `jyotish-desk.print.v1` anahtarında durur; okuma toleranslıdır; son sayfa kapatılamaz ve nedeni görünür.** Gerekçe: tema ve dil gibi bir arayüz tercihidir, danışan verisi içermez; şema değişmez (C3). Harita biçimi panodaki panellerden bağımsızdır: kâğıtta iki harita da aynı biçimde basılır.
- **D6 — Kâğıtta yazı tipi `"Helvetica Neue", Helvetica, Arial, sans-serif`'tir; arayüzün sistem fontu değişmez.** Gerekçe: ölçüm: sistem fontu Türkçe `lang` altında PDF'in metin katmanında noktalı "i"yi yitirir ("Tarih" → "Tar!h"); PDF'te arama ve kopyalama bozulur. Helvetica Neue her macOS'ta kuruludur; font gömülmez ve bağımlılık eklenmez (UI spec §2.2).
- **D7 — `tokens.css`'e `.paper` bloğu eklenir: `@media print` paletinin aynısı, sınıfla. Önizleme koyu temada da kâğıt beyazıdır. Kâğıtta lagna evi dolguyla değil kalın çizgiyle belirtilir (`.paper .chart-cell--lagna`).** Gerekçe: önizleme basılacak şeyi göstermelidir; baskıda `--select` beyazdır ve lagna dolgusu görünmez (Plan 04 Task 5 notu); renk tek başına anlam taşımaz (C9).
- **D8 — Baskı belgesi düz veridir (`PrintDocument`); çizen bileşen metin üretmez.** Gerekçe: v1 spec §12 baskı düzenlerinin soyut tutulmasını ister (Windows taşıması); model tek başına sınanır.
- **D9 — `⌘P` önizlemeyi açar, önizlemede yazdırma penceresini ister; her durumda tüketilir (web görünümü panoyu kendi başına yazdırmaz). Önizleme düğmesi ve palet komutu yalnızca açık harita varken vardır. Önizlemede `⌘1…9` düzen değiştirmez.** Gerekçe: iki adım yanlışlıkla yazdırmayı önler ve seçenekleri gösterir; görünmeyen panonun düzenini değiştirmek Plan 04 incelemesinin ertelenen bulgusudur (M4).
- **D10 — Önizleme açıkken `document.title` "Danışan – Harita" olur, kapanınca geri döner.** Gerekçe: ölçüm: yazdırma işinin adı, PDF'in başlık alanı ve "PDF olarak kaydet"in önerdiği dosya adı belge başlığından gelir. Başlık loglanmaz (C1) ve pencere başlığını değiştirmez (yetenek izni gerekmez).
- **D11 — Plan 10 kapsam dışı 16'nın yarısı bu plandadır: kenar çubuğunun açık/kapalı durumu saklanır (`jyotish-desk.sidebar.v1`). Genişliğin elle ayarlanması ve Plan 10 kapsam dışı 15 (paletten notun kendisine gitme) kapalı beta geri bildirimine bırakılır.** Gerekçe: durum kaydı birkaç satırdır ve oturum geri yüklemenin doğal parçasıdır; boyutlandırma erişilebilir bir ayırıcı ister, nota gitme `SearchHit`'e not kimliği (IPC değişikliği) ister; ikisi de baskıyla ilgisizdir ve istek gelmeden yapılmaz.
- **D12 — `PrintSpike.tsx`, testi ve `ui/print.css` silinir; `no-hex.test.ts` istisnaları kalkar; `shell.printSpike`, `shell.backToDesk`, `shell.print` anahtarları `print.*` altına taşınır.** Gerekçe: Plan 04'ün "M7'de kaldırılır" notu.
- **D13 — Basılı harita etkileşimsizdir: `ChartSvg` `interactive={false}` ile `role="img"` olur; evler ve gezegenler düğme ya da sekme durağı değildir. Varsayılan (`true`) çıktıyı değiştirmez.** Gerekçe: önizlemede iki harita 42 işlevsiz sekme durağı eklerdi; ayrıntıyı aynı belgedeki graha tablosu taşır. Mevcut SVG anlık görüntüleri ve görsel temeller değişmez (C11).
- **D14 — Doğum saati bilinmiyorsa kâğıt panellerle aynı kuralları izler: lagna ve ev çizilmez, graha tablosunda lagna satırı ve ev sütunu yoktur, Ay'ın saate bağlı değerleri "≈" ile başlar, harita, dasa ve graha tablosu sayfalarında açıklama notu, kapakta uyarı metni basılır.** Gerekçe: v1 spec §7; kâğıt ekrandan fazlasını iddia etmemelidir.
- **D15 — Görsel temellere dört kâğıt eklenir (yerel, darwin); mevcut altı temel değişmez.** Gerekçe: düzenler piksel düzeyinde korunur; C11.

## Global Constraints

- Lisans: AGPL-3.0-or-later. Bu plan bağımlılık eklemez (`Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml` değişmez); `cargo deny check` Task 8'de `advisories ok, bans ok, licenses ok, sources ok` verir.
- **Yasaklı ad kuralı (v1 spec §10 "İsim kuralları", ilk madde):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Kural harf dizisi düzeyindedir: Türkçe sözcüklerin içinde de geçemez (yerine ör. "diğer", "yarıda kesilen", "durur" yazılır). Her commit adımı eklenen satırları, dosya adlarını ve commit mesajını `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile tarar; çıktı boş olmalıdır (C13). Bu belgede de eşleşme yoktur.
- Kapılar: `pnpm --dir app test`, `pnpm --dir app build` (`tsc --noEmit`, `tsc --noEmit -p e2e`, `vite build`) her görev sonunda yeşil; `pnpm --dir app e2e` Task 6 ve 8'de yeşil. `tsc` `strict`. Tek dosya: `pnpm --dir app exec vitest run ui/<yol>`.
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez). Node 24+, pnpm 11.8.0. Sonda için Xcode komut satırı araçları (`xcrun swiftc`).
- **Çalışan `pnpm tauri dev` durdurulmaz** (port 1420). e2e 1421'i, sonda 1423'ü kullanır.
- CI işleri (`deny`, `core-macos`, `core`, `macos-x86_64`, `msrv`) bu planda değişmez. Yalnızca macOS hedeflenir; Windows ve Android işi yoktur. Yorum metni (astrolojik yorum) eklenmez.
- **Tabanlar (`ecf7ed5`):** Vitest `Test Files  54 passed (54)`, `Tests  461 passed (461)`; `jyotish-core` lib 110, golden 8, `store` lib 80 (+1 `#[ignore]`), `jyotish-desk` lib 34, `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2; e2e 9; görsel 6 (yerel, darwin). Rust sayıları bu planda değişmez.
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir:**
  - **C1** Gizlilik: danışan adları, not gövdeleri, doğum verisi, dosya yolları ve `CommandError.params` loglanmaz. `no-console.test.ts` durur; izin listesi büyümez. Baskı seçenekleri ve kenar çubuğu durumu `localStorage`'a yazılamazsa sessizce geçilir. Sonda yalnızca fikstür verisini kullanır ve çıktısında yalnızca fikstürün adı geçer.
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu almaz; dosya pencereleri Rust'ta açılır. Yetenek dosyası `dialog:` ya da `fs:` izni vermez; `tauri_plugin_fs` başlatılmaz. Bu plan komut eklemez; PDF'in yolunu macOS yazdırma penceresi sorar.
  - **C3** Göç V1 donmuştur; bu plan şemaya ve `ViewState`'e dokunmaz (D5, D11: tercihler `localStorage`'da).
  - **C4** Arayüz astrolojik hesap yapmaz. Kâğıttaki her değer Rust'tan gelen alanların biçimlenmesidir; `formatJdDate` tek JD→tarih kodudur. Kâğıda "bugün" basılmaz.
  - **C5** Görünüm, ayar ve not yazımları sıralıdır; baskı önizlemesi hiçbir şey kaydetmez.
  - **C6** Düzen ayrıştırıcıları ve `layout-parity.json` değişmez.
  - **C7** Kısayollar yalnızca `shortcuts.ts`'e (`matchShortcut` + `SHORTCUT_LIST`) eklenir; Shell'in tek dinleyicisi durur. Task 5 `⌘P` ekler.
  - **C8** Her pencere `shell/Dialog`'u kullanır. Baskı önizlemesi pencere değil, panonun yerini alan görünümdür.
  - **C9** Renk yalnızca token'lardan gelir (`ui/print` dahil; hex yalnızca `tokens.css`'te) ve tek başına anlam taşımaz: retro düz `R` (`showsRetro`: Rahu ve Ketu'da hiç), yanma `c`, vargottama altı çizili, lagna kalın çizgi; kâğıtta işaretlerin açıklaması basılır.
  - **C10** Üretilen dosyalar (`bindings.ts`, `delhi.json`) bu planda değişmez; elle düzenlenmez.
  - **C11** Görsel PNG temelleri yereldir ve yalnızca darwin'dir. Mevcut altı temel ve `ChartSvg` anlık görüntüleri değişmez (Task 3 ve 6 doğrular); dört yeni temel Task 6'da üretilir.
  - **C12** Yeni `CommandError.code` yoktur (`print_failed` zaten çevrilidir).
  - **C13** Yasaklı ad taraması her commit'te; depo dışındaki ilerleme notları ve `.superpowers/` hiçbir zaman `git add` edilmez (her commit yalnızca `app/`, `scripts/` ve `docs/` altını değiştirir).
  - **C14** Plan 07'nin gün doğumu toleransı değişmez.
  - **C15** `alert`, `confirm`, `prompt` çağrılmaz. `window.print()` de çağrılmaz: yazdırma `api.printWindow()` iledir.
  - **C16** ⌘ kısayolları metin girişinden de çalışır; `⌘P` metin düzenleme akoru değildir.
  - **C17** Otomatik kayıtların boşaltılması değişmez.
- **Erişilebilirlik:** WCAG AA; her işlem klavyeyle yapılır; odak halkası 2 px `--ink`; odak `body`'ye düşmez (önizleme açılınca "Yazdır…" düğmesine, kapanınca kararlı bir hedefe); devre dışı denetim yerel `disabled` değil `aria-disabled`'dır ve nedeni `aria-describedby` ile bağlı, görünür bir metindir.
- Commit mesajları İngilizcedir ve deponun `Co-Authored-By:` satırıyla biter (yürütücünün oturum yönergesindeki satır). Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Kod blokları `ecf7ed5` ağacına göre yazılmış ve bu plan yazılırken deponun bir kopyasında koşulmuştur (Self-Review madde 5). Yeni dosyalar tam içerikle verilir. Değişen dosyalar `diff` bloğuyla verilir: blok bir dosyaya konup depo kökünde `git apply <dosya>` ile ya da elle uygulanır; bağlam tutmazsa önceki görevin adımı denetlenir, elle uydurma yapılmaz.

## Dosya Yapısı (bu plan)

```
app/ui/
  theme/{tokens.css, tokens.test.ts}          .paper bloğu (baskı paleti, sınıfla)
  theme/no-hex.test.ts                         spike istisnaları silinir
  print/options.ts (+test)                     (yeni) baskı seçenekleri, localStorage
  print/model.ts (+test)                       (yeni) PrintDocument: sayfa sayfa düz veri
  print/PrintDocumentView.tsx (+test)          (yeni) A4 sayfalar
  print/print.css                              (yeni) @page, kâğıt, tablolar, araç çubuğu, @media print
  print/PrintView.tsx (+test)                  (yeni) önizleme ve araç çubuğu
  panels/grahaCells.ts                         (yeni) graha hücre metni (panel ve baskı ortak)
  panels/{GrahaTablePanel.tsx, BirthInfoPanel.tsx}   ortak metinleri kullanır / dışa aktarır
  chart/{ChartSvg.tsx, ChartSvg.test.tsx}      interactive={false}
  shell/{Shell.tsx, TopBar.tsx, shell.css}     önizleme durumu, ⌘P, yazdırma hatası, baskıda blok akışı
  shell/sidebarPref.ts (+test)                 (yeni) kenar çubuğu açık/kapalı kaydı
  shortcuts.ts (+test), palette/{items.ts, Palette.tsx}
  i18n/{tr,en,fr}.ts                           print.*, shortcuts.print, palette.command.print
  App.test.tsx
  PrintSpike.tsx, PrintSpike.test.tsx, print.css, shell/PrintView.tsx, shell/PrintView.test.tsx   (silinir)
app/e2e/{ipc.ts, desk.spec.ts, visual.spec.ts}
app/e2e/visual.spec.ts-snapshots/print-{cover,charts,dasha,grahas}-visual-darwin.png   (yeni)
scripts/print-probe.sh, scripts/print-probe/probe.swift                                 (yeni)
docs/spikes/2026-11-24-wkwebview-print.md                                               (bulgular)
docs/superpowers/specs/{2026-11-24-jyotish-desk-v1-design.md, 2026-11-24-ui-design.md} (düzeltmeler)
```

### Kapsam

| Madde | Yer |
|---|---|
| v1 spec §3/§4/§11 M7: Print ve PDF; UI spec §7 dört A4 düzen | Task 2, 3, 4 |
| Baskı her zaman açık tema, tek renk, siyah-beyazda okunur (UI spec §2.1, §7) | Task 1 (`.paper`), Task 3, Task 6 |
| Retro `R` (Rahu/Ketu'da yok), yanma `c`, vargottama altı çizili (UI spec §2.3) | Task 2, 3 |
| M0 baskı denemesinin kapanması, Typst kararı (v1 spec §4) | D2, Task 7 |
| Baskı düzenleri soyut (v1 spec §12) | D8, Task 2 |
| Plan 04 notları: `PrintSpike` ve hex istisnaları; baskıda lagna dolgusu; önizlemede `⌘1…9` | Task 4; Task 3; Task 5 |
| Plan 10 kapsam dışı 16 (kenar çubuğu durumu) | Task 5 (D11) |
| Plan 10 kapsam dışı 15, 16'nın boyutlandırma yarısı | Bu planın kapsamadıkları |
| Son doğrulama ve elle denetim | Task 8 |

---

### Task 1: Kâğıt paleti (`.paper`) ve baskı seçenekleri

**Files:**
- Modify: `app/ui/theme/tokens.css`, `app/ui/theme/tokens.test.ts`
- Create: `app/ui/print/options.ts`
- Test: `app/ui/print/options.test.ts`

**Interfaces:**
- Consumes: `isChartStyle`, `ChartStyle` (`app/ui/chart/types.ts`).
- Produces: CSS sınıfı `.paper`; `PRINT_PAGES`, `type PrintPageId = "cover" | "charts" | "dasha" | "grahas"`, `interface PrintOptions { pages: PrintPageId[]; style: ChartStyle }`, `PRINT_STORAGE_KEY = "jyotish-desk.print.v1"`, `defaultPrintOptions(): PrintOptions`, `parsePrintOptions(raw: unknown): PrintOptions`, `togglePage(pages, page): PrintPageId[]`, `loadPrintOptions(storage?): PrintOptions`, `savePrintOptions(options, storage?): void`.

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/theme/tokens.test.ts` ve `app/ui/theme/tokens.css` için aşağıdaki diff'in yalnızca test dosyası bölümünü uygula; `app/ui/print/options.test.ts`'i oluştur:

```ts
import { defaultPrintOptions, loadPrintOptions, parsePrintOptions, PRINT_STORAGE_KEY, savePrintOptions, togglePage } from "./options";

beforeEach(() => localStorage.clear());

test("defaults: all four pages in spec order, north style", () => {
  expect(defaultPrintOptions()).toEqual({ pages: ["cover", "charts", "dasha", "grahas"], style: "north" });
});

test("parsing keeps known pages in spec order without repeats and falls back field by field", () => {
  expect(parsePrintOptions({ pages: ["grahas", "cover", "grahas", "appendix", 3], style: "south" })).toEqual({ pages: ["cover", "grahas"], style: "south" });
  expect(parsePrintOptions({ pages: [], style: "hexagon" })).toEqual(defaultPrintOptions());
  expect(parsePrintOptions({ pages: "cover", style: "circle" })).toEqual({ ...defaultPrintOptions(), style: "circle" });
  for (const raw of [null, undefined, 7, "x", []]) expect(parsePrintOptions(raw)).toEqual(defaultPrintOptions());
});

test("toggling removes and re-adds a page in spec order; the last page stays", () => {
  expect(togglePage(["cover", "charts", "dasha", "grahas"], "charts")).toEqual(["cover", "dasha", "grahas"]);
  expect(togglePage(["cover", "grahas"], "dasha")).toEqual(["cover", "dasha", "grahas"]);
  expect(togglePage(["dasha"], "dasha")).toEqual(["dasha"]);
});

test("options round-trip through storage; unreadable storage gives the defaults", () => {
  savePrintOptions({ pages: ["charts"], style: "circle" });
  expect(JSON.parse(localStorage.getItem(PRINT_STORAGE_KEY)!)).toEqual({ pages: ["charts"], style: "circle" });
  expect(loadPrintOptions()).toEqual({ pages: ["charts"], style: "circle" });
  localStorage.setItem(PRINT_STORAGE_KEY, "{not json");
  expect(loadPrintOptions()).toEqual(defaultPrintOptions());
  localStorage.removeItem(PRINT_STORAGE_KEY);
  expect(loadPrintOptions()).toEqual(defaultPrintOptions());
});

test("a storage that throws neither breaks loading nor saving", () => {
  const broken = {
    getItem: () => {
      throw new Error("denied");
    },
    setItem: () => {
      throw new Error("full");
    },
  } as unknown as Storage;
  expect(loadPrintOptions(broken)).toEqual(defaultPrintOptions());
  expect(() => savePrintOptions(defaultPrintOptions(), broken)).not.toThrow();
});
```

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/print/options.test.ts ui/theme/tokens.test.ts`
Expected: FAIL — `options.test.ts` `./options` modülünü bulamaz; `tokens.test.ts`'te "the paper class carries the print palette to the screen preview" `.paper {` işaretini bulamadığı için düşer.

- [ ] **Step 3: Uygula**

`tokens.css` ve `tokens.test.ts` (test bölümü Step 1'de uygulandıysa yalnızca CSS bölümü):

```diff
diff --git a/app/ui/theme/tokens.css b/app/ui/theme/tokens.css
index b5c37a3..b2d25ef 100644
--- a/app/ui/theme/tokens.css
+++ b/app/ui/theme/tokens.css
@@ -55,6 +55,24 @@
   }
 }
 
+/* Kâğıt: baskı önizlemesi ekranda da baskı paletini kullanır (koyu temada da). Değerler
+   yukarıdaki @media print bloğuyla aynıdır; tokens.test.ts ikisini karşılaştırır. */
+.paper {
+  color-scheme: light;
+  --bg: #FFFFFF;
+  --panel: #FFFFFF;
+  --line: #534AB7;
+  --ink: #534AB7;
+  --ink-strong: #534AB7;
+  --select: #FFFFFF;
+  --text: #2C2C2A;
+  --text-2: #2C2C2A;
+  --warn: #2C2C2A;
+  --danger: #2C2C2A;
+  color: var(--text);
+  background: var(--panel);
+}
+
 html {
   font-family: var(--font-ui);
   font-size: var(--fs-body);
diff --git a/app/ui/theme/tokens.test.ts b/app/ui/theme/tokens.test.ts
index f1946c7..9954e87 100644
--- a/app/ui/theme/tokens.test.ts
+++ b/app/ui/theme/tokens.test.ts
@@ -69,6 +69,12 @@ test("print forces the light palette with only ink and text", () => {
   expect(print["--text"]).toBe(LIGHT["--text"]);
 });
 
+test("the paper class carries the print palette to the screen preview", () => {
+  const paper = colorTokens(".paper {");
+  expect(Object.keys(paper)).toHaveLength(10);
+  expect(paper).toEqual(colorTokens("@media print"));
+});
+
 test("typography: system font, two weights, four sizes, tabular numbers, 2px ink focus ring", () => {
   expect(css).toContain("-apple-system");
   expect(css).toMatch(/--fw-regular:\s*400;/);
```

`app/ui/print/options.ts`:

```ts
// Baskı seçenekleri (Plan 12 D5): hangi sayfalar basılır ve harita biçimi. Arayüz tercihidir
// (tema ve dil gibi `localStorage`'da durur); danışan verisi içermez. Okuma toleranslıdır:
// bozuk ya da eski kayıt varsayılana döner, bilinmeyen sayfa kimlikleri atlanır.
import { isChartStyle, type ChartStyle } from "../chart/types";

/** UI spec §7 sırası: kapak, D1+D9, dasa, graha tablosu. */
export const PRINT_PAGES = ["cover", "charts", "dasha", "grahas"] as const;
export type PrintPageId = (typeof PRINT_PAGES)[number];

export interface PrintOptions {
  /** `PRINT_PAGES` sırasında, tekrarsız, en az bir sayfa. */
  pages: PrintPageId[];
  style: ChartStyle;
}

export const PRINT_STORAGE_KEY = "jyotish-desk.print.v1";

export function defaultPrintOptions(): PrintOptions {
  return { pages: [...PRINT_PAGES], style: "north" };
}

function isPrintPageId(v: unknown): v is PrintPageId {
  return typeof v === "string" && (PRINT_PAGES as readonly string[]).includes(v);
}

export function parsePrintOptions(raw: unknown): PrintOptions {
  const fallback = defaultPrintOptions();
  if (typeof raw !== "object" || raw === null) return fallback;
  const { pages, style } = raw as { pages?: unknown; style?: unknown };
  const chosen = new Set(Array.isArray(pages) ? pages.filter(isPrintPageId) : []);
  return {
    pages: chosen.size === 0 ? fallback.pages : PRINT_PAGES.filter((p) => chosen.has(p)),
    style: isChartStyle(style) ? style : fallback.style,
  };
}

/** Sayfayı aç/kapat; son sayfa kapatılamaz (boş belge basılmaz). */
export function togglePage(pages: readonly PrintPageId[], page: PrintPageId): PrintPageId[] {
  if (pages.includes(page)) return pages.length === 1 ? [...pages] : pages.filter((p) => p !== page);
  return PRINT_PAGES.filter((p) => p === page || pages.includes(p));
}

export function loadPrintOptions(storage: Storage = window.localStorage): PrintOptions {
  try {
    return parsePrintOptions(JSON.parse(storage.getItem(PRINT_STORAGE_KEY) ?? "null"));
  } catch {
    return defaultPrintOptions();
  }
}

/** Yazılamazsa (depolama kapalı ya da dolu) seçenek yalnızca bu oturumda geçerli olur. */
export function savePrintOptions(options: PrintOptions, storage: Storage = window.localStorage): void {
  try {
    storage.setItem(PRINT_STORAGE_KEY, JSON.stringify(options));
  } catch {
    // Tercih kaybı zararsızdır ve loglanmaz (C1: log izin listesi büyümez).
  }
}
```

`options.ts`'teki yorumlarda "console" sözcüğünün ardından nokta gelmemelidir: `no-console.test.ts` yorumları da tarar.

- [ ] **Step 4: Testlerin geçtiğini gör**

Run: `pnpm --dir app test && pnpm --dir app build`
Expected: `Test Files  55 passed (55)`, `Tests  467 passed (467)`; build `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/theme/tokens.css app/ui/theme/tokens.test.ts app/ui/print/options.ts app/ui/print/options.test.ts
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): paper palette class and stored print options"
```
Expected: iki `grep` çıktısız (çıkış kodu 1 beklenir); commit mesajı da aynı desenle eşleşmez.

---

### Task 2: Baskı belgesi modeli; panel ve kâğıt için ortak hücre ve satır metinleri

**Files:**
- Create: `app/ui/panels/grahaCells.ts`, `app/ui/print/model.ts`
- Modify: `app/ui/panels/GrahaTablePanel.tsx`, `app/ui/panels/BirthInfoPanel.tsx`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: `app/ui/print/model.test.ts` (mevcut `panels/*.test.tsx` yeniden düzenlemeyi korur)

**Interfaces:**
- Consumes: `PrintOptions`, `PrintPageId` (Task 1); `chartDataFromChart`, `VARGA_NAMES` (`chart/adapter.ts`); `childrenOf`, `formatJdDate` (`dasha/tree.ts`); `chartLabels` (`panels/labels.ts`); `timeUnknown` (`session/birth.ts`); `ayanamsaKey` (`settings/model.ts`); `formatDegree` (`panels/format.ts`); `GRAHA_COLUMNS`, `GrahaColumn` (`panels/grahaColumns.ts`).
- Produces:
  - `panels/grahaCells.ts`: `NUMERIC_COLUMNS`, `APPROX_COLUMNS: ReadonlySet<GrahaColumn>`, `DASH`, `grahaCellText(t: TFunction, g: GrahaPosition, column: GrahaColumn): string`, `lagnaCellText(t: TFunction, l: Lagna, column: GrahaColumn): string`.
  - `panels/BirthInfoPanel.tsx`: `birthInfoRows(t: TFunction, opened: OpenedClient): [string, string][]`.
  - `print/model.ts`: `PrintRow`, `CoverPage`, `PrintChart`, `ChartsPage`, `DashaBlock`, `DashaPage`, `GrahasPage`, `PrintPage`, `PrintDocument { client: string; chart: string; title: string; pages: PrintPage[] }`, `PRINT_VARGAS`, `buildPrintDocument(t: TFunction, opened: OpenedClient, options: PrintOptions): PrintDocument`. Her sayfada `id: PrintPageId` ve `heading: string` vardır.
  - i18n: `print.page.{cover,charts,dasha,grahas}`, `print.legend`, `print.credit` (`{{app}}`, `{{engine}}`, `{{swe}}`), `print.dasha.{lord,start,end}`.

- [ ] **Step 1: Başarısız testi yaz**

`app/ui/print/model.test.ts`:

```ts
import { i18n } from "../i18n";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { buildPrintDocument, type ChartsPage, type CoverPage, type DashaPage, type GrahasPage } from "./model";
import { defaultPrintOptions } from "./options";

const t = i18n.getFixedT("tr");
const build = (opened = fixture().opened, options = defaultPrintOptions()) => buildPrintDocument(t, opened, options);
const page = <P extends { id: string }>(doc: { pages: { id: string }[] }, id: P["id"]) => doc.pages.find((p) => p.id === id) as P;

test("the document names the client and the chart and builds only the chosen pages, in order", () => {
  const doc = build();
  expect([doc.client, doc.chart, doc.title]).toEqual(["Örnek Danışan", "Doğum haritası", "Örnek Danışan – Doğum haritası"]);
  expect(doc.pages.map((p) => p.id)).toEqual(["cover", "charts", "dasha", "grahas"]);
  expect(build(fixture().opened, { pages: ["dasha"], style: "north" }).pages.map((p) => p.id)).toEqual(["dasha"]);
  const labelled = fixture().opened;
  labelled.chart_record.label = "Evlilik";
  expect(build(labelled).title).toBe("Örnek Danışan – Evlilik");
});

test("cover: stored birth data, the settings the chart was computed with and the engine versions", () => {
  const cover = page<CoverPage>(build(), "cover");
  expect(cover.heading).toBe("Örnek Danışan");
  expect(cover.birth).toEqual(expect.arrayContaining([["Tarih", "2000-01-01"], ["Saat", "17:30"], ["Yer", "New Delhi, Delhi, India"], ["UTC anı", "2000-01-01 12:00:00 UTC"]]));
  expect(cover.settings).toEqual([
    ["Zodyak", "Sidereal"],
    ["Ayanamsa", "Lahiri 23°51′"],
    ["Düğüm", "Ortalama"],
    ["Ev sistemi", "Tam burç"],
    ["Dasa yılı", "365.25 gün"],
  ]);
  expect(cover.warnings).toEqual([]);
  expect(cover.credit).toBe("Jyotish Desk 0.1.0 · Swiss Ephemeris 2.10.03");
});

test("cover warnings carry the name and the explanation as text", () => {
  const opened = unknownTimeOpened();
  opened.chart.meta.warnings = ["LowPrecision"];
  expect(page<CoverPage>(build(opened), "cover").warnings).toEqual([
    { name: "Saat bilinmiyor", hint: "Doğum saati bilinmiyor; öğlen 12:00 alındı." },
    { name: "Düşük hassasiyet", hint: "Güneş veya Ay yedek ephemeris ile hesaplandı." },
  ]);
});

test("charts: D1 and D9 in the chosen style with the lagna Rust sent; no lagna when the time is unknown", () => {
  const charts = page<ChartsPage>(build(fixture().opened, { pages: ["charts"], style: "south" }), "charts");
  expect(charts.style).toBe("south");
  expect(charts.charts.map((c) => c.caption)).toEqual(["D1 · Rasi", "D9 · Navamsa"]);
  expect(charts.charts[0].data?.ascSign).toBe(2);
  expect(charts.charts[0].data?.planets).toHaveLength(9);
  expect(charts.charts[0].labels.title).toBe("D1 haritası");
  expect(charts.note).toBeNull();
  const unknown = page<ChartsPage>(build(unknownTimeOpened()), "charts");
  expect(unknown.charts.map((c) => c.data?.ascSign)).toEqual([null, null]);
  expect(unknown.note).toContain("Doğum saati bilinmiyor");
});

test("dasha: nine mahadashas with nine antardashas each, dated in the birth offset and contiguous", () => {
  const dasha = page<DashaPage>(build(), "dasha");
  expect(dasha.blocks.map((b) => b.lord)).toEqual(["Rahu", "Jüpiter", "Satürn", "Merkür", "Ketu", "Venüs", "Güneş", "Ay", "Mars"]);
  expect(dasha.blocks[0]).toMatchObject({ start: "1982-09-19", end: "2000-09-18" });
  const jupiter = dasha.blocks[1];
  expect(jupiter.rows).toHaveLength(9);
  expect(jupiter.rows[0]).toEqual({ lord: "Jüpiter", start: "2000-09-18", end: "2002-11-07" });
  expect(jupiter.rows[8]).toEqual({ lord: "Rahu", start: "2014-04-26", end: "2016-09-18" });
  for (const block of dasha.blocks) {
    expect(block.rows).toHaveLength(9);
    expect(block.rows[0].start).toBe(block.start);
    expect(block.rows[8].end).toBe(block.end);
    block.rows.slice(1).forEach((row, i) => expect(row.start).toBe(block.rows[i].end));
  }
  expect(dasha.note).toBeNull();
  expect(page<DashaPage>(build(unknownTimeOpened()), "dasha").note).toBe("Doğum saati bilinmiyor: dasa tarihleri yaklaşıktır (≈).");
});

test("graha table: every column, lagna first, plain R for retrograde grahas but never for the nodes", () => {
  const grahas = page<GrahasPage>(build(), "grahas");
  expect(grahas.columns.map((c) => c.id)).toEqual(["sign", "degree", "lon", "lat", "speed", "retro", "nakshatra", "pada", "combust", "dignity", "house"]);
  expect(grahas.rows.map((r) => r.key)).toEqual(["Lagna", "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"]);
  const cell = (key: string, column: string) => grahas.rows.find((r) => r.key === key)!.cells[grahas.columns.findIndex((c) => c.id === column)];
  expect(cell("Saturn", "retro")).toBe("R");
  // Fikstürde Rahu ve Ketu'nun `retrograde` alanı doğrudur; işaret yine de basılmaz.
  expect(fixture().opened.chart.grahas.filter((g) => g.retrograde).map((g) => g.graha)).toEqual(["Saturn", "Rahu", "Ketu"]);
  expect([cell("Rahu", "retro"), cell("Ketu", "retro"), cell("Sun", "retro")]).toEqual(["", "", ""]);
  expect([cell("Sun", "sign"), cell("Sun", "degree"), cell("Sun", "nakshatra"), cell("Sun", "house")]).toEqual(["Yay", "16°30′", "Purva Ashadha", "7"]);
  expect([cell("Mercury", "combust"), cell("Saturn", "dignity"), cell("Rahu", "dignity")]).toEqual(["yanık", "Düşük", "—"]);
  expect([cell("Lagna", "sign"), cell("Lagna", "degree"), cell("Lagna", "house"), cell("Lagna", "retro")]).toEqual(["İki", "16°20′", "1", ""]);
});

test("graha table with an unknown birth time: no lagna row, no house column, the Moon marked approximate", () => {
  const grahas = page<GrahasPage>(build(unknownTimeOpened()), "grahas");
  expect(grahas.columns.map((c) => c.id)).not.toContain("house");
  expect(grahas.rows.map((r) => r.key)).not.toContain("Lagna");
  const moon = grahas.rows.find((r) => r.key === "Moon")!.cells;
  const at = (column: string) => grahas.columns.findIndex((c) => c.id === column);
  expect([moon[at("sign")], moon[at("degree")], moon[at("nakshatra")], moon[at("pada")]]).toEqual(["≈ Ter", "≈ 19°28′", "≈ Swati", "≈ 4"]);
  expect(moon[at("speed")]).toBe("+12.021");
  expect(grahas.rows.find((r) => r.key === "Sun")!.cells[at("sign")]).toBe("Yay");
});
```

- [ ] **Step 2: Testin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/print/model.test.ts`
Expected: FAIL — `./model` modülü yok.

- [ ] **Step 3: Ortak hücre metinlerini çıkar**

`app/ui/panels/grahaCells.ts`:

```ts
// Graha tablosu hücrelerinin metni. Pano paneli (GrahaTablePanel) ve baskı belgesi
// (print/model.ts) aynı metni kullanır; değerler Rust'tan hazır gelir, burada yalnızca biçimlenir.
import type { TFunction } from "i18next";
import type { GrahaPosition, Lagna } from "../bindings";
import { signIndex } from "../chart/adapter";
import { RETRO_MARK, showsRetro } from "../chart/types";
import { formatDegree, formatSignedDegree, formatSpeed } from "./format";
import type { GrahaColumn } from "./grahaColumns";
import { nakshatraName } from "./labels";

export const NUMERIC_COLUMNS: ReadonlySet<GrahaColumn> = new Set(["degree", "lon", "lat", "speed", "pada", "house"]);
/** Doğum saati bilinmiyorsa Ay'da "≈" ile işaretlenen (saate bağlı) sütunlar. */
export const APPROX_COLUMNS: ReadonlySet<GrahaColumn> = new Set(["sign", "degree", "lon", "nakshatra", "pada"]);
export const DASH = "—";

function deg(v: number | null): string {
  return v === null ? DASH : formatDegree(v);
}

/** Retro sütunu düz "R"dir ve Rahu ile Ketu'da boştur (`showsRetro`). */
export function grahaCellText(t: TFunction, g: GrahaPosition, column: GrahaColumn): string {
  switch (column) {
    case "sign":
      return t(`sign.${signIndex(g.sign)}`);
    case "degree":
      return deg(g.deg_in_sign);
    case "lon":
      return deg(g.lon);
    case "lat":
      return g.lat === null ? DASH : formatSignedDegree(g.lat);
    case "speed":
      return g.speed === null ? DASH : formatSpeed(g.speed);
    case "retro":
      return showsRetro(g.graha, g.retrograde) ? RETRO_MARK : "";
    case "nakshatra":
      return nakshatraName(g.nakshatra);
    case "pada":
      return String(g.pada);
    case "combust":
      return g.combust ? t("chart.combust") : "";
    case "dignity":
      return g.dignity === null ? DASH : t(`dignity.${g.dignity}`);
    case "house":
      return String(g.house);
  }
}

export function lagnaCellText(t: TFunction, l: Lagna, column: GrahaColumn): string {
  switch (column) {
    case "sign":
      return t(`sign.${signIndex(l.sign)}`);
    case "degree":
      return deg(l.deg_in_sign);
    case "lon":
      return deg(l.lon);
    case "nakshatra":
      return nakshatraName(l.nakshatra);
    case "pada":
      return String(l.pada);
    case "house":
      return "1";
    default:
      return "";
  }
}
```

`GrahaTablePanel.tsx` hücre metnini buradan alır ve yalnızca erişilebilirlik sarmalayıcılarını ekler; `BirthInfoPanel.tsx` satır listesini `birthInfoRows` olarak dışa aktarır; üç dil dosyasına `print` bloğu eklenir:

```diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index ac68d06..4929ddb 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -457,6 +457,12 @@ export const en: Messages = {
     body: "Start by entering the client's birth data.",
     action: "New chart",
   },
+  print: {
+    page: { cover: "Cover", charts: "Charts: D1 and D9", dasha: "Vimshottari dasha", grahas: "Graha table" },
+    legend: "R: retrograde · c: combust · underlined: vargottama",
+    credit: "{{app}} {{engine}} · Swiss Ephemeris {{swe}}",
+    dasha: { lord: "Antardasha", start: "Start", end: "End" },
+  },
   transit: {
     date: "Transit moment",
     dateLabel: "Date",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 60e9770..ced8daf 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -457,6 +457,12 @@ export const fr: Messages = {
     body: "Commencez par saisir les données de naissance du consultant.",
     action: "Nouvelle carte",
   },
+  print: {
+    page: { cover: "Couverture", charts: "Cartes : D1 et D9", dasha: "Dasha Vimshottari", grahas: "Tableau des grahas" },
+    legend: "R : rétrograde · c : combuste · souligné : vargottama",
+    credit: "{{app}} {{engine}} · Swiss Ephemeris {{swe}}",
+    dasha: { lord: "Antardasha", start: "Début", end: "Fin" },
+  },
   transit: {
     date: "Moment du transit",
     dateLabel: "Date",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 9bb8d3a..fb04392 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -456,6 +456,12 @@ export const tr = {
     body: "Danışanın doğum bilgisini girerek başlayın.",
     action: "Yeni harita",
   },
+  print: {
+    page: { cover: "Kapak", charts: "Haritalar: D1 ve D9", dasha: "Vimshottari dasa", grahas: "Graha tablosu" },
+    legend: "R: retro · c: yanık · altı çizili: vargottama",
+    credit: "{{app}} {{engine}} · Swiss Ephemeris {{swe}}",
+    dasha: { lord: "Antardasa", start: "Başlangıç", end: "Bitiş" },
+  },
   transit: {
     date: "Transit anı",
     dateLabel: "Tarih",
diff --git a/app/ui/panels/BirthInfoPanel.tsx b/app/ui/panels/BirthInfoPanel.tsx
index 28eb82f..1b1a9ad 100644
--- a/app/ui/panels/BirthInfoPanel.tsx
+++ b/app/ui/panels/BirthInfoPanel.tsx
@@ -39,12 +39,10 @@ export function BirthInfoPanel(_props: PanelProps) {
   return <BirthInfo opened={opened} />;
 }
 
-function BirthInfo({ opened }: { opened: OpenedClient }) {
-  const { t } = useTranslation();
-  const id = useId();
+/** Doğum bilgisi satırları (etiket, değer); panel ve baskı kapağı (print/model.ts) aynı listeyi kullanır. */
+export function birthInfoRows(t: TFunction, opened: OpenedClient): [string, string][] {
   const b = opened.chart_record.birth;
   const r = opened.chart_record.resolved;
-  const warnings: string[] = [...r.warnings, ...opened.chart.meta.warnings];
   const rows: [string, string][] = [
     [t("birthInfo.date"), b.date],
     [t("birthInfo.time"), timeText(t, b)],
@@ -59,6 +57,14 @@ function BirthInfo({ opened }: { opened: OpenedClient }) {
     [t("birthInfo.kind"), t(`chart.kind.${opened.chart_record.kind}`)],
   ];
   if (b.notes.trim() !== "") rows.push([t("birthInfo.source"), b.notes]);
+  return rows;
+}
+
+function BirthInfo({ opened }: { opened: OpenedClient }) {
+  const { t } = useTranslation();
+  const id = useId();
+  const warnings: string[] = [...opened.chart_record.resolved.warnings, ...opened.chart.meta.warnings];
+  const rows = birthInfoRows(t, opened);
   return (
     <section className="birth-info" aria-label={t("panel.birth_info")}>
       <dl className="birth-info-list">
diff --git a/app/ui/panels/GrahaTablePanel.tsx b/app/ui/panels/GrahaTablePanel.tsx
index 6256043..bcf5b40 100644
--- a/app/ui/panels/GrahaTablePanel.tsx
+++ b/app/ui/panels/GrahaTablePanel.tsx
@@ -1,27 +1,17 @@
 import type { TFunction } from "i18next";
 import type { ReactNode } from "react";
 import { useTranslation } from "react-i18next";
-import type { GrahaPosition, Lagna } from "../bindings";
-import { signIndex } from "../chart/adapter";
-import { RETRO_MARK, showsRetro } from "../chart/types";
+import type { GrahaPosition } from "../bindings";
 import { useHighlight } from "../highlight/store";
 import { sameTarget, type HighlightTarget } from "../highlight/target";
 import { useSession } from "../session/store";
 import { useLayoutStore } from "../view/store";
 import { timeUnknown } from "../session/birth";
-import { formatDegree, formatSignedDegree, formatSpeed } from "./format";
+import { APPROX_COLUMNS, grahaCellText, lagnaCellText, NUMERIC_COLUMNS as NUMERIC } from "./grahaCells";
 import { GRAHA_COLUMNS, grahaColumns, toggleColumn, type GrahaColumn } from "./grahaColumns";
-import { nakshatraName } from "./labels";
 import "./panels.css";
 import type { PanelProps } from "./types";
 
-const NUMERIC: ReadonlySet<GrahaColumn> = new Set(["degree", "lon", "lat", "speed", "pada", "house"]);
-const dash = "—";
-
-function deg(v: number | null): string {
-  return v === null ? dash : formatDegree(v);
-}
-
 /** Yaklaşık değer: "≈" işareti (fare için `title`) ve ekran okuyucu için bir kez okunan metin. */
 function Approx({ on, label, children }: { on: boolean; label: string; children: ReactNode }) {
   if (!on) return <>{children}</>;
@@ -36,62 +26,23 @@ function Approx({ on, label, children }: { on: boolean; label: string; children:
   );
 }
 
+/** Hücre metni `grahaCells.ts`'ten; burada yalnızca erişilebilirlik sarmalayıcıları eklenir. */
 function grahaCell(t: TFunction, g: GrahaPosition, column: GrahaColumn, approx: boolean): ReactNode {
-  const a = (node: ReactNode) => (
+  const text = grahaCellText(t, g, column);
+  if (column === "retro" && text !== "") {
+    return (
+      <>
+        <span aria-hidden="true">{text}</span>
+        <span className="visually-hidden">{t("chart.retro")}</span>
+      </>
+    );
+  }
+  if (!APPROX_COLUMNS.has(column)) return text;
+  return (
     <Approx on={approx} label={t("table.approx")}>
-      {node}
+      {text}
     </Approx>
   );
-  switch (column) {
-    case "sign":
-      return a(t(`sign.${signIndex(g.sign)}`));
-    case "degree":
-      return a(deg(g.deg_in_sign));
-    case "lon":
-      return a(deg(g.lon));
-    case "lat":
-      return g.lat === null ? dash : formatSignedDegree(g.lat);
-    case "speed":
-      return g.speed === null ? dash : formatSpeed(g.speed);
-    case "retro":
-      return showsRetro(g.graha, g.retrograde) ? (
-        <>
-          <span aria-hidden="true">{RETRO_MARK}</span>
-          <span className="visually-hidden">{t("chart.retro")}</span>
-        </>
-      ) : (
-        ""
-      );
-    case "nakshatra":
-      return a(nakshatraName(g.nakshatra));
-    case "pada":
-      return a(String(g.pada));
-    case "combust":
-      return g.combust ? t("chart.combust") : "";
-    case "dignity":
-      return g.dignity === null ? dash : t(`dignity.${g.dignity}`);
-    case "house":
-      return String(g.house);
-  }
-}
-
-function lagnaCell(t: TFunction, l: Lagna, column: GrahaColumn): ReactNode {
-  switch (column) {
-    case "sign":
-      return t(`sign.${signIndex(l.sign)}`);
-    case "degree":
-      return deg(l.deg_in_sign);
-    case "lon":
-      return deg(l.lon);
-    case "nakshatra":
-      return nakshatraName(l.nakshatra);
-    case "pada":
-      return String(l.pada);
-    case "house":
-      return "1";
-    default:
-      return "";
-  }
 }
 
 /**
@@ -149,7 +100,7 @@ export function GrahaTablePanel({ panel }: PanelProps) {
               <th scope="row">{t("table.lagnaRow")}</th>
               {columns.map((c) => (
                 <td key={c} className={NUMERIC.has(c) ? "num" : undefined}>
-                  {lagnaCell(t, opened.chart.lagna, c)}
+                  {lagnaCellText(t, opened.chart.lagna, c)}
                 </td>
               ))}
             </tr>
```

- [ ] **Step 4: Paneller değişmedi**

Run: `pnpm --dir app exec vitest run ui/panels ui/i18n`
Expected: PASS (`ui/panels` 7 dosya / 39 test; `i18n.test.ts` üç dilde aynı anahtarları bulur). Bir panel testi düşerse yeniden düzenleme davranış değiştirmiştir: diff'i yeniden uygula, testi değiştirme.

- [ ] **Step 5: Modeli yaz**

`app/ui/print/model.ts`:

```ts
// Baskı belgesi (Plan 12 D4): açık haritadan sayfa sayfa düz veri. Bütün metinler burada
// çözülür; çizen katman (PrintDocumentView) yalnızca dizer. Böylece düzen başka bir çıktı
// yoluna taşınabilir (v1 spec §12: baskı düzenleri soyut tutulur). Hesap yoktur (C4): burç,
// derece, retro, dasa dönemleri ve uyarılar Rust'tan gelir.
import type { TFunction } from "i18next";
import type { OpenedClient, Varga } from "../bindings";
import { chartDataFromChart, VARGA_NAMES } from "../chart/adapter";
import type { ChartData, ChartLabels, ChartStyle } from "../chart/types";
import { childrenOf, formatJdDate } from "../dasha/tree";
import { birthInfoRows } from "../panels/BirthInfoPanel";
import { formatDegree } from "../panels/format";
import { APPROX_COLUMNS, grahaCellText, lagnaCellText, NUMERIC_COLUMNS } from "../panels/grahaCells";
import { GRAHA_COLUMNS, type GrahaColumn } from "../panels/grahaColumns";
import { chartLabels } from "../panels/labels";
import { timeUnknown } from "../session/birth";
import { ayanamsaKey } from "../settings/model";
import type { PrintOptions, PrintPageId } from "./options";

export type PrintRow = readonly [label: string, value: string];

interface PageBase {
  id: PrintPageId;
  heading: string;
}

export interface CoverPage extends PageBase {
  id: "cover";
  birth: PrintRow[];
  settingsHeading: string;
  settings: PrintRow[];
  warningsHeading: string;
  warnings: { name: string; hint: string }[];
  /** Uygulama ve hesap motoru sürümleri. */
  credit: string;
}

export interface PrintChart {
  varga: Varga;
  caption: string;
  /** Varga haritada yoksa `null`; `missing` metni basılır. */
  data: ChartData | null;
  labels: ChartLabels;
  missing: string;
}

export interface ChartsPage extends PageBase {
  id: "charts";
  style: ChartStyle;
  charts: PrintChart[];
  /** Doğum saati bilinmiyorsa lagna ve evlerin neden çizilmediği. */
  note: string | null;
  /** Harita içindeki işaretlerin açıklaması (R, c, altı çizili). */
  legend: string;
}

export interface DashaBlock {
  lord: string;
  start: string;
  end: string;
  rows: { lord: string; start: string; end: string }[];
}

export interface DashaPage extends PageBase {
  id: "dasha";
  note: string | null;
  columns: { lord: string; start: string; end: string };
  blocks: DashaBlock[];
}

export interface GrahasPage extends PageBase {
  id: "grahas";
  rowHeader: string;
  columns: { id: GrahaColumn; label: string; numeric: boolean }[];
  rows: { key: string; name: string; cells: string[] }[];
}

export type PrintPage = CoverPage | ChartsPage | DashaPage | GrahasPage;

export interface PrintDocument {
  client: string;
  chart: string;
  /** Yazdırma işinin ve "PDF olarak kaydet"in önerdiği dosyanın adı. */
  title: string;
  pages: PrintPage[];
}

/** Kâğıtta basılan vargalar (UI spec §7: D1+D9). */
export const PRINT_VARGAS = ["D1", "D9"] as const satisfies readonly Varga[];

function coverPage(t: TFunction, opened: OpenedClient): CoverPage {
  const s = opened.chart.meta.settings;
  const ayanamsa = t(`settings.ayanamsa.${ayanamsaKey(s.ayanamsa)}`);
  const deg = opened.chart.ayanamsa_deg;
  const settings: PrintRow[] = [
    [t("settings.field.zodiac"), t(`settings.zodiac.${s.zodiac}`)],
    [t("settings.field.ayanamsa"), deg === null ? ayanamsa : `${ayanamsa} ${formatDegree(deg)}`],
    [t("settings.field.node"), t(`settings.node.${s.node}`)],
    [t("settings.field.house_system"), t(`settings.house.${s.house_system}`)],
    [t("settings.field.dasha_year"), s.dasha_year === null ? "—" : t("settings.dashaYearOption", { days: s.dasha_year })],
  ];
  const warnings = [...opened.chart_record.resolved.warnings, ...opened.chart.meta.warnings];
  return {
    id: "cover",
    heading: opened.client.name,
    birth: birthInfoRows(t, opened),
    settingsHeading: t("settings.title"),
    settings,
    warningsHeading: t("birthInfo.warnings"),
    warnings: warnings.map((w) => ({ name: t(`badge.${w}`), hint: t(`badgeHint.${w}`) })),
    credit: t("print.credit", { app: t("app.title"), engine: opened.chart.meta.engine_version, swe: opened.chart.meta.swe_version }),
  };
}

function chartsPage(t: TFunction, opened: OpenedClient, style: ChartStyle): ChartsPage {
  const unknown = timeUnknown(opened);
  return {
    id: "charts",
    heading: t("print.page.charts"),
    style,
    charts: PRINT_VARGAS.map((varga) => ({
      varga,
      caption: `${varga} · ${VARGA_NAMES[varga]}`,
      data: chartDataFromChart(opened.chart, varga, { showLagna: !unknown }),
      labels: chartLabels(t, varga),
      missing: t("chart.vargaMissing", { varga }),
    })),
    note: unknown ? `${t("chart.timeUnknown")} ${t("chart.timeUnknownVarga")}` : null,
    legend: t("print.legend"),
  };
}

function dashaPage(t: TFunction, opened: OpenedClient): DashaPage {
  const { periods } = opened.chart.dasha;
  const offset = opened.chart_record.resolved.offset_secs;
  const date = (jd: number | null) => formatJdDate(jd, offset);
  return {
    id: "dasha",
    heading: t("print.page.dasha"),
    note: timeUnknown(opened) ? t("dasha.approx") : null,
    columns: { lord: t("print.dasha.lord"), start: t("print.dasha.start"), end: t("print.dasha.end") },
    blocks: childrenOf(periods, []).map((maha) => ({
      lord: t(`grahaName.${maha.lords[0]}`),
      start: date(maha.start_jd_ut),
      end: date(maha.end_jd_ut),
      rows: childrenOf(periods, maha.lords).map((antar) => ({
        lord: t(`grahaName.${antar.lords[1]}`),
        start: date(antar.start_jd_ut),
        end: date(antar.end_jd_ut),
      })),
    })),
  };
}

function grahasPage(t: TFunction, opened: OpenedClient): GrahasPage {
  const unknown = timeUnknown(opened);
  const columns = GRAHA_COLUMNS.filter((c) => !(unknown && c === "house"));
  const lagna = { key: "Lagna", name: t("table.lagnaRow"), cells: columns.map((c) => lagnaCellText(t, opened.chart.lagna, c)) };
  const grahas = opened.chart.grahas.map((g) => ({
    key: g.graha,
    name: t(`grahaName.${g.graha}`),
    cells: columns.map((c) => {
      const text = grahaCellText(t, g, c);
      return unknown && g.graha === "Moon" && APPROX_COLUMNS.has(c) ? `≈ ${text}` : text;
    }),
  }));
  return {
    id: "grahas",
    heading: t("print.page.grahas"),
    rowHeader: t("table.graha"),
    columns: columns.map((c) => ({ id: c, label: t(`table.${c}`), numeric: NUMERIC_COLUMNS.has(c) })),
    rows: unknown ? grahas : [lagna, ...grahas],
  };
}

/** Seçilen sayfalar, `options.pages` sırasında (her zaman `PRINT_PAGES` sırası). */
export function buildPrintDocument(t: TFunction, opened: OpenedClient, options: PrintOptions): PrintDocument {
  const chart = opened.chart_record.label || t(`chart.kind.${opened.chart_record.kind}`);
  const build: Record<PrintPageId, () => PrintPage> = {
    cover: () => coverPage(t, opened),
    charts: () => chartsPage(t, opened, options.style),
    dasha: () => dashaPage(t, opened),
    grahas: () => grahasPage(t, opened),
  };
  return {
    client: opened.client.name,
    chart,
    title: `${opened.client.name} – ${chart}`,
    pages: options.pages.map((id) => build[id]()),
  };
}
```

- [ ] **Step 6: Testlerin geçtiğini gör**

Run: `pnpm --dir app test && pnpm --dir app build`
Expected: `Test Files  56 passed (56)`, `Tests  474 passed (474)`; build `✓ built`.

- [ ] **Step 7: Commit**

```bash
git add app/ui/panels/grahaCells.ts app/ui/panels/GrahaTablePanel.tsx app/ui/panels/BirthInfoPanel.tsx app/ui/print/model.ts app/ui/print/model.test.ts app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): print document model; graha cell and birth row text shared by panels and print"
```
Expected: iki `grep` çıktısız.

---

### Task 3: A4 sayfalar: etkileşimsiz harita çizimi, `PrintDocumentView`, `print.css`

**Files:**
- Modify: `app/ui/chart/ChartSvg.tsx`, `app/ui/chart/ChartSvg.test.tsx`, `app/ui/shell/shell.css`
- Create: `app/ui/print/PrintDocumentView.tsx`, `app/ui/print/print.css`
- Test: `app/ui/print/PrintDocumentView.test.tsx`

**Interfaces:**
- Consumes: `PrintDocument` ve sayfa tipleri, `buildPrintDocument` (Task 2); `.paper` (Task 1).
- Produces: `ChartSvgProps.interactive?: boolean` (varsayılan `true`); `PrintDocumentView({ doc }: { doc: PrintDocument })`. DOM sözleşmesi (Task 4, 6 ve 7 kullanır): her sayfa `section.print-page.paper[data-page="<id>"]`, başlığı `h2#print-page-<id>`, üst satırı `.print-running` (sayfa numarası `.num`); haritalar `figure.print-chart[data-varga]`; graha satırları `tr[data-row="<Graha|Lagna>"]`; notlar `.print-note`.

Bu görevin sonunda eski `ui/print.css` (deneme) hâlâ durur ve yeni `ui/print/print.css` ile birlikte yüklenir; ikisi Task 4'te teke iner. Arada uygulama çalışır, önizleme henüz bağlı değildir.

- [ ] **Step 1: Başarısız testleri yaz**

`ChartSvg.test.tsx`'in sonuna (aşağıdaki diff'in test bölümü) ve `app/ui/print/PrintDocumentView.test.tsx`:

```tsx
import { render, screen, within } from "@testing-library/react";
import { i18n } from "../i18n";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { buildPrintDocument } from "./model";
import { defaultPrintOptions, type PrintOptions } from "./options";
import { PrintDocumentView } from "./PrintDocumentView";
import shellCss from "../shell/shell.css?raw";
import css from "./print.css?raw";

const t = i18n.getFixedT("tr");
const renderDoc = (opened = fixture().opened, options: PrintOptions = defaultPrintOptions()) =>
  render(<PrintDocumentView doc={buildPrintDocument(t, opened, options)} />);
const sheet = (name: string) => screen.getByRole("region", { name });

test("four sheets in spec order, each on paper tokens with the client, the chart and its page number", () => {
  const { container } = renderDoc();
  const pages = [...container.querySelectorAll("section.print-page")];
  expect(pages.map((p) => p.getAttribute("data-page"))).toEqual(["cover", "charts", "dasha", "grahas"]);
  for (const p of pages) expect(p).toHaveClass("paper");
  expect(pages.map((p) => p.querySelector(".print-running")!.textContent)).toEqual([
    "Örnek Danışan · Doğum haritası1 / 4",
    "Örnek Danışan · Doğum haritası2 / 4",
    "Örnek Danışan · Doğum haritası3 / 4",
    "Örnek Danışan · Doğum haritası4 / 4",
  ]);
  expect(screen.getAllByRole("heading", { level: 2 }).map((h) => h.textContent)).toEqual(["Örnek Danışan", "Haritalar: D1 ve D9", "Vimshottari dasa", "Graha tablosu"]);
});

test("page numbers count only the chosen pages", () => {
  const { container } = renderDoc(fixture().opened, { pages: ["charts", "grahas"], style: "north" });
  expect([...container.querySelectorAll(".print-running .num")].map((n) => n.textContent)).toEqual(["1 / 2", "2 / 2"]);
});

test("cover lists birth data and settings; the warnings section appears only when there are warnings", () => {
  renderDoc();
  const cover = sheet("Örnek Danışan");
  expect(within(cover).getByText("New Delhi, Delhi, India")).toBeInTheDocument();
  expect(within(cover).getByText("Lahiri 23°51′")).toBeInTheDocument();
  expect(within(cover).getByText("Jyotish Desk 0.1.0 · Swiss Ephemeris 2.10.03")).toBeInTheDocument();
  expect(within(cover).queryByRole("heading", { name: "Uyarılar" })).toBeNull();
});

test("cover warnings are printed as words", () => {
  renderDoc(unknownTimeOpened());
  const cover = sheet("Örnek Danışan");
  expect(within(cover).getByRole("heading", { name: "Uyarılar" })).toBeInTheDocument();
  expect(within(cover).getByRole("listitem")).toHaveTextContent("Saat bilinmiyor: Doğum saati bilinmiyor; öğlen 12:00 alındı.");
});

test("charts sheet draws D1 and D9 as static images in the chosen style, with the mark legend", () => {
  renderDoc(fixture().opened, { pages: ["charts"], style: "south" });
  const charts = sheet("Haritalar: D1 ve D9");
  const images = within(charts).getAllByRole("img");
  expect(images.map((i) => i.getAttribute("aria-label"))).toEqual(["D1 haritası", "D9 haritası"]);
  for (const svg of images) expect(svg).toHaveAttribute("data-style", "south");
  expect(charts.querySelectorAll('[role="button"], [tabindex]')).toHaveLength(0);
  // Retro düz "R": Satürn'de var, hep geri giden düğümlerde yok.
  const d1 = charts.querySelector('[data-varga="D1"]')!;
  expect(d1.querySelector('[data-graha="Saturn"]')).toHaveTextContent(/^SaR$/);
  expect(d1.querySelector('[data-graha="Rahu"]')).toHaveTextContent(/^Ra$/);
  expect(d1.querySelector('[data-graha="Ketu"]')).toHaveTextContent(/^Ke$/);
  expect(d1.querySelector('[data-graha="Mercury"]')).toHaveTextContent(/^Mec$/);
  expect(d1.querySelectorAll(".chart-cell--lagna")).toHaveLength(1);
  expect(within(charts).getByText("R: retro · c: yanık · altı çizili: vargottama")).toBeInTheDocument();
});

test("with an unknown birth time the charts carry the note and no lagna cell", () => {
  renderDoc(unknownTimeOpened(), { pages: ["charts"], style: "north" });
  const charts = sheet("Haritalar: D1 ve D9");
  expect(within(charts).getByText(/Doğum saati bilinmiyor: lagna ve evler gösterilmiyor/)).toBeInTheDocument();
  expect(charts.querySelectorAll(".chart-cell--lagna")).toHaveLength(0);
});

test("dasha sheet: nine captioned tables of nine antardashas", () => {
  renderDoc(fixture().opened, { pages: ["dasha"], style: "north" });
  const tables = within(sheet("Vimshottari dasa")).getAllByRole("table");
  expect(tables).toHaveLength(9);
  expect(tables[1].querySelector("caption")).toHaveTextContent("Jüpiter 2000-09-18 – 2016-09-18");
  const rows = within(tables[1]).getAllByRole("row");
  expect(rows).toHaveLength(10);
  expect(rows[0]).toHaveTextContent("AntardasaBaşlangıçBitiş");
  expect(rows[1]).toHaveTextContent("Jüpiter2000-09-182002-11-07");
});

test("graha sheet: a row per graha under the lagna, R only where it is shown", () => {
  renderDoc(fixture().opened, { pages: ["grahas"], style: "north" });
  const table = within(sheet("Graha tablosu")).getByRole("table");
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Graha", "Burç", "Derece", "Boylam", "Enlem", "Hız (°/gün)", "Retro", "Nakshatra", "Pada", "Yanık", "Durum", "Ev"]);
  expect(within(table).getAllByRole("rowheader").map((h) => h.textContent)).toEqual(["Lagna", "Güneş", "Ay", "Mars", "Merkür", "Jüpiter", "Venüs", "Satürn", "Rahu", "Ketu"]);
  const retro = (key: string) => table.querySelector(`[data-row="${key}"]`)!.querySelectorAll("td")[5];
  expect(retro("Saturn")).toHaveTextContent(/^R$/);
  expect(retro("Rahu")).toHaveTextContent(/^$/);
  expect(retro("Ketu")).toHaveTextContent(/^$/);
});

test("print CSS: zero page margin with padded sheets, one sheet per page, chrome hidden, tokens only", () => {
  expect(css).toMatch(/@page\s*{[^}]*size:\s*A4;[^}]*margin:\s*0;/);
  expect(css).toMatch(/\.print-page\s*{[^}]*padding:\s*15mm;/);
  // Kâğıtta adlı yazı tipi: sistem fontu PDF metin katmanında noktalı "i"yi yitirir (D6).
  expect(css).toMatch(/\.print-page\s*{[^}]*font-family:\s*"Helvetica Neue", Helvetica, Arial, sans-serif;/);
  expect(css).not.toMatch(/-apple-system|system-ui|var\(--font-ui\)/);
  const print = css.slice(css.indexOf("@media print"));
  expect(print).toMatch(/\.no-print\s*{[^}]*display:\s*none\s*!important;/);
  expect(print).toMatch(/\.print-page\s*{[^}]*break-after:\s*page;[^}]*break-inside:\s*avoid;/);
  expect(print).toMatch(/\.print-page:last-child\s*{[^}]*break-after:\s*auto;/);
  expect(css).not.toMatch(/#[0-9a-fA-F]{3,8}\b/);
  // Kabuk baskıda blok akışına döner; kural shell.css'in sonundadır (ekran kurallarından sonra).
  const shellPrint = shellCss.slice(shellCss.lastIndexOf("@media print"));
  expect(shellPrint).toMatch(/\.shell,\s*\.shell-body,\s*\.shell-main\s*{[^}]*display:\s*block;[^}]*padding:\s*0;/);
  expect(shellCss.indexOf(".shell-main {")).toBeLessThan(shellCss.lastIndexOf("@media print"));
  expect(css).toMatch(/\.paper \.chart-cell--lagna\s*{[^}]*stroke-width:\s*1\.2;/);
});
```

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/chart/ChartSvg.test.tsx ui/print/PrintDocumentView.test.tsx`
Expected: FAIL — `PrintDocumentView` modülü yok; `ChartSvg` testinde `interactive` özelliği tanınmaz (`tsc`) ve `role="img"` bulunamaz.

- [ ] **Step 3: `ChartSvg`'ye `interactive` ekle**

```diff
diff --git a/app/ui/chart/ChartSvg.test.tsx b/app/ui/chart/ChartSvg.test.tsx
index 0baf372..8491f3a 100644
--- a/app/ui/chart/ChartSvg.test.tsx
+++ b/app/ui/chart/ChartSvg.test.tsx
@@ -286,3 +286,12 @@ test("without transits the drawing has no transit labels", () => {
   const { container } = render(<ChartSvg data={{ ascSign: 0, planets: [] }} style="south" labels={labels} />);
   expect(container.querySelector(".chart-transit")).toBeNull();
 });
+
+test("a static drawing is one image: no buttons or tab stops, the same cells and marks", () => {
+  const { container } = render(<ChartSvg data={data} style="north" labels={labels} interactive={false} />);
+  expect(screen.getByRole("img", { name: "Chart" })).toBe(container.querySelector("svg.chart"));
+  expect(container.querySelectorAll('[role="button"], [tabindex], [aria-pressed]')).toHaveLength(0);
+  expect(container.querySelectorAll(".chart-house")).toHaveLength(12);
+  expect(container.querySelector('[data-graha="Mercury"]')).toHaveTextContent(`Me${RETRO_MARK}c`);
+  expect(container.querySelector('[data-graha="Moon"] text')).toHaveAttribute("text-decoration", "underline");
+});
diff --git a/app/ui/chart/ChartSvg.tsx b/app/ui/chart/ChartSvg.tsx
index 0a8538c..096aa1c 100644
--- a/app/ui/chart/ChartSvg.tsx
+++ b/app/ui/chart/ChartSvg.tsx
@@ -17,6 +17,11 @@ export interface ChartSvgProps {
   pressed?: HighlightTarget | null;
   onHover?: (target: HighlightTarget | null) => void;
   onPin?: (target: HighlightTarget) => void;
+  /**
+   * `false`: basılı çizim (Plan 12). Evler ve gezegenler düğme değildir, sekme durağı olmaz;
+   * svg tek bir resimdir (`role="img"`) ve ayrıntıyı yanındaki tablo taşır. Varsayılan `true`.
+   */
+  interactive?: boolean;
 }
 
 function cellHighlighted(cell: ChartCell, h: HighlightTarget | null): boolean {
@@ -47,7 +52,7 @@ function planetName(p: ChartPlanet, labels: ChartLabels): string {
   return parts.join(", ");
 }
 
-export function ChartSvg({ data, style, labels, highlight = null, pressed = highlight, onHover, onPin }: ChartSvgProps) {
+export function ChartSvg({ data, style, labels, highlight = null, pressed = highlight, onHover, onPin, interactive = true }: ChartSvgProps) {
   // Saat bilinmiyorsa lagna yoktur: 1. ev Koç çizilir, lagna işareti ve dolgusu çizilmez,
   // evler yalnızca burç adıyla anılır (ev numarası anlamsızdır).
   const hasLagna = data.ascSign !== null;
@@ -84,13 +89,29 @@ export function ChartSvg({ data, style, labels, highlight = null, pressed = high
       onPin?.(target);
     }
   };
+  /** Düğme davranışı: yalnızca etkileşimli çizimde; basılı çizimde hiçbir öznitelik eklenmez. */
+  const button = (target: HighlightTarget, isPressed: boolean, name: string) =>
+    interactive
+      ? {
+          role: "button",
+          tabIndex: 0,
+          "aria-pressed": isPressed,
+          "aria-label": name,
+          onMouseEnter: () => onHover?.(target),
+          onMouseLeave: () => onHover?.(null),
+          onFocus: () => onHover?.(target),
+          onBlur: () => onHover?.(null),
+          onClick: () => onPin?.(target),
+          onKeyDown: activate(target),
+        }
+      : {};
 
   return (
     <svg
       className="chart"
       data-style={style}
       viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`}
-      role="group"
+      role={interactive ? "group" : "img"}
       aria-label={labels.title}
       xmlns="http://www.w3.org/2000/svg"
     >
@@ -107,16 +128,7 @@ export function ChartSvg({ data, style, labels, highlight = null, pressed = high
             className={`chart-house${on ? " chart-house--highlight" : ""}`}
             data-house={cell.house}
             data-sign={cell.sign}
-            role="button"
-            tabIndex={0}
-            aria-pressed={isPressed}
-            aria-label={hasLagna ? labels.house(cell.house, labels.signs[cell.sign]) : labels.signs[cell.sign]}
-            onMouseEnter={() => onHover?.(house)}
-            onMouseLeave={() => onHover?.(null)}
-            onFocus={() => onHover?.(house)}
-            onBlur={() => onHover?.(null)}
-            onClick={() => onPin?.(house)}
-            onKeyDown={activate(house)}
+            {...button(house, isPressed, hasLagna ? labels.house(cell.house, labels.signs[cell.sign]) : labels.signs[cell.sign])}
           >
             <path className={classes.join(" ")} d={pathOf(cell.points)} />
             {cell.signAnchor && (
@@ -146,16 +158,7 @@ export function ChartSvg({ data, style, labels, highlight = null, pressed = high
               data-house={cell.house}
               data-x={x}
               data-y={y}
-              role="button"
-              tabIndex={0}
-              aria-pressed={isPressed}
-              aria-label={planetName(p, labels)}
-              onMouseEnter={() => onHover?.(target)}
-              onMouseLeave={() => onHover?.(null)}
-              onFocus={() => onHover?.(target)}
-              onBlur={() => onHover?.(null)}
-              onClick={() => onPin?.(target)}
-              onKeyDown={activate(target)}
+              {...button(target, isPressed, planetName(p, labels))}
             >
               <rect
                 className="chart-planet-bg"
```

- [ ] **Step 4: Mevcut çizim değişmedi (C11)**

Run: `pnpm --dir app exec vitest run ui/chart ui/panels && git status --short app/ui/chart/__snapshots__`
Expected: PASS; çıktıda `obsolete` ya da `written` yok; `git status` satırı boş (anlık görüntü dosyaları değişmedi).

- [ ] **Step 5: Sayfaları ve stil dosyasını yaz**

`app/ui/print/PrintDocumentView.tsx`:

```tsx
// Baskı belgesini A4 sayfalar olarak dizer (UI spec §7). Her `.print-page` bir kâğıttır:
// kenar boşluğu sayfanın iç dolgusudur (print.css, Plan 12 D3) ve içerik tek kâğıda sığacak
// biçimde tasarlanmıştır. Ekranda aynı öğeler önizlemedir. Metinler `PrintDocument`'ten gelir.
import type { ReactNode } from "react";
import { ChartSvg } from "../chart/ChartSvg";
import type { ChartsPage, CoverPage, DashaPage, GrahasPage, PrintDocument, PrintPage, PrintRow } from "./model";
import "./print.css";

function Rows({ rows }: { rows: readonly PrintRow[] }) {
  return (
    <dl className="print-rows">
      {rows.map(([term, value]) => (
        <div key={term} className="print-row">
          <dt>{term}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}

function Cover({ page }: { page: CoverPage }) {
  return (
    <>
      <Rows rows={page.birth} />
      <h3 className="print-subheading">{page.settingsHeading}</h3>
      <Rows rows={page.settings} />
      {page.warnings.length > 0 && (
        <>
          <h3 className="print-subheading">{page.warningsHeading}</h3>
          <ul className="print-warnings">
            {page.warnings.map((w) => (
              <li key={w.name}>
                <strong>{w.name}</strong>: {w.hint}
              </li>
            ))}
          </ul>
        </>
      )}
      <p className="print-credit">{page.credit}</p>
    </>
  );
}

function Charts({ page }: { page: ChartsPage }) {
  return (
    <>
      {page.note && <p className="print-note">{page.note}</p>}
      {page.charts.map((c) => (
        <figure key={c.varga} className="print-chart" data-varga={c.varga}>
          <figcaption>{c.caption}</figcaption>
          {c.data ? <ChartSvg data={c.data} style={page.style} labels={c.labels} interactive={false} /> : <p className="print-note">{c.missing}</p>}
        </figure>
      ))}
      <p className="print-legend">{page.legend}</p>
    </>
  );
}

function Dasha({ page }: { page: DashaPage }) {
  return (
    <>
      {page.note && <p className="print-note">{page.note}</p>}
      <div className="print-dasha">
        {page.blocks.map((b) => (
          <table key={b.lord} className="print-table print-dasha-block">
            <caption>
              <strong>{b.lord}</strong> {b.start} – {b.end}
            </caption>
            <thead>
              <tr>
                <th scope="col">{page.columns.lord}</th>
                <th scope="col">{page.columns.start}</th>
                <th scope="col">{page.columns.end}</th>
              </tr>
            </thead>
            <tbody>
              {b.rows.map((r) => (
                <tr key={r.lord}>
                  <th scope="row">{r.lord}</th>
                  <td>{r.start}</td>
                  <td>{r.end}</td>
                </tr>
              ))}
            </tbody>
          </table>
        ))}
      </div>
    </>
  );
}

function Grahas({ page }: { page: GrahasPage }) {
  return (
    <table className="print-table print-grahas">
      <thead>
        <tr>
          <th scope="col">{page.rowHeader}</th>
          {page.columns.map((c) => (
            <th key={c.id} scope="col" className={c.numeric ? "num" : undefined}>
              {c.label}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {page.rows.map((r) => (
          <tr key={r.key} data-row={r.key}>
            <th scope="row">{r.name}</th>
            {r.cells.map((text, i) => (
              <td key={page.columns[i].id} className={page.columns[i].numeric ? "num" : undefined}>
                {text}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function body(page: PrintPage): ReactNode {
  switch (page.id) {
    case "cover":
      return <Cover page={page} />;
    case "charts":
      return <Charts page={page} />;
    case "dasha":
      return <Dasha page={page} />;
    case "grahas":
      return <Grahas page={page} />;
  }
}

export function PrintDocumentView({ doc }: { doc: PrintDocument }) {
  return (
    <div className="print-document">
      {doc.pages.map((page, i) => (
        <section key={page.id} className="print-page paper" data-page={page.id} aria-labelledby={`print-page-${page.id}`}>
          <header className="print-running">
            <span>
              {doc.client} · {doc.chart}
            </span>
            <span className="num">
              {i + 1} / {doc.pages.length}
            </span>
          </header>
          <h2 id={`print-page-${page.id}`} className="print-heading">
            {page.heading}
          </h2>
          {body(page)}
        </section>
      ))}
    </div>
  );
}
```

`app/ui/print/print.css`:

```css
/* A4 baskı düzenleri (UI spec §7). Kenar boşluğu sayfa kutusundan değil, sayfanın iç
   dolgusundan gelir: wry yazdırma işinde NSPrintInfo kenarlarını 0 yapar (Plan 12 D3).
   Renkler yalnızca belirteçlerdir; `.paper` (tokens.css) baskı paletini ekranda da verir. */
@page {
  size: A4;
  margin: 0;
}

.print-document {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 8px 0 24px;
}
/* Ekranda bir A4 kâğıt; baskıda kâğıdın kendisi (aşağıdaki @media print).
   Yazı tipi adıyla verilir (Plan 12 D6): sistem fontu Türkçe `lang` altında PDF'in metin
   katmanında noktalı "i"yi yitirir ("Tarih" → "Tar!h"); arama ve kopyalama bozulur. */
.print-page {
  font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
  box-sizing: border-box;
  width: 210mm;
  min-height: 297mm;
  padding: 15mm;
  border: 1px solid var(--line);
  font-size: 10pt;
  line-height: 1.35;
}
.print-running {
  display: flex;
  justify-content: space-between;
  gap: 8mm;
  padding-bottom: 1.5mm;
  border-bottom: 0.5pt solid var(--ink);
  font-size: 8pt;
}
.print-heading {
  margin: 5mm 0 4mm;
  font-size: 16pt;
  color: var(--ink);
}
.print-subheading {
  margin: 6mm 0 2mm;
  font-size: 11pt;
  color: var(--ink);
}
.print-note,
.print-legend,
.print-credit {
  margin: 0 0 3mm;
  font-size: 9pt;
}
.print-legend {
  margin-top: 2mm;
  text-align: center;
}
.print-credit {
  margin-top: 10mm;
}

.print-rows {
  margin: 0;
}
.print-row {
  display: flex;
  gap: 4mm;
  padding: 0.8mm 0;
  border-bottom: 0.5pt solid var(--line);
}
.print-row dt {
  flex: 0 0 42mm;
  font-weight: var(--fw-medium);
}
.print-row dd {
  margin: 0;
  overflow-wrap: anywhere;
}
.print-warnings {
  margin: 0;
  padding-left: 5mm;
}

/* İki harita alt alta; 82 mm ile sayfa, saat notu basılsa da Letter kâğıda sığar (bkz. D3: baskı ölçeği). */
.print-chart {
  width: 82mm;
  margin: 0 auto 3mm;
}
.print-chart figcaption {
  margin-bottom: 1mm;
  font-weight: var(--fw-medium);
  text-align: center;
}
.print-chart .chart {
  height: auto;
  aspect-ratio: 1;
}
/* Kâğıtta --select beyazdır: lagna evi dolguyla değil kalın çizgiyle belirtilir (C9). */
.paper .chart-cell--lagna {
  stroke-width: 1.2;
}

.print-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 9pt;
}
.print-table th,
.print-table td {
  padding: 0.6mm 1.5mm;
  border-bottom: 0.5pt solid var(--line);
  text-align: left;
  white-space: nowrap;
}
.print-table .num {
  text-align: right;
}
.print-table caption {
  padding-bottom: 1mm;
  text-align: left;
}
.print-dasha {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 5mm 6mm;
}
.print-dasha-block {
  font-size: 8pt;
}
.print-dasha-block th,
.print-dasha-block td {
  padding: 0.4mm 1mm;
}

@media print {
  .no-print {
    display: none !important;
  }
  .print-document {
    display: block;
    padding: 0;
  }
  .print-page {
    width: auto;
    min-height: 0;
    border: 0;
    break-after: page;
    break-inside: avoid;
  }
  .print-page:last-child {
    break-after: auto;
  }
}
```

`app/ui/shell/shell.css`'in **sonuna** ekle (kural bu dosyada olmalıdır: aynı seçicilerin ekran kuralları buradadır ve sonra gelen kazanır; `print.css`'e konursa `.shell-main`'in 8 px dolgusu kâğıda taşınır):

```css

/* Baskı (Plan 12): kabuk esnek kutu olmaktan çıkar, çünkü sayfa kesmeleri blok akışında
   güvenilirdir; dolgu ve zemin kâğıda taşınmaz. Kural bu dosyadadır: aynı seçicilerin
   ekran kuralları da buradadır ve sonra gelen kazanır. */
@media print {
  .shell,
  .shell-body,
  .shell-main {
    display: block;
    min-height: 0;
    padding: 0;
    background: var(--panel);
  }
}
```

- [ ] **Step 6: Testlerin geçtiğini gör**

Run: `pnpm --dir app test && pnpm --dir app build`
Expected: `Test Files  57 passed (57)`, `Tests  484 passed (484)`; build `✓ built`.

- [ ] **Step 7: Commit**

```bash
git add app/ui/chart/ChartSvg.tsx app/ui/chart/ChartSvg.test.tsx app/ui/print/PrintDocumentView.tsx app/ui/print/PrintDocumentView.test.tsx app/ui/print/print.css app/ui/shell/shell.css
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): A4 print sheets (cover, D1+D9, dasha, graha table) and a static chart drawing"
```
Expected: iki `grep` çıktısız.

---

### Task 4: Baskı önizlemesi; deneme görünümünün kaldırılması

**Files:**
- Create: `app/ui/print/PrintView.tsx`
- Modify: `app/ui/print/print.css`, `app/ui/shell/Shell.tsx`, `app/ui/shell/TopBar.tsx`, `app/ui/shell/shell.css`, `app/ui/theme/no-hex.test.ts`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`, `app/ui/App.test.tsx`
- Delete: `app/ui/PrintSpike.tsx`, `app/ui/PrintSpike.test.tsx`, `app/ui/print.css`, `app/ui/shell/PrintView.tsx`, `app/ui/shell/PrintView.test.tsx`
- Test: `app/ui/print/PrintView.test.tsx`, `app/ui/App.test.tsx`

**Interfaces:**
- Consumes: `buildPrintDocument` (Task 2), `PrintDocumentView` (Task 3), `loadPrintOptions`, `savePrintOptions`, `togglePage`, `PRINT_PAGES` (Task 1), `api.printWindow()` (mevcut; `Result<null, CommandError>`), `PanelError` (`panels/states.tsx`), `MAIN_ID` (`shell/focus.ts`).
- Produces: `PrintView({ error, onPrint }: { error: CommandError | null; onPrint(): void })`; kökü `div.print-view[data-print-preview]` (Task 5 bu özniteliğe bakar); araç çubuğu `div.print-toolbar.no-print[role="group"]`. `TopBarProps.onTogglePrint?: () => void` (açık harita yoksa verilmez). `Shell` içinde `printView: boolean` (türetilmiş), `togglePrint()`, `printNow(): Promise<void>` (Task 5 kısayola bağlar). i18n: `print.{open,back,toolbar,pages,style,print,pdfHint,lastPage}`; `shell.{printSpike,backToDesk,print}` silinir.

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/print/PrintView.test.tsx`:

```tsx
import { fireEvent, screen, within } from "@testing-library/react";
import { vi } from "vitest";
import { renderWithStores } from "../test/render";
import { PRINT_STORAGE_KEY } from "./options";
import { PrintView } from "./PrintView";

const renderView = (onPrint = vi.fn(), error: Parameters<typeof PrintView>[0]["error"] = null) => ({
  onPrint,
  ...renderWithStores(<PrintView error={error} onPrint={onPrint} />),
});
const sheets = (container: HTMLElement) => [...container.querySelectorAll("section.print-page")].map((s) => s.getAttribute("data-page"));
const toolbar = () => screen.getByRole("group", { name: "Baskı seçenekleri" });

test("the preview shows the four sheets; the toolbar is screen-only and its button asks for the print dialog", () => {
  const { container, onPrint } = renderView();
  expect(sheets(container)).toEqual(["cover", "charts", "dasha", "grahas"]);
  expect(toolbar()).toHaveClass("no-print");
  expect(toolbar().querySelector(".print-page")).toBeNull();
  const print = within(toolbar()).getByRole("button", { name: "Yazdır…" });
  expect(print).toHaveFocus();
  expect(onPrint).not.toHaveBeenCalled();
  fireEvent.click(print);
  expect(onPrint).toHaveBeenCalledOnce();
  expect(within(toolbar()).getByText(/PDF olarak kaydet/)).toBeInTheDocument();
});

test("unchecking a page removes its sheet and renumbers; the choice is stored", () => {
  const { container } = renderView();
  fireEvent.click(within(toolbar()).getByRole("checkbox", { name: "Vimshottari dasa" }));
  expect(sheets(container)).toEqual(["cover", "charts", "grahas"]);
  expect([...container.querySelectorAll(".print-running .num")].map((n) => n.textContent)).toEqual(["1 / 3", "2 / 3", "3 / 3"]);
  expect(JSON.parse(localStorage.getItem(PRINT_STORAGE_KEY)!)).toEqual({ pages: ["cover", "charts", "grahas"], style: "north" });
});

test("the last page cannot be unchecked and says why", () => {
  localStorage.setItem(PRINT_STORAGE_KEY, JSON.stringify({ pages: ["grahas"], style: "north" }));
  const { container } = renderView();
  const only = within(toolbar()).getByRole("checkbox", { name: "Graha tablosu" });
  expect(only).toHaveAttribute("aria-disabled", "true");
  expect(only).toHaveAccessibleDescription("En az bir sayfa seçili olmalıdır.");
  fireEvent.click(only);
  expect(only).toBeChecked();
  expect(sheets(container)).toEqual(["grahas"]);
  fireEvent.click(within(toolbar()).getByRole("checkbox", { name: "Kapak" }));
  expect(sheets(container)).toEqual(["cover", "grahas"]);
  expect(only).not.toHaveAttribute("aria-disabled");
  expect(screen.queryByText("En az bir sayfa seçili olmalıdır.")).toBeNull();
});

test("the chart style applies to both printed charts and is restored from storage", () => {
  localStorage.setItem(PRINT_STORAGE_KEY, JSON.stringify({ pages: ["charts"], style: "circle" }));
  const { container } = renderView();
  const style = within(toolbar()).getByRole("combobox", { name: "Harita biçimi" });
  expect(style).toHaveValue("circle");
  fireEvent.change(style, { target: { value: "south" } });
  expect([...container.querySelectorAll("svg.chart")].map((s) => s.getAttribute("data-style"))).toEqual(["south", "south"]);
  expect(JSON.parse(localStorage.getItem(PRINT_STORAGE_KEY)!).style).toBe("south");
});

test("the document title names the client and chart while the preview is open", () => {
  document.title = "Jyotish Desk";
  const { unmount } = renderView();
  expect(document.title).toBe("Örnek Danışan – Doğum haritası");
  unmount();
  expect(document.title).toBe("Jyotish Desk");
});

test("a print error is shown in the toolbar with a retry that prints again", () => {
  const { onPrint } = renderView(vi.fn(), { code: "print_failed", params: {} });
  expect(within(toolbar()).getByRole("alert")).toHaveTextContent("Yazdırma penceresi açılamadı.");
  fireEvent.click(within(toolbar()).getByRole("button", { name: "Yeniden dene" }));
  expect(onPrint).toHaveBeenCalledOnce();
});

test("without an open chart the preview renders nothing", () => {
  const { container } = renderWithStores(<PrintView error={null} onPrint={() => {}} />, { opened: null });
  expect(container.querySelector(".print-view")).toBeNull();
});
```

`App.test.tsx`'te iki deneme testi üç önizleme testiyle değişir ve "Cmd+Shift+N…" testindeki düğme adı güncellenir (aşağıdaki Step 3 diff'inin `App.test.tsx` bölümü).

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/print/PrintView.test.tsx`
Expected: FAIL — `./PrintView` modülü yok.

- [ ] **Step 3: Önizlemeyi yaz, kabuğa bağla, denemeyi sil**

`app/ui/print/PrintView.tsx`:

```tsx
// Baskı önizlemesi (UI spec §7, Plan 12): seçilen A4 sayfalar ekranda kâğıt olarak görünür;
// araç çubuğu (`no-print`) kâğıda çıkmaz. Yazdırma penceresini Rust açar (`print_window`);
// PDF, o pencerenin "PDF olarak kaydet" seçeneğidir (v1 spec §4). Önizleme açıkken belge
// başlığı danışan ve harita adıdır: yazdırma işi ve önerilen PDF dosya adı bunu kullanır.
import { useEffect, useId, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError } from "../bindings";
import { CHART_STYLES, isChartStyle } from "../chart/types";
import { PanelError } from "../panels/states";
import { useSession } from "../session/store";
import { MAIN_ID } from "../shell/focus";
import { buildPrintDocument } from "./model";
import { loadPrintOptions, PRINT_PAGES, savePrintOptions, togglePage, type PrintOptions } from "./options";
import { PrintDocumentView } from "./PrintDocumentView";

export interface PrintViewProps {
  /** Son yazdırma denemesinin hatası (Shell tutar; ⌘P de aynı yolu kullanır). */
  error: CommandError | null;
  onPrint(): void;
}

export function PrintView({ error, onPrint }: PrintViewProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const [options, setOptions] = useState<PrintOptions>(() => loadPrintOptions());
  const id = useId();
  const printButton = useRef<HTMLButtonElement>(null);
  const doc = useMemo(() => (opened ? buildPrintDocument(t, opened, options) : null), [t, opened, options]);
  const title = doc?.title ?? null;

  // Açılınca odak "Yazdır…" düğmesindedir (⌘P, Enter); kapanınca `body`'ye düşmez.
  useEffect(() => {
    printButton.current?.focus();
    return () => {
      if (document.activeElement === document.body) document.getElementById(MAIN_ID)?.focus();
    };
  }, []);

  useEffect(() => {
    if (title === null) return;
    const previous = document.title;
    document.title = title;
    return () => {
      document.title = previous;
    };
  }, [title]);

  if (!doc) return null;
  const update = (next: PrintOptions) => {
    savePrintOptions(next);
    setOptions(next);
  };
  const lastPage = options.pages.length === 1;
  return (
    <div className="print-view" data-print-preview="">
      <div className="print-toolbar no-print" role="group" aria-label={t("print.toolbar")}>
        <fieldset className="print-pages">
          <legend>{t("print.pages")}</legend>
          {PRINT_PAGES.map((page) => {
            const only = lastPage && options.pages[0] === page;
            return (
              <label key={page}>
                <input
                  type="checkbox"
                  checked={options.pages.includes(page)}
                  aria-disabled={only || undefined}
                  aria-describedby={only ? `${id}-last` : undefined}
                  onChange={() => update({ ...options, pages: togglePage(options.pages, page) })}
                />
                {t(`print.page.${page}`)}
              </label>
            );
          })}
          {lastPage && (
            <span id={`${id}-last`} className="print-hint">
              {t("print.lastPage")}
            </span>
          )}
        </fieldset>
        <label htmlFor={`${id}-style`}>{t("print.style")}</label>
        <select id={`${id}-style`} value={options.style} onChange={(e) => isChartStyle(e.target.value) && update({ ...options, style: e.target.value })}>
          {CHART_STYLES.map((s) => (
            <option key={s} value={s}>
              {t(`chartStyle.${s}`)}
            </option>
          ))}
        </select>
        <button ref={printButton} type="button" aria-keyshortcuts="Meta+P" onClick={onPrint}>
          {t("print.print")}
        </button>
        <p className="print-hint">{t("print.pdfHint")}</p>
        {error && <PanelError error={error} onRetry={onPrint} />}
      </div>
      <PrintDocumentView doc={doc} />
    </div>
  );
}
```

`app/ui/print/print.css`'te `@media print {` satırının hemen **üstüne** ekle:

```css
/* Önizleme araç çubuğu: yalnızca ekranda (`no-print`). */
.print-toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px 16px;
  padding: 8px 12px;
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 4px;
}
.print-pages {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px 12px;
  margin: 0;
  padding: 0;
  border: 0;
}
.print-pages legend {
  float: left;
  margin-right: 12px;
  padding: 0;
  font-weight: var(--fw-medium);
}
.print-pages label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.print-pages [aria-disabled="true"] {
  opacity: 0.6;
}
.print-hint {
  margin: 0;
  color: var(--text-2);
  font-size: var(--fs-table);
}
```

Deneme dosyalarını sil:

```bash
git rm -q app/ui/PrintSpike.tsx app/ui/PrintSpike.test.tsx app/ui/print.css app/ui/shell/PrintView.tsx app/ui/shell/PrintView.test.tsx
```

Kabuk, üst çubuk, dil dosyaları, hex testi ve kabuk testi:

```diff
diff --git a/app/ui/App.test.tsx b/app/ui/App.test.tsx
index e04fe4a..fda68d8 100644
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -184,7 +184,7 @@ test("Cmd+Shift+N works from a text field and is ignored on key repeat, during c
   fireEvent.keyDown(window, { key: "N", metaKey: true, shiftKey: true, isComposing: true });
   expect(notes()).toBeInTheDocument();
 
-  fireEvent.click(screen.getByRole("button", { name: "Baskı denemesi" }));
+  fireEvent.click(screen.getByRole("button", { name: "Baskı önizleme" }));
   const ev = new KeyboardEvent("keydown", { key: "N", metaKey: true, shiftKey: true, cancelable: true });
   window.dispatchEvent(ev);
   expect(ev.defaultPrevented).toBe(false);
@@ -292,23 +292,43 @@ test("language menu switches every string", async () => {
   expect(screen.getByRole("combobox", { name: "Theme" })).toBeInTheDocument();
 });
 
-test("the print test stays reachable and calls printWindow", async () => {
+test("the print preview replaces the desk, prints through printWindow and returns to the desk", async () => {
+  vi.mocked(api.printWindow).mockClear();
+  document.title = "Jyotish Desk";
   renderApp();
-  fireEvent.click(screen.getByRole("button", { name: "Baskı denemesi" }));
-  expect(await screen.findByText(/Doğum haritası — ğüşıöç İ/)).toBeInTheDocument();
-  expect(screen.getByTestId("sign-8")).toHaveTextContent("Sun");
+  fireEvent.click(screen.getByRole("button", { name: "Baskı önizleme" }));
+  expect(screen.getByRole("group", { name: "Baskı seçenekleri" })).toBeInTheDocument();
+  expect(screen.getAllByRole("img").map((i) => i.getAttribute("aria-label"))).toEqual(["D1 haritası", "D9 haritası"]);
+  // Pano, kenar çubuğu ve düzen seçici önizlemede yoktur.
+  expect(screen.queryByRole("group", { name: "D1 haritası" })).toBeNull();
+  expect(screen.queryByRole("complementary", { name: "Danışanlar" })).toBeNull();
+  expect(screen.queryByRole("combobox", { name: "Düzen" })).toBeNull();
+  expect(document.title).toBe("Örnek Danışan – Doğum haritası");
+  expect(screen.getByRole("button", { name: "Yazdır…" })).toHaveFocus();
+  expect(api.printWindow).not.toHaveBeenCalled();
   fireEvent.click(screen.getByRole("button", { name: "Yazdır…" }));
   expect(api.printWindow).toHaveBeenCalledOnce();
   fireEvent.click(screen.getByRole("button", { name: "Panoya dön" }));
   expect(await screen.findByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
+  expect(screen.queryByRole("group", { name: "Baskı seçenekleri" })).toBeNull();
+  expect(document.title).toBe("Jyotish Desk");
 });
 
-test("a print failure is shown with a retry", async () => {
+test("a print failure is shown with a retry; a retry that succeeds clears it", async () => {
+  vi.mocked(api.printWindow).mockClear();
   vi.mocked(api.printWindow).mockResolvedValueOnce({ status: "error", error: { code: "print_failed", params: {} } });
   renderApp();
-  fireEvent.click(screen.getByRole("button", { name: "Baskı denemesi" }));
+  fireEvent.click(screen.getByRole("button", { name: "Baskı önizleme" }));
   fireEvent.click(screen.getByRole("button", { name: "Yazdır…" }));
   expect(await screen.findByRole("alert")).toHaveTextContent("Yazdırma penceresi açılamadı.");
+  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
+  await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
+  expect(api.printWindow).toHaveBeenCalledTimes(2);
+});
+
+test("without an open chart there is nothing to print: the preview button is absent", () => {
+  renderApp({ opened: null });
+  expect(screen.queryByRole("button", { name: "Baskı önizleme" })).toBeNull();
 });
 
 test("? opens the shortcut help as a modal that traps focus, blocks shortcuts and closes with Esc", () => {
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 4929ddb..5785e21 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -166,9 +166,6 @@ export const en: Messages = {
   theme: { label: "Theme", light: "Light", dark: "Dark", system: "System" },
   language: { label: "Language", tr: "Türkçe", en: "English", fr: "Français" },
   shell: {
-    printSpike: "Print test",
-    backToDesk: "Back to desk",
-    print: "Print…",
     warnings: "Warnings",
   },
   badge: {
@@ -458,6 +455,14 @@ export const en: Messages = {
     action: "New chart",
   },
   print: {
+    open: "Print preview",
+    back: "Back to desk",
+    toolbar: "Print options",
+    pages: "Pages",
+    style: "Chart style",
+    print: "Print…",
+    pdfHint: "For a PDF, choose “Save as PDF” from the “PDF” menu of the print dialog.",
+    lastPage: "At least one page must stay selected.",
     page: { cover: "Cover", charts: "Charts: D1 and D9", dasha: "Vimshottari dasha", grahas: "Graha table" },
     legend: "R: retrograde · c: combust · underlined: vargottama",
     credit: "{{app}} {{engine}} · Swiss Ephemeris {{swe}}",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index ced8daf..645e777 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -166,9 +166,6 @@ export const fr: Messages = {
   theme: { label: "Thème", light: "Clair", dark: "Sombre", system: "Système" },
   language: { label: "Langue", tr: "Türkçe", en: "English", fr: "Français" },
   shell: {
-    printSpike: "Essai d’impression",
-    backToDesk: "Retour au tableau",
-    print: "Imprimer…",
     warnings: "Avertissements",
   },
   badge: {
@@ -458,6 +455,14 @@ export const fr: Messages = {
     action: "Nouvelle carte",
   },
   print: {
+    open: "Aperçu avant impression",
+    back: "Retour au tableau",
+    toolbar: "Options d’impression",
+    pages: "Pages",
+    style: "Style de carte",
+    print: "Imprimer…",
+    pdfHint: "Pour un PDF, choisissez « Enregistrer au format PDF » dans le menu « PDF » de la fenêtre d’impression.",
+    lastPage: "Au moins une page doit rester sélectionnée.",
     page: { cover: "Couverture", charts: "Cartes : D1 et D9", dasha: "Dasha Vimshottari", grahas: "Tableau des grahas" },
     legend: "R : rétrograde · c : combuste · souligné : vargottama",
     credit: "{{app}} {{engine}} · Swiss Ephemeris {{swe}}",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index fb04392..0adf845 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -165,9 +165,6 @@ export const tr = {
   theme: { label: "Tema", light: "Açık", dark: "Koyu", system: "Sistem" },
   language: { label: "Dil", tr: "Türkçe", en: "English", fr: "Français" },
   shell: {
-    printSpike: "Baskı denemesi",
-    backToDesk: "Panoya dön",
-    print: "Yazdır…",
     warnings: "Uyarılar",
   },
   badge: {
@@ -457,6 +454,14 @@ export const tr = {
     action: "Yeni harita",
   },
   print: {
+    open: "Baskı önizleme",
+    back: "Panoya dön",
+    toolbar: "Baskı seçenekleri",
+    pages: "Sayfalar",
+    style: "Harita biçimi",
+    print: "Yazdır…",
+    pdfHint: "PDF için yazdırma penceresinde “PDF” menüsünden “PDF olarak kaydet”i seçin.",
+    lastPage: "En az bir sayfa seçili olmalıdır.",
     page: { cover: "Kapak", charts: "Haritalar: D1 ve D9", dasha: "Vimshottari dasa", grahas: "Graha tablosu" },
     legend: "R: retro · c: yanık · altı çizili: vargottama",
     credit: "{{app}} {{engine}} · Swiss Ephemeris {{swe}}",
diff --git a/app/ui/shell/Shell.tsx b/app/ui/shell/Shell.tsx
index aab47b5..e9da5a6 100644
--- a/app/ui/shell/Shell.tsx
+++ b/app/ui/shell/Shell.tsx
@@ -1,5 +1,7 @@
 import { SettingsDialog } from "../settings/SettingsDialog";
 import { useEffect, useLayoutEffect, useRef, useState } from "react";
+import { api } from "../api";
+import type { CommandError } from "../bindings";
 import { BirthForm } from "../birth/BirthForm";
 import { Sidebar } from "../clients/Sidebar";
 import type { PaletteItem } from "../palette/items";
@@ -13,7 +15,7 @@ import { EmptyState } from "./EmptyState";
 import { leaveNotes, leaveSidebar, MAIN_ID } from "./focus";
 import { HelpDialog } from "./HelpDialog";
 import { NoticeBar, useNotices } from "./notices";
-import { PrintView } from "./PrintView";
+import { PrintView } from "../print/PrintView";
 import "./shell.css";
 import { TopBar } from "./TopBar";
 import { useElementWidth } from "./useElementWidth";
@@ -22,7 +24,8 @@ export function Shell() {
   const layout = useLayoutStore();
   const { toggleNotes } = layout;
   const [themePref, setThemePref] = useState<ThemePref>(() => loadThemePref());
-  const [printView, setPrintView] = useState(false);
+  const [printRequested, setPrintRequested] = useState(false);
+  const [printError, setPrintError] = useState<CommandError | null>(null);
   const [help, setHelp] = useState(false);
   const [settings, setSettings] = useState(false);
   const [form, setForm] = useState<"new" | "add" | "edit" | null>(null);
@@ -34,6 +37,18 @@ export function Shell() {
   };
   const session = useSession();
   const { opened } = session;
+  // Baskı önizlemesi yalnızca açık haritayla anlamlıdır; harita kapanırsa pano geri gelir.
+  const printView = printRequested && opened !== null;
+  const togglePrint = () => {
+    setPrintError(null);
+    setPrintRequested(!printView);
+  };
+  // Yazdırma penceresini Rust açar (C2′: arayüz yol vermez; PDF o pencerenin seçeneğidir).
+  const printNow = async () => {
+    setPrintError(null);
+    const r = await api.printWindow();
+    if (r.status === "error") setPrintError(r.error);
+  };
   const { push } = useNotices();
   const measureRef = useRef<HTMLDivElement>(null);
   const width = useElementWidth(measureRef);
@@ -123,7 +138,7 @@ export function Shell() {
         themePref={themePref}
         onThemeChange={changeTheme}
         printView={printView}
-        onTogglePrint={() => setPrintView((v) => !v)}
+        onTogglePrint={opened ? togglePrint : undefined}
         onHelp={() => setHelp(true)}
         onNewChart={() => setForm("new")}
         onEditChart={opened ? () => setForm("edit") : undefined}
@@ -136,7 +151,7 @@ export function Shell() {
       <div className="shell-body">
         {sidebar && !printView && <Sidebar onAddChart={() => setForm("add")} />}
         <main id={MAIN_ID} className="shell-main" tabIndex={-1}>
-          <div ref={measureRef}>{printView ? <PrintView /> : opened ? <Dashboard width={width} /> : <EmptyState onNew={() => setForm("new")} />}</div>
+          <div ref={measureRef}>{printView ? <PrintView error={printError} onPrint={() => void printNow()} /> : opened ? <Dashboard width={width} /> : <EmptyState onNew={() => setForm("new")} />}</div>
         </main>
       </div>
       {help && <HelpDialog onClose={() => setHelp(false)} />}
diff --git a/app/ui/shell/TopBar.tsx b/app/ui/shell/TopBar.tsx
index 8430913..0994f95 100644
--- a/app/ui/shell/TopBar.tsx
+++ b/app/ui/shell/TopBar.tsx
@@ -9,7 +9,8 @@ export interface TopBarProps {
   themePref: ThemePref;
   onThemeChange(pref: ThemePref): void;
   printView: boolean;
-  onTogglePrint(): void;
+  /** Açık harita yoksa verilmez (düğme gösterilmez). */
+  onTogglePrint?: () => void;
   onHelp(): void;
   onNewChart(): void;
   /** Açık harita yoksa verilmez (düğme gösterilmez). */
@@ -73,9 +74,11 @@ export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onH
         <button type="button" aria-keyshortcuts="?" onClick={onHelp}>
           {t("shortcuts.open")}
         </button>
-        <button type="button" onClick={onTogglePrint}>
-          {printView ? t("shell.backToDesk") : t("shell.printSpike")}
-        </button>
+        {onTogglePrint && (
+          <button type="button" onClick={onTogglePrint}>
+            {printView ? t("print.back") : t("print.open")}
+          </button>
+        )}
       </div>
     </header>
   );
diff --git a/app/ui/shell/shell.css b/app/ui/shell/shell.css
index a8b99c3..0446566 100644
--- a/app/ui/shell/shell.css
+++ b/app/ui/shell/shell.css
@@ -93,9 +93,6 @@
   padding: 8px;
   background: var(--bg);
 }
-.print-view-actions {
-  margin-bottom: 8px;
-}
 .topbar-client {
   margin: 0;
   font-size: var(--fs-client);
diff --git a/app/ui/theme/no-hex.test.ts b/app/ui/theme/no-hex.test.ts
index 038fc9f..9e546ee 100644
--- a/app/ui/theme/no-hex.test.ts
+++ b/app/ui/theme/no-hex.test.ts
@@ -6,9 +6,9 @@ const sources = import.meta.glob("/ui/**/*.{ts,tsx,css}", {
   eager: true,
 }) as Record<string, string>;
 
-// tokens.css belirteçlerin kaynağı; bindings.ts üretilmiş; PrintSpike ve print.css
-// M7'de baskı düzenleriyle değişecek spike dosyaları; testler beklenen değer olarak hex içerir.
-const EXEMPT = [/^\/ui\/theme\/tokens\.css$/, /^\/ui\/bindings\.ts$/, /^\/ui\/PrintSpike\.tsx$/, /^\/ui\/print\.css$/, /\.test\.tsx?$/];
+// tokens.css belirteçlerin kaynağı; bindings.ts üretilmiş; testler beklenen değer olarak hex
+// içerir. Baskı düzenleri (ui/print) istisna değildir: kâğıt paleti de tokens.css'tedir.
+const EXEMPT = [/^\/ui\/theme\/tokens\.css$/, /^\/ui\/bindings\.ts$/, /\.test\.tsx?$/];
 
 test("no hex colors outside theme tokens", () => {
   const files = Object.keys(sources).filter((f) => !EXEMPT.some((re) => re.test(f)));
```

- [ ] **Step 4: Testlerin geçtiğini gör**

Run: `pnpm --dir app test && pnpm --dir app build`
Expected: `Test Files  56 passed (56)`, `Tests  489 passed (489)` (iki deneme test dosyası gitti, bir önizleme test dosyası geldi); build `✓ built`.

Run: `grep -rn "PrintSpike\|printSpike\|print-view-actions" app/ui app/e2e`
Expected: çıktısız.

- [ ] **Step 5: Commit**

```bash
git add -A app/ui
git status --short
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): print preview with page and style options replaces the print spike"
```
Expected: `git status --short` yalnızca `app/ui` altını gösterir (beş `D`, `print/PrintView.tsx` ve testi `A`, diğerleri `M`); iki `grep` çıktısız.

---

### Task 5: `⌘P`, palet komutu, önizlemede düzen kısayolları, kenar çubuğu durumunun saklanması

**Files:**
- Create: `app/ui/shell/sidebarPref.ts`
- Modify: `app/ui/shortcuts.ts`, `app/ui/shortcuts.test.ts`, `app/ui/palette/items.ts`, `app/ui/palette/Palette.tsx`, `app/ui/shell/Shell.tsx`, `app/ui/shell/TopBar.tsx`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`, `app/ui/App.test.tsx`
- Test: `app/ui/shell/sidebarPref.test.ts`, `app/ui/shortcuts.test.ts`, `app/ui/App.test.tsx`

**Interfaces:**
- Consumes: `Shell` içindeki `printView`, `togglePrint`, `printNow` (Task 4); `[data-print-preview]` (Task 4).
- Produces: `Command` birleşimine `{ id: "print" }`; `SHORTCUT_LIST`'e `{ id: "print", keys: "⌘P" }` (`settings`'ten sonra); `printPreviewOpen(): boolean`; `PaletteCommand`'a `"print"`; `SIDEBAR_STORAGE_KEY = "jyotish-desk.sidebar.v1"`, `loadSidebarOpen(storage?): boolean`, `saveSidebarOpen(open, storage?): void`; i18n `shortcuts.print`, `palette.command.print`.

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/shell/sidebarPref.test.ts`:

```ts
import { loadSidebarOpen, saveSidebarOpen, SIDEBAR_STORAGE_KEY } from "./sidebarPref";

test("the sidebar is open unless it was stored closed", () => {
  expect(loadSidebarOpen()).toBe(true);
  saveSidebarOpen(false);
  expect(localStorage.getItem(SIDEBAR_STORAGE_KEY)).toBe("closed");
  expect(loadSidebarOpen()).toBe(false);
  saveSidebarOpen(true);
  expect(loadSidebarOpen()).toBe(true);
  localStorage.setItem(SIDEBAR_STORAGE_KEY, "sideways");
  expect(loadSidebarOpen()).toBe(true);
});

test("a storage that throws leaves the sidebar open and does not break saving", () => {
  const broken = {
    getItem: () => {
      throw new Error("denied");
    },
    setItem: () => {
      throw new Error("full");
    },
  } as unknown as Storage;
  expect(loadSidebarOpen(broken)).toBe(true);
  expect(() => saveSidebarOpen(false, broken)).not.toThrow();
});
```

`shortcuts.test.ts` ve `App.test.tsx` eklemeleri Step 3 diff'indedir (yalnızca bu iki dosyanın bölümlerini şimdi uygula).

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/shell/sidebarPref.test.ts ui/shortcuts.test.ts ui/App.test.tsx`
Expected: FAIL — `./sidebarPref` yok; `⌘P` `null` döner; `App.test.tsx`'te beş yeni test düşer.

- [ ] **Step 3: Uygula**

`app/ui/shell/sidebarPref.ts`:

```ts
// Kenar çubuğunun açık/kapalı durumu (Plan 10 kapsam dışı 16, Plan 12 D11). Tema ve dil gibi
// bir arayüz tercihidir; danışan verisi içermez. Okunamazsa kenar çubuğu açıktır.
export const SIDEBAR_STORAGE_KEY = "jyotish-desk.sidebar.v1";

export function loadSidebarOpen(storage: Storage = window.localStorage): boolean {
  try {
    return storage.getItem(SIDEBAR_STORAGE_KEY) !== "closed";
  } catch {
    return true;
  }
}

/** Yazılamazsa durum yalnızca bu oturumda geçerli olur; hata loglanmaz (C1). */
export function saveSidebarOpen(open: boolean, storage: Storage = window.localStorage): void {
  try {
    storage.setItem(SIDEBAR_STORAGE_KEY, open ? "open" : "closed");
  } catch {
    // Tercih kaybı zararsızdır.
  }
}
```

Kısayol, palet, kabuk, üst çubuk, dil dosyaları ve testler:

```diff
diff --git a/app/ui/App.test.tsx b/app/ui/App.test.tsx
index fda68d8..4ea497d 100644
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -35,6 +35,8 @@ function renderApp(boot: Partial<Boot> = {}, { store = false } = {}) {
 }
 
 const toggleNotes = () => fireEvent.keyDown(window, { key: "N", metaKey: true, shiftKey: true });
+/** Baskı önizlemesinin araç çubuğu; önizleme açık değilse hata verir. */
+const printToolbar = () => screen.getByRole("group", { name: "Baskı seçenekleri" });
 
 test("opens on the interview layout with the restored chart", () => {
   renderApp();
@@ -358,3 +360,61 @@ test("preference warnings from boot are shown in the notice bar", () => {
   renderApp({ opened: null, prefs: { ...fixture().prefs, warnings: [{ TooNew: { found: 2 } }] } });
   expect(screen.getByText(/Tercihler daha yeni bir uygulama sürümüyle kaydedilmiş \(sürüm 2\)/)).toBeInTheDocument();
 });
+
+test("Cmd+P opens the print preview, and inside it asks for the print dialog", () => {
+  vi.mocked(api.printWindow).mockClear();
+  renderApp();
+  // `fireEvent` varsayılan davranış engellendiyse `false` döner: web görünümü kendisi yazdırmaz.
+  expect(fireEvent.keyDown(window, { key: "p", metaKey: true })).toBe(false);
+  expect(printToolbar()).toBeInTheDocument();
+  expect(api.printWindow).not.toHaveBeenCalled();
+  fireEvent.keyDown(window, { key: "p", metaKey: true });
+  expect(api.printWindow).toHaveBeenCalledOnce();
+  expect(printToolbar()).toBeInTheDocument();
+});
+
+test("Cmd+P without an open chart is swallowed and opens nothing", () => {
+  vi.mocked(api.printWindow).mockClear();
+  renderApp({ opened: null });
+  expect(fireEvent.keyDown(window, { key: "p", metaKey: true })).toBe(false);
+  expect(screen.queryByRole("group", { name: "Baskı seçenekleri" })).toBeNull();
+  expect(api.printWindow).not.toHaveBeenCalled();
+});
+
+test("a layout shortcut pressed in the preview does not change the layout behind it", async () => {
+  renderApp();
+  fireEvent.keyDown(window, { key: "p", metaKey: true });
+  fireEvent.keyDown(window, { key: "2", code: "Digit2", metaKey: true });
+  fireEvent.click(screen.getByRole("button", { name: "Panoya dön" }));
+  expect(await screen.findByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Görüşme");
+  fireEvent.keyDown(window, { key: "2", code: "Digit2", metaKey: true });
+  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Araştırma");
+});
+
+test("the palette opens the print preview; leaving it from the keyboard moves focus to main", async () => {
+  renderApp();
+  fireEvent.keyDown(window, { key: "k", metaKey: true });
+  const palette = screen.getByRole("dialog", { name: "Komut paleti" });
+  fireEvent.change(within(palette).getByRole("combobox"), { target: { value: "baskı" } });
+  fireEvent.click(await within(palette).findByRole("option", { name: /Baskı önizleme/ }));
+  expect(printToolbar()).toBeInTheDocument();
+  const back = screen.getByRole("button", { name: "Panoya dön" });
+  expect(back).not.toHaveAttribute("aria-keyshortcuts");
+  // Odak önizlemenin içindeyken (Yazdır… düğmesi) önizleme kapanır: odak `body`'ye düşmez.
+  expect(screen.getByRole("button", { name: "Yazdır…" })).toHaveFocus();
+  fireEvent.click(back);
+  expect(document.activeElement).toBe(document.getElementById("shell-main"));
+  expect(screen.getByRole("button", { name: "Baskı önizleme" })).toHaveAttribute("aria-keyshortcuts", "Meta+P");
+});
+
+test("the sidebar's closed state is remembered at the next start", () => {
+  const first = renderApp();
+  fireEvent.keyDown(window, { key: "\\", code: "Backslash", metaKey: true });
+  expect(screen.queryByRole("complementary", { name: "Danışanlar" })).toBeNull();
+  first.unmount();
+  renderApp();
+  expect(screen.queryByRole("complementary", { name: "Danışanlar" })).toBeNull();
+  expect(screen.getByRole("button", { name: "Danışanlar" })).toHaveAttribute("aria-expanded", "false");
+  fireEvent.keyDown(window, { key: "\\", code: "Backslash", metaKey: true });
+  expect(screen.getByRole("complementary", { name: "Danışanlar" })).toBeInTheDocument();
+});
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 5785e21..5b641ef 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -254,6 +254,7 @@ export const en: Messages = {
     newChart: "New chart",
     editChart: "Edit the open chart",
     settings: "Settings",
+    print: "Open the print preview; print from the preview",
     selectLayout: "Switch to the layout at that position",
     toggleNotes: "Show or hide the notes panel",
     help: "Open this help",
@@ -380,6 +381,7 @@ export const en: Messages = {
       addChart: "Add a chart to the open client",
       editChart: "Edit the open chart",
       settings: "Settings",
+      print: "Print preview",
       toggleNotes: "Show or hide the notes panel",
       sidebar: "Show or hide the client sidebar",
       help: "Keyboard shortcuts",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 645e777..9b166b6 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -254,6 +254,7 @@ export const fr: Messages = {
     newChart: "Nouvelle carte",
     editChart: "Modifier la carte ouverte",
     settings: "Réglages",
+    print: "Ouvrir l’aperçu avant impression ; imprimer depuis l’aperçu",
     selectLayout: "Passer à la disposition de ce rang",
     toggleNotes: "Afficher ou masquer les notes",
     help: "Ouvrir cette aide",
@@ -380,6 +381,7 @@ export const fr: Messages = {
       addChart: "Ajouter une carte au client ouvert",
       editChart: "Modifier la carte ouverte",
       settings: "Réglages",
+      print: "Aperçu avant impression",
       toggleNotes: "Afficher ou masquer le panneau de notes",
       sidebar: "Afficher ou masquer la barre des clients",
       help: "Raccourcis clavier",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 0adf845..1cb2bd9 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -253,6 +253,7 @@ export const tr = {
     newChart: "Yeni harita",
     editChart: "Açık haritayı düzenle",
     settings: "Ayarlar",
+    print: "Baskı önizlemesini aç; önizlemede yazdır",
     selectLayout: "Sıradaki düzene geç",
     toggleNotes: "Notlar panelini aç/kapat",
     help: "Bu yardımı aç",
@@ -379,6 +380,7 @@ export const tr = {
       addChart: "Açık danışana harita ekle",
       editChart: "Açık haritayı düzenle",
       settings: "Ayarlar",
+      print: "Baskı önizleme",
       toggleNotes: "Notlar panelini aç/kapat",
       sidebar: "Danışan kenar çubuğunu aç/kapat",
       help: "Klavye kısayolları",
diff --git a/app/ui/palette/Palette.tsx b/app/ui/palette/Palette.tsx
index 4413047..d6ec058 100644
--- a/app/ui/palette/Palette.tsx
+++ b/app/ui/palette/Palette.tsx
@@ -94,6 +94,7 @@ export function Palette({ onClose, onRun }: PaletteProps) {
             { command: "addChart" as const, label: t("palette.command.addChart") },
             { command: "editChart" as const, label: t("palette.command.editChart") },
             { command: "toggleNotes" as const, label: t("palette.command.toggleNotes") },
+            { command: "print" as const, label: t("palette.command.print") },
           ]
         : []),
       { command: "sidebar", label: t("palette.command.sidebar") },
diff --git a/app/ui/palette/items.ts b/app/ui/palette/items.ts
index 8459303..d822721 100644
--- a/app/ui/palette/items.ts
+++ b/app/ui/palette/items.ts
@@ -4,7 +4,7 @@
 import type { ChartRecord, SearchHit, Varga } from "../bindings";
 import type { Layout } from "../layout/model";
 
-export type PaletteCommand = "newChart" | "addChart" | "editChart" | "settings" | "toggleNotes" | "sidebar" | "help";
+export type PaletteCommand = "newChart" | "addChart" | "editChart" | "settings" | "print" | "toggleNotes" | "sidebar" | "help";
 
 export type PaletteItem =
   | { kind: "client"; key: string; label: string; detail: string | null; clientId: number }
diff --git a/app/ui/shell/Shell.tsx b/app/ui/shell/Shell.tsx
index e9da5a6..67cdbbf 100644
--- a/app/ui/shell/Shell.tsx
+++ b/app/ui/shell/Shell.tsx
@@ -15,6 +15,7 @@ import { EmptyState } from "./EmptyState";
 import { leaveNotes, leaveSidebar, MAIN_ID } from "./focus";
 import { HelpDialog } from "./HelpDialog";
 import { NoticeBar, useNotices } from "./notices";
+import { loadSidebarOpen, saveSidebarOpen } from "./sidebarPref";
 import { PrintView } from "../print/PrintView";
 import "./shell.css";
 import { TopBar } from "./TopBar";
@@ -29,7 +30,9 @@ export function Shell() {
   const [help, setHelp] = useState(false);
   const [settings, setSettings] = useState(false);
   const [form, setForm] = useState<"new" | "add" | "edit" | null>(null);
-  const [sidebar, setSidebar] = useState(true);
+  const [sidebar, setSidebar] = useState(() => loadSidebarOpen());
+  // Açık/kapalı durumu bir sonraki açılışta da geçerlidir (Plan 12 D11).
+  useEffect(() => saveSidebarOpen(sidebar), [sidebar]);
   const [palette, setPalette] = useState(false);
   const toggleSidebar = () => {
     leaveSidebar();
@@ -64,8 +67,8 @@ export function Shell() {
   // (spec §3.2); baskı görünümünde (pano yokken) yok sayılır. `?` yardımı açar. Dinleyici bir
   // kez kurulur ve güncel durumu `latest`'ten okur: kayıttan hemen sonra basılan ⌘E, effect
   // yeniden kurulmadan önce de açık haritayı görür.
-  const latest = useRef({ toggleNotes, printView, opened });
-  latest.current = { toggleNotes, printView, opened };
+  const latest = useRef({ toggleNotes, printView, opened, togglePrint, printNow });
+  latest.current = { toggleNotes, printView, opened, togglePrint, printNow };
   useEffect(() => {
     const onKey = (e: KeyboardEvent) => {
       const command = matchShortcut(e);
@@ -93,6 +96,12 @@ export function Shell() {
       } else if (command?.id === "palette") {
         e.preventDefault();
         setPalette(true);
+      } else if (command?.id === "print") {
+        // ⌘P her zaman tüketilir: web görünümü panoyu kendi başına yazdırmaz.
+        e.preventDefault();
+        if (!now.opened) return;
+        if (now.printView) void now.printNow();
+        else now.togglePrint();
       }
     };
     window.addEventListener("keydown", onKey);
@@ -122,6 +131,7 @@ export function Shell() {
         }
         // Kısayollarındaki gibi: baskı görünümünde (pano ve kenar çubuğu yokken) yok sayılır.
         if (item.command === "settings") return setSettings(true);
+        if (item.command === "print") return printView ? undefined : togglePrint();
         if (item.command === "toggleNotes") {
           if (printView) return;
           leaveNotes();
diff --git a/app/ui/shell/TopBar.tsx b/app/ui/shell/TopBar.tsx
index 0994f95..647e77d 100644
--- a/app/ui/shell/TopBar.tsx
+++ b/app/ui/shell/TopBar.tsx
@@ -75,7 +75,7 @@ export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onH
           {t("shortcuts.open")}
         </button>
         {onTogglePrint && (
-          <button type="button" onClick={onTogglePrint}>
+          <button type="button" aria-keyshortcuts={printView ? undefined : "Meta+P"} onClick={onTogglePrint}>
             {printView ? t("print.back") : t("print.open")}
           </button>
         )}
diff --git a/app/ui/shortcuts.test.ts b/app/ui/shortcuts.test.ts
index 74d1172..264b9ef 100644
--- a/app/ui/shortcuts.test.ts
+++ b/app/ui/shortcuts.test.ts
@@ -14,6 +14,8 @@ test("global commands", () => {
   expect(key({ key: "n", metaKey: true })).toEqual({ id: "newChart" });
   expect(key({ key: "e", metaKey: true })).toEqual({ id: "editChart" });
   expect(key({ key: "k", metaKey: true })).toEqual({ id: "palette" });
+  expect(key({ key: "p", metaKey: true })).toEqual({ id: "print" });
+  expect(key({ key: "P", metaKey: true, shiftKey: true })).toBeNull();
   expect(key({ key: "\\", code: "Backslash", metaKey: true })).toEqual({ id: "sidebar" });
   expect(key({ key: ",", code: "Backslash", metaKey: true })).toEqual({ id: "sidebar" });
   expect(key({ key: "?", shiftKey: true })).toEqual({ id: "help" });
@@ -51,6 +53,16 @@ test("Cmd shortcuts and ? work from a select, checkbox or radio, and Cmd shortcu
   }
 });
 
+test("layout shortcuts are off while the print preview is open; Cmd+P still works", () => {
+  const preview = document.createElement("div");
+  preview.setAttribute("data-print-preview", "");
+  document.body.append(preview);
+  expect(key({ key: "2", code: "Digit2", metaKey: true })).toBeNull();
+  expect(key({ key: "p", metaKey: true })).toEqual({ id: "print" });
+  preview.remove();
+  expect(key({ key: "2", code: "Digit2", metaKey: true })).toEqual({ id: "selectLayout", index: 1 });
+});
+
 test("every listed shortcut has a description", () => {
   for (const s of SHORTCUT_LIST) expect(Object.keys(tr.shortcuts)).toContain(s.id);
   expect(Object.keys(tr.shortcuts).filter((k) => k !== "title" && k !== "open" && k !== "close" && k !== "keys" && k !== "action")).toEqual(
diff --git a/app/ui/shortcuts.ts b/app/ui/shortcuts.ts
index 8dbee7a..dc34b1b 100644
--- a/app/ui/shortcuts.ts
+++ b/app/ui/shortcuts.ts
@@ -9,13 +9,14 @@ export type Command =
   | { id: "newChart" }
   | { id: "editChart" }
   | { id: "settings" }
+  | { id: "print" }
   | { id: "palette" }
   | { id: "sidebar" }
   | { id: "help" };
 
 export interface ShortcutInfo {
   /** i18n anahtarı: `shortcuts.<id>`. */
-  id: "palette" | "sidebar" | "selectLayout" | "toggleNotes" | "newChart" | "editChart" | "settings" | "help" | "escape" | "movePanel" | "resizePanel" | "tree";
+  id: "palette" | "sidebar" | "selectLayout" | "toggleNotes" | "newChart" | "editChart" | "settings" | "print" | "help" | "escape" | "movePanel" | "resizePanel" | "tree";
   keys: string;
 }
 
@@ -25,6 +26,7 @@ export const SHORTCUT_LIST: readonly ShortcutInfo[] = [
   { id: "newChart", keys: "⌘N" },
   { id: "editChart", keys: "⌘E" },
   { id: "settings", keys: "⌘," },
+  { id: "print", keys: "⌘P" },
   { id: "selectLayout", keys: "⌘1 … ⌘9" },
   { id: "toggleNotes", keys: "⌘⇧N" },
   { id: "help", keys: "?" },
@@ -39,6 +41,11 @@ export function modalOpen(): boolean {
   return document.querySelector('[aria-modal="true"]') !== null;
 }
 
+/** Baskı önizlemesi açık mı (`PrintView` kökü). Önizlemede pano yoktur: düzen kısayolları kapalıdır. */
+export function printPreviewOpen(): boolean {
+  return document.querySelector("[data-print-preview]") !== null;
+}
+
 /**
  * Klavye olayı → genel komut. Tuş tekrarında, IME bileşiminde (WebKit'in 229 kodu dahil) ve modal açıkken `null`;
  * ⌘ kısayolları yazı alanında da çalışır, değiştiricisiz `?` yazı alanında yazıdır. Rakamlar ve ters bölü fiziksel tuştan (`code`) okunur: AZERTY'de `key`
@@ -49,13 +56,15 @@ export function matchShortcut(e: KeyboardEvent): Command | null {
   if (e.ctrlKey || e.altKey) return null;
   if (e.metaKey) {
     if (e.shiftKey) return e.key.toLowerCase() === "n" ? { id: "toggleNotes" } : null;
-    if (/^Digit[1-9]$/.test(e.code)) return { id: "selectLayout", index: Number(e.code.slice(5)) - 1 };
+    // Önizlemenin ardındaki düzen görünmeden değişmesin (Plan 04 incelemesi M4).
+    if (/^Digit[1-9]$/.test(e.code)) return printPreviewOpen() ? null : { id: "selectLayout", index: Number(e.code.slice(5)) - 1 };
     if (e.code === "Backslash" || e.key === "\\") return { id: "sidebar" };
     if (e.code === "Comma" || e.key === ",") return { id: "settings" };
     const k = e.key.toLowerCase();
     if (k === "k") return { id: "palette" };
     if (k === "n") return { id: "newChart" };
     if (k === "e") return { id: "editChart" };
+    if (k === "p") return { id: "print" };
     return null;
   }
   // Değiştiricisiz tuş yazıdır: metin girişinde komut sayılmaz. ⌘ kısayolları her yerde çalışır.
```

`tr.shortcuts` içinde `print` anahtarı `settings`'ten hemen sonra durmalıdır: `shortcuts.test.ts` anahtar sırasını `SHORTCUT_LIST` ile karşılaştırır.

- [ ] **Step 4: Testlerin geçtiğini gör**

Run: `pnpm --dir app test && pnpm --dir app build`
Expected: `Test Files  57 passed (57)`, `Tests  497 passed (497)`; build `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/shell/sidebarPref.ts app/ui/shell/sidebarPref.test.ts app/ui/shortcuts.ts app/ui/shortcuts.test.ts app/ui/palette/items.ts app/ui/palette/Palette.tsx app/ui/shell/Shell.tsx app/ui/shell/TopBar.tsx app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts app/ui/App.test.tsx
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): Cmd+P and palette command for the print preview; sidebar state is remembered"
```
Expected: iki `grep` çıktısız.

---

### Task 6: Uçtan uca: baskı ortamı, kâğıda sığma, görsel temeller

**Files:**
- Modify: `app/e2e/ipc.ts`, `app/e2e/desk.spec.ts`, `app/e2e/visual.spec.ts`
- Create: `app/e2e/visual.spec.ts-snapshots/print-{cover,charts,dasha,grahas}-visual-darwin.png` (üretilir)

**Interfaces:**
- Consumes: Task 3'ün DOM sözleşmesi, Task 4–5'in arayüz metinleri ("Baskı seçenekleri", "Vimshottari dasa", "Panoya dön"), `print_window` taklidi (mevcut).
- Produces: `IpcOptions.unknownTime?: boolean` (saati bilinmeyen fikstür); e2e 11 test; görsel 10 test.

- [ ] **Step 1: Testleri yaz**

```diff
diff --git a/app/e2e/desk.spec.ts b/app/e2e/desk.spec.ts
index fbef1bd..bb53c21 100644
--- a/app/e2e/desk.spec.ts
+++ b/app/e2e/desk.spec.ts
@@ -170,3 +170,69 @@ test("a tag written on the open chart appears in the tag filter and finds that c
   await expect(list.getByRole("button")).toContainText("Örnek Danışan");
   expect((await ipcCalls(page, "list_clients_by_tag")).at(-1)).toEqual({ tag: "Kariyer", limit: 200 });
 });
+
+test("Cmd+P opens the print preview; under print media only the sheets remain; Cmd+P again asks for the print dialog", async ({ page }) => {
+  await mockIpc(page, { session: true });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  await page.keyboard.press("Meta+p");
+  const toolbar = page.getByRole("group", { name: "Baskı seçenekleri" });
+  const sheets = page.locator("section.print-page");
+  await expect(toolbar).toBeVisible();
+  await expect(sheets).toHaveCount(4);
+  await expect(page).toHaveTitle("Örnek Danışan – Doğum haritası");
+  expect(await ipcCalls(page, "print_window")).toHaveLength(0);
+
+  await page.emulateMedia({ media: "print" });
+  await expect(toolbar).toBeHidden();
+  await expect(page.locator("header.topbar")).toBeHidden();
+  await expect(sheets.first()).toHaveCSS("break-after", "page");
+  await expect(sheets.last()).toHaveCSS("break-after", "auto");
+  // 15 mm iç dolgu = kâğıdın kenar boşluğu (96 dpi'de 56,69 px).
+  await expect(sheets.first()).toHaveCSS("padding-top", /^56\.6\d*px$/);
+  const retro = (row: string) => page.locator(`[data-page="grahas"] [data-row="${row}"] td`).nth(5);
+  await expect(retro("Saturn")).toHaveText("R");
+  await expect(retro("Rahu")).toHaveText("");
+  await expect(retro("Ketu")).toHaveText("");
+  await page.emulateMedia({ media: "screen" });
+
+  await page.keyboard.press("Meta+p");
+  await expect.poll(() => ipcCalls(page, "print_window")).toHaveLength(1);
+  await toolbar.getByRole("checkbox", { name: "Vimshottari dasa" }).uncheck();
+  await expect(sheets).toHaveCount(3);
+  await page.getByRole("button", { name: "Panoya dön" }).click();
+  await expect(page.getByRole("region", { name: "Harita D1", exact: true })).toBeVisible();
+  await expect(page).toHaveTitle("Jyotish Desk");
+});
+
+test("print sheets are paper-white in the dark theme and, at their longest, each fits one A4 and one Letter page", async ({ page }) => {
+  // Saati bilinmeyen harita: kapakta uyarı, harita ve dasa sayfalarında not basılır.
+  await mockIpc(page, { session: true, unknownTime: true });
+  await page.addInitScript(() => localStorage.setItem("jyotish-desk.theme.v1", "dark"));
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  await page.keyboard.press("Meta+p");
+  const sheets = page.locator("section.print-page");
+  await expect(sheets).toHaveCount(4);
+  await expect(page.locator('[data-page="charts"] .print-note')).toContainText("Doğum saati bilinmiyor");
+  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
+  for (const sheet of await sheets.all()) {
+    await expect(sheet).toHaveCSS("background-color", "rgb(255, 255, 255)");
+    await expect(sheet).toHaveCSS("color", "rgb(44, 44, 42)");
+  }
+  await expect(page.locator('[data-page="charts"] .chart-cell').first()).toHaveCSS("stroke", "rgb(83, 74, 183)");
+
+  // WKWebView yazdırırken içeriği yaklaşık 1,067 kat büyütür (scripts/print-probe.sh ölçümü):
+  // kâğıdın CSS karşılığı A4'te 744 × 1053 px, Letter'da 765 × 990 px. En dar genişlik ve en
+  // kısa yükseklik birlikte denenir.
+  await page.setViewportSize({ width: 744, height: 990 });
+  await page.emulateMedia({ media: "print" });
+  for (const sheet of await sheets.all()) {
+    const box = (await sheet.boundingBox())!;
+    expect(box.width).toBeLessThanOrEqual(744);
+    expect(box.height).toBeGreaterThan(200);
+    expect(box.height).toBeLessThanOrEqual(990);
+    // Hiçbir tablo ya da çizim kâğıdın iç genişliğinden taşmaz.
+    expect(await sheet.evaluate((el) => el.scrollWidth <= el.clientWidth)).toBe(true);
+  }
+});
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index 83fc088..249914b 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -9,11 +9,18 @@ export const FIXTURE = JSON.parse(readFileSync(new URL("../ui/test/fixtures/delh
 export interface IpcOptions {
   /** `restore_session` dönüşü: `true` Delhi haritası, `false` oturum yok. */
   session: boolean;
+  /** Doğum saati bilinmeyen harita: uyarı ve notlar basılır (baskının en uzun sayfaları). */
+  unknownTime?: boolean;
 }
 
-export async function mockIpc(page: Page, { session }: IpcOptions) {
+export async function mockIpc(page: Page, { session, unknownTime = false }: IpcOptions) {
   await page.addInitScript(
-    ({ fixture, session }) => {
+    ({ fixture, session, unknownTime }) => {
+      if (unknownTime) {
+        fixture.opened.chart_record.birth.time = null;
+        fixture.opened.chart_record.birth.time_certainty = "Unknown";
+        fixture.opened.chart_record.resolved.warnings = ["TimeUnknownNoon"];
+      }
       const calls: { cmd: string; args: unknown }[] = [];
       Object.assign(window, { __ipcCalls: calls });
       // İkinci danışan: aynı hesaplanmış harita, başka kimlik ve ad (arayüz hesap yapmaz).
@@ -102,7 +109,7 @@ export async function mockIpc(page: Page, { session }: IpcOptions) {
         },
       });
     },
-    { fixture: FIXTURE, session },
+    { fixture: FIXTURE, session, unknownTime },
   );
 }
 
diff --git a/app/e2e/visual.spec.ts b/app/e2e/visual.spec.ts
index e9743e0..0ef2618 100644
--- a/app/e2e/visual.spec.ts
+++ b/app/e2e/visual.spec.ts
@@ -22,3 +22,17 @@ for (const theme of ["light", "dark"] as const) {
     });
   }
 }
+
+// Baskı sayfaları (Plan 12): önizlemedeki dört A4 kâğıt. Koyu tema seçilidir; kâğıt yine de
+// beyazdır (`.paper`). Kuzey biçimi varsayılandır.
+for (const sheet of ["cover", "charts", "dasha", "grahas"] as const) {
+  test(`print sheet ${sheet}`, async ({ page }) => {
+    await mockIpc(page, { session: true });
+    await page.addInitScript(() => localStorage.setItem("jyotish-desk.theme.v1", "dark"));
+    await page.goto("/");
+    await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+    await page.keyboard.press("Meta+p");
+    await expect(page.locator("section.print-page")).toHaveCount(4);
+    await expect(page.locator(`[data-page="${sheet}"]`)).toHaveScreenshot(`print-${sheet}.png`);
+  });
+}
```

Sığma testindeki 744 × 990 px sınırı D3'ün ölçümüdür (A4'ün genişliği, Letter'ın yüksekliği); sınır gevşetilmez. Bir sayfa taşarsa düzen küçültülür (ör. `.print-chart` genişliği).

- [ ] **Step 2: e2e**

Run: `pnpm --dir app build && pnpm --dir app e2e`
Expected: build `✓ built` (`tsc --noEmit -p e2e` dahil); `11 passed`.

- [ ] **Step 3: Görsel temelleri üret (yalnızca yerel, darwin)**

```bash
pnpm --dir app e2e:visual
pnpm --dir app e2e:visual
rm -rf app/test-results
git status --short app/e2e
```
Expected: ilk koşu dört yeni temeli yazar ve onları "failed" sayar (`6 passed`, `4 failed`: "A snapshot doesn't exist … writing actual"); ikinci koşu `10 passed`. `git status` yalnızca üç değişen `.ts` dosyasını ve dört yeni `print-*-visual-darwin.png` dosyasını gösterir; mevcut altı `d1-*.png` listede **yoktur** (C11). Mevcut bir temel değiştiyse dur: Task 3'ün `interactive` değişikliği varsayılan çizimi bozmuştur.

- [ ] **Step 4: Temellere bak**

Dört PNG'yi aç ve gözle denetle: beyaz kâğıt (koyu tema seçiliyken), mor çizgi, kapakta doğum satırları ve "Hesap ayarları", iki harita alt alta ve lagna evi kalın çizgili, dokuz dasa tablosu 3 × 3, graha tablosunda Satürn satırında `R`, Rahu ve Ketu satırlarında retro hücresi boş. Bir sorun varsa temeli sil, düzeni düzelt, Step 3'ü yinele.

- [ ] **Step 5: Commit**

```bash
git add app/e2e/ipc.ts app/e2e/desk.spec.ts app/e2e/visual.spec.ts app/e2e/visual.spec.ts-snapshots/print-cover-visual-darwin.png app/e2e/visual.spec.ts-snapshots/print-charts-visual-darwin.png app/e2e/visual.spec.ts-snapshots/print-dasha-visual-darwin.png app/e2e/visual.spec.ts-snapshots/print-grahas-visual-darwin.png
git diff --cached -U0 -- '*.ts' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(e2e): print preview under print media, sheets fit A4 and Letter, visual baselines for the four sheets"
```
Expected: iki `grep` çıktısız.

---

### Task 7: Baskı sondası (gerçek yazdırma yolu, PDF denetimi); bulgu belgesi ve spec düzeltmeleri

**Files:**
- Create: `scripts/print-probe.sh`, `scripts/print-probe/probe.swift`
- Modify: `docs/spikes/2026-11-24-wkwebview-print.md`, `docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md`, `docs/superpowers/specs/2026-11-24-ui-design.md`

**Interfaces:**
- Consumes: Task 3–5'in DOM sözleşmesi (`h1.topbar-client`, `section.print-page`), `⌘P`, `app/ui/test/fixtures/delhi.json`.
- Produces: `scripts/print-probe.sh` (bağımsız değişkensiz; çıkış kodu 0 ve dört `print-probe: … ok` satırı). Task 8 koşar.

Sonda uygulamanın kendisini değil, aynı arayüzü aynı WebKit çağrısıyla yazdırır: Vite sunucusunu 1423 portunda başlatır, Tauri IPC'yi fikstürden taklit eder, `⌘P` ile önizlemeyi açar, `printOperation(with:)` ile (kenarlar 0, wry ile aynı) PDF'e yazar ve PDFKit ile denetler. Pencere gösterilmez; gerçek danışan verisi okunmaz.

- [ ] **Step 1: Sondayı yaz**

`scripts/print-probe/probe.swift`:

```swift
// Baskı sondası (Plan 12 D2): arayüzün baskı önizlemesini, uygulamanın kullandığı yolla
// (WKWebView `printOperation(with:)`, kenar boşlukları 0; wry `print()` ile aynı) pencere
// göstermeden PDF'e yazdırır ve PDF'i denetler: sayfa sayısı, kâğıt boyutu, her sayfanın metni.
// Tauri IPC, arayüz fikstüründen (ui/test/fixtures/delhi.json) taklit edilir; gerçek danışan
// verisi okunmaz. Kullanım: probe <url> <fikstür.json> <çıktı.pdf> <a4|letter> <exact|unknown>
// `unknown`: doğum saati bilinmeyen harita (uyarı ve notlar basılır; en uzun sayfalar).
import AppKit
import PDFKit
import WebKit

func fail(_ message: String) -> Never {
  FileHandle.standardError.write("print-probe: \(message)\n".data(using: .utf8)!)
  exit(1)
}

let args = CommandLine.arguments
guard args.count == 6 else { fail("usage: probe <url> <fixture.json> <out.pdf> <a4|letter> <exact|unknown>") }
guard let url = URL(string: args[1]) else { fail("bad url") }
guard let fixture = try? String(contentsOfFile: args[2], encoding: .utf8) else { fail("fixture not readable") }
let out = URL(fileURLWithPath: args[3])
// Punto cinsinden kâğıt: A4 210 × 297 mm, Letter 8,5 × 11 in.
let paper = args[4] == "letter" ? NSSize(width: 612, height: 792) : NSSize(width: 595.28, height: 841.89)
let unknownTime = args[5] == "unknown"
let label = "\(args[4]) \(args[5])"
/// Her sayfada bulunması gereken metin (Türkçe arayüz, fikstür haritası).
let expected = ["Örnek Danışan", "D9 · Navamsa", "Vimshottari dasa", "Nakshatra"]

let mock = """
(() => {
  const f = \(fixture);
  if (\(unknownTime)) {
    f.opened.chart_record.birth.time = null;
    f.opened.chart_record.birth.time_certainty = "Unknown";
    f.opened.chart_record.resolved.warnings = ["TimeUnknownNoon"];
  }
  const answers = {
    restore_session: f.opened, get_app_prefs: f.prefs, transits_at: f.transit,
    list_clients: [f.opened.client], list_charts: [f.opened.chart_record],
    list_tags: [], list_notes: [], list_clients_by_tag: [], search_clients: [],
  };
  window.__TAURI_INTERNALS__ = {
    invoke: async (cmd) => structuredClone(cmd in answers ? answers[cmd] : null),
    transformCallback: () => 0,
    unregisterCallback: () => {},
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main", windowLabel: "main" } },
  };
})();
"""

final class Probe: NSObject, WKNavigationDelegate {
  let web: WKWebView
  let window: NSWindow
  var tries = 0

  override init() {
    let config = WKWebViewConfiguration()
    config.websiteDataStore = .nonPersistent()
    config.userContentController.addUserScript(WKUserScript(source: mock, injectionTime: .atDocumentStart, forMainFrameOnly: true))
    window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 1200, height: 800), styleMask: [.titled], backing: .buffered, defer: false)
    web = WKWebView(frame: window.contentView!.bounds, configuration: config)
    window.contentView!.addSubview(web)
    super.init()
    web.navigationDelegate = self
  }

  func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) { waitFor("document.querySelector('h1.topbar-client') !== null", then: openPreview) }
  func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) { fail("page did not load") }
  func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) { fail("page did not load (is the dev server running?)") }

  /// `condition` doğru olana dek 100 ms arayla dener; 10 saniyede olmazsa çıkar.
  func waitFor(_ condition: String, then: @escaping () -> Void) {
    web.evaluateJavaScript(condition) { value, _ in
      if value as? Bool == true { self.tries = 0; then(); return }
      self.tries += 1
      if self.tries > 100 { fail("timed out waiting for: \(condition)") }
      DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { self.waitFor(condition, then: then) }
    }
  }

  func openPreview() {
    web.evaluateJavaScript("window.dispatchEvent(new KeyboardEvent('keydown', { key: 'p', metaKey: true, cancelable: true })); true") { _, _ in
      self.waitFor("document.querySelectorAll('section.print-page').length === 4", then: self.print)
    }
  }

  func print() {
    let info = NSPrintInfo()
    info.paperSize = paper
    info.topMargin = 0
    info.bottomMargin = 0
    info.leftMargin = 0
    info.rightMargin = 0
    info.jobDisposition = .save
    info.dictionary()[NSPrintInfo.AttributeKey.jobSavingURL] = out
    let operation = web.printOperation(with: info)
    operation.showsPrintPanel = false
    operation.showsProgressPanel = false
    operation.view?.frame = web.bounds
    let title = operation.jobTitle ?? ""
    operation.runModal(for: window, delegate: nil, didRun: nil, contextInfo: nil)
    waitForFile(title: title)
  }

  func waitForFile(title: String) {
    guard let pdf = PDFDocument(url: out), pdf.pageCount > 0 else {
      tries += 1
      if tries > 100 { fail("no PDF was written") }
      DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { self.waitForFile(title: title) }
      return
    }
    // Yazım sürerken okunmuş olabilir: sayfa sayısı durulana dek bir kez daha bakılır.
    DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { self.check(title: title) }
  }

  func check(title: String) {
    guard let pdf = PDFDocument(url: out) else { fail("PDF not readable") }
    guard pdf.pageCount == expected.count else { fail("\(label): expected \(expected.count) pages, got \(pdf.pageCount) (a sheet overflows its page)") }
    for (i, text) in expected.enumerated() {
      let page = pdf.page(at: i)!
      let box = page.bounds(for: .mediaBox)
      guard abs(box.width - paper.width) < 1, abs(box.height - paper.height) < 1 else { fail("page \(i + 1) is \(box.width) × \(box.height) pt") }
      guard (page.string ?? "").contains(text) else { fail("page \(i + 1) does not contain “\(text)” (text is not selectable or the page order changed)") }
    }
    guard !(pdf.string ?? "").contains("Yazdır…") else { fail("the toolbar was printed") }
    guard title == "Örnek Danışan – Doğum haritası" else { fail("job title is “\(title)”") }
    if unknownTime, !(pdf.page(at: 1)?.string ?? "").contains("Doğum saati bilinmiyor") { fail("the unknown-time note is missing on the charts page") }
    Swift.print("print-probe: \(label) ok, \(pdf.pageCount) pages, job title “\(title)”")
    exit(0)
  }
}

let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let probe = Probe()
probe.web.load(URLRequest(url: url))
app.run()
```

`scripts/print-probe.sh`:

```sh
#!/bin/sh
# Baskı önizlemesini A4 ve Letter kâğıda PDF olarak yazdırır ve sonucu denetler (Plan 12 D2).
# Yalnızca macOS; Xcode komut satırı araçları (swiftc) gerekir. Kullanım: scripts/print-probe.sh
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
port=1423
server=
trap '[ -z "$server" ] || kill "$server" 2>/dev/null || true; rm -rf "$work"' EXIT

xcrun swiftc -O "$root/scripts/print-probe/probe.swift" -o "$work/probe"
(cd "$root/app" && exec pnpm exec vite --port "$port" --strictPort >"$work/vite.log" 2>&1) &
server=$!
tries=0
until curl -sf "http://localhost:$port/" >/dev/null; do
  tries=$((tries + 1))
  [ "$tries" -lt 100 ] || { echo "print-probe: dev server did not start" >&2; exit 1; }
  sleep 0.2
done
for paper in a4 letter; do
  for time in exact unknown; do
    "$work/probe" "http://localhost:$port/" "$root/app/ui/test/fixtures/delhi.json" "$work/$paper-$time.pdf" "$paper" "$time"
  done
done
```

```bash
chmod +x scripts/print-probe.sh
```

- [ ] **Step 2: Sondanın hatayı gördüğünü kanıtla**

Geçici olarak `app/ui/print/print.css`'te `.print-chart` genişliğini `82mm`'den `130mm`'ye çıkar ve koş:

Run: `sh scripts/print-probe.sh; echo "exit=$?"`
Expected: `print-probe: a4 exact: expected 4 pages, got 5 (a sheet overflows its page)` ve `exit=1`. Sonra değişikliği geri al: `git checkout app/ui/print/print.css`.

- [ ] **Step 3: Sondayı koş**

Run: `sh scripts/print-probe.sh; echo "exit=$?"`
Expected (Swift derleyici uyarıları yok sayılır):
```
print-probe: a4 exact ok, 4 pages, job title “Örnek Danışan – Doğum haritası”
print-probe: a4 unknown ok, 4 pages, job title “Örnek Danışan – Doğum haritası”
print-probe: letter exact ok, 4 pages, job title “Örnek Danışan – Doğum haritası”
print-probe: letter unknown ok, 4 pages, job title “Örnek Danışan – Doğum haritası”
exit=0
```
`page N does not contain …` hatası metin katmanının bozulduğunu gösterir (D6: kâğıtta sistem fontu kullanılmamalıdır).

- [ ] **Step 4: Bulgu belgesini güncelle**

`docs/spikes/2026-11-24-wkwebview-print.md`'de `**Durum:**` satırını ve `## Bulgular` bölümünü (bölüm başlığından dosya sonuna) aşağıdakilerle değiştir; aradaki "Nasıl test edilir" ve "Kontrol listesi" bölümleri tarihçe olarak durur.

`**Durum:**` satırı:

```markdown
**Durum:** Kapandı (Plan 12, 2026-11-26). M7 baskı düzenleri HTML/CSS ile yapıldı; Typst yedeğine gerek görülmedi. Denetim artık `scripts/print-probe.sh` ile otomatiktir; aşağıdaki liste M0'daki elle denetimin tarihçesidir.
```

`## Bulgular` bölümü:

```markdown
## Bulgular

Ölçüm: macOS 27.2, WKWebView `printOperation(with:)`, `NSPrintInfo` kenarları 0 (wry 0.57.3 `print()` ile aynı çağrı), çıktı PDF. Sonda: `scripts/print-probe.sh`.

| # | Kontrol | Sonuç |
|---|---|---|
| 1 | `@page { size: A4 }` | Kâğıt boyutu yazdırma penceresinden gelir; A4 seçiliyken sayfa 595 × 842 pt. Düzenler Letter'a da sığar. |
| 2 | 15 mm kenar boşluğu | wry kâğıt kenarlarını 0 yapar; boşluk sayfanın iç dolgusudur (`.print-page { padding: 15mm }`). WebKit içeriği yaklaşık 1,067 kat büyütür: dolgu kâğıtta 16 mm olur. |
| 3 | `break-after: page` | Çalışır: her `.print-page` yeni kâğıtta başlar. Kabuk baskıda blok akışına döner (`shell.css`). |
| 4 | SVG vektör | Evet. |
| 5 | Metin seçilebilir, Türkçe karakterler | Adlı yazı tipiyle evet (Helvetica Neue). **Sistem fontu Türkçe `lang` altında metin katmanında noktalı "i"yi yitirir** ("Tarih" → "Tar!h"); kâğıtta sistem fontu kullanılmaz. |
| 6 | Devanagari | Kâğıtta Devanagari metin basılmıyor; denetlenmedi. |
| 7 | `.no-print` | Araç çubuğu ve üst çubuk PDF'te yok. |
| 8 | Koyu temada beyaz zemin | Evet (`.paper` ve `@media print` paleti). |
| 9 | Üst/alt bilgi | WKWebView yazdırma işi üst/alt bilgi eklemiyor. |
| 10 | Letter | Dört düzen dört sayfa; saati bilinmeyen haritada da. |

Ek bulgular:

- Kâğıdın CSS karşılığı: A4 744 × 1053 px, Letter 765 × 990 px. Düzenler bu bütçeyle tasarlanır; e2e testi aynı sınırı denetler.
- Yazdırma işinin adı, PDF'in başlık alanı ve "PDF olarak kaydet"in önerdiği dosya adı `document.title`'dan gelir; önizleme açıkken başlık "Danışan – Harita"dır.
```

- [ ] **Step 5: Spec'leri düzelt**

`docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md` §4'te şu paragrafı:

```markdown
**PDF:** Print'e özel HTML düzeni ve macOS'un yerleşik "PDF olarak kaydet" özelliği. M0'da spike yapılır. WKWebView `@page` desteği yetersiz çıkarsa yedek plan Rust tarafında Typst.
```

şununla değiştir:

```markdown
**PDF:** Print'e özel HTML düzeni ve macOS'un yerleşik "PDF olarak kaydet" özelliği. Yazdırma penceresini Rust açar (`print_window`); arayüz dosya yolu vermez. Kenar boşluğu sayfanın iç dolgusudur, çünkü WKWebView yazdırma işinde kâğıt kenarları sıfırdır. Yol `scripts/print-probe.sh` ile A4 ve Letter kâğıtta denetlenir (bulgular: `docs/spikes/2026-11-24-wkwebview-print.md`). Typst yedeğine gerek görülmedi.
```

`docs/superpowers/specs/2026-11-24-ui-design.md` §7'de şu iki maddeyi:

```markdown
- Pano baskıya doğrudan aktarılmaz. M7'de ayrı A4 baskı düzenleri tanımlanır: kapak, D1+D9, dasa ve graha tablosu.
- Baskı her zaman açık tema, tek renk mor mürekkep kullanır ve siyah-beyaz yazıcıda okunur.
```

şunlarla değiştir:

```markdown
- Pano baskıya doğrudan aktarılmaz. Dört ayrı A4 baskı düzeni vardır: kapak (doğum bilgisi, hesap ayarları, uyarılar), D1+D9, Vimshottari dasa (mahadasa ve antardasa) ve graha tablosu. Her düzen tek kâğıda sığar (A4 ve Letter).
- `Cmd+P` baskı önizlemesini açar; önizlemede basılacak sayfalar ve harita biçimi seçilir, `Cmd+P` ya da "Yazdır…" macOS yazdırma penceresini açar. PDF o pencerenin "PDF olarak kaydet" seçeneğidir. Seçimler kullanıcıya aittir ve saklanır.
- Baskı her zaman açık tema, tek renk mor mürekkep kullanır ve siyah-beyaz yazıcıda okunur. Önizleme koyu temada da kâğıt beyazıdır.
- Kâğıtta lagna evi kalın çizgiyle belirtilir (dolgu basılmaz); `R`, `c` ve altı çizili işaretlerinin açıklaması harita sayfasına basılır.
- Kâğıtta yazı tipi Helvetica Neue'dür: sistem fontu Türkçe metinde PDF'in metin katmanını bozar.
- Notlar, transit ve doğum saati bilinmeyen haritada lagna ile evler basılmaz.
```

- [ ] **Step 6: Commit**

```bash
git add scripts/print-probe.sh scripts/print-probe/probe.swift docs/spikes/2026-11-24-wkwebview-print.md docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md docs/superpowers/specs/2026-11-24-ui-design.md
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(print): headless print probe checks the PDF on A4 and Letter; spike findings and spec text"
```
Expected: iki `grep` çıktısız. (v1 spec'in §10'undaki kural satırı değişmediği için diff'te yoktur.)

---

### Task 8: Son doğrulama ve elle denetim kapısı

Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle kapanır ve bu görev baştan koşulur.

**Files:**
- Yok (yalnızca doğrulama). İlerleme notları depo dışıdır; `git add` edilmez.

**Interfaces:**
- Consumes: Task 1–7'nin tamamı.
- Produces: yeşil kapılar ve elle denetim sonucu (ilerleme notlarına yazılır).

- [ ] **Step 1: Rust kapıları (değişmediğinin kanıtı)**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
git diff ecf7ed5..HEAD --stat -- crates app/src-tauri Cargo.toml Cargo.lock app/package.json app/pnpm-lock.yaml
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo deny check
```
Expected: `git diff --stat` çıktısız (Rust, bağlamalar ve bağımlılık dosyaları değişmedi); fmt ve clippy çıktısız; testler: `jyotish-core` lib `110 passed`, golden `8 passed`, `store` lib `80 passed; 0 failed; 1 ignored`, `jyotish-desk` lib `34 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; deny `advisories ok, bans ok, licenses ok, sources ok`.

- [ ] **Step 2: Arayüz kapıları**

```bash
pnpm --dir app test
pnpm --dir app build
pnpm --dir app e2e
pnpm --dir app e2e:visual
rm -rf app/test-results
sh scripts/print-probe.sh
git status --short
```
Expected: `Test Files  57 passed (57)`, `Tests  500 passed (500)`; build `✓ built`; e2e `11 passed`; görsel `10 passed`; sonda dört `ok` satırı; `git status --short` boş.

- [ ] **Step 3: Kural taramaları (`ecf7ed5`'ten bu yana)**

```bash
git diff ecf7ed5..HEAD -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log ecf7ed5..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff ecf7ed5..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff ecf7ed5..HEAD --name-only | grep -vE '^(app|scripts|docs)/'
echo "trailers: $(git log ecf7ed5..HEAD --format=%B | grep -c '^Co-Authored-By: ') commits: $(git rev-list --count ecf7ed5..HEAD)"
git diff ecf7ed5..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
git diff ecf7ed5..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\(|window\.print\('
grep -rnE '#[0-9a-fA-F]{3,8}\b' app/ui/print --include='*.css' --include='*.tsx' --include='*.ts' | grep -v '\.test\.'
grep -n '"dialog:\|"fs:' app/src-tauri/capabilities/*.json
grep -rn 'tauri_plugin_fs' app/src-tauri/src
```
Expected: `trailers:` satırındaki iki sayı eşit; bunun dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur). İlk tarama bu plan dosyasını da kapsar; plan dosyasında eşleşme yoktur.

- [ ] **Step 4: Elle denetim kapısı (`pnpm --dir app tauri dev`)**

Gerçek uygulamada, deneme danışanlarıyla (gerçek danışan verisi kullanılmaz). Her madde ilerleme notlarına "geçti" ya da bulgu olarak yazılır; bulgu varsa düzeltme commit'i ve bu görev baştan.

- [ ] Açık harita varken `⌘P` baskı önizlemesini açar: araç çubuğu ve dört beyaz kâğıt. Açık harita yokken `⌘P` hiçbir şey yapmaz ve üst çubukta "Baskı önizleme" düğmesi yoktur.
- [ ] Koyu temada önizlemedeki kâğıtlar beyaz, çizgiler mor, metin koyudur.
- [ ] Önizlemede `⌘P` (ve "Yazdır…") macOS yazdırma penceresini uygulama penceresine bağlı sayfa olarak açar; pencerenin önizlemesinde dört sayfa vardır ve araç çubuğu ile üst çubuk görünmez.
- [ ] Yazdırma penceresinde "PDF ▸ PDF olarak kaydet" önerilen dosya adı olarak danışan ve harita adını verir. PDF Önizleme uygulamasında açılır: dört sayfa, A4; her sayfada kenar boşluğu vardır (yaklaşık 16 mm) ve hiçbir içerik kâğıt kenarına yapışmaz.
- [ ] PDF'te metin seçilir ve "Tarih" araması kapağı bulur (noktalı "i" metin katmanında doğrudur); yakınlaştırınca harita çizgileri ve yazıları keskin durur (vektör).
- [ ] PDF'te Satürn (retro ise) `R` taşır; Rahu ve Ketu ne haritada ne tabloda `R` taşır; yanık gezegen haritada `c`, tabloda "yanık"tır; lagna evi kalın çizgilidir; harita sayfasının altında işaret açıklaması vardır.
- [ ] Siyah-beyaz çıktı (yazdırma penceresinde "Siyah Beyaz" ya da gri tonlamalı PDF) okunur: hiçbir bilgi yalnızca renkle taşınmaz.
- [ ] Kâğıt boyutu "US Letter" seçilince yine dört sayfa çıkar; hiçbir düzen ikinci kâğıda taşmaz.
- [ ] "Vimshottari dasa" kutusu kapatılınca önizleme üç kâğıda iner ve numaralar "1 / 3 … 3 / 3" olur; tek sayfa seçiliyken o kutu kapatılamaz ve nedeni yazar. Harita biçimi "Güney" seçilince iki harita da Güney biçiminde çizilir. Uygulama kapanıp açılınca seçimler durur.
- [ ] Doğum saati bilinmeyen bir deneme haritasında: kapakta "Saat bilinmiyor" uyarısı metinle basılır; haritalarda lagna ve ev yoktur ve not vardır; graha tablosunda lagna satırı ve ev sütunu yoktur, Ay "≈" ile işaretlidir ve tablonun üstünde bunu açıklayan not vardır; dasa sayfasında yaklaşıklık notu vardır; belge yine dört sayfadır.
- [ ] Önizlemede `⌘2` düzeni değiştirmez; "Panoya dön" panoyu aynı düzenle geri getirir ve odak görünür bir öğededir. `⌘K` → "Baskı önizleme" de önizlemeyi açar.
- [ ] Önizleme açıkken yazdırma penceresinden vazgeçmek (İptal) hata göstermez; önizleme durur.
- [ ] Kenar çubuğu `⌘\` ile kapatılıp uygulama yeniden açılınca kapalı gelir; açılıp yeniden açılınca açık gelir.
- [ ] VoiceOver: önizlemede araç çubuğu "Baskı seçenekleri" grubu olarak, her kâğıt başlığıyla bölge olarak, haritalar "D1 haritası" ve "D9 haritası" resimleri olarak duyurulur.
- [ ] Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, not gövdesi, doğum verisi ya da dosya yolu görünmez.

Kapı geçtiyse ilerleme notlarına `Plan 12 Task 8 elle denetim: geçti (<tarih>)` yazılır. Bu adım commit üretmez.

---

## Başarı ölçütleri

- Spec M7 tamamdır: açık harita dört A4 düzenle yazdırılır ve macOS yazdırma penceresinden PDF olarak kaydedilir; baskı her zaman açık tema ve tek renktir; retro, yanma, vargottama ve lagna renksiz işaretlerle basılır.
- M0 baskı denemesi kapanmıştır: `PrintSpike` silinmiş, bulgu belgesi doldurulmuş, Typst kararı verilmiştir; yazdırma yolu `scripts/print-probe.sh` ile A4 ve Letter'da denetlenir.
- Rust, bağlamalar, şema ve bağımlılıklar değişmemiştir; C1–C17 taramaları temizdir.
- Kapılar: Vitest 57/500, e2e 11, görsel 10, sonda 4 `ok`; Rust sayıları tabandaki gibidir; elle denetim kapısı geçmiştir.

## Bu planın kapsamadıkları

- M8: imzalama, notarizasyon, güncelleme, kapalı beta (Plan 10 kapsam dışı 3 ve 4 ile).
- Plan 10 kapsam dışı 15 (paletten notun kendisine gitme) ve 16'nın boyutlandırma yarısı (kenar çubuğu genişliği): kapalı beta geri bildirimi isterse (D11).
- Plan 10 kapsam dışı 2 (JHD), 5 (göç 2), 8 (M6-5), 14: Plan 11'de olduğu gibi dışarıda.
- Notların, transit konumlarının, ashtakavarga ve panchang'ın basılması; D1 ve D9 dışındaki vargaların basılması; kâğıda basım tarihi (D4, C4).
- Ayrı "PDF olarak dışa aktar" komutu ve Typst (D1).
- Yatay sayfa, A4 dışı özel düzenler, sayfa başına birden fazla danışan, toplu yazdırma.
- Sondanın CI'da koşması (D2) ve Windows/WebView2 baskı yolu (yalnızca macOS).
- `transit/TransitTablePanel.test.tsx` "apply sends the wall-clock time…" testi bu plan yazılırken yük altında bir kez (yedi koşuda) zaman aşımıyla düştü; baskıyla ilgisizdir. Yinelenirse ayrı bir düzeltme commit'iyle ele alınır; sayaçlar bu yüzden elle değiştirilmez.

---

## Self-Review

**1. Spec kapsamı.**

| Gereksinim | Görev |
|---|---|
| v1 spec §3 "Print ve PDF", §4 "PDF" (HTML düzeni + "PDF olarak kaydet"), §11 M7 | Task 3, 4, 7 |
| UI spec §7: kapak, D1+D9, dasa, graha tablosu; pano doğrudan basılmaz | Task 2, 3 |
| UI spec §2.1/§7: baskı açık tema, yalnızca `--ink` ve `--text`, siyah-beyazda okunur | Task 1, 3, 6 (renk denetimi), Task 8 (elle) |
| UI spec §2.3: retro düz `R` (Rahu/Ketu'da yok), `c`, altı çizili; renk yok | Task 2 (model testi), Task 3 (çizim testi), Task 6 (e2e) |
| v1 spec §7: bilinmeyen doğum saati | Task 2, 3, 6, 7 (`unknown`) |
| v1 spec §12: baskı düzenleri soyut | Task 2 (D8) |
| M0 denemesinin kararı | Task 7 (D2) |
| Kısayol, palet, erişilebilirlik (UI spec §4, §6) | Task 4, 5 |
| Plan 04/10 ertelenenleri | Task 3, 4, 5 |
| Son doğrulama ve elle denetim | Task 8 |

Boşluk yok; kapsam dışı maddeler yukarıda listelidir.

**2. Yer tutucu taraması.** "TODO", "TBD", "daha sonra", "uygun hata işleme" yok. Task 8 Step 4'teki `<tarih>` denetimin yapıldığı günün yazılacağı yerdir, kod değildir. Her kod adımı tam içerik ya da `ecf7ed5` ağacına karşı üretilmiş diff taşır.

**3. Tip tutarlılığı.**
- `PrintPageId` (`"cover" | "charts" | "dasha" | "grahas"`) Task 1'de tanımlanır; `PrintOptions.pages`, modelin sayfa `id`'leri, `data-page` öznitelikleri, `print.page.*` i18n anahtarları, e2e ve görsel testler ile sondanın sayfa sırası aynı dört değeri aynı sırada kullanır.
- `buildPrintDocument(t, opened, options)` Task 2'de tanımlanır; Task 3 testleri ve Task 4 `PrintView` aynı imzayla çağırır. `PrintDocument.title` Task 4'te `document.title`'a yazılır; Task 6 (`toHaveTitle`) ve Task 7 (iş başlığı) aynı metni bekler: "Örnek Danışan – Doğum haritası" (kısa çizgi U+2013).
- `ChartSvgProps.interactive` Task 3'te eklenir ve yalnızca `PrintDocumentView` `false` verir.
- `PrintView` özellikleri `{ error, onPrint }` Task 4'te tanımlanır; `Shell` (`printError`, `printNow`) ve `PrintView.test.tsx` aynı adları kullanır. `togglePrint` ve `printNow` Task 4'te `Shell`'de tanımlanır, Task 5 `latest` başvurusuna ekler.
- `[data-print-preview]` Task 4'te `PrintView` köküne konur; Task 5 `printPreviewOpen()` aynı seçiciyi okur.
- i18n: `print.open` "Baskı önizleme", `print.back` "Panoya dön", `print.toolbar` "Baskı seçenekleri", `print.print` "Yazdır…", `print.page.dasha` "Vimshottari dasa", `print.lastPage` "En az bir sayfa seçili olmalıdır." — Vitest, e2e ve sonda aynı metinleri arar.
- `IpcOptions.unknownTime` Task 6'da eklenir; yalnızca Task 6'nın ikinci testi verir. Sonda aynı değişikliği kendi taklidinde `unknown` bağımsız değişkeniyle yapar.

**4. Sayaç zinciri.**

| Görev sonrası | Vitest dosya/test | e2e | Görsel | Not |
|---|---|---|---|---|
| Taban `ecf7ed5` | 54/461 | 9 | 6 | Rust: core 110, golden 8, store 80 (+1 ignored), desk lib 34, bindings 1, compute_chart 1, open_client 1, ui_fixture 2 |
| Task 1 | 55/467 | | | `print/options.test.ts` 5, `tokens.test.ts` +1 |
| Task 2 | 56/474 | | | `print/model.test.ts` 7 |
| Task 3 | 57/484 | | | `PrintDocumentView.test.tsx` 9, `ChartSvg.test.tsx` +1 |
| Task 4 | 56/489 | | | `PrintSpike.test.tsx` −2, `shell/PrintView.test.tsx` −1, `print/PrintView.test.tsx` +7, `App.test.tsx` +1 |
| Task 5 | 57/497 | | | `sidebarPref.test.ts` 2, `shortcuts.test.ts` +1, `App.test.tsx` +5 |
| Task 4 inceleme düzeltmesi (`ef5c399`, Task 5'ten sonra) | 57/498 | | | `App.test.tsx` +1 (yeniden denemede odak) |
| Task 6 | 57/498 | 11 | 10 | mevcut altı görsel temel değişmez |
| Task 7 | değişmez | | | sonda: 4 `ok` |
| Son inceleme düzeltmeleri | 57/500 | 11 | 10 | `App.test.tsx` +1 (harita önizlemenin ardında giderse istek biter), `PrintDocumentView.test.tsx` +1 (graha sayfasında "≈" notu); sonda dolu portu reddeder ve graha notunu arar |
| Task 8 | değişmez | | | Rust sayıları tabandaki gibi |

**5. Yürütme notu.** Bu plandaki kod, `ecf7ed5` ağacının bir kopyasında (çalışan uygulamaya dokunmadan) görev sırasıyla yazılmış ve koşulmuştur: Vitest 57/497, `pnpm build`, e2e 11, görsel 10 ve sondanın dört koşusu yeşildir; sayaç zinciri o koşulardan gelir. Sonda plan yazılırken iki hatayı buldu ve ikisi de planın içindedir: Letter kâğıtta saati bilinmeyen haritanın beşinci sayfaya taşması (harita 82 mm'ye indi, e2e sınırı 744 × 990 px oldu) ve sistem fontunun PDF metin katmanını bozması (D6). Bir görevin sayısı tutmazsa önceki görevin adımı denetlenir; sayı elle düzeltilmez. Rust sayıları ve elle denetim kapısı bu kopyada koşulmamıştır (Rust değişmez; kapı gerçek uygulama ister).

**6. Ön inceleme (Global Constraints ile çelişki ve boş testler).**
- C2′: yeni komut yok; `api.printWindow()` bağımsız değişken almaz; PDF yolu macOS penceresindedir. Çelişki yok.
- C1: `options.ts` ve `sidebarPref.ts` hata loglamaz; `no-console.test.ts` yeni dosyaları da tarar (Task 1 notu). `document.title` log değildir. Sonda yalnızca fikstür adını yazar.
- C9/no-hex: `ui/print` altında hex yoktur; istisnalar Task 4'te kalkar ve `PrintDocumentView.test.tsx` `print.css`'i ayrıca tarar. Kâğıttaki yazı tipi kuralı renk değildir.
- C11: Task 3 Step 4 ve Task 6 Step 3 mevcut temellerin değişmediğini `git status` ile kanıtlar.
- C7: `⌘P` `matchShortcut`'tan geçer; yeni dinleyici yoktur.
- Erişilebilirlik kuralı (devre dışı denetim): son sayfa kutusu `aria-disabled` ve görünür nedenlidir; testi nedeni `toHaveAccessibleDescription` ile okur.
- Boş test taraması: her test en az bir davranış iddiası taşır. `PrintDocumentView.test.tsx`'in CSS testi biçim kurallarını (`@page`, dolgu, sayfa kesmesi, yazı tipi) düzenli ifadeyle denetler; bunların gerçek etkisini Task 6 (hesaplanmış stil ve kutu ölçüsü) ve Task 7 (PDF) ayrıca kanıtlar. e2e sığma testi sayfa yüksekliğinin 200 px'ten büyük olduğunu da ister: gizli (0 yükseklikli) bir sayfa sınırı sahte biçimde geçemez. Task 7 Step 2 sondanın hatayı gerçekten gördüğünü gösterir.
