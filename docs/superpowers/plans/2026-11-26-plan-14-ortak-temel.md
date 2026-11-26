# Plan 14 — Alt proje 3.1: Ortak temel (bakışlar, ev lordları, modül IPC'si; panchang, ashtakavarga ve bakış panelleri)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** İleri modüllerin (Alt proje 3.2–3.9) üzerine kurulacağı ortak temeli vermek: `CalcSettings` şema sürümü 2 (`advanced` alt yapısı ve sürüm 1 → 2 göçü), ephemeris görmeyen ortak girdi (`ChartInput`, `Positions`), ev lordluğu, doğal iyicil/kötücül sınıfı, Kuja dosha olgusu, graha bakışları (burç temelli tam bakış ve sphuta drishti), ilk modül komutu `compute_drishti`, el örneği düzeneği ve üç yeni pano paneli (Panchang, Ashtakavarga, Bakışlar).

**Architecture:**
- **Saf çekirdek, ince toplayıcı (spec K3).** `jyotish-core`'a dört modül eklenir: `positions` (ortak girdi ve `compute_positions` toplayıcısı), `lordship`, `nature` (doğal sınıf ve Kuja dosha) ve `drishti`. Hesap fonksiyonları yalnızca `Positions` görür; el örnekleri ephemeris olmadan `Positions::synthetic` ile kurulur.
- **Modül komutu kalıbı (spec K1, K2).** `app/src-tauri/src/module_commands.rs`: tek `ChartInput`, hesap motor iş parçacığında, gövde `blocking` havuzunda, hata `CommandError`. `Chart` ve `open_client` değişmez; sonuç saklanmaz, panel açıkken istenir. Sonraki alt projelerin komutları aynı dosyaya aynı kalıpla eklenir.
- **Ayar sürümü 2 JSON göçüdür, SQLite göçü değildir.** `CalcSettings` haritanın `calc_settings_json` sütununda ve `AppPrefs` içinde JSON olarak durur; sürüm göçü `store::settings::CALC_SETTINGS_MIGRATIONS` zinciridir (Plan 03'ten beri hazır, bugüne dek boş). SQLite şeması (`schema.rs`, göç V1) değişmez (D3).
- **Arayüz hesap yapmaz.** Panchang ve Ashtakavarga panelleri `Chart`'ın v1'den beri taşıdığı veriyi çizer; Bakışlar paneli `compute_drishti` sonucunu çizer. Ortak `modules/useModule` kancası bir modül sonucunu panel açıkken ister, girdi değişince yeniler ve eski yanıtı atar. Modül panelleri etkin düzene komut paletinden eklenir ve kaldırılır (D9).
- **Testler:** el örnekleri (`tests/golden/modules/`, kaynak yalnızca `hand`), proptest değişmezleri, komut gövdesinin gerçek motorla entegrasyon testi (v1 haritasıyla çapraz denetim, bütçe ölçümü), Vitest (paneller, kanca, düzen, palet), Playwright e2e (palet → paneller → ortak vurgu).

**Tech Stack:** Rust 1.93+ (stable), Tauri 2.12, tauri-specta 2.0.0-rc.29, serde, proptest, React 19, TypeScript 7, Vitest 5, Playwright 1.65.1. **Yeni Rust ya da npm bağımlılığı yok.**

**Spec:** `docs/superpowers/specs/2026-11-26-alt-proje-3-ileri-moduller-design.md` (commit `cd95efe`): §2 (K1–K11), §4 (Alt proje 3.1), §13 (ortak test stratejisi), §14 (gizlilik), §15 (bütçeler), Ek A (adlandırma). §5–§12, 3.1'in ortak API'lerinin kimlere hizmet edeceğini belirler (bkz. "Sonraki alt projelere verilen API"). UI spec: `2026-11-24-ui-design.md` §3.3, §3.4, §5, §6, §8.

**Ön koşul:** Taban commit `cd95efe` (Plan 13 ve ileri modüller spec'i `main`'de). `main` bu commit'ten ilerlediyse Task 1 adım 1 tabanları yeniden ölçer ve sayaç zinciri o ölçümden yürütülür (farklar aynıdır). Plan 13'ün elle denetim kapısının yalnızca insana ait maddeleri sahibine devredilmiştir; bu planı engellemez.

## Kararlar

Sahibi kararları devretti; muhafazakâr seçenek seçildi ve her karar bir satır gerekçeyle yazıldı. "Spec boşluğu" diye işaretlenenler spec'te düzeltilmesi gereken yerlerdir ve belgenin sonundaki "Spec'e geri bildirim" bölümünde toplanmıştır. D8, D13 ve sayaç zinciri bu plan yazılırken deponun bir kopyasında ölçülmüştür (macOS 27.2, Rust 1.99).

- **D1 — `advanced` alt yapısı iki alanıyla bu planda tanımlanır: `karaka_scheme: KarakaScheme {Eight, Seven}` (varsayılan `Eight`) ve `jaimini_co_lords: bool` (varsayılan `true`). Bu planda hiçbir hesap bu alanları okumaz ve ayar penceresine denetim eklenmez; ikisi de 3.2'nin işidir.** Gerekçe: spec K7 sürüm 2'nin içeriğini bu iki alanla tanımlar ve §4 "sürüm 1 → 2 göçü varsayılanları yazar" der; alanları göçle aynı anda tanımlamak sürüm 2'yi tek adımda tamamlar ve 3.2'nin kalıcı veri biçimine yeniden dokunmasını önler. Fark rozeti alanı şimdiden kapsar (`SETTINGS_FIELDS`'e `advanced` eklenir). **Spec boşluğu 1.**
- **D2 — Tolerans `store`'un okuma yolundadır (`fill_advanced`); IPC tipi katıdır.** `CalcSettings.advanced` ve iki alanı serde'de zorunludur; `#[serde(default)]` kullanılmaz. `store::settings::load_calc_value` eksik `advanced` bloğunu ve eksik anahtarları varsayılanla doldurur; bilinmeyen anahtarı serde zaten yok sayar. Tanınmayan bir seçenek değeri (ör. `"karaka_scheme": "Nine"`) tolerans kapsamında değildir: ayar mevcut `CalcSettingsReset` yoluyla varsayılana döner, uyarı verilir ve saklanan JSON'a dokunulmaz. Gerekçe: `specta_builder` serde fazlarını kapatır ve "IPC tiplerinde faz farkı yaratan serde özniteliği yok" kuralını bir testle korur (`bindings.rs`); `default` özniteliği TS tipini isteğe bağlı yapar ya da tipi ikiye böler. Kalıcı verinin okunduğu tek huni zaten `load_calc_value`'dur (harita, tercihler, JSON içe aktarma). **Spec boşluğu 2.**
- **D3 — SQLite göçü 2 yoktur.** `CALC_SETTINGS_SCHEMA_VERSION` 2 olur ve `CALC_SETTINGS_MIGRATIONS = [fill_advanced]` olur. Sürüm 1 kaydı okunduğunda `SettingsLoad::Migrated` döner ve mevcut `settle_settings` sürüm 2 JSON'unu haritaya geri yazar. Sürüm 2 kaydında eksik anahtar tamamlanırsa sonuç `Current` sayılır ve geri yazılmaz (her okumada aynı sonuç; daha yeni bir sürümün bilinmeyen anahtarı kaybolmaz). `AppPrefs` şema sürümü 1 olarak durur; içindeki `calc_settings` aynı huniden geçer. Gerekçe: spec §4 "Yeni kalıcı veri yok"; C3 (göç V1 donuktur) korunur ve `schema.rs` değişmez.
- **D4 — Beş golden fixture'ın `settings` bloğu, üretici betik ve README sürüm 2'ye elle çevrilir; beklenen değerler değişmez.** Gerekçe: `compute_chart` yalnızca güncel sürümü kabul eder ve fixture şeması bilinmeyen alanı reddeder; fixture'ı `swetest` ile yeniden üretmek gereksizdir (astronomik beklenti aynıdır).
- **D5 — `Positions` için spec §4'teki tip esastır (K3'teki kısa tanım değil): `GrahaPoint { graha, sidereal_lon, rasi_sign, deg_in_sign, lat, speed }`, `Positions { lagna_lon, lagna_sign, grahas: [GrahaPoint; 9] }`. `lagna_lon` lagnanın **sidereal** boylamıdır, `lagna_sign` rasi çerçevesindeki burcudur; ikisi birlikte dolu ya da birlikte boştur. `Positions::synthetic(lagna_lon, lons)` sentetik konum kurar (rasi çerçevesi sidereal sayılır).** Gerekçe: sphuta drishti ve bhava ortası (3.4) sidereal boylamla çalışır; burç kuralları rasi çerçevesini izler (K5). Tropikal-rasi kipinde `Sign::of(lagna_lon) ≠ lagna_sign` olabilir; bir test bunu sabitler. **Spec boşluğu 3.**
- **D6 — Ay'ın büyüme aralığı `[0°, 180°)`'dir (`moon_is_waxing`): uzanım tam 180° ise Ay küçülen yarıdadır.** Gerekçe: spec K6 "0°–180°" der ve uç noktayı belirtmez; `[0°, 180°)` v1'deki `paksha` sınırıyla aynıdır (tithi 1–15 Shukla). Merkür'ün sınıfı hesaplanırken Ay da bu kuralla iyicil ya da kötücül sayılır. **Spec boşluğu 4.**
- **D7 — Bakış tablosu: `sign_aspects` her zaman 13 kayıttır (Güneş, Ay, Merkür, Venüs birer; Mars, Jüpiter, Satürn üçer), bakan graha sırasıyla ve ev sırasıyla; `sphuta[i][j]` bakan `ASPECTING_GRAHAS[i]` → bakılan `Graha::ALL[j]`; grahanın kendisine bakışı 0'dır; Rahu ve Ketu bakan olarak 0 verir. `method = "graha_drishti.bphs_v1"`.** Gerekçe: spec §4'ün veri modeli; sabit sıra arayüzün ve AI paketleyicisinin tabloyu dizinle okumasını sağlar.
- **D8 — `compute_drishti` temel haritayı `compute_chart` ile hesaplar ve `Positions`'a indirger; ayrı bir "hafif konum" yolu yazılmaz.** Gerekçe: bütçe ≤ 10 ms'dir (spec §15); ölçüm: hata ayıklama derlemesinde ortanca 0,6 ms. Hafif yol (`search_facts`) 3.9'un işidir. `ChartInput.time_known == false` iken toplayıcı lagnayı boşaltır; bakış tablosu lagnaya bağlı olmadığı için sonuç aynıdır (test eder).
- **D9 — Modül panelleri etkin düzene komut paletinden eklenir ve kaldırılır: "Panchang panelini aç/kapat", "Ashtakavarga panelini aç/kapat", "Bakışlar panelini aç/kapat" (yalnızca harita açıkken; baskı görünümünde yok sayılır). Yeni panel düzenin altına, sola, `MODULE_PANELS`'teki boyutuyla eklenir; hiçbir paneli yerinden oynatmaz; eklenince odak ona taşınır. Hazır düzenler değişmez, yeni kısayol yoktur.** Gerekçe: v1'de notlar dışında panel ekleme ya da kaldırma yolu yoktur ve spec dokuz alt projede on beşe yakın panel tanımlar ama düzene nasıl gireceklerini söylemez; bu olmadan üç panel ulaşılamaz kod olurdu. Notlar panelinin "aç/kapat" kalıbı genelleştirildi; hazır düzen eklemek `⌘1…9` sırasını kaydırırdı. Sonraki alt projeler panellerini `MODULE_PANELS`'e ekler. **Spec boşluğu 5.**
- **D10 — Panchang ve Ashtakavarga panelleri komut çağırmaz; `Chart.panchang` ve `Chart.ashtakavarga`'yı çizer. Doğum saati bilinmiyorsa Panchang değerleri "≈" ile işaretlenir (öğlen için hesaplanmıştır); Ashtakavarga gösterilmez (bindular lagnaya da bağlıdır).** Gerekçe: veri v1'den beri `Chart` içindedir ve K1 `Chart`'ın değişmemesini ister; bu yüzden K2'nin "lagnaya bağlı çıktı Rust'ta `None` döner" kuralı bu iki panelde uygulanamaz ve gizleme arayüzde yapılır (v1'deki lagna satırı ve ev sütunu gibi). **Spec boşluğu 6.**
- **D11 — Ashtakavarga tablosu sekiz satırdır: yedi BAV ve SAV. Satır toplamı gösterilen on iki hücrenin toplamıdır; sütun toplamı SAV satırıdır (Rust'tan). "Harita içi" biçim, burçların sabit kutularda durduğu 4 × 4 ızgaradır (güney düzeni) ve seçili görünümün (SAV ya da tek graha BAV) sayılarını gösterir.** Gerekçe: spec "12 burç × 8 satır; satır ve sütun toplamları" der; `Ashtakavarga` yalnızca BAV ve SAV taşır, katkı dökümünü (sekiz referansın ayrı ayrı binduları) taşımaz. Satır toplamı astrolojik kural değil tablo toplamıdır (C4'ün kapsamı dışında); test bunu Rust'ın `BAV_TOTALS` sabitleriyle (48, 49, 39, 54, 56, 52, 39) ve 337 ile karşılaştırır. **Spec boşluğu 7.**
- **D12 — Bakışlar panelinin tek ayarı `value: "full" | "virupa"`'dır (varsayılan `full`). `full`: burç temelli tam bakışlar `●` ile (ekran okuyucuda "tam bakış"); `virupa`: sphuta drishti değeri bir ondalıkla. Tür seçici (graha / rashi) bu planda çizilmez; 3.2 `rashi` türüyle birlikte ekler.** Gerekçe: tek seçenekli denetim gürültüdür (K7'nin ayar ölçütüyle aynı mantık). `[[f64; 9]; 7]` TS'te `number | null` demetlerine dönüşür (specta `f64`'ü böyle yazar); panel `null`'u çizgiyle gösterir.
- **D13 — `useModule` önbellek tutmaz; anahtar `inputKey(opened)` = harita kimliği + `ChartInput`'tur. Aynı anahtar ve deneme için süren istek yeniden gönderilmez.** Gerekçe: komut 1 ms'nin altındadır; önbellek bayatlama riski getirir. Ölçüm: geliştirme kipinde `StrictMode` effect'i iki kez koşar; istek paylaşılmazsa komut iki kez çağrılır (e2e bunu gösterdi).
- **D14 — Yeni görsel temel (PNG) ve yeni baskı düzeni yoktur (K10); üç panelde retro gösteren öğe yoktur.** Gerekçe: paneller metin tablolarıdır; içerikleri Vitest ve e2e ile doğrulanır. Mevcut on görsel temelin değişmemesi beklenir (Task 10 doğrular).
- **D15 — El örnekleri `tests/golden/modules/<modül>/*.json` altındadır; şema spec §13'tekidir ve `derivation` adım dizisidir. Bu plan iki modül açar: `foundation` (doğal sınıf, Kuja dosha, ev lordları; en az 6) ve `drishti` (13 tam bakışın tamamı ve en az on sphuta değeri; en az 4).** Gerekçe: K9 (başka yazılım çıktısı fixture olmaz) ve §13; düzenek sonraki alt projelerin kendi klasörünü eklemesine hazırdır.
- **D16 — Yeni `CommandError.code` yoktur.** `compute_drishti` yalnızca `compute_chart`'ın hatalarını verir (`invalid_location`, `invalid_julian_day`, `unsupported_schema_version`, `invalid_settings`, motor hataları); hepsi çevrilidir. Gövde `client_commands::blocking` ile koşar: panik `task_failed` olur ve iletisi taşınmaz (§14).

## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni bağımlılık yok: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `app/package.json` ve `app/pnpm-lock.yaml` değişmez; `cargo deny check` Task 10'da `advisories ok, bans ok, licenses ok, sources ok` verir.
- **Yasaklı ad kuralı (v1 spec §10 "İsim kuralları", ilk madde; ileri modüller spec'i Ek A):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Kural harf dizisi düzeyindedir: Türkçe sözcüklerin içinde de geçemez (yerine ör. "diğer", "durur", "geri", "artan", "sürer" yazılır). Klasik terimler de kapsamdadır: Shadbala'nın üçüncü gücü "zaman gücü" (`temporal`) diye anılır (Ek A; bu planda geçmez). Her commit adımı eklenen satırları, dosya adlarını ve commit mesajını `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile tarar; çıktı boş olmalıdır (C13). Bu belgede de eşleşme yoktur.
- **Clean-room ve yorum metni yok:** yöntemler klasik kuraldan özgün olarak uygulanır; başka ürünün adı, düzeni, metni ya da çıktısı kullanılmaz; kaynak gösterimi geneldir ("BPHS, bakışlar bölümü") ve alıntı yapılmaz. Paneller hesaplanmış olguları ve klasik etiketleri gösterir; bir bakışın, tithinin ya da bindu sayısının ne anlama geldiğini anlatan metin eklenmez.
- **Evler rasi evidir (K4), rasi çerçevesi `settings.zodiac`'ı izler (K5), doğal sınıf tek fonksiyondur (K6).** Yeni kodda "ev" lagna burcundan sayılan bütün burç evidir; `CalcSettings.house_system` yalnızca v1'deki bhava tablosunu etkiler. Hiçbir modül kendi iyicil/kötücül listesini tutmaz.
- **Doğum saati bilinmiyorsa lagnaya bağlı her çıktı Rust'ta `None` döner (K2):** `Positions.lagna_*`, `house_of`, `KujaDosha.from_lagna`. İstisna D10'dadır.
- **El örnekleri (§13, K9):** `source` yalnızca `hand`'dir; beklenen değer kuraldan elle türetilir ve `derivation`'da adım adım yazılır. Bir el örneği motorla uyuşmazsa önce kural yeniden okunur; **beklenen değer motor çıktısına bakarak düzeltilmez.** Gözden geçirme döngüsünün zorunlu maddesi: gözden geçiren, Task 2 ve Task 3'teki fixture'ların `derivation` adımlarını kuraldan yeniden hesaplar.
- Kapılar: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `pnpm --dir app test`, `pnpm --dir app build` her görev sonunda yeşil; `pnpm --dir app e2e` Task 9 ve 10'da. `tsc` `strict`.
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez). Node 24+, pnpm 11.8.0.
- **Çalışan `pnpm tauri dev` durdurulmaz** (port 1420); e2e 1421'i kullanır. `pkill -f` kullanılmaz. Üretilen `.app` çalıştırılmaz.
- Yalnızca macOS hedeflenir. Yorum metni ve AI paneli bu planın dışındadır.
- **Tabanlar (`cd95efe`):** `jyotish-core` lib 110, golden 8; `store` lib 81 (+1 `#[ignore]`); `jyotish-desk` lib 46, `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2; Vitest `Test Files  60 passed (60)`, `Tests  525 passed (525)`; e2e 12; görsel 10 (yerel, darwin); yayın betiği testleri 14.
- **Sayaç zinciri** (her satır görev bitince; boş hücre "önceki satırla aynı"). Bu plan yazılırken `cd95efe`'nin bir kopyasında her görev ayrı commit olarak uygulanıp ölçülmüştür.

  | Görev | `jyotish-core` lib | `modules_golden` | `store` lib | `jyotish-desk` `modules` | Vitest dosya / test | e2e |
  |---|---|---|---|---|---|---|
  | Taban | 110 | — | 81 (+1 ign.) | — | 60 / 525 | 12 |
  | Task 1 | 111 | | 84 (+1 ign.) | | 60 / 527 | |
  | Task 2 | 125 | 2 | | | | |
  | Task 3 | 137 | 3 | | | | |
  | Task 4 | 138 | | | 1 | 60 / 527 | |
  | Task 5 | | | | | 62 / 541 | |
  | Task 6 | | | | | 63 / 549 | |
  | Task 7 | | | | | 64 / 556 | |
  | Task 8 | | | | | 65 / 565 | |
  | Task 9 | | | | | | 13 |
  | Son inceleme düzeltmeleri | 139 | | | | 65 / 571 | |

  Değişmeyenler: `jyotish-core` golden 8; `jyotish-desk` lib 46, `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2; görsel 10; yayın betiği testleri 14.
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir:**
  - **C1** Gizlilik (spec §14): danışan adları, doğum verisi, not gövdeleri, yer bilgisi, dosya yolları ve `CommandError.params` loglanmaz; `no-console.test.ts` durur ve izin listesi büyümez. `ChartInput` ve `inputKey` doğum anı ve konumu taşır: loglanmaz, hata parametresine girmez. Hiçbir modül ağ isteği yapmaz; modül sonucu veritabanına yazılmaz.
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu ya da adres almaz. `compute_drishti`'nin tek bağımsız değişkeni `ChartInput`'tur (dizgi alanı yoktur); `api.test.ts`'in dizgi bağımsız değişken listesi değişmez.
  - **C3** Göç V1 donmuştur; `crates/store/src/schema.rs` değişmez (D3). `AppPrefs` ve `ViewState` şema sürümleri değişmez.
  - **C4** Arayüz astrolojik hesap yapmaz: bakış, lord, sınıf, panchang ve bindu Rust'tan gelir. JD → takvim biçimlemesi ve tablo toplamı (D11) gösterimdir.
  - **C5/C17** Otomatik kayıtlar ve boşaltılmaları değişmez. **C6** Düzen ayrıştırıcıları (`parseLayoutState`, Rust `check_panel_layout`) değişmez; yalnızca panel kataloğu büyür.
  - **C7** Kısayollar yalnızca `shortcuts.ts`'tedir; bu plan kısayol eklemez. **C8** Her pencere `shell/Dialog`'u kullanır (bu plan pencere eklemez).
  - **C9** Renk yalnızca token'lardan gelir (hex yok) ve tek başına anlam taşımaz: tam bakış `●` ve metinle, lagna burcu "Lg" metniyle, yaklaşık değer "≈" ve metinle, vurgu `--select` zemin ve yazı ağırlığıyla gösterilir.
  - **C10** Üretilen dosyalar elle düzenlenmez: `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` Task 1 ve Task 4'te komutla yeniden üretilir.
  - **C11** Görsel PNG temelleri yereldir; bu planda değişmeleri beklenmez (D14).
  - **C12** Her `CommandError.code` üç dilde çevrilidir; bu plan kod eklemez (D16). Her yeni i18n anahtarı üç dilde vardır (`i18n.test.ts` anahtar kümelerini karşılaştırır).
  - **C13** Yasaklı ad taraması her commit'te; depo dışındaki ilerleme notları ve `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** Plan 07'nin gün doğumu toleransı değişmez. **C15** `alert`, `confirm`, `prompt`, `window.print()` ve `window.open()` çağrılmaz. **C16** ⌘ kısayolları değişmez.
- **Retro** her yerde düz `R`'dir ve Rahu ile Ketu'da gösterilmez (UI spec §2.3). Bu planın panelleri retro göstermez (D14); kural v1 panellerinde olduğu gibi sürer.
- **Erişilebilirlik:** WCAG AA. Tablolar `caption`, `scope="col"` ve `scope="row"` taşır; vurgu düğmeleri klavyeyle çalışır (`aria-pressed`); ayar seçicilerin görünür etiketi vardır; yükleme iskeleti `aria-busy`, hata `role="alert"` ve "Yeniden dene" düğmesi taşır; paletten eklenen panele odak taşınır, kaldırılan panelden odak ana bölgeye geçer (odak `body`'ye düşmez).
- Commit mesajları İngilizcedir ve deponun `Co-Authored-By:` satırıyla biter (yürütücünün oturum yönergesindeki satır). Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, `git tag`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Kod blokları `cd95efe` ağacına göre yazılmış ve bu plan yazılırken deponun bir kopyasında koşulmuştur (Self-Review madde 5). Yeni dosyalar tam içerikle verilir ("**Yeni dosya:**"). Değişen dosyalar `diff` bloğuyla verilir: blok bir dosyaya konup depo kökünde `git apply <dosya>` ile ya da elle uygulanır; bağlam tutmazsa önceki görevin adımı denetlenir, elle uydurma yapılmaz. Bir dosyanın test ve uygulama değişiklikleri ayrı adımlarda ayrı bloklardır; satır numaraları ikinci blokta birkaç satır kayabilir (`git apply` bunu kendisi bulur). `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` diff olarak verilmez; adımlardaki komutlar üretir.

## Sonraki alt projelere verilen API

Bu planın ürettiği ve 3.2–3.9'un tüketeceği adlar. Hepsi `jyotish_core`'dan dışa aktarılır; imzalar Task 2–4'te tam olarak verilir.

| API | Tüketen | Not |
|---|---|---|
| `ChartInput { jd_ut, lat, lon, settings, time_known }` | 3.2–3.9'un her komutu | Tek komut girdisi (K2) |
| `Positions`, `GrahaPoint`, `Positions::{synthetic, without_lagna, point, sign_of, occupants, house_from, house_of}`, `compute_positions` | 3.2–3.6, 3.8, 3.9 | Saf fonksiyonların girdisi; el örnekleri `synthetic` ile |
| `house_sign`, `house_lord`, `houses_owned` | 3.2 (arudha), 3.4 (bhavadhipati), 3.5 (raja, dhana, viparita), 3.8, 3.9 | Rasi evleri; Jaimini eş lordları 3.2'de |
| `nature`, `Nature`, `moon_is_waxing` | 3.2 (argala 3. ev), 3.4 (paksha, drik bala), 3.5 (Amala), 3.8 | K6: tek sınıf fonksiyonu |
| `kuja_dosha`, `KujaDosha`, `KUJA_DOSHA_HOUSES` | 3.5 (dosha), 3.6 (iki harita) | İptal kuralı yok |
| `graha_aspect_houses`, `graha_aspect_signs`, `sphuta_drishti`, `drishti_table`, `SignAspect`, `DrishtiTable`, `ASPECTING_GRAHAS`, `FULL_DRISHTI` | 3.2 (`SignAspect` rashi drishti için de), 3.4 (drik ve bhava drishti bala: bakılan nokta herhangi bir boylam), 3.5 ("karşılıklı bakış"), 3.8, 3.9 (`GrahaAspects`) | |
| `AdvancedSettings`, `KarakaScheme` | 3.2, 3.3 | Tanım burada, tüketim orada (D1) |
| `module_commands.rs` kalıbı; `tests/modules.rs` (gerçek motor, bütçe ölçümü); `tests/golden/modules/` ve `modules_golden.rs` düzeneği | 3.2–3.9 | Yeni modül: bir klasör, bir `*Expected` yapısı, bir test |
| Arayüz: `modules/input.ts` (`chartInput`, `inputKey`), `modules/useModule.ts`, `MODULE_PANELS`, `.module-table` / `.module-panel` / `.module-toolbar`, `panels/Approx.tsx`, `formatJdMinute` | 3.2–3.9'un panelleri | Panel eklemek: `PANEL_KINDS`, `MODULE_PANELS`, `registry.ts`, `panel.<kind>` |

## Dosya Yapısı (bu plan)

```
crates/jyotish-core/
  src/settings.rs                 AdvancedSettings, KarakaScheme; şema sürümü 2
  src/positions.rs                ChartInput, GrahaPoint, Positions, compute_positions (yeni)
  src/lordship.rs                 house_sign, house_lord, houses_owned (yeni)
  src/nature.rs                   Nature, nature, moon_is_waxing, KujaDosha, kuja_dosha (yeni)
  src/drishti.rs                  bakışlar, DrishtiTable, compute_drishti (yeni)
  src/proptests.rs                foundation_properties, drishti_properties
  src/{lib,chart,transit}.rs      dışa aktarımlar; sürüm 2'ye göre iki test
  tests/modules_golden.rs         el örneği düzeneği (yeni)
crates/store/src/
  settings.rs                     fill_advanced; CALC_SETTINGS_MIGRATIONS = [fill_advanced]
  open.rs, csv.rs                 göç ve sürüm testleri
tests/golden/
  *.json, README.md               settings bloğu sürüm 2
  modules/README.md               el örneği şeması (yeni)
  modules/foundation/*.json       6 el örneği (yeni)
  modules/drishti/*.json          4 el örneği (yeni)
scripts/gen-golden-swetest.sh     settings bloğu sürüm 2
app/src-tauri/
  src/module_commands.rs          compute_drishti (yeni)
  src/lib.rs                      modül ve komut kaydı
  tests/modules.rs                gerçek motorla entegrasyon ve bütçe (yeni)
  tests/{compute_chart,ui_fixture}.rs   sürüm 2; fikstüre bakış tablosu
app/ui/
  bindings.ts                     üretilir
  test/fixtures/delhi.json        üretilir (advanced; drishti)
  test/fixture.ts                 UiFixture.drishti
  api.ts, api.test.ts             computeDrishti
  settings/model.ts (+test)       SETTINGS_FIELDS += advanced
  layout/{model,reducer}.ts (+test)   PANEL_KINDS, MODULE_PANELS, togglePanel
  view/store.tsx                  togglePanel
  palette/{items.ts,Palette.tsx} (+test)   "panel" öğesi
  shell/{Shell.tsx,focus.ts} (+test)       paletten panel açma/kapatma, odak
  modules/{input.ts,useModule.ts} (+test)  ortak girdi ve kanca (yeni)
  panels/Approx.tsx               "≈" işareti (GrahaTablePanel'den taşındı)
  panels/{format,labels}.ts       formatJdMinute; tithi, nitya yoga, vara adları
  panels/PanchangPanel.tsx (+test)        (yeni)
  panels/AshtakavargaPanel.tsx (+test)    (yeni)
  panels/DrishtiPanel.tsx (+test)         (yeni)
  panels/{registry.ts,panels.css,panels.test.tsx}
  i18n/{tr,en,fr}.ts              panel adları, palet, panchang, ashtakavarga, drishti
  App.test.tsx, shell/ChartTitle.test.tsx
app/e2e/{ipc.ts,desk.spec.ts}     compute_drishti taklidi; modül panelleri akışı
```

### Kapsam

| Spec maddesi | Yer |
|---|---|
| §4 Burç temelli graha bakışı; sphuta drishti (tablo, özel bakış eklemesi, 60'ta kesme) | Task 3 |
| §4 Ev lordluğu (`house_lord`, `houses_owned`) | Task 2 |
| §4 Kuja dosha olgusu | Task 2 |
| §4, K2, K3 `ChartInput`, `Positions`, `From<&Chart>` | Task 2 (D5) |
| K6 Doğal iyicil/kötücül sınıfı | Task 2 (D6) |
| K7 `CalcSettings` sürüm 2, `advanced`, sürüm 1 → 2 göçü, fark rozeti | Task 1 (D1–D4) |
| §4 `compute_drishti`; modül komutlarının ortak kalıbı (K1, K8) | Task 4 (D8, D16) |
| §4 Paneller: `panchang`, `ashtakavarga`, `drishti`; ortak seçim | Task 5–8 (D9–D12) |
| §13 El örnekleri, değişmezler, çapraz denetim, komut katmanı testi | Task 2, 3, 4 (D15) |
| §13 Panel durumları: boş, yükleniyor, hata, dolu, "saat bilinmiyor" | Task 6, 7, 8 |
| §14 Gizlilik | C1; Task 10 taramaları |
| §15 `compute_drishti` ≤ 10 ms | Task 4 (ölçüm testi) |
| Son doğrulama ve elle denetim | Task 10 |

---

### Task 1: `CalcSettings` şema sürümü 2: `advanced` alt yapısı ve sürüm 1 → 2 göçü

Spec K7 ve §4 "Göç". Davranış değişiklikleri: `CalcSettings` JSON'u `advanced` bloğunu taşır ve `schema_version` 2 olur; sürüm 1 kaydı (bugünkü her harita ve tercih) okunurken varsayılanlarla sürüm 2'ye çevrilir ve harita açılınca geri yazılır; sürüm 2 kaydındaki bilinmeyen `advanced` anahtarı açılışı engellemez; fark rozeti `advanced`'ı da adlandırır. Hiçbir hesap sonucu değişmez.

**Files:**
- Modify: `crates/jyotish-core/src/{settings,lib,chart,transit}.rs`, `crates/store/src/{settings,open,csv}.rs`, `app/src-tauri/tests/compute_chart.rs`, `tests/golden/*.json` (5 dosya), `tests/golden/README.md`, `scripts/gen-golden-swetest.sh`, `app/ui/settings/model.ts`, `app/ui/i18n/{tr,en,fr}.ts`
- Üretilir: `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`
- Test: `crates/jyotish-core/src/settings.rs` ve `crates/store/src/{settings,open}.rs` içindeki `mod tests`; `app/ui/settings/model.test.ts`, `app/ui/shell/ChartTitle.test.tsx`

**Interfaces:**
- Consumes: `store::settings::{ValueMigration, migrate_value, load_calc_value, SettingsLoad}`, `store::open::settle_settings` (değişmez).
- Produces: `jyotish_core::{AdvancedSettings { karaka_scheme: KarakaScheme, jaimini_co_lords: bool }, KarakaScheme { Eight, Seven }}`; `CalcSettings.advanced: AdvancedSettings`; `CALC_SETTINGS_SCHEMA_VERSION == 2`; `store::settings::CALC_SETTINGS_MIGRATIONS == [fill_advanced]`; TS `CalcSettings["advanced"]`, `SETTINGS_FIELDS` son öğesi `"advanced"`, i18n `settings.field.advanced`.

- [ ] **Step 1: Tabanları ölç**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
git log -1 --format=%h
git status --short
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `git status --short` boş; `jyotish-core` lib `110 passed`, golden `8 passed`; `store` lib `81 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `46 passed`, `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2; Vitest `60 passed (60)`, `525 passed (525)`. Commit `cd95efe` değilse ölçülen sayılar ilerleme notlarına taban olarak yazılır ve zincir oradan yürütülür. (Vitest'te `NotesPanel.test.tsx › New note adds an empty note…` yük altında seyrek olarak düşebilir; bu plandan bağımsızdır, yeniden koşulur ve ilerleme notlarına yazılır.)

- [ ] **Step 2: Başarısız testleri yaz**

Çekirdek: varsayılan JSON sürüm 2 ve `advanced` bloğuyla; bilinmeyen `advanced` anahtarı yok sayılır, eksik blok ve eksik alan IPC'de reddedilir (D2); sürüm 1 ve sürüm 3 `UnsupportedSchemaVersion` verir. `chart.rs`, `transit.rs` ve `compute_chart.rs`'teki "desteklenmeyen sürüm" örnekleri 2'den 1'e çevrilir (2 artık geçerli sürümdür).

```diff
diff --git a/crates/jyotish-core/src/settings.rs b/crates/jyotish-core/src/settings.rs
index 74a1fc4..968e07a 100644
--- a/crates/jyotish-core/src/settings.rs
+++ b/crates/jyotish-core/src/settings.rs
@@ -228,16 +260,24 @@ mod tests {
     use super::*;
 
     const DEFAULT_JSON: &str = concat!(
-        r#"{"schema_version":1,"zodiac":"Sidereal","ayanamsa":"Lahiri","node":"Mean","#,
+        r#"{"schema_version":2,"zodiac":"Sidereal","ayanamsa":"Lahiri","node":"Mean","#,
         r#""house_system":"WholeSign","dasha_year":365.25,"combustion_orbs":{"moon":12.0,"#,
         r#""mars":17.0,"mercury":14.0,"mercury_retro":12.0,"jupiter":11.0,"venus":10.0,"#,
-        r#""venus_retro":8.0,"saturn":15.0},"sunrise_def":"DiskCenterNoRefraction"}"#
+        r#""venus_retro":8.0,"saturn":15.0},"sunrise_def":"DiskCenterNoRefraction","#,
+        r#""advanced":{"karaka_scheme":"Eight","jaimini_co_lords":true}}"#
     );
 
     #[test]
     fn default_matches_spec_defaults() {
         let s = CalcSettings::default();
-        assert_eq!(s.schema_version, 1);
+        assert_eq!(s.schema_version, 2);
+        assert_eq!(
+            s.advanced,
+            AdvancedSettings {
+                karaka_scheme: KarakaScheme::Eight,
+                jaimini_co_lords: true
+            }
+        );
         assert_eq!(s.zodiac, ZodiacMode::Sidereal);
         assert_eq!(s.ayanamsa, AyanamsaSystem::Lahiri);
         assert_eq!(s.node, NodeKind::Mean);
@@ -268,6 +308,29 @@ mod tests {
         assert_eq!(back, CalcSettings::default());
     }
 
+    #[test]
+    fn advanced_block_ignores_unknown_keys_and_is_required_on_the_wire() {
+        // Bilinmeyen anahtar (daha yeni bir sürümün alanı) yok sayılır.
+        let extra = DEFAULT_JSON.replace(
+            r#""jaimini_co_lords":true"#,
+            r#""jaimini_co_lords":false,"future_option":3"#,
+        );
+        let s: CalcSettings = serde_json::from_str(&extra).unwrap();
+        assert!(!s.advanced.jaimini_co_lords);
+        assert_eq!(s.advanced.karaka_scheme, KarakaScheme::Eight);
+        // IPC tipi katıdır: eksik blok ya da eksik alan reddedilir (toleransı `store` sağlar).
+        let missing = DEFAULT_JSON.replace(
+            r#","advanced":{"karaka_scheme":"Eight","jaimini_co_lords":true}"#,
+            "",
+        );
+        assert!(serde_json::from_str::<CalcSettings>(&missing).is_err());
+        let partial = DEFAULT_JSON.replace(r#","jaimini_co_lords":true"#, "");
+        assert!(serde_json::from_str::<CalcSettings>(&partial).is_err());
+        // Bilinmeyen seçenek değeri reddedilir.
+        let bad = DEFAULT_JSON.replace(r#""Eight""#, r#""Nine""#);
+        assert!(serde_json::from_str::<CalcSettings>(&bad).is_err());
+    }
+
     #[test]
     fn custom_ayanamsa_round_trips() {
         let a = AyanamsaSystem::Custom {
@@ -331,17 +394,19 @@ mod tests {
 
     #[test]
     fn validate_rejects_bad_values() {
-        let s = CalcSettings {
-            schema_version: 2,
-            ..CalcSettings::default()
-        };
-        assert_eq!(
-            s.validate(),
-            Err(CoreError::UnsupportedSchemaVersion {
-                found: 2,
-                supported: 1
-            })
-        );
+        for found in [1, 3] {
+            let s = CalcSettings {
+                schema_version: found,
+                ..CalcSettings::default()
+            };
+            assert_eq!(
+                s.validate(),
+                Err(CoreError::UnsupportedSchemaVersion {
+                    found,
+                    supported: 2
+                })
+            );
+        }
 
         let s = CalcSettings {
             dasha_year: 0.0,
diff --git a/crates/jyotish-core/src/chart.rs b/crates/jyotish-core/src/chart.rs
index 409b313..8628bce 100644
--- a/crates/jyotish-core/src/chart.rs
+++ b/crates/jyotish-core/src/chart.rs
@@ -556,14 +556,14 @@ mod tests {
         assert!(compute_chart(&mut eph, 625_002.0, 0.0, 0.0, &s).is_ok());
         assert!(compute_chart(&mut eph, 2_817_997.0, 0.0, 0.0, &s).is_ok());
         let bad = CalcSettings {
-            schema_version: 2,
+            schema_version: 1,
             ..CalcSettings::default()
         };
         assert_eq!(
             compute_chart(&mut eph, JD, 0.0, 0.0, &bad),
             Err(CoreError::UnsupportedSchemaVersion {
-                found: 2,
-                supported: 1
+                found: 1,
+                supported: 2
             })
         );
     }
diff --git a/crates/jyotish-core/src/transit.rs b/crates/jyotish-core/src/transit.rs
index e8724c4..84db1ce 100644
--- a/crates/jyotish-core/src/transit.rs
+++ b/crates/jyotish-core/src/transit.rs
@@ -195,14 +195,14 @@ mod tests {
             Err(CoreError::InvalidJulianDay(1e300))
         );
         let old = CalcSettings {
-            schema_version: 2,
+            schema_version: 1,
             ..CalcSettings::default()
         };
         assert_eq!(
             compute_transits(&mut eph, JD, &old, &c.bhavas),
             Err(CoreError::UnsupportedSchemaVersion {
-                found: 2,
-                supported: 1
+                found: 1,
+                supported: 2
             })
         );
     }
diff --git a/app/src-tauri/tests/compute_chart.rs b/app/src-tauri/tests/compute_chart.rs
index 20a33fb..d8731ab 100644
--- a/app/src-tauri/tests/compute_chart.rs
+++ b/app/src-tauri/tests/compute_chart.rs
@@ -104,12 +104,12 @@ fn compute_chart_runs_on_the_engine_thread_and_maps_errors() {
     assert_eq!(err.params["lat"], "91");
 
     let old = CalcSettings {
-        schema_version: 2,
+        schema_version: 1,
         ..CalcSettings::default()
     };
     let err = compute_chart_blocking(&engine, 2_451_545.0, 0.0, 0.0, old).unwrap_err();
     assert_eq!(err.code, "unsupported_schema_version");
-    assert_eq!(err.params["found"], "2");
+    assert_eq!(err.params["found"], "1");
 
     let absurd = CalcSettings {
         ayanamsa: AyanamsaSystem::Custom {
```

Store: sürüm 1 kaydı varsayılanlarla göç eder ve yazılı değeri ezmez; sürüm 2 kaydı toleransla okunur (bilinmeyen anahtar, eksik blok, `null` blok, boş blok, eksik anahtar); tanınmayan seçenek değeri sıfırlanır; açılışta göç edilen ayar geri yazılır, bilinmeyen anahtarlı sürüm 2 kaydı olduğu gibi durur. CSV testindeki sürüm numarası 2 olur.

```diff
diff --git a/crates/store/src/settings.rs b/crates/store/src/settings.rs
index 88a46cb..3f02c04 100644
--- a/crates/store/src/settings.rs
+++ b/crates/store/src/settings.rs
@@ -225,7 +257,7 @@ impl Store {
 mod tests {
     use super::*;
     use crate::db::testutil::store;
-    use jyotish_core::{AyanamsaSystem, CoreError, NodeKind};
+    use jyotish_core::{AyanamsaSystem, CoreError, KarakaScheme, NodeKind};
     use serde_json::json;
 
     fn raman() -> CalcSettings {
@@ -278,7 +310,7 @@ mod tests {
             reset(&bad.to_string()),
             SettingsLoad::Reset {
                 settings: fb.clone(),
-                found_version: Some(1)
+                found_version: Some(2)
             }
         );
         let zero = json!({"schema_version": 0});
@@ -291,6 +323,85 @@ mod tests {
         );
     }
 
+    /// Sürüm 1 kaydı: `advanced` yok (Plan 03–13 böyle yazdı).
+    fn v1_json(settings: &CalcSettings) -> Value {
+        let mut v = serde_json::to_value(settings).unwrap();
+        let obj = v.as_object_mut().unwrap();
+        obj.insert("schema_version".into(), json!(1));
+        obj.remove("advanced");
+        v
+    }
+
+    #[test]
+    fn version_1_settings_migrate_to_2_with_advanced_defaults() {
+        let stored = v1_json(&raman());
+        assert_eq!(
+            load_calc_settings(&stored.to_string(), &CalcSettings::default()),
+            SettingsLoad::Migrated {
+                settings: raman(),
+                from: 1
+            }
+        );
+        assert_eq!(raman().schema_version, 2);
+        assert_eq!(raman().advanced, AdvancedSettings::default());
+        // Göç adımı var olan değeri ezmez, yalnızca eksikleri yazar.
+        let mut partial = v1_json(&raman());
+        partial["advanced"] = json!({"jaimini_co_lords": false});
+        let SettingsLoad::Migrated { settings, from: 1 } =
+            load_calc_settings(&partial.to_string(), &CalcSettings::default())
+        else {
+            panic!("expected a migration");
+        };
+        assert!(!settings.advanced.jaimini_co_lords);
+        assert_eq!(settings.advanced.karaka_scheme, KarakaScheme::Eight);
+    }
+
+    #[test]
+    fn version_2_advanced_block_is_read_tolerantly() {
+        let load = |edit: &dyn Fn(&mut Value)| {
+            let mut v = serde_json::to_value(raman()).unwrap();
+            edit(&mut v);
+            load_calc_settings(&v.to_string(), &CalcSettings::default())
+        };
+        // Bilinmeyen anahtar yok sayılır.
+        assert_eq!(
+            load(&|v| v["advanced"]["future_option"] = json!([1, 2])),
+            SettingsLoad::Current(raman())
+        );
+        // Eksik blok, nesne olmayan blok ve eksik anahtar varsayılanla dolar.
+        for edit in [
+            (&|v: &mut Value| {
+                v.as_object_mut().unwrap().remove("advanced");
+            }) as &dyn Fn(&mut Value),
+            &|v| v["advanced"] = json!(null),
+            &|v| v["advanced"] = json!({}),
+            &|v| v["advanced"] = json!({"karaka_scheme": "Eight"}),
+        ] {
+            assert_eq!(load(edit), SettingsLoad::Current(raman()));
+        }
+        // Yazılı değer korunur.
+        let seven = CalcSettings {
+            advanced: AdvancedSettings {
+                karaka_scheme: KarakaScheme::Seven,
+                jaimini_co_lords: false,
+            },
+            ..raman()
+        };
+        assert_eq!(
+            load(&|v| v["advanced"] =
+                json!({"karaka_scheme": "Seven", "jaimini_co_lords": false, "x": 1})),
+            SettingsLoad::Current(seven)
+        );
+        // Tanınmayan seçenek değeri tolerans kapsamında değildir: ayar sıfırlanır, uyarı verilir.
+        assert_eq!(
+            load(&|v| v["advanced"]["karaka_scheme"] = json!("Nine")),
+            SettingsLoad::Reset {
+                settings: CalcSettings::default(),
+                found_version: Some(2)
+            }
+        );
+    }
+
     #[derive(Debug, PartialEq, serde::Deserialize)]
     struct V3 {
         schema_version: u32,
diff --git a/crates/store/src/open.rs b/crates/store/src/open.rs
index ab3ff1c..fcb23db 100644
--- a/crates/store/src/open.rs
+++ b/crates/store/src/open.rs
@@ -324,6 +324,59 @@ mod tests {
         assert_eq!(stored, future);
     }
 
+    #[test]
+    fn version_1_settings_migrate_on_open_and_unknown_advanced_keys_do_not_block() {
+        let mut s = store();
+        let (a, ch) = s.create_client(&new_client("A")).unwrap();
+        let stored_json = |s: &Store| -> String {
+            s.conn
+                .query_row(
+                    "SELECT calc_settings_json FROM charts WHERE id = ?1",
+                    params![ch.id],
+                    |r| r.get(0),
+                )
+                .unwrap()
+        };
+        let write = |s: &Store, json: &str| {
+            s.conn
+                .execute(
+                    "UPDATE charts SET calc_settings_json = ?1 WHERE id = ?2",
+                    params![json, ch.id],
+                )
+                .unwrap();
+        };
+        // Sürüm 1 kaydı (Plan 03–13'ün yazdığı biçim): `advanced` yok.
+        let v1 = concat!(
+            r#"{"schema_version":1,"zodiac":"Sidereal","ayanamsa":"Raman","node":"Mean","#,
+            r#""house_system":"WholeSign","dasha_year":365.25,"combustion_orbs":{"moon":12.0,"#,
+            r#""mars":17.0,"mercury":14.0,"mercury_retro":12.0,"jupiter":11.0,"venus":10.0,"#,
+            r#""venus_retro":8.0,"saturn":15.0},"sunrise_def":"DiskCenterNoRefraction"}"#
+        );
+        write(&s, v1);
+        let raman = CalcSettings {
+            ayanamsa: AyanamsaSystem::Raman,
+            ..CalcSettings::default()
+        };
+        let o = s.open_client(a.id, None).unwrap();
+        assert_eq!(o.calc_settings, raman);
+        assert_eq!(o.calc_settings.schema_version, 2);
+        assert!(o.warnings.is_empty());
+        // Göç edilen ayar sürüm 2 olarak geri yazılır.
+        assert_eq!(stored_json(&s), serde_json::to_string(&raman).unwrap());
+
+        // Sürüm 2 kaydında bilinmeyen `advanced` anahtarı: açılış sürer, uyarı yok, kayıt
+        // olduğu gibi durur (daha yeni bir sürümün alanı kaybolmaz).
+        let with_extra = serde_json::to_string(&raman).unwrap().replace(
+            r#""jaimini_co_lords":true"#,
+            r#""jaimini_co_lords":true,"future_option":3"#,
+        );
+        write(&s, &with_extra);
+        let o = s.open_client(a.id, None).unwrap();
+        assert_eq!(o.calc_settings, raman);
+        assert!(o.warnings.is_empty());
+        assert_eq!(stored_json(&s), with_extra);
+    }
+
     #[test]
     fn tzdb_change_is_checked_and_reported() {
         let mut s = store();
diff --git a/crates/store/src/csv.rs b/crates/store/src/csv.rs
index 6316c75..deebbbf 100644
--- a/crates/store/src/csv.rs
+++ b/crates/store/src/csv.rs
@@ -158,7 +158,7 @@ mod tests {
         let lines: Vec<&str> = csv.trim_end_matches("\r\n").split("\r\n").collect();
         assert_eq!(lines.len(), 2);
         let expected_start = format!(
-            "{},\"Yılmaz, Ayşe\",,{},,Natal,2000-01-01,17:30:00,Exact,New Delhi,India,NCT,28.6139,77.209,Iana,Asia/Kolkata,,,2000-01-01T12:00:00Z,19800,{},a; b,\"doğum belgesi, 1. sayfa\",\"{{\"\"schema_version\"\":1,",
+            "{},\"Yılmaz, Ayşe\",,{},,Natal,2000-01-01,17:30:00,Exact,New Delhi,India,NCT,28.6139,77.209,Iana,Asia/Kolkata,,,2000-01-01T12:00:00Z,19800,{},a; b,\"doğum belgesi, 1. sayfa\",\"{{\"\"schema_version\"\":2,",
             c.uuid,
             ch.uuid,
             atlas::tzdb_version()
```

Arayüz: `advanced` tek alan olarak karşılaştırılır ve rozet onu adlandırır.

```diff
diff --git a/app/ui/settings/model.test.ts b/app/ui/settings/model.test.ts
index b21f24e..6c2725f 100644
--- a/app/ui/settings/model.test.ts
+++ b/app/ui/settings/model.test.ts
@@ -18,6 +18,12 @@ test("differences are listed in field order", () => {
   expect(settingsDiff(base(), b)).toEqual(["ayanamsa", "node", "combustion_orbs"]);
 });
 
+test("the advanced block is compared as one field (fark rozeti)", () => {
+  const b: CalcSettings = { ...base(), advanced: { ...base().advanced, karaka_scheme: "Seven" } };
+  expect(base().advanced).toEqual({ karaka_scheme: "Eight", jaimini_co_lords: true });
+  expect(settingsDiff(base(), b)).toEqual(["advanced"]);
+});
+
 test("orb text accepts a decimal comma and rejects junk or out of range", () => {
   expect(parseOrb("8")).toBe(8);
   expect(parseOrb(" 8,5 ")).toBe(8.5);
diff --git a/app/ui/shell/ChartTitle.test.tsx b/app/ui/shell/ChartTitle.test.tsx
index 824a129..72e0aa6 100644
--- a/app/ui/shell/ChartTitle.test.tsx
+++ b/app/ui/shell/ChartTitle.test.tsx
@@ -20,3 +20,10 @@ test("the difference badge names the differing fields and opens the settings", (
   fireEvent.click(badge);
   expect(onSettings).toHaveBeenCalledTimes(1);
 });
+
+test("a differing advanced setting is named in the badge", () => {
+  const opened = fixture().opened;
+  opened.calc_settings = { ...opened.calc_settings, advanced: { karaka_scheme: "Eight", jaimini_co_lords: false } };
+  renderWithStores(<ChartTitle onSettings={() => {}} />, { opened });
+  expect(screen.getByRole("button", { name: /Ayar farkı/ })).toHaveAttribute("title", "Bu haritanın ayarı varsayılandan farklı: İleri modül ayarları");
+});
```

- [ ] **Step 3: Testlerin başarısız olduğunu doğrula**

Run: `cargo test -p jyotish-core --lib settings 2>&1 | grep -E "^error" | head -3`
Expected: derleme hatası: `cannot find struct, variant or union type AdvancedSettings`, `cannot find type KarakaScheme`, `no field advanced on type settings::CalcSettings`.

- [ ] **Step 4: Çekirdek: `AdvancedSettings`, `KarakaScheme`, sürüm 2**

```diff
diff --git a/crates/jyotish-core/src/settings.rs b/crates/jyotish-core/src/settings.rs
index 74a1fc4..968e07a 100644
--- a/crates/jyotish-core/src/settings.rs
+++ b/crates/jyotish-core/src/settings.rs
@@ -6,7 +6,9 @@ use serde::{Deserialize, Serialize};
 
 use crate::CoreError;
 
-pub const CALC_SETTINGS_SCHEMA_VERSION: u32 = 1;
+/// Sürüm 2 (Alt proje 3.1): `advanced` alt yapısı eklendi. Sürüm 1 kayıtlarını `store` göç
+/// ettirir (`store::settings::CALC_SETTINGS_MIGRATIONS`).
+pub const CALC_SETTINGS_SCHEMA_VERSION: u32 = 2;
 
 #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
 #[cfg_attr(feature = "specta", derive(specta::Type))]
@@ -148,6 +150,34 @@ impl SunriseDef {
     }
 }
 
+/// Chara karaka şeması (ileri modüller spec'i §5): sekizli şemada Rahu da karaka olur.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
+#[cfg_attr(feature = "specta", derive(specta::Type))]
+pub enum KarakaScheme {
+    Eight,
+    Seven,
+}
+
+/// İleri modüllerin ayarları (ileri modüller spec'i K7). Yalnızca en az iki uygulanmış seçeneği
+/// olan kararlar burada durur. Bilinmeyen anahtarlar okunurken yok sayılır; eksik anahtarları
+/// `store` varsayılanla doldurur. IPC'de her alan zorunludur.
+#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+#[cfg_attr(feature = "specta", derive(specta::Type))]
+pub struct AdvancedSettings {
+    pub karaka_scheme: KarakaScheme,
+    /// Akrep için Ketu, Kova için Rahu eş lord sayılır.
+    pub jaimini_co_lords: bool,
+}
+
+impl Default for AdvancedSettings {
+    fn default() -> Self {
+        Self {
+            karaka_scheme: KarakaScheme::Eight,
+            jaimini_co_lords: true,
+        }
+    }
+}
+
 #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
 #[cfg_attr(feature = "specta", derive(specta::Type))]
 pub struct CalcSettings {
@@ -160,6 +190,7 @@ pub struct CalcSettings {
     pub dasha_year: f64,
     pub combustion_orbs: CombustionOrbs,
     pub sunrise_def: SunriseDef,
+    pub advanced: AdvancedSettings,
 }
 
 impl Default for CalcSettings {
@@ -173,6 +204,7 @@ impl Default for CalcSettings {
             dasha_year: 365.25,
             combustion_orbs: CombustionOrbs::default(),
             sunrise_def: SunriseDef::DiskCenterNoRefraction,
+            advanced: AdvancedSettings::default(),
         }
     }
 }
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index a912e88..d10db85 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -42,8 +42,8 @@ pub use panchang::{
     END_BISECTIONS, END_SEARCH_DAYS, KARANA_SPAN_DEG, SUNRISE_WINDOW_DAYS, TITHI_SPAN_DEG,
 };
 pub use settings::{
-    AyanamsaSystem, CalcSettings, CombustionOrbs, HouseSystem, NodeKind, SunriseDef, ZodiacMode,
-    CALC_SETTINGS_SCHEMA_VERSION, JD_UT_RANGE,
+    AdvancedSettings, AyanamsaSystem, CalcSettings, CombustionOrbs, HouseSystem, KarakaScheme,
+    NodeKind, SunriseDef, ZodiacMode, CALC_SETTINGS_SCHEMA_VERSION, JD_UT_RANGE,
 };
 pub use transit::{compute_transits, TransitGraha, TransitSnapshot};
 pub use varga::{
```

- [ ] **Step 5: Store: `fill_advanced` göç adımı ve toleranslı okuma**

```diff
diff --git a/crates/store/src/settings.rs b/crates/store/src/settings.rs
index 88a46cb..3f02c04 100644
--- a/crates/store/src/settings.rs
+++ b/crates/store/src/settings.rs
@@ -2,7 +2,7 @@
 //! (`AppPrefs`, spec §5). Göç başarısız olursa genel varsayılan kullanılır ve uyarı verilir;
 //! saklanan JSON üzerine yazılmaz (daha yeni bir sürümün verisi kaybolmasın).
 
-use jyotish_core::{CalcSettings, CALC_SETTINGS_SCHEMA_VERSION};
+use jyotish_core::{AdvancedSettings, CalcSettings, CALC_SETTINGS_SCHEMA_VERSION};
 use serde::de::DeserializeOwned;
 use serde::{Deserialize, Serialize};
 use serde_json::Value;
@@ -13,9 +13,34 @@ use crate::{Store, StoreError};
 /// çağıran yazar; adım yalnızca alanları dönüştürür.
 pub type ValueMigration = fn(Value) -> Result<Value, &'static str>;
 
-/// `CalcSettings` göçleri: indeks `i`, `i + 1 → i + 2` adımıdır. v1 tek sürümdür; liste
-/// boştur. v2 geldiğinde `CALC_SETTINGS_SCHEMA_VERSION` 2 olur ve buraya bir adım eklenir.
-pub const CALC_SETTINGS_MIGRATIONS: &[ValueMigration] = &[];
+/// `CalcSettings` göçleri: indeks `i`, `i + 1 → i + 2` adımıdır. Sürüm 1 → 2 `advanced`
+/// alt yapısını varsayılanlarla ekler (ileri modüller spec'i K7).
+pub const CALC_SETTINGS_MIGRATIONS: &[ValueMigration] = &[fill_advanced];
+
+/// `advanced` alt yapısını tamamlar: yoksa ya da nesne değilse varsayılanların tamamı, varsa
+/// yalnızca eksik anahtarlar yazılır. Var olan değerlere ve bilinmeyen anahtarlara dokunulmaz.
+/// Hem sürüm 1 → 2 göç adımıdır hem de sürüm 2 kayıtlarının toleranslı okumasıdır.
+fn fill_advanced(mut v: Value) -> Result<Value, &'static str> {
+    let obj = v.as_object_mut().ok_or("not an object")?;
+    let Ok(Value::Object(defaults)) = serde_json::to_value(AdvancedSettings::default()) else {
+        return Err("advanced defaults");
+    };
+    match obj.get_mut("advanced").and_then(Value::as_object_mut) {
+        Some(stored) => {
+            for (key, value) in defaults {
+                stored.entry(key).or_insert(value);
+            }
+        }
+        None => {
+            obj.insert("advanced".into(), Value::Object(defaults));
+        }
+    }
+    Ok(v)
+}
+
+fn stored_version(value: &Value) -> Option<u64> {
+    value.get("schema_version").and_then(Value::as_u64)
+}
 
 /// Ayar okumasının sonucu.
 #[derive(Debug, Clone, PartialEq)]
@@ -72,6 +97,13 @@ pub(crate) fn migrate_value<T: DeserializeOwned>(
 }
 
 pub(crate) fn load_calc_value(value: Value, fallback: &CalcSettings) -> SettingsLoad {
+    // Güncel sürümlü kayıt göç adımından geçmez; eksik `advanced` anahtarları burada dolar.
+    // Tamamlanan kayıt `Current` sayılır ve geri yazılmaz (her okumada aynı sonuç).
+    let value = if stored_version(&value) == Some(u64::from(CALC_SETTINGS_SCHEMA_VERSION)) {
+        fill_advanced(value.clone()).unwrap_or(value)
+    } else {
+        value
+    };
     match migrate_value::<CalcSettings>(
         value,
         CALC_SETTINGS_SCHEMA_VERSION,
```

- [ ] **Step 6: Golden fixture'ların `settings` bloğunu sürüm 2'ye çevir (D4)**

Beş fixture'da, README'de ve üretici betikte aynı iki satır değişir; beklenen değerlere dokunulmaz.

```diff
diff --git a/tests/golden/buenos-aires-1923-06-15.json b/tests/golden/buenos-aires-1923-06-15.json
index ad4f2f6..e6ba79f 100644
--- a/tests/golden/buenos-aires-1923-06-15.json
+++ b/tests/golden/buenos-aires-1923-06-15.json
@@ -4,7 +4,7 @@
   "source_version": "2.10.03",
   "notes": "güney yarıküre, UT gece yarısı; swetest -sid1 (Lahiri), mean node, UT 15.6.1923 0:00; d9_signs ve dasha awk ile türetildi (Plan 06); panchang swetest -rise -hindu ve ikiye bölmeyle, sav awk ile türetildi (Plan 07)",
   "settings": {
-    "schema_version": 1,
+    "schema_version": 2,
     "zodiac": "Sidereal",
     "ayanamsa": "Lahiri",
     "node": "Mean",
@@ -20,7 +20,8 @@
       "venus_retro": 8.0,
       "saturn": 15.0
     },
-    "sunrise_def": "DiskCenterNoRefraction"
+    "sunrise_def": "DiskCenterNoRefraction",
+    "advanced": { "karaka_scheme": "Eight", "jaimini_co_lords": true }
   },
   "input": { "jd_ut": 2423585.5000000000, "lat": -34.6037, "lon": -58.3816 },
   "expected": {
diff --git a/tests/golden/istanbul-1987-03-17.json b/tests/golden/istanbul-1987-03-17.json
index 23f6842..8d07de6 100644
--- a/tests/golden/istanbul-1987-03-17.json
+++ b/tests/golden/istanbul-1987-03-17.json
@@ -4,7 +4,7 @@
   "source_version": "2.10.03",
   "notes": "Türkiye, gün sonu; swetest -sid1 (Lahiri), mean node, UT 17.3.1987 23:59; d9_signs ve dasha awk ile türetildi (Plan 06); panchang swetest -rise -hindu ve ikiye bölmeyle, sav awk ile türetildi (Plan 07)",
   "settings": {
-    "schema_version": 1,
+    "schema_version": 2,
     "zodiac": "Sidereal",
     "ayanamsa": "Lahiri",
     "node": "Mean",
@@ -20,7 +20,8 @@
       "venus_retro": 8.0,
       "saturn": 15.0
     },
-    "sunrise_def": "DiskCenterNoRefraction"
+    "sunrise_def": "DiskCenterNoRefraction",
+    "advanced": { "karaka_scheme": "Eight", "jaimini_co_lords": true }
   },
   "input": { "jd_ut": 2446872.4993055556, "lat": 41.0082, "lon": 28.9784 },
   "expected": {
diff --git a/tests/golden/london-1850-01-01.json b/tests/golden/london-1850-01-01.json
index 4d29d57..58edc8f 100644
--- a/tests/golden/london-1850-01-01.json
+++ b/tests/golden/london-1850-01-01.json
@@ -4,7 +4,7 @@
   "source_version": "2.10.03",
   "notes": "19. yüzyıl, kuzey yarıküre, batı boylamı; swetest -sid1 (Lahiri), mean node, UT 1.1.1850 6:30; d9_signs ve dasha awk ile türetildi (Plan 06); panchang swetest -rise -hindu ve ikiye bölmeyle, sav awk ile türetildi (Plan 07)",
   "settings": {
-    "schema_version": 1,
+    "schema_version": 2,
     "zodiac": "Sidereal",
     "ayanamsa": "Lahiri",
     "node": "Mean",
@@ -20,7 +20,8 @@
       "venus_retro": 8.0,
       "saturn": 15.0
     },
-    "sunrise_def": "DiskCenterNoRefraction"
+    "sunrise_def": "DiskCenterNoRefraction",
+    "advanced": { "karaka_scheme": "Eight", "jaimini_co_lords": true }
   },
   "input": { "jd_ut": 2396758.7708333335, "lat": 51.5074, "lon": -0.1278 },
   "expected": {
diff --git a/tests/golden/new-delhi-2000-01-01.json b/tests/golden/new-delhi-2000-01-01.json
index 0267ad5..89900ed 100644
--- a/tests/golden/new-delhi-2000-01-01.json
+++ b/tests/golden/new-delhi-2000-01-01.json
@@ -4,7 +4,7 @@
   "source_version": "2.10.03",
   "notes": "J2000; swetest -sid1 (Lahiri), mean node, UT 1.1.2000 12:00; d9_signs ve dasha awk ile türetildi (Plan 06); panchang swetest -rise -hindu ve ikiye bölmeyle, sav awk ile türetildi (Plan 07)",
   "settings": {
-    "schema_version": 1,
+    "schema_version": 2,
     "zodiac": "Sidereal",
     "ayanamsa": "Lahiri",
     "node": "Mean",
@@ -20,7 +20,8 @@
       "venus_retro": 8.0,
       "saturn": 15.0
     },
-    "sunrise_def": "DiskCenterNoRefraction"
+    "sunrise_def": "DiskCenterNoRefraction",
+    "advanced": { "karaka_scheme": "Eight", "jaimini_co_lords": true }
   },
   "input": { "jd_ut": 2451545.0000000000, "lat": 28.6139, "lon": 77.2090 },
   "expected": {
diff --git a/tests/golden/tromso-2050-12-21.json b/tests/golden/tromso-2050-12-21.json
index 6efdec4..b0fd701 100644
--- a/tests/golden/tromso-2050-12-21.json
+++ b/tests/golden/tromso-2050-12-21.json
@@ -4,7 +4,7 @@
   "source_version": "2.10.03",
   "notes": "yüksek enlem (kutup dairesi içi); swetest -sid1 (Lahiri), mean node, UT 21.12.2050 18:15; d9_signs ve dasha awk ile türetildi (Plan 06); panchang swetest -rise -hindu ve ikiye bölmeyle, sav awk ile türetildi (Plan 07)",
   "settings": {
-    "schema_version": 1,
+    "schema_version": 2,
     "zodiac": "Sidereal",
     "ayanamsa": "Lahiri",
     "node": "Mean",
@@ -20,7 +20,8 @@
       "venus_retro": 8.0,
       "saturn": 15.0
     },
-    "sunrise_def": "DiskCenterNoRefraction"
+    "sunrise_def": "DiskCenterNoRefraction",
+    "advanced": { "karaka_scheme": "Eight", "jaimini_co_lords": true }
   },
   "input": { "jd_ut": 2470162.2604166665, "lat": 69.6492, "lon": 18.9553 },
   "expected": {
diff --git a/tests/golden/README.md b/tests/golden/README.md
index c7b1a8a..27dacfd 100644
--- a/tests/golden/README.md
+++ b/tests/golden/README.md
@@ -22,7 +22,7 @@ ayanamsa tanımı, nütasyon, mean/true düğüm, UT/ET.
   "source": "swetest | jhora",
   "source_version": "2.10.03",
   "notes": "yerel saat, saat dilimi, neden seçildiği",
-  "settings": { "...": "CalcSettings, tam döküm (schema_version 1)" },
+  "settings": { "...": "CalcSettings, tam döküm (schema_version 2, `advanced` bloğuyla)" },
   "input": { "jd_ut": 2451545.0, "lat": 28.6139, "lon": 77.2090 },
   "expected": {
     "ayanamsa_deg": 23.8532225,
diff --git a/scripts/gen-golden-swetest.sh b/scripts/gen-golden-swetest.sh
index beb9572..afcb9bc 100755
--- a/scripts/gen-golden-swetest.sh
+++ b/scripts/gen-golden-swetest.sh
@@ -171,7 +171,7 @@ for row in "${CHARTS[@]}"; do
   "source_version": "$SE_VERSION",
   "notes": "$note; swetest -sid1 (Lahiri), mean node, UT $d $t; d9_signs ve dasha awk ile türetildi (Plan 06); panchang swetest -rise -hindu ve ikiye bölmeyle, sav awk ile türetildi (Plan 07)",
   "settings": {
-    "schema_version": 1,
+    "schema_version": 2,
     "zodiac": "Sidereal",
     "ayanamsa": "Lahiri",
     "node": "Mean",
@@ -187,7 +187,8 @@ for row in "${CHARTS[@]}"; do
       "venus_retro": 8.0,
       "saturn": 15.0
     },
-    "sunrise_def": "DiskCenterNoRefraction"
+    "sunrise_def": "DiskCenterNoRefraction",
+    "advanced": { "karaka_scheme": "Eight", "jaimini_co_lords": true }
   },
   "input": { "jd_ut": $jd, "lat": $lat, "lon": $lon },
   "expected": {
```

Run: `cargo test -p jyotish-core -p store 2>&1 | grep -E "^test result"`
Expected: `jyotish-core` lib `111 passed`, golden `8 passed`; `store` lib `84 passed; 0 failed; 1 ignored`.

- [ ] **Step 7: Bağlamaları ve fikstürü yeniden üret; arayüzde `advanced` alanı**

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
git diff --stat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
```
Expected: iki `ok`; `bindings.ts` 15 satır eklenir (`AdvancedSettings`, `KarakaScheme`, `CalcSettings.advanced`); `delhi.json` tek satır değişir (ayar blokları `"schema_version":2` ve `"advanced":{"jaimini_co_lords":true,"karaka_scheme":"Eight"}` taşır).

```diff
diff --git a/app/ui/settings/model.ts b/app/ui/settings/model.ts
index 0af3b0a..2c4eae6 100644
--- a/app/ui/settings/model.ts
+++ b/app/ui/settings/model.ts
@@ -15,7 +15,7 @@ export const ORB_KEYS = ["moon", "mars", "mercury", "mercury_retro", "jupiter",
 export type OrbKey = (typeof ORB_KEYS)[number];
 
 /** Karşılaştırılan ve gösterilen alanlar, pencere sırasıyla (`schema_version` hariç). */
-export const SETTINGS_FIELDS = ["zodiac", "ayanamsa", "node", "house_system", "dasha_year", "combustion_orbs", "sunrise_def"] as const satisfies readonly (keyof CalcSettings)[];
+export const SETTINGS_FIELDS = ["zodiac", "ayanamsa", "node", "house_system", "dasha_year", "combustion_orbs", "sunrise_def", "advanced"] as const satisfies readonly (keyof CalcSettings)[];
 export type SettingsField = (typeof SETTINGS_FIELDS)[number];
 
 /** Yapısal eşitlik; nesne anahtarlarının sırası önemsizdir. */
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index bde9cca..371c5f5 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -424,6 +424,7 @@ export const tr = {
       dasha_year: "Dasa yılı",
       combustion_orbs: "Yanma orbları (derece)",
       sunrise_def: "Gün doğumu",
+      advanced: "İleri modül ayarları",
     },
     zodiac: { Sidereal: "Sidereal", TropicalRasiSiderealNakshatra: "Tropik rasi, sidereal nakshatra" },
     ayanamsa: { Lahiri: "Lahiri", Raman: "Raman", Kp: "KP (Krishnamurti)", TrueChitra: "True Chitrapaksha", Custom: "Özel" },
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 1d0753d..af15dc4 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -425,6 +425,7 @@ export const en: Messages = {
       dasha_year: "Dasha year",
       combustion_orbs: "Combustion orbs (degrees)",
       sunrise_def: "Sunrise",
+      advanced: "Advanced module settings",
     },
     zodiac: { Sidereal: "Sidereal", TropicalRasiSiderealNakshatra: "Tropical rasi, sidereal nakshatra" },
     ayanamsa: { Lahiri: "Lahiri", Raman: "Raman", Kp: "KP (Krishnamurti)", TrueChitra: "True Chitrapaksha", Custom: "Custom" },
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 947ddb5..3f48c61 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -425,6 +425,7 @@ export const fr: Messages = {
       dasha_year: "Année de dasha",
       combustion_orbs: "Orbes de combustion (degrés)",
       sunrise_def: "Lever du soleil",
+      advanced: "Réglages des modules avancés",
     },
     zodiac: { Sidereal: "Sidéral", TropicalRasiSiderealNakshatra: "Rasi tropical, nakshatra sidéral" },
     ayanamsa: { Lahiri: "Lahiri", Raman: "Raman", Kp: "KP (Krishnamurti)", TrueChitra: "True Chitrapaksha", Custom: "Personnalisé" },
```

- [ ] **Step 8: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | tail -1
git diff --stat -- crates/store/src/schema.rs Cargo.lock app/pnpm-lock.yaml
```
Expected: fmt ve clippy çıktısız; `jyotish-core` lib `111 passed`; `store` lib `84 passed; 0 failed; 1 ignored`; diğer Rust satırları tabandaki gibi (`bindings` ve `ui_fixture` yeşil); Vitest `60 passed (60)`, `527 passed (527)`; `✓ built`; son `git diff --stat` çıktısız (şema ve kilit dosyaları değişmedi).

- [ ] **Step 9: Commit**

```bash
git add crates/jyotish-core/src crates/store/src app/src-tauri/tests/compute_chart.rs tests/golden scripts/gen-golden-swetest.sh app/ui/bindings.ts app/ui/test/fixtures/delhi.json app/ui/settings app/ui/shell/ChartTitle.test.tsx app/ui/i18n
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(settings): CalcSettings schema version 2 with a tolerant advanced block; version 1 records migrate on read"
```
Expected: iki `grep` çıktısız (çıkış kodu 1 beklenir); commit mesajı da aynı desenle eşleşmez.

---

### Task 2: Çekirdek: ortak girdi (`ChartInput`, `Positions`), ev lordluğu, doğal sınıf, Kuja dosha; el örneği düzeneği

Spec K2, K3, K4, K6 ve §4 "Ev lordluğu", "Kuja dosha olgusu"; §13 katman 1 ve 2. Hiçbir mevcut davranış değişmez; yalnızca yeni modüller ve testler eklenir.

**Files:**
- Create: `crates/jyotish-core/src/positions.rs`, `crates/jyotish-core/src/lordship.rs`, `crates/jyotish-core/src/nature.rs`, `crates/jyotish-core/tests/modules_golden.rs`, `tests/golden/modules/README.md`, `tests/golden/modules/foundation/*.json` (6 dosya)
- Modify: `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/proptests.rs`
- Test: yeni dosyalardaki `mod tests`, `proptests.rs` (`foundation_properties`), `tests/modules_golden.rs`

**Interfaces:**
- Consumes: `compute_chart`, `Chart`, `Graha`, `Sign::{of, index, from_index, house_from}`, `dignity::sign_lord`, `zodiac::{normalize, deg_in_sign}`, `CalcSettings` (Task 1), `fake::FakeEph` (testlerde).
- Produces (hepsi `jyotish_core`'dan dışa aktarılır):
  - `pub struct ChartInput { pub jd_ut: f64, pub lat: f64, pub lon: f64, pub settings: CalcSettings, pub time_known: bool }`
  - `pub struct GrahaPoint { pub graha: Graha, pub sidereal_lon: f64, pub rasi_sign: Sign, pub deg_in_sign: f64, pub lat: f64, pub speed: f64 }` (`Copy`)
  - `pub struct Positions { pub lagna_lon: Option<f64>, pub lagna_sign: Option<Sign>, pub grahas: [GrahaPoint; 9] }`; `impl From<&Chart> for Positions`
  - `Positions::synthetic(lagna_lon: Option<f64>, lons: [f64; 9]) -> Positions`, `without_lagna(self) -> Positions`, `point(&self, Graha) -> &GrahaPoint`, `sign_of(&self, Graha) -> Sign`, `occupants(&self, Sign) -> Vec<Graha>`, `house_from(&self, Graha, reference: Sign) -> u8`, `house_of(&self, Graha) -> Option<u8>`
  - `pub fn compute_positions<E: Ephemeris + ?Sized>(eph: &mut E, input: &ChartInput) -> Result<Positions, CoreError>`
  - `pub fn house_sign(lagna_sign: Sign, house: u8) -> Sign`, `house_lord(lagna_sign: Sign, house: u8) -> Graha`, `houses_owned(lagna_sign: Sign, graha: Graha) -> Vec<u8>`
  - `pub enum Nature { Benefic, Malefic }`, `pub fn nature(graha: Graha, positions: &Positions) -> Nature`, `pub fn moon_is_waxing(positions: &Positions) -> bool`
  - `pub struct KujaDosha { pub from_lagna: Option<bool>, pub from_moon: bool, pub from_venus: bool }`, `pub const KUJA_DOSHA_HOUSES: [u8; 6]`, `pub fn kuja_dosha(positions: &Positions) -> KujaDosha`
  - El örneği düzeneği: `tests/modules_golden.rs` içinde `Fixture<X>`, `Input`, `load::<X>(module, min)`.

- [ ] **Step 1: El örneklerini yaz (kaynak: `hand`)**

Altı sentetik konum seti. Her biri kural adımlarını `derivation`'da taşır; beklenen değerler bu adımlardan elle çıkarılmıştır (motor koşulmadan önce). Kapsanan durumlar: Merkür yalnız; yalnızca Güneş'le; kötücül ve iyicille birlikte; yalnızca küçülen Ay'la; büyüyen Ay ve Rahu ile; yalnızca Rahu'yla. Ay büyüyen, küçülen ve tam 180° (D6). Kuja dosha'nın altı evi (1, 2, 4, 7, 8, 12) ve dosha olmayan evler; lagnasız harita.

**Yeni dosya:** `tests/golden/modules/README.md`

````markdown
# İleri modüllerin el örnekleri

`crates/jyotish-core/tests/modules_golden.rs` bu dizindeki her modül klasörünü yükler. Kaynak:
ileri modüller tasarım belgesi §13 (dört katmanlı test stratejisinin ilk katmanı).

- Girdi sentetik konumlardır: dokuz grahanın sidereal boylamı ve (varsa) lagnanın sidereal
  boylamı. Ephemeris gerekmez; rasi çerçevesi sidereal sayılır.
- Beklenen değer **klasik kuraldan elle türetilir** ve `derivation` alanında adım adım yazılır.
  Başka bir yazılımın çıktısı buraya girmez; `source` yalnızca `hand` olabilir.
- Bir örnek motorla uyuşmazsa önce kural metni yeniden okunur. Beklenen değer motor çıktısına
  bakarak düzeltilmez.
- `derivation`, örneği yazan dışında bir gözden geçirmede kuraldan yeniden hesaplanır.
- Bilinmeyen alan reddedilir. Dosya adı `id` ile aynıdır. Modül başına en az örnek sayısı test
  dosyasında sabittir (`MIN_*`); örnek silmek testi düşürür.

## Şema

```json
{
  "id": "dosya-adiyla-ayni",
  "source": "hand",
  "rule_ref": "genel gönderme, ör. BPHS, bakışlar bölümü (alıntı yapılmaz)",
  "derivation": ["adım 1", "adım 2"],
  "input": {
    "lagna_lon": 15.0,
    "grahas": { "Sun": 0.0, "Moon": 0.0, "Mars": 0.0, "Mercury": 0.0, "Jupiter": 0.0,
                "Venus": 0.0, "Saturn": 0.0, "Rahu": 0.0, "Ketu": 0.0 }
  },
  "expected": { "...": "modüle göre" }
}
```

`lagna_lon: null` doğum saati bilinmeyen haritadır: lagnaya bağlı her beklenen değer `null` olur.

## Modüller

| Klasör | `expected` | En az |
|---|---|---|
| `foundation` | `waxing`, `nature` (dokuz graha), `kuja_dosha`, `house_lords` (12), `houses`, `houses_owned` | 6 |

Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
````

**Yeni dosya:** `tests/golden/modules/foundation/mercury-alone-waxing-moon.json`

```json
{
  "id": "mercury-alone-waxing-moon",
  "source": "hand",
  "rule_ref": "BPHS, graha nitelikleri bölümü (doğal sınıf); evler lagna burcundan bütün burç sayımı; Kuja dosha: Mars 1, 2, 4, 7, 8, 12",
  "derivation": [
    "Burçlar: Güneş 280° Oğlak, Ay 40° Boğa, Mars 100° Yengeç, Merkür 340° Balık, Jüpiter 130° Aslan, Venüs 310° Kova, Satürn 190° Terazi, Rahu 70° İkizler, Ketu 250° Yay. Lagna 15° Koç.",
    "Ay: uzanım = 40 − 280 = −240 ≡ 120°; 0° ≤ 120° < 180° olduğundan büyüyor, iyicil.",
    "Merkür Balık'ta yalnız: aynı burçta kötücül yok, iyicil.",
    "Kuja dosha: Mars Yengeç'te. Koç'tan Yengeç 4. ev: var. Ay'ın burcu Boğa'dan Yengeç 3. ev: yok. Venüs'ün burcu Kova'dan sayım Kova 1, Balık 2, Koç 3, Boğa 4, İkizler 5, Yengeç 6: yok.",
    "Ev lordları, Koç lagna (evler Koç…Balık): Mars, Venüs, Merkür, Ay, Güneş, Merkür, Venüs, Mars, Jüpiter, Satürn, Satürn, Jüpiter.",
    "Evler, Koç'tan: Oğlak 10, Boğa 2, Yengeç 4, Balık 12, Aslan 5, Kova 11, Terazi 7, İkizler 3, Yay 9."
  ],
  "input": {
    "lagna_lon": 15.0,
    "grahas": {"Sun": 280.0, "Moon": 40.0, "Mars": 100.0, "Mercury": 340.0, "Jupiter": 130.0, "Venus": 310.0, "Saturn": 190.0, "Rahu": 70.0, "Ketu": 250.0}
  },
  "expected": {
    "waxing": true,
    "nature": {"Sun": "Malefic", "Moon": "Benefic", "Mars": "Malefic", "Mercury": "Benefic", "Jupiter": "Benefic", "Venus": "Benefic", "Saturn": "Malefic", "Rahu": "Malefic", "Ketu": "Malefic"},
    "kuja_dosha": {"from_lagna": true, "from_moon": false, "from_venus": false},
    "house_lords": ["Mars", "Venus", "Mercury", "Moon", "Sun", "Mercury", "Venus", "Mars", "Jupiter", "Saturn", "Saturn", "Jupiter"],
    "houses": {"Sun": 10, "Moon": 2, "Mars": 4, "Mercury": 12, "Jupiter": 5, "Venus": 11, "Saturn": 7, "Rahu": 3, "Ketu": 9},
    "houses_owned": {"Sun": [5], "Moon": [4], "Mars": [1, 8], "Mercury": [3, 6], "Jupiter": [9, 12], "Venus": [2, 7], "Saturn": [10, 11], "Rahu": [], "Ketu": []}
  }
}
```

**Yeni dosya:** `tests/golden/modules/foundation/mercury-with-sun-only.json`

```json
{
  "id": "mercury-with-sun-only",
  "source": "hand",
  "rule_ref": "BPHS, graha nitelikleri bölümü (doğal sınıf); evler lagna burcundan bütün burç sayımı; Kuja dosha: Mars 1, 2, 4, 7, 8, 12",
  "derivation": [
    "Burçlar: Güneş 155° Başak, Ay 130° Aslan, Mars 280° Oğlak, Merkür 170° Başak, Jüpiter 20° Koç, Venüs 190° Terazi, Satürn 310° Kova, Rahu 40° Boğa, Ketu 220° Akrep. Lagna 100° Yengeç.",
    "Ay: uzanım = 130 − 155 = −25 ≡ 335°; 180°'den büyük olduğundan küçülüyor, kötücül.",
    "Merkür Başak'ta yalnızca Güneş'le birlikte: kötücül var, iyicil yok; kötücül.",
    "Kuja dosha: Mars Oğlak'ta. Yengeç'ten Oğlak 7. ev: var. Aslan'dan sayım Aslan 1, Başak 2, Terazi 3, Akrep 4, Yay 5, Oğlak 6: yok. Terazi'den sayım Terazi 1, Akrep 2, Yay 3, Oğlak 4: var.",
    "Ev lordları, Yengeç lagna (evler Yengeç, Aslan, Başak, Terazi, Akrep, Yay, Oğlak, Kova, Balık, Koç, Boğa, İkizler): Ay, Güneş, Merkür, Venüs, Mars, Jüpiter, Satürn, Satürn, Jüpiter, Mars, Venüs, Merkür.",
    "Evler, Yengeç'ten: Başak 3, Aslan 2, Oğlak 7, Başak 3, Koç 10, Terazi 4, Kova 8, Boğa 11, Akrep 5."
  ],
  "input": {
    "lagna_lon": 100.0,
    "grahas": {"Sun": 155.0, "Moon": 130.0, "Mars": 280.0, "Mercury": 170.0, "Jupiter": 20.0, "Venus": 190.0, "Saturn": 310.0, "Rahu": 40.0, "Ketu": 220.0}
  },
  "expected": {
    "waxing": false,
    "nature": {"Sun": "Malefic", "Moon": "Malefic", "Mars": "Malefic", "Mercury": "Malefic", "Jupiter": "Benefic", "Venus": "Benefic", "Saturn": "Malefic", "Rahu": "Malefic", "Ketu": "Malefic"},
    "kuja_dosha": {"from_lagna": true, "from_moon": false, "from_venus": true},
    "house_lords": ["Moon", "Sun", "Mercury", "Venus", "Mars", "Jupiter", "Saturn", "Saturn", "Jupiter", "Mars", "Venus", "Mercury"],
    "houses": {"Sun": 3, "Moon": 2, "Mars": 7, "Mercury": 3, "Jupiter": 10, "Venus": 4, "Saturn": 8, "Rahu": 11, "Ketu": 5},
    "houses_owned": {"Sun": [2], "Moon": [1], "Mars": [5, 10], "Mercury": [3, 12], "Jupiter": [6, 9], "Venus": [4, 11], "Saturn": [7, 8], "Rahu": [], "Ketu": []}
  }
}
```

**Yeni dosya:** `tests/golden/modules/foundation/mercury-with-malefic-and-benefic-full-moon.json`

```json
{
  "id": "mercury-with-malefic-and-benefic-full-moon",
  "source": "hand",
  "rule_ref": "BPHS, graha nitelikleri bölümü (doğal sınıf); evler lagna burcundan bütün burç sayımı; Kuja dosha: Mars 1, 2, 4, 7, 8, 12",
  "derivation": [
    "Burçlar: Güneş 100° Yengeç, Ay 280° Oğlak, Mars 250° Yay, Merkür 65° İkizler, Jüpiter 85° İkizler, Venüs 130° Aslan, Satürn 62° İkizler, Rahu 10° Koç, Ketu 190° Terazi. Lagna 250° Yay.",
    "Ay: uzanım = 280 − 100 = 180°; büyüme aralığı [0°, 180°) olduğundan tam 180° küçülen yarıya aittir, kötücül.",
    "Merkür İkizler'de Satürn (kötücül) ve Jüpiter (iyicil) ile: iyicil bulunduğundan iyicil.",
    "Kuja dosha: Mars Yay'da. Yay'dan Yay 1. ev: var. Oğlak'tan Yay 12. ev: var. Aslan'dan sayım Aslan 1, Başak 2, Terazi 3, Akrep 4, Yay 5: yok.",
    "Ev lordları, Yay lagna (evler Yay, Oğlak, Kova, Balık, Koç, Boğa, İkizler, Yengeç, Aslan, Başak, Terazi, Akrep): Jüpiter, Satürn, Satürn, Jüpiter, Mars, Venüs, Merkür, Ay, Güneş, Merkür, Venüs, Mars.",
    "Evler, Yay'dan: Yengeç 8, Oğlak 2, Yay 1, İkizler 7, İkizler 7, Aslan 9, İkizler 7, Koç 5, Terazi 11."
  ],
  "input": {
    "lagna_lon": 250.0,
    "grahas": {"Sun": 100.0, "Moon": 280.0, "Mars": 250.0, "Mercury": 65.0, "Jupiter": 85.0, "Venus": 130.0, "Saturn": 62.0, "Rahu": 10.0, "Ketu": 190.0}
  },
  "expected": {
    "waxing": false,
    "nature": {"Sun": "Malefic", "Moon": "Malefic", "Mars": "Malefic", "Mercury": "Benefic", "Jupiter": "Benefic", "Venus": "Benefic", "Saturn": "Malefic", "Rahu": "Malefic", "Ketu": "Malefic"},
    "kuja_dosha": {"from_lagna": true, "from_moon": true, "from_venus": false},
    "house_lords": ["Jupiter", "Saturn", "Saturn", "Jupiter", "Mars", "Venus", "Mercury", "Moon", "Sun", "Mercury", "Venus", "Mars"],
    "houses": {"Sun": 8, "Moon": 2, "Mars": 1, "Mercury": 7, "Jupiter": 7, "Venus": 9, "Saturn": 7, "Rahu": 5, "Ketu": 11},
    "houses_owned": {"Sun": [9], "Moon": [8], "Mars": [5, 12], "Mercury": [7, 10], "Jupiter": [1, 4], "Venus": [6, 11], "Saturn": [2, 3], "Rahu": [], "Ketu": []}
  }
}
```

**Yeni dosya:** `tests/golden/modules/foundation/mercury-with-waning-moon-only.json`

```json
{
  "id": "mercury-with-waning-moon-only",
  "source": "hand",
  "rule_ref": "BPHS, graha nitelikleri bölümü (doğal sınıf); evler lagna burcundan bütün burç sayımı; Kuja dosha: Mars 1, 2, 4, 7, 8, 12",
  "derivation": [
    "Burçlar: Güneş 200° Terazi, Ay 170° Başak, Mars 325° Kova, Merkür 175° Başak, Jüpiter 40° Boğa, Venüs 230° Akrep, Satürn 100° Yengeç, Rahu 70° İkizler, Ketu 250° Yay. Lagna 310° Kova.",
    "Ay: uzanım = 170 − 200 = −30 ≡ 330°; küçülüyor, kötücül.",
    "Merkür Başak'ta yalnızca küçülen Ay ile: Ay kötücül sayılır, iyicil yok; kötücül.",
    "Kuja dosha: Mars Kova'da. Kova'dan Kova 1. ev: var. Başak'tan sayım Başak 1, Terazi 2, Akrep 3, Yay 4, Oğlak 5, Kova 6: yok. Akrep'ten sayım Akrep 1, Yay 2, Oğlak 3, Kova 4: var.",
    "Ev lordları, Kova lagna (evler Kova, Balık, Koç, Boğa, İkizler, Yengeç, Aslan, Başak, Terazi, Akrep, Yay, Oğlak): Satürn, Jüpiter, Mars, Venüs, Merkür, Ay, Güneş, Merkür, Venüs, Mars, Jüpiter, Satürn.",
    "Evler, Kova'dan: Terazi 9, Başak 8, Kova 1, Başak 8, Boğa 4, Akrep 10, Yengeç 6, İkizler 5, Yay 11."
  ],
  "input": {
    "lagna_lon": 310.0,
    "grahas": {"Sun": 200.0, "Moon": 170.0, "Mars": 325.0, "Mercury": 175.0, "Jupiter": 40.0, "Venus": 230.0, "Saturn": 100.0, "Rahu": 70.0, "Ketu": 250.0}
  },
  "expected": {
    "waxing": false,
    "nature": {"Sun": "Malefic", "Moon": "Malefic", "Mars": "Malefic", "Mercury": "Malefic", "Jupiter": "Benefic", "Venus": "Benefic", "Saturn": "Malefic", "Rahu": "Malefic", "Ketu": "Malefic"},
    "kuja_dosha": {"from_lagna": true, "from_moon": false, "from_venus": true},
    "house_lords": ["Saturn", "Jupiter", "Mars", "Venus", "Mercury", "Moon", "Sun", "Mercury", "Venus", "Mars", "Jupiter", "Saturn"],
    "houses": {"Sun": 9, "Moon": 8, "Mars": 1, "Mercury": 8, "Jupiter": 4, "Venus": 10, "Saturn": 6, "Rahu": 5, "Ketu": 11},
    "houses_owned": {"Sun": [7], "Moon": [6], "Mars": [3, 10], "Mercury": [5, 8], "Jupiter": [2, 11], "Venus": [4, 9], "Saturn": [1, 12], "Rahu": [], "Ketu": []}
  }
}
```

**Yeni dosya:** `tests/golden/modules/foundation/time-unknown-mercury-with-waxing-moon-and-rahu.json`

```json
{
  "id": "time-unknown-mercury-with-waxing-moon-and-rahu",
  "source": "hand",
  "rule_ref": "BPHS, graha nitelikleri bölümü (doğal sınıf); evler lagna burcundan bütün burç sayımı; Kuja dosha: Mars 1, 2, 4, 7, 8, 12",
  "derivation": [
    "Burçlar: Güneş 10° Koç, Ay 70° İkizler, Mars 345° Balık, Merkür 80° İkizler, Jüpiter 200° Terazi, Venüs 125° Aslan, Satürn 130° Aslan, Rahu 75° İkizler, Ketu 255° Yay. Doğum saati bilinmiyor: lagna yok.",
    "Ay: uzanım = 70 − 10 = 60°; büyüyor, iyicil.",
    "Merkür İkizler'de büyüyen Ay (iyicil) ve Rahu (kötücül) ile: iyicil bulunduğundan iyicil.",
    "Kuja dosha: Mars Balık'ta. Lagna yok: lagnadan değer boş. İkizler'den sayım İkizler 1, Yengeç 2, Aslan 3, Başak 4, Terazi 5, Akrep 6, Yay 7, Oğlak 8, Kova 9, Balık 10: yok. Aslan'dan sayım Aslan 1, Başak 2, Terazi 3, Akrep 4, Yay 5, Oğlak 6, Kova 7, Balık 8: var.",
    "Lagna olmadığından ev lordları, evler ve lordluk listeleri boştur."
  ],
  "input": {
    "lagna_lon": null,
    "grahas": {"Sun": 10.0, "Moon": 70.0, "Mars": 345.0, "Mercury": 80.0, "Jupiter": 200.0, "Venus": 125.0, "Saturn": 130.0, "Rahu": 75.0, "Ketu": 255.0}
  },
  "expected": {
    "waxing": true,
    "nature": {"Sun": "Malefic", "Moon": "Benefic", "Mars": "Malefic", "Mercury": "Benefic", "Jupiter": "Benefic", "Venus": "Benefic", "Saturn": "Malefic", "Rahu": "Malefic", "Ketu": "Malefic"},
    "kuja_dosha": {"from_lagna": null, "from_moon": false, "from_venus": true},
    "house_lords": null,
    "houses": null,
    "houses_owned": null
  }
}
```

**Yeni dosya:** `tests/golden/modules/foundation/mercury-with-rahu-only-mars-in-second.json`

```json
{
  "id": "mercury-with-rahu-only-mars-in-second",
  "source": "hand",
  "rule_ref": "BPHS, graha nitelikleri bölümü (doğal sınıf); evler lagna burcundan bütün burç sayımı; Kuja dosha: Mars 1, 2, 4, 7, 8, 12",
  "derivation": [
    "Burçlar: Güneş 50° Boğa, Ay 140° Aslan, Mars 215° Akrep, Merkür 20° Koç, Jüpiter 280° Oğlak, Venüs 95° Yengeç, Satürn 340° Balık, Rahu 25° Koç, Ketu 205° Terazi. Lagna 190° Terazi.",
    "Ay: uzanım = 140 − 50 = 90°; büyüyor, iyicil.",
    "Merkür Koç'ta yalnızca Rahu ile: kötücül var, iyicil yok; kötücül.",
    "Kuja dosha: Mars Akrep'te. Terazi'den Akrep 2. ev: var. Aslan'dan sayım Aslan 1, Başak 2, Terazi 3, Akrep 4: var. Yengeç'ten sayım Yengeç 1, Aslan 2, Başak 3, Terazi 4, Akrep 5: yok.",
    "Ev lordları, Terazi lagna (evler Terazi, Akrep, Yay, Oğlak, Kova, Balık, Koç, Boğa, İkizler, Yengeç, Aslan, Başak): Venüs, Mars, Jüpiter, Satürn, Satürn, Jüpiter, Mars, Venüs, Merkür, Ay, Güneş, Merkür.",
    "Evler, Terazi'den: Boğa 8, Aslan 11, Akrep 2, Koç 7, Oğlak 4, Yengeç 10, Balık 6, Koç 7, Terazi 1."
  ],
  "input": {
    "lagna_lon": 190.0,
    "grahas": {"Sun": 50.0, "Moon": 140.0, "Mars": 215.0, "Mercury": 20.0, "Jupiter": 280.0, "Venus": 95.0, "Saturn": 340.0, "Rahu": 25.0, "Ketu": 205.0}
  },
  "expected": {
    "waxing": true,
    "nature": {"Sun": "Malefic", "Moon": "Benefic", "Mars": "Malefic", "Mercury": "Malefic", "Jupiter": "Benefic", "Venus": "Benefic", "Saturn": "Malefic", "Rahu": "Malefic", "Ketu": "Malefic"},
    "kuja_dosha": {"from_lagna": true, "from_moon": true, "from_venus": false},
    "house_lords": ["Venus", "Mars", "Jupiter", "Saturn", "Saturn", "Jupiter", "Mars", "Venus", "Mercury", "Moon", "Sun", "Mercury"],
    "houses": {"Sun": 8, "Moon": 11, "Mars": 2, "Mercury": 7, "Jupiter": 4, "Venus": 10, "Saturn": 6, "Rahu": 7, "Ketu": 1},
    "houses_owned": {"Sun": [11], "Moon": [10], "Mars": [2, 7], "Mercury": [9, 12], "Jupiter": [3, 6], "Venus": [1, 8], "Saturn": [4, 5], "Rahu": [], "Ketu": []}
  }
}
```

- [ ] **Step 2: Düzeneği ve değişmezleri yaz (başarısız testler)**

**Yeni dosya:** `crates/jyotish-core/tests/modules_golden.rs`

```rust
//! İleri modüllerin el örnekleri (ileri modüller spec'i §13): `tests/golden/modules/<modül>/*.json`.
//! Girdi sentetik konumlardır (ephemeris gerekmez); beklenen değer klasik kuraldan elle
//! türetilmiştir ve `derivation` alanında adım adım yazılıdır. Bir örnek motorla uyuşmazsa
//! önce kural yeniden okunur; beklenen değer motor çıktısına bakarak düzeltilmez.

use std::collections::BTreeMap;
use std::path::PathBuf;

use jyotish_core::{
    house_lord, houses_owned, kuja_dosha, moon_is_waxing, nature, Graha, KujaDosha, Nature,
    Positions,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// Modül başına en az örnek sayısı; altına düşerse test düşer (sessiz silinmeye karşı).
const MIN_FOUNDATION: usize = 6;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture<X> {
    id: String,
    source: String,
    rule_ref: String,
    derivation: Vec<String>,
    input: Input,
    expected: X,
}

/// Sentetik konumlar: sidereal boylamlar, derece. `lagna_lon: null` doğum saati bilinmeyen
/// haritadır.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    lagna_lon: Option<f64>,
    grahas: BTreeMap<Graha, f64>,
}

impl Input {
    fn positions(&self, id: &str) -> Positions {
        assert_eq!(self.grahas.len(), 9, "{id}: all nine grahas are required");
        Positions::synthetic(
            self.lagna_lon,
            std::array::from_fn(|i| self.grahas[&Graha::ALL[i]]),
        )
    }
}

fn load<X: DeserializeOwned>(module: &str, min: usize) -> Vec<(Fixture<X>, Positions)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/golden/modules")
        .join(module);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    assert!(
        paths.len() >= min,
        "{module}: {} hand fixtures, at least {min} required",
        paths.len()
    );
    paths
        .into_iter()
        .map(|p| {
            let text =
                std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            let f: Fixture<X> =
                serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            assert_eq!(
                p.file_stem().and_then(|s| s.to_str()),
                Some(f.id.as_str()),
                "id must equal the file name"
            );
            // Başka yazılımın çıktısı fixture olamaz (K9).
            assert_eq!(f.source, "hand", "{}: only hand-derived fixtures", f.id);
            assert!(!f.rule_ref.trim().is_empty(), "{}: rule_ref", f.id);
            assert!(
                !f.derivation.is_empty() && f.derivation.iter().all(|s| !s.trim().is_empty()),
                "{}: derivation steps",
                f.id
            );
            let positions = f.input.positions(&f.id);
            (f, positions)
        })
        .collect()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FoundationExpected {
    waxing: bool,
    nature: BTreeMap<Graha, Nature>,
    kuja_dosha: KujaDosha,
    /// 1.–12. evin lordu; doğum saati bilinmiyorsa `null`.
    house_lords: Option<[Graha; 12]>,
    /// Her grahanın lagnadan evi; doğum saati bilinmiyorsa `null`.
    houses: Option<BTreeMap<Graha, u8>>,
    /// Her grahanın lordu olduğu evler; doğum saati bilinmiyorsa `null`.
    houses_owned: Option<BTreeMap<Graha, Vec<u8>>>,
}

#[test]
fn foundation_hand_fixtures() {
    for (f, p) in load::<FoundationExpected>("foundation", MIN_FOUNDATION) {
        let id = &f.id;
        let x = &f.expected;
        assert_eq!(moon_is_waxing(&p), x.waxing, "{id}: waxing");
        assert_eq!(x.nature.len(), 9, "{id}: nature lists nine grahas");
        for g in Graha::ALL {
            assert_eq!(nature(g, &p), x.nature[&g], "{id}: nature of {g:?}");
        }
        assert_eq!(kuja_dosha(&p), x.kuja_dosha, "{id}: kuja dosha");
        match p.lagna_sign {
            Some(lagna) => {
                let lords: [Graha; 12] = std::array::from_fn(|i| house_lord(lagna, i as u8 + 1));
                assert_eq!(Some(lords), x.house_lords, "{id}: house lords");
                let houses = x.houses.as_ref().expect("houses with a lagna");
                let owned = x.houses_owned.as_ref().expect("houses_owned with a lagna");
                assert_eq!((houses.len(), owned.len()), (9, 9), "{id}: nine grahas");
                for g in Graha::ALL {
                    assert_eq!(p.house_of(g), Some(houses[&g]), "{id}: house of {g:?}");
                    assert_eq!(houses_owned(lagna, g), owned[&g], "{id}: owned by {g:?}");
                }
            }
            None => {
                assert!(
                    x.house_lords.is_none() && x.houses.is_none() && x.houses_owned.is_none(),
                    "{id}: lagna-dependent values must be null without a lagna"
                );
                assert_eq!(x.kuja_dosha.from_lagna, None, "{id}");
                for g in Graha::ALL {
                    assert_eq!(p.house_of(g), None, "{id}");
                }
            }
        }
    }
}

#[test]
fn unknown_fields_are_rejected() {
    let base = r#"{"id":"x","source":"hand","rule_ref":"r","derivation":["d"],
        "input":{"lagna_lon":null,"grahas":{}},
        "expected":{"waxing":true,"nature":{},
            "kuja_dosha":{"from_lagna":null,"from_moon":false,"from_venus":false},
            "house_lords":null,"houses":null,"houses_owned":null}}"#;
    assert!(serde_json::from_str::<Fixture<FoundationExpected>>(base).is_ok());
    let extra = base.replace(r#""id":"x","#, r#""id":"x","note":"n","#);
    assert!(serde_json::from_str::<Fixture<FoundationExpected>>(&extra).is_err());
    let extra_input = base.replace(r#""grahas":{}"#, r#""grahas":{},"lagna":1"#);
    assert!(serde_json::from_str::<Fixture<FoundationExpected>>(&extra_input).is_err());
    let extra_expected = base.replace(r#""waxing":true,"#, r#""waxing":true,"more":1,"#);
    assert!(serde_json::from_str::<Fixture<FoundationExpected>>(&extra_expected).is_err());
}
```

`proptests.rs`'in sonuna:

```diff
diff --git a/crates/jyotish-core/src/proptests.rs b/crates/jyotish-core/src/proptests.rs
index ad27adb..c281da6 100644
--- a/crates/jyotish-core/src/proptests.rs
+++ b/crates/jyotish-core/src/proptests.rs
@@ -351,3 +351,74 @@ mod ashtakavarga_properties {
         }
     }
 }
+
+/// İleri modüllerin ortak temeli (Alt proje 3.1): doğal sınıf, lordluk, Kuja dosha.
+mod foundation_properties {
+    use proptest::prelude::*;
+
+    use crate::graha::Graha;
+    use crate::lordship::{house_lord, house_sign, houses_owned};
+    use crate::nature::{kuja_dosha, moon_is_waxing, nature, Nature, KUJA_DOSHA_HOUSES};
+    use crate::positions::Positions;
+    use crate::zodiac::Sign;
+
+    fn positions() -> impl Strategy<Value = Positions> {
+        (
+            proptest::option::of(0.0f64..360.0),
+            proptest::array::uniform9(0.0f64..360.0),
+        )
+            .prop_map(|(lagna, lons)| Positions::synthetic(lagna, lons))
+    }
+
+    proptest! {
+        #[test]
+        fn natural_class_rules_hold(p in positions()) {
+            for g in [Graha::Jupiter, Graha::Venus] {
+                prop_assert_eq!(nature(g, &p), Nature::Benefic);
+            }
+            for g in [Graha::Sun, Graha::Mars, Graha::Saturn, Graha::Rahu, Graha::Ketu] {
+                prop_assert_eq!(nature(g, &p), Nature::Malefic);
+            }
+            let moon = if moon_is_waxing(&p) { Nature::Benefic } else { Nature::Malefic };
+            prop_assert_eq!(nature(Graha::Moon, &p), moon);
+            // Merkür: aynı burçta kötücül varken iyicil yoksa kötücül, değilse iyicil.
+            let sign = p.sign_of(Graha::Mercury);
+            let others: Vec<Nature> = p
+                .occupants(sign)
+                .into_iter()
+                .filter(|&g| g != Graha::Mercury)
+                .map(|g| nature(g, &p))
+                .collect();
+            let want = if others.contains(&Nature::Malefic) && !others.contains(&Nature::Benefic) {
+                Nature::Malefic
+            } else {
+                Nature::Benefic
+            };
+            prop_assert_eq!(nature(Graha::Mercury, &p), want);
+        }
+
+        #[test]
+        fn houses_and_kuja_dosha_follow_the_sign_count(p in positions()) {
+            let k = kuja_dosha(&p);
+            prop_assert_eq!(k.from_lagna.is_some(), p.lagna_sign.is_some());
+            let mars_from = |s: Sign| KUJA_DOSHA_HOUSES.contains(&p.house_from(Graha::Mars, s));
+            prop_assert_eq!(k.from_moon, mars_from(p.sign_of(Graha::Moon)));
+            prop_assert_eq!(k.from_venus, mars_from(p.sign_of(Graha::Venus)));
+            if let Some(lagna) = p.lagna_sign {
+                prop_assert_eq!(k.from_lagna, Some(mars_from(lagna)));
+                for g in Graha::ALL {
+                    let h = p.house_of(g).unwrap();
+                    prop_assert!((1..=12).contains(&h));
+                    prop_assert_eq!(house_sign(lagna, h), p.sign_of(g));
+                }
+                for h in 1..=12u8 {
+                    prop_assert!(houses_owned(lagna, house_lord(lagna, h)).contains(&h));
+                }
+            } else {
+                for g in Graha::ALL {
+                    prop_assert_eq!(p.house_of(g), None);
+                }
+            }
+        }
+    }
+}
```

- [ ] **Step 3: Testlerin başarısız olduğunu doğrula**

Run: `cargo test -p jyotish-core 2>&1 | grep -E "^error" | head -3`
Expected: derleme hatası (`unresolved import crate::lordship`, `crate::nature`, `crate::positions`).

- [ ] **Step 4: `positions.rs`**

**Yeni dosya:** `crates/jyotish-core/src/positions.rs`

```rust
//! İleri modüllerin ortak girdisi (ileri modüller spec'i K2, K3): `ChartInput` komut
//! girdisidir, `Positions` ise her modülün saf hesap fonksiyonunun gördüğü, ephemeris
//! içermeyen düz konum kümesidir. El örnekleri `Positions::synthetic` ile kurulur.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{compute_chart, Chart};
use crate::graha::Graha;
use crate::settings::CalcSettings;
use crate::zodiac::{deg_in_sign, normalize, Sign};
use crate::CoreError;

/// Modül komutlarının ortak girdisi: `compute_chart` ile aynı alanlar ve `time_known`.
/// Doğum saati bilinmiyorsa (`time_known == false`) lagnaya bağlı her çıktı `None` döner;
/// arayüzün gizlemesine güvenilmez (K2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ChartInput {
    pub jd_ut: f64,
    /// Kuzey +.
    pub lat: f64,
    /// Doğu +.
    pub lon: f64,
    pub settings: CalcSettings,
    pub time_known: bool,
}

/// Bir grahanın modüllerin kullandığı konumu.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GrahaPoint {
    pub graha: Graha,
    /// Her zodyak kipinde sidereal boylam, [0, 360). Nakshatra ve derece temelli kurallar
    /// (sphuta drishti) bunu kullanır.
    pub sidereal_lon: f64,
    /// Rasi çerçevesindeki burç (`settings.zodiac`'ı izler, K5). Burca dayalı her kural bunu
    /// kullanır; tropikal-rasi kipinde `sidereal_lon`'un burcundan farklı olabilir.
    pub rasi_sign: Sign,
    /// Rasi çerçevesinde burç içi derece, [0, 30).
    pub deg_in_sign: f64,
    /// Ekliptik enlem, derece.
    pub lat: f64,
    /// Boylam hızı, derece/gün.
    pub speed: f64,
}

/// Ephemeris görmeyen konum kümesi. `grahas` her zaman `Graha::ALL` sırasındadır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Positions {
    /// Lagnanın sidereal boylamı; doğum saati bilinmiyorsa `None`.
    pub lagna_lon: Option<f64>,
    /// Lagnanın rasi çerçevesindeki burcu; evler bundan sayılır (K4). `lagna_lon` ile
    /// birlikte dolu ya da birlikte boştur.
    pub lagna_sign: Option<Sign>,
    pub grahas: [GrahaPoint; 9],
}

impl From<&Chart> for Positions {
    /// Haritanın konumları, lagnasıyla. Doğum saati bilinmiyorsa çağıran
    /// `without_lagna` uygular (`compute_positions` bunu kendisi yapar).
    fn from(chart: &Chart) -> Self {
        Positions {
            lagna_lon: Some(chart.lagna.sidereal_lon),
            lagna_sign: Some(chart.lagna.sign),
            // `Chart::grahas` her zaman 9 elemandır ve `Graha::ALL` sırasındadır.
            grahas: std::array::from_fn(|i| {
                let g = &chart.grahas[i];
                GrahaPoint {
                    graha: g.graha,
                    sidereal_lon: g.sidereal_lon,
                    rasi_sign: g.sign,
                    deg_in_sign: g.deg_in_sign,
                    lat: g.lat,
                    speed: g.speed,
                }
            }),
        }
    }
}

impl Positions {
    /// Sentetik konumlar (el örnekleri ve testler): `lons`, `Graha::ALL` sırasında sidereal
    /// boylamlardır ve rasi çerçevesi sidereal sayılır. Enlem ve hız sıfırdır.
    pub fn synthetic(lagna_lon: Option<f64>, lons: [f64; 9]) -> Positions {
        let lagna_lon = lagna_lon.map(normalize);
        Positions {
            lagna_lon,
            lagna_sign: lagna_lon.map(Sign::of),
            grahas: std::array::from_fn(|i| {
                let lon = normalize(lons[i]);
                GrahaPoint {
                    graha: Graha::ALL[i],
                    sidereal_lon: lon,
                    rasi_sign: Sign::of(lon),
                    deg_in_sign: deg_in_sign(lon),
                    lat: 0.0,
                    speed: 0.0,
                }
            }),
        }
    }

    /// Lagnasız kopya: doğum saati bilinmeyen harita.
    pub fn without_lagna(mut self) -> Positions {
        self.lagna_lon = None;
        self.lagna_sign = None;
        self
    }

    pub fn point(&self, graha: Graha) -> &GrahaPoint {
        &self.grahas[graha as usize]
    }

    /// Grahanın rasi çerçevesindeki burcu.
    pub fn sign_of(&self, graha: Graha) -> Sign {
        self.point(graha).rasi_sign
    }

    /// `sign` burcundaki grahalar, `Graha::ALL` sırasında.
    pub fn occupants(&self, sign: Sign) -> Vec<Graha> {
        self.grahas
            .iter()
            .filter(|p| p.rasi_sign == sign)
            .map(|p| p.graha)
            .collect()
    }

    /// Grahanın `reference` burcundan sayılan rasi evi, 1..=12 (K4).
    pub fn house_from(&self, graha: Graha, reference: Sign) -> u8 {
        self.sign_of(graha).house_from(reference)
    }

    /// Grahanın lagnadan sayılan rasi evi; doğum saati bilinmiyorsa `None`.
    pub fn house_of(&self, graha: Graha) -> Option<u8> {
        self.lagna_sign.map(|l| self.house_from(graha, l))
    }
}

/// Toplayıcı: temel haritayı hesaplar ve konumlara indirger. `input.time_known` yanlışsa
/// lagna boştur. Ayar, an ve konum denetimi `compute_chart`'ınkidir.
pub fn compute_positions<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<Positions, CoreError> {
    let chart = compute_chart(eph, input.jd_ut, input.lat, input.lon, &input.settings)?;
    let positions = Positions::from(&chart);
    Ok(if input.time_known {
        positions
    } else {
        positions.without_lagna()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::ZodiacMode;

    const JD: f64 = 2_451_545.0;

    fn input(settings: CalcSettings, time_known: bool) -> ChartInput {
        ChartInput {
            jd_ut: JD,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known,
        }
    }

    #[test]
    fn synthetic_positions_are_in_graha_order_with_sidereal_signs() {
        let p = Positions::synthetic(
            Some(370.0),
            [10.0, 95.0, 200.0, 20.0, 250.0, 40.0, 300.0, 120.0, -60.0],
        );
        assert_eq!(p.lagna_lon, Some(10.0));
        assert_eq!(p.lagna_sign, Some(Sign::Aries));
        for (i, g) in Graha::ALL.into_iter().enumerate() {
            assert_eq!(p.grahas[i].graha, g);
            assert_eq!(p.point(g).graha, g);
        }
        assert_eq!(p.sign_of(Graha::Moon), Sign::Cancer);
        assert_eq!(p.point(Graha::Moon).deg_in_sign, 5.0);
        assert_eq!(p.point(Graha::Ketu).sidereal_lon, 300.0);
        assert_eq!(p.occupants(Sign::Aries), [Graha::Sun, Graha::Mercury]);
        assert_eq!(p.occupants(Sign::Aquarius), [Graha::Saturn, Graha::Ketu]);
        assert!(p.occupants(Sign::Gemini).is_empty());
        assert_eq!(p.house_of(Graha::Moon), Some(4));
        assert_eq!(p.house_from(Graha::Sun, Sign::Cancer), 10);
        let q = p.clone().without_lagna();
        assert_eq!((q.lagna_lon, q.lagna_sign), (None, None));
        assert_eq!(q.house_of(Graha::Moon), None);
        assert_eq!(q.grahas, p.grahas);
    }

    #[test]
    fn positions_follow_the_chart_and_hide_the_lagna_when_time_is_unknown() {
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        let chart = compute_chart(&mut eph, JD, 41.0, 29.0, &s).unwrap();
        let p = compute_positions(&mut eph, &input(s.clone(), true)).unwrap();
        assert_eq!(p, Positions::from(&chart));
        assert_eq!(p.lagna_lon, Some(chart.lagna.sidereal_lon));
        assert_eq!(p.lagna_sign, Some(chart.lagna.sign));
        for (point, g) in p.grahas.iter().zip(&chart.grahas) {
            assert_eq!(
                (point.graha, point.sidereal_lon, point.rasi_sign),
                (g.graha, g.sidereal_lon, g.sign)
            );
            assert_eq!(
                (point.deg_in_sign, point.lat, point.speed),
                (g.deg_in_sign, g.lat, g.speed)
            );
        }
        let unknown = compute_positions(&mut eph, &input(s, false)).unwrap();
        assert_eq!((unknown.lagna_lon, unknown.lagna_sign), (None, None));
        assert_eq!(unknown.grahas, p.grahas);
    }

    #[test]
    fn tropical_rasi_mode_keeps_sidereal_longitudes_and_tropical_signs() {
        // FakeEph: Güneş tropikal 10° (Koç), ayanamsa 24° → sidereal 346° (Balık).
        let mut eph = FakeEph::default();
        let s = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        let p = compute_positions(&mut eph, &input(s, true)).unwrap();
        let sun = p.point(Graha::Sun);
        assert_eq!(sun.rasi_sign, Sign::Aries);
        assert!((sun.deg_in_sign - 10.0).abs() < 1e-9);
        assert!((sun.sidereal_lon - 346.0).abs() < 1e-9);
        // Lagna tropikal 70° (İkizler); sidereal boylamı 46°.
        assert_eq!(p.lagna_sign, Some(Sign::Gemini));
        assert!((p.lagna_lon.unwrap() - 46.0).abs() < 1e-9);
    }

    #[test]
    fn collector_reports_the_chart_errors() {
        let mut eph = FakeEph::default();
        let mut bad = input(CalcSettings::default(), true);
        bad.lat = 91.0;
        assert_eq!(
            compute_positions(&mut eph, &bad),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0
            })
        );
        let old = CalcSettings {
            schema_version: 1,
            ..CalcSettings::default()
        };
        assert_eq!(
            compute_positions(&mut eph, &input(old, true)),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 1,
                supported: 2
            })
        );
    }
}
```

- [ ] **Step 5: `lordship.rs`**

**Yeni dosya:** `crates/jyotish-core/src/lordship.rs`

```rust
//! Ev lordluğu yardımcıları (ileri modüller spec'i §4). Evler rasi evidir: lagna burcundan
//! sayılan bütün burç evleri (K4); `CalcSettings.house_system` bunları etkilemez. Burç lordu
//! `dignity::sign_lord`'dur (Akrep: Mars, Kova: Satürn); Jaimini eş lordları burada yoktur.

use crate::dignity::sign_lord;
use crate::graha::Graha;
use crate::zodiac::Sign;

/// Lagna burcundan sayılan `house`. evin burcu. `house` 1..=12'dir; aralık dışı değer 12'ye
/// göre sarılır (13 → 1, 0 → 12).
pub fn house_sign(lagna_sign: Sign, house: u8) -> Sign {
    Sign::from_index(lagna_sign.index() + (usize::from(house) + 11) % 12)
}

/// `house`. evin lordu.
pub fn house_lord(lagna_sign: Sign, house: u8) -> Graha {
    sign_lord(house_sign(lagna_sign, house))
}

/// Grahanın lordu olduğu evler, artan sırada. Güneş ve Ay birer, diğer beş graha ikişer
/// evin lordudur; Rahu ve Ketu için boştur.
pub fn houses_owned(lagna_sign: Sign, graha: Graha) -> Vec<u8> {
    (1..=12)
        .filter(|&h| house_lord(lagna_sign, h) == graha)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;

    #[test]
    fn house_signs_count_inclusively_from_the_lagna_sign() {
        assert_eq!(house_sign(Sign::Aries, 1), Sign::Aries);
        assert_eq!(house_sign(Sign::Aries, 7), Sign::Libra);
        assert_eq!(house_sign(Sign::Scorpio, 6), Sign::Aries);
        assert_eq!(house_sign(Sign::Pisces, 2), Sign::Aries);
        assert_eq!(house_sign(Sign::Gemini, 12), Sign::Taurus);
        // Aralık dışı değerler sarılır.
        assert_eq!(house_sign(Sign::Gemini, 13), Sign::Gemini);
        assert_eq!(house_sign(Sign::Gemini, 0), Sign::Taurus);
        for lagna in Sign::ALL {
            for h in 1..=12u8 {
                assert_eq!(house_sign(lagna, h).house_from(lagna), h);
            }
        }
    }

    #[test]
    fn house_lords_for_three_lagnas_match_the_hand_table() {
        // Koç lagna: evler Koç…Balık.
        let aries: Vec<Graha> = (1..=12).map(|h| house_lord(Sign::Aries, h)).collect();
        assert_eq!(
            aries,
            [
                Mars, Venus, Mercury, Moon, Sun, Mercury, Venus, Mars, Jupiter, Saturn, Saturn,
                Jupiter
            ]
        );
        // Yengeç lagna: evler Yengeç, Aslan, Başak, Terazi, Akrep, Yay, Oğlak, Kova, Balık,
        // Koç, Boğa, İkizler.
        let cancer: Vec<Graha> = (1..=12).map(|h| house_lord(Sign::Cancer, h)).collect();
        assert_eq!(
            cancer,
            [
                Moon, Sun, Mercury, Venus, Mars, Jupiter, Saturn, Saturn, Jupiter, Mars, Venus,
                Mercury
            ]
        );
        // Kova lagna: evler Kova, Balık, Koç, Boğa, İkizler, Yengeç, Aslan, Başak, Terazi,
        // Akrep, Yay, Oğlak.
        let aquarius: Vec<Graha> = (1..=12).map(|h| house_lord(Sign::Aquarius, h)).collect();
        assert_eq!(
            aquarius,
            [
                Saturn, Jupiter, Mars, Venus, Mercury, Moon, Sun, Mercury, Venus, Mars, Jupiter,
                Saturn
            ]
        );
    }

    #[test]
    fn houses_owned_lists_each_house_once() {
        assert_eq!(houses_owned(Sign::Aries, Mars), [1, 8]);
        assert_eq!(houses_owned(Sign::Aries, Sun), [5]);
        assert_eq!(houses_owned(Sign::Cancer, Saturn), [7, 8]);
        assert_eq!(houses_owned(Sign::Aquarius, Saturn), [1, 12]);
        assert_eq!(houses_owned(Sign::Libra, Mars), [2, 7]);
        assert!(houses_owned(Sign::Aries, Rahu).is_empty());
        assert!(houses_owned(Sign::Aries, Ketu).is_empty());
        for lagna in Sign::ALL {
            let mut all: Vec<u8> = Graha::ALL
                .into_iter()
                .flat_map(|g| houses_owned(lagna, g))
                .collect();
            all.sort_unstable();
            assert_eq!(all, (1..=12).collect::<Vec<u8>>(), "{lagna:?}");
            assert_eq!(houses_owned(lagna, Sun).len(), 1);
            assert_eq!(houses_owned(lagna, Moon).len(), 1);
            for g in [Mars, Mercury, Jupiter, Venus, Saturn] {
                assert_eq!(houses_owned(lagna, g).len(), 2, "{lagna:?} {g:?}");
            }
        }
    }
}
```

- [ ] **Step 6: `nature.rs`**

**Yeni dosya:** `crates/jyotish-core/src/nature.rs`

```rust
//! Doğal iyicil/kötücül sınıfı ve Kuja dosha olgusu (ileri modüller spec'i K6, §4). Modüller
//! kendi iyicil/kötücül listesini tutmaz; hepsi `nature` fonksiyonunu çağırır.
//!
//! Kaynak: BPHS, graha nitelikleri bölümü. Jüpiter ve Venüs iyicil; Güneş, Mars, Satürn, Rahu
//! ve Ketu kötücül; Ay büyürken iyicil, küçülürken kötücül; Merkür aynı burçta kötücül varken
//! iyicil yoksa kötücül, değilse iyicil.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::positions::Positions;
use crate::zodiac::normalize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Nature {
    Benefic,
    Malefic,
}

/// Ay büyüyor mu: Güneş'ten uzanım [0°, 180°) (Shukla paksha; `panchang::paksha` ile aynı
/// sınır). Uzanım iki boylamın farkıdır, ayanamsadan bağımsızdır.
pub fn moon_is_waxing(positions: &Positions) -> bool {
    let sun = positions.point(Graha::Sun).sidereal_lon;
    let moon = positions.point(Graha::Moon).sidereal_lon;
    normalize(moon - sun) < 180.0
}

/// Merkür dışındaki grahaların sınıfı (Merkür'ün sınıfı bunlara bağlıdır).
fn fixed_nature(graha: Graha, positions: &Positions) -> Nature {
    match graha {
        Graha::Jupiter | Graha::Venus => Nature::Benefic,
        Graha::Sun | Graha::Mars | Graha::Saturn | Graha::Rahu | Graha::Ketu => Nature::Malefic,
        Graha::Moon => {
            if moon_is_waxing(positions) {
                Nature::Benefic
            } else {
                Nature::Malefic
            }
        }
        // Çağrılmaz; `nature` Merkür'ü kendisi çözer.
        Graha::Mercury => Nature::Benefic,
    }
}

/// Grahanın doğal sınıfı (K6). "Aynı burç" rasi çerçevesindeki burçtur (K5).
pub fn nature(graha: Graha, positions: &Positions) -> Nature {
    if graha != Graha::Mercury {
        return fixed_nature(graha, positions);
    }
    let sign = positions.sign_of(Graha::Mercury);
    let (mut benefic, mut malefic) = (false, false);
    for other in Graha::ALL {
        if other == Graha::Mercury || positions.sign_of(other) != sign {
            continue;
        }
        match fixed_nature(other, positions) {
            Nature::Benefic => benefic = true,
            Nature::Malefic => malefic = true,
        }
    }
    if malefic && !benefic {
        Nature::Malefic
    } else {
        Nature::Benefic
    }
}

/// Kuja dosha olgusu: Mars ilgili referanstan sayılan 1, 2, 4, 7, 8 ya da 12. rasi evinde mi.
/// İptal kuralı uygulanmaz. Yoga (3.5) ve uyum (3.6) modülleri aynı olguyu kullanır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct KujaDosha {
    /// Doğum saati bilinmiyorsa `None`.
    pub from_lagna: Option<bool>,
    pub from_moon: bool,
    pub from_venus: bool,
}

/// Mars'ın dosha sayılan evleri.
pub const KUJA_DOSHA_HOUSES: [u8; 6] = [1, 2, 4, 7, 8, 12];

pub fn kuja_dosha(positions: &Positions) -> KujaDosha {
    let from =
        |reference| KUJA_DOSHA_HOUSES.contains(&positions.house_from(Graha::Mars, reference));
    KujaDosha {
        from_lagna: positions.lagna_sign.map(from),
        from_moon: from(positions.sign_of(Graha::Moon)),
        from_venus: from(positions.sign_of(Graha::Venus)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    /// Taban: hiçbir graha Merkür'le (İkizler 65°) aynı burçta değil; Ay büyüyor.
    const BASE: [f64; 9] = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];

    fn with(changes: &[(Graha, f64)]) -> Positions {
        let mut lons = BASE;
        for &(g, lon) in changes {
            lons[g as usize] = lon;
        }
        Positions::synthetic(Some(0.0), lons)
    }

    #[test]
    fn fixed_classes() {
        let p = with(&[]);
        for g in [Jupiter, Venus] {
            assert_eq!(nature(g, &p), Nature::Benefic, "{g:?}");
        }
        for g in [Sun, Mars, Saturn, Rahu, Ketu] {
            assert_eq!(nature(g, &p), Nature::Malefic, "{g:?}");
        }
    }

    #[test]
    fn moon_is_benefic_while_waxing() {
        // Güneş 10°: uzanım = Ay − 10°.
        for (moon, waxing) in [
            (10.0, true),    // 0°: yeni ay, Shukla 1'in başı
            (100.0, true),   // 90°
            (189.999, true), // 179,999°
            (190.0, false),  // 180°: dolunay, Krishna 1'in başı
            (280.0, false),  // 270°
            (9.999, false),  // 359,999°
        ] {
            let p = with(&[(Moon, moon)]);
            assert_eq!(moon_is_waxing(&p), waxing, "moon {moon}");
            let want = if waxing {
                Nature::Benefic
            } else {
                Nature::Malefic
            };
            assert_eq!(nature(Moon, &p), want, "moon {moon}");
        }
    }

    #[test]
    fn mercury_follows_its_sign_companions() {
        // Yalnız: iyicil.
        assert_eq!(nature(Mercury, &with(&[])), Nature::Benefic);
        // Yalnızca kötücülle (her biri ayrı ayrı): kötücül.
        for g in [Sun, Mars, Saturn, Rahu, Ketu] {
            assert_eq!(
                nature(Mercury, &with(&[(g, 70.0)])),
                Nature::Malefic,
                "{g:?}"
            );
        }
        // Yalnızca iyicille: iyicil.
        for g in [Jupiter, Venus] {
            assert_eq!(
                nature(Mercury, &with(&[(g, 70.0)])),
                Nature::Benefic,
                "{g:?}"
            );
        }
        // Kötücül ve iyicil birlikte: iyicil.
        assert_eq!(
            nature(Mercury, &with(&[(Saturn, 61.0), (Jupiter, 89.0)])),
            Nature::Benefic
        );
        // Büyüyen Ay iyicil sayılır (Güneş 10°, Ay 70°: uzanım 60°).
        assert_eq!(nature(Mercury, &with(&[(Moon, 70.0)])), Nature::Benefic);
        // Küçülen Ay kötücül sayılır (Güneş 100°, Ay 70°: uzanım 330°).
        assert_eq!(
            nature(Mercury, &with(&[(Sun, 100.0), (Moon, 70.0)])),
            Nature::Malefic
        );
        // Komşu burçtaki kötücül etkilemez (Mars 59,9° Boğa, Satürn 90° Yengeç).
        assert_eq!(
            nature(Mercury, &with(&[(Mars, 59.9), (Saturn, 90.0)])),
            Nature::Benefic
        );
    }

    #[test]
    fn kuja_dosha_houses_from_each_reference() {
        // Mars Akrep'te (220°). Referans burcu r için Mars'ın evi = Akrep'in r'den sayısı.
        // Lagna Koç (0°): 8. ev → dosha. Ay Yengeç (100°): 5. ev → yok. Venüs Boğa (40°): 7. ev
        // → dosha.
        assert_eq!(
            kuja_dosha(&with(&[])),
            KujaDosha {
                from_lagna: Some(true),
                from_moon: false,
                from_venus: true
            }
        );
        // On iki evin tamamı, üç referans için: Mars Koç'ta sabit, referans geriye kayar.
        for house in 1..=12u8 {
            // Mars'ın referanstan evi `house` olsun: referans burcu = Koç − (house − 1).
            let reference = f64::from(12 - (house - 1)) * 30.0 + 15.0;
            let expected = [1, 2, 4, 7, 8, 12].contains(&house);
            let mut lons = BASE;
            lons[Mars as usize] = 5.0;
            lons[Moon as usize] = reference;
            lons[Venus as usize] = reference;
            let p = Positions::synthetic(Some(reference), lons);
            assert_eq!(p.house_from(Mars, p.sign_of(Moon)), house);
            assert_eq!(
                kuja_dosha(&p),
                KujaDosha {
                    from_lagna: Some(expected),
                    from_moon: expected,
                    from_venus: expected
                },
                "house {house}"
            );
        }
    }

    #[test]
    fn kuja_dosha_from_lagna_is_none_without_a_birth_time() {
        let p = with(&[]).without_lagna();
        assert_eq!(
            kuja_dosha(&p),
            KujaDosha {
                from_lagna: None,
                from_moon: false,
                from_venus: true
            }
        );
    }
}
```

- [ ] **Step 7: Modülleri kaydet ve dışa aktar**

```diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index d10db85..187b469 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -2,7 +2,8 @@
 //! trait'i üzerinden erişir; I/O, Tauri ve atlas bağımlılığı yoktur.
 //!
 //! Kapsam: ayanamsa, grahalar, lagna, bhavalar, nakshatra, dignity, yanma (M2), 16 varga,
-//! Vimshottari dasha, panchang, ashtakavarga ve transit (M3).
+//! Vimshottari dasha, panchang, ashtakavarga ve transit (M3). İleri modüllerin ortak temeli
+//! (Alt proje 3.1): `positions` (ortak girdi), `lordship` ve `nature`.
 
 pub mod ashtakavarga;
 mod chart;
@@ -14,7 +15,10 @@ mod error;
 mod fake;
 mod graha;
 mod houses;
+pub mod lordship;
+pub mod nature;
 pub mod panchang;
+pub mod positions;
 #[cfg(test)]
 mod proptests;
 pub mod settings;
@@ -36,11 +40,14 @@ pub use dignity::{dignity, Dignity, Relation};
 pub use error::CoreError;
 pub use graha::{Graha, GrahaPosition};
 pub use houses::{house_from_cusps, whole_sign_cusps, whole_sign_house, Bhavas, Lagna};
+pub use lordship::{house_lord, house_sign, houses_owned};
+pub use nature::{kuja_dosha, moon_is_waxing, nature, KujaDosha, Nature, KUJA_DOSHA_HOUSES};
 pub use panchang::{
     compute_panchang, elongation, karana_name, karana_no, local_weekday, nitya_yoga_no, paksha,
     tithi_no, Karana, KaranaName, NityaYoga, Paksha, Panchang, PanchangNakshatra, Tithi, Vara,
     END_BISECTIONS, END_SEARCH_DAYS, KARANA_SPAN_DEG, SUNRISE_WINDOW_DAYS, TITHI_SPAN_DEG,
 };
+pub use positions::{compute_positions, ChartInput, GrahaPoint, Positions};
 pub use settings::{
     AdvancedSettings, AyanamsaSystem, CalcSettings, CombustionOrbs, HouseSystem, KarakaScheme,
     NodeKind, SunriseDef, ZodiacMode, CALC_SETTINGS_SCHEMA_VERSION, JD_UT_RANGE,
```

- [ ] **Step 8: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p jyotish-core 2>&1 | grep -E "^test result|Running"
```
Expected: fmt ve clippy çıktısız; lib `125 passed` (4 `positions`, 3 `lordship`, 5 `nature`, 2 proptest); golden `8 passed`; `modules_golden` `2 passed`. Bir el örneği düşerse önce `derivation` kuraldan yeniden hesaplanır; beklenen değer motor çıktısına uydurulmaz (Global Constraints).

- [ ] **Step 9: Commit**

```bash
git add crates/jyotish-core/src crates/jyotish-core/tests/modules_golden.rs tests/golden/modules
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): shared module input (ChartInput, Positions), house lordship, natural benefic and malefic class, kuja dosha fact; hand fixtures"
```
Expected: iki `grep` çıktısız.

---

### Task 3: Çekirdek: graha bakışları (burç temelli tam bakış ve sphuta drishti)

Spec §4 "Yöntem" ve "Test". Tablo sınır noktalarında süreklilik, tam noktalarda beklenen değerler (60° → 15, 90° → 45, 120° → 30, 180° → 60, 210° → 45, 240° → 30, 270° → 15), özel bakışlarla 60, değerin her zaman [0, 60] oluşu, her graha için hedef burç kümesinin el tablosuyla birebir oluşu ve 7. bakışın simetrisi burada test edilir.

**Files:**
- Create: `crates/jyotish-core/src/drishti.rs`, `tests/golden/modules/drishti/*.json` (4 dosya)
- Modify: `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/proptests.rs`, `crates/jyotish-core/tests/modules_golden.rs`, `tests/golden/modules/README.md`
- Test: `drishti.rs` içindeki `mod tests`, `proptests.rs` (`drishti_properties`), `tests/modules_golden.rs`

**Interfaces:**
- Consumes: `Positions` (`point`, `sign_of`, `occupants`, `synthetic`, `without_lagna`), `Graha`, `Sign`, `normalize`; `modules_golden.rs` düzeneği (`Fixture<X>`, `load`); `foundation_properties::positions()` üreteci.
- Produces (hepsi `jyotish_core`'dan dışa aktarılır):
  - `pub const DRISHTI_METHOD: &str = "graha_drishti.bphs_v1"`, `pub const FULL_DRISHTI: f64 = 60.0`, `pub const ASPECTING_GRAHAS: [Graha; 7]`
  - `pub fn graha_aspect_houses(graha: Graha) -> &'static [u8]`, `pub fn graha_aspect_signs(graha: Graha, from: Sign) -> Vec<Sign>`
  - `pub fn sphuta_drishti(aspecting: Graha, from_lon: f64, to_lon: f64) -> f64`
  - `pub struct SignAspect { pub graha: Graha, pub target: Sign, pub occupants: Vec<Graha> }`
  - `pub struct DrishtiTable { pub sign_aspects: Vec<SignAspect>, pub sphuta: [[f64; 9]; 7], pub method: String }`
  - `pub fn drishti_table(positions: &Positions) -> DrishtiTable`

- [ ] **Step 1: El örneklerini yaz (kaynak: `hand`)**

Dört sentetik konum seti: özel bakışların tam noktaları; aralık içi kısmi değerler; burç bakışı ile derece değerinin ayrıştığı burç sınırı durumları (60'ta kesme dahil); 0°'yi aşan açılarda üst özel aralıklar. Her biri on üç tam bakışın tamamını ve en az on sphuta değerini açısıyla verir.

**Yeni dosya:** `tests/golden/modules/drishti/exact-special-points.json`

```json
{
  "id": "exact-special-points",
  "source": "hand",
  "rule_ref": "BPHS, bakışlar bölümü: her graha 7. burca; Mars 4 ve 8, Jüpiter 5 ve 9, Satürn 3 ve 10. Sphuta drishti tablosu ve özel bakış eklemesi: ileri modüller spec'i §4.",
  "derivation": [
    "Burçlar: Güneş 190° Terazi, Ay 100° Yengeç, Mars 10° Koç, Merkür 130° Aslan, Jüpiter 10° Koç, Venüs 70° İkizler, Satürn 10° Koç, Rahu 220° Akrep, Ketu 40° Boğa.",
    "Tam bakışlar. Güneş Terazi'den 7. Koç: Mars, Jüpiter, Satürn. Ay Yengeç'ten 7. Oğlak: boş. Mars Koç'tan 4. Yengeç: Ay; 7. Terazi: Güneş; 8. Akrep: Rahu. Merkür Aslan'dan 7. Kova: boş. Jüpiter Koç'tan 5. Aslan: Merkür; 7. Terazi: Güneş; 9. Yay: boş. Venüs İkizler'den 7. Yay: boş. Satürn Koç'tan 3. İkizler: Venüs; 7. Terazi: Güneş; 10. Oğlak: boş.",
    "Mars → Ay: açı 100 − 10 = 90°; taban (120 − 90)/2 + 30 = 45; Mars [90°, 120°) içinde +15 = 60.",
    "Mars → Rahu: açı 210°; taban (300 − 210)/2 = 45; Mars [210°, 240°) içinde +15 = 60.",
    "Jüpiter → Merkür: açı 120°; taban 150 − 120 = 30; Jüpiter [120°, 150°) içinde +30 = 60.",
    "Satürn → Venüs: açı 60°; taban (60 − 60) + 15 = 15; Satürn [60°, 90°) içinde +45 = 60.",
    "Mars → Güneş: açı 180°; taban (300 − 180)/2 = 60. Güneş → Mars: açı 10 − 190 = −180 ≡ 180°; 60.",
    "Satürn → Ay: açı 90°; taban 45; 90° Satürn'ün [60°, 90°) aralığının dışında, ekleme yok; 45. Jüpiter → Ay: açı 90°; 45.",
    "Mars → Merkür: açı 120°; taban 30; 120° Mars'ın [90°, 120°) aralığının dışında; 30. Mars → Ketu: açı 30°; taban (30 − 30)/2 = 0.",
    "Ay → Güneş: açı 190 − 100 = 90°; 45. Venüs → Güneş: açı 190 − 70 = 120°; 30.",
    "Merkür → Venüs: açı 70 − 130 = −60 ≡ 300°; 300° ve üstü 0. Ay → Venüs: açı 70 − 100 = −30 ≡ 330°; 0."
  ],
  "input": {
    "lagna_lon": null,
    "grahas": {"Sun": 190.0, "Moon": 100.0, "Mars": 10.0, "Mercury": 130.0, "Jupiter": 10.0, "Venus": 70.0, "Saturn": 10.0, "Rahu": 220.0, "Ketu": 40.0}
  },
  "expected": {
    "sign_aspects": [
      {"graha": "Sun", "target": "Aries", "occupants": ["Mars", "Jupiter", "Saturn"]},
      {"graha": "Moon", "target": "Capricorn", "occupants": []},
      {"graha": "Mars", "target": "Cancer", "occupants": ["Moon"]},
      {"graha": "Mars", "target": "Libra", "occupants": ["Sun"]},
      {"graha": "Mars", "target": "Scorpio", "occupants": ["Rahu"]},
      {"graha": "Mercury", "target": "Aquarius", "occupants": []},
      {"graha": "Jupiter", "target": "Leo", "occupants": ["Mercury"]},
      {"graha": "Jupiter", "target": "Libra", "occupants": ["Sun"]},
      {"graha": "Jupiter", "target": "Sagittarius", "occupants": []},
      {"graha": "Venus", "target": "Sagittarius", "occupants": []},
      {"graha": "Saturn", "target": "Gemini", "occupants": ["Venus"]},
      {"graha": "Saturn", "target": "Libra", "occupants": ["Sun"]},
      {"graha": "Saturn", "target": "Capricorn", "occupants": []}
    ],
    "sphuta": [
      {"from": "Mars", "to": "Moon", "angle": 90.0, "value": 60.0},
      {"from": "Mars", "to": "Rahu", "angle": 210.0, "value": 60.0},
      {"from": "Jupiter", "to": "Mercury", "angle": 120.0, "value": 60.0},
      {"from": "Saturn", "to": "Venus", "angle": 60.0, "value": 60.0},
      {"from": "Mars", "to": "Sun", "angle": 180.0, "value": 60.0},
      {"from": "Sun", "to": "Mars", "angle": 180.0, "value": 60.0},
      {"from": "Saturn", "to": "Moon", "angle": 90.0, "value": 45.0},
      {"from": "Jupiter", "to": "Moon", "angle": 90.0, "value": 45.0},
      {"from": "Mars", "to": "Mercury", "angle": 120.0, "value": 30.0},
      {"from": "Mars", "to": "Ketu", "angle": 30.0, "value": 0.0},
      {"from": "Moon", "to": "Sun", "angle": 90.0, "value": 45.0},
      {"from": "Venus", "to": "Sun", "angle": 120.0, "value": 30.0},
      {"from": "Mercury", "to": "Venus", "angle": 300.0, "value": 0.0},
      {"from": "Moon", "to": "Venus", "angle": 330.0, "value": 0.0}
    ]
  }
}
```

**Yeni dosya:** `tests/golden/modules/drishti/partial-values-inside-intervals.json`

```json
{
  "id": "partial-values-inside-intervals",
  "source": "hand",
  "rule_ref": "BPHS, bakışlar bölümü: her graha 7. burca; Mars 4 ve 8, Jüpiter 5 ve 9, Satürn 3 ve 10. Sphuta drishti tablosu ve özel bakış eklemesi: ileri modüller spec'i §4.",
  "derivation": [
    "Burçlar: Güneş 0° Koç, Ay 45° Boğa, Mars 20° Koç, Merkür 75° İkizler, Jüpiter 100° Yengeç, Venüs 165° Başak, Satürn 225° Akrep, Rahu 285° Oğlak, Ketu 105° Yengeç.",
    "Tam bakışlar. Güneş Koç'tan 7. Terazi: boş. Ay Boğa'dan 7. Akrep: Satürn. Mars Koç'tan 4. Yengeç: Jüpiter, Ketu; 7. Terazi: boş; 8. Akrep: Satürn. Merkür İkizler'den 7. Yay: boş. Jüpiter Yengeç'ten 5. Akrep: Satürn; 7. Oğlak: Rahu; 9. Balık: boş. Venüs Başak'tan 7. Balık: boş. Satürn Akrep'ten 3. Oğlak: Rahu; 7. Boğa: Ay; 10. Aslan: boş.",
    "Güneş'ten (0°) açı bakılanın boylamıdır. Ay 45°: (45 − 30)/2 = 7,5. Merkür 75°: (75 − 60) + 15 = 30. Jüpiter 100°: (120 − 100)/2 + 30 = 40. Ketu 105°: (120 − 105)/2 + 30 = 37,5. Venüs 165°: (165 − 150) × 2 = 30. Satürn 225°: (300 − 225)/2 = 37,5. Rahu 285°: (300 − 285)/2 = 7,5.",
    "Mars'tan (20°). Jüpiter: açı 80°; (80 − 60) + 15 = 35; 80° Mars'ın [90°, 120°) aralığının dışında; 35. Ketu: açı 85°; 40. Satürn: açı 205°; (300 − 205)/2 = 47,5; 205° [210°, 240°) dışında; 47,5. Rahu: açı 265°; (300 − 265)/2 = 17,5.",
    "Jüpiter'den (100°). Satürn: açı 125°; taban 150 − 125 = 25; Jüpiter [120°, 150°) içinde +30 = 55. Rahu: açı 185°; (300 − 185)/2 = 57,5.",
    "Satürn'den (225°). Rahu: açı 60°; 15 + 45 = 60. Ay: açı 45 − 225 = −180 ≡ 180°; 60. Güneş: açı 0 − 225 ≡ 135°; 150 − 135 = 15; Satürn eklemesi yok; 15. Mars: açı 20 − 225 ≡ 155°; (155 − 150) × 2 = 10. Jüpiter: açı 100 − 225 ≡ 235°; (300 − 235)/2 = 32,5.",
    "Ay → Satürn: açı 225 − 45 = 180°; 60. Venüs → Güneş: açı 0 − 165 ≡ 195°; (300 − 195)/2 = 52,5."
  ],
  "input": {
    "lagna_lon": null,
    "grahas": {"Sun": 0.0, "Moon": 45.0, "Mars": 20.0, "Mercury": 75.0, "Jupiter": 100.0, "Venus": 165.0, "Saturn": 225.0, "Rahu": 285.0, "Ketu": 105.0}
  },
  "expected": {
    "sign_aspects": [
      {"graha": "Sun", "target": "Libra", "occupants": []},
      {"graha": "Moon", "target": "Scorpio", "occupants": ["Saturn"]},
      {"graha": "Mars", "target": "Cancer", "occupants": ["Jupiter", "Ketu"]},
      {"graha": "Mars", "target": "Libra", "occupants": []},
      {"graha": "Mars", "target": "Scorpio", "occupants": ["Saturn"]},
      {"graha": "Mercury", "target": "Sagittarius", "occupants": []},
      {"graha": "Jupiter", "target": "Scorpio", "occupants": ["Saturn"]},
      {"graha": "Jupiter", "target": "Capricorn", "occupants": ["Rahu"]},
      {"graha": "Jupiter", "target": "Pisces", "occupants": []},
      {"graha": "Venus", "target": "Pisces", "occupants": []},
      {"graha": "Saturn", "target": "Capricorn", "occupants": ["Rahu"]},
      {"graha": "Saturn", "target": "Taurus", "occupants": ["Moon"]},
      {"graha": "Saturn", "target": "Leo", "occupants": []}
    ],
    "sphuta": [
      {"from": "Sun", "to": "Moon", "angle": 45.0, "value": 7.5},
      {"from": "Sun", "to": "Mercury", "angle": 75.0, "value": 30.0},
      {"from": "Sun", "to": "Jupiter", "angle": 100.0, "value": 40.0},
      {"from": "Sun", "to": "Ketu", "angle": 105.0, "value": 37.5},
      {"from": "Sun", "to": "Venus", "angle": 165.0, "value": 30.0},
      {"from": "Sun", "to": "Saturn", "angle": 225.0, "value": 37.5},
      {"from": "Sun", "to": "Rahu", "angle": 285.0, "value": 7.5},
      {"from": "Mars", "to": "Jupiter", "angle": 80.0, "value": 35.0},
      {"from": "Mars", "to": "Ketu", "angle": 85.0, "value": 40.0},
      {"from": "Mars", "to": "Saturn", "angle": 205.0, "value": 47.5},
      {"from": "Mars", "to": "Rahu", "angle": 265.0, "value": 17.5},
      {"from": "Jupiter", "to": "Saturn", "angle": 125.0, "value": 55.0},
      {"from": "Jupiter", "to": "Rahu", "angle": 185.0, "value": 57.5},
      {"from": "Saturn", "to": "Rahu", "angle": 60.0, "value": 60.0},
      {"from": "Saturn", "to": "Moon", "angle": 180.0, "value": 60.0},
      {"from": "Saturn", "to": "Sun", "angle": 135.0, "value": 15.0},
      {"from": "Saturn", "to": "Mars", "angle": 155.0, "value": 10.0},
      {"from": "Saturn", "to": "Jupiter", "angle": 235.0, "value": 32.5},
      {"from": "Moon", "to": "Saturn", "angle": 180.0, "value": 60.0},
      {"from": "Venus", "to": "Sun", "angle": 195.0, "value": 52.5}
    ]
  }
}
```

**Yeni dosya:** `tests/golden/modules/drishti/sign-aspect-and-degree-value-disagree.json`

```json
{
  "id": "sign-aspect-and-degree-value-disagree",
  "source": "hand",
  "rule_ref": "BPHS, bakışlar bölümü: her graha 7. burca; Mars 4 ve 8, Jüpiter 5 ve 9, Satürn 3 ve 10. Sphuta drishti tablosu ve özel bakış eklemesi: ileri modüller spec'i §4.",
  "derivation": [
    "Burçlar: Güneş 149° Aslan, Ay 91° Yengeç, Mars 29° Koç, Merkür 179° Başak, Jüpiter 1° Koç, Venüs 88° İkizler, Satürn 2° Koç, Rahu 179° Başak, Ketu 359° Balık.",
    "Tam bakışlar. Güneş Aslan'dan 7. Kova: boş. Ay Yengeç'ten 7. Oğlak: boş. Mars Koç'tan 4. Yengeç: Ay; 7. Terazi: boş; 8. Akrep: boş. Merkür Başak'tan 7. Balık: Ketu. Jüpiter Koç'tan 5. Aslan: Güneş; 7. Terazi: boş; 9. Yay: boş. Venüs İkizler'den 7. Yay: boş. Satürn Koç'tan 3. İkizler: Venüs; 7. Terazi: boş; 10. Oğlak: boş.",
    "Burç bakışı tam olsa da derece değeri kısmi olabilir. Mars → Ay: açı 91 − 29 = 62°; (62 − 60) + 15 = 17; 62° Mars'ın [90°, 120°) aralığının dışında; 17.",
    "Jüpiter → Güneş: açı 148°; taban 150 − 148 = 2; Jüpiter [120°, 150°) içinde +30 = 32.",
    "Satürn → Venüs: açı 86°; taban (86 − 60) + 15 = 41; Satürn [60°, 90°) içinde +45 = 86; 60'ta kesilir.",
    "Merkür → Ketu: açı 359 − 179 = 180°; 60.",
    "Mars → Güneş: açı 120°; taban 30; ekleme yok; 30. Mars → Merkür: açı 150°; taban 150 − 150 = 0.",
    "Satürn → Merkür: açı 177°; (177 − 150) × 2 = 54. Jüpiter → Merkür: açı 178°; (178 − 150) × 2 = 56. Jüpiter → Ay: açı 90°; 45.",
    "Satürn → Ay: açı 89°; taban (89 − 60) + 15 = 44; +45 = 89; 60'ta kesilir.",
    "Güneş → Ketu: açı 359 − 149 = 210°; (300 − 210)/2 = 45. Ay → Mars: açı 29 − 91 = −62 ≡ 298°; (300 − 298)/2 = 1. Venüs → Ketu: açı 359 − 88 = 271°; (300 − 271)/2 = 14,5. Merkür → Jüpiter: açı 1 − 179 = −178 ≡ 182°; (300 − 182)/2 = 59."
  ],
  "input": {
    "lagna_lon": null,
    "grahas": {"Sun": 149.0, "Moon": 91.0, "Mars": 29.0, "Mercury": 179.0, "Jupiter": 1.0, "Venus": 88.0, "Saturn": 2.0, "Rahu": 179.0, "Ketu": 359.0}
  },
  "expected": {
    "sign_aspects": [
      {"graha": "Sun", "target": "Aquarius", "occupants": []},
      {"graha": "Moon", "target": "Capricorn", "occupants": []},
      {"graha": "Mars", "target": "Cancer", "occupants": ["Moon"]},
      {"graha": "Mars", "target": "Libra", "occupants": []},
      {"graha": "Mars", "target": "Scorpio", "occupants": []},
      {"graha": "Mercury", "target": "Pisces", "occupants": ["Ketu"]},
      {"graha": "Jupiter", "target": "Leo", "occupants": ["Sun"]},
      {"graha": "Jupiter", "target": "Libra", "occupants": []},
      {"graha": "Jupiter", "target": "Sagittarius", "occupants": []},
      {"graha": "Venus", "target": "Sagittarius", "occupants": []},
      {"graha": "Saturn", "target": "Gemini", "occupants": ["Venus"]},
      {"graha": "Saturn", "target": "Libra", "occupants": []},
      {"graha": "Saturn", "target": "Capricorn", "occupants": []}
    ],
    "sphuta": [
      {"from": "Mars", "to": "Moon", "angle": 62.0, "value": 17.0},
      {"from": "Jupiter", "to": "Sun", "angle": 148.0, "value": 32.0},
      {"from": "Saturn", "to": "Venus", "angle": 86.0, "value": 60.0},
      {"from": "Mercury", "to": "Ketu", "angle": 180.0, "value": 60.0},
      {"from": "Mars", "to": "Sun", "angle": 120.0, "value": 30.0},
      {"from": "Mars", "to": "Mercury", "angle": 150.0, "value": 0.0},
      {"from": "Saturn", "to": "Mercury", "angle": 177.0, "value": 54.0},
      {"from": "Jupiter", "to": "Mercury", "angle": 178.0, "value": 56.0},
      {"from": "Jupiter", "to": "Moon", "angle": 90.0, "value": 45.0},
      {"from": "Saturn", "to": "Moon", "angle": 89.0, "value": 60.0},
      {"from": "Sun", "to": "Ketu", "angle": 210.0, "value": 45.0},
      {"from": "Moon", "to": "Mars", "angle": 298.0, "value": 1.0},
      {"from": "Venus", "to": "Ketu", "angle": 271.0, "value": 14.5},
      {"from": "Mercury", "to": "Jupiter", "angle": 182.0, "value": 59.0}
    ]
  }
}
```

**Yeni dosya:** `tests/golden/modules/drishti/upper-special-intervals-across-zero.json`

```json
{
  "id": "upper-special-intervals-across-zero",
  "source": "hand",
  "rule_ref": "BPHS, bakışlar bölümü: her graha 7. burca; Mars 4 ve 8, Jüpiter 5 ve 9, Satürn 3 ve 10. Sphuta drishti tablosu ve özel bakış eklemesi: ileri modüller spec'i §4.",
  "derivation": [
    "Burçlar: Güneş 235° Akrep, Ay 225° Akrep, Mars 350° Balık, Merkür 120° Aslan, Jüpiter 340° Balık, Venüs 215° Akrep, Satürn 300° Kova, Rahu 30° Boğa, Ketu 210° Akrep. Akrep'teki grahalar sırayla: Güneş, Ay, Venüs, Ketu.",
    "Tam bakışlar. Güneş, Ay ve Venüs Akrep'ten 7. Boğa: Rahu. Mars Balık'tan 4. İkizler, 7. Başak, 8. Terazi: üçü de boş. Merkür Aslan'dan 7. Kova: Satürn. Jüpiter Balık'tan 5. Yengeç: boş; 7. Başak: boş; 9. Akrep: Güneş, Ay, Venüs, Ketu. Satürn Kova'dan 3. Koç: boş; 7. Aslan: Merkür; 10. Akrep: Güneş, Ay, Venüs, Ketu.",
    "Mars'tan (350°). Venüs: açı 215 − 350 = −135 ≡ 225°; taban (300 − 225)/2 = 37,5; Mars [210°, 240°) içinde +15 = 52,5 (Akrep Balık'tan 9. burçtur: burç bakışı yok, derece değeri var). Ketu: açı 210 − 350 ≡ 220°; 40 + 15 = 55. Güneş: açı 235 − 350 ≡ 245°; (300 − 245)/2 = 27,5; 245° aralığın dışında; 27,5.",
    "Jüpiter'den (340°). Güneş: açı 235 − 340 ≡ 255°; taban 22,5; Jüpiter [240°, 270°) içinde +30 = 52,5. Ay: açı 225 − 340 ≡ 245°; 27,5 + 30 = 57,5. Ketu: açı 210 − 340 ≡ 230°; (300 − 230)/2 = 35; 230° aralığın dışında; 35.",
    "Satürn'den (300°). Ay: açı 225 − 300 ≡ 285°; taban 7,5; Satürn [270°, 300°) içinde +45 = 52,5. Güneş: açı 295°; 2,5 + 45 = 47,5. Venüs: açı 275°; 12,5 + 45 = 57,5. Ketu: açı 270°; 15 + 45 = 60. Merkür: açı 120 − 300 ≡ 180°; 60. Rahu: açı 30 − 300 ≡ 90°; 45; 90° [60°, 90°) dışında; 45.",
    "Merkür → Satürn: açı 300 − 120 = 180°; 60.",
    "Rahu'ya (30°): Güneş'ten açı 30 − 235 ≡ 155°; (155 − 150) × 2 = 10. Venüs'ten açı 30 − 215 ≡ 175°; 50. Ay'dan açı 30 − 225 ≡ 165°; 30. Mars'tan açı 30 − 350 ≡ 40°; (40 − 30)/2 = 5."
  ],
  "input": {
    "lagna_lon": null,
    "grahas": {"Sun": 235.0, "Moon": 225.0, "Mars": 350.0, "Mercury": 120.0, "Jupiter": 340.0, "Venus": 215.0, "Saturn": 300.0, "Rahu": 30.0, "Ketu": 210.0}
  },
  "expected": {
    "sign_aspects": [
      {"graha": "Sun", "target": "Taurus", "occupants": ["Rahu"]},
      {"graha": "Moon", "target": "Taurus", "occupants": ["Rahu"]},
      {"graha": "Mars", "target": "Gemini", "occupants": []},
      {"graha": "Mars", "target": "Virgo", "occupants": []},
      {"graha": "Mars", "target": "Libra", "occupants": []},
      {"graha": "Mercury", "target": "Aquarius", "occupants": ["Saturn"]},
      {"graha": "Jupiter", "target": "Cancer", "occupants": []},
      {"graha": "Jupiter", "target": "Virgo", "occupants": []},
      {"graha": "Jupiter", "target": "Scorpio", "occupants": ["Sun", "Moon", "Venus", "Ketu"]},
      {"graha": "Venus", "target": "Taurus", "occupants": ["Rahu"]},
      {"graha": "Saturn", "target": "Aries", "occupants": []},
      {"graha": "Saturn", "target": "Leo", "occupants": ["Mercury"]},
      {"graha": "Saturn", "target": "Scorpio", "occupants": ["Sun", "Moon", "Venus", "Ketu"]}
    ],
    "sphuta": [
      {"from": "Mars", "to": "Venus", "angle": 225.0, "value": 52.5},
      {"from": "Mars", "to": "Ketu", "angle": 220.0, "value": 55.0},
      {"from": "Mars", "to": "Sun", "angle": 245.0, "value": 27.5},
      {"from": "Jupiter", "to": "Sun", "angle": 255.0, "value": 52.5},
      {"from": "Jupiter", "to": "Moon", "angle": 245.0, "value": 57.5},
      {"from": "Jupiter", "to": "Ketu", "angle": 230.0, "value": 35.0},
      {"from": "Saturn", "to": "Moon", "angle": 285.0, "value": 52.5},
      {"from": "Saturn", "to": "Sun", "angle": 295.0, "value": 47.5},
      {"from": "Saturn", "to": "Venus", "angle": 275.0, "value": 57.5},
      {"from": "Saturn", "to": "Ketu", "angle": 270.0, "value": 60.0},
      {"from": "Saturn", "to": "Mercury", "angle": 180.0, "value": 60.0},
      {"from": "Saturn", "to": "Rahu", "angle": 90.0, "value": 45.0},
      {"from": "Mercury", "to": "Saturn", "angle": 180.0, "value": 60.0},
      {"from": "Sun", "to": "Rahu", "angle": 155.0, "value": 10.0},
      {"from": "Venus", "to": "Rahu", "angle": 175.0, "value": 50.0},
      {"from": "Moon", "to": "Rahu", "angle": 165.0, "value": 30.0},
      {"from": "Mars", "to": "Rahu", "angle": 40.0, "value": 5.0}
    ]
  }
}
```

- [ ] **Step 2: Düzeneğe `drishti` modülünü ve değişmezleri ekle (başarısız testler)**

```diff
diff --git a/crates/jyotish-core/tests/modules_golden.rs b/crates/jyotish-core/tests/modules_golden.rs
index 62121d5..a21a777 100644
--- a/crates/jyotish-core/tests/modules_golden.rs
+++ b/crates/jyotish-core/tests/modules_golden.rs
@@ -7,14 +7,17 @@ use std::collections::BTreeMap;
 use std::path::PathBuf;
 
 use jyotish_core::{
-    house_lord, houses_owned, kuja_dosha, moon_is_waxing, nature, Graha, KujaDosha, Nature,
-    Positions,
+    drishti_table, house_lord, houses_owned, kuja_dosha, moon_is_waxing, nature, normalize, Graha,
+    KujaDosha, Nature, Positions, Sign, ASPECTING_GRAHAS, DRISHTI_METHOD,
 };
 use serde::de::DeserializeOwned;
 use serde::Deserialize;
 
 /// Modül başına en az örnek sayısı; altına düşerse test düşer (sessiz silinmeye karşı).
 const MIN_FOUNDATION: usize = 6;
+const MIN_DRISHTI: usize = 4;
+/// El hesabı ile motor arasındaki en büyük fark, virupa.
+const VIRUPA_TOL: f64 = 1e-9;
 
 #[derive(Debug, Deserialize)]
 #[serde(deny_unknown_fields)]
@@ -153,3 +156,73 @@ fn unknown_fields_are_rejected() {
     let extra_expected = base.replace(r#""waxing":true,"#, r#""waxing":true,"more":1,"#);
     assert!(serde_json::from_str::<Fixture<FoundationExpected>>(&extra_expected).is_err());
 }
+
+#[derive(Debug, Deserialize, PartialEq)]
+#[serde(deny_unknown_fields)]
+struct ExpectedAspect {
+    graha: Graha,
+    target: Sign,
+    occupants: Vec<Graha>,
+}
+
+/// Bir sphuta drishti değeri: `angle` = (bakılan − bakan) mod 360, `value` virupa.
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ExpectedSphuta {
+    from: Graha,
+    to: Graha,
+    angle: f64,
+    value: f64,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct DrishtiExpected {
+    /// On üç tam bakışın tamamı, tablo sırasıyla.
+    sign_aspects: Vec<ExpectedAspect>,
+    sphuta: Vec<ExpectedSphuta>,
+}
+
+#[test]
+fn drishti_hand_fixtures() {
+    for (f, p) in load::<DrishtiExpected>("drishti", MIN_DRISHTI) {
+        let id = &f.id;
+        let table = drishti_table(&p);
+        assert_eq!(table.method, DRISHTI_METHOD, "{id}");
+        let got: Vec<ExpectedAspect> = table
+            .sign_aspects
+            .iter()
+            .map(|a| ExpectedAspect {
+                graha: a.graha,
+                target: a.target,
+                occupants: a.occupants.clone(),
+            })
+            .collect();
+        assert_eq!(got, f.expected.sign_aspects, "{id}: sign aspects");
+        assert!(f.expected.sphuta.len() >= 10, "{id}: at least ten values");
+        for s in &f.expected.sphuta {
+            let row = ASPECTING_GRAHAS
+                .iter()
+                .position(|&g| g == s.from)
+                .unwrap_or_else(|| panic!("{id}: {:?} casts no graha aspect", s.from));
+            // Türetmedeki açı girdiyle tutarlı olmalı.
+            let angle = normalize(p.point(s.to).sidereal_lon - p.point(s.from).sidereal_lon);
+            assert!(
+                (angle - s.angle).abs() < 1e-9,
+                "{id}: {:?} → {:?} angle {angle}, fixture says {}",
+                s.from,
+                s.to,
+                s.angle
+            );
+            let got = table.sphuta[row][s.to as usize];
+            assert!(
+                (got - s.value).abs() <= VIRUPA_TOL,
+                "{id}: {:?} → {:?} at {}°: got {got}, hand value {}",
+                s.from,
+                s.to,
+                s.angle,
+                s.value
+            );
+        }
+    }
+}
diff --git a/tests/golden/modules/README.md b/tests/golden/modules/README.md
index 996d7a6..f2afc86 100644
--- a/tests/golden/modules/README.md
+++ b/tests/golden/modules/README.md
@@ -37,5 +37,6 @@ ileri modüller tasarım belgesi §13 (dört katmanlı test stratejisinin ilk ka
 | Klasör | `expected` | En az |
 |---|---|---|
 | `foundation` | `waxing`, `nature` (dokuz graha), `kuja_dosha`, `house_lords` (12), `houses`, `houses_owned` | 6 |
+| `drishti` | `sign_aspects` (on üç tam bakışın tamamı, tablo sırasıyla), `sphuta` (en az on değer: `from`, `to`, `angle`, `value`) | 4 |
 
 Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
diff --git a/crates/jyotish-core/src/proptests.rs b/crates/jyotish-core/src/proptests.rs
index c281da6..5362376 100644
--- a/crates/jyotish-core/src/proptests.rs
+++ b/crates/jyotish-core/src/proptests.rs
@@ -362,7 +362,7 @@ mod foundation_properties {
     use crate::positions::Positions;
     use crate::zodiac::Sign;
 
-    fn positions() -> impl Strategy<Value = Positions> {
+    pub(super) fn positions() -> impl Strategy<Value = Positions> {
         (
             proptest::option::of(0.0f64..360.0),
             proptest::array::uniform9(0.0f64..360.0),
@@ -422,3 +422,63 @@ mod foundation_properties {
         }
     }
 }
+
+/// Graha bakışları (Alt proje 3.1).
+mod drishti_properties {
+    use proptest::prelude::*;
+
+    use super::foundation_properties::positions;
+    use crate::drishti::{
+        drishti_table, graha_aspect_signs, sphuta_drishti, ASPECTING_GRAHAS, FULL_DRISHTI,
+    };
+    use crate::graha::Graha;
+
+    proptest! {
+        #[test]
+        fn sphuta_drishti_is_within_zero_and_sixty(
+            g in 0usize..9,
+            from in -720.0f64..720.0,
+            to in -720.0f64..720.0,
+        ) {
+            let v = sphuta_drishti(Graha::ALL[g], from, to);
+            prop_assert!(v.is_finite() && (0.0..=FULL_DRISHTI).contains(&v), "{v}");
+        }
+
+        #[test]
+        fn sphuta_drishti_depends_only_on_the_angle(
+            g in 0usize..7,
+            from in 0.0f64..360.0,
+            angle in 0.0f64..360.0,
+            shift in -360.0f64..360.0,
+        ) {
+            // Özel bakış aralıklarının uçlarında değer sıçrar; yuvarlama hatasıyla ucun öbür
+            // yanına düşmemek için 30°'nin katlarından uzak durulur.
+            prop_assume!((angle / 30.0 - (angle / 30.0).round()).abs() > 1e-6);
+            let graha = ASPECTING_GRAHAS[g];
+            let a = sphuta_drishti(graha, from, from + angle);
+            let b = sphuta_drishti(graha, from + shift, from + shift + angle);
+            prop_assert!((a - b).abs() < 1e-6, "{a} vs {b}");
+        }
+
+        #[test]
+        fn drishti_table_is_consistent_with_the_positions(p in positions()) {
+            let t = drishti_table(&p);
+            prop_assert_eq!(t.sign_aspects.len(), 13);
+            for a in &t.sign_aspects {
+                prop_assert!(!a.graha.is_node());
+                prop_assert!(graha_aspect_signs(a.graha, p.sign_of(a.graha)).contains(&a.target));
+                prop_assert_eq!(&a.occupants, &p.occupants(a.target));
+            }
+            for (i, row) in t.sphuta.iter().enumerate() {
+                for (j, v) in row.iter().enumerate() {
+                    prop_assert!((0.0..=FULL_DRISHTI).contains(v));
+                    if Graha::ALL[j] == ASPECTING_GRAHAS[i] {
+                        prop_assert_eq!(*v, 0.0);
+                    }
+                }
+            }
+            // Lagna bakış tablosunu etkilemez.
+            prop_assert_eq!(drishti_table(&p.clone().without_lagna()), t);
+        }
+    }
+}
```

- [ ] **Step 3: Testlerin başarısız olduğunu doğrula**

Run: `cargo test -p jyotish-core 2>&1 | grep -E "^error" | head -3`
Expected: derleme hatası (`unresolved import crate::drishti`).

- [ ] **Step 4: `drishti.rs`**

**Yeni dosya:** `crates/jyotish-core/src/drishti.rs`

```rust
//! Graha bakışları (ileri modüller spec'i §4): burç temelli tam bakış ve derece temelli bakış
//! değeri (sphuta drishti).
//!
//! Kaynak: BPHS, bakışlar bölümü. Her graha bulunduğu burçtan 7. burca bakar; ek tam bakışlar
//! Mars 4 ve 8, Jüpiter 5 ve 9, Satürn 3 ve 10'dur. Rahu ve Ketu'ya graha bakışı atanmaz.
//! Sphuta drishti parçalı doğrusaldır (virupa, 0–60); özel bakışlar eklenir ve sonuç 60'ta
//! kesilir. Evler ve burçlar rasi çerçevesindedir (K4, K5); açı sidereal boylamlardan alınır.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::positions::Positions;
use crate::zodiac::{normalize, Sign};

/// `DrishtiTable::method`: bu dosyadaki yöntemin kimliği.
pub const DRISHTI_METHOD: &str = "graha_drishti.bphs_v1";

/// Tam bakış değeri, virupa.
pub const FULL_DRISHTI: f64 = 60.0;

/// Bakan grahalar: `DrishtiTable::sphuta` satır sırası.
pub const ASPECTING_GRAHAS: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// Grahanın tam bakışla baktığı evler (kendi burcundan sayılır), artan sırada. Düğümler için
/// boştur.
pub fn graha_aspect_houses(graha: Graha) -> &'static [u8] {
    match graha {
        Graha::Sun | Graha::Moon | Graha::Mercury | Graha::Venus => &[7],
        Graha::Mars => &[4, 7, 8],
        Graha::Jupiter => &[5, 7, 9],
        Graha::Saturn => &[3, 7, 10],
        Graha::Rahu | Graha::Ketu => &[],
    }
}

/// `from` burcundaki grahanın tam bakışla baktığı burçlar, ev sırasıyla.
pub fn graha_aspect_signs(graha: Graha, from: Sign) -> Vec<Sign> {
    graha_aspect_houses(graha)
        .iter()
        .map(|&h| Sign::from_index(from.index() + usize::from(h) - 1))
        .collect()
}

/// Özel bakış eklemesi: (açı aralığının başı, eklenen virupa). Aralık 30° genişliğindedir.
fn special_bonus(aspecting: Graha) -> &'static [(f64, f64)] {
    match aspecting {
        Graha::Mars => &[(90.0, 15.0), (210.0, 15.0)],
        Graha::Jupiter => &[(120.0, 30.0), (240.0, 30.0)],
        Graha::Saturn => &[(60.0, 45.0), (270.0, 45.0)],
        _ => &[],
    }
}

/// Özel bakışsız temel değer: `angle` = (bakılan − bakan) mod 360, derece.
fn base_drishti(angle: f64) -> f64 {
    let a = angle;
    if a < 30.0 {
        0.0
    } else if a < 60.0 {
        (a - 30.0) / 2.0
    } else if a < 90.0 {
        (a - 60.0) + 15.0
    } else if a < 120.0 {
        (120.0 - a) / 2.0 + 30.0
    } else if a < 150.0 {
        150.0 - a
    } else if a < 180.0 {
        (a - 150.0) * 2.0
    } else if a < 300.0 {
        (300.0 - a) / 2.0
    } else {
        0.0
    }
}

/// `aspecting` grahasının `from_lon` boylamından `to_lon` boylamına bakış değeri, virupa,
/// [0, 60]. Boylamlar sidereal derecedir; bakılan nokta herhangi bir boylam olabilir (graha ya
/// da bhava ortası). Rahu ve Ketu bakan olarak 0 verir.
pub fn sphuta_drishti(aspecting: Graha, from_lon: f64, to_lon: f64) -> f64 {
    if aspecting.is_node() {
        return 0.0;
    }
    let angle = normalize(to_lon - from_lon);
    let bonus: f64 = special_bonus(aspecting)
        .iter()
        .filter(|&&(start, _)| angle >= start && angle < start + 30.0)
        .map(|&(_, add)| add)
        .sum();
    (base_drishti(angle) + bonus).min(FULL_DRISHTI)
}

/// Bir grahanın bir burca tam bakışı ve o burçtaki grahalar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SignAspect {
    pub graha: Graha,
    pub target: Sign,
    /// Hedef burçtaki grahalar, `Graha::ALL` sırasında (dokuz graha da sayılır).
    pub occupants: Vec<Graha>,
}

/// `compute_drishti` sonucu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DrishtiTable {
    /// Yedi grahanın tam bakışları: `ASPECTING_GRAHAS` sırasında, her graha için ev sırasıyla.
    /// Her zaman 13 kayıt: Güneş, Ay, Merkür ve Venüs birer; Mars, Jüpiter ve Satürn üçer.
    pub sign_aspects: Vec<SignAspect>,
    /// `sphuta[i][j]`: `ASPECTING_GRAHAS[i]`'nin `Graha::ALL[j]`'ye bakış değeri, virupa.
    /// Grahanın kendisine bakışı 0'dır.
    pub sphuta: [[f64; 9]; 7],
    pub method: String,
}

/// Saf hesap: konumlardan bakış tablosu. Lagnaya bağlı değildir.
pub fn drishti_table(positions: &Positions) -> DrishtiTable {
    let sign_aspects = ASPECTING_GRAHAS
        .into_iter()
        .flat_map(|graha| {
            graha_aspect_signs(graha, positions.sign_of(graha))
                .into_iter()
                .map(move |target| SignAspect {
                    graha,
                    target,
                    occupants: positions.occupants(target),
                })
        })
        .collect();
    let sphuta = std::array::from_fn(|i| {
        let from = positions.point(ASPECTING_GRAHAS[i]);
        std::array::from_fn(|j| {
            let to = &positions.grahas[j];
            if to.graha == from.graha {
                0.0
            } else {
                sphuta_drishti(from.graha, from.sidereal_lon, to.sidereal_lon)
            }
        })
    });
    DrishtiTable {
        sign_aspects,
        sphuta,
        method: DRISHTI_METHOD.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;
    use Sign::*;

    #[test]
    fn sign_aspects_match_the_hand_table_for_every_graha() {
        // Koç'tan sayım: 3 İkizler, 4 Yengeç, 5 Aslan, 7 Terazi, 8 Akrep, 9 Yay, 10 Oğlak.
        assert_eq!(graha_aspect_signs(Sun, Aries), [Libra]);
        assert_eq!(graha_aspect_signs(Moon, Aries), [Libra]);
        assert_eq!(graha_aspect_signs(Mercury, Aries), [Libra]);
        assert_eq!(graha_aspect_signs(Venus, Aries), [Libra]);
        assert_eq!(graha_aspect_signs(Mars, Aries), [Cancer, Libra, Scorpio]);
        assert_eq!(
            graha_aspect_signs(Jupiter, Aries),
            [Leo, Libra, Sagittarius]
        );
        assert_eq!(
            graha_aspect_signs(Saturn, Aries),
            [Gemini, Libra, Capricorn]
        );
        assert!(graha_aspect_signs(Rahu, Aries).is_empty());
        assert!(graha_aspect_signs(Ketu, Aries).is_empty());
        // Burç sonundan sarma: Oğlak'tan 4 Koç, 7 Yengeç, 8 Aslan; Balık'tan 3 Boğa, 10 Yay.
        assert_eq!(graha_aspect_signs(Mars, Capricorn), [Aries, Cancer, Leo]);
        assert_eq!(
            graha_aspect_signs(Saturn, Pisces),
            [Taurus, Virgo, Sagittarius]
        );
        assert_eq!(
            graha_aspect_signs(Jupiter, Sagittarius),
            [Aries, Gemini, Leo]
        );
    }

    #[test]
    fn seventh_aspect_is_symmetric_and_every_graha_has_it() {
        for from in Sign::ALL {
            for g in ASPECTING_GRAHAS {
                let seventh = Sign::from_index(from.index() + 6);
                let targets = graha_aspect_signs(g, from);
                assert!(targets.contains(&seventh), "{g:?} {from:?}");
                // Karşı burçtaki aynı graha geri bakar.
                assert!(graha_aspect_signs(g, seventh).contains(&from));
                assert_eq!(targets.len(), graha_aspect_houses(g).len());
                assert!(!targets.contains(&from));
            }
        }
    }

    #[test]
    fn base_values_at_the_table_points() {
        // Özel bakışı olmayan graha (Güneş) temel tabloyu verir.
        for (angle, want) in [
            (0.0, 0.0),
            (15.0, 0.0),
            (30.0, 0.0),
            (45.0, 7.5),
            (60.0, 15.0),
            (75.0, 30.0),
            (90.0, 45.0),
            (105.0, 37.5),
            (120.0, 30.0),
            (135.0, 15.0),
            (150.0, 0.0),
            (165.0, 30.0),
            (180.0, 60.0),
            (210.0, 45.0),
            (240.0, 30.0),
            (270.0, 15.0),
            (285.0, 7.5),
            (300.0, 0.0),
            (330.0, 0.0),
            (359.999, 0.0),
        ] {
            assert_eq!(sphuta_drishti(Sun, 0.0, angle), want, "angle {angle}");
            // Yalnızca fark önemlidir; 360°'den sarma da aynı sonucu verir.
            let got = sphuta_drishti(Venus, 350.0, 350.0 + angle);
            assert!((got - want).abs() < 1e-9, "wrapped {angle}: {got}");
        }
    }

    #[test]
    fn base_table_is_continuous_at_every_breakpoint() {
        for edge in [30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 300.0] {
            let below = sphuta_drishti(Moon, 0.0, edge - 1e-9);
            let at = sphuta_drishti(Moon, 0.0, edge);
            let above = sphuta_drishti(Moon, 0.0, edge + 1e-9);
            assert!((below - at).abs() < 1e-6, "below {edge}: {below} vs {at}");
            assert!((above - at).abs() < 1e-6, "above {edge}: {above} vs {at}");
        }
    }

    #[test]
    fn special_aspects_are_full_at_their_exact_points() {
        // Mars 4. ve 8. (90°, 210°), Jüpiter 5. ve 9. (120°, 240°), Satürn 3. ve 10. (60°, 270°).
        for (g, angle) in [
            (Mars, 90.0),
            (Mars, 210.0),
            (Jupiter, 120.0),
            (Jupiter, 240.0),
            (Saturn, 60.0),
            (Saturn, 270.0),
        ] {
            assert_eq!(sphuta_drishti(g, 0.0, angle), 60.0, "{g:?} {angle}");
        }
        // Herkes için 7. bakış (180°) tamdır.
        for g in ASPECTING_GRAHAS {
            assert_eq!(sphuta_drishti(g, 0.0, 180.0), 60.0, "{g:?}");
        }
    }

    #[test]
    fn special_bonus_applies_only_inside_its_interval() {
        // Mars [90, 120): taban + 15. 105° → 37,5 + 15 = 52,5. 120° aralık dışı → 30.
        assert_eq!(sphuta_drishti(Mars, 0.0, 105.0), 52.5);
        assert_eq!(sphuta_drishti(Mars, 0.0, 120.0), 30.0);
        assert!((sphuta_drishti(Mars, 0.0, 89.999) - 44.999).abs() < 1e-9);
        // Mars [210, 240): 225° → 37,5 + 15 = 52,5. 240° → 30.
        assert_eq!(sphuta_drishti(Mars, 0.0, 225.0), 52.5);
        assert_eq!(sphuta_drishti(Mars, 0.0, 240.0), 30.0);
        // Jüpiter [120, 150): 135° → 15 + 30 = 45. 150° → 0. [240, 270): 255° → 22,5 + 30.
        assert_eq!(sphuta_drishti(Jupiter, 0.0, 135.0), 45.0);
        assert_eq!(sphuta_drishti(Jupiter, 0.0, 150.0), 0.0);
        assert_eq!(sphuta_drishti(Jupiter, 0.0, 255.0), 52.5);
        assert_eq!(sphuta_drishti(Jupiter, 0.0, 270.0), 15.0);
        // Satürn [60, 90): 75° → 30 + 45 = 75, 60'ta kesilir. 90° → 45.
        assert_eq!(sphuta_drishti(Saturn, 0.0, 75.0), 60.0);
        assert_eq!(sphuta_drishti(Saturn, 0.0, 90.0), 45.0);
        // Satürn [270, 300): 285° → 7,5 + 45 = 52,5. 300° → 0.
        assert_eq!(sphuta_drishti(Saturn, 0.0, 285.0), 52.5);
        assert_eq!(sphuta_drishti(Saturn, 0.0, 300.0), 0.0);
        // Başka grahanın özel aralığında ekleme yoktur.
        assert_eq!(sphuta_drishti(Mars, 0.0, 135.0), 15.0);
        assert_eq!(sphuta_drishti(Jupiter, 0.0, 105.0), 37.5);
        assert_eq!(sphuta_drishti(Saturn, 0.0, 225.0), 37.5);
    }

    #[test]
    fn nodes_cast_no_graha_aspect() {
        for angle in [0.0, 90.0, 180.0, 270.0] {
            assert_eq!(sphuta_drishti(Rahu, 0.0, angle), 0.0);
            assert_eq!(sphuta_drishti(Ketu, 0.0, angle), 0.0);
        }
    }

    #[test]
    fn table_lists_thirteen_sign_aspects_with_occupants() {
        // Güneş Koç 10°, Ay Terazi 190°, Mars Koç 5°, Merkür Koç 20°, Jüpiter Aslan 125°,
        // Venüs Boğa 40°, Satürn Oğlak 280°, Rahu Yengeç 100°, Ketu Oğlak 280°.
        let p = Positions::synthetic(
            None,
            [10.0, 190.0, 5.0, 20.0, 125.0, 40.0, 280.0, 100.0, 280.0],
        );
        let t = drishti_table(&p);
        assert_eq!(t.method, "graha_drishti.bphs_v1");
        let got: Vec<(Graha, Sign, Vec<Graha>)> = t
            .sign_aspects
            .iter()
            .map(|a| (a.graha, a.target, a.occupants.clone()))
            .collect();
        assert_eq!(
            got,
            [
                (Sun, Libra, vec![Moon]),
                (Moon, Aries, vec![Sun, Mars, Mercury]),
                (Mars, Cancer, vec![Rahu]),
                (Mars, Libra, vec![Moon]),
                (Mars, Scorpio, vec![]),
                (Mercury, Libra, vec![Moon]),
                // Jüpiter Aslan: 5. Yay, 7. Kova, 9. Koç.
                (Jupiter, Sagittarius, vec![]),
                (Jupiter, Aquarius, vec![]),
                (Jupiter, Aries, vec![Sun, Mars, Mercury]),
                (Venus, Scorpio, vec![]),
                // Satürn Oğlak: 3. Balık, 7. Yengeç, 10. Terazi.
                (Saturn, Pisces, vec![]),
                (Saturn, Cancer, vec![Rahu]),
                (Saturn, Libra, vec![Moon]),
            ]
        );
    }

    #[test]
    fn table_sphuta_matrix_matches_the_function() {
        let p = Positions::synthetic(
            None,
            [10.0, 190.0, 5.0, 20.0, 125.0, 40.0, 280.0, 100.0, 280.0],
        );
        let t = drishti_table(&p);
        // Güneş (10°) → Ay (190°): 180° → 60. Ay → Güneş: 180° → 60.
        assert_eq!(t.sphuta[0][1], 60.0);
        assert_eq!(t.sphuta[1][0], 60.0);
        // Mars (5°) → Rahu (100°): 95° → (120 − 95)/2 + 30 = 42,5; özel +15 → 57,5.
        assert_eq!(t.sphuta[2][7], 57.5);
        // Jüpiter (125°) → Güneş (10°): 245° → (300 − 245)/2 = 27,5; özel +30 → 57,5.
        assert_eq!(t.sphuta[4][0], 57.5);
        // Satürn (280°) → Ay (190°): 270° → 15; özel +45 → 60.
        assert_eq!(t.sphuta[6][1], 60.0);
        // Satürn (280°) → Ketu (280°): 0° → 0. Köşegen 0.
        assert_eq!(t.sphuta[6][8], 0.0);
        for (i, g) in ASPECTING_GRAHAS.into_iter().enumerate() {
            assert_eq!(t.sphuta[i][g as usize], 0.0, "{g:?}");
            for (j, to) in Graha::ALL.into_iter().enumerate() {
                if to != g {
                    assert_eq!(
                        t.sphuta[i][j],
                        sphuta_drishti(g, p.point(g).sidereal_lon, p.point(to).sidereal_lon)
                    );
                }
            }
        }
    }
}
```

- [ ] **Step 5: Modülü kaydet ve dışa aktar**

```diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 187b469..78fedc0 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -3,13 +3,14 @@
 //!
 //! Kapsam: ayanamsa, grahalar, lagna, bhavalar, nakshatra, dignity, yanma (M2), 16 varga,
 //! Vimshottari dasha, panchang, ashtakavarga ve transit (M3). İleri modüllerin ortak temeli
-//! (Alt proje 3.1): `positions` (ortak girdi), `lordship` ve `nature`.
+//! (Alt proje 3.1): `positions` (ortak girdi), `lordship`, `nature` ve `drishti`.
 
 pub mod ashtakavarga;
 mod chart;
 mod combustion;
 pub mod dasha;
 pub mod dignity;
+pub mod drishti;
 mod error;
 #[cfg(test)]
 mod fake;
@@ -37,6 +38,10 @@ pub use dasha::{
     VIMSHOTTARI_TOTAL_YEARS,
 };
 pub use dignity::{dignity, Dignity, Relation};
+pub use drishti::{
+    drishti_table, graha_aspect_houses, graha_aspect_signs, sphuta_drishti, DrishtiTable,
+    SignAspect, ASPECTING_GRAHAS, DRISHTI_METHOD, FULL_DRISHTI,
+};
 pub use error::CoreError;
 pub use graha::{Graha, GrahaPosition};
 pub use houses::{house_from_cusps, whole_sign_cusps, whole_sign_house, Bhavas, Lagna};
```

- [ ] **Step 6: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p jyotish-core 2>&1 | grep -E "^test result|Running"
```
Expected: fmt ve clippy çıktısız; lib `137 passed` (9 `drishti`, 3 proptest); golden `8 passed`; `modules_golden` `3 passed`. Düşen el örneğinde Task 2 Step 8'deki kural geçerlidir.

- [ ] **Step 7: Commit**

```bash
git add crates/jyotish-core/src crates/jyotish-core/tests/modules_golden.rs tests/golden/modules
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): graha aspects by sign and by degree (sphuta drishti); hand fixtures and invariants"
```
Expected: iki `grep` çıktısız.

---

### Task 4: Uygulama: `compute_drishti` komutu, bağlamalar, sarmalayıcı ve fikstür

Spec §4 "Veri modeli ve IPC", K1, K8; §13 "Komut katmanı"; §15 bütçesi. Modül komutlarının ortak kalıbı burada kurulur: tek `ChartInput`, motor iş parçacığı, `blocking` havuzu, `CommandError`.

**Files:**
- Create: `app/src-tauri/src/module_commands.rs`, `app/src-tauri/tests/modules.rs`
- Modify: `crates/jyotish-core/src/{drishti,lib}.rs`, `app/src-tauri/src/lib.rs`, `app/src-tauri/tests/ui_fixture.rs`, `app/ui/api.ts`, `app/ui/api.test.ts`, `app/ui/test/fixture.ts`, `app/e2e/ipc.ts`
- Üretilir: `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`
- Test: `app/src-tauri/tests/modules.rs`, `drishti.rs` içindeki `mod tests`, `app/ui/api.test.ts`

**Interfaces:**
- Consumes: `compute_positions`, `drishti_table`, `ChartInput`, `DrishtiTable` (Task 2, 3); `ephemeris::{Engine, SwissEph}`; `client_commands::blocking`; `commands::{CommandError, compute_chart_blocking}`.
- Produces:
  - `jyotish_core::compute_drishti<E: Ephemeris + ?Sized>(eph: &mut E, input: &ChartInput) -> Result<DrishtiTable, CoreError>`
  - `module_commands::compute_drishti_blocking(engine: &Engine, input: ChartInput) -> Result<DrishtiTable, CommandError>`
  - Tauri komutu `compute_drishti(input: ChartInput) -> Result<DrishtiTable, CommandError>`
  - TS: `commands.computeDrishti(input)`, `api.computeDrishti(input: ChartInput)`, tipler `ChartInput`, `DrishtiTable`, `SignAspect`; `UiFixture.drishti: DrishtiTable`; e2e taklidinde `compute_drishti`.

- [ ] **Step 1: Başarısız entegrasyon testini yaz**

Beş golden anında komut gövdesi hata vermez; tablo, v1'de golden fixture'larla doğrulanmış haritanın konumlarından `sphuta_drishti` ve `graha_aspect_signs` ile yeniden kurulan tabloyla birebir aynıdır (çapraz denetim, §13 katman 4); doğum saati bilinmese de aynıdır; hatalar `compute_chart`'ın kodlarına eşlenir; ortanca süre ≤ 10 ms.

**Yeni dosya:** `app/src-tauri/tests/modules.rs`

```rust
//! İleri modül komut gövdeleri, gerçek `Engine` ile uçtan uca (ileri modüller spec'i §13
//! "Komut katmanı"; §15 bütçeleri). Tek test: süreçte yalnızca bir SE örneği açılabilir.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use ephemeris::Engine;
use jyotish_core::{
    graha_aspect_signs, sphuta_drishti, CalcSettings, ChartInput, Graha, ASPECTING_GRAHAS,
    DRISHTI_METHOD,
};
use jyotish_desk_lib::commands::compute_chart_blocking;
use jyotish_desk_lib::module_commands::compute_drishti_blocking;
use serde_json::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// `tests/golden/*.json` fixture'larının anları (ad, girdi).
fn golden_inputs() -> Vec<(String, ChartInput)> {
    let dir = root().join("../../tests/golden");
    let mut out: Vec<(String, ChartInput)> = std::fs::read_dir(&dir)
        .expect("golden dir")
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .map(|p| {
            let v: Value =
                serde_json::from_str(&std::fs::read_to_string(&p).expect("read fixture"))
                    .expect("parse fixture");
            let num = |k: &str| v["input"][k].as_f64().expect("number");
            let input = ChartInput {
                jd_ut: num("jd_ut"),
                lat: num("lat"),
                lon: num("lon"),
                settings: serde_json::from_value(v["settings"].clone()).expect("settings"),
                time_known: true,
            };
            (v["id"].as_str().expect("id").to_owned(), input)
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[test]
fn compute_drishti_runs_on_the_engine_thread_and_maps_errors() {
    let engine = Engine::spawn(root().join("../../data/ephe")).expect("spawn engine");
    let inputs = golden_inputs();
    assert!(inputs.len() >= 5, "{} golden moments", inputs.len());

    for (id, input) in &inputs {
        let table = compute_drishti_blocking(&engine, input.clone())
            .unwrap_or_else(|e| panic!("{id}: {}", e.code));
        assert_eq!(table.method, DRISHTI_METHOD, "{id}");
        assert_eq!(table.sign_aspects.len(), 13, "{id}");
        // Çapraz denetim: v1'de golden fixture'larla doğrulanmış haritanın konumlarından
        // aynı tablo çıkar.
        let chart = compute_chart_blocking(
            &engine,
            input.jd_ut,
            input.lat,
            input.lon,
            input.settings.clone(),
        )
        .expect("chart");
        let pos = |g: Graha| &chart.grahas[g as usize];
        for a in &table.sign_aspects {
            assert!(
                graha_aspect_signs(a.graha, pos(a.graha).sign).contains(&a.target),
                "{id}: {:?} → {:?}",
                a.graha,
                a.target
            );
            let occupants: Vec<Graha> = chart
                .grahas
                .iter()
                .filter(|p| p.sign == a.target)
                .map(|p| p.graha)
                .collect();
            assert_eq!(a.occupants, occupants, "{id}: occupants of {:?}", a.target);
        }
        for (i, from) in ASPECTING_GRAHAS.into_iter().enumerate() {
            for to in Graha::ALL {
                let want = if to == from {
                    0.0
                } else {
                    sphuta_drishti(from, pos(from).sidereal_lon, pos(to).sidereal_lon)
                };
                let got = table.sphuta[i][to as usize];
                assert!((0.0..=60.0).contains(&got), "{id}: {got}");
                assert_eq!(got, want, "{id}: {from:?} → {to:?}");
            }
        }
        // Bakış tablosu lagnaya bağlı değildir: saat bilinmese de aynıdır.
        let unknown = ChartInput {
            time_known: false,
            ..input.clone()
        };
        assert_eq!(
            compute_drishti_blocking(&engine, unknown).expect("time unknown"),
            table,
            "{id}"
        );
    }

    // Hatalar `compute_chart` ile aynı kodlara eşlenir; parametreler danışan verisi taşımaz.
    let delhi = inputs
        .iter()
        .find(|(id, _)| id == "new-delhi-2000-01-01")
        .expect("delhi fixture")
        .1
        .clone();
    let bad_place = ChartInput {
        lat: 91.0,
        ..delhi.clone()
    };
    assert_eq!(
        compute_drishti_blocking(&engine, bad_place)
            .unwrap_err()
            .code,
        "invalid_location"
    );
    let old_settings = ChartInput {
        settings: CalcSettings {
            schema_version: 1,
            ..CalcSettings::default()
        },
        ..delhi.clone()
    };
    let err = compute_drishti_blocking(&engine, old_settings).unwrap_err();
    assert_eq!(err.code, "unsupported_schema_version");
    assert_eq!(err.params["found"], "1");
    let bad_moment = ChartInput {
        jd_ut: f64::NAN,
        ..delhi.clone()
    };
    assert_eq!(
        compute_drishti_blocking(&engine, bad_moment)
            .unwrap_err()
            .code,
        "invalid_julian_day"
    );

    // Performans: spec §15 "compute_drishti ≤ 10 ms (temel harita hesabı dahil)". Ortanca,
    // ısınmadan sonra (open_client ölçümüyle aynı düzenek).
    compute_drishti_blocking(&engine, delhi.clone()).expect("warm-up");
    let mut times: Vec<Duration> = (0..15)
        .map(|_| {
            let t = Instant::now();
            compute_drishti_blocking(&engine, delhi.clone()).expect("drishti");
            t.elapsed()
        })
        .collect();
    times.sort();
    let median = times[times.len() / 2];
    eprintln!(
        "compute_drishti median {median:?}, max {:?}",
        times[times.len() - 1]
    );
    assert!(
        median <= Duration::from_millis(10),
        "compute_drishti median {median:?}"
    );
}
```

Run: `cargo test -p jyotish-desk --test modules 2>&1 | grep -E "^error" | head -3`
Expected: derleme hatası (`unresolved import jyotish_desk_lib::module_commands`).

- [ ] **Step 2: Çekirdek toplayıcı `compute_drishti`**

```diff
diff --git a/crates/jyotish-core/src/drishti.rs b/crates/jyotish-core/src/drishti.rs
index 648a1a8..6d3077f 100644
--- a/crates/jyotish-core/src/drishti.rs
+++ b/crates/jyotish-core/src/drishti.rs
@@ -6,11 +6,13 @@
 //! Sphuta drishti parçalı doğrusaldır (virupa, 0–60); özel bakışlar eklenir ve sonuç 60'ta
 //! kesilir. Evler ve burçlar rasi çerçevesindedir (K4, K5); açı sidereal boylamlardan alınır.
 
+use ephemeris::Ephemeris;
 use serde::{Deserialize, Serialize};
 
 use crate::graha::Graha;
-use crate::positions::Positions;
+use crate::positions::{compute_positions, ChartInput, Positions};
 use crate::zodiac::{normalize, Sign};
+use crate::CoreError;
 
 /// `DrishtiTable::method`: bu dosyadaki yöntemin kimliği.
 pub const DRISHTI_METHOD: &str = "graha_drishti.bphs_v1";
@@ -152,12 +154,50 @@ pub fn drishti_table(positions: &Positions) -> DrishtiTable {
     }
 }
 
+/// `compute_drishti` komutunun hesabı: temel harita, konumlar, bakış tablosu. Bakış tablosu
+/// lagnaya bağlı değildir; `input.time_known` sonucu değiştirmez.
+pub fn compute_drishti<E: Ephemeris + ?Sized>(
+    eph: &mut E,
+    input: &ChartInput,
+) -> Result<DrishtiTable, CoreError> {
+    Ok(drishti_table(&compute_positions(eph, input)?))
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::fake::FakeEph;
+    use crate::settings::CalcSettings;
     use Graha::*;
     use Sign::*;
 
+    #[test]
+    fn collector_builds_the_table_from_the_chart_positions() {
+        let mut eph = FakeEph::default();
+        let mut input = ChartInput {
+            jd_ut: 2_451_545.0,
+            lat: 41.0,
+            lon: 29.0,
+            settings: CalcSettings::default(),
+            time_known: true,
+        };
+        let table = compute_drishti(&mut eph, &input).unwrap();
+        let positions = compute_positions(&mut eph, &input).unwrap();
+        assert_eq!(table, drishti_table(&positions));
+        // FakeEph (ayanamsa 24°): Güneş sidereal 346°, Ay 71°: açı 85° → (85 − 60) + 15 = 40.
+        assert!((table.sphuta[0][1] - 40.0).abs() < 1e-9);
+        input.time_known = false;
+        assert_eq!(compute_drishti(&mut eph, &input).unwrap(), table);
+        input.lat = 91.0;
+        assert_eq!(
+            compute_drishti(&mut eph, &input),
+            Err(CoreError::InvalidLocation {
+                lat: 91.0,
+                lon: 29.0
+            })
+        );
+    }
+
     #[test]
     fn sign_aspects_match_the_hand_table_for_every_graha() {
         // Koç'tan sayım: 3 İkizler, 4 Yengeç, 5 Aslan, 7 Terazi, 8 Akrep, 9 Yay, 10 Oğlak.
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 78fedc0..63ad8dd 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -39,8 +39,8 @@ pub use dasha::{
 };
 pub use dignity::{dignity, Dignity, Relation};
 pub use drishti::{
-    drishti_table, graha_aspect_houses, graha_aspect_signs, sphuta_drishti, DrishtiTable,
-    SignAspect, ASPECTING_GRAHAS, DRISHTI_METHOD, FULL_DRISHTI,
+    compute_drishti, drishti_table, graha_aspect_houses, graha_aspect_signs, sphuta_drishti,
+    DrishtiTable, SignAspect, ASPECTING_GRAHAS, DRISHTI_METHOD, FULL_DRISHTI,
 };
 pub use error::CoreError;
 pub use graha::{Graha, GrahaPosition};
```

- [ ] **Step 3: `module_commands.rs` ve komut kaydı**

**Yeni dosya:** `app/src-tauri/src/module_commands.rs`

```rust
//! İleri modül komutları (ileri modüller spec'i K1–K3). Ortak kalıp: tek `ChartInput`, hesap
//! motor iş parçacığında, komut gövdesi `blocking` havuzunda, hata `CommandError`. Modül
//! sonuçları `Chart`'a eklenmez ve saklanmaz; panel açıkken istenir. Sonraki alt projelerin
//! komutları bu dosyaya aynı kalıpla eklenir.

use std::sync::Arc;

use ephemeris::{Engine, SwissEph};
use jyotish_core::{ChartInput, DrishtiTable};
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::CommandError;

/// `compute_drishti` komutunun gövdesi; bloklar.
pub fn compute_drishti_blocking(
    engine: &Engine,
    input: ChartInput,
) -> Result<DrishtiTable, CommandError> {
    let table =
        engine.call(move |se: &mut SwissEph| jyotish_core::compute_drishti(se, &input))??;
    Ok(table)
}

/// Graha bakışları: on üç tam bakış ve 7 × 9 sphuta drishti matrisi. `input`, açık haritanın
/// `jd_ut`, konum ve ayarlarıdır; `time_known` doğum saatinin bilinip bilinmediğidir.
#[tauri::command]
#[specta::specta]
pub async fn compute_drishti(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
) -> Result<DrishtiTable, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_drishti_blocking(&engine, input)).await
}
```

```diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index 5f35a8a..e3b9687 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -2,6 +2,7 @@ pub mod atlas_commands;
 pub mod client_commands;
 pub mod commands;
 pub mod data_dir;
+pub mod module_commands;
 pub mod store_commands;
 pub mod transit_commands;
 pub mod update_commands;
@@ -28,6 +29,7 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             transit_commands::transits_at,
             commands::dasha_children,
             commands::print_window,
+            module_commands::compute_drishti,
             client_commands::open_client,
             client_commands::restore_session,
             client_commands::create_client,
```

Run: `cargo test -p jyotish-core --lib drishti 2>&1 | grep -E "^test result"; cargo test -p jyotish-desk --test modules -- --nocapture 2>&1 | grep -E "median|^test result"`
Expected: `13 passed` (adında `drishti` geçen 10 birim testi ve 3 proptest); `compute_drishti median …` satırı (bu plan yazılırken 0,6 ms) ve `1 passed`.

- [ ] **Step 4: Fikstüre bakış tablosunu ekle; bağlamaları ve fikstürü üret**

```diff
diff --git a/app/src-tauri/tests/ui_fixture.rs b/app/src-tauri/tests/ui_fixture.rs
index 691d64f..8cf0902 100644
--- a/app/src-tauri/tests/ui_fixture.rs
+++ b/app/src-tauri/tests/ui_fixture.rs
@@ -11,9 +11,11 @@ use std::sync::Mutex;
 
 use atlas::Timestamp;
 use ephemeris::Engine;
+use jyotish_core::ChartInput;
 use jyotish_desk_lib::atlas_commands::{preview_moment_body, search_places_blocking, AtlasState};
 use jyotish_desk_lib::client_commands::create_client_blocking;
 use jyotish_desk_lib::commands::dasha_children;
+use jyotish_desk_lib::module_commands::compute_drishti_blocking;
 use jyotish_desk_lib::transit_commands::{transits_at_blocking, TransitAt};
 use serde_json::{json, Value};
 use store::{BirthData, ChartKind, NewChart, NewClient, Place, Store, TimeCertainty, TzChoice};
@@ -93,6 +95,18 @@ fn ui_fixture_is_up_to_date() {
         opened.chart.dasha.seed,
     )
     .expect("transit");
+    // Bakış tablosu: arayüzün `chartInput(opened)` ile kurduğu girdinin aynısı.
+    let drishti = compute_drishti_blocking(
+        &engine,
+        ChartInput {
+            jd_ut: opened.chart.jd_ut,
+            lat: opened.chart.geo_lat,
+            lon: opened.chart.geo_lon,
+            settings: opened.chart.meta.settings.clone(),
+            time_known: true,
+        },
+    )
+    .expect("drishti");
     // Kaydedilen doğum verisi girdinin kanonik biçimidir (Plan 10 kapsam dışı 21).
     assert_eq!(opened.chart_record.birth, delhi());
     let prefs = store
@@ -108,6 +122,7 @@ fn ui_fixture_is_up_to_date() {
         "gap_error": gap_error,
         "transit": transit,
         "prefs": prefs,
+        "drishti": drishti,
     });
     let text = serde_json::to_string(&value)
         .expect("serialize")
diff --git a/app/ui/test/fixture.ts b/app/ui/test/fixture.ts
index 0bb88a2..bb1dc51 100644
--- a/app/ui/test/fixture.ts
+++ b/app/ui/test/fixture.ts
@@ -1,7 +1,7 @@
 // Gerçek veri fikstürü (app/src-tauri/tests/ui_fixture.rs üretir): Delhi 2000-01-01 17:30
 // için `create_client` dönüşü, ilk 3. seviye dönemin alt dönemleri, "new delhi" atlas araması,
-// an önizlemesi, bir DST boşluğu hatası ve aynı sabit saatte transit. Arayüz hesap yapmadığı için testler bunu çizer.
-import type { CommandError, DashaPeriod, Graha, MomentPreview, OpenedClient, PlaceHit, PrefsLoad, TransitView } from "../bindings";
+// an önizlemesi, bir DST boşluğu hatası, aynı sabit saatte transit ve bakış tablosu. Arayüz hesap yapmadığı için testler bunu çizer.
+import type { CommandError, DashaPeriod, DrishtiTable, Graha, MomentPreview, OpenedClient, PlaceHit, PrefsLoad, TransitView } from "../bindings";
 import raw from "./fixtures/delhi.json?raw";
 
 export interface UiFixture {
@@ -14,6 +14,8 @@ export interface UiFixture {
   transit: TransitView;
   /** `get_app_prefs` boş store için: varsayılanlar, uyarı yok. */
   prefs: PrefsLoad;
+  /** `compute_drishti`, açık haritanın girdisiyle (`time_known: true`). */
+  drishti: DrishtiTable;
 }
 
 const FIXTURE = JSON.parse(raw) as UiFixture;
```

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
git diff --stat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
grep -c "computeDrishti" app/ui/bindings.ts
```
Expected: iki `ok`; `bindings.ts` 43 satır eklenir (`computeDrishti`, `ChartInput`, `DrishtiTable`, `SignAspect`); `delhi.json` tek satır değişir (`"drishti"` anahtarı eklenir); son komut `1` yazar.

- [ ] **Step 5: Arayüz sarmalayıcısı, testi ve e2e taklidi**

```diff
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index 15695df..d315fff 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -28,6 +28,7 @@ vi.mock("./bindings", () => ({
     exportClientsCsv: vi.fn(),
     importClientsJson: vi.fn(),
     dashaChildren: vi.fn(),
+    computeDrishti: vi.fn(),
     searchPlaces: vi.fn(),
     previewMoment: vi.fn(),
     getAppPrefs: vi.fn(),
@@ -107,6 +108,7 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["exportClientsCsv", []],
     ["importClientsJson", []],
     ["dashaChildren", [{ birth_jd_ut: 1 }, ["Sun"]]],
+    ["computeDrishti", [{ jd_ut: 1, time_known: true }]],
     ["searchPlaces", ["delhi", 8]],
     ["previewMoment", [{ date: "2000-01-01" }]],
     ["getAppPrefs", []],
diff --git a/app/ui/api.ts b/app/ui/api.ts
index b1c12e7..ba1f93c 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -10,6 +10,7 @@ import {
   type AppPrefs,
   type Bhavas,
   type CalcSettings,
+  type ChartInput,
   type CommandError,
   type DashaSeed,
   type Gender,
@@ -75,6 +76,9 @@ export const api = {
   exportClientsCsv: () => wrap(() => commands.exportClientsCsv()),
   importClientsJson: () => wrap(() => commands.importClientsJson()),
   dashaChildren: (seed: DashaSeed, path: Graha[]) => wrap(() => commands.dashaChildren(seed, path)),
+  // İleri modüller: sonuç saklanmaz, panel açıkken istenir (ileri modüller spec'i K1). Girdi
+  // `modules/input.ts`'teki `chartInput(opened)` ile kurulur.
+  computeDrishti: (input: ChartInput) => wrap(() => commands.computeDrishti(input)),
   searchPlaces: (query: string, limit: number) => wrap(() => commands.searchPlaces(query, limit)),
   previewMoment: (birth: BirthData) => wrap(() => commands.previewMoment(birth)),
   appInfo: () => wrap(() => commands.appInfo()),
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index 9f937b0..f4b4868 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -93,6 +93,8 @@ export async function mockIpc(page: Page, { session, unknownTime = false }: IpcO
         dasha_children: (args) =>
           JSON.stringify(args.path) === JSON.stringify(fixture.dasha_children.path) ? fixture.dasha_children.periods : [],
         transits_at: () => fixture.transit,
+        // Bakış tablosu yeniden hesaplanmaz: fikstürdeki gerçek sonuç döner.
+        compute_drishti: () => fixture.drishti,
         print_window: () => null,
         // Güncelleme: sahte yayın; ağ yoktur. Kurulum yalnızca denetimin bildirdiği sürümü kabul eder.
         app_info: () => ({ version: "0.9.0", updaterConfigured: true }),
```

- [ ] **Step 6: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | tail -1
```
Expected: fmt ve clippy çıktısız; `jyotish-core` lib `138 passed`; `jyotish-desk` `modules` `1 passed`, `bindings` `1 passed`, `ui_fixture` `2 passed`; diğer Rust satırları Task 3'teki gibi; Vitest `60 passed (60)`, `527 passed (527)` (`api.test.ts`'in tablosu büyür, test sayısı değişmez; "dizgi bağımsız değişken" testi değişmeden geçer: C2′); `✓ built`.

- [ ] **Step 7: Commit**

```bash
git add crates/jyotish-core/src app/src-tauri/src app/src-tauri/tests app/ui/bindings.ts app/ui/test app/ui/api.ts app/ui/api.test.ts app/e2e/ipc.ts
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(app): compute_drishti command on the shared ChartInput pattern; bindings, wrapper and fixture"
```
Expected: iki `grep` çıktısız.

---

### Task 5: Arayüz altyapısı: modül panel türleri, paletten ekleme/kaldırma, ortak girdi ve `useModule`

D9 ve D13. Üç yeni tür kataloğa girer ve bu görevde "sonraki bir sürümde gelecek" yer tutucusuyla (`UnavailablePanel`) kayıtlıdır; içerikleri Task 6–8'de gelir. Davranış: palette (harita açıkken) üç yeni "… panelini aç/kapat" öğesi görünür; seçilen panel etkin düzenin altına eklenir ve odak ona geçer; yeniden seçilince kaldırılır ve odak paneldeyse ana bölgeye geçer.

**Files:**
- Create: `app/ui/modules/input.ts`, `app/ui/modules/useModule.ts`, `app/ui/modules/input.test.ts`, `app/ui/modules/useModule.test.tsx`
- Modify: `app/ui/layout/{model,reducer}.ts`, `app/ui/view/store.tsx`, `app/ui/palette/{items.ts,Palette.tsx}`, `app/ui/shell/{Shell.tsx,focus.ts}`, `app/ui/panels/registry.ts`, `app/ui/i18n/{tr,en,fr}.ts`
- Test: `app/ui/layout/{model,reducer}.test.ts`, `app/ui/palette/items.test.ts`, `app/ui/shell/focus.test.ts`, `app/ui/panels/panels.test.tsx`, `app/ui/App.test.tsx`, yeni iki test dosyası

**Interfaces:**
- Consumes: `ChartInput`, `OpenedClient`, `CommandError` (bağlamalar); `Result<T>` (`api.ts`); `timeUnknown` (`session/birth.ts`); `layoutReducer`, `LayoutStore`, `PaletteItem`, `leaveNotes` kalıbı.
- Produces:
  - `layout/model.ts`: `PANEL_KINDS` sonuna `"panchang"`, `"ashtakavarga"`, `"drishti"`; `MODULE_PANELS: readonly { kind; w; h }[]` (`panchang` 4 × 9, `ashtakavarga` 6 × 11, `drishti` 6 × 10); `type ModulePanelKind`
  - `layout/reducer.ts`: eylem `{ type: "togglePanel"; kind: ModulePanelKind; panelId: string }`
  - `view/store.tsx`: `LayoutStore.togglePanel(kind: ModulePanelKind): void`
  - `palette/items.ts`: `PaletteItem` üyesi `{ kind: "panel"; key; label; panelKind: ModulePanelKind }`; `LocalSources.panels`
  - `shell/focus.ts`: `leavePanel(kind: string): void`
  - `modules/input.ts`: `chartInput(opened: OpenedClient): ChartInput`, `inputKey(opened: OpenedClient): string`
  - `modules/useModule.ts`: `useModule<T>(key: string | null, fetch: () => Promise<Result<T>>): ModuleState<T>`; `interface ModuleState<T> { data: T | null; error: CommandError | null; loading: boolean; retry(): void }`
  - i18n: `panel.{panchang,ashtakavarga,drishti}`, `palette.kind.panel`, `palette.togglePanel` (`{{name}}`)

- [ ] **Step 1: Başarısız testleri yaz**

**Yeni dosya:** `app/ui/modules/input.test.ts`

```ts
import { expect, test } from "vitest";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { chartInput, inputKey } from "./input";

test("module input is the computed chart's moment, place and settings", () => {
  const opened = fixture().opened;
  const input = chartInput(opened);
  expect(input).toEqual({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true });
  // Hesapta kullanılan ayarlar: haritanın `meta.settings` kopyası, `advanced` bloğuyla.
  expect(input.settings.schema_version).toBe(2);
  expect(input.settings.advanced).toEqual({ karaka_scheme: "Eight", jaimini_co_lords: true });
});

test("an unknown birth time is passed on so that Rust hides lagna-dependent output", () => {
  expect(chartInput(unknownTimeOpened()).time_known).toBe(false);
});

test("the input key changes with the chart, the birth time certainty and the settings", () => {
  const base = fixture().opened;
  const key = inputKey(base);
  expect(inputKey(fixture().opened)).toBe(key);
  const other = fixture().opened;
  other.chart_record.id += 1;
  expect(inputKey(other)).not.toBe(key);
  expect(inputKey(unknownTimeOpened())).not.toBe(key);
  const raman = fixture().opened;
  raman.chart.meta.settings = { ...raman.chart.meta.settings, ayanamsa: "Raman" };
  expect(inputKey(raman)).not.toBe(key);
  const moved = fixture().opened;
  moved.chart.jd_ut = 2451545.5;
  expect(inputKey(moved)).not.toBe(key);
});
```

**Yeni dosya:** `app/ui/modules/useModule.test.tsx`

```tsx
import { act, render, screen } from "@testing-library/react";
import { StrictMode } from "react";
import { expect, test, vi } from "vitest";
import type { Result } from "../api";
import type { CommandError } from "../bindings";
import { useModule } from "./useModule";

const failure: CommandError = { code: "calc", params: { detail: "x" } };

/** Elle çözülen söz: yanıtın geliş sırasını test belirler. */
function deferred<T>() {
  let resolve!: (r: Result<T>) => void;
  const promise = new Promise<Result<T>>((r) => (resolve = r));
  return { promise, resolve };
}

function Probe({ k, fetch }: { k: string | null; fetch: () => Promise<Result<string>> }) {
  const m = useModule(k, fetch);
  return (
    <div>
      <span data-testid="state">{m.loading ? "loading" : m.error ? `error:${m.error.code}` : (m.data ?? "idle")}</span>
      <button type="button" onClick={m.retry}>
        retry
      </button>
    </div>
  );
}

const state = () => screen.getByTestId("state").textContent;

test("fetches once per key and shows the result", async () => {
  const d = deferred<string>();
  const fetch = vi.fn(() => d.promise);
  const { rerender } = render(<Probe k="a" fetch={fetch} />);
  expect(state()).toBe("loading");
  await act(async () => d.resolve({ status: "ok", data: "A" }));
  expect(state()).toBe("A");
  // Aynı anahtarla yeniden çizim yeni istek göndermez.
  rerender(<Probe k="a" fetch={fetch} />);
  expect(fetch).toHaveBeenCalledTimes(1);
});

test("without a key nothing is requested", () => {
  const fetch = vi.fn(async (): Promise<Result<string>> => ({ status: "ok", data: "A" }));
  render(<Probe k={null} fetch={fetch} />);
  expect(state()).toBe("idle");
  expect(fetch).not.toHaveBeenCalled();
});

test("an error is shown and retry asks again", async () => {
  const fetch = vi
    .fn<() => Promise<Result<string>>>()
    .mockResolvedValueOnce({ status: "error", error: failure })
    .mockResolvedValueOnce({ status: "ok", data: "B" });
  render(<Probe k="a" fetch={fetch} />);
  await act(async () => {});
  expect(state()).toBe("error:calc");
  await act(async () => screen.getByRole("button", { name: "retry" }).click());
  expect(fetch).toHaveBeenCalledTimes(2);
  expect(state()).toBe("B");
});

test("when the key changes the old result is hidden and a late answer for the old key is dropped", async () => {
  const first = deferred<string>();
  const second = deferred<string>();
  const fetch = vi.fn<() => Promise<Result<string>>>().mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
  const { rerender } = render(<Probe k="a" fetch={fetch} />);
  rerender(<Probe k="b" fetch={fetch} />);
  expect(state()).toBe("loading");
  // Eski anahtarın yanıtı sonradan gelir: gösterilmez.
  await act(async () => first.resolve({ status: "ok", data: "A" }));
  expect(state()).toBe("loading");
  await act(async () => second.resolve({ status: "ok", data: "B" }));
  expect(state()).toBe("B");
  // Önceki anahtara dönülünce eski sonuç yeniden istenir (önbellek yok).
  const third = deferred<string>();
  fetch.mockReturnValueOnce(third.promise);
  rerender(<Probe k="a" fetch={fetch} />);
  expect(state()).toBe("loading");
  await act(async () => third.resolve({ status: "ok", data: "A2" }));
  expect(state()).toBe("A2");
});

test("StrictMode's second effect run does not call the command again", async () => {
  const d = deferred<string>();
  const fetch = vi.fn(() => d.promise);
  render(
    <StrictMode>
      <Probe k="a" fetch={fetch} />
    </StrictMode>,
  );
  expect(fetch).toHaveBeenCalledTimes(1);
  await act(async () => d.resolve({ status: "ok", data: "A" }));
  expect(state()).toBe("A");
  // Yeniden deneme yeni bir istektir.
  const again = deferred<string>();
  fetch.mockReturnValueOnce(again.promise);
  await act(async () => screen.getByRole("button", { name: "retry" }).click());
  expect(fetch).toHaveBeenCalledTimes(2);
  await act(async () => again.resolve({ status: "ok", data: "A2" }));
  expect(state()).toBe("A2");
});
```

```diff
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index 8e858b7..0286383 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -1,11 +1,14 @@
+import { GRID_COLUMNS, minSize } from "./grid";
 import {
   BUILTIN_LAYOUT_IDS,
   builtinLayout,
+  builtinLayouts,
   defaultLayoutState,
   isKnownPanelKind,
   LAYOUT_SCHEMA_VERSION,
   MAX_LAYOUTS,
   MAX_PANELS,
+  MODULE_PANELS,
   PANEL_KINDS,
   parseLayoutState,
   serializeLayoutState,
@@ -21,7 +24,7 @@ function overlaps(a: Panel, b: Panel) {
   return a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
 }
 
-test("catalog has the seven v1 panel kinds", () => {
+test("catalog has the seven v1 panel kinds and the three foundation module panels", () => {
   expect(PANEL_KINDS).toEqual([
     "chart",
     "graha_table",
@@ -30,11 +33,27 @@ test("catalog has the seven v1 panel kinds", () => {
     "transit_table",
     "notes",
     "birth_info",
+    "panchang",
+    "ashtakavarga",
+    "drishti",
   ]);
   expect(isKnownPanelKind("chart")).toBe(true);
+  expect(isKnownPanelKind("drishti")).toBe(true);
   expect(isKnownPanelKind("yoga_list")).toBe(false);
 });
 
+test("module panels are known kinds, listed once, with a size the grid accepts", () => {
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti"]);
+  for (const m of MODULE_PANELS) {
+    expect(isKnownPanelKind(m.kind)).toBe(true);
+    expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
+    expect(m.h).toBeGreaterThanOrEqual(minSize(m.kind).h);
+    expect(m.w).toBeLessThanOrEqual(GRID_COLUMNS);
+  }
+  // Hazır düzenler değişmedi: modül panelleri paletten eklenir.
+  for (const l of builtinLayouts()) expect(l.panels.some((p) => MODULE_PANELS.some((m) => m.kind === p.kind))).toBe(false);
+});
+
 test("default state has the three builtin layouts with interview active", () => {
   const s = defaultLayoutState();
   expect(s.schema_version).toBe(LAYOUT_SCHEMA_VERSION);
diff --git a/app/ui/layout/reducer.test.ts b/app/ui/layout/reducer.test.ts
index 769aac2..a90903d 100644
--- a/app/ui/layout/reducer.test.ts
+++ b/app/ui/layout/reducer.test.ts
@@ -1,5 +1,5 @@
 import { hasOverlap } from "./grid";
-import { BUILTIN_LAYOUT_IDS, builtinLayout, defaultLayoutState, MAX_LAYOUTS, type LayoutState } from "./model";
+import { BUILTIN_LAYOUT_IDS, builtinLayout, defaultLayoutState, MAX_LAYOUTS, MAX_PANELS, type LayoutState } from "./model";
 import { activeLayout, layoutReducer, newId, NOTES_PANEL, normalizeState } from "./reducer";
 
 const { interview, research, transit } = BUILTIN_LAYOUT_IDS;
@@ -104,6 +104,41 @@ test("toggleNotes removes notes when present and adds a 4-column panel on the ri
   expect(panels.find((p) => p.id === "d9")!.y).toBe(NOTES_PANEL.h);
 });
 
+test("togglePanel adds a module panel below the layout without moving the others and removes it again", () => {
+  const before = activeLayout(defaultLayoutState()).panels;
+  const bottom = Math.max(...before.map((p) => p.y + p.h));
+  let s = layoutReducer(defaultLayoutState(), { type: "togglePanel", kind: "drishti", panelId: "dr1" });
+  let panels = activeLayout(s).panels;
+  expect(panels.slice(0, before.length)).toEqual(before);
+  expect(panels.at(-1)).toEqual({ id: "dr1", kind: "drishti", x: 0, y: bottom, w: 6, h: 10, settings: {} });
+  expect(hasOverlap(panels)).toBe(false);
+  // İkinci panel öncekinin altına gelir.
+  s = layoutReducer(s, { type: "togglePanel", kind: "panchang", panelId: "pa1" });
+  panels = activeLayout(s).panels;
+  expect(panels.at(-1)).toMatchObject({ id: "pa1", kind: "panchang", x: 0, y: bottom + 10, w: 4, h: 9 });
+  expect(hasOverlap(panels)).toBe(false);
+  // Aynı tür yeniden istenince kaldırılır; diğer paneller yerinde durur.
+  s = layoutReducer(s, { type: "togglePanel", kind: "drishti", panelId: "dr2" });
+  panels = activeLayout(s).panels;
+  expect(panels.some((p) => p.kind === "drishti")).toBe(false);
+  expect(panels.find((p) => p.id === "pa1")).toMatchObject({ x: 0, y: bottom + 10 });
+  // Yalnızca etkin düzen değişir.
+  expect(s.layouts[1]).toEqual(defaultLayoutState().layouts[1]);
+});
+
+test("togglePanel adds to an empty layout at the top and does nothing at the panel limit or on a duplicate id", () => {
+  const empty = layoutReducer(defaultLayoutState(), { type: "updatePanels", panels: [] });
+  const added = layoutReducer(empty, { type: "togglePanel", kind: "ashtakavarga", panelId: "av" });
+  expect(activeLayout(added).panels).toEqual([{ id: "av", kind: "ashtakavarga", x: 0, y: 0, w: 6, h: 11, settings: {} }]);
+  const full = layoutReducer(defaultLayoutState(), {
+    type: "updatePanels",
+    panels: Array.from({ length: MAX_PANELS }, (_, i) => ({ id: `n${i}`, kind: "notes", x: 0, y: i * 4, w: 3, h: 4, settings: {} })),
+  });
+  expect(layoutReducer(full, { type: "togglePanel", kind: "drishti", panelId: "dr" })).toEqual(full);
+  const state = defaultLayoutState();
+  expect(layoutReducer(state, { type: "togglePanel", kind: "drishti", panelId: "d1" })).toEqual(state);
+});
+
 test("normalizeState fixes stored geometry", () => {
   const s = defaultLayoutState();
   s.layouts[0].panels = [
diff --git a/app/ui/palette/items.test.ts b/app/ui/palette/items.test.ts
index 0d4f932..67ade41 100644
--- a/app/ui/palette/items.test.ts
+++ b/app/ui/palette/items.test.ts
@@ -20,8 +20,10 @@ test("local items keep command, chart, layout, varga order and are filtered by t
     layoutLabel: (l) => l.id,
     vargas: [{ varga: "D9", label: "D9 Navamsa" }],
     commands: [{ command: "newChart", label: "Yeni harita" }],
+    panels: [{ kind: "drishti", label: "Bakışlar panelini aç/kapat" }],
   };
-  expect(localItems(src, "", "tr").map((i) => i.kind)).toEqual(["command", "chart", "layout", "layout", "layout", "varga"]);
+  expect(localItems(src, "", "tr").map((i) => i.kind)).toEqual(["command", "panel", "chart", "layout", "layout", "layout", "varga"]);
+  expect(localItems(src, "bakış", "tr")).toEqual([{ kind: "panel", key: "panel:drishti", label: "Bakışlar panelini aç/kapat", panelKind: "drishti" }]);
   expect(localItems(src, "harita", "tr").map((i) => i.key)).toEqual(["command:newChart", `chart:${chart.id}`]);
   expect(localItems(src, "navamsa", "tr")).toEqual([{ kind: "varga", key: "varga:D9", label: "D9 Navamsa", varga: "D9" }]);
 });
diff --git a/app/ui/shell/focus.test.ts b/app/ui/shell/focus.test.ts
index 238faae..bf12b1c 100644
--- a/app/ui/shell/focus.test.ts
+++ b/app/ui/shell/focus.test.ts
@@ -1,4 +1,4 @@
-import { leaveNotes, leaveSidebar, MAIN_ID } from "./focus";
+import { leaveNotes, leavePanel, leaveSidebar, MAIN_ID } from "./focus";
 
 afterEach(() => (document.body.innerHTML = ""));
 
@@ -20,3 +20,19 @@ test("leaving the sidebar moves focus to its toggle", () => {
   leaveSidebar();
   expect(document.activeElement).toBe(document.querySelector("button"));
 });
+
+test("leaving a module panel moves focus to main only when focus is inside a panel of that kind", () => {
+  document.body.innerHTML = `<main id="${MAIN_ID}" tabindex="-1"><section class="panel" data-kind="drishti" tabindex="0"><button>Gü</button></section><section class="panel" data-kind="panchang" tabindex="0"></section></main>`;
+  const main = document.getElementById(MAIN_ID);
+  const inside = document.querySelector("button")!;
+  inside.focus();
+  leavePanel("panchang");
+  expect(document.activeElement).toBe(inside);
+  leavePanel("drishti");
+  expect(document.activeElement).toBe(main);
+  // Panelin kendisi odaktayken de.
+  const frame = document.querySelector<HTMLElement>('[data-kind="panchang"]')!;
+  frame.focus();
+  leavePanel("panchang");
+  expect(document.activeElement).toBe(main);
+});
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index 5a0b372..e27ccd7 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -25,7 +25,7 @@ const chartPanel = (settings: Record<string, unknown> = { varga: "D1", style: "n
 });
 const tablePanel: Panel = { id: "t", kind: "graha_table", x: 6, y: 0, w: 6, h: 8, settings: {} };
 
-test("registry covers the seven kinds and every known kind is implemented", () => {
+test("registry covers every catalog kind", () => {
   expect(Object.keys(PANEL_REGISTRY).sort()).toEqual([...PANEL_KINDS].sort());
   expect(PANEL_REGISTRY.chart.component).toBe(ChartPanel);
   expect(PANEL_REGISTRY.graha_table.component).toBe(GrahaTablePanel);
diff --git a/app/ui/App.test.tsx b/app/ui/App.test.tsx
index 2db0d1d..2f79cf5 100644
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -175,6 +175,52 @@ test("closing the notes panel from inside it moves focus to main, by shortcut an
   expect(document.activeElement).toBe(main);
 });
 
+test("the palette adds a module panel below the layout and focuses it; again removes it and focus goes to main", async () => {
+  renderApp();
+  const panel = () => screen.queryByRole("region", { name: "Bakışlar" });
+  expect(panel()).toBeNull();
+  const run = async () => {
+    fireEvent.keyDown(window, { key: "k", metaKey: true });
+    const palette = screen.getByRole("dialog", { name: "Komut paleti" });
+    fireEvent.change(within(palette).getByRole("combobox"), { target: { value: "bakış" } });
+    const option = await within(palette).findByRole("option", { name: /Bakışlar panelini aç\/kapat/ });
+    expect(option.firstChild).toHaveTextContent("Panel");
+    fireEvent.click(option);
+    expect(screen.queryByRole("dialog")).toBeNull();
+  };
+  await run();
+  // Görüşme düzeninin altı (satır 18): 6 × 10, solda.
+  expect(panel()).toHaveAttribute("data-y", "18");
+  expect(panel()).toHaveAttribute("data-x", "0");
+  expect(panel()).toHaveAttribute("data-w", "6");
+  await waitFor(() => expect(panel()).toHaveFocus());
+  // Diğer paneller yerinde.
+  expect(screen.getByRole("region", { name: "Harita D1" })).toHaveAttribute("data-y", "0");
+  await run();
+  expect(panel()).toBeNull();
+  expect(document.activeElement).toBe(screen.getByRole("main"));
+});
+
+test("module panel commands are offered only with an open chart and are ignored in print view", async () => {
+  const { unmount } = renderApp({ opened: null });
+  fireEvent.keyDown(window, { key: "k", metaKey: true });
+  let palette = screen.getByRole("dialog", { name: "Komut paleti" });
+  await waitFor(() => expect(within(palette).getAllByRole("option").length).toBeGreaterThan(0));
+  expect(within(palette).queryByRole("option", { name: /panelini aç\/kapat/ })).toBeNull();
+  unmount();
+  renderApp();
+  fireEvent.keyDown(window, { key: "p", metaKey: true });
+  printToolbar();
+  fireEvent.keyDown(window, { key: "k", metaKey: true });
+  palette = screen.getByRole("dialog", { name: "Komut paleti" });
+  fireEvent.change(within(palette).getByRole("combobox"), { target: { value: "panchang" } });
+  fireEvent.click(await within(palette).findByRole("option", { name: /Panchang panelini aç\/kapat/ }));
+  // Önizleme kapatılınca düzen değişmemiş olur.
+  fireEvent.click(screen.getByRole("button", { name: "Panoya dön" }));
+  expect(screen.getByRole("region", { name: "Harita D1" })).toBeInTheDocument();
+  expect(screen.queryByRole("region", { name: "Panchang" })).toBeNull();
+});
+
 test("Cmd+Shift+N works from a text field and is ignored on key repeat, during composition and in print view", () => {
   renderApp();
   const notes = () => screen.queryByRole("region", { name: "Notlar" });
```

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

Run: `pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "`
Expected: `Test Files  7 failed | 55 passed (62)` (yeni iki dosya ile `model`, `reducer`, `items`, `focus` ve `App` testleri düşer; `panels.test.tsx`'te yalnızca test adı değişti, geçer).

- [ ] **Step 3: Ortak girdi ve kanca**

**Yeni dosya:** `app/ui/modules/input.ts`

```ts
// İleri modül komutlarının ortak girdisi (ileri modüller spec'i K2). Değerler açık haritadan
// alınır; burada hesap yoktur (C4). Modül sonucu saklanmaz: panel açıkken istenir (K1).
import type { ChartInput, OpenedClient } from "../bindings";
import { timeUnknown } from "../session/birth";

/** Açık haritanın modül girdisi: hesaplanan haritanın anı, konumu ve ayarları. */
export function chartInput(opened: OpenedClient): ChartInput {
  const { chart } = opened;
  return { jd_ut: chart.jd_ut, lat: chart.geo_lat, lon: chart.geo_lon, settings: chart.meta.settings, time_known: !timeUnknown(opened) };
}

/**
 * Sonucun ait olduğu girdinin anahtarı. Başka harita açılınca, doğum verisi düzenlenince ya da
 * hesap ayarı değişince anahtar değişir ve sonuç yeniden istenir. Kişisel veridir: loglanmaz.
 */
export function inputKey(opened: OpenedClient): string {
  return JSON.stringify([opened.chart_record.id, chartInput(opened)]);
}
```

**Yeni dosya:** `app/ui/modules/useModule.ts`

```ts
// Bir modül komutunun sonucunu panel açıkken ister (ileri modüller spec'i K1). `key` sonucun
// ait olduğu girdidir (`inputKey`): değişince eski sonuç gösterilmez ve geç gelen eski yanıt
// atılır. Hata "yeniden dene" ile yeniden istenir. Paneller yükleme evresini
// `usePanelLoadPhase(loading)` ile çizer (UI spec §5).
import { useCallback, useEffect, useRef, useState } from "react";
import type { Result } from "../api";
import type { CommandError } from "../bindings";

export interface ModuleState<T> {
  data: T | null;
  error: CommandError | null;
  loading: boolean;
  retry(): void;
}

/** `key === null`: açık harita yok, komut çağrılmaz. */
export function useModule<T>(key: string | null, fetch: () => Promise<Result<T>>): ModuleState<T> {
  const [attempt, setAttempt] = useState(0);
  const [settled, setSettled] = useState<{ key: string; attempt: number; result: Result<T> } | null>(null);
  const latest = useRef(fetch);
  latest.current = fetch;
  // Süren istek: aynı girdi ve deneme için effect yeniden koşarsa (StrictMode'un geliştirme
  // sırasındaki ikinci koşusu) komut yeniden çağrılmaz, süren yanıt beklenir.
  const inflight = useRef<{ id: string; promise: Promise<Result<T>> } | null>(null);

  useEffect(() => {
    if (key === null) return;
    const id = `${attempt}:${key}`;
    if (inflight.current?.id !== id) inflight.current = { id, promise: latest.current() };
    let live = true;
    void inflight.current.promise.then((result) => {
      if (live) setSettled({ key, attempt, result });
    });
    return () => {
      live = false;
    };
  }, [key, attempt]);

  const retry = useCallback(() => setAttempt((n) => n + 1), []);
  const result = settled !== null && settled.key === key && settled.attempt === attempt ? settled.result : null;
  return {
    data: result?.status === "ok" ? result.data : null,
    error: result?.status === "error" ? result.error : null,
    loading: key !== null && result === null,
    retry,
  };
}
```

- [ ] **Step 4: Katalog, `togglePanel`, depo**

```diff
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index ece194c..0a89253 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -1,7 +1,7 @@
 // Pano düzeni veri modeli (spec §3.2). Kalıcı biçim `ViewState.panel_layout` ile aynıdır;
 // M4'teki store komutu gelene kadar localStorage'da tutulur (bkz. persistence.ts).
 
-/** v1 panel kataloğu (spec §3.3). */
+/** Panel kataloğu: v1 (UI spec §3.3) ve ileri modül panelleri (ileri modüller spec'i §4). */
 export const PANEL_KINDS = [
   "chart",
   "graha_table",
@@ -10,9 +10,23 @@ export const PANEL_KINDS = [
   "transit_table",
   "notes",
   "birth_info",
+  "panchang",
+  "ashtakavarga",
+  "drishti",
 ] as const;
 export type KnownPanelKind = (typeof PANEL_KINDS)[number];
 
+/**
+ * Paletten etkin düzene eklenip kaldırılan paneller ve eklenirken aldıkları boyut (Plan 14 D9).
+ * Sonraki alt projeler kendi panellerini buraya ekler.
+ */
+export const MODULE_PANELS = [
+  { kind: "panchang", w: 4, h: 9 },
+  { kind: "ashtakavarga", w: 6, h: 11 },
+  { kind: "drishti", w: 6, h: 10 },
+] as const satisfies readonly { kind: KnownPanelKind; w: number; h: number }[];
+export type ModulePanelKind = (typeof MODULE_PANELS)[number]["kind"];
+
 export type PanelSettings = Record<string, unknown>;
 
 /**
diff --git a/app/ui/layout/reducer.ts b/app/ui/layout/reducer.ts
index d4c58ae..10a2b61 100644
--- a/app/ui/layout/reducer.ts
+++ b/app/ui/layout/reducer.ts
@@ -1,6 +1,6 @@
 // Düzen durumu için saf reducer. Kimlikler eylemle gelir; reducer rastgelelik üretmez.
 import { movePanel, normalizePanels } from "./grid";
-import { builtinLayout, MAX_LAYOUTS, type Layout, type LayoutState, type Panel } from "./model";
+import { builtinLayout, MAX_LAYOUTS, MAX_PANELS, MODULE_PANELS, type Layout, type LayoutState, type ModulePanelKind, type Panel } from "./model";
 
 export type LayoutAction =
   | { type: "select"; id: string }
@@ -12,6 +12,7 @@ export type LayoutAction =
   | { type: "resetBuiltin"; id: string }
   | { type: "updatePanels"; panels: Panel[] }
   | { type: "toggleNotes"; panelId: string }
+  | { type: "togglePanel"; kind: ModulePanelKind; panelId: string }
   | { type: "setPanelSettings"; panelId: string; patch: Record<string, unknown> };
 
 /** Notlar paneli düzende yoksa sağa 4 sütun genişliğinde eklenir (spec §3.2). */
@@ -101,5 +102,18 @@ export function layoutReducer(state: LayoutState, action: LayoutAction): LayoutS
         // Notlar sağ üste sabitlenir; çakışan paneller aşağı itilir.
         return { ...l, panels: movePanel([...l.panels, notes], notes.id, NOTES_PANEL.x, NOTES_PANEL.y) };
       });
+    case "togglePanel":
+      return mapLayout(state, state.active_layout_id, (l) => {
+        if (l.panels.some((p) => p.kind === action.kind)) {
+          // Kaldırırken diğer paneller yerinde durur (notlar gibi).
+          return { ...l, panels: l.panels.filter((p) => p.kind !== action.kind) };
+        }
+        const size = MODULE_PANELS.find((m) => m.kind === action.kind);
+        // Sınırda panel eklenmez (Rust fazlasını okumaz).
+        if (!size || l.panels.length >= MAX_PANELS || l.panels.some((p) => p.id === action.panelId)) return l;
+        // Yeni panel düzenin altına, sola eklenir: hiçbir paneli yerinden oynatmaz.
+        const y = Math.max(0, ...l.panels.map((p) => p.y + p.h));
+        return { ...l, panels: [...l.panels, { id: action.panelId, kind: action.kind, x: 0, y, w: size.w, h: size.h, settings: {} }] };
+      });
   }
 }
diff --git a/app/ui/view/store.tsx b/app/ui/view/store.tsx
index 0970be6..c961832 100644
--- a/app/ui/view/store.tsx
+++ b/app/ui/view/store.tsx
@@ -4,7 +4,7 @@
 // dasha yoluna `useViewStore` ile ulaşır; birbirini çağırmaz.
 import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type Dispatch, type ReactNode } from "react";
 import type { Graha, ViewState } from "../bindings";
-import type { Layout, LayoutState, Panel } from "../layout/model";
+import type { Layout, LayoutState, ModulePanelKind, Panel } from "../layout/model";
 import { activeLayout, newId, normalizeState, type LayoutAction } from "../layout/reducer";
 import { toViewState, type ViewModel } from "./model";
 import { matchShortcut } from "../shortcuts";
@@ -25,6 +25,8 @@ export interface LayoutStore {
   resetBuiltin(id: string): void;
   updatePanels(panels: Panel[]): void;
   toggleNotes(): void;
+  /** İleri modül panelini etkin düzene ekler; düzende varsa kaldırır. */
+  togglePanel(kind: ModulePanelKind): void;
   /** Etkin düzendeki bir panelin ayarlarına `patch`'i ekler (varga, biçim, sütunlar). */
   setPanelSettings(panelId: string, patch: Record<string, unknown>): void;
 }
@@ -138,6 +140,7 @@ export function ViewStateProvider({
       resetBuiltin: (id) => layoutDispatch({ type: "resetBuiltin", id }),
       updatePanels: (panels) => layoutDispatch({ type: "updatePanels", panels }),
       toggleNotes: () => layoutDispatch({ type: "toggleNotes", panelId: newId("notes") }),
+      togglePanel: (kind) => layoutDispatch({ type: "togglePanel", kind, panelId: newId(kind) }),
       setPanelSettings: (panelId, patch) => layoutDispatch({ type: "setPanelSettings", panelId, patch }),
     };
     const view: ViewStore = {
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index 8f12b3c..f5d4158 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -18,6 +18,10 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   transit_table: { titleKey: "panel.transit_table", component: TransitTablePanel },
   notes: { titleKey: "panel.notes", component: NotesPanel },
   birth_info: { titleKey: "panel.birth_info", component: BirthInfoPanel },
+  // İleri modül panelleri (Plan 14): içerikleri Task 6–8'de gelir.
+  panchang: { titleKey: "panel.panchang", component: UnavailablePanel },
+  ashtakavarga: { titleKey: "panel.ashtakavarga", component: UnavailablePanel },
+  drishti: { titleKey: "panel.drishti", component: UnavailablePanel },
 };
 
 const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };
```

- [ ] **Step 5: Palet, kabuk, odak ve metinler**

```diff
diff --git a/app/ui/palette/items.ts b/app/ui/palette/items.ts
index 20030bc..984cd8f 100644
--- a/app/ui/palette/items.ts
+++ b/app/ui/palette/items.ts
@@ -1,8 +1,8 @@
-// Komut paletinin öğeleri (UI spec §4: danışan, harita, varga, düzen ve komut tek arama
+// Komut paletinin öğeleri (UI spec §4: danışan, harita, varga, düzen, panel ve komut tek arama
 // kutusunda). Danışanlar Rust'ta aranır (`search_clients`); diğer öğeler arayüzün kendi
 // listeleridir ve burada metinle süzülür. Süzme bir dizgi karşılaştırmasıdır, hesap değildir.
 import type { ChartRecord, SearchHit, Varga } from "../bindings";
-import type { Layout } from "../layout/model";
+import type { Layout, ModulePanelKind } from "../layout/model";
 
 export type PaletteCommand = "newChart" | "addChart" | "editChart" | "settings" | "print" | "toggleNotes" | "sidebar" | "help" | "about";
 
@@ -11,7 +11,8 @@ export type PaletteItem =
   | { kind: "chart"; key: string; label: string; detail: string; clientId: number; chartId: number }
   | { kind: "layout"; key: string; label: string; layoutId: string }
   | { kind: "varga"; key: string; label: string; varga: Varga }
-  | { kind: "command"; key: string; label: string; command: PaletteCommand };
+  | { kind: "command"; key: string; label: string; command: PaletteCommand }
+  | { kind: "panel"; key: string; label: string; panelKind: ModulePanelKind };
 
 /** Büyük/küçük harf ve aksan duyarsız karşılaştırma anahtarı ("Navāmsa" → "navamsa"). */
 export function foldText(text: string, locale: string): string {
@@ -37,12 +38,15 @@ export interface LocalSources {
   /** Etkin düzende harita paneli yoksa varga öğesi gösterilmez. */
   vargas: readonly { varga: Varga; label: string }[];
   commands: readonly { command: PaletteCommand; label: string }[];
+  /** Etkin düzene eklenip kaldırılabilen paneller (Plan 14 D9). */
+  panels: readonly { kind: ModulePanelKind; label: string }[];
 }
 
-/** Arayüzün kendi öğeleri, sorguyla süzülmüş: komutlar, haritalar, düzenler, vargalar. */
+/** Arayüzün kendi öğeleri, sorguyla süzülmüş: komutlar, paneller, haritalar, düzenler, vargalar. */
 export function localItems(src: LocalSources, query: string, locale: string): PaletteItem[] {
   const out: PaletteItem[] = [
     ...src.commands.map((c): PaletteItem => ({ kind: "command", key: `command:${c.command}`, label: c.label, command: c.command })),
+    ...src.panels.map((p): PaletteItem => ({ kind: "panel", key: `panel:${p.kind}`, label: p.label, panelKind: p.kind })),
     ...src.charts.map(
       (c): PaletteItem => ({ kind: "chart", key: `chart:${c.id}`, label: src.chartLabel(c), detail: c.birth.date, clientId: c.client_id, chartId: c.id }),
     ),
diff --git a/app/ui/palette/Palette.tsx b/app/ui/palette/Palette.tsx
index 41cec40..c2f16a2 100644
--- a/app/ui/palette/Palette.tsx
+++ b/app/ui/palette/Palette.tsx
@@ -11,6 +11,7 @@ import { api } from "../api";
 import type { ChartRecord, CommandError, SearchHit } from "../bindings";
 import { VARGA_NAMES, VARGAS } from "../chart/adapter";
 import { errorMessage } from "../i18n/errors";
+import { MODULE_PANELS } from "../layout/model";
 import { useSession } from "../session/store";
 import { Dialog } from "../shell/Dialog";
 import { layoutLabel } from "../shell/LayoutPicker";
@@ -111,6 +112,8 @@ export function Palette({ onClose, onRun }: PaletteProps) {
         layoutLabel: (l) => layoutLabel(t, l),
         vargas: opened && hasChartPanel ? VARGAS.map((v) => ({ varga: v, label: `${v} ${VARGA_NAMES[v]}` })) : [],
         commands,
+        // Paneller de notlar gibi yalnızca harita açıkken panoda görünür.
+        panels: opened ? MODULE_PANELS.map((m) => ({ kind: m.kind, label: t("palette.togglePanel", { name: t(`panel.${m.kind}`) }) })) : [],
       },
       query,
       i18n.language,
diff --git a/app/ui/shell/focus.ts b/app/ui/shell/focus.ts
index 18af667..0a6f986 100644
--- a/app/ui/shell/focus.ts
+++ b/app/ui/shell/focus.ts
@@ -12,6 +12,12 @@ export function leaveSidebar() {
   }
 }
 
+/** Bir modül paneli düzenden kaldırılmadan önce: odak o paneldeyse ana bölgeye geçer. */
+export function leavePanel(kind: string) {
+  const panel = document.activeElement?.closest<HTMLElement>(".panel");
+  if (panel?.dataset.kind === kind) document.getElementById(MAIN_ID)?.focus();
+}
+
 /** Notlar paneli kapanmadan önce (`⌘⇧N`): odak paneldeyse ana bölgeye geçer. */
 export function leaveNotes() {
   if (document.activeElement?.closest(".notes-panel")) document.getElementById(MAIN_ID)?.focus();
diff --git a/app/ui/shell/Shell.tsx b/app/ui/shell/Shell.tsx
index 107d983..25312ca 100644
--- a/app/ui/shell/Shell.tsx
+++ b/app/ui/shell/Shell.tsx
@@ -6,6 +6,7 @@ import { api } from "../api";
 import type { CommandError } from "../bindings";
 import { BirthForm } from "../birth/BirthForm";
 import { Sidebar } from "../clients/Sidebar";
+import type { ModulePanelKind } from "../layout/model";
 import type { PaletteItem } from "../palette/items";
 import { Palette } from "../palette/Palette";
 import { Dashboard } from "../dashboard/Dashboard";
@@ -14,7 +15,7 @@ import { matchShortcut } from "../shortcuts";
 import { useLayoutStore } from "../view/store";
 import { applyTheme, loadThemePref, saveThemePref, type ThemePref } from "../theme/theme";
 import { EmptyState } from "./EmptyState";
-import { leaveNotes, leaveSidebar, MAIN_ID } from "./focus";
+import { leaveNotes, leavePanel, leaveSidebar, MAIN_ID } from "./focus";
 import { HelpDialog } from "./HelpDialog";
 import { NoticeBar, useNotices } from "./notices";
 import { loadSidebarOpen, saveSidebarOpen } from "./sidebarPref";
@@ -60,6 +61,23 @@ export function Shell() {
     const r = await api.printWindow();
     if (r.status === "error") setPrintError(r.error);
   };
+  // Paletten eklenen panel düzenin altına gelir; çizildikten sonra odak ona taşınır (görünür
+  // alana da kaydırır). Kaldırmada odak paneldeyse ana bölgeye geçer.
+  const [addedPanel, setAddedPanel] = useState<ModulePanelKind | null>(null);
+  useEffect(() => {
+    if (addedPanel === null) return;
+    document.querySelector<HTMLElement>(`.panel[data-kind="${addedPanel}"]`)?.focus();
+    setAddedPanel(null);
+  }, [addedPanel]);
+  const toggleModulePanel = (kind: ModulePanelKind) => {
+    if (layout.active.panels.some((p) => p.kind === kind)) {
+      leavePanel(kind);
+      layout.togglePanel(kind);
+    } else {
+      layout.togglePanel(kind);
+      setAddedPanel(kind);
+    }
+  };
   const { push } = useNotices();
   // Açılışta güncelleme denetimi yalnızca kullanıcı tercihi açtıysa yapılır (Plan 13 D1) ve
   // bir kez: StrictMode'un ikinci effect koşusu yeni istek göndermez.
@@ -141,6 +159,9 @@ export function Shell() {
         const panel = layout.active.panels.find((p) => p.kind === "chart");
         return panel && layout.setPanelSettings(panel.id, { varga: item.varga });
       }
+      case "panel":
+        // Notlar gibi: baskı görünümünde (pano yokken) yok sayılır.
+        return printView ? undefined : toggleModulePanel(item.panelKind);
       case "command":
         if (item.command === "newChart" || item.command === "addChart" || item.command === "editChart") {
           return setForm(item.command === "newChart" ? "new" : item.command === "addChart" ? "add" : "edit");
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 371c5f5..cbb8c4e 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -55,6 +55,9 @@ export const tr = {
     transit_table: "Transit tablosu",
     notes: "Notlar",
     birth_info: "Doğum bilgisi",
+    panchang: "Panchang",
+    ashtakavarga: "Ashtakavarga",
+    drishti: "Bakışlar",
     unknown: "Bilinmeyen panel",
     loading: "Yükleniyor",
     unavailable: "Bu panel sonraki bir sürümde gelecek.",
@@ -381,7 +384,9 @@ export const tr = {
       layout: "Düzen",
       varga: "Varga",
       command: "Komut",
+      panel: "Panel",
     },
+    togglePanel: "{{name}} panelini aç/kapat",
     command: {
       newChart: "Yeni danışan ve harita",
       addChart: "Açık danışana harita ekle",
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index af15dc4..4c573f5 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -56,6 +56,9 @@ export const en: Messages = {
     transit_table: "Transit table",
     notes: "Notes",
     birth_info: "Birth details",
+    panchang: "Panchang",
+    ashtakavarga: "Ashtakavarga",
+    drishti: "Aspects",
     unknown: "Unknown panel",
     loading: "Loading",
     unavailable: "This panel arrives in a later version.",
@@ -382,7 +385,9 @@ export const en: Messages = {
       layout: "Layout",
       varga: "Varga",
       command: "Command",
+      panel: "Panel",
     },
+    togglePanel: "Show or hide the {{name}} panel",
     command: {
       newChart: "New client and chart",
       addChart: "Add a chart to the open client",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 3f48c61..abbdf7d 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -56,6 +56,9 @@ export const fr: Messages = {
     transit_table: "Tableau des transits",
     notes: "Notes",
     birth_info: "Données de naissance",
+    panchang: "Panchang",
+    ashtakavarga: "Ashtakavarga",
+    drishti: "Aspects",
     unknown: "Panneau inconnu",
     loading: "Chargement",
     unavailable: "Ce panneau arrivera dans une version ultérieure.",
@@ -382,7 +385,9 @@ export const fr: Messages = {
       layout: "Disposition",
       varga: "Varga",
       command: "Commande",
+      panel: "Panneau",
     },
+    togglePanel: "Afficher ou masquer le panneau {{name}}",
     command: {
       newChart: "Nouveau client et carte",
       addChart: "Ajouter une carte au client ouvert",
```

- [ ] **Step 6: Kapılar**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | tail -1
```
Expected: `Test Files  62 passed (62)`, `Tests  541 passed (541)` (+14: `model` 1, `reducer` 2, `focus` 1, `input` 3, `useModule` 5, `App` 2); `✓ built`.

- [ ] **Step 7: Commit**

```bash
git add app/ui/modules app/ui/layout app/ui/view/store.tsx app/ui/palette app/ui/shell app/ui/panels app/ui/i18n app/ui/App.test.tsx
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): module panel kinds, palette commands to add or remove them, shared module input and fetch hook"
```
Expected: iki `grep` çıktısız.

---

### Task 6: Panchang paneli

Spec §4 "Arayüz", `panchang` satırı; D10. Panel `Chart.panchang`'ı çizer: tithi (paksha ile), vara, nakshatra ve pada, nitya yoga, karana, her birinin bitiş saati, gün doğumu ve batımı. Saatler doğumun çözülmüş ofsetindedir. Klasik adlar üç dilde aynı transliterasyonla yazılır; yalnızca günün çevirisi ve başlıklar dile göre değişir (K11). Durumlar: harita yok; dolu; saat bilinmiyor (uyarı ve "≈"); gün doğumu yok (çizgi ve mevcut uyarı metni). Yükleme ve hata durumu yoktur: veri açık haritayla birlikte gelir.

**Files:**
- Create: `app/ui/panels/PanchangPanel.tsx`, `app/ui/panels/PanchangPanel.test.tsx`, `app/ui/panels/Approx.tsx`
- Modify: `app/ui/panels/{format,labels,registry,GrahaTablePanel}.ts(x)`, `app/ui/panels/panels.css`, `app/ui/i18n/{tr,en,fr}.ts`
- Test: `app/ui/panels/PanchangPanel.test.tsx`, `app/ui/panels/format.test.ts`, `app/ui/panels/panels.test.tsx`

**Interfaces:**
- Consumes: `Panchang` (bağlamalar), `useSession`, `useHighlight`, `timeUnknown`, `formatFixedOffset` (`birth/model.ts`), `nakshatraName`, `PanelProps`, i18n `badge.SunriseUnavailable`, `badgeHint.SunriseUnavailable`, `table.approx`.
- Produces:
  - `panels/Approx.tsx`: `Approx({ on, label, children })` (GrahaTablePanel'deki yerel bileşen buraya taşınır; davranışı aynıdır)
  - `panels/format.ts`: `formatJdMinute(jd: number | null, offsetSecs: number): string` → `"YYYY-AA-GG SS:DD"` ya da `"—"`
  - `panels/labels.ts`: `TITHI_NAMES`, `tithiInPaksha(no)`, `tithiName(no)`, `NITYA_YOGA_NAMES`, `nityaYogaName(n)`, `VARA_NAMES`
  - `panels/PanchangPanel.tsx`: `PanchangPanel`, `panchangRows(t, p, offsetSecs): PanchangRow[]`, `interface PanchangRow`
  - CSS: `.module-table`, `.module-panel`, `.module-toolbar` (Task 7 ve 8 de kullanır)
  - i18n: `panchang.*`

- [ ] **Step 1: Başarısız testleri yaz**

Beklenen saatler fikstürdeki JD'lerden elle çevrilmiştir (örnek: tithi bitişi JD 2451545,8262355 = 2000-01-02 07:49:46 UT = 13:19:46 IST → `2000-01-02 13:19`).

**Yeni dosya:** `app/ui/panels/PanchangPanel.test.tsx`

```tsx
import { fireEvent, screen } from "@testing-library/react";
import { expect, test } from "vitest";
import { useHighlight } from "../highlight/store";
import { setLanguage } from "../i18n";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { nityaYogaName, NITYA_YOGA_NAMES, tithiInPaksha, tithiName, VARA_NAMES } from "./labels";
import { PanchangPanel } from "./PanchangPanel";

const panel: Panel = { id: "pa", kind: "panchang", x: 0, y: 0, w: 4, h: 9, settings: {} };

/** Ortak vurguyu dışarıdan süren düğmeler (başka bir panelin yerine). */
function Pins() {
  const h = useHighlight();
  return (
    <>
      <button onClick={() => h.togglePin({ type: "graha", id: "Moon" })}>pin moon</button>
      <button onClick={() => h.togglePin({ type: "graha", id: "Sun" })}>pin sun</button>
      <button onClick={() => h.togglePin({ type: "graha", id: "Mars" })}>pin mars</button>
    </>
  );
}

const cells = (key: string) => [...document.querySelector(`tr[data-row="${key}"]`)!.children].map((c) => c.textContent);
const highlighted = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.row);

afterEach(() => setLanguage("tr"));

test("five limbs with their end times and sunrise and sunset, in the birth place's time", () => {
  renderWithStores(<PanchangPanel panel={panel} />);
  // Delhi 2000-01-01 17:30 IST (+05:30); değerler tests/golden/new-delhi-2000-01-01.json ile aynı.
  expect(screen.getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Öğe", "Değer", "Bitiş"]);
  expect(cells("tithi")).toEqual(["Tithi", "Krishna 11 · Ekadashi", "2000-01-02 13:19"]);
  expect(cells("vara")).toEqual(["Vara", "Shanivara · Cumartesi", ""]);
  expect(cells("nakshatra")).toEqual(["Nakshatra", "Swati · pada 4", "2000-01-01 18:33"]);
  expect(cells("yoga")).toEqual(["Nitya yoga", "Dhriti", "2000-01-02 13:14"]);
  expect(cells("karana")).toEqual(["Karana", "Bava", "2000-01-02 00:09"]);
  expect(cells("sunrise")).toEqual(["Gün doğumu", "2000-01-01 07:17", ""]);
  expect(cells("sunset")).toEqual(["Gün batımı", "2000-01-01 17:30", ""]);
  expect(screen.getByRole("table", { name: "Doğum anının panchangı; saatler doğum yerinin saatiyle (IST +05:30)" })).toBeInTheDocument();
  expect(document.querySelector('[title="yaklaşık"]')).toBeNull();
  expect(screen.queryByText(/Gün doğumu yok/)).toBeNull();
});

test("without an open chart the panel says so", () => {
  renderWithStores(<PanchangPanel panel={panel} />, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
});

test("unknown birth time: a notice, and the time-bound values are marked approximate", () => {
  renderWithStores(<PanchangPanel panel={panel} />, { opened: unknownTimeOpened() });
  expect(screen.getByText("Doğum saati bilinmiyor: değerler öğlen 12:00 içindir ve yaklaşıktır (≈).")).toBeInTheDocument();
  for (const key of ["tithi", "vara", "nakshatra", "yoga", "karana"]) {
    const mark = document.querySelector(`tr[data-row="${key}"] [title="yaklaşık"]`);
    expect(mark, key).not.toBeNull();
    expect(mark).toHaveAttribute("aria-hidden", "true");
  }
  expect(cells("nakshatra")[1]).toBe("≈yaklaşık Swati · pada 4");
  // Gün doğumu ve batımı o günündür; saate bağlı değildir.
  for (const key of ["sunrise", "sunset"]) expect(document.querySelector(`tr[data-row="${key}"] [title="yaklaşık"]`), key).toBeNull();
});

test("no sunrise at the birth place: dashes and the existing warning text", () => {
  const opened = fixture().opened;
  opened.chart.panchang.sunrise_jd_ut = null;
  opened.chart.panchang.sunset_jd_ut = null;
  renderWithStores(<PanchangPanel panel={panel} />, { opened });
  expect(cells("sunrise")).toEqual(["Gün doğumu", "—", ""]);
  expect(cells("sunset")).toEqual(["Gün batımı", "—", ""]);
  expect(screen.getByText("Gün doğumu yok")).toBeInTheDocument();
  expect(screen.getByText(/Doğum yerinde gün doğumu ya da batımı yok\./)).toBeInTheDocument();
  // Beş uzuv yine gösterilir.
  expect(cells("tithi")[1]).toBe("Krishna 11 · Ekadashi");
});

test("shared highlight: the Moon marks its rows, the Sun its rows, another graha none", () => {
  renderWithStores(
    <>
      <Pins />
      <PanchangPanel panel={panel} />
    </>,
  );
  expect(highlighted()).toEqual([]);
  fireEvent.click(screen.getByRole("button", { name: "pin moon" }));
  expect(highlighted()).toEqual(["tithi", "nakshatra", "yoga", "karana"]);
  fireEvent.click(screen.getByRole("button", { name: "pin sun" }));
  expect(highlighted()).toEqual(["tithi", "yoga", "karana", "sunrise", "sunset"]);
  fireEvent.click(screen.getByRole("button", { name: "pin mars" }));
  expect(highlighted()).toEqual([]);
});

test("classical names stay the same in every language; only the weekday gloss and headers change", async () => {
  await setLanguage("en");
  const { unmount } = renderWithStores(<PanchangPanel panel={panel} />);
  expect(cells("tithi")).toEqual(["Tithi", "Krishna 11 · Ekadashi", "2000-01-02 13:19"]);
  expect(cells("vara")).toEqual(["Vara", "Shanivara · Saturday", ""]);
  expect(cells("sunrise")[0]).toBe("Sunrise");
  unmount();
  await setLanguage("fr");
  renderWithStores(<PanchangPanel panel={panel} />);
  expect(cells("vara")).toEqual(["Vara", "Shanivara · samedi", ""]);
  expect(cells("yoga")).toEqual(["Nitya yoga", "Dhriti", "2000-01-02 13:14"]);
});

test("tithi, yoga and vara names", () => {
  expect([1, 14, 15, 16, 29, 30].map(tithiInPaksha)).toEqual([1, 14, 15, 1, 14, 15]);
  expect([1, 11, 14, 15, 16, 26, 30].map(tithiName)).toEqual(["Pratipada", "Ekadashi", "Chaturdashi", "Purnima", "Pratipada", "Ekadashi", "Amavasya"]);
  expect([0, 31, 1.5].map(tithiName)).toEqual(["0", "31", "1.5"]);
  expect(NITYA_YOGA_NAMES).toHaveLength(27);
  expect(new Set(NITYA_YOGA_NAMES).size).toBe(27);
  expect([1, 8, 27, 28].map(nityaYogaName)).toEqual(["Vishkambha", "Dhriti", "Vaidhriti", "28"]);
  expect(Object.keys(VARA_NAMES)).toEqual(["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]);
});
```

```diff
diff --git a/app/ui/panels/format.test.ts b/app/ui/panels/format.test.ts
index 20c1260..f197cf4 100644
--- a/app/ui/panels/format.test.ts
+++ b/app/ui/panels/format.test.ts
@@ -1,4 +1,4 @@
-import { formatDegree, formatSignedDegree, formatSpeed } from "./format";
+import { formatDegree, formatJdMinute, formatSignedDegree, formatSpeed } from "./format";
 
 test("formatDegree writes degrees and zero-padded arc minutes, rounding down", () => {
   expect(formatDegree(0)).toBe("0°00′");
@@ -14,3 +14,18 @@ test("signed degrees and speeds carry an explicit sign and a true minus", () =>
   expect(formatSpeed(12.021263924093429)).toBe("+12.021");
   expect(formatSpeed(-0.05)).toBe("−0.050");
 });
+
+test("a julian day is written as local date and minute at a fixed offset, floored", () => {
+  // JD 2451545.0 = 2000-01-01 12:00 UT.
+  expect(formatJdMinute(2451545, 0)).toBe("2000-01-01 12:00");
+  expect(formatJdMinute(2451545, 19800)).toBe("2000-01-01 17:30");
+  expect(formatJdMinute(2451545, -18000)).toBe("2000-01-01 07:00");
+  // 59,9 saniye aşağı yuvarlanır; gün sınırı ofsetle geçilir.
+  expect(formatJdMinute(2451545 + 59.9 / 86400, 0)).toBe("2000-01-01 12:00");
+  expect(formatJdMinute(2451545 + 60 / 86400 + 1e-9, 0)).toBe("2000-01-01 12:01");
+  // 0,40001 gün = 9 sa 36 dk 0,9 sn → 21:36 UT → +05:30 ertesi gün 03:06.
+  expect(formatJdMinute(2451545.40001, 19800)).toBe("2000-01-02 03:06");
+  expect(formatJdMinute(null, 0)).toBe("—");
+  expect(formatJdMinute(Number.NaN, 0)).toBe("—");
+  expect(formatJdMinute(1e300, 0)).toBe("—");
+});
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index e27ccd7..f81a3fa 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -11,6 +11,7 @@ import { TransitTablePanel } from "../transit/TransitTablePanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel } from "./ChartPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
+import { PanchangPanel } from "./PanchangPanel";
 import { panelDefinition, PANEL_REGISTRY } from "./registry";
 import { UnavailablePanel } from "./UnavailablePanel";
 
@@ -34,6 +35,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.birth_info.component).toBe(BirthInfoPanel);
   expect(PANEL_REGISTRY.dasha_strip.component).toBe(DashaStripPanel);
   expect(PANEL_REGISTRY.transit_table.component).toBe(TransitTablePanel);
+  expect(PANEL_REGISTRY.panchang.component).toBe(PanchangPanel);
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
   const keys = new Set(Object.keys(tr.panel).map((k) => `panel.${k}`));
   for (const kind of [...PANEL_KINDS, "yoga_list"]) expect(keys.has(panelDefinition(kind).titleKey)).toBe(true);
```

Run: `pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "`
Expected: `Test Files  3 failed | 60 passed (63)`.

- [ ] **Step 2: Biçimleme, adlar ve `Approx`**

**Yeni dosya:** `app/ui/panels/Approx.tsx`

```tsx
import type { ReactNode } from "react";

/** Yaklaşık değer: "≈" işareti (fare için `title`) ve ekran okuyucu için bir kez okunan metin. */
export function Approx({ on, label, children }: { on: boolean; label: string; children: ReactNode }) {
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

```diff
diff --git a/app/ui/panels/GrahaTablePanel.tsx b/app/ui/panels/GrahaTablePanel.tsx
index bcf5b40..399b1c3 100644
--- a/app/ui/panels/GrahaTablePanel.tsx
+++ b/app/ui/panels/GrahaTablePanel.tsx
@@ -7,25 +7,12 @@ import { sameTarget, type HighlightTarget } from "../highlight/target";
 import { useSession } from "../session/store";
 import { useLayoutStore } from "../view/store";
 import { timeUnknown } from "../session/birth";
+import { Approx } from "./Approx";
 import { APPROX_COLUMNS, grahaCellText, lagnaCellText, NUMERIC_COLUMNS as NUMERIC } from "./grahaCells";
 import { GRAHA_COLUMNS, grahaColumns, toggleColumn, type GrahaColumn } from "./grahaColumns";
 import "./panels.css";
 import type { PanelProps } from "./types";
 
-/** Yaklaşık değer: "≈" işareti (fare için `title`) ve ekran okuyucu için bir kez okunan metin. */
-function Approx({ on, label, children }: { on: boolean; label: string; children: ReactNode }) {
-  if (!on) return <>{children}</>;
-  return (
-    <span>
-      <span aria-hidden="true" title={label}>
-        ≈
-      </span>
-      <span className="visually-hidden">{label} </span>
-      {children}
-    </span>
-  );
-}
-
 /** Hücre metni `grahaCells.ts`'ten; burada yalnızca erişilebilirlik sarmalayıcıları eklenir. */
 function grahaCell(t: TFunction, g: GrahaPosition, column: GrahaColumn, approx: boolean): ReactNode {
   const text = grahaCellText(t, g, column);
diff --git a/app/ui/panels/format.ts b/app/ui/panels/format.ts
index 3c201b5..0bad911 100644
--- a/app/ui/panels/format.ts
+++ b/app/ui/panels/format.ts
@@ -11,6 +11,25 @@ export function formatSignedDegree(deg: number): string {
   return `${deg < 0 ? "−" : "+"}${formatDegree(Math.abs(deg))}`;
 }
 
+const UNIX_EPOCH_JD = 2440587.5;
+const DAY_MS = 86_400_000;
+const MINUTE_MS = 60_000;
+
+/**
+ * JD (UT) → sabit ofsette "YYYY-AA-GG SS:DD" (dakika aşağı yuvarlanır). Yalnızca gösterim:
+ * "arayüz hesap yapmaz" kuralı astrolojik hesap içindir, takvim biçimlemesi değil.
+ */
+export function formatJdMinute(jd: number | null, offsetSecs: number): string {
+  if (jd === null || !Number.isFinite(jd)) return "—";
+  const ms = Math.floor(((jd - UNIX_EPOCH_JD) * DAY_MS + offsetSecs * 1000) / MINUTE_MS) * MINUTE_MS;
+  const d = new Date(ms);
+  if (Number.isNaN(d.getTime())) return "—";
+  const two = (n: number) => String(n).padStart(2, "0");
+  const y = d.getUTCFullYear();
+  const year = y < 0 ? `−${String(-y).padStart(4, "0")}` : String(y).padStart(4, "0");
+  return `${year}-${two(d.getUTCMonth() + 1)}-${two(d.getUTCDate())} ${two(d.getUTCHours())}:${two(d.getUTCMinutes())}`;
+}
+
 /** Günlük hız, derece/gün, üç ondalık: 12.0212 → "+12.021", -0.05 → "−0.050". */
 export function formatSpeed(degPerDay: number): string {
   return `${degPerDay < 0 ? "−" : "+"}${Math.abs(degPerDay).toFixed(3)}`;
diff --git a/app/ui/panels/labels.ts b/app/ui/panels/labels.ts
index b436075..e5c6b42 100644
--- a/app/ui/panels/labels.ts
+++ b/app/ui/panels/labels.ts
@@ -53,6 +53,83 @@ export const NAKSHATRA_NAMES = [
   "Revati",
 ] as const;
 
+/** Tithi adları, paksha içi sıra 1–14; 15. tithi Shukla'da Purnima, Krishna'da Amavasya'dır. */
+export const TITHI_NAMES = [
+  "Pratipada",
+  "Dwitiya",
+  "Tritiya",
+  "Chaturthi",
+  "Panchami",
+  "Shashthi",
+  "Saptami",
+  "Ashtami",
+  "Navami",
+  "Dashami",
+  "Ekadashi",
+  "Dwadashi",
+  "Trayodashi",
+  "Chaturdashi",
+] as const;
+
+/** Tithinin (1–30) paksha içi sırası, 1–15. */
+export function tithiInPaksha(no: number): number {
+  return ((no - 1) % 15) + 1;
+}
+
+/** Tithinin (1–30) adı; aralık dışıysa numaranın kendisi. */
+export function tithiName(no: number): string {
+  if (!Number.isInteger(no) || no < 1 || no > 30) return String(no);
+  if (no === 15) return "Purnima";
+  if (no === 30) return "Amavasya";
+  return TITHI_NAMES[tithiInPaksha(no) - 1];
+}
+
+/** Nitya yoga adları, 1 = Vishkambha … 27 = Vaidhriti. */
+export const NITYA_YOGA_NAMES = [
+  "Vishkambha",
+  "Priti",
+  "Ayushman",
+  "Saubhagya",
+  "Shobhana",
+  "Atiganda",
+  "Sukarma",
+  "Dhriti",
+  "Shula",
+  "Ganda",
+  "Vriddhi",
+  "Dhruva",
+  "Vyaghata",
+  "Harshana",
+  "Vajra",
+  "Siddhi",
+  "Vyatipata",
+  "Variyan",
+  "Parigha",
+  "Shiva",
+  "Siddha",
+  "Sadhya",
+  "Shubha",
+  "Shukla",
+  "Brahma",
+  "Indra",
+  "Vaidhriti",
+] as const;
+
+export function nityaYogaName(n: number): string {
+  return NITYA_YOGA_NAMES[n - 1] ?? String(n);
+}
+
+/** Vara adları (Sanskritçe terim; dile göre değişmez). Günün çevirisi i18n'dedir. */
+export const VARA_NAMES = {
+  Sunday: "Ravivara",
+  Monday: "Somavara",
+  Tuesday: "Mangalavara",
+  Wednesday: "Budhavara",
+  Thursday: "Guruvara",
+  Friday: "Shukravara",
+  Saturday: "Shanivara",
+} as const;
+
 /** 1 tabanlı nakshatra numarasının adı; aralık dışıysa numaranın kendisi. */
 export function nakshatraName(n: number): string {
   return NAKSHATRA_NAMES[n - 1] ?? String(n);
```

- [ ] **Step 3: Panel, kayıt, stil ve metinler**

**Yeni dosya:** `app/ui/panels/PanchangPanel.tsx`

```tsx
// Panchang paneli (ileri modüller spec'i §4): doğum anının beş uzvu, bitiş saatleri, gün doğumu
// ve batımı. Değerler `Chart.panchang`'tan hazır gelir (v1'den beri hesaplanır); komut çağırmaz.
// Saatler doğumun çözülmüş ofsetinde gösterilir. Ortak vurgu: Ay vurgulanınca Ay'a bağlı
// satırlar, Güneş vurgulanınca Güneş'e bağlı satırlar `--select` zemin alır.
import type { TFunction } from "i18next";
import { useTranslation } from "react-i18next";
import type { Panchang } from "../bindings";
import { formatFixedOffset } from "../birth/model";
import type { GrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { Approx } from "./Approx";
import { formatJdMinute } from "./format";
import { nakshatraName, nityaYogaName, tithiInPaksha, tithiName, VARA_NAMES } from "./labels";
import "./panels.css";
import type { PanelProps } from "./types";

export interface PanchangRow {
  key: "tithi" | "vara" | "nakshatra" | "yoga" | "karana" | "sunrise" | "sunset";
  label: string;
  value: string;
  /** Bitiş anı; öğenin bitişi yoksa `null`. */
  ends: string | null;
  /** Değerin türediği grahalar (ortak vurgu için). */
  grahas: readonly GrahaId[];
  /** Doğum saati bilinmiyorsa "≈" ile işaretlenir (saate bağlı değer). */
  timeBound: boolean;
}

const SUN_MOON: readonly GrahaId[] = ["Sun", "Moon"];

/** Panchang satırları; `offsetSecs` doğumun çözülmüş ofsetidir. */
export function panchangRows(t: TFunction, p: Panchang, offsetSecs: number): PanchangRow[] {
  const at = (jd: number | null) => formatJdMinute(jd, offsetSecs);
  return [
    {
      key: "tithi",
      label: t("panchang.tithi"),
      value: `${p.tithi.paksha} ${tithiInPaksha(p.tithi.no)} · ${tithiName(p.tithi.no)}`,
      ends: at(p.tithi.ends_jd_ut),
      grahas: SUN_MOON,
      timeBound: true,
    },
    { key: "vara", label: t("panchang.vara"), value: `${VARA_NAMES[p.vara]} · ${t(`panchang.weekday.${p.vara}`)}`, ends: null, grahas: [], timeBound: true },
    {
      key: "nakshatra",
      label: t("panchang.nakshatra"),
      value: `${nakshatraName(p.nakshatra.no)} · ${t("panchang.pada", { pada: p.nakshatra.pada })}`,
      ends: at(p.nakshatra.ends_jd_ut),
      grahas: ["Moon"],
      timeBound: true,
    },
    { key: "yoga", label: t("panchang.yoga"), value: nityaYogaName(p.yoga.no), ends: at(p.yoga.ends_jd_ut), grahas: SUN_MOON, timeBound: true },
    { key: "karana", label: t("panchang.karana"), value: p.karana.name, ends: at(p.karana.ends_jd_ut), grahas: SUN_MOON, timeBound: true },
    { key: "sunrise", label: t("panchang.sunrise"), value: at(p.sunrise_jd_ut), ends: null, grahas: ["Sun"], timeBound: false },
    { key: "sunset", label: t("panchang.sunset"), value: at(p.sunset_jd_ut), ends: null, grahas: ["Sun"], timeBound: false },
  ];
}

export function PanchangPanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { active } = useHighlight();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;

  const panchang = opened.chart.panchang;
  const resolved = opened.chart_record.resolved;
  const unknown = timeUnknown(opened);
  const noSunrise = panchang.sunrise_jd_ut === null || panchang.sunset_jd_ut === null;
  const rows = panchangRows(t, panchang, resolved.offset_secs);
  const offset = `${resolved.abbrev} ${formatFixedOffset(resolved.offset_secs)}`.trim();

  return (
    <div className="module-panel">
      {unknown && <p className="panel-notice">{t("panchang.timeUnknown")}</p>}
      <table className="module-table">
        <caption className="visually-hidden">{t("panchang.caption", { offset })}</caption>
        <thead>
          <tr>
            <th scope="col">{t("panchang.item")}</th>
            <th scope="col">{t("panchang.value")}</th>
            <th scope="col" className="num">
              {t("panchang.ends")}
            </th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => {
            const on = active?.type === "graha" && r.grahas.includes(active.id);
            return (
              <tr key={r.key} data-row={r.key} className={on ? "is-highlighted" : undefined}>
                <th scope="row">{r.label}</th>
                <td>
                  <Approx on={unknown && r.timeBound} label={t("table.approx")}>
                    {r.value}
                  </Approx>
                </td>
                <td className="num">{r.ends ?? ""}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {noSunrise && (
        <p className="panel-notice">
          <strong>{t("badge.SunriseUnavailable")}</strong>: {t("badgeHint.SunriseUnavailable")}
        </p>
      )}
    </div>
  );
}
```

```diff
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index f5d4158..d4ccb82 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -7,6 +7,7 @@ import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel, chartVarga } from "./ChartPanel";
 import { TransitTablePanel } from "../transit/TransitTablePanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
+import { PanchangPanel } from "./PanchangPanel";
 import type { PanelDefinition } from "./types";
 import { UnavailablePanel } from "./UnavailablePanel";
 
@@ -18,8 +19,8 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   transit_table: { titleKey: "panel.transit_table", component: TransitTablePanel },
   notes: { titleKey: "panel.notes", component: NotesPanel },
   birth_info: { titleKey: "panel.birth_info", component: BirthInfoPanel },
-  // İleri modül panelleri (Plan 14): içerikleri Task 6–8'de gelir.
-  panchang: { titleKey: "panel.panchang", component: UnavailablePanel },
+  // İleri modül panelleri (Plan 14).
+  panchang: { titleKey: "panel.panchang", component: PanchangPanel },
   ashtakavarga: { titleKey: "panel.ashtakavarga", component: UnavailablePanel },
   drishti: { titleKey: "panel.drishti", component: UnavailablePanel },
 };
diff --git a/app/ui/panels/panels.css b/app/ui/panels/panels.css
index ca12cd6..0d06ce6 100644
--- a/app/ui/panels/panels.css
+++ b/app/ui/panels/panels.css
@@ -16,25 +16,47 @@
   flex: 1;
   min-height: 0;
 }
-.graha-table {
+.graha-table,
+.module-table {
   width: 100%;
   border-collapse: collapse;
 }
 .graha-table th,
-.graha-table td {
+.graha-table td,
+.module-table th,
+.module-table td {
   text-align: left;
   padding: 2px 6px;
   border-bottom: 1px solid var(--line);
 }
-.graha-table td.num {
+.graha-table td.num,
+.module-table td.num,
+.module-table th.num {
   text-align: right;
+  font-variant-numeric: tabular-nums;
 }
 .graha-table tr.is-highlighted th,
-.graha-table tr.is-highlighted td {
+.graha-table tr.is-highlighted td,
+.module-table tr.is-highlighted th,
+.module-table tr.is-highlighted td,
+.module-table th.is-highlighted,
+.module-table td.is-highlighted {
   background: var(--select);
   color: var(--ink-strong);
   font-weight: var(--fw-medium);
 }
+.module-panel {
+  display: flex;
+  flex-direction: column;
+  gap: 4px;
+}
+.module-toolbar {
+  display: flex;
+  align-items: center;
+  flex-wrap: wrap;
+  gap: 6px 12px;
+  color: var(--text-2);
+}
 .link-button {
   all: unset;
   cursor: default;
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index cbb8c4e..98f218c 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -67,6 +67,30 @@ export const tr = {
     announce: "Sütun {{col}}, satır {{row}}, {{w}}×{{h}}",
     noChart: "Açık harita yok.",
   },
+  panchang: {
+    caption: "Doğum anının panchangı; saatler doğum yerinin saatiyle ({{offset}})",
+    item: "Öğe",
+    value: "Değer",
+    ends: "Bitiş",
+    tithi: "Tithi",
+    vara: "Vara",
+    nakshatra: "Nakshatra",
+    yoga: "Nitya yoga",
+    karana: "Karana",
+    sunrise: "Gün doğumu",
+    sunset: "Gün batımı",
+    pada: "pada {{pada}}",
+    timeUnknown: "Doğum saati bilinmiyor: değerler öğlen 12:00 içindir ve yaklaşıktır (≈).",
+    weekday: {
+      Sunday: "Pazar",
+      Monday: "Pazartesi",
+      Tuesday: "Salı",
+      Wednesday: "Çarşamba",
+      Thursday: "Perşembe",
+      Friday: "Cuma",
+      Saturday: "Cumartesi",
+    },
+  },
   table: {
     graha: "Graha",
     sign: "Burç",
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 4c573f5..141a392 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -68,6 +68,30 @@ export const en: Messages = {
     announce: "Column {{col}}, row {{row}}, {{w}}×{{h}}",
     noChart: "No chart is open.",
   },
+  panchang: {
+    caption: "Panchang of the birth moment; times in the birth place's time ({{offset}})",
+    item: "Item",
+    value: "Value",
+    ends: "Ends",
+    tithi: "Tithi",
+    vara: "Vara",
+    nakshatra: "Nakshatra",
+    yoga: "Nitya yoga",
+    karana: "Karana",
+    sunrise: "Sunrise",
+    sunset: "Sunset",
+    pada: "pada {{pada}}",
+    timeUnknown: "Birth time unknown: values are for 12:00 noon and approximate (≈).",
+    weekday: {
+      Sunday: "Sunday",
+      Monday: "Monday",
+      Tuesday: "Tuesday",
+      Wednesday: "Wednesday",
+      Thursday: "Thursday",
+      Friday: "Friday",
+      Saturday: "Saturday",
+    },
+  },
   table: {
     graha: "Graha",
     sign: "Sign",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index abbdf7d..c40b377 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -68,6 +68,30 @@ export const fr: Messages = {
     announce: "Colonne {{col}}, ligne {{row}}, {{w}}×{{h}}",
     noChart: "Aucune carte ouverte.",
   },
+  panchang: {
+    caption: "Panchang du moment de naissance ; heures du lieu de naissance ({{offset}})",
+    item: "Élément",
+    value: "Valeur",
+    ends: "Fin",
+    tithi: "Tithi",
+    vara: "Vara",
+    nakshatra: "Nakshatra",
+    yoga: "Nitya yoga",
+    karana: "Karana",
+    sunrise: "Lever du soleil",
+    sunset: "Coucher du soleil",
+    pada: "pada {{pada}}",
+    timeUnknown: "Heure de naissance inconnue : valeurs pour 12 h 00 et approximatives (≈).",
+    weekday: {
+      Sunday: "dimanche",
+      Monday: "lundi",
+      Tuesday: "mardi",
+      Wednesday: "mercredi",
+      Thursday: "jeudi",
+      Friday: "vendredi",
+      Saturday: "samedi",
+    },
+  },
   table: {
     graha: "Graha",
     sign: "Signe",
```

- [ ] **Step 4: Kapılar**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | tail -1
```
Expected: `Test Files  63 passed (63)`, `Tests  549 passed (549)` (+8: `format` 1, `PanchangPanel` 7); `✓ built`. `GrahaTablePanel.test.tsx` değişmeden geçer (`Approx` taşındı, çıktısı aynı).

- [ ] **Step 5: Commit**

```bash
git add app/ui/panels app/ui/i18n
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): panchang panel"
```
Expected: iki `grep` çıktısız.

---

### Task 7: Ashtakavarga paneli

Spec §4 "Arayüz", `ashtakavarga` satırı; D10, D11. Ayarlar: görünüm (`view`: `"sav"` ya da yedi grahadan biri) ve biçim (`format`: `"table"` ya da `"chart"`). Durumlar: harita yok; dolu (tablo ve harita içi); saat bilinmiyor (tablo yerine tek satır açıklama). Ortak seçim: graha satırı; burç sütunu; ev vurgusu D1 evinin burç sütununa düşer.

**Files:**
- Create: `app/ui/panels/AshtakavargaPanel.tsx`, `app/ui/panels/AshtakavargaPanel.test.tsx`
- Modify: `app/ui/panels/registry.ts`, `app/ui/panels/panels.css`, `app/ui/i18n/{tr,en,fr}.ts`
- Test: `app/ui/panels/AshtakavargaPanel.test.tsx`, `app/ui/panels/panels.test.tsx`

**Interfaces:**
- Consumes: `Ashtakavarga` (bağlamalar: `bav` 7 × 12, `sav` 12; burç sütunları Koç'tan), `signIndex` (`chart/adapter.ts`), `useHighlight`, `sameTarget`, `HighlightTarget`, `useLayoutStore().setPanelSettings`, `timeUnknown`, `.module-table` / `.module-panel` / `.module-toolbar` (Task 6), i18n `chart.lagna`, `table.graha`, `sign.*`, `grahaName.*`.
- Produces: `AshtakavargaPanel`, `BAV_GRAHAS`, `ashtakavargaView(settings): "sav" | BavGraha`, `ashtakavargaFormat(settings): "table" | "chart"`; i18n `ashtakavarga.*`; CSS `.av-*`.

- [ ] **Step 1: Başarısız testleri yaz**

SAV satırı `tests/golden/new-delhi-2000-01-01.json`'daki `sav` ile birebirdir; BAV satır toplamları Rust'ın `BAV_TOTALS` sabitleridir.

**Yeni dosya:** `app/ui/panels/AshtakavargaPanel.test.tsx`

```tsx
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { expect, test } from "vitest";
import { useHighlight } from "../highlight/store";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { ashtakavargaFormat, AshtakavargaPanel, ashtakavargaView, BAV_GRAHAS } from "./AshtakavargaPanel";

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  const panel: Panel = { id: "av", kind: "ashtakavarga", x: 0, y: 0, w: 6, h: 11, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(
    <>
      <HousePin />
      <AshtakavargaPanel panel={panel} />
    </>,
    { layout, opened },
  );
}

/** Harita panelinin yerine: 1. evi vurgular. */
function HousePin() {
  const h = useHighlight();
  return <button onClick={() => h.togglePin({ type: "house", house: 1 })}>pin house 1</button>;
}

const SIGNS = ["Koç", "Boğ", "İki", "Yen", "Asl", "Baş", "Ter", "Akr", "Yay", "Oğl", "Kov", "Bal"];
const rowCells = (name: string) => [...screen.getByRole("rowheader", { name }).closest("tr")!.querySelectorAll("td")].map((c) => c.textContent);
const savedSettings = (persistence: ReturnType<typeof renderPanel>["persistence"]) =>
  persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings;

test("default view: seven BAV rows and the SAV row, twelve sign columns from Aries, row totals", () => {
  renderPanel();
  // Lagna İkizler: burç başlığında "Lg" metniyle işaretli.
  expect(screen.getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Graha", "Koç", "Boğ", "İki Lg", ...SIGNS.slice(3), "Toplam"]);
  // tests/golden/new-delhi-2000-01-01.json: SAV birebir.
  expect(rowCells("SAV")).toEqual(["29", "21", "23", "26", "34", "33", "30", "30", "28", "21", "32", "30", "337"]);
  expect(rowCells("Güneş")).toEqual(["4", "5", "1", "3", "6", "5", "5", "4", "5", "2", "4", "4", "48"]);
  // BPHS: her BAV'ın toplamı yerleşimden bağımsızdır (Rust `BAV_TOTALS`).
  expect(["Güneş", "Ay", "Mars", "Merkür", "Jüpiter", "Venüs", "Satürn"].map((g) => rowCells(g).at(-1))).toEqual(["48", "49", "39", "54", "56", "52", "39"]);
  expect(screen.getAllByRole("row")).toHaveLength(9);
  expect(screen.getByRole("table", { name: "Ashtakavarga, SAV: burç başına bindu" })).toBeInTheDocument();
  expect(screen.getByRole("combobox", { name: "Görünüm" })).toHaveDisplayValue("SAV");
  expect(screen.getByRole("combobox", { name: "Biçim" })).toHaveDisplayValue("Tablo");
});

test("a single graha view shows only that BAV row", () => {
  renderPanel({ view: "Moon" });
  expect(screen.getByRole("combobox", { name: "Görünüm" })).toHaveDisplayValue("Ay BAV");
  expect(screen.getAllByRole("row")).toHaveLength(2);
  expect(rowCells("Ay")).toEqual(["5", "2", "4", "6", "4", "4", "5", "3", "3", "2", "5", "6", "49"]);
  expect(screen.queryByRole("rowheader", { name: "SAV" })).toBeNull();
});

test("in-chart format: twelve fixed sign boxes with the chosen view's numbers", () => {
  renderPanel({ format: "chart" });
  const group = screen.getByRole("group", { name: "Ashtakavarga, SAV: burç başına bindu" });
  const items = [...group.querySelectorAll<HTMLElement>("li[data-sign]")];
  expect(items.map((li) => li.textContent)).toEqual(["Koç29", "Boğ21", "İki Lg23", "Yen26", "Asl34", "Baş33", "Ter30", "Akr30", "Yay28", "Oğl21", "Kov32", "Bal30"]);
  // Güney düzeni: Balık sol üst, Koç onun sağı, Terazi alt sırada.
  const cell = (sign: number) => [items[sign].style.gridRow, items[sign].style.gridColumn];
  expect([cell(11), cell(0), cell(3), cell(6), cell(9)]).toEqual([
    ["1", "1"],
    ["1", "2"],
    ["2", "4"],
    ["4", "3"],
    ["3", "1"],
  ]);
  expect(within(group).getByText("SAV toplamı 337")).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
});

test("in-chart format follows the view", () => {
  renderPanel({ format: "chart", view: "Saturn" });
  const group = screen.getByRole("group", { name: "Ashtakavarga, Satürn BAV: burç başına bindu" });
  expect([...group.querySelectorAll(".av-cell-value")].map((v) => v.textContent)).toEqual(["3", "1", "4", "3", "5", "6", "3", "3", "3", "2", "2", "4"]);
  expect(within(group).getByText("Satürn BAV toplamı 39")).toBeInTheDocument();
});

test("view and format are panel settings, saved with the layout; unknown values fall back", async () => {
  const { persistence } = renderPanel();
  fireEvent.change(screen.getByRole("combobox", { name: "Görünüm" }), { target: { value: "Jupiter" } });
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  expect(savedSettings(persistence)).toEqual({ view: "Jupiter" });
  fireEvent.change(screen.getByRole("combobox", { name: "Biçim" }), { target: { value: "chart" } });
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ view: "Jupiter", format: "chart" }));
  expect(ashtakavargaView({ view: "Rahu" })).toBe("sav");
  expect(ashtakavargaView({ view: 3 })).toBe("sav");
  expect(ashtakavargaView({})).toBe("sav");
  expect(BAV_GRAHAS.map((g) => ashtakavargaView({ view: g }))).toEqual([...BAV_GRAHAS]);
  expect([ashtakavargaFormat({}), ashtakavargaFormat({ format: "pie" }), ashtakavargaFormat({ format: "chart" })]).toEqual(["table", "table", "chart"]);
});

test("without an open chart, and with an unknown birth time, no numbers are shown", () => {
  const first = renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  first.unmount();
  renderPanel({}, unknownTimeOpened());
  expect(screen.getByText("Doğum saati bilinmiyor: ashtakavarga lagnaya da bağlı olduğu için gösterilmiyor.")).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.queryByRole("combobox")).toBeNull();
});

test("shared highlight: a graha marks its row, a sign or a house its column", () => {
  renderPanel();
  const marked = () => document.querySelectorAll("td.is-highlighted").length;
  const mars = screen.getByRole("button", { name: "Mars" });
  fireEvent.click(mars);
  expect(mars).toHaveAttribute("aria-pressed", "true");
  expect(mars.closest("tr")).toHaveClass("is-highlighted");
  expect(screen.getByRole("button", { name: "Güneş" }).closest("tr")).not.toHaveClass("is-highlighted");
  // Burç başlığı: yedi BAV hücresi ve SAV hücresi.
  const leo = screen.getByRole("button", { name: "Asl" });
  fireEvent.click(leo);
  expect(leo).toHaveAttribute("aria-pressed", "true");
  expect(mars.closest("tr")).not.toHaveClass("is-highlighted");
  expect(marked()).toBe(8);
  expect(document.querySelector('th[data-sign="4"]')).toHaveClass("is-highlighted");
  expect(screen.getByRole("rowheader", { name: "SAV" }).closest("tr")!.querySelector("td.is-highlighted")).toHaveTextContent("34");
  // 1. ev: lagna burcu İkizler'in sütunu.
  fireEvent.click(screen.getByRole("button", { name: "pin house 1" }));
  expect(document.querySelector('th[data-sign="2"]')).toHaveClass("is-highlighted");
  expect(document.querySelector('th[data-sign="4"]')).not.toHaveClass("is-highlighted");
  expect(marked()).toBe(8);
  // Esc vurguyu kaldırır.
  fireEvent.keyDown(window, { key: "Escape" });
  expect(marked()).toBe(0);
});
```

```diff
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index f81a3fa..a991b7a 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -8,6 +8,7 @@ import { NotesPanel } from "../notes/NotesPanel";
 import { fixture } from "../test/fixture";
 import { renderWithStores } from "../test/render";
 import { TransitTablePanel } from "../transit/TransitTablePanel";
+import { AshtakavargaPanel } from "./AshtakavargaPanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel } from "./ChartPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
@@ -36,6 +37,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.dasha_strip.component).toBe(DashaStripPanel);
   expect(PANEL_REGISTRY.transit_table.component).toBe(TransitTablePanel);
   expect(PANEL_REGISTRY.panchang.component).toBe(PanchangPanel);
+  expect(PANEL_REGISTRY.ashtakavarga.component).toBe(AshtakavargaPanel);
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
   const keys = new Set(Object.keys(tr.panel).map((k) => `panel.${k}`));
   for (const kind of [...PANEL_KINDS, "yoga_list"]) expect(keys.has(panelDefinition(kind).titleKey)).toBe(true);
```

Run: `pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "`
Expected: `Test Files  2 failed | 62 passed (64)`.

- [ ] **Step 2: Panel, kayıt, stil ve metinler**

**Yeni dosya:** `app/ui/panels/AshtakavargaPanel.tsx`

```tsx
// Ashtakavarga paneli (ileri modüller spec'i §4): yedi grahanın BAV'ı ve SAV. Değerler
// `Chart.ashtakavarga`'dan hazır gelir (v1'den beri hesaplanır); komut çağırmaz. Burç sütunları
// Koç'tan Balık'a sabittir. Görünüm (SAV / tek graha BAV) ve biçim (tablo / harita içi) panel
// ayarıdır. Satır toplamı gösterilen hücrelerin toplamıdır (Plan 14 D11); sütun toplamı SAV
// satırıdır ve Rust'tan gelir. Bindular lagnaya da bağlıdır: doğum saati bilinmiyorsa tablo
// gösterilmez. Ortak vurgu: graha satırı, burç ve ev sütunu `--select` zemin alır.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { Ashtakavarga } from "../bindings";
import { signIndex } from "../chart/adapter";
import type { GrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { sameTarget, type HighlightTarget } from "../highlight/target";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import "./panels.css";
import type { PanelProps } from "./types";

/** BAV satır sırası (Rust `ASHTAKAVARGA_GRAHAS`). */
export const BAV_GRAHAS = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"] as const satisfies readonly GrahaId[];
export type BavGraha = (typeof BAV_GRAHAS)[number];
export type AshtakavargaView = "sav" | BavGraha;
export const ASHTAKAVARGA_FORMATS = ["table", "chart"] as const;
export type AshtakavargaFormat = (typeof ASHTAKAVARGA_FORMATS)[number];

export function ashtakavargaView(settings: Record<string, unknown>): AshtakavargaView {
  const v = settings.view;
  return typeof v === "string" && (BAV_GRAHAS as readonly string[]).includes(v) ? (v as BavGraha) : "sav";
}

export function ashtakavargaFormat(settings: Record<string, unknown>): AshtakavargaFormat {
  return settings.format === "chart" ? "chart" : "table";
}

const SIGNS = Array.from({ length: 12 }, (_, i) => i);
const total = (row: readonly number[]) => row.reduce((a, b) => a + b, 0);

/** Güney Hint düzeni: burçlar sabit kutularda, Balık sol üstte, saat yönünde. `[satır, sütun]`, 1 tabanlı. */
const SOUTH_CELLS: readonly (readonly [number, number])[] = [
  [1, 2],
  [1, 3],
  [1, 4],
  [2, 4],
  [3, 4],
  [4, 4],
  [4, 3],
  [4, 2],
  [4, 1],
  [3, 1],
  [2, 1],
  [1, 1],
];

export function AshtakavargaPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { setPanelSettings } = useLayoutStore();
  const highlight = useHighlight();
  const id = useId();
  const view = ashtakavargaView(panel.settings);
  const format = ashtakavargaFormat(panel.settings);

  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (timeUnknown(opened)) return <p className="panel-notice">{t("ashtakavarga.timeUnknown")}</p>;

  const av: Ashtakavarga = opened.chart.ashtakavarga;
  const lagna = signIndex(opened.chart.lagna.sign);
  const viewLabel = (v: AshtakavargaView) => (v === "sav" ? t("ashtakavarga.sav") : t("ashtakavarga.bavOf", { graha: t(`grahaName.${v}`) }));
  const active = highlight.active;
  // Ev vurgusu D1 evidir: lagna burcundan sayılan burcun sütunu.
  const signOn = (sign: number) => (active?.type === "sign" && active.sign === sign) || (active?.type === "house" && (lagna + active.house - 1) % 12 === sign);
  const grahaOn = (g: BavGraha) => active?.type === "graha" && active.id === g;
  const pinButton = (target: HighlightTarget, label: string) => (
    <button
      type="button"
      className="link-button"
      aria-pressed={sameTarget(highlight.pinned, target)}
      onClick={() => highlight.togglePin(target)}
      onMouseEnter={() => highlight.hover(target)}
      onMouseLeave={() => highlight.hover(null)}
      onFocus={() => highlight.hover(target)}
      onBlur={() => highlight.hover(null)}
    >
      {label}
    </button>
  );
  const signHeader = (sign: number) => (
    <>
      {pinButton({ type: "sign", sign }, t(`sign.${sign}`))}
      {sign === lagna && <span className="av-lagna"> {t("chart.lagna")}</span>}
    </>
  );
  const bavRows = view === "sav" ? BAV_GRAHAS : [view];
  const values = view === "sav" ? av.sav : av.bav[BAV_GRAHAS.indexOf(view)];

  return (
    <div className="module-panel">
      <div className="module-toolbar">
        <label htmlFor={`${id}-view`}>{t("ashtakavarga.view")}</label>
        <select id={`${id}-view`} value={view} onChange={(e) => setPanelSettings(panel.id, { view: e.target.value })}>
          {(["sav", ...BAV_GRAHAS] as const).map((v) => (
            <option key={v} value={v}>
              {viewLabel(v)}
            </option>
          ))}
        </select>
        <label htmlFor={`${id}-format`}>{t("ashtakavarga.format")}</label>
        <select id={`${id}-format`} value={format} onChange={(e) => setPanelSettings(panel.id, { format: e.target.value })}>
          {ASHTAKAVARGA_FORMATS.map((f) => (
            <option key={f} value={f}>
              {t(`ashtakavarga.formats.${f}`)}
            </option>
          ))}
        </select>
      </div>
      {format === "table" ? (
        <table className="module-table av-table">
          <caption className="visually-hidden">{t("ashtakavarga.caption", { view: viewLabel(view) })}</caption>
          <thead>
            <tr>
              <th scope="col">{t("table.graha")}</th>
              {SIGNS.map((s) => (
                <th key={s} scope="col" data-sign={s} className={signOn(s) ? "num is-highlighted" : "num"}>
                  {signHeader(s)}
                </th>
              ))}
              <th scope="col" className="num">
                {t("ashtakavarga.total")}
              </th>
            </tr>
          </thead>
          <tbody>
            {bavRows.map((g) => {
              const row = av.bav[BAV_GRAHAS.indexOf(g)];
              return (
                <tr key={g} data-graha={g} className={grahaOn(g) ? "is-highlighted" : undefined}>
                  <th scope="row">{pinButton({ type: "graha", id: g }, t(`grahaName.${g}`))}</th>
                  {row.map((n, s) => (
                    <td key={s} className={signOn(s) ? "num is-highlighted" : "num"}>
                      {n}
                    </td>
                  ))}
                  <td className="num">{total(row)}</td>
                </tr>
              );
            })}
          </tbody>
          {view === "sav" && (
            <tfoot>
              <tr data-row="sav">
                <th scope="row">{t("ashtakavarga.sav")}</th>
                {av.sav.map((n, s) => (
                  <td key={s} className={signOn(s) ? "num is-highlighted" : "num"}>
                    {n}
                  </td>
                ))}
                <td className="num">{total(av.sav)}</td>
              </tr>
            </tfoot>
          )}
        </table>
      ) : (
        <div className="av-chart" role="group" aria-label={t("ashtakavarga.caption", { view: viewLabel(view) })}>
          <ul className="av-chart-grid">
            {SIGNS.map((s) => (
              <li key={s} data-sign={s} className={signOn(s) ? "av-cell is-highlighted" : "av-cell"} style={{ gridRow: SOUTH_CELLS[s][0], gridColumn: SOUTH_CELLS[s][1] }}>
                <span className="av-cell-sign">{signHeader(s)}</span>
                <span className="av-cell-value">{values[s]}</span>
              </li>
            ))}
            <li className="av-chart-centre" aria-hidden="true">
              {viewLabel(view)} · {total(values)}
            </li>
          </ul>
          <p className="visually-hidden">{t("ashtakavarga.totalOf", { view: viewLabel(view), total: total(values) })}</p>
        </div>
      )}
    </div>
  );
}
```

```diff
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index d4ccb82..bcddccb 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -3,6 +3,7 @@ import { DashaStripPanel } from "../dasha/DashaStripPanel";
 import { DashaTreePanel } from "../dasha/DashaTreePanel";
 import { isKnownPanelKind, type KnownPanelKind } from "../layout/model";
 import { NotesPanel } from "../notes/NotesPanel";
+import { AshtakavargaPanel } from "./AshtakavargaPanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel, chartVarga } from "./ChartPanel";
 import { TransitTablePanel } from "../transit/TransitTablePanel";
@@ -21,7 +22,7 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   birth_info: { titleKey: "panel.birth_info", component: BirthInfoPanel },
   // İleri modül panelleri (Plan 14).
   panchang: { titleKey: "panel.panchang", component: PanchangPanel },
-  ashtakavarga: { titleKey: "panel.ashtakavarga", component: UnavailablePanel },
+  ashtakavarga: { titleKey: "panel.ashtakavarga", component: AshtakavargaPanel },
   drishti: { titleKey: "panel.drishti", component: UnavailablePanel },
 };
 
diff --git a/app/ui/panels/panels.css b/app/ui/panels/panels.css
index 0d06ce6..66318ac 100644
--- a/app/ui/panels/panels.css
+++ b/app/ui/panels/panels.css
@@ -134,3 +134,51 @@
   margin: 0;
   padding-left: 16px;
 }
+.av-table th,
+.av-table td {
+  padding: 2px 4px;
+}
+.av-lagna {
+  color: var(--text-2);
+  font-size: 0.85em;
+}
+.av-chart-grid {
+  display: grid;
+  grid-template-columns: repeat(4, 1fr);
+  grid-template-rows: repeat(4, minmax(44px, 1fr));
+  margin: 0;
+  padding: 0;
+  list-style: none;
+  border-top: 1px solid var(--line);
+  border-left: 1px solid var(--line);
+}
+.av-cell {
+  display: flex;
+  flex-direction: column;
+  align-items: center;
+  justify-content: center;
+  gap: 2px;
+  border-right: 1px solid var(--line);
+  border-bottom: 1px solid var(--line);
+}
+.av-cell.is-highlighted {
+  background: var(--select);
+  color: var(--ink-strong);
+}
+.av-cell-sign {
+  color: var(--text-2);
+}
+.av-cell-value {
+  font-variant-numeric: tabular-nums;
+  font-weight: var(--fw-medium);
+}
+.av-chart-centre {
+  grid-row: 2 / span 2;
+  grid-column: 2 / span 2;
+  display: flex;
+  align-items: center;
+  justify-content: center;
+  color: var(--text-2);
+  border-right: 1px solid var(--line);
+  border-bottom: 1px solid var(--line);
+}
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 98f218c..a8bf25b 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -91,6 +91,17 @@ export const tr = {
       Saturday: "Cumartesi",
     },
   },
+  ashtakavarga: {
+    view: "Görünüm",
+    format: "Biçim",
+    sav: "SAV",
+    bavOf: "{{graha}} BAV",
+    formats: { table: "Tablo", chart: "Harita içi" },
+    total: "Toplam",
+    caption: "Ashtakavarga, {{view}}: burç başına bindu",
+    totalOf: "{{view}} toplamı {{total}}",
+    timeUnknown: "Doğum saati bilinmiyor: ashtakavarga lagnaya da bağlı olduğu için gösterilmiyor.",
+  },
   table: {
     graha: "Graha",
     sign: "Burç",
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 141a392..b1026e4 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -92,6 +92,17 @@ export const en: Messages = {
       Saturday: "Saturday",
     },
   },
+  ashtakavarga: {
+    view: "View",
+    format: "Format",
+    sav: "SAV",
+    bavOf: "{{graha}} BAV",
+    formats: { table: "Table", chart: "In chart" },
+    total: "Total",
+    caption: "Ashtakavarga, {{view}}: bindus per sign",
+    totalOf: "{{view}} total {{total}}",
+    timeUnknown: "Birth time unknown: ashtakavarga also depends on the lagna and is not shown.",
+  },
   table: {
     graha: "Graha",
     sign: "Sign",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index c40b377..11d5481 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -92,6 +92,17 @@ export const fr: Messages = {
       Saturday: "samedi",
     },
   },
+  ashtakavarga: {
+    view: "Vue",
+    format: "Format",
+    sav: "SAV",
+    bavOf: "BAV de {{graha}}",
+    formats: { table: "Tableau", chart: "Dans la carte" },
+    total: "Total",
+    caption: "Ashtakavarga, {{view}} : bindus par signe",
+    totalOf: "Total {{view}} : {{total}}",
+    timeUnknown: "Heure de naissance inconnue : l'ashtakavarga dépend aussi du lagna et n'est pas affiché.",
+  },
   table: {
     graha: "Graha",
     sign: "Signe",
```

- [ ] **Step 3: Kapılar**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | tail -1
```
Expected: `Test Files  64 passed (64)`, `Tests  556 passed (556)` (+7); `✓ built`.

- [ ] **Step 4: Commit**

```bash
git add app/ui/panels app/ui/i18n
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): ashtakavarga panel"
```
Expected: iki `grep` çıktısız.

---

### Task 8: Bakışlar paneli

Spec §4 "Arayüz", `drishti` satırı; D12, D13. Panel açıkken `compute_drishti`'yi ister ve bakan yedi graha × bakılan dokuz graha matrisini çizer. Durumlar (UI spec §5, §8): harita yok (komut çağrılmaz); yükleniyor (100 ms altında gösterge yok, 1 s'ye dek başlıkta satır içi gösterge, sonra iskelet); hata (panel içinde, "Yeniden dene"); dolu; saat bilinmiyor (`time_known: false` gönderilir, uyarı gösterilir, Ay'ın satır ve sütun başlığı "≈" alır). Ortak seçim: bir graha vurgulanınca satırı ve sütunu `--select` alır; başlıklar vurguyu sabitleyen düğmelerdir.

**Files:**
- Create: `app/ui/panels/DrishtiPanel.tsx`, `app/ui/panels/DrishtiPanel.test.tsx`
- Modify: `app/ui/panels/registry.ts`, `app/ui/panels/panels.css`, `app/ui/i18n/{tr,en,fr}.ts`
- Test: `app/ui/panels/DrishtiPanel.test.tsx`, `app/ui/panels/panels.test.tsx`, `app/ui/App.test.tsx`

**Interfaces:**
- Consumes: `api.computeDrishti` (Task 4), `chartInput`, `inputKey`, `useModule` (Task 5), `DrishtiTable` (`sign_aspects[].{graha, occupants}`, `sphuta[7][9]`, `method`), `GRAHA_IDS`, `PanelError`, `PanelSkeleton`, `usePanelLoadPhase`, `Approx` (Task 6), `.module-*` sınıfları, `UiFixture.drishti` (Task 4).
- Produces: `DrishtiPanel`, `ASPECTING_GRAHAS`, `DRISHTI_VALUES`, `drishtiValue(settings): "full" | "virupa"`, `fullAspectPairs(table): Set<string>` (`"Bakan>Bakılan"`), `formatVirupa(v): string`, `FULL_MARK = "●"`; i18n `drishti.*`.

- [ ] **Step 1: Başarısız testleri yaz**

Beklenen tam bakışlar Delhi haritasının burçlarından elle çıkarılmıştır: Ay Terazi'den Koç'a (Jüpiter, Satürn); Jüpiter Koç'tan Terazi'ye (Ay) ve Yay'a (Güneş, Merkür); Satürn Koç'tan Terazi'ye (Ay) ve Oğlak'a (Ketu) bakar; yedi işaret.

**Yeni dosya:** `app/ui/panels/DrishtiPanel.test.tsx`

```tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeDrishti: vi.fn() } }));

import { api, type Result } from "../api";
import type { DrishtiTable } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { ASPECTING_GRAHAS, DrishtiPanel, drishtiValue, formatVirupa, fullAspectPairs, FULL_MARK } from "./DrishtiPanel";

const ok = (): Result<DrishtiTable> => ({ status: "ok", data: fixture().drishti });

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  const panel: Panel = { id: "dr", kind: "drishti", x: 0, y: 0, w: 6, h: 10, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<DrishtiPanel panel={panel} />, { layout, opened });
}

const rowHeader = (name: string) => within(document.querySelector("tbody")!).getByRole("button", { name });
const rowCells = (name: string) => [...rowHeader(name).closest("tr")!.querySelectorAll("td")].map((c) => c.textContent);

beforeEach(() => {
  vi.mocked(api.computeDrishti).mockReset();
  vi.mocked(api.computeDrishti).mockImplementation(async () => ok());
});

test("asks for the open chart's aspects and marks full sign aspects, seven rows by nine columns", async () => {
  renderPanel();
  await screen.findByRole("table", { name: "Graha bakışları: satırlar bakan, sütunlar bakılan graha" });
  const opened = fixture().opened;
  expect(api.computeDrishti).toHaveBeenCalledTimes(1);
  expect(api.computeDrishti).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true });
  expect(screen.getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Bakan", "Gü", "Ay", "Ma", "Me", "Jü", "Ve", "Sa", "Ra", "Ke"]);
  expect(screen.getAllByRole("rowheader").map((h) => h.textContent)).toEqual(["Güneş", "Ay", "Mars", "Merkür", "Jüpiter", "Venüs", "Satürn"]);
  // Delhi fikstürü: Ay Terazi'den Koç'a (Jüpiter, Satürn); Jüpiter Koç'tan Terazi'ye (Ay) ve
  // Yay'a (Güneş, Merkür); Satürn Koç'tan Terazi'ye (Ay) ve Oğlak'a (Ketu) bakar.
  const full = `${FULL_MARK}tam bakış`;
  expect(rowCells("Güneş")).toEqual(["—", "", "", "", "", "", "", "", ""]);
  expect(rowCells("Ay")).toEqual(["", "—", "", "", full, "", full, "", ""]);
  expect(rowCells("Mars")).toEqual(["", "", "—", "", "", "", "", "", ""]);
  expect(rowCells("Jüpiter")).toEqual([full, full, "", full, "—", "", "", "", ""]);
  expect(rowCells("Satürn")).toEqual(["", full, "", "", "", "", "—", "", full]);
  // İşaret ekran okuyucudan gizlidir; "tam bakış" sözcükle okunur.
  expect(document.querySelectorAll('td [aria-hidden="true"]')).toHaveLength(7);
  expect(screen.getByText(`${FULL_MARK} burç temelli tam bakış. Rahu ve Ketu'ya graha bakışı atanmaz.`)).toBeInTheDocument();
  expect(screen.getByText("Yöntem: graha_drishti.bphs_v1")).toBeInTheDocument();
  expect(document.querySelector('[title="yaklaşık"]')).toBeNull();
});

test("the virupa view shows the degree-based value with one decimal", async () => {
  renderPanel({ value: "virupa" });
  await screen.findByRole("table");
  expect(screen.getByRole("combobox", { name: "Değer" })).toHaveDisplayValue("Virupa");
  expect(rowCells("Güneş")).toEqual(["—", "0.0", "8.8", "0.0", "37.6", "0.0", "30.0", "47.7", "0.0"]);
  expect(rowCells("Ay")[4]).toBe("23.9");
  expect(document.querySelector("td [aria-hidden]")).toBeNull();
  expect(screen.getByText("Derece temelli bakış değeri (sphuta drishti), virupa: 0–60.")).toBeInTheDocument();
});

test("the value setting is saved with the layout; unknown values fall back to the full mark", async () => {
  const { persistence } = renderPanel();
  fireEvent.change(await screen.findByRole("combobox", { name: "Değer" }), { target: { value: "virupa" } });
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  expect(persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ value: "virupa" });
  // Ayar değişimi yeniden hesap istemez.
  expect(api.computeDrishti).toHaveBeenCalledTimes(1);
  expect([drishtiValue({}), drishtiValue({ value: "rashi" }), drishtiValue({ value: "virupa" })]).toEqual(["full", "full", "virupa"]);
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeDrishti).not.toHaveBeenCalled();
});

test("unknown birth time is sent to Rust and the Moon is marked approximate", async () => {
  renderPanel({}, unknownTimeOpened());
  await screen.findByRole("table");
  expect(vi.mocked(api.computeDrishti).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByText("Doğum saati bilinmiyor: Ay'ın konumu ve bakışları yaklaşıktır (≈).")).toBeInTheDocument();
  // Ay'ın satır ve sütun başlığı işaretlidir; başka graha değil.
  expect(document.querySelectorAll('[title="yaklaşık"]')).toHaveLength(2);
  expect(rowHeader("Ay").closest("th")!.querySelector('[title="yaklaşık"]')).not.toBeNull();
  expect(rowHeader("Güneş").closest("th")!.querySelector('[title="yaklaşık"]')).toBeNull();
});

test("an error is shown in the panel and retry asks again", async () => {
  vi.mocked(api.computeDrishti).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.queryByRole("table")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await screen.findByRole("table");
  expect(api.computeDrishti).toHaveBeenCalledTimes(2);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("loading: nothing under a second, then a skeleton, then the table", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<DrishtiTable>) => void;
    vi.mocked(api.computeDrishti).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.queryByRole("table")).toBeNull();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getByRole("table")).toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
});

test("shared highlight: a graha marks its row and its column; nodes have a column only", async () => {
  renderPanel();
  await screen.findByRole("table");
  const marked = () => document.querySelectorAll("td.is-highlighted").length;
  fireEvent.click(rowHeader("Jüpiter"));
  expect(rowHeader("Jüpiter")).toHaveAttribute("aria-pressed", "true");
  expect(rowHeader("Jüpiter").closest("tr")).toHaveClass("is-highlighted");
  expect(document.querySelector('thead th[data-graha="Jupiter"]')).toHaveClass("is-highlighted");
  // Sütun: yedi satırın Jüpiter hücresi.
  expect(marked()).toBe(7);
  // Sütun başlığı kısaltmadır; erişilebilir adı grahanın tam adıdır.
  const ketu = within(document.querySelector("thead")!).getByRole("button", { name: "Ketu" });
  expect(ketu).toHaveTextContent("Ke");
  fireEvent.click(ketu);
  expect(document.querySelector("tbody tr.is-highlighted")).toBeNull();
  expect(document.querySelector('thead th[data-graha="Ketu"]')).toHaveClass("is-highlighted");
  expect(marked()).toBe(7);
  fireEvent.keyDown(window, { key: "Escape" });
  expect(marked()).toBe(0);
});

test("helpers: full-aspect pairs come from the occupants, virupa text has one decimal", () => {
  const pairs = fullAspectPairs(fixture().drishti);
  expect([...pairs].sort()).toEqual(["Jupiter>Mercury", "Jupiter>Moon", "Jupiter>Sun", "Moon>Jupiter", "Moon>Saturn", "Saturn>Ketu", "Saturn>Moon"]);
  expect([formatVirupa(0), formatVirupa(60), formatVirupa(37.55), formatVirupa(null), formatVirupa(Number.NaN)]).toEqual(["0.0", "60.0", "37.5", "—", "—"]);
  expect(ASPECTING_GRAHAS).toEqual(["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"]);
});
```

```diff
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index a991b7a..d82f400 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -11,6 +11,7 @@ import { TransitTablePanel } from "../transit/TransitTablePanel";
 import { AshtakavargaPanel } from "./AshtakavargaPanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel } from "./ChartPanel";
+import { DrishtiPanel } from "./DrishtiPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
 import { PanchangPanel } from "./PanchangPanel";
 import { panelDefinition, PANEL_REGISTRY } from "./registry";
@@ -38,6 +39,9 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.transit_table.component).toBe(TransitTablePanel);
   expect(PANEL_REGISTRY.panchang.component).toBe(PanchangPanel);
   expect(PANEL_REGISTRY.ashtakavarga.component).toBe(AshtakavargaPanel);
+  expect(PANEL_REGISTRY.drishti.component).toBe(DrishtiPanel);
+  // Kataloğun her türü uygulanmıştır: yer tutucu yoktur.
+  for (const kind of PANEL_KINDS) expect(PANEL_REGISTRY[kind].component, kind).not.toBe(UnavailablePanel);
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
   const keys = new Set(Object.keys(tr.panel).map((k) => `panel.${k}`));
   for (const kind of [...PANEL_KINDS, "yoga_list"]) expect(keys.has(panelDefinition(kind).titleKey)).toBe(true);
diff --git a/app/ui/App.test.tsx b/app/ui/App.test.tsx
index 2f79cf5..ee2b776 100644
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -4,6 +4,7 @@ import { vi } from "vitest";
 vi.mock("./api", () => ({
   api: {
     transitsAt: vi.fn(async () => ({ status: "ok", data: (await import("./test/fixture")).fixture().transit })),
+    computeDrishti: vi.fn(async () => ({ status: "ok", data: (await import("./test/fixture")).fixture().drishti })),
     printWindow: vi.fn(async () => ({ status: "ok", data: null })),
     previewMoment: vi.fn(async () => ({ status: "error", error: { code: "invalid_input", params: { field: "birth.date" } } })),
     searchPlaces: vi.fn(async () => ({ status: "ok", data: [] })),
@@ -194,6 +195,9 @@ test("the palette adds a module panel below the layout and focuses it; again rem
   expect(panel()).toHaveAttribute("data-x", "0");
   expect(panel()).toHaveAttribute("data-w", "6");
   await waitFor(() => expect(panel()).toHaveFocus());
+  // Panel açılınca bakış tablosunu ister ve çizer.
+  expect(await within(panel()!).findByRole("table", { name: /Graha bakışları/ })).toBeInTheDocument();
+  expect(api.computeDrishti).toHaveBeenCalledTimes(1);
   // Diğer paneller yerinde.
   expect(screen.getByRole("region", { name: "Harita D1" })).toHaveAttribute("data-y", "0");
   await run();
```

Run: `pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "`
Expected: `Test Files  3 failed | 62 passed (65)`.

- [ ] **Step 2: Panel, kayıt, stil ve metinler**

**Yeni dosya:** `app/ui/panels/DrishtiPanel.tsx`

```tsx
// Bakışlar paneli (ileri modüller spec'i §4): bakan yedi graha × bakılan dokuz graha matrisi.
// Değerler `compute_drishti`'den gelir ve panel açıkken istenir (K1); burada hesap yoktur.
// "Tam bakış işareti" görünümü burç temelli tam bakışları `●` ile, "virupa" görünümü derece
// temelli bakış değerini sayı olarak gösterir; renk anlam taşımaz. Rashi drishti türü Jaimini
// alt projesiyle gelir. Ortak vurgu: bir graha vurgulanınca satırı ve sütunu `--select` alır.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { DrishtiTable, Graha, OpenedClient } from "../bindings";
import { GRAHA_IDS, type GrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { sameTarget, type HighlightTarget } from "../highlight/target";
import type { Panel } from "../layout/model";
import { chartInput, inputKey } from "../modules/input";
import { useModule } from "../modules/useModule";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import { Approx } from "./Approx";
import "./panels.css";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Bakan grahalar: `DrishtiTable.sphuta` satır sırası (Rust `ASPECTING_GRAHAS`). */
export const ASPECTING_GRAHAS = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"] as const satisfies readonly GrahaId[];
export const DRISHTI_VALUES = ["full", "virupa"] as const;
export type DrishtiValue = (typeof DRISHTI_VALUES)[number];
export const FULL_MARK = "●";
const DASH = "—";

export function drishtiValue(settings: Record<string, unknown>): DrishtiValue {
  return settings.value === "virupa" ? "virupa" : "full";
}

/** Burç temelli tam bakışı olan (bakan, bakılan) çiftleri: bakılan graha hedef burçtadır. */
export function fullAspectPairs(table: DrishtiTable): Set<string> {
  const pairs = new Set<string>();
  for (const a of table.sign_aspects) for (const target of a.occupants) pairs.add(`${a.graha}>${target}`);
  return pairs;
}

/** Virupa değeri, bir ondalık; sonlu olmayan değer çizgi. */
export function formatVirupa(v: number | null): string {
  return v === null || !Number.isFinite(v) ? DASH : v.toFixed(1);
}

export function DrishtiPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <DrishtiMatrix panel={panel} opened={opened} />;
}

function DrishtiMatrix({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const highlight = useHighlight();
  const id = useId();
  const value = drishtiValue(panel.settings);
  const module = useModule(inputKey(opened), () => api.computeDrishti(chartInput(opened)));
  const phase = usePanelLoadPhase(module.loading);

  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  const table = module.data;
  if (table === null) return phase === "skeleton" ? <PanelSkeleton /> : null;

  const unknown = timeUnknown(opened);
  const full = fullAspectPairs(table);
  const active = highlight.active;
  const on = (g: Graha) => active?.type === "graha" && active.id === g;
  const header = (g: GrahaId, short: boolean) => {
    const target: HighlightTarget = { type: "graha", id: g };
    return (
      <Approx on={unknown && g === "Moon"} label={t("table.approx")}>
        <button
          type="button"
          className="link-button"
          aria-label={short ? t(`grahaName.${g}`) : undefined}
          aria-pressed={sameTarget(highlight.pinned, target)}
          onClick={() => highlight.togglePin(target)}
          onMouseEnter={() => highlight.hover(target)}
          onMouseLeave={() => highlight.hover(null)}
          onFocus={() => highlight.hover(target)}
          onBlur={() => highlight.hover(null)}
        >
          {short ? t(`graha.${g}`) : t(`grahaName.${g}`)}
        </button>
      </Approx>
    );
  };

  return (
    <div className="module-panel">
      <div className="module-toolbar">
        <label htmlFor={`${id}-value`}>{t("drishti.value")}</label>
        <select id={`${id}-value`} value={value} onChange={(e) => setPanelSettings(panel.id, { value: e.target.value })}>
          {DRISHTI_VALUES.map((v) => (
            <option key={v} value={v}>
              {t(`drishti.values.${v}`)}
            </option>
          ))}
        </select>
      </div>
      {unknown && <p className="panel-notice">{t("drishti.timeUnknown")}</p>}
      <table className="module-table drishti-table">
        <caption className="visually-hidden">{t("drishti.caption")}</caption>
        <thead>
          <tr>
            <th scope="col">{t("drishti.aspecting")}</th>
            {GRAHA_IDS.map((g) => (
              <th key={g} scope="col" data-graha={g} className={on(g) ? "num is-highlighted" : "num"}>
                {header(g, true)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {ASPECTING_GRAHAS.map((from, i) => (
            <tr key={from} data-graha={from} className={on(from) ? "is-highlighted" : undefined}>
              <th scope="row">{header(from, false)}</th>
              {GRAHA_IDS.map((to, j) => {
                const self = from === to;
                const isFull = full.has(`${from}>${to}`);
                return (
                  <td key={to} className={on(to) ? "num is-highlighted" : "num"}>
                    {self ? (
                      DASH
                    ) : value === "virupa" ? (
                      formatVirupa(table.sphuta[i][j])
                    ) : isFull ? (
                      <>
                        <span aria-hidden="true">{FULL_MARK}</span>
                        <span className="visually-hidden">{t("drishti.full")}</span>
                      </>
                    ) : (
                      ""
                    )}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
      <p className="panel-message">{value === "full" ? t("drishti.legendFull", { mark: FULL_MARK }) : t("drishti.legendVirupa")}</p>
      <p className="panel-message">{t("drishti.method", { method: table.method })}</p>
    </div>
  );
}
```

```diff
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index bcddccb..5584100 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -7,6 +7,7 @@ import { AshtakavargaPanel } from "./AshtakavargaPanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel, chartVarga } from "./ChartPanel";
 import { TransitTablePanel } from "../transit/TransitTablePanel";
+import { DrishtiPanel } from "./DrishtiPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
 import { PanchangPanel } from "./PanchangPanel";
 import type { PanelDefinition } from "./types";
@@ -23,7 +24,7 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   // İleri modül panelleri (Plan 14).
   panchang: { titleKey: "panel.panchang", component: PanchangPanel },
   ashtakavarga: { titleKey: "panel.ashtakavarga", component: AshtakavargaPanel },
-  drishti: { titleKey: "panel.drishti", component: UnavailablePanel },
+  drishti: { titleKey: "panel.drishti", component: DrishtiPanel },
 };
 
 const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };
diff --git a/app/ui/panels/panels.css b/app/ui/panels/panels.css
index 66318ac..977f060 100644
--- a/app/ui/panels/panels.css
+++ b/app/ui/panels/panels.css
@@ -182,3 +182,7 @@
   border-right: 1px solid var(--line);
   border-bottom: 1px solid var(--line);
 }
+.drishti-table th,
+.drishti-table td {
+  padding: 2px 4px;
+}
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index a8bf25b..aaa14d3 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -102,6 +102,17 @@ export const tr = {
     totalOf: "{{view}} toplamı {{total}}",
     timeUnknown: "Doğum saati bilinmiyor: ashtakavarga lagnaya da bağlı olduğu için gösterilmiyor.",
   },
+  drishti: {
+    value: "Değer",
+    values: { full: "Tam bakış işareti", virupa: "Virupa" },
+    caption: "Graha bakışları: satırlar bakan, sütunlar bakılan graha",
+    aspecting: "Bakan",
+    full: "tam bakış",
+    legendFull: "{{mark}} burç temelli tam bakış. Rahu ve Ketu'ya graha bakışı atanmaz.",
+    legendVirupa: "Derece temelli bakış değeri (sphuta drishti), virupa: 0–60.",
+    method: "Yöntem: {{method}}",
+    timeUnknown: "Doğum saati bilinmiyor: Ay'ın konumu ve bakışları yaklaşıktır (≈).",
+  },
   table: {
     graha: "Graha",
     sign: "Burç",
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index b1026e4..e4a6c04 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -103,6 +103,17 @@ export const en: Messages = {
     totalOf: "{{view}} total {{total}}",
     timeUnknown: "Birth time unknown: ashtakavarga also depends on the lagna and is not shown.",
   },
+  drishti: {
+    value: "Value",
+    values: { full: "Full aspect mark", virupa: "Virupa" },
+    caption: "Graha aspects: rows aspect, columns are aspected",
+    aspecting: "Aspecting",
+    full: "full aspect",
+    legendFull: "{{mark}} full aspect by sign. Rahu and Ketu are given no graha aspect.",
+    legendVirupa: "Aspect value by degree (sphuta drishti), in virupas: 0–60.",
+    method: "Method: {{method}}",
+    timeUnknown: "Birth time unknown: the Moon's position and aspects are approximate (≈).",
+  },
   table: {
     graha: "Graha",
     sign: "Sign",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 11d5481..ad5a209 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -103,6 +103,17 @@ export const fr: Messages = {
     totalOf: "Total {{view}} : {{total}}",
     timeUnknown: "Heure de naissance inconnue : l'ashtakavarga dépend aussi du lagna et n'est pas affiché.",
   },
+  drishti: {
+    value: "Valeur",
+    values: { full: "Marque d'aspect plein", virupa: "Virupa" },
+    caption: "Aspects des grahas : les lignes aspectent, les colonnes sont aspectées",
+    aspecting: "Aspectant",
+    full: "aspect plein",
+    legendFull: "{{mark}} aspect plein par signe. Rahu et Ketu ne reçoivent pas d'aspect de graha.",
+    legendVirupa: "Valeur d'aspect par degré (sphuta drishti), en virupas : 0–60.",
+    method: "Méthode : {{method}}",
+    timeUnknown: "Heure de naissance inconnue : la position et les aspects de la Lune sont approximatifs (≈).",
+  },
   table: {
     graha: "Graha",
     sign: "Signe",
```

- [ ] **Step 3: Kapılar**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | tail -1
grep -c "component: UnavailablePanel" app/ui/panels/registry.ts
```
Expected: `Test Files  65 passed (65)`, `Tests  565 passed (565)` (+9); `✓ built`; son komut `1` yazar (yer tutucu yalnızca bilinmeyen türün yedeğidir).

- [ ] **Step 4: Commit**

```bash
git add app/ui/panels app/ui/i18n app/ui/App.test.tsx
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): aspects (drishti) panel"
```
Expected: iki `grep` çıktısız.

---

### Task 9: Uçtan uca: modül panelleri

Gerçek WebKit'te (IPC taklidiyle) tek akış: panel açılmadan modül komutu çağrılmaz (K1); palet üç paneli ekler, odak eklenen panele geçer ve panel görünür alandadır; bakış tablosu bir kez istenir (`StrictMode` altında da) ve doğru girdiyle; "virupa" görünümü yeniden hesap istemez; ortak vurgu bakış tablosundan haritaya ve ashtakavarga satırına yayılır; paneller düzenle birlikte kaydedilir; aynı komut paneli kaldırır.

**Files:**
- Modify: `app/e2e/desk.spec.ts`
- Test: aynı dosya

**Interfaces:**
- Consumes: `mockIpc`, `ipcCalls` (`e2e/ipc.ts`; `compute_drishti` taklidi Task 4'te eklendi), Task 5–8'in arayüzü ve metinleri.
- Produces: e2e testi `module panels: …`.

- [ ] **Step 1: Testi yaz**

```diff
diff --git a/app/e2e/desk.spec.ts b/app/e2e/desk.spec.ts
index 9ac7ad7..763e35e 100644
--- a/app/e2e/desk.spec.ts
+++ b/app/e2e/desk.spec.ts
@@ -263,3 +263,61 @@ test("updates: nothing is checked until asked; About checks, shows the notes and
   await expect(page.getByRole("region", { name: "Bildirimler" })).toContainText("Yeni sürüm var: 0.9.1.");
   expect(await ipcCalls(page, "check_for_update")).toHaveLength(1);
 });
+
+test("module panels: the palette adds Aspects, Panchang and Ashtakavarga; aspects are computed once and share the highlight", async ({ page }) => {
+  await mockIpc(page, { session: true });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  const toggle = async (query: string, name: RegExp) => {
+    await page.keyboard.press("Meta+k");
+    const palette = page.getByRole("dialog", { name: "Komut paleti" });
+    await palette.getByRole("combobox").fill(query);
+    await palette.getByRole("option", { name }).click();
+    await expect(page.getByRole("dialog")).toHaveCount(0);
+  };
+  // Modül sonucu panel açılmadan istenmez (ileri modüller spec'i K1).
+  expect(await ipcCalls(page, "compute_drishti")).toHaveLength(0);
+
+  await toggle("bakış", /Bakışlar panelini aç\/kapat/);
+  const drishti = page.getByRole("region", { name: "Bakışlar" });
+  await expect(drishti).toBeFocused();
+  await expect(drishti).toBeInViewport();
+  await expect(drishti.getByRole("table")).toBeVisible();
+  const calls = (await ipcCalls(page, "compute_drishti")) as { input: { jd_ut: number; time_known: boolean; settings: { schema_version: number } } }[];
+  expect(calls).toHaveLength(1);
+  expect(calls[0].input).toMatchObject({ jd_ut: 2451545, time_known: true, settings: { schema_version: 2 } });
+  // Jüpiter Koç'tan Yay'daki Güneş'e tam bakar; virupa görünümü sayıyı gösterir.
+  const jupiter = drishti.locator('tbody tr[data-graha="Jupiter"]');
+  await expect(jupiter.locator("td").first()).toContainText("●");
+  await drishti.getByRole("combobox", { name: "Değer" }).selectOption("virupa");
+  await expect(jupiter.locator("td").first()).toHaveText("52.4");
+  expect(await ipcCalls(page, "compute_drishti")).toHaveLength(1);
+
+  await toggle("panchang", /Panchang panelini aç\/kapat/);
+  const panchang = page.getByRole("region", { name: "Panchang" });
+  await expect(panchang.locator('tr[data-row="tithi"]')).toContainText("Krishna 11 · Ekadashi");
+  await expect(panchang.locator('tr[data-row="tithi"]')).toContainText("2000-01-02 13:19");
+  await toggle("ashtakavarga", /Ashtakavarga panelini aç\/kapat/);
+  const ashtakavarga = page.getByRole("region", { name: "Ashtakavarga" });
+  await expect(ashtakavarga.locator('tr[data-row="sav"]')).toContainText("337");
+  await expect(ashtakavarga).toBeFocused();
+
+  // Ortak vurgu: bakış tablosundaki Jüpiter satırı haritada ve ashtakavarga satırında da vurgulanır.
+  await jupiter.getByRole("button", { name: "Jüpiter" }).hover();
+  await expect(page.locator('svg [data-graha="Jupiter"]').first()).toHaveClass(/chart-planet--highlight/);
+  await expect(ashtakavarga.locator('tr[data-graha="Jupiter"]')).toHaveClass(/is-highlighted/);
+
+  // Paneller düzenle birlikte kaydedilir.
+  await expect
+    .poll(async () => {
+      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: unknown }[] }[] } } } | undefined;
+      const layout = saved?.viewState.panel_layout;
+      return layout?.layouts.find((l) => l.id === layout.active_layout_id)?.panels.map((p) => p.kind);
+    })
+    .toEqual(["chart", "chart", "dasha_strip", "notes", "drishti", "panchang", "ashtakavarga"]);
+
+  // Aynı komut paneli kaldırır.
+  await toggle("bakış", /Bakışlar panelini aç\/kapat/);
+  await expect(drishti).toHaveCount(0);
+  await expect(panchang).toBeVisible();
+});
```

- [ ] **Step 2: Kapılar**

```bash
pnpm --dir app build 2>&1 | tail -1
pnpm --dir app e2e 2>&1 | tail -1
rm -rf app/test-results
```
Expected: `✓ built` (e2e dosyaları da `tsc` ile denetlenir); `13 passed`. 1421 portunda başka bir depo kopyasının Vite sunucusu açıksa Playwright onu yeniden kullanır ve eski arayüzü sınar: önce `lsof -i :1421` boş olmalıdır.

- [ ] **Step 3: Commit**

```bash
git add app/e2e/desk.spec.ts
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(e2e): module panels from the palette; aspects computed once; shared highlight"
```
Expected: `grep` çıktısız.

---

### Task 10: Son doğrulama ve elle denetim kapısı

Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle çözülür ve bu görev baştan koşulur.

**Files:**
- Yok (yalnızca doğrulama). İlerleme notları depo dışıdır; `git add` edilmez.

**Interfaces:**
- Consumes: Task 1–9'un tamamı.
- Produces: yeşil kapılar, kural taramaları, elle denetim sonucu ve sahibine devredilen maddeler (ilerleme notlarına yazılır).

- [ ] **Step 1: Rust kapıları**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test -p jyotish-desk --test modules -- --nocapture 2>&1 | grep median
cargo deny check 2>&1 | tail -1
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p jyotish-core -p store -p jyotish-desk --all-targets 2>&1 | tail -1
git diff cd95efe..HEAD --stat -- deny.toml Cargo.toml Cargo.lock app/package.json app/pnpm-lock.yaml app/src-tauri/capabilities app/src-tauri/tauri.conf.json crates/store/src/schema.rs
```
Expected: fmt ve clippy çıktısız; testler: `jyotish-core` lib `139 passed` (son inceleme düzeltmeleriyle; Task 9 sonunda 138), golden `8 passed`, `modules_golden` `3 passed`; `store` lib `84 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `46 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `modules` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; `compute_drishti median` 10 ms'nin altında; deny `advisories ok, bans ok, licenses ok, sources ok`; MSRV `Finished`; son `git diff --stat` çıktısız (bağımlılık, yetenek, yapılandırma ve SQLite şeması değişmedi).

- [ ] **Step 2: Arayüz kapıları**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | tail -1
pnpm --dir app e2e 2>&1 | tail -1
pnpm --dir app e2e:visual 2>&1 | tail -1
rm -rf app/test-results
node --test "scripts/release/*.test.mjs" 2>&1 | grep -E "ℹ (pass|fail)"
git status --short
```
Expected: `Test Files  65 passed (65)`, `Tests  571 passed (571)` (son inceleme düzeltmeleriyle; Task 9 sonunda 565); `✓ built`; e2e `13 passed`; görsel `10 passed` (temel PNG'ler değişmedi: D14); betik testleri `pass 14`, `fail 0`; `git status --short` boş.

- [ ] **Step 3: Kural taramaları (`cd95efe`'den bu yana)**

```bash
git diff cd95efe..HEAD -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log cd95efe..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff cd95efe..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff cd95efe..HEAD --name-only | grep -vE '^(app|crates|scripts|tests|docs)/'
echo "trailers: $(git log cd95efe..HEAD --format=%B | grep -c '^Co-Authored-By: ') commits: $(git rev-list --count cd95efe..HEAD)"
git diff cd95efe..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
git diff cd95efe..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\(|window\.(print|open)\('
git diff cd95efe..HEAD -U0 -- app/ui/panels/panels.css | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
grep -rn '"source"' tests/golden/modules | grep -v '"source": "hand"'
grep -rnE 'serde\(default' crates/jyotish-core/src/settings.rs crates/jyotish-core/src/positions.rs crates/jyotish-core/src/drishti.rs crates/jyotish-core/src/nature.rs
grep -rnE 'fetch\(|XMLHttpRequest|WebSocket' app/ui/modules app/ui/panels
git tag --points-at HEAD
```
Expected: `trailers:` satırındaki iki sayı eşit; bunun dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur): yasaklı ad yok, beklenmeyen yerde dosya yok, `console` ve yasak çağrı yok, yeni stilde hex ya da `rgb(` yok (C9), el örneklerinde `hand` dışında kaynak yok (K9), IPC tiplerinde `serde(default)` yok (D2), modül ve panel kodunda ağ çağrısı yok (C1), ajan etiket atmamış.

- [ ] **Step 4: Elle denetim kapısı (`pnpm --dir app tauri dev`)**

Gerçek uygulamada, deneme danışanlarıyla (gerçek danışan verisi kullanılmaz). Her madde ilerleme notlarına "geçti" ya da bulgu olarak yazılır; bulgu varsa düzeltme commit'i ve bu görev baştan. Her maddenin başında, otomatik bir karşılığı olup olmadığı yazılıdır. **[otomatik]** maddelerin karşılığı testlerde vardır ve yürütücü ajan bunları testin adını göstererek "geçti" sayabilir; **[insan]** maddeler yalnızca gerçek uygulamaya bakan bir insan tarafından doğrulanabilir ve sahibi yoksa ona devredilir (bu devir `git push`'u engellemez; hiçbir şey yayımlanmaz).

- [ ] **[otomatik]** `⌘K` → "bakış" → "Bakışlar panelini aç/kapat": panel etkin düzenin altına eklenir, odak ona geçer ve panel görünür alana kayar; aynı komut paneli kaldırır. Panchang ve Ashtakavarga için de aynısı. (e2e `module panels: …`; `App.test.tsx` "the palette adds a module panel…")
- [ ] **[otomatik]** Bakışlar paneli Delhi deneme haritasında yedi `●` gösterir; "Değer → Virupa" sayıları gösterir ve yeniden hesap istemez; Jüpiter satırına gelince haritadaki Jüpiter ve Ashtakavarga'daki Jüpiter satırı vurgulanır; `Esc` vurguyu kaldırır. (`DrishtiPanel.test.tsx`; e2e)
- [ ] **[otomatik]** Doğum saati "bilinmiyor" olan haritada: Panchang değerleri "≈" taşır ve uyarı satırı vardır; Ashtakavarga tablo yerine tek satır açıklama gösterir; Bakışlar uyarı satırı gösterir ve Ay'ın başlıkları "≈" taşır. (`PanchangPanel.test.tsx`, `AshtakavargaPanel.test.tsx`, `DrishtiPanel.test.tsx`)
- [ ] **[otomatik]** Sürüm 1 ayarlı harita açılır, uyarı çıkmaz, ayar sürüm 2 olarak geri yazılır. (`store` `version_1_settings_migrate_on_open_and_unknown_advanced_keys_do_not_block`)
- [ ] **[insan]** Bu plandan önce oluşturulmuş gerçek veritabanıyla açılış: uygulama uyarı bandı göstermeden açılır; birkaç eski harita sırayla açılır; her birinde harita, dasa ve notlar öncekiyle aynıdır; "Ayar farkı" rozeti yalnızca daha önce de farklı olan haritalarda görünür. (Önce veritabanının bir kopyası alınır: `cp -p ~/Library/Application\ Support/io.github.turgaydogusan.jyotishdesk/jyotish-desk.sqlite* <yedek klasörü>/`.)
- [ ] **[insan]** Üç panel açık ve koyu temada, Türkçe, İngilizce ve Fransızca dilde okunur; 6 sütun genişliğinde Ashtakavarga tablosu ve Bakışlar matrisi yatay taşmaz, panel 4 sütuna daraltılınca içerik panel içinde kaydırılır; "harita içi" Ashtakavarga kutuları kare kutulara yakın görünür ve lagna burcu "Lg" ile seçilir.
- [ ] **[insan]** VoiceOver: Bakışlar matrisinde bir `●` hücresi "tam bakış" diye, sütun başlığı grahanın tam adıyla okunur; Panchang tablosu başlığıyla duyurulur; paletten eklenen panel adıyla duyurulur.
- [ ] **[insan]** Bilinen bir haritada (astrologun kendi elindeki bir örnek) Panchang beş uzvu ve bitiş saatleri, SAV satırı ve birkaç tam bakış, astrologun beklediğiyle uyuşur. Uyuşmazlık bulgu olarak yazılır; yöntem farkıysa (`method` satırı) "Bu planın kapsamadıkları"na göre sınıflanır.
- [ ] **[insan]** Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum verisi, konum ya da dosya yolu görünmez.

Kapı geçtiyse ilerleme notlarına `Plan 14 Task 10 elle denetim: geçti (<tarih>)` yazılır; **[insan]** maddeleri devredildiyse `Plan 14 Task 10 elle denetim: otomatik maddeler geçti, insan maddeleri sahibine devredildi (<tarih>)` yazılır. Bu adım commit üretmez.

- [ ] **Step 5: El örneklerinin yeniden türetilmesi (gözden geçirme maddesi)**

Spec §13: "`derivation` ayrı bir gözden geçirmede kuraldan yeniden hesaplanır." Gözden geçiren (motoru yazan dışında bir ajan ya da insan), `tests/golden/modules/foundation/` ve `tests/golden/modules/drishti/` altındaki on dosyanın her birinde girdiden başlayıp `rule_ref`'teki kuralı ve spec §4'teki tabloyu uygulayarak `expected` değerlerini motoru çalıştırmadan yeniden çıkarır ve farkları yazar. Fark varsa önce kural yeniden okunur; düzeltme hangi tarafta olursa olsun ayrı commit'tir. Sonuç ilerleme notlarına `Plan 14 el örnekleri yeniden türetildi: 10/10 (<tarih>)` olarak yazılır.

---

## Başarı ölçütleri

- `CalcSettings` sürüm 2'dir ve `advanced` bloğunu taşır; sürüm 1 kayıtları (harita, tercihler, içe aktarılan dosya) uyarısız göç eder; sürüm 2 kaydındaki bilinmeyen `advanced` anahtarı açılışı engellemez ve kaybolmaz. SQLite şeması değişmemiştir.
- `jyotish-core` ortak temeli verir: `ChartInput`, `Positions`, `compute_positions`, ev lordluğu, doğal sınıf, Kuja dosha, burç temelli tam bakış ve sphuta drishti. Hepsi el örnekleri (10 dosya, kaynak `hand`), birim testleri ve proptest değişmezleriyle doğrulanır; doğum saati bilinmiyorsa lagnaya bağlı çıktılar `None`'dır.
- `compute_drishti` ortak kalıpla çalışır (tek `ChartInput`, motor iş parçacığı, `blocking`, `CommandError`), v1 haritasıyla çapraz denetlenir ve bütçesinin (≤ 10 ms) içindedir.
- Üç panel (Panchang, Ashtakavarga, Bakışlar) paletten etkin düzene eklenir ve kaldırılır; ortak seçime bağlıdır; boş, yükleniyor, hata, dolu ve "saat bilinmiyor" durumları testlidir; renk tek başına anlam taşımaz; yorum metni yoktur.
- Kapılar: `jyotish-core` lib 139, `modules_golden` 3, `store` lib 84, `jyotish-desk` `modules` 1; Vitest 65/571 (son inceleme düzeltmeleri öncesinde 138 ve 565); e2e 13; görsel 10; `cargo deny` temiz; diğer sayılar tabandaki gibi. Elle denetim kapısının **[otomatik]** maddeleri testlerle doğrulanmış, **[insan]** maddeleri yapılmış ya da sahibine devredilmiştir.

## Bu planın kapsamadıkları

- **Spec'in 3.1 için kapsam dışı saydıkları:** düğüm bakışı varyantları, Tajika bakışları, bakış çizgilerinin harita üstüne çizimi.
- **Sonraki alt projeler:** rashi drishti ve Bakışlar panelinin tür seçicisi (3.2); `karaka_scheme` ve `jaimini_co_lords` ayarlarının hesapta kullanımı ve ayar penceresindeki denetimleri (3.2, 3.3); bhava ortasına sphuta drishti (3.4; fonksiyon hazırdır, çağıran 3.4'tür); Kuja dosha'nın yoga listesinde ve uyum panelinde gösterimi (3.5, 3.6); hafif konum yolu `search_facts` (3.9).
- Ashtakavarga'nın katkı dökümü (her referansın binduları), trikona ve ekadhipatya shodhana, kaksha tablosu (D11).
- Gün panchangı, gün dilimleri ve zaman çizelgesi (3.7); bu plandaki Panchang paneli yalnızca doğum anınınkidir.
- Genel bir "panel ekle" kataloğu, panel başlığında kapatma düğmesi, hazır düzenlere modül paneli eklenmesi (D9; beta geri bildirimi isterse ayrı iş).
- Modül panellerinin baskı düzenleri (K10); yeni görsel temeller (D14).
- Modül sonuçlarının oturum içi önbelleği (D13) ve veritabanında saklanması (spec §16).
- Yorum metinleri ve AI yorum paneli; Windows ve Android.
- v1'den taşınan seyrek test dalgalanması (`NotesPanel.test.tsx › New note adds an empty note…`, yük altında): bu planın dosyalarına dokunmaz; ayrı bir bakım işidir.

## Spec'e geri bildirim (spec boşlukları)

Plan yazılırken bulunan ve Kararlar'da çözülen spec boşlukları. Dokuz madde son incelemede `2026-11-26-alt-proje-3-ileri-moduller-design.md`'ye işlenmiştir (K2, K3, K6, K7, yeni K12, §4 ve §5).

1. **K7 / §4 / §5 — `advanced` alanlarının hangi alt projede tanımlandığı.** §4 "sürüm 1 → 2 göçü varsayılanları yazar" der, §5 iki alanı 3.2'nin "yeni kalıcı verisi" sayar. Çözüm (D1): alanlar ve göç 3.1'de; tüketim ve ayar penceresi 3.2'de. §5'in "Gizlilik" satırı "yeni kalıcı veri yok; 3.1'de tanımlanan iki ayar alanı kullanılır" olmalı.
2. **K7 — toleransın yeri.** "Tüm alanları varsayılanlıdır ve bilinmeyen anahtarlar yok sayılır" cümlesi IPC tipi için değil saklanan kaydın okunması için geçerlidir (D2). Spec, toleransın `store` okuma yolunda olduğunu, IPC'de alanların zorunlu olduğunu ve tanınmayan seçenek değerinin ayarı sıfırladığını yazmalı.
3. **K3 ile §4 arasındaki `Positions` farkı; `lagna_lon`'un çerçevesi.** K3 kısa bir tip verir (`rasi_sign`, `deg_in_sign`, `lagna_sign` yok); §4'teki esastır. `lagna_lon`'un sidereal, `lagna_sign`'ın rasi çerçevesinde olduğu yazılmalı (D5).
4. **K6 — Ay'ın büyüme aralığının uçları.** "0°–180°" yerine "`[0°, 180°)`; tam 180° küçülen yarıdadır (v1 `paksha` sınırı)" (D6).
5. **Panellerin düzene nasıl girdiği tanımsız.** v1'de panel ekleme/kaldırma yoktur (yalnızca notlar için `⌘⇧N`); spec dokuz alt projede yeni paneller tanımlar ama bunların bir düzene nasıl ekleneceğini söylemez. Çözüm (D9): palet komutları ve `MODULE_PANELS`. Spec §2'ye ortak karar olarak eklenmeli (ör. "K12 — modül panelleri paletten eklenir ve kaldırılır").
6. **K2'nin `Chart`'tan okunan panellere uygulanamaması.** Panchang ve Ashtakavarga v1'den beri `Chart` içindedir ve K1 `Chart`'ı dondurur; "gizleme tek yerde, Rust'ta" kuralı bu ikisinde arayüzde uygulanır (D10). Spec §4'e istisna olarak yazılmalı.
7. **§4 Ashtakavarga: "12 burç × 8 satır; satır ve sütun toplamları".** Sekiz satırın yedi BAV + SAV olduğu ve satır toplamının gösterim toplamı olduğu yazılmalı; "tek graha BAV" görünümünün o grahanın tek satırı olduğu (katkı dökümü değil) belirtilmeli (D11).
8. **§4 "Paneller: boş, yükleniyor, hata, dolu ve saat bilinmiyor durumları".** Panchang ve Ashtakavarga komut çağırmadığı için "yükleniyor" ve "hata" durumları yoktur; bu durumlar yalnızca Bakışlar paneli içindir.
9. **§4 "Retro her yerde düz `R`'dir".** Üç panelin hiçbiri retro göstermez; cümle bu alt proje için boştur (kural v1 panellerinde sürer).

## Self-Review

**1. Spec kapsamı.**

| Gereksinim (ileri modüller spec'i) | Görev |
|---|---|
| §4 Kapsam: burç temelli tam bakış ve sphuta drishti | Task 3 |
| §4 Kapsam: ev lordluğu yardımcıları ve ortak `kuja_dosha` | Task 2 |
| §4 Kapsam: `ChartInput`, `Positions`, `CalcSettings` sürüm 2 göçü, modül komutlarının ortak kalıbı | Task 1, 2, 4 |
| §4 Kapsam: üç panel | Task 5–8 |
| §4 Yöntem: 7. bakış; Mars 4/8, Jüpiter 5/9, Satürn 3/10; düğümlere bakış atanmaz | Task 3 (`graha_aspect_houses`) |
| §4 Yöntem: sphuta drishti tablosu, özel ekleme, 60'ta kesme; bakan yalnızca yedi graha; bakılan herhangi bir boylam | Task 3 (`sphuta_drishti`) |
| §4 Yöntem: `house_lord`, `houses_owned`; Akrep Mars, Kova Satürn | Task 2 |
| §4 Yöntem: `kuja_dosha` (1, 2, 4, 7, 8, 12; iptal yok; `from_lagna: Option`) | Task 2 |
| §4 Veri modeli: `DrishtiTable`, `SignAspect`, `compute_drishti(input)` | Task 3, 4 |
| §4 Arayüz: `panchang` (beş uzuv, bitişler, gün doğumu/batımı, uyarı) | Task 6 |
| §4 Arayüz: `ashtakavarga` (görünüm, biçim, toplamlar) | Task 7 |
| §4 Arayüz: `drishti` (değer ayarı; `●`; renk yok); ortak seçim | Task 8 (tür seçici 3.2'de: D12) |
| §4 Test: sphuta sınır ve tam nokta değerleri; [0, 60] property | Task 3 |
| §4 Test: hedef burç kümeleri el tablosuyla; 7. bakış simetrik | Task 3 |
| §4 Test: göç; bilinmeyen `advanced` anahtarı | Task 1 |
| §4 Test: panel durumları | Task 6, 7, 8 |
| §4 Performans ≤ 10 ms | Task 4 |
| K1 (istek üzerine, `Chart` değişmez), K2 (`time_known`), K3 (saf çekirdek), K4, K5, K6 | Task 2, 4; Global Constraints |
| K7 (sürüm 2; fark rozeti) | Task 1 |
| K8 (hata yolu, parametre gizliliği), §14 | Task 4 (D16), C1 |
| K9, §13 (el örnekleri, değişmezler, çapraz denetim, komut katmanı) | Task 2, 3, 4, 10 Step 5 |
| K10 (baskı düzeni yok), K11 (panel dili) | D14; Task 6 (klasik adlar üç dilde aynı) |
| Ek A (adlandırma) | C13 taramaları |

Boşluk: yok. Spec'in 3.1 için istemediği ama planın eklediği tek şey panellerin paletten eklenmesidir (D9, spec boşluğu 5).

**2. Yer tutucu taraması.** "TODO", "TBD", "daha sonra", "uygun hata işleme" yok. Task 10'daki `<tarih>` denetimin yapıldığı günün, `<yedek klasörü>` sahibinin seçtiği klasörün yazılacağı yerdir. Her kod adımı tam dosya ya da `cd95efe` ağacına (ve önceki görevlere) karşı üretilmiş diff taşır.

**3. Tip tutarlılığı.**
- `AdvancedSettings { karaka_scheme, jaimini_co_lords }` ve `KarakaScheme::{Eight, Seven}` Task 1'de Rust'ta tanımlanır; `store::settings::fill_advanced` aynı anahtar adlarını `serde_json::to_value(AdvancedSettings::default())`'tan alır (elle yazılmış anahtar yoktur); TS `CalcSettings["advanced"]`, `settings/model.test.ts`, `ChartTitle.test.tsx` ve `modules/input.test.ts` aynı adları kullanır.
- `ChartInput { jd_ut, lat, lon, settings, time_known }` Task 2'de tanımlanır; Task 4'te komut girdisidir; TS'te `chartInput(opened)` aynı beş alanı kurar; `ui_fixture.rs`, `tests/modules.rs`, `DrishtiPanel.test.tsx` ve e2e aynı alanları denetler.
- `Positions` yöntem adları (`synthetic`, `without_lagna`, `point`, `sign_of`, `occupants`, `house_from`, `house_of`) Task 2'de tanımlanır; `nature.rs`, `drishti.rs`, `proptests.rs` ve `modules_golden.rs` aynı adlarla çağırır.
- `ASPECTING_GRAHAS` (Rust, 7 graha) ile `DrishtiPanel.tsx`'teki `ASPECTING_GRAHAS` aynı sıradadır; `sphuta[i][j]`'nin `j` dizini Rust'ta `Graha::ALL`, arayüzde `GRAHA_IDS` sırasıdır (ikisi de Güneş…Ketu; `chart/types.ts` bunu `satisfies readonly Graha[]` ile bağlar).
- `DRISHTI_METHOD = "graha_drishti.bphs_v1"` Rust'ta, `modules_golden.rs`'te, `tests/modules.rs`'te ve `DrishtiPanel.test.tsx`'teki "Yöntem: …" metninde aynıdır.
- `MODULE_PANELS` türleri (`panchang`, `ashtakavarga`, `drishti`) `PANEL_KINDS`, `PANEL_REGISTRY`, i18n `panel.*` ve e2e'deki bölge adlarıyla (Panchang, Ashtakavarga, Bakışlar) aynıdır; boyutlar (4 × 9, 6 × 11, 6 × 10) `reducer.test.ts`, `App.test.tsx` ve panel testlerindeki `Panel` nesnelerinde aynıdır.
- Panel ayar anahtarları: Ashtakavarga `view`, `format`; Bakışlar `value`. Testler aynı anahtarları kaydedilen düzende arar.
- i18n: `palette.togglePanel` "{{name}} panelini aç/kapat"; `App.test.tsx` ve e2e "Bakışlar panelini aç/kapat" metnini arar. `panchang.timeUnknown`, `ashtakavarga.timeUnknown`, `drishti.timeUnknown` metinleri testlerde birebir geçer.
- El örneği şeması: `Fixture { id, source, rule_ref, derivation, input, expected }`, `Input { lagna_lon, grahas }`; README, on JSON dosyası ve `modules_golden.rs` aynı alan adlarını kullanır.

**4. Kural denetimi.** C1 (`ChartInput` loglanmaz; panik iletisi taşınmaz), C2′ (komut dizgi almaz), C3 (`schema.rs` değişmez), C4 (bakış, panchang ve bindu Rust'tan; biçimleme ve tablo toplamı gösterim), C7 (kısayol eklenmez), C9 (`●`, "Lg", "≈" ve metin; yeni stilde hex yok), C10 (iki üretilen dosya komutla), C12 (her yeni anahtar üç dilde), C13 (her commit'te tarama), K9 (yalnızca `hand`) görevlerin adımlarında ve Task 10 Step 3'te denetlenir.

**5. Koşulan doğrulama.** Bu plandaki kod, deponun `cd95efe` üzerindeki bir kopyasında dokuz commit olarak uygulanmış ve her commit'te ölçülmüştür: Vitest 527 → 527 → 541 → 549 → 556 → 565; `jyotish-core` lib 111 → 125 → 137 → 138; `store` lib 84; `modules_golden` 2 → 3; `jyotish-desk` `modules` 1 (ortanca 0,6 ms). Son durumda `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`, `cargo deny check` (dört `ok`), 1.93 ile `cargo check`, `pnpm build`, e2e 13, görsel 10 ve yayın betiği testleri 14 yeşildir; Task 10 Step 3'ün taramaları çıktısızdır. Ayrıca bu belgedeki bloklar (yeni dosyalar ve diff'ler) belgeden okunup sırayla temiz bir `cd95efe` ağacına uygulanmış ve her görevin sonunda o commit'in ağacıyla birebir aynı sonucu vermiştir (üretilen iki dosya dışında; onları adımlardaki komutlar üretir). El örneklerinin onu da ilk koşuda geçmiştir; yazım sırasında bir birim testinin yorumundaki burç yanlış yazılmış (200° Akrep sanılmış, doğrusu Terazi), kural yeniden okunarak girdi düzeltilmiştir (beklenen değer değil). **Koşulmayanlar:** `pnpm tauri dev` ile gerçek uygulama ve gerçek veritabanı (Task 10 Step 4'ün **[insan]** maddeleri); el örneklerinin bağımsız bir gözden geçirenle yeniden türetilmesi (Task 10 Step 5).
