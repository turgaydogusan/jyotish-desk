# Plan 15 — Alt proje 3.2: Jaimini çekirdeği (chara karaka, karakamsha, rashi drishti, arudha padalar, argala)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Jaimini çekirdeğini vermek: chara karaka sırası (sekizli ya da yedili şema), karakamsha, rashi drishti, Jaimini eş lordları, on iki arudha pada ve argala/virodhargala; tek modül komutu `compute_jaimini`, üç yeni pano paneli (Chara karaka, Arudha padalar, Argala), var olan üç panelde eklemeler (harita: arudha işaretleri; Bakışlar: rashi türü; graha tablosu: karaka sütunu) ve 3.1'de tanımlanan iki `advanced` ayarının (`karaka_scheme`, `jaimini_co_lords`) hesapta kullanımı ile ayar penceresindeki denetimleri.

**Architecture:**
- **Saf çekirdek, ince toplayıcı (spec K3).** `jyotish-core`'a `jaimini` modülü eklenir (beş dosya: `rashi`, `karaka`, `lords`, `arudha`, `argala`). Hesap fonksiyonları yalnızca `Positions` ve iki ayarı görür; el örnekleri ephemeris olmadan `Positions::synthetic` ile kurulur. `compute_jaimini` temel haritayı hesaplayıp konumlara indirger ve raporu kurar.
- **Tek komut, tek rapor (spec K1, K2).** `compute_jaimini(input: ChartInput) -> JaiminiReport`, Plan 14'ün `module_commands.rs` kalıbıyla: motor iş parçacığı, `blocking` havuzu, `CommandError`. `Chart` ve `open_client` değişmez; rapor saklanmaz. Doğum saati bilinmiyorsa padalar Rust'ta `None` döner ve argala satırları Koç'tan sıralanır.
- **Arayüz hesap yapmaz.** Altı tüketici (üç yeni panel, harita işaretleri, rashi matrisi, karaka sütunu) aynı raporu ortak `useJaimini` kancasıyla okur; kanca Plan 14'ün `useModule`'ünü sarar. Yeni paneller `MODULE_PANELS`'e eklenir ve komut paletinden açılıp çıkarılır (K12).
- **Ayarlar yeni kalıcı biçim getirmez.** `CalcSettings` sürüm 2 ve `advanced` bloğu 3.1'den beri vardır; bu plan iki alanı hesapta okur ve ayar penceresine iki denetim ekler. SQLite şeması, `AppPrefs` ve `ViewState` sürümleri değişmez.
- **Testler:** el örnekleri (`tests/golden/modules/jaimini/`, 8 dosya, kaynak yalnızca `hand`), proptest değişmezleri, rashi drishti için elle yazılmış ikinci tablo, komut gövdesinin gerçek motorla entegrasyon testi (v1 haritasının D9'uyla çapraz denetim, bütçe ölçümü), Vitest (paneller, ayar penceresi, harita çizimi), Playwright e2e (palet → paneller → ortak seçim → ayar).

**Tech Stack:** Rust 1.93+ (stable), Tauri 2.12, tauri-specta 2.0.0-rc.29, serde, proptest, React 19, TypeScript 7, Vitest 5, Playwright 1.65.1. **Yeni Rust ya da npm bağımlılığı yok.**

**Spec:** `docs/superpowers/specs/2026-11-26-alt-proje-3-ileri-moduller-design.md` (commit `32cc752`; Plan 14'ün geri bildirimi işlenmiş hâli): §2 (K1–K12), §5 (Alt proje 3.2), §13 (ortak test stratejisi), §14 (gizlilik), §15 (bütçeler), Ek A (adlandırma). §6 (3.3 Chara dasha) bu planın `jaimini_lord` ve `counts_forward` fonksiyonlarını tüketir (bkz. "Sonraki alt projelere verilen API"). UI spec: `2026-11-24-ui-design.md` §3.3, §3.4, §5, §6, §8.

**Ön koşul:** Taban commit `32cc752` (Plan 14 ve son inceleme düzeltmeleri `main`'de). `main` bu commit'ten ilerlediyse Task 1 adım 1 tabanları yeniden ölçer ve sayaç zinciri o ölçümden yürütülür (farklar aynıdır). Plan 14'ün elle denetim kapısının yalnızca insana ait maddeleri sahibine devredilmiştir; bu planı engellemez.

## Kararlar

Sahibi kararları devretti; muhafazakâr seçenek seçildi ve her karar bir satır gerekçeyle yazıldı. "Spec boşluğu" diye işaretlenenler spec'te düzeltilmesi gereken yerlerdir ve belgenin sonundaki "Spec'e geri bildirim" bölümünde toplanmıştır. D16, D22 ve sayaç zinciri bu plan yazılırken deponun bir kopyasında ölçülmüştür (macOS 27.2, Rust 1.99).

- **D1 — Modül `crates/jyotish-core/src/jaimini/` dizinidir: `rashi.rs` (burç türü, sayım yönü, rashi drishti), `karaka.rs` (chara karaka, karakamsha), `lords.rs` (eş lordlar), `arudha.rs`, `argala.rs` ve `mod.rs` (`JaiminiReport`, `jaimini_report`, `compute_jaimini`). `method = "jaimini.core_v1"`.** Gerekçe: beş kural birbirinden bağımsız okunur ve sınanır; tek dosya 1 400 satırı geçerdi. Yöntem kimliği spec K7'nin "her modül sonucu `method` taşır" kuralıdır.
- **D2 — Karaka sırası rasi çerçevesindeki burç içi dereceyi (`GrahaPoint.deg_in_sign`) kullanır.** Gerekçe: K5 "burca dayalı her kural rasi çerçevesini kullanır" der; burç içi derece burcun parçasıdır ve haritada gösterilen dereceyle aynı olmalıdır. Tropikal-rasi kipinde sıra tropikal dereceyle kurulur. **Spec boşluğu 1.**
- **D3 — Eşitlik: sıradaki iki komşunun etkin derece farkı `≤ 1″` (`KARAKA_TIE_DEG = 1/3600`) ise ikisi de `tie: true` taşır. Sıra `f64::total_cmp` ile büyükten küçüğe, tam eşitlikte Güneş…Rahu doğal sırasıyla kurulur (kararlı sıralama). Rahu tam 0°'deyse etkin derecesi 30'dur: `effective_deg` Rahu için `(0, 30]`, diğerleri için `[0, 30)` aralığındadır.** Gerekçe: spec "1″ içinde eşitse sonuç `tie: true` taşır" der ama hangi kaydın taşıdığını ve sınırın dahil olup olmadığını söylemez; iki komşuyu da işaretlemek astrologa hangi çiftin yakın olduğunu gösterir. **Spec boşluğu 2.**
- **D4 — Rashi drishti formülle hesaplanır (yürük: +4, +7, +10; sabit: +2, +5, +8; çift doğalı: +3, +6, +9 burç) ve testte elle yazılmış on iki satırlık tabloyla karşılaştırılır. `graha_rashi_drishti` her zaman 27 kayıttır: `Graha::ALL` sırasında dokuz graha (düğümler dahil), her biri için üç hedef burç kendi burcundan zodyak sırasıyla.** Gerekçe: spec §13 katman 3 (bağımsız ikinci yazım); sabit sıra arayüzün ve AI paketleyicisinin listeyi dizinle okumasını sağlar. Spec kayıt sayısını ve sırasını vermez. **Spec boşluğu 3.**
- **D5 — Jaimini sayım yönü bu planda tanımlanır: `counts_forward(sign)` (Koç, Boğa, İkizler, Terazi, Akrep, Yay ileri; diğer altısı geri) ve `directed_count(from, to)`.** Gerekçe: eş lord seçiminin 5. ölçütü "burcun sayım yönünde (3.3)" der; tanım §6'dadır ama 3.2 ona muhtaçtır ve 3.3, 3.2'ye bağımlıdır (ters yön). 3.3 aynı fonksiyonları kullanır. **Spec kusuru 4.**
- **D6 — Eş lord seçiminin 3. ölçütü yalnızca graha lordun (Mars ya da Satürn) durumuna bakar: yücelme burcundaysa o seçilir; değilse ve düşüş burcundaysa düğüm seçilir; ikisi de değilse ölçüt ayırmaz. Düğümlerin yücelme ve düşüş burcu yoktur.** Gerekçe: spec "Düğümlere bu adım uygulanmaz" der; her eş lord çiftinde bir düğüm olduğundan cümle harfiyen okunursa ölçüt hiçbir zaman çalışmaz ve §5 "Test"teki "altı adımın her birinin belirleyici olduğu durumlar" koşulu sağlanamaz. Seçilen okuma cümlenin öncesiyle ("yücelme burcunda olan; ikisi de değilse, düşüş burcunda olmayan") tutarlıdır. **Spec kusuru 5.**
- **D7 — Diğer ölçütlerin ayrıntısı: 1. ölçüt yalnızca tam olarak biri burcun kendisindeyken ayırır (ikisi de içerideyse ya da ikisi de dışarıdaysa sonraki ölçüte geçilir). 2. ölçütte "birlikte olduğu graha" dokuz grahadan o burcun iki eş lordu dışındakilerdir (diğer düğüm sayılır). 5. ölçütte uzaklık, söz konusu burçtan lordun burcuna o burcun yönünde kapsayıcı sayımdır. `co_lord_choice(sign, positions) -> Option<CoLordChoice { lord, step }>` seçimi ve belirleyici ölçütü verir (el örnekleri `step`'i de sınar); `stronger_co_lord(sign, positions) -> Graha` spec'teki imzadır; `jaimini_lord(sign, positions, co_lords: bool) -> Graha` ayarı uygular.** Gerekçe: `step` rapora girmez (AI sözleşmesi spec'teki alanlarla sınırlıdır) ama hangi ölçütün belirlediği test edilebilir olmalıdır. **Spec boşluğu 13.**
- **D8 — `ArudhaPada.lord`, evin hesapta kullanılan lordudur (Akrep ve Kova evlerinde seçilen eş lord); padanın düştüğü burcun lordu değildir. `arudha_of(sign, lord_sign) -> (Sign, bool)` tek evin kuralıdır.** Gerekçe: spec alanı adlandırır ama anlamını yazmaz; hesapta kullanılan lord, eş lord seçiminin sonucunu panelde görünür kılar. **Spec boşluğu 6.**
- **D9 — Karakamsha, Atmakaraka'nın rasi çerçevesindeki konumundan v1'in `varga_position(Varga::D9, …)` kuralıyla bulunur; `houses_in_d1` ve `houses_in_d9` `Graha::ALL` sırasındadır. Doğum saati bilinmiyorsa "yaklaşık" işareti arayüzdedir (uyarı satırı; Ay'ın derecesinde ve karakamsha burcunda "≈").** Gerekçe: `JaiminiReport`'ta yaklaşıklık alanı yoktur, K1 `ChartWarning`'e yeni üye eklemeyi dışlar (`Chart` tipi değişirdi) ve v1 panelleri Ay'ın yaklaşıklığını aynı yolla gösterir. Girdideki `time_known` AI paketleyicisinin de elindedir. **Spec boşluğu 7.**
- **D10 — Argala: çift sırası `ARGALA_PAIRS = [(2, 12), (4, 10), (11, 3), (5, 9)]`; iki listede dokuz graha da sayılır ve `Graha::ALL` sırasındadır; durum yalnızca sayıya bakar (argala tarafı boşsa `None`, engel tarafı dolu olsa da). 3. ev kötücül argalası `nature` fonksiyonunu kullanır (K6): küçülen Ay ve kötücülle birlikte olan Merkür sayılır. Satırlar lagna burcundan, lagna yoksa Koç'tan zodyak sırasıyladır.** Gerekçe: spec §5; K6 "modüller kendi listesini tutmaz".
- **D11 — `JaiminiReport.warnings` yalnızca temel haritanın `LowPrecision` uyarısını taşır; `PlacidusPolarFallback` ve `SunriseUnavailable` bu modülü ilgilendirmez ve taşınmaz. Saf `jaimini_report` boş liste döner; toplayıcı doldurur.** Gerekçe: spec alanı tanımlar ama içeriğini yazmaz; konum duyarlılığı karaka sırasını etkileyebilir, ev sistemi ve gün doğumu etkilemez. **Spec boşluğu 7.**
- **D12 — `JaiminiReport` spec §5'teki alanlarla birebirdir. Sabit uzunluklu diziler (`[ArudhaPada; 12]`, `[ArgalaRow; 12]`, `[ArgalaPair; 4]`, `[[bool; 12]; 12]`, `[u8; 9]`) TS'te demet olur; `effective_deg` `number | null` olur (specta `f64`'ü böyle yazar) ve paneller `null`'u çizgiyle gösterir.** Gerekçe: rapor AI paneli için sözleşmedir.
- **D13 — Yeni `CommandError.code`, yeni bağımlılık, yeni SQLite göçü ve yeni ayar alanı yoktur.** `compute_jaimini` yalnızca `compute_chart`'ın hatalarını verir; hepsi çevrilidir. Gövde `client_commands::blocking` ile koşar: panik `task_failed` olur ve iletisi taşınmaz (§14).
- **D14 — Ayar penceresine "İleri modül ayarları" grubu eklenir: "Chara karaka şeması" seçicisi (sekiz / yedi) ve "Jaimini eş lordları" kutusu. Fark işareti grubun başlığındadır (`advanced` tek fark alanıdır: Plan 14 D1). İki denetim de bu plandadır.** Gerekçe: K7 denetimleri "3.2 ve 3.3"e bırakır ama hangisinin nerede olduğunu söylemez; `jaimini_co_lords` 3.2'de arudha hesabında kullanılır, bu yüzden ikisi de burada görünür olmalıdır (kullanılan ama değiştirilemeyen ayar olmaz). **Spec boşluğu 8.**
- **D15 — Üç yeni panel `MODULE_PANELS`'e eklenir: `jaimini_karakas` (5 × 14, ad "Chara karaka"), `arudha` (4 × 12, "Arudha padalar"), `argala` (6 × 9, "Argala"). Rol adları ve kısaltmaları (AK Atmakaraka … DK Darakaraka) ile pada etiketleri (AL, A2 … A11, UL) üç dilde aynıdır ve `panels/labels.ts`'te sabittir.** Gerekçe: K11 (klasik terim aynı transliterasyonla), K12 (palet).
- **D16 — Her tüketici raporu kendi `useJaimini` çağrısıyla ister; paylaşılan önbellek yoktur (Plan 14 D13 sürer).** Gerekçe: ölçüm: `compute_jaimini` hata ayıklama derlemesinde ortanca 0,7–0,8 ms; beş tüketici açıkken beş çağrı 5 ms'nin altındadır. Önbellek bayatlama riski ve bir depo daha getirirdi. Kanca `enabled === false` iken (işaretler ve sütun kapalıyken, tür "graha" iken) komutu çağırmaz.
- **D17 — Argala panelinin `reference` ayarı `"lagna" | "selected"`'dır (varsayılan `lagna`). "Seçili burç" ortak seçimde sabitlenen öğenin burcudur: burç kendisi; ev, lagna burcundan sayılan burç (lagna yoksa haritanın çizdiği gibi Koç'tan); graha, haritadaki D1 burcu. Panel ayrıca on iki burç düğmesi gösterir. Saat bilinmiyorsa ve referans lagnaysa tablo yerine bir satır açıklama çıkar.** Gerekçe: spec "referans (lagna / seçili burç)" der; harita hücreleri ortak seçime ev olarak girer (UI spec §3.4), yani "seçili burç" tanımsızdır. Ev → burç eşlemesi Ashtakavarga panelindeki gösterim eşlemesiyle aynıdır (C4'ün kapsamı dışında). **Spec boşluğu 9.**
- **D18 — `chart` panelinin `arudha_marks` ayarı yalnızca D1'de ve doğum saati biliniyorken çizer; diğer durumlarda kutu kapalıdır ve nedeni kutuya `aria-describedby` ile bağlıdır. İşaretler etkileşimsiz kısa metinlerdir ("AL", "A2" … "UL"), gezegen ve transit etiketleriyle aynı yerleşim hesabını paylaşır ve ekran okuyucu için ayrı bir liste olarak okunur.** Gerekçe: padalar rasi evlerinden hesaplanır (K4) ve lagnaya bağlıdır; transit halkasının D1 kuralıyla aynı kalıp. Spec hangi vargada çizileceğini söylemez. **Spec boşluğu 10.**
- **D19 — Bakışlar paneline `kind: "graha" | "rashi"` ayarı ("Tür" seçicisi) eklenir. `rashi` türü dokuz graha × dokuz graha matrisidir (`graha_rashi_drishti`'den); virupa değeri olmadığı için "Değer" seçicisi çizilmez. Araç çubuğu yükleme ve hata sırasında da görünür (tür hata durumunda değiştirilebilsin).** Gerekçe: Plan 14 D12 tür seçicisini bu plana bırakmıştı; matris biçimi panelin mevcut biçimidir.
- **D20 — Graha tablosunun "Karaka" sütunu `settings.columns` listesinde değil, ayrı bir `settings.karaka: boolean` ayarındadır (varsayılan kapalı).** Gerekçe: baskı tablosu (`print/model.ts`) `GRAHA_COLUMNS` listesinin tamamını basar; sütunu o listeye eklemek "kısa rapor" düzenine sütun eklerdi (K10 baskı düzenlerini dondurur). **Spec boşluğu 11.**
- **D21 — El örnekleri `tests/golden/modules/jaimini/` altında sekiz dosyadır ve üç görevde büyür: Task 1 karaka ve karakamsha, Task 2 eş lord ve pada, Task 3 argala bölümünü ekler. Girdi `advanced` bloğunu taşır (şema ve eş lord ayarı). `co_lords` beklentisi ayar kapalıyken de yazılır (seçim kuralı ayardan bağımsız sınanır). Ayrı bir test, spec'in zorunlu durumlarının örneklerde bulunduğunu sabitler.** Gerekçe: spec §5 "her biri karaka sırasını, on iki padayı, karakamshayı ve en az iki referans için argala listelerini verir" der; görev başına bölüm eklemek her görevin kendi gözden geçirmesinde kendi kuralının yeniden türetilmesini sağlar. Spec §13'ün şemasında ayar alanı yoktur. **Spec boşluğu 12.**
- **D22 — `pnpm build` Task 8'den itibaren Vite'ın 500 kB parça uyarısını basar (ölçüm: 493 kB → 506 kB); yapılandırma değiştirilmez ve kapılar `✓ built` satırını arar.** Gerekçe: uyarı bilgilendiricidir; paket yerel diskten yüklenir. Eşiği yükseltmek ya da panelleri tembel yüklemek (yükleme durumlarını değiştirir) ayrı bir işin konusudur.
- **D23 — Yeni görsel temel (PNG) ve yeni baskı düzeni yoktur (K10).** Gerekçe: yeni paneller metin tablolarıdır; arudha işaretleri varsayılan olarak kapalıdır, bu yüzden mevcut on görsel temel ve üç harita anlık görüntüsü değişmez (Task 11 doğrular).


## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni bağımlılık yok: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `app/package.json`, `app/pnpm-lock.yaml` ve `app/vite.config.ts` değişmez; `cargo deny check` Task 11'de `advisories ok, bans ok, licenses ok, sources ok` verir.
- **Yasaklı ad kuralı (v1 spec §10 "İsim kuralları", ilk madde; ileri modüller spec'i Ek A):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Kural harf dizisi düzeyindedir: Türkçe sözcüklerin içinde de geçemez (yerine ör. "diğer", "durur", "geri", "artan", "sürer" yazılır). **Bu planda özel dikkat:** "karaka" sözcüğüne `-lar` çoğul eki ya da `L` ile başlayan bir ad eklenince (bitişik yazımda ya da camelCase'te) yasak dizi oluşur. Çoğul yerine "karaka sırası", "karaka rolleri", "sekiz karaka" yazılır; kod kimliklerinde `karaka` sözcüğünü `l` harfi izlemez (ör. `KARAKA_ROLES`, `karaka_roles`, `karakaOn`). Eş karakası "Dara karaka"dır (`KarakaRole::Dara`, Ek A). Her commit adımı eklenen satırları, dosya adlarını ve commit mesajını `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile tarar; çıktı boş olmalıdır (C13). Bu belgede de eşleşme yoktur.
- **Clean-room ve yorum metni yok:** yöntemler klasik kuraldan özgün olarak uygulanır; başka ürünün adı, düzeni, metni ya da çıktısı kullanılmaz; kaynak gösterimi geneldir ("BPHS, pada bölümü", "Jaimini Upadesha Sutraları") ve alıntı yapılmaz. Paneller hesaplanmış olguları ve klasik etiketleri gösterir ("AK", "A7", "geçerli"); bir karaka rolünün, padanın ya da argalanın ne anlama geldiğini anlatan metin eklenmez. Argala durumları olgu etiketidir (sayı karşılaştırmasının sonucu), hüküm değildir.
- **Evler rasi evidir (K4), rasi çerçevesi `settings.zodiac`'ı izler (K5), doğal sınıf tek fonksiyondur (K6).** Padalar ve argala evleri burç sayımıdır; `CalcSettings.house_system` bunları etkilemez. 3. ev kötücül argalası `nature` fonksiyonunu çağırır; modül kendi iyicil/kötücül listesini tutmaz.
- **Doğum saati bilinmiyorsa lagnaya bağlı her çıktı Rust'ta `None` döner (K2):** `JaiminiReport.arudhas`. Argala lagnaya bağlı değildir; yalnızca satır sırası Koç'tan başlar. Karaka sırası ve karakamsha döner; arayüz "yaklaşık" işaretini koyar (D9).
- **El örnekleri (§13, K9):** `source` yalnızca `hand`'dir; beklenen değer kuraldan elle türetilir ve `derivation`'da adım adım yazılır. Bir el örneği motorla uyuşmazsa önce kural yeniden okunur; **beklenen değer motor çıktısına bakarak düzeltilmez.** Gözden geçirme döngüsünün zorunlu maddesi: gözden geçiren, Task 1, 2 ve 3'te eklenen `derivation` adımlarını kuraldan yeniden hesaplar.
- Kapılar: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `pnpm --dir app test`, `pnpm --dir app build` her görev sonunda yeşil; `pnpm --dir app e2e` Task 10 ve 11'de. `tsc` `strict`. `pnpm build` çıktısında aranan satır `✓ built`'tir (D22).
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez). Node 24+, pnpm 11.8.0. Depo kısa bir yolda durmalıdır: Swiss Ephemeris 242 bayttan uzun ephemeris yolunu reddeder (çalışma ağacı uzun bir yoldaysa `/tmp` altında kısa bir yola klonlanır).
- **Çalışan `pnpm tauri dev` durdurulmaz** (port 1420); e2e 1421'i kullanır. `pkill -f` kullanılmaz. Üretilen `.app` çalıştırılmaz.
- Yalnızca macOS hedeflenir. Yorum metni ve AI paneli bu planın dışındadır. Chara dasha Alt proje 3.3'tür.
- **Tabanlar (`32cc752`):** `jyotish-core` lib 139, golden 8, `modules_golden` 3; `store` lib 84 (+1 `#[ignore]`); `jyotish-desk` lib 46, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `Test Files  65 passed (65)`, `Tests  571 passed (571)`; e2e 13; görsel 10 (yerel, darwin); yayın betiği testleri 14.
- **Sayaç zinciri** (her satır görev bitince; boş hücre "önceki satırla aynı"). Bu plan yazılırken `32cc752`'nin bir kopyasında her görev ayrı commit olarak uygulanıp ölçülmüştür.

  | Görev | `jyotish-core` lib | `modules_golden` | `jyotish-desk` `modules` | Vitest dosya / test | e2e |
  |---|---|---|---|---|---|
  | Taban | 139 | 3 | 1 | 65 / 571 | 13 |
  | Task 1 | 151 | 5 | | | |
  | Task 2 | 164 | | | | |
  | Task 3 | 172 | | | | |
  | Task 4 | | | 1 | 65 / 571 | |
  | Task 5 | | | | 65 / 574 | |
  | Task 6 | | | | 66 / 585 | |
  | Task 7 | | | | 67 / 599 | |
  | Task 8 | | | | 68 / 608 | |
  | Task 9 | | | | 68 / 617 | |
  | Task 10 | | | | | 14 |

  Değişmeyenler: `jyotish-core` golden 8; `store` lib 84 (+1 ign.); `jyotish-desk` lib 46, `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2; görsel 10; yayın betiği testleri 14. `modules_golden` Task 1'de iki test kazanır (`jaimini_hand_fixtures`, `jaimini_fixtures_cover_the_required_cases`); Task 2 ve 3 aynı iki testi büyütür. `jyotish-desk` `modules` tek test olarak durur (süreçte tek SE örneği); Task 4 onu yeniden adlandırır ve Jaimini denetimlerini ekler.
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir:**
  - **C1** Gizlilik (spec §14): danışan adları, doğum verisi, not gövdeleri, yer bilgisi, dosya yolları ve `CommandError.params` loglanmaz; `no-console.test.ts` durur ve izin listesi büyümez. `ChartInput` ve `inputKey` doğum anı ve konumu taşır: loglanmaz, hata parametresine girmez. Hiçbir modül ağ isteği yapmaz; `JaiminiReport` veritabanına yazılmaz.
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu ya da adres almaz. `compute_jaimini`'nin tek bağımsız değişkeni `ChartInput`'tur (dizgi alanı yoktur); `api.test.ts`'in dizgi bağımsız değişken listesi değişmez.
  - **C3** Göç V1 donmuştur; `crates/store/src/schema.rs` değişmez. `CalcSettings`, `AppPrefs` ve `ViewState` şema sürümleri değişmez (D13).
  - **C4** Arayüz astrolojik hesap yapmaz: karaka sırası, etkin derece, karakamsha, padalar, rashi drishti ve argala Rust'tan gelir. Derece biçimlemesi, sabitlenen evin burca eşlenmesi (D17) ve pada etiketinin evden kurulması ("A7") gösterimdir.
  - **C5/C17** Otomatik kayıtlar ve boşaltılmaları değişmez. **C6** Düzen ayrıştırıcıları (`parseLayoutState`, Rust `check_panel_layout`) değişmez; yalnızca panel kataloğu büyür.
  - **C7** Kısayollar yalnızca `shortcuts.ts`'tedir; bu plan kısayol eklemez. **C8** Her pencere `shell/Dialog`'u kullanır (bu plan pencere eklemez).
  - **C9** Renk yalnızca token'lardan gelir (hex yok) ve tek başına anlam taşımaz: eşitlik `=` ve metinle, istisna `*` ve dipnotla, rashi drishti `●` ve metinle, yaklaşık değer "≈" ve metinle, argala durumu sözcükle, vurgu `--select` zemin ve yazı ağırlığıyla gösterilir.
  - **C10** Üretilen dosyalar elle düzenlenmez: `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` Task 4'te komutla yeniden üretilir.
  - **C11** Görsel PNG temelleri yereldir; bu planda değişmeleri beklenmez (D23).
  - **C12** Her `CommandError.code` üç dilde çevrilidir; bu plan kod eklemez (D13). Her yeni i18n anahtarı üç dilde vardır (`i18n.test.ts` anahtar kümelerini ve yer tutucuları karşılaştırır).
  - **C13** Yasaklı ad taraması her commit'te; depo dışındaki ilerleme notları ve `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** Plan 07'nin gün doğumu toleransı değişmez. **C15** `alert`, `confirm`, `prompt`, `window.print()` ve `window.open()` çağrılmaz. **C16** ⌘ kısayolları değişmez.
- **Retro** her yerde düz `R`'dir ve Rahu ile Ketu'da gösterilmez (UI spec §2.3). Bu planın yeni panelleri retro göstermez; kural v1 panellerinde olduğu gibi sürer.
- **Erişilebilirlik:** WCAG AA. Tablolar `caption`, `scope="col"` ve `scope="row"` taşır; vurgu düğmeleri klavyeyle çalışır (`aria-pressed`); kısaltma gösteren düğmenin erişilebilir adı tam addır; seçicilerin ve kutuların görünür etiketi vardır; kapalı bir kutunun nedeni `aria-describedby` ile bağlıdır; yükleme iskeleti `aria-busy`, hata `role="alert"` ve "Yeniden dene" düğmesi taşır; haritadaki işaretler `aria-hidden`'dır ve ayrı bir metin listesi olarak okunur.
- Commit mesajları İngilizcedir ve deponun `Co-Authored-By:` satırıyla biter (yürütücünün oturum yönergesindeki satır). Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, `git tag`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Kod blokları `32cc752` ağacına göre yazılmış ve bu plan yazılırken deponun bir kopyasında koşulmuştur (Self-Review madde 5). Yeni dosyalar tam içerikle verilir ("**Yeni dosya:**"). Değişen dosyalar `diff` bloğuyla verilir: blok bir dosyaya konup depo kökünde `git apply <dosya>` ile ya da elle uygulanır; bağlam tutmazsa önceki görevin adımı denetlenir, elle uydurma yapılmaz. Bir dosyanın test ve uygulama değişiklikleri ayrı adımlarda ayrı bloklardır. Dosya ve diff blokları dört ters tırnakla çevrilidir (içlerinde üç ters tırnak geçebilir); komut blokları üç ters tırnaklıdır. `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` diff olarak verilmez; adımlardaki komutlar üretir.

## Sonraki alt projelere verilen API

Bu planın ürettiği ve 3.3–3.9'un tüketeceği adlar. Hepsi `jyotish_core`'dan dışa aktarılır; imzalar Task 1–3'te tam olarak verilir.

| API | Tüketen | Not |
|---|---|---|
| `jaimini_lord(sign, positions, co_lords) -> Graha`, `stronger_co_lord`, `co_lord_choice`, `co_lords(sign)`, `CoLordChoice` | 3.3 (chara dasha yılı: Akrep ve Kova'nın lordu) | "İki eş lord da burcun içindeyse 12 yıl" kuralı 3.3'tedir; `co_lords(sign)` iki lordu verir |
| `counts_forward(sign) -> bool`, `directed_count(from, to) -> u8` | 3.3 (dasha sırası ve yıl sayımı) | Burç grupları spec §6'daki listedir (D5) |
| `sign_kind(sign) -> SignKind`, `rashi_drishti_signs(sign) -> [Sign; 3]`, `rashi_drishti_matrix()`, `graha_rashi_drishti(positions)` | 3.5 (burç bakışı isteyen yoga kuralı olursa), 3.8, 3.9 | |
| `chara_karakas(positions, scheme) -> Vec<CharaKaraka>`, `karaka_roles(scheme)`, `KarakaRole`, `karakamsha(positions, atmakaraka)`, `navamsa_sign(point)` | 3.5 (karaka temelli yogalar), 3.9 (`CharaKaraka` koşulu) | |
| `arudha_padas(positions, co_lords) -> Option<[ArudhaPada; 12]>`, `arudha_of(sign, lord_sign)` | 3.5, 3.9 (`ArudhaInSign` koşulu) | Lagna yoksa `None` |
| `argala_row(reference, positions)`, `argala_rows(positions)`, `ArgalaStatus` | AI paketleyicisi | |
| `jaimini_report(positions, &AdvancedSettings) -> JaiminiReport`, `compute_jaimini(eph, &ChartInput)`, `JAIMINI_METHOD` | AI paketleyicisi; 3.9'un hafif yolu saf fonksiyonu çağırır | Rapor AI sözleşmesidir |
| Arayüz: `modules/useJaimini.ts` (`useJaimini(opened, enabled?)`), `panels/PinButton.tsx`, `KARAKA_ROLES`, `arudhaLabel`, `ChartData.marks` / `ChartMark` | 3.3 (dasa panelleri), 3.5–3.9'un panelleri | `PinButton` yeni panellerin ortak vurgu düğmesidir |

## Dosya Yapısı (bu plan)

```
crates/jyotish-core/
  src/jaimini/mod.rs              JaiminiReport, jaimini_report, compute_jaimini (yeni)
  src/jaimini/rashi.rs            SignKind, counts_forward, rashi drishti (yeni)
  src/jaimini/karaka.rs           chara karaka, karakamsha (yeni)
  src/jaimini/lords.rs            eş lordlar: co_lord_choice, jaimini_lord (yeni)
  src/jaimini/arudha.rs           arudha padalar (yeni)
  src/jaimini/argala.rs           argala ve virodhargala (yeni)
  src/lib.rs                      modül kaydı ve dışa aktarımlar
  src/proptests.rs                jaimini_properties
  tests/modules_golden.rs         Input.advanced; jaimini el örnekleri ve kapsam testi
tests/golden/modules/
  README.md                       advanced bloğu; jaimini satırı
  jaimini/*.json                  8 el örneği (yeni)
app/src-tauri/
  src/module_commands.rs          compute_jaimini
  src/lib.rs                      komut kaydı
  tests/modules.rs                Jaimini entegrasyonu ve bütçe
  tests/ui_fixture.rs             fikstüre Jaimini raporu
app/ui/
  bindings.ts                     üretilir
  test/fixtures/delhi.json        üretilir (jaimini)
  test/fixture.ts                 UiFixture.jaimini
  api.ts, api.test.ts             computeJaimini
  settings/{model.ts,SettingsDialog.tsx,settings.css} (+test)   ileri modül ayarları grubu
  modules/useJaimini.ts           ortak kanca (yeni)
  layout/model.ts (+test)         PANEL_KINDS, MODULE_PANELS += üç tür
  panels/PinButton.tsx            ortak vurgu düğmesi (yeni)
  panels/labels.ts                KARAKA_ROLES, arudhaLabel
  panels/JaiminiKarakasPanel.tsx (+test)   (yeni)
  panels/ArudhaPanel.tsx (+test)           (yeni)
  panels/ArgalaPanel.tsx (+test)           (yeni)
  panels/ChartPanel.tsx (+test)            arudha_marks
  panels/DrishtiPanel.tsx (+test)          kind: graha | rashi
  panels/GrahaTablePanel.tsx (+test)       karaka sütunu
  panels/{registry.ts,panels.css,panels.test.tsx}
  chart/{types.ts,ChartSvg.tsx,chart.css} (+test)   ChartMark, burç işaretleri
  i18n/{tr,en,fr}.ts              panel adları, jaimini, arudha, argala, drishti, chart, settings, table
app/e2e/{ipc.ts,desk.spec.ts}     compute_jaimini taklidi; Jaimini akışı
```

### Kapsam

| Spec maddesi | Yer |
|---|---|
| §5 Chara karaka (sekizli ve yedili şema, Rahu'nun etkin derecesi, eşitlik) | Task 1 (D2, D3) |
| §5 Rashi drishti (burç matrisi, grahaların bakışı) | Task 1 (D4) |
| §5 Karakamsha (D1 ve D9 görünümü) | Task 1 (D9) |
| §5 Eş lordlar (altı ölçüt, `jaimini_co_lords`) | Task 2 (D5–D7) |
| §5 Arudha pada (A1…A12, istisna) | Task 2 (D8) |
| §5 Argala (dört çift, durum, 3. ev kötücül argalası) | Task 3 (D10) |
| §5 `JaiminiReport`, `compute_jaimini`; K1, K2, K8 | Task 3, 4 (D11–D13) |
| K7 İki `advanced` ayarının hesapta kullanımı ve ayar penceresindeki denetimleri | Task 3 (hesap), Task 5 (D14) |
| §5 Paneller: `jaimini_karakas`, `arudha`, `argala`; K11, K12 | Task 6, 7, 8 (D15–D17) |
| §5 `chart` paneli `arudha_marks` | Task 7 (D18) |
| §5 `drishti` paneli `rashi` türü; `graha_table` "karaka" sütunu | Task 9 (D19, D20) |
| §5, §13 El örnekleri (en az 8), değişmezler, ikinci yazım, uçtan uca | Task 1–4 (D21) |
| §13 Panel durumları: boş, yükleniyor, hata, dolu, "saat bilinmiyor" | Task 6, 7, 8, 9 |
| §14 Gizlilik | C1; Task 11 taramaları |
| §15 `compute_jaimini` ≤ 10 ms | Task 4 (ölçüm testi) |
| Uçtan uca akış | Task 10 |
| Son doğrulama ve elle denetim | Task 11 |

---


### Task 1: Çekirdek: burç türleri ve rashi drishti, chara karaka, karakamsha; Jaimini el örnekleri

Spec §5 "Chara karaka", "Rashi drishti", "Karakamsha"; §13 katman 1, 2 ve 3. Hiçbir mevcut davranış değişmez; yalnızca yeni modül, el örnekleri ve testler eklenir. Sekiz el örneği bu görevde karaka ve karakamsha bölümleriyle yazılır; Task 2 ve Task 3 aynı dosyalara kendi bölümlerini ekler (D21).

**Files:**
- Create: `crates/jyotish-core/src/jaimini/{mod,rashi,karaka}.rs`, `tests/golden/modules/jaimini/*.json` (8 dosya)
- Modify: `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/proptests.rs`, `crates/jyotish-core/tests/modules_golden.rs`, `tests/golden/modules/README.md`
- Test: `rashi.rs` ve `karaka.rs` içindeki `mod tests`, `proptests.rs` (`jaimini_properties`), `tests/modules_golden.rs`

**Interfaces:**
- Consumes: `Positions` (`grahas`, `point`, `sign_of`, `occupants`, `synthetic`), `GrahaPoint { graha, rasi_sign, deg_in_sign, … }`, `Graha` (`Graha::ALL`: Güneş…Satürn, Rahu, Ketu), `Sign::{ALL, index, from_index, house_from}`, `SignAspect { graha, target, occupants }` (Plan 14), `KarakaScheme::{Eight, Seven}`, `AdvancedSettings`, `varga::{varga_position, Varga, is_vargottama}`; `modules_golden.rs` düzeneği (`Fixture<X>`, `Input`, `load`); `foundation_properties::positions()` üreteci.
- Produces (hepsi `jyotish_core`'dan dışa aktarılır):
  - `pub enum SignKind { Movable, Fixed, Dual }`, `pub fn sign_kind(sign: Sign) -> SignKind`
  - `pub fn counts_forward(sign: Sign) -> bool`, `pub fn directed_count(from: Sign, to: Sign) -> u8` (1..=12; `from`'un yönünde kapsayıcı sayım)
  - `pub fn rashi_drishti_signs(from: Sign) -> [Sign; 3]`, `pub fn rashi_drishti_matrix() -> [[bool; 12]; 12]` (`[from][to]`), `pub fn graha_rashi_drishti(positions: &Positions) -> Vec<SignAspect>` (27 kayıt)
  - `pub enum KarakaRole { Atma, Amatya, Bhratri, Matri, Pitri, Putra, Gnati, Dara }`, `pub const EIGHT_KARAKA_ROLES: [KarakaRole; 8]`, `pub const SEVEN_KARAKA_ROLES: [KarakaRole; 7]`, `pub fn karaka_roles(scheme: KarakaScheme) -> &'static [KarakaRole]`, `pub const KARAKA_TIE_DEG: f64`
  - `pub struct CharaKaraka { pub role: KarakaRole, pub graha: Graha, pub effective_deg: f64, pub tie: bool }` (`Copy`), `pub fn chara_karakas(positions: &Positions, scheme: KarakaScheme) -> Vec<CharaKaraka>`
  - `pub struct Karakamsha { pub atmakaraka: Graha, pub sign: Sign, pub houses_in_d1: [u8; 9], pub houses_in_d9: [u8; 9] }`, `pub fn navamsa_sign(point: &GrahaPoint) -> Sign`, `pub fn karakamsha(positions: &Positions, atmakaraka: Graha) -> Karakamsha`
  - El örneği düzeneği: `Input.advanced: Option<AdvancedSettings>`; `JaiminiExpected { karakas, karakamsha }`; `MIN_JAIMINI = 8`.

- [ ] **Step 1: Tabanları ölç**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
git log -1 --format=%h
git status --short
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `git status --short` boş; `jyotish-core` lib `139 passed`, golden `8 passed`, `modules_golden` `3 passed`; `store` lib `84 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `46 passed`, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `65 passed (65)`, `571 passed (571)`. Commit `32cc752` değilse ölçülen sayılar ilerleme notlarına taban olarak yazılır ve zincir oradan yürütülür. (Vitest'te `NotesPanel.test.tsx › New note adds an empty note…` yük altında seyrek olarak düşebilir; bu plandan bağımsızdır, yeniden koşulur ve ilerleme notlarına yazılır.)

- [ ] **Step 2: El örneklerini yaz (kaynak: `hand`)**

Sekiz sentetik konum seti. Her biri kural adımlarını `derivation`'da taşır; beklenen değerler bu adımlardan elle çıkarılmıştır (motor koşulmadan önce). Konumlar sentetiktir (astronomik olarak olası olmaları gerekmez); Rahu ile Ketu her sette tam karşılıklıdır. Derece değerleri navamsa parça sınırlarına (3°20′'nin katları) denk getirilmemiştir. Bu görevde her dosya beş türetme adımı taşır: "Burçlar", "Karaka", "Karakamsha", "D9" ve "D1 evleri … D9 evleri".

| Dosya | Lagna | Şema | Bu görevde kapsadığı | Task 2 ve 3'te kapsayacağı |
|---|---|---|---|---|
| `rahu-atmakaraka-arudha-exceptions` | Koç | sekiz | Rahu Atmakaraka (etkin 29°) | lordu 1., 4., 7., 10. burçta olan istisnalar; eş lord 4. ölçüt (çift doğalı > yürük) |
| `tie-within-one-arcsecond-seven-scheme` | Yengeç | yedi | 0,72″ farkla eşitlik (iki kayıt işaretli), 1,44″ farkla eşit olmayan komşu; Pitri rolü yok | eş lord 2. ve 5. ölçüt (geri sayım); engelli ve çekişmeli argala |
| `atmakaraka-vargottama-co-lords-off` | Akrep | sekiz | Atmakaraka vargottama (karakamsha = rasi burcu) | eş lord ayarı devre dışı (padalar Mars ve Satürn'le); seçim kuralında 2. ve 3. ölçüt |
| `time-unknown-third-house-malefics` | yok | sekiz | lagnasız harita | padalar `null`; eş lord 1. ve 2. ölçüt; 3. ev kötücül argalası (küçülen Ay sayılır) |
| `co-lord-exaltation-and-debilitation` | Aslan | sekiz | — | eş lord 3. ölçüt iki yönde (Mars yücelmiş; Satürn düşmüş) |
| `co-lords-in-the-same-sign` | Kova | sekiz | — | eş lord 6. ölçüt (ikisi de burçta; ikisi de aynı başka burçta) |
| `node-in-its-sign-and-sign-kind` | İkizler | yedi | yedili şema | eş lord 1. ölçüt (düğüm burçta); 4. ölçüt (sabit > yürük) |
| `co-lord-company-and-forward-count` | Başak | sekiz | — | eş lord 2. ve 5. ölçüt (ileri sayım); dört çifti de boş argala satırı |

Navamsa kuralı (v1 `varga` modülüyle aynı; `rule_ref`'te de yazılıdır): her burç 3°20′'lik dokuz parçaya bölünür; ateş burçları (Koç, Aslan, Yay) Koç'tan, toprak burçları (Boğa, Başak, Oğlak) Oğlak'tan, hava burçları (İkizler, Terazi, Kova) Terazi'den, su burçları (Yengeç, Akrep, Balık) Yengeç'ten sayılır; `k`. parçadaki nokta başlangıç burcundan `k`. burçtadır.

**Yeni dosya:** `tests/golden/modules/jaimini/atmakaraka-vargottama-co-lords-off.json`

````json
{
  "id": "atmakaraka-vargottama-co-lords-off",
  "source": "hand",
  "rule_ref": "BPHS, karaka, pada ve argala bölümleri; Jaimini Upadesha Sutraları (eş lordlar, rashi sayımı); navamsa: her burç dokuz eşit parça, ateş burçları Koç'tan, toprak Oğlak'tan, hava Terazi'den, su Yengeç'ten sayılır",
  "derivation": [
    "Burçlar: Güneş 319° Kova 19°, Ay 126° Aslan 6°, Mars 43° Boğa 13°, Merkür 295° Oğlak 25°, Jüpiter 358° Balık 28°, Venüs 333° Balık 3°, Satürn 202° Terazi 22°, Rahu 106° Yengeç 16°, Ketu 286° Oğlak 16°. Lagna 220° Akrep.",
    "Karaka (sekizli şema): Rahu'nun etkin derecesi 30 − 16 = 14. Büyükten küçüğe: Jüpiter 28, Merkür 25, Satürn 22, Güneş 19, Rahu 14, Mars 13, Ay 6, Venüs 3. Eşitlik yok.",
    "Karakamsha: Atmakaraka Jüpiter Balık 28°. Su burcu Yengeç'ten sayılır; 28° dokuzuncu parçadadır (26°40′–30°): Yengeç'ten 9. burç Balık. Navamsa burcu rasi burcuyla aynı: Atmakaraka vargottamadır.",
    "D9: Güneş Kova 19° (hava, Terazi'den; 6. parça 16°40′–20°): Balık. Ay Aslan 6° (ateş, Koç'tan; 2. parça 3°20′–6°40′): Boğa. Mars Boğa 13° (toprak, Oğlak'tan; 4. parça 10°–13°20′): Koç. Merkür Oğlak 25° (toprak; 8. parça 23°20′–26°40′): Aslan. Jüpiter: Balık. Venüs Balık 3° (su; 1. parça): Yengeç. Satürn Terazi 22° (hava; 7. parça 20°–23°20′): Koç. Rahu Yengeç 16° (su; 5. parça 13°20′–16°40′): Akrep. Ketu Oğlak 16° (toprak; 5. parça): Boğa.",
    "D1 evleri, Balık'tan (Güneş…Ketu): Kova 12, Aslan 6, Boğa 3, Oğlak 11, Balık 1, Balık 1, Terazi 8, Yengeç 5, Oğlak 11. D9 evleri, Balık'tan: Balık 1, Boğa 3, Koç 2, Aslan 6, Balık 1, Yengeç 5, Koç 2, Akrep 9, Boğa 3."
  ],
  "input": {
    "lagna_lon": 220.0,
    "grahas": {"Sun": 319.0, "Moon": 126.0, "Mars": 43.0, "Mercury": 295.0, "Jupiter": 358.0, "Venus": 333.0, "Saturn": 202.0, "Rahu": 106.0, "Ketu": 286.0},
    "advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": false}
  },
  "expected": {
    "karakas": [
      {"role": "Atma", "graha": "Jupiter", "effective_deg": 28.0, "tie": false},
      {"role": "Amatya", "graha": "Mercury", "effective_deg": 25.0, "tie": false},
      {"role": "Bhratri", "graha": "Saturn", "effective_deg": 22.0, "tie": false},
      {"role": "Matri", "graha": "Sun", "effective_deg": 19.0, "tie": false},
      {"role": "Pitri", "graha": "Rahu", "effective_deg": 14.0, "tie": false},
      {"role": "Putra", "graha": "Mars", "effective_deg": 13.0, "tie": false},
      {"role": "Gnati", "graha": "Moon", "effective_deg": 6.0, "tie": false},
      {"role": "Dara", "graha": "Venus", "effective_deg": 3.0, "tie": false}
    ],
    "karakamsha": {
      "atmakaraka": "Jupiter", "sign": "Pisces",
      "houses_in_d1": {"Sun": 12, "Moon": 6, "Mars": 3, "Mercury": 11, "Jupiter": 1, "Venus": 1, "Saturn": 8, "Rahu": 5, "Ketu": 11},
      "houses_in_d9": {"Sun": 1, "Moon": 3, "Mars": 2, "Mercury": 6, "Jupiter": 1, "Venus": 5, "Saturn": 2, "Rahu": 9, "Ketu": 3}
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/jaimini/co-lord-company-and-forward-count.json`

````json
{
  "id": "co-lord-company-and-forward-count",
  "source": "hand",
  "rule_ref": "BPHS, karaka, pada ve argala bölümleri; Jaimini Upadesha Sutraları (eş lordlar, rashi sayımı); navamsa: her burç dokuz eşit parça, ateş burçları Koç'tan, toprak Oğlak'tan, hava Terazi'den, su Yengeç'ten sayılır",
  "derivation": [
    "Burçlar: Güneş 42° Boğa 12°, Ay 302° Kova 2°, Mars 134° Aslan 14°, Merkür 53° Boğa 23°, Jüpiter 65° İkizler 5°, Venüs 85° İkizler 25°, Satürn 77° İkizler 17°, Rahu 141° Aslan 21°, Ketu 321° Kova 21°. Lagna 165° Başak.",
    "Karaka (sekizli şema): Rahu'nun etkin derecesi 30 − 21 = 9. Büyükten küçüğe: Venüs 25, Merkür 23, Satürn 17, Mars 14, Güneş 12, Rahu 9, Jüpiter 5, Ay 2. Eşitlik yok.",
    "Karakamsha: Atmakaraka Venüs İkizler 25°. Hava burcu Terazi'den sayılır; 25° sekizinci parçadadır (23°20′–26°40′): Terazi'den 8. burç Boğa.",
    "D9: Güneş Boğa 12° (toprak, Oğlak'tan; 4. parça 10°–13°20′): Koç. Ay Kova 2° (hava; 1. parça): Terazi. Mars Aslan 14° (ateş, Koç'tan; 5. parça 13°20′–16°40′): Aslan. Merkür Boğa 23° (toprak; 7. parça 20°–23°20′): Yengeç. Jüpiter İkizler 5° (hava; 2. parça 3°20′–6°40′): Akrep. Venüs: Boğa. Satürn İkizler 17° (hava; 6. parça 16°40′–20°): Balık. Rahu Aslan 21° (ateş; 7. parça): Terazi. Ketu Kova 21° (hava; 7. parça): Koç.",
    "D1 evleri, Boğa'dan (Güneş…Ketu): Boğa 1, Kova 10, Aslan 4, Boğa 1, İkizler 2, İkizler 2, İkizler 2, Aslan 4, Kova 10. D9 evleri, Boğa'dan: Koç 12, Terazi 6, Aslan 4, Yengeç 3, Akrep 7, Boğa 1, Balık 11, Terazi 6, Koç 12."
  ],
  "input": {
    "lagna_lon": 165.0,
    "grahas": {"Sun": 42.0, "Moon": 302.0, "Mars": 134.0, "Mercury": 53.0, "Jupiter": 65.0, "Venus": 85.0, "Saturn": 77.0, "Rahu": 141.0, "Ketu": 321.0},
    "advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}
  },
  "expected": {
    "karakas": [
      {"role": "Atma", "graha": "Venus", "effective_deg": 25.0, "tie": false},
      {"role": "Amatya", "graha": "Mercury", "effective_deg": 23.0, "tie": false},
      {"role": "Bhratri", "graha": "Saturn", "effective_deg": 17.0, "tie": false},
      {"role": "Matri", "graha": "Mars", "effective_deg": 14.0, "tie": false},
      {"role": "Pitri", "graha": "Sun", "effective_deg": 12.0, "tie": false},
      {"role": "Putra", "graha": "Rahu", "effective_deg": 9.0, "tie": false},
      {"role": "Gnati", "graha": "Jupiter", "effective_deg": 5.0, "tie": false},
      {"role": "Dara", "graha": "Moon", "effective_deg": 2.0, "tie": false}
    ],
    "karakamsha": {
      "atmakaraka": "Venus", "sign": "Taurus",
      "houses_in_d1": {"Sun": 1, "Moon": 10, "Mars": 4, "Mercury": 1, "Jupiter": 2, "Venus": 2, "Saturn": 2, "Rahu": 4, "Ketu": 10},
      "houses_in_d9": {"Sun": 12, "Moon": 6, "Mars": 4, "Mercury": 3, "Jupiter": 7, "Venus": 1, "Saturn": 11, "Rahu": 6, "Ketu": 12}
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/jaimini/co-lord-exaltation-and-debilitation.json`

````json
{
  "id": "co-lord-exaltation-and-debilitation",
  "source": "hand",
  "rule_ref": "BPHS, karaka, pada ve argala bölümleri; Jaimini Upadesha Sutraları (eş lordlar, rashi sayımı); navamsa: her burç dokuz eşit parça, ateş burçları Koç'tan, toprak Oğlak'tan, hava Terazi'den, su Yengeç'ten sayılır",
  "derivation": [
    "Burçlar: Güneş 173° Başak 23°, Ay 355° Balık 25°, Mars 285° Oğlak 15°, Merkür 156° Başak 6°, Jüpiter 107° Yengeç 17°, Venüs 192° Terazi 12°, Satürn 5° Koç 5°, Rahu 249° Yay 9°, Ketu 69° İkizler 9°. Lagna 130° Aslan.",
    "Karaka (sekizli şema): Rahu'nun etkin derecesi 30 − 9 = 21. Büyükten küçüğe: Ay 25, Güneş 23, Rahu 21, Jüpiter 17, Mars 15, Venüs 12, Merkür 6, Satürn 5. Komşu farkların en küçüğü 1°: eşitlik yok.",
    "Karakamsha: Atmakaraka Ay Balık 25°. Su burcu Yengeç'ten sayılır; 25° sekizinci parçadadır (23°20′–26°40′): Yengeç'ten 8. burç Kova.",
    "D9: Güneş Başak 23° (toprak, Oğlak'tan; 7. parça 20°–23°20′): Yengeç. Ay: Kova. Mars Oğlak 15° (toprak; 5. parça 13°20′–16°40′): Boğa. Merkür Başak 6° (toprak; 2. parça 3°20′–6°40′): Kova. Jüpiter Yengeç 17° (su; 6. parça 16°40′–20°): Yay. Venüs Terazi 12° (hava, Terazi'den; 4. parça 10°–13°20′): Oğlak. Satürn Koç 5° (ateş, Koç'tan; 2. parça): Boğa. Rahu Yay 9° (ateş; 3. parça 6°40′–10°): İkizler. Ketu İkizler 9° (hava; 3. parça): Yay.",
    "D1 evleri, Kova'dan (Güneş…Ketu): Başak 8, Balık 2, Oğlak 12, Başak 8, Yengeç 6, Terazi 9, Koç 3, Yay 11, İkizler 5. D9 evleri, Kova'dan: Yengeç 6, Kova 1, Boğa 4, Kova 1, Yay 11, Oğlak 12, Boğa 4, İkizler 5, Yay 11."
  ],
  "input": {
    "lagna_lon": 130.0,
    "grahas": {"Sun": 173.0, "Moon": 355.0, "Mars": 285.0, "Mercury": 156.0, "Jupiter": 107.0, "Venus": 192.0, "Saturn": 5.0, "Rahu": 249.0, "Ketu": 69.0},
    "advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}
  },
  "expected": {
    "karakas": [
      {"role": "Atma", "graha": "Moon", "effective_deg": 25.0, "tie": false},
      {"role": "Amatya", "graha": "Sun", "effective_deg": 23.0, "tie": false},
      {"role": "Bhratri", "graha": "Rahu", "effective_deg": 21.0, "tie": false},
      {"role": "Matri", "graha": "Jupiter", "effective_deg": 17.0, "tie": false},
      {"role": "Pitri", "graha": "Mars", "effective_deg": 15.0, "tie": false},
      {"role": "Putra", "graha": "Venus", "effective_deg": 12.0, "tie": false},
      {"role": "Gnati", "graha": "Mercury", "effective_deg": 6.0, "tie": false},
      {"role": "Dara", "graha": "Saturn", "effective_deg": 5.0, "tie": false}
    ],
    "karakamsha": {
      "atmakaraka": "Moon", "sign": "Aquarius",
      "houses_in_d1": {"Sun": 8, "Moon": 2, "Mars": 12, "Mercury": 8, "Jupiter": 6, "Venus": 9, "Saturn": 3, "Rahu": 11, "Ketu": 5},
      "houses_in_d9": {"Sun": 6, "Moon": 1, "Mars": 4, "Mercury": 1, "Jupiter": 11, "Venus": 12, "Saturn": 4, "Rahu": 5, "Ketu": 11}
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/jaimini/co-lords-in-the-same-sign.json`

````json
{
  "id": "co-lords-in-the-same-sign",
  "source": "hand",
  "rule_ref": "BPHS, karaka, pada ve argala bölümleri; Jaimini Upadesha Sutraları (eş lordlar, rashi sayımı); navamsa: her burç dokuz eşit parça, ateş burçları Koç'tan, toprak Oğlak'tan, hava Terazi'den, su Yengeç'ten sayılır",
  "derivation": [
    "Burçlar: Güneş 238° Akrep 28°, Ay 98° Yengeç 8°, Mars 231° Akrep 21°, Merkür 253° Yay 13°, Jüpiter 323° Kova 23°, Venüs 199° Terazi 19°, Satürn 46° Boğa 16°, Rahu 34° Boğa 4°, Ketu 214° Akrep 4°. Lagna 310° Kova.",
    "Karaka (sekizli şema): Rahu'nun etkin derecesi 30 − 4 = 26. Büyükten küçüğe: Güneş 28, Rahu 26, Jüpiter 23, Mars 21, Venüs 19, Satürn 16, Merkür 13, Ay 8. Eşitlik yok.",
    "Karakamsha: Atmakaraka Güneş Akrep 28°. Su burcu Yengeç'ten sayılır; 28° dokuzuncu parçadadır: Yengeç'ten 9. burç Balık.",
    "D9: Güneş: Balık. Ay Yengeç 8° (su; 3. parça 6°40′–10°): Başak. Mars Akrep 21° (su; 7. parça 20°–23°20′): Oğlak. Merkür Yay 13° (ateş, Koç'tan; 4. parça 10°–13°20′): Yengeç. Jüpiter Kova 23° (hava, Terazi'den; 7. parça): Koç. Venüs Terazi 19° (hava; 6. parça 16°40′–20°): Balık. Satürn Boğa 16° (toprak, Oğlak'tan; 5. parça 13°20′–16°40′): Boğa. Rahu Boğa 4° (toprak; 2. parça 3°20′–6°40′): Kova. Ketu Akrep 4° (su; 2. parça): Aslan.",
    "D1 evleri, Balık'tan (Güneş…Ketu): Akrep 9, Yengeç 5, Akrep 9, Yay 10, Kova 12, Terazi 8, Boğa 3, Boğa 3, Akrep 9. D9 evleri, Balık'tan: Balık 1, Başak 7, Oğlak 11, Yengeç 5, Koç 2, Balık 1, Boğa 3, Kova 12, Aslan 6."
  ],
  "input": {
    "lagna_lon": 310.0,
    "grahas": {"Sun": 238.0, "Moon": 98.0, "Mars": 231.0, "Mercury": 253.0, "Jupiter": 323.0, "Venus": 199.0, "Saturn": 46.0, "Rahu": 34.0, "Ketu": 214.0},
    "advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}
  },
  "expected": {
    "karakas": [
      {"role": "Atma", "graha": "Sun", "effective_deg": 28.0, "tie": false},
      {"role": "Amatya", "graha": "Rahu", "effective_deg": 26.0, "tie": false},
      {"role": "Bhratri", "graha": "Jupiter", "effective_deg": 23.0, "tie": false},
      {"role": "Matri", "graha": "Mars", "effective_deg": 21.0, "tie": false},
      {"role": "Pitri", "graha": "Venus", "effective_deg": 19.0, "tie": false},
      {"role": "Putra", "graha": "Saturn", "effective_deg": 16.0, "tie": false},
      {"role": "Gnati", "graha": "Mercury", "effective_deg": 13.0, "tie": false},
      {"role": "Dara", "graha": "Moon", "effective_deg": 8.0, "tie": false}
    ],
    "karakamsha": {
      "atmakaraka": "Sun", "sign": "Pisces",
      "houses_in_d1": {"Sun": 9, "Moon": 5, "Mars": 9, "Mercury": 10, "Jupiter": 12, "Venus": 8, "Saturn": 3, "Rahu": 3, "Ketu": 9},
      "houses_in_d9": {"Sun": 1, "Moon": 7, "Mars": 11, "Mercury": 5, "Jupiter": 2, "Venus": 1, "Saturn": 3, "Rahu": 12, "Ketu": 6}
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/jaimini/node-in-its-sign-and-sign-kind.json`

````json
{
  "id": "node-in-its-sign-and-sign-kind",
  "source": "hand",
  "rule_ref": "BPHS, karaka, pada ve argala bölümleri; Jaimini Upadesha Sutraları (eş lordlar, rashi sayımı); navamsa: her burç dokuz eşit parça, ateş burçları Koç'tan, toprak Oğlak'tan, hava Terazi'den, su Yengeç'ten sayılır",
  "derivation": [
    "Burçlar: Güneş 3° Koç 3°, Ay 292° Oğlak 22°, Mars 161° Başak 11°, Merkür 19° Koç 19°, Jüpiter 127° Aslan 7°, Venüs 345° Balık 15°, Satürn 116° Yengeç 26°, Rahu 43° Boğa 13°, Ketu 223° Akrep 13°. Lagna 75° İkizler.",
    "Karaka (yedili şema, Rahu sıralanmaz): Satürn 26, Ay 22, Merkür 19, Venüs 15, Mars 11, Jüpiter 7, Güneş 3. Eşitlik yok.",
    "Karakamsha: Atmakaraka Satürn Yengeç 26°. Su burcu Yengeç'ten sayılır; 26° sekizinci parçadadır (23°20′–26°40′): Yengeç'ten 8. burç Kova.",
    "D9: Güneş Koç 3° (ateş, Koç'tan; 1. parça): Koç. Ay Oğlak 22° (toprak, Oğlak'tan; 7. parça 20°–23°20′): Yengeç. Mars Başak 11° (toprak; 4. parça 10°–13°20′): Koç. Merkür Koç 19° (ateş; 6. parça 16°40′–20°): Başak. Jüpiter Aslan 7° (ateş; 3. parça 6°40′–10°): İkizler. Venüs Balık 15° (su; 5. parça 13°20′–16°40′): Akrep. Satürn: Kova. Rahu Boğa 13° (toprak; 4. parça): Koç. Ketu Akrep 13° (su; 4. parça): Terazi.",
    "D1 evleri, Kova'dan (Güneş…Ketu): Koç 3, Oğlak 12, Başak 8, Koç 3, Aslan 7, Balık 2, Yengeç 6, Boğa 4, Akrep 10. D9 evleri, Kova'dan: Koç 3, Yengeç 6, Koç 3, Başak 8, İkizler 5, Akrep 10, Kova 1, Koç 3, Terazi 9."
  ],
  "input": {
    "lagna_lon": 75.0,
    "grahas": {"Sun": 3.0, "Moon": 292.0, "Mars": 161.0, "Mercury": 19.0, "Jupiter": 127.0, "Venus": 345.0, "Saturn": 116.0, "Rahu": 43.0, "Ketu": 223.0},
    "advanced": {"karaka_scheme": "Seven", "jaimini_co_lords": true}
  },
  "expected": {
    "karakas": [
      {"role": "Atma", "graha": "Saturn", "effective_deg": 26.0, "tie": false},
      {"role": "Amatya", "graha": "Moon", "effective_deg": 22.0, "tie": false},
      {"role": "Bhratri", "graha": "Mercury", "effective_deg": 19.0, "tie": false},
      {"role": "Matri", "graha": "Venus", "effective_deg": 15.0, "tie": false},
      {"role": "Putra", "graha": "Mars", "effective_deg": 11.0, "tie": false},
      {"role": "Gnati", "graha": "Jupiter", "effective_deg": 7.0, "tie": false},
      {"role": "Dara", "graha": "Sun", "effective_deg": 3.0, "tie": false}
    ],
    "karakamsha": {
      "atmakaraka": "Saturn", "sign": "Aquarius",
      "houses_in_d1": {"Sun": 3, "Moon": 12, "Mars": 8, "Mercury": 3, "Jupiter": 7, "Venus": 2, "Saturn": 6, "Rahu": 4, "Ketu": 10},
      "houses_in_d9": {"Sun": 3, "Moon": 6, "Mars": 3, "Mercury": 8, "Jupiter": 5, "Venus": 10, "Saturn": 1, "Rahu": 3, "Ketu": 9}
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/jaimini/rahu-atmakaraka-arudha-exceptions.json`

````json
{
  "id": "rahu-atmakaraka-arudha-exceptions",
  "source": "hand",
  "rule_ref": "BPHS, karaka, pada ve argala bölümleri; Jaimini Upadesha Sutraları (eş lordlar, rashi sayımı); navamsa: her burç dokuz eşit parça, ateş burçları Koç'tan, toprak Oğlak'tan, hava Terazi'den, su Yengeç'ten sayılır",
  "derivation": [
    "Burçlar: Güneş 218° Akrep 8°, Ay 21° Koç 21°, Mars 4° Koç 4°, Merkür 242° Yay 2°, Jüpiter 47° Boğa 17°, Venüs 206° Terazi 26°, Satürn 281° Oğlak 11°, Rahu 61° İkizler 1°, Ketu 241° Yay 1°. Lagna 12° Koç.",
    "Karaka (sekizli şema): Rahu'nun etkin derecesi 30 − 1 = 29. Büyükten küçüğe: Rahu 29, Venüs 26, Ay 21, Jüpiter 17, Satürn 11, Güneş 8, Mars 4, Merkür 2. Komşu farkların en küçüğü 2°: eşitlik yok.",
    "Karakamsha: Atmakaraka Rahu İkizler 1°. Hava burcu Terazi'den sayılır; 1° ilk parçadadır (0°–3°20′): Terazi.",
    "D9: Güneş Akrep 8° (su, Yengeç'ten; 3. parça 6°40′–10°): Başak. Ay Koç 21° (ateş, Koç'tan; 7. parça 20°–23°20′): Terazi. Mars Koç 4° (2. parça): Boğa. Merkür Yay 2° (ateş; 1. parça): Koç. Jüpiter Boğa 17° (toprak, Oğlak'tan; 6. parça 16°40′–20°): İkizler. Venüs Terazi 26° (hava; 8. parça 23°20′–26°40′): Boğa. Satürn Oğlak 11° (toprak; 4. parça 10°–13°20′): Koç. Rahu: Terazi. Ketu Yay 1° (1. parça): Koç.",
    "D1 evleri, Terazi'den (Güneş…Ketu): Akrep 2, Koç 7, Koç 7, Yay 3, Boğa 8, Terazi 1, Oğlak 4, İkizler 9, Yay 3. D9 evleri, Terazi'den: Başak 12, Terazi 1, Boğa 8, Koç 7, İkizler 9, Boğa 8, Koç 7, Terazi 1, Koç 7."
  ],
  "input": {
    "lagna_lon": 12.0,
    "grahas": {"Sun": 218.0, "Moon": 21.0, "Mars": 4.0, "Mercury": 242.0, "Jupiter": 47.0, "Venus": 206.0, "Saturn": 281.0, "Rahu": 61.0, "Ketu": 241.0},
    "advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}
  },
  "expected": {
    "karakas": [
      {"role": "Atma", "graha": "Rahu", "effective_deg": 29.0, "tie": false},
      {"role": "Amatya", "graha": "Venus", "effective_deg": 26.0, "tie": false},
      {"role": "Bhratri", "graha": "Moon", "effective_deg": 21.0, "tie": false},
      {"role": "Matri", "graha": "Jupiter", "effective_deg": 17.0, "tie": false},
      {"role": "Pitri", "graha": "Saturn", "effective_deg": 11.0, "tie": false},
      {"role": "Putra", "graha": "Sun", "effective_deg": 8.0, "tie": false},
      {"role": "Gnati", "graha": "Mars", "effective_deg": 4.0, "tie": false},
      {"role": "Dara", "graha": "Mercury", "effective_deg": 2.0, "tie": false}
    ],
    "karakamsha": {
      "atmakaraka": "Rahu", "sign": "Libra",
      "houses_in_d1": {"Sun": 2, "Moon": 7, "Mars": 7, "Mercury": 3, "Jupiter": 8, "Venus": 1, "Saturn": 4, "Rahu": 9, "Ketu": 3},
      "houses_in_d9": {"Sun": 12, "Moon": 1, "Mars": 8, "Mercury": 7, "Jupiter": 9, "Venus": 8, "Saturn": 7, "Rahu": 1, "Ketu": 7}
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/jaimini/tie-within-one-arcsecond-seven-scheme.json`

````json
{
  "id": "tie-within-one-arcsecond-seven-scheme",
  "source": "hand",
  "rule_ref": "BPHS, karaka, pada ve argala bölümleri; Jaimini Upadesha Sutraları (eş lordlar, rashi sayımı); navamsa: her burç dokuz eşit parça, ateş burçları Koç'tan, toprak Oğlak'tan, hava Terazi'den, su Yengeç'ten sayılır",
  "derivation": [
    "Burçlar: Güneş 65° İkizler 5°, Ay 48° Boğa 18°, Mars 147° Aslan 27°, Merkür 83° İkizler 23°, Jüpiter 347,9998° Balık 17,9998°, Venüs 107,9994° Yengeç 17,9994°, Satürn 249° Yay 9°, Rahu 162° Başak 12°, Ketu 342° Balık 12°. Lagna 100° Yengeç.",
    "Karaka (yedili şema, Rahu sıralanmaz, Pitri rolü yok): Mars 27, Merkür 23, Ay 18, Jüpiter 17,9998, Venüs 17,9994, Satürn 9, Güneş 5. 1″ = 0,000278°. Ay − Jüpiter = 0,0002° (0,72″): eşit sayılır, ikisi de işaretlenir; sıra tam değerle Ay önce. Jüpiter − Venüs = 0,0004° (1,44″): eşit değil; Venüs'ün öbür komşusu Satürn 9° uzakta: Venüs işaretlenmez.",
    "Karakamsha: Atmakaraka Mars Aslan 27°. Ateş burcu Koç'tan sayılır; 27° dokuzuncu parçadadır (26°40′–30°): Koç'tan 9. burç Yay.",
    "D9: Güneş İkizler 5° (hava, Terazi'den; 2. parça 3°20′–6°40′): Akrep. Ay Boğa 18° (toprak, Oğlak'tan; 6. parça 16°40′–20°): İkizler. Mars: Yay. Merkür İkizler 23° (hava; 7. parça 20°–23°20′): Koç. Jüpiter Balık 17,9998° (su, Yengeç'ten; 6. parça): Yay. Venüs Yengeç 17,9994° (su; 6. parça): Yay. Satürn Yay 9° (ateş; 3. parça 6°40′–10°): İkizler. Rahu Başak 12° (toprak; 4. parça 10°–13°20′): Koç. Ketu Balık 12° (su; 4. parça): Terazi.",
    "D1 evleri, Yay'dan (Güneş…Ketu): İkizler 7, Boğa 6, Aslan 9, İkizler 7, Balık 4, Yengeç 8, Yay 1, Başak 10, Balık 4. D9 evleri, Yay'dan: Akrep 12, İkizler 7, Yay 1, Koç 5, Yay 1, Yay 1, İkizler 7, Koç 5, Terazi 11."
  ],
  "input": {
    "lagna_lon": 100.0,
    "grahas": {"Sun": 65.0, "Moon": 48.0, "Mars": 147.0, "Mercury": 83.0, "Jupiter": 347.9998, "Venus": 107.9994, "Saturn": 249.0, "Rahu": 162.0, "Ketu": 342.0},
    "advanced": {"karaka_scheme": "Seven", "jaimini_co_lords": true}
  },
  "expected": {
    "karakas": [
      {"role": "Atma", "graha": "Mars", "effective_deg": 27.0, "tie": false},
      {"role": "Amatya", "graha": "Mercury", "effective_deg": 23.0, "tie": false},
      {"role": "Bhratri", "graha": "Moon", "effective_deg": 18.0, "tie": true},
      {"role": "Matri", "graha": "Jupiter", "effective_deg": 17.9998, "tie": true},
      {"role": "Putra", "graha": "Venus", "effective_deg": 17.9994, "tie": false},
      {"role": "Gnati", "graha": "Saturn", "effective_deg": 9.0, "tie": false},
      {"role": "Dara", "graha": "Sun", "effective_deg": 5.0, "tie": false}
    ],
    "karakamsha": {
      "atmakaraka": "Mars", "sign": "Sagittarius",
      "houses_in_d1": {"Sun": 7, "Moon": 6, "Mars": 9, "Mercury": 7, "Jupiter": 4, "Venus": 8, "Saturn": 1, "Rahu": 10, "Ketu": 4},
      "houses_in_d9": {"Sun": 12, "Moon": 7, "Mars": 1, "Mercury": 5, "Jupiter": 1, "Venus": 1, "Saturn": 7, "Rahu": 5, "Ketu": 11}
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/jaimini/time-unknown-third-house-malefics.json`

````json
{
  "id": "time-unknown-third-house-malefics",
  "source": "hand",
  "rule_ref": "BPHS, karaka, pada ve argala bölümleri; Jaimini Upadesha Sutraları (eş lordlar, rashi sayımı); navamsa: her burç dokuz eşit parça, ateş burçları Koç'tan, toprak Oğlak'tan, hava Terazi'den, su Yengeç'ten sayılır",
  "derivation": [
    "Burçlar: Güneş 74° İkizler 14°, Ay 62° İkizler 2°, Mars 84° İkizler 24°, Merkür 59° Boğa 29°, Jüpiter 187° Terazi 7°, Venüs 99° Yengeç 9°, Satürn 318° Kova 18°, Rahu 221° Akrep 11°, Ketu 41° Boğa 11°. Lagna yok (doğum saati bilinmiyor).",
    "Karaka (sekizli şema): Rahu'nun etkin derecesi 30 − 11 = 19. Büyükten küçüğe: Merkür 29, Mars 24, Rahu 19, Satürn 18, Güneş 14, Venüs 9, Jüpiter 7, Ay 2. Eşitlik yok.",
    "Karakamsha: Atmakaraka Merkür Boğa 29°. Toprak burcu Oğlak'tan sayılır; 29° dokuzuncu parçadadır: Oğlak'tan 9. burç Başak.",
    "D9: Güneş İkizler 14° (hava, Terazi'den; 5. parça 13°20′–16°40′): Kova. Ay İkizler 2° (1. parça): Terazi. Mars İkizler 24° (8. parça 23°20′–26°40′): Boğa. Merkür: Başak. Jüpiter Terazi 7° (hava; 3. parça 6°40′–10°): Yay. Venüs Yengeç 9° (su, Yengeç'ten; 3. parça): Başak. Satürn Kova 18° (hava; 6. parça 16°40′–20°): Balık. Rahu Akrep 11° (su; 4. parça 10°–13°20′): Terazi. Ketu Boğa 11° (toprak; 4. parça): Koç.",
    "D1 evleri, Başak'tan (Güneş…Ketu): İkizler 10, İkizler 10, İkizler 10, Boğa 9, Terazi 2, Yengeç 11, Kova 6, Akrep 3, Boğa 9. D9 evleri, Başak'tan: Kova 6, Terazi 2, Boğa 9, Başak 1, Yay 4, Başak 1, Balık 7, Terazi 2, Koç 8."
  ],
  "input": {
    "lagna_lon": null,
    "grahas": {"Sun": 74.0, "Moon": 62.0, "Mars": 84.0, "Mercury": 59.0, "Jupiter": 187.0, "Venus": 99.0, "Saturn": 318.0, "Rahu": 221.0, "Ketu": 41.0},
    "advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}
  },
  "expected": {
    "karakas": [
      {"role": "Atma", "graha": "Mercury", "effective_deg": 29.0, "tie": false},
      {"role": "Amatya", "graha": "Mars", "effective_deg": 24.0, "tie": false},
      {"role": "Bhratri", "graha": "Rahu", "effective_deg": 19.0, "tie": false},
      {"role": "Matri", "graha": "Saturn", "effective_deg": 18.0, "tie": false},
      {"role": "Pitri", "graha": "Sun", "effective_deg": 14.0, "tie": false},
      {"role": "Putra", "graha": "Venus", "effective_deg": 9.0, "tie": false},
      {"role": "Gnati", "graha": "Jupiter", "effective_deg": 7.0, "tie": false},
      {"role": "Dara", "graha": "Moon", "effective_deg": 2.0, "tie": false}
    ],
    "karakamsha": {
      "atmakaraka": "Mercury", "sign": "Virgo",
      "houses_in_d1": {"Sun": 10, "Moon": 10, "Mars": 10, "Mercury": 9, "Jupiter": 2, "Venus": 11, "Saturn": 6, "Rahu": 3, "Ketu": 9},
      "houses_in_d9": {"Sun": 6, "Moon": 2, "Mars": 9, "Mercury": 1, "Jupiter": 4, "Venus": 1, "Saturn": 7, "Rahu": 2, "Ketu": 8}
    }
  }
}
````

`README.md`'ye `advanced` bloğu ve modül satırı eklenir:

````diff
diff --git a/tests/golden/modules/README.md b/tests/golden/modules/README.md
index f2afc86..9e7fac0 100644
--- a/tests/golden/modules/README.md
+++ b/tests/golden/modules/README.md
@@ -32,11 +32,16 @@ ileri modüller tasarım belgesi §13 (dört katmanlı test stratejisinin ilk ka
 
 `lagna_lon: null` doğum saati bilinmeyen haritadır: lagnaya bağlı her beklenen değer `null` olur.
 
+Ayara bağlı modüller (Jaimini) girdiye `advanced` bloğunu ekler:
+`"advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}`. Diğer modüllerde bu alan
+yazılmaz.
+
 ## Modüller
 
 | Klasör | `expected` | En az |
 |---|---|---|
 | `foundation` | `waxing`, `nature` (dokuz graha), `kuja_dosha`, `house_lords` (12), `houses`, `houses_owned` | 6 |
 | `drishti` | `sign_aspects` (on üç tam bakışın tamamı, tablo sırasıyla), `sphuta` (en az on değer: `from`, `to`, `angle`, `value`) | 4 |
+| `jaimini` | `karakas` (rol sırasında: `role`, `graha`, `effective_deg`, `tie`), `karakamsha` (`atmakaraka`, `sign`, dokuz grahanın `houses_in_d1` ve `houses_in_d9` değeri) | 8 |
 
 Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
````

- [ ] **Step 3: Düzeneğe `jaimini` modülünü ve değişmezleri ekle (başarısız testler)**

Düzenek: `Input` isteğe bağlı `advanced` bloğunu okur (yalnızca test tipidir; IPC tipi değildir, bu yüzden `serde(default)` burada serbesttir); `jaimini_hand_fixtures` karaka sırasını (rol, graha, etkin derece 1e-9 toleransla, eşitlik işareti) ve karakamshayı (burç, on sekiz ev sayısı) karşılaştırır; `jaimini_fixtures_cover_the_required_cases` spec'in zorunlu durumlarının örneklerde bulunduğunu sabitler (örnek silinir ya da değiştirilirse kapsam sessizce daralmasın).

Değişmezler (`jaimini_properties`): karaka sırası şemadaki grahaların permütasyonudur, Ketu yoktur, dereceler artmayan sıradadır, eşitlik işareti iki komşuyu da kapsar; karakamsha evleri iki haritadan sayılır ve Atmakaraka'nın D9 evi 1'dir; vargottama Atmakaraka'da D1 evi de 1'dir; grahaların rashi drishtisi burç tablosunu izler ve karşılıklıdır.

````diff
diff --git a/crates/jyotish-core/src/proptests.rs b/crates/jyotish-core/src/proptests.rs
index 5362376..85b4356 100644
--- a/crates/jyotish-core/src/proptests.rs
+++ b/crates/jyotish-core/src/proptests.rs
@@ -482,3 +482,86 @@ mod drishti_properties {
         }
     }
 }
+
+/// Jaimini çekirdeği (Alt proje 3.2).
+mod jaimini_properties {
+    use proptest::prelude::*;
+
+    use super::foundation_properties::positions;
+    use crate::graha::Graha;
+    use crate::jaimini::{
+        chara_karakas, graha_rashi_drishti, karaka_roles, karakamsha, navamsa_sign,
+        rashi_drishti_signs, KARAKA_TIE_DEG,
+    };
+    use crate::settings::KarakaScheme;
+    use crate::varga::is_vargottama;
+
+    fn scheme() -> impl Strategy<Value = KarakaScheme> {
+        prop_oneof![Just(KarakaScheme::Eight), Just(KarakaScheme::Seven)]
+    }
+
+    proptest! {
+        #[test]
+        fn karaka_order_is_a_permutation_of_the_scheme_grahas(p in positions(), s in scheme()) {
+            let k = chara_karakas(&p, s);
+            let roles = karaka_roles(s);
+            prop_assert_eq!(k.len(), roles.len());
+            let mut grahas: Vec<Graha> = k.iter().map(|c| c.graha).collect();
+            grahas.sort();
+            prop_assert_eq!(&grahas[..], &Graha::ALL[..roles.len()]);
+            prop_assert!(k.iter().all(|c| c.graha != Graha::Ketu));
+            for (i, c) in k.iter().enumerate() {
+                prop_assert_eq!(c.role, roles[i]);
+                prop_assert!((0.0..=30.0).contains(&c.effective_deg));
+                let deg = p.point(c.graha).deg_in_sign;
+                let want = if c.graha == Graha::Rahu { 30.0 - deg } else { deg };
+                prop_assert_eq!(c.effective_deg, want);
+            }
+            for w in k.windows(2) {
+                prop_assert!(w[0].effective_deg >= w[1].effective_deg);
+                let near = w[0].effective_deg - w[1].effective_deg <= KARAKA_TIE_DEG;
+                if near {
+                    prop_assert!(w[0].tie && w[1].tie);
+                }
+                if w[0].effective_deg == w[1].effective_deg {
+                    prop_assert!(w[0].graha < w[1].graha);
+                }
+            }
+            for (i, c) in k.iter().enumerate() {
+                let near = |j: usize| (c.effective_deg - k[j].effective_deg).abs() <= KARAKA_TIE_DEG;
+                let expected = (i > 0 && near(i - 1)) || (i + 1 < k.len() && near(i + 1));
+                prop_assert_eq!(c.tie, expected);
+            }
+        }
+
+        #[test]
+        fn karakamsha_houses_follow_the_two_charts(p in positions(), g in 0usize..8) {
+            let ak = Graha::ALL[g];
+            let k = karakamsha(&p, ak);
+            prop_assert_eq!(k.sign, navamsa_sign(p.point(ak)));
+            prop_assert_eq!(k.houses_in_d9[g], 1);
+            for (i, point) in p.grahas.iter().enumerate() {
+                prop_assert_eq!(k.houses_in_d1[i], point.rasi_sign.house_from(k.sign));
+                prop_assert_eq!(k.houses_in_d9[i], navamsa_sign(point).house_from(k.sign));
+            }
+            // Vargottama Atmakaraka: karakamsha kendi rasi burcudur.
+            let lon = p.point(ak).rasi_sign.index() as f64 * 30.0 + p.point(ak).deg_in_sign;
+            prop_assert_eq!(is_vargottama(lon), k.houses_in_d1[g] == 1);
+        }
+
+        #[test]
+        fn graha_rashi_drishti_follows_the_sign_table(p in positions()) {
+            let aspects = graha_rashi_drishti(&p);
+            prop_assert_eq!(aspects.len(), 27);
+            for (i, a) in aspects.iter().enumerate() {
+                prop_assert_eq!(a.graha, Graha::ALL[i / 3]);
+                prop_assert_eq!(a.target, rashi_drishti_signs(p.sign_of(a.graha))[i % 3]);
+                prop_assert_eq!(&a.occupants, &p.occupants(a.target));
+                // Karşılıklılık: hedef burçtaki her graha da bu grahanın burcuna bakar.
+                for &other in &a.occupants {
+                    prop_assert!(rashi_drishti_signs(p.sign_of(other)).contains(&p.sign_of(a.graha)));
+                }
+            }
+        }
+    }
+}
diff --git a/crates/jyotish-core/tests/modules_golden.rs b/crates/jyotish-core/tests/modules_golden.rs
index a21a777..94d6af2 100644
--- a/crates/jyotish-core/tests/modules_golden.rs
+++ b/crates/jyotish-core/tests/modules_golden.rs
@@ -7,8 +7,9 @@ use std::collections::BTreeMap;
 use std::path::PathBuf;
 
 use jyotish_core::{
-    drishti_table, house_lord, houses_owned, kuja_dosha, moon_is_waxing, nature, normalize, Graha,
-    KujaDosha, Nature, Positions, Sign, ASPECTING_GRAHAS, DRISHTI_METHOD,
+    chara_karakas, drishti_table, house_lord, houses_owned, karakamsha, kuja_dosha, moon_is_waxing,
+    nature, normalize, AdvancedSettings, Graha, KarakaRole, KujaDosha, Nature, Positions, Sign,
+    ASPECTING_GRAHAS, DRISHTI_METHOD,
 };
 use serde::de::DeserializeOwned;
 use serde::Deserialize;
@@ -16,6 +17,7 @@ use serde::Deserialize;
 /// Modül başına en az örnek sayısı; altına düşerse test düşer (sessiz silinmeye karşı).
 const MIN_FOUNDATION: usize = 6;
 const MIN_DRISHTI: usize = 4;
+const MIN_JAIMINI: usize = 8;
 /// El hesabı ile motor arasındaki en büyük fark, virupa.
 const VIRUPA_TOL: f64 = 1e-9;
 
@@ -31,12 +33,14 @@ struct Fixture<X> {
 }
 
 /// Sentetik konumlar: sidereal boylamlar, derece. `lagna_lon: null` doğum saati bilinmeyen
-/// haritadır.
+/// haritadır. `advanced` yalnızca ayara bağlı modüllerde (Jaimini) verilir.
 #[derive(Debug, Deserialize)]
 #[serde(deny_unknown_fields)]
 struct Input {
     lagna_lon: Option<f64>,
     grahas: BTreeMap<Graha, f64>,
+    #[serde(default)]
+    advanced: Option<AdvancedSettings>,
 }
 
 impl Input {
@@ -226,3 +230,110 @@ fn drishti_hand_fixtures() {
         }
     }
 }
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ExpectedKaraka {
+    role: KarakaRole,
+    graha: Graha,
+    effective_deg: f64,
+    tie: bool,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ExpectedKarakamsha {
+    atmakaraka: Graha,
+    sign: Sign,
+    houses_in_d1: BTreeMap<Graha, u8>,
+    houses_in_d9: BTreeMap<Graha, u8>,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct JaiminiExpected {
+    /// Rol sırasında: sekizli şemada 8, yedili şemada 7 kayıt.
+    karakas: Vec<ExpectedKaraka>,
+    karakamsha: ExpectedKarakamsha,
+}
+
+#[test]
+fn jaimini_hand_fixtures() {
+    for (f, p) in load::<JaiminiExpected>("jaimini", MIN_JAIMINI) {
+        let id = &f.id;
+        let x = &f.expected;
+        let advanced = f
+            .input
+            .advanced
+            .unwrap_or_else(|| panic!("{id}: jaimini fixtures state the advanced settings"));
+
+        // Chara karaka.
+        let karakas = chara_karakas(&p, advanced.karaka_scheme);
+        assert_eq!(karakas.len(), x.karakas.len(), "{id}: karaka count");
+        for (got, want) in karakas.iter().zip(&x.karakas) {
+            assert_eq!(
+                (got.role, got.graha, got.tie),
+                (want.role, want.graha, want.tie),
+                "{id}: {:?}",
+                want.role
+            );
+            assert!(
+                (got.effective_deg - want.effective_deg).abs() < 1e-9,
+                "{id}: {:?} effective degree {}, hand value {}",
+                want.role,
+                got.effective_deg,
+                want.effective_deg
+            );
+        }
+
+        // Karakamsha.
+        assert_eq!(x.karakamsha.atmakaraka, x.karakas[0].graha, "{id}");
+        let k = karakamsha(&p, x.karakamsha.atmakaraka);
+        assert_eq!(k.sign, x.karakamsha.sign, "{id}: karakamsha");
+        assert_eq!(
+            (
+                x.karakamsha.houses_in_d1.len(),
+                x.karakamsha.houses_in_d9.len()
+            ),
+            (9, 9),
+            "{id}: nine grahas"
+        );
+        for g in Graha::ALL {
+            let i = g as usize;
+            assert_eq!(
+                k.houses_in_d1[i], x.karakamsha.houses_in_d1[&g],
+                "{id}: D1 house of {g:?}"
+            );
+            assert_eq!(
+                k.houses_in_d9[i], x.karakamsha.houses_in_d9[&g],
+                "{id}: D9 house of {g:?}"
+            );
+        }
+    }
+}
+
+/// Zorunlu durumlar (ileri modüller spec'i §5 "Test"): örnekler silinir ya da değiştirilirse
+/// kapsam sessizce daralmasın.
+#[test]
+fn jaimini_fixtures_cover_the_required_cases() {
+    let all = load::<JaiminiExpected>("jaimini", MIN_JAIMINI);
+    let any = |pred: &dyn Fn(&Fixture<JaiminiExpected>, &Positions) -> bool| {
+        all.iter().any(|(f, p)| pred(f, p))
+    };
+    assert!(
+        any(&|f, _| f.expected.karakas[0].graha == Graha::Rahu),
+        "Rahu as Atmakaraka"
+    );
+    assert!(
+        any(&|f, _| f.expected.karakas.iter().any(|k| k.tie)),
+        "a tie within one arcsecond"
+    );
+    assert!(
+        any(&|f, _| f.expected.karakas.len() == 7),
+        "the seven-karaka scheme"
+    );
+    assert!(
+        any(&|f, p| p.sign_of(f.expected.karakamsha.atmakaraka) == f.expected.karakamsha.sign),
+        "a vargottama Atmakaraka"
+    );
+}
````

- [ ] **Step 4: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-core 2>&1 | grep -E "^error" | sort | uniq -c
```
Expected: derleme hatası: ``error[E0432]: unresolved import `crate::jaimini` `` ve ``error[E0432]: unresolved imports `jyotish_core::chara_karakas`, `jyotish_core::karakamsha`, `jyotish_core::KarakaRole` ``.

- [ ] **Step 5: `jaimini/rashi.rs`**

Burç türü `index % 3`'tür (Koç 0: yürük, Boğa 1: sabit, İkizler 2: çift doğalı). Bakılan burçlar formülle bulunur; testteki `HAND_TABLE` on iki satırı elle yazar (spec §13 katman 3) ve ikisi her burçta eşit olmalıdır. Sayım yönü grupları spec §6'daki listedir (D5).

**Yeni dosya:** `crates/jyotish-core/src/jaimini/rashi.rs`

````rust
//! Burç türleri, sayım yönü ve rashi drishti (ileri modüller spec'i §5, §6).
//!
//! Kaynak: Jaimini Upadesha Sutraları, bakışlar bölümü. Yürük burç, kendisinden sonraki sabit
//! burç dışındaki üç sabit burca; sabit burç, kendisinden önceki yürük burç dışındaki üç yürük
//! burca; çift doğalı burç diğer üç çift doğalı burca bakar. Bir burçtaki grahalar o burcun
//! baktığı burçlara ve oradaki grahalara bakar. Burçlar rasi çerçevesindedir (K5).

use serde::{Deserialize, Serialize};

use crate::drishti::SignAspect;
use crate::graha::Graha;
use crate::positions::Positions;
use crate::zodiac::Sign;

/// Burcun türü: yürük (chara), sabit (sthira), çift doğalı (dwiswabhava).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum SignKind {
    Movable,
    Fixed,
    Dual,
}

/// Koç, Yengeç, Terazi, Oğlak yürük; Boğa, Aslan, Akrep, Kova sabit; diğer dördü çift doğalı.
pub fn sign_kind(sign: Sign) -> SignKind {
    match sign.index() % 3 {
        0 => SignKind::Movable,
        1 => SignKind::Fixed,
        _ => SignKind::Dual,
    }
}

/// Jaimini sayım yönü: Koç, Boğa, İkizler, Terazi, Akrep ve Yay'dan ileri (zodyak sırasıyla),
/// diğer altı burçtan geri sayılır. Eş lord seçiminin 5. ölçütü ve chara dasha (Alt proje 3.3)
/// bunu kullanır.
pub fn counts_forward(sign: Sign) -> bool {
    matches!(
        sign,
        Sign::Aries | Sign::Taurus | Sign::Gemini | Sign::Libra | Sign::Scorpio | Sign::Sagittarius
    )
}

/// `from`'dan `to`'ya, `from`'un sayım yönünde kapsayıcı sayım, 1..=12 (aynı burç 1).
pub fn directed_count(from: Sign, to: Sign) -> u8 {
    if counts_forward(from) {
        to.house_from(from)
    } else {
        from.house_from(to)
    }
}

/// `from` burcunun rashi drishti ile baktığı üç burç, zodyak sırasıyla `from`'dan başlayarak.
pub fn rashi_drishti_signs(from: Sign) -> [Sign; 3] {
    let offsets: [usize; 3] = match sign_kind(from) {
        SignKind::Movable => [4, 7, 10],
        SignKind::Fixed => [2, 5, 8],
        SignKind::Dual => [3, 6, 9],
    };
    offsets.map(|o| Sign::from_index(from.index() + o))
}

/// `matrix[from][to]`: `from` burcu `to` burcuna bakıyor mu. Dizinler `Sign::ALL` sırasıdır.
pub fn rashi_drishti_matrix() -> [[bool; 12]; 12] {
    let mut matrix = [[false; 12]; 12];
    for from in Sign::ALL {
        for to in rashi_drishti_signs(from) {
            matrix[from.index()][to.index()] = true;
        }
    }
    matrix
}

/// Dokuz grahanın rashi drishtisi: `Graha::ALL` sırasında, her graha için üç hedef burç ve
/// oradaki grahalar. Her zaman 27 kayıt.
pub fn graha_rashi_drishti(positions: &Positions) -> Vec<SignAspect> {
    Graha::ALL
        .into_iter()
        .flat_map(|graha| {
            rashi_drishti_signs(positions.sign_of(graha))
                .into_iter()
                .map(move |target| SignAspect {
                    graha,
                    target,
                    occupants: positions.occupants(target),
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use Sign::*;

    /// Elle yazılmış tablo (bağımsız ikinci yazım, spec §13): her burcun baktığı üç burç.
    const HAND_TABLE: [(Sign, [Sign; 3]); 12] = [
        (Aries, [Leo, Scorpio, Aquarius]),
        (Taurus, [Cancer, Libra, Capricorn]),
        (Gemini, [Virgo, Sagittarius, Pisces]),
        (Cancer, [Scorpio, Aquarius, Taurus]),
        (Leo, [Libra, Capricorn, Aries]),
        (Virgo, [Sagittarius, Pisces, Gemini]),
        (Libra, [Aquarius, Taurus, Leo]),
        (Scorpio, [Capricorn, Aries, Cancer]),
        (Sagittarius, [Pisces, Gemini, Virgo]),
        (Capricorn, [Taurus, Leo, Scorpio]),
        (Aquarius, [Aries, Cancer, Libra]),
        (Pisces, [Gemini, Virgo, Sagittarius]),
    ];

    #[test]
    fn sign_kinds_follow_the_zodiac_in_threes() {
        for s in [Aries, Cancer, Libra, Capricorn] {
            assert_eq!(sign_kind(s), SignKind::Movable, "{s:?}");
        }
        for s in [Taurus, Leo, Scorpio, Aquarius] {
            assert_eq!(sign_kind(s), SignKind::Fixed, "{s:?}");
        }
        for s in [Gemini, Virgo, Sagittarius, Pisces] {
            assert_eq!(sign_kind(s), SignKind::Dual, "{s:?}");
        }
    }

    #[test]
    fn rashi_drishti_matches_the_hand_table() {
        for (from, targets) in HAND_TABLE {
            assert_eq!(rashi_drishti_signs(from), targets, "{from:?}");
        }
        let matrix = rashi_drishti_matrix();
        for (from, targets) in HAND_TABLE {
            for to in Sign::ALL {
                assert_eq!(
                    matrix[from.index()][to.index()],
                    targets.contains(&to),
                    "{from:?} → {to:?}"
                );
            }
        }
    }

    #[test]
    fn rashi_drishti_is_symmetric_and_skips_the_adjacent_sign() {
        let matrix = rashi_drishti_matrix();
        for from in Sign::ALL {
            let row = matrix[from.index()];
            assert_eq!(row.iter().filter(|&&b| b).count(), 3, "{from:?}");
            assert!(!row[from.index()], "{from:?} does not aspect itself");
            for to in Sign::ALL {
                assert_eq!(
                    matrix[from.index()][to.index()],
                    matrix[to.index()][from.index()],
                    "{from:?} ↔ {to:?}"
                );
                if !row[to.index()] {
                    continue;
                }
                match sign_kind(from) {
                    SignKind::Movable => assert_eq!(sign_kind(to), SignKind::Fixed),
                    SignKind::Fixed => assert_eq!(sign_kind(to), SignKind::Movable),
                    SignKind::Dual => assert_eq!(sign_kind(to), SignKind::Dual),
                }
            }
        }
        // Bitişik burca bakış yoktur: yürük burç için sonraki, sabit burç için önceki burç.
        for s in [Aries, Cancer, Libra, Capricorn] {
            assert!(!matrix[s.index()][(s.index() + 1) % 12], "{s:?}");
        }
        for s in [Taurus, Leo, Scorpio, Aquarius] {
            assert!(!matrix[s.index()][(s.index() + 11) % 12], "{s:?}");
        }
    }

    #[test]
    fn counting_direction_and_directed_count() {
        let forward: Vec<Sign> = Sign::ALL
            .into_iter()
            .filter(|&s| counts_forward(s))
            .collect();
        assert_eq!(
            forward,
            [Aries, Taurus, Gemini, Libra, Scorpio, Sagittarius]
        );
        // Akrep'ten ileri: Akrep 1, Yay 2, Oğlak 3.
        assert_eq!(directed_count(Scorpio, Scorpio), 1);
        assert_eq!(directed_count(Scorpio, Capricorn), 3);
        assert_eq!(directed_count(Scorpio, Libra), 12);
        // Kova'dan geri: Kova 1, Oğlak 2, Yay 3; Balık 12.
        assert_eq!(directed_count(Aquarius, Aquarius), 1);
        assert_eq!(directed_count(Aquarius, Sagittarius), 3);
        assert_eq!(directed_count(Aquarius, Pisces), 12);
    }

    #[test]
    fn grahas_aspect_the_signs_their_sign_aspects() {
        // Güneş ve Merkür Koç'ta, Ay Yengeç'te, Mars Aslan'da, Jüpiter Yay'da, Venüs Boğa'da,
        // Satürn Kova'da, Rahu Akrep'te, Ketu Boğa'da.
        let p = Positions::synthetic(
            None,
            [10.0, 95.0, 130.0, 20.0, 250.0, 40.0, 300.0, 220.0, 40.0],
        );
        let aspects = graha_rashi_drishti(&p);
        assert_eq!(aspects.len(), 27);
        for (i, a) in aspects.iter().enumerate() {
            assert_eq!(a.graha, Graha::ALL[i / 3]);
            assert_eq!(a.target, rashi_drishti_signs(p.sign_of(a.graha))[i % 3]);
            assert_eq!(a.occupants, p.occupants(a.target));
        }
        // Güneş Koç'tan Aslan'a (Mars), Akrep'e (Rahu) ve Kova'ya (Satürn) bakar.
        assert_eq!(aspects[0].target, Leo);
        assert_eq!(aspects[0].occupants, [Graha::Mars]);
        assert_eq!(aspects[1].occupants, [Graha::Rahu]);
        assert_eq!(aspects[2].occupants, [Graha::Saturn]);
        // Jüpiter Yay'dan Balık, İkizler ve Başak'a bakar: üçü de boş.
        assert!(aspects[12..15].iter().all(|a| a.occupants.is_empty()));
    }
}
````

- [ ] **Step 6: `jaimini/karaka.rs`**

Sıralama kararlıdır: adaylar `Graha::ALL` sırasıyla toplanır (ilk yedisi Güneş…Satürn, sekizincisi Rahu) ve etkin dereceye göre büyükten küçüğe dizilir; tam eşit derecede doğal sıra durur. Karakamsha için navamsa burcu, rasi çerçevesindeki burç ve burç içi dereceden yeniden kurulan boylamla v1'in `varga_position` fonksiyonundan alınır (haritanın D9'uyla aynı kural; Task 4 bunu gerçek haritayla çapraz denetler).

**Yeni dosya:** `crates/jyotish-core/src/jaimini/karaka.rs`

````rust
//! Chara karaka ve karakamsha (ileri modüller spec'i §5).
//!
//! Kaynak: BPHS, karaka bölümü; Jaimini Upadesha Sutraları. Grahalar burç içi derecelerine göre
//! büyükten küçüğe sıralanır. Sekizli şemada Rahu da sıralanır ve derecesi burcun sonundan
//! ölçülür (30 − derece); yedili şemada Rahu yoktur. Ketu karaka olmaz. Karaka birleşmesi
//! kuralları uygulanmaz. Derece rasi çerçevesindedir (K5).

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::positions::{GrahaPoint, Positions};
use crate::settings::KarakaScheme;
use crate::varga::{varga_position, Varga};
use crate::zodiac::Sign;

/// Eşitlik eşiği: iki etkin derece bu fark içindeyse (1″) ikisi de `tie: true` taşır.
pub const KARAKA_TIE_DEG: f64 = 1.0 / 3600.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum KarakaRole {
    Atma,
    Amatya,
    Bhratri,
    Matri,
    Pitri,
    Putra,
    Gnati,
    Dara,
}

/// Sekizli şemanın rolleri, sırayla.
pub const EIGHT_KARAKA_ROLES: [KarakaRole; 8] = [
    KarakaRole::Atma,
    KarakaRole::Amatya,
    KarakaRole::Bhratri,
    KarakaRole::Matri,
    KarakaRole::Pitri,
    KarakaRole::Putra,
    KarakaRole::Gnati,
    KarakaRole::Dara,
];

/// Yedili şemanın rolleri, sırayla (Pitri yoktur).
pub const SEVEN_KARAKA_ROLES: [KarakaRole; 7] = [
    KarakaRole::Atma,
    KarakaRole::Amatya,
    KarakaRole::Bhratri,
    KarakaRole::Matri,
    KarakaRole::Putra,
    KarakaRole::Gnati,
    KarakaRole::Dara,
];

/// Şemanın rolleri, sırayla.
pub fn karaka_roles(scheme: KarakaScheme) -> &'static [KarakaRole] {
    match scheme {
        KarakaScheme::Eight => &EIGHT_KARAKA_ROLES,
        KarakaScheme::Seven => &SEVEN_KARAKA_ROLES,
    }
}

/// Bir rolün grahası.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CharaKaraka {
    pub role: KarakaRole,
    pub graha: Graha,
    /// Sıralamada kullanılan derece: burç içi derece, [0, 30); Rahu için 30 − derece, (0, 30].
    pub effective_deg: f64,
    /// Etkin derecesi sıradaki komşusuyla 1″ içinde eşit.
    pub tie: bool,
}

/// Grahanın sıralamada kullanılan derecesi.
fn effective_deg(point: &GrahaPoint) -> f64 {
    if point.graha == Graha::Rahu {
        30.0 - point.deg_in_sign
    } else {
        point.deg_in_sign
    }
}

/// Chara karaka sırası: `scheme`'e göre 8 ya da 7 kayıt, rol sırasında. Eşit derecede sıra tam
/// `f64` değeriyle, o da eşitse Güneş…Rahu doğal sırasıyla belirlenir.
pub fn chara_karakas(positions: &Positions, scheme: KarakaScheme) -> Vec<CharaKaraka> {
    let roles = karaka_roles(scheme);
    // `Graha::ALL`'un ilk yedisi Güneş…Satürn, sekizincisi Rahu'dur.
    let mut ranked: Vec<(Graha, f64)> = positions.grahas[..roles.len()]
        .iter()
        .map(|p| (p.graha, effective_deg(p)))
        .collect();
    // Kararlı sıralama: eşit derecede doğal sıra korunur.
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    let near = |i: usize, j: usize| (ranked[i].1 - ranked[j].1).abs() <= KARAKA_TIE_DEG;
    (0..ranked.len())
        .map(|i| CharaKaraka {
            role: roles[i],
            graha: ranked[i].0,
            effective_deg: ranked[i].1,
            tie: (i > 0 && near(i, i - 1)) || (i + 1 < ranked.len() && near(i, i + 1)),
        })
        .collect()
}

/// Karakamsha: Atmakaraka'nın navamsa burcu ve grahaların ondan ev sayıları.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Karakamsha {
    pub atmakaraka: Graha,
    /// Atmakaraka'nın navamsa (D9) burcu.
    pub sign: Sign,
    /// Karakamsha rasi haritasına taşınmış: her grahanın D1 burcunun karakamshadan ev sayısı,
    /// `Graha::ALL` sırasında.
    pub houses_in_d1: [u8; 9],
    /// Swamsha: her grahanın D9 burcunun karakamshadan ev sayısı, `Graha::ALL` sırasında.
    pub houses_in_d9: [u8; 9],
}

/// Grahanın navamsa burcu, rasi çerçevesindeki konumundan (haritanın D9'uyla aynı kural).
pub fn navamsa_sign(point: &GrahaPoint) -> Sign {
    let lon = point.rasi_sign.index() as f64 * 30.0 + point.deg_in_sign;
    varga_position(Varga::D9, lon).0
}

/// `atmakaraka`'nın karakamshası. Lagnaya bağlı değildir.
pub fn karakamsha(positions: &Positions, atmakaraka: Graha) -> Karakamsha {
    let sign = navamsa_sign(positions.point(atmakaraka));
    Karakamsha {
        atmakaraka,
        sign,
        houses_in_d1: std::array::from_fn(|i| positions.grahas[i].rasi_sign.house_from(sign)),
        houses_in_d9: std::array::from_fn(|i| navamsa_sign(&positions.grahas[i]).house_from(sign)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;

    fn order(k: &[CharaKaraka]) -> Vec<Graha> {
        k.iter().map(|c| c.graha).collect()
    }

    #[test]
    fn eight_scheme_ranks_rahu_from_the_end_of_its_sign() {
        // Burç içi dereceler: Güneş 10, Ay 5, Mars 25, Merkür 20, Jüpiter 15, Venüs 12,
        // Satürn 2, Rahu 4 (etkin 26), Ketu 4.
        let p = Positions::synthetic(
            None,
            [10.0, 95.0, 145.0, 200.0, 255.0, 42.0, 302.0, 124.0, 304.0],
        );
        let k = chara_karakas(&p, KarakaScheme::Eight);
        assert_eq!(
            order(&k),
            [Rahu, Mars, Mercury, Jupiter, Venus, Sun, Moon, Saturn]
        );
        assert_eq!(
            k.iter().map(|c| c.role).collect::<Vec<_>>(),
            EIGHT_KARAKA_ROLES
        );
        assert_eq!(k[0].effective_deg, 26.0);
        assert_eq!(k[1].effective_deg, 25.0);
        assert!(k.iter().all(|c| !c.tie));
        // Yedili şema: Rahu yok, Pitri rolü yok.
        let k7 = chara_karakas(&p, KarakaScheme::Seven);
        assert_eq!(
            order(&k7),
            [Mars, Mercury, Jupiter, Venus, Sun, Moon, Saturn]
        );
        assert_eq!(
            k7.iter().map(|c| c.role).collect::<Vec<_>>(),
            SEVEN_KARAKA_ROLES
        );
        assert!(!SEVEN_KARAKA_ROLES.contains(&KarakaRole::Pitri));
    }

    #[test]
    fn ties_within_one_arcsecond_are_flagged_and_ordered() {
        let sec = 1.0 / 3600.0;
        // Mars 20°, Jüpiter 20° − 0,5″: 1″ içinde. Venüs 20° − 2,5″: Jüpiter'e 2″ uzak.
        let p = Positions::synthetic(
            None,
            [
                5.0,
                40.0,
                80.0,
                101.0,
                140.0 - 0.5 * sec,
                170.0 - 2.5 * sec,
                212.0,
                273.0,
                93.0,
            ],
        );
        let k = chara_karakas(&p, KarakaScheme::Eight);
        // Rahu etkin 27; Mars 20; Jüpiter; Venüs; Satürn 2; ...
        assert_eq!(order(&k)[..4], [Rahu, Mars, Jupiter, Venus]);
        assert_eq!(
            k.iter().map(|c| c.tie).collect::<Vec<_>>(),
            [false, true, true, false, false, false, false, false]
        );
        // Tam eşit derecede doğal sıra: Güneş, Ay'dan önce.
        let q = Positions::synthetic(
            None,
            [48.0, 18.0, 80.0, 101.0, 125.0, 157.0, 212.0, 273.0, 93.0],
        );
        let k = chara_karakas(&q, KarakaScheme::Seven);
        assert_eq!(order(&k)[..2], [Mars, Sun]);
        assert_eq!(order(&k)[2], Moon);
        assert_eq!((k[1].tie, k[2].tie, k[0].tie), (true, true, false));
    }

    #[test]
    fn rahu_at_the_start_of_a_sign_has_thirty_effective_degrees() {
        let p = Positions::synthetic(
            None,
            [29.5, 40.0, 80.0, 101.0, 142.0, 173.0, 212.0, 270.0, 90.0],
        );
        let k = chara_karakas(&p, KarakaScheme::Eight);
        assert_eq!((k[0].graha, k[0].effective_deg), (Rahu, 30.0));
        assert_eq!(k[1].graha, Sun);
    }

    #[test]
    fn karakamsha_counts_houses_from_the_navamsa_sign_of_the_atmakaraka() {
        // Mars Aslan 25°: ateş burcu Koç'tan başlar; 25° sekizinci parça (23°20′–26°40′):
        // Koç'tan 8. burç Akrep. Güneş Koç 11°: dördüncü parça (10°–13°20′): Yengeç.
        let p = Positions::synthetic(
            None,
            [11.0, 95.0, 145.0, 200.0, 255.0, 42.0, 302.0, 124.0, 304.0],
        );
        assert_eq!(navamsa_sign(p.point(Mars)), Sign::Scorpio);
        assert_eq!(navamsa_sign(p.point(Sun)), Sign::Cancer);
        let k = karakamsha(&p, Mars);
        assert_eq!((k.atmakaraka, k.sign), (Mars, Sign::Scorpio));
        // D1: Güneş Koç, Akrep'ten 6. burç; Mars Aslan, Akrep'ten 10.
        assert_eq!(k.houses_in_d1[Sun as usize], 6);
        assert_eq!(k.houses_in_d1[Mars as usize], 10);
        // D9: Mars'ın navamsa burcu karakamshanın kendisidir (1); Güneş Yengeç, Akrep'ten 9.
        assert_eq!(k.houses_in_d9[Mars as usize], 1);
        assert_eq!(k.houses_in_d9[Sun as usize], 9);
        for h in k.houses_in_d1.iter().chain(&k.houses_in_d9) {
            assert!((1..=12).contains(h));
        }
    }
}
````

- [ ] **Step 7: Modülü kaydet ve dışa aktar**

**Yeni dosya:** `crates/jyotish-core/src/jaimini/mod.rs`

````rust
//! Jaimini çekirdeği (ileri modüller spec'i §5): chara karaka, karakamsha ve rashi drishti.
//! Hesap fonksiyonları yalnızca `Positions` görür (K3).

mod karaka;
mod rashi;

pub use karaka::{
    chara_karakas, karaka_roles, karakamsha, navamsa_sign, CharaKaraka, KarakaRole, Karakamsha,
    EIGHT_KARAKA_ROLES, KARAKA_TIE_DEG, SEVEN_KARAKA_ROLES,
};
pub use rashi::{
    counts_forward, directed_count, graha_rashi_drishti, rashi_drishti_matrix, rashi_drishti_signs,
    sign_kind, SignKind,
};
````

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 63ad8dd..a8a43fd 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -3,7 +3,8 @@
 //!
 //! Kapsam: ayanamsa, grahalar, lagna, bhavalar, nakshatra, dignity, yanma (M2), 16 varga,
 //! Vimshottari dasha, panchang, ashtakavarga ve transit (M3). İleri modüllerin ortak temeli
-//! (Alt proje 3.1): `positions` (ortak girdi), `lordship`, `nature` ve `drishti`.
+//! (Alt proje 3.1): `positions` (ortak girdi), `lordship`, `nature` ve `drishti`. Jaimini
+//! çekirdeği (Alt proje 3.2): `jaimini`.
 
 pub mod ashtakavarga;
 mod chart;
@@ -16,6 +17,7 @@ mod error;
 mod fake;
 mod graha;
 mod houses;
+pub mod jaimini;
 pub mod lordship;
 pub mod nature;
 pub mod panchang;
@@ -45,6 +47,11 @@ pub use drishti::{
 pub use error::CoreError;
 pub use graha::{Graha, GrahaPosition};
 pub use houses::{house_from_cusps, whole_sign_cusps, whole_sign_house, Bhavas, Lagna};
+pub use jaimini::{
+    chara_karakas, counts_forward, directed_count, graha_rashi_drishti, karaka_roles, karakamsha,
+    navamsa_sign, rashi_drishti_matrix, rashi_drishti_signs, sign_kind, CharaKaraka, KarakaRole,
+    Karakamsha, SignKind, EIGHT_KARAKA_ROLES, KARAKA_TIE_DEG, SEVEN_KARAKA_ROLES,
+};
 pub use lordship::{house_lord, house_sign, houses_owned};
 pub use nature::{kuja_dosha, moon_is_waxing, nature, KujaDosha, Nature, KUJA_DOSHA_HOUSES};
 pub use panchang::{
````

- [ ] **Step 8: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p jyotish-core 2>&1 | grep -E "^test result|Running"
```
Expected: fmt ve clippy çıktısız; lib `151 passed` (+12: 5 `rashi`, 4 `karaka`, 3 proptest); golden `8 passed`; `modules_golden` `5 passed` (+2). Bir el örneği düşerse önce `derivation` kuraldan yeniden hesaplanır; beklenen değer motor çıktısına uydurulmaz (Global Constraints).

- [ ] **Step 9: Commit**

```bash
git add crates/jyotish-core/src crates/jyotish-core/tests/modules_golden.rs tests/golden/modules
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): jaimini sign kinds and rashi drishti, chara karaka order for both schemes, karakamsha; hand fixtures"
```
Expected: iki `grep` çıktısız.

---

### Task 2: Çekirdek: Jaimini eş lordları ve arudha padalar

Spec §5 "Eş lordlar", "Arudha pada"; D5–D8. `jaimini_co_lords` ayarı burada hesaba girer: açıkken Akrep ve Kova evlerinin lordu altı ölçütle seçilen eş lorddur, devre dışıyken Mars ve Satürn'dür. Padalar lagnaya bağlıdır: lagna yoksa `None` (K2).

**Files:**
- Create: `crates/jyotish-core/src/jaimini/lords.rs`, `crates/jyotish-core/src/jaimini/arudha.rs`
- Modify: `crates/jyotish-core/src/jaimini/mod.rs`, `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/proptests.rs`, `crates/jyotish-core/tests/modules_golden.rs`, `tests/golden/modules/README.md`, `tests/golden/modules/jaimini/*.json` (8 dosya)
- Test: `lords.rs` ve `arudha.rs` içindeki `mod tests`, `proptests.rs`, `tests/modules_golden.rs`

**Interfaces:**
- Consumes: Task 1'in `sign_kind`, `SignKind`, `directed_count`; `dignity::{sign_lord, exaltation_point, debilitation_point}` (yücelme: Mars Oğlak, Satürn Terazi; düşüş: Mars Yengeç, Satürn Koç); `lordship::house_sign`; `Positions::{sign_of, occupants, lagna_sign}`.
- Produces (hepsi `jyotish_core`'dan dışa aktarılır):
  - `pub fn co_lords(sign: Sign) -> Option<(Graha, Graha)>` (Akrep: `(Mars, Ketu)`, Kova: `(Saturn, Rahu)`; diğerleri `None`)
  - `pub struct CoLordChoice { pub lord: Graha, pub step: u8 }` (`Copy`, `Eq`), `pub fn co_lord_choice(sign: Sign, positions: &Positions) -> Option<CoLordChoice>` (`step` 1..=6)
  - `pub fn stronger_co_lord(sign: Sign, positions: &Positions) -> Graha` (eş lordsuz burçta `sign_lord`), `pub fn jaimini_lord(sign: Sign, positions: &Positions, co_lords: bool) -> Graha`
  - `pub struct ArudhaPada { pub house: u8, pub sign: Sign, pub lord: Graha, pub exception: bool }` (`Copy`, `Eq`), `pub fn arudha_of(sign: Sign, lord_sign: Sign) -> (Sign, bool)`, `pub fn arudha_padas(positions: &Positions, co_lords: bool) -> Option<[ArudhaPada; 12]>`
  - Düzenek: `JaiminiExpected` alanları `co_lords: BTreeMap<Sign, ExpectedCoLord>`, `arudhas: Option<Vec<ExpectedPada>>`.

- [ ] **Step 1: El örneklerine eş lord ve pada bölümlerini ekle (kaynak: `hand`)**

Her dosyaya "Eş lord" ve "Pada" türetme adımları ile `co_lords` ve `arudhas` beklentileri eklenir. Eş lord seçimi ayar devre dışıyken de yazılır (`atmakaraka-vargottama-co-lords-off`: padalar Mars ve Satürn'le hesaplanır, seçim kuralı ayrıca izlenir). Altı ölçütün her biri en az bir dosyada belirleyicidir (Task 1 adım 2'deki tablo); lordun evden 1., 4., 7. ve 10. burçta olduğu istisnalar `rahu-atmakaraka-arudha-exceptions`'ta bir aradadır.

Arudha kuralı (türetmelerde kullanılan biçim): evin burcu `S`, lordu `T` burcunda; `n`, `S`'den `T`'ye kapsayıcı sayım; hesaplanan burç `T`'den `n`. burçtur. Hesaplanan burç `S`'nin kendisi (lord 1. ya da 7. burçta) ya da `S`'den 7. burç (lord 4. ya da 10. burçta) ise pada hesaplanan burçtan 10. burçtur.

````diff
diff --git a/tests/golden/modules/README.md b/tests/golden/modules/README.md
index 9e7fac0..8936ce2 100644
--- a/tests/golden/modules/README.md
+++ b/tests/golden/modules/README.md
@@ -42,6 +42,6 @@ yazılmaz.
 |---|---|---|
 | `foundation` | `waxing`, `nature` (dokuz graha), `kuja_dosha`, `house_lords` (12), `houses`, `houses_owned` | 6 |
 | `drishti` | `sign_aspects` (on üç tam bakışın tamamı, tablo sırasıyla), `sphuta` (en az on değer: `from`, `to`, `angle`, `value`) | 4 |
-| `jaimini` | `karakas` (rol sırasında: `role`, `graha`, `effective_deg`, `tie`), `karakamsha` (`atmakaraka`, `sign`, dokuz grahanın `houses_in_d1` ve `houses_in_d9` değeri) | 8 |
+| `jaimini` | `karakas` (rol sırasında: `role`, `graha`, `effective_deg`, `tie`), `karakamsha` (`atmakaraka`, `sign`, dokuz grahanın `houses_in_d1` ve `houses_in_d9` değeri), `co_lords` (Akrep ve Kova: `lord`, belirleyici ölçüt `step`), `arudhas` (on iki pada: `house`, `sign`, `lord`, `exception`; saat bilinmiyorsa `null`) | 8 |
 
 Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
diff --git a/tests/golden/modules/jaimini/atmakaraka-vargottama-co-lords-off.json b/tests/golden/modules/jaimini/atmakaraka-vargottama-co-lords-off.json
index 76fcc21..7241d72 100644
--- a/tests/golden/modules/jaimini/atmakaraka-vargottama-co-lords-off.json
+++ b/tests/golden/modules/jaimini/atmakaraka-vargottama-co-lords-off.json
@@ -7,7 +7,11 @@
     "Karaka (sekizli şema): Rahu'nun etkin derecesi 30 − 16 = 14. Büyükten küçüğe: Jüpiter 28, Merkür 25, Satürn 22, Güneş 19, Rahu 14, Mars 13, Ay 6, Venüs 3. Eşitlik yok.",
     "Karakamsha: Atmakaraka Jüpiter Balık 28°. Su burcu Yengeç'ten sayılır; 28° dokuzuncu parçadadır (26°40′–30°): Yengeç'ten 9. burç Balık. Navamsa burcu rasi burcuyla aynı: Atmakaraka vargottamadır.",
     "D9: Güneş Kova 19° (hava, Terazi'den; 6. parça 16°40′–20°): Balık. Ay Aslan 6° (ateş, Koç'tan; 2. parça 3°20′–6°40′): Boğa. Mars Boğa 13° (toprak, Oğlak'tan; 4. parça 10°–13°20′): Koç. Merkür Oğlak 25° (toprak; 8. parça 23°20′–26°40′): Aslan. Jüpiter: Balık. Venüs Balık 3° (su; 1. parça): Yengeç. Satürn Terazi 22° (hava; 7. parça 20°–23°20′): Koç. Rahu Yengeç 16° (su; 5. parça 13°20′–16°40′): Akrep. Ketu Oğlak 16° (toprak; 5. parça): Boğa.",
-    "D1 evleri, Balık'tan (Güneş…Ketu): Kova 12, Aslan 6, Boğa 3, Oğlak 11, Balık 1, Balık 1, Terazi 8, Yengeç 5, Oğlak 11. D9 evleri, Balık'tan: Balık 1, Boğa 3, Koç 2, Aslan 6, Balık 1, Yengeç 5, Koç 2, Akrep 9, Boğa 3."
+    "D1 evleri, Balık'tan (Güneş…Ketu): Kova 12, Aslan 6, Boğa 3, Oğlak 11, Balık 1, Balık 1, Terazi 8, Yengeç 5, Oğlak 11. D9 evleri, Balık'tan: Balık 1, Boğa 3, Koç 2, Aslan 6, Balık 1, Yengeç 5, Koç 2, Akrep 9, Boğa 3.",
+    "Eş lord: ayar devre dışı; padalarda Akrep'in lordu Mars, Kova'nın lordu Satürn'dür. Seçim kuralı yine de izlenir (ayar açık olsaydı kullanılırdı). Akrep (Mars Boğa'da, Ketu Oğlak'ta): 1. ölçüt ayırmaz; 2. ölçüt: Mars yalnız (0), Ketu Merkür'le (1): Ketu. Kova (Satürn Terazi'de, Rahu Yengeç'te): 1. ölçüt ayırmaz; 2. ölçüt: ikisi de yalnız; 3. ölçüt: Satürn yücelme burcunda (Terazi): Satürn.",
+    "Pada, Akrep lagna. 1. ev Akrep, Mars Boğa'da (7.): Boğa'dan 7. burç Akrep, evin kendisi; istisna: Akrep'ten 10. burç Aslan. 2. ev Yay, Jüpiter Balık'ta (4.): Balık'tan 4. burç İkizler, evden 7.; istisna: İkizler'den 10. burç Balık. 3. ev Oğlak, Satürn Terazi'de (10.): Terazi'den 10. burç Yengeç, evden 7.; istisna: Yengeç'ten 10. burç Koç. 4. ev Kova, Satürn Terazi'de (9.): Terazi'den 9. burç İkizler.",
+    "Pada (5–8). 5. ev Balık, Jüpiter Balık'ta (1.): istisna: Balık'tan 10. burç Yay. 6. ev Koç, Mars Boğa'da (2.): Boğa'dan 2. burç İkizler. 7. ev Boğa, Venüs Balık'ta (11.): Balık'tan 11. burç Oğlak. 8. ev İkizler, Merkür Oğlak'ta (8.): Oğlak'tan 8. burç Aslan.",
+    "Pada (9–12). 9. ev Yengeç, Ay Aslan'da (2.): Aslan'dan 2. burç Başak. 10. ev Aslan, Güneş Kova'da (7.): Kova'dan 7. burç Aslan, evin kendisi; istisna: Aslan'dan 10. burç Boğa. 11. ev Başak, Merkür Oğlak'ta (5.): Oğlak'tan 5. burç Boğa. 12. ev Terazi, Venüs Balık'ta (6.): Balık'tan 6. burç Aslan."
   ],
   "input": {
     "lagna_lon": 220.0,
@@ -29,6 +33,21 @@
       "atmakaraka": "Jupiter", "sign": "Pisces",
       "houses_in_d1": {"Sun": 12, "Moon": 6, "Mars": 3, "Mercury": 11, "Jupiter": 1, "Venus": 1, "Saturn": 8, "Rahu": 5, "Ketu": 11},
       "houses_in_d9": {"Sun": 1, "Moon": 3, "Mars": 2, "Mercury": 6, "Jupiter": 1, "Venus": 5, "Saturn": 2, "Rahu": 9, "Ketu": 3}
-    }
+    },
+    "co_lords": {"Scorpio": {"lord": "Ketu", "step": 2}, "Aquarius": {"lord": "Saturn", "step": 3}},
+    "arudhas": [
+      {"house": 1, "sign": "Leo", "lord": "Mars", "exception": true},
+      {"house": 2, "sign": "Pisces", "lord": "Jupiter", "exception": true},
+      {"house": 3, "sign": "Aries", "lord": "Saturn", "exception": true},
+      {"house": 4, "sign": "Gemini", "lord": "Saturn", "exception": false},
+      {"house": 5, "sign": "Sagittarius", "lord": "Jupiter", "exception": true},
+      {"house": 6, "sign": "Gemini", "lord": "Mars", "exception": false},
+      {"house": 7, "sign": "Capricorn", "lord": "Venus", "exception": false},
+      {"house": 8, "sign": "Leo", "lord": "Mercury", "exception": false},
+      {"house": 9, "sign": "Virgo", "lord": "Moon", "exception": false},
+      {"house": 10, "sign": "Taurus", "lord": "Sun", "exception": true},
+      {"house": 11, "sign": "Taurus", "lord": "Mercury", "exception": false},
+      {"house": 12, "sign": "Leo", "lord": "Venus", "exception": false}
+    ]
   }
 }
diff --git a/tests/golden/modules/jaimini/co-lord-company-and-forward-count.json b/tests/golden/modules/jaimini/co-lord-company-and-forward-count.json
index 41b34ae..f7e38f2 100644
--- a/tests/golden/modules/jaimini/co-lord-company-and-forward-count.json
+++ b/tests/golden/modules/jaimini/co-lord-company-and-forward-count.json
@@ -7,7 +7,12 @@
     "Karaka (sekizli şema): Rahu'nun etkin derecesi 30 − 21 = 9. Büyükten küçüğe: Venüs 25, Merkür 23, Satürn 17, Mars 14, Güneş 12, Rahu 9, Jüpiter 5, Ay 2. Eşitlik yok.",
     "Karakamsha: Atmakaraka Venüs İkizler 25°. Hava burcu Terazi'den sayılır; 25° sekizinci parçadadır (23°20′–26°40′): Terazi'den 8. burç Boğa.",
     "D9: Güneş Boğa 12° (toprak, Oğlak'tan; 4. parça 10°–13°20′): Koç. Ay Kova 2° (hava; 1. parça): Terazi. Mars Aslan 14° (ateş, Koç'tan; 5. parça 13°20′–16°40′): Aslan. Merkür Boğa 23° (toprak; 7. parça 20°–23°20′): Yengeç. Jüpiter İkizler 5° (hava; 2. parça 3°20′–6°40′): Akrep. Venüs: Boğa. Satürn İkizler 17° (hava; 6. parça 16°40′–20°): Balık. Rahu Aslan 21° (ateş; 7. parça): Terazi. Ketu Kova 21° (hava; 7. parça): Koç.",
-    "D1 evleri, Boğa'dan (Güneş…Ketu): Boğa 1, Kova 10, Aslan 4, Boğa 1, İkizler 2, İkizler 2, İkizler 2, Aslan 4, Kova 10. D9 evleri, Boğa'dan: Koç 12, Terazi 6, Aslan 4, Yengeç 3, Akrep 7, Boğa 1, Balık 11, Terazi 6, Koç 12."
+    "D1 evleri, Boğa'dan (Güneş…Ketu): Boğa 1, Kova 10, Aslan 4, Boğa 1, İkizler 2, İkizler 2, İkizler 2, Aslan 4, Kova 10. D9 evleri, Boğa'dan: Koç 12, Terazi 6, Aslan 4, Yengeç 3, Akrep 7, Boğa 1, Balık 11, Terazi 6, Koç 12.",
+    "Eş lord, Akrep (Mars Aslan'da, Ketu Kova'da): 1. ölçüt ayırmaz. 2. ölçüt: Mars Rahu'yla (Rahu Akrep'in eş lordu değildir, sayılır: 1), Ketu Ay'la (1): eşit. 3. ölçüt: Mars Aslan'da ne yücelmiş ne düşmüş. 4. ölçüt: Aslan ve Kova sabit: eşit. 5. ölçüt: Akrep ileri sayar: Akrep 1, Yay 2, Oğlak 3, Kova 4 (Ketu); Balık 5, Koç 6, Boğa 7, İkizler 8, Yengeç 9, Aslan 10 (Mars): Mars daha uzak.",
+    "Eş lord, Kova (Satürn İkizler'de, Rahu Aslan'da): 1. ölçüt ayırmaz (Kova'daki Ketu Kova'nın eş lordu değildir). 2. ölçüt: Satürn Jüpiter ve Venüs'le (2), Rahu Mars'la (1): Satürn.",
+    "Pada, Başak lagna. 1. ev Başak, Merkür Boğa'da (9.): Boğa'dan 9. burç Oğlak. 2. ev Terazi, Venüs İkizler'de (9.): İkizler'den 9. burç Kova. 3. ev Akrep, lord Mars Aslan'da (10.): Aslan'dan 10. burç Boğa, evden 7.; istisna: Boğa'dan 10. burç Kova. 4. ev Yay, Jüpiter İkizler'de (7.): İkizler'den 7. burç Yay, evin kendisi; istisna: Yay'dan 10. burç Başak.",
+    "Pada (5–8). 5. ev Oğlak, Satürn İkizler'de (6.): İkizler'den 6. burç Akrep. 6. ev Kova, lord Satürn İkizler'de (5.): İkizler'den 5. burç Terazi. 7. ev Balık, Jüpiter İkizler'de (4.): İkizler'den 4. burç Başak, evden 7.; istisna: Başak'tan 10. burç İkizler. 8. ev Koç, Mars Aslan'da (5.): Aslan'dan 5. burç Yay.",
+    "Pada (9–12). 9. ev Boğa, Venüs İkizler'de (2.): İkizler'den 2. burç Yengeç. 10. ev İkizler, Merkür Boğa'da (12.): Boğa'dan 12. burç Koç. 11. ev Yengeç, Ay Kova'da (8.): Kova'dan 8. burç Başak. 12. ev Aslan, Güneş Boğa'da (10.): Boğa'dan 10. burç Kova, evden 7.; istisna: Kova'dan 10. burç Akrep."
   ],
   "input": {
     "lagna_lon": 165.0,
@@ -29,6 +34,21 @@
       "atmakaraka": "Venus", "sign": "Taurus",
       "houses_in_d1": {"Sun": 1, "Moon": 10, "Mars": 4, "Mercury": 1, "Jupiter": 2, "Venus": 2, "Saturn": 2, "Rahu": 4, "Ketu": 10},
       "houses_in_d9": {"Sun": 12, "Moon": 6, "Mars": 4, "Mercury": 3, "Jupiter": 7, "Venus": 1, "Saturn": 11, "Rahu": 6, "Ketu": 12}
-    }
+    },
+    "co_lords": {"Scorpio": {"lord": "Mars", "step": 5}, "Aquarius": {"lord": "Saturn", "step": 2}},
+    "arudhas": [
+      {"house": 1, "sign": "Capricorn", "lord": "Mercury", "exception": false},
+      {"house": 2, "sign": "Aquarius", "lord": "Venus", "exception": false},
+      {"house": 3, "sign": "Aquarius", "lord": "Mars", "exception": true},
+      {"house": 4, "sign": "Virgo", "lord": "Jupiter", "exception": true},
+      {"house": 5, "sign": "Scorpio", "lord": "Saturn", "exception": false},
+      {"house": 6, "sign": "Libra", "lord": "Saturn", "exception": false},
+      {"house": 7, "sign": "Gemini", "lord": "Jupiter", "exception": true},
+      {"house": 8, "sign": "Sagittarius", "lord": "Mars", "exception": false},
+      {"house": 9, "sign": "Cancer", "lord": "Venus", "exception": false},
+      {"house": 10, "sign": "Aries", "lord": "Mercury", "exception": false},
+      {"house": 11, "sign": "Virgo", "lord": "Moon", "exception": false},
+      {"house": 12, "sign": "Scorpio", "lord": "Sun", "exception": true}
+    ]
   }
 }
diff --git a/tests/golden/modules/jaimini/co-lord-exaltation-and-debilitation.json b/tests/golden/modules/jaimini/co-lord-exaltation-and-debilitation.json
index 13c1b26..5d3a56a 100644
--- a/tests/golden/modules/jaimini/co-lord-exaltation-and-debilitation.json
+++ b/tests/golden/modules/jaimini/co-lord-exaltation-and-debilitation.json
@@ -7,7 +7,12 @@
     "Karaka (sekizli şema): Rahu'nun etkin derecesi 30 − 9 = 21. Büyükten küçüğe: Ay 25, Güneş 23, Rahu 21, Jüpiter 17, Mars 15, Venüs 12, Merkür 6, Satürn 5. Komşu farkların en küçüğü 1°: eşitlik yok.",
     "Karakamsha: Atmakaraka Ay Balık 25°. Su burcu Yengeç'ten sayılır; 25° sekizinci parçadadır (23°20′–26°40′): Yengeç'ten 8. burç Kova.",
     "D9: Güneş Başak 23° (toprak, Oğlak'tan; 7. parça 20°–23°20′): Yengeç. Ay: Kova. Mars Oğlak 15° (toprak; 5. parça 13°20′–16°40′): Boğa. Merkür Başak 6° (toprak; 2. parça 3°20′–6°40′): Kova. Jüpiter Yengeç 17° (su; 6. parça 16°40′–20°): Yay. Venüs Terazi 12° (hava, Terazi'den; 4. parça 10°–13°20′): Oğlak. Satürn Koç 5° (ateş, Koç'tan; 2. parça): Boğa. Rahu Yay 9° (ateş; 3. parça 6°40′–10°): İkizler. Ketu İkizler 9° (hava; 3. parça): Yay.",
-    "D1 evleri, Kova'dan (Güneş…Ketu): Başak 8, Balık 2, Oğlak 12, Başak 8, Yengeç 6, Terazi 9, Koç 3, Yay 11, İkizler 5. D9 evleri, Kova'dan: Yengeç 6, Kova 1, Boğa 4, Kova 1, Yay 11, Oğlak 12, Boğa 4, İkizler 5, Yay 11."
+    "D1 evleri, Kova'dan (Güneş…Ketu): Başak 8, Balık 2, Oğlak 12, Başak 8, Yengeç 6, Terazi 9, Koç 3, Yay 11, İkizler 5. D9 evleri, Kova'dan: Yengeç 6, Kova 1, Boğa 4, Kova 1, Yay 11, Oğlak 12, Boğa 4, İkizler 5, Yay 11.",
+    "Eş lord, Akrep (Mars Oğlak'ta, Ketu İkizler'de): 1. ölçüt ayırmaz. 2. ölçüt: ikisi de yalnız. 3. ölçüt: Mars yücelme burcunda (Oğlak): Mars.",
+    "Eş lord, Kova (Satürn Koç'ta, Rahu Yay'da): 1. ölçüt ayırmaz. 2. ölçüt: ikisi de yalnız. 3. ölçüt: Satürn yücelme burcunda değil (Terazi), düşüş burcunda (Koç): düşüşte olmayan seçilir: Rahu.",
+    "Pada, Aslan lagna. 1. ev Aslan, Güneş Başak'ta (2.): Başak'tan 2. burç Terazi. 2. ev Başak, Merkür Başak'ta (1.): istisna: Başak'tan 10. burç İkizler. 3. ev Terazi, Venüs Terazi'de (1.): istisna: Terazi'den 10. burç Yengeç. 4. ev Akrep, lord Mars Oğlak'ta (3.): Oğlak'tan 3. burç Balık.",
+    "Pada (5–8). 5. ev Yay, Jüpiter Yengeç'te (8.): Yengeç'ten 8. burç Kova. 6. ev Oğlak, Satürn Koç'ta (4.): Koç'tan 4. burç Yengeç, evden 7.; istisna: Yengeç'ten 10. burç Koç. 7. ev Kova, lord Rahu Yay'da (11.): Yay'dan 11. burç Terazi. 8. ev Balık, Jüpiter Yengeç'te (5.): Yengeç'ten 5. burç Akrep.",
+    "Pada (9–12). 9. ev Koç, Mars Oğlak'ta (10.): Oğlak'tan 10. burç Terazi, evden 7.; istisna: Terazi'den 10. burç Yengeç. 10. ev Boğa, Venüs Terazi'de (6.): Terazi'den 6. burç Balık. 11. ev İkizler, Merkür Başak'ta (4.): Başak'tan 4. burç Yay, evden 7.; istisna: Yay'dan 10. burç Başak. 12. ev Yengeç, Ay Balık'ta (9.): Balık'tan 9. burç Akrep."
   ],
   "input": {
     "lagna_lon": 130.0,
@@ -29,6 +34,21 @@
       "atmakaraka": "Moon", "sign": "Aquarius",
       "houses_in_d1": {"Sun": 8, "Moon": 2, "Mars": 12, "Mercury": 8, "Jupiter": 6, "Venus": 9, "Saturn": 3, "Rahu": 11, "Ketu": 5},
       "houses_in_d9": {"Sun": 6, "Moon": 1, "Mars": 4, "Mercury": 1, "Jupiter": 11, "Venus": 12, "Saturn": 4, "Rahu": 5, "Ketu": 11}
-    }
+    },
+    "co_lords": {"Scorpio": {"lord": "Mars", "step": 3}, "Aquarius": {"lord": "Rahu", "step": 3}},
+    "arudhas": [
+      {"house": 1, "sign": "Libra", "lord": "Sun", "exception": false},
+      {"house": 2, "sign": "Gemini", "lord": "Mercury", "exception": true},
+      {"house": 3, "sign": "Cancer", "lord": "Venus", "exception": true},
+      {"house": 4, "sign": "Pisces", "lord": "Mars", "exception": false},
+      {"house": 5, "sign": "Aquarius", "lord": "Jupiter", "exception": false},
+      {"house": 6, "sign": "Aries", "lord": "Saturn", "exception": true},
+      {"house": 7, "sign": "Libra", "lord": "Rahu", "exception": false},
+      {"house": 8, "sign": "Scorpio", "lord": "Jupiter", "exception": false},
+      {"house": 9, "sign": "Cancer", "lord": "Mars", "exception": true},
+      {"house": 10, "sign": "Pisces", "lord": "Venus", "exception": false},
+      {"house": 11, "sign": "Virgo", "lord": "Mercury", "exception": true},
+      {"house": 12, "sign": "Scorpio", "lord": "Moon", "exception": false}
+    ]
   }
 }
diff --git a/tests/golden/modules/jaimini/co-lords-in-the-same-sign.json b/tests/golden/modules/jaimini/co-lords-in-the-same-sign.json
index 315f7bd..5626e67 100644
--- a/tests/golden/modules/jaimini/co-lords-in-the-same-sign.json
+++ b/tests/golden/modules/jaimini/co-lords-in-the-same-sign.json
@@ -7,7 +7,12 @@
     "Karaka (sekizli şema): Rahu'nun etkin derecesi 30 − 4 = 26. Büyükten küçüğe: Güneş 28, Rahu 26, Jüpiter 23, Mars 21, Venüs 19, Satürn 16, Merkür 13, Ay 8. Eşitlik yok.",
     "Karakamsha: Atmakaraka Güneş Akrep 28°. Su burcu Yengeç'ten sayılır; 28° dokuzuncu parçadadır: Yengeç'ten 9. burç Balık.",
     "D9: Güneş: Balık. Ay Yengeç 8° (su; 3. parça 6°40′–10°): Başak. Mars Akrep 21° (su; 7. parça 20°–23°20′): Oğlak. Merkür Yay 13° (ateş, Koç'tan; 4. parça 10°–13°20′): Yengeç. Jüpiter Kova 23° (hava, Terazi'den; 7. parça): Koç. Venüs Terazi 19° (hava; 6. parça 16°40′–20°): Balık. Satürn Boğa 16° (toprak, Oğlak'tan; 5. parça 13°20′–16°40′): Boğa. Rahu Boğa 4° (toprak; 2. parça 3°20′–6°40′): Kova. Ketu Akrep 4° (su; 2. parça): Aslan.",
-    "D1 evleri, Balık'tan (Güneş…Ketu): Akrep 9, Yengeç 5, Akrep 9, Yay 10, Kova 12, Terazi 8, Boğa 3, Boğa 3, Akrep 9. D9 evleri, Balık'tan: Balık 1, Başak 7, Oğlak 11, Yengeç 5, Koç 2, Balık 1, Boğa 3, Kova 12, Aslan 6."
+    "D1 evleri, Balık'tan (Güneş…Ketu): Akrep 9, Yengeç 5, Akrep 9, Yay 10, Kova 12, Terazi 8, Boğa 3, Boğa 3, Akrep 9. D9 evleri, Balık'tan: Balık 1, Başak 7, Oğlak 11, Yengeç 5, Koç 2, Balık 1, Boğa 3, Kova 12, Aslan 6.",
+    "Eş lord, Akrep (Mars ve Ketu Akrep'te): 1. ölçüt ayırmaz (ikisi de burcun kendisinde). 2. ölçüt: ikisinin de eşliği Güneş (1–1). 3. ölçüt: Mars Akrep'te ne yücelmiş ne düşmüş. 4. ve 5. ölçüt: aynı burç, eşit. 6. ölçüt: Mars.",
+    "Eş lord, Kova (Satürn ve Rahu Boğa'da): 1. ölçüt ayırmaz (ikisi de Kova'nın dışında). 2. ölçüt: Boğa'da başka graha yok (0–0). 3. ölçüt: Satürn Boğa'da ne yücelmiş ne düşmüş. 4. ve 5. ölçüt: aynı burç, eşit. 6. ölçüt: Satürn.",
+    "Pada, Kova lagna. 1. ev Kova, lord Satürn Boğa'da (4.): Boğa'dan 4. burç Aslan, evden 7.; istisna: Aslan'dan 10. burç Boğa. 2. ev Balık, Jüpiter Kova'da (12.): Kova'dan 12. burç Oğlak. 3. ev Koç, Mars Akrep'te (8.): Akrep'ten 8. burç İkizler. 4. ev Boğa, Venüs Terazi'de (6.): Terazi'den 6. burç Balık.",
+    "Pada (5–8). 5. ev İkizler, Merkür Yay'da (7.): Yay'dan 7. burç İkizler, evin kendisi; istisna: İkizler'den 10. burç Balık. 6. ev Yengeç, Ay Yengeç'te (1.): istisna: Yengeç'ten 10. burç Koç. 7. ev Aslan, Güneş Akrep'te (4.): Akrep'ten 4. burç Kova, evden 7.; istisna: Kova'dan 10. burç Akrep. 8. ev Başak, Merkür Yay'da (4.): Yay'dan 4. burç Balık, evden 7.; istisna: Balık'tan 10. burç Yay.",
+    "Pada (9–12). 9. ev Terazi, Venüs Terazi'de (1.): istisna: Terazi'den 10. burç Yengeç. 10. ev Akrep, lord Mars Akrep'te (1.): istisna: Akrep'ten 10. burç Aslan. 11. ev Yay, Jüpiter Kova'da (3.): Kova'dan 3. burç Koç. 12. ev Oğlak, Satürn Boğa'da (5.): Boğa'dan 5. burç Başak."
   ],
   "input": {
     "lagna_lon": 310.0,
@@ -29,6 +34,21 @@
       "atmakaraka": "Sun", "sign": "Pisces",
       "houses_in_d1": {"Sun": 9, "Moon": 5, "Mars": 9, "Mercury": 10, "Jupiter": 12, "Venus": 8, "Saturn": 3, "Rahu": 3, "Ketu": 9},
       "houses_in_d9": {"Sun": 1, "Moon": 7, "Mars": 11, "Mercury": 5, "Jupiter": 2, "Venus": 1, "Saturn": 3, "Rahu": 12, "Ketu": 6}
-    }
+    },
+    "co_lords": {"Scorpio": {"lord": "Mars", "step": 6}, "Aquarius": {"lord": "Saturn", "step": 6}},
+    "arudhas": [
+      {"house": 1, "sign": "Taurus", "lord": "Saturn", "exception": true},
+      {"house": 2, "sign": "Capricorn", "lord": "Jupiter", "exception": false},
+      {"house": 3, "sign": "Gemini", "lord": "Mars", "exception": false},
+      {"house": 4, "sign": "Pisces", "lord": "Venus", "exception": false},
+      {"house": 5, "sign": "Pisces", "lord": "Mercury", "exception": true},
+      {"house": 6, "sign": "Aries", "lord": "Moon", "exception": true},
+      {"house": 7, "sign": "Scorpio", "lord": "Sun", "exception": true},
+      {"house": 8, "sign": "Sagittarius", "lord": "Mercury", "exception": true},
+      {"house": 9, "sign": "Cancer", "lord": "Venus", "exception": true},
+      {"house": 10, "sign": "Leo", "lord": "Mars", "exception": true},
+      {"house": 11, "sign": "Aries", "lord": "Jupiter", "exception": false},
+      {"house": 12, "sign": "Virgo", "lord": "Saturn", "exception": false}
+    ]
   }
 }
diff --git a/tests/golden/modules/jaimini/node-in-its-sign-and-sign-kind.json b/tests/golden/modules/jaimini/node-in-its-sign-and-sign-kind.json
index 96a24af..cc886ee 100644
--- a/tests/golden/modules/jaimini/node-in-its-sign-and-sign-kind.json
+++ b/tests/golden/modules/jaimini/node-in-its-sign-and-sign-kind.json
@@ -7,7 +7,12 @@
     "Karaka (yedili şema, Rahu sıralanmaz): Satürn 26, Ay 22, Merkür 19, Venüs 15, Mars 11, Jüpiter 7, Güneş 3. Eşitlik yok.",
     "Karakamsha: Atmakaraka Satürn Yengeç 26°. Su burcu Yengeç'ten sayılır; 26° sekizinci parçadadır (23°20′–26°40′): Yengeç'ten 8. burç Kova.",
     "D9: Güneş Koç 3° (ateş, Koç'tan; 1. parça): Koç. Ay Oğlak 22° (toprak, Oğlak'tan; 7. parça 20°–23°20′): Yengeç. Mars Başak 11° (toprak; 4. parça 10°–13°20′): Koç. Merkür Koç 19° (ateş; 6. parça 16°40′–20°): Başak. Jüpiter Aslan 7° (ateş; 3. parça 6°40′–10°): İkizler. Venüs Balık 15° (su; 5. parça 13°20′–16°40′): Akrep. Satürn: Kova. Rahu Boğa 13° (toprak; 4. parça): Koç. Ketu Akrep 13° (su; 4. parça): Terazi.",
-    "D1 evleri, Kova'dan (Güneş…Ketu): Koç 3, Oğlak 12, Başak 8, Koç 3, Aslan 7, Balık 2, Yengeç 6, Boğa 4, Akrep 10. D9 evleri, Kova'dan: Koç 3, Yengeç 6, Koç 3, Başak 8, İkizler 5, Akrep 10, Kova 1, Koç 3, Terazi 9."
+    "D1 evleri, Kova'dan (Güneş…Ketu): Koç 3, Oğlak 12, Başak 8, Koç 3, Aslan 7, Balık 2, Yengeç 6, Boğa 4, Akrep 10. D9 evleri, Kova'dan: Koç 3, Yengeç 6, Koç 3, Başak 8, İkizler 5, Akrep 10, Kova 1, Koç 3, Terazi 9.",
+    "Eş lord, Akrep (Mars Başak'ta, Ketu Akrep'te): 1. ölçüt: yalnızca Ketu burcun kendisinde; burcun dışındaki lord seçilir: Mars.",
+    "Eş lord, Kova (Satürn Yengeç'te, Rahu Boğa'da): 1. ölçüt ayırmaz. 2. ölçüt: ikisi de yalnız. 3. ölçüt: Satürn Yengeç'te ne yücelmiş ne düşmüş. 4. ölçüt: Yengeç yürük, Boğa sabit: Rahu.",
+    "Pada, İkizler lagna. 1. ev İkizler, Merkür Koç'ta (11.): Koç'tan 11. burç Kova. 2. ev Yengeç, Ay Oğlak'ta (7.): Oğlak'tan 7. burç Yengeç, evin kendisi; istisna: Yengeç'ten 10. burç Koç. 3. ev Aslan, Güneş Koç'ta (9.): Koç'tan 9. burç Yay. 4. ev Başak, Merkür Koç'ta (8.): Koç'tan 8. burç Akrep.",
+    "Pada (5–8). 5. ev Terazi, Venüs Balık'ta (6.): Balık'tan 6. burç Aslan. 6. ev Akrep, lord Mars Başak'ta (11.): Başak'tan 11. burç Yengeç. 7. ev Yay, Jüpiter Aslan'da (9.): Aslan'dan 9. burç Koç. 8. ev Oğlak, Satürn Yengeç'te (7.): Yengeç'ten 7. burç Oğlak, evin kendisi; istisna: Oğlak'tan 10. burç Terazi.",
+    "Pada (9–12). 9. ev Kova, lord Rahu Boğa'da (4.): Boğa'dan 4. burç Aslan, evden 7.; istisna: Aslan'dan 10. burç Boğa. 10. ev Balık, Jüpiter Aslan'da (6.): Aslan'dan 6. burç Oğlak. 11. ev Koç, Mars Başak'ta (6.): Başak'tan 6. burç Kova. 12. ev Boğa, Venüs Balık'ta (11.): Balık'tan 11. burç Oğlak."
   ],
   "input": {
     "lagna_lon": 75.0,
@@ -28,6 +33,21 @@
       "atmakaraka": "Saturn", "sign": "Aquarius",
       "houses_in_d1": {"Sun": 3, "Moon": 12, "Mars": 8, "Mercury": 3, "Jupiter": 7, "Venus": 2, "Saturn": 6, "Rahu": 4, "Ketu": 10},
       "houses_in_d9": {"Sun": 3, "Moon": 6, "Mars": 3, "Mercury": 8, "Jupiter": 5, "Venus": 10, "Saturn": 1, "Rahu": 3, "Ketu": 9}
-    }
+    },
+    "co_lords": {"Scorpio": {"lord": "Mars", "step": 1}, "Aquarius": {"lord": "Rahu", "step": 4}},
+    "arudhas": [
+      {"house": 1, "sign": "Aquarius", "lord": "Mercury", "exception": false},
+      {"house": 2, "sign": "Aries", "lord": "Moon", "exception": true},
+      {"house": 3, "sign": "Sagittarius", "lord": "Sun", "exception": false},
+      {"house": 4, "sign": "Scorpio", "lord": "Mercury", "exception": false},
+      {"house": 5, "sign": "Leo", "lord": "Venus", "exception": false},
+      {"house": 6, "sign": "Cancer", "lord": "Mars", "exception": false},
+      {"house": 7, "sign": "Aries", "lord": "Jupiter", "exception": false},
+      {"house": 8, "sign": "Libra", "lord": "Saturn", "exception": true},
+      {"house": 9, "sign": "Taurus", "lord": "Rahu", "exception": true},
+      {"house": 10, "sign": "Capricorn", "lord": "Jupiter", "exception": false},
+      {"house": 11, "sign": "Aquarius", "lord": "Mars", "exception": false},
+      {"house": 12, "sign": "Capricorn", "lord": "Venus", "exception": false}
+    ]
   }
 }
diff --git a/tests/golden/modules/jaimini/rahu-atmakaraka-arudha-exceptions.json b/tests/golden/modules/jaimini/rahu-atmakaraka-arudha-exceptions.json
index 5a48d22..a9db36a 100644
--- a/tests/golden/modules/jaimini/rahu-atmakaraka-arudha-exceptions.json
+++ b/tests/golden/modules/jaimini/rahu-atmakaraka-arudha-exceptions.json
@@ -7,7 +7,12 @@
     "Karaka (sekizli şema): Rahu'nun etkin derecesi 30 − 1 = 29. Büyükten küçüğe: Rahu 29, Venüs 26, Ay 21, Jüpiter 17, Satürn 11, Güneş 8, Mars 4, Merkür 2. Komşu farkların en küçüğü 2°: eşitlik yok.",
     "Karakamsha: Atmakaraka Rahu İkizler 1°. Hava burcu Terazi'den sayılır; 1° ilk parçadadır (0°–3°20′): Terazi.",
     "D9: Güneş Akrep 8° (su, Yengeç'ten; 3. parça 6°40′–10°): Başak. Ay Koç 21° (ateş, Koç'tan; 7. parça 20°–23°20′): Terazi. Mars Koç 4° (2. parça): Boğa. Merkür Yay 2° (ateş; 1. parça): Koç. Jüpiter Boğa 17° (toprak, Oğlak'tan; 6. parça 16°40′–20°): İkizler. Venüs Terazi 26° (hava; 8. parça 23°20′–26°40′): Boğa. Satürn Oğlak 11° (toprak; 4. parça 10°–13°20′): Koç. Rahu: Terazi. Ketu Yay 1° (1. parça): Koç.",
-    "D1 evleri, Terazi'den (Güneş…Ketu): Akrep 2, Koç 7, Koç 7, Yay 3, Boğa 8, Terazi 1, Oğlak 4, İkizler 9, Yay 3. D9 evleri, Terazi'den: Başak 12, Terazi 1, Boğa 8, Koç 7, İkizler 9, Boğa 8, Koç 7, Terazi 1, Koç 7."
+    "D1 evleri, Terazi'den (Güneş…Ketu): Akrep 2, Koç 7, Koç 7, Yay 3, Boğa 8, Terazi 1, Oğlak 4, İkizler 9, Yay 3. D9 evleri, Terazi'den: Başak 12, Terazi 1, Boğa 8, Koç 7, İkizler 9, Boğa 8, Koç 7, Terazi 1, Koç 7.",
+    "Eş lord, Akrep (Mars Koç'ta, Ketu Yay'da): 1. ölçüt ayırmaz (ikisi de Akrep'in dışında). 2. ölçüt: Mars Ay'la (1), Ketu Merkür'le (1): eşit. 3. ölçüt: Mars Koç'ta ne yücelmiş ne düşmüş. 4. ölçüt: Koç yürük, Yay çift doğalı: Ketu.",
+    "Eş lord, Kova (Satürn Oğlak'ta, Rahu İkizler'de): 1. ölçüt ayırmaz. 2. ölçüt: ikisi de yalnız (0–0). 3. ölçüt: Satürn Oğlak'ta ne yücelmiş ne düşmüş. 4. ölçüt: Oğlak yürük, İkizler çift doğalı: Rahu.",
+    "Pada, Koç lagna. 1. ev Koç, lord Mars Koç'ta (1.): hesaplanan burç Koç, evin kendisi; istisna: Koç'tan 10. burç Oğlak. 2. ev Boğa, Venüs Terazi'de (6.): Terazi'den 6. burç Balık. 3. ev İkizler, Merkür Yay'da (7.): Yay'dan 7. burç İkizler, evin kendisi; istisna: İkizler'den 10. burç Balık. 4. ev Yengeç, Ay Koç'ta (10.): Koç'tan 10. burç Oğlak, evden 7.; istisna: Oğlak'tan 10. burç Terazi.",
+    "Pada (5–8). 5. ev Aslan, Güneş Akrep'te (4.): Akrep'ten 4. burç Kova, evden 7.; istisna: Kova'dan 10. burç Akrep. 6. ev Başak, Merkür Yay'da (4.): Yay'dan 4. burç Balık, evden 7.; istisna: Balık'tan 10. burç Yay. 7. ev Terazi, Venüs Terazi'de (1.): istisna: Terazi'den 10. burç Yengeç. 8. ev Akrep, lord Ketu Yay'da (2.): Yay'dan 2. burç Oğlak.",
+    "Pada (9–12). 9. ev Yay, Jüpiter Boğa'da (6.): Boğa'dan 6. burç Terazi. 10. ev Oğlak, Satürn Oğlak'ta (1.): istisna: Oğlak'tan 10. burç Terazi. 11. ev Kova, lord Rahu İkizler'de (5.): İkizler'den 5. burç Terazi. 12. ev Balık, Jüpiter Boğa'da (3.): Boğa'dan 3. burç Yengeç."
   ],
   "input": {
     "lagna_lon": 12.0,
@@ -29,6 +34,21 @@
       "atmakaraka": "Rahu", "sign": "Libra",
       "houses_in_d1": {"Sun": 2, "Moon": 7, "Mars": 7, "Mercury": 3, "Jupiter": 8, "Venus": 1, "Saturn": 4, "Rahu": 9, "Ketu": 3},
       "houses_in_d9": {"Sun": 12, "Moon": 1, "Mars": 8, "Mercury": 7, "Jupiter": 9, "Venus": 8, "Saturn": 7, "Rahu": 1, "Ketu": 7}
-    }
+    },
+    "co_lords": {"Scorpio": {"lord": "Ketu", "step": 4}, "Aquarius": {"lord": "Rahu", "step": 4}},
+    "arudhas": [
+      {"house": 1, "sign": "Capricorn", "lord": "Mars", "exception": true},
+      {"house": 2, "sign": "Pisces", "lord": "Venus", "exception": false},
+      {"house": 3, "sign": "Pisces", "lord": "Mercury", "exception": true},
+      {"house": 4, "sign": "Libra", "lord": "Moon", "exception": true},
+      {"house": 5, "sign": "Scorpio", "lord": "Sun", "exception": true},
+      {"house": 6, "sign": "Sagittarius", "lord": "Mercury", "exception": true},
+      {"house": 7, "sign": "Cancer", "lord": "Venus", "exception": true},
+      {"house": 8, "sign": "Capricorn", "lord": "Ketu", "exception": false},
+      {"house": 9, "sign": "Libra", "lord": "Jupiter", "exception": false},
+      {"house": 10, "sign": "Libra", "lord": "Saturn", "exception": true},
+      {"house": 11, "sign": "Libra", "lord": "Rahu", "exception": false},
+      {"house": 12, "sign": "Cancer", "lord": "Jupiter", "exception": false}
+    ]
   }
 }
diff --git a/tests/golden/modules/jaimini/tie-within-one-arcsecond-seven-scheme.json b/tests/golden/modules/jaimini/tie-within-one-arcsecond-seven-scheme.json
index 56d570c..0f342c8 100644
--- a/tests/golden/modules/jaimini/tie-within-one-arcsecond-seven-scheme.json
+++ b/tests/golden/modules/jaimini/tie-within-one-arcsecond-seven-scheme.json
@@ -7,7 +7,12 @@
     "Karaka (yedili şema, Rahu sıralanmaz, Pitri rolü yok): Mars 27, Merkür 23, Ay 18, Jüpiter 17,9998, Venüs 17,9994, Satürn 9, Güneş 5. 1″ = 0,000278°. Ay − Jüpiter = 0,0002° (0,72″): eşit sayılır, ikisi de işaretlenir; sıra tam değerle Ay önce. Jüpiter − Venüs = 0,0004° (1,44″): eşit değil; Venüs'ün öbür komşusu Satürn 9° uzakta: Venüs işaretlenmez.",
     "Karakamsha: Atmakaraka Mars Aslan 27°. Ateş burcu Koç'tan sayılır; 27° dokuzuncu parçadadır (26°40′–30°): Koç'tan 9. burç Yay.",
     "D9: Güneş İkizler 5° (hava, Terazi'den; 2. parça 3°20′–6°40′): Akrep. Ay Boğa 18° (toprak, Oğlak'tan; 6. parça 16°40′–20°): İkizler. Mars: Yay. Merkür İkizler 23° (hava; 7. parça 20°–23°20′): Koç. Jüpiter Balık 17,9998° (su, Yengeç'ten; 6. parça): Yay. Venüs Yengeç 17,9994° (su; 6. parça): Yay. Satürn Yay 9° (ateş; 3. parça 6°40′–10°): İkizler. Rahu Başak 12° (toprak; 4. parça 10°–13°20′): Koç. Ketu Balık 12° (su; 4. parça): Terazi.",
-    "D1 evleri, Yay'dan (Güneş…Ketu): İkizler 7, Boğa 6, Aslan 9, İkizler 7, Balık 4, Yengeç 8, Yay 1, Başak 10, Balık 4. D9 evleri, Yay'dan: Akrep 12, İkizler 7, Yay 1, Koç 5, Yay 1, Yay 1, İkizler 7, Koç 5, Terazi 11."
+    "D1 evleri, Yay'dan (Güneş…Ketu): İkizler 7, Boğa 6, Aslan 9, İkizler 7, Balık 4, Yengeç 8, Yay 1, Başak 10, Balık 4. D9 evleri, Yay'dan: Akrep 12, İkizler 7, Yay 1, Koç 5, Yay 1, Yay 1, İkizler 7, Koç 5, Terazi 11.",
+    "Eş lord, Akrep (Mars Aslan'da, Ketu Balık'ta): 1. ölçüt ayırmaz. 2. ölçüt: Mars yalnız (0), Ketu Jüpiter'le (1): Ketu.",
+    "Eş lord, Kova (Satürn Yay'da, Rahu Başak'ta): 1. ölçüt ayırmaz. 2. ölçüt: ikisi de yalnız. 3. ölçüt: Satürn Yay'da ne yücelmiş ne düşmüş. 4. ölçüt: Yay ve Başak çift doğalı: eşit. 5. ölçüt: Kova geri sayar: Kova 1, Oğlak 2, Yay 3 (Satürn); Akrep 4, Terazi 5, Başak 6 (Rahu): Rahu daha uzak.",
+    "Pada, Yengeç lagna. 1. ev Yengeç, Ay Boğa'da (11.): Boğa'dan 11. burç Balık. 2. ev Aslan, Güneş İkizler'de (11.): İkizler'den 11. burç Koç. 3. ev Başak, Merkür İkizler'de (10.): İkizler'den 10. burç Balık, evden 7.; istisna: Balık'tan 10. burç Yay. 4. ev Terazi, Venüs Yengeç'te (10.): Yengeç'ten 10. burç Koç, evden 7.; istisna: Koç'tan 10. burç Oğlak.",
+    "Pada (5–8). 5. ev Akrep, lord Ketu Balık'ta (5.): Balık'tan 5. burç Yengeç. 6. ev Yay, Jüpiter Balık'ta (4.): Balık'tan 4. burç İkizler, evden 7.; istisna: İkizler'den 10. burç Balık. 7. ev Oğlak, Satürn Yay'da (12.): Yay'dan 12. burç Akrep. 8. ev Kova, lord Rahu Başak'ta (8.): Başak'tan 8. burç Koç.",
+    "Pada (9–12). 9. ev Balık, Jüpiter Balık'ta (1.): istisna: Balık'tan 10. burç Yay. 10. ev Koç, Mars Aslan'da (5.): Aslan'dan 5. burç Yay. 11. ev Boğa, Venüs Yengeç'te (3.): Yengeç'ten 3. burç Başak. 12. ev İkizler, Merkür İkizler'de (1.): istisna: İkizler'den 10. burç Balık."
   ],
   "input": {
     "lagna_lon": 100.0,
@@ -28,6 +33,21 @@
       "atmakaraka": "Mars", "sign": "Sagittarius",
       "houses_in_d1": {"Sun": 7, "Moon": 6, "Mars": 9, "Mercury": 7, "Jupiter": 4, "Venus": 8, "Saturn": 1, "Rahu": 10, "Ketu": 4},
       "houses_in_d9": {"Sun": 12, "Moon": 7, "Mars": 1, "Mercury": 5, "Jupiter": 1, "Venus": 1, "Saturn": 7, "Rahu": 5, "Ketu": 11}
-    }
+    },
+    "co_lords": {"Scorpio": {"lord": "Ketu", "step": 2}, "Aquarius": {"lord": "Rahu", "step": 5}},
+    "arudhas": [
+      {"house": 1, "sign": "Pisces", "lord": "Moon", "exception": false},
+      {"house": 2, "sign": "Aries", "lord": "Sun", "exception": false},
+      {"house": 3, "sign": "Sagittarius", "lord": "Mercury", "exception": true},
+      {"house": 4, "sign": "Capricorn", "lord": "Venus", "exception": true},
+      {"house": 5, "sign": "Cancer", "lord": "Ketu", "exception": false},
+      {"house": 6, "sign": "Pisces", "lord": "Jupiter", "exception": true},
+      {"house": 7, "sign": "Scorpio", "lord": "Saturn", "exception": false},
+      {"house": 8, "sign": "Aries", "lord": "Rahu", "exception": false},
+      {"house": 9, "sign": "Sagittarius", "lord": "Jupiter", "exception": true},
+      {"house": 10, "sign": "Sagittarius", "lord": "Mars", "exception": false},
+      {"house": 11, "sign": "Virgo", "lord": "Venus", "exception": false},
+      {"house": 12, "sign": "Pisces", "lord": "Mercury", "exception": true}
+    ]
   }
 }
diff --git a/tests/golden/modules/jaimini/time-unknown-third-house-malefics.json b/tests/golden/modules/jaimini/time-unknown-third-house-malefics.json
index 684f69b..3aeeba0 100644
--- a/tests/golden/modules/jaimini/time-unknown-third-house-malefics.json
+++ b/tests/golden/modules/jaimini/time-unknown-third-house-malefics.json
@@ -7,7 +7,10 @@
     "Karaka (sekizli şema): Rahu'nun etkin derecesi 30 − 11 = 19. Büyükten küçüğe: Merkür 29, Mars 24, Rahu 19, Satürn 18, Güneş 14, Venüs 9, Jüpiter 7, Ay 2. Eşitlik yok.",
     "Karakamsha: Atmakaraka Merkür Boğa 29°. Toprak burcu Oğlak'tan sayılır; 29° dokuzuncu parçadadır: Oğlak'tan 9. burç Başak.",
     "D9: Güneş İkizler 14° (hava, Terazi'den; 5. parça 13°20′–16°40′): Kova. Ay İkizler 2° (1. parça): Terazi. Mars İkizler 24° (8. parça 23°20′–26°40′): Boğa. Merkür: Başak. Jüpiter Terazi 7° (hava; 3. parça 6°40′–10°): Yay. Venüs Yengeç 9° (su, Yengeç'ten; 3. parça): Başak. Satürn Kova 18° (hava; 6. parça 16°40′–20°): Balık. Rahu Akrep 11° (su; 4. parça 10°–13°20′): Terazi. Ketu Boğa 11° (toprak; 4. parça): Koç.",
-    "D1 evleri, Başak'tan (Güneş…Ketu): İkizler 10, İkizler 10, İkizler 10, Boğa 9, Terazi 2, Yengeç 11, Kova 6, Akrep 3, Boğa 9. D9 evleri, Başak'tan: Kova 6, Terazi 2, Boğa 9, Başak 1, Yay 4, Başak 1, Balık 7, Terazi 2, Koç 8."
+    "D1 evleri, Başak'tan (Güneş…Ketu): İkizler 10, İkizler 10, İkizler 10, Boğa 9, Terazi 2, Yengeç 11, Kova 6, Akrep 3, Boğa 9. D9 evleri, Başak'tan: Kova 6, Terazi 2, Boğa 9, Başak 1, Yay 4, Başak 1, Balık 7, Terazi 2, Koç 8.",
+    "Eş lord, Akrep (Mars İkizler'de, Ketu Boğa'da): 1. ölçüt ayırmaz (Akrep'teki Rahu Akrep'in eş lordu değildir). 2. ölçüt: Mars Güneş ve Ay'la (2), Ketu Merkür'le (1): Mars.",
+    "Eş lord, Kova (Satürn Kova'da, Rahu Akrep'te): 1. ölçüt: yalnızca Satürn burcun kendisinde; burcun dışındaki lord seçilir: Rahu.",
+    "Pada: lagna olmadığı için padalar verilmez."
   ],
   "input": {
     "lagna_lon": null,
@@ -29,6 +32,8 @@
       "atmakaraka": "Mercury", "sign": "Virgo",
       "houses_in_d1": {"Sun": 10, "Moon": 10, "Mars": 10, "Mercury": 9, "Jupiter": 2, "Venus": 11, "Saturn": 6, "Rahu": 3, "Ketu": 9},
       "houses_in_d9": {"Sun": 6, "Moon": 2, "Mars": 9, "Mercury": 1, "Jupiter": 4, "Venus": 1, "Saturn": 7, "Rahu": 2, "Ketu": 8}
-    }
+    },
+    "co_lords": {"Scorpio": {"lord": "Mars", "step": 2}, "Aquarius": {"lord": "Rahu", "step": 1}},
+    "arudhas": null
   }
 }
````

- [ ] **Step 2: Düzeneği ve değişmezleri büyüt (başarısız testler)**

Düzenek eş lord seçimini (lord ve belirleyici ölçüt) iki burç için de karşılaştırır, `jaimini_lord(…, true)`'nun aynı lordu verdiğini ve on iki padanın (burç, lord, istisna) birebir tuttuğunu denetler. Kapsam testi şunları sabitler: eş lordların devre dışı olduğu bir örnek; lagnasız bir örnek; lordun evden 1., 4., 7. ve 10. burçta olduğu birer istisna; altı ölçütün her biri. `house_sign_of` test yardımcısı ev burcunu motorun `house_sign`'ı yerine elle sayar.

Değişmezler: seçilen lord iki eş lorddan biridir; yalnızca biri burçtaysa 1. ölçüt belirler ve dışarıdaki seçilir; ikisi aynı burçtaysa yalnızca 3. ya da 6. ölçüt belirleyebilir; 6. ölçüt hep graha lordu verir. Hiçbir pada evin kendi burcuna ya da 7. burcuna düşmez; pada, lordun burcundan lordun evden uzaklığı kadar ötededir (istisnada bundan 10. burç); istisna tam olarak lord 1., 4., 7. ya da 10. burçtayken uygulanır; lagna yoksa `None`.

````diff
diff --git a/crates/jyotish-core/src/proptests.rs b/crates/jyotish-core/src/proptests.rs
index 85b4356..dabd332 100644
--- a/crates/jyotish-core/src/proptests.rs
+++ b/crates/jyotish-core/src/proptests.rs
@@ -490,11 +490,13 @@ mod jaimini_properties {
     use super::foundation_properties::positions;
     use crate::graha::Graha;
     use crate::jaimini::{
-        chara_karakas, graha_rashi_drishti, karaka_roles, karakamsha, navamsa_sign,
-        rashi_drishti_signs, KARAKA_TIE_DEG,
+        arudha_padas, chara_karakas, co_lord_choice, co_lords, graha_rashi_drishti, karaka_roles,
+        karakamsha, navamsa_sign, rashi_drishti_signs, KARAKA_TIE_DEG,
     };
+    use crate::lordship::house_sign;
     use crate::settings::KarakaScheme;
     use crate::varga::is_vargottama;
+    use crate::zodiac::Sign;
 
     fn scheme() -> impl Strategy<Value = KarakaScheme> {
         prop_oneof![Just(KarakaScheme::Eight), Just(KarakaScheme::Seven)]
@@ -563,5 +565,58 @@ mod jaimini_properties {
                 }
             }
         }
+
+        #[test]
+        fn co_lord_choice_is_one_of_the_two_lords(p in positions()) {
+            for sign in [Sign::Scorpio, Sign::Aquarius] {
+                let (planet, node) = co_lords(sign).expect("co-lorded");
+                let c = co_lord_choice(sign, &p).expect("choice");
+                prop_assert!(c.lord == planet || c.lord == node);
+                prop_assert!((1..=6).contains(&c.step));
+                let (ps, ns) = (p.sign_of(planet), p.sign_of(node));
+                // 1. ölçüt: yalnızca biri burçtaysa dışarıdaki seçilir.
+                if (ps == sign) != (ns == sign) {
+                    prop_assert_eq!(c.step, 1);
+                    prop_assert_eq!(c.lord, if ps == sign { node } else { planet });
+                } else {
+                    prop_assert!(c.step > 1);
+                }
+                // İkisi aynı burçtaysa hiçbir ölçüt ayırmaz; yücelme ve düşüş dışında.
+                if ps == ns {
+                    prop_assert!(c.step == 3 || c.step == 6);
+                }
+                if c.step == 6 {
+                    prop_assert_eq!(c.lord, planet);
+                }
+            }
+        }
+
+        #[test]
+        fn no_pada_falls_in_its_house_or_the_seventh(p in positions(), co in any::<bool>()) {
+            match (p.lagna_sign, arudha_padas(&p, co)) {
+                (None, padas) => prop_assert_eq!(padas, None),
+                (Some(lagna), Some(padas)) => {
+                    for (i, pada) in padas.iter().enumerate() {
+                        let house = i as u8 + 1;
+                        prop_assert_eq!(pada.house, house);
+                        let from_house = pada.sign.house_from(house_sign(lagna, house));
+                        prop_assert!(from_house != 1 && from_house != 7);
+                        // Pada, evin lordunun burcundan lordun evden uzaklığı kadar ötededir;
+                        // istisnada bundan 10. burçtur.
+                        let lord_sign = p.sign_of(pada.lord);
+                        let n = lord_sign.house_from(house_sign(lagna, house));
+                        let computed = Sign::from_index(lord_sign.index() + usize::from(n) - 1);
+                        let want = if pada.exception {
+                            Sign::from_index(computed.index() + 9)
+                        } else {
+                            computed
+                        };
+                        prop_assert_eq!(pada.sign, want);
+                        prop_assert_eq!(pada.exception, matches!(n, 1 | 4 | 7 | 10));
+                    }
+                }
+                (Some(_), None) => prop_assert!(false, "padas with a lagna"),
+            }
+        }
     }
 }
diff --git a/crates/jyotish-core/tests/modules_golden.rs b/crates/jyotish-core/tests/modules_golden.rs
index 94d6af2..d7e7fad 100644
--- a/crates/jyotish-core/tests/modules_golden.rs
+++ b/crates/jyotish-core/tests/modules_golden.rs
@@ -7,9 +7,9 @@ use std::collections::BTreeMap;
 use std::path::PathBuf;
 
 use jyotish_core::{
-    chara_karakas, drishti_table, house_lord, houses_owned, karakamsha, kuja_dosha, moon_is_waxing,
-    nature, normalize, AdvancedSettings, Graha, KarakaRole, KujaDosha, Nature, Positions, Sign,
-    ASPECTING_GRAHAS, DRISHTI_METHOD,
+    arudha_padas, chara_karakas, co_lord_choice, drishti_table, house_lord, houses_owned,
+    jaimini_lord, karakamsha, kuja_dosha, moon_is_waxing, nature, normalize, AdvancedSettings,
+    Graha, KarakaRole, KujaDosha, Nature, Positions, Sign, ASPECTING_GRAHAS, DRISHTI_METHOD,
 };
 use serde::de::DeserializeOwned;
 use serde::Deserialize;
@@ -249,12 +249,33 @@ struct ExpectedKarakamsha {
     houses_in_d9: BTreeMap<Graha, u8>,
 }
 
+/// Eş lord seçimi: seçilen lord ve belirleyici ölçütün numarası.
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ExpectedCoLord {
+    lord: Graha,
+    step: u8,
+}
+
+#[derive(Debug, Deserialize, PartialEq)]
+#[serde(deny_unknown_fields)]
+struct ExpectedPada {
+    house: u8,
+    sign: Sign,
+    lord: Graha,
+    exception: bool,
+}
+
 #[derive(Debug, Deserialize)]
 #[serde(deny_unknown_fields)]
 struct JaiminiExpected {
     /// Rol sırasında: sekizli şemada 8, yedili şemada 7 kayıt.
     karakas: Vec<ExpectedKaraka>,
     karakamsha: ExpectedKarakamsha,
+    /// Akrep ve Kova için seçim kuralının sonucu (ayar devre dışıyken de izlenir).
+    co_lords: BTreeMap<Sign, ExpectedCoLord>,
+    /// A1…A12; doğum saati bilinmiyorsa `null`.
+    arudhas: Option<Vec<ExpectedPada>>,
 }
 
 #[test]
@@ -309,6 +330,39 @@ fn jaimini_hand_fixtures() {
                 "{id}: D9 house of {g:?}"
             );
         }
+
+        // Eş lordlar.
+        assert_eq!(
+            x.co_lords.keys().copied().collect::<Vec<_>>(),
+            [Sign::Scorpio, Sign::Aquarius],
+            "{id}: co-lorded signs"
+        );
+        for (&sign, want) in &x.co_lords {
+            let got = co_lord_choice(sign, &p).expect("co-lorded sign");
+            assert_eq!(
+                (got.lord, got.step),
+                (want.lord, want.step),
+                "{id}: co-lord of {sign:?}"
+            );
+            assert_eq!(jaimini_lord(sign, &p, true), want.lord, "{id}: {sign:?}");
+        }
+
+        // Arudha padalar.
+        let padas = arudha_padas(&p, advanced.jaimini_co_lords).map(|a| {
+            a.iter()
+                .map(|x| ExpectedPada {
+                    house: x.house,
+                    sign: x.sign,
+                    lord: x.lord,
+                    exception: x.exception,
+                })
+                .collect::<Vec<_>>()
+        });
+        assert_eq!(padas, x.arudhas, "{id}: arudha padas");
+        assert_eq!(x.arudhas.is_some(), p.lagna_sign.is_some(), "{id}");
+        if let Some(a) = &x.arudhas {
+            assert_eq!(a.len(), 12, "{id}: twelve padas");
+        }
     }
 }
 
@@ -336,4 +390,39 @@ fn jaimini_fixtures_cover_the_required_cases() {
         any(&|f, p| p.sign_of(f.expected.karakamsha.atmakaraka) == f.expected.karakamsha.sign),
         "a vargottama Atmakaraka"
     );
+    assert!(
+        any(&|f, _| f.input.advanced.is_some_and(|a| !a.jaimini_co_lords)),
+        "co-lords switched off"
+    );
+    assert!(
+        any(&|f, _| f.expected.arudhas.is_none()),
+        "birth time unknown"
+    );
+    // Lordun evden 1., 4., 7. ve 10. burçta olduğu istisnalar.
+    for n in [1u8, 4, 7, 10] {
+        assert!(
+            any(&|f, p| {
+                let (Some(lagna), Some(padas)) = (p.lagna_sign, &f.expected.arudhas) else {
+                    return false;
+                };
+                padas.iter().any(|x| {
+                    x.exception && p.sign_of(x.lord).house_from(house_sign_of(lagna, x.house)) == n
+                })
+            }),
+            "arudha exception with the lord in house {n}"
+        );
+    }
+    // Eş lord seçiminin altı ölçütünden her biri en az bir örnekte belirleyicidir.
+    for step in 1..=6u8 {
+        assert!(
+            any(&|f, _| f.expected.co_lords.values().any(|c| c.step == step)),
+            "co-lord step {step}"
+        );
+    }
+}
+
+/// `lagna` burcundan sayılan `house`. evin burcu (test yardımcısı; motorun `house_sign`'ı
+/// yerine elle sayım).
+fn house_sign_of(lagna: Sign, house: u8) -> Sign {
+    Sign::ALL[(lagna as usize + usize::from(house) - 1) % 12]
 }
````

- [ ] **Step 3: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-core 2>&1 | grep -E "^error" | sort | uniq -c
```
Expected: derleme hatası: ``unresolved imports `crate::jaimini::arudha_padas`, `crate::jaimini::co_lord_choice`, `crate::jaimini::co_lords` `` ve ``unresolved imports `jyotish_core::arudha_padas`, `jyotish_core::co_lord_choice`, `jyotish_core::jaimini_lord` ``.

- [ ] **Step 4: `jaimini/lords.rs`**

Altı ölçüt sırayla uygulanır; ilk ayrım belirleyicidir. 3. ölçüt yalnızca graha lordun yücelme ve düşüş burcuna bakar (D6). Birim testleri her ölçütü ortak bir taban konum setinden tek tek kurar; test adındaki numara belirleyici ölçüttür.

**Yeni dosya:** `crates/jyotish-core/src/jaimini/lords.rs`

````rust
//! Jaimini eş lordları (ileri modüller spec'i §5): Akrep'in lordları Mars ile Ketu, Kova'nın
//! lordları Satürn ile Rahu'dur ve güçlü olan lord sayılır. Arudha pada ve chara dasha (Alt
//! proje 3.3) aynı seçimi kullanır.
//!
//! Güçlü lord sırayla altı ölçütle seçilir; ilk ayrım belirleyicidir:
//! 1. yalnızca biri burcun kendisindeyse, burcun dışındaki;
//! 2. bulunduğu burçta daha çok grahayla birlikte olan (diğer eş lord sayılmaz);
//! 3. yücelme burcunda olan; değilse düşüş burcunda olmayan (düğümlerin yücelme ve düşüş burcu
//!    yoktur: bu ölçüt yalnızca Mars'ın ya da Satürn'ün durumuna bakar);
//! 4. bulunduğu burcun türü: çift doğalı > sabit > yürük;
//! 5. söz konusu burçtan, o burcun sayım yönünde daha uzakta olan;
//! 6. Mars (Akrep) ya da Satürn (Kova).

use crate::dignity::{debilitation_point, exaltation_point, sign_lord};
use crate::graha::Graha;
use crate::positions::Positions;
use crate::zodiac::Sign;

use super::rashi::{directed_count, sign_kind, SignKind};

/// Burcun eş lordları: (graha lord, düğüm lord). Akrep ve Kova dışında `None`.
pub fn co_lords(sign: Sign) -> Option<(Graha, Graha)> {
    match sign {
        Sign::Scorpio => Some((Graha::Mars, Graha::Ketu)),
        Sign::Aquarius => Some((Graha::Saturn, Graha::Rahu)),
        _ => None,
    }
}

/// Eş lord seçimi: seçilen lord ve belirleyici ölçütün numarası (1..=6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoLordChoice {
    pub lord: Graha,
    pub step: u8,
}

/// Burç türünün 4. ölçütteki sırası.
fn kind_rank(kind: SignKind) -> u8 {
    match kind {
        SignKind::Dual => 3,
        SignKind::Fixed => 2,
        SignKind::Movable => 1,
    }
}

/// `sign` burcunun eş lordlarından güçlü olanı ve hangi ölçütün belirlediği. Akrep ve Kova
/// dışında `None`.
pub fn co_lord_choice(sign: Sign, positions: &Positions) -> Option<CoLordChoice> {
    let (planet, node) = co_lords(sign)?;
    let planet_sign = positions.sign_of(planet);
    let node_sign = positions.sign_of(node);
    let pick = |lord, step| Some(CoLordChoice { lord, step });
    // Büyük olan tarafın lordu; eşitse `None`.
    let stronger = |p: u8, n: u8| match p.cmp(&n) {
        std::cmp::Ordering::Greater => Some(planet),
        std::cmp::Ordering::Less => Some(node),
        std::cmp::Ordering::Equal => None,
    };

    // 1. Yalnızca biri burcun kendisindeyse, burcun dışındaki.
    match (planet_sign == sign, node_sign == sign) {
        (true, false) => return pick(node, 1),
        (false, true) => return pick(planet, 1),
        _ => {}
    }
    // 2. Daha çok grahayla birlikte olan; iki eş lord sayılmaz.
    let company = |s: Sign| {
        positions
            .occupants(s)
            .into_iter()
            .filter(|&g| g != planet && g != node)
            .count() as u8
    };
    if let Some(lord) = stronger(company(planet_sign), company(node_sign)) {
        return pick(lord, 2);
    }
    // 3. Yücelme ve düşüş: düğümlerin yücelme ve düşüş burcu yoktur.
    if exaltation_point(planet).is_some_and(|(s, _)| s == planet_sign) {
        return pick(planet, 3);
    }
    if debilitation_point(planet).is_some_and(|(s, _)| s == planet_sign) {
        return pick(node, 3);
    }
    // 4. Bulunduğu burcun türü.
    if let Some(lord) = stronger(
        kind_rank(sign_kind(planet_sign)),
        kind_rank(sign_kind(node_sign)),
    ) {
        return pick(lord, 4);
    }
    // 5. Burçtan, sayım yönünde daha uzakta olan.
    if let Some(lord) = stronger(
        directed_count(sign, planet_sign),
        directed_count(sign, node_sign),
    ) {
        return pick(lord, 5);
    }
    // 6. Mars ya da Satürn.
    pick(planet, 6)
}

/// Eş lordlu burcun güçlü lordu; diğer burçlarda burcun tek lordu.
pub fn stronger_co_lord(sign: Sign, positions: &Positions) -> Graha {
    co_lord_choice(sign, positions).map_or_else(|| sign_lord(sign), |c| c.lord)
}

/// Burcun Jaimini lordu: `co_lords` açıksa Akrep ve Kova için güçlü eş lord, değilse (ve diğer
/// burçlarda) `dignity::sign_lord` (Akrep: Mars, Kova: Satürn).
pub fn jaimini_lord(sign: Sign, positions: &Positions, co_lords: bool) -> Graha {
    if co_lords {
        stronger_co_lord(sign, positions)
    } else {
        sign_lord(sign)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;

    /// `Graha::ALL` sırası. Taban: Mars Koç 5°, Ketu Terazi 15°, Satürn Yengeç 5°, Rahu Koç
    /// 15°; diğer grahalar eş lordların burçlarında değil (Güneş İkizler, Ay Başak, Merkür
    /// İkizler, Jüpiter Yay, Venüs Boğa).
    const BASE: [f64; 9] = [65.0, 160.0, 5.0, 70.0, 250.0, 40.0, 95.0, 15.0, 195.0];

    fn with(changes: &[(Graha, f64)]) -> Positions {
        let mut lons = BASE;
        for &(g, lon) in changes {
            lons[g as usize] = lon;
        }
        Positions::synthetic(None, lons)
    }

    fn choice(sign: Sign, p: &Positions) -> (Graha, u8) {
        let c = co_lord_choice(sign, p).expect("co-lorded sign");
        (c.lord, c.step)
    }

    #[test]
    fn only_scorpio_and_aquarius_have_co_lords() {
        let p = with(&[]);
        for s in Sign::ALL {
            match s {
                Sign::Scorpio => assert_eq!(co_lords(s), Some((Mars, Ketu))),
                Sign::Aquarius => assert_eq!(co_lords(s), Some((Saturn, Rahu))),
                _ => {
                    assert_eq!(co_lords(s), None);
                    assert_eq!(co_lord_choice(s, &p), None);
                    assert_eq!(stronger_co_lord(s, &p), sign_lord(s));
                }
            }
            assert_eq!(jaimini_lord(s, &p, false), sign_lord(s));
            assert_eq!(jaimini_lord(s, &p, true), stronger_co_lord(s, &p));
        }
    }

    #[test]
    fn step_1_the_lord_outside_the_sign_wins() {
        // Mars Akrep'te, Ketu Terazi'de: Ketu.
        assert_eq!(choice(Sign::Scorpio, &with(&[(Mars, 215.0)])), (Ketu, 1));
        // Ketu Akrep'te, Mars Koç'ta: Mars.
        assert_eq!(
            choice(Sign::Scorpio, &with(&[(Ketu, 215.0), (Rahu, 35.0)])),
            (Mars, 1)
        );
        // Rahu Kova'da, Satürn Yengeç'te: Satürn.
        assert_eq!(
            choice(Sign::Aquarius, &with(&[(Rahu, 305.0), (Ketu, 125.0)])),
            (Saturn, 1)
        );
    }

    #[test]
    fn step_2_more_company_wins_and_the_other_co_lord_does_not_count() {
        // Taban: Mars Koç'ta Rahu'yla (1), Ketu Terazi'de yalnız (0): Mars.
        assert_eq!(choice(Sign::Scorpio, &with(&[])), (Mars, 2));
        // Ay da Terazi'ye, Jüpiter de Terazi'ye: Ketu 2, Mars 1.
        assert_eq!(
            choice(Sign::Scorpio, &with(&[(Moon, 185.0), (Jupiter, 190.0)])),
            (Ketu, 2)
        );
        // Kova: Satürn Yengeç'te yalnız (0), Rahu Koç'ta Mars'la (1): Rahu.
        assert_eq!(choice(Sign::Aquarius, &with(&[])), (Rahu, 2));
        // İkisi aynı burçta (Koç): Mars ve Ketu birbirini saymaz, eşitlik sürer; Mars Koç'ta
        // ne yücelmiş ne düşmüş, burç türü ve uzaklık aynı: 6. ölçüt.
        assert_eq!(
            choice(Sign::Scorpio, &with(&[(Ketu, 25.0), (Rahu, 205.0)])),
            (Mars, 6)
        );
    }

    #[test]
    fn step_3_exaltation_then_debilitation_of_the_planet_lord() {
        // Mars Oğlak'ta yalnız, Ketu Terazi'de yalnız (eşlik 0–0): Mars yücelmiş.
        assert_eq!(choice(Sign::Scorpio, &with(&[(Mars, 275.0)])), (Mars, 3));
        // Mars Yengeç'te (düşüş); Satürn Aslan'a alınır ki Mars yalnız olsun: Ketu.
        assert_eq!(
            choice(Sign::Scorpio, &with(&[(Mars, 100.0), (Saturn, 125.0)])),
            (Ketu, 3)
        );
        // Kova: Satürn Terazi'de (yücelme) Ketu'yla; Ketu Kova'nın eş lordu değildir, sayılır
        // (1). Rahu Koç'ta Mars'la (1): eşit; Satürn yücelmiş.
        assert_eq!(
            choice(Sign::Aquarius, &with(&[(Saturn, 200.0)])),
            (Saturn, 3)
        );
        // Satürn Koç'ta (düşüş) Mars ve Rahu'yla: Satürn'ün eşliği Mars (Rahu sayılmaz),
        // Rahu'nun eşliği Mars (Satürn sayılmaz): 1–1; Satürn düşmüş: Rahu.
        assert_eq!(choice(Sign::Aquarius, &with(&[(Saturn, 20.0)])), (Rahu, 3));
    }

    #[test]
    fn step_4_sign_kind_dual_over_fixed_over_movable() {
        // Mars İkizler'de (çift doğalı); Güneş ve Merkür Aslan'a alınır ki Mars yalnız olsun.
        // Ketu Terazi'de (yürük) yalnız. Eşlik 0–0; yücelme ve düşüş yok; çift doğalı > yürük.
        let p = with(&[(Mars, 75.0), (Sun, 125.0), (Mercury, 130.0)]);
        assert_eq!(choice(Sign::Scorpio, &p), (Mars, 4));
        // Ketu Boğa'da (sabit) Venüs'le (1); Mars Koç'ta (yürük) Güneş'le (1; Rahu Akrep'te):
        // sabit > yürük: Ketu.
        let p = with(&[(Ketu, 45.0), (Rahu, 225.0), (Sun, 12.0)]);
        assert_eq!(choice(Sign::Scorpio, &p), (Ketu, 4));
    }

    #[test]
    fn step_5_the_lord_farther_in_the_counting_direction() {
        // Akrep ileri sayar: Mars Koç'ta (Akrep'ten 6.), Ketu Terazi'de (Akrep'ten 12.).
        // Eşlik: Mars Rahu'yla (Rahu Akrep'in eş lordu değildir, sayılır: 1), Ketu Ay'la (1).
        // Mars ne yücelmiş ne düşmüş; iki burç da yürük; 6 < 12: Ketu.
        let p = with(&[(Moon, 185.0)]);
        assert_eq!(choice(Sign::Scorpio, &p), (Ketu, 5));
        // Kova geri sayar: Kova 1, Oğlak 2, Yay 3, Akrep 4, Terazi 5, Başak 6, Aslan 7,
        // Yengeç 8 (Satürn), İkizler 9, Boğa 10, Koç 11 (Rahu). Mars Boğa'ya alınır ki Rahu
        // yalnız olsun (eşlik 0–0); iki burç da yürük; 8 < 11: Rahu.
        let p = with(&[(Mars, 35.0)]);
        assert_eq!(choice(Sign::Aquarius, &p), (Rahu, 5));
        // İleri sayılsaydı Satürn (6.) Rahu'dan (3.) uzak olurdu: yön belirleyicidir.
        assert_eq!(Sign::Cancer.house_from(Sign::Aquarius), 6);
        assert_eq!(Sign::Aries.house_from(Sign::Aquarius), 3);
    }

    #[test]
    fn step_6_the_planet_lord_when_nothing_separates_them() {
        // İkisi de burcun kendisinde: 1. ölçüt ayırmaz, sonrakiler eşittir.
        let p = with(&[(Mars, 215.0), (Ketu, 225.0), (Rahu, 45.0)]);
        assert_eq!(choice(Sign::Scorpio, &p), (Mars, 6));
        let p = with(&[(Saturn, 305.0), (Rahu, 320.0), (Ketu, 140.0)]);
        assert_eq!(choice(Sign::Aquarius, &p), (Saturn, 6));
    }
}
````

- [ ] **Step 5: `jaimini/arudha.rs`**

**Yeni dosya:** `crates/jyotish-core/src/jaimini/arudha.rs`

````rust
//! Arudha padalar (ileri modüller spec'i §5).
//!
//! Kaynak: BPHS, pada bölümü. Evin burcu `S`, lordu `T` burcunda olsun; `n`, `S`'den `T`'ye
//! kapsayıcı sayımdır. Pada `T`'den `n`. burçtur. Hesaplanan burç `S`'nin kendisi ya da `S`'den
//! 7. burç ise pada, hesaplanan burçtan 10. burçtur. Evler lagna burcundan sayılan rasi
//! evleridir (K4); Akrep ve Kova'nın lordu `jaimini_lord` ile seçilir. Graha arudhaları yoktur.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::lordship::house_sign;
use crate::positions::Positions;
use crate::zodiac::Sign;

use super::lords::jaimini_lord;

/// Bir evin padası. A1 Arudha Lagna (AL), A12 Upapada (UL) diye de anılır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ArudhaPada {
    /// Ev, 1..=12.
    pub house: u8,
    /// Padanın düştüğü burç.
    pub sign: Sign,
    /// Evin hesapta kullanılan lordu (Akrep ve Kova'da seçilen eş lord).
    pub lord: Graha,
    /// İstisna uygulandı: hesaplanan burç evin kendisi ya da 7. burcuydu.
    pub exception: bool,
}

/// `sign` burcunun, lordu `lord_sign` burcundayken padası ve istisnanın uygulanıp
/// uygulanmadığı.
pub fn arudha_of(sign: Sign, lord_sign: Sign) -> (Sign, bool) {
    let n = usize::from(lord_sign.house_from(sign));
    let computed = Sign::from_index(lord_sign.index() + n - 1);
    if matches!(computed.house_from(sign), 1 | 7) {
        (Sign::from_index(computed.index() + 9), true)
    } else {
        (computed, false)
    }
}

/// On iki evin padası, ev sırasıyla. Doğum saati bilinmiyorsa (lagna yoksa) `None`.
pub fn arudha_padas(positions: &Positions, co_lords: bool) -> Option<[ArudhaPada; 12]> {
    let lagna = positions.lagna_sign?;
    Some(std::array::from_fn(|i| {
        let house = i as u8 + 1;
        let house_sign = house_sign(lagna, house);
        let lord = jaimini_lord(house_sign, positions, co_lords);
        let (sign, exception) = arudha_of(house_sign, positions.sign_of(lord));
        ArudhaPada {
            house,
            sign,
            lord,
            exception,
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;
    use Sign::*;

    #[test]
    fn pada_is_as_far_from_the_lord_as_the_lord_is_from_the_house() {
        // Lord 2. burçta: n = 2; lordun burcundan 2. burç, evden 3. burçtur.
        assert_eq!(arudha_of(Aries, Taurus), (Gemini, false));
        // Lord 5. burçta: Aslan'dan 5. burç Yay.
        assert_eq!(arudha_of(Aries, Leo), (Sagittarius, false));
        // Lord 12. burçta: Balık'tan 12. burç Kova.
        assert_eq!(arudha_of(Aries, Pisces), (Aquarius, false));
        // Lord 9. burçta: Yay'dan 9. burç Aslan.
        assert_eq!(arudha_of(Aries, Sagittarius), (Leo, false));
    }

    #[test]
    fn exceptions_move_the_pada_ten_signs_on() {
        // Lord evin kendi burcunda: hesaplanan burç evin kendisi; padası evden 10. burç.
        assert_eq!(arudha_of(Aries, Aries), (Capricorn, true));
        // Lord 7. burçta: Terazi'den 7. burç yine Koç; pada Oğlak.
        assert_eq!(arudha_of(Aries, Libra), (Capricorn, true));
        // Lord 4. burçta: Yengeç'ten 4. burç Terazi (evden 7.); Terazi'den 10. burç Yengeç.
        assert_eq!(arudha_of(Aries, Cancer), (Cancer, true));
        // Lord 10. burçta: Oğlak'tan 10. burç Terazi (evden 7.); pada Yengeç.
        assert_eq!(arudha_of(Aries, Capricorn), (Cancer, true));
    }

    #[test]
    fn no_pada_falls_in_the_house_or_its_seventh_and_the_exceptions_are_four() {
        for sign in Sign::ALL {
            let mut exceptions = Vec::new();
            for lord_sign in Sign::ALL {
                let (pada, exception) = arudha_of(sign, lord_sign);
                assert!(
                    !matches!(pada.house_from(sign), 1 | 7),
                    "{sign:?} with lord in {lord_sign:?}"
                );
                if exception {
                    exceptions.push(lord_sign.house_from(sign));
                    // İstisnada pada evden 10. (lord 1. ya da 7.) ya da 4. (lord 4. ya da 10.).
                    let want = if matches!(lord_sign.house_from(sign), 1 | 7) {
                        10
                    } else {
                        4
                    };
                    assert_eq!(pada.house_from(sign), want);
                }
            }
            exceptions.sort_unstable();
            assert_eq!(exceptions, [1, 4, 7, 10], "{sign:?}");
        }
    }

    #[test]
    fn twelve_padas_from_the_lagna_and_none_without_it() {
        // Koç lagna. Güneş Aslan, Ay Boğa, Mars Yengeç, Merkür Başak, Jüpiter Yay, Venüs
        // Balık, Satürn Terazi, Rahu İkizler, Ketu Yay.
        let lons = [125.0, 40.0, 100.0, 160.0, 250.0, 340.0, 190.0, 70.0, 250.0];
        let p = Positions::synthetic(Some(5.0), lons);
        let padas = arudha_padas(&p, false).expect("lagna");
        for (i, pada) in padas.iter().enumerate() {
            assert_eq!(pada.house, i as u8 + 1);
        }
        // 1. ev Koç, lord Mars Yengeç'te (4.): istisna, pada Yengeç.
        assert_eq!(
            (padas[0].sign, padas[0].lord, padas[0].exception),
            (Cancer, Mars, true)
        );
        // 5. ev Aslan, lord Güneş Aslan'da: istisna, pada Aslan'dan 10. burç Boğa.
        assert_eq!(
            (padas[4].sign, padas[4].lord, padas[4].exception),
            (Taurus, Sun, true)
        );
        // 2. ev Boğa, lord Venüs Balık'ta (11.): Balık'tan 11. burç Oğlak.
        assert_eq!(
            (padas[1].sign, padas[1].lord, padas[1].exception),
            (Capricorn, Venus, false)
        );
        // 8. ev Akrep: eş lordlar devre dışıyken Mars (Yengeç, 9.): Yengeç'ten 9. burç Balık.
        assert_eq!((padas[7].sign, padas[7].lord), (Pisces, Mars));
        // Eş lordlar açıkken: Mars Yengeç'te yalnız, Ketu Yay'da Jüpiter'le: Ketu (2. ölçüt).
        // Ketu Yay'da (Akrep'ten 2.): Yay'dan 2. burç Oğlak.
        let with_co_lords = arudha_padas(&p, true).expect("lagna");
        assert_eq!(
            (with_co_lords[7].sign, with_co_lords[7].lord),
            (Capricorn, Ketu)
        );
        assert_eq!(arudha_padas(&p.clone().without_lagna(), true), None);
    }
}
````

- [ ] **Step 6: Modülleri kaydet ve dışa aktar**

````diff
diff --git a/crates/jyotish-core/src/jaimini/mod.rs b/crates/jyotish-core/src/jaimini/mod.rs
index c3de5df..8bdc531 100644
--- a/crates/jyotish-core/src/jaimini/mod.rs
+++ b/crates/jyotish-core/src/jaimini/mod.rs
@@ -1,13 +1,17 @@
-//! Jaimini çekirdeği (ileri modüller spec'i §5): chara karaka, karakamsha ve rashi drishti.
-//! Hesap fonksiyonları yalnızca `Positions` görür (K3).
+//! Jaimini çekirdeği (ileri modüller spec'i §5): chara karaka, karakamsha, rashi drishti, eş
+//! lordlar ve arudha padalar. Hesap fonksiyonları yalnızca `Positions` görür (K3).
 
+mod arudha;
 mod karaka;
+mod lords;
 mod rashi;
 
+pub use arudha::{arudha_of, arudha_padas, ArudhaPada};
 pub use karaka::{
     chara_karakas, karaka_roles, karakamsha, navamsa_sign, CharaKaraka, KarakaRole, Karakamsha,
     EIGHT_KARAKA_ROLES, KARAKA_TIE_DEG, SEVEN_KARAKA_ROLES,
 };
+pub use lords::{co_lord_choice, co_lords, jaimini_lord, stronger_co_lord, CoLordChoice};
 pub use rashi::{
     counts_forward, directed_count, graha_rashi_drishti, rashi_drishti_matrix, rashi_drishti_signs,
     sign_kind, SignKind,
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index a8a43fd..319c452 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -48,9 +48,11 @@ pub use error::CoreError;
 pub use graha::{Graha, GrahaPosition};
 pub use houses::{house_from_cusps, whole_sign_cusps, whole_sign_house, Bhavas, Lagna};
 pub use jaimini::{
-    chara_karakas, counts_forward, directed_count, graha_rashi_drishti, karaka_roles, karakamsha,
-    navamsa_sign, rashi_drishti_matrix, rashi_drishti_signs, sign_kind, CharaKaraka, KarakaRole,
-    Karakamsha, SignKind, EIGHT_KARAKA_ROLES, KARAKA_TIE_DEG, SEVEN_KARAKA_ROLES,
+    arudha_of, arudha_padas, chara_karakas, co_lord_choice, co_lords, counts_forward,
+    directed_count, graha_rashi_drishti, jaimini_lord, karaka_roles, karakamsha, navamsa_sign,
+    rashi_drishti_matrix, rashi_drishti_signs, sign_kind, stronger_co_lord, ArudhaPada,
+    CharaKaraka, CoLordChoice, KarakaRole, Karakamsha, SignKind, EIGHT_KARAKA_ROLES,
+    KARAKA_TIE_DEG, SEVEN_KARAKA_ROLES,
 };
 pub use lordship::{house_lord, house_sign, houses_owned};
 pub use nature::{kuja_dosha, moon_is_waxing, nature, KujaDosha, Nature, KUJA_DOSHA_HOUSES};
````

- [ ] **Step 7: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p jyotish-core 2>&1 | grep -E "^test result|Running"
```
Expected: fmt ve clippy çıktısız; lib `164 passed` (+13: 7 `lords`, 4 `arudha`, 2 proptest); golden `8 passed`; `modules_golden` `5 passed`. Bir el örneği düşerse önce `derivation` kuraldan yeniden hesaplanır.

- [ ] **Step 8: Commit**

```bash
git add crates/jyotish-core/src crates/jyotish-core/tests/modules_golden.rs tests/golden/modules
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): jaimini co-lords of Scorpio and Aquarius by six criteria; twelve arudha padas with the exception rule"
```
Expected: iki `grep` çıktısız.

---

### Task 3: Çekirdek: argala, `JaiminiReport` ve `compute_jaimini` toplayıcısı

Spec §5 "Argala" ve "Veri modeli ve IPC"; D10–D12. `karaka_scheme` ayarı burada hesaba girer (rapor şemaya göre 8 ya da 7 kayıt taşır). Toplayıcı temel haritayı `compute_chart` ile hesaplar (Plan 14 D8: ayrı bir hafif konum yolu yoktur), `time_known` yanlışsa lagnayı boşaltır ve temel haritanın `LowPrecision` uyarısını rapora taşır.

**Files:**
- Create: `crates/jyotish-core/src/jaimini/argala.rs`
- Modify: `crates/jyotish-core/src/jaimini/mod.rs`, `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/proptests.rs`, `crates/jyotish-core/tests/modules_golden.rs`, `tests/golden/modules/README.md`, `tests/golden/modules/jaimini/*.json` (8 dosya)
- Test: `argala.rs` ve `mod.rs` içindeki `mod tests`, `proptests.rs`, `tests/modules_golden.rs`

**Interfaces:**
- Consumes: Task 1 ve 2'nin tamamı; `nature`, `Nature` (K6); `compute_chart`, `ChartWarning`, `ChartInput`, `Positions::{from, without_lagna, lagna_sign}`; `AdvancedSettings { karaka_scheme, jaimini_co_lords }`; `fake::FakeEph` (testlerde).
- Produces (hepsi `jyotish_core`'dan dışa aktarılır):
  - `pub const ARGALA_PAIRS: [(u8, u8); 4]` (`[(2, 12), (4, 10), (11, 3), (5, 9)]`), `pub const THIRD_HOUSE_MALEFICS: usize` (3)
  - `pub enum ArgalaStatus { None, Stands, Contested, Obstructed }`, `pub fn argala_status(argala_count: usize, virodha_count: usize) -> ArgalaStatus`
  - `pub struct ArgalaPair { pub argala_house: u8, pub virodha_house: u8, pub argala_grahas: Vec<Graha>, pub virodha_grahas: Vec<Graha>, pub status: ArgalaStatus }`
  - `pub struct ArgalaRow { pub reference: Sign, pub pairs: [ArgalaPair; 4], pub third_house_malefic_argala: bool }`
  - `pub fn argala_row(reference: Sign, positions: &Positions) -> ArgalaRow`, `pub fn argala_rows(positions: &Positions) -> [ArgalaRow; 12]`
  - `pub const JAIMINI_METHOD: &str = "jaimini.core_v1"`
  - `pub struct JaiminiReport { pub scheme: KarakaScheme, pub co_lords: bool, pub method: String, pub karakas: Vec<CharaKaraka>, pub karakamsha: Karakamsha, pub rashi_drishti: [[bool; 12]; 12], pub graha_rashi_drishti: Vec<SignAspect>, pub arudhas: Option<[ArudhaPada; 12]>, pub argala: [ArgalaRow; 12], pub warnings: Vec<ChartWarning> }`
  - `pub fn jaimini_report(positions: &Positions, advanced: &AdvancedSettings) -> JaiminiReport` (`warnings` boş)
  - `pub fn compute_jaimini<E: Ephemeris + ?Sized>(eph: &mut E, input: &ChartInput) -> Result<JaiminiReport, CoreError>`
  - Düzenek: `JaiminiExpected.argala: Vec<ExpectedArgalaRow>` (en az iki referans).

- [ ] **Step 1: El örneklerine argala bölümünü ekle (kaynak: `hand`)**

Her dosyaya iki referans burç için "Argala" türetme adımı ve `argala` beklentisi eklenir: her çift için iki graha listesi (`Graha::ALL` sırasında), durum ve 3. ev kötücül argalası. Dört durumun dördü ve 3. ev kötücül argalası örneklerde bulunur; `time-unknown-third-house-malefics`'te üçüncü kötücül küçülen Ay'dır (K6).

````diff
diff --git a/tests/golden/modules/README.md b/tests/golden/modules/README.md
index 8936ce2..2ff0bce 100644
--- a/tests/golden/modules/README.md
+++ b/tests/golden/modules/README.md
@@ -42,6 +42,6 @@ yazılmaz.
 |---|---|---|
 | `foundation` | `waxing`, `nature` (dokuz graha), `kuja_dosha`, `house_lords` (12), `houses`, `houses_owned` | 6 |
 | `drishti` | `sign_aspects` (on üç tam bakışın tamamı, tablo sırasıyla), `sphuta` (en az on değer: `from`, `to`, `angle`, `value`) | 4 |
-| `jaimini` | `karakas` (rol sırasında: `role`, `graha`, `effective_deg`, `tie`), `karakamsha` (`atmakaraka`, `sign`, dokuz grahanın `houses_in_d1` ve `houses_in_d9` değeri), `co_lords` (Akrep ve Kova: `lord`, belirleyici ölçüt `step`), `arudhas` (on iki pada: `house`, `sign`, `lord`, `exception`; saat bilinmiyorsa `null`) | 8 |
+| `jaimini` | `karakas` (rol sırasında: `role`, `graha`, `effective_deg`, `tie`), `karakamsha` (`atmakaraka`, `sign`, dokuz grahanın `houses_in_d1` ve `houses_in_d9` değeri), `co_lords` (Akrep ve Kova: `lord`, belirleyici ölçüt `step`), `arudhas` (on iki pada: `house`, `sign`, `lord`, `exception`; saat bilinmiyorsa `null`), `argala` (en az iki referans burç: dört çiftin iki graha listesi ve durumu, `third_house_malefic_argala`) | 8 |
 
 Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
diff --git a/tests/golden/modules/jaimini/atmakaraka-vargottama-co-lords-off.json b/tests/golden/modules/jaimini/atmakaraka-vargottama-co-lords-off.json
index 7241d72..a389df2 100644
--- a/tests/golden/modules/jaimini/atmakaraka-vargottama-co-lords-off.json
+++ b/tests/golden/modules/jaimini/atmakaraka-vargottama-co-lords-off.json
@@ -11,7 +11,9 @@
     "Eş lord: ayar devre dışı; padalarda Akrep'in lordu Mars, Kova'nın lordu Satürn'dür. Seçim kuralı yine de izlenir (ayar açık olsaydı kullanılırdı). Akrep (Mars Boğa'da, Ketu Oğlak'ta): 1. ölçüt ayırmaz; 2. ölçüt: Mars yalnız (0), Ketu Merkür'le (1): Ketu. Kova (Satürn Terazi'de, Rahu Yengeç'te): 1. ölçüt ayırmaz; 2. ölçüt: ikisi de yalnız; 3. ölçüt: Satürn yücelme burcunda (Terazi): Satürn.",
     "Pada, Akrep lagna. 1. ev Akrep, Mars Boğa'da (7.): Boğa'dan 7. burç Akrep, evin kendisi; istisna: Akrep'ten 10. burç Aslan. 2. ev Yay, Jüpiter Balık'ta (4.): Balık'tan 4. burç İkizler, evden 7.; istisna: İkizler'den 10. burç Balık. 3. ev Oğlak, Satürn Terazi'de (10.): Terazi'den 10. burç Yengeç, evden 7.; istisna: Yengeç'ten 10. burç Koç. 4. ev Kova, Satürn Terazi'de (9.): Terazi'den 9. burç İkizler.",
     "Pada (5–8). 5. ev Balık, Jüpiter Balık'ta (1.): istisna: Balık'tan 10. burç Yay. 6. ev Koç, Mars Boğa'da (2.): Boğa'dan 2. burç İkizler. 7. ev Boğa, Venüs Balık'ta (11.): Balık'tan 11. burç Oğlak. 8. ev İkizler, Merkür Oğlak'ta (8.): Oğlak'tan 8. burç Aslan.",
-    "Pada (9–12). 9. ev Yengeç, Ay Aslan'da (2.): Aslan'dan 2. burç Başak. 10. ev Aslan, Güneş Kova'da (7.): Kova'dan 7. burç Aslan, evin kendisi; istisna: Aslan'dan 10. burç Boğa. 11. ev Başak, Merkür Oğlak'ta (5.): Oğlak'tan 5. burç Boğa. 12. ev Terazi, Venüs Balık'ta (6.): Balık'tan 6. burç Aslan."
+    "Pada (9–12). 9. ev Yengeç, Ay Aslan'da (2.): Aslan'dan 2. burç Başak. 10. ev Aslan, Güneş Kova'da (7.): Kova'dan 7. burç Aslan, evin kendisi; istisna: Aslan'dan 10. burç Boğa. 11. ev Başak, Merkür Oğlak'ta (5.): Oğlak'tan 5. burç Boğa. 12. ev Terazi, Venüs Balık'ta (6.): Balık'tan 6. burç Aslan.",
+    "Argala, referans Akrep: 2 (Yay: boş) ↔ 12 (Terazi: Satürn): yok. 4 (Kova: Güneş) ↔ 10 (Aslan: Ay): 1 = 1, çekişmeli. 11 (Başak: boş) ↔ 3 (Oğlak: Merkür, Ketu): yok. 5 (Balık: Jüpiter, Venüs) ↔ 9 (Yengeç: Rahu): 2 > 1, geçerli. 3. evde (Oğlak) iki kötücül var (Ketu; Merkür yalnızca Ketu'yla birlikte olduğundan kötücül): üçten az.",
+    "Argala, referans Balık: 2 (Koç: boş) ↔ 12 (Kova: Güneş): yok. 4 (İkizler: boş) ↔ 10 (Yay: boş): yok. 11 (Oğlak: Merkür, Ketu) ↔ 3 (Boğa: Mars): 2 > 1, geçerli. 5 (Yengeç: Rahu) ↔ 9 (Akrep: boş): geçerli. 3. evde (Boğa) tek kötücül var."
   ],
   "input": {
     "lagna_lon": 220.0,
@@ -48,6 +50,28 @@
       {"house": 10, "sign": "Taurus", "lord": "Sun", "exception": true},
       {"house": 11, "sign": "Taurus", "lord": "Mercury", "exception": false},
       {"house": 12, "sign": "Leo", "lord": "Venus", "exception": false}
+    ],
+    "argala": [
+      {
+        "reference": "Scorpio",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": [], "virodha_grahas": ["Saturn"], "status": "None"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": ["Sun"], "virodha_grahas": ["Moon"], "status": "Contested"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": [], "virodha_grahas": ["Mercury", "Ketu"], "status": "None"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": ["Jupiter", "Venus"], "virodha_grahas": ["Rahu"], "status": "Stands"}
+        ],
+        "third_house_malefic_argala": false
+      },
+      {
+        "reference": "Pisces",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": [], "virodha_grahas": ["Sun"], "status": "None"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": [], "virodha_grahas": [], "status": "None"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": ["Mercury", "Ketu"], "virodha_grahas": ["Mars"], "status": "Stands"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": ["Rahu"], "virodha_grahas": [], "status": "Stands"}
+        ],
+        "third_house_malefic_argala": false
+      }
     ]
   }
 }
diff --git a/tests/golden/modules/jaimini/co-lord-company-and-forward-count.json b/tests/golden/modules/jaimini/co-lord-company-and-forward-count.json
index f7e38f2..f043703 100644
--- a/tests/golden/modules/jaimini/co-lord-company-and-forward-count.json
+++ b/tests/golden/modules/jaimini/co-lord-company-and-forward-count.json
@@ -12,7 +12,9 @@
     "Eş lord, Kova (Satürn İkizler'de, Rahu Aslan'da): 1. ölçüt ayırmaz (Kova'daki Ketu Kova'nın eş lordu değildir). 2. ölçüt: Satürn Jüpiter ve Venüs'le (2), Rahu Mars'la (1): Satürn.",
     "Pada, Başak lagna. 1. ev Başak, Merkür Boğa'da (9.): Boğa'dan 9. burç Oğlak. 2. ev Terazi, Venüs İkizler'de (9.): İkizler'den 9. burç Kova. 3. ev Akrep, lord Mars Aslan'da (10.): Aslan'dan 10. burç Boğa, evden 7.; istisna: Boğa'dan 10. burç Kova. 4. ev Yay, Jüpiter İkizler'de (7.): İkizler'den 7. burç Yay, evin kendisi; istisna: Yay'dan 10. burç Başak.",
     "Pada (5–8). 5. ev Oğlak, Satürn İkizler'de (6.): İkizler'den 6. burç Akrep. 6. ev Kova, lord Satürn İkizler'de (5.): İkizler'den 5. burç Terazi. 7. ev Balık, Jüpiter İkizler'de (4.): İkizler'den 4. burç Başak, evden 7.; istisna: Başak'tan 10. burç İkizler. 8. ev Koç, Mars Aslan'da (5.): Aslan'dan 5. burç Yay.",
-    "Pada (9–12). 9. ev Boğa, Venüs İkizler'de (2.): İkizler'den 2. burç Yengeç. 10. ev İkizler, Merkür Boğa'da (12.): Boğa'dan 12. burç Koç. 11. ev Yengeç, Ay Kova'da (8.): Kova'dan 8. burç Başak. 12. ev Aslan, Güneş Boğa'da (10.): Boğa'dan 10. burç Kova, evden 7.; istisna: Kova'dan 10. burç Akrep."
+    "Pada (9–12). 9. ev Boğa, Venüs İkizler'de (2.): İkizler'den 2. burç Yengeç. 10. ev İkizler, Merkür Boğa'da (12.): Boğa'dan 12. burç Koç. 11. ev Yengeç, Ay Kova'da (8.): Kova'dan 8. burç Başak. 12. ev Aslan, Güneş Boğa'da (10.): Boğa'dan 10. burç Kova, evden 7.; istisna: Kova'dan 10. burç Akrep.",
+    "Argala, referans Başak: 2 (Terazi: boş) ↔ 12 (Aslan: Mars, Rahu): yok. 4 (Yay: boş) ↔ 10 (İkizler: Jüpiter, Venüs, Satürn): yok. 11 (Yengeç: boş) ↔ 3 (Akrep: boş): yok. 5 (Oğlak: boş) ↔ 9 (Boğa: Güneş, Merkür): yok. 3. ev (Akrep) boş.",
+    "Argala, referans Boğa: 2 (İkizler: Jüpiter, Venüs, Satürn) ↔ 12 (Koç: boş): geçerli. 4 (Aslan: Mars, Rahu) ↔ 10 (Kova: Ay, Ketu): 2 = 2, çekişmeli. 11 (Balık: boş) ↔ 3 (Yengeç: boş): yok. 5 (Başak: boş) ↔ 9 (Oğlak: boş): yok. 3. ev (Yengeç) boş."
   ],
   "input": {
     "lagna_lon": 165.0,
@@ -49,6 +51,28 @@
       {"house": 10, "sign": "Aries", "lord": "Mercury", "exception": false},
       {"house": 11, "sign": "Virgo", "lord": "Moon", "exception": false},
       {"house": 12, "sign": "Scorpio", "lord": "Sun", "exception": true}
+    ],
+    "argala": [
+      {
+        "reference": "Virgo",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": [], "virodha_grahas": ["Mars", "Rahu"], "status": "None"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": [], "virodha_grahas": ["Jupiter", "Venus", "Saturn"], "status": "None"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": [], "virodha_grahas": [], "status": "None"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": [], "virodha_grahas": ["Sun", "Mercury"], "status": "None"}
+        ],
+        "third_house_malefic_argala": false
+      },
+      {
+        "reference": "Taurus",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": ["Jupiter", "Venus", "Saturn"], "virodha_grahas": [], "status": "Stands"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": ["Mars", "Rahu"], "virodha_grahas": ["Moon", "Ketu"], "status": "Contested"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": [], "virodha_grahas": [], "status": "None"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": [], "virodha_grahas": [], "status": "None"}
+        ],
+        "third_house_malefic_argala": false
+      }
     ]
   }
 }
diff --git a/tests/golden/modules/jaimini/co-lord-exaltation-and-debilitation.json b/tests/golden/modules/jaimini/co-lord-exaltation-and-debilitation.json
index 5d3a56a..bb33b01 100644
--- a/tests/golden/modules/jaimini/co-lord-exaltation-and-debilitation.json
+++ b/tests/golden/modules/jaimini/co-lord-exaltation-and-debilitation.json
@@ -12,7 +12,9 @@
     "Eş lord, Kova (Satürn Koç'ta, Rahu Yay'da): 1. ölçüt ayırmaz. 2. ölçüt: ikisi de yalnız. 3. ölçüt: Satürn yücelme burcunda değil (Terazi), düşüş burcunda (Koç): düşüşte olmayan seçilir: Rahu.",
     "Pada, Aslan lagna. 1. ev Aslan, Güneş Başak'ta (2.): Başak'tan 2. burç Terazi. 2. ev Başak, Merkür Başak'ta (1.): istisna: Başak'tan 10. burç İkizler. 3. ev Terazi, Venüs Terazi'de (1.): istisna: Terazi'den 10. burç Yengeç. 4. ev Akrep, lord Mars Oğlak'ta (3.): Oğlak'tan 3. burç Balık.",
     "Pada (5–8). 5. ev Yay, Jüpiter Yengeç'te (8.): Yengeç'ten 8. burç Kova. 6. ev Oğlak, Satürn Koç'ta (4.): Koç'tan 4. burç Yengeç, evden 7.; istisna: Yengeç'ten 10. burç Koç. 7. ev Kova, lord Rahu Yay'da (11.): Yay'dan 11. burç Terazi. 8. ev Balık, Jüpiter Yengeç'te (5.): Yengeç'ten 5. burç Akrep.",
-    "Pada (9–12). 9. ev Koç, Mars Oğlak'ta (10.): Oğlak'tan 10. burç Terazi, evden 7.; istisna: Terazi'den 10. burç Yengeç. 10. ev Boğa, Venüs Terazi'de (6.): Terazi'den 6. burç Balık. 11. ev İkizler, Merkür Başak'ta (4.): Başak'tan 4. burç Yay, evden 7.; istisna: Yay'dan 10. burç Başak. 12. ev Yengeç, Ay Balık'ta (9.): Balık'tan 9. burç Akrep."
+    "Pada (9–12). 9. ev Koç, Mars Oğlak'ta (10.): Oğlak'tan 10. burç Terazi, evden 7.; istisna: Terazi'den 10. burç Yengeç. 10. ev Boğa, Venüs Terazi'de (6.): Terazi'den 6. burç Balık. 11. ev İkizler, Merkür Başak'ta (4.): Başak'tan 4. burç Yay, evden 7.; istisna: Yay'dan 10. burç Başak. 12. ev Yengeç, Ay Balık'ta (9.): Balık'tan 9. burç Akrep.",
+    "Argala, referans Aslan: 2 (Başak: Güneş, Merkür) ↔ 12 (Yengeç: Jüpiter): 2 > 1, geçerli. 4 (Akrep: boş) ↔ 10 (Boğa: boş): yok. 11 (İkizler: Ketu) ↔ 3 (Terazi: Venüs): çekişmeli. 5 (Yay: Rahu) ↔ 9 (Koç: Satürn): çekişmeli. 3. evde (Terazi) kötücül yok.",
+    "Argala, referans Kova: 2 (Balık: Ay) ↔ 12 (Oğlak: Mars): çekişmeli. 4 (Boğa: boş) ↔ 10 (Akrep: boş): yok. 11 (Yay: Rahu) ↔ 3 (Koç: Satürn): çekişmeli. 5 (İkizler: Ketu) ↔ 9 (Terazi: Venüs): çekişmeli. 3. evde (Koç) tek kötücül var."
   ],
   "input": {
     "lagna_lon": 130.0,
@@ -49,6 +51,28 @@
       {"house": 10, "sign": "Pisces", "lord": "Venus", "exception": false},
       {"house": 11, "sign": "Virgo", "lord": "Mercury", "exception": true},
       {"house": 12, "sign": "Scorpio", "lord": "Moon", "exception": false}
+    ],
+    "argala": [
+      {
+        "reference": "Leo",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": ["Sun", "Mercury"], "virodha_grahas": ["Jupiter"], "status": "Stands"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": [], "virodha_grahas": [], "status": "None"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": ["Ketu"], "virodha_grahas": ["Venus"], "status": "Contested"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": ["Rahu"], "virodha_grahas": ["Saturn"], "status": "Contested"}
+        ],
+        "third_house_malefic_argala": false
+      },
+      {
+        "reference": "Aquarius",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": ["Moon"], "virodha_grahas": ["Mars"], "status": "Contested"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": [], "virodha_grahas": [], "status": "None"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": ["Rahu"], "virodha_grahas": ["Saturn"], "status": "Contested"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": ["Ketu"], "virodha_grahas": ["Venus"], "status": "Contested"}
+        ],
+        "third_house_malefic_argala": false
+      }
     ]
   }
 }
diff --git a/tests/golden/modules/jaimini/co-lords-in-the-same-sign.json b/tests/golden/modules/jaimini/co-lords-in-the-same-sign.json
index 5626e67..309db3e 100644
--- a/tests/golden/modules/jaimini/co-lords-in-the-same-sign.json
+++ b/tests/golden/modules/jaimini/co-lords-in-the-same-sign.json
@@ -12,7 +12,9 @@
     "Eş lord, Kova (Satürn ve Rahu Boğa'da): 1. ölçüt ayırmaz (ikisi de Kova'nın dışında). 2. ölçüt: Boğa'da başka graha yok (0–0). 3. ölçüt: Satürn Boğa'da ne yücelmiş ne düşmüş. 4. ve 5. ölçüt: aynı burç, eşit. 6. ölçüt: Satürn.",
     "Pada, Kova lagna. 1. ev Kova, lord Satürn Boğa'da (4.): Boğa'dan 4. burç Aslan, evden 7.; istisna: Aslan'dan 10. burç Boğa. 2. ev Balık, Jüpiter Kova'da (12.): Kova'dan 12. burç Oğlak. 3. ev Koç, Mars Akrep'te (8.): Akrep'ten 8. burç İkizler. 4. ev Boğa, Venüs Terazi'de (6.): Terazi'den 6. burç Balık.",
     "Pada (5–8). 5. ev İkizler, Merkür Yay'da (7.): Yay'dan 7. burç İkizler, evin kendisi; istisna: İkizler'den 10. burç Balık. 6. ev Yengeç, Ay Yengeç'te (1.): istisna: Yengeç'ten 10. burç Koç. 7. ev Aslan, Güneş Akrep'te (4.): Akrep'ten 4. burç Kova, evden 7.; istisna: Kova'dan 10. burç Akrep. 8. ev Başak, Merkür Yay'da (4.): Yay'dan 4. burç Balık, evden 7.; istisna: Balık'tan 10. burç Yay.",
-    "Pada (9–12). 9. ev Terazi, Venüs Terazi'de (1.): istisna: Terazi'den 10. burç Yengeç. 10. ev Akrep, lord Mars Akrep'te (1.): istisna: Akrep'ten 10. burç Aslan. 11. ev Yay, Jüpiter Kova'da (3.): Kova'dan 3. burç Koç. 12. ev Oğlak, Satürn Boğa'da (5.): Boğa'dan 5. burç Başak."
+    "Pada (9–12). 9. ev Terazi, Venüs Terazi'de (1.): istisna: Terazi'den 10. burç Yengeç. 10. ev Akrep, lord Mars Akrep'te (1.): istisna: Akrep'ten 10. burç Aslan. 11. ev Yay, Jüpiter Kova'da (3.): Kova'dan 3. burç Koç. 12. ev Oğlak, Satürn Boğa'da (5.): Boğa'dan 5. burç Başak.",
+    "Argala, referans Kova: 2 (Balık: boş) ↔ 12 (Oğlak: boş): yok. 4 (Boğa: Satürn, Rahu) ↔ 10 (Akrep: Güneş, Mars, Ketu): 2 < 3, engelli. 11 (Yay: Merkür) ↔ 3 (Koç: boş): geçerli. 5 (İkizler: boş) ↔ 9 (Terazi: Venüs): yok. 3. ev (Koç) boş.",
+    "Argala, referans Başak: 2 (Terazi: Venüs) ↔ 12 (Aslan: boş): geçerli. 4 (Yay: Merkür) ↔ 10 (İkizler: boş): geçerli. 11 (Yengeç: Ay) ↔ 3 (Akrep: Güneş, Mars, Ketu): 1 < 3, engelli. 5 (Oğlak: boş) ↔ 9 (Boğa: Satürn, Rahu): yok. 3. evde (Akrep) üç kötücül var (Güneş, Mars, Ketu): 3. ev kötücül argalası var."
   ],
   "input": {
     "lagna_lon": 310.0,
@@ -49,6 +51,28 @@
       {"house": 10, "sign": "Leo", "lord": "Mars", "exception": true},
       {"house": 11, "sign": "Aries", "lord": "Jupiter", "exception": false},
       {"house": 12, "sign": "Virgo", "lord": "Saturn", "exception": false}
+    ],
+    "argala": [
+      {
+        "reference": "Aquarius",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": [], "virodha_grahas": [], "status": "None"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": ["Saturn", "Rahu"], "virodha_grahas": ["Sun", "Mars", "Ketu"], "status": "Obstructed"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": ["Mercury"], "virodha_grahas": [], "status": "Stands"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": [], "virodha_grahas": ["Venus"], "status": "None"}
+        ],
+        "third_house_malefic_argala": false
+      },
+      {
+        "reference": "Virgo",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": ["Venus"], "virodha_grahas": [], "status": "Stands"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": ["Mercury"], "virodha_grahas": [], "status": "Stands"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": ["Moon"], "virodha_grahas": ["Sun", "Mars", "Ketu"], "status": "Obstructed"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": [], "virodha_grahas": ["Saturn", "Rahu"], "status": "None"}
+        ],
+        "third_house_malefic_argala": true
+      }
     ]
   }
 }
diff --git a/tests/golden/modules/jaimini/node-in-its-sign-and-sign-kind.json b/tests/golden/modules/jaimini/node-in-its-sign-and-sign-kind.json
index cc886ee..b31ba9d 100644
--- a/tests/golden/modules/jaimini/node-in-its-sign-and-sign-kind.json
+++ b/tests/golden/modules/jaimini/node-in-its-sign-and-sign-kind.json
@@ -12,7 +12,9 @@
     "Eş lord, Kova (Satürn Yengeç'te, Rahu Boğa'da): 1. ölçüt ayırmaz. 2. ölçüt: ikisi de yalnız. 3. ölçüt: Satürn Yengeç'te ne yücelmiş ne düşmüş. 4. ölçüt: Yengeç yürük, Boğa sabit: Rahu.",
     "Pada, İkizler lagna. 1. ev İkizler, Merkür Koç'ta (11.): Koç'tan 11. burç Kova. 2. ev Yengeç, Ay Oğlak'ta (7.): Oğlak'tan 7. burç Yengeç, evin kendisi; istisna: Yengeç'ten 10. burç Koç. 3. ev Aslan, Güneş Koç'ta (9.): Koç'tan 9. burç Yay. 4. ev Başak, Merkür Koç'ta (8.): Koç'tan 8. burç Akrep.",
     "Pada (5–8). 5. ev Terazi, Venüs Balık'ta (6.): Balık'tan 6. burç Aslan. 6. ev Akrep, lord Mars Başak'ta (11.): Başak'tan 11. burç Yengeç. 7. ev Yay, Jüpiter Aslan'da (9.): Aslan'dan 9. burç Koç. 8. ev Oğlak, Satürn Yengeç'te (7.): Yengeç'ten 7. burç Oğlak, evin kendisi; istisna: Oğlak'tan 10. burç Terazi.",
-    "Pada (9–12). 9. ev Kova, lord Rahu Boğa'da (4.): Boğa'dan 4. burç Aslan, evden 7.; istisna: Aslan'dan 10. burç Boğa. 10. ev Balık, Jüpiter Aslan'da (6.): Aslan'dan 6. burç Oğlak. 11. ev Koç, Mars Başak'ta (6.): Başak'tan 6. burç Kova. 12. ev Boğa, Venüs Balık'ta (11.): Balık'tan 11. burç Oğlak."
+    "Pada (9–12). 9. ev Kova, lord Rahu Boğa'da (4.): Boğa'dan 4. burç Aslan, evden 7.; istisna: Aslan'dan 10. burç Boğa. 10. ev Balık, Jüpiter Aslan'da (6.): Aslan'dan 6. burç Oğlak. 11. ev Koç, Mars Başak'ta (6.): Başak'tan 6. burç Kova. 12. ev Boğa, Venüs Balık'ta (11.): Balık'tan 11. burç Oğlak.",
+    "Argala, referans İkizler: 2 (Yengeç: Satürn) ↔ 12 (Boğa: Rahu): çekişmeli. 4 (Başak: Mars) ↔ 10 (Balık: Venüs): çekişmeli. 11 (Koç: Güneş, Merkür) ↔ 3 (Aslan: Jüpiter): 2 > 1, geçerli. 5 (Terazi: boş) ↔ 9 (Kova: boş): yok. 3. evde (Aslan) kötücül yok.",
+    "Argala, referans Oğlak: 2 (Kova: boş) ↔ 12 (Yay: boş): yok. 4 (Koç: Güneş, Merkür) ↔ 10 (Terazi: boş): geçerli. 11 (Akrep: Ketu) ↔ 3 (Balık: Venüs): çekişmeli. 5 (Boğa: Rahu) ↔ 9 (Başak: Mars): çekişmeli. 3. evde (Balık) kötücül yok."
   ],
   "input": {
     "lagna_lon": 75.0,
@@ -48,6 +50,28 @@
       {"house": 10, "sign": "Capricorn", "lord": "Jupiter", "exception": false},
       {"house": 11, "sign": "Aquarius", "lord": "Mars", "exception": false},
       {"house": 12, "sign": "Capricorn", "lord": "Venus", "exception": false}
+    ],
+    "argala": [
+      {
+        "reference": "Gemini",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": ["Saturn"], "virodha_grahas": ["Rahu"], "status": "Contested"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": ["Mars"], "virodha_grahas": ["Venus"], "status": "Contested"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": ["Sun", "Mercury"], "virodha_grahas": ["Jupiter"], "status": "Stands"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": [], "virodha_grahas": [], "status": "None"}
+        ],
+        "third_house_malefic_argala": false
+      },
+      {
+        "reference": "Capricorn",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": [], "virodha_grahas": [], "status": "None"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": ["Sun", "Mercury"], "virodha_grahas": [], "status": "Stands"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": ["Ketu"], "virodha_grahas": ["Venus"], "status": "Contested"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": ["Rahu"], "virodha_grahas": ["Mars"], "status": "Contested"}
+        ],
+        "third_house_malefic_argala": false
+      }
     ]
   }
 }
diff --git a/tests/golden/modules/jaimini/rahu-atmakaraka-arudha-exceptions.json b/tests/golden/modules/jaimini/rahu-atmakaraka-arudha-exceptions.json
index a9db36a..1ed3268 100644
--- a/tests/golden/modules/jaimini/rahu-atmakaraka-arudha-exceptions.json
+++ b/tests/golden/modules/jaimini/rahu-atmakaraka-arudha-exceptions.json
@@ -12,7 +12,9 @@
     "Eş lord, Kova (Satürn Oğlak'ta, Rahu İkizler'de): 1. ölçüt ayırmaz. 2. ölçüt: ikisi de yalnız (0–0). 3. ölçüt: Satürn Oğlak'ta ne yücelmiş ne düşmüş. 4. ölçüt: Oğlak yürük, İkizler çift doğalı: Rahu.",
     "Pada, Koç lagna. 1. ev Koç, lord Mars Koç'ta (1.): hesaplanan burç Koç, evin kendisi; istisna: Koç'tan 10. burç Oğlak. 2. ev Boğa, Venüs Terazi'de (6.): Terazi'den 6. burç Balık. 3. ev İkizler, Merkür Yay'da (7.): Yay'dan 7. burç İkizler, evin kendisi; istisna: İkizler'den 10. burç Balık. 4. ev Yengeç, Ay Koç'ta (10.): Koç'tan 10. burç Oğlak, evden 7.; istisna: Oğlak'tan 10. burç Terazi.",
     "Pada (5–8). 5. ev Aslan, Güneş Akrep'te (4.): Akrep'ten 4. burç Kova, evden 7.; istisna: Kova'dan 10. burç Akrep. 6. ev Başak, Merkür Yay'da (4.): Yay'dan 4. burç Balık, evden 7.; istisna: Balık'tan 10. burç Yay. 7. ev Terazi, Venüs Terazi'de (1.): istisna: Terazi'den 10. burç Yengeç. 8. ev Akrep, lord Ketu Yay'da (2.): Yay'dan 2. burç Oğlak.",
-    "Pada (9–12). 9. ev Yay, Jüpiter Boğa'da (6.): Boğa'dan 6. burç Terazi. 10. ev Oğlak, Satürn Oğlak'ta (1.): istisna: Oğlak'tan 10. burç Terazi. 11. ev Kova, lord Rahu İkizler'de (5.): İkizler'den 5. burç Terazi. 12. ev Balık, Jüpiter Boğa'da (3.): Boğa'dan 3. burç Yengeç."
+    "Pada (9–12). 9. ev Yay, Jüpiter Boğa'da (6.): Boğa'dan 6. burç Terazi. 10. ev Oğlak, Satürn Oğlak'ta (1.): istisna: Oğlak'tan 10. burç Terazi. 11. ev Kova, lord Rahu İkizler'de (5.): İkizler'den 5. burç Terazi. 12. ev Balık, Jüpiter Boğa'da (3.): Boğa'dan 3. burç Yengeç.",
+    "Argala, referans Koç: 2 (Boğa: Jüpiter) ↔ 12 (Balık: boş): 1 > 0, geçerli. 4 (Yengeç: boş) ↔ 10 (Oğlak: Satürn): argala tarafı boş, yok. 11 (Kova: boş) ↔ 3 (İkizler: Rahu): yok. 5 (Aslan: boş) ↔ 9 (Yay: Merkür, Ketu): yok. 3. evde (İkizler) tek kötücül var: Rahu.",
+    "Argala, referans Yay: 2 (Oğlak: Satürn) ↔ 12 (Akrep: Güneş): 1 = 1, çekişmeli. 4 (Balık: boş) ↔ 10 (Başak: boş): yok. 11 (Terazi: Venüs) ↔ 3 (Kova: boş): geçerli. 5 (Koç: Ay, Mars) ↔ 9 (Aslan: boş): geçerli. 3. ev (Kova) boş."
   ],
   "input": {
     "lagna_lon": 12.0,
@@ -49,6 +51,28 @@
       {"house": 10, "sign": "Libra", "lord": "Saturn", "exception": true},
       {"house": 11, "sign": "Libra", "lord": "Rahu", "exception": false},
       {"house": 12, "sign": "Cancer", "lord": "Jupiter", "exception": false}
+    ],
+    "argala": [
+      {
+        "reference": "Aries",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": ["Jupiter"], "virodha_grahas": [], "status": "Stands"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": [], "virodha_grahas": ["Saturn"], "status": "None"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": [], "virodha_grahas": ["Rahu"], "status": "None"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": [], "virodha_grahas": ["Mercury", "Ketu"], "status": "None"}
+        ],
+        "third_house_malefic_argala": false
+      },
+      {
+        "reference": "Sagittarius",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": ["Saturn"], "virodha_grahas": ["Sun"], "status": "Contested"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": [], "virodha_grahas": [], "status": "None"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": ["Venus"], "virodha_grahas": [], "status": "Stands"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": ["Moon", "Mars"], "virodha_grahas": [], "status": "Stands"}
+        ],
+        "third_house_malefic_argala": false
+      }
     ]
   }
 }
diff --git a/tests/golden/modules/jaimini/tie-within-one-arcsecond-seven-scheme.json b/tests/golden/modules/jaimini/tie-within-one-arcsecond-seven-scheme.json
index 0f342c8..5df2b2a 100644
--- a/tests/golden/modules/jaimini/tie-within-one-arcsecond-seven-scheme.json
+++ b/tests/golden/modules/jaimini/tie-within-one-arcsecond-seven-scheme.json
@@ -12,7 +12,9 @@
     "Eş lord, Kova (Satürn Yay'da, Rahu Başak'ta): 1. ölçüt ayırmaz. 2. ölçüt: ikisi de yalnız. 3. ölçüt: Satürn Yay'da ne yücelmiş ne düşmüş. 4. ölçüt: Yay ve Başak çift doğalı: eşit. 5. ölçüt: Kova geri sayar: Kova 1, Oğlak 2, Yay 3 (Satürn); Akrep 4, Terazi 5, Başak 6 (Rahu): Rahu daha uzak.",
     "Pada, Yengeç lagna. 1. ev Yengeç, Ay Boğa'da (11.): Boğa'dan 11. burç Balık. 2. ev Aslan, Güneş İkizler'de (11.): İkizler'den 11. burç Koç. 3. ev Başak, Merkür İkizler'de (10.): İkizler'den 10. burç Balık, evden 7.; istisna: Balık'tan 10. burç Yay. 4. ev Terazi, Venüs Yengeç'te (10.): Yengeç'ten 10. burç Koç, evden 7.; istisna: Koç'tan 10. burç Oğlak.",
     "Pada (5–8). 5. ev Akrep, lord Ketu Balık'ta (5.): Balık'tan 5. burç Yengeç. 6. ev Yay, Jüpiter Balık'ta (4.): Balık'tan 4. burç İkizler, evden 7.; istisna: İkizler'den 10. burç Balık. 7. ev Oğlak, Satürn Yay'da (12.): Yay'dan 12. burç Akrep. 8. ev Kova, lord Rahu Başak'ta (8.): Başak'tan 8. burç Koç.",
-    "Pada (9–12). 9. ev Balık, Jüpiter Balık'ta (1.): istisna: Balık'tan 10. burç Yay. 10. ev Koç, Mars Aslan'da (5.): Aslan'dan 5. burç Yay. 11. ev Boğa, Venüs Yengeç'te (3.): Yengeç'ten 3. burç Başak. 12. ev İkizler, Merkür İkizler'de (1.): istisna: İkizler'den 10. burç Balık."
+    "Pada (9–12). 9. ev Balık, Jüpiter Balık'ta (1.): istisna: Balık'tan 10. burç Yay. 10. ev Koç, Mars Aslan'da (5.): Aslan'dan 5. burç Yay. 11. ev Boğa, Venüs Yengeç'te (3.): Yengeç'ten 3. burç Başak. 12. ev İkizler, Merkür İkizler'de (1.): istisna: İkizler'den 10. burç Balık.",
+    "Argala, referans Yengeç: 2 (Aslan: Mars) ↔ 12 (İkizler: Güneş, Merkür): 1 < 2, engelli. 4 (Terazi: boş) ↔ 10 (Koç: boş): yok. 11 (Boğa: Ay) ↔ 3 (Başak: Rahu): 1 = 1, çekişmeli. 5 (Akrep: boş) ↔ 9 (Balık: Jüpiter, Ketu): yok. 3. evde (Başak) tek kötücül var.",
+    "Argala, referans Boğa: 2 (İkizler: Güneş, Merkür) ↔ 12 (Koç: boş): geçerli. 4 (Aslan: Mars) ↔ 10 (Kova: boş): geçerli. 11 (Balık: Jüpiter, Ketu) ↔ 3 (Yengeç: Venüs): 2 > 1, geçerli. 5 (Başak: Rahu) ↔ 9 (Oğlak: boş): geçerli. 3. evde (Yengeç) kötücül yok."
   ],
   "input": {
     "lagna_lon": 100.0,
@@ -48,6 +50,28 @@
       {"house": 10, "sign": "Sagittarius", "lord": "Mars", "exception": false},
       {"house": 11, "sign": "Virgo", "lord": "Venus", "exception": false},
       {"house": 12, "sign": "Pisces", "lord": "Mercury", "exception": true}
+    ],
+    "argala": [
+      {
+        "reference": "Cancer",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": ["Mars"], "virodha_grahas": ["Sun", "Mercury"], "status": "Obstructed"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": [], "virodha_grahas": [], "status": "None"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": ["Moon"], "virodha_grahas": ["Rahu"], "status": "Contested"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": [], "virodha_grahas": ["Jupiter", "Ketu"], "status": "None"}
+        ],
+        "third_house_malefic_argala": false
+      },
+      {
+        "reference": "Taurus",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": ["Sun", "Mercury"], "virodha_grahas": [], "status": "Stands"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": ["Mars"], "virodha_grahas": [], "status": "Stands"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": ["Jupiter", "Ketu"], "virodha_grahas": ["Venus"], "status": "Stands"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": ["Rahu"], "virodha_grahas": [], "status": "Stands"}
+        ],
+        "third_house_malefic_argala": false
+      }
     ]
   }
 }
diff --git a/tests/golden/modules/jaimini/time-unknown-third-house-malefics.json b/tests/golden/modules/jaimini/time-unknown-third-house-malefics.json
index 3aeeba0..5a63d99 100644
--- a/tests/golden/modules/jaimini/time-unknown-third-house-malefics.json
+++ b/tests/golden/modules/jaimini/time-unknown-third-house-malefics.json
@@ -10,7 +10,9 @@
     "D1 evleri, Başak'tan (Güneş…Ketu): İkizler 10, İkizler 10, İkizler 10, Boğa 9, Terazi 2, Yengeç 11, Kova 6, Akrep 3, Boğa 9. D9 evleri, Başak'tan: Kova 6, Terazi 2, Boğa 9, Başak 1, Yay 4, Başak 1, Balık 7, Terazi 2, Koç 8.",
     "Eş lord, Akrep (Mars İkizler'de, Ketu Boğa'da): 1. ölçüt ayırmaz (Akrep'teki Rahu Akrep'in eş lordu değildir). 2. ölçüt: Mars Güneş ve Ay'la (2), Ketu Merkür'le (1): Mars.",
     "Eş lord, Kova (Satürn Kova'da, Rahu Akrep'te): 1. ölçüt: yalnızca Satürn burcun kendisinde; burcun dışındaki lord seçilir: Rahu.",
-    "Pada: lagna olmadığı için padalar verilmez."
+    "Pada: lagna olmadığı için padalar verilmez.",
+    "Argala, referans Koç (lagna olmadığından satırlar Koç'tan sıralanır): 2 (Boğa: Merkür, Ketu) ↔ 12 (Balık: boş): geçerli. 4 (Yengeç: Venüs) ↔ 10 (Oğlak: boş): geçerli. 11 (Kova: Satürn) ↔ 3 (İkizler: Güneş, Ay, Mars): 1 < 3, engelli. 5 (Aslan: boş) ↔ 9 (Yay: boş): yok. 3. evde (İkizler) üç kötücül var: Güneş, Mars ve küçülen Ay (uzanım 62 − 74 ≡ 348°): 3. ev kötücül argalası var.",
+    "Argala, referans Aslan: 2 (Başak: boş) ↔ 12 (Yengeç: Venüs): yok. 4 (Akrep: Rahu) ↔ 10 (Boğa: Merkür, Ketu): 1 < 2, engelli. 11 (İkizler: Güneş, Ay, Mars) ↔ 3 (Terazi: Jüpiter): 3 > 1, geçerli. 5 (Yay: boş) ↔ 9 (Koç: boş): yok. 3. evde (Terazi) kötücül yok."
   ],
   "input": {
     "lagna_lon": null,
@@ -34,6 +36,28 @@
       "houses_in_d9": {"Sun": 6, "Moon": 2, "Mars": 9, "Mercury": 1, "Jupiter": 4, "Venus": 1, "Saturn": 7, "Rahu": 2, "Ketu": 8}
     },
     "co_lords": {"Scorpio": {"lord": "Mars", "step": 2}, "Aquarius": {"lord": "Rahu", "step": 1}},
-    "arudhas": null
+    "arudhas": null,
+    "argala": [
+      {
+        "reference": "Aries",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": ["Mercury", "Ketu"], "virodha_grahas": [], "status": "Stands"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": ["Venus"], "virodha_grahas": [], "status": "Stands"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": ["Saturn"], "virodha_grahas": ["Sun", "Moon", "Mars"], "status": "Obstructed"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": [], "virodha_grahas": [], "status": "None"}
+        ],
+        "third_house_malefic_argala": true
+      },
+      {
+        "reference": "Leo",
+        "pairs": [
+          {"argala_house": 2, "virodha_house": 12, "argala_grahas": [], "virodha_grahas": ["Venus"], "status": "None"},
+          {"argala_house": 4, "virodha_house": 10, "argala_grahas": ["Rahu"], "virodha_grahas": ["Mercury", "Ketu"], "status": "Obstructed"},
+          {"argala_house": 11, "virodha_house": 3, "argala_grahas": ["Sun", "Moon", "Mars"], "virodha_grahas": ["Jupiter"], "status": "Stands"},
+          {"argala_house": 5, "virodha_house": 9, "argala_grahas": [], "virodha_grahas": [], "status": "None"}
+        ],
+        "third_house_malefic_argala": false
+      }
+    ]
   }
 }
````

- [ ] **Step 2: Düzeneği ve değişmezleri büyüt (başarısız testler)**

Düzenek her örnekte en az iki referans satırını (dört çift ve 3. ev işareti) karşılaştırır, ilk satırın lagna burcu (lagna yoksa Koç) olduğunu denetler ve `jaimini_report`'un aynı parçalardan kurulduğunu doğrular. Kapsam testi dört durumu ve 3. ev kötücül argalasını sabitler.

Değişmez (`report_is_consistent`): rapor iki ayarı yansıtır; Atmakaraka karakamshanın grahasıdır; padalar yalnızca lagna varken doludur; argala satırları lagna burcundan (ya da Koç'tan) sıralıdır; aynı graha bir çiftin iki tarafında görünmez; durum yalnızca sayılardan çıkar; listelerdeki her graha gerçekten o evdedir ve sekiz ev (2, 3, 4, 5, 9, 10, 11, 12) birer kez sayılır; lagnayı kaldırmak yalnızca padaları boşaltır.

````diff
diff --git a/crates/jyotish-core/src/proptests.rs b/crates/jyotish-core/src/proptests.rs
index dabd332..2a8c2ae 100644
--- a/crates/jyotish-core/src/proptests.rs
+++ b/crates/jyotish-core/src/proptests.rs
@@ -490,11 +490,12 @@ mod jaimini_properties {
     use super::foundation_properties::positions;
     use crate::graha::Graha;
     use crate::jaimini::{
-        arudha_padas, chara_karakas, co_lord_choice, co_lords, graha_rashi_drishti, karaka_roles,
-        karakamsha, navamsa_sign, rashi_drishti_signs, KARAKA_TIE_DEG,
+        arudha_padas, chara_karakas, co_lord_choice, co_lords, graha_rashi_drishti, jaimini_report,
+        karaka_roles, karakamsha, navamsa_sign, rashi_drishti_signs, ArgalaStatus, JAIMINI_METHOD,
+        KARAKA_TIE_DEG,
     };
     use crate::lordship::house_sign;
-    use crate::settings::KarakaScheme;
+    use crate::settings::{AdvancedSettings, KarakaScheme};
     use crate::varga::is_vargottama;
     use crate::zodiac::Sign;
 
@@ -618,5 +619,55 @@ mod jaimini_properties {
                 (Some(_), None) => prop_assert!(false, "padas with a lagna"),
             }
         }
+
+        #[test]
+        fn report_is_consistent(p in positions(), s in scheme(), co in any::<bool>()) {
+            let advanced = AdvancedSettings { karaka_scheme: s, jaimini_co_lords: co };
+            let r = jaimini_report(&p, &advanced);
+            prop_assert_eq!(r.method.as_str(), JAIMINI_METHOD);
+            prop_assert_eq!((r.scheme, r.co_lords), (s, co));
+            prop_assert_eq!(r.karakamsha.atmakaraka, r.karakas[0].graha);
+            prop_assert_eq!(r.arudhas.is_some(), p.lagna_sign.is_some());
+            prop_assert!(r.warnings.is_empty());
+            let first = p.lagna_sign.unwrap_or(Sign::Aries);
+            for (i, row) in r.argala.iter().enumerate() {
+                prop_assert_eq!(row.reference, Sign::from_index(first.index() + i));
+                let mut seen = 0usize;
+                for pair in &row.pairs {
+                    // Aynı graha bir çiftin iki tarafında görünmez.
+                    prop_assert!(pair.argala_grahas.iter().all(|g| !pair.virodha_grahas.contains(g)));
+                    let (a, v) = (pair.argala_grahas.len(), pair.virodha_grahas.len());
+                    let want = if a == 0 {
+                        ArgalaStatus::None
+                    } else if a > v {
+                        ArgalaStatus::Stands
+                    } else if a == v {
+                        ArgalaStatus::Contested
+                    } else {
+                        ArgalaStatus::Obstructed
+                    };
+                    prop_assert_eq!(pair.status, want);
+                    for &g in &pair.argala_grahas {
+                        prop_assert_eq!(p.house_from(g, row.reference), pair.argala_house);
+                    }
+                    for &g in &pair.virodha_grahas {
+                        prop_assert_eq!(p.house_from(g, row.reference), pair.virodha_house);
+                    }
+                    seen += a + v;
+                }
+                // Sekiz ev (2, 3, 4, 5, 9, 10, 11, 12) birer kez sayılır.
+                let counted = Graha::ALL
+                    .into_iter()
+                    .filter(|&g| !matches!(p.house_from(g, row.reference), 1 | 6 | 7 | 8))
+                    .count();
+                prop_assert_eq!(seen, counted);
+            }
+            // Lagna yalnızca padaları ve argala satır sırasını etkiler.
+            let q = jaimini_report(&p.clone().without_lagna(), &advanced);
+            prop_assert_eq!(&q.karakas, &r.karakas);
+            prop_assert_eq!(&q.karakamsha, &r.karakamsha);
+            prop_assert_eq!(&q.graha_rashi_drishti, &r.graha_rashi_drishti);
+            prop_assert_eq!(q.arudhas, None);
+        }
     }
 }
diff --git a/crates/jyotish-core/tests/modules_golden.rs b/crates/jyotish-core/tests/modules_golden.rs
index d7e7fad..a35fda9 100644
--- a/crates/jyotish-core/tests/modules_golden.rs
+++ b/crates/jyotish-core/tests/modules_golden.rs
@@ -7,9 +7,10 @@ use std::collections::BTreeMap;
 use std::path::PathBuf;
 
 use jyotish_core::{
-    arudha_padas, chara_karakas, co_lord_choice, drishti_table, house_lord, houses_owned,
-    jaimini_lord, karakamsha, kuja_dosha, moon_is_waxing, nature, normalize, AdvancedSettings,
-    Graha, KarakaRole, KujaDosha, Nature, Positions, Sign, ASPECTING_GRAHAS, DRISHTI_METHOD,
+    argala_rows, arudha_padas, chara_karakas, co_lord_choice, drishti_table, house_lord,
+    houses_owned, jaimini_lord, jaimini_report, karakamsha, kuja_dosha, moon_is_waxing, nature,
+    normalize, AdvancedSettings, ArgalaStatus, Graha, KarakaRole, KujaDosha, Nature, Positions,
+    Sign, ASPECTING_GRAHAS, DRISHTI_METHOD, JAIMINI_METHOD,
 };
 use serde::de::DeserializeOwned;
 use serde::Deserialize;
@@ -266,6 +267,24 @@ struct ExpectedPada {
     exception: bool,
 }
 
+#[derive(Debug, Deserialize, PartialEq)]
+#[serde(deny_unknown_fields)]
+struct ExpectedArgalaPair {
+    argala_house: u8,
+    virodha_house: u8,
+    argala_grahas: Vec<Graha>,
+    virodha_grahas: Vec<Graha>,
+    status: ArgalaStatus,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ExpectedArgalaRow {
+    reference: Sign,
+    pairs: Vec<ExpectedArgalaPair>,
+    third_house_malefic_argala: bool,
+}
+
 #[derive(Debug, Deserialize)]
 #[serde(deny_unknown_fields)]
 struct JaiminiExpected {
@@ -276,6 +295,8 @@ struct JaiminiExpected {
     co_lords: BTreeMap<Sign, ExpectedCoLord>,
     /// A1…A12; doğum saati bilinmiyorsa `null`.
     arudhas: Option<Vec<ExpectedPada>>,
+    /// En az iki referans burç.
+    argala: Vec<ExpectedArgalaRow>,
 }
 
 #[test]
@@ -363,6 +384,45 @@ fn jaimini_hand_fixtures() {
         if let Some(a) = &x.arudhas {
             assert_eq!(a.len(), 12, "{id}: twelve padas");
         }
+
+        // Argala.
+        assert!(x.argala.len() >= 2, "{id}: at least two references");
+        let rows = argala_rows(&p);
+        assert_eq!(
+            rows[0].reference,
+            p.lagna_sign.unwrap_or(Sign::Aries),
+            "{id}: first argala row"
+        );
+        for want in &x.argala {
+            let got = rows
+                .iter()
+                .find(|r| r.reference == want.reference)
+                .expect("twelve references");
+            let pairs: Vec<ExpectedArgalaPair> = got
+                .pairs
+                .iter()
+                .map(|x| ExpectedArgalaPair {
+                    argala_house: x.argala_house,
+                    virodha_house: x.virodha_house,
+                    argala_grahas: x.argala_grahas.clone(),
+                    virodha_grahas: x.virodha_grahas.clone(),
+                    status: x.status,
+                })
+                .collect();
+            assert_eq!(pairs, want.pairs, "{id}: argala from {:?}", want.reference);
+            assert_eq!(
+                got.third_house_malefic_argala, want.third_house_malefic_argala,
+                "{id}: third house from {:?}",
+                want.reference
+            );
+        }
+
+        // Rapor aynı parçalardan kurulur.
+        let report = jaimini_report(&p, &advanced);
+        assert_eq!(report.method, JAIMINI_METHOD, "{id}");
+        assert_eq!(report.karakas, karakas, "{id}");
+        assert_eq!(report.karakamsha, k, "{id}");
+        assert_eq!(report.argala, rows, "{id}");
     }
 }
 
@@ -419,6 +479,30 @@ fn jaimini_fixtures_cover_the_required_cases() {
             "co-lord step {step}"
         );
     }
+    // Argalanın dört durumu ve 3. ev kötücül argalası.
+    for status in [
+        ArgalaStatus::None,
+        ArgalaStatus::Stands,
+        ArgalaStatus::Contested,
+        ArgalaStatus::Obstructed,
+    ] {
+        assert!(
+            any(&|f, _| f
+                .expected
+                .argala
+                .iter()
+                .any(|r| r.pairs.iter().any(|x| x.status == status))),
+            "argala status {status:?}"
+        );
+    }
+    assert!(
+        any(&|f, _| f
+            .expected
+            .argala
+            .iter()
+            .any(|r| r.third_house_malefic_argala)),
+        "three malefics in the third house"
+    );
 }
 
 /// `lagna` burcundan sayılan `house`. evin burcu (test yardımcısı; motorun `house_sign`'ı
````

- [ ] **Step 3: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-core 2>&1 | grep -E "^error" | sort | uniq -c
```
Expected: derleme hatası: ``unresolved imports `crate::jaimini::jaimini_report`, `crate::jaimini::ArgalaStatus`, `crate::jaimini::JAIMINI_METHOD` `` ve ``unresolved imports `jyotish_core::argala_rows`, `jyotish_core::jaimini_report`, `jyotish_core::ArgalaStatus`, `jyotish_core::JAIMINI_METHOD` ``.

- [ ] **Step 4: `jaimini/argala.rs`**

**Yeni dosya:** `crates/jyotish-core/src/jaimini/argala.rs`

````rust
//! Argala ve virodhargala (ileri modüller spec'i §5).
//!
//! Kaynak: BPHS, argala bölümü; Jaimini Upadesha Sutraları. Referans burçtan 2., 4. ve 11.
//! evdeki grahalar argala verir; 12., 10. ve 3. evdekiler sırasıyla engeller. İkincil çift:
//! 5. ev argala, 9. ev engel. Durum yalnızca sayıya bakar; güç karşılaştırması yapılmaz ve
//! Ketu için ters sayım uygulanmaz. Referanstan 3. evde üç ya da daha çok doğal kötücül varsa
//! bu ayrıca bildirilir. Evler referans burçtan sayılan rasi evleridir (K4).

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::nature::{nature, Nature};
use crate::positions::Positions;
use crate::zodiac::Sign;

/// Argala veren ev ve engelleyeni: 2 ↔ 12, 4 ↔ 10, 11 ↔ 3 ve ikincil 5 ↔ 9.
pub const ARGALA_PAIRS: [(u8, u8); 4] = [(2, 12), (4, 10), (11, 3), (5, 9)];

/// 3. evdeki kötücül argala için gereken en az doğal kötücül sayısı.
pub const THIRD_HOUSE_MALEFICS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum ArgalaStatus {
    /// Argala evinde graha yok.
    None,
    /// Argala tarafında daha çok graha var.
    Stands,
    /// İki tarafta eşit sayıda graha var.
    Contested,
    /// Engel tarafında daha çok graha var.
    Obstructed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ArgalaPair {
    pub argala_house: u8,
    pub virodha_house: u8,
    /// Argala evindeki grahalar, `Graha::ALL` sırasında (dokuz graha da sayılır).
    pub argala_grahas: Vec<Graha>,
    /// Engel evindeki grahalar, `Graha::ALL` sırasında.
    pub virodha_grahas: Vec<Graha>,
    pub status: ArgalaStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ArgalaRow {
    pub reference: Sign,
    /// `ARGALA_PAIRS` sırasında.
    pub pairs: [ArgalaPair; 4],
    pub third_house_malefic_argala: bool,
}

/// Sayıya göre durum.
pub fn argala_status(argala_count: usize, virodha_count: usize) -> ArgalaStatus {
    if argala_count == 0 {
        ArgalaStatus::None
    } else {
        match argala_count.cmp(&virodha_count) {
            std::cmp::Ordering::Greater => ArgalaStatus::Stands,
            std::cmp::Ordering::Equal => ArgalaStatus::Contested,
            std::cmp::Ordering::Less => ArgalaStatus::Obstructed,
        }
    }
}

/// `reference` burcundan sayılan `house`. evdeki grahalar.
fn in_house(positions: &Positions, reference: Sign, house: u8) -> Vec<Graha> {
    positions.occupants(Sign::from_index(reference.index() + usize::from(house) - 1))
}

/// `reference` burcu için dört çift ve 3. ev kötücül argalası.
pub fn argala_row(reference: Sign, positions: &Positions) -> ArgalaRow {
    let pairs = ARGALA_PAIRS.map(|(argala_house, virodha_house)| {
        let argala_grahas = in_house(positions, reference, argala_house);
        let virodha_grahas = in_house(positions, reference, virodha_house);
        ArgalaPair {
            argala_house,
            virodha_house,
            status: argala_status(argala_grahas.len(), virodha_grahas.len()),
            argala_grahas,
            virodha_grahas,
        }
    });
    let third_house_malefics = in_house(positions, reference, 3)
        .into_iter()
        .filter(|&g| nature(g, positions) == Nature::Malefic)
        .count();
    ArgalaRow {
        reference,
        pairs,
        third_house_malefic_argala: third_house_malefics >= THIRD_HOUSE_MALEFICS,
    }
}

/// On iki referans burç için argala: lagna burcundan başlayarak zodyak sırasıyla; doğum saati
/// bilinmiyorsa Koç'tan. Argala lagnaya bağlı değildir; lagna yalnızca satır sırasını belirler.
pub fn argala_rows(positions: &Positions) -> [ArgalaRow; 12] {
    let first = positions.lagna_sign.unwrap_or(Sign::Aries);
    std::array::from_fn(|i| argala_row(Sign::from_index(first.index() + i), positions))
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;
    use Sign::*;

    #[test]
    fn status_looks_at_the_counts_only() {
        assert_eq!(argala_status(0, 0), ArgalaStatus::None);
        assert_eq!(argala_status(0, 3), ArgalaStatus::None);
        assert_eq!(argala_status(1, 0), ArgalaStatus::Stands);
        assert_eq!(argala_status(2, 1), ArgalaStatus::Stands);
        assert_eq!(argala_status(1, 1), ArgalaStatus::Contested);
        assert_eq!(argala_status(1, 2), ArgalaStatus::Obstructed);
    }

    #[test]
    fn four_pairs_for_a_reference_sign() {
        // Güneş Boğa, Ay Yengeç, Mars İkizler, Merkür Boğa, Jüpiter Balık, Venüs Oğlak,
        // Satürn İkizler, Rahu İkizler, Ketu Yay.
        let p = Positions::synthetic(
            None,
            [40.0, 100.0, 65.0, 50.0, 340.0, 280.0, 75.0, 80.0, 260.0],
        );
        let row = argala_row(Aries, &p);
        assert_eq!(row.reference, Aries);
        let houses: Vec<(u8, u8)> = row
            .pairs
            .iter()
            .map(|x| (x.argala_house, x.virodha_house))
            .collect();
        assert_eq!(houses, ARGALA_PAIRS);
        // 2 (Boğa: Güneş, Merkür) ↔ 12 (Balık: Jüpiter): 2 > 1.
        assert_eq!(row.pairs[0].argala_grahas, [Sun, Mercury]);
        assert_eq!(row.pairs[0].virodha_grahas, [Jupiter]);
        assert_eq!(row.pairs[0].status, ArgalaStatus::Stands);
        // 4 (Yengeç: Ay) ↔ 10 (Oğlak: Venüs): eşit.
        assert_eq!(row.pairs[1].argala_grahas, [Moon]);
        assert_eq!(row.pairs[1].virodha_grahas, [Venus]);
        assert_eq!(row.pairs[1].status, ArgalaStatus::Contested);
        // 11 (Kova: boş) ↔ 3 (İkizler: Mars, Satürn, Rahu): argala tarafı boş.
        assert!(row.pairs[2].argala_grahas.is_empty());
        assert_eq!(row.pairs[2].virodha_grahas, [Mars, Saturn, Rahu]);
        assert_eq!(row.pairs[2].status, ArgalaStatus::None);
        // 5 (Aslan: boş) ↔ 9 (Yay: Ketu).
        assert_eq!(row.pairs[3].status, ArgalaStatus::None);
        assert_eq!(row.pairs[3].virodha_grahas, [Ketu]);
        // 3. evde üç doğal kötücül (Mars, Satürn, Rahu).
        assert!(row.third_house_malefic_argala);
        // Boğa'dan 3. ev Yengeç: yalnızca Ay.
        assert!(!argala_row(Taurus, &p).third_house_malefic_argala);
        // Koç'tan 12. evdeki graha sayısı artınca 2 ↔ 12 engellenir.
        let q = Positions::synthetic(
            None,
            [40.0, 345.0, 65.0, 350.0, 340.0, 280.0, 75.0, 80.0, 260.0],
        );
        let row = argala_row(Aries, &q);
        assert_eq!(row.pairs[0].argala_grahas, [Sun]);
        assert_eq!(row.pairs[0].virodha_grahas, [Moon, Mercury, Jupiter]);
        assert_eq!(row.pairs[0].status, ArgalaStatus::Obstructed);
    }

    #[test]
    fn third_house_needs_three_natural_malefics() {
        // İkizler'de Mars, Satürn ve Jüpiter: iki kötücül, bir iyicil.
        let p = Positions::synthetic(
            None,
            [40.0, 100.0, 65.0, 50.0, 70.0, 280.0, 75.0, 200.0, 20.0],
        );
        assert!(!argala_row(Aries, &p).third_house_malefic_argala);
        // Küçülen Ay kötücüldür (K6): Güneş Aslan 10°, Ay İkizler 5° (uzanım 295°).
        let q = Positions::synthetic(
            None,
            [130.0, 65.0, 70.0, 250.0, 340.0, 280.0, 75.0, 200.0, 20.0],
        );
        assert!(argala_row(Aries, &q).third_house_malefic_argala);
        // Aynı yerleşimde büyüyen Ay iyicildir: Güneş Boğa 10°, Ay İkizler 5° (uzanım 25°).
        let r = Positions::synthetic(
            None,
            [40.0, 65.0, 70.0, 250.0, 340.0, 280.0, 75.0, 200.0, 20.0],
        );
        assert!(!argala_row(Aries, &r).third_house_malefic_argala);
    }

    #[test]
    fn rows_start_at_the_lagna_sign_or_at_aries() {
        let lons = [40.0, 100.0, 65.0, 50.0, 340.0, 280.0, 75.0, 80.0, 260.0];
        let with_lagna = argala_rows(&Positions::synthetic(Some(215.0), lons));
        let refs: Vec<Sign> = with_lagna.iter().map(|r| r.reference).collect();
        assert_eq!(refs[0], Scorpio);
        assert_eq!(refs[11], Libra);
        let without = argala_rows(&Positions::synthetic(None, lons));
        let refs: Vec<Sign> = without.iter().map(|r| r.reference).collect();
        assert_eq!(refs, Sign::ALL);
        // Satırların içeriği aynıdır; yalnızca sıra değişir.
        for row in &with_lagna {
            assert_eq!(row, &without[row.reference.index()]);
        }
        // Aynı graha bir çiftin iki tarafında görünmez.
        for row in &without {
            for pair in &row.pairs {
                assert!(pair
                    .argala_grahas
                    .iter()
                    .all(|g| !pair.virodha_grahas.contains(g)));
            }
        }
    }
}
````

- [ ] **Step 5: Rapor ve toplayıcı; dışa aktarımlar**

````diff
diff --git a/crates/jyotish-core/src/jaimini/mod.rs b/crates/jyotish-core/src/jaimini/mod.rs
index 8bdc531..03822fc 100644
--- a/crates/jyotish-core/src/jaimini/mod.rs
+++ b/crates/jyotish-core/src/jaimini/mod.rs
@@ -1,11 +1,28 @@
 //! Jaimini çekirdeği (ileri modüller spec'i §5): chara karaka, karakamsha, rashi drishti, eş
-//! lordlar ve arudha padalar. Hesap fonksiyonları yalnızca `Positions` görür (K3).
+//! lordlar, arudha padalar ve argala. Hesap fonksiyonları yalnızca `Positions` görür (K3);
+//! `compute_jaimini` temel haritayı hesaplayıp onlara veren ince toplayıcıdır. Chara dasha
+//! Alt proje 3.3'tedir ve bu modülün `jaimini_lord` ile `counts_forward` fonksiyonlarını
+//! kullanır.
 
+mod argala;
 mod arudha;
 mod karaka;
 mod lords;
 mod rashi;
 
+use ephemeris::Ephemeris;
+use serde::{Deserialize, Serialize};
+
+use crate::chart::{compute_chart, ChartWarning};
+use crate::drishti::SignAspect;
+use crate::positions::{ChartInput, Positions};
+use crate::settings::{AdvancedSettings, KarakaScheme};
+use crate::CoreError;
+
+pub use argala::{
+    argala_row, argala_rows, argala_status, ArgalaPair, ArgalaRow, ArgalaStatus, ARGALA_PAIRS,
+    THIRD_HOUSE_MALEFICS,
+};
 pub use arudha::{arudha_of, arudha_padas, ArudhaPada};
 pub use karaka::{
     chara_karakas, karaka_roles, karakamsha, navamsa_sign, CharaKaraka, KarakaRole, Karakamsha,
@@ -16,3 +33,180 @@ pub use rashi::{
     counts_forward, directed_count, graha_rashi_drishti, rashi_drishti_matrix, rashi_drishti_signs,
     sign_kind, SignKind,
 };
+
+/// `JaiminiReport::method`: bu modüldeki yöntemin kimliği.
+pub const JAIMINI_METHOD: &str = "jaimini.core_v1";
+
+/// `compute_jaimini` sonucu. AI yorum paneli için sözleşmedir: alan adları değişirse AI
+/// paketleyicisi de değişir.
+#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
+#[cfg_attr(feature = "specta", derive(specta::Type))]
+pub struct JaiminiReport {
+    /// Kullanılan karaka şeması (`settings.advanced.karaka_scheme`).
+    pub scheme: KarakaScheme,
+    /// Eş lordlar kullanıldı mı (`settings.advanced.jaimini_co_lords`).
+    pub co_lords: bool,
+    pub method: String,
+    /// 8 (`Eight`) ya da 7 (`Seven`) kayıt, rol sırasında.
+    pub karakas: Vec<CharaKaraka>,
+    pub karakamsha: Karakamsha,
+    /// `rashi_drishti[from][to]`, `Sign::ALL` sırasında.
+    pub rashi_drishti: [[bool; 12]; 12],
+    /// 27 kayıt: `Graha::ALL` sırasında, her graha için üç hedef burç ve oradaki grahalar.
+    pub graha_rashi_drishti: Vec<SignAspect>,
+    /// A1…A12; doğum saati bilinmiyorsa `None`.
+    pub arudhas: Option<[ArudhaPada; 12]>,
+    /// On iki referans burç: lagna burcundan sıralı; doğum saati bilinmiyorsa Koç'tan.
+    pub argala: [ArgalaRow; 12],
+    pub warnings: Vec<ChartWarning>,
+}
+
+/// Saf hesap: konumlardan ve iki ayardan rapor. `warnings` boştur; toplayıcı doldurur.
+pub fn jaimini_report(positions: &Positions, advanced: &AdvancedSettings) -> JaiminiReport {
+    let karakas = chara_karakas(positions, advanced.karaka_scheme);
+    // Sıra hiçbir şemada boş değildir: ilk kayıt Atmakaraka'dır.
+    let atmakaraka = karakas[0].graha;
+    JaiminiReport {
+        scheme: advanced.karaka_scheme,
+        co_lords: advanced.jaimini_co_lords,
+        method: JAIMINI_METHOD.to_owned(),
+        karakamsha: karakamsha(positions, atmakaraka),
+        karakas,
+        rashi_drishti: rashi_drishti_matrix(),
+        graha_rashi_drishti: graha_rashi_drishti(positions),
+        arudhas: arudha_padas(positions, advanced.jaimini_co_lords),
+        argala: argala_rows(positions),
+        warnings: Vec::new(),
+    }
+}
+
+/// `compute_jaimini` komutunun hesabı: temel harita, konumlar, rapor. `input.time_known`
+/// yanlışsa lagna boşaltılır: padalar `None` olur ve argala satırları Koç'tan sıralanır.
+/// Temel haritanın konum duyarlılığı uyarısı (`LowPrecision`) rapora taşınır; diğer harita
+/// uyarıları (ev sistemi, gün doğumu) bu modülü ilgilendirmez.
+pub fn compute_jaimini<E: Ephemeris + ?Sized>(
+    eph: &mut E,
+    input: &ChartInput,
+) -> Result<JaiminiReport, CoreError> {
+    let chart = compute_chart(eph, input.jd_ut, input.lat, input.lon, &input.settings)?;
+    let positions = Positions::from(&chart);
+    let positions = if input.time_known {
+        positions
+    } else {
+        positions.without_lagna()
+    };
+    let mut report = jaimini_report(&positions, &input.settings.advanced);
+    report.warnings = chart
+        .meta
+        .warnings
+        .iter()
+        .copied()
+        .filter(|w| *w == ChartWarning::LowPrecision)
+        .collect();
+    Ok(report)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::fake::FakeEph;
+    use crate::graha::Graha;
+    use crate::settings::CalcSettings;
+    use crate::zodiac::Sign;
+
+    fn input(advanced: AdvancedSettings, time_known: bool) -> ChartInput {
+        ChartInput {
+            jd_ut: 2_451_545.0,
+            lat: 41.0,
+            lon: 29.0,
+            settings: CalcSettings {
+                advanced,
+                ..CalcSettings::default()
+            },
+            time_known,
+        }
+    }
+
+    #[test]
+    fn report_follows_the_two_advanced_settings() {
+        // Lagna Koç; Güneş Aslan 12°, Ay Boğa 7°, Mars Yengeç 22°, Merkür Başak 3°, Jüpiter
+        // Yay 16°, Venüs Balık 25°, Satürn Terazi 9°, Rahu İkizler 2° (etkin 28°), Ketu Yay 2°.
+        let p = Positions::synthetic(
+            Some(5.0),
+            [132.0, 37.0, 112.0, 153.0, 256.0, 355.0, 189.0, 62.0, 242.0],
+        );
+        let eight = jaimini_report(&p, &AdvancedSettings::default());
+        assert_eq!(eight.method, JAIMINI_METHOD);
+        assert_eq!((eight.scheme, eight.co_lords), (KarakaScheme::Eight, true));
+        assert_eq!(eight.karakas.len(), 8);
+        assert_eq!(eight.karakas[0].graha, Graha::Rahu);
+        assert_eq!(eight.karakamsha.atmakaraka, Graha::Rahu);
+        assert_eq!(eight.rashi_drishti, rashi_drishti_matrix());
+        assert_eq!(eight.graha_rashi_drishti.len(), 27);
+        assert_eq!(eight.argala[0].reference, Sign::Aries);
+        assert!(eight.warnings.is_empty());
+        // 8. ev Akrep: Mars Yengeç'te yalnız, Ketu Yay'da Jüpiter'le: Ketu.
+        assert_eq!(eight.arudhas.expect("lagna")[7].lord, Graha::Ketu);
+
+        let seven = jaimini_report(
+            &p,
+            &AdvancedSettings {
+                karaka_scheme: KarakaScheme::Seven,
+                jaimini_co_lords: false,
+            },
+        );
+        assert_eq!((seven.scheme, seven.co_lords), (KarakaScheme::Seven, false));
+        assert_eq!(seven.karakas.len(), 7);
+        // Rahu sıralanmaz: en yüksek derece Venüs 25°.
+        assert_eq!(seven.karakamsha.atmakaraka, Graha::Venus);
+        assert_eq!(seven.arudhas.expect("lagna")[7].lord, Graha::Mars);
+        // Argala ve rashi drishti ayarlardan bağımsızdır.
+        assert_eq!(seven.argala, eight.argala);
+        assert_eq!(seven.graha_rashi_drishti, eight.graha_rashi_drishti);
+    }
+
+    #[test]
+    fn collector_hides_lagna_dependent_values_when_time_is_unknown() {
+        let mut eph = FakeEph::default();
+        let known = compute_jaimini(&mut eph, &input(AdvancedSettings::default(), true)).unwrap();
+        let chart =
+            compute_chart(&mut eph, 2_451_545.0, 41.0, 29.0, &CalcSettings::default()).unwrap();
+        let positions = Positions::from(&chart);
+        let mut want = jaimini_report(&positions, &AdvancedSettings::default());
+        want.warnings = known.warnings.clone();
+        assert_eq!(known, want);
+        assert!(known.arudhas.is_some());
+        assert_eq!(known.argala[0].reference, chart.lagna.sign);
+
+        let unknown =
+            compute_jaimini(&mut eph, &input(AdvancedSettings::default(), false)).unwrap();
+        assert_eq!(unknown.arudhas, None);
+        assert_eq!(unknown.argala[0].reference, Sign::Aries);
+        // Lagnaya bağlı olmayan her şey aynıdır.
+        assert_eq!(unknown.karakas, known.karakas);
+        assert_eq!(unknown.karakamsha, known.karakamsha);
+        assert_eq!(unknown.graha_rashi_drishti, known.graha_rashi_drishti);
+        for row in &unknown.argala {
+            let same = known
+                .argala
+                .iter()
+                .find(|r| r.reference == row.reference)
+                .expect("row");
+            assert_eq!(row, same);
+        }
+    }
+
+    #[test]
+    fn collector_reports_the_chart_errors() {
+        let mut eph = FakeEph::default();
+        let mut bad = input(AdvancedSettings::default(), true);
+        bad.lat = 91.0;
+        assert_eq!(
+            compute_jaimini(&mut eph, &bad),
+            Err(CoreError::InvalidLocation {
+                lat: 91.0,
+                lon: 29.0
+            })
+        );
+    }
+}
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 319c452..9512cef 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -48,11 +48,13 @@ pub use error::CoreError;
 pub use graha::{Graha, GrahaPosition};
 pub use houses::{house_from_cusps, whole_sign_cusps, whole_sign_house, Bhavas, Lagna};
 pub use jaimini::{
-    arudha_of, arudha_padas, chara_karakas, co_lord_choice, co_lords, counts_forward,
-    directed_count, graha_rashi_drishti, jaimini_lord, karaka_roles, karakamsha, navamsa_sign,
-    rashi_drishti_matrix, rashi_drishti_signs, sign_kind, stronger_co_lord, ArudhaPada,
-    CharaKaraka, CoLordChoice, KarakaRole, Karakamsha, SignKind, EIGHT_KARAKA_ROLES,
-    KARAKA_TIE_DEG, SEVEN_KARAKA_ROLES,
+    argala_row, argala_rows, argala_status, arudha_of, arudha_padas, chara_karakas, co_lord_choice,
+    co_lords, compute_jaimini, counts_forward, directed_count, graha_rashi_drishti, jaimini_lord,
+    jaimini_report, karaka_roles, karakamsha, navamsa_sign, rashi_drishti_matrix,
+    rashi_drishti_signs, sign_kind, stronger_co_lord, ArgalaPair, ArgalaRow, ArgalaStatus,
+    ArudhaPada, CharaKaraka, CoLordChoice, JaiminiReport, KarakaRole, Karakamsha, SignKind,
+    ARGALA_PAIRS, EIGHT_KARAKA_ROLES, JAIMINI_METHOD, KARAKA_TIE_DEG, SEVEN_KARAKA_ROLES,
+    THIRD_HOUSE_MALEFICS,
 };
 pub use lordship::{house_lord, house_sign, houses_owned};
 pub use nature::{kuja_dosha, moon_is_waxing, nature, KujaDosha, Nature, KUJA_DOSHA_HOUSES};
````

- [ ] **Step 6: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p jyotish-core 2>&1 | grep -E "^test result|Running"
grep -rnE 'serde\(default' crates/jyotish-core/src/jaimini
```
Expected: fmt ve clippy çıktısız; lib `172 passed` (+8: 4 `argala`, 3 `jaimini::tests`, 1 proptest); golden `8 passed`; `modules_golden` `5 passed`; son `grep` çıktısız (IPC tiplerinde `serde(default)` yoktur: Plan 14 D2).

- [ ] **Step 7: Commit**

```bash
git add crates/jyotish-core/src crates/jyotish-core/tests/modules_golden.rs tests/golden/modules
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): argala and virodhargala for twelve reference signs; JaiminiReport and the compute_jaimini collector"
```
Expected: iki `grep` çıktısız.

---


### Task 4: Uygulama: `compute_jaimini` komutu, bağlamalar, sarmalayıcı ve fikstür

Spec §5 "Veri modeli ve IPC", K1, K8; §5 "Test: uçtan uca"; §13 "Komut katmanı"; §15 bütçesi. Komut Plan 14'ün ortak kalıbını izler: tek `ChartInput`, motor iş parçacığı, `blocking` havuzu, `CommandError`.

**Files:**
- Modify: `app/src-tauri/src/{module_commands,lib}.rs`, `app/src-tauri/tests/{modules,ui_fixture}.rs`, `app/ui/api.ts`, `app/ui/api.test.ts`, `app/ui/test/fixture.ts`, `app/e2e/ipc.ts`
- Üretilir: `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`
- Test: `app/src-tauri/tests/modules.rs`, `app/ui/api.test.ts`

**Interfaces:**
- Consumes: `jyotish_core::{compute_jaimini, jaimini_report, JaiminiReport, ChartInput, Positions, co_lords, house_lord, house_sign, rashi_drishti_signs, Varga}` (Task 1–3); `ephemeris::{Engine, SwissEph}`; `client_commands::blocking`; `commands::{CommandError, compute_chart_blocking}`; `module_commands::compute_drishti_blocking` kalıbı.
- Produces:
  - `module_commands::compute_jaimini_blocking(engine: &Engine, input: ChartInput) -> Result<JaiminiReport, CommandError>`
  - Tauri komutu `compute_jaimini(input: ChartInput) -> Result<JaiminiReport, CommandError>`
  - TS: `commands.computeJaimini(input)`, `api.computeJaimini(input: ChartInput)`; tipler `JaiminiReport`, `CharaKaraka`, `KarakaRole`, `Karakamsha`, `ArudhaPada`, `ArgalaRow`, `ArgalaPair`, `ArgalaStatus`; `UiFixture.jaimini: JaiminiReport`; e2e taklidinde `compute_jaimini` (`time_known` yanlışsa `arudhas: null`).

- [ ] **Step 1: Başarısız entegrasyon testini yaz**

`tests/modules.rs` tek test olarak durur (süreçte tek SE örneği açılabilir); adı `module_commands_run_on_the_engine_thread_and_map_errors` olur, Plan 14'ün bakış denetimleri `drishti_checks`'e taşınır ve `jaimini_checks` eklenir. Beş golden anında: komut hata vermez; rapor, v1'de golden fixture'larla doğrulanmış haritanın konumlarından `jaimini_report` ile kurulan raporla birebir aynıdır; karakamsha haritanın D9'undaki Atmakaraka burcudur ve swamsha evleri haritanın D9 yerleşimlerinden sayılır (çapraz denetim, §13 katman 4); spec'in değişmezleri sağlanır (permütasyon, simetri ve üç bakış, pada evin kendisinde ya da 7. burcunda değil, argalada tekrar yok); saat bilinmeyince padalar `None` ve ilk argala satırı Koç'tur; yedili şema ve devre dışı eş lordlar hesaba yansır; hatalar `compute_chart`'ın kodlarına eşlenir; ortanca süre ≤ 10 ms.

````diff
diff --git a/app/src-tauri/tests/modules.rs b/app/src-tauri/tests/modules.rs
index 3816490..5176d03 100644
--- a/app/src-tauri/tests/modules.rs
+++ b/app/src-tauri/tests/modules.rs
@@ -6,11 +6,12 @@ use std::time::{Duration, Instant};
 
 use ephemeris::Engine;
 use jyotish_core::{
-    graha_aspect_signs, sphuta_drishti, CalcSettings, ChartInput, Graha, ASPECTING_GRAHAS,
-    DRISHTI_METHOD,
+    co_lords, graha_aspect_signs, house_lord, house_sign, jaimini_report, rashi_drishti_signs,
+    sphuta_drishti, AdvancedSettings, CalcSettings, ChartInput, Graha, KarakaScheme, Positions,
+    Sign, Varga, ASPECTING_GRAHAS, DRISHTI_METHOD, JAIMINI_METHOD,
 };
 use jyotish_desk_lib::commands::compute_chart_blocking;
-use jyotish_desk_lib::module_commands::compute_drishti_blocking;
+use jyotish_desk_lib::module_commands::{compute_drishti_blocking, compute_jaimini_blocking};
 use serde_json::Value;
 
 fn root() -> PathBuf {
@@ -43,21 +44,26 @@ fn golden_inputs() -> Vec<(String, ChartInput)> {
     out
 }
 
+/// Motoru tek test açar; modül başına denetimler ayrı fonksiyonlardadır.
 #[test]
-fn compute_drishti_runs_on_the_engine_thread_and_maps_errors() {
+fn module_commands_run_on_the_engine_thread_and_map_errors() {
     let engine = Engine::spawn(root().join("../../data/ephe")).expect("spawn engine");
     let inputs = golden_inputs();
     assert!(inputs.len() >= 5, "{} golden moments", inputs.len());
+    drishti_checks(&engine, &inputs);
+    jaimini_checks(&engine, &inputs);
+}
 
-    for (id, input) in &inputs {
-        let table = compute_drishti_blocking(&engine, input.clone())
+fn drishti_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
+    for (id, input) in inputs {
+        let table = compute_drishti_blocking(engine, input.clone())
             .unwrap_or_else(|e| panic!("{id}: {}", e.code));
         assert_eq!(table.method, DRISHTI_METHOD, "{id}");
         assert_eq!(table.sign_aspects.len(), 13, "{id}");
         // Çapraz denetim: v1'de golden fixture'larla doğrulanmış haritanın konumlarından
         // aynı tablo çıkar.
         let chart = compute_chart_blocking(
-            &engine,
+            engine,
             input.jd_ut,
             input.lat,
             input.lon,
@@ -98,7 +104,7 @@ fn compute_drishti_runs_on_the_engine_thread_and_maps_errors() {
             ..input.clone()
         };
         assert_eq!(
-            compute_drishti_blocking(&engine, unknown).expect("time unknown"),
+            compute_drishti_blocking(engine, unknown).expect("time unknown"),
             table,
             "{id}"
         );
@@ -116,7 +122,7 @@ fn compute_drishti_runs_on_the_engine_thread_and_maps_errors() {
         ..delhi.clone()
     };
     assert_eq!(
-        compute_drishti_blocking(&engine, bad_place)
+        compute_drishti_blocking(engine, bad_place)
             .unwrap_err()
             .code,
         "invalid_location"
@@ -128,7 +134,7 @@ fn compute_drishti_runs_on_the_engine_thread_and_maps_errors() {
         },
         ..delhi.clone()
     };
-    let err = compute_drishti_blocking(&engine, old_settings).unwrap_err();
+    let err = compute_drishti_blocking(engine, old_settings).unwrap_err();
     assert_eq!(err.code, "unsupported_schema_version");
     assert_eq!(err.params["found"], "1");
     let bad_moment = ChartInput {
@@ -136,7 +142,7 @@ fn compute_drishti_runs_on_the_engine_thread_and_maps_errors() {
         ..delhi.clone()
     };
     assert_eq!(
-        compute_drishti_blocking(&engine, bad_moment)
+        compute_drishti_blocking(engine, bad_moment)
             .unwrap_err()
             .code,
         "invalid_julian_day"
@@ -144,11 +150,11 @@ fn compute_drishti_runs_on_the_engine_thread_and_maps_errors() {
 
     // Performans: spec §15 "compute_drishti ≤ 10 ms (temel harita hesabı dahil)". Ortanca,
     // ısınmadan sonra (open_client ölçümüyle aynı düzenek).
-    compute_drishti_blocking(&engine, delhi.clone()).expect("warm-up");
+    compute_drishti_blocking(engine, delhi.clone()).expect("warm-up");
     let mut times: Vec<Duration> = (0..15)
         .map(|_| {
             let t = Instant::now();
-            compute_drishti_blocking(&engine, delhi.clone()).expect("drishti");
+            compute_drishti_blocking(engine, delhi.clone()).expect("drishti");
             t.elapsed()
         })
         .collect();
@@ -163,3 +169,191 @@ fn compute_drishti_runs_on_the_engine_thread_and_maps_errors() {
         "compute_drishti median {median:?}"
     );
 }
+
+/// Isınmadan sonra on beş koşunun ortancası ve en büyüğü (`open_client` ölçümüyle aynı düzenek).
+fn median_of_15(mut run: impl FnMut()) -> (Duration, Duration) {
+    run();
+    let mut times: Vec<Duration> = (0..15)
+        .map(|_| {
+            let t = Instant::now();
+            run();
+            t.elapsed()
+        })
+        .collect();
+    times.sort();
+    (times[times.len() / 2], times[times.len() - 1])
+}
+
+fn jaimini_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
+    for (id, input) in inputs {
+        let report = compute_jaimini_blocking(engine, input.clone())
+            .unwrap_or_else(|e| panic!("{id}: {}", e.code));
+        assert_eq!(report.method, JAIMINI_METHOD, "{id}");
+        assert_eq!(
+            (report.scheme, report.co_lords),
+            (KarakaScheme::Eight, true),
+            "{id}: fixture settings are the defaults"
+        );
+        let chart = compute_chart_blocking(
+            engine,
+            input.jd_ut,
+            input.lat,
+            input.lon,
+            input.settings.clone(),
+        )
+        .expect("chart");
+        // Çapraz denetim: v1'de golden fixture'larla doğrulanmış haritanın konumlarından
+        // aynı rapor çıkar.
+        let positions = Positions::from(&chart);
+        let mut want = jaimini_report(&positions, &input.settings.advanced);
+        want.warnings = report.warnings.clone();
+        assert_eq!(report, want, "{id}");
+
+        // Karaka sırası sekiz grahanın permütasyonudur; Ketu yoktur.
+        let mut grahas: Vec<Graha> = report.karakas.iter().map(|k| k.graha).collect();
+        grahas.sort();
+        assert_eq!(grahas[..], Graha::ALL[..8], "{id}");
+        for w in report.karakas.windows(2) {
+            assert!(w[0].effective_deg >= w[1].effective_deg, "{id}");
+        }
+        // Karakamsha haritanın D9'undaki Atmakaraka burcudur; swamsha evleri D9'dan sayılır.
+        let d9 = chart
+            .vargas
+            .iter()
+            .find(|v| v.varga == Varga::D9)
+            .expect("D9");
+        let d9_sign = |g: Graha| d9.placements[g as usize].sign;
+        let ak = report.karakamsha.atmakaraka;
+        assert_eq!(ak, report.karakas[0].graha, "{id}");
+        assert_eq!(report.karakamsha.sign, d9_sign(ak), "{id}");
+        for g in Graha::ALL {
+            let i = g as usize;
+            assert_eq!(
+                report.karakamsha.houses_in_d9[i],
+                d9_sign(g).house_from(report.karakamsha.sign),
+                "{id}: swamsha house of {g:?}"
+            );
+            assert_eq!(
+                report.karakamsha.houses_in_d1[i],
+                chart.grahas[i].sign.house_from(report.karakamsha.sign),
+                "{id}: D1 house of {g:?}"
+            );
+        }
+        // Rashi drishti simetriktir ve her burç tam üç burca bakar.
+        for from in Sign::ALL {
+            let row = report.rashi_drishti[from.index()];
+            assert_eq!(row.iter().filter(|&&b| b).count(), 3, "{id}: {from:?}");
+            for to in Sign::ALL {
+                assert_eq!(
+                    row[to.index()],
+                    report.rashi_drishti[to.index()][from.index()],
+                    "{id}"
+                );
+            }
+        }
+        assert_eq!(report.graha_rashi_drishti.len(), 27, "{id}");
+        for a in &report.graha_rashi_drishti {
+            let from = chart.grahas[a.graha as usize].sign;
+            assert!(rashi_drishti_signs(from).contains(&a.target), "{id}");
+        }
+        // Hiçbir pada evin kendi burcuna ya da 7. burcuna düşmez; lord evin lordudur ya da
+        // (Akrep ve Kova'da) eş lordu.
+        let lagna = chart.lagna.sign;
+        let padas = report.arudhas.expect("padas with a known birth time");
+        for pada in &padas {
+            let sign = house_sign(lagna, pada.house);
+            assert!(
+                !matches!(pada.sign.house_from(sign), 1 | 7),
+                "{id}: A{}",
+                pada.house
+            );
+            let allowed = match co_lords(sign) {
+                Some((planet, node)) => vec![planet, node],
+                None => vec![house_lord(lagna, pada.house)],
+            };
+            assert!(allowed.contains(&pada.lord), "{id}: A{} lord", pada.house);
+        }
+        // Argala lagna burcundan sıralanır; aynı graha bir çiftin iki tarafında görünmez.
+        assert_eq!(report.argala[0].reference, lagna, "{id}");
+        for row in &report.argala {
+            for pair in &row.pairs {
+                assert!(
+                    pair.argala_grahas
+                        .iter()
+                        .all(|g| !pair.virodha_grahas.contains(g)),
+                    "{id}"
+                );
+            }
+        }
+
+        // Saat bilinmiyorsa padalar Rust'ta boş döner (K2) ve argala Koç'tan sıralanır.
+        let unknown = compute_jaimini_blocking(
+            engine,
+            ChartInput {
+                time_known: false,
+                ..input.clone()
+            },
+        )
+        .expect("time unknown");
+        assert_eq!(unknown.arudhas, None, "{id}");
+        assert_eq!(unknown.argala[0].reference, Sign::Aries, "{id}");
+        assert_eq!(unknown.karakas, report.karakas, "{id}");
+        assert_eq!(unknown.karakamsha, report.karakamsha, "{id}");
+
+        // İki ayar hesapta kullanılır: yedili şemada Rahu sıralanmaz; eş lordlar devre
+        // dışıyken Akrep ve Kova evlerinin lordu Mars ve Satürn'dür.
+        let mut plain = input.clone();
+        plain.settings.advanced = AdvancedSettings {
+            karaka_scheme: KarakaScheme::Seven,
+            jaimini_co_lords: false,
+        };
+        let plain = compute_jaimini_blocking(engine, plain).expect("seven scheme");
+        assert_eq!((plain.scheme, plain.co_lords), (KarakaScheme::Seven, false));
+        assert_eq!(plain.karakas.len(), 7, "{id}");
+        assert!(plain.karakas.iter().all(|k| !k.graha.is_node()), "{id}");
+        for pada in plain.arudhas.expect("padas") {
+            assert_eq!(pada.lord, house_lord(lagna, pada.house), "{id}");
+        }
+    }
+
+    // Hatalar `compute_chart` ile aynı kodlara eşlenir.
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
+        compute_jaimini_blocking(engine, bad_place)
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
+        compute_jaimini_blocking(engine, old_settings)
+            .unwrap_err()
+            .code,
+        "unsupported_schema_version"
+    );
+
+    // Performans: spec §15 "compute_jaimini ≤ 10 ms (temel harita hesabı dahil)".
+    let (median, max) = median_of_15(|| {
+        compute_jaimini_blocking(engine, delhi.clone()).expect("jaimini");
+    });
+    eprintln!("compute_jaimini median {median:?}, max {max:?}");
+    assert!(
+        median <= Duration::from_millis(10),
+        "compute_jaimini median {median:?}"
+    );
+}
diff --git a/app/src-tauri/tests/ui_fixture.rs b/app/src-tauri/tests/ui_fixture.rs
index 8cf0902..bcc0f59 100644
--- a/app/src-tauri/tests/ui_fixture.rs
+++ b/app/src-tauri/tests/ui_fixture.rs
@@ -15,7 +15,7 @@ use jyotish_core::ChartInput;
 use jyotish_desk_lib::atlas_commands::{preview_moment_body, search_places_blocking, AtlasState};
 use jyotish_desk_lib::client_commands::create_client_blocking;
 use jyotish_desk_lib::commands::dasha_children;
-use jyotish_desk_lib::module_commands::compute_drishti_blocking;
+use jyotish_desk_lib::module_commands::{compute_drishti_blocking, compute_jaimini_blocking};
 use jyotish_desk_lib::transit_commands::{transits_at_blocking, TransitAt};
 use serde_json::{json, Value};
 use store::{BirthData, ChartKind, NewChart, NewClient, Place, Store, TimeCertainty, TzChoice};
@@ -95,18 +95,16 @@ fn ui_fixture_is_up_to_date() {
         opened.chart.dasha.seed,
     )
     .expect("transit");
-    // Bakış tablosu: arayüzün `chartInput(opened)` ile kurduğu girdinin aynısı.
-    let drishti = compute_drishti_blocking(
-        &engine,
-        ChartInput {
-            jd_ut: opened.chart.jd_ut,
-            lat: opened.chart.geo_lat,
-            lon: opened.chart.geo_lon,
-            settings: opened.chart.meta.settings.clone(),
-            time_known: true,
-        },
-    )
-    .expect("drishti");
+    // Modül sonuçları: arayüzün `chartInput(opened)` ile kurduğu girdinin aynısı.
+    let module_input = ChartInput {
+        jd_ut: opened.chart.jd_ut,
+        lat: opened.chart.geo_lat,
+        lon: opened.chart.geo_lon,
+        settings: opened.chart.meta.settings.clone(),
+        time_known: true,
+    };
+    let drishti = compute_drishti_blocking(&engine, module_input.clone()).expect("drishti");
+    let jaimini = compute_jaimini_blocking(&engine, module_input).expect("jaimini");
     // Kaydedilen doğum verisi girdinin kanonik biçimidir (Plan 10 kapsam dışı 21).
     assert_eq!(opened.chart_record.birth, delhi());
     let prefs = store
@@ -123,6 +121,7 @@ fn ui_fixture_is_up_to_date() {
         "transit": transit,
         "prefs": prefs,
         "drishti": drishti,
+        "jaimini": jaimini,
     });
     let text = serde_json::to_string(&value)
         .expect("serialize")
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index d315fff..26202d6 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -29,6 +29,7 @@ vi.mock("./bindings", () => ({
     importClientsJson: vi.fn(),
     dashaChildren: vi.fn(),
     computeDrishti: vi.fn(),
+    computeJaimini: vi.fn(),
     searchPlaces: vi.fn(),
     previewMoment: vi.fn(),
     getAppPrefs: vi.fn(),
@@ -109,6 +110,7 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["importClientsJson", []],
     ["dashaChildren", [{ birth_jd_ut: 1 }, ["Sun"]]],
     ["computeDrishti", [{ jd_ut: 1, time_known: true }]],
+    ["computeJaimini", [{ jd_ut: 1, time_known: false }]],
     ["searchPlaces", ["delhi", 8]],
     ["previewMoment", [{ date: "2000-01-01" }]],
     ["getAppPrefs", []],
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-desk --test modules 2>&1 | grep -E "^error" | sort | uniq -c
pnpm --dir app exec vitest run ui/api.test.ts 2>&1 | grep -E "FAIL|Tests "
```
Expected: ``error[E0432]: unresolved import `jyotish_desk_lib::module_commands::compute_jaimini_blocking` ``; Vitest `FAIL  ui/api.test.ts > every wrapper forwards its arguments to the command of the same name`, `1 failed | 6 passed (7)`.

- [ ] **Step 3: `module_commands.rs` ve komut kaydı**

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index e3b9687..a104093 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -30,6 +30,7 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             commands::dasha_children,
             commands::print_window,
             module_commands::compute_drishti,
+            module_commands::compute_jaimini,
             client_commands::open_client,
             client_commands::restore_session,
             client_commands::create_client,
diff --git a/app/src-tauri/src/module_commands.rs b/app/src-tauri/src/module_commands.rs
index 3e97346..3c3297f 100644
--- a/app/src-tauri/src/module_commands.rs
+++ b/app/src-tauri/src/module_commands.rs
@@ -6,7 +6,7 @@
 use std::sync::Arc;
 
 use ephemeris::{Engine, SwissEph};
-use jyotish_core::{ChartInput, DrishtiTable};
+use jyotish_core::{ChartInput, DrishtiTable, JaiminiReport};
 use tauri::State;
 
 use crate::client_commands::blocking;
@@ -33,3 +33,26 @@ pub async fn compute_drishti(
     let engine = Arc::clone(&engine);
     blocking(move || compute_drishti_blocking(&engine, input)).await
 }
+
+/// `compute_jaimini` komutunun gövdesi; bloklar.
+pub fn compute_jaimini_blocking(
+    engine: &Engine,
+    input: ChartInput,
+) -> Result<JaiminiReport, CommandError> {
+    let report =
+        engine.call(move |se: &mut SwissEph| jyotish_core::compute_jaimini(se, &input))??;
+    Ok(report)
+}
+
+/// Jaimini çekirdeği: chara karaka, karakamsha, rashi drishti, arudha padalar ve argala.
+/// Karaka şeması ve eş lord kuralı `input.settings.advanced`'ten okunur. Doğum saati
+/// bilinmiyorsa (`time_known == false`) padalar `null` döner.
+#[tauri::command]
+#[specta::specta]
+pub async fn compute_jaimini(
+    engine: State<'_, Arc<Engine>>,
+    input: ChartInput,
+) -> Result<JaiminiReport, CommandError> {
+    let engine = Arc::clone(&engine);
+    blocking(move || compute_jaimini_blocking(&engine, input)).await
+}
````

- [ ] **Step 4: Bağlamaları ve fikstürü üret; arayüz sarmalayıcısı ve e2e taklidi**

````diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index f4b4868..ee67cfe 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -95,6 +95,9 @@ export async function mockIpc(page: Page, { session, unknownTime = false }: IpcO
         transits_at: () => fixture.transit,
         // Bakış tablosu yeniden hesaplanmaz: fikstürdeki gerçek sonuç döner.
         compute_drishti: () => fixture.drishti,
+        // Jaimini raporu: fikstürdeki gerçek sonuç; saat bilinmiyorsa Rust padaları boş döner.
+        compute_jaimini: (args) =>
+          (args.input as { time_known: boolean }).time_known ? fixture.jaimini : { ...fixture.jaimini, arudhas: null },
         print_window: () => null,
         // Güncelleme: sahte yayın; ağ yoktur. Kurulum yalnızca denetimin bildirdiği sürümü kabul eder.
         app_info: () => ({ version: "0.9.0", updaterConfigured: true }),
diff --git a/app/ui/api.ts b/app/ui/api.ts
index ba1f93c..65936b9 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -79,6 +79,7 @@ export const api = {
   // İleri modüller: sonuç saklanmaz, panel açıkken istenir (ileri modüller spec'i K1). Girdi
   // `modules/input.ts`'teki `chartInput(opened)` ile kurulur.
   computeDrishti: (input: ChartInput) => wrap(() => commands.computeDrishti(input)),
+  computeJaimini: (input: ChartInput) => wrap(() => commands.computeJaimini(input)),
   searchPlaces: (query: string, limit: number) => wrap(() => commands.searchPlaces(query, limit)),
   previewMoment: (birth: BirthData) => wrap(() => commands.previewMoment(birth)),
   appInfo: () => wrap(() => commands.appInfo()),
diff --git a/app/ui/test/fixture.ts b/app/ui/test/fixture.ts
index bb1dc51..86032e5 100644
--- a/app/ui/test/fixture.ts
+++ b/app/ui/test/fixture.ts
@@ -1,7 +1,7 @@
 // Gerçek veri fikstürü (app/src-tauri/tests/ui_fixture.rs üretir): Delhi 2000-01-01 17:30
 // için `create_client` dönüşü, ilk 3. seviye dönemin alt dönemleri, "new delhi" atlas araması,
-// an önizlemesi, bir DST boşluğu hatası, aynı sabit saatte transit ve bakış tablosu. Arayüz hesap yapmadığı için testler bunu çizer.
-import type { CommandError, DashaPeriod, DrishtiTable, Graha, MomentPreview, OpenedClient, PlaceHit, PrefsLoad, TransitView } from "../bindings";
+// an önizlemesi, bir DST boşluğu hatası, aynı sabit saatte transit, bakış tablosu ve Jaimini raporu. Arayüz hesap yapmadığı için testler bunu çizer.
+import type { CommandError, DashaPeriod, DrishtiTable, Graha, JaiminiReport, MomentPreview, OpenedClient, PlaceHit, PrefsLoad, TransitView } from "../bindings";
 import raw from "./fixtures/delhi.json?raw";
 
 export interface UiFixture {
@@ -16,6 +16,8 @@ export interface UiFixture {
   prefs: PrefsLoad;
   /** `compute_drishti`, açık haritanın girdisiyle (`time_known: true`). */
   drishti: DrishtiTable;
+  /** `compute_jaimini`, aynı girdiyle (varsayılan ayarlar: sekizli şema, eş lordlar açık). */
+  jaimini: JaiminiReport;
 }
 
 const FIXTURE = JSON.parse(raw) as UiFixture;
````

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
git diff --stat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
grep -c "computeJaimini" app/ui/bindings.ts
```
Expected: iki `ok`; `bindings.ts` 95 satır eklenir (`computeJaimini` ve sekiz tip); `delhi.json` tek satır değişir (`"jaimini"` anahtarı eklenir); son komut `1` yazar. Fikstürdeki rapor (Delhi, İkizler lagna, varsayılan ayarlar) sonraki görevlerin panel testlerinin beklediği değerleri taşır: Atmakaraka Ay (19°28′), Amatya Rahu (gerçek 11°11′, etkin 18°48′), karakamsha Balık, A1 Balık (istisna), A12 Kova (istisna), 6. ev Akrep'in lordu Mars, 9. ev Kova'nın lordu Satürn.

- [ ] **Step 5: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test -p jyotish-desk --test modules -- --nocapture 2>&1 | grep median
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: fmt ve clippy çıktısız; `jyotish-core` lib `172 passed`; `jyotish-desk` `modules` `1 passed`, `bindings` `1 passed`, `ui_fixture` `2 passed`; diğer Rust satırları tabandaki gibi; iki `median` satırı (`compute_drishti` ve `compute_jaimini`), ikisi de 10 ms'nin altında (ölçüm: 0,6–0,8 ms); Vitest `65 passed (65)`, `571 passed (571)` (`api.test.ts`'in tablosu büyür, test sayısı değişmez; "dizgi bağımsız değişken" testi değişmeden geçer: C2′); `✓ built`.

- [ ] **Step 6: Commit**

```bash
git add app/src-tauri/src app/src-tauri/tests app/ui/bindings.ts app/ui/test app/ui/api.ts app/ui/api.test.ts app/e2e/ipc.ts
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(app): compute_jaimini command on the shared ChartInput pattern; bindings, wrapper and fixture"
```
Expected: iki `grep` çıktısız.

---

### Task 5: Ayar penceresi: ileri modül ayarları (karaka şeması, Jaimini eş lordları)

Spec K7; D14. Pencereye "İleri modül ayarları" grubu eklenir. Açık harita varsa haritanın ayarı değişir ve harita yeniden hesaplanır (mevcut `save_calc_settings` yolu); yoksa yeni haritaların varsayılanı değişir. Fark işareti ("≠ varsayılandan farklı") grubun başlığındadır. Kayıt, debounce ve boşaltma düzeni değişmez (C5).

**Files:**
- Modify: `app/ui/settings/{model.ts,SettingsDialog.tsx,settings.css}`, `app/ui/i18n/{tr,en,fr}.ts`
- Test: `app/ui/settings/SettingsDialog.test.tsx`

**Interfaces:**
- Consumes: `CalcSettings["advanced"]` (`{ karaka_scheme: "Eight" | "Seven"; jaimini_co_lords: boolean }`), pencerenin `change`, `differs`, `mark` yardımcıları, `SETTINGS_FIELDS` (`advanced` Plan 14'ten beri listededir), i18n `settings.field.advanced` ("İleri modül ayarları").
- Produces: `settings/model.ts`: `KARAKA_SCHEMES = ["Eight", "Seven"]`; i18n `settings.karakaScheme`, `settings.karakaSchemes.{Eight,Seven}`, `settings.jaiminiCoLords`; CSS `.settings-advanced`, `.settings-check`. `settings.karakaSchemes.*` metinleri Task 6'da karaka panelinin "Şema" satırında da kullanılır.

- [ ] **Step 1: Başarısız testleri yaz**

````diff
diff --git a/app/ui/settings/SettingsDialog.test.tsx b/app/ui/settings/SettingsDialog.test.tsx
index 3e57a43..6361147 100644
--- a/app/ui/settings/SettingsDialog.test.tsx
+++ b/app/ui/settings/SettingsDialog.test.tsx
@@ -221,3 +221,51 @@ test("a pending edit is written when the page hides or unloads", async () => {
   await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(2));
   expect(vi.mocked(api.saveCalcSettings).mock.calls[1][1].node).toBe("Mean");
 });
+
+test("the karaka scheme and the co-lord rule are edited in the advanced group and saved with the chart", async () => {
+  vi.mocked(api.saveCalcSettings).mockResolvedValue(okChart());
+  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />);
+  const group = screen.getByRole("group", { name: "İleri modül ayarları" });
+  const scheme = screen.getByLabelText("Chara karaka şeması");
+  const coLords = screen.getByRole("checkbox", { name: "Jaimini eş lordları (Akrep: Mars ve Ketu; Kova: Satürn ve Rahu)" });
+  expect(group).toContainElement(scheme);
+  expect(group).toContainElement(coLords);
+  expect(scheme).toHaveDisplayValue("Sekiz karaka (Rahu dahil)");
+  expect(coLords).toBeChecked();
+  fireEvent.change(scheme, { target: { value: "Seven" } });
+  await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(1));
+  expect(vi.mocked(api.saveCalcSettings).mock.calls[0][1].advanced).toEqual({ karaka_scheme: "Seven", jaimini_co_lords: true });
+  expect(scheme).toHaveDisplayValue("Yedi karaka (Rahu hariç)");
+  fireEvent.click(coLords);
+  await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(2));
+  expect(vi.mocked(api.saveCalcSettings).mock.calls[1][1].advanced).toEqual({ karaka_scheme: "Seven", jaimini_co_lords: false });
+  // Grup varsayılandan ayrıldığını metinle söyler (renk tek başına değil).
+  expect(group).toHaveTextContent("varsayılandan farklı");
+  // Diğer alanlar değişmez.
+  expect(vi.mocked(api.saveCalcSettings).mock.calls[1][1].node).toBe(fixture().opened.calc_settings.node);
+});
+
+test("an advanced difference is marked on its group and reset restores both fields", async () => {
+  vi.mocked(api.saveCalcSettings).mockResolvedValue(okChart());
+  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />, {
+    opened: openedWith({ advanced: { karaka_scheme: "Seven", jaimini_co_lords: false } }),
+  });
+  expect(screen.getAllByText("varsayılandan farklı")).toHaveLength(1);
+  expect(screen.getByRole("group", { name: "İleri modül ayarları varsayılandan farklı" })).toBeInTheDocument();
+  expect(screen.getByLabelText("Chara karaka şeması")).toHaveDisplayValue("Yedi karaka (Rahu hariç)");
+  fireEvent.click(screen.getByRole("button", { name: "Varsayılana döndür" }));
+  await waitFor(() => expect(api.saveCalcSettings).toHaveBeenCalledTimes(1));
+  expect(vi.mocked(api.saveCalcSettings).mock.calls[0][1].advanced).toEqual({ karaka_scheme: "Eight", jaimini_co_lords: true });
+  await waitFor(() => expect(screen.queryByText("varsayılandan farklı")).toBeNull());
+  expect(screen.getByRole("checkbox", { name: /Jaimini eş lordları/ })).toBeChecked();
+});
+
+test("without an open chart the advanced fields edit the defaults for new charts", async () => {
+  vi.mocked(api.setAppPrefs).mockResolvedValue({ status: "ok", data: null });
+  renderWithStores(<SettingsDialog onClose={() => {}} debounceMs={0} />, { opened: null });
+  fireEvent.click(screen.getByRole("checkbox", { name: /Jaimini eş lordları/ }));
+  await waitFor(() => expect(api.setAppPrefs).toHaveBeenCalledTimes(1));
+  expect(vi.mocked(api.setAppPrefs).mock.calls[0][0].calc_settings.advanced).toEqual({ karaka_scheme: "Eight", jaimini_co_lords: false });
+  expect(api.saveCalcSettings).not.toHaveBeenCalled();
+  expect(screen.queryByText("varsayılandan farklı")).toBeNull();
+});
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
pnpm --dir app exec vitest run ui/settings/SettingsDialog.test.tsx 2>&1 | grep -E "Test Files|Tests "
```
Expected: `Tests  3 failed | 13 passed (16)`.

- [ ] **Step 3: Model, pencere, stil ve metinler**

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index e4a6c04..f85c28e 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -489,6 +489,9 @@ export const en: Messages = {
     sunrise: { DiskCenterNoRefraction: "Disc centre, no refraction", UpperLimbRefracted: "Upper limb, refracted" },
     orb: { moon: "Moon", mars: "Mars", mercury: "Mercury", mercury_retro: "Mercury (retro)", jupiter: "Jupiter", venus: "Venus", venus_retro: "Venus (retro)", saturn: "Saturn" },
     orbInvalid: "Enter a number from 0 to 30.",
+    karakaScheme: "Chara karaka scheme",
+    karakaSchemes: { Eight: "Eight karakas (with Rahu)", Seven: "Seven karakas (without Rahu)" },
+    jaiminiCoLords: "Jaimini co-lords (Scorpio: Mars and Ketu; Aquarius: Saturn and Rahu)",
     differs: "differs from default",
     resetToDefault: "Reset to default",
     makeDefault: "Make this the default",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 423399a..1842d9c 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -489,6 +489,9 @@ export const fr: Messages = {
     sunrise: { DiskCenterNoRefraction: "Centre du disque, sans réfraction", UpperLimbRefracted: "Bord supérieur, avec réfraction" },
     orb: { moon: "Lune", mars: "Mars", mercury: "Mercure", mercury_retro: "Mercure (rétro)", jupiter: "Jupiter", venus: "Vénus", venus_retro: "Vénus (rétro)", saturn: "Saturne" },
     orbInvalid: "Saisissez un nombre entre 0 et 30.",
+    karakaScheme: "Schéma des chara karakas",
+    karakaSchemes: { Eight: "Huit karakas (avec Rahu)", Seven: "Sept karakas (sans Rahu)" },
+    jaiminiCoLords: "Co-maîtres Jaimini (Scorpion : Mars et Ketu ; Verseau : Saturne et Rahu)",
     differs: "différent du défaut",
     resetToDefault: "Revenir au défaut",
     makeDefault: "En faire le défaut",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index aaa14d3..c8cde92 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -488,6 +488,9 @@ export const tr = {
     sunrise: { DiskCenterNoRefraction: "Disk merkezi, kırılmasız", UpperLimbRefracted: "Üst kenar, kırılmalı" },
     orb: { moon: "Ay", mars: "Mars", mercury: "Merkür", mercury_retro: "Merkür (retro)", jupiter: "Jüpiter", venus: "Venüs", venus_retro: "Venüs (retro)", saturn: "Satürn" },
     orbInvalid: "0 ile 30 arasında bir sayı girin.",
+    karakaScheme: "Chara karaka şeması",
+    karakaSchemes: { Eight: "Sekiz karaka (Rahu dahil)", Seven: "Yedi karaka (Rahu hariç)" },
+    jaiminiCoLords: "Jaimini eş lordları (Akrep: Mars ve Ketu; Kova: Satürn ve Rahu)",
     differs: "varsayılandan farklı",
     resetToDefault: "Varsayılana döndür",
     makeDefault: "Bunu varsayılan yap",
diff --git a/app/ui/settings/SettingsDialog.tsx b/app/ui/settings/SettingsDialog.tsx
index ed21beb..f64d1f6 100644
--- a/app/ui/settings/SettingsDialog.tsx
+++ b/app/ui/settings/SettingsDialog.tsx
@@ -16,6 +16,7 @@ import {
   ayanamsaKey,
   DASHA_YEARS,
   HOUSE_SYSTEMS,
+  KARAKA_SCHEMES,
   NODES,
   ORB_KEYS,
   parseOrb,
@@ -265,6 +266,33 @@ export function SettingsDialog({ onClose, debounceMs = SAVE_DEBOUNCE_MS }: { onC
               );
             })}
           </fieldset>
+          <fieldset className={`settings-advanced${differs("advanced") ? " settings-row--differs" : ""}`}>
+            <legend>
+              {t("settings.field.advanced")} {mark("advanced")}
+            </legend>
+            <div className="settings-row">
+              <label htmlFor={`${id}-karaka_scheme`}>{t("settings.karakaScheme")}</label>
+              <select
+                id={`${id}-karaka_scheme`}
+                value={draft.advanced.karaka_scheme}
+                onChange={(e) => change({ ...draft, advanced: { ...draft.advanced, karaka_scheme: e.target.value as CalcSettings["advanced"]["karaka_scheme"] } })}
+              >
+                {KARAKA_SCHEMES.map((k) => (
+                  <option key={k} value={k}>
+                    {t(`settings.karakaSchemes.${k}`)}
+                  </option>
+                ))}
+              </select>
+            </div>
+            <label className="settings-check">
+              <input
+                type="checkbox"
+                checked={draft.advanced.jaimini_co_lords}
+                onChange={(e) => change({ ...draft, advanced: { ...draft.advanced, jaimini_co_lords: e.target.checked } })}
+              />
+              {t("settings.jaiminiCoLords")}
+            </label>
+          </fieldset>
         </div>
       )}
       {error && (
diff --git a/app/ui/settings/model.ts b/app/ui/settings/model.ts
index 2c4eae6..4cd6e9b 100644
--- a/app/ui/settings/model.ts
+++ b/app/ui/settings/model.ts
@@ -11,6 +11,8 @@ export const HOUSE_SYSTEMS = ["WholeSign", "Sripati", "Placidus"] as const satis
 export const SUNRISE_DEFS = ["DiskCenterNoRefraction", "UpperLimbRefracted"] as const satisfies readonly CalcSettings["sunrise_def"][];
 /** Seçilebilir Vimshottari yıl uzunlukları (gün): Julian yılı, savana yılı, tropik yıl. */
 export const DASHA_YEARS = [365.25, 360, 365.2422] as const;
+/** Chara karaka şemaları (`advanced.karaka_scheme`): sekizli şemada Rahu da karaka olur. */
+export const KARAKA_SCHEMES = ["Eight", "Seven"] as const satisfies readonly CalcSettings["advanced"]["karaka_scheme"][];
 export const ORB_KEYS = ["moon", "mars", "mercury", "mercury_retro", "jupiter", "venus", "venus_retro", "saturn"] as const satisfies readonly (keyof CombustionOrbs)[];
 export type OrbKey = (typeof ORB_KEYS)[number];
 
diff --git a/app/ui/settings/settings.css b/app/ui/settings/settings.css
index b3408c5..9ab4323 100644
--- a/app/ui/settings/settings.css
+++ b/app/ui/settings/settings.css
@@ -41,3 +41,16 @@
   width: 5rem;
   font-variant-numeric: tabular-nums;
 }
+
+.settings-advanced {
+  display: grid;
+  gap: 6px;
+  border: 1px solid var(--line);
+  padding: 8px;
+}
+
+.settings-check {
+  display: flex;
+  align-items: center;
+  gap: 6px;
+}
````

- [ ] **Step 4: Kapılar**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: `Test Files  65 passed (65)`, `Tests  574 passed (574)` (+3); `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/settings app/ui/i18n
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): settings dialog edits the karaka scheme and the jaimini co-lord rule"
```
Expected: iki `grep` çıktısız.

---

### Task 6: Chara karaka paneli; ortak `useJaimini` kancası ve vurgu düğmesi

Spec §5 "Arayüz", `jaimini_karakas` satırı; D9, D15, D16. İçerik: rol (kısaltma ve ad), graha, burç, burç içi derece (Rahu için gerçek derece ve parantez içinde etkin derece), eşitlik işareti `=`; altında karakamsha burcu ve dokuz grahanın karakamshadan ev sayıları (D1 ve D9). Durumlar: harita yok; yükleniyor (1 s'den sonra iskelet); hata ("Yeniden dene"); dolu; saat bilinmiyor (uyarı satırı, Ay'ın derecesinde ve karakamsha burcunda "≈"). Ortak seçim: graha iki tabloda da satırını vurgular; karakamsha burcu sabitlenebilir.

**Files:**
- Create: `app/ui/modules/useJaimini.ts`, `app/ui/panels/PinButton.tsx`, `app/ui/panels/JaiminiKarakasPanel.tsx`, `app/ui/panels/JaiminiKarakasPanel.test.tsx`
- Modify: `app/ui/layout/model.ts`, `app/ui/panels/{labels.ts,registry.ts,panels.css}`, `app/ui/i18n/{tr,en,fr}.ts`
- Test: `app/ui/panels/JaiminiKarakasPanel.test.tsx`, `app/ui/layout/model.test.ts`, `app/ui/panels/panels.test.tsx`

**Interfaces:**
- Consumes: `api.computeJaimini`, `JaiminiReport`, `KarakaRole` (Task 4); `useModule`, `ModuleState`, `chartInput`, `inputKey` (Plan 14); `useHighlight`, `sameTarget`, `HighlightTarget`; `Approx`, `formatDegree`, `PanelError`, `PanelSkeleton`, `usePanelLoadPhase`; `signIndex`, `GRAHA_IDS`; `timeUnknown`; i18n `table.{graha,sign,degree,approx}`, `badge.*`, `badgeHint.*`, `settings.karakaSchemes.*` (Task 5).
- Produces:
  - `modules/useJaimini.ts`: `useJaimini(opened: OpenedClient | null, enabled = true): ModuleState<JaiminiReport>`
  - `panels/PinButton.tsx`: `PinButton({ target: HighlightTarget, label?: string, children })`
  - `panels/labels.ts`: `KARAKA_ROLES: Record<KarakaRole, { short: string; name: string }>`, `arudhaLabel(house: number): string` (`"AL"`, `"A2"` … `"A11"`, `"UL"`)
  - `panels/JaiminiKarakasPanel.tsx`: `JaiminiKarakasPanel`, `TIE_MARK`, `degreeText(deg: number | null): string`
  - `layout/model.ts`: `PANEL_KINDS` ve `MODULE_PANELS` sonuna `jaimini_karakas` (5 × 14)
  - i18n: `panel.jaimini_karakas`, `jaimini.*` (`jaimini.method` Task 7 ve 8'de de kullanılır); CSS `.jaimini-karakamsha`.

- [ ] **Step 1: Başarısız testleri yaz**

**Yeni dosya:** `app/ui/panels/JaiminiKarakasPanel.test.tsx`

````tsx
import { act, fireEvent, screen, within } from "@testing-library/react";
import { expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeJaimini: vi.fn() } }));

import { api, type Result } from "../api";
import type { JaiminiReport } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { degreeText, JaiminiKarakasPanel, TIE_MARK } from "./JaiminiKarakasPanel";
import { arudhaLabel, KARAKA_ROLES } from "./labels";

const ok = (report: JaiminiReport = fixture().jaimini): Result<JaiminiReport> => ({ status: "ok", data: report });

function renderPanel(opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  const panel: Panel = { id: "jk", kind: "jaimini_karakas", x: 0, y: 0, w: 5, h: 14, settings: {} };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<JaiminiKarakasPanel panel={panel} />, { layout, opened });
}

const karakaTable = () => screen.findByRole("table", { name: "Chara karaka: rol sırasıyla graha, burç ve burç içi derece" });
const housesTable = () => screen.getByRole("table", { name: "Grahaların karakamshadan ev sayıları: rasi (D1) ve navamsa (D9)" });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);

beforeEach(() => {
  vi.mocked(api.computeJaimini).mockReset();
  vi.mocked(api.computeJaimini).mockImplementation(async () => ok());
});

test("asks for the open chart's report and lists the eight karakas in role order", async () => {
  renderPanel();
  const table = await karakaTable();
  const opened = fixture().opened;
  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
  expect(api.computeJaimini).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true });
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Rol", "Graha", "Burç", "Derece"]);
  const rows = within(table).getAllByRole("row").slice(1);
  // Delhi fikstürü, sekizli şema: Rahu'nun gerçek derecesi 11°11′, etkin derecesi 30° − 11°11′.
  expect(rows.map(cells)).toEqual([
    ["AK Atmakaraka", "Ay", "Ter", "19°28′"],
    ["AmK Amatyakaraka", "Rahu", "Yen", "11°11′ (etkin 18°48′)"],
    ["BK Bhratrikaraka", "Satürn", "Koç", "16°32′"],
    ["MK Matrikaraka", "Güneş", "Yay", "16°30′"],
    ["PiK Pitrikaraka", "Merkür", "Yay", "8°02′"],
    ["PK Putrakaraka", "Venüs", "Akr", "7°42′"],
    ["GK Gnatikaraka", "Mars", "Kov", "4°06′"],
    ["DK Darakaraka", "Jüpiter", "Koç", "1°23′"],
  ]);
  // Rol kısaltmasının açılımı hem `title` hem ekran okuyucu metnidir.
  expect(within(table).getByTitle("Atmakaraka")).toHaveTextContent("AK");
  expect(screen.getByText("Rahu'nun etkin derecesi burcun sonundan ölçülür (30° − derece).")).toBeInTheDocument();
  expect(screen.queryByText(/1″ içinde eşit/)).toBeNull();
  expect(screen.getByText("Şema: Sekiz karaka (Rahu dahil)")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: jaimini.core_v1")).toBeInTheDocument();
  expect(document.querySelector('[title="yaklaşık"]')).toBeNull();
});

test("karakamsha and the houses of the nine grahas from it, in D1 and D9", async () => {
  renderPanel();
  await karakaTable();
  // Atmakaraka Ay'ın navamsa burcu Balık.
  expect(document.querySelector(".jaimini-karakamsha")).toHaveTextContent("Karakamsha: Bal Atmakaraka Ay, navamsa burcu");
  const rows = within(housesTable()).getAllByRole("row");
  expect(cells(rows[0])).toEqual(["Graha", "D1 evi", "D9 evi"]);
  expect(rows.slice(1).map(cells)).toEqual([
    ["Güneş", "10", "6"],
    ["Ay", "8", "1"],
    ["Mars", "12", "9"],
    ["Merkür", "10", "4"],
    ["Jüpiter", "2", "2"],
    ["Venüs", "9", "7"],
    ["Satürn", "2", "6"],
    ["Rahu", "5", "8"],
    ["Ketu", "11", "2"],
  ]);
});

test("a tie is marked with the sign and with words, and explained once", async () => {
  const report = fixture().jaimini;
  report.karakas[2].tie = true;
  report.karakas[3].tie = true;
  vi.mocked(api.computeJaimini).mockResolvedValue(ok(report));
  renderPanel();
  const rows = within(await karakaTable()).getAllByRole("row").slice(1);
  expect(rows[2].lastElementChild).toHaveTextContent(`16°32′ ${TIE_MARK}eşit derece`);
  expect(rows[3].lastElementChild).toHaveTextContent(`16°30′ ${TIE_MARK}eşit derece`);
  expect(rows[1].lastElementChild).not.toHaveTextContent(TIE_MARK);
  // İşaret ekran okuyucudan gizlidir; sözcük okunur.
  expect(rows[2].querySelector('[aria-hidden="true"]')).toHaveTextContent(TIE_MARK);
  expect(screen.getByText(`${TIE_MARK} sıradaki komşusuyla 1″ içinde eşit derece; sıra tam değerle belirlenir.`)).toBeInTheDocument();
});

test("the seven-karaka scheme has no Rahu row, no Pitri role and no Rahu note", async () => {
  const report = fixture().jaimini;
  const roles = ["Atma", "Amatya", "Bhratri", "Matri", "Putra", "Gnati", "Dara"] as const;
  report.scheme = "Seven";
  report.karakas = report.karakas.filter((k) => k.graha !== "Rahu").map((k, i) => ({ ...k, role: roles[i] }));
  vi.mocked(api.computeJaimini).mockResolvedValue(ok(report));
  renderPanel();
  const rows = within(await karakaTable()).getAllByRole("row").slice(1);
  expect(rows.map((r) => cells(r).slice(0, 2))).toEqual([
    ["AK Atmakaraka", "Ay"],
    ["AmK Amatyakaraka", "Satürn"],
    ["BK Bhratrikaraka", "Güneş"],
    ["MK Matrikaraka", "Merkür"],
    ["PK Putrakaraka", "Venüs"],
    ["GK Gnatikaraka", "Mars"],
    ["DK Darakaraka", "Jüpiter"],
  ]);
  expect(screen.queryByText(/Rahu'nun etkin derecesi/)).toBeNull();
  expect(screen.getByText("Şema: Yedi karaka (Rahu hariç)")).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderPanel(null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeJaimini).not.toHaveBeenCalled();
});

test("unknown birth time is sent to Rust; the Moon's degree and the karakamsha are marked approximate", async () => {
  renderPanel(unknownTimeOpened());
  const table = await karakaTable();
  expect(vi.mocked(api.computeJaimini).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByText("Doğum saati bilinmiyor: Ay'ın derecesi yaklaşıktır (≈); karaka sırası ve karakamsha değişebilir.")).toBeInTheDocument();
  const rows = within(table).getAllByRole("row").slice(1);
  expect(rows[0].lastElementChild).toHaveTextContent("≈yaklaşık 19°28′");
  expect(rows[2].querySelector('[title="yaklaşık"]')).toBeNull();
  expect(document.querySelector(".jaimini-karakamsha")!.querySelector('[title="yaklaşık"]')).not.toBeNull();
  expect(document.querySelectorAll('[title="yaklaşık"]')).toHaveLength(2);
});

test("a low-precision warning from the report is shown as text", async () => {
  const report = fixture().jaimini;
  report.warnings = ["LowPrecision"];
  vi.mocked(api.computeJaimini).mockResolvedValue(ok(report));
  renderPanel();
  await karakaTable();
  expect(screen.getByText("Düşük hassasiyet")).toBeInTheDocument();
  expect(screen.getByText(/Güneş veya Ay yedek ephemeris ile hesaplandı\./)).toBeInTheDocument();
});

test("an error is shown in the panel and retry asks again", async () => {
  vi.mocked(api.computeJaimini).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.queryByRole("table")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await karakaTable();
  expect(api.computeJaimini).toHaveBeenCalledTimes(2);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("loading: nothing under a second, then a skeleton, then the tables", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<JaiminiReport>) => void;
    vi.mocked(api.computeJaimini).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getAllByRole("table")).toHaveLength(2);
  } finally {
    vi.useRealTimers();
  }
});

test("shared highlight: a graha marks its row in both tables; the karakamsha sign can be pinned", async () => {
  renderPanel();
  const table = await karakaTable();
  const saturn = within(table).getByRole("button", { name: "Satürn" });
  fireEvent.click(saturn);
  expect(saturn).toHaveAttribute("aria-pressed", "true");
  expect(document.querySelectorAll("tr.is-highlighted")).toHaveLength(2);
  expect(document.querySelector('tr[data-role="Bhratri"]')).toHaveClass("is-highlighted");
  expect(document.querySelector('tr[data-house-of="Saturn"]')).toHaveClass("is-highlighted");
  // Ketu karaka değildir: yalnızca ev tablosunda satırı vardır.
  fireEvent.click(within(housesTable()).getByRole("button", { name: "Ketu" }));
  expect(document.querySelectorAll("tr.is-highlighted")).toHaveLength(1);
  const sign = screen.getByRole("button", { name: "Bal" });
  fireEvent.click(sign);
  expect(sign).toHaveAttribute("aria-pressed", "true");
  expect(document.querySelectorAll("tr.is-highlighted")).toHaveLength(0);
  fireEvent.keyDown(window, { key: "Escape" });
  expect(sign).toHaveAttribute("aria-pressed", "false");
});

test("helpers: degree text, role names and pada labels", () => {
  expect([degreeText(16.5), degreeText(0), degreeText(null), degreeText(Number.NaN)]).toEqual(["16°30′", "0°00′", "—", "—"]);
  expect(Object.values(KARAKA_ROLES).map((r) => r.short)).toEqual(["AK", "AmK", "BK", "MK", "PiK", "PK", "GK", "DK"]);
  expect(new Set(Object.values(KARAKA_ROLES).map((r) => r.name)).size).toBe(8);
  expect([1, 2, 7, 11, 12].map(arudhaLabel)).toEqual(["AL", "A2", "A7", "A11", "UL"]);
});
````

````diff
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index 0286383..697ad29 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -24,7 +24,7 @@ function overlaps(a: Panel, b: Panel) {
   return a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
 }
 
-test("catalog has the seven v1 panel kinds and the three foundation module panels", () => {
+test("catalog has the seven v1 panel kinds and the module panels", () => {
   expect(PANEL_KINDS).toEqual([
     "chart",
     "graha_table",
@@ -36,6 +36,7 @@ test("catalog has the seven v1 panel kinds and the three foundation module panel
     "panchang",
     "ashtakavarga",
     "drishti",
+    "jaimini_karakas",
   ]);
   expect(isKnownPanelKind("chart")).toBe(true);
   expect(isKnownPanelKind("drishti")).toBe(true);
@@ -43,7 +44,7 @@ test("catalog has the seven v1 panel kinds and the three foundation module panel
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index d82f400..764900e 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -13,6 +13,7 @@ import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel } from "./ChartPanel";
 import { DrishtiPanel } from "./DrishtiPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
+import { JaiminiKarakasPanel } from "./JaiminiKarakasPanel";
 import { PanchangPanel } from "./PanchangPanel";
 import { panelDefinition, PANEL_REGISTRY } from "./registry";
 import { UnavailablePanel } from "./UnavailablePanel";
@@ -40,6 +41,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.panchang.component).toBe(PanchangPanel);
   expect(PANEL_REGISTRY.ashtakavarga.component).toBe(AshtakavargaPanel);
   expect(PANEL_REGISTRY.drishti.component).toBe(DrishtiPanel);
+  expect(PANEL_REGISTRY.jaimini_karakas.component).toBe(JaiminiKarakasPanel);
   // Kataloğun her türü uygulanmıştır: yer tutucu yoktur.
   for (const kind of PANEL_KINDS) expect(PANEL_REGISTRY[kind].component, kind).not.toBe(UnavailablePanel);
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
pnpm --dir app exec vitest run ui/panels/JaiminiKarakasPanel.test.tsx ui/layout/model.test.ts ui/panels/panels.test.tsx 2>&1 | grep -E "Test Files|Tests "
```
Expected: `Test Files  3 failed (3)` (yeni test dosyası içe aktarımda düşer; diğer ikisinde birer test düşer).

- [ ] **Step 3: Ortak parçalar: `useJaimini`, `PinButton`, rol ve pada etiketleri**

**Yeni dosya:** `app/ui/modules/useJaimini.ts`

````ts
// Jaimini raporunu (`compute_jaimini`) isteyen ortak kanca. Karaka, arudha ve argala panelleri,
// harita panelinin arudha işaretleri, Bakışlar panelinin rashi türü ve graha tablosunun karaka
// sütunu aynı raporu okur. Rapor saklanmaz ve arayüzde hesap yoktur (K1, C4); her tüketici
// kendi isteğini yapar (komut 1 ms'nin altındadır).
import { api } from "../api";
import type { JaiminiReport, OpenedClient } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

/** `enabled === false`: rapor gerekmiyor (ör. işaretler kapalı), komut çağrılmaz. */
export function useJaimini(opened: OpenedClient | null, enabled = true): ModuleState<JaiminiReport> {
  return useModule(opened !== null && enabled ? inputKey(opened) : null, () => api.computeJaimini(chartInput(opened as OpenedClient)));
}
````

**Yeni dosya:** `app/ui/panels/PinButton.tsx`

````tsx
// Ortak vurgu düğmesi (UI spec §3.4): üzerine gelince ya da odaklanınca geçici vurgu, tıklayınca
// sabitleme. İleri modül panellerindeki graha ve burç adları bununla yazılır.
import type { ReactNode } from "react";
import { useHighlight } from "../highlight/store";
import { sameTarget, type HighlightTarget } from "../highlight/target";

export function PinButton({ target, label, children }: { target: HighlightTarget; /** Kısaltma gösterilirken okunacak tam ad. */ label?: string; children: ReactNode }) {
  const highlight = useHighlight();
  return (
    <button
      type="button"
      className="link-button"
      aria-label={label}
      aria-pressed={sameTarget(highlight.pinned, target)}
      onClick={() => highlight.togglePin(target)}
      onMouseEnter={() => highlight.hover(target)}
      onMouseLeave={() => highlight.hover(null)}
      onFocus={() => highlight.hover(target)}
      onBlur={() => highlight.hover(null)}
    >
      {children}
    </button>
  );
}
````

````diff
diff --git a/app/ui/panels/labels.ts b/app/ui/panels/labels.ts
index e5c6b42..e963dc3 100644
--- a/app/ui/panels/labels.ts
+++ b/app/ui/panels/labels.ts
@@ -1,4 +1,5 @@
 import type { TFunction } from "i18next";
+import type { KarakaRole } from "../bindings";
 import { GRAHA_IDS, type ChartLabels, type GrahaId } from "../chart/types";
 
 export function grahaAbbreviations(t: TFunction): Record<GrahaId, string> {
@@ -134,3 +135,20 @@ export const VARA_NAMES = {
 export function nakshatraName(n: number): string {
   return NAKSHATRA_NAMES[n - 1] ?? String(n);
 }
+
+/** Chara karaka rollerinin kısaltması ve adı (Sanskritçe terim; dile göre değişmez). */
+export const KARAKA_ROLES: Record<KarakaRole, { short: string; name: string }> = {
+  Atma: { short: "AK", name: "Atmakaraka" },
+  Amatya: { short: "AmK", name: "Amatyakaraka" },
+  Bhratri: { short: "BK", name: "Bhratrikaraka" },
+  Matri: { short: "MK", name: "Matrikaraka" },
+  Pitri: { short: "PiK", name: "Pitrikaraka" },
+  Putra: { short: "PK", name: "Putrakaraka" },
+  Gnati: { short: "GK", name: "Gnatikaraka" },
+  Dara: { short: "DK", name: "Darakaraka" },
+};
+
+/** Padanın etiketi: A1 Arudha Lagna (AL), A12 Upapada (UL), diğerleri A2…A11. */
+export function arudhaLabel(house: number): string {
+  return house === 1 ? "AL" : house === 12 ? "UL" : `A${house}`;
+}
````

- [ ] **Step 4: Panel, kayıt, stil ve metinler**

**Yeni dosya:** `app/ui/panels/JaiminiKarakasPanel.tsx`

````tsx
// Jaimini karaka tablosu (ileri modüller spec'i §5): rol, graha, burç içi derece; altında
// karakamsha burcu ve grahaların karakamshadan ev sayıları (D1 ve D9). Değerler
// `compute_jaimini`'den gelir; gerçek derece açık haritadandır. Burada hesap yoktur (C4).
// Eşitlik `=` ve metinle, yaklaşık değer `≈` ve metinle gösterilir; renk anlam taşımaz (C9).
import { useTranslation } from "react-i18next";
import type { JaiminiReport, OpenedClient } from "../bindings";
import { signIndex } from "../chart/adapter";
import { GRAHA_IDS } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { useJaimini } from "../modules/useJaimini";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { Approx } from "./Approx";
import { formatDegree } from "./format";
import { KARAKA_ROLES } from "./labels";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export const TIE_MARK = "=";
const DASH = "—";

/** Derece metni; sonlu olmayan değer çizgi. */
export function degreeText(deg: number | null): string {
  return deg === null || !Number.isFinite(deg) ? DASH : formatDegree(deg);
}

export function JaiminiKarakasPanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const module = useJaimini(opened);
  const phase = usePanelLoadPhase(module.loading);
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (module.data === null) return phase === "skeleton" ? <PanelSkeleton /> : null;
  return <Karakas report={module.data} opened={opened} />;
}

function Karakas({ report, opened }: { report: JaiminiReport; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const unknown = timeUnknown(opened);
  const positions = new Map(opened.chart.grahas.map((g) => [g.graha, g]));
  const grahaOn = (g: string) => active?.type === "graha" && active.id === g;
  const km = report.karakamsha;
  const kmSign = signIndex(km.sign);
  const anyTie = report.karakas.some((k) => k.tie);
  const hasRahu = report.karakas.some((k) => k.graha === "Rahu");

  return (
    <div className="module-panel">
      {unknown && <p className="panel-notice">{t("jaimini.timeUnknown")}</p>}
      {report.warnings.map((w) => (
        <p key={w} className="panel-notice">
          <strong>{t(`badge.${w}`)}</strong>: {t(`badgeHint.${w}`)}
        </p>
      ))}
      <table className="module-table">
        <caption className="visually-hidden">{t("jaimini.karakaCaption")}</caption>
        <thead>
          <tr>
            <th scope="col">{t("jaimini.role")}</th>
            <th scope="col">{t("table.graha")}</th>
            <th scope="col">{t("table.sign")}</th>
            <th scope="col" className="num">
              {t("table.degree")}
            </th>
          </tr>
        </thead>
        <tbody>
          {report.karakas.map((k) => {
            const role = KARAKA_ROLES[k.role];
            const g = positions.get(k.graha);
            const real = degreeText(g?.deg_in_sign ?? null);
            return (
              <tr key={k.role} data-role={k.role} data-graha={k.graha} className={grahaOn(k.graha) ? "is-highlighted" : undefined}>
                <th scope="row">
                  <abbr title={role.name}>{role.short}</abbr>
                  <span className="visually-hidden"> {role.name}</span>
                </th>
                <td>
                  <PinButton target={{ type: "graha", id: k.graha }}>{t(`grahaName.${k.graha}`)}</PinButton>
                </td>
                <td>{g ? t(`sign.${signIndex(g.sign)}`) : DASH}</td>
                <td className="num">
                  <Approx on={unknown && k.graha === "Moon"} label={t("table.approx")}>
                    {real}
                  </Approx>
                  {k.graha === "Rahu" && <> ({t("jaimini.effective", { deg: degreeText(k.effective_deg) })})</>}
                  {k.tie && (
                    <>
                      {" "}
                      <span aria-hidden="true">{TIE_MARK}</span>
                      <span className="visually-hidden">{t("jaimini.tie")}</span>
                    </>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {anyTie && <p className="panel-message">{t("jaimini.tieLegend", { mark: TIE_MARK })}</p>}
      {hasRahu && <p className="panel-message">{t("jaimini.rahuLegend")}</p>}
      <p className="jaimini-karakamsha">
        {t("jaimini.karakamsha")}:{" "}
        <Approx on={unknown} label={t("table.approx")}>
          <PinButton target={{ type: "sign", sign: kmSign }}>{t(`sign.${kmSign}`)}</PinButton>
        </Approx>{" "}
        <span className="panel-message">{t("jaimini.karakamshaOf", { graha: t(`grahaName.${km.atmakaraka}`) })}</span>
      </p>
      <table className="module-table">
        <caption className="visually-hidden">{t("jaimini.housesCaption")}</caption>
        <thead>
          <tr>
            <th scope="col">{t("table.graha")}</th>
            <th scope="col" className="num">
              {t("jaimini.d1")}
            </th>
            <th scope="col" className="num">
              {t("jaimini.d9")}
            </th>
          </tr>
        </thead>
        <tbody>
          {GRAHA_IDS.map((g, i) => (
            <tr key={g} data-house-of={g} className={grahaOn(g) ? "is-highlighted" : undefined}>
              <th scope="row">
                <PinButton target={{ type: "graha", id: g }}>{t(`grahaName.${g}`)}</PinButton>
              </th>
              <td className="num">{km.houses_in_d1[i]}</td>
              <td className="num">{km.houses_in_d9[i]}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <p className="panel-message">{t("jaimini.scheme", { scheme: t(`settings.karakaSchemes.${report.scheme}`) })}</p>
      <p className="panel-message">{t("jaimini.method", { method: report.method })}</p>
    </div>
  );
}
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index f85c28e..8448579 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -59,6 +59,7 @@ export const en: Messages = {
     panchang: "Panchang",
     ashtakavarga: "Ashtakavarga",
     drishti: "Aspects",
+    jaimini_karakas: "Chara karaka",
     unknown: "Unknown panel",
     loading: "Loading",
     unavailable: "This panel arrives in a later version.",
@@ -114,6 +115,22 @@ export const en: Messages = {
     method: "Method: {{method}}",
     timeUnknown: "Birth time unknown: the Moon's position and aspects are approximate (≈).",
   },
+  jaimini: {
+    karakaCaption: "Chara karaka: graha, sign and degree in sign, in role order",
+    role: "Role",
+    effective: "effective {{deg}}",
+    tie: "equal degree",
+    tieLegend: "{{mark}} equal to its neighbour in rank within 1″; the order follows the exact value.",
+    rahuLegend: "Rahu's effective degree is measured from the end of its sign (30° − degree).",
+    karakamsha: "Karakamsha",
+    karakamshaOf: "Atmakaraka {{graha}}, navamsa sign",
+    housesCaption: "Houses of the grahas from the karakamsha: rasi (D1) and navamsa (D9)",
+    d1: "D1 house",
+    d9: "D9 house",
+    scheme: "Scheme: {{scheme}}",
+    method: "Method: {{method}}",
+    timeUnknown: "Birth time unknown: the Moon's degree is approximate (≈); the karaka order and the karakamsha may differ.",
+  },
   table: {
     graha: "Graha",
     sign: "Sign",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 1842d9c..da84ddb 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -59,6 +59,7 @@ export const fr: Messages = {
     panchang: "Panchang",
     ashtakavarga: "Ashtakavarga",
     drishti: "Aspects",
+    jaimini_karakas: "Chara karaka",
     unknown: "Panneau inconnu",
     loading: "Chargement",
     unavailable: "Ce panneau arrivera dans une version ultérieure.",
@@ -114,6 +115,22 @@ export const fr: Messages = {
     method: "Méthode : {{method}}",
     timeUnknown: "Heure de naissance inconnue : la position et les aspects de la Lune sont approximatifs (≈).",
   },
+  jaimini: {
+    karakaCaption: "Chara karaka : graha, signe et degré dans le signe, dans l'ordre des rôles",
+    role: "Rôle",
+    effective: "effectif {{deg}}",
+    tie: "degré égal",
+    tieLegend: "{{mark}} égal à son voisin de rang à 1″ près ; l'ordre suit la valeur exacte.",
+    rahuLegend: "Le degré effectif de Rahu se mesure depuis la fin du signe (30° − degré).",
+    karakamsha: "Karakamsha",
+    karakamshaOf: "Atmakaraka {{graha}}, signe de navamsa",
+    housesCaption: "Maisons des grahas depuis le karakamsha : rasi (D1) et navamsa (D9)",
+    d1: "Maison D1",
+    d9: "Maison D9",
+    scheme: "Schéma : {{scheme}}",
+    method: "Méthode : {{method}}",
+    timeUnknown: "Heure de naissance inconnue : le degré de la Lune est approximatif (≈) ; l'ordre des karakas et le karakamsha peuvent différer.",
+  },
   table: {
     graha: "Graha",
     sign: "Signe",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index c8cde92..8e021c1 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -58,6 +58,7 @@ export const tr = {
     panchang: "Panchang",
     ashtakavarga: "Ashtakavarga",
     drishti: "Bakışlar",
+    jaimini_karakas: "Chara karaka",
     unknown: "Bilinmeyen panel",
     loading: "Yükleniyor",
     unavailable: "Bu panel sonraki bir sürümde gelecek.",
@@ -113,6 +114,22 @@ export const tr = {
     method: "Yöntem: {{method}}",
     timeUnknown: "Doğum saati bilinmiyor: Ay'ın konumu ve bakışları yaklaşıktır (≈).",
   },
+  jaimini: {
+    karakaCaption: "Chara karaka: rol sırasıyla graha, burç ve burç içi derece",
+    role: "Rol",
+    effective: "etkin {{deg}}",
+    tie: "eşit derece",
+    tieLegend: "{{mark}} sıradaki komşusuyla 1″ içinde eşit derece; sıra tam değerle belirlenir.",
+    rahuLegend: "Rahu'nun etkin derecesi burcun sonundan ölçülür (30° − derece).",
+    karakamsha: "Karakamsha",
+    karakamshaOf: "Atmakaraka {{graha}}, navamsa burcu",
+    housesCaption: "Grahaların karakamshadan ev sayıları: rasi (D1) ve navamsa (D9)",
+    d1: "D1 evi",
+    d9: "D9 evi",
+    scheme: "Şema: {{scheme}}",
+    method: "Yöntem: {{method}}",
+    timeUnknown: "Doğum saati bilinmiyor: Ay'ın derecesi yaklaşıktır (≈); karaka sırası ve karakamsha değişebilir.",
+  },
   table: {
     graha: "Graha",
     sign: "Burç",
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index 0a89253..2ced278 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -1,7 +1,7 @@
 // Pano düzeni veri modeli (spec §3.2). Kalıcı biçim `ViewState.panel_layout` ile aynıdır;
 // M4'teki store komutu gelene kadar localStorage'da tutulur (bkz. persistence.ts).
 
-/** Panel kataloğu: v1 (UI spec §3.3) ve ileri modül panelleri (ileri modüller spec'i §4). */
+/** Panel kataloğu: v1 (UI spec §3.3) ve ileri modül panelleri (ileri modüller spec'i §4, §5). */
 export const PANEL_KINDS = [
   "chart",
   "graha_table",
@@ -13,6 +13,7 @@ export const PANEL_KINDS = [
   "panchang",
   "ashtakavarga",
   "drishti",
+  "jaimini_karakas",
 ] as const;
 export type KnownPanelKind = (typeof PANEL_KINDS)[number];
 
@@ -24,6 +25,8 @@ export const MODULE_PANELS = [
   { kind: "panchang", w: 4, h: 9 },
   { kind: "ashtakavarga", w: 6, h: 11 },
   { kind: "drishti", w: 6, h: 10 },
+  // Jaimini çekirdeği (Plan 15).
+  { kind: "jaimini_karakas", w: 5, h: 14 },
 ] as const satisfies readonly { kind: KnownPanelKind; w: number; h: number }[];
 export type ModulePanelKind = (typeof MODULE_PANELS)[number]["kind"];
 
diff --git a/app/ui/panels/panels.css b/app/ui/panels/panels.css
index e806655..7fb48c3 100644
--- a/app/ui/panels/panels.css
+++ b/app/ui/panels/panels.css
@@ -190,3 +190,10 @@
 .drishti-table td {
   padding: 2px 4px;
 }
+.jaimini-karakamsha {
+  margin: 8px 0 0;
+  font-weight: var(--fw-medium);
+}
+.jaimini-karakamsha .panel-message {
+  font-weight: var(--fw-regular);
+}
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index 5584100..7b6caf5 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -9,6 +9,7 @@ import { ChartPanel, chartVarga } from "./ChartPanel";
 import { TransitTablePanel } from "../transit/TransitTablePanel";
 import { DrishtiPanel } from "./DrishtiPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
+import { JaiminiKarakasPanel } from "./JaiminiKarakasPanel";
 import { PanchangPanel } from "./PanchangPanel";
 import type { PanelDefinition } from "./types";
 import { UnavailablePanel } from "./UnavailablePanel";
@@ -25,6 +26,8 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   panchang: { titleKey: "panel.panchang", component: PanchangPanel },
   ashtakavarga: { titleKey: "panel.ashtakavarga", component: AshtakavargaPanel },
   drishti: { titleKey: "panel.drishti", component: DrishtiPanel },
+  // Jaimini çekirdeği (Plan 15).
+  jaimini_karakas: { titleKey: "panel.jaimini_karakas", component: JaiminiKarakasPanel },
 };
 
 const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };
````

- [ ] **Step 5: Kapılar**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: `Test Files  66 passed (66)`, `Tests  585 passed (585)` (+11: `JaiminiKarakasPanel`); `✓ built`. Paletteki "Chara karaka panelini aç/kapat" öğesi `MODULE_PANELS`'ten kendiliğinden gelir (Plan 14 D9).

- [ ] **Step 6: Commit**

```bash
git add app/ui/modules app/ui/panels app/ui/layout app/ui/i18n
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): chara karaka panel with karakamsha; shared jaimini hook and pin button"
```
Expected: iki `grep` çıktısız.

---

### Task 7: Arudha padalar paneli ve harita panelinde arudha işaretleri

Spec §5 "Arayüz", `arudha` satırı ve "`chart` paneline `arudha_marks` ayarı eklenir"; D8, D18. Panel: A1 (AL) … A12 (UL) için padanın burcu, evin lordu, istisnada `*` ve dipnot; eş lord ayarının durumu bir satırla. Saat bilinmiyorsa Rust padaları boş döner ve panel tek satır açıklama gösterir. Harita: "Arudha padalar" kutusu açıkken D1'de on iki pada burçlarına yazılır.

**Files:**
- Create: `app/ui/panels/ArudhaPanel.tsx`, `app/ui/panels/ArudhaPanel.test.tsx`
- Modify: `app/ui/layout/model.ts`, `app/ui/panels/{registry.ts,ChartPanel.tsx}`, `app/ui/chart/{types.ts,ChartSvg.tsx,chart.css}`, `app/ui/i18n/{tr,en,fr}.ts`
- Test: `app/ui/panels/ArudhaPanel.test.tsx`, `app/ui/panels/ChartPanel.test.tsx`, `app/ui/chart/ChartSvg.test.tsx`, `app/ui/layout/model.test.ts`, `app/ui/panels/panels.test.tsx`

**Interfaces:**
- Consumes: `useJaimini`, `PinButton`, `arudhaLabel` (Task 6); `JaiminiReport["arudhas"]`; `signIndex`, `signAbbreviations`; `layoutLabels` (harita etiket yerleşimi); `PanelError`; i18n `jaimini.method`, `chart.transitList` kalıbı.
- Produces:
  - `panels/ArudhaPanel.tsx`: `ArudhaPanel`, `EXCEPTION_MARK`
  - `chart/types.ts`: `interface ChartMark { sign: number; text: string }`, `ChartData.marks?: ChartMark[]`
  - `chart/ChartSvg.tsx`: işaretler `<text class="chart-mark" data-mark data-house aria-hidden>` olarak çizilir
  - `panels/ChartPanel.tsx`: `settings.arudha_marks: boolean`
  - `layout/model.ts`: `PANEL_KINDS` ve `MODULE_PANELS` sonuna `arudha` (4 × 12)
  - i18n: `panel.arudha`, `arudha.*`, `chart.{arudhaMarks,arudhaMarksOnlyD1,arudhaMarksTimeUnknown,arudhaList}`; CSS `.chart-mark`.

- [ ] **Step 1: Başarısız testleri yaz**

**Yeni dosya:** `app/ui/panels/ArudhaPanel.test.tsx`

````tsx
import { fireEvent, screen, within } from "@testing-library/react";
import { expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeJaimini: vi.fn() } }));

import { api, type Result } from "../api";
import type { JaiminiReport } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { ArudhaPanel, EXCEPTION_MARK } from "./ArudhaPanel";

const ok = (report: JaiminiReport = fixture().jaimini): Result<JaiminiReport> => ({ status: "ok", data: report });

function renderPanel(opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  const panel: Panel = { id: "ar", kind: "arudha", x: 0, y: 0, w: 4, h: 12, settings: {} };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<ArudhaPanel panel={panel} />, { layout, opened });
}

const table = () => screen.findByRole("table", { name: "Arudha padalar: on iki evin padası, burcu ve evin lordu" });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);

beforeEach(() => {
  vi.mocked(api.computeJaimini).mockReset();
  vi.mocked(api.computeJaimini).mockImplementation(async () => ok());
});

test("lists the twelve padas with sign and lord; AL and UL are labelled; exceptions carry a mark and a note", async () => {
  renderPanel();
  const rows = within(await table()).getAllByRole("row");
  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.computeJaimini).mock.calls[0][0].time_known).toBe(true);
  expect(cells(rows[0])).toEqual(["Pada", "Burç", "Evin lordu"]);
  const x = `${EXCEPTION_MARK} istisna`;
  // Delhi fikstürü, İkizler lagna, eş lordlar açık (6. ev Akrep: Mars; 9. ev Kova: Satürn).
  expect(rows.slice(1).map(cells)).toEqual([
    ["A1 · AL", `Bal${x}`, "Merkür"],
    ["A2", `Ter${x}`, "Ay"],
    ["A3", "Koç", "Güneş"],
    ["A4", `Yay${x}`, "Merkür"],
    ["A5", "Yay", "Venüs"],
    ["A6", `Kov${x}`, "Mars"],
    ["A7", "Asl", "Jüpiter"],
    ["A8", `Koç${x}`, "Satürn"],
    ["A9", "İki", "Satürn"],
    ["A10", "Boğ", "Jüpiter"],
    ["A11", "Yay", "Mars"],
    ["A12 · UL", `Kov${x}`, "Venüs"],
  ]);
  // İşaret ekran okuyucudan gizlidir; "istisna" sözcükle okunur.
  expect(document.querySelectorAll('td [aria-hidden="true"]')).toHaveLength(6);
  expect(screen.getByText(`${EXCEPTION_MARK} İstisna: hesaplanan burç evin kendisi ya da 7. burcuydu; pada ondan 10. burçtur.`)).toBeInTheDocument();
  expect(screen.getByText("Jaimini eş lordları açık: Akrep ve Kova'nın lordu, güçlü olan eş lorddur.")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: jaimini.core_v1")).toBeInTheDocument();
});

test("without exceptions there is no note; switched-off co-lords are stated", async () => {
  const report = fixture().jaimini;
  report.co_lords = false;
  report.arudhas = report.arudhas!.map((p) => ({ ...p, exception: false })) as JaiminiReport["arudhas"];
  vi.mocked(api.computeJaimini).mockResolvedValue(ok(report));
  renderPanel();
  await table();
  expect(screen.queryByText(/İstisna:/)).toBeNull();
  expect(document.querySelector('td [aria-hidden="true"]')).toBeNull();
  expect(screen.getByText("Jaimini eş lordları devre dışı: Akrep'in lordu Mars, Kova'nın lordu Satürn'dür.")).toBeInTheDocument();
});

test("unknown birth time: Rust returns no padas and the panel says why", async () => {
  const report = fixture().jaimini;
  report.arudhas = null;
  vi.mocked(api.computeJaimini).mockResolvedValue(ok(report));
  renderPanel(unknownTimeOpened());
  expect(await screen.findByText("Doğum saati bilinmiyor: arudha padalar lagnaya bağlı olduğu için gösterilmiyor.")).toBeInTheDocument();
  expect(vi.mocked(api.computeJaimini).mock.calls[0][0].time_known).toBe(false);
  expect(screen.queryByRole("table")).toBeNull();
});

test("without an open chart nothing is requested", () => {
  renderPanel(null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeJaimini).not.toHaveBeenCalled();
});

test("an error is shown in the panel and retry asks again", async () => {
  vi.mocked(api.computeJaimini).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await table();
  expect(api.computeJaimini).toHaveBeenCalledTimes(2);
});

test("shared highlight: a sign marks every pada in it, a graha every pada it rules, a house its sign", async () => {
  renderPanel();
  const t = await table();
  const marked = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.pada);
  // Yay'da üç pada var: A4, A5, A11.
  const sag = within(t).getAllByRole("button", { name: "Yay" })[0];
  fireEvent.click(sag);
  expect(sag).toHaveAttribute("aria-pressed", "true");
  expect(marked()).toEqual(["4", "5", "11"]);
  // Satürn iki evin lordu: A8, A9.
  fireEvent.click(within(t).getAllByRole("button", { name: "Satürn" })[0]);
  expect(marked()).toEqual(["8", "9"]);
  fireEvent.keyDown(window, { key: "Escape" });
  expect(marked()).toEqual([]);
});
````

````diff
diff --git a/app/ui/chart/ChartSvg.test.tsx b/app/ui/chart/ChartSvg.test.tsx
index 8491f3a..f5ebfcf 100644
--- a/app/ui/chart/ChartSvg.test.tsx
+++ b/app/ui/chart/ChartSvg.test.tsx
@@ -287,6 +287,44 @@ test("without transits the drawing has no transit labels", () => {
   expect(container.querySelector(".chart-transit")).toBeNull();
 });
 
+test.each(STYLES)("%s: sign marks share the house layout, are plain text and not interactive", (style) => {
+  const marked: ChartData = {
+    ascSign: 0,
+    planets: [{ id: "Sun", sign: 0, degInSign: 10, retrograde: false }],
+    transits: [{ id: "Saturn", sign: 0, degInSign: 3, retrograde: true }],
+    marks: [
+      { sign: 0, text: "AL" },
+      { sign: 0, text: "A5" },
+      { sign: 7, text: "UL" },
+      // Çizilemeyen işaretler açıkça atlanır.
+      { sign: 12, text: "A9" },
+      { sign: 3, text: "" },
+    ],
+  };
+  const { container } = render(<ChartSvg data={marked} style={style} labels={labels} />);
+  const marks = [...container.querySelectorAll(".chart-mark")];
+  expect(marks.map((m) => m.textContent)).toEqual(["AL", "A5", "UL"]);
+  expect(marks.map((m) => m.getAttribute("data-house"))).toEqual(["1", "1", "8"]);
+  for (const m of marks) {
+    expect(m.closest("[role=button]")).toBeNull();
+    expect(m).toHaveAttribute("aria-hidden", "true");
+  }
+  // Aynı hücredeki dört etiket (natal, transit, iki işaret) ayrı yerlerdedir.
+  const sun = container.querySelector('[data-graha="Sun"]')!;
+  const spots = [
+    `${sun.getAttribute("data-x")},${sun.getAttribute("data-y")}`,
+    ...[container.querySelector('[data-transit="Saturn"]')!, marks[0], marks[1]].map((e) => `${e.getAttribute("x")},${e.getAttribute("y")}`),
+  ];
+  expect(new Set(spots).size).toBe(4);
+  // İşaretler evlerin erişilebilir adını değiştirmez.
+  expect(screen.getAllByRole("button", { name: /^House / })).toHaveLength(12);
+});
+
+test("without marks the drawing has no mark labels", () => {
+  const { container } = render(<ChartSvg data={data} style="north" labels={labels} />);
+  expect(container.querySelector(".chart-mark")).toBeNull();
+});
+
 test("a static drawing is one image: no buttons or tab stops, the same cells and marks", () => {
   const { container } = render(<ChartSvg data={data} style="north" labels={labels} interactive={false} />);
   expect(screen.getByRole("img", { name: "Chart" })).toBe(container.querySelector("svg.chart"));
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index 697ad29..c65d68d 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -37,6 +37,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "ashtakavarga",
     "drishti",
     "jaimini_karakas",
+    "arudha",
   ]);
   expect(isKnownPanelKind("chart")).toBe(true);
   expect(isKnownPanelKind("drishti")).toBe(true);
@@ -44,7 +45,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
diff --git a/app/ui/panels/ChartPanel.test.tsx b/app/ui/panels/ChartPanel.test.tsx
index cbf4137..0231e1a 100644
--- a/app/ui/panels/ChartPanel.test.tsx
+++ b/app/ui/panels/ChartPanel.test.tsx
@@ -5,7 +5,7 @@ import { fixture, unknownTimeOpened } from "../test/fixture";
 import { renderWithStores } from "../test/render";
 import { ChartPanel } from "./ChartPanel";
 
-vi.mock("../api", () => ({ api: { transitsAt: vi.fn() } }));
+vi.mock("../api", () => ({ api: { transitsAt: vi.fn(), computeJaimini: vi.fn() } }));
 
 const LAYOUT = (settings: Record<string, unknown>) => ({
   schema_version: 1,
@@ -100,3 +100,64 @@ test("the screen reader transit list says which transits are retrograde, never t
   expect(items).toHaveLength(9);
   expect(items.filter((item) => item.endsWith(" retro"))).toEqual([expect.stringMatching(/^Satürn .*° retro$/)]);
 });
+
+test("the arudha marks toggle is saved with the layout and asks for nothing while off", async () => {
+  const { persistence, container } = renderPanel({ varga: "D1", style: "north" });
+  const marks = screen.getByRole("checkbox", { name: "Arudha padalar" });
+  expect(marks).not.toBeChecked();
+  expect(marks).toBeEnabled();
+  expect(api.computeJaimini).not.toHaveBeenCalled();
+  expect(container.querySelector(".chart-mark")).toBeNull();
+  vi.mocked(api.computeJaimini).mockResolvedValue({ status: "ok", data: fixture().jaimini });
+  fireEvent.click(marks);
+  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
+  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0];
+  expect(saved.settings).toEqual({ varga: "D1", style: "north", arudha_marks: true });
+});
+
+test("with the marks on, D1 writes the twelve padas in their signs and lists them for screen readers", async () => {
+  vi.mocked(api.computeJaimini).mockClear();
+  vi.mocked(api.computeJaimini).mockResolvedValue({ status: "ok", data: fixture().jaimini });
+  const { container } = renderPanel({ varga: "D1", style: "south", arudha_marks: true });
+  await waitFor(() => expect(container.querySelectorAll(".chart-mark")).toHaveLength(12));
+  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
+  expect(vi.mocked(api.computeJaimini).mock.calls[0][0].time_known).toBe(true);
+  // Delhi fikstürü: AL Balık (10. ev), A7 Aslan (3. ev), UL Kova (9. ev); Yay'da üç pada.
+  const houseOf = (text: string) => container.querySelector(`[data-mark="${text}"]`)!.getAttribute("data-house");
+  expect([houseOf("AL"), houseOf("A7"), houseOf("UL")]).toEqual(["10", "3", "9"]);
+  expect([houseOf("A4"), houseOf("A5"), houseOf("A11")]).toEqual(["7", "7", "7"]);
+  const list = screen.getByText(/^Arudha padalar: /);
+  expect(list).toHaveClass("visually-hidden");
+  expect(list).toHaveTextContent("Arudha padalar: AL Bal, A2 Ter, A3 Koç, A4 Yay, A5 Yay, A6 Kov, A7 Asl, A8 Koç, A9 İki, A10 Boğ, A11 Yay, UL Kov");
+  // Grahalar yerinde durur ve düğme olmayı sürdürür.
+  expect(screen.getByRole("button", { name: "Gü, Yay 16°" })).toBeInTheDocument();
+});
+
+test("marks are written on D1 only and never without a birth time; the checkbox says why", async () => {
+  vi.mocked(api.computeJaimini).mockClear();
+  const d9 = renderPanel({ varga: "D9", arudha_marks: true });
+  const marks = screen.getByRole("checkbox", { name: "Arudha padalar" });
+  expect(marks).toBeDisabled();
+  expect(marks).toHaveAccessibleDescription("Arudha padalar yalnızca D1'de yazılır.");
+  expect(d9.container.querySelector(".chart-mark")).toBeNull();
+  d9.unmount();
+  const unknown = renderPanel({ varga: "D1", arudha_marks: true }, unknownTimeOpened());
+  const off = screen.getByRole("checkbox", { name: "Arudha padalar" });
+  expect(off).toBeDisabled();
+  expect(off).toHaveAccessibleDescription("Doğum saati bilinmiyor: arudha padalar lagnaya bağlıdır ve yazılamaz.");
+  expect(unknown.container.querySelector(".chart-mark")).toBeNull();
+  expect(api.computeJaimini).not.toHaveBeenCalled();
+});
+
+test("a failed pada request leaves the chart drawn and offers a retry", async () => {
+  vi.mocked(api.computeJaimini).mockReset();
+  vi.mocked(api.computeJaimini).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
+  vi.mocked(api.computeJaimini).mockResolvedValue({ status: "ok", data: fixture().jaimini });
+  const { container } = renderPanel({ varga: "D1", arudha_marks: true });
+  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
+  expect(screen.getByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
+  expect(container.querySelector(".chart-mark")).toBeNull();
+  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
+  await waitFor(() => expect(container.querySelectorAll(".chart-mark")).toHaveLength(12));
+  expect(screen.queryByRole("alert")).toBeNull();
+});
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index 764900e..238a674 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -8,6 +8,7 @@ import { NotesPanel } from "../notes/NotesPanel";
 import { fixture } from "../test/fixture";
 import { renderWithStores } from "../test/render";
 import { TransitTablePanel } from "../transit/TransitTablePanel";
+import { ArudhaPanel } from "./ArudhaPanel";
 import { AshtakavargaPanel } from "./AshtakavargaPanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel } from "./ChartPanel";
@@ -42,6 +43,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.ashtakavarga.component).toBe(AshtakavargaPanel);
   expect(PANEL_REGISTRY.drishti.component).toBe(DrishtiPanel);
   expect(PANEL_REGISTRY.jaimini_karakas.component).toBe(JaiminiKarakasPanel);
+  expect(PANEL_REGISTRY.arudha.component).toBe(ArudhaPanel);
   // Kataloğun her türü uygulanmıştır: yer tutucu yoktur.
   for (const kind of PANEL_KINDS) expect(PANEL_REGISTRY[kind].component, kind).not.toBe(UnavailablePanel);
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
pnpm --dir app exec vitest run ui/panels/ArudhaPanel.test.tsx ui/chart/ChartSvg.test.tsx ui/panels/ChartPanel.test.tsx ui/layout/model.test.ts ui/panels/panels.test.tsx 2>&1 | grep -E "Test Files|Tests "
```
Expected: `Test Files  5 failed (5)`, `Tests  9 failed | 60 passed (69)` (yeni test dosyası içe aktarımda düşer).

- [ ] **Step 3: Arudha paneli, kayıt ve metinler**

**Yeni dosya:** `app/ui/panels/ArudhaPanel.tsx`

````tsx
// Arudha padalar paneli (ileri modüller spec'i §5): A1 (AL) … A12 (UL) için padanın burcu ve
// evin hesapta kullanılan lordu. Değerler `compute_jaimini`'den gelir; burada hesap yoktur (C4).
// İstisna `*` ve dipnotla gösterilir; renk anlam taşımaz (C9). Doğum saati bilinmiyorsa padalar
// Rust'tan boş gelir (K2) ve panel bunu bir satırla söyler.
import { useTranslation } from "react-i18next";
import type { JaiminiReport, OpenedClient } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import { useJaimini } from "../modules/useJaimini";
import { useSession } from "../session/store";
import { arudhaLabel } from "./labels";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export const EXCEPTION_MARK = "*";

export function ArudhaPanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const module = useJaimini(opened);
  const phase = usePanelLoadPhase(module.loading);
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (module.data === null) return phase === "skeleton" ? <PanelSkeleton /> : null;
  if (module.data.arudhas === null) return <p className="panel-notice">{t("arudha.timeUnknown")}</p>;
  return <Padas report={module.data} padas={module.data.arudhas} opened={opened} />;
}

function Padas({ report, padas, opened }: { report: JaiminiReport; padas: NonNullable<JaiminiReport["arudhas"]>; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const lagna = signIndex(opened.chart.lagna.sign);
  // Ev vurgusu D1 evidir: lagna burcundan sayılan burç (gösterim eşlemesi, hesap değil).
  const activeSign = active?.type === "sign" ? active.sign : active?.type === "house" ? (lagna + active.house - 1) % 12 : null;
  const anyException = padas.some((p) => p.exception);

  return (
    <div className="module-panel">
      <table className="module-table">
        <caption className="visually-hidden">{t("arudha.caption")}</caption>
        <thead>
          <tr>
            <th scope="col">{t("arudha.pada")}</th>
            <th scope="col">{t("table.sign")}</th>
            <th scope="col">{t("arudha.lord")}</th>
          </tr>
        </thead>
        <tbody>
          {padas.map((p) => {
            const sign = signIndex(p.sign);
            const on = activeSign === sign || (active?.type === "graha" && active.id === p.lord);
            const label = arudhaLabel(p.house);
            return (
              <tr key={p.house} data-pada={p.house} className={on ? "is-highlighted" : undefined}>
                <th scope="row">
                  {`A${p.house}`}
                  {label !== `A${p.house}` && <> · {label}</>}
                </th>
                <td>
                  <PinButton target={{ type: "sign", sign }}>{t(`sign.${sign}`)}</PinButton>
                  {p.exception && (
                    <>
                      <span aria-hidden="true">{EXCEPTION_MARK}</span>
                      <span className="visually-hidden"> {t("arudha.exception")}</span>
                    </>
                  )}
                </td>
                <td>
                  <PinButton target={{ type: "graha", id: p.lord }}>{t(`grahaName.${p.lord}`)}</PinButton>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {anyException && <p className="panel-message">{t("arudha.exceptionNote", { mark: EXCEPTION_MARK })}</p>}
      <p className="panel-message">{t(report.co_lords ? "arudha.coLordsOn" : "arudha.coLordsOff")}</p>
      <p className="panel-message">{t("jaimini.method", { method: report.method })}</p>
    </div>
  );
}
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 8448579..e79c5e6 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -32,6 +32,10 @@ export const en: Messages = {
     transitRing: "Transit ring",
     transitRingOnlyD1: "The transit ring is drawn in D1 only.",
     transitList: "Transit: {{list}}",
+    arudhaMarks: "Arudha padas",
+    arudhaMarksOnlyD1: "Arudha padas are written on D1 only.",
+    arudhaMarksTimeUnknown: "Birth time unknown: arudha padas depend on the lagna and cannot be written.",
+    arudhaList: "Arudha padas: {{list}}",
     title: "{{varga}} chart",
     lagna: "As",
     house: "House {{house}}, {{sign}}",
@@ -60,6 +64,7 @@ export const en: Messages = {
     ashtakavarga: "Ashtakavarga",
     drishti: "Aspects",
     jaimini_karakas: "Chara karaka",
+    arudha: "Arudha padas",
     unknown: "Unknown panel",
     loading: "Loading",
     unavailable: "This panel arrives in a later version.",
@@ -131,6 +136,16 @@ export const en: Messages = {
     method: "Method: {{method}}",
     timeUnknown: "Birth time unknown: the Moon's degree is approximate (≈); the karaka order and the karakamsha may differ.",
   },
+  arudha: {
+    caption: "Arudha padas: the pada of each of the twelve houses, its sign and the lord of the house",
+    pada: "Pada",
+    lord: "Lord of the house",
+    exception: "exception",
+    exceptionNote: "{{mark}} Exception: the computed sign was the house itself or its 7th; the pada is the 10th from it.",
+    coLordsOn: "Jaimini co-lords are on: the lord of Scorpio and of Aquarius is the stronger co-lord.",
+    coLordsOff: "Jaimini co-lords are off: Mars is the lord of Scorpio, Saturn the lord of Aquarius.",
+    timeUnknown: "Birth time unknown: arudha padas depend on the lagna and are not shown.",
+  },
   table: {
     graha: "Graha",
     sign: "Sign",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index da84ddb..26ebeab 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -32,6 +32,10 @@ export const fr: Messages = {
     transitRing: "Anneau des transits",
     transitRingOnlyD1: "L’anneau des transits n’est tracé qu’en D1.",
     transitList: "Transit : {{list}}",
+    arudhaMarks: "Arudha padas",
+    arudhaMarksOnlyD1: "Les arudha padas ne sont inscrits que sur D1.",
+    arudhaMarksTimeUnknown: "Heure de naissance inconnue : les arudha padas dépendent du lagna et ne peuvent pas être inscrits.",
+    arudhaList: "Arudha padas : {{list}}",
     title: "Carte {{varga}}",
     lagna: "As",
     house: "Maison {{house}}, {{sign}}",
@@ -60,6 +64,7 @@ export const fr: Messages = {
     ashtakavarga: "Ashtakavarga",
     drishti: "Aspects",
     jaimini_karakas: "Chara karaka",
+    arudha: "Arudha padas",
     unknown: "Panneau inconnu",
     loading: "Chargement",
     unavailable: "Ce panneau arrivera dans une version ultérieure.",
@@ -131,6 +136,16 @@ export const fr: Messages = {
     method: "Méthode : {{method}}",
     timeUnknown: "Heure de naissance inconnue : le degré de la Lune est approximatif (≈) ; l'ordre des karakas et le karakamsha peuvent différer.",
   },
+  arudha: {
+    caption: "Arudha padas : le pada de chacune des douze maisons, son signe et le maître de la maison",
+    pada: "Pada",
+    lord: "Maître de la maison",
+    exception: "exception",
+    exceptionNote: "{{mark}} Exception : le signe obtenu était la maison elle-même ou sa 7e ; le pada est le 10e à partir de ce signe.",
+    coLordsOn: "Co-maîtres Jaimini activés : le maître du Scorpion et du Verseau est le co-maître le plus fort.",
+    coLordsOff: "Co-maîtres Jaimini désactivés : Mars est le maître du Scorpion, Saturne celui du Verseau.",
+    timeUnknown: "Heure de naissance inconnue : les arudha padas dépendent du lagna et ne sont pas affichés.",
+  },
   table: {
     graha: "Graha",
     sign: "Signe",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 8e021c1..6e274f2 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -31,6 +31,10 @@ export const tr = {
     transitRing: "Transit halkası",
     transitRingOnlyD1: "Transit halkası yalnızca D1'de çizilir.",
     transitList: "Transit: {{list}}",
+    arudhaMarks: "Arudha padalar",
+    arudhaMarksOnlyD1: "Arudha padalar yalnızca D1'de yazılır.",
+    arudhaMarksTimeUnknown: "Doğum saati bilinmiyor: arudha padalar lagnaya bağlıdır ve yazılamaz.",
+    arudhaList: "Arudha padalar: {{list}}",
     title: "{{varga}} haritası",
     lagna: "Lg",
     house: "{{house}}. ev, {{sign}}",
@@ -59,6 +63,7 @@ export const tr = {
     ashtakavarga: "Ashtakavarga",
     drishti: "Bakışlar",
     jaimini_karakas: "Chara karaka",
+    arudha: "Arudha padalar",
     unknown: "Bilinmeyen panel",
     loading: "Yükleniyor",
     unavailable: "Bu panel sonraki bir sürümde gelecek.",
@@ -130,6 +135,16 @@ export const tr = {
     method: "Yöntem: {{method}}",
     timeUnknown: "Doğum saati bilinmiyor: Ay'ın derecesi yaklaşıktır (≈); karaka sırası ve karakamsha değişebilir.",
   },
+  arudha: {
+    caption: "Arudha padalar: on iki evin padası, burcu ve evin lordu",
+    pada: "Pada",
+    lord: "Evin lordu",
+    exception: "istisna",
+    exceptionNote: "{{mark}} İstisna: hesaplanan burç evin kendisi ya da 7. burcuydu; pada ondan 10. burçtur.",
+    coLordsOn: "Jaimini eş lordları açık: Akrep ve Kova'nın lordu, güçlü olan eş lorddur.",
+    coLordsOff: "Jaimini eş lordları devre dışı: Akrep'in lordu Mars, Kova'nın lordu Satürn'dür.",
+    timeUnknown: "Doğum saati bilinmiyor: arudha padalar lagnaya bağlı olduğu için gösterilmiyor.",
+  },
   table: {
     graha: "Graha",
     sign: "Burç",
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index 2ced278..7e1e8a7 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -14,6 +14,7 @@ export const PANEL_KINDS = [
   "ashtakavarga",
   "drishti",
   "jaimini_karakas",
+  "arudha",
 ] as const;
 export type KnownPanelKind = (typeof PANEL_KINDS)[number];
 
@@ -27,6 +28,7 @@ export const MODULE_PANELS = [
   { kind: "drishti", w: 6, h: 10 },
   // Jaimini çekirdeği (Plan 15).
   { kind: "jaimini_karakas", w: 5, h: 14 },
+  { kind: "arudha", w: 4, h: 12 },
 ] as const satisfies readonly { kind: KnownPanelKind; w: number; h: number }[];
 export type ModulePanelKind = (typeof MODULE_PANELS)[number]["kind"];
 
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index 7b6caf5..2003f2f 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -3,6 +3,7 @@ import { DashaStripPanel } from "../dasha/DashaStripPanel";
 import { DashaTreePanel } from "../dasha/DashaTreePanel";
 import { isKnownPanelKind, type KnownPanelKind } from "../layout/model";
 import { NotesPanel } from "../notes/NotesPanel";
+import { ArudhaPanel } from "./ArudhaPanel";
 import { AshtakavargaPanel } from "./AshtakavargaPanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel, chartVarga } from "./ChartPanel";
@@ -28,6 +29,7 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   drishti: { titleKey: "panel.drishti", component: DrishtiPanel },
   // Jaimini çekirdeği (Plan 15).
   jaimini_karakas: { titleKey: "panel.jaimini_karakas", component: JaiminiKarakasPanel },
+  arudha: { titleKey: "panel.arudha", component: ArudhaPanel },
 };
 
 const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };
````

- [ ] **Step 4: Harita işaretleri**

`ChartSvg` işaretleri gezegen ve transit etiketlerinin ardından aynı hücre yerleşimine ekler (üçü de `layoutLabels`'a tek liste olarak girer; etiketler birbirine değmez). İşaretler düğme değildir. `ChartPanel` raporu yalnızca kutu açıkken, D1'de ve doğum saati biliniyorken ister; aksi durumda kutu kapalıdır ve nedeni kutuya bağlıdır.

````diff
diff --git a/app/ui/chart/ChartSvg.tsx b/app/ui/chart/ChartSvg.tsx
index ebfaa01..cac5e01 100644
--- a/app/ui/chart/ChartSvg.tsx
+++ b/app/ui/chart/ChartSvg.tsx
@@ -1,12 +1,13 @@
 // Kuzey Hint, Güney Hint ve daire haritası. Yalnızca belirteç sınıfları kullanılır;
 // gezegenlere renk verilmez. Retro "R", yanma "c", vargottama altı çizili (spec §2.3).
+// Transit etiketleri parantezli, burç işaretleri (arudha padalar) düz kısa metindir.
 // svg `role="group"`dur: `img` alt öğeleri sunumsal yapar ve ev/gezegen düğmelerini
 // ekran okuyucudan gizlerdi (spec §6).
 import { useMemo, type KeyboardEvent } from "react";
 import { sameTarget, type HighlightTarget } from "../highlight/target";
 import "./chart.css";
 import { chartCells, layoutLabels, pathOf, VIEWBOX, type ChartCell, type LabelLayout } from "./geometry";
-import { RETRO_MARK, showsRetro, type ChartData, type ChartLabels, type ChartPlanet, type ChartStyle } from "./types";
+import { RETRO_MARK, showsRetro, type ChartData, type ChartLabels, type ChartMark, type ChartPlanet, type ChartStyle } from "./types";
 
 export interface ChartSvgProps {
   data: ChartData;
@@ -34,6 +35,11 @@ function drawable(p: ChartPlanet): boolean {
   return Number.isInteger(p.sign) && p.sign >= 0 && p.sign <= 11 && Number.isFinite(p.degInSign);
 }
 
+/** Çizilebilir işaret: burç 0…11 tamsayı ve metin boş değil. */
+function drawableMark(m: ChartMark): boolean {
+  return Number.isInteger(m.sign) && m.sign >= 0 && m.sign <= 11 && m.text !== "";
+}
+
 function labelText(p: ChartPlanet, graha: ChartLabels["graha"]): string {
   return `${graha[p.id]}${showsRetro(p.id, p.retrograde) ? RETRO_MARK : ""}${p.combust ? "c" : ""}`;
 }
@@ -73,15 +79,17 @@ export function ChartSvg({ data, style, labels, highlight = null, pressed = high
     };
     const natal = group(data.planets);
     const transit = group(data.transits ?? []);
+    const marked = (data.marks ?? []).filter(drawableMark);
     return cells.map((cell) => {
       const planets = natal.get(cell.sign) ?? [];
       const transits = transit.get(cell.sign) ?? [];
-      const texts = [...planets.map((p) => labelText(p, graha)), ...transits.map((p) => transitText(p, graha))];
+      const marks = marked.filter((m) => m.sign === cell.sign);
+      const texts = [...planets.map((p) => labelText(p, graha)), ...transits.map((p) => transitText(p, graha)), ...marks.map((m) => m.text)];
       const layout: LabelLayout = layoutLabels(cell, texts, hasLagna && cell.lagnaAnchor ? lagna : "");
-      return { cell, planets, transits, layout };
+      return { cell, planets, transits, marks, layout };
     });
     // textKey, graha ve lagna'yı kapsar.
-  }, [cells, data.planets, data.transits, textKey, hasLagna]);
+  }, [cells, data.planets, data.transits, data.marks, textKey, hasLagna]);
 
   const activate = (target: HighlightTarget) => (e: KeyboardEvent) => {
     if (e.key === "Enter" || e.key === " ") {
@@ -207,6 +215,27 @@ export function ChartSvg({ data, style, labels, highlight = null, pressed = high
           );
         }),
       )}
+      {placed.map(({ cell, planets, transits, marks, layout }) =>
+        marks.map((m, i) => {
+          const { x, y } = layout.boxes[planets.length + transits.length + i];
+          return (
+            <text
+              key={`mark-${m.text}`}
+              className="chart-mark"
+              data-mark={m.text}
+              data-house={cell.house}
+              x={x}
+              y={y}
+              textAnchor="middle"
+              dominantBaseline="central"
+              style={{ fontSize: `${layout.fontSize}px` }}
+              aria-hidden="true"
+            >
+              {m.text}
+            </text>
+          );
+        }),
+      )}
     </svg>
   );
 }
diff --git a/app/ui/chart/chart.css b/app/ui/chart/chart.css
index f24cd5f..0e12c9d 100644
--- a/app/ui/chart/chart.css
+++ b/app/ui/chart/chart.css
@@ -76,3 +76,9 @@
   font-style: italic;
   pointer-events: none;
 }
+
+/* Burç işaretleri (arudha padalar): kısa metin etiketi; anlamı metnin kendisi taşır (C9). */
+.chart-mark {
+  fill: var(--text-2);
+  pointer-events: none;
+}
diff --git a/app/ui/chart/types.ts b/app/ui/chart/types.ts
index 8f78cd0..19dd7d5 100644
--- a/app/ui/chart/types.ts
+++ b/app/ui/chart/types.ts
@@ -20,12 +20,21 @@ export interface ChartPlanet {
   vargottama?: boolean;
 }
 
+/** Bir burca yazılan kısa işaret (ör. arudha pada "AL", "A2"); etkileşimsizdir. */
+export interface ChartMark {
+  /** 0 = Mesha (Koç) … 11 = Meena (Balık). */
+  sign: number;
+  text: string;
+}
+
 export interface ChartData {
   /** Lagna burcu; henüz hesaplanmıyorsa `null` (1. ev Koç kabul edilir, çağıran uyarı gösterir). */
   ascSign: number | null;
   planets: ChartPlanet[];
   /** Transit halkası (yalnızca D1, Plan 11 D9): etkileşimsiz, parantezli kısaltmalar. */
   transits?: ChartPlanet[];
+  /** Burç işaretleri (yalnızca D1; ileri modüller spec'i §5 `arudha_marks`). */
+  marks?: ChartMark[];
 }
 
 export type ChartStyle = "north" | "south" | "circle";
diff --git a/app/ui/panels/ChartPanel.tsx b/app/ui/panels/ChartPanel.tsx
index 20707f4..356f449 100644
--- a/app/ui/panels/ChartPanel.tsx
+++ b/app/ui/panels/ChartPanel.tsx
@@ -1,16 +1,18 @@
 import { useId } from "react";
 import { useTranslation } from "react-i18next";
 import type { Varga } from "../bindings";
-import { chartDataFromChart, isVarga, transitPlanets, VARGA_NAMES, VARGAS } from "../chart/adapter";
+import { chartDataFromChart, isVarga, signIndex, transitPlanets, VARGA_NAMES, VARGAS } from "../chart/adapter";
 import { ChartSvg, type ChartSvgProps } from "../chart/ChartSvg";
-import { CHART_STYLES, isChartStyle, showsRetro, type ChartData } from "../chart/types";
+import { CHART_STYLES, isChartStyle, showsRetro, type ChartData, type ChartMark } from "../chart/types";
 import { useHighlight } from "../highlight/store";
+import { useJaimini } from "../modules/useJaimini";
 import { timeUnknown } from "../session/birth";
 import { useSession } from "../session/store";
 import { useTransit } from "../transit/store";
 import { useLayoutStore } from "../view/store";
-import { chartLabels } from "./labels";
+import { arudhaLabel, chartLabels, signAbbreviations } from "./labels";
 import "./panels.css";
+import { PanelError } from "./states";
 import type { PanelProps } from "./types";
 
 export function chartVarga(settings: Record<string, unknown>): Varga {
@@ -20,6 +22,8 @@ export function chartVarga(settings: Record<string, unknown>): Varga {
 /**
  * Harita paneli: açık haritanın `settings.varga` vargası, `settings.style` biçiminde.
  * Varga ve biçim seçimi panel ayarına yazılır; düzenle birlikte kaydedilir.
+ * `settings.arudha_marks` açıkken (yalnızca D1, doğum saati biliniyorsa) arudha padalar
+ * burçlarına "AL", "A2" … "UL" olarak yazılır; padalar `compute_jaimini`'den gelir.
  */
 export function ChartPanel({ panel }: PanelProps) {
   const { t } = useTranslation();
@@ -31,10 +35,18 @@ export function ChartPanel({ panel }: PanelProps) {
   const style = isChartStyle(panel.settings.style) ? panel.settings.style : "north";
   const ringOn = panel.settings.transit_ring === true;
   const ringDrawn = ringOn && varga === "D1";
+  const marksOn = panel.settings.arudha_marks === true;
+  const unknown = opened !== null && timeUnknown(opened);
+  // Padalar lagnaya bağlıdır: saat bilinmiyorsa istenmez (Rust zaten boş döner, K2).
+  const marksOff = varga !== "D1" ? "arudhaMarksOnlyD1" : unknown ? "arudhaMarksTimeUnknown" : null;
+  const jaimini = useJaimini(opened, marksOn && marksOff === null);
 
   if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
-  const unknown = timeUnknown(opened);
-  const data = chartDataFromChart(opened.chart, varga, { showLagna: !unknown });
+  const base = chartDataFromChart(opened.chart, varga, { showLagna: !unknown });
+  const marks: ChartMark[] = (jaimini.data?.arudhas ?? []).map((p) => ({ sign: signIndex(p.sign), text: arudhaLabel(p.house) }));
+  const data = base && marks.length > 0 ? { ...base, marks } : base;
+  const signs = signAbbreviations(t);
+  const markList = marks.map((m) => `${m.text} ${signs[m.sign]}`).join(", ");
   const svgProps = {
     style,
     labels: chartLabels(t, varga),
@@ -85,7 +97,23 @@ export function ChartPanel({ panel }: PanelProps) {
             {t("chart.transitRingOnlyD1")}
           </span>
         )}
+        <label className="chart-ring-toggle">
+          <input
+            type="checkbox"
+            checked={marksOn}
+            disabled={marksOff !== null}
+            aria-describedby={marksOff !== null ? `${id}-marks` : undefined}
+            onChange={(e) => setPanelSettings(panel.id, { arudha_marks: e.target.checked })}
+          />
+          {t("chart.arudhaMarks")}
+        </label>
+        {marksOff !== null && (
+          <span id={`${id}-marks`} className="visually-hidden">
+            {t(`chart.${marksOff}`)}
+          </span>
+        )}
       </div>
+      {jaimini.error && <PanelError error={jaimini.error} onRetry={jaimini.retry} />}
       {unknown && (
         <p className="panel-notice" role="note">
           {t("chart.timeUnknown")}
@@ -100,6 +128,8 @@ export function ChartPanel({ panel }: PanelProps) {
       ) : (
         <p className="panel-message">{t("chart.vargaMissing", { varga })}</p>
       )}
+      {/* Çizimdeki işaretler `aria-hidden`'dır: padalar burada sözcükle okunur. */}
+      {data && marks.length > 0 && <p className="visually-hidden">{t("chart.arudhaList", { list: markList })}</p>}
     </div>
   );
 }
````

- [ ] **Step 5: Kapılar**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
git status --short app/ui/chart/__snapshots__
```
Expected: `Test Files  67 passed (67)`, `Tests  599 passed (599)` (+14: `ArudhaPanel` 6, `ChartSvg` 4, `ChartPanel` 4); `✓ built`; son komut çıktısız (üç harita anlık görüntüsü değişmedi: işaretler varsayılan olarak kapalıdır).

- [ ] **Step 6: Commit**

```bash
git add app/ui/panels app/ui/chart app/ui/layout app/ui/i18n
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): arudha pada panel; optional arudha marks on the D1 chart"
```
Expected: iki `grep` çıktısız.

---

### Task 8: Argala paneli

Spec §5 "Arayüz", `argala` satırı; D10, D17. Ayar: `reference` (`"lagna"` ya da `"selected"`). İçerik: referans burç için dört çift; her çiftte argala evi, argala grahaları, engel evi, engel grahaları ve durum metni ("geçerli", "çekişmeli", "engelli", "yok"); 3. ev kötücül argalası varsa bir satır. Durumlar: harita yok; yükleniyor; hata; dolu; "seçili burç" seçiliyken hiçbir şey sabitlenmemiş (tek satır yönlendirme ve on iki burç düğmesi); saat bilinmiyor ve referans lagna (tek satır açıklama).

**Files:**
- Create: `app/ui/panels/ArgalaPanel.tsx`, `app/ui/panels/ArgalaPanel.test.tsx`
- Modify: `app/ui/layout/model.ts`, `app/ui/panels/{registry.ts,panels.css}`, `app/ui/i18n/{tr,en,fr}.ts`
- Test: `app/ui/panels/ArgalaPanel.test.tsx`, `app/ui/layout/model.test.ts`, `app/ui/panels/panels.test.tsx`

**Interfaces:**
- Consumes: `useJaimini`, `PinButton` (Task 6); `ArgalaPair`, `JaiminiReport["argala"]`; `useHighlight().pinned`; `useLayoutStore().setPanelSettings`; `timeUnknown`; `signIndex`; i18n `chart.lagna`, `graha.*`, `grahaName.*`, `sign.*`, `jaimini.method`; CSS `.av-lagna`.
- Produces: `ArgalaPanel`, `ARGALA_REFERENCES`, `argalaReference(settings): "lagna" | "selected"`, `selectedSign(pinned: HighlightTarget | null, opened: OpenedClient, lagna: number | null): number | null`; `layout/model.ts`: `argala` (6 × 9); i18n `panel.argala`, `argala.*`; CSS `.argala-signs`, `.argala-reference`.

- [ ] **Step 1: Başarısız testleri yaz**

**Yeni dosya:** `app/ui/panels/ArgalaPanel.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeJaimini: vi.fn() } }));

import { api, type Result } from "../api";
import type { JaiminiReport } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { ArgalaPanel, argalaReference, selectedSign } from "./ArgalaPanel";

const ok = (report: JaiminiReport = fixture().jaimini): Result<JaiminiReport> => ({ status: "ok", data: report });

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  const panel: Panel = { id: "ag", kind: "argala", x: 0, y: 0, w: 6, h: 9, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<ArgalaPanel panel={panel} />, { layout, opened });
}

const rows = () => [...document.querySelectorAll("tbody tr")].map((r) => [...r.querySelectorAll("th, td")].map((c) => c.textContent));
const signs = () => screen.getByRole("group", { name: "Referans burcu seç" });

beforeEach(() => {
  vi.mocked(api.computeJaimini).mockReset();
  vi.mocked(api.computeJaimini).mockImplementation(async () => ok());
});

test("by default the reference is the lagna sign: four pairs, two graha lists and the status as text", async () => {
  renderPanel();
  await screen.findByRole("table", { name: "Argala, referans İki: argala veren evler, engelleyen evler ve durum" });
  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
  expect(screen.getByRole("combobox", { name: "Referans" })).toHaveDisplayValue("Lagna");
  expect(screen.getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Argala evi", "Argala grahaları", "Engel evi", "Engel grahaları", "Durum"]);
  // Delhi fikstürü, İkizler'den: 2 Yengeç (Rahu) ↔ 12 Boğa; 4 Başak ↔ 10 Balık; 11 Koç (Jüpiter,
  // Satürn) ↔ 3 Aslan; 5 Terazi (Ay) ↔ 9 Kova (Mars).
  expect(rows()).toEqual([
    ["2", "Ra", "12", "—", "geçerli"],
    ["4", "—", "10", "—", "yok"],
    ["11", "Jü Sa", "3", "—", "geçerli"],
    ["5", "Ay", "9", "Ma", "çekişmeli"],
  ]);
  expect(screen.getByText("Referans: İki")).toHaveTextContent("Referans: İki Lg");
  // Kısaltmanın erişilebilir adı grahanın tam adıdır.
  expect(screen.getByRole("button", { name: "Jüpiter" })).toHaveTextContent("Jü");
  expect(screen.queryByRole("group", { name: "Referans burcu seç" })).toBeNull();
  expect(screen.queryByText(/kötücül argalası/)).toBeNull();
  expect(screen.getByText("Durum yalnızca graha sayısına bakar; güç karşılaştırması yapılmaz.")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: jaimini.core_v1")).toBeInTheDocument();
});

test("selected sign: nothing pinned asks for a sign; picking one shows its row", async () => {
  renderPanel({ reference: "selected" });
  expect(await screen.findByText(/^Bir burç seçin:/)).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
  expect(within(signs()).getAllByRole("button").map((b) => b.textContent)).toEqual(["Koç", "Boğ", "İki", "Yen", "Asl", "Baş", "Ter", "Akr", "Yay", "Oğl", "Kov", "Bal"]);
  const cancer = within(signs()).getByRole("button", { name: "Yen" });
  fireEvent.click(cancer);
  expect(cancer).toHaveAttribute("aria-pressed", "true");
  await screen.findByRole("table", { name: /^Argala, referans Yen:/ });
  // Yengeç'ten: 4 Terazi (Ay) ↔ 10 Koç (Jüpiter, Satürn) engelli; 5 Akrep (Venüs) ↔ 9 Balık.
  expect(rows()).toEqual([
    ["2", "—", "12", "—", "yok"],
    ["4", "Ay", "10", "Jü Sa", "engelli"],
    ["11", "—", "3", "—", "yok"],
    ["5", "Ve", "9", "—", "geçerli"],
  ]);
  expect(screen.getByText("Referans: Yen")).toHaveTextContent(/^Referans: Yen$/);
  expect(screen.queryByText(/^Bir burç seçin:/)).toBeNull();
  // Aynı komut yeniden çağrılmaz: on iki satır tek raporda gelir.
  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
  // Tablodaki bir graha sabitlenince referans o grahanın burcu olur: Venüs Akrep'te.
  fireEvent.click(screen.getByRole("button", { name: "Venüs" }));
  await screen.findByRole("table", { name: /^Argala, referans Akr:/ });
  fireEvent.keyDown(window, { key: "Escape" });
  expect(await screen.findByText(/^Bir burç seçin:/)).toBeInTheDocument();
});

test("the reference setting is saved with the layout; unknown values fall back to the lagna", async () => {
  const { persistence } = renderPanel();
  fireEvent.change(await screen.findByRole("combobox", { name: "Referans" }), { target: { value: "selected" } });
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  expect(persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ reference: "selected" });
  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
  expect([argalaReference({}), argalaReference({ reference: "moon" }), argalaReference({ reference: "selected" })]).toEqual(["lagna", "lagna", "selected"]);
});

test("unknown birth time: no lagna reference, but a selected sign still has its row", async () => {
  const report = fixture().jaimini;
  // Rust saat bilinmeyince padaları boş, satırları Koç'tan sıralı döner.
  report.arudhas = null;
  report.argala = [...report.argala.slice(10), ...report.argala.slice(0, 10)] as JaiminiReport["argala"];
  expect(report.argala[0].reference).toBe("Aries");
  vi.mocked(api.computeJaimini).mockResolvedValue(ok(report));
  const first = renderPanel({}, unknownTimeOpened());
  expect(await screen.findByText("Doğum saati bilinmiyor: lagna referansı yok. Referans olarak “Seçili burç”u kullanın.")).toBeInTheDocument();
  expect(vi.mocked(api.computeJaimini).mock.calls[0][0].time_known).toBe(false);
  expect(screen.queryByRole("table")).toBeNull();
  first.unmount();
  renderPanel({ reference: "selected" }, unknownTimeOpened());
  fireEvent.click(within(await screen.findByRole("group", { name: "Referans burcu seç" })).getByRole("button", { name: "İki" }));
  await screen.findByRole("table", { name: /^Argala, referans İki:/ });
  expect(rows()[0]).toEqual(["2", "Ra", "12", "—", "geçerli"]);
  // Lagna bilinmediği için "Lg" işareti yazılmaz.
  expect(screen.getByText("Referans: İki")).toHaveTextContent(/^Referans: İki$/);
});

test("three malefics in the third house are reported as text", async () => {
  const report = fixture().jaimini;
  report.argala[0].third_house_malefic_argala = true;
  vi.mocked(api.computeJaimini).mockResolvedValue(ok(report));
  renderPanel();
  expect(await screen.findByText("3. evde üç ya da daha çok doğal kötücül var: 3. ev kötücül argalası.")).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeJaimini).not.toHaveBeenCalled();
});

test("an error is shown in the panel and retry asks again", async () => {
  vi.mocked(api.computeJaimini).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await screen.findByRole("table");
  expect(api.computeJaimini).toHaveBeenCalledTimes(2);
});

test("loading: nothing under a second, then a skeleton, then the table", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<JaiminiReport>) => void;
    vi.mocked(api.computeJaimini).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
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

test("helper: the sign of the pinned target", () => {
  const opened = fixture().opened;
  expect(selectedSign(null, opened, 2)).toBeNull();
  expect(selectedSign({ type: "sign", sign: 7 }, opened, 2)).toBe(7);
  // Ev: lagna burcundan sayılır; lagna yoksa haritanın çizdiği gibi Koç'tan.
  expect(selectedSign({ type: "house", house: 1 }, opened, 2)).toBe(2);
  expect(selectedSign({ type: "house", house: 12 }, opened, 2)).toBe(1);
  expect(selectedSign({ type: "house", house: 4 }, opened, null)).toBe(3);
  // Graha: haritadaki burcu (Ay Terazi'de).
  expect(selectedSign({ type: "graha", id: "Moon" }, opened, 2)).toBe(6);
});
````

````diff
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index c65d68d..6b43845 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -38,6 +38,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "drishti",
     "jaimini_karakas",
     "arudha",
+    "argala",
   ]);
   expect(isKnownPanelKind("chart")).toBe(true);
   expect(isKnownPanelKind("drishti")).toBe(true);
@@ -45,7 +46,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index 238a674..78d899d 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -8,6 +8,7 @@ import { NotesPanel } from "../notes/NotesPanel";
 import { fixture } from "../test/fixture";
 import { renderWithStores } from "../test/render";
 import { TransitTablePanel } from "../transit/TransitTablePanel";
+import { ArgalaPanel } from "./ArgalaPanel";
 import { ArudhaPanel } from "./ArudhaPanel";
 import { AshtakavargaPanel } from "./AshtakavargaPanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
@@ -44,6 +45,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.drishti.component).toBe(DrishtiPanel);
   expect(PANEL_REGISTRY.jaimini_karakas.component).toBe(JaiminiKarakasPanel);
   expect(PANEL_REGISTRY.arudha.component).toBe(ArudhaPanel);
+  expect(PANEL_REGISTRY.argala.component).toBe(ArgalaPanel);
   // Kataloğun her türü uygulanmıştır: yer tutucu yoktur.
   for (const kind of PANEL_KINDS) expect(PANEL_REGISTRY[kind].component, kind).not.toBe(UnavailablePanel);
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
pnpm --dir app exec vitest run ui/panels/ArgalaPanel.test.tsx ui/layout/model.test.ts ui/panels/panels.test.tsx 2>&1 | grep -E "Test Files|Tests "
```
Expected: `Test Files  3 failed (3)`.

- [ ] **Step 3: Panel, kayıt, stil ve metinler**

**Yeni dosya:** `app/ui/panels/ArgalaPanel.tsx`

````tsx
// Argala paneli (ileri modüller spec'i §5): bir referans burç için dört çiftin iki graha listesi
// ve durumu. Referans lagna burcu ya da ortak seçimde sabitlenen burçtur (burç, ev ya da bir
// grahanın burcu). Değerler `compute_jaimini`'den gelir; burada hesap yoktur (C4). Durumlar
// yalnızca metinle ayrılır; renk anlam taşımaz (C9).
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { ArgalaPair, Graha, JaiminiReport, OpenedClient } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import type { HighlightTarget } from "../highlight/target";
import type { Panel } from "../layout/model";
import { useJaimini } from "../modules/useJaimini";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export const ARGALA_REFERENCES = ["lagna", "selected"] as const;
export type ArgalaReference = (typeof ARGALA_REFERENCES)[number];
const SIGNS = Array.from({ length: 12 }, (_, i) => i);
const DASH = "—";

export function argalaReference(settings: Record<string, unknown>): ArgalaReference {
  return settings.reference === "selected" ? "selected" : "lagna";
}

/**
 * Sabitlenen öğenin burcu: burç kendisi, ev lagna burcundan sayılan burç (lagna yoksa haritanın
 * çizdiği gibi Koç'tan), graha haritadaki burcu. Gösterim eşlemesidir; hesap değildir.
 */
export function selectedSign(pinned: HighlightTarget | null, opened: OpenedClient, lagna: number | null): number | null {
  if (pinned === null) return null;
  if (pinned.type === "sign") return pinned.sign;
  if (pinned.type === "house") return ((lagna ?? 0) + pinned.house - 1) % 12;
  const g = opened.chart.grahas.find((x) => x.graha === pinned.id);
  return g ? signIndex(g.sign) : null;
}

export function ArgalaPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const module = useJaimini(opened);
  const phase = usePanelLoadPhase(module.loading);
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (module.data === null) return phase === "skeleton" ? <PanelSkeleton /> : null;
  return <Argala panel={panel} report={module.data} opened={opened} />;
}

function Argala({ panel, report, opened }: { panel: Panel; report: JaiminiReport; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const { pinned } = useHighlight();
  const id = useId();
  const mode = argalaReference(panel.settings);
  const lagna = timeUnknown(opened) ? null : signIndex(opened.chart.lagna.sign);
  const reference = mode === "lagna" ? lagna : selectedSign(pinned, opened, lagna);
  const row = reference === null ? undefined : report.argala.find((r) => signIndex(r.reference) === reference);

  const grahas = (list: Graha[]) =>
    list.length === 0
      ? DASH
      : list.map((g, i) => (
          <span key={g}>
            {i > 0 && " "}
            <PinButton target={{ type: "graha", id: g }} label={t(`grahaName.${g}`)}>
              {t(`graha.${g}`)}
            </PinButton>
          </span>
        ));
  const pairRow = (p: ArgalaPair) => (
    <tr key={p.argala_house} data-pair={p.argala_house} data-status={p.status}>
      <th scope="row" className="num">
        {p.argala_house}
      </th>
      <td>{grahas(p.argala_grahas)}</td>
      <td className="num">{p.virodha_house}</td>
      <td>{grahas(p.virodha_grahas)}</td>
      <td>{t(`argala.status.${p.status}`)}</td>
    </tr>
  );

  return (
    <div className="module-panel">
      <div className="module-toolbar">
        <label htmlFor={`${id}-reference`}>{t("argala.reference")}</label>
        <select id={`${id}-reference`} value={mode} onChange={(e) => setPanelSettings(panel.id, { reference: e.target.value })}>
          {ARGALA_REFERENCES.map((r) => (
            <option key={r} value={r}>
              {t(`argala.references.${r}`)}
            </option>
          ))}
        </select>
      </div>
      {mode === "selected" && (
        <div className="argala-signs" role="group" aria-label={t("argala.pickLabel")}>
          {SIGNS.map((s) => (
            <PinButton key={s} target={{ type: "sign", sign: s }}>
              {t(`sign.${s}`)}
            </PinButton>
          ))}
        </div>
      )}
      {mode === "lagna" && lagna === null && <p className="panel-notice">{t("argala.noLagna")}</p>}
      {mode === "selected" && reference === null && <p className="panel-message">{t("argala.pickSign")}</p>}
      {row && reference !== null && (
        <>
          <p className="argala-reference">
            {t("argala.referenceLine", { sign: t(`sign.${reference}`) })}
            {reference === lagna && <span className="av-lagna"> {t("chart.lagna")}</span>}
          </p>
          <table className="module-table">
            <caption className="visually-hidden">{t("argala.caption", { sign: t(`sign.${reference}`) })}</caption>
            <thead>
              <tr>
                <th scope="col" className="num">
                  {t("argala.argalaHouse")}
                </th>
                <th scope="col">{t("argala.argalaGrahas")}</th>
                <th scope="col" className="num">
                  {t("argala.virodhaHouse")}
                </th>
                <th scope="col">{t("argala.virodhaGrahas")}</th>
                <th scope="col">{t("argala.statusColumn")}</th>
              </tr>
            </thead>
            <tbody>{row.pairs.map(pairRow)}</tbody>
          </table>
          {row.third_house_malefic_argala && <p className="panel-notice">{t("argala.thirdHouse")}</p>}
        </>
      )}
      <p className="panel-message">{t("argala.note")}</p>
      <p className="panel-message">{t("jaimini.method", { method: report.method })}</p>
    </div>
  );
}
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index e79c5e6..6cd261a 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -65,6 +65,7 @@ export const en: Messages = {
     drishti: "Aspects",
     jaimini_karakas: "Chara karaka",
     arudha: "Arudha padas",
+    argala: "Argala",
     unknown: "Unknown panel",
     loading: "Loading",
     unavailable: "This panel arrives in a later version.",
@@ -136,6 +137,23 @@ export const en: Messages = {
     method: "Method: {{method}}",
     timeUnknown: "Birth time unknown: the Moon's degree is approximate (≈); the karaka order and the karakamsha may differ.",
   },
+  argala: {
+    reference: "Reference",
+    references: { lagna: "Lagna", selected: "Selected sign" },
+    pickLabel: "Choose the reference sign",
+    pickSign: "Choose a sign: click one of the signs above, a house in the chart or a graha in any table.",
+    noLagna: "Birth time unknown: there is no lagna reference. Use “Selected sign” as the reference.",
+    referenceLine: "Reference: {{sign}}",
+    caption: "Argala, reference {{sign}}: houses that give argala, houses that obstruct, and the status",
+    argalaHouse: "Argala house",
+    argalaGrahas: "Argala grahas",
+    virodhaHouse: "Obstructing house",
+    virodhaGrahas: "Obstructing grahas",
+    statusColumn: "Status",
+    status: { None: "none", Stands: "stands", Contested: "contested", Obstructed: "obstructed" },
+    thirdHouse: "Three or more natural malefics in the 3rd house: third-house malefic argala.",
+    note: "The status looks at the number of grahas only; strength is not compared.",
+  },
   arudha: {
     caption: "Arudha padas: the pada of each of the twelve houses, its sign and the lord of the house",
     pada: "Pada",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 26ebeab..e9594b5 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -65,6 +65,7 @@ export const fr: Messages = {
     drishti: "Aspects",
     jaimini_karakas: "Chara karaka",
     arudha: "Arudha padas",
+    argala: "Argala",
     unknown: "Panneau inconnu",
     loading: "Chargement",
     unavailable: "Ce panneau arrivera dans une version ultérieure.",
@@ -136,6 +137,23 @@ export const fr: Messages = {
     method: "Méthode : {{method}}",
     timeUnknown: "Heure de naissance inconnue : le degré de la Lune est approximatif (≈) ; l'ordre des karakas et le karakamsha peuvent différer.",
   },
+  argala: {
+    reference: "Référence",
+    references: { lagna: "Lagna", selected: "Signe sélectionné" },
+    pickLabel: "Choisir le signe de référence",
+    pickSign: "Choisissez un signe : cliquez sur l'un des signes ci-dessus, sur une maison du thème ou sur un graha dans un tableau.",
+    noLagna: "Heure de naissance inconnue : pas de référence au lagna. Utilisez « Signe sélectionné » comme référence.",
+    referenceLine: "Référence : {{sign}}",
+    caption: "Argala, référence {{sign}} : maisons qui donnent l'argala, maisons qui l'entravent et état",
+    argalaHouse: "Maison d'argala",
+    argalaGrahas: "Grahas d'argala",
+    virodhaHouse: "Maison d'entrave",
+    virodhaGrahas: "Grahas d'entrave",
+    statusColumn: "État",
+    status: { None: "aucun", Stands: "valide", Contested: "disputé", Obstructed: "entravé" },
+    thirdHouse: "Trois maléfiques naturels ou plus en maison 3 : argala maléfique de la 3e maison.",
+    note: "L'état ne tient compte que du nombre de grahas ; la force n'est pas comparée.",
+  },
   arudha: {
     caption: "Arudha padas : le pada de chacune des douze maisons, son signe et le maître de la maison",
     pada: "Pada",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 6e274f2..b0ebf9f 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -64,6 +64,7 @@ export const tr = {
     drishti: "Bakışlar",
     jaimini_karakas: "Chara karaka",
     arudha: "Arudha padalar",
+    argala: "Argala",
     unknown: "Bilinmeyen panel",
     loading: "Yükleniyor",
     unavailable: "Bu panel sonraki bir sürümde gelecek.",
@@ -135,6 +136,23 @@ export const tr = {
     method: "Yöntem: {{method}}",
     timeUnknown: "Doğum saati bilinmiyor: Ay'ın derecesi yaklaşıktır (≈); karaka sırası ve karakamsha değişebilir.",
   },
+  argala: {
+    reference: "Referans",
+    references: { lagna: "Lagna", selected: "Seçili burç" },
+    pickLabel: "Referans burcu seç",
+    pickSign: "Bir burç seçin: yukarıdaki burçlardan birine, haritada bir eve ya da herhangi bir tabloda bir grahaya tıklayın.",
+    noLagna: "Doğum saati bilinmiyor: lagna referansı yok. Referans olarak “Seçili burç”u kullanın.",
+    referenceLine: "Referans: {{sign}}",
+    caption: "Argala, referans {{sign}}: argala veren evler, engelleyen evler ve durum",
+    argalaHouse: "Argala evi",
+    argalaGrahas: "Argala grahaları",
+    virodhaHouse: "Engel evi",
+    virodhaGrahas: "Engel grahaları",
+    statusColumn: "Durum",
+    status: { None: "yok", Stands: "geçerli", Contested: "çekişmeli", Obstructed: "engelli" },
+    thirdHouse: "3. evde üç ya da daha çok doğal kötücül var: 3. ev kötücül argalası.",
+    note: "Durum yalnızca graha sayısına bakar; güç karşılaştırması yapılmaz.",
+  },
   arudha: {
     caption: "Arudha padalar: on iki evin padası, burcu ve evin lordu",
     pada: "Pada",
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index 7e1e8a7..2a303aa 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -15,6 +15,7 @@ export const PANEL_KINDS = [
   "drishti",
   "jaimini_karakas",
   "arudha",
+  "argala",
 ] as const;
 export type KnownPanelKind = (typeof PANEL_KINDS)[number];
 
@@ -29,6 +30,7 @@ export const MODULE_PANELS = [
   // Jaimini çekirdeği (Plan 15).
   { kind: "jaimini_karakas", w: 5, h: 14 },
   { kind: "arudha", w: 4, h: 12 },
+  { kind: "argala", w: 6, h: 9 },
 ] as const satisfies readonly { kind: KnownPanelKind; w: number; h: number }[];
 export type ModulePanelKind = (typeof MODULE_PANELS)[number]["kind"];
 
diff --git a/app/ui/panels/panels.css b/app/ui/panels/panels.css
index 7fb48c3..e9ec024 100644
--- a/app/ui/panels/panels.css
+++ b/app/ui/panels/panels.css
@@ -197,3 +197,16 @@
 .jaimini-karakamsha .panel-message {
   font-weight: var(--fw-regular);
 }
+.argala-signs {
+  display: flex;
+  flex-wrap: wrap;
+  gap: 2px 10px;
+}
+.argala-signs .link-button[aria-pressed="true"] {
+  font-weight: var(--fw-medium);
+  text-decoration: underline;
+}
+.argala-reference {
+  margin: 4px 0 0;
+  font-weight: var(--fw-medium);
+}
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index 2003f2f..0f20aca 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -3,6 +3,7 @@ import { DashaStripPanel } from "../dasha/DashaStripPanel";
 import { DashaTreePanel } from "../dasha/DashaTreePanel";
 import { isKnownPanelKind, type KnownPanelKind } from "../layout/model";
 import { NotesPanel } from "../notes/NotesPanel";
+import { ArgalaPanel } from "./ArgalaPanel";
 import { ArudhaPanel } from "./ArudhaPanel";
 import { AshtakavargaPanel } from "./AshtakavargaPanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
@@ -30,6 +31,7 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   // Jaimini çekirdeği (Plan 15).
   jaimini_karakas: { titleKey: "panel.jaimini_karakas", component: JaiminiKarakasPanel },
   arudha: { titleKey: "panel.arudha", component: ArudhaPanel },
+  argala: { titleKey: "panel.argala", component: ArgalaPanel },
 };
 
 const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };
````

- [ ] **Step 4: Kapılar**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep -E "✓ built|larger than 500 kB"
grep -c "component: UnavailablePanel" app/ui/panels/registry.ts
```
Expected: `Test Files  68 passed (68)`, `Tests  608 passed (608)` (+9); build iki satır yazar: `(!) Some chunks are larger than 500 kB after minification.` ve `✓ built` (D22: uyarı bu görevde ilk kez çıkar, bilgilendiricidir; yapılandırma değiştirilmez); son komut `1` yazar (yer tutucu yalnızca bilinmeyen türün yedeğidir).

- [ ] **Step 5: Commit**

```bash
git add app/ui/panels app/ui/layout app/ui/i18n
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): argala panel with lagna or selected-sign reference"
```
Expected: iki `grep` çıktısız.

---

### Task 9: Bakışlar panelinde rashi türü; graha tablosunda karaka sütunu

Spec §5 "Arayüz": "`drishti` paneline `rashi` türü eklenir", "`graha_table` paneline isteğe bağlı karaka sütunu eklenir"; D19, D20. Bakışlar: "Tür" seçicisi (Graha drishti / Rashi drishti); rashi türünde dokuz graha × dokuz graha matrisi, `●` rashi drishti; "Değer" seçicisi yalnızca graha türünde çizilir; araç çubuğu yükleme ve hata sırasında da durur. Graha tablosu: sütun seçicide "Karaka" kutusu; açıkken son sütun her grahanın rol kısaltmasını gösterir (Ketu ve lagna satırı boş).

**Files:**
- Modify: `app/ui/panels/DrishtiPanel.tsx`, `app/ui/panels/GrahaTablePanel.tsx`, `app/ui/i18n/{tr,en,fr}.ts`
- Test: `app/ui/panels/DrishtiPanel.test.tsx`, `app/ui/panels/GrahaTablePanel.test.tsx`

**Interfaces:**
- Consumes: `useJaimini`, `KARAKA_ROLES` (Task 6); `JaiminiReport.graha_rashi_drishti`, `JaiminiReport.karakas`; Plan 14'ün `DrishtiPanel` parçaları (`fullAspectPairs`, `formatVirupa`, `ASPECTING_GRAHAS`, `FULL_MARK`); `PanelError`.
- Produces: `DrishtiPanel.tsx`: `DRISHTI_KINDS`, `drishtiKind(settings): "graha" | "rashi"`, `rashiAspectPairs(report: JaiminiReport): Set<string>`, panel ayarı `kind`; `GrahaTablePanel.tsx`: panel ayarı `karaka: boolean`; i18n `drishti.{kind,kinds.graha,kinds.rashi,captionRashi,rashiFull,legendRashi}`, `table.karaka`.

- [ ] **Step 1: Başarısız testleri yaz**

`DrishtiPanel.test.tsx`'in Plan 14'ten gelen on testi değişmeden geçmelidir (graha türü varsayılandır). `GrahaTablePanel.test.tsx` artık `api`'yi taklit eder (sütun açıkken panel komut çağırır).

````diff
diff --git a/app/ui/panels/DrishtiPanel.test.tsx b/app/ui/panels/DrishtiPanel.test.tsx
index 6f1edf7..d91040c 100644
--- a/app/ui/panels/DrishtiPanel.test.tsx
+++ b/app/ui/panels/DrishtiPanel.test.tsx
@@ -1,14 +1,14 @@
 import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
 import { expect, test, vi } from "vitest";
 
-vi.mock("../api", () => ({ api: { computeDrishti: vi.fn() } }));
+vi.mock("../api", () => ({ api: { computeDrishti: vi.fn(), computeJaimini: vi.fn() } }));
 
 import { api, type Result } from "../api";
-import type { DrishtiTable } from "../bindings";
+import type { DrishtiTable, JaiminiReport } from "../bindings";
 import type { Panel } from "../layout/model";
 import { fixture, unknownTimeOpened } from "../test/fixture";
 import { renderWithStores } from "../test/render";
-import { ASPECTING_GRAHAS, DrishtiPanel, drishtiValue, formatVirupa, fullAspectPairs, FULL_MARK } from "./DrishtiPanel";
+import { ASPECTING_GRAHAS, DrishtiPanel, drishtiKind, drishtiValue, formatVirupa, fullAspectPairs, FULL_MARK, rashiAspectPairs } from "./DrishtiPanel";
 
 const ok = (): Result<DrishtiTable> => ({ status: "ok", data: fixture().drishti });
 
@@ -21,9 +21,13 @@ function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<
 const rowHeader = (name: string) => within(document.querySelector("tbody")!).getByRole("button", { name });
 const rowCells = (name: string) => [...rowHeader(name).closest("tr")!.querySelectorAll("td")].map((c) => c.textContent);
 
+const okJaimini = (): Result<JaiminiReport> => ({ status: "ok", data: fixture().jaimini });
+
 beforeEach(() => {
   vi.mocked(api.computeDrishti).mockReset();
   vi.mocked(api.computeDrishti).mockImplementation(async () => ok());
+  vi.mocked(api.computeJaimini).mockReset();
+  vi.mocked(api.computeJaimini).mockImplementation(async () => okJaimini());
 });
 
 test("asks for the open chart's aspects and marks full sign aspects, seven rows by nine columns", async () => {
@@ -164,3 +168,69 @@ test("helpers: full-aspect pairs come from the occupants, virupa text has one de
   expect([formatVirupa(0), formatVirupa(60), formatVirupa(37.55), formatVirupa(null), formatVirupa(Number.NaN)]).toEqual(["0.0", "60.0", "37.5", "—", "—"]);
   expect(ASPECTING_GRAHAS).toEqual(["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"]);
 });
+
+test("the kind selector defaults to graha drishti and asks for no Jaimini report", async () => {
+  renderPanel();
+  await screen.findByRole("table");
+  expect(screen.getByRole("combobox", { name: "Tür" })).toHaveDisplayValue("Graha drishti");
+  expect(screen.getByRole("combobox", { name: "Değer" })).toBeInTheDocument();
+  expect(api.computeJaimini).not.toHaveBeenCalled();
+  expect([drishtiKind({}), drishtiKind({ kind: "tajika" }), drishtiKind({ kind: "rashi" })]).toEqual(["graha", "graha", "rashi"]);
+});
+
+test("rashi drishti: nine rows by nine columns from the Jaimini report, marked by sign aspect", async () => {
+  renderPanel({ kind: "rashi" });
+  await screen.findByRole("table", { name: "Rashi drishti: satırlar bakan, sütunlar bakılan graha" });
+  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
+  expect(api.computeJaimini).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: fixture().opened.chart.meta.settings, time_known: true });
+  expect(api.computeDrishti).not.toHaveBeenCalled();
+  // Rashi drishtide düğümler de bakar: dokuz satır. Virupa değeri yoktur: "Değer" seçici çizilmez.
+  expect(screen.getAllByRole("rowheader").map((h) => h.textContent)).toEqual(["Güneş", "Ay", "Mars", "Merkür", "Jüpiter", "Venüs", "Satürn", "Rahu", "Ketu"]);
+  expect(screen.queryByRole("combobox", { name: "Değer" })).toBeNull();
+  // Delhi fikstürü: Yay (Güneş, Merkür) boş burçlara bakar; Terazi (Ay) Kova'ya (Mars); Kova
+  // Koç'a (Jüpiter, Satürn), Yengeç'e (Rahu) ve Terazi'ye; Akrep (Venüs) Oğlak'a (Ketu), Koç'a
+  // ve Yengeç'e bakar. Bakış karşılıklıdır.
+  const f = `${FULL_MARK}rashi drishti`;
+  expect(rowCells("Güneş")).toEqual(["—", "", "", "", "", "", "", "", ""]);
+  expect(rowCells("Ay")).toEqual(["", "—", f, "", "", "", "", "", ""]);
+  expect(rowCells("Mars")).toEqual(["", f, "—", "", f, "", f, f, ""]);
+  expect(rowCells("Merkür")).toEqual(["", "", "", "—", "", "", "", "", ""]);
+  expect(rowCells("Jüpiter")).toEqual(["", "", f, "", "—", f, "", "", ""]);
+  expect(rowCells("Venüs")).toEqual(["", "", "", "", f, "—", f, f, f]);
+  expect(rowCells("Satürn")).toEqual(["", "", f, "", "", f, "—", "", ""]);
+  expect(rowCells("Rahu")).toEqual(["", "", f, "", "", f, "", "—", ""]);
+  expect(rowCells("Ketu")).toEqual(["", "", "", "", "", f, "", "", "—"]);
+  expect(document.querySelectorAll('td [aria-hidden="true"]')).toHaveLength(16);
+  expect(screen.getByText(/^● rashi drishti: grahanın burcu, bakılan grahanın burcuna bakar/)).toBeInTheDocument();
+  expect(screen.getByText("Yöntem: jaimini.core_v1")).toBeInTheDocument();
+});
+
+test("switching the kind is saved with the layout and asks the other command once", async () => {
+  const { persistence } = renderPanel({ value: "virupa" });
+  fireEvent.change(await screen.findByRole("combobox", { name: "Tür" }), { target: { value: "rashi" } });
+  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
+  expect(persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ value: "virupa", kind: "rashi" });
+});
+
+test("rashi drishti: an error is shown under the toolbar, retry asks again, and the kind can still be changed", async () => {
+  vi.mocked(api.computeJaimini).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
+  renderPanel({ kind: "rashi" });
+  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
+  expect(screen.getByRole("combobox", { name: "Tür" })).toHaveDisplayValue("Rashi drishti");
+  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
+  await screen.findByRole("table");
+  expect(api.computeJaimini).toHaveBeenCalledTimes(2);
+});
+
+test("rashi drishti: unknown birth time marks the Moon; pairs come from the occupants", async () => {
+  renderPanel({ kind: "rashi" }, unknownTimeOpened());
+  await screen.findByRole("table");
+  expect(vi.mocked(api.computeJaimini).mock.calls[0][0].time_known).toBe(false);
+  expect(document.querySelectorAll('[title="yaklaşık"]')).toHaveLength(2);
+  const pairs = rashiAspectPairs(fixture().jaimini);
+  expect(pairs.size).toBe(16);
+  for (const p of pairs) {
+    const [from, to] = p.split(">");
+    expect(pairs.has(`${to}>${from}`), p).toBe(true);
+  }
+});
diff --git a/app/ui/panels/GrahaTablePanel.test.tsx b/app/ui/panels/GrahaTablePanel.test.tsx
index 5462a93..6901025 100644
--- a/app/ui/panels/GrahaTablePanel.test.tsx
+++ b/app/ui/panels/GrahaTablePanel.test.tsx
@@ -1,9 +1,17 @@
 import { fireEvent, screen, waitFor, within } from "@testing-library/react";
+import { api } from "../api";
 import type { Panel } from "../layout/model";
 import { fixture, unknownTimeOpened } from "../test/fixture";
 import { renderWithStores } from "../test/render";
 import { GrahaTablePanel } from "./GrahaTablePanel";
 
+vi.mock("../api", () => ({ api: { computeJaimini: vi.fn() } }));
+
+beforeEach(() => {
+  vi.mocked(api.computeJaimini).mockReset();
+  vi.mocked(api.computeJaimini).mockImplementation(async () => ({ status: "ok", data: fixture().jaimini }));
+});
+
 function renderTable(settings: Record<string, unknown> = {}, opened = fixture().opened) {
   const panel: Panel = { id: "t", kind: "graha_table", x: 0, y: 0, w: 6, h: 8, settings };
   const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
@@ -69,3 +77,51 @@ test("the lagna row shows its longitude, leaves latitude and speed empty, and a
   renderTable({ columns: ["lon"] }, o);
   expect([...lagna().querySelectorAll("td")].map((c) => c.textContent)).toEqual(["—"]);
 });
+
+test("the karaka column is off by default and asks for nothing", () => {
+  renderTable();
+  expect(headers()).not.toContain("Karaka");
+  expect(api.computeJaimini).not.toHaveBeenCalled();
+});
+
+test("the karaka column shows each graha's role from the Jaimini report; Ketu and the lagna have none", async () => {
+  renderTable({ columns: ["sign"], karaka: true });
+  expect(headers()).toEqual(["Graha", "Burç", "Karaka"]);
+  await waitFor(() => expect(document.querySelector('[data-karaka="Moon"]')).toHaveTextContent("AK Atmakaraka"));
+  expect(api.computeJaimini).toHaveBeenCalledTimes(1);
+  const role = (g: string) => document.querySelector(`[data-karaka="${g}"]`)!.textContent;
+  expect(["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"].map(role)).toEqual([
+    "MK Matrikaraka",
+    "AK Atmakaraka",
+    "GK Gnatikaraka",
+    "PiK Pitrikaraka",
+    "DK Darakaraka",
+    "PK Putrakaraka",
+    "BK Bhratrikaraka",
+    "AmK Amatyakaraka",
+    "",
+  ]);
+  expect(within(row("Ay")).getByTitle("Atmakaraka")).toHaveTextContent("AK");
+  const lagna = screen.getByRole("rowheader", { name: "Lagna" }).closest("tr")!;
+  expect([...lagna.querySelectorAll("td")].map((c) => c.textContent)).toEqual(["İki", ""]);
+});
+
+test("the karaka checkbox writes settings.karaka and leaves settings.columns alone", async () => {
+  const { persistence } = renderTable({ columns: ["sign", "house"] });
+  const summary = screen.getByText("Sütunlar", { selector: "summary" });
+  fireEvent.click(summary);
+  fireEvent.click(within(summary.closest("details")!).getByRole("checkbox", { name: "Karaka" }));
+  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
+  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!;
+  expect(saved.panels[0].settings).toEqual({ columns: ["sign", "house"], karaka: true });
+});
+
+test("a failed karaka request keeps the table and offers a retry", async () => {
+  vi.mocked(api.computeJaimini).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
+  renderTable({ karaka: true });
+  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
+  expect(row("Ay")).toBeInTheDocument();
+  expect(document.querySelector('[data-karaka="Moon"]')).toHaveTextContent(/^$/);
+  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
+  await waitFor(() => expect(document.querySelector('[data-karaka="Moon"]')).toHaveTextContent("AK Atmakaraka"));
+});
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
pnpm --dir app exec vitest run ui/panels/DrishtiPanel.test.tsx ui/panels/GrahaTablePanel.test.tsx 2>&1 | grep -E "Test Files|Tests "
```
Expected: `Test Files  2 failed (2)`, `Tests  8 failed | 16 passed (24)` ("sütun varsayılan olarak yok" testi uygulamadan önce de geçer).

- [ ] **Step 3: Bakışlar paneli: tür seçicisi ve rashi matrisi**

Matris çizimi tek yardımcıya (`matrix`) toplanır: satırlar graha türünde yedi, rashi türünde dokuz grahadır; hücre içeriğini çağıran verir. Yalnızca seçili türün komutu çağrılır (diğerinin anahtarı `null`'dur).

````diff
diff --git a/app/ui/panels/DrishtiPanel.tsx b/app/ui/panels/DrishtiPanel.tsx
index d1140e1..815a336 100644
--- a/app/ui/panels/DrishtiPanel.tsx
+++ b/app/ui/panels/DrishtiPanel.tsx
@@ -1,17 +1,20 @@
 // Bakışlar paneli (ileri modüller spec'i §4): bakan yedi graha × bakılan dokuz graha matrisi.
 // Değerler `compute_drishti`'den gelir ve panel açıkken istenir (K1); burada hesap yoktur.
 // "Tam bakış işareti" görünümü burç temelli tam bakışları `●` ile, "virupa" görünümü derece
-// temelli bakış değerini sayı olarak gösterir; renk anlam taşımaz. Rashi drishti türü Jaimini
-// alt projesiyle gelir. Ortak vurgu: bir graha vurgulanınca satırı ve sütunu `--select` alır.
-import { useId } from "react";
+// temelli bakış değerini sayı olarak gösterir; renk anlam taşımaz. "Rashi drishti" türü
+// (ileri modüller spec'i §5) dokuz grahanın burç bakışını `compute_jaimini`'den okur: bir
+// burçtaki grahalar o burcun baktığı burçlardaki grahalara bakar; virupa değeri yoktur.
+// Ortak vurgu: bir graha vurgulanınca satırı ve sütunu `--select` alır.
+import { useId, type ReactNode } from "react";
 import { useTranslation } from "react-i18next";
 import { api } from "../api";
-import type { DrishtiTable, Graha, OpenedClient } from "../bindings";
+import type { DrishtiTable, Graha, JaiminiReport, OpenedClient } from "../bindings";
 import { GRAHA_IDS, type GrahaId } from "../chart/types";
 import { useHighlight } from "../highlight/store";
 import { sameTarget, type HighlightTarget } from "../highlight/target";
 import type { Panel } from "../layout/model";
 import { chartInput, inputKey } from "../modules/input";
+import { useJaimini } from "../modules/useJaimini";
 import { useModule } from "../modules/useModule";
 import { timeUnknown } from "../session/birth";
 import { useSession } from "../session/store";
@@ -23,6 +26,8 @@ import type { PanelProps } from "./types";
 
 /** Bakan grahalar: `DrishtiTable.sphuta` satır sırası (Rust `ASPECTING_GRAHAS`). */
 export const ASPECTING_GRAHAS = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"] as const satisfies readonly GrahaId[];
+export const DRISHTI_KINDS = ["graha", "rashi"] as const;
+export type DrishtiKind = (typeof DRISHTI_KINDS)[number];
 export const DRISHTI_VALUES = ["full", "virupa"] as const;
 export type DrishtiValue = (typeof DRISHTI_VALUES)[number];
 export const FULL_MARK = "●";
@@ -32,6 +37,17 @@ export function drishtiValue(settings: Record<string, unknown>): DrishtiValue {
   return settings.value === "virupa" ? "virupa" : "full";
 }
 
+export function drishtiKind(settings: Record<string, unknown>): DrishtiKind {
+  return settings.kind === "rashi" ? "rashi" : "graha";
+}
+
+/** Rashi drishtisi olan (bakan, bakılan) çiftleri: bakılan graha hedef burçtadır. */
+export function rashiAspectPairs(report: JaiminiReport): Set<string> {
+  const pairs = new Set<string>();
+  for (const a of report.graha_rashi_drishti) for (const target of a.occupants) pairs.add(`${a.graha}>${target}`);
+  return pairs;
+}
+
 /** Burç temelli tam bakışı olan (bakan, bakılan) çiftleri: bakılan graha hedef burçtadır. */
 export function fullAspectPairs(table: DrishtiTable): Set<string> {
   const pairs = new Set<string>();
@@ -56,16 +72,15 @@ function DrishtiMatrix({ panel, opened }: { panel: Panel; opened: OpenedClient }
   const { setPanelSettings } = useLayoutStore();
   const highlight = useHighlight();
   const id = useId();
+  const kind = drishtiKind(panel.settings);
   const value = drishtiValue(panel.settings);
-  const module = useModule(inputKey(opened), () => api.computeDrishti(chartInput(opened)));
+  // Yalnızca seçili türün komutu çağrılır; diğerinin anahtarı boştur.
+  const graha = useModule(kind === "graha" ? inputKey(opened) : null, () => api.computeDrishti(chartInput(opened)));
+  const rashi = useJaimini(opened, kind === "rashi");
+  const module = kind === "graha" ? graha : rashi;
   const phase = usePanelLoadPhase(module.loading);
 
-  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
-  const table = module.data;
-  if (table === null) return phase === "skeleton" ? <PanelSkeleton /> : null;
-
   const unknown = timeUnknown(opened);
-  const full = fullAspectPairs(table);
   const active = highlight.active;
   const on = (g: Graha) => active?.type === "graha" && active.id === g;
   const header = (g: GrahaId, short: boolean) => {
@@ -88,62 +103,98 @@ function DrishtiMatrix({ panel, opened }: { panel: Panel; opened: OpenedClient }
       </Approx>
     );
   };
+  const fullMark = (label: string) => (
+    <>
+      <span aria-hidden="true">{FULL_MARK}</span>
+      <span className="visually-hidden">{label}</span>
+    </>
+  );
+  /** Matris: `rows` bakan grahalar, sütunlar dokuz graha; `cell` hücrenin içeriğini verir. */
+  const matrix = (caption: string, rows: readonly GrahaId[], cell: (from: GrahaId, i: number, to: GrahaId, j: number) => ReactNode) => (
+    <table className="module-table drishti-table">
+      <caption className="visually-hidden">{caption}</caption>
+      <thead>
+        <tr>
+          <th scope="col">{t("drishti.aspecting")}</th>
+          {GRAHA_IDS.map((g) => (
+            <th key={g} scope="col" data-graha={g} className={on(g) ? "num is-highlighted" : "num"}>
+              {header(g, true)}
+            </th>
+          ))}
+        </tr>
+      </thead>
+      <tbody>
+        {rows.map((from, i) => (
+          <tr key={from} data-graha={from} className={on(from) ? "is-highlighted" : undefined}>
+            <th scope="row">{header(from, false)}</th>
+            {GRAHA_IDS.map((to, j) => (
+              <td key={to} className={on(to) ? "num is-highlighted" : "num"}>
+                {from === to ? DASH : cell(from, i, to, j)}
+              </td>
+            ))}
+          </tr>
+        ))}
+      </tbody>
+    </table>
+  );
+
+  const grahaBody = (table: DrishtiTable) => {
+    const full = fullAspectPairs(table);
+    return (
+      <>
+        {matrix(t("drishti.caption"), ASPECTING_GRAHAS, (from, i, to, j) =>
+          value === "virupa" ? formatVirupa(table.sphuta[i][j]) : full.has(`${from}>${to}`) ? fullMark(t("drishti.full")) : "",
+        )}
+        <p className="panel-message">{value === "full" ? t("drishti.legendFull", { mark: FULL_MARK }) : t("drishti.legendVirupa")}</p>
+        <p className="panel-message">{t("drishti.method", { method: table.method })}</p>
+      </>
+    );
+  };
+  const rashiBody = (report: JaiminiReport) => {
+    const pairs = rashiAspectPairs(report);
+    return (
+      <>
+        {matrix(t("drishti.captionRashi"), GRAHA_IDS, (from, _i, to) => (pairs.has(`${from}>${to}`) ? fullMark(t("drishti.rashiFull")) : ""))}
+        <p className="panel-message">{t("drishti.legendRashi", { mark: FULL_MARK })}</p>
+        <p className="panel-message">{t("drishti.method", { method: report.method })}</p>
+      </>
+    );
+  };
 
   return (
     <div className="module-panel">
       <div className="module-toolbar">
-        <label htmlFor={`${id}-value`}>{t("drishti.value")}</label>
-        <select id={`${id}-value`} value={value} onChange={(e) => setPanelSettings(panel.id, { value: e.target.value })}>
-          {DRISHTI_VALUES.map((v) => (
-            <option key={v} value={v}>
-              {t(`drishti.values.${v}`)}
+        <label htmlFor={`${id}-kind`}>{t("drishti.kind")}</label>
+        <select id={`${id}-kind`} value={kind} onChange={(e) => setPanelSettings(panel.id, { kind: e.target.value })}>
+          {DRISHTI_KINDS.map((k) => (
+            <option key={k} value={k}>
+              {t(`drishti.kinds.${k}`)}
             </option>
           ))}
         </select>
+        {kind === "graha" && (
+          <>
+            <label htmlFor={`${id}-value`}>{t("drishti.value")}</label>
+            <select id={`${id}-value`} value={value} onChange={(e) => setPanelSettings(panel.id, { value: e.target.value })}>
+              {DRISHTI_VALUES.map((v) => (
+                <option key={v} value={v}>
+                  {t(`drishti.values.${v}`)}
+                </option>
+              ))}
+            </select>
+          </>
+        )}
       </div>
-      {unknown && <p className="panel-notice">{t("drishti.timeUnknown")}</p>}
-      <table className="module-table drishti-table">
-        <caption className="visually-hidden">{t("drishti.caption")}</caption>
-        <thead>
-          <tr>
-            <th scope="col">{t("drishti.aspecting")}</th>
-            {GRAHA_IDS.map((g) => (
-              <th key={g} scope="col" data-graha={g} className={on(g) ? "num is-highlighted" : "num"}>
-                {header(g, true)}
-              </th>
-            ))}
-          </tr>
-        </thead>
-        <tbody>
-          {ASPECTING_GRAHAS.map((from, i) => (
-            <tr key={from} data-graha={from} className={on(from) ? "is-highlighted" : undefined}>
-              <th scope="row">{header(from, false)}</th>
-              {GRAHA_IDS.map((to, j) => {
-                const self = from === to;
-                const isFull = full.has(`${from}>${to}`);
-                return (
-                  <td key={to} className={on(to) ? "num is-highlighted" : "num"}>
-                    {self ? (
-                      DASH
-                    ) : value === "virupa" ? (
-                      formatVirupa(table.sphuta[i][j])
-                    ) : isFull ? (
-                      <>
-                        <span aria-hidden="true">{FULL_MARK}</span>
-                        <span className="visually-hidden">{t("drishti.full")}</span>
-                      </>
-                    ) : (
-                      ""
-                    )}
-                  </td>
-                );
-              })}
-            </tr>
-          ))}
-        </tbody>
-      </table>
-      <p className="panel-message">{value === "full" ? t("drishti.legendFull", { mark: FULL_MARK }) : t("drishti.legendVirupa")}</p>
-      <p className="panel-message">{t("drishti.method", { method: table.method })}</p>
+      {module.error ? (
+        <PanelError error={module.error} onRetry={module.retry} />
+      ) : module.data === null ? (
+        phase === "skeleton" && <PanelSkeleton />
+      ) : (
+        <>
+          {unknown && <p className="panel-notice">{t("drishti.timeUnknown")}</p>}
+          {kind === "graha" ? graha.data && grahaBody(graha.data) : rashi.data && rashiBody(rashi.data)}
+        </>
+      )}
     </div>
   );
 }
````

- [ ] **Step 4: Graha tablosu: karaka sütunu; metinler**

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 6cd261a..b934827 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -111,6 +111,11 @@ export const en: Messages = {
     timeUnknown: "Birth time unknown: ashtakavarga also depends on the lagna and is not shown.",
   },
   drishti: {
+    kind: "Kind",
+    kinds: { graha: "Graha drishti", rashi: "Rashi drishti" },
+    captionRashi: "Rashi drishti: rows aspect, columns are aspected",
+    rashiFull: "rashi drishti",
+    legendRashi: "{{mark}} rashi drishti: the sign of the graha aspects the sign of the aspected graha; all nine grahas count. Grahas in the same sign do not aspect each other.",
     value: "Value",
     values: { full: "Full aspect mark", virupa: "Virupa" },
     caption: "Graha aspects: rows aspect, columns are aspected",
@@ -181,6 +186,7 @@ export const en: Messages = {
     columns: "Columns",
     lagnaRow: "Lagna",
     approx: "approximate",
+    karaka: "Karaka",
   },
   errors: {
     dasha_empty: "The sub-periods of this period could not be loaded.",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index e9594b5..2270627 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -111,6 +111,11 @@ export const fr: Messages = {
     timeUnknown: "Heure de naissance inconnue : l'ashtakavarga dépend aussi du lagna et n'est pas affiché.",
   },
   drishti: {
+    kind: "Type",
+    kinds: { graha: "Graha drishti", rashi: "Rashi drishti" },
+    captionRashi: "Rashi drishti : les lignes aspectent, les colonnes sont aspectées",
+    rashiFull: "rashi drishti",
+    legendRashi: "{{mark}} rashi drishti : le signe du graha aspecte le signe du graha aspecté ; les neuf grahas comptent. Les grahas d'un même signe ne s'aspectent pas.",
     value: "Valeur",
     values: { full: "Marque d'aspect plein", virupa: "Virupa" },
     caption: "Aspects des grahas : les lignes aspectent, les colonnes sont aspectées",
@@ -181,6 +186,7 @@ export const fr: Messages = {
     columns: "Colonnes",
     lagnaRow: "Lagna",
     approx: "approximatif",
+    karaka: "Karaka",
   },
   errors: {
     dasha_empty: "Les sous-périodes de cette période n’ont pas pu être chargées.",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index b0ebf9f..b7c7cb9 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -110,6 +110,11 @@ export const tr = {
     timeUnknown: "Doğum saati bilinmiyor: ashtakavarga lagnaya da bağlı olduğu için gösterilmiyor.",
   },
   drishti: {
+    kind: "Tür",
+    kinds: { graha: "Graha drishti", rashi: "Rashi drishti" },
+    captionRashi: "Rashi drishti: satırlar bakan, sütunlar bakılan graha",
+    rashiFull: "rashi drishti",
+    legendRashi: "{{mark}} rashi drishti: grahanın burcu, bakılan grahanın burcuna bakar; dokuz graha da sayılır. Aynı burçtaki grahalar arasında bakış yoktur.",
     value: "Değer",
     values: { full: "Tam bakış işareti", virupa: "Virupa" },
     caption: "Graha bakışları: satırlar bakan, sütunlar bakılan graha",
@@ -180,6 +185,7 @@ export const tr = {
     columns: "Sütunlar",
     lagnaRow: "Lagna",
     approx: "yaklaşık",
+    karaka: "Karaka",
   },
   errors: {
     dasha_empty: "Bu dönemin alt dönemleri alınamadı.",
diff --git a/app/ui/panels/GrahaTablePanel.tsx b/app/ui/panels/GrahaTablePanel.tsx
index 399b1c3..8210f57 100644
--- a/app/ui/panels/GrahaTablePanel.tsx
+++ b/app/ui/panels/GrahaTablePanel.tsx
@@ -4,13 +4,16 @@ import { useTranslation } from "react-i18next";
 import type { GrahaPosition } from "../bindings";
 import { useHighlight } from "../highlight/store";
 import { sameTarget, type HighlightTarget } from "../highlight/target";
+import { useJaimini } from "../modules/useJaimini";
 import { useSession } from "../session/store";
 import { useLayoutStore } from "../view/store";
 import { timeUnknown } from "../session/birth";
 import { Approx } from "./Approx";
 import { APPROX_COLUMNS, grahaCellText, lagnaCellText, NUMERIC_COLUMNS as NUMERIC } from "./grahaCells";
 import { GRAHA_COLUMNS, grahaColumns, toggleColumn, type GrahaColumn } from "./grahaColumns";
+import { KARAKA_ROLES } from "./labels";
 import "./panels.css";
+import { PanelError } from "./states";
 import type { PanelProps } from "./types";
 
 /** Hücre metni `grahaCells.ts`'ten; burada yalnızca erişilebilirlik sarmalayıcıları eklenir. */
@@ -35,16 +38,21 @@ function grahaCell(t: TFunction, g: GrahaPosition, column: GrahaColumn, approx:
 /**
  * Graha tablosu (spec §3.3): sütunlar `settings.columns`'tan, değerler `Chart.grahas` ve
  * `Chart.lagna`'dan. Saat bilinmiyorsa lagna satırı ve ev sütunu gizlenir, Ay'ın saate
- * bağlı değerleri "≈" ile işaretlenir.
+ * bağlı değerleri "≈" ile işaretlenir. İsteğe bağlı "Karaka" sütunu (`settings.karaka`)
+ * chara karaka rolünü `compute_jaimini`'den okur; açık değilken komut çağrılmaz. Sütun
+ * `columns` listesinde değildir: baskı tablosu o listeyi kullanır ve değişmez (K10).
  */
 export function GrahaTablePanel({ panel }: PanelProps) {
   const { t } = useTranslation();
   const { opened } = useSession();
   const { setPanelSettings } = useLayoutStore();
   const highlight = useHighlight();
+  const karakaOn = panel.settings.karaka === true;
+  const jaimini = useJaimini(opened, karakaOn);
 
   if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
   if (opened.chart.grahas.length === 0) return <p className="panel-message">{t("table.empty")}</p>;
+  const roles = new Map((jaimini.data?.karakas ?? []).map((k) => [k.graha, KARAKA_ROLES[k.role]]));
 
   const unknown = timeUnknown(opened);
   const chosen = grahaColumns(panel.settings);
@@ -68,8 +76,13 @@ export function GrahaTablePanel({ panel }: PanelProps) {
               {t(`table.${c}`)}
             </label>
           ))}
+          <label>
+            <input type="checkbox" checked={karakaOn} onChange={(e) => setPanelSettings(panel.id, { karaka: e.target.checked })} />
+            {t("table.karaka")}
+          </label>
         </fieldset>
       </details>
+      {jaimini.error && <PanelError error={jaimini.error} onRetry={jaimini.retry} />}
       <table className="graha-table">
         <thead>
           <tr>
@@ -79,6 +92,7 @@ export function GrahaTablePanel({ panel }: PanelProps) {
                 {t(`table.${c}`)}
               </th>
             ))}
+            {karakaOn && <th scope="col">{t("table.karaka")}</th>}
           </tr>
         </thead>
         <tbody>
@@ -90,6 +104,7 @@ export function GrahaTablePanel({ panel }: PanelProps) {
                   {lagnaCellText(t, opened.chart.lagna, c)}
                 </td>
               ))}
+              {karakaOn && <td />}
             </tr>
           )}
           {opened.chart.grahas.map((g) => {
@@ -122,6 +137,16 @@ export function GrahaTablePanel({ panel }: PanelProps) {
                     {grahaCell(t, g, c, approx)}
                   </td>
                 ))}
+                {karakaOn && (
+                  <td data-karaka={g.graha}>
+                    {roles.has(g.graha) && (
+                      <>
+                        <abbr title={roles.get(g.graha)!.name}>{roles.get(g.graha)!.short}</abbr>
+                        <span className="visually-hidden"> {roles.get(g.graha)!.name}</span>
+                      </>
+                    )}
+                  </td>
+                )}
               </tr>
             );
           })}
````

- [ ] **Step 5: Kapılar**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
git diff --stat HEAD -- app/ui/print app/ui/panels/grahaColumns.ts app/ui/panels/grahaCells.ts
```
Expected: `Test Files  68 passed (68)`, `Tests  617 passed (617)` (+9: `DrishtiPanel` 5, `GrahaTablePanel` 4); `✓ built`; son komut çıktısız (baskı tablosu ve sütun listesi değişmedi: D20, K10).

- [ ] **Step 6: Commit**

```bash
git add app/ui/panels app/ui/i18n
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): rashi drishti kind in the aspects panel; optional karaka column in the graha table"
```
Expected: iki `grep` çıktısız.

---

### Task 10: Uçtan uca: Jaimini panelleri

Gerçek WebKit'te (IPC taklidiyle) tek akış: rapor panel açılmadan istenmez (K1); palet üç paneli ekler, odak eklenen panele geçer; rapor doğru girdiyle istenir (`advanced` varsayılanları dahil); Argala paneli ortak seçimi izler (haritada Ay'a tıklanınca referans Terazi olur ve aynı seçim karaka tablosunda vurgulanır); harita işaretleri on iki padayı yazar ve D9'da kutu kapalıdır; Bakışlar panelinin rashi türü dokuz satır çizer; beş tüketici raporu birer kez ister (`StrictMode` altında da); karaka şeması ayarı değişince harita kaydedilir ve beş tüketici raporu yeni ayarla yeniden ister; paneller ve ayarları düzenle birlikte kaydedilir; aynı komut paneli çıkarır ve odak gövdeye düşmez.

**Files:**
- Modify: `app/e2e/desk.spec.ts`
- Test: aynı dosya

**Interfaces:**
- Consumes: `mockIpc`, `ipcCalls` (`e2e/ipc.ts`; `compute_jaimini` taklidi Task 4'te eklendi; `save_calc_settings` taklidi yeni ayarı haritanın `meta.settings`'ine yazar), Task 5–9'un arayüzü ve metinleri.
- Produces: e2e testi `jaimini panels: …`.

- [ ] **Step 1: Testi yaz**

````diff
diff --git a/app/e2e/desk.spec.ts b/app/e2e/desk.spec.ts
index 20552fa..2062ec2 100644
--- a/app/e2e/desk.spec.ts
+++ b/app/e2e/desk.spec.ts
@@ -324,3 +324,105 @@ test("module panels: the palette adds Aspects, Panchang and Ashtakavarga; aspect
   expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
   expect(await ipcCalls(page, "compute_drishti")).toHaveLength(1);
 });
+
+test("jaimini panels: karakas, arudha padas and argala come from one command; chart marks, rashi drishti and the karaka scheme setting", async ({ page }) => {
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
+  type JaiminiCall = { input: { jd_ut: number; time_known: boolean; settings: { schema_version: number; advanced: { karaka_scheme: string; jaimini_co_lords: boolean } } } };
+  const jaiminiCalls = async () => (await ipcCalls(page, "compute_jaimini")) as JaiminiCall[];
+  // Rapor panel açılmadan istenmez (ileri modüller spec'i K1).
+  expect(await jaiminiCalls()).toHaveLength(0);
+
+  await toggle("karaka", /Chara karaka panelini aç\/kapat/);
+  const karakas = page.getByRole("region", { name: "Chara karaka" });
+  await expect(karakas).toBeFocused();
+  await expect(karakas).toBeInViewport();
+  await expect(karakas.locator('tr[data-role="Atma"]')).toContainText("Ay");
+  await expect(karakas.locator('tr[data-role="Amatya"]')).toContainText("11°11′ (etkin 18°48′)");
+  await expect(karakas.locator(".jaimini-karakamsha")).toContainText("Karakamsha: Bal");
+  const first = await jaiminiCalls();
+  expect(first).toHaveLength(1);
+  expect(first[0].input).toMatchObject({ jd_ut: 2451545, time_known: true, settings: { schema_version: 2, advanced: { karaka_scheme: "Eight", jaimini_co_lords: true } } });
+
+  await toggle("arudha", /Arudha padalar panelini aç\/kapat/);
+  const arudha = page.getByRole("region", { name: "Arudha padalar" });
+  await expect(arudha.locator('tr[data-pada="1"]')).toContainText("A1 · AL");
+  await expect(arudha.locator('tr[data-pada="1"]')).toContainText("Bal*");
+  await expect(arudha.locator('tr[data-pada="12"]')).toContainText("A12 · UL");
+
+  await toggle("argala", /Argala panelini aç\/kapat/);
+  const argala = page.getByRole("region", { name: "Argala" });
+  await expect(argala.locator('tr[data-pair="5"]')).toContainText("çekişmeli");
+  // Referans ortak seçimi izler: haritada Ay'a tıklanınca Ay'ın burcu (Terazi) referans olur.
+  await argala.getByRole("combobox", { name: "Referans" }).selectOption("selected");
+  await expect(argala.getByText(/^Bir burç seçin:/)).toBeVisible();
+  await page.locator('svg [data-graha="Moon"]').first().click();
+  await expect(argala.getByText("Referans: Ter")).toBeVisible();
+  // Ortak vurgu: sabitlenen Ay karaka tablosunda da vurgulanır.
+  await expect(karakas.locator('tr[data-role="Atma"]')).toHaveClass(/is-highlighted/);
+  await page.keyboard.press("Escape");
+
+  // Harita panelinde arudha işaretleri: on iki pada burçlarına yazılır.
+  const d1 = page.getByRole("region", { name: "Harita D1" });
+  await d1.getByRole("checkbox", { name: "Arudha padalar" }).check();
+  await expect(d1.locator(".chart-mark")).toHaveCount(12);
+  await expect(d1.locator('[data-mark="AL"]')).toHaveAttribute("data-house", "10");
+  // D9 panelinde işaret kutusu kapalıdır.
+  await expect(page.getByRole("region", { name: "Harita D9" }).getByRole("checkbox", { name: "Arudha padalar" })).toBeDisabled();
+
+  // Bakışlar panelinde rashi türü aynı raporu okur; graha bakışı istenmez.
+  await toggle("bakış", /Bakışlar panelini aç\/kapat/);
+  const drishti = page.getByRole("region", { name: "Bakışlar" });
+  await drishti.getByRole("combobox", { name: "Tür" }).selectOption("rashi");
+  await expect(drishti.locator("tbody tr")).toHaveCount(9);
+  await expect(drishti.locator('tbody tr[data-graha="Ketu"] td').nth(5)).toContainText("●");
+
+  // Her tüketici raporu bir kez ister: üç panel, harita işaretleri ve rashi matrisi.
+  await expect.poll(async () => (await jaiminiCalls()).length).toBe(5);
+
+  // Ayar: karaka şeması yedili yapılınca harita yeniden hesaplanır ve rapor yeni ayarla istenir.
+  await page.keyboard.press("Meta+Comma");
+  const settings = page.getByRole("dialog", { name: "Hesap ayarları" });
+  await settings.getByLabel("Chara karaka şeması").selectOption("Seven");
+  await expect.poll(async () => (await ipcCalls(page, "save_calc_settings")).length).toBe(1);
+  const [saved] = (await ipcCalls(page, "save_calc_settings")) as { settings: { advanced: unknown } }[];
+  expect(saved.settings.advanced).toEqual({ karaka_scheme: "Seven", jaimini_co_lords: true });
+  await settings.getByRole("button", { name: "Kapat" }).click();
+  await expect.poll(async () => (await jaiminiCalls()).length).toBe(10);
+  const last = (await jaiminiCalls()).at(-1)!;
+  expect(last.input.settings.advanced).toEqual({ karaka_scheme: "Seven", jaimini_co_lords: true });
+  // Fark rozeti ileri modül ayarını da adlandırır.
+  await expect(page.getByText(/Ayar farkı/).first()).toBeVisible();
+
+  // Paneller ve ayarları düzenle birlikte kaydedilir.
+  await expect
+    .poll(async () => {
+      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: Record<string, unknown> }[] }[] } } } | undefined;
+      const layout = saved?.viewState.panel_layout;
+      return layout?.layouts.find((l) => l.id === layout.active_layout_id)?.panels.map((p) => [p.kind, p.settings.arudha_marks ?? p.settings.reference ?? p.settings.kind ?? null]);
+    })
+    .toEqual([
+      ["chart", true],
+      ["chart", null],
+      ["dasha_strip", null],
+      ["notes", null],
+      ["jaimini_karakas", null],
+      ["arudha", null],
+      ["argala", "selected"],
+      ["drishti", "rashi"],
+    ]);
+
+  // Aynı komut paneli kaldırır; odak gövdeye düşmez.
+  await toggle("karaka", /Chara karaka panelini aç\/kapat/);
+  await expect(karakas).toHaveCount(0);
+  await expect(arudha).toBeVisible();
+  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
+});
````

- [ ] **Step 2: Kapılar**

```bash
lsof -i :1421
pnpm --dir app build 2>&1 | grep "✓ built"
pnpm --dir app e2e 2>&1 | tail -1
rm -rf app/test-results
```
Expected: `lsof` çıktısız (1421 portunda başka bir depo kopyasının Vite sunucusu açıksa Playwright onu yeniden kullanır ve eski arayüzü sınar); `✓ built` (e2e dosyaları da `tsc` ile denetlenir); `14 passed`. Test bu görevde yazıldığı hâliyle ilk koşuda geçer: sınadığı davranışlar Task 4–9'da uygulanmıştır (bu görev yalnızca uçtan uca kanıttır).

- [ ] **Step 3: Commit**

```bash
git add app/e2e/desk.spec.ts
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(e2e): jaimini panels from the palette; one report per consumer; shared selection; karaka scheme setting"
```
Expected: `grep` çıktısız.

---


### Task 11: Son doğrulama ve elle denetim kapısı

Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle çözülür ve bu görev baştan koşulur.

**Files:**
- Yok (yalnızca doğrulama). İlerleme notları depo dışıdır; `git add` edilmez.

**Interfaces:**
- Consumes: Task 1–10'un tamamı.
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
git diff 32cc752..HEAD --stat -- deny.toml Cargo.toml Cargo.lock app/package.json app/pnpm-lock.yaml app/vite.config.ts app/src-tauri/capabilities app/src-tauri/tauri.conf.json crates/store crates/jyotish-core/src/settings.rs crates/jyotish-core/src/chart.rs
```
Expected: fmt ve clippy çıktısız; testler: `jyotish-core` lib `172 passed`, golden `8 passed`, `modules_golden` `5 passed`; `store` lib `84 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `46 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `modules` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; `compute_drishti median` ve `compute_jaimini median` 10 ms'nin altında; deny `advisories ok, bans ok, licenses ok, sources ok`; MSRV `Finished`; son `git diff --stat` çıktısız (bağımlılık, yetenek, yapılandırma, `store`, ayar tipi ve `Chart` değişmedi: K1, D13).

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
Expected: `lsof` çıktısız; `Test Files  68 passed (68)`, `Tests  617 passed (617)`; `✓ built`; e2e `14 passed`; görsel `10 passed` (temel PNG'ler değişmedi: D23); betik testleri `pass 14`, `fail 0`; `git status --short` boş.

- [ ] **Step 3: Kural taramaları (`32cc752`'den bu yana)**

```bash
git diff 32cc752..HEAD -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log 32cc752..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff 32cc752..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
git diff 32cc752..HEAD --name-only | grep -vE '^(app|crates|scripts|tests|docs)/'
echo "trailers: $(git log 32cc752..HEAD --format=%B | grep -c '^Co-Authored-By: ') commits: $(git rev-list --count 32cc752..HEAD)"
git diff 32cc752..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
git diff 32cc752..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\(|window\.(print|open)\('
git diff 32cc752..HEAD -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
grep -rn '"source"' tests/golden/modules | grep -v '"source": "hand"'
ls tests/golden/modules/jaimini/*.json | wc -l
grep -rnE 'serde\(default' crates/jyotish-core/src
grep -rnE 'XMLHttpRequest|WebSocket|\bfetch\(' app/ui/modules app/ui/panels
git tag --points-at HEAD
```
Expected: `trailers:` satırındaki iki sayı eşit; `grep -ciE` satırı `0`, `ls … | wc -l` satırı `8` yazar; bunların dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur): yasaklı ad yok (üretilen fikstürde de), beklenmeyen yerde dosya yok, `console` ve yasak çağrı yok, yeni stilde hex ya da `rgb(` yok (C9), el örneklerinde `hand` dışında kaynak yok (K9), çekirdek tiplerinde `serde(default)` yok, modül ve panel kodunda ağ çağrısı yok (C1), ajan etiket atmamış.

- [ ] **Step 4: Elle denetim kapısı (`pnpm --dir app tauri dev`)**

Gerçek uygulamada, deneme danışanlarıyla (gerçek danışan verisi kullanılmaz). Her madde ilerleme notlarına "geçti" ya da bulgu olarak yazılır; bulgu varsa düzeltme commit'i ve bu görev baştan. Her maddenin başında, otomatik bir karşılığı olup olmadığı yazılıdır. **[otomatik]** maddelerin karşılığı testlerde vardır ve yürütücü ajan bunları testin adını göstererek "geçti" sayabilir; **[insan]** maddeler yalnızca gerçek uygulamaya bakan bir insan tarafından doğrulanabilir ve sahibi yoksa ona devredilir (bu devir `git push`'u engellemez; hiçbir şey yayımlanmaz).

- [ ] **[otomatik]** `⌘K` → "karaka" → "Chara karaka panelini aç/kapat": panel etkin düzenin altına eklenir, odak ona geçer ve panel görünür alana gelir; aynı komut paneli çıkarır ve odak gövdeye düşmez. "Arudha padalar" ve "Argala" için de palet öğesi vardır. (e2e `jaimini panels: …`)
- [ ] **[otomatik]** Delhi deneme haritasında karaka tablosu sekiz rol gösterir; Rahu satırı gerçek ve etkin dereceyi birlikte taşır; karakamsha Balık'tır; ev tablosu dokuz grahanın D1 ve D9 evini verir. (`JaiminiKarakasPanel.test.tsx`)
- [ ] **[otomatik]** Arudha paneli on iki padayı, AL ve UL etiketlerini, istisna işaretini ve dipnotu gösterir; "Arudha padalar" kutusu D1 haritasına on iki işaret yazar ve D9'da kapalıdır. (`ArudhaPanel.test.tsx`, `ChartPanel.test.tsx`, e2e)
- [ ] **[otomatik]** Argala paneli lagna referansıyla dört çifti ve durum metinlerini gösterir; "Seçili burç"te haritada bir grahaya tıklamak referansı o grahanın burcuna çevirir; `Esc` seçimi kaldırır. (`ArgalaPanel.test.tsx`, e2e)
- [ ] **[otomatik]** Bakışlar panelinde "Tür → Rashi drishti" dokuz satır çizer ve graha bakışı istemez; graha tablosunda "Karaka" kutusu rol sütununu ekler. (`DrishtiPanel.test.tsx`, `GrahaTablePanel.test.tsx`, e2e)
- [ ] **[otomatik]** `⌘,` → "Chara karaka şeması → Yedi karaka": harita ayarı kaydedilir, açık Jaimini tüketicileri raporu yeni ayarla yeniden ister, başlıkta "Ayar farkı" rozeti görünür. (`SettingsDialog.test.tsx`, e2e)
- [ ] **[otomatik]** Doğum saati "bilinmiyor" olan haritada: karaka paneli uyarı satırı ve iki "≈" gösterir; Arudha paneli tablo yerine tek satır açıklama gösterir; Argala paneli lagna referansında açıklama, seçili burçta tablo gösterir; harita kutusu kapalıdır. (dört panel testi; Rust `modules` testi padaların `None` döndüğünü doğrular)
- [ ] **[insan]** Gerçek yedili şema: bir haritada şema "Yedi karaka" yapılınca karaka paneli yedi satıra iner, Rahu satırı ve Pitri rolü yoktur, "Şema" satırı değişir; "Jaimini eş lordları" kutusu kapatılınca Arudha panelinin son satırı "devre dışı" der ve Akrep ile Kova evlerinin lordu Mars ve Satürn olur. (e2e taklidi raporu yeniden hesaplamaz; bu yalnızca gerçek motorla görülür.)
- [ ] **[insan]** Üç yeni panel, rashi matrisi ve harita işaretleri açık ve koyu temada, Türkçe, İngilizce ve Fransızca dilde okunur; karaka paneli 5 sütunda, argala paneli 6 sütunda yatay taşmaz, panel daraltılınca içerik panel içinde kaydırılır; harita işaretleri üç biçimde de (kuzey, güney, daire) gezegen etiketlerinin üstüne binmez ve bir burçta üç pada varken okunur.
- [ ] **[insan]** VoiceOver: rol kısaltması açılımıyla ("AK Atmakaraka") okunur; eşitlik "eşit derece", istisna "istisna", rashi işareti "rashi drishti" diye okunur; argala tablosunda graha kısaltması tam adıyla okunur; haritadaki işaretler tek tek okunmaz, haritanın altındaki "Arudha padalar: …" listesi okunur; kapalı "Arudha padalar" kutusunun nedeni duyurulur.
- [ ] **[insan]** Bilinen bir haritada (astrologun kendi elindeki bir örnek) karaka sırası, karakamsha, AL ve UL ile lagnadan argala listeleri astrologun beklediğiyle uyuşur. Uyuşmazlık bulgu olarak yazılır; yöntem farkıysa (sekizli şema varsayılanı, eş lord ölçütleri, arudha istisnasının biçimi, sayıya bakan argala durumu) "Bu planın kapsamadıkları"na göre sınıflanır.
- [ ] **[insan]** Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum verisi, konum ya da dosya yolu görünmez.

Kapı geçtiyse ilerleme notlarına `Plan 15 Task 11 elle denetim: geçti (<tarih>)` yazılır; **[insan]** maddeleri devredildiyse `Plan 15 Task 11 elle denetim: otomatik maddeler geçti, insan maddeleri sahibine devredildi (<tarih>)` yazılır. Bu adım commit üretmez.

- [ ] **Step 5: El örneklerinin yeniden türetilmesi (gözden geçirme maddesi)**

Spec §13: "`derivation` ayrı bir gözden geçirmede kuraldan yeniden hesaplanır." Gözden geçiren (motoru yazan dışında bir ajan ya da insan), `tests/golden/modules/jaimini/` altındaki sekiz dosyanın her birinde girdiden başlar ve motoru çalıştırmadan `expected` değerlerini yeniden çıkarır: (1) burçlar ve burç içi dereceler; (2) karaka sırası, Rahu'nun etkin derecesi, eşitlik işaretleri; (3) dokuz grahanın navamsa burcu (`rule_ref`'teki parça kuralı), karakamsha ve on sekiz ev sayısı; (4) Akrep ve Kova için altı ölçütün sırayla uygulanması (spec §5 ve D6, D7) ve belirleyici ölçütün numarası; (5) on iki pada ve istisnalar; (6) iki referans için dört çiftin graha listeleri, durumları ve 3. ev işareti (doğal sınıf K6'dan). Farklar yazılır. Fark varsa önce kural yeniden okunur; düzeltme hangi tarafta olursa olsun ayrı commit'tir. Görev başına gözden geçirmede yalnızca o görevin eklediği bölüm (Task 1: 1–3; Task 2: 4–5; Task 3: 6) yeniden türetilir; bu adım hepsinin yapıldığını toplar. Sonuç ilerleme notlarına `Plan 15 el örnekleri yeniden türetildi: 8/8 (<tarih>)` olarak yazılır.

---

## Başarı ölçütleri

- `jyotish-core` Jaimini çekirdeğini verir: chara karaka (iki şema, Rahu'nun etkin derecesi, eşitlik işareti), karakamsha (D1 ve D9 evleri), rashi drishti (burç matrisi ve dokuz grahanın bakışı), altı ölçütlü eş lord seçimi, on iki arudha pada (istisna kuralıyla) ve on iki referans için argala. Hepsi sekiz el örneği (kaynak `hand`), birim testleri, proptest değişmezleri ve rashi drishti için elle yazılmış ikinci tabloyla doğrulanır; doğum saati bilinmiyorsa padalar `None`'dır.
- `compute_jaimini` ortak kalıpla çalışır (tek `ChartInput`, motor iş parçacığı, `blocking`, `CommandError`), beş golden anında v1 haritasıyla çapraz denetlenir ve bütçesinin (≤ 10 ms) içindedir. `karaka_scheme` ve `jaimini_co_lords` ayarları hesaba girer ve ayar penceresinden değiştirilir.
- Üç yeni panel (Chara karaka, Arudha padalar, Argala) paletten etkin düzene eklenir ve çıkarılır; harita paneli arudha işaretlerini, Bakışlar paneli rashi türünü, graha tablosu karaka sütununu kazanır. Boş, yükleniyor, hata, dolu ve "saat bilinmiyor" durumları testlidir; ortak seçime bağlıdırlar; renk tek başına anlam taşımaz; yorum metni yoktur.
- `Chart`, `open_client`, SQLite şeması, ayar şema sürümü, baskı düzenleri ve görsel temeller değişmemiştir.
- Kapılar: `jyotish-core` lib 172, `modules_golden` 5, `jyotish-desk` `modules` 1; Vitest 68/617; e2e 14; görsel 10; `cargo deny` temiz; diğer sayılar tabandaki gibi. Elle denetim kapısının **[otomatik]** maddeleri testlerle doğrulanmış, **[insan]** maddeleri yapılmış ya da sahibine devredilmiştir.

## Bu planın kapsamadıkları

- **Spec'in 3.2 için kapsam dışı saydıkları:** graha arudhaları, karaka birleşmesi, Ketu için ters argala, güçle hükmedilen argala, Jaimini rasi güç sıralamasının tamamı, sthira karaka şeması, pada lagnaları dışındaki özel lagnalar (hora, ghati, bhava lagna).
- **Sonraki alt projeler:** chara dasha ve dasa panellerinde sistem seçimi (3.3; bu planın `jaimini_lord` ve `counts_forward` fonksiyonlarını kullanır; "iki eş lord da burcun içindeyse 12 yıl" kuralı oradadır); karaka ve pada temelli yoga kuralları (3.5); `CharaKaraka` ve `ArudhaInSign` arama koşulları (3.9).
- Eş lord seçiminde belirleyici ölçütün panelde gösterimi (`CoLordChoice.step` rapora girmez: rapor AI sözleşmesidir ve spec'teki alanlarla sınırlıdır; beta geri bildirimi isterse rapora alan eklemek ayrı bir iştir).
- Burç × burç rashi drishti tablosunun ayrı gösterimi (`JaiminiReport.rashi_drishti` raporda vardır ve AI paketleyicisine gider; panel graha × graha matrisini çizer).
- Bölüm haritalarında (D9 ve diğerleri) arudha işaretleri; arudha işaretlerinin baskıda gösterimi; graha tablosunun karaka sütununun baskıda gösterimi (K10).
- Jaimini raporunun oturum içi paylaşılan önbelleği (D16) ve veritabanında saklanması (spec §16).
- Paket parçalama (D22: 500 kB uyarısı), genel bir "panel ekle" kataloğu, hazır düzenlere modül paneli eklenmesi (K12).
- Yorum metinleri ve AI yorum paneli; Windows ve Android.
- v1'den taşınan seyrek test dalgalanması (`NotesPanel.test.tsx › New note adds an empty note…`, yük altında): bu planın dosyalarına dokunmaz; ayrı bir bakım işidir.

## Spec'e geri bildirim (spec kusurları ve boşlukları)

Plan yazılırken bulunan ve Kararlar'da çözülen maddeler. "Kusur" spec'in kendi içinde çelişen ya da harfiyen uygulanamayan yeridir; "boşluk" yazılmamış ayrıntıdır. Son incelemede `2026-11-26-alt-proje-3-ileri-moduller-design.md`'ye işlenmelidir.

1. **Boşluk — §5 Chara karaka: derecenin çerçevesi.** "Burç içi derece" tropikal-rasi kipinde tropikal mi sidereal mi, yazılmamış. Çözüm (D2): rasi çerçevesi (K5); K5'e "burç içi derece de rasi çerçevesindedir" eklenmeli.
2. **Boşluk — §5 Chara karaka: eşitlik.** "`tie: true` taşır" hangi kaydın taşıdığını, 1″ sınırının dahil olup olmadığını ve Rahu 0°'deyken etkin derecenin 30 olduğunu söylemiyor. Çözüm (D3): iki komşu da işaretlenir, sınır dahildir, `effective_deg` Rahu için `(0, 30]`.
3. **Boşluk — §5 Rashi drishti: `graha_rashi_drishti`'nin boyutu ve sırası.** Çözüm (D4): 27 kayıt, `Graha::ALL` sırasında, düğümler dahil.
4. **Kusur — §5 Eş lordlar, 5. ölçüt: "burcun sayım yönünde (3.3)".** Sayım yönü grupları §6'da (3.3) tanımlı, ama 3.2 onlara muhtaç ve bağımlılık yönü tersine (3.3 → 3.2). Çözüm (D5): gruplar ve sayım 3.2'de (`counts_forward`, `directed_count`); §6 bunlara gönderme yapmalı.
5. **Kusur — §5 Eş lordlar, 3. ölçüt: "Düğümlere bu adım uygulanmaz."** Her eş lord çiftinde bir düğüm vardır; cümle harfiyen okunursa ölçüt ölüdür ve §5 "Test"teki "altı adımın her birinin belirleyici olduğu durumlar" koşuluyla çelişir. Çözüm (D6): "Düğümlerin yücelme ve düşüş burcu yoktur; ölçüt yalnızca Mars'ın ya da Satürn'ün durumuna bakar: yücelmişse o, düşmüşse düğüm seçilir."
6. **Boşluk — §5 `ArudhaPada.lord`.** Evin lordu mu, padanın burcunun lordu mu, yazılmamış. Çözüm (D8): evin hesapta kullanılan lordu.
7. **Boşluk — §5 Karakamsha "yaklaşık işaretlenir"; `JaiminiReport.warnings`.** Raporda yaklaşıklık alanı yok ve `warnings`'in içeriği tanımsız. Çözüm (D9, D11): yaklaşıklık işareti arayüzde (v1 panelleri gibi); `warnings` yalnızca temel haritanın `LowPrecision` uyarısı. AI paketleyicisi yaklaşıklığı girdideki `time_known`'dan okur; spec bunu yazmalı ya da rapora `approximate: bool` eklemeli.
8. **Boşluk — K7: iki denetimin hangi alt projede olduğu.** "3.2 ve 3.3'tedir" ayrım yapmıyor. Çözüm (D14): ikisi de 3.2'de (`jaimini_co_lords` arudha hesabında kullanılıyor); 3.3 yeni denetim eklemez.
9. **Boşluk — §5 Argala paneli: "referans (lagna / seçili burç)".** Harita hücreleri ortak seçime ev olarak girer; "seçili burç" tanımsız. Saat bilinmiyorken lagna referansının ne göstereceği de yazılmamış. Çözüm (D17).
10. **Boşluk — §5 `arudha_marks`: hangi varga, saat bilinmiyorken ne olur.** Çözüm (D18): yalnızca D1; saat bilinmiyorsa kutu kapalı.
11. **Kusur — §5 "`graha_table` paneline isteğe bağlı karaka sütunu" ile K10.** Baskı tablosu graha tablosunun sütun listesini paylaşır; sütunu listeye eklemek baskı düzenini değiştirir. Çözüm (D20): ayrı `karaka` ayarı; spec "sütun baskıya girmez" demeli.
12. **Boşluk — §13 el örneği şeması: ayara bağlı modüller.** Şemada (`input`) ayar alanı yok; Jaimini örnekleri şemaya ve eş lord ayarına bağlı. Çözüm (D21): `input.advanced`.
13. **Boşluk — §5 Eş lordlar: 1. ve 2. ölçütün ayrıntısı.** İkisi de burcun içindeyse 1. ölçütün ayırmadığı; "diğer eş lord sayılmaz" derken öteki düğümün sayıldığı (ör. Akrep için Rahu) yazılmalı. Çözüm (D7).
14. **Boşluk — §3 tablosu "3.2 … üç panel".** 3.2 üç yeni panelin yanında var olan üç paneli de değiştirir (`chart`, `drishti`, `graha_table`) ve ayar penceresine denetim ekler; tablo satırı bunu anmalı.

## Self-Review

**1. Spec kapsamı.**

| Gereksinim (ileri modüller spec'i) | Görev |
|---|---|
| §5 Chara karaka: `Eight` (varsayılan) ve `Seven`; Rahu `30 − deg_in_sign`; Ketu karaka olmaz; eşitlik 1″, tam `f64`, doğal sıra | Task 1 (`chara_karakas`) |
| §5 Rashi drishti: yürük/sabit/çift doğalı kuralı; örnekler (Koç, Boğa, İkizler); grahaların bakışı | Task 1 (`rashi_drishti_signs`, `HAND_TABLE`, `graha_rashi_drishti`) |
| §5 Karakamsha: AK'nın D9 burcu; D1 ve D9 görünümü; lagnaya bağlı değil | Task 1 (`karakamsha`) |
| §5 Eş lordlar: altı ölçüt; `jaimini_co_lords = false` iken Mars ve Satürn; arudha ve chara dasha ortak kullanır | Task 2 (`co_lord_choice`, `jaimini_lord`) |
| §5 Arudha pada: kural, istisna (`S` ya da 7. burç → 10. burç), A1…A12 | Task 2 (`arudha_of`, `arudha_padas`) |
| §5 Argala: 2 ↔ 12, 4 ↔ 10, 11 ↔ 3, 5 ↔ 9; dört durum; 3. evde üç kötücül; 12 satır | Task 3 (`argala_row`, `argala_rows`) |
| §5 Veri modeli: `KarakaRole`, `CharaKaraka`, `ArudhaPada`, `Karakamsha`, `ArgalaStatus`, `ArgalaPair`, `ArgalaRow`, `JaiminiReport`; `compute_jaimini(input)` | Task 1–4 (alan adları spec'tekiyle birebir) |
| §5 Arayüz: `jaimini_karakas`, `arudha`, `argala` panelleri | Task 6, 7, 8 |
| §5 Arayüz: `chart` `arudha_marks`; `drishti` `rashi`; `graha_table` karaka sütunu; durumlar yalnızca metinle | Task 7, 9 |
| §5 Test: en az 8 el örneği; zorunlu durumlar (Rahu AK, 1″ eşitlik, lord 1./4./7./10. burçta, altı ölçüt, vargottama AK) | Task 1–3 (`jaimini_fixtures_cover_the_required_cases` sabitler) |
| §5 Test: property'ler (permütasyon; simetri, üç bakış, tür; pada evde ya da 7.'de değil; argalada tekrar yok) | Task 1–3 (`jaimini_properties`, `rashi` birim testleri) |
| §5 Test: beş `swetest` anında `compute_jaimini` hata vermez ve property'leri sağlar | Task 4 (`jaimini_checks`) |
| §5 Gizlilik, performans ≤ 10 ms | C1; Task 4 (ölçüm), Task 11 |
| K1 (istek üzerine; `Chart` değişmez), K2 (`time_known`; padalar `None`), K3 (saf çekirdek) | Task 3, 4 |
| K4, K5, K6 | Task 2 (rasi evleri), Task 1 (D2), Task 3 (`nature`) |
| K7 (ayarların hesapta kullanımı ve denetimleri; `method`) | Task 2, 3, 5 |
| K8 (hata yolu), §14 | Task 4 (D13), C1 |
| K9, §13 (el örnekleri, değişmezler, ikinci yazım, çapraz denetim, komut katmanı) | Task 1–4, Task 11 Step 5 |
| K10 (baskı düzeni yok), K11 (panel dili), K12 (palet) | D20, D23; D15; Task 6–8 |
| Ek A (Dara karaka; adlandırma) | `KarakaRole::Dara`; C13 taramaları |

Boşluk: yok. Spec'in istemediği ama planın eklediği parçalar: `CoLordChoice.step` (test edilebilirlik, D7), Argala panelindeki on iki burç düğmesi ve grahadan burç seçimi (D17), haritadaki işaretlerin ekran okuyucu listesi (D18).

**2. Yer tutucu taraması.** "TODO", "TBD", "daha sonra", "uygun hata işleme" yok. Task 11'deki `<tarih>` denetimin yapıldığı günün yazılacağı yerdir. Her kod adımı tam dosya ya da `32cc752` ağacına (ve önceki görevlere) karşı üretilmiş diff taşır.

**3. Tip tutarlılığı.**
- `JaiminiReport` alan adları (`scheme`, `co_lords`, `method`, `karakas`, `karakamsha`, `rashi_drishti`, `graha_rashi_drishti`, `arudhas`, `argala`, `warnings`) Task 3'te Rust'ta tanımlanır; TS bağlaması Task 4'te üretilir; paneller (Task 6–9), `ui_fixture.rs`, `tests/modules.rs` ve e2e aynı adları okur.
- `CharaKaraka { role, graha, effective_deg, tie }`, `Karakamsha { atmakaraka, sign, houses_in_d1, houses_in_d9 }`, `ArudhaPada { house, sign, lord, exception }`, `ArgalaPair { argala_house, virodha_house, argala_grahas, virodha_grahas, status }`, `ArgalaRow { reference, pairs, third_house_malefic_argala }`: Rust tipleri, el örneği düzeneğindeki `Expected*` yapıları, README satırı ve sekiz JSON dosyası aynı alan adlarını kullanır.
- `KarakaRole` üyeleri (`Atma` … `Dara`) Rust'ta, `EIGHT_KARAKA_ROLES` / `SEVEN_KARAKA_ROLES` sabitlerinde, JSON'larda ve TS `KARAKA_ROLES` kaydında aynıdır (`Record<KarakaRole, …>` eksik üyeyi derlemede reddeder).
- `ArgalaStatus` üyeleri (`None`, `Stands`, `Contested`, `Obstructed`) Rust'ta, JSON'larda ve i18n `argala.status.*` anahtarlarında aynıdır.
- `JAIMINI_METHOD = "jaimini.core_v1"` Rust'ta, `modules_golden.rs`'te, `tests/modules.rs`'te ve üç panel testindeki "Yöntem: jaimini.core_v1" metninde aynıdır.
- `co_lord_choice` / `CoLordChoice { lord, step }` Task 2'de tanımlanır; düzenekteki `ExpectedCoLord { lord, step }` ve JSON'lardaki `co_lords` aynı adları kullanır.
- Panel türleri (`jaimini_karakas`, `arudha`, `argala`) `PANEL_KINDS`, `MODULE_PANELS`, `PANEL_REGISTRY`, i18n `panel.*`, `model.test.ts` ve e2e'deki bölge adlarıyla (Chara karaka, Arudha padalar, Argala) aynıdır; boyutlar (5 × 14, 4 × 12, 6 × 9) `model.ts` ve panel testlerindeki `Panel` nesnelerinde aynıdır.
- Panel ayar anahtarları: harita `arudha_marks`; Argala `reference`; Bakışlar `kind` (ve Plan 14'ün `value`'su); graha tablosu `karaka`. Testler ve e2e aynı anahtarları kaydedilen düzende arar.
- `useJaimini(opened, enabled?)` Task 6'da tanımlanır; Task 7 (`ArudhaPanel`, `ChartPanel`), Task 8 ve Task 9 aynı imzayla çağırır. `PinButton({ target, label?, children })` ve `arudhaLabel(house)` için de aynısı.
- `ChartMark { sign, text }` ve `ChartData.marks` Task 7'de tanımlanır; `ChartSvg` ve `ChartPanel` aynı adları kullanır.
- i18n: `jaimini.method` üç yeni panelde ortak; `settings.karakaSchemes.*` hem ayar penceresinde hem karaka panelinin "Şema" satırında; `argala.pickSign`, `arudha.timeUnknown`, `jaimini.timeUnknown`, `argala.noLagna` metinleri testlerde birebir geçer.

**4. Kural denetimi.** C1 (`ChartInput` loglanmaz; panik iletisi taşınmaz), C2′ (komut dizgi almaz), C3 (`store` ve ayar tipi değişmez), C4 (karaka, pada, argala ve rashi drishti Rust'tan; biçimleme ve ev → burç eşlemesi gösterim), C7 (kısayol eklenmez), C9 (`=`, `*`, `●`, "≈" ve metin; yeni stilde hex yok), C10 (iki üretilen dosya komutla), C12 (her yeni anahtar üç dilde), C13 (her commit'te tarama; "karaka" çoğulu ve camelCase tuzağı Global Constraints'te), K9 (yalnızca `hand`), K10 (baskı ve sütun listesi değişmez) görevlerin adımlarında ve Task 11 Step 3'te denetlenir.

**5. Koşulan doğrulama.** Bu plandaki kod, deponun `32cc752` üzerindeki bir kopyasında (kısa bir yolda) on commit olarak uygulanmış ve her commit'te ölçülmüştür: `jyotish-core` lib 151 → 164 → 172; `modules_golden` 5; `jyotish-desk` `modules` 1 (`compute_jaimini` ortancası 0,7–0,8 ms, `compute_drishti` 0,6–0,8 ms); Vitest 571 → 574 → 585 → 599 → 608 → 617; e2e 14. Son durumda `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`, `cargo deny check` (dört `ok`), 1.93 ile `cargo check`, `pnpm build`, görsel 10 ve yayın betiği testleri 14 yeşildir; Task 11 Step 3'ün taramaları beklendiği gibidir. "Testlerin başarısız olduğunu doğrula" adımlarındaki çıktılar da aynı kopyada, bir önceki görevin ağacına yalnızca test dosyaları konarak ölçülmüştür. Paket boyutu Task 7'de 499,5 kB, Task 8'de 505,6 kB'dir (D22). Ayrıca bu belgedeki bloklar (yeni dosyalar ve diff'ler) belgeden okunup sırayla temiz bir `32cc752` ağacına uygulanmış ve her görevin sonunda o commit'in ağacıyla birebir aynı sonucu vermiştir (üretilen iki dosyayı adımlardaki komutlar üretmiştir; onlar da aynıdır).

El örneklerinin sekizi de (üç görevdeki hâlleriyle) motorun ilk koşusunda geçmiştir; hiçbir beklenen değer motor çıktısına bakılarak değiştirilmemiştir. Yazım sırasında testlerde düzeltilenler: bir karaka birim testinin girdisinde Jüpiter'in derecesi (Aslan 22°) Mars'ınkini (20°) geçiyordu ve yorumdaki sırayla çelişiyordu; koşudan önce elle denetimde görülüp girdi düzeltildi. Arudha birim testinde istisna evleri burç sırasıyla toplandığı için liste `[1, 4, 7, 10]` sırasında değildi; test listeyi sıralar hâle getirildi (beklenti değişmedi). Karaka paneli testinde bir metin sorgusu iki öğeyle eşleşiyordu; sorgu sınıf seçicisiyle değiştirildi. **Koşulmayanlar:** `pnpm tauri dev` ile gerçek uygulama ve gerçek veritabanı (Task 11 Step 4'ün **[insan]** maddeleri; özellikle gerçek motorla yedili şema ve devre dışı eş lordların panellere yansıması); el örneklerinin bağımsız bir gözden geçirenle yeniden türetilmesi (Task 11 Step 5); uzak CI (hiçbir şey gönderilmedi).
