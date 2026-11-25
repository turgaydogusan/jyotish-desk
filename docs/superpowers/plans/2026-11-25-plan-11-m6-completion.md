# Plan 11 — M6'nın tamamlanması: ayarlar, fark rozeti, tzdb kabul akışı, transit katmanı, dasa şeridi, doğum bilgisi paneli ve M6 borçları

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** M6'yı kapatmak: hesap ayarları ve genel varsayılanlar arayüzden düzenlenir (`CalcSettings`, `AppPrefs`, `AppPrefs.schema_version` denetimi); haritanın ayarı varsayılandan farklıysa üst çubukta fark rozeti çıkar; tzdb farkı bir bantta "yeni zamanı kabul et / saklanan zamanı koru" seçimiyle çözülür; transit tablosu, haritada transit halkası ve tarih seçimi çalışır (tarih oturumluk, her açılışta "şimdi"); dasa şeridi ve doğum bilgisi paneli dolar; doğum formuna cinsiyet alanı ve erişilebilirlik düzeltmeleri gelir; Plan 10'un M6'ya bağlı ertelenen maddeleri kapanır.

**Architecture:**
- **Store (Rust):** `Store::app_prefs_load()` tercihleri sürüm denetimi ve uyarılarla okur, daha yeni sürümlü kaydın üzerine yazmaz (Task 1). `save_view_state` ve `save_panel_layout` yazarken de 64/48 sınırını ve `schema_version`'ı denetler (Task 1). tzdb sessiz yolu çözülmüş anın kısaltma, ofset ve uyarılarını yeniler; `keep_stored_moment` saklanan anı korur (Task 2). İçe/dışa aktarma sertleşir (Task 14). Şema değişmez (D2).
- **Çekirdek ve komutlar:** `jyotish_core::dasha_path_at` bir andaki dasa yolunu verir; yeni `transit_commands.rs` "şimdi"yi Rust'ta okur, yerel tarih-saati tarayıcının IANA diliminde çözer ve `TransitView` döner (Task 3). `get_app_prefs` `PrefsLoad` döner, `keep_stored_moment` komutu, atlas açılamama nedeni (Task 4).
- **Arayüz durumu:** `settings/prefs.tsx` (`PrefsProvider`), `transit/store.tsx` (`TransitProvider`, oturumluk an, isteğe bağlı okuma). Kayıtlar sıralıdır (C5) ve sayfa gizlenince boşaltılır (C17).
- **Arayüz:** `settings/SettingsDialog.tsx` (`⌘,`), üst çubukta fark rozeti, bildirim bandında tzdb eylemleri (Task 5–6); `transit/TransitTablePanel.tsx`, harita panelinde transit halkası (Task 7–8); `dasha/DashaStripPanel.tsx` (Task 9); `panels/BirthInfoPanel.tsx` (Task 10); doğum formu (Task 11); erişilebilirlik geçişi M6-7 (Task 12); notlar ve kenar çubuğu küçükleri (Task 13).
- **Testler:** her görev Vitest ya da `cargo test` ile; Task 15 Playwright e2e ve M6-9 test borcu; Task 16 son doğrulama.

**Tech Stack:** React 19, Vite 8, TypeScript 7 (`strict`), Vitest 5 (jsdom), i18next 27, Playwright 1.65.1 (WebKit), Tauri 2.12.7, tauri-specta rc.29 (Result hata kipi), specta rc.28, `tauri-plugin-dialog` `=2.8.4`, rusqlite (FTS5), jiff (gömülü tzdb, `atlas` yeniden dışa aktarır). **Yeni bağımlılık yok** (Rust ve npm).

**Spec:** `docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md` §4 ("Ayar katmanları", "Kalıcılık", tzdb farkı), §5 (`AppPrefs`, `CalcSettings`, dışa aktarma), §7 (`{code, params}`, uyarılar), §8 (gidiş-dönüş), §11 (M6); `docs/superpowers/specs/2026-11-24-ui-design.md` §3.2 (Transit düzeni), §3.3 (panel kataloğu: `dasha_strip`, `transit_table`, `birth_info`, harita paneli transit halkası), §3.4, §5 (rozetler), §6.

**Ön koşul:** Taban commit `6d48791` (Plan 10 ve inceleme düzeltmeleri `main`'de, CI yeşil). Plan 10 Task 12 adım 5'in tek kapısı (M5 + Plan 10 elle denetimi) `progress.md`'de kayıtlı olmalıdır; değilse Task 1 başlamaz.

## Kararlar

Muhafazakâr seçenek seçildi; her karar bir satır gerekçeyle.

- **D1 — Plan 11 M6'yı tek planda tamamlar; JHD (Plan 10 kapsam dışı madde 2) ve M6-5 dışarıda.** Gerekçe: JHD gerçek dosyalar olmadan kapatılamaz; M6-5 nakshatra kısaltmalarının haritaya taşınmasıyla aynı etiket yerleşimi işidir ve görsel temelleri bir kez değiştirmelidir.
- **D2 — Şema değişmez; göç yok (C3).** Gerekçe: `keep_stored_moment` ve tzdb sessiz yolu var olan `charts` sütunlarını yazar; yedekli göç 2 bu planın işine değmez.
- **D3 — `AppPrefs` okuması toleranslı kalır ama uyarı verir; daha yeni sürümlü kayıt bellekte varsayılanla değiştirilir ve üzerine yazılmaz (`prefs_too_new`).** Gerekçe: daha yeni bir uygulamanın yazdığı tercihi eski uygulama sessizce ezmemeli; açılışı da engellememeli.
- **D4 — Yazım sınırları kırpmaz, reddeder (`invalid_input`, `field: "view_state"` / `"panel_layout"`).** Gerekçe: arayüz sınırın ötesine yazmaz (C6); reddetmek bir hatayı görünür kılar, kırpmak saklanan biçimi sessizce değiştirir.
- **D5 — tzdb farkı: "Yeni zamanı kabul et" aynı doğum verisiyle `update_chart`, "Saklanan zamanı koru" `keep_stored_moment` (yalnızca `tzdb_version` damgası).** Gerekçe: ikisi de var olan yazım yollarıdır; karar kullanıcınındır ve hiçbir yol anı sessizce kaydırmaz (spec §4).
- **D6 — tzdb sessiz yolu (UTC aynı) kısaltma, ofset ve uyarıları da yeniler.** Gerekçe: Plan 08 Task 8 bulgusu; aynı anı başka kısaltmayla göstermek yanlış bilgidir, UTC değişmediği için kullanıcı kararı gerekmez.
- **D7 — "Şimdi" Rust'ta okunur; seçilen yerel tarih-saat tarayıcının IANA diliminde Rust'ta çözülür (boşlukta hata ve öneri, örtüşmede önceki).** Gerekçe: C4 ("bugün" Rust'u bekler); kullanıcının duvar saati yerel saattir; DST boşluğu sessizce kaydırılmaz.
- **D8 — Transit tarihi oturumluktur, kaydedilmez; her açılışta ve her yeni haritada "şimdi"; transit halkası açık/kapalı ise panel ayarıdır (`transit_ring`) ve düzenle kaydedilir.** Gerekçe: görev kapsamı (tarih kaydedilmez, açık/kapalı kaydedilir); düzen zaten panel ayarını taşır.
- **D9 — Transit halkası yalnızca D1'de çizilir; transit etiketi parantezli kısaltmadır, `(Sa)`, italik ve etkileşimsiz; erişilebilir metin harita altında gizli listededir.** Gerekçe: transit konumu varga olarak tanımsızdır; parantez biçimsel işarettir (C9: renk tek başına anlam taşımaz, baskıda okunur); halka çizimini etiket yerleşimine eklemek yeni geometri istemez.
- **D10 — Güncel dasa dönemi Rust'tan gelir (`TransitView.dasha_path`); şeritte bir döneme tıklamak transit anını dönemin başlangıcına taşır. Dasa ağacından transit bağı bu planda yok.** Gerekçe: C4; ağaçta `Enter` dal açar, ona ikinci anlam yüklemek klavye desenini bozar (Bu planın kapsamadıkları madde 3).
- **D11 — Ayar düzenleyici 500 ms debounce ile sıralı otomatik kaydeder (son kazanır), sayfa gizlenince boşaltır (C17); `Custom` ayanamsa gösterilir ama düzenlenmez.** Gerekçe: düzen kaydıyla aynı ritim; `Custom`'ın `t0_jd`/`deg` girişi bir JD alanı ister ve içe aktarmayla gelen değeri korumak yeter.
- **D12 — Fark rozeti açık haritanın ayarı `AppPrefs.calc_settings`'ten farklıysa çıkar; rozet bir düğmedir ve ayarlar penceresini farklı alanlar işaretli açar. Pencerede "Varsayılana döndür" ve "Bunu varsayılan yap" vardır; onay istemez.** Gerekçe: ikisi de geri alınabilir (öbür düğme ya da alan düzenlemesi); onay penceresi akışı yavaşlatır.
- **D13 — Arayüz dili `localStorage`'da kalır (ilk boyamadan önce gerekir); "Bunu varsayılan yap" ve varsayılan düzenlemesi `AppPrefs.locale`'e o anki dili yazar.** Gerekçe: Plan 04'ün dil anahtarı açılıştan önce okunur; `AppPrefs.locale` yedek ve dışa aktarma içindir.
- **D14 — CSV etiket ayırıcısı `"; "` kalır; belge bunu ve JSON'un kayıpsız biçim olduğunu yazar. Etiket virgül içeremez: `normalize_tags` virgülde böler.** Gerekçe: CSV tek yönlü dışa aktarmadır, biçimini değiştirmek elektronik tablo kullanıcılarını bozar; virgüllü etiket etiket kutusunda ikiye bölünüyordu (Plan 10 madde 23).
- **D15 — Dışa aktarma boyutu: komut `ExportResult { file, bytes, over_import_limit }` döner; sınır aşılırsa `warning` bildirimi. Dışa aktarma engellenmez, sınır yükseltilmez.** Gerekçe: Plan 10 madde 17'nin iki seçeneğinden bellek ölçümü gerektirmeyenidir.
- **D16 — `file_io.detail` kararlı bir kod kümesine eşlenir (`not_found`, `permission_denied`, `already_exists`, `invalid_input`, `invalid_data`, `storage_full`, `other`) ve arayüzde `errors.fileIoDetail.*` ile çevrilir.** Gerekçe: `ErrorKind` metni İngilizcedir ve sürümler arasında değişebilir.
- **D17 — Boş "Yeni not" odak çıkışında ya da panel kapanınca silinir.** Gerekçe: Plan 10 madde 23; boş not hiçbir bilgi taşımaz ve listede yer kaplar.
- **D18 — Atlas açılamama nedeni `atlas_unavailable.reason` (`missing`, `schema`, `io`) olarak gelir; dosya yolu taşınmaz.** Gerekçe: Plan 09 kapsam dışı 13; C1/C2′ yol vermez.
- **D19 — Oturum geri yükleme mevcut akıştır (Plan 09/10); bu plan yalnızca spec §8 gidiş-dönüş testini tercihleri ve `save_view_state` komut gövdesini kapsayacak biçimde genişletir.** Gerekçe: `tests/open_client.rs` ayarları, düzeni, dasha yolunu, etiketleri ve notları zaten denetler; eksik olan tercihler ve komut gövdesidir.
- **D20 (ön inceleme) — `transit_commands` lib testleri tek bir `Engine`'i paylaşır (`OnceLock`).** Gerekçe: süreçte yalnızca bir SE örneği açılabilir (`EphError::AlreadyOpen`); paralel koşan üç testin her biri motor açarsa testler aralıklı düşer.
- **D21 (ön inceleme) — `ImportReport.warnings_total` `#[serde(default)]` taşımaz.** Gerekçe: `specta_builder` serde fazlarını kapatır ve IPC tiplerinde `default` özniteliği olmadığını varsayar; rapor yalnızca Rust'tan arayüze gider, eski veriden okunmaz.
- **D22 (ön inceleme; Task 12 incelemesinde değişti, `63395f6`) — Düzen menüsünde hiçbir öğe yerel `disabled` değildir; kullanıcı düzeninde "Varsayılana döndür" de `aria-disabled` olur, odaklanır ve nedeni (`layout.resetBuiltinOnly`) vardır. Devre dışı öğeler soluk çizilir; nedenler menünün altında görünür satırlardır (`.layout-menu-hint`) ve `aria-describedby` ile bağlanır.** Gerekçe: devre dışı denetimin nedeni her yerde bağlanır; gören kullanıcı da öğenin neden çalışmadığını okur; ok tuşları hiçbir öğede takılmaz. Aynı incelemede: danışan listesinin sekme durağı satır sırasına değil danışana bağlıdır, değiştirici tuşlu oklar yutulmaz; `Dialog` kapanırken açan öğe kaldırılmışsa odak ana bölgeye geçer. Vitest toplamı +5 (Task 12: 438).
- **D23 (ön inceleme) — Task 16 yasaklı sözcük taraması plan belgelerini (`docs/superpowers/plans`) dışarıda tutar.** Gerekçe: kuralın kendi metni bu dosyadadır ve dosya `6d48791`'den sonra commit edilmiştir; tarama kodu, mesajları ve diğer belgeleri kapsar.

## Global Constraints

- Lisans: AGPL-3.0-or-later. Bu plan bağımlılık eklemez; `cargo deny check` Task 16'da `advisories ok, bans ok, licenses ok, sources ok` verir.
- "Kala", "Kāla", "Kaala", "Cala" ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Türkçe sözcükler de kapsamdadır (ör. "kalan", "kalabilir", "yakalama", "arkalarında"): eklenen satırlarda `k(a|ā|aa)la` ve `\bcala\b` bulunmaz. Her commit adımı eklenen satırları, dosya adlarını ve commit mesajını `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile tarar; çıktı boş olmalıdır (C13). Bu belgede kuralın kendi metni dışında eşleşme yoktur.
- Kapılar: `pnpm --dir app test`, `pnpm --dir app build` (`tsc --noEmit`, `tsc --noEmit -p e2e`, `vite build`) her görev sonunda yeşil; `pnpm --dir app e2e` Task 15 ve 16'da yeşil. `tsc` `strict`. Tek dosya: `pnpm --dir app exec vitest run ui/<yol>`.
- Bu plandaki Rust kod blokları `rustfmt` çıktısı değildir: Rust değişen her görevde `--check`'ten önce `cargo fmt --all` koşulur (yalnızca biçim değişir).
- Rust değişen görevlerde (1, 2, 3, 4, 14, 15): `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo clippy -p store --features specta --all-targets -- -D warnings`, ilgili `cargo test`; MSRV 1.93: `PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store -p jyotish-desk --all-targets`.
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez; `cargo deny` `~/.cargo/bin`'dedir). Node 24+, pnpm 11.8.0. SE ephemeris yolu 242 baytın altında olmalıdır (kopyada kısa yol, ör. `/Users/ttd/jd-p11`).
- CI işleri: `deny`, `core-macos`, `core` (ubuntu ve windows; yalnızca crate'ler), `macos-x86_64`, `msrv`. Hiçbiri bu planda değişmez.
- **Üretilen dosyalar elle düzenlenmez (C10):** IPC tipi değişince `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings` (Task 1, 3, 4, 14); komut çıktısı değişince `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture` (Task 3, 4).
- **Tabanlar (`6d48791`):** Vitest `Test Files  43 passed (43)`, `Tests  363 passed (363)`; `jyotish-desk` lib 24, `store` lib 71, `jyotish-core` lib 108, golden 8, `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2; e2e 6; görsel 6 (yerel, darwin).
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir:**
  - **C1** Gizlilik: danışan adları, not gövdeleri, doğum verisi ve `CommandError.params` loglanmaz. `no-console.test.ts` durur; izin listesi büyümez. Transit tarihi ve ayarlar da loglanmaz.
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu almaz; dosya pencereleri Rust'ta açılır. Yetenek dosyası `dialog:` ya da `fs:` izni vermez; `tauri_plugin_fs` başlatılmaz. `import_jhd` kayıtsız kalır.
  - **C3** Göç V1 donmuştur; şema değişikliği yedekle başlayan göç 2'ye gider. Bu plan şemayı değiştirmez (D2).
  - **C4** Arayüz astrolojik hesap yapmaz. `formatJdDate` tek JD→tarih kodudur; "şimdi", güncel dasa dönemi ve transit konumları Rust'tan gelir. UTC dizgisini `Intl.DateTimeFormat` ile göstermek hesap değildir.
  - **C5** Görünüm, ayar ve not yazımları sıralıdır (tek uçuşta kayıt, son kazanır) ve haritayı ya da danışanı değiştiren her işlemden önce boşaltılır.
  - **C6** Tek düzen anahtarı ve iki ayrıştırıcıda tek sınır çifti (64 düzen, 48 panel); ayrıştırıcı değişikliği `layout-parity.json`'ı ve iki testi aynı commit'te günceller. Task 1 yazım denetimini ekler ama ayrıştırıcıları değiştirmez.
  - **C7** Kısayollar yalnızca `shortcuts.ts`'e (`matchShortcut` + `SHORTCUT_LIST`) eklenir; Shell'in tek dinleyicisi durur. Task 5 `⌘,` ekler.
  - **C8** Her pencere `shell/Dialog`'u kullanır (ayarlar penceresi dahil).
  - **C9** Renk yalnızca token'lardan gelir ve tek başına anlam taşımaz; `--warn` metin rengi değildir. Bu planda: fark rozeti işaret + metin; transit anındaki dasa dönemi `aria-current` + "transit anında" metni + kalın kenar; transit etiketi parantez + italik.
  - **C10** Üretilen dosyalar yukarıdaki komutlarla.
  - **C11** Görsel PNG temelleri yereldir ve yalnızca darwin'dir; harita çizimi değişirse temeller ve SVG anlık görüntüleri aynı commit'te yeniden üretilir. Transit halkası varsayılan kapalıdır: mevcut temeller değişmez (Task 8 bunu doğrular).
  - **C12** Arayüzden ulaşılabilen her yeni `CommandError.code` aynı görevde tr/en/fr metni alır ve `i18n.test.ts` `BACKEND_CODES`'a eklenir (Task 1: `prefs_too_new`).
  - **C13** Yasaklı sözcük taraması her commit'te; depo dışındaki ilerleme notları hiçbir zaman `git add` edilmez (her commit yalnızca `app/`, `crates/` ve `docs/` altını değiştirir).
  - **C14** Plan 07'nin gün doğumu toleransı değişmez.
  - **C15** `alert`, `confirm`, `prompt` çağrılmaz; onaylar `shell/ConfirmDialog` ile.
  - **C16** ⌘ kısayolları metin girişinden de çalışır; yeni kısayol metin düzenleme akorlarını (⌘A/C/V/X/Z, ⌘ok) almaz. `⌘,` düzenleme akoru değildir.
  - **C17** Her otomatik kaydedilen şey (görünüm, notlar, ayarlar) `visibilitychange` hidden ve `pagehide` olayında boşaltılır ve bileşen kaldırıldıktan sonra başarısız olan kaydı bildirir.
- **Erişilebilirlik:** WCAG AA; her işlem klavyeyle yapılır; odak halkası 2 px `--ink`; modal pencerede odak tuzağı ve geri dönüş (`shell/Dialog`); devre dışı denetimin nedeni `aria-describedby` ile bağlıdır; durum değişiklikleri `role=status` ile duyurulur.
- Commit mesajları İngilizce, `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` satırıyla biter. Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit).
- Kod blokları `6d48791` ağacına göre yazılmıştır. Değişen dosyalarda "şu bloğu şununla değiştir" biçimi kullanılır; bağlam tutmazsa önceki görevin adımını denetle, elle uydurma yapma.

## Dosya Yapısı (bu plan)

```
crates/store/src/
  settings.rs        PrefsLoad, PrefsWarning, app_prefs_load, sürüm korumalı set_app_prefs
  error.rs           StoreError::PrefsTooNew
  view_state.rs      check_view_state, check_panel_layout, MAX_VIEW_STATE_BYTES
  charts.rs          save_view_state yazım denetimi; keep_stored_moment
  layout.rs          save_panel_layout yazım denetimi
  open.rs            tzdb sessiz yolu çözülmüş alanları yeniler
  notes.rs           normalize_tags virgülde böler
  exchange.rs        BOM, uyarı sınırı (MAX_IMPORT_WARNINGS, warnings_total)
  lib.rs             yeni dışa aktarımlar
crates/jyotish-core/src/
  dasha.rs           dasha_path_at
  lib.rs             dışa aktarım
app/src-tauri/src/
  transit_commands.rs   (yeni) TransitAt, TransitView, transits_at
  client_commands.rs    PrefsTooNew eşlemesi, keep_stored_moment, save_view_state_body
  store_commands.rs     get_app_prefs → PrefsLoad; ExportResult; UTF-8 denetimi; fsync; set_parent; io_detail
  atlas_commands.rs     AtlasUnavailable nedeni
  lib.rs                komut kaydı
app/src-tauri/tests/{ui_fixture.rs, open_client.rs}
app/ui/
  api.ts, boot.ts, App.tsx, test/render.tsx, test/fixture.ts
  settings/{model.ts, prefs.tsx, saves.ts, SettingsDialog.tsx, settings.css}   (yeni)
  transit/{store.tsx, TransitDate.tsx, TransitTablePanel.tsx, transit.css}      (yeni)
  dasha/{DashaStripPanel.tsx, DashaTreePanel.tsx, dasha.css}
  birth/{BirthForm.tsx, PlaceSearch.tsx, model.ts}
  chart/{types.ts, ChartSvg.tsx, chart.css}, panels/{ChartPanel.tsx, BirthInfoPanel.tsx, registry.ts}
  shell/{ChartTitle.tsx, TopBar.tsx, Shell.tsx, notices.tsx, TzdbActions.tsx, LayoutPicker.tsx, shell.css}
  session/store.tsx, shortcuts.ts, palette/{items.ts, Palette.tsx}
  clients/{ClientList.tsx, OpenClient.tsx, TagEditor.tsx, DataActions.tsx, clients.css}
  notes/{NotesPanel.tsx, NoteEditor.tsx}, view/persistence.ts
  theme/tokens.css, dashboard/dashboard.css
  i18n/{tr,en,fr}.ts, i18n/errors.ts, i18n/i18n.test.ts
app/e2e/{ipc.ts, desk.spec.ts}
docs/export-format.md                                                         (yeni)
docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md                   (düzeltmeler)
```

### Kapsam

| Madde | Yer |
|---|---|
| Ayarlar: `CalcSettings`, `AppPrefs`, `AppPrefs.schema_version`, tercih `calc_settings`'in sessiz geri dönüşü | Task 1, 4, 5, 6 |
| Fark rozeti ve tzdb farkını kabul akışı (D6 sessiz yolu dahil) | Task 2, 4, 6 |
| Oturum geri yükleme | Task 4 (gidiş-dönüş testi), D19 |
| Transit bindirme: halka, `transit_table`, tarih seçimi, açık/kapalı kaydı, "şimdi" | Task 3, 7, 8 |
| Plan 10 kapsam dışı 1: `birth_info` ve rozetler, `dasha_strip`, cinsiyet alanı | Task 9, 10, 11 |
| Plan 10 kapsam dışı 6 (`save_view_state` yazım sınırı) | Task 1 |
| Plan 10 kapsam dışı 7 (M6-1) | Task 11 |
| Plan 10 kapsam dışı 9 (M6-6) | Task 9 |
| Plan 10 kapsam dışı 10, 18, 19 (M6-7) | Task 12 |
| Plan 10 kapsam dışı 11, 21, 25 (M6-9) | Task 4, 15 |
| Plan 10 kapsam dışı 12 (Plan 09 kapsam dışı 13–15) | Task 4 (atlas nedeni), 11 |
| Plan 10 kapsam dışı 13 (Plan 11 bölümleri), 17, 22, 24 | Task 14 |
| Plan 10 kapsam dışı 20, 23, 26 | Task 13 (arayüz), Task 14 (Rust bölümleri) |
| Palet test boşlukları (Esc odak dönüşü, eski yanıtın düşürülmesi) | Task 12 |
| Plan 10 kapsam dışı 2, 3, 4, 5, 8, 14, 15, 16 | Bu planın kapsamadıkları |

---

### Task 1: Store: sürüm denetimli tercihler ve görünüm kayıtlarında yazım sınırları

Spec §5 `AppPrefs.schema_version` ve Plan 08 Task 3 bulgusu ("tercih `calc_settings`'i sessizce sıfırlanır"): okuma toleranslı kalır ama sonucu uyarılarla döner (`PrefsLoad`). Daha yeni sürümlü kayıt bellekte varsayılanla değiştirilir ve `set_app_prefs` onun üzerine yazmaz (D3). Plan 10 kapsam dışı 6: `save_view_state` ve `save_panel_layout` yazarken de 64/48 sınırını, `schema_version`'ı, dasha yolu uzunluğunu ve 1 MiB JSON boyutunu denetler; aşan kayıt reddedilir (D4). Ayrıştırıcılar değişmez (C6).

**Files:**
- Modify: `crates/store/src/settings.rs`
- Modify: `crates/store/src/error.rs`
- Modify: `crates/store/src/view_state.rs`
- Modify: `crates/store/src/charts.rs` (`save_view_state`, testler)
- Modify: `crates/store/src/layout.rs`
- Modify: `crates/store/src/lib.rs`
- Modify: `app/src-tauri/src/client_commands.rs` (`StoreError::PrefsTooNew` eşlemesi ve testi)
- Modify: `app/ui/i18n/{tr,en,fr}.ts`, `app/ui/i18n/i18n.test.ts` (C12: `prefs_too_new`)

**Interfaces:**
- Consumes: `Store::app_state_get/app_state_set`, `load_calc_value`, `SettingsLoad`, `MAX_LAYOUTS`, `MAX_PANELS`, `VIEW_STATE_SCHEMA_VERSION`, `jyotish_core::MAX_DASHA_LEVEL`.
- Produces:
  - `pub enum PrefsWarning { Unreadable, CalcSettingsReset { found_version: Option<u32> }, TooNew { found: u32 } }` (serde + specta)
  - `pub struct PrefsLoad { pub prefs: AppPrefs, pub warnings: Vec<PrefsWarning> }` (Serialize + specta)
  - `Store::app_prefs_load(&self) -> Result<PrefsLoad, StoreError>`; `Store::app_prefs()` artık `app_prefs_load()?.prefs`
  - `StoreError::PrefsTooNew { found: u32 }` → `CommandError { code: "prefs_too_new", params: { found, supported } }`
  - `pub const MAX_VIEW_STATE_BYTES: usize = 1 << 20;` `pub fn check_view_state(&ViewState) -> Result<(), StoreError>`, `pub fn check_panel_layout(&PanelLayout) -> Result<(), StoreError>` (hata: `InvalidInput("view_state")` / `InvalidInput("panel_layout")`)

- [ ] **Step 1: Başarısız testleri yaz**

`crates/store/src/settings.rs` `mod tests` sonuna:

```rust
    #[test]
    fn app_prefs_load_reports_reset_unreadable_and_too_new() {
        let s = store();
        assert_eq!(
            s.app_prefs_load().unwrap(),
            PrefsLoad {
                prefs: AppPrefs::default(),
                warnings: vec![]
            }
        );
        // Okunamayan genel varsayılan: yerleşik varsayılan, dil korunur, uyarı.
        s.app_state_set(
            APP_PREFS_KEY,
            r#"{"schema_version":1,"locale":"en","calc_settings":{"schema_version":9}}"#,
        )
        .unwrap();
        let load = s.app_prefs_load().unwrap();
        assert_eq!(load.prefs.locale, "en");
        assert_eq!(load.prefs.calc_settings, CalcSettings::default());
        assert_eq!(
            load.warnings,
            [PrefsWarning::CalcSettingsReset {
                found_version: Some(9)
            }]
        );
        // Bozuk JSON.
        s.app_state_set(APP_PREFS_KEY, "[1,2]").unwrap();
        assert_eq!(
            s.app_prefs_load().unwrap().warnings,
            [PrefsWarning::Unreadable]
        );
        // Daha yeni uygulamanın kaydı: hiçbir alanı okunmaz.
        s.app_state_set(
            APP_PREFS_KEY,
            r#"{"schema_version":2,"locale":"fr","calc_settings":{}}"#,
        )
        .unwrap();
        let load = s.app_prefs_load().unwrap();
        assert_eq!(load.prefs, AppPrefs::default());
        assert_eq!(load.warnings, [PrefsWarning::TooNew { found: 2 }]);
        assert_eq!(s.app_prefs().unwrap(), AppPrefs::default());
    }

    #[test]
    fn set_app_prefs_refuses_to_overwrite_newer_prefs() {
        let s = store();
        let newer = r#"{"schema_version":2,"locale":"fr","calc_settings":{}}"#;
        s.app_state_set(APP_PREFS_KEY, newer).unwrap();
        assert!(matches!(
            s.set_app_prefs(&AppPrefs::default()),
            Err(StoreError::PrefsTooNew { found: 2 })
        ));
        assert_eq!(s.app_state_get(APP_PREFS_KEY).unwrap().as_deref(), Some(newer));
        // Okunamayan eski kayıt ise yenisiyle değiştirilebilir.
        s.app_state_set(APP_PREFS_KEY, "[1,2]").unwrap();
        s.set_app_prefs(&AppPrefs::default()).unwrap();
        assert!(s.app_prefs_load().unwrap().warnings.is_empty());
    }
```

`crates/store/src/charts.rs` `mod tests` sonuna:

```rust
    fn layouts(n: usize, panels: usize) -> PanelLayout {
        PanelLayout {
            active_layout_id: "user:0".into(),
            layouts: (0..n)
                .map(|i| Layout {
                    id: format!("user:{i}"),
                    name: format!("L{i}"),
                    builtin: false,
                    panels: (0..panels)
                        .map(|j| Panel {
                            id: format!("p{j}"),
                            kind: "notes".into(),
                            x: 0,
                            y: j as u32 * 4,
                            w: 3,
                            h: 4,
                            settings: serde_json::Map::new(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    #[test]
    fn save_view_state_rejects_what_the_parsers_would_trim() {
        let mut s = store();
        let (_, ch) = s.create_client(&new_client("A")).unwrap();
        let ok = ViewState {
            panel_layout: Some(layouts(MAX_LAYOUTS, MAX_PANELS)),
            ..ViewState::default()
        };
        s.save_view_state(ch.id, &ok).unwrap();
        for bad in [
            ViewState {
                panel_layout: Some(layouts(MAX_LAYOUTS + 1, 1)),
                ..ViewState::default()
            },
            ViewState {
                panel_layout: Some(layouts(1, MAX_PANELS + 1)),
                ..ViewState::default()
            },
            ViewState {
                schema_version: 2,
                ..ViewState::default()
            },
            ViewState {
                dasha_expanded_path: vec![jyotish_core::Graha::Sun; 6],
                ..ViewState::default()
            },
        ] {
            assert!(
                matches!(
                    s.save_view_state(ch.id, &bad),
                    Err(StoreError::InvalidInput("view_state" | "panel_layout"))
                ),
                "{bad:?}"
            );
        }
        // Reddedilen kayıt öncekini değiştirmez.
        assert_eq!(s.view_state(ch.id).unwrap(), ok);
    }

    #[test]
    fn save_panel_layout_rejects_over_cap_and_oversized_layouts() {
        let mut s = store();
        assert!(matches!(
            s.save_panel_layout(Some(&layouts(MAX_LAYOUTS + 1, 1))),
            Err(StoreError::InvalidInput("panel_layout"))
        ));
        let mut huge = layouts(1, 1);
        huge.layouts[0].panels[0]
            .settings
            .insert("blob".into(), serde_json::Value::String("x".repeat(MAX_VIEW_STATE_BYTES)));
        assert!(matches!(
            s.save_panel_layout(Some(&huge)),
            Err(StoreError::InvalidInput("panel_layout"))
        ));
        assert_eq!(s.panel_layout().unwrap(), None);
    }
```

`charts.rs` test modülündeki `use crate::view_state::{Layout, Panel};` satırını şununla **değiştir** (ikinci bir `use` ekleme: aynı ad iki kez içe aktarılırsa derlenmez; `PanelLayout` ve `ViewState` `use super::*` ile gelir): `use crate::view_state::{Layout, Panel, MAX_LAYOUTS, MAX_PANELS, MAX_VIEW_STATE_BYTES};`. `Panel.y` `u32`'dir.

`app/src-tauri/src/client_commands.rs` `mod tests` içindeki eşleme testinin (`assert_eq!(CommandError::from(StoreError::LastChart).code, "last_chart");` satırının) hemen altına:

```rust
        let e = CommandError::from(StoreError::PrefsTooNew { found: 2 });
        assert_eq!(e.code, "prefs_too_new");
        assert_eq!((e.params["found"].as_str(), e.params["supported"].as_str()), ("2", "1"));
```

`app/ui/i18n/i18n.test.ts` `BACKEND_CODES`'un son satırını genişlet:

```ts
  "file_io", "dialog_path", "import_too_large", "import_format", "import_version", "import_record",
  "prefs_too_new",
] as const;
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
cargo test -p store app_prefs_load set_app_prefs_refuses save_view_state_rejects save_panel_layout_rejects 2>&1 | tail -5
pnpm --dir app exec vitest run ui/i18n
```
Expected: `store` derlenmez (`cannot find type PrefsLoad`, `no variant PrefsTooNew`, `cannot find value MAX_VIEW_STATE_BYTES`); vitest `every backend error code reachable from the UI has a message` düşer (`prefs_too_new`).

- [ ] **Step 3: Uygula**

`crates/store/src/error.rs`, `ImportVersion` varyantının altına:

```rust
    /// Tercihler bu sürümden yeni bir uygulamayla yazılmış; üzerine yazılmaz (Plan 11 D3).
    #[error("app preferences version {found} is newer than supported")]
    PrefsTooNew { found: u32 },
```

`crates/store/src/settings.rs`: `const APP_PREFS_KEY` satırının altına ekle:

```rust
/// Tercih okumasının ölümcül olmayan durumları; arayüz açılış bandında gösterir (spec §7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum PrefsWarning {
    /// Kayıt JSON nesnesi değil; yerleşik varsayılanlar kullanıldı.
    Unreadable,
    /// Genel hesap varsayılanı okunamadı ya da göç edilemedi; yerleşik varsayılan kullanıldı.
    /// Saklanan JSON değiştirilmedi.
    CalcSettingsReset { found_version: Option<u32> },
    /// Tercihler bu sürümden yeni bir uygulamayla yazıldı; yerleşik varsayılanlar kullanıldı ve
    /// kayıt korunur (`set_app_prefs` üzerine yazmaz).
    TooNew { found: u32 },
}

/// `app_prefs_load` sonucu: kullanılacak tercihler ve okuma uyarıları.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PrefsLoad {
    pub prefs: AppPrefs,
    pub warnings: Vec<PrefsWarning>,
}

/// Saklanan kaydın sürümü; alan yoksa ya da sayı değilse güncel sürüm sayılır (Plan 03 kayıtları
/// her zaman damgalıdır; damgasız kayıt elle yazılmıştır).
fn stored_prefs_version(obj: &serde_json::Map<String, Value>) -> u32 {
    obj.get("schema_version")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(APP_PREFS_SCHEMA_VERSION)
}
```

`impl Store` içindeki `app_prefs` ve `set_app_prefs` fonksiyonlarını şununla değiştir:

```rust
    /// Toleranslı okuma, uyarılarla: kayıt yoksa varsayılanlar; tanınmayan dil "tr"; okunamayan
    /// hesap varsayılanı yerleşik varsayılanla değişir ve `CalcSettingsReset` uyarısı verir;
    /// daha yeni sürümlü kayıtta hiçbir alan okunmaz (`TooNew`).
    pub fn app_prefs_load(&self) -> Result<PrefsLoad, StoreError> {
        let mut out = PrefsLoad {
            prefs: AppPrefs::default(),
            warnings: Vec::new(),
        };
        let Some(raw) = self.app_state_get(APP_PREFS_KEY)? else {
            return Ok(out);
        };
        let Ok(Value::Object(mut obj)) = serde_json::from_str::<Value>(&raw) else {
            out.warnings.push(PrefsWarning::Unreadable);
            return Ok(out);
        };
        let found = stored_prefs_version(&obj);
        if found > APP_PREFS_SCHEMA_VERSION {
            out.warnings.push(PrefsWarning::TooNew { found });
            return Ok(out);
        }
        if let Some(Value::String(l)) = obj.get("locale") {
            if LOCALES.contains(&l.as_str()) {
                out.prefs.locale = l.clone();
            }
        }
        match obj
            .remove("calc_settings")
            .map(|v| load_calc_value(v, &CalcSettings::default()))
        {
            Some(SettingsLoad::Reset { found_version, .. }) => out
                .warnings
                .push(PrefsWarning::CalcSettingsReset { found_version }),
            Some(load) => out.prefs.calc_settings = load.settings().clone(),
            None => {}
        }
        Ok(out)
    }

    /// `app_prefs_load` tercihleri, uyarısız (yeni harita varsayılanı için).
    pub fn app_prefs(&self) -> Result<AppPrefs, StoreError> {
        Ok(self.app_prefs_load()?.prefs)
    }

    /// Doğrular ve güncel sürümle yazar. Daha yeni sürümlü kaydın üzerine yazmaz.
    pub fn set_app_prefs(&self, prefs: &AppPrefs) -> Result<(), StoreError> {
        if !LOCALES.contains(&prefs.locale.as_str()) {
            return Err(StoreError::InvalidInput("locale"));
        }
        prefs.calc_settings.validate()?;
        if let Some(raw) = self.app_state_get(APP_PREFS_KEY)? {
            if let Ok(Value::Object(obj)) = serde_json::from_str::<Value>(&raw) {
                let found = stored_prefs_version(&obj);
                if found > APP_PREFS_SCHEMA_VERSION {
                    return Err(StoreError::PrefsTooNew { found });
                }
            }
        }
        let stored = AppPrefs {
            schema_version: APP_PREFS_SCHEMA_VERSION,
            ..prefs.clone()
        };
        self.app_state_set(APP_PREFS_KEY, &serde_json::to_string(&stored)?)
    }
```

`crates/store/src/view_state.rs`: `pub const MAX_PANELS` satırının altına:

```rust
/// Yazılan görünüm kaydının en büyük JSON boyutu (bayt). Arayüzün ürettiği en büyük düzen
/// (64 × 48 panel) bunun onda biri bile değildir; sınır bozuk ya da kötü niyetli yazımı keser.
pub const MAX_VIEW_STATE_BYTES: usize = 1 << 20;

/// Yazım denetimi (Plan 11 D4): ayrıştırıcıların kırpacağı kayıt reddedilir, saklanmaz.
pub fn check_panel_layout(pl: &PanelLayout) -> Result<(), crate::StoreError> {
    let fits = pl.layouts.len() <= MAX_LAYOUTS && pl.layouts.iter().all(|l| l.panels.len() <= MAX_PANELS);
    if !fits || serde_json::to_vec(pl)?.len() > MAX_VIEW_STATE_BYTES {
        return Err(crate::StoreError::InvalidInput("panel_layout"));
    }
    Ok(())
}

/// `save_view_state` denetimi: sürüm, dasha yolu uzunluğu ve düzen sınırları.
pub fn check_view_state(vs: &ViewState) -> Result<(), crate::StoreError> {
    if vs.schema_version != VIEW_STATE_SCHEMA_VERSION
        || vs.dasha_expanded_path.len() > usize::from(MAX_DASHA_LEVEL)
    {
        return Err(crate::StoreError::InvalidInput("view_state"));
    }
    match &vs.panel_layout {
        Some(pl) => check_panel_layout(pl),
        None => Ok(()),
    }
}
```

`crates/store/src/charts.rs` `save_view_state` gövdesinin ilk satırı olarak (işlem açılmadan önce):

```rust
        crate::view_state::check_view_state(vs)?;
```

`crates/store/src/layout.rs` `save_panel_layout` gövdesinin ilk satırı olarak:

```rust
        if let Some(pl) = layout {
            crate::view_state::check_panel_layout(pl)?;
        }
```

`crates/store/src/lib.rs` dışa aktarımları:

```rust
pub use settings::{
    load_calc_settings, AppPrefs, PrefsLoad, PrefsWarning, SettingsLoad, ValueMigration,
    APP_PREFS_SCHEMA_VERSION, CALC_SETTINGS_MIGRATIONS, LOCALES,
};
pub use view_state::{
    chart_view_json, chart_view_path, check_panel_layout, check_view_state, parse_dasha_path,
    parse_panel_layout, parse_view_state, Layout, Panel, PanelLayout, ViewState, MAX_LAYOUTS,
    MAX_PANELS, MAX_VIEW_STATE_BYTES, VIEW_STATE_SCHEMA_VERSION,
};
```

`app/src-tauri/src/client_commands.rs` `impl From<StoreError> for CommandError` içinde `StoreError::ImportVersion` kolunun altına:

```rust
            StoreError::PrefsTooNew { found } => Self::new(
                "prefs_too_new",
                &[
                    ("found", found.to_string()),
                    ("supported", store::APP_PREFS_SCHEMA_VERSION.to_string()),
                ],
            ),
```

`app/ui/i18n/tr.ts` `errors` içinde `store_too_new` satırının altına:

```ts
    prefs_too_new: "Tercihler bu sürümden yeni bir uygulamayla kaydedilmiş (sürüm {{found}}, desteklenen {{supported}}). Değişiklik kaydedilmedi; uygulamayı güncelleyin.",
```

`app/ui/i18n/en.ts`:

```ts
    prefs_too_new: "Preferences were saved by a newer version of the app (version {{found}}, supported {{supported}}). The change was not saved; update the app.",
```

`app/ui/i18n/fr.ts`:

```ts
    prefs_too_new: "Les préférences ont été enregistrées par une version plus récente de l’application (version {{found}}, prise en charge {{supported}}). La modification n’a pas été enregistrée ; mettez l’application à jour.",
```

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
cargo test -p store 2>&1 | grep 'test result'
cargo test -p jyotish-desk --lib 2>&1 | grep 'test result'
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo clippy -p store --features specta --all-targets -- -D warnings
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store -p jyotish-desk --all-targets
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
pnpm --dir app test && pnpm --dir app build
```
Expected: `store` lib `75 passed` (+4); `jyotish-desk` lib `24 passed` (eşleme testi genişledi); fmt/clippy çıktısız; MSRV `Finished`; `bindings.ts`'e `PrefsLoad` ve `PrefsWarning` eklenmez (henüz hiçbir komut dönmüyor; `git diff app/ui/bindings.ts` boş); Vitest `43 passed`, `363 passed`; build `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add crates/store/src app/src-tauri/src/client_commands.rs app/ui/i18n
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(store): versioned app prefs load and write-side view state caps

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: `grep` çıktısız; commit oluşur.

---

### Task 2: Store: tzdb sessiz yolu çözülmüş alanları yeniler; saklanan anı koruma

Plan 08 Task 8 "D13 sessiz yolu": tzdb değişip UTC aynı kaldığında yalnızca `tzdb_version` güncelleniyordu; kısaltma, ofset ve uyarılar eski tzdb'nin değerleri olarak duruyordu. Artık hepsi yeni çözümden yazılır (D6). UTC değiştiğinde `TzdbChanged` uyarısı durur; kullanıcı "Saklanan zamanı koru" derse `keep_stored_moment` yalnızca `tzdb_version`'ı güncel sürümle damgalar: an değişmez ve sonraki açılışta uyarı çıkmaz (D5). "Yeni zamanı kabul et" var olan `update_chart`'tır.

**Files:**
- Modify: `crates/store/src/open.rs` (`check_tzdb`, test)
- Modify: `crates/store/src/charts.rs` (`keep_stored_moment`, test)

**Interfaces:**
- Consumes: `check_tzdb(conn, &mut ChartRecord, &mut Vec<StoreWarning>)`, `resolve_parsed`, `Store::chart(chart_id)`, `Store::now()`.
- Produces: `Store::keep_stored_moment(&mut self, chart_id: u32) -> Result<ChartRecord, StoreError>` (yok: `NotFound { what: "chart", id }`).

- [ ] **Step 1: Başarısız testleri yaz**

`crates/store/src/open.rs` `mod tests` sonuna:

```rust
    #[test]
    fn silent_tzdb_refresh_rewrites_abbrev_offset_and_warnings() {
        let mut s = store();
        let (a, ch) = s.create_client(&new_client("A")).unwrap();
        let fresh = ch.resolved.clone();
        s.conn
            .execute(
                "UPDATE charts SET tzdb_version = '1999a', resolved_abbrev = 'OLD',
                 resolved_offset_secs = 1, resolved_warnings_json = '[\"PreTzdbReliability\"]'
                 WHERE id = ?1",
                params![ch.id],
            )
            .unwrap();
        let o = s.open_client(a.id, None).unwrap();
        assert!(o.warnings.is_empty());
        assert_eq!(o.chart.resolved, fresh);
        assert_eq!(s.chart(ch.id).unwrap().resolved, fresh);
    }
```

`crates/store/src/charts.rs` `mod tests` sonuna:

```rust
    #[test]
    fn keep_stored_moment_stamps_the_current_tzdb_and_keeps_the_moment() {
        let mut s = store();
        let (a, ch) = s.create_client(&new_client("A")).unwrap();
        s.conn
            .execute(
                "UPDATE charts SET tzdb_version = '1999a', resolved_utc = '2000-01-01T11:00:00Z',
                 resolved_jd_ut = 2451544.9583333335 WHERE id = ?1",
                params![ch.id],
            )
            .unwrap();
        assert_eq!(s.open_client(a.id, None).unwrap().warnings.len(), 1);
        let kept = s.keep_stored_moment(ch.id).unwrap();
        assert_eq!(kept.resolved.tzdb_version, atlas::tzdb_version());
        assert_eq!(kept.resolved.utc, "2000-01-01T11:00:00Z");
        assert_eq!(kept.resolved.jd_ut, 2_451_544.958_333_333_5);
        let o = s.open_client(a.id, None).unwrap();
        assert!(o.warnings.is_empty());
        assert_eq!(o.chart.resolved.utc, "2000-01-01T11:00:00Z");
        assert!(matches!(
            s.keep_stored_moment(999),
            Err(StoreError::NotFound { what: "chart", id: 999 })
        ));
    }
```

`charts.rs` test modülünde `params` ve `atlas` kullanımı için gerekirse `use rusqlite::params;` ekle (`atlas` store'un bağımlılığıdır).

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
cargo test -p store silent_tzdb_refresh keep_stored_moment 2>&1 | tail -8
```
Expected: `keep_stored_moment` için `no method named keep_stored_moment`; derlenmediği için ikisi de çalışmaz. (Önce yalnızca open.rs testini eklersen `assertion failed: left == right` ile `abbrev: "OLD"` görünür.)

- [ ] **Step 3: Uygula**

`crates/store/src/open.rs` `check_tzdb` içindeki `Ok(r) if r.utc == chart.resolved.utc => { … }` kolunu şununla değiştir:

```rust
        Ok(r) if r.utc == chart.resolved.utc => {
            // An aynı; yeni tzdb'nin kısaltması, ofseti ve uyarıları yazılır (Plan 11 D6).
            conn.execute(
                "UPDATE charts SET tzdb_version = ?1, resolved_abbrev = ?2,
                 resolved_offset_secs = ?3, resolved_warnings_json = ?4 WHERE id = ?5",
                params![
                    current,
                    r.abbrev,
                    r.offset_secs,
                    serde_json::to_string(&r.warnings)?,
                    chart.id
                ],
            )?;
            chart.resolved = r;
        }
```

`crates/store/src/charts.rs` `impl Store` içinde `update_chart`'ın altına:

```rust
    /// tzdb farkında "Saklanan zamanı koru" (Plan 11 D5): saklanan an değişmez, kayıt güncel
    /// tzdb ile damgalanır; sonraki açılışta `TzdbChanged` çıkmaz. Kabul yolu `update_chart`'tır.
    pub fn keep_stored_moment(&mut self, chart_id: u32) -> Result<ChartRecord, StoreError> {
        let now = self.now();
        let n = self.conn.execute(
            "UPDATE charts SET tzdb_version = ?1, updated_at = ?2 WHERE id = ?3",
            params![atlas::tzdb_version(), now, chart_id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound {
                what: "chart",
                id: chart_id,
            });
        }
        self.chart(chart_id)
    }
```

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
cargo test -p store 2>&1 | grep 'test result'
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo clippy -p store --features specta --all-targets -- -D warnings
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store -p jyotish-desk --all-targets
```
Expected: `store` lib `77 passed` (+2; `tzdb_change_is_checked_and_reported` değişmeden geçer); fmt/clippy çıktısız; MSRV `Finished`.

- [ ] **Step 5: Commit**

```bash
git add crates/store/src
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "fix(store): refresh resolved fields on silent tzdb update; keep stored moment

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Çekirdek ve komut: bir andaki dasa yolu, `transits_at` (Rust'ta "şimdi", tarayıcı diliminde yerel an)

UI spec §3.3 transit tablosu ("tarih, varsayılan: şimdi") ve dasa şeridi (güncel dönem). C4 gereği "şimdi" ve güncel dönem Rust'tan gelir (D7, D10). `jyotish_core::dasha_path_at(seed, jd_ut, depth)` kökten başlayarak o anı içeren dönemlerin lordlarını verir. Yeni `transit_commands.rs`:
- `TransitAt::Now`: saat Rust'ta okunur (testte enjekte edilir).
- `TransitAt::Local { date, time, tz }`: arayüzün gönderdiği duvar saati, tarayıcının IANA dilimi (`Intl.DateTimeFormat().resolvedOptions().timeZone`); `store::resolve_birth` ile çözülür: DST boşluğu `nonexistent_local_time` (öneriyle), örtüşmede önceki an (D7).
- `TransitAt::Jd(f64)`: dasa şeridinden bir dönemin başlangıcı.

`compute_transits` komutu kayıtta durur (Plan 05'ten beri bindings'te; arayüz kullanmaz). Fikstür `transit` anahtarını kazanır.

**Files:**
- Modify: `crates/jyotish-core/src/dasha.rs` (`dasha_path_at`, testler)
- Modify: `crates/jyotish-core/src/lib.rs` (dışa aktarım)
- Create: `app/src-tauri/src/transit_commands.rs`
- Modify: `app/src-tauri/src/lib.rs` (`mod`, komut kaydı)
- Modify: `app/src-tauri/tests/ui_fixture.rs`
- Modify: `app/ui/test/fixture.ts` (`UiFixture.transit`)
- Generated: `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`

**Interfaces:**
- Consumes: `mahadashas(&DashaSeed)`, `dasha_children(&DashaSeed, &[Graha])`, `DashaSeed::validate`, `MAX_DASHA_LEVEL`, `compute_transits_blocking(engine, jd_ut, settings, bhavas)`, `store::resolve_birth(&BirthData)`, `atlas::Timestamp`, `client_commands::blocking`.
- Produces:
  - `pub fn dasha_path_at(seed: &DashaSeed, jd_ut: f64, depth: u8) -> Result<Vec<Graha>, CoreError>` (`depth` 1..=5; aralık dışı an: boş ya da kısmi yol; geçersiz girdi `InvalidDashaInput("depth"|"jd_ut")`)
  - `pub const TRANSIT_DASHA_DEPTH: u8 = 3;`
  - `#[derive(Deserialize, Type)] pub enum TransitAt { Now, Local { date: String, time: String, tz: String }, Jd(f64) }`
  - `#[derive(Serialize, Type)] pub struct TransitView { pub snapshot: TransitSnapshot, pub utc: String, pub dasha_path: Vec<Graha> }`
  - `pub fn transits_at_blocking(engine: &Engine, at: TransitAt, now: Timestamp, settings: CalcSettings, natal_bhavas: Bhavas, seed: DashaSeed) -> Result<TransitView, CommandError>`
  - komut `transits_at(at, settings, natal_bhavas, seed)` → TS `commands.transitsAt(at, settings, natalBhavas, seed)`
  - fikstür: `transit: TransitView` (sabit saat `1_795_600_800`, `TransitAt::Now`)

- [ ] **Step 1: Başarısız testleri yaz**

`crates/jyotish-core/src/dasha.rs` `mod tests` sonuna:

```rust
    #[test]
    fn dasha_path_at_follows_the_containing_periods() {
        let seed = DashaSeed {
            birth_jd_ut: 2_451_545.0,
            moon_sidereal_lon: 199.470_529,
            year_days: 365.25,
        };
        // Doğum anı: doğum mahadashası ve onun içindeki dönemler.
        let at_birth = dasha_path_at(&seed, seed.birth_jd_ut, 3).unwrap();
        assert_eq!(at_birth.len(), 3);
        assert_eq!(at_birth[0], birth_dasha(seed.moon_sidereal_lon).0);
        // Her seviyede seçilen dönem anı içerir.
        let jd = seed.birth_jd_ut + 9_000.0;
        let path = dasha_path_at(&seed, jd, 5).unwrap();
        assert_eq!(path.len(), 5);
        for depth in 1..=5usize {
            let parent = &path[..depth - 1];
            let p = dasha_children(&seed, parent)
                .unwrap()
                .into_iter()
                .find(|p| p.lords == path[..depth])
                .unwrap();
            assert!(p.start_jd_ut <= jd && jd < p.end_jd_ut, "level {depth}");
        }
        // 120 yıllık döngünün dışı: yol yok.
        assert!(dasha_path_at(&seed, seed.birth_jd_ut + 200.0 * 365.25, 3)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn dasha_path_at_rejects_bad_input() {
        let seed = DashaSeed {
            birth_jd_ut: 2_451_545.0,
            moon_sidereal_lon: 10.0,
            year_days: 365.25,
        };
        for depth in [0, 6] {
            assert!(matches!(
                dasha_path_at(&seed, 2_451_545.0, depth),
                Err(CoreError::InvalidDashaInput("depth"))
            ));
        }
        assert!(matches!(
            dasha_path_at(&seed, f64::NAN, 3),
            Err(CoreError::InvalidDashaInput("jd_ut"))
        ));
    }
```

Create `app/src-tauri/src/transit_commands.rs` önce yalnızca testlerle (Step 3 gövdeyi ekler):

```rust
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// Süreçte yalnızca bir SE örneği açılabilir (`EphError::AlreadyOpen`) ve testler paralel
    /// koşar: bu modülün testleri tek motoru paylaşır (Plan 11 D20). Başka bir lib testi motor
    /// açmaz; açması gerekirse bu işlevi kullanır.
    fn engine() -> &'static Engine {
        static ENGINE: std::sync::OnceLock<Engine> = std::sync::OnceLock::new();
        ENGINE.get_or_init(|| {
            let ephe = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/ephe");
            Engine::spawn(ephe).expect("spawn engine")
        })
    }

    /// tests/golden/new-delhi-2000-01-01.json: JD 2451545.0, Ay 199.470529°.
    fn seed() -> DashaSeed {
        DashaSeed {
            birth_jd_ut: 2_451_545.0,
            moon_sidereal_lon: 199.470_529,
            year_days: 365.25,
        }
    }

    fn natal(engine: &Engine) -> Bhavas {
        crate::commands::compute_chart_blocking(engine, 2_451_545.0, 28.6139, 77.2090, CalcSettings::default())
            .expect("chart")
            .bhavas
    }

    #[test]
    fn now_jd_local_and_jd_resolve_to_the_same_moment() {
        let engine = engine();
        let bhavas = natal(engine);
        // 2026-11-24T00:00:00Z.
        let now = Timestamp::from_second(1_795_478_400).unwrap();
        let a = transits_at_blocking(engine, TransitAt::Now, now, CalcSettings::default(), bhavas.clone(), seed()).unwrap();
        assert_eq!(a.utc, "2026-11-24T00:00:00Z");
        let b = transits_at_blocking(
            engine,
            TransitAt::Local {
                date: "2026-11-24".into(),
                time: "03:00".into(),
                tz: "Europe/Istanbul".into(),
            },
            now,
            CalcSettings::default(),
            bhavas.clone(),
            seed(),
        )
        .unwrap();
        assert_eq!(b.utc, a.utc);
        assert_eq!(b.snapshot.jd_ut, a.snapshot.jd_ut);
        let c = transits_at_blocking(engine, TransitAt::Jd(a.snapshot.jd_ut), now, CalcSettings::default(), bhavas, seed()).unwrap();
        assert_eq!(c.snapshot, a.snapshot);
        assert_eq!(a.dasha_path.len(), usize::from(TRANSIT_DASHA_DEPTH));
        assert_eq!(a.dasha_path, c.dasha_path);
    }

    #[test]
    fn local_moments_follow_the_dst_rules() {
        let engine = engine();
        let bhavas = natal(engine);
        let now = Timestamp::from_second(1_795_478_400).unwrap();
        let local = |date: &str, time: &str| TransitAt::Local {
            date: date.into(),
            time: time.into(),
            tz: "Europe/Istanbul".into(),
        };
        // Boşluk: hata ve öneri, sessiz kaydırma yok.
        let gap = transits_at_blocking(engine, local("2014-03-31", "03:30"), now, CalcSettings::default(), bhavas.clone(), seed())
            .unwrap_err();
        assert_eq!(gap.code, "nonexistent_local_time");
        assert_eq!(gap.params["suggestion"], "2014-03-31T04:30:00");
        // Örtüşme: önceki an (yaz saati, +02:00).
        let berlin = TransitAt::Local {
            date: "2014-10-26".into(),
            time: "02:30".into(),
            tz: "Europe/Berlin".into(),
        };
        let overlap = transits_at_blocking(engine, berlin, now, CalcSettings::default(), bhavas.clone(), seed())
            .unwrap();
        assert_eq!(overlap.utc, "2014-10-26T00:30:00Z");
        // Bilinmeyen dilim ve bozuk tarih.
        let bad_tz = TransitAt::Local {
            date: "2026-11-24".into(),
            time: "03:00".into(),
            tz: "Mars/Olympus".into(),
        };
        assert_eq!(
            transits_at_blocking(engine, bad_tz, now, CalcSettings::default(), bhavas.clone(), seed()).unwrap_err().code,
            "unknown_time_zone"
        );
        assert_eq!(
            transits_at_blocking(engine, local("2026-13-01", "03:00"), now, CalcSettings::default(), bhavas, seed())
                .unwrap_err()
                .params["field"],
            "birth.date"
        );
    }

    #[test]
    fn bad_jd_is_an_error_not_a_panic() {
        let engine = engine();
        let bhavas = natal(engine);
        let now = Timestamp::from_second(1_795_478_400).unwrap();
        let e = transits_at_blocking(engine, TransitAt::Jd(f64::INFINITY), now, CalcSettings::default(), bhavas, seed())
            .unwrap_err();
        assert!(["invalid_julian_day", "invalid_transit_input"].contains(&e.code.as_str()), "{}", e.code);
    }
}
```

Not: `engine()` `&'static Engine` döner; testler onu `engine` olarak doğrudan geçirir (`&engine` değil). `Engine` `Send + Sync`'tir (`crates/ephemeris/src/engine.rs`).

Not: `Europe/Berlin`'de 2014-10-26 02:30 iki kez yaşandı (03:00 CEST → 02:00 CET); önceki an `+02:00` ofsetlidir, UTC `00:30`. İstanbul boşluğu (2014-03-31 03:30 → öneri 04:30) `tests/open_client.rs`'te doğrulanmış değerdir. (İstanbul'un 2014 güz geçişi seçim nedeniyle 9 Kasım'a ertelendiği için örtüşme testi Berlin'le yapılır.)

`app/src-tauri/src/lib.rs` modül listesine `pub mod transit_commands;` ekle (dosya yalnızca testlerle derlenmez; Step 2 bunu gösterir).

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
cargo test -p jyotish-core dasha_path_at 2>&1 | tail -4
cargo test -p jyotish-desk --lib transit_commands 2>&1 | tail -4
```
Expected: `cannot find function dasha_path_at`; `cannot find type TransitAt` / `transits_at_blocking`.

- [ ] **Step 3: Uygula**

`crates/jyotish-core/src/dasha.rs`, `dasha_children` fonksiyonunun altına:

```rust
/// `jd_ut` anını içeren dönemlerin lordları, kökten `depth` seviyeye (1 maha … 5 prana).
/// An 120 yıllık Vimshottari döngüsünün dışındaysa yol orada biter (boş ya da kısa yol).
/// Arayüzün "güncel dönem"i buradan gelir; arayüz tarih karşılaştırması yapmaz (C4).
pub fn dasha_path_at(seed: &DashaSeed, jd_ut: f64, depth: u8) -> Result<Vec<Graha>, CoreError> {
    if depth == 0 || depth > MAX_DASHA_LEVEL {
        return Err(CoreError::InvalidDashaInput("depth"));
    }
    if !jd_ut.is_finite() {
        return Err(CoreError::InvalidDashaInput("jd_ut"));
    }
    let mut path = Vec::with_capacity(usize::from(depth));
    let mut periods = mahadashas(seed)?;
    loop {
        let Some(p) = periods
            .iter()
            .find(|p| p.start_jd_ut <= jd_ut && jd_ut < p.end_jd_ut)
        else {
            return Ok(path);
        };
        path.push(*p.lords.last().expect("a period has a lord"));
        if path.len() == usize::from(depth) {
            return Ok(path);
        }
        periods = dasha_children(seed, &path)?;
    }
}
```

`crates/jyotish-core/src/lib.rs`'teki `dasha` dışa aktarım satırına `dasha_path_at`'ı ekle (ör. `pub use dasha::{dasha_children, dasha_path_at, …}`; listede ne varsa koru, yalnızca adı ekle).

`app/src-tauri/src/transit_commands.rs`'in başına (test modülünün üstüne):

```rust
//! Transit anı (UI spec §3.3 `transit_table`, harita panelinde transit halkası, dasa şeridi).
//! "Şimdi" Rust'ta okunur; seçilen yerel tarih-saat tarayıcının IANA diliminde çözülür
//! (boşlukta hata, örtüşmede önceki an; Plan 11 D7). Arayüz tarih hesabı yapmaz (C4).
//! Tarih ve ayarlar loglanmaz (C1).

use std::sync::Arc;

use atlas::Timestamp;
use ephemeris::Engine;
use jyotish_core::{dasha_path_at, Bhavas, CalcSettings, DashaSeed, Graha, TransitSnapshot};
use serde::{Deserialize, Serialize};
use specta::Type;
use store::{BirthData, FoldChoice, Place, TimeCertainty, TzChoice};
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::{compute_transits_blocking, CommandError};

/// Şeritte ve tabloda gösterilen dasa derinliği: maha, antar, pratyantar.
pub const TRANSIT_DASHA_DEPTH: u8 = 3;

/// Transit anı. `Local`: duvar saati (`YYYY-MM-DD`, `HH:MM`) ve IANA dilimi.
#[derive(Debug, Clone, PartialEq, Deserialize, Type)]
pub enum TransitAt {
    Now,
    Local { date: String, time: String, tz: String },
    Jd(f64),
}

/// Transit konumları, anın UTC'si (RFC 3339) ve o andaki dasa yolu.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct TransitView {
    pub snapshot: TransitSnapshot,
    pub utc: String,
    pub dasha_path: Vec<Graha>,
}

/// Duvar saatini çözer. Koordinat ve doğum notu anı etkilemez; doğrulama ve hata eşlemesi
/// doğum verisininkiyle aynıdır (`birth.date`, `birth.time`, `nonexistent_local_time` …).
fn resolve_local(date: &str, time: &str, tz: TzChoice) -> Result<(f64, String), CommandError> {
    let birth = BirthData {
        date: date.to_owned(),
        time: Some(time.to_owned()),
        time_certainty: TimeCertainty::Exact,
        place: Place {
            name: String::new(),
            country: String::new(),
            admin1: String::new(),
            lat: 0.0,
            lon: 0.0,
            geoname_id: None,
        },
        tz,
        fold: Some(FoldChoice::Earlier),
        notes: String::new(),
    };
    let (_, r) = store::resolve_birth(&birth)?;
    Ok((r.jd_ut, r.utc))
}

fn transit_moment(at: &TransitAt, now: Timestamp) -> Result<(f64, String), CommandError> {
    match at {
        TransitAt::Now => {
            // UTC duvar saati, saniye çözünürlüklü; JD dönüşümü atlas'ta.
            let date = now.strftime("%Y-%m-%d").to_string();
            let time = now.strftime("%H:%M:%S").to_string();
            resolve_local(&date, &time, TzChoice::Fixed(0))
        }
        TransitAt::Local { date, time, tz } => resolve_local(date, time, TzChoice::Iana(tz.clone())),
        TransitAt::Jd(jd) => {
            let secs = (jd - 2_440_587.5) * 86_400.0;
            let utc = if secs.is_finite() && secs.abs() < 1e13 {
                Timestamp::from_second(secs.round() as i64)
                    .map(|t| t.to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            };
            Ok((*jd, utc))
        }
    }
}

/// `transits_at` gövdesi; `now` testte sabitlenir. Bloklar: `spawn_blocking` içinde çağrılır.
pub fn transits_at_blocking(
    engine: &Engine,
    at: TransitAt,
    now: Timestamp,
    settings: CalcSettings,
    natal_bhavas: Bhavas,
    seed: DashaSeed,
) -> Result<TransitView, CommandError> {
    let (jd_ut, utc) = transit_moment(&at, now)?;
    let snapshot = compute_transits_blocking(engine, jd_ut, settings, natal_bhavas)?;
    let dasha_path = dasha_path_at(&seed, jd_ut, TRANSIT_DASHA_DEPTH)?;
    Ok(TransitView {
        snapshot,
        utc,
        dasha_path,
    })
}

/// Açık haritanın transitleri. `settings`, `natal_bhavas` ve `seed` açık haritadan gelir.
#[tauri::command]
#[specta::specta]
pub async fn transits_at(
    engine: State<'_, Arc<Engine>>,
    at: TransitAt,
    settings: CalcSettings,
    natal_bhavas: Bhavas,
    seed: DashaSeed,
) -> Result<TransitView, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || transits_at_blocking(&engine, at, Timestamp::now(), settings, natal_bhavas, seed)).await
}
```

`TransitAt::Jd` kolundaki `2_440_587.5` Unix çağının JD'sidir (`atlas::moment`'teki özel sabitle aynı değer); yalnızca gösterim dizgisi içindir, JD'nin kendisi değişmeden motora gider. `Timestamp::strftime` jiff'in `Timestamp` yöntemidir (`atlas::Timestamp` jiff'in yeniden dışa aktarımıdır). `store` crate'i `BirthData`, `FoldChoice`, `Place`, `TimeCertainty`, `TzChoice` ve `resolve_birth`'i kökten dışa aktarır (`crates/store/src/lib.rs`); `Place` alan adları farklıysa `crates/store/src/model.rs`'teki `pub struct Place`'e uy. `blocking` `pub(crate)`'tir ve aynı crate'ten çağrılır.

`app/src-tauri/src/lib.rs` `specta_builder` içindeki `collect_commands![…]` listesinde `commands::compute_transits,` satırının altına:

```rust
            transit_commands::transits_at,
```

`app/src-tauri/tests/ui_fixture.rs`: `use` satırlarına `use jyotish_desk_lib::transit_commands::{transits_at_blocking, TransitAt};` ekle; `let value = json!({` satırından önce:

```rust
    // Transit: sabit saat (fikstürün saati), "şimdi".
    let transit = transits_at_blocking(
        &engine,
        TransitAt::Now,
        Timestamp::from_second(1_795_600_800).expect("timestamp"),
        opened.calc_settings.clone(),
        opened.chart.bhavas.clone(),
        opened.chart.dasha.seed,
    )
    .expect("transit");
```

ve `json!` nesnesine `"transit": transit,` satırını `"gap_error": gap_error,`'un altına ekle. `DashaSeed` `Copy` değilse `opened.chart.dasha.seed.clone()` yaz.

`app/ui/test/fixture.ts`: tip içe aktarımına `TransitView` ekle ve arayüze:

```ts
export interface UiFixture {
  opened: OpenedClient;
  dasha_children: { path: Graha[]; periods: DashaPeriod[] };
  places: PlaceHit[];
  preview: MomentPreview;
  gap_error: CommandError;
  /** `transits_at(Now)`, saat 1 795 600 800 (fikstürün sabit saati). */
  transit: TransitView;
}
```

Başlık yorumuna "ve aynı sabit saatte transit" ifadesini ekle.

- [ ] **Step 4: Üret ve testlerin geçtiğini gör**

```bash
cargo test -p jyotish-core 2>&1 | grep 'test result'
cargo test -p jyotish-desk --lib 2>&1 | grep 'test result'
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture
cargo test -p jyotish-desk --test bindings --test ui_fixture
grep -c '"transit"' app/ui/test/fixtures/delhi.json
grep -n 'transitsAt\|export type TransitAt\|export type TransitView' app/ui/bindings.ts
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store -p jyotish-desk --all-targets
pnpm --dir app test && pnpm --dir app build
```
Expected: `jyotish-core` lib `110 passed` (+2); `jyotish-desk` lib `27 passed` (+3); fikstür ve bağlamalar yeniden üretilir, ardından ikisi değişmeden geçer; `grep -c` `1`; `bindings.ts`'te `transitsAt(at: TransitAt, settings: CalcSettings, natalBhavas: Bhavas, seed: DashaSeed)`, `TransitAt = "Now" | { Local: … } | { Jd: number | null }` ve `TransitView`; fmt/clippy çıktısız; MSRV `Finished`; Vitest `43 passed`, `363 passed`; build `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add crates/jyotish-core/src app/src-tauri/src app/src-tauri/tests/ui_fixture.rs app/ui/bindings.ts app/ui/test
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core,app): dasha path at a moment and transits_at with Rust-side now

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Komutlar ve sarmalayıcılar: `PrefsLoad`, `keep_stored_moment`, atlas nedeni, `save_view_state` gövdesi; gidiş-dönüş testleri

`get_app_prefs` `PrefsLoad` döner (Task 1); `keep_stored_moment` komutu saklanan anı korur ve haritayı yeniden açar (Task 2, D5). Atlas açılamazsa `atlas_unavailable.reason` nedeni taşır (D18). `save_view_state` komutunun gövdesi test edilebilir bir fonksiyona çıkar (Plan 10 kapsam dışı 21). Spec §8 gidiş-dönüş testi tercihleri de kapsar (D19); `ui_fixture` doğum verisini girdisiyle karşılaştırır ve `prefs` anahtarını kazanır. `api.ts` Plan 11 arayüzünün çağıracağı komutları sarar.

**Files:**
- Modify: `app/src-tauri/src/client_commands.rs` (`keep_stored_moment_blocking`, komut, `save_view_state_body`, testler)
- Modify: `app/src-tauri/src/store_commands.rs` (`get_app_prefs`)
- Modify: `app/src-tauri/src/atlas_commands.rs` (`AtlasState` nedeni, test)
- Modify: `app/src-tauri/src/lib.rs` (kayıt, `AtlasState::open` çağrısı değişmez)
- Modify: `app/src-tauri/tests/open_client.rs`, `app/src-tauri/tests/ui_fixture.rs`
- Modify: `app/ui/api.ts`, `app/ui/api.test.ts`, `app/ui/test/fixture.ts`
- Modify: `app/ui/i18n/{tr,en,fr}.ts` (`errors.atlas_unavailable` nedenleri)
- Generated: `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`

**Interfaces:**
- Consumes: `Store::app_prefs_load`, `Store::keep_stored_moment`, `open_client_blocking`, `with_store`, `lock`.
- Produces:
  - `pub fn keep_stored_moment_blocking(engine: &Engine, store: &Mutex<Store>, chart_id: u32) -> Result<OpenedClient, CommandError>`; komut `keep_stored_moment(chart_id)` → `commands.keepStoredMoment(chartId)`
  - `pub fn save_view_state_body(store: &Mutex<Store>, chart_id: u32, view_state: &ViewState) -> Result<(), CommandError>`
  - `get_app_prefs` → `Result<PrefsLoad, CommandError>`
  - `pub enum AtlasUnavailable { Missing, Schema, Io }` (yalnızca Rust); `atlas_unavailable` parametresi `reason`: `"missing" | "schema" | "io"`
  - `api.getAppPrefs()`, `api.setAppPrefs(prefs)`, `api.saveCalcSettings(chartId, settings)`, `api.updateClient(clientId, name, gender)`, `api.transitsAt(at, settings, natalBhavas, seed)`, `api.keepStoredMoment(chartId)`
  - fikstür: `prefs: PrefsLoad` (boş store: varsayılanlar, uyarı yok)

- [ ] **Step 1: Başarısız testleri yaz**

`app/src-tauri/src/client_commands.rs` `mod tests` sonuna:

```rust
    #[test]
    fn save_view_state_body_writes_and_maps_errors() {
        use store::{PanelLayout, ViewState};
        // `store`'un test yardımcısı crate dışına açık değildir; danışan burada kurulur.
        let new_client = store::NewClient {
            name: "A".into(),
            gender: None,
            chart: store::NewChart {
                label: String::new(),
                kind: store::ChartKind::Natal,
                birth: store::BirthData {
                    date: "2000-01-01".into(),
                    time: Some("17:30".into()),
                    time_certainty: store::TimeCertainty::Exact,
                    place: store::Place {
                        name: "New Delhi".into(),
                        country: "India".into(),
                        admin1: "NCT".into(),
                        lat: 28.6139,
                        lon: 77.2090,
                        geoname_id: Some(1_261_481),
                    },
                    tz: store::TzChoice::Iana("Asia/Kolkata".into()),
                    fold: None,
                    notes: String::new(),
                },
            },
        };
        let store = Mutex::new(Store::open_in_memory().unwrap());
        let (_, chart) = lock(&store).unwrap().create_client(&new_client).unwrap();
        let vs = ViewState {
            dasha_expanded_path: vec![jyotish_core::Graha::Rahu],
            ..ViewState::default()
        };
        save_view_state_body(&store, chart.id, &vs).unwrap();
        assert_eq!(lock(&store).unwrap().view_state(chart.id).unwrap(), vs);
        let e = save_view_state_body(&store, 999, &vs).unwrap_err();
        assert_eq!((e.code.as_str(), e.params["what"].as_str()), ("not_found", "chart"));
        let bad = ViewState {
            schema_version: 9,
            panel_layout: None::<PanelLayout>,
            ..ViewState::default()
        };
        let e = save_view_state_body(&store, chart.id, &bad).unwrap_err();
        assert_eq!((e.code.as_str(), e.params["field"].as_str()), ("invalid_input", "view_state"));
    }
```

`app/src-tauri/src/atlas_commands.rs` `mod tests` sonuna:

```rust
    #[test]
    fn unavailable_atlas_reports_why_without_a_path() {
        let dir = tempfile::tempdir().unwrap();
        let missing = AtlasState::open(&dir.path().join("atlas.sqlite"));
        let e = search_places_blocking(&missing, "delhi", 5).unwrap_err();
        assert_eq!((e.code.as_str(), e.params["reason"].as_str()), ("atlas_unavailable", "missing"));
        assert_eq!(e.params.len(), 1);
        let junk = dir.path().join("junk.sqlite");
        std::fs::write(&junk, b"not a database").unwrap();
        let e = search_places_blocking(&AtlasState::open(&junk), "delhi", 5).unwrap_err();
        assert!(["schema", "io"].contains(&e.params["reason"].as_str()), "{:?}", e.params);
        assert_eq!(
            search_places_blocking(&AtlasState::unavailable(), "delhi", 5).unwrap_err().params["reason"],
            "missing"
        );
    }
```

(`tempfile` `jyotish-desk`'in dev bağımlılığıdır; `tests/open_client.rs` kullanır.)

`app/src-tauri/tests/open_client.rs`: `use` listesine `keep_stored_moment_blocking`'i ve `store::AppPrefs`'i ekle; `// Kapat ve yeniden aç (uygulama yeniden başlatma).` satırından önce:

```rust
    // Genel varsayılan (spec §4 "Ayar katmanları") da yeniden başlatmadan sağ çıkar.
    let prefs = AppPrefs {
        locale: "en".into(),
        calc_settings: raman.clone(),
        ..AppPrefs::default()
    };
    store.lock().unwrap().set_app_prefs(&prefs).unwrap();
```

ve `assert!(after.warnings.is_empty());` satırının altına:

```rust
    let load = store.lock().unwrap().app_prefs_load().unwrap();
    assert_eq!(load.prefs, prefs);
    assert!(load.warnings.is_empty());
    // "Saklanan zamanı koru" haritayı yeniden açar; an ve harita değişmez.
    let kept = keep_stored_moment_blocking(&engine, &store, chart_id).expect("keep");
    assert_eq!(kept.chart_record.resolved.utc, after.chart_record.resolved.utc);
    assert_eq!(kept.chart, after.chart);
```

`app/src-tauri/tests/ui_fixture.rs`: `let value = json!({` satırından önce:

```rust
    // Kaydedilen doğum verisi girdinin kanonik biçimidir (Plan 10 kapsam dışı 21).
    assert_eq!(opened.chart_record.birth, delhi());
    let prefs = store.lock().expect("store").app_prefs_load().expect("prefs");
```

ve `json!`'a `"prefs": prefs,` ekle. Aynı dosyadaki `delhi()` içinde `time: Some("17:30".into()),` satırını `time: Some("17:30:00".into()),` yap: store saati jiff'in kanonik biçiminde saklar (`17:30:00`; fikstürdeki `"time":"17:30:00"` bunu gösterir) ve eşitlik ancak kanonik girdiyle tutar. Üretilen fikstür bu değişiklikle değişmez (çıktı zaten kanoniktir). Kanonikleştirmeye dokunma.

`app/ui/api.test.ts`: sarmalayıcı iletim testinin (her `api` anahtarının aynı adlı `commands` işlevine argümanları ilettiğini denetleyen test) `vi.mock("./bindings", …)` taklit listesine `getAppPrefs`, `setAppPrefs`, `saveCalcSettings`, `updateClient`, `transitsAt`, `keepStoredMoment` ekle; beklenen anahtar listesi varsa aynılarını ekle. Dosyada beklenen anahtarlar `Object.keys(api)` ile karşılaştırılıyorsa yeni anahtarlar kendiliğinden kapsanır; taklit edilmeyen komut testi `commands.X is not a function` ile düşürür (Step 2'de görülecek hata budur).

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
cargo test -p jyotish-desk --lib save_view_state_body unavailable_atlas 2>&1 | tail -4
cargo test -p jyotish-desk --test open_client 2>&1 | tail -4
pnpm --dir app exec vitest run ui/api.test.ts
```
Expected: `cannot find function save_view_state_body`, `no field reason`/`params["reason"]` paniği; `open_client` derlenmez (`keep_stored_moment_blocking`); vitest api testi henüz değişmeyen `api` ile geçer (taklit eklendi, sarmalayıcı yok: Step 3'ten sonra yeni anahtarlar iletim testine girer).

- [ ] **Step 3: Uygula**

`app/src-tauri/src/client_commands.rs`, `update_chart_blocking`'in altına:

```rust
/// tzdb farkında "Saklanan zamanı koru" (Plan 11 D5): kayıt güncel tzdb ile damgalanır, an
/// değişmez; harita yeniden açılır (uyarısız).
pub fn keep_stored_moment_blocking(
    engine: &Engine,
    store: &Mutex<Store>,
    chart_id: u32,
) -> Result<OpenedClient, CommandError> {
    let rec = lock(store)?.keep_stored_moment(chart_id)?;
    open_client_blocking(engine, store, rec.client_id, Some(chart_id))
}

/// `save_view_state` gövdesi (görünüm durumu: dasha yolu + kullanıcının düzeni).
pub fn save_view_state_body(
    store: &Mutex<Store>,
    chart_id: u32,
    view_state: &ViewState,
) -> Result<(), CommandError> {
    Ok(lock(store)?.save_view_state(chart_id, view_state)?)
}
```

Var olan `save_view_state` komutunun gövdesini şununla değiştir (imza aynı kalır):

```rust
    let store = Arc::clone(&store);
    blocking(move || save_view_state_body(&store, chart_id, &view_state)).await
```

ve `update_chart` komutunun altına:

```rust
#[tauri::command]
#[specta::specta]
pub async fn keep_stored_moment(
    engine: State<'_, Arc<Engine>>,
    store: State<'_, StoreState>,
    chart_id: u32,
) -> Result<OpenedClient, CommandError> {
    let (engine, store) = (Arc::clone(&engine), Arc::clone(&store));
    blocking(move || keep_stored_moment_blocking(&engine, &store, chart_id)).await
}
```

(`save_view_state` komutunun parametre adları dosyadakiyle aynı kalmalıdır: `chart_id`, `view_state`; farklıysa gövdedeki adları onlara uydur.)

`app/src-tauri/src/store_commands.rs` `get_app_prefs`:

```rust
#[tauri::command]
#[specta::specta]
pub async fn get_app_prefs(store: State<'_, StoreState>) -> Result<PrefsLoad, CommandError> {
    with_store(&store, |s| s.app_prefs_load()).await
}
```

ve `use store::{…}` listesine `PrefsLoad` ekle (`AppPrefs` `set_app_prefs` için kalır).

`app/src-tauri/src/atlas_commands.rs`: `pub struct AtlasState(Option<Mutex<Atlas>>);` ve `impl AtlasState` bloğunu şununla değiştir:

```rust
/// Atlasın neden açılamadığı (Plan 11 D18). Yol taşımaz (C1, C2′).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtlasUnavailable {
    /// Dosya yok (paket eksik ya da geliştirme kopyasında atlas üretilmemiş).
    Missing,
    /// Dosya var ama şema beklenen değil ya da SQLite veritabanı değil.
    Schema,
    /// Okuma hatası (izin, aygıt).
    Io,
}

impl AtlasUnavailable {
    fn code(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Schema => "schema",
            Self::Io => "io",
        }
    }
}

pub struct AtlasState(Result<Mutex<Atlas>, AtlasUnavailable>);

impl AtlasState {
    pub fn open(path: &Path) -> Self {
        if !path.exists() {
            return Self(Err(AtlasUnavailable::Missing));
        }
        Self(Atlas::open(path).map(Mutex::new).map_err(|e| match e {
            AtlasError::Io(_) => AtlasUnavailable::Io,
            AtlasError::Sqlite(_) | AtlasError::SchemaMismatch { .. } | AtlasError::Geonames(_) => {
                AtlasUnavailable::Schema
            }
        }))
    }

    pub fn unavailable() -> Self {
        Self(Err(AtlasUnavailable::Missing))
    }

    pub fn is_available(&self) -> bool {
        self.0.is_ok()
    }

    fn get(&self) -> Result<&Mutex<Atlas>, CommandError> {
        self.0
            .as_ref()
            .map_err(|r| CommandError::new("atlas_unavailable", &[("reason", r.code().to_owned())]))
    }
}
```

`search_places_blocking` içindeki `let Some(atlas) = &atlas.0 else { … };` bloğunu `let atlas = atlas.get()?;` ile, kilit hatasındaki `CommandError::new("atlas_unavailable", &[])`'yi `CommandError::new("atlas_unavailable", &[("reason", "io".to_owned())])` ile değiştir. `use atlas::{…}` listesine `AtlasError` ekle. `AtlasError` varyant adları (`Sqlite`, `Io`, `Geonames`, `SchemaMismatch`) `crates/atlas/src/db.rs`'tekiyle aynıdır; bir varyantın alanı yoksa deseni `AtlasError::Geonames` biçiminde yaz. `atlas_commands.rs`'te `atlas.0`'a başka erişim varsa (ör. `preview_moment`) `get()` üzerinden yap.

`app/src-tauri/src/lib.rs` `collect_commands![…]` listesinde `client_commands::update_chart,` satırının altına `client_commands::keep_stored_moment,` ekle.

`app/ui/api.ts`: tip içe aktarımına `AppPrefs`, `Bhavas`, `CalcSettings`, `Gender`, `TransitAt` ekle; `api` nesnesinde `updateChart` satırının altına:

```ts
  keepStoredMoment: (chartId: number) => wrap(() => commands.keepStoredMoment(chartId)),
  updateClient: (clientId: number, name: string, gender: Gender | null) => wrap(() => commands.updateClient(clientId, name, gender)),
  saveCalcSettings: (chartId: number, settings: CalcSettings) => wrap(() => commands.saveCalcSettings(chartId, settings)),
  getAppPrefs: () => wrap(() => commands.getAppPrefs()),
  setAppPrefs: (prefs: AppPrefs) => wrap(() => commands.setAppPrefs(prefs)),
  transitsAt: (at: TransitAt, settings: CalcSettings, natalBhavas: Bhavas, seed: DashaSeed) => wrap(() => commands.transitsAt(at, settings, natalBhavas, seed)),
```

Dosya başı yorumundaki "İçe ve dışa aktarma komutları … bilerek yoktur" paragrafı Plan 10'da güncellendiyse dokunma; güncellenmediyse "JHD içe aktarma kayıtsızdır (C2′)" cümlesiyle değiştir.

`app/ui/test/fixture.ts` arayüzüne `prefs: PrefsLoad;` ekle (tip içe aktarımına `PrefsLoad`).

`app/ui/i18n/tr.ts` `errors.atlas_unavailable` metnini neden bağlamlı yap; i18next `context` seçeneği ayrılmış olduğundan (`errors.ts` `RESERVED_OPTIONS`) neden ayrı anahtardır. `errors` içinde `atlas_unavailable` satırını şununla değiştir:

```ts
    atlas_unavailable: "Yer araması kullanılamıyor ({{reason}}). Enlem, boylam ve saat dilimini elle girebilirsiniz.",
```

ve `errors` nesnesinin dışına, `notice`'in üstüne:

```ts
  atlasReason: { missing: "atlas dosyası yok", schema: "atlas dosyası okunamıyor", io: "atlas dosyasına erişilemiyor" },
```

`en.ts`:

```ts
    atlas_unavailable: "Place search is unavailable ({{reason}}). You can enter latitude, longitude and time zone by hand.",
```
```ts
  atlasReason: { missing: "the atlas file is missing", schema: "the atlas file cannot be read", io: "the atlas file cannot be accessed" },
```

`fr.ts`:

```ts
    atlas_unavailable: "La recherche de lieux est indisponible ({{reason}}). Vous pouvez saisir la latitude, la longitude et le fuseau à la main.",
```
```ts
  atlasReason: { missing: "fichier d’atlas absent", schema: "fichier d’atlas illisible", io: "fichier d’atlas inaccessible" },
```

`app/ui/i18n/errors.ts` `errorMessage`'i neden çevirisiyle genişlet:

```ts
export function errorMessage(t: TFunction, i18n: I18n, error: CommandError): string {
  const key = `errors.${error.code}`;
  if (!i18n.exists(key)) return t("errors.unknown", { code: error.code });
  const params = safeParams(error.params);
  // Kararlı kodlu parametreler arayüz dilinde gösterilir.
  if (error.code === "atlas_unavailable") {
    const reason = String(error.params.reason ?? "missing");
    params.reason = i18n.exists(`atlasReason.${reason}`) ? t(`atlasReason.${reason}`) : reason;
  }
  return t(key, params);
}
```

`app/ui/i18n/i18n.test.ts` sonuna:

```ts
test("atlas reasons are localized in every language", () => {
  for (const lang of [tr, en, fr]) expect(Object.keys(lang.atlasReason).sort()).toEqual(["io", "missing", "schema"]);
});
```

(`en` ve `fr` dosyada içe aktarılmıyorsa `import { en } from "./en"; import { fr } from "./fr";` ekle; dışa aktarım adları `tr.ts`'teki `export const tr` ile aynı biçimdedir.)

- [ ] **Step 4: Üret ve testlerin geçtiğini gör**

```bash
cargo test -p jyotish-desk --lib 2>&1 | grep 'test result'
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture
cargo test -p jyotish-desk --test bindings --test ui_fixture --test open_client
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store -p jyotish-desk --all-targets
pnpm --dir app test && pnpm --dir app build
```
Expected: `jyotish-desk` lib `29 passed` (+2); `open_client` 1, `ui_fixture` 2, `bindings` 1 geçer; `bindings.ts`'te `getAppPrefs(): Promise<Result<PrefsLoad, CommandError>>` ve `keepStoredMoment(chartId: number)`; Vitest `43 passed`, `Tests  364 passed (364)` (+1: atlas nedenleri); build `✓ built`. `boot.ts` henüz `getAppPrefs` çağırmaz.

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri app/ui/api.ts app/ui/api.test.ts app/ui/bindings.ts app/ui/test app/ui/i18n
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(app): prefs load, keep stored moment, atlas reason and round-trip tests

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Arayüz: tercih deposu ve ayarlar penceresi (`⌘,`)

Spec §4 "Ayar katmanları": açık haritanın ayarı düzenlenir ve harita Rust'ta yeniden hesaplanır (`save_calc_settings`); harita yoksa pencere yeni haritaların genel varsayılanını (`AppPrefs.calc_settings`) düzenler. Kayıt 500 ms debounce ile sıralıdır (son kazanır), sayfa gizlenince ve pencere kapanınca boşaltılır; kapandıktan sonra başarısız olan kayıt bildirim bandına düşer (C5, C17, D11). Açılış tercihleri okur (`get_app_prefs`). `⌘,` yalnızca `shortcuts.ts`'te (C7); palet ve üst çubuk da pencereyi açar.

**Files:**
- Create: `app/ui/settings/model.ts`, `app/ui/settings/model.test.ts`
- Create: `app/ui/settings/prefs.tsx`
- Create: `app/ui/settings/saves.ts`
- Create: `app/ui/settings/SettingsDialog.tsx`, `app/ui/settings/SettingsDialog.test.tsx`, `app/ui/settings/settings.css`
- Modify: `app/ui/boot.ts`, `app/ui/App.tsx`, `app/ui/test/render.tsx`
- Modify: `app/ui/session/store.tsx` (`updateSettings`)
- Modify: `app/ui/shortcuts.ts`, `app/ui/shortcuts.test.ts`
- Modify: `app/ui/palette/items.ts`, `app/ui/palette/Palette.tsx`
- Modify: `app/ui/shell/Shell.tsx`, `app/ui/shell/TopBar.tsx`
- Modify: `app/ui/i18n/{tr,en,fr}.ts`
- Modify: `app/e2e/ipc.ts` (`get_app_prefs`, `set_app_prefs`, `save_calc_settings`)

**Interfaces:**
- Consumes: `api.getAppPrefs`, `api.setAppPrefs`, `api.saveCalcSettings` (Task 4); `PrefsLoad` (Task 1); `SAVE_DEBOUNCE_MS` (`view/store.tsx`); `shell/Dialog`; `errorMessage`; `isLanguage`, `DEFAULT_LANGUAGE` (`i18n/index.ts`).
- Produces:
  - `settings/model.ts`: `ZODIACS`, `PRESET_AYANAMSAS`, `NODES`, `HOUSE_SYSTEMS`, `SUNRISE_DEFS`, `DASHA_YEARS`, `ORB_KEYS`, `OrbKey`, `SETTINGS_FIELDS`, `SettingsField`, `sameValue(a, b)`, `settingsDiff(a, b): SettingsField[]`, `ayanamsaKey(a)`, `MAX_ORB_DEG`, `parseOrb(text): number | null`
  - `settings/prefs.tsx`: `PrefsProvider({ initial: AppPrefs | null })`, `usePrefs(): { prefs: AppPrefs | null; save(next): Promise<CommandError | null> }`
  - `settings/saves.ts`: `createSaver<T>(write, { debounceMs, onError, onSaved }): { schedule(v: T): void; flush(): void }`
  - `SettingsDialog({ onClose, debounceMs? })`
  - `Session.updateSettings(chartId: number, settings: CalcSettings, chart: Chart): void`
  - `Boot.prefs: PrefsLoad | null`, `Boot.prefsError: CommandError | null`
  - `Command` `{ id: "settings" }`, `ShortcutInfo.id` `"settings"`, `PaletteCommand` `"settings"`
  - `RenderOptions.prefs?: AppPrefs | null` (varsayılan `fixture().prefs.prefs`)
  - i18n: `settings.*` (üst çubuk düğmesi `settings.open`), `shortcuts.settings`, `palette.command.settings`, `notice.settingsSaveFailed`, `notice.prefsLoadFailed`

- [ ] **Step 1: Başarısız testleri yaz**

Create `app/ui/settings/model.test.ts`:

```ts
import { expect, test } from "vitest";
import type { CalcSettings } from "../bindings";
import { fixture } from "../test/fixture";
import { ayanamsaKey, parseOrb, sameValue, settingsDiff } from "./model";

const base = (): CalcSettings => fixture().opened.calc_settings;

test("equal settings have no difference, key order does not matter", () => {
  const a = base();
  const b = { ...base(), combustion_orbs: Object.fromEntries(Object.entries(base().combustion_orbs).reverse()) } as CalcSettings;
  expect(settingsDiff(a, b)).toEqual([]);
  expect(sameValue({ x: [1, { y: 2 }] }, { x: [1, { y: 2 }] })).toBe(true);
  expect(sameValue({ x: 1 }, { x: 1, y: undefined })).toBe(false);
});

test("differences are listed in field order", () => {
  const b: CalcSettings = { ...base(), node: "True", ayanamsa: "Raman", combustion_orbs: { ...base().combustion_orbs, moon: 1 } };
  expect(settingsDiff(base(), b)).toEqual(["ayanamsa", "node", "combustion_orbs"]);
});

test("orb text accepts a decimal comma and rejects junk or out of range", () => {
  expect(parseOrb("8")).toBe(8);
  expect(parseOrb(" 8,5 ")).toBe(8.5);
  expect(parseOrb("30")).toBe(30);
  for (const bad of ["", "-1", "31", "8.", "abc", "1e2"]) expect(parseOrb(bad), bad).toBeNull();
});

test("custom ayanamsa is recognised", () => {
  expect(ayanamsaKey("Lahiri")).toBe("Lahiri");
  expect(ayanamsaKey({ Custom: { t0_jd: 2451545, deg: 23.85 } })).toBe("Custom");
});
```

Create `app/ui/settings/SettingsDialog.test.tsx`:

```tsx
import { act, fireEvent, screen, waitFor } from "@testing-library/react";
import { useState } from "react";
import { afterEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    saveCalcSettings: vi.fn(),
    setAppPrefs: vi.fn(),
  },
}));

import { api } from "../api";
import type { CalcSettings, OpenedClient } from "../bindings";
import { useSession } from "../session/store";
import { NoticeBar } from "../shell/notices";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { SettingsDialog } from "./SettingsDialog";

afterEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
});

function Probe() {
  const { opened } = useSession();
  // `<output>` örtük `role="status"` taşır ve pencerenin durum satırıyla karışır; `<span>` kullanılır.
  return <span data-testid="node">{opened?.calc_settings.node}</span>;
}

function openedWith(patch: Partial<CalcSettings>): OpenedClient {
  const o = fixture().opened;
  o.calc_settings = { ...o.calc_settings, ...patch };
  return o;
}

const okChart = () => ({ status: "ok" as const, data: fixture().opened.chart });

test("an edit is saved after the debounce and the open chart takes the new settings", async () => {
  vi.mocked(api.saveCalcSettings).mockResolvedValue(okChart());
  renderWithStores(
    <>
      <SettingsDialog onClose={() => {}} debounceMs={0} />
      <Probe />
    </>,
  );
  fireEvent.change(screen.getByLabelText("Düğüm"), { target: { value: "True" } });
  await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(1));
  const [chartId, settings] = vi.mocked(api.saveCalcSettings).mock.calls[0];
  expect(chartId).toBe(fixture().opened.chart_record.id);
  expect(settings.node).toBe("True");
  await waitFor(() => expect(screen.getByTestId("node")).toHaveTextContent("True"));
  expect(screen.getByRole("status")).toHaveTextContent("Kaydedildi.");
});

test("rapid edits are written once and the last one wins", async () => {
  vi.mocked(api.saveCalcSettings).mockResolvedValue(okChart());
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={50} />);
  fireEvent.change(screen.getByLabelText("Ev sistemi"), { target: { value: "Placidus" } });
  fireEvent.change(screen.getByLabelText("Ev sistemi"), { target: { value: "Sripati" } });
  await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(1));
  expect(vi.mocked(api.saveCalcSettings).mock.calls[0][1].house_system).toBe("Sripati");
});

test("fields that differ from the default are marked and reset to default saves the default", async () => {
  vi.mocked(api.saveCalcSettings).mockResolvedValue(okChart());
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />, { opened: openedWith({ node: "True" }) });
  expect(screen.getAllByText("varsayılandan farklı")).toHaveLength(1);
  fireEvent.click(screen.getByRole("button", { name: "Varsayılana döndür" }));
  await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(1));
  expect(vi.mocked(api.saveCalcSettings).mock.calls[0][1]).toEqual(fixture().prefs.prefs.calc_settings);
  await waitFor(() => expect(screen.queryByText("varsayılandan farklı")).toBeNull());
});

test("make default writes the app prefs with the current language", async () => {
  vi.mocked(api.setAppPrefs).mockResolvedValue({ status: "ok", data: null });
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />, { opened: openedWith({ node: "True" }) });
  fireEvent.click(screen.getByRole("button", { name: "Bunu varsayılan yap" }));
  await waitFor(() => expect(api.setAppPrefs).toHaveBeenCalledTimes(1));
  const prefs = vi.mocked(api.setAppPrefs).mock.calls[0][0];
  expect(prefs.locale).toBe("tr");
  expect(prefs.calc_settings.node).toBe("True");
  await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("Yeni haritaların varsayılanı olarak kaydedildi."));
  expect(screen.queryByText("varsayılandan farklı")).toBeNull();
});

test("an invalid orb is flagged and not saved; a custom ayanamsa is shown but locked", async () => {
  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />, {
    opened: openedWith({ ayanamsa: { Custom: { t0_jd: 2451545, deg: 23.85 } } }),
  });
  const ayanamsa = screen.getByLabelText("Ayanamsa");
  expect(ayanamsa).toBeDisabled();
  expect(ayanamsa).toHaveAccessibleDescription("İçe aktarılmış özel ayanamsa burada değiştirilemez.");
  const moon = screen.getByLabelText("Ay");
  fireEvent.change(moon, { target: { value: "abc" } });
  expect(moon).toHaveAttribute("aria-invalid", "true");
  expect(moon).toHaveAccessibleDescription("0 ile 30 arasında bir sayı girin.");
  await act(() => new Promise((r) => setTimeout(r, 20)));
  expect(api.saveCalcSettings).not.toHaveBeenCalled();
});

test("a save that fails after the dialog closed is reported in the notice bar", async () => {
  vi.mocked(api.saveCalcSettings).mockResolvedValue({ status: "error", error: { code: "store_sqlite", params: { detail: "disk" } } });
  function Host() {
    const [open, setOpen] = useState(true);
    return (
      <>
        {open && <SettingsDialog onClose={() => setOpen(false)} debounceMs={10_000} />}
        <NoticeBar />
      </>
    );
  }
  renderWithStores(<Host />);
  fireEvent.change(screen.getByLabelText("Düğüm"), { target: { value: "True" } });
  fireEvent.click(screen.getByRole("button", { name: "Kapat" }));
  await waitFor(() => expect(screen.getByRole("alert")).toHaveTextContent("Ayarlar kaydedilemedi"));
  expect(api.saveCalcSettings).toHaveBeenCalledTimes(1);
});
```

`app/ui/shortcuts.test.ts` sonuna (dosyadaki olay yardımcısı `key(…)` ya da `new KeyboardEvent` hangisiyse onu kullan; aşağıdaki doğrudan olay kurar):

```ts
test("⌘, opens the settings, also from a text field", () => {
  const input = document.createElement("input");
  document.body.append(input);
  const e = new KeyboardEvent("keydown", { key: ",", code: "Comma", metaKey: true, bubbles: true });
  Object.defineProperty(e, "target", { value: input });
  expect(matchShortcut(e)).toEqual({ id: "settings" });
  expect(SHORTCUT_LIST.some((s) => s.id === "settings" && s.keys === "⌘,")).toBe(true);
  input.remove();
});
```

(`SHORTCUT_LIST` dosyada içe aktarılmıyorsa içe aktarıma ekle.)

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/settings ui/shortcuts.test.ts
```
Expected: `Failed to resolve import "./model"` / `"./SettingsDialog"`; shortcuts testi `expected null to deeply equal { id: 'settings' }`.

- [ ] **Step 3: Uygula**

Create `app/ui/settings/model.ts`:

```ts
// Hesap ayarlarının arayüz modeli (spec §4 "Ayar katmanları"). Ayarların anlamı ve doğrulaması
// Rust'tadır (`CalcSettings::validate`); burada seçenek listeleri, alan karşılaştırması ve metin
// girişinin sayıya çevrilmesi vardır. Hesap yoktur (C4).
import type { CalcSettings, CombustionOrbs } from "../bindings";

export const ZODIACS = ["Sidereal", "TropicalRasiSiderealNakshatra"] as const satisfies readonly CalcSettings["zodiac"][];
/** Seçilebilir ayanamsalar; `Custom` yalnızca gösterilir (Plan 11 D11). */
export const PRESET_AYANAMSAS = ["Lahiri", "Raman", "Kp", "TrueChitra"] as const;
export const NODES = ["Mean", "True"] as const satisfies readonly CalcSettings["node"][];
export const HOUSE_SYSTEMS = ["WholeSign", "Sripati", "Placidus"] as const satisfies readonly CalcSettings["house_system"][];
export const SUNRISE_DEFS = ["DiskCenterNoRefraction", "UpperLimbRefracted"] as const satisfies readonly CalcSettings["sunrise_def"][];
/** Seçilebilir Vimshottari yıl uzunlukları (gün): Julian yılı, savana yılı, tropik yıl. */
export const DASHA_YEARS = [365.25, 360, 365.2422] as const;
export const ORB_KEYS = ["moon", "mars", "mercury", "mercury_retro", "jupiter", "venus", "venus_retro", "saturn"] as const satisfies readonly (keyof CombustionOrbs)[];
export type OrbKey = (typeof ORB_KEYS)[number];

/** Karşılaştırılan ve gösterilen alanlar, pencere sırasıyla (`schema_version` hariç). */
export const SETTINGS_FIELDS = ["zodiac", "ayanamsa", "node", "house_system", "dasha_year", "combustion_orbs", "sunrise_def"] as const satisfies readonly (keyof CalcSettings)[];
export type SettingsField = (typeof SETTINGS_FIELDS)[number];

/** Yapısal eşitlik; nesne anahtarlarının sırası önemsizdir. */
export function sameValue(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (typeof a !== "object" || typeof b !== "object" || a === null || b === null) return false;
  const ka = Object.keys(a);
  const kb = Object.keys(b);
  return ka.length === kb.length && ka.every((k) => k in b && sameValue((a as Record<string, unknown>)[k], (b as Record<string, unknown>)[k]));
}

/** `a`'nın `b`'den farklı alanları (fark rozeti, Plan 11 D12). */
export function settingsDiff(a: CalcSettings, b: CalcSettings): SettingsField[] {
  return SETTINGS_FIELDS.filter((f) => !sameValue(a[f], b[f]));
}

export function ayanamsaKey(a: CalcSettings["ayanamsa"]): (typeof PRESET_AYANAMSAS)[number] | "Custom" {
  return typeof a === "string" ? a : "Custom";
}

/** Arayüzün kabul ettiği en büyük yanma orbu (derece); kesin aralık Rust'ta denetlenir. */
export const MAX_ORB_DEG = 30;

/** Orb metni → derece. Ondalık ayırıcı nokta ya da virgül olabilir. */
export function parseOrb(text: string): number | null {
  const s = text.trim().replace(",", ".");
  if (!/^\d+(\.\d+)?$/.test(s)) return null;
  const v = Number(s);
  return v <= MAX_ORB_DEG ? v : null;
}
```

Create `app/ui/settings/saves.ts`:

```ts
// Sıralı, debounce'lu otomatik kayıt (C5): uçuşta en çok bir yazım; bekleyen değer yalnızca
// sonuncusudur. `flush` bekleyeni hemen yazar (pencere kapanırken, sayfa gizlenirken; C17).
import type { CommandError } from "../bindings";

export interface Saver<T> {
  schedule(value: T): void;
  flush(): void;
}

export interface SaverOptions<T> {
  debounceMs: number;
  onError(error: CommandError): void;
  onSaved(value: T): void;
}

export function createSaver<T>(write: (value: T) => Promise<CommandError | null>, opts: SaverOptions<T>): Saver<T> {
  let pending: { value: T } | null = null;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let flying = false;

  const pump = async (): Promise<void> => {
    if (flying || pending === null) return;
    const { value } = pending;
    pending = null;
    flying = true;
    try {
      const error = await write(value);
      if (error) opts.onError(error);
      else opts.onSaved(value);
    } finally {
      flying = false;
    }
    // Uçuştayken gelen değer: zamanlayıcı bitmişse hemen yazılır.
    if (pending !== null && timer === null) await pump();
  };

  const flush = () => {
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
    }
    void pump();
  };

  return {
    schedule(value) {
      pending = { value };
      if (timer !== null) clearTimeout(timer);
      timer = setTimeout(() => {
        timer = null;
        void pump();
      }, opts.debounceMs);
    },
    flush,
  };
}
```

Create `app/ui/settings/prefs.tsx`:

```tsx
// Genel tercihler (spec §5 `AppPrefs`): açılışta okunur (`boot`), ayarlar penceresi yazar.
// Yazımlar sıralıdır; başarıdan sonra yeni tercihler geçerli olur.
import { createContext, useCallback, useContext, useMemo, useRef, useState, type ReactNode } from "react";
import { api } from "../api";
import type { AppPrefs, CommandError } from "../bindings";

export interface Prefs {
  /** `null`: açılışta okunamadı; varsayılanla ilgili eylemler kapalıdır. */
  prefs: AppPrefs | null;
  save(next: AppPrefs): Promise<CommandError | null>;
}

const PrefsContext = createContext<Prefs | null>(null);

export function PrefsProvider({ initial, children }: { initial: AppPrefs | null; children: ReactNode }) {
  const [prefs, setPrefs] = useState(initial);
  const queue = useRef<Promise<unknown>>(Promise.resolve());
  const save = useCallback((next: AppPrefs) => {
    const run = queue.current.then(async () => {
      const r = await api.setAppPrefs(next);
      if (r.status === "error") return r.error;
      setPrefs(next);
      return null;
    });
    queue.current = run;
    return run;
  }, []);
  const value = useMemo(() => ({ prefs, save }), [prefs, save]);
  return <PrefsContext.Provider value={value}>{children}</PrefsContext.Provider>;
}

export function usePrefs(): Prefs {
  const p = useContext(PrefsContext);
  if (p === null) throw new Error("usePrefs must be used inside <PrefsProvider>");
  return p;
}
```

Create `app/ui/settings/SettingsDialog.tsx`:

```tsx
// Hesap ayarları penceresi (spec §4 "Ayar katmanları"). Açık harita varsa onun ayarı düzenlenir
// ve harita Rust'ta yeniden hesaplanır (`save_calc_settings`); yoksa yeni haritaların genel
// varsayılanı (`AppPrefs.calc_settings`). Kayıt sıralı ve debounce'ludur, kapanırken ve sayfa
// gizlenirken boşaltılır (C5, C17). Varsayılandan farklı alanlar metinle işaretlenir (C9).
import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CalcSettings, CommandError } from "../bindings";
import { DEFAULT_LANGUAGE, isLanguage } from "../i18n";
import { errorMessage } from "../i18n/errors";
import { useSession } from "../session/store";
import { Dialog } from "../shell/Dialog";
import { useNotices } from "../shell/notices";
import { SAVE_DEBOUNCE_MS } from "../view/store";
import {
  ayanamsaKey,
  DASHA_YEARS,
  HOUSE_SYSTEMS,
  NODES,
  ORB_KEYS,
  parseOrb,
  PRESET_AYANAMSAS,
  sameValue,
  SUNRISE_DEFS,
  ZODIACS,
  type OrbKey,
  type SettingsField,
} from "./model";
import { usePrefs } from "./prefs";
import { createSaver } from "./saves";
import "./settings.css";

type Status = "idle" | "saving" | "saved" | "madeDefault";

function orbTexts(s: CalcSettings | null): Record<OrbKey, string> {
  return Object.fromEntries(ORB_KEYS.map((k) => [k, s ? String(s.combustion_orbs[k] ?? "") : ""])) as Record<OrbKey, string>;
}

export function SettingsDialog({ onClose, debounceMs = SAVE_DEBOUNCE_MS }: { onClose(): void; debounceMs?: number }) {
  const { t, i18n } = useTranslation();
  const { opened, updateSettings } = useSession();
  const { prefs, save } = usePrefs();
  const { push } = useNotices();
  const id = useId();
  const [chartId] = useState(() => opened?.chart_record.id ?? null);
  const [draft, setDraft] = useState<CalcSettings | null>(() => opened?.calc_settings ?? prefs?.calc_settings ?? null);
  const [orbs, setOrbs] = useState(() => orbTexts(draft));
  const [error, setError] = useState<CommandError | null>(null);
  const [status, setStatus] = useState<Status>("idle");
  const mounted = useRef(true);
  const latest = useRef({ prefs, language: i18n.language, updateSettings, push, save });
  latest.current = { prefs, language: i18n.language, updateSettings, push, save };
  const locale = () => (isLanguage(latest.current.language) ? latest.current.language : DEFAULT_LANGUAGE);

  const [saver] = useState(() =>
    createSaver<CalcSettings>(
      async (s) => {
        if (chartId !== null) {
          const r = await api.saveCalcSettings(chartId, s);
          if (r.status === "error") return r.error;
          latest.current.updateSettings(chartId, s, r.data);
          return null;
        }
        const base = latest.current.prefs;
        return base ? latest.current.save({ ...base, locale: locale(), calc_settings: s }) : null;
      },
      {
        debounceMs,
        onError: (e) => {
          if (mounted.current) {
            setError(e);
            setStatus("idle");
          } else latest.current.push({ id: "settings", type: "error", messageKey: "notice.settingsSaveFailed", error: e });
        },
        onSaved: () => {
          if (!mounted.current) return;
          setError(null);
          setStatus("saved");
        },
      },
    ),
  );

  useEffect(() => {
    mounted.current = true;
    const hidden = () => document.visibilityState === "hidden" && saver.flush();
    const pagehide = () => saver.flush();
    document.addEventListener("visibilitychange", hidden);
    window.addEventListener("pagehide", pagehide);
    return () => {
      mounted.current = false;
      document.removeEventListener("visibilitychange", hidden);
      window.removeEventListener("pagehide", pagehide);
      saver.flush();
    };
  }, [saver]);

  const change = (next: CalcSettings) => {
    setDraft(next);
    setStatus("saving");
    saver.schedule(next);
  };

  const defaults = prefs?.calc_settings ?? null;
  const differs = (f: SettingsField) => chartId !== null && defaults !== null && draft !== null && !sameValue(draft[f], defaults[f]);
  const anyDiff = draft !== null && defaults !== null && chartId !== null && !sameValue({ ...draft, schema_version: 0 }, { ...defaults, schema_version: 0 });
  const mark = (f: SettingsField) =>
    differs(f) && (
      <span className="settings-differs">
        <span aria-hidden="true">≠ </span>
        {t("settings.differs")}
      </span>
    );

  const row = (f: SettingsField, control: ReactNode) => (
    <div className={`settings-row${differs(f) ? " settings-row--differs" : ""}`}>
      <label htmlFor={`${id}-${f}`}>{t(`settings.field.${f}`)}</label>
      {control}
      {mark(f)}
    </div>
  );

  const resetToDefault = () => {
    if (!defaults) return;
    setOrbs(orbTexts(defaults));
    change(defaults);
    saver.flush();
  };

  const makeDefault = async () => {
    if (!prefs || !draft) return;
    saver.flush();
    const e = await save({ ...prefs, locale: locale(), calc_settings: draft });
    if (!mounted.current) return;
    if (e) setError(e);
    else {
      setError(null);
      setStatus("madeDefault");
    }
  };

  const editOrb = (k: OrbKey, text: string) => {
    const next = { ...orbs, [k]: text };
    setOrbs(next);
    if (!draft) return;
    const values = ORB_KEYS.map((key) => parseOrb(next[key]));
    if (values.some((v) => v === null)) return;
    change({ ...draft, combustion_orbs: Object.fromEntries(ORB_KEYS.map((key, i) => [key, values[i]])) as CalcSettings["combustion_orbs"] });
  };

  const label = opened ? opened.chart_record.label || t(`chart.kind.${opened.chart_record.kind}`) : "";
  return (
    <Dialog title={t("settings.title")} onClose={onClose}>
      <p className="settings-scope">{chartId !== null && opened ? t("settings.chartScope", { name: opened.client.name, chart: label }) : t("settings.defaultsScope")}</p>
      {draft === null ? (
        <p>{t("settings.noPrefs")}</p>
      ) : (
        <div className="settings-form">
          {row(
            "zodiac",
            <select id={`${id}-zodiac`} value={draft.zodiac} data-autofocus onChange={(e) => change({ ...draft, zodiac: e.target.value as CalcSettings["zodiac"] })}>
              {ZODIACS.map((z) => (
                <option key={z} value={z}>
                  {t(`settings.zodiac.${z}`)}
                </option>
              ))}
            </select>,
          )}
          {row(
            "ayanamsa",
            ayanamsaKey(draft.ayanamsa) === "Custom" ? (
              <>
                <select id={`${id}-ayanamsa`} value="Custom" disabled aria-describedby={`${id}-custom`}>
                  <option value="Custom">{t("settings.ayanamsa.Custom")}</option>
                </select>
                <span id={`${id}-custom`} className="settings-hint">
                  {t("settings.customLocked")}
                </span>
              </>
            ) : (
              <select id={`${id}-ayanamsa`} value={ayanamsaKey(draft.ayanamsa)} onChange={(e) => change({ ...draft, ayanamsa: e.target.value as CalcSettings["ayanamsa"] })}>
                {PRESET_AYANAMSAS.map((a) => (
                  <option key={a} value={a}>
                    {t(`settings.ayanamsa.${a}`)}
                  </option>
                ))}
              </select>
            ),
          )}
          {row(
            "node",
            <select id={`${id}-node`} value={draft.node} onChange={(e) => change({ ...draft, node: e.target.value as CalcSettings["node"] })}>
              {NODES.map((n) => (
                <option key={n} value={n}>
                  {t(`settings.node.${n}`)}
                </option>
              ))}
            </select>,
          )}
          {row(
            "house_system",
            <select id={`${id}-house_system`} value={draft.house_system} onChange={(e) => change({ ...draft, house_system: e.target.value as CalcSettings["house_system"] })}>
              {HOUSE_SYSTEMS.map((h) => (
                <option key={h} value={h}>
                  {t(`settings.house.${h}`)}
                </option>
              ))}
            </select>,
          )}
          {row(
            "dasha_year",
            <select id={`${id}-dasha_year`} value={String(draft.dasha_year)} onChange={(e) => change({ ...draft, dasha_year: Number(e.target.value) })}>
              {[...new Set<number>([...DASHA_YEARS, draft.dasha_year ?? 365.25])].map((d) => (
                <option key={d} value={String(d)}>
                  {t("settings.dashaYearOption", { days: d })}
                </option>
              ))}
            </select>,
          )}
          {row(
            "sunrise_def",
            <select id={`${id}-sunrise_def`} value={draft.sunrise_def} onChange={(e) => change({ ...draft, sunrise_def: e.target.value as CalcSettings["sunrise_def"] })}>
              {SUNRISE_DEFS.map((s) => (
                <option key={s} value={s}>
                  {t(`settings.sunrise.${s}`)}
                </option>
              ))}
            </select>,
          )}
          <fieldset className={`settings-orbs${differs("combustion_orbs") ? " settings-row--differs" : ""}`}>
            <legend>
              {t("settings.field.combustion_orbs")} {mark("combustion_orbs")}
            </legend>
            {ORB_KEYS.map((k) => {
              const invalid = parseOrb(orbs[k]) === null;
              return (
                <div key={k} className="settings-orb">
                  <label htmlFor={`${id}-orb-${k}`}>{t(`settings.orb.${k}`)}</label>
                  <input
                    id={`${id}-orb-${k}`}
                    inputMode="decimal"
                    value={orbs[k]}
                    aria-invalid={invalid || undefined}
                    aria-describedby={invalid ? `${id}-orb-${k}-error` : undefined}
                    onChange={(e) => editOrb(k, e.target.value)}
                  />
                  {invalid && (
                    <span id={`${id}-orb-${k}-error`} className="field-error">
                      {t("settings.orbInvalid")}
                    </span>
                  )}
                </div>
              );
            })}
          </fieldset>
        </div>
      )}
      {error && (
        <p className="field-error" role="alert">
          {errorMessage(t, i18n, error)}
        </p>
      )}
      <p role="status" className="settings-status">
        {status === "idle" ? "" : t(`settings.status.${status}`)}
      </p>
      <div className="dialog-actions">
        {chartId !== null && (
          <>
            <button type="button" disabled={!anyDiff} onClick={resetToDefault}>
              {t("settings.resetToDefault")}
            </button>
            <button type="button" disabled={!anyDiff || prefs === null} onClick={() => void makeDefault()}>
              {t("settings.makeDefault")}
            </button>
          </>
        )}
        <button type="button" onClick={onClose}>
          {t("settings.close")}
        </button>
      </div>
    </Dialog>
  );
}
```

Create `app/ui/settings/settings.css`:

```css
.settings-form {
  display: grid;
  gap: 8px;
  min-width: 420px;
}

.settings-row {
  display: grid;
  grid-template-columns: 10rem 1fr auto;
  align-items: center;
  gap: 8px;
}

.settings-row--differs {
  border-left: 2px solid var(--ink);
  padding-left: 6px;
}

.settings-differs,
.settings-hint,
.settings-scope,
.settings-status {
  color: var(--text-2);
  font-size: 12px;
}

.settings-orbs {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 6px 12px;
  border: 1px solid var(--line);
  padding: 8px;
}

.settings-orb {
  display: grid;
  gap: 2px;
}

.settings-orb input {
  width: 5rem;
  font-variant-numeric: tabular-nums;
}
```

`app/ui/session/store.tsx`: `Session` arayüzüne `touch()`'un altına:

```ts
  /** Ayar kaydı yeniden hesapladı (`save_calc_settings` dönüşü): açık harita bu haritaysa ayar ve harita değişir. */
  updateSettings(chartId: number, settings: CalcSettings, chart: Chart): void;
```

tip içe aktarımına `CalcSettings`, `Chart` ekle; `setTags`'in altına:

```tsx
  const updateSettings = useCallback((chartId: number, settings: CalcSettings, chart: Chart) => {
    setOpened((o) => (o && o.chart_record.id === chartId ? { ...o, calc_settings: settings, chart } : o));
  }, []);
```

ve `value` `useMemo`'suna `updateSettings`'i (nesneye ve bağımlılık listesine) ekle.

`app/ui/boot.ts`'i şununla değiştir:

```ts
// Açılış: kayıtlı düzen, tercihler ve son açılan harita tek seferde okunur (spec §4: uygulama
// kapanıp açılınca oturum geri yüklenir). Hiçbir hata açılışı engellemez; hatalar ve tercih
// uyarıları bildirim bandında gösterilir.
import { api } from "./api";
import type { CommandError, OpenedClient, PrefsLoad } from "./bindings";
import { loadLayout } from "./view/persistence";

export interface Boot {
  layout: unknown | null;
  /** Kayıtlı düzen okunamadı: hazır düzenler gösterilir, görünüm kayıtları tutulur. */
  layoutError: CommandError | null;
  /** `get_app_prefs`; okunamadıysa `null` ve `prefsError`. */
  prefs: PrefsLoad | null;
  prefsError: CommandError | null;
  opened: OpenedClient | null;
  error: CommandError | null;
}

export async function boot(): Promise<Boot> {
  const [loaded, prefs, session] = await Promise.all([loadLayout(), api.getAppPrefs(), api.restoreSession()]);
  const base = {
    layout: loaded.layout,
    layoutError: loaded.error,
    prefs: prefs.status === "ok" ? prefs.data : null,
    prefsError: prefs.status === "error" ? prefs.error : null,
  };
  if (session.status === "error") return { ...base, opened: null, error: session.error };
  return { ...base, opened: session.data, error: null };
}

export const EMPTY_BOOT: Boot = { layout: null, layoutError: null, prefs: null, prefsError: null, opened: null, error: null };
```

`app/ui/App.tsx`: `import { PrefsProvider } from "./settings/prefs";` ekle; `App` gövdesini şununla değiştir:

```tsx
export function App({ boot = EMPTY_BOOT, persistence }: { boot?: Boot; persistence?: ViewPersistence }) {
  const [notices] = useState(() => bootNotices(boot));
  return (
    <NoticeProvider initial={notices}>
      <PrefsProvider initial={boot.prefs?.prefs ?? null}>
        <Stores boot={boot} persistence={persistence}>
          <HighlightProvider>
            <Shell />
          </HighlightProvider>
        </Stores>
      </PrefsProvider>
    </NoticeProvider>
  );
}
```

`app/ui/test/render.tsx`: `RenderOptions`'a `prefs?: AppPrefs | null;` ekle (yorum: `/** Varsayılan: fikstürün tercihleri. */`), tip içe aktarımına `AppPrefs`, `import { PrefsProvider } from "../settings/prefs";`; imzayı `{ opened = fixture().opened, layout = null, debounceMs = 0, prefs = fixture().prefs.prefs }` yap ve `NoticeProvider`'ın hemen içine `<PrefsProvider initial={prefs}>…</PrefsProvider>` sar.

`app/ui/App.test.tsx` `renderApp` içindeki `Boot` nesnesi açıkça yazılıdır; yeni alanları ekle: `const full: Boot = { layout: null, layoutError: null, prefs: null, prefsError: null, opened: fixture().opened, error: null, ...boot };` (eklenmezse `pnpm --dir app build` içindeki `tsc` düşer).

`app/ui/shortcuts.ts`: `Command` birliğine `| { id: "settings" }`; `ShortcutInfo.id`'e `"settings"`; `SHORTCUT_LIST`'te `editChart` satırının altına `{ id: "settings", keys: "⌘," },`; `matchShortcut` içinde `if (e.code === "Backslash" …` satırının altına:

```ts
    if (e.code === "Comma" || e.key === ",") return { id: "settings" };
```

`app/ui/palette/items.ts`: `export type PaletteCommand = "newChart" | "addChart" | "editChart" | "settings" | "toggleNotes" | "sidebar" | "help";`

`app/ui/palette/Palette.tsx` `commands` dizisinde `help` satırından önce:

```ts
      { command: "settings", label: t("palette.command.settings") },
```

`app/ui/shell/Shell.tsx`: `import { SettingsDialog } from "../settings/SettingsDialog";`; durumlara `const [settings, setSettings] = useState(false);`; kısayol dinleyicisinde `palette` kolunun altına:

```ts
      } else if (command?.id === "settings") {
        e.preventDefault();
        setSettings(true);
```

`runPalette` `case "command":` içinde `toggleNotes` satırından önce:

```ts
        if (item.command === "settings") return setSettings(true);
```

`<TopBar … />`'a `onSettings={() => setSettings(true)}`; `{palette && …}` satırının altına:

```tsx
      {settings && <SettingsDialog onClose={() => setSettings(false)} />}
```

`app/ui/shell/TopBar.tsx`: `TopBarProps`'a `onSettings(): void;`, yapı bozma listesine `onSettings`; `onPalette` düğmesinin altına:

```tsx
        <button type="button" aria-keyshortcuts="Meta+Comma" onClick={onSettings}>
          {t("settings.open")}
        </button>
```

i18n — `app/ui/i18n/tr.ts` (üst düzey, `data`'nın altına):

```ts
  settings: {
    open: "Ayarlar",
    title: "Hesap ayarları",
    chartScope: "{{name}} · {{chart}} haritasının ayarları. Değişiklik haritayı yeniden hesaplar.",
    defaultsScope: "Yeni haritalar için varsayılan ayarlar.",
    noPrefs: "Tercihler okunamadı; varsayılanlar düzenlenemiyor.",
    field: {
      zodiac: "Zodyak",
      ayanamsa: "Ayanamsa",
      node: "Düğüm",
      house_system: "Ev sistemi",
      dasha_year: "Dasa yılı",
      combustion_orbs: "Yanma orbları (derece)",
      sunrise_def: "Gün doğumu",
    },
    zodiac: { Sidereal: "Sidereal", TropicalRasiSiderealNakshatra: "Tropik rasi, sidereal nakshatra" },
    ayanamsa: { Lahiri: "Lahiri", Raman: "Raman", Kp: "KP (Krishnamurti)", TrueChitra: "True Chitrapaksha", Custom: "Özel" },
    customLocked: "İçe aktarılmış özel ayanamsa burada değiştirilemez.",
    node: { Mean: "Ortalama", True: "Gerçek" },
    house: { WholeSign: "Tam burç", Sripati: "Sripati", Placidus: "Placidus" },
    dashaYearOption: "{{days}} gün",
    sunrise: { DiskCenterNoRefraction: "Disk merkezi, kırılmasız", UpperLimbRefracted: "Üst kenar, kırılmalı" },
    orb: { moon: "Ay", mars: "Mars", mercury: "Merkür", mercury_retro: "Merkür (retro)", jupiter: "Jüpiter", venus: "Venüs", venus_retro: "Venüs (retro)", saturn: "Satürn" },
    orbInvalid: "0 ile 30 arasında bir sayı girin.",
    differs: "varsayılandan farklı",
    resetToDefault: "Varsayılana döndür",
    makeDefault: "Bunu varsayılan yap",
    close: "Kapat",
    status: { saving: "Kaydediliyor…", saved: "Kaydedildi.", madeDefault: "Yeni haritaların varsayılanı olarak kaydedildi." },
  },
```

`shortcuts` içine, **`editChart` satırının hemen altına** `settings: "Ayarlar",` (`shortcuts.test.ts` `tr.shortcuts` anahtar sırasını `SHORTCUT_LIST` sırasıyla karşılaştırır; sona eklenirse test düşer; `en.ts` ve `fr.ts`'te de aynı yere); `palette.command` içine `settings: "Ayarlar",`; `notice` içine:

```ts
    settingsSaveFailed: "Ayarlar kaydedilemedi: {{message}}",
    prefsLoadFailed: "Tercihler okunamadı; yerleşik varsayılanlar kullanılıyor: {{message}}",
```

`app/ui/i18n/en.ts` (aynı anahtarlar):

```ts
  settings: {
    open: "Settings",
    title: "Calculation settings",
    chartScope: "Settings of the {{name}} · {{chart}} chart. A change recalculates the chart.",
    defaultsScope: "Default settings for new charts.",
    noPrefs: "Preferences could not be read; defaults cannot be edited.",
    field: {
      zodiac: "Zodiac",
      ayanamsa: "Ayanamsa",
      node: "Node",
      house_system: "House system",
      dasha_year: "Dasha year",
      combustion_orbs: "Combustion orbs (degrees)",
      sunrise_def: "Sunrise",
    },
    zodiac: { Sidereal: "Sidereal", TropicalRasiSiderealNakshatra: "Tropical rasi, sidereal nakshatra" },
    ayanamsa: { Lahiri: "Lahiri", Raman: "Raman", Kp: "KP (Krishnamurti)", TrueChitra: "True Chitrapaksha", Custom: "Custom" },
    customLocked: "An imported custom ayanamsa cannot be changed here.",
    node: { Mean: "Mean", True: "True" },
    house: { WholeSign: "Whole sign", Sripati: "Sripati", Placidus: "Placidus" },
    dashaYearOption: "{{days}} days",
    sunrise: { DiskCenterNoRefraction: "Disc centre, no refraction", UpperLimbRefracted: "Upper limb, refracted" },
    orb: { moon: "Moon", mars: "Mars", mercury: "Mercury", mercury_retro: "Mercury (retro)", jupiter: "Jupiter", venus: "Venus", venus_retro: "Venus (retro)", saturn: "Saturn" },
    orbInvalid: "Enter a number from 0 to 30.",
    differs: "differs from default",
    resetToDefault: "Reset to default",
    makeDefault: "Make this the default",
    close: "Close",
    status: { saving: "Saving…", saved: "Saved.", madeDefault: "Saved as the default for new charts." },
  },
```

`shortcuts.settings: "Settings"`, `palette.command.settings: "Settings"`, `notice.settingsSaveFailed: "Settings could not be saved: {{message}}"`, `notice.prefsLoadFailed: "Preferences could not be read; built-in defaults are in use: {{message}}"`.

`app/ui/i18n/fr.ts`:

```ts
  settings: {
    open: "Réglages",
    title: "Réglages de calcul",
    chartScope: "Réglages de la carte {{name}} · {{chart}}. Une modification recalcule la carte.",
    defaultsScope: "Réglages par défaut des nouvelles cartes.",
    noPrefs: "Les préférences n’ont pas pu être lues ; les valeurs par défaut ne sont pas modifiables.",
    field: {
      zodiac: "Zodiaque",
      ayanamsa: "Ayanamsa",
      node: "Nœud",
      house_system: "Système de maisons",
      dasha_year: "Année de dasha",
      combustion_orbs: "Orbes de combustion (degrés)",
      sunrise_def: "Lever du soleil",
    },
    zodiac: { Sidereal: "Sidéral", TropicalRasiSiderealNakshatra: "Rasi tropical, nakshatra sidéral" },
    ayanamsa: { Lahiri: "Lahiri", Raman: "Raman", Kp: "KP (Krishnamurti)", TrueChitra: "True Chitrapaksha", Custom: "Personnalisé" },
    customLocked: "Un ayanamsa personnalisé importé ne peut pas être modifié ici.",
    node: { Mean: "Moyen", True: "Vrai" },
    house: { WholeSign: "Signe entier", Sripati: "Sripati", Placidus: "Placidus" },
    dashaYearOption: "{{days}} jours",
    sunrise: { DiskCenterNoRefraction: "Centre du disque, sans réfraction", UpperLimbRefracted: "Bord supérieur, avec réfraction" },
    orb: { moon: "Lune", mars: "Mars", mercury: "Mercure", mercury_retro: "Mercure (rétro)", jupiter: "Jupiter", venus: "Vénus", venus_retro: "Vénus (rétro)", saturn: "Saturne" },
    orbInvalid: "Saisissez un nombre entre 0 et 30.",
    differs: "différent du défaut",
    resetToDefault: "Revenir au défaut",
    makeDefault: "En faire le défaut",
    close: "Fermer",
    status: { saving: "Enregistrement…", saved: "Enregistré.", madeDefault: "Enregistré comme défaut des nouvelles cartes." },
  },
```

`shortcuts.settings: "Réglages"`, `palette.command.settings: "Réglages"`, `notice.settingsSaveFailed: "Les réglages n’ont pas pu être enregistrés : {{message}}"`, `notice.prefsLoadFailed: "Les préférences n’ont pas pu être lues ; les valeurs intégrées sont utilisées : {{message}}"`.

(`en.ts` ve `fr.ts` `tr`'nin tipine uyacak biçimde yazılıysa — ör. `satisfies typeof tr` — eksik anahtar `tsc`'de görünür; öyle değilse `i18n.test.ts`'in anahtar eşitliği testi gösterir.)

`app/e2e/ipc.ts` `respond` nesnesine (`load_panel_layout`'un yanına). Açılış artık tercihleri okur; taklit olmazsa e2e ve görsel testlerde "tercihler okunamadı" bildirimi çıkar ve görsel temeller bozulur:

```ts
        get_app_prefs: () => fixture.prefs,
        set_app_prefs: () => null,
        // Yeniden hesap taklit edilmez: aynı harita, yeni ayarların kopyasıyla döner.
        save_calc_settings: (args) => ({ ...fixture.opened.chart, meta: { ...fixture.opened.chart.meta, settings: args.settings } }),
```

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/settings ui/shortcuts.test.ts ui/session ui/App.test.tsx ui/palette
pnpm --dir app test && pnpm --dir app build
pnpm --dir app e2e && pnpm --dir app e2e:visual
rm -rf app/test-results
```
Expected: `settings/model.test.ts` 4, `settings/SettingsDialog.test.tsx` 6, shortcuts +1 geçer; toplam `Test Files  45 passed (45)`, `Tests  375 passed (375)`; build `✓ built`; e2e `6 passed`, görsel `6 passed` (bildirim bandı boş; üst çubuğa "Ayarlar" düğmesi eklendiği için üst çubuğu içeren bir temel fark gösterirse farkın yalnızca bu düğme olduğunu `test-results` içindeki fark görüntüsünde doğrula ve temeli `pnpm --dir app e2e:visual --update-snapshots` ile bu commit'te yenile).

- [ ] **Step 5: Commit**

```bash
git add app/ui app/e2e
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): settings dialog with ordered autosave, app prefs store and ⌘, shortcut

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Arayüz: fark rozeti, tzdb kabul bandı ve tercih uyarıları

Fark rozeti (D12): açık haritanın ayarı genel varsayılandan farklıysa üst çubukta, uyarı rozetlerinin yanında `≠` işaretli bir düğme çıkar; ipucu ve gizli metin farklı alanları sayar, tıklama ayarlar penceresini açar. tzdb bandı (D5): `TzdbChanged` bildirimi iki eylem taşır — "Yeni zamanı kabul et" (`update_chart`, aynı doğum verisi) ve "Saklanan zamanı koru" (`keep_stored_moment`); ikisi de görünüm kaydını önce boşaltır (C5) ve dönen haritayı açar. Tercih okuma uyarıları (Task 1) açılış bandında gösterilir.

**Files:**
- Modify: `app/ui/shell/ChartTitle.tsx`, `app/ui/shell/TopBar.tsx`, `app/ui/shell/shell.css`
- Create: `app/ui/shell/ChartTitle.test.tsx`
- Modify: `app/ui/shell/notices.tsx`
- Create: `app/ui/shell/TzdbActions.tsx`, `app/ui/shell/TzdbActions.test.tsx`
- Modify: `app/ui/App.tsx`, `app/ui/App.test.tsx`
- Modify: `app/ui/i18n/{tr,en,fr}.ts`

**Interfaces:**
- Consumes: `settingsDiff`, `usePrefs` (Task 5); `api.updateChart`, `api.keepStoredMoment` (Task 4); `Session.open`; `ViewStore.flush`.
- Produces:
  - `ChartTitle({ onSettings?: () => void })`; `TopBar` `onSettings` prop'u `ChartTitle`'a iletilir
  - `Notice` uyarı/bilgi varyantı: `action?: "tzdb"`; `storeWarningNotices` `TzdbChanged`'e `action: "tzdb"` ve `params.utc` ekler
  - `TzdbActions()` bileşeni
  - `prefsNotices(boot: Boot): Notice[]` (`App.tsx`, dışa aktarılır)
  - i18n: `settingsBadge.{label,hint}`, `tzdb.{accept,keep}`, `notice.tzdbFailed`, `prefsWarning.{Unreadable,CalcSettingsReset,TooNew}`; `storeWarning.TzdbChanged` metni `{{utc}}` alır

- [ ] **Step 1: Başarısız testleri yaz**

Create `app/ui/shell/ChartTitle.test.tsx`:

```tsx
import { fireEvent, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { ChartTitle } from "./ChartTitle";

test("no difference badge when the chart uses the default settings", () => {
  renderWithStores(<ChartTitle onSettings={() => {}} />);
  expect(screen.queryByRole("button", { name: /Ayar farkı/ })).toBeNull();
});

test("the difference badge names the differing fields and opens the settings", () => {
  const opened = fixture().opened;
  opened.calc_settings = { ...opened.calc_settings, node: "True", ayanamsa: "Raman" };
  const onSettings = vi.fn();
  renderWithStores(<ChartTitle onSettings={onSettings} />, { opened });
  const badge = screen.getByRole("button", { name: /Ayar farkı/ });
  expect(badge).toHaveAttribute("title", "Bu haritanın ayarı varsayılandan farklı: Ayanamsa, Düğüm");
  expect(badge).toHaveTextContent("≠");
  fireEvent.click(badge);
  expect(onSettings).toHaveBeenCalledTimes(1);
});
```

Create `app/ui/shell/TzdbActions.test.tsx`:

```tsx
import { fireEvent, screen, waitFor } from "@testing-library/react";
import { useEffect } from "react";
import { afterEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    updateChart: vi.fn(),
    keepStoredMoment: vi.fn(),
  },
}));

import { api } from "../api";
import type { OpenedClient } from "../bindings";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { NoticeBar, storeWarningNotices, useNotices } from "./notices";

afterEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
});

function tzdbOpened(): OpenedClient {
  const o = fixture().opened;
  o.warnings = [{ TzdbChanged: { stored_version: "2024a", current_version: "2026a", current_utc: "2000-01-01T12:00:00Z" } }];
  return o;
}

function Seed({ opened }: { opened: OpenedClient }) {
  const { push } = useNotices();
  useEffect(() => {
    for (const n of storeWarningNotices(opened.warnings)) push(n);
  }, [opened, push]);
  return <NoticeBar />;
}

test("accept re-saves the same birth data and reopens the chart without the band", async () => {
  const o = tzdbOpened();
  vi.mocked(api.updateChart).mockResolvedValue({ status: "ok", data: fixture().opened });
  renderWithStores(<Seed opened={o} />, { opened: o });
  expect(screen.getByRole("status")).toHaveTextContent("2024a → 2026a");
  fireEvent.click(screen.getByRole("button", { name: "Yeni zamanı kabul et" }));
  await waitFor(() => expect(api.updateChart).toHaveBeenCalledTimes(1));
  const [chartId, chart] = vi.mocked(api.updateChart).mock.calls[0];
  expect(chartId).toBe(o.chart_record.id);
  expect(chart).toEqual({ label: o.chart_record.label, kind: o.chart_record.kind, birth: o.chart_record.birth });
  await waitFor(() => expect(screen.queryByRole("button", { name: "Yeni zamanı kabul et" })).toBeNull());
});

test("keep stamps the stored moment and closes the band", async () => {
  const o = tzdbOpened();
  vi.mocked(api.keepStoredMoment).mockResolvedValue({ status: "ok", data: fixture().opened });
  renderWithStores(<Seed opened={o} />, { opened: o });
  fireEvent.click(screen.getByRole("button", { name: "Saklanan zamanı koru" }));
  await waitFor(() => expect(api.keepStoredMoment).toHaveBeenCalledWith(o.chart_record.id));
  await waitFor(() => expect(screen.queryByRole("button", { name: "Saklanan zamanı koru" })).toBeNull());
  expect(api.updateChart).not.toHaveBeenCalled();
});

test("a failed decision keeps the band and reports the error", async () => {
  const o = tzdbOpened();
  vi.mocked(api.keepStoredMoment).mockResolvedValue({ status: "error", error: { code: "store_sqlite", params: { detail: "busy" } } });
  renderWithStores(<Seed opened={o} />, { opened: o });
  fireEvent.click(screen.getByRole("button", { name: "Saklanan zamanı koru" }));
  await waitFor(() => expect(screen.getByRole("alert")).toHaveTextContent("Saat dilimi kararı kaydedilemedi"));
  expect(screen.getByRole("button", { name: "Saklanan zamanı koru" })).toBeInTheDocument();
});
```

`app/ui/App.test.tsx` sonuna (dosyanın `renderApp` yardımcısıyla; bellek içi kalıcılık kullanır):

```tsx
test("preference warnings from boot are shown in the notice bar", () => {
  renderApp({ opened: null, prefs: { ...fixture().prefs, warnings: [{ TooNew: { found: 2 } }] } });
  expect(screen.getByText(/Tercihler daha yeni bir uygulama sürümüyle kaydedilmiş \(sürüm 2\)/)).toBeInTheDocument();
});
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/shell/ChartTitle.test.tsx ui/shell/TzdbActions.test.tsx ui/App.test.tsx
```
Expected: rozet bulunamaz (`Unable to find role="button" and name /Ayar farkı/`); "Yeni zamanı kabul et" düğmesi yok; tercih uyarısı metni yok.

- [ ] **Step 3: Uygula**

`app/ui/shell/ChartTitle.tsx`'i şununla değiştir:

```tsx
// Üst çubuğun solu (UI spec §4): danışan adı, harita etiketi, uyarı rozetleri (UI spec §5) ve
// fark rozeti (Plan 11 D12). Rozet amber kenarlı ve işaretlidir ama anlamı metinle taşır;
// açıklama ipucu olarak (`title`) ve ekran okuyucu için gizli metin olarak verilir.
import { useTranslation } from "react-i18next";
import { useSession } from "../session/store";
import { settingsDiff } from "../settings/model";
import { usePrefs } from "../settings/prefs";

export function ChartTitle({ onSettings }: { onSettings?: () => void }) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { prefs } = usePrefs();
  if (!opened) return <h1 className="topbar-title">{t("app.title")}</h1>;
  const label = opened.chart_record.label || t(`chart.kind.${opened.chart_record.kind}`);
  const warnings = [...opened.chart_record.resolved.warnings, ...opened.chart.meta.warnings];
  const diff = prefs ? settingsDiff(opened.calc_settings, prefs.calc_settings) : [];
  const diffHint = t("settingsBadge.hint", { fields: diff.map((f) => t(`settings.field.${f}`)).join(", ") });
  return (
    <div className="topbar-title">
      <h1 className="topbar-client">{opened.client.name}</h1>
      <span className="topbar-chart">{label}</span>
      {(warnings.length > 0 || diff.length > 0) && (
        <ul className="badges" aria-label={t("shell.warnings")}>
          {warnings.map((w) => (
            <li key={w} className="badge" tabIndex={0} title={t(`badgeHint.${w}`)}>
              <span className="badge-mark" aria-hidden="true">
                ▲
              </span>
              {t(`badge.${w}`)}
              <span className="visually-hidden">: {t(`badgeHint.${w}`)}</span>
            </li>
          ))}
          {diff.length > 0 && (
            <li>
              <button type="button" className="badge badge-button" title={diffHint} onClick={onSettings}>
                <span className="badge-mark" aria-hidden="true">
                  ≠
                </span>
                {t("settingsBadge.label")}
                <span className="visually-hidden">: {diffHint}</span>
              </button>
            </li>
          )}
        </ul>
      )}
    </div>
  );
}
```

`app/ui/shell/TopBar.tsx`: `<ChartTitle />` → `<ChartTitle onSettings={onSettings} />`.

`app/ui/shell/shell.css` sonuna:

```css
.badge-button {
  font: inherit;
  background: none;
  cursor: pointer;
}

.notice-actions {
  display: inline-flex;
  gap: 6px;
}
```

`app/ui/shell/notices.tsx`: `Notice` tipini şununla değiştir:

```ts
export type Notice =
  | { id: string; type: "error"; messageKey: string; error: CommandError }
  | {
      id: string;
      type: "warning" | "info";
      messageKey: string;
      params: Record<string, string>;
      /** Bildirimle gelen kararın eylemleri; bileşen oturumdan okur (veri, işlev değil). */
      action?: "tzdb";
    };
```

`storeWarningNotices` içindeki `TzdbChanged` dönüşünü şununla değiştir:

```ts
    return {
      id: "store:TzdbChanged",
      type: "warning",
      messageKey: "storeWarning.TzdbChanged",
      params: { stored: w.TzdbChanged.stored_version, current: w.TzdbChanged.current_version, utc: w.TzdbChanged.current_utc ?? "—" },
      action: "tzdb",
    };
```

`NoticeBar`'da metin `span`'ının altına (kapatma düğmesinden önce):

```tsx
          {n.type !== "error" && n.action === "tzdb" && <TzdbActions />}
```

ve dosyanın başına `import { TzdbActions } from "./TzdbActions";`.

Create `app/ui/shell/TzdbActions.tsx`:

```tsx
// tzdb farkı kararı (spec §4; Plan 11 D5). Saklanan an kullanılır; kullanıcı yeni tzdb'nin anını
// kabul eder (`update_chart`, aynı doğum verisi) ya da saklananı korur (`keep_stored_moment`).
// İki yol da görünüm kaydını önce boşaltır (C5) ve dönen haritayı açar; an sessizce kaymaz.
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { api, type Result } from "../api";
import type { OpenedClient } from "../bindings";
import { useSession } from "../session/store";
import { useViewStore } from "../view/store";
import { useNotices } from "./notices";

export function TzdbActions() {
  const { t } = useTranslation();
  const session = useSession();
  const { flush } = useViewStore();
  const { push } = useNotices();
  const [busy, setBusy] = useState(false);
  const o = session.opened;
  if (!o) return null;
  const chartId = o.chart_record.id;

  const run = async (call: () => Promise<Result<OpenedClient>>) => {
    if (busy) return;
    setBusy(true);
    flush();
    const r = await call();
    setBusy(false);
    if (r.status === "error") {
      push({ id: "tzdb", type: "error", messageKey: "notice.tzdbFailed", error: r.error });
      return;
    }
    session.open(r.data);
  };

  return (
    <span className="notice-actions">
      <button
        type="button"
        aria-disabled={busy || undefined}
        onClick={() => void run(() => api.updateChart(chartId, { label: o.chart_record.label, kind: o.chart_record.kind, birth: o.chart_record.birth }))}
      >
        {t("tzdb.accept")}
      </button>
      <button type="button" aria-disabled={busy || undefined} onClick={() => void run(() => api.keepStoredMoment(chartId))}>
        {t("tzdb.keep")}
      </button>
    </span>
  );
}
```

`Session.open` `store:*` bildirimlerini kaldırır; dönen haritada uyarı yoksa bant kapanır. `api.ts` `Result` tipini dışa aktarır (`export type Result<T>`).

`app/ui/App.tsx`: `bootNotices`'i şununla değiştir:

```tsx
/** Tercih okuma uyarıları ve hatası (Plan 11 Task 1). */
export function prefsNotices(boot: Boot): Notice[] {
  if (boot.prefsError) return [{ id: "prefs", type: "error", messageKey: "notice.prefsLoadFailed", error: boot.prefsError }];
  return (boot.prefs?.warnings ?? []).map((w): Notice => {
    if (w === "Unreadable") return { id: "prefs:Unreadable", type: "warning", messageKey: "prefsWarning.Unreadable", params: {} };
    if ("TooNew" in w) return { id: "prefs:TooNew", type: "warning", messageKey: "prefsWarning.TooNew", params: { found: String(w.TooNew.found) } };
    return { id: "prefs:CalcSettingsReset", type: "warning", messageKey: "prefsWarning.CalcSettingsReset", params: {} };
  });
}

function bootNotices(boot: Boot): Notice[] {
  const layout: Notice[] = boot.layoutError ? [{ id: "layout", type: "error", messageKey: "notice.layoutLoadFailed", error: boot.layoutError }] : [];
  const prefs = prefsNotices(boot);
  if (boot.error) return [...layout, ...prefs, { id: "boot", type: "error", messageKey: "notice.restoreFailed", error: boot.error }];
  return [...layout, ...prefs, ...(boot.opened ? storeWarningNotices(boot.opened.warnings) : [])];
}
```

i18n — `tr.ts`:

```ts
  settingsBadge: { label: "Ayar farkı", hint: "Bu haritanın ayarı varsayılandan farklı: {{fields}}" },
  tzdb: { accept: "Yeni zamanı kabul et", keep: "Saklanan zamanı koru" },
  prefsWarning: {
    Unreadable: "Tercihler okunamadı; yerleşik varsayılanlar kullanılıyor.",
    CalcSettingsReset: "Yeni haritaların varsayılan ayarı okunamadı; yerleşik varsayılan kullanılıyor.",
    TooNew: "Tercihler daha yeni bir uygulama sürümüyle kaydedilmiş (sürüm {{found}}); yerleşik varsayılanlar kullanılıyor ve kayıt korunuyor.",
  },
```

`notice` içine `tzdbFailed: "Saat dilimi kararı kaydedilemedi: {{message}}",`; `storeWarning.TzdbChanged` metnini şununla değiştir:

```ts
    TzdbChanged: "Saat dilimi veritabanı değişti ({{stored}} → {{current}}) ve bu haritanın anını değiştiriyor (yeni UTC: {{utc}}). Saklanan an kullanılıyor.",
```

`en.ts`:

```ts
  settingsBadge: { label: "Settings differ", hint: "This chart's settings differ from the default: {{fields}}" },
  tzdb: { accept: "Accept the new time", keep: "Keep the stored time" },
  prefsWarning: {
    Unreadable: "Preferences could not be read; built-in defaults are in use.",
    CalcSettingsReset: "The default settings for new charts could not be read; the built-in default is in use.",
    TooNew: "Preferences were saved by a newer version of the app (version {{found}}); built-in defaults are in use and the saved preferences are kept.",
  },
```

`notice.tzdbFailed: "The time zone decision could not be saved: {{message}}"`; `storeWarning.TzdbChanged: "The time zone database changed ({{stored}} → {{current}}) and moves this chart's moment (new UTC: {{utc}}). The stored moment is in use."`

`fr.ts`:

```ts
  settingsBadge: { label: "Réglages différents", hint: "Les réglages de cette carte diffèrent du défaut : {{fields}}" },
  tzdb: { accept: "Accepter la nouvelle heure", keep: "Garder l’heure enregistrée" },
  prefsWarning: {
    Unreadable: "Les préférences n’ont pas pu être lues ; les valeurs intégrées sont utilisées.",
    CalcSettingsReset: "Les réglages par défaut des nouvelles cartes n’ont pas pu être lus ; la valeur intégrée est utilisée.",
    TooNew: "Les préférences ont été enregistrées par une version plus récente de l’application (version {{found}}) ; les valeurs intégrées sont utilisées et l’enregistrement est conservé.",
  },
```

`notice.tzdbFailed: "La décision de fuseau n’a pas pu être enregistrée : {{message}}"`; `storeWarning.TzdbChanged: "La base des fuseaux a changé ({{stored}} → {{current}}) et déplace le moment de cette carte (nouvel UTC : {{utc}}). Le moment enregistré est utilisé."`

Ayar farkı testi Türkçe alan adlarını `settings.field.*`'tan okur ("Ayanamsa, Düğüm", `SETTINGS_FIELDS` sırası).

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/shell ui/App.test.tsx
pnpm --dir app test && pnpm --dir app build
```
Expected: `ChartTitle.test.tsx` 2, `TzdbActions.test.tsx` 3, `App.test.tsx` +1 geçer; toplam `Test Files  48 passed (48)`, `Tests  391 passed (391)`; build `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): settings difference badge, tzdb decision band and prefs warnings

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Arayüz: oturumluk transit anı, transit tablosu ve tarih seçimi

UI spec §3.3 `transit_table` ("tarih, varsayılan: şimdi"). `TransitProvider` açık haritanın transitlerini yalnızca bir tüketici (panel) varken ister; an oturumluktur ve kaydedilmez, yeni harita açılınca "şimdi"ye döner (D8). Panelin `settings.date` alanı okunmaz (Plan 04 düzenlerinde `date: null` durur; ayrıştırıcıya dokunulmaz, C6). Eski ana ait geç yanıt düşürülür. Tarih seçici duvar saatini tarayıcının IANA diliminde gönderir (D7); gösterilen an `Intl.DateTimeFormat` ile biçimlenir (C4: biçimleme hesap değildir).

**Files:**
- Create: `app/ui/transit/store.tsx`, `app/ui/transit/store.test.tsx`
- Create: `app/ui/transit/TransitDate.tsx`
- Create: `app/ui/transit/TransitTablePanel.tsx`, `app/ui/transit/TransitTablePanel.test.tsx`, `app/ui/transit/transit.css`
- Modify: `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx` (kayıt testi)
- Modify: `app/ui/App.tsx`, `app/ui/test/render.tsx`
- Modify: `app/ui/i18n/{tr,en,fr}.ts`

**Interfaces:**
- Consumes: `api.transitsAt` (Task 4); `TransitAt`, `TransitView` (bindings); `useSession`; `usePanelLoadPhase`, `PanelError`; `signIndex`; `formatDegree`; `nakshatraName`; `showsRetro`, `RETRO_MARK`; `timeUnknown`.
- Produces:
  - `TransitProvider({ children })`; `useTransit(): Transit` (tüketici kaydı yapar); `useTransitControl(): Transit` (kayıt yapmaz)
  - `interface Transit { at: TransitAt; view: TransitView | null; error: CommandError | null; loading: boolean; setAt(at: TransitAt): void; retry(): void }`
  - `browserTimeZone(): string`, `localParts(utc: string, zone: string): { date: string; time: string } | null`, `formatMoment(utc: string, language: string, zone: string): string`
  - `TransitDate()`; `TransitTablePanel(props: PanelProps)`
  - i18n: `transit.*`

- [ ] **Step 1: Başarısız testleri yaz**

Create `app/ui/transit/store.test.tsx`:

```tsx
import { act, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn() } }));

import { api } from "../api";
import type { OpenedClient, TransitView } from "../bindings";
import { useSession } from "../session/store";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { browserTimeZone, useTransit, useTransitControl, type Transit } from "./store";

const ok = (data: TransitView) => ({ status: "ok" as const, data });
let control: Transit;
let session: ReturnType<typeof useSession>;

function Consumer() {
  const t = useTransit();
  return <output data-testid="utc">{t.view?.utc ?? ""}</output>;
}
function Control() {
  control = useTransitControl();
  session = useSession();
  return null;
}

beforeEach(() => vi.mocked(api.transitsAt).mockResolvedValue(ok(fixture().transit)));
afterEach(() => vi.mocked(api.transitsAt).mockReset());

test("nothing is requested while no panel needs transits", async () => {
  renderWithStores(<Control />);
  await act(() => Promise.resolve());
  expect(api.transitsAt).not.toHaveBeenCalled();
});

test("a consumer asks once for now, with the open chart's settings, houses and dasha seed", async () => {
  const o = fixture().opened;
  renderWithStores(<Consumer />);
  await waitFor(() => expect(screen.getByTestId("utc")).toHaveTextContent(fixture().transit.utc));
  expect(api.transitsAt).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.transitsAt).mock.calls[0]).toEqual(["Now", o.calc_settings, o.chart.bhavas, o.chart.dasha.seed]);
});

test("a later moment replaces the view; a late reply for the old moment is dropped", async () => {
  let release: (v: ReturnType<typeof ok>) => void = () => {};
  vi.mocked(api.transitsAt).mockReturnValueOnce(new Promise((r) => (release = r)));
  const later = { ...fixture().transit, utc: "2030-01-01T00:00:00Z" };
  vi.mocked(api.transitsAt).mockResolvedValueOnce(ok(later));
  renderWithStores(
    <>
      <Consumer />
      <Control />
    </>,
  );
  act(() => control.setAt({ Local: { date: "2030-01-01", time: "03:00", tz: browserTimeZone() } }));
  await waitFor(() => expect(screen.getByTestId("utc")).toHaveTextContent("2030-01-01T00:00:00Z"));
  await act(async () => release(ok(fixture().transit)));
  expect(screen.getByTestId("utc")).toHaveTextContent("2030-01-01T00:00:00Z");
});

test("opening another chart goes back to now", async () => {
  renderWithStores(
    <>
      <Consumer />
      <Control />
    </>,
  );
  act(() => control.setAt({ Jd: 2_460_000.5 }));
  await waitFor(() => expect(vi.mocked(api.transitsAt).mock.calls.at(-1)?.[0]).toEqual({ Jd: 2_460_000.5 }));
  const other: OpenedClient = fixture().opened;
  other.chart_record = { ...other.chart_record, id: 77 };
  act(() => session.open(other));
  await waitFor(() => expect(vi.mocked(api.transitsAt).mock.calls.at(-1)?.[0]).toBe("Now"));
  expect(control.at).toBe("Now");
});
```

Create `app/ui/transit/TransitTablePanel.test.tsx`:

```tsx
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn() } }));

import { api } from "../api";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { browserTimeZone } from "./store";
import { TransitTablePanel } from "./TransitTablePanel";

const panel: Panel = { id: "tt", kind: "transit_table", x: 0, y: 0, w: 6, h: 8, settings: {} };

beforeEach(() => vi.mocked(api.transitsAt).mockResolvedValue({ status: "ok", data: fixture().transit }));
afterEach(() => vi.mocked(api.transitsAt).mockReset());

test("nine rows: sign, degree, nakshatra, natal house and the retro mark (none on the nodes)", async () => {
  renderWithStores(<TransitTablePanel panel={panel} />);
  const table = await screen.findByRole("table");
  const rows = within(table).getAllByRole("row").slice(1);
  expect(rows).toHaveLength(9);
  const tr = fixture().transit.snapshot.grahas;
  rows.forEach((row, i) => {
    const cells = within(row).getAllByRole("cell");
    expect(cells[3]).toHaveTextContent(String(tr[i].natal_house));
    const nodes = tr[i].graha === "Rahu" || tr[i].graha === "Ketu";
    expect(cells[4]).toHaveTextContent(tr[i].retrograde && !nodes ? "R" : "");
  });
});

test("unknown birth time hides the natal house column values", async () => {
  renderWithStores(<TransitTablePanel panel={panel} />, { opened: unknownTimeOpened() });
  const table = await screen.findByRole("table");
  for (const row of within(table).getAllByRole("row").slice(1)) expect(within(row).getAllByRole("cell")[3]).toHaveTextContent("—");
});

test("apply sends the wall-clock time in the browser's time zone; now goes back to now", async () => {
  renderWithStores(<TransitTablePanel panel={panel} />);
  await screen.findByRole("table");
  fireEvent.change(screen.getByLabelText("Tarih"), { target: { value: "2026-12-01" } });
  fireEvent.change(screen.getByLabelText("Saat"), { target: { value: "09:30" } });
  fireEvent.click(screen.getByRole("button", { name: "Uygula" }));
  await waitFor(() =>
    expect(vi.mocked(api.transitsAt).mock.calls.at(-1)?.[0]).toEqual({ Local: { date: "2026-12-01", time: "09:30", tz: browserTimeZone() } }),
  );
  const now = screen.getByRole("button", { name: "Şimdi" });
  expect(now).toHaveAttribute("aria-pressed", "false");
  fireEvent.click(now);
  await waitFor(() => expect(vi.mocked(api.transitsAt).mock.calls.at(-1)?.[0]).toBe("Now"));
  expect(now).toHaveAttribute("aria-pressed", "true");
});

test("an error is shown in the panel with a retry", async () => {
  vi.mocked(api.transitsAt).mockResolvedValueOnce({ status: "error", error: { code: "nonexistent_local_time", params: { local: "2014-03-31T03:30:00", suggestion: "2014-03-31T04:30:00" } } });
  renderWithStores(<TransitTablePanel panel={panel} />);
  fireEvent.click(await screen.findByRole("button", { name: "Yeniden dene" }));
  expect(await screen.findByRole("table")).toBeInTheDocument();
  expect(api.transitsAt).toHaveBeenCalledTimes(2);
});
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/transit
```
Expected: `Failed to resolve import "./store"` ve `"./TransitTablePanel"`.

- [ ] **Step 3: Uygula**

Create `app/ui/transit/store.tsx`:

```tsx
// Transit anı (UI spec §3.3, §3.4; Plan 11 D7, D8). An oturumluktur: kaydedilmez, her açılışta ve
// her yeni haritada "şimdi"dir. Transitler yalnızca bir panel istediğinde okunur (`useTransit`);
// konumlar, "şimdi" ve dasa yolu Rust'tan gelir (C4). Eski ana ait geç yanıt düşürülür.
import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { api } from "../api";
import type { CommandError, TransitAt, TransitView } from "../bindings";
import { useSession } from "../session/store";

export interface Transit {
  at: TransitAt;
  /** Bu haritanın son yanıtı (yeni an yüklenirken eskisi gösterilir); hata varsa `null`. */
  view: TransitView | null;
  error: CommandError | null;
  loading: boolean;
  setAt(at: TransitAt): void;
  retry(): void;
}

interface Context extends Transit {
  register(): () => void;
}

const TransitContext = createContext<Context | null>(null);

interface Reply {
  key: string;
  chartId: number;
  view: TransitView | null;
  error: CommandError | null;
}

export function TransitProvider({ children }: { children: ReactNode }) {
  const { opened } = useSession();
  const [at, setAtState] = useState<TransitAt>("Now");
  const [demand, setDemand] = useState(0);
  const [nonce, setNonce] = useState(0);
  const [reply, setReply] = useState<Reply | null>(null);
  const chartId = opened?.chart_record.id ?? null;
  // Yeni harita: an "şimdi"ye döner (çizim sırasında; eski an bir kez bile istenmez).
  const [shownChart, setShownChart] = useState(chartId);
  if (shownChart !== chartId) {
    setShownChart(chartId);
    setAtState("Now");
  }

  const key = opened && demand > 0 ? JSON.stringify([chartId, at, nonce, opened.chart.jd_ut, opened.calc_settings]) : null;
  useEffect(() => {
    if (key === null || opened === null) return;
    let live = true;
    const id = opened.chart_record.id;
    void api.transitsAt(at, opened.calc_settings, opened.chart.bhavas, opened.chart.dasha.seed).then((r) => {
      if (!live) return;
      setReply(r.status === "ok" ? { key, chartId: id, view: r.data, error: null } : { key, chartId: id, view: null, error: r.error });
    });
    return () => {
      live = false;
    };
    // `key` açık haritayı, anı, ayarı ve yeniden denemeyi kapsar.
  }, [key]);

  const register = useCallback(() => {
    setDemand((d) => d + 1);
    return () => setDemand((d) => d - 1);
  }, []);
  const setAt = useCallback((next: TransitAt) => setAtState(next), []);
  const retry = useCallback(() => setNonce((n) => n + 1), []);

  const mine = reply !== null && reply.chartId === chartId ? reply : null;
  const value = useMemo<Context>(
    () => ({
      at,
      view: mine?.view ?? null,
      error: mine && mine.key === key ? mine.error : null,
      loading: key !== null && mine?.key !== key,
      setAt,
      retry,
      register,
    }),
    [at, mine, key, setAt, retry, register],
  );
  return <TransitContext.Provider value={value}>{children}</TransitContext.Provider>;
}

function useContextValue(): Context {
  const c = useContext(TransitContext);
  if (c === null) throw new Error("useTransit must be used inside <TransitProvider>");
  return c;
}

/** Transitleri gösteren bileşenler: çizildiği sürece transitler okunur. */
export function useTransit(): Transit {
  const c = useContextValue();
  const { register } = c;
  useEffect(() => register(), [register]);
  return c;
}

/** Yalnızca anı değiştiren ya da okuyan bileşenler: kendi başına okuma başlatmaz. */
export function useTransitControl(): Transit {
  return useContextValue();
}

/** Tarayıcının IANA dilimi; bilinmiyorsa UTC. */
export function browserTimeZone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
}

/** UTC dizgisi → dilimdeki duvar saati (`YYYY-MM-DD`, `HH:MM`); biçimlemedir, hesap değil. */
export function localParts(utc: string, zone: string): { date: string; time: string } | null {
  const d = new Date(utc);
  if (Number.isNaN(d.getTime())) return null;
  const parts = Object.fromEntries(
    new Intl.DateTimeFormat("en-CA", { timeZone: zone, year: "numeric", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", hourCycle: "h23" })
      .formatToParts(d)
      .map((p) => [p.type, p.value]),
  );
  return { date: `${parts.year}-${parts.month}-${parts.day}`, time: `${parts.hour}:${parts.minute}` };
}

export function formatMoment(utc: string, language: string, zone: string): string {
  const d = new Date(utc);
  if (Number.isNaN(d.getTime())) return "—";
  return new Intl.DateTimeFormat(language, { dateStyle: "medium", timeStyle: "short", timeZone: zone }).format(d);
}
```

Create `app/ui/transit/TransitDate.tsx`:

```tsx
// Transit anı seçici: duvar saati tarayıcının IANA diliminde gönderilir, Rust çözer (boşlukta
// hata ve öneri, örtüşmede önceki an; Plan 11 D7). "Şimdi" anı Rust'un saatine bırakır.
import { useEffect, useId, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { browserTimeZone, formatMoment, localParts, useTransitControl } from "./store";

export function TransitDate() {
  const { t, i18n } = useTranslation();
  const { at, view, setAt } = useTransitControl();
  const id = useId();
  const zone = browserTimeZone();
  const shown = view ? localParts(view.utc, zone) : null;
  const [date, setDate] = useState(shown?.date ?? "");
  const [time, setTime] = useState(shown?.time ?? "");
  const [editing, setEditing] = useState(false);

  // Gösterilen an değişince (şimdi, şerit) kutular onu gösterir; kullanıcı yazarken değişmez.
  useEffect(() => {
    if (editing || !shown) return;
    setDate(shown.date);
    setTime(shown.time);
  }, [shown?.date, shown?.time, editing]);

  const apply = (e: FormEvent) => {
    e.preventDefault();
    if (!date || !time) return;
    setEditing(false);
    setAt({ Local: { date, time, tz: zone } });
  };

  const isNow = at === "Now";
  return (
    <form className="transit-date" aria-label={t("transit.date")} onSubmit={apply}>
      <label htmlFor={`${id}-date`}>{t("transit.dateLabel")}</label>
      <input
        id={`${id}-date`}
        type="date"
        value={date}
        onChange={(e) => {
          setEditing(true);
          setDate(e.target.value);
        }}
      />
      <label htmlFor={`${id}-time`}>{t("transit.timeLabel")}</label>
      <input
        id={`${id}-time`}
        type="time"
        value={time}
        onChange={(e) => {
          setEditing(true);
          setTime(e.target.value);
        }}
      />
      <span className="transit-zone">{zone}</span>
      <button type="submit">{t("transit.apply")}</button>
      <button
        type="button"
        aria-pressed={isNow}
        onClick={() => {
          setEditing(false);
          setAt("Now");
        }}
      >
        {t("transit.now")}
      </button>
      <span className="transit-shown" role="status">
        {view ? t(isNow ? "transit.shownNow" : "transit.shown", { when: formatMoment(view.utc, i18n.language, zone) }) : ""}
      </span>
    </form>
  );
}
```

Create `app/ui/transit/TransitTablePanel.tsx`:

```tsx
// Transit tablosu (UI spec §3.3). Konumlar ve natal evler Rust'tan (`transits_at`); saat
// bilinmiyorsa natal ev gösterilmez (spec §7). Retro düz "R", düğümlerde yok (spec §2.3).
import { useTranslation } from "react-i18next";
import { signIndex } from "../chart/adapter";
import { RETRO_MARK, showsRetro, type GrahaId } from "../chart/types";
import { formatDegree } from "../panels/format";
import { nakshatraName } from "../panels/labels";
import { PanelError, usePanelLoadPhase } from "../panels/states";
import type { PanelProps } from "../panels/types";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { browserTimeZone, formatMoment, useTransit } from "./store";
import { TransitDate } from "./TransitDate";
import "./transit.css";

export function TransitTablePanel(_: PanelProps) {
  const { t, i18n } = useTranslation();
  const { opened } = useSession();
  const transit = useTransit();
  usePanelLoadPhase(opened !== null && transit.loading);
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  const unknown = timeUnknown(opened);
  const view = transit.view;
  return (
    <div className="transit-panel">
      <TransitDate />
      {transit.error ? (
        <PanelError error={transit.error} onRetry={transit.retry} />
      ) : (
        view && (
          <table className="transit-table">
            <caption className="visually-hidden">{t("transit.caption", { when: formatMoment(view.utc, i18n.language, browserTimeZone()) })}</caption>
            <thead>
              <tr>
                <th scope="col">{t("transit.col.graha")}</th>
                <th scope="col">{t("transit.col.sign")}</th>
                <th scope="col">{t("transit.col.degree")}</th>
                <th scope="col">{t("transit.col.nakshatra")}</th>
                <th scope="col">{t("transit.col.house")}</th>
                <th scope="col">{t("transit.col.retro")}</th>
              </tr>
            </thead>
            <tbody>
              {view.snapshot.grahas.map((g) => (
                <tr key={g.graha}>
                  <th scope="row">{t(`grahaName.${g.graha}`)}</th>
                  <td>{t(`sign.${signIndex(g.sign)}`)}</td>
                  <td className="num">{g.deg_in_sign === null ? "—" : formatDegree(g.deg_in_sign)}</td>
                  <td>
                    {nakshatraName(g.nakshatra)} {g.pada}
                  </td>
                  <td className="num">{unknown ? "—" : g.natal_house}</td>
                  <td>{showsRetro(g.graha as GrahaId, g.retrograde) ? RETRO_MARK : ""}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )
      )}
    </div>
  );
}
```

Create `app/ui/transit/transit.css`:

```css
.transit-panel {
  display: grid;
  gap: 8px;
  align-content: start;
}

.transit-date {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}

.transit-zone,
.transit-shown {
  color: var(--text-2);
  font-size: 12px;
}

.transit-table {
  border-collapse: collapse;
  font-size: 12px;
}

.transit-table th,
.transit-table td {
  padding: 2px 8px;
  text-align: left;
  border-bottom: 1px solid var(--line);
}

.transit-table .num {
  font-variant-numeric: tabular-nums;
  text-align: right;
}
```

`app/ui/panels/registry.ts`: `import { TransitTablePanel } from "../transit/TransitTablePanel";` ve `transit_table: { titleKey: "panel.transit_table", component: TransitTablePanel },`.

`app/ui/panels/panels.test.tsx` kayıt testi (`"registry covers the seven kinds; …"`) bugün `["dasha_strip", "transit_table", "birth_info"]` türlerinin `UnavailablePanel` olmasını bekler. Döngü listesinden `"transit_table"`'ı çıkar (`["dasha_strip", "birth_info"]`), `import { TransitTablePanel } from "../transit/TransitTablePanel";` ekle ve döngünün altına `expect(PANEL_REGISTRY.transit_table.component).toBe(TransitTablePanel);` yaz.

`app/ui/App.tsx`: `import { TransitProvider } from "./transit/store";`; `Stores` dönüşünde `<SessionProvider initial={boot.opened}><TransitProvider>{children}</TransitProvider></SessionProvider>`.

`app/ui/test/render.tsx`: aynı biçimde `SessionProvider`'ın içine `<TransitProvider>` sar.

i18n — `tr.ts`:

```ts
  transit: {
    date: "Transit anı",
    dateLabel: "Tarih",
    timeLabel: "Saat",
    apply: "Uygula",
    now: "Şimdi",
    shownNow: "Şimdi: {{when}}",
    shown: "Gösterilen an: {{when}}",
    caption: "Transit konumları, {{when}}",
    col: { graha: "Graha", sign: "Burç", degree: "Derece", nakshatra: "Nakshatra", house: "Natal ev", retro: "Retro" },
  },
```

`en.ts`:

```ts
  transit: {
    date: "Transit moment",
    dateLabel: "Date",
    timeLabel: "Time",
    apply: "Apply",
    now: "Now",
    shownNow: "Now: {{when}}",
    shown: "Shown moment: {{when}}",
    caption: "Transit positions, {{when}}",
    col: { graha: "Graha", sign: "Sign", degree: "Degree", nakshatra: "Nakshatra", house: "Natal house", retro: "Retro" },
  },
```

`fr.ts`:

```ts
  transit: {
    date: "Moment du transit",
    dateLabel: "Date",
    timeLabel: "Heure",
    apply: "Appliquer",
    now: "Maintenant",
    shownNow: "Maintenant : {{when}}",
    shown: "Moment affiché : {{when}}",
    caption: "Positions des transits, {{when}}",
    col: { graha: "Graha", sign: "Signe", degree: "Degré", nakshatra: "Nakshatra", house: "Maison natale", retro: "Rétro" },
  },
```

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/transit ui/panels
pnpm --dir app test && pnpm --dir app build
```
Expected: `transit/store.test.tsx` 4, `transit/TransitTablePanel.test.tsx` 4 geçer; `panels.test.tsx` yeşil (kayıt testinin beklentisi Step 3'te `TransitTablePanel`'e çevrildi); toplam `Test Files  50 passed (50)`, `Tests  399 passed (399)`; build `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): session-only transit moment, transit table and date chooser

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: Arayüz: harita panelinde transit halkası (yalnızca D1, kaydedilen panel ayarı)

UI spec §3.3 harita paneli "transit halkası açık/kapalı". Ayar `settings.transit_ring` (Plan 04'ün Transit düzeninde zaten `true`) düzenle kaydedilir; an kaydedilmez (D8). Halka yalnızca D1'de çizilir; başka vargada onay kutusu devre dışıdır ve nedeni bağlıdır (D9). Transit kısaltmaları natal etiketlerle aynı hücre yerleşimine girer, parantezli ve italiktir, etkileşimsizdir (`aria-hidden`); erişilebilir liste haritanın altında gizli metindir. Halka kapalıyken çizim değişmez: SVG anlık görüntüleri ve görsel temeller aynı kalır (C11).

**Files:**
- Modify: `app/ui/chart/types.ts` (`ChartData.transits`)
- Modify: `app/ui/chart/adapter.ts` (`transitPlanets`)
- Modify: `app/ui/chart/ChartSvg.tsx`, `app/ui/chart/chart.css`, `app/ui/chart/ChartSvg.test.tsx`
- Modify: `app/ui/panels/ChartPanel.tsx`, `app/ui/panels/ChartPanel.test.tsx`
- Modify: `app/ui/i18n/{tr,en,fr}.ts`

**Interfaces:**
- Consumes: `useTransit` (Task 7); `layoutLabels`; `setPanelSettings`.
- Produces: `ChartData.transits?: ChartPlanet[]`; `transitPlanets(view: TransitView): ChartPlanet[]`; `TRANSIT_LABEL(abbr) = "(" + abbr + ")"` biçimi; i18n `chart.transitRing`, `chart.transitRingOnlyD1`, `chart.transitList`.

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/chart/ChartSvg.test.tsx` sonuna (dosyadaki `labels` ve `data` yardımcılarını kullan; yoksa aşağıdaki yerel kurulum yeter):

```tsx
test("transit labels share the house layout, are parenthesised and not interactive", () => {
  const data: ChartData = {
    ascSign: 0,
    planets: [{ id: "Sun", sign: 0, degInSign: 10, retrograde: false }],
    transits: [
      { id: "Saturn", sign: 0, degInSign: 3, retrograde: true },
      { id: "Moon", sign: 5, degInSign: 20, retrograde: false },
    ],
  };
  const { container } = render(<ChartSvg data={data} style="north" labels={labels} />);
  const transits = [...container.querySelectorAll(".chart-transit")];
  expect(transits.map((t) => t.textContent)).toEqual([`(${labels.graha.Saturn}R)`, `(${labels.graha.Moon})`]);
  for (const t of transits) {
    expect(t.closest("[role=button]")).toBeNull();
    expect(t).toHaveAttribute("aria-hidden", "true");
  }
  // Natal ve transit etiketi aynı hücrede çakışmaz.
  const sun = container.querySelector('[data-graha="Sun"]')!;
  const sat = container.querySelector('[data-transit="Saturn"]')!;
  expect(`${sun.getAttribute("data-x")},${sun.getAttribute("data-y")}`).not.toBe(`${sat.getAttribute("x")},${sat.getAttribute("y")}`);
});

test("without transits the drawing has no transit labels", () => {
  const { container } = render(<ChartSvg data={{ ascSign: 0, planets: [] }} style="south" labels={labels} />);
  expect(container.querySelector(".chart-transit")).toBeNull();
});
```

(`labels` dosyada `chartLabels`/yerel bir `ChartLabels` nesnesi olarak tanımlıdır; adı farklıysa onu kullan. `ChartData` tipi içe aktarımda yoksa ekle.)

`app/ui/panels/ChartPanel.test.tsx`: dosyanın başına (içe aktarımlardan önce) `vi.mock("../api", () => ({ api: { transitsAt: vi.fn() } }));`, içe aktarımlara `import { api } from "../api";`; sonuna:

```tsx
test("the transit ring toggle is saved with the layout and reads no transits while off", async () => {
  const { persistence } = renderPanel({ varga: "D1", style: "north", transit_ring: false });
  const ring = screen.getByRole("checkbox", { name: "Transit halkası" });
  expect(ring).not.toBeChecked();
  expect(api.transitsAt).not.toHaveBeenCalled();
  fireEvent.click(ring);
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0];
  expect(saved.settings).toEqual({ varga: "D1", style: "north", transit_ring: true });
});

test("with the ring on, D1 draws the nine transits and lists them for screen readers; other vargas do not", async () => {
  vi.mocked(api.transitsAt).mockResolvedValue({ status: "ok", data: fixture().transit });
  const d1 = renderPanel({ varga: "D1", transit_ring: true });
  await waitFor(() => expect(d1.container.querySelectorAll(".chart-transit")).toHaveLength(9));
  expect(screen.getByText(/^Transit:/)).toHaveClass("visually-hidden");
  d1.unmount();
  vi.mocked(api.transitsAt).mockClear();
  const d9 = renderPanel({ varga: "D9", transit_ring: true });
  expect(d9.container.querySelector(".chart-transit")).toBeNull();
  const ring = screen.getByRole("checkbox", { name: "Transit halkası" });
  expect(ring).toBeDisabled();
  expect(ring).toHaveAccessibleDescription("Transit halkası yalnızca D1'de çizilir.");
  expect(api.transitsAt).not.toHaveBeenCalled();
});
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/chart/ChartSvg.test.tsx ui/panels/ChartPanel.test.tsx
```
Expected: `.chart-transit` yok (`expected [] to deeply equal [...]`); "Transit halkası" onay kutusu bulunamaz.

- [ ] **Step 3: Uygula**

`app/ui/chart/types.ts` `ChartData`:

```ts
export interface ChartData {
  /** Lagna burcu; henüz hesaplanmıyorsa `null` (1. ev Koç kabul edilir, çağıran uyarı gösterir). */
  ascSign: number | null;
  planets: ChartPlanet[];
  /** Transit halkası (yalnızca D1, Plan 11 D9): etkileşimsiz, parantezli kısaltmalar. */
  transits?: ChartPlanet[];
}
```

`app/ui/chart/adapter.ts` sonuna (tip içe aktarımına `TransitView`):

```ts
/** `transits_at` dönüşü → çizim verisi (burç ve derece Rust'tan; dönüşüm yalnızca biçimdir). */
export function transitPlanets(view: TransitView): ChartPlanet[] {
  return view.snapshot.grahas.flatMap((g) =>
    isGrahaId(g.graha) && g.deg_in_sign !== null ? [{ id: g.graha, sign: signIndex(g.sign), degInSign: g.deg_in_sign, retrograde: g.retrograde }] : [],
  );
}
```

(`isGrahaId` `./types`'tan içe aktarılır.)

`app/ui/chart/ChartSvg.tsx`: `labelText`'in altına:

```ts
/** Transit etiketi: parantez biçimsel işarettir (renk değil, C9). */
function transitText(p: ChartPlanet, graha: ChartLabels["graha"]): string {
  return `(${graha[p.id]}${showsRetro(p.id, p.retrograde) ? RETRO_MARK : ""})`;
}
```

`placed` `useMemo`'sunu şununla değiştir:

```ts
  const placed = useMemo(() => {
    const group = (list: readonly ChartPlanet[]) => {
      const bySign = new Map<number, ChartPlanet[]>();
      for (const p of list) {
        if (!drawable(p)) continue;
        bySign.set(p.sign, [...(bySign.get(p.sign) ?? []), p]);
      }
      return bySign;
    };
    const natal = group(data.planets);
    const transit = group(data.transits ?? []);
    return cells.map((cell) => {
      const planets = natal.get(cell.sign) ?? [];
      const transits = transit.get(cell.sign) ?? [];
      const texts = [...planets.map((p) => labelText(p, graha)), ...transits.map((p) => transitText(p, graha))];
      const layout: LabelLayout = layoutLabels(cell, texts, hasLagna && cell.lagnaAnchor ? lagna : "");
      return { cell, planets, transits, layout };
    });
    // textKey, graha ve lagna'yı kapsar.
  }, [cells, data.planets, data.transits, textKey, hasLagna]);
```

`</svg>`'den hemen önce (natal gezegen grubunun ardından):

```tsx
      {placed.map(({ cell, planets, transits, layout }) =>
        transits.map((p, i) => {
          const { x, y } = layout.boxes[planets.length + i];
          return (
            <text
              key={`transit-${p.id}`}
              className="chart-transit"
              data-transit={p.id}
              data-house={cell.house}
              x={x}
              y={y}
              textAnchor="middle"
              dominantBaseline="central"
              style={{ fontSize: `${layout.fontSize}px` }}
              aria-hidden="true"
            >
              {transitText(p, labels.graha)}
            </text>
          );
        }),
      )}
```

`app/ui/chart/chart.css` sonuna:

```css
/* Transit halkası: italik ve parantezli; renk tek başına anlam taşımaz (C9), baskıda okunur. */
.chart-transit {
  fill: var(--text-2);
  font-style: italic;
  pointer-events: none;
}
```

`app/ui/panels/ChartPanel.tsx`: içe aktarımlara `import { transitPlanets } from "../chart/adapter";` (var olan `chartDataFromChart` satırına ekle) ve `import { useTransit } from "../transit/store";`. Panelin gövdesinde `const style = …` satırının altına:

```tsx
  const ringOn = panel.settings.transit_ring === true;
  const ringDrawn = ringOn && varga === "D1";
```

Araç çubuğunun (`chart-panel-toolbar`) sonuna, stil seçiminin altına:

```tsx
        <label className="chart-ring-toggle">
          <input
            type="checkbox"
            checked={ringOn}
            disabled={varga !== "D1"}
            aria-describedby={varga !== "D1" ? `${id}-ring` : undefined}
            onChange={(e) => setPanelSettings(panel.id, { transit_ring: e.target.checked })}
          />
          {t("chart.transitRing")}
        </label>
        {varga !== "D1" && (
          <span id={`${id}-ring`} className="visually-hidden">
            {t("chart.transitRingOnlyD1")}
          </span>
        )}
```

Harita çizimini (`<ChartSvg data={data} … />`) şununla değiştir:

```tsx
      {ringDrawn ? <RingChart data={data} {...svgProps} /> : <ChartSvg data={data} {...svgProps} />}
```

Burada `svgProps` panelde `ChartSvg`'ye bugün verilen bütün prop'lardır (`style`, `labels`, `highlight`, `pressed`, `onHover`, `onPin`); `const svgProps = { style, labels: chartLabels(t, varga), … }` olarak bir kez kur ve iki dalda kullan. Dosyanın sonuna:

```tsx
/** Halka açıkken: transitler okunur (tüketici kaydı) ve natal çizime eklenir. */
function RingChart({ data, ...props }: { data: ChartData } & Omit<ChartSvgProps, "data">) {
  const { t } = useTranslation();
  const { view } = useTransit();
  const transits = view ? transitPlanets(view) : [];
  const list = transits.map((p) => `${t(`grahaName.${p.id}`)} ${t(`sign.${p.sign}`)} ${Math.floor(p.degInSign)}°`).join(", ");
  return (
    <>
      <ChartSvg data={{ ...data, transits }} {...props} />
      {view && <p className="visually-hidden">{t("chart.transitList", { list })}</p>}
    </>
  );
}
```

İçe aktarımlara `ChartData` (`../chart/types`) ve `ChartSvgProps` (`../chart/ChartSvg`) ekle. Halka kapalıyken `RingChart` çizilmez, dolayısıyla transit okunmaz.

i18n — `tr.ts` `chart` içine:

```ts
    transitRing: "Transit halkası",
    transitRingOnlyD1: "Transit halkası yalnızca D1'de çizilir.",
    transitList: "Transit: {{list}}",
```

`en.ts`: `transitRing: "Transit ring"`, `transitRingOnlyD1: "The transit ring is drawn in D1 only."`, `transitList: "Transit: {{list}}"`.

`fr.ts`: `transitRing: "Anneau des transits"`, `transitRingOnlyD1: "L’anneau des transits n’est tracé qu’en D1."`, `transitList: "Transit : {{list}}"`.

Türkçe test "Transit:" önekini bekler (`/^Transit:/`).

- [ ] **Step 4: Testlerin ve görsel temellerin değişmediğini gör**

```bash
pnpm --dir app exec vitest run ui/chart ui/panels
git status --short app/ui/chart/__snapshots__
pnpm --dir app test && pnpm --dir app build
pnpm --dir app e2e:visual
rm -rf app/test-results
```
Expected: ChartSvg +2, ChartPanel +2 geçer; `__snapshots__` değişmez (`git status` boş: halka kapalıyken çizim aynı); toplam `Test Files  50 passed (50)`, `Tests  403 passed (403)`; build `✓ built`; görsel `6 passed` (temeller değişmedi, C11).

- [ ] **Step 5: Commit**

```bash
git add app/ui
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): transit ring on the D1 chart panel as a saved panel setting

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: Arayüz: dasa şeridi; boş tembel dasa seviyesi hata olur (M6-6)

UI spec §3.3 `dasha_strip` (seviye: maha / antar / pratyantar, panel ayarı `settings.level`, varsayılan Plan 04'ün Görüşme düzenindeki `antar`). Şerit transit anının dasa yolunu (`TransitView.dasha_path`, Rust) gösterir: seçilen seviyede, o anın üst döneminin alt dönemleri; anı içeren dönem `aria-current`, "transit anında" metni ve kalın kenarla işaretlenir (C9). Bir döneme tıklamak transit anını dönemin başlangıcına taşır (UI spec §3.4, D10). Dönemler haritayla gelen ilk üç seviyedendir (`chart.dasha.periods`); tarih biçimi `formatJdDate` (C4). M6-6: dasa ağacında tembel yüklenen seviye boş gelirse sessiz boş liste yerine panel hatası ve "Yeniden dene" gösterilir.

Görüşme düzeni (varsayılan) `dasha_strip` içerir: şerit artık `transits_at` çağırır. Panoyu çizen testlerin `../api` taklitleri (`App.test.tsx`, `dashboard/Dashboard.test.tsx`) ve e2e IPC taklidi bu görevde `transitsAt` / `transits_at` kazanır.

**Files:**
- Create: `app/ui/dasha/DashaStripPanel.tsx`, `app/ui/dasha/DashaStripPanel.test.tsx`
- Modify: `app/ui/dasha/DashaTreePanel.tsx`, `app/ui/dasha/DashaTreePanel.test.tsx`, `app/ui/dasha/dasha.css`
- Modify: `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx` (kayıt testi)
- Modify: `app/ui/App.test.tsx`, `app/ui/dashboard/Dashboard.test.tsx` (taklit)
- Modify: `app/e2e/ipc.ts` (`transits_at`)
- Modify: `app/ui/i18n/{tr,en,fr}.ts`

**Interfaces:**
- Consumes: `useTransit` (Task 7); `childrenOf`, `formatJdDate`, `samePath` (`dasha/tree.ts`); `setPanelSettings`.
- Produces: `STRIP_LEVELS = ["maha", "antar", "pratyantar"]`, `stripLevel(settings): StripLevel`, `DashaStripPanel(props: PanelProps)`; i18n `dasha.strip.*`, `errors.dasha_empty`.

- [ ] **Step 1: Başarısız testleri yaz**

Create `app/ui/dasha/DashaStripPanel.test.tsx`:

```tsx
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn() } }));

import { api } from "../api";
import type { Panel } from "../layout/model";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { DashaStripPanel } from "./DashaStripPanel";
import { childrenOf } from "./tree";

const strip = (level?: string): Panel => ({ id: "ds", kind: "dasha_strip", x: 0, y: 0, w: 12, h: 4, settings: level ? { level } : {} });

beforeEach(() => vi.mocked(api.transitsAt).mockResolvedValue({ status: "ok", data: fixture().transit }));
afterEach(() => vi.mocked(api.transitsAt).mockReset());

test("antar (default): the current maha's nine antardashas, the one at the transit moment marked", async () => {
  renderWithStores(<DashaStripPanel panel={strip()} />);
  const list = await screen.findByRole("list", { name: /Dasa şeridi/ });
  const items = within(list).getAllByRole("button");
  expect(items).toHaveLength(9);
  const [maha, antar] = fixture().transit.dasha_path;
  const expected = childrenOf(fixture().opened.chart.dasha.periods, [maha]);
  expect(items.map((b) => b.getAttribute("data-lord"))).toEqual(expected.map((p) => p.lords[1]));
  const current = items.filter((b) => b.getAttribute("aria-current") === "true");
  expect(current).toHaveLength(1);
  expect(current[0]).toHaveAttribute("data-lord", antar);
  expect(current[0]).toHaveTextContent("transit anında");
});

test("maha level lists the nine mahadashas", async () => {
  renderWithStores(<DashaStripPanel panel={strip("maha")} />);
  const list = await screen.findByRole("list", { name: /Dasa şeridi/ });
  expect(within(list).getAllByRole("button")).toHaveLength(9);
});

test("choosing a period moves the transit moment to its start", async () => {
  renderWithStores(<DashaStripPanel panel={strip("pratyantar")} />);
  const list = await screen.findByRole("list", { name: /Dasa şeridi/ });
  const first = within(list).getAllByRole("button")[0];
  fireEvent.click(first);
  const [maha, antar] = fixture().transit.dasha_path;
  const start = childrenOf(fixture().opened.chart.dasha.periods, [maha, antar])[0].start_jd_ut;
  await waitFor(() => expect(vi.mocked(api.transitsAt).mock.calls.at(-1)?.[0]).toEqual({ Jd: start }));
});

test("a transit error is shown with a retry", async () => {
  vi.mocked(api.transitsAt).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderWithStores(<DashaStripPanel panel={strip()} />);
  fireEvent.click(await screen.findByRole("button", { name: "Yeniden dene" }));
  expect(await screen.findByRole("list", { name: /Dasa şeridi/ })).toBeInTheDocument();
});

test("no open chart: a message, no request", () => {
  renderWithStores(<DashaStripPanel panel={strip()} />, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.transitsAt).not.toHaveBeenCalled();
});
```

(`"Açık harita yok."` `panel.noChart`'ın Türkçe metnidir; dosyadaki metin farklıysa `tr.panel.noChart`'ı içe aktarıp onu kullan.)

`app/ui/dasha/DashaTreePanel.test.tsx` sonuna (dosyanın `api.dashaChildren` taklidini ve panel tanımını kullan):

```tsx
test("an empty lazily loaded level is an error with a retry, not a silent empty list (M6-6)", async () => {
  vi.mocked(api.dashaChildren).mockResolvedValueOnce({ status: "ok", data: [] });
  const opened = fixture().opened;
  opened.view_state.dasha_expanded_path = fixture().dasha_children.path;
  renderWithStores(<DashaTreePanel panel={panel} />, { opened });
  expect(await screen.findByText("Bu dönemin alt dönemleri alınamadı.")).toBeInTheDocument();
  vi.mocked(api.dashaChildren).mockResolvedValueOnce({ status: "ok", data: fixture().dasha_children.periods });
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(screen.queryByText("Bu dönemin alt dönemleri alınamadı.")).toBeNull());
});
```

(`panel` dosyanın başındaki dasa ağacı paneli sabitidir; `fixture`, `renderWithStores`, `screen`, `fireEvent`, `waitFor` zaten içe aktarılıdır.)

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/dasha
```
Expected: `Failed to resolve import "./DashaStripPanel"`; ağaç testi "Bu dönemin alt dönemleri alınamadı." metnini bulamaz.

- [ ] **Step 3: Uygula**

Create `app/ui/dasha/DashaStripPanel.tsx`:

```tsx
// Dasa şeridi (UI spec §3.3, §3.4). Transit anının dasa yolu Rust'tan gelir (`transits_at`,
// Plan 11 D10); arayüz tarih karşılaştırmaz (C4). Seçilen seviyede anın üst döneminin alt
// dönemleri sıralanır; bir döneme tıklamak transit anını o dönemin başlangıcına taşır.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { DashaPeriod } from "../bindings";
import { PanelError, usePanelLoadPhase } from "../panels/states";
import type { PanelProps } from "../panels/types";
import { useSession } from "../session/store";
import { useTransit } from "../transit/store";
import { useLayoutStore } from "../view/store";
import "./dasha.css";
import { childrenOf, formatJdDate, samePath } from "./tree";

export const STRIP_LEVELS = ["maha", "antar", "pratyantar"] as const;
export type StripLevel = (typeof STRIP_LEVELS)[number];

export function stripLevel(settings: Record<string, unknown>): StripLevel {
  return (STRIP_LEVELS as readonly unknown[]).includes(settings.level) ? (settings.level as StripLevel) : "antar";
}

/** Seviyenin üst yol uzunluğu: maha için kök (0), antar için 1, pratyantar için 2. */
const PARENT_DEPTH: Record<StripLevel, number> = { maha: 0, antar: 1, pratyantar: 2 };

export function DashaStripPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Strip level={stripLevel(panel.settings)} id={id} onLevel={(level) => setPanelSettings(panel.id, { level })} />;
}

function Strip({ level, id, onLevel }: { level: StripLevel; id: string; onLevel(level: StripLevel): void }) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const transit = useTransit();
  usePanelLoadPhase(transit.loading && transit.view === null);
  if (!opened) return null;
  const depth = PARENT_DEPTH[level];
  const path = transit.view?.dasha_path ?? null;
  const offset = opened.chart_record.resolved.offset_secs;
  const periods: DashaPeriod[] = path && path.length > depth ? childrenOf(opened.chart.dasha.periods, path.slice(0, depth)) : [];
  const currentPath = path?.slice(0, depth + 1) ?? [];

  const name = (p: DashaPeriod) => p.lords.map((g) => t(`grahaName.${g}`)).join("–");
  return (
    <div className="dasha-strip-panel">
      <div className="dasha-strip-toolbar">
        <label htmlFor={`${id}-level`}>{t("dasha.strip.level")}</label>
        <select id={`${id}-level`} value={level} onChange={(e) => onLevel(e.target.value as StripLevel)}>
          {STRIP_LEVELS.map((l) => (
            <option key={l} value={l}>
              {t(`dasha.strip.levels.${l}`)}
            </option>
          ))}
        </select>
      </div>
      {transit.error ? (
        <PanelError error={transit.error} onRetry={transit.retry} />
      ) : transit.view && periods.length === 0 ? (
        <p className="panel-message">{t("dasha.strip.outside")}</p>
      ) : (
        periods.length > 0 && (
          <ol className="dasha-strip" aria-label={t("dasha.strip.label", { level: t(`dasha.strip.levels.${level}`) })}>
            {periods.map((p) => {
              const current = samePath(p.lords, currentPath);
              return (
                <li key={p.lords.join(",")}>
                  <button
                    type="button"
                    className={`dasha-strip-item${current ? " dasha-strip-item--current" : ""}`}
                    data-lord={p.lords[p.lords.length - 1]}
                    aria-current={current ? "true" : undefined}
                    onClick={() => transit.setAt({ Jd: p.start_jd_ut })}
                  >
                    <span className="dasha-strip-name">{name(p)}</span>
                    <span className="dasha-strip-date">{formatJdDate(p.start_jd_ut, offset)}</span>
                    {current && <span className="dasha-strip-now">{t("dasha.strip.current")}</span>}
                  </button>
                </li>
              );
            })}
          </ol>
        )
      )}
    </div>
  );
}
```

`dasha/tree.ts`'te `samePath` var (`export function samePath`); `childrenOf(periods, path)` `path`'in doğrudan alt dönemlerini verir. `DashaPeriod.start_jd_ut` TS'de `number | null`'dır; `{ Jd: p.start_jd_ut }` `TransitAt`'ın `{ Jd: number | null }` biçimine uyar.

`app/ui/dasha/dasha.css` sonuna:

```css
.dasha-strip-panel {
  display: grid;
  gap: 6px;
}

.dasha-strip {
  display: flex;
  gap: 4px;
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-x: auto;
}

.dasha-strip-item {
  display: grid;
  gap: 2px;
  min-width: 6rem;
  padding: 4px 6px;
  border: 1px solid var(--line);
  background: var(--panel);
  color: var(--text);
  font: inherit;
  text-align: left;
}

/* Güncel dönem: zemin + 2 px kenar + metin; renk tek başına anlam taşımaz (C9). */
.dasha-strip-item--current {
  background: var(--select);
  border: 2px solid var(--ink);
  font-weight: var(--fw-medium);
}

.dasha-strip-date,
.dasha-strip-now {
  font-size: 12px;
  color: var(--text-2);
  font-variant-numeric: tabular-nums;
}
```

`app/ui/dasha/DashaTreePanel.tsx` `childList`'i şununla değiştir:

```tsx
  // Yüklenen liste de `childrenOf` süzgecinden geçer: yalnızca bu yolun bir alt seviyesi
  // kabul edilir (yanlış bir yanıt ağacı sonsuz döngüye sokamaz). Boş yanıt hatadır (M6-6):
  // her dönemin dokuz alt dönemi vardır, boş liste sessizce boş dal olarak gösterilmez.
  const childList = (path: readonly Graha[]): DashaPeriod[] | LazyEntry | undefined => {
    if (path.length < PRELOADED_LEVEL) return childrenOf(periods, path);
    const entry = lazy.entries[pathKey(path)];
    if (entry?.status !== "ok") return entry;
    const kids = childrenOf(entry.periods, path);
    return kids.length > 0 ? { status: "ok", periods: kids } : { status: "error", error: { code: "dasha_empty", params: {} } };
  };
```

`app/ui/panels/registry.ts`: `import { DashaStripPanel } from "../dasha/DashaStripPanel";` ve `dasha_strip: { titleKey: "panel.dasha_strip", component: DashaStripPanel },`.

`app/ui/panels/panels.test.tsx` kayıt testinde (Task 7'den sonra döngü listesi `["dasha_strip", "birth_info"]`'dur) listeden `"dasha_strip"`'i çıkar (`["birth_info"]`), `import { DashaStripPanel } from "../dasha/DashaStripPanel";` ekle ve `expect(PANEL_REGISTRY.dasha_strip.component).toBe(DashaStripPanel);` yaz. Aynı dosyada `UnavailablePanel`'i `kind: "dasha_strip"` ile doğrudan çizen test değişmez (bileşeni dener, kaydı değil).

`app/ui/App.test.tsx` ve `app/ui/dashboard/Dashboard.test.tsx` `vi.mock(…api…)` nesnelerine:

```ts
    transitsAt: vi.fn(async () => ({ status: "ok", data: (await import("./test/fixture")).fixture().transit })),
```

(`Dashboard.test.tsx`'te yol `"../test/fixture"`'dır.) Taklit fabrikası en üste taşındığı için fikstür çağrı anında içe aktarılır.

`app/e2e/ipc.ts` `respond` nesnesine `dasha_children`'ın altına:

```ts
        transits_at: () => fixture.transit,
```

i18n — `tr.ts` `dasha` içine:

```ts
    strip: {
      level: "Seviye",
      levels: { maha: "Mahadasa", antar: "Antardasa", pratyantar: "Pratyantardasa" },
      label: "Dasa şeridi: {{level}}",
      current: "transit anında",
      outside: "Transit anı Vimshottari döngüsünün dışında.",
    },
```

`errors` içine `dasha_empty: "Bu dönemin alt dönemleri alınamadı.",`.

`en.ts`: `strip: { level: "Level", levels: { maha: "Mahadasha", antar: "Antardasha", pratyantar: "Pratyantardasha" }, label: "Dasha strip: {{level}}", current: "at the transit moment", outside: "The transit moment is outside the Vimshottari cycle." }`, `errors.dasha_empty: "The sub-periods of this period could not be loaded."`.

`fr.ts`: `strip: { level: "Niveau", levels: { maha: "Mahadasha", antar: "Antardasha", pratyantar: "Pratyantardasha" }, label: "Bande des dashas : {{level}}", current: "au moment du transit", outside: "Le moment du transit est hors du cycle Vimshottari." }`, `errors.dasha_empty: "Les sous-périodes de cette période n’ont pas pu être chargées."`.

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/dasha ui/dashboard ui/App.test.tsx ui/panels
pnpm --dir app test && pnpm --dir app build
pnpm --dir app e2e
rm -rf app/test-results
```
Expected: `DashaStripPanel.test.tsx` 5, ağaç +1 geçer; panel kayıt testi Step 3'teki beklentiyle yeşil; toplam `Test Files  51 passed (51)`, `Tests  409 passed (409)`; build `✓ built`; e2e `6 passed` (şerit taklitten `transits_at` alır).

- [ ] **Step 5: Commit**

```bash
git add app/ui app/e2e/ipc.ts
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): dasha strip at the transit moment; empty lazy dasha level is an error

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---
### Task 10: Arayüz: `birth_info` paneli ve uyarı rozetlerinin açıklaması

Açık haritanın saklanan doğum verisi (tarih, saat, kesinlik, yer, koordinat, saat dilimi, kaynak notu), Rust'ın çözdüğü an (UTC, kısaltma, ofset, tzdb sürümü), danışanın cinsiyeti ve anın ve haritanın uyarıları. Uyarılar rozet adı ve açıklamasıyla görünür metindir; `title`'a dayanmaz (renk ya da fareyle üzerine gelme tek başına bilgi taşımaz). Panel yalnızca `session.opened`'ı okur, komut çağırmaz.

**Files:**
- Create: `app/ui/panels/BirthInfoPanel.tsx`
- Create: `app/ui/panels/BirthInfoPanel.test.tsx`
- Modify: `app/ui/panels/registry.ts`
- Modify: `app/ui/panels/panels.test.tsx`
- Modify: `app/ui/panels/panels.css`
- Modify: `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`

**Interfaces:**
- Consumes: `useSession().opened: OpenedClient | null`; `formatFixedOffset(secs: number): string` (`birth/model.ts`); `badge.*`, `badgeHint.*`, `birth.certainty*`, `birth.tzLmt`, `chart.kind.*` i18n anahtarları (mevcut).
- Produces: `BirthInfoPanel(props: PanelProps)`; `PANEL_REGISTRY.birth_info.component === BirthInfoPanel`; i18n `birthInfo.*` ve üst düzey `gender.{label,none,Male,Female,Other}` (Task 11 doğum formu da kullanır).

- [ ] **Step 1: Başarısız testi yaz**

`app/ui/panels/BirthInfoPanel.test.tsx`:

```tsx
import { screen, within } from "@testing-library/react";
import type { OpenedClient } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { BirthInfoPanel } from "./BirthInfoPanel";

const panel: Panel = { id: "b", kind: "birth_info", x: 0, y: 0, w: 4, h: 6, settings: {} };

function renderInfo(opened: OpenedClient | null = fixture().opened) {
  return renderWithStores(<BirthInfoPanel panel={panel} />, { opened });
}

/** `<dt>` metnine göre yanındaki `<dd>`. */
function value(term: string): string {
  const dt = screen.getAllByRole("term").find((d) => d.textContent === term);
  if (!dt) throw new Error(`no term ${term}`);
  return dt.nextElementSibling?.textContent ?? "";
}

test("shows the stored birth data and the moment Rust resolved", () => {
  renderInfo();
  expect(screen.getByRole("region", { name: "Doğum bilgisi" })).toBeInTheDocument();
  expect(value("Tarih")).toBe("2000-01-01");
  expect(value("Saat")).toBe("17:30");
  expect(value("Saat kesinliği")).toBe("Kesin");
  expect(value("Yer")).toBe("New Delhi, Delhi, India");
  expect(value("Koordinat")).toBe("28.6139, 77.2090");
  expect(value("Saat dilimi")).toBe("Asia/Kolkata");
  expect(value("UTC anı")).toBe("2000-01-01 12:00:00 UTC");
  expect(value("Ofset")).toBe("IST +05:30");
  expect(value("tzdb sürümü")).toBe("2026f");
  expect(value("Cinsiyet")).toBe("Belirtilmemiş");
  expect(value("Harita türü")).toBe("Doğum haritası");
  // `term` rolünün adı içeriğinden gelmez: ada göre sorgu her zaman boş dönerdi. Metne bakılır.
  expect(screen.getAllByRole("term").map((d) => d.textContent)).not.toContain("Kaynak notu");
  expect(screen.getByText("Uyarı yok.")).toBeInTheDocument();
});

test("an unknown time says so and its warning is explained in text, not only in a badge", () => {
  const o = unknownTimeOpened();
  o.chart.meta.warnings = ["LowPrecision"];
  renderInfo(o);
  expect(value("Saat")).toBe("Bilinmiyor");
  const warnings = within(screen.getByRole("list", { name: "Uyarılar" })).getAllByRole("listitem");
  expect(warnings.map((w) => w.textContent)).toEqual([
    "Saat bilinmiyor: Doğum saati bilinmiyor; öğlen 12:00 alındı.",
    "Düşük hassasiyet: Güneş veya Ay yedek ephemeris ile hesaplandı.",
  ]);
});

test("fixed offset, LMT, manual coordinates, gender, source note; no chart open", () => {
  const o = fixture().opened;
  o.chart_record.birth.tz = { Fixed: -17_776 };
  o.chart_record.birth.place = { ...o.chart_record.birth.place, name: "", admin1: "", country: "" };
  o.chart_record.birth.notes = "Nüfus kaydı";
  o.client.gender = "Female";
  const { unmount } = renderInfo(o);
  expect(value("Saat dilimi")).toBe("-04:56:16");
  expect(value("Yer")).toBe("Elle girilmiş koordinat");
  expect(value("Cinsiyet")).toBe("Kadın");
  expect(value("Kaynak notu")).toBe("Nüfus kaydı");
  unmount();
  o.chart_record.birth.tz = "Lmt";
  const second = renderInfo(o);
  expect(value("Saat dilimi")).toBe("Yerel ortalama saat (LMT)");
  second.unmount();
  renderInfo(null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
});
```

`app/ui/panels/panels.test.tsx` kayıt testinde (Task 7 ve 9'dan sonra `UnavailablePanel` beklenen tek tür `birth_info`'dur) döngüyü şu satırla değiştir ve `BirthInfoPanel`'i içe aktar:

```tsx
  expect(PANEL_REGISTRY.birth_info.component).toBe(BirthInfoPanel);
```

```tsx
import { BirthInfoPanel } from "./BirthInfoPanel";
```

Test adını `"registry covers the seven kinds and every known kind is implemented"` yap.

- [ ] **Step 2: Testin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/panels/BirthInfoPanel.test.tsx ui/panels/panels.test.tsx
```
Expected: `Failed to resolve import "./BirthInfoPanel"`.

- [ ] **Step 3: Uygula**

`app/ui/panels/BirthInfoPanel.tsx`:

```tsx
// Doğum bilgisi paneli (spec §3.3): açık haritanın saklanan doğum verisi, Rust'ın çözdüğü an
// ve uyarılar. Uyarılar ad ve açıklamasıyla düz metindir (rozetin `title`'ı klavyeyle okunmaz).
// Komut çağırmaz; değerler `session.opened`'dan gelir.
import type { TFunction } from "i18next";
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { BirthData, OpenedClient, TzChoice } from "../bindings";
import { formatFixedOffset } from "../birth/model";
import { useSession } from "../session/store";
import "./panels.css";
import type { PanelProps } from "./types";

function tzText(t: TFunction, tz: TzChoice): string {
  if (tz === "Lmt") return t("birth.tzLmt");
  if ("Iana" in tz && tz.Iana !== undefined) return tz.Iana;
  return formatFixedOffset(tz.Fixed ?? 0);
}

/** "17:30:00" → "17:30"; sıfırdan farklı saniye ve kesir korunur. */
function timeText(t: TFunction, b: BirthData): string {
  if (b.time_certainty === "Unknown" || b.time === null) return t("birth.certaintyUnknown");
  return b.time.replace(/^(\d{2}:\d{2}):00(\.0+)?$/, "$1");
}

function placeText(t: TFunction, b: BirthData): string {
  const parts = [b.place.name, b.place.admin1, b.place.country].filter((s) => s.trim() !== "");
  return parts.length > 0 ? parts.join(", ") : t("birthInfo.manualPlace");
}

function coordText(b: BirthData): string {
  const { lat, lon } = b.place;
  return lat === null || lon === null ? "—" : `${lat.toFixed(4)}, ${lon.toFixed(4)}`;
}

export function BirthInfoPanel(_props: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <BirthInfo opened={opened} />;
}

function BirthInfo({ opened }: { opened: OpenedClient }) {
  const { t } = useTranslation();
  const id = useId();
  const b = opened.chart_record.birth;
  const r = opened.chart_record.resolved;
  const warnings: string[] = [...r.warnings, ...opened.chart.meta.warnings];
  const rows: [string, string][] = [
    [t("birthInfo.date"), b.date],
    [t("birthInfo.time"), timeText(t, b)],
    [t("birthInfo.certainty"), t(`birth.certainty${b.time_certainty}`)],
    [t("birthInfo.place"), placeText(t, b)],
    [t("birthInfo.coords"), coordText(b)],
    [t("birthInfo.tz"), tzText(t, b.tz)],
    [t("birthInfo.utc"), r.utc.replace("T", " ").replace("Z", " UTC")],
    [t("birthInfo.offset"), `${r.abbrev} ${formatFixedOffset(r.offset_secs)}`.trim()],
    [t("birthInfo.tzdb"), r.tzdb_version],
    [t("gender.label"), opened.client.gender ? t(`gender.${opened.client.gender}`) : t("gender.none")],
    [t("birthInfo.kind"), t(`chart.kind.${opened.chart_record.kind}`)],
  ];
  if (b.notes.trim() !== "") rows.push([t("birthInfo.source"), b.notes]);
  return (
    <section className="birth-info" aria-label={t("panel.birth_info")}>
      <dl className="birth-info-list">
        {rows.map(([term, value]) => (
          <div key={term} className="birth-info-row">
            <dt>{term}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
      <h3 id={`${id}-warnings`} className="birth-info-heading">
        {t("birthInfo.warnings")}
      </h3>
      {warnings.length === 0 ? (
        <p className="panel-message">{t("birthInfo.noWarnings")}</p>
      ) : (
        <ul className="birth-info-warnings" aria-labelledby={`${id}-warnings`}>
          {warnings.map((w) => (
            <li key={w}>
              <strong>{t(`badge.${w}`)}</strong>: {t(`badgeHint.${w}`)}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
```

`<dt>`/`<dd>` çiftleri bir `<div>` içindedir (HTML `dl` içinde `div` gruplamaya izin verir); `nextElementSibling` testte çifti bulur. `<li>` metni `"Saat bilinmiyor: Doğum saati…"` biçimindedir: `<strong>` ile iki nokta arasında boşluk yoktur.

`app/ui/panels/registry.ts`: `import { BirthInfoPanel } from "./BirthInfoPanel";` ve satırı değiştir:

```ts
  birth_info: { titleKey: "panel.birth_info", component: BirthInfoPanel },
```

`app/ui/panels/panels.css` sonuna:

```css
.birth-info-list {
  display: grid;
  grid-template-columns: max-content 1fr;
  gap: 2px 12px;
  margin: 0;
}
.birth-info-row {
  display: contents;
}
.birth-info-row dt {
  color: var(--text-2);
}
.birth-info-row dd {
  margin: 0;
  font-variant-numeric: tabular-nums;
}
.birth-info-heading {
  margin: 12px 0 4px;
  font-size: inherit;
  font-weight: 500;
}
.birth-info-warnings {
  margin: 0;
  padding-left: 16px;
}
```

i18n — `tr.ts` üst düzeye (`birth`'ün yanına):

```ts
  birthInfo: {
    date: "Tarih",
    time: "Saat",
    certainty: "Saat kesinliği",
    place: "Yer",
    coords: "Koordinat",
    tz: "Saat dilimi",
    utc: "UTC anı",
    offset: "Ofset",
    tzdb: "tzdb sürümü",
    kind: "Harita türü",
    source: "Kaynak notu",
    warnings: "Uyarılar",
    noWarnings: "Uyarı yok.",
    manualPlace: "Elle girilmiş koordinat",
  },
  gender: { label: "Cinsiyet", none: "Belirtilmemiş", Male: "Erkek", Female: "Kadın", Other: "Diğer" },
```

`en.ts`:

```ts
  birthInfo: {
    date: "Date",
    time: "Time",
    certainty: "Time accuracy",
    place: "Place",
    coords: "Coordinates",
    tz: "Time zone",
    utc: "UTC moment",
    offset: "Offset",
    tzdb: "tzdb version",
    kind: "Chart type",
    source: "Source note",
    warnings: "Warnings",
    noWarnings: "No warnings.",
    manualPlace: "Coordinates entered by hand",
  },
  gender: { label: "Gender", none: "Not given", Male: "Male", Female: "Female", Other: "Other" },
```

`fr.ts`:

```ts
  birthInfo: {
    date: "Date",
    time: "Heure",
    certainty: "Précision de l’heure",
    place: "Lieu",
    coords: "Coordonnées",
    tz: "Fuseau horaire",
    utc: "Instant UTC",
    offset: "Décalage",
    tzdb: "Version tzdb",
    kind: "Type de thème",
    source: "Note de source",
    warnings: "Avertissements",
    noWarnings: "Aucun avertissement.",
    manualPlace: "Coordonnées saisies à la main",
  },
  gender: { label: "Genre", none: "Non indiqué", Male: "Homme", Female: "Femme", Other: "Autre" },
```

(`chart.kind.Natal` Türkçede "Doğum haritası"dır; `i18n.test.ts`'in üç dilde aynı anahtar kümesi denetimi yeni anahtarları kendiliğinden kapsar.)

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/panels
pnpm --dir app test && pnpm --dir app build
```
Expected: `BirthInfoPanel.test.tsx` 3 geçer, `panels.test.tsx` yeşil; toplam `Test Files  52 passed (52)`, `Tests  412 passed (412)`; build `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/panels app/ui/i18n
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): birth_info panel with the resolved moment and explained warnings

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
(`grep` çıktısı boş olmalıdır.)

---

### Task 11: Doğum formu: cinsiyet, duyurulan hatalar ve `radiogroup` (M6-1), `fold` sıfırlama, Esc onayı, `-0:00`, eski arama listesinde Enter

Plan 10 madde 7 (M6-1) ve madde 12 (Plan 09 kapsam dışı 13–15) burada kapanır. Değişiklikler:

1. **Cinsiyet** seçimi (Belirtilmemiş/Erkek/Kadın/Diğer). Yeni danışanda `NewClient.gender` ile gider; ekleme kipinde (açık danışana harita) danışan alanları gibi kilitlidir; düzenleme kipinde değişmişse harita kaydından sonra `api.updateClient(clientId, name, gender)` çağrılır ve dönen `Client` açılan sonuca yazılır.
2. **M6-1:** gönderimde (biçim hatası ya da alana bağlanan arka uç hatası) odak ilk `aria-invalid` alana gider; görünmez bir `role="status"` "Düzeltilmesi gereken N alan var." der. Katlama grubu `role="radiogroup"`, `aria-invalid` ve hatasına `aria-describedby` taşır; belirsiz saat açıklaması canlı önizleme bölümünde duyurulur.
3. Tarih değişince `fold` sıfırlanır (saat ve dilim gibi).
4. Taslak değişmişken Esc onay sorar (iç içe `ConfirmDialog`); değişmemiş form hemen kapanır. "Vazgeç" düğmesi açık bir istektir, sormaz. `Dialog` yalnızca en üstteki modal pencere olarak tuşları işler.
5. `parseFixedOffset("-0:00")` `-0` değil `0` döner; `errorField` eşleme tablosunun tamamı test edilir.
6. `PlaceSearch`: yazım sonrası yanıt gelmeden (liste eskiyken) Enter tüketilir ve eski sonucu seçmez.

**Files:**
- Modify: `app/ui/birth/BirthForm.tsx`
- Modify: `app/ui/birth/BirthForm.test.tsx`
- Modify: `app/ui/birth/model.ts`
- Modify: `app/ui/birth/model.test.ts`
- Modify: `app/ui/birth/PlaceSearch.tsx`
- Modify: `app/ui/birth/PlaceSearch.test.tsx`
- Modify: `app/ui/shell/Dialog.tsx`
- Modify: `app/ui/shell/Dialog.test.tsx`
- Modify: `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`

**Interfaces:**
- Consumes: `api.updateClient(clientId: number, name: string, gender: Gender | null): Promise<Result<Client>>` (Task 4); `gender.*` i18n (Task 10); `ConfirmDialog({ title, message, confirmLabel, onConfirm, onCancel })` (mevcut).
- Produces: `Dialog` yalnızca belgedeki son `[role="dialog"][aria-modal="true"]` ise Esc/Tab işler; i18n `birth.foldHint`, `birth.errorSummary`, `birth.discardTitle`, `birth.discardMessage`, `birth.discardConfirm`.

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/shell/Dialog.test.tsx` sonuna:

```tsx
test("only the topmost dialog handles Esc and Tab", () => {
  const outer = vi.fn();
  const inner = vi.fn();
  const { rerender } = render(
    <Dialog title="Dış" onClose={outer}>
      <button>dış</button>
    </Dialog>,
  );
  // İç pencere sonra açılır (gerçek akış: onay penceresi formun üstünde).
  rerender(
    <Dialog title="Dış" onClose={outer}>
      <button>dış</button>
      <Dialog title="İç" onClose={inner}>
        <button>iç</button>
      </Dialog>
    </Dialog>,
  );
  expect(document.activeElement).toBe(button("iç"));
  expect(tab(button("iç"))).toBe(true);
  expect(document.activeElement).toBe(button("iç"));
  fireEvent.keyDown(button("iç"), { key: "Escape" });
  expect(inner).toHaveBeenCalledOnce();
  expect(outer).not.toHaveBeenCalled();
});
```

`app/ui/birth/model.test.ts` sonuna:

```ts
test("a signed zero offset is zero, not negative zero", () => {
  expect(Object.is(parseFixedOffset("-0:00"), 0)).toBe(true);
  expect(Object.is(parseFixedOffset("+00:00"), 0)).toBe(true);
  expect(Object.is(parseFixedOffset("−00:00:00"), 0)).toBe(true);
});

test("errorField maps every field-bound backend code and nothing else", () => {
  const e = (code: string, params: Record<string, string> = {}) => ({ code, params });
  const table: [ReturnType<typeof e>, string | null][] = [
    [e("invalid_input", { field: "birth.date" }), "date"],
    [e("invalid_input", { field: "birth.time" }), "time"],
    [e("invalid_input", { field: "birth.time_certainty" }), "certainty"],
    [e("invalid_input", { field: "birth.place.lat" }), "lat"],
    [e("invalid_input", { field: "birth.place.lon" }), "lon"],
    [e("invalid_input", { field: "name" }), "name"],
    [e("invalid_input", { field: "label" }), "label"],
    [e("invalid_input", { field: "notes" }), null],
    [e("nonexistent_local_time", { local: "x", suggestion: "y" }), "time"],
    [e("ambiguous_local_time"), "fold"],
    [e("unknown_time_zone"), "tz"],
    [e("offset_out_of_range"), "tz"],
    [e("longitude_out_of_range"), "lon"],
    [e("date_out_of_range"), "date"],
    [e("atlas_unavailable", { reason: "missing" }), null],
    [e("task_failed"), null],
  ];
  for (const [error, field] of table) expect([error.code, errorField(error)]).toEqual([error.code, field]);
});
```

`app/ui/birth/PlaceSearch.test.tsx` sonuna:

```tsx
test("Enter while the list belongs to the previous text is consumed and selects nothing", async () => {
  const onSelect = vi.fn();
  render(<Harness onSelect={onSelect} />);
  await type("new delhi");
  expect(screen.getAllByRole("option")).toHaveLength(1);
  vi.mocked(api.searchPlaces).mockReturnValueOnce(new Promise(() => {}));
  await type("new delhi x");
  // Liste hâlâ "new delhi"nin sonucunu gösteriyor; yanıt gelmedi.
  expect(fireEvent.keyDown(box(), { key: "Enter" })).toBe(false);
  expect(onSelect).not.toHaveBeenCalled();
  expect(screen.getByRole("listbox")).toHaveAttribute("aria-busy", "true");
});
```

`app/ui/birth/BirthForm.test.tsx`:

`vi.mock` nesnesine `updateClient: vi.fn()` ekle; `beforeEach` sonuna:

```ts
  vi.mocked(api.updateClient).mockImplementation(async (id, name, gender) => ({ status: "ok", data: { ...fixture().opened.client, id, name, gender } }));
  vi.mocked(api.addChart).mockResolvedValue({ status: "ok", data: fixture().opened });
```

"an ambiguous time asks which occurrence and sends the choice" testinde `getByRole("group", …)` satırını şu dört satırla değiştir (denetimler seçimden önce yapılır; seçimden sonraki önizleme hatasızdır):

```tsx
  const group = within(dialog()).getByRole("radiogroup", { name: "Bu saat iki kez yaşandı" });
  expect(group).toHaveAttribute("aria-invalid", "true");
  expect(group).toHaveAccessibleDescription("2014-10-26T03:30:00 iki kez yaşandı (yaz saati dönüşü); hangisi olduğunu seçin.");
  expect(within(dialog()).getByRole("region", { name: "Önizleme" })).toHaveTextContent("Bu saat iki kez yaşandı; aşağıdan hangisi olduğunu seçin.");
```

"save errors land on their field or above the buttons; Esc closes" testinin adını `"save errors land on their field or above the buttons"` yap ve son iki satırını (`fireEvent.keyDown(… "Escape")` ve `expect(onClose).toHaveBeenCalledOnce()`) sil; Esc yeni teste taşınır.

Dosya sonuna:

```tsx
test("gender: a new client sends the chosen gender", async () => {
  renderForm("new");
  await fillDelhi();
  fireEvent.change(input("Cinsiyet"), { target: { value: "Female" } });
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(vi.mocked(api.createClient).mock.lastCall![0].gender).toBe("Female");
});

test("editing a gender calls update_client after the chart is saved and opens the updated client", async () => {
  renderForm("edit");
  expect(input("Cinsiyet")).toHaveValue("");
  fireEvent.change(input("Cinsiyet"), { target: { value: "Male" } });
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(api.updateChart).toHaveBeenCalledOnce();
  expect(api.updateClient).toHaveBeenCalledWith(1, "Örnek Danışan", "Male");
  expect(vi.mocked(api.updateChart).mock.invocationCallOrder[0]).toBeLessThan(vi.mocked(api.updateClient).mock.invocationCallOrder[0]);
  expect(onClose).toHaveBeenCalledOnce();
});

test("an unchanged gender does not call update_client; add mode locks it", async () => {
  const first = renderForm("edit");
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(api.updateClient).not.toHaveBeenCalled();
  first.unmount();
  renderForm("add");
  expect(input("Cinsiyet")).toBeDisabled();
});

test("submitting with errors focuses the first invalid field and announces the count", async () => {
  renderForm("new");
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(document.activeElement).toBe(input("Danışan adı"));
  // Yer kutusunun durum satırı da `role="status"`dır; özet görünmez satırdır.
  expect(dialog().querySelector('p.visually-hidden[role="status"]')).toHaveTextContent(/^Düzeltilmesi gereken \d+ alan var\.$/);
  fireEvent.change(input("Danışan adı"), { target: { value: "Ayşe" } });
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(document.activeElement).toBe(input("Tarih"));
});

test("changing the date clears the chosen occurrence of an ambiguous time", async () => {
  vi.mocked(api.previewMoment).mockResolvedValueOnce({
    status: "error",
    error: { code: "ambiguous_local_time", params: { local: "2014-10-26T03:30:00", earlier_offset_secs: "10800", later_offset_secs: "7200" } },
  });
  renderForm("new");
  await fillDelhi();
  fireEvent.click(within(dialog()).getByRole("radio", { name: "İkinci kez (geçişten sonra)" }));
  fireEvent.change(input("Tarih"), { target: { value: "2000-01-02" } });
  await wait(PREVIEW_DEBOUNCE_MS);
  expect(vi.mocked(api.previewMoment).mock.lastCall![0].fold).toBeNull();
});

test("Esc on an untouched form closes at once; with changes it asks first", async () => {
  const first = renderForm("edit");
  fireEvent.keyDown(input("Tarih"), { key: "Escape" });
  expect(onClose).toHaveBeenCalledOnce();
  first.unmount();
  onClose.mockReset();
  renderForm("new");
  fireEvent.change(input("Danışan adı"), { target: { value: "Ayşe" } });
  fireEvent.keyDown(input("Danışan adı"), { key: "Escape" });
  expect(onClose).not.toHaveBeenCalled();
  const confirm = screen.getByRole("dialog", { name: "Değişiklikler atılsın mı?" });
  // Onay penceresinde Esc yalnızca onu kapatır; form ve taslak yerinde.
  fireEvent.keyDown(within(confirm).getByRole("button", { name: "Vazgeç" }), { key: "Escape" });
  expect(screen.queryByRole("dialog", { name: "Değişiklikler atılsın mı?" })).toBeNull();
  expect(input("Danışan adı")).toHaveValue("Ayşe");
  expect(onClose).not.toHaveBeenCalled();
  fireEvent.keyDown(input("Danışan adı"), { key: "Escape" });
  fireEvent.click(within(screen.getByRole("dialog", { name: "Değişiklikler atılsın mı?" })).getByRole("button", { name: "Değişiklikleri at" }));
  expect(onClose).toHaveBeenCalledOnce();
});
```

Bu adımda dosyaya eklenen test sayısı altıdır (cinsiyet ×3, odak, `fold` sıfırlama, Esc).

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/birth ui/shell/Dialog.test.tsx
```
Expected: Dialog: `inner` ve `outer` ikisi de çağrılır (`expected "spy" to not be called`); model: `Object.is(-0, 0)` `false`; PlaceSearch: `fireEvent.keyDown` `true` döner ya da `onSelect` çağrılır; BirthForm: `Unable to find a label with the text of: Cinsiyet`, `getByRole("radiogroup")` bulunamaz, odak `body`'de, `fold` `"Later"` kalır, Esc'te `onClose` hemen çağrılır.

- [ ] **Step 3: Uygula**

`app/ui/shell/Dialog.tsx` — `onKeyDown` içinde `if (!root) return;` satırının hemen altına:

```tsx
      // İç içe pencerede (formun üstünde onay) tuşları yalnızca en üstteki işler.
      const modals = document.querySelectorAll('[role="dialog"][aria-modal="true"]');
      if (modals[modals.length - 1] !== root) return;
```

Dosya başındaki yorumun sonuna bir satır ekle: `// İç içe açılan pencerelerde Esc ve Tab'ı yalnızca en son açılan işler.`

`app/ui/birth/model.ts` — `parseFixedOffset`'in dönüş satırını değiştir:

```ts
  // `|| 0`: "-0:00" negatif sıfır değil sıfırdır (JSON'da ve karşılaştırmada fark yaratmasın).
  return (m[1] === "+" ? 1 : -1) * (h * 3600 + min * 60 + s) || 0;
```

`app/ui/birth/PlaceSearch.tsx`:

`const [query, setQuery] = useState<string | null>(null);` satırının altına:

```tsx
  // Liste hangi metnin yanıtı: yazım sürüp yanıt gelmediyse liste eskidir.
  const [answered, setAnswered] = useState<string | null>(null);
  const stale = query !== null && query !== answered;
```

Arama yanıtında `if (mine !== seq.current) return;` satırının altına `setAnswered(query);` ekle.

`onKeyDown`'daki Enter dalını değiştir:

```tsx
    } else if (e.key === "Enter" && shown) {
      // Görünen liste önceki metnin sonucuysa Enter tüketilir ama seçmez (eski yer seçilmez).
      e.preventDefault();
      if (!stale && active >= 0 && hits[active]) choose(hits[active]);
```

Listeye (`<ul … role="listbox">`) `aria-busy={stale || undefined}` ekle. Dosya başındaki yorumun dördüncü satırını şöyle genişlet: `// tutulur; liste görünmüyorken Enter ve Esc tüketilmez (Esc pencereye gider); liste önceki metnin sonucuyken Enter seçmez.`

`app/ui/birth/BirthForm.tsx`:

İçe aktarmalara `import type { Gender } from "../bindings";` (mevcut `import type { CommandError, MomentPreview, OpenedClient }` satırına `Gender` ekle) ve `import { ConfirmDialog } from "../shell/ConfirmDialog";` ekle.

`const [draft, setDraft] = useState<BirthDraft>(() => initialDraft(mode, target));` satırını şunlarla değiştir:

```tsx
  // Açılıştaki taslak: Esc'te değişiklik var mı diye bununla karşılaştırılır.
  const [start] = useState<BirthDraft>(() => initialDraft(mode, target));
  const [draft, setDraft] = useState<BirthDraft>(start);
  const dirty = JSON.stringify(draft) !== JSON.stringify(start);
  const [confirmClose, setConfirmClose] = useState(false);
  const [errorCount, setErrorCount] = useState(0);
  const formRef = useRef<HTMLFormElement>(null);
  const focusInvalid = useRef(false);
```

`blur` tanımının altına:

```tsx
  // Gönderimden sonra odak ilk hatalı alana gider (M6-1); katlama grubunda ilk seçeneğe.
  useEffect(() => {
    if (!focusInvalid.current) return;
    focusInvalid.current = false;
    formRef.current?.querySelector<HTMLElement>('input[aria-invalid="true"], [role="radiogroup"][aria-invalid="true"] input')?.focus();
  });

  const requestClose = () => (dirty ? setConfirmClose(true) : onClose());
```

`submit`'i değiştir:

```tsx
  const submit = async (e: FormEvent) => {
    e.preventDefault();
    if (saving) return;
    setFormError(null);
    const { birth, errors } = toBirthData(draft);
    if (!birth) {
      setShown(errors);
      setErrorCount(Object.keys(errors).length);
      focusInvalid.current = true;
      return;
    }
    setErrorCount(0);
    setSaving(true);
    const r =
      editing && target
        ? await api.updateChart(target.chart_record.id, toNewChart(draft, birth))
        : adding && target
          ? await api.addChart(target.client.id, toNewChart(draft, birth))
          : await api.createClient(toNewClient(draft, birth));
    if (r.status === "ok" && editing && target && draft.gender !== target.client.gender) {
      // Cinsiyet danışana aittir: harita kaydından sonra yazılır, dönen danışan açılana konur.
      const c = await api.updateClient(target.client.id, target.client.name, draft.gender);
      setSaving(false);
      if (c.status === "error") {
        session.open(r.data);
        setFormError(c.error);
        return;
      }
      session.open({ ...r.data, client: c.data });
      onClose();
      return;
    }
    setSaving(false);
    if (r.status === "ok") {
      session.open(r.data);
      onClose();
      return;
    }
    const f = errorField(r.error);
    if (f) {
      setShown((s) => ({ ...s, [f]: { error: r.error } }));
      setErrorCount(1);
      focusInvalid.current = true;
    } else setFormError(r.error);
  };
```

`<Dialog … onClose={onClose} variant="sheet">` içinde `onClose={requestClose}` yap ve `<form` etiketine `ref={formRef}` ekle.

"Harita etiketi" alanından sonra (harita türü `<div className="form-field">`'ından önce) cinsiyet alanı:

```tsx
        <div className="form-field">
          <label htmlFor={`${id}-gender`}>{t("gender.label")}</label>
          <select
            id={`${id}-gender`}
            value={draft.gender ?? ""}
            disabled={adding}
            onChange={(e) => update({ gender: e.target.value === "" ? null : (e.target.value as Gender) })}
          >
            <option value="">{t("gender.none")}</option>
            <option value="Male">{t("gender.Male")}</option>
            <option value="Female">{t("gender.Female")}</option>
            <option value="Other">{t("gender.Other")}</option>
          </select>
        </div>
```

Tarih girişinin `onChange`'ini değiştir:

```tsx
            <input {...p} type="date" value={draft.date} onChange={(e) => update({ date: e.target.value, fold: null }, "date")} onBlur={blur("date")} />
```

Katlama `fieldset`'ini değiştir:

```tsx
        {ambiguous && (
          <fieldset
            className="form-field"
            role="radiogroup"
            aria-labelledby={`${id}-fold-legend`}
            aria-invalid={fieldError("fold") ? true : undefined}
            aria-describedby={fieldError("fold") ? `${id}-fold-error` : undefined}
          >
            <legend id={`${id}-fold-legend`}>{t("birth.fold")}</legend>
            {(["Earlier", "Later"] as const).map((f) => (
              <label key={f} className="choice">
                <input
                  type="radio"
                  name={`${id}-fold`}
                  checked={draft.fold === f}
                  aria-invalid={fieldError("fold") ? true : undefined}
                  onChange={() => update({ fold: f }, "fold")}
                />
                {t(`birth.fold${f}`)}
              </label>
            ))}
            {fieldError("fold") && (
              <p id={`${id}-fold-error`} className="field-error">
                <span aria-hidden="true">▲ </span>
                {fieldError("fold")}
              </p>
            )}
          </fieldset>
        )}
```

(`role` değişince bazı ekran okuyucular adı `legend`'dan almaz; ad `aria-labelledby` ile bağlanır. Radyo girişlerindeki `aria-invalid` odak seçicisinin ilk seçeneği bulması içindir; grup `role="radiogroup"` olduğu için `aria-invalid` geçerli bir özelliktir.)

Önizleme bölümünde (`<section className="birth-preview" …>`) `{previewError && previewField === null && …}` satırının altına:

```tsx
          {previewField === "fold" && <p className="field-hint">{t("birth.foldHint")}</p>}
```

`{formError && (…)}` bloğunun üstüne:

```tsx
        <p className="visually-hidden" role="status">
          {errorCount > 0 ? t("birth.errorSummary", { count: errorCount }) : ""}
        </p>
```

`</form>`'dan sonra, `</Dialog>`'dan önce:

```tsx
      {confirmClose && (
        <ConfirmDialog
          title={t("birth.discardTitle")}
          message={t("birth.discardMessage")}
          confirmLabel={t("birth.discardConfirm")}
          onConfirm={onClose}
          onCancel={() => setConfirmClose(false)}
        />
      )}
```

Dosya başındaki yorumun sonuna: `// Değişmiş taslakta Esc onay sorar; "Vazgeç" düğmesi sormadan kapatır.`

i18n — `tr.ts` `birth` içine:

```ts
    foldHint: "Bu saat iki kez yaşandı; aşağıdan hangisi olduğunu seçin.",
    errorSummary: "Düzeltilmesi gereken {{count}} alan var.",
    discardTitle: "Değişiklikler atılsın mı?",
    discardMessage: "Formdaki kaydedilmemiş değişiklikler kaybolur.",
    discardConfirm: "Değişiklikleri at",
```

`en.ts` `birth` içine:

```ts
    foldHint: "This time occurred twice; choose which one below.",
    errorSummary: "Fields to correct: {{count}}.",
    discardTitle: "Discard your changes?",
    discardMessage: "The unsaved changes in the form will be lost.",
    discardConfirm: "Discard changes",
```

`fr.ts` `birth` içine:

```ts
    foldHint: "Cette heure a eu lieu deux fois ; choisissez laquelle ci-dessous.",
    errorSummary: "Champs à corriger : {{count}}.",
    discardTitle: "Abandonner les modifications ?",
    discardMessage: "Les modifications non enregistrées du formulaire seront perdues.",
    discardConfirm: "Abandonner",
```

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/birth ui/shell
pnpm --dir app test && pnpm --dir app build
pnpm --dir app e2e
rm -rf app/test-results
```
Expected: Dialog +1, model +2, PlaceSearch +1, BirthForm +6 geçer; toplam `Test Files  52 passed (52)`, `Tests  422 passed (422)`; build `✓ built`; e2e `6 passed` (e2e akışları kaydederek kapatır; Esc onayına girmez).

- [ ] **Step 5: Commit**

```bash
git add app/ui/birth app/ui/shell/Dialog.tsx app/ui/shell/Dialog.test.tsx app/ui/i18n
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): gender in the birth form, announced errors and radiogroup fold (M6-1), Esc asks before discarding

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---
### Task 12: Erişilebilirlik geçişi (M6-7): ilk boyamadaki bildirim, `≈`, devre dışı menü öğesinin nedeni, ortak sınıflar, tek sekme durağı, odak dönüşü, meşgul düğmeler, palet test boşlukları

Plan 10 madde 10, 18, 19 ve madde 26'nın odak/meşgul/`.button-danger` parçaları; Plan 10 Task 8 kararıyla ertelenen palet test boşlukları. Davranış değişiklikleri:

1. `NoticeBar` ilk boyamada bildirim içermez; bildirimler bileşen bağlandıktan sonra eklenir. Açılışta var olan `role="alert"`/`role="status"` öğesi eklendiği anda duyurulur (ilk boyamada var olan canlı bölge çoğu ekran okuyucuda duyurulmaz).
2. Yaklaşık değerde `title` görünmez olan `≈` işaretine taşınır; ekran okuyucu "yaklaşık"ı bir kez okur (görünmez metin), `title` açıklama olarak ikinci kez okunmaz.
3. Düzen menüsünün devre dışı öğeleri `disabled` yerine `aria-disabled="true"` ve nedenine `aria-describedby` taşır; oklar onlara da gider (WAI-ARIA menü deseni), etkinleştirme bir şey yapmaz.
4. `.visually-hidden` `theme/tokens.css`'e, `.button-danger` `shell/shell.css`'e taşınır (ikisi de her ekranda gerekir).
5. Kenar çubuğu danışan listesi tek sekme durağıdır (roving tabindex): ↑/↓/Home/End satırlar arasında gezer; sekme durağı açık danışandadır, yoksa ilk satırda.
6. Onaylı silmeden sonra odak kararlı bir hedefe gider: not silinince "Yeni not" düğmesine, harita silinince açık danışan başlığına (`tabIndex={-1}`), danışan silinince kenar çubuğu arama kutusuna (`CLIENT_SEARCH_ID`).
7. Veri düğmeleri meşgulken `aria-disabled` ve "Çalışıyor…" durum metni taşır (odak düğmede durabilir; JSON penceresi kapanınca odak devre dışı olmayan düğmeye döner). Meşgul durumu modül düzeyindedir: kenar çubuğu kapanıp açılınca sıfırlanmaz, ikinci dosya penceresi açılmaz.
8. `⌘⇧N` odak notlar panelindeyken paneli kapatırsa odak `main`'e (`tabIndex={-1}`) gider. `leaveSidebar` ve yeni `leaveNotes` `shell/focus.ts`'e taşınır.

**Files:**
- Create: `app/ui/shell/notices.test.tsx`
- Create: `app/ui/shell/focus.ts`
- Create: `app/ui/shell/focus.test.ts`
- Modify: `app/ui/shell/notices.tsx`
- Modify: `app/ui/shell/Shell.tsx`
- Modify: `app/ui/shell/LayoutPicker.tsx`, `app/ui/shell/LayoutPicker.test.tsx`
- Modify: `app/ui/shell/shell.css`, `app/ui/clients/clients.css`, `app/ui/dashboard/dashboard.css`, `app/ui/theme/tokens.css`, `app/ui/theme/tokens.test.ts`
- Modify: `app/ui/panels/GrahaTablePanel.tsx`, `app/ui/panels/GrahaTablePanel.test.tsx`
- Modify: `app/ui/clients/ClientList.tsx`, `app/ui/clients/OpenClient.tsx`, `app/ui/clients/DataActions.tsx`
- Modify: `app/ui/clients/Sidebar.test.tsx`, `app/ui/clients/DataActions.test.tsx`
- Modify: `app/ui/notes/NotesPanel.tsx`, `app/ui/notes/NotesPanel.test.tsx`
- Modify: `app/ui/palette/Palette.test.tsx`
- Modify: `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`

**Interfaces:**
- Consumes: `SIDEBAR_ID` (`clients/Sidebar.tsx`); `useLayoutStore().toggleNotes()`; `ConfirmDialog` odak geri yüklemesi (`Dialog` temizliği önceki odağı geri verir).
- Produces: `CLIENT_SEARCH_ID = "client-search"` (`clients/ClientList.tsx`); `MAIN_ID = "shell-main"` ve `leaveSidebar(): void`, `leaveNotes(): void` (`shell/focus.ts`); `resetExchangeBusy(): void` (`clients/DataActions.tsx`, yalnızca testler); i18n `layout.builtinLocked`, `data.working`.

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/shell/notices.test.tsx`:

```tsx
import { act, render, screen } from "@testing-library/react";
import { renderToStaticMarkup } from "react-dom/server";
import { NoticeBar, NoticeProvider, type Notice } from "./notices";

const boot: Notice = { id: "boot", type: "error", messageKey: "notice.openFailed", error: { code: "store_sqlite", params: { detail: "disk" } } };

test("a notice present at start is added after the first paint, so it is announced", async () => {
  const tree = (
    <NoticeProvider initial={[boot]}>
      <NoticeBar />
    </NoticeProvider>
  );
  // İlk boyama (sunucu çıktısı ilk render'a eşittir): bant var, bildirim yok.
  const first = renderToStaticMarkup(tree);
  expect(first).not.toContain('role="alert"');
  render(tree);
  await act(async () => {});
  expect(screen.getByRole("alert")).toHaveTextContent("disk");
});
```

(`NoticeProvider` açılış bildirimlerini `initial` ile alır; `App.tsx` açılış hatalarını bu yolla verir.)

`app/ui/shell/focus.test.ts`:

```ts
import { leaveNotes, leaveSidebar, MAIN_ID } from "./focus";

afterEach(() => (document.body.innerHTML = ""));

test("leaving the notes panel moves focus to main; focus elsewhere is left alone", () => {
  document.body.innerHTML = `<main id="${MAIN_ID}" tabindex="-1"><div class="notes-panel"><textarea></textarea></div><button>diğer</button></main>`;
  const box = document.querySelector("textarea")!;
  box.focus();
  leaveNotes();
  expect(document.activeElement).toBe(document.getElementById(MAIN_ID));
  const other = document.querySelector("button")!;
  other.focus();
  leaveNotes();
  expect(document.activeElement).toBe(other);
});

test("leaving the sidebar moves focus to its toggle", () => {
  document.body.innerHTML = `<button aria-controls="client-sidebar">aç</button><aside id="client-sidebar"><input /></aside>`;
  document.querySelector("input")!.focus();
  leaveSidebar();
  expect(document.activeElement).toBe(document.querySelector("button"));
});
```

`app/ui/theme/tokens.test.ts` sonuna (dosya başına içe aktarmaları ekle):

```ts
import clientsCss from "../clients/clients.css?raw";
import dashboardCss from "../dashboard/dashboard.css?raw";
import shellCss from "../shell/shell.css?raw";

test("shared helper classes live in the stylesheets every screen loads", () => {
  expect(css).toMatch(/\.visually-hidden\s*\{/);
  expect(dashboardCss).not.toMatch(/\.visually-hidden\s*\{/);
  expect(shellCss).toMatch(/\.button-danger\s*\{/);
  expect(clientsCss).not.toMatch(/\.button-danger\s*\{/);
});
```

`app/ui/panels/GrahaTablePanel.test.tsx` "unknown birth time hides the lagna row…" testinde `expect(moon.querySelector('[title="yaklaşık"]')).not.toBeNull();` satırının altına:

```tsx
  // `title` yalnızca görünmez işarettedir: "yaklaşık" ekran okuyucuda bir kez okunur.
  expect(moon.querySelector('[title="yaklaşık"]')).toHaveAttribute("aria-hidden", "true");
```

`app/ui/shell/LayoutPicker.test.tsx`:

"builtin layouts can be reset but not renamed or deleted" testindeki iki `toBeDisabled()` satırını değiştir:

```tsx
  expect(item("Yeniden adlandır…")).toHaveAttribute("aria-disabled", "true");
  expect(item("Yeniden adlandır…")).toHaveAccessibleDescription("Hazır düzenler yeniden adlandırılamaz ve silinemez.");
  expect(item("Sil…")).toHaveAttribute("aria-disabled", "true");
  fireEvent.click(item("Sil…"));
  expect(screen.queryByText(/düzeni silinsin mi\?/)).toBeNull();
  expect(screen.getByRole("menu")).toBeInTheDocument();
```

"save as and duplicate are disabled at the layout limit" testindeki son üç satırı değiştir:

```tsx
  expect(item("Farklı kaydet…")).toHaveAttribute("aria-disabled", "true");
  expect(item("Kopyala")).toHaveAttribute("aria-disabled", "true");
  expect(item("Kopyala")).toHaveAccessibleDescription(`En çok ${MAX_LAYOUTS} düzen kaydedilebilir.`);
  fireEvent.click(item("Kopyala"));
  expect(options()).toHaveLength(MAX_LAYOUTS);
```

"arrows, Home and End rove over enabled items only" testini şununla değiştir:

```tsx
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
```

(Menü öğelerinin sırası `LayoutPicker.tsx`'teki sıradır: Farklı kaydet, Yeniden adlandır, Kopyala, Sil, Varsayılana döndür. Sil onayı satır içi bir kiptir; devre dışı "Sil…" tıklanınca onay metni görünmez ve menü açık kalır.)

`app/ui/clients/Sidebar.test.tsx` sonuna:

```tsx
test("the client list is one tab stop; arrows, Home and End move between clients", async () => {
  vi.mocked(api.listClients).mockResolvedValue(ok([opened().client, ayse, client(3, "Can Demir")]));
  renderSidebar();
  const buttons = await within(list()).findAllByRole("button");
  expect(buttons.map((b) => b.tabIndex)).toEqual([0, -1, -1]);
  buttons[0].focus();
  fireEvent.keyDown(buttons[0], { key: "ArrowDown" });
  expect(document.activeElement).toBe(buttons[1]);
  fireEvent.keyDown(buttons[1], { key: "End" });
  expect(document.activeElement).toBe(buttons[2]);
  expect(buttons.map((b) => b.tabIndex)).toEqual([-1, -1, 0]);
  fireEvent.keyDown(buttons[2], { key: "ArrowDown" });
  expect(document.activeElement).toBe(buttons[2]);
  fireEvent.keyDown(buttons[2], { key: "Home" });
  expect(document.activeElement).toBe(buttons[0]);
});

test("after deleting a chart focus goes to the client heading, after deleting the client to the search box", async () => {
  const charts = [chart(1, ""), chart(5, "Düzeltilmiş", "2000-01-02")];
  vi.mocked(api.listCharts).mockResolvedValueOnce(ok(charts)).mockResolvedValue(ok([charts[0]]));
  vi.mocked(api.deleteChart).mockResolvedValue(ok(null));
  vi.mocked(api.deleteClient).mockResolvedValue(ok(null));
  renderSidebar();
  fireEvent.click(await within(sidebar()).findByRole("button", { name: "Haritayı sil: “Düzeltilmiş”" }));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Harita silinsin mi?" })).getByRole("button", { name: "Haritayı sil" }));
  await waitFor(() => expect(document.activeElement).toBe(within(sidebar()).getByRole("heading", { name: "Örnek Danışan" })));
  fireEvent.click(within(sidebar()).getByRole("button", { name: "Danışanı sil…" }));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Danışan silinsin mi?" })).getByRole("button", { name: "Danışanı sil" }));
  await waitFor(() => expect(document.activeElement).toBe(within(sidebar()).getByRole("searchbox", { name: "Danışan ara" })));
});
```

`app/ui/notes/NotesPanel.test.tsx` sonuna:

```tsx
test("after a note is deleted focus goes to New note", async () => {
  vi.mocked(api.deleteNote).mockResolvedValue(ok(null));
  renderPanel();
  const items = await within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).findAllByRole("listitem");
  fireEvent.click(within(items[1]).getByRole("button", { name: /tarihli notu sil/ }));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Not silinsin mi?" })).getByRole("button", { name: "Notu sil" }));
  await waitFor(() => expect(document.activeElement).toBe(screen.getByRole("button", { name: "Yeni not" })));
});
```

`app/ui/clients/DataActions.test.tsx`: `import { DataActions, resetExchangeBusy } from "./DataActions";`; `renderData` `renderWithStores(…)`'ın sonucunu döndürsün (`return renderWithStores(…)`); `beforeEach`'in başına `resetExchangeBusy();`. "an import with reset settings is a warning…" testindeki `await waitFor(() => expect(button).toBeEnabled());` satırını `await waitFor(() => expect(button).not.toHaveAttribute("aria-disabled"));` yap (düğme artık `disabled` olmaz; bekleme meşgulün bitmesini beklemelidir). Sonuna:

```tsx
test("while a file dialog is open the buttons say why and nothing else starts", async () => {
  let release!: () => void;
  vi.mocked(api.exportClientsCsv).mockReturnValue(new Promise((r) => (release = () => r(ok(null)))));
  renderData();
  const csv = within(section()).getByRole("button", { name: "CSV olarak dışa aktar…" });
  fireEvent.click(csv);
  await waitFor(() => expect(csv).toHaveAttribute("aria-disabled", "true"));
  expect(within(section()).getByRole("status")).toHaveTextContent("Çalışıyor…");
  fireEvent.click(csv);
  fireEvent.click(within(section()).getByRole("button", { name: "JSON içe aktar…" }));
  fireEvent.click(within(section()).getByRole("button", { name: "JSON olarak dışa aktar…" }));
  expect(api.exportClientsCsv).toHaveBeenCalledOnce();
  expect(api.importClientsJson).not.toHaveBeenCalled();
  expect(screen.queryByRole("dialog")).toBeNull();
  release();
  await waitFor(() => expect(csv).not.toHaveAttribute("aria-disabled"));
  expect(within(section()).getByRole("status")).toHaveTextContent(/^$/);
});

test("busy survives closing and reopening the sidebar while a file dialog is open", async () => {
  let release!: () => void;
  vi.mocked(api.importClientsJson).mockReturnValue(new Promise((r) => (release = () => r(ok(null)))));
  const first = renderData();
  fireEvent.click(within(section()).getByRole("button", { name: "JSON içe aktar…" }));
  await waitFor(() => expect(within(section()).getByRole("button", { name: "JSON içe aktar…" })).toHaveAttribute("aria-disabled", "true"));
  first.unmount();
  renderData();
  const again = within(section()).getByRole("button", { name: "JSON içe aktar…" });
  expect(again).toHaveAttribute("aria-disabled", "true");
  fireEvent.click(again);
  expect(api.importClientsJson).toHaveBeenCalledOnce();
  release();
  await waitFor(() => expect(again).not.toHaveAttribute("aria-disabled"));
});
```

`app/ui/palette/Palette.test.tsx`: `import { useState } from "react";` ekle. Sonuna:

```tsx
test("Esc closes the palette and focus returns to what was focused before it opened", async () => {
  function Host() {
    const [open, setOpen] = useState(false);
    return (
      <>
        <button type="button" onClick={() => setOpen(true)}>
          aç
        </button>
        {open && <Palette onClose={() => setOpen(false)} onRun={onRun} />}
      </>
    );
  }
  renderWithStores(<Host />, { opened: opened() });
  const opener = screen.getByRole("button", { name: "aç" });
  opener.focus();
  fireEvent.click(opener);
  await waitFor(() => expect(document.activeElement).toBe(box()));
  fireEvent.keyDown(box(), { key: "Escape" });
  expect(screen.queryByRole("combobox")).toBeNull();
  expect(document.activeElement).toBe(opener);
});

test("an older search reply that arrives after a newer one is dropped", async () => {
  let older: (r: Awaited<ReturnType<typeof api.searchClients>>) => void = () => {};
  vi.mocked(api.searchClients).mockReturnValueOnce(new Promise((resolve) => (older = resolve)));
  vi.mocked(api.searchClients).mockResolvedValueOnce(ok([{ client: client(2, "Ayşe Yılmaz"), matched: "Name", snippet: null }]));
  renderPalette();
  fireEvent.change(box(), { target: { value: "ayş" } });
  await waitFor(() => expect(api.searchClients).toHaveBeenCalledTimes(1));
  fireEvent.change(box(), { target: { value: "ayşe" } });
  await waitFor(() => expect(options()[0]).toHaveTextContent("Ayşe Yılmaz"));
  await act(async () => older(ok([{ client: client(3, "Ayla Demir"), matched: "Name", snippet: null }])));
  expect(screen.queryByText("Ayla Demir")).toBeNull();
  expect(options()[0]).toHaveTextContent("Ayşe Yılmaz");
});
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/shell ui/theme ui/panels/GrahaTablePanel.test.tsx ui/clients ui/notes ui/palette
```
Expected: `notices.test.tsx` ilk çıktıda `role="alert"` bulur (`expected … not to contain 'role="alert"'`); `focus.test.ts` `Failed to resolve import "./focus"`; tokens testi `.visually-hidden` bulamaz; Graha `aria-hidden` yok; LayoutPicker `aria-disabled` yok; Sidebar: sekme sırası `[0, 0, 0]`, odak `body`; Notes odak `body`; DataActions `aria-disabled` yok; palet Esc testi geçebilir (Dialog odak geri verir; bu test borç kapatmadır), eski yanıt testi geçer (sıra koruması var; borç kapatma).

- [ ] **Step 3: Uygula**

`app/ui/shell/notices.tsx` — `NoticeBar`'ı değiştir (`useEffect`, `useState` içe aktar):

```tsx
export function NoticeBar() {
  const { t, i18n } = useTranslation();
  const { notices, dismiss } = useNotices();
  // İlk boyamada var olan canlı bölge çoğu ekran okuyucuda duyurulmaz: açılış bildirimleri
  // bileşen bağlandıktan sonra eklenir ve eklendikleri anda duyurulur.
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  if (notices.length === 0) return null;
  return (
    <section className="notice-bar no-print" aria-label={t("notice.region")}>
      {mounted &&
        notices.map((n) => (
          <div key={n.id} className={`notice notice--${n.type}`} role={n.type === "error" ? "alert" : "status"}>
            <strong className="notice-kind">{t(`notice.${n.type}`)}</strong>
            <span className="notice-text">
              {n.type === "error" ? t(n.messageKey, { message: errorMessage(t, i18n, n.error) }) : t(n.messageKey, n.params)}
            </span>
            {n.type !== "error" && n.action === "tzdb" && <TzdbActions />}
            <button type="button" onClick={() => dismiss(n.id)}>
              {t("notice.dismiss")}
            </button>
          </div>
        ))}
    </section>
  );
}
```

(`TzdbActions` satırı Task 6'dandır, değişmez; yeni olan yalnızca `mounted &&` sarmalıdır.) Bildirimleri hemen bekleyen mevcut testler `findBy…`/`waitFor` kullanır; ilk render'dan hemen sonra `getByRole("alert")` ile arayan bir test düşerse onu `await screen.findByRole("alert")` yap (bildirim bir etki turu sonra eklenir).

`app/ui/shell/focus.ts`:

```ts
// Kabuk düzeyinde odak taşıma: bir bölge gizlenmeden önce odak onun içindeyse kararlı bir
// hedefe geçer. Odak `body`'ye düşmez ve odaktaki alan (etiket, not) çıkışta kaydını yapar;
// kaldırılan öğe odak çıkışı olayı almaz.
import { SIDEBAR_ID } from "../clients/Sidebar";

export const MAIN_ID = "shell-main";

/** Kenar çubuğu gizlenmeden önce: odak içindeyse açma düğmesine geçer. */
export function leaveSidebar() {
  if (document.getElementById(SIDEBAR_ID)?.contains(document.activeElement)) {
    document.querySelector<HTMLElement>(`[aria-controls="${SIDEBAR_ID}"]`)?.focus();
  }
}

/** Notlar paneli kapanmadan önce (`⌘⇧N`): odak paneldeyse ana bölgeye geçer. */
export function leaveNotes() {
  if (document.activeElement?.closest(".notes-panel")) document.getElementById(MAIN_ID)?.focus();
}
```

`app/ui/shell/Shell.tsx`: dosyadaki `leaveSidebar` işlevini ve üstündeki yorumu sil; `import { leaveNotes, leaveSidebar, MAIN_ID } from "./focus";` ekle; `SIDEBAR_ID` artık kullanılmıyorsa `import { Sidebar } from "../clients/Sidebar";` yap. Kısayol dinleyicisinde ve paletteki iki `toggleNotes` çağrısını sar:

```tsx
      if (command?.id === "toggleNotes" && !now.printView) {
        e.preventDefault();
        leaveNotes();
        now.toggleNotes();
```

```tsx
        if (item.command === "toggleNotes") {
          if (printView) return;
          leaveNotes();
          return toggleNotes();
        }
```

`<main className="shell-main">` → `<main id={MAIN_ID} className="shell-main" tabIndex={-1}>`. `shell.css` sonuna:

```css
.shell-main:focus {
  outline: none;
}
.button-danger {
  border: 1px solid var(--danger);
}
```

(`main` programla odak alır, klavye durağı değildir; çerçeve çizilmez. `:focus-visible` kuralı klavyeyle gelinen öğelerde çizer.)

`app/ui/clients/clients.css`'ten `.button-danger { … }` bloğunu sil. `app/ui/dashboard/dashboard.css`'ten `.visually-hidden { … }` bloğunu sil ve aynısını `app/ui/theme/tokens.css` sonuna ekle:

```css
.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  margin: -1px;
  padding: 0;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
  border: 0;
}
```

`app/ui/panels/GrahaTablePanel.tsx` — `Approx`'u değiştir:

```tsx
/** Yaklaşık değer: "≈" işareti (fare için `title`) ve ekran okuyucu için bir kez okunan metin. */
function Approx({ on, label, children }: { on: boolean; label: string; children: ReactNode }) {
  if (!on) return <>{children}</>;
  return (
    <span>
      <span aria-hidden="true" title={label}>
        ≈
      </span>
      <span className="visually-hidden">{label} </span>
      {children}
    </span>
  );
}
```

`app/ui/shell/LayoutPicker.tsx`:

`useId` içe aktarılmıyorsa ekle; bileşenin başına `const hintId = useId();`. `enabledItems`'ı değiştir (ad da değişir; dosyadaki üç kullanımı `menuItems` yap):

```tsx
  // Devre dışı öğeler de odaklanır (WAI-ARIA menü deseni); nedenleri `aria-describedby`'dadır.
  // Gerçekten `disabled` olan öğe (kullanıcı düzeninde "Varsayılana döndür") odak alamaz; gezintiye girmez (D22).
  const menuItems = () => [...(menuRef.current?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]:not(:disabled)') ?? [])];
```

`runItem`'in yanına:

```tsx
  /** `aria-disabled` öğe tıklanınca hiçbir şey yapmaz; menü açık kalır. */
  const unless = (off: boolean, action: () => void) => () => {
    if (!off) action();
  };
```

Dört öğeyi değiştir (`disabled` ve `title` kalkar):

```tsx
            <button
              type="button"
              role="menuitem"
              tabIndex={-1}
              aria-disabled={atCap || undefined}
              aria-describedby={atCap ? `${hintId}-cap` : undefined}
              onClick={unless(atCap, () => startMode("saveAs", uniqueCopyName()))}
            >
              {t("layout.saveAs")}
            </button>
```

```tsx
            <button
              type="button"
              role="menuitem"
              tabIndex={-1}
              aria-disabled={active.builtin || undefined}
              aria-describedby={active.builtin ? `${hintId}-builtin` : undefined}
              onClick={unless(active.builtin, () => startMode("rename", active.name))}
            >
              {t("layout.rename")}
            </button>
```

```tsx
            <button
              type="button"
              role="menuitem"
              tabIndex={-1}
              aria-disabled={atCap || undefined}
              aria-describedby={atCap ? `${hintId}-cap` : undefined}
              onClick={unless(atCap, () => runItem(() => store.duplicate(active.id, uniqueCopyName())))}
            >
              {t("layout.duplicate")}
            </button>
```

```tsx
            <button
              type="button"
              role="menuitem"
              tabIndex={-1}
              aria-disabled={active.builtin || undefined}
              aria-describedby={active.builtin ? `${hintId}-builtin` : undefined}
              onClick={unless(active.builtin, () => startMode("confirmDelete"))}
            >
              {t("layout.delete")}
            </button>
```

`</ul>`'den (menü listesinin kapanışı) hemen sonra, menü açıkken görünen iki görünmez açıklama:

```tsx
        <p id={`${hintId}-cap`} className="visually-hidden">
          {t("layout.limit", { max: MAX_LAYOUTS })}
        </p>
        <p id={`${hintId}-builtin`} className="visually-hidden">
          {t("layout.builtinLocked")}
        </p>
```

(Bu iki `<p>` menüyü saran `{menuOpen && (…)}` bloğunun içindeyse bloğu bir fragment'a (`<>…</>`) çevir.) `shell.css`'te `.layout-menu [role="menuitem"]:disabled` kuralı varsa seçiciyi `[aria-disabled="true"]` yap.

`app/ui/clients/ClientList.tsx`:

İçe aktarmaya `type KeyboardEvent` ekle (`import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";`). `SIDEBAR_LIMIT`'in altına:

```tsx
/** Kenar çubuğu arama kutusu: silmeden sonra odak buraya döner (OpenClient). */
export const CLIENT_SEARCH_ID = "client-search";
```

`const seq = useRef(0);` altına:

```tsx
  // Liste tek sekme durağıdır: ↑/↓/Home/End satırlarda gezer (roving tabindex).
  const [focusIndex, setFocusIndex] = useState(0);
  const rowsRef = useRef<HTMLUListElement>(null);
```

`const openId = session.opened?.client.id;` altına:

```tsx
  // Liste değişince sekme durağı açık danışana, yoksa ilk satıra gider.
  useEffect(() => {
    if (!rows) return;
    const i = rows.findIndex((r) => r.client.id === openId);
    setFocusIndex(i >= 0 ? i : 0);
  }, [rows, openId]);

  const onRowsKey = (e: KeyboardEvent<HTMLUListElement>) => {
    if (!rows || rows.length === 0) return;
    const last = rows.length - 1;
    const next =
      e.key === "ArrowDown" ? Math.min(focusIndex + 1, last) : e.key === "ArrowUp" ? Math.max(focusIndex - 1, 0) : e.key === "Home" ? 0 : e.key === "End" ? last : null;
    if (next === null) return;
    e.preventDefault();
    setFocusIndex(next);
    rowsRef.current?.querySelectorAll<HTMLButtonElement>("button.client-row")[next]?.focus();
  };
```

Arama kutusunda `id={`${id}-q`}` → `id={CLIENT_SEARCH_ID}`, `htmlFor={`${id}-q`}` → `htmlFor={CLIENT_SEARCH_ID}`. Satır listesi:

```tsx
        <ul ref={rowsRef} className="client-rows" onKeyDown={onRowsKey}>
          {rows.map(({ client, hit }, i) => (
            <li key={client.id}>
              <button
                type="button"
                className="client-row"
                tabIndex={i === focusIndex ? 0 : -1}
                aria-current={client.id === openId ? "true" : undefined}
                onFocus={() => setFocusIndex(i)}
                onClick={() => void choose(client)}
              >
```

(Düğmenin iç içeriği değişmez.) Dosya başındaki yoruma ekle: `// Liste tek sekme durağıdır; oklarla gezilir.`

`app/ui/clients/OpenClient.tsx`:

`import { useEffect, useId, useRef, useState } from "react";` ve `import { CLIENT_SEARCH_ID } from "./ClientList";`. `const [pending, …]` altına `const headingRef = useRef<HTMLHeadingElement>(null);`. Başlık:

```tsx
      <h2 id={`${id}-name`} ref={headingRef} tabIndex={-1} className="sidebar-heading">
```

`deleteChart`'ta `if (chart.id !== opened.chart_record.id) return session.touch();` satırını değiştir:

```tsx
    if (chart.id !== opened.chart_record.id) {
      session.touch();
      // Silinen satırın düğmesi kaybolur: odak danışan başlığına geçer.
      headingRef.current?.focus();
      return;
    }
```

Aynı işlevin sonunu değiştir:

```tsx
    // Diğer harita açılamazsa silinen harita ekranda bırakılmaz: pano boşalır.
    if (!next || e) {
      session.close();
      document.getElementById(CLIENT_SEARCH_ID)?.focus();
    } else headingRef.current?.focus();
    if (e) push({ id: "client", type: "error", messageKey: "notice.openFailed", error: e });
```

`deleteClient`'ın sonu:

```tsx
    session.close();
    document.getElementById(CLIENT_SEARCH_ID)?.focus();
```

`app/ui/notes/NotesPanel.tsx`: `ClientNotes` içinde `const addRef = useRef<HTMLButtonElement>(null);`; "Yeni not" düğmesine `ref={addRef}`; `remove`'da `setNotes(…filter…)` satırının altına `addRef.current?.focus();`.

`app/ui/clients/DataActions.tsx`: `import { useId, useState, useSyncExternalStore } from "react";`. Bileşenin üstüne:

```tsx
// Meşgul durumu modül düzeyindedir: yerel dosya penceresi açıkken kenar çubuğu kapanıp açılsa
// da sıfırlanmaz, ikinci pencere açılmaz.
let busyNow = false;
const busyListeners = new Set<() => void>();
function setExchangeBusy(value: boolean) {
  busyNow = value;
  for (const l of busyListeners) l();
}
const subscribeBusy = (l: () => void) => {
  busyListeners.add(l);
  return () => void busyListeners.delete(l);
};

/** Yalnızca testler: bir önceki testten taşınan meşgul durumunu sıfırlar. */
export function resetExchangeBusy() {
  setExchangeBusy(false);
}
```

Bileşende `const [busy, setBusy] = useState(false);` satırını `const busy = useSyncExternalStore(subscribeBusy, () => busyNow);` yap. `run`:

```tsx
  const run = async (work: () => Promise<void>) => {
    if (busyNow) return;
    setExchangeBusy(true);
    dismiss("exchange");
    try {
      await work();
    } finally {
      setExchangeBusy(false);
    }
  };

  const openExport = () => {
    if (busyNow) return;
    setIncludeNotes(false);
    setAsking(true);
  };
```

Üç düğme:

```tsx
        <button type="button" aria-disabled={busy || undefined} onClick={openExport}>
          {t("data.exportJson")}
        </button>
        <button type="button" aria-disabled={busy || undefined} onClick={() => void exportCsv()}>
          {t("data.exportCsv")}
        </button>
        <button type="button" aria-disabled={busy || undefined} onClick={() => void importJson()}>
          {t("data.importJson")}
        </button>
      </div>
      <p className="sidebar-hint" role="status">
        {busy ? t("data.working") : ""}
      </p>
```

(`exportJson` `run` içinden çağrılır; JSON penceresinin düğmesi `setAsking(false)` ile pencereyi kapatır, odak meşgulken de odaklanabilen "JSON olarak dışa aktar…" düğmesine döner.) `clients.css`'e:

```css
.sidebar-actions [aria-disabled="true"] {
  opacity: 0.6;
  cursor: default;
}
```

i18n — `tr.ts`: `layout` içine `builtinLocked: "Hazır düzenler yeniden adlandırılamaz ve silinemez.",`; `data` içine `working: "Çalışıyor…",`.
`en.ts`: `builtinLocked: "Built-in layouts cannot be renamed or deleted.",`; `working: "Working…",`.
`fr.ts`: `builtinLocked: "Les dispositions prédéfinies ne peuvent être ni renommées ni supprimées.",`; `working: "En cours…",`.

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/shell ui/theme ui/panels ui/clients ui/notes ui/palette
pnpm --dir app test && pnpm --dir app build
pnpm --dir app e2e
pnpm --dir app e2e:visual
rm -rf app/test-results
```
Expected: yeni dosyalar `notices.test.tsx` 1, `focus.test.ts` 2; tokens +1, Sidebar +2, Notes +1, DataActions +2, Palette +2 geçer; değişen LayoutPicker ve Graha testleri yeşil; toplam `Test Files  54 passed (54)`, `Tests  438 passed (438)`; build `✓ built`; e2e `6 passed`; görsel `6 passed` (`.visually-hidden` ve `.button-danger` yer değiştirdi, kuralları değişmedi; görüntü farkı çıkarsa kural metnini karşılaştır, temeli yenileme).

- [ ] **Step 5: Commit**

```bash
git add app/ui
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "fix(a11y): announced start notices, roving client list, focus after deletes, aria-disabled busy and menu items (M6-7)

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 13: Notlar ve danışanlar: sınır ipuçları, boş taze notlar, etiket süzgeci, yeniden okuma, yarışlar, kayıt pompası

Plan 10 madde 20, 23 (arayüz parçaları) ve 26'nın açık parçaları. Değişiklikler:

1. Not metni `MAX_NOTE_UNITS`'e ulaşınca kutunun altında "Not en çok 16 384 karakter olabilir; fazlası eklenmedi." görünür ve kutuya `aria-describedby` ile bağlanır. Danışanın `NOTES_LIMIT` (100) notu listelenince "En yeni 100 not gösteriliyor." görünür.
2. **D17:** "Yeni not" ile eklenen ve boş bırakılan not, odak nottan çıkınca ya da panel kapanınca silinir (onay sormaz; kullanıcı bir şey yazmadı). StrictMode'un kur-temizle-kur döngüsünde silinmez (temizlik bir mikro görev sonra hâlâ bağlı değilse siler).
3. Etiket süzgecinde seçili etiket artık yoksa (son haritadan kaldırıldı) süzgeç "Tüm etiketler"e döner. `list_tags` hatası süzgecin altında gösterilir; eski yanıt yok sayılır.
4. Notlar `Session.revision` değişince (içe aktarma, silme) bekleyen kayıtlar bittikten sonra yeniden okunur.
5. `OpenClient.deleteChart`: silme sürerken başka danışan ya da harita açıldıysa silme bitince ilk danışan yeniden açılmaz; yalnızca listeler yenilenir.
6. Etiket kutusunun Enter'ı 229 kodunu da denetler (WebKit IME onayı). `NoteEditor` silme sürerken ikinci silmeyi yok sayar.
7. `storePersistence` pompası: `send` hata fırlatırsa `task_failed` ile `onError`'a gider, kuyruk durmaz; pompa her durumda `running`'i sıfırlar (`try/finally`).

**Files:**
- Modify: `app/ui/notes/NoteEditor.tsx`, `app/ui/notes/NotesPanel.tsx`, `app/ui/notes/NotesPanel.test.tsx`
- Modify: `app/ui/clients/ClientList.tsx`, `app/ui/clients/OpenClient.tsx`, `app/ui/clients/TagEditor.tsx`, `app/ui/clients/Sidebar.test.tsx`
- Modify: `app/ui/view/persistence.ts`, `app/ui/view/persistence.test.ts`
- Modify: `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`

**Interfaces:**
- Consumes: `NOTES_LIMIT`, `MAX_NOTE_UNITS`, `enqueueNoteSave`, `notesIdle` (mevcut); `useSession().revision`, `session.touch()`; `CLIENT_SEARCH_ID` (Task 12).
- Produces: `NoteEditorProps.discardWhenEmpty: boolean`; i18n `notes.atLimit`, `notes.listLimit`, `clients.tagsFailed`.

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/notes/NotesPanel.test.tsx`: `import { useSession } from "../session/store";` ekle; `beforeEach` sonuna `vi.mocked(api.deleteNote).mockResolvedValue(ok(null));` (boş taze not artık bağlantı kesilince silinir; silme yanıtı her testte tanımlı olmalı). Sonuna:

```tsx
test("a note at the length limit and a client at the list limit say so", async () => {
  const many = Array.from({ length: NOTES_LIMIT }, (_, i) => note(i + 10, "x", `2026-01-${String((i % 28) + 1).padStart(2, "0")}T10:00:00.000Z`));
  vi.mocked(api.listNotes).mockResolvedValue(ok(many));
  renderPanel();
  expect(await screen.findByText(`En yeni ${NOTES_LIMIT} not gösteriliyor.`)).toBeInTheDocument();
  const box = within(list()).getAllByRole("textbox")[0];
  fireEvent.change(box, { target: { value: "a".repeat(MAX_NOTE_UNITS) } });
  expect(box).toHaveAccessibleDescription("Not en çok 16 384 karakter olabilir; fazlası eklenmedi.");
});

test("an empty new note is removed when focus leaves it, without asking", async () => {
  vi.mocked(api.addNote).mockResolvedValue(ok(note(3, "", "2026-03-01T09:00:00.000Z", 1)));
  renderPanel();
  await screen.findByRole("list", { name: "Örnek Danışan notları" });
  fireEvent.click(screen.getByRole("button", { name: "Yeni not" }));
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(3));
  fireEvent.blur(within(list()).getAllByRole("textbox")[0]);
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(2));
  expect(api.deleteNote).toHaveBeenCalledWith(3);
  expect(screen.queryByRole("dialog")).toBeNull();
});

test("an empty new note is removed when the panel closes; a written one and an old empty one stay", async () => {
  vi.mocked(api.listNotes).mockResolvedValue(ok([note(9, "", "2026-01-01T10:00:00.000Z"), older]));
  vi.mocked(api.addNote).mockResolvedValueOnce(ok(note(3, "", "2026-03-01T09:00:00.000Z", 1)));
  renderPanel();
  await screen.findByRole("list", { name: "Örnek Danışan notları" });
  fireEvent.click(screen.getByRole("button", { name: "Yeni not" }));
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(3));
  act(() => setShown(false));
  await waitFor(() => expect(api.deleteNote).toHaveBeenCalledOnce());
  expect(api.deleteNote).toHaveBeenCalledWith(3);
});

test("under StrictMode a new note is not removed by the mount, cleanup, mount cycle", async () => {
  vi.mocked(api.addNote).mockResolvedValue(ok(note(3, "", "2026-03-01T09:00:00.000Z", 1)));
  renderWithStores(
    <StrictMode>
      <NotesPanel panel={panel} />
    </StrictMode>,
  );
  await screen.findByRole("list", { name: "Örnek Danışan notları" });
  fireEvent.click(screen.getByRole("button", { name: "Yeni not" }));
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(3));
  await act(async () => {});
  expect(api.deleteNote).not.toHaveBeenCalled();
});

test("notes are read again after the session revision changes", async () => {
  let touch: () => void = () => {};
  function Toucher() {
    touch = useSession().touch;
    return null;
  }
  renderWithStores(
    <>
      <Toucher />
      <NotesPanel panel={panel} />
    </>,
  );
  await screen.findByRole("list", { name: "Örnek Danışan notları" });
  expect(api.listNotes).toHaveBeenCalledOnce();
  vi.mocked(api.listNotes).mockResolvedValue(ok([note(4, "içe aktarılan", "2026-04-01T10:00:00.000Z"), newer, older]));
  act(() => touch());
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(3));
  expect(api.listNotes).toHaveBeenCalledTimes(2);
});

test("a second confirmed delete while the first is running is ignored", async () => {
  let release!: () => void;
  vi.mocked(api.deleteNote).mockReturnValue(new Promise((r) => (release = () => r(ok(null)))));
  renderPanel();
  const items = await within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).findAllByRole("listitem");
  const del = within(items[1]).getByRole("button", { name: /tarihli notu sil/ });
  fireEvent.click(del);
  fireEvent.click(within(screen.getByRole("dialog", { name: "Not silinsin mi?" })).getByRole("button", { name: "Notu sil" }));
  fireEvent.click(del);
  fireEvent.click(within(screen.getByRole("dialog", { name: "Not silinsin mi?" })).getByRole("button", { name: "Notu sil" }));
  release();
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(1));
  expect(api.deleteNote).toHaveBeenCalledOnce();
});
```

`app/ui/clients/Sidebar.test.tsx` sonuna:

```tsx
test("a tag filter whose tag disappeared falls back to all tags; a tag list error is shown", async () => {
  vi.mocked(api.listTags).mockResolvedValue(ok([{ name: "Kariyer", charts: 1 }]));
  vi.mocked(api.listClientsByTag).mockResolvedValue(ok([ayse]));
  let touch: () => void = () => {};
  function Toucher() {
    touch = useSession().touch;
    return null;
  }
  renderWithStores(
    <>
      <Toucher />
      <Sidebar onAddChart={onAddChart} />
    </>,
    { opened: opened() },
  );
  const filter = await within(sidebar()).findByRole("combobox", { name: "Etiket" });
  await within(filter).findByRole("option", { name: "Kariyer (1)" });
  fireEvent.change(filter, { target: { value: "Kariyer" } });
  await waitFor(() => expect(api.listClientsByTag).toHaveBeenCalledWith("Kariyer", 200));
  vi.mocked(api.listTags).mockResolvedValue(ok([]));
  act(() => touch());
  await waitFor(() => expect(filter).toHaveValue(""));
  expect(within(sidebar()).getByRole("heading", { name: "Son açılanlar" })).toBeInTheDocument();
  vi.mocked(api.listTags).mockResolvedValue({ status: "error", error: { code: "store_sqlite", params: { detail: "disk" } } });
  act(() => touch());
  expect(await within(sidebar()).findByText(/Etiketler okunamadı: .*disk/)).toBeInTheDocument();
});

test("switching to another client while the open chart is being deleted does not reopen the first client", async () => {
  const charts = [chart(1, ""), chart(5, "Düzeltilmiş", "2000-01-02")];
  vi.mocked(api.listCharts).mockResolvedValue(ok(charts));
  let release!: () => void;
  vi.mocked(api.deleteChart).mockReturnValue(new Promise((r) => (release = () => r(ok(null)))));
  const other = opened();
  other.client = ayse;
  other.chart_record = { ...other.chart_record, id: 20, client_id: 2 };
  vi.mocked(api.openClient).mockResolvedValue(ok(other));
  renderSidebar();
  fireEvent.click(await within(sidebar()).findByRole("button", { name: "Haritayı sil: “Doğum haritası”" }));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Harita silinsin mi?" })).getByRole("button", { name: "Haritayı sil" }));
  fireEvent.click(await within(list()).findByRole("button", { name: /Ayşe Yılmaz/ }));
  await waitFor(() => expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Ayşe Yılmaz#20"));
  release();
  await act(async () => {});
  expect(api.openClient).toHaveBeenCalledOnce();
  expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Ayşe Yılmaz#20");
});

test("Enter that commits IME text in the tag box does not save", async () => {
  vi.mocked(api.setChartTags).mockResolvedValue(ok(["a"]));
  renderSidebar();
  const box = await within(sidebar()).findByRole("textbox", { name: "Etiketler" });
  fireEvent.change(box, { target: { value: "a" } });
  fireEvent.keyDown(box, { key: "Enter", keyCode: 229 });
  expect(api.setChartTags).not.toHaveBeenCalled();
  fireEvent.keyDown(box, { key: "Enter" });
  await waitFor(() => expect(api.setChartTags).toHaveBeenCalledOnce());
});
```

`app/ui/view/persistence.test.ts` sonuna:

```ts
test("a save that throws reaches onError as task_failed and the queue keeps going", async () => {
  vi.mocked(api.saveViewState).mockRejectedValueOnce(new Error("ipc")).mockResolvedValue(ok(null));
  const onError = vi.fn();
  const onSaved = vi.fn();
  const p = storePersistence({ onError, onSaved });
  p.save({ chartId: 4, viewState: vs });
  p.save({ chartId: 5, viewState: vs });
  await p.idle();
  expect(onError).toHaveBeenCalledWith({ code: "task_failed", params: {} });
  expect(onSaved).toHaveBeenCalledOnce();
  p.save({ chartId: 4, viewState: vs });
  await p.idle();
  expect(api.saveViewState).toHaveBeenCalledTimes(3);
});
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/notes ui/clients ui/view/persistence.test.ts
```
Expected: sınır metinleri bulunamaz; boş taze not silinmez (`deleteNote` çağrılmaz); `listNotes` bir kez çağrılır; ikinci silme çağrılır; süzgeç `"Kariyer"`de kalır ve etiket hatası görünmez; ilk danışan yeniden açılır (`openClient` iki kez); 229'lu Enter kaydeder; pompa: `idle()` reddedilir (`Error: ipc`).

- [ ] **Step 3: Uygula**

`app/ui/view/persistence.ts` — `pump`'ı değiştir:

```ts
  const pump = async () => {
    try {
      for (let next = queue.shift(); next; next = queue.shift()) {
        let r: Awaited<ReturnType<typeof send>>;
        try {
          r = await send(next);
        } catch {
          // `wrap` reddetmez; yine de fırlatan bir çağrı kuyruğu durdurmaz. Hata metni ve kayıt
          // içeriği bildirime konmaz.
          r = { status: "error", error: { code: "task_failed", params: {} } };
        }
        if (r.status === "error") events.onError?.(r.error);
        else events.onSaved?.();
      }
    } finally {
      running = null;
    }
  };
```

`app/ui/notes/NoteEditor.tsx`:

`NoteEditorProps`'a:

```tsx
  /** "Yeni not" ile eklendi ve henüz bu oturumda yazılmadı: boş bırakılırsa silinir (D17). */
  discardWhenEmpty: boolean;
```

`onDelete`'in tipini değiştir:

```tsx
  /** Onaylı silme; `false`: silinemedi (bildirim panelin işidir), not yerinde kalır. `quiet`: boş taze notun onaysız silinmesi, odak taşınmaz. */
  onDelete(note: Note, quiet?: boolean): Promise<boolean>;
```

Bileşen imzasına `discardWhenEmpty` ekle. `flush`'ın altına:

```tsx
  // Boş taze not: odak çıkınca ya da panel kapanınca onaysız silinir. Silme sürüyorsa tekrar edilmez.
  const discardable = () => discardWhenEmpty && !latest.current.deleted && latest.current.body.trim() === "";
  const discard = () => {
    const now = latest.current;
    now.deleted = true;
    if (now.timer) clearTimeout(now.timer);
    now.timer = null;
    // Sessiz silme: odak kullanıcının gittiği yerde durur (onaylı silmeden farklı olarak).
    // Silinemezse not listede durur; `deleted` geri alınır ki sonraki yazım kaydedilsin.
    void onDelete(note, true).then((ok) => {
      if (!ok) now.deleted = false;
    });
  };
  const discardRef = useRef({ discardable, discard });
  discardRef.current = { discardable, discard };
```

Bağlanma etkisinin temizliğini değiştir:

```tsx
    return () => {
      window.removeEventListener("pagehide", flush);
      document.removeEventListener("visibilitychange", onVisibility);
      mounted.current = false;
      flush();
      // StrictMode kur-temizle-kur yapar: bir mikro görev sonra hâlâ bağlı değilse boş taze not silinir.
      queueMicrotask(() => {
        if (!mounted.current && discardRef.current.discardable()) discardRef.current.discard();
      });
    };
```

`remove`'u değiştir:

```tsx
  const remove = async () => {
    setConfirm(false);
    // Silme sürerken ikinci onay yok sayılır.
    if (latest.current.deleted) return;
    latest.current.deleted = true;
    if (latest.current.timer) clearTimeout(latest.current.timer);
    if (await onDelete(note)) return;
    // Silinemedi: not yerinde kalır; silme başlarken durdurulan kayıt beklemeden gönderilir.
    latest.current.deleted = false;
    flush();
  };
```

`textarea`'nın `onBlur`'unu ve `aria-describedby`'ını değiştir, sınır metnini ekle:

```tsx
        aria-describedby={[error ? `${id}-error` : null, body.length >= MAX_NOTE_UNITS ? `${id}-limit` : null].filter(Boolean).join(" ") || undefined}
        onChange={(e) => change(e.target.value)}
        onBlur={(e) => {
          // Odak notun kendi düğmesine (Sil) geçiyorsa boş taze not silinmez; kullanıcı orada.
          if (discardable() && !e.currentTarget.closest("li")?.contains(e.relatedTarget as Node | null)) discard();
          else flush();
        }}
      />
      {body.length >= MAX_NOTE_UNITS && (
        <p id={`${id}-limit`} className="field-hint">
          {t("notes.atLimit")}
        </p>
      )}
```

(Sayı metne gömülüdür: Türkçe ve Fransızca binlik ayırıcısı boşluk, İngilizce virgüldür; `toLocaleString` Türkçede nokta yazardı.)

`app/ui/notes/NotesPanel.tsx`:

`const { push } = useNotices();` satırının altına `const { revision } = useSession();`. Okuma etkisinin bağımlılıkları `[clientId, retry, revision]` olsun. `NoteEditor`'a `discardWhenEmpty={n.id === fresh}` ver. Listenin altına (`</ul>` ile `)}` arasından sonra):

```tsx
      {notes && notes.length >= NOTES_LIMIT && <p className="field-hint">{t("notes.listLimit", { max: NOTES_LIMIT })}</p>}
```

`remove`'u değiştir (Task 12'nin odak satırı yalnızca onaylı silmede çalışır):

```tsx
  const remove = async (n: Note, quiet = false) => {
    // Silme, kayıtlarla aynı sırada gider: notun bekleyen kaydı silinmiş nota yazmaya çalışmaz.
    const r = await enqueueNoteSave(() => api.deleteNote(n.id));
    if (r.status === "error") {
      push({ id: "delete", type: "error", messageKey: "notice.deleteFailed", error: r.error });
      return false;
    }
    setNotes((list) => list && list.filter((x) => x.id !== n.id));
    setFresh((f) => (f === n.id ? null : f));
    if (!quiet) addRef.current?.focus();
    return true;
  };
```

Dosya başındaki yorumun sonuna: `// "Yeni not" ile eklenip boş bırakılan not, odak çıkınca ya da panel kapanınca silinir (D17).`

`app/ui/clients/ClientList.tsx`:

`const [tags, setTags] = useState<TagCount[]>([]);` altına `const [tagsError, setTagsError] = useState<CommandError | null>(null);`. Etiket okuma etkisini değiştir:

```tsx
  useEffect(() => {
    let live = true;
    void api.listTags().then((r) => {
      if (!live) return;
      if (r.status === "error") return setTagsError(r.error);
      setTagsError(null);
      setTags(r.data);
      // Seçili etiket artık hiçbir haritada yoksa süzgeç "Tüm etiketler"e döner.
      setTag((current) => (current !== "" && !r.data.some((tc) => tc.name === current) ? "" : current));
    });
    return () => {
      live = false;
    };
  }, [session.revision]);
```

Etiket `<select>`'inin kapanışından sonra (aynı `sidebar-field` içinde):

```tsx
        {tagsError && (
          <p className="sidebar-error" role="alert">
            {t("clients.tagsFailed", { message: errorMessage(t, i18n, tagsError) })}
          </p>
        )}
```

`app/ui/clients/OpenClient.tsx`:

`headingRef`'in altına:

```tsx
  // Silme sürerken başka harita ya da danışan açılırsa silme bitince geri dönülmez.
  const current = useRef(opened);
  current.current = opened;
  const live = useRef(true);
  useEffect(() => {
    live.current = true;
    return () => {
      live.current = false;
    };
  }, []);
```

`deleteChart`'ı değiştir (Task 12'nin odak satırları korunur):

```tsx
  const deleteChart = async (chart: ChartRecord) => {
    setPending(null);
    // Bekleyen görünüm kaydı silmeden önce yazılır; silinen haritaya sonradan kayıt gitmez (C5).
    flush();
    const r = await api.deleteChart(chart.id);
    if (r.status === "error") return failed(r.error);
    // Silme sürerken başka danışana (bileşen ayrıldı) ya da başka haritaya geçildiyse yalnızca listeler yenilenir.
    if (!live.current || chart.id !== current.current.chart_record.id) {
      session.touch();
      if (live.current) headingRef.current?.focus();
      return;
    }
    // Açık harita silindi: diğer haritalardan en son güncelleneni açılır (uyarısız).
    const rest = await api.listCharts(clientId);
    if (!live.current || current.current.chart_record.id !== chart.id) return session.touch();
    const next = rest.status === "ok" ? rest.data[0] : undefined;
    const e = next ? await session.openClient(clientId, next.id) : rest.status === "error" ? rest.error : null;
    // Diğer harita açılamazsa silinen harita ekranda bırakılmaz: pano boşalır.
    if (!next || e) {
      session.close();
      document.getElementById(CLIENT_SEARCH_ID)?.focus();
    } else headingRef.current?.focus();
    if (e) push({ id: "client", type: "error", messageKey: "notice.openFailed", error: e });
  };
```

`app/ui/clients/TagEditor.tsx` — Enter koşulu:

```tsx
          // IME bileşimini onaylayan Enter kaydetmez (WebKit bunu 229 koduyla bildirir).
          if (e.key === "Enter" && !e.nativeEvent.isComposing && e.keyCode !== 229) {
```

i18n — `tr.ts`: `notes` içine

```ts
    atLimit: "Not en çok 16 384 karakter olabilir; fazlası eklenmedi.",
    listLimit: "En yeni {{max}} not gösteriliyor.",
```

`clients` içine `tagsFailed: "Etiketler okunamadı: {{message}}",`.

`en.ts`: `atLimit: "A note can be at most 16,384 characters; the rest was not added.",`, `listLimit: "Showing the newest {{max}} notes.",`, `tagsFailed: "Tags could not be read: {{message}}",`.

`fr.ts`: `atLimit: "Une note peut contenir au plus 16 384 caractères ; le surplus n’a pas été ajouté.",`, `listLimit: "Les {{max}} notes les plus récentes sont affichées.",`, `tagsFailed: "Les étiquettes n’ont pas pu être lues : {{message}}",`.

(Sayı metindedir: `MAX_NOTE_UNITS` değişirse üç metin aynı commit'te değişir. `NoteEditor.tsx`'te `MAX_NOTE_UNITS` tanımının yorumuna bunu ekle: `// Değişirse notes.atLimit metinleri de değişir.`)

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/notes ui/clients ui/view
pnpm --dir app test && pnpm --dir app build
pnpm --dir app e2e
rm -rf app/test-results
```
Expected: Notes +6, Sidebar +3, persistence +1 geçer; toplam `Test Files  54 passed (54)`, `Tests  448 passed (448)`; build `✓ built`; e2e `6 passed` (e2e not akışı notu yazdığı için boş taze not silinmez).

- [ ] **Step 5: Commit**

```bash
git add app/ui
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "fix(ui): note limits shown, empty new notes removed, stale tag filter reset, delete races and a sturdier save pump

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---
### Task 14: İçe/dışa aktarma: UTF-8 ve BOM, kararlı `file_io.detail`, uyarı sınırı, klasör `fsync`'i, pencere sahibi, dışa aktarma boyutu, virgüllü etiket, biçim belgesi ve spec düzeltmeleri

Plan 10 madde 13 (Plan 11 parçaları), 17, 22, 23 (Rust parçaları), 24 ve m4, m6, m8, m10, m11 kayıt maddeleri. Değişiklikler:

1. `read_bounded` dosyayı bayt olarak okur: UTF-8 değilse `import_format` (`detail: "utf8"`), başındaki UTF-8 BOM atılır.
2. `file_io.detail` `ErrorKind` metni değil kararlı koddur (D16); arayüz `errors.fileIoDetail.*` ile çevirir, bilinmeyen kod olduğu gibi görünür.
3. `ImportReport.warnings` en çok `MAX_IMPORT_WARNINGS` (100) öğe taşır; `warnings_total` gerçek sayıdır. Arayüz sayıyı `warnings_total`'dan yazar.
4. Unix'te `rename`'den sonra hedef klasör `fsync` edilir (yalnızca dayanıklılık; hata yok sayılır, dosya zaten yerindedir).
5. Kaydetme ve açma pencereleri `set_parent` ile uygulama penceresine bağlanır (komutlar `tauri::WebviewWindow` alır; bağlamalar değişmez, Tauri bu bağımsız değişkeni kendisi verir).
6. Dışa aktarma komutları `ExportResult { file, bytes, over_import_limit }` döner (D15); yazılan dosya `MAX_IMPORT_BYTES`'ı aşarsa arayüz `warning` bildirimi gösterir.
7. `normalize_tags` virgülde böler (D14): içe aktarmadan gelen virgüllü etiket ikiye ayrılır, etiket kutusunda bölünüp yeniden kaydedilince değişmez.
8. `api.test.ts`: bağlamalardaki her `string`/`string[]` bağımsız değişkeni adından bağımsız bulunur ve izin listesiyle karşılaştırılır (yol taşıyan yeni bir dize bağımsız değişkeni testi düşürür); iletim tablosu `api`'nin her anahtarını kapsar (Plan 09 Task 3 bağımsız değişken fikstürleri).
9. `docs/export-format.md` yazılır; v1 spec §4 (sekme ifadesi, Plan 10 D4) ve §5 (64 KiB not, 32 MiB içe aktarma sınırı) düzeltilir.
10. Arama ölçümü (Plan 08 Task 7 "etiket sorgusunun ham biçimi ve `load_client` N+1"): 200 danışanlı aramanın süresi `#[ignore]`'lu bir testle ölçülür ve commit mesajına yazılır. Bütçe (50 ms, spec §4) aşılırsa görev durur ve bulgu bildirilir; optimizasyon bu planda yapılmaz.

**Files:**
- Modify: `app/src-tauri/src/store_commands.rs`
- Modify: `crates/store/src/exchange.rs`, `crates/store/src/jhd.rs`, `crates/store/src/notes.rs`, `crates/store/src/search.rs`, `crates/store/src/lib.rs`
- Modify: `app/ui/bindings.ts` (üretilir: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings`)
- Modify: `app/ui/clients/DataActions.tsx`, `app/ui/clients/DataActions.test.tsx`
- Modify: `app/ui/i18n/errors.ts`, `app/ui/i18n/i18n.test.ts`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Modify: `app/ui/api.test.ts`
- Create: `docs/export-format.md`
- Modify: `docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md`

**Interfaces:**
- Consumes: `MAX_IMPORT_BYTES` (32 MiB), `base_name`, `write_atomic_with`, `save_dialog`, `chosen` (mevcut); `StoreError::ImportFormat` → `import_format` (mevcut eşleme).
- Produces:
  - `pub struct ExportResult { pub file: String, pub bytes: u32, pub over_import_limit: bool }` (Rust ve bağlama); `export_clients_json`/`export_clients_csv` → `Result<Option<ExportResult>, CommandError>`
  - `fn io_detail(kind: ErrorKind) -> &'static str` (modül içi)
  - `pub const MAX_IMPORT_WARNINGS: usize = 100` (`store`); `ImportReport.warnings_total: u32` (`serde(default)` yok, D21; `ImportReport` `Default` türettiği için `..Default::default()` kuran yerler değişmez); `ImportReport::warn(&mut self, w: ImportWarning)`
  - `export_json_file_blocking(...) -> Result<usize, CommandError>`, `export_csv_file_blocking(...) -> Result<usize, CommandError>` (yazılan bayt)
  - i18n `errors.fileIoDetail.{not_found,permission_denied,already_exists,invalid_input,invalid_data,storage_full,other}`, `data.exportedOverLimit`

- [ ] **Step 1: Başarısız testleri yaz**

`crates/store/src/notes.rs` `mod tests` sonuna:

```rust
    #[test]
    fn a_comma_inside_a_tag_splits_it() {
        let tags = normalize_tags(&["Kariyer, aile".into(), "kariyer".into(), " , ".into()]).unwrap();
        assert_eq!(tags, ["Kariyer", "aile"]);
    }
```

`crates/store/src/exchange.rs` `mod tests` sonuna:

```rust
    #[test]
    fn import_warnings_are_capped_but_counted() {
        let mut report = ImportReport::default();
        for i in 0..(MAX_IMPORT_WARNINGS + 50) {
            report.warn(ImportWarning {
                item: format!("u{i}"),
                code: ImportWarningCode::CalcSettingsReset,
            });
        }
        assert_eq!(report.warnings.len(), MAX_IMPORT_WARNINGS);
        assert_eq!(report.warnings_total as usize, MAX_IMPORT_WARNINGS + 50);
        assert_eq!(report.warnings[0].item, "u0");
    }
```

Aynı dosyadaki `ImportReport { clients_added: 1, …, warnings: vec![] }` karşılaştırmasına `warnings_total: 0,` ekle.

`crates/store/src/search.rs` `mod tests` sonuna (ölçüm; CI'da koşmaz):

```rust
    /// Plan 08 Task 7 ölçümü: 200 danışan, her biri iki etiketli bir haritayla; etiketle arama.
    /// `cargo test -p store --release -- --ignored search_of_two_hundred_clients --nocapture`
    #[test]
    #[ignore]
    fn search_of_two_hundred_clients_stays_within_budget() {
        let mut s = crate::db::testutil::store();
        for i in 0..200 {
            let (_, ch) = s.create_client(&crate::charts::testutil::new_client(&format!("Danışan {i}"))).unwrap();
            s.set_chart_tags(ch.id, &["kariyer".into(), format!("grup {}", i % 7)]).unwrap();
        }
        let start = std::time::Instant::now();
        let hits = s.search_clients("kariyer", 200).unwrap();
        let took = start.elapsed();
        println!("search_clients(200 hits): {took:?}");
        assert_eq!(hits.len(), 200);
        assert!(took < std::time::Duration::from_millis(50), "{took:?}");
    }
```

(`search.rs`'in test modülü yoksa `#[cfg(test)] mod tests { use super::*; … }` ile kur. `set_chart_tags`'in imzası `(&mut self, chart_id: u32, tags: &[String])` değilse çağrıyı dosyadaki imzaya uydur.)

`app/src-tauri/src/store_commands.rs` `mod tests` sonuna:

```rust
    #[test]
    fn an_import_that_is_not_utf8_is_a_format_error_and_a_bom_is_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let latin = dir.path().join("eski.json");
        std::fs::write(&latin, b"{\"name\": \"Ay\xfee\"}").unwrap();
        let e = read_bounded(&latin).unwrap_err();
        assert_eq!((e.code.as_str(), e.params["detail"].as_str()), ("import_format", "utf8"));
        let bom = dir.path().join("bom.json");
        std::fs::write(&bom, "\u{feff}{\"a\":1}").unwrap();
        assert_eq!(read_bounded(&bom).unwrap(), "{\"a\":1}");
    }

    #[test]
    fn an_exported_backup_with_a_bom_imports() {
        let src = populated();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("yedek.json");
        let text = lock(&src).unwrap().export_json(&ExportOptions::default()).unwrap();
        std::fs::write(&path, format!("\u{feff}{text}")).unwrap();
        let dst = Mutex::new(Store::open_in_memory().unwrap());
        let report = import_json_file_blocking(&dst, &path).unwrap();
        assert_eq!(report.clients_added, 1);
    }

    #[test]
    fn file_errors_carry_a_stable_detail_code() {
        let e = io_error(Path::new("/x/a.json"), ErrorKind::PermissionDenied.into());
        assert_eq!(e.params["detail"], "permission_denied");
        let codes: Vec<_> = [
            ErrorKind::NotFound,
            ErrorKind::AlreadyExists,
            ErrorKind::InvalidInput,
            ErrorKind::InvalidData,
            ErrorKind::StorageFull,
            ErrorKind::Interrupted,
        ]
        .into_iter()
        .map(io_detail)
        .collect();
        assert_eq!(codes, ["not_found", "already_exists", "invalid_input", "invalid_data", "storage_full", "other"]);
    }

    #[test]
    fn an_export_reports_its_size_and_whether_it_can_be_imported_again() {
        let r = export_result(Path::new("/x/yedek.json"), 1_000);
        assert_eq!((r.file.as_str(), r.bytes, r.over_import_limit), ("yedek.json", 1_000, false));
        let big = export_result(Path::new("/x/büyük.json"), MAX_IMPORT_BYTES as usize + 1);
        assert!(big.over_import_limit);
        assert_eq!(big.bytes as u64, MAX_IMPORT_BYTES + 1);
        let store = populated();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.json");
        let n = export_json_file_blocking(&store, &path, &ExportOptions::default()).unwrap();
        assert_eq!(n as u64, std::fs::metadata(&path).unwrap().len());
    }
```

`app/ui/i18n/i18n.test.ts` başına `import { errorMessage } from "./errors";` ekle (`i18n`, `tr`, `en`, `fr` zaten içe aktarılır); sonuna:

```ts
test("file error details are localized in every language", () => {
  const codes = ["already_exists", "invalid_data", "invalid_input", "not_found", "other", "permission_denied", "storage_full"];
  for (const lang of [tr, en, fr]) expect(Object.keys(lang.errors.fileIoDetail).sort()).toEqual(codes);
});

test("a file error shows its detail in the UI language and an unknown detail as it came", () => {
  const t = i18n.getFixedT("tr");
  expect(errorMessage(t, i18n, { code: "file_io", params: { file: "a.csv", detail: "permission_denied" } })).toBe("Dosya okunamadı ya da yazılamadı (a.csv: izin yok).");
  expect(errorMessage(t, i18n, { code: "file_io", params: { file: "a.csv", detail: "weird" } })).toBe("Dosya okunamadı ya da yazılamadı (a.csv: weird).");
});
```

`app/ui/clients/DataActions.test.tsx`:

`report` sabitine `warnings_total: 0` ekle; "an import with reset settings…" testindeki `{ ...report, warnings: [{ item: "u", code: "CalcSettingsReset" as const }] }` nesnesine `warnings_total: 1` ekle. Dışa aktarma taklitleri artık `ExportResult` döner: dosyadaki `ok("danisanlar.json")` gibi her dışa aktarma yanıtını `ok({ file: "danisanlar.json", bytes: 1_000, over_import_limit: false })` yap (`ok(null)` vazgeçmedir, değişmez). "CSV export errors are shown with the file name only" testindeki taklit hatanın `detail`'ini `"permission_denied"` ve beklenen metni `"Dışa aktarılamadı: Dosya okunamadı ya da yazılamadı (haritalar.csv: izin yok)."` yap. Sonuna:

```tsx
test("an export bigger than the import limit is saved and says it cannot be imported again", async () => {
  vi.mocked(api.exportClientsCsv).mockResolvedValue(ok({ file: "büyük.csv", bytes: 40 * 1024 * 1024, over_import_limit: true }));
  renderData();
  fireEvent.click(within(section()).getByRole("button", { name: "CSV olarak dışa aktar…" }));
  const notice = await screen.findByText(/büyük\.csv/);
  expect(notice).toHaveTextContent("“büyük.csv” kaydedildi (40 MB). Dosya içe aktarma sınırını (32 MB) aşıyor; bu uygulamaya olduğu gibi geri yüklenemez.");
  expect(notice.closest('[role="status"]')).toHaveClass("notice--warning");
});
```

`app/ui/api.test.ts`:

"every wrapper forwards its arguments…" testindeki `calls` dizisine Task 4'ün ve dosyanın başındaki testlerin sardığı ama tabloda olmayan her anahtarı ekle; dizinin sonuna:

```ts
    ["engineStatus", []],
    ["printWindow", []],
    ["keepStoredMoment", [3]],
    ["updateClient", [2, "Ayşe", "Female"]],
    ["saveCalcSettings", [3, { schema_version: 1 }]],
    ["getAppPrefs", []],
    ["setAppPrefs", [{ schema_version: 1 }]],
    ["transitsAt", ["Now", { schema_version: 1 }, { cusps: [] }, { birth_jd_ut: 1 }]],
```

(Task 4 bu satırlardan bazılarını eklediyse ikinci kez ekleme.) `vi.mock("./bindings")` listesinde eksik olan komutu (`engineStatus` ve `printWindow` zaten var) ekle. Döngüden sonra:

```ts
  // Tablo her sarmalayıcıyı kapsar: yeni bir sarmalayıcı burada da yazılmadan geçmez.
  expect(calls.map(([name]) => name).sort()).toEqual(Object.keys(api).sort());
```

Dosya sonuna:

```ts
test("every string argument of a command is a known, non-path one (found by type, not by name)", () => {
  const block = bindingsSource.slice(bindingsSource.indexOf("export const commands = {"));
  const found: string[] = [];
  for (const m of block.matchAll(/^\s+(\w+): \(([^)]*)\) =>/gm)) {
    for (const p of m[2].split(",")) {
      const arg = /^\s*(\w+)\??: (string(?:\[\])?)(?: \| null)?\s*$/.exec(p);
      if (arg) found.push(`${m[1]}.${arg[1]}`);
    }
  }
  expect(found.sort()).toEqual(
    ["addNote.body", "listClientsByTag.tag", "searchClients.query", "searchPlaces.query", "setChartTags.tags", "updateClient.name", "updateNote.body"].sort(),
  );
});
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
cargo test -p store --lib 2>&1 | tail -4
cargo test -p jyotish-desk --lib 2>&1 | tail -4
pnpm --dir app exec vitest run ui/i18n ui/clients/DataActions.test.tsx ui/api.test.ts
```
Expected: store `cannot find function warn`/`MAX_IMPORT_WARNINGS`, `no field warnings_total`; desk `cannot find function io_detail`/`export_result`, `read_bounded` `file_io` döner; vitest: `fileIoDetail` yok, `"permission_denied"` çevrilmez, dışa aktarma bildirimi `[object Object]` ya da dosya adsız, `Object.keys(api)` tablodan uzun ise iletim testi düşer, dize testi `found` listesini yazdırır (izin listesiyle aynıysa geçer: bu test bir korumadır).

- [ ] **Step 3: Uygula**

`crates/store/src/notes.rs` — `normalize_tags`:

```rust
/// Kırpma, NFC, büyük/küçük harf ve tekrar normalleştirmesi. Etiket virgül içeremez: arayüz
/// etiketleri virgülle ayırır, içe aktarmadan gelen virgüllü etiket burada bölünür (D14).
pub fn normalize_tags(tags: &[String]) -> Result<Vec<String>, StoreError> {
    let mut out: Vec<String> = Vec::new();
    let mut keys: Vec<String> = Vec::new();
    for t in tags.iter().flat_map(|t| t.split(',')) {
        let t = nfc(t.trim());
        let key = tag_key(&t);
        if key.is_empty() {
            continue;
        }
        if t.chars().count() > MAX_TAG_CHARS {
            return Err(StoreError::InvalidInput("tags"));
        }
        if !keys.contains(&key) {
            keys.push(key);
            out.push(t);
        }
    }
    Ok(out)
}
```

(Var olan belge yorumu farklıysa ilk satırı koru, D14 cümlesini ekle.)

`crates/store/src/exchange.rs`:

```rust
/// `ImportReport.warnings`'te tutulan en çok öğe; `warnings_total` gerçek sayıdır.
pub const MAX_IMPORT_WARNINGS: usize = 100;
```

`ImportReport`'a alan ve yöntem:

```rust
    pub warnings: Vec<ImportWarning>,
    /// Bütün uyarıların sayısı; `warnings` ilk `MAX_IMPORT_WARNINGS` tanesidir. `serde(default)`
    /// yoktur: IPC tiplerinde faz farkı yaratan öznitelik kullanılmaz (`specta_builder`, D21).
    pub warnings_total: u32,
}

impl ImportReport {
    /// Uyarıyı sayar; liste `MAX_IMPORT_WARNINGS`'e ulaştıysa öğeyi eklemez.
    pub fn warn(&mut self, w: ImportWarning) {
        self.warnings_total = self.warnings_total.saturating_add(1);
        if self.warnings.len() < MAX_IMPORT_WARNINGS {
            self.warnings.push(w);
        }
    }
}
```

`report.warnings.push(ImportWarning { … })` çağrısını `report.warn(ImportWarning { … })` yap. `crates/store/src/jhd.rs`'teki `out.report.warnings.extend(parsed.warnings.iter().map(…))` çağrısını şununla değiştir:

```rust
                    for &code in &parsed.warnings {
                        out.report.warn(ImportWarning {
                            item: file_name.clone(),
                            code,
                        });
                    }
```

`crates/store/src/lib.rs` `exchange` dışa aktarmalarına `MAX_IMPORT_WARNINGS` ekle (`pub use exchange::{…}` satırı).

`app/src-tauri/src/store_commands.rs`:

`io_error`'ı değiştir ve altına `io_detail`:

```rust
fn io_error(path: &Path, e: std::io::Error) -> CommandError {
    CommandError::new(
        "file_io",
        &[("file", base_name(path)), ("detail", io_detail(e.kind()).to_owned())],
    )
}

/// `ErrorKind` → kararlı kod (D16). Metin İngilizcedir ve sürümler arasında değişebilir; arayüz
/// kodu kendi dilinde gösterir (`errors.fileIoDetail`).
fn io_detail(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NotFound => "not_found",
        ErrorKind::PermissionDenied => "permission_denied",
        ErrorKind::AlreadyExists => "already_exists",
        ErrorKind::InvalidInput => "invalid_input",
        ErrorKind::InvalidData => "invalid_data",
        ErrorKind::StorageFull => "storage_full",
        _ => "other",
    }
}
```

`write_atomic_with`'te `std::fs::rename(&tmp, path)` satırını şununla değiştir:

```rust
        std::fs::rename(&tmp, path)?;
        sync_parent(path);
        Ok(())
```

ve işlevin altına:

```rust
/// Taşımanın kendisi de diske yazılsın (Unix: klasör girdisi). Yalnızca dayanıklılıktır: dosya
/// zaten yerindedir, hata yok sayılır.
fn sync_parent(path: &Path) {
    // Öznitelik `if` ifadesine doğrudan yazılamaz (derlenmez); blok sarılır.
    #[cfg(unix)]
    {
        let dir = path.parent().filter(|d| !d.as_os_str().is_empty());
        if let Some(Ok(d)) = dir.map(File::open) {
            let _ = d.sync_all();
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}
```

`read_bounded`'ın okuma bölümünü değiştir (`text` yerine bayt):

```rust
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|f| f.take(MAX_IMPORT_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|e| io_error(path, e))?;
    if bytes.len() as u64 > MAX_IMPORT_BYTES {
        return Err(too_large());
    }
    // Başındaki UTF-8 BOM (bazı düzenleyiciler ekler) atılır; UTF-8 olmayan dosya biçim hatasıdır.
    let body = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    String::from_utf8(body.to_vec()).map_err(|_| CommandError::new("import_format", &[("detail", "utf8".to_owned())]))
```

(Belge yorumuna ekle: `UTF-8 olmayan dosya import_format (detail utf8); baştaki BOM atılır.`)

Dışa aktarma gövdeleri yazılan bayt sayısını döner:

```rust
pub fn export_json_file_blocking(
    store: &Mutex<Store>,
    path: &Path,
    options: &ExportOptions,
) -> Result<usize, CommandError> {
    let text = lock(store)?.export_json(options)?;
    write_atomic(path, &text)?;
    Ok(text.len())
}

pub fn export_csv_file_blocking(store: &Mutex<Store>, path: &Path) -> Result<usize, CommandError> {
    let text = lock(store)?.export_csv()?;
    write_atomic(path, &text)?;
    Ok(text.len())
}

/// Dışa aktarılan dosyanın adı ve boyutu; içe aktarma sınırını aşan dosya bu uygulamaya geri
/// yüklenemez (D15: dışa aktarma engellenmez, arayüz uyarır).
#[derive(Debug, Clone, Serialize, Type)]
pub struct ExportResult {
    pub file: String,
    /// `u32::MAX`'ta doyar (4 GiB).
    pub bytes: u32,
    pub over_import_limit: bool,
}

fn export_result(path: &Path, bytes: usize) -> ExportResult {
    ExportResult {
        file: base_name(path),
        bytes: u32::try_from(bytes).unwrap_or(u32::MAX),
        over_import_limit: bytes as u64 > MAX_IMPORT_BYTES,
    }
}
```

`save_dialog`'a pencere:

```rust
/// Kaydetme penceresi; uygulama penceresine bağlıdır (`set_parent`). `None`: kullanıcı vazgeçti.
fn save_dialog(
    app: &AppHandle,
    window: &WebviewWindow,
    filter: &str,
    ext: &str,
    name: &str,
) -> Result<Option<PathBuf>, CommandError> {
    chosen(
        app.dialog()
            .file()
            .set_parent(window)
            .add_filter(filter, &[ext])
            .set_file_name(name)
            .blocking_save_file(),
    )
}
```

`use tauri::{AppHandle, State};` → `use tauri::{AppHandle, State, WebviewWindow};`. Üç komut:

```rust
pub async fn export_clients_json(
    app: AppHandle,
    window: WebviewWindow,
    store: State<'_, StoreState>,
    include_notes: bool,
) -> Result<Option<ExportResult>, CommandError> {
    let store = Arc::clone(&store);
    blocking(move || {
        let Some(path) = save_dialog(&app, &window, "JSON", "json", EXPORT_JSON_NAME)? else {
            return Ok(None);
        };
        let options = ExportOptions {
            include_notes,
            client_ids: None,
        };
        let bytes = export_json_file_blocking(&store, &path, &options)?;
        Ok(Some(export_result(&path, bytes)))
    })
    .await
}
```

```rust
pub async fn export_clients_csv(
    app: AppHandle,
    window: WebviewWindow,
    store: State<'_, StoreState>,
) -> Result<Option<ExportResult>, CommandError> {
    let store = Arc::clone(&store);
    blocking(move || {
        let Some(path) = save_dialog(&app, &window, "CSV", "csv", EXPORT_CSV_NAME)? else {
            return Ok(None);
        };
        let bytes = export_csv_file_blocking(&store, &path)?;
        Ok(Some(export_result(&path, bytes)))
    })
    .await
}
```

`import_clients_json`'a `window: WebviewWindow,` bağımsız değişkeni (`app`'ten sonra) ve pencere zincirine `.set_parent(&window)` (`.file()`'dan sonra). Komutların belge yorumlarında "dosyanın adı" → "dosyanın adı ve boyutu (`ExportResult`)". `blocking` kapanışı `'static` ister: `WebviewWindow` `Clone + Send`'dir, `move` ile taşınır.

(`set_parent` `tauri-plugin-dialog` `=2.8.4`'te `#[cfg(desktop)]`'tur; uygulama yalnızca masaüstüdür. `WebviewWindow` `HasWindowHandle + HasDisplayHandle` sağlar. `tauri-specta` `WebviewWindow` bağımsız değişkenini bağlamaya yazmaz; `bindings.ts`'te yalnızca dönüş tipi ve `ImportReport.warnings_total` değişir.)

Bağlamaları üret:

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
```

`app/ui/clients/DataActions.tsx` — `exported`'ı değiştir (`ExportResult`'ı `../bindings`'ten içe aktar):

```tsx
  /** İçe aktarma sınırı (Rust `MAX_IMPORT_BYTES`); uyarı metni bu sayıyı taşır. */
  const exported = (r: ExportResult | null) => {
    if (r === null) return;
    if (!r.over_import_limit) return push({ id: "exchange", type: "info", messageKey: "data.exported", params: { file: r.file } });
    push({ id: "exchange", type: "warning", messageKey: "data.exportedOverLimit", params: { file: r.file, mb: String(Math.ceil(r.bytes / (1024 * 1024))) } });
  };
```

İçe aktarma bildiriminde `report.warnings.length` geçen üç yeri `report.warnings_total` yap (`type`, `messageKey` ve `warnings: String(report.warnings_total)`).

`app/ui/i18n/errors.ts` — `atlas_unavailable` dalının altına:

```ts
  if (error.code === "file_io") {
    const detail = String(error.params.detail ?? "other");
    params.detail = i18n.exists(`errors.fileIoDetail.${detail}`) ? t(`errors.fileIoDetail.${detail}`) : detail;
  }
```

i18n — `tr.ts` `errors` içine:

```ts
    fileIoDetail: {
      not_found: "dosya ya da klasör yok",
      permission_denied: "izin yok",
      already_exists: "dosya zaten var",
      invalid_input: "sıradan bir dosya değil",
      invalid_data: "dosya içeriği okunamadı",
      storage_full: "disk dolu",
      other: "beklenmeyen bir dosya hatası",
    },
```

`data` içine:

```ts
    exportedOverLimit: "“{{file}}” kaydedildi ({{mb}} MB). Dosya içe aktarma sınırını (32 MB) aşıyor; bu uygulamaya olduğu gibi geri yüklenemez.",
```

`en.ts`: `fileIoDetail: { not_found: "no such file or folder", permission_denied: "permission denied", already_exists: "the file already exists", invalid_input: "not a regular file", invalid_data: "the file content could not be read", storage_full: "the disk is full", other: "an unexpected file error" }`, `exportedOverLimit: "“{{file}}” saved ({{mb}} MB). The file is over the import limit (32 MB); it cannot be restored into this app as it is."`.

`fr.ts`: `fileIoDetail: { not_found: "fichier ou dossier introuvable", permission_denied: "permission refusée", already_exists: "le fichier existe déjà", invalid_input: "pas un fichier ordinaire", invalid_data: "contenu du fichier illisible", storage_full: "disque plein", other: "erreur de fichier inattendue" }`, `exportedOverLimit: "« {{file}} » enregistré ({{mb}} Mo). Le fichier dépasse la limite d’importation (32 Mo) ; il ne peut pas être restauré tel quel dans cette application."`.

(32 sayısı metindedir: `MAX_IMPORT_BYTES` değişirse üç metin aynı commit'te değişir; `store_commands.rs`'teki sabitin yorumuna `// Değişirse data.exportedOverLimit metinleri de değişir.` ekle.)

`docs/export-format.md`:

````markdown
# Jyotish Desk dışa aktarma biçimleri

Bu belge `export_clients_json` ve `export_clients_csv` komutlarının yazdığı dosyaları ve
`import_clients_json`'ın neyi kabul ettiğini anlatır. Kod: `crates/store/src/exchange.rs`,
`crates/store/src/csv.rs`, `app/src-tauri/src/store_commands.rs`.

## JSON (kayıpsız yedek, içe aktarılabilir)

```json
{
  "format": "jyotish-desk",
  "schema_version": 1,
  "exported_at": "2026-11-25T10:00:00.000Z",
  "clients": [
    {
      "uuid": "…", "name": "…", "gender": "Female" | null,
      "created_at": "…", "updated_at": "…",
      "charts": [
        {
          "uuid": "…", "label": "", "kind": "Natal" | "Event",
          "birth": { "date": "2000-01-01", "time": "17:30:00" | null, "time_certainty": "Exact",
                     "place": { "name": "…", "country": "…", "admin1": "…", "lat": 28.6139, "lon": 77.209, "geoname_id": 1261481 | null },
                     "tz": { "Iana": "Asia/Kolkata" } | { "Fixed": 19800 } | "Lmt",
                     "fold": "Earlier" | "Later" | null, "notes": "" },
          "resolved": { "utc": "2000-01-01T12:00:00Z", "jd_ut": 2451545.0, "offset_secs": 19800, "abbrev": "IST", "warnings": [], "tzdb_version": "2026f" },
          "calc_settings": { … } | null,
          "view_state": { "dasha_expanded_path": [] } | null,
          "tags": ["…"],
          "created_at": "…", "updated_at": "…"
        }
      ],
      "notes": [ { "uuid": "…", "chart_uuid": "…" | null, "body": "…", "created_at": "…", "updated_at": "…" } ] | null
    }
  ]
}
```

- **Notlar** yalnızca "Notları dahil et" işaretliyse yazılır; değilse `notes` `null`'dır (boş dizi değil).
- **`calc_settings` ve `view_state`** saklanan JSON'un aynısıdır: daha yeni bir sürümün alanları korunur.
  Saklanan JSON okunamıyorsa alan `null` yazılır. İçe aktarmada `null` ya da okunamayan
  `calc_settings` o anki genel varsayılanla değiştirilir ve bir `CalcSettingsReset` uyarısı sayılır;
  `null` `view_state` boş dasha yoluyla açılır. Düzenler (`panel_layout`) kullanıcınındır, aktarılmaz.
- **Bilinmeyen alanlar** içe aktarmada yok sayılır (reddedilmez); `format` `"jyotish-desk"` değilse ya da
  `schema_version` bu sürümden büyükse dosya reddedilir (`import_format`, `import_version`).
- **Birleştirme** UUID iledir: var olan danışan, harita ya da not atlanır ve `skipped` sayılır; üzerine yazılmaz.
  İçe aktarma tek işlemdir: bir kayıt hatalıysa hiçbir şey eklenmez (`import_record`).
- **Etiketler** virgül içeremez; virgüllü etiket içe aktarmada bölünür.
- **Kodlama** UTF-8'dir; baştaki BOM kabul edilir. UTF-8 olmayan dosya `import_format` (`utf8`) ile reddedilir.

## CSV (tek yönlü, tablo programları için)

Harita başına bir satır, RFC 4180: virgül ayırıcı, CRLF satır sonu, gerekirse çift tırnak, UTF-8 BOM ile
başlar. Notlar hiçbir zaman CSV'ye yazılmaz. Sütunlar (sırasıyla):

`client_uuid, client_name, gender, chart_uuid, label, kind, date, time, time_certainty, place_name,
country, admin1, lat, lon, tz_kind, tz_id, offset_override, fold, resolved_utc, resolved_offset_secs,
tzdb_version, tags, birth_notes, calc_settings_json`

- `tz_kind` `Iana`, `Fixed` ya da `Lmt`; `tz_id` yalnızca `Iana`'da, `offset_override` (saniye) yalnızca `Fixed`'de doludur.
- `tags` etiketleri `"; "` ile birleştirir (virgül CSV ayırıcısıdır; etiket virgül içeremez).
- `=`, `+`, `-`, `@`, sekme ya da CR ile başlayan metin alanlarının başına `'` eklenir (formül enjeksiyonu).
- CSV içe aktarılamaz; yedek için JSON kullanılır.

## Sınırlar

| Sınır | Değer | Nerede |
|---|---|---|
| İçe aktarılan JSON | 32 MiB (`MAX_IMPORT_BYTES`); dosya okunmadan önce boyutuyla denetlenir | `import_too_large` |
| Not gövdesi | 65 536 bayt UTF-8 (`MAX_NOTE_BYTES`); arayüz 16 384 karakter | `invalid_input` (`body`) |
| Etiket | `MAX_TAG_CHARS` karakter | `invalid_input` (`tags`) |
| İçe aktarma uyarıları | ilk 100 öğe listelenir, `warnings_total` hepsini sayar | `ImportReport` |
| Dışa aktarma | sınır yok; dosya 32 MiB'ı aşarsa uyarı gösterilir (bu uygulamaya olduğu gibi geri yüklenemez) | `ExportResult.over_import_limit` |

Hata bildirimlerinde yalnızca dosya adı bulunur; tam yol ve dosya içeriği gösterilmez ve loglanmaz.
````

`docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md`:

§4 "Danışan açma" listesinde iki satırı değiştir:

```markdown
- `Cmd+K` danışan arama paleti: isim, etiket ve not araması. `Enter` haritayı açar, `Cmd+Enter` danışanı açar ve kenar çubuğunu gösterir (sekme yoktur; UI spec §4).
- Tek harita açıktır (UI spec §4 "Sekme yoktur"); her harita kendi `ViewState`'ini saklar. Uygulama kapanıp açılınca son harita ve düzen geri yüklenir.
```

§5 "İçe ve dışa aktarma" listesinin sonuna:

```markdown
- Sınırlar: içe aktarılan JSON en çok 32 MiB (okunmadan önce denetlenir); not gövdesi en çok 64 KiB (65 536 bayt UTF-8, arayüzde 16 384 karakter). Dışa aktarma sınırlanmaz; dosya içe aktarma sınırını aşarsa kullanıcı uyarılır. Biçimin ayrıntısı: `docs/export-format.md`.
```

- [ ] **Step 4: Testlerin geçtiğini gör ve ölçümü al**

```bash
cargo test -p store --lib 2>&1 | grep -E "test result"
cargo test -p jyotish-desk --lib 2>&1 | grep -E "test result"
cargo test -p jyotish-desk --test bindings --test ui_fixture --test open_client
cargo test -p store --release --lib -- --ignored search_of_two_hundred_clients --nocapture 2>&1 | grep -E "search_clients|test result"
cargo clippy --workspace --all-targets -- -D warnings
pnpm --dir app test && pnpm --dir app build
```
Expected: store `79 passed; 0 failed; 1 ignored`; desk lib `33 passed`; bağlama, fikstür ve gidiş-dönüş testleri geçer (`ui_fixture` değişmez: fikstürde dışa aktarma yoktur); ölçüm satırı `search_clients(200 hits): …ms` ve `1 passed` (50 ms'yi aşarsa test düşer: görev burada durur, süre ve `EXPLAIN QUERY PLAN` çıktısı bildirilir); clippy çıktısız; vitest `Test Files  54 passed (54)`, `Tests  452 passed (452)` (i18n +2, DataActions +1, api +1); build `✓ built`.

- [ ] **Step 5: Commit**

Ölçülen süreyi commit gövdesine yaz (`<süre>` yerine Step 4'ün çıktısı):

```bash
git add app/src-tauri/src/store_commands.rs crates/store/src app/ui docs/export-format.md docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "fix(exchange): UTF-8 and BOM on import, stable file error codes, capped warnings, parented dialogs, export size warning, format document

Search of 200 tagged clients (release): <süre>.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 15: Uçtan uca akışlar ve test borcu (M6-9)

Plan 10 madde 11, 21 ve 25. Değişiklikler:

1. e2e IPC taklidi gerçekçi olur: `list_clients` ve `search_clients` `limit`'e uyar; etiketler bellek içidir (`set_chart_tags` yazar, `list_tags` sayar, `list_clients_by_tag` süzer).
2. Yeni e2e akışları: `⌘,` ile ayar değişikliği ve fark rozeti; transit düzeninde tarih seçimi ve "Şimdi"; etiket yazıp etiket süzgeciyle danışanı bulma. e2e 6 → 9.
3. Dasha e2e testinde `dasha_children` sayısı beklenerek (`expect.poll`) okunur.
4. Plan 09 Task 1: arama sorgusunun kesilmesi saf bir işlevle sınanır (10 000 karakterlik sorgu kesilmeden de boş dönerdi). Task 7: lagna satırının boylam hücresi ve boş boylamda `—`. Task 12: eski önizleme yanıtı, çift kayıt, saniyeli öneri.
5. Plan 08 Task 12: DST boşluğuna düşen `update_chart` saklanan haritayı değiştirmez. (Doğum verisinin fikstürle karşılaştırılması ve `save_view_state` gövdesinin testi Task 4'tedir.)

**Files:**
- Modify: `app/e2e/ipc.ts`, `app/e2e/desk.spec.ts`
- Modify: `app/src-tauri/src/atlas_commands.rs`
- Modify: `crates/store/src/charts.rs`
- Modify: `app/ui/panels/GrahaTablePanel.test.tsx`
- Modify: `app/ui/birth/BirthForm.test.tsx`

**Interfaces:**
- Consumes: `fixture.prefs`, `fixture.transit` (Task 3, 4); e2e taklitleri `get_app_prefs`, `set_app_prefs`, `save_calc_settings` (Task 5), `transits_at` (Task 9); `MAX_QUERY_CHARS` (100).
- Produces: `fn clean_query(query: &str) -> String` (`atlas_commands.rs`, modül içi); e2e taklidinde `keep_stored_moment: () => fixture.opened`, `update_client`.

- [ ] **Step 1: Testleri yaz**

`app/src-tauri/src/atlas_commands.rs` `mod tests` içinde `empty_query_and_zero_limit_return_nothing_and_long_queries_are_cut` testinin son iki satırını şunlarla değiştir:

```rust
        // Kesme saf işlevde ölçülür: 10 000 karakterlik sorgu kesilmeden de boş dönerdi.
        assert_eq!(clean_query(&"a".repeat(10_000)).chars().count(), MAX_QUERY_CHARS);
        assert_eq!(clean_query(&"ş".repeat(10_000)).chars().count(), MAX_QUERY_CHARS);
        assert_eq!(clean_query("  new delhi  "), "new delhi");
```

ve yeni test:

```rust
    #[test]
    fn a_query_cut_at_the_limit_still_finds_its_prefix() {
        let atlas = shipped_atlas();
        let padded = format!("new delhi{}", " ".repeat(MAX_QUERY_CHARS));
        let hits = search_places_blocking(&atlas, &padded, 5).unwrap();
        assert_eq!(hits.first().map(|h| h.name.as_str()), Some("New Delhi"));
    }
```

`crates/store/src/charts.rs` `mod tests` sonuna:

```rust
    #[test]
    fn an_update_into_a_dst_gap_changes_nothing() {
        let mut s = store();
        let (_, ch) = s.create_client(&new_client("Ayşe")).unwrap();
        let before = s.chart(ch.id).unwrap();
        // İstanbul 2014-03-31 03:00–04:00 yaşanmadı (yaz saati geçişi).
        let e = s
            .update_chart(
                ch.id,
                &NewChart {
                    label: "x".into(),
                    kind: ChartKind::Event,
                    birth: istanbul("2014-03-31", "03:30"),
                },
            )
            .unwrap_err();
        assert!(matches!(e, StoreError::Moment(_)), "{e:?}");
        assert_eq!(s.chart(ch.id).unwrap(), before);
    }
```

(`ChartRecord` `PartialEq` türetmiyorsa son satırı `assert_eq!(serde_json::to_value(s.chart(ch.id).unwrap()).unwrap(), serde_json::to_value(before).unwrap());` yap. `NewChart`, `ChartKind`, `StoreError` test modülünde `super::*` ile gelmiyorsa içe aktar.)

`app/ui/panels/GrahaTablePanel.test.tsx` sonuna:

```tsx
test("the lagna row shows its longitude, leaves latitude and speed empty, and a missing longitude is a dash", () => {
  const lagna = () => screen.getByRole("rowheader", { name: "Lagna" }).closest("tr")!;
  const first = renderTable({ columns: ["lon", "lat", "speed"] });
  expect([...lagna().querySelectorAll("td")].map((c) => c.textContent)).toEqual(["76°20′", "", ""]);
  first.unmount();
  const o = fixture().opened;
  o.chart.lagna.lon = null;
  renderTable({ columns: ["lon"] }, o);
  expect([...lagna().querySelectorAll("td")].map((c) => c.textContent)).toEqual(["—"]);
});
```

`app/ui/birth/BirthForm.test.tsx` sonuna:

```tsx
test("a preview reply for an older draft is ignored", async () => {
  let first!: (v: Awaited<ReturnType<typeof api.previewMoment>>) => void;
  vi.mocked(api.previewMoment).mockReturnValueOnce(new Promise((r) => (first = r)));
  renderForm("new");
  await fillDelhi();
  fireEvent.change(input("Saat"), { target: { value: "17:31" } });
  await wait(PREVIEW_DEBOUNCE_MS);
  await act(async () => first({ status: "error", error: fixture().gap_error }));
  expect(input("Saat")).not.toHaveAttribute("aria-invalid");
  expect(within(dialog()).getByRole("region", { name: "Önizleme" })).toHaveTextContent("UTC 2000-01-01");
});

test("a second save click while saving sends nothing", async () => {
  let release!: () => void;
  vi.mocked(api.createClient).mockReturnValueOnce(new Promise((r) => (release = () => r({ status: "ok", data: fixture().opened }))));
  renderForm("new");
  await fillDelhi();
  const save = within(dialog()).getByRole("button", { name: "Haritayı kaydet" });
  fireEvent.click(save);
  fireEvent.click(save);
  expect(save).toBeDisabled();
  await act(async () => release());
  expect(api.createClient).toHaveBeenCalledOnce();
  expect(onClose).toHaveBeenCalledOnce();
});

test("a DST suggestion with seconds keeps them in the button and the field", async () => {
  vi.mocked(api.previewMoment).mockResolvedValueOnce({
    status: "error",
    error: { code: "nonexistent_local_time", params: { local: "2014-03-31T03:30:15", suggestion: "2014-03-31T04:30:15" } },
  });
  renderForm("new");
  await fillDelhi();
  fireEvent.click(within(dialog()).getByRole("button", { name: "04:30:15 kullan" }));
  expect(input("Saat")).toHaveValue("04:30:15");
});
```

`app/e2e/ipc.ts` — `mockIpc` içinde `notes` tanımının altına:

```ts
      // Etiketler bellek içi: harita kimliği → etiketler (Rust normalleştirmesi taklit edilmez).
      const tags: Record<number, string[]> = {};
      const chartOf = (clientId: number) => opened[clientId].chart_record.id;
```

`respond`'daki dört satırı değiştir:

```ts
        list_clients: (args) => clients().slice(0, args.limit as number),
        list_clients_by_tag: (args) => clients().filter((c) => (tags[chartOf(c.id)] ?? []).includes(String(args.tag))).slice(0, args.limit as number),
        list_tags: () => {
          const count = new Map<string, number>();
          for (const list of Object.values(tags)) for (const t of list) count.set(t, (count.get(t) ?? 0) + 1);
          return [...count].map(([name, charts]) => ({ name, charts }));
        },
        set_chart_tags: (args) => {
          tags[args.chartId as number] = (args.tags as string[]).filter((t) => t.trim() !== "");
          return tags[args.chartId as number];
        },
```

`search_clients`'in sonuna `.slice(0, args.limit as number)` ekle. `respond`'a:

```ts
        keep_stored_moment: () => fixture.opened,
        update_client: (args) => ({ ...opened[args.clientId as number].client, name: args.name, gender: args.gender }),
```

(Task 5'in `get_app_prefs`/`set_app_prefs`/`save_calc_settings` ve Task 9'un `transits_at` satırları yerindedir.)

`app/e2e/desk.spec.ts`:

"dasha tree by keyboard loads level 4 on demand" testindeki `expect(await ipcCalls(page, "dasha_children")).toHaveLength(1);` satırını değiştir:

```ts
  await expect.poll(async () => (await ipcCalls(page, "dasha_children")).length).toBe(1);
```

Dosya sonuna:

```ts
test("Cmd+, edits the open chart's settings; the change is saved for that chart and the difference badge appears", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Comma");
  const dialog = page.getByRole("dialog", { name: "Hesap ayarları" });
  await expect(dialog).toBeVisible();
  await dialog.getByRole("combobox", { name: "Ayanamsa" }).selectOption({ label: "Raman" });
  await expect.poll(async () => (await ipcCalls(page, "save_calc_settings")).length).toBe(1);
  const [call] = (await ipcCalls(page, "save_calc_settings")) as { chartId: number; settings: { ayanamsa: unknown } }[];
  expect(call.chartId).toBe(1);
  expect(call.settings.ayanamsa).toBe("Raman");
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole("button", { name: /Ayar farkı/ })).toBeVisible();
});

test("the transit layout shows Now; a chosen moment is applied and Now returns to the present", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit3");
  const table = page.getByRole("region", { name: "Transit tablosu" });
  await expect(table).toBeVisible();
  const now = table.getByRole("button", { name: "Şimdi" });
  await expect(now).toHaveAttribute("aria-pressed", "true");
  await table.getByLabel("Tarih").fill("2030-01-01");
  await table.getByLabel("Saat").fill("12:00");
  await table.getByRole("button", { name: "Uygula" }).click();
  await expect(now).toHaveAttribute("aria-pressed", "false");
  await expect.poll(async () => JSON.stringify((await ipcCalls(page, "transits_at")).at(-1))).toContain("2030-01-01");
  await now.click();
  await expect(now).toHaveAttribute("aria-pressed", "true");
  await expect.poll(async () => JSON.stringify((await ipcCalls(page, "transits_at")).at(-1))).toContain('"Now"');
});

test("a tag written on the open chart appears in the tag filter and finds that client", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const sidebar = page.getByRole("complementary", { name: "Danışanlar" });
  const box = sidebar.getByRole("textbox", { name: "Etiketler" });
  await box.fill("Kariyer");
  await box.press("Enter");
  const filter = sidebar.getByRole("combobox", { name: "Etiket" });
  await expect(filter.locator("option", { hasText: "Kariyer (1)" })).toHaveCount(1);
  await filter.selectOption({ label: "Kariyer (1)" });
  const list = sidebar.getByRole("region", { name: "“Kariyer” etiketli" });
  await expect(list.getByRole("button")).toHaveCount(1);
  await expect(list.getByRole("button")).toContainText("Örnek Danışan");
  expect((await ipcCalls(page, "list_clients_by_tag")).at(-1)).toEqual({ tag: "Kariyer", limit: 200 });
});
```

(Transit düzeni hazır düzenlerin üçüncüsüdür, `⌘3`; tarih ve saat alanlarının etiketleri `transit.dateLabel`/`transit.timeLabel`'dır. Ayanamsa seçeneklerinin görünen adı `settings.ayanamsa.Raman`'dır.)

- [ ] **Step 2: Testleri koş**

```bash
cargo test -p jyotish-desk --lib atlas 2>&1 | tail -4
cargo test -p store --lib an_update_into_a_dst_gap 2>&1 | tail -4
pnpm --dir app exec vitest run ui/panels/GrahaTablePanel.test.tsx ui/birth/BirthForm.test.tsx
pnpm --dir app e2e
```
Expected: `cannot find function clean_query`; DST testi geçer (davranış zaten doğrudur; bu bir borç kapatmadır, geçmezse `update_chart`'ın çözümden önce yazdığı bir alan vardır: görev durur ve bildirilir); vitest yeni testler geçer (borç kapatma; düşen bir test bulunan bir hatadır ve aynı görevde düzeltilir); e2e: yeni üç akıştan etiket akışı `Kariyer (1)` seçeneğini bulamaz (taklit `list_tags`'te boş döner), ayar ve transit akışları geçer.

- [ ] **Step 3: Uygula**

`app/src-tauri/src/atlas_commands.rs`:

```rust
/// Kırpılmış ve `MAX_QUERY_CHARS` karaktere (bayta değil) kesilmiş sorgu.
fn clean_query(query: &str) -> String {
    query.trim().chars().take(MAX_QUERY_CHARS).collect()
}
```

`search_places_blocking`'te `let query: String = query.trim().chars().take(MAX_QUERY_CHARS).collect();` satırını `let query = clean_query(query);` yap.

`ipc.ts` değişiklikleri Step 1'dedir (taklit değişikliği bu görevin uygulamasıdır; Step 2'deki düşüş taklit değişmeden önce koşulursa görülür).

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
cargo test -p jyotish-desk --lib 2>&1 | grep -E "test result"
cargo test -p store --lib 2>&1 | grep -E "test result"
pnpm --dir app test && pnpm --dir app build
pnpm --dir app e2e
pnpm --dir app e2e
pnpm --dir app e2e:visual
rm -rf app/test-results
```
Expected: desk lib `34 passed`; store `80 passed; 0 failed; 1 ignored`; vitest `Test Files  54 passed (54)`, `Tests  456 passed (456)`; build `✓ built`; e2e iki kez `9 passed`; görsel `6 passed` (temeller yenilenmez).

- [ ] **Step 5: Commit**

```bash
git add app/e2e app/src-tauri/src/atlas_commands.rs crates/store/src/charts.rs app/ui
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test: settings, transit and tag e2e flows with a realistic IPC mock; M6-9 test debt

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---
### Task 16: Son doğrulama ve elle denetim kapısı

Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle kapanır ve bu görev baştan koşulur.

**Files:**
- Yok (yalnızca doğrulama). İlerleme notları depo dışıdır; `git add` edilmez.

**Interfaces:**
- Consumes: Task 1–15'in tamamı.
- Produces: yeşil kapılar ve elle denetim sonucu (ilerleme notlarına yazılır).

- [ ] **Step 1: Rust kapıları**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p store --features specta --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store -p jyotish-desk --all-targets
cargo deny check
```
Expected: fmt ve clippy çıktısız; testler: `jyotish-core` lib `110 passed`, golden `8 passed`, `store` lib `80 passed; 0 failed; 1 ignored`, `jyotish-desk` lib `34 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`, hiçbir satırda `failed` sıfırdan büyük değil; MSRV `Finished`; deny `advisories ok, bans ok, licenses ok, sources ok`.

- [ ] **Step 2: Üretilen dosyalar güncel**

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture
git status --short
```
Expected: iki test geçer, `git status --short` boş (bağlamalar ve fikstür son commit'lerle aynı).

- [ ] **Step 3: Arayüz kapıları**

```bash
pnpm --dir app test
pnpm --dir app build
pnpm --dir app e2e
pnpm --dir app e2e:visual
rm -rf app/test-results
```
Expected: `Test Files  54 passed (54)`, `Tests  456 passed (456)`; build `✓ built`; e2e `9 passed`; görsel `6 passed`.

- [ ] **Step 4: Kural taramaları (`6d48791`'den bu yana)**

```bash
git diff 6d48791..HEAD -U0 -- . ':(exclude)docs/superpowers/plans' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log 6d48791..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff 6d48791..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff 6d48791..HEAD --name-only | grep -vE '^(app|crates|docs)/'
echo "trailers: $(git log 6d48791..HEAD --format=%B | grep -c '^Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>$') commits: $(git rev-list --count 6d48791..HEAD)"
git diff 6d48791..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
grep -nE '\bpath: string\b|\bpaths: string\[\]' app/ui/bindings.ts
grep -n '"dialog:\|"fs:' app/src-tauri/capabilities/*.json
grep -rn 'tauri_plugin_fs' app/src-tauri/src
git diff 6d48791..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\('
```
Expected: `trailers:` satırındaki iki sayı eşit; bunun dışındaki her komut çıktısızdır. (`grep` çıktısız olunca 1 ile çıkar; beklenen budur.) Kural metninin kendisi bu planın dosyasındadır ve dosya `6d48791`'den sonra commit edilmiştir; ilk tarama bu yüzden plan klasörünü dışarıda tutar (D23). `trailers`/`commits` sayımı plan commit'lerini de kapsar; onlar da aynı satırla biter.

- [ ] **Step 5: Elle denetim kapısı (`pnpm --dir app tauri dev`)**

Gerçek uygulamada, deneme danışanlarıyla (gerçek danışan verisi kullanılmaz). Her madde ilerleme notlarına "geçti" ya da bulgu olarak yazılır; bulgu varsa düzeltme commit'i ve bu görev baştan.

- [ ] `⌘,` ayarlar penceresini açar; macOS uygulama menüsündeki "Settings…" (varsa) aynı pencereyi açar ya da hiç tepki vermez, iki pencere açılmaz.
- [ ] Ayanamsa değiştirilince harita yeniden hesaplanır (Lagna derecesi değişir), 500 ms içinde kaydedilir; uygulama kapanıp açılınca değişiklik durur.
- [ ] Haritanın ayarı varsayılandan farklıyken üst çubukta "Ayar farkı" rozeti görünür; "Varsayılana döndür" haritayı geri hesaplar, "Bunu varsayılan yap" yeni haritaların ayarını değiştirir ve rozet kaybolur.
- [ ] tzdb sürümü farklı bir harita (eski bir yedekten içe aktarılmış) açılınca bant görünür; "Yeni zamanı kabul et" anı yeniler, "Saklanan zamanı koru" bandı kapatır ve bir dahaki açılışta bant yine gelmez.
- [ ] Transit düzeninde (`⌘3`) "Şimdi" saatin yerel zamanını gösterir; seçilen tarih uygulanınca tablo değişir; uygulama yeniden açılınca transit tarihi "Şimdi"ye döner (D8).
- [ ] Harita panelinde transit halkası açılıp kapanır, yalnızca D1'de görünür; açık durumu uygulama yeniden açılınca korunur.
- [ ] Dasa şeridinde transit anını içeren dönem "transit anında" metniyle işaretlidir; bir döneme tıklamak transit anını o dönemin başlangıcına taşır.
- [ ] `birth_info` paneli doğum anını, dilimi, ofseti ve tzdb sürümünü gösterir; uyarı rozetlerinin açıklaması okunur.
- [ ] Danışan düzenleme formunda cinsiyet değişikliği kaydedilir ve kenar çubuğunda görünür.
- [ ] VoiceOver: belirsiz yerel saatte iki seçenek "radio group" olarak ve hatası ile duyurulur; geçersiz kayıtta odak ilk hatalı alana gider ve hata özeti okunur.
- [ ] Dışa ve içe aktarma pencereleri uygulama penceresine bağlı sayfa (sheet) olarak açılır; ana pencere arkada tıklanamaz.
- [ ] 32 MB'ı aşan bir dışa aktarma (ya da Rust testindeki eşik geçici olarak düşürülerek değil, büyük deneme verisiyle) uyarı bildirimi verir; dosya yazılmıştır.
- [ ] Değişiklik yapılmış doğum formunda Esc "Değişiklikler atılsın mı?" onayını açar; "Vazgeç" düğmesi sormadan kapatır.
- [ ] Boş "Yeni not" alanından çıkılınca not listeden silinir; yazılı not durur.
- [ ] Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, not gövdesi, doğum verisi ya da dosya yolu görünmez.

Kapı geçtiyse ilerleme notlarına `Plan 11 Task 16 elle denetim: geçti (<tarih>)` yazılır. Bu adım commit üretmez.

---

## Başarı ölçütleri

- Spec M6 tamamdır: ayarlar penceresi ve haritaya özel ayarlar, fark rozeti, tzdb farkını kabul ya da saklanan anı koruma, oturum geri yükleme (gidiş-dönüş testiyle), transit tablosu, halkası ve tarih seçimi, `birth_info` ve `dasha_strip` panelleri, cinsiyet alanı.
- Plan 10'un kapsam dışı listesindeki 1, 6, 7, 9, 10 (18, 19 ile), 11, 12, 13 (Plan 11 bölümleri), 17, 20, 21, 22, 23, 24, 25, 26 maddeleri ve Plan 11'e işaretli kayıt maddeleri (palet test boşlukları, m4, m6, m8, m10, m11) kapanmıştır.
- Şema değişmemiştir (göç 1); yeni bağımlılık yoktur; C1–C17 taramaları temizdir.
- Kapılar: Vitest 54/456, `jyotish-desk` lib 34, `store` lib 80 (+1 ölçüm testi `#[ignore]`), `jyotish-core` lib 110, golden 8, `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2, e2e 9, görsel 6; elle denetim kapısı geçmiştir.

## Bu planın kapsamadıkları

- Plan 10 kapsam dışı 2: JHD içe aktarma arayüzü ve `import_jhd` komutunun kaydı (D1; C2′ altında ayrı plan).
- Plan 10 kapsam dışı 3 ve 4: M8 işleri (dağıtım, imzalama, güncelleme).
- Plan 10 kapsam dışı 5: bir sonraki göç (göç 2); bu plan şemayı değiştirmez (D2).
- Plan 10 kapsam dışı 8: M6-5 (D1).
- Plan 10 kapsam dışı 14: "istek olursa" maddesi; istek gelmedikçe açılmaz.
- Plan 10 kapsam dışı 15 ve 16: M7/M8'e bağlı maddeler.
- Dasa ağacından transit anına bağlantı (D10: yalnızca şerit tıklaması taşır).
- Özel ayanamsa düzenleyicisi (ayarlar penceresi yalnızca hazır ayanamsaları sunar).
- Transit halkasının dış halka geometrisi (D1 dışındaki varga çizimleri, halka yarıçapı ayarı).
- M7 yazdırma ve PDF çıktısı.
- Plan 08 Task 6'nın kırpma testi: ilgili dosyaya bir sonraki dokunuşta eklenir.
- 200 danışanlı arama bütçeyi aşarsa (Task 14 Step 4) gereken optimizasyon: ölçüm bu plandadır, iyileştirme sonraki plandadır.

---

## Self-Review

**1. Spec kapsamı.**

| Gereksinim | Görev |
|---|---|
| Spec §5 ayarlar ve haritaya özel `calc_settings`, `AppPrefs` sürüm koruması | Task 1, 4, 5 |
| Spec §5/§8 fark rozeti, tzdb farkı kabul ve sessiz yol | Task 2, 4, 6 |
| Spec §8 oturum geri yükleme | Task 4 (gidiş-dönüş testi), D19 |
| Spec §4 transit bindirme (tablo, halka, tarih, "şimdi") | Task 3, 7, 8, 15 (e2e) |
| `birth_info`, `dasha_strip`, cinsiyet | Task 9, 10, 11 |
| M6-1, M6-6, M6-7, M6-9 | Task 11, 9, 12, 4 ve 15 |
| İçe/dışa aktarma sınırları, biçim belgesi, spec düzeltmeleri | Task 14 |
| Kayıt maddeleri m4 (`fsync`), m6 (UTF-8, `file_io.detail`), m8 (yol testi), m10 (uyarı sınırı), m11 (`set_parent`) | Task 14 |
| Palet test boşlukları | Task 12 |
| Son doğrulama ve elle denetim | Task 16 |

Boşluk yok; kapsam dışı maddeler yukarıda listelidir.

**2. Yer tutucu taraması.** `TODO`, `TBD`, "daha sonra", "uygun hata işleme" yok. Task 14 Step 5'teki `<süre>` ve Task 16 Step 5'teki `<tarih>` ölçülen değerin yazılacağı yerdir, kod değildir. Koşullu cümleler ("imza farklıysa dosyadaki imzaya uydur") `6d48791` ağacında doğrulanamayan tek satırlık uyumlar içindir; davranış ve beklenen sonuç her adımda yazılıdır.

**3. Tip tutarlılığı.**
- `ExportResult { file, bytes: u32, over_import_limit }` Task 14'te Rust'ta tanımlanır, `bindings.ts`'e üretilir, `DataActions.tsx` ve testi aynı adları kullanır.
- `ImportReport.warnings_total: u32` Rust'ta düz bir alandır (`serde(default)` yok, D21) ve bağlamada `number`'dır; arayüz bildirimi ve `DataActions.test.tsx` `report` sabiti bu alanı taşır.
- `file_io.detail` kodları (`not_found`, `permission_denied`, `already_exists`, `invalid_input`, `invalid_data`, `storage_full`, `other`) D16, `io_detail`, `errors.fileIoDetail` (tr/en/fr) ve `i18n.test.ts`'te aynıdır.
- e2e taklitleri: `get_app_prefs`, `set_app_prefs`, `save_calc_settings` (Task 5, `Chart` döner), `transits_at` (Task 9), `keep_stored_moment`, `update_client` ve gerçekçi etiket taklidi (Task 15); komut bağımsız değişken adları (`chartId`, `clientId`, `tag`, `limit`, `tags`) `bindings.ts`'in camelCase biçimidir.
- i18n anahtarları: `settings.*` ("Hesap ayarları", "Ayanamsa", "Ayar farkı") Task 5–6, `transit.*` ("Tarih", "Saat", "Uygula", "Şimdi") Task 7, `birthInfo.*` ve `gender.*` Task 10, `data.working` Task 12, `notes.atLimit` Task 13, `data.exportedOverLimit` Task 14; e2e ve Vitest testleri aynı metinleri arar.

**4. Sayaç zinciri.**

| Görev sonrası | Vitest dosya/test | `jyotish-desk` lib | `store` lib | Diğer |
|---|---|---|---|---|
| Taban `6d48791` | 43/363 | 24 | 71 | core 108, e2e 6, görsel 6 |
| Task 1 | 43/363 | 24 | 75 | |
| Task 2 | 43/363 | 24 | 77 | |
| Task 3 | 43/363 | 27 | 77 | core 110 |
| Task 4 | 43/364 | 29 | 77 | `ui_fixture` 2, `open_client` 1 |
| Task 5 | 45/375 | 29 | 77 | e2e 6, görsel 6 |
| Task 5 inceleme düzeltmesi | 46/385 | 29 | 77 | `settings/saves.test.ts` 3, `SettingsDialog.test.tsx` +7 |
| Task 6 | 48/391 | 29 | 77 | |
| Task 7 | 50/399 | 29 | 77 | |
| Task 8 | 50/403 | 29 | 77 | görsel 6 (temeller değişmez) |
| Task 9 | 51/409 | 29 | 77 | |
| Task 10 | 52/412 | | | |
| Task 11 | 52/422 | | | |
| Task 12 | 54/438 | | | |
| Task 13 | 54/448 | | | |
| Task 14 | 54/452 | 33 | 79 (+1 ignored) | |
| Task 15 | 54/456 | 34 | 80 (+1 ignored) | e2e 9, görsel 6 |
| Task 16 | değişmez | | | |

**5. Yürütme notu.** Kod blokları `6d48791` ağacına karşı yazılmıştır ve plan yazılırken koşulmamıştır. Beklenen sayılar bu zincirden gelir: bir görevin sayısı tutmazsa önceki görevin adımı denetlenir; sayı elle düzeltilmez.
