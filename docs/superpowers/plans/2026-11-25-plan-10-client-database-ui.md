# Plan 10 — Danışan veritabanı arayüzü: kenar çubuğu, `Cmd+K` paleti, notlar, etiketler, çoklu harita ve güvenli içe/dışa aktarma (M6, birinci dilim)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** M6'nın danışan veritabanı dilimini tamamlamak: danışanlar kenar çubuğunda aranır, etiketle süzülür ve açılır; `Cmd+K` paleti danışan, harita, düzen, varga ve komutlara tek kutudan ulaşır ve danışanı formsuz açar (spec §8); notlar paneli tarih damgalı notları otomatik kaydeder; aynı danışana ikinci harita eklenir, harita ve danışan onayla silinir; bütün danışanlar JSON/CSV olarak dışa, JSON olarak içe aktarılır (dosya pencereleri Rust'ta, boyut ve alan sınırlarıyla). M6'nın öbür parçaları (fark rozeti, ayarlar, transit katmanı, doğum bilgisi paneli, JHD) Plan 11'dedir.

**Architecture:**
- **Store (Rust):** `add_chart`, `delete_chart` ve `open_client` tek SQLite işleminde yazar; `Store::now()` kesin artan milisaniye verir (Task 1). Etiket kimliği NFC'dir ve `clients_with_tag` etiket süzgecini sağlar (Task 2). Not gövdesi 64 KiB ile sınırlıdır (Task 9). İçe aktarmada dosyadan olduğu gibi saklanan alanlar sınırlıdır (Task 10).
- **Komutlar:** `list_clients_by_tag`, `JoinError → task_failed` (Task 3); yol almayan `export_clients_json`, `export_clients_csv`, `import_clients_json` — pencere Rust'ta `tauri-plugin-dialog` ile açılır, yol alan eski dört komut kayıttan çıkar (Task 10).
- **Durum:** görünüm kayıtları sıralıdır (C5) ve okunamayan düzende tutulur (Task 4). `Session` kimlikle açar, kapatır, etiket günceller ve `revision` ile listeleri yeniler (Task 6). Sekme yoktur: tek açık harita.
- **Arayüz:** `clients/` kenar çubuğu (Task 7) ve veri bölümü (Task 10), `palette/` komut paleti (Task 8), `notes/` notlar paneli (Task 9), `shell/ConfirmDialog` (Task 7). Kısayollar yalnızca `shortcuts.ts`'te (C7): `⌘K`, `⌘\` (Task 5).
- **Testler:** her görev Vitest ile; Task 11 Playwright WebKit e2e (IPC taklidi) ve CI izleri.

**Tech Stack:** React 19, Vite 8, TypeScript 7 (`strict`), Vitest 5 (jsdom), i18next 27, Playwright 1.65.1 (WebKit), Tauri 2.12.7, tauri-specta rc.29, rusqlite (FTS5). **Yeni Rust bağımlılıkları:** `tauri-plugin-dialog` `=2.8.4` (Apache-2.0 OR MIT; ağaca `rfd` 0.17.2 MIT, `tauri-plugin` 2.7.4 ve `tauri-plugin-fs` 2.7.1 Apache-2.0 OR MIT ekler; MSRV 1.90), `icu_normalizer` `=2.4.0` (Unicode-3.0; `default-features = false`, `compiled_data`; kilit dosyasında zaten vardı, MSRV 1.88). İkisi de `cargo deny check`'ten geçer. npm bağımlılığı değişmez.

**Spec:** `docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md` §4 ("Danışan açma", "Kalıcılık", `Cmd+K`), §5 (şema, "İçe ve dışa aktarma"), §7 (`{code, params}`), §8 (`Cmd+K` → ad → `Enter`, form yok), §11 (M6); `docs/superpowers/specs/2026-11-24-ui-design.md` §3.3 (notlar paneli), §4 (kenar çubuğu, palet, "Sekme yoktur"), §5, §6, §8.

**Ön koşul:** Taban commit `0424a42`. Plan 09'un elle duman testi Task 1'i engellemez (Fable kararı, Plan 10 incelemesi §1): Plan 09 Task 14 adım 5 ile bu planın Task 12 adım 5'i **tek kapı** olarak Plan 10 push'undan önce gerçek uygulamada bir kişi tarafından çalıştırılır ve sonuçlar `progress.md`'ye yazılır. O zamana kadar hiçbir Plan 10 commit'i push edilmez ve M5 kapanmış sayılmaz. M5'e ait hata ayrı commit'le (`fix(m5): …`) push'tan önce düzeltilir. Plan 11 Task 1, Plan 10 push'undan önce başlamaz. Bu erteleme bir kez kullanılır. Çıkış denetimi (Task 12 adım 5 madde 9) görünüm durumu ya da not metni kaybettirirse önceden onaylı düzeltme Rust'ta bir `CloseRequested` işleyicisidir: kapanmayı engeller, bir boşaltma olayı yayar, arayüzün onayını en çok 1 s bekler, sonra kapatır; push'tan önce iner.

## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni bağımlılık yalnızca tam sürümle (`=x.y.z`) ve AGPL uyumlu lisansla eklenir; bağımlılık değişen her görevde (2, 10) `cargo deny check` `advisories ok, bans ok, licenses ok, sources ok` verir.
- "Kala", "Kāla", "Kaala", "Cala" ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Türkçe sözcükler de kapsamdadır ("kalan", "yarım kalan", "kalabilir", "yakalama"): eklenen satırlarda `k(a|ā|aa)la` ve `\bcala\b` bulunmaz. Her commit adımı eklenen satırları ve dosya adlarını `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile tarar; çıktı boş olmalıdır (C13). Bu planın uygulanmasında iki yorum bu kuralla yeniden yazıldı ("kalan" → "diğer", "yarım kalan" → "yarıda kesilen").
- Kapılar: `pnpm --dir app test`, `pnpm --dir app build` (`tsc --noEmit`, `tsc --noEmit -p e2e`, `vite build`) her görev sonunda yeşil; `pnpm --dir app e2e` Task 11 ve 12'de yeşil. `tsc` `strict`. Tek dosya ya da klasör: `pnpm --dir app exec vitest run ui/<yol>`.
- Rust değişen görevlerde (1, 2, 3, 9, 10): `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, ilgili `cargo test`; kütüphane crate'i (`store`) için MSRV 1.93: `PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store --all-targets` (CI ayrıca `cargo +1.93 check -p store -p jyotish-desk --all-targets` çalıştırır; Task 10 ikisini birden denetler).
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH"` (oturum başına bir kez; `cargo deny` ve `cargo tree` `~/.cargo/bin`'dedir). Node 24+ (yerelde 26), pnpm 11.8.0. SE ephemeris yolu 242 baytın altında kalmalıdır (kopyada kısa yol, ör. `/Users/ttd/jd-p10`).
- **Üretilen dosyalar elle düzenlenmez (C10):** IPC tipi değişince `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings` (Task 3, 10); komut çıktısı değişince `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture` (Task 1). `Cargo.lock` cargo'nun çözümüdür.
- **Taşınan kurallar (Plan 09 son incelemesi C1–C14; Plan 10 incelemesi C2′, C15–C17), her görevin gereksinimidir ve Plan 11'i de bağlar:**
  - **C1** Gizlilik: danışan adları, not gövdeleri, doğum verisi ve `CommandError.params` loglanmaz. `no-console.test.ts` durur; izin listesi büyümez. Panik iletisi `task_failed`'e taşınmaz (Task 3); not metni hata iletisine girmez (Task 9); dosya hatalarında yalnızca dosya adı (Task 10).
  - **C2** (Task 10'a kadar) İçe/dışa aktarma komutları Plan 08 ertelenen madde 1 ve 3 kapanana kadar `api.ts` dışında kalır; `api.test.ts` denetimi yalnızca onları kapatan görevce değiştirilir. Bu plan madde 3'ü JSON için kapatır (Task 10): denetim "yol alan komut yok, JHD yok" biçimine güçlenir; madde 1 (JHD) açık kalır. Task 10'dan sonra yerini C2′ alır.
  - **C2′** (C2'nin yerine) Hiçbir IPC komutu arayüzden dosya yolu almaz; dosya pencereleri Rust'ta açılır. Yetenek dosyası hiçbir `dialog:` ya da `fs:` izni vermez ve `tauri_plugin_fs` hiçbir zaman başlatılmaz. `import_jhd` Plan 08 madde 1 kapanana kadar kayıtsız kalır. `api.test.ts` ve yetki testleri yalnızca madde 1'i kapatan görevce değiştirilir.
  - **C3** Göç V1 donmuştur; şema değişikliği yedekle başlayan göç 2'ye gider. **Bu plan şemayı değiştirmez** (D7).
  - **C4** Arayüz astrolojik hesap yapmaz. `formatJdDate` tek JD→tarih kodudur; "bugün" ve güncel dönem mantığı Rust komutunu bekler. Paletin metin süzmesi ve notların tarih biçimlemesi (`Intl.DateTimeFormat`) hesap değildir.
  - **C5** Görünüm yazımları harita ya da danışan değiştiren her özellikten önce sıralanır (tek uçuşta kayıt, son kazanır). `openChart` (ve yeni `closeChart`) önceki haritaya boşaltır ve kendisi kirli değişiklik değildir (Task 4, 6).
  - **C6** Tek düzen anahtarı (`app_state` `panel_layout`) ve iki ayrıştırıcıda tek sınır çifti (64 düzen, 48 panel); ayrıştırıcı değişikliği `layout-parity.json`'ı ve iki testi aynı commit'te günceller. Arayüz sınırın ötesine yazmaz. Bu plan ayrıştırıcılara dokunmaz.
  - **C7** Kısayollar yalnızca `shortcuts.ts`'e (`matchShortcut` + `SHORTCUT_LIST`) eklenir; hiçbir bileşen kendi genel tuş dinleyicisini eklemez (Shell'in tek dinleyicisi durur). Yerel Tauri menüsü eklendiğinde `⌘N`/`⌘E`/`⌘1…9`/`⌘K`/`⌘\` ona karşı denetlenir.
  - **C8** Her pencere `shell/Dialog`'u kullanır (`ConfirmDialog`, palet, dışa aktarma penceresi dahil). Pencere içeriği `summary`, `contenteditable` ya da gizli odaklanabilir öğe kazanırsa `FOCUSABLE` aynı görevde genişler (bu planda kazanmaz).
  - **C9** Renk yalnızca token'lardan gelir ve tek başına anlam taşımaz; `--warn` metin rengi değildir. Kuzey stilde lagna konumla taşınır; güney ve dairede "Lg" etiketi kalır. Bu planda: açık danışan satırı `aria-current` + zemin; paletin etkin öğesi zemin + 2 px çizgi; tehlikeli düğme yalnızca kenar rengi + metin; `info` bildirimi kenar + "Bilgi" metni; haritaya bağlı not "bu harita" metniyle.
  - **C10** Yukarıdaki "Üretilen dosyalar" maddesi; motor sürümü artışında da fikstür yeniden üretilir.
  - **C11** Görsel PNG temelleri yereldir ve yalnızca darwin'dir; harita çizimi değişirse temeller ve SVG anlık görüntüleri aynı commit'te yeniden üretilir. Fontlar ve WebKit sabitlenene kadar CI'da koşmaz. Bu plan çizimi değiştirmez; görsel test kenar çubuğunu kapatarak eski temelleri korur (Task 11).
  - **C12** Arayüzden ulaşılabilen her yeni `CommandError.code` aynı görevde tr/en/fr metni alır ve `i18n.test.ts` `BACKEND_CODES`'a eklenir (Task 3: `task_failed`; Task 10: `file_io`, `dialog_path`, `import_too_large`, `import_format`, `import_version`, `import_record`).
  - **C13** Yasaklı sözcük taraması her commit'te eklenen satırlarda, dosya adlarında ve commit mesajlarında; `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** Plan 07'nin gün doğumu sarmalayıcı toleransı ve yüksek enlem kuralı değişmez (bu plan dokunmaz).
  - **C15** Arayüz kodu hiçbir zaman `alert`, `confirm` ya da `prompt` çağırmaz (dialog eklentisi onların yerini alır); onaylar `shell/ConfirmDialog` ile alınır.
  - **C16** (C7'yi genişletir) ⌘ kısayolları metin girişinden de çalışır; orada yalnızca değiştiricisiz tuşlar yok sayılır. Yeni kısayol metin düzenleme akorlarını (⌘A/C/V/X/Z, ⌘ok tuşları) almaz.
  - **C17** Her otomatik kaydedilen metin (görünüm durumu, notlar, Plan 11'in ekleyeceği her şey) `visibilitychange` hidden ve `pagehide` olayında boşaltılır ve bileşeni kaldırıldıktan sonra başarısız olan kaydı bildirir.
- **Plan 08'den taşınan kurallar:** `CommandError` parametreleri loglanmaz ve arayüzden başka yere gönderilmez (serde `detail` ve `import_record.item` dosya içeriği taşıyabilir; Task 10 120 karakterde keser). JHD içe aktarma arayüzden erişilemez (madde 1 açık). Göç V1 düzenlenmez.
- **Erişilebilirlik:** WCAG AA; her işlem klavyeyle yapılır; odak halkası 2 px `--ink`; modal pencerede odak tuzağı ve geri dönüş (`shell/Dialog`); geri alınamaz işlemde odak "Vazgeç"te başlar; devre dışı düğmenin nedeni `aria-describedby` ile bağlıdır; liste ve sonuç sayısı `role=status` ile duyurulur.
- Commit mesajları İngilizce, `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` satırıyla biter.
- Kod blokları kopyada (`/Users/ttd/jd-p10`, Fable düzeltmeleriyle `/Users/ttd/jd-f10`) uygulanan görev commit'lerinden alınmıştır: yeni dosyalar tam içerikle, değişen dosyalar `git diff -U3` birleşik farkıyla (`git apply` ile uygulanabilir). Bir fark bağlamı tutmazsa önceki görevin adımını denetle; elle "düzeltme" yapma.

### Bu planın kararları

| # | Konu | Karar | Gerekçe |
|---|---|---|---|
| D1 | M6 tek plan mı | **Bölünür.** Plan 10 = danışan veritabanı dilimi (store işlemleri ve etiket kimliği, kenar çubuğu, `Cmd+K`, notlar, etiketler, oturum ve ikinci harita, JSON/CSV içe/dışa aktarma ve kapıları, e2e). **Plan 11** = fark rozeti (tzdb farkını kabul akışı, Plan 08 Task 8 "D13 sessiz yolu"), ayarlar arayüzü (`CalcSettings`/`AppPrefs`, `AppPrefs.schema_version` denetimi), transit katmanı (halka, `transit_table`, tarih seçimi, açık/kapalı kaydı), `birth_info` paneli ve oradaki rozetler, `dasha_strip`, cinsiyet alanı, M6-1, M6-5, M6-6, M6-7, M6-9, JHD gerçek dosya denetim listesi + JHD arayüzü + toplu içe aktarma, Plan 09 kapsam dışı maddeleri 13–15. M6-1, M6-6 ve M6-7 M6'yı kapatmadan önce iner ve bir kez daha ertelenemez | M6 spec'te sekiz alan; her biri kendi Rust ucu ve paneliyle 4–6 görevdir. Tek plan ~25 görev olurdu. Seçilen dilim kendi başına çalışan yazılımdır (spec §8 kabul ölçütü dahil) ve Plan 11'in her parçası bu dilimin `Session`, kenar çubuğu ve bildirim altyapısına dayanır |
| D2 | İçe/dışa aktarmanın erişilebilirliği | **JSON içe/dışa ve CSV dışa aktarma bu planda açılır; JHD kapalı kalır.** Pencere Rust'ta açılır (`tauri-plugin-dialog` `=2.8.4`, Apache-2.0 OR MIT, `DialogExt::blocking_save_file`/`blocking_pick_file`, `spawn_blocking` içinde); arayüz yol göndermez, eklenti JS komutlarına yetenek verilmez; yol alan dört komut kayıttan çıkar. Kapılar: okumadan önce 32 MiB sınırı (`import_too_large`), atomik yazım, parametrelerde yalnızca dosya adı, 120 karakter kesme, korunan an alanlarında 32 karakter / ±18 saat, not gövdesi sınırı. JHD'nin 64 KiB sınırı, toplu içe aktarma ve kilit bırakma JHD ile Plan 11'e | Plan 08 madde 3'ün JSON için bütün ölçütleri (boyut, yeni kod, kısmi okuma yok, ad, kesme) karşılanır; madde 1 gerçek dosya olmadan kapatılamaz. Rust tarafı pencere yol enjeksiyonunu imkânsız kılar (WebView ele geçirilse bile keyfi yol yazamaz). 5000 danışanlık dışa aktarma 9,7 MB'dır (Plan 08 ölçümü); 32 MiB üç kat pay bırakır |
| D3 | `icu_normalizer` | `=2.4.0`, `default-features = false`, `compiled_data`; Unicode-3.0 (`deny.toml` izinli). NFC yalnızca etiket anahtarında ve saklanan etiket adında | Kilit dosyasında zaten vardı (`idna` yolu): tek satırlık kilit farkı, yeni kod indirilmez. FTS5 ad araması `unicode61 remove_diacritics` ile zaten iki biçimi eşler; sorun yalnızca etiket anahtarıydı |
| D4 | Sekme yok, `Cmd+Enter` | UI spec §4 "Sekme yoktur": tek açık harita. Palette `Cmd+Enter` danışanı açar **ve** kenar çubuğunu gösterir (yeni sekme anlamı yerine). Oturum geri yüklemesi "son harita + düzen"dir (Plan 09) | Spec'in sekme benzeri ifadesi bu planda tek açık harita modeline eşlenir; çoklu açık harita (karşılaştırma) v1 dışı |
| D5 | Okunamayan düzende kayıt (M6-3) | `load_panel_layout` hata dönerse bildirim gösterilir ve oturum boyunca **bütün** görünüm kayıtları (düzen ve dasha yolu) tutulur; bildirim kaydın tutulduğunu söyler | Kısmi tutma (yalnızca düzen) `save_view_state`'in düzeni de yazması yüzünden imkânsız; yeniden deneme döngüsü kullanıcıya görünmeyen yazım sırası yaratır |
| D6 | `task_failed` parametreleri | Boş. `JoinError` paniği iletisi taşıyabilir (danışan adı içeren `expect` iletisi gibi) | C1 |
| D7 | Şema | Değişmez; göç yok. NFD etiket anahtarı yok sayılır: M5'te etiket arayüzü yoktu, saklanmış anahtarlar yalnızca Plan 08 içe aktarma testlerinden gelir. `chart_tags(tag_id)` indeksi (Plan 08 madde 14) sonraki göçe | C3; yedekli göç 2 bu planın işine değmez |
| D8 | Etiket sırası | Arayüzde `Intl.Collator(dil, { sensitivity: "base" })`; Rust sırası (NOCASE, ASCII) olduğu gibi gelir | Türkçe "ç/ö/ş" sırası; Rust'a ICU sıralaması eklemek büyük bağımlılıktır |
| D9 | Notların kaynağı | Panel `list_notes(clientId, 100)` okur; `recent_notes` kullanılmaz. Bütün not kayıtları tek sırada; liste sıradaki kayıtlar bitince okunur | `recent_notes` bu oturumdaki düzenlemelerden sonra eskir; eski metinle açılan düzenleyici sonraki yazımda yeni metni ezerdi |
| D10 | Not gövdesi sınırı | Rust `MAX_NOTE_BYTES = 65 536` (ekleme, güncelleme, içe aktarma); arayüz `maxLength = 16 384` | Sınırsız gövde FTS ve IPC'yi şişirir; 16 384 UTF-16 birimi en çok 49 152 UTF-8 bayttır, kullanıcı Rust hatasını görmez |
| D11 | Dışa aktarma kapsamı | Bütün danışanlar; seçili danışan dışa aktarma (`ExportOptions.client_ids`) arayüzde yok | Spec §5 seçim istemez; komut alanı ileride kullanılabilir |
| D12 | Bildirim türü `info` | Başarılı dışa/içe aktarma `role=status` `info` bildirimi; ayarı sıfırlanan harita varsa `warning` | Sonuç kullanıcıya duyurulmalı; hata değildir |
| D13 | `last_opened_at` eşitliği | `Store::now()` kesin artan (`max(saat, son + 1 ms)`); `ORDER BY last_opened_at DESC, id DESC` değişmez | Plan 09 Task 3 ertelemesi; saat geri giderse de sıra açılış sırasıdır |

### Spec belirsizlikleri ve çözümleri

- **"Sekme" (spec §4 palet, `Cmd+Enter`):** UI spec §4 sekme olmadığını söyler; D4.
- **Notlar paneli ve harita bağı:** spec notu "isteğe bağlı olarak bir haritaya" bağlar. Yeni not açık haritaya bağlanır; panel danışanın bütün notlarını gösterir ve açık haritanınkileri "bu harita" ile işaretler. Bağı değiştiren arayüz yoktur.
- **Dışa aktarma dosya adı:** spec belirtmez; varsayılan `jyotish-desk-clients.json` ve `jyotish-desk-charts.csv` (pencerede değiştirilebilir, dile bağlı değil).
- **Etiket girişi biçimi:** spec belirtmez; virgülle ayrılmış metin kutusu, Rust normalleştirir (Plan 08 `normalize_tags`: kırpma, 64 karakter, büyük/küçük harf duyarsız tekilleştirme).
- **İçe aktarma uyarısı:** JSON içe aktarmada tek uyarı `CalcSettingsReset`'tir (JHD kapalı); metin buna göre yazılmıştır.

## Dosya Yapısı (bu plan)

```
crates/store/
  Cargo.toml                  icu_normalizer
  src/db.rs                   kesin artan now(), stuck_store
  src/open.rs                 açılış tek işlem; settle_settings/check_tzdb serbest fonksiyon
  src/charts.rs               add_chart/delete_chart tek işlem
  src/notes.rs                NFC etiket; MAX_NOTE_BYTES
  src/search.rs               clients_with_tag
  src/exchange.rs             içe aktarma alan sınırları
  src/lib.rs                  MAX_NOTE_BYTES dışa aktarımı
app/src-tauri/
  Cargo.toml                  tauri-plugin-dialog
  src/client_commands.rs      task_failed, MAX_PARAM_CHARS kesme
  src/store_commands.rs       list_clients_by_tag; pencereli içe/dışa aktarma, write_atomic, read_bounded
  src/lib.rs                  komut kaydı, dialog eklentisi
app/ui/
  api.ts                      danışan veritabanı ve içe/dışa aktarma sarmalayıcıları
  boot.ts, App.tsx            layoutError, kayıt tutma, kayıt bildirimi
  keyboard.ts, shortcuts.ts   isTextEntry, ⌘K, ⌘\
  view/{persistence,reducer}.ts, view/store.tsx   sıralı kayıt, closeChart
  session/{store.tsx,birth.ts}                     openClient/close/setTags/touch, timeUnknown
  shell/{ConfirmDialog.tsx,Shell.tsx,TopBar.tsx,notices.tsx,shell.css}
  birth/BirthForm.tsx         "add" kipi
  clients/{Sidebar,ClientList,OpenClient,TagEditor,DataActions}.tsx, clients.css   (yeni)
  palette/{items.ts,Palette.tsx,palette.css}       (yeni)
  notes/{saves.ts,NoteEditor.tsx,NotesPanel.tsx,notes.css}   (yeni)
  panels/registry.ts          notes → NotesPanel
  i18n/{tr,en,fr}.ts          clients, palette, notes, data, yeni hata ve bildirim metinleri
app/e2e/{ipc.ts,desk.spec.ts,visual.spec.ts}
.github/workflows/ci.yml      Playwright izleri
```

Kapsamı (M6 maddeleri ve ertelenenler):

| Madde | Yer |
|---|---|
| Plan 08 madde 9 (işlemler) ve `last_opened_at` eşitliği | Task 1 |
| Plan 08 Task 7 NFD etiketleri | Task 2 |
| Plan 08 madde 11 (`task_failed`) | Task 3 (kilit zehirlenmesi M8) |
| C5, M6-3, M6-8a | Task 4 |
| M6-2, C16 | Task 5 |
| M6-8b, M6-10 | Task 6 |
| Plan 08 Task 6 etiket sırası | Task 7 |
| Plan 08 madde 3 (JSON), madde 13, Task 9/13 küçük bulguları (ad, kesme, sınır, atomik yazım, ince testler) | Task 10 |
| M6-4 | Task 11 |
| Plan 08 madde 1, 10, 14; M6-1, 5, 6, 7, 9 | Bu planın kapsamadıkları (gerekçeli) |

---

### Task 1: Store: tek işlemde harita ekleme, silme ve açılış; kesin artan zaman damgaları

Plan 08 ertelenen madde 9 ve Plan 09 Task 3'ün `last_opened_at` milisaniye eşitliği. `add_chart` iki yazımı (harita + danışanın `updated_at`'ı), `delete_chart` sayımı ve silmeyi, `open_client` ayar yerleşimini, tzdb denetimini ve iki `last_opened_at` güncellemesini tek SQLite işleminde yapar; okumalar işlemden önce. `Store::now()` aynı milisaniyede çağrılsa da kesin artan damga verir (`last_ms + 1`), böylece "son açılan" sıralaması aynı milisaniyede açılan iki danışanda da açılış sırasını izler. Hata enjeksiyonu geçici (`TEMP`) tetikleyicilerle yapılır: tetikleyici ikinci yazımı düşürür, test ilk yazımın da geri alındığını görür.

Zaman damgası değişikliği `ui_fixture`'ın sabit saatli çıktısını değiştirir (`last_opened_at` `…00.000Z` yerine `…00.001Z`): C10 gereği fikstür aynı görevde yeniden üretilir.

**Files:**
- Modify: `crates/store/src/charts.rs`
- Modify: `crates/store/src/db.rs`
- Modify: `crates/store/src/open.rs`
- Modify: `app/ui/test/fixtures/delhi.json` (üretilir: `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture`)

**Interfaces:**
- Consumes: `Store` (Plan 08), `db::testutil::store()`.
- Produces: `db::testutil::stuck_store() -> Store` (saat sabit, 1_764_000_000 sn; yalnızca testler); `Store::now()` kesin artan (genel API değişmez); `open.rs` `settle_settings(conn: &Connection, …)`, `check_tzdb(conn: &Connection, …)` serbest fonksiyonlar (modül içi). Davranış: `add_chart`, `delete_chart`, `open_client` ya tamamen yazar ya hiç.

- [ ] **Step 1: Testleri ve uygulamayı yaz**

Önce farkların `#[cfg(test)] mod tests` bölümlerini uygula (`db.rs` `stuck_store` ve `timestamps_strictly_increase_when_the_clock_does_not`, `open.rs` iki test, `charts.rs` bir test):

```bash
cargo test -p store --lib
```
Expected: `timestamps_strictly_increase_when_the_clock_does_not`, `last_opened_follows_open_order_within_one_millisecond`, `open_writes_are_one_transaction` ve `add_chart_writes_the_chart_and_the_client_together` FAIL (aynı milisaniyede iki kez `.000Z`; tetikleyici düşürdükten sonra ilk yazım kalıcı).

Sonra farkların tamamını uygula.

`crates/store/src/charts.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/crates/store/src/charts.rs
+++ b/crates/store/src/charts.rs
@@ -323,8 +323,10 @@ impl Store {
         let (birth, resolved) = resolve_birth(&chart.birth)?;
         let calc_json = serde_json::to_string(&self.app_prefs()?.calc_settings)?;
         let now = self.now();
+        // Harita ve danışanın `updated_at`'i birlikte yazılır (Plan 08 ertelenen madde 9).
+        let tx = self.conn.transaction()?;
         let id = insert_chart(
-            &self.conn,
+            &tx,
             &ChartRow {
                 uuid: &Uuid::new_v4().to_string(),
                 client_id,
@@ -338,10 +340,11 @@ impl Store {
                 updated_at: &now,
             },
         )?;
-        self.conn.execute(
+        tx.execute(
             "UPDATE clients SET updated_at = ?1 WHERE id = ?2",
             params![now, client_id],
         )?;
+        tx.commit()?;
         self.chart(id)
     }
 
@@ -566,7 +569,9 @@ impl Store {
     /// danışanda kalır (`chart_id` boşalır).
     pub fn delete_chart(&mut self, id: u32) -> Result<(), StoreError> {
         let chart = self.chart(id)?;
-        let count: u32 = self.conn.query_row(
+        // Sayım ve silme aynı işlemde: arada eklenen/silinen harita kuralı bozamaz.
+        let tx = self.conn.transaction()?;
+        let count: u32 = tx.query_row(
             "SELECT count(*) FROM charts WHERE client_id = ?1",
             params![chart.client_id],
             |r| r.get(0),
@@ -574,7 +579,6 @@ impl Store {
         if count <= 1 {
             return Err(StoreError::LastChart);
         }
-        let tx = self.conn.transaction()?;
         tx.execute("DELETE FROM charts WHERE id = ?1", params![id])?;
         prune_tags(&tx)?;
         tx.commit()?;
@@ -815,6 +819,23 @@ mod tests {
         ));
     }
 
+    #[test]
+    fn add_chart_writes_the_chart_and_the_client_together() {
+        let mut s = store();
+        let (c, _) = s.create_client(&new_client("A")).unwrap();
+        s.conn
+            .execute_batch(
+                "CREATE TEMP TRIGGER fail_touch BEFORE UPDATE OF updated_at ON clients
+                 BEGIN SELECT RAISE(ABORT, 'fail'); END;",
+            )
+            .unwrap();
+        assert!(matches!(
+            s.add_chart(c.id, &new_client("A").chart),
+            Err(StoreError::Sqlite(_))
+        ));
+        assert_eq!(s.charts_of(c.id).unwrap().len(), 1);
+    }
+
     #[test]
     fn list_and_update_clients() {
         let mut s = store();
```

`crates/store/src/db.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/crates/store/src/db.rs
+++ b/crates/store/src/db.rs
@@ -16,6 +16,8 @@ pub type Clock = Box<dyn FnMut() -> Timestamp + Send>;
 pub struct Store {
     pub(crate) conn: Connection,
     clock: Clock,
+    /// Son verilen zaman damgası (ms). `now` bundan büyük bir değer verir.
+    last_ms: Option<i64>,
 }
 
 impl Store {
@@ -48,6 +50,7 @@ impl Store {
         Ok(Store {
             conn,
             clock: Box::new(Timestamp::now),
+            last_ms: None,
         })
     }
 
@@ -57,9 +60,16 @@ impl Store {
         self
     }
 
-    /// Şimdi, `format_ts` biçiminde.
+    /// Şimdi, `format_ts` biçiminde. Aynı `Store`'dan gelen damgalar milisaniye düzeyinde
+    /// kesin artandır: aynı milisaniyedeki iki açılış ya da saat geri giderse bir önceki
+    /// damganın 1 ms sonrası verilir. Böylece `last_opened_at` sırası açılış sırasıdır.
     pub fn now(&mut self) -> String {
-        format_ts((self.clock)())
+        let mut ms = (self.clock)().as_millisecond();
+        if let Some(last) = self.last_ms {
+            ms = ms.max(last + 1);
+        }
+        self.last_ms = Some(ms);
+        format_ts(Timestamp::from_millisecond(ms).unwrap_or(Timestamp::MAX))
     }
 
     /// Veritabanının şema sürümü (`PRAGMA user_version`).
@@ -125,13 +135,30 @@ pub(crate) mod testutil {
             .expect("open in memory")
             .with_clock(step_clock())
     }
+
+    /// Hiç ilerlemeyen saat (2025-11-24T16:00:00Z): aynı milisaniyedeki işlemler için.
+    pub fn stuck_store() -> Store {
+        Store::open_in_memory()
+            .expect("open in memory")
+            .with_clock(Box::new(|| {
+                Timestamp::from_second(1_764_000_000).expect("valid timestamp")
+            }))
+    }
 }
 
 #[cfg(test)]
 mod tests {
-    use super::testutil::store;
+    use super::testutil::{store, stuck_store};
     use super::*;
 
+    #[test]
+    fn timestamps_strictly_increase_when_the_clock_does_not() {
+        let mut s = stuck_store();
+        assert_eq!(s.now(), "2025-11-24T16:00:00.000Z");
+        assert_eq!(s.now(), "2025-11-24T16:00:00.001Z");
+        assert_eq!(s.now(), "2025-11-24T16:00:00.002Z");
+    }
+
     #[test]
     fn fresh_database_has_schema_and_foreign_keys() {
         let s = store();
```

`crates/store/src/open.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/crates/store/src/open.rs
+++ b/crates/store/src/open.rs
@@ -3,7 +3,7 @@
 //! uygulama katmanındadır.
 
 use jyotish_core::CalcSettings;
-use rusqlite::{params, OptionalExtension};
+use rusqlite::{params, Connection, OptionalExtension};
 use serde::{Deserialize, Serialize};
 
 use crate::birth::{parse_birth, resolve_parsed};
@@ -72,22 +72,25 @@ impl Store {
             None => self.pick_chart(&client, &mut warnings)?,
         };
         let mut chart = self.chart(chart_id)?;
-
         let load = self.calc_settings(chart_id)?;
-        let calc_settings = self.settle_settings(chart_id, load, &mut warnings)?;
         let view_state = self.view_state(chart_id)?;
-        self.check_tzdb(&mut chart, &mut warnings)?;
         let recent_notes = self.notes(client_id, RECENT_NOTES)?;
 
+        // Ayar göçü, tzdb sürümü ve "son açılan" işaretleri tek işlemde yazılır: biri
+        // başarısız olursa hiçbiri kalmaz (Plan 08 ertelenen madde 9).
         let now = self.now();
-        self.conn.execute(
+        let tx = self.conn.transaction()?;
+        let calc_settings = settle_settings(&tx, chart_id, load, &mut warnings)?;
+        check_tzdb(&tx, &mut chart, &mut warnings)?;
+        tx.execute(
             "UPDATE clients SET last_chart_id = ?1, last_opened_at = ?2 WHERE id = ?3",
             params![chart_id, now, client_id],
         )?;
-        self.conn.execute(
+        tx.execute(
             "UPDATE charts SET last_opened_at = ?1 WHERE id = ?2",
             params![now, chart_id],
         )?;
+        tx.commit()?;
         client.last_chart_id = Some(chart_id);
         client.last_opened_at = Some(now.clone());
         chart.last_opened_at = Some(now);
@@ -101,33 +104,6 @@ impl Store {
         })
     }
 
-    /// Ayar okumasının sonucunu uygular: göç edilmiş ayar yeni JSON olarak yazılır; sıfırlanan
-    /// ayar uyarı verir ve saklanan JSON'a dokunulmaz.
-    fn settle_settings(
-        &self,
-        chart_id: u32,
-        load: SettingsLoad,
-        warnings: &mut Vec<StoreWarning>,
-    ) -> Result<CalcSettings, StoreError> {
-        Ok(match load {
-            SettingsLoad::Current(s) => s,
-            SettingsLoad::Migrated { settings, .. } => {
-                self.conn.execute(
-                    "UPDATE charts SET calc_settings_json = ?1 WHERE id = ?2",
-                    params![serde_json::to_string(&settings)?, chart_id],
-                )?;
-                settings
-            }
-            SettingsLoad::Reset {
-                settings,
-                found_version,
-            } => {
-                warnings.push(StoreWarning::CalcSettingsReset { found_version });
-                settings
-            }
-        })
-    }
-
     fn pick_chart(
         &self,
         client: &Client,
@@ -160,35 +136,6 @@ impl Store {
             })
     }
 
-    /// Saklanan tzdb sürümü güncel değilse ve saat dilimi IANA ise an yeniden çözülür.
-    /// Sonuç aynıysa sürüm sessizce güncellenir; farklıysa ya da çözülemezse uyarı.
-    fn check_tzdb(
-        &self,
-        chart: &mut ChartRecord,
-        warnings: &mut Vec<StoreWarning>,
-    ) -> Result<(), StoreError> {
-        let current = atlas::tzdb_version();
-        if chart.resolved.tzdb_version == current || !matches!(chart.birth.tz, TzChoice::Iana(_)) {
-            return Ok(());
-        }
-        let fresh = parse_birth(&chart.birth).and_then(|p| resolve_parsed(&p));
-        match fresh {
-            Ok(r) if r.utc == chart.resolved.utc => {
-                self.conn.execute(
-                    "UPDATE charts SET tzdb_version = ?1 WHERE id = ?2",
-                    params![current, chart.id],
-                )?;
-                chart.resolved.tzdb_version = current.to_owned();
-            }
-            other => warnings.push(StoreWarning::TzdbChanged {
-                stored_version: chart.resolved.tzdb_version.clone(),
-                current_version: current.to_owned(),
-                current_utc: other.ok().map(|r| r.utc),
-            }),
-        }
-        Ok(())
-    }
-
     /// Oturum geri yükleme: en son açılan danışan ve haritası.
     pub fn last_opened(&self) -> Result<Option<(u32, u32)>, StoreError> {
         Ok(self
@@ -203,11 +150,67 @@ impl Store {
     }
 }
 
+/// Ayar okumasının sonucunu uygular: göç edilmiş ayar yeni JSON olarak yazılır; sıfırlanan
+/// ayar uyarı verir ve saklanan JSON'a dokunulmaz.
+fn settle_settings(
+    conn: &Connection,
+    chart_id: u32,
+    load: SettingsLoad,
+    warnings: &mut Vec<StoreWarning>,
+) -> Result<CalcSettings, StoreError> {
+    Ok(match load {
+        SettingsLoad::Current(s) => s,
+        SettingsLoad::Migrated { settings, .. } => {
+            conn.execute(
+                "UPDATE charts SET calc_settings_json = ?1 WHERE id = ?2",
+                params![serde_json::to_string(&settings)?, chart_id],
+            )?;
+            settings
+        }
+        SettingsLoad::Reset {
+            settings,
+            found_version,
+        } => {
+            warnings.push(StoreWarning::CalcSettingsReset { found_version });
+            settings
+        }
+    })
+}
+
+/// Saklanan tzdb sürümü güncel değilse ve saat dilimi IANA ise an yeniden çözülür.
+/// Sonuç aynıysa sürüm sessizce güncellenir; farklıysa ya da çözülemezse uyarı.
+fn check_tzdb(
+    conn: &Connection,
+    chart: &mut ChartRecord,
+    warnings: &mut Vec<StoreWarning>,
+) -> Result<(), StoreError> {
+    let current = atlas::tzdb_version();
+    if chart.resolved.tzdb_version == current || !matches!(chart.birth.tz, TzChoice::Iana(_)) {
+        return Ok(());
+    }
+    let fresh = parse_birth(&chart.birth).and_then(|p| resolve_parsed(&p));
+    match fresh {
+        Ok(r) if r.utc == chart.resolved.utc => {
+            conn.execute(
+                "UPDATE charts SET tzdb_version = ?1 WHERE id = ?2",
+                params![current, chart.id],
+            )?;
+            chart.resolved.tzdb_version = current.to_owned();
+        }
+        other => warnings.push(StoreWarning::TzdbChanged {
+            stored_version: chart.resolved.tzdb_version.clone(),
+            current_version: current.to_owned(),
+            current_utc: other.ok().map(|r| r.utc),
+        }),
+    }
+    Ok(())
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
     use crate::charts::testutil::new_client;
-    use crate::db::testutil::store;
+    use crate::db::testutil::{store, stuck_store};
     use crate::settings::AppPrefs;
     use jyotish_core::{AyanamsaSystem, Graha};
     use proptest::prelude::*;
@@ -359,16 +362,16 @@ mod tests {
             ..CalcSettings::default()
         };
         let mut warnings = Vec::new();
-        let got = s
-            .settle_settings(
-                ch.id,
-                SettingsLoad::Migrated {
-                    settings: migrated.clone(),
-                    from: 0,
-                },
-                &mut warnings,
-            )
-            .unwrap();
+        let got = settle_settings(
+            &s.conn,
+            ch.id,
+            SettingsLoad::Migrated {
+                settings: migrated.clone(),
+                from: 0,
+            },
+            &mut warnings,
+        )
+        .unwrap();
         assert_eq!(got, migrated);
         assert!(warnings.is_empty());
         let stored: String = s
@@ -381,16 +384,16 @@ mod tests {
             .unwrap();
         assert_eq!(stored, serde_json::to_string(&migrated).unwrap());
         // Reset: uyarı verir, saklanan JSON'a dokunmaz.
-        let reset = s
-            .settle_settings(
-                ch.id,
-                SettingsLoad::Reset {
-                    settings: CalcSettings::default(),
-                    found_version: Some(9),
-                },
-                &mut warnings,
-            )
-            .unwrap();
+        let reset = settle_settings(
+            &s.conn,
+            ch.id,
+            SettingsLoad::Reset {
+                settings: CalcSettings::default(),
+                found_version: Some(9),
+            },
+            &mut warnings,
+        )
+        .unwrap();
         assert_eq!(reset, CalcSettings::default());
         assert_eq!(
             warnings,
@@ -409,6 +412,42 @@ mod tests {
         assert_eq!(after, stored);
     }
 
+    #[test]
+    fn open_writes_are_one_transaction() {
+        let mut s = store();
+        let (a, ch) = s.create_client(&new_client("A")).unwrap();
+        // Eski tzdb sürümü, aynı sonuç: açılış sürümü yazmak ister; son yazım başarısız olur.
+        s.conn
+            .execute(
+                "UPDATE charts SET tzdb_version = '1999a' WHERE id = ?1",
+                params![ch.id],
+            )
+            .unwrap();
+        s.conn
+            .execute_batch(
+                "CREATE TEMP TRIGGER fail_open BEFORE UPDATE OF last_opened_at ON charts
+                 BEGIN SELECT RAISE(ABORT, 'fail'); END;",
+            )
+            .unwrap();
+        assert!(matches!(
+            s.open_client(a.id, None),
+            Err(StoreError::Sqlite(_))
+        ));
+        assert_eq!(s.client(a.id).unwrap().last_opened_at, None);
+        assert_eq!(s.chart(ch.id).unwrap().resolved.tzdb_version, "1999a");
+        assert_eq!(s.last_opened().unwrap(), None);
+    }
+
+    #[test]
+    fn last_opened_follows_open_order_within_one_millisecond() {
+        let mut s = stuck_store();
+        let (a, _) = s.create_client(&new_client("A")).unwrap();
+        let (b, _) = s.create_client(&new_client("B")).unwrap();
+        s.open_client(b.id, None).unwrap();
+        s.open_client(a.id, None).unwrap();
+        assert_eq!(s.last_opened().unwrap().map(|(c, _)| c), Some(a.id));
+    }
+
     proptest! {
         #![proptest_config(ProptestConfig::with_cases(64))]
         /// Spec §8: rastgele ya da bozuk `view_state_json` ile açılış asla hata dönmez.
```

- [ ] **Step 2: Gerçek veri fikstürünü yeniden üret (C10)**

```bash
cargo test -p jyotish-desk --test ui_fixture
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture
cargo test -p jyotish-desk --test ui_fixture
git diff --stat app/ui/test/fixtures/delhi.json
```
Expected: ilk çalıştırma `app/ui/test/fixtures/delhi.json is stale` ile FAIL; ikinci yazar; üçüncü `test result: ok. 2 passed`. Fark tek satırdır (`last_opened_at` `2026-11-25T10:00:00.001Z`). Dosya elle düzenlenmez.

- [ ] **Step 3: Testlerin geçtiğini gör**

```bash
cargo test -p store --lib
cargo test -p jyotish-desk
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store --all-targets
pnpm --dir app test
```
Expected: `store` lib `test result: ok. 68 passed` (+4); `jyotish-desk` lib 13, `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2; fmt ve clippy çıktısız; MSRV `Finished`; vitest `Test Files  36 passed (36)`, `Tests  299 passed (299)`.

- [ ] **Step 4: Commit**

```bash
git add app/ui/test/fixtures/delhi.json \
  crates/store/src/charts.rs \
  crates/store/src/db.rs \
  crates/store/src/open.rs
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git diff --cached --name-only | grep -iE 'k(a|ā|aa)la|\bcala\b|^\.superpowers/'   # boş olmalı
git commit -m "fix(store): write add_chart, delete_chart and open_client in one transaction; strictly increasing timestamps

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: commit oluşur; iki tarama da çıktısız. Commit mesajında yasaklı sözcük yoktur (C13).

---

### Task 2: Store: NFC etiket kimliği ve etiket süzgeci sorgusu

Plan 08 Task 7 küçük bulgusu ("NFD işaretleri bölünür") ve kenar çubuğunun etiket süzgeci. Etiket anahtarı (`tag_key`) ve saklanan etiket adı NFC'ye çevrilir; "ş" iki biçimde de (U+015F ya da s + U+0327) aynı etikettir. NFC `icu_normalizer` `=2.4.0` ile yapılır (Unicode-3.0; kilit dosyasında zaten vardı, `compiled_data` dışındaki özellikler kapalı). Göç gerekmez: M5'te etiket arayüzü yoktu, saklanan NFD anahtar yok sayılabilir (D7). `clients_with_tag` etiket anahtarına tam eşleşen en az bir haritası olan danışanları son açılan önce döndürür (`MAX_LIST` sınırı).

**Files:**
- Modify: `crates/store/Cargo.toml`
- Modify: `crates/store/src/notes.rs`
- Modify: `crates/store/src/search.rs`
- Modify: `Cargo.lock` (üretilir: `cargo` bağımlılığı çözerken günceller)

**Interfaces:**
- Consumes: `notes::tag_key`, `normalize_tags`, `charts::MAX_LIST`, `row_to_client` (Plan 08).
- Produces: `notes::nfc(text: &str) -> String` (`pub(crate)`); `Store::clients_with_tag(&self, tag: &str, limit: u32) -> Result<Vec<Client>, StoreError>`. Yeni bağımlılık: `icu_normalizer = { version = "=2.4.0", default-features = false, features = ["compiled_data"] }`.

- [ ] **Step 1: Testleri ve uygulamayı yaz**

Önce `notes.rs` ve `search.rs` farklarının `mod tests` bölümlerini uygula:

```bash
cargo test -p store --lib
```
Expected: derleme hatası `error[E0599]: no method named clients_with_tag found for struct Store`; `clients_with_tag` gövdesi yazılınca `decomposed_and_precomposed_tags_are_one_tag` FAIL (iki ayrı etiket).

Sonra farkların tamamını uygula.

`crates/store/Cargo.toml` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/crates/store/Cargo.toml
+++ b/crates/store/Cargo.toml
@@ -12,6 +12,9 @@ specta = ["dep:specta", "dep:specta-typescript", "jyotish-core/specta"]
 
 [dependencies]
 atlas = { path = "../atlas" }
+# Etiket kimliği için Unicode NFC (Plan 08 ertelenen madde 8). Ağaçta zaten var (url → idna);
+# yalnızca derlenmiş veri, ek özellik yok.
+icu_normalizer = { version = "=2.4.0", default-features = false, features = ["compiled_data"] }
 jyotish-core = { path = "../jyotish-core" }
 rusqlite = { version = "0.43", features = ["bundled"] }
 rusqlite_migration = "=2.9.0"
```

`crates/store/src/notes.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/crates/store/src/notes.rs
+++ b/crates/store/src/notes.rs
@@ -36,25 +36,34 @@ fn row_to_note(r: &Row<'_>) -> rusqlite::Result<Note> {
     })
 }
 
-/// Etiketin kimlik anahtarı (`tags.name_key`): Unicode küçük harf; ayrıca Türkçe dört "i"
-/// (`I`, `ı`, `İ`, `i`) tek harfe iner ("KARIYER", "KARİYER", "kariyer" aynı etikettir).
-/// Aksanlar korunur ("Öfke" ile "Ofke" ayrı etiketlerdir).
+/// Unicode NFC: "O" + U+0308 ile hazır "Ö" aynı dizgi olur (macOS dosya adları ve bazı
+/// klavyeler ayrışık biçim üretir).
+pub(crate) fn nfc(text: &str) -> String {
+    icu_normalizer::ComposingNormalizerBorrowed::new_nfc()
+        .normalize(text)
+        .into_owned()
+}
+
+/// Etiketin kimlik anahtarı (`tags.name_key`): NFC, sonra Unicode küçük harf; ayrıca Türkçe
+/// dört "i" (`I`, `ı`, `İ`, `i`) tek harfe iner ("KARIYER", "KARİYER", "kariyer" aynı
+/// etikettir). Aksanlar korunur ("Öfke" ile "Ofke" ayrı etiketlerdir).
 pub(crate) fn tag_key(tag: &str) -> String {
-    tag.to_lowercase()
+    nfc(tag)
+        .to_lowercase()
         .chars()
         .filter(|c| *c != '\u{307}')
         .map(|c| if c == 'ı' { 'i' } else { c })
         .collect()
 }
 
-/// Etiket listesini kırpar, boşları atar, büyük/küçük harf duyarsız (`tag_key`)
-/// tekilleştirir (ilk yazım korunur).
+/// Etiket listesini NFC'ye çevirir, kırpar, boşları atar, büyük/küçük harf duyarsız
+/// (`tag_key`) tekilleştirir (ilk yazım korunur).
 pub fn normalize_tags(tags: &[String]) -> Result<Vec<String>, StoreError> {
     let mut out: Vec<String> = Vec::new();
     let mut keys: Vec<String> = Vec::new();
     for t in tags {
-        let t = t.trim();
-        let key = tag_key(t);
+        let t = nfc(t.trim());
+        let key = tag_key(&t);
         if key.is_empty() {
             continue;
         }
@@ -63,7 +72,7 @@ pub fn normalize_tags(tags: &[String]) -> Result<Vec<String>, StoreError> {
         }
         if !keys.contains(&key) {
             keys.push(key);
-            out.push(t.to_owned());
+            out.push(t);
         }
     }
     Ok(out)
@@ -336,4 +345,29 @@ mod tests {
         // Yalnızca birleştirici noktadan oluşan etiket boş sayılır.
         assert!(normalize_tags(&["\u{307}".into()]).unwrap().is_empty());
     }
+
+    #[test]
+    fn decomposed_and_precomposed_tags_are_one_tag() {
+        let mut s = store();
+        let (_, a) = s.create_client(&new_client("A")).unwrap();
+        let (_, b) = s.create_client(&new_client("B")).unwrap();
+        // "O" + U+0308 (ayrışık) ilk yazımdır; saklanan ad NFC'dir.
+        let got = s.set_chart_tags(a.id, &["O\u{308}fke".into()]).unwrap();
+        assert_eq!(got, ["Öfke"]);
+        let got = s.set_chart_tags(b.id, &["öfke".into()]).unwrap();
+        assert_eq!(got, ["Öfke"]);
+        assert_eq!(
+            s.tags().unwrap(),
+            [TagCount {
+                name: "Öfke".into(),
+                charts: 2
+            }]
+        );
+        // Sınır NFC'den sonra sayılır: 64 ayrışık "Ö" 128 kod noktasıdır ama 64 karakterdir.
+        let long = "O\u{308}".repeat(MAX_TAG_CHARS);
+        assert_eq!(
+            normalize_tags(&[long]).unwrap()[0].chars().count(),
+            MAX_TAG_CHARS
+        );
+    }
 }
```

`crates/store/src/search.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/crates/store/src/search.rs
+++ b/crates/store/src/search.rs
@@ -125,6 +125,27 @@ impl Store {
         }
         Ok(hits)
     }
+
+    /// Kenar çubuğu etiket süzgeci: etiketin (`tag_key` ile tam eşleşme) bağlı olduğu en az
+    /// bir haritası olan danışanlar, son açılan önce. Bilinmeyen etiket boş liste döner.
+    pub fn clients_with_tag(&self, tag: &str, limit: u32) -> Result<Vec<Client>, StoreError> {
+        let mut stmt = self.conn.prepare_cached(
+            "SELECT DISTINCT c.id, c.uuid, c.name, c.gender, c.last_chart_id, c.last_opened_at,
+                    c.created_at, c.updated_at
+             FROM tags t
+             JOIN chart_tags ct ON ct.tag_id = t.id
+             JOIN charts ch ON ch.id = ct.chart_id
+             JOIN clients c ON c.id = ch.client_id
+             WHERE t.name_key = ?1
+             ORDER BY c.last_opened_at DESC NULLS LAST, c.name COLLATE NOCASE, c.id
+             LIMIT ?2",
+        )?;
+        let rows = stmt.query_map(
+            params![tag_key(tag.trim()), limit.min(MAX_LIST)],
+            row_to_client,
+        )?;
+        Ok(rows.collect::<Result<Vec<_>, _>>()?)
+    }
 }
 
 #[cfg(test)]
@@ -296,4 +317,30 @@ mod tests {
         assert_eq!(s.search_clients("a", 1).unwrap().len(), 1);
         assert_eq!(s.search_clients("a", u32::MAX).unwrap().len(), 2);
     }
+
+    #[test]
+    fn clients_with_tag_match_the_whole_tag_key() {
+        let mut s = store();
+        let (a, ach) = s.create_client(&new_client("Ayşe")).unwrap();
+        let (b, bch) = s.create_client(&new_client("Burak")).unwrap();
+        let (c, cch) = s.create_client(&new_client("Can")).unwrap();
+        s.set_chart_tags(ach.id, &["Kariyer".into()]).unwrap();
+        s.set_chart_tags(bch.id, &["KARİYER".into(), "araştırma".into()])
+            .unwrap();
+        s.set_chart_tags(cch.id, &["Kariyerler".into()]).unwrap();
+        s.open_client(b.id, None).unwrap();
+        let ids = |s: &Store, q: &str| -> Vec<u32> {
+            s.clients_with_tag(q, 10)
+                .unwrap()
+                .iter()
+                .map(|c| c.id)
+                .collect()
+        };
+        // Son açılan önce; önek eşleşmesi yok ("Kariyerler" ayrı etiket).
+        assert_eq!(ids(&s, " kariyer "), [b.id, a.id]);
+        assert_eq!(ids(&s, "kariyerler"), [c.id]);
+        assert!(ids(&s, "kari").is_empty());
+        assert!(ids(&s, "").is_empty());
+        assert_eq!(s.clients_with_tag("kariyer", 1).unwrap().len(), 1);
+    }
 }
```

- [ ] **Step 2: Bağımlılık denetimi**

```bash
cargo tree -p store -i icu_normalizer
cargo deny check
git diff --stat Cargo.lock
```
Expected: `icu_normalizer v2.4.0` → `store v0.1.0` (tek satır bağımlı); `advisories ok, bans ok, licenses ok, sources ok`; `Cargo.lock` farkı tek satır (`store`'un bağımlılık listesine `icu_normalizer`).

- [ ] **Step 3: Testlerin geçtiğini gör**

```bash
cargo test -p store --lib
cargo clippy -p store --all-targets -- -D warnings
cargo clippy -p store --features specta --all-targets -- -D warnings
cargo fmt --all --check
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store --all-targets
```
Expected: `store` lib `test result: ok. 70 passed` (+2); clippy ikisi de çıktısız; fmt çıktısız; MSRV `Finished`.

- [ ] **Step 4: Commit**

```bash
git add Cargo.lock \
  crates/store/Cargo.toml \
  crates/store/src/notes.rs \
  crates/store/src/search.rs
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git diff --cached --name-only | grep -iE 'k(a|ā|aa)la|\bcala\b|^\.superpowers/'   # boş olmalı
git commit -m "feat(store): NFC tag identity and an exact tag filter for the client list

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: commit oluşur; iki tarama da çıktısız. Commit mesajında yasaklı sözcük yoktur (C13).

---

### Task 3: Uygulama: `task_failed`, etiket süzgeci komutu ve danışan veritabanı sarmalayıcıları

Plan 08 ertelenen madde 11'in M6 bölümü: `spawn_blocking` görevi panik ederse `JoinError` artık `engine_stopped` değil `task_failed` olur ve parametresi boştur (panik iletisi veri taşıyabilir, C1). `list_clients_by_tag(tag, limit)` komutu `clients_with_tag`'i açar. `api.ts` kenar çubuğu, palet ve notların kullanacağı store komutlarını sarar; içe/dışa aktarma komutları hâlâ sarılmaz (C2, Task 10'a kadar).

**Files:**
- Modify: `app/src-tauri/src/client_commands.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/src/store_commands.rs`
- Test: `app/ui/api.test.ts`
- Modify: `app/ui/api.ts`
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Test: `app/ui/i18n/i18n.test.ts`
- Modify: `app/ui/i18n/tr.ts`
- Modify: `app/ui/bindings.ts` (üretilir: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings`)

**Interfaces:**
- Consumes: `Store::clients_with_tag` (Task 2), `client_commands::blocking`, `with_store`.
- Produces: IPC `list_clients_by_tag(tag: String, limit: u32) -> Vec<Client>`; hata kodu `task_failed` (tr/en/fr metni, `BACKEND_CODES`'ta). `api` yeni sarmalayıcıları: `openClient(clientId, chartId: number | null)`, `addChart(clientId, chart)`, `deleteClient(clientId)`, `deleteChart(chartId)`, `listClients(limit)`, `listClientsByTag(tag, limit)`, `listCharts(clientId)`, `searchClients(query, limit)`, `listTags()`, `setChartTags(chartId, tags)`, `listNotes(clientId, limit)`, `addNote(clientId, chartId | null, body)`, `updateNote(noteId, body)`, `deleteNote(noteId)`; hepsi `Promise<Result<T>>`.

- [ ] **Step 1: Önce testleri yaz**

`app/ui/api.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -5,8 +5,22 @@ vi.mock("./bindings", () => ({
     engineStatus: vi.fn(),
     printWindow: vi.fn(),
     restoreSession: vi.fn(),
+    openClient: vi.fn(),
     createClient: vi.fn(),
+    addChart: vi.fn(),
     updateChart: vi.fn(),
+    deleteClient: vi.fn(),
+    deleteChart: vi.fn(),
+    listClients: vi.fn(),
+    listClientsByTag: vi.fn(),
+    listCharts: vi.fn(),
+    searchClients: vi.fn(),
+    listTags: vi.fn(),
+    setChartTags: vi.fn(),
+    listNotes: vi.fn(),
+    addNote: vi.fn(),
+    updateNote: vi.fn(),
+    deleteNote: vi.fn(),
     saveViewState: vi.fn(),
     loadPanelLayout: vi.fn(),
     savePanelLayout: vi.fn(),
@@ -55,8 +69,22 @@ test("catches thrown errors into an ipc error", async () => {
 test("every wrapper forwards its arguments to the command of the same name", async () => {
   const calls: [keyof typeof api, unknown[]][] = [
     ["restoreSession", []],
+    ["openClient", [2, null]],
     ["createClient", [{ name: "A" }]],
+    ["addChart", [2, { label: "Olay" }]],
     ["updateChart", [3, { label: "" }]],
+    ["deleteClient", [2]],
+    ["deleteChart", [3]],
+    ["listClients", [100]],
+    ["listClientsByTag", ["kariyer", 100]],
+    ["listCharts", [2]],
+    ["searchClients", ["ayşe", 8]],
+    ["listTags", []],
+    ["setChartTags", [3, ["kariyer"]]],
+    ["listNotes", [2, 100]],
+    ["addNote", [2, 3, ""]],
+    ["updateNote", [5, "metin"]],
+    ["deleteNote", [5]],
     ["saveViewState", [3, { schema_version: 1 }]],
     ["loadPanelLayout", []],
     ["savePanelLayout", [null]],
```

`app/ui/i18n/i18n.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/i18n.test.ts
+++ b/app/ui/i18n/i18n.test.ts
@@ -69,7 +69,7 @@ test("chart accessible-name words exist in every language", () => {
 // Arka ucun döndürebildiği her `CommandError.code` (commands.rs, client_commands.rs,
 // atlas_commands.rs). Dosya ve içe aktarma kodları M6'da arayüze açılınca eklenir.
 const BACKEND_CODES = [
-  "ipc", "calc", "engine_stopped", "data_files_missing", "invalid_path", "env_override", "already_open", "job_panicked", "print_failed",
+  "ipc", "calc", "engine_stopped", "task_failed", "data_files_missing", "invalid_path", "env_override", "already_open", "job_panicked", "print_failed",
   "store_sqlite", "store_migration", "store_json", "store_too_new", "store_unavailable", "not_found", "invalid_input",
   "unknown_time_zone", "nonexistent_local_time", "ambiguous_local_time", "offset_out_of_range", "longitude_out_of_range",
   "date_out_of_range", "last_chart", "unsupported_schema_version", "invalid_settings", "invalid_location", "invalid_julian_day",
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/api.test.ts ui/i18n
cargo test -p jyotish-desk --lib a_panicking_blocking_task
```
Expected: vitest `every wrapper forwards its arguments…` FAIL (`api[name] is not a function`), `every backend error code…` FAIL (`task_failed`); Rust testi `left: "engine_stopped"` ile FAIL (test Step 3 farkında `client_commands.rs` içindedir; önce yalnızca test hunk'ını uygula).

- [ ] **Step 3: Uygulamayı yaz**

`app/src-tauri/src/client_commands.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/src-tauri/src/client_commands.rs
+++ b/app/src-tauri/src/client_commands.rs
@@ -200,13 +200,14 @@ pub fn save_calc_settings_blocking(
     compute_chart_blocking(engine, rec.resolved.jd_ut, place.lat, place.lon, settings)
 }
 
-/// Bloklayan gövdeyi havuzda çalıştırır; `JoinError` `engine_stopped` olur.
+/// Bloklayan gövdeyi havuzda çalıştırır. Görev paniklerse (`JoinError`) `task_failed` döner
+/// (Plan 08 ertelenen madde 11); panik iletisi parametreye konmaz, kişisel veri taşıyabilir.
 pub(crate) async fn blocking<T: Send + 'static>(
     f: impl FnOnce() -> Result<T, CommandError> + Send + 'static,
 ) -> Result<T, CommandError> {
     tauri::async_runtime::spawn_blocking(f)
         .await
-        .map_err(|e| CommandError::new("engine_stopped", &[("detail", e.to_string())]))?
+        .map_err(|_| CommandError::new("task_failed", &[]))?
 }
 
 /// Yalnızca store işi yapan komutlar için: kilit, çağrı, hata eşleme.
@@ -302,6 +303,13 @@ mod tests {
     use super::*;
     use atlas::Date;
 
+    #[test]
+    fn a_panicking_blocking_task_is_task_failed_without_its_message() {
+        let e = tauri::async_runtime::block_on(blocking::<()>(|| panic!("Ayşe"))).unwrap_err();
+        assert_eq!(e.code, "task_failed");
+        assert!(e.params.is_empty());
+    }
+
     #[test]
     fn store_error_codes_are_stable() {
         let e = CommandError::from(StoreError::NotFound {
```

`app/src-tauri/src/lib.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -35,6 +35,7 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             store_commands::delete_client,
             store_commands::delete_chart,
             store_commands::list_clients,
+            store_commands::list_clients_by_tag,
             store_commands::list_charts,
             store_commands::search_clients,
             store_commands::add_note,
```

`app/src-tauri/src/store_commands.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/src-tauri/src/store_commands.rs
+++ b/app/src-tauri/src/store_commands.rs
@@ -126,6 +126,17 @@ pub async fn list_clients(
     with_store(&store, move |s| s.list_clients(limit)).await
 }
 
+/// Kenar çubuğu etiket süzgeci: etiketin tam adıyla (büyük/küçük harf ve NFC duyarsız).
+#[tauri::command]
+#[specta::specta]
+pub async fn list_clients_by_tag(
+    store: State<'_, StoreState>,
+    tag: String,
+    limit: u32,
+) -> Result<Vec<Client>, CommandError> {
+    with_store(&store, move |s| s.clients_with_tag(&tag, limit)).await
+}
+
 #[tauri::command]
 #[specta::specta]
 pub async fn list_charts(
```

`app/ui/api.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -4,7 +4,7 @@
 //
 // Arayüz yalnızca burada listelenen komutları çağırır. İçe ve dışa aktarma komutları
 // (`importJson`, `importJhd`, `exportJson`, `exportCsv`) Plan 08 ertelenen maddeleri 1 ve 3
-// kapanana kadar (M6) bilerek yoktur; `api.test.ts` bunu denetler.
+// kapanana kadar bilerek yoktur; `api.test.ts` bunu denetler.
 // Hata parametreleri (`CommandError.params`) kişisel veri içerebilir: hiçbir yerde loglanmaz.
 import {
   commands,
@@ -42,8 +42,22 @@ export const api = {
   engineStatus: () => wrap(() => commands.engineStatus()),
   printWindow: () => wrap(() => commands.printWindow()),
   restoreSession: () => wrap(() => commands.restoreSession()),
+  openClient: (clientId: number, chartId: number | null) => wrap(() => commands.openClient(clientId, chartId)),
   createClient: (input: NewClient) => wrap(() => commands.createClient(input)),
+  addChart: (clientId: number, chart: NewChart) => wrap(() => commands.addChart(clientId, chart)),
   updateChart: (chartId: number, chart: NewChart) => wrap(() => commands.updateChart(chartId, chart)),
+  deleteClient: (clientId: number) => wrap(() => commands.deleteClient(clientId)),
+  deleteChart: (chartId: number) => wrap(() => commands.deleteChart(chartId)),
+  listClients: (limit: number) => wrap(() => commands.listClients(limit)),
+  listClientsByTag: (tag: string, limit: number) => wrap(() => commands.listClientsByTag(tag, limit)),
+  listCharts: (clientId: number) => wrap(() => commands.listCharts(clientId)),
+  searchClients: (query: string, limit: number) => wrap(() => commands.searchClients(query, limit)),
+  listTags: () => wrap(() => commands.listTags()),
+  setChartTags: (chartId: number, tags: string[]) => wrap(() => commands.setChartTags(chartId, tags)),
+  listNotes: (clientId: number, limit: number) => wrap(() => commands.listNotes(clientId, limit)),
+  addNote: (clientId: number, chartId: number | null, body: string) => wrap(() => commands.addNote(clientId, chartId, body)),
+  updateNote: (noteId: number, body: string) => wrap(() => commands.updateNote(noteId, body)),
+  deleteNote: (noteId: number) => wrap(() => commands.deleteNote(noteId)),
   saveViewState: (chartId: number, viewState: ViewState) => wrap(() => commands.saveViewState(chartId, viewState)),
   loadPanelLayout: () => wrap(() => commands.loadPanelLayout()),
   savePanelLayout: (layout: PanelLayout | null) => wrap(() => commands.savePanelLayout(layout)),
```

`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -84,6 +84,7 @@ export const en: Messages = {
     ipc: "Could not reach the application core.",
     calc: "The calculation could not finish ({{detail}}).",
     engine_stopped: "The calculation engine stopped. If this persists, restart the application.",
+    task_failed: "A background task stopped unexpectedly. Try the action again.",
     data_files_missing: "Ephemeris files were not found: {{path}}",
     invalid_path: "The ephemeris folder path cannot be used.",
     env_override: "The ephemeris path was changed by an environment variable; this is not supported.",
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -84,6 +84,7 @@ export const fr: Messages = {
     ipc: "Impossible de joindre le cœur de l’application.",
     calc: "Le calcul n’a pas pu aboutir ({{detail}}).",
     engine_stopped: "Le moteur de calcul s’est arrêté. Si le problème persiste, redémarrez l’application.",
+    task_failed: "Une tâche d’arrière-plan s’est arrêtée de façon inattendue. Réessayez l’action.",
     data_files_missing: "Fichiers d’éphémérides introuvables : {{path}}",
     invalid_path: "Le chemin du dossier des éphémérides est inutilisable.",
     env_override: "Le chemin des éphémérides a été modifié par une variable d’environnement ; ce n’est pas pris en charge.",
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -83,6 +83,7 @@ export const tr = {
     ipc: "Uygulama çekirdeğine ulaşılamadı.",
     calc: "Hesap tamamlanamadı ({{detail}}).",
     engine_stopped: "Hesap motoru durdu. Sorun sürerse uygulamayı yeniden başlatın.",
+    task_failed: "Arka plandaki iş beklenmedik biçimde durdu. İşlemi yeniden deneyin.",
     data_files_missing: "Ephemeris dosyaları bulunamadı: {{path}}",
     invalid_path: "Ephemeris klasörünün yolu kullanılamıyor.",
     env_override: "Ephemeris yolu bir ortam değişkeniyle değiştirilmiş; bu desteklenmiyor.",
```

- [ ] **Step 4: Bağlamaları yeniden üret (C10)**

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
cargo test -p jyotish-desk --test bindings
git diff --stat app/ui/bindings.ts
```
Expected: ikinci çalıştırma `test result: ok. 1 passed`; `bindings.ts`'e yalnızca `listClientsByTag` eklenir.

- [ ] **Step 5: Testlerin geçtiğini gör**

```bash
cargo test -p jyotish-desk
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
pnpm --dir app test
pnpm --dir app build
```
Expected: `jyotish-desk` lib `test result: ok. 14 passed` (+1); fmt, clippy çıktısız; vitest `36 passed`, `299 passed` (test sayısı değişmez, iki test genişler); `build` `✓ built`.

- [ ] **Step 6: Commit**

```bash
git add app/src-tauri/src/client_commands.rs \
  app/src-tauri/src/lib.rs \
  app/src-tauri/src/store_commands.rs \
  app/ui/api.test.ts \
  app/ui/api.ts \
  app/ui/bindings.ts \
  app/ui/i18n/en.ts \
  app/ui/i18n/fr.ts \
  app/ui/i18n/i18n.test.ts \
  app/ui/i18n/tr.ts
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git diff --cached --name-only | grep -iE 'k(a|ā|aa)la|\bcala\b|^\.superpowers/'   # boş olmalı
git commit -m "feat(app): task_failed for a panicked blocking task, a tag filter command and client database wrappers

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: commit oluşur; iki tarama da çıktısız. Commit mesajında yasaklı sözcük yoktur (C13).

---

### Task 4: Sıralı görünüm kayıtları (C5), okunamayan düzende kaydın tutulması (M6-3), kayıt hatasının kalkması (M6-8a)

Danışan değiştirme özelliğinden önce C5: `storePersistence` tek uçuşlu FIFO kuyruğu kullanır; bekleyen bir kayıt aynı haritaya yeni kayıt gelince onunla değiştirilir (son kazanır), başka haritanın kaydı sırasını korur. `loadLayout()` `{layout, error}` döner; okuma hatasında `Boot.layoutError` dolar, bildirim bandında `notice.layoutLoadFailed` görünür ve oturum boyunca **bütün** görünüm kayıtları tutulur (`hold`): kayıtlı düzen varsayılanla ezilmez (M6-3). Başarılı kayıt `onSaved` ile `save` bildirimini kaldırır (M6-8a). İki App testi zaman aşımını 3000 ms yapar (Plan 09 Task 12 kararsızlığı).

**Files:**
- Test: `app/ui/App.test.tsx`
- Modify: `app/ui/App.tsx`
- Modify: `app/ui/boot.ts`
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Modify: `app/ui/i18n/tr.ts`
- Test: `app/ui/view/persistence.test.ts`
- Modify: `app/ui/view/persistence.ts`

**Interfaces:**
- Consumes: `api.saveViewState`, `api.savePanelLayout`, `api.loadPanelLayout`, `useNotices().push/dismiss`.
- Produces: `storePersistence(events?: PersistenceEvents, options?: { hold?: boolean }): ViewPersistence & { idle(): Promise<void> }`; `PersistenceEvents { onError?(e: CommandError): void; onSaved?(): void }`; `loadLayout(storage?): Promise<LoadedLayout>` ve `LoadedLayout { layout: LayoutState | null; error: CommandError | null }`; `Boot.layoutError: CommandError | null`; `EMPTY_BOOT.layoutError = null`; i18n `notice.layoutLoadFailed`.

- [ ] **Step 1: Önce testleri yaz**

`app/ui/App.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -1,4 +1,4 @@
-import { fireEvent, render, screen, within } from "@testing-library/react";
+import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
 import { vi } from "vitest";
 
 vi.mock("./api", () => ({
@@ -8,6 +8,8 @@ vi.mock("./api", () => ({
     searchPlaces: vi.fn(async () => ({ status: "ok", data: [] })),
     createClient: vi.fn(),
     updateChart: vi.fn(),
+    saveViewState: vi.fn(async () => ({ status: "ok", data: null })),
+    savePanelLayout: vi.fn(async () => ({ status: "ok", data: null })),
   },
 }));
 
@@ -18,11 +20,14 @@ import { fixture, unknownTimeOpened } from "./test/fixture";
 import { THEME_STORAGE_KEY } from "./theme/theme";
 import { memoryPersistence } from "./view/persistence";
 
-function renderApp(boot: Partial<Boot> = {}) {
-  const full: Boot = { layout: null, opened: fixture().opened, error: null, ...boot };
-  return render(<App boot={full} persistence={memoryPersistence()} />);
+/** `store: true` gerçek store kalıcılığını (taklit `api`) kullanır; varsayılan bellek içi. */
+function renderApp(boot: Partial<Boot> = {}, { store = false } = {}) {
+  const full: Boot = { layout: null, layoutError: null, opened: fixture().opened, error: null, ...boot };
+  return render(<App boot={full} persistence={store ? undefined : memoryPersistence()} />);
 }
 
+const toggleNotes = () => fireEvent.keyDown(window, { key: "N", metaKey: true, shiftKey: true });
+
 test("opens on the interview layout with the restored chart", () => {
   renderApp();
   expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Görüşme");
@@ -91,6 +96,26 @@ test("a restore failure and store warnings are shown in the notice bar and can b
   ]);
 });
 
+test("a layout read failure is announced and view changes are not saved over the stored layout", async () => {
+  vi.mocked(api.saveViewState).mockClear();
+  renderApp({ layoutError: { code: "store_sqlite", params: { detail: "disk" } } }, { store: true });
+  expect(within(screen.getByRole("region", { name: "Bildirimler" })).getByRole("alert")).toHaveTextContent(
+    "Kayıtlı düzen okunamadı: Danışan veritabanı okunamadı ya da yazılamadı (disk).",
+  );
+  toggleNotes();
+  await new Promise((r) => setTimeout(r, 600));
+  expect(api.saveViewState).not.toHaveBeenCalled();
+});
+
+test("a save error notice goes away after the next successful save", async () => {
+  vi.mocked(api.saveViewState).mockResolvedValueOnce({ status: "error", error: { code: "store_sqlite", params: { detail: "disk" } } });
+  renderApp({}, { store: true });
+  toggleNotes();
+  expect(await screen.findByRole("alert", {}, { timeout: 3000 })).toHaveTextContent("Görünüm kaydedilemedi");
+  toggleNotes();
+  await waitFor(() => expect(screen.queryByRole("alert")).toBeNull(), { timeout: 3000 });
+});
+
 test("Cmd+2 switches to the research layout with the graha table", () => {
   renderApp();
   fireEvent.keyDown(window, { key: "2", code: "Digit2", metaKey: true });
```

`app/ui/view/persistence.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/view/persistence.test.ts
+++ b/app/ui/view/persistence.test.ts
@@ -13,31 +13,81 @@ const vs = { schema_version: 1, panel_layout: { active_layout_id: "a", layouts:
 
 afterEach(() => vi.mocked(api.savePanelLayout).mockReset());
 
+afterEach(() => vi.mocked(api.saveViewState).mockReset());
+
+/** Elle çözülen yanıt: kaydın ne zaman bittiğini test belirler. */
+function deferred() {
+  let resolve!: (v: { status: "ok"; data: null }) => void;
+  const promise = new Promise<{ status: "ok"; data: null }>((r) => (resolve = r));
+  return { promise, resolve: () => resolve(ok(null)) };
+}
+
 test("with an open chart the whole ViewState goes to save_view_state, without one only the layout", async () => {
   vi.mocked(api.saveViewState).mockResolvedValue(ok(null));
   vi.mocked(api.savePanelLayout).mockResolvedValue(ok(null));
   const p = storePersistence();
   p.save({ chartId: 4, viewState: vs });
   p.save({ chartId: null, viewState: vs });
+  await p.idle();
   expect(api.saveViewState).toHaveBeenCalledWith(4, vs);
   expect(api.savePanelLayout).toHaveBeenCalledWith(vs.panel_layout);
 });
 
-test("save errors reach onError", async () => {
+test("saves are sent one at a time in order and a waiting save to the same target keeps only the latest", async () => {
+  const first = deferred();
+  vi.mocked(api.saveViewState).mockReturnValueOnce(first.promise).mockResolvedValue(ok(null));
+  vi.mocked(api.savePanelLayout).mockResolvedValue(ok(null));
+  const p = storePersistence();
+  const at = (path: string[]) => ({ ...vs, dasha_expanded_path: path as never });
+  p.save({ chartId: 1, viewState: at(["Sun"]) });
+  p.save({ chartId: 2, viewState: at(["Moon"]) });
+  p.save({ chartId: 2, viewState: at(["Mars"]) });
+  p.save({ chartId: null, viewState: vs });
+  // İlk kayıt bitmeden ikincisi gönderilmez.
+  expect(api.saveViewState).toHaveBeenCalledTimes(1);
+  expect(api.savePanelLayout).not.toHaveBeenCalled();
+  first.resolve();
+  await p.idle();
+  expect(vi.mocked(api.saveViewState).mock.calls).toEqual([
+    [1, at(["Sun"])],
+    [2, at(["Mars"])],
+  ]);
+  expect(api.savePanelLayout).toHaveBeenCalledTimes(1);
+  expect(vi.mocked(api.saveViewState).mock.invocationCallOrder[1]).toBeLessThan(vi.mocked(api.savePanelLayout).mock.invocationCallOrder[0]);
+});
+
+test("save errors reach onError and a later success reaches onSaved", async () => {
   const error = { code: "store_sqlite", params: { detail: "disk full" } };
-  vi.mocked(api.savePanelLayout).mockResolvedValue({ status: "error", error });
+  vi.mocked(api.savePanelLayout).mockResolvedValueOnce({ status: "error", error }).mockResolvedValueOnce(ok(null));
   const onError = vi.fn();
-  storePersistence(onError).save({ chartId: null, viewState: vs });
-  await vi.waitFor(() => expect(onError).toHaveBeenCalledWith(error));
+  const onSaved = vi.fn();
+  const p = storePersistence({ onError, onSaved });
+  p.save({ chartId: null, viewState: vs });
+  await p.idle();
+  expect(onError).toHaveBeenCalledWith(error);
+  expect(onSaved).not.toHaveBeenCalled();
+  p.save({ chartId: null, viewState: vs });
+  await p.idle();
+  expect(onSaved).toHaveBeenCalledOnce();
+});
+
+test("held persistence sends nothing (the stored layout could not be read)", async () => {
+  const p = storePersistence({}, { hold: true });
+  p.save({ chartId: 4, viewState: vs });
+  p.save({ chartId: null, viewState: vs });
+  await p.idle();
+  expect(api.saveViewState).not.toHaveBeenCalled();
+  expect(api.savePanelLayout).not.toHaveBeenCalled();
 });
 
-test("loadLayout prefers the store and falls back to defaults on error", async () => {
+test("loadLayout prefers the store and reports a read error without a layout", async () => {
   vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(vs.panel_layout));
-  expect(await loadLayout()).toEqual(vs.panel_layout);
-  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce({ status: "error", error: { code: "store_unavailable", params: {} } });
-  expect(await loadLayout()).toBeNull();
+  expect(await loadLayout()).toEqual({ layout: vs.panel_layout, error: null });
+  const error = { code: "store_unavailable", params: {} };
+  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce({ status: "error", error });
+  expect(await loadLayout()).toEqual({ layout: null, error });
   vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
-  expect(await loadLayout()).toBeNull();
+  expect(await loadLayout()).toEqual({ layout: null, error: null });
   expect(api.savePanelLayout).not.toHaveBeenCalled();
 });
 
@@ -45,7 +95,7 @@ test("a Plan 04 layout in localStorage moves to the store once", async () => {
   localStorage.setItem(LEGACY_LAYOUT_KEY, JSON.stringify({ active_layout_id: BUILTIN_LAYOUT_IDS.transit, layouts: [] }));
   vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
   vi.mocked(api.savePanelLayout).mockResolvedValueOnce(ok(null));
-  const loaded = (await loadLayout()) as { active_layout_id: string };
+  const loaded = (await loadLayout()).layout as { active_layout_id: string };
   expect(loaded.active_layout_id).toBe(BUILTIN_LAYOUT_IDS.transit);
   expect(vi.mocked(api.savePanelLayout).mock.calls[0][0]).toMatchObject({ active_layout_id: BUILTIN_LAYOUT_IDS.transit });
   expect(localStorage.getItem(LEGACY_LAYOUT_KEY)).toBeNull();
@@ -54,10 +104,10 @@ test("a Plan 04 layout in localStorage moves to the store once", async () => {
 test("a corrupt legacy entry is ignored and a failed move keeps it for the next start", async () => {
   localStorage.setItem(LEGACY_LAYOUT_KEY, "{not json");
   vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
-  expect(await loadLayout()).toBeNull();
+  expect((await loadLayout()).layout).toBeNull();
   localStorage.setItem(LEGACY_LAYOUT_KEY, JSON.stringify({ layouts: [] }));
   vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
   vi.mocked(api.savePanelLayout).mockResolvedValueOnce({ status: "error", error: { code: "store_sqlite", params: {} } });
-  expect(await loadLayout()).not.toBeNull();
+  expect((await loadLayout()).layout).not.toBeNull();
   expect(localStorage.getItem(LEGACY_LAYOUT_KEY)).not.toBeNull();
 });
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/view/persistence.test.ts ui/App.test.tsx
```
Expected: `persistence.test.ts` sıralama, `onSaved` ve `hold` testleri FAIL (`storePersistence(...).idle is not a function`; `loadLayout` dönüşü `LayoutState`); App'in iki yeni testi FAIL (`Kayıtlı düzen okunamadı` bulunamadı).

- [ ] **Step 3: Uygulamayı yaz**

`app/ui/App.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/App.tsx
+++ b/app/ui/App.tsx
@@ -9,15 +9,27 @@ import { storePersistence, type ViewPersistence } from "./view/persistence";
 import { ViewStateProvider } from "./view/store";
 
 function bootNotices(boot: Boot): Notice[] {
-  if (boot.error) return [{ id: "boot", type: "error", messageKey: "notice.restoreFailed", error: boot.error }];
-  return boot.opened ? storeWarningNotices(boot.opened.warnings) : [];
+  const layout: Notice[] = boot.layoutError ? [{ id: "layout", type: "error", messageKey: "notice.layoutLoadFailed", error: boot.layoutError }] : [];
+  if (boot.error) return [...layout, { id: "boot", type: "error", messageKey: "notice.restoreFailed", error: boot.error }];
+  return [...layout, ...(boot.opened ? storeWarningNotices(boot.opened.warnings) : [])];
 }
 
-/** Görünüm deposu ve kayıt: kayıt hatası bildirim bandına gider (aynı kimlik, tek bildirim). */
+/**
+ * Görünüm deposu ve kayıt: kayıt hatası bildirim bandına gider (aynı kimlik, tek bildirim) ve
+ * sonraki başarılı kayıtla kalkar. Kayıtlı düzen okunamadıysa kayıtlar tutulur (M6-3).
+ */
 function Stores({ boot, persistence, children }: { boot: Boot; persistence?: ViewPersistence; children: ReactNode }) {
-  const { push } = useNotices();
+  const { push, dismiss } = useNotices();
   const [store] = useState(
-    () => persistence ?? storePersistence((error) => push({ id: "save", type: "error", messageKey: "notice.saveFailed", error })),
+    () =>
+      persistence ??
+      storePersistence(
+        {
+          onError: (error) => push({ id: "save", type: "error", messageKey: "notice.saveFailed", error }),
+          onSaved: () => dismiss("save"),
+        },
+        { hold: boot.layoutError !== null },
+      ),
   );
   const [initial] = useState(() =>
     initialViewModel(
```

`app/ui/boot.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/boot.ts
+++ b/app/ui/boot.ts
@@ -7,14 +7,17 @@ import { loadLayout } from "./view/persistence";
 
 export interface Boot {
   layout: unknown | null;
+  /** Kayıtlı düzen okunamadı: hazır düzenler gösterilir, görünüm kayıtları tutulur. */
+  layoutError: CommandError | null;
   opened: OpenedClient | null;
   error: CommandError | null;
 }
 
 export async function boot(): Promise<Boot> {
-  const [layout, session] = await Promise.all([loadLayout(), api.restoreSession()]);
-  if (session.status === "error") return { layout, opened: null, error: session.error };
-  return { layout, opened: session.data, error: null };
+  const [loaded, session] = await Promise.all([loadLayout(), api.restoreSession()]);
+  const base = { layout: loaded.layout, layoutError: loaded.error };
+  if (session.status === "error") return { ...base, opened: null, error: session.error };
+  return { ...base, opened: session.data, error: null };
 }
 
-export const EMPTY_BOOT: Boot = { layout: null, opened: null, error: null };
+export const EMPTY_BOOT: Boot = { layout: null, layoutError: null, opened: null, error: null };
```

`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -172,6 +172,7 @@ export const en: Messages = {
     warning: "Warning",
     restoreFailed: "The last session could not be opened: {{message}}",
     saveFailed: "The view could not be saved: {{message}}",
+    layoutLoadFailed: "The saved layout could not be read: {{message}} View changes in this session will not be saved; the saved layout is left as it is.",
   },
   storeWarning: {
     LastChartMissing: "The last opened chart was deleted; the most recently updated chart was opened.",
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -172,6 +172,7 @@ export const fr: Messages = {
     warning: "Avertissement",
     restoreFailed: "La dernière session n’a pas pu être ouverte : {{message}}",
     saveFailed: "La vue n’a pas pu être enregistrée : {{message}}",
+    layoutLoadFailed: "La disposition enregistrée n’a pas pu être lue : {{message}} Les changements de vue de cette session ne seront pas enregistrés ; la disposition enregistrée reste intacte.",
   },
   storeWarning: {
     LastChartMissing: "La dernière carte ouverte a été supprimée ; la plus récemment modifiée a été ouverte.",
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -171,6 +171,7 @@ export const tr = {
     warning: "Uyarı",
     restoreFailed: "Son oturum açılamadı: {{message}}",
     saveFailed: "Görünüm kaydedilemedi: {{message}}",
+    layoutLoadFailed: "Kayıtlı düzen okunamadı: {{message}} Bu oturumdaki görünüm değişiklikleri kaydedilmeyecek; kayıtlı düzen olduğu gibi duruyor.",
   },
   storeWarning: {
     LastChartMissing: "Son açılan harita silinmiş; en son güncellenen harita açıldı.",
```

`app/ui/view/persistence.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/view/persistence.ts
+++ b/app/ui/view/persistence.ts
@@ -1,6 +1,8 @@
 // Görünüm kalıcılığı (spec §4 otomatik kayıt): açık harita varsa `save_view_state`
-// (düzen + haritanın dasha yolu), yoksa `save_panel_layout` (yalnızca düzen). Hatalar
-// `onError` ile bildirilir ve loglanmaz (parametreler kişisel veri içerebilir).
+// (düzen + haritanın dasha yolu), yoksa `save_panel_layout` (yalnızca düzen). Kayıtlar
+// sıralıdır: aynı anda tek istek uçar, sıradaki aynı hedefe giden kayıt yalnızca en sonuyla
+// gider (carried rule C5). Hatalar `onError` ile bildirilir ve loglanmaz (parametreler
+// kişisel veri içerebilir).
 import { api } from "../api";
 import type { CommandError, ViewState } from "../bindings";
 import { parseLayoutState } from "../layout/model";
@@ -14,14 +16,46 @@ export interface ViewPersistence {
   save(snapshot: ViewSnapshot): void;
 }
 
-export function storePersistence(onError: (error: CommandError) => void = () => {}): ViewPersistence {
+export interface PersistenceEvents {
+  onError?(error: CommandError): void;
+  /** Bir kayıt başarıyla yazıldı (ör. önceki kayıt hatası bildirimi kaldırılır). */
+  onSaved?(): void;
+}
+
+export interface StorePersistenceOptions {
+  /**
+   * Kayıtlı düzen okunamadıysa `true`: hiçbir görünüm kaydı gönderilmez. Bellekteki hazır
+   * düzenler okunamayan kaydın üzerine yazılmaz (Plan 09 incelemesi M6-3).
+   */
+  hold?: boolean;
+}
+
+export function storePersistence(events: PersistenceEvents = {}, { hold = false }: StorePersistenceOptions = {}): ViewPersistence & { idle(): Promise<void> } {
+  const queue: ViewSnapshot[] = [];
+  let running: Promise<void> | null = null;
+
+  const send = (s: ViewSnapshot) => (s.chartId === null ? api.savePanelLayout(s.viewState.panel_layout) : api.saveViewState(s.chartId, s.viewState));
+
+  const pump = async () => {
+    for (let next = queue.shift(); next; next = queue.shift()) {
+      const r = await send(next);
+      if (r.status === "error") events.onError?.(r.error);
+      else events.onSaved?.();
+    }
+    running = null;
+  };
+
   return {
-    save({ chartId, viewState }) {
-      const call = chartId === null ? api.savePanelLayout(viewState.panel_layout) : api.saveViewState(chartId, viewState);
-      void call.then((r) => {
-        if (r.status === "error") onError(r.error);
-      });
+    save(snapshot) {
+      if (hold) return;
+      // Uçmakta olan istek kuyrukta değildir; yalnızca bekleyen son kayıt birleştirilir.
+      // Farklı haritanın kaydı birleştirilmez: ikisi de düzeni yazdığı için sıra korunur.
+      const last = queue[queue.length - 1];
+      if (last && last.chartId === snapshot.chartId) queue[queue.length - 1] = snapshot;
+      else queue.push(snapshot);
+      running ??= pump();
     },
+    idle: () => running ?? Promise.resolve(),
   };
 }
 
@@ -43,16 +77,20 @@ function readLegacy(storage: Storage): unknown | null {
   }
 }
 
-/**
- * Kayıtlı düzen (ham; `initialViewModel` toleranslı okur) ya da `null`. Hata açılışı
- * engellemez: hazır düzenler kullanılır (spec §7).
- */
-export async function loadLayout(storage: Storage = window.localStorage): Promise<unknown | null> {
+export interface LoadedLayout {
+  /** Kayıtlı düzen (ham; `initialViewModel` toleranslı okur) ya da `null`. */
+  layout: unknown | null;
+  /** `load_panel_layout` hatası: hazır düzenler kullanılır, kayıtlar tutulur. */
+  error: CommandError | null;
+}
+
+/** Hata açılışı engellemez: hazır düzenler kullanılır (spec §7). */
+export async function loadLayout(storage: Storage = window.localStorage): Promise<LoadedLayout> {
   const stored = await api.loadPanelLayout();
-  if (stored.status === "error") return null;
-  if (stored.data !== null) return stored.data;
+  if (stored.status === "error") return { layout: null, error: stored.error };
+  if (stored.data !== null) return { layout: stored.data, error: null };
   const legacy = readLegacy(storage);
-  if (legacy === null) return null;
+  if (legacy === null) return { layout: null, error: null };
   const parsed = parseLayoutState(legacy);
   const moved = await api.savePanelLayout({ active_layout_id: parsed.active_layout_id, layouts: parsed.layouts });
   if (moved.status === "ok") {
@@ -62,5 +100,5 @@ export async function loadLayout(storage: Storage = window.localStorage): Promis
       // Silinemezse bir sonraki açılışta store'daki düzen önce gelir; zararı yok.
     }
   }
-  return parsed;
+  return { layout: parsed, error: null };
 }
```

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app test
pnpm --dir app build
```
Expected: `Test Files  36 passed (36)`, `Tests  303 passed (303)`; `build` `✓ built`. Tam takımı 3 kez çalıştır: kararsızlık yok.

- [ ] **Step 5: Commit**

```bash
git add app/ui/App.test.tsx \
  app/ui/App.tsx \
  app/ui/boot.ts \
  app/ui/i18n/en.ts \
  app/ui/i18n/fr.ts \
  app/ui/i18n/tr.ts \
  app/ui/view/persistence.test.ts \
  app/ui/view/persistence.ts
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git diff --cached --name-only | grep -iE 'k(a|ā|aa)la|\bcala\b|^\.superpowers/'   # boş olmalı
git commit -m "feat(ui): ordered view saves, hold saves after a failed layout read, clear the save error on success

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: commit oluşur; iki tarama da çıktısız. Commit mesajında yasaklı sözcük yoktur (C13).

---

### Task 5: Kısayollar: `Cmd+K`, `Cmd+\`; ⌘ kısayolları yazı alanından da, `?` seçim kutularından çalışır (M6-2)

M6-2: `isEditableTarget` her `input` ve `select`'i yazı alanı sayıyordu; varga, stil, düzen, tema ya da dil seçildikten sonra `⌘1…9`, `⌘N`, `⌘E` ve `?` çalışmıyordu. `isTextEntry` yalnızca gerçek metin girişini sayar: `textarea`, metin türündeki `input` (onay kutusu, radyo, düğme, aralık, renk ve dosya değil) ve `contenteditable`; `select` metin girişi değildir. ⌘ kısayolları metin girişinin içinden de çalışır (not ya da arama kutusundayken `⌘K`, `⌘\`, `⌘⇧N`, `⌘1…9`); orada yalnızca değiştiricisiz `?` yok sayılır, çünkü yazılan bir karakterdir (C16, Fable MF1). Tuş tekrarı, IME birleştirmesi ve açık modal pencere her kısayolu durdurmaya devam eder. C7 gereği iki yeni komut yalnızca `shortcuts.ts`'e eklenir: `palette` (`⌘K`) ve `sidebar` (`⌘\`, `e.code === "Backslash"` ya da `e.key === "\\"`); yardım listesi ikisini başta gösterir.

**Files:**
- Test: `app/ui/App.test.tsx`
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Modify: `app/ui/i18n/tr.ts`
- Test: `app/ui/keyboard.test.ts`
- Modify: `app/ui/keyboard.ts`
- Test: `app/ui/shortcuts.test.ts`
- Modify: `app/ui/shortcuts.ts`
- Test: `app/ui/view/store.test.tsx`

**Interfaces:**
- Consumes: `matchShortcut`, `SHORTCUT_LIST`, `modalOpen` (Plan 09).
- Produces: `keyboard.isTextEntry(target: EventTarget | null): boolean` (`isEditableTarget`'ın yerine; `matchShortcut` onu yalnızca `?` için sorar); `Command` birleşimine `{ id: "palette" }` ve `{ id: "sidebar" }`; `ShortcutInfo.id`'ye `"palette" | "sidebar"`; i18n `shortcuts.palette`, `shortcuts.sidebar`. Task 7 `sidebar`, Task 8 `palette` komutunu Shell'de bağlar.

- [ ] **Step 1: Önce testleri yaz**

`app/ui/App.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -135,7 +135,7 @@ test("Cmd+Shift+N toggles the notes panel in the active layout", () => {
   expect(notes).toHaveAttribute("data-w", "4");
 });
 
-test("Cmd+Shift+N is ignored from editable fields, on key repeat, during composition and in print view", () => {
+test("Cmd+Shift+N works from a text field and is ignored on key repeat, during composition and in print view", () => {
   renderApp();
   const notes = () => screen.queryByRole("region", { name: "Notlar" });
   fireEvent.click(screen.getByRole("button", { name: "Düzen işlemleri" }));
@@ -143,8 +143,8 @@ test("Cmd+Shift+N is ignored from editable fields, on key repeat, during composi
   const input = screen.getByRole("textbox", { name: "Düzen adı" });
   input.focus();
   fireEvent.keyDown(input, { key: "N", metaKey: true, shiftKey: true });
-  expect(notes()).toBeInTheDocument();
-  fireEvent.keyDown(screen.getByRole("combobox", { name: "Tema" }), { key: "N", metaKey: true, shiftKey: true });
+  expect(notes()).toBeNull();
+  fireEvent.keyDown(input, { key: "N", metaKey: true, shiftKey: true });
   expect(notes()).toBeInTheDocument();
   fireEvent.keyDown(window, { key: "N", metaKey: true, shiftKey: true, repeat: true });
   expect(notes()).toBeInTheDocument();
@@ -159,6 +159,14 @@ test("Cmd+Shift+N is ignored from editable fields, on key repeat, during composi
   expect(notes()).toBeInTheDocument();
 });
 
+test("Cmd+Shift+N still works after a choice in a select (M6-2)", () => {
+  renderApp();
+  const theme = screen.getByRole("combobox", { name: "Tema" });
+  theme.focus();
+  fireEvent.keyDown(theme, { key: "N", metaKey: true, shiftKey: true });
+  expect(screen.queryByRole("region", { name: "Notlar" })).toBeNull();
+});
+
 test("theme menu sets data-theme and remembers the choice", () => {
   renderApp();
   const select = screen.getByRole("combobox", { name: "Tema" });
```

`app/ui/keyboard.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/keyboard.test.ts
+++ b/app/ui/keyboard.test.ts
@@ -1,25 +1,26 @@
-import { isEditableTarget } from "./keyboard";
+import { isTextEntry } from "./keyboard";
 
-test("inputs, textareas, selects and contenteditable are editable targets", () => {
-  const input = document.createElement("input");
+const input = (type: string) => Object.assign(document.createElement("input"), { type });
+
+test("text inputs, textareas and contenteditable are text entry", () => {
   const textarea = document.createElement("textarea");
-  const select = document.createElement("select");
   const editable = document.createElement("div");
   editable.setAttribute("contenteditable", "true"); // jsdom contentEditable özelliğini uygulamaz
   const inner = document.createElement("span");
   editable.appendChild(inner);
   document.body.append(editable);
-  expect(isEditableTarget(input)).toBe(true);
-  expect(isEditableTarget(textarea)).toBe(true);
-  expect(isEditableTarget(select)).toBe(true);
-  expect(isEditableTarget(editable)).toBe(true);
-  expect(isEditableTarget(inner)).toBe(true);
+  for (const type of ["text", "search", "number", "date", "time", "email"]) expect(isTextEntry(input(type)), type).toBe(true);
+  expect(isTextEntry(textarea)).toBe(true);
+  expect(isTextEntry(editable)).toBe(true);
+  expect(isTextEntry(inner)).toBe(true);
   editable.remove();
 });
 
-test("buttons, plain elements, window and null are not editable", () => {
-  expect(isEditableTarget(document.createElement("button"))).toBe(false);
-  expect(isEditableTarget(document.createElement("div"))).toBe(false);
-  expect(isEditableTarget(window)).toBe(false);
-  expect(isEditableTarget(null)).toBe(false);
+test("selects, checkboxes, radios, buttons, plain elements, window and null are not text entry", () => {
+  expect(isTextEntry(document.createElement("select"))).toBe(false);
+  for (const type of ["checkbox", "radio", "button", "submit", "range"]) expect(isTextEntry(input(type)), type).toBe(false);
+  expect(isTextEntry(document.createElement("button"))).toBe(false);
+  expect(isTextEntry(document.createElement("div"))).toBe(false);
+  expect(isTextEntry(window)).toBe(false);
+  expect(isTextEntry(null)).toBe(false);
 });
```

`app/ui/shortcuts.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shortcuts.test.ts
+++ b/app/ui/shortcuts.test.ts
@@ -13,6 +13,9 @@ test("global commands", () => {
   expect(key({ key: "N", metaKey: true, shiftKey: true })).toEqual({ id: "toggleNotes" });
   expect(key({ key: "n", metaKey: true })).toEqual({ id: "newChart" });
   expect(key({ key: "e", metaKey: true })).toEqual({ id: "editChart" });
+  expect(key({ key: "k", metaKey: true })).toEqual({ id: "palette" });
+  expect(key({ key: "\\", code: "Backslash", metaKey: true })).toEqual({ id: "sidebar" });
+  expect(key({ key: ",", code: "Backslash", metaKey: true })).toEqual({ id: "sidebar" });
   expect(key({ key: "?", shiftKey: true })).toEqual({ id: "help" });
 });
 
@@ -31,6 +34,22 @@ test("ignored: Ctrl/Alt, digit 0, repeat, composition, editable fields and open
   expect(key({ key: "n", metaKey: true })).toEqual({ id: "newChart" });
 });
 
+test("Cmd shortcuts and ? work from a select, checkbox or radio, and Cmd shortcuts also work from text entry, where only ? is ignored", () => {
+  const select = document.createElement("select");
+  const checkbox = Object.assign(document.createElement("input"), { type: "checkbox" });
+  const radio = Object.assign(document.createElement("input"), { type: "radio" });
+  for (const target of [select, checkbox, radio]) {
+    expect(key({ key: "2", code: "Digit2", metaKey: true }, target)).toEqual({ id: "selectLayout", index: 1 });
+    expect(key({ key: "k", metaKey: true }, target)).toEqual({ id: "palette" });
+    expect(key({ key: "?", shiftKey: true }, target)).toEqual({ id: "help" });
+  }
+  for (const target of [document.createElement("input"), document.createElement("textarea")]) {
+    expect(key({ key: "k", metaKey: true }, target)).toEqual({ id: "palette" });
+    expect(key({ key: "N", metaKey: true, shiftKey: true }, target)).toEqual({ id: "toggleNotes" });
+    expect(key({ key: "?", shiftKey: true }, target)).toBeNull();
+  }
+});
+
 test("every listed shortcut has a description", () => {
   for (const s of SHORTCUT_LIST) expect(Object.keys(tr.shortcuts)).toContain(s.id);
   expect(Object.keys(tr.shortcuts).filter((k) => k !== "title" && k !== "open" && k !== "close" && k !== "keys" && k !== "action")).toEqual(
```

`app/ui/view/store.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/view/store.test.tsx
+++ b/app/ui/view/store.test.tsx
@@ -207,7 +207,7 @@ test("Cmd+1…9 selects layouts by order", () => {
   expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.interview);
 });
 
-test("Cmd+digit uses the physical key (AZERTY) and ignores out-of-range, Ctrl, Alt, 0, repeat, IME and inputs", () => {
+test("Cmd+digit uses the physical key (AZERTY), works from a text field and ignores out-of-range, Ctrl, Alt, 0, repeat and IME", () => {
   setup();
   fireEvent.keyDown(window, { key: "&", code: "Digit3", metaKey: true });
   expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
@@ -223,8 +223,9 @@ test("Cmd+digit uses the physical key (AZERTY) and ignores out-of-range, Ctrl, A
     fireEvent.keyDown(window, init);
     expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.research);
   }
+  // ⌘ kısayolları yazı alanında da çalışır (Plan 10 incelemesi MF1).
   fireEvent.keyDown(screen.getByLabelText("field"), { key: "3", code: "Digit3", metaKey: true });
-  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.research);
+  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
 });
 
 test("Cmd+9 with 3 layouts keeps the selection and does not preventDefault", () => {
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/keyboard.test.ts ui/shortcuts.test.ts ui/App.test.tsx ui/view/store.test.tsx
```
Expected: 7 FAIL: `keyboard.test.ts` iki test (`isTextEntry is not a function`); `shortcuts.test.ts` `global commands` ve `Cmd shortcuts and ? work from a select…` (`⌘K`/`⌘\` ve yazı alanındaki ⌘ eşleşmeleri `null`); App `Cmd+Shift+N works from a text field…` ve `Cmd+Shift+N still works after a choice in a select (M6-2)`; `view/store.test.tsx` `Cmd+digit … works from a text field…` (yazı alanındaki `⌘2` artık düzeni değiştirmelidir).

- [ ] **Step 3: Uygulamayı yaz**

`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -204,6 +204,8 @@ export const en: Messages = {
     close: "Close",
     keys: "Keys",
     action: "Action",
+    palette: "Search clients, layouts and commands",
+    sidebar: "Show or hide the client sidebar",
     newChart: "New chart",
     editChart: "Edit the open chart",
     selectLayout: "Switch to the layout at that position",
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -204,6 +204,8 @@ export const fr: Messages = {
     close: "Fermer",
     keys: "Touches",
     action: "Action",
+    palette: "Rechercher clients, dispositions et commandes",
+    sidebar: "Afficher ou masquer la barre des clients",
     newChart: "Nouvelle carte",
     editChart: "Modifier la carte ouverte",
     selectLayout: "Passer à la disposition de ce rang",
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -203,6 +203,8 @@ export const tr = {
     close: "Kapat",
     keys: "Tuşlar",
     action: "İşlem",
+    palette: "Danışan, düzen ve komut ara",
+    sidebar: "Danışan kenar çubuğunu aç/kapat",
     newChart: "Yeni harita",
     editChart: "Açık haritayı düzenle",
     selectLayout: "Sıradaki düzene geç",
```

`app/ui/keyboard.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/keyboard.ts
+++ b/app/ui/keyboard.ts
@@ -1,10 +1,14 @@
-// Genel kısayollar yazı alanlarında tetiklenmez (ör. ad girerken Cmd+Shift+N).
+// Yazı girilen alanlarda yalnızca değiştiricisiz kısayollar (`?`) yazı sayılır; ⌘ kısayolları
+// her yerde çalışır (not yazarken ⌘K, ⌘⇧N). Seçim kutusu, onay kutusu ve radyo düğmesi yazı
+// alanı değildir: oralarda `?` da çalışır (Plan 09 incelemesi M6-2, Plan 10 incelemesi MF1).
 
-/** Olay hedefi metin girilen bir alan mı: input, textarea, select ya da contenteditable. */
-export function isEditableTarget(target: EventTarget | null): boolean {
+/** Yazı girmeyen `input` türleri: bunlarda genel kısayollar çalışır. */
+const NON_TEXT_INPUTS = new Set(["checkbox", "radio", "button", "submit", "reset", "range", "color", "file", "image"]);
+
+/** Olay hedefi metin girilen bir alan mı: metin türü input, textarea ya da contenteditable. */
+export function isTextEntry(target: EventTarget | null): boolean {
   if (!(target instanceof HTMLElement)) return false;
-  if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) {
-    return true;
-  }
+  if (target instanceof HTMLTextAreaElement) return true;
+  if (target instanceof HTMLInputElement) return !NON_TEXT_INPUTS.has(target.type);
   return target.isContentEditable || target.closest("[contenteditable]:not([contenteditable='false'])") !== null;
 }
```

`app/ui/shortcuts.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shortcuts.ts
+++ b/app/ui/shortcuts.ts
@@ -1,22 +1,26 @@
 // Klavye kısayollarının tek kaynağı (spec §4 "tam klavye"). Genel komutlar burada eşlenir;
 // yardım penceresi (`?`) listeyi buradan çizer. Panel içi tuşlar (Alt+ok, ağaç gezintisi)
 // odaktaki öğede işlenir ve burada yalnızca listelenir.
-import { isEditableTarget } from "./keyboard";
+import { isTextEntry } from "./keyboard";
 
 export type Command =
   | { id: "selectLayout"; index: number }
   | { id: "toggleNotes" }
   | { id: "newChart" }
   | { id: "editChart" }
+  | { id: "palette" }
+  | { id: "sidebar" }
   | { id: "help" };
 
 export interface ShortcutInfo {
   /** i18n anahtarı: `shortcuts.<id>`. */
-  id: "selectLayout" | "toggleNotes" | "newChart" | "editChart" | "help" | "escape" | "movePanel" | "resizePanel" | "tree";
+  id: "palette" | "sidebar" | "selectLayout" | "toggleNotes" | "newChart" | "editChart" | "help" | "escape" | "movePanel" | "resizePanel" | "tree";
   keys: string;
 }
 
 export const SHORTCUT_LIST: readonly ShortcutInfo[] = [
+  { id: "palette", keys: "⌘K" },
+  { id: "sidebar", keys: "⌘\\" },
   { id: "newChart", keys: "⌘N" },
   { id: "editChart", keys: "⌘E" },
   { id: "selectLayout", keys: "⌘1 … ⌘9" },
@@ -34,18 +38,23 @@ export function modalOpen(): boolean {
 }
 
 /**
- * Klavye olayı → genel komut. Yazı alanlarında, tuş tekrarında, IME bileşiminde ve modal
- * açıkken `null`. Rakamlar fiziksel tuştan (`code`) okunur: AZERTY'de `key` rakam değildir.
+ * Klavye olayı → genel komut. Tuş tekrarında, IME bileşiminde ve modal açıkken `null`;
+ * ⌘ kısayolları yazı alanında da çalışır, değiştiricisiz `?` yazı alanında yazıdır. Rakamlar ve ters bölü fiziksel tuştan (`code`) okunur: AZERTY'de `key`
+ * rakam değildir, Türkçe Q klavyede ters bölü tek tuşla yazılmaz.
  */
 export function matchShortcut(e: KeyboardEvent): Command | null {
-  if (e.repeat || e.isComposing || isEditableTarget(e.target) || modalOpen()) return null;
+  if (e.repeat || e.isComposing || modalOpen()) return null;
   if (e.ctrlKey || e.altKey) return null;
   if (e.metaKey) {
     if (e.shiftKey) return e.key.toLowerCase() === "n" ? { id: "toggleNotes" } : null;
     if (/^Digit[1-9]$/.test(e.code)) return { id: "selectLayout", index: Number(e.code.slice(5)) - 1 };
-    if (e.key.toLowerCase() === "n") return { id: "newChart" };
-    if (e.key.toLowerCase() === "e") return { id: "editChart" };
+    if (e.code === "Backslash" || e.key === "\\") return { id: "sidebar" };
+    const k = e.key.toLowerCase();
+    if (k === "k") return { id: "palette" };
+    if (k === "n") return { id: "newChart" };
+    if (k === "e") return { id: "editChart" };
     return null;
   }
-  return e.key === "?" ? { id: "help" } : null;
+  // Değiştiricisiz tuş yazıdır: metin girişinde komut sayılmaz. ⌘ kısayolları her yerde çalışır.
+  return e.key === "?" && !isTextEntry(e.target) ? { id: "help" } : null;
 }
```

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app test
pnpm --dir app build
```
Expected: `Test Files  36 passed (36)`, `Tests  305 passed (305)`; `build` `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/App.test.tsx \
  app/ui/i18n/en.ts \
  app/ui/i18n/fr.ts \
  app/ui/i18n/tr.ts \
  app/ui/keyboard.test.ts \
  app/ui/keyboard.ts \
  app/ui/shortcuts.test.ts \
  app/ui/shortcuts.ts \
  app/ui/view/store.test.tsx
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git diff --cached --name-only | grep -iE 'k(a|ā|aa)la|\bcala\b|^\.superpowers/'   # boş olmalı
git commit -m "feat(ui): Cmd+K and Cmd+\ in the shortcut registry; Cmd shortcuts work from text entry, ? from selects, checkboxes and radios

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: commit oluşur; iki tarama da çıktısız. Commit mesajında yasaklı sözcük yoktur (C13).

---

### Task 6: Oturum: kimlikle danışan açma, kapatma, etiket güncelleme; harita ekleme formu; güncel listeden bildirim kaldırma (M6-8b); `timeUnknown` taşınır (M6-10)

Kenar çubuğu ve paletin ortak altyapısı. `Session` `openClient(clientId, chartId?)` (sıra bekçili: art arda iki istekte yalnızca sonuncusu açılır), `close()` (açık danışan silinince), `setTags()` ve `touch()` kazanır; `revision` sayacı liste görünümlerinin yeniden okuma tetiğidir. Bildirim temizliği `dismissWhere` ile güncel listeden yapılır (`await` öncesi okunan eski liste değil, M6-8b) ve `BirthForm`'dan `session.open`'a taşınır; `open` açılış (`boot`), danışan (`client`, açma ve silme hataları) ve store bildirimlerini kaldırır, böylece başarılı açılıştan sonra eski "açılamadı" bildirimi kalmaz (Fable S2). Görünüm deposunda `closeChart` eylemi: `openChart` gibi önce bekleyen kaydı boşaltır (C5) ve kirli değişiklik sayılmaz. `BirthForm`'a `"add"` kipi: açık danışana `add_chart` ile yeni harita (ad ve cinsiyet danışandan, kilitli). `timeUnknown` `session/birth.ts`'e taşınır (M6-10).

**Files:**
- Test: `app/ui/birth/BirthForm.test.tsx`
- Modify: `app/ui/birth/BirthForm.tsx`
- Modify: `app/ui/dasha/DashaTreePanel.tsx`
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Modify: `app/ui/i18n/tr.ts`
- Modify: `app/ui/panels/ChartPanel.tsx`
- Modify: `app/ui/panels/GrahaTablePanel.tsx`
- Create: `app/ui/session/birth.ts`
- Test (yeni): `app/ui/session/store.test.tsx`
- Modify: `app/ui/session/store.tsx`
- Modify: `app/ui/shell/notices.tsx`
- Modify: `app/ui/view/reducer.ts`
- Modify: `app/ui/view/store.tsx`

**Interfaces:**
- Consumes: `api.openClient`, `api.addChart` (Task 3); `useViewStore().openChart` (Plan 09).
- Produces: `Session { opened; revision: number; open(o); openClient(clientId: number, chartId?: number | null): Promise<CommandError | null>; close(); setTags(chartId: number, tags: string[]); touch() }`; `ViewAction` `{ type: "closeChart" }` ve `ViewStore.closeChart(): void`; `Notices.dismissWhere(match: (n: Notice) => boolean): void`; `session/birth.ts` `timeUnknown(opened: OpenedClient): boolean`; `BirthFormProps.mode: "new" | "add" | "edit"`; i18n `birth.titleAdd`.

- [ ] **Step 1: Önce testleri yaz**

`app/ui/birth/BirthForm.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/birth/BirthForm.test.tsx
+++ b/app/ui/birth/BirthForm.test.tsx
@@ -3,7 +3,7 @@ import { useEffect } from "react";
 import { vi } from "vitest";
 
 vi.mock("../api", () => ({
-  api: { searchPlaces: vi.fn(), previewMoment: vi.fn(), createClient: vi.fn(), updateChart: vi.fn() },
+  api: { searchPlaces: vi.fn(), previewMoment: vi.fn(), createClient: vi.fn(), addChart: vi.fn(), updateChart: vi.fn() },
 }));
 
 import { api } from "../api";
@@ -20,7 +20,7 @@ function OpenedName() {
 }
 
 const onClose = vi.fn();
-function renderForm(mode: "new" | "edit", opened = mode === "edit" ? fixture().opened : null) {
+function renderForm(mode: "new" | "add" | "edit", opened = mode === "new" ? null : fixture().opened) {
   return renderWithStores(
     <>
       <OpenedName />
@@ -145,6 +145,24 @@ test("edit mode starts from the stored chart and saves with update_chart", async
   expect(api.createClient).not.toHaveBeenCalled();
 });
 
+test("add mode keeps the open client's name locked and saves with add_chart", async () => {
+  const added = fixture().opened;
+  added.chart_record = { ...added.chart_record, id: 9, label: "Evlilik", kind: "Event" };
+  vi.mocked(api.addChart).mockResolvedValueOnce({ status: "ok", data: added });
+  renderForm("add");
+  expect(dialog()).toHaveAccessibleName("Harita ekle");
+  expect(input("Danışan adı")).toBeDisabled();
+  expect(input("Danışan adı")).toHaveValue("Örnek Danışan");
+  expect(input("Tarih")).toHaveValue("");
+  await fillDelhi();
+  fireEvent.change(input("Harita etiketi"), { target: { value: "Evlilik" } });
+  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
+  await wait(0);
+  expect(api.addChart).toHaveBeenCalledWith(fixture().opened.client.id, expect.objectContaining({ label: "Evlilik", birth: expect.objectContaining({ date: "2000-01-01" }) }));
+  expect(api.createClient).not.toHaveBeenCalled();
+  expect(onClose).toHaveBeenCalledOnce();
+});
+
 test("saving replaces boot and store notices with the warnings of the returned chart", async () => {
   const stale: Notice[] = [
     { id: "boot", type: "error", messageKey: "notice.restoreFailed", error: { code: "store_sqlite", params: {} } },
```

`app/ui/session/store.test.tsx` oluştur:

```tsx
import { act, fireEvent, screen, waitFor } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("../api", () => ({ api: { openClient: vi.fn() } }));

import { api } from "../api";
import type { OpenedClient } from "../bindings";
import { NoticeBar, useNotices } from "../shell/notices";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useViewStore } from "../view/store";
import { useSession } from "./store";

/** İkinci danışan: aynı harita verisi, başka kimlikler. */
function other(): OpenedClient {
  const o = fixture().opened;
  o.client = { ...o.client, id: 2, name: "İkinci Danışan" };
  o.chart_record = { ...o.chart_record, id: 9, client_id: 2, tags: ["kariyer"] };
  o.view_state = { ...o.view_state, dasha_expanded_path: ["Venus"] };
  o.warnings = ["LastChartMissing"];
  return o;
}

function Probe() {
  const session = useSession();
  const view = useViewStore();
  const { push } = useNotices();
  return (
    <div>
      <output aria-label="client">{session.opened?.client.name ?? "-"}</output>
      <output aria-label="chart">{String(view.model.chartId)}</output>
      <output aria-label="path">{view.model.dashaPath.join(">")}</output>
      <output aria-label="tags">{session.opened?.chart_record.tags.join(",") ?? "-"}</output>
      <output aria-label="revision">{session.revision}</output>
      <button onClick={() => view.setDashaPath(["Rahu"])}>path</button>
      <button onClick={() => void session.openClient(2)}>open2</button>
      <button onClick={() => session.close()}>close</button>
      <button onClick={() => session.setTags(1, ["araştırma"])}>tags</button>
      <button onClick={() => push({ id: "boot", type: "error", messageKey: "notice.restoreFailed", error: { code: "ipc", params: {} } })}>boot</button>
      <NoticeBar />
    </div>
  );
}

const out = (name: string) => screen.getByRole("status", { name });

beforeEach(() => vi.mocked(api.openClient).mockReset());

test("openClient flushes the previous chart, opens the reply and swaps the notices", async () => {
  vi.mocked(api.openClient).mockResolvedValue({ status: "ok", data: other() });
  const { persistence } = renderWithStores(<Probe />, { debounceMs: 10_000 });
  fireEvent.click(screen.getByText("boot"));
  fireEvent.click(screen.getByText("path"));
  fireEvent.click(screen.getByText("open2"));
  await waitFor(() => expect(out("client")).toHaveTextContent("İkinci Danışan"));
  expect(api.openClient).toHaveBeenCalledWith(2, null);
  // Bekleyen kayıt önceki haritaya (1) yazıldı; açılan harita kirli değildir.
  expect(persistence.saved.map((s) => [s.chartId, s.viewState.dasha_expanded_path])).toEqual([[1, ["Rahu"]]]);
  expect(out("chart")).toHaveTextContent("9");
  expect(out("path")).toHaveTextContent("Venus");
  expect(screen.queryByText(/Son oturum açılamadı/)).toBeNull();
  expect(screen.getByText("Son açılan harita silinmiş; en son güncellenen harita açıldı.")).toBeInTheDocument();
  expect(out("revision")).toHaveTextContent("1");
});

test("only the latest openClient reply is opened and an error leaves the chart as it is", async () => {
  let first!: (v: { status: "ok"; data: OpenedClient }) => void;
  vi.mocked(api.openClient)
    .mockReturnValueOnce(new Promise((r) => (first = r)))
    .mockResolvedValueOnce({ status: "error", error: { code: "not_found", params: { what: "client", id: "2" } } });
  renderWithStores(<Probe />);
  fireEvent.click(screen.getByText("open2"));
  fireEvent.click(screen.getByText("open2"));
  await act(async () => first({ status: "ok", data: other() }));
  expect(out("client")).toHaveTextContent("Örnek Danışan");
  expect(out("chart")).toHaveTextContent("1");
});

test("close empties the desk and saves the pending change to the closed chart", () => {
  const { persistence } = renderWithStores(<Probe />, { debounceMs: 10_000 });
  fireEvent.click(screen.getByText("path"));
  fireEvent.click(screen.getByText("close"));
  expect(out("client")).toHaveTextContent("-");
  expect(out("chart")).toHaveTextContent("null");
  expect(persistence.saved.map((s) => s.chartId)).toEqual([1]);
});

test("setTags updates the open chart's tags and bumps the revision", () => {
  renderWithStores(<Probe />);
  fireEvent.click(screen.getByText("tags"));
  expect(out("tags")).toHaveTextContent("araştırma");
  expect(out("revision")).toHaveTextContent("1");
});
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/session ui/birth/BirthForm.test.tsx
```
Expected: `session/store.test.tsx` FAIL (`session.openClient is not a function`); `BirthForm.test.tsx` add kipi testi `Harita ekle` başlığı bulunamadığı için FAIL.

- [ ] **Step 3: Uygulamayı yaz**

`app/ui/birth/BirthForm.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/birth/BirthForm.tsx
+++ b/app/ui/birth/BirthForm.tsx
@@ -1,17 +1,16 @@
 // Doğum verisi girişi (spec §4 "Yeni danışan/harita" ve "Düzenle"). Sağdan kayan modal
 // pencere. Form biçimi `model.ts`'de denetlenir; an Rust'ta çözülür: alanlar değiştikçe
 // 300 ms sonra `preview_moment` çağrılır (UTC, ofset, uyarılar, DST boşluğu/örtüşmesi).
-// Kayıt `create_client` (yeni) ya da `update_chart` (düzenle) ile yapılır; dönen harita açılır.
-// Açılış ve önceki haritanın bildirimleri (`boot`, `store:*`) kalkar; yeni haritanın store
-// uyarıları gösterilir.
+// Kayıt `create_client` (yeni danışan), `add_chart` (açık danışana yeni harita) ya da
+// `update_chart` (düzenle) ile yapılır; dönen harita `session.open` ile açılır (önceki haritanın
+// bildirimleri orada kalkar, yeni haritanın store uyarıları gösterilir).
 import { useEffect, useId, useRef, useState, type FormEvent, type ReactNode } from "react";
 import { useTranslation } from "react-i18next";
 import { api } from "../api";
-import type { CommandError, MomentPreview } from "../bindings";
+import type { CommandError, MomentPreview, OpenedClient } from "../bindings";
 import { errorMessage } from "../i18n/errors";
 import { useSession } from "../session/store";
 import { Dialog } from "../shell/Dialog";
-import { storeWarningNotices, useNotices } from "../shell/notices";
 import "./birth.css";
 import {
   applyPlace,
@@ -34,17 +33,24 @@ import { PlaceSearch } from "./PlaceSearch";
 export const PREVIEW_DEBOUNCE_MS = 300;
 
 export interface BirthFormProps {
-  mode: "new" | "edit";
+  /** `new`: yeni danışan; `add`: açık danışana yeni harita; `edit`: açık haritayı düzenle. */
+  mode: "new" | "add" | "edit";
   onClose(): void;
 }
 
+function initialDraft(mode: BirthFormProps["mode"], opened: OpenedClient | null): BirthDraft {
+  if (!opened || mode === "new") return emptyDraft();
+  if (mode === "edit") return draftFromOpened(opened);
+  return { ...emptyDraft(), name: opened.client.name, gender: opened.client.gender };
+}
+
 export function BirthForm({ mode, onClose }: BirthFormProps) {
   const { t, i18n } = useTranslation();
   const session = useSession();
   const id = useId();
-  const notices = useNotices();
   const editing = mode === "edit" && session.opened !== null;
-  const [draft, setDraft] = useState<BirthDraft>(() => (editing ? draftFromOpened(session.opened!) : emptyDraft()));
+  const adding = mode === "add" && session.opened !== null;
+  const [draft, setDraft] = useState<BirthDraft>(() => initialDraft(mode, session.opened));
   const [shown, setShown] = useState<FieldErrors>({});
   const [preview, setPreview] = useState<{ ok: MomentPreview } | { error: CommandError } | null>(null);
   const [formError, setFormError] = useState<CommandError | null>(null);
@@ -97,14 +103,15 @@ export function BirthForm({ mode, onClose }: BirthFormProps) {
       return;
     }
     setSaving(true);
+    const o = session.opened;
     const r =
-      editing && session.opened
-        ? await api.updateChart(session.opened.chart_record.id, toNewChart(draft, birth))
-        : await api.createClient(toNewClient(draft, birth));
+      editing && o
+        ? await api.updateChart(o.chart_record.id, toNewChart(draft, birth))
+        : adding && o
+          ? await api.addChart(o.client.id, toNewChart(draft, birth))
+          : await api.createClient(toNewClient(draft, birth));
     setSaving(false);
     if (r.status === "ok") {
-      for (const n of notices.notices) if (n.id === "boot" || n.id.startsWith("store:")) notices.dismiss(n.id);
-      for (const n of storeWarningNotices(r.data.warnings)) notices.push(n);
       session.open(r.data);
       onClose();
       return;
@@ -142,10 +149,10 @@ export function BirthForm({ mode, onClose }: BirthFormProps) {
   const ambiguous = previewError?.code === "ambiguous_local_time" || draft.fold !== null;
 
   return (
-    <Dialog title={editing ? t("birth.titleEdit") : t("birth.titleNew")} onClose={onClose} variant="sheet">
+    <Dialog title={editing ? t("birth.titleEdit") : adding ? t("birth.titleAdd") : t("birth.titleNew")} onClose={onClose} variant="sheet">
       <form className="birth-form" noValidate onSubmit={(e) => void submit(e)}>
         {field("name", t("birth.name"), (p) => (
-          <input {...p} value={draft.name} disabled={editing} data-autofocus onChange={(e) => update({ name: e.target.value }, "name")} onBlur={blur("name")} />
+          <input {...p} value={draft.name} disabled={editing || adding} data-autofocus onChange={(e) => update({ name: e.target.value }, "name")} onBlur={blur("name")} />
         ))}
         {field("label", t("birth.label"), (p) => (
           <input {...p} value={draft.label} onChange={(e) => update({ label: e.target.value }, "label")} />
```

`app/ui/dasha/DashaTreePanel.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/dasha/DashaTreePanel.tsx
+++ b/app/ui/dasha/DashaTreePanel.tsx
@@ -1,7 +1,7 @@
 import { useEffect, useRef, useState, type KeyboardEvent } from "react";
 import { useTranslation } from "react-i18next";
 import type { DashaPeriod, Graha } from "../bindings";
-import { timeUnknown } from "../panels/ChartPanel";
+import { timeUnknown } from "../session/birth";
 import { PanelError, usePanelLoadPhase } from "../panels/states";
 import type { PanelProps } from "../panels/types";
 import { useSession } from "../session/store";
```

`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -221,6 +221,7 @@ export const en: Messages = {
     placeResults: "Matching places",
     placeCount: "Results: {{count}}",
     titleNew: "New chart",
+    titleAdd: "Add chart",
     titleEdit: "Edit chart",
     edit: "Edit",
     name: "Client name",
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -221,6 +221,7 @@ export const fr: Messages = {
     placeResults: "Lieux trouvés",
     placeCount: "Résultats : {{count}}",
     titleNew: "Nouvelle carte",
+    titleAdd: "Ajouter une carte",
     titleEdit: "Modifier la carte",
     edit: "Modifier",
     name: "Nom du consultant",
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -220,6 +220,7 @@ export const tr = {
     placeResults: "Bulunan yerler",
     placeCount: "{{count}} sonuç",
     titleNew: "Yeni harita",
+    titleAdd: "Harita ekle",
     titleEdit: "Haritayı düzenle",
     edit: "Düzenle",
     name: "Danışan adı",
```

`app/ui/panels/ChartPanel.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/ChartPanel.tsx
+++ b/app/ui/panels/ChartPanel.tsx
@@ -1,10 +1,11 @@
 import { useId } from "react";
 import { useTranslation } from "react-i18next";
-import type { OpenedClient, Varga } from "../bindings";
+import type { Varga } from "../bindings";
 import { chartDataFromChart, isVarga, VARGA_NAMES, VARGAS } from "../chart/adapter";
 import { ChartSvg } from "../chart/ChartSvg";
 import { CHART_STYLES, isChartStyle } from "../chart/types";
 import { useHighlight } from "../highlight/store";
+import { timeUnknown } from "../session/birth";
 import { useSession } from "../session/store";
 import { useLayoutStore } from "../view/store";
 import { chartLabels } from "./labels";
@@ -15,11 +16,6 @@ export function chartVarga(settings: Record<string, unknown>): Varga {
   return isVarga(settings.varga) ? settings.varga : "D1";
 }
 
-/** Doğum saati bilinmiyor: lagna, evler ve saate bağlı değerler gösterilmez (spec §7). */
-export function timeUnknown(opened: OpenedClient): boolean {
-  return opened.chart_record.birth.time_certainty === "Unknown";
-}
-
 /**
  * Harita paneli: açık haritanın `settings.varga` vargası, `settings.style` biçiminde.
  * Varga ve biçim seçimi panel ayarına yazılır; düzenle birlikte kaydedilir.
```

`app/ui/panels/GrahaTablePanel.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/GrahaTablePanel.tsx
+++ b/app/ui/panels/GrahaTablePanel.tsx
@@ -7,7 +7,7 @@ import { useHighlight } from "../highlight/store";
 import { sameTarget, type HighlightTarget } from "../highlight/target";
 import { useSession } from "../session/store";
 import { useLayoutStore } from "../view/store";
-import { timeUnknown } from "./ChartPanel";
+import { timeUnknown } from "../session/birth";
 import { formatDegree, formatSignedDegree, formatSpeed } from "./format";
 import { GRAHA_COLUMNS, grahaColumns, toggleColumn, type GrahaColumn } from "./grahaColumns";
 import { nakshatraName } from "./labels";
```

`app/ui/session/birth.ts` oluştur:

```ts
// Açık haritanın doğum verisinden türeyen gösterim kararları (hesap değil; veri Rust'tan).

import type { OpenedClient } from "../bindings";

/** Doğum saati bilinmiyor: lagna, evler ve saate bağlı değerler gösterilmez (spec §7). */
export function timeUnknown(opened: OpenedClient): boolean {
  return opened.chart_record.birth.time_certainty === "Unknown";
}
```

`app/ui/session/store.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/session/store.tsx
+++ b/app/ui/session/store.tsx
@@ -1,13 +1,35 @@
 // Açık harita (oturum). Harita Rust'ta hesaplanmış gelir (`OpenedClient`); arayüz yalnızca
-// çizer (spec §4). Yeni harita açılınca görünüm deposu o haritanın dasha yoluna geçer.
-import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";
-import type { OpenedClient } from "../bindings";
+// çizer (spec §4). Yeni harita açılınca görünüm deposu o haritanın dasha yoluna geçer;
+// önceki haritanın bekleyen görünüm kaydı önce ona yazılır (carried rule C5).
+import { createContext, useCallback, useContext, useMemo, useRef, useState, type ReactNode } from "react";
+import { api } from "../api";
+import type { CommandError, OpenedClient } from "../bindings";
+import { storeWarningNotices, useNotices } from "../shell/notices";
 import { useViewStore } from "../view/store";
 
 export interface Session {
   opened: OpenedClient | null;
-  /** `create_client`/`update_chart` dönüşü: haritayı açar, görünümü ona bağlar. */
+  /**
+   * Kayıtlı veride liste görünümlerini (kenar çubuğu, palet) etkileyen her değişiklikte artar:
+   * açılış, silme, etiket değişikliği. Listeler bu sayaca göre yeniden okunur.
+   */
+  revision: number;
+  /**
+   * Komut dönüşünü (`create_client`, `add_chart`, `update_chart`, `open_client`) açar. Açılış, açma
+   * hatası ve önceki haritanın bildirimleri (`boot`, `client`, `store:*`) kalkar, yeni haritanın uyarıları gösterilir.
+   */
   open(opened: OpenedClient): void;
+  /**
+   * `open_client`: danışanı `chartId` verilmezse son haritasıyla açar. Hata dönerse açık harita
+   * değişmez. Art arda iki istekte yalnızca sonuncusunun yanıtı açılır.
+   */
+  openClient(clientId: number, chartId?: number | null): Promise<CommandError | null>;
+  /** Açık haritayı kapatır (ör. danışan silindi): pano boş duruma döner. */
+  close(): void;
+  /** Açık haritanın etiketleri kaydedildi (`set_chart_tags` dönüşü). */
+  setTags(chartId: number, tags: string[]): void;
+  /** Liste yenilemesi gerektiren, açık haritayı değiştirmeyen kayıt (ör. başka harita silindi). */
+  touch(): void;
 }
 
 const SessionContext = createContext<Session | null>(null);
@@ -18,15 +40,55 @@ const SessionContext = createContext<Session | null>(null);
  */
 export function SessionProvider({ initial, children }: { initial: OpenedClient | null; children: ReactNode }) {
   const [opened, setOpened] = useState(initial);
-  const { openChart } = useViewStore();
+  const [revision, setRevision] = useState(0);
+  const { openChart, closeChart } = useViewStore();
+  const { push, dismissWhere } = useNotices();
+  const seq = useRef(0);
+
+  const touch = useCallback(() => setRevision((r) => r + 1), []);
+
   const open = useCallback(
     (o: OpenedClient) => {
+      seq.current += 1;
+      // Eski açılış hatası ("client") da kalkar: yeni harita açıldı.
+      dismissWhere((n) => n.id === "boot" || n.id === "client" || n.id.startsWith("store:"));
+      for (const n of storeWarningNotices(o.warnings)) push(n);
       openChart(o.chart_record.id, o.view_state.dasha_expanded_path);
       setOpened(o);
+      touch();
+    },
+    [openChart, dismissWhere, push, touch],
+  );
+
+  const openClient = useCallback(
+    async (clientId: number, chartId: number | null = null) => {
+      const mine = ++seq.current;
+      const r = await api.openClient(clientId, chartId);
+      if (mine !== seq.current) return null;
+      if (r.status === "error") return r.error;
+      open(r.data);
+      return null;
     },
-    [openChart],
+    [open],
   );
-  const value = useMemo(() => ({ opened, open }), [opened, open]);
+
+  const close = useCallback(() => {
+    seq.current += 1;
+    dismissWhere((n) => n.id.startsWith("store:"));
+    closeChart();
+    setOpened(null);
+    touch();
+  }, [closeChart, dismissWhere, touch]);
+
+  const setTags = useCallback(
+    (chartId: number, tags: string[]) => {
+      setOpened((o) => (o && o.chart_record.id === chartId ? { ...o, chart_record: { ...o.chart_record, tags } } : o));
+      touch();
+    },
+    [touch],
+  );
+
+  const value = useMemo(() => ({ opened, revision, open, openClient, close, setTags, touch }), [opened, revision, open, openClient, close, setTags, touch]);
   return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
 }
 
```

`app/ui/shell/notices.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/notices.tsx
+++ b/app/ui/shell/notices.tsx
@@ -14,6 +14,8 @@ export interface Notices {
   notices: Notice[];
   push(notice: Notice): void;
   dismiss(id: string): void;
+  /** Koşula uyan bildirimleri o anki listeden kaldırır (`await` sonrası eski liste okunmaz). */
+  dismissWhere(match: (notice: Notice) => boolean): void;
 }
 
 const NoticeContext = createContext<Notices | null>(null);
@@ -22,7 +24,8 @@ export function NoticeProvider({ initial = [], children }: { initial?: Notice[];
   const [notices, setNotices] = useState(initial);
   const push = useCallback((n: Notice) => setNotices((all) => [...all.filter((m) => m.id !== n.id), n]), []);
   const dismiss = useCallback((id: string) => setNotices((all) => all.filter((m) => m.id !== id)), []);
-  const value = useMemo(() => ({ notices, push, dismiss }), [notices, push, dismiss]);
+  const dismissWhere = useCallback((match: (n: Notice) => boolean) => setNotices((all) => all.filter((m) => !match(m))), []);
+  const value = useMemo(() => ({ notices, push, dismiss, dismissWhere }), [notices, push, dismiss, dismissWhere]);
   return <NoticeContext.Provider value={value}>{children}</NoticeContext.Provider>;
 }
 
```

`app/ui/view/reducer.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/view/reducer.ts
+++ b/app/ui/view/reducer.ts
@@ -7,7 +7,8 @@ import { parseDashaPath, type ViewModel } from "./model";
 export type ViewAction =
   | { type: "layout"; action: LayoutAction }
   | { type: "setDashaPath"; path: Graha[] }
-  | { type: "openChart"; chartId: number; dashaPath: unknown };
+  | { type: "openChart"; chartId: number; dashaPath: unknown }
+  | { type: "closeChart" };
 
 export function viewReducer(state: ViewModel, action: ViewAction): ViewModel {
   switch (action.type) {
@@ -22,5 +23,7 @@ export function viewReducer(state: ViewModel, action: ViewAction): ViewModel {
     }
     case "openChart":
       return { ...state, chartId: action.chartId, dashaPath: parseDashaPath(action.dashaPath) };
+    case "closeChart":
+      return state.chartId === null ? state : { ...state, chartId: null, dashaPath: [] };
   }
 }
```

`app/ui/view/store.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/view/store.tsx
+++ b/app/ui/view/store.tsx
@@ -36,6 +36,8 @@ export interface ViewStore {
   setDashaPath(path: Graha[]): void;
   /** Yeni açılan harita: bekleyen kayıt önceki haritaya yazılır, yeni haritanın yolu yüklenir. */
   openChart(chartId: number, dashaPath: unknown): void;
+  /** Açık harita kapandı (ör. danışan silindi): bekleyen kayıt yazılır, sonraki kayıt yalnızca düzendir. */
+  closeChart(): void;
 }
 
 const ViewContext = createContext<{ view: ViewStore; layout: LayoutStore } | null>(null);
@@ -76,13 +78,14 @@ export function ViewStateProvider({
   });
 
   const dispatch = useCallback((action: ViewAction) => {
-    // Yeni harita: bekleyen kayıt önce önceki haritaya yazılır; açılan haritanın kendi durumu
-    // store'dan geldiği için yeniden yazılmaz.
-    if (action.type === "openChart") flushRef.current();
+    // Yeni ya da kapanan harita: bekleyen kayıt önce önceki haritaya yazılır; açılan haritanın
+    // kendi durumu store'dan geldiği için yeniden yazılmaz (carried rule C5).
+    const switching = action.type === "openChart" || action.type === "closeChart";
+    if (switching) flushRef.current();
     const next = viewReducer(stateRef.current, action);
     if (next === stateRef.current) return;
     stateRef.current = next;
-    if (action.type !== "openChart") {
+    if (!switching) {
       pending.current = next;
       clearTimeout(timer.current);
       timer.current = setTimeout(flushRef.current, debounceRef.current);
@@ -137,6 +140,7 @@ export function ViewStateProvider({
       viewState: toViewState(state),
       setDashaPath: (path) => dispatch({ type: "setDashaPath", path }),
       openChart: (chartId, dashaPath) => dispatch({ type: "openChart", chartId, dashaPath }),
+      closeChart: () => dispatch({ type: "closeChart" }),
     };
     return { view, layout };
   }, [state, dispatch]);
```

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app test
pnpm --dir app build
```
Expected: `Test Files  37 passed (37)`, `Tests  310 passed (310)`; `build` `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/birth/BirthForm.test.tsx \
  app/ui/birth/BirthForm.tsx \
  app/ui/dasha/DashaTreePanel.tsx \
  app/ui/i18n/en.ts \
  app/ui/i18n/fr.ts \
  app/ui/i18n/tr.ts \
  app/ui/panels/ChartPanel.tsx \
  app/ui/panels/GrahaTablePanel.tsx \
  app/ui/session/birth.ts \
  app/ui/session/store.test.tsx \
  app/ui/session/store.tsx \
  app/ui/shell/notices.tsx \
  app/ui/view/reducer.ts \
  app/ui/view/store.tsx
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git diff --cached --name-only | grep -iE 'k(a|ā|aa)la|\bcala\b|^\.superpowers/'   # boş olmalı
git commit -m "feat(ui): session opens clients and charts by id, closes and refreshes; add-chart form mode; notices dismissed by the current list

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: commit oluşur; iki tarama da çıktısız. Commit mesajında yasaklı sözcük yoktur (C13).

---

### Task 7: Danışan kenar çubuğu: arama, etiket süzgeci, son açılanlar, harita listesi, onaylı silme ve harita etiketleri; `Cmd+\`

UI spec §4 kenar çubuğu. Sekme yoktur ("Sekme yoktur"): tek açık harita, kenar çubuğu açık danışanın haritalarını ve listeyi gösterir. Liste boş sorguda `list_clients` (son açılan önce, Rust sıralar), etiket seçiliyse `list_clients_by_tag`, sorgu varsa 150 ms sonra `search_clients` ile okunur; eşleşme nedeni metinle gösterilir ("Notta: …", "Etiket eşleşmesi"). Etiketler arayüzde `Intl.Collator` ile dilin alfabesinde sıralanır (Rust `NOCASE` yalnızca ASCII katlar; Plan 08 Task 6 bulgusu). Silme geri alınamaz: `ConfirmDialog` (`shell/Dialog` üzerine, C8) odağı "Vazgeç"te başlatır. Danışanın tek haritası silinemez: düğme `aria-disabled` ve nedeni `aria-describedby` ile bağlı. Açık harita silinince danışanın en son güncellenen diğer haritası açılır; o harita açılamazsa (liste ya da açma hatası) silinen harita ekranda bırakılmaz, pano boşalır ve `notice.openFailed` gösterilir (Fable S1); açık danışan silinince pano boşalır. Silme düğmesinin erişilebilir adı haritanın adını taşır (`Haritayı sil: “…”`, Fable S3). Etiketler virgülle ayrılır, Enter ya da odak çıkışında yalnızca değiştiyse kaydedilir; ekranda Rust'ın normalleştirdiği liste görünür. `TagEditor` haritanın kimliğiyle anahtarlanır (`key={chart_record.id}`): harita değişince yazılmamış taslak bir haritadan ötekine taşınmaz (Fable MF5). Shell'in klavye dinleyicisi bir kez kurulur ve güncel durumu bir ref'ten okur (Plan 09 Task 12'deki `Cmd+E` kararsızlığının nedeni eski kapanıştı).

**Files:**
- Test: `app/ui/App.test.tsx`
- Create: `app/ui/clients/ClientList.tsx`
- Create: `app/ui/clients/OpenClient.tsx`
- Test (yeni): `app/ui/clients/Sidebar.test.tsx`
- Create: `app/ui/clients/Sidebar.tsx`
- Create: `app/ui/clients/TagEditor.tsx`
- Create: `app/ui/clients/clients.css`
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Modify: `app/ui/i18n/tr.ts`
- Create: `app/ui/shell/ConfirmDialog.tsx`
- Modify: `app/ui/shell/Shell.tsx`
- Modify: `app/ui/shell/TopBar.tsx`
- Modify: `app/ui/shell/shell.css`

**Interfaces:**
- Consumes: `Session` (Task 6), `api.listClients/listClientsByTag/searchClients/listTags/listCharts/deleteChart/deleteClient/setChartTags` (Task 3), `matchShortcut` `sidebar` (Task 5), `Dialog` (Plan 09).
- Produces: `shell/ConfirmDialog` `ConfirmDialog({ title, message, confirmLabel, onConfirm(), onCancel() })`; `clients/Sidebar` `Sidebar({ onAddChart(): void })`, `SIDEBAR_ID = "client-sidebar"`; `clients/ClientList` `ClientList()`, `CLIENT_SEARCH_DEBOUNCE_MS = 150`, `SIDEBAR_LIMIT = 200`, `sortTags(tags: TagCount[], language: string): TagCount[]`; `clients/OpenClient` `OpenClient({ opened, onAddChart })`; `clients/TagEditor` `TagEditor({ chartId, tags })`, `splitTags(text: string): string[]`; `TopBarProps.sidebarOpen: boolean`, `onToggleSidebar(): void`; i18n `clients.*`, `notice.openFailed/deleteFailed/tagsFailed`; CSS `.button-danger` (yalnızca kenar rengi; metin kontrastı değişmez).

- [ ] **Step 1: Önce testleri yaz**

`app/ui/App.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -10,6 +10,9 @@ vi.mock("./api", () => ({
     updateChart: vi.fn(),
     saveViewState: vi.fn(async () => ({ status: "ok", data: null })),
     savePanelLayout: vi.fn(async () => ({ status: "ok", data: null })),
+    listClients: vi.fn(async () => ({ status: "ok", data: [] })),
+    listTags: vi.fn(async () => ({ status: "ok", data: [] })),
+    listCharts: vi.fn(async () => ({ status: "ok", data: [] })),
   },
 }));
 
@@ -167,6 +170,18 @@ test("Cmd+Shift+N still works after a choice in a select (M6-2)", () => {
   expect(screen.queryByRole("region", { name: "Notlar" })).toBeNull();
 });
 
+test("Cmd+\\ and the Clients button show and hide the client sidebar", () => {
+  renderApp();
+  const toggle = screen.getByRole("button", { name: "Danışanlar" });
+  expect(toggle).toHaveAttribute("aria-expanded", "true");
+  expect(screen.getByRole("complementary", { name: "Danışanlar" })).toBeInTheDocument();
+  fireEvent.keyDown(window, { key: "\\", code: "Backslash", metaKey: true });
+  expect(screen.queryByRole("complementary", { name: "Danışanlar" })).toBeNull();
+  expect(toggle).toHaveAttribute("aria-expanded", "false");
+  fireEvent.click(toggle);
+  expect(screen.getByRole("complementary", { name: "Danışanlar" })).toBeInTheDocument();
+});
+
 test("theme menu sets data-theme and remembers the choice", () => {
   renderApp();
   const select = screen.getByRole("combobox", { name: "Tema" });
```

`app/ui/clients/Sidebar.test.tsx` oluştur:

```tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    listClients: vi.fn(),
    listClientsByTag: vi.fn(),
    searchClients: vi.fn(),
    listTags: vi.fn(),
    listCharts: vi.fn(),
    openClient: vi.fn(),
    deleteChart: vi.fn(),
    deleteClient: vi.fn(),
    setChartTags: vi.fn(),
  },
}));

import { api } from "../api";
import type { ChartRecord, Client, OpenedClient } from "../bindings";
import { useSession } from "../session/store";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { CLIENT_SEARCH_DEBOUNCE_MS, sortTags } from "./ClientList";
import { Sidebar } from "./Sidebar";

const ok = <T,>(data: T) => ({ status: "ok" as const, data });
const opened = () => fixture().opened;
const client = (id: number, name: string): Client => ({ ...opened().client, id, name, last_chart_id: id * 10 });
const chart = (id: number, label: string, date = "2000-01-01"): ChartRecord => ({
  ...opened().chart_record,
  id,
  label,
  birth: { ...opened().chart_record.birth, date },
});
const ayse = client(2, "Ayşe Yılmaz");
const onAddChart = vi.fn();

function OpenedProbe() {
  const { opened } = useSession();
  return <output aria-label="açık">{opened ? `${opened.client.name}#${opened.chart_record.id}` : "-"}</output>;
}

function renderSidebar(o: OpenedClient | null = opened()) {
  return renderWithStores(
    <>
      <OpenedProbe />
      <Sidebar onAddChart={onAddChart} />
    </>,
    { opened: o },
  );
}

const sidebar = () => screen.getByRole("complementary", { name: "Danışanlar" });
const list = () => within(sidebar()).getByRole("region", { name: /Son açılanlar|Arama sonuçları|etiketli/ });

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  onAddChart.mockReset();
  vi.mocked(api.listClients).mockResolvedValue(ok([opened().client, ayse]));
  vi.mocked(api.listTags).mockResolvedValue(ok([]));
  vi.mocked(api.listCharts).mockResolvedValue(ok([opened().chart_record]));
});

test("recent clients come from list_clients, the open one is marked and choosing another opens it", async () => {
  const other = opened();
  other.client = ayse;
  other.chart_record = { ...other.chart_record, id: 20, client_id: 2 };
  vi.mocked(api.openClient).mockResolvedValueOnce(ok(other));
  renderSidebar();
  const mine = await within(list()).findByRole("button", { name: "Örnek Danışan" });
  expect(mine).toHaveAttribute("aria-current", "true");
  expect(api.listClients).toHaveBeenCalledWith(200);
  fireEvent.click(within(list()).getByRole("button", { name: "Ayşe Yılmaz" }));
  await waitFor(() => expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Ayşe Yılmaz#20"));
  expect(api.openClient).toHaveBeenCalledWith(2, null);
  // Açılış listeyi yeniden okur; açık danışan işareti yeni danışana geçer.
  await waitFor(() => expect(within(list()).getByRole("button", { name: "Ayşe Yılmaz" })).toHaveAttribute("aria-current", "true"));
});

test("typing searches after the debounce and shows why a client matched; clearing shows recent again", async () => {
  vi.useFakeTimers();
  vi.mocked(api.searchClients).mockResolvedValue(ok([{ client: ayse, matched: "Note", snippet: "…satürn dönüşü…" }]));
  renderSidebar(null);
  const box = within(sidebar()).getByRole("searchbox", { name: "Danışan ara" });
  fireEvent.change(box, { target: { value: "satürn" } });
  expect(api.searchClients).not.toHaveBeenCalled();
  await act(async () => vi.advanceTimersByTime(CLIENT_SEARCH_DEBOUNCE_MS));
  vi.useRealTimers();
  expect(api.searchClients).toHaveBeenCalledWith("satürn", 200);
  expect(within(list()).getByRole("heading")).toHaveTextContent("Arama sonuçları");
  expect(within(list()).getByRole("button", { name: /Ayşe Yılmaz/ })).toHaveTextContent("Notta: …satürn dönüşü…");
  fireEvent.change(box, { target: { value: "" } });
  expect(await within(list()).findByRole("button", { name: "Örnek Danışan" })).toBeInTheDocument();
});

test("the tag filter lists clients by tag; tags are in the language's alphabetical order", async () => {
  vi.mocked(api.listTags).mockResolvedValue(ok([{ name: "zeki", charts: 1 }, { name: "çocuk", charts: 2 }, { name: "araştırma", charts: 3 }]));
  vi.mocked(api.listClientsByTag).mockResolvedValue(ok([ayse]));
  renderSidebar(null);
  const select = within(sidebar()).getByRole("combobox", { name: "Etiket" });
  await waitFor(() => expect(within(select).getAllByRole("option")).toHaveLength(4));
  expect(within(select).getAllByRole("option").map((o) => o.textContent)).toEqual(["Tüm etiketler", "araştırma (3)", "çocuk (2)", "zeki (1)"]);
  fireEvent.change(select, { target: { value: "çocuk" } });
  await waitFor(() => expect(api.listClientsByTag).toHaveBeenCalledWith("çocuk", 200));
  expect(await within(list()).findByRole("button", { name: "Ayşe Yılmaz" })).toBeInTheDocument();
  expect(within(list()).getByRole("heading")).toHaveTextContent("“çocuk” etiketli");
  expect(sortTags([{ name: "Öfke", charts: 1 }, { name: "Zeki", charts: 1 }, { name: "Ofis", charts: 1 }], "tr").map((t) => t.name)).toEqual(["Ofis", "Öfke", "Zeki"]);
});

test("a list error is shown with a retry", async () => {
  vi.mocked(api.listClients).mockResolvedValueOnce({ status: "error", error: { code: "store_unavailable", params: {} } });
  renderSidebar(null);
  const alert = await within(sidebar()).findByRole("alert");
  expect(alert).toHaveTextContent("Liste okunamadı: Danışan veritabanı kullanılamıyor.");
  fireEvent.click(within(alert).getByRole("button", { name: "Yeniden dene" }));
  expect(await within(list()).findByRole("button", { name: "Örnek Danışan" })).toBeInTheDocument();
});

test("a client's only chart cannot be deleted and the reason is attached to the button", async () => {
  renderSidebar();
  const del = await within(sidebar()).findByRole("button", { name: "Haritayı sil: “Doğum haritası”" });
  expect(del).toHaveAttribute("aria-disabled", "true");
  expect(del).toHaveAccessibleDescription("Danışanın tek haritası silinemez; gerekirse danışanı silin.");
  fireEvent.click(del);
  expect(screen.queryByRole("dialog")).toBeNull();
  fireEvent.click(within(sidebar()).getByRole("button", { name: "Harita ekle" }));
  expect(onAddChart).toHaveBeenCalledOnce();
});

test("deleting the open chart after confirmation opens the most recently updated other chart", async () => {
  const charts = [chart(1, ""), chart(5, "Düzeltilmiş", "2000-01-02")];
  vi.mocked(api.listCharts).mockResolvedValueOnce(ok(charts)).mockResolvedValue(ok([charts[1]]));
  vi.mocked(api.deleteChart).mockResolvedValue(ok(null));
  const next = opened();
  next.chart_record = charts[1];
  vi.mocked(api.openClient).mockResolvedValue(ok(next));
  renderSidebar();
  const del = await within(sidebar()).findByRole("button", { name: "Haritayı sil: “Doğum haritası”" });
  fireEvent.click(del);
  const dialog = screen.getByRole("dialog", { name: "Harita silinsin mi?" });
  expect(document.activeElement).toBe(within(dialog).getByRole("button", { name: "Vazgeç" }));
  expect(dialog).toHaveTextContent("“Doğum haritası” haritası silinecek. Bu işlem geri alınamaz; danışanın notları silinmez.");
  fireEvent.click(within(dialog).getByRole("button", { name: "Haritayı sil" }));
  await waitFor(() => expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Örnek Danışan#5"));
  expect(api.deleteChart).toHaveBeenCalledWith(1);
  expect(api.openClient).toHaveBeenCalledWith(1, 5);
});

test("deleting the open client after confirmation empties the desk; cancelling deletes nothing", async () => {
  vi.mocked(api.deleteClient).mockResolvedValue(ok(null));
  renderSidebar();
  fireEvent.click(within(sidebar()).getByRole("button", { name: "Danışanı sil…" }));
  fireEvent.click(within(screen.getByRole("dialog", { name: "Danışan silinsin mi?" })).getByRole("button", { name: "Vazgeç" }));
  expect(api.deleteClient).not.toHaveBeenCalled();
  fireEvent.click(within(sidebar()).getByRole("button", { name: "Danışanı sil…" }));
  const dialog = screen.getByRole("dialog", { name: "Danışan silinsin mi?" });
  expect(dialog).toHaveTextContent("“Örnek Danışan” bütün haritaları ve notlarıyla silinecek. Bu işlem geri alınamaz.");
  fireEvent.click(within(dialog).getByRole("button", { name: "Danışanı sil" }));
  await waitFor(() => expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("-"));
  expect(api.deleteClient).toHaveBeenCalledWith(1);
});

test("tags are saved on Enter as the normalised list Rust returns", async () => {
  vi.mocked(api.setChartTags).mockResolvedValue(ok(["araştırma", "Kariyer"]));
  renderSidebar();
  const box = within(sidebar()).getByRole("textbox", { name: "Etiketler" });
  expect(box).toHaveAccessibleDescription("Virgülle ayırın; Enter kaydeder.");
  fireEvent.change(box, { target: { value: " Kariyer, araştırma ,, kariyer" } });
  fireEvent.keyDown(box, { key: "Enter" });
  await waitFor(() => expect(box).toHaveValue("araştırma, Kariyer"));
  expect(api.setChartTags).toHaveBeenCalledWith(1, ["Kariyer", "araştırma", "kariyer"]);
  // Değişmeyen metin yeniden kaydedilmez.
  fireEvent.blur(box);
  expect(api.setChartTags).toHaveBeenCalledOnce();
});
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/clients ui/App.test.tsx
```
Expected: `clients/Sidebar.test.tsx` `Failed to resolve import "./ClientList"` ile FAIL; App `Cmd+\\ and the Clients button…` FAIL (`Danışanlar` düğmesi yok).

- [ ] **Step 3: Uygulamayı yaz**

`app/ui/clients/ClientList.tsx` oluştur:

```tsx
// Kenar çubuğunun liste yarısı (UI spec §4): arama, etiket süzgeci ve son açılanlar. Arama
// `search_clients` (ad, etiket, not), süzgeç `list_clients_by_tag`, boş sorgu `list_clients`
// ile okunur; sıralama Rust'tadır (son açılan önce). Danışan seçilince son haritası açılır.
import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, type Result } from "../api";
import type { Client, CommandError, SearchHit, TagCount } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { useSession } from "../session/store";
import { useNotices } from "../shell/notices";

export const CLIENT_SEARCH_DEBOUNCE_MS = 150;
/** Kenar çubuğunda gösterilen en çok danışan (Rust `MAX_LIST` 500). */
export const SIDEBAR_LIMIT = 200;

type Row = { client: Client; hit?: SearchHit };

/** Etiketleri dilin alfabesiyle sıralar (Rust yalnızca ASCII büyük/küçük harf katlar). */
export function sortTags(tags: TagCount[], language: string): TagCount[] {
  const collator = new Intl.Collator(language, { sensitivity: "base" });
  return [...tags].sort((a, b) => collator.compare(a.name, b.name));
}

export function ClientList() {
  const { t, i18n } = useTranslation();
  const session = useSession();
  const { push } = useNotices();
  const id = useId();
  const [query, setQuery] = useState("");
  const [tag, setTag] = useState("");
  const [tags, setTags] = useState<TagCount[]>([]);
  const [rows, setRows] = useState<Row[] | null>(null);
  const [error, setError] = useState<CommandError | null>(null);
  const [retry, setRetry] = useState(0);
  const seq = useRef(0);

  useEffect(() => {
    void api.listTags().then((r) => r.status === "ok" && setTags(r.data));
  }, [session.revision]);

  useEffect(() => {
    const mine = ++seq.current;
    const q = query.trim();
    const run = async () => {
      let r: Result<Row[]>;
      if (q !== "") {
        const hits = await api.searchClients(q, SIDEBAR_LIMIT);
        r = hits.status === "ok" ? { status: "ok", data: hits.data.map((hit) => ({ client: hit.client, hit })) } : hits;
      } else {
        const list = tag !== "" ? await api.listClientsByTag(tag, SIDEBAR_LIMIT) : await api.listClients(SIDEBAR_LIMIT);
        r = list.status === "ok" ? { status: "ok", data: list.data.map((client) => ({ client })) } : list;
      }
      if (mine !== seq.current) return;
      if (r.status === "ok") {
        setRows(r.data);
        setError(null);
      } else setError(r.error);
    };
    // Yalnızca yazarken beklenir; süzgeç, yenileme ve ilk okuma hemen.
    const timer = setTimeout(() => void run(), q !== "" ? CLIENT_SEARCH_DEBOUNCE_MS : 0);
    return () => clearTimeout(timer);
  }, [query, tag, session.revision, retry]);

  const choose = async (client: Client) => {
    const e = await session.openClient(client.id);
    if (e) push({ id: "client", type: "error", messageKey: "notice.openFailed", error: e });
  };

  const heading = query.trim() !== "" ? t("clients.results") : tag !== "" ? t("clients.tagged", { tag }) : t("clients.recent");
  const openId = session.opened?.client.id;

  return (
    <section className="client-list" aria-labelledby={`${id}-heading`}>
      <div className="sidebar-field">
        <label htmlFor={`${id}-q`}>{t("clients.search")}</label>
        <input
          id={`${id}-q`}
          type="search"
          value={query}
          aria-describedby={`${id}-q-hint`}
          onChange={(e) => {
            setQuery(e.target.value);
            if (e.target.value.trim() !== "") setTag("");
          }}
        />
        <p id={`${id}-q-hint`} className="sidebar-hint">
          {t("clients.searchHint")}
        </p>
      </div>
      <div className="sidebar-field">
        <label htmlFor={`${id}-tag`}>{t("clients.tag")}</label>
        <select
          id={`${id}-tag`}
          value={tag}
          onChange={(e) => {
            setTag(e.target.value);
            setQuery("");
          }}
        >
          <option value="">{t("clients.allTags")}</option>
          {sortTags(tags, i18n.language).map((tc) => (
            <option key={tc.name} value={tc.name}>
              {t("clients.tagOption", { tag: tc.name, n: tc.charts })}
            </option>
          ))}
        </select>
      </div>
      <h2 id={`${id}-heading`} className="sidebar-heading">
        {heading}
      </h2>
      {error && (
        <div className="sidebar-error" role="alert">
          <p>{t("clients.loadFailed", { message: errorMessage(t, i18n, error) })}</p>
          <button type="button" onClick={() => setRetry((n) => n + 1)}>
            {t("clients.retry")}
          </button>
        </div>
      )}
      {rows && rows.length === 0 && !error && <p className="sidebar-empty">{query.trim() !== "" || tag !== "" ? t("clients.noResults") : t("clients.none")}</p>}
      {rows && rows.length > 0 && (
        <ul className="client-rows">
          {rows.map(({ client, hit }) => (
            <li key={client.id}>
              <button type="button" aria-current={client.id === openId ? "true" : undefined} onClick={() => void choose(client)}>
                <span className="client-name">{client.name}</span>
                {hit?.matched === "Tag" && <span className="client-meta">{t("clients.matchedTag")}</span>}
                {hit?.matched === "Note" && hit.snippet && <span className="client-meta">{t("clients.matchedNote", { snippet: hit.snippet })}</span>}
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
```

`app/ui/clients/OpenClient.tsx` oluştur:

```tsx
// Kenar çubuğunun açık danışan bölümü: haritaları (`list_charts`), harita ekleme, harita ve
// danışan silme (onaylı, geri alınamaz) ve açık haritanın etiketleri. Açık harita silinirse
// danışanın en son güncellenen haritası açılır; açık danışan silinirse pano boşalır.
import { useEffect, useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { ChartRecord, CommandError, OpenedClient } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { useSession } from "../session/store";
import { ConfirmDialog } from "../shell/ConfirmDialog";
import { useNotices } from "../shell/notices";
import { TagEditor } from "./TagEditor";

type Pending = { kind: "chart"; chart: ChartRecord } | { kind: "client" };

export function OpenClient({ opened, onAddChart }: { opened: OpenedClient; onAddChart(): void }) {
  const { t, i18n } = useTranslation();
  const session = useSession();
  const { push } = useNotices();
  const id = useId();
  const clientId = opened.client.id;
  const [charts, setCharts] = useState<ChartRecord[]>([]);
  const [error, setError] = useState<CommandError | null>(null);
  const [pending, setPending] = useState<Pending | null>(null);

  useEffect(() => {
    let live = true;
    void api.listCharts(clientId).then((r) => {
      if (!live) return;
      if (r.status === "ok") {
        setCharts(r.data);
        setError(null);
      } else setError(r.error);
    });
    return () => {
      live = false;
    };
  }, [clientId, session.revision]);

  const label = (c: ChartRecord) => c.label || t(`chart.kind.${c.kind}`);
  const failed = (e: CommandError) => push({ id: "delete", type: "error", messageKey: "notice.deleteFailed", error: e });

  const deleteChart = async (chart: ChartRecord) => {
    setPending(null);
    const r = await api.deleteChart(chart.id);
    if (r.status === "error") return failed(r.error);
    if (chart.id !== opened.chart_record.id) return session.touch();
    // Açık harita silindi: diğer haritalardan en son güncelleneni açılır (uyarısız).
    const rest = await api.listCharts(clientId);
    const next = rest.status === "ok" ? rest.data[0] : undefined;
    const e = next ? await session.openClient(clientId, next.id) : rest.status === "error" ? rest.error : null;
    // Diğer harita açılamazsa silinen harita ekranda bırakılmaz: pano boşalır.
    if (!next || e) session.close();
    if (e) push({ id: "client", type: "error", messageKey: "notice.openFailed", error: e });
  };

  const deleteClient = async () => {
    setPending(null);
    const r = await api.deleteClient(clientId);
    if (r.status === "error") return failed(r.error);
    session.close();
  };

  const onlyChart = charts.length <= 1;
  return (
    <section className="open-client" aria-labelledby={`${id}-name`}>
      <h2 id={`${id}-name`} className="sidebar-heading">
        {opened.client.name}
      </h2>
      <h3 className="sidebar-subheading">{t("clients.charts")}</h3>
      {error && (
        <p className="sidebar-error" role="alert">
          {t("clients.loadFailed", { message: errorMessage(t, i18n, error) })}
        </p>
      )}
      <ul className="chart-rows">
        {charts.map((c) => (
          <li key={c.id}>
            <button
              type="button"
              aria-current={c.id === opened.chart_record.id ? "true" : undefined}
              onClick={() => void session.openClient(clientId, c.id).then((e) => e && push({ id: "client", type: "error", messageKey: "notice.openFailed", error: e }))}
            >
              <span className="client-name">{label(c)}</span>
              <span className="client-meta">{c.birth.date}</span>
            </button>
            <button
              type="button"
              className="icon-button"
              aria-label={t("clients.deleteChartNamed", { label: label(c) })}
              aria-disabled={onlyChart ? "true" : undefined}
              aria-describedby={onlyChart ? `${id}-last` : undefined}
              onClick={() => !onlyChart && setPending({ kind: "chart", chart: c })}
            >
              {t("clients.deleteChart")}
            </button>
          </li>
        ))}
      </ul>
      {onlyChart && (
        <p id={`${id}-last`} className="sidebar-hint">
          {t("clients.lastChartHint")}
        </p>
      )}
      <div className="sidebar-actions">
        <button type="button" onClick={onAddChart}>
          {t("clients.addChart")}
        </button>
        <button type="button" onClick={() => setPending({ kind: "client" })}>
          {t("clients.deleteClient")}
        </button>
      </div>
      {/* Harita başına anahtar: önceki haritanın geç gelen kayıt yanıtı yeni haritanın kutusuna yazılmaz. */}
      <TagEditor key={opened.chart_record.id} chartId={opened.chart_record.id} tags={opened.chart_record.tags} />
      {pending?.kind === "chart" && (
        <ConfirmDialog
          title={t("clients.deleteChartTitle")}
          message={t("clients.confirmDeleteChart", { label: label(pending.chart) })}
          confirmLabel={t("clients.deleteChart")}
          onConfirm={() => void deleteChart(pending.chart)}
          onCancel={() => setPending(null)}
        />
      )}
      {pending?.kind === "client" && (
        <ConfirmDialog
          title={t("clients.deleteClientTitle")}
          message={t("clients.confirmDeleteClient", { name: opened.client.name })}
          confirmLabel={t("clients.deleteClientConfirm")}
          onConfirm={() => void deleteClient()}
          onCancel={() => setPending(null)}
        />
      )}
    </section>
  );
}
```

`app/ui/clients/Sidebar.tsx` oluştur:

```tsx
// Sol kenar çubuğu (UI spec §4): açık danışanın haritaları ve etiketleri, danışan listesi,
// arama, etiket süzgeci ve son açılanlar. `Cmd+\` ile açılıp kapanır (Shell).
import { useTranslation } from "react-i18next";
import { useSession } from "../session/store";
import { ClientList } from "./ClientList";
import "./clients.css";
import { OpenClient } from "./OpenClient";

export const SIDEBAR_ID = "client-sidebar";

export function Sidebar({ onAddChart }: { onAddChart(): void }) {
  const { t } = useTranslation();
  const { opened } = useSession();
  return (
    <aside id={SIDEBAR_ID} className="sidebar no-print" aria-label={t("clients.region")}>
      {opened && <OpenClient key={opened.client.id} opened={opened} onAddChart={onAddChart} />}
      <ClientList />
    </aside>
  );
}
```

`app/ui/clients/TagEditor.tsx` oluştur:

```tsx
// Açık haritanın etiketleri (spec §5: etiketler harita düzeyindedir). Virgülle ayrılmış metin;
// Enter ya da alandan çıkış kaydeder. Normalleştirme (kırpma, NFC, büyük/küçük harf, tekrar)
// Rust'tadır; alan kayıttan dönen listeyi gösterir.
import { useEffect, useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { useSession } from "../session/store";

export function splitTags(text: string): string[] {
  return text
    .split(",")
    .map((t) => t.trim())
    .filter((t) => t !== "");
}

export function TagEditor({ chartId, tags }: { chartId: number; tags: string[] }) {
  const { t, i18n } = useTranslation();
  const { setTags } = useSession();
  const id = useId();
  const saved = tags.join(", ");
  const [text, setText] = useState(saved);
  const [error, setError] = useState<CommandError | null>(null);
  useEffect(() => setText(saved), [saved]);

  const commit = async () => {
    const next = splitTags(text);
    if (next.join(", ") === saved) return;
    const r = await api.setChartTags(chartId, next);
    if (r.status === "error") {
      setError(r.error);
      return;
    }
    setError(null);
    setText(r.data.join(", "));
    setTags(chartId, r.data);
  };

  return (
    <div className="tag-editor">
      <label htmlFor={`${id}-tags`}>{t("clients.tags")}</label>
      <input
        id={`${id}-tags`}
        value={text}
        aria-describedby={`${id}-hint${error ? ` ${id}-error` : ""}`}
        aria-invalid={error ? true : undefined}
        onChange={(e) => setText(e.target.value)}
        onBlur={() => void commit()}
        onKeyDown={(e) => {
          if (e.key === "Enter" && !e.nativeEvent.isComposing) {
            e.preventDefault();
            void commit();
          }
        }}
      />
      <p id={`${id}-hint`} className="sidebar-hint">
        {t("clients.tagsHint")}
      </p>
      {error && (
        <p id={`${id}-error`} className="sidebar-error" role="alert">
          {t("notice.tagsFailed", { message: errorMessage(t, i18n, error) })}
        </p>
      )}
    </div>
  );
}
```

`app/ui/clients/clients.css` oluştur:

```css
.sidebar {
  flex: 0 0 248px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 8px;
  background: var(--panel);
  border-right: 1px solid var(--line);
  color: var(--text);
  overflow: auto;
}
.sidebar-heading {
  margin: 4px 0;
  font-size: var(--fs-panel-title);
  font-weight: var(--fw-medium);
}
.sidebar-subheading {
  margin: 4px 0;
  font-size: var(--fs-body);
  font-weight: var(--fw-medium);
  color: var(--text-2);
}
.sidebar-field {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.sidebar-hint {
  margin: 2px 0 0;
  color: var(--text-2);
  font-size: var(--fs-table);
}
.sidebar-error {
  color: var(--danger);
  font-size: var(--fs-table);
}
.sidebar-empty {
  color: var(--text-2);
}
.sidebar-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.client-rows,
.chart-rows {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.client-rows button,
.chart-rows button:first-child {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  text-align: left;
  padding: 4px 6px;
  border: 1px solid transparent;
  border-radius: 4px;
  background: transparent;
  color: var(--text);
}
.chart-rows li {
  display: flex;
  gap: 4px;
  align-items: center;
}
/* Açık danışan ve harita: zemin rengi + kenar çizgisi + kalın ad (renk tek başına değil). */
.client-rows button[aria-current="true"],
.chart-rows button[aria-current="true"] {
  background: var(--select);
  border-color: var(--ink);
  font-weight: var(--fw-medium);
}
.client-meta {
  color: var(--text-2);
  font-size: var(--fs-table);
}
.icon-button[aria-disabled="true"] {
  opacity: 0.6;
  cursor: not-allowed;
}
.tag-editor {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.button-danger {
  border: 1px solid var(--danger);
}
```

`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -173,6 +173,9 @@ export const en: Messages = {
     restoreFailed: "The last session could not be opened: {{message}}",
     saveFailed: "The view could not be saved: {{message}}",
     layoutLoadFailed: "The saved layout could not be read: {{message}} View changes in this session will not be saved; the saved layout is left as it is.",
+    openFailed: "The client could not be opened: {{message}}",
+    deleteFailed: "Could not delete: {{message}}",
+    tagsFailed: "Tags could not be saved: {{message}}",
   },
   storeWarning: {
     LastChartMissing: "The last opened chart was deleted; the most recently updated chart was opened.",
@@ -258,6 +261,38 @@ export const en: Messages = {
       offset: "Enter the offset as +05:30.",
     },
   },
+  clients: {
+    region: "Clients",
+    toggle: "Clients",
+    search: "Search clients",
+    searchHint: "Name, tag or note",
+    tag: "Tag",
+    allTags: "All tags",
+    tagOption: "{{tag}} ({{n}})",
+    recent: "Recently opened",
+    results: "Search results",
+    tagged: "Tagged “{{tag}}”",
+    none: "No clients yet.",
+    noResults: "No results.",
+    matchedTag: "Tag match",
+    matchedNote: "In a note: {{snippet}}",
+    loadFailed: "The list could not be read: {{message}}",
+    retry: "Try again",
+    charts: "Charts",
+    addChart: "Add chart",
+    deleteChart: "Delete chart",
+    deleteChartNamed: "Delete chart “{{label}}”",
+    deleteChartTitle: "Delete this chart?",
+    confirmDeleteChart: "Chart “{{label}}” will be deleted. This cannot be undone; the client’s notes are not deleted.",
+    lastChartHint: "A client’s only chart cannot be deleted; delete the client instead if needed.",
+    deleteClient: "Delete client…",
+    deleteClientTitle: "Delete this client?",
+    deleteClientConfirm: "Delete client",
+    confirmDeleteClient: "“{{name}}” will be deleted with all charts and notes. This cannot be undone.",
+    cancel: "Cancel",
+    tags: "Tags",
+    tagsHint: "Separate with commas; Enter saves.",
+  },
   empty: {
     title: "No chart is open",
     body: "Start by entering the client's birth data.",
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -173,6 +173,9 @@ export const fr: Messages = {
     restoreFailed: "La dernière session n’a pas pu être ouverte : {{message}}",
     saveFailed: "La vue n’a pas pu être enregistrée : {{message}}",
     layoutLoadFailed: "La disposition enregistrée n’a pas pu être lue : {{message}} Les changements de vue de cette session ne seront pas enregistrés ; la disposition enregistrée reste intacte.",
+    openFailed: "Le client n’a pas pu être ouvert : {{message}}",
+    deleteFailed: "Suppression impossible : {{message}}",
+    tagsFailed: "Les étiquettes n’ont pas pu être enregistrées : {{message}}",
   },
   storeWarning: {
     LastChartMissing: "La dernière carte ouverte a été supprimée ; la plus récemment modifiée a été ouverte.",
@@ -258,6 +261,38 @@ export const fr: Messages = {
       offset: "Saisissez le décalage sous la forme +05:30.",
     },
   },
+  clients: {
+    region: "Clients",
+    toggle: "Clients",
+    search: "Rechercher un client",
+    searchHint: "Nom, étiquette ou note",
+    tag: "Étiquette",
+    allTags: "Toutes les étiquettes",
+    tagOption: "{{tag}} ({{n}})",
+    recent: "Ouverts récemment",
+    results: "Résultats",
+    tagged: "Étiquette « {{tag}} »",
+    none: "Aucun client pour l’instant.",
+    noResults: "Aucun résultat.",
+    matchedTag: "Étiquette correspondante",
+    matchedNote: "Dans une note : {{snippet}}",
+    loadFailed: "La liste n’a pas pu être lue : {{message}}",
+    retry: "Réessayer",
+    charts: "Cartes",
+    addChart: "Ajouter une carte",
+    deleteChart: "Supprimer la carte",
+    deleteChartNamed: "Supprimer la carte « {{label}} »",
+    deleteChartTitle: "Supprimer cette carte ?",
+    confirmDeleteChart: "La carte « {{label}} » sera supprimée. Cette action est irréversible ; les notes du client ne sont pas supprimées.",
+    lastChartHint: "La seule carte d’un client ne peut pas être supprimée ; supprimez le client si nécessaire.",
+    deleteClient: "Supprimer le client…",
+    deleteClientTitle: "Supprimer ce client ?",
+    deleteClientConfirm: "Supprimer le client",
+    confirmDeleteClient: "« {{name}} » sera supprimé avec toutes ses cartes et notes. Cette action est irréversible.",
+    cancel: "Annuler",
+    tags: "Étiquettes",
+    tagsHint: "Séparez par des virgules ; Entrée enregistre.",
+  },
   empty: {
     title: "Aucune carte ouverte",
     body: "Commencez par saisir les données de naissance du consultant.",
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -172,6 +172,9 @@ export const tr = {
     restoreFailed: "Son oturum açılamadı: {{message}}",
     saveFailed: "Görünüm kaydedilemedi: {{message}}",
     layoutLoadFailed: "Kayıtlı düzen okunamadı: {{message}} Bu oturumdaki görünüm değişiklikleri kaydedilmeyecek; kayıtlı düzen olduğu gibi duruyor.",
+    openFailed: "Danışan açılamadı: {{message}}",
+    deleteFailed: "Silinemedi: {{message}}",
+    tagsFailed: "Etiketler kaydedilemedi: {{message}}",
   },
   storeWarning: {
     LastChartMissing: "Son açılan harita silinmiş; en son güncellenen harita açıldı.",
@@ -257,6 +260,38 @@ export const tr = {
       offset: "Ofseti +05:30 biçiminde girin.",
     },
   },
+  clients: {
+    region: "Danışanlar",
+    toggle: "Danışanlar",
+    search: "Danışan ara",
+    searchHint: "Ad, etiket ya da not",
+    tag: "Etiket",
+    allTags: "Tüm etiketler",
+    tagOption: "{{tag}} ({{n}})",
+    recent: "Son açılanlar",
+    results: "Arama sonuçları",
+    tagged: "“{{tag}}” etiketli",
+    none: "Henüz danışan yok.",
+    noResults: "Sonuç yok.",
+    matchedTag: "Etiket eşleşmesi",
+    matchedNote: "Notta: {{snippet}}",
+    loadFailed: "Liste okunamadı: {{message}}",
+    retry: "Yeniden dene",
+    charts: "Haritalar",
+    addChart: "Harita ekle",
+    deleteChart: "Haritayı sil",
+    deleteChartNamed: "Haritayı sil: “{{label}}”",
+    deleteChartTitle: "Harita silinsin mi?",
+    confirmDeleteChart: "“{{label}}” haritası silinecek. Bu işlem geri alınamaz; danışanın notları silinmez.",
+    lastChartHint: "Danışanın tek haritası silinemez; gerekirse danışanı silin.",
+    deleteClient: "Danışanı sil…",
+    deleteClientTitle: "Danışan silinsin mi?",
+    deleteClientConfirm: "Danışanı sil",
+    confirmDeleteClient: "“{{name}}” bütün haritaları ve notlarıyla silinecek. Bu işlem geri alınamaz.",
+    cancel: "Vazgeç",
+    tags: "Etiketler",
+    tagsHint: "Virgülle ayırın; Enter kaydeder.",
+  },
   empty: {
     title: "Açık harita yok",
     body: "Danışanın doğum bilgisini girerek başlayın.",
```

`app/ui/shell/ConfirmDialog.tsx` oluştur:

```tsx
// Geri alınamaz işlemin onayı (silme). Odak "Vazgeç"te başlar: Enter yanlışlıkla silmez.
import { useTranslation } from "react-i18next";
import { Dialog } from "./Dialog";

export interface ConfirmDialogProps {
  title: string;
  message: string;
  /** Onay düğmesinin metni; işlemi adlandırır ("Danışanı sil"). */
  confirmLabel: string;
  onConfirm(): void;
  onCancel(): void;
}

export function ConfirmDialog({ title, message, confirmLabel, onConfirm, onCancel }: ConfirmDialogProps) {
  const { t } = useTranslation();
  return (
    <Dialog title={title} onClose={onCancel}>
      <p>{message}</p>
      <div className="dialog-actions">
        <button type="button" data-autofocus onClick={onCancel}>
          {t("clients.cancel")}
        </button>
        <button type="button" className="button-danger" onClick={onConfirm}>
          {confirmLabel}
        </button>
      </div>
    </Dialog>
  );
}
```

`app/ui/shell/Shell.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/Shell.tsx
+++ b/app/ui/shell/Shell.tsx
@@ -1,5 +1,6 @@
 import { useEffect, useLayoutEffect, useRef, useState } from "react";
 import { BirthForm } from "../birth/BirthForm";
+import { Sidebar } from "../clients/Sidebar";
 import { Dashboard } from "../dashboard/Dashboard";
 import { useSession } from "../session/store";
 import { matchShortcut } from "../shortcuts";
@@ -18,7 +19,8 @@ export function Shell() {
   const [themePref, setThemePref] = useState<ThemePref>(() => loadThemePref());
   const [printView, setPrintView] = useState(false);
   const [help, setHelp] = useState(false);
-  const [form, setForm] = useState<"new" | "edit" | null>(null);
+  const [form, setForm] = useState<"new" | "add" | "edit" | null>(null);
+  const [sidebar, setSidebar] = useState(true);
   const { opened } = useSession();
   const measureRef = useRef<HTMLDivElement>(null);
   const width = useElementWidth(measureRef);
@@ -31,27 +33,35 @@ export function Shell() {
   };
 
   // Genel kısayollar (shortcuts.ts). Cmd+Shift+N notlar panelini etkin düzende açar/kapatır
-  // (spec §3.2); baskı görünümünde (pano yokken) yok sayılır. `?` yardımı açar.
+  // (spec §3.2); baskı görünümünde (pano yokken) yok sayılır. `?` yardımı açar. Dinleyici bir
+  // kez kurulur ve güncel durumu `latest`'ten okur: kayıttan hemen sonra basılan ⌘E, effect
+  // yeniden kurulmadan önce de açık haritayı görür.
+  const latest = useRef({ toggleNotes, printView, opened });
+  latest.current = { toggleNotes, printView, opened };
   useEffect(() => {
     const onKey = (e: KeyboardEvent) => {
       const command = matchShortcut(e);
-      if (command?.id === "toggleNotes" && !printView) {
+      const now = latest.current;
+      if (command?.id === "toggleNotes" && !now.printView) {
         e.preventDefault();
-        toggleNotes();
+        now.toggleNotes();
       } else if (command?.id === "help") {
         e.preventDefault();
         setHelp(true);
       } else if (command?.id === "newChart") {
         e.preventDefault();
         setForm("new");
-      } else if (command?.id === "editChart" && opened) {
+      } else if (command?.id === "editChart" && now.opened) {
         e.preventDefault();
         setForm("edit");
+      } else if (command?.id === "sidebar" && !now.printView) {
+        e.preventDefault();
+        setSidebar((v) => !v);
       }
     };
     window.addEventListener("keydown", onKey);
     return () => window.removeEventListener("keydown", onKey);
-  }, [toggleNotes, printView, opened]);
+  }, []);
 
   return (
     <div className="shell">
@@ -63,11 +73,16 @@ export function Shell() {
         onHelp={() => setHelp(true)}
         onNewChart={() => setForm("new")}
         onEditChart={opened ? () => setForm("edit") : undefined}
+        sidebarOpen={sidebar}
+        onToggleSidebar={() => setSidebar((v) => !v)}
       />
       <NoticeBar />
-      <main className="shell-main">
-        <div ref={measureRef}>{printView ? <PrintView /> : opened ? <Dashboard width={width} /> : <EmptyState onNew={() => setForm("new")} />}</div>
-      </main>
+      <div className="shell-body">
+        {sidebar && !printView && <Sidebar onAddChart={() => setForm("add")} />}
+        <main className="shell-main">
+          <div ref={measureRef}>{printView ? <PrintView /> : opened ? <Dashboard width={width} /> : <EmptyState onNew={() => setForm("new")} />}</div>
+        </main>
+      </div>
       {help && <HelpDialog onClose={() => setHelp(false)} />}
       {form && <BirthForm mode={form} onClose={() => setForm(null)} />}
     </div>
```

`app/ui/shell/TopBar.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/TopBar.tsx
+++ b/app/ui/shell/TopBar.tsx
@@ -1,6 +1,7 @@
 import { useTranslation } from "react-i18next";
 import { isLanguage, LANGUAGES, setLanguage, DEFAULT_LANGUAGE } from "../i18n";
 import { isThemePref, THEME_PREFS, type ThemePref } from "../theme/theme";
+import { SIDEBAR_ID } from "../clients/Sidebar";
 import { ChartTitle } from "./ChartTitle";
 import { LayoutPicker } from "./LayoutPicker";
 
@@ -13,14 +14,21 @@ export interface TopBarProps {
   onNewChart(): void;
   /** Açık harita yoksa verilmez (düğme gösterilmez). */
   onEditChart?: () => void;
+  sidebarOpen: boolean;
+  onToggleSidebar(): void;
 }
 
 /** Üst çubuk (spec §4): solda danışan ve harita (uyarı rozetleriyle), ortada düzen seçici, sağda tema, dil ve baskı. */
-export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onHelp, onNewChart, onEditChart }: TopBarProps) {
+export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onHelp, onNewChart, onEditChart, sidebarOpen, onToggleSidebar }: TopBarProps) {
   const { t, i18n } = useTranslation();
   const language = isLanguage(i18n.language) ? i18n.language : DEFAULT_LANGUAGE;
   return (
     <header className="topbar no-print">
+      {!printView && (
+        <button type="button" aria-expanded={sidebarOpen} aria-controls={SIDEBAR_ID} aria-keyshortcuts="Meta+Backslash" onClick={onToggleSidebar}>
+          {t("clients.toggle")}
+        </button>
+      )}
       <ChartTitle />
       <div className="topbar-actions">
         <button type="button" aria-keyshortcuts="Meta+N" onClick={onNewChart}>
```

`app/ui/shell/shell.css` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/shell.css
+++ b/app/ui/shell/shell.css
@@ -72,8 +72,14 @@
 .layout-error:empty {
   display: none;
 }
+.shell-body {
+  flex: 1;
+  display: flex;
+  min-height: 0;
+}
 .shell-main {
   flex: 1;
+  min-width: 0;
   padding: 8px;
   background: var(--bg);
 }
```

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app test
pnpm --dir app build
for i in 1 2 3; do pnpm --dir app test 2>&1 | grep -E "Tests "; done
```
Expected: `Test Files  38 passed (38)`, `Tests  319 passed (319)`; `build` `✓ built`; üç tekrarın üçü de `319 passed`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/App.test.tsx \
  app/ui/clients/ClientList.tsx \
  app/ui/clients/OpenClient.tsx \
  app/ui/clients/Sidebar.test.tsx \
  app/ui/clients/Sidebar.tsx \
  app/ui/clients/TagEditor.tsx \
  app/ui/clients/clients.css \
  app/ui/i18n/en.ts \
  app/ui/i18n/fr.ts \
  app/ui/i18n/tr.ts \
  app/ui/shell/ConfirmDialog.tsx \
  app/ui/shell/Shell.tsx \
  app/ui/shell/TopBar.tsx \
  app/ui/shell/shell.css
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git diff --cached --name-only | grep -iE 'k(a|ā|aa)la|\bcala\b|^\.superpowers/'   # boş olmalı
git commit -m "feat(ui): client sidebar with search, tag filter, recent clients, chart list, confirmed deletes and chart tags; Cmd+\ toggles it

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: commit oluşur; iki tarama da çıktısız. Commit mesajında yasaklı sözcük yoktur (C13).

---

### Task 8: Komut paleti (`Cmd+K`): danışan, harita, düzen, varga ve komut

Spec §4 ve UI spec §4: tek arama kutusu. Danışanlar Rust'ta aranır (boş sorguda son açılan 5, yazınca 120 ms sonra `search_clients` ile 8; eski yanıt sıra bekçisiyle atılır); harita (açık danışanın), düzen, varga (etkin düzende harita paneli varsa) ve komutlar arayüzün kendi listeleridir ve büyük/küçük harf ve aksan duyarsız metin karşılaştırmasıyla süzülür (`foldText`: NFD + işaret silme + yerel küçük harf + `ı→i`; hesap değil). WAI-ARIA combobox + listbox: `aria-activedescendant`, ↑/↓ sarar, Enter çalıştırır, Esc kapatır; etkin öğe zemin rengi **ve** 2 px çizgiyle ayrılır (C9) ve ok tuşlarıyla her değiştiğinde `scrollIntoView({ block: "nearest" })` ile görünür alana getirilir (liste kaydırılabilir; Fable MF2). Spec §8: danışan seçimi doğrudan açar, form ya da onay yoktur. Sekme olmadığından `Cmd+Enter` danışanı açar ve kenar çubuğunu gösterir (D4). Üst çubukta "Ara" düğmesi (`aria-keyshortcuts="Meta+K"`).

**Files:**
- Test: `app/ui/App.test.tsx`
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Modify: `app/ui/i18n/tr.ts`
- Test (yeni): `app/ui/palette/Palette.test.tsx`
- Create: `app/ui/palette/Palette.tsx`
- Test (yeni): `app/ui/palette/items.test.ts`
- Create: `app/ui/palette/items.ts`
- Create: `app/ui/palette/palette.css`
- Modify: `app/ui/shell/Shell.tsx`
- Modify: `app/ui/shell/TopBar.tsx`

**Interfaces:**
- Consumes: `api.listClients/searchClients/listCharts`, `Session.openClient` (Task 6), `useLayoutStore().select/setPanelSettings/active/state.layouts`, `VARGAS`, `VARGA_NAMES`, `layoutLabel`, `Dialog`.
- Produces: `palette/items` `PaletteCommand = "newChart" | "addChart" | "editChart" | "toggleNotes" | "sidebar" | "help"`, `PaletteItem` (birleşim: `client {clientId}`, `chart {clientId, chartId}`, `layout {layoutId}`, `varga {varga}`, `command {command}`; hepsinde `key`, `label`), `foldText(text, locale)`, `matches(label, query, locale)`, `clientItems(hits, detail)`, `localItems(src: LocalSources, query, locale)`; `palette/Palette` `Palette({ onClose(), onRun(item: PaletteItem, meta: boolean) })`, `PALETTE_SEARCH_DEBOUNCE_MS = 120`, `PALETTE_RECENT = 5`, `PALETTE_CLIENTS = 8`; `TopBarProps.onPalette(): void`; i18n `palette.*`.

- [ ] **Step 1: Önce testleri yaz**

`app/ui/App.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -13,6 +13,8 @@ vi.mock("./api", () => ({
     listClients: vi.fn(async () => ({ status: "ok", data: [] })),
     listTags: vi.fn(async () => ({ status: "ok", data: [] })),
     listCharts: vi.fn(async () => ({ status: "ok", data: [] })),
+    searchClients: vi.fn(async () => ({ status: "ok", data: [] })),
+    openClient: vi.fn(),
   },
 }));
 
@@ -182,6 +184,42 @@ test("Cmd+\\ and the Clients button show and hide the client sidebar", () => {
   expect(screen.getByRole("complementary", { name: "Danışanlar" })).toBeInTheDocument();
 });
 
+test("Cmd+K, a name and Enter open that client's last chart with no form in between (spec §8)", async () => {
+  const other = fixture().opened;
+  other.client = { ...other.client, id: 2, name: "Ayşe Yılmaz" };
+  other.chart_record = { ...other.chart_record, id: 20, client_id: 2 };
+  vi.mocked(api.searchClients).mockResolvedValueOnce({ status: "ok", data: [{ client: other.client, matched: "Name", snippet: null }] });
+  vi.mocked(api.openClient).mockResolvedValueOnce({ status: "ok", data: other });
+  renderApp();
+  fireEvent.keyDown(window, { key: "\\", code: "Backslash", metaKey: true });
+  expect(screen.queryByRole("complementary", { name: "Danışanlar" })).toBeNull();
+  fireEvent.keyDown(window, { key: "k", metaKey: true });
+  const palette = screen.getByRole("dialog", { name: "Komut paleti" });
+  fireEvent.change(within(palette).getByRole("combobox"), { target: { value: "ayşe" } });
+  await within(palette).findByRole("option", { name: /Ayşe Yılmaz/ }, { timeout: 3000 });
+  fireEvent.keyDown(within(palette).getByRole("combobox"), { key: "Enter", metaKey: true });
+  expect(await screen.findByRole("heading", { level: 1, name: "Ayşe Yılmaz" })).toBeInTheDocument();
+  expect(api.openClient).toHaveBeenCalledWith(2, null);
+  expect(screen.queryByRole("dialog")).toBeNull();
+  // Cmd+Enter kenar çubuğunu da gösterir (sekme yoktur).
+  expect(screen.getByRole("complementary", { name: "Danışanlar" })).toBeInTheDocument();
+});
+
+test("the palette's varga item sets the first chart panel and its layout item switches layouts", async () => {
+  renderApp();
+  fireEvent.click(screen.getByRole("button", { name: "Ara" }));
+  let palette = screen.getByRole("dialog", { name: "Komut paleti" });
+  fireEvent.change(within(palette).getByRole("combobox"), { target: { value: "saptamsa" } });
+  fireEvent.click(await within(palette).findByRole("option", { name: /D7 Saptamsa/ }));
+  expect(screen.getByRole("group", { name: "D7 haritası" })).toBeInTheDocument();
+  expect(screen.queryByRole("group", { name: "D1 haritası" })).toBeNull();
+  fireEvent.keyDown(window, { key: "k", metaKey: true });
+  palette = screen.getByRole("dialog", { name: "Komut paleti" });
+  fireEvent.change(within(palette).getByRole("combobox"), { target: { value: "araştırma" } });
+  fireEvent.click(await within(palette).findByRole("option", { name: /Araştırma/ }));
+  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Araştırma");
+});
+
 test("theme menu sets data-theme and remembers the choice", () => {
   renderApp();
   const select = screen.getByRole("combobox", { name: "Tema" });
```

`app/ui/palette/Palette.test.tsx` oluştur:

```tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    listClients: vi.fn(),
    searchClients: vi.fn(),
    listCharts: vi.fn(),
  },
}));

import { api } from "../api";
import type { Client, OpenedClient } from "../bindings";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { foldText, matches } from "./items";
import { Palette, PALETTE_CLIENTS, PALETTE_RECENT, PALETTE_SEARCH_DEBOUNCE_MS } from "./Palette";

const ok = <T,>(data: T) => ({ status: "ok" as const, data });
const opened = () => fixture().opened;
const client = (id: number, name: string): Client => ({ ...opened().client, id, name });
const onClose = vi.fn();
const onRun = vi.fn();

function renderPalette(o: OpenedClient | null = opened()) {
  return renderWithStores(<Palette onClose={onClose} onRun={onRun} />, { opened: o });
}

const box = () => screen.getByRole("combobox", { name: "Danışan, harita, varga, düzen ya da komut ara" });
const options = () => within(screen.getByRole("listbox", { name: "Sonuçlar" })).getAllByRole("option");

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  onClose.mockReset();
  onRun.mockReset();
  vi.mocked(api.listClients).mockResolvedValue(ok([opened().client, client(2, "Ayşe Yılmaz")]));
  vi.mocked(api.listCharts).mockResolvedValue(ok([opened().chart_record]));
});

test("opens on the search box with recent clients first, then commands, charts, layouts and vargas", async () => {
  renderPalette();
  expect(document.activeElement).toBe(box());
  await waitFor(() => expect(options()[0]).toHaveTextContent("DanışanÖrnek Danışan"));
  expect(api.listClients).toHaveBeenCalledWith(PALETTE_RECENT);
  const kinds = options().map((o) => o.firstChild?.textContent);
  expect(kinds.slice(0, 3)).toEqual(["Danışan", "Danışan", "Komut"]);
  expect(kinds).toContain("Harita");
  expect(kinds).toContain("Düzen");
  expect(kinds).toContain("Varga");
  expect(options()[0]).toHaveAttribute("aria-selected", "true");
  expect(box()).toHaveAttribute("aria-activedescendant", options()[0].id);
});

test("typing searches clients after the debounce and filters the local items accent-blind", async () => {
  vi.useFakeTimers();
  vi.mocked(api.searchClients).mockResolvedValue(ok([{ client: client(2, "Ayşe Yılmaz"), matched: "Name", snippet: null }]));
  renderPalette();
  fireEvent.change(box(), { target: { value: "navāmsa" } });
  expect(api.searchClients).not.toHaveBeenCalled();
  await act(async () => vi.advanceTimersByTime(PALETTE_SEARCH_DEBOUNCE_MS));
  vi.useRealTimers();
  expect(api.searchClients).toHaveBeenCalledWith("navāmsa", PALETTE_CLIENTS);
  await waitFor(() => expect(options().map((o) => o.textContent)).toEqual(["DanışanAyşe Yılmaz", "VargaD9 Navamsa"]));
  expect(matches("Navāmsa", "NAVAMSA", "tr")).toBe(true);
  expect(foldText("IŞIK", "tr")).toBe("isik");
});

test("arrow keys move the active option and wrap; Enter closes and runs it", async () => {
  renderPalette();
  await waitFor(() => expect(options()[1]).toHaveTextContent("Ayşe Yılmaz"));
  fireEvent.keyDown(box(), { key: "ArrowUp" });
  expect(options().at(-1)).toHaveAttribute("aria-selected", "true");
  fireEvent.keyDown(box(), { key: "ArrowDown" });
  fireEvent.keyDown(box(), { key: "ArrowDown" });
  expect(options()[1]).toHaveAttribute("aria-selected", "true");
  fireEvent.keyDown(box(), { key: "Enter" });
  expect(onClose).toHaveBeenCalledOnce();
  expect(onRun).toHaveBeenCalledWith(expect.objectContaining({ kind: "client", clientId: 2 }), false);
});

test("the active option is scrolled into view as the arrows move it", async () => {
  const scroll = vi.spyOn(Element.prototype, "scrollIntoView");
  renderPalette();
  await waitFor(() => expect(options()[1]).toHaveTextContent("Ayşe Yılmaz"));
  fireEvent.keyDown(box(), { key: "ArrowUp" });
  expect(scroll.mock.contexts.at(-1)).toBe(options().at(-1));
  expect(scroll).toHaveBeenLastCalledWith({ block: "nearest" });
  fireEvent.keyDown(box(), { key: "ArrowDown" });
  expect(scroll.mock.contexts.at(-1)).toBe(options()[0]);
  scroll.mockRestore();
});

test("Cmd+Enter on a client is passed through as meta", async () => {
  renderPalette();
  await waitFor(() => expect(options()[1]).toHaveTextContent("Ayşe Yılmaz"));
  fireEvent.keyDown(box(), { key: "ArrowDown" });
  fireEvent.keyDown(box(), { key: "Enter", metaKey: true });
  expect(onRun).toHaveBeenCalledWith(expect.objectContaining({ kind: "client", clientId: 2 }), true);
});

test("without an open client there are no charts, vargas or client-only commands", async () => {
  renderPalette(null);
  await waitFor(() => expect(options()[0]).toHaveTextContent("Örnek Danışan"));
  const labels = options().map((o) => o.textContent ?? "");
  expect(labels.some((l) => l.startsWith("Harita") || l.startsWith("Varga"))).toBe(false);
  expect(labels).not.toContain("KomutAçık haritayı düzenle");
  expect(labels).toContain("KomutYeni danışan ve harita");
  expect(api.listCharts).not.toHaveBeenCalled();
});

test("nothing matching shows the empty text and Enter does nothing", async () => {
  vi.mocked(api.searchClients).mockResolvedValue(ok([]));
  renderPalette();
  fireEvent.change(box(), { target: { value: "zzzz" } });
  expect(await screen.findByText("Eşleşen öğe yok.")).toBeInTheDocument();
  fireEvent.keyDown(box(), { key: "Enter" });
  expect(onRun).not.toHaveBeenCalled();
  expect(box()).not.toHaveAttribute("aria-activedescendant");
});
```

`app/ui/palette/items.test.ts` oluştur:

```ts
import { fixture } from "../test/fixture";
import { builtinLayouts } from "../layout/model";
import { foldText, localItems, matches, type LocalSources } from "./items";

test("text matching ignores case and marks and needs every word", () => {
  expect(foldText("Navāmsa İŞ", "tr")).toBe("navamsa is");
  expect(matches("D9 Navamsa", "navām", "tr")).toBe(true);
  expect(matches("Yeni harita", "har yen", "tr")).toBe(true);
  expect(matches("Yeni harita", "harita düzen", "tr")).toBe(false);
  expect(matches("Araştırma", "ARAŞ", "tr")).toBe(true);
  expect(matches("Anything", "  ", "tr")).toBe(true);
});

test("local items keep command, chart, layout, varga order and are filtered by the query", () => {
  const chart = fixture().opened.chart_record;
  const src: LocalSources = {
    charts: [chart],
    chartLabel: () => "Doğum haritası",
    layouts: builtinLayouts(),
    layoutLabel: (l) => l.id,
    vargas: [{ varga: "D9", label: "D9 Navamsa" }],
    commands: [{ command: "newChart", label: "Yeni harita" }],
  };
  expect(localItems(src, "", "tr").map((i) => i.kind)).toEqual(["command", "chart", "layout", "layout", "layout", "varga"]);
  expect(localItems(src, "harita", "tr").map((i) => i.key)).toEqual(["command:newChart", `chart:${chart.id}`]);
  expect(localItems(src, "navamsa", "tr")).toEqual([{ kind: "varga", key: "varga:D9", label: "D9 Navamsa", varga: "D9" }]);
});
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/palette ui/App.test.tsx
```
Expected: `palette/items.test.ts` ve `palette/Palette.test.tsx` `Failed to resolve import "./items"` / `"./Palette"` ile FAIL; App'in iki palet testi `Komut paleti` penceresi bulunamadığı için FAIL.

- [ ] **Step 3: Uygulamayı yaz**

`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -293,6 +293,31 @@ export const en: Messages = {
     tags: "Tags",
     tagsHint: "Separate with commas; Enter saves.",
   },
+  palette: {
+    title: "Command palette",
+    open: "Search",
+    search: "Search clients, charts, vargas, layouts or commands",
+    placeholder: "Client, chart, varga, layout, command…",
+    hint: "↑ ↓ select, Enter opens, ⌘Enter opens the client and shows the sidebar, Esc closes.",
+    results: "Results",
+    empty: "Nothing matches.",
+    count: "{{n}} items",
+    kind: {
+      client: "Client",
+      chart: "Chart",
+      layout: "Layout",
+      varga: "Varga",
+      command: "Command",
+    },
+    command: {
+      newChart: "New client and chart",
+      addChart: "Add a chart to the open client",
+      editChart: "Edit the open chart",
+      toggleNotes: "Show or hide the notes panel",
+      sidebar: "Show or hide the client sidebar",
+      help: "Keyboard shortcuts",
+    },
+  },
   empty: {
     title: "No chart is open",
     body: "Start by entering the client's birth data.",
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -293,6 +293,31 @@ export const fr: Messages = {
     tags: "Étiquettes",
     tagsHint: "Séparez par des virgules ; Entrée enregistre.",
   },
+  palette: {
+    title: "Palette de commandes",
+    open: "Rechercher",
+    search: "Rechercher clients, cartes, vargas, dispositions ou commandes",
+    placeholder: "Client, carte, varga, disposition, commande…",
+    hint: "↑ ↓ sélectionne, Entrée ouvre, ⌘Entrée ouvre le client et affiche la barre, Échap ferme.",
+    results: "Résultats",
+    empty: "Aucune correspondance.",
+    count: "{{n}} éléments",
+    kind: {
+      client: "Client",
+      chart: "Carte",
+      layout: "Disposition",
+      varga: "Varga",
+      command: "Commande",
+    },
+    command: {
+      newChart: "Nouveau client et carte",
+      addChart: "Ajouter une carte au client ouvert",
+      editChart: "Modifier la carte ouverte",
+      toggleNotes: "Afficher ou masquer le panneau de notes",
+      sidebar: "Afficher ou masquer la barre des clients",
+      help: "Raccourcis clavier",
+    },
+  },
   empty: {
     title: "Aucune carte ouverte",
     body: "Commencez par saisir les données de naissance du consultant.",
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -292,6 +292,31 @@ export const tr = {
     tags: "Etiketler",
     tagsHint: "Virgülle ayırın; Enter kaydeder.",
   },
+  palette: {
+    title: "Komut paleti",
+    open: "Ara",
+    search: "Danışan, harita, varga, düzen ya da komut ara",
+    placeholder: "Danışan, harita, varga, düzen, komut…",
+    hint: "↑ ↓ seçer, Enter açar, ⌘Enter danışanı açıp kenar çubuğunu gösterir, Esc kapatır.",
+    results: "Sonuçlar",
+    empty: "Eşleşen öğe yok.",
+    count: "{{n}} öğe",
+    kind: {
+      client: "Danışan",
+      chart: "Harita",
+      layout: "Düzen",
+      varga: "Varga",
+      command: "Komut",
+    },
+    command: {
+      newChart: "Yeni danışan ve harita",
+      addChart: "Açık danışana harita ekle",
+      editChart: "Açık haritayı düzenle",
+      toggleNotes: "Notlar panelini aç/kapat",
+      sidebar: "Danışan kenar çubuğunu aç/kapat",
+      help: "Klavye kısayolları",
+    },
+  },
   empty: {
     title: "Açık harita yok",
     body: "Danışanın doğum bilgisini girerek başlayın.",
```

`app/ui/palette/Palette.tsx` oluştur:

```tsx
// Komut paleti (`Cmd+K`, spec §4 ve UI spec §4): danışan, açık danışanın haritaları, varga,
// düzen ve komut tek arama kutusunda. WAI-ARIA combobox + listbox; ↑/↓ etkin öğeyi taşır,
// Enter çalıştırır, Esc kapatır. Danışanda `Cmd+Enter` danışanı açar ve kenar çubuğunu
// gösterir (sekme yoktur, UI spec §4). Danışan araması `search_clients` iledir; eski yanıt
// yok sayılır. Seçimden haritaya form ya da onay penceresi çıkmaz (spec §8).
import { useEffect, useId, useMemo, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { ChartRecord, SearchHit } from "../bindings";
import { VARGA_NAMES, VARGAS } from "../chart/adapter";
import { useSession } from "../session/store";
import { Dialog } from "../shell/Dialog";
import { layoutLabel } from "../shell/LayoutPicker";
import { useLayoutStore } from "../view/store";
import { clientItems, localItems, type PaletteCommand, type PaletteItem } from "./items";
import "./palette.css";

export const PALETTE_SEARCH_DEBOUNCE_MS = 120;
/** Boş sorguda gösterilen son açılan danışan sayısı; aramada en çok danışan. */
export const PALETTE_RECENT = 5;
export const PALETTE_CLIENTS = 8;

export interface PaletteProps {
  onClose(): void;
  /** Öğe seçildi; `meta`: Cmd+Enter. Palet önce kapanır, sonra çağrılır. */
  onRun(item: PaletteItem, meta: boolean): void;
}

export function Palette({ onClose, onRun }: PaletteProps) {
  const { t, i18n } = useTranslation();
  const { opened, revision } = useSession();
  const layout = useLayoutStore();
  const id = useId();
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[]>([]);
  const [charts, setCharts] = useState<ChartRecord[]>([]);
  const [active, setActive] = useState(0);
  const seq = useRef(0);
  const listRef = useRef<HTMLUListElement>(null);

  useEffect(() => {
    if (!opened) {
      setCharts([]);
      return;
    }
    let live = true;
    void api.listCharts(opened.client.id).then((r) => live && r.status === "ok" && setCharts(r.data));
    return () => {
      live = false;
    };
  }, [opened, revision]);

  useEffect(() => {
    const mine = ++seq.current;
    const q = query.trim();
    const timer = setTimeout(
      () => {
        const call =
          q === ""
            ? api.listClients(PALETTE_RECENT).then((r) => (r.status === "ok" ? r.data.map((client): SearchHit => ({ client, matched: "Name", snippet: null })) : []))
            : api.searchClients(q, PALETTE_CLIENTS).then((r) => (r.status === "ok" ? r.data : []));
        void call.then((data) => {
          if (mine === seq.current) setHits(data);
        });
      },
      q === "" ? 0 : PALETTE_SEARCH_DEBOUNCE_MS,
    );
    return () => clearTimeout(timer);
  }, [query]);

  const hasChartPanel = layout.active.panels.some((p) => p.kind === "chart");
  const items = useMemo(() => {
    const commands: { command: PaletteCommand; label: string }[] = [
      { command: "newChart", label: t("palette.command.newChart") },
      ...(opened
        ? [
            { command: "addChart" as const, label: t("palette.command.addChart") },
            { command: "editChart" as const, label: t("palette.command.editChart") },
            { command: "toggleNotes" as const, label: t("palette.command.toggleNotes") },
          ]
        : []),
      { command: "sidebar", label: t("palette.command.sidebar") },
      { command: "help", label: t("palette.command.help") },
    ];
    const clients = clientItems(hits, (h) => (h.matched === "Note" && h.snippet ? t("clients.matchedNote", { snippet: h.snippet }) : h.matched === "Tag" ? t("clients.matchedTag") : null));
    const local = localItems(
      {
        charts: opened ? charts : [],
        chartLabel: (c) => c.label || t(`chart.kind.${c.kind}`),
        layouts: layout.state.layouts,
        layoutLabel: (l) => layoutLabel(t, l),
        vargas: opened && hasChartPanel ? VARGAS.map((v) => ({ varga: v, label: `${v} ${VARGA_NAMES[v]}` })) : [],
        commands,
      },
      query,
      i18n.language,
    );
    return [...clients, ...local];
  }, [hits, charts, opened, layout.state.layouts, hasChartPanel, query, t, i18n.language]);

  useEffect(() => setActive(0), [query]);
  const current = Math.min(active, items.length - 1);

  // Etkin seçenek (aria-activedescendant) kaydırılan listede görünür tutulur (PlaceSearch gibi).
  useEffect(() => {
    listRef.current?.children[current]?.scrollIntoView({ block: "nearest" });
  }, [current, items]);

  const run = (item: PaletteItem, meta: boolean) => {
    onClose();
    onRun(item, meta);
  };

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.nativeEvent.isComposing) return;
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      if (items.length === 0) return;
      const step = e.key === "ArrowDown" ? 1 : -1;
      setActive((current + step + items.length) % items.length);
    } else if (e.key === "Enter") {
      e.preventDefault();
      const item = items[current];
      if (item) run(item, e.metaKey);
    }
  };

  const kindLabel = (item: PaletteItem) => t(`palette.kind.${item.kind}`);
  const optionId = (i: number) => `${id}-opt-${i}`;
  return (
    <Dialog title={t("palette.title")} onClose={onClose}>
      <div className="palette">
        <label htmlFor={`${id}-q`} className="visually-hidden">
          {t("palette.search")}
        </label>
        <input
          id={`${id}-q`}
          data-autofocus
          role="combobox"
          aria-expanded="true"
          aria-controls={`${id}-list`}
          aria-autocomplete="list"
          aria-activedescendant={items.length > 0 ? optionId(current) : undefined}
          aria-describedby={`${id}-hint`}
          placeholder={t("palette.placeholder")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={onKeyDown}
        />
        <p id={`${id}-hint`} className="palette-hint">
          {t("palette.hint")}
        </p>
        <ul ref={listRef} id={`${id}-list`} role="listbox" aria-label={t("palette.results")} className="palette-list">
          {items.map((item, i) => (
            <li
              key={item.key}
              id={optionId(i)}
              role="option"
              aria-selected={i === current}
              className={i === current ? "is-active" : undefined}
              onMouseDown={(e) => e.preventDefault()}
              onClick={(e) => run(item, e.metaKey)}
            >
              <span className="palette-kind">{kindLabel(item)}</span>
              <span className="palette-label">{item.label}</span>
              {"detail" in item && item.detail && <span className="palette-detail">{item.detail}</span>}
            </li>
          ))}
        </ul>
        {items.length === 0 && <p className="palette-empty">{t("palette.empty")}</p>}
        <p className="visually-hidden" role="status">
          {t("palette.count", { n: items.length })}
        </p>
      </div>
    </Dialog>
  );
}
```

`app/ui/palette/items.ts` oluştur:

```ts
// Komut paletinin öğeleri (UI spec §4: danışan, harita, varga, düzen ve komut tek arama
// kutusunda). Danışanlar Rust'ta aranır (`search_clients`); diğer öğeler arayüzün kendi
// listeleridir ve burada metinle süzülür. Süzme bir dizgi karşılaştırmasıdır, hesap değildir.
import type { ChartRecord, SearchHit, Varga } from "../bindings";
import type { Layout } from "../layout/model";

export type PaletteCommand = "newChart" | "addChart" | "editChart" | "toggleNotes" | "sidebar" | "help";

export type PaletteItem =
  | { kind: "client"; key: string; label: string; detail: string | null; clientId: number }
  | { kind: "chart"; key: string; label: string; detail: string; clientId: number; chartId: number }
  | { kind: "layout"; key: string; label: string; layoutId: string }
  | { kind: "varga"; key: string; label: string; varga: Varga }
  | { kind: "command"; key: string; label: string; command: PaletteCommand };

/** Büyük/küçük harf ve aksan duyarsız karşılaştırma anahtarı ("Navāmsa" → "navamsa"). */
export function foldText(text: string, locale: string): string {
  return text.normalize("NFD").replace(/\p{M}/gu, "").toLocaleLowerCase(locale).replace(/ı/g, "i");
}

/** Sorgunun her kelimesi öğe adında geçiyorsa eşleşir; boş sorgu her şeyle eşleşir. */
export function matches(label: string, query: string, locale: string): boolean {
  const words = foldText(query, locale).split(/\s+/).filter(Boolean);
  const hay = foldText(label, locale);
  return words.every((w) => hay.includes(w));
}

export function clientItems(hits: readonly SearchHit[], detail: (hit: SearchHit) => string | null): PaletteItem[] {
  return hits.map((hit) => ({ kind: "client", key: `client:${hit.client.id}`, label: hit.client.name, detail: detail(hit), clientId: hit.client.id }));
}

export interface LocalSources {
  charts: readonly ChartRecord[];
  chartLabel(chart: ChartRecord): string;
  layouts: readonly Layout[];
  layoutLabel(layout: Layout): string;
  /** Etkin düzende harita paneli yoksa varga öğesi gösterilmez. */
  vargas: readonly { varga: Varga; label: string }[];
  commands: readonly { command: PaletteCommand; label: string }[];
}

/** Arayüzün kendi öğeleri, sorguyla süzülmüş: komutlar, haritalar, düzenler, vargalar. */
export function localItems(src: LocalSources, query: string, locale: string): PaletteItem[] {
  const out: PaletteItem[] = [
    ...src.commands.map((c): PaletteItem => ({ kind: "command", key: `command:${c.command}`, label: c.label, command: c.command })),
    ...src.charts.map(
      (c): PaletteItem => ({ kind: "chart", key: `chart:${c.id}`, label: src.chartLabel(c), detail: c.birth.date, clientId: c.client_id, chartId: c.id }),
    ),
    ...src.layouts.map((l): PaletteItem => ({ kind: "layout", key: `layout:${l.id}`, label: src.layoutLabel(l), layoutId: l.id })),
    ...src.vargas.map((v): PaletteItem => ({ kind: "varga", key: `varga:${v.varga}`, label: v.label, varga: v.varga })),
  ];
  return out.filter((i) => matches(i.label, query, locale));
}
```

`app/ui/palette/palette.css` oluştur:

```css
.palette {
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: min(560px, 90vw);
}
.palette input {
  font-size: var(--fs-panel-title);
  padding: 6px 8px;
}
.palette-hint,
.palette-empty {
  margin: 0;
  color: var(--text-2);
  font-size: var(--fs-table);
}
.palette-list {
  margin: 0;
  padding: 2px;
  list-style: none;
  max-height: 50vh;
  overflow: auto;
}
.palette-list li {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding: 4px 6px;
  border-radius: 4px;
  cursor: default;
}
/* Etkin öğe yalnızca renkle ayrılmaz: zemin + odak halkasıyla aynı çizgi. */
.palette-list li.is-active {
  background: var(--select);
  color: var(--ink-strong);
  outline: 2px solid var(--ink);
  outline-offset: -2px;
}
.palette-kind {
  flex: 0 0 72px;
  color: var(--text-2);
  font-size: var(--fs-table);
}
.palette-label {
  flex: 1;
}
.palette-detail {
  color: var(--text-2);
  font-size: var(--fs-table);
}
```

`app/ui/shell/Shell.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/Shell.tsx
+++ b/app/ui/shell/Shell.tsx
@@ -1,6 +1,8 @@
 import { useEffect, useLayoutEffect, useRef, useState } from "react";
 import { BirthForm } from "../birth/BirthForm";
 import { Sidebar } from "../clients/Sidebar";
+import type { PaletteItem } from "../palette/items";
+import { Palette } from "../palette/Palette";
 import { Dashboard } from "../dashboard/Dashboard";
 import { useSession } from "../session/store";
 import { matchShortcut } from "../shortcuts";
@@ -8,20 +10,24 @@ import { useLayoutStore } from "../view/store";
 import { applyTheme, loadThemePref, saveThemePref, type ThemePref } from "../theme/theme";
 import { EmptyState } from "./EmptyState";
 import { HelpDialog } from "./HelpDialog";
-import { NoticeBar } from "./notices";
+import { NoticeBar, useNotices } from "./notices";
 import { PrintView } from "./PrintView";
 import "./shell.css";
 import { TopBar } from "./TopBar";
 import { useElementWidth } from "./useElementWidth";
 
 export function Shell() {
-  const { toggleNotes } = useLayoutStore();
+  const layout = useLayoutStore();
+  const { toggleNotes } = layout;
   const [themePref, setThemePref] = useState<ThemePref>(() => loadThemePref());
   const [printView, setPrintView] = useState(false);
   const [help, setHelp] = useState(false);
   const [form, setForm] = useState<"new" | "add" | "edit" | null>(null);
   const [sidebar, setSidebar] = useState(true);
-  const { opened } = useSession();
+  const [palette, setPalette] = useState(false);
+  const session = useSession();
+  const { opened } = session;
+  const { push } = useNotices();
   const measureRef = useRef<HTMLDivElement>(null);
   const width = useElementWidth(measureRef);
 
@@ -57,12 +63,42 @@ export function Shell() {
       } else if (command?.id === "sidebar" && !now.printView) {
         e.preventDefault();
         setSidebar((v) => !v);
+      } else if (command?.id === "palette") {
+        e.preventDefault();
+        setPalette(true);
       }
     };
     window.addEventListener("keydown", onKey);
     return () => window.removeEventListener("keydown", onKey);
   }, []);
 
+  const openClient = (clientId: number, chartId: number | null = null) =>
+    void session.openClient(clientId, chartId).then((error) => error && push({ id: "client", type: "error", messageKey: "notice.openFailed", error }));
+
+  // Paletten seçilen öğe. Danışan seçimi doğrudan açar (form ya da onay yok, spec §8).
+  const runPalette = (item: PaletteItem, meta: boolean) => {
+    switch (item.kind) {
+      case "client":
+        if (meta) setSidebar(true);
+        return openClient(item.clientId);
+      case "chart":
+        return openClient(item.clientId, item.chartId);
+      case "layout":
+        return layout.select(item.layoutId);
+      case "varga": {
+        const panel = layout.active.panels.find((p) => p.kind === "chart");
+        return panel && layout.setPanelSettings(panel.id, { varga: item.varga });
+      }
+      case "command":
+        if (item.command === "newChart" || item.command === "addChart" || item.command === "editChart") {
+          return setForm(item.command === "newChart" ? "new" : item.command === "addChart" ? "add" : "edit");
+        }
+        if (item.command === "toggleNotes") return toggleNotes();
+        if (item.command === "sidebar") return setSidebar((v) => !v);
+        return setHelp(true);
+    }
+  };
+
   return (
     <div className="shell">
       <TopBar
@@ -75,6 +111,7 @@ export function Shell() {
         onEditChart={opened ? () => setForm("edit") : undefined}
         sidebarOpen={sidebar}
         onToggleSidebar={() => setSidebar((v) => !v)}
+        onPalette={() => setPalette(true)}
       />
       <NoticeBar />
       <div className="shell-body">
@@ -84,6 +121,7 @@ export function Shell() {
         </main>
       </div>
       {help && <HelpDialog onClose={() => setHelp(false)} />}
+      {palette && <Palette onClose={() => setPalette(false)} onRun={runPalette} />}
       {form && <BirthForm mode={form} onClose={() => setForm(null)} />}
     </div>
   );
```

`app/ui/shell/TopBar.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/TopBar.tsx
+++ b/app/ui/shell/TopBar.tsx
@@ -16,10 +16,11 @@ export interface TopBarProps {
   onEditChart?: () => void;
   sidebarOpen: boolean;
   onToggleSidebar(): void;
+  onPalette(): void;
 }
 
 /** Üst çubuk (spec §4): solda danışan ve harita (uyarı rozetleriyle), ortada düzen seçici, sağda tema, dil ve baskı. */
-export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onHelp, onNewChart, onEditChart, sidebarOpen, onToggleSidebar }: TopBarProps) {
+export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onHelp, onNewChart, onEditChart, sidebarOpen, onToggleSidebar, onPalette }: TopBarProps) {
   const { t, i18n } = useTranslation();
   const language = isLanguage(i18n.language) ? i18n.language : DEFAULT_LANGUAGE;
   return (
@@ -42,6 +43,9 @@ export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onH
       </div>
       <div className="topbar-center">{!printView && <LayoutPicker />}</div>
       <div className="topbar-right">
+        <button type="button" aria-keyshortcuts="Meta+K" onClick={onPalette}>
+          {t("palette.open")}
+        </button>
         <label>
           {t("theme.label")}
           <select value={themePref} onChange={(e) => isThemePref(e.target.value) && onThemeChange(e.target.value)}>
```

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app test
pnpm --dir app build
```
Expected: `Test Files  40 passed (40)`, `Tests  330 passed (330)`; `build` `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/App.test.tsx \
  app/ui/i18n/en.ts \
  app/ui/i18n/fr.ts \
  app/ui/i18n/tr.ts \
  app/ui/palette/Palette.test.tsx \
  app/ui/palette/Palette.tsx \
  app/ui/palette/items.test.ts \
  app/ui/palette/items.ts \
  app/ui/palette/palette.css \
  app/ui/shell/Shell.tsx \
  app/ui/shell/TopBar.tsx
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git diff --cached --name-only | grep -iE 'k(a|ā|aa)la|\bcala\b|^\.superpowers/'   # boş olmalı
git commit -m "feat(ui): Cmd+K command palette for clients, charts, layouts, vargas and commands

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: commit oluşur; iki tarama da çıktısız. Commit mesajında yasaklı sözcük yoktur (C13).

---

### Task 9: Notlar paneli: sıralı otomatik kayıt (en uzun bekleme, sayfa gizlenince kayıt), onaylı silme ve not gövdesi sınırı

Spec §3.3 ve §4 notlar: tarih damgalı seans notları, en yeni önce. Panel `list_notes(clientId, 100)` ile okur; `open_client`'ın `recent_notes` alanı kullanılmaz, çünkü bu oturumdaki düzenlemelerden sonra eskir (D9). "Yeni not" açık haritaya bağlı boş bir not ekler ve odaklar. Metin yazım durunca 500 ms sonra, durmadan yazılsa da ilk kirli değişiklikten en geç 2 s sonra (`NOTE_SAVE_MAX_WAIT_MS`), odak çıkınca, panel kapanınca ve sayfa gizlenince (`visibilitychange` hidden, `pagehide`: uygulamadan çıkış) hemen kaydedilir (C17, Fable MF3); bütün not kayıtları tek sırada gider (`enqueueNoteSave`) ve liste bekleyen kayıtlar bitmeden okunmaz (`notesIdle`), böylece kapanırken gönderilen metin yeniden açılan panelde görünür. Başarısız kayıt notun altında metinle gösterilir (not metni hata iletisine girmez, C1) ve sonraki yazım ya da odak çıkışı yeniden dener; panel kapandıktan sonra başarısız olan kayıt bildirim bandında `notice.noteSaveLost` ile duyurulur (yine not metni olmadan). Silme `ConfirmDialog` ile; silinen notun bekleyen kaydı gönderilmez. Rust tarafında not gövdesi `MAX_NOTE_BYTES = 65 536` bayt ile sınırlanır (`invalid_input{field: "body"}`); arayüz `maxLength = 16 384` UTF-16 birimiyle daha önce durur (16 384 × 3 bayt < 65 536).

**Files:**
- Test: `app/ui/App.test.tsx`
- Test: `app/ui/dashboard/Dashboard.test.tsx`
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Modify: `app/ui/i18n/tr.ts`
- Create: `app/ui/notes/NoteEditor.tsx`
- Test (yeni): `app/ui/notes/NotesPanel.test.tsx`
- Create: `app/ui/notes/NotesPanel.tsx`
- Create: `app/ui/notes/notes.css`
- Create: `app/ui/notes/saves.ts`
- Test: `app/ui/panels/panels.test.tsx`
- Modify: `app/ui/panels/registry.ts`
- Modify: `crates/store/src/lib.rs`
- Modify: `crates/store/src/notes.rs`

**Interfaces:**
- Consumes: `api.listNotes/addNote/updateNote/deleteNote` (Task 3), `ConfirmDialog` (Task 7), `useSession`, `useNotices`.
- Produces: `store::MAX_NOTE_BYTES: usize = 65_536` (`lib.rs`'ten dışa aktarılır); `add_note`/`update_note` sınırı aşan gövdede `StoreError::InvalidInput("body")` ve hiçbir şey yazmaz; `notes/saves` `enqueueNoteSave<T>(save: () => Promise<T>): Promise<T>`, `notesIdle(): Promise<void>`; `notes/NoteEditor` `NoteEditor({ note, autoFocus, onThisChart, onSaved(note), onDelete(note): Promise<boolean> })`, `NOTE_SAVE_DEBOUNCE_MS = 500`, `NOTE_SAVE_MAX_WAIT_MS = 2000`, `MAX_NOTE_UNITS = 16_384`; `notes/NotesPanel` `NotesPanel(props: PanelProps)`, `NOTES_LIMIT = 100`; panel kaydında `notes → NotesPanel`; i18n `notes.*`, `notice.noteFailed`, `notice.noteSaveLost`.

- [ ] **Step 1: Önce testleri yaz**

`app/ui/App.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -15,6 +15,7 @@ vi.mock("./api", () => ({
     listCharts: vi.fn(async () => ({ status: "ok", data: [] })),
     searchClients: vi.fn(async () => ({ status: "ok", data: [] })),
     openClient: vi.fn(),
+    listNotes: vi.fn(async () => ({ status: "ok", data: [] })),
   },
 }));
 
```

`app/ui/dashboard/Dashboard.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/dashboard/Dashboard.test.tsx
+++ b/app/ui/dashboard/Dashboard.test.tsx
@@ -1,7 +1,7 @@
-import { act, fireEvent, screen, waitFor } from "@testing-library/react";
+import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
 import { afterEach, vi } from "vitest";
 
-vi.mock("../api", () => ({ api: { dashaChildren: vi.fn() } }));
+vi.mock("../api", () => ({ api: { dashaChildren: vi.fn(), listNotes: vi.fn(async () => ({ status: "ok", data: [] })) } }));
 
 import { api } from "../api";
 import { fixture } from "../test/fixture";
@@ -60,7 +60,7 @@ test("renders every panel of the active layout on a 12-column grid", async () =>
   expect(frame("b").style.gridRow).toBe("1 / span 4");
   expect(screen.getByRole("region", { name: "Graha tablosu" })).toBeInTheDocument();
   expect(await screen.findByRole("button", { name: "Güneş" })).toBeInTheDocument();
-  expect(screen.getByRole("region", { name: "Notlar" })).toHaveTextContent("Bu panel sonraki bir sürümde gelecek.");
+  expect(await within(screen.getByRole("region", { name: "Notlar" })).findByRole("button", { name: "Yeni not" })).toBeInTheDocument();
   expect(screen.getByRole("region", { name: "Bilinmeyen panel" })).toHaveTextContent("“yoga_list” paneli");
 });
 
```

`app/ui/notes/NotesPanel.test.tsx` oluştur:

```tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { useState } from "react";
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    listNotes: vi.fn(),
    addNote: vi.fn(),
    updateNote: vi.fn(),
    deleteNote: vi.fn(),
  },
}));

import { api } from "../api";
import type { Note } from "../bindings";
import type { Panel } from "../layout/model";
import { NoticeBar } from "../shell/notices";
import { renderWithStores } from "../test/render";
import { MAX_NOTE_UNITS, NOTE_SAVE_DEBOUNCE_MS, NOTE_SAVE_MAX_WAIT_MS } from "./NoteEditor";
import { NOTES_LIMIT, NotesPanel } from "./NotesPanel";

const ok = <T,>(data: T) => ({ status: "ok" as const, data });
const panel: Panel = { id: "notes", kind: "notes", x: 0, y: 0, w: 4, h: 6, settings: {} };
const note = (id: number, body: string, created_at: string, chart_id: number | null = null): Note => ({
  id,
  uuid: `u${id}`,
  client_id: 1,
  chart_id,
  body,
  created_at,
  updated_at: created_at,
});
const older = note(1, "ilk seans", "2026-01-05T10:00:00.000Z");
const newer = note(2, "ikinci seans", "2026-02-10T15:30:00.000Z", 1);

let setShown: (shown: boolean) => void = () => {};
function Host() {
  const [shown, set] = useState(true);
  setShown = set;
  return (
    <>
      <NoticeBar />
      {shown && <NotesPanel panel={panel} />}
    </>
  );
}

function renderPanel() {
  return renderWithStores(<Host />);
}

const list = () => screen.getByRole("list", { name: "Örnek Danışan notları" });

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.listNotes).mockResolvedValue(ok([newer, older]));
  vi.mocked(api.updateNote).mockImplementation(async (id, body) => ok({ ...(id === 1 ? older : newer), body }));
});

test("notes are read from list_notes, newest first, with a date and the chart they belong to", async () => {
  renderPanel();
  const items = await within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).findAllByRole("listitem");
  expect(api.listNotes).toHaveBeenCalledWith(1, NOTES_LIMIT);
  expect(items).toHaveLength(2);
  const first = within(items[0]).getByRole("textbox");
  expect(first).toHaveValue("ikinci seans");
  expect(first).toHaveAccessibleName(/10 Şub 2026.*bu harita/);
  expect(first).toHaveAttribute("maxlength", String(MAX_NOTE_UNITS));
  expect(within(items[1]).getByRole("textbox")).toHaveAccessibleName(/5 Oca 2026/);
  expect(within(items[0]).getByText(/10 Şub 2026/).closest("time")).toHaveAttribute("datetime", newer.created_at);
});

test("typing saves once after the debounce, with the last text", async () => {
  vi.useFakeTimers();
  renderPanel();
  await act(async () => {});
  const box = within(list()).getAllByRole("textbox")[1];
  fireEvent.change(box, { target: { value: "ilk seans, d" } });
  await act(async () => vi.advanceTimersByTime(NOTE_SAVE_DEBOUNCE_MS - 1));
  fireEvent.change(box, { target: { value: "ilk seans, düzeltildi" } });
  await act(async () => vi.advanceTimersByTime(NOTE_SAVE_DEBOUNCE_MS - 1));
  expect(api.updateNote).not.toHaveBeenCalled();
  await act(async () => vi.advanceTimersByTime(1));
  vi.useRealTimers();
  expect(api.updateNote).toHaveBeenCalledOnce();
  expect(api.updateNote).toHaveBeenCalledWith(1, "ilk seans, düzeltildi");
});

test("continuous typing is still saved at least every NOTE_SAVE_MAX_WAIT_MS", async () => {
  vi.useFakeTimers();
  renderPanel();
  await act(async () => {});
  const box = within(list()).getAllByRole("textbox")[1];
  const step = 400;
  for (let i = 0; i * step < NOTE_SAVE_MAX_WAIT_MS; i++) {
    fireEvent.change(box, { target: { value: `yazım ${i}` } });
    await act(async () => vi.advanceTimersByTime(step));
  }
  vi.useRealTimers();
  // 400 ms aralıkla yazım hiçbir zaman 500 ms durmaz; yine de 2 400 ms dolmadan kaydedilir.
  expect(api.updateNote).toHaveBeenCalledOnce();
  expect(api.updateNote).toHaveBeenCalledWith(1, "yazım 4");
});

test("hiding the window saves pending text at once", async () => {
  renderPanel();
  const box = within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).getAllByRole("textbox")[0];
  fireEvent.change(box, { target: { value: "gizlenmeden önce" } });
  Object.defineProperty(document, "visibilityState", { configurable: true, get: () => "hidden" });
  try {
    act(() => void document.dispatchEvent(new Event("visibilitychange")));
  } finally {
    delete (document as { visibilityState?: unknown }).visibilityState;
  }
  await waitFor(() => expect(api.updateNote).toHaveBeenCalledWith(2, "gizlenmeden önce"));
});

test("a save that fails after the panel closed is reported without the note text", async () => {
  vi.mocked(api.updateNote).mockResolvedValue({ status: "error", error: { code: "store_unavailable", params: {} } });
  renderPanel();
  const box = within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).getAllByRole("textbox")[0];
  fireEvent.change(box, { target: { value: "gizli seans metni" } });
  act(() => setShown(false));
  const alert = await screen.findByRole("alert");
  expect(alert).toHaveTextContent("Panel kapanırken bir not kaydedilemedi; son değişiklikler yazılmadı: Danışan veritabanı kullanılamıyor.");
  expect(alert).not.toHaveTextContent("gizli seans metni");
});

test("leaving the box saves at once and closing the panel saves pending text before the next read", async () => {
  renderPanel();
  const box = within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).getAllByRole("textbox")[0];
  fireEvent.change(box, { target: { value: "ikinci seans +" } });
  fireEvent.blur(box);
  await waitFor(() => expect(api.updateNote).toHaveBeenCalledWith(2, "ikinci seans +"));
  // Değişmeyen metin yeniden yazılmaz.
  fireEvent.blur(box);
  expect(api.updateNote).toHaveBeenCalledOnce();
  fireEvent.change(box, { target: { value: "panel kapanırken" } });
  let release!: () => void;
  vi.mocked(api.updateNote).mockImplementationOnce(
    (id, body) => new Promise((resolve) => (release = () => resolve(ok({ ...newer, id, body })))),
  );
  act(() => setShown(false));
  await waitFor(() => expect(api.updateNote).toHaveBeenLastCalledWith(2, "panel kapanırken"));
  vi.mocked(api.listNotes).mockClear();
  act(() => setShown(true));
  // Liste, bekleyen kayıt bitmeden okunmaz.
  await act(async () => {});
  expect(api.listNotes).not.toHaveBeenCalled();
  release();
  await waitFor(() => expect(api.listNotes).toHaveBeenCalledOnce());
});

test("a failed save is shown at the note and retried on the next leave", async () => {
  vi.mocked(api.updateNote).mockResolvedValueOnce({ status: "error", error: { code: "store_unavailable", params: {} } });
  renderPanel();
  const box = within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).getAllByRole("textbox")[0];
  fireEvent.change(box, { target: { value: "kaydedilmeyen" } });
  fireEvent.blur(box);
  const alert = await screen.findByRole("alert");
  expect(alert).toHaveTextContent("Not kaydedilemedi: Danışan veritabanı kullanılamıyor.");
  expect(alert).not.toHaveTextContent("kaydedilmeyen");
  expect(box).toHaveAccessibleDescription(/Not kaydedilemedi/);
  fireEvent.blur(box);
  await waitFor(() => expect(api.updateNote).toHaveBeenCalledTimes(2));
  await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
});

test("New note adds an empty note on the open chart and focuses it", async () => {
  const added = note(3, "", "2026-03-01T09:00:00.000Z", 1);
  vi.mocked(api.addNote).mockResolvedValue(ok(added));
  renderPanel();
  await screen.findByRole("list", { name: "Örnek Danışan notları" });
  fireEvent.click(screen.getByRole("button", { name: "Yeni not" }));
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(3));
  expect(api.addNote).toHaveBeenCalledWith(1, 1, "");
  expect(document.activeElement).toBe(within(list()).getAllByRole("textbox")[0]);
});

test("deleting asks first; after confirming the note goes away, with no save after it", async () => {
  vi.mocked(api.deleteNote).mockResolvedValue(ok(null));
  renderPanel();
  const items = await within(await screen.findByRole("list", { name: "Örnek Danışan notları" })).findAllByRole("listitem");
  fireEvent.change(within(items[1]).getByRole("textbox"), { target: { value: "silinecek" } });
  fireEvent.click(within(items[1]).getByRole("button", { name: /5 Oca 2026.* tarihli notu sil/ }));
  const dialog = screen.getByRole("dialog", { name: "Not silinsin mi?" });
  expect(document.activeElement).toBe(within(dialog).getByRole("button", { name: "Vazgeç" }));
  fireEvent.click(within(dialog).getByRole("button", { name: "Notu sil" }));
  await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(1));
  expect(api.deleteNote).toHaveBeenCalledWith(1);
  await new Promise((r) => setTimeout(r, NOTE_SAVE_DEBOUNCE_MS + 50));
  expect(api.updateNote).not.toHaveBeenCalled();
});

test("a read error has a retry; with no chart open the panel says so", async () => {
  vi.mocked(api.listNotes).mockResolvedValueOnce({ status: "error", error: { code: "store_unavailable", params: {} } });
  renderPanel();
  const alert = await screen.findByRole("alert");
  expect(alert).toHaveTextContent("Notlar okunamadı: Danışan veritabanı kullanılamıyor.");
  fireEvent.click(within(alert).getByRole("button", { name: "Yeniden dene" }));
  expect(await screen.findByRole("list", { name: "Örnek Danışan notları" })).toBeInTheDocument();
});

test("an empty client shows the empty text", async () => {
  vi.mocked(api.listNotes).mockResolvedValue(ok([]));
  renderPanel();
  expect(await screen.findByText("Henüz not yok.")).toBeInTheDocument();
});
```

`app/ui/panels/panels.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -3,6 +3,7 @@ import { DashaTreePanel } from "../dasha/DashaTreePanel";
 import { setLanguage } from "../i18n";
 import { tr } from "../i18n/tr";
 import { PANEL_KINDS, type Panel } from "../layout/model";
+import { NotesPanel } from "../notes/NotesPanel";
 import { fixture } from "../test/fixture";
 import { renderWithStores } from "../test/render";
 import { ChartPanel } from "./ChartPanel";
@@ -21,12 +22,13 @@ const chartPanel = (settings: Record<string, unknown> = { varga: "D1", style: "n
 });
 const tablePanel: Panel = { id: "t", kind: "graha_table", x: 6, y: 0, w: 6, h: 8, settings: {} };
 
-test("registry covers the seven kinds; chart, graha_table and dasha_tree are implemented", () => {
+test("registry covers the seven kinds; chart, graha_table, dasha_tree and notes are implemented", () => {
   expect(Object.keys(PANEL_REGISTRY).sort()).toEqual([...PANEL_KINDS].sort());
   expect(PANEL_REGISTRY.chart.component).toBe(ChartPanel);
   expect(PANEL_REGISTRY.graha_table.component).toBe(GrahaTablePanel);
   expect(PANEL_REGISTRY.dasha_tree.component).toBe(DashaTreePanel);
-  for (const kind of ["dasha_strip", "transit_table", "notes", "birth_info"] as const) {
+  expect(PANEL_REGISTRY.notes.component).toBe(NotesPanel);
+  for (const kind of ["dasha_strip", "transit_table", "birth_info"] as const) {
     expect(PANEL_REGISTRY[kind].component).toBe(UnavailablePanel);
   }
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/notes ui/panels/panels.test.tsx ui/dashboard
cargo test -p store --lib a_note_body_over_the_cap
```
Expected: `notes/NotesPanel.test.tsx` `Failed to resolve import "./NoteEditor"` ile FAIL; `panels.test.tsx` `PANEL_REGISTRY.notes.component` `UnavailablePanel` olduğu için FAIL; `Dashboard.test.tsx` `Yeni not` düğmesi bulunamadığı için FAIL; Rust testi (önce yalnızca `notes.rs`'in test hunk'ı) `cannot find value MAX_NOTE_BYTES` ile derlenmez.

- [ ] **Step 3: Uygulamayı yaz**

`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -176,6 +176,8 @@ export const en: Messages = {
     openFailed: "The client could not be opened: {{message}}",
     deleteFailed: "Could not delete: {{message}}",
     tagsFailed: "Tags could not be saved: {{message}}",
+    noteFailed: "The note could not be added: {{message}}",
+    noteSaveLost: "A note could not be saved while its panel was closing; the last changes were not written: {{message}}",
   },
   storeWarning: {
     LastChartMissing: "The last opened chart was deleted; the most recently updated chart was opened.",
@@ -318,6 +320,20 @@ export const en: Messages = {
       help: "Keyboard shortcuts",
     },
   },
+  notes: {
+    add: "New note",
+    list: "Notes for {{name}}",
+    empty: "No notes yet.",
+    thisChart: "this chart",
+    delete: "Delete",
+    deleteNamed: "Delete the note from {{date}}",
+    deleteTitle: "Delete note?",
+    confirmDelete: "The note from {{date}} will be deleted. This cannot be undone.",
+    deleteConfirm: "Delete note",
+    saveFailed: "The note was not saved: {{message}} Keep typing or leave the box to try again.",
+    loadFailed: "Notes could not be read: {{message}}",
+    retry: "Retry",
+  },
   empty: {
     title: "No chart is open",
     body: "Start by entering the client's birth data.",
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -176,6 +176,8 @@ export const fr: Messages = {
     openFailed: "Le client n’a pas pu être ouvert : {{message}}",
     deleteFailed: "Suppression impossible : {{message}}",
     tagsFailed: "Les étiquettes n’ont pas pu être enregistrées : {{message}}",
+    noteFailed: "La note n’a pas pu être ajoutée : {{message}}",
+    noteSaveLost: "Une note n’a pas pu être enregistrée à la fermeture du panneau ; les dernières modifications n’ont pas été écrites : {{message}}",
   },
   storeWarning: {
     LastChartMissing: "La dernière carte ouverte a été supprimée ; la plus récemment modifiée a été ouverte.",
@@ -318,6 +320,20 @@ export const fr: Messages = {
       help: "Raccourcis clavier",
     },
   },
+  notes: {
+    add: "Nouvelle note",
+    list: "Notes de {{name}}",
+    empty: "Aucune note pour l’instant.",
+    thisChart: "cette carte",
+    delete: "Supprimer",
+    deleteNamed: "Supprimer la note du {{date}}",
+    deleteTitle: "Supprimer la note ?",
+    confirmDelete: "La note du {{date}} sera supprimée. Cette action est irréversible.",
+    deleteConfirm: "Supprimer la note",
+    saveFailed: "La note n’a pas été enregistrée : {{message}} Continuez à écrire ou quittez le champ pour réessayer.",
+    loadFailed: "Les notes n’ont pas pu être lues : {{message}}",
+    retry: "Réessayer",
+  },
   empty: {
     title: "Aucune carte ouverte",
     body: "Commencez par saisir les données de naissance du consultant.",
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -175,6 +175,8 @@ export const tr = {
     openFailed: "Danışan açılamadı: {{message}}",
     deleteFailed: "Silinemedi: {{message}}",
     tagsFailed: "Etiketler kaydedilemedi: {{message}}",
+    noteFailed: "Not eklenemedi: {{message}}",
+    noteSaveLost: "Panel kapanırken bir not kaydedilemedi; son değişiklikler yazılmadı: {{message}}",
   },
   storeWarning: {
     LastChartMissing: "Son açılan harita silinmiş; en son güncellenen harita açıldı.",
@@ -317,6 +319,20 @@ export const tr = {
       help: "Klavye kısayolları",
     },
   },
+  notes: {
+    add: "Yeni not",
+    list: "{{name}} notları",
+    empty: "Henüz not yok.",
+    thisChart: "bu harita",
+    delete: "Sil",
+    deleteNamed: "{{date}} tarihli notu sil",
+    deleteTitle: "Not silinsin mi?",
+    confirmDelete: "{{date}} tarihli not silinecek. Bu işlem geri alınamaz.",
+    deleteConfirm: "Notu sil",
+    saveFailed: "Not kaydedilemedi: {{message}} Yazmaya devam edin ya da kutudan çıkın; yeniden denenir.",
+    loadFailed: "Notlar okunamadı: {{message}}",
+    retry: "Yeniden dene",
+  },
   empty: {
     title: "Açık harita yok",
     body: "Danışanın doğum bilgisini girerek başlayın.",
```

`app/ui/notes/NoteEditor.tsx` oluştur:

```tsx
// Tek not: tarih damgası, otomatik kaydedilen metin ve onaylı silme. Yazım durunca
// `NOTE_SAVE_DEBOUNCE_MS` sonra (kesintisiz yazımda en geç `NOTE_SAVE_MAX_WAIT_MS`'de bir), odak
// çıkınca, pencere gizlenince ve panel kapanınca hemen kaydedilir (C17). Not metni loglanmaz ve
// hata bildirimine konmaz.
import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError, Note } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { ConfirmDialog } from "../shell/ConfirmDialog";
import { useNotices } from "../shell/notices";
import { enqueueNoteSave } from "./saves";

export const NOTE_SAVE_DEBOUNCE_MS = 500;
/** Kesintisiz yazımda iki kayıt arasındaki en uzun süre. */
export const NOTE_SAVE_MAX_WAIT_MS = 2000;
/** UTF-16 birimi; 16 384 × 3 bayt Rust `MAX_NOTE_BYTES` (65 536) altında kalır. */
export const MAX_NOTE_UNITS = 16_384;

export interface NoteEditorProps {
  note: Note;
  /** Yeni eklenen not: metin kutusu odak alır. */
  autoFocus: boolean;
  /** Bu haritaya bağlı not (açık harita). */
  onThisChart: boolean;
  onSaved(note: Note): void;
  /** Onaylı silme; `false`: silinemedi (bildirim panelin işidir), not yerinde kalır. */
  onDelete(note: Note): Promise<boolean>;
}

export function NoteEditor({ note, autoFocus, onThisChart, onSaved, onDelete }: NoteEditorProps) {
  const { t, i18n } = useTranslation();
  const { push } = useNotices();
  const id = useId();
  const [body, setBody] = useState(note.body);
  const [error, setError] = useState<CommandError | null>(null);
  const [confirm, setConfirm] = useState(false);
  // `saved`: Rust'a gönderilmiş son metin; `null` son kayıt başarısız oldu demektir.
  // `dirtyAt`: kaydedilmemiş ilk değişikliğin zamanı (en uzun bekleme bundan sayılır).
  const latest = useRef<{
    body: string;
    saved: string | null;
    timer: ReturnType<typeof setTimeout> | null;
    dirtyAt: number | null;
    deleted: boolean;
  }>({
    body: note.body,
    saved: note.body,
    timer: null,
    dirtyAt: null,
    deleted: false,
  });
  const mounted = useRef(true);
  const box = useRef<HTMLTextAreaElement>(null);

  const flush = () => {
    const now = latest.current;
    if (now.timer) clearTimeout(now.timer);
    now.timer = null;
    now.dirtyAt = null;
    if (now.deleted || now.body === now.saved) return;
    const text = now.body;
    now.saved = text;
    void enqueueNoteSave(() => api.updateNote(note.id, text)).then((r) => {
      if (r.status === "ok") {
        setError(null);
        onSaved(r.data);
      } else {
        // Kaydedilmedi: sonraki yazım ya da odak çıkışı yeniden dener.
        if (latest.current.saved === text) latest.current.saved = null;
        // Panel kapandıktan sonra düşen kayıt sessiz kalmaz: bildirim bandında görünür.
        if (mounted.current) setError(r.error);
        else push({ id: "note-save", type: "error", messageKey: "notice.noteSaveLost", error: r.error });
      }
    });
  };

  // Panel kapanırken, pencere gizlenirken ve uygulama kapanırken bekleyen metin yazılır
  // (görünüm deposuyla aynı olaylar; Tauri webview kapanışı `pagehide` atlayabilir).
  // `flush` yalnızca ref ve kararlı değerleri okur (not kimliği anahtardır).
  useEffect(() => {
    const onVisibility = () => {
      if (document.visibilityState === "hidden") flush();
    };
    window.addEventListener("pagehide", flush);
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      window.removeEventListener("pagehide", flush);
      document.removeEventListener("visibilitychange", onVisibility);
      mounted.current = false;
      flush();
    };
  }, []);

  useEffect(() => {
    if (autoFocus) box.current?.focus();
  }, [autoFocus]);

  const change = (text: string) => {
    setBody(text);
    const now = latest.current;
    now.body = text;
    if (now.timer) clearTimeout(now.timer);
    now.dirtyAt ??= Date.now();
    // Kesintisiz yazımda da en geç NOTE_SAVE_MAX_WAIT_MS'de bir kaydedilir.
    const wait = Math.min(NOTE_SAVE_DEBOUNCE_MS, now.dirtyAt + NOTE_SAVE_MAX_WAIT_MS - Date.now());
    now.timer = setTimeout(flush, Math.max(0, wait));
  };

  const remove = async () => {
    setConfirm(false);
    latest.current.deleted = true;
    if (latest.current.timer) clearTimeout(latest.current.timer);
    if (!(await onDelete(note))) latest.current.deleted = false;
  };

  const date = new Intl.DateTimeFormat(i18n.language, { dateStyle: "medium", timeStyle: "short" }).format(new Date(note.created_at));
  return (
    <li className="note">
      <div className="note-head">
        <label htmlFor={`${id}-body`} className="note-date">
          <time dateTime={note.created_at}>{date}</time>
          {onThisChart && <span className="note-chart">{t("notes.thisChart")}</span>}
        </label>
        <button type="button" className="icon-button" aria-label={t("notes.deleteNamed", { date })} onClick={() => setConfirm(true)}>
          {t("notes.delete")}
        </button>
      </div>
      <textarea
        id={`${id}-body`}
        ref={box}
        value={body}
        maxLength={MAX_NOTE_UNITS}
        rows={3}
        aria-describedby={error ? `${id}-error` : undefined}
        onChange={(e) => change(e.target.value)}
        onBlur={flush}
      />
      {error && (
        <p id={`${id}-error`} className="note-error" role="alert">
          {t("notes.saveFailed", { message: errorMessage(t, i18n, error) })}
        </p>
      )}
      {confirm && (
        <ConfirmDialog
          title={t("notes.deleteTitle")}
          message={t("notes.confirmDelete", { date })}
          confirmLabel={t("notes.deleteConfirm")}
          onConfirm={() => void remove()}
          onCancel={() => setConfirm(false)}
        />
      )}
    </li>
  );
}
```

`app/ui/notes/NotesPanel.tsx` oluştur:

```tsx
// Notlar paneli (spec §3.3, §4): açık danışanın tarih damgalı notları, en yeni önce. Liste
// `list_notes` ile okunur (bekleyen kayıtlar bittikten sonra); `open_client`'ın `recent_notes`
// alanı kullanılmaz, çünkü bu oturumdaki düzenlemelerden sonra eskir. "Yeni not" boş bir not
// ekler ve ona odaklanır; metin otomatik kaydedilir.
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError, Note, OpenedClient } from "../bindings";
import { errorMessage } from "../i18n/errors";
import type { PanelProps } from "../panels/types";
import { useSession } from "../session/store";
import { useNotices } from "../shell/notices";
import { NoteEditor } from "./NoteEditor";
import { notesIdle } from "./saves";
import "./notes.css";

/** Panelde gösterilen en çok not (Rust `MAX_LIST` 500). */
export const NOTES_LIMIT = 100;

export function NotesPanel(_props: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <ClientNotes key={opened.client.id} opened={opened} />;
}

function ClientNotes({ opened }: { opened: OpenedClient }) {
  const { t, i18n } = useTranslation();
  const { push } = useNotices();
  const clientId = opened.client.id;
  const [notes, setNotes] = useState<Note[] | null>(null);
  const [error, setError] = useState<CommandError | null>(null);
  const [retry, setRetry] = useState(0);
  const [fresh, setFresh] = useState<number | null>(null);

  useEffect(() => {
    let live = true;
    void notesIdle()
      .then(() => api.listNotes(clientId, NOTES_LIMIT))
      .then((r) => {
        if (!live) return;
        if (r.status === "ok") {
          setNotes(r.data);
          setError(null);
        } else setError(r.error);
      });
    return () => {
      live = false;
    };
  }, [clientId, retry]);

  const add = async () => {
    const r = await api.addNote(clientId, opened.chart_record.id, "");
    if (r.status === "error") return push({ id: "note", type: "error", messageKey: "notice.noteFailed", error: r.error });
    setNotes((list) => [r.data, ...(list ?? [])]);
    setFresh(r.data.id);
  };

  const saved = useCallback((n: Note) => setNotes((list) => list && list.map((x) => (x.id === n.id ? n : x))), []);

  const remove = async (n: Note) => {
    const r = await api.deleteNote(n.id);
    if (r.status === "error") {
      push({ id: "delete", type: "error", messageKey: "notice.deleteFailed", error: r.error });
      return false;
    }
    setNotes((list) => list && list.filter((x) => x.id !== n.id));
    return true;
  };

  return (
    <div className="notes-panel">
      <div className="notes-actions">
        <button type="button" onClick={() => void add()} disabled={notes === null}>
          {t("notes.add")}
        </button>
      </div>
      {error && (
        <div className="note-error" role="alert">
          <p>{t("notes.loadFailed", { message: errorMessage(t, i18n, error) })}</p>
          <button type="button" onClick={() => setRetry((n) => n + 1)}>
            {t("notes.retry")}
          </button>
        </div>
      )}
      {notes && notes.length === 0 && <p className="panel-message">{t("notes.empty")}</p>}
      {notes && notes.length > 0 && (
        <ul className="notes-list" aria-label={t("notes.list", { name: opened.client.name })}>
          {notes.map((n) => (
            <NoteEditor
              key={n.id}
              note={n}
              autoFocus={n.id === fresh}
              onThisChart={n.chart_id === opened.chart_record.id}
              onSaved={saved}
              onDelete={remove}
            />
          ))}
        </ul>
      )}
    </div>
  );
}
```

`app/ui/notes/notes.css` oluştur:

```css
.notes-panel {
  display: flex;
  flex-direction: column;
  gap: 6px;
  height: 100%;
  min-height: 0;
}
.notes-list {
  flex: 1;
  min-height: 0;
  overflow: auto;
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.note {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.note-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.note-date {
  color: var(--text-2);
  font-size: var(--fs-table);
}
/* Haritaya bağlı not metinle de belirtilir; renk tek taşıyıcı değildir. */
.note-chart {
  margin-left: 6px;
  padding: 0 4px;
  border: 1px solid var(--line);
  border-radius: 3px;
}
.note textarea {
  width: 100%;
  resize: vertical;
  font: inherit;
}
.note-error {
  margin: 0;
  color: var(--danger);
  font-size: var(--fs-table);
}
```

`app/ui/notes/saves.ts` oluştur:

```ts
// Not kayıtlarının tek sırası. Otomatik kayıtlar sırayla gider (aynı notun eski metni yenisinin
// üstüne yazılmaz) ve not listesi okunmadan önce bekleyen kayıtlar biter: panel kapanırken
// gönderilen son kayıt, yeniden açılan panelin okuduğu listede görünür.
let chain: Promise<unknown> = Promise.resolve();

export function enqueueNoteSave<T>(save: () => Promise<T>): Promise<T> {
  const next = chain.then(save, save);
  chain = next.catch(() => undefined);
  return next;
}

/** Bekleyen bütün not kayıtları bitince çözülür. */
export function notesIdle(): Promise<void> {
  return chain.then(() => undefined);
}
```

`app/ui/panels/registry.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -1,6 +1,7 @@
 // Panel türü → bileşen (spec §3.3). Yeni panel türü yalnızca buraya eklenir.
 import { DashaTreePanel } from "../dasha/DashaTreePanel";
 import { isKnownPanelKind, type KnownPanelKind } from "../layout/model";
+import { NotesPanel } from "../notes/NotesPanel";
 import { ChartPanel, chartVarga } from "./ChartPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
 import type { PanelDefinition } from "./types";
@@ -12,7 +13,7 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   dasha_tree: { titleKey: "panel.dasha_tree", component: DashaTreePanel },
   dasha_strip: { titleKey: "panel.dasha_strip", component: UnavailablePanel },
   transit_table: { titleKey: "panel.transit_table", component: UnavailablePanel },
-  notes: { titleKey: "panel.notes", component: UnavailablePanel },
+  notes: { titleKey: "panel.notes", component: NotesPanel },
   birth_info: { titleKey: "panel.birth_info", component: UnavailablePanel },
 };
 
```

`crates/store/src/lib.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/crates/store/src/lib.rs
+++ b/crates/store/src/lib.rs
@@ -32,7 +32,7 @@ pub use model::{
     BirthData, ChartKind, ChartRecord, Client, FoldChoice, Gender, MomentWarning, NewChart,
     NewClient, Note, Place, ResolvedRecord, TimeCertainty, TzChoice,
 };
-pub use notes::{normalize_tags, TagCount, MAX_TAG_CHARS, RECENT_NOTES};
+pub use notes::{normalize_tags, TagCount, MAX_NOTE_BYTES, MAX_TAG_CHARS, RECENT_NOTES};
 pub use open::{OpenRecord, StoreWarning};
 pub use schema::SCHEMA_VERSION;
 pub use search::{SearchHit, SearchMatch};
```

`crates/store/src/notes.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/crates/store/src/notes.rs
+++ b/crates/store/src/notes.rs
@@ -14,6 +14,16 @@ use crate::{Store, StoreError};
 pub const RECENT_NOTES: u32 = 20;
 /// Etiket uzunluğu sınırı (karakter).
 pub const MAX_TAG_CHARS: usize = 64;
+/// Not gövdesi sınırı (UTF-8 bayt). Arayüz `maxLength` ile daha önce durur
+/// (`MAX_NOTE_UNITS` 16 384 UTF-16 birimi en çok 49 152 bayttır).
+pub const MAX_NOTE_BYTES: usize = 65_536;
+
+fn check_body(body: &str) -> Result<(), StoreError> {
+    if body.len() > MAX_NOTE_BYTES {
+        return Err(StoreError::InvalidInput("body"));
+    }
+    Ok(())
+}
 
 #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
 #[cfg_attr(feature = "specta", derive(specta::Type))]
@@ -111,6 +121,7 @@ impl Store {
         chart_id: Option<u32>,
         body: &str,
     ) -> Result<Note, StoreError> {
+        check_body(body)?;
         self.client(client_id)?;
         if let Some(ch) = chart_id {
             if self.chart(ch)?.client_id != client_id {
@@ -129,6 +140,7 @@ impl Store {
 
     /// Otomatik kayıt; `created_at` değişmez.
     pub fn update_note(&mut self, id: u32, body: &str) -> Result<Note, StoreError> {
+        check_body(body)?;
         let now = self.now();
         let n = self.conn.execute(
             "UPDATE client_notes SET body = ?1, updated_at = ?2 WHERE id = ?3",
@@ -251,6 +263,25 @@ mod tests {
         ));
     }
 
+    #[test]
+    fn a_note_body_over_the_cap_is_invalid_input_and_nothing_is_written() {
+        let mut s = store();
+        let (a, _) = s.create_client(&new_client("A")).unwrap();
+        let at_cap = "ş".repeat(MAX_NOTE_BYTES / 2);
+        let n = s.add_note(a.id, None, &at_cap).unwrap();
+        let over = format!("{at_cap}x");
+        assert!(matches!(
+            s.add_note(a.id, None, &over),
+            Err(StoreError::InvalidInput("body"))
+        ));
+        assert!(matches!(
+            s.update_note(n.id, &over),
+            Err(StoreError::InvalidInput("body"))
+        ));
+        assert_eq!(s.note(n.id).unwrap().body, at_cap);
+        assert_eq!(s.notes(a.id, 10).unwrap().len(), 1);
+    }
+
     #[test]
     fn notes_survive_chart_deletion_and_go_with_client() {
         let mut s = store();
```

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
cargo test -p store --lib
cargo clippy -p store --all-targets -- -D warnings
cargo fmt --all --check
pnpm --dir app test
pnpm --dir app build
for i in 1 2 3; do pnpm --dir app exec vitest run ui/notes 2>&1 | grep -E "Tests "; done
```
Expected: `store` lib `test result: ok. 71 passed` (+1); clippy, fmt çıktısız; `Test Files  41 passed (41)`, `Tests  341 passed (341)`; `build` `✓ built`; notlar testi üç kez `11 passed`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/App.test.tsx \
  app/ui/dashboard/Dashboard.test.tsx \
  app/ui/i18n/en.ts \
  app/ui/i18n/fr.ts \
  app/ui/i18n/tr.ts \
  app/ui/notes/NoteEditor.tsx \
  app/ui/notes/NotesPanel.test.tsx \
  app/ui/notes/NotesPanel.tsx \
  app/ui/notes/notes.css \
  app/ui/notes/saves.ts \
  app/ui/panels/panels.test.tsx \
  app/ui/panels/registry.ts \
  crates/store/src/lib.rs \
  crates/store/src/notes.rs
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git diff --cached --name-only | grep -iE 'k(a|ā|aa)la|\bcala\b|^\.superpowers/'   # boş olmalı
git commit -m "feat: notes panel with ordered autosave, confirmed delete and a note body cap

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: commit oluşur; iki tarama da çıktısız. Commit mesajında yasaklı sözcük yoktur (C13).

---

### Task 10: İçe/dışa aktarma: Rust tarafı dosya pencereleri, boyut sınırı, atomik yazım ve dosyadan gelen alanların sınırları

Plan 08 ertelenen madde 3 (JSON için) ve 13; madde 1 (JHD) kapanmaz, JHD arayüzden erişilemez kalır (D2). `tauri-plugin-dialog` `=2.8.4` (Apache-2.0 OR MIT; `rfd` 0.17.2 MIT, `tauri-plugin-fs` 2.7.1 Apache-2.0 OR MIT getirir; MSRV 1.90) eklenir ve pencereler **Rust'ta** `DialogExt` ile `spawn_blocking` içinde açılır: arayüz hiçbir zaman dosya yolu göndermez, eklentinin JS komutları için yetenek (capability) verilmez. Yol alan dört komut (`export_json`, `export_csv`, `import_json`, `import_jhd`) specta kaydından çıkar; yerlerine `export_clients_json(include_notes)`, `export_clients_csv()`, `import_clients_json()` gelir (vazgeçilirse `null`). Kapılar: içe aktarma dosyası okunmadan önce `metadata().len()` ile `MAX_IMPORT_BYTES = 32 MiB` denetlenir (yeni kod `import_too_large{file, max_mb}`; okurken büyüyen dosya da reddedilir); dışa aktarma aynı klasörde geçici dosyaya yazılıp `sync_all` + `rename` ile taşınır (yarıda kesilen yazım eski dosyayı bozmaz); hata parametrelerinde yalnızca dosya adı (`file`), tam yol yok; `import_format.detail` ve `import_record.item` 120 karakterde kesilir; başka tzdb ile korunan anda `abbrev` ve `tzdb_version` 32 karakteri, ofset ±18 saati aşarsa kayıt reddedilir; içe aktarılan not gövdesi `MAX_NOTE_BYTES`'ı aşarsa kayıt reddedilir (hepsi ya da hiçbiri). Arayüz: kenar çubuğunun "Veri" bölümü; JSON dışa aktarma önce bir pencere açar, "Notları dahil et" varsayılan kapalı ve hassasiyet ipucu bağlı (spec §5). Sonuçlar bildirim bandında: yeni `info` bildirimi (kenar çizgisi + "Bilgi" metni, renk tek taşıyıcı değil). İçe aktarmadan sonra `session.touch()` listeleri yeniler. `api.test.ts`'in C2 denetimi güçlenir: `bindings.ts`'te yol alan hiçbir komut yoktur (C2'nin kaldırılma koşulu bu görevdir; yerine C2′). "Arayüzün dosya penceresi ya da dosya sistemi yetkisi yok" iddiası bir Rust testiyle korunur: `capabilities/` klasöründe tek dosya vardır ve `default.json`'ın izinleri yalnızca `["core:event:default"]`'tur (C2′, Fable MF4).

**Files:**
- Modify: `app/src-tauri/Cargo.toml`
- Modify: `app/src-tauri/src/client_commands.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/src/store_commands.rs`
- Test: `app/ui/api.test.ts`
- Modify: `app/ui/api.ts`
- Test (yeni): `app/ui/clients/DataActions.test.tsx`
- Create: `app/ui/clients/DataActions.tsx`
- Modify: `app/ui/clients/Sidebar.tsx`
- Modify: `app/ui/clients/clients.css`
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Test: `app/ui/i18n/i18n.test.ts`
- Modify: `app/ui/i18n/tr.ts`
- Modify: `app/ui/shell/notices.tsx`
- Modify: `app/ui/shell/shell.css`
- Modify: `crates/store/src/exchange.rs`
- Modify: `Cargo.lock` (üretilir: `cargo` bağımlılığı çözerken günceller)
- Modify: `app/ui/bindings.ts` (üretilir: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings`)

**Interfaces:**
- Consumes: `Store::export_json/export_csv/import_json` (Plan 08), `MAX_NOTE_BYTES` (Task 9), `blocking`, `lock`, `DataActions` için `Session.touch` (Task 6) ve `Dialog`.
- Produces: Rust `store_commands::{MAX_IMPORT_BYTES: u64, EXPORT_JSON_NAME, EXPORT_CSV_NAME, write_atomic(path: &Path, text: &str) -> Result<(), CommandError>, read_bounded(path: &Path) -> Result<String, CommandError>, ImportResult { file: String, report: ImportReport }}`; `export_json_file_blocking(store, path: &Path, options)`, `export_csv_file_blocking(store, path: &Path)`, `import_json_file_blocking(store, path: &Path)` (artık `&Path`); `client_commands::MAX_PARAM_CHARS = 120`; IPC `export_clients_json(include_notes: bool) -> Option<String>`, `export_clients_csv() -> Option<String>`, `import_clients_json() -> Option<ImportResult>`; hata kodları `file_io{file, detail}`, `dialog_path`, `import_too_large{file, max_mb}`, `import_format{detail}`, `import_version{found, supported}`, `import_record{item, field}` (hepsinin tr/en/fr metni, C12). Rust testi `the_webview_is_granted_no_dialog_or_file_system_permission`; TS `api.exportClientsJson(includeNotes)`, `api.exportClientsCsv()`, `api.importClientsJson()`; `Notice` `type: "info"`; `clients/DataActions` `DataActions()`; i18n `data.*`, `notice.exportFailed/importFailed/info`.

- [ ] **Step 1: Önce testleri yaz**

`app/ui/api.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -24,6 +24,9 @@ vi.mock("./bindings", () => ({
     saveViewState: vi.fn(),
     loadPanelLayout: vi.fn(),
     savePanelLayout: vi.fn(),
+    exportClientsJson: vi.fn(),
+    exportClientsCsv: vi.fn(),
+    importClientsJson: vi.fn(),
     dashaChildren: vi.fn(),
     searchPlaces: vi.fn(),
     previewMoment: vi.fn(),
@@ -32,6 +35,7 @@ vi.mock("./bindings", () => ({
 
 import { api } from "./api";
 import { commands } from "./bindings";
+import bindingsSource from "./bindings.ts?raw";
 
 test("passes ok results through", async () => {
   vi.mocked(commands.engineStatus).mockResolvedValueOnce({
@@ -88,6 +92,9 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["saveViewState", [3, { schema_version: 1 }]],
     ["loadPanelLayout", []],
     ["savePanelLayout", [null]],
+    ["exportClientsJson", [false]],
+    ["exportClientsCsv", []],
+    ["importClientsJson", []],
     ["dashaChildren", [{ birth_jd_ut: 1 }, ["Sun"]]],
     ["searchPlaces", ["delhi", 8]],
     ["previewMoment", [{ date: "2000-01-01" }]],
@@ -101,7 +108,12 @@ test("every wrapper forwards its arguments to the command of the same name", asy
   }
 });
 
-test("import and export commands are not reachable from the UI (Plan 08 deferred items 1 and 3)", () => {
+test("no command takes a file path from the UI and JHD import is not reachable (Plan 08 deferred items 1 and 3)", () => {
+  // Dosya yolu yalnızca Rust'taki pencereden gelir; yol alan komut bağlamalarda yoktur.
+  for (const name of ["export_json", "export_csv", "import_json", "import_jhd"]) {
+    expect(bindingsSource).not.toContain(`"${name}"`);
+  }
+  expect(bindingsSource).not.toMatch(/\bpath: string\b|\bpaths: string\[\]/);
   for (const name of ["importJson", "importJhd", "exportJson", "exportCsv"]) {
     expect(Object.keys(api)).not.toContain(name);
   }
```

`app/ui/clients/DataActions.test.tsx` oluştur:

```tsx
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    exportClientsJson: vi.fn(),
    exportClientsCsv: vi.fn(),
    importClientsJson: vi.fn(),
  },
}));

import { api } from "../api";
import { useSession } from "../session/store";
import { NoticeBar } from "../shell/notices";
import { renderWithStores } from "../test/render";
import { DataActions } from "./DataActions";

const ok = <T,>(data: T) => ({ status: "ok" as const, data });
const report = { clients_added: 2, charts_added: 3, notes_added: 0, skipped: 1, warnings: [] };

function Revision() {
  return <output aria-label="sürüm">{useSession().revision}</output>;
}

function renderData() {
  renderWithStores(
    <>
      <NoticeBar />
      <Revision />
      <DataActions />
    </>,
  );
}

const section = () => screen.getByRole("region", { name: "Veri" });

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
});

test("JSON export asks about notes first, unchecked by default, and reports the saved file name", async () => {
  vi.mocked(api.exportClientsJson).mockResolvedValue(ok("danisanlar.json"));
  renderData();
  fireEvent.click(within(section()).getByRole("button", { name: "JSON olarak dışa aktar…" }));
  const dialog = screen.getByRole("dialog", { name: "JSON olarak dışa aktar" });
  const notes = within(dialog).getByRole("checkbox", { name: "Notları dahil et" });
  expect(notes).not.toBeChecked();
  expect(notes).toHaveAccessibleDescription("Notlar hassas veridir; dosyayı kiminle paylaştığınıza dikkat edin.");
  fireEvent.click(within(dialog).getByRole("button", { name: "Dosya seç ve kaydet…" }));
  expect(await screen.findByRole("status", { name: "" })).toHaveTextContent("“danisanlar.json” kaydedildi.");
  expect(api.exportClientsJson).toHaveBeenCalledWith(false);
  expect(screen.queryByRole("dialog")).toBeNull();
});

test("checking Include notes exports with notes; cancelling the file dialog says nothing", async () => {
  vi.mocked(api.exportClientsJson).mockResolvedValue(ok(null));
  renderData();
  fireEvent.click(within(section()).getByRole("button", { name: "JSON olarak dışa aktar…" }));
  const dialog = screen.getByRole("dialog", { name: "JSON olarak dışa aktar" });
  fireEvent.click(within(dialog).getByRole("checkbox", { name: "Notları dahil et" }));
  fireEvent.click(within(dialog).getByRole("button", { name: "Dosya seç ve kaydet…" }));
  await waitFor(() => expect(api.exportClientsJson).toHaveBeenCalledWith(true));
  await waitFor(() => expect(within(section()).getByRole("button", { name: "JSON olarak dışa aktar…" })).toBeEnabled());
  expect(screen.queryByRole("region", { name: "Bildirimler" })).toBeNull();
});

test("CSV export errors are shown with the file name only", async () => {
  vi.mocked(api.exportClientsCsv).mockResolvedValue({ status: "error", error: { code: "file_io", params: { file: "haritalar.csv", detail: "permission denied" } } });
  renderData();
  fireEvent.click(within(section()).getByRole("button", { name: "CSV olarak dışa aktar…" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Dışa aktarılamadı: Dosya okunamadı ya da yazılamadı (haritalar.csv: permission denied).");
});

test("an import reports what was added and refreshes the lists", async () => {
  vi.mocked(api.importClientsJson).mockResolvedValue(ok({ file: "yedek.json", report }));
  renderData();
  expect(screen.getByRole("status", { name: "sürüm" })).toHaveTextContent("0");
  fireEvent.click(within(section()).getByRole("button", { name: "JSON içe aktar…" }));
  await waitFor(() => expect(screen.getByRole("status", { name: "sürüm" })).toHaveTextContent("1"));
  expect(screen.getByRole("region", { name: "Bildirimler" })).toHaveTextContent(
    "“yedek.json” içe aktarıldı: 2 danışan, 3 harita, 0 not eklendi; 1 kayıt zaten vardı.",
  );
});

test("an import with reset settings is a warning; a refused import adds nothing and says so", async () => {
  vi.mocked(api.importClientsJson)
    .mockResolvedValueOnce(ok({ file: "eski.json", report: { ...report, warnings: [{ item: "u", code: "CalcSettingsReset" as const }] } }))
    .mockResolvedValueOnce({ status: "error", error: { code: "import_too_large", params: { file: "dev.json", max_mb: "32" } } });
  renderData();
  const button = within(section()).getByRole("button", { name: "JSON içe aktar…" });
  fireEvent.click(button);
  expect(await screen.findByText(/1 haritanın hesap ayarı okunamadı/)).toBeInTheDocument();
  await waitFor(() => expect(button).toBeEnabled());
  fireEvent.click(button);
  expect(await screen.findByRole("alert")).toHaveTextContent("İçe aktarılamadı; hiçbir kayıt eklenmedi: “dev.json” çok büyük; en çok 32 MB içe aktarılabilir.");
  expect(screen.getByRole("status", { name: "sürüm" })).toHaveTextContent("1");
});
```

`app/ui/i18n/i18n.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/i18n.test.ts
+++ b/app/ui/i18n/i18n.test.ts
@@ -75,6 +75,7 @@ const BACKEND_CODES = [
   "date_out_of_range", "last_chart", "unsupported_schema_version", "invalid_settings", "invalid_location", "invalid_julian_day",
   "non_finite_result", "invalid_dasha_input", "panchang_search", "invalid_transit_input",
   "atlas_unavailable", "atlas_sqlite", "atlas_io", "atlas_schema",
+  "file_io", "dialog_path", "import_too_large", "import_format", "import_version", "import_record",
 ] as const;
 
 test("every backend error code reachable from the UI has a message", () => {
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/api.test.ts ui/i18n ui/clients/DataActions.test.tsx
cargo test -p store --lib imported_moments_uuids_and_empty_clients_are_checked
```
Expected: `DataActions.test.tsx` `Failed to resolve import "./DataActions"` ile FAIL; `api.test.ts` yol alan komutlar (`"export_json"` …) `bindings.ts`'te bulunduğu için FAIL; `i18n` yeni kodlar için FAIL; Rust testi (önce yalnızca `exchange.rs` test hunk'ı) `abbrev` için `panicked … Ok(ImportReport` ile FAIL.

- [ ] **Step 3: Uygulamayı yaz**

`app/src-tauri/Cargo.toml` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/src-tauri/Cargo.toml
+++ b/app/src-tauri/Cargo.toml
@@ -25,6 +25,9 @@ serde_json = { version = "1", features = ["float_roundtrip"] }
 specta = { version = "=2.0.0-rc.28", features = ["derive"] }
 tauri-specta = { version = "=2.0.0-rc.29", features = ["derive", "typescript"] }
 specta-typescript = "0.0.15"
+# Dosya seçme/kaydetme pencereleri Rust tarafında açılır (`DialogExt`); arayüze yol seçtiren
+# bir komut ve yetenek (capability) yoktur. Apache-2.0 OR MIT.
+tauri-plugin-dialog = "=2.8.4"
 
 [dev-dependencies]
 tempfile = "3"
```

`app/src-tauri/src/client_commands.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/src-tauri/src/client_commands.rs
+++ b/app/src-tauri/src/client_commands.rs
@@ -85,7 +85,9 @@ impl From<StoreError> for CommandError {
             },
             StoreError::Settings(e) => e.into(),
             StoreError::LastChart => Self::new("last_chart", &[]),
-            StoreError::ImportFormat(detail) => Self::new("import_format", &[("detail", detail)]),
+            StoreError::ImportFormat(detail) => {
+                Self::new("import_format", &[("detail", clip(&detail))])
+            }
             StoreError::ImportVersion { found, supported } => Self::new(
                 "import_version",
                 &[
@@ -95,7 +97,7 @@ impl From<StoreError> for CommandError {
             ),
             StoreError::ImportRecord { item, field } => Self::new(
                 "import_record",
-                &[("item", item), ("field", field.to_owned())],
+                &[("item", clip(&item)), ("field", field.to_owned())],
             ),
             StoreError::Jhd { line, field } => Self::new(
                 "jhd_invalid",
@@ -105,6 +107,17 @@ impl From<StoreError> for CommandError {
     }
 }
 
+/// Dosyadan gelen metin (ayrıştırıcı iletisi, kayıt kimliği) hata parametresinde en çok
+/// `MAX_PARAM_CHARS` karakterdir: bildirim satırı taşmaz, dosya içeriği taşınmaz.
+pub const MAX_PARAM_CHARS: usize = 120;
+
+fn clip(text: &str) -> String {
+    match text.char_indices().nth(MAX_PARAM_CHARS) {
+        Some((i, _)) => format!("{}…", &text[..i]),
+        None => text.to_owned(),
+    }
+}
+
 pub(crate) fn lock(store: &Mutex<Store>) -> Result<std::sync::MutexGuard<'_, Store>, CommandError> {
     store
         .lock()
@@ -375,4 +388,19 @@ mod tests {
             ("import_record", "u")
         );
     }
+
+    #[test]
+    fn import_parameters_from_the_file_are_clipped() {
+        let long = "ş".repeat(MAX_PARAM_CHARS + 30);
+        let e = CommandError::from(StoreError::ImportRecord {
+            item: long.clone(),
+            field: "uuid",
+        });
+        assert_eq!(e.params["item"].chars().count(), MAX_PARAM_CHARS + 1);
+        assert!(e.params["item"].ends_with('…'));
+        let e = CommandError::from(StoreError::ImportFormat(long));
+        assert_eq!(e.params["detail"].chars().count(), MAX_PARAM_CHARS + 1);
+        let e = CommandError::from(StoreError::ImportFormat("kısa".into()));
+        assert_eq!(e.params["detail"], "kısa");
+    }
 }
```

`app/src-tauri/src/lib.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -48,10 +48,9 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             store_commands::set_app_prefs,
             store_commands::load_panel_layout,
             store_commands::save_panel_layout,
-            store_commands::export_json,
-            store_commands::export_csv,
-            store_commands::import_json,
-            store_commands::import_jhd,
+            store_commands::export_clients_json,
+            store_commands::export_clients_csv,
+            store_commands::import_clients_json,
             atlas_commands::search_places,
             atlas_commands::preview_moment,
         ])
@@ -60,6 +59,7 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
 pub fn run() {
     let builder = specta_builder();
     tauri::Builder::default()
+        .plugin(tauri_plugin_dialog::init())
         .invoke_handler(builder.invoke_handler())
         .setup(move |app| {
             builder.mount_events(app);
```

`app/src-tauri/src/store_commands.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/src-tauri/src/store_commands.rs
+++ b/app/src-tauri/src/store_commands.rs
@@ -1,7 +1,9 @@
 //! Kayıt yönetimi komutları (danışan, harita, not, etiket, tercihler) ve dosya içe/dışa
 //! aktarma. Harita hesabı gerektirmez; gövdeler `store`'dadır.
 
-use std::path::Path;
+use std::fs::File;
+use std::io::{Read, Write};
+use std::path::{Path, PathBuf};
 use std::sync::{Arc, Mutex};
 
 use serde::Serialize;
@@ -10,7 +12,8 @@ use store::{
     AppPrefs, ChartRecord, Client, ExportOptions, Gender, ImportReport, Note, PanelLayout,
     SearchHit, Store, TagCount,
 };
-use tauri::State;
+use tauri::{AppHandle, State};
+use tauri_plugin_dialog::{DialogExt, FilePath};
 
 use crate::client_commands::{blocking, lock, with_store, StoreState};
 use crate::commands::CommandError;
@@ -28,16 +31,65 @@ pub struct JhdImportResult {
     pub failed: Vec<FileFailure>,
 }
 
+/// İçe aktarılan JSON dosyasının sınırı; dosya okunmadan önce boyutuyla denetlenir.
+pub const MAX_IMPORT_BYTES: u64 = 32 * 1024 * 1024;
+
+/// Hata parametrelerinde yalnızca dosya adı bulunur: tam yol kullanıcı adını taşır.
+fn base_name(path: &Path) -> String {
+    path.file_name()
+        .map(|n| n.to_string_lossy().into_owned())
+        .unwrap_or_default()
+}
+
 fn io_error(path: &Path, e: std::io::Error) -> CommandError {
     CommandError::new(
         "file_io",
-        &[
-            ("path", path.display().to_string()),
-            ("detail", e.to_string()),
-        ],
+        &[("file", base_name(path)), ("detail", e.kind().to_string())],
     )
 }
 
+/// Aynı klasörde geçici dosyaya yazıp yerine taşır: yarıda kesilen yazım eski dosyayı bozmaz.
+pub fn write_atomic(path: &Path, text: &str) -> Result<(), CommandError> {
+    let tmp = path.with_file_name(format!(".{}.{}.tmp", base_name(path), std::process::id()));
+    let written = (|| {
+        let mut f = File::create(&tmp)?;
+        f.write_all(text.as_bytes())?;
+        f.sync_all()?;
+        std::fs::rename(&tmp, path)
+    })();
+    written.map_err(|e| {
+        let _ = std::fs::remove_file(&tmp);
+        io_error(path, e)
+    })
+}
+
+/// `MAX_IMPORT_BYTES`'tan büyük dosya okunmaz (okurken büyüyen dosya da reddedilir).
+pub fn read_bounded(path: &Path) -> Result<String, CommandError> {
+    let too_large = || {
+        CommandError::new(
+            "import_too_large",
+            &[
+                ("file", base_name(path)),
+                ("max_mb", (MAX_IMPORT_BYTES / (1024 * 1024)).to_string()),
+            ],
+        )
+    };
+    let len = std::fs::metadata(path)
+        .map_err(|e| io_error(path, e))?
+        .len();
+    if len > MAX_IMPORT_BYTES {
+        return Err(too_large());
+    }
+    let mut text = String::new();
+    File::open(path)
+        .and_then(|f| f.take(MAX_IMPORT_BYTES + 1).read_to_string(&mut text))
+        .map_err(|e| io_error(path, e))?;
+    if text.len() as u64 > MAX_IMPORT_BYTES {
+        return Err(too_large());
+    }
+    Ok(text)
+}
+
 pub fn import_jhd_files_blocking(
     store: &Mutex<Store>,
     paths: &[String],
@@ -46,10 +98,7 @@ pub fn import_jhd_files_blocking(
     let mut failed = Vec::new();
     for p in paths {
         let path = Path::new(p);
-        let name = path
-            .file_name()
-            .map(|n| n.to_string_lossy().into_owned())
-            .unwrap_or_else(|| p.clone());
+        let name = base_name(path);
         match std::fs::read(path) {
             Ok(bytes) => files.push((name, bytes)),
             Err(e) => failed.push(FileFailure {
@@ -71,26 +120,57 @@ pub fn import_jhd_files_blocking(
 
 pub fn export_json_file_blocking(
     store: &Mutex<Store>,
-    path: &str,
+    path: &Path,
     options: &ExportOptions,
 ) -> Result<(), CommandError> {
     let text = lock(store)?.export_json(options)?;
-    std::fs::write(path, text).map_err(|e| io_error(Path::new(path), e))
+    write_atomic(path, &text)
 }
 
-pub fn export_csv_file_blocking(store: &Mutex<Store>, path: &str) -> Result<(), CommandError> {
+pub fn export_csv_file_blocking(store: &Mutex<Store>, path: &Path) -> Result<(), CommandError> {
     let text = lock(store)?.export_csv()?;
-    std::fs::write(path, text).map_err(|e| io_error(Path::new(path), e))
+    write_atomic(path, &text)
 }
 
 pub fn import_json_file_blocking(
     store: &Mutex<Store>,
-    path: &str,
+    path: &Path,
 ) -> Result<ImportReport, CommandError> {
-    let text = std::fs::read_to_string(path).map_err(|e| io_error(Path::new(path), e))?;
+    let text = read_bounded(path)?;
     Ok(lock(store)?.import_json(&text)?)
 }
 
+/// Seçilen dosyanın adı ve içe aktarma özeti.
+#[derive(Debug, Clone, Serialize, Type)]
+pub struct ImportResult {
+    pub file: String,
+    pub report: ImportReport,
+}
+
+fn chosen(path: Option<FilePath>) -> Result<Option<PathBuf>, CommandError> {
+    path.map(|p| {
+        p.into_path()
+            .map_err(|_| CommandError::new("dialog_path", &[]))
+    })
+    .transpose()
+}
+
+/// Kaydetme penceresi. `None`: kullanıcı vazgeçti.
+fn save_dialog(
+    app: &AppHandle,
+    filter: &str,
+    ext: &str,
+    name: &str,
+) -> Result<Option<PathBuf>, CommandError> {
+    chosen(
+        app.dialog()
+            .file()
+            .add_filter(filter, &[ext])
+            .set_file_name(name)
+            .blocking_save_file(),
+    )
+}
+
 #[tauri::command]
 #[specta::specta]
 pub async fn update_client(
@@ -243,56 +323,192 @@ pub async fn save_panel_layout(
     with_store(&store, move |s| s.save_panel_layout(layout.as_ref())).await
 }
 
+/// Bütün danışanlar JSON'a. Pencere Rust'ta açılır; arayüz yol vermez. Dönüş: kaydedilen
+/// dosyanın adı, vazgeçilirse `None`. Notlar yalnızca `include_notes` ile (spec §5).
 #[tauri::command]
 #[specta::specta]
-pub async fn export_json(
+pub async fn export_clients_json(
+    app: AppHandle,
     store: State<'_, StoreState>,
-    path: String,
-    selection: ExportOptions,
-) -> Result<(), CommandError> {
+    include_notes: bool,
+) -> Result<Option<String>, CommandError> {
     let store = Arc::clone(&store);
-    blocking(move || export_json_file_blocking(&store, &path, &selection)).await
-}
-
-#[tauri::command]
-#[specta::specta]
-pub async fn export_csv(store: State<'_, StoreState>, path: String) -> Result<(), CommandError> {
-    let store = Arc::clone(&store);
-    blocking(move || export_csv_file_blocking(&store, &path)).await
+    blocking(move || {
+        let Some(path) = save_dialog(&app, "JSON", "json", EXPORT_JSON_NAME)? else {
+            return Ok(None);
+        };
+        let options = ExportOptions {
+            include_notes,
+            client_ids: None,
+        };
+        export_json_file_blocking(&store, &path, &options)?;
+        Ok(Some(base_name(&path)))
+    })
+    .await
 }
 
+/// Bütün haritalar CSV'ye (yalnızca dışa aktarma; notlar CSV'de yoktur).
 #[tauri::command]
 #[specta::specta]
-pub async fn import_json(
+pub async fn export_clients_csv(
+    app: AppHandle,
     store: State<'_, StoreState>,
-    path: String,
-) -> Result<ImportReport, CommandError> {
+) -> Result<Option<String>, CommandError> {
     let store = Arc::clone(&store);
-    blocking(move || import_json_file_blocking(&store, &path)).await
+    blocking(move || {
+        let Some(path) = save_dialog(&app, "CSV", "csv", EXPORT_CSV_NAME)? else {
+            return Ok(None);
+        };
+        export_csv_file_blocking(&store, &path)?;
+        Ok(Some(base_name(&path)))
+    })
+    .await
 }
 
+/// JSON içe aktarma (hepsi ya da hiçbiri). Vazgeçilirse `None`.
 #[tauri::command]
 #[specta::specta]
-pub async fn import_jhd(
+pub async fn import_clients_json(
+    app: AppHandle,
     store: State<'_, StoreState>,
-    paths: Vec<String>,
-) -> Result<JhdImportResult, CommandError> {
+) -> Result<Option<ImportResult>, CommandError> {
     let store = Arc::clone(&store);
-    blocking(move || import_jhd_files_blocking(&store, &paths)).await
+    blocking(move || {
+        let picked = app
+            .dialog()
+            .file()
+            .add_filter("JSON", &["json"])
+            .blocking_pick_file();
+        let Some(path) = chosen(picked)? else {
+            return Ok(None);
+        };
+        let report = import_json_file_blocking(&store, &path)?;
+        Ok(Some(ImportResult {
+            file: base_name(&path),
+            report,
+        }))
+    })
+    .await
 }
 
+pub const EXPORT_JSON_NAME: &str = "jyotish-desk-clients.json";
+pub const EXPORT_CSV_NAME: &str = "jyotish-desk-charts.csv";
+
 #[cfg(test)]
 mod tests {
     use super::*;
 
+    fn populated() -> Mutex<Store> {
+        let store = Store::open_in_memory().unwrap();
+        let store = Mutex::new(store);
+        let new = store::NewClient {
+            name: "Ayşe".into(),
+            gender: None,
+            chart: store::NewChart {
+                label: String::new(),
+                kind: store::ChartKind::Natal,
+                birth: store::BirthData {
+                    date: "2000-01-01".into(),
+                    time: Some("17:30".into()),
+                    time_certainty: store::TimeCertainty::Exact,
+                    place: store::Place {
+                        name: "New Delhi".into(),
+                        country: "India".into(),
+                        admin1: "NCT".into(),
+                        lat: 28.6139,
+                        lon: 77.2090,
+                        geoname_id: None,
+                    },
+                    tz: store::TzChoice::Iana("Asia/Kolkata".into()),
+                    fold: None,
+                    notes: String::new(),
+                },
+            },
+        };
+        let (c, _) = lock(&store).unwrap().create_client(&new).unwrap();
+        lock(&store).unwrap().add_note(c.id, None, "gizli").unwrap();
+        store
+    }
+
     #[test]
-    fn file_commands_report_io_errors_per_file() {
+    fn file_errors_name_only_the_file() {
         let store = Mutex::new(Store::open_in_memory().unwrap());
         let r = import_jhd_files_blocking(&store, &["/nonexistent/dir/a.jhd".into()]).unwrap();
         assert_eq!(r.failed.len(), 1);
         assert_eq!(r.failed[0].file, "a.jhd");
         assert_eq!(r.failed[0].error.code, "file_io");
-        let e = import_json_file_blocking(&store, "/nonexistent/x.json").unwrap_err();
+        let e =
+            import_json_file_blocking(&store, Path::new("/nonexistent/dir/x.json")).unwrap_err();
+        assert_eq!(e.code, "file_io");
+        assert_eq!(e.params["file"], "x.json");
+        assert!(!e.params.values().any(|v| v.contains("/nonexistent")));
+    }
+
+    #[test]
+    fn export_writes_atomically_and_round_trips_without_notes_by_default() {
+        let dir = tempfile::tempdir().unwrap();
+        let path = dir.path().join("out.json");
+        std::fs::write(&path, "eski").unwrap();
+        let src = populated();
+        export_json_file_blocking(&src, &path, &ExportOptions::default()).unwrap();
+        let text = std::fs::read_to_string(&path).unwrap();
+        assert!(text.contains("Ayşe") && !text.contains("gizli"));
+        // Geçici dosya kalmaz.
+        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
+        let dst = Mutex::new(Store::open_in_memory().unwrap());
+        let report = import_json_file_blocking(&dst, &path).unwrap();
+        assert_eq!(
+            (
+                report.clients_added,
+                report.charts_added,
+                report.notes_added
+            ),
+            (1, 1, 0)
+        );
+        let csv = dir.path().join("out.csv");
+        export_csv_file_blocking(&src, &csv).unwrap();
+        assert!(std::fs::read_to_string(&csv).unwrap().contains("Ayşe"));
+    }
+
+    #[test]
+    fn a_failed_write_leaves_the_old_file_and_no_temporary() {
+        let dir = tempfile::tempdir().unwrap();
+        // Hedef bir klasör: taşıma başarısız olur.
+        let target = dir.path().join("taken");
+        std::fs::create_dir(&target).unwrap();
+        std::fs::write(target.join("keep"), "x").unwrap();
+        let e = write_atomic(&target, "yeni").unwrap_err();
         assert_eq!(e.code, "file_io");
+        assert_eq!(e.params["file"], "taken");
+        let names: Vec<_> = std::fs::read_dir(dir.path())
+            .unwrap()
+            .map(|e| e.unwrap().file_name())
+            .collect();
+        assert_eq!(names, ["taken"]);
+    }
+
+    /// D2: pencere Rust'ta açılır; web görünümüne dosya penceresi ya da dosya sistemi izni
+    /// verilmez (C2′).
+    #[test]
+    fn the_webview_is_granted_no_dialog_or_file_system_permission() {
+        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/capabilities");
+        assert_eq!(std::fs::read_dir(dir).unwrap().count(), 1);
+        let v: serde_json::Value =
+            serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();
+        assert_eq!(v["permissions"], serde_json::json!(["core:event:default"]));
+    }
+
+    #[test]
+    fn an_import_over_the_byte_cap_is_refused_before_reading() {
+        let dir = tempfile::tempdir().unwrap();
+        let path = dir.path().join("big.json");
+        let f = File::create(&path).unwrap();
+        f.set_len(MAX_IMPORT_BYTES + 1).unwrap();
+        let store = Mutex::new(Store::open_in_memory().unwrap());
+        let e = import_json_file_blocking(&store, &path).unwrap_err();
+        assert_eq!(e.code, "import_too_large");
+        assert_eq!(e.params["file"], "big.json");
+        assert_eq!(e.params["max_mb"], "32");
+        assert!(lock(&store).unwrap().list_clients(10).unwrap().is_empty());
     }
 }
```

`app/ui/api.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -61,6 +61,9 @@ export const api = {
   saveViewState: (chartId: number, viewState: ViewState) => wrap(() => commands.saveViewState(chartId, viewState)),
   loadPanelLayout: () => wrap(() => commands.loadPanelLayout()),
   savePanelLayout: (layout: PanelLayout | null) => wrap(() => commands.savePanelLayout(layout)),
+  exportClientsJson: (includeNotes: boolean) => wrap(() => commands.exportClientsJson(includeNotes)),
+  exportClientsCsv: () => wrap(() => commands.exportClientsCsv()),
+  importClientsJson: () => wrap(() => commands.importClientsJson()),
   dashaChildren: (seed: DashaSeed, path: Graha[]) => wrap(() => commands.dashaChildren(seed, path)),
   searchPlaces: (query: string, limit: number) => wrap(() => commands.searchPlaces(query, limit)),
   previewMoment: (birth: BirthData) => wrap(() => commands.previewMoment(birth)),
```

`app/ui/clients/DataActions.tsx` oluştur:

```tsx
// Kenar çubuğunun veri bölümü (spec §5 içe ve dışa aktarma): bütün danışanlar JSON ya da CSV
// olarak dışa, JSON olarak içe aktarılır. Dosya penceresi Rust'ta açılır; arayüz yol görmez.
// "Notları dahil et" varsayılan kapalıdır (notlar hassas veridir). JHD içe aktarma bu sürümde
// yoktur (gerçek dosyalarla doğrulanacak, Plan 11).
import { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import { useSession } from "../session/store";
import { Dialog } from "../shell/Dialog";
import { useNotices } from "../shell/notices";

export function DataActions() {
  const { t } = useTranslation();
  const session = useSession();
  const { push, dismiss } = useNotices();
  const id = useId();
  const [asking, setAsking] = useState(false);
  const [includeNotes, setIncludeNotes] = useState(false);
  const [busy, setBusy] = useState(false);

  const run = async (work: () => Promise<void>) => {
    setBusy(true);
    dismiss("exchange");
    try {
      await work();
    } finally {
      setBusy(false);
    }
  };

  const exported = (file: string | null) => file !== null && push({ id: "exchange", type: "info", messageKey: "data.exported", params: { file } });

  const exportJson = () =>
    run(async () => {
      setAsking(false);
      const r = await api.exportClientsJson(includeNotes);
      if (r.status === "error") return push({ id: "exchange", type: "error", messageKey: "notice.exportFailed", error: r.error });
      exported(r.data);
    });

  const exportCsv = () =>
    run(async () => {
      const r = await api.exportClientsCsv();
      if (r.status === "error") return push({ id: "exchange", type: "error", messageKey: "notice.exportFailed", error: r.error });
      exported(r.data);
    });

  const importJson = () =>
    run(async () => {
      const r = await api.importClientsJson();
      if (r.status === "error") return push({ id: "exchange", type: "error", messageKey: "notice.importFailed", error: r.error });
      if (r.data === null) return;
      const { file, report } = r.data;
      push({
        id: "exchange",
        type: report.warnings.length > 0 ? "warning" : "info",
        messageKey: report.warnings.length > 0 ? "data.importedWithWarnings" : "data.imported",
        params: {
          file,
          clients: String(report.clients_added),
          charts: String(report.charts_added),
          notes: String(report.notes_added),
          skipped: String(report.skipped),
          warnings: String(report.warnings.length),
        },
      });
      session.touch();
    });

  return (
    <section className="data-actions" aria-labelledby={`${id}-heading`}>
      <h2 id={`${id}-heading`} className="sidebar-heading">
        {t("data.heading")}
      </h2>
      <div className="sidebar-actions">
        <button type="button" disabled={busy} onClick={() => setAsking(true)}>
          {t("data.exportJson")}
        </button>
        <button type="button" disabled={busy} onClick={() => void exportCsv()}>
          {t("data.exportCsv")}
        </button>
        <button type="button" disabled={busy} onClick={() => void importJson()}>
          {t("data.importJson")}
        </button>
      </div>
      {asking && (
        <Dialog title={t("data.exportTitle")} onClose={() => setAsking(false)}>
          <p>{t("data.exportIntro")}</p>
          <label className="checkbox-row">
            <input type="checkbox" checked={includeNotes} aria-describedby={`${id}-notes-hint`} onChange={(e) => setIncludeNotes(e.target.checked)} />
            {t("data.includeNotes")}
          </label>
          <p id={`${id}-notes-hint`} className="sidebar-hint">
            {t("data.includeNotesHint")}
          </p>
          <div className="dialog-actions">
            <button type="button" onClick={() => setAsking(false)}>
              {t("clients.cancel")}
            </button>
            <button type="button" data-autofocus onClick={() => void exportJson()}>
              {t("data.chooseFile")}
            </button>
          </div>
        </Dialog>
      )}
    </section>
  );
}
```

`app/ui/clients/Sidebar.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/clients/Sidebar.tsx
+++ b/app/ui/clients/Sidebar.tsx
@@ -1,9 +1,10 @@
 // Sol kenar çubuğu (UI spec §4): açık danışanın haritaları ve etiketleri, danışan listesi,
-// arama, etiket süzgeci ve son açılanlar. `Cmd+\` ile açılıp kapanır (Shell).
+// arama, etiket süzgeci, son açılanlar ve veri içe/dışa aktarma. `Cmd+\` ile açılıp kapanır (Shell).
 import { useTranslation } from "react-i18next";
 import { useSession } from "../session/store";
 import { ClientList } from "./ClientList";
 import "./clients.css";
+import { DataActions } from "./DataActions";
 import { OpenClient } from "./OpenClient";
 
 export const SIDEBAR_ID = "client-sidebar";
@@ -15,6 +16,7 @@ export function Sidebar({ onAddChart }: { onAddChart(): void }) {
     <aside id={SIDEBAR_ID} className="sidebar no-print" aria-label={t("clients.region")}>
       {opened && <OpenClient key={opened.client.id} opened={opened} onAddChart={onAddChart} />}
       <ClientList />
+      <DataActions />
     </aside>
   );
 }
```

`app/ui/clients/clients.css` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/clients/clients.css
+++ b/app/ui/clients/clients.css
@@ -92,3 +92,12 @@
 .button-danger {
   border: 1px solid var(--danger);
 }
+.checkbox-row {
+  display: flex;
+  align-items: center;
+  gap: 6px;
+}
+.data-actions {
+  border-top: 1px solid var(--line);
+  padding-top: 8px;
+}
```

`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -118,6 +118,12 @@ export const en: Messages = {
     atlas_sqlite: "The place atlas could not be read ({{detail}}).",
     atlas_io: "The place atlas file could not be read ({{detail}}).",
     atlas_schema: "Place atlas version mismatch ({{found}}, expected {{expected}}).",
+    file_io: "The file could not be read or written ({{file}}: {{detail}}).",
+    dialog_path: "The chosen location is not a file path.",
+    import_too_large: "“{{file}}” is too large; at most {{max_mb}} MB can be imported.",
+    import_format: "The file is not a Jyotish Desk export or is damaged ({{detail}}).",
+    import_version: "The file is newer than this version (version {{found}}, supported {{supported}}). Update the app.",
+    import_record: "A record in the file is invalid ({{item}}: {{field}}).",
   },
   layout: {
     picker: "Layout",
@@ -170,6 +176,7 @@ export const en: Messages = {
     dismiss: "Dismiss",
     error: "Error",
     warning: "Warning",
+    info: "Info",
     restoreFailed: "The last session could not be opened: {{message}}",
     saveFailed: "The view could not be saved: {{message}}",
     layoutLoadFailed: "The saved layout could not be read: {{message}} View changes in this session will not be saved; the saved layout is left as it is.",
@@ -178,6 +185,8 @@ export const en: Messages = {
     tagsFailed: "Tags could not be saved: {{message}}",
     noteFailed: "The note could not be added: {{message}}",
     noteSaveLost: "A note could not be saved while its panel was closing; the last changes were not written: {{message}}",
+    exportFailed: "Export failed: {{message}}",
+    importFailed: "Import failed; nothing was added: {{message}}",
   },
   storeWarning: {
     LastChartMissing: "The last opened chart was deleted; the most recently updated chart was opened.",
@@ -334,6 +343,20 @@ export const en: Messages = {
     loadFailed: "Notes could not be read: {{message}}",
     retry: "Retry",
   },
+  data: {
+    heading: "Data",
+    exportJson: "Export as JSON…",
+    exportCsv: "Export as CSV…",
+    importJson: "Import JSON…",
+    exportTitle: "Export as JSON",
+    exportIntro: "All clients, their charts, calculation settings and tags are written to one file.",
+    includeNotes: "Include notes",
+    includeNotesHint: "Notes are sensitive; be careful who you share the file with.",
+    chooseFile: "Choose file and save…",
+    exported: "“{{file}}” saved.",
+    imported: "“{{file}}” imported: {{clients}} clients, {{charts}} charts, {{notes}} notes added; {{skipped}} records were already there.",
+    importedWithWarnings: "“{{file}}” imported: {{clients}} clients, {{charts}} charts, {{notes}} notes added; {{skipped}} records were already there. The calculation settings of {{warnings}} charts could not be read and the defaults were written.",
+  },
   empty: {
     title: "No chart is open",
     body: "Start by entering the client's birth data.",
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -118,6 +118,12 @@ export const fr: Messages = {
     atlas_sqlite: "L’atlas des lieux est illisible ({{detail}}).",
     atlas_io: "Le fichier de l’atlas est illisible ({{detail}}).",
     atlas_schema: "Version de l’atlas incompatible ({{found}}, attendue {{expected}}).",
+    file_io: "Le fichier n’a pas pu être lu ou écrit ({{file}} : {{detail}}).",
+    dialog_path: "L’emplacement choisi n’est pas un chemin de fichier.",
+    import_too_large: "« {{file}} » est trop volumineux ; au plus {{max_mb}} Mo peuvent être importés.",
+    import_format: "Le fichier n’est pas un export Jyotish Desk ou il est endommagé ({{detail}}).",
+    import_version: "Le fichier est plus récent que cette version (version {{found}}, prise en charge {{supported}}). Mettez l’application à jour.",
+    import_record: "Un enregistrement du fichier est invalide ({{item}} : {{field}}).",
   },
   layout: {
     picker: "Disposition",
@@ -170,6 +176,7 @@ export const fr: Messages = {
     dismiss: "Fermer",
     error: "Erreur",
     warning: "Avertissement",
+    info: "Information",
     restoreFailed: "La dernière session n’a pas pu être ouverte : {{message}}",
     saveFailed: "La vue n’a pas pu être enregistrée : {{message}}",
     layoutLoadFailed: "La disposition enregistrée n’a pas pu être lue : {{message}} Les changements de vue de cette session ne seront pas enregistrés ; la disposition enregistrée reste intacte.",
@@ -178,6 +185,8 @@ export const fr: Messages = {
     tagsFailed: "Les étiquettes n’ont pas pu être enregistrées : {{message}}",
     noteFailed: "La note n’a pas pu être ajoutée : {{message}}",
     noteSaveLost: "Une note n’a pas pu être enregistrée à la fermeture du panneau ; les dernières modifications n’ont pas été écrites : {{message}}",
+    exportFailed: "Échec de l’export : {{message}}",
+    importFailed: "Échec de l’import ; rien n’a été ajouté : {{message}}",
   },
   storeWarning: {
     LastChartMissing: "La dernière carte ouverte a été supprimée ; la plus récemment modifiée a été ouverte.",
@@ -334,6 +343,20 @@ export const fr: Messages = {
     loadFailed: "Les notes n’ont pas pu être lues : {{message}}",
     retry: "Réessayer",
   },
+  data: {
+    heading: "Données",
+    exportJson: "Exporter en JSON…",
+    exportCsv: "Exporter en CSV…",
+    importJson: "Importer du JSON…",
+    exportTitle: "Exporter en JSON",
+    exportIntro: "Tous les clients, leurs cartes, réglages de calcul et étiquettes sont écrits dans un seul fichier.",
+    includeNotes: "Inclure les notes",
+    includeNotesHint: "Les notes sont sensibles ; faites attention à qui vous partagez le fichier.",
+    chooseFile: "Choisir le fichier et enregistrer…",
+    exported: "« {{file}} » enregistré.",
+    imported: "« {{file}} » importé : {{clients}} clients, {{charts}} cartes, {{notes}} notes ajoutés ; {{skipped}} enregistrements existaient déjà.",
+    importedWithWarnings: "« {{file}} » importé : {{clients}} clients, {{charts}} cartes, {{notes}} notes ajoutés ; {{skipped}} enregistrements existaient déjà. Les réglages de calcul de {{warnings}} cartes étaient illisibles ; les valeurs par défaut ont été écrites.",
+  },
   empty: {
     title: "Aucune carte ouverte",
     body: "Commencez par saisir les données de naissance du consultant.",
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -117,6 +117,12 @@ export const tr = {
     atlas_sqlite: "Yer atlası okunamadı ({{detail}}).",
     atlas_io: "Yer atlası dosyası okunamadı ({{detail}}).",
     atlas_schema: "Yer atlası sürümü uyuşmuyor ({{found}}, beklenen {{expected}}).",
+    file_io: "Dosya okunamadı ya da yazılamadı ({{file}}: {{detail}}).",
+    dialog_path: "Seçilen konum bir dosya yolu değil.",
+    import_too_large: "“{{file}}” çok büyük; en çok {{max_mb}} MB içe aktarılabilir.",
+    import_format: "Dosya bir Jyotish Desk dışa aktarımı değil ya da bozuk ({{detail}}).",
+    import_version: "Dosya bu sürümden yeni (sürüm {{found}}, desteklenen {{supported}}). Uygulamayı güncelleyin.",
+    import_record: "Dosyadaki bir kayıt geçersiz ({{item}}: {{field}}).",
   },
   layout: {
     picker: "Düzen",
@@ -169,6 +175,7 @@ export const tr = {
     dismiss: "Kapat",
     error: "Hata",
     warning: "Uyarı",
+    info: "Bilgi",
     restoreFailed: "Son oturum açılamadı: {{message}}",
     saveFailed: "Görünüm kaydedilemedi: {{message}}",
     layoutLoadFailed: "Kayıtlı düzen okunamadı: {{message}} Bu oturumdaki görünüm değişiklikleri kaydedilmeyecek; kayıtlı düzen olduğu gibi duruyor.",
@@ -177,6 +184,8 @@ export const tr = {
     tagsFailed: "Etiketler kaydedilemedi: {{message}}",
     noteFailed: "Not eklenemedi: {{message}}",
     noteSaveLost: "Panel kapanırken bir not kaydedilemedi; son değişiklikler yazılmadı: {{message}}",
+    exportFailed: "Dışa aktarılamadı: {{message}}",
+    importFailed: "İçe aktarılamadı; hiçbir kayıt eklenmedi: {{message}}",
   },
   storeWarning: {
     LastChartMissing: "Son açılan harita silinmiş; en son güncellenen harita açıldı.",
@@ -333,6 +342,20 @@ export const tr = {
     loadFailed: "Notlar okunamadı: {{message}}",
     retry: "Yeniden dene",
   },
+  data: {
+    heading: "Veri",
+    exportJson: "JSON olarak dışa aktar…",
+    exportCsv: "CSV olarak dışa aktar…",
+    importJson: "JSON içe aktar…",
+    exportTitle: "JSON olarak dışa aktar",
+    exportIntro: "Bütün danışanlar, haritaları, hesap ayarları ve etiketleri tek dosyaya yazılır.",
+    includeNotes: "Notları dahil et",
+    includeNotesHint: "Notlar hassas veridir; dosyayı kiminle paylaştığınıza dikkat edin.",
+    chooseFile: "Dosya seç ve kaydet…",
+    exported: "“{{file}}” kaydedildi.",
+    imported: "“{{file}}” içe aktarıldı: {{clients}} danışan, {{charts}} harita, {{notes}} not eklendi; {{skipped}} kayıt zaten vardı.",
+    importedWithWarnings: "“{{file}}” içe aktarıldı: {{clients}} danışan, {{charts}} harita, {{notes}} not eklendi; {{skipped}} kayıt zaten vardı. {{warnings}} haritanın hesap ayarı okunamadı, varsayılan ayar yazıldı.",
+  },
   empty: {
     title: "Açık harita yok",
     body: "Danışanın doğum bilgisini girerek başlayın.",
```

`app/ui/shell/notices.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/notices.tsx
+++ b/app/ui/shell/notices.tsx
@@ -8,7 +8,7 @@ import { errorMessage } from "../i18n/errors";
 
 export type Notice =
   | { id: string; type: "error"; messageKey: string; error: CommandError }
-  | { id: string; type: "warning"; messageKey: string; params: Record<string, string> };
+  | { id: string; type: "warning" | "info"; messageKey: string; params: Record<string, string> };
 
 export interface Notices {
   notices: Notice[];
```

`app/ui/shell/shell.css` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/shell.css
+++ b/app/ui/shell/shell.css
@@ -134,6 +134,10 @@
   border-left: 3px solid var(--warn);
   padding-left: 8px;
 }
+.notice--info {
+  border-left: 3px solid var(--line);
+  padding-left: 8px;
+}
 .notice--error {
   border-left: 3px solid var(--danger);
   padding-left: 8px;
```

`crates/store/src/exchange.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/crates/store/src/exchange.rs
+++ b/crates/store/src/exchange.rs
@@ -133,6 +133,14 @@ fn import_resolved(p: &ParsedBirth, r: &ResolvedRecord) -> Option<ResolvedRecord
     }
     let other_tzdb =
         matches!(p.canonical.tz, TzChoice::Iana(_)) && r.tzdb_version != atlas::tzdb_version();
+    // Dosyadan olduğu gibi saklanan alanlar sınırlı: kısaltma ve tzdb sürümü kısa metin,
+    // ofset gerçek saat dilimlerinin aralığında.
+    let bounded = r.abbrev.chars().count() <= MAX_IMPORT_LABEL_CHARS
+        && r.tzdb_version.chars().count() <= MAX_IMPORT_LABEL_CHARS
+        && r.offset_secs.unsigned_abs() <= MAX_OFFSET_SECS;
+    if !bounded {
+        return None;
+    }
     let jd = stored.as_millisecond() as f64 / 86_400_000.0 + 2_440_587.5;
     if !other_tzdb || !JD_UT_RANGE.contains(&r.jd_ut) || (r.jd_ut - jd).abs() * 86_400.0 > 1.0 {
         return None;
@@ -143,6 +151,11 @@ fn import_resolved(p: &ParsedBirth, r: &ResolvedRecord) -> Option<ResolvedRecord
     })
 }
 
+/// İçe alınan çözülmüş anın `abbrev` ve `tzdb_version` alanlarının sınırı (karakter).
+const MAX_IMPORT_LABEL_CHARS: usize = 32;
+/// İçe alınan çözülmüş anın ofset sınırı: ±18 saat.
+const MAX_OFFSET_SECS: u32 = 18 * 3600;
+
 /// Geçerli bir `format_ts` zaman damgası değilse `now`.
 fn ts_or(ts: &str, now: &str) -> String {
     match ts.parse::<atlas::Timestamp>() {
@@ -357,6 +370,12 @@ impl Store {
                     report.skipped += 1;
                     continue;
                 }
+                if n.body.len() > crate::notes::MAX_NOTE_BYTES {
+                    return Err(StoreError::ImportRecord {
+                        item: note_uuid,
+                        field: "body",
+                    });
+                }
                 let chart_id: Option<u32> = match n.chart_uuid.as_deref().map(Uuid::parse_str) {
                     Some(Ok(u)) => tx
                         .query_row(
@@ -599,6 +618,27 @@ mod tests {
         let o = dst.open_client(c.id, None).unwrap();
         assert_eq!(o.chart.resolved.utc, "2000-01-01T11:00:00Z");
         assert!(matches!(o.warnings[..], [StoreWarning::TzdbChanged { .. }]));
+        // Saklanan alanlar sınırlı: uzun kısaltma, uzun tzdb sürümü, ±18 saati aşan ofset.
+        for (field, value) in [
+            ("abbrev", json!("X".repeat(33))),
+            ("tzdb_version", json!("1".repeat(33))),
+            ("offset_secs", json!(18 * 3600 + 1)),
+        ] {
+            let mut w = v.clone();
+            w["clients"][0]["charts"][0]["resolved"][field] = value;
+            assert_eq!(rejected(&w), "resolved", "{field}");
+        }
+        // Not gövdesi sınırı içe aktarmada da geçerlidir.
+        let mut v = serde_json::from_str::<Value>(
+            &src.export_json(&ExportOptions {
+                include_notes: true,
+                client_ids: None,
+            })
+            .unwrap(),
+        )
+        .unwrap();
+        v["clients"][0]["notes"][0]["body"] = json!("x".repeat(crate::notes::MAX_NOTE_BYTES + 1));
+        assert_eq!(rejected(&v), "body");
         // Haritasız danışan oluşmaz.
         let mut v = exported.clone();
         v["clients"][0]["charts"] = json!([]);
```

- [ ] **Step 4: Bağımlılık, bağlamalar ve MSRV**

```bash
cargo tree -p jyotish-desk -i rfd
cargo deny check
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
cargo test -p jyotish-desk --test bindings
grep -n "exportClientsJson\|exportClientsCsv\|importClientsJson\|export type ImportResult" app/ui/bindings.ts
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store -p jyotish-desk --all-targets
```
Expected: `rfd v0.17.2` yalnızca `tauri-plugin-dialog v2.8.4` altında; `advisories ok, bans ok, licenses ok, sources ok`; ikinci `bindings` çalıştırması `1 passed`; `grep` dört satır bulur, `exportJson`/`importJhd` yoktur; MSRV `Finished`. `Cargo.lock` ve `bindings.ts` elle düzenlenmez.

- [ ] **Step 5: Testlerin geçtiğini gör**

```bash
cargo test -p store --lib
cargo test -p jyotish-desk
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
pnpm --dir app test
pnpm --dir app build
```
Expected: `store` lib `71 passed`; `jyotish-desk` lib `test result: ok. 19 passed` (+5: üç dosya komutu testi, kesme testi ve yetki testi), `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2; fmt, clippy çıktısız; `Test Files  42 passed (42)`, `Tests  346 passed (346)`; `build` `✓ built`.

- [ ] **Step 6: Commit**

```bash
git add Cargo.lock \
  app/src-tauri/Cargo.toml \
  app/src-tauri/src/client_commands.rs \
  app/src-tauri/src/lib.rs \
  app/src-tauri/src/store_commands.rs \
  app/ui/api.test.ts \
  app/ui/api.ts \
  app/ui/bindings.ts \
  app/ui/clients/DataActions.test.tsx \
  app/ui/clients/DataActions.tsx \
  app/ui/clients/Sidebar.tsx \
  app/ui/clients/clients.css \
  app/ui/i18n/en.ts \
  app/ui/i18n/fr.ts \
  app/ui/i18n/i18n.test.ts \
  app/ui/i18n/tr.ts \
  app/ui/shell/notices.tsx \
  app/ui/shell/shell.css \
  crates/store/src/exchange.rs
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git diff --cached --name-only | grep -iE 'k(a|ā|aa)la|\bcala\b|^\.superpowers/'   # boş olmalı
git commit -m "feat: JSON/CSV export and JSON import through Rust-side file dialogs with size, write and field bounds

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: commit oluşur; iki tarama da çıktısız. Commit mesajında yasaklı sözcük yoktur (C13).

---

### Task 11: Uçtan uca: `Cmd+K` ile formsuz danışan açma (spec §8), kenar çubuğu ve notlar; CI'da Playwright izleri (M6-4)

Spec §8'in kabul ölçütü: `Cmd+K` → ad → `Enter` ile danışanın haritası açılır ve hiçbir form ya da onay penceresi görünmez. IPC taklidine (`e2e/ipc.ts`) M6 komutları eklenir: ikinci danışan aynı hesaplanmış haritanın kimliği değiştirilmiş kopyasıdır (arayüz hesap yapmaz, C4), notlar bellek içidir. İkinci test `⌘\`'ı odak not kutusundayken basar: ⌘ kısayolları yazı alanından çalışır (C16). Kenar çubuğu artık varsayılan açık olduğundan görsel testler ekran görüntüsünden önce `⌘\` ile kapatır; temel PNG'ler değişmez (C11). CI'da e2e başarısız olursa `app/test-results/` (izler, `retain-on-failure`) 7 gün saklanır; yalnızca fikstür verisi içerir.

**Files:**
- Modify: `.github/workflows/ci.yml`
- Test: `app/e2e/desk.spec.ts`
- Modify: `app/e2e/ipc.ts`
- Test: `app/e2e/visual.spec.ts`

**Interfaces:**
- Consumes: bütün önceki görevler; `FIXTURE.opened`.
- Produces: `mockIpc` yeni yanıtları `open_client`, `list_clients`, `list_clients_by_tag`, `search_clients`, `list_tags`, `list_charts`, `set_chart_tags`, `list_notes`, `add_note`, `update_note`, `delete_note`; iki yeni e2e testi; CI adımı `actions/upload-artifact@v4` (`if: failure()`).

- [ ] **Step 1: Önce testleri yaz**

`app/e2e/desk.spec.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/e2e/desk.spec.ts
+++ b/app/e2e/desk.spec.ts
@@ -70,3 +70,40 @@ test("dasha tree by keyboard loads level 4 on demand", async ({ page }) => {
   await expect(page.locator(":focus")).toHaveAttribute("aria-level", "4");
   expect(await ipcCalls(page, "dasha_children")).toHaveLength(1);
 });
+
+test("spec §8: Cmd+K, a name and Enter open that client's chart with no form or confirmation", async ({ page }) => {
+  await mockIpc(page, { session: true });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  await page.keyboard.press("Meta+KeyK");
+  const palette = page.getByRole("dialog", { name: "Komut paleti" });
+  await expect(palette.getByRole("combobox")).toBeFocused();
+  await page.keyboard.type("ayşe");
+  await expect(palette.getByRole("option").first()).toContainText("Ayşe Yılmaz");
+  await page.keyboard.press("Enter");
+  await expect(page.getByRole("dialog")).toHaveCount(0);
+  await expect(page.getByRole("heading", { level: 1, name: "Ayşe Yılmaz" })).toBeVisible();
+  await expect(page.getByRole("group", { name: "D1 haritası" })).toBeVisible();
+  expect(await ipcCalls(page, "open_client")).toEqual([{ clientId: 2, chartId: null }]);
+});
+
+test("the sidebar opens a recent client, a new note is saved as it is typed, and Cmd+\\ from the note hides the sidebar", async ({ page }) => {
+  await mockIpc(page, { session: true });
+  await page.goto("/");
+  const sidebar = page.getByRole("complementary", { name: "Danışanlar" });
+  await sidebar.getByRole("button", { name: "Ayşe Yılmaz" }).click();
+  await expect(page.getByRole("heading", { level: 1, name: "Ayşe Yılmaz" })).toBeVisible();
+  await expect(sidebar.getByRole("button", { name: "Ayşe Yılmaz" })).toHaveAttribute("aria-current", "true");
+  const notes = page.getByRole("region", { name: "Notlar" });
+  await notes.getByRole("button", { name: "Yeni not" }).click();
+  const box = notes.getByRole("textbox");
+  await expect(box).toBeFocused();
+  await page.keyboard.type("Satürn dönüşü konuşuldu");
+  await page.keyboard.press("Tab");
+  await expect.poll(async () => ((await ipcCalls(page, "update_note")).at(-1) as { body?: string } | undefined)?.body).toBe("Satürn dönüşü konuşuldu");
+  // ⌘ kısayolları not kutusunun içinden de çalışır (Plan 10 incelemesi MF1).
+  await box.focus();
+  await page.keyboard.press("Meta+Backslash");
+  await expect(sidebar).toHaveCount(0);
+  await expect(page.getByRole("button", { name: "Danışanlar" })).toHaveAttribute("aria-expanded", "false");
+});
```

`app/e2e/visual.spec.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/e2e/visual.spec.ts
+++ b/app/e2e/visual.spec.ts
@@ -10,6 +10,9 @@ for (const theme of ["light", "dark"] as const) {
       await mockIpc(page, { session: true });
       await page.addInitScript((t) => localStorage.setItem("jyotish-desk.theme.v1", t), theme);
       await page.goto("/");
+      // Kenar çubuğu kapalı: temel görüntüler haritanın kendisidir, pano genişliğine bağlı değil.
+      await page.keyboard.press("Meta+Backslash");
+      await expect(page.getByRole("complementary", { name: "Danışanlar" })).toHaveCount(0);
       const chart = page.getByRole("region", { name: "Harita D1", exact: true });
       await chart.getByRole("combobox", { name: "Biçim" }).selectOption(style);
       await expect(chart.locator("svg.chart")).toHaveAttribute("data-style", style);
```

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app e2e
```
Expected: iki yeni test FAIL (`ipc.ts` taklidi henüz yok: `Ayşe Yılmaz` seçeneği ve düğmesi bulunamaz). `visual.spec.ts` değişikliği olmadan `pnpm --dir app e2e:visual` 6 FAIL verir (`Expected an image 690px by 400px, received 558px by 400px`: Task 7'den beri açık kenar çubuğu panoyu daraltır); değişiklikle eski temeller geçer.

- [ ] **Step 3: Uygulamayı yaz**

`.github/workflows/ci.yml` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/.github/workflows/ci.yml
+++ b/.github/workflows/ci.yml
@@ -29,6 +29,14 @@ jobs:
           key: playwright-webkit-${{ runner.os }}-${{ hashFiles('app/pnpm-lock.yaml') }}
       - run: pnpm --dir app exec playwright install webkit
       - run: pnpm --dir app e2e
+      # Başarısız e2e'nin Playwright izleri (trace: retain-on-failure) indirilebilir kalır (M6-4).
+      - if: failure()
+        uses: actions/upload-artifact@v4
+        with:
+          name: playwright-traces
+          path: app/test-results/
+          if-no-files-found: ignore
+          retention-days: 7
       - uses: dtolnay/rust-toolchain@stable
         with:
           components: rustfmt, clippy
```

`app/e2e/ipc.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -16,7 +16,46 @@ export async function mockIpc(page: Page, { session }: IpcOptions) {
     ({ fixture, session }) => {
       const calls: { cmd: string; args: unknown }[] = [];
       Object.assign(window, { __ipcCalls: calls });
+      // İkinci danışan: aynı hesaplanmış harita, başka kimlik ve ad (arayüz hesap yapmaz).
+      const ayse = structuredClone(fixture.opened);
+      ayse.client = { ...ayse.client, id: 2, uuid: "00000000-0000-4000-8000-000000000003", name: "Ayşe Yılmaz", last_chart_id: 20 };
+      ayse.chart_record = { ...ayse.chart_record, id: 20, client_id: 2, uuid: "00000000-0000-4000-8000-000000000004" };
+      ayse.recent_notes = [];
+      const opened: Record<number, typeof ayse> = { 1: fixture.opened, 2: ayse };
+      const clients = () => [opened[2].client, opened[1].client];
+      // Notlar bellek içi; kimlik ve zaman sırayla artar.
+      const notes: { id: number; uuid: string; client_id: number; chart_id: number | null; body: string; created_at: string; updated_at: string }[] = [];
+      let noteSeq = 0;
+      const stamp = () => new Date(Date.UTC(2026, 10, 25, 10, 0, noteSeq)).toISOString();
       const respond: Record<string, (args: Record<string, unknown>) => unknown> = {
+        open_client: (args) => opened[args.clientId as number],
+        list_clients: () => clients(),
+        list_clients_by_tag: () => [],
+        search_clients: (args) => {
+          const q = String(args.query).toLocaleLowerCase("tr");
+          return clients()
+            .filter((c) => c.name.toLocaleLowerCase("tr").includes(q))
+            .map((client) => ({ client, matched: "Name", snippet: null }));
+        },
+        list_tags: () => [],
+        list_charts: (args) => [opened[args.clientId as number].chart_record],
+        set_chart_tags: (args) => args.tags,
+        list_notes: (args) => notes.filter((n) => n.client_id === args.clientId).reverse(),
+        add_note: (args) => {
+          noteSeq += 1;
+          const n = { id: noteSeq, uuid: `note-${noteSeq}`, client_id: args.clientId as number, chart_id: args.chartId as number | null, body: String(args.body), created_at: stamp(), updated_at: stamp() };
+          notes.push(n);
+          return n;
+        },
+        update_note: (args) => {
+          const n = notes.find((x) => x.id === args.noteId)!;
+          n.body = String(args.body);
+          return n;
+        },
+        delete_note: (args) => {
+          notes.splice(notes.findIndex((x) => x.id === args.noteId), 1);
+          return null;
+        },
         restore_session: () => (session ? fixture.opened : null),
         load_panel_layout: () => null,
         save_panel_layout: () => null,
```

- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app e2e
pnpm --dir app e2e
pnpm --dir app e2e:visual
pnpm --dir app build
git status --short app/e2e/visual.spec.ts-snapshots
rm -rf app/test-results
```
Expected: e2e iki kez `6 passed`; görsel `6 passed` (temel PNG'ler yeniden üretilmez, `git status` boş); `build` `✓ built` (`tsc -p e2e` dahil).

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/ci.yml \
  app/e2e/desk.spec.ts \
  app/e2e/ipc.ts \
  app/e2e/visual.spec.ts
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git diff --cached --name-only | grep -iE 'k(a|ā|aa)la|\bcala\b|^\.superpowers/'   # boş olmalı
git commit -m "test(e2e): Cmd+K opens a client with no form, sidebar and notes flows; upload Playwright traces on CI failure

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: commit oluşur; iki tarama da çıktısız. Commit mesajında yasaklı sözcük yoktur (C13).

---

### Task 12: Son doğrulama

Bütün kapılar temiz ağaçta. Kod yazılmaz; bir adım beklenen çıktıyı vermezse ilgili görevin adımına dönülür.

**Files:** yok (yalnızca denetim).

**Interfaces:** —

- [ ] **Step 1: Rust kapıları**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p store --features specta --all-targets -- -D warnings
cargo test --workspace
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store -p jyotish-desk --all-targets
cargo deny check
```
Expected: fmt ve iki clippy çıktısız; `cargo test --workspace` her ikili `ok`: atlas 6+10+12+8+3, ephemeris 1+7+1+1+1+7+5+15, `jyotish-core` lib 108, golden 8, `jyotish-desk` lib 19, `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2, `store` lib 71; 0 failed. MSRV `Finished`; deny `advisories ok, bans ok, licenses ok, sources ok`.

- [ ] **Step 2: Üretilen dosyalar güncel**

```bash
cargo test -p jyotish-desk --test bindings --test ui_fixture
git status --short
```
Expected: `bindings` 1 ve `ui_fixture` 2 geçer; `git status` boş.

- [ ] **Step 3: Arayüz kapıları**

```bash
pnpm --dir app test
pnpm --dir app build
pnpm --dir app e2e
pnpm --dir app e2e:visual
rm -rf app/test-results
```
Expected: `Test Files  42 passed (42)`, `Tests  346 passed (346)`; `build` `✓ built`; e2e `6 passed`; görsel `6 passed` (yerel).

- [ ] **Step 4: Kurallar**

```bash
git diff 0424a42..HEAD -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log --format=%B 0424a42..HEAD | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --name-only 0424a42..HEAD | grep -iE 'k(a|ā|aa)la|\bcala\b'
git ls-files | grep '^\.superpowers/'
git log --format=%B 0424a42..HEAD | grep -c 'Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>'
grep -rn "console\." app/ui --include='*.ts' --include='*.tsx' | grep -v "\.test\.\|no-console"
grep -n '"export_json"\|"export_csv"\|"import_json"\|"import_jhd"\|path: string' app/ui/bindings.ts
grep -rn '"dialog:\|"fs:' app/src-tauri/capabilities
grep -rn 'tauri_plugin_fs' app/src-tauri/src
grep -rnE '\b(alert|confirm|prompt)\(' app/ui --include='*.ts' --include='*.tsx' | grep -v '\.test\.'
```
Expected: ilk dört komut çıktısız; trailer sayısı `11`; `console.` yalnızca Plan 04'ün izinli tema/dil uyarılarında (izin listesi değişmedi, `no-console.test.ts` yeşil); `bindings.ts` `grep`'i çıktısız (yol alan komut yok, JHD yok); son üç `grep` çıktısız (C2′: yetenek dosyasında `dialog:`/`fs:` izni yok, `tauri_plugin_fs` başlatılmıyor; C15: `alert`/`confirm`/`prompt` yok).

- [ ] **Step 5: Elle denetim: M5 duman testi ile tek kapı (macOS, `pnpm --dir app tauri dev`)**

```bash
pnpm --dir app tauri dev
```
Dosya pencereleri, WKWebView tuş davranışı ve uygulamadan çıkış jsdom ve Playwright'ta denetlenemez. Bu adım Plan 09 Task 14 adım 5 ile birlikte **tek kapıdır** (Ön koşul): Plan 10 push'undan önce gerçek uygulamada bir kişi tarafından çalıştırılır. Sırayla:

Plan 09 Task 14 adım 5 (M5; Plan 09'daki listenin yedi maddesi, M1–M7):

- **M1** Boş masada "Yeni harita" görünür; `⌘N` formu açar ve odak "Danışan adı"ndadır.
- **M2** Formu doldur (New Delhi araması, 2000-01-01 17:30) ve Kaydet: harita, tablo ve dasha ağacı dolar.
- **M3** `⌘E` düzenleme formunu açar; odak pencerenin içindedir (ad alanı kilitli), Esc tıklamadan kapatır.
- **M4** `⌘1`, `⌘2`, `⌘3` düzenleri değiştirir; macOS menüsü bu tuşları (ve `⌘N`/`⌘E`'yi) yutmaz.
- **M5** `?` kısayol yardımını açar, Esc kapatır ve odak tetikleyene döner.
- **M6** Uygulamayı kapatıp açınca aynı harita, düzen ve dasha yolu geri gelir.
- **M7** Terminalde kişisel veri içeren bir satır yoktur.

Plan 10 (1–12):

1. "JSON olarak dışa aktar…" → "Notları dahil et" kapalı → kaydet; dosyada not yok, klasörde `.…tmp` dosyası kalmamış.
2. Aynı dosyanın üzerine yeniden kaydet (macOS "değiştir" sorusu).
3. "JSON içe aktar…" → aynı dosya → bildirim "0 danışan … zaten vardı".
4. Pencerede vazgeç → bildirim yok.
5. `Cmd+K` → ad → `Enter` → form yok.
6. `Cmd+\` iki kez; odak kaybolmaz.
7. Not yaz, `Cmd+2` ile notsuz düzene geç, `Cmd+1` ile dön: metin durur.
8. VoiceOver: palette ↑/↓ etkin seçeneği okur; silme penceresi "Vazgeç"te açılır.
9. Bir nota yaz, durmadan `⌘Q`; yeniden aç: son yazılan metin duruyor. Aynısı dasha yolu ve düzen için.
10. İmleç not kutusundayken ve kenar çubuğu arama kutusundayken `⌘K`, `⌘\`, `⌘⇧N`, `⌘2` çalışır.
11. `⌘K` ve `⌘\` macOS menüsüne takılmadan sayfaya ulaşır; dosya penceresi açıkken uygulama penceresi kullanılabilir durumda kalmıyorsa not edilir.
12. Bozuk bir JSON içe aktar: bildirimde yalnızca dosya adı var; `tauri dev` terminalinde kişisel veri yok.

Expected: M1–M7 ve 1–12 beklendiği gibi ve her sonuç `progress.md`'ye yazılmış. M5'e ait bir hata ayrı commit'le (`fix(m5): …`) push'tan önce düzeltilir; Plan 10'a ait bir hata kendi görevine döner. Madde 9'da metin, dasha yolu ya da düzen kaybolursa push'tan önce önceden onaylı `CloseRequested` düzeltmesi iner (Rust: kapanmayı engelle, boşaltma olayı yay, arayüzün onayını en çok 1 s bekle, sonra kapat). O zamana kadar Plan 10 push edilmez ve Plan 11 Task 1 başlamaz.

---

## Başarı ölçütleri

- Kenar çubuğu danışanları arar (ad, etiket, not), etiketle süzer, son açılanları gösterir ve açar; açık danışanın haritaları listelenir, harita eklenir, harita ve danışan onayla silinir, etiketler kaydedilir. `Cmd+\` kenar çubuğunu açıp kapatır.
- `Cmd+K` → ad → `Enter` danışanın son haritasını formsuz ve onaysız açar (spec §8); palet harita, düzen, varga ve komutlara da ulaşır.
- Notlar paneli tarih damgalı notları gösterir, ekler, otomatik ve sıralı kaydeder (durmadan yazarken de en geç 2 s'de), onayla siler; kapanırken ya da sayfa gizlenirken yazılan metin kaybolmaz, kapandıktan sonra başarısız olan kayıt duyurulur.
- JSON/CSV dışa ve JSON içe aktarma Rust tarafı pencerelerle çalışır; 32 MiB üstü dosya okunmaz; dışa aktarma atomiktir; notlar yalnızca istenirse eklenir; JHD erişilemez.
- Görünüm kayıtları sıralıdır; okunamayan düzen ezilmez; kayıt hatası sonraki başarılı kayıtta kalkar; ⌘ kısayolları seçim kutularından ve yazı alanlarından da çalışır.
- Store'da üç yazım tek işlemdir, zaman damgaları kesin artandır, etiketler NFC'dir.
- Task 12'nin bütün adımları beklenen çıktıyı verir.

## Bu planın kapsamadıkları

1. **Plan 11 (M6 ikinci dilim):** fark rozeti ve tzdb farkını kabul akışı (Plan 08 Task 8 "D13 sessiz yolu eski kısaltma/uyarıyı tutar" ile; tzdb kabul bandı); ayarlar arayüzü (`CalcSettings`, `AppPrefs`, `AppPrefs.schema_version` denetimi ve tercih `calc_settings`'in sessiz geri dönüşü, Plan 08 Task 3); transit katmanı (halka, `transit_table`, tarih seçimi, açık/kapalı kaydı; `compute_transits` Rust'ta hazır); `birth_info` paneli ve rozetlerin orada gösterilmesi; `dasha_strip`; doğum formunda cinsiyet alanı. Gerekçe: D1.
2. **Plan 08 madde 1 ve madde 3'ün JHD bölümü:** gerçek Jagannatha Hora dosyalarıyla denetim listesi (kod sayfası 1252/1254, UTF-8 olmayan dosyada BOM, "Nan" yer, çıplak `.jhd` adı, geçersiz tarihte satır, saniye yuvarlama), JHD için 64 KiB dosya sınırı ve dosya sayısı, toplu içe aktarma (arayüz seçer, ilerleme, partiler arasında kilit bırakılır; Plan 08 Task 11 "yarıda DB hatası raporu kaybeder" ve Task 13 "içe aktarma kilidi tutar"), isteğe bağlı `clients(name)` indeksi, gerçek dosya kurallarıyla proptest. **Plan 11; gerçek dosyalar olmadan kapatılamaz.** `import_jhd_files_blocking` Rust'ta durur, komut kayıtlı değildir.
3. **Plan 08 madde 10** (veri klasörü ve dosyası için 0700/0600): M8; Windows/Linux taşımasıyla.
4. **Plan 08 madde 11'in ikinci yarısı** (zehirlenmiş kilitten kurtulma): M8.
5. **Plan 08 madde 14** (`chart_tags(tag_id)` indeksi) ve NFD etiket anahtarlarının göçü: sonraki şema göçü (C3, D7).
6. **`save_view_state` sınırları:** Plan 09 D6/C6 ile kapandı (iki ayrıştırıcıda 64/48, arayüz sınırın ötesine yazmaz, Rust okurken kırpar). Rust'ın yazarken sınırlaması (Plan 09 Task 2 küçük bulgusu) Plan 11'e, ayarlar ve `schema_version` denetimiyle birlikte: saklanan biçimin sözleşmesini değiştirir.
7. **M6-1** (form hatalarının ekran okuyucuya duyurulması, katlama grubu `radiogroup`): Plan 11, doğum formunu cinsiyet alanıyla birlikte değiştiren görevde. M6 kapanmadan iner; yeniden ertelenemez (D1).
8. **M6-5** (retro işareti aralığı): nakshatra kısaltmalarını haritaya taşıyan planda (etiket yerleşimi zaten yeniden yapılır; C11 temelleri bir kez yeniden üretilir).
9. **M6-6** (boş tembel dasha seviyesi): Plan 11, `dasha_strip` ile (aynı `useLazyDasha`). M6 kapanmadan iner; yeniden ertelenemez (D1).
10. **M6-7** (küçük erişilebilirlik geçişi: açılış uyarısının ilk boyamada duyurulması, `≈` çift duyurusu, devre dışı düzen menüsü öğelerinin nedeni, `.visually-hidden`'ın `tokens.css`'e taşınması; madde 18 ve 19 da bu geçiştedir): Plan 11; M6 kapanmadan iner, yeniden ertelenemez (D1). Bu plan `.visually-hidden`'ı kullanır, yerini değiştirmez.
11. **M6-9** (test borcu: Plan 09 Task 1 kesme, Task 3 bağımsız değişken fikstürleri, Task 7 lagna hücreleri, Task 12 boşlukları, e2e'de `dasha_children` sayısı): Plan 11.
12. **Plan 09 kapsam dışı 13–15** (atlas açılamama nedeni, tarih değişince `fold`'un sıfırlanması, Esc ile yarım taslağın kaybı, sabit ofset `-0:00`, `errorField` testleri, arama debounce aralığında eski liste ve WKWebView elle denetimleri): Plan 11, doğum formu görevinde.
13. **Plan 08 küçük bulguları:** Task 7 etiket sorgusunun ham biçimi ve `load_client` N+1 (arama sınırı 200 ile sınırlı; ölçüm gerektirir: Plan 11); Task 9 okunamayan JSON'da `null` dışa aktarma ve bilinmeyen alan politikası, Task 10 CSV `"; "` etiket ayırıcısı ve sınırsız dışa aktarma (dışa aktarma biçim belgesi: Plan 11); Task 6 kırpma testi (sonraki dokunuşta).
14. **Seçili danışanları dışa aktarma** (`ExportOptions.client_ids`): istek olursa (D11).
15. **Paletin danışan sonucu dışında not araması:** `search_clients` notta eşleşen danışanı gösterir; notun kendisine gitme yok.
16. **Kenar çubuğu genişliği:** sabittir; yeniden boyutlandırma ve açık/kapalı durumunun kaydı yoktur (her açılışta açık).
17. **Dışa/içe aktarma boyut simetrisi:** dışa aktarmanın boyut sınırı yok, içe aktarma 32 MiB üstünü reddeder ve D2'nin 9,7 MB ölçümü notsuzdu; notlu bir yedek içe aktarma sınırını aşabilir ve kullanıcı bunu geri yüklerken öğrenir. Plan 11: dışa aktarma komutu yazılan dosya `MAX_IMPORT_BYTES`'ı aşınca bildirir (`warning` bildirimi) ya da sınır ölçülmüş bir bellek değeriyle yükseltilir. **M6 kapanmadan iner.**
18. **Kenar çubuğu listesi tek sekme durağı:** arama kutusu ile "Veri" bölümü arasında sekme sırasında 200'e kadar danışan düğmesi var. Ok tuşlarıyla gezilen tek sekme durağı (roving tabindex), M6-7 geçişinde. **M6'yı kapatmayı engeller.**
19. **Odak dönüşü:** onaylı silmeden (not, harita, danışan) sonra açan öğe yoktur ve odak `body`'ye düşer; JSON dışa aktarma penceresi odağı meşgulken `disabled` olan bir düğmeye döndürür. Odak kararlı bir hedefe taşınır; meşgul düğmelerde `aria-disabled` ve "çalışıyor" durum metni (UI spec §5, 1 s üstü işlemler). M6-7 geçişi.
20. **Not sınırı ipucu ve 100 not:** not `MAX_NOTE_UNITS`'e ulaşınca (yapıştırma bugün sessizce kesilir) ve danışanın `NOTES_LIMIT`'ten fazla notu olunca (eskiler gösterilmez) bir metin gösterilir.
21. **Plan 08 Task 12 test boşlukları** (doğum verisinin fikstürle karşılaştırılmaması, DST boşluğunda değişiklik olmaması, `save_view_state` komutunun testsizliği): M6-9 ile.
22. **Spec düzeltmeleri:** v1 spec §4/§8 sekme ifadesi (D4), 64 KiB not sınırı (D10) ve 32 MiB içe aktarma sınırı (D2) spec'lere yazılır.
23. **Küçükler:** içe aktarma dosyasının başında UTF-8 BOM; içe aktarmadan gelen virgüllü etiketin etiket kutusunda bölünmesi; etiket süzgecinin artık olmayan etiketi tutması; `list_tags` hatalarının yok sayılması; "Yeni not"un geride bıraktığı boş notlar; yerel pencerenin `set_parent` ile uygulama penceresine bağlanması; `storePersistence` kayıt pompasının çevresinde `try/finally`.

---

## Self-Review

### Spec kapsamı

| Gereksinim | Kaynak | Görev |
|---|---|---|
| Danışan veritabanı arayüzü: liste, arama, son açılanlar | v1 §11 M6; UI §4 | 3, 7 |
| `Cmd+K` paleti: danışan, harita, varga, düzen, komut | v1 §4; UI §4 | 5, 8 |
| `Cmd+K` → ad → `Enter`, form yok | v1 §8 | 8, 11 |
| Notlar paneli, tarih damgalı, haritaya isteğe bağlı bağ | v1 §5; UI §3.3 | 9 |
| Etiketler (harita düzeyi), süzgeç | v1 §5, §11 | 2, 3, 7 |
| Oturum geri yükleme, ikinci harita, silme | v1 §4, §11 | 1, 6, 7 |
| JSON içe/dışa, CSV dışa; notlar varsayılan kapalı | v1 §5 | 10 |
| JHD içe aktarma | v1 §5 | Plan 11 (madde 2) |
| Fark rozeti, transit katmanı, ayarlar | v1 §11 M6 | Plan 11 (madde 1) |
| `{code, params}` hata metinleri | v1 §7 | 3, 10 |
| Plan 08 madde 3, 9, 11, 13 | Plan 08 | 10, 1, 3, 10 |
| M6-2, M6-3, M6-4, M6-8, M6-10, C5 | Plan 09 | 5, 4, 11, 4+6, 6, 4 |
| Fable Plan 10 incelemesi MF1–MF5, S1–S3, C2′, C15–C17 | Plan 10 incelemesi | 5+11, 8, 9, 10, 7, 7, 6, 7, 10+12, 12, 5, 9 |
| Kişisel veri loglanmaz | v1 §4; C1 | 3, 9, 10 |

### Yer tutucu taraması

Belirsiz adım yok; her kod adımı tam dosya içeriğini ya da `git apply` ile uygulanabilir birleşik farkı içerir. Üretilen dosyalar (`bindings.ts`, `delhi.json`, `Cargo.lock`) elle yazılmaz; her biri üreten komutla verilir.

### Tip tutarlılığı

- `Session.openClient(clientId, chartId?)` Task 6'da tanımlanır; Task 7 (`ClientList`, `OpenClient`), Task 8 (Shell `runPalette`) aynı imzayla çağırır ve `Promise<CommandError | null>` dönüşünü `notice.openFailed`'e bağlar.
- `Session.touch()` Task 6'da; Task 7 (başka harita silindi) ve Task 10 (içe aktarma) kullanır. `revision` Task 7 `ClientList`/`OpenClient` ve Task 8 `Palette` bağımlılığıdır.
- `ConfirmDialog` Task 7'de tanımlanır; Task 9 `NoteEditor` aynı alanlarla kullanır.
- `matchShortcut` `palette`/`sidebar` Task 5'te; Shell Task 7'de `sidebar`'ı, Task 8'de `palette`'i bağlar.
- `api.*` adları Task 3 ve 10'da `bindings.ts`'teki komut adlarıyla (`listClientsByTag`, `exportClientsJson`, …) aynıdır; `api.test.ts` her birinin iletimini denetler.
- `MAX_NOTE_BYTES` Task 9'da `store`'dan dışa aktarılır, Task 10 `exchange.rs` `crate::notes::MAX_NOTE_BYTES` ile kullanır; arayüz sabiti `MAX_NOTE_UNITS` yorumda ona bağlanır.
- `Notice` `type: "info"` Task 10'da eklenir; `NoticeBar` `t(\`notice.${type}\`)` ile `notice.info` metnini kullanır (üç dilde var).

### Görev başına test sayıları (kopyada ölçüldü)

| Görev | Vitest dosya | Vitest test | Rust |
|---|---|---|---|
| Taban (`0424a42`) | 36 | 299 | `jyotish-desk` lib 13, `store` lib 64, `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2 |
| 1 | 36 | 299 | `store` lib 68 (+4) |
| 2 | 36 | 299 | `store` lib 70 (+2) |
| 3 | 36 | 299 | `jyotish-desk` lib 14 (+1) |
| 4 | 36 | 303 | — |
| 5 | 36 | 305 | — |
| 6 | 37 | 310 | — |
| 7 | 38 | 319 | — |
| 8 | 40 | 330 | — |
| 9 | 41 | 341 | `store` lib 71 (+1) |
| 10 | 42 | 346 | `jyotish-desk` lib 19 (+5); `store` 71 (bir test genişledi) |
| 11 | 42 | 346 | + Playwright e2e 6 (+2), görsel 6 (yerel, temeller değişmedi) |
| 12 | 42 | 346 | doğrulama |

### Doğrulama

Plan kopyada (`/Users/ttd/jd-p10`, `0424a42`'den APFS klonu) görev görev uygulandı ve her görevin kapıları çalıştırıldı; bu belgenin kod blokları o commit'lerden üretildi. Ardından belge ikinci bir temiz kopyaya (`0424a42`) betikle uygulandı (yeni dosyalar yazılarak, farklar `git apply` ile, üretilen dosyalar komutlarıyla) ve sonuç ağaç ilk kopyanın son durumuyla karşılaştırıldı. Uygulama sırasında bulunan ve plana işlenen sorunlar: (1) Task 1'in zaman damgası değişikliği `ui_fixture`'ı bayatlatıyordu: fikstür yeniden üretimi Task 1'e eklendi (C10); (2) `NotesPanel.test` ilk sürümünde `rerender` sağlayıcıları düşürüyordu ve takılan kayıt sonraki testleri kilitliyordu: test durumlu bir `Host` ile yazıldı; (3) iki yorum yasaklı sözcük taramasına takıldı ve yeniden yazıldı; (4) kenar çubuğu görsel temelleri daraltıyordu: görsel test kenar çubuğunu kapatır (C11, temeller değişmedi); (5) `api.test.ts` C2 denetimi yalnızca `api` anahtarlarına bakıyordu: artık `bindings.ts` kaynağını da tarar.

Fable'ın Plan 10 incelemesinden (APPROVE_WITH_FIXES) sonra MF1–MF5 ve S1–S3 kodda uygulandı: özgün görev commit'leri ikinci bir kopyada (`/Users/ttd/jd-f10`) sırayla yeniden uygulanıp her biri düzeltmeleriyle değiştirildi ve her görevin kapıları yeniden çalıştırıldı (vitest 305, 310, 319, 330, 341, 346, 346; `jyotish-desk` lib 19; e2e iki kez 6; görsel 6). Kod blokları o commit'lerden yeniden üretildi; belge yine temiz bir kopyaya (`0424a42`) betikle uygulandı ve sonuç ağaç düzeltilmiş kopyanın son durumuyla karşılaştırıldı. Bu geçişte bulunanlar: (6) MF1 `view/store.test.tsx`'in "yazı alanında ⌘digit yok sayılır" beklentisini de tersine çevirdi (Task 5'e eklendi); (7) Task 10'un i18n blokları Task 9'un yeni `notice.noteSaveLost` satırıyla aynı yere düşüyordu: iki satır birlikte tutuldu. MF6 (tek kapı) ve MF7 (madde 17–23) belge düzeyindedir; C2′ ve C15–C17 Global Constraints'e eklendi.
