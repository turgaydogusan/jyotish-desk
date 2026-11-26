# Plan 16 — Alt proje 3.3: Chara dasha (padakrama düzeni; dasa ağacında ve şeridinde sistem seçimi)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Burç temelli Jaimini dashasını (chara dasha, padakrama düzeni) vermek: yön, on iki burcun yılları, iki çevrim, üç seviye dönem; üç modül komutu (`compute_chara_dasha`, `chara_dasha_children`, `chara_dasha_at`); dasa ağacı ve dasa şeridi panellerinde `system` ayarı (`vimshottari | chara`); açık dalın sistem başına ayrı saklanması.

**Architecture:**
- **Saf çekirdek, ince toplayıcı (spec K3).** `jyotish-core`'un `jaimini` modülüne `chara_dasha` alt modülü eklenir. Hesap yalnızca `Positions`, eş lord ayarı, doğum anı ve yıl uzunluğunu görür; Plan 15'in `counts_forward`, `directed_count` ve `jaimini_lord` fonksiyonlarını kullanır. El örnekleri ephemeris olmadan `Positions::synthetic` ile kurulur. Üç toplayıcı temel haritayı hesaplayıp saf fonksiyonlara verir.
- **Üç komut, Plan 14'ün kalıbıyla (spec K1, K2).** Tek `ChartInput` (ve yol, çevrim ya da an), motor iş parçacığı, `blocking` havuzu, `CommandError`. `Chart` ve `open_client` değişmez; sonuç saklanmaz. Doğum saati bilinmiyorsa dasha Rust'ta `None` döner.
- **Arayüz hesap yapmaz.** Dönemler, yön, yıllar ve transit anının dönem yolu Rust'tan gelir. Dasa ağacının çizimi ve klavyesi ortak bir `PeriodTree` bileşenine alınır; Vimshottari ve chara ağaçları aynı bileşeni besler. Yeni panel türü yoktur: iki mevcut panel `system` ayarı kazanır.
- **Kalıcı biçim:** `ViewState` yeni bir alan kazanır (`chara_expanded_path`); şema sürümü, SQLite şeması ve `CalcSettings` değişmez. Alanı taşımayan eski kayıt Vimshottari yolu sayılır.
- **Testler:** el örnekleri (`tests/golden/modules/chara_dasha/`, 7 dosya, kaynak yalnızca `hand`), elle yazılmış 144 hücrelik yıl tablosu (bağımsız ikinci yazım), proptest değişmezleri, komut gövdelerinin gerçek motorla entegrasyon testi ve bütçe ölçümü, `store` ayrıştırma testleri, Vitest (iki panel, görünüm modeli), Playwright e2e.

**Tech Stack:** Rust 1.93+ (stable), Tauri 2.12, tauri-specta 2.0.0-rc.29, serde, proptest, React 19, TypeScript 7, Vitest 5, Playwright 1.65.1. **Yeni Rust ya da npm bağımlılığı yok.**

**Spec:** `docs/superpowers/specs/2026-11-26-alt-proje-3-ileri-moduller-design.md` (commit `ac2b412`; Plan 15'in geri bildirimi işlenmiş hâli): §2 (K1–K12), §5 "Sayım yönü" ve "Eş lordlar", §6 (Alt proje 3.3), §13 (ortak test stratejisi), §14 (gizlilik), §15 (bütçeler), Ek A (adlandırma). UI spec: `2026-11-24-ui-design.md` §3.3, §3.4, §5, §6, §8.

**Ön koşul:** Taban commit `ac2b412` (Plan 15 ve son inceleme düzeltmeleri `main`'de). `main` bu commit'ten ilerlediyse Task 1 adım 1 tabanları yeniden ölçer ve sayaç zinciri o ölçümden yürütülür (farklar aynıdır). Plan 15'in elle denetim kapısının yalnızca insana ait maddeleri sahibine devredilmiştir; bu planı engellemez.

## Kararlar

Sahibi kararları devretti; muhafazakâr seçenek seçildi ve her karar bir satır gerekçeyle yazıldı. "Spec boşluğu" ya da "Spec kusuru" diye işaretlenenler spec'te düzeltilmesi gereken yerlerdir ve belgenin sonundaki "Spec'e geri bildirim" bölümünde toplanmıştır. D4'ün sınırı, D18 ve sayaç zinciri bu plan yazılırken deponun bir kopyasında ölçülmüştür (macOS 27.2, Rust 1.99).

- **D1 — Chara dasha `crates/jyotish-core/src/jaimini/chara_dasha.rs` dosyasıdır ve `jaimini` modülünden dışa aktarılır; `JaiminiReport`'a girmez. `method = "chara_dasha.padakrama_v1"` (`CHARA_DASHA_METHOD`).** Gerekçe: kural 3.2'nin `rashi` ve `lords` alt modüllerini kullanır; spec §6 ayrı komut ve ayrı tip verir, K1 modül sonuçlarını ayrı tutar.
- **D2 — Yön ve yıl iki küçük saf fonksiyondur: `chara_dasha_forward(lagna)` (lagnadan 9. burç düz sayan gruptaysa ileri) ve `chara_dasha_years(sign, lord_sign)` (`directed_count − 1`; 0 ise 12). Yıl kuralı testte elle yazılmış 12 × 12 tabloyla (144 hücre) karşılaştırılır.** Gerekçe: spec §6 "Çapraz denetim" ve §13 katman 3 (bağımsız ikinci yazım).
- **D3 — Burcun lordu `jaimini_lord(sign, positions, co_lords)` ile bulunur (ayar kapalıyken Mars ve Satürn). "İki eş lord da burcun içindeyse 12 yıl" ayrı bir dal değildir: seçilen lord zaten burcun içindedir ve sayım 1'dir.** Gerekçe: spec §6 "`stronger_co_lord` ile seçilir" der; bu harfiyen uygulanırsa `jaimini_co_lords = false` ayarı chara dashada yok sayılırdı (K7 ayarın 3.3'te de kullanıldığını söyler). **Spec kusuru 1.**
- **D4 — İkinci çevrimin bir dashası, başlangıcı doğumdan sonraki 120. yıldan önceyse (birikimli yıl `< 120`) bütün süresiyle üretilir; tam 120. yılda başlayacak dasha ve sonrakiler üretilmez. İlk çevrimin on iki dashası her zaman üretilir. Karşılaştırma tamsayı yıllarla yapılır (`CHARA_DASHA_HORIZON_YEARS = 120`).** Gerekçe: spec "120 yıl sonrasını geçen ilk dashada durur" der; tam 120'de biten dashadan sonra ne olacağı yazılmamış. Seçilen okuma Vimshottari gibi en az 120 yılı kapsar ve ufkun içinde hiçbir anı olmayan dönem üretmez. İlk çevrim 120 yıla ulaşamaz (lordların iki burçtaki yılları birlikte sınırlıdır; el hesabıyla üst sınır 118, proptest `< 120` olduğunu sabitler), yani "on iki burç birer kez gelir" ile çelişki doğmaz. **Spec boşluğu 2.**
- **D5 — Alt dönemler ebeveyni on iki eşit parçaya böler; sınırlar `başlangıç + süre × k / 12`'dir ve sonuncusu ebeveynin bitişine tam eşittir. Sıra haritanın yönündedir (dasha burcunun kendi grubu değil): `sub_period_signs(sign, forward)`. `CharaDasha.periods` iki çevrimin 1. ve 2. seviyesini ön-sırada taşır (her dashanın ardından on iki alt dönemi; Vimshottari'nin `Chart.dasha.periods` düzeni); 3. seviye `sign_dasha_children` ile istenir. `years` yalnızca ilk çevrimin yıllarıdır.** Gerekçe: spec `// 2 seviye` der ama sırayı ve iki çevrimin birlikte geldiğini yazmaz; arayüz aynı gruplama kalıbını kullanır. **Spec boşluğu 3.**
- **D6 — Dasha sınırları `doğum anı + birikimli tamsayı yıl × settings.dasha_year` olarak hesaplanır (bir dashanın bitişi ile sonrakinin başlangıcı aynı ifadedir: boşluk ya da örtüşme olmaz). İlk dasha doğum anında tam süresiyle başlar.** Gerekçe: spec §6 "Başlangıç". `dasha_year` alanının adı ve ayar penceresindeki etiketi ("Dasa yılı") iki sistemi de kapsar; ayar tipi değişmez.
- **D7 — Üç komut vardır. Spec'teki ikisi: `compute_chara_dasha(input) -> Option<CharaDasha>` ve `chara_dasha_children(input, path, cycle) -> Vec<SignDashaPeriod>`. Üçüncüsü bu planın eklediğidir: `chara_dasha_at(input, jd_ut) -> Option<SignDashaPath>` (anı içeren dönemlerin yolu: çevrim ve üç burç).** Gerekçe: dasa şeridi "güncel dönem"i işaretler; C4 ve Plan 11 D10 arayüzün tarih karşılaştırmasını yasaklar (Vimshottari'de yol `transits_at`'ten gelir). Spec §6 chara dasha için böyle bir yol vermez. **Spec kusuru 4.**
- **D8 — `chara_dasha_children`: boş `path` o çevrimin dashalarını, bir burç on iki alt dönemi, iki burç on iki 3. seviye dönemi verir; `cycle` 1 ya da 2'dir. Geçersiz çevrim, üç burçluk yol ve o çevrimde üretilmemiş dönem `CoreError::InvalidDashaInput("cycle" | "path")`, sonlu olmayan an `InvalidDashaInput("jd_ut")` olur (kod `invalid_dasha_input`, üç dilde çevrilidir). Doğum saati bilinmiyorsa `compute_chara_dasha` ve `chara_dasha_at` `None`, `chara_dasha_children` boş liste döner; ayar, an ve konum denetimi (`compute_chart`) saat bilinmese de yapılır.** Gerekçe: K2 "lagnaya bağlı her çıktı `None`"; `Vec` dönen komutta bunun karşılığı boş listedir. Yeni `CommandError.code` yoktur. **Spec boşluğu 5.**
- **D9 — `CharaDasha` spec §6'daki alanlarla birebirdir; `warnings` alanı yoktur.** Gerekçe: tip AI paketleyicisinin okuyacağı sözleşmedir; temel haritanın `LowPrecision` uyarısı harita başlığındaki rozette zaten görünür. K8 "ölümcül olmayan durumlar `warnings` alanındadır" der ama §6'nın tipi alanı taşımaz. **Spec boşluğu 6.**
- **D10 — Açık dal sistem başına ayrı tutulur: `ViewState.dasha_expanded_path` Vimshottari yolu olarak durur, yanına `chara_expanded_path: SignDashaPath { cycle, signs }` eklenir. `VIEW_STATE_SCHEMA_VERSION` 1 olarak durur: okuma toleranslıdır, alanı taşımayan kayıt boş chara yolu (çevrim 1) verir; geçersiz çevrim, geçersiz burç ya da üçten uzun yol bütünüyle boş yola döner. Yazımda `check_view_state` aynı sınırları denetler. JSON dışa/içe aktarma iki yolu da taşır.** Gerekçe: spec "sistem başına ayrı tutulur (eski kayıt Vimshottari yolu sayılır)" der ama biçimi vermez; kardeş alan eski kaydı olduğu gibi geçerli bırakır ve göç gerektirmez. Chara yolu çevrimi de taşımalıdır: aynı burç iki çevrimde iki ayrı dönemdir. **Spec boşluğu 7.**
- **D11 — `system` panelin ayarıdır (`panel.settings.system`: `"vimshottari" | "chara"`; tanınmayan değer ve eksik anahtar Vimshottari). Yeni panel türü, `MODULE_PANELS` girdisi, palet komutu ve ayar penceresi denetimi eklenmez.** Gerekçe: spec §6 sistemi iki mevcut panelin ayarı yapar; hazır "Araştırma" düzeni ayarı v1'den beri `system: "vimshottari"` olarak taşır. K7 "3.3 yeni denetim eklemez". K12'nin "her alt proje panelini `MODULE_PANELS`'e ekler" cümlesi yeni panel türü getirmeyen 3.3 için boştur. **Spec boşluğu 8.**
- **D12 — Dasa ağacının çizimi ve klavyesi `dasha/PeriodTree.tsx`'e alınır; `DashaTreePanel` sistem seçicisini çizer ve Vimshottari ağacını ya da `CharaDashaTree`'yi gösterir. Vimshottari ağacının davranışı değişmez: mevcut on dört testi olduğu gibi geçer.** Gerekçe: iki ağaç aynı WAI-ARIA desenini ve aynı odak kurallarını ister; kopyalamak yerine tek bileşen (düğüm anahtarı virgüllü yoldur: üst düğüm son parça atılınca çıkar).
- **D13 — Chara ağacı: başlık satırı yönü ("ileri" / "geri") ve on iki burcun yıllarını dasha sırasıyla gösterir; satırlar burç kısaltması taşır (`sign.N` anahtarları); ikinci çevrimin dashaları "2." önekiyle ayrılır ve erişilebilir adda da önek vardır; altta "Yöntem: chara_dasha.padakrama_v1" satırı durur. Tek açık yol vardır (`ViewState.chara_expanded_path`); 3. seviye yalnızca açık yol için `chara_dasha_children` ile istenir (`useModule`; ayrı önbellek yoktur). Yüklenen liste istenen yolun bir alt seviyesi değilse yok sayılır; boş yanıt hatadır (`dasha_empty`). Bu haritada bulunmayan kayıtlı bir yol hiçbir şey açmaz ve istek yapmaz. Doğum saati bilinmiyorsa (komut `null` döner) ağaç yerine bir satır açıklama çıkar; sistem seçicisi yükleme, hata ve "saat bilinmiyor" durumlarında da görünür.** Gerekçe: spec §6 "Arayüz"; Plan 15 D19 (araç çubuğu hata durumunda da kullanılabilsin), Plan 09 M6-6 (boş alt seviye sessiz geçilmez).
- **D14 — Chara şeridi: seviye adları Vimshottari şeridiyle aynıdır (maha / antar / pratyantar = 1. / 2. / 3. seviye). "Maha" iki çevrimin dashalarını tek şeritte gösterir (ikinci çevrim "2." önekli); "antar" anın dashasının on iki alt dönemini; "pratyantar" anın alt döneminin 3. seviyesini (`chara_dasha_children`) gösterir. Güncel dönem `chara_dasha_at`'in döndürdüğü yoldur (`TransitView.snapshot.jd_ut` ile sorulur). An üretilen dönemlerin dışındaysa şerit yerine bir satır ileti çıkar. Tıklama transit anını dönemin başlangıcına taşır.** Gerekçe: spec §6 "Dönem seçimi transit tarihini v1'deki gibi sürer"; seviyelerin chara dashadaki karşılığı ve "maha"da iki çevrimin gösterimi yazılmamış. **Spec boşluğu 9.**
- **D15 — Transit anını yalnızca şeritteki dönem seçimi sürer; ağaçta bir döneme tıklamak v1'deki gibi dalı açar ya da kapatır.** Gerekçe: spec "v1'deki gibi" der; v1 ağacı transit anını sürmez (UI spec §3.4 "ağacında ya da şeridinde" dese de). v1 davranışını bu planda değiştirmek kapsam dışıdır. **Spec boşluğu 10.**
- **D16 — El örnekleri `tests/golden/modules/chara_dasha/` altında yedi dosyadır: altı harita ve doğum saati bilinmeyen bir örnek (`expected: null`). `expected`: `forward`, `order`, `years`, `second_cycle` (üretilen dashalar: burç ve yıl) ve `sub_periods` (ilk iki dasha). Girdi `advanced` bloğunu taşır. Doğum anı ve yıl uzunluğu girdide yoktur: test sabit bir an (JD 2451545,0) ve 365,25 günlük yıl kullanır ve tarihleri el yıllarının birikimli toplamından kurar. Ayrı bir test spec'in zorunlu durumlarının örneklerde bulunduğunu sabitler; bu test motorun `counts_forward` ve `sign_lord` fonksiyonları yerine elle yazılmış iki liste kullanır.** Gerekçe: spec §6 "Test" ve §13; şema chara dashanın zaman girdisini ve boş sonucu tanımlamaz. "Lordu 2. ve 12. burçta" zodyak sırasıyla sonraki ve önceki burç olarak okunmuştur (düz sayan burçta 1 ve 11 yıl, ters sayan burçta 11 ve 1 yıl: dört durum). **Spec boşluğu 11.**
- **D17 — Yeni görsel temel (PNG) ve yeni baskı düzeni yoktur (K10); Vimshottari baskı sayfası değişmez. `pnpm build` Vite'ın 500 kB parça uyarısını basmayı sürdürür (ölçüm: 516,2 kB) ve kapılar `✓ built` satırını arar (Plan 15 D22).** Gerekçe: on görsel temel harita öğelerini ve baskı sayfalarını çeker; panel araç çubukları bu görüntülerde yoktur (ölçüm: `e2e:visual` 10 geçti, temel dosyalar değişmedi).
- **D18 — Bütçeler entegrasyon testinde ölçülür: `compute_chara_dasha` ≤ 10 ms (§15), `chara_dasha_children` ≤ 5 ms (§6), `chara_dasha_at` ≤ 10 ms (bu planın eklediği komut; `compute_chara_dasha` ile aynı işi yapar).** Gerekçe: ölçüm (hata ayıklama derlemesi, temel harita hesabı dahil): üçü de 0,53–0,58 ms. §15 tablosu son iki komutu anmaz. **Spec boşluğu 12.**
- **D19 — Chara dasha sonucu oturumda paylaşılmaz: ağaç ve şerit kendi isteğini yapar (Plan 15 D16 sürer).** Gerekçe: komut 1 ms'nin altındadır; önbellek bayatlama riski getirirdi.


## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni bağımlılık yok: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `app/package.json`, `app/pnpm-lock.yaml` ve `app/vite.config.ts` değişmez; `cargo deny check` Task 7'de `advisories ok, bans ok, licenses ok, sources ok` verir.
- **Yasaklı ad kuralı (v1 spec §10 "İsim kuralları", ilk madde; ileri modüller spec'i Ek A):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Kural harf dizisi düzeyindedir: Türkçe sözcüklerin içinde de geçemez (yerine ör. "diğer", "durur", "geri", "artan", "sürer", "tutar" yazılır). **Bu planda özel dikkat:** "karaka" sözcüğüne `-lar` çoğul eki ya da `L` ile başlayan bir ad eklenince (bitişik yazımda ya da camelCase'te) yasak dizi oluşur; süre anlatırken "geri … yıl" ya da "sonraki dönemler" yazılır. Her commit adımı eklenen satırları, dosya adlarını ve commit mesajını `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile tarar; çıktı boş olmalıdır (C13). Bu belgede de eşleşme yoktur.
- **Clean-room ve yorum metni yok:** yöntem klasik kuraldan özgün olarak uygulanır; başka ürünün adı, düzeni, metni ya da çıktısı kullanılmaz; kaynak gösterimi geneldir ("Jaimini Upadesha Sutraları, dasha bölümü") ve alıntı yapılmaz. Paneller hesaplanmış olguları ve klasik etiketleri gösterir (burç, tarih, yıl, yön); bir dönemin ne anlama geldiğini anlatan metin eklenmez.
- **Evler rasi evidir (K4), rasi çerçevesi `settings.zodiac`'ı izler (K5).** Dasha sırası ve yıllar burç sayımıdır; `CalcSettings.house_system` bunları etkilemez. Burçlar `Positions`'ın rasi çerçevesindeki burçlarıdır.
- **Doğum saati bilinmiyorsa lagnaya bağlı her çıktı Rust'ta boş döner (K2):** `compute_chara_dasha` ve `chara_dasha_at` `None`, `chara_dasha_children` boş liste (D8). Arayüzün gizlemesine güvenilmez.
- **El örnekleri (§13, K9):** `source` yalnızca `hand`'dir; beklenen değer kuraldan elle türetilir ve `derivation`'da adım adım yazılır. Bir el örneği motorla uyuşmazsa önce kural yeniden okunur; **beklenen değer motor çıktısına bakarak düzeltilmez.** Gözden geçirme döngüsünün zorunlu maddesi: gözden geçiren, Task 1'de eklenen `derivation` adımlarını kuraldan yeniden hesaplar (Task 7 Step 5).
- Kapılar: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `pnpm --dir app test`, `pnpm --dir app build` her görev sonunda yeşil; `pnpm --dir app e2e` Task 6 ve 7'de. `tsc` `strict`. `pnpm build` çıktısında aranan satır `✓ built`'tir (D17).
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez). Node 24+, pnpm 11.8.0. Depo kısa bir yolda durmalıdır: Swiss Ephemeris 242 bayttan uzun ephemeris yolunu reddeder (çalışma ağacı uzun bir yoldaysa `/tmp` altında kısa bir yola klonlanır).
- **Çalışan `pnpm tauri dev` durdurulmaz** (port 1420); e2e 1421'i kullanır. `pkill -f` kullanılmaz. Üretilen `.app` çalıştırılmaz.
- Yalnızca macOS hedeflenir. Yorum metni ve AI paneli bu planın dışındadır. Shadbala ve sonrası (3.4+) bu planın dışındadır.
- **Tabanlar (`ac2b412`):** `jyotish-core` lib 173, golden 8, `modules_golden` 5; `store` lib 84 (+1 `#[ignore]`); `jyotish-desk` lib 46, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `Test Files  68 passed (68)`, `Tests  624 passed (624)`; e2e 14; görsel 10 (yerel, darwin); yayın betiği testleri 14.
- **Sayaç zinciri** (her satır görev bitince; boş hücre "önceki satırla aynı"). Bu plan yazılırken `ac2b412`'nin bir kopyasında her görev ayrı commit olarak uygulanıp ölçülmüştür.

  | Görev | `jyotish-core` lib | `modules_golden` | `store` lib | `jyotish-desk` `modules` | Vitest dosya / test | e2e |
  |---|---|---|---|---|---|---|
  | Taban | 173 | 5 | 84 (+1 ign.) | 1 | 68 / 624 | 14 |
  | Task 1 | 191 | 7 | | | | |
  | Task 2 | | | | 1 | 68 / 624 | |
  | Task 3 | | | 86 (+1 ign.) | | 68 / 628 | |
  | Task 4 | | | | | 69 / 638 | |
  | Task 5 | | | | | 70 / 649 | |
  | Task 6 | | | | | | 15 |
  | Final inceleme | | | | | 70 / 653 | 15 |

  Değişmeyenler: `jyotish-core` golden 8; `jyotish-desk` lib 46, `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2; görsel 10; yayın betiği testleri 14. `jyotish-core` lib Task 1'de 18 test kazanır (13 birim testi, 5 proptest); `modules_golden` iki test kazanır (`chara_dasha_hand_fixtures`, `chara_dasha_fixtures_cover_the_required_cases`). `jyotish-desk` `modules` tek test olarak durur (süreçte tek SE örneği); Task 2 ona `chara_dasha_checks`'i ekler. `jyotish-desk` lib 46 olarak durur (Task 3 var olan bir testi büyütür).
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir:**
  - **C1** Gizlilik (spec §14): danışan adları, doğum verisi, not gövdeleri, yer bilgisi, dosya yolları ve `CommandError.params` loglanmaz; `no-console.test.ts` durur ve izin listesi büyümez. `ChartInput`, `inputKey` ve transit anı doğum verisinden türer: loglanmaz, hata parametresine girmez (`invalid_dasha_input` yalnızca alan adını taşır). Hiçbir modül ağ isteği yapmaz; `CharaDasha` veritabanına yazılmaz.
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu ya da adres almaz. Yeni komutların bağımsız değişkenleri `ChartInput`, `Vec<Sign>`, `u8` ve `f64`'tür (dizgi yoktur); `api.test.ts`'in dizgi bağımsız değişken listesi değişmez.
  - **C3** Göç V1 donmuştur; `crates/store/src/schema.rs` değişmez. `CalcSettings` ve `AppPrefs` şema sürümleri değişmez; `ViewState` yeni alan kazanır ama `VIEW_STATE_SCHEMA_VERSION` 1 olarak durur (D10).
  - **C4** Arayüz astrolojik hesap yapmaz: yön, yıllar, dönemler ve anın dönem yolu Rust'tan gelir; arayüz tarih karşılaştırmaz (D7). JD'nin yerel tarihe biçimlenmesi, dönemlerin yola göre gruplanması ve "2." öneki gösterimdir.
  - **C5/C17** Otomatik kayıtlar ve boşaltılmaları değişmez: chara yolu Vimshottari yoluyla aynı debounce ve aynı `save_view_state` kaydıyla yazılır. **C6** Düzen ayrıştırıcıları (`parseLayoutState`, Rust `check_panel_layout`) değişmez.
  - **C7** Kısayollar yalnızca `shortcuts.ts`'tedir; bu plan kısayol eklemez. **C8** Her pencere `shell/Dialog`'u kullanır (bu plan pencere eklemez).
  - **C9** Renk yalnızca token'lardan gelir (hex yok) ve tek başına anlam taşımaz: ikinci çevrim "2." önekiyle, güncel dönem zemin, kenar ve "transit anında" metniyle, açık dal `aria-expanded` ve yazı ağırlığıyla gösterilir.
  - **C10** Üretilen dosyalar elle düzenlenmez: `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` Task 2, 3 ve 5'te komutla yeniden üretilir.
  - **C11** Görsel PNG temelleri yereldir; bu planda değişmeleri beklenmez (D17).
  - **C12** Her `CommandError.code` üç dilde çevrilidir; bu plan kod eklemez (D8). Her yeni i18n anahtarı üç dilde vardır (`i18n.test.ts` anahtar kümelerini ve yer tutucuları karşılaştırır).
  - **C13** Yasaklı ad taraması her commit'te; depo dışındaki ilerleme notları ve `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** Plan 07'nin gün doğumu toleransı değişmez. **C15** `alert`, `confirm`, `prompt`, `window.print()` ve `window.open()` çağrılmaz. **C16** ⌘ kısayolları değişmez.
- **Retro** her yerde düz `R`'dir ve Rahu ile Ketu'da gösterilmez (UI spec §2.3). Chara dasha görünümleri retro göstermez.
- **Erişilebilirlik:** WCAG AA. Ağaç WAI-ARIA ağaç desenini izler (`role="tree"`, `treeitem`, `aria-level`, `aria-setsize`, `aria-posinset`, `aria-expanded`; ok tuşları, Home, End, Enter, boşluk); seçicilerin görünür etiketi vardır; şeritte güncel dönem `aria-current` taşır; yükleme iskeleti `aria-busy`, hata `role="alert"` ve "Yeniden dene" düğmesi taşır; "Yeniden dene" hata kutusunu değiştirirken odak panelde durur.
- Commit mesajları İngilizcedir ve deponun `Co-Authored-By:` satırıyla biter (yürütücünün oturum yönergesindeki satır). Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, `git tag`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Kod blokları `ac2b412` ağacına göre yazılmış ve bu plan yazılırken deponun bir kopyasında koşulmuştur (Self-Review madde 5). Yeni dosyalar tam içerikle verilir ("**Yeni dosya:**"). Değişen dosyalar `diff` bloğuyla verilir: blok bir dosyaya konup depo kökünde `git apply <dosya>` ile ya da elle uygulanır; bağlam tutmazsa önceki görevin adımı denetlenir, elle uydurma yapılmaz. Bir dosyanın test ve uygulama değişiklikleri ayrı adımlarda ayrı bloklardır. Dosya ve diff blokları dört ters tırnakla çevrilidir (içlerinde üç ters tırnak geçebilir); komut blokları üç ters tırnaklıdır. `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` diff olarak verilmez; adımlardaki komutlar üretir.

## Sonraki alt projelere verilen API

Bu planın ürettiği ve sonraki işlerin tüketeceği adlar. Rust adları `jyotish_core`'dan dışa aktarılır; imzalar Task 1'de tam olarak verilir.

| API | Tüketen | Not |
|---|---|---|
| `chara_dasha(positions, co_lords, birth_jd_ut, year_days) -> Option<CharaDasha>`, `CharaDasha`, `SignDashaPeriod`, `CHARA_DASHA_METHOD` | AI paketleyicisi; 3.9'un hafif yolu (dasha koşulu chara dashaya genişlerse) | Lagna yoksa `None` |
| `sign_dasha_children(dasha, path, cycle)`, `sign_dasha_path_at(dasha, jd_ut)`, `SignDashaPath` | AI paketleyicisi ("şu anki dönem"), 3.9 | Saf; ephemeris görmez |
| `chara_dasha_forward`, `chara_dasha_order`, `chara_dasha_years`, `chara_dasha_sign_years`, `second_cycle_years`, `sub_period_signs`, `sign_sequence` | Başka bir burç dashası eklenirse (spec §6 kapsam dışı) aynı sıra ve bölme yardımcıları | |
| `compute_chara_dasha`, `compute_chara_dasha_children`, `compute_chara_dasha_at` (toplayıcılar); Tauri komutları `compute_chara_dasha`, `chara_dasha_children`, `chara_dasha_at` | Arayüz; AI paneli | |
| `store::ViewState.chara_expanded_path`, `empty_chara_path`, `parse_chara_path` | Sonraki sistemler aynı kalıpla kendi alanını ekler | Şema sürümü değişmez |
| Arayüz: `dasha/PeriodTree.tsx` (`PeriodNode`, `PeriodLevel`), `dasha/system.tsx` (`dashaSystem`, `SystemSelect`), `dasha/chara.ts` (`useCharaDasha`, `useCharaChildren`, `charaName`) | Dönem ağacı gösteren sonraki paneller; AI paneli | |

## Dosya Yapısı (bu plan)

```
crates/jyotish-core/
  src/jaimini/chara_dasha.rs      chara dasha: yön, yıllar, dönemler, yol, toplayıcılar (yeni)
  src/jaimini/mod.rs              alt modül kaydı ve dışa aktarımlar
  src/lib.rs                      dışa aktarımlar
  src/proptests.rs                chara_dasha_properties
  tests/modules_golden.rs         chara dasha el örnekleri ve kapsam testi
tests/golden/modules/
  README.md                       chara_dasha satırı
  chara_dasha/*.json              7 el örneği (yeni)
crates/store/
  src/view_state.rs               chara_expanded_path: tip, ayrıştırma, denetim
  src/charts.rs, src/exchange.rs  okuma, yazma, dışa/içe aktarma
  src/lib.rs                      dışa aktarımlar
app/src-tauri/
  src/module_commands.rs          compute_chara_dasha, chara_dasha_children, chara_dasha_at
  src/lib.rs                      komut kaydı
  src/client_commands.rs          save_view_state testi (chara yolu)
  tests/modules.rs                chara dasha entegrasyonu ve bütçeler
  tests/ui_fixture.rs             fikstüre chara dasha
  tests/open_client.rs            görünüm kaydında chara yolu
app/ui/
  bindings.ts                     üretilir
  test/fixtures/delhi.json        üretilir
  test/fixture.ts                 UiFixture.chara_dasha*
  test/render.tsx                 chartViewOf
  api.ts, api.test.ts             computeCharaDasha, charaDashaChildren, charaDashaAt
  view/{model,reducer,store}.ts(x) (+test)   charaPath, setCharaPath
  session/store.tsx (+test), App.tsx         açılan haritanın chara yolu
  dasha/PeriodTree.tsx            ortak ağaç çizimi ve klavyesi (yeni)
  dasha/system.tsx                sistem ve seviye ayarları, SystemSelect (yeni)
  dasha/chara.ts                  istek kancaları, gruplama, adlandırma (yeni)
  dasha/CharaDashaTree.tsx (+test)           (yeni)
  dasha/CharaDashaStrip.tsx (+test)          (yeni)
  dasha/DashaTreePanel.tsx        sistem seçicisi; Vimshottari ağacı PeriodTree üzerinde
  dasha/DashaStripPanel.tsx       sistem seçicisi
  dasha/dasha.css
  i18n/{tr,en,fr}.ts              dasha.system, dasha.systems, dasha.chara
app/e2e/{ipc.ts,desk.spec.ts}     üç komutun taklidi; chara dasha akışı
```

### Kapsam

| Spec maddesi | Yer |
|---|---|
| §6 Burç grupları, dasha sırası (yön) | Task 1 (D2) |
| §6 Dasha süresi (sayım − 1; kendi burcunda 12; Akrep ve Kova'nın lordu) | Task 1 (D2, D3) |
| §6 İkinci çevrim (12 − yıl; atlanan burç; 120 yıl) | Task 1 (D4) |
| §6 Alt dönemler (on iki eşit parça; sıra; 3 seviye) | Task 1 (D5) |
| §6 Başlangıç ve yıl uzunluğu (`settings.dasha_year`) | Task 1 (D6) |
| §6 Doğum saati bilinmiyorsa hesaplanmaz (K2) | Task 1, 2 (D8) |
| §6 `SignDashaPeriod`, `CharaDasha`; `compute_chara_dasha`, `chara_dasha_children`; K1, K8 | Task 1, 2 (D7–D9) |
| §6 Arayüz: `dasha_tree` ve `dasha_strip` `system` ayarı; burç kısaltmaları; "2." öneki | Task 4, 5 (D11–D14) |
| §6 Arayüz: ağaç başlığında yön ve yıl dizisi | Task 4 (D13) |
| §6 Arayüz: dönem seçimi transit tarihini sürer | Task 5 (D14, D15) |
| §6 Arayüz: `ViewState.dasha_expanded_path` sistem başına ayrı | Task 3 (D10) |
| §6 Test: en az 6 el örneği ve zorunlu durumlar | Task 1 (D16) |
| §6 Test: property'ler | Task 1 (`chara_dasha_properties`), Task 2 |
| §6 Test: 144 hücrelik tabloyla çapraz denetim; §13 katman 3 | Task 1 (`HAND_YEARS`) |
| §13 Komut katmanı; §15 bütçeler (≤ 10 ms; alt dönemler ≤ 5 ms) | Task 2 (D18) |
| §13 Panel durumları: boş, yükleniyor, hata, dolu, "saat bilinmiyor" | Task 4, 5 |
| §14 Gizlilik | C1; Task 7 taramaları |
| Uçtan uca akış | Task 6 |
| Son doğrulama, elle denetim, el örneklerinin yeniden türetilmesi | Task 7 |

---

### Task 1: Çekirdek: chara dasha (yön, yıllar, iki çevrim, alt dönemler, anın yolu) ve el örnekleri

Spec §6 "Kapsam ve yöntem", "Veri modeli", "Test"; §13 katman 1, 2 ve 3. Hiçbir mevcut davranış değişmez; yalnızca yeni alt modül, el örnekleri ve testler eklenir.

**Files:**
- Create: `crates/jyotish-core/src/jaimini/chara_dasha.rs`, `tests/golden/modules/chara_dasha/*.json` (7 dosya)
- Modify: `crates/jyotish-core/src/jaimini/mod.rs`, `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/proptests.rs`, `crates/jyotish-core/tests/modules_golden.rs`, `tests/golden/modules/README.md`
- Test: `chara_dasha.rs` içindeki `mod tests`, `proptests.rs` (`chara_dasha_properties`), `tests/modules_golden.rs`

**Interfaces:**
- Consumes (Plan 14 ve 15): `Positions` (`lagna_sign`, `sign_of`, `synthetic`, `without_lagna`, `From<&Chart>`), `ChartInput`, `Sign::{ALL, index, from_index}` (`from_index` modülerdir), `counts_forward(sign) -> bool`, `directed_count(from, to) -> u8` (1..=12, `from`'un yönünde kapsayıcı sayım), `jaimini_lord(sign, positions, co_lords) -> Graha`, `co_lord_choice`, `compute_chart`, `CoreError::InvalidDashaInput(&'static str)`, `CalcSettings::{dasha_year, advanced}`; `modules_golden.rs` düzeneği (`Fixture<X>`, `Input`, `load`); `foundation_properties::positions()` üreteci; `fake::FakeEph`.
- Produces (hepsi `jyotish_core`'dan dışa aktarılır):
  - `pub const CHARA_DASHA_METHOD: &str = "chara_dasha.padakrama_v1"`, `pub const MAX_SIGN_DASHA_LEVEL: u8 = 3`, `pub const CHARA_DASHA_LEVELS: u8 = 2`, `pub const CHARA_DASHA_HORIZON_YEARS: u16 = 120`
  - `pub struct SignDashaPeriod { pub signs: Vec<Sign>, pub level: u8, pub cycle: u8, pub start_jd_ut: f64, pub end_jd_ut: f64, pub has_children: bool }`
  - `pub struct CharaDasha { pub forward: bool, pub years: [u8; 12], pub periods: Vec<SignDashaPeriod>, pub method: String }`
  - `pub struct SignDashaPath { pub cycle: u8, pub signs: Vec<Sign> }` (`Eq`)
  - `pub fn chara_dasha_forward(lagna: Sign) -> bool`, `pub fn sign_sequence(from: Sign, forward: bool) -> [Sign; 12]`, `pub fn chara_dasha_order(lagna: Sign) -> [Sign; 12]`
  - `pub fn chara_dasha_years(sign: Sign, lord_sign: Sign) -> u8` (1..=12), `pub fn chara_dasha_sign_years(sign: Sign, positions: &Positions, co_lords: bool) -> u8`, `pub fn second_cycle_years(first_cycle_years: u8) -> u8`, `pub fn sub_period_signs(sign: Sign, forward: bool) -> [Sign; 12]`
  - `pub fn chara_dasha(positions: &Positions, co_lords: bool, birth_jd_ut: f64, year_days: f64) -> Option<CharaDasha>`
  - `pub fn sign_dasha_children(dasha: &CharaDasha, path: &[Sign], cycle: u8) -> Result<Vec<SignDashaPeriod>, CoreError>`
  - `pub fn sign_dasha_path_at(dasha: &CharaDasha, jd_ut: f64) -> Result<Option<SignDashaPath>, CoreError>`
  - Toplayıcılar: `pub fn compute_chara_dasha<E: Ephemeris + ?Sized>(eph: &mut E, input: &ChartInput) -> Result<Option<CharaDasha>, CoreError>`, `compute_chara_dasha_children(eph, input, path: &[Sign], cycle: u8) -> Result<Vec<SignDashaPeriod>, CoreError>`, `compute_chara_dasha_at(eph, input, jd_ut: f64) -> Result<Option<SignDashaPath>, CoreError>`
  - El örneği düzeneği: `CharaDashaExpected { forward, order, years, second_cycle, sub_periods }`; `MIN_CHARA_DASHA = 7`.

- [ ] **Step 1: Tabanları ölç**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
git log -1 --format=%h
git status --short
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `git status --short` boş; `jyotish-core` lib `173 passed`, golden `8 passed`, `modules_golden` `5 passed`; `store` lib `84 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `46 passed`, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `68 passed (68)`, `624 passed (624)`. Commit `ac2b412` değilse ölçülen sayılar ilerleme notlarına taban olarak yazılır ve zincir oradan yürütülür. (Vitest'te `NotesPanel.test.tsx › New note adds an empty note…` yük altında seyrek olarak düşebilir; bu plandan bağımsızdır, yeniden koşulur ve ilerleme notlarına yazılır.)

- [ ] **Step 2: El örneklerini yaz (kaynak: `hand`)**

Yedi dosya: altı sentetik harita ve doğum saati bilinmeyen bir örnek. Her biri kural adımlarını `derivation`'da taşır; beklenen değerler bu adımlardan elle çıkarılmıştır (motor koşulmadan önce). Konumlar sentetiktir (astronomik olarak olası olmaları gerekmez); Rahu ile Ketu her sette tam karşılıklıdır. Her harita şu adımları taşır: "Burçlar", "Yön", "Eş lord, Akrep", "Eş lord, Kova", "Yıllar (1–6)", "Yıllar (7–12)", "İkinci çevrim", "Alt dönemler". Örneklerin kapsadığı durumlar:

| Dosya | Yön | Öne çıkan durum |
|---|---|---|
| `forward-aries-lagna-lord-in-second-and-twelfth` | ileri | Düz sayan burçta lord sonraki burçta (1 yıl) ve önceki burçta (11 yıl); ters sayan burçta lord önceki burçta (1 yıl); ikinci çevrimde atlanan iki burç; eş lord 2. ölçütle |
| `backward-taurus-lagna-ends-exactly-at-horizon` | geri | Ters sayan burçta lord sonraki burçta (11 yıl); ikinci çevrim tam 120. yılda biter ve sonraki dasha üretilmez; eş lord 1. ve 3. ölçütle |
| `both-co-lords-in-scorpio` | ileri | Mars ve Ketu Akrep'te: 12 yıl; Kova'da dışarıdaki Rahu |
| `co-lords-off-backward-cancer-lagna` | geri | Ayar kapalı: Akrep ve Kova'nın lordu Mars ve Satürn (ikisi de kendi burcunda: 12 yıl) |
| `lagna-sign-skipped-in-second-cycle` | ileri | Lagna burcu 12 yıl: ikinci çevrim onu atlar; eş lord 4. ve 5. ölçütle |
| `both-co-lords-in-aquarius-mars-exalted` | geri | Satürn ve Rahu Kova'da: 12 yıl; Akrep'te yücelmiş Mars (3. ölçüt) |
| `time-unknown-no-dasha` | — | Lagna yok: `expected: null` |

**Yeni dosya:** `tests/golden/modules/chara_dasha/backward-taurus-lagna-ends-exactly-at-horizon.json`

````json
{
  "id": "backward-taurus-lagna-ends-exactly-at-horizon",
  "source": "hand",
  "rule_ref": "Jaimini Upadesha Sutraları, dasha bölümü; padakrama düzeni: düz sayan burçlar Koç, Boğa, İkizler, Terazi, Akrep, Yay; ters sayanlar Yengeç, Aslan, Başak, Oğlak, Kova, Balık; yıl = burçtan lorduna burcun yönünde kapsayıcı sayım − 1, lord kendi burcundaysa 12; eş lordlar BPHS ve Jaimini geleneğindeki altı ölçütle",
  "derivation": [
    "Burçlar: Lagna 40° Boğa. Güneş 155° Başak 5°, Ay 100° Yengeç 10°, Mars 220° Akrep 10°, Merkür 50° Boğa 20°, Jüpiter 250° Yay 10°, Venüs 190° Terazi 10°, Satürn 5° Koç 5°, Rahu 70° İkizler 10°, Ketu 250° Yay 10°.",
    "Yön: lagna Boğa; Boğa'dan 9. burç Oğlak, ters sayan gruptadır: dashalar ters sırayla ilerler. Sıra: Boğa, Koç, Balık, Kova, Oğlak, Yay, Akrep, Terazi, Başak, Aslan, Yengeç, İkizler.",
    "Eş lord, Akrep (Mars Akrep'te, Ketu Yay'da): 1. ölçüt: yalnızca Mars burcun içinde; dışarıdaki lord Ketu.",
    "Eş lord, Kova (Satürn Koç'ta, Rahu İkizler'de): 1. ölçüt ayırmaz. 2. ölçüt: ikisinin de yanında graha yok (0–0). 3. ölçüt: Satürn Koç'ta düşüş burcundadır: düğüm seçilir, Rahu.",
    "Yıllar (1–6). Boğa (düz), Venüs Terazi'de: Boğa 1, İkizler 2, Yengeç 3, Aslan 4, Başak 5, Terazi 6; 5 yıl. Koç (düz), Mars Akrep'te: Koç 1 … Akrep 8; 7 yıl. Balık (ters), Jüpiter Yay'da: Balık 1, Kova 2, Oğlak 3, Yay 4; 3 yıl. Kova (ters), lord Rahu İkizler'de: Kova 1, Oğlak 2, Yay 3, Akrep 4, Terazi 5, Başak 6, Aslan 7, Yengeç 8, İkizler 9; 8 yıl. Oğlak (ters), Satürn Koç'ta: Oğlak 1, Yay 2, Akrep 3, Terazi 4, Başak 5, Aslan 6, Yengeç 7, İkizler 8, Boğa 9, Koç 10; 9 yıl. Yay (düz), Jüpiter Yay'da: 12 yıl.",
    "Yıllar (7–12). Akrep (düz), lord Ketu Yay'da: Akrep 1, Yay 2; 1 yıl. Terazi (düz), Venüs Terazi'de: 12 yıl. Başak (ters), Merkür Boğa'da: Başak 1, Aslan 2, Yengeç 3, İkizler 4, Boğa 5; 4 yıl. Aslan (ters), Güneş Başak'ta: Aslan 1, Yengeç 2, … Terazi 11, Başak 12; sayım 12, 11 yıl (ters sayan burçta lord sonraki burçta). Yengeç (ters), Ay Yengeç'te: 12 yıl. İkizler (düz), Merkür Boğa'da: İkizler 1, Yengeç 2, … Koç 11, Boğa 12; 11 yıl. Toplam 5 + 7 + 3 + 8 + 9 + 12 + 1 + 12 + 4 + 11 + 12 + 11 = 95.",
    "İkinci çevrim: Boğa 7 (95→102), Koç 5 (102→107), Balık 9 (107→116), Kova 4 (116→120). Oğlak tam 120. yılda başlardı; başlangıcı 120. yıldan önce değildir: üretilmez.",
    "Alt dönemler (geri). Boğa dashası: Koç, Balık, Kova, Oğlak, Yay, Akrep, Terazi, Başak, Aslan, Yengeç, İkizler, Boğa. Koç dashası: Balık, Kova, Oğlak, Yay, Akrep, Terazi, Başak, Aslan, Yengeç, İkizler, Boğa, Koç."
  ],
  "input": {
    "lagna_lon": 40.0,
    "grahas": {"Sun": 155.0, "Moon": 100.0, "Mars": 220.0, "Mercury": 50.0, "Jupiter": 250.0, "Venus": 190.0, "Saturn": 5.0, "Rahu": 70.0, "Ketu": 250.0},
    "advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}
  },
  "expected": {
    "forward": false,
    "order": ["Taurus", "Aries", "Pisces", "Aquarius", "Capricorn", "Sagittarius", "Scorpio", "Libra", "Virgo", "Leo", "Cancer", "Gemini"],
    "years": [5, 7, 3, 8, 9, 12, 1, 12, 4, 11, 12, 11],
    "second_cycle": [
      {"sign": "Taurus", "years": 7},
      {"sign": "Aries", "years": 5},
      {"sign": "Pisces", "years": 9},
      {"sign": "Aquarius", "years": 4}
    ],
    "sub_periods": [
      ["Aries", "Pisces", "Aquarius", "Capricorn", "Sagittarius", "Scorpio", "Libra", "Virgo", "Leo", "Cancer", "Gemini", "Taurus"],
      ["Pisces", "Aquarius", "Capricorn", "Sagittarius", "Scorpio", "Libra", "Virgo", "Leo", "Cancer", "Gemini", "Taurus", "Aries"]
    ]
  }
}
````

**Yeni dosya:** `tests/golden/modules/chara_dasha/both-co-lords-in-aquarius-mars-exalted.json`

````json
{
  "id": "both-co-lords-in-aquarius-mars-exalted",
  "source": "hand",
  "rule_ref": "Jaimini Upadesha Sutraları, dasha bölümü; padakrama düzeni: düz sayan burçlar Koç, Boğa, İkizler, Terazi, Akrep, Yay; ters sayanlar Yengeç, Aslan, Başak, Oğlak, Kova, Balık; yıl = burçtan lorduna burcun yönünde kapsayıcı sayım − 1, lord kendi burcundaysa 12; eş lordlar BPHS ve Jaimini geleneğindeki altı ölçütle",
  "derivation": [
    "Burçlar: Lagna 280° Oğlak. Güneş 10° Koç 10°, Ay 40° Boğa 10°, Mars 275° Oğlak 5°, Merkür 350° Balık 20°, Jüpiter 100° Yengeç 10°, Venüs 345° Balık 15°, Satürn 310° Kova 10°, Rahu 320° Kova 20°, Ketu 140° Aslan 20°.",
    "Yön: lagna Oğlak; Oğlak'tan 9. burç Başak, ters sayan gruptadır: geri. Sıra: Oğlak, Yay, Akrep, Terazi, Başak, Aslan, Yengeç, İkizler, Boğa, Koç, Balık, Kova.",
    "Eş lord, Akrep (Mars Oğlak'ta, Ketu Aslan'da): 1. ölçüt ayırmaz. 2. ölçüt: ikisinin de yanında graha yok (0–0; lagna graha değildir). 3. ölçüt: Mars Oğlak'ta yücelme burcundadır: Mars.",
    "Eş lord, Kova (Satürn ve Rahu Kova'da): iki eş lord da burcun içinde; süre 12 yıl.",
    "Yıllar (1–6). Oğlak (ters), Satürn Kova'da: Oğlak 1, Yay 2, … Balık 11, Kova 12; 11 yıl. Yay (düz), Jüpiter Yengeç'te: Yay 1, Oğlak 2, Kova 3, Balık 4, Koç 5, Boğa 6, İkizler 7, Yengeç 8; 7 yıl. Akrep (düz), lord Mars Oğlak'ta: Akrep 1, Yay 2, Oğlak 3; 2 yıl. Terazi (düz), Venüs Balık'ta: Terazi 1, Akrep 2, Yay 3, Oğlak 4, Kova 5, Balık 6; 5 yıl. Başak (ters), Merkür Balık'ta: Başak 1, Aslan 2, Yengeç 3, İkizler 4, Boğa 5, Koç 6, Balık 7; 6 yıl. Aslan (ters), Güneş Koç'ta: Aslan 1, Yengeç 2, İkizler 3, Boğa 4, Koç 5; 4 yıl.",
    "Yıllar (7–12). Yengeç (ters), Ay Boğa'da: Yengeç 1, İkizler 2, Boğa 3; 2 yıl. İkizler (düz), Merkür Balık'ta: İkizler 1 … Balık 10; 9 yıl. Boğa (düz), Venüs Balık'ta: Boğa 1 … Balık 11; 10 yıl. Koç (düz), Mars Oğlak'ta: Koç 1 … Oğlak 10; 9 yıl. Balık (ters), Jüpiter Yengeç'te: Balık 1, Kova 2, Oğlak 3, Yay 4, Akrep 5, Terazi 6, Başak 7, Aslan 8, Yengeç 9; 8 yıl. Kova: 12 yıl. Toplam 11 + 7 + 2 + 5 + 6 + 4 + 2 + 9 + 10 + 9 + 8 + 12 = 85.",
    "İkinci çevrim: Oğlak 1 (85→86), Yay 5 (86→91), Akrep 10 (91→101), Terazi 7 (101→108), Başak 6 (108→114), Aslan 8 (114→122; 114 < 120, üretilir). Yengeç 122. yılda başlardı: üretilmez.",
    "Alt dönemler (geri). Oğlak dashası: Yay, Akrep, Terazi, Başak, Aslan, Yengeç, İkizler, Boğa, Koç, Balık, Kova, Oğlak. Yay dashası: Akrep, Terazi, Başak, Aslan, Yengeç, İkizler, Boğa, Koç, Balık, Kova, Oğlak, Yay."
  ],
  "input": {
    "lagna_lon": 280.0,
    "grahas": {"Sun": 10.0, "Moon": 40.0, "Mars": 275.0, "Mercury": 350.0, "Jupiter": 100.0, "Venus": 345.0, "Saturn": 310.0, "Rahu": 320.0, "Ketu": 140.0},
    "advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}
  },
  "expected": {
    "forward": false,
    "order": ["Capricorn", "Sagittarius", "Scorpio", "Libra", "Virgo", "Leo", "Cancer", "Gemini", "Taurus", "Aries", "Pisces", "Aquarius"],
    "years": [11, 7, 2, 5, 6, 4, 2, 9, 10, 9, 8, 12],
    "second_cycle": [
      {"sign": "Capricorn", "years": 1},
      {"sign": "Sagittarius", "years": 5},
      {"sign": "Scorpio", "years": 10},
      {"sign": "Libra", "years": 7},
      {"sign": "Virgo", "years": 6},
      {"sign": "Leo", "years": 8}
    ],
    "sub_periods": [
      ["Sagittarius", "Scorpio", "Libra", "Virgo", "Leo", "Cancer", "Gemini", "Taurus", "Aries", "Pisces", "Aquarius", "Capricorn"],
      ["Scorpio", "Libra", "Virgo", "Leo", "Cancer", "Gemini", "Taurus", "Aries", "Pisces", "Aquarius", "Capricorn", "Sagittarius"]
    ]
  }
}
````

**Yeni dosya:** `tests/golden/modules/chara_dasha/both-co-lords-in-scorpio.json`

````json
{
  "id": "both-co-lords-in-scorpio",
  "source": "hand",
  "rule_ref": "Jaimini Upadesha Sutraları, dasha bölümü; padakrama düzeni: düz sayan burçlar Koç, Boğa, İkizler, Terazi, Akrep, Yay; ters sayanlar Yengeç, Aslan, Başak, Oğlak, Kova, Balık; yıl = burçtan lorduna burcun yönünde kapsayıcı sayım − 1, lord kendi burcundaysa 12; eş lordlar BPHS ve Jaimini geleneğindeki altı ölçütle",
  "derivation": [
    "Burçlar: Lagna 310° Kova. Güneş 20° Koç 20°, Ay 340° Balık 10°, Mars 215° Akrep 5°, Merkür 10° Koç 10°, Jüpiter 100° Yengeç 10°, Venüs 70° İkizler 10°, Satürn 305° Kova 5°, Rahu 45° Boğa 15°, Ketu 225° Akrep 15°.",
    "Yön: lagna Kova; Kova'dan 9. burç Terazi, düz sayan gruptadır: ileri. Sıra: Kova, Balık, Koç, Boğa, İkizler, Yengeç, Aslan, Başak, Terazi, Akrep, Yay, Oğlak.",
    "Eş lord, Akrep (Mars ve Ketu Akrep'te): iki eş lord da burcun içinde; süre 12 yıl. (Seçim: 1. ölçüt ayırmaz; yanlarında başka graha yok; Mars Akrep'te ne yücelmiş ne düşmüş; burç ve uzaklık aynı; 6. ölçüt Mars, o da Akrep'te.)",
    "Eş lord, Kova (Satürn Kova'da, Rahu Boğa'da): 1. ölçüt: yalnızca Satürn burcun içinde; dışarıdaki lord Rahu.",
    "Yıllar (1–6). Kova (ters), lord Rahu Boğa'da: Kova 1, Oğlak 2, Yay 3, Akrep 4, Terazi 5, Başak 6, Aslan 7, Yengeç 8, İkizler 9, Boğa 10; 9 yıl. Balık (ters), Jüpiter Yengeç'te: Balık 1, Kova 2, Oğlak 3, Yay 4, Akrep 5, Terazi 6, Başak 7, Aslan 8, Yengeç 9; 8 yıl. Koç (düz), Mars Akrep'te: Koç 1 … Akrep 8; 7 yıl. Boğa (düz), Venüs İkizler'de: Boğa 1, İkizler 2; 1 yıl. İkizler (düz), Merkür Koç'ta: İkizler 1, Yengeç 2, … Balık 10, Koç 11; 10 yıl. Yengeç (ters), Ay Balık'ta: Yengeç 1, İkizler 2, Boğa 3, Koç 4, Balık 5; 4 yıl.",
    "Yıllar (7–12). Aslan (ters), Güneş Koç'ta: Aslan 1, Yengeç 2, İkizler 3, Boğa 4, Koç 5; 4 yıl. Başak (ters), Merkür Koç'ta: Başak 1, Aslan 2, Yengeç 3, İkizler 4, Boğa 5, Koç 6; 5 yıl. Terazi (düz), Venüs İkizler'de: Terazi 1, Akrep 2, Yay 3, Oğlak 4, Kova 5, Balık 6, Koç 7, Boğa 8, İkizler 9; 8 yıl. Akrep: 12 yıl. Yay (düz), Jüpiter Yengeç'te: Yay 1, Oğlak 2, Kova 3, Balık 4, Koç 5, Boğa 6, İkizler 7, Yengeç 8; 7 yıl. Oğlak (ters), Satürn Kova'da: Oğlak 1, Yay 2, … Balık 11, Kova 12; 11 yıl. Toplam 9 + 8 + 7 + 1 + 10 + 4 + 4 + 5 + 8 + 12 + 7 + 11 = 86.",
    "İkinci çevrim: Kova 3 (86→89), Balık 4 (89→93), Koç 5 (93→98), Boğa 11 (98→109), İkizler 2 (109→111), Yengeç 8 (111→119), Aslan 8 (119→127; 119 < 120, üretilir). Başak 127. yılda başlardı: üretilmez.",
    "Alt dönemler (ileri). Kova dashası: Balık, Koç, Boğa, İkizler, Yengeç, Aslan, Başak, Terazi, Akrep, Yay, Oğlak, Kova. Balık dashası: Koç, Boğa, İkizler, Yengeç, Aslan, Başak, Terazi, Akrep, Yay, Oğlak, Kova, Balık."
  ],
  "input": {
    "lagna_lon": 310.0,
    "grahas": {"Sun": 20.0, "Moon": 340.0, "Mars": 215.0, "Mercury": 10.0, "Jupiter": 100.0, "Venus": 70.0, "Saturn": 305.0, "Rahu": 45.0, "Ketu": 225.0},
    "advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}
  },
  "expected": {
    "forward": true,
    "order": ["Aquarius", "Pisces", "Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo", "Libra", "Scorpio", "Sagittarius", "Capricorn"],
    "years": [9, 8, 7, 1, 10, 4, 4, 5, 8, 12, 7, 11],
    "second_cycle": [
      {"sign": "Aquarius", "years": 3},
      {"sign": "Pisces", "years": 4},
      {"sign": "Aries", "years": 5},
      {"sign": "Taurus", "years": 11},
      {"sign": "Gemini", "years": 2},
      {"sign": "Cancer", "years": 8},
      {"sign": "Leo", "years": 8}
    ],
    "sub_periods": [
      ["Pisces", "Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo", "Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius"],
      ["Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo", "Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces"]
    ]
  }
}
````

**Yeni dosya:** `tests/golden/modules/chara_dasha/co-lords-off-backward-cancer-lagna.json`

````json
{
  "id": "co-lords-off-backward-cancer-lagna",
  "source": "hand",
  "rule_ref": "Jaimini Upadesha Sutraları, dasha bölümü; padakrama düzeni: düz sayan burçlar Koç, Boğa, İkizler, Terazi, Akrep, Yay; ters sayanlar Yengeç, Aslan, Başak, Oğlak, Kova, Balık; yıl = burçtan lorduna burcun yönünde kapsayıcı sayım − 1, lord kendi burcundaysa 12; eş lordlar BPHS ve Jaimini geleneğindeki altı ölçütle",
  "derivation": [
    "Burçlar: Lagna 100° Yengeç. Güneş 280° Oğlak 10°, Ay 130° Aslan 10°, Mars 220° Akrep 10°, Merkür 290° Oğlak 20°, Jüpiter 190° Terazi 10°, Venüs 320° Kova 20°, Satürn 310° Kova 10°, Rahu 10° Koç 10°, Ketu 190° Terazi 10°.",
    "Yön: lagna Yengeç; Yengeç'ten 9. burç Balık, ters sayan gruptadır: geri. Sıra: Yengeç, İkizler, Boğa, Koç, Balık, Kova, Oğlak, Yay, Akrep, Terazi, Başak, Aslan.",
    "Eş lordlar devre dışı (ayar): Akrep'in lordu Mars, Kova'nın lordu Satürn'dür. Mars Akrep'te: 12 yıl. Satürn Kova'da: 12 yıl. (Ayar açık olsaydı her ikisinde de 1. ölçüt dışarıdaki düğümü seçerdi: Akrep için Ketu Terazi'de, sayım 12, 11 yıl; Kova için Rahu Koç'ta, sayım 11, 10 yıl.)",
    "Yıllar (1–6). Yengeç (ters), Ay Aslan'da: Yengeç 1, İkizler 2, … Başak 11, Aslan 12; 11 yıl. İkizler (düz), Merkür Oğlak'ta: İkizler 1, Yengeç 2, Aslan 3, Başak 4, Terazi 5, Akrep 6, Yay 7, Oğlak 8; 7 yıl. Boğa (düz), Venüs Kova'da: Boğa 1 … Kova 10; 9 yıl. Koç (düz), Mars Akrep'te: Koç 1 … Akrep 8; 7 yıl. Balık (ters), Jüpiter Terazi'de: Balık 1, Kova 2, Oğlak 3, Yay 4, Akrep 5, Terazi 6; 5 yıl. Kova: 12 yıl.",
    "Yıllar (7–12). Oğlak (ters), Satürn Kova'da: Oğlak 1, Yay 2, … Balık 11, Kova 12; 11 yıl. Yay (düz), Jüpiter Terazi'de: Yay 1, Oğlak 2, … Başak 10, Terazi 11; 10 yıl. Akrep: 12 yıl. Terazi (düz), Venüs Kova'da: Terazi 1, Akrep 2, Yay 3, Oğlak 4, Kova 5; 4 yıl. Başak (ters), Merkür Oğlak'ta: Başak 1, Aslan 2, Yengeç 3, İkizler 4, Boğa 5, Koç 6, Balık 7, Kova 8, Oğlak 9; 8 yıl. Aslan (ters), Güneş Oğlak'ta: Aslan 1, Yengeç 2, İkizler 3, Boğa 4, Koç 5, Balık 6, Kova 7, Oğlak 8; 7 yıl. Toplam 11 + 7 + 9 + 7 + 5 + 12 + 11 + 10 + 12 + 4 + 8 + 7 = 103.",
    "İkinci çevrim: Yengeç 1 (103→104), İkizler 5 (104→109), Boğa 3 (109→112), Koç 5 (112→117), Balık 7 (117→124; 117 < 120, üretilir). Kova 0 yıl; Oğlak 124. yılda başlardı: üretilmez.",
    "Alt dönemler (geri). Yengeç dashası: İkizler, Boğa, Koç, Balık, Kova, Oğlak, Yay, Akrep, Terazi, Başak, Aslan, Yengeç. İkizler dashası: Boğa, Koç, Balık, Kova, Oğlak, Yay, Akrep, Terazi, Başak, Aslan, Yengeç, İkizler."
  ],
  "input": {
    "lagna_lon": 100.0,
    "grahas": {"Sun": 280.0, "Moon": 130.0, "Mars": 220.0, "Mercury": 290.0, "Jupiter": 190.0, "Venus": 320.0, "Saturn": 310.0, "Rahu": 10.0, "Ketu": 190.0},
    "advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": false}
  },
  "expected": {
    "forward": false,
    "order": ["Cancer", "Gemini", "Taurus", "Aries", "Pisces", "Aquarius", "Capricorn", "Sagittarius", "Scorpio", "Libra", "Virgo", "Leo"],
    "years": [11, 7, 9, 7, 5, 12, 11, 10, 12, 4, 8, 7],
    "second_cycle": [
      {"sign": "Cancer", "years": 1},
      {"sign": "Gemini", "years": 5},
      {"sign": "Taurus", "years": 3},
      {"sign": "Aries", "years": 5},
      {"sign": "Pisces", "years": 7}
    ],
    "sub_periods": [
      ["Gemini", "Taurus", "Aries", "Pisces", "Aquarius", "Capricorn", "Sagittarius", "Scorpio", "Libra", "Virgo", "Leo", "Cancer"],
      ["Taurus", "Aries", "Pisces", "Aquarius", "Capricorn", "Sagittarius", "Scorpio", "Libra", "Virgo", "Leo", "Cancer", "Gemini"]
    ]
  }
}
````

**Yeni dosya:** `tests/golden/modules/chara_dasha/forward-aries-lagna-lord-in-second-and-twelfth.json`

````json
{
  "id": "forward-aries-lagna-lord-in-second-and-twelfth",
  "source": "hand",
  "rule_ref": "Jaimini Upadesha Sutraları, dasha bölümü; padakrama düzeni: düz sayan burçlar Koç, Boğa, İkizler, Terazi, Akrep, Yay; ters sayanlar Yengeç, Aslan, Başak, Oğlak, Kova, Balık; yıl = burçtan lorduna burcun yönünde kapsayıcı sayım − 1, lord kendi burcundaysa 12; eş lordlar BPHS ve Jaimini geleneğindeki altı ölçütle",
  "derivation": [
    "Burçlar: Lagna 5° Koç. Güneş 125° Aslan 5°, Ay 70° İkizler 10°, Mars 40° Boğa 10°, Merkür 160° Başak 10°, Jüpiter 340° Balık 10°, Venüs 15° Koç 15°, Satürn 280° Oğlak 10°, Rahu 100° Yengeç 10°, Ketu 280° Oğlak 10°.",
    "Yön: lagna Koç; Koç'tan 9. burç Yay, düz sayan gruptadır: dashalar zodyak sırasıyla ilerler. Sıra: Koç, Boğa, İkizler, Yengeç, Aslan, Başak, Terazi, Akrep, Yay, Oğlak, Kova, Balık.",
    "Eş lord, Akrep (Mars Boğa'da, Ketu Oğlak'ta): 1. ölçüt ayırmaz (ikisi de Akrep'in dışında). 2. ölçüt: Mars'ın yanında graha yok (0), Ketu'nun yanında Satürn var (1): Ketu.",
    "Eş lord, Kova (Satürn Oğlak'ta, Rahu Yengeç'te): 1. ölçüt ayırmaz. 2. ölçüt: Satürn'ün yanında Ketu var (Ketu Kova'nın eş lordu değildir, sayılır: 1), Rahu'nun yanında graha yok (0): Satürn.",
    "Yıllar (1–6). Koç (düz), Mars Boğa'da: Koç 1, Boğa 2; sayım 2, 1 yıl (lord sonraki burçta). Boğa (düz), Venüs Koç'ta: Boğa 1, İkizler 2, … Balık 11, Koç 12; sayım 12, 11 yıl (lord önceki burçta). İkizler (düz), Merkür Başak'ta: İkizler 1, Yengeç 2, Aslan 3, Başak 4; 3 yıl. Yengeç (ters), Ay İkizler'de: Yengeç 1, İkizler 2; 1 yıl (ters sayan burçta lord önceki burçta). Aslan (ters), Güneş Aslan'da: kendi burcu, 12 yıl. Başak (ters), Merkür Başak'ta: 12 yıl.",
    "Yıllar (7–12). Terazi (düz), Venüs Koç'ta: Terazi 1, Akrep 2, Yay 3, Oğlak 4, Kova 5, Balık 6, Koç 7; 6 yıl. Akrep (düz), lord Ketu Oğlak'ta: Akrep 1, Yay 2, Oğlak 3; 2 yıl. Yay (düz), Jüpiter Balık'ta: Yay 1, Oğlak 2, Kova 3, Balık 4; 3 yıl. Oğlak (ters), Satürn Oğlak'ta: 12 yıl. Kova (ters), lord Satürn Oğlak'ta: Kova 1, Oğlak 2; 1 yıl. Balık (ters), Jüpiter Balık'ta: 12 yıl. Toplam 1 + 11 + 3 + 1 + 12 + 12 + 6 + 2 + 3 + 12 + 1 + 12 = 76.",
    "İkinci çevrim (12 − yıl; başlangıç 120. yıldan önceyse üretilir): Koç 11 (76→87), Boğa 1 (87→88), İkizler 9 (88→97), Yengeç 11 (97→108), Aslan 0 ve Başak 0: atlanır, Terazi 6 (108→114), Akrep 10 (114→124; 114 < 120, üretilir). Yay 124. yılda başlardı: üretilmez.",
    "Alt dönemler (ileri; dasha burcundan sonraki burçla başlar, dasha burcuyla biter). Koç dashası: Boğa, İkizler, Yengeç, Aslan, Başak, Terazi, Akrep, Yay, Oğlak, Kova, Balık, Koç. Boğa dashası: İkizler, Yengeç, Aslan, Başak, Terazi, Akrep, Yay, Oğlak, Kova, Balık, Koç, Boğa."
  ],
  "input": {
    "lagna_lon": 5.0,
    "grahas": {"Sun": 125.0, "Moon": 70.0, "Mars": 40.0, "Mercury": 160.0, "Jupiter": 340.0, "Venus": 15.0, "Saturn": 280.0, "Rahu": 100.0, "Ketu": 280.0},
    "advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}
  },
  "expected": {
    "forward": true,
    "order": ["Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo", "Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces"],
    "years": [1, 11, 3, 1, 12, 12, 6, 2, 3, 12, 1, 12],
    "second_cycle": [
      {"sign": "Aries", "years": 11},
      {"sign": "Taurus", "years": 1},
      {"sign": "Gemini", "years": 9},
      {"sign": "Cancer", "years": 11},
      {"sign": "Libra", "years": 6},
      {"sign": "Scorpio", "years": 10}
    ],
    "sub_periods": [
      ["Taurus", "Gemini", "Cancer", "Leo", "Virgo", "Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces", "Aries"],
      ["Gemini", "Cancer", "Leo", "Virgo", "Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces", "Aries", "Taurus"]
    ]
  }
}
````

**Yeni dosya:** `tests/golden/modules/chara_dasha/lagna-sign-skipped-in-second-cycle.json`

````json
{
  "id": "lagna-sign-skipped-in-second-cycle",
  "source": "hand",
  "rule_ref": "Jaimini Upadesha Sutraları, dasha bölümü; padakrama düzeni: düz sayan burçlar Koç, Boğa, İkizler, Terazi, Akrep, Yay; ters sayanlar Yengeç, Aslan, Başak, Oğlak, Kova, Balık; yıl = burçtan lorduna burcun yönünde kapsayıcı sayım − 1, lord kendi burcundaysa 12; eş lordlar BPHS ve Jaimini geleneğindeki altı ölçütle",
  "derivation": [
    "Burçlar: Lagna 190° Terazi. Güneş 95° Yengeç 5°, Ay 215° Akrep 5°, Mars 130° Aslan 10°, Merkür 100° Yengeç 10°, Jüpiter 40° Boğa 10°, Venüs 185° Terazi 5°, Satürn 250° Yay 10°, Rahu 340° Balık 10°, Ketu 160° Başak 10°.",
    "Yön: lagna Terazi; Terazi'den 9. burç İkizler, düz sayan gruptadır: ileri. Sıra: Terazi, Akrep, Yay, Oğlak, Kova, Balık, Koç, Boğa, İkizler, Yengeç, Aslan, Başak.",
    "Eş lord, Akrep (Mars Aslan'da, Ketu Başak'ta): 1. ölçüt ayırmaz. 2. ölçüt: ikisinin de yanında graha yok (0–0). 3. ölçüt: Mars Aslan'da ne yücelmiş (Oğlak) ne düşmüş (Yengeç). 4. ölçüt: Aslan sabit, Başak çift doğalı; çift doğalı üstündür: Ketu.",
    "Eş lord, Kova (Satürn Yay'da, Rahu Balık'ta): 1. ölçüt ayırmaz. 2. ölçüt: 0–0. 3. ölçüt: Satürn Yay'da ne yücelmiş (Terazi) ne düşmüş (Koç). 4. ölçüt: Yay ve Balık çift doğalı, eşit. 5. ölçüt: Kova ters sayar; Kova'dan Yay'a Kova 1, Oğlak 2, Yay 3; Kova'dan Balık'a ters yönde 12. Uzak olan Rahu.",
    "Yıllar (1–6). Terazi (düz), Venüs Terazi'de: 12 yıl. Akrep (düz), lord Ketu Başak'ta: Akrep 1, Yay 2, … Aslan 10, Başak 11; 10 yıl. Yay (düz), Jüpiter Boğa'da: Yay 1, Oğlak 2, Kova 3, Balık 4, Koç 5, Boğa 6; 5 yıl. Oğlak (ters), Satürn Yay'da: Oğlak 1, Yay 2; 1 yıl. Kova (ters), lord Rahu Balık'ta: sayım 12, 11 yıl. Balık (ters), Jüpiter Boğa'da: Balık 1, Kova 2, Oğlak 3, Yay 4, Akrep 5, Terazi 6, Başak 7, Aslan 8, Yengeç 9, İkizler 10, Boğa 11; 10 yıl.",
    "Yıllar (7–12). Koç (düz), Mars Aslan'da: Koç 1, Boğa 2, İkizler 3, Yengeç 4, Aslan 5; 4 yıl. Boğa (düz), Venüs Terazi'de: Boğa 1 … Terazi 6; 5 yıl. İkizler (düz), Merkür Yengeç'te: İkizler 1, Yengeç 2; 1 yıl. Yengeç (ters), Ay Akrep'te: Yengeç 1, İkizler 2, Boğa 3, Koç 4, Balık 5, Kova 6, Oğlak 7, Yay 8, Akrep 9; 8 yıl. Aslan (ters), Güneş Yengeç'te: Aslan 1, Yengeç 2; 1 yıl. Başak (ters), Merkür Yengeç'te: Başak 1, Aslan 2, Yengeç 3; 2 yıl. Toplam 12 + 10 + 5 + 1 + 11 + 10 + 4 + 5 + 1 + 8 + 1 + 2 = 70.",
    "İkinci çevrim: Terazi 0 yıl, atlanır (lagna burcu ikinci çevrimde gelmez). Akrep 2 (70→72), Yay 7 (72→79), Oğlak 11 (79→90), Kova 1 (90→91), Balık 2 (91→93), Koç 8 (93→101), Boğa 7 (101→108), İkizler 11 (108→119), Yengeç 4 (119→123; 119 < 120, üretilir). Aslan 123. yılda başlardı: üretilmez.",
    "Alt dönemler (ileri). Terazi dashası: Akrep, Yay, Oğlak, Kova, Balık, Koç, Boğa, İkizler, Yengeç, Aslan, Başak, Terazi. Akrep dashası: Yay, Oğlak, Kova, Balık, Koç, Boğa, İkizler, Yengeç, Aslan, Başak, Terazi, Akrep."
  ],
  "input": {
    "lagna_lon": 190.0,
    "grahas": {"Sun": 95.0, "Moon": 215.0, "Mars": 130.0, "Mercury": 100.0, "Jupiter": 40.0, "Venus": 185.0, "Saturn": 250.0, "Rahu": 340.0, "Ketu": 160.0},
    "advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}
  },
  "expected": {
    "forward": true,
    "order": ["Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces", "Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo"],
    "years": [12, 10, 5, 1, 11, 10, 4, 5, 1, 8, 1, 2],
    "second_cycle": [
      {"sign": "Scorpio", "years": 2},
      {"sign": "Sagittarius", "years": 7},
      {"sign": "Capricorn", "years": 11},
      {"sign": "Aquarius", "years": 1},
      {"sign": "Pisces", "years": 2},
      {"sign": "Aries", "years": 8},
      {"sign": "Taurus", "years": 7},
      {"sign": "Gemini", "years": 11},
      {"sign": "Cancer", "years": 4}
    ],
    "sub_periods": [
      ["Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces", "Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo", "Libra"],
      ["Sagittarius", "Capricorn", "Aquarius", "Pisces", "Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo", "Libra", "Scorpio"]
    ]
  }
}
````

**Yeni dosya:** `tests/golden/modules/chara_dasha/time-unknown-no-dasha.json`

````json
{
  "id": "time-unknown-no-dasha",
  "source": "hand",
  "rule_ref": "Jaimini Upadesha Sutraları, dasha bölümü; padakrama düzeni: düz sayan burçlar Koç, Boğa, İkizler, Terazi, Akrep, Yay; ters sayanlar Yengeç, Aslan, Başak, Oğlak, Kova, Balık; yıl = burçtan lorduna burcun yönünde kapsayıcı sayım − 1, lord kendi burcundaysa 12; eş lordlar BPHS ve Jaimini geleneğindeki altı ölçütle",
  "derivation": [
    "Burçlar: lagna yok (doğum saati bilinmiyor). Grahalar `forward-aries-lagna-lord-in-second-and-twelfth` örneğindeki gibidir: Güneş 125° Aslan, Ay 70° İkizler, Mars 40° Boğa, Merkür 160° Başak, Jüpiter 340° Balık, Venüs 15° Koç, Satürn 280° Oğlak, Rahu 100° Yengeç, Ketu 280° Oğlak.",
    "İlk dasha lagna burcudur ve yön lagnadan 9. burca bağlıdır. Lagna olmadan sıra kurulamaz: chara dasha hesaplanmaz, sonuç boştur."
  ],
  "input": {
    "lagna_lon": null,
    "grahas": {"Sun": 125.0, "Moon": 70.0, "Mars": 40.0, "Mercury": 160.0, "Jupiter": 340.0, "Venus": 15.0, "Saturn": 280.0, "Rahu": 100.0, "Ketu": 280.0},
    "advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}
  },
  "expected": null
}
````

`README.md`'ye modül satırı ve chara dashanın zaman girdisi eklenir:

````diff
diff --git a/tests/golden/modules/README.md b/tests/golden/modules/README.md
index 2ff0bce..c5a8e81 100644
--- a/tests/golden/modules/README.md
+++ b/tests/golden/modules/README.md
@@ -32,9 +32,10 @@ ileri modüller tasarım belgesi §13 (dört katmanlı test stratejisinin ilk ka
 
 `lagna_lon: null` doğum saati bilinmeyen haritadır: lagnaya bağlı her beklenen değer `null` olur.
 
-Ayara bağlı modüller (Jaimini) girdiye `advanced` bloğunu ekler:
+Ayara bağlı modüller (Jaimini, chara dasha) girdiye `advanced` bloğunu ekler:
 `"advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}`. Diğer modüllerde bu alan
-yazılmaz.
+yazılmaz. Chara dasha örneklerinde doğum anı ve yıl uzunluğu girdide yoktur: test sabit bir
+doğum anı ve 365,25 günlük yıl kullanır, tarihleri el yıllarının birikimli toplamından kurar.
 
 ## Modüller
 
@@ -43,5 +44,6 @@ yazılmaz.
 | `foundation` | `waxing`, `nature` (dokuz graha), `kuja_dosha`, `house_lords` (12), `houses`, `houses_owned` | 6 |
 | `drishti` | `sign_aspects` (on üç tam bakışın tamamı, tablo sırasıyla), `sphuta` (en az on değer: `from`, `to`, `angle`, `value`) | 4 |
 | `jaimini` | `karakas` (rol sırasında: `role`, `graha`, `effective_deg`, `tie`), `karakamsha` (`atmakaraka`, `sign`, dokuz grahanın `houses_in_d1` ve `houses_in_d9` değeri), `co_lords` (Akrep ve Kova: `lord`, belirleyici ölçüt `step`), `arudhas` (on iki pada: `house`, `sign`, `lord`, `exception`; saat bilinmiyorsa `null`), `argala` (en az iki referans burç: dört çiftin iki graha listesi ve durumu, `third_house_malefic_argala`) | 8 |
+| `chara_dasha` | `forward`, `order` (dasha sırası, lagnadan), `years` (ilk çevrim, `order` sırasında), `second_cycle` (ikinci çevrimde üretilen dashalar: `sign`, `years`), `sub_periods` (ilk iki dashanın alt dönem sırası); saat bilinmiyorsa `expected` `null` | 7 |
 
 Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
````

- [ ] **Step 3: Başarısız testleri yaz (el örneği düzeneği ve değişmezler)**

`modules_golden.rs` iki test kazanır. `chara_dasha_hand_fixtures` her örnekte yönü, sırayı, on iki yılı (hem `CharaDasha.years` hem burç başına `chara_dasha_sign_years`), dashaların sırasını ve tarihlerini (el yıllarının birikimli toplamından), ilk iki dashanın alt dönem sırasını, on ikide bir süreyi ve 3. seviyenin aynı kuralı izlediğini denetler. `chara_dasha_fixtures_cover_the_required_cases` spec'in zorunlu durumlarının örneklerde bulunduğunu sabitler ve motorun grup ve lord fonksiyonları yerine elle yazılmış iki liste kullanır.

````diff
diff --git a/crates/jyotish-core/tests/modules_golden.rs b/crates/jyotish-core/tests/modules_golden.rs
index a35fda9..6f575e3 100644
--- a/crates/jyotish-core/tests/modules_golden.rs
+++ b/crates/jyotish-core/tests/modules_golden.rs
@@ -7,10 +7,11 @@ use std::collections::BTreeMap;
 use std::path::PathBuf;
 
 use jyotish_core::{
-    argala_rows, arudha_padas, chara_karakas, co_lord_choice, drishti_table, house_lord,
-    houses_owned, jaimini_lord, jaimini_report, karakamsha, kuja_dosha, moon_is_waxing, nature,
-    normalize, AdvancedSettings, ArgalaStatus, Graha, KarakaRole, KujaDosha, Nature, Positions,
-    Sign, ASPECTING_GRAHAS, DRISHTI_METHOD, JAIMINI_METHOD,
+    argala_rows, arudha_padas, chara_dasha, chara_dasha_order, chara_dasha_sign_years,
+    chara_karakas, co_lord_choice, drishti_table, house_lord, houses_owned, jaimini_lord,
+    jaimini_report, karakamsha, kuja_dosha, moon_is_waxing, nature, normalize, sign_dasha_children,
+    AdvancedSettings, ArgalaStatus, Graha, KarakaRole, KujaDosha, Nature, Positions, Sign,
+    SignDashaPeriod, ASPECTING_GRAHAS, CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD,
 };
 use serde::de::DeserializeOwned;
 use serde::Deserialize;
@@ -19,6 +20,7 @@ use serde::Deserialize;
 const MIN_FOUNDATION: usize = 6;
 const MIN_DRISHTI: usize = 4;
 const MIN_JAIMINI: usize = 8;
+const MIN_CHARA_DASHA: usize = 7;
 /// El hesabı ile motor arasındaki en büyük fark, virupa.
 const VIRUPA_TOL: f64 = 1e-9;
 
@@ -34,7 +36,7 @@ struct Fixture<X> {
 }
 
 /// Sentetik konumlar: sidereal boylamlar, derece. `lagna_lon: null` doğum saati bilinmeyen
-/// haritadır. `advanced` yalnızca ayara bağlı modüllerde (Jaimini) verilir.
+/// haritadır. `advanced` yalnızca ayara bağlı modüllerde (Jaimini, chara dasha) verilir.
 #[derive(Debug, Deserialize)]
 #[serde(deny_unknown_fields)]
 struct Input {
@@ -510,3 +512,270 @@ fn jaimini_fixtures_cover_the_required_cases() {
 fn house_sign_of(lagna: Sign, house: u8) -> Sign {
     Sign::ALL[(lagna as usize + usize::from(house) - 1) % 12]
 }
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ExpectedSecondCycle {
+    sign: Sign,
+    years: u8,
+}
+
+/// Chara dasha el örneği. Doğum saati bilinmeyen haritada `expected` `null`'dır.
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct CharaDashaExpected {
+    /// Dashalar zodyak sırasıyla mı ilerliyor.
+    forward: bool,
+    /// Dasha sırası, lagna burcundan.
+    order: [Sign; 12],
+    /// İlk çevrimin yılları, `order` sırasında.
+    years: [u8; 12],
+    /// İkinci çevrimde üretilen dashalar, sırasıyla (0 yıllık burçlar ve 120. yıldan sonra
+    /// başlayanlar yoktur).
+    second_cycle: Vec<ExpectedSecondCycle>,
+    /// İlk iki dashanın alt dönem sırası.
+    sub_periods: [[Sign; 12]; 2],
+}
+
+/// El örneklerinde doğum anı ve yıl uzunluğu sabittir; tarihler el yıllarından kurulur.
+const CHARA_BIRTH_JD: f64 = 2_451_545.0;
+const CHARA_YEAR_DAYS: f64 = 365.25;
+
+fn chara_fixtures() -> Vec<(
+    Fixture<Option<CharaDashaExpected>>,
+    Positions,
+    AdvancedSettings,
+)> {
+    load::<Option<CharaDashaExpected>>("chara_dasha", MIN_CHARA_DASHA)
+        .into_iter()
+        .map(|(f, p)| {
+            let advanced = f.input.advanced.unwrap_or_else(|| {
+                panic!("{}: chara dasha fixtures state the advanced settings", f.id)
+            });
+            (f, p, advanced)
+        })
+        .collect()
+}
+
+#[test]
+fn chara_dasha_hand_fixtures() {
+    for (f, p, advanced) in chara_fixtures() {
+        let id = &f.id;
+        let co_lords = advanced.jaimini_co_lords;
+        let got = chara_dasha(&p, co_lords, CHARA_BIRTH_JD, CHARA_YEAR_DAYS);
+        let Some(x) = &f.expected else {
+            assert!(p.lagna_sign.is_none(), "{id}: null only without a lagna");
+            assert_eq!(got, None, "{id}: no dasha without a lagna");
+            continue;
+        };
+        let lagna = p
+            .lagna_sign
+            .unwrap_or_else(|| panic!("{id}: a lagna is required"));
+        let d = got.unwrap_or_else(|| panic!("{id}: a dasha with a lagna"));
+        assert_eq!(d.method, CHARA_DASHA_METHOD, "{id}");
+
+        // Yön, sıra ve yıllar.
+        assert_eq!(d.forward, x.forward, "{id}: direction");
+        assert_eq!(x.order[0], lagna, "{id}: the first dasha is the lagna sign");
+        assert_eq!(chara_dasha_order(lagna), x.order, "{id}: order");
+        assert_eq!(d.years, x.years, "{id}: years");
+        for (&sign, &years) in x.order.iter().zip(&x.years) {
+            assert_eq!(
+                chara_dasha_sign_years(sign, &p, co_lords),
+                years,
+                "{id}: years of {sign:?}"
+            );
+        }
+
+        // Dashalar: ilk çevrimin on iki burcu, sonra ikinci çevrimde üretilenler; tarihler el
+        // yıllarının birikimli toplamından.
+        let mut want: Vec<(u8, Sign, u8)> = x
+            .order
+            .iter()
+            .zip(&x.years)
+            .map(|(&sign, &years)| (1, sign, years))
+            .collect();
+        want.extend(x.second_cycle.iter().map(|c| (2, c.sign, c.years)));
+        let dashas: Vec<&SignDashaPeriod> = d.periods.iter().filter(|q| q.level == 1).collect();
+        assert_eq!(dashas.len(), want.len(), "{id}: dasha count");
+        let mut elapsed = 0.0;
+        for (got, &(cycle, sign, years)) in dashas.iter().zip(&want) {
+            assert_eq!(
+                (got.cycle, got.signs.as_slice()),
+                (cycle, &[sign][..]),
+                "{id}: dasha after {elapsed} years"
+            );
+            let start = CHARA_BIRTH_JD + elapsed * CHARA_YEAR_DAYS;
+            elapsed += f64::from(years);
+            let end = CHARA_BIRTH_JD + elapsed * CHARA_YEAR_DAYS;
+            assert!(
+                (got.start_jd_ut - start).abs() < 1e-6 && (got.end_jd_ut - end).abs() < 1e-6,
+                "{id}: {sign:?} (cycle {cycle}) runs {} – {}, hand value {start} – {end}",
+                got.start_jd_ut,
+                got.end_jd_ut
+            );
+        }
+
+        // İlk iki dashanın alt dönemleri: sıra, on iki eşit parça, 3. seviyede aynı kural.
+        for (i, subs) in x.sub_periods.iter().enumerate() {
+            let dasha_sign = x.order[i];
+            assert_eq!(subs[11], dasha_sign, "{id}: last sub-period");
+            let kids = sign_dasha_children(&d, &[dasha_sign], 1).expect("sub-periods");
+            let signs: Vec<Sign> = kids.iter().map(|k| k.signs[1]).collect();
+            assert_eq!(signs, subs, "{id}: sub-periods of {dasha_sign:?}");
+            let part = f64::from(x.years[i]) * CHARA_YEAR_DAYS / 12.0;
+            for k in &kids {
+                assert!(
+                    (k.end_jd_ut - k.start_jd_ut - part).abs() < 1e-6,
+                    "{id}: {:?} is one twelfth of the dasha",
+                    k.signs
+                );
+            }
+            // 3. seviye: ilk alt dönemin alt dönemleri ondan sonraki burçla başlar (bu, 2.
+            // seviyenin ikinci burcudur) ve kendi burcuyla biter.
+            let third = sign_dasha_children(&d, &[dasha_sign, subs[0]], 1).expect("third level");
+            assert_eq!(third.len(), 12, "{id}");
+            assert_eq!(third[0].signs[2], subs[1], "{id}: third level starts");
+            assert_eq!(third[11].signs[2], subs[0], "{id}: third level ends");
+        }
+    }
+}
+
+/// Elle yazılmış listeler (motorun `counts_forward` ve `sign_lord` fonksiyonları yerine).
+const HAND_FORWARD_GROUP: [Sign; 6] = [
+    Sign::Aries,
+    Sign::Taurus,
+    Sign::Gemini,
+    Sign::Libra,
+    Sign::Scorpio,
+    Sign::Sagittarius,
+];
+/// Tek lordlu on burcun lordu (Akrep ve Kova eş lordludur, burada yoktur).
+const HAND_SINGLE_LORDS: [(Sign, Graha); 10] = [
+    (Sign::Aries, Graha::Mars),
+    (Sign::Taurus, Graha::Venus),
+    (Sign::Gemini, Graha::Mercury),
+    (Sign::Cancer, Graha::Moon),
+    (Sign::Leo, Graha::Sun),
+    (Sign::Virgo, Graha::Mercury),
+    (Sign::Libra, Graha::Venus),
+    (Sign::Sagittarius, Graha::Jupiter),
+    (Sign::Capricorn, Graha::Saturn),
+    (Sign::Pisces, Graha::Jupiter),
+];
+
+/// Zorunlu durumlar (ileri modüller spec'i §6 "Test"): örnekler silinir ya da değiştirilirse
+/// kapsam sessizce daralmasın.
+#[test]
+fn chara_dasha_fixtures_cover_the_required_cases() {
+    let all = chara_fixtures();
+    type Case<'a> = (&'a CharaDashaExpected, &'a Positions, &'a AdvancedSettings);
+    let any = |pred: &dyn Fn(Case) -> bool| {
+        all.iter()
+            .any(|(f, p, a)| f.expected.as_ref().is_some_and(|x| pred((x, p, a))))
+    };
+    assert!(any(&|(x, _, _)| x.forward), "forward direction");
+    assert!(any(&|(x, _, _)| !x.forward), "backward direction");
+    assert!(
+        all.iter().any(|(f, _, _)| f.expected.is_none()),
+        "birth time unknown"
+    );
+
+    // Tek lordlu bir burçta lordun burçtan `offset` burç ötede olduğu ve yılın `years` olduğu
+    // örnek. `offset` 0: kendi burcu; 1: sonraki burç (2. burç); 11: önceki burç (12. burç).
+    let lord_case = |forward_group: bool, offset: usize, years: u8| {
+        any(&|(x, p, _)| {
+            x.order.iter().zip(&x.years).any(|(&sign, &y)| {
+                let Some(&(_, lord)) = HAND_SINGLE_LORDS.iter().find(|(s, _)| *s == sign) else {
+                    return false;
+                };
+                HAND_FORWARD_GROUP.contains(&sign) == forward_group
+                    && p.sign_of(lord) == Sign::ALL[(sign as usize + offset) % 12]
+                    && y == years
+            })
+        })
+    };
+    for forward_group in [true, false] {
+        assert!(
+            lord_case(forward_group, 0, 12),
+            "lord in its own sign (forward group: {forward_group})"
+        );
+    }
+    assert!(
+        lord_case(true, 1, 1),
+        "forward sign, lord in the second: 1 year"
+    );
+    assert!(
+        lord_case(true, 11, 11),
+        "forward sign, lord in the twelfth: 11 years"
+    );
+    assert!(
+        lord_case(false, 1, 11),
+        "reverse sign, lord in the second: 11 years"
+    );
+    assert!(
+        lord_case(false, 11, 1),
+        "reverse sign, lord in the twelfth: 1 year"
+    );
+
+    // Akrep ve Kova'da eş lord seçimi: düğümün seçildiği, graha lordun seçildiği, iki eş
+    // lordun da burcun içinde olduğu (12 yıl) ve ayarın devre dışı olduğu örnekler.
+    let years_of = |x: &CharaDashaExpected, sign: Sign| {
+        x.years[x.order.iter().position(|s| *s == sign).expect("in order")]
+    };
+    for (sign, planet, node) in [
+        (Sign::Scorpio, Graha::Mars, Graha::Ketu),
+        (Sign::Aquarius, Graha::Saturn, Graha::Rahu),
+    ] {
+        for lord in [planet, node] {
+            assert!(
+                any(&|(_, p, a)| a.jaimini_co_lords
+                    && p.sign_of(planet) != p.sign_of(node)
+                    && co_lord_choice(sign, p).is_some_and(|c| c.lord == lord)),
+                "{lord:?} chosen for {sign:?}"
+            );
+        }
+        assert!(
+            any(&|(x, p, a)| a.jaimini_co_lords
+                && p.sign_of(planet) == sign
+                && p.sign_of(node) == sign
+                && years_of(x, sign) == 12),
+            "both co-lords in {sign:?}: 12 years"
+        );
+    }
+    assert!(
+        any(&|(x, p, a)| !a.jaimini_co_lords
+            && p.sign_of(Graha::Mars) == Sign::Scorpio
+            && p.sign_of(Graha::Ketu) != Sign::Scorpio
+            && years_of(x, Sign::Scorpio) == 12),
+        "co-lords switched off: Mars in Scorpio gives 12 years"
+    );
+
+    // İkinci çevrim: 12 yıllık bir burç atlanır ve ondan sonraki bir burç üretilir; lagna
+    // burcunun atlandığı örnek; çevrimin tam 120. yılda ve 120. yıldan sonra bittiği örnekler.
+    assert!(
+        any(&|(x, _, _)| {
+            let skipped = x.years.iter().position(|y| *y == 12).expect("a 12");
+            x.second_cycle
+                .iter()
+                .any(|c| x.order.iter().position(|s| *s == c.sign) > Some(skipped))
+        }),
+        "a sign skipped inside the second cycle"
+    );
+    assert!(
+        any(&|(x, _, _)| x.years[0] == 12 && x.second_cycle[0].sign != x.order[0]),
+        "the lagna sign skipped in the second cycle"
+    );
+    let total = |x: &CharaDashaExpected| {
+        x.years.iter().map(|y| u32::from(*y)).sum::<u32>()
+            + x.second_cycle
+                .iter()
+                .map(|c| u32::from(c.years))
+                .sum::<u32>()
+    };
+    assert!(
+        any(&|(x, _, _)| total(x) == 120),
+        "ends exactly at 120 years"
+    );
+    assert!(any(&|(x, _, _)| total(x) > 120), "ends after 120 years");
+}
````

`proptests.rs`'e `chara_dasha_properties` eklenir (dosyanın sonuna): lagna yoksa dasha yoktur; ilk çevrim lagnadan başlayan bir permütasyondur, her süre 1–12 yıldır ve toplam 120'nin altındadır; dashalar boşluksuz ardışıktır, iki çevrimde aynı burcun yılları toplamı 12'dir ve üretim 120. yıla ulaşan ilk dashada durur; alt dönemler iki seviyede de ebeveyni on iki eşit parçaya böler ve ebeveynin burcuyla biter; bir anın yolu o anı içeren dönemleri adlandırır.

````diff
diff --git a/crates/jyotish-core/src/proptests.rs b/crates/jyotish-core/src/proptests.rs
index 2a8c2ae..5fe5080 100644
--- a/crates/jyotish-core/src/proptests.rs
+++ b/crates/jyotish-core/src/proptests.rs
@@ -671,3 +671,182 @@ mod jaimini_properties {
         }
     }
 }
+
+mod chara_dasha_properties {
+    use proptest::prelude::*;
+
+    use super::foundation_properties::positions;
+    use crate::jaimini::{
+        chara_dasha, chara_dasha_years, jaimini_lord, second_cycle_years, sign_dasha_children,
+        sign_dasha_path_at, SignDashaPeriod, CHARA_DASHA_HORIZON_YEARS, CHARA_DASHA_METHOD,
+    };
+    use crate::zodiac::Sign;
+
+    /// JD ~2.4e6 civarında f64 çözünürlüğü ~5e-10 gün; 1e-6 gün (~0.09 sn) bol paydır.
+    const TOL_DAYS: f64 = 1e-6;
+
+    fn birth() -> impl Strategy<Value = (f64, f64)> {
+        (
+            2_300_000.0f64..2_500_000.0,
+            prop::sample::select(vec![360.0, 365.2422, 365.25]),
+        )
+    }
+
+    /// Alt dönemler ebeveyni boşluksuz ve örtüşmesiz on iki eşit parçaya böler; sonuncusu
+    /// ebeveynin burcudur ve hiçbir burç yinelenmez.
+    fn check_partition(
+        parent: &SignDashaPeriod,
+        children: &[SignDashaPeriod],
+    ) -> Result<(), TestCaseError> {
+        prop_assert_eq!(children.len(), 12);
+        prop_assert_eq!(children[0].start_jd_ut, parent.start_jd_ut);
+        prop_assert_eq!(children[11].end_jd_ut, parent.end_jd_ut);
+        for w in children.windows(2) {
+            prop_assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
+        }
+        let span = parent.end_jd_ut - parent.start_jd_ut;
+        let mut sum = 0.0;
+        let mut seen = [false; 12];
+        for c in children {
+            prop_assert_eq!(c.level, parent.level + 1);
+            prop_assert_eq!(c.cycle, parent.cycle);
+            prop_assert_eq!(&c.signs[..parent.signs.len()], &parent.signs[..]);
+            let d = c.end_jd_ut - c.start_jd_ut;
+            prop_assert!((d - span / 12.0).abs() < TOL_DAYS, "{:?}: {}", c.signs, d);
+            sum += d;
+            seen[c.signs.last().expect("sign").index()] = true;
+        }
+        prop_assert!((sum - span).abs() < TOL_DAYS);
+        prop_assert!(seen.iter().all(|s| *s));
+        prop_assert_eq!(children[11].signs.last(), parent.signs.last());
+        Ok(())
+    }
+
+    proptest! {
+        #[test]
+        fn no_dasha_without_a_lagna(p in positions(), co in any::<bool>(), (jd, year) in birth()) {
+            let d = chara_dasha(&p, co, jd, year);
+            prop_assert_eq!(d.is_some(), p.lagna_sign.is_some());
+        }
+
+        #[test]
+        fn first_cycle_is_a_permutation_from_the_lagna(
+            p in positions(),
+            co in any::<bool>(),
+            (jd, year) in birth(),
+        ) {
+            let Some(lagna) = p.lagna_sign else { return Ok(()); };
+            let d = chara_dasha(&p, co, jd, year).expect("lagna");
+            prop_assert_eq!(d.method.as_str(), CHARA_DASHA_METHOD);
+            let dashas: Vec<&SignDashaPeriod> = d.periods.iter().filter(|q| q.level == 1).collect();
+            let first: Vec<&SignDashaPeriod> =
+                dashas.iter().copied().filter(|q| q.cycle == 1).collect();
+            prop_assert_eq!(first.len(), 12);
+            prop_assert_eq!(first[0].signs[0], lagna);
+            prop_assert_eq!(first[0].start_jd_ut, jd);
+            let mut seen = [false; 12];
+            for (i, q) in first.iter().enumerate() {
+                let sign = q.signs[0];
+                seen[sign.index()] = true;
+                // Sıra tek yönde birer burç ilerler.
+                let step = if d.forward { i } else { 12 - i };
+                prop_assert_eq!(sign, Sign::from_index(lagna.index() + step));
+                // Her süre 1–12 yıldır ve burçtan lorduna sayımdan gelir.
+                let years = d.years[i];
+                prop_assert!((1..=12).contains(&years));
+                let lord_sign = p.sign_of(jaimini_lord(sign, &p, co));
+                prop_assert_eq!(years, chara_dasha_years(sign, lord_sign));
+                prop_assert_eq!(years == 12, lord_sign == sign);
+                let length = q.end_jd_ut - q.start_jd_ut;
+                prop_assert!((length - f64::from(years) * year).abs() < TOL_DAYS);
+            }
+            prop_assert!(seen.iter().all(|s| *s));
+            // İlk çevrim 120 yıla ulaşmaz: on iki dasha her zaman tamdır.
+            let total: u16 = d.years.iter().map(|y| u16::from(*y)).sum();
+            prop_assert!((12..CHARA_DASHA_HORIZON_YEARS).contains(&total), "{}", total);
+        }
+
+        #[test]
+        fn dashas_are_contiguous_and_the_second_cycle_completes_twelve_years(
+            p in positions(),
+            co in any::<bool>(),
+            (jd, year) in birth(),
+        ) {
+            let Some(d) = chara_dasha(&p, co, jd, year) else { return Ok(()); };
+            let dashas: Vec<&SignDashaPeriod> = d.periods.iter().filter(|q| q.level == 1).collect();
+            for w in dashas.windows(2) {
+                prop_assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
+                prop_assert!(w[0].cycle <= w[1].cycle);
+            }
+            let order: Vec<Sign> = dashas.iter().filter(|q| q.cycle == 1).map(|q| q.signs[0]).collect();
+            let horizon = jd + f64::from(CHARA_DASHA_HORIZON_YEARS) * year;
+            let mut last_index = None;
+            for q in dashas.iter().filter(|q| q.cycle == 2) {
+                let i = order.iter().position(|s| *s == q.signs[0]).expect("in the first cycle");
+                // Aynı sıra; 0 yıllık burçlar atlanır.
+                prop_assert!(last_index < Some(i));
+                for skipped in last_index.map_or(0, |l: usize| l + 1)..i {
+                    prop_assert_eq!(d.years[skipped], 12);
+                }
+                last_index = Some(i);
+                // İki çevrimde aynı burcun yılları toplamı 12'dir.
+                let years = (q.end_jd_ut - q.start_jd_ut) / year;
+                prop_assert!((years - f64::from(second_cycle_years(d.years[i]))).abs() < 1e-6);
+                prop_assert!((years + f64::from(d.years[i]) - 12.0).abs() < 1e-6);
+                // Başlangıcı 120. yıldan önce olan dasha üretilir.
+                prop_assert!(q.start_jd_ut < horizon - TOL_DAYS);
+            }
+            // Üretim 120. yıla ulaşan ilk dashada durur.
+            let end = dashas.last().expect("dashas").end_jd_ut;
+            prop_assert!(end > horizon - TOL_DAYS);
+        }
+
+        #[test]
+        fn sub_periods_partition_their_parent_at_both_levels(
+            p in positions(),
+            co in any::<bool>(),
+            (jd, year) in birth(),
+            pick in 0usize..24,
+            sub in 0usize..12,
+        ) {
+            let Some(d) = chara_dasha(&p, co, jd, year) else { return Ok(()); };
+            let dashas: Vec<&SignDashaPeriod> = d.periods.iter().filter(|q| q.level == 1).collect();
+            let parent = dashas[pick % dashas.len()];
+            let second = sign_dasha_children(&d, &parent.signs, parent.cycle).unwrap();
+            check_partition(parent, &second)?;
+            // İlk alt dönem dasha burcundan sonraki burçtur (haritanın yönünde).
+            let dasha_sign = parent.signs[0];
+            let step = if d.forward { 1 } else { 11 };
+            prop_assert_eq!(second[0].signs[1], Sign::from_index(dasha_sign.index() + step));
+            let child = &second[sub];
+            let third = sign_dasha_children(&d, &child.signs, child.cycle).unwrap();
+            check_partition(child, &third)?;
+            prop_assert!(third.iter().all(|q| !q.has_children));
+            prop_assert_eq!(third[0].signs[2], Sign::from_index(child.signs[1].index() + step));
+        }
+
+        #[test]
+        fn path_at_names_the_periods_that_contain_the_moment(
+            p in positions(),
+            co in any::<bool>(),
+            (jd, year) in birth(),
+            fraction in 0.0f64..1.0,
+        ) {
+            let Some(d) = chara_dasha(&p, co, jd, year) else { return Ok(()); };
+            let end = d.periods.iter().rfind(|q| q.level == 1).expect("dashas").end_jd_ut;
+            let at = jd + (end - jd) * fraction;
+            let path = sign_dasha_path_at(&d, at).unwrap().expect("inside the produced span");
+            prop_assert_eq!(path.signs.len(), 3);
+            for depth in 1..=3usize {
+                let period = sign_dasha_children(&d, &path.signs[..depth - 1], path.cycle)
+                    .unwrap()
+                    .into_iter()
+                    .find(|q| q.signs == path.signs[..depth])
+                    .expect("on the path");
+                prop_assert!(period.start_jd_ut <= at && at < period.end_jd_ut);
+            }
+            prop_assert_eq!(sign_dasha_path_at(&d, jd - 1.0).unwrap(), None);
+            prop_assert_eq!(sign_dasha_path_at(&d, end).unwrap(), None);
+        }
+    }
+}
````

- [ ] **Step 4: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-core 2>&1 | grep -E "^error" | sort | uniq -c
```
Expected: derleme hatası; satırlar arasında `error[E0432]: unresolved imports` (`crate::jaimini::chara_dasha`, `chara_dasha_years`, `sign_dasha_children`, `SignDashaPeriod`, `CHARA_DASHA_METHOD` …) ve `could not compile` vardır (tip çıkarımına bağlı bir `E0689` satırı da görülebilir; aynı eksik adlardan doğar).

- [ ] **Step 5: Chara dasha alt modülünü yaz**

Dosya üç bölümdür: kural fonksiyonları (yön, sıra, yıl, alt dönem sırası), dönemlerin kurulması (`chara_dasha`, `sign_dasha_children`, `sign_dasha_path_at`) ve üç toplayıcı. Birim testleri elle yazılmış 144 hücrelik yıl tablosunu (`HAND_YEARS`) taşır: tablodan yıl satır ve sütun aranarak okunur, formül kullanılmaz.

**Yeni dosya:** `crates/jyotish-core/src/jaimini/chara_dasha.rs`

````rust
//! Chara dasha, padakrama düzeni (ileri modüller spec'i §6). Burç temelli Jaimini dashası:
//! on iki burcun her biri bir dönemdir; sıra ve süre burç sayımından çıkar.
//!
//! Kaynak: Jaimini Upadesha Sutraları, dasha bölümü; 20. yüzyıl yorum geleneğinin padakrama
//! düzeni. Kural adımları:
//! 1. İlk dasha lagna burcudur. Lagnadan 9. burç düz sayan gruptaysa (`counts_forward`) dashalar
//!    zodyak sırasıyla, değilse ters sırayla ilerler.
//! 2. Bir burcun süresi, burçtan lorduna burcun kendi yönünde kapsayıcı sayımın bir eksiğidir;
//!    lord kendi burcundaysa 12 yıldır. Akrep ve Kova'nın lordu `jaimini_lord` ile seçilir.
//! 3. On iki dashadan sonra ikinci çevrim gelir: her burcun süresi `12 − ilk çevrimdeki yıl`dır
//!    ve 0 yıllık burçlar atlanır. İkinci çevrimin bir dashası, başlangıcı doğumdan sonraki
//!    120. yıldan önceyse üretilir.
//! 4. Her dönem on iki eşit alt döneme bölünür; sıra haritanın yönündedir, ilk alt dönem dasha
//!    burcundan sonraki burç, sonuncusu dasha burcunun kendisidir. En çok üç seviye.
//!
//! Doğumda ilk dasha tam süresiyle başlar. Zamanlar UT Jülyen günüdür. Yücelme ve düşüş için
//! yıl düzeltmesi uygulanmaz.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::compute_chart;
use crate::positions::{ChartInput, Positions};
use crate::zodiac::Sign;
use crate::CoreError;

use super::lords::jaimini_lord;
use super::rashi::{counts_forward, directed_count};

/// `CharaDasha::method`: bu modüldeki yöntemin kimliği.
pub const CHARA_DASHA_METHOD: &str = "chara_dasha.padakrama_v1";
/// 1 dasha, 2 alt dönem, 3 alt-alt dönem.
pub const MAX_SIGN_DASHA_LEVEL: u8 = 3;
/// `CharaDasha::periods`'un taşıdığı seviye sayısı; 3. seviye `sign_dasha_children` ile istenir.
pub const CHARA_DASHA_LEVELS: u8 = 2;
/// İkinci çevrimin bir dashası, başlangıcı doğumdan bu kadar yıl geçmeden önceyse üretilir.
pub const CHARA_DASHA_HORIZON_YEARS: u16 = 120;

/// Burç dashasının bir dönemi. Zamanlar UT Jülyen günüdür.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SignDashaPeriod {
    /// Kökten bu döneme burçlar: `[dasha]`, `[dasha, alt dönem]`, …
    pub signs: Vec<Sign>,
    /// 1..=3; `signs.len()` ile aynı.
    pub level: u8,
    /// 1 ya da 2: dönemin ait olduğu çevrim.
    pub cycle: u8,
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
    /// Alt dönemleri istenebilir (`level < MAX_SIGN_DASHA_LEVEL`).
    pub has_children: bool,
}

/// `compute_chara_dasha` sonucu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CharaDasha {
    /// Dashalar zodyak sırasıyla mı ilerliyor (lagnadan 9. burç düz sayan gruptaysa).
    pub forward: bool,
    /// İlk çevrimin yılları, lagnadan dasha sırasıyla.
    pub years: [u8; 12],
    /// İlk iki seviye, ön-sırada: her dashanın ardından on iki alt dönemi. İki çevrim de
    /// buradadır; `cycle` ayırır.
    pub periods: Vec<SignDashaPeriod>,
    pub method: String,
}

/// Bir anı içeren dönemlerin yolu: çevrim ve kökten burçlar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SignDashaPath {
    pub cycle: u8,
    pub signs: Vec<Sign>,
}

/// Dashalar zodyak sırasıyla mı ilerler: lagnadan 9. burç düz sayan gruptaysa evet.
pub fn chara_dasha_forward(lagna: Sign) -> bool {
    counts_forward(Sign::from_index(lagna.index() + 8))
}

/// `from`'dan başlayan on iki burç: ileri yönde zodyak sırasıyla, geri yönde ters sırayla.
pub fn sign_sequence(from: Sign, forward: bool) -> [Sign; 12] {
    std::array::from_fn(|k| {
        let step = if forward { k } else { 12 - k };
        Sign::from_index(from.index() + step)
    })
}

/// Dasha sırası: lagna burcundan başlar, on iki burç birer kez gelir.
pub fn chara_dasha_order(lagna: Sign) -> [Sign; 12] {
    sign_sequence(lagna, chara_dasha_forward(lagna))
}

/// Yıl kuralı: `sign`'dan lordunun burcuna, `sign`'ın kendi yönünde kapsayıcı sayım eksi bir;
/// lord kendi burcundaysa 12. Sonuç 1..=12.
pub fn chara_dasha_years(sign: Sign, lord_sign: Sign) -> u8 {
    match directed_count(sign, lord_sign) - 1 {
        0 => 12,
        n => n,
    }
}

/// `sign` burcunun ilk çevrimdeki yılı. Akrep ve Kova'nın lordu `jaimini_lord` ile seçilir;
/// iki eş lord da burcun içindeyse seçilen lord burcun içindedir ve süre 12 yıldır.
pub fn chara_dasha_sign_years(sign: Sign, positions: &Positions, co_lords: bool) -> u8 {
    let lord = jaimini_lord(sign, positions, co_lords);
    chara_dasha_years(sign, positions.sign_of(lord))
}

/// İkinci çevrimdeki yıl: `12 − ilk çevrimdeki yıl` (0: burç atlanır).
pub fn second_cycle_years(first_cycle_years: u8) -> u8 {
    12 - first_cycle_years
}

/// Alt dönem sırası: `sign`'dan sonraki burçtan başlar, `sign`'ın kendisiyle biter.
pub fn sub_period_signs(sign: Sign, forward: bool) -> [Sign; 12] {
    let sequence = sign_sequence(sign, forward);
    std::array::from_fn(|k| sequence[(k + 1) % 12])
}

/// Dönemi on iki eşit alt döneme böler. Bitişler baştan hesaplanır; sonuncusu ebeveynin
/// bitişine tam eşittir.
fn subdivide(parent: &SignDashaPeriod, forward: bool) -> Vec<SignDashaPeriod> {
    let level = parent.level + 1;
    let span = parent.end_jd_ut - parent.start_jd_ut;
    let dasha_sign = *parent.signs.last().expect("a period has a sign");
    let mut from = parent.start_jd_ut;
    sub_period_signs(dasha_sign, forward)
        .into_iter()
        .enumerate()
        .map(|(k, sign)| {
            let to = if k == 11 {
                parent.end_jd_ut
            } else {
                parent.start_jd_ut + span * (k as f64 + 1.0) / 12.0
            };
            let mut signs = parent.signs.clone();
            signs.push(sign);
            let period = SignDashaPeriod {
                signs,
                level,
                cycle: parent.cycle,
                start_jd_ut: from,
                end_jd_ut: to,
                has_children: level < MAX_SIGN_DASHA_LEVEL,
            };
            from = to;
            period
        })
        .collect()
}

/// Saf hesap: konumlardan chara dasha. Lagna yoksa (doğum saati bilinmiyor) `None`.
/// `birth_jd_ut` ilk dashanın başlangıcıdır; `year_days` yıl uzunluğudur
/// (`CalcSettings::dasha_year`).
pub fn chara_dasha(
    positions: &Positions,
    co_lords: bool,
    birth_jd_ut: f64,
    year_days: f64,
) -> Option<CharaDasha> {
    let lagna = positions.lagna_sign?;
    let forward = chara_dasha_forward(lagna);
    let order = sign_sequence(lagna, forward);
    let years = order.map(|sign| chara_dasha_sign_years(sign, positions, co_lords));
    let mut periods = Vec::new();
    let mut elapsed: u16 = 0;
    for cycle in 1..=2u8 {
        for (i, &sign) in order.iter().enumerate() {
            let length = if cycle == 1 {
                years[i]
            } else {
                second_cycle_years(years[i])
            };
            if length == 0 {
                continue;
            }
            if cycle == 2 && elapsed >= CHARA_DASHA_HORIZON_YEARS {
                break;
            }
            let until = elapsed + u16::from(length);
            let dasha = SignDashaPeriod {
                signs: vec![sign],
                level: 1,
                cycle,
                start_jd_ut: birth_jd_ut + f64::from(elapsed) * year_days,
                end_jd_ut: birth_jd_ut + f64::from(until) * year_days,
                has_children: true,
            };
            let children = subdivide(&dasha, forward);
            periods.push(dasha);
            periods.extend(children);
            elapsed = until;
        }
    }
    Some(CharaDasha {
        forward,
        years,
        periods,
        method: CHARA_DASHA_METHOD.to_owned(),
    })
}

/// `cycle` çevriminde `path` ile seçilen dönemin alt dönemleri. `[]` o çevrimin dashalarını,
/// `[dasha]` on iki alt dönemi, `[dasha, alt dönem]` on iki 3. seviye dönemi verir. Daha uzun
/// yol, 1 ve 2 dışındaki çevrim ve o çevrimde üretilmemiş bir dasha hatadır.
pub fn sign_dasha_children(
    dasha: &CharaDasha,
    path: &[Sign],
    cycle: u8,
) -> Result<Vec<SignDashaPeriod>, CoreError> {
    if !(1..=2).contains(&cycle) {
        return Err(CoreError::InvalidDashaInput("cycle"));
    }
    if path.len() >= usize::from(MAX_SIGN_DASHA_LEVEL) {
        return Err(CoreError::InvalidDashaInput("path"));
    }
    let in_cycle = || dasha.periods.iter().filter(move |p| p.cycle == cycle);
    if path.is_empty() {
        return Ok(in_cycle().filter(|p| p.level == 1).cloned().collect());
    }
    let parent = in_cycle()
        .find(|p| p.signs == path)
        .ok_or(CoreError::InvalidDashaInput("path"))?;
    if parent.level < CHARA_DASHA_LEVELS {
        let level = parent.level + 1;
        return Ok(in_cycle()
            .filter(|p| p.level == level && p.signs.starts_with(path))
            .cloned()
            .collect());
    }
    Ok(subdivide(parent, dasha.forward))
}

/// `jd_ut` anını içeren dönemlerin yolu, üç seviye. An üretilen dönemlerin dışındaysa (doğumdan
/// önce ya da son dashadan sonra) `None`. Arayüzün "güncel dönem"i buradan gelir; arayüz tarih
/// karşılaştırması yapmaz.
pub fn sign_dasha_path_at(
    dasha: &CharaDasha,
    jd_ut: f64,
) -> Result<Option<SignDashaPath>, CoreError> {
    if !jd_ut.is_finite() {
        return Err(CoreError::InvalidDashaInput("jd_ut"));
    }
    let holds = |p: &&SignDashaPeriod| p.start_jd_ut <= jd_ut && jd_ut < p.end_jd_ut;
    let Some(second) = dasha
        .periods
        .iter()
        .filter(|p| p.level == CHARA_DASHA_LEVELS)
        .find(holds)
    else {
        return Ok(None);
    };
    // Alt dönemler ebeveyni boşluksuz böler: içeren bir 3. seviye dönem her zaman vardır.
    let third = subdivide(second, dasha.forward);
    let signs = third
        .iter()
        .find(holds)
        .map_or_else(|| second.signs.clone(), |p| p.signs.clone());
    Ok(Some(SignDashaPath {
        cycle: second.cycle,
        signs,
    }))
}

/// `compute_chara_dasha` komutunun hesabı: temel harita, konumlar, dasha. Doğum saati
/// bilinmiyorsa (`input.time_known == false`) `None`: sıra lagnaya bağlıdır. Ayar, an ve konum
/// denetimi `compute_chart`'ınkidir ve saat bilinmese de yapılır.
pub fn compute_chara_dasha<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<Option<CharaDasha>, CoreError> {
    let chart = compute_chart(eph, input.jd_ut, input.lat, input.lon, &input.settings)?;
    if !input.time_known {
        return Ok(None);
    }
    Ok(chara_dasha(
        &Positions::from(&chart),
        input.settings.advanced.jaimini_co_lords,
        input.jd_ut,
        input.settings.dasha_year,
    ))
}

/// `chara_dasha_children` komutunun hesabı. Doğum saati bilinmiyorsa boş liste.
pub fn compute_chara_dasha_children<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
    path: &[Sign],
    cycle: u8,
) -> Result<Vec<SignDashaPeriod>, CoreError> {
    match compute_chara_dasha(eph, input)? {
        Some(dasha) => sign_dasha_children(&dasha, path, cycle),
        None => Ok(Vec::new()),
    }
}

/// `chara_dasha_at` komutunun hesabı: `jd_ut` anının dönem yolu. Doğum saati bilinmiyorsa ya da
/// an üretilen dönemlerin dışındaysa `None`.
pub fn compute_chara_dasha_at<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
    jd_ut: f64,
) -> Result<Option<SignDashaPath>, CoreError> {
    match compute_chara_dasha(eph, input)? {
        Some(dasha) => sign_dasha_path_at(&dasha, jd_ut),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::{AdvancedSettings, CalcSettings, KarakaScheme};
    use Sign::*;

    const JD: f64 = 2_451_545.0;
    const YEAR: f64 = 365.25;

    /// Elle yazılmış yıl tablosu (bağımsız ikinci yazım, spec §6 "Çapraz denetim", §13): satır
    /// dasha burcu, sütun lordun bulunduğu burç (Koç…Balık). 12 × 12 = 144 hücre. Düz sayan
    /// burçlarda (Koç, Boğa, İkizler, Terazi, Akrep, Yay) lord bir sonraki burçtaysa 1, bir
    /// önceki burçtaysa 11 yıl; ters sayanlarda tersi; lord kendi burcundaysa 12.
    #[rustfmt::skip]
    const HAND_YEARS: [(Sign, [u8; 12]); 12] = [
        //             Koç Boğ İki Yen Asl Baş Ter Akr Yay Oğl Kov Bal
        (Aries,       [12,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11]),
        (Taurus,      [11, 12,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10]),
        (Gemini,      [10, 11, 12,  1,  2,  3,  4,  5,  6,  7,  8,  9]),
        (Cancer,      [ 3,  2,  1, 12, 11, 10,  9,  8,  7,  6,  5,  4]),
        (Leo,         [ 4,  3,  2,  1, 12, 11, 10,  9,  8,  7,  6,  5]),
        (Virgo,       [ 5,  4,  3,  2,  1, 12, 11, 10,  9,  8,  7,  6]),
        (Libra,       [ 6,  7,  8,  9, 10, 11, 12,  1,  2,  3,  4,  5]),
        (Scorpio,     [ 5,  6,  7,  8,  9, 10, 11, 12,  1,  2,  3,  4]),
        (Sagittarius, [ 4,  5,  6,  7,  8,  9, 10, 11, 12,  1,  2,  3]),
        (Capricorn,   [ 9,  8,  7,  6,  5,  4,  3,  2,  1, 12, 11, 10]),
        (Aquarius,    [10,  9,  8,  7,  6,  5,  4,  3,  2,  1, 12, 11]),
        (Pisces,      [11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1, 12]),
    ];

    /// Tablodan yıl: satırı ve sütunu arayarak (formül kullanmadan).
    fn hand_years(sign: Sign, lord_sign: Sign) -> u8 {
        let row = HAND_YEARS
            .iter()
            .find(|(s, _)| *s == sign)
            .expect("twelve rows");
        row.1[Sign::ALL
            .iter()
            .position(|s| *s == lord_sign)
            .expect("twelve columns")]
    }

    /// Lagna Koç 5°; Güneş Aslan, Ay İkizler, Mars Boğa, Merkür Başak, Jüpiter Balık, Venüs Koç,
    /// Satürn Oğlak, Rahu Yengeç, Ketu Oğlak. Yıllar (el hesabı, `forward-…` el örneğiyle aynı
    /// harita): 1, 11, 3, 1, 12, 12, 6, 2, 3, 12, 1, 12; toplam 76.
    fn aries_chart() -> Positions {
        Positions::synthetic(
            Some(5.0),
            [125.0, 70.0, 40.0, 160.0, 340.0, 15.0, 280.0, 100.0, 280.0],
        )
    }

    fn first_level(dasha: &CharaDasha) -> Vec<&SignDashaPeriod> {
        dasha.periods.iter().filter(|p| p.level == 1).collect()
    }

    fn last_signs(periods: &[SignDashaPeriod]) -> Vec<Sign> {
        periods.iter().map(|p| *p.signs.last().unwrap()).collect()
    }

    #[test]
    fn direction_follows_the_ninth_sign_from_the_lagna() {
        // 9. burç düz sayan gruptaysa ileri: Koç → Yay, Aslan → Koç, Başak → Boğa,
        // Terazi → İkizler, Kova → Terazi, Balık → Akrep.
        let forward: Vec<Sign> = Sign::ALL
            .into_iter()
            .filter(|&s| chara_dasha_forward(s))
            .collect();
        assert_eq!(forward, [Aries, Leo, Virgo, Libra, Aquarius, Pisces]);
        assert_eq!(
            chara_dasha_order(Aries),
            [
                Aries,
                Taurus,
                Gemini,
                Cancer,
                Leo,
                Virgo,
                Libra,
                Scorpio,
                Sagittarius,
                Capricorn,
                Aquarius,
                Pisces
            ]
        );
        // Boğa'dan 9. burç Oğlak (ters sayan): geri.
        assert_eq!(
            chara_dasha_order(Taurus),
            [
                Taurus,
                Aries,
                Pisces,
                Aquarius,
                Capricorn,
                Sagittarius,
                Scorpio,
                Libra,
                Virgo,
                Leo,
                Cancer,
                Gemini
            ]
        );
    }

    #[test]
    fn years_match_the_hand_table_in_every_cell() {
        for sign in Sign::ALL {
            for lord_sign in Sign::ALL {
                assert_eq!(
                    chara_dasha_years(sign, lord_sign),
                    hand_years(sign, lord_sign),
                    "{sign:?} with its lord in {lord_sign:?}"
                );
            }
        }
        // Tablonun kendi tutarlılığı: her satır 1..=12'nin permütasyonudur ve köşegen 12'dir.
        for (i, (sign, row)) in HAND_YEARS.iter().enumerate() {
            assert_eq!(*sign, Sign::ALL[i]);
            let mut sorted = *row;
            sorted.sort_unstable();
            assert_eq!(sorted, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], "{sign:?}");
            assert_eq!(row[i], 12, "{sign:?}");
        }
    }

    #[test]
    fn years_use_the_jaimini_lord_of_scorpio_and_aquarius() {
        let p = aries_chart();
        // Akrep: Mars Boğa'da yalnız, Ketu Oğlak'ta Satürn'le: Ketu. Akrep'ten Oğlak'a 3: 2 yıl.
        assert_eq!(chara_dasha_sign_years(Scorpio, &p, true), 2);
        // Eş lordlar devre dışı: Mars Boğa'da, Akrep'ten 7.: 6 yıl.
        assert_eq!(chara_dasha_sign_years(Scorpio, &p, false), 6);
        // Kova: Satürn Oğlak'ta Ketu'yla (1), Rahu Yengeç'te yalnız (0): Satürn; geri 2: 1 yıl.
        assert_eq!(chara_dasha_sign_years(Aquarius, &p, true), 1);
        assert_eq!(chara_dasha_sign_years(Aquarius, &p, false), 1);
        // İki eş lord da Akrep'te: 12 yıl. Yalnızca Mars Akrep'teyse lord dışarıdaki Ketu'dur.
        let both = Positions::synthetic(
            Some(5.0),
            [125.0, 70.0, 215.0, 160.0, 340.0, 15.0, 280.0, 45.0, 225.0],
        );
        assert_eq!(chara_dasha_sign_years(Scorpio, &both, true), 12);
        let mars_only = Positions::synthetic(
            Some(5.0),
            [125.0, 70.0, 215.0, 160.0, 340.0, 15.0, 280.0, 75.0, 255.0],
        );
        // Ketu Yay'da: Akrep'ten 2.: 1 yıl. Ayar devre dışıyken Mars kendi burcunda: 12 yıl.
        assert_eq!(chara_dasha_sign_years(Scorpio, &mars_only, true), 1);
        assert_eq!(chara_dasha_sign_years(Scorpio, &mars_only, false), 12);
    }

    #[test]
    fn first_cycle_starts_at_birth_with_the_lagna_sign_and_runs_without_gaps() {
        let d = chara_dasha(&aries_chart(), true, JD, YEAR).expect("lagna");
        assert_eq!(d.method, CHARA_DASHA_METHOD);
        assert!(d.forward);
        assert_eq!(d.years, [1, 11, 3, 1, 12, 12, 6, 2, 3, 12, 1, 12]);
        let dashas = first_level(&d);
        let first: Vec<&&SignDashaPeriod> = dashas.iter().filter(|p| p.cycle == 1).collect();
        assert_eq!(first.len(), 12);
        assert_eq!(first[0].signs, [Aries]);
        assert_eq!(first[0].start_jd_ut, JD);
        // Koç 1 yıl, Boğa 11 yıl: Boğa doğumdan 1 yıl sonra başlar, 12 yıl sonra biter.
        assert_eq!(first[0].end_jd_ut, JD + YEAR);
        assert_eq!(first[1].signs, [Taurus]);
        assert_eq!(first[1].end_jd_ut, JD + 12.0 * YEAR);
        // İlk çevrim 76 yıl sürer.
        assert_eq!(first[11].signs, [Pisces]);
        assert_eq!(first[11].end_jd_ut, JD + 76.0 * YEAR);
        for w in dashas.windows(2) {
            assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
        }
        assert!(d
            .periods
            .iter()
            .all(|p| p.has_children && p.level <= 2 && usize::from(p.level) == p.signs.len()));
    }

    #[test]
    fn second_cycle_skips_zero_year_signs_and_stops_at_the_horizon() {
        let d = chara_dasha(&aries_chart(), true, JD, YEAR).expect("lagna");
        let second: Vec<(Sign, f64)> = first_level(&d)
            .into_iter()
            .filter(|p| p.cycle == 2)
            .map(|p| (p.signs[0], (p.end_jd_ut - p.start_jd_ut) / YEAR))
            .collect();
        // 12 − yıl: Koç 11 (76→87), Boğa 1 (→88), İkizler 9 (→97), Yengeç 11 (→108); Aslan ve
        // Başak 0 yıl, atlanır; Terazi 6 (→114); Akrep 10 (114→124): 114 < 120, üretilir.
        // Yay 124. yılda başlardı: üretilmez.
        assert_eq!(
            second,
            [
                (Aries, 11.0),
                (Taurus, 1.0),
                (Gemini, 9.0),
                (Cancer, 11.0),
                (Libra, 6.0),
                (Scorpio, 10.0)
            ]
        );
        let last = d.periods.iter().rfind(|p| p.level == 1).expect("dashas");
        assert_eq!(last.end_jd_ut, JD + 124.0 * YEAR);
    }

    #[test]
    fn a_dasha_starting_exactly_at_the_horizon_is_not_produced() {
        // `backward-…` el örneğinin haritası: yıllar 5, 7, 3, 8, 9, 12, 1, 12, 4, 11, 12, 11
        // (toplam 95); ikinci çevrim Boğa 7 (→102), Koç 5 (→107), Balık 9 (→116), Kova 4 (→120).
        // Oğlak tam 120. yılda başlardı: üretilmez.
        let p = Positions::synthetic(
            Some(40.0),
            [155.0, 100.0, 220.0, 50.0, 250.0, 190.0, 5.0, 70.0, 250.0],
        );
        let d = chara_dasha(&p, true, JD, YEAR).expect("lagna");
        assert!(!d.forward);
        assert_eq!(d.years, [5, 7, 3, 8, 9, 12, 1, 12, 4, 11, 12, 11]);
        let second: Vec<Sign> = first_level(&d)
            .into_iter()
            .filter(|p| p.cycle == 2)
            .map(|p| p.signs[0])
            .collect();
        assert_eq!(second, [Taurus, Aries, Pisces, Aquarius]);
        let last = d.periods.iter().rfind(|p| p.level == 1).expect("dashas");
        assert_eq!(last.end_jd_ut, JD + 120.0 * YEAR);
    }

    #[test]
    fn sub_periods_follow_the_chart_direction_and_end_with_the_dasha_sign() {
        assert_eq!(
            sub_period_signs(Aries, true),
            [
                Taurus,
                Gemini,
                Cancer,
                Leo,
                Virgo,
                Libra,
                Scorpio,
                Sagittarius,
                Capricorn,
                Aquarius,
                Pisces,
                Aries
            ]
        );
        assert_eq!(
            sub_period_signs(Taurus, false),
            [
                Aries,
                Pisces,
                Aquarius,
                Capricorn,
                Sagittarius,
                Scorpio,
                Libra,
                Virgo,
                Leo,
                Cancer,
                Gemini,
                Taurus
            ]
        );
        let d = chara_dasha(&aries_chart(), true, JD, YEAR).expect("lagna");
        // Ön-sıra: dasha, ardından on iki alt dönemi.
        assert_eq!(d.periods[0].signs, [Aries]);
        assert_eq!(d.periods[1].signs, [Aries, Taurus]);
        assert_eq!(d.periods[12].signs, [Aries, Aries]);
        assert_eq!(d.periods[13].signs, [Taurus]);
        // Boğa dashası 11 yıl: her alt dönem 11/12 yıl; sonuncusu ebeveynin bitişine tam eşit.
        let taurus = &d.periods[13];
        let subs = &d.periods[14..26];
        assert_eq!(last_signs(subs), sub_period_signs(Taurus, true));
        assert_eq!(subs[0].start_jd_ut, taurus.start_jd_ut);
        assert_eq!(subs[11].end_jd_ut, taurus.end_jd_ut);
        for (k, s) in subs.iter().enumerate() {
            assert_eq!((s.level, s.cycle), (2, 1));
            let want = taurus.start_jd_ut + 11.0 * YEAR * (k as f64 + 1.0) / 12.0;
            assert!((s.end_jd_ut - want).abs() < 1e-6, "{k}");
        }
        for w in subs.windows(2) {
            assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
        }
    }

    #[test]
    fn children_are_served_per_cycle_down_to_the_third_level() {
        let d = chara_dasha(&aries_chart(), true, JD, YEAR).expect("lagna");
        let roots = sign_dasha_children(&d, &[], 1).unwrap();
        assert_eq!(last_signs(&roots), chara_dasha_order(Aries));
        let second = sign_dasha_children(&d, &[], 2).unwrap();
        assert_eq!(
            last_signs(&second),
            [Aries, Taurus, Gemini, Cancer, Libra, Scorpio]
        );
        // 2. seviye haritadaki dönemlerdir.
        let subs = sign_dasha_children(&d, &[Taurus], 1).unwrap();
        assert_eq!(subs[..], d.periods[14..26]);
        // 3. seviye istenince bölünür: aynı yön, son dönem ebeveyn burcu.
        let third = sign_dasha_children(&d, &[Taurus, Leo], 1).unwrap();
        assert_eq!(last_signs(&third), sub_period_signs(Leo, true));
        let parent = &d.periods[14 + 2];
        assert_eq!(parent.signs, [Taurus, Leo]);
        assert_eq!(third[0].start_jd_ut, parent.start_jd_ut);
        assert_eq!(third[11].end_jd_ut, parent.end_jd_ut);
        assert!(third
            .iter()
            .all(|p| p.level == 3 && !p.has_children && p.cycle == 1 && p.signs.len() == 3));
        // İkinci çevrimin dönemi kendi çevrimiyle istenir: Koç'un iki çevrimi ayrı dönemlerdir.
        let again = sign_dasha_children(&d, &[Aries], 2).unwrap();
        assert_eq!(again[0].cycle, 2);
        assert_eq!(again[0].start_jd_ut, JD + 76.0 * YEAR);
    }

    #[test]
    fn children_reject_bad_cycles_and_paths() {
        let d = chara_dasha(&aries_chart(), true, JD, YEAR).expect("lagna");
        for cycle in [0, 3] {
            assert_eq!(
                sign_dasha_children(&d, &[], cycle),
                Err(CoreError::InvalidDashaInput("cycle"))
            );
        }
        // Üç burçluk yolun alt dönemi yoktur (en çok üç seviye).
        assert_eq!(
            sign_dasha_children(&d, &[Aries, Taurus, Gemini], 1),
            Err(CoreError::InvalidDashaInput("path"))
        );
        // Aslan ikinci çevrimde atlanır, Yay üretilmez.
        for sign in [Leo, Sagittarius] {
            assert_eq!(
                sign_dasha_children(&d, &[sign], 2),
                Err(CoreError::InvalidDashaInput("path"))
            );
        }
    }

    #[test]
    fn path_at_follows_the_containing_periods() {
        let d = chara_dasha(&aries_chart(), true, JD, YEAR).expect("lagna");
        // Doğum anı: Koç dashası, ilk alt dönem Boğa, onun ilk alt dönemi İkizler.
        assert_eq!(
            sign_dasha_path_at(&d, JD).unwrap(),
            Some(SignDashaPath {
                cycle: 1,
                signs: vec![Aries, Taurus, Gemini]
            })
        );
        // 80 yıl sonra: ikinci çevrimin Koç dashası (76→87).
        let at = sign_dasha_path_at(&d, JD + 80.0 * YEAR)
            .unwrap()
            .expect("inside");
        assert_eq!((at.cycle, at.signs[0], at.signs.len()), (2, Aries, 3));
        for depth in 1..=3usize {
            let period = sign_dasha_children(&d, &at.signs[..depth - 1], at.cycle)
                .unwrap()
                .into_iter()
                .find(|p| p.signs == at.signs[..depth])
                .expect("on the path");
            let jd = JD + 80.0 * YEAR;
            assert!(
                period.start_jd_ut <= jd && jd < period.end_jd_ut,
                "level {depth}"
            );
        }
        // Dönemlerin dışı: doğumdan önce ve son dashanın bitişinde (124. yıl) yol yoktur.
        assert_eq!(sign_dasha_path_at(&d, JD - 1.0).unwrap(), None);
        assert_eq!(sign_dasha_path_at(&d, JD + 124.0 * YEAR).unwrap(), None);
        assert_eq!(
            sign_dasha_path_at(&d, f64::NAN),
            Err(CoreError::InvalidDashaInput("jd_ut"))
        );
    }

    #[test]
    fn no_lagna_means_no_dasha() {
        let p = aries_chart().without_lagna();
        assert_eq!(chara_dasha(&p, true, JD, YEAR), None);
    }

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
    fn collectors_follow_the_chart_the_settings_and_the_birth_time() {
        let mut eph = FakeEph::default();
        let settings = CalcSettings {
            dasha_year: 360.0,
            advanced: AdvancedSettings {
                karaka_scheme: KarakaScheme::Eight,
                jaimini_co_lords: false,
            },
            ..CalcSettings::default()
        };
        let chart = compute_chart(&mut eph, JD, 41.0, 29.0, &settings).unwrap();
        let want = chara_dasha(&Positions::from(&chart), false, JD, 360.0).expect("lagna");
        let known = input(settings.clone(), true);
        let got = compute_chara_dasha(&mut eph, &known)
            .unwrap()
            .expect("time known");
        assert_eq!(got, want);
        assert_eq!(got.periods[0].signs, [chart.lagna.sign]);
        assert_eq!(
            got.periods[0].end_jd_ut,
            JD + f64::from(got.years[0]) * 360.0
        );
        let first = got.periods[0].signs[0];
        assert_eq!(
            compute_chara_dasha_children(&mut eph, &known, &[first], 1).unwrap(),
            sign_dasha_children(&want, &[first], 1).unwrap()
        );
        assert_eq!(
            compute_chara_dasha_at(&mut eph, &known, JD + 400.0).unwrap(),
            sign_dasha_path_at(&want, JD + 400.0).unwrap()
        );

        // Doğum saati bilinmiyor: dasha yok, alt dönem listesi boş, yol yok (K2).
        let unknown = input(settings, false);
        assert_eq!(compute_chara_dasha(&mut eph, &unknown).unwrap(), None);
        assert_eq!(
            compute_chara_dasha_children(&mut eph, &unknown, &[first], 1).unwrap(),
            []
        );
        assert_eq!(
            compute_chara_dasha_at(&mut eph, &unknown, JD).unwrap(),
            None
        );
    }

    #[test]
    fn collectors_report_the_chart_errors_even_without_a_birth_time() {
        let mut eph = FakeEph::default();
        for time_known in [true, false] {
            let mut bad = input(CalcSettings::default(), time_known);
            bad.lat = 91.0;
            let err = Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0,
            });
            assert_eq!(compute_chara_dasha(&mut eph, &bad), err);
            assert_eq!(
                compute_chara_dasha_children(&mut eph, &bad, &[], 1),
                Err(CoreError::InvalidLocation {
                    lat: 91.0,
                    lon: 29.0
                })
            );
            assert_eq!(
                compute_chara_dasha_at(&mut eph, &bad, JD),
                Err(CoreError::InvalidLocation {
                    lat: 91.0,
                    lon: 29.0
                })
            );
        }
        let ok = input(CalcSettings::default(), true);
        assert_eq!(
            compute_chara_dasha_children(&mut eph, &ok, &[], 3),
            Err(CoreError::InvalidDashaInput("cycle"))
        );
    }
}
````

Alt modül kaydı ve dışa aktarımlar:

````diff
diff --git a/crates/jyotish-core/src/jaimini/mod.rs b/crates/jyotish-core/src/jaimini/mod.rs
index eeedb4d..b34a807 100644
--- a/crates/jyotish-core/src/jaimini/mod.rs
+++ b/crates/jyotish-core/src/jaimini/mod.rs
@@ -1,11 +1,12 @@
 //! Jaimini çekirdeği (ileri modüller spec'i §5): chara karaka, karakamsha, rashi drishti, eş
 //! lordlar, arudha padalar ve argala. Hesap fonksiyonları yalnızca `Positions` görür (K3);
 //! `compute_jaimini` temel haritayı hesaplayıp onlara veren ince toplayıcıdır. Chara dasha
-//! Alt proje 3.3'tedir ve bu modülün `jaimini_lord` ile `counts_forward` fonksiyonlarını
-//! kullanır.
+//! (Alt proje 3.3, spec §6) `chara_dasha` alt modülündedir ve bu modülün `jaimini_lord` ile
+//! `counts_forward` fonksiyonlarını kullanır; kendi komutları vardır ve rapora girmez.
 
 mod argala;
 mod arudha;
+mod chara_dasha;
 mod karaka;
 mod lords;
 mod rashi;
@@ -24,6 +25,13 @@ pub use argala::{
     THIRD_HOUSE_MALEFICS,
 };
 pub use arudha::{arudha_of, arudha_padas, ArudhaPada};
+pub use chara_dasha::{
+    chara_dasha, chara_dasha_forward, chara_dasha_order, chara_dasha_sign_years, chara_dasha_years,
+    compute_chara_dasha, compute_chara_dasha_at, compute_chara_dasha_children, second_cycle_years,
+    sign_dasha_children, sign_dasha_path_at, sign_sequence, sub_period_signs, CharaDasha,
+    SignDashaPath, SignDashaPeriod, CHARA_DASHA_HORIZON_YEARS, CHARA_DASHA_LEVELS,
+    CHARA_DASHA_METHOD, MAX_SIGN_DASHA_LEVEL,
+};
 pub use karaka::{
     chara_karakas, karaka_roles, karakamsha, navamsa_sign, CharaKaraka, KarakaRole, Karakamsha,
     EIGHT_KARAKA_ROLES, KARAKA_TIE_DEG, SEVEN_KARAKA_ROLES,
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 9512cef..00a0538 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -4,7 +4,7 @@
 //! Kapsam: ayanamsa, grahalar, lagna, bhavalar, nakshatra, dignity, yanma (M2), 16 varga,
 //! Vimshottari dasha, panchang, ashtakavarga ve transit (M3). İleri modüllerin ortak temeli
 //! (Alt proje 3.1): `positions` (ortak girdi), `lordship`, `nature` ve `drishti`. Jaimini
-//! çekirdeği (Alt proje 3.2): `jaimini`.
+//! çekirdeği (Alt proje 3.2) ve chara dasha (Alt proje 3.3): `jaimini`.
 
 pub mod ashtakavarga;
 mod chart;
@@ -48,13 +48,17 @@ pub use error::CoreError;
 pub use graha::{Graha, GrahaPosition};
 pub use houses::{house_from_cusps, whole_sign_cusps, whole_sign_house, Bhavas, Lagna};
 pub use jaimini::{
-    argala_row, argala_rows, argala_status, arudha_of, arudha_padas, chara_karakas, co_lord_choice,
-    co_lords, compute_jaimini, counts_forward, directed_count, graha_rashi_drishti, jaimini_lord,
-    jaimini_report, karaka_roles, karakamsha, navamsa_sign, rashi_drishti_matrix,
-    rashi_drishti_signs, sign_kind, stronger_co_lord, ArgalaPair, ArgalaRow, ArgalaStatus,
-    ArudhaPada, CharaKaraka, CoLordChoice, JaiminiReport, KarakaRole, Karakamsha, SignKind,
-    ARGALA_PAIRS, EIGHT_KARAKA_ROLES, JAIMINI_METHOD, KARAKA_TIE_DEG, SEVEN_KARAKA_ROLES,
-    THIRD_HOUSE_MALEFICS,
+    argala_row, argala_rows, argala_status, arudha_of, arudha_padas, chara_dasha,
+    chara_dasha_forward, chara_dasha_order, chara_dasha_sign_years, chara_dasha_years,
+    chara_karakas, co_lord_choice, co_lords, compute_chara_dasha, compute_chara_dasha_at,
+    compute_chara_dasha_children, compute_jaimini, counts_forward, directed_count,
+    graha_rashi_drishti, jaimini_lord, jaimini_report, karaka_roles, karakamsha, navamsa_sign,
+    rashi_drishti_matrix, rashi_drishti_signs, second_cycle_years, sign_dasha_children,
+    sign_dasha_path_at, sign_kind, sign_sequence, stronger_co_lord, sub_period_signs, ArgalaPair,
+    ArgalaRow, ArgalaStatus, ArudhaPada, CharaDasha, CharaKaraka, CoLordChoice, JaiminiReport,
+    KarakaRole, Karakamsha, SignDashaPath, SignDashaPeriod, SignKind, ARGALA_PAIRS,
+    CHARA_DASHA_HORIZON_YEARS, CHARA_DASHA_LEVELS, CHARA_DASHA_METHOD, EIGHT_KARAKA_ROLES,
+    JAIMINI_METHOD, KARAKA_TIE_DEG, MAX_SIGN_DASHA_LEVEL, SEVEN_KARAKA_ROLES, THIRD_HOUSE_MALEFICS,
 };
 pub use lordship::{house_lord, house_sign, houses_owned};
 pub use nature::{kuja_dosha, moon_is_waxing, nature, KujaDosha, Nature, KUJA_DOSHA_HOUSES};
````

- [ ] **Step 6: Testleri koş**

```bash
cargo test -p jyotish-core 2>&1 | grep -E "^test result|Running"
cargo test -p jyotish-core --lib chara_dasha 2>&1 | grep -E "^test result"
```
Expected: lib `191 passed` (173 + 13 birim testi + 5 proptest), golden `8 passed`, `modules_golden` `7 passed`; ikinci komut `18 passed`. Yedi el örneği motorun ilk koşusunda geçmelidir; bir örnek düşerse önce kural (spec §6 ve D2–D5) yeniden okunur, beklenen değer motor çıktısına bakarak değiştirilmez.

- [ ] **Step 7: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
grep -rn '"source"' tests/golden/modules | grep -v '"source": "hand"'
ls tests/golden/modules/chara_dasha/*.json | wc -l
```
Expected: fmt ve clippy çıktısız; `jyotish-core` lib `191 passed`, `modules_golden` `7 passed`; diğer Rust satırları tabandaki gibi; `grep` çıktısız; son komut `7` yazar.

- [ ] **Step 8: Commit**

```bash
git add crates/jyotish-core tests/golden/modules
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): chara dasha in the padakrama order: direction, years, two cycles, sub-periods; seven hand fixtures and a 144-cell hand table"
```
Expected: iki `grep` çıktısız.

---

### Task 2: Uygulama: `compute_chara_dasha`, `chara_dasha_children` ve `chara_dasha_at` komutları; bağlamalar, sarmalayıcılar ve fikstür

Spec §6 "Veri modeli ve IPC", K1, K2, K8; §13 "Komut katmanı"; §15 ve §6 bütçeleri; D7, D8, D18. Komutlar Plan 14'ün ortak kalıbını izler: motor iş parçacığı, `blocking` havuzu, `CommandError`.

**Files:**
- Modify: `app/src-tauri/src/{module_commands,lib}.rs`, `app/src-tauri/tests/{modules,ui_fixture}.rs`, `app/ui/api.ts`, `app/ui/api.test.ts`, `app/ui/test/fixture.ts`
- Üretilir: `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`
- Test: `app/src-tauri/tests/modules.rs`, `app/ui/api.test.ts`

**Interfaces:**
- Consumes: `jyotish_core::{compute_chara_dasha, compute_chara_dasha_children, compute_chara_dasha_at, chara_dasha, sign_dasha_children, sign_dasha_path_at, CharaDasha, SignDashaPeriod, SignDashaPath, Sign, ChartInput, Positions, CHARA_DASHA_METHOD}` (Task 1); `ephemeris::{Engine, SwissEph}`; `client_commands::blocking`; `commands::{CommandError, compute_chart_blocking}`; `tests/modules.rs`'teki `golden_inputs` ve `median_of_15`; `transits_at_blocking` dönüşündeki `snapshot.jd_ut`.
- Produces:
  - `module_commands::compute_chara_dasha_blocking(engine: &Engine, input: ChartInput) -> Result<Option<CharaDasha>, CommandError>`
  - `module_commands::chara_dasha_children_blocking(engine: &Engine, input: ChartInput, path: Vec<Sign>, cycle: u8) -> Result<Vec<SignDashaPeriod>, CommandError>`
  - `module_commands::chara_dasha_at_blocking(engine: &Engine, input: ChartInput, jd_ut: f64) -> Result<Option<SignDashaPath>, CommandError>`
  - Tauri komutları: `compute_chara_dasha(input)`, `chara_dasha_children(input, path, cycle)`, `chara_dasha_at(input, jd_ut)`
  - TS: `commands.computeCharaDasha(input)`, `commands.charaDashaChildren(input, path, cycle)`, `commands.charaDashaAt(input, jdUt)`; `api.computeCharaDasha(input: ChartInput)`, `api.charaDashaChildren(input: ChartInput, path: Sign[], cycle: number)`, `api.charaDashaAt(input: ChartInput, jdUt: number)`; tipler `CharaDasha`, `SignDashaPeriod`, `SignDashaPath`
  - Fikstür: `UiFixture.chara_dasha: CharaDasha`, `UiFixture.chara_dasha_children: { path: Sign[]; cycle: number; periods: SignDashaPeriod[] }` (ilk dashanın ilk alt döneminin 3. seviyesi), `UiFixture.chara_dasha_at: SignDashaPath` (fikstürdeki transit anı için).

- [ ] **Step 1: Başarısız testleri yaz**

`tests/modules.rs` tek test olarak durur; `chara_dasha_checks` eklenir. Beş golden anında: komut `Some` döner ve sonuç, v1'de golden fixture'larla doğrulanmış haritanın konumlarından `chara_dasha` ile kurulan sonuçla birebir aynıdır (çapraz denetim, §13 katman 4); ilk dasha doğum anında haritanın lagna burcuyla başlar; spec'in değişmezleri sağlanır; 2. seviye haritadaki dönemlerdir, 3. seviye ebeveyni böler; doğumdan 30 yıl sonrasının yolu o anı içerir; saat bilinmeyince üç komut da boş döner; eş lord ayarı ve yıl uzunluğu hesaba yansır; hatalar `compute_chart`'ın kodlarına ve `invalid_dasha_input`'a eşlenir; üç ortanca süre bütçenin içindedir (D18).

````diff
diff --git a/app/src-tauri/tests/modules.rs b/app/src-tauri/tests/modules.rs
index 6a295c3..8ae9caa 100644
--- a/app/src-tauri/tests/modules.rs
+++ b/app/src-tauri/tests/modules.rs
@@ -6,13 +6,17 @@ use std::time::{Duration, Instant};
 
 use ephemeris::Engine;
 use jyotish_core::{
-    co_lords, graha_aspect_signs, house_lord, house_sign, jaimini_report, rashi_drishti_signs,
-    sphuta_drishti, AdvancedSettings, CalcSettings, ChartInput, ChartWarning, Graha, KarakaRole,
-    KarakaScheme, Positions, Sign, Varga, ASPECTING_GRAHAS, DRISHTI_METHOD, JAIMINI_METHOD,
+    chara_dasha, co_lords, graha_aspect_signs, house_lord, house_sign, jaimini_report,
+    rashi_drishti_signs, sign_dasha_children, sign_dasha_path_at, sphuta_drishti, AdvancedSettings,
+    CalcSettings, ChartInput, ChartWarning, Graha, KarakaRole, KarakaScheme, Positions, Sign,
+    SignDashaPeriod, Varga, ASPECTING_GRAHAS, CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD,
     SEVEN_KARAKA_ROLES,
 };
 use jyotish_desk_lib::commands::compute_chart_blocking;
-use jyotish_desk_lib::module_commands::{compute_drishti_blocking, compute_jaimini_blocking};
+use jyotish_desk_lib::module_commands::{
+    chara_dasha_at_blocking, chara_dasha_children_blocking, compute_chara_dasha_blocking,
+    compute_drishti_blocking, compute_jaimini_blocking,
+};
 use serde_json::Value;
 
 fn root() -> PathBuf {
@@ -53,6 +57,7 @@ fn module_commands_run_on_the_engine_thread_and_map_errors() {
     assert!(inputs.len() >= 5, "{} golden moments", inputs.len());
     drishti_checks(&engine, &inputs);
     jaimini_checks(&engine, &inputs);
+    chara_dasha_checks(&engine, &inputs);
 }
 
 fn drishti_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
@@ -389,3 +394,251 @@ fn jaimini_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
         "compute_jaimini median {median:?}"
     );
 }
+
+fn chara_dasha_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
+    for (id, input) in inputs {
+        let dasha = compute_chara_dasha_blocking(engine, input.clone())
+            .unwrap_or_else(|e| panic!("{id}: {}", e.code))
+            .unwrap_or_else(|| panic!("{id}: a dasha with a known birth time"));
+        assert_eq!(dasha.method, CHARA_DASHA_METHOD, "{id}");
+        let chart = compute_chart_blocking(
+            engine,
+            input.jd_ut,
+            input.lat,
+            input.lon,
+            input.settings.clone(),
+        )
+        .expect("chart");
+        // Çapraz denetim: v1'de golden fixture'larla doğrulanmış haritanın konumlarından aynı
+        // dasha çıkar; ilk dasha doğum anında, haritanın lagna burcuyla başlar.
+        let positions = Positions::from(&chart);
+        let year = input.settings.dasha_year;
+        let want = chara_dasha(&positions, true, input.jd_ut, year).expect("lagna");
+        assert_eq!(dasha, want, "{id}");
+        let dashas: Vec<&SignDashaPeriod> = dasha.periods.iter().filter(|p| p.level == 1).collect();
+        assert_eq!(dashas[0].signs, [chart.lagna.sign], "{id}");
+        assert_eq!(dashas[0].start_jd_ut, input.jd_ut, "{id}");
+
+        // Değişmezler: ilk çevrim on iki burcun permütasyonudur; her süre 1–12 yıldır; dönemler
+        // boşluksuz ardışıktır; iki çevrimde aynı burcun yılları toplamı 12'dir.
+        let first: Vec<Sign> = dashas
+            .iter()
+            .filter(|p| p.cycle == 1)
+            .map(|p| p.signs[0])
+            .collect();
+        let mut sorted = first.clone();
+        sorted.sort_by_key(|s| s.index());
+        assert_eq!(sorted[..], Sign::ALL[..], "{id}");
+        assert!(dasha.years.iter().all(|y| (1..=12).contains(y)), "{id}");
+        for w in dashas.windows(2) {
+            assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut, "{id}");
+        }
+        for p in dashas.iter().filter(|p| p.cycle == 2) {
+            let i = first
+                .iter()
+                .position(|s| *s == p.signs[0])
+                .expect("in the first cycle");
+            let years = (p.end_jd_ut - p.start_jd_ut) / year;
+            assert!(
+                (years + f64::from(dasha.years[i]) - 12.0).abs() < 1e-6,
+                "{id}: {:?}",
+                p.signs
+            );
+        }
+
+        // Alt dönemler: 2. seviye haritadaki dönemlerdir; 3. seviye ebeveyni on iki parçaya
+        // böler ve ebeveynin burcuyla biter.
+        let lagna = chart.lagna.sign;
+        let second = chara_dasha_children_blocking(engine, input.clone(), vec![lagna], 1)
+            .expect("second level");
+        assert_eq!(second[..], dasha.periods[1..13], "{id}");
+        let parent = &second[0];
+        let third = chara_dasha_children_blocking(engine, input.clone(), parent.signs.clone(), 1)
+            .expect("third level");
+        assert_eq!(
+            third,
+            sign_dasha_children(&want, &parent.signs, 1).unwrap(),
+            "{id}"
+        );
+        assert_eq!(third.len(), 12, "{id}");
+        assert_eq!(third[0].start_jd_ut, parent.start_jd_ut, "{id}");
+        assert_eq!(third[11].end_jd_ut, parent.end_jd_ut, "{id}");
+        assert_eq!(third[11].signs[2], parent.signs[1], "{id}");
+        assert!(
+            third.iter().all(|p| p.level == 3 && !p.has_children),
+            "{id}"
+        );
+        let roots =
+            chara_dasha_children_blocking(engine, input.clone(), vec![], 2).expect("second cycle");
+        assert!(
+            !roots.is_empty() && roots.iter().all(|p| p.cycle == 2 && p.level == 1),
+            "{id}"
+        );
+
+        // Bir anın yolu: doğumdan 30 yıl sonrası ilk çevrimdedir ve dönemleri anı içerir.
+        let at = input.jd_ut + 30.0 * year;
+        let path = chara_dasha_at_blocking(engine, input.clone(), at)
+            .expect("path")
+            .expect("inside the produced span");
+        assert_eq!(
+            Some(path.clone()),
+            sign_dasha_path_at(&want, at).unwrap(),
+            "{id}"
+        );
+        assert_eq!(path.signs.len(), 3, "{id}");
+        let holder = dashas
+            .iter()
+            .find(|p| p.cycle == path.cycle && p.signs[0] == path.signs[0])
+            .expect("dasha on the path");
+        assert!(holder.start_jd_ut <= at && at < holder.end_jd_ut, "{id}");
+        assert_eq!(
+            chara_dasha_at_blocking(engine, input.clone(), input.jd_ut - 1.0).expect("before"),
+            None,
+            "{id}"
+        );
+
+        // Saat bilinmiyorsa dasha Rust'ta boş döner (K2): sıra lagnaya bağlıdır.
+        let unknown = ChartInput {
+            time_known: false,
+            ..input.clone()
+        };
+        assert_eq!(
+            compute_chara_dasha_blocking(engine, unknown.clone()).expect("time unknown"),
+            None,
+            "{id}"
+        );
+        assert_eq!(
+            chara_dasha_children_blocking(engine, unknown.clone(), vec![lagna], 1)
+                .expect("time unknown"),
+            [],
+            "{id}"
+        );
+        assert_eq!(
+            chara_dasha_at_blocking(engine, unknown, at).expect("time unknown"),
+            None,
+            "{id}"
+        );
+
+        // Ayarlar hesapta kullanılır: eş lord kuralı ve yıl uzunluğu.
+        let mut plain = input.clone();
+        plain.settings.advanced.jaimini_co_lords = false;
+        plain.settings.dasha_year = 360.0;
+        let plain_dasha = compute_chara_dasha_blocking(engine, plain)
+            .expect("plain lords")
+            .expect("dasha");
+        assert_eq!(
+            plain_dasha,
+            chara_dasha(&positions, false, input.jd_ut, 360.0).expect("lagna"),
+            "{id}"
+        );
+        assert_eq!(
+            plain_dasha.periods[0].end_jd_ut,
+            input.jd_ut + f64::from(plain_dasha.years[0]) * 360.0,
+            "{id}"
+        );
+    }
+
+    // Hatalar `compute_chart` ile aynı kodlara eşlenir; yol ve çevrim hataları Vimshottari'nin
+    // koduyla (`invalid_dasha_input`) döner. Parametreler danışan verisi taşımaz.
+    let delhi = inputs
+        .iter()
+        .find(|(id, _)| id == "new-delhi-2000-01-01")
+        .expect("delhi fixture")
+        .1
+        .clone();
+    let bad_place = ChartInput {
+        lat: 91.0,
+        ..delhi.clone()
+    };
+    assert_eq!(
+        compute_chara_dasha_blocking(engine, bad_place.clone())
+            .unwrap_err()
+            .code,
+        "invalid_location"
+    );
+    assert_eq!(
+        chara_dasha_children_blocking(engine, bad_place.clone(), vec![], 1)
+            .unwrap_err()
+            .code,
+        "invalid_location"
+    );
+    assert_eq!(
+        chara_dasha_at_blocking(engine, bad_place, delhi.jd_ut)
+            .unwrap_err()
+            .code,
+        "invalid_location"
+    );
+    let old_settings = ChartInput {
+        settings: CalcSettings {
+            schema_version: 1,
+            ..CalcSettings::default()
+        },
+        ..delhi.clone()
+    };
+    assert_eq!(
+        compute_chara_dasha_blocking(engine, old_settings)
+            .unwrap_err()
+            .code,
+        "unsupported_schema_version"
+    );
+    let bad_moment = ChartInput {
+        jd_ut: f64::NAN,
+        ..delhi.clone()
+    };
+    assert_eq!(
+        compute_chara_dasha_blocking(engine, bad_moment)
+            .unwrap_err()
+            .code,
+        "invalid_julian_day"
+    );
+    let err = chara_dasha_children_blocking(engine, delhi.clone(), vec![], 3).unwrap_err();
+    assert_eq!(
+        (err.code.as_str(), err.params["field"].as_str()),
+        ("invalid_dasha_input", "cycle")
+    );
+    let err =
+        chara_dasha_children_blocking(engine, delhi.clone(), vec![Sign::Aries; 3], 1).unwrap_err();
+    assert_eq!(
+        (err.code.as_str(), err.params["field"].as_str()),
+        ("invalid_dasha_input", "path")
+    );
+    let err = chara_dasha_at_blocking(engine, delhi.clone(), f64::NAN).unwrap_err();
+    assert_eq!(
+        (err.code.as_str(), err.params["field"].as_str()),
+        ("invalid_dasha_input", "jd_ut")
+    );
+
+    // Performans: spec §15 "compute_chara_dasha ≤ 10 ms"; §6 "alt dönemler ≤ 5 ms" (temel harita
+    // hesabı dahil).
+    let first = compute_chara_dasha_blocking(engine, delhi.clone())
+        .expect("dasha")
+        .expect("time known")
+        .periods[1]
+        .signs
+        .clone();
+    let (median, max) = median_of_15(|| {
+        compute_chara_dasha_blocking(engine, delhi.clone()).expect("chara dasha");
+    });
+    eprintln!("compute_chara_dasha median {median:?}, max {max:?}");
+    assert!(
+        median <= Duration::from_millis(10),
+        "compute_chara_dasha median {median:?}"
+    );
+    let (median, max) = median_of_15(|| {
+        chara_dasha_children_blocking(engine, delhi.clone(), first.clone(), 1).expect("children");
+    });
+    eprintln!("chara_dasha_children median {median:?}, max {max:?}");
+    assert!(
+        median <= Duration::from_millis(5),
+        "chara_dasha_children median {median:?}"
+    );
+    let at = delhi.jd_ut + 9_000.0;
+    let (median, max) = median_of_15(|| {
+        chara_dasha_at_blocking(engine, delhi.clone(), at).expect("path");
+    });
+    eprintln!("chara_dasha_at median {median:?}, max {max:?}");
+    assert!(
+        median <= Duration::from_millis(10),
+        "chara_dasha_at median {median:?}"
+    );
+}
````

Fikstür üç yeni anahtar kazanır:

````diff
diff --git a/app/src-tauri/tests/ui_fixture.rs b/app/src-tauri/tests/ui_fixture.rs
index bcc0f59..a5ee7d2 100644
--- a/app/src-tauri/tests/ui_fixture.rs
+++ b/app/src-tauri/tests/ui_fixture.rs
@@ -15,7 +15,10 @@ use jyotish_core::ChartInput;
 use jyotish_desk_lib::atlas_commands::{preview_moment_body, search_places_blocking, AtlasState};
 use jyotish_desk_lib::client_commands::create_client_blocking;
 use jyotish_desk_lib::commands::dasha_children;
-use jyotish_desk_lib::module_commands::{compute_drishti_blocking, compute_jaimini_blocking};
+use jyotish_desk_lib::module_commands::{
+    chara_dasha_at_blocking, chara_dasha_children_blocking, compute_chara_dasha_blocking,
+    compute_drishti_blocking, compute_jaimini_blocking,
+};
 use jyotish_desk_lib::transit_commands::{transits_at_blocking, TransitAt};
 use serde_json::{json, Value};
 use store::{BirthData, ChartKind, NewChart, NewClient, Place, Store, TimeCertainty, TzChoice};
@@ -104,7 +107,19 @@ fn ui_fixture_is_up_to_date() {
         time_known: true,
     };
     let drishti = compute_drishti_blocking(&engine, module_input.clone()).expect("drishti");
-    let jaimini = compute_jaimini_blocking(&engine, module_input).expect("jaimini");
+    let jaimini = compute_jaimini_blocking(&engine, module_input.clone()).expect("jaimini");
+    // Chara dasha: iki seviye, ilk alt dönemin 3. seviyesi (istenerek yüklenen seviye) ve
+    // transit anının dönem yolu.
+    let chara_dasha = compute_chara_dasha_blocking(&engine, module_input.clone())
+        .expect("chara dasha")
+        .expect("time known");
+    let chara_path = chara_dasha.periods[1].signs.clone();
+    let chara_children =
+        chara_dasha_children_blocking(&engine, module_input.clone(), chara_path.clone(), 1)
+            .expect("chara dasha children");
+    let chara_at = chara_dasha_at_blocking(&engine, module_input, transit.snapshot.jd_ut)
+        .expect("chara dasha at")
+        .expect("inside the produced span");
     // Kaydedilen doğum verisi girdinin kanonik biçimidir (Plan 10 kapsam dışı 21).
     assert_eq!(opened.chart_record.birth, delhi());
     let prefs = store
@@ -122,6 +137,9 @@ fn ui_fixture_is_up_to_date() {
         "prefs": prefs,
         "drishti": drishti,
         "jaimini": jaimini,
+        "chara_dasha": chara_dasha,
+        "chara_dasha_children": { "path": chara_path, "cycle": 1, "periods": chara_children },
+        "chara_dasha_at": chara_at,
     });
     let text = serde_json::to_string(&value)
         .expect("serialize")
````

`api.test.ts`'in iki tablosu (taklit komutlar ve sarmalayıcı çağrıları) üç yeni adı taşır:

````diff
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index 26202d6..a7aaef8 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -30,6 +30,9 @@ vi.mock("./bindings", () => ({
     dashaChildren: vi.fn(),
     computeDrishti: vi.fn(),
     computeJaimini: vi.fn(),
+    computeCharaDasha: vi.fn(),
+    charaDashaChildren: vi.fn(),
+    charaDashaAt: vi.fn(),
     searchPlaces: vi.fn(),
     previewMoment: vi.fn(),
     getAppPrefs: vi.fn(),
@@ -111,6 +114,9 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["dashaChildren", [{ birth_jd_ut: 1 }, ["Sun"]]],
     ["computeDrishti", [{ jd_ut: 1, time_known: true }]],
     ["computeJaimini", [{ jd_ut: 1, time_known: false }]],
+    ["computeCharaDasha", [{ jd_ut: 1, time_known: true }]],
+    ["charaDashaChildren", [{ jd_ut: 1, time_known: true }, ["Aries", "Taurus"], 2]],
+    ["charaDashaAt", [{ jd_ut: 1, time_known: true }, 2451545.5]],
     ["searchPlaces", ["delhi", 8]],
     ["previewMoment", [{ date: "2000-01-01" }]],
     ["getAppPrefs", []],
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-desk --test modules 2>&1 | grep -E "^error" | sort | uniq -c
pnpm --dir app test api.test 2>&1 | grep -E "Tests |TypeError"
```
Expected: ilk komutta `error[E0432]: unresolved imports` (`chara_dasha_at_blocking`, `chara_dasha_children_blocking`, `compute_chara_dasha_blocking`) ve `could not compile`; ikincide `TypeError: call is not a function` ve `1 failed | 6 passed (7)`.

- [ ] **Step 3: Komutları yaz ve kaydet**

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index a104093..2d9f9f3 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -31,6 +31,9 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             commands::print_window,
             module_commands::compute_drishti,
             module_commands::compute_jaimini,
+            module_commands::compute_chara_dasha,
+            module_commands::chara_dasha_children,
+            module_commands::chara_dasha_at,
             client_commands::open_client,
             client_commands::restore_session,
             client_commands::create_client,
diff --git a/app/src-tauri/src/module_commands.rs b/app/src-tauri/src/module_commands.rs
index 3c3297f..08c426e 100644
--- a/app/src-tauri/src/module_commands.rs
+++ b/app/src-tauri/src/module_commands.rs
@@ -6,7 +6,9 @@
 use std::sync::Arc;
 
 use ephemeris::{Engine, SwissEph};
-use jyotish_core::{ChartInput, DrishtiTable, JaiminiReport};
+use jyotish_core::{
+    CharaDasha, ChartInput, DrishtiTable, JaiminiReport, Sign, SignDashaPath, SignDashaPeriod,
+};
 use tauri::State;
 
 use crate::client_commands::blocking;
@@ -56,3 +58,79 @@ pub async fn compute_jaimini(
     let engine = Arc::clone(&engine);
     blocking(move || compute_jaimini_blocking(&engine, input)).await
 }
+
+/// `compute_chara_dasha` komutunun gövdesi; bloklar.
+pub fn compute_chara_dasha_blocking(
+    engine: &Engine,
+    input: ChartInput,
+) -> Result<Option<CharaDasha>, CommandError> {
+    let dasha =
+        engine.call(move |se: &mut SwissEph| jyotish_core::compute_chara_dasha(se, &input))??;
+    Ok(dasha)
+}
+
+/// Chara dasha (padakrama düzeni): yön, on iki burcun yılları ve iki seviye dönem. Eş lord
+/// kuralı `input.settings.advanced`'ten, yıl uzunluğu `input.settings.dasha_year`'dan okunur.
+/// Doğum saati bilinmiyorsa (`time_known == false`) `null` döner: sıra lagnaya bağlıdır.
+#[tauri::command]
+#[specta::specta]
+pub async fn compute_chara_dasha(
+    engine: State<'_, Arc<Engine>>,
+    input: ChartInput,
+) -> Result<Option<CharaDasha>, CommandError> {
+    let engine = Arc::clone(&engine);
+    blocking(move || compute_chara_dasha_blocking(&engine, input)).await
+}
+
+/// `chara_dasha_children` komutunun gövdesi; bloklar.
+pub fn chara_dasha_children_blocking(
+    engine: &Engine,
+    input: ChartInput,
+    path: Vec<Sign>,
+    cycle: u8,
+) -> Result<Vec<SignDashaPeriod>, CommandError> {
+    let periods = engine.call(move |se: &mut SwissEph| {
+        jyotish_core::compute_chara_dasha_children(se, &input, &path, cycle)
+    })??;
+    Ok(periods)
+}
+
+/// Bir chara dasha döneminin alt dönemleri. `path` dönemin `signs`'ı, `cycle` çevrimidir (1 ya
+/// da 2); boş yol o çevrimin dashalarını verir. Doğum saati bilinmiyorsa liste boştur.
+#[tauri::command]
+#[specta::specta]
+pub async fn chara_dasha_children(
+    engine: State<'_, Arc<Engine>>,
+    input: ChartInput,
+    path: Vec<Sign>,
+    cycle: u8,
+) -> Result<Vec<SignDashaPeriod>, CommandError> {
+    let engine = Arc::clone(&engine);
+    blocking(move || chara_dasha_children_blocking(&engine, input, path, cycle)).await
+}
+
+/// `chara_dasha_at` komutunun gövdesi; bloklar.
+pub fn chara_dasha_at_blocking(
+    engine: &Engine,
+    input: ChartInput,
+    jd_ut: f64,
+) -> Result<Option<SignDashaPath>, CommandError> {
+    let path = engine
+        .call(move |se: &mut SwissEph| jyotish_core::compute_chara_dasha_at(se, &input, jd_ut))??;
+    Ok(path)
+}
+
+/// `jd_ut` anını içeren chara dasha dönemlerinin yolu (üç seviye). Dasa şeridinin "güncel
+/// dönem"i buradan gelir; arayüz tarih karşılaştırmaz. `jd_ut` transit anıdır
+/// (`TransitView.snapshot.jd_ut`). Doğum saati bilinmiyorsa ya da an dönemlerin dışındaysa
+/// `null` döner.
+#[tauri::command]
+#[specta::specta]
+pub async fn chara_dasha_at(
+    engine: State<'_, Arc<Engine>>,
+    input: ChartInput,
+    jd_ut: f64,
+) -> Result<Option<SignDashaPath>, CommandError> {
+    let engine = Arc::clone(&engine);
+    blocking(move || chara_dasha_at_blocking(&engine, input, jd_ut)).await
+}
````

- [ ] **Step 4: Sarmalayıcılar ve fikstür tipi; üretilen dosyaları yeniden üret**

````diff
diff --git a/app/ui/api.ts b/app/ui/api.ts
index 65936b9..ff9ea85 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -19,6 +19,7 @@ import {
   type NewClient,
   type BirthData,
   type PanelLayout,
+  type Sign,
   type TransitAt,
   type ViewState,
 } from "./bindings";
@@ -80,6 +81,11 @@ export const api = {
   // `modules/input.ts`'teki `chartInput(opened)` ile kurulur.
   computeDrishti: (input: ChartInput) => wrap(() => commands.computeDrishti(input)),
   computeJaimini: (input: ChartInput) => wrap(() => commands.computeJaimini(input)),
+  // Chara dasha: `null` doğum saati bilinmeyen haritadır. `path` dönemin `signs`'ı, `cycle`
+  // çevrimidir; `charaDashaAt` transit anının dönem yolunu verir (arayüz tarih karşılaştırmaz).
+  computeCharaDasha: (input: ChartInput) => wrap(() => commands.computeCharaDasha(input)),
+  charaDashaChildren: (input: ChartInput, path: Sign[], cycle: number) => wrap(() => commands.charaDashaChildren(input, path, cycle)),
+  charaDashaAt: (input: ChartInput, jdUt: number) => wrap(() => commands.charaDashaAt(input, jdUt)),
   searchPlaces: (query: string, limit: number) => wrap(() => commands.searchPlaces(query, limit)),
   previewMoment: (birth: BirthData) => wrap(() => commands.previewMoment(birth)),
   appInfo: () => wrap(() => commands.appInfo()),
diff --git a/app/ui/test/fixture.ts b/app/ui/test/fixture.ts
index 86032e5..1169a4e 100644
--- a/app/ui/test/fixture.ts
+++ b/app/ui/test/fixture.ts
@@ -1,7 +1,23 @@
 // Gerçek veri fikstürü (app/src-tauri/tests/ui_fixture.rs üretir): Delhi 2000-01-01 17:30
 // için `create_client` dönüşü, ilk 3. seviye dönemin alt dönemleri, "new delhi" atlas araması,
-// an önizlemesi, bir DST boşluğu hatası, aynı sabit saatte transit, bakış tablosu ve Jaimini raporu. Arayüz hesap yapmadığı için testler bunu çizer.
-import type { CommandError, DashaPeriod, DrishtiTable, Graha, JaiminiReport, MomentPreview, OpenedClient, PlaceHit, PrefsLoad, TransitView } from "../bindings";
+// an önizlemesi, bir DST boşluğu hatası, aynı sabit saatte transit, bakış tablosu, Jaimini raporu
+// ve chara dasha. Arayüz hesap yapmadığı için testler bunu çizer.
+import type {
+  CharaDasha,
+  CommandError,
+  DashaPeriod,
+  DrishtiTable,
+  Graha,
+  JaiminiReport,
+  MomentPreview,
+  OpenedClient,
+  PlaceHit,
+  PrefsLoad,
+  Sign,
+  SignDashaPath,
+  SignDashaPeriod,
+  TransitView,
+} from "../bindings";
 import raw from "./fixtures/delhi.json?raw";
 
 export interface UiFixture {
@@ -18,6 +34,12 @@ export interface UiFixture {
   drishti: DrishtiTable;
   /** `compute_jaimini`, aynı girdiyle (varsayılan ayarlar: sekizli şema, eş lordlar açık). */
   jaimini: JaiminiReport;
+  /** `compute_chara_dasha`, aynı girdiyle: iki seviye dönem, iki çevrim. */
+  chara_dasha: CharaDasha;
+  /** `chara_dasha_children`: ilk dashanın ilk alt döneminin 3. seviyesi. */
+  chara_dasha_children: { path: Sign[]; cycle: number; periods: SignDashaPeriod[] };
+  /** `chara_dasha_at`, fikstürdeki transit anı için (`transit.snapshot.jd_ut`). */
+  chara_dasha_at: SignDashaPath;
 }
 
 const FIXTURE = JSON.parse(raw) as UiFixture;
````

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
git diff --stat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
grep -cE "computeCharaDasha|charaDashaChildren|charaDashaAt" app/ui/bindings.ts
```
Expected: iki `ok`; `bindings.ts` 52 satır kazanır (üç komut ve üç tip); `delhi.json` tek satır değişir; son komut `3` yazar. Fikstürdeki dasha (Delhi, İkizler lagna, varsayılan ayarlar) sonraki görevlerin panel testlerinin beklediği değerleri taşır: yön geri (İkizler'den 9. burç Kova); sıra İkizler, Boğa, Koç, Balık, Kova, Oğlak, Yay, Akrep, Terazi, Başak, Aslan, Yengeç; yıllar 6, 6, 10, 11, 10, 9, 4, 3, 1, 9, 8, 9 (toplam 86); ikinci çevrimde sekiz dasha (İkizler … Akrep; Akrep 114. yılda başlar); transit anının (2026-11-25) yolu ilk çevrim, Balık, Başak, Koç.

- [ ] **Step 5: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test -p jyotish-desk --test modules -- --nocapture 2>&1 | grep median
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: fmt ve clippy çıktısız; `jyotish-desk` `modules` `1 passed`, `bindings` `1 passed`, `ui_fixture` `2 passed`; diğer Rust satırları Task 1'deki gibi; beş `median` satırı (`compute_drishti`, `compute_jaimini`, `compute_chara_dasha`, `chara_dasha_children`, `chara_dasha_at`), hepsi bütçesinin altında (ölçüm: 0,5–0,8 ms); Vitest `68 passed (68)`, `624 passed (624)` (`api.test.ts`'in tablosu büyür, test sayısı değişmez; "dizgi bağımsız değişken" testi değişmeden geçer: C2′); `✓ built`.

- [ ] **Step 6: Commit**

```bash
git add app/src-tauri/src app/src-tauri/tests app/ui/bindings.ts app/ui/test app/ui/api.ts app/ui/api.test.ts
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
git commit -m "feat(app): compute_chara_dasha, chara_dasha_children and chara_dasha_at commands; bindings, wrappers and fixture"
```
Expected: iki `grep` çıktısız; `grep -ciE` `0` yazar.

---

### Task 3: Görünüm durumu: açık dal sistem başına ayrı (`chara_expanded_path`)

Spec §6 "Arayüz" son madde; D10. `ViewState` yeni alan kazanır; şema sürümü ve SQLite şeması değişmez. Arayüzün görünüm modeli iki yolu ayrı tutar ve tek `save_view_state` kaydıyla yazar.

**Files:**
- Modify: `crates/store/src/{view_state,charts,exchange,lib}.rs`, `app/src-tauri/src/client_commands.rs` (yalnızca test), `app/src-tauri/tests/open_client.rs`, `app/ui/view/{model,reducer}.ts`, `app/ui/view/store.tsx`, `app/ui/session/store.tsx`, `app/ui/App.tsx`, `app/ui/test/render.tsx`
- Üretilir: `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`
- Test: `view_state.rs`, `charts.rs` ve `exchange.rs` içindeki `mod tests`; `app/ui/view/{model,store,persistence}.test.ts(x)`, `app/ui/session/store.test.tsx`

**Interfaces:**
- Consumes: `jyotish_core::{Sign, SignDashaPath, MAX_SIGN_DASHA_LEVEL}` (Task 1); `store::ViewState`, `chart_view_json`, `chart_view_path`, `parse_dasha_path`, `check_view_state`; arayüzde `SIGNS` (`chart/adapter.ts`), `viewReducer`, `ViewStateProvider`.
- Produces:
  - Rust: `ViewState.chara_expanded_path: SignDashaPath`; `pub fn empty_chara_path() -> SignDashaPath` (`{ cycle: 1, signs: [] }`); `pub fn parse_chara_path(v: Option<&Value>) -> SignDashaPath`; `pub fn chart_view_json(path: &[Graha], chara: &SignDashaPath) -> String` (imza değişir); `pub fn empty_chart_view_json() -> String`; `pub fn chart_view_chara_path(raw: &str) -> SignDashaPath`
  - TS (`view/model.ts`): `ViewModel.charaPath: SignDashaPath`; `MAX_CHARA_PATH = 3`; `emptyCharaPath()`; `parseCharaPath(raw: unknown): SignDashaPath`; `sameCharaPath(a, b): boolean`; `interface ChartView { id: number; dashaPath: unknown; charaPath?: unknown }`; `chartViewOf(opened): ChartView`; `initialViewModel(rawLayout, chart: ChartView | null)`
  - TS (`view/store.tsx`): `ViewStore.setCharaPath(path: SignDashaPath): void`; `openChart(chartId: number, dashaPath: unknown, charaPath?: unknown): void`; reducer eylemi `{ type: "setCharaPath"; path: SignDashaPath }`.

- [ ] **Step 1: Başarısız testleri yaz (store)**

`view_state.rs`: örnek kayıt chara yolunu taşır; haritaya düşen JSON iki yolu da içerir; bu alandan önce yazılmış kayıt Vimshottari yolu olarak okunur ve chara yolu boştur; geçersiz chara yolu bütünüyle düşer; proptest üreteçleri yeni alanı kapsar. `charts.rs`: iki yol haritaya yazılır ve okunur; geçersiz çevrim ve dört burçluk yol yazımda reddedilir. `exchange.rs`: dışa ve içe aktarma chara yolunu taşır.

````diff
diff --git a/crates/store/src/charts.rs b/crates/store/src/charts.rs
index 8bd2fe1..32b9309 100644
--- a/crates/store/src/charts.rs
+++ b/crates/store/src/charts.rs
@@ -629,8 +629,10 @@ mod tests {
     use crate::birth::fixtures::{delhi, istanbul};
     use crate::db::testutil::store;
     use crate::settings::AppPrefs;
-    use crate::view_state::{Layout, Panel, MAX_LAYOUTS, MAX_PANELS, MAX_VIEW_STATE_BYTES};
-    use jyotish_core::{AyanamsaSystem, Graha};
+    use crate::view_state::{
+        empty_chara_path, Layout, Panel, MAX_LAYOUTS, MAX_PANELS, MAX_VIEW_STATE_BYTES,
+    };
+    use jyotish_core::{AyanamsaSystem, Graha, Sign, SignDashaPath};
     use serde_json::Map;
 
     #[test]
@@ -787,14 +789,19 @@ mod tests {
         let vs = ViewState {
             panel_layout: Some(layout.clone()),
             dasha_expanded_path: vec![Graha::Rahu, Graha::Saturn],
+            chara_expanded_path: SignDashaPath {
+                cycle: 2,
+                signs: vec![Sign::Gemini, Sign::Taurus],
+            },
             ..ViewState::default()
         };
         s.save_view_state(a.id, &vs).unwrap();
         assert_eq!(s.view_state(a.id).unwrap(), vs);
-        // Düzen kullanıcınındır: diğer harita aynı düzeni, kendi (boş) dasha yolunu görür.
+        // Düzen kullanıcınındır: diğer harita aynı düzeni, kendi (boş) dasha yollarını görür.
         let other = s.view_state(b.id).unwrap();
         assert_eq!(other.panel_layout, Some(layout));
         assert!(other.dasha_expanded_path.is_empty());
+        assert_eq!(other.chara_expanded_path, empty_chara_path());
         // Görünüm kaydı içerik değildir.
         assert_eq!(s.chart(a.id).unwrap().updated_at, a.updated_at);
         s.save_view_state(a.id, &ViewState::default()).unwrap();
@@ -929,6 +936,20 @@ mod tests {
                 dasha_expanded_path: vec![jyotish_core::Graha::Sun; 6],
                 ..ViewState::default()
             },
+            ViewState {
+                chara_expanded_path: SignDashaPath {
+                    cycle: 3,
+                    signs: Vec::new(),
+                },
+                ..ViewState::default()
+            },
+            ViewState {
+                chara_expanded_path: SignDashaPath {
+                    cycle: 1,
+                    signs: vec![Sign::Aries; 4],
+                },
+                ..ViewState::default()
+            },
         ] {
             assert!(
                 matches!(
diff --git a/crates/store/src/exchange.rs b/crates/store/src/exchange.rs
index 9c56011..b66142f 100644
--- a/crates/store/src/exchange.rs
+++ b/crates/store/src/exchange.rs
@@ -428,7 +428,7 @@ mod tests {
     use crate::db::testutil::store;
     use crate::open::StoreWarning;
     use crate::view_state::ViewState;
-    use jyotish_core::{CalcSettings, Graha, NodeKind};
+    use jyotish_core::{CalcSettings, Graha, NodeKind, Sign, SignDashaPath};
     use serde_json::json;
 
     fn populated() -> (Store, u32) {
@@ -444,6 +444,10 @@ mod tests {
             ch.id,
             &ViewState {
                 dasha_expanded_path: vec![Graha::Venus],
+                chara_expanded_path: SignDashaPath {
+                    cycle: 2,
+                    signs: vec![Sign::Leo],
+                },
                 ..ViewState::default()
             },
         )
@@ -508,9 +512,14 @@ mod tests {
             src.calc_settings(ca.id).unwrap(),
             dst.calc_settings(cb.id).unwrap()
         );
+        let view = dst.view_state(cb.id).unwrap();
+        assert_eq!(view.dasha_expanded_path, [Graha::Venus]);
         assert_eq!(
-            dst.view_state(cb.id).unwrap().dasha_expanded_path,
-            [Graha::Venus]
+            view.chara_expanded_path,
+            SignDashaPath {
+                cycle: 2,
+                signs: vec![Sign::Leo]
+            }
         );
         assert_eq!(b.last_chart_id, Some(cb.id));
         let na = &src.notes(a.id, 10).unwrap()[0];
@@ -598,11 +607,12 @@ mod tests {
             dst.calc_settings(ch.id).unwrap(),
             SettingsLoad::Current(CalcSettings::default())
         );
-        assert!(dst
-            .view_state(ch.id)
-            .unwrap()
-            .dasha_expanded_path
-            .is_empty());
+        let view = dst.view_state(ch.id).unwrap();
+        assert!(view.dasha_expanded_path.is_empty());
+        assert_eq!(
+            view.chara_expanded_path,
+            crate::view_state::empty_chara_path()
+        );
     }
 
     #[test]
diff --git a/crates/store/src/view_state.rs b/crates/store/src/view_state.rs
index 84864e5..fec41fc 100644
--- a/crates/store/src/view_state.rs
+++ b/crates/store/src/view_state.rs
@@ -259,6 +259,10 @@ mod tests {
                 }],
             }),
             dasha_expanded_path: vec![Graha::Rahu, Graha::Jupiter],
+            chara_expanded_path: SignDashaPath {
+                cycle: 2,
+                signs: vec![Sign::Aries, Sign::Taurus],
+            },
         }
     }
 
@@ -302,6 +306,7 @@ mod tests {
         let vs = parse_view_state(&raw);
         assert_eq!(vs.schema_version, 1);
         assert!(vs.dasha_expanded_path.is_empty());
+        assert_eq!(vs.chara_expanded_path, empty_chara_path());
         let pl = vs.panel_layout.unwrap();
         assert_eq!(pl.active_layout_id, "");
         assert_eq!(pl.layouts.len(), 1);
@@ -360,14 +365,63 @@ mod tests {
     }
 
     #[test]
-    fn chart_view_keeps_only_the_dasha_path() {
-        let raw = chart_view_json(&[Graha::Venus]);
+    fn chart_view_keeps_only_the_two_dasha_paths() {
+        let chara = SignDashaPath {
+            cycle: 2,
+            signs: vec![Sign::Leo],
+        };
+        let raw = chart_view_json(&[Graha::Venus], &chara);
         assert_eq!(
             raw,
-            r#"{"dasha_expanded_path":["Venus"],"schema_version":1}"#
+            r#"{"chara_expanded_path":{"cycle":2,"signs":["Leo"]},"dasha_expanded_path":["Venus"],"schema_version":1}"#
         );
         assert_eq!(chart_view_path(&raw), [Graha::Venus]);
+        assert_eq!(chart_view_chara_path(&raw), chara);
         assert!(chart_view_path("{broken").is_empty());
+        assert_eq!(chart_view_chara_path("{broken"), empty_chara_path());
+        assert_eq!(
+            empty_chart_view_json(),
+            r#"{"chara_expanded_path":{"cycle":1,"signs":[]},"dasha_expanded_path":[],"schema_version":1}"#
+        );
+    }
+
+    #[test]
+    fn a_record_written_before_the_chara_path_reads_as_the_vimshottari_path() {
+        // Plan 16 öncesinin kaydı: yalnızca Vimshottari yolu vardır.
+        let old = r#"{"dasha_expanded_path":["Rahu","Saturn"],"schema_version":1}"#;
+        assert_eq!(chart_view_path(old), [Graha::Rahu, Graha::Saturn]);
+        assert_eq!(chart_view_chara_path(old), empty_chara_path());
+        let vs = parse_view_state(old);
+        assert_eq!(vs.dasha_expanded_path, [Graha::Rahu, Graha::Saturn]);
+        assert_eq!(vs.chara_expanded_path, empty_chara_path());
+    }
+
+    #[test]
+    fn invalid_chara_paths_are_dropped_whole() {
+        let ok = json!({"cycle": 2, "signs": ["Aries", "Taurus", "Gemini"]});
+        assert_eq!(
+            parse_chara_path(Some(&ok)),
+            SignDashaPath {
+                cycle: 2,
+                signs: vec![Sign::Aries, Sign::Taurus, Sign::Gemini]
+            }
+        );
+        for bad in [
+            json!(null),
+            json!(["Aries"]),
+            json!({"signs": ["Aries"]}),
+            json!({"cycle": 1}),
+            json!({"cycle": 0, "signs": ["Aries"]}),
+            json!({"cycle": 3, "signs": ["Aries"]}),
+            json!({"cycle": 1.5, "signs": ["Aries"]}),
+            json!({"cycle": "1", "signs": ["Aries"]}),
+            json!({"cycle": 1, "signs": "Aries"}),
+            json!({"cycle": 1, "signs": ["Aries", "Ophiuchus"]}),
+            json!({"cycle": 1, "signs": ["Aries", "Taurus", "Gemini", "Cancer"]}),
+        ] {
+            assert_eq!(parse_chara_path(Some(&bad)), empty_chara_path(), "{bad}");
+        }
+        assert_eq!(parse_chara_path(None), empty_chara_path());
     }
 
     fn arb_json() -> impl Strategy<Value = Value> {
@@ -389,6 +443,9 @@ mod tests {
                         Just("layouts".to_owned()),
                         Just("panels".to_owned()),
                         Just("dasha_expanded_path".to_owned()),
+                        Just("chara_expanded_path".to_owned()),
+                        Just("cycle".to_owned()),
+                        Just("signs".to_owned()),
                         Just("id".to_owned()),
                         Just("name".to_owned()),
                         Just("kind".to_owned()),
@@ -457,8 +514,10 @@ mod tests {
         (
             prop::option::of(("[a-z:0-9]{1,8}", prop::collection::vec(layout, 0..4))),
             prop::collection::vec(arb_graha(), 0..=MAX_PATH),
+            1u8..=2,
+            prop::collection::vec(prop::sample::select(Sign::ALL.to_vec()), 0..=MAX_SIGN_PATH),
         )
-            .prop_map(|(pl, path)| ViewState {
+            .prop_map(|(pl, path, cycle, signs)| ViewState {
                 schema_version: VIEW_STATE_SCHEMA_VERSION,
                 panel_layout: pl.map(|(active, ls)| {
                     let mut uniq: Vec<Layout> = Vec::new();
@@ -473,6 +532,7 @@ mod tests {
                     }
                 }),
                 dasha_expanded_path: path,
+                chara_expanded_path: SignDashaPath { cycle, signs },
             })
     }
 
@@ -487,6 +547,7 @@ mod tests {
         fn any_json_parses_within_bounds(v in arb_json()) {
             let vs = parse_view_state(&v.to_string());
             prop_assert!(vs.dasha_expanded_path.len() <= MAX_PATH);
+            prop_assert!(chara_path_fits(&vs.chara_expanded_path));
             if let Some(pl) = &vs.panel_layout {
                 for l in &pl.layouts {
                     prop_assert!(!l.id.trim().is_empty());
````

Uygulama katmanı: `save_view_state` gövdesi chara yolunu yazar ve geçersiz çevrimi reddeder; oturum geri yüklemesi yolu geri getirir.

````diff
diff --git a/app/src-tauri/src/client_commands.rs b/app/src-tauri/src/client_commands.rs
index b0cb96e..4fad432 100644
--- a/app/src-tauri/src/client_commands.rs
+++ b/app/src-tauri/src/client_commands.rs
@@ -508,6 +508,10 @@ mod tests {
         let (_, chart) = lock(&store).unwrap().create_client(&new_client).unwrap();
         let vs = ViewState {
             dasha_expanded_path: vec![jyotish_core::Graha::Rahu],
+            chara_expanded_path: jyotish_core::SignDashaPath {
+                cycle: 2,
+                signs: vec![jyotish_core::Sign::Gemini],
+            },
             ..ViewState::default()
         };
         save_view_state_body(&store, chart.id, &vs).unwrap();
@@ -527,5 +531,19 @@ mod tests {
             (e.code.as_str(), e.params["field"].as_str()),
             ("invalid_input", "view_state")
         );
+        // Chara dasha yolunun çevrimi 1 ya da 2'dir.
+        let bad = ViewState {
+            chara_expanded_path: jyotish_core::SignDashaPath {
+                cycle: 0,
+                signs: Vec::new(),
+            },
+            ..ViewState::default()
+        };
+        let e = save_view_state_body(&store, chart.id, &bad).unwrap_err();
+        assert_eq!(
+            (e.code.as_str(), e.params["field"].as_str()),
+            ("invalid_input", "view_state")
+        );
+        assert_eq!(lock(&store).unwrap().view_state(chart.id).unwrap(), vs);
     }
 }
diff --git a/app/src-tauri/tests/open_client.rs b/app/src-tauri/tests/open_client.rs
index ecd3952..9a073c1 100644
--- a/app/src-tauri/tests/open_client.rs
+++ b/app/src-tauri/tests/open_client.rs
@@ -7,7 +7,7 @@ use std::sync::Mutex;
 use std::time::{Duration, Instant};
 
 use ephemeris::Engine;
-use jyotish_core::{AyanamsaSystem, CalcSettings, Graha};
+use jyotish_core::{AyanamsaSystem, CalcSettings, Graha, Sign, SignDashaPath};
 use jyotish_desk_lib::client_commands::{
     add_chart_blocking, create_client_blocking, keep_stored_moment_blocking, open_client_blocking,
     restore_session_blocking, save_calc_settings_blocking, update_chart_blocking, DB_FILE_NAME,
@@ -106,6 +106,10 @@ fn open_client_round_trips_and_is_fast() {
     let vs = ViewState {
         panel_layout: Some(layout()),
         dasha_expanded_path: vec![Graha::Rahu, Graha::Jupiter],
+        chara_expanded_path: SignDashaPath {
+            cycle: 1,
+            signs: vec![Sign::Pisces, Sign::Virgo],
+        },
         ..ViewState::default()
     };
     {
@@ -216,6 +220,10 @@ fn open_client_round_trips_and_is_fast() {
         restored.view_state.dasha_expanded_path,
         vs2.dasha_expanded_path
     );
+    assert_eq!(
+        restored.view_state.chara_expanded_path,
+        vs.chara_expanded_path
+    );
     assert_eq!(restored.view_state.panel_layout, vs2.panel_layout);
 
     // Harita olmadan düzen kaydı (`save_panel_layout`/`load_panel_layout` gövdeleri).
````

- [ ] **Step 2: Başarısız testleri yaz (arayüz)**

````diff
diff --git a/app/ui/session/store.test.tsx b/app/ui/session/store.test.tsx
index 047f055..70f6266 100644
--- a/app/ui/session/store.test.tsx
+++ b/app/ui/session/store.test.tsx
@@ -16,7 +16,7 @@ function other(): OpenedClient {
   const o = fixture().opened;
   o.client = { ...o.client, id: 2, name: "İkinci Danışan" };
   o.chart_record = { ...o.chart_record, id: 9, client_id: 2, tags: ["kariyer"] };
-  o.view_state = { ...o.view_state, dasha_expanded_path: ["Venus"] };
+  o.view_state = { ...o.view_state, dasha_expanded_path: ["Venus"], chara_expanded_path: { cycle: 2, signs: ["Gemini"] } };
   o.warnings = ["LastChartMissing"];
   return o;
 }
@@ -30,6 +30,7 @@ function Probe() {
       <output aria-label="client">{session.opened?.client.name ?? "-"}</output>
       <output aria-label="chart">{String(view.model.chartId)}</output>
       <output aria-label="path">{view.model.dashaPath.join(">")}</output>
+      <output aria-label="chara">{`${view.model.charaPath.cycle}:${view.model.charaPath.signs.join(">")}`}</output>
       <output aria-label="tags">{session.opened?.chart_record.tags.join(",") ?? "-"}</output>
       <output aria-label="revision">{session.revision}</output>
       <button onClick={() => view.setDashaPath(["Rahu"])}>path</button>
@@ -58,6 +59,8 @@ test("openClient flushes the previous chart, opens the reply and swaps the notic
   expect(persistence.saved.map((s) => [s.chartId, s.viewState.dasha_expanded_path])).toEqual([[1, ["Rahu"]]]);
   expect(out("chart")).toHaveTextContent("9");
   expect(out("path")).toHaveTextContent("Venus");
+  // Açılan haritanın chara dasha yolu da yüklenir.
+  expect(out("chara")).toHaveTextContent("2:Gemini");
   expect(screen.queryByText(/Son oturum açılamadı/)).toBeNull();
   expect(screen.getByText("Son açılan harita silinmiş; en son güncellenen harita açıldı.")).toBeInTheDocument();
   expect(out("revision")).toHaveTextContent("1");
diff --git a/app/ui/view/model.test.ts b/app/ui/view/model.test.ts
index 0eaed08..6c3148b 100644
--- a/app/ui/view/model.test.ts
+++ b/app/ui/view/model.test.ts
@@ -1,5 +1,6 @@
 import { BUILTIN_LAYOUT_IDS, defaultLayoutState } from "../layout/model";
-import { initialViewModel, parseDashaPath, toViewState, VIEW_STATE_SCHEMA_VERSION } from "./model";
+import { fixture } from "../test/fixture";
+import { chartViewOf, emptyCharaPath, initialViewModel, parseCharaPath, parseDashaPath, toViewState, VIEW_STATE_SCHEMA_VERSION } from "./model";
 import { viewReducer } from "./reducer";
 
 test("parseDashaPath is tolerant like the Rust store", () => {
@@ -10,24 +11,61 @@ test("parseDashaPath is tolerant like the Rust store", () => {
   expect(parseDashaPath(null)).toEqual([]);
 });
 
+test("parseCharaPath is tolerant like the Rust store", () => {
+  expect(parseCharaPath({ cycle: 2, signs: ["Aries", "Taurus", "Gemini"] })).toEqual({ cycle: 2, signs: ["Aries", "Taurus", "Gemini"] });
+  expect(parseCharaPath({ cycle: 1, signs: [] })).toEqual(emptyCharaPath());
+  // Alanı taşımayan eski kayıt, yanlış çevrim, geçersiz burç ve dört burçluk yol: boş yol.
+  for (const bad of [undefined, null, ["Aries"], { signs: ["Aries"] }, { cycle: 1 }, { cycle: 0, signs: ["Aries"] }, { cycle: 3, signs: [] }, { cycle: "1", signs: [] }, { cycle: 1, signs: ["Aries", "Ophiuchus"] }, { cycle: 1, signs: ["Aries", "Taurus", "Gemini", "Cancer"] }]) {
+    expect(parseCharaPath(bad)).toEqual(emptyCharaPath());
+  }
+});
+
 test("toViewState serialises the whole model in the IPC shape", () => {
-  const m = initialViewModel({ active_layout_id: BUILTIN_LAYOUT_IDS.research }, { id: 3, dashaPath: ["Venus"] });
+  const m = initialViewModel({ active_layout_id: BUILTIN_LAYOUT_IDS.research }, { id: 3, dashaPath: ["Venus"], charaPath: { cycle: 2, signs: ["Leo"] } });
   const vs = toViewState(m);
   expect(vs).toEqual({
     schema_version: VIEW_STATE_SCHEMA_VERSION,
     panel_layout: { active_layout_id: BUILTIN_LAYOUT_IDS.research, layouts: defaultLayoutState().layouts },
     dasha_expanded_path: ["Venus"],
+    chara_expanded_path: { cycle: 2, signs: ["Leo"] },
   });
   expect(m.chartId).toBe(3);
   // Serileştirilen durum yeniden okunduğunda aynı modeli verir.
-  expect(initialViewModel(vs.panel_layout, { id: 3, dashaPath: vs.dasha_expanded_path })).toEqual(m);
+  expect(initialViewModel(vs.panel_layout, { id: 3, dashaPath: vs.dasha_expanded_path, charaPath: vs.chara_expanded_path })).toEqual(m);
+});
+
+test("a view written before the chara path opens with the Vimshottari path and an empty chara path", () => {
+  const m = initialViewModel(null, { id: 3, dashaPath: ["Rahu", "Saturn"] });
+  expect([m.dashaPath, m.charaPath]).toEqual([["Rahu", "Saturn"], emptyCharaPath()]);
+  expect(initialViewModel(null).charaPath).toEqual(emptyCharaPath());
+  // Açılan haritanın görünüm kısmı iki yolu da taşır.
+  const opened = fixture().opened;
+  opened.view_state.chara_expanded_path = { cycle: 1, signs: ["Gemini"] };
+  expect(chartViewOf(opened)).toEqual({ id: opened.chart_record.id, dashaPath: [], charaPath: { cycle: 1, signs: ["Gemini"] } });
 });
 
 test("the reducer keeps identity for no-op changes", () => {
   const m = initialViewModel(null);
   expect(viewReducer(m, { type: "setDashaPath", path: [] })).toBe(m);
   expect(viewReducer(m, { type: "layout", action: { type: "select", id: "nope" } })).toBe(m);
+  expect(viewReducer(m, { type: "setCharaPath", path: emptyCharaPath() })).toBe(m);
   const opened = viewReducer(m, { type: "openChart", chartId: 1, dashaPath: ["Sun"] });
   expect([opened.chartId, opened.dashaPath]).toEqual([1, ["Sun"]]);
   expect(opened.layout).toBe(m.layout);
 });
+
+test("the two dasha paths change independently and both reset when the chart closes", () => {
+  let m = viewReducer(initialViewModel(null), { type: "openChart", chartId: 1, dashaPath: ["Sun"], charaPath: { cycle: 1, signs: ["Aries"] } });
+  expect(m.charaPath).toEqual({ cycle: 1, signs: ["Aries"] });
+  m = viewReducer(m, { type: "setCharaPath", path: { cycle: 2, signs: ["Aries", "Taurus"] } });
+  expect([m.dashaPath, m.charaPath]).toEqual([["Sun"], { cycle: 2, signs: ["Aries", "Taurus"] }]);
+  // Aynı burçlar, başka çevrim: ayrı bir yoldur.
+  expect(viewReducer(m, { type: "setCharaPath", path: { cycle: 2, signs: ["Aries", "Taurus"] } })).toBe(m);
+  expect(viewReducer(m, { type: "setCharaPath", path: { cycle: 1, signs: ["Aries", "Taurus"] } })).not.toBe(m);
+  m = viewReducer(m, { type: "setDashaPath", path: ["Moon"] });
+  expect([m.dashaPath, m.charaPath]).toEqual([["Moon"], { cycle: 2, signs: ["Aries", "Taurus"] }]);
+  // Geçersiz yol boş yola döner.
+  expect(viewReducer(m, { type: "setCharaPath", path: { cycle: 5, signs: [] } }).charaPath).toEqual(emptyCharaPath());
+  const closed = viewReducer(m, { type: "closeChart" });
+  expect([closed.dashaPath, closed.charaPath]).toEqual([[], emptyCharaPath()]);
+});
diff --git a/app/ui/view/persistence.test.ts b/app/ui/view/persistence.test.ts
index 6f2e1c6..e00e769 100644
--- a/app/ui/view/persistence.test.ts
+++ b/app/ui/view/persistence.test.ts
@@ -9,7 +9,7 @@ import { BUILTIN_LAYOUT_IDS } from "../layout/model";
 import { LEGACY_LAYOUT_KEY, loadLayout, storePersistence } from "./persistence";
 
 const ok = <T,>(data: T) => ({ status: "ok" as const, data });
-const vs = { schema_version: 1, panel_layout: { active_layout_id: "a", layouts: [] }, dasha_expanded_path: [] };
+const vs = { schema_version: 1, panel_layout: { active_layout_id: "a", layouts: [] }, dasha_expanded_path: [], chara_expanded_path: { cycle: 1, signs: [] } };
 
 afterEach(() => vi.mocked(api.savePanelLayout).mockReset());
 
diff --git a/app/ui/view/store.test.tsx b/app/ui/view/store.test.tsx
index d242c3e..1d0ef0a 100644
--- a/app/ui/view/store.test.tsx
+++ b/app/ui/view/store.test.tsx
@@ -15,10 +15,12 @@ function Probe() {
       <span data-testid="count">{store.state.layouts.length}</span>
       <span data-testid="chart">{String(view.model.chartId)}</span>
       <span data-testid="path">{view.model.dashaPath.join(">")}</span>
+      <span data-testid="chara">{`${view.model.charaPath.cycle}:${view.model.charaPath.signs.join(">")}`}</span>
       <button onClick={() => store.saveAs("Benim")}>saveAs</button>
       <button onClick={() => store.select(BUILTIN_LAYOUT_IDS.research)}>research</button>
       <button onClick={() => view.setDashaPath(["Rahu", "Jupiter"] as Graha[])}>path</button>
-      <button onClick={() => view.openChart(7, ["Venus"])}>open7</button>
+      <button onClick={() => view.setCharaPath({ cycle: 2, signs: ["Aries", "Taurus"] })}>chara</button>
+      <button onClick={() => view.openChart(7, ["Venus"], { cycle: 1, signs: ["Leo"] })}>open7</button>
       <button onClick={() => view.openChart(8, "garbage")}>open8</button>
       <button onClick={() => view.flush()}>flush</button>
       <button
@@ -86,6 +88,23 @@ test("the dasha path is part of the saved ViewState of the open chart", () => {
   ]);
 });
 
+test("the chara dasha path is saved with the open chart and loaded per chart", () => {
+  const p = memoryPersistence();
+  setup(p);
+  fireEvent.click(screen.getByText("open7"));
+  expect(screen.getByTestId("chara")).toHaveTextContent("1:Leo");
+  fireEvent.click(screen.getByText("chara"));
+  expect(screen.getByTestId("chara")).toHaveTextContent("2:Aries>Taurus");
+  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS));
+  // Vimshottari yolu olduğu gibi durur: iki yol ayrıdır.
+  expect(p.saved).toEqual<ViewSnapshot[]>([
+    { chartId: 7, viewState: expect.objectContaining({ dasha_expanded_path: ["Venus"], chara_expanded_path: { cycle: 2, signs: ["Aries", "Taurus"] } }) },
+  ]);
+  // Yolu olmayan haritada boş yol.
+  fireEvent.click(screen.getByText("open8"));
+  expect(screen.getByTestId("chara")).toHaveTextContent("1:");
+});
+
 test("opening another chart flushes the pending save to the previous chart first", () => {
   const p = memoryPersistence();
   setup(p);
````

- [ ] **Step 3: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p store 2>&1 | grep -E "^error" | sort | uniq -c | head
(cd app && npx vitest run ui/view ui/session 2>&1 | grep -E "Test Files|Tests ")
```
Expected: ilk komutta derleme hataları (`cannot find function empty_chara_path`, `chart_view_chara_path`, `empty_chart_view_json`; `cannot find struct … SignDashaPath`; `this function takes 1 argument but 2 arguments were supplied`) ve `could not compile`; ikincide `3 failed | 1 passed (4)` dosya ve `24 failed | 11 passed (35)` test.

- [ ] **Step 4: Store: tip, ayrıştırma, denetim, okuma ve yazma**

````diff
diff --git a/crates/store/src/charts.rs b/crates/store/src/charts.rs
index 32b9309..d7e56d0 100644
--- a/crates/store/src/charts.rs
+++ b/crates/store/src/charts.rs
@@ -13,7 +13,8 @@ use crate::model::{
 };
 use crate::settings::{load_calc_settings, SettingsLoad};
 use crate::view_state::{
-    chart_view_json, chart_view_path, parse_panel_layout, PanelLayout, ViewState,
+    chart_view_chara_path, chart_view_json, chart_view_path, empty_chart_view_json,
+    parse_panel_layout, PanelLayout, ViewState,
 };
 use crate::{Store, StoreError};
 
@@ -299,7 +300,7 @@ impl Store {
                 birth: &birth,
                 resolved: &resolved,
                 calc_json: &calc_json,
-                view_json: &chart_view_json(&[]),
+                view_json: &empty_chart_view_json(),
                 created_at: &now,
                 updated_at: &now,
             },
@@ -335,7 +336,7 @@ impl Store {
                 birth: &birth,
                 resolved: &resolved,
                 calc_json: &calc_json,
-                view_json: &chart_view_json(&[]),
+                view_json: &empty_chart_view_json(),
                 created_at: &now,
                 updated_at: &now,
             },
@@ -517,7 +518,7 @@ impl Store {
             .and_then(|v| parse_panel_layout(&v)))
     }
 
-    /// Görünüm durumu: düzenler `app_state`'ten, dasha yolu haritadan (toleranslı).
+    /// Görünüm durumu: düzenler `app_state`'ten, iki dasha yolu haritadan (toleranslı).
     pub fn view_state(&self, chart_id: u32) -> Result<ViewState, StoreError> {
         let raw: String = self
             .conn
@@ -534,6 +535,7 @@ impl Store {
         Ok(ViewState {
             panel_layout: self.panel_layout()?,
             dasha_expanded_path: chart_view_path(&raw),
+            chara_expanded_path: chart_view_chara_path(&raw),
             ..ViewState::default()
         })
     }
@@ -545,7 +547,10 @@ impl Store {
         let tx = self.conn.transaction()?;
         let n = tx.execute(
             "UPDATE charts SET view_state_json = ?1 WHERE id = ?2",
-            params![chart_view_json(&vs.dasha_expanded_path), chart_id],
+            params![
+                chart_view_json(&vs.dasha_expanded_path, &vs.chara_expanded_path),
+                chart_id
+            ],
         )?;
         if n == 0 {
             return Err(StoreError::NotFound {
diff --git a/crates/store/src/exchange.rs b/crates/store/src/exchange.rs
index b66142f..e9ac5d3 100644
--- a/crates/store/src/exchange.rs
+++ b/crates/store/src/exchange.rs
@@ -13,7 +13,7 @@ use crate::charts::{clean_name, insert_chart, insert_client, ChartRow};
 use crate::db::format_ts;
 use crate::model::{BirthData, ChartKind, Gender, ResolvedRecord, TzChoice};
 use crate::settings::{load_calc_value, SettingsLoad};
-use crate::view_state::{chart_view_json, parse_dasha_path};
+use crate::view_state::{chart_view_json, parse_chara_path, parse_dasha_path};
 use crate::{Store, StoreError};
 
 pub const EXPORT_FORMAT: &str = "jyotish-desk";
@@ -57,7 +57,8 @@ pub struct ExportChart {
     pub resolved: ResolvedRecord,
     /// Saklanan JSON aynen (daha yeni sürümün alanları korunur).
     pub calc_settings: Value,
-    /// Haritanın görünüm kısmı (`dasha_expanded_path`); düzenler kullanıcınındır, aktarılmaz.
+    /// Haritanın görünüm kısmı (`dasha_expanded_path`, `chara_expanded_path`); düzenler
+    /// kullanıcınındır, aktarılmaz.
     pub view_state: Value,
     pub tags: Vec<String>,
     pub created_at: String,
@@ -342,6 +343,7 @@ impl Store {
                     other => other.settings().clone(),
                 };
                 let path = parse_dasha_path(ch.view_state.get("dasha_expanded_path"));
+                let chara = parse_chara_path(ch.view_state.get("chara_expanded_path"));
                 let chart_id = insert_chart(
                     &tx,
                     &ChartRow {
@@ -352,7 +354,7 @@ impl Store {
                         birth: &parsed.canonical,
                         resolved: &resolved,
                         calc_json: &serde_json::to_string(&settings)?,
-                        view_json: &chart_view_json(&path),
+                        view_json: &chart_view_json(&path, &chara),
                         created_at: &ts_or(&ch.created_at, &now),
                         updated_at: &ts_or(&ch.updated_at, &now),
                     },
diff --git a/crates/store/src/lib.rs b/crates/store/src/lib.rs
index 0b9b4fa..850b3e8 100644
--- a/crates/store/src/lib.rs
+++ b/crates/store/src/lib.rs
@@ -41,7 +41,8 @@ pub use settings::{
     APP_PREFS_SCHEMA_VERSION, CALC_SETTINGS_MIGRATIONS, LOCALES,
 };
 pub use view_state::{
-    chart_view_json, chart_view_path, check_panel_layout, check_view_state, parse_dasha_path,
+    chart_view_chara_path, chart_view_json, chart_view_path, check_panel_layout, check_view_state,
+    empty_chara_path, empty_chart_view_json, parse_chara_path, parse_dasha_path,
     parse_panel_layout, parse_view_state, Layout, Panel, PanelLayout, ViewState, MAX_LAYOUTS,
     MAX_PANELS, MAX_VIEW_STATE_BYTES, VIEW_STATE_SCHEMA_VERSION,
 };
diff --git a/crates/store/src/view_state.rs b/crates/store/src/view_state.rs
index fec41fc..a15d5f0 100644
--- a/crates/store/src/view_state.rs
+++ b/crates/store/src/view_state.rs
@@ -3,15 +3,35 @@
 //! değerde varsayılan kullanılır).
 //!
 //! Saklama ikiye ayrılır (UI spec §3.2 "Düzenler kullanıcıya aittir"): `panel_layout`
-//! kullanıcınındır ve `app_state` `panel_layout` anahtarındadır; `dasha_expanded_path`
+//! kullanıcınındır ve `app_state` `panel_layout` anahtarındadır; dasha ağacının açık dalı
 //! haritanındır ve `charts.view_state_json`'dadır. IPC'de ikisi tek `ViewState`'tir.
+//!
+//! Açık dal dasa sistemi başına ayrı tutulur (ileri modüller spec'i §6): Vimshottari yolu
+//! `dasha_expanded_path`, chara dasha yolu `chara_expanded_path` alanındadır. Şema sürümü
+//! değişmez: ikinci alanı taşımayan eski kayıt Vimshottari yolu sayılır ve chara yolu boş
+//! okunur.
 
-use jyotish_core::{Graha, MAX_DASHA_LEVEL};
+use jyotish_core::{Graha, Sign, SignDashaPath, MAX_DASHA_LEVEL, MAX_SIGN_DASHA_LEVEL};
 use serde::{Deserialize, Serialize};
 use serde_json::{json, Map, Value};
 
 /// Dasha yolunun en çok uzunluğu (`MAX_DASHA_LEVEL` seviye).
 const MAX_PATH: usize = MAX_DASHA_LEVEL as usize;
+/// Chara dasha yolunun en çok uzunluğu (`MAX_SIGN_DASHA_LEVEL` seviye).
+const MAX_SIGN_PATH: usize = MAX_SIGN_DASHA_LEVEL as usize;
+
+/// Boş chara dasha yolu: ilk çevrim, açık dal yok.
+pub fn empty_chara_path() -> SignDashaPath {
+    SignDashaPath {
+        cycle: 1,
+        signs: Vec::new(),
+    }
+}
+
+/// Chara dasha yolu geçerli mi: çevrim 1 ya da 2, en çok `MAX_SIGN_DASHA_LEVEL` burç.
+fn chara_path_fits(path: &SignDashaPath) -> bool {
+    (1..=2).contains(&path.cycle) && path.signs.len() <= MAX_SIGN_PATH
+}
 
 pub const VIEW_STATE_SCHEMA_VERSION: u32 = 1;
 /// Okunan en çok düzen (hazır düzenler dahil); fazlası atılır. Arayüzün `MAX_LAYOUTS`'u
@@ -33,10 +53,11 @@ pub fn check_panel_layout(pl: &PanelLayout) -> Result<(), crate::StoreError> {
     Ok(())
 }
 
-/// `save_view_state` denetimi: sürüm, dasha yolu uzunluğu ve düzen sınırları.
+/// `save_view_state` denetimi: sürüm, iki dasha yolunun sınırları ve düzen sınırları.
 pub fn check_view_state(vs: &ViewState) -> Result<(), crate::StoreError> {
     if vs.schema_version != VIEW_STATE_SCHEMA_VERSION
         || vs.dasha_expanded_path.len() > usize::from(MAX_DASHA_LEVEL)
+        || !chara_path_fits(&vs.chara_expanded_path)
     {
         return Err(crate::StoreError::InvalidInput("view_state"));
     }
@@ -53,8 +74,11 @@ pub struct ViewState {
     /// Kullanıcının pano düzenleri. `None`: hiç kaydedilmemiş ya da okunamadı; arayüz
     /// hazır düzenleri (`defaultLayoutState`) kullanır.
     pub panel_layout: Option<PanelLayout>,
-    /// Dasha ağacında açık dal: kökten lord yolu, en çok `MAX_DASHA_LEVEL`.
+    /// Vimshottari ağacında açık dal: kökten lord yolu, en çok `MAX_DASHA_LEVEL`.
     pub dasha_expanded_path: Vec<Graha>,
+    /// Chara dasha ağacında açık dal: çevrim ve kökten burç yolu, en çok
+    /// `MAX_SIGN_DASHA_LEVEL`. Boş yolun çevrimi 1'dir.
+    pub chara_expanded_path: SignDashaPath,
 }
 
 impl Default for ViewState {
@@ -63,6 +87,7 @@ impl Default for ViewState {
             schema_version: VIEW_STATE_SCHEMA_VERSION,
             panel_layout: None,
             dasha_expanded_path: Vec::new(),
+            chara_expanded_path: empty_chara_path(),
         }
     }
 }
@@ -201,6 +226,26 @@ pub fn parse_dasha_path(v: Option<&Value>) -> Vec<Graha> {
         .unwrap_or_default()
 }
 
+/// Chara dasha yolu: `{"cycle": 1 | 2, "signs": [burç, …]}`. Nesne değilse, çevrim ya da bir
+/// burç geçersizse ya da yol `MAX_SIGN_DASHA_LEVEL`'den uzunsa boş yol. Alan hiç yoksa (bu
+/// alandan önce yazılmış kayıt) yine boş yol.
+pub fn parse_chara_path(v: Option<&Value>) -> SignDashaPath {
+    let parsed = (|| {
+        let o = v?.as_object()?;
+        let cycle = u8::try_from(o.get("cycle")?.as_u64()?).ok()?;
+        let signs = o
+            .get("signs")?
+            .as_array()?
+            .iter()
+            .map(|s| serde_json::from_value::<Sign>(s.clone()).ok())
+            .collect::<Option<Vec<_>>>()?;
+        Some(SignDashaPath { cycle, signs })
+    })();
+    parsed
+        .filter(chara_path_fits)
+        .unwrap_or_else(empty_chara_path)
+}
+
 /// Tam `ViewState` JSON'unu toleranslı okur (içe aktarma ve testler). Hiçbir girdide
 /// hata ya da panik yoktur.
 pub fn parse_view_state(raw: &str) -> ViewState {
@@ -211,18 +256,26 @@ pub fn parse_view_state(raw: &str) -> ViewState {
         schema_version: VIEW_STATE_SCHEMA_VERSION,
         panel_layout: v.get("panel_layout").and_then(parse_panel_layout),
         dasha_expanded_path: parse_dasha_path(v.get("dasha_expanded_path")),
+        chara_expanded_path: parse_chara_path(v.get("chara_expanded_path")),
     }
 }
 
-/// Haritaya düşen kısım: `{"schema_version":1,"dasha_expanded_path":[…]}`.
-pub fn chart_view_json(path: &[Graha]) -> String {
+/// Haritaya düşen kısım:
+/// `{"schema_version":1,"dasha_expanded_path":[…],"chara_expanded_path":{"cycle":1,"signs":[…]}}`.
+pub fn chart_view_json(path: &[Graha], chara: &SignDashaPath) -> String {
     json!({
         "schema_version": VIEW_STATE_SCHEMA_VERSION,
         "dasha_expanded_path": path,
+        "chara_expanded_path": chara,
     })
     .to_string()
 }
 
+/// Yeni haritanın görünüm kaydı: iki yol da boş.
+pub fn empty_chart_view_json() -> String {
+    chart_view_json(&[], &empty_chara_path())
+}
+
 /// `charts.view_state_json`'dan lord yolu; bozuk JSON boş yol verir.
 pub fn chart_view_path(raw: &str) -> Vec<Graha> {
     serde_json::from_str::<Value>(raw)
@@ -230,6 +283,13 @@ pub fn chart_view_path(raw: &str) -> Vec<Graha> {
         .unwrap_or_default()
 }
 
+/// `charts.view_state_json`'dan chara dasha yolu; bozuk JSON ya da eksik alan boş yol verir.
+pub fn chart_view_chara_path(raw: &str) -> SignDashaPath {
+    serde_json::from_str::<Value>(raw)
+        .map(|v| parse_chara_path(v.get("chara_expanded_path")))
+        .unwrap_or_else(|_| empty_chara_path())
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
````

- [ ] **Step 5: Arayüz: görünüm modeli, reducer, depo ve açılış**

`chartViewOf` açılan haritanın iki yolunu tek yerden verir; `App.tsx`, `test/render.tsx` ve oturum deposu onu (ya da aynı iki alanı) kullanır. `openChart`'ın üçüncü bağımsız değişkeni isteğe bağlıdır: verilmezse chara yolu boştur.

````diff
diff --git a/app/ui/App.tsx b/app/ui/App.tsx
index 3528516..24f32c0 100644
--- a/app/ui/App.tsx
+++ b/app/ui/App.tsx
@@ -6,7 +6,7 @@ import { PrefsProvider } from "./settings/prefs";
 import { SessionProvider } from "./session/store";
 import { NoticeProvider, storeWarningNotices, useNotices, type Notice } from "./shell/notices";
 import { Shell } from "./shell/Shell";
-import { initialViewModel } from "./view/model";
+import { chartViewOf, initialViewModel } from "./view/model";
 import { storePersistence, type ViewPersistence } from "./view/persistence";
 import { ViewStateProvider } from "./view/store";
 
@@ -45,10 +45,7 @@ function Stores({ boot, persistence, children }: { boot: Boot; persistence?: Vie
       ),
   );
   const [initial] = useState(() =>
-    initialViewModel(
-      boot.layout,
-      boot.opened && { id: boot.opened.chart_record.id, dashaPath: boot.opened.view_state.dasha_expanded_path },
-    ),
+    initialViewModel(boot.layout, boot.opened && chartViewOf(boot.opened)),
   );
   return (
     <ViewStateProvider initial={initial} persistence={store}>
diff --git a/app/ui/session/store.tsx b/app/ui/session/store.tsx
index 653306f..15c7d43 100644
--- a/app/ui/session/store.tsx
+++ b/app/ui/session/store.tsx
@@ -38,7 +38,7 @@ const SessionContext = createContext<Session | null>(null);
 
 /**
  * `initial` açılışta geri yüklenen haritadır (`restore_session`); görünüm deposunun ilk
- * modeli aynı haritayla kurulmuş olmalıdır (`initialViewModel(layout, { id, dashaPath })`).
+ * modeli aynı haritayla kurulmuş olmalıdır (`initialViewModel(layout, chartViewOf(opened))`).
  */
 export function SessionProvider({ initial, children }: { initial: OpenedClient | null; children: ReactNode }) {
   const [opened, setOpened] = useState(initial);
@@ -55,7 +55,7 @@ export function SessionProvider({ initial, children }: { initial: OpenedClient |
       // Eski açılış hatası ("client") da kalkar: yeni harita açıldı.
       dismissWhere((n) => n.id === "boot" || n.id === "client" || n.id.startsWith("store:"));
       for (const n of storeWarningNotices(o.warnings)) push(n);
-      openChart(o.chart_record.id, o.view_state.dasha_expanded_path);
+      openChart(o.chart_record.id, o.view_state.dasha_expanded_path, o.view_state.chara_expanded_path);
       setOpened(o);
       touch();
     },
diff --git a/app/ui/test/render.tsx b/app/ui/test/render.tsx
index 1af082e..e596c83 100644
--- a/app/ui/test/render.tsx
+++ b/app/ui/test/render.tsx
@@ -9,7 +9,7 @@ import { PrefsProvider } from "../settings/prefs";
 import { SessionProvider } from "../session/store";
 import { NoticeProvider } from "../shell/notices";
 import { TransitProvider } from "../transit/store";
-import { initialViewModel } from "../view/model";
+import { chartViewOf, initialViewModel } from "../view/model";
 import { memoryPersistence } from "../view/persistence";
 import { ViewStateProvider } from "../view/store";
 import { fixture } from "./fixture";
@@ -25,10 +25,7 @@ export interface RenderOptions {
 
 export function renderWithStores(ui: ReactNode, { opened = fixture().opened, layout = null, debounceMs = 0, prefs = fixture().prefs.prefs }: RenderOptions = {}) {
   const persistence = memoryPersistence();
-  const initial = initialViewModel(
-    layout,
-    opened && { id: opened.chart_record.id, dashaPath: opened.view_state.dasha_expanded_path },
-  );
+  const initial = initialViewModel(layout, opened && chartViewOf(opened));
   const utils = render(
     <NoticeProvider>
       <PrefsProvider initial={prefs}>
diff --git a/app/ui/view/model.ts b/app/ui/view/model.ts
index 0855f00..2519092 100644
--- a/app/ui/view/model.ts
+++ b/app/ui/view/model.ts
@@ -1,20 +1,32 @@
 // Görünüm durumu (spec §5 `ViewState`, §11 M5): pano düzeni ve dasha ağacının açık dalı
 // tek bir modelde tutulur ve tek bir yerden (`toViewState`) serileştirilir. Hesap ayarları
-// (`CalcSettings`) burada değildir; ikisi ayrıdır (spec §4 "Ayar katmanları").
-import type { Graha, ViewState } from "../bindings";
+// (`CalcSettings`) burada değildir; ikisi ayrıdır (spec §4 "Ayar katmanları"). Açık dal dasa
+// sistemi başına ayrıdır (ileri modüller spec'i §6): Vimshottari'nin lord yolu ve chara
+// dashanın burç yolu.
+import type { Graha, Sign, SignDashaPath, ViewState } from "../bindings";
+import { SIGNS } from "../chart/adapter";
 import { isGrahaId } from "../chart/types";
 import { parseLayoutState, type LayoutState } from "../layout/model";
 
 export const VIEW_STATE_SCHEMA_VERSION = 1;
 /** Rust `MAX_DASHA_LEVEL`. */
 export const MAX_DASHA_PATH = 5;
+/** Rust `MAX_SIGN_DASHA_LEVEL`. */
+export const MAX_CHARA_PATH = 3;
+
+/** Boş chara dasha yolu: ilk çevrim, açık dal yok (Rust `empty_chara_path`). */
+export function emptyCharaPath(): SignDashaPath {
+  return { cycle: 1, signs: [] };
+}
 
 export interface ViewModel {
   /** Görünümün yazıldığı harita; danışan açık değilse `null` (yalnızca düzen kaydedilir). */
   chartId: number | null;
   layout: LayoutState;
-  /** Dasha ağacında açık dal: kökten lord yolu (`ViewState.dasha_expanded_path`). */
+  /** Vimshottari ağacında açık dal: kökten lord yolu (`ViewState.dasha_expanded_path`). */
   dashaPath: Graha[];
+  /** Chara dasha ağacında açık dal: çevrim ve burç yolu (`ViewState.chara_expanded_path`). */
+  charaPath: SignDashaPath;
 }
 
 /** Toleranslı: dizi değilse, geçersiz lord içeriyorsa ya da 5'ten uzunsa boş yol (Rust ile aynı). */
@@ -23,12 +35,41 @@ export function parseDashaPath(raw: unknown): Graha[] {
   return raw.every((g) => typeof g === "string" && isGrahaId(g)) ? ([...raw] as Graha[]) : [];
 }
 
-/** Açılış modeli: düzen ham kayıttan (toleranslı), dasha yolu açılan haritadan. */
-export function initialViewModel(rawLayout: unknown, chart: { id: number; dashaPath: unknown } | null = null): ViewModel {
+/**
+ * Toleranslı: nesne değilse, çevrim 1 ya da 2 değilse, geçersiz burç içeriyorsa ya da 3'ten
+ * uzunsa boş yol (Rust `parse_chara_path` ile aynı). Alanı taşımayan eski kayıt da boş yoldur.
+ */
+export function parseCharaPath(raw: unknown): SignDashaPath {
+  if (typeof raw !== "object" || raw === null) return emptyCharaPath();
+  const { cycle, signs } = raw as { cycle?: unknown; signs?: unknown };
+  if ((cycle !== 1 && cycle !== 2) || !Array.isArray(signs) || signs.length > MAX_CHARA_PATH) return emptyCharaPath();
+  if (!signs.every((s) => typeof s === "string" && (SIGNS as readonly string[]).includes(s))) return emptyCharaPath();
+  return { cycle, signs: [...signs] as Sign[] };
+}
+
+export function sameCharaPath(a: SignDashaPath, b: SignDashaPath): boolean {
+  return a.cycle === b.cycle && a.signs.length === b.signs.length && a.signs.every((s, i) => s === b.signs[i]);
+}
+
+/** Açılan haritanın görünüm kısmı: kimlik ve iki dasha yolu (ham; toleranslı okunur). */
+export interface ChartView {
+  id: number;
+  dashaPath: unknown;
+  charaPath?: unknown;
+}
+
+/** `OpenedClient`'tan görünüm kısmı. */
+export function chartViewOf(opened: { chart_record: { id: number }; view_state: ViewState }): ChartView {
+  return { id: opened.chart_record.id, dashaPath: opened.view_state.dasha_expanded_path, charaPath: opened.view_state.chara_expanded_path };
+}
+
+/** Açılış modeli: düzen ham kayıttan (toleranslı), dasha yolları açılan haritadan. */
+export function initialViewModel(rawLayout: unknown, chart: ChartView | null = null): ViewModel {
   return {
     chartId: chart?.id ?? null,
     layout: parseLayoutState(rawLayout),
     dashaPath: chart ? parseDashaPath(chart.dashaPath) : [],
+    charaPath: chart ? parseCharaPath(chart.charaPath) : emptyCharaPath(),
   };
 }
 
@@ -38,5 +79,6 @@ export function toViewState(m: ViewModel): ViewState {
     schema_version: VIEW_STATE_SCHEMA_VERSION,
     panel_layout: { active_layout_id: m.layout.active_layout_id, layouts: m.layout.layouts },
     dasha_expanded_path: [...m.dashaPath],
+    chara_expanded_path: { cycle: m.charaPath.cycle, signs: [...m.charaPath.signs] },
   };
 }
diff --git a/app/ui/view/reducer.ts b/app/ui/view/reducer.ts
index dbf7306..86d3054 100644
--- a/app/ui/view/reducer.ts
+++ b/app/ui/view/reducer.ts
@@ -1,13 +1,14 @@
-// Görünüm modeli için saf reducer: düzen eylemleri düzen reducer'ına gider, dasha yolu ve
+// Görünüm modeli için saf reducer: düzen eylemleri düzen reducer'ına gider, iki dasha yolu ve
 // açılan harita burada değişir.
-import type { Graha } from "../bindings";
+import type { Graha, SignDashaPath } from "../bindings";
 import { layoutReducer, type LayoutAction } from "../layout/reducer";
-import { parseDashaPath, type ViewModel } from "./model";
+import { emptyCharaPath, parseCharaPath, parseDashaPath, sameCharaPath, type ViewModel } from "./model";
 
 export type ViewAction =
   | { type: "layout"; action: LayoutAction }
   | { type: "setDashaPath"; path: Graha[] }
-  | { type: "openChart"; chartId: number; dashaPath: unknown }
+  | { type: "setCharaPath"; path: SignDashaPath }
+  | { type: "openChart"; chartId: number; dashaPath: unknown; charaPath?: unknown }
   | { type: "closeChart" };
 
 export function viewReducer(state: ViewModel, action: ViewAction): ViewModel {
@@ -21,9 +22,13 @@ export function viewReducer(state: ViewModel, action: ViewAction): ViewModel {
       const same = path.length === state.dashaPath.length && path.every((g, i) => g === state.dashaPath[i]);
       return same ? state : { ...state, dashaPath: path };
     }
+    case "setCharaPath": {
+      const path = parseCharaPath(action.path);
+      return sameCharaPath(path, state.charaPath) ? state : { ...state, charaPath: path };
+    }
     case "openChart":
-      return { ...state, chartId: action.chartId, dashaPath: parseDashaPath(action.dashaPath) };
+      return { ...state, chartId: action.chartId, dashaPath: parseDashaPath(action.dashaPath), charaPath: parseCharaPath(action.charaPath) };
     case "closeChart":
-      return state.chartId === null ? state : { ...state, chartId: null, dashaPath: [] };
+      return state.chartId === null ? state : { ...state, chartId: null, dashaPath: [], charaPath: emptyCharaPath() };
   }
 }
diff --git a/app/ui/view/store.tsx b/app/ui/view/store.tsx
index c961832..a51aba0 100644
--- a/app/ui/view/store.tsx
+++ b/app/ui/view/store.tsx
@@ -1,9 +1,9 @@
-// Görünüm durumunun tek merkezi (spec §11 M5): düzenler ve dasha yolu tek saf reducer'dan geçer;
+// Görünüm durumunun tek merkezi (spec §11 M5): düzenler ve dasha yolları tek saf reducer'dan geçer;
 // ≈ 500 ms debounce ile kayıt (spec §4), pencere gizlenince ya da kapanınca hemen kayıt,
 // Cmd+1…9 ile sıradaki düzene geçiş (UI spec §3.2). Paneller düzene `useLayoutStore`,
-// dasha yoluna `useViewStore` ile ulaşır; birbirini çağırmaz.
+// dasha yollarına `useViewStore` ile ulaşır; birbirini çağırmaz.
 import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type Dispatch, type ReactNode } from "react";
-import type { Graha, ViewState } from "../bindings";
+import type { Graha, SignDashaPath, ViewState } from "../bindings";
 import type { Layout, LayoutState, ModulePanelKind, Panel } from "../layout/model";
 import { activeLayout, newId, normalizeState, type LayoutAction } from "../layout/reducer";
 import { toViewState, type ViewModel } from "./model";
@@ -36,8 +36,10 @@ export interface ViewStore {
   /** Serileştirilmiş biçim (kayıtla aynı). */
   viewState: ViewState;
   setDashaPath(path: Graha[]): void;
-  /** Yeni açılan harita: bekleyen kayıt önceki haritaya yazılır, yeni haritanın yolu yüklenir. */
-  openChart(chartId: number, dashaPath: unknown): void;
+  /** Chara dasha ağacının açık dalı (çevrim ve burç yolu). */
+  setCharaPath(path: SignDashaPath): void;
+  /** Yeni açılan harita: bekleyen kayıt önceki haritaya yazılır, yeni haritanın yolları yüklenir. */
+  openChart(chartId: number, dashaPath: unknown, charaPath?: unknown): void;
   /** Açık harita kapandı (ör. danışan silindi): bekleyen kayıt yazılır, sonraki kayıt yalnızca düzendir. */
   closeChart(): void;
   /**
@@ -147,7 +149,8 @@ export function ViewStateProvider({
       model: state,
       viewState: toViewState(state),
       setDashaPath: (path) => dispatch({ type: "setDashaPath", path }),
-      openChart: (chartId, dashaPath) => dispatch({ type: "openChart", chartId, dashaPath }),
+      setCharaPath: (path) => dispatch({ type: "setCharaPath", path }),
+      openChart: (chartId, dashaPath, charaPath) => dispatch({ type: "openChart", chartId, dashaPath, charaPath }),
       closeChart: () => dispatch({ type: "closeChart" }),
       flush: () => flushRef.current(),
     };
````

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
git diff --stat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
```
Expected: iki `ok`; `bindings.ts`'te `ViewState` `chara_expanded_path: SignDashaPath` alanını kazanır (6 satır eklenir, 1 satır silinir); `delhi.json` tek satır değişir (`opened.view_state` yeni alanı boş yolla taşır).

- [ ] **Step 6: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
git diff --stat HEAD -- crates/store/src/schema.rs crates/jyotish-core/src/settings.rs
```
Expected: fmt ve clippy çıktısız; `store` lib `86 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `46 passed`, `open_client` `1 passed`, `bindings` 1, `ui_fixture` 2; diğer Rust satırları Task 2'deki gibi; Vitest `68 passed (68)`, `628 passed (628)`; `✓ built`; son komut çıktısız (C3).

- [ ] **Step 7: Commit**

```bash
git add crates/store app/src-tauri app/ui
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(store,ui): the open dasha branch is kept per system: chara_expanded_path next to the Vimshottari path, tolerant on read"
```
Expected: iki `grep` çıktısız.

---

### Task 4: Dasa ağacı: sistem seçicisi, ortak dönem ağacı ve chara dasha ağacı

Spec §6 "Arayüz"; D11, D12, D13. `DashaTreePanel` sistem seçicisini kazanır; ağacın çizimi ve klavyesi `PeriodTree`'ye taşınır; `CharaDashaTree` eklenir. Vimshottari ağacının mevcut testleri (`DashaTreePanel.test.tsx`, 14 test) değişmeden geçmelidir: bu, taşımanın davranışı koruduğunun kanıtıdır.

**Files:**
- Create: `app/ui/dasha/PeriodTree.tsx`, `app/ui/dasha/system.tsx`, `app/ui/dasha/chara.ts`, `app/ui/dasha/CharaDashaTree.tsx`, `app/ui/dasha/CharaDashaTree.test.tsx`
- Modify: `app/ui/dasha/DashaTreePanel.tsx`, `app/ui/dasha/dasha.css`, `app/ui/i18n/{tr,en,fr}.ts`
- Test: `app/ui/dasha/CharaDashaTree.test.tsx` (yeni), `app/ui/dasha/DashaTreePanel.test.tsx` (değişmez)

**Interfaces:**
- Consumes: `api.computeCharaDasha`, `api.charaDashaChildren` (Task 2); `fixture().chara_dasha`, `fixture().chara_dasha_children` (Task 2); `useViewStore().model.charaPath`, `setCharaPath` (Task 3); `useModule`, `chartInput`, `inputKey` (Plan 14); `useLayoutStore().setPanelSettings`; `PanelError`, `PanelSkeleton`, `usePanelLoadPhase`; `signIndex`; `formatJdDate`, `childrenOf`, `isOpen`, `useLazyDasha`, `pathKey` (v1).
- Produces:
  - `dasha/PeriodTree.tsx`: `interface PeriodNode { key: string; level: number; name: string; label: string; dates: string; expandable: boolean; open: boolean; kids?: PeriodLevel; toggle(): void }`; `type PeriodLevel = { status: "ok"; nodes: PeriodNode[] } | { status: "loading" } | { status: "error"; error: CommandError; retry(): void }`; `PeriodTree({ label, roots })`
  - `dasha/system.tsx`: `DASHA_SYSTEMS`, `type DashaSystem = "vimshottari" | "chara"`, `dashaSystem(settings): DashaSystem`, `SystemSelect({ value, onChange })`
  - `dasha/chara.ts`: `MAX_CHARA_LEVEL = 3`, `CHARA_PRELOADED_LEVEL = 2`, `charaKey(cycle, signs): string`, `charaChildrenOf(periods, cycle, path)`, `charaDashas(periods)`, `isCharaOpen(open, cycle, signs)`, `charaName(t, cycle, signs)`, `charaYears(dasha)`, `useCharaDasha(opened, enabled?): ModuleState<CharaDasha | null>`, `useCharaChildren(opened, cycle, path | null): ModuleState<SignDashaPeriod[]>`
  - `dasha/CharaDashaTree.tsx`: `CharaDashaTree({ opened })`
  - i18n: `dasha.system`, `dasha.systems.{vimshottari,chara}`, `dasha.chara.{direction,forward,backward,years,secondCycle,timeUnknown,method}`
  - Panel ayarı: `dasha_tree` `settings.system`.

- [ ] **Step 1: Başarısız testi yaz**

Testler paneli bir bölge çerçevesinin içinde çizer (yeniden deneme odağı için) ve fikstürdeki gerçek dashayı kullanır: Delhi haritasında yön geri, on iki dasha ve ikinci çevrimin sekiz dashası.

**Yeni dosya:** `app/ui/dasha/CharaDashaTree.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { dashaChildren: vi.fn(), computeCharaDasha: vi.fn(), charaDashaChildren: vi.fn() } }));

import { api, type Result } from "../api";
import type { CharaDasha, OpenedClient, SignDashaPath, SignDashaPeriod } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { charaChildrenOf, charaDashas, charaKey, charaYears, isCharaOpen } from "./chara";
import { DashaTreePanel } from "./DashaTreePanel";
import { dashaSystem } from "./system";

const ok = (): Result<CharaDasha | null> => ({ status: "ok", data: fixture().chara_dasha });
const okChildren = (): Result<SignDashaPeriod[]> => ({ status: "ok", data: fixture().chara_dasha_children.periods });

function renderPanel(settings: Record<string, unknown> = { system: "chara" }, path: SignDashaPath = { cycle: 1, signs: [] }, opened: OpenedClient = fixture().opened) {
  const panel: Panel = { id: "dt", kind: "dasha_tree", x: 0, y: 0, w: 6, h: 9, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  opened.view_state.chara_expanded_path = path;
  return renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Dasa ağacı paneli">
      <DashaTreePanel panel={panel} />
    </section>,
    { layout, opened },
  );
}

const tree = () => screen.findByRole("tree", { name: "Dasa ağacı" });
const level = (n: number) => screen.queryAllByRole("treeitem").filter((i) => i.getAttribute("aria-level") === String(n));
const names = (items: HTMLElement[]) => items.map((i) => i.getAttribute("aria-label")!.split(",")[0]);
const click = (name: RegExp) => fireEvent.click(screen.getByRole("treeitem", { name }).querySelector(".dasha-row")!);

beforeEach(() => {
  vi.mocked(api.dashaChildren).mockReset();
  vi.mocked(api.computeCharaDasha).mockReset();
  vi.mocked(api.computeCharaDasha).mockImplementation(async () => ok());
  vi.mocked(api.charaDashaChildren).mockReset();
  vi.mocked(api.charaDashaChildren).mockImplementation(async () => okChildren());
});

test("Vimshottari is the default system; chara dasha is requested only once chosen and the choice is saved with the layout", async () => {
  const { persistence } = renderPanel({});
  const select = screen.getByRole("combobox", { name: "Sistem" });
  expect(select).toHaveDisplayValue("Vimshottari");
  expect(within(await tree()).getAllByRole("treeitem")).toHaveLength(9);
  expect(api.computeCharaDasha).not.toHaveBeenCalled();
  fireEvent.change(select, { target: { value: "chara" } });
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  expect(persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ system: "chara" });
  expect([dashaSystem({}), dashaSystem({ system: "yogini" }), dashaSystem({ system: "vimshottari" }), dashaSystem({ system: "chara" })]).toEqual(["vimshottari", "vimshottari", "vimshottari", "chara"]);
});

test("chara: direction and the twelve years in a header line; rows carry sign abbreviations; the second cycle is prefixed", async () => {
  renderPanel();
  await tree();
  const opened = fixture().opened;
  expect(api.computeCharaDasha).toHaveBeenCalledTimes(1);
  expect(api.computeCharaDasha).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true });
  expect(screen.getByRole("combobox", { name: "Sistem" })).toHaveDisplayValue("Chara dasha");
  // Delhi fikstürü: lagna İkizler, 9. burç Kova (ters sayan): geri.
  expect(screen.getByText("Yön: geri")).toBeInTheDocument();
  expect(screen.getByText("Yıllar: İki 6 · Boğ 6 · Koç 10 · Bal 11 · Kov 10 · Oğl 9 · Yay 4 · Akr 3 · Ter 1 · Baş 9 · Asl 8 · Yen 9")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: chara_dasha.padakrama_v1")).toBeInTheDocument();
  // On iki dasha ve ikinci çevrimde üretilen sekiz dasha; hepsi kapalı.
  const dashas = level(1);
  expect(names(dashas)).toEqual([
    ...["İki", "Boğ", "Koç", "Bal", "Kov", "Oğl", "Yay", "Akr", "Ter", "Baş", "Asl", "Yen"],
    ...["2. İki", "2. Boğ", "2. Koç", "2. Bal", "2. Kov", "2. Oğl", "2. Yay", "2. Akr"],
  ]);
  expect(dashas[0]).toHaveAccessibleName("İki, 2000-01-01 – 2006-01-01");
  expect(dashas[12]).toHaveAccessibleName("2. İki, 2086-01-01 – 2092-01-01");
  expect(dashas[12].querySelector(".dasha-lord")).toHaveTextContent("2. İki");
  expect(dashas.every((d) => d.getAttribute("aria-expanded") === "false")).toBe(true);
  expect(dashas[0]).toHaveAttribute("aria-setsize", "20");
  // Chara ağacında "yaklaşık" notu yoktur ve Vimshottari'nin alt dönem komutu çağrılmaz.
  expect(screen.queryByRole("note")).toBeNull();
  expect(api.dashaChildren).not.toHaveBeenCalled();
});

test("one open path, kept apart from the Vimshottari path and saved with the chart", async () => {
  const opened = fixture().opened;
  opened.view_state.dasha_expanded_path = ["Rahu"];
  const { persistence } = renderPanel({ system: "chara" }, { cycle: 1, signs: [] }, opened);
  await tree();
  click(/^İki, /);
  // Geri yön: ilk alt dönem dasha burcundan sonraki (Boğa), sonuncusu dasha burcu (İkizler).
  expect(names(level(2))).toEqual(["İki–Boğ", "İki–Koç", "İki–Bal", "İki–Kov", "İki–Oğl", "İki–Yay", "İki–Akr", "İki–Ter", "İki–Baş", "İki–Asl", "İki–Yen", "İki–İki"]);
  expect(level(2)[0].querySelector(".dasha-lord")).toHaveTextContent("Boğ");
  // İkinci çevrimin aynı burcu ayrı bir dönemdir: açılınca ilki kapanır.
  click(/^2\. İki, /);
  expect(screen.getByRole("treeitem", { name: /^İki, / })).toHaveAttribute("aria-expanded", "false");
  expect(names(level(2))[0]).toBe("2. İki–Boğ");
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.chara_expanded_path).toEqual({ cycle: 2, signs: ["Gemini"] }));
  expect(persistence.saved.at(-1)!.viewState.dasha_expanded_path).toEqual(["Rahu"]);
  expect(persistence.saved.at(-1)!.chartId).toBe(opened.chart_record.id);
  click(/^2\. İki, /);
  expect(level(2)).toHaveLength(0);
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.chara_expanded_path).toEqual({ cycle: 2, signs: [] }));
});

test("the third level loads on demand for the open path and is the last level", async () => {
  const { path, cycle } = fixture().chara_dasha_children;
  renderPanel({ system: "chara" }, { cycle, signs: path });
  await tree();
  await waitFor(() => expect(level(3)).toHaveLength(12));
  expect(api.charaDashaChildren).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.charaDashaChildren).mock.calls[0].slice(1)).toEqual([["Gemini", "Taurus"], 1]);
  expect(names(level(3))[0]).toBe("İki–Boğ–Koç");
  expect(names(level(3))[11]).toBe("İki–Boğ–Boğ");
  expect(level(3).every((i) => !i.hasAttribute("aria-expanded"))).toBe(true);
  // Klavye yüklenen seviyeye iner; son seviyede sağ ok bir şey açmaz.
  const parent = screen.getByRole("treeitem", { name: /^İki–Boğ, / });
  act(() => parent.focus());
  fireEvent.keyDown(parent, { key: "ArrowRight" });
  expect(document.activeElement).toHaveAccessibleName(/^İki–Boğ–Koç, /);
  fireEvent.keyDown(document.activeElement!, { key: "ArrowRight" });
  expect(document.activeElement).toHaveAccessibleName(/^İki–Boğ–Koç, /);
  fireEvent.keyDown(document.activeElement!, { key: "ArrowLeft" });
  expect(document.activeElement).toBe(parent);
  // Dalı kapatıp açmak yeniden istemez; başka bir alt dönem kendi isteğini yapar.
  click(/^İki–Boğ, /);
  click(/^İki–Boğ, /);
  expect(api.charaDashaChildren).toHaveBeenCalledTimes(1);
});

test("a failed or empty third level is an in-tree error with retry", async () => {
  vi.mocked(api.charaDashaChildren).mockResolvedValueOnce({ status: "error", error: { code: "invalid_dasha_input", params: { field: "path" } } });
  renderPanel({ system: "chara" }, { cycle: 1, signs: ["Gemini", "Taurus"] });
  expect(await screen.findByRole("alert")).toHaveTextContent("Dasa isteği geçersiz: path.");
  // Boş yanıt sessizce boş dal olarak gösterilmez.
  vi.mocked(api.charaDashaChildren).mockResolvedValueOnce({ status: "ok", data: [] });
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  expect(await screen.findByText("Bu dönemin alt dönemleri alınamadı.")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(level(3)).toHaveLength(12));
  expect(api.charaDashaChildren).toHaveBeenCalledTimes(3);
  expect(screen.getByRole("treeitem", { name: /^İki–Boğ, / })).toHaveAttribute("aria-expanded", "true");
});

test("a path that this chart does not have opens nothing and asks for nothing", async () => {
  // Yay ikinci çevrimde üretilir ama Terazi üretilmez (120. yıldan sonra başlardı).
  renderPanel({ system: "chara" }, { cycle: 2, signs: ["Libra", "Scorpio"] });
  await tree();
  expect(level(2)).toHaveLength(0);
  expect(api.charaDashaChildren).not.toHaveBeenCalled();
});

test("unknown birth time: Rust returns no dasha and the panel says why; the selector stays", async () => {
  vi.mocked(api.computeCharaDasha).mockResolvedValue({ status: "ok", data: null });
  renderPanel({ system: "chara" }, { cycle: 1, signs: ["Gemini", "Taurus"] }, unknownTimeOpened());
  expect(await screen.findByText("Doğum saati bilinmiyor: chara dasha lagnaya bağlıdır ve hesaplanmaz.")).toBeInTheDocument();
  expect(vi.mocked(api.computeCharaDasha).mock.calls[0][0].time_known).toBe(false);
  expect(screen.queryByRole("tree")).toBeNull();
  expect(api.charaDashaChildren).not.toHaveBeenCalled();
  expect(screen.getByRole("combobox", { name: "Sistem" })).toHaveDisplayValue("Chara dasha");
});

test("an error is shown under the selector; retry from the keyboard keeps the focus in the panel", async () => {
  vi.mocked(api.computeCharaDasha).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.getByRole("combobox", { name: "Sistem" })).toBeInTheDocument();
  const retry = screen.getByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  await tree();
  expect(api.computeCharaDasha).toHaveBeenCalledTimes(2);
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Dasa ağacı paneli" }));
});

test("loading: nothing under a second, then a skeleton, then the tree", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<CharaDasha | null>) => void;
    vi.mocked(api.computeCharaDasha).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getByRole("tree", { name: "Dasa ağacı" })).toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
});

test("helpers group the periods Rust sent; nothing is computed here", () => {
  const dasha = fixture().chara_dasha;
  expect(charaDashas(dasha.periods)).toHaveLength(20);
  expect(charaChildrenOf(dasha.periods, 2, ["Gemini"]).map((p) => p.signs[1])).toEqual(charaChildrenOf(dasha.periods, 1, ["Gemini"]).map((p) => p.signs[1]));
  expect(charaChildrenOf(dasha.periods, 2, ["Libra"])).toEqual([]);
  expect(charaYears(dasha).map((y) => y.years)).toEqual(dasha.years);
  expect(charaYears(dasha)[0].sign).toBe("Gemini");
  expect(charaKey(2, ["Gemini", "Taurus"])).toBe("2:Gemini,Taurus");
  const open = { cycle: 2, signs: ["Gemini", "Taurus"] } satisfies SignDashaPath;
  expect([isCharaOpen(open, 2, ["Gemini"]), isCharaOpen(open, 1, ["Gemini"]), isCharaOpen(open, 2, ["Gemini", "Aries"]), isCharaOpen(open, 2, ["Gemini", "Taurus", "Aries"])]).toEqual([true, false, false, false]);
});
````

- [ ] **Step 2: Testin başarısız olduğunu doğrula**

```bash
(cd app && npx vitest run ui/dasha/CharaDashaTree.test.tsx 2>&1 | grep -E "Test Files|Tests |Error")
```
Expected: `Error: Failed to resolve import "./chara"`; `Test Files  1 failed (1)`.

- [ ] **Step 3: Ortak dönem ağacı ve sistem seçicisi**

**Yeni dosya:** `app/ui/dasha/PeriodTree.tsx`

````tsx
// Dönem ağacı (WAI-ARIA ağaç deseni): Vimshottari ve chara dasha ağaçlarının ortak çizimi ve
// klavyesi. Düğümleri panel kurar (dönemler Rust'tan gelir; burada hesap yoktur); bu bileşen
// yalnızca çizer, odağı taşır ve açma/kapama isteğini düğümün `toggle`'ına iletir.
//
// Anahtar virgülle ayrılmış yoldur: üst düğümün anahtarı son parça atılınca çıkar
// (`"Rahu,Jupiter"` → `"Rahu"`; `"2:Aries,Taurus"` → `"2:Aries"`).
import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError } from "../bindings";
import { PanelError } from "../panels/states";

export interface PeriodNode {
  key: string;
  /** 1 tabanlı seviye (`aria-level`). */
  level: number;
  /** Satırda görünen ad: dönemin son lordu ya da burcu. */
  name: string;
  /** Erişilebilir ad: kökten yol ve tarihler. */
  label: string;
  dates: string;
  expandable: boolean;
  open: boolean;
  /** Açık düğümün alt seviyesi. */
  kids?: PeriodLevel;
  toggle(): void;
}

export type PeriodLevel = { status: "ok"; nodes: PeriodNode[] } | { status: "loading" } | { status: "error"; error: CommandError; retry(): void };

const parentKey = (key: string) => key.split(",").slice(0, -1).join(",");

export function PeriodTree({ label, roots }: { label: string; roots: PeriodLevel }) {
  const { t } = useTranslation();
  const [focusKey, setFocusKey] = useState<string | null>(null);
  const items = useRef(new Map<string, HTMLLIElement>());
  const treeRef = useRef<HTMLUListElement>(null);

  // Odak klavyeyle taşındıysa yeni öğeye geçer (fare ve ilk çizim odağı çalmaz).
  useEffect(() => {
    if (focusKey === null || !treeRef.current?.contains(document.activeElement)) return;
    items.current.get(focusKey)?.focus();
  }, [focusKey]);

  // Görünür öğeler, ekrandaki sırayla (klavye gezintisi için).
  const visible: PeriodNode[] = [];
  const walk = (level: PeriodLevel | undefined) => {
    if (level?.status !== "ok") return;
    for (const node of level.nodes) {
      visible.push(node);
      if (node.open) walk(node.kids);
    }
  };
  walk(roots);
  // Sekme durağı her zaman görünür bir öğedir: odaklı öğe ağaçtan çıktıysa (başka harita, kapanan
  // yol, silinen tembel seviye) görünür en yakın atası, o da yoksa ilk öğe.
  const visibleKeys = new Set(visible.map((v) => v.key));
  let near = focusKey ?? "";
  while (near !== "" && !visibleKeys.has(near)) near = parentKey(near);
  const current = near !== "" ? near : (visible[0]?.key ?? null);

  const onKeyDown = (e: KeyboardEvent) => {
    // Ağaç tuşları yalnızca öğelerindir; öğenin içindeki düğme ("Yeniden dene") kendi tuşlarını alır.
    if ((e.target as Element).getAttribute("role") !== "treeitem") return;
    const i = visible.findIndex((v) => v.key === current);
    if (i < 0) return;
    const v = visible[i];
    const go = (j: number) => visible[j] && setFocusKey(visible[j].key);
    switch (e.key) {
      case "ArrowDown":
        go(i + 1);
        break;
      case "ArrowUp":
        go(i - 1);
        break;
      case "Home":
        go(0);
        break;
      case "End":
        go(visible.length - 1);
        break;
      case "ArrowRight":
        if (v.expandable && !v.open) v.toggle();
        else if (v.open && visible[i + 1]?.level === v.level + 1) go(i + 1);
        break;
      case "ArrowLeft":
        if (v.open) v.toggle();
        else if (v.level > 1) setFocusKey(parentKey(v.key));
        break;
      case "Enter":
      case " ":
        if (v.expandable) v.toggle();
        break;
      default:
        return;
    }
    e.preventDefault();
  };

  const renderLevel = (level: PeriodLevel) => {
    if (level.status === "loading")
      return (
        <li role="none" className="dasha-pending">
          {t("panel.loading")}
        </li>
      );
    if (level.status === "error")
      return (
        <li role="none">
          <PanelError error={level.error} onRetry={level.retry} />
        </li>
      );
    return level.nodes.map((node, i) => (
      <li
        key={node.key}
        ref={(el) => {
          if (el) items.current.set(node.key, el);
          else items.current.delete(node.key);
        }}
        role="treeitem"
        aria-level={node.level}
        aria-setsize={level.nodes.length}
        aria-posinset={i + 1}
        aria-expanded={node.expandable ? node.open : undefined}
        aria-label={node.label}
        tabIndex={node.key === current ? 0 : -1}
        className="dasha-item"
        onFocus={(e) => e.target === e.currentTarget && setFocusKey(node.key)}
      >
        {/* Tıklama satırdadır: iç içe öğelerin (ve hata düğmesinin) tıklaması üst düğümü kapatmaz. */}
        <span
          className="dasha-row"
          style={{ paddingInlineStart: `${(node.level - 1) * 14}px` }}
          onClick={() => {
            setFocusKey(node.key);
            if (node.expandable) node.toggle();
          }}
        >
          <span className="dasha-twisty" aria-hidden="true">
            {node.expandable ? (node.open ? "▾" : "▸") : ""}
          </span>
          <span className="dasha-lord">{node.name}</span>
          <span className="dasha-dates">{node.dates}</span>
        </span>
        {node.open && node.kids && <ul role="group">{renderLevel(node.kids)}</ul>}
      </li>
    ));
  };

  return (
    <ul ref={treeRef} role="tree" aria-label={label} className="dasha-tree" onKeyDown={onKeyDown}>
      {renderLevel(roots)}
    </ul>
  );
}
````

**Yeni dosya:** `app/ui/dasha/system.tsx`

````tsx
// Dasa sistemi seçimi (ileri modüller spec'i §6): dasa ağacı ve dasa şeridi panellerinin
// `system` ayarı. Ayar panelindir ve düzenle kaydedilir; tanınmayan değer Vimshottari sayılır.
import { useId } from "react";
import { useTranslation } from "react-i18next";

export const DASHA_SYSTEMS = ["vimshottari", "chara"] as const;
export type DashaSystem = (typeof DASHA_SYSTEMS)[number];

export function dashaSystem(settings: Record<string, unknown>): DashaSystem {
  return settings.system === "chara" ? "chara" : "vimshottari";
}

/** "Sistem" seçicisi: görünür etiketli `select`; panelin araç çubuğuna konur. */
export function SystemSelect({ value, onChange }: { value: DashaSystem; onChange(system: DashaSystem): void }) {
  const { t } = useTranslation();
  const id = useId();
  return (
    <>
      <label htmlFor={`${id}-system`}>{t("dasha.system")}</label>
      <select id={`${id}-system`} value={value} onChange={(e) => onChange(dashaSystem({ system: e.target.value }))}>
        {DASHA_SYSTEMS.map((s) => (
          <option key={s} value={s}>
            {t(`dasha.systems.${s}`)}
          </option>
        ))}
      </select>
    </>
  );
}
````

- [ ] **Step 4: Chara dasha yardımcıları ve ağacı**

**Yeni dosya:** `app/ui/dasha/chara.ts`

````ts
// Chara dasha (ileri modüller spec'i §6) için arayüz yardımcıları. Dönemler, yön ve yıllar
// Rust'tan hazır gelir (`compute_chara_dasha`, `chara_dasha_children`, `chara_dasha_at`);
// burada hesap yoktur (C4): yalnızca istek, gruplama ve adlandırma.
import type { TFunction } from "i18next";
import { api } from "../api";
import type { CharaDasha, OpenedClient, Sign, SignDashaPath, SignDashaPeriod } from "../bindings";
import { signIndex } from "../chart/adapter";
import { chartInput, inputKey } from "../modules/input";
import { useModule, type ModuleState } from "../modules/useModule";

/** Rust `MAX_SIGN_DASHA_LEVEL`. */
export const MAX_CHARA_LEVEL = 3;
/** `CharaDasha.periods`'un içerdiği en derin seviye; 3. seviye istenerek yüklenir. */
export const CHARA_PRELOADED_LEVEL = 2;

/** Dönemin ağaç anahtarı: `"<çevrim>:<burç>,<burç>…"` (bkz. `PeriodTree`). */
export const charaKey = (cycle: number, signs: readonly Sign[]) => `${cycle}:${signs.join(",")}`;

/** `path` yolunun doğrudan alt dönemleri (boş yol: o çevrimin dashaları), sırası korunur. */
export function charaChildrenOf(periods: readonly SignDashaPeriod[], cycle: number, path: readonly Sign[]): SignDashaPeriod[] {
  return periods.filter((p) => p.cycle === cycle && p.level === path.length + 1 && path.every((s, i) => p.signs[i] === s));
}

/** Dashalar (1. seviye), iki çevrim birlikte, zaman sırasıyla. */
export function charaDashas(periods: readonly SignDashaPeriod[]): SignDashaPeriod[] {
  return periods.filter((p) => p.level === 1);
}

/** `node`, açık yolun bir öneki mi (o düğüm açık mı)? */
export function isCharaOpen(open: SignDashaPath, cycle: number, signs: readonly Sign[]): boolean {
  return open.cycle === cycle && signs.length <= open.signs.length && signs.every((s, i) => open.signs[i] === s);
}

/** Dönemin adı: burç kısaltmaları; ikinci çevrim "2." önekiyle ayrılır. */
export function charaName(t: TFunction, cycle: number, signs: readonly Sign[]): string {
  const name = signs.map((s) => t(`sign.${signIndex(s)}`)).join("–");
  return cycle === 2 ? t("dasha.chara.secondCycle", { name }) : name;
}

/** İlk çevrimin burçları ve yılları, dasha sırasıyla (başlık satırı için). */
export function charaYears(dasha: CharaDasha): { sign: Sign; years: number }[] {
  return charaDashas(dasha.periods)
    .filter((p) => p.cycle === 1)
    .map((p, i) => ({ sign: p.signs[0], years: dasha.years[i] }));
}

/** `enabled === false`: sistem chara değil, komut çağrılmaz. `data === null`: saat bilinmiyor. */
export function useCharaDasha(opened: OpenedClient | null, enabled = true): ModuleState<CharaDasha | null> {
  return useModule(opened !== null && enabled ? inputKey(opened) : null, () => api.computeCharaDasha(chartInput(opened as OpenedClient)));
}

/** Bir dönemin istenerek yüklenen alt dönemleri; `path === null` ise komut çağrılmaz. */
export function useCharaChildren(opened: OpenedClient | null, cycle: number, path: readonly Sign[] | null): ModuleState<SignDashaPeriod[]> {
  const key = opened !== null && path !== null ? `${inputKey(opened)}|${charaKey(cycle, path)}` : null;
  return useModule(key, () => api.charaDashaChildren(chartInput(opened as OpenedClient), [...(path ?? [])], cycle));
}
````

**Yeni dosya:** `app/ui/dasha/CharaDashaTree.tsx`

````tsx
// Chara dasha ağacı (ileri modüller spec'i §6): dashalar burçtur. Yön, yıllar ve iki seviye
// dönem `compute_chara_dasha`'dan gelir ve panel açıkken istenir (K1); 3. seviye
// `chara_dasha_children` ile istenerek yüklenir. Burada hesap yoktur (C4). Açık yol
// `ViewState.chara_expanded_path`'tir (çevrim ve burç yolu). İkinci çevrimin satırları "2."
// önekiyle ayrılır. Doğum saati bilinmiyorsa komut `null` döner ve ağaç yerine bir satır
// açıklama çıkar: sıra lagnaya bağlıdır.
import { useTranslation } from "react-i18next";
import type { CharaDasha, OpenedClient, Sign, SignDashaPeriod } from "../bindings";
import { signIndex } from "../chart/adapter";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "../panels/states";
import { useViewStore } from "../view/store";
import {
  CHARA_PRELOADED_LEVEL,
  charaChildrenOf,
  charaDashas,
  charaKey,
  charaName,
  charaYears,
  isCharaOpen,
  MAX_CHARA_LEVEL,
  useCharaChildren,
  useCharaDasha,
} from "./chara";
import { PeriodTree, type PeriodLevel } from "./PeriodTree";
import { formatJdDate } from "./tree";

export function CharaDashaTree({ opened }: { opened: OpenedClient }) {
  const { t } = useTranslation();
  const module = useCharaDasha(opened);
  const { model } = useViewStore();
  const open = model.charaPath;
  // Tek açık yol: istenerek yüklenen tek seviye, açık yolun ikinci düğümünün alt dönemleridir.
  // Yol bu haritada yoksa (ör. o burç ikinci çevrimde atlanıyor) istek yapılmaz.
  const wanted = open.signs.slice(0, CHARA_PRELOADED_LEVEL);
  const known = (module.data?.periods ?? []).some((p) => p.cycle === open.cycle && p.level === CHARA_PRELOADED_LEVEL && wanted.every((s, i) => p.signs[i] === s));
  const lazyPath = wanted.length === CHARA_PRELOADED_LEVEL && known ? wanted : null;
  const lazy = useCharaChildren(opened, open.cycle, lazyPath);
  const phase = usePanelLoadPhase(module.loading || lazy.loading);
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (module.loading) return phase === "skeleton" ? <PanelSkeleton /> : null;
  if (module.data === null) return <p className="panel-notice">{t("dasha.chara.timeUnknown")}</p>;
  return <Tree dasha={module.data} opened={opened} lazy={lazy} />;
}

function Tree({ dasha, opened, lazy }: { dasha: CharaDasha; opened: OpenedClient; lazy: ReturnType<typeof useCharaChildren> }) {
  const { t } = useTranslation();
  const { model, setCharaPath } = useViewStore();
  const open = model.charaPath;
  const offset = opened.chart_record.resolved.offset_secs;
  const dates = (p: SignDashaPeriod) => `${formatJdDate(p.start_jd_ut, offset)} – ${formatJdDate(p.end_jd_ut, offset)}`;
  const toggle = (p: SignDashaPeriod) => setCharaPath({ cycle: p.cycle, signs: isCharaOpen(open, p.cycle, p.signs) ? p.signs.slice(0, -1) : p.signs });

  const nodes = (periods: readonly SignDashaPeriod[]): PeriodLevel => ({
    status: "ok",
    nodes: periods.map((p) => {
      const expandable = p.has_children && p.level < MAX_CHARA_LEVEL;
      const o = expandable && isCharaOpen(open, p.cycle, p.signs);
      return {
        key: charaKey(p.cycle, p.signs),
        level: p.level,
        name: charaName(t, p.level === 1 ? p.cycle : 1, p.signs.slice(-1)),
        label: `${charaName(t, p.cycle, p.signs)}, ${dates(p)}`,
        dates: dates(p),
        expandable,
        open: o,
        kids: o ? level(p.cycle, p.signs) : undefined,
        toggle: () => toggle(p),
      };
    }),
  });

  // Yüklenen 3. seviye de süzgeçten geçer: yalnızca istenen yolun bir alt seviyesi kabul
  // edilir. Boş yanıt hatadır: her dönemin on iki alt dönemi vardır.
  const level = (cycle: number, path: readonly Sign[]): PeriodLevel => {
    if (path.length < CHARA_PRELOADED_LEVEL) return nodes(charaChildrenOf(dasha.periods, cycle, path));
    if (lazy.error) return { status: "error", error: lazy.error, retry: lazy.retry };
    if (lazy.data === null) return { status: "loading" };
    const kids = charaChildrenOf(lazy.data, cycle, path);
    return kids.length > 0 ? nodes(kids) : { status: "error", error: { code: "dasha_empty", params: {} }, retry: lazy.retry };
  };

  const years = charaYears(dasha)
    .map(({ sign, years: y }) => `${t(`sign.${signIndex(sign)}`)} ${y}`)
    .join(" · ");
  return (
    <>
      <p className="dasha-chara-summary">
        <span>{t("dasha.chara.direction", { direction: t(dasha.forward ? "dasha.chara.forward" : "dasha.chara.backward") })}</span>
        <span>{t("dasha.chara.years", { list: years })}</span>
      </p>
      <PeriodTree label={t("panel.dasha_tree")} roots={nodes(charaDashas(dasha.periods))} />
      <p className="panel-message">{t("dasha.chara.method", { method: dasha.method })}</p>
    </>
  );
}
````

- [ ] **Step 5: Paneli ortak ağaca taşı; stil ve metinler**

`DashaTreePanel` araç çubuğunu (sistem seçicisi) çizer; Vimshottari ağacı aynı dönem listelerini `PeriodTree` düğümlerine çevirir. Odak, sekme durağı ve tuşlar `PeriodTree`'dedir.

````diff
diff --git a/app/ui/dasha/DashaTreePanel.tsx b/app/ui/dasha/DashaTreePanel.tsx
index 69bc755..59e4324 100644
--- a/app/ui/dasha/DashaTreePanel.tsx
+++ b/app/ui/dasha/DashaTreePanel.tsx
@@ -1,46 +1,49 @@
-import { useEffect, useRef, useState, type KeyboardEvent } from "react";
 import { useTranslation } from "react-i18next";
-import type { DashaPeriod, Graha } from "../bindings";
+import type { DashaPeriod, Graha, OpenedClient } from "../bindings";
 import { timeUnknown } from "../session/birth";
-import { PanelError, usePanelLoadPhase } from "../panels/states";
+import { usePanelLoadPhase } from "../panels/states";
 import type { PanelProps } from "../panels/types";
 import { useSession } from "../session/store";
-import { useViewStore } from "../view/store";
+import { useLayoutStore, useViewStore } from "../view/store";
+import { CharaDashaTree } from "./CharaDashaTree";
 import "./dasha.css";
+import { PeriodTree, type PeriodLevel } from "./PeriodTree";
+import { dashaSystem, SystemSelect } from "./system";
 import { childrenOf, formatJdDate, isOpen, MAX_DASHA_LEVEL, PRELOADED_LEVEL } from "./tree";
 import { pathKey, useLazyDasha, type LazyEntry } from "./useLazyDasha";
 
-interface Visible {
-  period: DashaPeriod;
-  expandable: boolean;
-  open: boolean;
+/**
+ * Dasha ağacı (spec §3.3). `system` ayarı Vimshottari ile chara dasha arasında seçer (ileri
+ * modüller spec'i §6). Tek açık yol: bir düğümü açmak kardeşlerini kapatır; yol haritanın
+ * `ViewState`'indedir (sistem başına ayrı) ve haritayla kaydedilir. Klavye: WAI-ARIA ağaç deseni
+ * (`PeriodTree`).
+ */
+export function DashaTreePanel({ panel }: PanelProps) {
+  const { t } = useTranslation();
+  const { opened } = useSession();
+  const { setPanelSettings } = useLayoutStore();
+  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
+  const system = dashaSystem(panel.settings);
+  return (
+    <div className="dasha-panel">
+      <div className="module-toolbar">
+        <SystemSelect value={system} onChange={(next) => setPanelSettings(panel.id, { system: next })} />
+      </div>
+      {system === "chara" ? <CharaDashaTree opened={opened} /> : <VimshottariTree opened={opened} />}
+    </div>
+  );
 }
 
 /**
- * Dasha ağacı (spec §3.3, Vimshottari). Tek açık yol: bir düğümü açmak kardeşlerini kapatır;
- * yol haritanın `ViewState.dasha_expanded_path`'idir ve haritayla kaydedilir. 1–3. seviye
- * haritayla gelir, 4–5. seviye istenerek yüklenir. Klavye: WAI-ARIA ağaç deseni.
+ * Vimshottari ağacı: 1–3. seviye haritayla gelir, 4–5. seviye istenerek yüklenir. Açık yol
+ * `ViewState.dasha_expanded_path`'tir.
  */
-export function DashaTreePanel(_: PanelProps) {
+function VimshottariTree({ opened }: { opened: OpenedClient }) {
   const { t } = useTranslation();
-  const { opened } = useSession();
   const { model, setDashaPath } = useViewStore();
   const open = model.dashaPath;
-  const chartId = opened?.chart_record.id ?? 0;
-  const seed = opened?.chart.dasha.seed ?? { birth_jd_ut: null, moon_sidereal_lon: null, year_days: null };
-  const lazy = useLazyDasha(chartId, seed, opened ? open : []);
-  usePanelLoadPhase(opened !== null && lazy.loading);
-  const [focusKey, setFocusKey] = useState<string | null>(null);
-  const items = useRef(new Map<string, HTMLLIElement>());
-  const treeRef = useRef<HTMLUListElement>(null);
-
-  // Odak klavyeyle taşındıysa yeni öğeye geçer (fare ve ilk çizim odağı çalmaz).
-  useEffect(() => {
-    if (focusKey === null || !treeRef.current?.contains(document.activeElement)) return;
-    items.current.get(focusKey)?.focus();
-  }, [focusKey]);
-
-  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
+  const lazy = useLazyDasha(opened.chart_record.id, opened.chart.dasha.seed, open);
+  usePanelLoadPhase(lazy.loading);
   const { periods } = opened.chart.dasha;
   const offset = opened.chart_record.resolved.offset_secs;
 
@@ -55,148 +58,42 @@ export function DashaTreePanel(_: PanelProps) {
     return kids.length > 0 ? { status: "ok", periods: kids } : { status: "error", error: { code: "dasha_empty", params: {} } };
   };
 
-  /** Yüklenmiş alt dönemler; yükleniyorsa ya da hatalıysa `null`. */
-  const loaded = (path: readonly Graha[]): readonly DashaPeriod[] | null => {
-    const list = childList(path);
-    if (Array.isArray(list)) return list;
-    return list?.status === "ok" ? list.periods : null;
-  };
-
-  // Görünür öğeler, ekrandaki sırayla (klavye gezintisi için).
-  const visible: Visible[] = [];
-  const walk = (path: readonly Graha[]) => {
-    for (const p of loaded(path) ?? []) {
-      const expandable = p.has_children && p.level < MAX_DASHA_LEVEL;
-      const o = expandable && isOpen(open, p.lords);
-      visible.push({ period: p, expandable, open: o });
-      if (o) walk(p.lords);
-    }
-  };
-  walk([]);
-  // Sekme durağı her zaman görünür bir öğedir: odaklı öğe ağaçtan çıktıysa (başka harita, kapanan
-  // yol, silinen tembel seviye) görünür en yakın atası, o da yoksa ilk öğe.
-  const visibleKeys = new Set(visible.map((v) => pathKey(v.period.lords)));
-  const focusPath = focusKey === null ? [] : focusKey.split(",");
-  while (focusPath.length > 0 && !visibleKeys.has(focusPath.join(","))) focusPath.pop();
-  const current = focusPath.length > 0 ? focusPath.join(",") : visible[0] ? pathKey(visible[0].period.lords) : null;
-
+  const dates = (p: DashaPeriod) => `${formatJdDate(p.start_jd_ut, offset)} – ${formatJdDate(p.end_jd_ut, offset)}`;
   const toggle = (p: DashaPeriod) => setDashaPath(isOpen(open, p.lords) ? p.lords.slice(0, -1) : p.lords);
 
-  const onKeyDown = (e: KeyboardEvent) => {
-    // Ağaç tuşları yalnızca öğelerindir; öğenin içindeki düğme ("Yeniden dene") kendi tuşlarını alır.
-    if ((e.target as Element).getAttribute("role") !== "treeitem") return;
-    const i = visible.findIndex((v) => pathKey(v.period.lords) === current);
-    if (i < 0) return;
-    const v = visible[i];
-    const go = (j: number) => visible[j] && setFocusKey(pathKey(visible[j].period.lords));
-    switch (e.key) {
-      case "ArrowDown":
-        go(i + 1);
-        break;
-      case "ArrowUp":
-        go(i - 1);
-        break;
-      case "Home":
-        go(0);
-        break;
-      case "End":
-        go(visible.length - 1);
-        break;
-      case "ArrowRight":
-        if (v.expandable && !v.open) toggle(v.period);
-        else if (v.open && visible[i + 1]?.period.level === v.period.level + 1) go(i + 1);
-        break;
-      case "ArrowLeft":
-        if (v.open) toggle(v.period);
-        else if (v.period.level > 1) setFocusKey(pathKey(v.period.lords.slice(0, -1)));
-        break;
-      case "Enter":
-      case " ":
-        if (v.expandable) toggle(v.period);
-        break;
-      default:
-        return;
-    }
-    e.preventDefault();
-  };
-
-  const label = (p: DashaPeriod) =>
-    `${p.lords.map((g) => t(`grahaName.${g}`)).join("–")}, ${formatJdDate(p.start_jd_ut, offset)} – ${formatJdDate(p.end_jd_ut, offset)}`;
-
-  const renderLevel = (path: readonly Graha[]) => {
+  const level = (path: readonly Graha[]): PeriodLevel => {
     const list = childList(path);
-    if (list === undefined)
-      return (
-        <li role="none" className="dasha-pending">
-          {t("panel.loading")}
-        </li>
-      );
-    if (!Array.isArray(list)) {
-      if (list.status === "error")
-        return (
-          <li role="none">
-            <PanelError error={list.error} onRetry={() => lazy.retry(path)} />
-          </li>
-        );
-      return renderItems(list.periods);
-    }
-    return renderItems(list);
+    if (list === undefined) return { status: "loading" };
+    if (!Array.isArray(list) && list.status === "error") return { status: "error", error: list.error, retry: () => lazy.retry(path) };
+    const loaded = Array.isArray(list) ? list : list.periods;
+    return {
+      status: "ok",
+      nodes: loaded.map((p) => {
+        const expandable = p.has_children && p.level < MAX_DASHA_LEVEL;
+        const o = expandable && isOpen(open, p.lords);
+        return {
+          key: pathKey(p.lords),
+          level: p.level,
+          name: t(`grahaName.${p.lords[p.lords.length - 1]}`),
+          label: `${p.lords.map((g) => t(`grahaName.${g}`)).join("–")}, ${dates(p)}`,
+          dates: dates(p),
+          expandable,
+          open: o,
+          kids: o ? level(p.lords) : undefined,
+          toggle: () => toggle(p),
+        };
+      }),
+    };
   };
 
-  const renderItems = (list: readonly DashaPeriod[]) =>
-    list.map((p, i) => {
-      const key = pathKey(p.lords);
-      const expandable = p.has_children && p.level < MAX_DASHA_LEVEL;
-      const o = expandable && isOpen(open, p.lords);
-      return (
-        <li
-          key={key}
-          ref={(el) => {
-            if (el) items.current.set(key, el);
-            else items.current.delete(key);
-          }}
-          role="treeitem"
-          aria-level={p.level}
-          aria-setsize={list.length}
-          aria-posinset={i + 1}
-          aria-expanded={expandable ? o : undefined}
-          aria-label={label(p)}
-          tabIndex={key === current ? 0 : -1}
-          className="dasha-item"
-          onFocus={(e) => e.target === e.currentTarget && setFocusKey(key)}
-        >
-          {/* Tıklama satırdadır: iç içe öğelerin (ve hata düğmesinin) tıklaması üst düğümü kapatmaz. */}
-          <span
-            className="dasha-row"
-            style={{ paddingInlineStart: `${(p.level - 1) * 14}px` }}
-            onClick={() => {
-              setFocusKey(key);
-              if (expandable) toggle(p);
-            }}
-          >
-            <span className="dasha-twisty" aria-hidden="true">
-              {expandable ? (o ? "▾" : "▸") : ""}
-            </span>
-            <span className="dasha-lord">{t(`grahaName.${p.lords[p.lords.length - 1]}`)}</span>
-            <span className="dasha-dates">
-              {formatJdDate(p.start_jd_ut, offset)} – {formatJdDate(p.end_jd_ut, offset)}
-            </span>
-          </span>
-          {o && <ul role="group">{renderLevel(p.lords)}</ul>}
-        </li>
-      );
-    });
-
   return (
-    <div className="dasha-panel">
+    <>
       {timeUnknown(opened) && (
         <p className="panel-notice" role="note">
           {t("dasha.approx")}
         </p>
       )}
-      <ul ref={treeRef} role="tree" aria-label={t("panel.dasha_tree")} className="dasha-tree" onKeyDown={onKeyDown}>
-        {renderLevel([])}
-      </ul>
-    </div>
+      <PeriodTree label={t("panel.dasha_tree")} roots={level([])} />
+    </>
   );
 }
diff --git a/app/ui/dasha/dasha.css b/app/ui/dasha/dasha.css
index 51da500..5401c73 100644
--- a/app/ui/dasha/dasha.css
+++ b/app/ui/dasha/dasha.css
@@ -39,6 +39,16 @@
   padding-inline-start: 24px;
   color: var(--text-2);
 }
+/* Chara dasha başlık satırı: yön ve on iki burcun yılları. */
+.dasha-chara-summary {
+  display: flex;
+  flex-wrap: wrap;
+  gap: 2px 12px;
+  margin: 0;
+  font-size: 12px;
+  color: var(--text-2);
+  font-variant-numeric: tabular-nums;
+}
 
 .dasha-strip-panel {
   display: grid;
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 830d75d..a3cabcc 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -347,6 +347,17 @@ export const en: Messages = {
   },
   dasha: {
     approx: "Birth time unknown: dasha dates are approximate (≈).",
+    system: "System",
+    systems: { vimshottari: "Vimshottari", chara: "Chara dasha" },
+    chara: {
+      direction: "Direction: {{direction}}",
+      forward: "forward",
+      backward: "reverse",
+      years: "Years: {{list}}",
+      secondCycle: "2. {{name}}",
+      timeUnknown: "Birth time unknown: chara dasha depends on the lagna and is not computed.",
+      method: "Method: {{method}}",
+    },
     strip: {
       level: "Level",
       levels: { maha: "Mahadasha", antar: "Antardasha", pratyantar: "Pratyantardasha" },
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 17300a7..1cd6e1d 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -347,6 +347,17 @@ export const fr: Messages = {
   },
   dasha: {
     approx: "Heure de naissance inconnue : les dates de dasha sont approximatives (≈).",
+    system: "Système",
+    systems: { vimshottari: "Vimshottari", chara: "Chara dasha" },
+    chara: {
+      direction: "Sens : {{direction}}",
+      forward: "direct",
+      backward: "inverse",
+      years: "Années : {{list}}",
+      secondCycle: "2. {{name}}",
+      timeUnknown: "Heure de naissance inconnue : la chara dasha dépend du lagna et n'est pas calculée.",
+      method: "Méthode : {{method}}",
+    },
     strip: {
       level: "Niveau",
       levels: { maha: "Mahadasha", antar: "Antardasha", pratyantar: "Pratyantardasha" },
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 8b2a4c7..137e274 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -346,6 +346,17 @@ export const tr = {
   },
   dasha: {
     approx: "Doğum saati bilinmiyor: dasa tarihleri yaklaşıktır (≈).",
+    system: "Sistem",
+    systems: { vimshottari: "Vimshottari", chara: "Chara dasha" },
+    chara: {
+      direction: "Yön: {{direction}}",
+      forward: "ileri",
+      backward: "geri",
+      years: "Yıllar: {{list}}",
+      secondCycle: "2. {{name}}",
+      timeUnknown: "Doğum saati bilinmiyor: chara dasha lagnaya bağlıdır ve hesaplanmaz.",
+      method: "Yöntem: {{method}}",
+    },
     strip: {
       level: "Seviye",
       levels: { maha: "Mahadasa", antar: "Antardasa", pratyantar: "Pratyantardasa" },
````

- [ ] **Step 6: Testleri koş**

```bash
(cd app && npx vitest run ui/dasha 2>&1 | grep -E "Test Files|Tests ")
```
Expected: `Test Files  4 passed (4)`, `Tests  32 passed (32)` (yeni dosya 10 test; `DashaTreePanel.test.tsx`'in 14, `tree.test.ts`'in 3 ve `DashaStripPanel.test.tsx`'in 5 testi değişmeden geçer).

- [ ] **Step 7: Kapılar**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
git diff -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
```
Expected: `69 passed (69)`, `638 passed (638)`; `✓ built`; son komut çıktısız (C9).

- [ ] **Step 8: Commit**

```bash
git add app/ui
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): dasha tree system selector and the chara dasha tree on a shared period tree"
```
Expected: iki `grep` çıktısız.

---

### Task 5: Dasa şeridi: sistem seçicisi ve chara dasha şeridi

Spec §6 "Arayüz"; D14, D15. Şerit sistem seçicisini kazanır; chara şeridi güncel dönemi `chara_dasha_at`'ten okur ve tıklama transit anını dönemin başlangıcına taşır. Vimshottari şeridinin mevcut beş testi değişmeden geçmelidir.

**Files:**
- Create: `app/ui/dasha/CharaDashaStrip.tsx`, `app/ui/dasha/CharaDashaStrip.test.tsx`
- Modify: `app/ui/dasha/DashaStripPanel.tsx`, `app/ui/dasha/system.tsx`, `app/ui/dasha/dasha.css`, `app/ui/i18n/{tr,en,fr}.ts`, `app/src-tauri/tests/ui_fixture.rs`, `app/ui/test/fixture.ts`
- Üretilir: `app/ui/test/fixtures/delhi.json`
- Test: `app/ui/dasha/CharaDashaStrip.test.tsx` (yeni), `app/ui/dasha/DashaStripPanel.test.tsx` (değişmez)

**Interfaces:**
- Consumes: `api.charaDashaAt`, `api.charaDashaChildren`, `api.computeCharaDasha` (Task 2); `useCharaDasha`, `useCharaChildren`, `charaDashas`, `charaChildrenOf`, `charaKey`, `charaName`, `SystemSelect`, `dashaSystem` (Task 4); `sameCharaPath` (Task 3); `useTransit` (`view.snapshot.jd_ut`, `setAt`); `useModule`.
- Produces:
  - `dasha/system.tsx`: `STRIP_LEVELS`, `type StripLevel`, `stripLevel(settings)`, `PARENT_DEPTH` (şerit panelinden buraya taşınır; `CharaDashaStrip` ile panel arasında içe aktarma döngüsü olmasın)
  - `dasha/CharaDashaStrip.tsx`: `CharaDashaStrip({ opened, level })`
  - Fikstür: `UiFixture.chara_dasha_at_children: SignDashaPeriod[]` (transit anındaki alt dönemin 3. seviyesi)
  - i18n: `dasha.chara.outside`
  - Panel ayarı: `dasha_strip` `settings.system`.

- [ ] **Step 1: Başarısız testi yaz**

Testler transit taklidini istenen ana göre yanıtlatır (`{ Jd }` verilince yanıtın `snapshot.jd_ut`'u o andır): böylece bir döneme tıklamanın anın yolunu yeniden sordurduğu görülür.

**Yeni dosya:** `app/ui/dasha/CharaDashaStrip.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn(), computeCharaDasha: vi.fn(), charaDashaChildren: vi.fn(), charaDashaAt: vi.fn() } }));

import { api, type Result } from "../api";
import type { CharaDasha, OpenedClient, TransitAt } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { DashaStripPanel } from "./DashaStripPanel";
import { stripLevel } from "./system";

function renderStrip(settings: Record<string, unknown> = { system: "chara" }, opened: OpenedClient = fixture().opened) {
  const panel: Panel = { id: "ds", kind: "dasha_strip", x: 0, y: 0, w: 12, h: 4, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Dasa şeridi paneli">
      <DashaStripPanel panel={panel} />
    </section>,
    { layout, opened },
  );
}

const list = () => screen.findByRole("list", { name: /Dasa şeridi/ });
const buttons = async () => within(await list()).getAllByRole("button");
const names = (items: HTMLElement[]) => items.map((b) => b.querySelector(".dasha-strip-name")!.textContent);
const current = (items: HTMLElement[]) => items.filter((b) => b.getAttribute("aria-current") === "true");

beforeEach(() => {
  // Transit anı: istenen an bir JD ise yanıt o anı taşır; değilse fikstürün "şimdi"si.
  vi.mocked(api.transitsAt).mockImplementation(async (at: TransitAt) => {
    const view = fixture().transit;
    if (typeof at === "object" && "Jd" in at) view.snapshot.jd_ut = at.Jd ?? null;
    return { status: "ok", data: view };
  });
  vi.mocked(api.computeCharaDasha).mockImplementation(async () => ({ status: "ok", data: fixture().chara_dasha }));
  vi.mocked(api.charaDashaAt).mockImplementation(async () => ({ status: "ok", data: fixture().chara_dasha_at }));
  vi.mocked(api.charaDashaChildren).mockImplementation(async () => ({ status: "ok", data: fixture().chara_dasha_at_children }));
});
afterEach(() => {
  for (const f of [api.transitsAt, api.computeCharaDasha, api.charaDashaAt, api.charaDashaChildren]) vi.mocked(f).mockReset();
});

test("Vimshottari is the default system; chara dasha is requested only once chosen and both settings are saved", async () => {
  const { persistence } = renderStrip({ level: "maha" });
  expect(await buttons()).toHaveLength(9);
  expect(screen.getByRole("combobox", { name: "Sistem" })).toHaveDisplayValue("Vimshottari");
  expect(api.computeCharaDasha).not.toHaveBeenCalled();
  expect(api.charaDashaAt).not.toHaveBeenCalled();
  fireEvent.change(screen.getByRole("combobox", { name: "Sistem" }), { target: { value: "chara" } });
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  expect(persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ level: "maha", system: "chara" });
  expect([stripLevel({}), stripLevel({ level: "prana" }), stripLevel({ level: "maha" })]).toEqual(["antar", "antar", "maha"]);
});

test("chara, antar (default): the twelve sub-periods of the dasha at the transit moment, the current one marked", async () => {
  renderStrip();
  const items = await buttons();
  const opened = fixture().opened;
  const input = { jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true };
  expect(api.computeCharaDasha).toHaveBeenCalledWith(input);
  // Güncel dönem Rust'tan gelir: transit anı (`snapshot.jd_ut`) ile sorulur.
  expect(api.charaDashaAt).toHaveBeenCalledTimes(1);
  expect(api.charaDashaAt).toHaveBeenCalledWith(input, fixture().transit.snapshot.jd_ut);
  // Fikstür: an ilk çevrimin Balık dashasında, Başak alt döneminde. Yön geri: Kova … Balık.
  expect(fixture().chara_dasha_at).toEqual({ cycle: 1, signs: ["Pisces", "Virgo", "Aries"] });
  expect(names(items)).toEqual(["Bal–Kov", "Bal–Oğl", "Bal–Yay", "Bal–Akr", "Bal–Ter", "Bal–Baş", "Bal–Asl", "Bal–Yen", "Bal–İki", "Bal–Boğ", "Bal–Koç", "Bal–Bal"]);
  expect(current(items)).toHaveLength(1);
  expect(current(items)[0]).toHaveAttribute("data-sign", "Virgo");
  expect(current(items)[0]).toHaveTextContent("transit anında");
  expect(api.charaDashaChildren).not.toHaveBeenCalled();
});

test("chara, maha: both cycles in one strip, the second one prefixed", async () => {
  renderStrip({ system: "chara", level: "maha" });
  const items = await buttons();
  expect(names(items)).toEqual([
    ...["İki", "Boğ", "Koç", "Bal", "Kov", "Oğl", "Yay", "Akr", "Ter", "Baş", "Asl", "Yen"],
    ...["2. İki", "2. Boğ", "2. Koç", "2. Bal", "2. Kov", "2. Oğl", "2. Yay", "2. Akr"],
  ]);
  // İlk çevrimin Balık'ı günceldir; ikinci çevrimin Balık'ı değildir.
  expect(current(items).map((b) => [b.getAttribute("data-sign"), b.getAttribute("data-cycle")])).toEqual([["Pisces", "1"]]);
  expect(items[0]).toHaveTextContent("2000-01-01");
});

test("chara, pratyantar: the third level is asked for the sub-period at the transit moment; choosing a period moves the transit moment", async () => {
  renderStrip({ system: "chara", level: "pratyantar" });
  const items = await buttons();
  expect(api.charaDashaChildren).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.charaDashaChildren).mock.calls[0].slice(1)).toEqual([["Pisces", "Virgo"], 1]);
  expect(items).toHaveLength(12);
  expect(names(items)[0]).toBe("Bal–Baş–Asl");
  expect(names(items)[11]).toBe("Bal–Baş–Baş");
  expect(current(items).map((b) => b.getAttribute("data-sign"))).toEqual(["Aries"]);
  // Tıklama transit anını dönemin başlangıcına taşır; yeni anın yolu yeniden sorulur.
  const start = fixture().chara_dasha_at_children[0].start_jd_ut;
  fireEvent.click(items[0]);
  await waitFor(() => expect(vi.mocked(api.transitsAt).mock.calls.at(-1)?.[0]).toEqual({ Jd: start }));
  await waitFor(() => expect(vi.mocked(api.charaDashaAt).mock.calls.at(-1)?.[1]).toBe(start));
  expect(api.charaDashaAt).toHaveBeenCalledTimes(2);
});

test("the current period follows the path Rust returns for the new moment", async () => {
  renderStrip();
  const items = await buttons();
  // Yeni an için Rust başka bir dasha döner: şerit o dashanın alt dönemlerine geçer.
  vi.mocked(api.charaDashaAt).mockImplementation(async () => ({ status: "ok", data: { cycle: 2, signs: ["Gemini", "Aries", "Pisces"] } }));
  fireEvent.click(items[0]);
  await waitFor(async () => expect(names(await buttons())[0]).toBe("2. İki–Boğ"));
  const next = await buttons();
  expect(names(next)).toEqual(["Boğ", "Koç", "Bal", "Kov", "Oğl", "Yay", "Akr", "Ter", "Baş", "Asl", "Yen", "İki"].map((s) => `2. İki–${s}`));
  expect(current(next).map((b) => [b.getAttribute("data-sign"), b.getAttribute("data-cycle")])).toEqual([["Aries", "2"]]);
});

test("a moment outside the produced periods: a message instead of the strip", async () => {
  vi.mocked(api.charaDashaAt).mockImplementation(async () => ({ status: "ok", data: null }));
  renderStrip();
  expect(await screen.findByText("Transit anı chara dasha dönemlerinin dışında.")).toBeInTheDocument();
  expect(screen.queryByRole("list")).toBeNull();
});

test("unknown birth time: Rust returns no dasha, the panel says why and asks for no path", async () => {
  vi.mocked(api.computeCharaDasha).mockImplementation(async () => ({ status: "ok", data: null }));
  renderStrip({ system: "chara" }, unknownTimeOpened());
  expect(await screen.findByText("Doğum saati bilinmiyor: chara dasha lagnaya bağlıdır ve hesaplanmaz.")).toBeInTheDocument();
  expect(vi.mocked(api.computeCharaDasha).mock.calls[0][0].time_known).toBe(false);
  expect(api.charaDashaAt).not.toHaveBeenCalled();
  expect(screen.queryByRole("list")).toBeNull();
  expect(screen.getByRole("combobox", { name: "Sistem" })).toHaveDisplayValue("Chara dasha");
});

test("errors keep the selectors; retry from the keyboard keeps the focus in the panel", async () => {
  vi.mocked(api.computeCharaDasha).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderStrip();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.getByRole("combobox", { name: "Sistem" })).toBeInTheDocument();
  expect(screen.getByRole("combobox", { name: "Seviye" })).toBeInTheDocument();
  const retry = screen.getByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  expect(await buttons()).toHaveLength(12);
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Dasa şeridi paneli" }));
});

test("a failed path or third-level request is an error with retry; an empty third level is not a silent empty strip", async () => {
  vi.mocked(api.charaDashaAt).mockResolvedValueOnce({ status: "error", error: { code: "invalid_dasha_input", params: { field: "jd_ut" } } });
  vi.mocked(api.charaDashaChildren).mockResolvedValueOnce({ status: "ok", data: [] });
  renderStrip({ system: "chara", level: "pratyantar" });
  expect(await screen.findByRole("alert")).toHaveTextContent("Dasa isteği geçersiz: jd_ut.");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  expect(await screen.findByText("Bu dönemin alt dönemleri alınamadı.")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  expect(await buttons()).toHaveLength(12);
});

test("loading: nothing under a second, then a skeleton, then the strip", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<CharaDasha | null>) => void;
    vi.mocked(api.computeCharaDasha).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderStrip();
    await act(async () => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    await act(async () => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve({ status: "ok", data: fixture().chara_dasha }));
    await act(async () => {});
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getByRole("list", { name: /Dasa şeridi/ })).toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
});

test("no open chart: a message, no request", () => {
  renderWithStores(<DashaStripPanel panel={{ id: "x", kind: "dasha_strip", x: 0, y: 0, w: 12, h: 4, settings: { system: "chara" } }} />, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  for (const f of [api.transitsAt, api.computeCharaDasha, api.charaDashaAt, api.charaDashaChildren]) expect(f).not.toHaveBeenCalled();
});
````

- [ ] **Step 2: Testin başarısız olduğunu doğrula**

```bash
(cd app && npx vitest run ui/dasha/CharaDashaStrip.test.tsx 2>&1 | grep -E "Test Files|Tests |Unable to find" | head -4)
```
Expected: `Tests  10 failed | 1 passed (11)`; ilk hata `Unable to find an accessible element with the role "combobox" and name "Sistem"`.

- [ ] **Step 3: Fikstüre transit anındaki 3. seviyeyi ekle**

````diff
diff --git a/app/src-tauri/tests/ui_fixture.rs b/app/src-tauri/tests/ui_fixture.rs
index a5ee7d2..e60289e 100644
--- a/app/src-tauri/tests/ui_fixture.rs
+++ b/app/src-tauri/tests/ui_fixture.rs
@@ -117,9 +117,17 @@ fn ui_fixture_is_up_to_date() {
     let chara_children =
         chara_dasha_children_blocking(&engine, module_input.clone(), chara_path.clone(), 1)
             .expect("chara dasha children");
-    let chara_at = chara_dasha_at_blocking(&engine, module_input, transit.snapshot.jd_ut)
+    let chara_at = chara_dasha_at_blocking(&engine, module_input.clone(), transit.snapshot.jd_ut)
         .expect("chara dasha at")
         .expect("inside the produced span");
+    // Transit anındaki alt dönemin 3. seviyesi: dasa şeridinin "pratyantar" görünümü.
+    let chara_at_children = chara_dasha_children_blocking(
+        &engine,
+        module_input,
+        chara_at.signs[..2].to_vec(),
+        chara_at.cycle,
+    )
+    .expect("chara dasha children at the transit moment");
     // Kaydedilen doğum verisi girdinin kanonik biçimidir (Plan 10 kapsam dışı 21).
     assert_eq!(opened.chart_record.birth, delhi());
     let prefs = store
@@ -140,6 +148,7 @@ fn ui_fixture_is_up_to_date() {
         "chara_dasha": chara_dasha,
         "chara_dasha_children": { "path": chara_path, "cycle": 1, "periods": chara_children },
         "chara_dasha_at": chara_at,
+        "chara_dasha_at_children": chara_at_children,
     });
     let text = serde_json::to_string(&value)
         .expect("serialize")
diff --git a/app/ui/test/fixture.ts b/app/ui/test/fixture.ts
index 1169a4e..6433569 100644
--- a/app/ui/test/fixture.ts
+++ b/app/ui/test/fixture.ts
@@ -40,6 +40,8 @@ export interface UiFixture {
   chara_dasha_children: { path: Sign[]; cycle: number; periods: SignDashaPeriod[] };
   /** `chara_dasha_at`, fikstürdeki transit anı için (`transit.snapshot.jd_ut`). */
   chara_dasha_at: SignDashaPath;
+  /** `chara_dasha_children`: transit anındaki alt dönemin (`chara_dasha_at` yolunun ilk iki burcu) 3. seviyesi. */
+  chara_dasha_at_children: SignDashaPeriod[];
 }
 
 const FIXTURE = JSON.parse(raw) as UiFixture;
````

```bash
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
git diff --stat -- app/ui/test/fixtures/delhi.json
```
Expected: `ok`; `delhi.json` tek satır değişir (`chara_dasha_at_children` anahtarı: Balık–Başak alt döneminin on iki 3. seviye dönemi).

- [ ] **Step 4: Seviye yardımcılarını taşı; chara şeridi**

````diff
diff --git a/app/ui/dasha/system.tsx b/app/ui/dasha/system.tsx
index a8040f2..fafaf09 100644
--- a/app/ui/dasha/system.tsx
+++ b/app/ui/dasha/system.tsx
@@ -1,5 +1,6 @@
-// Dasa sistemi seçimi (ileri modüller spec'i §6): dasa ağacı ve dasa şeridi panellerinin
-// `system` ayarı. Ayar panelindir ve düzenle kaydedilir; tanınmayan değer Vimshottari sayılır.
+// Dasa panellerinin ayarları. Sistem seçimi (ileri modüller spec'i §6): dasa ağacı ve dasa
+// şeridi panellerinin `system` ayarı; şeridin `level` ayarı (UI spec §3.3). Ayarlar panelindir
+// ve düzenle kaydedilir; tanınmayan sistem Vimshottari, tanınmayan seviye antar sayılır.
 import { useId } from "react";
 import { useTranslation } from "react-i18next";
 
@@ -10,6 +11,16 @@ export function dashaSystem(settings: Record<string, unknown>): DashaSystem {
   return settings.system === "chara" ? "chara" : "vimshottari";
 }
 
+export const STRIP_LEVELS = ["maha", "antar", "pratyantar"] as const;
+export type StripLevel = (typeof STRIP_LEVELS)[number];
+
+export function stripLevel(settings: Record<string, unknown>): StripLevel {
+  return (STRIP_LEVELS as readonly unknown[]).includes(settings.level) ? (settings.level as StripLevel) : "antar";
+}
+
+/** Seviyenin üst yol uzunluğu: maha için kök (0), antar için 1, pratyantar için 2. */
+export const PARENT_DEPTH: Record<StripLevel, number> = { maha: 0, antar: 1, pratyantar: 2 };
+
 /** "Sistem" seçicisi: görünür etiketli `select`; panelin araç çubuğuna konur. */
 export function SystemSelect({ value, onChange }: { value: DashaSystem; onChange(system: DashaSystem): void }) {
   const { t } = useTranslation();
````

**Yeni dosya:** `app/ui/dasha/CharaDashaStrip.tsx`

````tsx
// Chara dasha şeridi (ileri modüller spec'i §6). Dönemler `compute_chara_dasha`'dan (iki
// seviye) ve `chara_dasha_children`'dan (3. seviye), transit anının dönem yolu
// `chara_dasha_at`'tan gelir; arayüz tarih karşılaştırmaz (C4). "Maha" seviyesi iki çevrimin
// dashalarını birlikte gösterir; ikinci çevrim "2." önekiyle ayrılır. Bir döneme tıklamak
// transit anını o dönemin başlangıcına taşır (Vimshottari şeridiyle aynı).
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { OpenedClient, SignDashaPeriod } from "../bindings";
import { chartInput, inputKey } from "../modules/input";
import { useModule } from "../modules/useModule";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "../panels/states";
import { useTransit } from "../transit/store";
import { sameCharaPath } from "../view/model";
import { charaChildrenOf, charaDashas, charaKey, charaName, useCharaChildren, useCharaDasha } from "./chara";
import { PARENT_DEPTH, type StripLevel } from "./system";
import { formatJdDate } from "./tree";

export function CharaDashaStrip({ opened, level }: { opened: OpenedClient; level: StripLevel }) {
  const { t } = useTranslation();
  const transit = useTransit();
  const module = useCharaDasha(opened);
  const dasha = module.data;
  const jd = transit.view?.snapshot.jd_ut ?? null;
  // Transit anının dönem yolu; dasha yoksa (saat bilinmiyor) ya da an henüz gelmediyse istenmez.
  const at = useModule(dasha !== null && jd !== null ? `${inputKey(opened)}|at|${jd}` : null, () => api.charaDashaAt(chartInput(opened), jd as number));
  const path = at.data;
  const depth = PARENT_DEPTH[level];
  // 3. seviye istenerek yüklenir: anın alt döneminin (yolun ilk iki burcu) alt dönemleri.
  const lazy = useCharaChildren(opened, path?.cycle ?? 1, path !== null && depth === 2 ? path.signs.slice(0, 2) : null);
  const loading = module.loading || at.loading || lazy.loading || (transit.loading && transit.view === null);
  const phase = usePanelLoadPhase(loading);

  if (transit.error) return <PanelError error={transit.error} onRetry={transit.retry} />;
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (at.error) return <PanelError error={at.error} onRetry={at.retry} />;
  if (lazy.error) return <PanelError error={lazy.error} onRetry={lazy.retry} />;
  if (module.loading || transit.view === null || at.loading || lazy.loading) return phase === "skeleton" ? <PanelSkeleton /> : null;
  if (dasha === null) return <p className="panel-notice">{t("dasha.chara.timeUnknown")}</p>;
  if (path === null) return <p className="panel-message">{t("dasha.chara.outside")}</p>;

  const parent = path.signs.slice(0, depth);
  const periods: SignDashaPeriod[] =
    depth === 0 ? charaDashas(dasha.periods) : depth === 1 ? charaChildrenOf(dasha.periods, path.cycle, parent) : charaChildrenOf(lazy.data ?? [], path.cycle, parent);
  // Boş yanıt sessizce boş şerit olarak gösterilmez: her dönemin on iki alt dönemi vardır.
  if (periods.length === 0) return <PanelError error={{ code: "dasha_empty", params: {} }} onRetry={depth === 2 ? lazy.retry : module.retry} />;
  const offset = opened.chart_record.resolved.offset_secs;
  const now = { cycle: path.cycle, signs: path.signs.slice(0, depth + 1) };
  return (
    <ol className="dasha-strip" aria-label={t("dasha.strip.label", { level: t(`dasha.strip.levels.${level}`) })}>
      {periods.map((p) => {
        const current = sameCharaPath({ cycle: p.cycle, signs: p.signs }, now);
        return (
          <li key={charaKey(p.cycle, p.signs)}>
            <button
              type="button"
              className={`dasha-strip-item${current ? " dasha-strip-item--current" : ""}`}
              data-sign={p.signs[p.signs.length - 1]}
              data-cycle={p.cycle}
              aria-current={current ? "true" : undefined}
              onClick={() => transit.setAt({ Jd: p.start_jd_ut })}
            >
              <span className="dasha-strip-name">{charaName(t, p.cycle, p.signs)}</span>
              <span className="dasha-strip-date">{formatJdDate(p.start_jd_ut, offset)}</span>
              {current && <span className="dasha-strip-now">{t("dasha.strip.current")}</span>}
            </button>
          </li>
        );
      })}
    </ol>
  );
}
````

- [ ] **Step 5: Panel, stil ve metin**

Araç çubuğu (sistem ve seviye seçicileri) panele taşınır ve yükleme, hata ve "saat bilinmiyor" durumlarında da görünür. Seviye seçicisinin değeri `stripLevel` ile daraltılır.

````diff
diff --git a/app/ui/dasha/DashaStripPanel.tsx b/app/ui/dasha/DashaStripPanel.tsx
index 93309e0..cedeefb 100644
--- a/app/ui/dasha/DashaStripPanel.tsx
+++ b/app/ui/dasha/DashaStripPanel.tsx
@@ -1,6 +1,8 @@
-// Dasa şeridi (UI spec §3.3, §3.4). Transit anının dasa yolu Rust'tan gelir (`transits_at`,
-// Plan 11 D10); arayüz tarih karşılaştırmaz (C4). Seçilen seviyede anın üst döneminin alt
-// dönemleri sıralanır; bir döneme tıklamak transit anını o dönemin başlangıcına taşır.
+// Dasa şeridi (UI spec §3.3, §3.4). Transit anının dasa yolu Rust'tan gelir (Vimshottari:
+// `transits_at`, Plan 11 D10; chara dasha: `chara_dasha_at`); arayüz tarih karşılaştırmaz (C4).
+// Seçilen seviyede anın üst döneminin alt dönemleri sıralanır; bir döneme tıklamak transit
+// anını o dönemin başlangıcına taşır. `system` ayarı Vimshottari ile chara dasha arasında seçer
+// (ileri modüller spec'i §6).
 import { useId } from "react";
 import { useTranslation } from "react-i18next";
 import type { DashaPeriod } from "../bindings";
@@ -9,29 +11,38 @@ import type { PanelProps } from "../panels/types";
 import { useSession } from "../session/store";
 import { useTransit } from "../transit/store";
 import { useLayoutStore } from "../view/store";
+import { CharaDashaStrip } from "./CharaDashaStrip";
 import "./dasha.css";
+import { dashaSystem, PARENT_DEPTH, STRIP_LEVELS, stripLevel, SystemSelect, type StripLevel } from "./system";
 import { childrenOf, formatJdDate, samePath } from "./tree";
 
-export const STRIP_LEVELS = ["maha", "antar", "pratyantar"] as const;
-export type StripLevel = (typeof STRIP_LEVELS)[number];
-
-export function stripLevel(settings: Record<string, unknown>): StripLevel {
-  return (STRIP_LEVELS as readonly unknown[]).includes(settings.level) ? (settings.level as StripLevel) : "antar";
-}
-
-/** Seviyenin üst yol uzunluğu: maha için kök (0), antar için 1, pratyantar için 2. */
-const PARENT_DEPTH: Record<StripLevel, number> = { maha: 0, antar: 1, pratyantar: 2 };
-
 export function DashaStripPanel({ panel }: PanelProps) {
   const { t } = useTranslation();
   const { opened } = useSession();
   const { setPanelSettings } = useLayoutStore();
   const id = useId();
   if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
-  return <Strip level={stripLevel(panel.settings)} id={id} onLevel={(level) => setPanelSettings(panel.id, { level })} />;
+  const system = dashaSystem(panel.settings);
+  const level = stripLevel(panel.settings);
+  return (
+    <div className="dasha-strip-panel">
+      <div className="dasha-strip-toolbar">
+        <SystemSelect value={system} onChange={(next) => setPanelSettings(panel.id, { system: next })} />
+        <label htmlFor={`${id}-level`}>{t("dasha.strip.level")}</label>
+        <select id={`${id}-level`} value={level} onChange={(e) => setPanelSettings(panel.id, { level: stripLevel({ level: e.target.value }) })}>
+          {STRIP_LEVELS.map((l) => (
+            <option key={l} value={l}>
+              {t(`dasha.strip.levels.${l}`)}
+            </option>
+          ))}
+        </select>
+      </div>
+      {system === "chara" ? <CharaDashaStrip opened={opened} level={level} /> : <Strip level={level} />}
+    </div>
+  );
 }
 
-function Strip({ level, id, onLevel }: { level: StripLevel; id: string; onLevel(level: StripLevel): void }) {
+function Strip({ level }: { level: StripLevel }) {
   const { t } = useTranslation();
   const { opened } = useSession();
   const transit = useTransit();
@@ -44,46 +55,29 @@ function Strip({ level, id, onLevel }: { level: StripLevel; id: string; onLevel(
   const currentPath = path?.slice(0, depth + 1) ?? [];
 
   const name = (p: DashaPeriod) => p.lords.map((g) => t(`grahaName.${g}`)).join("–");
+  if (transit.error) return <PanelError error={transit.error} onRetry={transit.retry} />;
+  if (transit.view && periods.length === 0) return <p className="panel-message">{t("dasha.strip.outside")}</p>;
+  if (periods.length === 0) return null;
   return (
-    <div className="dasha-strip-panel">
-      <div className="dasha-strip-toolbar">
-        <label htmlFor={`${id}-level`}>{t("dasha.strip.level")}</label>
-        <select id={`${id}-level`} value={level} onChange={(e) => onLevel(e.target.value as StripLevel)}>
-          {STRIP_LEVELS.map((l) => (
-            <option key={l} value={l}>
-              {t(`dasha.strip.levels.${l}`)}
-            </option>
-          ))}
-        </select>
-      </div>
-      {transit.error ? (
-        <PanelError error={transit.error} onRetry={transit.retry} />
-      ) : transit.view && periods.length === 0 ? (
-        <p className="panel-message">{t("dasha.strip.outside")}</p>
-      ) : (
-        periods.length > 0 && (
-          <ol className="dasha-strip" aria-label={t("dasha.strip.label", { level: t(`dasha.strip.levels.${level}`) })}>
-            {periods.map((p) => {
-              const current = samePath(p.lords, currentPath);
-              return (
-                <li key={p.lords.join(",")}>
-                  <button
-                    type="button"
-                    className={`dasha-strip-item${current ? " dasha-strip-item--current" : ""}`}
-                    data-lord={p.lords[p.lords.length - 1]}
-                    aria-current={current ? "true" : undefined}
-                    onClick={() => transit.setAt({ Jd: p.start_jd_ut })}
-                  >
-                    <span className="dasha-strip-name">{name(p)}</span>
-                    <span className="dasha-strip-date">{formatJdDate(p.start_jd_ut, offset)}</span>
-                    {current && <span className="dasha-strip-now">{t("dasha.strip.current")}</span>}
-                  </button>
-                </li>
-              );
-            })}
-          </ol>
-        )
-      )}
-    </div>
+    <ol className="dasha-strip" aria-label={t("dasha.strip.label", { level: t(`dasha.strip.levels.${level}`) })}>
+      {periods.map((p) => {
+        const current = samePath(p.lords, currentPath);
+        return (
+          <li key={p.lords.join(",")}>
+            <button
+              type="button"
+              className={`dasha-strip-item${current ? " dasha-strip-item--current" : ""}`}
+              data-lord={p.lords[p.lords.length - 1]}
+              aria-current={current ? "true" : undefined}
+              onClick={() => transit.setAt({ Jd: p.start_jd_ut })}
+            >
+              <span className="dasha-strip-name">{name(p)}</span>
+              <span className="dasha-strip-date">{formatJdDate(p.start_jd_ut, offset)}</span>
+              {current && <span className="dasha-strip-now">{t("dasha.strip.current")}</span>}
+            </button>
+          </li>
+        );
+      })}
+    </ol>
   );
 }
diff --git a/app/ui/dasha/dasha.css b/app/ui/dasha/dasha.css
index 5401c73..62f19ca 100644
--- a/app/ui/dasha/dasha.css
+++ b/app/ui/dasha/dasha.css
@@ -55,6 +55,13 @@
   gap: 6px;
 }
 
+.dasha-strip-toolbar {
+  display: flex;
+  align-items: center;
+  flex-wrap: wrap;
+  gap: 6px 12px;
+}
+
 .dasha-strip {
   display: flex;
   gap: 4px;
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index a3cabcc..54a8533 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -357,6 +357,7 @@ export const en: Messages = {
       secondCycle: "2. {{name}}",
       timeUnknown: "Birth time unknown: chara dasha depends on the lagna and is not computed.",
       method: "Method: {{method}}",
+      outside: "The transit moment is outside the chara dasha periods.",
     },
     strip: {
       level: "Level",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 1cd6e1d..5fc1cc2 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -357,6 +357,7 @@ export const fr: Messages = {
       secondCycle: "2. {{name}}",
       timeUnknown: "Heure de naissance inconnue : la chara dasha dépend du lagna et n'est pas calculée.",
       method: "Méthode : {{method}}",
+      outside: "Le moment du transit est hors des périodes de la chara dasha.",
     },
     strip: {
       level: "Niveau",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 137e274..e8691c9 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -356,6 +356,7 @@ export const tr = {
       secondCycle: "2. {{name}}",
       timeUnknown: "Doğum saati bilinmiyor: chara dasha lagnaya bağlıdır ve hesaplanmaz.",
       method: "Yöntem: {{method}}",
+      outside: "Transit anı chara dasha dönemlerinin dışında.",
     },
     strip: {
       level: "Seviye",
````

- [ ] **Step 6: Testleri koş**

```bash
(cd app && npx vitest run ui/dasha 2>&1 | grep -E "Test Files|Tests ")
```
Expected: `Test Files  5 passed (5)`, `Tests  43 passed (43)` (yeni dosya 11 test; `DashaStripPanel.test.tsx`'in 5 testi değişmeden geçer).

- [ ] **Step 7: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p jyotish-desk 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
git diff -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
```
Expected: fmt ve clippy çıktısız; `ui_fixture` `2 passed`, diğer satırlar Task 3'teki gibi; Vitest `70 passed (70)`, `649 passed (649)`; `✓ built`; son komut çıktısız.

- [ ] **Step 8: Commit**

```bash
git add app/ui app/src-tauri/tests/ui_fixture.rs
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
git commit -m "feat(ui): dasha strip system selector and the chara dasha strip; the current period comes from chara_dasha_at"
```
Expected: iki `grep` çıktısız; `grep -ciE` `0` yazar.

---

### Task 6: Uçtan uca: chara dasha

Tek test, gerçek veri fikstürüyle: K1 (seçilmeden istenmez), sistem seçicisi, ağaçta klavye ve istenerek yüklenen 3. seviye, açık dalın sistem başına ayrı kaydı, şeritte Rust'ın adlandırdığı güncel dönem ve tıklamanın transit anını taşıması. IPC taklidi dashayı yeniden hesaplamaz: fikstürdeki sonucu döndürür; `time_known` yanlışsa `null` döner (Rust gibi).

**Files:**
- Modify: `app/e2e/ipc.ts`, `app/e2e/desk.spec.ts`

**Interfaces:**
- Consumes: Task 2–5'in tamamı; `FIXTURE.chara_dasha`, `chara_dasha_children`, `chara_dasha_at`, `chara_dasha_at_children`.
- Produces: e2e taklidinde `compute_chara_dasha`, `chara_dasha_children`, `chara_dasha_at`; test `chara dasha: …`.

- [ ] **Step 1: IPC taklidini ve testi yaz**

````diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index ee67cfe..aa1f33a 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -98,6 +98,16 @@ export async function mockIpc(page: Page, { session, unknownTime = false }: IpcO
         // Jaimini raporu: fikstürdeki gerçek sonuç; saat bilinmiyorsa Rust padaları boş döner.
         compute_jaimini: (args) =>
           (args.input as { time_known: boolean }).time_known ? fixture.jaimini : { ...fixture.jaimini, arudhas: null },
+        // Chara dasha: fikstürdeki gerçek sonuç; saat bilinmiyorsa Rust `null` döner. 3. seviye
+        // yalnızca fikstürde bulunan iki yol için vardır; transit anının yolu sabittir.
+        compute_chara_dasha: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.chara_dasha : null),
+        chara_dasha_children: (args) => {
+          const asked = JSON.stringify([args.cycle, args.path]);
+          if (asked === JSON.stringify([fixture.chara_dasha_children.cycle, fixture.chara_dasha_children.path])) return fixture.chara_dasha_children.periods;
+          if (asked === JSON.stringify([fixture.chara_dasha_at.cycle, fixture.chara_dasha_at.signs.slice(0, 2)])) return fixture.chara_dasha_at_children;
+          return [];
+        },
+        chara_dasha_at: () => fixture.chara_dasha_at,
         print_window: () => null,
         // Güncelleme: sahte yayın; ağ yoktur. Kurulum yalnızca denetimin bildirdiği sürümü kabul eder.
         app_info: () => ({ version: "0.9.0", updaterConfigured: true }),
````

````diff
diff --git a/app/e2e/desk.spec.ts b/app/e2e/desk.spec.ts
index dfd0e6a..400adb9 100644
--- a/app/e2e/desk.spec.ts
+++ b/app/e2e/desk.spec.ts
@@ -434,3 +434,77 @@ test("jaimini panels: karakas, arudha padas and argala come from one command; ch
   await expect(arudha).toBeVisible();
   expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
 });
+
+test("chara dasha: the system selector in the dasha tree and strip; each system keeps its own open path; the strip marks the period Rust names", async ({ page }) => {
+  await mockIpc(page, { session: true });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  await page.keyboard.press("Meta+Digit2");
+  const panel = page.getByRole("region", { name: "Dasa ağacı" });
+  const tree = panel.getByRole("tree", { name: "Dasa ağacı" });
+  await expect(tree.getByRole("treeitem")).toHaveCount(9);
+  // Chara dasha seçilmeden istenmez (ileri modüller spec'i K1).
+  expect(await ipcCalls(page, "compute_chara_dasha")).toHaveLength(0);
+
+  const system = panel.getByRole("combobox", { name: "Sistem" });
+  await expect(system.locator("option:checked")).toHaveText("Vimshottari");
+  await system.selectOption("chara");
+  await expect(panel.getByText("Yön: geri")).toBeVisible();
+  await expect(panel.getByText("Yıllar: İki 6 · Boğ 6 · Koç 10 · Bal 11 · Kov 10 · Oğl 9 · Yay 4 · Akr 3 · Ter 1 · Baş 9 · Asl 8 · Yen 9")).toBeVisible();
+  await expect(tree.getByRole("treeitem")).toHaveCount(20);
+  await expect(tree.getByRole("treeitem").first()).toHaveAccessibleName("İki, 2000-01-01 – 2006-01-01");
+  await expect(tree.getByRole("treeitem", { name: /^2\. İki, / })).toBeVisible();
+  const computed = (await ipcCalls(page, "compute_chara_dasha")) as { input: { jd_ut: number; time_known: boolean; settings: { schema_version: number } } }[];
+  expect(computed).toHaveLength(1);
+  expect(computed[0].input).toMatchObject({ jd_ut: 2451545, time_known: true, settings: { schema_version: 2 } });
+
+  // Klavye: sağ ok açar, sonra ilk alt döneme iner; 3. seviye istenerek yüklenir ve son seviyedir.
+  await tree.getByRole("treeitem").first().focus();
+  for (const key of ["ArrowRight", "ArrowRight", "ArrowRight"]) await page.keyboard.press(key);
+  await expect(tree.getByRole("treeitem", { name: /^İki–Boğ–Koç, / })).toBeVisible();
+  await page.keyboard.press("ArrowRight");
+  await expect(page.locator(":focus")).toHaveAttribute("aria-level", "3");
+  await expect(page.locator(":focus")).not.toHaveAttribute("aria-expanded");
+  const children = (await ipcCalls(page, "chara_dasha_children")) as { path: string[]; cycle: number }[];
+  expect(children.map((c) => [c.path, c.cycle])).toEqual([[["Gemini", "Taurus"], 1]]);
+
+  // Açık dal sistem başına ayrı kaydedilir; sistem panelin ayarıdır.
+  await expect
+    .poll(async () => {
+      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as
+        | { viewState: { dasha_expanded_path: string[]; chara_expanded_path: unknown; panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: Record<string, unknown> }[] }[] } } }
+        | undefined;
+      const layout = saved?.viewState.panel_layout;
+      const settings = layout?.layouts.find((l) => l.id === layout.active_layout_id)?.panels.find((p) => p.kind === "dasha_tree")?.settings;
+      return [saved?.viewState.chara_expanded_path, saved?.viewState.dasha_expanded_path, settings?.system];
+    })
+    .toEqual([{ cycle: 1, signs: ["Gemini", "Taurus"] }, [], "chara"]);
+  // Vimshottari'ye dönünce onun (kapalı) ağacı gelir; chara dashaya dönünce açık dal yerindedir.
+  await system.selectOption("vimshottari");
+  await expect(tree.getByRole("treeitem")).toHaveCount(9);
+  await system.selectOption("chara");
+  await expect(tree.getByRole("treeitem", { name: /^İki–Boğ, / })).toHaveAttribute("aria-expanded", "true");
+  await expect(tree.getByRole("treeitem", { name: /^İki–Boğ–Koç, / })).toBeVisible();
+
+  // Dasa şeridi: güncel dönem Rust'tan gelir (`chara_dasha_at`); tıklama transit anını taşır.
+  await page.keyboard.press("Meta+Digit3");
+  const strip = page.getByRole("region", { name: "Dasa şeridi" });
+  await expect(strip.getByRole("list").getByRole("button")).toHaveCount(9);
+  expect(await ipcCalls(page, "chara_dasha_at")).toHaveLength(0);
+  await strip.getByRole("combobox", { name: "Sistem" }).selectOption("chara");
+  const items = strip.getByRole("list").getByRole("button");
+  await expect(items).toHaveCount(12);
+  await expect(items.first()).toContainText("Bal–Kov");
+  const current = strip.locator('[aria-current="true"]');
+  await expect(current).toHaveCount(1);
+  await expect(current).toHaveAttribute("data-sign", "Virgo");
+  await expect(current).toContainText("transit anında");
+  const asked = (await ipcCalls(page, "chara_dasha_at")) as { jdUt: number; input: { time_known: boolean } }[];
+  expect(asked).toHaveLength(1);
+  expect(asked[0].jdUt).toBeCloseTo(2461369.9166666665, 6);
+  await strip.getByRole("combobox", { name: "Seviye" }).selectOption("maha");
+  await expect(items).toHaveCount(20);
+  await expect(strip.locator('[aria-current="true"]')).toHaveAttribute("data-sign", "Pisces");
+  await items.first().click();
+  await expect.poll(async () => JSON.stringify((await ipcCalls(page, "transits_at")).at(-1))).toContain('"Jd":2451545');
+});
````

- [ ] **Step 2: Koş**

```bash
lsof -i :1421
pnpm --dir app build 2>&1 | grep "✓ built"
pnpm --dir app e2e 2>&1 | tail -1
pnpm --dir app e2e:visual 2>&1 | tail -1
rm -rf app/test-results
git status --short
```
Expected: `lsof` çıktısız (port boş); `✓ built` (`tsc -p e2e` dahil); e2e `15 passed`; görsel `10 passed` (temel PNG'ler değişmedi: D17); `git status --short` yalnızca iki e2e dosyasını gösterir.

- [ ] **Step 3: Commit**

```bash
git add app/e2e/ipc.ts app/e2e/desk.spec.ts
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(e2e): chara dasha in the dasha tree and strip; one open path per system; the current period from chara_dasha_at"
```
Expected: `grep` çıktısız.

---

### Task 7: Son doğrulama, elle denetim kapısı ve el örneklerinin yeniden türetilmesi

Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle çözülür ve bu görev baştan koşulur.

**Files:**
- Yok (yalnızca doğrulama). İlerleme notları depo dışıdır; `git add` edilmez.

**Interfaces:**
- Consumes: Task 1–6'nın tamamı.
- Produces: yeşil kapılar, kural taramaları, elle denetim sonucu, sahibine devredilen maddeler ve el örneklerinin bağımsız yeniden türetilmesi (ilerleme notlarına yazılır).

- [ ] **Step 1: Rust kapıları**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test -p jyotish-desk --test modules -- --nocapture 2>&1 | grep median
cargo deny check 2>&1 | tail -1
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p jyotish-core -p store -p jyotish-desk --all-targets 2>&1 | tail -1
git diff ac2b412..HEAD --stat -- deny.toml Cargo.toml Cargo.lock app/package.json app/pnpm-lock.yaml app/vite.config.ts app/src-tauri/capabilities app/src-tauri/tauri.conf.json crates/store/src/schema.rs crates/jyotish-core/src/settings.rs crates/jyotish-core/src/chart.rs
```
Expected: fmt ve clippy çıktısız; testler: `jyotish-core` lib `191 passed`, golden `8 passed`, `modules_golden` `7 passed`; `store` lib `86 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `46 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `modules` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; beş `median` satırı, `compute_chara_dasha` ve `chara_dasha_at` 10 ms'nin, `chara_dasha_children` 5 ms'nin altında; deny `advisories ok, bans ok, licenses ok, sources ok`; MSRV `Finished`; son `git diff --stat` çıktısız (bağımlılık, yetenek, yapılandırma, SQLite şeması, ayar tipi ve `Chart` değişmedi: K1, C3).

- [ ] **Step 2: Arayüz kapıları**

```bash
lsof -i :1421
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
pnpm --dir app e2e 2>&1 | tail -1
pnpm --dir app e2e:visual 2>&1 | tail -1
rm -rf app/test-results
node --test "scripts/release/*.test.mjs" 2>&1 | grep -E "ℹ (pass|fail)"
git status --short
```
Expected: `lsof` çıktısız; `Test Files  70 passed (70)`, `Tests  649 passed (649)`; `✓ built`; e2e `15 passed`; görsel `10 passed` (temel PNG'ler değişmedi: D17); betik testleri `pass 14`, `fail 0`; `git status --short` boş.

- [ ] **Step 3: Kural taramaları (`ac2b412`'den bu yana)**

```bash
git diff ac2b412..HEAD -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log ac2b412..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff ac2b412..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
git diff ac2b412..HEAD --name-only | grep -vE '^(app|crates|scripts|tests|docs)/'
echo "trailers: $(git log ac2b412..HEAD --format=%B | grep -c '^Co-Authored-By: ') commits: $(git rev-list --count ac2b412..HEAD)"
git diff ac2b412..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
git diff ac2b412..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\(|window\.(print|open)\('
git diff ac2b412..HEAD -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
grep -rn '"source"' tests/golden/modules | grep -v '"source": "hand"'
ls tests/golden/modules/chara_dasha/*.json | wc -l
grep -rnE 'serde\(default' crates/jyotish-core/src crates/store/src/view_state.rs
grep -rnE 'XMLHttpRequest|WebSocket|\bfetch\(' app/ui/modules app/ui/panels app/ui/dasha
git diff ac2b412..HEAD --stat -- app/ui/print app/e2e/visual.spec.ts-snapshots
git tag --points-at HEAD
```
Expected: `trailers:` satırındaki iki sayı eşit; `grep -ciE` satırı `0`, `ls … | wc -l` satırı `7` yazar; bunların dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur): yasaklı ad yok (üretilen fikstürde de), beklenmeyen yerde dosya yok, `console` ve yasak çağrı yok, yeni stilde hex ya da `rgb(` yok (C9), el örneklerinde `hand` dışında kaynak yok (K9), çekirdek tiplerinde ve görünüm kaydında `serde(default)` yok, modül, panel ve dasa kodunda ağ çağrısı yok (C1), baskı ve görsel temeller değişmedi (K10, D17), ajan etiket atmamış.

- [ ] **Step 4: Elle denetim kapısı (`pnpm --dir app tauri dev`)**

Gerçek uygulamada, deneme danışanlarıyla (gerçek danışan verisi kullanılmaz). Her madde ilerleme notlarına "geçti" ya da bulgu olarak yazılır; bulgu varsa düzeltme commit'i ve bu görev baştan. Her maddenin başında, otomatik bir karşılığı olup olmadığı yazılıdır. **[otomatik]** maddelerin karşılığı testlerde vardır ve yürütücü ajan bunları testin adını göstererek "geçti" sayabilir; **[insan]** maddeler yalnızca gerçek uygulamaya bakan bir insan tarafından doğrulanabilir ve sahibi yoksa ona devredilir (bu devir `git push`'u engellemez; hiçbir şey yayımlanmaz).

- [ ] **[otomatik]** "Araştırma" düzeninde dasa ağacı Vimshottari ile açılır ve chara dasha istenmez; "Sistem → Chara dasha" seçilince ağaç yirmi dasha gösterir (on iki burç ve ikinci çevrimde sekiz), başlık satırı "Yön: geri" ve on iki yılı taşır, ikinci çevrim satırları "2." ile başlar. (`CharaDashaTree.test.tsx`, e2e `chara dasha: …`)
- [ ] **[otomatik]** Ağaçta ok tuşları dalı açar ve alt döneme iner; 3. seviye istenerek yüklenir ve son seviyedir; Vimshottari ağacının klavyesi ve tembel seviyeleri değişmemiştir. (`CharaDashaTree.test.tsx`, `DashaTreePanel.test.tsx`, e2e)
- [ ] **[otomatik]** Açık dal sistem başına ayrı kaydedilir: chara dalı açıkken Vimshottari'ye geçip dönünce dal yerindedir; haritanın görünüm kaydı iki yolu da taşır; bu alandan önce yazılmış kayıt Vimshottari yolu olarak açılır. (`view/model.test.ts`, `view/store.test.tsx`, `store` `view_state` testleri, e2e)
- [ ] **[otomatik]** Dasa şeridinde "Sistem → Chara dasha": antar seviyesi anın dashasının on iki alt dönemini gösterir, güncel dönem "transit anında" diye işaretlidir; "maha" iki çevrimi birlikte gösterir; "pratyantar" 3. seviyeyi ister; bir döneme tıklamak transit anını o dönemin başlangıcına taşır. (`CharaDashaStrip.test.tsx`, e2e)
- [ ] **[otomatik]** Doğum saati "bilinmiyor" olan haritada ağaç ve şerit, dönemler yerine tek satır açıklama gösterir; sistem seçicisi yerindedir; Rust dashayı boş döndürür. (iki panel testi; Rust `modules` testi ve çekirdek `collectors_follow_…` testi)
- [ ] **[otomatik]** Hata durumunda seçiciler görünür durur; "Yeniden dene" klavyeyle kullanılınca odak panelde durur. (iki panel testi)
- [ ] **[insan]** Gerçek motorla ayarlar: "Jaimini eş lordları" kutusu kapatılınca chara ağacındaki yıl dizisi (Akrep ve Kova'nın yılları) değişir; "Dasa yılı" 360 güne alınınca dönem tarihleri öne gelir. (e2e taklidi dashayı yeniden hesaplamaz; bu yalnızca gerçek motorla görülür.)
- [ ] **[insan]** Chara ağacı ve şeridi açık ve koyu temada, Türkçe, İngilizce ve Fransızca dilde okunur; başlıktaki yıl satırı dar panelde alt satıra iner ve taşmaz; şeritte yirmi dasha yatay kaydırmayla gezilir; iki seçici ("Sistem", "Seviye") dar panelde alt alta düşer ve okunur.
- [ ] **[insan]** VoiceOver: ağaç öğesi burç kısaltması ve tarihlerle okunur, ikinci çevrim "2." ile ayrışır; "Sistem" seçicisi adıyla duyurulur; şeritte güncel dönem "transit anında" metniyle okunur.
- [ ] **[insan]** Bilinen bir haritada (astrologun kendi elindeki bir örnek) yön, on iki burcun yılları ve ilk dashaların alt dönem sırası astrologun beklediğiyle uyuşur. Uyuşmazlık bulgu olarak yazılır; yöntem farkıysa (padakrama düzeni, eş lord ölçütleri, ±1 yıl düzeltmesinin uygulanmaması, alt dönem sırası, ikinci çevrim) "Bu planın kapsamadıkları"na göre sınıflanır.
- [ ] **[insan]** Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum verisi, konum ya da dosya yolu görünmez.

Kapı geçtiyse ilerleme notlarına `Plan 16 Task 7 elle denetim: geçti (<tarih>)` yazılır; **[insan]** maddeleri devredildiyse `Plan 16 Task 7 elle denetim: otomatik maddeler geçti, insan maddeleri sahibine devredildi (<tarih>)` yazılır. Bu adım commit üretmez.

- [ ] **Step 5: El örneklerinin bağımsız yeniden türetilmesi (gözden geçirme maddesi)**

Spec §13: "`derivation` ayrı bir gözden geçirmede kuraldan yeniden hesaplanır." Gözden geçiren (motoru yazan dışında bir ajan ya da insan), `tests/golden/modules/chara_dasha/` altındaki yedi dosyanın her birinde yalnızca `input`'tan başlar ve motoru çalıştırmadan, Rust kaynağını okumadan `expected` değerlerini yeniden çıkarır: (1) lagnanın ve dokuz grahanın burcu; (2) yön (lagnadan 9. burcun grubu) ve dasha sırası; (3) Akrep ve Kova için altı ölçütün sırayla uygulanması (spec §5; ayar kapalıysa Mars ve Satürn) ve belirleyici ölçüt; (4) on iki burcun yılı: burcun kendi yönünde lorduna kapsayıcı sayım eksi bir, kendi burcunda 12; (5) ikinci çevrim: `12 − yıl`, 0 yıllık burçlar atlanır, birikimli yıl 120'nin altındayken başlayan dashalar (D4); (6) ilk iki dashanın alt dönem sırası: haritanın yönünde, dasha burcundan sonraki burçla başlar ve dasha burcuyla biter. Sonra kendi değerlerini `expected` ile, adımlarını `derivation` metniyle karşılaştırır; farklar yazılır. Fark varsa önce kural yeniden okunur; düzeltme hangi tarafta olursa olsun ayrı commit'tir ve beklenen değer motor çıktısına bakarak değiştirilmez. Ayrıca `chara_dasha.rs`'teki `HAND_YEARS` tablosunun 144 hücresi aynı kuraldan satır satır denetlenir. Sonuç ilerleme notlarına `Plan 16 el örnekleri yeniden türetildi: 7/7 (<tarih>)` olarak yazılır.

---

## Başarı ölçütleri

- `jyotish-core` chara dashayı verir: yön, on iki burcun yılları (eş lord ayarıyla), iki çevrim (atlanan burçlar ve 120 yıl kuralıyla), üç seviye dönem ve bir anın dönem yolu. Hepsi yedi el örneği (kaynak `hand`), elle yazılmış 144 hücrelik yıl tablosu, birim testleri ve proptest değişmezleriyle doğrulanır; doğum saati bilinmiyorsa dasha `None`'dır.
- Üç komut ortak kalıpla çalışır (motor iş parçacığı, `blocking`, `CommandError`), beş golden anında v1 haritasıyla çapraz denetlenir ve bütçelerinin içindedir (≤ 10 ms; alt dönemler ≤ 5 ms).
- Dasa ağacı ve dasa şeridi `system` ayarını kazanır; chara seçiliyken satırlar burç kısaltması taşır, ikinci çevrim "2." önekiyle ayrılır, ağaç başlığı yönü ve yılları gösterir, şerit güncel dönemi Rust'tan okur ve dönem seçimi transit anını sürer. Boş, yükleniyor, hata, dolu ve "saat bilinmiyor" durumları testlidir; renk tek başına anlam taşımaz; yorum metni yoktur.
- Açık dal sistem başına ayrı saklanır (`chara_expanded_path`); eski kayıt Vimshottari yolu olarak açılır; şema sürümleri ve SQLite şeması değişmemiştir.
- `Chart`, `open_client`, `CalcSettings`, baskı düzenleri ve görsel temeller değişmemiştir; Vimshottari ağacının ve şeridinin mevcut testleri değişmeden geçer.
- Kapılar: `jyotish-core` lib 191, `modules_golden` 7, `store` lib 86 (+1 ign.), `jyotish-desk` `modules` 1; Vitest 70/649; e2e 15; görsel 10; `cargo deny` temiz; diğer sayılar tabandaki gibi. Elle denetim kapısının **[otomatik]** maddeleri testlerle doğrulanmış, **[insan]** maddeleri yapılmış ya da sahibine devredilmiştir; el örnekleri bağımsız olarak yeniden türetilmiştir.

## Bu planın kapsamadıkları

- **Spec'in 3.3 için kapsam dışı saydıkları:** diğer chara dasha varyantları, yücelme ve düşüş için ±1 yıl düzeltmesi, sthira, shoola, narayana ve diğer burç dashaları, Yogini ve koşullu nakshatra dashaları.
- **Sonraki alt projeler (3.4+):** Shadbala ve Bhava bala, yogalar, uyum, Muhurta, Prashna, arama. `DashaAt` arama koşulunun chara dashaya genişlemesi 3.9'un işidir.
- Ağaçta dönem seçiminin transit anını sürmesi (D15: v1 ağacı sürmez; UI spec §3.4 ile fark ayrı bir iştir).
- Chara dashanın baskı düzeni (K10) ve Vimshottari baskı sayfasında sistem seçimi.
- Chara dasha sonucunun oturum içi paylaşılan önbelleği (D19) ve veritabanında saklanması (spec §16).
- `CharaDasha`'da uyarı alanı (D9) ve eş lord seçiminde belirleyici ölçütün gösterimi.
- Chara ağacında "güncel dönem" işareti (v1 ağacında da yoktur; güncel dönem şeritte gösterilir).
- Ayar penceresinde yeni denetim (K7), yeni panel türü ve palet komutu (D11), hazır düzenlere chara dashanın varsayılan olarak eklenmesi.
- Paket parçalama (D17: 500 kB uyarısı).
- Yorum metinleri ve AI yorum paneli; Windows ve Android.
- v1'den taşınan seyrek test dalgalanması (`NotesPanel.test.tsx › New note adds an empty note…`, yük altında): bu planın dosyalarına dokunmaz; ayrı bir bakım işidir.

## Spec'e geri bildirim (spec kusurları ve boşlukları)

Plan yazılırken bulunan ve Kararlar'da çözülen maddeler. "Kusur" spec'in kendi içinde çelişen ya da harfiyen uygulanamayan yeridir; "boşluk" yazılmamış ayrıntıdır.

1. **Kusur — §6 Dasha süresi: "lord `stronger_co_lord` ile seçilir".** `stronger_co_lord` ayara bakmaz; harfiyen uygulanırsa `jaimini_co_lords = false` chara dashada yok sayılır ve K7 ile çelişir. Çözüm (D3): `jaimini_lord(sign, positions, co_lords)`. "İki eş lord da burcun içindeyse 12 yıl" ayrı kural gibi yazılmış; seçimin doğal sonucudur, spec bunu öyle söylemeli.
2. **Boşluk — §6 İkinci çevrim: "120 yıl sonrasını geçen ilk dashada durur".** Tam 120. yılda biten dashadan sonra ne olacağı ve kuralın ilk çevrime uygulanıp uygulanmadığı yazılmamış. Çözüm (D4): ikinci çevrimde başlangıcı 120. yıldan önce olan dasha üretilir; ilk çevrim her zaman tamdır (zaten 120 yıla ulaşamaz; spec bunu not etmeli).
3. **Boşluk — §6 `CharaDasha.periods "// 2 seviye"`.** Sıra (ön-sıra), iki çevrimin aynı listede olduğu ve `years`'ın yalnızca ilk çevrimi verdiği yazılmamış. Alt dönem sınırlarının nasıl hesaplandığı da yok. Çözüm (D5, D6).
4. **Kusur — §6 IPC: şeridin "güncel dönem"i için komut yok.** Vimshottari'de anın yolu `transits_at`'ten gelir; arayüz tarih karşılaştırmaz (C4, Plan 11 D10). Çözüm (D7): `chara_dasha_at(input, jd_ut) -> Option<SignDashaPath>`; spec komutu ve `SignDashaPath` tipini eklemeli.
5. **Boşluk — §6 `chara_dasha_children`.** Boş yolun anlamı, saat bilinmiyorken dönüş (`Vec`, `None` olamaz), geçersiz çevrim ve yolun hata kodu yazılmamış. Çözüm (D8): boş liste; `invalid_dasha_input` (`cycle`, `path`, `jd_ut`).
6. **Boşluk — §6 `CharaDasha` ile K8.** K8 uyarıları sonucun `warnings` alanına koyar; tipte alan yok (`JaiminiReport` `LowPrecision`'ı taşıyor). Çözüm (D9): tip korunur; spec ya alanı eklemeli ya da uyarının harita başlığından okunduğunu yazmalı.
7. **Boşluk — §6 "`ViewState.dasha_expanded_path` sistem başına ayrı tutulur".** Biçim verilmemiş; chara yolunun çevrimi de taşıması gerektiği yazılmamış. Aynı bölümün "Kalıcı veri: yalnızca panel ayarı" cümlesi bu alanı anmıyor (çelişki). Çözüm (D10): kardeş alan `chara_expanded_path: { cycle, signs }`, şema sürümü 1; kalıcı veri panel ayarı (`system`) ve haritanın görünüm kaydındaki chara yoludur.
8. **Boşluk — K12 ile 3.3.** "Her alt proje panelini `MODULE_PANELS` listesine ekler"; 3.3 panel türü eklemez. Çözüm (D11): liste değişmez; K12 "yeni panel türü getiren her alt proje" demeli.
9. **Boşluk — §6 Arayüz: şerit.** Seviye adlarının chara dashadaki karşılığı, "maha" seviyesinde iki çevrimin gösterimi, 3. seviyenin istenerek yüklenmesi ve an dönemlerin dışındayken gösterilecek metin yazılmamış. Çözüm (D14).
10. **Boşluk — §6 "Dönem seçimi transit tarihini v1'deki gibi sürer".** v1'de yalnızca şerit sürer; UI spec §3.4 ağacı da sayar ama v1 ağacı bunu yapmaz. Çözüm (D15): v1 davranışı korunur; fark UI spec'te ya da ayrı bir işte kapatılmalı.
11. **Boşluk — §6 Test ve §13 şeması.** Chara dasha el örneğinin zaman girdisi (doğum anı, yıl uzunluğu) ve boş sonucun (`expected: null`) gösterimi tanımsız; "lordu 2. ve 12. burçta" ifadesinin hangi yönde sayıldığı yazılmamış. Çözüm (D16): test sabitleri; zodyak sırasıyla sonraki ve önceki burç, iki grup için dört durum.
12. **Boşluk — §15 bütçe tablosu.** `chara_dasha_children` (§6'da ≤ 5 ms) ve anın yolunu veren komut tabloda yok. Çözüm (D18).
13. **Boşluk — §6 Arayüz: ağaç.** "Saat bilinmiyor" durumunda ne gösterileceği, 3. seviyenin son seviye olduğu ve yöntem kimliğinin panelde gösterimi yazılmamış. Çözüm (D13).

## Self-Review

**1. Spec kapsamı.**

| Gereksinim (ileri modüller spec'i) | Görev |
|---|---|
| §6 Burç grupları: `counts_forward`, `directed_count` (3.2'de tanımlı) | Task 1 (tüketir) |
| §6 Dasha sırası: ilk dasha lagna; 9. burcun grubu yönü belirler; on iki burç birer kez | Task 1 (`chara_dasha_forward`, `chara_dasha_order`) |
| §6 Dasha süresi: sayım − 1; kendi burcunda 12; Akrep ve Kova; ±1 yıl yok | Task 1 (`chara_dasha_years`, `chara_dasha_sign_years`, `HAND_YEARS`) |
| §6 İkinci çevrim: `12 − yıl`; 0 yıl atlanır; 120 yıl | Task 1 (`chara_dasha`, `second_cycle_years`) |
| §6 Alt dönemler: on iki eşit parça; harita yönü; sonraki burçla başlar, dasha burcuyla biter; 3. seviye; en çok 3 | Task 1 (`sub_period_signs`, `sign_dasha_children`) |
| §6 Başlangıç: tam süre; `settings.dasha_year` | Task 1 (`chara_dasha`, toplayıcı) |
| §6 Saat bilinmiyorsa hesaplanmaz | Task 1 (toplayıcılar), Task 2 (komut testi), Task 4, 5 (paneller) |
| §6 Veri modeli: `SignDashaPeriod`, `CharaDasha` (alan adları spec'tekiyle birebir) | Task 1 |
| §6 Komutlar: `compute_chara_dasha`, `chara_dasha_children` | Task 2 (ve `chara_dasha_at`: D7) |
| §6 Arayüz: `system` ayarı; burç kısaltması; "2." öneki | Task 4, 5 |
| §6 Arayüz: başlıkta yön ve yıl dizisi | Task 4 |
| §6 Arayüz: dönem seçimi transit tarihini sürer | Task 5 |
| §6 Arayüz: açık dal sistem başına ayrı; toleranslı ayrıştırma; eski kayıt Vimshottari | Task 3 |
| §6 Test: en az 6 el örneği; zorunlu durumlar (ileri ve geri; 12 yıl; 2. ve 12. burç iki grup için; eş lord seçimi; atlanan burç) | Task 1 (`chara_dasha_fixtures_cover_the_required_cases`) |
| §6 Test: property'ler (1–12 yıl; permütasyon ve lagna; ardışık; alt dönem toplamı; son alt dönem ebeveyn; iki çevrim toplamı 12) | Task 1 (`chara_dasha_properties`), Task 2 (`chara_dasha_checks`) |
| §6 Test: 144 hücrelik tablo | Task 1 (`years_match_the_hand_table_in_every_cell`) |
| §6 Gizlilik, performans (≤ 10 ms; ≤ 5 ms) | C1; Task 2 (ölçüm), Task 7 |
| K1, K2, K3 | Task 1, 2 |
| K4, K5 | Task 1 (burç sayımı; `Positions`'ın rasi burçları) |
| K7 (ayarın hesapta kullanımı; `method`; yeni denetim yok) | Task 1 (D3), D11 |
| K8, §14 | D8 (yeni kod yok), C1 |
| K9, §13 (el örnekleri, değişmezler, ikinci yazım, çapraz denetim, komut katmanı) | Task 1, 2; Task 7 Step 5 |
| K10, K11, K12 | D17; burç kısaltmaları ve "Chara dasha" adı; D11 |
| Ek A | C13 taramaları (bu planda yeni adlandırma eşlemesi gerekmez) |

Boşluk: yok. Spec'in istemediği ama planın eklediği parçalar: `chara_dasha_at` komutu ve `SignDashaPath` (D7), yöntem satırı (D13), ortak `PeriodTree` (D12), doğum saati bilinmeyen el örneği (D16).

**2. Yer tutucu taraması.** "TODO", "TBD", "daha sonra", "uygun hata işleme" yok. Task 7'deki `<tarih>` denetimin yapıldığı günün yazılacağı yerdir. Her kod adımı tam dosya ya da `ac2b412` ağacına (ve önceki görevlere) karşı üretilmiş diff taşır.

**3. Tip tutarlılığı.**
- `CharaDasha { forward, years, periods, method }` ve `SignDashaPeriod { signs, level, cycle, start_jd_ut, end_jd_ut, has_children }` Task 1'de Rust'ta tanımlanır; TS bağlaması Task 2'de üretilir; `ui_fixture.rs`, `tests/modules.rs`, `dasha/chara.ts`, iki panel ve e2e aynı adları okur.
- `SignDashaPath { cycle, signs }` Task 1'de tanımlanır; `chara_dasha_at`'in dönüşü (Task 2), `ViewState.chara_expanded_path` (Task 3), `ViewModel.charaPath` ve `setCharaPath` (Task 3), `isCharaOpen` ve `sameCharaPath` (Task 4, 5) aynı tipi kullanır.
- El örneği alanları (`forward`, `order`, `years`, `second_cycle[].sign`, `second_cycle[].years`, `sub_periods`) düzenekteki `CharaDashaExpected`, README satırı ve yedi JSON dosyasında aynıdır.
- `CHARA_DASHA_METHOD = "chara_dasha.padakrama_v1"` Rust'ta, `modules_golden.rs`'te, `tests/modules.rs`'te, panel testindeki "Yöntem: chara_dasha.padakrama_v1" metninde ve spec §6'da aynıdır.
- Komut adları: Rust `compute_chara_dasha`, `chara_dasha_children`, `chara_dasha_at`; TS `computeCharaDasha`, `charaDashaChildren`, `charaDashaAt`; e2e taklidindeki anahtarlar ve bağımsız değişken adları (`input`, `path`, `cycle`, `jdUt`) aynıdır.
- Fikstür anahtarları (`chara_dasha`, `chara_dasha_children { path, cycle, periods }`, `chara_dasha_at`, `chara_dasha_at_children`) `ui_fixture.rs`, `test/fixture.ts`, panel testleri ve `e2e/ipc.ts`'te aynıdır.
- Panel ayar anahtarları: `dasha_tree` `system`; `dasha_strip` `system` ve `level`. Testler ve e2e aynı anahtarları kaydedilen düzende arar.
- `PeriodNode` / `PeriodLevel` Task 4'te tanımlanır; Vimshottari ağacı ve `CharaDashaTree` aynı alanları doldurur. `STRIP_LEVELS`, `stripLevel` ve `PARENT_DEPTH` Task 5'te `system.tsx`'e taşınır; `DashaStripPanel` ve `CharaDashaStrip` oradan okur.
- i18n: `dasha.system`, `dasha.systems.*` iki panelde ortak; `dasha.chara.timeUnknown` ağaçta ve şeritte; `dasha.chara.outside` yalnızca şeritte; metinler testlerde birebir geçer.

**4. Kural denetimi.** C1 (`ChartInput` ve transit anı loglanmaz; hata yalnızca alan adı taşır), C2′ (komutlar dizgi almaz), C3 (SQLite şeması ve ayar tipi değişmez; `ViewState` sürümü 1), C4 (yön, yıllar, dönemler ve anın yolu Rust'tan; biçimleme ve gruplama gösterim), C5 (chara yolu aynı kayıt yoluyla), C7 (kısayol eklenmez), C9 ("2." öneki, `aria-current` ve metin; yeni stilde hex yok), C10 (iki üretilen dosya komutla), C12 (her yeni anahtar üç dilde; yeni hata kodu yok), C13 (her commit'te tarama), K9 (yalnızca `hand`), K10 (baskı ve görsel temeller değişmez) görevlerin adımlarında ve Task 7 Step 3'te denetlenir.

**5. Koşulan doğrulama.** Bu plandaki kod, deponun `ac2b412` üzerindeki bir kopyasında (kısa bir yolda) altı commit olarak uygulanmış ve her commit'te ölçülmüştür: `jyotish-core` lib 173 → 191; `modules_golden` 5 → 7; `store` lib 84 → 86; `jyotish-desk` `modules` 1 (ortancalar: `compute_chara_dasha` 0,53–0,58 ms, `chara_dasha_children` 0,56–0,58 ms, `chara_dasha_at` 0,53–0,56 ms; `compute_drishti` 0,75–0,78 ms, `compute_jaimini` 0,63 ms); Vitest 624 → 624 → 628 → 638 → 649 (dosya 68 → 69 → 70); e2e 14 → 15. Son durumda `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`, `cargo deny check` (dört `ok`), 1.93 ile `cargo check`, `pnpm build` (516,2 kB; D17), görsel 10 (temel dosyalar değişmeden) ve yayın betiği testleri 14 yeşildir; Task 7 Step 3'ün taramaları beklendiği gibidir. "Testlerin başarısız olduğunu doğrula" adımlarındaki çıktılar da aynı kopyada, bir önceki görevin ağacına yalnızca o görevin test değişiklikleri konarak ölçülmüştür. Ayrıca bu belgedeki bloklar (15 yeni dosya ve 22 diff) belgeden okunup sırayla temiz bir `ac2b412` ağacına uygulanmış ve her görevin sonunda o commit'in ağacıyla birebir aynı sonucu vermiştir (üretilen iki dosyayı adımlardaki komutlar üretmiştir; onlar da aynıdır).

El örneklerinin yedisi de motorun ilk koşusunda geçmiştir; hiçbir beklenen değer motor çıktısına bakılarak değiştirilmemiştir. `HAND_YEARS` tablosunun 144 hücresi ve birim testlerindeki el yılları da ilk koşuda geçmiştir. Plan yazılırken yedi örnek, motoru yazmayan ayrı bir ajana yalnızca kural metni ve `input` bölümleri verilerek (Rust kaynağı okunmadan, motor çalıştırılmadan) yeniden türettirilmiştir: 7/7 uyuştu, `derivation` adımlarında yanlış burç, sayım, ölçüt numarası ya da birikimli yıl bulunmadı; tek bulgu bir ifade belirsizliğiydi ("ilk örnekteki gibi" yerine örneğin adı yazıldı). Bu, Task 7 Step 5'in yerine geçmez: o adım yürütme sırasında, yürütücüden bağımsız bir gözden geçirenle yinelenir. Yazım sırasında kodda düzeltilenler: bir belge yorumu "1. ve 2. seviye" diye başladığı için clippy onu liste maddesi saydı (`doc_lazy_continuation`); cümle "İlk iki seviye" olarak yazıldı. Şerit testlerinin ilk hâlinde bir beklenti yalnızca uzunluğu karşılaştırıyordu; on iki adın açık listesiyle değiştirildi. `STRIP_LEVELS` ve `PARENT_DEPTH` panelden `system.tsx`'e taşındı (içe aktarma döngüsü). İlk çevrimin 120 yıla ulaşamayacağı (D4) el hesabıyla (üst sınır 118) ve proptest'le (`< 120`) denetlendi; biçimsel bir kanıt yazılmadı. **Koşulmayanlar:** `pnpm tauri dev` ile gerçek uygulama ve gerçek veritabanı (Task 7 Step 4'ün **[insan]** maddeleri; özellikle gerçek motorla eş lord ayarının ve dasa yılının panellere yansıması); gerçek bir eski veritabanı dosyasının açılması (eski kayıt biçimi `store` testlerinde sentetik JSON ile sınandı); uzak CI (hiçbir şey gönderilmedi).
