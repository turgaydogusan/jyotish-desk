# Plan 23 — Doğum anı ekleri: panchang ekleri (yogi noktası, yogi, eş yogi, avayogi, dagdha rasi), navatara tablosu ve Vimshopaka bala

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Doğum haritası okumasının başında bakılan üç tabloyu eklemek; var olan hiçbir hesabın davranışı değişmez. (1) **Panchang ekleri:** yogi noktası, yogi, eş yogi, avayogi ve doğum tithisinin dagdha burçları; Panchang panelinin ikinci tablosu olarak. (2) **Navatara:** doğum Ay'ının nakshatrasından sayılan dokuz tara, üç turda 27 nakshatra, her nakshatranın lordu ve orada duran doğum grahaları; altında seçili transit anındaki dokuz grahanın tarası; yeni `navatara` paneli. (3) **Vimshopaka bala:** yedi grahanın dört varga kümesindeki (shadvarga, saptavarga, dashavarga, shodashavarga) yirmi üzerinden değeri ve on altı vargadaki basamağı; yeni `vimshopaka` paneli. Üç katman: `jyotish-core`'da yeni `natal_extras` modülü, uygulamada üç komut, arayüzde bir ek tablo ve iki panel. Yorum, hüküm ve sıralama yoktur.

**Architecture:**
- **Tek yeni çekirdek modülü: `crates/jyotish-core/src/natal_extras/`.** Beş dosya: `yogi.rs`, `dagdha.rs`, `navatara.rs`, `vimshopaka.rs` (saf fonksiyonlar; ephemeris görmezler, K3) ve `collect.rs` (üç toplayıcı). Girdi `Positions`'tır; var olan modüllerden yalnızca okunur: `panchang::{elongation, tithi_no}`, `dasha::VIMSHOTTARI_ORDER`, `dignity` (lordlar ve bileşik ilişki), `shadbala::rasi_lon`, `varga::varga_position`, `muhurta::tara_bala`, `transit::compute_transits`. `panchang.rs`, `dasha.rs`, `dignity.rs`, `varga.rs` ve `shadbala/sthana.rs` **değişmez**.
- **Sonuç saklanmaz ve `Chart`'a eklenmez (K1).** Üç komut (`compute_panchang_extras`, `compute_navatara`, `compute_vimshopaka`) panel açıkken çağrılır; girdi ortak `ChartInput`'tur. Veritabanı şeması değişmez (Y6).
- **Üç rapor da lagna istemez.** Doğum saati bilinmiyorsa rapor yine döner ve Rust'ın `approximate` alanı doludur (K2'nin `None` kuralı lagnaya bağlı çıktı içindir; burada lagnaya bağlı çıktı yoktur). Arayüz "yaklaşık" bilgisini bu alandan okur.
- **Tek okuma, yöntem kimliğinde (Y5).** `panchang_extras.v1`, `navatara.moon_v1`, `vimshopaka.bphs_v1`. Varyant noktaları (dagdha rasi tablosunun paksha ayrımı, avayoginin sayımı, navataranın lagnadan sayımı, Vimshopaka'da yücelme ve moolatrikonanın yeri, geçici dostluğun okunduğu varga) Kararlar'da tek okumayla çözülür ve `tests/golden/modules/natal_extras/REVIEW.md`'de sahibinin onayına yazılır (Y4). Bellekten yazılan iki tablo ailesi (dagdha rasi; Vimshopaka ağırlıkları ve basamakları) onaya dek panellerde "doğrulanmamış tablo" notuyla sunulur.
- **Arayüz hesap yapmaz (C4).** Paneller Rust'tan gelen olguları yazar. Navatara'nın 27 satırını dokuz tara × üç tur olarak dizmek ve küme değerini üç ondalıkla yazmak gösterimdir. Transit anı transit deposundan okunur ve Rust'a olduğu gibi gönderilir.
- **Eşzamanlı planlarla çakışmayı küçük tutmak.** Plan 24 (avasthalar) ve Plan 25 (bhava chalit, Sudarshana) aynı anda ayrı git worktree'lerinde yürür. Bu plan yeni kodu yeni dosyalara koyar (çekirdek modülü, `natal_extras_commands.rs`, `tests/natal_extras.rs`, üç yeni bileşen, tek kanca dosyası) ve paylaşılan dosyalara yalnızca ek satır yazar. Ayrıntı ve birleştirme kuralı "Paylaşılan dosyalar" bölümündedir.
- **Testler:** çekirdekte her kural için el örneği ve ikinci yazım (dagdha tablosu burçtan; avayogi eklenen yayla; tara adım adım sayımla; Vimshopaka iki tam el örneği ve ağırlık değişmezi), sahte ephemeris'le toplayıcı testleri; uygulamada gerçek motorla beş golden anda ve iki zodyak kipinde çapraz denetim (tithi ve nitya yoga v1 panchangıyla, varga burçları v1'in varga haritalarıyla, basamaklar Shadbala'nın saptavargaja kalemiyle) ve bütçeler; arayüzde Vitest ve bir uçtan uca akış.

**Tech Stack:** Rust 1.93+ (stable; yerelde ve CI'da 1.100), Tauri 2.12, tauri-specta 2.0.0-rc.29, serde, React 19, TypeScript 7, Vitest 5, Playwright 1.65.1. **Yeni Rust ya da npm bağımlılığı yok.**

**Spec:** `docs/superpowers/specs/2026-11-27-eksikler-yol-haritasi.md` (aşağıda "yol haritası"): §1 (taşınan kurallar), §2 (Y1–Y8), §4.1 (panchang ekleri), §4.2 (navatara), §4.3 (Vimshopaka bala), §5 "Plan 23" (görev taslağı ve sahibin denetimi), Ek A (adlandırma). İleri modüller spec'i (`2026-11-26-alt-proje-3-ileri-moduller-design.md`, aşağıda "İM spec"): §2 (K1–K12), §4 (`ChartInput`, `Positions`), §13 (test stratejisi), §14 (gizlilik), §15 (bütçeler). UI spec: `2026-11-24-ui-design.md` §3.3–§3.4, §5, §8. Yol haritası bir tasarım belgesi değildir (başlığındaki durum notu): bu planın tasarımı Architecture ve Kararlar bölümleridir.

**Ön koşul:** Taban, yol haritasının yazıldığı uçtur. Bu plan yazılırken `main` commit `7270550`'deydi (Plan 22 ve "Klasik" hazır düzeni dahil); kod blokları ve sayaçlar o ağaca göre yazılmış ve ölçülmüştür. `main` bu commit'ten ilerlediyse (özellikle Plan 24 ya da Plan 25 önce birleştiyse) Task 1 Step 1 tabanları yeniden ölçer ve sayaç zinciri o ölçümden yürütülür (farklar aynıdır); bir `diff` bloğunun bağlamı tutmazsa önce "Paylaşılan dosyalar" bölümündeki kural okunur. Önceki planların elle denetim kapılarının yalnızca insana ait maddeleri bu planı engellemez (Y3).

## Kararlar

Sahibi kararları devretti; muhafazakâr seçenek seçildi ve her karar bir satır gerekçeyle yazıldı. "Yol haritası farkı" ya da "Yol haritası boşluğu" diye işaretlenenler yol haritasında düzeltilmesi gereken yerlerdir ve belgenin sonundaki "Yol haritasına geri bildirim" bölümünde aynı numarayla toplanmıştır. D17'deki ölçümler ve sayaç zinciri bu plan yazılırken deponun bir kopyasında, her görev ayrı commit olarak uygulanarak alınmıştır (macOS 27.2, Rust 1.100, hata ayıklama derlemesi).

- **D1 — Çekirdek modülü `crates/jyotish-core/src/natal_extras/`'tır: `yogi.rs`, `dagdha.rs`, `navatara.rs`, `vimshopaka.rs`, `collect.rs`. Yöntem kimlikleri `PANCHANG_EXTRAS_METHOD = "panchang_extras.v1"`, `NAVATARA_METHOD = "navatara.moon_v1"`, `VIMSHOPAKA_METHOD = "vimshopaka.bphs_v1"`'dir.** Gerekçe: yol haritası çekirdeği `panchang_extras` diye anar ama modülü adlandırmaz; üç kalemin tek modülde durması paylaşılan `lib.rs`'e tek `pub mod` ve tek `pub use` bloğu demektir. Y5 tek okumanın `method`'a yazılmasını ister. **Yol haritası boşluğu 1.**
- **D2 — Yogi noktası `normalize(Güneş + Ay + 93°20′)`'dir (sidereal boylamlar). Nokta ve nakshatrası her zodyak kipinde sidereal boylamdandır; noktanın burcu ve burç içi derecesi rasi çerçevesindedir (K5). Çerçeve ötelemesi `Positions`'tan, Güneş'in iki boylamının farkından okunur (`rasi_frame_offset`); 10⁻⁹°'den küçük fark sıfır sayılır.** Gerekçe: nitya yoga sidereal toplamdır; "burca dayalı her kural rasi çerçevesindedir" (K5) eş yogiyi de kapsar. `Positions` ayanamsayı taşımaz ve tipine alan eklemek paylaşılan bir dosyayı değiştirirdi. **Yol haritası boşluğu 2.**
- **D3 — Avayogi, yogi nakshatrasından altıncı nakshatranın lordudur (yogi nakshatrası 1 sayılır); rapor o nakshatrayı da taşır (`avayogi_nakshatra`). Öteki yazım (noktaya 186°40′ eklemek) uygulanmaz, testte ikinci yazım olarak durur: varılan nakshatra farklıdır (on beşinci), lord aynıdır.** Gerekçe: yol haritası §4.1 "ikisi aynı lordu vermeli: ikinci yazım testi" der.
- **D4 — Eş yoginin alan adı `co_yogi`'dir; arayüzde Türkçe "Eş yogi", İngilizce "Duplicate yogi", Fransızca "Co-yogi" yazılır.** Gerekçe: yol haritasının Türkçe terimi; İngilizce karşılığı yerleşik kullanımdır.
- **D5 — Dagdha rasi tek tablodur (paksha içi tithi 1–15 → burçlar) ve Shukla ile Krishna paksha aynı satırı okur; chaturdashide dört burç, dolunayda ve yeni ayda boş liste. Tablo bellekten yazılır, `REVIEW.md` §1.3'te hücre hücre ve §1.4'te burçtan ikinci kez durur; panel "doğrulanmamış tablo" notu taşır.** Gerekçe: yol haritası §4.1 paksha ayrımını varyant noktası sayar; Y5 tek okuma ister, Y4 onaya dek not ister. Rapor yalnızca burçları verir: o burçlarda duran grahaları ya da lordlarını saymak yoruma kapı açardı.
- **D6 — Panchang ekleri `Panchang` tipine ve `Chart`'a eklenmez; ayrı komuttan gelir (`compute_panchang_extras`) ve Panchang panelinin **ikinci tablosu** olarak çizilir (`panels/PanchangExtras.tsx`). Beş uzvun tablosu komut beklemeden çizilir; ek tablonun hatası beş uzvu kaldırmaz.** Gerekçe: K1 ve C14 (`panchang.rs`'e dokunulmaz). Yol haritası "Panchang paneline ek satırlar" der; aynı tabloya satır eklemek "Bitiş" sütununu boş bırakır ve panelin komutsuz çizimini bozardı. **Yol haritası farkı 3.**
- **D7 — Doğum saati bilinmiyorsa (`time_known` yanlış) üç rapor da döner ve `approximate: true` taşır; sayı prashnasında (`lagna_number`) raporlar sayısız haritayla aynıdır.** Gerekçe: yol haritası §4.1 "saat bilinmiyorsa değerler 'yaklaşık' işaretini alır" der; üç kalemde de lagnaya bağlı çıktı yoktur (K2 uygulanacak alan bulmaz). Arayüz işareti `timeUnknown(opened)`'dan değil Rust'ın alanından okur.
- **D8 — Navatara yalnızca doğum Ay'ının nakshatrasından sayılır; lagna nakshatrasından sayım uygulanmaz ve raporda lagna yoktur.** Gerekçe: yol haritası §4.2'nin önerisi; Y5 (ikinci okuma ancak uygulanınca ayar olur).
- **D9 — Navatara satırı taranın numarasını ve elverişli kümesini muhurta modülünün `TaraBala` tipiyle taşır (`tara_bala`); yeni tara tablosu, yeni ad listesi ve "doğrulanmamış tablo" notu yoktur. Tur (`cycle`, 1–3) yeni bir alandır (`tara_cycle`).** Gerekçe: yol haritası §4.2 "yeni tablo getirmez"; adlar ve küme `muhurta/REVIEW.md`'de onay bekliyor ve arayüzde `TARA_NAMES` zaten var.
- **D10 — Rapor 27 satırdır (doğum nakshatrasından başlar); panel bunu dokuz satır × üç tur olarak çizer (her satır bir tara; lord üç nakshatrada aynıdır).** Gerekçe: klasik tablonun okunuşu budur; dizme gösterimdir (C4).
- **D11 — Geçiş tarası aynı komuttadır: `compute_navatara(input, transit_jd_ut: Option<f64>)`. An, transit deposunun anıdır (`TransitView.snapshot.jd_ut`); konumlar `compute_transits` ile, haritanın ayarlarıyla hesaplanır. Geçiş anı haritadan önce denetlenir ve hatası yalnızca alan adını taşır: `CoreError::InvalidTransitInput("transit_jd_ut")` → `invalid_transit_input { field }`. Panel transit anı seçicisini (`TransitDate`) de taşır.** Gerekçe: yol haritası "ikinci görünüm: seçilen geçiş anında dokuz grahanın tarası"; "şimdi"yi çekirdek bilmez; arayüz an hesabı yapmaz (C4). Var olan kod yeniden kullanılır, yeni hata kodu gerekmez. **Yol haritası boşluğu 4.**
- **D12 — Vimshopaka'nın dört kümesi ve ağırlıkları: shadvarga (D1 6, D2 2, D3 4, D9 5, D12 2, D30 1), saptavarga (D1 5, D2 2, D3 3, D7 2½, D9 4½, D12 2, D30 1), dashavarga (D1 3, D60 5, D2, D3, D7, D9, D10, D12, D16, D30 birer buçuk), shodashavarga (D1 3½, D2 1, D3 1, D9 3, D16 2, D30 1, D60 4, öteki dokuz varga yarımşar). Her kümenin toplamı 20'dir (değişmez testi).** Gerekçe: yol haritası §4.3 (a); BPHS'nin değerleri bellekten yazılmıştır ve `REVIEW.md` §3.2'de onay bekler.
- **D13 — Basamaklar: kendi burcu 20, çok dost 18, dost 15, nötr 10, düşman 7, çok düşman 5. Yücelme, moolatrikona ve düşüş ayrı basamak değildir: kendi burcunda olmayan graha, varga burcunun lorduyla bileşik ilişkisinden okunur. Basamak IPC'de var olan `Dignity` tipinin altı değeriyle taşınır (yeni tip yok).** Gerekçe: yol haritası §4.3 (b) yaygın diziyi yazar ve yücelme ile moolatrikonanın yerini varyant sayar; muhafazakâr okuma, dizinin altı basamağından fazlasını uydurmamaktır. Sonuç: Ay Boğa'da (moolatrikona burcu, kendi burcu değil) Venüs'le ilişkisinden okunur; bu `REVIEW.md` §3.1 madde 4'te açıkça yazılıdır.
- **D14 — Geçici dostluk her vargada D1 burçlarından okunur (Shadbala'nın saptavargaja kalemindeki okuma); vargalar rasi çerçevesindeki boylamdan hesaplanır (K5).** Gerekçe: yol haritası "Shadbala'da D1 seçilmişti"; iki modülün aynı ilişkiyi iki ayrı okumayla vermesi kullanıcıyı şaşırtırdı. Bu seçim bir çapraz denetim de verir: yedi vargada basamaklar `saptavargaja_points` ile birebir eşlenir (Task 5).
- **D15 — Rapor yedi grahayı taşır (Güneş … Satürn); Rahu ve Ketu'ya değer verilmez. Her graha için on altı varganın burcu, basamağı ve değeri ile dört kümenin değeri döner; ağırlıklar rapora girmez (sabit tablodur, `REVIEW.md`'de ve `VargaScheme::weights`'te durur).** Gerekçe: yol haritası §4.3.
- **D16 — Küme değerleri arayüzde üç ondalıkla, yuvarlanmadan yazılır ("11.725").** Gerekçe: ağırlıklar yarımın, basamaklar bütünün katıdır; değer 0,025'in katıdır. İki ondalık yuvarlama gerektirir ve ikili kayan noktada 11,725 "11.72" ya da "11.73" yazılabilir.
- **D17 — Bütçe: üç komutun her biri, temel harita hesabı dahil, hata ayıklama derlemesinde ≤ 20 ms; testte `assert!` ile doğrulanır.** Ölçüm (boş makine, 15 koşunun ortancası): `compute_panchang_extras` 0,57 ms, `compute_navatara` (geçiş anıyla) 0,63 ms, `compute_vimshopaka` 0,61 ms. Gerekçe: yol haritası bütçe yazmaz; İM spec §15'in tablosunda bu büyüklükteki komutlar 10–20 ms sınıfındadır. Üst sınıf seçildi, çünkü bu plan yazılırken öteki worktree'lerin derlemeleri sürerken (yük ortalaması 90'ın üstünde) ortanca 5 ms'ye çıktı ve `cargo test --workspace` bir koşuda başarısız bir sonuç satırı verdi (hangi testin düştüğü saptanamadı; önceki koşular ve sonraki üç koşu temizdi). Bütçe testi yük altında düşerse makine boşken yeniden koşulur; bütçe gevşetilmez. **Yol haritası boşluğu 5.**
- **D18 — El örnekleri Rust birim testleridir (modül dosyalarındaki `mod tests`; türetme test yorumunda); `tests/golden/modules/` altına JSON fikstür eklenmez ve `modules_golden.rs` ile `tests/golden/modules/README.md` değişmez. Onay kaydı tek dosyadır: `tests/golden/modules/natal_extras/REVIEW.md` (Task 2'de kurulur, Task 3, 4 ve 8'de eklenir).** Gerekçe: Plan 22 D9'un yolu; `modules_golden.rs` üç eşzamanlı planın ortak dosyası olurdu. Y4 `tests/golden/modules/<modül>/REVIEW.md` der; modül `natal_extras`'tır. K9 geçerlidir.
- **D19 — Komutlar yeni dosyadadır (`app/src-tauri/src/natal_extras_commands.rs`); gerçek motorlu denetimler yeni bir entegrasyon testindedir (`app/src-tauri/tests/natal_extras.rs`: ayrı süreç, kendi motoru). `module_commands.rs` ve `tests/modules.rs` değişmez.** Gerekçe: paylaşılan dosyalara dokunmamak; her `tests/*.rs` ayrı süreçtir ve "süreçte tek SE örneği" kuralı korunur.
- **D20 — Yeni hata kodu yoktur: `error.rs`, `commands.rs` ve `errors.*` metinleri değişmez.** Gerekçe: üç komutun girdisi var olan denetimlerden geçer; tek yeni denetim (geçiş anı) var olan `invalid_transit_input` kodunu kullanır (D11).
- **D21 — Panel kataloğuna iki tür eklenir: `navatara` (7 × 16) ve `vimshopaka` (7 × 12); paletten eklenir ve çıkarılır (K12). Panchang panelinin varsayılan boyutu ve hazır düzenler değişmez; ek tablo panel içinde kaydırmayla okunur.** Gerekçe: hazır düzenlerin ve görsel temellerin oynamaması; boyutun büyütülüp büyütülmeyeceği Task 8'in **[insan]** maddesidir.
- **D22 — Vimshopaka panelinin tek ayarı `detail`'dir (`summary` | `vargas`) ve düzenle birlikte saklanır. Doğum saati bilinmiyorsa `vimshopaka.timeUnknown`, saat "yaklaşık" girilmişse (`time_certainty === "Approx"`) `vimshopaka.timeApprox` notu görünür.** Gerekçe: yol haritası §4.3 "saat 'yaklaşık' ise uyarı". Saat kesinliği doğum kaydındandır; ona bakmak gösterim kararıdır (C4).
- **D23 — Arayüz fikstürü tek bir anahtar kazanır: `natal_extras { panchang_extras, navatara, vimshopaka, schemes }` (Delhi haritası; navatara fikstürdeki transit anıyla; `schemes` Rust `VargaScheme::ALL`). E2e taklidi üç komutu fikstürden verir.** Gerekçe: arayüz hesap yapmaz; testler gerçek motorun ürettiği veriyi çizer.
- **D24 — Adlandırma: bu planın getirdiği adların hiçbiri yasak diziyi içermez. Türkçe metinlerde dizi şu sözcüklerle oluşur ve yerlerine ötekiler yazılır: "… durur", "geri", "öteki", "sürer", "örnek" (çoğul ekiyle yasak diziyi üreten sözcük yerine), "dk". Klasik terimler çoğul ekiyle bitişik yazılmaz ("Vimshopaka bala değerleri", "chara karaka rolleri").** Gerekçe: v1 spec §10, İM spec Ek A, yol haritası Y8 ve Ek A; yasak harf dizisi düzeyindedir.
- **D25 — Yeni görsel temel (PNG), yeni stil dosyası ve yeni baskı düzeni yoktur (K10); paneller var olan `module-table`, `module-toolbar`, `bala-scroll`, `panel-notice` ve `panel-message` sınıflarını kullanır. `pnpm build` Vite'ın 500 kB parça uyarısını basmayı sürdürür ve kapılar `✓ built` satırını arar.** Gerekçe: on görsel temel bu panelleri taşımaz; `panels.css` paylaşılan dosyadır.

## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni bağımlılık yok: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `app/package.json`, `app/pnpm-lock.yaml` ve `app/vite.config.ts` değişmez; `cargo deny check` Task 8'de `advisories ok, bans ok, licenses ok, sources ok` verir.
- **Yasaklı ad kuralı (v1 spec §10 "İsim kuralları", ilk madde; İM spec Ek A; yol haritası Y8):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Kural harf dizisi düzeyindedir: Türkçe sözcüklerin içinde de geçemez (D24). Her commit'ten önce eklenen satırlar şu komutla taranır ve çıktı boş olmalıdır: `git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'`.
- **Clean-room ve yorum metni yok:** kurallar klasik tanımdan özgün olarak uygulanır; başka ürünün adı, düzeni, metni ya da çıktısı kullanılmaz; kaynak gösterimi geneldir ve alıntı yapılmaz. Üç rapor olgu döner: **"iyi/kötü" hükmü, sıralama, renkli derecelendirme ve bir olgunun ne anlama geldiğini anlatan metin eklenmez.** Taranın "elverişli küme" bilgisi muhurta modülünün var olan olgusudur ve sözle yazılır.
- **Bellekten yazılan tablolar (Y4):** dagdha rasi tablosu ile Vimshopaka'nın ağırlık ve basamak tabloları `REVIEW.md`'ye bilinen varyant noktalarıyla yazılır ve onaya dek panelde "doğrulanmamış tablo" notu durur. **Yürüten ajan tabloları başka bir yazılımla ya da çevrimiçi kaynakla karşılaştırmaz** ve tabloyu "düzeltmez"; şüphe ilerleme notlarına yazılır.
- **Çerçeveler (K4, K5; D2, D14):** burçlar rasi çerçevesindedir (`settings.zodiac`'ı izler); nakshatra, pada, tithi ve yogi noktası sidereal boylamdandır; vargalar rasi çerçevesindeki boylamdan hesaplanır. Bu planda ev (bhava) kullanılmaz.
- **Doğum saati bilinmiyorsa (K2, D7):** lagna hesaplanmaz; üç rapor lagna istemediği için döner ve `approximate` taşır. Arayüzün gizlemesine ya da tahminine güvenilmez: işaret Rust'tan gelir.
- **Gizlilik (İM spec §14):** (1) Girdi (`ChartInput`: an, yer, ayar), raporlar ve transit anı hiçbir yerde loglanmaz. (2) Bu planın yeni girdi hatası yalnızca alan adını taşır (`invalid_transit_input { field: "transit_jd_ut" }`). (3) Hiçbir modül ağ isteği yapmaz. (4) Testler ve fikstür yalnızca deneme verisi kullanır ("Örnek Danışan"; gerçek kişi verisi yoktur).
- **Denetim sırası:** geçiş anı, haritadan ve motordan önce denetlenir (D11).
- **Yeniden okuma:** aynı haritanın yeniden okunmasında (ayar değişimi, yeni transit anı) tablo DOM'dan çıkarılıp yeniden kurulmaz (`useLastData`) ve odak gövdeye düşmez; hatadan sonraki okuma hata öncesi veriyi göstermez. Raporlar panel açılmadan istenmez.
- **Bütçeler testte doğrulanır** (D17): `assert!` ile; `eprintln!` yalnızca ölçümü görünür kılar ve yalnızca test dosyasındadır.
- **El örnekleri (İM spec §13, K9):** beklenen değer kuraldan elle türetilir ve test yorumunda yazılıdır. Bir el örneği motorla uyuşmazsa önce kural yeniden okunur; **beklenen değer motor çıktısına bakarak düzeltilmez.** (Bu plan yazılırken çekirdeğin 25 testi ilk koşuda geçti; arayüz testlerinde iki beklenti test yazımından düştü: aynı adı taşıyan iki düğme için tek öğe sorgusu ve fikstürden tahminle yazılmış bir varga satırı. İkincisi kuraldan yeniden türetildi; türetme testin yorumundadır.)
- Kapılar: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (1.100), `cargo test --workspace`, `pnpm --dir app test`, `pnpm --dir app build` her görev sonunda yeşil; `pnpm --dir app e2e` Task 7 ve 8'de. `tsc` `strict`. `pnpm build` çıktısında aranan satır `✓ built`'tir (D25). Tek bir Vitest dosyası `pnpm --dir app exec vitest run ui/…` ile koşulur: yol `app/` dizinine göredir. `cargo test --workspace` çıktısı uzun olduğu için başarısızlık şu sayaçla denetlenir ve `0` yazmalıdır: `cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."`.
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez). Node 24+, pnpm 11.8.0. Depo kısa bir yolda durmalıdır: Swiss Ephemeris 242 bayttan uzun ephemeris yolunu reddeder.
- **Çalışan `pnpm tauri dev` durdurulmaz** (port 1420); e2e 1421'i kullanır. **Eşzamanlı worktree'ler aynı portu paylaşır:** `playwright.config.ts` var olan sunucuyu yeniden kullanır (`reuseExistingServer`), yani 1421'de başka bir worktree'nin Vite'ı çalışıyorsa testler onun arayüzünü sınar. E2e'den önce `lsof -i :1421` çıktısız olmalıdır; doluysa öteki worktree'nin koşusu beklenir, süreç öldürülmez. `pkill -f` kullanılmaz. Üretilen `.app` ve gerçek uygulama çalıştırılmaz. `~/Library/Application Support/io.github.turgaydogusan.jyotishdesk` dizinine dokunulmaz.
- Yalnızca macOS hedeflenir. Yorum metni ve AI paneli bu planın dışındadır (Y7).
- **Tabanlar (`7270550`):** `jyotish-core` lib 404, golden 8, `modules_golden` 17; `atlas` `moment_fixed_lmt` 11, `moment_iana` 13 (`build` 6, `search` 8, `shipped` 3); `store` lib 122 (+1 `#[ignore]`); `jyotish-desk` lib 58, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `Test Files  88 passed (88)`, `Tests  887 passed (887)`; e2e 27; görsel 10 (yerel, darwin); yayın betiği testleri 14.
- **Sayaç zinciri** (her satır görev bitince; boş hücre "önceki satırla aynı"). Bu plan yazılırken `7270550`'nin bir kopyasında her görev ayrı commit olarak uygulanıp ölçülmüştür.

  | Görev | `jyotish-core` lib | `jyotish-desk` `natal_extras` | Vitest dosya / test | e2e |
  |---|---|---|---|---|
  | Taban | 404 | (yok) | 88 / 887 | 27 |
  | Task 1 | 410 | | | |
  | Task 2 | 414 | | | |
  | Task 3 | 419 | | | |
  | Task 4 | 425 | | | |
  | Task 5 | 429 | 1 | 88 / 887 | |
  | Task 6 | | | 90 / 902 | |
  | Task 7 | | | 91 / 910 | 28 |

  Değişmeyenler: `jyotish-core` golden 8, `modules_golden` 17; `atlas` 11 / 13, `build` 6, `search` 8, `shipped` 3; `store` lib 122 (+1 `#[ignore]`); `jyotish-desk` lib 58, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; görsel 10; yayın betiği testleri 14. `jyotish-core` lib 25 test kazanır: `natal_extras/yogi.rs` 6, `dagdha.rs` 4, `navatara.rs` 5, `vimshopaka.rs` 6, `collect.rs` 4. `jyotish-desk` yeni bir test ikilisi kazanır (`tests/natal_extras.rs`, tek test). Vitest Task 5'te sayı değiştirmez (üç sarmalayıcı `api.test.ts`'in var olan tablosuna girer); Task 6'da iki dosya ve 15 test (`panels/PanchangExtras.test.tsx` 7, `panels/NavataraPanel.test.tsx` 8), Task 7'de bir dosya ve 8 test (`panels/VimshopakaPanel.test.tsx`) kazanır.
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir; bu planda değişen okumalar açıkça yazılıdır:**
  - **C1** Gizlilik: yukarıdaki "Gizlilik" maddesi; `no-console.test.ts` durur ve izin listesi büyümez. Yeni `CommandError` kodu yoktur (D20).
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu ya da adres almaz. Bu planın üç komutu dizgi bağımsız değişken almaz; `api.test.ts`'in dizgi bağımsız değişken listesi değişmez.
  - **C3** Şema sürümleri değişmez (`SCHEMA_VERSION`, `CALC_SETTINGS_SCHEMA_VERSION`, `VIEW_STATE_SCHEMA_VERSION`, `APP_PREFS_SCHEMA_VERSION`, `EXPORT_SCHEMA_VERSION`); `crates/store` bu planda hiç değişmez (Y6).
  - **C4** Arayüz astrolojik hesap yapmaz: nokta, lordlar, tithi, dagdha burçları, tara, tur, küme, varga burcu, basamak ve küme değeri Rust'tan gelir. Satırları taraya göre dizmek, sayıyı üç ondalıkla yazmak, kimliği adına çevirmek, saat kesinliğine göre not seçmek ve UTC anını tarayıcının diliminde yazmak gösterimdir.
  - **C5/C17** Otomatik kayıtlar değişmez; Vimshopaka panelinin ayarı düzenle birlikte var olan yoldan kaydedilir. **C6** Düzen ayrıştırıcıları (`parseLayoutState`, Rust `check_panel_layout`) değişmez; iki panel türü kataloğa eklenir.
  - **C7/C16** Yeni kısayol yoktur. **C8** Yeni pencere yoktur.
  - **C9** Renk yalnızca token'lardan gelir (hex yok) ve tek başına anlam taşımaz: yaklaşık değer "≈" ve sözle, elverişli küme sözle, basamak sayıyla yazılır. Bu plan stil dosyası eklemez (D25).
  - **C10** Üretilen dosyalar elle düzenlenmez: `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` Task 5'te komutla yeniden üretilir; birleştirme çakışmasında da elle birleştirilmez, yeniden üretilir.
  - **C11** Görsel PNG temelleri yereldir; bu planda değişmeleri beklenmez (D25).
  - **C12** Her yeni i18n anahtarı üç dilde vardır (`i18n.test.ts` anahtar kümelerini ve yer tutucuları karşılaştırır). Bu planın Fransızca metinleri tipografik kesme işareti (’) kullanır; panel testleri düz kesme işaretinin yokluğunu sabitler.
  - **C13** Yasaklı ad taraması her commit'te; depo dışındaki ilerleme notları ve `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** Plan 07'nin gün doğumu toleransı ve `compute_panchang` değişmez (`panchang.rs`'e dokunulmaz). **C15** `alert`, `confirm`, `prompt`, `window.print()` ve `window.open()` çağrılmaz.
- **Erişilebilirlik:** WCAG AA. Tablolar `caption`, `scope="col"` ve `scope="row"` taşır; geniş tabloların kaydırma kabı klavye odağı alır ve adını tablonun başlığından okur (`BalaScroll`); kısaltmayla yazılan graha tam adıyla okunur (`aria-label`); yaklaşık değerin işareti ekran okuyucuya bir kez okunur (`Approx`); hata `role="alert"`'tir; seçicinin görünür etiketi vardır; her işlem klavyeyle yapılır.
- Commit mesajları İngilizcedir ve oturumun bildirdiği `Co-Authored-By:` satırıyla biter (bu planın komutlarında `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`; yürüten oturum başka bir satır bildiriyorsa o yazılır ve Task 8 Step 3'teki sayaç ona göre okunur). Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, `git tag`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Kod blokları `7270550` ağacına göre yazılmış ve bu plan yazılırken deponun bir kopyasında koşulmuştur (Self-Review madde 5). Yeni dosyalar tam içerikle verilir ("**Yeni dosya:**"). Değişen dosyalar `diff` bloğuyla verilir: blok bir dosyaya konup depo kökünde `git apply <dosya>` ile ya da elle uygulanır; bağlam tutmazsa önceki görevin adımı ve "Paylaşılan dosyalar" kuralı denetlenir, elle uydurma yapılmaz. Yeni bir Rust dosyasının testleri ve uygulaması ayrı adımlarda verilir: önce dosya yalnızca `#[cfg(test)] mod tests` bloğuyla oluşturulur, sonra uygulama **o bloğun üstüne**, arada bir boş satırla eklenir; `cargo fmt --all --check` birleştirmeyi doğrular.

## Paylaşılan dosyalar

Plan 24 ve Plan 25 aynı anda ayrı worktree'lerde yazılıp uygulanır. Bu planın dokunduğu, öteki iki planın da dokunması beklenen dosyaların tam listesi aşağıdadır. Bu dosyalardaki her değişiklik **yalnızca ektir**: `7270550`'ye göre hiçbir satır silinmez ya da yeniden sarılmaz (tek istisna `layout/model.test.ts`'teki tek satırlık liste; aşağıda). Task 8 Step 3 bunu sayaçla doğrular.

| Dosya | Bu planın eki | Görev |
|---|---|---|
| `crates/jyotish-core/src/lib.rs` | modül belgesine bir satır; `pub mod natal_extras;` (`muhurta` ile `nature` arasında); tek `pub use natal_extras::{…}` bloğu (`muhurta` ile `nature` blokları arasında) | 1–5 |
| `app/src-tauri/src/lib.rs` | `pub mod natal_extras_commands;`; komut listesinde `module_commands::compute_compat,` satırından sonra üç satır | 5 |
| `app/src-tauri/tests/ui_fixture.rs` | iki `use` (biri ayrı satırda `use jyotish_core::VargaScheme;`: ortak `use jyotish_core::{…}` listesi yeniden sarılmasın diye); üç hesap satırı; `json!` içinde `"shadbala"`dan sonra tek anahtar `"natal_extras"` | 5 |
| `app/ui/bindings.ts` | **üretilir** (C10): üç komut ve dokuz tip | 5 |
| `app/ui/test/fixtures/delhi.json` | **üretilir** (C10): `natal_extras` anahtarı | 5 |
| `app/ui/test/fixture.ts` | üç tip içe aktarımı; `UiFixture.natal_extras` alanı | 5 |
| `app/ui/api.ts`, `app/ui/api.test.ts` | `computeShadbala`'dan sonra üç sarmalayıcı; taklit listesinde ve çağrı tablosunda üçer satır | 5 |
| `app/ui/layout/model.ts` | `PANEL_KINDS` sonuna `"navatara"`, `"vimshopaka"`; `MODULE_PANELS` sonuna iki satır | 6, 7 |
| `app/ui/layout/model.test.ts` | iki listenin sonuna aynı iki tür (`MODULE_PANELS` beklentisi tek satırlık bir dizidir: o satır değişir) | 6, 7 |
| `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx` | iki içe aktarım; `PANEL_REGISTRY` sonuna iki kayıt; iki beklenti | 6, 7 |
| `app/ui/i18n/tr.ts`, `en.ts`, `fr.ts` | `panel` bloğunda `event_search`'ten sonra iki satır; üç yeni üst blok: `panchangExtras` (`ashtakavarga`'dan önce), `navatara` ve `vimshopaka` (`yogas`'tan önce). Dosyanın sonuna ek yapılmaz | 6, 7 |
| `app/ui/App.test.tsx` | `api` taklidine bir satır (`computePanchangExtras`): Panchang paneli artık bir komut çağırır | 6 |
| `app/e2e/ipc.ts` | `compute_shadbala` taklidinden sonra üç komut | 6, 7 |
| `app/e2e/desk.spec.ts` | dosyanın sonuna bir test | 7 |

Paylaşılmayan ama var olan iki dosya da değişir: `app/ui/panels/PanchangPanel.tsx` (bir içe aktarım, bir bileşen satırı, baş yorum) ve `app/ui/panels/PanchangPanel.test.tsx` (`api` taklidi). Bilerek **dokunulmayanlar:** `crates/jyotish-core/src/error.rs`, `panchang.rs`, `dasha.rs`, `dignity.rs`, `varga.rs`, `shadbala/`, `muhurta/`; `crates/jyotish-core/tests/modules_golden.rs`; `tests/golden/modules/README.md`; `app/src-tauri/src/commands.rs`, `module_commands.rs`; `app/src-tauri/tests/modules.rs`; `app/ui/panels/panels.css`, `labels.ts`, `grahaColumns.ts`; `app/ui/i18n/i18n.test.ts`; `crates/store`.

**Birleştirme kuralı.** (1) Çakışan her ek listesinde iki taraf da tutulur; sıra plan numarasıdır (23, 24, 25). Bu, `PANEL_KINDS`, `MODULE_PANELS`, `model.test.ts`'in iki listesi, `PANEL_REGISTRY`, `panel.*` metinleri, komut listesi, `json!` anahtarları ve `api` sarmalayıcıları için geçerlidir. (2) `bindings.ts` ve `delhi.json` elle birleştirilmez: çakışmada taraflardan biri alınır, sonra `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings` ve `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture` ile yeniden üretilir. (3) Birleştirmeden sonra `cargo test --workspace` ve `pnpm --dir app test` koşulur; sayaçlar iki planın farklarının toplamıdır. (4) Öteki plan önce birleştiyse bu planın `diff` bloklarındaki bağlam satırları bir-iki satır kayar: ek, aynı komşu satırın (yukarıdaki tabloda adı geçen) yanına elle konur.

## Sonraki işlere verilen API

Bu planın ürettiği ve sonraki işlerin tüketeceği adlar. Rust adları yıldızlıysa `jyotish_core` kökünden dışa aktarılır; imzalar görevlerde tam olarak verilir.

| API | Tüketen | Not |
|---|---|---|
| `nakshatra_lord`\*, `rasi_frame_offset`\* | Plan 26 (nakshatra lordu sütunları), Plan 31 | Nakshatra numarasından Vimshottari lordu; rasi çerçevesinin sidereal çerçeveye ötelenmesi |
| `yogi_point`\*, `yogi_facts`\*, `avayogi_nakshatra`\*, `YogiFacts`\*, `dagdha_rasis`\*, `panchang_extras`\*, `PanchangExtras`\*, `PANCHANG_EXTRAS_METHOD`\* | AI paneli; geçiş takvimi (Plan 29) | Saf; lagna istemez |
| `navatara`\*, `navatara_rows`\*, `transit_taras`\*, `tara_cycle`\*, `Navatara`\*, `NavataraRow`\*, `TransitTara`\*, `NAVATARA_METHOD`\* | Plan 31 (Sarvatobhadra), geçiş araçları (Plan 26) | Tara ve küme muhurta modülünün `TaraBala`'sıdır |
| `vimshopaka`\*, `vimshopaka_score`\*, `varga_relation`\*, `rung_points`\*, `VargaScheme`\*, `VimshopakaReport`\*, `GrahaVimshopaka`\*, `VimshopakaVarga`\*, `VIMSHOPAKA_GRAHAS`\*, `VIMSHOPAKA_MAX`\*, `VIMSHOPAKA_METHOD`\* | AI paneli; Varshaphala II (panchavargiya bala, Plan 28) | Bileşik ilişkinin vargada okunuşu yeniden kullanılabilir |
| `compute_panchang_extras`\*, `compute_navatara`\*, `compute_vimshopaka`\* | komut katmanı | Toplayıcılar; `ChartInput` |
| `natal_extras_commands::{compute_panchang_extras_blocking, compute_navatara_blocking, compute_vimshopaka_blocking}` | testler, AI paneli | Kalıp `module_commands` ile aynı |
| Arayüz: `modules/useNatalExtras.ts` (`usePanchangExtras`, `useNavatara`, `useVimshopaka`), `panels/PanchangExtras.tsx`, `panels/NavataraPanel.tsx`, `panels/VimshopakaPanel.tsx`; fikstürde `natal_extras` | sonraki paneller | i18n: `panchangExtras.*`, `navatara.*`, `vimshopaka.*`, `panel.navatara`, `panel.vimshopaka` |

## Dosya Yapısı (bu plan)

```
crates/jyotish-core/
  src/natal_extras/mod.rs         modül kaydı ve dışa aktarımlar (yeni)
  src/natal_extras/yogi.rs        yogi noktası, yogi, eş yogi, avayogi; nakshatra_lord; el örnekleri (yeni)
  src/natal_extras/dagdha.rs      dagdha rasi tablosu, ikinci yazım; PanchangExtras raporu (yeni)
  src/natal_extras/navatara.rs    doğum tablosu, geçiş tarası, tur (yeni)
  src/natal_extras/vimshopaka.rs  dört küme, ağırlıklar, basamaklar, rapor; el örnekleri (yeni)
  src/natal_extras/collect.rs     üç toplayıcı; sahte ephemeris testleri (yeni)
  src/lib.rs                      modül kaydı ve dışa aktarımlar (ek)
app/src-tauri/
  src/natal_extras_commands.rs    üç komut (yeni)
  src/lib.rs                      komut kaydı (ek)
  tests/natal_extras.rs           gerçek motorla çapraz denetim, hata eşlemesi, bütçeler (yeni)
  tests/ui_fixture.rs             fikstüre natal_extras (ek)
app/ui/
  bindings.ts                     üretilir
  test/fixtures/delhi.json        üretilir
  test/fixture.ts                 UiFixture'ın yeni alanı (ek)
  api.ts, api.test.ts             üç sarmalayıcı (ek)
  modules/useNatalExtras.ts       üç kanca (yeni)
  panels/PanchangExtras.tsx (+test)   Panchang panelinin ikinci tablosu (yeni)
  panels/PanchangPanel.tsx (+test)    ek tabloyu çizer; testte api taklidi
  panels/NavataraPanel.tsx (+test)    (yeni)
  panels/VimshopakaPanel.tsx (+test)  (yeni)
  panels/registry.ts, panels/panels.test.tsx   iki tür (ek)
  layout/model.ts (+test)         navatara ve vimshopaka türleri, MODULE_PANELS (ek)
  i18n/{tr,en,fr}.ts              panel, panchangExtras, navatara, vimshopaka (ek)
  App.test.tsx                    api taklidine bir satır
app/e2e/{ipc.ts,desk.spec.ts}     üç komutun taklidi; bir akış (ek)
tests/golden/modules/natal_extras/REVIEW.md   okuma kuralları, tablolar, onay ve elle denetim kaydı (yeni)
```

### Kapsam

| Yol haritası maddesi | Yer |
|---|---|
| §4.1 Yogi noktası: Güneş + Ay + sabit yay; yogi nakshatra lordu, eş yogi burç lordu | Task 1 (D2, D4) |
| §4.1 Avayogi: altıncı nakshatra; ikinci yazım aynı lordu verir | Task 1 (D3) |
| §4.1 Dagdha rasi tablosu: on dört tithi, chaturdashide dört burç, dolunay ve yeni ayda yok; paksha varyantı | Task 2 (D5) |
| §4.1 Lagna gerekmez; saat bilinmiyorsa "yaklaşık" | Task 2, Task 5 (D7), Task 6 |
| §4.1 Panchang paneli beş değeri gösterir | Task 6 (D6) |
| §4.2 27 nakshatra, dokuz tara, üç tur; nakshatra, lord, orada duran doğum grahaları | Task 3 (D9, D10), Task 6 |
| §4.2 Seçilen geçiş anında dokuz grahanın tarası | Task 3, Task 5 (D11), Task 6 |
| §4.2 Sayım yalnızca Ay'dan (lagna ikinci okuma) | Task 3 (D8) |
| §4.3 Dört küme; ağırlıkların toplamı 20 (değişmez testi) | Task 4 (D12) |
| §4.3 Basamaklar; yücelme ve moolatrikonanın yeri; geçici dostluğun okunduğu varga | Task 4 (D13, D14) |
| §4.3 Rahu ve Ketu'ya puan yok; saat "yaklaşık" ise uyarı | Task 4 (D15), Task 7 (D22) |
| §4.3 Shadbala paneline kardeş panel | Task 7 (D21) |
| §5 (5) El örnekleri ve komutlar (`compute_panchang_extras`, `compute_navatara`, `compute_vimshopaka`) | Task 1–4 (el örnekleri), Task 5 (D18, D19) |
| §5 (2) REVIEW kaydı; Y4 "doğrulanmamış tablo" notu | Task 2, 3, 4 (REVIEW), Task 6, 7 (not), Task 8 |
| §5 (7) i18n üç dil | Task 6, Task 7 |
| §5 (8) Kapılar ve elle denetim; "Sahibin denetimi" maddeleri | Task 8 |
| K12 Paletten ekleme; uçtan uca akış | Task 6, Task 7 |

---

### Task 1: Tabanlar ve `natal_extras` çekirdeği: yogi noktası, yogi, eş yogi ve avayogi

Yol haritası §4.1 (ilk dört olgu) ve §5 "Plan 23" görev taslağının (1). maddesi; K3, K5; D1–D4. Var olan hiçbir hesabın davranışı değişmez: `panchang.rs`, `dasha.rs`, `dignity.rs` ve `shadbala/` yalnızca okunur. Bu görev modülü kurar ve paylaşılan `lib.rs`'e modülün üç ekini koyar; sonraki dört görev yalnızca `pub use` listesini uzatır.

**Files:**
- Create: `crates/jyotish-core/src/natal_extras/mod.rs`, `crates/jyotish-core/src/natal_extras/yogi.rs`
- Modify: `crates/jyotish-core/src/lib.rs`
- Test: `yogi.rs` `mod tests` (6)

**Interfaces:**
- Consumes (v1, Plan 14, 17; hepsi var): `dasha::VIMSHOTTARI_ORDER`; `dignity::sign_lord`; `positions::Positions`; `shadbala::rasi_lon`; `zodiac::{deg_in_sign, nakshatra_pada, normalize, Sign, NAKSHATRA_SPAN_DEG}`; testte `panchang::nitya_yoga_no`.
- Produces (`jyotish_core` kökünden de dışa aktarılır):
  - `YOGI_ARC_DEG: f64` (93°20′ = 280/3), `AVAYOGI_COUNT: u8` (6).
  - `nakshatra_lord(nakshatra: u8) -> Graha` (1..=27; aralık dışı numara sarılır).
  - `yogi_point(sun_sidereal: f64, moon_sidereal: f64) -> f64` ([0, 360)).
  - `avayogi_nakshatra(yogi_nakshatra: u8) -> u8` (1..=27).
  - `rasi_frame_offset(positions: &Positions) -> f64` (`Sidereal` kipinde 0, tropikal-rasi kipinde ayanamsa).
  - `YogiFacts { point_lon: f64, sign: Sign, deg_in_sign: f64, nakshatra: u8, pada: u8, yogi: Graha, co_yogi: Graha, avayogi_nakshatra: u8, avayogi: Graha }`.
  - `yogi_facts(positions: &Positions) -> YogiFacts`.

- [ ] **Step 1: Tabanları ölç**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
git log -1 --format=%h
git status --short
rustc --version
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `git status --short` boş; `rustc 1.100.0` ya da yenisi; `jyotish-core` lib `404 passed`, golden `8 passed`, `modules_golden` `17 passed`; `atlas` `moment_fixed_lmt` `11 passed`, `moment_iana` `13 passed`; `store` lib `122 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `58 passed`, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `88 passed (88)`, `887 passed (887)`. Sayılar farklıysa (`main` ilerlediyse ya da Plan 24 ya da Plan 25 önce birleştiyse) ölçülen değerler ilerleme notlarına yazılır ve sayaç zinciri o tabandan, aynı farklarla yürütülür.

- [ ] **Step 2: Modül kaydı ve testler: yay, lordlar, üç el örneği, iki ikinci yazım ve çerçeve**

Önce modül kaydı (testlerin derlenebilmesi için), sonra test bloğu. Testler: yayın yedi nakshatra olduğu ve noktanın 360°'de sarıldığı; nakshatra lordlarının dokuzda bir yinelendiği; üç el örneği (A: 204°20′ Vishakha, B: 150°20′ Uttara Phalguni, C: 325° Purva Bhadrapada ve Revati'yi aşan sayım; türetmeler yorumlarda); yogi nakshatrasının nitya yogadan yedi ileride olduğu (ikinci yazım; 2 592 noktalık ızgara, dilim sınırlarına düşmez); avayoginin altıncı nakshatra olduğu, dokuz yogi–avayogi çifti ve eklenen yayla ikinci yazım; noktanın burcunun rasi çerçevesini izlediği.

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 61db775..1d059d0 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -9,6 +9,7 @@
 //! Muhurta (Alt proje 3.7): `timeline` (aralık dizileri) ve `muhurta` (gün panchangı, arama).
 //! Prashna (Alt proje 3.8): `prashna` (sayı prashnasının lagnası, soru haritasının olguları).
 //! Arama (Alt proje 3.9): `search` (harita araması sorgusu ve hafif yol, olay araması).
+//! Doğum anı ekleri (yol haritası Plan 23): `natal_extras` (panchang ekleri, navatara, Vimshopaka).
 
 pub mod ashtakavarga;
 mod chart;
@@ -25,6 +26,7 @@ pub mod jaimini;
 pub mod kuta;
 pub mod lordship;
 pub mod muhurta;
+pub mod natal_extras;
 pub mod nature;
 pub mod panchang;
 pub mod positions;
@@ -87,6 +89,10 @@ pub use muhurta::{
     PersonalFactors, TaraBala, TithiGroup, WindowSnapshot, MUHURTA_CHUNK_DAYS,
     MUHURTA_MAX_RANGE_DAYS, MUHURTA_METHOD,
 };
+pub use natal_extras::{
+    avayogi_nakshatra, nakshatra_lord, rasi_frame_offset, yogi_facts, yogi_point, YogiFacts,
+    AVAYOGI_COUNT, YOGI_ARC_DEG,
+};
 pub use nature::{kuja_dosha, moon_is_waxing, nature, KujaDosha, Nature, KUJA_DOSHA_HOUSES};
 pub use panchang::{
     compute_panchang, elongation, karana_name, karana_no, local_weekday, nitya_yoga_no, paksha,
````

**Yeni dosya:** `crates/jyotish-core/src/natal_extras/mod.rs`

````rust
//! Doğum anı ekleri (yol haritası §4.1–§4.3, Plan 23): panchang ekleri (yogi noktası, yogi, eş
//! yogi, avayogi, dagdha rasi), navatara tablosu ve Vimshopaka bala. Hepsi var olan konumlar
//! üzerinde saf aritmetiktir (K3): ephemeris'i yalnızca toplayıcılar görür. Sonuçlar `Chart`'a
//! eklenmez, panel açıkken istenir (K1). Hiçbiri lagna istemez; doğum saati bilinmiyorsa değerler
//! öğlen konumlarından gelir ve rapor `approximate` işaretini taşır.

pub mod yogi;

pub use yogi::{
    avayogi_nakshatra, nakshatra_lord, rasi_frame_offset, yogi_facts, yogi_point, YogiFacts,
    AVAYOGI_COUNT, YOGI_ARC_DEG,
};
````

**Yeni dosya:** `crates/jyotish-core/src/natal_extras/yogi.rs` (şimdilik yalnızca test bloğu)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::panchang::nitya_yoga_no;
    use crate::zodiac::NAKSHATRA_SPAN_DEG;
    use Graha::*;

    /// Yalnızca Güneş ve Ay'ı verilen sentetik konumlar; öteki grahalar Koç 5°'dedir.
    fn sky(sun: f64, moon: f64) -> Positions {
        let mut lons = [5.0; 9];
        lons[Sun as usize] = sun;
        lons[Moon as usize] = moon;
        Positions::synthetic(None, lons)
    }

    /// Tropikal-rasi kipinin konumları: sidereal boylamlar yerinde durur, burç ve burç içi
    /// derece `ayanamsa` kadar ötelenmiş boylamdan okunur.
    fn tropical(mut p: Positions, ayanamsa: f64) -> Positions {
        for g in &mut p.grahas {
            let frame = normalize(g.sidereal_lon + ayanamsa);
            g.rasi_sign = Sign::of(frame);
            g.deg_in_sign = deg_in_sign(frame);
        }
        p
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn the_arc_is_seven_nakshatras_and_the_point_wraps() {
        assert!(close(YOGI_ARC_DEG, 7.0 * NAKSHATRA_SPAN_DEG));
        assert!(close(YOGI_ARC_DEG, 93.0 + 20.0 / 60.0));
        // 10° + 101° + 93°20′ = 204°20′.
        assert!(close(yogi_point(10.0, 101.0), 204.0 + 20.0 / 60.0));
        // Toplam 360°'yi aşar: 346° + 71° = 417° → 57°; + 93°20′ = 150°20′.
        assert!(close(yogi_point(346.0, 71.0), 150.0 + 20.0 / 60.0));
        // Toplamın kendisi 360°'yi aşmaz ama yay ekleyince aşar: 300° + 10° + 93°20′ = 43°20′.
        assert!(close(yogi_point(300.0, 10.0), 43.0 + 20.0 / 60.0));
        // Girdi sarılmamış olabilir.
        assert!(close(yogi_point(370.0, -259.0), yogi_point(10.0, 101.0)));
    }

    #[test]
    fn nakshatra_lords_repeat_every_nine() {
        // Ashwini'den: Ketu, Venüs, Güneş, Ay, Mars, Rahu, Jüpiter, Satürn, Merkür; üç tur.
        let hand = [Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury];
        for n in 1..=27u8 {
            assert_eq!(nakshatra_lord(n), hand[usize::from(n - 1) % 9], "{n}");
        }
        // Aralık dışı numara sarılır.
        assert_eq!(nakshatra_lord(28), Ketu);
        assert_eq!(nakshatra_lord(0), Mercury);
    }

    #[test]
    fn hand_examples_give_the_point_the_yogi_the_co_yogi_and_the_avayogi() {
        // A: Güneş 10°, Ay 101° → nokta 204°20′. Vishakha (16: 200°–213°20′), 2. pada
        // (203°20′–206°40′); lordu Jüpiter. Burç Terazi (180°–210°), 24°20′; lordu Venüs.
        // Altıncı nakshatra: 16, 17, 18, 19, 20, 21 → Uttara Ashadha (21); lordu Güneş.
        let a = yogi_facts(&sky(10.0, 101.0));
        assert!(close(a.point_lon, 204.0 + 20.0 / 60.0));
        assert_eq!((a.nakshatra, a.pada, a.yogi), (16, 2, Jupiter));
        assert_eq!((a.sign, a.co_yogi), (Sign::Libra, Venus));
        assert!(close(a.deg_in_sign, 24.0 + 20.0 / 60.0));
        assert_eq!((a.avayogi_nakshatra, a.avayogi), (21, Sun));

        // B: Güneş 346°, Ay 71° → nokta 150°20′. Uttara Phalguni (12: 146°40′–160°), 2. pada
        // (150°–153°20′); lordu Güneş. Burç Başak (150°–180°), 0°20′; lordu Merkür. Altıncı
        // nakshatra: 12 … 17 → Anuradha (17); lordu Satürn.
        let b = yogi_facts(&sky(346.0, 71.0));
        assert_eq!((b.nakshatra, b.pada, b.yogi), (12, 2, Sun));
        assert_eq!((b.sign, b.co_yogi), (Sign::Virgo, Mercury));
        assert!(close(b.deg_in_sign, 20.0 / 60.0));
        assert_eq!((b.avayogi_nakshatra, b.avayogi), (17, Saturn));

        // C: Güneş 200°, Ay 31°40′ → nokta 325°. Purva Bhadrapada (25: 320°–333°20′), 2. pada
        // (323°20′–326°40′); lordu Jüpiter. Burç Kova (300°–330°), 25°; lordu Satürn. Altıncı
        // nakshatra Revati'yi aşar: 25, 26, 27, 1, 2, 3 → Krittika (3); lordu Güneş.
        let c = yogi_facts(&sky(200.0, 31.0 + 40.0 / 60.0));
        assert!(close(c.point_lon, 325.0));
        assert_eq!((c.nakshatra, c.pada, c.yogi), (25, 2, Jupiter));
        assert_eq!((c.sign, c.co_yogi), (Sign::Aquarius, Saturn));
        assert_eq!((c.avayogi_nakshatra, c.avayogi), (3, Sun));
        // Lagna sonucu etkilemez.
        let mut with_lagna = sky(200.0, 31.0 + 40.0 / 60.0);
        with_lagna.lagna_lon = Some(15.0);
        with_lagna.lagna_sign = Some(Sign::Aries);
        assert_eq!(yogi_facts(&with_lagna), c);
    }

    #[test]
    fn the_yogi_nakshatra_is_the_nitya_yoga_shifted_by_seven() {
        // İkinci yazım: nitya yoga, toplamın 13°20′'lik dilimidir (1..=27); nokta toplamdan tam
        // yedi dilim ileridedir. Yoga k ise yogi nakshatrası (k + 7), 27'yi aşınca sarılır.
        // Izgara dilim sınırlarına düşmez (sınırda iki yazım son basamakta ayrışabilir).
        let mut seen = [false; 27];
        for i in 0..72 {
            for j in 0..36 {
                let sun = f64::from(i) * 5.0 + 0.11;
                let moon = f64::from(j) * 10.0 + 0.07;
                let yoga = nitya_yoga_no(sun, moon);
                let facts = yogi_facts(&sky(sun, moon));
                assert_eq!(facts.nakshatra, (yoga + 6) % 27 + 1, "{sun} {moon}");
                seen[usize::from(facts.nakshatra) - 1] = true;
            }
        }
        assert!(seen.iter().all(|s| *s), "the grid visits every nakshatra");
    }

    #[test]
    fn avayogi_is_the_sixth_nakshatra_and_agrees_with_the_added_arc() {
        // Adım adım sayım: yogi nakshatrası 1, sonra beş adım.
        for n in 1..=27u8 {
            let mut at = n;
            for _ in 1..AVAYOGI_COUNT {
                at = if at == 27 { 1 } else { at + 1 };
            }
            assert_eq!(avayogi_nakshatra(n), at, "{n}");
        }
        // Yogi → avayogi çiftleri, elle (lord sırasında beş adım ileri).
        let pairs = [
            (Ketu, Rahu),
            (Venus, Jupiter),
            (Sun, Saturn),
            (Moon, Mercury),
            (Mars, Ketu),
            (Rahu, Venus),
            (Jupiter, Sun),
            (Saturn, Moon),
            (Mercury, Mars),
        ];
        for n in 1..=27u8 {
            let yogi = nakshatra_lord(n);
            let expected = pairs.iter().find(|(y, _)| *y == yogi).expect("pair").1;
            assert_eq!(nakshatra_lord(avayogi_nakshatra(n)), expected, "{n}");
        }
        // İkinci yazım: noktaya 186°40′ (on dört nakshatra) eklenir ve varılan nakshatranın
        // lorduna bakılır. Nakshatra aynı değildir (6. yerine 15.), lord aynıdır: 14 = 9 + 5.
        for i in 0..72 {
            for j in 0..36 {
                let facts = yogi_facts(&sky(f64::from(i) * 5.0 + 0.11, f64::from(j) * 10.0 + 0.07));
                let (other, _) = nakshatra_pada(facts.point_lon + 14.0 * NAKSHATRA_SPAN_DEG);
                assert_eq!(nakshatra_lord(other), facts.avayogi, "{i} {j}");
                assert_eq!((other + 27 - facts.nakshatra) % 27, 14);
            }
        }
    }

    #[test]
    fn the_point_sign_follows_the_rasi_frame() {
        // Güneş 10°, Ay 68°40′ → nokta sidereal 172°: Hasta (13: 160°–173°20′), lordu Ay.
        // Sidereal çerçevede burç Başak (lordu Merkür).
        let sidereal = sky(10.0, 68.0 + 40.0 / 60.0);
        assert_eq!(rasi_frame_offset(&sidereal), 0.0);
        let s = yogi_facts(&sidereal);
        assert_eq!((s.nakshatra, s.yogi), (13, Moon));
        assert_eq!((s.sign, s.co_yogi), (Sign::Virgo, Mercury));
        // Tropikal-rasi kipi, ayanamsa 24°: nokta rasi çerçevesinde 196°, Terazi 16° (lordu
        // Venüs). Nokta, nakshatra, yogi ve avayogi değişmez.
        let frame = tropical(sidereal, 24.0);
        assert!(close(rasi_frame_offset(&frame), 24.0));
        let t = yogi_facts(&frame);
        assert_eq!((t.sign, t.co_yogi), (Sign::Libra, Venus));
        assert!(close(t.deg_in_sign, 16.0));
        assert_eq!(
            (t.point_lon, t.nakshatra, t.pada, t.yogi, t.avayogi),
            (s.point_lon, s.nakshatra, s.pada, s.yogi, s.avayogi)
        );
        // Güneş 350°'deyken öteleme 360°'yi aşar; fark yine 24°'dir.
        let wrapped = tropical(sky(350.0, 20.0), 24.0);
        assert!(close(rasi_frame_offset(&wrapped), 24.0));
        // `Sidereal` kipinde son basamak gürültüsü öteleme sayılmaz.
        for k in 0..360 {
            let p = sky(f64::from(k) + 1.0 / 3.0, 0.0);
            assert_eq!(rasi_frame_offset(&p), 0.0, "{k}");
        }
    }
}
````

- [ ] **Step 3: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-core --lib natal_extras 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası (`error[E0432]: unresolved imports`: `mod.rs`'in ve `lib.rs`'in dışa aktardığı `yogi_facts`, `yogi_point`, `nakshatra_lord`, `YogiFacts`, `YOGI_ARC_DEG` … henüz yok; ardından `error: could not compile`). Test koşmaz.

- [ ] **Step 4: Yogi noktası ve üç lord**

**`crates/jyotish-core/src/natal_extras/yogi.rs`: uygulama, test bloğunun üstüne (arada bir boş satır)**

````rust
//! Yogi noktası, yogi, eş yogi ve avayogi (yol haritası §4.1).
//!
//! Tanım (nitya yoga geleneği): yogi noktası, Güneş ve Ay'ın sidereal boylamları toplamına
//! 93°20′ (yedi nakshatra; Pushya'nın başı) eklenerek bulunur. Yogi o noktanın nakshatra
//! lordudur (Vimshottari sırası); eş yogi noktanın burcunun lordudur; avayogi, yogi
//! nakshatrasından altıncı nakshatranın lordudur (yogi nakshatrası 1 sayılır).
//!
//! Nokta ve nakshatraları her zodyak kipinde sidereal boylamdandır; noktanın burcu rasi
//! çerçevesindedir (K5): tropikal-rasi kipinde nokta, grahalarla aynı yay kadar ötelenir.

use serde::{Deserialize, Serialize};

use crate::dasha::VIMSHOTTARI_ORDER;
use crate::dignity::sign_lord;
use crate::graha::Graha;
use crate::positions::Positions;
use crate::shadbala::rasi_lon;
use crate::zodiac::{deg_in_sign, nakshatra_pada, normalize, Sign};

/// Yogi noktasının sabit yayı: 93°20′ = 7 × 13°20′.
pub const YOGI_ARC_DEG: f64 = 280.0 / 3.0;
/// Avayogi nakshatrası yogi nakshatrasından kaçıncıdır (yogi nakshatrası 1 sayılır).
pub const AVAYOGI_COUNT: u8 = 6;
/// Rasi çerçevesi ile sidereal çerçeve arasındaki yay bundan küçükse sıfır sayılır (`Sidereal`
/// kipinde iki boylam aynı sayının iki yazımıdır ve son basamakta ayrışabilir).
const FRAME_EPS_DEG: f64 = 1e-9;

/// Nakshatranın (1..=27) Vimshottari lordu: Ashwini Ketu, Bharani Venüs … sıra dokuzda bir
/// yinelenir. Aralık dışı numara 27'ye göre sarılır (panik yok).
pub fn nakshatra_lord(nakshatra: u8) -> Graha {
    VIMSHOTTARI_ORDER[(usize::from(nakshatra) + 26) % 27 % 9]
}

/// Yogi noktasının sidereal boylamı, [0, 360).
pub fn yogi_point(sun_sidereal: f64, moon_sidereal: f64) -> f64 {
    normalize(normalize(sun_sidereal + moon_sidereal) + YOGI_ARC_DEG)
}

/// Yogi nakshatrasından (1..=27) altıncı nakshatra, 1..=27.
pub fn avayogi_nakshatra(yogi_nakshatra: u8) -> u8 {
    ((usize::from(yogi_nakshatra) + 26 + usize::from(AVAYOGI_COUNT) - 1) % 27 + 1) as u8
}

/// Rasi çerçevesinin sidereal çerçeveye göre ötelenmesi, derece: `Sidereal` kipinde 0,
/// tropikal-rasi kipinde ayanamsa. Güneş'in iki boylamının farkından okunur (`Positions`
/// ayanamsayı taşımaz).
pub fn rasi_frame_offset(positions: &Positions) -> f64 {
    let sun = positions.point(Graha::Sun);
    let offset = normalize(rasi_lon(sun) - sun.sidereal_lon);
    if (FRAME_EPS_DEG..=360.0 - FRAME_EPS_DEG).contains(&offset) {
        offset
    } else {
        0.0
    }
}

/// Yogi noktası ve ondan okunan üç graha.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct YogiFacts {
    /// Yogi noktasının sidereal boylamı, [0, 360).
    pub point_lon: f64,
    /// Noktanın rasi çerçevesindeki burcu (K5).
    pub sign: Sign,
    /// Rasi çerçevesinde burç içi derece, [0, 30).
    pub deg_in_sign: f64,
    /// Noktanın nakshatrası, 1..=27 (sidereal).
    pub nakshatra: u8,
    /// 1..=4.
    pub pada: u8,
    /// Noktanın nakshatra lordu.
    pub yogi: Graha,
    /// Eş yogi: noktanın burcunun lordu.
    pub co_yogi: Graha,
    /// Yogi nakshatrasından altıncı nakshatra, 1..=27.
    pub avayogi_nakshatra: u8,
    /// O nakshatranın lordu.
    pub avayogi: Graha,
}

/// Konumlardan yogi olguları. Lagna gerekmez.
pub fn yogi_facts(positions: &Positions) -> YogiFacts {
    let sun = positions.point(Graha::Sun).sidereal_lon;
    let moon = positions.point(Graha::Moon).sidereal_lon;
    let point_lon = yogi_point(sun, moon);
    let frame_lon = normalize(point_lon + rasi_frame_offset(positions));
    let sign = Sign::of(frame_lon);
    let (nakshatra, pada) = nakshatra_pada(point_lon);
    let avayogi_nakshatra = avayogi_nakshatra(nakshatra);
    YogiFacts {
        point_lon,
        sign,
        deg_in_sign: deg_in_sign(frame_lon),
        nakshatra,
        pada,
        yogi: nakshatra_lord(nakshatra),
        co_yogi: sign_lord(sign),
        avayogi_nakshatra,
        avayogi: nakshatra_lord(avayogi_nakshatra),
    }
}
````

- [ ] **Step 5: Testler ve kapılar**

```bash
cargo fmt --all
cargo test -p jyotish-core --lib natal_extras::yogi 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: `6 passed`; lib `410 passed`, golden `8 passed`, `modules_golden` `17 passed`; fmt ve clippy çıktısız; sayaç `0`. Bir el örneği tutmazsa önce yorumdaki türetme ve kural yeniden okunur; beklenti motor çıktısına bakarak değiştirilmez.

- [ ] **Step 6: Commit**

```bash
git add crates/jyotish-core/src/natal_extras crates/jyotish-core/src/lib.rs
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): natal_extras module with the yogi point, yogi, co-yogi and avayogi from the Sun and Moon longitudes" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: üç dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 2: Dagdha rasi tablosu, ikinci yazım, panchang ekleri raporu ve REVIEW kaydı

Yol haritası §4.1 (dagdha rasi; "zor yer ve kaynak riski") ve görev taslağının (2). maddesi; Y4, Y5; D5, D7, D18. Tablo bellekten yazılır; bu görev onu üç yerde tutar ve birbirine bağlar: motorun tablosu (`DAGDHA_RASI`, tithiden burca), testteki ikinci yazım (`BY_SIGN`, burçtan tithiye) ve `REVIEW.md`'nin iki tablosu. **Yürüten ajan tabloyu başka bir yazılımla ya da çevrimiçi kaynakla karşılaştırmaz;** şüphelendiği hücreyi ilerleme notlarına yazar.

**Files:**
- Create: `crates/jyotish-core/src/natal_extras/dagdha.rs`, `tests/golden/modules/natal_extras/REVIEW.md`
- Modify: `crates/jyotish-core/src/natal_extras/mod.rs`, `crates/jyotish-core/src/lib.rs`
- Test: `dagdha.rs` `mod tests` (4)

**Interfaces:**
- Consumes: Task 1 (`yogi_facts`, `YogiFacts`); `panchang::{elongation, tithi_no}`; `positions::Positions`; `zodiac::Sign`.
- Produces (`jyotish_core` kökünden de dışa aktarılır):
  - `PANCHANG_EXTRAS_METHOD: &str` (`"panchang_extras.v1"`).
  - `dagdha_rasis(tithi_no: u8) -> &'static [Sign]` (1..=30; 15 ve 30 için ve aralık dışında boş).
  - `PanchangExtras { yogi: YogiFacts, tithi: u8, dagdha_rasis: Vec<Sign>, approximate: bool, method: String }`.
  - `panchang_extras(positions: &Positions, approximate: bool) -> PanchangExtras`.

- [ ] **Step 1: Kayıt ve testler: tablonun biçimi, on dört satırı, ikinci yazımı ve rapor**

Testler: on üç tithide iki, chaturdashide dört, on beşinci tithide sıfır burç; hücreler zodyak sırasında ve iki paksha ortak; on dört satır elle; ikinci yazımla (burçtan) hücre hücre aynılık ve 30 hücre; aralık dışı tithi; raporun el örnekleri (uzanım 91° → 8. tithi; 191°40′ → 16. tithi; 175° ve 355° → dagdha rasi yok; 340° → dört burç).

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 1d059d0..a73e671 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -90,8 +90,9 @@ pub use muhurta::{
     MUHURTA_MAX_RANGE_DAYS, MUHURTA_METHOD,
 };
 pub use natal_extras::{
-    avayogi_nakshatra, nakshatra_lord, rasi_frame_offset, yogi_facts, yogi_point, YogiFacts,
-    AVAYOGI_COUNT, YOGI_ARC_DEG,
+    avayogi_nakshatra, dagdha_rasis, nakshatra_lord, panchang_extras, rasi_frame_offset,
+    yogi_facts, yogi_point, PanchangExtras, YogiFacts, AVAYOGI_COUNT, PANCHANG_EXTRAS_METHOD,
+    YOGI_ARC_DEG,
 };
 pub use nature::{kuja_dosha, moon_is_waxing, nature, KujaDosha, Nature, KUJA_DOSHA_HOUSES};
 pub use panchang::{
diff --git a/crates/jyotish-core/src/natal_extras/mod.rs b/crates/jyotish-core/src/natal_extras/mod.rs
index 678f4e2..0a43677 100644
--- a/crates/jyotish-core/src/natal_extras/mod.rs
+++ b/crates/jyotish-core/src/natal_extras/mod.rs
@@ -4,8 +4,10 @@
 //! eklenmez, panel açıkken istenir (K1). Hiçbiri lagna istemez; doğum saati bilinmiyorsa değerler
 //! öğlen konumlarından gelir ve rapor `approximate` işaretini taşır.
 
+pub mod dagdha;
 pub mod yogi;
 
+pub use dagdha::{dagdha_rasis, panchang_extras, PanchangExtras, PANCHANG_EXTRAS_METHOD};
 pub use yogi::{
     avayogi_nakshatra, nakshatra_lord, rasi_frame_offset, yogi_facts, yogi_point, YogiFacts,
     AVAYOGI_COUNT, YOGI_ARC_DEG,
````

**Yeni dosya:** `crates/jyotish-core/src/natal_extras/dagdha.rs` (şimdilik yalnızca test bloğu)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use Sign::*;

    fn sky(sun: f64, moon: f64) -> Positions {
        let mut lons = [5.0; 9];
        lons[Graha::Sun as usize] = sun;
        lons[Graha::Moon as usize] = moon;
        Positions::synthetic(None, lons)
    }

    #[test]
    fn two_signs_on_thirteen_tithis_four_on_the_fourteenth_none_on_the_fifteenth() {
        for tithi in 1..=30u8 {
            let signs = dagdha_rasis(tithi);
            let expected = match (tithi - 1) % 15 + 1 {
                14 => 4,
                15 => 0,
                _ => 2,
            };
            assert_eq!(signs.len(), expected, "tithi {tithi}");
            // Hücre zodyak sırasındadır ve bir burcu iki kez saymaz.
            assert!(signs.windows(2).all(|w| w[0] < w[1]), "tithi {tithi}");
        }
        // İki paksha aynı tabloyu paylaşır.
        for tithi in 1..=15u8 {
            assert_eq!(
                dagdha_rasis(tithi),
                dagdha_rasis(tithi + 15),
                "tithi {tithi}"
            );
        }
        // Tablonun satırları, elle (paksha içi sıra).
        assert_eq!(dagdha_rasis(1), [Libra, Capricorn]);
        assert_eq!(dagdha_rasis(2), [Sagittarius, Pisces]);
        assert_eq!(dagdha_rasis(3), [Leo, Capricorn]);
        assert_eq!(dagdha_rasis(4), [Taurus, Aquarius]);
        assert_eq!(dagdha_rasis(5), [Gemini, Virgo]);
        assert_eq!(dagdha_rasis(6), [Aries, Leo]);
        assert_eq!(dagdha_rasis(7), [Cancer, Sagittarius]);
        assert_eq!(dagdha_rasis(8), [Gemini, Virgo]);
        assert_eq!(dagdha_rasis(9), [Leo, Scorpio]);
        assert_eq!(dagdha_rasis(10), [Leo, Scorpio]);
        assert_eq!(dagdha_rasis(11), [Sagittarius, Pisces]);
        assert_eq!(dagdha_rasis(12), [Libra, Capricorn]);
        assert_eq!(dagdha_rasis(13), [Taurus, Leo]);
        assert_eq!(dagdha_rasis(14), [Gemini, Virgo, Sagittarius, Pisces]);
        assert!(dagdha_rasis(15).is_empty());
        assert!(dagdha_rasis(30).is_empty());
    }

    /// İkinci yazım: tablo burçtan okunur. Her burç için, o burcun dagdha olduğu paksha içi
    /// tithiler (REVIEW.md'deki ikinci tablo).
    const BY_SIGN: [(Sign, &[u8]); 12] = [
        (Aries, &[6]),
        (Taurus, &[4, 13]),
        (Gemini, &[5, 8, 14]),
        (Cancer, &[7]),
        (Leo, &[3, 6, 9, 10, 13]),
        (Virgo, &[5, 8, 14]),
        (Libra, &[1, 12]),
        (Scorpio, &[9, 10]),
        (Sagittarius, &[2, 7, 11, 14]),
        (Capricorn, &[1, 3, 12]),
        (Aquarius, &[4]),
        (Pisces, &[2, 11, 14]),
    ];

    #[test]
    fn the_table_agrees_with_its_second_writing_by_sign() {
        let mut cells = 0;
        for (sign, tithis) in BY_SIGN {
            for tithi in 1..=15u8 {
                assert_eq!(
                    dagdha_rasis(tithi).contains(&sign),
                    tithis.contains(&tithi),
                    "{sign:?} on tithi {tithi}"
                );
            }
            cells += tithis.len();
        }
        // 13 × 2 + 4 = 30 hücre.
        assert_eq!(cells, 30);
        assert_eq!((1..=15u8).map(|t| dagdha_rasis(t).len()).sum::<usize>(), 30);
    }

    #[test]
    fn out_of_range_tithis_have_no_dagdha_sign() {
        assert!(dagdha_rasis(0).is_empty());
        assert!(dagdha_rasis(31).is_empty());
        assert!(dagdha_rasis(u8::MAX).is_empty());
    }

    #[test]
    fn extras_join_the_yogi_facts_the_tithi_and_the_dagdha_signs() {
        // Güneş 10°, Ay 101°: uzanım 91° → 91 / 12 = 7,58 → 8. tithi (Shukla Ashtami):
        // İkizler ve Başak. Yogi olguları `yogi.rs`'in A örneğidir (nokta 204°20′).
        let a = panchang_extras(&sky(10.0, 101.0), false);
        assert_eq!(a.tithi, 8);
        assert_eq!(a.dagdha_rasis, [Gemini, Virgo]);
        assert_eq!(a.yogi, yogi_facts(&sky(10.0, 101.0)));
        assert_eq!((a.yogi.nakshatra, a.yogi.yogi), (16, Graha::Jupiter));
        assert!(!a.approximate);
        assert_eq!(a.method, "panchang_extras.v1");
        // Güneş 200°, Ay 31°40′: uzanım 191°40′ → 15,97 → 16. tithi (Krishna Pratipada):
        // Shukla Pratipada ile aynı satır, Terazi ve Oğlak.
        let c = panchang_extras(&sky(200.0, 31.0 + 40.0 / 60.0), true);
        assert_eq!(c.tithi, 16);
        assert_eq!(c.dagdha_rasis, [Libra, Capricorn]);
        assert!(c.approximate);
        // Uzanım 175° → 15. tithi (Purnima) ve 355° → 30. tithi (Amavasya): dagdha rasi yok.
        assert_eq!(panchang_extras(&sky(10.0, 185.0), false).tithi, 15);
        assert!(panchang_extras(&sky(10.0, 185.0), false)
            .dagdha_rasis
            .is_empty());
        assert_eq!(panchang_extras(&sky(10.0, 5.0), false).tithi, 30);
        assert!(panchang_extras(&sky(10.0, 5.0), false)
            .dagdha_rasis
            .is_empty());
        // Uzanım 340° → 29. tithi (Krishna Chaturdashi): dört burç.
        let k14 = panchang_extras(&sky(10.0, 350.0), false);
        assert_eq!(k14.tithi, 29);
        assert_eq!(k14.dagdha_rasis, [Gemini, Virgo, Sagittarius, Pisces]);
    }
}
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-core --lib natal_extras::dagdha 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası (`error[E0432]: unresolved imports`: `dagdha_rasis`, `panchang_extras`, `PanchangExtras`, `PANCHANG_EXTRAS_METHOD` henüz yok). Test koşmaz.

- [ ] **Step 3: Tablo ve rapor**

**`crates/jyotish-core/src/natal_extras/dagdha.rs`: uygulama, test bloğunun üstüne (arada bir boş satır)**

````rust
//! Dagdha rasi ve panchang eklerinin raporu (yol haritası §4.1).
//!
//! Dagdha rasi, doğum tithisine bağlı burçlardır: on üç tithide iki burç, chaturdashide dört
//! burç; dolunayda ve yeni ayda yoktur. Tablo bellekten yazılmıştır ve basılı kaynakla hücre
//! hücre denetlenmeyi bekler (Y4): `tests/golden/modules/natal_extras/REVIEW.md`. Shukla ve
//! Krishna paksha aynı tabloyu paylaşır (tek okuma; `PANCHANG_EXTRAS_METHOD`).

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::panchang::{elongation, tithi_no};
use crate::positions::Positions;
use crate::zodiac::Sign;

use super::yogi::{yogi_facts, YogiFacts};

/// Yöntem kimliği (Y5): 93°20′ yay, altıncı nakshatradan avayogi, iki pakshada ortak dagdha
/// rasi tablosu.
pub const PANCHANG_EXTRAS_METHOD: &str = "panchang_extras.v1";

/// Paksha içi tithi sırasına (1..=15) göre dagdha burçlar; her hücre zodyak sırasındadır.
const DAGDHA_RASI: [&[Sign]; 15] = [
    &[Sign::Libra, Sign::Capricorn],
    &[Sign::Sagittarius, Sign::Pisces],
    &[Sign::Leo, Sign::Capricorn],
    &[Sign::Taurus, Sign::Aquarius],
    &[Sign::Gemini, Sign::Virgo],
    &[Sign::Aries, Sign::Leo],
    &[Sign::Cancer, Sign::Sagittarius],
    &[Sign::Gemini, Sign::Virgo],
    &[Sign::Leo, Sign::Scorpio],
    &[Sign::Leo, Sign::Scorpio],
    &[Sign::Sagittarius, Sign::Pisces],
    &[Sign::Libra, Sign::Capricorn],
    &[Sign::Taurus, Sign::Leo],
    &[Sign::Gemini, Sign::Virgo, Sign::Sagittarius, Sign::Pisces],
    &[],
];

/// Tithinin (1..=30) dagdha burçları; 15 (Purnima) ve 30 (Amavasya) için boştur. Aralık dışı
/// numara boş liste verir (panik yok).
pub fn dagdha_rasis(tithi_no: u8) -> &'static [Sign] {
    match tithi_no {
        1..=30 => DAGDHA_RASI[usize::from(tithi_no - 1) % 15],
        _ => &[],
    }
}

/// `compute_panchang_extras` sonucu: doğum anının beş ek panchang olgusu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PanchangExtras {
    pub yogi: YogiFacts,
    /// Doğum tithisi, 1..=30; dagdha rasi buna bağlıdır.
    pub tithi: u8,
    /// Zodyak sırasında; dolunay ve yeni ayda boş.
    pub dagdha_rasis: Vec<Sign>,
    /// Doğum saati bilinmiyor: değerler öğlen konumlarındandır (Ay günde yaklaşık 13° yürür).
    pub approximate: bool,
    pub method: String,
}

/// Konumlardan panchang ekleri. Tithi, Güneş ve Ay'ın sidereal boylamlarının farkındandır
/// (uzanım ayanamsadan bağımsızdır). Lagna gerekmez.
pub fn panchang_extras(positions: &Positions, approximate: bool) -> PanchangExtras {
    let sun = positions.point(Graha::Sun).sidereal_lon;
    let moon = positions.point(Graha::Moon).sidereal_lon;
    let tithi = tithi_no(elongation(sun, moon));
    PanchangExtras {
        yogi: yogi_facts(positions),
        tithi,
        dagdha_rasis: dagdha_rasis(tithi).to_vec(),
        approximate,
        method: PANCHANG_EXTRAS_METHOD.to_owned(),
    }
}
````

- [ ] **Step 4: REVIEW kaydı: okuma kuralları, iki tablo ve onay bekleyen noktalar**

Dosya bu görevde kurulur; Task 3 ve Task 4 kendi bölümlerini, Task 8 elle denetim kaydını ekler. §1.3'teki tablo `DAGDHA_RASI` ile, §1.4'teki tablo testteki `BY_SIGN` ile satır satır aynı olmalıdır; yazdıktan sonra ikisi kodla gözle karşılaştırılır.

**Yeni dosya:** `tests/golden/modules/natal_extras/REVIEW.md`

````markdown
# Doğum anı ekleri: okuma kuralları, tablolar, onay bekleyen noktalar ve gözden geçirme kaydı

Kaynak: yol haritası (`docs/superpowers/specs/2026-11-27-eksikler-yol-haritasi.md`) §4.1–§4.3 ve
Plan 23'ün kararları. Modül: `crates/jyotish-core/src/natal_extras/`.

Bu dizinde JSON örnek yoktur. El örnekleri modülün birim testleridir (`yogi.rs`, `dagdha.rs`,
`navatara.rs`, `vimshopaka.rs` ve `collect.rs` içindeki `mod tests`); her beklenen değerin
türetmesi testin yorumunda adım adım yazılıdır. Değerler kuraldan elle türetilmiştir; motor
çıktısı ve başka bir yazılımın çıktısı buraya ve testlere kopyalanmaz (K9). Bir değer motorla
uyuşmazsa önce kural yeniden okunur; beklenen değer motor çıktısına bakarak düzeltilmez.

**Bu dosyadaki tablolar bellekten yazılmıştır ve sahibinin basılı kaynakla onayını bekler (Y4).**
Onaya dek Panchang panelinin ek tablosu ve Vimshopaka paneli "doğrulanmamış tablo" notunu taşır.
Yürüten ajan tabloları başka bir yazılımla ya da çevrimiçi kaynakla karşılaştırmaz.

## 1. Panchang ekleri (yöntem `panchang_extras.v1`)

### 1.1 Okuma kuralları

1. **Yogi noktası.** Güneş ve Ay'ın sidereal boylamları toplanır, 93°20′ eklenir ve sonuç 360°'ye
   göre sarılır. 93°20′ yedi nakshatradır (Pushya'nın başı): nokta, nitya yoganın dilimini yedi
   nakshatra ileri taşır. Nitya yoga k ise (1–27) yogi nakshatrası k + 7'dir (27'yi aşınca 27
   çıkarılır).
2. **Yogi.** Noktanın nakshatrasının Vimshottari lordu: Ashwini Ketu, Bharani Venüs, Krittika
   Güneş, Rohini Ay, Mrigashira Mars, Ardra Rahu, Punarvasu Jüpiter, Pushya Satürn, Ashlesha
   Merkür; sıra dokuzda bir yinelenir.
3. **Eş yogi.** Noktanın bulunduğu burcun lordu. Burç rasi çerçevesindedir (K5): tropikal-rasi
   kipinde nokta ayanamsa kadar ötelenmiş boylamdan okunur. Noktanın nakshatrası her kipte
   sidereal boylamdandır.
4. **Avayogi.** Yogi nakshatrasından altıncı nakshatranın lordu (yogi nakshatrası 1 sayılır).
   İkinci yazım: noktaya 186°40′ (on dört nakshatra) eklenir ve varılan nakshatranın lorduna
   bakılır. Varılan nakshatra farklıdır (on beşinci), lord aynıdır (14 = 9 + 5).
5. **Dagdha rasi.** Doğum tithisinin paksha içi sırasına (1–15) göre 1.3'teki tablodan. Shukla ve
   Krishna paksha aynı satırı okur. Dolunayda (15) ve yeni ayda (30) dagdha rasi yoktur.
6. **Doğum saati bilinmiyorsa** değerler öğlen konumlarından hesaplanır ve "yaklaşık" işaretini
   taşır (Ay günde yaklaşık 13° yürür: nokta ve tithi, dolayısıyla beş değerin hepsi değişebilir).
   Lagna gerekmez.

### 1.2 Yogi ve avayogi çiftleri (kuraldan türeyen denetim tablosu)

| Yogi | Ketu | Venüs | Güneş | Ay | Mars | Rahu | Jüpiter | Satürn | Merkür |
|---|---|---|---|---|---|---|---|---|---|
| Avayogi | Rahu | Jüpiter | Satürn | Merkür | Ketu | Venüs | Güneş | Ay | Mars |

### 1.3 Dagdha rasi tablosu (bellekten; onay bekler)

Motorun tablosu: `crates/jyotish-core/src/natal_extras/dagdha.rs`, `DAGDHA_RASI`.

| Paksha içi tithi | Ad | Dagdha burçlar |
|---|---|---|
| 1 | Pratipada | Terazi, Oğlak |
| 2 | Dwitiya | Yay, Balık |
| 3 | Tritiya | Aslan, Oğlak |
| 4 | Chaturthi | Boğa, Kova |
| 5 | Panchami | İkizler, Başak |
| 6 | Shashthi | Koç, Aslan |
| 7 | Saptami | Yengeç, Yay |
| 8 | Ashtami | İkizler, Başak |
| 9 | Navami | Aslan, Akrep |
| 10 | Dashami | Aslan, Akrep |
| 11 | Ekadashi | Yay, Balık |
| 12 | Dwadashi | Terazi, Oğlak |
| 13 | Trayodashi | Boğa, Aslan |
| 14 | Chaturdashi | İkizler, Başak, Yay, Balık |
| 15 | Purnima / Amavasya | yok |

### 1.4 İkinci yazım: burçtan

Aynı tablo burçtan okunur; testteki `BY_SIGN` listesi budur. Toplam 30 hücre (13 × 2 + 4).

| Burç | Dagdha olduğu paksha içi tithiler |
|---|---|
| Koç | 6 |
| Boğa | 4, 13 |
| İkizler | 5, 8, 14 |
| Yengeç | 7 |
| Aslan | 3, 6, 9, 10, 13 |
| Başak | 5, 8, 14 |
| Terazi | 1, 12 |
| Akrep | 9, 10 |
| Yay | 2, 7, 11, 14 |
| Oğlak | 1, 3, 12 |
| Kova | 4 |
| Balık | 2, 11, 14 |

### 1.5 Onay bekleyen noktalar (sahibi)

- [ ] Dagdha rasi tablosunun on dört satırı, hücre hücre.
- [ ] Shukla ve Krishna pakshanın aynı tabloyu paylaşması.
- [ ] Chaturdashinin dört burcu; dolunayda ve yeni ayda dagdha rasi olmaması.
- [ ] Yogi noktasının sabit yayı (93°20′).
- [ ] Avayoginin sayımı (yogi nakshatrasından altıncı nakshatra).
- [ ] Eş yoginin burcunun tropikal-rasi kipinde rasi çerçevesinden okunması (K5).
````

- [ ] **Step 5: Testler ve kapılar**

```bash
cargo fmt --all
cargo test -p jyotish-core --lib natal_extras::dagdha 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
grep -c '^| [0-9]' tests/golden/modules/natal_extras/REVIEW.md
```
Expected: `4 passed`; lib `414 passed`, golden `8 passed`, `modules_golden` `17 passed`; fmt ve clippy çıktısız; sayaç `0`; son komut `15` yazar (dagdha tablosunun on beş satırı).

- [ ] **Step 6: Commit**

```bash
git add crates/jyotish-core/src/natal_extras crates/jyotish-core/src/lib.rs tests/golden/modules/natal_extras/REVIEW.md
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): dagdha rasi table with its second writing by sign; panchang extras report; REVIEW record for the natal extras" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: dört dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 3: Navatara çekirdeği: doğum tablosu ve geçiş tarası

Yol haritası §4.2 ve görev taslağının (3). maddesi; D8–D11. Tara ve elverişli küme muhurta modülünün var olan `tara_bala` fonksiyonundan gelir; bu görev yeni tablo getirmez. Yeni olan, sayımın turu (`tara_cycle`), 27 satırlık doğum tablosu ve geçiş grahalarının satırlarıdır.

**Files:**
- Create: `crates/jyotish-core/src/natal_extras/navatara.rs`
- Modify: `crates/jyotish-core/src/natal_extras/mod.rs`, `crates/jyotish-core/src/lib.rs`, `tests/golden/modules/natal_extras/REVIEW.md`
- Test: `navatara.rs` `mod tests` (5)

**Interfaces:**
- Consumes: Task 1 (`nakshatra_lord`); `muhurta::{tara_bala, TaraBala}` (testte `TARA_FAVOURABLE`); `positions::Positions`; `zodiac::nakshatra_pada`.
- Produces (`jyotish_core` kökünden de dışa aktarılır):
  - `NAVATARA_METHOD: &str` (`"navatara.moon_v1"`).
  - `NavataraRow { nakshatra: u8, tara: TaraBala, cycle: u8, lord: Graha, grahas: Vec<Graha> }`.
  - `TransitTara { graha: Graha, nakshatra: u8, pada: u8, tara: TaraBala, cycle: u8 }`.
  - `Navatara { janma_nakshatra: u8, rows: Vec<NavataraRow>, transit: Option<Vec<TransitTara>>, transit_jd_ut: Option<f64>, approximate: bool, method: String }`.
  - `tara_cycle(from: u8, to: u8) -> u8` (1..=3).
  - `navatara_rows(positions: &Positions) -> Vec<NavataraRow>` (27 satır, doğum nakshatrasından).
  - `transit_taras(janma_nakshatra: u8, lons: &[(Graha, f64)]) -> Vec<TransitTara>`.
  - `navatara(positions: &Positions, transit: Option<(f64, &[(Graha, f64)])>, approximate: bool) -> Navatara`.

- [ ] **Step 1: Kayıt ve testler: 27 satır, üç tur, grahaların satırları, adım adım sayım ve geçiş taraları**

Testler tek bir sentetik harita üzerindedir (dokuz grahanın nakshatrası test başındaki yorumda türetilmiştir): tablonun doğum nakshatrasından başlayıp üç tur yürüdüğü ve bir taranın üç nakshatrasının aynı lordu taşıdığı; her grahanın kendi nakshatrasının satırında durduğu (sekiz satırın sayımı elle); tara ve turun 729 çiftte adım adım sayımla aynı olduğu (ikinci yazım: formül kullanmaz); dört geçiş grahasının tarası; raporun geçiş bölümünü yalnızca an verilince taşıdığı.

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index a73e671..9100024 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -90,8 +90,9 @@ pub use muhurta::{
     MUHURTA_MAX_RANGE_DAYS, MUHURTA_METHOD,
 };
 pub use natal_extras::{
-    avayogi_nakshatra, dagdha_rasis, nakshatra_lord, panchang_extras, rasi_frame_offset,
-    yogi_facts, yogi_point, PanchangExtras, YogiFacts, AVAYOGI_COUNT, PANCHANG_EXTRAS_METHOD,
+    avayogi_nakshatra, dagdha_rasis, nakshatra_lord, navatara, navatara_rows, panchang_extras,
+    rasi_frame_offset, tara_cycle, transit_taras, yogi_facts, yogi_point, Navatara, NavataraRow,
+    PanchangExtras, TransitTara, YogiFacts, AVAYOGI_COUNT, NAVATARA_METHOD, PANCHANG_EXTRAS_METHOD,
     YOGI_ARC_DEG,
 };
 pub use nature::{kuja_dosha, moon_is_waxing, nature, KujaDosha, Nature, KUJA_DOSHA_HOUSES};
diff --git a/crates/jyotish-core/src/natal_extras/mod.rs b/crates/jyotish-core/src/natal_extras/mod.rs
index 0a43677..c8b8738 100644
--- a/crates/jyotish-core/src/natal_extras/mod.rs
+++ b/crates/jyotish-core/src/natal_extras/mod.rs
@@ -5,9 +5,14 @@
 //! öğlen konumlarından gelir ve rapor `approximate` işaretini taşır.
 
 pub mod dagdha;
+pub mod navatara;
 pub mod yogi;
 
 pub use dagdha::{dagdha_rasis, panchang_extras, PanchangExtras, PANCHANG_EXTRAS_METHOD};
+pub use navatara::{
+    navatara, navatara_rows, tara_cycle, transit_taras, Navatara, NavataraRow, TransitTara,
+    NAVATARA_METHOD,
+};
 pub use yogi::{
     avayogi_nakshatra, nakshatra_lord, rasi_frame_offset, yogi_facts, yogi_point, YogiFacts,
     AVAYOGI_COUNT, YOGI_ARC_DEG,
````

**Yeni dosya:** `crates/jyotish-core/src/natal_extras/navatara.rs` (şimdilik yalnızca test bloğu)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::muhurta::TARA_FAVOURABLE;
    use Graha::*;

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    /// Nakshatralar: Güneş 10° Ashwini (1); Ay 100° Pushya (8: 93°20′–106°40′); Mars 220°
    /// Anuradha (17: 213°20′–226°40′); Merkür 65° Mrigashira (5: 53°20′–66°40′); Jüpiter 250°
    /// Mula (19: 240°–253°20′); Venüs 40° Rohini (4: 40°–53°20′); Satürn 300° Dhanishta (23:
    /// 293°20′–306°40′); Rahu 95° Pushya (8); Ketu 275° Uttara Ashadha (21: 266°40′–280°).
    const BASE: [f64; 9] = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 95.0, 275.0];

    fn base() -> Positions {
        Positions::synthetic(Some(0.0), BASE)
    }

    /// Adım adım sayım: `from`'dan `to`'ya nakshatra nakshatra yürür; taranın numarasını ve
    /// turunu formülsüz bulur.
    fn hand_count(from: u8, to: u8) -> (u8, u8) {
        let (mut at, mut tara, mut cycle) = (from, 1, 1);
        while at != to {
            at = if at == 27 { 1 } else { at + 1 };
            if tara == 9 {
                tara = 1;
                cycle += 1;
            } else {
                tara += 1;
            }
        }
        (tara, cycle)
    }

    #[test]
    fn the_table_starts_at_the_birth_nakshatra_and_runs_three_rounds_of_nine() {
        let rows = navatara_rows(&base());
        assert_eq!(rows.len(), 27);
        // Doğum nakshatrası Pushya (8): satırlar 8, 9, … 27, 1, … 7.
        let order: Vec<u8> = rows.iter().map(|r| r.nakshatra).collect();
        let hand: Vec<u8> = (8..=27).chain(1..=7).collect();
        assert_eq!(order, hand);
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(usize::from(row.tara.number), i % 9 + 1, "row {i}");
            assert_eq!(usize::from(row.cycle), i / 9 + 1, "row {i}");
            assert_eq!(
                row.tara.favourable,
                TARA_FAVOURABLE.contains(&row.tara.number),
                "row {i}"
            );
            assert_eq!(row.lord, nakshatra_lord(row.nakshatra), "row {i}");
        }
        // Bir taranın üç nakshatrası aynı lordu taşır: Janma satırları Pushya (8), Anuradha (17)
        // ve Uttara Bhadrapada (26); üçünün lordu Satürn.
        let janma: Vec<(u8, Graha)> = rows
            .iter()
            .filter(|r| r.tara.number == 1)
            .map(|r| (r.nakshatra, r.lord))
            .collect();
        assert_eq!(janma, [(8, Saturn), (17, Saturn), (26, Saturn)]);
    }

    #[test]
    fn natal_grahas_sit_in_the_rows_of_their_nakshatras() {
        let rows = navatara_rows(&base());
        let at = |n: u8| rows.iter().find(|r| r.nakshatra == n).expect("row");
        // Pushya (8): Ay ve Rahu; 1. tara (Janma), 1. tur.
        assert_eq!(at(8).grahas, [Moon, Rahu]);
        assert_eq!((at(8).tara.number, at(8).cycle), (1, 1));
        // Anuradha (17): Mars. Pushya'dan sayım 8 → 17: on nakshatra → 1. tara, 2. tur.
        assert_eq!(at(17).grahas, [Mars]);
        assert_eq!((at(17).tara.number, at(17).cycle), (1, 2));
        // Mula (19): Jüpiter. Sayım 12 → 3. tara (Vipat), 2. tur.
        assert_eq!(at(19).grahas, [Jupiter]);
        assert_eq!((at(19).tara.number, at(19).cycle), (3, 2));
        // Uttara Ashadha (21): Ketu. Sayım 14 → 5. tara (Pratyak), 2. tur.
        assert_eq!(
            (at(21).grahas.as_slice(), at(21).tara.number),
            (&[Ketu][..], 5)
        );
        // Dhanishta (23): Satürn. Sayım 16 → 7. tara (Naidhana), 2. tur.
        assert_eq!(
            (at(23).grahas.as_slice(), at(23).tara.number),
            (&[Saturn][..], 7)
        );
        // Ashwini (1): Güneş. Sayım 8 → 27 yirmi, sonra 1: yirmi bir → 3. tara, 3. tur.
        assert_eq!(at(1).grahas, [Sun]);
        assert_eq!((at(1).tara.number, at(1).cycle), (3, 3));
        // Rohini (4): Venüs. Sayım 24 → 6. tara (Sadhana), 3. tur; elverişli kümede.
        assert_eq!(at(4).grahas, [Venus]);
        assert_eq!(
            (at(4).tara.number, at(4).cycle, at(4).tara.favourable),
            (6, 3, true)
        );
        // Mrigashira (5): Merkür. Sayım 25 → 7. tara, 3. tur.
        assert_eq!(
            (at(5).grahas.as_slice(), at(5).tara.number),
            (&[Mercury][..], 7)
        );
        // Her graha tam bir satırdadır; öteki satırlar boştur.
        assert_eq!(rows.iter().map(|r| r.grahas.len()).sum::<usize>(), 9);
        assert_eq!(rows.iter().filter(|r| r.grahas.is_empty()).count(), 19);
    }

    #[test]
    fn tara_and_cycle_match_the_step_by_step_count_on_every_pair() {
        for from in 1..=27u8 {
            for to in 1..=27u8 {
                let (tara, cycle) = hand_count(from, to);
                assert_eq!(tara_bala(from, to).number, tara, "{from} → {to}");
                assert_eq!(tara_cycle(from, to), cycle, "{from} → {to}");
            }
        }
    }

    #[test]
    fn transit_taras_count_each_graha_from_the_birth_nakshatra() {
        // Doğum nakshatrası Pushya (8). Geçiş: Güneş 100° (Pushya, 3. pada: 100°–103°20′) → 1.
        // tara, 1. tur; Ay 0° (Ashwini, 1. pada): sayım 21 → 3. tara, 3. tur; Jüpiter 359°
        // (Revati 27, 4. pada): sayım 20 → 2. tara (Sampat), 3. tur; Satürn 110°30′ (Ashlesha 9,
        // 2. pada: 110°–113°20′): sayım 2 → 2. tara, 1. tur.
        let lons = [(Sun, 100.0), (Moon, 0.0), (Jupiter, 359.0), (Saturn, 110.5)];
        let got: Vec<(Graha, u8, u8, u8, u8, bool)> = transit_taras(8, &lons)
            .into_iter()
            .map(|t| {
                (
                    t.graha,
                    t.nakshatra,
                    t.pada,
                    t.tara.number,
                    t.cycle,
                    t.tara.favourable,
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                (Sun, 8, 3, 1, 1, false),
                (Moon, 1, 1, 3, 3, false),
                (Jupiter, 27, 4, 2, 3, true),
                (Saturn, 9, 2, 2, 1, true),
            ]
        );
    }

    #[test]
    fn the_report_carries_the_transit_only_when_a_moment_is_given() {
        let natal = navatara(&base(), None, false);
        assert_eq!(natal.janma_nakshatra, 8);
        assert_eq!(natal.rows, navatara_rows(&base()));
        assert_eq!((natal.transit.as_ref(), natal.transit_jd_ut), (None, None));
        assert!(!natal.approximate);
        assert_eq!(natal.method, "navatara.moon_v1");
        let lons: Vec<(Graha, f64)> = Graha::ALL.into_iter().zip(BASE).collect();
        let with = navatara(&base(), Some((2_460_000.5, &lons)), true);
        assert_eq!(with.transit_jd_ut, Some(2_460_000.5));
        assert!(with.approximate);
        // Geçiş konumları doğum konumlarının aynısıysa her graha kendi satırının tarasını alır.
        let transit = with.transit.expect("transit");
        assert_eq!(transit.len(), 9);
        for t in &transit {
            let row = with
                .rows
                .iter()
                .find(|r| r.nakshatra == t.nakshatra)
                .expect("row");
            assert!(row.grahas.contains(&t.graha), "{:?}", t.graha);
            assert_eq!((t.tara, t.cycle), (row.tara, row.cycle), "{:?}", t.graha);
        }
        // Lagna tabloyu etkilemez.
        assert_eq!(navatara_rows(&base().without_lagna()), natal.rows);
    }
}
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-core --lib natal_extras::navatara 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası (`error[E0432]: unresolved imports`: `navatara`, `navatara_rows`, `tara_cycle`, `transit_taras` ve tipleri henüz yok). Test koşmaz.

- [ ] **Step 3: Doğum tablosu, tur ve geçiş taraları**

**`crates/jyotish-core/src/natal_extras/navatara.rs`: uygulama, test bloğunun üstüne (arada bir boş satır)**

````rust
//! Navatara: doğum Ay'ının nakshatrasından başlayarak 27 nakshatranın dokuz taraya üç turda
//! dağılımı ve seçilen geçiş anında dokuz grahanın tarası (yol haritası §4.2).
//!
//! Sayım doğum nakshatrasını da sayar (aynı nakshatra 1, Janma) ve dokuzda bir yinelenir; tur
//! sayımın hangi dokuzluda olduğudur (1..=3). Taranın numarası ve elverişli kümesi muhurta
//! modülününkidir (`tara_bala`); bu dosya yeni tablo getirmez. Sayım yalnızca Ay'dandır
//! (`NAVATARA_METHOD`); lagna nakshatrasından sayım uygulanmamıştır.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::muhurta::{tara_bala, TaraBala};
use crate::positions::Positions;
use crate::zodiac::nakshatra_pada;

use super::yogi::nakshatra_lord;

/// Yöntem kimliği (Y5): sayım doğum Ay'ının nakshatrasından.
pub const NAVATARA_METHOD: &str = "navatara.moon_v1";

/// Tablonun bir satırı: bir nakshatra ve doğum nakshatrasına göre tarası.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct NavataraRow {
    /// 1..=27.
    pub nakshatra: u8,
    pub tara: TaraBala,
    /// Sayımın turu, 1..=3.
    pub cycle: u8,
    /// Nakshatranın Vimshottari lordu.
    pub lord: Graha,
    /// Bu nakshatrada duran doğum grahaları, `Graha::ALL` sırasında.
    pub grahas: Vec<Graha>,
}

/// Geçiş anındaki bir grahanın tarası.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TransitTara {
    pub graha: Graha,
    /// Grahanın geçiş anındaki nakshatrası, 1..=27.
    pub nakshatra: u8,
    /// 1..=4.
    pub pada: u8,
    pub tara: TaraBala,
    /// Sayımın turu, 1..=3.
    pub cycle: u8,
}

/// `compute_navatara` sonucu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Navatara {
    /// Doğum Ay'ının nakshatrası, 1..=27.
    pub janma_nakshatra: u8,
    /// 27 satır; ilk satır doğum nakshatrasıdır, sonrakiler zodyak sırasıyla.
    pub rows: Vec<NavataraRow>,
    /// Geçiş anı verildiyse dokuz graha, `Graha::ALL` sırasında.
    pub transit: Option<Vec<TransitTara>>,
    /// Geçiş anı, UT Jülyen günü.
    pub transit_jd_ut: Option<f64>,
    /// Doğum saati bilinmiyor: Ay'ın nakshatrası öğlen konumundandır.
    pub approximate: bool,
    pub method: String,
}

/// `from` nakshatrasından `to` nakshatrasına sayımın turu, 1..=3: sayım 1–9 ise 1, 10–18 ise
/// 2, 19–27 ise 3. İkisi de 1..=27 olmalıdır (aralık dışı değer 27'ye göre sarılır).
pub fn tara_cycle(from: u8, to: u8) -> u8 {
    ((usize::from(to) + 27 - usize::from(from) % 27) % 27 / 9 + 1) as u8
}

/// Doğum tablosu: 27 satır, doğum Ay'ının nakshatrasından başlayarak.
pub fn navatara_rows(positions: &Positions) -> Vec<NavataraRow> {
    let (janma, _) = nakshatra_pada(positions.point(Graha::Moon).sidereal_lon);
    (0..27u8)
        .map(|step| {
            let nakshatra = (janma - 1 + step) % 27 + 1;
            NavataraRow {
                nakshatra,
                tara: tara_bala(janma, nakshatra),
                cycle: step / 9 + 1,
                lord: nakshatra_lord(nakshatra),
                grahas: positions
                    .grahas
                    .iter()
                    .filter(|p| nakshatra_pada(p.sidereal_lon).0 == nakshatra)
                    .map(|p| p.graha)
                    .collect(),
            }
        })
        .collect()
}

/// Geçiş grahalarının tarası. `lons` (graha, sidereal boylam) çiftleridir; sıra korunur.
/// `janma_nakshatra` 1..=27 olmalıdır.
pub fn transit_taras(janma_nakshatra: u8, lons: &[(Graha, f64)]) -> Vec<TransitTara> {
    lons.iter()
        .map(|&(graha, lon)| {
            let (nakshatra, pada) = nakshatra_pada(lon);
            TransitTara {
                graha,
                nakshatra,
                pada,
                tara: tara_bala(janma_nakshatra, nakshatra),
                cycle: tara_cycle(janma_nakshatra, nakshatra),
            }
        })
        .collect()
}

/// Rapor: doğum tablosu ve (verildiyse) geçiş anının taraları. `transit`, anın UT Jülyen günü
/// ve dokuz grahanın sidereal boylamıdır.
pub fn navatara(
    positions: &Positions,
    transit: Option<(f64, &[(Graha, f64)])>,
    approximate: bool,
) -> Navatara {
    let rows = navatara_rows(positions);
    let janma_nakshatra = rows[0].nakshatra;
    Navatara {
        janma_nakshatra,
        rows,
        transit: transit.map(|(_, lons)| transit_taras(janma_nakshatra, lons)),
        transit_jd_ut: transit.map(|(jd, _)| jd),
        approximate,
        method: NAVATARA_METHOD.to_owned(),
    }
}
````

- [ ] **Step 4: REVIEW kaydına navatara bölümü**

**Dosyanın sonuna, önüne bir boş satır konarak eklenir:** `tests/golden/modules/natal_extras/REVIEW.md`

````markdown
## 2. Navatara (yöntem `navatara.moon_v1`)

### 2.1 Okuma kuralları

1. **Sayım** doğum Ay'ının nakshatrasından başlar ve onu da sayar (aynı nakshatra 1).
2. **Tara** sayımın dokuzlu döngüdeki yeridir (10 → 1, 19 → 1); **tur** sayımın hangi dokuzlukta
   olduğudur: 1–9 birinci, 10–18 ikinci, 19–27 üçüncü tur.
3. **Adlar ve elverişli küme** muhurta modülününkidir (`../muhurta/REVIEW.md`, kural 7): 1 Janma,
   2 Sampat, 3 Vipat, 4 Kshema, 5 Pratyak, 6 Sadhana, 7 Naidhana, 8 Mitra, 9 Parama Mitra;
   elverişli küme {2, 4, 6, 8, 9}. Bu modül yeni tablo getirmez; o tablonun onayı burada da
   geçerlidir.
4. **Satır** nakshatranın Vimshottari lordunu ve o nakshatrada duran doğum grahalarını taşır.
   Bir taranın üç nakshatrası aynı lordu taşır (dokuz nakshatra arayla).
5. **Geçiş tarası.** Seçilen andaki dokuz grahanın sidereal nakshatrası, doğum nakshatrasından
   aynı sayımla okunur. Geçiş anı haritanın kendi ayarlarıyla (ayanamsa, düğüm türü) hesaplanır.
6. **Sayım yalnızca Ay'dandır.** Lagna nakshatrasından sayım uygulanmamıştır; istenirse ikinci
   okuma olarak eklenir ve o zaman ayar olur (Y5).
7. **Doğum saati bilinmiyorsa** Ay'ın nakshatrası öğlen konumundandır; Ay o gün nakshatra
   değiştiriyorsa bütün tablo bir nakshatra kayar. Panel bunu söyler.

### 2.2 Onay bekleyen noktalar (sahibi)

- [ ] Navataranın lagna nakshatrasından da sayılıp sayılmayacağı (öneri: yalnızca Ay).
- [ ] Janma tarasının elverişli kümede sayılmaması (muhurta tablosunun aynı noktası).
````

- [ ] **Step 5: Testler ve kapılar**

```bash
cargo fmt --all
cargo test -p jyotish-core --lib natal_extras::navatara 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
grep -c '^## ' tests/golden/modules/natal_extras/REVIEW.md
```
Expected: `5 passed`; lib `419 passed`, golden `8 passed`, `modules_golden` `17 passed`; fmt ve clippy çıktısız; sayaç `0`; son komut `2` yazar (iki bölüm).

- [ ] **Step 6: Commit**

```bash
git add crates/jyotish-core/src/natal_extras crates/jyotish-core/src/lib.rs tests/golden/modules/natal_extras/REVIEW.md
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): navatara table from the birth nakshatra in three rounds of nine and the taras of transit grahas" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: dört dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 4: Vimshopaka çekirdeği: ağırlık tabloları, basamaklar ve değişmezler

Yol haritası §4.3 ve görev taslağının (4). maddesi; Y4, Y5; D12–D15. Bileşik ilişki var olan `dignity` fonksiyonlarıyla, Shadbala'nın saptavargaja kalemindeki okumayla kurulur; `shadbala/sthana.rs` **değişmez** (yalnızca `rasi_lon` ve `SHADBALA_GRAHAS` okunur). İki tablo ailesi bellekten yazılır: dört kümenin ağırlıkları ve altı basamak. **Yürüten ajan tabloları başka bir yazılımla ya da çevrimiçi kaynakla karşılaştırmaz.**

**Files:**
- Create: `crates/jyotish-core/src/natal_extras/vimshopaka.rs`
- Modify: `crates/jyotish-core/src/natal_extras/mod.rs`, `crates/jyotish-core/src/lib.rs`, `tests/golden/modules/natal_extras/REVIEW.md`
- Test: `vimshopaka.rs` `mod tests` (6)

**Interfaces:**
- Consumes: `dignity::{compound_relation, natural_relation, own_signs, sign_lord, temporal_friend, Dignity}`; `positions::Positions`; `shadbala::{rasi_lon, SHADBALA_GRAHAS}`; `varga::{varga_position, Varga}`; `zodiac::Sign`.
- Produces (`jyotish_core` kökünden de dışa aktarılır):
  - `VIMSHOPAKA_METHOD: &str` (`"vimshopaka.bphs_v1"`), `VIMSHOPAKA_MAX: f64` (20), `VIMSHOPAKA_GRAHAS: [Graha; 7]`.
  - `VargaScheme { Shadvarga, Saptavarga, Dashavarga, Shodashavarga }`, `VargaScheme::ALL`, `VargaScheme::weights(self) -> &'static [(Varga, f64)]`.
  - `rung_points(relation: Dignity) -> f64`.
  - `varga_relation(graha: Graha, varga: Varga, positions: &Positions) -> Option<(Sign, Dignity)>` (düğümler `None`).
  - `vimshopaka_score(graha: Graha, scheme: VargaScheme, positions: &Positions) -> f64`.
  - `VimshopakaVarga { varga: Varga, sign: Sign, relation: Dignity, points: f64 }`, `GrahaVimshopaka { graha: Graha, vargas: Vec<VimshopakaVarga>, shadvarga: f64, saptavarga: f64, dashavarga: f64, shodashavarga: f64 }`, `VimshopakaReport { grahas: Vec<GrahaVimshopaka>, approximate: bool, method: String }`.
  - `vimshopaka(positions: &Positions, approximate: bool) -> VimshopakaReport`.

- [ ] **Step 1: Kayıt ve testler: ağırlıkların toplamı, altı basamak, iki tam el örneği, aralık ve okuma kuralları**

Testler: dört kümenin vargaları elle ve ağırlıkların toplamı 20 (değişmez), üç varganın hücreleri elle; altı basamağın D1'deki örnekleri ve yücelme ile moolatrikonanın ayrı basamak olmadığı; iki tam el örneği (Güneş Aslan 0° ve Ay Boğa 0°: on altı varga burcu, on altı basamak ve dört değer; türetmeler yorumlarda, her örnekte varga kuralı sözle yazılıdır); 240 konumluk ızgarada raporun biçimi, değerin kendi satırlarından yeniden toplanışı ve 5–20 aralığı; geçici dostluğun D1'den, vargaların rasi çerçevesinden okunduğu.

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 9100024..b55094b 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -91,9 +91,11 @@ pub use muhurta::{
 };
 pub use natal_extras::{
     avayogi_nakshatra, dagdha_rasis, nakshatra_lord, navatara, navatara_rows, panchang_extras,
-    rasi_frame_offset, tara_cycle, transit_taras, yogi_facts, yogi_point, Navatara, NavataraRow,
-    PanchangExtras, TransitTara, YogiFacts, AVAYOGI_COUNT, NAVATARA_METHOD, PANCHANG_EXTRAS_METHOD,
-    YOGI_ARC_DEG,
+    rasi_frame_offset, rung_points, tara_cycle, transit_taras, varga_relation, vimshopaka,
+    vimshopaka_score, yogi_facts, yogi_point, GrahaVimshopaka, Navatara, NavataraRow,
+    PanchangExtras, TransitTara, VargaScheme, VimshopakaReport, VimshopakaVarga, YogiFacts,
+    AVAYOGI_COUNT, NAVATARA_METHOD, PANCHANG_EXTRAS_METHOD, VIMSHOPAKA_GRAHAS, VIMSHOPAKA_MAX,
+    VIMSHOPAKA_METHOD, YOGI_ARC_DEG,
 };
 pub use nature::{kuja_dosha, moon_is_waxing, nature, KujaDosha, Nature, KUJA_DOSHA_HOUSES};
 pub use panchang::{
diff --git a/crates/jyotish-core/src/natal_extras/mod.rs b/crates/jyotish-core/src/natal_extras/mod.rs
index c8b8738..4cbfcab 100644
--- a/crates/jyotish-core/src/natal_extras/mod.rs
+++ b/crates/jyotish-core/src/natal_extras/mod.rs
@@ -6,6 +6,7 @@
 
 pub mod dagdha;
 pub mod navatara;
+pub mod vimshopaka;
 pub mod yogi;
 
 pub use dagdha::{dagdha_rasis, panchang_extras, PanchangExtras, PANCHANG_EXTRAS_METHOD};
@@ -13,6 +14,10 @@ pub use navatara::{
     navatara, navatara_rows, tara_cycle, transit_taras, Navatara, NavataraRow, TransitTara,
     NAVATARA_METHOD,
 };
+pub use vimshopaka::{
+    rung_points, varga_relation, vimshopaka, vimshopaka_score, GrahaVimshopaka, VargaScheme,
+    VimshopakaReport, VimshopakaVarga, VIMSHOPAKA_GRAHAS, VIMSHOPAKA_MAX, VIMSHOPAKA_METHOD,
+};
 pub use yogi::{
     avayogi_nakshatra, nakshatra_lord, rasi_frame_offset, yogi_facts, yogi_point, YogiFacts,
     AVAYOGI_COUNT, YOGI_ARC_DEG,
````

**Yeni dosya:** `crates/jyotish-core/src/natal_extras/vimshopaka.rs` (şimdilik yalnızca test bloğu)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::zodiac::{deg_in_sign, normalize};
    use Graha::*;
    use Varga::*;

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    /// `sthana.rs` testlerinin tabanıdır.
    const BASE: [f64; 9] = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];

    fn with(changes: &[(Graha, f64)]) -> Positions {
        let mut lons = BASE;
        for &(g, lon) in changes {
            lons[g as usize] = lon;
        }
        Positions::synthetic(None, lons)
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn points_of(row: &GrahaVimshopaka) -> Vec<f64> {
        row.vargas.iter().map(|v| v.points).collect()
    }

    #[test]
    fn each_scheme_weighs_its_vargas_to_twenty() {
        // Kümelerin vargaları, elle (BPHS sırası).
        let hand: [(VargaScheme, &[Varga]); 4] = [
            (VargaScheme::Shadvarga, &[D1, D2, D3, D9, D12, D30]),
            (VargaScheme::Saptavarga, &[D1, D2, D3, D7, D9, D12, D30]),
            (
                VargaScheme::Dashavarga,
                &[D1, D2, D3, D7, D9, D10, D12, D16, D30, D60],
            ),
            (VargaScheme::Shodashavarga, &Varga::ALL),
        ];
        for (scheme, vargas) in hand {
            let weights = scheme.weights();
            let listed: Vec<Varga> = weights.iter().map(|w| w.0).collect();
            assert_eq!(listed, vargas, "{scheme:?}");
            // Değişmez: ağırlıkların toplamı 20'dir ve hiçbiri sıfır ya da eksi değildir.
            assert_eq!(weights.iter().map(|w| w.1).sum::<f64>(), 20.0, "{scheme:?}");
            assert!(weights.iter().all(|w| w.1 > 0.0), "{scheme:?}");
            // Ağırlıklar yarımın katıdır: çarpımlar ve toplamlar ikili kayan noktada tamdır.
            assert!(
                weights.iter().all(|w| (w.1 * 2.0).fract() == 0.0),
                "{scheme:?}"
            );
        }
        assert_eq!(VargaScheme::ALL.len(), 4);
        // Tek tek hücreler, elle: D1 6 / 5 / 3 / 3½; D9 5 / 4½ / 1½ / 3; D60 – / – / 5 / 4.
        let weight = |scheme: VargaScheme, varga: Varga| {
            scheme.weights().iter().find(|w| w.0 == varga).map(|w| w.1)
        };
        let d1: Vec<_> = VargaScheme::ALL
            .into_iter()
            .map(|s| weight(s, D1))
            .collect();
        assert_eq!(d1, [Some(6.0), Some(5.0), Some(3.0), Some(3.5)]);
        let d9: Vec<_> = VargaScheme::ALL
            .into_iter()
            .map(|s| weight(s, D9))
            .collect();
        assert_eq!(d9, [Some(5.0), Some(4.5), Some(1.5), Some(3.0)]);
        let d60: Vec<_> = VargaScheme::ALL
            .into_iter()
            .map(|s| weight(s, D60))
            .collect();
        assert_eq!(d60, [None, None, Some(5.0), Some(4.0)]);
        assert_eq!(weight(VargaScheme::Saptavarga, D7), Some(2.5));
        assert_eq!(weight(VargaScheme::Shodashavarga, D16), Some(2.0));
    }

    #[test]
    fn the_six_rungs_in_d1_and_no_separate_rung_for_exaltation_or_moolatrikona() {
        let rung = |g: Graha, p: &Positions| varga_relation(g, D1, p).expect("not a node");
        // Kendi burcu: Mars Akrep'te (220°).
        assert_eq!(rung(Mars, &with(&[])), (Sign::Scorpio, Dignity::OwnSign));
        // Güneş Koç 10°'de (yücelme derecesi): ayrı basamak yoktur, Koç'un lordu Mars'a göre
        // okunur. Doğal: Güneş → Mars dost. Geçici: Mars Akrep'te, Koç'tan 8. burç → düşman.
        // Dost + düşman = nötr.
        assert_eq!(rung(Sun, &with(&[])), (Sign::Aries, Dignity::Neutral));
        // Mars Boğa'ya (2. burç) alınırsa geçici dost: dost + dost = çok dost.
        assert_eq!(rung(Sun, &with(&[(Mars, 40.0)])).1, Dignity::GreatFriend);
        // Ay Boğa 10°'de (moolatrikona aralığı): ayrı basamak yoktur. Boğa'nın lordu Venüs de
        // Boğa'da (aynı burç → geçici düşman); doğal Ay → Venüs nötr: düşman.
        assert_eq!(rung(Moon, &with(&[(Moon, 40.0)])).1, Dignity::Enemy);
        // Venüs İkizler'e (2. burç) alınırsa nötr + dost = dost.
        assert_eq!(
            rung(Moon, &with(&[(Moon, 40.0), (Venus, 70.0)])).1,
            Dignity::Friend
        );
        // Merkür Yengeç 5°'de: lord Ay da Yengeç'te (geçici düşman); doğal Merkür → Ay düşman:
        // çok düşman.
        assert_eq!(
            rung(Mercury, &with(&[(Mercury, 95.0)])).1,
            Dignity::GreatEnemy
        );
        // Güneş Aslan 5°'te (moolatrikona aralığı) kendi burcundadır: 20.
        assert_eq!(rung(Sun, &with(&[(Sun, 125.0)])).1, Dignity::OwnSign);
        // Basamakların değeri.
        let points: Vec<f64> = [
            Dignity::OwnSign,
            Dignity::GreatFriend,
            Dignity::Friend,
            Dignity::Neutral,
            Dignity::Enemy,
            Dignity::GreatEnemy,
        ]
        .into_iter()
        .map(rung_points)
        .collect();
        assert_eq!(points, [20.0, 18.0, 15.0, 10.0, 7.0, 5.0]);
        // Düğümlere değer verilmez.
        for node in [Rahu, Ketu] {
            assert_eq!(varga_relation(node, D1, &with(&[])), None);
            assert_eq!(
                vimshopaka_score(node, VargaScheme::Shodashavarga, &with(&[])),
                0.0
            );
        }
    }

    #[test]
    fn hand_example_sun_at_the_start_of_leo() {
        // Güneş Aslan 0° (120°): tek, sabit, ateş burcu; her vargada ilk parça. Mars Başak'ta
        // (160°), Jüpiter Aslan'da (130°).
        // D1 Aslan. D2 tek burcun ilk yarısı: Güneş'in horası, Aslan. D3, D4, D12, D60 ilk
        // parça: aynı burç, Aslan. D7 ve D10 tek burçta aynı burçtan sayılır: Aslan. D9 ateş
        // burcu Koç'tan sayılır: Koç. D16 ve D45 sabit burçta Aslan'dan: Aslan. D20 sabit burçta
        // Yay'dan: Yay. D24 tek burçta Aslan'dan: Aslan. D27 ateş burcu Koç'tan: Koç. D30 tek
        // burcun ilk 5°'si Mars'ındır: Koç. D40 tek burçta Koç'tan: Koç.
        // Basamaklar: Aslan kendi burcu → 20. Koç'un lordu Mars: doğal dost; Mars Başak'ta,
        // Aslan'dan 2. burç → geçici dost; çok dost → 18. Yay'ın lordu Jüpiter: doğal dost;
        // Jüpiter Aslan'da, aynı burç → geçici düşman; nötr → 10.
        let p = with(&[(Sun, 120.0), (Mars, 160.0), (Jupiter, 130.0)]);
        let report = vimshopaka(&p, false);
        let sun = &report.grahas[0];
        assert_eq!(sun.graha, Sun);
        let signs: Vec<Sign> = sun.vargas.iter().map(|v| v.sign).collect();
        use Sign::{Aries, Leo, Sagittarius};
        assert_eq!(
            signs,
            [
                Leo,
                Leo,
                Leo,
                Leo,
                Leo,
                Aries,
                Leo,
                Leo,
                Leo,
                Sagittarius,
                Leo,
                Aries,
                Aries,
                Aries,
                Leo,
                Leo
            ]
        );
        assert_eq!(
            points_of(sun),
            [
                20.0, 20.0, 20.0, 20.0, 20.0, 18.0, 20.0, 20.0, 20.0, 10.0, 20.0, 18.0, 18.0, 18.0,
                20.0, 20.0
            ]
        );
        // Shadvarga: 6·20 + 2·20 + 4·20 + 5·18 + 2·20 + 1·18 = 388 → 19,4.
        assert!(close(sun.shadvarga, 19.4));
        // Saptavarga: 5·20 + 2·20 + 3·20 + 2½·20 + 4½·18 + 2·20 + 1·18 = 389 → 19,45.
        assert!(close(sun.saptavarga, 19.45));
        // Dashavarga: 3·20 + 1½·(20 + 20 + 20 + 18 + 20 + 20 + 20 + 18) + 5·20 = 394 → 19,7.
        assert!(close(sun.dashavarga, 19.7));
        // Shodashavarga: 70 + 20 + 20 + 10 + 10 + 54 + 10 + 10 + 40 + 5 + 10 + 9 + 18 + 9 + 10 +
        // 80 = 385 → 19,25.
        assert!(close(sun.shodashavarga, 19.25));
    }

    #[test]
    fn hand_example_moon_at_the_start_of_taurus_scores_by_the_sign_lords() {
        // Ay Boğa 0° (30°; yücelme burcu): çift, sabit, toprak burcu; her vargada ilk parça.
        // Güneş Balık'ta (340°), Mars Boğa'da (45°), Jüpiter Aslan'da (125°), Venüs İkizler'de
        // (65°), Satürn Akrep'te (220°).
        // D1, D3, D4, D12, D60 Boğa. D2 çift burcun ilk yarısı: Ay'ın horası, Yengeç. D7 çift
        // burçta yedinci burçtan: Akrep. D9 toprak burcu Oğlak'tan: Oğlak. D10 çift burçta
        // dokuzuncu burçtan: Oğlak. D16 ve D45 sabit burçta Aslan'dan: Aslan. D20 sabit burçta
        // Yay'dan: Yay. D24 çift burçta Yengeç'ten: Yengeç. D27 toprak burcu Yengeç'ten: Yengeç.
        // D30 çift burcun ilk 5°'si Venüs'ündür: Boğa. D40 çift burçta Terazi'den: Terazi.
        // Basamaklar (geçici dostluk Boğa'dan sayılır): Venüs doğal nötr, İkizler 2. burç → dost
        // → 15. Yengeç kendi burcu → 20. Mars doğal nötr, Boğa'da (aynı burç) → düşman → 7.
        // Satürn doğal nötr, Akrep 7. burç → düşman → 7. Güneş doğal dost, Balık 11. burç → çok
        // dost → 18. Jüpiter doğal nötr, Aslan 4. burç → dost → 15.
        let p = with(&[
            (Sun, 340.0),
            (Moon, 30.0),
            (Mars, 45.0),
            (Jupiter, 125.0),
            (Venus, 65.0),
            (Saturn, 220.0),
        ]);
        let report = vimshopaka(&p, false);
        let moon = &report.grahas[1];
        assert_eq!(moon.graha, Moon);
        use Sign::{Cancer, Capricorn, Leo, Libra, Sagittarius, Scorpio, Taurus};
        let signs: Vec<Sign> = moon.vargas.iter().map(|v| v.sign).collect();
        assert_eq!(
            signs,
            [
                Taurus,
                Cancer,
                Taurus,
                Taurus,
                Scorpio,
                Capricorn,
                Capricorn,
                Taurus,
                Leo,
                Sagittarius,
                Cancer,
                Cancer,
                Taurus,
                Libra,
                Leo,
                Taurus
            ]
        );
        assert_eq!(
            points_of(moon),
            [
                15.0, 20.0, 15.0, 15.0, 7.0, 7.0, 7.0, 15.0, 18.0, 15.0, 20.0, 20.0, 15.0, 15.0,
                18.0, 15.0
            ]
        );
        assert_eq!(moon.vargas[0].relation, Dignity::Friend);
        assert_eq!(moon.vargas[1].relation, Dignity::OwnSign);
        // Shadvarga: 6·15 + 2·20 + 4·15 + 5·7 + 2·15 + 1·15 = 270 → 13,5.
        assert!(close(moon.shadvarga, 13.5));
        // Saptavarga: 75 + 40 + 45 + 17,5 + 31,5 + 30 + 15 = 254 → 12,7.
        assert!(close(moon.saptavarga, 12.7));
        // Dashavarga: 3·15 + 1½·(20 + 15 + 7 + 7 + 7 + 15 + 18 + 15) + 5·15 = 276 → 13,8.
        assert!(close(moon.dashavarga, 13.8));
        // Shodashavarga: 52,5 + 20 + 15 + 7,5 + 3,5 + 21 + 3,5 + 7,5 + 36 + 7,5 + 10 + 10 + 15 +
        // 7,5 + 9 + 60 = 285,5 → 14,275.
        assert!(close(moon.shodashavarga, 14.275));
    }

    #[test]
    fn the_report_lists_seven_grahas_and_every_score_stays_between_five_and_twenty() {
        let mut lowest = f64::MAX;
        let mut highest = f64::MIN;
        for k in 0..240 {
            // Dokuz graha ayrı hızlarla yürür: ızgara bütün burçları ve parçaları gezer.
            let lons: [f64; 9] =
                std::array::from_fn(|i| f64::from(k) * (7.3 + 3.1 * i as f64) + 11.0 * i as f64);
            let p = Positions::synthetic(None, lons);
            let report = vimshopaka(&p, k % 2 == 0);
            assert_eq!(report.approximate, k % 2 == 0);
            assert_eq!(report.method, "vimshopaka.bphs_v1");
            let order: Vec<Graha> = report.grahas.iter().map(|g| g.graha).collect();
            assert_eq!(order, VIMSHOPAKA_GRAHAS);
            for row in &report.grahas {
                let vargas: Vec<Varga> = row.vargas.iter().map(|v| v.varga).collect();
                assert_eq!(vargas, Varga::ALL);
                for v in &row.vargas {
                    assert_eq!(v.points, rung_points(v.relation));
                    assert!([20.0, 18.0, 15.0, 10.0, 7.0, 5.0].contains(&v.points));
                    assert_eq!(
                        v.relation == Dignity::OwnSign,
                        own_signs(row.graha).contains(&v.sign)
                    );
                }
                let scores = [
                    row.shadvarga,
                    row.saptavarga,
                    row.dashavarga,
                    row.shodashavarga,
                ];
                for (scheme, score) in VargaScheme::ALL.into_iter().zip(scores) {
                    // Değer, raporun kendi satırlarından yeniden toplanır.
                    let again: f64 = scheme
                        .weights()
                        .iter()
                        .map(|&(varga, w)| {
                            w * row
                                .vargas
                                .iter()
                                .find(|v| v.varga == varga)
                                .expect("varga")
                                .points
                        })
                        .sum::<f64>()
                        / 20.0;
                    assert!(close(score, again), "{:?} {scheme:?}", row.graha);
                    assert!((5.0..=20.0).contains(&score), "{:?} {scheme:?}", row.graha);
                    lowest = lowest.min(score);
                    highest = highest.max(score);
                }
            }
        }
        // Izgara aralığın iki yarısını da görür.
        assert!(lowest < 10.0 && highest > 15.0, "{lowest} {highest}");
    }

    #[test]
    fn the_temporal_relation_is_read_from_d1_and_the_vargas_from_the_rasi_frame() {
        // Güneş Koç 10°'de: D9 burcu Yengeç'tir (Koç'un dördüncü navamsası: 10°–13°20′), lordu
        // Ay. Ay'ın D1 burcu Yengeç (100°), Koç'tan 4. burç → geçici dost; doğal dost: çok dost.
        assert_eq!(
            varga_relation(Sun, D9, &with(&[])),
            Some((Sign::Cancer, Dignity::GreatFriend))
        );
        // Ay Koç'a (5°) alınırsa D1'de Güneş'le aynı burçtadır → geçici düşman; dost + düşman =
        // nötr. Geçici dostluk D9 burçlarından okunsaydı sonuç değişirdi: Ay'ın D9 burcu Boğa
        // (Koç'un ikinci navamsası), Güneş'in D9 burcu Yengeç'ten 11. burçtur → geçici dost →
        // çok dost.
        assert_eq!(
            varga_relation(Sun, D9, &with(&[(Moon, 5.0)])),
            Some((Sign::Cancer, Dignity::Neutral))
        );
        // Tropikal-rasi kipi: vargalar rasi çerçevesindeki boylamdan hesaplanır. Sidereal 346°
        // (Balık 16°) + 24° = Koç 10°.
        let mut frame = with(&[(Sun, 346.0)]);
        for g in &mut frame.grahas {
            let lon = normalize(g.sidereal_lon + 24.0);
            g.rasi_sign = Sign::of(lon);
            g.deg_in_sign = deg_in_sign(lon);
        }
        assert_eq!(varga_relation(Sun, D1, &frame).expect("sun").0, Sign::Aries);
        assert_eq!(
            varga_relation(Sun, D9, &frame).expect("sun").0,
            Sign::Cancer
        );
    }
}
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-core --lib natal_extras::vimshopaka 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası (`error[E0432]: unresolved imports`: `vimshopaka`, `varga_relation`, `rung_points`, `VargaScheme` ve öteki adlar henüz yok). Test koşmaz.

- [ ] **Step 3: Kümeler, basamaklar ve rapor**

**`crates/jyotish-core/src/natal_extras/vimshopaka.rs`: uygulama, test bloğunun üstüne (arada bir boş satır)**

````rust
//! Vimshopaka bala: yedi grahanın varga kümelerindeki konum gücü, yirmi üzerinden (yol
//! haritası §4.3). Kaynak: BPHS, vargaların değerlendirilmesi bölümü; özgün olarak uygulanır.
//!
//! Dört küme vardır (shadvarga, saptavarga, dashavarga, shodashavarga); her küme vargalarına
//! toplamı 20 olan ağırlıklar verir. Graha her vargada, o vargadaki burcuna göre altı
//! basamaktan birini alır: kendi burcu 20, çok dost 18, dost 15, nötr 10, düşman 7, çok düşman
//! 5. Kümenin değeri `Σ ağırlık × basamak / 20`'dir (5–20).
//!
//! Okuma (tek okuma; `VIMSHOPAKA_METHOD`): yücelme ve moolatrikona ayrı basamak değildir; kendi
//! burcunda olmayan graha, varga burcunun lorduyla bileşik ilişkisinden okunur. Geçici dostluk
//! her vargada D1 burçlarından alınır (Shadbala'nın saptavargaja kalemindeki okuma). Vargalar
//! rasi çerçevesindeki boylamdan hesaplanır (K5). Düğümlere değer verilmez. Ağırlık tabloları
//! ve basamaklar bellekten yazılmıştır ve sahibinin onayını bekler (Y4):
//! `tests/golden/modules/natal_extras/REVIEW.md`.

use serde::{Deserialize, Serialize};

use crate::dignity::{
    compound_relation, natural_relation, own_signs, sign_lord, temporal_friend, Dignity,
};
use crate::graha::Graha;
use crate::positions::Positions;
use crate::shadbala::{rasi_lon, SHADBALA_GRAHAS};
use crate::varga::{varga_position, Varga};
use crate::zodiac::Sign;

/// Yöntem kimliği (Y5): BPHS ağırlıkları, altı basamak, geçici dostluk D1'den.
pub const VIMSHOPAKA_METHOD: &str = "vimshopaka.bphs_v1";
/// Bir kümenin en büyük değeri ve ağırlıklarının toplamı.
pub const VIMSHOPAKA_MAX: f64 = 20.0;
/// Değer verilen grahalar: Güneş … Satürn.
pub const VIMSHOPAKA_GRAHAS: [Graha; 7] = SHADBALA_GRAHAS;

/// Dört varga kümesi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum VargaScheme {
    /// Altı varga.
    Shadvarga,
    /// Yedi varga.
    Saptavarga,
    /// On varga.
    Dashavarga,
    /// On altı varga.
    Shodashavarga,
}

const SHADVARGA: [(Varga, f64); 6] = [
    (Varga::D1, 6.0),
    (Varga::D2, 2.0),
    (Varga::D3, 4.0),
    (Varga::D9, 5.0),
    (Varga::D12, 2.0),
    (Varga::D30, 1.0),
];

const SAPTAVARGA: [(Varga, f64); 7] = [
    (Varga::D1, 5.0),
    (Varga::D2, 2.0),
    (Varga::D3, 3.0),
    (Varga::D7, 2.5),
    (Varga::D9, 4.5),
    (Varga::D12, 2.0),
    (Varga::D30, 1.0),
];

const DASHAVARGA: [(Varga, f64); 10] = [
    (Varga::D1, 3.0),
    (Varga::D2, 1.5),
    (Varga::D3, 1.5),
    (Varga::D7, 1.5),
    (Varga::D9, 1.5),
    (Varga::D10, 1.5),
    (Varga::D12, 1.5),
    (Varga::D16, 1.5),
    (Varga::D30, 1.5),
    (Varga::D60, 5.0),
];

const SHODASHAVARGA: [(Varga, f64); 16] = [
    (Varga::D1, 3.5),
    (Varga::D2, 1.0),
    (Varga::D3, 1.0),
    (Varga::D4, 0.5),
    (Varga::D7, 0.5),
    (Varga::D9, 3.0),
    (Varga::D10, 0.5),
    (Varga::D12, 0.5),
    (Varga::D16, 2.0),
    (Varga::D20, 0.5),
    (Varga::D24, 0.5),
    (Varga::D27, 0.5),
    (Varga::D30, 1.0),
    (Varga::D40, 0.5),
    (Varga::D45, 0.5),
    (Varga::D60, 4.0),
];

impl VargaScheme {
    pub const ALL: [VargaScheme; 4] = [
        VargaScheme::Shadvarga,
        VargaScheme::Saptavarga,
        VargaScheme::Dashavarga,
        VargaScheme::Shodashavarga,
    ];

    /// Kümenin vargaları ve ağırlıkları, `Varga::ALL` sırasında; ağırlıkların toplamı 20'dir.
    pub fn weights(self) -> &'static [(Varga, f64)] {
        match self {
            Self::Shadvarga => &SHADVARGA,
            Self::Saptavarga => &SAPTAVARGA,
            Self::Dashavarga => &DASHAVARGA,
            Self::Shodashavarga => &SHODASHAVARGA,
        }
    }
}

/// Basamağın değeri. `varga_relation` yalnızca ilk altı değeri üretir; ötekiler 0 verir.
pub fn rung_points(relation: Dignity) -> f64 {
    match relation {
        Dignity::OwnSign => 20.0,
        Dignity::GreatFriend => 18.0,
        Dignity::Friend => 15.0,
        Dignity::Neutral => 10.0,
        Dignity::Enemy => 7.0,
        Dignity::GreatEnemy => 5.0,
        Dignity::Exalted | Dignity::Moolatrikona | Dignity::Debilitated => 0.0,
    }
}

/// Grahanın `varga`'daki burcu ve basamağı: kendi burcundaysa `OwnSign`, değilse varga burcunun
/// lorduyla bileşik ilişki (doğal ilişki + D1 burçlarından geçici dostluk). Düğümler `None`.
pub fn varga_relation(
    graha: Graha,
    varga: Varga,
    positions: &Positions,
) -> Option<(Sign, Dignity)> {
    if graha.is_node() {
        return None;
    }
    let point = positions.point(graha);
    let (sign, _) = varga_position(varga, rasi_lon(point));
    if own_signs(graha).contains(&sign) {
        return Some((sign, Dignity::OwnSign));
    }
    let lord = sign_lord(sign);
    let relation = compound_relation(
        natural_relation(graha, lord),
        temporal_friend(point.rasi_sign, positions.sign_of(lord)),
    );
    Some((sign, relation))
}

/// Grahanın bir kümedeki değeri, 5–20. Düğümler 0 verir.
pub fn vimshopaka_score(graha: Graha, scheme: VargaScheme, positions: &Positions) -> f64 {
    scheme
        .weights()
        .iter()
        .map(|&(varga, weight)| {
            varga_relation(graha, varga, positions)
                .map_or(0.0, |(_, relation)| weight * rung_points(relation))
        })
        .sum::<f64>()
        / VIMSHOPAKA_MAX
}

/// Grahanın bir vargadaki yeri ve basamağı.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct VimshopakaVarga {
    pub varga: Varga,
    /// Varga burcu.
    pub sign: Sign,
    /// Basamak: `OwnSign`, `GreatFriend`, `Friend`, `Neutral`, `Enemy` ya da `GreatEnemy`.
    pub relation: Dignity,
    /// Basamağın değeri: 20, 18, 15, 10, 7 ya da 5.
    pub points: f64,
}

/// Bir grahanın on altı vargası ve dört kümedeki değeri.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GrahaVimshopaka {
    pub graha: Graha,
    /// On altı varga, `Varga::ALL` sırasında.
    pub vargas: Vec<VimshopakaVarga>,
    pub shadvarga: f64,
    pub saptavarga: f64,
    pub dashavarga: f64,
    pub shodashavarga: f64,
}

/// `compute_vimshopaka` sonucu. AI yorum paneli için sözleşmedir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct VimshopakaReport {
    /// `VIMSHOPAKA_GRAHAS` sırasında yedi graha.
    pub grahas: Vec<GrahaVimshopaka>,
    /// Doğum saati bilinmiyor: yüksek bölümlü vargalar (D16 ve üstü) birkaç dakikada burç
    /// değiştirir; değerler öğlen konumlarındandır.
    pub approximate: bool,
    pub method: String,
}

/// Konumlardan rapor. Lagna gerekmez.
pub fn vimshopaka(positions: &Positions, approximate: bool) -> VimshopakaReport {
    let grahas = VIMSHOPAKA_GRAHAS
        .into_iter()
        .map(|graha| GrahaVimshopaka {
            graha,
            vargas: Varga::ALL
                .into_iter()
                .filter_map(|varga| {
                    varga_relation(graha, varga, positions).map(|(sign, relation)| {
                        VimshopakaVarga {
                            varga,
                            sign,
                            relation,
                            points: rung_points(relation),
                        }
                    })
                })
                .collect(),
            shadvarga: vimshopaka_score(graha, VargaScheme::Shadvarga, positions),
            saptavarga: vimshopaka_score(graha, VargaScheme::Saptavarga, positions),
            dashavarga: vimshopaka_score(graha, VargaScheme::Dashavarga, positions),
            shodashavarga: vimshopaka_score(graha, VargaScheme::Shodashavarga, positions),
        })
        .collect();
    VimshopakaReport {
        grahas,
        approximate,
        method: VIMSHOPAKA_METHOD.to_owned(),
    }
}
````

- [ ] **Step 4: REVIEW kaydına Vimshopaka bölümü**

§3.2'deki tablo `SHADVARGA`, `SAPTAVARGA`, `DASHAVARGA` ve `SHODASHAVARGA` ile, §3.3'teki tablo `rung_points` ile hücre hücre aynı olmalıdır; yazdıktan sonra kodla gözle karşılaştırılır.

**Dosyanın sonuna, önüne bir boş satır konarak eklenir:** `tests/golden/modules/natal_extras/REVIEW.md`

````markdown
## 3. Vimshopaka bala (yöntem `vimshopaka.bphs_v1`)

### 3.1 Okuma kuralları

1. **Grahalar.** Değer yedi grahaya verilir (Güneş … Satürn); Rahu ve Ketu'ya verilmez.
2. **Basamak.** Graha her vargada, o vargadaki burcuna göre altı basamaktan birini alır (3.3).
   Kendi burcu, grahanın sahibi olduğu burçtur (Güneş ve Ay için tek burç, ötekiler için iki).
3. **Bileşik ilişki.** Graha kendi burcunda değilse varga burcunun lorduyla bileşik ilişkisine
   (panchadha maitri) bakılır: doğal ilişki ile geçici dostluğun birleşimi. **Geçici dostluk her
   vargada D1 burçlarından okunur:** lordun D1 burcu, grahanın D1 burcundan 2, 3, 4, 10, 11 ya da
   12. burçsa dosttur. Shadbala'nın saptavargaja kalemindeki okumanın aynısıdır.
4. **Yücelme ve moolatrikona ayrı basamak değildir.** Yücelme burcundaki graha, o burcun lorduyla
   ilişkisinden okunur. Moolatrikona burcu altı graha için zaten kendi burcudur; Ay'ın
   moolatrikona burcu Boğa'dır ve Venüs'le ilişkiden okunur. Düşüş için de ayrı basamak yoktur.
5. **Vargalar** rasi çerçevesindeki boylamdan hesaplanır (K5); v1'in on altı varga tanımı
   kullanılır.
6. **Kümenin değeri** = Σ (varga ağırlığı × basamak değeri) / 20. En düşük 5, en yüksek 20; her
   değer 0,025'in katıdır.
7. **Doğum saati bilinmiyorsa** rapor yine verilir ve "yaklaşık" işaretini taşır; saat "yaklaşık"
   girilmişse panel D16 ve üstü vargaların saate duyarlı olduğunu söyler.

### 3.2 Varga ağırlıkları (bellekten; onay bekler)

Motorun tabloları: `crates/jyotish-core/src/natal_extras/vimshopaka.rs`, `SHADVARGA`,
`SAPTAVARGA`, `DASHAVARGA`, `SHODASHAVARGA`.

| Varga | Shadvarga | Saptavarga | Dashavarga | Shodashavarga |
|---|---|---|---|---|
| D1 | 6 | 5 | 3 | 3½ |
| D2 | 2 | 2 | 1½ | 1 |
| D3 | 4 | 3 | 1½ | 1 |
| D4 | – | – | – | ½ |
| D7 | – | 2½ | 1½ | ½ |
| D9 | 5 | 4½ | 1½ | 3 |
| D10 | – | – | 1½ | ½ |
| D12 | 2 | 2 | 1½ | ½ |
| D16 | – | – | 1½ | 2 |
| D20 | – | – | – | ½ |
| D24 | – | – | – | ½ |
| D27 | – | – | – | ½ |
| D30 | 1 | 1 | 1½ | 1 |
| D40 | – | – | – | ½ |
| D45 | – | – | – | ½ |
| D60 | – | – | 5 | 4 |
| **Toplam** | 20 | 20 | 20 | 20 |

### 3.3 Basamaklar (bellekten; onay bekler)

| Basamak | Kendi burcu | Çok dost | Dost | Nötr | Düşman | Çok düşman |
|---|---|---|---|---|---|---|
| Değer | 20 | 18 | 15 | 10 | 7 | 5 |

### 3.4 Onay bekleyen noktalar (sahibi)

- [ ] Dört kümenin ağırlıkları, hücre hücre (her sütunun toplamı 20).
- [ ] Altı basamağın değerleri (20, 18, 15, 10, 7, 5).
- [ ] Yücelme ve moolatrikonanın ayrı basamak olmaması (öteki okuma: ikisi de 20 sayılır).
- [ ] Geçici dostluğun her vargada D1 burçlarından okunması (öteki okuma: o vargadan).
- [ ] Rahu ve Ketu'ya değer verilmemesi.
````

- [ ] **Step 5: Testler ve kapılar**

```bash
cargo fmt --all
cargo test -p jyotish-core --lib natal_extras::vimshopaka 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
grep -c '^| D[0-9]' tests/golden/modules/natal_extras/REVIEW.md
git diff --stat -- crates/jyotish-core/src/shadbala crates/jyotish-core/src/dignity.rs crates/jyotish-core/src/varga.rs
```
Expected: `6 passed`; lib `425 passed`, golden `8 passed`, `modules_golden` `17 passed`; fmt ve clippy çıktısız; sayaç `0`; `grep` `16` yazar (on altı varga satırı); son komut çıktısız (Shadbala, dignity ve varga değişmedi).

- [ ] **Step 6: Commit**

```bash
git add crates/jyotish-core/src/natal_extras crates/jyotish-core/src/lib.rs tests/golden/modules/natal_extras/REVIEW.md
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): Vimshopaka bala: four varga sets weighing to twenty, six rungs, temporal friendship read from D1" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: dört dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 5: El örnekleri ve komutlar: toplayıcılar, üç komut, gerçek motorla çapraz denetim, bağlamalar ve fikstür

Yol haritası görev taslağının (5). maddesi; K1–K3, K9; D7, D11, D17–D20, D23. Toplayıcılar temel haritayı hesaplar ve Task 1–4'ün saf fonksiyonlarına verir. Komutlar yeni bir dosyadadır ve `module_commands` kalıbını izler. Gerçek motorlu denetimler yeni bir entegrasyon testindedir (ayrı süreç, kendi motoru): `tests/modules.rs` değişmez. `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` komutla yeniden üretilir (C10).

**Files:**
- Create: `crates/jyotish-core/src/natal_extras/collect.rs`, `app/src-tauri/src/natal_extras_commands.rs`, `app/src-tauri/tests/natal_extras.rs`
- Modify: `crates/jyotish-core/src/natal_extras/mod.rs`, `crates/jyotish-core/src/lib.rs`, `app/src-tauri/src/lib.rs`, `app/src-tauri/tests/ui_fixture.rs`, `app/ui/bindings.ts` (üretilir), `app/ui/test/fixtures/delhi.json` (üretilir), `app/ui/test/fixture.ts`, `app/ui/api.ts`, `app/ui/api.test.ts`
- Test: `collect.rs` `mod tests` (4); `tests/natal_extras.rs` (1 test, dört denetim fonksiyonu); `api.test.ts` (var olan tablo üç satır kazanır)

**Interfaces:**
- Consumes: Task 1–4; `chart::compute_chart_for`; `positions::{compute_positions, ChartInput, Positions}`; `settings::JD_UT_RANGE`; `transit::compute_transits`; uygulamada `client_commands::blocking`, `commands::{CommandError, compute_chart_blocking}`, `ephemeris::{Engine, SwissEph}`.
- Produces:
  - Çekirdek (`jyotish_core` kökünden): `compute_panchang_extras<E: Ephemeris + ?Sized>(eph: &mut E, input: &ChartInput) -> Result<PanchangExtras, CoreError>`; `compute_navatara<E: Ephemeris + ?Sized>(eph: &mut E, input: &ChartInput, transit_jd_ut: Option<f64>) -> Result<Navatara, CoreError>` (geçiş anı aralık dışındaysa `CoreError::InvalidTransitInput("transit_jd_ut")`, haritadan önce); `compute_vimshopaka<E: Ephemeris + ?Sized>(eph: &mut E, input: &ChartInput) -> Result<VimshopakaReport, CoreError>`.
  - Uygulama: `natal_extras_commands::{compute_panchang_extras_blocking(&Engine, ChartInput), compute_navatara_blocking(&Engine, ChartInput, Option<f64>), compute_vimshopaka_blocking(&Engine, ChartInput)}` ve aynı adlı üç `#[tauri::command]`.
  - Arayüz: `api.computePanchangExtras(input)`, `api.computeNavatara(input, transitJdUt)`, `api.computeVimshopaka(input)`; bağlamalarda `PanchangExtras`, `YogiFacts`, `Navatara`, `NavataraRow`, `TransitTara`, `VimshopakaReport`, `GrahaVimshopaka`, `VimshopakaVarga` (`VargaScheme` bağlamalara girmez: hiçbir komut onu taşımaz); fikstürde `natal_extras`.

- [ ] **Step 1: Kayıt ve testler: sahte gökte üç rapor, gerçek motorla çapraz denetim, hata eşlemesi ve bütçeler**

`collect.rs` testleri sahte ephemeris'in göğündedir (boylamlar test dosyasındaki yorumda): panchang ekleri (`yogi.rs`'in B örneği; saat bilinmezliği; tropikal-rasi kipi), navatara (doğum tablosu ve on gün sonraki geçiş taraları), Vimshopaka (saf fonksiyonla aynılık ve bir hücre elle), girdi denetimi (geçiş anı haritadan önce; sayı prashnası). `tests/natal_extras.rs` gerçek motorla beş golden anı iki zodyak kipinde gezer: tithi ve nitya yoga v1'in panchangıyla, navatara satırları haritanın nakshatralarıyla, geçiş taraları `compute_transits`'in nakshatralarıyla, varga burçları `Chart.vargas` ile, yedi vargadaki basamaklar `saptavargaja_points` ile karşılaştırılır; Delhi anının olguları golden dosyadaki tithi ve yogadan elle türetilmiştir; bütçeler `assert!` ile doğrulanır.

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index a18fd91..5f273da 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -4,6 +4,7 @@ pub mod commands;
 pub mod data_dir;
 pub mod module_commands;
 pub mod muhurta_commands;
+pub mod natal_extras_commands;
 pub mod prashna_commands;
 pub mod search_commands;
 pub mod store_commands;
@@ -40,6 +41,9 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             module_commands::compute_shadbala,
             module_commands::compute_yogas,
             module_commands::compute_compat,
+            natal_extras_commands::compute_panchang_extras,
+            natal_extras_commands::compute_navatara,
+            natal_extras_commands::compute_vimshopaka,
             muhurta_commands::day_panchang,
             muhurta_commands::muhurta_search,
             prashna_commands::create_prashna,
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index b55094b..cc9bcf1 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -90,12 +90,12 @@ pub use muhurta::{
     MUHURTA_MAX_RANGE_DAYS, MUHURTA_METHOD,
 };
 pub use natal_extras::{
-    avayogi_nakshatra, dagdha_rasis, nakshatra_lord, navatara, navatara_rows, panchang_extras,
-    rasi_frame_offset, rung_points, tara_cycle, transit_taras, varga_relation, vimshopaka,
-    vimshopaka_score, yogi_facts, yogi_point, GrahaVimshopaka, Navatara, NavataraRow,
-    PanchangExtras, TransitTara, VargaScheme, VimshopakaReport, VimshopakaVarga, YogiFacts,
-    AVAYOGI_COUNT, NAVATARA_METHOD, PANCHANG_EXTRAS_METHOD, VIMSHOPAKA_GRAHAS, VIMSHOPAKA_MAX,
-    VIMSHOPAKA_METHOD, YOGI_ARC_DEG,
+    avayogi_nakshatra, compute_navatara, compute_panchang_extras, compute_vimshopaka, dagdha_rasis,
+    nakshatra_lord, navatara, navatara_rows, panchang_extras, rasi_frame_offset, rung_points,
+    tara_cycle, transit_taras, varga_relation, vimshopaka, vimshopaka_score, yogi_facts,
+    yogi_point, GrahaVimshopaka, Navatara, NavataraRow, PanchangExtras, TransitTara, VargaScheme,
+    VimshopakaReport, VimshopakaVarga, YogiFacts, AVAYOGI_COUNT, NAVATARA_METHOD,
+    PANCHANG_EXTRAS_METHOD, VIMSHOPAKA_GRAHAS, VIMSHOPAKA_MAX, VIMSHOPAKA_METHOD, YOGI_ARC_DEG,
 };
 pub use nature::{kuja_dosha, moon_is_waxing, nature, KujaDosha, Nature, KUJA_DOSHA_HOUSES};
 pub use panchang::{
diff --git a/crates/jyotish-core/src/natal_extras/mod.rs b/crates/jyotish-core/src/natal_extras/mod.rs
index 4cbfcab..11aa684 100644
--- a/crates/jyotish-core/src/natal_extras/mod.rs
+++ b/crates/jyotish-core/src/natal_extras/mod.rs
@@ -4,11 +4,13 @@
 //! eklenmez, panel açıkken istenir (K1). Hiçbiri lagna istemez; doğum saati bilinmiyorsa değerler
 //! öğlen konumlarından gelir ve rapor `approximate` işaretini taşır.
 
+mod collect;
 pub mod dagdha;
 pub mod navatara;
 pub mod vimshopaka;
 pub mod yogi;
 
+pub use collect::{compute_navatara, compute_panchang_extras, compute_vimshopaka};
 pub use dagdha::{dagdha_rasis, panchang_extras, PanchangExtras, PANCHANG_EXTRAS_METHOD};
 pub use navatara::{
     navatara, navatara_rows, tara_cycle, transit_taras, Navatara, NavataraRow, TransitTara,
````

**Yeni dosya:** `crates/jyotish-core/src/natal_extras/collect.rs` (şimdilik yalnızca test bloğu)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::{CalcSettings, ZodiacMode};
    use crate::zodiac::Sign;
    use Graha::*;

    const JD: f64 = 2_451_545.0;

    fn input(settings: CalcSettings, time_known: bool) -> ChartInput {
        ChartInput {
            jd_ut: JD,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known,
            lagna_number: None,
        }
    }

    // FakeEph (ayanamsa 24°): sidereal Güneş 346°, Ay 71°, Mars 176°, Merkür 356°, Jüpiter 226°,
    // Venüs 16°, Satürn 276°, Rahu 96° (ortalama düğüm), Ketu 276°.

    #[test]
    fn panchang_extras_of_the_fake_sky() {
        // `yogi.rs`'in B örneği: nokta 150°20′, Uttara Phalguni (12) 2. pada, yogi Güneş; Başak
        // 0°20′, eş yogi Merkür; avayogi nakshatrası Anuradha (17), avayogi Satürn. Uzanım
        // 71° − 346° = 85° → 8. tithi: İkizler ve Başak.
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        let got = compute_panchang_extras(&mut eph, &input(s.clone(), true)).unwrap();
        assert!((got.yogi.point_lon - (150.0 + 20.0 / 60.0)).abs() < 1e-9);
        assert_eq!((got.yogi.nakshatra, got.yogi.pada), (12, 2));
        assert_eq!((got.yogi.yogi, got.yogi.co_yogi), (Sun, Mercury));
        assert_eq!(got.yogi.sign, Sign::Virgo);
        assert_eq!((got.yogi.avayogi_nakshatra, got.yogi.avayogi), (17, Saturn));
        assert_eq!(got.tithi, 8);
        assert_eq!(got.dagdha_rasis, [Sign::Gemini, Sign::Virgo]);
        assert!(!got.approximate);
        // Doğum saati bilinmiyor: aynı değerler, yaklaşık işaretiyle.
        let unknown = compute_panchang_extras(&mut eph, &input(s.clone(), false)).unwrap();
        assert!(unknown.approximate);
        assert_eq!((&unknown.yogi, unknown.tithi), (&got.yogi, got.tithi));
        // Tropikal-rasi kipi: nokta, nakshatra ve lordlar değişmez; burç rasi çerçevesindedir
        // (150°20′ + 24° = 174°20′, yine Başak; burç içi derece 24°20′ olur).
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..s
        };
        let t = compute_panchang_extras(&mut eph, &input(tropical, true)).unwrap();
        assert!((t.yogi.point_lon - got.yogi.point_lon).abs() < 1e-9);
        assert_eq!((t.yogi.nakshatra, t.yogi.yogi), (12, Sun));
        assert_eq!((t.yogi.sign, t.yogi.co_yogi), (Sign::Virgo, Mercury));
        assert!((t.yogi.deg_in_sign - (24.0 + 20.0 / 60.0)).abs() < 1e-9);
        assert_eq!((t.tithi, &t.dagdha_rasis), (8, &got.dagdha_rasis));
    }

    #[test]
    fn navatara_of_the_fake_sky_with_and_without_a_transit_moment() {
        // Ay 71°: Ardra (6: 66°40′–80°). Grahalar: Güneş 346° Uttara Bhadrapada (26); Mars 176°
        // Chitra (14); Merkür 356° Revati (27); Jüpiter 226° Anuradha (17); Venüs 16° Bharani
        // (2); Satürn ve Ketu 276° Uttara Ashadha (21); Rahu 96° Pushya (8).
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        let natal = compute_navatara(&mut eph, &input(s.clone(), true), None).unwrap();
        assert_eq!(natal.janma_nakshatra, 6);
        assert_eq!((natal.transit.as_ref(), natal.transit_jd_ut), (None, None));
        let at = |n: u8| natal.rows.iter().find(|r| r.nakshatra == n).expect("row");
        assert_eq!(at(6).grahas, [Moon]);
        // Ardra'dan Pushya'ya sayım 3 → Vipat, 1. tur.
        assert_eq!(
            (at(8).grahas.as_slice(), at(8).tara.number, at(8).cycle),
            (&[Rahu][..], 3, 1)
        );
        // Chitra'ya sayım 9 → Parama Mitra, 1. tur.
        assert_eq!(
            (at(14).grahas.as_slice(), at(14).tara.number, at(14).cycle),
            (&[Mars][..], 9, 1)
        );
        // Anuradha'ya sayım 12 → 3. tara, 2. tur.
        assert_eq!(
            (at(17).grahas.as_slice(), at(17).tara.number, at(17).cycle),
            (&[Jupiter][..], 3, 2)
        );
        // Uttara Ashadha'ya sayım 16 → 7. tara, 2. tur; iki graha.
        assert_eq!(
            (at(21).grahas.as_slice(), at(21).tara.number),
            (&[Saturn, Ketu][..], 7)
        );
        // Uttara Bhadrapada'ya sayım 21 → 3. tara, 3. tur; Revati'ye 22 → 4. tara; Bharani'ye
        // 24 → 6. tara.
        assert_eq!(
            (at(26).grahas.as_slice(), at(26).tara.number, at(26).cycle),
            (&[Sun][..], 3, 3)
        );
        assert_eq!(
            (at(27).grahas.as_slice(), at(27).tara.number),
            (&[Mercury][..], 4)
        );
        assert_eq!(
            (at(2).grahas.as_slice(), at(2).tara.number),
            (&[Venus][..], 6)
        );

        // On gün sonra: Ay 71° + 132° = 203° → Vishakha (16), 1. pada (200°–203°20′): sayım 11 →
        // Sampat (2), 2. tur. Güneş 346° + 9,8° = 355,8° → Revati (27), 3. pada (353°20′–
        // 356°40′): sayım 22 → Kshema (4), 3. tur.
        let with = compute_navatara(&mut eph, &input(s.clone(), false), Some(JD + 10.0)).unwrap();
        assert!(with.approximate);
        assert_eq!(with.transit_jd_ut, Some(JD + 10.0));
        assert_eq!(with.rows, natal.rows);
        let transit = with.transit.expect("transit");
        let order: Vec<Graha> = transit.iter().map(|t| t.graha).collect();
        assert_eq!(order, Graha::ALL);
        let moon = &transit[Moon as usize];
        assert_eq!(
            (moon.nakshatra, moon.pada, moon.tara.number, moon.cycle),
            (16, 1, 2, 2)
        );
        assert!(moon.tara.favourable);
        let sun = &transit[Sun as usize];
        assert_eq!(
            (sun.nakshatra, sun.pada, sun.tara.number, sun.cycle),
            (27, 3, 4, 3)
        );
    }

    #[test]
    fn vimshopaka_collector_scores_the_chart_positions() {
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        let positions = compute_positions(&mut eph, &input(s.clone(), true)).unwrap();
        let known = compute_vimshopaka(&mut eph, &input(s.clone(), true)).unwrap();
        assert_eq!(known, vimshopaka(&positions, false));
        // Güneş sidereal 346° (Balık 16°): D1'de Balık'ın lordu Jüpiter Akrep'tedir (226°),
        // Balık'tan 9. burç → geçici düşman; doğal dost: nötr → 10.
        assert_eq!(known.grahas[0].vargas[0].sign, Sign::Pisces);
        assert_eq!(known.grahas[0].vargas[0].points, 10.0);
        // Doğum saati bilinmiyor: rapor yine döner (lagna gerekmez), yaklaşık işaretiyle.
        let unknown = compute_vimshopaka(&mut eph, &input(s, false)).unwrap();
        assert!(unknown.approximate);
        assert_eq!(unknown.grahas, known.grahas);
    }

    #[test]
    fn collectors_check_the_input_before_and_through_the_chart() {
        let mut eph = FakeEph::default();
        let mut bad = input(CalcSettings::default(), true);
        bad.lat = 91.0;
        let location = CoreError::InvalidLocation {
            lat: 91.0,
            lon: 29.0,
        };
        assert_eq!(
            compute_panchang_extras(&mut eph, &bad),
            Err(location.clone())
        );
        assert_eq!(compute_vimshopaka(&mut eph, &bad), Err(location.clone()));
        assert_eq!(compute_navatara(&mut eph, &bad, None), Err(location));
        // Geçiş anı haritadan önce denetlenir ve hata yalnızca alan adını taşır.
        for jd in [f64::NAN, f64::INFINITY, 0.0, 1e9] {
            assert_eq!(
                compute_navatara(&mut eph, &bad, Some(jd)),
                Err(CoreError::InvalidTransitInput("transit_jd_ut"))
            );
        }
        // Sayı prashnasında grahalar anın grahalarıdır: üç rapor da sayısız haritayla aynıdır.
        let plain = input(CalcSettings::default(), true);
        let mut numbered = plain.clone();
        numbered.lagna_number = Some(30);
        assert_eq!(
            compute_panchang_extras(&mut eph, &numbered),
            compute_panchang_extras(&mut eph, &plain)
        );
        assert_eq!(
            compute_navatara(&mut eph, &numbered, None),
            compute_navatara(&mut eph, &plain, None)
        );
        assert_eq!(
            compute_vimshopaka(&mut eph, &numbered),
            compute_vimshopaka(&mut eph, &plain)
        );
    }
}
````

**Yeni dosya:** `app/src-tauri/tests/natal_extras.rs`

````rust
//! Doğum anı eklerinin komut gövdeleri, gerçek `Engine` ile uçtan uca (yol haritası Plan 23;
//! ileri modüller spec'i §13 "Komut katmanı"). Tek test: süreçte yalnızca bir SE örneği
//! açılabilir. Çapraz denetimler v1'de golden fixture'larla doğrulanmış haritaya (tithi, nitya
//! yoga, nakshatra, varga burçları) ve Shadbala'nın saptavargaja kalemine dayanır.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use ephemeris::{Engine, SwissEph};
use jyotish_core::dignity::{own_signs, sign_lord};
use jyotish_core::{
    compute_positions, compute_transits, nakshatra_lord, nakshatra_pada, normalize,
    saptavargaja_points, tara_bala, tara_cycle, CalcSettings, Chart, ChartInput, Graha, Sign,
    Varga, VargaScheme, ZodiacMode, NAVATARA_METHOD, PANCHANG_EXTRAS_METHOD, SAPTAVARGA,
    VIMSHOPAKA_GRAHAS, VIMSHOPAKA_METHOD,
};
use jyotish_desk_lib::commands::compute_chart_blocking;
use jyotish_desk_lib::natal_extras_commands::{
    compute_navatara_blocking, compute_panchang_extras_blocking, compute_vimshopaka_blocking,
};
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
                lagna_number: None,
            };
            (v["id"].as_str().expect("id").to_owned(), input)
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Her golden an iki zodyak kipinde.
fn cases() -> Vec<(String, ChartInput)> {
    let mut out = Vec::new();
    for (id, input) in golden_inputs() {
        let tropical = ChartInput {
            settings: CalcSettings {
                zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
                ..input.settings.clone()
            },
            ..input.clone()
        };
        out.push((format!("{id} (tropical rasi)"), tropical));
        out.push((id, input));
    }
    out
}

fn chart_of(engine: &Engine, input: &ChartInput) -> Chart {
    compute_chart_blocking(
        engine,
        input.jd_ut,
        input.lat,
        input.lon,
        input.settings.clone(),
    )
    .expect("chart")
}

fn unknown_time(input: &ChartInput) -> ChartInput {
    ChartInput {
        time_known: false,
        ..input.clone()
    }
}

fn median_of_15(mut run: impl FnMut()) -> (Duration, Duration) {
    run();
    let mut times: Vec<Duration> = (0..15)
        .map(|_| {
            let t = Instant::now();
            run();
            t.elapsed()
        })
        .collect();
    times.sort();
    (times[times.len() / 2], times[times.len() - 1])
}

/// Motoru tek test açar; modül başına denetimler ayrı fonksiyonlardadır.
#[test]
fn natal_extras_commands_run_on_the_engine_thread_and_agree_with_the_chart() {
    let engine = Engine::spawn(root().join("../../data/ephe")).expect("spawn engine");
    let cases = cases();
    assert!(cases.len() >= 10, "{} cases", cases.len());
    panchang_extras_checks(&engine, &cases);
    navatara_checks(&engine, &cases);
    vimshopaka_checks(&engine, &cases);
    error_and_budget_checks(&engine, &cases);
}

fn panchang_extras_checks(engine: &Engine, cases: &[(String, ChartInput)]) {
    for (id, input) in cases {
        let extras = compute_panchang_extras_blocking(engine, input.clone())
            .unwrap_or_else(|e| panic!("{id}: {}", e.code));
        let chart = chart_of(engine, input);
        assert_eq!(extras.method, PANCHANG_EXTRAS_METHOD, "{id}");
        assert!(!extras.approximate, "{id}");
        // Tithi, v1'in panchangındaki tithidir.
        assert_eq!(extras.tithi, chart.panchang.tithi.no, "{id}");
        // Yogi nakshatrası nitya yogadan yedi ileridedir (27'de sarılır).
        let yogi = &extras.yogi;
        assert_eq!(
            yogi.nakshatra,
            (chart.panchang.yoga.no + 6) % 27 + 1,
            "{id}"
        );
        assert_eq!(
            nakshatra_pada(yogi.point_lon),
            (yogi.nakshatra, yogi.pada),
            "{id}"
        );
        assert_eq!(yogi.yogi, nakshatra_lord(yogi.nakshatra), "{id}");
        // Nokta, haritanın Güneş ve Ay sidereal boylamlarından.
        let sum = chart.grahas[Graha::Sun as usize].sidereal_lon
            + chart.grahas[Graha::Moon as usize].sidereal_lon;
        assert!(
            (yogi.point_lon - normalize(sum + 280.0 / 3.0)).abs() < 1e-9,
            "{id}"
        );
        // Burç rasi çerçevesindedir: tropikal-rasi kipinde nokta ayanamsa kadar ötelenir.
        let frame = match input.settings.zodiac {
            ZodiacMode::Sidereal => yogi.point_lon,
            ZodiacMode::TropicalRasiSiderealNakshatra => {
                normalize(yogi.point_lon + chart.ayanamsa_deg)
            }
        };
        // Burç sınırına bir yay saniyesinden yakın nokta ayrıca denetlenmez.
        let edge = frame % 30.0;
        if (1.0 / 3600.0..30.0 - 1.0 / 3600.0).contains(&edge) {
            assert_eq!(yogi.sign, Sign::of(frame), "{id}");
            assert!((yogi.deg_in_sign - edge).abs() < 1e-6, "{id}");
        }
        assert_eq!(yogi.co_yogi, sign_lord(yogi.sign), "{id}");
        // Avayogi: altıncı nakshatra; ikinci yazımla (nokta + 186°40′) aynı lord.
        assert_eq!(
            yogi.avayogi_nakshatra,
            (yogi.nakshatra + 4) % 27 + 1,
            "{id}"
        );
        assert_eq!(yogi.avayogi, nakshatra_lord(yogi.avayogi_nakshatra), "{id}");
        let (other, _) = nakshatra_pada(yogi.point_lon + 560.0 / 3.0);
        assert_eq!(nakshatra_lord(other), yogi.avayogi, "{id}");
        // Dagdha rasi: dolunay ve yeni ayda yok, chaturdashide dört, ötekilerde iki burç.
        let expected = match (extras.tithi - 1) % 15 + 1 {
            15 => 0,
            14 => 4,
            _ => 2,
        };
        assert_eq!(extras.dagdha_rasis.len(), expected, "{id}");
        // Doğum saati bilinmiyor: aynı an için aynı değerler, yaklaşık işaretiyle.
        let unknown = compute_panchang_extras_blocking(engine, unknown_time(input)).expect(id);
        assert!(unknown.approximate, "{id}");
        assert_eq!(
            (&unknown.yogi, unknown.tithi, &unknown.dagdha_rasis),
            (&extras.yogi, extras.tithi, &extras.dagdha_rasis),
            "{id}"
        );
    }
    // Delhi 2000-01-01 (tests/golden/new-delhi-2000-01-01.json: tithi 26, nitya yoga 8 Dhriti):
    // yogi nakshatrası 8 + 7 = 15 (Swati), lordu Rahu; avayogi nakshatrası 20 (Purva Ashadha),
    // lordu Venüs; Krishna Ekadashi (paksha içi 11): Yay ve Balık.
    let (_, delhi) = cases
        .iter()
        .find(|(id, _)| id == "new-delhi-2000-01-01")
        .expect("delhi");
    let extras = compute_panchang_extras_blocking(engine, delhi.clone()).expect("delhi");
    assert_eq!(extras.tithi, 26);
    assert_eq!((extras.yogi.nakshatra, extras.yogi.yogi), (15, Graha::Rahu));
    assert_eq!(
        (extras.yogi.avayogi_nakshatra, extras.yogi.avayogi),
        (20, Graha::Venus)
    );
    assert_eq!(extras.dagdha_rasis, [Sign::Sagittarius, Sign::Pisces]);
}

fn navatara_checks(engine: &Engine, cases: &[(String, ChartInput)]) {
    for (id, input) in cases {
        let chart = chart_of(engine, input);
        let natal = compute_navatara_blocking(engine, input.clone(), None)
            .expect("navatara without transit");
        assert_eq!(natal.method, NAVATARA_METHOD, "{id}");
        assert_eq!(
            (natal.transit.as_ref(), natal.transit_jd_ut),
            (None, None),
            "{id}"
        );
        let janma = chart.grahas[Graha::Moon as usize].nakshatra;
        assert_eq!(natal.janma_nakshatra, janma, "{id}");
        assert_eq!(natal.rows.len(), 27, "{id}");
        for (i, row) in natal.rows.iter().enumerate() {
            assert_eq!(
                usize::from(row.nakshatra),
                (usize::from(janma) - 1 + i) % 27 + 1
            );
            assert_eq!(row.tara, tara_bala(janma, row.nakshatra), "{id} row {i}");
            assert_eq!(usize::from(row.tara.number), i % 9 + 1, "{id} row {i}");
            assert_eq!(usize::from(row.cycle), i / 9 + 1, "{id} row {i}");
            assert_eq!(row.lord, nakshatra_lord(row.nakshatra), "{id} row {i}");
            // Satırdaki grahalar, haritada o nakshatrada duranlardır.
            let expected: Vec<Graha> = chart
                .grahas
                .iter()
                .filter(|g| g.nakshatra == row.nakshatra)
                .map(|g| g.graha)
                .collect();
            assert_eq!(row.grahas, expected, "{id} row {i}");
        }
        // Geçiş anı doğum anının kendisiyse her graha kendi satırının tarasını alır.
        let same = compute_navatara_blocking(engine, input.clone(), Some(input.jd_ut))
            .expect("navatara at the birth moment");
        assert_eq!(same.rows, natal.rows, "{id}");
        for t in same.transit.as_ref().expect("transit") {
            let g = &chart.grahas[t.graha as usize];
            assert_eq!(
                (t.nakshatra, t.pada),
                (g.nakshatra, g.pada),
                "{id} {:?}",
                t.graha
            );
            let row = natal
                .rows
                .iter()
                .find(|r| r.nakshatra == t.nakshatra)
                .expect("row");
            assert_eq!(
                (t.tara, t.cycle),
                (row.tara, row.cycle),
                "{id} {:?}",
                t.graha
            );
        }
        // Başka bir an (1 000 gün sonra): nakshatralar transit komutunun verdiğiyle aynıdır.
        let later = input.jd_ut + 1000.0;
        let moved = compute_navatara_blocking(engine, input.clone(), Some(later))
            .expect("navatara at a later moment");
        assert_eq!(moved.transit_jd_ut, Some(later), "{id}");
        let snapshot = {
            let (settings, bhavas) = (input.settings.clone(), chart.bhavas.clone());
            engine
                .call(move |se: &mut SwissEph| compute_transits(se, later, &settings, &bhavas))
                .expect("engine")
                .expect("transits")
        };
        let transit = moved.transit.expect("transit");
        assert_eq!(transit.len(), 9, "{id}");
        for (t, g) in transit.iter().zip(&snapshot.grahas) {
            assert_eq!(
                (t.graha, t.nakshatra, t.pada),
                (g.graha, g.nakshatra, g.pada),
                "{id}"
            );
            assert_eq!(t.tara, tara_bala(janma, g.nakshatra), "{id}");
            assert_eq!(t.cycle, tara_cycle(janma, g.nakshatra), "{id}");
        }
        // Doğum saati bilinmiyor: tablo aynı an için aynıdır, yaklaşık işaretiyle.
        let unknown =
            compute_navatara_blocking(engine, unknown_time(input), None).expect("time unknown");
        assert!(unknown.approximate && !natal.approximate, "{id}");
        assert_eq!(unknown.rows, natal.rows, "{id}");
    }
}

fn vimshopaka_checks(engine: &Engine, cases: &[(String, ChartInput)]) {
    for (id, input) in cases {
        let chart = chart_of(engine, input);
        let report = compute_vimshopaka_blocking(engine, input.clone()).expect("vimshopaka");
        assert_eq!(report.method, VIMSHOPAKA_METHOD, "{id}");
        assert!(!report.approximate, "{id}");
        let order: Vec<Graha> = report.grahas.iter().map(|g| g.graha).collect();
        assert_eq!(order, VIMSHOPAKA_GRAHAS, "{id}");
        let positions = {
            let input = input.clone();
            engine
                .call(move |se: &mut SwissEph| compute_positions(se, &input))
                .expect("engine")
                .expect("positions")
        };
        for row in &report.grahas {
            assert_eq!(row.vargas.len(), 16, "{id}");
            for (v, varga_chart) in row.vargas.iter().zip(&chart.vargas) {
                // Varga burcu, v1'in on altı varga haritasındaki burçtur.
                assert_eq!(v.varga, varga_chart.varga, "{id}");
                let placed = &varga_chart.placements[row.graha as usize];
                assert_eq!(
                    (placed.graha, placed.sign),
                    (row.graha, v.sign),
                    "{id} {:?}",
                    v.varga
                );
                assert_eq!(
                    v.points == 20.0,
                    own_signs(row.graha).contains(&v.sign),
                    "{id} {:?} {:?}",
                    row.graha,
                    v.varga
                );
            }
            // Çapraz denetim: Shadbala'nın saptavargaja kalemi aynı ilişkiyi başka puanlarla
            // okur (kendi burcu 30, çok dost 20, dost 15, nötr 10, düşman 4, çok düşman 2;
            // D1'de moolatrikona 45). Yedi vargada basamaklar birebir eşleşmelidir.
            for varga in SAPTAVARGA {
                let v = row.vargas.iter().find(|v| v.varga == varga).expect("varga");
                let sapta = saptavargaja_points(row.graha, varga, &positions);
                // Moolatrikona (45; yalnızca D1) Vimshopaka'da ayrı basamak değildir: eşlenmez.
                let expected = [
                    (30.0, 20.0),
                    (20.0, 18.0),
                    (15.0, 15.0),
                    (10.0, 10.0),
                    (4.0, 7.0),
                    (2.0, 5.0),
                ]
                .into_iter()
                .find(|(points, _)| *points == sapta)
                .map_or(v.points, |(_, rung)| rung);
                assert_eq!(v.points, expected, "{id} {:?} {varga:?}", row.graha);
            }
            let scores = [
                row.shadvarga,
                row.saptavarga,
                row.dashavarga,
                row.shodashavarga,
            ];
            for (scheme, score) in VargaScheme::ALL.into_iter().zip(scores) {
                let again: f64 = scheme
                    .weights()
                    .iter()
                    .map(|&(varga, w)| {
                        w * row
                            .vargas
                            .iter()
                            .find(|v| v.varga == varga)
                            .expect("varga")
                            .points
                    })
                    .sum::<f64>()
                    / 20.0;
                assert!(
                    (score - again).abs() < 1e-9,
                    "{id} {:?} {scheme:?}",
                    row.graha
                );
                assert!(
                    (5.0..=20.0).contains(&score),
                    "{id} {:?} {scheme:?}",
                    row.graha
                );
            }
            assert_eq!(row.vargas[0].varga, Varga::D1, "{id}");
            assert_eq!(
                row.vargas[0].sign, chart.grahas[row.graha as usize].sign,
                "{id}"
            );
        }
        let unknown =
            compute_vimshopaka_blocking(engine, unknown_time(input)).expect("time unknown");
        assert!(unknown.approximate, "{id}");
        assert_eq!(unknown.grahas, report.grahas, "{id}");
    }
}

fn error_and_budget_checks(engine: &Engine, cases: &[(String, ChartInput)]) {
    let (_, delhi) = cases
        .iter()
        .find(|(id, _)| id == "new-delhi-2000-01-01")
        .expect("delhi");
    // Hata eşlemesi: geçersiz konum haritanın hatasıdır; geçiş anı yalnızca alan adıyla
    // reddedilir (değer taşımaz) ve haritadan önce denetlenir.
    let mut bad = delhi.clone();
    bad.lat = 91.0;
    let e = compute_panchang_extras_blocking(engine, bad.clone()).expect_err("bad latitude");
    assert_eq!(e.code, "invalid_location");
    let e = compute_vimshopaka_blocking(engine, bad.clone()).expect_err("bad latitude");
    assert_eq!(e.code, "invalid_location");
    let e = compute_navatara_blocking(engine, bad.clone(), None).expect_err("bad latitude");
    assert_eq!(e.code, "invalid_location");
    for jd in [f64::NAN, f64::INFINITY, 0.0, 1e9] {
        let e = compute_navatara_blocking(engine, bad.clone(), Some(jd))
            .expect_err("bad transit moment");
        assert_eq!(e.code, "invalid_transit_input");
        let params: Vec<(&str, &str)> = e
            .params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(params, [("field", "transit_jd_ut")]);
    }

    // Bütçeler (Plan 23 D17): temel harita hesabı dahil komut başına ≤ 20 ms, hata ayıklama
    // derlemesinde. Boş makinede ölçüm 1 ms'nin altındadır; pay, yan yana derleyen çalışma
    // ağaçlarının yükü içindir.
    let budget = Duration::from_millis(20);
    let (median, max) = median_of_15(|| {
        compute_panchang_extras_blocking(engine, delhi.clone()).expect("extras");
    });
    eprintln!("compute_panchang_extras: median {median:?}, max {max:?}");
    assert!(
        median <= budget,
        "compute_panchang_extras median {median:?}"
    );
    let (median, max) = median_of_15(|| {
        compute_navatara_blocking(engine, delhi.clone(), Some(delhi.jd_ut + 9000.0))
            .expect("navatara");
    });
    eprintln!("compute_navatara: median {median:?}, max {max:?}");
    assert!(median <= budget, "compute_navatara median {median:?}");
    let (median, max) = median_of_15(|| {
        compute_vimshopaka_blocking(engine, delhi.clone()).expect("vimshopaka");
    });
    eprintln!("compute_vimshopaka: median {median:?}, max {max:?}");
    assert!(median <= budget, "compute_vimshopaka median {median:?}");
}
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

```bash
cargo test -p jyotish-core --lib natal_extras::collect 2>&1 | grep -E "^error" | sort | uniq -c | head
cargo test -p jyotish-desk --test natal_extras 2>&1 | grep -E "^error" | sort | uniq -c | head
```
Expected: iki komut da aynı derleme hatasını verir (`error[E0432]: unresolved imports`: `collect::compute_navatara`, `collect::compute_panchang_extras`, `collect::compute_vimshopaka`); uygulama testi çekirdek derlenmediği için kurulamaz (`error: could not compile`), çekirdek derlendiğinde de `natal_extras_commands` dosyası henüz yoktur. Test koşmaz.

- [ ] **Step 3: Toplayıcılar ve komutlar**

**`crates/jyotish-core/src/natal_extras/collect.rs`: uygulama, test bloğunun üstüne (arada bir boş satır)**

````rust
//! Toplayıcılar (K3): temel haritayı hesaplar, konumlara indirger ve saf fonksiyonlara verir.
//! Ayar, an ve konum denetimi `compute_chart`'ınkidir. Üç rapor da lagna istemez: doğum saati
//! bilinmiyorsa (`input.time_known` yanlış) rapor yine döner ve `approximate` işaretini taşır.

use ephemeris::Ephemeris;

use crate::chart::compute_chart_for;
use crate::graha::Graha;
use crate::positions::{compute_positions, ChartInput, Positions};
use crate::settings::JD_UT_RANGE;
use crate::transit::compute_transits;
use crate::CoreError;

use super::dagdha::{panchang_extras, PanchangExtras};
use super::navatara::{navatara, Navatara};
use super::vimshopaka::{vimshopaka, VimshopakaReport};

/// `compute_panchang_extras` komutunun hesabı.
pub fn compute_panchang_extras<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<PanchangExtras, CoreError> {
    let positions = compute_positions(eph, input)?;
    Ok(panchang_extras(&positions, !input.time_known))
}

/// `compute_navatara` komutunun hesabı. `transit_jd_ut` verilirse o anın dokuz grahası haritanın
/// ayarlarıyla hesaplanır ve taraları rapora eklenir. Geçiş anı, ephemeris sorulmadan önce
/// denetlenir; hatası yalnızca alan adını taşır.
pub fn compute_navatara<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
    transit_jd_ut: Option<f64>,
) -> Result<Navatara, CoreError> {
    if transit_jd_ut.is_some_and(|jd| !JD_UT_RANGE.contains(&jd)) {
        return Err(CoreError::InvalidTransitInput("transit_jd_ut"));
    }
    let chart = compute_chart_for(eph, input)?;
    let positions = Positions::from(&chart);
    let transit: Option<(f64, Vec<(Graha, f64)>)> = match transit_jd_ut {
        Some(jd) => {
            let snapshot = compute_transits(eph, jd, &input.settings, &chart.bhavas)?;
            let lons = snapshot
                .grahas
                .iter()
                .map(|g| (g.graha, g.sidereal_lon))
                .collect();
            Some((jd, lons))
        }
        None => None,
    };
    Ok(navatara(
        &positions,
        transit.as_ref().map(|(jd, lons)| (*jd, lons.as_slice())),
        !input.time_known,
    ))
}

/// `compute_vimshopaka` komutunun hesabı.
pub fn compute_vimshopaka<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<VimshopakaReport, CoreError> {
    let positions = compute_positions(eph, input)?;
    Ok(vimshopaka(&positions, !input.time_known))
}
````

**Yeni dosya:** `app/src-tauri/src/natal_extras_commands.rs`

````rust
//! Doğum anı eklerinin komutları (yol haritası Plan 23): panchang ekleri, navatara ve
//! Vimshopaka bala. Kalıp `module_commands` ile aynıdır: tek `ChartInput`, hesap motor iş
//! parçacığında, komut gövdesi `blocking` havuzunda, hata `CommandError`. Sonuçlar `Chart`'a
//! eklenmez ve saklanmaz; panel açıkken istenir (K1). Girdiler doğum verisidir: loglanmaz.

use std::sync::Arc;

use ephemeris::{Engine, SwissEph};
use jyotish_core::{ChartInput, Navatara, PanchangExtras, VimshopakaReport};
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::CommandError;

/// `compute_panchang_extras` komutunun gövdesi; bloklar.
pub fn compute_panchang_extras_blocking(
    engine: &Engine,
    input: ChartInput,
) -> Result<PanchangExtras, CommandError> {
    let extras = engine
        .call(move |se: &mut SwissEph| jyotish_core::compute_panchang_extras(se, &input))??;
    Ok(extras)
}

/// Panchang ekleri: yogi noktası, yogi, eş yogi, avayogi ve doğum tithisinin dagdha burçları.
/// Lagna gerekmez; doğum saati bilinmiyorsa (`time_known == false`) değerler öğlen
/// konumlarındandır ve sonuç `approximate` işaretlidir.
#[tauri::command]
#[specta::specta]
pub async fn compute_panchang_extras(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
) -> Result<PanchangExtras, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_panchang_extras_blocking(&engine, input)).await
}

/// `compute_navatara` komutunun gövdesi; bloklar.
pub fn compute_navatara_blocking(
    engine: &Engine,
    input: ChartInput,
    transit_jd_ut: Option<f64>,
) -> Result<Navatara, CommandError> {
    let table = engine.call(move |se: &mut SwissEph| {
        jyotish_core::compute_navatara(se, &input, transit_jd_ut)
    })??;
    Ok(table)
}

/// Navatara: doğum Ay'ının nakshatrasından başlayan 27 satırlık tara tablosu (nakshatra, tara,
/// tur, lord, o nakshatradaki doğum grahaları). `transit_jd_ut` verilirse (transit anı:
/// `TransitView.snapshot.jd_ut`) o andaki dokuz grahanın tarası da döner; arayüz an hesabı
/// yapmaz. Doğum saati bilinmiyorsa sonuç `approximate` işaretlidir.
#[tauri::command]
#[specta::specta]
pub async fn compute_navatara(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
    transit_jd_ut: Option<f64>,
) -> Result<Navatara, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_navatara_blocking(&engine, input, transit_jd_ut)).await
}

/// `compute_vimshopaka` komutunun gövdesi; bloklar.
pub fn compute_vimshopaka_blocking(
    engine: &Engine,
    input: ChartInput,
) -> Result<VimshopakaReport, CommandError> {
    let report =
        engine.call(move |se: &mut SwissEph| jyotish_core::compute_vimshopaka(se, &input))??;
    Ok(report)
}

/// Vimshopaka bala: yedi grahanın on altı vargadaki burcu ve basamağı ile dört kümedeki
/// (shadvarga, saptavarga, dashavarga, shodashavarga) yirmi üzerinden değeri. Lagna gerekmez;
/// doğum saati bilinmiyorsa sonuç `approximate` işaretlidir.
#[tauri::command]
#[specta::specta]
pub async fn compute_vimshopaka(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
) -> Result<VimshopakaReport, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_vimshopaka_blocking(&engine, input)).await
}
````

- [ ] **Step 4: Rust testleri ve ölçümler**

```bash
cargo fmt --all
cargo test -p jyotish-core --lib natal_extras 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --test natal_extras -- --nocapture 2>&1 | grep -E "^compute_|^test result"
```
Expected: `25 passed`; lib `429 passed`, golden `8 passed`, `modules_golden` `17 passed`; üç ölçüm satırı (`compute_panchang_extras`, `compute_navatara`, `compute_vimshopaka`; bu plan yazılırken boş makinede ortancalar 0,57 ms, 0,63 ms ve 0,61 ms'ydi; bütçe 20 ms: D17) ve `1 passed`. Bağlama testi (`bindings`) bu adımda henüz düşer: Step 6 onu günceller. Bir çapraz denetim tutmazsa önce hangi tarafın (bu plan mı, v1 mi) kuralı yanlış okuduğu aranır; denetim gevşetilmez.

- [ ] **Step 5: Fikstür: `natal_extras`**

````diff
diff --git a/app/src-tauri/tests/ui_fixture.rs b/app/src-tauri/tests/ui_fixture.rs
index 9e77aa0..6ed14ce 100644
--- a/app/src-tauri/tests/ui_fixture.rs
+++ b/app/src-tauri/tests/ui_fixture.rs
@@ -29,6 +29,11 @@ use jyotish_desk_lib::module_commands::{
 use jyotish_desk_lib::muhurta_commands::{
     day_panchang_blocking, muhurta_search_blocking, MuhurtaPlace, MuhurtaQuery,
 };
+// Ayrı satır: ortak `use jyotish_core::{…}` listesi yeniden sarılmasın (eşzamanlı planlar).
+use jyotish_core::VargaScheme;
+use jyotish_desk_lib::natal_extras_commands::{
+    compute_navatara_blocking, compute_panchang_extras_blocking, compute_vimshopaka_blocking,
+};
 use jyotish_desk_lib::prashna_commands::{
     compute_prashna_facts_blocking, create_prashna_blocking, PrashnaMoment, PrashnaRequest,
 };
@@ -174,6 +179,15 @@ fn ui_fixture_is_up_to_date() {
         chara_at.cycle,
     )
     .expect("chara dasha children at the transit moment");
+    // Doğum anı ekleri (Plan 23): panchang ekleri, navatara (fikstürdeki transit anının
+    // taralarıyla) ve Vimshopaka bala; aynı girdiyle.
+    let panchang_extras =
+        compute_panchang_extras_blocking(&engine, module_input.clone()).expect("panchang extras");
+    let navatara =
+        compute_navatara_blocking(&engine, module_input.clone(), Some(transit.snapshot.jd_ut))
+            .expect("navatara");
+    let vimshopaka =
+        compute_vimshopaka_blocking(&engine, module_input.clone()).expect("vimshopaka");
     // Shadbala ve Bhava bala: aynı girdiyle (doğum saati biliniyor, Güneş doğup batıyor).
     let shadbala = compute_shadbala_blocking(&engine, module_input.clone())
         .expect("shadbala")
@@ -481,6 +495,12 @@ fn ui_fixture_is_up_to_date() {
         "chara_dasha_at": chara_at,
         "chara_dasha_at_children": chara_at_children,
         "shadbala": shadbala,
+        "natal_extras": {
+            "panchang_extras": panchang_extras,
+            "navatara": navatara,
+            "vimshopaka": vimshopaka,
+            "schemes": VargaScheme::ALL,
+        },
         "yogas": yogas,
         "yogas_time_unknown": yogas_time_unknown,
         "yoga_catalog": yoga_catalog,
````

- [ ] **Step 6: Bağlamaları ve fikstürü yeniden üret (C10)**

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --test bindings --test ui_fixture 2>&1 | grep -E "^test result"
git diff --stat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
grep -c "compute_panchang_extras\|compute_navatara\|compute_vimshopaka" app/ui/bindings.ts
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
node -e 'const f=require("./app/ui/test/fixtures/delhi.json").natal_extras; console.log(f.panchang_extras.yogi.yogi, f.panchang_extras.tithi, f.panchang_extras.dagdha_rasis.join("+"), f.navatara.janma_nakshatra, f.navatara.rows.length, f.navatara.transit.length, f.vimshopaka.grahas.length, f.schemes.join("+"))'
```
Expected: üç `ok` satırı (`1 passed`, `2 passed`, sonra `1 passed` ve `2 passed`); `bindings.ts` 127 satır kazanır, `delhi.json` tek satırdır ve değişir; komut adları sayacı `6` yazar (üç komut satırı ve üç belge yorumu); yasaklı ad sayacı `0`; son komut `Rahu 26 Sagittarius+Pisces 15 27 9 7 Shadvarga+Saptavarga+Dashavarga+Shodashavarga` yazar (Delhi: yogi Rahu, Krishna Ekadashi, Yay ve Balık; doğum nakshatrası Swati; 27 satır, dokuz geçiş grahası, yedi graha, dört küme). İki dosya elle düzenlenmez.

- [ ] **Step 7: Sarmalayıcılar ve fikstür tipi**

````diff
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index abf51e5..1aa314f 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -34,6 +34,9 @@ vi.mock("./bindings", () => ({
     charaDashaChildren: vi.fn(),
     charaDashaAt: vi.fn(),
     computeShadbala: vi.fn(),
+    computePanchangExtras: vi.fn(),
+    computeNavatara: vi.fn(),
+    computeVimshopaka: vi.fn(),
     computeYogas: vi.fn(),
     computeCompat: vi.fn(),
     chartInput: vi.fn(),
@@ -138,6 +141,9 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["charaDashaChildren", [{ jd_ut: 1, time_known: true }, ["Aries", "Taurus"], 2]],
     ["charaDashaAt", [{ jd_ut: 1, time_known: true }, 2451545.5]],
     ["computeShadbala", [{ jd_ut: 1, time_known: true }]],
+    ["computePanchangExtras", [{ jd_ut: 1, time_known: false }]],
+    ["computeNavatara", [{ jd_ut: 1, time_known: true }, 2461369.5]],
+    ["computeVimshopaka", [{ jd_ut: 1, time_known: true }]],
     ["computeYogas", [{ jd_ut: 1, time_known: false }]],
     ["computeCompat", [{ jd_ut: 1, time_known: true }, { jd_ut: 2, time_known: false }]],
     ["chartInput", ["00000000-0000-4000-8000-000000000012"]],
diff --git a/app/ui/api.ts b/app/ui/api.ts
index b3b76a1..18beee0 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -101,6 +101,12 @@ export const api = {
   // Shadbala ve Bhava bala: `null` doğum saati bilinmeyen ya da o gün Güneş'in doğmadığı ya da
   // batmadığı haritadır.
   computeShadbala: (input: ChartInput) => wrap(() => commands.computeShadbala(input)),
+  // Doğum anı ekleri (Plan 23): üçü de lagna istemez; doğum saati bilinmiyorsa sonuç
+  // `approximate` işaretlidir. `transitJdUt` transit anıdır (`TransitView.snapshot.jd_ut`);
+  // `null` ise navatara yalnızca doğum tablosunu döner.
+  computePanchangExtras: (input: ChartInput) => wrap(() => commands.computePanchangExtras(input)),
+  computeNavatara: (input: ChartInput, transitJdUt: number | null) => wrap(() => commands.computeNavatara(input, transitJdUt)),
+  computeVimshopaka: (input: ChartInput) => wrap(() => commands.computeVimshopaka(input)),
   // Yogalar: doğum saati bilinmiyorsa rapor yine döner; lagna isteyen kurallar atlanmış,
   // diğer kayıtlar `approximate` işaretlidir.
   computeYogas: (input: ChartInput) => wrap(() => commands.computeYogas(input)),
diff --git a/app/ui/test/fixture.ts b/app/ui/test/fixture.ts
index 62b0474..8aa4912 100644
--- a/app/ui/test/fixture.ts
+++ b/app/ui/test/fixture.ts
@@ -31,7 +31,9 @@ import type {
   MuhurtaSearchView,
   NakshatraNature,
   NatalMoon,
+  Navatara,
   OpenedClient,
+  PanchangExtras,
   PlaceHit,
   PrashnaFacts,
   PrefsLoad,
@@ -45,6 +47,7 @@ import type {
   TithiGroup,
   TransitView,
   Varga,
+  VimshopakaReport,
   YogaGroup,
   YogaReport,
 } from "../bindings";
@@ -74,6 +77,12 @@ export interface UiFixture {
   chara_dasha_at_children: SignDashaPeriod[];
   /** `compute_shadbala`, aynı girdiyle: yedi graha, zaman lordları, on iki bhava. */
   shadbala: ShadbalaReport;
+  /**
+   * Doğum anı ekleri (Plan 23), aynı girdiyle: `compute_panchang_extras`, `compute_navatara`
+   * (fikstürdeki transit anıyla: `transit.snapshot.jd_ut`) ve `compute_vimshopaka` dönüşleri;
+   * `schemes` Rust `VargaScheme::ALL`.
+   */
+  natal_extras: { panchang_extras: PanchangExtras; navatara: Navatara; vimshopaka: VimshopakaReport; schemes: string[] };
   /** `compute_yogas`, aynı girdiyle: 41 kural değerlendirilir. */
   yogas: YogaReport;
   /** `compute_yogas`, `time_known: false` ile: lagna isteyen 17 kural atlanır, kayıtlar yaklaşıktır. */
````

- [ ] **Step 8: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
cargo test -p jyotish-desk --lib 2>&1 | grep -E "^test result"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
git diff --stat -- app/src-tauri/src/commands.rs app/src-tauri/src/module_commands.rs app/src-tauri/tests/modules.rs crates/jyotish-core/src/error.rs
```
Expected: fmt ve clippy çıktısız; sayaç `0`; `jyotish-desk` lib `58 passed`; Vitest `88 passed (88)`, `887 passed (887)` (sayı değişmez: üç sarmalayıcı `api.test.ts`'in var olan tablosundadır); `✓ built`; son komut çıktısız (hata eşlemesi, var olan komutlar ve var olan entegrasyon testi değişmedi).

- [ ] **Step 9: Commit**

```bash
git add crates/jyotish-core/src/natal_extras crates/jyotish-core/src/lib.rs app/src-tauri/src/natal_extras_commands.rs app/src-tauri/src/lib.rs app/src-tauri/tests/natal_extras.rs app/src-tauri/tests/ui_fixture.rs app/ui/bindings.ts app/ui/test/fixtures/delhi.json app/ui/test/fixture.ts app/ui/api.ts app/ui/api.test.ts
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(app): compute_panchang_extras, compute_navatara and compute_vimshopaka commands; cross-checks against the chart and the saptavargaja item on the real engine; bindings and UI fixture" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: on iki dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 6: Arayüz: Panchang paneline ek tablo ve Navatara paneli

Yol haritası görev taslağının (6). maddesi; K1, K12, Y4; C4, C9, C12; D6, D7, D9–D11, D21. Panchang paneli beş uzvu bugünkü gibi komut beklemeden çizer; altına `PanchangExtrasSection` eklenir ve kendi komutunu çağırır. Navatara paneli transit deposunun anını okur ve Rust'a olduğu gibi gönderir. İki bileşen de var olan stil sınıflarını kullanır; `panels.css` değişmez.

**Files:**
- Create: `app/ui/modules/useNatalExtras.ts`, `app/ui/panels/PanchangExtras.tsx`, `app/ui/panels/PanchangExtras.test.tsx`, `app/ui/panels/NavataraPanel.tsx`, `app/ui/panels/NavataraPanel.test.tsx`
- Modify: `app/ui/panels/PanchangPanel.tsx`, `app/ui/panels/PanchangPanel.test.tsx`, `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx`, `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`, `app/ui/App.test.tsx`, `app/e2e/ipc.ts`
- Test: `PanchangExtras.test.tsx` (7), `NavataraPanel.test.tsx` (8); `model.test.ts` ve `panels.test.tsx` (birer tür daha); `PanchangPanel.test.tsx` ve `App.test.tsx` (yalnızca `api` taklidi)

**Interfaces:**
- Consumes: Task 5 (`api.computePanchangExtras`, `api.computeNavatara`; `PanchangExtras`, `Navatara`, `NavataraRow` tipleri; fikstürde `natal_extras`); `modules/useModule.ts` (`useModule`, `useLastData`), `modules/input.ts` (`chartInput`, `inputKey`); `transit/store.tsx` (`useTransit`, `formatMoment`, `browserTimeZone`), `transit/TransitDate.tsx`; `panels/{Approx, PinButton, bala (BalaScroll), format (formatDegree), labels (nakshatraName, taraName), states}`; `chart/adapter` (`signIndex`).
- Produces:
  - `usePanchangExtras(opened: OpenedClient | null): ModuleState<PanchangExtras>`, `useNavatara(opened: OpenedClient | null, transitJdUt: number | null): ModuleState<Navatara>`.
  - `PanchangExtrasSection({ opened }: { opened: OpenedClient })`: tablo satırları `data-extra` taşır (`yogi_point`, `yogi`, `co_yogi`, `avayogi`, `dagdha`).
  - `NavataraPanel(_: PanelProps)`: doğum tablosunun satırları `data-tara` (1–9), geçiş tablosunun satırları `data-graha` taşır.
  - Panel türü `navatara` (`MODULE_PANELS`: 7 × 16); i18n `panel.navatara`, `panchangExtras.*`, `navatara.*` (üç dil).
  - E2e taklidi: `compute_panchang_extras`, `compute_navatara`.

- [ ] **Step 1: Testler: ek tablo, Navatara paneli ve katalog**

`PanchangExtras.test.tsx` paneli bütünüyle çizer (ek tablo panelin parçasıdır): Delhi'nin beş olgusu ve iki not; açık harita yokken istek yok; saat bilinmezliği (istek `time_known: false`, beş değer "≈"); dolunayda "yok"; ortak vurgu (graha ve burç); hata beş uzvu kaldırmaz ve yeniden denenir; üç dil ve Fransızca kesme işareti. `NavataraPanel.test.tsx`: dokuz satırın tamamı (türetme yorumda); geçiş tablosu ve Rust'a giden an; açık harita yok; saat bilinmezliği; ortak vurgu; yeni transit anında tabloların yerinde durması; rapor ve transit hataları; üç dil. Var olan iki test dosyası yalnızca `api` taklidi kazanır: Panchang paneli artık bir komut çağırır.

**Yeni dosya:** `app/ui/panels/PanchangExtras.test.tsx`

````tsx
import { fireEvent, screen, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computePanchangExtras: vi.fn() } }));

import { api, type Result } from "../api";
import type { PanchangExtras } from "../bindings";
import { setLanguage } from "../i18n";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { PanchangPanel } from "./PanchangPanel";

const panel: Panel = { id: "pa", kind: "panchang", x: 0, y: 0, w: 4, h: 9, settings: {} };
const ok = (extras: PanchangExtras = fixture().natal_extras.panchang_extras): Result<PanchangExtras> => ({ status: "ok", data: extras });
const extrasTable = () => screen.findByRole("table", { name: "Panchang ekleri" });
const cells = (key: string) => [...document.querySelector(`tr[data-extra="${key}"]`)!.children].map((c) => c.textContent);
const highlighted = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.extra);

beforeEach(() => {
  vi.mocked(api.computePanchangExtras).mockReset();
  vi.mocked(api.computePanchangExtras).mockImplementation(async () => ok());
});

test("five extra facts under the panchang: yogi point, yogi, co-yogi, avayogi and the dagdha signs", async () => {
  renderWithStores(<PanchangPanel panel={panel} />);
  const table = await extrasTable();
  const opened = fixture().opened;
  expect(api.computePanchangExtras).toHaveBeenCalledTimes(1);
  expect(api.computePanchangExtras).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null });
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Öğe", "Değer"]);
  // Delhi 2000-01-01: nitya yoga Dhriti (8) → yogi nakshatrası Swati (15), lordu Rahu; nokta
  // Terazi'de (lordu Venüs); altıncı nakshatra Purva Ashadha (20), lordu Venüs; Krishna
  // Ekadashi: Yay ve Balık.
  expect(cells("yogi_point")).toEqual(["Yogi noktası", "Ter 9°19′ · Swati · pada 1"]);
  expect(cells("yogi")).toEqual(["Yogi", "Rahu"]);
  expect(cells("co_yogi")).toEqual(["Eş yogi", "Venüs"]);
  expect(cells("avayogi")).toEqual(["Avayogi", "Venüs · Purva Ashadha"]);
  expect(cells("dagdha")).toEqual(["Dagdha rasi", "Yay, Bal"]);
  expect(screen.getByText("Doğrulanmamış tablo: dagdha rasi tablosu basılı kaynakla henüz karşılaştırılmadı.")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: panchang_extras.v1")).toBeInTheDocument();
  expect(document.querySelector('[title="yaklaşık"]')).toBeNull();
  // Beş uzvun tablosu yerindedir.
  expect(screen.getByRole("table", { name: /^Doğum anının panchangı/ })).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderWithStores(<PanchangPanel panel={panel} />, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computePanchangExtras).not.toHaveBeenCalled();
});

test("unknown birth time: the request says so and every extra value is marked approximate", async () => {
  const extras = fixture().natal_extras.panchang_extras;
  extras.approximate = true;
  vi.mocked(api.computePanchangExtras).mockImplementation(async () => ok(extras));
  renderWithStores(<PanchangPanel panel={panel} />, { opened: unknownTimeOpened() });
  await extrasTable();
  expect(vi.mocked(api.computePanchangExtras).mock.calls[0][0].time_known).toBe(false);
  for (const key of ["yogi_point", "yogi", "co_yogi", "avayogi", "dagdha"]) {
    expect(document.querySelector(`tr[data-extra="${key}"] [title="yaklaşık"]`), key).not.toBeNull();
  }
  expect(cells("yogi")[1]).toBe("≈yaklaşık Rahu");
});

test("full or new moon: no dagdha sign, said in words", async () => {
  const extras = fixture().natal_extras.panchang_extras;
  extras.tithi = 15;
  extras.dagdha_rasis = [];
  vi.mocked(api.computePanchangExtras).mockImplementation(async () => ok(extras));
  renderWithStores(<PanchangPanel panel={panel} />);
  await extrasTable();
  expect(cells("dagdha")).toEqual(["Dagdha rasi", "yok (dolunay ve yeni ayda dagdha rasi olmaz)"]);
});

test("shared highlight: a graha marks the rows it lords, a sign the rows that name it", async () => {
  renderWithStores(<PanchangPanel panel={panel} />);
  const table = await extrasTable();
  expect(highlighted()).toEqual([]);
  // Venüs eş yogi ve avayogidir.
  fireEvent.click(within(table).getAllByRole("button", { name: "Venüs" })[0]);
  expect(highlighted()).toEqual(["co_yogi", "avayogi"]);
  fireEvent.click(within(table).getByRole("button", { name: "Rahu" }));
  expect(highlighted()).toEqual(["yogi"]);
  fireEvent.click(within(table).getByRole("button", { name: "Ter" }));
  expect(highlighted()).toEqual(["yogi_point"]);
  fireEvent.click(within(table).getByRole("button", { name: "Bal" }));
  expect(highlighted()).toEqual(["dagdha"]);
  expect(within(table).getByRole("button", { name: "Bal" })).toHaveAttribute("aria-pressed", "true");
});

test("an error leaves the five limbs in place and is retried from the panel", async () => {
  vi.mocked(api.computePanchangExtras).mockImplementationOnce(async () => ({ status: "error", error: { code: "engine_stopped", params: {} } }));
  renderWithStores(<PanchangPanel panel={panel} />);
  const alert = await screen.findByRole("alert");
  expect(screen.getByRole("table", { name: /^Doğum anının panchangı/ })).toBeInTheDocument();
  expect(screen.queryByRole("table", { name: "Panchang ekleri" })).toBeNull();
  fireEvent.click(within(alert).getByRole("button", { name: "Yeniden dene" }));
  await extrasTable();
  expect(api.computePanchangExtras).toHaveBeenCalledTimes(2);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("labels in three languages; classical names stay; French uses the typographic apostrophe", async () => {
  await setLanguage("en");
  const first = renderWithStores(<PanchangPanel panel={panel} />);
  await screen.findByRole("table", { name: "Panchang additions" });
  expect(cells("yogi_point")).toEqual(["Yogi point", "Li 9°19′ · Swati · pada 1"]);
  expect(cells("co_yogi")).toEqual(["Duplicate yogi", "Venus"]);
  expect(cells("avayogi")).toEqual(["Avayogi", "Venus · Purva Ashadha"]);
  first.unmount();
  await setLanguage("fr");
  renderWithStores(<PanchangPanel panel={panel} />);
  await screen.findByRole("table", { name: "Compléments du panchang" });
  expect(cells("yogi")).toEqual(["Yogi", "Rahu"]);
  expect(cells("dagdha")[0]).toBe("Dagdha rasi");
  for (const text of [...Object.values(fr.panchangExtras), ...Object.values(fr.navatara)]) expect(text).not.toContain("'");
  expect(Object.keys(en.panchangExtras)).toEqual(Object.keys(fr.panchangExtras));
});
````

**Yeni dosya:** `app/ui/panels/NavataraPanel.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeNavatara: vi.fn(), transitsAt: vi.fn() } }));

import { api, type Result } from "../api";
import type { Navatara } from "../bindings";
import { setLanguage } from "../i18n";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { browserTimeZone, formatMoment } from "../transit/store";
import { TARA_NAMES } from "./labels";
import { NavataraPanel } from "./NavataraPanel";

const panel: Panel = { id: "nv", kind: "navatara", x: 0, y: 0, w: 7, h: 16, settings: {} };
const ok = (table: Navatara = fixture().natal_extras.navatara): Result<Navatara> => ({ status: "ok", data: table });
const NATAL = "Navatara: doğum nakshatrası Swati; dokuz tara, üç tur";
const natalTable = () => screen.findByRole("table", { name: NATAL });
const when = () => formatMoment(fixture().transit.utc, "tr", browserTimeZone());
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const bodyRows = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);

beforeEach(() => {
  vi.mocked(api.computeNavatara).mockReset();
  vi.mocked(api.computeNavatara).mockImplementation(async () => ok());
  vi.mocked(api.transitsAt).mockReset();
  vi.mocked(api.transitsAt).mockImplementation(async () => ({ status: "ok", data: fixture().transit }));
});

test("nine taras in three rounds from the birth nakshatra, with the natal grahas, the lord and the set", async () => {
  renderWithStores(<NavataraPanel panel={panel} />);
  const table = await natalTable();
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Tara", "1. tur", "2. tur", "3. tur", "Lord", "Küme"]);
  // Delhi 2000-01-01: Ay Swati'de (15). Türetme: tara n'nin üç nakshatrası 15 + (n − 1),
  // dokuz ve on sekiz sonrası (27'de sarılır); lord üçünde de aynıdır.
  expect(bodyRows(table)).toEqual([
    ["1 · Janma", "Swati (Ay)", "Shatabhisha", "Ardra", "Rahu", "elverişli kümede değil"],
    ["2 · Sampat", "Vishakha", "Purva Bhadrapada", "Punarvasu", "Jüpiter", "elverişli kümede"],
    ["3 · Vipat", "Anuradha (Ve)", "Uttara Bhadrapada", "Pushya (Ra)", "Satürn", "elverişli kümede değil"],
    ["4 · Kshema", "Jyeshtha", "Revati", "Ashlesha", "Merkür", "elverişli kümede"],
    ["5 · Pratyak", "Mula (Me)", "Ashwini (Jü)", "Magha", "Ketu", "elverişli kümede değil"],
    ["6 · Sadhana", "Purva Ashadha (Gü)", "Bharani (Sa)", "Purva Phalguni", "Venüs", "elverişli kümede"],
    ["7 · Naidhana", "Uttara Ashadha", "Krittika", "Uttara Phalguni", "Güneş", "elverişli kümede değil"],
    ["8 · Mitra", "Shravana (Ke)", "Rohini", "Hasta", "Ay", "elverişli kümede"],
    ["9 · Parama Mitra", "Dhanishta (Ma)", "Mrigashira", "Chitra", "Mars", "elverişli kümede"],
  ]);
  // Kısaltmayla yazılan graha tam adıyla okunur.
  expect(within(table).getAllByRole("button", { name: "Jüpiter" }).map((b) => b.textContent)).toEqual(["Jüpiter", "Jü"]);
  expect(screen.getByText(/Sayım doğum Ay'ının nakshatrasından başlar/)).toBeInTheDocument();
  expect(screen.getByText("Yöntem: navatara.moon_v1")).toBeInTheDocument();
  expect(screen.queryByText(/Doğum saati bilinmiyor/)).toBeNull();
});

test("the taras of the nine transit grahas at the transit moment, which is sent to Rust as it is", async () => {
  renderWithStores(<NavataraPanel panel={panel} />);
  const table = await screen.findByRole("table", { name: `Transit grahalarının tarası, ${when()}` });
  const opened = fixture().opened;
  const input = { jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null };
  // İlk istek transit anı gelmeden (yalnızca doğum tablosu), ikincisi anla yapılır.
  expect(vi.mocked(api.computeNavatara).mock.calls).toEqual([
    [input, null],
    [input, fixture().transit.snapshot.jd_ut],
  ]);
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Graha", "Nakshatra", "Tara", "Tur", "Küme"]);
  // Fikstürün transit anı (2026-11-26): Swati'den (15) sayım. Güneş Anuradha (17): 3 → Vipat;
  // Ay Rohini (4): 17 → 8, 2. tur; Mars, Jüpiter ve Ketu Magha (10): 23 → 5, 3. tur; Merkür
  // Vishakha (16): 2; Venüs Chitra (14): 27 → 9, 3. tur; Satürn Uttara Bhadrapada (26): 12 →
  // 3, 2. tur; Rahu Dhanishta (23): 9.
  expect(bodyRows(table)).toEqual([
    ["Güneş", "Anuradha · pada 2", "3 · Vipat", "1", "elverişli kümede değil"],
    ["Ay", "Rohini · pada 4", "8 · Mitra", "2", "elverişli kümede"],
    ["Mars", "Magha · pada 2", "5 · Pratyak", "3", "elverişli kümede değil"],
    ["Merkür", "Vishakha · pada 1", "2 · Sampat", "1", "elverişli kümede"],
    ["Jüpiter", "Magha · pada 1", "5 · Pratyak", "3", "elverişli kümede değil"],
    ["Venüs", "Chitra · pada 3", "9 · Parama Mitra", "3", "elverişli kümede"],
    ["Satürn", "Uttara Bhadrapada · pada 4", "3 · Vipat", "2", "elverişli kümede değil"],
    ["Rahu", "Dhanishta · pada 3", "9 · Parama Mitra", "1", "elverişli kümede"],
    ["Ketu", "Magha · pada 1", "5 · Pratyak", "3", "elverişli kümede değil"],
  ]);
  // Transit anı panelden de seçilir.
  expect(screen.getByRole("form", { name: "Transit anı" })).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderWithStores(<NavataraPanel panel={panel} />, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeNavatara).not.toHaveBeenCalled();
  expect(api.transitsAt).not.toHaveBeenCalled();
});

test("unknown birth time: the table is shown with a notice", async () => {
  const table = fixture().natal_extras.navatara;
  table.approximate = true;
  vi.mocked(api.computeNavatara).mockImplementation(async () => ok(table));
  renderWithStores(<NavataraPanel panel={panel} />, { opened: unknownTimeOpened() });
  await natalTable();
  expect(vi.mocked(api.computeNavatara).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByText("Doğum saati bilinmiyor: Ay öğlen 12:00 konumundadır; Ay o gün nakshatra değiştiriyorsa bütün tablo bir nakshatra kayar.")).toBeInTheDocument();
});

test("shared highlight: a graha marks the tara rows it sits in or lords, and its transit row", async () => {
  renderWithStores(<NavataraPanel panel={panel} />);
  const natal = await natalTable();
  const transit = await screen.findByRole("table", { name: /^Transit grahalarının tarası,/ });
  const marked = (table: HTMLElement) => [...table.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.tara ?? (r as HTMLElement).dataset.graha);
  expect(marked(natal)).toEqual([]);
  // Venüs Anuradha'dadır (3. tara) ve 6. taranın lordudur.
  fireEvent.click(within(natal).getAllByRole("button", { name: "Venüs" })[0]);
  expect(marked(natal)).toEqual(["3", "6"]);
  expect(marked(transit)).toEqual(["Venus"]);
  fireEvent.click(within(transit).getByRole("button", { name: "Rahu" }));
  expect(marked(natal)).toEqual(["1", "3"]);
  expect(marked(transit)).toEqual(["Rahu"]);
});

test("a new transit moment is read again while the tables stay in place", async () => {
  renderWithStores(<NavataraPanel panel={panel} />);
  await screen.findByRole("table", { name: /^Transit grahalarının tarası,/ });
  const later = fixture().transit;
  later.snapshot.jd_ut = (later.snapshot.jd_ut ?? 0) + 30;
  later.utc = "2026-12-26T01:30:00Z";
  vi.mocked(api.transitsAt).mockImplementation(async () => ({ status: "ok", data: later }));
  let resolve: (r: Result<Navatara>) => void = () => {};
  vi.mocked(api.computeNavatara).mockImplementation(() => new Promise((r) => (resolve = r)));
  fireEvent.change(screen.getByLabelText("Tarih"), { target: { value: "2026-12-26" } });
  fireEvent.change(screen.getByLabelText("Saat"), { target: { value: "04:30" } });
  fireEvent.click(screen.getByRole("button", { name: "Uygula" }));
  await waitFor(() => expect(vi.mocked(api.computeNavatara).mock.calls.at(-1)?.[1]).toBe(later.snapshot.jd_ut));
  // Yanıt beklenirken iki tablo da durur; geçiş tablosu eski anın taralarını gösterdiği için
  // başlığında an yazmaz.
  expect(screen.getByRole("table", { name: NATAL })).toBeInTheDocument();
  expect(screen.getByRole("table", { name: "Transit grahalarının tarası" })).toBeInTheDocument();
  const next = fixture().natal_extras.navatara;
  next.transit_jd_ut = later.snapshot.jd_ut;
  next.transit![0].nakshatra = 20;
  await act(async () => resolve(ok(next)));
  const table = await screen.findByRole("table", { name: `Transit grahalarının tarası, ${formatMoment(later.utc, "tr", browserTimeZone())}` });
  expect(bodyRows(table)[0][1]).toBe("Purva Ashadha · pada 2");
});

test("errors: the report error is retried; a transit error leaves the natal table", async () => {
  vi.mocked(api.computeNavatara).mockImplementationOnce(async () => ({ status: "error", error: { code: "engine_stopped", params: {} } }));
  vi.mocked(api.computeNavatara).mockImplementationOnce(async () => ({ status: "error", error: { code: "engine_stopped", params: {} } }));
  const first = renderWithStores(<NavataraPanel panel={panel} />);
  const alert = await screen.findByRole("alert");
  expect(screen.queryByRole("table")).toBeNull();
  fireEvent.click(within(alert).getByRole("button", { name: "Yeniden dene" }));
  await natalTable();
  expect(screen.queryByRole("alert")).toBeNull();
  first.unmount();

  vi.mocked(api.transitsAt).mockImplementation(async () => ({ status: "error", error: { code: "invalid_julian_day", params: { jd: "1" } } }));
  const natalOnly = fixture().natal_extras.navatara;
  natalOnly.transit = null;
  natalOnly.transit_jd_ut = null;
  vi.mocked(api.computeNavatara).mockClear();
  vi.mocked(api.computeNavatara).mockImplementation(async () => ok(natalOnly));
  renderWithStores(<NavataraPanel panel={panel} />);
  await natalTable();
  expect(await screen.findByRole("alert")).toBeInTheDocument();
  expect(screen.queryByRole("table", { name: /^Transit grahalarının tarası/ })).toBeNull();
  expect(vi.mocked(api.computeNavatara).mock.calls.every((c) => c[1] === null)).toBe(true);
});

test("tara names stay the same in every language; headers and the set are translated", async () => {
  expect(TARA_NAMES).toHaveLength(9);
  await setLanguage("en");
  const first = renderWithStores(<NavataraPanel panel={panel} />);
  const table = await screen.findByRole("table", { name: "Navatara: birth nakshatra Swati; nine taras in three rounds" });
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Tara", "Round 1", "Round 2", "Round 3", "Lord", "Set"]);
  expect(bodyRows(table)[0]).toEqual(["1 · Janma", "Swati (Mo)", "Shatabhisha", "Ardra", "Rahu", "not in the favourable set"]);
  first.unmount();
  await setLanguage("fr");
  renderWithStores(<NavataraPanel panel={panel} />);
  const french = await screen.findByRole("table", { name: "Navatara : nakshatra de naissance Swati ; neuf taras en trois tours" });
  expect(bodyRows(french)[1]).toEqual(["2 · Sampat", "Vishakha", "Purva Bhadrapada", "Punarvasu", "Jupiter", "dans l’ensemble favorable"]);
});
````

````diff
diff --git a/app/ui/App.test.tsx b/app/ui/App.test.tsx
index 842cdb3..b04bb47 100644
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -5,6 +5,7 @@ vi.mock("./api", () => ({
   api: {
     transitsAt: vi.fn(async () => ({ status: "ok", data: (await import("./test/fixture")).fixture().transit })),
     computeDrishti: vi.fn(async () => ({ status: "ok", data: (await import("./test/fixture")).fixture().drishti })),
+    computePanchangExtras: vi.fn(async () => ({ status: "ok", data: (await import("./test/fixture")).fixture().natal_extras.panchang_extras })),
     printWindow: vi.fn(async () => ({ status: "ok", data: null })),
     previewMoment: vi.fn(async () => ({ status: "error", error: { code: "invalid_input", params: { field: "birth.date" } } })),
     searchPlaces: vi.fn(async () => ({ status: "ok", data: [] })),
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index 08fa4b3..c467ec7 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -53,6 +53,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "muhurta_search",
     "prashna_facts",
     "event_search",
+    "navatara",
   ]);
   expect(isKnownPanelKind("chart")).toBe(true);
   expect(isKnownPanelKind("drishti")).toBe(true);
@@ -60,7 +61,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search", "navatara"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
diff --git a/app/ui/panels/PanchangPanel.test.tsx b/app/ui/panels/PanchangPanel.test.tsx
index 14ac132..e063ac6 100644
--- a/app/ui/panels/PanchangPanel.test.tsx
+++ b/app/ui/panels/PanchangPanel.test.tsx
@@ -1,5 +1,12 @@
 import { fireEvent, screen } from "@testing-library/react";
-import { expect, test } from "vitest";
+import { expect, test, vi } from "vitest";
+
+// Panelin ikinci tablosu (panchang ekleri) bir komuttan gelir; bu dosya beş uzvu sınar, ekleri
+// `PanchangExtras.test.tsx` sınar.
+vi.mock("../api", () => ({
+  api: { computePanchangExtras: vi.fn(async () => ({ status: "ok", data: (await import("../test/fixture")).fixture().natal_extras.panchang_extras })) },
+}));
+
 import { useHighlight } from "../highlight/store";
 import { setLanguage } from "../i18n";
 import type { Panel } from "../layout/model";
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index 36744d1..9f0f3f1 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -21,6 +21,7 @@ import { JaiminiKarakasPanel } from "./JaiminiKarakasPanel";
 import { MuhurtaDayPanel } from "./MuhurtaDayPanel";
 import { EventSearchPanel } from "./EventSearchPanel";
 import { MuhurtaSearchPanel } from "./MuhurtaSearchPanel";
+import { NavataraPanel } from "./NavataraPanel";
 import { PanchangPanel } from "./PanchangPanel";
 import { panelDefinition, PANEL_REGISTRY } from "./registry";
 import { ShadbalaPanel } from "./ShadbalaPanel";
@@ -60,6 +61,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.muhurta_day.component).toBe(MuhurtaDayPanel);
   expect(PANEL_REGISTRY.muhurta_search.component).toBe(MuhurtaSearchPanel);
   expect(PANEL_REGISTRY.event_search.component).toBe(EventSearchPanel);
+  expect(PANEL_REGISTRY.navatara.component).toBe(NavataraPanel);
   // Kataloğun her türü uygulanmıştır: yer tutucu yoktur.
   for (const kind of PANEL_KINDS) expect(PANEL_REGISTRY[kind].component, kind).not.toBe(UnavailablePanel);
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
````

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/panels/PanchangExtras.test.tsx ui/panels/PanchangPanel.test.tsx ui/panels/NavataraPanel.test.tsx ui/layout/model.test.ts ui/panels/panels.test.tsx 2>&1 | grep -E "Test Files|Tests |Failed to resolve" | sort -u`
Expected: `Test Files  4 failed | 1 passed (5)`; `NavataraPanel.test.tsx` ve `panels.test.tsx` yüklenmez (`Failed to resolve import "./NavataraPanel"`); `PanchangExtras.test.tsx`'in altı testi ("Panchang ekleri" tablosu yok) ve `model.test.ts`'in iki testi düşer; `PanchangPanel.test.tsx` geçer.

- [ ] **Step 3: Kancalar, ek tablo ve Navatara paneli**

**Yeni dosya:** `app/ui/modules/useNatalExtras.ts`

````ts
// Doğum anı eklerinin (Plan 23) kancaları: `compute_panchang_extras` ve `compute_navatara`.
// Sonuç saklanmaz, panel açıkken istenir (K1); arayüzde hesap yoktur (C4). İkisi de lagna
// istemez: doğum saati bilinmiyorsa sonuç yine gelir ve `approximate` işaretlidir.
import { api } from "../api";
import type { Navatara, OpenedClient, PanchangExtras } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

export function usePanchangExtras(opened: OpenedClient | null): ModuleState<PanchangExtras> {
  return useModule(opened !== null ? inputKey(opened) : null, () => api.computePanchangExtras(chartInput(opened as OpenedClient)));
}

/**
 * `transitJdUt` transit anıdır (`TransitView.snapshot.jd_ut`); `null` ise yalnızca doğum tablosu
 * istenir. An değişince rapor yeniden istenir; arayüz an hesabı yapmaz.
 */
export function useNavatara(opened: OpenedClient | null, transitJdUt: number | null): ModuleState<Navatara> {
  return useModule(opened !== null ? JSON.stringify([inputKey(opened), transitJdUt]) : null, () => api.computeNavatara(chartInput(opened as OpenedClient), transitJdUt));
}
````

**Yeni dosya:** `app/ui/panels/PanchangExtras.tsx`

````tsx
// Panchang ekleri (yol haritası §4.1): yogi noktası, yogi, eş yogi, avayogi ve dagdha rasi.
// Panchang panelinin ikinci tablosudur. Değerler `compute_panchang_extras`'tan gelir ve panel
// açıkken istenir (K1); burada hesap yoktur (C4). Doğum saati bilinmiyorsa Rust `approximate`
// döner ve değerler "≈" ile işaretlenir. Dagdha rasi tablosu sahibinin onayını bekler (Y4):
// tablo "doğrulanmamış tablo" notunu taşır.
import { useTranslation } from "react-i18next";
import type { OpenedClient, PanchangExtras } from "../bindings";
import { signIndex } from "../chart/adapter";
import type { GrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { useLastData } from "../modules/useModule";
import { usePanchangExtras } from "../modules/useNatalExtras";
import { Approx } from "./Approx";
import { formatDegree } from "./format";
import { nakshatraName } from "./labels";
import { PinButton } from "./PinButton";
import { PanelError, usePanelLoadPhase } from "./states";

export function PanchangExtrasSection({ opened }: { opened: OpenedClient }) {
  const module = usePanchangExtras(opened);
  // Aynı haritanın yeniden okunmasında (ayar değişimi) tablo yerinde durur.
  const extras = useLastData(module, String(opened.chart_record.id));
  usePanelLoadPhase(module.loading);
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (extras === null) return null;
  return <ExtrasTable extras={extras} />;
}

function ExtrasTable({ extras }: { extras: PanchangExtras }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const { yogi } = extras;
  const pointSign = signIndex(yogi.sign);
  const dagdha = extras.dagdha_rasis.map(signIndex);
  const onGraha = (g: GrahaId) => active?.type === "graha" && active.id === g;
  const onSign = (signs: readonly number[]) => active?.type === "sign" && signs.includes(active.sign);
  const row = (on: boolean | undefined) => (on ? "is-highlighted" : undefined);
  const approx = { on: extras.approximate, label: t("table.approx") };
  const graha = (g: GrahaId) => <PinButton target={{ type: "graha", id: g }}>{t(`grahaName.${g}`)}</PinButton>;
  const sign = (s: number) => <PinButton target={{ type: "sign", sign: s }}>{t(`sign.${s}`)}</PinButton>;

  return (
    <>
      <table className="module-table">
        <caption>{t("panchangExtras.caption")}</caption>
        <thead>
          <tr>
            <th scope="col">{t("panchang.item")}</th>
            <th scope="col">{t("panchang.value")}</th>
          </tr>
        </thead>
        <tbody>
          <tr data-extra="yogi_point" className={row(onSign([pointSign]))}>
            <th scope="row">{t("panchangExtras.yogiPoint")}</th>
            <td>
              <Approx {...approx}>
                {sign(pointSign)} {yogi.deg_in_sign === null ? "—" : formatDegree(yogi.deg_in_sign)} · {nakshatraName(yogi.nakshatra)} · {t("panchang.pada", { pada: yogi.pada })}
              </Approx>
            </td>
          </tr>
          <tr data-extra="yogi" className={row(onGraha(yogi.yogi))}>
            <th scope="row">{t("panchangExtras.yogi")}</th>
            <td>
              <Approx {...approx}>{graha(yogi.yogi)}</Approx>
            </td>
          </tr>
          <tr data-extra="co_yogi" className={row(onGraha(yogi.co_yogi))}>
            <th scope="row">{t("panchangExtras.coYogi")}</th>
            <td>
              <Approx {...approx}>{graha(yogi.co_yogi)}</Approx>
            </td>
          </tr>
          <tr data-extra="avayogi" className={row(onGraha(yogi.avayogi))}>
            <th scope="row">{t("panchangExtras.avayogi")}</th>
            <td>
              <Approx {...approx}>
                {graha(yogi.avayogi)} · {nakshatraName(yogi.avayogi_nakshatra)}
              </Approx>
            </td>
          </tr>
          <tr data-extra="dagdha" className={row(onSign(dagdha))}>
            <th scope="row">{t("panchangExtras.dagdha")}</th>
            <td>
              <Approx {...approx}>
                {dagdha.length === 0
                  ? t("panchangExtras.none")
                  : dagdha.map((s, i) => (
                      <span key={s}>
                        {i > 0 && ", "}
                        {sign(s)}
                      </span>
                    ))}
              </Approx>
            </td>
          </tr>
        </tbody>
      </table>
      <p className="panel-message">{t("panchangExtras.unverified")}</p>
      <p className="panel-message">{t("panchangExtras.method", { method: extras.method })}</p>
    </>
  );
}
````

````diff
diff --git a/app/ui/panels/PanchangPanel.tsx b/app/ui/panels/PanchangPanel.tsx
index afc12dd..90a7b47 100644
--- a/app/ui/panels/PanchangPanel.tsx
+++ b/app/ui/panels/PanchangPanel.tsx
@@ -1,5 +1,6 @@
 // Panchang paneli (ileri modüller spec'i §4): doğum anının beş uzvu, bitiş saatleri, gün doğumu
-// ve batımı. Değerler `Chart.panchang`'tan hazır gelir (v1'den beri hesaplanır); komut çağırmaz.
+// ve batımı. Değerler `Chart.panchang`'tan hazır gelir (v1'den beri hesaplanır). Altındaki
+// panchang ekleri (yogi, avayogi, dagdha rasi; Plan 23) ayrı bir komuttan gelir: `PanchangExtras`.
 // Saatler doğumun çözülmüş ofsetinde gösterilir. Ortak vurgu: Ay vurgulanınca Ay'a bağlı
 // satırlar, Güneş vurgulanınca Güneş'e bağlı satırlar `--select` zemin alır.
 import type { TFunction } from "i18next";
@@ -13,6 +14,7 @@ import { useSession } from "../session/store";
 import { Approx } from "./Approx";
 import { formatJdMinute } from "./format";
 import { nakshatraName, nityaYogaName, tithiInPaksha, tithiName, VARA_NAMES } from "./labels";
+import { PanchangExtrasSection } from "./PanchangExtras";
 import "./panels.css";
 import type { PanelProps } from "./types";
 
@@ -107,6 +109,7 @@ export function PanchangPanel(_: PanelProps) {
           <strong>{t("badge.SunriseUnavailable")}</strong>: {t("badgeHint.SunriseUnavailable")}
         </p>
       )}
+      <PanchangExtrasSection opened={opened} />
     </div>
   );
 }
````

**Yeni dosya:** `app/ui/panels/NavataraPanel.tsx`

````tsx
// Navatara paneli (yol haritası §4.2): doğum Ay'ının nakshatrasından sayılan dokuz tara, üç
// turda 27 nakshatra; her nakshatrada duran doğum grahaları ve nakshatranın lordu. Altında
// transit anındaki dokuz grahanın tarası. Değerler `compute_navatara`'dan gelir ve panel
// açıkken istenir (K1); transit anı transit deposunun anıdır ve Rust'a olduğu gibi gönderilir.
// Burada hesap yoktur (C4): satırları taraya göre üç sütuna dizmek gösterimdir. Elverişli küme
// sözle yazılır; renk anlam taşımaz (C9).
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { Navatara, NavataraRow, OpenedClient } from "../bindings";
import type { GrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { inputKey } from "../modules/input";
import { useLastData } from "../modules/useModule";
import { useNavatara } from "../modules/useNatalExtras";
import { useSession } from "../session/store";
import { browserTimeZone, formatMoment, useTransit } from "../transit/store";
import { TransitDate } from "../transit/TransitDate";
import { BalaScroll } from "./bala";
import { nakshatraName, taraName } from "./labels";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Dokuz tara ve üç tur. */
const TARAS = [1, 2, 3, 4, 5, 6, 7, 8, 9] as const;
const CYCLES = [1, 2, 3] as const;

export function NavataraPanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <NavataraBody opened={opened} />;
}

function NavataraBody({ opened }: { opened: OpenedClient }) {
  const { t, i18n } = useTranslation();
  const transit = useTransit();
  const view = transit.view;
  const module = useNavatara(opened, view?.snapshot.jd_ut ?? null);
  // Transit anı değişirken tablo yerinde durur; yeni yanıt gelince değişir.
  const data = useLastData(module, inputKey(opened));
  const phase = usePanelLoadPhase(module.loading || (transit.loading && view === null));
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (data === null) return phase === "skeleton" ? <PanelSkeleton /> : null;
  // Gösterilen taralar transit deposundaki anın taralarıysa an yazılır; yeniden okuma sürerken
  // (bir önceki anın taraları duruyorken) yazılmaz.
  const when = view !== null && data.transit_jd_ut === view.snapshot.jd_ut ? formatMoment(view.utc, i18n.language, browserTimeZone()) : null;
  return (
    <div className="module-panel">
      {data.approximate && <p className="panel-notice">{t("navatara.timeUnknown")}</p>}
      <NatalTable data={data} />
      <p className="panel-message">{t("navatara.countNote")}</p>
      <TransitDate />
      {transit.error ? <PanelError error={transit.error} onRetry={transit.retry} /> : data.transit !== null && <TransitTable data={data} when={when} />}
      <p className="panel-message">{t("navatara.method", { method: data.method })}</p>
    </div>
  );
}

function NatalTable({ data }: { data: Navatara }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const id = useId();
  const on = (rows: readonly NavataraRow[]) => active?.type === "graha" && rows.some((r) => r.lord === active.id || r.grahas.includes(active.id));
  return (
    <BalaScroll labelledBy={`${id}-natal`}>
      <table className="module-table">
        <caption id={`${id}-natal`} className="visually-hidden">
          {t("navatara.caption", { nakshatra: nakshatraName(data.janma_nakshatra) })}
        </caption>
        <thead>
          <tr>
            <th scope="col">{t("navatara.tara")}</th>
            {CYCLES.map((c) => (
              <th key={c} scope="col">
                {t("navatara.cycle", { n: c })}
              </th>
            ))}
            <th scope="col">{t("navatara.lord")}</th>
            <th scope="col">{t("navatara.set")}</th>
          </tr>
        </thead>
        <tbody>
          {TARAS.map((n) => {
            const rows = data.rows.filter((r) => r.tara.number === n);
            const first = rows[0];
            if (first === undefined) return null;
            return (
              <tr key={n} data-tara={n} className={on(rows) ? "is-highlighted" : undefined}>
                <th scope="row">{t("navatara.taraValue", { number: n, name: taraName(n) })}</th>
                {CYCLES.map((c) => {
                  const cell = rows.find((r) => r.cycle === c);
                  return (
                    <td key={c} data-nakshatra={cell?.nakshatra}>
                      {cell && (
                        <>
                          {nakshatraName(cell.nakshatra)}
                          {cell.grahas.length > 0 && (
                            <>
                              {" ("}
                              {cell.grahas.map((g, i) => (
                                <span key={g}>
                                  {i > 0 && " "}
                                  <PinButton target={{ type: "graha", id: g as GrahaId }} label={t(`grahaName.${g}`)}>
                                    {t(`graha.${g}`)}
                                  </PinButton>
                                </span>
                              ))}
                              {")"}
                            </>
                          )}
                        </>
                      )}
                    </td>
                  );
                })}
                <td>
                  <PinButton target={{ type: "graha", id: first.lord }}>{t(`grahaName.${first.lord}`)}</PinButton>
                </td>
                <td>{t(first.tara.favourable ? "navatara.favourable" : "navatara.notFavourable")}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </BalaScroll>
  );
}

function TransitTable({ data, when }: { data: Navatara; when: string | null }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const id = useId();
  return (
    <BalaScroll labelledBy={`${id}-transit`}>
      <table className="module-table">
        <caption id={`${id}-transit`}>{when === null ? t("navatara.transitBusy") : t("navatara.transitCaption", { when })}</caption>
        <thead>
          <tr>
            <th scope="col">{t("navatara.graha")}</th>
            <th scope="col">{t("navatara.nakshatra")}</th>
            <th scope="col">{t("navatara.tara")}</th>
            <th scope="col" className="num">
              {t("navatara.cycleColumn")}
            </th>
            <th scope="col">{t("navatara.set")}</th>
          </tr>
        </thead>
        <tbody>
          {(data.transit ?? []).map((row) => (
            <tr key={row.graha} data-graha={row.graha} className={active?.type === "graha" && active.id === row.graha ? "is-highlighted" : undefined}>
              <th scope="row">
                <PinButton target={{ type: "graha", id: row.graha }}>{t(`grahaName.${row.graha}`)}</PinButton>
              </th>
              <td>
                {nakshatraName(row.nakshatra)} · {t("panchang.pada", { pada: row.pada })}
              </td>
              <td>{t("navatara.taraValue", { number: row.tara.number, name: taraName(row.tara.number) })}</td>
              <td className="num">{row.cycle}</td>
              <td>{t(row.tara.favourable ? "navatara.favourable" : "navatara.notFavourable")}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </BalaScroll>
  );
}
````

- [ ] **Step 4: Katalog, üç dilde metinler ve e2e taklidi**

Metinler dosyaların sonuna değil, ilgili komşu bloğun yanına eklenir ("Paylaşılan dosyalar"): `panel.navatara` `event_search`'ten sonra; `panchangExtras` bloğu `ashtakavarga`'dan önce; `navatara` bloğu `yogas`'tan önce. E2e taklidi bu görevdedir: hazır düzenlerin ikisi (Prashna, Klasik) Panchang panelini taşır ve taklit olmadan ek tablo hata kutusu çizerdi.

````diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index 0b96d84..2ee3e04 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -128,6 +128,10 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
         chara_dasha_at: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.chara_dasha_at : null),
         // Shadbala: fikstürdeki gerçek sonuç; saat bilinmiyorsa Rust `null` döner.
         compute_shadbala: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.shadbala : null),
+        // Doğum anı ekleri (Plan 23): fikstürdeki gerçek sonuçlar. Navatara transit anı verilmezse
+        // yalnızca doğum tablosunu döner (Rust'taki gibi).
+        compute_panchang_extras: () => fixture.natal_extras.panchang_extras,
+        compute_navatara: (args) => (args.transitJdUt === null ? { ...fixture.natal_extras.navatara, transit: null, transit_jd_ut: null } : fixture.natal_extras.navatara),
         // Yogalar: fikstürdeki iki gerçek sonuç (doğum saati bilinen ve bilinmeyen girdi).
         compute_yogas: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.yogas : fixture.yogas_time_unknown),
         // Uyum: eş haritanın girdisi ve kısa kimliği; başka kimlik `null` döner (harita yok).
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 17db154..9ae80ed 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -77,6 +77,7 @@ export const en: Messages = {
     muhurta_search: "Muhurta search",
     prashna_facts: "Prashna facts",
     event_search: "Event search",
+    navatara: "Navatara",
     unknown: "Unknown panel",
     loading: "Loading",
     unavailable: "This panel arrives in a later version.",
@@ -110,6 +111,17 @@ export const en: Messages = {
       Saturday: "Saturday",
     },
   },
+  panchangExtras: {
+    caption: "Panchang additions",
+    yogiPoint: "Yogi point",
+    yogi: "Yogi",
+    coYogi: "Duplicate yogi",
+    avayogi: "Avayogi",
+    dagdha: "Dagdha rasi",
+    none: "none (there is no dagdha rasi at the full and the new moon)",
+    unverified: "Unverified table: the dagdha rasi table has not yet been checked against a printed source.",
+    method: "Method: {{method}}",
+  },
   ashtakavarga: {
     view: "View",
     format: "Format",
@@ -226,6 +238,24 @@ export const en: Messages = {
     drishti: "Drishti",
     note: "The bhavas are equal: each midpoint is counted from the lagna in steps of 30° and does not depend on the house system setting. Bhavadhipati is the Shadbala total of the lord.",
   },
+  navatara: {
+    caption: "Navatara: birth nakshatra {{nakshatra}}; nine taras in three rounds",
+    tara: "Tara",
+    taraValue: "{{number}} · {{name}}",
+    cycle: "Round {{n}}",
+    cycleColumn: "Round",
+    lord: "Lord",
+    set: "Set",
+    favourable: "in the favourable set",
+    notFavourable: "not in the favourable set",
+    graha: "Graha",
+    nakshatra: "Nakshatra",
+    countNote: "The count starts from the birth Moon's nakshatra and includes it; the grahas in brackets are the natal grahas in that nakshatra. There is no count from the lagna nakshatra.",
+    timeUnknown: "Birth time unknown: the Moon is at its 12:00 noon position; if the Moon changes nakshatra that day, the whole table shifts by one nakshatra.",
+    transitCaption: "Taras of the transit grahas, {{when}}",
+    transitBusy: "Taras of the transit grahas",
+    method: "Method: {{method}}",
+  },
   yogas: {
     listLabel: "Yogas that form",
     groupLabel: "Group",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 9b6621f..2fbe5dd 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -77,6 +77,7 @@ export const fr: Messages = {
     muhurta_search: "Recherche de muhurta",
     prashna_facts: "Faits du prashna",
     event_search: "Recherche d’événements",
+    navatara: "Navatara",
     unknown: "Panneau inconnu",
     loading: "Chargement",
     unavailable: "Ce panneau arrivera dans une version ultérieure.",
@@ -110,6 +111,17 @@ export const fr: Messages = {
       Saturday: "samedi",
     },
   },
+  panchangExtras: {
+    caption: "Compléments du panchang",
+    yogiPoint: "Point yogi",
+    yogi: "Yogi",
+    coYogi: "Co-yogi",
+    avayogi: "Avayogi",
+    dagdha: "Dagdha rasi",
+    none: "aucun (pas de dagdha rasi à la pleine lune ni à la nouvelle lune)",
+    unverified: "Table non vérifiée : la table des dagdha rasi n’a pas encore été comparée à une source imprimée.",
+    method: "Méthode : {{method}}",
+  },
   ashtakavarga: {
     view: "Vue",
     format: "Format",
@@ -226,6 +238,24 @@ export const fr: Messages = {
     drishti: "Drishti",
     note: "Les bhavas sont égaux : chaque milieu est compté depuis le lagna par pas de 30° et ne dépend pas du système de maisons choisi. Bhavadhipati est le total Shadbala du maître.",
   },
+  navatara: {
+    caption: "Navatara : nakshatra de naissance {{nakshatra}} ; neuf taras en trois tours",
+    tara: "Tara",
+    taraValue: "{{number}} · {{name}}",
+    cycle: "Tour {{n}}",
+    cycleColumn: "Tour",
+    lord: "Maître",
+    set: "Ensemble",
+    favourable: "dans l’ensemble favorable",
+    notFavourable: "hors de l’ensemble favorable",
+    graha: "Graha",
+    nakshatra: "Nakshatra",
+    countNote: "Le compte part du nakshatra de la Lune natale et l’inclut ; les grahas entre parenthèses sont les grahas natals de ce nakshatra. Il n’y a pas de compte depuis le nakshatra du lagna.",
+    timeUnknown: "Heure de naissance inconnue : la Lune est à sa position de 12 h 00 ; si la Lune change de nakshatra ce jour-là, toute la table se décale d’un nakshatra.",
+    transitCaption: "Taras des grahas en transit, {{when}}",
+    transitBusy: "Taras des grahas en transit",
+    method: "Méthode : {{method}}",
+  },
   yogas: {
     listLabel: "Yogas formés",
     groupLabel: "Groupe",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 8b5784d..0eef523 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -76,6 +76,7 @@ export const tr = {
     muhurta_search: "Muhurta araması",
     prashna_facts: "Prashna olguları",
     event_search: "Olay araması",
+    navatara: "Navatara",
     unknown: "Bilinmeyen panel",
     loading: "Yükleniyor",
     unavailable: "Bu panel sonraki bir sürümde gelecek.",
@@ -109,6 +110,17 @@ export const tr = {
       Saturday: "Cumartesi",
     },
   },
+  panchangExtras: {
+    caption: "Panchang ekleri",
+    yogiPoint: "Yogi noktası",
+    yogi: "Yogi",
+    coYogi: "Eş yogi",
+    avayogi: "Avayogi",
+    dagdha: "Dagdha rasi",
+    none: "yok (dolunay ve yeni ayda dagdha rasi olmaz)",
+    unverified: "Doğrulanmamış tablo: dagdha rasi tablosu basılı kaynakla henüz karşılaştırılmadı.",
+    method: "Yöntem: {{method}}",
+  },
   ashtakavarga: {
     view: "Görünüm",
     format: "Biçim",
@@ -225,6 +237,24 @@ export const tr = {
     drishti: "Drishti",
     note: "Bhavalar eşittir: bhava ortası lagnadan 30°'lik adımlarla sayılır ve ev sistemi ayarından bağımsızdır. Bhavadhipati, lordun Shadbala toplamıdır.",
   },
+  navatara: {
+    caption: "Navatara: doğum nakshatrası {{nakshatra}}; dokuz tara, üç tur",
+    tara: "Tara",
+    taraValue: "{{number}} · {{name}}",
+    cycle: "{{n}}. tur",
+    cycleColumn: "Tur",
+    lord: "Lord",
+    set: "Küme",
+    favourable: "elverişli kümede",
+    notFavourable: "elverişli kümede değil",
+    graha: "Graha",
+    nakshatra: "Nakshatra",
+    countNote: "Sayım doğum Ay'ının nakshatrasından başlar ve onu da sayar; parantez içindekiler o nakshatradaki doğum grahalarıdır. Lagna nakshatrasından sayım yoktur.",
+    timeUnknown: "Doğum saati bilinmiyor: Ay öğlen 12:00 konumundadır; Ay o gün nakshatra değiştiriyorsa bütün tablo bir nakshatra kayar.",
+    transitCaption: "Transit grahalarının tarası, {{when}}",
+    transitBusy: "Transit grahalarının tarası",
+    method: "Yöntem: {{method}}",
+  },
   yogas: {
     listLabel: "Oluşan yogalar",
     groupLabel: "Grup",
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index c3629ce..071d415 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -24,6 +24,7 @@ export const PANEL_KINDS = [
   "muhurta_search",
   "prashna_facts",
   "event_search",
+  "navatara",
 ] as const;
 export type KnownPanelKind = (typeof PANEL_KINDS)[number];
 
@@ -53,6 +54,8 @@ export const MODULE_PANELS = [
   { kind: "prashna_facts", w: 7, h: 20 },
   // Olay araması (Plan 22).
   { kind: "event_search", w: 7, h: 18 },
+  // Doğum anı ekleri (Plan 23).
+  { kind: "navatara", w: 7, h: 16 },
 ] as const satisfies readonly { kind: KnownPanelKind; w: number; h: number }[];
 export type ModulePanelKind = (typeof MODULE_PANELS)[number]["kind"];
 
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index cee04ca..7fb670d 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -17,6 +17,7 @@ import { GrahaTablePanel } from "./GrahaTablePanel";
 import { JaiminiKarakasPanel } from "./JaiminiKarakasPanel";
 import { MuhurtaDayPanel } from "./MuhurtaDayPanel";
 import { MuhurtaSearchPanel } from "./MuhurtaSearchPanel";
+import { NavataraPanel } from "./NavataraPanel";
 import { PanchangPanel } from "./PanchangPanel";
 import { EventSearchPanel } from "./EventSearchPanel";
 import { PrashnaFactsPanel } from "./PrashnaFactsPanel";
@@ -59,6 +60,8 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   prashna_facts: { titleKey: "panel.prashna_facts", component: PrashnaFactsPanel },
   // Olay araması (Plan 22).
   event_search: { titleKey: "panel.event_search", component: EventSearchPanel },
+  // Doğum anı ekleri (Plan 23).
+  navatara: { titleKey: "panel.navatara", component: NavataraPanel },
 };
 
 const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };
````

- [ ] **Step 5: Testler ve kapılar**

```bash
pnpm --dir app exec vitest run ui/panels/PanchangExtras.test.tsx ui/panels/PanchangPanel.test.tsx ui/panels/NavataraPanel.test.tsx ui/layout/model.test.ts ui/panels/panels.test.tsx ui/App.test.tsx 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
grep -nE 'console\.|localStorage|sessionStorage|new Date\(|Date\.now|\bfetch\(' app/ui/modules/useNatalExtras.ts app/ui/panels/PanchangExtras.tsx app/ui/panels/NavataraPanel.tsx
git diff --stat -- app/ui/panels/panels.css app/ui/i18n/i18n.test.ts
```
Expected: `6 passed (6)`, `98 passed (98)`; tümü `90 passed (90)`, `902 passed (902)`; `✓ built`; tarama çıktısız; son komut çıktısız (stil dosyası ve i18n testi değişmedi).

- [ ] **Step 6: Commit**

```bash
git add app/ui/modules/useNatalExtras.ts app/ui/panels/PanchangExtras.tsx app/ui/panels/PanchangExtras.test.tsx app/ui/panels/PanchangPanel.tsx app/ui/panels/PanchangPanel.test.tsx app/ui/panels/NavataraPanel.tsx app/ui/panels/NavataraPanel.test.tsx app/ui/panels/registry.ts app/ui/panels/panels.test.tsx app/ui/layout/model.ts app/ui/layout/model.test.ts app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts app/ui/App.test.tsx app/e2e/ipc.ts
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): panchang additions under the Panchang panel; Navatara panel with the taras of the transit grahas" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: on altı dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 7: Arayüz: Vimshopaka paneli, üç dilde metinler ve uçtan uca akış

Yol haritası görev taslağının (7). maddesi; K12, Y4; C4, C9, C12; D16, D21, D22. Panel Shadbala panelinin kardeşidir: aynı araç çubuğu kalıbı (`detail` ayarı), aynı kaydırma kabı (`BalaScroll`) ve aynı sayı biçimleyici (`formatFixed`). Uçtan uca akış üç yüzeyi birlikte gezer.

**Files:**
- Create: `app/ui/panels/VimshopakaPanel.tsx`, `app/ui/panels/VimshopakaPanel.test.tsx`
- Modify: `app/ui/modules/useNatalExtras.ts`, `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx`, `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`, `app/e2e/ipc.ts`, `app/e2e/desk.spec.ts`
- Test: `VimshopakaPanel.test.tsx` (8); `model.test.ts` ve `panels.test.tsx` (birer tür daha); `desk.spec.ts` (1 akış)

**Interfaces:**
- Consumes: Task 5 (`api.computeVimshopaka`; `VimshopakaReport`, `GrahaVimshopaka` tipleri; fikstürde `natal_extras.vimshopaka` ve `natal_extras.schemes`); Task 6 (`modules/useNatalExtras.ts`); `panels/bala` (`BalaScroll`, `formatFixed`); `view/store` (`useLayoutStore().setPanelSettings`); `session/birth` (`timeUnknown`).
- Produces:
  - `useVimshopaka(opened: OpenedClient | null): ModuleState<VimshopakaReport>`.
  - `VimshopakaPanel({ panel }: PanelProps)`; `VIMSHOPAKA_DETAILS` (`summary`, `vargas`), `vimshopakaDetail(settings)`, `VIMSHOPAKA_SCHEMES` (raporun dört alanı ve klasik adları), `formatVimshopaka(value: number | null): string` (üç ondalık).
  - Panel türü `vimshopaka` (`MODULE_PANELS`: 7 × 12; ayar `{ detail }`); i18n `panel.vimshopaka`, `vimshopaka.*` (üç dil).
  - E2e taklidi `compute_vimshopaka`; akış `natal extras: …`.

- [ ] **Step 1: Testler: özet, vargalar görünümü, biçim, notlar, vurgu, hata ve diller**

Testler: Delhi'nin yedi satırı, üç ondalıkla; "Vargalar" görünümünde Güneş'in ilk altı vargası (türetme yorumda) ve ayarın düzenle saklanması; küme sütunlarının Rust'ın dört kümesi olduğu ve değerlerin 0,025'in katı olduğu; açık harita yok; saat bilinmezliği ve yaklaşık saat notları; ortak vurgu; hata ve yeniden deneme; üç dil ve Fransızca kesme işareti.

**Yeni dosya:** `app/ui/panels/VimshopakaPanel.test.tsx`

````tsx
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeVimshopaka: vi.fn() } }));

import { api, type Result } from "../api";
import type { VimshopakaReport } from "../bindings";
import { setLanguage } from "../i18n";
import { fr } from "../i18n/fr";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { formatVimshopaka, VIMSHOPAKA_SCHEMES, VimshopakaPanel, vimshopakaDetail } from "./VimshopakaPanel";

const ok = (report: VimshopakaReport = fixture().natal_extras.vimshopaka): Result<VimshopakaReport> => ({ status: "ok", data: report });
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "vb", kind: "vimshopaka", x: 0, y: 0, w: 7, h: 12, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};

/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  return <VimshopakaPanel panel={active.panels[0]} />;
}

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened });
}

const CAPTION = "Vimshopaka bala: yedi grahanın dört varga kümesindeki değeri, yirmi üzerinden";
const summary = () => screen.findByRole("table", { name: CAPTION });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const bodyRows = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);

beforeEach(() => {
  vi.mocked(api.computeVimshopaka).mockReset();
  vi.mocked(api.computeVimshopaka).mockImplementation(async () => ok());
});

test("summary: seven grahas with their value in the four varga sets, three decimals, unrounded", async () => {
  renderPanel();
  const table = await summary();
  const opened = fixture().opened;
  expect(api.computeVimshopaka).toHaveBeenCalledTimes(1);
  expect(api.computeVimshopaka).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null });
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Graha", "Shadvarga", "Saptavarga", "Dashavarga", "Shodashavarga"]);
  // Delhi fikstürü.
  expect(bodyRows(table)).toEqual([
    ["Güneş", "14.600", "13.950", "11.725", "12.475"],
    ["Ay", "12.050", "11.650", "10.600", "10.275"],
    ["Mars", "17.100", "16.975", "16.425", "16.300"],
    ["Merkür", "10.550", "10.225", "10.800", "11.700"],
    ["Jüpiter", "17.200", "17.200", "14.150", "13.700"],
    ["Venüs", "14.900", "15.200", "13.300", "13.075"],
    ["Satürn", "7.100", "6.850", "10.550", "10.350"],
  ]);
  // Özet görünümünde varga tablosu yoktur.
  expect(screen.getAllByRole("table")).toHaveLength(1);
  expect(screen.getByText(/^Basamaklar: 20 kendi burcu · 18 çok dost · 15 dost · 10 nötr · 7 düşman · 5 çok düşman\./)).toBeInTheDocument();
  expect(screen.getByText(/^Yücelme ve moolatrikona ayrı basamak değildir;/)).toBeInTheDocument();
  expect(screen.getByText("Doğrulanmamış tablo: küme ağırlıkları ve basamak değerleri basılı kaynakla henüz karşılaştırılmadı.")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: vimshopaka.bphs_v1")).toBeInTheDocument();
  expect(screen.queryByText(/Doğum saati/)).toBeNull();
});

test("vargas view: the sign and the rung value in each of the sixteen vargas; the setting is stored with the panel", async () => {
  const { persistence } = renderPanel();
  await summary();
  fireEvent.change(screen.getByRole("combobox", { name: "Ayrıntı" }), { target: { value: "vargas" } });
  const table = await screen.findByRole("table", { name: "On altı vargada burç ve basamak değeri" });
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Graha", "D1", "D2", "D3", "D4", "D7", "D9", "D10", "D12", "D16", "D20", "D24", "D27", "D30", "D40", "D45", "D60"]);
  // Güneş Yay 16°'dedir (tek, çift doğalı burç). D2: tek burcun ikinci yarısı Ay'ın horasıdır
  // (Yengeç). D3: ikinci drekkana, beşinci burç (Koç). D4: üçüncü parça, yedinci burç
  // (İkizler). D7: dördüncü parça, Yay'dan dördüncü burç (Balık). D9: çift doğalı burçta
  // beşinci burçtan (Koç) sayılır, beşinci navamsa: Aslan, kendi burcu (20).
  const sun = within(table).getAllByRole("row")[1];
  expect(cells(sun).slice(0, 7)).toEqual(["Güneş", "Yay 10", "Yen 18", "Koç 18", "İki 7", "Bal 10", "Asl 20"]);
  // Yeniden istek yoktur: görünüm aynı raporu çizer.
  expect(api.computeVimshopaka).toHaveBeenCalledTimes(1);
  await waitFor(() =>
    expect(persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ detail: "vargas" }),
  );
  expect(vimshopakaDetail({ detail: "vargas" })).toBe("vargas");
  expect(vimshopakaDetail({ detail: "items" })).toBe("summary");
  expect(vimshopakaDetail({})).toBe("summary");
});

test("the scheme columns are Rust's four sets and values are written without rounding", () => {
  expect(VIMSHOPAKA_SCHEMES.map(([, name]) => name)).toEqual(fixture().natal_extras.schemes);
  expect(VIMSHOPAKA_SCHEMES.map(([key]) => key)).toEqual(fixture().natal_extras.schemes.map((s) => s.toLowerCase()));
  expect([20, 5, 11.725, 12.475, 14.275, null, Number.NaN].map(formatVimshopaka)).toEqual(["20.000", "5.000", "11.725", "12.475", "14.275", "—", "—"]);
  // Her değer 0,025'in katıdır: üç ondalık yuvarlama gerektirmez.
  for (const g of fixture().natal_extras.vimshopaka.grahas) {
    for (const [key] of VIMSHOPAKA_SCHEMES) expect(Math.abs((g[key] as number) * 40 - Math.round((g[key] as number) * 40))).toBeLessThan(1e-9);
  }
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeVimshopaka).not.toHaveBeenCalled();
});

test("unknown and approximate birth times are said in words; the table is still shown", async () => {
  const report = fixture().natal_extras.vimshopaka;
  report.approximate = true;
  vi.mocked(api.computeVimshopaka).mockImplementation(async () => ok(report));
  const first = renderPanel({}, unknownTimeOpened());
  await summary();
  expect(vi.mocked(api.computeVimshopaka).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByText(/^Doğum saati bilinmiyor: vargalar öğlen 12:00 konumlarındandır\./)).toBeInTheDocument();
  expect(screen.queryByText(/^Doğum saati yaklaşık/)).toBeNull();
  first.unmount();

  vi.mocked(api.computeVimshopaka).mockImplementation(async () => ok());
  const approx = fixture().opened;
  approx.chart_record.birth.time_certainty = "Approx";
  renderPanel({}, approx);
  await summary();
  expect(screen.getByText(/^Doğum saati yaklaşık: D16 ve üstü vargalar/)).toBeInTheDocument();
  expect(screen.queryByText(/^Doğum saati bilinmiyor/)).toBeNull();
});

test("shared highlight: pinning a graha marks its row in both tables", async () => {
  renderPanel({ detail: "vargas" });
  const table = await summary();
  const marked = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.graha);
  expect(marked()).toEqual([]);
  fireEvent.click(within(table).getByRole("button", { name: "Mars" }));
  expect(marked()).toEqual(["Mars", "Mars"]);
  expect(within(table).getByRole("button", { name: "Mars" })).toHaveAttribute("aria-pressed", "true");
});

test("an error keeps the toolbar and is retried; focus stays inside the panel", async () => {
  vi.mocked(api.computeVimshopaka).mockImplementationOnce(async () => ({ status: "error", error: { code: "engine_stopped", params: {} } }));
  renderPanel();
  const alert = await screen.findByRole("alert");
  expect(screen.getByRole("combobox", { name: "Ayrıntı" })).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
  fireEvent.click(within(alert).getByRole("button", { name: "Yeniden dene" }));
  await summary();
  expect(api.computeVimshopaka).toHaveBeenCalledTimes(2);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("set names stay the same in every language; the rest is translated; French uses the typographic apostrophe", async () => {
  await setLanguage("en");
  const first = renderPanel({ detail: "vargas" });
  const table = await screen.findByRole("table", { name: "Vimshopaka bala: the value of the seven grahas in the four varga sets, out of twenty" });
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Graha", "Shadvarga", "Saptavarga", "Dashavarga", "Shodashavarga"]);
  expect(screen.getByRole("table", { name: "Sign and rung value in the sixteen vargas" })).toBeInTheDocument();
  expect(screen.getByRole("combobox", { name: "Detail" })).toHaveDisplayValue("Vargas");
  first.unmount();
  await setLanguage("fr");
  renderPanel();
  const french = await screen.findByRole("table", { name: "Vimshopaka bala : valeur des sept grahas dans les quatre ensembles de vargas, sur vingt" });
  expect(bodyRows(french)[0]).toEqual(["Soleil", "14.600", "13.950", "11.725", "12.475"]);
  const texts = Object.values(fr.vimshopaka).flatMap((v) => (typeof v === "string" ? [v] : Object.values(v)));
  for (const text of texts) expect(text).not.toContain("'");
});
````

````diff
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index c467ec7..a767bed 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -54,6 +54,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "prashna_facts",
     "event_search",
     "navatara",
+    "vimshopaka",
   ]);
   expect(isKnownPanelKind("chart")).toBe(true);
   expect(isKnownPanelKind("drishti")).toBe(true);
@@ -61,7 +62,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search", "navatara"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search", "navatara", "vimshopaka"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index 9f0f3f1..83d015f 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -26,6 +26,7 @@ import { PanchangPanel } from "./PanchangPanel";
 import { panelDefinition, PANEL_REGISTRY } from "./registry";
 import { ShadbalaPanel } from "./ShadbalaPanel";
 import { UnavailablePanel } from "./UnavailablePanel";
+import { VimshopakaPanel } from "./VimshopakaPanel";
 import { YogasPanel } from "./YogasPanel";
 
 const chartPanel = (settings: Record<string, unknown> = { varga: "D1", style: "north" }): Panel => ({
@@ -62,6 +63,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.muhurta_search.component).toBe(MuhurtaSearchPanel);
   expect(PANEL_REGISTRY.event_search.component).toBe(EventSearchPanel);
   expect(PANEL_REGISTRY.navatara.component).toBe(NavataraPanel);
+  expect(PANEL_REGISTRY.vimshopaka.component).toBe(VimshopakaPanel);
   // Kataloğun her türü uygulanmıştır: yer tutucu yoktur.
   for (const kind of PANEL_KINDS) expect(PANEL_REGISTRY[kind].component, kind).not.toBe(UnavailablePanel);
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
````

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/panels/VimshopakaPanel.test.tsx ui/layout/model.test.ts ui/panels/panels.test.tsx 2>&1 | grep -E "Test Files|Tests |Failed to resolve" | sort -u`
Expected: `Test Files  3 failed (3)`; `VimshopakaPanel.test.tsx` ve `panels.test.tsx` yüklenmez (`Failed to resolve import "./VimshopakaPanel"`); `model.test.ts`'in iki testi düşer.

- [ ] **Step 3: Kanca, panel, katalog ve metinler**

`panel.vimshopaka` `navatara`'dan sonra, `vimshopaka` bloğu `yogas`'tan önce (Task 6'nın `navatara` bloğunun ardına) eklenir. Küme adları (Shadvarga …) klasik terimdir, üç dilde aynı yazılır ve i18n'e girmez (K11).

````diff
diff --git a/app/ui/modules/useNatalExtras.ts b/app/ui/modules/useNatalExtras.ts
index 9c03a73..46e7fdc 100644
--- a/app/ui/modules/useNatalExtras.ts
+++ b/app/ui/modules/useNatalExtras.ts
@@ -1,8 +1,8 @@
-// Doğum anı eklerinin (Plan 23) kancaları: `compute_panchang_extras` ve `compute_navatara`.
-// Sonuç saklanmaz, panel açıkken istenir (K1); arayüzde hesap yoktur (C4). İkisi de lagna
-// istemez: doğum saati bilinmiyorsa sonuç yine gelir ve `approximate` işaretlidir.
+// Doğum anı eklerinin (Plan 23) kancaları: `compute_panchang_extras`, `compute_navatara` ve
+// `compute_vimshopaka`. Sonuç saklanmaz, panel açıkken istenir (K1); arayüzde hesap yoktur
+// (C4). Üçü de lagna istemez: doğum saati bilinmiyorsa sonuç yine gelir ve `approximate` işaretlidir.
 import { api } from "../api";
-import type { Navatara, OpenedClient, PanchangExtras } from "../bindings";
+import type { Navatara, OpenedClient, PanchangExtras, VimshopakaReport } from "../bindings";
 import { chartInput, inputKey } from "./input";
 import { useModule, type ModuleState } from "./useModule";
 
@@ -17,3 +17,7 @@ export function usePanchangExtras(opened: OpenedClient | null): ModuleState<Panc
 export function useNavatara(opened: OpenedClient | null, transitJdUt: number | null): ModuleState<Navatara> {
   return useModule(opened !== null ? JSON.stringify([inputKey(opened), transitJdUt]) : null, () => api.computeNavatara(chartInput(opened as OpenedClient), transitJdUt));
 }
+
+export function useVimshopaka(opened: OpenedClient | null): ModuleState<VimshopakaReport> {
+  return useModule(opened !== null ? inputKey(opened) : null, () => api.computeVimshopaka(chartInput(opened as OpenedClient)));
+}
````

**Yeni dosya:** `app/ui/panels/VimshopakaPanel.tsx`

````tsx
// Vimshopaka bala paneli (yol haritası §4.3): yedi grahanın dört varga kümesindeki (shadvarga,
// saptavarga, dashavarga, shodashavarga) yirmi üzerinden değeri; "Vargalar" görünümünde on altı
// vargadaki burç ve basamak değeri. Değerler `compute_vimshopaka`'dan gelir ve panel açıkken
// istenir (K1); burada hesap yoktur (C4). Değerler 0,025'in katıdır ve üç ondalıkla,
// yuvarlanmadan yazılır. Ağırlık ve basamak tabloları sahibinin onayını bekler (Y4): panel
// "doğrulanmamış tablo" notunu taşır. Renk anlam taşımaz (C9).
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { GrahaVimshopaka, OpenedClient, VimshopakaReport } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import type { Panel } from "../layout/model";
import { useLastData } from "../modules/useModule";
import { useVimshopaka } from "../modules/useNatalExtras";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import { BalaScroll, formatFixed } from "./bala";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export const VIMSHOPAKA_DETAILS = ["summary", "vargas"] as const;
export type VimshopakaDetail = (typeof VIMSHOPAKA_DETAILS)[number];

/** Dört küme: raporun alanı ve kümenin klasik adı (üç dilde aynı yazılır, K11). Rust `VargaScheme::ALL` sırası. */
export const VIMSHOPAKA_SCHEMES = [
  ["shadvarga", "Shadvarga"],
  ["saptavarga", "Saptavarga"],
  ["dashavarga", "Dashavarga"],
  ["shodashavarga", "Shodashavarga"],
] as const satisfies readonly (readonly [keyof GrahaVimshopaka, string])[];

/** Panelin ayrıntı ayarı; tanınmayan değer ve eksik anahtar özettir. */
export function vimshopakaDetail(settings: Record<string, unknown>): VimshopakaDetail {
  return settings.detail === "vargas" ? "vargas" : "summary";
}

/** Küme değeri, üç ondalık (değerler 0,025'in katıdır: yuvarlama olmaz). */
export function formatVimshopaka(value: number | null): string {
  return formatFixed(value, 3);
}

export function VimshopakaPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Vimshopaka panel={panel} opened={opened} />;
}

function Vimshopaka({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  const module = useVimshopaka(opened);
  // Aynı haritanın yeniden okunmasında (ayar değişimi) tablo yerinde durur.
  const report = useLastData(module, String(opened.chart_record.id));
  const phase = usePanelLoadPhase(module.loading);
  const detail = vimshopakaDetail(panel.settings);
  // Saat kesinliği doğum kaydındandır (gösterim kararı; hesap değil).
  const approxTime = !timeUnknown(opened) && opened.chart_record.birth.time_certainty === "Approx";
  return (
    <div className="module-panel">
      {/* Araç çubuğu yükleme ve hata durumlarında da kullanılabilir. */}
      <div className="module-toolbar">
        <label htmlFor={`${id}-detail`}>{t("vimshopaka.detail")}</label>
        <select id={`${id}-detail`} value={detail} onChange={(e) => setPanelSettings(panel.id, { detail: e.target.value })}>
          {VIMSHOPAKA_DETAILS.map((d) => (
            <option key={d} value={d}>
              {t(`vimshopaka.details.${d}`)}
            </option>
          ))}
        </select>
      </div>
      {module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : report === null ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : (
        <>
          {report.approximate && <p className="panel-notice">{t("vimshopaka.timeUnknown")}</p>}
          {!report.approximate && approxTime && <p className="panel-notice">{t("vimshopaka.timeApprox")}</p>}
          <Report report={report} detail={detail} />
        </>
      )}
    </div>
  );
}

function Report({ report, detail }: { report: VimshopakaReport; detail: VimshopakaDetail }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const id = useId();
  const row = (g: GrahaVimshopaka) => (active?.type === "graha" && active.id === g.graha ? "is-highlighted" : undefined);
  return (
    <>
      <BalaScroll labelledBy={`${id}-summary`}>
        <table className="module-table">
          <caption id={`${id}-summary`} className="visually-hidden">
            {t("vimshopaka.caption")}
          </caption>
          <thead>
            <tr>
              <th scope="col">{t("vimshopaka.graha")}</th>
              {VIMSHOPAKA_SCHEMES.map(([key, name]) => (
                <th key={key} scope="col" className="num">
                  {name}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {report.grahas.map((g) => (
              <tr key={g.graha} data-graha={g.graha} className={row(g)}>
                <th scope="row">
                  <PinButton target={{ type: "graha", id: g.graha }}>{t(`grahaName.${g.graha}`)}</PinButton>
                </th>
                {VIMSHOPAKA_SCHEMES.map(([key]) => (
                  <td key={key} className="num">
                    {formatVimshopaka(g[key])}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </BalaScroll>
      {detail === "vargas" && (
        <BalaScroll labelledBy={`${id}-vargas`}>
          <table className="module-table">
            <caption id={`${id}-vargas`}>{t("vimshopaka.vargasCaption")}</caption>
            <thead>
              <tr>
                <th scope="col">{t("vimshopaka.graha")}</th>
                {(report.grahas[0]?.vargas ?? []).map((v) => (
                  <th key={v.varga} scope="col">
                    {v.varga}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {report.grahas.map((g) => (
                <tr key={g.graha} data-graha={g.graha} className={row(g)}>
                  <th scope="row">{t(`grahaName.${g.graha}`)}</th>
                  {g.vargas.map((v) => (
                    <td key={v.varga} data-varga={v.varga}>
                      {t(`sign.${signIndex(v.sign)}`)} {v.points === null ? "—" : v.points}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </BalaScroll>
      )}
      <p className="panel-message">{t("vimshopaka.legend")}</p>
      <p className="panel-message">{t("vimshopaka.reading")}</p>
      <p className="panel-message">{t("vimshopaka.unverified")}</p>
      <p className="panel-message">{t("vimshopaka.method", { method: report.method })}</p>
    </>
  );
}
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 9ae80ed..e994596 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -78,6 +78,7 @@ export const en: Messages = {
     prashna_facts: "Prashna facts",
     event_search: "Event search",
     navatara: "Navatara",
+    vimshopaka: "Vimshopaka bala",
     unknown: "Unknown panel",
     loading: "Loading",
     unavailable: "This panel arrives in a later version.",
@@ -256,6 +257,19 @@ export const en: Messages = {
     transitBusy: "Taras of the transit grahas",
     method: "Method: {{method}}",
   },
+  vimshopaka: {
+    caption: "Vimshopaka bala: the value of the seven grahas in the four varga sets, out of twenty",
+    graha: "Graha",
+    detail: "Detail",
+    details: { summary: "Summary", vargas: "Vargas" },
+    vargasCaption: "Sign and rung value in the sixteen vargas",
+    legend: "Rungs: 20 own sign · 18 great friend · 15 friend · 10 neutral · 7 enemy · 5 great enemy. The value of a set is one twentieth of the weighted sum (5–20).",
+    reading: "Exaltation and moolatrikona are not separate rungs; the temporal friendship is read from the D1 signs in every varga. Rahu and Ketu get no value.",
+    timeUnknown: "Birth time unknown: the vargas are from the 12:00 noon positions. High divisions change sign within minutes; the values are approximate.",
+    timeApprox: "Birth time approximate: D16 and higher vargas change sign within minutes; the values that rest on them are not exact.",
+    unverified: "Unverified table: the set weights and the rung values have not yet been checked against a printed source.",
+    method: "Method: {{method}}",
+  },
   yogas: {
     listLabel: "Yogas that form",
     groupLabel: "Group",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 2fbe5dd..d9089c9 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -78,6 +78,7 @@ export const fr: Messages = {
     prashna_facts: "Faits du prashna",
     event_search: "Recherche d’événements",
     navatara: "Navatara",
+    vimshopaka: "Vimshopaka bala",
     unknown: "Panneau inconnu",
     loading: "Chargement",
     unavailable: "Ce panneau arrivera dans une version ultérieure.",
@@ -256,6 +257,19 @@ export const fr: Messages = {
     transitBusy: "Taras des grahas en transit",
     method: "Méthode : {{method}}",
   },
+  vimshopaka: {
+    caption: "Vimshopaka bala : valeur des sept grahas dans les quatre ensembles de vargas, sur vingt",
+    graha: "Graha",
+    detail: "Détail",
+    details: { summary: "Résumé", vargas: "Vargas" },
+    vargasCaption: "Signe et valeur d’échelon dans les seize vargas",
+    legend: "Échelons : 20 propre signe · 18 grand ami · 15 ami · 10 neutre · 7 ennemi · 5 grand ennemi. La valeur d’un ensemble est le vingtième de la somme pondérée (5–20).",
+    reading: "L’exaltation et le moolatrikona ne sont pas des échelons distincts ; l’amitié temporaire se lit dans les signes du D1 pour chaque varga. Rahu et Ketu ne reçoivent pas de valeur.",
+    timeUnknown: "Heure de naissance inconnue : les vargas viennent des positions de 12 h 00. Les divisions élevées changent de signe en quelques minutes ; les valeurs sont approximatives.",
+    timeApprox: "Heure de naissance approximative : les vargas D16 et au-delà changent de signe en quelques minutes ; les valeurs qui en dépendent ne sont pas exactes.",
+    unverified: "Table non vérifiée : les poids des ensembles et les valeurs des échelons n’ont pas encore été comparés à une source imprimée.",
+    method: "Méthode : {{method}}",
+  },
   yogas: {
     listLabel: "Yogas formés",
     groupLabel: "Groupe",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 0eef523..a5ab77f 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -77,6 +77,7 @@ export const tr = {
     prashna_facts: "Prashna olguları",
     event_search: "Olay araması",
     navatara: "Navatara",
+    vimshopaka: "Vimshopaka bala",
     unknown: "Bilinmeyen panel",
     loading: "Yükleniyor",
     unavailable: "Bu panel sonraki bir sürümde gelecek.",
@@ -255,6 +256,19 @@ export const tr = {
     transitBusy: "Transit grahalarının tarası",
     method: "Yöntem: {{method}}",
   },
+  vimshopaka: {
+    caption: "Vimshopaka bala: yedi grahanın dört varga kümesindeki değeri, yirmi üzerinden",
+    graha: "Graha",
+    detail: "Ayrıntı",
+    details: { summary: "Özet", vargas: "Vargalar" },
+    vargasCaption: "On altı vargada burç ve basamak değeri",
+    legend: "Basamaklar: 20 kendi burcu · 18 çok dost · 15 dost · 10 nötr · 7 düşman · 5 çok düşman. Kümenin değeri, ağırlıklı toplamın yirmide biridir (5–20).",
+    reading: "Yücelme ve moolatrikona ayrı basamak değildir; geçici dostluk her vargada D1 burçlarından okunur. Rahu ve Ketu'ya değer verilmez.",
+    timeUnknown: "Doğum saati bilinmiyor: vargalar öğlen 12:00 konumlarındandır. Yüksek bölümlü vargalar birkaç dakikada burç değiştirir; değerler yaklaşıktır.",
+    timeApprox: "Doğum saati yaklaşık: D16 ve üstü vargalar birkaç dakikada burç değiştirir; onlara dayanan değerler kesin değildir.",
+    unverified: "Doğrulanmamış tablo: küme ağırlıkları ve basamak değerleri basılı kaynakla henüz karşılaştırılmadı.",
+    method: "Yöntem: {{method}}",
+  },
   yogas: {
     listLabel: "Oluşan yogalar",
     groupLabel: "Grup",
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index 071d415..b014c73 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -25,6 +25,7 @@ export const PANEL_KINDS = [
   "prashna_facts",
   "event_search",
   "navatara",
+  "vimshopaka",
 ] as const;
 export type KnownPanelKind = (typeof PANEL_KINDS)[number];
 
@@ -56,6 +57,7 @@ export const MODULE_PANELS = [
   { kind: "event_search", w: 7, h: 18 },
   // Doğum anı ekleri (Plan 23).
   { kind: "navatara", w: 7, h: 16 },
+  { kind: "vimshopaka", w: 7, h: 12 },
 ] as const satisfies readonly { kind: KnownPanelKind; w: number; h: number }[];
 export type ModulePanelKind = (typeof MODULE_PANELS)[number]["kind"];
 
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index 7fb670d..9ddd7e7 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -24,6 +24,7 @@ import { PrashnaFactsPanel } from "./PrashnaFactsPanel";
 import { ShadbalaPanel } from "./ShadbalaPanel";
 import type { PanelDefinition } from "./types";
 import { UnavailablePanel } from "./UnavailablePanel";
+import { VimshopakaPanel } from "./VimshopakaPanel";
 import { YogasPanel } from "./YogasPanel";
 
 export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
@@ -62,6 +63,7 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   event_search: { titleKey: "panel.event_search", component: EventSearchPanel },
   // Doğum anı ekleri (Plan 23).
   navatara: { titleKey: "panel.navatara", component: NavataraPanel },
+  vimshopaka: { titleKey: "panel.vimshopaka", component: VimshopakaPanel },
 };
 
 const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };
````

- [ ] **Step 4: E2e taklidi ve akış**

Akış: raporlar panel açılmadan istenmez (K1); Panchang panelinde ek tablonun dört satırı ve notu; Navatara'da Janma satırı, geçiş tablosunda Ay'ın satırı, Rust'a giden an ve ortak vurgu (tablodaki Ay haritadaki Ay'ı ve lordu olduğu 8. tarayı vurgular); Vimshopaka'da Jüpiter'in dört değeri, "Vargalar" görünümü ve notu; iki panelin ayarlarıyla kaydedilmesi; panelin aynı komutla çıkarılması ve odağın gövdeye düşmemesi.

````diff
diff --git a/app/e2e/desk.spec.ts b/app/e2e/desk.spec.ts
index a42e99c..a3ab541 100644
--- a/app/e2e/desk.spec.ts
+++ b/app/e2e/desk.spec.ts
@@ -1239,3 +1239,75 @@ test("event search: the panel from the palette, the event list in the chart's ti
   await expect(panel).toHaveCount(0);
   expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
 });
+
+test("natal extras: panchang additions, navatara with the transit taras and vimshopaka bala from the palette", async ({ page }) => {
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
+  // Üç rapor da panel açılmadan istenmez (K1).
+  for (const cmd of ["compute_panchang_extras", "compute_navatara", "compute_vimshopaka"]) expect(await ipcCalls(page, cmd)).toHaveLength(0);
+
+  // Panchang paneli: beş uzvun altında panchang ekleri.
+  await toggle("panchang", /Panchang panelini aç\/kapat/);
+  const panchang = page.getByRole("region", { name: "Panchang", exact: true });
+  const extras = panchang.getByRole("table", { name: "Panchang ekleri" });
+  await expect(extras.locator('tr[data-extra="yogi_point"] td')).toHaveText("Ter 9°19′ · Swati · pada 1");
+  await expect(extras.locator('tr[data-extra="yogi"] td')).toHaveText("Rahu");
+  await expect(extras.locator('tr[data-extra="avayogi"] td')).toHaveText("Venüs · Purva Ashadha");
+  await expect(extras.locator('tr[data-extra="dagdha"] td')).toHaveText("Yay, Bal");
+  await expect(panchang.getByText(/^Doğrulanmamış tablo: dagdha rasi tablosu/)).toBeVisible();
+  expect(await ipcCalls(page, "compute_panchang_extras")).toHaveLength(1);
+
+  // Navatara: doğum tablosu ve transit anının taraları; an Rust'a transit deposundan gider.
+  await toggle("navatara", /Navatara panelini aç\/kapat/);
+  const navatara = page.getByRole("region", { name: "Navatara" });
+  await expect(navatara).toBeFocused();
+  const natal = navatara.getByRole("table", { name: /^Navatara: doğum nakshatrası Swati/ });
+  await expect(natal.locator('tbody tr[data-tara="1"] td')).toHaveText(["Swati (Ay)", "Shatabhisha", "Ardra", "Rahu", "elverişli kümede değil"]);
+  const transit = navatara.getByRole("table", { name: /^Transit grahalarının tarası,/ });
+  await expect(transit.locator('tbody tr[data-graha="Moon"] td')).toHaveText(["Rohini · pada 4", "8 · Mitra", "2", "elverişli kümede"]);
+  const navataraCalls = (await ipcCalls(page, "compute_navatara")) as { transitJdUt: number | null }[];
+  expect(navataraCalls.at(-1)?.transitJdUt).toBe(FIXTURE.transit.snapshot.jd_ut);
+  // Ortak vurgu: tablodaki Ay, haritadaki Ay'ı vurgular.
+  await natal.locator('tbody tr[data-tara="1"]').getByRole("button", { name: "Ay" }).hover();
+  await expect(page.locator('svg [data-graha="Moon"]').first()).toHaveClass(/chart-planet--highlight/);
+  await expect(natal.locator('tbody tr[data-tara="8"]')).toHaveClass(/is-highlighted/);
+
+  // Vimshopaka bala: özet ve varga görünümü; ayar düzenle birlikte saklanır.
+  await toggle("vimshopaka", /Vimshopaka bala panelini aç\/kapat/);
+  const vimshopaka = page.getByRole("region", { name: "Vimshopaka bala" });
+  await expect(vimshopaka).toBeFocused();
+  const summary = vimshopaka.getByRole("table", { name: /^Vimshopaka bala: yedi grahanın dört varga kümesindeki değeri/ });
+  await expect(summary.locator('tbody tr[data-graha="Jupiter"] td')).toHaveText(["17.200", "17.200", "14.150", "13.700"]);
+  await vimshopaka.getByRole("combobox", { name: "Ayrıntı" }).selectOption("vargas");
+  const vargas = vimshopaka.getByRole("table", { name: "On altı vargada burç ve basamak değeri" });
+  await expect(vargas.locator('tbody tr[data-graha="Sun"] td[data-varga="D9"]')).toHaveText("Asl 20");
+  await expect(vimshopaka.getByText(/^Doğrulanmamış tablo: küme ağırlıkları/)).toBeVisible();
+  expect(await ipcCalls(page, "compute_vimshopaka")).toHaveLength(1);
+  await expect
+    .poll(async () => {
+      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: unknown }[] }[] } } } | undefined;
+      const layout = saved?.viewState.panel_layout;
+      return layout?.layouts
+        .find((l) => l.id === layout.active_layout_id)
+        ?.panels.filter((p) => p.kind === "navatara" || p.kind === "vimshopaka")
+        .map((p) => [p.kind, p.settings]);
+    })
+    .toEqual([
+      ["navatara", {}],
+      ["vimshopaka", { detail: "vargas" }],
+    ]);
+
+  // Aynı komut paneli kaldırır; odak gövdeye düşmez.
+  await toggle("navatara", /Navatara panelini aç\/kapat/);
+  await expect(navatara).toHaveCount(0);
+  await expect(vimshopaka).toBeVisible();
+  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
+});
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index 2ee3e04..bd1a118 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -132,6 +132,7 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
         // yalnızca doğum tablosunu döner (Rust'taki gibi).
         compute_panchang_extras: () => fixture.natal_extras.panchang_extras,
         compute_navatara: (args) => (args.transitJdUt === null ? { ...fixture.natal_extras.navatara, transit: null, transit_jd_ut: null } : fixture.natal_extras.navatara),
+        compute_vimshopaka: () => fixture.natal_extras.vimshopaka,
         // Yogalar: fikstürdeki iki gerçek sonuç (doğum saati bilinen ve bilinmeyen girdi).
         compute_yogas: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.yogas : fixture.yogas_time_unknown),
         // Uyum: eş haritanın girdisi ve kısa kimliği; başka kimlik `null` döner (harita yok).
````

- [ ] **Step 5: Testler, akış ve kapılar**

```bash
pnpm --dir app exec vitest run ui/panels/VimshopakaPanel.test.tsx ui/layout/model.test.ts ui/panels/panels.test.tsx 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
lsof -i :1421
pnpm --dir app exec tsc --noEmit -p e2e
pnpm --dir app exec playwright test --project=e2e -g "natal extras" 2>&1 | tail -1
pnpm --dir app e2e 2>&1 | tail -1
rm -rf app/test-results
grep -nE 'console\.|localStorage|sessionStorage|new Date\(|Date\.now|\bfetch\(' app/ui/panels/VimshopakaPanel.tsx
git status --short
```
Expected: `3 passed (3)`, `44 passed (44)`; tümü `91 passed (91)`, `910 passed (910)`; `✓ built`; `lsof` çıktısız (doluysa başka bir worktree'nin e2e koşusu beklenir; Global Constraints); `tsc` çıktısız; `1 passed`; tümü `28 passed`; tarama çıktısız; `git status --short` yalnızca bu görevin on iki dosyasını gösterir.

- [ ] **Step 6: Commit**

```bash
git add app/ui/modules/useNatalExtras.ts app/ui/panels/VimshopakaPanel.tsx app/ui/panels/VimshopakaPanel.test.tsx app/ui/panels/registry.ts app/ui/panels/panels.test.tsx app/ui/layout/model.ts app/ui/layout/model.test.ts app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts app/e2e/ipc.ts app/e2e/desk.spec.ts
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): Vimshopaka bala panel with the vargas view; end-to-end flow for the natal extras" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: on iki dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 8: Kapılar, REVIEW.md'nin elle denetim kaydı ve sahibine devir

Yol haritası görev taslağının (8). maddesi ve "Sahibin denetimi" paragrafı; Y2–Y4. Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle çözülür ve bu görev baştan koşulur. Bu görevin tek dosyası `REVIEW.md`'dir: elle denetim kaydı ona eklenir ve tek bir belge commit'i üretilir. Gerçek uygulama çalıştırılmaz: **[insan]** maddeleri sahibine devredilir ve sonraki planı engellemez (Y3). Aşağıda `<TABAN>`, Task 1 Step 1'de `git log -1 --format=%h` ile okunan commit'tir (bu plan yazılırken `7270550`); komutlarda `BASE` değişkeniyle verilir.

**Files:**
- Modify: `tests/golden/modules/natal_extras/REVIEW.md`

**Interfaces:**
- Consumes: Task 1–7'nin tamamı.
- Produces: yeşil kapılar, kural ve gizlilik taramaları, `REVIEW.md` §4 (elle denetim kaydı: **[otomatik]** maddeler işaretli, **[insan]** maddeler sahibine devredilmiş) ve §5 (gözden geçirme kaydı tablosu, boş).

- [ ] **Step 1: Rust kapıları**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
BASE=7270550
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
cargo test -p jyotish-desk --test natal_extras -- --nocapture 2>&1 | grep -E "^compute_"
cargo deny check 2>&1 | tail -1
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p jyotish-core -p atlas -p store -p jyotish-desk --all-targets 2>&1 | tail -1
git diff $BASE..HEAD --stat -- deny.toml Cargo.toml Cargo.lock app/package.json app/pnpm-lock.yaml app/vite.config.ts app/src-tauri/capabilities app/src-tauri/tauri.conf.json app/src-tauri/Cargo.toml crates/store crates/atlas crates/ephemeris crates/jyotish-core/Cargo.toml crates/jyotish-core/src/chart.rs crates/jyotish-core/src/positions.rs crates/jyotish-core/src/settings.rs crates/jyotish-core/src/panchang.rs crates/jyotish-core/src/error.rs crates/jyotish-core/src/dasha.rs crates/jyotish-core/src/dignity.rs crates/jyotish-core/src/varga.rs crates/jyotish-core/src/transit.rs crates/jyotish-core/src/muhurta crates/jyotish-core/src/kuta crates/jyotish-core/src/shadbala crates/jyotish-core/src/jaimini crates/jyotish-core/src/yogas crates/jyotish-core/src/prashna crates/jyotish-core/src/search crates/jyotish-core/tests app/src-tauri/src/commands.rs app/src-tauri/src/module_commands.rs app/src-tauri/tests/modules.rs tests/golden/modules/README.md
ls crates/jyotish-core/proptest-regressions 2>/dev/null
```
Expected: fmt ve clippy çıktısız; testler: `jyotish-core` lib `429 passed`, golden `8 passed`, `modules_golden` `17 passed`; `atlas` `moment_fixed_lmt` `11 passed`, `moment_iana` `13 passed`; `store` lib `122 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `58 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `modules` `1 passed`, `natal_extras` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; sayaç `0`; üç ölçüm satırı, hepsi 20 ms'nin altında (D17; bütçeyi aşan değer testi zaten düşürür; yük altında düşen bütçe testi makine boşken yeniden koşulur); deny `advisories ok, bans ok, licenses ok, sources ok`; MSRV `Finished`; `git diff --stat` çıktısız (bağımlılıklar, yetenek dosyası, var olan hesap modülleri, hata türleri, var olan komutlar, var olan entegrasyon ve golden testleri ve `store` değişmedi); `ls` çıktısız. Taban `7270550` değilse (öteki planlardan biri önce birleştiyse) sayılar Task 1 Step 1'de ölçülen tabandan aynı farklarla okunur.

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
Expected: `lsof` çıktısız (doluysa başka bir worktree'nin koşusu beklenir; süreç öldürülmez); `Test Files  91 passed (91)`, `Tests  910 passed (910)`; `✓ built`; e2e `28 passed`; görsel `10 passed` (temel PNG'ler değişmedi: D25); betik testleri `pass 14`, `fail 0`; `git status --short` boş.

- [ ] **Step 3: Kural, gizlilik ve "yalnızca ek" taramaları (tabandan bu yana)**

```bash
BASE=7270550
git diff $BASE..HEAD -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log $BASE..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff $BASE..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
git diff $BASE..HEAD --name-only | grep -vE '^(app|crates|tests)/'
echo "trailers: $(git log $BASE..HEAD --format=%B | grep -c '^Co-Authored-By: ') commits: $(git rev-list --count $BASE..HEAD)"
git diff $BASE..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
git diff $BASE..HEAD -U0 -- crates app/src-tauri/src | grep '^+' | grep -E 'println!|eprintln!|dbg!|log::|tracing::'
git diff $BASE..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\(|window\.(print|open)\('
git diff $BASE..HEAD --stat -- 'app/ui/**/*.css' app/ui/chart app/ui/print app/e2e/visual.spec.ts-snapshots
grep -rnE '#\[serde\(default' crates/jyotish-core/src/natal_extras app/src-tauri/src/natal_extras_commands.rs
grep -rnE 'XMLHttpRequest|WebSocket|\bfetch\(|localStorage|sessionStorage|new Date\(|Date\.now' app/ui/modules/useNatalExtras.ts app/ui/panels/PanchangExtras.tsx app/ui/panels/NavataraPanel.tsx app/ui/panels/VimshopakaPanel.tsx
grep -cE 'natal_extras_commands::(compute_panchang_extras|compute_navatara|compute_vimshopaka),' app/src-tauri/src/lib.rs
grep -c 'CommandError::new(' app/src-tauri/src/natal_extras_commands.rs
grep -rnE 'InvalidTransitInput\((format!|&format|String)' crates app/src-tauri/src
git diff $BASE..HEAD -U0 -- crates/jyotish-core/src/lib.rs app/src-tauri/src/lib.rs app/src-tauri/tests/ui_fixture.rs app/ui/api.ts app/ui/api.test.ts app/ui/test/fixture.ts app/ui/layout/model.ts app/ui/panels/registry.ts app/ui/panels/panels.test.tsx app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts app/ui/App.test.tsx app/e2e/ipc.ts app/e2e/desk.spec.ts | grep -c '^-[^-]'
git diff $BASE..HEAD -U0 -- app/ui/layout/model.test.ts | grep -c '^-[^-]'
git tag --points-at HEAD
```
Expected: `trailers:` satırındaki iki sayı eşit; `grep -ciE` satırı `0`; komut sayacı `3` (üç komut kayıtlıdır); `CommandError::new` sayacı `0` (her hata `CoreError` eşlemesinden geçer; yeni kod yoktur: D20); "yalnızca ek" sayacı `0` (paylaşılan on beş dosyada tabana göre silinen ya da yeniden sarılan satır yok); `model.test.ts` sayacı `1` (tek satırlık `MODULE_PANELS` beklentisi). Bunların dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur): yasaklı ad yok (üretilen fikstürde de), `app/`, `crates/` ve `tests/` dışında dosya yok, arayüzde `console` ve Rust'ta log çağrısı yok (`eprintln!` yalnızca `app/src-tauri/tests/` altındadır ve taranan yolların dışındadır), yasak çağrı yok, stil dosyası, harita çizimi, baskı ve görsel temeller değişmedi (D25), IPC tiplerinde `serde(default)` yok, yeni yüzeylerde ağ, tarayıcı deposu ve saat çağrısı yok (C4), hata alan adı çalışma anında kurulmuyor, etiket (git tag) yok.

- [ ] **Step 4: REVIEW.md: elle denetim kaydı**

Kayıt `REVIEW.md`'nin sonuna eklenir. Her **[otomatik]** madde, adı geçen testler bu oturumda (Step 1 ve Step 2) yeşil görüldükten sonra `- [x]` yapılır; düşen bir test varsa madde işaretlenmez, düzeltme commit'i atılır ve bu görev baştan koşulur. **[insan]** maddeleri ve §1.5, §2.2, §3.4'teki onay noktaları **işaretlenmez**: yalnızca sahibi, basılı kaynağa ya da gerçek uygulamaya bakarak işaretler. Yürüten ajan tabloları başka bir yazılımla ya da çevrimiçi kaynakla karşılaştırmaz (Y4); gerçek uygulamayı (`pnpm --dir app tauri dev`, üretilen `.app`) çalıştırmaz.

**Dosyanın sonuna, önüne bir boş satır konarak eklenir:** `tests/golden/modules/natal_extras/REVIEW.md`

````markdown
## 4. Elle denetim kaydı (Plan 23 Task 8)

Her maddenin başında, otomatik bir karşılığı olup olmadığı yazılıdır. **[otomatik]** maddelerin
karşılığı testlerdedir; yürüten ajan testin adını göstererek işaretler. **[insan]** maddeler
yalnızca basılı kaynağa ya da gerçek uygulamaya bakan bir insan tarafından doğrulanır ve sahibine
devredilir; devir sonraki planı engellemez (Y3). Gerçek uygulamada yalnızca deneme danışanları
kullanılır.

- [ ] **[otomatik]** Yogi noktası Güneş + Ay + 93°20′'dir ve 360°'de sarılır; yogi nakshatrası
  nitya yogadan yedi ileridedir; üç el örneği (A, B, C) noktayı, yogiyi, eş yogiyi ve avayogiyi
  verir. (`yogi.rs › the_arc_is_seven_nakshatras_and_the_point_wraps`,
  `› hand_examples_give_the_point_the_yogi_the_co_yogi_and_the_avayogi`,
  `› the_yogi_nakshatra_is_the_nitya_yoga_shifted_by_seven`; `tests/natal_extras.rs`)
- [ ] **[otomatik]** Avayogi altıncı nakshatranın lordudur; ikinci yazım (nokta + 186°40′) aynı
  lordu verir; dokuz yogi–avayogi çifti 1.2'deki tabloyla aynıdır.
  (`yogi.rs › avayogi_is_the_sixth_nakshatra_and_agrees_with_the_added_arc`,
  `› nakshatra_lords_repeat_every_nine`)
- [ ] **[otomatik]** Eş yoginin burcu rasi çerçevesindedir; `Sidereal` kipinde öteleme sıfırdır;
  nakshatra ve lordlar kipten bağımsızdır. (`yogi.rs › the_point_sign_follows_the_rasi_frame`;
  `collect.rs › panchang_extras_of_the_fake_sky`; `tests/natal_extras.rs`, tropikal-rasi kipindeki örnekler)
- [ ] **[otomatik]** Dagdha rasi tablosu burçtan yazılan ikinci tabloyla aynıdır (30 hücre); iki
  paksha aynı satırı okur; dolunay ve yeni ayda boştur; aralık dışı tithi boş liste verir.
  (`dagdha.rs › two_signs_on_thirteen_tithis_four_on_the_fourteenth_none_on_the_fifteenth`,
  `› the_table_agrees_with_its_second_writing_by_sign`, `› out_of_range_tithis_have_no_dagdha_sign`)
- [ ] **[otomatik]** Raporun tithisi v1'in panchangındaki tithidir; Delhi 2000-01-01 için yogi
  Rahu (Swati), avayogi Venüs (Purva Ashadha), dagdha Yay ve Balık.
  (`dagdha.rs › extras_join_the_yogi_facts_the_tithi_and_the_dagdha_signs`; `tests/natal_extras.rs`)
- [ ] **[otomatik]** Navatara 27 satırdır, doğum nakshatrasından başlar, üç tur yürür; tara ve tur
  adım adım sayımla aynıdır (729 çift); her graha kendi nakshatrasının satırındadır; geçiş taraları
  transit komutunun nakshatralarından sayılır. (`navatara.rs` beş test;
  `collect.rs › navatara_of_the_fake_sky_with_and_without_a_transit_moment`; `tests/natal_extras.rs`)
- [ ] **[otomatik]** Vimshopaka: dört kümenin ağırlıkları 20'ye toplanır; altı basamak 20, 18, 15,
  10, 7, 5'tir; yücelme ve moolatrikona ayrı basamak değildir; geçici dostluk D1'den okunur; iki el
  örneği (Güneş Aslan 0°, Ay Boğa 0°) dört değeri verir; her değer 5 ile 20 arasındadır.
  (`vimshopaka.rs` altı test)
- [ ] **[otomatik]** Vimshopaka'nın varga burçları v1'in on altı varga haritasıyla, basamakları
  Shadbala'nın saptavargaja kalemiyle (yedi vargada) aynıdır. (`tests/natal_extras.rs`)
- [ ] **[otomatik]** Doğum saati bilinmiyorsa üç rapor da döner ve `approximate` işaretlidir;
  sayı prashnasında raporlar sayısız haritayla aynıdır. (`collect.rs` dört test;
  `PanchangExtras.test.tsx › unknown birth time …`, `NavataraPanel.test.tsx › unknown birth time …`,
  `VimshopakaPanel.test.tsx › unknown and approximate birth times …`)
- [ ] **[otomatik]** Geçiş anı haritadan önce denetlenir; hatası yalnızca alan adını taşır
  (`invalid_transit_input { field: transit_jd_ut }`). (`collect.rs ›
  collectors_check_the_input_before_and_through_the_chart`; `tests/natal_extras.rs`)
- [ ] **[otomatik]** Üç komut hata ayıklama derlemesinde 20 ms'nin altındadır.
  (`tests/natal_extras.rs`: `assert!`)
- [ ] **[otomatik]** Raporlar panel açılmadan istenmez; Panchang paneli beş uzvu komut beklemeden
  çizer ve ek tabloyu altına ekler; ek tablonun hatası beş uzvu kaldırmaz.
  (`PanchangExtras.test.tsx` yedi test; e2e `natal extras: …`)
- [ ] **[otomatik]** Navatara paneli dokuz tarayı üç turda, lord ve kümeyle yazar; transit anını
  Rust'a olduğu gibi gönderir; an değişirken tablolar yerinde durur; transit hatasında doğum tablosu
  durur. (`NavataraPanel.test.tsx` sekiz test; e2e)
- [ ] **[otomatik]** Vimshopaka paneli yedi grahanın dört değerini üç ondalıkla yazar; "Vargalar"
  görünümü on altı vargada burcu ve basamağı gösterir; ayar düzenle birlikte saklanır.
  (`VimshopakaPanel.test.tsx` sekiz test; e2e)
- [ ] **[otomatik]** Her metin üç dilde vardır; klasik adlar (tara, nakshatra, küme adları) üç
  dilde aynıdır; Fransızca metinlerde düz kesme işareti yoktur. (`i18n.test.ts`; üç panel testinin
  dil testleri)
- [ ] **[insan]** Dagdha rasi tablosu basılı kaynakla hücre hücre karşılaştırılır (1.3, 1.5);
  paksha ayrımı olup olmadığına bakılır.
- [ ] **[insan]** Yogi noktasının yayı (93°20′) ve avayoginin sayımı basılı kaynakla doğrulanır
  (1.5).
- [ ] **[insan]** Vimshopaka'nın dört ağırlık sütunu ve altı basamağı basılı kaynakla
  karşılaştırılır (3.2, 3.3); yücelme ile moolatrikonanın yeri ve geçici dostluğun okunduğu varga
  için karar verilir (3.4).
- [ ] **[insan]** Navataranın lagna nakshatrasından da sayılıp sayılmayacağına karar verilir (2.2).
- [ ] **[insan]** Gerçek uygulamada Panchang paneli: ek tablo beş uzvun altındadır ve panelin
  varsayılan boyutunda kaydırmayla okunur; boyutun büyütülmesi gerekiyorsa karar olarak yazılır.
- [ ] **[insan]** Gerçek uygulamada Navatara paneli: transit anı değiştirilince geçiş tablosu
  değişir; "Şimdi" çalışır; nakshatralar transit tablosundakilerle aynıdır.
- [ ] **[insan]** Gerçek uygulamada Vimshopaka paneli: saati "yaklaşık" girilmiş bir deneme
  haritasında not görünür; "Vargalar" tablosu dar panelde yana kayar; açık ve koyu temada, üç
  dilde okunur.
- [ ] **[insan]** VoiceOver: üç tablo başlıklarıyla duyurulur; kısaltmayla yazılan grahalar tam
  adıyla okunur; "doğrulanmamış tablo" notları okunur.
- [ ] **[insan]** Denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum verisi,
  koordinat ya da dosya yolu görünmez.

## 5. Gözden geçirme kaydı

| Tarih | Gözden geçiren | Kapsam | Sonuç |
|---|---|---|---|
| | | El örneklerinin kuraldan yeniden türetilmesi (yogi A–C, navatara tabanı, Vimshopaka'nın iki örneği) | |
| | | Sahibinin basılı kaynak onayı (1.5, 2.2, 3.4) | |
````

```bash
grep -c '^- \[.\] \*\*\[otomatik\]\*\*' tests/golden/modules/natal_extras/REVIEW.md
grep -c '^- \[x\] \*\*\[otomatik\]\*\*' tests/golden/modules/natal_extras/REVIEW.md
grep -c '^- \[ \] \*\*\[insan\]\*\*' tests/golden/modules/natal_extras/REVIEW.md
grep -c '^- \[ \] ' tests/golden/modules/natal_extras/REVIEW.md
grep -inE 'k(a|ā|aa)la|\bcala\b' tests/golden/modules/natal_extras/REVIEW.md
```
Expected: `15`, `15` (on beş otomatik maddenin hepsi işaretli), `9` (dokuz insan maddesi açık), `22` (dokuz insan maddesi ve §1.5, §2.2, §3.4'ün on üç onay noktası açık); son komut çıktısız.

- [ ] **Step 5: Commit ve ilerleme notu**

```bash
git add tests/golden/modules/natal_extras/REVIEW.md
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "docs(review): manual check record for the natal extras; human items handed to the owner" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: bir dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

İlerleme notlarına (depo dışında; `git add` edilmez) `Plan 23 Task 8: kapılar geçti; otomatik maddeler işaretlendi, insan maddeleri ve tablo onayları sahibine devredildi (<tarih>)` yazılır. Sahibine devredilenler, yol haritasının "Sahibin denetimi" paragrafındaki dört başlıktır: dagdha rasi tablosu (hücre hücre; paksha ayrımı); yogi noktasının sabit yayı ve avayoginin sayımı; Vimshopaka'nın dört ağırlık tablosu ve puan basamakları (yücelme ve moolatrikonanın yeri, geçici dostluğun okunduğu varga); navataranın lagnadan da sayılıp sayılmayacağı. Onay gelince ilgili not (`panchangExtras.unverified`, `vimshopaka.unverified`) ayrı bir commit'le kaldırılır; onay bir hücreyi değiştirirse önce `REVIEW.md` ve test (ikinci yazım dahil), sonra tablo değişir ve yöntem kimliği `v2` olur.

---

## Başarı ölçütleri

- Panchang paneli beş uzvun altında yogi noktasını, yogiyi, eş yogiyi, avayogiyi ve dagdha burçları gösterir; değerler ayrı bir komuttan gelir ve `Chart` ile `panchang.rs` değişmez.
- Navatara paneli doğum Ay'ının nakshatrasından dokuz tarayı üç turda, lordu ve orada duran doğum grahalarıyla yazar; seçili transit anındaki dokuz grahanın tarasını Rust'tan alır.
- Vimshopaka paneli yedi grahanın dört kümedeki değerini (5–20) ve on altı vargadaki burcunu ve basamağını gösterir; ağırlıkların toplamı her kümede 20'dir.
- Her kuralın el örneği ve ikinci yazımı vardır; gerçek motorla sonuçlar v1'in panchangı, varga haritaları ve Shadbala'nın saptavargaja kalemiyle çapraz denetlenir; üç komut 20 ms'nin altındadır.
- Doğum saati bilinmiyorsa üç rapor da döner ve "yaklaşık" bilgisini Rust verir; hiçbir rapor lagna istemez.
- Bellekten yazılan tablolar `REVIEW.md`'de onay bekler ve panellerde "doğrulanmamış tablo" notuyla sunulur.
- Paylaşılan dosyalara yalnızca ek yapılmıştır (sayaç `0`); yeni hata kodu, yeni bağımlılık, yeni stil dosyası, şema değişikliği ve yeni görsel temel yoktur.
- Sayaçlar: `jyotish-core` lib 429, `jyotish-desk` `natal_extras` 1, Vitest 91 dosya / 910 test, e2e 28.

## Bu planın kapsamadıkları

- Navataranın lagna nakshatrasından sayımı (ikinci okuma; D8); taraların yorumu; tara dashası.
- Dagdha burçlarda duran grahaların, dagdha burç lordlarının ya da yogi ile avayoginin haritadaki durumunun (ev, dignity, dasha dönemi) raporlanması; yogi noktasının harita çiziminde işaretlenmesi.
- Vimshopaka'da yücelme ve moolatrikona için ayrı basamak, geçici dostluğun vargadan okunması (ikinci okumalar; D13, D14); Rahu ve Ketu'ya değer; varga sayımları (parijatamsa ve benzeri adlandırmalar); değerlerin sınıflandırılması ya da sıralanması.
- Panchang eklerinin gün panchangı panelinde (geçiş günü için) gösterilmesi; navatara ve Vimshopaka'nın harita aramasına koşul olarak eklenmesi (Plan 22'nin koşul listesi değişmez).
- Graha tablosuna Vimshopaka ya da tara sütunu; nakshatra lordu sütunları (yol haritası §4.7, Plan 26).
- Panchang panelinin varsayılan boyutunun ve hazır düzenlerin değiştirilmesi (D21; Task 8'in **[insan]** maddesi).
- `tests/golden/modules/` altında JSON el örnekleri ve `modules_golden.rs` testleri (D18).
- Baskı düzeni (K10), yorum metni ve AI paneli (Y7); Windows ve Android.

## Yol haritasına geri bildirim (farklar ve boşluklar)

Kararlar bölümünde işaretlenen maddeler; yol haritası bir sonraki düzenlemede bunlara göre güncellenmelidir.

1. **(D1) Modül adı ve yöntem kimlikleri.** Yol haritası çekirdeği `panchang_extras` diye anar; modül `natal_extras`'tır ve üç kalemi taşır. Yöntem kimlikleri `panchang_extras.v1`, `navatara.moon_v1`, `vimshopaka.bphs_v1`'dir.
2. **(D2) Eş yoginin çerçevesi.** Yogi noktasının burcunun rasi çerçevesinde (K5), nakshatrasının sidereal boylamdan okunduğu yazılmalıdır; tropikal-rasi kipinde eş yogi sidereal burcun lordundan farklı olabilir.
3. **(D6) "Panchang paneline ek satırlar".** Ekler aynı tablonun satırları değil, panelin ikinci tablosudur ve ayrı bir komuttan gelir; beş uzuv komut beklemez.
4. **(D11) Geçiş tarasının girdisi ve hatası.** Komut `compute_navatara(input, transit_jd_ut)`'dir; an transit deposundan gelir; geçersiz an `invalid_transit_input { field: transit_jd_ut }` ile reddedilir. Panel transit anı seçicisini de taşır.
5. **(D17) Bütçe.** Üç komut için ≤ 20 ms (hata ayıklama derlemesi; boş makinede ölçüm ≈ 0,6 ms). İM spec §15'in tablosuna eklenmelidir.
6. **(D13) Ay'ın moolatrikona burcu.** "Yücelme ve moolatrikonanın ayrı basamak olup olmadığı" varyantının tek okuması, Ay'ı Boğa'da Venüs'le ilişkisinden okutur; yol haritası bu sonucu açıkça söylemelidir, çünkü sahibin onayında en çok soru çıkaracak hücre budur.
7. **(D16) Gösterim.** Küme değerleri üç ondalıkla yazılır (0,025'in katı); yol haritası biçim yazmaz.
8. **(D18) El örneklerinin yeri.** Yol haritası §4 "el örnekleri"ni İM spec §13'e bağlar (JSON fikstür); bu planda örnekler birim testidir ve onay kaydı tek bir `REVIEW.md`'dir. Plan 24 ve 25 aynı anda yürürken `modules_golden.rs`'in ortak dosya olması bunun nedenidir; planlar birleştikten sonra örneklerin JSON'a taşınması ayrı bir karardır.
9. **(§8 "Denetim birikimi")** Bu plan iki tablo ailesi ekler (dagdha rasi; Vimshopaka ağırlıkları ve basamakları) ve iki panelde "doğrulanmamış tablo" notu gösterir; yol haritasının sayımı ("en az dokuz tablo") buna göre güncel tutulmalıdır.

## Self-Review

1. **Yol haritası kapsamı:** §4.1–§4.3'ün her maddesi ve §5 "Plan 23" görev taslağının sekiz maddesi "Kapsam" tablosunda bir göreve bağlıdır; görev başlıkları taslağın sırasını izler. Taslaktan bilerek ayrılan yerler Kararlar'da işaretlidir (D6: ek satırlar yerine ikinci tablo; D18: el örnekleri birim testi) ve geri bildirim listesindedir. "Sahibin denetimi" paragrafının dört başlığı `REVIEW.md` §1.5, §2.2 ve §3.4'te onay noktası, §4'te **[insan]** maddesidir.
2. **Yer tutucu taraması:** planda "TBD", "TODO", "sonra doldurulur", "uygun hata işleme ekle" ya da kodu olmayan kod adımı yoktur; her kod adımı tam dosya, uygulanabilir `diff` ya da dosya sonuna eklenecek tam metin taşır. `<TABAN>` ve `<tarih>` yürütme anında okunan değerlerdir (Task 8).
3. **Tip ve ad tutarlılığı:** `YogiFacts`, `PanchangExtras`, `NavataraRow`, `TransitTara`, `Navatara`, `VargaScheme`, `VimshopakaVarga`, `GrahaVimshopaka`, `VimshopakaReport` Task 1–4'te tanımlanır ve Task 5–7'de aynı adlarla kullanılır. Komut adları Rust'ta `snake_case`, arayüzde `camelCase`'tir (`compute_navatara` / `computeNavatara`; bağımsız değişken `transit_jd_ut` / `transitJdUt`). Bağlamalarda `f64` alanları `number | null`'dır; paneller boş değeri çizgiyle yazar. Arayüzün sabit listesi (`VIMSHOPAKA_SCHEMES`) fikstürdeki Rust değerleriyle testte karşılaştırılır.
4. **Dersler:** hata yalnızca sabit alan adı taşır ve geçiş anı haritadan önce denetlenir (D11; `collect.rs` ve `tests/natal_extras.rs`); girdi ve raporlar loglanmaz (Task 8 Step 3); "yaklaşık" bilgisi Rust'tan gelir (D7); yeniden okumada tablo sökülmez (`useLastData`; `NavataraPanel.test.tsx › a new transit moment …`); hatadan sonraki okuma hata öncesi veriyi göstermez; raporlar panel açılmadan istenmez (e2e); bütçeler `assert!` ile doğrulanır (D17); bellekten yazılan tablo ikinci yazımıyla sınanır ve onaya dek notla sunulur (Y4); üretilen dosyalar elle düzenlenmez (C10); eşzamanlı worktree'lerin e2e portu paylaştığı Global Constraints'te yazılıdır.
5. **Koşulmuş kod:** bu plandaki her kod bloğu, `7270550`'nin bir kopyasında görev başına bir commit olarak uygulanmış ağaçtan alınmıştır (bloklar o commit'lerin dosyalarından ve `git diff` çıktılarından üretilmiştir). Task 1–4 commit'lerinde `cargo test -p jyotish-core --lib` (410, 414, 419, 425) ve `cargo clippy -p jyotish-core --all-targets -- -D warnings`; son ağaçta `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (başarısız sonuç satırı 0; lib 429, `natal_extras` 1), `bindings` ve `ui_fixture`, Vitest (Task 6 ağacında 90 / 902, son ağaçta 91 / 910), `pnpm build`, e2e (28), görsel (10), `cargo deny check`, 1.93 ile `cargo check` ve yayın betiği testleri (14) koşulmuş ve geçmiştir. Task 6 ve Task 7'nin "testlerin düştüğünü gör" adımları da kopyada koşulmuştur (beklenen çıktılar oradan yazılmıştır). E2e ve görsel testler kopyada ayrı bir portta koşulmuştur (1421'i başka worktree'lere bırakmak için); görsel karşılaştırma depodaki temellerle yapılmıştır. Son olarak bu belgedeki bloklar (yeni dosyalar, `diff` blokları ve dosya sonu ekleri) sırasıyla boş bir kopyaya yeniden uygulanmış, iki üretilen dosya komutla yeniden üretilmiş ve ortaya çıkan ağaç doğrulanmış ağaçla karşılaştırılmıştır: fark yoktur. İki küçük değişiklik bu son koşulardan sonra yapılmıştır ve yalnızca Rust testleriyle (`ui_fixture`, `natal_extras`, clippy) yeniden doğrulanmıştır: `ui_fixture.rs`'te `VargaScheme` içe aktarımının ayrı satıra alınması ve bütçenin 10 ms'den 20 ms'ye çıkarılması (D17); ikisi de arayüz ağacını değiştirmez. Task 1–5'in "testlerin derlenmediğini gör" adımları da kopyada koşulmuştur (beşi de `error[E0432]` ile durur). Koşulmayanlar: commit adımlarının trailer sayacı (kopyadaki commit'ler bu satırı taşımaz) ve **[insan]** maddeleri; gerçek uygulama çalıştırılmamıştır.
6. **Paylaşılan dosyalar:** "Paylaşılan dosyalar" tablosundaki on beş dosyada tabana göre silinen satır yoktur (kopyada sayaç `0`); tek istisna `model.test.ts`'in tek satırlık listesidir ve tabloda yazılıdır. Plan 24 ve Plan 25'in belgeleri bu plan yazılırken görülmemiştir: dokunacakları dosyalar tahmindir ve birleştirme kuralı o yüzden geneldir.
7. **Yasaklı ad:** bu belge ve bütün kod blokları `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile taranmıştır; çıktı boştur.
