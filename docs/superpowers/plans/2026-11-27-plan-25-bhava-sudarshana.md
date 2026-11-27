# Plan 25 — Bhava chalit, ev başlangıcı dizisi ve Sudarshana chakra (bhava tablosu çekirdeği ve paneli; harita panelinde chalit görünümü; Sudarshana chakra çekirdeği, çizimi ve paneli)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Yol haritasının üçüncü planı (kalem 4.5 ve 4.6). (1) **Bhava tablosu:** haritanın ev sisteminde on iki bhavanın başlangıcı, ortası ve bitişi (burç, derece, nakshatra, pada, burç lordu, nakshatra lordu), her grahanın bhavası ve bhava başlangıcına ve ortasına uzaklığı; yeni `bhava_table` paneli. (2) **Chalit görünümü:** harita panelinde grahaları burca göre değil bhavaya göre yerleştiren görünüm (yalnızca D1). (3) **Sudarshana chakra:** lagna, Ay ve Güneş'i birinci ev sayan üç iç içe halka; yeni `sudarshana` paneli (çizim ve tablo karşılığı). Üç katman: `jyotish-core`'da yeni `bhava` modülü (iki saf hesap ve iki ince toplayıcı); uygulamada iki komut (`compute_bhava_table`, `compute_sudarshana`); arayüzde iki panel, harita panelinin bir ayarı ve yeni bir SVG bileşeni. Yorum, puan ve hüküm yoktur. Veritabanı şeması, `Chart` tipi, hata türleri ve bağımlılıklar değişmez.

**Architecture:**
- **`Chart` dondurulmuştur (K1).** Bhava tablosu ve chakra istek üzerine hesaplanır ve saklanmaz. Toplayıcılar temel haritayı (`compute_chart_for`) hesaplar ve saf fonksiyona verir (K3): `bhava_table(...)` ve `sudarshana(&Positions)` ephemeris görmez.
- **Bhava ortası ev sistemine göre okunur.** Tam burçta lagnanın derecesi (Bhava bala ile aynı nokta); Sripati'de Porphyry noktası (lagna ile MC'den saf aritmetikle kurulur; yeni FFI yok); Placidus'ta tanımlanmaz (`None`). Sripati'nin ikinci yazımı (her sandhi iki komşu ortanın ortasıdır) gerçek motorla sınanır.
- **Chalit görünümü yeni komut istemez.** Grahanın bhavası haritada zaten vardır (`Chart.grahas[].house`, `house_from_cusps`). Arayüz her grahayı o numaralı rasi evinin hücresine çizer (gösterim eşlemesi); hücreler, burç numaraları ve geometri değişmez. Yer değiştiren graha `*` ile işaretlenir ve sözle okunur.
- **Sudarshana çizimi tek bir resimdir.** 36 hücre ve 27 etiket sekme durağı olmaz; klavye ve ekran okuyucu yolu panelin tablosudur (burç ve graha düğmeleri ortak vurguyu sürer). Fare çizimde de çalışır.
- **Arayüz hesap yapmaz (C4).** Burç, derece, nakshatra, lordlar, bhava, uzaklık, halka evleri ve "yer değiştirdi" bilgisi Rust'tan gelir. Arayüz biçimler, kimliği ada çevirir ve ev numarasını hücreye eşler.
- **Paralel planlarla çakışmayı azaltan yerleşim.** Yeni çekirdek kod yeni modüldedir; komutlar yeni dosyadadır (`bhava_commands.rs`); gerçek motor testi yeni dosyadadır (`tests/bhava.rs`); el örnekleri Rust birim testleridir (`modules_golden.rs` ve `modules.rs` değişmez); yeni hata türü yoktur (`error.rs`, `commands.rs`, `i18n.test.ts` değişmez). Paylaşılan dosyalardaki her ekleme küçüktür ve "Paylaşılan dosyalar" bölümünde satırıyla yazılıdır.
- **Testler:** çekirdekte el örnekleri (Sripati tablosu, tam burç, Placidus, nokta okuması, lord sırası, üç halka) ve toplayıcı testleri; uygulamada gerçek motorla beş golden anı × üç ev sistemi × iki zodyak kipi; arayüzde Vitest (iki panel, chalit görünümü, geometri) ve üç uçtan uca akış; beş yeni görsel temel.

**Tech Stack:** Rust 1.93+ (stable; yerelde ve CI'da 1.100), Tauri 2.12, tauri-specta 2.0.0-rc.29, serde, React 19, TypeScript 7, Vitest 5, Playwright 1.65.1. **Yeni Rust ya da npm bağımlılığı yok.**

**Spec:** `docs/superpowers/specs/2026-11-27-eksikler-yol-haritasi.md`: §2 (Y1–Y8), §4.5, §4.6, §5 "Plan 25". Değişmeyen kurallar: `2026-11-26-alt-proje-3-ileri-moduller-design.md` §2 (K1–K12), §13 (test stratejisi), §14 (gizlilik), Ek A (adlandırma). UI spec: `2026-11-24-ui-design.md` §2.3 (retro düz `R`), §3.3–§3.4 (panel kataloğu, ortak vurgu), §5 (yükleme evreleri), §6 (erişilebilirlik), §8 (panel durumları).

**Ön koşul:** Taban `main` commit `7270550`'dir (yol haritası commit'i; Plan 22'nin son ucu). Kod blokları ve sayaçlar o ağaca göre yazılmış ve deponun bir kopyasında, her görev ayrı commit olarak uygulanıp ölçülmüştür. Bu plan ayrı bir çalışma ağacında yürütülür; Plan 23 (panchang ekleri, navatara, Vimshopaka) ve Plan 24 (avasthalar) başka çalışma ağaçlarında aynı tabandan paralel yürür. Çalışma ağacı `7270550`'den ilerlemişse Task 1 Step 1 tabanları yeniden ölçer ve sayaç zinciri o ölçümden, aynı farklarla yürütülür; bir `diff` bloğunun bağlamı tutmazsa önce o dosya okunur ve ekleme "Paylaşılan dosyalar" bölümündeki çapa satırına göre elle yapılır.

**Yürüten ajan için (bu plan kendi başına yeterlidir):** Her görevde adımlar sırayla uygulanır. "**Yeni dosya:**" bloğu dosyanın tam içeriğidir. `diff` bloğu depo kökünde bir dosyaya kaydedilip `git apply <dosya>` ile ya da elle uygulanır. Üretilen iki dosya (`app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`) hiçbir zaman elle düzenlenmez; yalnızca şu iki komutla yeniden üretilir: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings` ve `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture`. Gerçek uygulama (`pnpm tauri dev`) hiçbir adımda çalıştırılmaz. `.superpowers/` dizini hiçbir zaman `git add` edilmez. Commit iletisinin son satırı tam olarak `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` satırıdır.

## Kararlar

Sahibi ürün kararlarını devretti; muhafazakâr seçenek seçildi ve her karar bir satır gerekçeyle yazıldı. "Yol haritası farkı" diye işaretlenenler, yol haritasındaki görev taslağından bilerek ayrılan yerlerdir. D2–D4, D12 ve D14–D15 sahibin onayını bekleyen okuma seçimleridir ve Task 8'in `REVIEW.md` dosyasında varyantlarıyla yazılıdır. D16'daki ölçüm ve sayaç zinciri bu plan yazılırken deponun bir kopyasında alınmıştır (macOS 27.2, Rust 1.100, hata ayıklama derlemesi).

- **D1 — Çekirdek modülü `crates/jyotish-core/src/bhava/`'dır: `table.rs` (bhava tablosu) ve `sudarshana.rs` (chakra). Yöntem kimlikleri `BHAVA_METHOD = "bhava_table.v1"` ve `SUDARSHANA_METHOD = "sudarshana.v1"`'dir. `lib.rs` kaydı dosyanın başında, ayrı bir blokta durur.** Gerekçe: K7 her sonucun yöntem kimliği taşımasını ister. `rustfmt` aynı gruptaki `mod` ve `use` satırlarını alfabetik sıraya zorlar; ayrı blok, paralel planların alfabetik eklemeleriyle aynı satırlara dokunmaz.
- **D2 — Sripati'de ephemeris'in döndürdüğü on iki nokta bhava başlangıcıdır (sandhi); bhava ortası Porphyry noktasıdır ve lagna ile MC'den saf aritmetikle kurulur (`porphyry_points`): MC → lagna yayı ve lagna → IC yayı üçe bölünür, öteki altı nokta karşılarıdır. Toplayıcı MC için evleri bir kez daha ister; `ephemeris` kasasına yeni ev sistemi ya da FFI eklenmez.** Gerekçe: yol haritası §4.5 "orta ayrıca hesaplanır". `Chart` MC taşımaz ve dondurulmuştur (K1). İkinci yazım (`sandhis_from_mids`: her sandhi iki komşu ortanın ortasıdır) gerçek motorla beş golden anında, iki zodyak kipinde 10⁻⁶° içinde doğrulanmıştır (Tromsø dahil).
- **D3 — Tam burçta bhava burcun kendisidir (başlangıç 0°, yay 30°); orta, lagnanın derecesinin o burçtaki karşılığıdır (`whole_sign_mids`: lagna + 30° × (n − 1)).** Gerekçe: Bhava bala aynı noktayı kullanır (`shadbala::bhava_midpoint`); iki panel aynı "orta"yı gösterir. Burcun 15°'si ikinci okumadır (Y5: ayar olmaz).
- **D4 — Placidus'ta nokta bhavanın başlangıcıdır ve bhava ortası tanımlanmaz: `BhavaRow.mid` ve `GrahaBhava.from_mid_deg` `None`'dır; panel çizgi yazar ve nedenini söyler.** Gerekçe: KP okumasında "orta" kavramı yoktur; başlangıç ile bitişin yay ortasını yazmak, hiçbir kaynağa dayanmayan bir değer üretmek olurdu.
- **D5 — Bitiş, sonraki bhavanın başlangıcıdır; `span_deg` ikisinin farkıdır. Grahanın bhavası `house_from_cusps` ile bulunur (graha tablosunun "ev" sütunuyla aynı kural: başlangıç noktası bhavaya dahildir). `from_start_deg` [0°, yay) aralığındadır; `from_mid_deg` işaretlidir ((−180°, 180°]; eksi: graha ortadan önce). `sign_house` burç evidir (K4); `shifted = bhava != sign_house`.** Gerekçe: tablo ile graha tablosu aynı evi göstermelidir; işaretli uzaklık "ortadan önce mi sonra mı" sorusunu tek sütunda yanıtlar.
- **D6 — Bhava noktası rasi çerçevesindedir (boylam, burç, derece; K5); nakshatra ve pada sidereal boylamdandır. Burç lordu `dignity::sign_lord`'dur. Nakshatra lordu modülün kendi fonksiyonudur (`nakshatra_lord`: `VIMSHOTTARI_ORDER`'ın dokuzlu sırası); `dasha.rs` değişmez.** Gerekçe: Plan 23 ve 26 da nakshatra lordu ister; paylaşılan bir dosyaya üç plandan dokunmak yerine küçük bir yerel fonksiyon yazıldı. Test onu `birth_dasha` ile 27 nakshatrada karşılaştırır; ortaklaştırma sonraki bir temizlik işidir.
- **D7 — Doğum saati bilinmiyorsa `compute_bhava_table` `None` döner (K2); ayar, an ve konum denetimi yine yapılır. Sayı prashnasında tablo haritayı izler: ev sistemi ayarı ne olursa olsun tam burç. Placidus kutup dairesinde Porphyry'ye düşerse (`ChartWarning::PlacidusPolarFallback`) tablo `porphyry_fallback: true` taşır ve iki yüzey bunu sözle söyler.** Gerekçe: yol haritası §4.5; var olan uyarı rozeti yalnızca harita başlığındadır.
- **D8 — Yeni hata türü, yeni `CommandError` kodu ve yeni i18n hata iletisi yoktur.** Gerekçe: iki komutun girdisi yalnızca `ChartInput`'tur; hataları `compute_chart`'ınkilerdir. `error.rs`, `commands.rs` ve `i18n.test.ts` paralel planların dokunacağı dosyalardır.
- **D9 — İki komut yeni bir dosyadadır (`app/src-tauri/src/bhava_commands.rs`): `compute_bhava_table(input) -> Option<BhavaTable>` ve `compute_sudarshana(input) -> SudarshanaChakra`. Gerçek motor testi yeni bir test dosyasıdır (`app/src-tauri/tests/bhava.rs`; kendi süreci, kendi motoru).** Gerekçe: `module_commands.rs` ve `tests/modules.rs` paralel planların eklemelerini alır. **Yol haritası farkı:** taslak komutu Bhava tablosu paneliyle aynı göreve koyar; burada iki komut, bağlamalar ve fikstür tek görevdedir (Task 3): üretilen dosyalar bir kez yeniden üretilir.
- **D10 — El örnekleri Rust birim testleridir (`table.rs` ve `sudarshana.rs`); `tests/golden/modules/` altına JSON fikstür eklenmez ve `modules_golden.rs` değişmez. `tests/golden/modules/bhava/REVIEW.md` okuma kurallarını, onay bekleyen seçimleri ve denetim maddelerini taşır.** Gerekçe: Plan 22 D9'un yolu; beklenen değerler birkaç sayıdır ve türetmesi test yorumundadır. K9 geçerlidir: beklenen değer başka yazılımdan alınmaz.
- **D11 — Bhava tablosu paneli `bhava_table` türüdür (`MODULE_PANELS`: 7 × 20; paletten eklenir ve çıkarılır, K12). Panelin tek ayarı noktadır (`point`: `start`, `mid`, `end`; varsayılan başlangıç) ve düzenle birlikte saklanır. İki tablo: bhavaların seçili noktası (burç ve derece, nakshatra, pada, burç lordu, nakshatra lordu, yay) ve grahaların bhavası (burç evi, bhava, başlangıçtan, ortadan).** Gerekçe: üç noktanın beşer sütunu tek tabloda on beş sütun ederdi; nokta seçimi tabloyu yedi sütunda tutar ve "graha–ev başlangıcı dizisi" (başlangıç noktası) varsayılandır.
- **D12 — Chalit görünümü harita panelinin `chalit` ayarıdır. Hücreler yerinde durur: n. hücre n. rasi evinin hücresidir ve burç numarası o evin burcudur; her graha bhavasının numarasını taşıyan hücreye çizilir (`ChartPlanet.cell`); burcundan farklı bir hücreye giden graha `*` (`SHIFT_MARK`) taşır ve erişilebilir adına "bhavası farklı" eklenir. İki burca yayılan bhava tek hücredir; kesin sınırlar Bhava tablosu panelindedir. Görünüm yalnızca D1'de, doğum saati biliniyorsa ve ev sistemi tam burç değilse çizilir. Kutu, ev sistemi tam burçken yalnızca ayar açıksa gösterilir ve nedenini söyler.** Gerekçe: yol haritası §4.5 "Kuzey ve Güney Hint çizimlerinin tutarlı gösterimi": üç biçimde de aynı kural (hücre = ev numarası) uygulanır ve geometri değişmez. Tam burçta kutunun hiç sunulmaması, varsayılan ayardaki haritaların araç çubuğunu ve var olan altı görsel temeli değiştirmez. **Yol haritası farkı:** taslak "adaptör ve geometri" der; `geometry.ts` değişmez.
- **D13 — Chalit görünümünde transit halkası, eş halkası ve arudha padalar çizilmez ve istenmez; ayarı açık olan kutu işaretli durur ve nedenini `aria-describedby` ile bağlı görünür bir satırdan okur.** Gerekçe: üçü de burca göre yerleşir; bhava hücresinde burca göre duran bir etiket yanlış okunurdu. Ayarlar silinmez: chalit görünümü çıkınca katman geri gelir.
- **D14 — Sudarshana chakra: halka sırası içten dışa lagna, Ay, Güneş (`SudarshanaRef::ALL`). Her halkada n. ev, halkanın başlangıç burcundan sayılan n. burçtur (K4) ve grahalar rasi çerçevesindeki burçlarından okunur. Doğum saati bilinmiyorsa lagna halkasının evleri `None`'dır ve sonuç `approximate` işaretlidir (Ay'ın burcu o gün değişmiş olabilir). Sudarshana dashası hesaplanmaz.** Gerekçe: yol haritası §4.6 ve §5'in önerisi.
- **D15 — Çizim: 1. ev solda, evler saat yönünün tersine (daire haritasıyla aynı yön); aynı numaralı ev üç halkada aynı açıda. Yarıçaplar `[5, 22, 36, 49]`; her hücrenin iç şeridinde burç numarası (yazı 2,2), dışında graha kısaltmaları (var olan `layoutLabels`). Her halkanın 1. evi dolguludur; lagna halkasının 1. evinde lagna işareti yazar. Çizimde yalnızca graha kısaltması yazılır (retro, yanma ve vargottama işareti yok). Çizim `role="img"`'dir; tablo karşılığı görünürdür ve çizimin altında durur. Panel `sudarshana` türüdür (`MODULE_PANELS`: 6 × 24).** Gerekçe: yarıçaplar ölçülerek seçildi: en dar açıdaki hücrede bile iç halkaya altı, orta ve dış halkaya dört etiket sığar (`sudarshana.test.ts` bunu sabitler); daha çok graha en küçük yazı boyutunda taşarak yazılır ve tablo tam listeyi taşır. 63 sekme durağı klavyeyle kullanılamaz bir çizim olurdu.
- **D16 — Arayüz fikstürü `bhava` anahtarını kazanır: açık haritanın (Delhi) tam burç, Sripati ve Placidus tabloları, Placidus haritası (`chart_placidus`), kutup dairesinde Placidus tablosu (Tromsø), iki chakra (saat bilinen ve bilinmeyen) ve `SudarshanaRef::ALL`. Chalit görünümünün verisi Placidus haritasıdır.** Gerekçe: ölçüm: Delhi fikstüründe Sripati hiçbir grahayı yer değiştirmez; Placidus üçünü değiştirir (Merkür 7 → 6, Jüpiter 11 → 10, Venüs 6 → 5). Fikstür üretimi bunu `assert_eq!` ile sabitler. Yeni danışan eklenmez: Plan 22'nin arama fikstüründeki harita sayıları değişmez.
- **D17 — Beş yeni görsel temel (yerel, darwin): `chalit-north`, `chalit-south`, `chalit-circle` (açık tema) ve `sudarshana-light`, `sudarshana-dark`. Var olan on temel değişmez. Yeni temeller yalnızca kendi adlarıyla üretilir (`-g "chalit|sudarshana" --update-snapshots`); öteki temeller `--update-snapshots` ile yeniden yazılmaz. Yeni baskı düzeni yoktur (K10).** Gerekçe: C11; yol haritası §4.5 ve §4.6 "görsel temeller".
- **D18 — C4 okuması: "graha yer değiştirdi mi" bilgisini çizim, Rust'ın verdiği iki değeri (`GrahaPosition.house` ve burç) var olan gösterim eşlemesiyle (`signHouse`) karşılaştırarak bulur; ayrı bir kural uygulamaz. Bir test, çizimin işaretlediği grahaların Rust'ın `GrahaBhava.shifted` listesiyle aynı olduğunu fikstürde sabitler.** Gerekçe: Bhava bala ve arudha panellerindeki "ev ↔ burç" eşlemesiyle aynı okuma; chalit görünümü için ikinci bir komut çağrısı, haritanın kendisinde duran veriyi yeniden isterdi.
- **D19 — Bu planın kapsamadıkları: KP alt lordu (nakshatra alt lordu); Sudarshana dashası; bhava ortasına göre bakış ve güç; geçiş grahalarının chalit yerleşimi; bölüm haritalarında chalit; baskı belgesinde chalit ve chakra; bhava başlangıçlarının harita çizimine derece olarak yazılması; ev sistemi ayarının değiştirilmesi (Ayarlar'daki var olan seçim kullanılır).** Gerekçe: yol haritası §4.5, §4.6, §7; K10.
- **D20 — Adlandırma: bu planın getirdiği adların hiçbiri yasak diziyi içermez. Türkçe metinlerde dizi `-ka` ile biten sözcüklerin çoğulunda oluşur; "halka" sözcüğü çoğul yazılmaz ("üç halka", "halka sırası", "her halkada"), süre ve sayı sözcüklerinin çoğulu yazılmaz. "Geriye duran", "öteki", "sürer", "yerinde durur" yazılır.** Gerekçe: Y8, v1 spec §10 ve Ek A; yasak harf dizisi düzeyindedir.

## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni bağımlılık yok: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `app/package.json`, `app/pnpm-lock.yaml` ve `app/vite.config.ts` değişmez; `cargo deny check` Task 8'de `advisories ok, bans ok, licenses ok, sources ok` verir.
- **Yasaklı ad kuralı (v1 spec §10 "İsim kuralları", ilk madde; ileri modüller spec'i Ek A; yol haritası Y8):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Kural harf dizisi düzeyindedir: Türkçe sözcüklerin içinde de geçemez (D20). Her commit'ten önce eklenen satırlar şu komutla taranır ve çıktı boş olmalıdır: `git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'`.
- **Clean-room ve yorum metni yok:** kurallar klasik tanımdan özgün olarak uygulanır; başka ürünün adı, düzeni, metni ya da çıktısı kullanılmaz; kaynak gösterimi geneldir ve alıntı yapılmaz. Tablo ve çizim olgu gösterir: **"iyi/kötü" hükmü, puan, renkli derecelendirme ve bir yerleşimin ne anlama geldiğini anlatan metin eklenmez.** Yürüten ajan okuma seçimlerini başka yazılımla ya da çevrimiçi kaynakla karşılaştırmaz (Y4).
- **Çerçeveler (K4, K5):** bhava noktaları ve grahalar rasi çerçevesindedir (`settings.zodiac`'ı izler); nakshatra ve pada sidereal boylamdandır. Sudarshana'da ev, halkanın başlangıç burcundan sayılan burç evidir. Bhava tablosu haritanın kendi `calc_settings`'iyle hesaplanır.
- **Doğum saati bilinmiyorsa (K2):** `time_known` yanlışsa bhava tablosu `None`'dır, chakranın lagna halkası `None`'dır; arayüzün gizlemesine güvenilmez. Arayüz bunu söyler: tablo paneli bir satırla, chalit kutusu nedeniyle, chakra paneli notla ve "≈" ile.
- **Gizlilik (spec §14):** iki komutun girdisi doğum verisidir (`ChartInput`): hiçbir yerde loglanmaz. Hata parametreleri `compute_chart`'ınkilerdir; yeni parametre yoktur. Hiçbir modül ağ isteği yapmaz. Testler ve fikstürler yalnızca deneme adları kullanır ("Örnek Danışan"); gerçek kişi verisi yoktur.
- **El örnekleri (§13, K9):** beklenen değer kuraldan elle türetilir ve test yorumunda yazılıdır. Bir el örneği motorla uyuşmazsa önce kural yeniden okunur; **beklenen değer motor çıktısına bakarak düzeltilmez.**
- **Kapılar:** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (1.100), `cargo test --workspace`, `pnpm --dir app test`, `pnpm --dir app build` her görev sonunda yeşil; `pnpm --dir app e2e` ve `pnpm --dir app e2e:visual` Task 7 ve 8'de; `cargo deny check` Task 8'de. `tsc` `strict`. `pnpm build` çıktısında aranan satır `✓ built`'tir. Tek bir Vitest dosyası `pnpm --dir app exec vitest run ui/…` ile koşulur: yol `app/` dizinine göredir. `cargo test --workspace` çıktısı uzun olduğu için başarısızlık şu sayaçla denetlenir ve `0` yazmalıdır: `cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."`.
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez). Node 24+, pnpm 11.8.0. Depo kısa bir yolda durmalıdır: Swiss Ephemeris 242 bayttan uzun ephemeris yolunu reddeder. Yeni bir çalışma ağacında `app/node_modules` yoksa önce `pnpm --dir app install --frozen-lockfile` koşulur (kilit dosyası değişmez).
- **Gerçek uygulama çalıştırılmaz:** `pnpm tauri dev` ve `pnpm --dir app tauri dev` hiçbir adımda koşulmaz; üretilen `.app` çalıştırılmaz; `~/Library/Application Support/io.github.turgaydogusan.jyotishdesk` dizinine dokunulmaz. Başka bir oturumun çalışan geliştirme sunucusu durdurulmaz (port 1420); e2e 1421'i kullanır. `pkill -f` kullanılmaz. Task 8'in **[insan]** maddeleri sahibine devredilir (Y3).
- Yalnızca macOS hedeflenir. Yorum metni ve AI paneli bu planın dışındadır.
- **Tabanlar (`7270550`):** `jyotish-core` lib 404, golden 8, `modules_golden` 17; `atlas` `moment_fixed_lmt` 11, `moment_iana` 13 (`build` 6, `search` 8, `shipped` 3); `store` lib 122 (+1 `#[ignore]`); `jyotish-desk` lib 58, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `Test Files  88 passed (88)`, `Tests  887 passed (887)`; e2e 27; görsel 10 (yerel, darwin); yayın betiği testleri 14.
- **Sayaç zinciri** (her satır görev bitince; boş hücre "önceki satırla aynı"). Bu plan yazılırken `7270550`'nin bir kopyasında her görev ayrı commit olarak uygulanıp ölçülmüştür.

  | Görev | `jyotish-core` lib | `jyotish-desk` test dosyaları | Vitest dosya / test | e2e | görsel |
  |---|---|---|---|---|---|
  | Taban | 404 | lib 58; `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2 | 88 / 887 | 27 | 10 |
  | Task 1 | 412 | | | | |
  | Task 2 | 415 | | | | |
  | Task 3 | | + `bhava` 1 | 88 / 888 | | |
  | Task 4 | | | 89 / 899 | | |
  | Task 5 | | | 89 / 914 | | |
  | Task 6 | | | 92 / 930 | | |
  | Task 7 | | | | 30 | 15 |
  | Task 8 | | | | | |

  Değişmeyenler: `jyotish-core` golden 8, `modules_golden` 17; `atlas` 11 / 13, `build` 6, `search` 8, `shipped` 3; `store` lib 122 (+1 `#[ignore]`); `jyotish-desk` lib 58, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; yayın betiği testleri 14. `jyotish-core` lib Task 1'de 8 test (`bhava/table.rs`), Task 2'de 3 test (`bhava/sudarshana.rs`) kazanır. Vitest Task 3'te 1 test (`test/fixture.test.ts`), Task 4'te bir dosya ve 11 test (`panels/BhavaTablePanel.test.tsx`), Task 5'te 15 test (`chart/adapter.test.ts` 2, `chart/ChartSvg.test.tsx` 4, `panels/ChartPanel.test.tsx` 9), Task 6'da üç dosya ve 16 test (`chart/sudarshana.test.ts` 4, `chart/SudarshanaSvg.test.tsx` 5, `panels/SudarshanaPanel.test.tsx` 7) kazanır. **Paralel planlar birleştirildikten sonra sayaçlar toplanır; bu zincir yalnızca bu planın çalışma ağacı içindir.**
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir; bu plandaki okumaları:**
  - **C1** Gizlilik: yukarıdaki madde; `no-console.test.ts` durur ve izin listesi büyümez. Yeni `CommandError` kodu yoktur (D8).
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu ya da adres almaz. İki yeni komut yalnızca `ChartInput` alır; `api.test.ts`'in dizgi bağımsız değişken listesi değişmez.
  - **C3** Veritabanı şeması ve göçler değişmez (Y6); `CalcSettings`, `ViewState` ve `AppPrefs` tipleri ve şema sürümleri değişmez. Panel ayarları (`point`, `chalit`) serbest biçimli `settings` nesnesindedir ve düzenle birlikte var olan yoldan kaydedilir.
  - **C4** Arayüz astrolojik hesap yapmaz (D18). Burç dizini, derece biçimi, kimliğin ada çevrilmesi ve ev numarasının hücreye eşlenmesi gösterimdir.
  - **C5/C17** Otomatik kayıtlar değişmez. **C6** Düzen ayrıştırıcıları (`parseLayoutState`, Rust `check_panel_layout`) değişmez; iki panel türü kataloğa eklenir.
  - **C7/C16** Yeni kısayol yoktur. **C8** Yeni pencere yoktur.
  - **C9** Renk yalnızca token'lardan gelir (hex yok) ve tek başına anlam taşımaz: yer değiştiren graha `*` ile ve sözle, her halkanın 1. evi dolguyla ve tablonun ilk satırıyla, yaklaşık değer "≈" ile ve sözle gösterilir.
  - **C10** Üretilen dosyalar elle düzenlenmez: `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` Task 3'te komutla yeniden üretilir.
  - **C11** Görsel PNG temelleri yereldir; var olan on temel değişmez, beş yenisi Task 7'de üretilir (D17).
  - **C12** Her yeni i18n anahtarı üç dilde vardır (`i18n.test.ts` anahtar kümelerini ve yer tutucuları karşılaştırır). Fransızca metinler tipografik kesme işareti (’) kullanır.
  - **C13** Yasaklı ad taraması her commit'te; depo dışındaki ilerleme notları ve `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** `compute_panchang`, `chart.rs`, `houses.rs` ve `ephemeris` kasası değişmez. **C15** `alert`, `confirm`, `prompt`, `window.print()` ve `window.open()` çağrılmaz.
- **Retro** her yerde düz `R`'dir ve Rahu ile Ketu'da gösterilmez (UI spec §2.3); chalit görünümü var olan işaretleri korur ve `*`'ı sona ekler ("Mec*"). Sudarshana çiziminde retro yazılmaz (D15).
- **Erişilebilirlik:** WCAG AA. Form alanlarının görünür etiketi vardır; kullanılamayan kutu yerel `disabled` değil `aria-disabled`'dır ve nedeni `aria-describedby` ile bağlı görünür bir satırdır; tablo `caption`, `scope="col"` ve `scope="row"` taşır ve kaydırma kabı klavye odağı alır; hata `role="alert"`'tir; "yeniden dene" sonrasında odak gövdeye düşmez; `aria-hidden` işaretlerin (`*`, "≈") sözle okunan karşılığı vardır.
- Commit mesajları İngilizcedir ve tam olarak `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` satırıyla biter. Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, `git tag`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Kod blokları `7270550` ağacına göre yazılmış ve bu plan yazılırken deponun bir kopyasında koşulmuştur (Self-Review madde 5). Yeni dosyalar tam içerikle verilir ("**Yeni dosya:**"). Değişen dosyalar `diff` bloğuyla verilir: blok bir dosyaya konup depo kökünde `git apply <dosya>` ile ya da elle uygulanır; bağlam tutmazsa önceki görevin adımı denetlenir, elle uydurma yapılmaz. Yeni bir Rust dosyasının testleri ve uygulaması ayrı adımlarda verilir: önce dosya yalnızca `#[cfg(test)] mod tests` bloğuyla oluşturulur, sonra uygulama **o bloğun üstüne**, arada bir boş satırla eklenir; `cargo fmt --all --check` birleştirmeyi doğrular.

## Paylaşılan dosyalar

Plan 23 ve Plan 24 aynı tabandan paralel yürür. Aşağıdaki dosyalara onlar da dokunabilir. Her satır bu planın eklemesini ve çapa satırını (eklemenin hemen önündeki var olan satır) verir. Birleştirmede kural: **iki tarafın eklemeleri de tutulur** (bu planın hiçbir eklemesi var olan bir satırı silmez; tek istisna `model.test.ts`'teki tek satırlık beklenen listedir). Yeni çekirdek kod, komutlar ve testler yeni dosyalardadır ve çakışmaz.

| Dosya | Bu planın eklemesi | Çapa ve birleştirme notu |
|---|---|---|
| `crates/jyotish-core/src/lib.rs` | Task 1–2: ayrı blok (`pub mod bhava;` ve `pub use bhava::{…};`) | Dosyanın başında, `//!` belge yorumundan sonra ve `pub mod ashtakavarga;` satırından önce, boş satırlarla ayrılmış. Alfabetik `mod` ve `use` gruplarına dokunmaz. `//!` başlık yorumu değişmez. |
| `app/src-tauri/src/lib.rs` | Task 3: `pub mod bhava_commands;` ve `collect_commands!` içinde iki satır | `mod` satırı alfabetik yerindedir (`atlas_commands`'tan sonra; `rustfmt` zorlar). Komut satırları `transit_commands::transits_at,` satırından sonra. |
| `app/src-tauri/tests/ui_fixture.rs` | Task 3: bir `let bhava = { … };` bloğu ve `json!` içinde bir anahtar satırı | Blok `let value = json!({` satırından hemen önce; anahtar `"opened": opened,` satırından hemen sonra. `use` satırları değişmez (blok kendi `use`'larını içeride yapar). |
| `app/ui/bindings.ts` | Task 3: üretilir | **Elle birleştirilmez.** Çakışmada herhangi bir taraf alınır ve `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings` ile yeniden üretilir. |
| `app/ui/test/fixtures/delhi.json` | Task 3: üretilir (tek satırlık dosya) | **Elle birleştirilmez.** Çakışmada herhangi bir taraf alınır ve `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture` ile yeniden üretilir. |
| `app/ui/test/fixture.ts` | Task 3: beş tip adı `import type` listesine; `UiFixture.bhava` alanı; `placidusOpened()` | Alan `opened: OpenedClient;` satırından sonra; fonksiyon `unknownTimeOpened`'ın belge yorumundan önce. |
| `app/ui/test/fixture.test.ts` | Task 3: bir test ve `import` satırında bir ad | Test `"fixture copies are independent"` testinden önce. |
| `app/ui/api.ts`, `app/ui/api.test.ts` | Task 3: iki sarmalayıcı; taklit listesinde iki satır; çağrı tablosunda iki satır | `api.ts` ve çağrı tablosu: `computeDrishti` satırından sonra. Taklit listesi: `dashaChildren: vi.fn(),` satırından sonra. |
| `app/e2e/ipc.ts` | Task 3: `placidus` seçeneği; iki komutun taklidi | Seçenek `partnerUnknownTime`'dan sonra; taklitler `compute_drishti` satırından sonra. |
| `app/ui/layout/model.ts` | Task 4 ve 6: `PANEL_KINDS`'e iki tür; `MODULE_PANELS`'e iki girdi | İkisi de `bhava_bala` satırından sonra (önce `bhava_table`, sonra `sudarshana`). |
| `app/ui/layout/model.test.ts` | Task 4 ve 6: iki beklenen listede iki ad | `bhava_bala`'dan sonra. **Tek satırlık `MODULE_PANELS` beklentisi her planın değiştirdiği satırdır:** birleştirmede liste `model.ts`'teki sırayla, iki tarafın adlarının birleşimi olarak yeniden yazılır. |
| `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx` | Task 4 ve 6: iki `import`, iki kayıt, iki beklenti | Kayıtlar ve beklentiler `bhava_bala` satırından sonra; `import`'lar `BhavaBalaPanel` ve `ShadbalaPanel` satırlarından sonra. |
| `app/ui/i18n/tr.ts`, `en.ts`, `fr.ts` | Task 4–6: `panel.bhava_table`, `panel.sudarshana`; `chart.*` altında dokuz anahtar; iki yeni üst düzey bölüm (`bhavaTable`, `sudarshana`) | `panel.*`: `argala` satırından sonra. `chart.*`: `arudhaList` satırından sonra. Üst düzey bölümler: üst düzey `arudha: {` bölümünden hemen önce (dosya sonuna değil). `errors` bölümü ve `i18n.test.ts` değişmez. |
| `app/ui/chart/chart.css`, `app/ui/panels/panels.css` | Task 6: dosya sonuna birer küçük blok | Dosya sonuna ekleme: paralel bir plan da sona eklediyse iki blok da tutulur. |
| `app/e2e/desk.spec.ts`, `app/e2e/visual.spec.ts` | Task 7: dosya sonuna üç akış ve beş görsel test | Dosya sonuna ekleme: iki tarafın testleri de tutulur. Beş yeni PNG yeni dosyadır. |

Yalnızca bu planın dokunması beklenen var olan dosyalar: `app/ui/chart/types.ts`, `app/ui/chart/adapter.ts` (+test), `app/ui/chart/ChartSvg.tsx` (+test), `app/ui/panels/ChartPanel.tsx` (+test). Bu planın **dokunmadığı** paylaşılan dosyalar: `crates/jyotish-core/src/error.rs`, `chart.rs`, `houses.rs`, `dasha.rs`, `positions.rs`; `crates/jyotish-core/tests/modules_golden.rs`; `app/src-tauri/src/commands.rs`, `module_commands.rs`; `app/src-tauri/tests/modules.rs`; `app/ui/i18n/i18n.test.ts`; `app/ui/chart/geometry.ts`; `app/ui/palette/*`; `crates/store/*`.

**Birleştirmeden sonra (bu planın işi değildir; birleştiren yapar):** metin çakışmaları iki taraf tutularak çözülür; `bindings.ts` ve `delhi.json` iki komutla yeniden üretilir; `model.test.ts`'in iki listesi `model.ts`'e göre yazılır; bütün kapılar yeniden koşulur. Görsel temeller birleşmeden etkilenmez (her plan kendi PNG'lerini ekler).

## Sonraki işlere verilen API

Bu planın ürettiği ve sonraki işlerin tüketeceği adlar. Rust adları yıldızlıysa `jyotish_core` kökünden dışa aktarılır; imzalar görevlerde tam olarak verilir.

| API | Tüketen | Not |
|---|---|---|
| `bhava_table`\*, `compute_bhava_table`\*, `BhavaTable`\*, `BhavaRow`\*, `BhavaPoint`\*, `GrahaBhava`\*, `BHAVA_METHOD`\* | Plan 27–28 (yıllık haritanın bhavaları), AI paneli | Saf tablo; `None`: saat bilinmiyor |
| `bhava_point`\*, `nakshatra_lord`\*, `signed_arc`\*, `whole_sign_mids`\*, `porphyry_points`\*, `sandhis_from_mids`\* | Plan 26 (nakshatra lordu sütunları) | `nakshatra_lord` ortaklaştırılabilir (D6) |
| `sudarshana`\*, `compute_sudarshana`\*, `SudarshanaChakra`\*, `SudarshanaRing`\*, `SudarshanaHouse`\*, `SudarshanaRef`\*, `SUDARSHANA_METHOD`\* | Sudarshana dashası (planlanmadı), AI paneli | Halka sırası `SudarshanaRef::ALL` |
| `bhava_commands::{compute_bhava_table_blocking, compute_sudarshana_blocking}` | fikstür, testler | Kalıp `module_commands` ile aynı |
| Arayüz: `modules/useBhavaTable.ts`, `modules/useSudarshana.ts`, `panels/BhavaTablePanel.tsx`, `panels/SudarshanaPanel.tsx`, `chart/sudarshana.ts`, `chart/SudarshanaSvg.tsx`, `chart/adapter.chalitData`, `chart/types.SHIFT_MARK`, `ChartPlanet.cell`, `ChartPlanet.shifted`; fikstürde `bhava`; `test/fixture.placidusOpened`; e2e `mockIpc({ placidus })` | sonraki çizimler (Sarvatobhadra), baskı | i18n: `bhavaTable.*`, `sudarshana.*`, `chart.chalit*`, `panel.bhava_table`, `panel.sudarshana` |

## Dosya Yapısı (bu plan)

```
crates/jyotish-core/
  src/bhava/mod.rs                modül kaydı ve dışa aktarımlar (yeni)
  src/bhava/table.rs              BhavaTable, bhava_table, compute_bhava_table; el örnekleri (yeni)
  src/bhava/sudarshana.rs         SudarshanaChakra, sudarshana, compute_sudarshana; el örnekleri (yeni)
  src/lib.rs                      ayrı blokta modül kaydı ve dışa aktarımlar
app/src-tauri/
  src/bhava_commands.rs           iki komut (yeni)
  src/lib.rs                      modül ve komut kaydı
  tests/bhava.rs                  gerçek motorla denetim (yeni)
  tests/ui_fixture.rs             fikstüre bhava
app/ui/
  bindings.ts                     üretilir
  test/fixtures/delhi.json        üretilir
  test/fixture.ts (+test)         UiFixture.bhava, placidusOpened
  api.ts, api.test.ts             iki sarmalayıcı
  modules/useBhavaTable.ts        (yeni)
  modules/useSudarshana.ts        (yeni)
  panels/BhavaTablePanel.tsx (+test)   (yeni)
  panels/SudarshanaPanel.tsx (+test)   (yeni)
  panels/ChartPanel.tsx (+test)   chalit ayarı, nedenler, açıklama satırı
  panels/registry.ts, panels/panels.test.tsx, panels/panels.css
  chart/types.ts                  ChartPlanet.cell, ChartPlanet.shifted, ChartLabels.shifted, SHIFT_MARK
  chart/adapter.ts (+test)        chalitData
  chart/ChartSvg.tsx (+test)      hücreye göre gruplama, `*` işareti
  chart/sudarshana.ts (+test)     halka geometrisi (yeni)
  chart/SudarshanaSvg.tsx (+test) (yeni)
  chart/chart.css
  layout/model.ts (+test)         bhava_table ve sudarshana türleri, MODULE_PANELS
  i18n/{tr,en,fr}.ts              panel, chart, bhavaTable, sudarshana
app/e2e/
  ipc.ts                          placidus seçeneği; iki komutun taklidi
  desk.spec.ts                    üç akış
  visual.spec.ts (+5 PNG)         chalit × 3, sudarshana × 2
tests/golden/modules/bhava/REVIEW.md   okuma kuralları, onay bekleyen seçimler, denetim maddeleri (yeni)
```

### Kapsam

| Yol haritası maddesi | Yer |
|---|---|
| §4.5 On iki bhavanın başlangıcı, ortası ve bitişi (burç, derece, nakshatra-pada, iki lord) | Task 1 (`table.rs`; D2–D6), Task 4 |
| §4.5 Her grahanın bhavası ve bhava ortasına uzaklığı | Task 1 (D5), Task 4 |
| §4.5 Sripati'de orta ayrıca hesaplanır | Task 1 (`porphyry_points`; D2), Task 3 (ikinci yazım, gerçek motor) |
| §4.5 Tam burçta chalit görünümü rasi ile aynıdır ve panel bunu söyler | Task 4 (sistem notu), Task 5 (D12) |
| §4.5 Kutup dairesinde Placidus'un Porphyry'ye düşmesi burada da gösterilir | Task 1 (D7), Task 4, Task 5 |
| §4.5 `Chart` dondurulduğu için yeni komut | Task 3 (D9) |
| §4.5 Chalit harita görünümü; iki burca yayılan bhavanın tutarlı gösterimi; görsel temeller | Task 5 (D12, D13), Task 7 (D17) |
| §4.5 KP alt lordu kapsam dışı | D19; Task 8 (`REVIEW.md` satır 8) |
| §4.6 Üç iç içe halka; her halkada on iki ev ve grahalar | Task 2 (D14), Task 6 (D15) |
| §4.6 36 hücreye sığan etiketler, klavye erişimi, tablo karşılığı, görsel temeller | Task 6 (`sudarshana.test.ts`, tablo), Task 7 |
| §4.6 Ortak seçim (UI spec §3.4) | Task 6 |
| §4.6 Saat bilinmiyorsa lagna halkası çizilmez | Task 2, Task 6 |
| §4.6 Sudarshana dashası kapsam dışı | D14, D19 |
| §5 Plan 25 görev (7): i18n ve "saat bilinmiyor" | Task 4, 5, 6 (her görev kendi üç dildeki metinlerini ve durumunu taşır) |
| §5 Plan 25 görev (8): kapılar ve elle denetim; sahibin denetimi | Task 8 (`REVIEW.md`, **[otomatik]** / **[insan]**) |
| Y3 Kapı testlerdir, gerçek uygulama değil | Global Constraints; Task 8 |
| Y4 Onay bekleyen okumalar `REVIEW.md`'de | Task 8 |
| Y6 Yeni kalıcı veri yok | C3 |
| K12 Paletten ekleme | Task 4, 6, 7 |

---

### Task 1: Çekirdek: `bhava` modülü ve bhava tablosu (başlangıç, orta, bitiş; graha uzaklığı)

Yol haritası §4.5; K1–K5, K7; D1–D8, D10. Var olan hiçbir hesabın davranışı değişmez: `chart.rs`, `houses.rs`, `dasha.rs` ve `dignity.rs` yalnızca okunur (`chart::{compute_chart_for, finite, rasi_frame, to_sidereal}` zaten `pub(crate)`'tir).

**Files:**
- Create: `crates/jyotish-core/src/bhava/mod.rs`, `crates/jyotish-core/src/bhava/table.rs`
- Modify: `crates/jyotish-core/src/lib.rs`
- Test: `table.rs` `mod tests` (8)

**Interfaces:**
- Consumes (hepsi var): `ephemeris::Ephemeris`; crate içi `chart::{compute_chart_for, finite, rasi_frame, to_sidereal, ChartWarning}`; `dasha::VIMSHOTTARI_ORDER`; `dignity::sign_lord`; `houses::house_from_cusps`; `positions::ChartInput`; `settings::{HouseSystem, ZodiacMode}`; `zodiac::{deg_in_sign, nakshatra_pada, normalize, Sign}`.
- Produces (`jyotish_core` kökünden de dışa aktarılır):
  - `BhavaPoint { lon: f64, sign: Sign, deg_in_sign: f64, nakshatra: u8, pada: u8, sign_lord: Graha, nakshatra_lord: Graha }`.
  - `BhavaRow { house: u8, start: BhavaPoint, mid: Option<BhavaPoint>, end: BhavaPoint, span_deg: f64 }`.
  - `GrahaBhava { graha: Graha, sign_house: u8, bhava: u8, from_start_deg: f64, from_mid_deg: Option<f64>, shifted: bool }`.
  - `BhavaTable { method: String, system: HouseSystem, porphyry_fallback: bool, rows: Vec<BhavaRow>, grahas: Vec<GrahaBhava> }` (12 satır, 9 graha).
  - `nakshatra_lord(nakshatra: u8) -> Graha`, `signed_arc(from: f64, to: f64) -> f64`, `bhava_point(lon: f64, zodiac: ZodiacMode, ayanamsa_deg: f64) -> BhavaPoint`, `whole_sign_mids(lagna_lon: f64) -> [f64; 12]`, `porphyry_points(asc: f64, mc: f64) -> [f64; 12]`, `sandhis_from_mids(mids: &[f64; 12]) -> [f64; 12]`.
  - `bhava_table(system: HouseSystem, cusps: &[f64; 12], mids: Option<&[f64; 12]>, lagna_sign: Sign, grahas: &[(Graha, f64); 9], zodiac: ZodiacMode, ayanamsa_deg: f64) -> BhavaTable`.
  - `compute_bhava_table<E: Ephemeris + ?Sized>(eph: &mut E, input: &ChartInput) -> Result<Option<BhavaTable>, CoreError>`.
  - `BHAVA_METHOD = "bhava_table.v1"`.

- [ ] **Step 1: Tabanları ölç**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
git log -1 --format=%h
git status --short
rustc --version
ls app/node_modules >/dev/null 2>&1 || pnpm --dir app install --frozen-lockfile
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `git log` `7270550` (ya da ondan ilerlemiş taban); `git status --short` boş; `rustc 1.100.0` ya da yenisi; `jyotish-core` lib `404 passed`, golden `8 passed`, `modules_golden` `17 passed`; `atlas` `moment_fixed_lmt` `11 passed`, `moment_iana` `13 passed`; `store` lib `122 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `58 passed`, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `88 passed (88)`, `887 passed (887)`. Sayılar farklıysa ölçülen değerler depo dışındaki ilerleme notlarına yazılır ve sayaç zinciri o tabandan, aynı farklarla yürütülür.

- [ ] **Step 2: Testler: lord sırası, işaretli yay, nokta okuması, Porphyry ve sandhi, Sripati tablosu, tam burç ve Placidus, toplayıcı; modül kaydı**

Önce modül kaydı (testlerin derlenebilmesi için), sonra test gövdesi. Testlerin el örneği: lagna 101° (Yengeç 11°), MC 341°; MC → lagna yayı 120°, lagna → IC yayı 60°; ortalar ve sandhiler tam sayıdır ve türetmeleri yorumlardadır. Örnek bilerek nakshatra ve burç sınırlarından uzak seçilmiştir (kayan nokta sınır etkisi olmasın).

**Yeni dosya:** `crates/jyotish-core/src/bhava/mod.rs`

````rust
//! Bhava tablosu (yol haritası Plan 25; §4.5). Saf hesap ve ince toplayıcısı: `table` seçili ev
//! sisteminde on iki bhavanın başlangıcını, ortasını ve bitişini ve her grahanın bhavasını verir.
//! Sonuç `Chart`'a eklenmez; panel açıkken istenir (K1).

mod table;

pub use table::{
    bhava_point, bhava_table, compute_bhava_table, nakshatra_lord, porphyry_points,
    sandhis_from_mids, signed_arc, whole_sign_mids, BhavaPoint, BhavaRow, BhavaTable, GrahaBhava,
    BHAVA_METHOD,
};
````

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 61db775..d1a9665 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -10,6 +10,15 @@
 //! Prashna (Alt proje 3.8): `prashna` (sayı prashnasının lagnası, soru haritasının olguları).
 //! Arama (Alt proje 3.9): `search` (harita araması sorgusu ve hafif yol, olay araması).
 
+// Bhava tablosu (yol haritası Plan 25). Ayrı blok: paralel planların alfabetik sıradaki
+// eklemeleriyle aynı satırlara dokunmaz.
+pub mod bhava;
+pub use bhava::{
+    bhava_point, bhava_table, compute_bhava_table, nakshatra_lord, porphyry_points,
+    sandhis_from_mids, signed_arc, whole_sign_mids, BhavaPoint, BhavaRow, BhavaTable, GrahaBhava,
+    BHAVA_METHOD,
+};
+
 pub mod ashtakavarga;
 mod chart;
 mod combustion;
````

`lib.rs`'teki blok bilerek dosyanın başındadır ve boş satırlarla ayrılmıştır (D1, "Paylaşılan dosyalar"); alfabetik `pub mod` grubuna taşınmaz.

**Yeni dosya:** `crates/jyotish-core/src/bhava/table.rs` (şimdilik yalnızca test modülü)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dasha::birth_dasha;
    use crate::fake::FakeEph;
    use crate::houses::whole_sign_cusps;
    use crate::settings::CalcSettings;

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

    fn close(a: f64, b: f64) -> bool {
        signed_arc(a, b).abs() < 1e-9
    }

    /// El örneğinin grahaları (`Graha::ALL` sırasında, sidereal).
    const LONS: [f64; 9] = [82.0, 80.5, 101.0, 110.0, 359.5, 1.0, 46.0, 201.0, 21.0];

    fn grahas() -> [(Graha, f64); 9] {
        std::array::from_fn(|i| (Graha::ALL[i], LONS[i]))
    }

    #[test]
    fn nakshatra_lords_repeat_every_nine_and_agree_with_the_dasha_seed() {
        assert_eq!(nakshatra_lord(1), Graha::Ketu);
        assert_eq!(nakshatra_lord(4), Graha::Moon);
        assert_eq!(nakshatra_lord(9), Graha::Mercury);
        assert_eq!(nakshatra_lord(10), Graha::Ketu);
        assert_eq!(nakshatra_lord(27), Graha::Mercury);
        // İkinci yazım: Vimshottari'nin doğum lordu, Ay'ın nakshatrasının lordudur.
        for n in 1..=27u8 {
            let mid = (f64::from(n) - 0.5) * 40.0 / 3.0;
            assert_eq!(nakshatra_lord(n), birth_dasha(mid).0, "nakshatra {n}");
        }
    }

    #[test]
    fn signed_arcs_are_short_and_signed() {
        assert_eq!(signed_arc(100.0, 110.0), 10.0);
        assert_eq!(signed_arc(110.0, 100.0), -10.0);
        assert_eq!(signed_arc(350.0, 10.0), 20.0);
        assert_eq!(signed_arc(10.0, 350.0), -20.0);
        assert_eq!(signed_arc(0.0, 180.0), 180.0);
        assert_eq!(signed_arc(180.0, 0.0), 180.0);
        assert_eq!(signed_arc(45.0, 45.0), 0.0);
    }

    #[test]
    fn a_point_reads_sign_nakshatra_and_both_lords() {
        // 81° sidereal: İkizler 21°; pada dizini 81 × 0,3 = 24,3 → 24: 7. nakshatra
        // (Punarvasu), 1. pada. Burç lordu Merkür; 7. nakshatranın lordu Jüpiter.
        let p = bhava_point(81.0, ZodiacMode::Sidereal, 24.0);
        assert_eq!((p.lon, p.sign, p.deg_in_sign), (81.0, Sign::Gemini, 21.0));
        assert_eq!((p.nakshatra, p.pada), (7, 1));
        assert_eq!(
            (p.sign_lord, p.nakshatra_lord),
            (Graha::Mercury, Graha::Jupiter)
        );
        // Tropikal-rasi kipinde burç tropikal boylamdan, nakshatra sidereal boylamdandır:
        // 81° tropikal − 24° = 57° sidereal → dizin 17: 5. nakshatra (Mrigashira), 2. pada.
        let t = bhava_point(81.0, ZodiacMode::TropicalRasiSiderealNakshatra, 24.0);
        assert_eq!((t.sign, t.nakshatra, t.pada), (Sign::Gemini, 5, 2));
        assert_eq!(
            (t.sign_lord, t.nakshatra_lord),
            (Graha::Mercury, Graha::Mars)
        );
        // Boylam [0, 360) aralığına iner.
        assert_eq!(
            bhava_point(-30.0, ZodiacMode::Sidereal, 0.0).sign,
            Sign::Pisces
        );
    }

    #[test]
    fn porphyry_points_trisect_the_quadrants_and_sandhis_bisect_the_points() {
        // Lagna 101°, MC 341°: MC → lagna yayı 120° (üçte biri 40°), lagna → IC yayı 60° (üçte
        // biri 20°). IC 161°, batı ufku 281°.
        let mids = porphyry_points(101.0, 341.0);
        let expected = [
            101.0, 121.0, 141.0, 161.0, 201.0, 241.0, 281.0, 301.0, 321.0, 341.0, 21.0, 61.0,
        ];
        for (i, (got, want)) in mids.iter().zip(expected).enumerate() {
            assert!(close(*got, want), "mid {} = {got}", i + 1);
        }
        // Sandhi iki komşu ortanın ortasıdır: 1. bhava 61° ile 101°'nin ortasında başlar.
        let sandhis = sandhis_from_mids(&mids);
        let expected = [
            81.0, 111.0, 131.0, 151.0, 181.0, 221.0, 261.0, 291.0, 311.0, 331.0, 1.0, 41.0,
        ];
        for (i, (got, want)) in sandhis.iter().zip(expected).enumerate() {
            assert!(close(*got, want), "sandhi {} = {got}", i + 1);
        }
        // Eşit çeyrekler (MC lagnadan 90° geride): noktalar 30° arayla durur.
        let equal = porphyry_points(70.0, 340.0);
        for (i, m) in equal.iter().enumerate() {
            assert!(close(*m, 70.0 + 30.0 * i as f64), "equal {}", i + 1);
        }
        // Tam burç: orta, lagnanın derecesinin o burçtaki karşılığıdır.
        assert_eq!(whole_sign_mids(346.0)[..3], [346.0, 16.0, 46.0]);
    }

    #[test]
    fn sripati_table_by_hand() {
        // Yukarıdaki örnek: lagna 101° (Yengeç 11°), MC 341°. Sandhiler ve ortalar elle
        // türetildi. Grahalar: Güneş 82°, Ay 80,5°, Mars 101°, Merkür 110°, Jüpiter 359,5°,
        // Venüs 1°, Satürn 46°, Rahu 201°, Ketu 21°.
        let mids = porphyry_points(101.0, 341.0);
        let cusps = sandhis_from_mids(&mids);
        let t = bhava_table(
            HouseSystem::Sripati,
            &cusps,
            Some(&mids),
            Sign::Cancer,
            &grahas(),
            ZodiacMode::Sidereal,
            24.0,
        );
        assert_eq!(t.method, BHAVA_METHOD);
        assert_eq!(t.system, HouseSystem::Sripati);
        assert!(!t.porphyry_fallback);
        assert_eq!((t.rows.len(), t.grahas.len()), (12, 9));
        // 1. bhava: 81° (İkizler 21°) → orta 101° (Yengeç 11°) → 111° (Yengeç 21°); yay 30°.
        let first = &t.rows[0];
        assert_eq!(first.house, 1);
        assert_eq!(
            (first.start.sign, first.end.sign),
            (Sign::Gemini, Sign::Cancer)
        );
        assert!(close(first.start.lon, 81.0) && close(first.end.lon, 111.0));
        assert!(close(first.mid.unwrap().lon, 101.0));
        assert!((first.span_deg - 30.0).abs() < 1e-9);
        // 81° sidereal: pada dizini 81 × 0,3 = 24,3 → 24: 7. nakshatra (Punarvasu, lord
        // Jüpiter), 1. pada. 101°: 30,3 → 30: 8. nakshatra (Pushya, lord Satürn), 3. pada.
        assert_eq!((first.start.nakshatra, first.start.pada), (7, 1));
        assert_eq!(first.start.nakshatra_lord, Graha::Jupiter);
        let mid = first.mid.unwrap();
        assert_eq!(
            (mid.nakshatra, mid.pada, mid.nakshatra_lord),
            (8, 3, Graha::Saturn)
        );
        // Yaylar: 30, 20, 20, 30, 40, 40, 30, 20, 20, 30, 40, 40 (toplam 360).
        let spans: Vec<f64> = t.rows.iter().map(|r| r.span_deg).collect();
        let expected = [
            30.0, 20.0, 20.0, 30.0, 40.0, 40.0, 30.0, 20.0, 20.0, 30.0, 40.0, 40.0,
        ];
        for (got, want) in spans.iter().zip(expected) {
            assert!((got - want).abs() < 1e-9, "span {got}");
        }
        assert!((spans.iter().sum::<f64>() - 360.0).abs() < 1e-9);
        // Her bhavanın bitişi sonrakinin başlangıcıdır.
        for i in 0..12 {
            assert_eq!(t.rows[i].end, t.rows[(i + 1) % 12].start);
        }
        // Grahalar: (burç evi, bhava, başlangıçtan, ortadan).
        //   Güneş 82° İkizler: burç evi 12; 81° ≤ 82° < 111° → 1. bhava; +1°; ortadan −19°.
        //   Ay 80,5° İkizler: burç evi 12; 41° ≤ 80,5° < 81° → 12. bhava; +39,5°; orta 61° →
        //     +19,5°.
        //   Mars 101° Yengeç: burç evi 1; 1. bhava; +20°; ortada (0°).
        //   Merkür 110° Yengeç: burç evi 1; 1. bhava; +29°; +9°.
        //   Jüpiter 359,5° Balık: burç evi 9; 331° ≤ 359,5° → 10. bhava; +28,5°; orta 341° →
        //     +18,5°.
        //   Venüs 1° Koç: burç evi 10; başlangıç noktası bhavaya dahildir → 11. bhava; 0°;
        //     orta 21° → −20°.
        //   Satürn 46° Boğa: burç evi 11; 41° ≤ 46° → 12. bhava; +5°; orta 61° → −15°.
        //   Rahu 201° Terazi: burç evi 4; 181° ≤ 201° < 221° → 5. bhava; +20°; ortada (0°).
        //   Ketu 21° Koç: burç evi 10; 1° ≤ 21° < 41° → 11. bhava; +20°; ortada (0°).
        let expected: [(u8, u8, f64, f64); 9] = [
            (12, 1, 1.0, -19.0),
            (12, 12, 39.5, 19.5),
            (1, 1, 20.0, 0.0),
            (1, 1, 29.0, 9.0),
            (9, 10, 28.5, 18.5),
            (10, 11, 0.0, -20.0),
            (11, 12, 5.0, -15.0),
            (4, 5, 20.0, 0.0),
            (10, 11, 20.0, 0.0),
        ];
        for (g, (sign_house, bhava, from_start, from_mid)) in t.grahas.iter().zip(expected) {
            assert_eq!(
                (g.sign_house, g.bhava),
                (sign_house, bhava),
                "{:?}",
                g.graha
            );
            assert!(
                (g.from_start_deg - from_start).abs() < 1e-9,
                "{:?}",
                g.graha
            );
            assert!(
                (g.from_mid_deg.unwrap() - from_mid).abs() < 1e-9,
                "{:?}",
                g.graha
            );
            assert_eq!(g.shifted, sign_house != bhava, "{:?}", g.graha);
        }
        let shifted: Vec<Graha> = t
            .grahas
            .iter()
            .filter(|g| g.shifted)
            .map(|g| g.graha)
            .collect();
        assert_eq!(
            shifted,
            [
                Graha::Sun,
                Graha::Jupiter,
                Graha::Venus,
                Graha::Saturn,
                Graha::Rahu,
                Graha::Ketu
            ]
        );
    }

    #[test]
    fn whole_sign_table_never_shifts_and_placidus_has_no_middle() {
        // Tam burç, lagna Yengeç 10°: bhava burcun kendisidir; orta burcun 10°'sidir.
        let cusps: [f64; 12] = whole_sign_cusps(Sign::Cancer).try_into().unwrap();
        let mids = whole_sign_mids(100.0);
        let t = bhava_table(
            HouseSystem::WholeSign,
            &cusps,
            Some(&mids),
            Sign::Cancer,
            &grahas(),
            ZodiacMode::Sidereal,
            24.0,
        );
        for (i, row) in t.rows.iter().enumerate() {
            assert_eq!(row.start.sign, Sign::from_index(3 + i));
            assert_eq!(row.start.deg_in_sign, 0.0);
            assert_eq!(row.span_deg, 30.0);
            assert!((row.mid.unwrap().deg_in_sign - 10.0).abs() < 1e-9);
        }
        assert!(t
            .grahas
            .iter()
            .all(|g| !g.shifted && g.bhava == g.sign_house));
        // Güneş İkizler 22°: 12. bhava; başlangıçtan 22°, ortadan (İkizler 10°) +12°.
        let sun = t.grahas[0];
        assert_eq!((sun.bhava, sun.from_start_deg), (12, 22.0));
        assert!((sun.from_mid_deg.unwrap() - 12.0).abs() < 1e-9);
        // Placidus: aynı başlangıçlar, orta yok.
        let p = bhava_table(
            HouseSystem::Placidus,
            &cusps,
            None,
            Sign::Cancer,
            &grahas(),
            ZodiacMode::Sidereal,
            24.0,
        );
        assert!(p.rows.iter().all(|r| r.mid.is_none()));
        assert!(p.grahas.iter().all(|g| g.from_mid_deg.is_none()));
        assert_eq!(p.grahas[0].from_start_deg, 22.0);
    }

    #[test]
    fn collector_follows_the_chart_and_hides_the_table_when_time_is_unknown() {
        let mut eph = FakeEph::default();
        // Tam burç (varsayılan): FakeEph lagnası sidereal 46° (Boğa 16°).
        let i = input(CalcSettings::default(), true);
        let t = compute_bhava_table(&mut eph, &i).unwrap().expect("table");
        let chart = compute_chart_for(&mut eph, &i).unwrap();
        assert_eq!(t.system, HouseSystem::WholeSign);
        assert_eq!(t.rows[0].start.sign, Sign::Taurus);
        assert!((t.rows[0].mid.unwrap().deg_in_sign - 16.0).abs() < 1e-9);
        for (g, c) in t.grahas.iter().zip(&chart.grahas) {
            assert_eq!((g.graha, g.bhava), (c.graha, c.house));
            assert!(!g.shifted);
        }
        // Sripati: tablo haritanın başlangıçlarını ve graha evlerini izler. FakeEph eşit
        // evler verir (MC lagnadan 90° geride): ortalar 30° arayla, lagnadan başlar.
        let sripati = CalcSettings {
            house_system: HouseSystem::Sripati,
            ..CalcSettings::default()
        };
        let i = input(sripati, true);
        let t = compute_bhava_table(&mut eph, &i).unwrap().expect("table");
        let chart = compute_chart_for(&mut eph, &i).unwrap();
        assert_eq!(t.system, HouseSystem::Sripati);
        for (row, cusp) in t.rows.iter().zip(&chart.bhavas.cusps) {
            assert_eq!(row.start.lon, *cusp);
        }
        for (k, row) in t.rows.iter().enumerate() {
            assert!(close(row.mid.unwrap().lon, 46.0 + 30.0 * k as f64));
        }
        for (g, c) in t.grahas.iter().zip(&chart.grahas) {
            assert_eq!(g.bhava, c.house);
        }
        // Placidus kutup dairesinde Porphyry'ye düşerse tablo bunu söyler; orta yoktur.
        let mut polar = FakeEph {
            porphyry_fallback: true,
            ..FakeEph::default()
        };
        let placidus = CalcSettings {
            house_system: HouseSystem::Placidus,
            ..CalcSettings::default()
        };
        let t = compute_bhava_table(&mut polar, &input(placidus.clone(), true))
            .unwrap()
            .expect("table");
        assert!(t.porphyry_fallback);
        assert_eq!(t.system, HouseSystem::Placidus);
        assert!(t.rows.iter().all(|r| r.mid.is_none()));
        // Saat bilinmiyor: tablo yok; girdi yine de denetlenir.
        assert_eq!(
            compute_bhava_table(&mut eph, &input(placidus, false)),
            Ok(None)
        );
        let mut bad = input(CalcSettings::default(), false);
        bad.lat = 91.0;
        assert_eq!(
            compute_bhava_table(&mut eph, &bad),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0
            })
        );
    }

    #[test]
    fn tropical_rasi_mode_and_number_prashna_follow_the_chart() {
        let mut eph = FakeEph::default();
        // Tropikal-rasi: burçlar tropikal (lagna 70°, İkizler 10°), nakshatra sidereal (46°:
        // dizin 13 → 4. nakshatra, Rohini, 2. pada; lord Ay).
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        let t = compute_bhava_table(&mut eph, &input(tropical, true))
            .unwrap()
            .expect("table");
        let mid = t.rows[0].mid.unwrap();
        assert_eq!(mid.sign, Sign::Gemini);
        assert!((mid.deg_in_sign - 10.0).abs() < 1e-9);
        assert_eq!(
            (mid.nakshatra, mid.pada, mid.nakshatra_lord),
            (4, 2, Graha::Moon)
        );
        // Sayı prashnası: ev sistemi ayarı ne olursa olsun tam burç, lagna sayıdan.
        let mut numbered = input(
            CalcSettings {
                house_system: HouseSystem::Placidus,
                ..CalcSettings::default()
            },
            true,
        );
        numbered.lagna_number = Some(30);
        let t = compute_bhava_table(&mut eph, &numbered)
            .unwrap()
            .expect("table");
        assert_eq!(t.system, HouseSystem::WholeSign);
        assert_eq!(t.rows[0].start.sign, Sign::Cancer);
        assert!(t.rows[0].mid.is_some());
    }
}
````

- [ ] **Step 3: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-core --lib bhava 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası (`error[E0432]: unresolved import` ve `error[E0425]`/`E0433`: `bhava_table`, `BhavaTable`, `porphyry_points` henüz yok). Test koşmaz.

- [ ] **Step 4: Uygulama: noktalar, ortalar, saf tablo ve toplayıcı**

Aşağıdaki içerik `table.rs`'in **başına**, test modülünün üstüne, arada bir boş satırla eklenir.

````rust
//! Bhava tablosu: on iki bhavanın başlangıcı, ortası ve bitişi; her grahanın bhavası ve bhava
//! başlangıcına ve ortasına uzaklığı. Boylamlar rasi çerçevesindedir (K5); nakshatra ve pada
//! sidereal boylamdan okunur.
//!
//! Okuma (yol haritası §4.5; `tests/golden/modules/bhava/REVIEW.md`):
//! - **Tam burç:** bhava burcun kendisidir (başlangıç 0°, bitiş 30°). Orta, lagnanın derecesinin
//!   o burçtaki karşılığıdır (lagna + 30° × (n − 1)); Bhava bala da aynı noktayı kullanır.
//! - **Sripati:** ephemeris'in döndürdüğü noktalar bhava başlangıcıdır (sandhi). Orta, Porphyry
//!   noktasıdır: lagna ile MC arasındaki çeyrekler üçe bölünür; 1. bhavanın ortası lagna, 10.
//!   bhavanın ortası MC'dir. Her sandhi iki komşu ortanın tam ortasıdır.
//! - **Placidus:** nokta bhavanın başlangıcıdır (KP okuması); bhava ortası tanımlanmaz (`None`).

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{compute_chart_for, finite, rasi_frame, to_sidereal, ChartWarning};
use crate::dasha::VIMSHOTTARI_ORDER;
use crate::dignity::sign_lord;
use crate::graha::Graha;
use crate::houses::house_from_cusps;
use crate::positions::ChartInput;
use crate::settings::{HouseSystem, ZodiacMode};
use crate::zodiac::{deg_in_sign, nakshatra_pada, normalize, Sign};
use crate::CoreError;

/// Sonucun yöntem kimliği (K7).
pub const BHAVA_METHOD: &str = "bhava_table.v1";

/// Bir bhava noktası (başlangıç, orta ya da bitiş).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct BhavaPoint {
    /// Rasi çerçevesinde boylam, [0, 360).
    pub lon: f64,
    pub sign: Sign,
    /// Burç içi derece, [0, 30).
    pub deg_in_sign: f64,
    /// 1..=27; sidereal boylamdan.
    pub nakshatra: u8,
    /// 1..=4.
    pub pada: u8,
    pub sign_lord: Graha,
    pub nakshatra_lord: Graha,
}

/// Bir bhava: başlangıç, orta ve bitiş. Bitiş, sonraki bhavanın başlangıcıdır.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct BhavaRow {
    /// 1..=12.
    pub house: u8,
    pub start: BhavaPoint,
    /// Placidus'ta `None`.
    pub mid: Option<BhavaPoint>,
    pub end: BhavaPoint,
    /// Başlangıçtan bitişe yay, derece, (0, 360).
    pub span_deg: f64,
}

/// Bir grahanın bhavası.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GrahaBhava {
    pub graha: Graha,
    /// Lagna burcundan sayılan burç evi (K4), 1..=12.
    pub sign_house: u8,
    /// Seçili ev sistemindeki bhava, 1..=12.
    pub bhava: u8,
    /// Bhava başlangıcından grahaya yay, derece, [0, bhavanın yayı).
    pub from_start_deg: f64,
    /// Bhava ortasından grahaya işaretli yay, derece, (−180, 180]: eksi, graha ortadan öncedir.
    /// Placidus'ta `None`.
    pub from_mid_deg: Option<f64>,
    /// Bhava, burç evinden farklı.
    pub shifted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct BhavaTable {
    pub method: String,
    /// Tablonun ev sistemi (`Chart.bhavas.system`): sayı prashnasında ayar ne olursa olsun
    /// `WholeSign`.
    pub system: HouseSystem,
    /// Placidus bu enlemde hesaplanamadı; başlangıç noktaları Porphyry'dir.
    pub porphyry_fallback: bool,
    /// On iki bhava, 1. bhavadan başlayarak.
    pub rows: Vec<BhavaRow>,
    /// Dokuz graha, `Graha::ALL` sırasında.
    pub grahas: Vec<GrahaBhava>,
}

/// Nakshatranın (1..=27) Vimshottari lordu: Ashwini'den başlayan dokuzlu sıra üç kez yinelenir.
pub fn nakshatra_lord(nakshatra: u8) -> Graha {
    VIMSHOTTARI_ORDER[(usize::from(nakshatra) + 8) % 9]
}

/// `from` boylamından `to` boylamına işaretli kısa yay, derece, (−180, 180].
pub fn signed_arc(from: f64, to: f64) -> f64 {
    let d = (to - from).rem_euclid(360.0);
    if d > 180.0 {
        d - 360.0
    } else {
        d
    }
}

/// Rasi çerçevesindeki bir boylamın bhava noktası. Nakshatra ve pada sidereal boylamdandır
/// (`zodiac` tropikal-rasi ise `ayanamsa_deg` düşülür).
pub fn bhava_point(lon: f64, zodiac: ZodiacMode, ayanamsa_deg: f64) -> BhavaPoint {
    let lon = normalize(lon);
    let sign = Sign::of(lon);
    let (nakshatra, pada) = nakshatra_pada(to_sidereal(zodiac, lon, ayanamsa_deg));
    BhavaPoint {
        lon,
        sign,
        deg_in_sign: deg_in_sign(lon),
        nakshatra,
        pada,
        sign_lord: sign_lord(sign),
        nakshatra_lord: nakshatra_lord(nakshatra),
    }
}

/// Tam burç evlerinde bhava ortaları: lagna + 30° × (n − 1).
pub fn whole_sign_mids(lagna_lon: f64) -> [f64; 12] {
    std::array::from_fn(|i| normalize(lagna_lon + 30.0 * i as f64))
}

/// Porphyry noktaları (Sripati'de bhava ortaları): MC'den lagnaya ve lagnadan IC'ye uzanan iki
/// çeyrek üçe bölünür; öteki altı nokta bunların karşısıdır. `[0]` lagna, `[3]` IC, `[6]`
/// batı ufku, `[9]` MC.
pub fn porphyry_points(asc: f64, mc: f64) -> [f64; 12] {
    // MC → lagna yayı (10., 11. ve 12. bhavaların ortaları) ve lagna → IC yayı.
    let upper = normalize(asc - mc);
    let lower = 180.0 - upper;
    let mut p = [0.0; 12];
    for k in 0..3 {
        let thirds = k as f64;
        p[k] = normalize(asc + lower * thirds / 3.0);
        p[9 + k] = normalize(mc + upper * thirds / 3.0);
    }
    for k in 0..6 {
        let opposite = (k + 9) % 12;
        // p[3..=8]: p[9..=11] ve p[0..=2] noktalarının karşısı.
        p[3 + k] = normalize(p[opposite] + 180.0);
    }
    p
}

/// Bhava ortalarından sandhiler: n. bhavanın başlangıcı, (n − 1). ve n. ortanın tam ortasıdır.
pub fn sandhis_from_mids(mids: &[f64; 12]) -> [f64; 12] {
    std::array::from_fn(|i| {
        let prev = mids[(i + 11) % 12];
        normalize(prev + (mids[i] - prev).rem_euclid(360.0) / 2.0)
    })
}

/// Saf tablo. `cusps` on iki bhavanın başlangıcı, `mids` (varsa) ortasıdır; ikisi de rasi
/// çerçevesindedir. `grahas`, `Graha::ALL` sırasında rasi çerçevesindeki boylamlardır.
pub fn bhava_table(
    system: HouseSystem,
    cusps: &[f64; 12],
    mids: Option<&[f64; 12]>,
    lagna_sign: Sign,
    grahas: &[(Graha, f64); 9],
    zodiac: ZodiacMode,
    ayanamsa_deg: f64,
) -> BhavaTable {
    let point = |lon: f64| bhava_point(lon, zodiac, ayanamsa_deg);
    let rows = (0..12)
        .map(|i| {
            let end = cusps[(i + 1) % 12];
            BhavaRow {
                house: (i + 1) as u8,
                start: point(cusps[i]),
                mid: mids.map(|m| point(m[i])),
                end: point(end),
                span_deg: (end - cusps[i]).rem_euclid(360.0),
            }
        })
        .collect();
    let grahas = grahas
        .iter()
        .map(|&(graha, lon)| {
            let bhava = house_from_cusps(lon, cusps);
            let at = usize::from(bhava) - 1;
            let sign_house = Sign::of(lon).house_from(lagna_sign);
            GrahaBhava {
                graha,
                sign_house,
                bhava,
                from_start_deg: (lon - cusps[at]).rem_euclid(360.0),
                from_mid_deg: mids.map(|m| signed_arc(m[at], lon)),
                shifted: bhava != sign_house,
            }
        })
        .collect();
    BhavaTable {
        method: BHAVA_METHOD.to_owned(),
        system,
        porphyry_fallback: false,
        rows,
        grahas,
    }
}

/// `compute_bhava_table` komutunun hesabı. Doğum saati bilinmiyorsa (`input.time_known` yanlış)
/// `None` döner (K2); ayar, an ve konum denetimi (`compute_chart`) her durumda yapılır. Ev
/// sistemi haritanınkidir (`Chart.bhavas.system`). Sripati'de MC için ephemeris'ten evler bir kez
/// daha istenir.
pub fn compute_bhava_table<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<Option<BhavaTable>, CoreError> {
    let chart = compute_chart_for(eph, input)?;
    if !input.time_known {
        return Ok(None);
    }
    let cusps: [f64; 12] = chart
        .bhavas
        .cusps
        .as_slice()
        .try_into()
        .map_err(|_| CoreError::NonFiniteResult("bhavas"))?;
    let mids = match chart.bhavas.system {
        HouseSystem::WholeSign => Some(whole_sign_mids(chart.lagna.lon)),
        HouseSystem::Sripati => {
            let houses = eph.houses(
                input.jd_ut,
                input.lat,
                input.lon,
                HouseSystem::Sripati.ephemeris_system(),
                rasi_frame(&input.settings),
            )?;
            finite(houses.mc, "bhavas")?;
            Some(porphyry_points(chart.lagna.lon, houses.mc))
        }
        HouseSystem::Placidus => None,
    };
    // `Chart::grahas` her zaman 9 elemandır ve `Graha::ALL` sırasındadır.
    let grahas: [(Graha, f64); 9] =
        std::array::from_fn(|i| (chart.grahas[i].graha, chart.grahas[i].lon));
    let mut table = bhava_table(
        chart.bhavas.system,
        &cusps,
        mids.as_ref(),
        chart.lagna.sign,
        &grahas,
        input.settings.zodiac,
        chart.ayanamsa_deg,
    );
    table.porphyry_fallback = chart
        .meta
        .warnings
        .contains(&ChartWarning::PlacidusPolarFallback);
    Ok(Some(table))
}
````

- [ ] **Step 5: Çekirdek testleri ve kapılar**

```bash
cargo fmt --all --check
cargo test -p jyotish-core --lib bhava 2>&1 | grep -E "^test result|FAILED|panicked"
cargo test -p jyotish-core --lib 2>&1 | grep -E "^test result"
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: fmt çıktısız; süzgeçli koşu `ok` (8 `bhava::table` testi; süzgeç `shadbala::bhava` testlerini de sayar); lib `412 passed`; clippy çıktısız; sayaç `0`. Bir el örneği düşerse önce test yorumundaki türetme yeniden okunur; beklenen değer motor çıktısına bakarak değiştirilmez.

- [ ] **Step 6: Commit**

```bash
git add crates/jyotish-core/src/bhava crates/jyotish-core/src/lib.rs
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): bhava table with start, middle and end of the twelve bhavas and each graha's bhava and distances" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: üç dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 2: Çekirdek: Sudarshana chakra (üç halka; saf fonksiyon ve toplayıcı)

Yol haritası §4.6; K2–K4, K7; D14. Hesap `Positions` üzerinde saf bir sayımdır; toplayıcı `compute_positions`'ı kullanır (sayı prashnasında lagna sayıdan, saat bilinmiyorsa lagna boş gelir).

**Files:**
- Create: `crates/jyotish-core/src/bhava/sudarshana.rs`
- Modify: `crates/jyotish-core/src/bhava/mod.rs`, `crates/jyotish-core/src/lib.rs`
- Test: `sudarshana.rs` `mod tests` (3)

**Interfaces:**
- Consumes: `positions::{compute_positions, ChartInput, Positions}` (`Positions::occupants`, `Positions::sign_of`, `Positions::synthetic`); `zodiac::Sign`; `graha::Graha`.
- Produces (`jyotish_core` kökünden de dışa aktarılır):
  - `SudarshanaRef { Lagna, Moon, Sun }` (`SudarshanaRef::ALL`: içten dışa halka sırası).
  - `SudarshanaHouse { house: u8, sign: Sign, grahas: Vec<Graha> }`, `SudarshanaRing { reference: SudarshanaRef, houses: Option<Vec<SudarshanaHouse>> }`, `SudarshanaChakra { method: String, rings: Vec<SudarshanaRing>, approximate: bool }` (her zaman üç halka).
  - `sudarshana(positions: &Positions) -> SudarshanaChakra`, `compute_sudarshana<E: Ephemeris + ?Sized>(eph: &mut E, input: &ChartInput) -> Result<SudarshanaChakra, CoreError>`.
  - `SUDARSHANA_METHOD = "sudarshana.v1"`.

- [ ] **Step 1: Testler: üç halkanın el örneği, saat bilinmeyen harita, toplayıcı; modül kaydı**

El örneği: lagna Akrep 5°; Güneş Koç 10°, Ay Yengeç 5°, Mars Akrep 20°, Merkür Koç 20°, Jüpiter Yay 10°, Venüs Boğa 10°, Satürn Kova 0°, Rahu Aslan 0°, Ketu Kova 0°. Her halkanın evleri burç sayımıyla elle türetilmiştir (test yorumları).

````diff
diff --git a/crates/jyotish-core/src/bhava/mod.rs b/crates/jyotish-core/src/bhava/mod.rs
index 9fc808f..c3ade0b 100644
--- a/crates/jyotish-core/src/bhava/mod.rs
+++ b/crates/jyotish-core/src/bhava/mod.rs
@@ -1,9 +1,15 @@
-//! Bhava tablosu (yol haritası Plan 25; §4.5). Saf hesap ve ince toplayıcısı: `table` seçili ev
-//! sisteminde on iki bhavanın başlangıcını, ortasını ve bitişini ve her grahanın bhavasını verir.
-//! Sonuç `Chart`'a eklenmez; panel açıkken istenir (K1).
+//! Bhava tablosu ve Sudarshana chakra (yol haritası Plan 25; §4.5 ve §4.6). İki saf hesap ve
+//! ince toplayıcıları: `table` seçili ev sisteminde on iki bhavanın başlangıcını, ortasını ve
+//! bitişini ve her grahanın bhavasını verir; `sudarshana` lagna, Ay ve Güneş'i birinci ev sayan
+//! üç halkayı verir. Sonuçlar `Chart`'a eklenmez; panel açıkken istenir (K1).
 
+mod sudarshana;
 mod table;
 
+pub use sudarshana::{
+    compute_sudarshana, sudarshana, SudarshanaChakra, SudarshanaHouse, SudarshanaRef,
+    SudarshanaRing, SUDARSHANA_METHOD,
+};
 pub use table::{
     bhava_point, bhava_table, compute_bhava_table, nakshatra_lord, porphyry_points,
     sandhis_from_mids, signed_arc, whole_sign_mids, BhavaPoint, BhavaRow, BhavaTable, GrahaBhava,
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index d1a9665..352c5aa 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -10,13 +10,14 @@
 //! Prashna (Alt proje 3.8): `prashna` (sayı prashnasının lagnası, soru haritasının olguları).
 //! Arama (Alt proje 3.9): `search` (harita araması sorgusu ve hafif yol, olay araması).
 
-// Bhava tablosu (yol haritası Plan 25). Ayrı blok: paralel planların alfabetik sıradaki
-// eklemeleriyle aynı satırlara dokunmaz.
+// Bhava tablosu ve Sudarshana chakra (yol haritası Plan 25). Ayrı blok: paralel planların
+// alfabetik sıradaki eklemeleriyle aynı satırlara dokunmaz.
 pub mod bhava;
 pub use bhava::{
-    bhava_point, bhava_table, compute_bhava_table, nakshatra_lord, porphyry_points,
-    sandhis_from_mids, signed_arc, whole_sign_mids, BhavaPoint, BhavaRow, BhavaTable, GrahaBhava,
-    BHAVA_METHOD,
+    bhava_point, bhava_table, compute_bhava_table, compute_sudarshana, nakshatra_lord,
+    porphyry_points, sandhis_from_mids, signed_arc, sudarshana, whole_sign_mids, BhavaPoint,
+    BhavaRow, BhavaTable, GrahaBhava, SudarshanaChakra, SudarshanaHouse, SudarshanaRef,
+    SudarshanaRing, BHAVA_METHOD, SUDARSHANA_METHOD,
 };
 
 pub mod ashtakavarga;
````

**Yeni dosya:** `crates/jyotish-core/src/bhava/sudarshana.rs` (şimdilik yalnızca test modülü)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::CalcSettings;

    /// El örneği: lagna Akrep 5°; Güneş Koç 10°, Ay Yengeç 5°, Mars Akrep 20°, Merkür Koç
    /// 20°, Jüpiter Yay 10°, Venüs Boğa 10°, Satürn Kova 0°, Rahu Aslan 0°, Ketu Kova 0°.
    fn sample() -> Positions {
        Positions::synthetic(
            Some(215.0),
            [10.0, 95.0, 230.0, 20.0, 250.0, 40.0, 300.0, 120.0, 300.0],
        )
    }

    fn house(ring: &SudarshanaRing, house: u8) -> &SudarshanaHouse {
        &ring.houses.as_ref().expect("houses")[usize::from(house) - 1]
    }

    #[test]
    fn three_rings_count_from_the_lagna_the_moon_and_the_sun() {
        let c = sudarshana(&sample());
        assert_eq!(c.method, SUDARSHANA_METHOD);
        assert!(!c.approximate);
        let refs: Vec<SudarshanaRef> = c.rings.iter().map(|r| r.reference).collect();
        assert_eq!(refs, SudarshanaRef::ALL);
        let (lagna, moon, sun) = (&c.rings[0], &c.rings[1], &c.rings[2]);
        // Lagna halkası: 1. ev Akrep (Mars); 2. ev Yay (Jüpiter); 4. ev Kova (Satürn, Ketu);
        // 6. ev Koç (Güneş, Merkür); 7. ev Boğa (Venüs); 9. ev Yengeç (Ay); 10. ev Aslan (Rahu).
        assert_eq!(house(lagna, 1).sign, Sign::Scorpio);
        assert_eq!(house(lagna, 1).grahas, [Graha::Mars]);
        assert_eq!(house(lagna, 2).grahas, [Graha::Jupiter]);
        assert_eq!(house(lagna, 4).grahas, [Graha::Saturn, Graha::Ketu]);
        assert_eq!(house(lagna, 6).grahas, [Graha::Sun, Graha::Mercury]);
        assert_eq!(house(lagna, 7).grahas, [Graha::Venus]);
        assert_eq!(house(lagna, 9).grahas, [Graha::Moon]);
        assert_eq!(house(lagna, 10).grahas, [Graha::Rahu]);
        // Ay halkası: 1. ev Yengeç (Ay); 2. ev Aslan (Rahu); 5. ev Akrep (Mars); 6. ev Yay
        // (Jüpiter); 8. ev Kova (Satürn, Ketu); 10. ev Koç (Güneş, Merkür); 11. ev Boğa (Venüs).
        assert_eq!(house(moon, 1).sign, Sign::Cancer);
        assert_eq!(house(moon, 1).grahas, [Graha::Moon]);
        assert_eq!(house(moon, 2).grahas, [Graha::Rahu]);
        assert_eq!(house(moon, 5).grahas, [Graha::Mars]);
        assert_eq!(house(moon, 6).grahas, [Graha::Jupiter]);
        assert_eq!(house(moon, 8).grahas, [Graha::Saturn, Graha::Ketu]);
        assert_eq!(house(moon, 10).grahas, [Graha::Sun, Graha::Mercury]);
        assert_eq!(house(moon, 11).grahas, [Graha::Venus]);
        // Güneş halkası: 1. ev Koç (Güneş, Merkür); 2. ev Boğa (Venüs); 4. ev Yengeç (Ay);
        // 5. ev Aslan (Rahu); 8. ev Akrep (Mars); 9. ev Yay (Jüpiter); 11. ev Kova (Satürn, Ketu).
        assert_eq!(house(sun, 1).sign, Sign::Aries);
        assert_eq!(house(sun, 1).grahas, [Graha::Sun, Graha::Mercury]);
        assert_eq!(house(sun, 2).grahas, [Graha::Venus]);
        assert_eq!(house(sun, 4).grahas, [Graha::Moon]);
        assert_eq!(house(sun, 5).grahas, [Graha::Rahu]);
        assert_eq!(house(sun, 8).grahas, [Graha::Mars]);
        assert_eq!(house(sun, 9).grahas, [Graha::Jupiter]);
        assert_eq!(house(sun, 11).grahas, [Graha::Saturn, Graha::Ketu]);
        // Her halkada on iki ev, on iki ayrı burç ve dokuz grahanın her biri tam bir kez.
        for ring in &c.rings {
            let houses = ring.houses.as_ref().expect("houses");
            assert_eq!(houses.len(), 12);
            for (i, h) in houses.iter().enumerate() {
                assert_eq!(usize::from(h.house), i + 1);
                assert_eq!(h.sign, Sign::from_index(houses[0].sign.index() + i));
            }
            let mut all: Vec<Graha> = houses.iter().flat_map(|h| h.grahas.clone()).collect();
            all.sort();
            assert_eq!(all, Graha::ALL);
        }
    }

    #[test]
    fn without_a_birth_time_the_lagna_ring_is_empty_and_the_result_is_approximate() {
        let c = sudarshana(&sample().without_lagna());
        assert!(c.approximate);
        assert_eq!(c.rings.len(), 3);
        assert_eq!(c.rings[0].reference, SudarshanaRef::Lagna);
        assert_eq!(c.rings[0].houses, None);
        assert_eq!(house(&c.rings[1], 1).sign, Sign::Cancer);
        assert_eq!(house(&c.rings[2], 1).sign, Sign::Aries);
    }

    #[test]
    fn collector_reads_the_chart_positions() {
        let mut eph = FakeEph::default();
        let mut input = ChartInput {
            jd_ut: 2_451_545.0,
            lat: 41.0,
            lon: 29.0,
            settings: CalcSettings::default(),
            time_known: true,
            lagna_number: None,
        };
        let positions = compute_positions(&mut eph, &input).unwrap();
        let c = compute_sudarshana(&mut eph, &input).unwrap();
        assert_eq!(c, sudarshana(&positions));
        // FakeEph: lagna sidereal 46° (Boğa), Ay 71° (İkizler), Güneş 346° (Balık).
        assert_eq!(house(&c.rings[0], 1).sign, Sign::Taurus);
        assert_eq!(house(&c.rings[1], 1).sign, Sign::Gemini);
        assert_eq!(house(&c.rings[2], 1).sign, Sign::Pisces);
        // Sayı prashnası: lagna halkası sayının lagnasından (30 → Yengeç).
        input.lagna_number = Some(30);
        let numbered = compute_sudarshana(&mut eph, &input).unwrap();
        assert_eq!(house(&numbered.rings[0], 1).sign, Sign::Cancer);
        input.time_known = false;
        let unknown = compute_sudarshana(&mut eph, &input).unwrap();
        assert!(unknown.approximate);
        assert_eq!(unknown.rings[0].houses, None);
        input.lat = 91.0;
        assert_eq!(
            compute_sudarshana(&mut eph, &input),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0
            })
        );
    }
}
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-core --lib sudarshana 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası (`error[E0432]: unresolved import` ve `error[E0425]`/`E0433`: `sudarshana`, `SudarshanaChakra`, `SudarshanaRef` henüz yok). Test koşmaz.

- [ ] **Step 3: Uygulama: tipler, saf hesap ve toplayıcı**

Aşağıdaki içerik `sudarshana.rs`'in **başına**, test modülünün üstüne, arada bir boş satırla eklenir.

````rust
//! Sudarshana chakra (yol haritası §4.6): lagna, Ay ve Güneş'i birinci ev sayan üç halka. Her
//! halkada on iki ev vardır; ev, o halkanın başlangıç burcundan sayılan burç evidir (K4) ve
//! içindeki grahalar rasi çerçevesindeki burçlarından okunur. Halka sırası içten dışadır: lagna,
//! Ay, Güneş. Sudarshana dashası hesaplanmaz.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::positions::{compute_positions, ChartInput, Positions};
use crate::zodiac::Sign;
use crate::CoreError;

/// Sonucun yöntem kimliği (K7).
pub const SUDARSHANA_METHOD: &str = "sudarshana.v1";

/// Halkanın birinci evini veren nokta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum SudarshanaRef {
    Lagna,
    Moon,
    Sun,
}

impl SudarshanaRef {
    /// İçten dışa halka sırası.
    pub const ALL: [SudarshanaRef; 3] = [
        SudarshanaRef::Lagna,
        SudarshanaRef::Moon,
        SudarshanaRef::Sun,
    ];
}

/// Bir halkanın bir evi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SudarshanaHouse {
    /// 1..=12.
    pub house: u8,
    pub sign: Sign,
    /// Bu burçtaki grahalar, `Graha::ALL` sırasında.
    pub grahas: Vec<Graha>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SudarshanaRing {
    pub reference: SudarshanaRef,
    /// On iki ev, 1. evden başlayarak. Lagna halkasında doğum saati bilinmiyorsa `None` (K2).
    pub houses: Option<Vec<SudarshanaHouse>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SudarshanaChakra {
    pub method: String,
    /// Üç halka, `SudarshanaRef::ALL` sırasında (içten dışa).
    pub rings: Vec<SudarshanaRing>,
    /// Doğum saati bilinmiyor: Ay'ın burcu o gün değişmiş olabilir.
    pub approximate: bool,
}

fn ring_houses(positions: &Positions, first: Sign) -> Vec<SudarshanaHouse> {
    (0..12)
        .map(|i| {
            let sign = Sign::from_index(first.index() + i);
            SudarshanaHouse {
                house: (i + 1) as u8,
                sign,
                grahas: positions.occupants(sign),
            }
        })
        .collect()
}

/// Saf hesap: üç halka. Lagna yoksa (doğum saati bilinmiyor) lagna halkasının evleri `None`
/// döner ve sonuç `approximate` işaretlenir.
pub fn sudarshana(positions: &Positions) -> SudarshanaChakra {
    let rings = SudarshanaRef::ALL
        .into_iter()
        .map(|reference| {
            let first = match reference {
                SudarshanaRef::Lagna => positions.lagna_sign,
                SudarshanaRef::Moon => Some(positions.sign_of(Graha::Moon)),
                SudarshanaRef::Sun => Some(positions.sign_of(Graha::Sun)),
            };
            SudarshanaRing {
                reference,
                houses: first.map(|sign| ring_houses(positions, sign)),
            }
        })
        .collect();
    SudarshanaChakra {
        method: SUDARSHANA_METHOD.to_owned(),
        rings,
        approximate: positions.lagna_sign.is_none(),
    }
}

/// `compute_sudarshana` komutunun hesabı: temel haritayı hesaplar ve `sudarshana`'ya verir.
/// Doğum saati bilinmiyorsa lagna halkası boştur; sayı prashnasında lagna sayıdan gelir.
pub fn compute_sudarshana<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<SudarshanaChakra, CoreError> {
    Ok(sudarshana(&compute_positions(eph, input)?))
}
````

- [ ] **Step 4: Testler ve kapılar**

```bash
cargo fmt --all --check
cargo test -p jyotish-core --lib sudarshana 2>&1 | grep -E "^test result|FAILED|panicked"
cargo test -p jyotish-core --lib 2>&1 | grep -E "^test result"
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: fmt çıktısız; süzgeçli koşu `3 passed`; lib `415 passed`; clippy çıktısız; sayaç `0`.

- [ ] **Step 5: Commit**

```bash
git add crates/jyotish-core/src/bhava crates/jyotish-core/src/lib.rs
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): sudarshana chakra as three rings of sign houses from the lagna, the Moon and the Sun" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: üç dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 3: Uygulama: iki komut, gerçek motorla denetim, bağlamalar ve fikstür

Yol haritası §4.5 ("yeni komut gerekir"); K1, C1, C2′, C10; D9, D16. Komutların gövdesi `module_commands` kalıbındadır: tek `ChartInput`, hesap motor iş parçacığında, hata `CommandError`. Bu görev üretilen iki dosyayı yeniden üretir ve arayüzün sarmalayıcılarını, fikstür tipini ve e2e taklidini de ekler (yeni komutun sarmalayıcısı yoksa `api.test.ts` zaten düşer; taklidi yoksa sonraki görevlerin e2e akışları "no mock" hatası alır).

**Files:**
- Create: `app/src-tauri/src/bhava_commands.rs`, `app/src-tauri/tests/bhava.rs`
- Modify: `app/src-tauri/src/lib.rs`, `app/src-tauri/tests/ui_fixture.rs`, `app/ui/api.ts`, `app/ui/api.test.ts`, `app/ui/test/fixture.ts`, `app/ui/test/fixture.test.ts`, `app/e2e/ipc.ts`
- Generated (komutla): `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`
- Test: `tests/bhava.rs` (1; gerçek motor); `api.test.ts` (var olan tabloya iki satır); `fixture.test.ts` (1 yeni test)

**Interfaces:**
- Consumes: Task 1–2'nin `compute_bhava_table`, `compute_sudarshana`, `porphyry_points`, `sandhis_from_mids`, `signed_arc`, tipler; `client_commands::blocking` (`pub(crate)`); `commands::{CommandError, compute_chart_for_blocking}`; `ephemeris::{Engine, SwissEph}`.
- Produces:
  - `bhava_commands::compute_bhava_table_blocking(engine: &Engine, input: ChartInput) -> Result<Option<BhavaTable>, CommandError>` ve komutu `compute_bhava_table(input)`.
  - `bhava_commands::compute_sudarshana_blocking(engine: &Engine, input: ChartInput) -> Result<SudarshanaChakra, CommandError>` ve komutu `compute_sudarshana(input)`.
  - Bağlamalar: `commands.computeBhavaTable`, `commands.computeSudarshana`; tipler `BhavaTable`, `BhavaRow`, `BhavaPoint`, `GrahaBhava`, `SudarshanaChakra`, `SudarshanaRing`, `SudarshanaHouse`, `SudarshanaRef`. Sayı alanları bağlamalarda `number | null`'dır (Rust'ta sonlu olmayan değer `null` olur).
  - `api.computeBhavaTable(input)`, `api.computeSudarshana(input)`.
  - Fikstür: `UiFixture.bhava` (`chart_placidus`, `table`, `table_sripati`, `table_placidus`, `table_placidus_polar`, `sudarshana`, `sudarshana_time_unknown`, `sudarshana_refs`); `placidusOpened()`.
  - E2e: `mockIpc(page, { placidus: true })`; `compute_bhava_table` ve `compute_sudarshana` taklitleri.

- [ ] **Step 1: Testler: gerçek motorla tablo ve chakra; sarmalayıcı tablosu; fikstür beklentisi**

`tests/bhava.rs` beş golden anını üç ev sistemi ve iki zodyak kipiyle dolaşır: tablo haritanın başlangıçlarını ve graha evlerini izler; Sripati sandhileri Porphyry ortalarının ikişer ikişer ortasıdır (D2'nin ikinci yazımı); tam burçta hiçbir graha yer değiştirmez; Placidus'ta orta yoktur ve yalnızca Tromsø Porphyry'ye düşer; chakra haritanın lagna, Ay ve Güneş burcundan sayar; saat bilinmeyen harita, sayı prashnası ve hata eşlemesi.

**Yeni dosya:** `app/src-tauri/tests/bhava.rs`

````rust
//! Bhava tablosu ve Sudarshana chakra komut gövdeleri, gerçek `Engine` ile uçtan uca (yol
//! haritası Plan 25). Tek test: süreçte yalnızca bir SE örneği açılabilir.

use std::path::PathBuf;

use ephemeris::{Engine, Ephemeris, SwissEph, Zodiac};
use jyotish_core::{
    porphyry_points, sandhis_from_mids, signed_arc, CalcSettings, ChartInput, Graha, HouseSystem,
    Sign, SudarshanaRef, ZodiacMode, BHAVA_METHOD, SUDARSHANA_METHOD,
};
use jyotish_desk_lib::bhava_commands::{compute_bhava_table_blocking, compute_sudarshana_blocking};
use jyotish_desk_lib::commands::compute_chart_for_blocking;
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

fn with_system(input: &ChartInput, system: HouseSystem, zodiac: ZodiacMode) -> ChartInput {
    ChartInput {
        settings: CalcSettings {
            house_system: system,
            zodiac,
            ..input.settings.clone()
        },
        ..input.clone()
    }
}

#[test]
fn bhava_commands_run_on_the_engine_thread_and_agree_with_the_chart() {
    let engine = Engine::spawn(root().join("../../data/ephe")).expect("spawn engine");
    let inputs = golden_inputs();
    assert_eq!(inputs.len(), 5);
    let systems = [
        HouseSystem::WholeSign,
        HouseSystem::Sripati,
        HouseSystem::Placidus,
    ];
    let zodiacs = [
        ZodiacMode::Sidereal,
        ZodiacMode::TropicalRasiSiderealNakshatra,
    ];
    let mut fallbacks = 0;
    let mut shifted_charts = 0;
    for (id, base) in &inputs {
        for system in systems {
            for zodiac in zodiacs {
                let input = with_system(base, system, zodiac);
                let chart = compute_chart_for_blocking(&engine, input.clone()).expect("chart");
                let table = compute_bhava_table_blocking(&engine, input.clone())
                    .expect("bhava table")
                    .expect("time known");
                let ctx = format!("{id} {system:?} {zodiac:?}");
                assert_eq!(table.method, BHAVA_METHOD, "{ctx}");
                assert_eq!(table.system, chart.bhavas.system, "{ctx}");
                assert_eq!((table.rows.len(), table.grahas.len()), (12, 9), "{ctx}");
                // Başlangıçlar haritanın bhava başlangıçlarıdır; bitiş sonrakinin başlangıcıdır;
                // yaylar çemberi tam bir kez dolanır.
                for (i, row) in table.rows.iter().enumerate() {
                    assert_eq!(usize::from(row.house), i + 1, "{ctx}");
                    assert_eq!(row.start.lon, chart.bhavas.cusps[i], "{ctx}");
                    assert_eq!(row.end, table.rows[(i + 1) % 12].start, "{ctx}");
                    assert!(row.span_deg > 0.0 && row.span_deg < 180.0, "{ctx}");
                    assert_eq!(row.mid.is_some(), system != HouseSystem::Placidus, "{ctx}");
                }
                let total: f64 = table.rows.iter().map(|r| r.span_deg).sum();
                assert!((total - 360.0).abs() < 1e-6, "{ctx}: {total}");
                // Grahanın bhavası graha tablosundaki evdir; burç evi lagna burcundan sayılır.
                for (g, c) in table.grahas.iter().zip(&chart.grahas) {
                    assert_eq!((g.graha, g.bhava), (c.graha, c.house), "{ctx}");
                    assert_eq!(g.sign_house, c.sign.house_from(chart.lagna.sign), "{ctx}");
                    assert_eq!(g.shifted, g.bhava != g.sign_house, "{ctx}");
                    let row = &table.rows[usize::from(g.bhava) - 1];
                    assert!(
                        g.from_start_deg >= 0.0 && g.from_start_deg < row.span_deg,
                        "{ctx} {:?}",
                        g.graha
                    );
                    assert_eq!(g.from_mid_deg.is_some(), row.mid.is_some(), "{ctx}");
                    if let (Some(d), Some(mid)) = (g.from_mid_deg, row.mid) {
                        assert!((d - signed_arc(mid.lon, c.lon)).abs() < 1e-9, "{ctx}");
                    }
                }
                match system {
                    HouseSystem::WholeSign => {
                        // Tam burç: hiçbir graha yer değiştirmez; orta lagnanın derecesidir.
                        assert!(table.grahas.iter().all(|g| !g.shifted), "{ctx}");
                        for row in &table.rows {
                            assert_eq!(row.start.deg_in_sign, 0.0, "{ctx}");
                            let mid = row.mid.expect("mid");
                            assert!(
                                (mid.deg_in_sign - chart.lagna.deg_in_sign).abs() < 1e-9,
                                "{ctx}"
                            );
                        }
                    }
                    HouseSystem::Sripati => {
                        // İkinci yazım: ephemeris'in Sripati sandhileri, lagna ile MC'den kurulan
                        // Porphyry ortalarının ikişer ikişer ortasıdır.
                        let (jd, lat, lon) = (input.jd_ut, input.lat, input.lon);
                        let frame = match zodiac {
                            ZodiacMode::Sidereal => {
                                Zodiac::Sidereal(input.settings.ayanamsa.sidereal_mode())
                            }
                            ZodiacMode::TropicalRasiSiderealNakshatra => Zodiac::Tropical,
                        };
                        let houses = engine
                            .call(move |se: &mut SwissEph| {
                                se.houses(jd, lat, lon, ephemeris::HouseSystem::Sripati, frame)
                            })
                            .expect("engine")
                            .expect("houses");
                        let mids = porphyry_points(houses.asc, houses.mc);
                        let sandhis = sandhis_from_mids(&mids);
                        for i in 0..12 {
                            assert!(
                                signed_arc(sandhis[i], chart.bhavas.cusps[i]).abs() < 1e-6,
                                "{ctx}: sandhi {} {} vs {}",
                                i + 1,
                                sandhis[i],
                                chart.bhavas.cusps[i]
                            );
                            let mid = table.rows[i].mid.expect("mid");
                            assert!(signed_arc(mids[i], mid.lon).abs() < 1e-9, "{ctx}");
                            // Orta, bhavanın içindedir.
                            let from_start = (mid.lon - table.rows[i].start.lon).rem_euclid(360.0);
                            assert!(from_start < table.rows[i].span_deg, "{ctx}");
                        }
                        // 1. bhavanın ortası lagnadır.
                        let first = table.rows[0].mid.expect("mid");
                        assert!(signed_arc(first.lon, chart.lagna.lon).abs() < 1e-9, "{ctx}");
                        if table.grahas.iter().any(|g| g.shifted) {
                            shifted_charts += 1;
                        }
                    }
                    HouseSystem::Placidus => {
                        assert!(
                            table.grahas.iter().all(|g| g.from_mid_deg.is_none()),
                            "{ctx}"
                        );
                        if table.porphyry_fallback {
                            fallbacks += 1;
                            assert!(id.starts_with("tromso"), "{ctx}");
                        }
                    }
                }
                // Nakshatra ve pada sidereal boylamdandır: lagnanın noktası haritanın lagnasıyla
                // aynı nakshatrayı verir (tam burç ve Sripati'de 1. bhavanın ortası lagnadır).
                if let Some(mid) = table.rows[0].mid {
                    assert_eq!(
                        (mid.nakshatra, mid.pada),
                        (chart.lagna.nakshatra, chart.lagna.pada),
                        "{ctx}"
                    );
                }

                // Sudarshana: üç halka, haritanın lagna, Ay ve Güneş burcundan.
                let chakra = compute_sudarshana_blocking(&engine, input.clone()).expect("chakra");
                assert_eq!(chakra.method, SUDARSHANA_METHOD, "{ctx}");
                assert!(!chakra.approximate, "{ctx}");
                let firsts: Vec<(SudarshanaRef, Sign)> = chakra
                    .rings
                    .iter()
                    .map(|r| (r.reference, r.houses.as_ref().expect("houses")[0].sign))
                    .collect();
                assert_eq!(
                    firsts,
                    [
                        (SudarshanaRef::Lagna, chart.lagna.sign),
                        (SudarshanaRef::Moon, chart.grahas[Graha::Moon as usize].sign),
                        (SudarshanaRef::Sun, chart.grahas[Graha::Sun as usize].sign),
                    ],
                    "{ctx}"
                );
                for ring in &chakra.rings {
                    let houses = ring.houses.as_ref().expect("houses");
                    for c in &chart.grahas {
                        let at = houses
                            .iter()
                            .find(|h| h.grahas.contains(&c.graha))
                            .expect("every graha is in a house");
                        assert_eq!(at.sign, c.sign, "{ctx}");
                        assert_eq!(at.house, c.sign.house_from(houses[0].sign), "{ctx}");
                    }
                }
            }
        }
    }
    // Tromsø (kutup dairesi) Placidus'ta Porphyry'ye düşer: iki zodyak kipinde birer kez.
    assert_eq!(fallbacks, 2);
    // Sripati'de en az bir haritada bir graha burç evinden başka bir bhavadadır.
    assert!(shifted_charts > 0);

    // Doğum saati bilinmiyor: tablo yok; chakra lagna halkası olmadan ve yaklaşık döner.
    let mut unknown = inputs[0].1.clone();
    unknown.time_known = false;
    assert_eq!(
        compute_bhava_table_blocking(&engine, unknown.clone()).expect("no table"),
        None
    );
    let chakra = compute_sudarshana_blocking(&engine, unknown.clone()).expect("chakra");
    assert!(chakra.approximate);
    assert_eq!(chakra.rings[0].houses, None);
    assert!(chakra.rings[1].houses.is_some() && chakra.rings[2].houses.is_some());

    // Sayı prashnası: ev sistemi ayarı ne olursa olsun tam burç.
    let mut numbered = with_system(&inputs[0].1, HouseSystem::Placidus, ZodiacMode::Sidereal);
    numbered.lagna_number = Some(54);
    let table = compute_bhava_table_blocking(&engine, numbered)
        .expect("bhava table")
        .expect("time known");
    assert_eq!(table.system, HouseSystem::WholeSign);
    assert!(!table.porphyry_fallback);

    // Hata eşlemesi: geçersiz konum, saat bilinmese de bildirilir.
    unknown.lat = 91.0;
    let e = compute_bhava_table_blocking(&engine, unknown.clone()).expect_err("invalid location");
    assert_eq!(e.code, "invalid_location");
    let e = compute_sudarshana_blocking(&engine, unknown).expect_err("invalid location");
    assert_eq!(e.code, "invalid_location");
}
````

````diff
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index abf51e5..b33ea5d 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -28,6 +28,8 @@ vi.mock("./bindings", () => ({
     exportClientsCsv: vi.fn(),
     importClientsJson: vi.fn(),
     dashaChildren: vi.fn(),
+    computeBhavaTable: vi.fn(),
+    computeSudarshana: vi.fn(),
     computeDrishti: vi.fn(),
     computeJaimini: vi.fn(),
     computeCharaDasha: vi.fn(),
@@ -133,6 +135,8 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["importClientsJson", []],
     ["dashaChildren", [{ birth_jd_ut: 1 }, ["Sun"]]],
     ["computeDrishti", [{ jd_ut: 1, time_known: true }]],
+    ["computeBhavaTable", [{ jd_ut: 1, time_known: true }]],
+    ["computeSudarshana", [{ jd_ut: 1, time_known: false }]],
     ["computeJaimini", [{ jd_ut: 1, time_known: false }]],
     ["computeCharaDasha", [{ jd_ut: 1, time_known: true }]],
     ["charaDashaChildren", [{ jd_ut: 1, time_known: true }, ["Aries", "Taurus"], 2]],
diff --git a/app/ui/test/fixture.test.ts b/app/ui/test/fixture.test.ts
index 9e16dd3..973646c 100644
--- a/app/ui/test/fixture.test.ts
+++ b/app/ui/test/fixture.test.ts
@@ -1,4 +1,4 @@
-import { fixture, unknownTimeOpened } from "./fixture";
+import { fixture, placidusOpened, unknownTimeOpened } from "./fixture";
 
 test("the fixture is a real Delhi chart with three dasha levels and atlas hits", () => {
   const f = fixture();
@@ -12,6 +12,19 @@ test("the fixture is a real Delhi chart with three dasha levels and atlas hits",
   expect(f.gap_error).toEqual({ code: "nonexistent_local_time", params: { local: "2014-03-31T03:30:00", suggestion: "2014-03-31T04:30:00" } });
 });
 
+test("the bhava fixture has the three house systems of the open chart and the polar fallback", () => {
+  const f = fixture();
+  expect([f.bhava.table.system, f.bhava.table_sripati.system, f.bhava.table_placidus.system]).toEqual(["WholeSign", "Sripati", "Placidus"]);
+  expect(f.bhava.table_placidus.grahas.filter((g) => g.shifted).map((g) => g.graha)).toEqual(["Mercury", "Jupiter", "Venus"]);
+  expect(f.bhava.table_sripati.rows.every((r) => r.mid !== null)).toBe(true);
+  expect(f.bhava.table_placidus.rows.every((r) => r.mid === null)).toBe(true);
+  expect(f.bhava.table_placidus_polar.porphyry_fallback).toBe(true);
+  expect(placidusOpened().chart.bhavas.system).toBe("Placidus");
+  expect(placidusOpened().chart.jd_ut).toBe(f.opened.chart.jd_ut);
+  expect(f.bhava.sudarshana.rings.map((r) => r.reference)).toEqual(f.bhava.sudarshana_refs);
+  expect(f.bhava.sudarshana_time_unknown.rings[0].houses).toBeNull();
+});
+
 test("fixture copies are independent", () => {
   const a = fixture();
   a.opened.client.name = "x";
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

```bash
cargo test -p jyotish-desk --test bhava 2>&1 | grep -E "^error" | sort | uniq -c | head
pnpm --dir app exec vitest run ui/api.test.ts ui/test/fixture.test.ts 2>&1 | grep -E "Test Files|Tests |FAIL" | head
```
Expected: Rust derleme hatası (`error[E0432]: unresolved import jyotish_desk_lib::bhava_commands`); Vitest iki dosyada da düşer (`placidusOpened` dışa aktarılmıyor; `computeBhavaTable` sarmalayıcısı yok).

- [ ] **Step 3: Komutlar ve kayıt**

**Yeni dosya:** `app/src-tauri/src/bhava_commands.rs`

````rust
//! Bhava tablosu ve Sudarshana chakra komutları (yol haritası Plan 25). Kalıp
//! `module_commands` ile aynıdır: tek `ChartInput`, hesap motor iş parçacığında, komut gövdesi
//! `blocking` havuzunda, hata `CommandError`. Sonuçlar `Chart`'a eklenmez ve saklanmaz; panel
//! açıkken istenir (K1). Girdi doğum verisidir: loglanmaz.

use std::sync::Arc;

use ephemeris::{Engine, SwissEph};
use jyotish_core::{BhavaTable, ChartInput, SudarshanaChakra};
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::CommandError;

/// `compute_bhava_table` komutunun gövdesi; bloklar.
pub fn compute_bhava_table_blocking(
    engine: &Engine,
    input: ChartInput,
) -> Result<Option<BhavaTable>, CommandError> {
    let table =
        engine.call(move |se: &mut SwissEph| jyotish_core::compute_bhava_table(se, &input))??;
    Ok(table)
}

/// Bhava tablosu: haritanın ev sisteminde on iki bhavanın başlangıcı, ortası ve bitişi (burç,
/// derece, nakshatra, pada, burç lordu, nakshatra lordu) ve dokuz grahanın bhavası ile bhava
/// başlangıcına ve ortasına uzaklığı. Doğum saati bilinmiyorsa (`time_known == false`) `null`
/// döner. Placidus'ta bhava ortası yoktur; sayı prashnasında evler tam burçtur.
#[tauri::command]
#[specta::specta]
pub async fn compute_bhava_table(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
) -> Result<Option<BhavaTable>, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_bhava_table_blocking(&engine, input)).await
}

/// `compute_sudarshana` komutunun gövdesi; bloklar.
pub fn compute_sudarshana_blocking(
    engine: &Engine,
    input: ChartInput,
) -> Result<SudarshanaChakra, CommandError> {
    let chakra =
        engine.call(move |se: &mut SwissEph| jyotish_core::compute_sudarshana(se, &input))??;
    Ok(chakra)
}

/// Sudarshana chakra: lagna, Ay ve Güneş'i birinci ev sayan üç halka (içten dışa), her halkada
/// on iki ev ve içindeki grahalar. Doğum saati bilinmiyorsa lagna halkasının evleri `null` döner
/// ve sonuç `approximate` işaretlidir.
#[tauri::command]
#[specta::specta]
pub async fn compute_sudarshana(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
) -> Result<SudarshanaChakra, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_sudarshana_blocking(&engine, input)).await
}
````

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index a18fd91..e166e5e 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -1,4 +1,5 @@
 pub mod atlas_commands;
+pub mod bhava_commands;
 pub mod client_commands;
 pub mod commands;
 pub mod data_dir;
@@ -30,6 +31,8 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             commands::compute_chart,
             commands::compute_transits,
             transit_commands::transits_at,
+            bhava_commands::compute_bhava_table,
+            bhava_commands::compute_sudarshana,
             commands::dasha_children,
             commands::print_window,
             module_commands::compute_drishti,
````

```bash
cargo fmt --all --check
cargo test -p jyotish-desk --test bhava 2>&1 | grep -E "^test result|FAILED|panicked"
```
Expected: fmt çıktısız; `1 passed`. Sripati denetimi düşerse (`sandhi n … vs …`) `porphyry_points` Task 1'deki el örneğiyle yeniden karşılaştırılır; tolerans gevşetilmez.

- [ ] **Step 4: Fikstür: `bhava`**

Blok kendi `use` satırlarını içeride yapar; dosyanın başındaki `use` satırları değişmez ("Paylaşılan dosyalar"). `assert_eq!(…, 3, …)` D16'nın ölçümünü sabitler: Delhi fikstüründe Placidus üç grahayı yer değiştirir.

````diff
diff --git a/app/src-tauri/tests/ui_fixture.rs b/app/src-tauri/tests/ui_fixture.rs
index 9e77aa0..3886486 100644
--- a/app/src-tauri/tests/ui_fixture.rs
+++ b/app/src-tauri/tests/ui_fixture.rs
@@ -466,8 +466,71 @@ fn ui_fixture_is_up_to_date() {
             "the fixture range has a {kind:?}"
         );
     }
+    // Bhava tablosu ve Sudarshana chakra (Plan 25). Açık haritanın girdisiyle (tam burç) tablo ve
+    // chakra; aynı an ve yer için Sripati tablosu (bhava ortalarıyla) ile Placidus haritası ve
+    // tablosu (chalit görünümünün verisi: üç graha burç evinden başka bir bhavadadır); kutup
+    // dairesinde Placidus (Porphyry'ye düşer; tests/golden/tromso-2050-12-21.json anı); doğum
+    // saati bilinmeyen girdiyle chakra (lagna halkası yok).
+    let bhava = {
+        use jyotish_core::{CalcSettings, HouseSystem};
+        use jyotish_desk_lib::bhava_commands::{
+            compute_bhava_table_blocking, compute_sudarshana_blocking,
+        };
+        use jyotish_desk_lib::commands::compute_chart_for_blocking;
+        let with_system = |system: HouseSystem| ChartInput {
+            settings: CalcSettings {
+                house_system: system,
+                ..module_input.settings.clone()
+            },
+            ..module_input.clone()
+        };
+        let table_of = |input: ChartInput| {
+            compute_bhava_table_blocking(&engine, input)
+                .expect("bhava table")
+                .expect("time known")
+        };
+        let table = table_of(module_input.clone());
+        let table_sripati = table_of(with_system(HouseSystem::Sripati));
+        let placidus = with_system(HouseSystem::Placidus);
+        let chart_placidus =
+            compute_chart_for_blocking(&engine, placidus.clone()).expect("placidus chart");
+        let table_placidus = table_of(placidus.clone());
+        assert_eq!(
+            table_placidus.grahas.iter().filter(|g| g.shifted).count(),
+            3,
+            "three grahas of the fixture chart are outside their sign house"
+        );
+        let table_placidus_polar = table_of(ChartInput {
+            jd_ut: 2_470_162.260_416_666_5,
+            lat: 69.6492,
+            lon: 18.9553,
+            ..placidus
+        });
+        assert!(table_placidus_polar.porphyry_fallback);
+        let sudarshana =
+            compute_sudarshana_blocking(&engine, module_input.clone()).expect("sudarshana");
+        let sudarshana_time_unknown = compute_sudarshana_blocking(
+            &engine,
+            ChartInput {
+                time_known: false,
+                ..module_input.clone()
+            },
+        )
+        .expect("sudarshana without a birth time");
+        json!({
+            "chart_placidus": chart_placidus,
+            "table": table,
+            "table_sripati": table_sripati,
+            "table_placidus": table_placidus,
+            "table_placidus_polar": table_placidus_polar,
+            "sudarshana": sudarshana,
+            "sudarshana_time_unknown": sudarshana_time_unknown,
+            "sudarshana_refs": jyotish_core::SudarshanaRef::ALL,
+        })
+    };
     let value = json!({
         "opened": opened,
+        "bhava": bhava,
         "dasha_children": { "path": path, "periods": children },
         "places": places,
         "preview": preview,
````

- [ ] **Step 5: Bağlamaları ve fikstürü yeniden üret (C10)**

```bash
cargo fmt --all --check
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --test bindings --test ui_fixture 2>&1 | grep -E "^test result"
git diff --stat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
grep -c "compute_bhava_table\|compute_sudarshana" app/ui/bindings.ts
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
node -e 'const f=require("./app/ui/test/fixtures/delhi.json").bhava; console.log(f.table.system, f.table_sripati.system, f.table_placidus.grahas.filter(g=>g.shifted).length, f.table_placidus_polar.porphyry_fallback, f.chart_placidus.bhavas.system, f.sudarshana.rings.length, f.sudarshana_time_unknown.rings[0].houses)'
```
Expected: fmt çıktısız; dört `ok` satırı (`1 passed`, `2 passed`, sonra `1 passed` ve `2 passed`); `bindings.ts` 99 satır kazanır, `delhi.json` tek satırdır ve değişir; komut adları sayacı `2`; yasaklı ad sayacı `0`; son komut `WholeSign Sripati 3 true Placidus 3 null` yazar. İki dosya elle düzenlenmez. Fikstür üretimi `three grahas of the fixture chart are outside their sign house` iletisiyle düşerse D16'nın ölçümü bu ağaçta tutmuyordur: durulur ve sahibine bildirilir (beklenen sayı çıktıya bakarak değiştirilmez).

- [ ] **Step 6: Sarmalayıcılar, fikstür tipi ve e2e taklidi**

````diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index 0b96d84..c235a61 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -13,11 +13,17 @@ export interface IpcOptions {
   unknownTime?: boolean;
   /** Uyum: eş haritanın (Ayşe) doğum saati bilinmiyor; `chart_input` `time_known: false` döner. */
   partnerUnknownTime?: boolean;
+  /** Açık harita Placidus ayarlıdır (`bhava.chart_placidus`): chalit görünümünde üç graha yer değiştirir. */
+  placidus?: boolean;
 }
 
-export async function mockIpc(page: Page, { session, unknownTime = false, partnerUnknownTime = false }: IpcOptions) {
+export async function mockIpc(page: Page, { session, unknownTime = false, partnerUnknownTime = false, placidus = false }: IpcOptions) {
   await page.addInitScript(
-    ({ fixture, session, unknownTime, partnerUnknownTime }) => {
+    ({ fixture, session, unknownTime, partnerUnknownTime, placidus }) => {
+      if (placidus) {
+        fixture.opened.chart = fixture.bhava.chart_placidus;
+        fixture.opened.calc_settings = fixture.bhava.chart_placidus.meta.settings;
+      }
       if (unknownTime) {
         fixture.opened.chart_record.birth.time = null;
         fixture.opened.chart_record.birth.time_certainty = "Unknown";
@@ -111,6 +117,16 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
         transits_at: () => fixture.transit,
         // Bakış tablosu yeniden hesaplanmaz: fikstürdeki gerçek sonuç döner.
         compute_drishti: () => fixture.drishti,
+        // Bhava tablosu ve Sudarshana chakra (Plan 25): fikstürdeki gerçek sonuçlar. Tablo
+        // girdinin ev sistemine göre seçilir; saat bilinmiyorsa Rust tabloyu `null`, chakrayı
+        // lagna halkası olmadan döner.
+        compute_bhava_table: (args) => {
+          const input = args.input as { time_known: boolean; settings: { house_system: string } };
+          if (!input.time_known) return null;
+          if (input.settings.house_system === "Sripati") return fixture.bhava.table_sripati;
+          return input.settings.house_system === "Placidus" ? fixture.bhava.table_placidus : fixture.bhava.table;
+        },
+        compute_sudarshana: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.bhava.sudarshana : fixture.bhava.sudarshana_time_unknown),
         // Jaimini raporu: fikstürdeki gerçek sonuç; saat bilinmiyorsa Rust padaları boş döner.
         compute_jaimini: (args) =>
           (args.input as { time_known: boolean }).time_known ? fixture.jaimini : { ...fixture.jaimini, arudhas: null },
@@ -249,7 +265,7 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
         },
       });
     },
-    { fixture: FIXTURE, session, unknownTime, partnerUnknownTime },
+    { fixture: FIXTURE, session, unknownTime, partnerUnknownTime, placidus },
   );
 }
 
diff --git a/app/ui/api.ts b/app/ui/api.ts
index b3b76a1..cac8507 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -92,6 +92,10 @@ export const api = {
   // İleri modüller: sonuç saklanmaz, panel açıkken istenir (ileri modüller spec'i K1). Girdi
   // `modules/input.ts`'teki `chartInput(opened)` ile kurulur.
   computeDrishti: (input: ChartInput) => wrap(() => commands.computeDrishti(input)),
+  // Bhava tablosu ve Sudarshana chakra (Plan 25). Tablo: `null` doğum saati bilinmeyen haritadır.
+  // Chakra saat bilinmese de döner; lagna halkasının evleri `null`, sonuç `approximate` olur.
+  computeBhavaTable: (input: ChartInput) => wrap(() => commands.computeBhavaTable(input)),
+  computeSudarshana: (input: ChartInput) => wrap(() => commands.computeSudarshana(input)),
   computeJaimini: (input: ChartInput) => wrap(() => commands.computeJaimini(input)),
   // Chara dasha: `null` doğum saati bilinmeyen haritadır. `path` dönemin `signs`'ı, `cycle`
   // çevrimidir; `charaDashaAt` transit anının dönem yolunu verir (arayüz tarih karşılaştırmaz).
diff --git a/app/ui/test/fixture.ts b/app/ui/test/fixture.ts
index 62b0474..37303fc 100644
--- a/app/ui/test/fixture.ts
+++ b/app/ui/test/fixture.ts
@@ -5,7 +5,9 @@
 // gün panchangı (iki gün), üç dilimlik bir muhurta araması ve iki prashna haritası (olgularıyla).
 // Arayüz hesap yapmadığı için testler bunu çizer.
 import type {
+  BhavaTable,
   CharaDasha,
+  Chart,
   ChartBrief,
   ChartInput,
   ChartQuery,
@@ -42,6 +44,8 @@ import type {
   SignDashaPath,
   SignDashaPeriod,
   SkyEventKind,
+  SudarshanaChakra,
+  SudarshanaRef,
   TithiGroup,
   TransitView,
   Varga,
@@ -52,6 +56,27 @@ import raw from "./fixtures/delhi.json?raw";
 
 export interface UiFixture {
   opened: OpenedClient;
+  /**
+   * Bhava tablosu ve Sudarshana chakra (Plan 25), açık haritanın anı ve yeri için. `table` açık
+   * haritanın girdisiyle (tam burç), `table_sripati` Sripati ayarıyla (bhava ortalarıyla; hiçbir
+   * graha yer değiştirmez), `table_placidus` Placidus ayarıyla (orta yok; Merkür, Jüpiter ve
+   * Venüs burç evinden bir önceki bhavadadır) `compute_bhava_table` dönüşüdür;
+   * `chart_placidus` aynı Placidus ayarının haritasıdır (chalit görünümünün verisi).
+   * `table_placidus_polar` kutup dairesinde (Tromsø, 2050-12-21) Placidus'un Porphyry'ye düştüğü
+   * tablodur. `sudarshana` açık haritanın girdisiyle, `sudarshana_time_unknown`
+   * `time_known: false` ile `compute_sudarshana` dönüşüdür; `sudarshana_refs` Rust
+   * `SudarshanaRef::ALL` (içten dışa halka sırası).
+   */
+  bhava: {
+    chart_placidus: Chart;
+    table: BhavaTable;
+    table_sripati: BhavaTable;
+    table_placidus: BhavaTable;
+    table_placidus_polar: BhavaTable;
+    sudarshana: SudarshanaChakra;
+    sudarshana_time_unknown: SudarshanaChakra;
+    sudarshana_refs: SudarshanaRef[];
+  };
   dasha_children: { path: Graha[]; periods: DashaPeriod[] };
   places: PlaceHit[];
   preview: MomentPreview;
@@ -169,6 +194,17 @@ export function fixture(): UiFixture {
   return structuredClone(FIXTURE);
 }
 
+/**
+ * Placidus ayarlı açık harita: aynı danışan ve doğum, harita `bhava.chart_placidus` (üç graha burç
+ * evinden başka bir bhavada). `save_calc_settings` dönüşünün arayüzdeki karşılığıdır.
+ */
+export function placidusOpened(): OpenedClient {
+  const f = fixture();
+  f.opened.chart = f.bhava.chart_placidus;
+  f.opened.calc_settings = f.bhava.chart_placidus.meta.settings;
+  return f.opened;
+}
+
 /** Saati bilinmeyen doğum: aynı harita, `time_certainty: "Unknown"` ve öğle uyarısı. */
 export function unknownTimeOpened(): OpenedClient {
   const o = fixture().opened;
````

- [ ] **Step 7: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
cargo test -p jyotish-desk 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
grep -nE 'println!|eprintln!|dbg!|log::|tracing::' app/src-tauri/src/bhava_commands.rs
```
Expected: fmt ve clippy çıktısız; sayaç `0`; `jyotish-desk` lib `58 passed`, `bhava` `1 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `modules` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; Vitest `88 passed (88)`, `888 passed (888)`; `✓ built` (`pnpm build` e2e dizinini de tipler: `ipc.ts` derlenir); log taraması çıktısız.

- [ ] **Step 8: Commit**

```bash
git add app/src-tauri/src/bhava_commands.rs app/src-tauri/src/lib.rs app/src-tauri/tests/bhava.rs app/src-tauri/tests/ui_fixture.rs app/ui/bindings.ts app/ui/test/fixtures/delhi.json app/ui/test/fixture.ts app/ui/test/fixture.test.ts app/ui/api.ts app/ui/api.test.ts app/e2e/ipc.ts
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(app): compute_bhava_table and compute_sudarshana commands, checked against the real engine; bindings and fixture" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: on bir dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 4: Arayüz: Bhava tablosu paneli

Yol haritası §4.5; UI spec §3.3, §3.4, §5, §8; K12; D11. Panel Bhava bala panelinin kalıbını kullanır: araç çubuğu her durumda durur; tablo kaydırma kabındadır (`BalaScroll`); graha ve bhava adları ortak vurgunun düğmeleridir (`PinButton`).

**Files:**
- Create: `app/ui/modules/useBhavaTable.ts`, `app/ui/panels/BhavaTablePanel.tsx`, `app/ui/panels/BhavaTablePanel.test.tsx`
- Modify: `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: yeni dosya (11); `layout/model.test.ts` ve `panels/panels.test.tsx` (var olan testlere beklenti)

**Interfaces:**
- Consumes: Task 3'ün `api.computeBhavaTable`, bağlama tipleri (`BhavaTable`, `BhavaPoint`), fikstür (`bhava.*`, `placidusOpened`); `modules/input` (`chartInput`, `inputKey`), `modules/useModule`; `panels/bala.BalaScroll`, `panels/format` (`formatDegree`, `formatSignedDegree`), `panels/labels.nakshatraName`, `panels/PinButton`, `panels/states`; `chart/adapter.signIndex`; `highlight/store.useHighlight`; `view/store.useLayoutStore`.
- Produces:
  - `modules/useBhavaTable.ts`: `useBhavaTable(opened: OpenedClient | null): ModuleState<BhavaTable | null>`.
  - `panels/BhavaTablePanel.tsx`: `BhavaTablePanel({ panel })`, `BHAVA_POINTS`, `BhavaPointKind`, `bhavaPointKind(settings)`, `SHIFT_MARK` (Task 5'te `chart/types.ts`'e taşınır).
  - `layout/model.ts`: panel türü `"bhava_table"`; `MODULE_PANELS` girdisi `{ kind: "bhava_table", w: 7, h: 20 }`.
  - i18n: `panel.bhava_table`, `bhavaTable.*` (üç dil).

- [ ] **Step 1: Testler: panel ve katalog**

Beklenen hücreler fikstürdeki Rust değerlerinin biçimlenmiş hâlidir (derece ve yay dakikası aşağı yuvarlanır). Delhi, İkizler lagna 16°20′: tam burçta 1. bhava "İki 0°00′"; Sripati'de 1. bhava "Boğ 29°12′"de başlar ve ortası lagnadır; Placidus'ta 1. bhava lagnada başlar.

````diff
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index 08fa4b3..f945a41 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -47,6 +47,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "argala",
     "shadbala",
     "bhava_bala",
+    "bhava_table",
     "yogas",
     "compat",
     "muhurta_day",
@@ -60,7 +61,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "bhava_table", "yogas", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index 36744d1..f96b547 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -12,6 +12,7 @@ import { ArgalaPanel } from "./ArgalaPanel";
 import { ArudhaPanel } from "./ArudhaPanel";
 import { AshtakavargaPanel } from "./AshtakavargaPanel";
 import { BhavaBalaPanel } from "./BhavaBalaPanel";
+import { BhavaTablePanel } from "./BhavaTablePanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel } from "./ChartPanel";
 import { CompatPanel } from "./CompatPanel";
@@ -55,6 +56,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.argala.component).toBe(ArgalaPanel);
   expect(PANEL_REGISTRY.shadbala.component).toBe(ShadbalaPanel);
   expect(PANEL_REGISTRY.bhava_bala.component).toBe(BhavaBalaPanel);
+  expect(PANEL_REGISTRY.bhava_table.component).toBe(BhavaTablePanel);
   expect(PANEL_REGISTRY.yogas.component).toBe(YogasPanel);
   expect(PANEL_REGISTRY.compat.component).toBe(CompatPanel);
   expect(PANEL_REGISTRY.muhurta_day.component).toBe(MuhurtaDayPanel);
````

**Yeni dosya:** `app/ui/panels/BhavaTablePanel.test.tsx`

````tsx
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeBhavaTable: vi.fn() } }));

import { api, type Result } from "../api";
import type { BhavaTable } from "../bindings";
import { setLanguage } from "../i18n";
import type { Panel } from "../layout/model";
import { fixture, placidusOpened, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { bhavaPointKind, BhavaTablePanel } from "./BhavaTablePanel";

const ok = (table: BhavaTable | null = fixture().bhava.table): Result<BhavaTable | null> => ({ status: "ok", data: table });
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "bt", kind: "bhava_table", x: 0, y: 0, w: 7, h: 20, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};

/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  return <BhavaTablePanel panel={active.panels[0]} />;
}

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened });
}

const bhavas = (point = "başlangıç") => screen.findByRole("table", { name: `Bhava tablosu: on iki bhavanın ${point} noktası, nakshatrası, lordları ve yayı` });
const grahas = () => screen.findByRole("table", { name: "Grahaların bhavası: burç evi, bhava, bhava başlangıcına ve ortasına uzaklık" });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const body = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);

beforeEach(() => {
  vi.mocked(api.computeBhavaTable).mockReset();
  vi.mocked(api.computeBhavaTable).mockImplementation(async () => ok());
});

test("whole sign: twelve bhavas start at the sign and no graha leaves its sign house", async () => {
  renderPanel();
  const table = await bhavas();
  expect(api.computeBhavaTable).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.computeBhavaTable).mock.calls[0][0].time_known).toBe(true);
  const rows = within(table).getAllByRole("row");
  expect(cells(rows[0])).toEqual(["Bhava", "Başlangıç", "Nakshatra", "Pada", "Burç lordu", "Nakshatra lordu", "Yay"]);
  expect(rows).toHaveLength(13);
  // Delhi fikstürü, İkizler lagna 16°20′: 1. bhava İkizler'in başında başlar.
  expect(body(table).slice(0, 3)).toEqual([
    ["1", "İki 0°00′", "Mrigashira", "3", "Merkür", "Mars", "30°00′"],
    ["2", "Yen 0°00′", "Punarvasu", "4", "Ay", "Jüpiter", "30°00′"],
    ["3", "Asl 0°00′", "Magha", "1", "Güneş", "Ketu", "30°00′"],
  ]);
  // Bhava numarası tam adıyla okunur.
  expect(within(rows[1]).getByRole("button", { name: "1. bhava" })).toHaveTextContent("1");
  const g = await grahas();
  expect(cells(within(g).getAllByRole("row")[0])).toEqual(["Graha", "Burç evi", "Bhava", "Başlangıçtan", "Ortadan"]);
  expect(body(g)).toEqual([
    ["Güneş", "7", "7", "16°30′", "+0°10′"],
    ["Ay", "5", "5", "19°28′", "+3°07′"],
    ["Mars", "9", "9", "4°06′", "−12°13′"],
    ["Merkür", "7", "7", "8°02′", "−8°18′"],
    ["Jüpiter", "11", "11", "1°23′", "−14°56′"],
    ["Venüs", "6", "6", "7°42′", "−8°37′"],
    ["Satürn", "11", "11", "16°32′", "+0°12′"],
    ["Rahu", "2", "2", "11°11′", "−5°09′"],
    ["Ketu", "8", "8", "11°11′", "−5°09′"],
  ]);
  expect(screen.getByText(/^Ev sistemi tam burç: bhava burcun kendisidir ve chalit görünümü rasi ile aynıdır\./)).toBeInTheDocument();
  expect(screen.queryByText(/Graha, burcunun evinden başka bir bhavadadır/)).toBeNull();
  expect(screen.getByText("Yöntem: bhava_table.v1")).toBeInTheDocument();
});

test("both tables sit in keyboard-focusable scroll groups named by their captions", async () => {
  renderPanel();
  for (const table of [await bhavas(), await grahas()]) {
    const group = table.closest(".bala-scroll") as HTMLElement;
    expect(group).toHaveAttribute("role", "group");
    expect(group).toHaveAttribute("tabindex", "0");
    expect(group).toHaveAccessibleName(table.querySelector("caption")!.textContent!);
  }
});

test("Sripati: the point setting switches between the sandhi, the Porphyry middle and the end, and is saved with the layout", async () => {
  vi.mocked(api.computeBhavaTable).mockImplementation(async () => ok(fixture().bhava.table_sripati));
  const { persistence } = renderPanel();
  expect(body(await bhavas()).slice(0, 2)).toEqual([
    ["1", "Boğ 29°12′", "Mrigashira", "2", "Venüs", "Mars", "30°00′"],
    ["2", "İki 29°12′", "Punarvasu", "3", "Merkür", "Jüpiter", "25°45′"],
  ]);
  fireEvent.change(screen.getByRole("combobox", { name: "Nokta" }), { target: { value: "mid" } });
  // 1. bhavanın ortası lagnadır (İkizler 16°20′).
  expect(body(await bhavas("orta")).slice(0, 2)).toEqual([
    ["1", "İki 16°20′", "Ardra", "3", "Merkür", "Rahu", "30°00′"],
    ["2", "Yen 12°05′", "Pushya", "3", "Ay", "Satürn", "25°45′"],
  ]);
  expect(within(await bhavas("orta")).getAllByRole("row")[0]).toHaveTextContent("Orta");
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ point: "mid" }));
  fireEvent.change(screen.getByRole("combobox", { name: "Nokta" }), { target: { value: "end" } });
  // Bitiş, sonraki bhavanın başlangıcıdır.
  expect(body(await bhavas("bitiş"))[0]).toEqual(["1", "İki 29°12′", "Punarvasu", "3", "Merkür", "Jüpiter", "30°00′"]);
  expect(body(await grahas())[1]).toEqual(["Ay", "5", "5", "28°44′", "+11°37′"]);
  expect(screen.getByText(/^Ev sistemi Sripati: başlangıç sandhidir; orta Porphyry noktasıdır/)).toBeInTheDocument();
  expect(screen.queryByText(/Placidus/)).toBeNull();
  // Ayar değişimi yeni bir istek değildir.
  expect(api.computeBhavaTable).toHaveBeenCalledTimes(1);
});

test("Placidus: shifted grahas carry a mark read as words, and there is no middle", async () => {
  vi.mocked(api.computeBhavaTable).mockImplementation(async () => ok(fixture().bhava.table_placidus));
  renderPanel({ point: "mid" }, placidusOpened());
  expect(vi.mocked(api.computeBhavaTable).mock.calls[0][0].settings.house_system).toBe("Placidus");
  expect(body(await bhavas("orta"))[0]).toEqual(["1", "—", "—", "—", "—", "—", "22°37′"]);
  expect(screen.getByText("Placidus'ta bhava ortası tanımlanmaz: nokta bhavanın başlangıcıdır.")).toHaveAttribute("role", "note");
  const rows = body(await grahas());
  expect(rows[0]).toEqual(["Güneş", "7", "7", "0°10′", "—"]);
  expect(rows.slice(3, 6)).toEqual([
    ["Merkür", "7", "6* burç evinden farklı", "24°07′", "—"],
    ["Jüpiter", "11", "10* burç evinden farklı", "27°47′", "—"],
    ["Venüs", "6", "5* burç evinden farklı", "29°27′", "—"],
  ]);
  const mercury = within(await grahas()).getAllByRole("row")[4];
  expect(mercury.querySelector('[aria-hidden="true"]')).toHaveTextContent("*");
  expect(mercury.querySelector(".visually-hidden")).toHaveTextContent("burç evinden farklı");
  expect(screen.getByText("* Graha, burcunun evinden başka bir bhavadadır.")).toBeInTheDocument();
  expect(screen.getByText("Ev sistemi Placidus: nokta bhavanın başlangıcıdır; bhava ortası tanımlanmaz.")).toBeInTheDocument();
  fireEvent.change(screen.getByRole("combobox", { name: "Nokta" }), { target: { value: "start" } });
  expect(body(await bhavas())[0]).toEqual(["1", "İki 16°20′", "Ardra", "3", "Merkür", "Rahu", "22°37′"]);
  expect(screen.queryByText(/^Placidus'ta bhava ortası tanımlanmaz/)).toBeNull();
});

test("Placidus inside the polar circle: the Porphyry fallback is said in words", async () => {
  vi.mocked(api.computeBhavaTable).mockImplementation(async () => ok(fixture().bhava.table_placidus_polar));
  renderPanel();
  // İki bhava aynı burçta başlar (Aslan 2°46′ ve Aslan 22°00′).
  expect(body(await bhavas()).slice(0, 2).map((r) => r[1])).toEqual(["Asl 2°46′", "Asl 22°00′"]);
  expect(screen.getByText("Placidus bu enlemde hesaplanamadı; başlangıç noktaları Porphyry noktalarıdır.")).toHaveAttribute("role", "note");
});

test("pinning a graha or a bhava highlights its rows", async () => {
  vi.mocked(api.computeBhavaTable).mockImplementation(async () => ok(fixture().bhava.table_placidus));
  renderPanel({}, placidusOpened());
  const g = await grahas();
  const row = (name: string) => within(g).getByRole("button", { name }).closest("tr")!;
  fireEvent.click(within(g).getByRole("button", { name: "Merkür" }));
  expect(row("Merkür")).toHaveClass("is-highlighted");
  expect(row("Güneş")).not.toHaveClass("is-highlighted");
  // 7. bhava: bhava satırı ve o bhavadaki graha (Güneş; Merkür Placidus'ta 6. bhavadadır).
  fireEvent.click(within(await bhavas()).getByRole("button", { name: "7. bhava" }));
  expect(within(await bhavas()).getByRole("button", { name: "7. bhava" }).closest("tr")).toHaveClass("is-highlighted");
  expect(row("Güneş")).toHaveClass("is-highlighted");
  expect(row("Merkür")).not.toHaveClass("is-highlighted");
});

test("unknown birth time: Rust returns no table and the panel says why; the toolbar stays", async () => {
  vi.mocked(api.computeBhavaTable).mockResolvedValue(ok(null));
  renderPanel({}, unknownTimeOpened());
  expect(await screen.findByText("Doğum saati bilinmiyor: bhavalar lagnaya bağlı olduğu için gösterilmiyor.")).toBeInTheDocument();
  expect(vi.mocked(api.computeBhavaTable).mock.calls[0][0].time_known).toBe(false);
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.getByRole("combobox", { name: "Nokta" })).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeBhavaTable).not.toHaveBeenCalled();
});

test("an error is shown in the panel; retry asks again and keeps the focus in the panel", async () => {
  vi.mocked(api.computeBhavaTable).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Bhava tablosu">
      <LivePanel />
    </section>,
    { layout: LAYOUT({}), opened: fixture().opened },
  );
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  const retry = screen.getByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  await bhavas();
  expect(api.computeBhavaTable).toHaveBeenCalledTimes(2);
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Bhava tablosu" }));
});

test("the point setting falls back to the start for unknown values", () => {
  expect(bhavaPointKind({})).toBe("start");
  expect(bhavaPointKind({ point: "mid" })).toBe("mid");
  expect(bhavaPointKind({ point: "end" })).toBe("end");
  expect(bhavaPointKind({ point: "cusp" })).toBe("start");
  expect(bhavaPointKind({ point: 2 })).toBe("start");
});

test("the panel is named in three languages", async () => {
  try {
    await setLanguage("en");
    renderPanel();
    expect(await screen.findByRole("table", { name: "Bhava table: start point, nakshatra, lords and arc of the twelve bhavas" })).toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: "Point" })).toBeInTheDocument();
    await setLanguage("fr");
    expect(await screen.findByRole("table", { name: "Table des bhavas : début, nakshatra, maîtres et arc des douze bhavas" })).toBeInTheDocument();
    expect(screen.getByText("Méthode : bhava_table.v1")).toBeInTheDocument();
  } finally {
    await setLanguage("tr");
  }
});
````

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/panels/BhavaTablePanel.test.tsx ui/panels/panels.test.tsx ui/layout/model.test.ts 2>&1 | grep -E "Test Files|Tests |FAIL|Error" | head`
Expected: üç dosya da düşer (`./BhavaTablePanel` çözülemez; katalogda `bhava_table` yok).

- [ ] **Step 3: Kanca, panel, katalog ve metinler**

**Yeni dosya:** `app/ui/modules/useBhavaTable.ts`

````ts
// Bhava tablosunu (`compute_bhava_table`) isteyen kanca. Tablo saklanmaz ve arayüzde hesap yoktur
// (K1, C4). `data === null` (yükleme bittiğinde): doğum saati bilinmiyor (Rust `None` döner, K2).
import { api } from "../api";
import type { BhavaTable, OpenedClient } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

export function useBhavaTable(opened: OpenedClient | null): ModuleState<BhavaTable | null> {
  return useModule(opened !== null ? inputKey(opened) : null, () => api.computeBhavaTable(chartInput(opened as OpenedClient)));
}
````

**Yeni dosya:** `app/ui/panels/BhavaTablePanel.tsx`

````tsx
// Bhava tablosu paneli (yol haritası §4.5): haritanın ev sisteminde on iki bhavanın seçilen
// noktası (başlangıç, orta ya da bitiş: burç, derece, nakshatra, pada, burç lordu, nakshatra
// lordu) ve dokuz grahanın bhavası ile bhava başlangıcına ve ortasına uzaklığı. Değerler
// `compute_bhava_table`'dan gelir; burada hesap yoktur (C4). Burç evinden başka bir bhavadaki
// graha `*` ve dipnotla gösterilir; renk anlam taşımaz (C9). Doğum saati bilinmiyorsa tablo
// Rust'tan boş gelir (K2) ve panel bunu bir satırla söyler.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { BhavaPoint, BhavaTable } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import type { Panel } from "../layout/model";
import { useBhavaTable } from "../modules/useBhavaTable";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import { BalaScroll } from "./bala";
import { formatDegree, formatSignedDegree } from "./format";
import { nakshatraName } from "./labels";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export const BHAVA_POINTS = ["start", "mid", "end"] as const;
export type BhavaPointKind = (typeof BHAVA_POINTS)[number];
/** Burç evinden başka bir bhavadaki grahanın işareti. */
export const SHIFT_MARK = "*";
const DASH = "—";

/** Derece ve yay dakikası; Rust'ta sonlu olmayan değer (`null`) çizgidir. */
function deg(v: number | null): string {
  return v === null ? DASH : formatDegree(v);
}

/** Panelin nokta ayarı; tanınmayan değer ve eksik anahtar başlangıçtır. */
export function bhavaPointKind(settings: Record<string, unknown>): BhavaPointKind {
  return settings.point === "mid" || settings.point === "end" ? settings.point : "start";
}

export function BhavaTablePanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const module = useBhavaTable(opened);
  const phase = usePanelLoadPhase(module.loading);
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return (
    <div className="module-panel">
      {/* Araç çubuğu yükleme, hata ve "tablo yok" durumlarında da kullanılabilir. */}
      <div className="module-toolbar">
        <PointSelect panel={panel} />
      </div>
      {module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : module.loading ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : module.data === null ? (
        <p className="panel-notice">{t("bhavaTable.timeUnknown")}</p>
      ) : (
        <Tables table={module.data} point={bhavaPointKind(panel.settings)} />
      )}
    </div>
  );
}

/** Nokta seçicisi; değer panelin `point` ayarıdır ve düzenle birlikte saklanır. */
function PointSelect({ panel }: { panel: Panel }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  return (
    <>
      <label htmlFor={id}>{t("bhavaTable.point")}</label>
      <select id={id} value={bhavaPointKind(panel.settings)} onChange={(e) => setPanelSettings(panel.id, { point: e.target.value })}>
        {BHAVA_POINTS.map((p) => (
          <option key={p} value={p}>
            {t(`bhavaTable.points.${p}`)}
          </option>
        ))}
      </select>
    </>
  );
}

function Tables({ table, point }: { table: BhavaTable; point: BhavaPointKind }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const bhavaCaption = useId();
  const grahaCaption = useId();
  const pointName = t(`bhavaTable.points.${point}`);
  const noMid = table.rows.every((r) => r.mid === null);
  const anyShifted = table.grahas.some((g) => g.shifted);
  const activeHouse = active?.type === "house" ? active.house : null;
  return (
    <>
      <BalaScroll labelledBy={bhavaCaption}>
        <table className="module-table">
          <caption id={bhavaCaption} className="visually-hidden">
            {t("bhavaTable.caption", { point: t(`bhavaTable.pointName.${point}`) })}
          </caption>
          <thead>
            <tr>
              <th scope="col">{t("bhavaTable.bhava")}</th>
              <th scope="col">{pointName}</th>
              <th scope="col">{t("table.nakshatra")}</th>
              <th scope="col" className="num">
                {t("table.pada")}
              </th>
              <th scope="col">{t("bhavaTable.signLord")}</th>
              <th scope="col">{t("bhavaTable.nakshatraLord")}</th>
              <th scope="col" className="num">
                {t("bhavaTable.span")}
              </th>
            </tr>
          </thead>
          <tbody>
            {table.rows.map((row) => {
              const p: BhavaPoint | null = row[point];
              return (
                <tr key={row.house} data-house={row.house} className={activeHouse === row.house ? "is-highlighted" : undefined}>
                  <th scope="row">
                    <PinButton target={{ type: "house", house: row.house }} label={t("bhavaTable.houseLabel", { house: row.house })}>
                      {row.house}
                    </PinButton>
                  </th>
                  <td>{p === null ? DASH : `${t(`sign.${signIndex(p.sign)}`)} ${deg(p.deg_in_sign)}`}</td>
                  <td>{p === null ? DASH : nakshatraName(p.nakshatra)}</td>
                  <td className="num">{p === null ? DASH : p.pada}</td>
                  <td>{p === null ? DASH : <PinButton target={{ type: "graha", id: p.sign_lord }}>{t(`grahaName.${p.sign_lord}`)}</PinButton>}</td>
                  <td>{p === null ? DASH : <PinButton target={{ type: "graha", id: p.nakshatra_lord }}>{t(`grahaName.${p.nakshatra_lord}`)}</PinButton>}</td>
                  <td className="num">{deg(row.span_deg)}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </BalaScroll>
      {point === "mid" && noMid && (
        <p className="panel-notice" role="note">
          {t("bhavaTable.noMid")}
        </p>
      )}
      <BalaScroll labelledBy={grahaCaption}>
        <table className="module-table">
          <caption id={grahaCaption} className="visually-hidden">
            {t("bhavaTable.grahaCaption")}
          </caption>
          <thead>
            <tr>
              <th scope="col">{t("table.graha")}</th>
              <th scope="col" className="num">
                {t("bhavaTable.signHouse")}
              </th>
              <th scope="col" className="num">
                {t("bhavaTable.bhava")}
              </th>
              <th scope="col" className="num">
                {t("bhavaTable.fromStart")}
              </th>
              <th scope="col" className="num">
                {t("bhavaTable.fromMid")}
              </th>
            </tr>
          </thead>
          <tbody>
            {table.grahas.map((g) => {
              const on = (active?.type === "graha" && active.id === g.graha) || activeHouse === g.bhava;
              return (
                <tr key={g.graha} data-graha={g.graha} className={on ? "is-highlighted" : undefined}>
                  <th scope="row">
                    <PinButton target={{ type: "graha", id: g.graha }}>{t(`grahaName.${g.graha}`)}</PinButton>
                  </th>
                  <td className="num">{g.sign_house}</td>
                  <td className="num">
                    {g.bhava}
                    {g.shifted && (
                      <>
                        <span aria-hidden="true">{SHIFT_MARK}</span>
                        <span className="visually-hidden"> {t("bhavaTable.shifted")}</span>
                      </>
                    )}
                  </td>
                  <td className="num">{deg(g.from_start_deg)}</td>
                  <td className="num">{g.from_mid_deg === null ? DASH : formatSignedDegree(g.from_mid_deg)}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </BalaScroll>
      {anyShifted && <p className="panel-message">{t("bhavaTable.shiftedNote", { mark: SHIFT_MARK })}</p>}
      {table.porphyry_fallback && (
        <p className="panel-notice" role="note">
          {t("bhavaTable.fallback")}
        </p>
      )}
      <p className="panel-message">{t(`bhavaTable.system.${table.system}`)}</p>
      <p className="panel-message">{t("bhavaTable.method", { method: table.method })}</p>
    </>
  );
}
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 17db154..b3cea43 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -69,6 +69,7 @@ export const en: Messages = {
     jaimini_karakas: "Chara karaka",
     arudha: "Arudha padas",
     argala: "Argala",
+    bhava_table: "Bhava table",
     shadbala: "Shadbala",
     bhava_bala: "Bhava bala",
     yogas: "Yogas",
@@ -778,6 +779,32 @@ export const en: Messages = {
     },
     method: "Method: {{method}}. Moments are in the time zone of the selected place; the moment in a row changes the transit date.",
   },
+  bhavaTable: {
+    caption: "Bhava table: {{point}} point, nakshatra, lords and arc of the twelve bhavas",
+    grahaCaption: "Bhava of each graha: sign house, bhava, distance from the bhava start and middle",
+    point: "Point",
+    points: { start: "Start", mid: "Middle", end: "End" },
+    pointName: { start: "start", mid: "middle", end: "end" },
+    bhava: "Bhava",
+    houseLabel: "Bhava {{house}}",
+    signLord: "Sign lord",
+    nakshatraLord: "Nakshatra lord",
+    span: "Arc",
+    signHouse: "Sign house",
+    fromStart: "From start",
+    fromMid: "From middle",
+    shifted: "differs from the sign house",
+    shiftedNote: "{{mark}} The graha is in a bhava other than the house of its sign.",
+    noMid: "Placidus defines no bhava middle: the point is the start of the bhava.",
+    fallback: "Placidus could not be computed at this latitude; the starting points are Porphyry points.",
+    system: {
+      WholeSign: "House system: whole sign. The bhava is the sign itself and the chalit view equals the rasi chart. The middle is the degree of the lagna in that sign.",
+      Sripati: "House system: Sripati. The start is the sandhi; the middle is the Porphyry point (the middle of bhava 1 is the lagna, of bhava 10 the MC).",
+      Placidus: "House system: Placidus. The point is the start of the bhava; no bhava middle is defined.",
+    },
+    method: "Method: {{method}}",
+    timeUnknown: "Birth time unknown: the bhavas depend on the lagna and are not shown.",
+  },
   arudha: {
     caption: "Arudha padas: the pada of each of the twelve houses, its sign and the lord of the house",
     pada: "Pada",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 9b6621f..fb1664d 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -69,6 +69,7 @@ export const fr: Messages = {
     jaimini_karakas: "Chara karaka",
     arudha: "Arudha padas",
     argala: "Argala",
+    bhava_table: "Table des bhavas",
     shadbala: "Shadbala",
     bhava_bala: "Bhava bala",
     yogas: "Yogas",
@@ -778,6 +779,32 @@ export const fr: Messages = {
     },
     method: "Méthode : {{method}}. Les instants sont dans le fuseau du lieu choisi ; l’instant d’une ligne change la date de transit.",
   },
+  bhavaTable: {
+    caption: "Table des bhavas : {{point}}, nakshatra, maîtres et arc des douze bhavas",
+    grahaCaption: "Bhava de chaque graha : maison du signe, bhava, distance au début et au milieu du bhava",
+    point: "Point",
+    points: { start: "Début", mid: "Milieu", end: "Fin" },
+    pointName: { start: "début", mid: "milieu", end: "fin" },
+    bhava: "Bhava",
+    houseLabel: "Bhava {{house}}",
+    signLord: "Maître du signe",
+    nakshatraLord: "Maître du nakshatra",
+    span: "Arc",
+    signHouse: "Maison du signe",
+    fromStart: "Depuis le début",
+    fromMid: "Depuis le milieu",
+    shifted: "diffère de la maison du signe",
+    shiftedNote: "{{mark}} Le graha est dans un bhava autre que la maison de son signe.",
+    noMid: "Placidus ne définit pas de milieu de bhava : le point est le début du bhava.",
+    fallback: "Placidus n’a pas pu être calculé à cette latitude ; les points de départ sont des points de Porphyre.",
+    system: {
+      WholeSign: "Système de maisons : signe entier. Le bhava est le signe lui-même et la vue chalit est identique à la carte rasi. Le milieu est le degré du lagna dans ce signe.",
+      Sripati: "Système de maisons : Sripati. Le début est le sandhi ; le milieu est le point de Porphyre (le milieu du bhava 1 est le lagna, celui du bhava 10 le MC).",
+      Placidus: "Système de maisons : Placidus. Le point est le début du bhava ; aucun milieu de bhava n’est défini.",
+    },
+    method: "Méthode : {{method}}",
+    timeUnknown: "Heure de naissance inconnue : les bhavas dépendent du lagna et ne sont pas affichés.",
+  },
   arudha: {
     caption: "Arudha padas : le pada de chacune des douze maisons, son signe et le maître de la maison",
     pada: "Pada",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 8b5784d..4fced2a 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -68,6 +68,7 @@ export const tr = {
     jaimini_karakas: "Chara karaka",
     arudha: "Arudha padalar",
     argala: "Argala",
+    bhava_table: "Bhava tablosu",
     shadbala: "Shadbala",
     bhava_bala: "Bhava bala",
     yogas: "Yogalar",
@@ -777,6 +778,32 @@ export const tr = {
     },
     method: "Yöntem: {{method}}. Anlar seçilen yerin saat dilimindedir; satırdaki an transit tarihini değiştirir.",
   },
+  bhavaTable: {
+    caption: "Bhava tablosu: on iki bhavanın {{point}} noktası, nakshatrası, lordları ve yayı",
+    grahaCaption: "Grahaların bhavası: burç evi, bhava, bhava başlangıcına ve ortasına uzaklık",
+    point: "Nokta",
+    points: { start: "Başlangıç", mid: "Orta", end: "Bitiş" },
+    pointName: { start: "başlangıç", mid: "orta", end: "bitiş" },
+    bhava: "Bhava",
+    houseLabel: "{{house}}. bhava",
+    signLord: "Burç lordu",
+    nakshatraLord: "Nakshatra lordu",
+    span: "Yay",
+    signHouse: "Burç evi",
+    fromStart: "Başlangıçtan",
+    fromMid: "Ortadan",
+    shifted: "burç evinden farklı",
+    shiftedNote: "{{mark}} Graha, burcunun evinden başka bir bhavadadır.",
+    noMid: "Placidus'ta bhava ortası tanımlanmaz: nokta bhavanın başlangıcıdır.",
+    fallback: "Placidus bu enlemde hesaplanamadı; başlangıç noktaları Porphyry noktalarıdır.",
+    system: {
+      WholeSign: "Ev sistemi tam burç: bhava burcun kendisidir ve chalit görünümü rasi ile aynıdır. Orta, lagnanın derecesinin o burçtaki karşılığıdır.",
+      Sripati: "Ev sistemi Sripati: başlangıç sandhidir; orta Porphyry noktasıdır (1. bhavanın ortası lagna, 10. bhavanın ortası MC).",
+      Placidus: "Ev sistemi Placidus: nokta bhavanın başlangıcıdır; bhava ortası tanımlanmaz.",
+    },
+    method: "Yöntem: {{method}}",
+    timeUnknown: "Doğum saati bilinmiyor: bhavalar lagnaya bağlı olduğu için gösterilmiyor.",
+  },
   arudha: {
     caption: "Arudha padalar: on iki evin padası, burcu ve evin lordu",
     pada: "Pada",
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index c3629ce..8ddd706 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -18,6 +18,7 @@ export const PANEL_KINDS = [
   "argala",
   "shadbala",
   "bhava_bala",
+  "bhava_table",
   "yogas",
   "compat",
   "muhurta_day",
@@ -42,6 +43,8 @@ export const MODULE_PANELS = [
   // Shadbala ve Bhava bala (Plan 17).
   { kind: "shadbala", w: 8, h: 12 },
   { kind: "bhava_bala", w: 6, h: 14 },
+  // Bhava tablosu (yol haritası Plan 25).
+  { kind: "bhava_table", w: 7, h: 20 },
   // Yogalar (Plan 18).
   { kind: "yogas", w: 7, h: 14 },
   // Uyum (Plan 19).
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index cee04ca..6adb861 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -8,6 +8,7 @@ import { ArgalaPanel } from "./ArgalaPanel";
 import { ArudhaPanel } from "./ArudhaPanel";
 import { AshtakavargaPanel } from "./AshtakavargaPanel";
 import { BhavaBalaPanel } from "./BhavaBalaPanel";
+import { BhavaTablePanel } from "./BhavaTablePanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel, chartVarga } from "./ChartPanel";
 import { CompatPanel } from "./CompatPanel";
@@ -48,6 +49,8 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   // Shadbala ve Bhava bala (Plan 17).
   shadbala: { titleKey: "panel.shadbala", component: ShadbalaPanel },
   bhava_bala: { titleKey: "panel.bhava_bala", component: BhavaBalaPanel },
+  // Bhava tablosu (yol haritası Plan 25).
+  bhava_table: { titleKey: "panel.bhava_table", component: BhavaTablePanel },
   // Yogalar (Plan 18).
   yogas: { titleKey: "panel.yogas", component: YogasPanel },
   // Uyum (Plan 19).
````

i18n eklemelerinin yeri "Paylaşılan dosyalar" bölümündeki gibidir: `panel.bhava_table` `argala` satırından sonra; `bhavaTable` bölümü üst düzey `arudha` bölümünden hemen önce.

- [ ] **Step 4: Testler ve kapılar**

```bash
pnpm --dir app exec vitest run ui/panels/BhavaTablePanel.test.tsx 2>&1 | grep -E "Tests |FAIL"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
git diff -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
```
Expected: `11 passed`; Vitest `89 passed (89)`, `899 passed (899)`; `✓ built`; son tarama çıktısız (bu görev stil eklemez).

- [ ] **Step 5: Commit**

```bash
git add app/ui/modules/useBhavaTable.ts app/ui/panels/BhavaTablePanel.tsx app/ui/panels/BhavaTablePanel.test.tsx app/ui/layout/model.ts app/ui/layout/model.test.ts app/ui/panels/registry.ts app/ui/panels/panels.test.tsx app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): bhava table panel with the start, middle or end of the twelve bhavas and each graha's bhava" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: on dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 5: Arayüz: harita panelinde chalit görünümü (adaptör, çizim, ev sistemi durumları)

Yol haritası §4.5; UI spec §2.3, §3.4, §6; D12, D13, D18. `geometry.ts` değişmez. Var olan üç SVG anlık görüntüsü (`chart/__snapshots__/delhi-d1-*.svg`) ve altı görsel temel değişmez: chalit kapalıyken çizim ve araç çubuğu aynıdır (tam burçta kutu sunulmaz).

**Files:**
- Modify: `app/ui/chart/types.ts`, `app/ui/chart/adapter.ts`, `app/ui/chart/adapter.test.ts`, `app/ui/chart/ChartSvg.tsx`, `app/ui/chart/ChartSvg.test.tsx`, `app/ui/panels/ChartPanel.tsx`, `app/ui/panels/ChartPanel.test.tsx`, `app/ui/panels/BhavaTablePanel.tsx`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: `adapter.test.ts` (+2), `ChartSvg.test.tsx` (+4: üç biçim ve bir kenar durumu), `ChartPanel.test.tsx` (+9: biri üç biçimle)

**Interfaces:**
- Consumes: `Chart.grahas[].house`, `Chart.bhavas.system`, `Chart.meta.warnings`; `chart/geometry` (`houseSign`, `signHouse`); Task 3'ün fikstürü (`bhava.chart_placidus`, `bhava.table_placidus`, `placidusOpened`).
- Produces:
  - `chart/types.ts`: `ChartPlanet.cell?: number`, `ChartPlanet.shifted?: boolean`, `ChartLabels.shifted?: string`, `SHIFT_MARK = "*"`.
  - `chart/adapter.ts`: `chalitData(chart: Chart, data: ChartData): ChartData`.
  - `ChartSvg`: graha `cell ?? sign` burcunun hücresine çizilir; `shifted` graha etiketin sonunda `*` taşır, `data-shifted="true"` alır ve erişilebilir adına `labels.shifted` eklenir.
  - `ChartPanel`: `settings.chalit`; nedenler `chart.chalitOnlyD1`, `chart.chalitTimeUnknown`, `chart.chalitWholeSign`; `chart.chalitNoOverlays`; açıklama `chart.chalitLegend`; `chart.chalitFallback`.
  - `BhavaTablePanel` `SHIFT_MARK`'ı `chart/types`'tan alır (yerel sabit silinir).
  - i18n: `chart.chalit`, `chalitTitle`, `chalitOnlyD1`, `chalitTimeUnknown`, `chalitWholeSign`, `chalitNoOverlays`, `chalitLegend`, `chalitFallback`, `shifted` (üç dil).

**Davranış özeti (testlerin sabitlediği):**
- Kutu ("Bhava chalit") yalnızca ev sistemi tam burç değilse ya da ayar zaten açıksa gösterilir. D1 dışında, saat bilinmezken ve tam burçta `aria-disabled`'dır; nedeni görünür ipucu satırındadır ve `aria-describedby` ile bağlıdır; tıklamak ayarı değiştirmez.
- Çizilirken: svg'nin adı "Bhava chalit haritası" olur; her graha `Chart.grahas[].house` numaralı hücrededir; yer değiştiren graha `*` taşır ve adı "…, bhavası farklı" ile biter; hücrelerin adı ve burç numarası değişmez. Altında açıklama satırı ve (uyarı varsa) Porphyry notu durur. Hiçbir komut çağrılmaz.
- Transit halkası, eş halkası ve arudha padalar çizilmez ve istenmez; ayarı açık olan kutu işaretli durur ve `chart.chalitNoOverlays` satırıyla açıklanır. Chalit kapanınca katmanlar geri gelir.

- [ ] **Step 1: Testler: adaptör, çizim ve panel**

````diff
diff --git a/app/ui/chart/ChartSvg.test.tsx b/app/ui/chart/ChartSvg.test.tsx
index f5ebfcf..f56e89e 100644
--- a/app/ui/chart/ChartSvg.test.tsx
+++ b/app/ui/chart/ChartSvg.test.tsx
@@ -333,3 +333,38 @@ test("a static drawing is one image: no buttons or tab stops, the same cells and
   expect(container.querySelector('[data-graha="Mercury"]')).toHaveTextContent(`Me${RETRO_MARK}c`);
   expect(container.querySelector('[data-graha="Moon"] text')).toHaveAttribute("text-decoration", "underline");
 });
+
+test.each(STYLES)("%s: a shifted graha is drawn in its cell with a mark read as a word", (style) => {
+  // Lagna Aslan (4). Satürn Kova'da (7. burç evi) ama 6. bhavada: Oğlak (9) hücresine çizilir.
+  const chalit: ChartData = {
+    ascSign: 4,
+    planets: [
+      { id: "Sun", sign: 4, degInSign: 12.5, retrograde: false },
+      { id: "Saturn", sign: 10, degInSign: 1, retrograde: true, cell: 9, shifted: true },
+    ],
+  };
+  const { container } = render(<ChartSvg data={chalit} style={style} labels={{ ...labels, shifted: "bhava differs" }} />);
+  const saturn = container.querySelector('[data-graha="Saturn"]')!;
+  expect(saturn).toHaveAttribute("data-house", "6");
+  expect(saturn).toHaveAttribute("data-shifted", "true");
+  expect(saturn.querySelector("text")).toHaveTextContent(`Sa${RETRO_MARK}*`);
+  // Ad grahanın kendi burcunu ve derecesini söyler; işaret sözcükle okunur.
+  expect(screen.getByRole("button", { name: "Sa, Aq 1°, retrograde, bhava differs" })).toBe(saturn);
+  const sun = container.querySelector('[data-graha="Sun"]')!;
+  expect(sun).toHaveAttribute("data-house", "1");
+  expect(sun).not.toHaveAttribute("data-shifted");
+  expect(sun.querySelector("text")).toHaveTextContent("Su");
+});
+
+test("without a word for the mark the name stays as it was; a bad cell is skipped", () => {
+  const chalit: ChartData = {
+    ascSign: 4,
+    planets: [
+      { id: "Saturn", sign: 10, degInSign: 1, retrograde: false, cell: 9, shifted: true },
+      { id: "Mars", sign: 3, degInSign: 2, retrograde: false, cell: 12, shifted: true },
+    ],
+  };
+  const { container } = render(<ChartSvg data={chalit} style="north" labels={labels} />);
+  expect(screen.getByRole("button", { name: "Sa, Aq 1°" })).toBeInTheDocument();
+  expect(container.querySelector('[data-graha="Mars"]')).toBeNull();
+});
diff --git a/app/ui/chart/adapter.test.ts b/app/ui/chart/adapter.test.ts
index c2005f4..a0b7c1e 100644
--- a/app/ui/chart/adapter.test.ts
+++ b/app/ui/chart/adapter.test.ts
@@ -1,5 +1,5 @@
 import { fixture } from "../test/fixture";
-import { chartDataFromChart, isVarga, signIndex, SIGNS, VARGAS } from "./adapter";
+import { chalitData, chartDataFromChart, isVarga, signIndex, SIGNS, VARGAS } from "./adapter";
 
 test("sign and varga lists follow the Rust enums", () => {
   expect(SIGNS).toHaveLength(12);
@@ -50,3 +50,42 @@ test("hidden lagna, missing vargas and null degrees", () => {
   chart.vargas = [];
   expect(chartDataFromChart(chart, "D1", { showLagna: true })).toBeNull();
 });
+
+test("chalit data moves a graha to the cell of its bhava and marks it; sign and degree stay", () => {
+  const f = fixture();
+  // Tam burç: bhava burç evidir; hiçbir graha yer değiştirmez ve nesne aynen döner.
+  const rasi = chartDataFromChart(f.opened.chart, "D1", { showLagna: true })!;
+  const same = chalitData(f.opened.chart, rasi);
+  expect(same.planets).toEqual(rasi.planets);
+  expect(same.planets.some((p) => p.shifted || p.cell !== undefined)).toBe(false);
+  // Placidus (fikstür): Merkür Yay'da (7. burç evi) ama 6. bhavada; Jüpiter Koç'ta (11.) ama
+  // 10. bhavada; Venüs Akrep'te (6.) ama 5. bhavada. Lagna İkizler (2): n. hücrenin burcu
+  // (2 + n − 1) mod 12'dir.
+  const chart = f.bhava.chart_placidus;
+  const data = chartDataFromChart(chart, "D1", { showLagna: true })!;
+  const chalit = chalitData(chart, data);
+  expect(chalit.ascSign).toBe(2);
+  const moved = chalit.planets.filter((p) => p.shifted).map((p) => [p.id, p.sign, p.cell]);
+  expect(moved).toEqual([
+    ["Mercury", 8, 7],
+    ["Jupiter", 0, 11],
+    ["Venus", 7, 6],
+  ]);
+  for (const p of chalit.planets) {
+    const before = data.planets.find((x) => x.id === p.id)!;
+    expect([p.sign, p.degInSign, p.retrograde, p.combust, p.vargottama]).toEqual([before.sign, before.degInSign, before.retrograde, before.combust, before.vargottama]);
+    if (!p.shifted) expect(p).toBe(before);
+  }
+  // Fikstür tablosuyla aynı grahalar (Rust `GrahaBhava.shifted`): arayüz ayrı bir kural uygulamaz.
+  expect(chalit.planets.filter((p) => p.shifted).map((p) => p.id)).toEqual(f.bhava.table_placidus.grahas.filter((g) => g.shifted).map((g) => g.graha));
+  // Girdi değişmez; transit ve işaret alanları taşınır.
+  expect(data.planets.some((p) => p.shifted)).toBe(false);
+  expect(chalitData(chart, { ...data, marks: [{ sign: 1, text: "AL" }] }).marks).toEqual([{ sign: 1, text: "AL" }]);
+});
+
+test("chalit data without a lagna is the data itself", () => {
+  const chart = fixture().bhava.chart_placidus;
+  const data = chartDataFromChart(chart, "D1", { showLagna: false })!;
+  expect(data.ascSign).toBeNull();
+  expect(chalitData(chart, data)).toBe(data);
+});
diff --git a/app/ui/panels/ChartPanel.test.tsx b/app/ui/panels/ChartPanel.test.tsx
index 2a5e04e..e14f4cf 100644
--- a/app/ui/panels/ChartPanel.test.tsx
+++ b/app/ui/panels/ChartPanel.test.tsx
@@ -1,7 +1,7 @@
 import { fireEvent, screen, waitFor } from "@testing-library/react";
 import { api } from "../api";
 import type { Panel } from "../layout/model";
-import { fixture, unknownTimeOpened } from "../test/fixture";
+import { fixture, placidusOpened, unknownTimeOpened } from "../test/fixture";
 import { renderWithStores } from "../test/render";
 import { useLayoutStore } from "../view/store";
 import { ChartPanel, partnerPlanets } from "./ChartPanel";
@@ -336,3 +336,135 @@ test("a failed partner report leaves the chart drawn and offers a retry", async
   fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
   await waitFor(() => expect(container.querySelectorAll(".chart-transit")).toHaveLength(9));
 });
+
+// Chalit görünümü (yol haritası Plan 25). Fikstür: Delhi, Placidus; Merkür, Jüpiter ve Venüs
+// burç evinden bir önceki bhavadadır.
+const unknownPlacidus = () => {
+  const o = placidusOpened();
+  o.chart_record.birth.time = null;
+  o.chart_record.birth.time_certainty = "Unknown";
+  o.chart_record.resolved.warnings = ["TimeUnknownNoon"];
+  return o;
+};
+
+test("whole sign: the chalit box is not offered; a stored setting shows it with the reason and draws the rasi chart", () => {
+  const off = renderPanel({ varga: "D1", style: "north" });
+  expect(screen.queryByRole("checkbox", { name: "Bhava chalit" })).toBeNull();
+  expect(off.container.querySelector(".chart-toolbar-hint")).toBeNull();
+  off.unmount();
+  const { container } = renderPanel({ varga: "D1", style: "north", chalit: true });
+  const box = screen.getByRole("checkbox", { name: "Bhava chalit" });
+  expect(box).toBeChecked();
+  expect(box).toBeEnabled();
+  expect(box).toHaveAttribute("aria-disabled", "true");
+  expect(box).toHaveAccessibleDescription("Ev sistemi tam burç: chalit görünümü rasi ile aynıdır.");
+  expect(screen.getByText("Ev sistemi tam burç: chalit görünümü rasi ile aynıdır.")).not.toHaveClass("visually-hidden");
+  expect(screen.getByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
+  expect(container.querySelector("[data-shifted]")).toBeNull();
+  expect(screen.queryByText(/^Hücreler bhavadır/)).toBeNull();
+});
+
+test("the chalit toggle is saved with the layout and redraws the grahas by bhava without a request", async () => {
+  for (const f of [api.transitsAt, api.computeJaimini, api.computeCompat]) vi.mocked(f).mockClear();
+  const opened = placidusOpened();
+  const { container, persistence } = renderWithStores(<LivePanel />, { layout: LAYOUT({ varga: "D1", style: "north" }), opened });
+  const box = screen.getByRole("checkbox", { name: "Bhava chalit" });
+  expect(box).not.toBeChecked();
+  expect(box).not.toHaveAttribute("aria-disabled");
+  // Rasi görünümü: Merkür burcunun evinde (7).
+  expect(container.querySelector('[data-graha="Mercury"]')).toHaveAttribute("data-house", "7");
+  expect(container.querySelector("[data-shifted]")).toBeNull();
+  fireEvent.click(box);
+  await waitFor(() => expect(screen.getByRole("group", { name: "Bhava chalit haritası" })).toBeInTheDocument());
+  const houses = Object.fromEntries([...container.querySelectorAll("[data-graha]")].map((g) => [g.getAttribute("data-graha"), Number(g.getAttribute("data-house"))]));
+  // Her graha haritanın verdiği bhavadadır (`Chart.grahas[].house`).
+  expect(houses).toEqual(Object.fromEntries(opened.chart.grahas.map((g) => [g.graha, g.house])));
+  expect(houses).toMatchObject({ Sun: 7, Mercury: 6, Jupiter: 10, Venus: 5 });
+  // Çizim sırası hücre sırasıdır (5., 6. ve 10. bhava).
+  expect([...container.querySelectorAll("[data-shifted]")].map((g) => g.getAttribute("data-graha"))).toEqual(["Venus", "Mercury", "Jupiter"]);
+  // İşaret öteki işaretlerden sonra yazılır (Merkür bu haritada yanıktır: "Mec*").
+  expect(container.querySelector('[data-graha="Mercury"] text')).toHaveTextContent(/^Mec\*$/);
+  expect(container.querySelector('[data-graha="Venus"] text')).toHaveTextContent(/^Ve\*$/);
+  // Ad grahanın kendi burcunu söyler; işaret sözcükle okunur.
+  expect(screen.getByRole("button", { name: /^Me, Yay \d+°, yanık, bhavası farklı$/ })).toBeInTheDocument();
+  expect(screen.getByRole("button", { name: /^Ve, Akr \d+°, bhavası farklı$/ })).toBeInTheDocument();
+  expect(screen.getByRole("button", { name: /^Gü, Yay \d+°$/ })).toBeInTheDocument();
+  // Hücreler yerinde: 1. ev lagna burcudur (İkizler).
+  expect(screen.getByRole("button", { name: "1. ev, İki" })).toBeInTheDocument();
+  expect(screen.getByText("Hücreler bhavadır (Placidus); burç numarası o numaralı rasi evinin burcudur. * Graha, burcunun evinden başka bir bhavadadır.")).toBeInTheDocument();
+  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ varga: "D1", style: "north", chalit: true }));
+  fireEvent.click(screen.getByRole("checkbox", { name: "Bhava chalit" }));
+  await waitFor(() => expect(screen.getByRole("group", { name: "D1 haritası" })).toBeInTheDocument());
+  expect(container.querySelector("[data-shifted]")).toBeNull();
+  expect(screen.queryByText(/^Hücreler bhavadır/)).toBeNull();
+  for (const f of [api.transitsAt, api.computeJaimini, api.computeCompat]) expect(f).not.toHaveBeenCalled();
+});
+
+test.each(["north", "south", "circle"])("chalit %s: every graha sits in the cell of its bhava", (style) => {
+  const opened = placidusOpened();
+  const { container } = renderPanel({ varga: "D1", style, chalit: true }, opened);
+  expect(container.querySelector("svg")!.dataset.style).toBe(style);
+  for (const g of opened.chart.grahas) expect(container.querySelector(`[data-graha="${g.graha}"]`), g.graha).toHaveAttribute("data-house", String(g.house));
+  expect(container.querySelectorAll("[data-shifted]")).toHaveLength(3);
+});
+
+test("chalit is drawn on D1 only and never without a birth time; the box says why", () => {
+  const d9 = renderPanel({ varga: "D9", chalit: true }, placidusOpened());
+  const box = screen.getByRole("checkbox", { name: "Bhava chalit" });
+  expect(box).toBeEnabled();
+  expect(box).toHaveAttribute("aria-disabled", "true");
+  expect(box).toHaveAccessibleDescription("Bhava chalit yalnızca D1'de çizilir.");
+  expect(screen.getByRole("group", { name: "D9 haritası" })).toBeInTheDocument();
+  expect(d9.container.querySelector("[data-shifted]")).toBeNull();
+  // Kapalı kutu ayarı değiştirmez.
+  fireEvent.click(box);
+  expect(box).toBeChecked();
+  d9.unmount();
+  const unknown = renderPanel({ varga: "D1", chalit: true }, unknownPlacidus());
+  const off = screen.getByRole("checkbox", { name: "Bhava chalit" });
+  expect(off).toHaveAttribute("aria-disabled", "true");
+  expect(off).toHaveAccessibleDescription("Doğum saati bilinmiyor: bhavalar lagnaya bağlıdır ve chalit görünümü çizilemez.");
+  expect(screen.getByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
+  expect(unknown.container.querySelector("[data-shifted]")).toBeNull();
+});
+
+test("the chalit view draws no ring and no marks, asks for none and says so on the checked boxes", async () => {
+  for (const f of [api.transitsAt, api.computeJaimini, api.computeCompat]) vi.mocked(f).mockClear();
+  const { container } = renderWithStores(<LivePanel />, { layout: LAYOUT({ varga: "D1", transit_ring: true, arudha_marks: true, chalit: true }), opened: placidusOpened() });
+  expect(screen.getByRole("group", { name: "Bhava chalit haritası" })).toBeInTheDocument();
+  expect(container.querySelector(".chart-transit")).toBeNull();
+  expect(container.querySelector(".chart-mark")).toBeNull();
+  const note = "Chalit görünümünde transit halkası, eş halkası ve arudha padalar çizilmez.";
+  expect(screen.getByText(note)).not.toHaveClass("visually-hidden");
+  for (const name of ["Transit halkası", "Arudha padalar"]) {
+    const box = screen.getByRole("checkbox", { name });
+    expect(box).toBeChecked();
+    expect(box).not.toHaveAttribute("aria-disabled");
+    expect(box).toHaveAccessibleDescription(note);
+  }
+  expect(api.transitsAt).not.toHaveBeenCalled();
+  expect(api.computeJaimini).not.toHaveBeenCalled();
+  // Chalit kapanınca katmanlar geri gelir: istekler o zaman yapılır.
+  vi.mocked(api.transitsAt).mockResolvedValue({ status: "ok", data: fixture().transit });
+  vi.mocked(api.computeJaimini).mockResolvedValue({ status: "ok", data: fixture().jaimini });
+  fireEvent.click(screen.getByRole("checkbox", { name: "Bhava chalit" }));
+  await waitFor(() => expect(container.querySelectorAll(".chart-transit")).toHaveLength(9));
+  await waitFor(() => expect(container.querySelectorAll(".chart-mark")).toHaveLength(12));
+  expect(screen.queryByText(note)).toBeNull();
+});
+
+test("an unchecked layer carries no chalit note", () => {
+  renderPanel({ varga: "D1", chalit: true }, placidusOpened());
+  expect(screen.queryByText("Chalit görünümünde transit halkası, eş halkası ve arudha padalar çizilmez.")).toBeNull();
+  expect(screen.getByRole("checkbox", { name: "Transit halkası" })).not.toHaveAccessibleDescription();
+});
+
+test("Placidus inside the polar circle: the chalit view says the bhavas are Porphyry", () => {
+  const opened = placidusOpened();
+  opened.chart.meta.warnings = ["PlacidusPolarFallback"];
+  renderPanel({ varga: "D1", chalit: true }, opened);
+  expect(screen.getByText("Placidus bu enlemde hesaplanamadı; bhavalar Porphyry noktalarından sayılır.")).toHaveAttribute("role", "note");
+  // Rasi görünümünde not yoktur (uyarı rozeti harita başlığındadır).
+  const rasi = renderPanel({ varga: "D1" }, opened);
+  expect(rasi.container.querySelector('[role="note"]')).toBeNull();
+});
````

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/chart/adapter.test.ts ui/chart/ChartSvg.test.tsx ui/panels/ChartPanel.test.tsx 2>&1 | grep -E "Test Files|Tests |FAIL" | head`
Expected: üç dosya da düşer (`chalitData` dışa aktarılmıyor; `data-shifted` yok; "Bhava chalit" kutusu yok). Var olan testler geçer.

- [ ] **Step 3: Tipler, adaptör ve çizim**

````diff
diff --git a/app/ui/chart/ChartSvg.tsx b/app/ui/chart/ChartSvg.tsx
index cac5e01..77ec10b 100644
--- a/app/ui/chart/ChartSvg.tsx
+++ b/app/ui/chart/ChartSvg.tsx
@@ -7,7 +7,7 @@ import { useMemo, type KeyboardEvent } from "react";
 import { sameTarget, type HighlightTarget } from "../highlight/target";
 import "./chart.css";
 import { chartCells, layoutLabels, pathOf, VIEWBOX, type ChartCell, type LabelLayout } from "./geometry";
-import { RETRO_MARK, showsRetro, type ChartData, type ChartLabels, type ChartMark, type ChartPlanet, type ChartStyle } from "./types";
+import { RETRO_MARK, SHIFT_MARK, showsRetro, type ChartData, type ChartLabels, type ChartMark, type ChartPlanet, type ChartStyle } from "./types";
 
 export interface ChartSvgProps {
   data: ChartData;
@@ -30,9 +30,16 @@ function cellHighlighted(cell: ChartCell, h: HighlightTarget | null): boolean {
   return (h.type === "house" && h.house === cell.house) || (h.type === "sign" && h.sign === cell.sign);
 }
 
-/** Çizilebilir gezegen: burç 0…11 tamsayı, derece sonlu. Diğerleri açıkça atlanır. */
+/** Grahanın çizildiği hücrenin burcu: chalit görünümünde `cell`, değilse kendi burcu. */
+function cellSign(p: ChartPlanet): number {
+  return p.cell ?? p.sign;
+}
+
+const isSign = (n: number) => Number.isInteger(n) && n >= 0 && n <= 11;
+
+/** Çizilebilir gezegen: burç ve hücre 0…11 tamsayı, derece sonlu. Diğerleri açıkça atlanır. */
 function drawable(p: ChartPlanet): boolean {
-  return Number.isInteger(p.sign) && p.sign >= 0 && p.sign <= 11 && Number.isFinite(p.degInSign);
+  return isSign(p.sign) && isSign(cellSign(p)) && Number.isFinite(p.degInSign);
 }
 
 /** Çizilebilir işaret: burç 0…11 tamsayı ve metin boş değil. */
@@ -41,7 +48,7 @@ function drawableMark(m: ChartMark): boolean {
 }
 
 function labelText(p: ChartPlanet, graha: ChartLabels["graha"]): string {
-  return `${graha[p.id]}${showsRetro(p.id, p.retrograde) ? RETRO_MARK : ""}${p.combust ? "c" : ""}`;
+  return `${graha[p.id]}${showsRetro(p.id, p.retrograde) ? RETRO_MARK : ""}${p.combust ? "c" : ""}${p.shifted ? SHIFT_MARK : ""}`;
 }
 
 /** Transit etiketi: parantez biçimsel işarettir (renk değil, C9). */
@@ -55,6 +62,7 @@ function planetName(p: ChartPlanet, labels: ChartLabels): string {
   if (showsRetro(p.id, p.retrograde)) parts.push(labels.retro);
   if (p.combust) parts.push(labels.combust);
   if (p.vargottama) parts.push(labels.vargottama);
+  if (p.shifted && labels.shifted) parts.push(labels.shifted);
   return parts.join(", ");
 }
 
@@ -73,7 +81,7 @@ export function ChartSvg({ data, style, labels, highlight = null, pressed = high
       const bySign = new Map<number, ChartPlanet[]>();
       for (const p of list) {
         if (!drawable(p)) continue;
-        bySign.set(p.sign, [...(bySign.get(p.sign) ?? []), p]);
+        bySign.set(cellSign(p), [...(bySign.get(cellSign(p)) ?? []), p]);
       }
       return bySign;
     };
@@ -165,6 +173,7 @@ export function ChartSvg({ data, style, labels, highlight = null, pressed = high
               className={`chart-planet${on ? " chart-planet--highlight" : ""}`}
               data-graha={p.id}
               data-house={cell.house}
+              data-shifted={p.shifted ? "true" : undefined}
               data-x={x}
               data-y={y}
               {...button(target, isPressed, planetName(p, labels))}
@@ -189,6 +198,7 @@ export function ChartSvg({ data, style, labels, highlight = null, pressed = high
                 <tspan>{labels.graha[p.id]}</tspan>
                 {showsRetro(p.id, p.retrograde) && <tspan>{RETRO_MARK}</tspan>}
                 {p.combust && <tspan>c</tspan>}
+                {p.shifted && <tspan>{SHIFT_MARK}</tspan>}
               </text>
             </g>
           );
diff --git a/app/ui/chart/adapter.ts b/app/ui/chart/adapter.ts
index f329b16..2162e43 100644
--- a/app/ui/chart/adapter.ts
+++ b/app/ui/chart/adapter.ts
@@ -1,6 +1,7 @@
 // Chart → ChartData. Burç, derece, retro, yanma ve vargottama Rust'tan hazır gelir
 // (`Chart.vargas`, `Chart.grahas`); arayüz hiçbir boylamdan burç türetmez (spec §4).
 import type { Chart, Sign, TransitView, Varga } from "../bindings";
+import { houseSign, signHouse } from "./geometry";
 import { isGrahaId, type ChartData, type ChartPlanet } from "./types";
 
 /** 0 = Mesha (Koç) … 11 = Meena (Balık); bindings `Sign` sırası. */
@@ -74,6 +75,25 @@ export function chartDataFromChart(chart: Chart, varga: Varga, { showLagna }: {
   return { ascSign: showLagna ? signIndex(v.lagna_sign) : null, planets };
 }
 
+/**
+ * Chalit görünümü (yol haritası §4.5): her graha, bhavasının numarasını taşıyan hücreye çizilir.
+ * Bhava Rust'tan gelir (`Chart.grahas[].house`: haritanın ev sistemindeki ev); hücre, o numaralı
+ * rasi evinin hücresidir (`houseSign`: gösterim eşlemesi, hesap değil). Burcunun evinden başka
+ * bir bhavadaki graha `shifted` işaretini alır; burcu ve derecesi değişmez. Lagna yoksa (saat
+ * bilinmiyor) veri olduğu gibi döner.
+ */
+export function chalitData(chart: Chart, data: ChartData): ChartData {
+  const asc = data.ascSign;
+  if (asc === null) return data;
+  const houses = new Map(chart.grahas.map((g) => [g.graha, g.house]));
+  const planets = data.planets.map((p) => {
+    const house = houses.get(p.id);
+    if (house === undefined || house === signHouse(asc, p.sign)) return p;
+    return { ...p, cell: houseSign(asc, house), shifted: true };
+  });
+  return { ...data, planets };
+}
+
 /** `transits_at` dönüşü → çizim verisi (burç ve derece Rust'tan; dönüşüm yalnızca biçimdir). */
 export function transitPlanets(view: TransitView): ChartPlanet[] {
   return view.snapshot.grahas.flatMap((g) =>
diff --git a/app/ui/chart/types.ts b/app/ui/chart/types.ts
index 19dd7d5..cd88eb2 100644
--- a/app/ui/chart/types.ts
+++ b/app/ui/chart/types.ts
@@ -18,6 +18,13 @@ export interface ChartPlanet {
   retrograde: boolean;
   combust?: boolean;
   vargottama?: boolean;
+  /**
+   * Chalit görünümü: grahanın çizildiği hücrenin burcu (bhavasının numarasını taşıyan hücre).
+   * Yoksa graha kendi burcunun hücresine çizilir.
+   */
+  cell?: number;
+  /** Chalit görünümü: graha burcunun evinden başka bir bhavadadır; `SHIFT_MARK` ile işaretlenir. */
+  shifted?: boolean;
 }
 
 /** Bir burca yazılan kısa işaret (ör. arudha pada "AL", "A2"); etkileşimsizdir. */
@@ -62,11 +69,16 @@ export interface ChartLabels {
   combust: string;
   /** Erişilebilir adda alt çizgi yerine okunan sözcük (örn. "vargottama"). */
   vargottama: string;
+  /** Erişilebilir adda `*` yerine okunan sözcük (chalit görünümü; örn. "bhavası farklı"). */
+  shifted?: string;
 }
 
 /** Retro işareti: harita etiketinde ve gezegen tablosunda düz "R" (spec §2.3). */
 export const RETRO_MARK = "R";
 
+/** Burç evinden başka bir bhavadaki grahanın işareti (chalit görünümü ve bhava tablosu). */
+export const SHIFT_MARK = "*";
+
 /**
  * Retro işareti Rahu ve Ketu'da gösterilmez (kullanıcı kararı): ortalama düğüm hep geri gider.
  * Gerçek düğüm ara sıra ileri gidebilir; yönü gezegen tablosundaki hız sütunu gösterir.
diff --git a/app/ui/panels/BhavaTablePanel.tsx b/app/ui/panels/BhavaTablePanel.tsx
index 177d58e..f376967 100644
--- a/app/ui/panels/BhavaTablePanel.tsx
+++ b/app/ui/panels/BhavaTablePanel.tsx
@@ -8,6 +8,7 @@ import { useId } from "react";
 import { useTranslation } from "react-i18next";
 import type { BhavaPoint, BhavaTable } from "../bindings";
 import { signIndex } from "../chart/adapter";
+import { SHIFT_MARK } from "../chart/types";
 import { useHighlight } from "../highlight/store";
 import type { Panel } from "../layout/model";
 import { useBhavaTable } from "../modules/useBhavaTable";
@@ -23,8 +24,6 @@ import type { PanelProps } from "./types";
 
 export const BHAVA_POINTS = ["start", "mid", "end"] as const;
 export type BhavaPointKind = (typeof BHAVA_POINTS)[number];
-/** Burç evinden başka bir bhavadaki grahanın işareti. */
-export const SHIFT_MARK = "*";
 const DASH = "—";
 
 /** Derece ve yay dakikası; Rust'ta sonlu olmayan değer (`null`) çizgidir. */
````

- [ ] **Step 4: Harita paneli ve metinler**

İpucu satırı bir listeden kurulur (`hints`): var olan üç neden aynı kimliklerle (`…-ring`, `…-partner`, `…-marks`) ve aynı sırayla durur; chalit nedeni ve "katmanlar çizilmez" satırı sona eklenir. `unknown` hesabı yukarı taşınır (chalit nedeni onu kullanır).

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index b3cea43..b362ae7 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -39,6 +39,15 @@ export const en: Messages = {
     arudhaMarksOnlyD1: "Arudha padas are written on D1 only.",
     arudhaMarksTimeUnknown: "Birth time unknown: arudha padas depend on the lagna and cannot be written.",
     arudhaList: "Arudha padas: {{list}}",
+    chalit: "Bhava chalit",
+    chalitTitle: "Bhava chalit chart",
+    chalitOnlyD1: "Bhava chalit is drawn in D1 only.",
+    chalitTimeUnknown: "Birth time unknown: the bhavas depend on the lagna and the chalit view cannot be drawn.",
+    chalitWholeSign: "House system is whole sign: the chalit view equals the rasi chart.",
+    chalitNoOverlays: "The chalit view does not draw the transit ring, the partner ring or the arudha padas.",
+    chalitLegend: "The cells are bhavas ({{system}}); the sign number is the sign of the rasi house with that number. {{mark}} The graha is in a bhava other than the house of its sign.",
+    chalitFallback: "Placidus could not be computed at this latitude; the bhavas are counted from Porphyry points.",
+    shifted: "bhava differs",
     title: "{{varga}} chart",
     lagna: "As",
     house: "House {{house}}, {{sign}}",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index fb1664d..f78b568 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -39,6 +39,15 @@ export const fr: Messages = {
     arudhaMarksOnlyD1: "Les arudha padas ne sont inscrits que sur D1.",
     arudhaMarksTimeUnknown: "Heure de naissance inconnue : les arudha padas dépendent du lagna et ne peuvent pas être inscrits.",
     arudhaList: "Arudha padas : {{list}}",
+    chalit: "Bhava chalit",
+    chalitTitle: "Carte bhava chalit",
+    chalitOnlyD1: "Le bhava chalit n’est tracé qu’en D1.",
+    chalitTimeUnknown: "Heure de naissance inconnue : les bhavas dépendent du lagna et la vue chalit ne peut pas être tracée.",
+    chalitWholeSign: "Système de maisons en signe entier : la vue chalit est identique à la carte rasi.",
+    chalitNoOverlays: "La vue chalit ne trace ni l’anneau des transits, ni l’anneau du partenaire, ni les arudha padas.",
+    chalitLegend: "Les cases sont des bhavas ({{system}}) ; le numéro de signe est celui de la maison rasi de même numéro. {{mark}} Le graha est dans un bhava autre que la maison de son signe.",
+    chalitFallback: "Placidus n’a pas pu être calculé à cette latitude ; les bhavas sont comptés depuis des points de Porphyre.",
+    shifted: "bhava différent",
     title: "Carte {{varga}}",
     lagna: "As",
     house: "Maison {{house}}, {{sign}}",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 4fced2a..6a4b78a 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -38,6 +38,15 @@ export const tr = {
     arudhaMarksOnlyD1: "Arudha padalar yalnızca D1'de yazılır.",
     arudhaMarksTimeUnknown: "Doğum saati bilinmiyor: arudha padalar lagnaya bağlıdır ve yazılamaz.",
     arudhaList: "Arudha padalar: {{list}}",
+    chalit: "Bhava chalit",
+    chalitTitle: "Bhava chalit haritası",
+    chalitOnlyD1: "Bhava chalit yalnızca D1'de çizilir.",
+    chalitTimeUnknown: "Doğum saati bilinmiyor: bhavalar lagnaya bağlıdır ve chalit görünümü çizilemez.",
+    chalitWholeSign: "Ev sistemi tam burç: chalit görünümü rasi ile aynıdır.",
+    chalitNoOverlays: "Chalit görünümünde transit halkası, eş halkası ve arudha padalar çizilmez.",
+    chalitLegend: "Hücreler bhavadır ({{system}}); burç numarası o numaralı rasi evinin burcudur. {{mark}} Graha, burcunun evinden başka bir bhavadadır.",
+    chalitFallback: "Placidus bu enlemde hesaplanamadı; bhavalar Porphyry noktalarından sayılır.",
+    shifted: "bhavası farklı",
     title: "{{varga}} haritası",
     lagna: "Lg",
     house: "{{house}}. ev, {{sign}}",
diff --git a/app/ui/panels/ChartPanel.tsx b/app/ui/panels/ChartPanel.tsx
index cfd60d8..2b7fb2b 100644
--- a/app/ui/panels/ChartPanel.tsx
+++ b/app/ui/panels/ChartPanel.tsx
@@ -1,9 +1,9 @@
-import { useId } from "react";
+import { Fragment, useId } from "react";
 import { useTranslation } from "react-i18next";
 import type { KutaReport, KutaRole, Varga } from "../bindings";
-import { chartDataFromChart, isVarga, signIndex, transitPlanets, VARGA_NAMES, VARGAS } from "../chart/adapter";
+import { chalitData, chartDataFromChart, isVarga, signIndex, transitPlanets, VARGA_NAMES, VARGAS } from "../chart/adapter";
 import { ChartSvg, type ChartSvgProps } from "../chart/ChartSvg";
-import { CHART_STYLES, isChartStyle, isGrahaId, showsRetro, type ChartData, type ChartMark, type ChartPlanet } from "../chart/types";
+import { CHART_STYLES, isChartStyle, isGrahaId, SHIFT_MARK, showsRetro, type ChartData, type ChartMark, type ChartPlanet } from "../chart/types";
 import { useHighlight } from "../highlight/store";
 import { useCompat } from "../modules/useCompat";
 import { useJaimini } from "../modules/useJaimini";
@@ -41,6 +41,12 @@ export function partnerPlanets(report: KutaReport, openRole: KutaRole): ChartPla
  * diğeri kapanır, kayıtta ikisi de açıksa transit halkası çizilir. Eş haritanındır, düzen
  * kullanıcınındır: kutu yalnızca açık haritanın bir eşi varken gösterilir; eşi olmayan haritada
  * ayar açık olsa da hiçbir şey istenmez ve çizilmez.
+ * `settings.chalit` açıkken (yalnızca D1, doğum saati biliniyorsa, ev sistemi tam burç değilse)
+ * grahalar burçlarına göre değil bhavalarına göre yerleşir (yol haritası §4.5): bhava haritanın
+ * kendisinden gelir (`Chart.grahas[].house`), yeni bir istek yapılmaz. Burcunun evinden başka bir
+ * bhavadaki graha `*` ile işaretlenir. Chalit görünümünde transit halkası, eş halkası ve arudha
+ * padalar çizilmez ve istenmez (onlar burca göre yerleşir). Kutu, ev sistemi tam burçken yalnızca
+ * ayar açıksa gösterilir ve nedenini söyler: tam burçta chalit görünümü rasi ile aynıdır.
  */
 export function ChartPanel({ panel }: PanelProps) {
   const { t } = useTranslation();
@@ -50,19 +56,23 @@ export function ChartPanel({ panel }: PanelProps) {
   const id = useId();
   const varga = chartVarga(panel.settings);
   const style = isChartStyle(panel.settings.style) ? panel.settings.style : "north";
+  const unknown = opened !== null && timeUnknown(opened);
+  const wholeSign = opened?.chart.bhavas.system === "WholeSign";
+  const chalitOn = panel.settings.chalit === true;
+  const chalitOff = varga !== "D1" ? "chalitOnlyD1" : unknown ? "chalitTimeUnknown" : wholeSign ? "chalitWholeSign" : null;
+  const chalitDrawn = chalitOn && chalitOff === null;
   const ringOn = panel.settings.transit_ring === true;
   const ringOff = varga !== "D1";
-  const ringDrawn = ringOn && !ringOff;
+  const ringDrawn = ringOn && !ringOff && !chalitDrawn;
   const partnerOn = panel.settings.partner_ring === true && !ringOn;
   // Eş bağı açık haritanın görünüm kaydındadır; rapor yalnızca halka çizilecekse istenir.
   const hasPartner = useViewStore().model.compat.partnerUuid !== null;
   const partnerOff = varga !== "D1";
-  const compat = useCompat(opened, hasPartner && partnerOn && !partnerOff);
+  const compat = useCompat(opened, hasPartner && partnerOn && !partnerOff && !chalitDrawn);
   const marksOn = panel.settings.arudha_marks === true;
-  const unknown = opened !== null && timeUnknown(opened);
   // Padalar lagnaya bağlıdır: saat bilinmiyorsa istenmez (Rust zaten boş döner, K2).
   const marksOff = varga !== "D1" ? "arudhaMarksOnlyD1" : unknown ? "arudhaMarksTimeUnknown" : null;
-  const jaimini = useJaimini(opened, marksOn && marksOff === null);
+  const jaimini = useJaimini(opened, marksOn && marksOff === null && !chalitDrawn);
 
   if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
   const base = chartDataFromChart(opened.chart, varga, { showLagna: !unknown });
@@ -83,6 +93,16 @@ export function ChartPanel({ panel }: PanelProps) {
     onHover: highlight.hover,
     onPin: highlight.togglePin,
   };
+  // Chalit görünümünde çizilmeyen, ama ayarı açık olan katmanlar: kutuları nedenini söyler.
+  const hiddenByChalit = chalitDrawn && (ringOn || (hasPartner && partnerOn) || marksOn);
+  const hints: { id: string; text: string }[] = [];
+  if (ringOff) hints.push({ id: `${id}-ring`, text: t("chart.transitRingOnlyD1") });
+  if (hasPartner && partnerOff) hints.push({ id: `${id}-partner`, text: t("chart.partnerRingOnlyD1") });
+  if (marksOff !== null) hints.push({ id: `${id}-marks`, text: t(`chart.${marksOff}`) });
+  const showChalit = !wholeSign || chalitOn;
+  if (showChalit && chalitOff !== null) hints.push({ id: `${id}-chalit`, text: t(`chart.${chalitOff}`) });
+  if (hiddenByChalit) hints.push({ id: `${id}-overlays`, text: t("chart.chalitNoOverlays") });
+  const overlayHint = (on: boolean, reason: string | undefined) => reason ?? (chalitDrawn && on ? `${id}-overlays` : undefined);
   return (
     <div className="chart-panel">
       <div className="chart-panel-toolbar">
@@ -117,7 +137,7 @@ export function ChartPanel({ panel }: PanelProps) {
             type="checkbox"
             checked={ringOn}
             aria-disabled={ringOff || undefined}
-            aria-describedby={ringOff ? `${id}-ring` : undefined}
+            aria-describedby={overlayHint(ringOn, ringOff ? `${id}-ring` : undefined)}
             onChange={(e) => !ringOff && setPanelSettings(panel.id, e.target.checked && panel.settings.partner_ring === true ? { transit_ring: true, partner_ring: false } : { transit_ring: e.target.checked })}
           />
           {t("chart.transitRing")}
@@ -128,7 +148,7 @@ export function ChartPanel({ panel }: PanelProps) {
               type="checkbox"
               checked={partnerOn}
               aria-disabled={partnerOff || undefined}
-              aria-describedby={partnerOff ? `${id}-partner` : undefined}
+              aria-describedby={overlayHint(partnerOn, partnerOff ? `${id}-partner` : undefined)}
               onChange={(e) => !partnerOff && setPanelSettings(panel.id, e.target.checked && ringOn ? { partner_ring: true, transit_ring: false } : { partner_ring: e.target.checked })}
             />
             {t("chart.partnerRing")}
@@ -139,23 +159,32 @@ export function ChartPanel({ panel }: PanelProps) {
             type="checkbox"
             checked={marksOn}
             aria-disabled={marksOff !== null || undefined}
-            aria-describedby={marksOff !== null ? `${id}-marks` : undefined}
+            aria-describedby={overlayHint(marksOn, marksOff !== null ? `${id}-marks` : undefined)}
             onChange={(e) => marksOff === null && setPanelSettings(panel.id, { arudha_marks: e.target.checked })}
           />
           {t("chart.arudhaMarks")}
         </label>
+        {showChalit && (
+          <label className={chalitOff !== null ? "chart-ring-toggle is-off" : "chart-ring-toggle"}>
+            <input
+              type="checkbox"
+              checked={chalitOn}
+              aria-disabled={chalitOff !== null || undefined}
+              aria-describedby={chalitOff !== null ? `${id}-chalit` : undefined}
+              onChange={(e) => chalitOff === null && setPanelSettings(panel.id, { chalit: e.target.checked })}
+            />
+            {t("chart.chalit")}
+          </label>
+        )}
       </div>
-      {(ringOff || marksOff !== null) && (
+      {hints.length > 0 && (
         <p className="chart-toolbar-hint">
-          {ringOff && <span id={`${id}-ring`}>{t("chart.transitRingOnlyD1")}</span>}
-          {hasPartner && partnerOff && (
-            <>
-              {" "}
-              <span id={`${id}-partner`}>{t("chart.partnerRingOnlyD1")}</span>
-            </>
-          )}
-          {ringOff && marksOff !== null && " "}
-          {marksOff !== null && <span id={`${id}-marks`}>{t(`chart.${marksOff}`)}</span>}
+          {hints.map((h, i) => (
+            <Fragment key={h.id}>
+              {i > 0 && " "}
+              <span id={h.id}>{h.text}</span>
+            </Fragment>
+          ))}
         </p>
       )}
       {jaimini.error && <PanelError error={jaimini.error} onRetry={jaimini.retry} />}
@@ -169,11 +198,27 @@ export function ChartPanel({ panel }: PanelProps) {
       )}
       {data ? (
         <div className="chart-panel-figure">
-          {ringDrawn ? <RingChart data={data} {...svgProps} /> : <ChartSvg data={partner.length > 0 ? { ...data, transits: partner } : data} {...svgProps} />}
+          {chalitDrawn ? (
+            <ChartSvg data={chalitData(opened.chart, data)} {...svgProps} labels={{ ...svgProps.labels, title: t("chart.chalitTitle"), shifted: t("chart.shifted") }} />
+          ) : ringDrawn ? (
+            <RingChart data={data} {...svgProps} />
+          ) : (
+            <ChartSvg data={partner.length > 0 ? { ...data, transits: partner } : data} {...svgProps} />
+          )}
         </div>
       ) : (
         <p className="panel-message">{t("chart.vargaMissing", { varga })}</p>
       )}
+      {data && chalitDrawn && (
+        <>
+          <p className="panel-message">{t("chart.chalitLegend", { mark: SHIFT_MARK, system: t(`settings.house.${opened.chart.bhavas.system}`) })}</p>
+          {opened.chart.meta.warnings.includes("PlacidusPolarFallback") && (
+            <p className="panel-notice" role="note">
+              {t("chart.chalitFallback")}
+            </p>
+          )}
+        </>
+      )}
       {/* Çizimdeki işaretler `aria-hidden`'dır: padalar ve eşin grahaları burada sözcükle okunur. */}
       {data && marks.length > 0 && <p className="visually-hidden">{t("chart.arudhaList", { list: markList })}</p>}
       {data && partner.length > 0 && <p className="visually-hidden">{t("chart.partnerList", { name: compat.partner?.brief.client_name ?? "", list: partnerList })}</p>}
````

- [ ] **Step 5: Testler ve kapılar**

```bash
pnpm --dir app exec vitest run ui/chart ui/panels/ChartPanel.test.tsx ui/panels/BhavaTablePanel.test.tsx 2>&1 | grep -E "Test Files|Tests |FAIL|obsolete|written"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
git status --short -- app/ui/chart/__snapshots__ app/ui/chart/geometry.ts
```
Expected: süzgeçli koşu yeşil ve anlık görüntü satırı yok (`obsolete` ya da `written` yazmaz: üç SVG anlık görüntüsü değişmedi); Vitest `89 passed (89)`, `914 passed (914)`; `✓ built`; son komut çıktısız.

- [ ] **Step 6: Commit**

```bash
git add app/ui/chart/types.ts app/ui/chart/adapter.ts app/ui/chart/adapter.test.ts app/ui/chart/ChartSvg.tsx app/ui/chart/ChartSvg.test.tsx app/ui/panels/ChartPanel.tsx app/ui/panels/ChartPanel.test.tsx app/ui/panels/BhavaTablePanel.tsx app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): chalit view in the chart panel draws each graha in the cell of its bhava and marks the ones that left their sign house" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: on bir dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 6: Arayüz: Sudarshana geometrisi, çizimi ve paneli (ortak seçim, tablo karşılığı)

Yol haritası §4.6; UI spec §3.3, §3.4, §6, §8; K12; D14, D15. Etiket yerleşimi var olan `layoutLabels`'tır (`geometry.ts` değişmez): her hücrenin etiket alanı, halka diliminin burç numarası şeridi dışındaki parçasıdır.

**Files:**
- Create: `app/ui/chart/sudarshana.ts`, `app/ui/chart/sudarshana.test.ts`, `app/ui/chart/SudarshanaSvg.tsx`, `app/ui/chart/SudarshanaSvg.test.tsx`, `app/ui/modules/useSudarshana.ts`, `app/ui/panels/SudarshanaPanel.tsx`, `app/ui/panels/SudarshanaPanel.test.tsx`
- Modify: `app/ui/chart/chart.css`, `app/ui/panels/panels.css`, `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: üç yeni dosya (4 + 5 + 7); `layout/model.test.ts` ve `panels/panels.test.tsx` (var olan testlere beklenti)

**Interfaces:**
- Consumes: Task 3'ün `api.computeSudarshana`, bağlama tipi `SudarshanaChakra`, fikstür (`bhava.sudarshana`, `bhava.sudarshana_time_unknown`, `bhava.sudarshana_refs`); `chart/geometry` (`layoutLabels`, `pathOf`, `VIEWBOX`, `ChartCell`, `Point`, `LABEL_FONT_MIN`, `pointInPolygon`); `chart/types` (`GrahaId`, `isGrahaId`); `panels/Approx`, `panels/bala.BalaScroll`, `panels/labels.grahaAbbreviations`, `panels/PinButton`, `panels/states`; `highlight/store.useHighlight`.
- Produces:
  - `chart/sudarshana.ts`: `SUDARSHANA_RING_COUNT` (3), `SUDARSHANA_RADII` (`[5, 22, 36, 49]`), `SUDARSHANA_SIGN_BAND` (2.6), `SUDARSHANA_SIGN_FONT` (2.2), `SudarshanaCell`, `sudarshanaCell(ring, house)`, `sudarshanaCells()`.
  - `chart/SudarshanaSvg.tsx`: `SudarshanaSvg({ rings, labels, highlight, onHover, onPin })`, `SudarshanaRingData`, `SudarshanaLabels`.
  - `modules/useSudarshana.ts`: `useSudarshana(opened): ModuleState<SudarshanaChakra>`.
  - `panels/SudarshanaPanel.tsx`: `SudarshanaPanel(props)`.
  - `layout/model.ts`: panel türü `"sudarshana"`; `MODULE_PANELS` girdisi `{ kind: "sudarshana", w: 6, h: 24 }`.
  - i18n: `panel.sudarshana`, `sudarshana.*` (üç dil). Stil: `.sudarshana-sign`, `.sudarshana-hub`, `.sudarshana-figure`.

- [ ] **Step 1: Testler: geometri, çizim, panel ve katalog**

Panel testinin tablosu fikstürdeki chakradır (Delhi: lagna İkizler, Ay Terazi, Güneş Yay); çizim testleri sentetik üç halka kullanır (lagna Akrep, Ay Yengeç, Güneş Koç: Task 2'nin el örneğiyle aynı yerleşim).

````diff
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index f945a41..9bfb04c 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -48,6 +48,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "shadbala",
     "bhava_bala",
     "bhava_table",
+    "sudarshana",
     "yogas",
     "compat",
     "muhurta_day",
@@ -61,7 +62,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "bhava_table", "yogas", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "bhava_table", "sudarshana", "yogas", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index f96b547..1b3f9ab 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -25,6 +25,7 @@ import { MuhurtaSearchPanel } from "./MuhurtaSearchPanel";
 import { PanchangPanel } from "./PanchangPanel";
 import { panelDefinition, PANEL_REGISTRY } from "./registry";
 import { ShadbalaPanel } from "./ShadbalaPanel";
+import { SudarshanaPanel } from "./SudarshanaPanel";
 import { UnavailablePanel } from "./UnavailablePanel";
 import { YogasPanel } from "./YogasPanel";
 
@@ -57,6 +58,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.shadbala.component).toBe(ShadbalaPanel);
   expect(PANEL_REGISTRY.bhava_bala.component).toBe(BhavaBalaPanel);
   expect(PANEL_REGISTRY.bhava_table.component).toBe(BhavaTablePanel);
+  expect(PANEL_REGISTRY.sudarshana.component).toBe(SudarshanaPanel);
   expect(PANEL_REGISTRY.yogas.component).toBe(YogasPanel);
   expect(PANEL_REGISTRY.compat.component).toBe(CompatPanel);
   expect(PANEL_REGISTRY.muhurta_day.component).toBe(MuhurtaDayPanel);
````

**Yeni dosya:** `app/ui/chart/sudarshana.test.ts`

````ts
import { LABEL_FONT_MIN, layoutLabels, pointInPolygon } from "./geometry";
import { SUDARSHANA_RADII, SUDARSHANA_RING_COUNT, SUDARSHANA_SIGN_BAND, sudarshanaCell, sudarshanaCells } from "./sudarshana";

const radius = (p: { x: number; y: number }) => Math.hypot(p.x - 50, p.y - 50);
/** Matematiksel açı, derece, [0, 360). */
const angle = (p: { x: number; y: number }) => ((Math.atan2(50 - p.y, p.x - 50) * 180) / Math.PI + 360) % 360;

test("thirty-six cells: three rings of twelve houses inside the view box", () => {
  const cells = sudarshanaCells();
  expect(cells).toHaveLength(SUDARSHANA_RING_COUNT * 12);
  expect(cells.map((c) => [c.ring, c.house])).toEqual(Array.from({ length: 36 }, (_, i) => [Math.floor(i / 12), (i % 12) + 1]));
  for (const c of cells) {
    for (const p of c.points) {
      expect(radius(p)).toBeGreaterThanOrEqual(SUDARSHANA_RADII[c.ring] - 1e-9);
      expect(radius(p)).toBeLessThanOrEqual(SUDARSHANA_RADII[c.ring + 1] + 1e-9);
    }
  }
  // Üç halka iç içedir ve kutunun içinde durur.
  expect([...SUDARSHANA_RADII]).toEqual([...SUDARSHANA_RADII].sort((a, b) => a - b));
  expect(SUDARSHANA_RADII[SUDARSHANA_RING_COUNT]).toBeLessThan(50);
});

test("house one is on the left and houses run counter-clockwise at the same angle in every ring", () => {
  for (let ring = 0; ring < SUDARSHANA_RING_COUNT; ring++) {
    expect(angle(sudarshanaCell(ring, 1).signAnchor)).toBeCloseTo(180, 6);
    expect(angle(sudarshanaCell(ring, 4).signAnchor)).toBeCloseTo(270, 6);
    expect(angle(sudarshanaCell(ring, 7).signAnchor)).toBeCloseTo(0, 6);
    expect(angle(sudarshanaCell(ring, 10).signAnchor)).toBeCloseTo(90, 6);
    for (let house = 1; house <= 12; house++) {
      const c = sudarshanaCell(ring, house);
      expect(angle(c.labelCell.planetAnchor)).toBeCloseTo(angle(sudarshanaCell(0, house).labelCell.planetAnchor), 6);
      expect(c.labelCell.house).toBe(house);
    }
  }
});

test("the sign number sits in the inner band and the labels outside it, both inside the cell", () => {
  for (const c of sudarshanaCells()) {
    const rInner = SUDARSHANA_RADII[c.ring];
    expect(pointInPolygon(c.signAnchor, c.points)).toBe(true);
    expect(radius(c.signAnchor)).toBeLessThan(rInner + SUDARSHANA_SIGN_BAND);
    expect(pointInPolygon(c.labelCell.planetAnchor, c.labelCell.points)).toBe(true);
    expect(c.labelCell.sector!.rInner).toBeCloseTo(rInner + SUDARSHANA_SIGN_BAND, 9);
    for (const p of c.labelCell.points) expect(radius(p)).toBeGreaterThanOrEqual(rInner + SUDARSHANA_SIGN_BAND - 1e-9);
  }
});

test("labels fit every cell: up to six in the inner ring and four in the outer rings", () => {
  const texts = ["Gü", "Ay", "Ma", "Me", "Jü", "Ve", "Sa", "Ra", "Ke"];
  for (const c of sudarshanaCells()) {
    const most = c.ring === 0 ? 6 : 4;
    for (let n = 1; n <= most; n++) {
      const { fontSize, boxes } = layoutLabels(c.labelCell, texts.slice(0, n));
      expect(boxes).toHaveLength(n);
      expect(fontSize).toBeGreaterThanOrEqual(LABEL_FONT_MIN);
      const band = c.labelCell.sector!;
      for (const b of boxes) {
        const corners = [
          { x: b.x - b.width / 2, y: b.y - b.height / 2 },
          { x: b.x + b.width / 2, y: b.y - b.height / 2 },
          { x: b.x + b.width / 2, y: b.y + b.height / 2 },
          { x: b.x - b.width / 2, y: b.y + b.height / 2 },
        ];
        for (const p of corners) {
          expect(radius(p), `ring ${c.ring} house ${c.house} n ${n}`).toBeLessThanOrEqual(band.rOuter);
          expect(radius(p), `ring ${c.ring} house ${c.house} n ${n}`).toBeGreaterThanOrEqual(band.rInner);
        }
      }
      // Kutular birbirine değmez.
      for (let i = 0; i < n; i++)
        for (let j = i + 1; j < n; j++) {
          const [a, b] = [boxes[i], boxes[j]];
          const apart = Math.abs(a.x - b.x) >= (a.width + b.width) / 2 || Math.abs(a.y - b.y) >= (a.height + b.height) / 2;
          expect(apart, `ring ${c.ring} house ${c.house} n ${n}`).toBe(true);
        }
    }
  }
});
````

**Yeni dosya:** `app/ui/chart/SudarshanaSvg.test.tsx`

````tsx
import { fireEvent, render, screen } from "@testing-library/react";
import { SudarshanaSvg, type SudarshanaLabels, type SudarshanaRingData } from "./SudarshanaSvg";
import type { GrahaId } from "./types";

const labels: SudarshanaLabels = {
  title: "Sudarshana",
  graha: { Sun: "Su", Moon: "Mo", Mars: "Ma", Mercury: "Me", Jupiter: "Ju", Venus: "Ve", Saturn: "Sa", Rahu: "Ra", Ketu: "Ke" },
  lagna: "As",
};

/** Burç → grahalar; halka `first` burcundan başlar. */
const PLACED: Record<number, GrahaId[]> = { 0: ["Sun", "Mercury"], 1: ["Venus"], 3: ["Moon"], 4: ["Rahu"], 7: ["Mars"], 8: ["Jupiter"], 10: ["Saturn", "Ketu"] };
const ring = (first: number): SudarshanaRingData => ({
  houses: Array.from({ length: 12 }, (_, i) => ({ house: i + 1, sign: (first + i) % 12, grahas: PLACED[(first + i) % 12] ?? [] })),
});
/** Lagna Akrep (7), Ay Yengeç (3), Güneş Koç (0). */
const RINGS = [ring(7), ring(3), ring(0)];

test("draws three rings of twelve cells as one image with nothing to tab to", () => {
  const { container } = render(<SudarshanaSvg rings={RINGS} labels={labels} />);
  const svg = screen.getByRole("img", { name: "Sudarshana" });
  expect(svg).toHaveClass("chart", "sudarshana");
  expect(container.querySelectorAll("path.chart-cell")).toHaveLength(36);
  expect(container.querySelectorAll("[tabindex], [role='button']")).toHaveLength(0);
  expect([...svg.children].every((c) => c.getAttribute("aria-hidden") === "true")).toBe(true);
  // Her halkanın 1. evi dolguludur ve halkanın başlangıç burcunu taşır.
  const firsts = [...container.querySelectorAll('g[data-house="1"]:has(> .chart-cell--lagna)')];
  expect(firsts.map((g) => [g.getAttribute("data-ring"), g.getAttribute("data-sign")])).toEqual([
    ["0", "7"],
    ["1", "3"],
    ["2", "0"],
  ]);
  expect(container.querySelectorAll(".chart-cell--lagna")).toHaveLength(3);
  // Burç numarası 1…12 (Koç 1): lagna halkasının 1. evi Akrep (8).
  expect(container.querySelector('g[data-ring="0"][data-house="1"] .sudarshana-sign')).toHaveTextContent("8");
  expect(container.querySelector('g[data-ring="2"][data-house="12"] .sudarshana-sign')).toHaveTextContent("12");
});

test("every graha is written once in each ring, in the house counted from that ring's first sign", () => {
  const { container } = render(<SudarshanaSvg rings={RINGS} labels={labels} />);
  expect(container.querySelectorAll(".chart-planet")).toHaveLength(27);
  const house = (ringNo: number, graha: string) => container.querySelector(`.chart-planet[data-ring="${ringNo}"][data-graha="${graha}"]`)!.getAttribute("data-house");
  // Güneş Koç'ta: Akrep'ten 6., Yengeç'ten 10., Koç'tan 1. ev.
  expect([house(0, "Sun"), house(1, "Sun"), house(2, "Sun")]).toEqual(["6", "10", "1"]);
  // Ay Yengeç'te: 9., 1. ve 4. ev.
  expect([house(0, "Moon"), house(1, "Moon"), house(2, "Moon")]).toEqual(["9", "1", "4"]);
  expect(container.querySelector('.chart-planet[data-ring="1"][data-graha="Moon"] text')).toHaveTextContent("Mo");
  // Lagna işareti yalnızca iç halkanın 1. evindedir.
  const marks = container.querySelectorAll(".chart-mark");
  expect(marks).toHaveLength(1);
  expect(marks[0]).toHaveTextContent("As");
  expect([marks[0].getAttribute("data-ring"), marks[0].getAttribute("data-house")]).toEqual(["0", "1"]);
});

test("a ring without houses is not drawn", () => {
  const { container } = render(<SudarshanaSvg rings={[{ houses: null }, RINGS[1], RINGS[2]]} labels={labels} />);
  expect(container.querySelectorAll("path.chart-cell")).toHaveLength(24);
  expect(container.querySelector('[data-ring="0"]')).toBeNull();
  expect(container.querySelector(".chart-mark")).toBeNull();
  expect(container.querySelectorAll(".chart-planet")).toHaveLength(18);
  expect(container.querySelector(".sudarshana-hub")).not.toBeNull();
});

test("a sign, a house or a graha highlight marks its three cells or labels", () => {
  const { container, rerender } = render(<SudarshanaSvg rings={RINGS} labels={labels} highlight={{ type: "sign", sign: 10 }} />);
  const lit = () => [...container.querySelectorAll(".chart-cell--highlight")].map((p) => p.parentElement!).map((g) => [g.getAttribute("data-ring"), g.getAttribute("data-house")]);
  // Kova: Akrep'ten 4., Yengeç'ten 8., Koç'tan 11. ev.
  expect(lit()).toEqual([
    ["0", "4"],
    ["1", "8"],
    ["2", "11"],
  ]);
  // Ev vurgusu D1 evidir: lagna halkasının 4. evi Kova'dır; aynı üç hücre.
  rerender(<SudarshanaSvg rings={RINGS} labels={labels} highlight={{ type: "house", house: 4 }} />);
  expect(lit()).toEqual([
    ["0", "4"],
    ["1", "8"],
    ["2", "11"],
  ]);
  // Lagna halkası yoksa ev vurgusunun karşılığı yoktur.
  rerender(<SudarshanaSvg rings={[{ houses: null }, RINGS[1], RINGS[2]]} labels={labels} highlight={{ type: "house", house: 4 }} />);
  expect(lit()).toEqual([]);
  rerender(<SudarshanaSvg rings={RINGS} labels={labels} highlight={{ type: "graha", id: "Mars" }} />);
  expect(lit()).toEqual([]);
  expect([...container.querySelectorAll(".chart-planet--highlight")].map((g) => g.getAttribute("data-graha"))).toEqual(["Mars", "Mars", "Mars"]);
});

test("the mouse drives the shared highlight: hover, leave and click", () => {
  const onHover = vi.fn();
  const onPin = vi.fn();
  const { container } = render(<SudarshanaSvg rings={RINGS} labels={labels} onHover={onHover} onPin={onPin} />);
  const cell = container.querySelector('g[data-ring="1"][data-house="2"]')!;
  fireEvent.mouseEnter(cell);
  expect(onHover).toHaveBeenLastCalledWith({ type: "sign", sign: 4 });
  fireEvent.mouseLeave(cell);
  expect(onHover).toHaveBeenLastCalledWith(null);
  fireEvent.click(cell);
  expect(onPin).toHaveBeenLastCalledWith({ type: "sign", sign: 4 });
  const venus = container.querySelector('.chart-planet[data-ring="2"][data-graha="Venus"]')!;
  fireEvent.mouseEnter(venus);
  expect(onHover).toHaveBeenLastCalledWith({ type: "graha", id: "Venus" });
  fireEvent.click(venus);
  expect(onPin).toHaveBeenLastCalledWith({ type: "graha", id: "Venus" });
});
````

**Yeni dosya:** `app/ui/panels/SudarshanaPanel.test.tsx`

````tsx
import { fireEvent, screen, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeSudarshana: vi.fn() } }));

import { api, type Result } from "../api";
import type { SudarshanaChakra } from "../bindings";
import { setLanguage } from "../i18n";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { SudarshanaPanel } from "./SudarshanaPanel";

const ok = (chakra: SudarshanaChakra = fixture().bhava.sudarshana): Result<SudarshanaChakra> => ({ status: "ok", data: chakra });
const PANEL: Panel = { id: "su", kind: "sudarshana", x: 0, y: 0, w: 6, h: 24, settings: {} };

function renderPanel(opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  return renderWithStores(<SudarshanaPanel panel={PANEL} />, { opened });
}

const CAPTION = "Sudarshana chakra: on iki evin lagnadan, Ay'dan ve Güneş'ten burcu ve grahaları";
const table = () => screen.findByRole("table", { name: CAPTION });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);

beforeEach(() => {
  vi.mocked(api.computeSudarshana).mockReset();
  vi.mocked(api.computeSudarshana).mockImplementation(async () => ok());
});

test("draws the three rings and lists the same houses in a table", async () => {
  const { container } = renderPanel();
  const rows = within(await table()).getAllByRole("row");
  expect(api.computeSudarshana).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.computeSudarshana).mock.calls[0][0].time_known).toBe(true);
  expect(cells(rows[0])).toEqual(["Ev", "Lagnadan", "Ay'dan", "Güneş'ten"]);
  // Delhi fikstürü: lagna İkizler, Ay Terazi, Güneş Yay.
  expect(rows.slice(1).map(cells)).toEqual([
    ["1", "İki", "Ter Ay", "Yay Gü Me"],
    ["2", "Yen Ra", "Akr Ve", "Oğl Ke"],
    ["3", "Asl", "Yay Gü Me", "Kov Ma"],
    ["4", "Baş", "Oğl Ke", "Bal"],
    ["5", "Ter Ay", "Kov Ma", "Koç Jü Sa"],
    ["6", "Akr Ve", "Bal", "Boğ"],
    ["7", "Yay Gü Me", "Koç Jü Sa", "İki"],
    ["8", "Oğl Ke", "Boğ", "Yen Ra"],
    ["9", "Kov Ma", "İki", "Asl"],
    ["10", "Bal", "Yen Ra", "Baş"],
    ["11", "Koç Jü Sa", "Asl", "Ter Ay"],
    ["12", "Boğ", "Baş", "Akr Ve"],
  ]);
  // Çizim tablonun aynısıdır: 36 hücre, her graha üç kez, lagna işareti iç halkanın 1. evinde.
  expect(screen.getByRole("img", { name: "Sudarshana chakra: içten dışa lagna, Ay ve Güneş için üç halka" })).toBeInTheDocument();
  expect(container.querySelectorAll("svg.sudarshana path.chart-cell")).toHaveLength(36);
  expect(container.querySelectorAll("svg.sudarshana .chart-planet")).toHaveLength(27);
  for (const [ring, graha, house] of [
    ["0", "Sun", "7"],
    ["1", "Sun", "3"],
    ["2", "Sun", "1"],
    ["0", "Moon", "5"],
    ["1", "Moon", "1"],
    ["2", "Moon", "11"],
  ]) {
    expect(container.querySelector(`svg.sudarshana .chart-planet[data-ring="${ring}"][data-graha="${graha}"]`), `${ring} ${graha}`).toHaveAttribute("data-house", house);
  }
  expect(container.querySelector("svg.sudarshana .chart-mark")).toHaveTextContent("Lg");
  expect(screen.getByText(/^Üç halka içten dışa: lagna \(Lg\), Ay, Güneş\./)).toBeInTheDocument();
  expect(screen.getByText("Yöntem: sudarshana.v1")).toBeInTheDocument();
  expect(screen.queryByRole("note")).toBeNull();
});

test("the table is the keyboard path: signs and grahas are highlight buttons with full names", async () => {
  const { container } = renderPanel();
  const t = await table();
  const group = t.closest(".bala-scroll") as HTMLElement;
  expect(group).toHaveAttribute("tabindex", "0");
  expect(group).toHaveAccessibleName(CAPTION);
  // Çizimde sekme durağı yoktur.
  expect(container.querySelectorAll("svg.sudarshana [tabindex]")).toHaveLength(0);
  const row7 = within(t).getAllByRole("row")[7];
  // Kısaltma tam adıyla okunur.
  const sun = within(row7).getByRole("button", { name: "Güneş" });
  expect(sun).toHaveTextContent("Gü");
  fireEvent.click(sun);
  expect(sun).toHaveAttribute("aria-pressed", "true");
  // Güneş üç halkada ve tablonun üç hücresinde vurgulanır.
  expect([...container.querySelectorAll("svg.sudarshana .chart-planet--highlight")].map((g) => g.getAttribute("data-graha"))).toEqual(["Sun", "Sun", "Sun"]);
  expect([...t.querySelectorAll("td.is-highlighted")].map((td) => td.textContent)).toEqual(["Yay Gü Me", "Yay Gü Me", "Yay Gü Me"]);
  // Burç: Terazi üç halkada (5., 1. ve 11. ev).
  fireEvent.click(within(within(t).getAllByRole("row")[5]).getByRole("button", { name: "Ter" }));
  const lit = [...container.querySelectorAll("svg.sudarshana .chart-cell--highlight")].map((p) => p.parentElement!);
  expect(lit.map((g) => [g.getAttribute("data-ring"), g.getAttribute("data-house")])).toEqual([
    ["0", "5"],
    ["1", "1"],
    ["2", "11"],
  ]);
  expect([...t.querySelectorAll("td.is-highlighted")].map((td) => td.textContent)).toEqual(["Ter Ay", "Ter Ay", "Ter Ay"]);
});

test("hovering a cell of the drawing highlights the same sign in the table", async () => {
  const { container } = renderPanel();
  const t = await table();
  fireEvent.mouseEnter(container.querySelector('svg.sudarshana g[data-ring="2"][data-house="3"]')!);
  expect([...t.querySelectorAll("td.is-highlighted")].map((td) => td.textContent)).toEqual(["Kov Ma", "Kov Ma", "Kov Ma"]);
  fireEvent.mouseLeave(container.querySelector('svg.sudarshana g[data-ring="2"][data-house="3"]')!);
  expect(t.querySelector("td.is-highlighted")).toBeNull();
});

test("unknown birth time: no lagna ring, the lagna column is empty and the Moon is approximate", async () => {
  vi.mocked(api.computeSudarshana).mockImplementation(async () => ok(fixture().bhava.sudarshana_time_unknown));
  const { container } = renderPanel(unknownTimeOpened());
  const rows = within(await table()).getAllByRole("row");
  expect(vi.mocked(api.computeSudarshana).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByRole("note")).toHaveTextContent("Doğum saati bilinmiyor: lagna halkası çizilmiyor. Ay'ın burcu yaklaşıktır (≈).");
  expect(container.querySelectorAll("svg.sudarshana path.chart-cell")).toHaveLength(24);
  expect(container.querySelector('svg.sudarshana [data-ring="0"]')).toBeNull();
  expect(container.querySelector("svg.sudarshana .chart-mark")).toBeNull();
  expect(cells(rows[1])).toEqual(["1", "—", "Ter Ay", "Yay Gü Me"]);
  expect(rows.slice(1).every((r) => cells(r)[1] === "—")).toBe(true);
  // "≈" yalnızca Ay sütunundadır ve sözle de okunur.
  const moon = rows[0].querySelector('th[data-ring="Moon"]')!;
  expect(moon).toHaveTextContent("≈yaklaşık Ay'dan");
  expect(rows[0].querySelector('th[data-ring="Sun"]')).toHaveTextContent(/^Güneş'ten$/);
});

test("without an open chart nothing is requested", () => {
  renderPanel(null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeSudarshana).not.toHaveBeenCalled();
});

test("an error is shown in the panel and retry asks again", async () => {
  vi.mocked(api.computeSudarshana).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await table();
  expect(api.computeSudarshana).toHaveBeenCalledTimes(2);
});

test("the panel is named in three languages and the ring names follow the Rust order", async () => {
  expect(fixture().bhava.sudarshana_refs).toEqual(["Lagna", "Moon", "Sun"]);
  try {
    await setLanguage("en");
    renderPanel();
    const t = await screen.findByRole("table", { name: "Sudarshana chakra: sign and grahas of the twelve houses from the lagna, the Moon and the Sun" });
    expect(cells(within(t).getAllByRole("row")[0])).toEqual(["House", "From lagna", "From Moon", "From Sun"]);
    await setLanguage("fr");
    expect(await screen.findByRole("img", { name: "Sudarshana chakra : anneaux du lagna, de la Lune et du Soleil, du centre vers l’extérieur" })).toBeInTheDocument();
    expect(screen.getByText("Méthode : sudarshana.v1")).toBeInTheDocument();
  } finally {
    await setLanguage("tr");
  }
});
````

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/chart/sudarshana.test.ts ui/chart/SudarshanaSvg.test.tsx ui/panels/SudarshanaPanel.test.tsx ui/panels/panels.test.tsx ui/layout/model.test.ts 2>&1 | grep -E "Test Files|Tests |FAIL|Error" | head`
Expected: beş dosya da düşer (`./sudarshana`, `./SudarshanaSvg` ve `./SudarshanaPanel` çözülemez; katalogda `sudarshana` yok).

- [ ] **Step 3: Geometri ve çizim**

**Yeni dosya:** `app/ui/chart/sudarshana.ts`

````ts
// Sudarshana chakra için saf geometri (yol haritası §4.6): üç iç içe halka, her halkada on iki
// ev. Koordinatlar 100 × 100 kutudadır (svg viewBox). Aynı numaralı ev üç halkada da aynı
// açıdadır: 1. ev solda (daire haritasıyla aynı yön), evler saat yönünün tersine ilerler.
// Halka sırası içten dışa `SudarshanaRef::ALL` sırasıdır (lagna, Ay, Güneş).
import type { ChartCell, Point } from "./geometry";

/** Halka sayısı (Rust `SudarshanaRef::ALL`). */
export const SUDARSHANA_RING_COUNT = 3;
/**
 * Halka sınırları: göbek, sonra üç halkanın dış yarıçapı. Değerler, en dar açıdaki hücrede bile
 * iç halkaya altı, orta ve dış halkaya dört etiket sığacak biçimde seçilmiştir
 * (`sudarshana.test.ts`); daha çok graha en küçük yazı boyutunda, taşarak yazılır.
 */
export const SUDARSHANA_RADII = [5, 22, 36, 49] as const;
/** Burç numarasının yazıldığı iç şeridin kalınlığı; etiketler şeridin dışına yerleşir. */
export const SUDARSHANA_SIGN_BAND = 2.6;
/** Burç numarasının yazı boyutu. */
export const SUDARSHANA_SIGN_FONT = 2.2;
const ARC_STEPS = 8;

function polar(r: number, deg: number): Point {
  const rad = (deg * Math.PI) / 180;
  // Matematiksel açı (saat yönünün tersi, y yukarı) → ekran (y aşağı).
  return { x: 50 + r * Math.cos(rad), y: 50 - r * Math.sin(rad) };
}

/** 30°'lik halka diliminin çokgeni: dış yay ileri, iç yay geri. */
function sectorPoints(rInner: number, rOuter: number, startDeg: number): Point[] {
  const outer: Point[] = [];
  const inner: Point[] = [];
  for (let i = 0; i <= ARC_STEPS; i++) {
    outer.push(polar(rOuter, startDeg + (30 * i) / ARC_STEPS));
    inner.push(polar(rInner, startDeg + 30 - (30 * i) / ARC_STEPS));
  }
  return [...outer, ...inner];
}

export interface SudarshanaCell {
  /** 0 = iç halka (lagna), 1 = orta (Ay), 2 = dış (Güneş). */
  ring: number;
  /** 1…12. */
  house: number;
  /** Hücrenin tam çokgeni (çizim). */
  points: Point[];
  /** Burç numarasının yeri: iç şeridin ortası. */
  signAnchor: Point;
  /** Etiket alanı: iç şerit dışındaki dilim; `layoutLabels` bununla çağrılır. */
  labelCell: ChartCell;
}

/** Bir halkanın bir evi. `ring` 0…2, `house` 1…12. */
export function sudarshanaCell(ring: number, house: number): SudarshanaCell {
  const rInner = SUDARSHANA_RADII[ring];
  const rOuter = SUDARSHANA_RADII[ring + 1];
  const centre = 180 + (house - 1) * 30;
  const start = centre - 15;
  const rLabel = rInner + SUDARSHANA_SIGN_BAND;
  return {
    ring,
    house,
    points: sectorPoints(rInner, rOuter, start),
    signAnchor: polar(rInner + SUDARSHANA_SIGN_BAND / 2 + 0.2, centre),
    labelCell: {
      house,
      sign: 0,
      points: sectorPoints(rLabel, rOuter, start),
      planetAnchor: polar((rLabel + rOuter) / 2, centre),
      signAnchor: null,
      lagnaAnchor: null,
      sector: { rInner: rLabel, rOuter, startDeg: start, endDeg: start + 30 },
    },
  };
}

/** 36 hücre: halka sırasıyla (içten dışa), her halkada 1. evden 12. eve. */
export function sudarshanaCells(): SudarshanaCell[] {
  const cells: SudarshanaCell[] = [];
  for (let ring = 0; ring < SUDARSHANA_RING_COUNT; ring++) for (let house = 1; house <= 12; house++) cells.push(sudarshanaCell(ring, house));
  return cells;
}
````

**Yeni dosya:** `app/ui/chart/SudarshanaSvg.tsx`

````tsx
// Sudarshana chakra çizimi: üç iç içe halka (içten dışa lagna, Ay, Güneş), her halkada on iki
// ev. Yalnızca belirteç sınıfları kullanılır; renk anlam taşımaz (C9). Her halkanın 1. evi
// dolguludur; lagna halkasının 1. evinde lagna işareti yazar (Ay halkasının 1. evinde Ay'ın,
// Güneş halkasının 1. evinde Güneş'in kendisi durur). Çizim tek bir resimdir (`role="img"`): 36 hücre ve 27 etiket
// sekme durağı olmaz; klavye ve ekran okuyucu karşılığı panelin tablosudur. Fare ile üzerine
// gelmek ve tıklamak ortak vurguyu sürer (UI spec §3.4). Halkası olmayan (`houses: null`: doğum
// saati bilinmiyor) halka çizilmez.
import { useMemo } from "react";
import type { HighlightTarget } from "../highlight/target";
import "./chart.css";
import { layoutLabels, pathOf, VIEWBOX } from "./geometry";
import { SUDARSHANA_RADII, SUDARSHANA_SIGN_FONT, sudarshanaCell } from "./sudarshana";
import type { GrahaId } from "./types";

export interface SudarshanaRingData {
  /** On iki ev, 1. evden başlayarak; `null`: halka çizilmez. */
  houses: { house: number; sign: number; grahas: GrahaId[] }[] | null;
}

export interface SudarshanaLabels {
  /** Erişilebilir ad (svg `role="img"` + `aria-label`). */
  title: string;
  /** Graha kısaltmaları. */
  graha: Record<GrahaId, string>;
  /** Lagna halkasının 1. evindeki işaret. */
  lagna: string;
}

export interface SudarshanaSvgProps {
  /** Üç halka, içten dışa. */
  rings: readonly SudarshanaRingData[];
  labels: SudarshanaLabels;
  highlight?: HighlightTarget | null;
  onHover?: (target: HighlightTarget | null) => void;
  onPin?: (target: HighlightTarget) => void;
}

export function SudarshanaSvg({ rings, labels, highlight = null, onHover, onPin }: SudarshanaSvgProps) {
  const { graha, lagna } = labels;
  const textKey = JSON.stringify([graha, lagna]);
  const placed = useMemo(
    () =>
      rings.flatMap((ring, k) =>
        (ring.houses ?? []).map((h) => {
          const cell = sudarshanaCell(k, h.house);
          // Lagna işareti yalnızca iç halkanın 1. evindedir ve grahalardan sonra yerleşir.
          const mark = k === 0 && h.house === 1 ? lagna : null;
          const texts = [...h.grahas.map((g) => graha[g]), ...(mark !== null ? [mark] : [])];
          return { ring: k, house: h.house, sign: h.sign, grahas: h.grahas, mark, cell, layout: layoutLabels(cell.labelCell, texts) };
        }),
      ),
    // textKey, graha ve lagna'yı kapsar.
    [rings, textKey],
  );
  // Ev vurgusu D1 evidir: lagna halkasının o evindeki burç (Rust'tan gelen veri; hesap değil).
  const activeSign = highlight?.type === "sign" ? highlight.sign : highlight?.type === "house" ? (rings[0]?.houses?.[highlight.house - 1]?.sign ?? null) : null;
  const activeGraha = highlight?.type === "graha" ? highlight.id : null;
  const pointer = (target: HighlightTarget) => ({
    onMouseEnter: () => onHover?.(target),
    onMouseLeave: () => onHover?.(null),
    onClick: () => onPin?.(target),
  });

  return (
    <svg className="chart sudarshana" viewBox={`0 0 ${VIEWBOX} ${VIEWBOX}`} role="img" aria-label={labels.title} xmlns="http://www.w3.org/2000/svg">
      <circle className="sudarshana-hub" cx={VIEWBOX / 2} cy={VIEWBOX / 2} r={SUDARSHANA_RADII[0]} aria-hidden="true" />
      {placed.map(({ ring, house, sign, cell }) => {
        const on = activeSign === sign;
        const classes = ["chart-cell"];
        if (house === 1) classes.push("chart-cell--lagna");
        if (on) classes.push("chart-cell--highlight");
        return (
          <g
            key={`${ring}-${house}`}
            className={`chart-house${on ? " chart-house--highlight" : ""}`}
            data-ring={ring}
            data-house={house}
            data-sign={sign}
            aria-hidden="true"
            {...pointer({ type: "sign", sign })}
          >
            <path className={classes.join(" ")} d={pathOf(cell.points)} />
            <text
              className="sudarshana-sign"
              x={cell.signAnchor.x}
              y={cell.signAnchor.y}
              textAnchor="middle"
              dominantBaseline="central"
              style={{ fontSize: `${SUDARSHANA_SIGN_FONT}px` }}
            >
              {sign + 1}
            </text>
          </g>
        );
      })}
      {placed.map(({ ring, house, grahas, layout }) =>
        grahas.map((g, i) => {
          const { x, y, width, height } = layout.boxes[i];
          return (
            <g
              key={`${ring}-${g}`}
              className={`chart-planet${activeGraha === g ? " chart-planet--highlight" : ""}`}
              data-ring={ring}
              data-graha={g}
              data-house={house}
              aria-hidden="true"
              {...pointer({ type: "graha", id: g })}
            >
              <rect className="chart-planet-bg" x={x - width / 2} y={y - height / 2} width={width} height={height} rx={layout.fontSize * 0.25} />
              <text className="chart-text" x={x} y={y} textAnchor="middle" dominantBaseline="central" style={{ fontSize: `${layout.fontSize}px` }}>
                {labels.graha[g]}
              </text>
            </g>
          );
        }),
      )}
      {placed.map(({ ring, house, grahas, mark, layout }) => {
        if (mark === null) return null;
        const { x, y } = layout.boxes[grahas.length];
        return (
          <text
            key={`mark-${ring}`}
            className="chart-mark"
            data-mark={mark}
            data-ring={ring}
            data-house={house}
            x={x}
            y={y}
            textAnchor="middle"
            dominantBaseline="central"
            style={{ fontSize: `${layout.fontSize}px` }}
            aria-hidden="true"
          >
            {mark}
          </text>
        );
      })}
    </svg>
  );
}
````

````diff
diff --git a/app/ui/chart/chart.css b/app/ui/chart/chart.css
index 0e12c9d..4878061 100644
--- a/app/ui/chart/chart.css
+++ b/app/ui/chart/chart.css
@@ -82,3 +82,15 @@
   fill: var(--text-2);
   pointer-events: none;
 }
+
+/* Sudarshana chakra: burç numarası halkanın iç şeridinde, küçük ve ikincil tonda. */
+.sudarshana-sign {
+  fill: var(--text-2);
+  font-variant-numeric: tabular-nums;
+  pointer-events: none;
+}
+.sudarshana-hub {
+  fill: var(--panel);
+  stroke: var(--ink);
+  stroke-width: 0.4;
+}
````

```bash
pnpm --dir app exec vitest run ui/chart/sudarshana.test.ts ui/chart/SudarshanaSvg.test.tsx 2>&1 | grep -E "Test Files|Tests |FAIL"
```
Expected: `2 passed (2)`, `9 passed (9)`. "labels fit every cell" testi düşerse yarıçaplar ya da şerit kalınlığı plandakinden farklı yazılmıştır; test gevşetilmez.

- [ ] **Step 4: Kanca, panel, katalog, stil ve metinler**

**Yeni dosya:** `app/ui/modules/useSudarshana.ts`

````ts
// Sudarshana chakrayı (`compute_sudarshana`) isteyen kanca. Sonuç saklanmaz ve arayüzde hesap
// yoktur (K1, C4); panel açıkken istenir. Doğum saati bilinmiyorsa sonuç yine döner: lagna
// halkasının evleri `null`, sonuç `approximate` işaretlidir (K2).
import { api } from "../api";
import type { OpenedClient, SudarshanaChakra } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

export function useSudarshana(opened: OpenedClient | null): ModuleState<SudarshanaChakra> {
  return useModule(opened !== null ? inputKey(opened) : null, () => api.computeSudarshana(chartInput(opened as OpenedClient)));
}
````

**Yeni dosya:** `app/ui/panels/SudarshanaPanel.tsx`

````tsx
// Sudarshana chakra paneli (yol haritası §4.6): lagna, Ay ve Güneş'i birinci ev sayan üç halka
// ve çizimin tablo karşılığı. Değerler `compute_sudarshana`'dan gelir; burada hesap yoktur (C4).
// Çizim tek bir resimdir; klavye ve ekran okuyucu tabloyu kullanır: her hücrede burç ve grahalar
// ortak vurgunun düğmeleridir (UI spec §3.4). Doğum saati bilinmiyorsa lagna halkası Rust'tan boş
// gelir (K2): halka çizilmez, sütunu çizgi taşır ve Ay'ın burcu yaklaşık (≈) işaretlenir.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { SudarshanaChakra } from "../bindings";
import { signIndex } from "../chart/adapter";
import { SudarshanaSvg, type SudarshanaRingData } from "../chart/SudarshanaSvg";
import { isGrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { useSudarshana } from "../modules/useSudarshana";
import { useSession } from "../session/store";
import { Approx } from "./Approx";
import { BalaScroll } from "./bala";
import { grahaAbbreviations } from "./labels";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

const DASH = "—";

export function SudarshanaPanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const module = useSudarshana(opened);
  const phase = usePanelLoadPhase(module.loading);
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (module.data === null) return phase === "skeleton" ? <PanelSkeleton /> : null;
  return <Chakra chakra={module.data} />;
}

function Chakra({ chakra }: { chakra: SudarshanaChakra }) {
  const { t } = useTranslation();
  const highlight = useHighlight();
  const { active } = highlight;
  const captionId = useId();
  // Çizim verisi: burç dizini ve graha kimlikleri (biçim dönüşümü; hesap değil).
  const rings: SudarshanaRingData[] = chakra.rings.map((r) => ({
    houses: r.houses === null ? null : r.houses.map((h) => ({ house: h.house, sign: signIndex(h.sign), grahas: h.grahas.filter(isGrahaId) })),
  }));
  // Ev vurgusu D1 evidir: lagna halkasının o evindeki burç.
  const activeSign = active?.type === "sign" ? active.sign : active?.type === "house" ? (rings[0]?.houses?.[active.house - 1]?.sign ?? null) : null;
  const activeGraha = active?.type === "graha" ? active.id : null;
  return (
    <div className="module-panel">
      {chakra.approximate && (
        <p className="panel-notice" role="note">
          {t("sudarshana.timeUnknown")}
        </p>
      )}
      <div className="sudarshana-figure">
        <SudarshanaSvg
          rings={rings}
          labels={{ title: t("sudarshana.title"), graha: grahaAbbreviations(t), lagna: t("chart.lagna") }}
          highlight={active}
          onHover={highlight.hover}
          onPin={highlight.togglePin}
        />
      </div>
      <p className="panel-message">{t("sudarshana.legend", { lagna: t("chart.lagna") })}</p>
      <BalaScroll labelledBy={captionId}>
        <table className="module-table">
          <caption id={captionId} className="visually-hidden">
            {t("sudarshana.caption")}
          </caption>
          <thead>
            <tr>
              <th scope="col">{t("sudarshana.house")}</th>
              {chakra.rings.map((r) => (
                <th key={r.reference} scope="col" data-ring={r.reference}>
                  <Approx on={chakra.approximate && r.reference === "Moon"} label={t("table.approx")}>
                    {t(`sudarshana.ring.${r.reference}`)}
                  </Approx>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {Array.from({ length: 12 }, (_, i) => (
              <tr key={i} data-house={i + 1}>
                <th scope="row">{i + 1}</th>
                {rings.map((ring, k) => {
                  const h = ring.houses?.[i];
                  if (h === undefined) return <td key={chakra.rings[k].reference}>{DASH}</td>;
                  const on = activeSign === h.sign || (activeGraha !== null && h.grahas.includes(activeGraha));
                  return (
                    <td key={chakra.rings[k].reference} className={on ? "is-highlighted" : undefined}>
                      <PinButton target={{ type: "sign", sign: h.sign }}>{t(`sign.${h.sign}`)}</PinButton>
                      {h.grahas.map((g) => (
                        <span key={g}>
                          {" "}
                          <PinButton target={{ type: "graha", id: g }} label={t(`grahaName.${g}`)}>
                            {t(`graha.${g}`)}
                          </PinButton>
                        </span>
                      ))}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </BalaScroll>
      <p className="panel-message">{t("sudarshana.method", { method: chakra.method })}</p>
    </div>
  );
}
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index b362ae7..2086607 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -79,6 +79,7 @@ export const en: Messages = {
     arudha: "Arudha padas",
     argala: "Argala",
     bhava_table: "Bhava table",
+    sudarshana: "Sudarshana chakra",
     shadbala: "Shadbala",
     bhava_bala: "Bhava bala",
     yogas: "Yogas",
@@ -814,6 +815,15 @@ export const en: Messages = {
     method: "Method: {{method}}",
     timeUnknown: "Birth time unknown: the bhavas depend on the lagna and are not shown.",
   },
+  sudarshana: {
+    title: "Sudarshana chakra: rings of the lagna, the Moon and the Sun from the centre outwards",
+    caption: "Sudarshana chakra: sign and grahas of the twelve houses from the lagna, the Moon and the Sun",
+    house: "House",
+    ring: { Lagna: "From lagna", Moon: "From Moon", Sun: "From Sun" },
+    legend: "Rings from the centre outwards: lagna ({{lagna}}), Moon, Sun. House 1 is on the left and filled; houses run counter-clockwise and a house has the same angle in all three rings. The number in a cell is the number of the sign.",
+    method: "Method: {{method}}",
+    timeUnknown: "Birth time unknown: the lagna ring is not drawn. The sign of the Moon is approximate (≈).",
+  },
   arudha: {
     caption: "Arudha padas: the pada of each of the twelve houses, its sign and the lord of the house",
     pada: "Pada",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index f78b568..9faee41 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -79,6 +79,7 @@ export const fr: Messages = {
     arudha: "Arudha padas",
     argala: "Argala",
     bhava_table: "Table des bhavas",
+    sudarshana: "Sudarshana chakra",
     shadbala: "Shadbala",
     bhava_bala: "Bhava bala",
     yogas: "Yogas",
@@ -814,6 +815,15 @@ export const fr: Messages = {
     method: "Méthode : {{method}}",
     timeUnknown: "Heure de naissance inconnue : les bhavas dépendent du lagna et ne sont pas affichés.",
   },
+  sudarshana: {
+    title: "Sudarshana chakra : anneaux du lagna, de la Lune et du Soleil, du centre vers l’extérieur",
+    caption: "Sudarshana chakra : signe et grahas des douze maisons depuis le lagna, la Lune et le Soleil",
+    house: "Maison",
+    ring: { Lagna: "Depuis le lagna", Moon: "Depuis la Lune", Sun: "Depuis le Soleil" },
+    legend: "Anneaux du centre vers l’extérieur : lagna ({{lagna}}), Lune, Soleil. La maison 1 est à gauche et remplie ; les maisons tournent dans le sens antihoraire et une maison a le même angle dans les trois anneaux. Le nombre dans une case est le numéro du signe.",
+    method: "Méthode : {{method}}",
+    timeUnknown: "Heure de naissance inconnue : l’anneau du lagna n’est pas tracé. Le signe de la Lune est approximatif (≈).",
+  },
   arudha: {
     caption: "Arudha padas : le pada de chacune des douze maisons, son signe et le maître de la maison",
     pada: "Pada",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 6a4b78a..f1cb08d 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -78,6 +78,7 @@ export const tr = {
     arudha: "Arudha padalar",
     argala: "Argala",
     bhava_table: "Bhava tablosu",
+    sudarshana: "Sudarshana chakra",
     shadbala: "Shadbala",
     bhava_bala: "Bhava bala",
     yogas: "Yogalar",
@@ -813,6 +814,15 @@ export const tr = {
     method: "Yöntem: {{method}}",
     timeUnknown: "Doğum saati bilinmiyor: bhavalar lagnaya bağlı olduğu için gösterilmiyor.",
   },
+  sudarshana: {
+    title: "Sudarshana chakra: içten dışa lagna, Ay ve Güneş için üç halka",
+    caption: "Sudarshana chakra: on iki evin lagnadan, Ay'dan ve Güneş'ten burcu ve grahaları",
+    house: "Ev",
+    ring: { Lagna: "Lagnadan", Moon: "Ay'dan", Sun: "Güneş'ten" },
+    legend: "Üç halka içten dışa: lagna ({{lagna}}), Ay, Güneş. 1. ev solda ve dolguludur; evler saat yönünün tersine ilerler ve aynı numaralı ev üç halkada aynı açıdadır. Hücredeki sayı burcun numarasıdır.",
+    method: "Yöntem: {{method}}",
+    timeUnknown: "Doğum saati bilinmiyor: lagna halkası çizilmiyor. Ay'ın burcu yaklaşıktır (≈).",
+  },
   arudha: {
     caption: "Arudha padalar: on iki evin padası, burcu ve evin lordu",
     pada: "Pada",
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index 8ddd706..fba32c2 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -19,6 +19,7 @@ export const PANEL_KINDS = [
   "shadbala",
   "bhava_bala",
   "bhava_table",
+  "sudarshana",
   "yogas",
   "compat",
   "muhurta_day",
@@ -45,6 +46,8 @@ export const MODULE_PANELS = [
   { kind: "bhava_bala", w: 6, h: 14 },
   // Bhava tablosu (yol haritası Plan 25).
   { kind: "bhava_table", w: 7, h: 20 },
+  // Sudarshana chakra (yol haritası Plan 25).
+  { kind: "sudarshana", w: 6, h: 24 },
   // Yogalar (Plan 18).
   { kind: "yogas", w: 7, h: 14 },
   // Uyum (Plan 19).
diff --git a/app/ui/panels/panels.css b/app/ui/panels/panels.css
index a4d89e7..6962c3a 100644
--- a/app/ui/panels/panels.css
+++ b/app/ui/panels/panels.css
@@ -430,3 +430,11 @@
   max-height: 24em;
   overflow: auto;
 }
+
+/* Sudarshana chakra: çizim panelin genişliğine sığan bir karedir; tablo altında kayar. */
+.sudarshana-figure {
+  width: 100%;
+  max-width: 34rem;
+  aspect-ratio: 1;
+  margin: 0 auto;
+}
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index 6adb861..4708b81 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -22,6 +22,7 @@ import { PanchangPanel } from "./PanchangPanel";
 import { EventSearchPanel } from "./EventSearchPanel";
 import { PrashnaFactsPanel } from "./PrashnaFactsPanel";
 import { ShadbalaPanel } from "./ShadbalaPanel";
+import { SudarshanaPanel } from "./SudarshanaPanel";
 import type { PanelDefinition } from "./types";
 import { UnavailablePanel } from "./UnavailablePanel";
 import { YogasPanel } from "./YogasPanel";
@@ -51,6 +52,8 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   bhava_bala: { titleKey: "panel.bhava_bala", component: BhavaBalaPanel },
   // Bhava tablosu (yol haritası Plan 25).
   bhava_table: { titleKey: "panel.bhava_table", component: BhavaTablePanel },
+  // Sudarshana chakra (yol haritası Plan 25).
+  sudarshana: { titleKey: "panel.sudarshana", component: SudarshanaPanel },
   // Yogalar (Plan 18).
   yogas: { titleKey: "panel.yogas", component: YogasPanel },
   // Uyum (Plan 19).
````

Türkçe metinlerde "halka" sözcüğü çoğul yazılmaz (D20): "üç halka", "her halkada".

- [ ] **Step 5: Testler ve kapılar**

```bash
pnpm --dir app exec vitest run ui/chart/sudarshana.test.ts ui/chart/SudarshanaSvg.test.tsx ui/panels/SudarshanaPanel.test.tsx 2>&1 | grep -E "Test Files|Tests |FAIL"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
git diff -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
git diff -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
```
Expected: `3 passed (3)`, `16 passed (16)`; Vitest `92 passed (92)`, `930 passed (930)`; `✓ built`; iki tarama çıktısız (yeni stil yalnızca token kullanır: C9).

- [ ] **Step 6: Commit**

```bash
git add app/ui/chart/sudarshana.ts app/ui/chart/sudarshana.test.ts app/ui/chart/SudarshanaSvg.tsx app/ui/chart/SudarshanaSvg.test.tsx app/ui/chart/chart.css app/ui/modules/useSudarshana.ts app/ui/panels/SudarshanaPanel.tsx app/ui/panels/SudarshanaPanel.test.tsx app/ui/panels/panels.css app/ui/layout/model.ts app/ui/layout/model.test.ts app/ui/panels/registry.ts app/ui/panels/panels.test.tsx app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): sudarshana chakra panel with three rings drawn as one image and a table that drives the shared highlight" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: on altı dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 7: Uçtan uca akışlar ve görsel temeller (bhava tablosu ve chalit; Sudarshana; saat bilinmeyen harita)

Yol haritası §4.5 ve §4.6 "görsel temeller"; K12; D16, D17. Taklit komutlar Task 3'te eklenmiştir. Akışlar WebKit'te, Vite geliştirme sunucusunda (port 1421) koşar; gerçek uygulama çalıştırılmaz.

**Files:**
- Modify: `app/e2e/desk.spec.ts`, `app/e2e/visual.spec.ts`
- Create (komutla): `app/e2e/visual.spec.ts-snapshots/chalit-north-visual-darwin.png`, `chalit-south-visual-darwin.png`, `chalit-circle-visual-darwin.png`, `sudarshana-light-visual-darwin.png`, `sudarshana-dark-visual-darwin.png`
- Test: e2e +3 (27 → 30); görsel +5 (10 → 15)

**Interfaces:**
- Consumes: Task 3'ün `mockIpc(page, { placidus })` ve iki taklidi; Task 4–6'nın panelleri ve metinleri.
- Produces: üç akış ve beş görsel temel.

- [ ] **Step 1: Üç akış ve beş görsel test**

````diff
diff --git a/app/e2e/desk.spec.ts b/app/e2e/desk.spec.ts
index a42e99c..ffd211d 100644
--- a/app/e2e/desk.spec.ts
+++ b/app/e2e/desk.spec.ts
@@ -1239,3 +1239,112 @@ test("event search: the panel from the palette, the event list in the chart's ti
   await expect(panel).toHaveCount(0);
   expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
 });
+
+// Bhava tablosu, chalit görünümü ve Sudarshana chakra (yol haritası Plan 25).
+test("bhava table and chalit: a palette panel with the cusps of a Placidus chart; the chart panel redraws the grahas by bhava", async ({ page }) => {
+  await mockIpc(page, { session: true, placidus: true });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  type BhavaCall = { input: { jd_ut: number; time_known: boolean; settings: { house_system: string } } };
+  const calls = async () => (await ipcCalls(page, "compute_bhava_table")) as BhavaCall[];
+  // Tablo panel açılmadan istenmez (K1).
+  expect(await calls()).toHaveLength(0);
+  await page.keyboard.press("Meta+k");
+  const palette = page.getByRole("dialog", { name: "Komut paleti" });
+  await palette.getByRole("combobox").fill("bhava tablosu");
+  await palette.getByRole("option", { name: /Bhava tablosu panelini aç\/kapat/ }).click();
+  await expect(page.getByRole("dialog")).toHaveCount(0);
+  const panel = page.getByRole("region", { name: "Bhava tablosu" });
+  await expect(panel).toBeFocused();
+  const bhavas = panel.getByRole("table", { name: /^Bhava tablosu: on iki bhavanın başlangıç noktası/ });
+  await expect(bhavas.locator('tbody tr[data-house="1"]').locator("th, td")).toHaveText(["1", "İki 16°20′", "Ardra", "3", "Merkür", "Rahu", "22°37′"]);
+  expect((await calls()).map((c) => [c.input.jd_ut, c.input.time_known, c.input.settings.house_system])).toEqual([[2451545, true, "Placidus"]]);
+  // Burç evinden başka bir bhavadaki graha `*` taşır; işaret sözle de okunur.
+  const grahas = panel.getByRole("table", { name: /^Grahaların bhavası/ });
+  await expect(grahas.locator('tbody tr[data-graha="Mercury"]').locator("th, td")).toHaveText(["Merkür", "7", "6* burç evinden farklı", "24°07′", "—"]);
+  await expect(grahas.locator('tbody tr[data-graha="Sun"]').locator("th, td")).toHaveText(["Güneş", "7", "7", "0°10′", "—"]);
+  await expect(panel.getByText("* Graha, burcunun evinden başka bir bhavadadır.")).toBeVisible();
+  // Placidus'ta orta yoktur: nokta ayarı bunu söyler ve yeni istek yapmaz.
+  await panel.getByRole("combobox", { name: "Nokta" }).selectOption("mid");
+  await expect(panel.getByRole("note")).toHaveText("Placidus'ta bhava ortası tanımlanmaz: nokta bhavanın başlangıcıdır.");
+  await panel.getByRole("combobox", { name: "Nokta" }).selectOption("end");
+  await expect(panel.getByRole("table", { name: /^Bhava tablosu: on iki bhavanın bitiş noktası/ }).locator('tbody tr[data-house="1"] td').first()).toHaveText("Yen 8°57′");
+  expect(await calls()).toHaveLength(1);
+
+  // Harita panelinde chalit görünümü: grahalar bhavalarına göre yerleşir; istek yapılmaz.
+  const chart = page.getByRole("region", { name: "Harita D1", exact: true });
+  const mercury = chart.locator('svg.chart [data-graha="Mercury"]');
+  await expect(mercury).toHaveAttribute("data-house", "7");
+  await chart.getByRole("checkbox", { name: "Bhava chalit" }).check();
+  await expect(chart.getByRole("group", { name: "Bhava chalit haritası" })).toBeVisible();
+  await expect(mercury).toHaveAttribute("data-house", "6");
+  await expect(mercury).toHaveAttribute("data-shifted", "true");
+  await expect(chart.locator("svg.chart [data-shifted]")).toHaveCount(3);
+  await expect(chart.locator('svg.chart [data-graha="Sun"]')).toHaveAttribute("data-house", "7");
+  await expect(chart.getByText(/^Hücreler bhavadır \(Placidus\)/)).toBeVisible();
+  // Ortak vurgu: tablodaki 6. bhava, haritada o hücreyi ve tablodaki grahasını vurgular.
+  await panel.getByRole("table", { name: /^Bhava tablosu:/ }).getByRole("button", { name: "6. bhava" }).hover();
+  await expect(chart.locator('svg.chart g.chart-house[data-house="6"]')).toHaveClass(/chart-house--highlight/);
+  await expect(grahas.locator('tbody tr[data-graha="Mercury"]')).toHaveClass(/is-highlighted/);
+  expect(await calls()).toHaveLength(1);
+  // Görünüm ve nokta ayarı düzenle birlikte kaydedilir.
+  await expect
+    .poll(async () => {
+      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { id: string; kind: string; settings: Record<string, unknown> }[] }[] } } } | undefined;
+      const layout = saved?.viewState.panel_layout;
+      const panels = layout?.layouts.find((l) => l.id === layout.active_layout_id)?.panels ?? [];
+      return [panels.find((p) => p.id === "d1")?.settings.chalit, panels.find((p) => p.kind === "bhava_table")?.settings];
+    })
+    .toEqual([true, { point: "end" }]);
+});
+
+test("sudarshana: a palette panel with three rings and their table; the table drives the shared highlight", async ({ page }) => {
+  await mockIpc(page, { session: true });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  expect(await ipcCalls(page, "compute_sudarshana")).toHaveLength(0);
+  await page.keyboard.press("Meta+k");
+  const palette = page.getByRole("dialog", { name: "Komut paleti" });
+  await palette.getByRole("combobox").fill("sudarshana");
+  await palette.getByRole("option", { name: /Sudarshana chakra panelini aç\/kapat/ }).click();
+  await expect(page.getByRole("dialog")).toHaveCount(0);
+  const panel = page.getByRole("region", { name: "Sudarshana chakra" });
+  await expect(panel).toBeFocused();
+  const svg = panel.getByRole("img", { name: "Sudarshana chakra: içten dışa lagna, Ay ve Güneş için üç halka" });
+  await expect(svg).toBeVisible();
+  await expect(svg.locator("path.chart-cell")).toHaveCount(36);
+  await expect(svg.locator(".chart-planet")).toHaveCount(27);
+  await expect(svg.locator(".chart-mark")).toHaveText("Lg");
+  expect(((await ipcCalls(page, "compute_sudarshana")) as { input: { time_known: boolean } }[]).map((c) => c.input.time_known)).toEqual([true]);
+  // Tablo çizimin karşılığıdır: 7. ev lagnadan Yay (Güneş, Merkür), Ay'dan Koç, Güneş'ten İkizler.
+  const table = panel.getByRole("table", { name: /^Sudarshana chakra: on iki evin/ });
+  await expect(table.locator('tbody tr[data-house="7"]').locator("th, td")).toHaveText(["7", "Yay Gü Me", "Koç Jü Sa", "İki"]);
+  // Klavyeyle: tablodaki Güneş düğmesi sabitlenir; üç halkada ve D1 haritasında vurgulanır.
+  const sun = table.locator('tbody tr[data-house="7"]').getByRole("button", { name: "Güneş" });
+  await sun.focus();
+  await page.keyboard.press("Enter");
+  await expect(sun).toHaveAttribute("aria-pressed", "true");
+  await expect(svg.locator(".chart-planet--highlight")).toHaveCount(3);
+  await expect(page.getByRole("region", { name: "Harita D1", exact: true }).locator('svg.chart [data-graha="Sun"]')).toHaveClass(/chart-planet--highlight/);
+  await page.keyboard.press("Escape");
+  await expect(svg.locator(".chart-planet--highlight")).toHaveCount(0);
+  // Çizimde sekme durağı yoktur.
+  await expect(svg.locator("[tabindex]")).toHaveCount(0);
+});
+
+test("sudarshana with an unknown birth time: the lagna ring is not drawn and the Moon is marked approximate", async ({ page }) => {
+  await mockIpc(page, { session: true, unknownTime: true });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  await page.keyboard.press("Meta+k");
+  const palette = page.getByRole("dialog", { name: "Komut paleti" });
+  await palette.getByRole("combobox").fill("sudarshana");
+  await palette.getByRole("option", { name: /Sudarshana chakra panelini aç\/kapat/ }).click();
+  const panel = page.getByRole("region", { name: "Sudarshana chakra" });
+  await expect(panel.getByRole("note")).toHaveText("Doğum saati bilinmiyor: lagna halkası çizilmiyor. Ay'ın burcu yaklaşıktır (≈).");
+  const svg = panel.getByRole("img");
+  await expect(svg.locator("path.chart-cell")).toHaveCount(24);
+  await expect(svg.locator('[data-ring="0"]')).toHaveCount(0);
+  await expect(panel.locator('tbody tr[data-house="1"]').locator("th, td")).toHaveText(["1", "—", "Ter Ay", "Yay Gü Me"]);
+  expect(((await ipcCalls(page, "compute_sudarshana")) as { input: { time_known: boolean } }[]).map((c) => c.input.time_known)).toEqual([false]);
+});
diff --git a/app/e2e/visual.spec.ts b/app/e2e/visual.spec.ts
index 0ef2618..65abe07 100644
--- a/app/e2e/visual.spec.ts
+++ b/app/e2e/visual.spec.ts
@@ -36,3 +36,41 @@ for (const sheet of ["cover", "charts", "dasha", "grahas"] as const) {
     await expect(page.locator(`[data-page="${sheet}"]`)).toHaveScreenshot(`print-${sheet}.png`);
   });
 }
+
+// Chalit görünümü (yol haritası Plan 25): Placidus ayarlı Delhi haritası, üç biçim, açık tema.
+// Merkür, Jüpiter ve Venüs burç evinden bir önceki bhavanın hücresinde, `*` işaretiyle.
+for (const style of ["north", "south", "circle"] as const) {
+  test(`chalit ${style}`, async ({ page }) => {
+    await mockIpc(page, { session: true, placidus: true });
+    await page.addInitScript(() => localStorage.setItem("jyotish-desk.theme.v1", "light"));
+    await page.goto("/");
+    await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+    await page.keyboard.press("Meta+Backslash");
+    await expect(page.getByRole("complementary", { name: "Danışanlar" })).toHaveCount(0);
+    const chart = page.getByRole("region", { name: "Harita D1", exact: true });
+    await chart.getByRole("combobox", { name: "Biçim" }).selectOption(style);
+    await chart.getByRole("checkbox", { name: "Bhava chalit" }).check();
+    await expect(chart.locator("svg.chart [data-shifted]")).toHaveCount(3);
+    await expect(chart.locator("svg.chart")).toHaveScreenshot(`chalit-${style}.png`);
+  });
+}
+
+// Sudarshana chakra (yol haritası Plan 25): gerçek Delhi haritası, açık ve koyu tema.
+for (const theme of ["light", "dark"] as const) {
+  test(`sudarshana ${theme}`, async ({ page }) => {
+    await mockIpc(page, { session: true });
+    await page.addInitScript((t) => localStorage.setItem("jyotish-desk.theme.v1", t), theme);
+    await page.goto("/");
+    await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+    await page.keyboard.press("Meta+Backslash");
+    await expect(page.getByRole("complementary", { name: "Danışanlar" })).toHaveCount(0);
+    await page.keyboard.press("Meta+k");
+    const palette = page.getByRole("dialog", { name: "Komut paleti" });
+    await palette.getByRole("combobox").fill("sudarshana");
+    await palette.getByRole("option", { name: /Sudarshana chakra panelini aç\/kapat/ }).click();
+    const svg = page.getByRole("region", { name: "Sudarshana chakra" }).locator("svg.sudarshana");
+    await expect(svg.locator(".chart-planet")).toHaveCount(27);
+    await svg.scrollIntoViewIfNeeded();
+    await expect(svg).toHaveScreenshot(`sudarshana-${theme}.png`);
+  });
+}
````

- [ ] **Step 2: Akışları koş**

```bash
lsof -i :1421
pnpm --dir app exec tsc --noEmit -p e2e
pnpm --dir app exec playwright test --project=e2e -g "bhava table and chalit|sudarshana" 2>&1 | tail -3
pnpm --dir app e2e 2>&1 | tail -1
```
Expected: `lsof` çıktısız (1421 boş; doluysa başka bir çalışma ağacının sunucusudur ve testler yanlış arayüzü görür: o sunucu sahibine sorulmadan durdurulmaz; bu adım o bitene dek bekler); `tsc` çıktısız; `3 passed`; `30 passed`.

- [ ] **Step 3: Yeni görsel temelleri üret; var olanların değişmediğini gör**

Önce var olan on temel koşulur (yeni beş test temeli olmadığı için düşer), sonra **yalnızca yeni beş temel** üretilir, sonra hepsi birlikte koşulur. `--update-snapshots` süzgeçsiz koşulmaz: var olan temelleri yeniden yazardı (C11, D17).

```bash
pnpm --dir app e2e:visual 2>&1 | tail -8
pnpm --dir app e2e:visual -g "chalit|sudarshana" --update-snapshots 2>&1 | tail -2
pnpm --dir app e2e:visual 2>&1 | tail -1
git status --short -- app/e2e/visual.spec.ts-snapshots
rm -rf app/test-results
```
Expected: ilk koşu `5 failed` (beş yeni test: temel yok; Playwright eksik temeli yazar ve o testi başarısız sayar) ve `10 passed`; ikinci koşu `5 passed`; üçüncü koşu `15 passed`; `git status` yalnızca beş yeni dosyayı (`??`) listeler, hiçbir `M` satırı yoktur. Var olan temellerden biri ilk koşuda düşerse (font ya da işletim sistemi farkı) bu görevin değişikliği değildir: durulur, fark ilerleme notlarına yazılır ve o temel yeniden üretilmez.

Yeni beş PNG gözle denetlenir (dosya açılarak; uygulama çalıştırılmaz): chalit çizimlerinde Merkür, Jüpiter ve Venüs `*` taşır ve 6., 10. ve 5. evin hücresindedir; Sudarshana çiziminde üç halka, soldaki dolgulu 1. evler, iç halkanın 1. evinde lagna işareti ve her hücrede burç numarası okunur; hiçbir etiket hücresinden taşmaz.

- [ ] **Step 4: Kapılar**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: Vitest `92 passed (92)`, `930 passed (930)`; `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/e2e/desk.spec.ts app/e2e/visual.spec.ts app/e2e/visual.spec.ts-snapshots/chalit-north-visual-darwin.png app/e2e/visual.spec.ts-snapshots/chalit-south-visual-darwin.png app/e2e/visual.spec.ts-snapshots/chalit-circle-visual-darwin.png app/e2e/visual.spec.ts-snapshots/sudarshana-light-visual-darwin.png app/e2e/visual.spec.ts-snapshots/sudarshana-dark-visual-darwin.png
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(e2e): bhava table and chalit view on a Placidus chart, sudarshana panel with and without a birth time; five visual baselines" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: yedi dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 8: Kapılar, `REVIEW.md` ve elle denetim kapısı

Bu görev `REVIEW.md` dosyasını ekler (tek commit) ve bütün kapıları koşar. Kapılarda bulunan bir hata kendi düzeltme commit'iyle çözülür ve bu görev Step 2'den baştan koşulur. Sonuçlar depo dışındaki ilerleme notlarına yazılır (`git add` edilmez). Aşağıda `BASE`, Task 1 Step 1'de `git log -1 --format=%h` ile okunan commit'tir (bu plan yazılırken `7270550`).

**Files:**
- Create: `tests/golden/modules/bhava/REVIEW.md`
- Değişen başka dosya yok.

**Interfaces:**
- Consumes: Task 1–7'nin tamamı.
- Produces: okuma kuralları, onay bekleyen seçimler ve denetim maddeleri (`REVIEW.md`); yeşil kapılar; kural ve gizlilik taramaları; sahibine devredilen **[insan]** maddeleri (ilerleme notlarına yazılır).

- [ ] **Step 1: `REVIEW.md`**

`crates/jyotish-core/tests/modules_golden.rs` yalnızca adını bildiği modül klasörlerindeki `.json` dosyalarını yükler; yalnızca `REVIEW.md` taşıyan yeni klasör testi etkilemez (D10).

**Yeni dosya:** `tests/golden/modules/bhava/REVIEW.md`

````markdown
# Bhava tablosu ve Sudarshana chakra: okuma kuralları, onay bekleyen noktalar ve gözden geçirme kaydı

Kaynak: yol haritası (`docs/superpowers/specs/2026-11-27-eksikler-yol-haritasi.md`) §4.5 ve
§4.6, Plan 25'in kararları (D1–D20) ve ileri modüller tasarım belgesinin K1–K5 kuralları. Bu
modül bellekten aktarılmış klasik tablo taşımaz (Y4'ün "doğrulanmamış tablo" notu gerekmez);
taşıdığı şey okuma seçimleridir ve aşağıda sahibin onayını bekler. Bu dizinde JSON örneği
yoktur: el örnekleri Rust birim testleridir ve türetmeleri test yorumlarında yazılıdır
(`crates/jyotish-core/src/bhava/table.rs`, `crates/jyotish-core/src/bhava/sudarshana.rs`).
Beklenen değerler kuraldan elle türetilmiştir; motor çıktısı kopyalanmaz (K9). Hiçbir satır
hüküm değildir: tablo ve çizim olguları sayar.

## Okuma kuralları

1. **Çerçeve.** Bhava noktalarının boylamı, burcu ve burç içi derecesi rasi çerçevesindedir
   (zodyak kipini izler, K5). Nakshatra ve pada sidereal boylamdan okunur; tropikal-rasi kipinde
   boylamdan ayanamsa düşülür.
2. **Nokta.** Burç lordu v1'in burç lordu tablosundandır (Akrep: Mars, Kova: Satürn). Nakshatra
   lordu Vimshottari sırasıdır: Ashwini'den başlayarak Ketu, Venüs, Güneş, Ay, Mars, Rahu,
   Jüpiter, Satürn, Merkür; sıra üç kez yinelenir. İkinci yazım: bir boylamın nakshatra lordu,
   o boylamdaki Ay'ın Vimshottari doğum lordudur (test iki yazımı 27 nakshatrada karşılaştırır).
3. **Tam burç.** Bhava burcun kendisidir: başlangıç 0°, bitiş 30°. Orta, lagnanın derecesinin
   o burçtaki karşılığıdır (lagna + 30° × (n − 1)); Bhava bala da aynı noktayı kullanır. Hiçbir
   graha burç evinden başka bir bhavada olamaz.
4. **Sripati.** Ephemeris'in döndürdüğü on iki nokta bhava başlangıcıdır (sandhi). Orta,
   Porphyry noktasıdır: MC'den lagnaya uzanan yay ve lagnadan IC'ye uzanan yay üçe bölünür;
   öteki altı nokta bunların karşısıdır. 1. bhavanın ortası lagna, 4. bhavanınki IC, 7.
   bhavanınki batı ufku, 10. bhavanınki MC'dir. İkinci yazım: her sandhi, iki komşu ortanın tam
   ortasıdır (gerçek motorla beş golden anında, iki zodyak kipinde 10⁻⁶° içinde sınanır).
5. **Placidus.** Nokta bhavanın başlangıcıdır (KP okuması). Bhava ortası tanımlanmaz: orta
   sütunu çizgidir ve "ortadan uzaklık" verilmez. Kutup dairesinde Placidus hesaplanamazsa
   başlangıçlar Porphyry noktalarıdır; tablo ve chalit görünümü bunu sözle söyler.
6. **Bitiş ve yay.** Bir bhavanın bitişi sonraki bhavanın başlangıcıdır; yay ikisinin farkıdır
   (mod 360°). On iki yayın toplamı 360°'dir.
7. **Grahanın bhavası.** Boylamdan geriye en yakın başlangıç, içinde bulunulan bhavanın
   başlangıcıdır; başlangıç noktası o bhavaya dahildir (graha tablosunun "ev" sütunuyla aynı
   kural). "Başlangıçtan" değeri [0°, yay) aralığındadır. "Ortadan" değeri işaretlidir: eksi,
   graha ortadan öncedir; artı, sonradır.
8. **Burç evi ve işaret.** Burç evi, grahanın burcunun lagna burcundan sayısıdır (K4). Bhava
   burç evinden farklıysa graha `*` ile işaretlenir.
9. **Chalit görünümü.** Hücreler yerinde durur: n. hücre n. rasi evinin hücresidir ve burç
   numarası o evin burcudur. Her graha, bhavasının numarasını taşıyan hücreye çizilir; burcu ve
   derecesi değişmez (erişilebilir ad grahanın kendi burcunu söyler). İki burca yayılan bhava
   tek hücredir; kesin sınırlar Bhava tablosu panelindedir. Görünüm yalnızca D1'de, doğum saati
   biliniyorsa ve ev sistemi tam burç değilse çizilir; transit halkası, eş halkası ve arudha
   padalar bu görünümde çizilmez.
10. **Sayı prashnası.** Lagna sayıdan gelir ve evler ayar ne olursa olsun tam burçtur.
11. **Doğum saati bilinmiyorsa** bhava tablosu yoktur (Rust `None` döner); chalit görünümü
    çizilmez.
12. **Sudarshana chakra.** Üç halka içten dışa lagna, Ay ve Güneş içindir. Her halkada n. ev, o
    halkanın başlangıç burcundan sayılan n. burçtur (burç evi, K4) ve grahalar rasi
    çerçevesindeki burçlarından okunur. Aynı numaralı ev üç halkada aynı açıdadır; 1. ev
    soldadır ve evler saat yönünün tersine ilerler. Doğum saati bilinmiyorsa lagna halkası
    çizilmez ve Ay'ın burcu yaklaşık (≈) işaretlenir. Çizimde yalnızca graha kısaltması yazılır
    (retro, yanma ve vargottama işaretleri yazılmaz). Sudarshana dashası hesaplanmaz.

## El örnekleri (Rust birim testleri)

| Test | Sınadığı kural |
|---|---|
| `table.rs › nakshatra_lords_repeat_every_nine_and_agree_with_the_dasha_seed` | 2 (ikinci yazım) |
| `table.rs › a_point_reads_sign_nakshatra_and_both_lords` | 1, 2 |
| `table.rs › porphyry_points_trisect_the_quadrants_and_sandhis_bisect_the_points` | 3, 4 |
| `table.rs › sripati_table_by_hand` (lagna 101°, MC 341°; dokuz graha) | 4, 6, 7, 8 |
| `table.rs › whole_sign_table_never_shifts_and_placidus_has_no_middle` | 3, 5 |
| `table.rs › collector_follows_the_chart_and_hides_the_table_when_time_is_unknown` | 5, 11 |
| `table.rs › tropical_rasi_mode_and_number_prashna_follow_the_chart` | 1, 10 |
| `sudarshana.rs › three_rings_count_from_the_lagna_the_moon_and_the_sun` | 12 |
| `sudarshana.rs › without_a_birth_time_the_lagna_ring_is_empty_and_the_result_is_approximate` | 12 |
| `app/src-tauri/tests/bhava.rs` (gerçek motor; beş golden anı × üç ev sistemi × iki zodyak kipi) | 3–8, 10–12 |

## Sahibin onayını bekleyen okuma seçimleri ve bilinen varyant noktaları

Aşağıdaki her satır tek okumayla uygulanmıştır (Y5); ikinci okuma ayar değildir. Sahibi basılı
kaynağıyla karşılaştırır ve sonucu alttaki tabloya yazar. Yürüten ajan bu seçimleri başka
yazılımla ya da çevrimiçi kaynakla karşılaştırmaz.

| # | Nokta | Uygulanan okuma | Bilinen öteki okuma |
|---|---|---|---|
| 1 | Sripati'de bhava ortası | Porphyry noktası (1. bhavanın ortası lagna) | Ortayı başlangıç ile bitişin yay ortası sayan okuma (köşe bhavalarında, yani 1., 4., 7. ve 10. bhavada Porphyry noktasından ayrışır) |
| 2 | Sripati'de bhava sınırı | Ephemeris'in döndürdüğü sandhi bhavanın başlangıcıdır | Porphyry noktasını bhava başlangıcı sayan (sandhisiz) okuma |
| 3 | Tam burçta bhava ortası | Lagnanın derecesi (her burçta aynı derece) | Burcun 15°'sini orta sayan okuma |
| 4 | Placidus'ta bhava ortası | Tanımlanmaz (çizgi) | Başlangıç ile bitişin yay ortasını yazan uygulamalar |
| 5 | Chalit çiziminde iki burca yayılan bhava | Tek hücre: n. bhava n. rasi evinin hücresinde; yer değiştiren graha `*` taşır | Hücreye bhava ortasının burcunu yazan çizim; sandhi derecelerini hücre kenarına yazan çizim |
| 6 | Sudarshana'da halka sırası | İçten dışa lagna, Ay, Güneş | İçten dışa Güneş, Ay, lagna |
| 7 | Sudarshana'da yön | 1. ev solda, saat yönünün tersine | 1. ev üstte; saat yönünde |
| 8 | KP alt lordu (nakshatra alt lordu) | Hesaplanmaz (kapsam dışı) | Bhava başlangıçları ve grahalar için alt lord sütunu |

| Denetleyen | Tarih | Kaynak (genel ad) | Sonuç |
|---|---|---|---|
| — | — | — | Bekliyor: sahibine devredildi (Plan 25 Task 8) |

## Denetim maddeleri

**[otomatik]** maddelerin karşılığı testlerde vardır; yürüten ajan testin adını göstererek
"geçti" sayar. **[insan]** maddeler yalnızca gerçek uygulamaya bakan biri tarafından
doğrulanabilir ve sahibine devredilir (Y3: sonraki planı engellemez).

- [ ] **[otomatik]** Sripati sandhileri Porphyry ortalarının ikişer ikişer ortasıdır; 1. bhavanın ortası lagnadır. (`tests/bhava.rs`; `table.rs › porphyry_points_…`)
- [ ] **[otomatik]** Grahanın bhavası graha tablosundaki evdir; tam burçta hiçbir graha yer değiştirmez; Placidus'ta orta yoktur. (`tests/bhava.rs`; `table.rs › whole_sign_table_…`)
- [ ] **[otomatik]** Doğum saati bilinmiyorsa tablo yoktur, chakra lagna halkası olmadan ve yaklaşık döner; sayı prashnasında evler tam burçtur; geçersiz konum saat bilinmese de bildirilir. (`tests/bhava.rs`; `table.rs › collector_…`; `sudarshana.rs › collector_reads_…`)
- [ ] **[otomatik]** Bhava tablosu paneli üç ev sistemini, kutup dairesindeki düşmeyi, nokta ayarını, `*` işaretini (sözle de) ve "saat bilinmiyor" durumunu gösterir. (`BhavaTablePanel.test.tsx`, on bir test)
- [ ] **[otomatik]** Chalit görünümü grahaları haritanın verdiği bhavaya çizer, yeni istek yapmaz, tam burçta kutuyu sunmaz, D1 dışında ve saat bilinmezken nedenini söyler, öteki katmanları çizmez. (`ChartPanel.test.tsx` chalit testleri; `adapter.test.ts`; `ChartSvg.test.tsx`)
- [ ] **[otomatik]** Sudarshana çizimi 36 hücredir, her graha üç kez yazılır, çizimde sekme durağı yoktur; tablo aynı veriyi taşır ve ortak vurguyu sürer. (`SudarshanaSvg.test.tsx`, `SudarshanaPanel.test.tsx`, `sudarshana.test.ts`; e2e)
- [ ] **[otomatik]** Görsel temeller: chalit (üç biçim) ve Sudarshana (açık, koyu). (`pnpm --dir app e2e:visual`: 15)
- [ ] **[insan]** Sripati ayarlı bir deneme haritasında Bhava tablosunun başlangıç ve orta değerleri, sahibin basılı kaynağındaki bir örnekle (ya da elle Porphyry bölmesiyle) uyuşur; yukarıdaki 1–4. satırlar onaylanır ya da düzeltme istenir.
- [ ] **[insan]** Chalit görünümü ekranda okunur: iki burca yayılan bhavanın tek hücrede gösterimi, `*` işareti ve açıklama satırı yeterlidir (5. satır); Kuzey, Güney ve daire biçimlerinde yer değiştiren graha aynı bhavadadır.
- [ ] **[insan]** Sudarshana çizimi açık ve koyu temada, üç dilde okunur; dar panelde tablo çizimin altında kayar; halka sırası ve yön onaylanır (6. ve 7. satırlar); beş grahalık bir yığılmada etiketler okunur.
- [ ] **[insan]** VoiceOver: Sudarshana çizimi tek bir resim olarak adıyla okunur; tabloda burç ve graha düğmeleri tam adlarıyla okunur; chalit görünümünde yer değiştiren graha "bhavası farklı" diye okunur.
- [ ] **[insan]** KP alt lordunun istenip istenmediği sahibin kararına yazılır (8. satır).

## Gözden geçirme kaydı

Gözden geçiren (motoru yazan dışında biri) `table.rs` ve `sudarshana.rs` testlerindeki el
örneklerini yalnızca girdilerinden (lagna, MC ve graha boylamları) başlayarak, motoru
çalıştırmadan, yukarıdaki okuma kurallarından yeniden hesaplar; sonucunu testteki beklenen
değerlerle karşılaştırır ve aşağıya adını, tarihini ve bulduğu farkları yazar.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
| — | — | Bekliyor (Plan 25 Task 8) |
````

```bash
grep -inE 'k(a|ā|aa)la|\bcala\b' tests/golden/modules/bhava/REVIEW.md
cargo test -p jyotish-core --test modules_golden 2>&1 | grep -E "^test result"
git add tests/golden/modules/bhava/REVIEW.md
git diff --cached --stat
git commit -m "docs(review): reading rules, pending choices and check items for the bhava table and the sudarshana chakra" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: tarama çıktısız; `17 passed`; bir dosya; `git status --short` boş.

- [ ] **Step 2: Rust kapıları**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
BASE=7270550
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
cargo deny check 2>&1 | tail -1
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p jyotish-core -p atlas -p store -p jyotish-desk --all-targets 2>&1 | tail -1
git diff $BASE..HEAD --stat -- deny.toml Cargo.toml Cargo.lock app/package.json app/pnpm-lock.yaml app/vite.config.ts app/src-tauri/capabilities app/src-tauri/tauri.conf.json app/src-tauri/Cargo.toml crates/store crates/atlas crates/ephemeris crates/jyotish-core/Cargo.toml crates/jyotish-core/src/chart.rs crates/jyotish-core/src/houses.rs crates/jyotish-core/src/positions.rs crates/jyotish-core/src/settings.rs crates/jyotish-core/src/dasha.rs crates/jyotish-core/src/dignity.rs crates/jyotish-core/src/error.rs crates/jyotish-core/src/shadbala crates/jyotish-core/tests app/src-tauri/src/commands.rs app/src-tauri/src/module_commands.rs app/src-tauri/tests/modules.rs
git diff $BASE..HEAD --stat -- crates/jyotish-core/src/lib.rs app/src-tauri/src/lib.rs | tail -1
ls crates/jyotish-core/proptest-regressions 2>/dev/null
```
Expected: fmt ve clippy çıktısız; testler: `jyotish-core` lib `415 passed`, golden `8 passed`, `modules_golden` `17 passed`; `atlas` `moment_fixed_lmt` `11 passed`, `moment_iana` `13 passed`; `store` lib `122 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `58 passed`, `bhava` `1 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `modules` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; sayaç `0`; deny `advisories ok, bans ok, licenses ok, sources ok`; MSRV `Finished`; ilk `git diff --stat` çıktısız (bağımlılıklar, şema, var olan hesap modülleri, hata türleri ve paylaşılan test dosyaları değişmedi); iki `lib.rs` için `2 files changed, 13 insertions(+)` (yalnızca ekleme); `ls` çıktısız. 1.93 araç zinciri kurulu değilse MSRV satırı atlanır ve ilerleme notlarına "MSRV yerelde koşulmadı" yazılır (araç zinciri bu görev için kurulmaz).

- [ ] **Step 3: Arayüz kapıları**

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
Expected: `lsof` çıktısız; `Test Files  92 passed (92)`, `Tests  930 passed (930)`; `✓ built`; e2e `30 passed`; görsel `15 passed`; betik testleri `pass 14`, `fail 0`; `git status --short` boş.

- [ ] **Step 4: Kural ve gizlilik taramaları (tabandan bu yana)**

```bash
BASE=7270550
git diff $BASE..HEAD -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log $BASE..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff $BASE..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
git diff $BASE..HEAD --name-only | grep -vE '^(app|crates|tests)/'
git diff $BASE..HEAD --name-only | grep -c '^\.superpowers/'
echo "trailers: $(git log $BASE..HEAD --format=%B | grep -c '^Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>$') commits: $(git rev-list --count $BASE..HEAD)"
git diff $BASE..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
git diff $BASE..HEAD -U0 -- crates app/src-tauri/src | grep '^+' | grep -E 'println!|eprintln!|dbg!|log::|tracing::'
git diff $BASE..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\(|window\.(print|open)\('
git diff $BASE..HEAD -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
grep -rnE '#\[serde\(default' crates/jyotish-core/src/bhava app/src-tauri/src/bhava_commands.rs
grep -rnE 'XMLHttpRequest|WebSocket|\bfetch\(|localStorage|sessionStorage|new Date\(|Date\.now' app/ui/panels/BhavaTablePanel.tsx app/ui/panels/SudarshanaPanel.tsx app/ui/chart/SudarshanaSvg.tsx app/ui/chart/sudarshana.ts app/ui/modules/useBhavaTable.ts app/ui/modules/useSudarshana.ts
grep -cE 'bhava_commands::(compute_bhava_table|compute_sudarshana),' app/src-tauri/src/lib.rs
grep -cE 'CommandError::new\(' app/src-tauri/src/bhava_commands.rs
git diff $BASE..HEAD --stat -- app/ui/chart/geometry.ts app/ui/chart/__snapshots__ app/ui/print app/ui/i18n/i18n.test.ts app/ui/palette
git diff $BASE..HEAD --name-status -- app/e2e/visual.spec.ts-snapshots
git tag --points-at HEAD
```
Expected: `trailers:` satırındaki iki sayı eşit (sekiz commit, sekiz satır; inceleme düzeltmesi commit'i varsa ikisi birlikte artar); `grep -ciE` satırı `0`; `.superpowers` sayacı `0`; komut sayacı `2`; `CommandError::new` sayacı `0` (her hata `CoreError` eşlemesinden geçer). `--name-status` yalnızca beş `A` satırı yazar (var olan temeller değişmedi). Bunların dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur): yasaklı ad yok (üretilen fikstürde de), `app/`, `crates/` ve `tests/` dışında dosya yok, arayüzde `console` ve Rust'ta log çağrısı yok (C1), yasak çağrı yok (C15), yeni stilde hex ya da `rgb(` yok (C9), IPC tiplerinde `serde(default)` yok, yeni yüzeylerde ağ, tarayıcı deposu ve saat çağrısı yok (C4), geometri, SVG anlık görüntüleri, baskı, `i18n.test.ts` ve palet değişmedi, etiket (git tag) yok.

- [ ] **Step 5: Elle denetim kapısı (gerçek uygulama çalıştırılmaz)**

Her maddenin başında, otomatik bir karşılığı olup olmadığı yazılıdır. **[otomatik]** maddelerin karşılığı testlerde vardır ve yürüten ajan bunları testin adını göstererek "geçti" sayar. **[insan]** maddeler yalnızca gerçek uygulamaya bakan bir insan tarafından doğrulanabilir: **yürüten ajan uygulamayı çalıştırmaz** (`pnpm tauri dev` yok) ve bu maddeleri sahibine devreder; devir sonraki planı engellemez (Y3). Maddeler `REVIEW.md`'deki "Denetim maddeleri" ile aynıdır; sonuçlar ilerleme notlarına yazılır.

- [ ] **[otomatik]** Sripati sandhileri Porphyry ortalarının ikişer ikişer ortasıdır; 1. bhavanın ortası lagnadır; beş golden anında, iki zodyak kipinde. (`tests/bhava.rs › bhava_commands_run_on_the_engine_thread_and_agree_with_the_chart`; `table.rs › porphyry_points_trisect_the_quadrants_and_sandhis_bisect_the_points`)
- [ ] **[otomatik]** Grahanın bhavası graha tablosundaki evdir; "başlangıçtan" bhavanın yayı içindedir; tam burçta hiçbir graha yer değiştirmez; Placidus'ta orta yoktur; el örneğindeki dokuz graha türetmeyle uyuşur. (`tests/bhava.rs`; `table.rs › sripati_table_by_hand`, `› whole_sign_table_never_shifts_and_placidus_has_no_middle`)
- [ ] **[otomatik]** Nokta: burç ve derece rasi çerçevesinden, nakshatra ve pada sidereal boylamdan; nakshatra lordu Vimshottari doğum lorduyla aynıdır. (`table.rs › a_point_reads_sign_nakshatra_and_both_lords`, `› nakshatra_lords_repeat_every_nine_and_agree_with_the_dasha_seed`, `› tropical_rasi_mode_and_number_prashna_follow_the_chart`)
- [ ] **[otomatik]** Doğum saati bilinmiyorsa tablo yoktur ve chakra lagna halkası olmadan, yaklaşık döner; sayı prashnasında evler tam burçtur; kutup dairesinde Placidus düşmesi tabloya yazılır; geçersiz konum saat bilinmese de bildirilir. (`table.rs › collector_follows_the_chart_and_hides_the_table_when_time_is_unknown`; `sudarshana.rs › collector_reads_the_chart_positions`; `tests/bhava.rs`)
- [ ] **[otomatik]** Üç halka lagnadan, Ay'dan ve Güneş'ten sayar; her halkada on iki ayrı burç ve dokuz grahanın her biri tam bir kez. (`sudarshana.rs › three_rings_count_from_the_lagna_the_moon_and_the_sun`; `tests/bhava.rs`)
- [ ] **[otomatik]** Bhava tablosu paneli: üç ev sistemi, nokta ayarı (kaydedilir, yeni istek yapmaz), `*` işareti ve sözle okunuşu, Porphyry notu, vurgu, "saat bilinmiyor", hata ve yeniden deneme (odak panelde durur), üç dil. (`BhavaTablePanel.test.tsx`, on bir test; e2e `bhava table and chalit`)
- [ ] **[otomatik]** Chalit görünümü: her graha haritanın verdiği bhavanın hücresinde (üç biçim); işaretlenen grahalar Rust'ın `shifted` listesiyle aynı; hiçbir komut çağrılmaz; tam burçta kutu sunulmaz, kayıtlı ayar nedenini söyler; D1 dışında ve saat bilinmezken neden; öteki katmanlar çizilmez ve istenmez, chalit çıkınca geri gelir; Porphyry notu. (`adapter.test.ts`, `ChartSvg.test.tsx`, `ChartPanel.test.tsx` chalit testleri; e2e)
- [ ] **[otomatik]** Chalit kapalıyken çizim değişmedi: üç SVG anlık görüntüsü ve altı D1 görsel temeli aynı. (`ChartPanel.test.tsx › Delhi D1 … markup matches the snapshot`; `e2e:visual` `d1-*`)
- [ ] **[otomatik]** Sudarshana geometrisi: 36 hücre, 1. ev solda, aynı ev üç halkada aynı açıda; iç halkaya altı, öteki iki halkaya dört etiket taşmadan sığar. (`sudarshana.test.ts`, dört test)
- [ ] **[otomatik]** Sudarshana çizimi ve paneli: tek resim, sekme durağı yok; tablo aynı veriyi taşır ve klavyeyle ortak vurguyu sürer; fare çizimde vurgular; saat bilinmezken lagna halkası yok, sütunu çizgi, Ay "≈"; hata ve yeniden deneme; üç dil. (`SudarshanaSvg.test.tsx`, `SudarshanaPanel.test.tsx`; e2e iki akış)
- [ ] **[otomatik]** İki panel paletten eklenir; ayarları düzenle birlikte kaydedilir; her yeni metin üç dildedir. (`model.test.ts`, `panels.test.tsx`, `i18n.test.ts`; e2e)
- [ ] **[otomatik]** Görsel temeller: chalit (üç biçim) ve Sudarshana (açık, koyu); toplam 15. (`pnpm --dir app e2e:visual`)
- [ ] **[insan]** Sripati ayarlı bir deneme haritasında Bhava tablosunun başlangıç ve orta değerleri, sahibin basılı kaynağındaki bir örnekle (ya da elle Porphyry bölmesiyle) uyuşur; `REVIEW.md`'nin 1–4. satırları onaylanır ya da düzeltme istenir.
- [ ] **[insan]** Chalit görünümü ekranda okunur: iki burca yayılan bhavanın tek hücrede gösterimi, `*` işareti ve açıklama satırı yeterlidir (`REVIEW.md` 5. satır); Kuzey, Güney ve daire biçimlerinde yer değiştiren graha aynı bhavadadır; graha tablosunun "ev" sütunuyla aynı evdedir.
- [ ] **[insan]** Ev sistemi Ayarlar'dan tam burca çevrilince chalit kutusu (ayar açık değilse) kaybolur ve Bhava tablosu paneli "tam burç" notunu yazar; Sripati'ye dönünce kutu geri gelir.
- [ ] **[insan]** Sudarshana çizimi açık ve koyu temada, Türkçe, İngilizce ve Fransızca dilde okunur; dar panelde tablo çizimin altında kayar; halka sırası ve yön onaylanır (`REVIEW.md` 6. ve 7. satırlar); beş grahalık bir yığılması olan deneme haritasında etiketler okunur.
- [ ] **[insan]** VoiceOver: Sudarshana çizimi tek bir resim olarak adıyla okunur; tabloda burç ve graha düğmeleri tam adlarıyla okunur; chalit görünümünde yer değiştiren graha "bhavası farklı" diye okunur; kullanılamayan chalit kutusu nedenini okur.
- [ ] **[insan]** KP alt lordunun istenip istenmediği sahibin kararına yazılır (`REVIEW.md` 8. satır).
- [ ] **[insan]** Denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum verisi, yer ya da koordinat görünmez.

Kapı geçtiyse ilerleme notlarına `Plan 25 Task 8 elle denetim: otomatik maddeler geçti, insan maddeleri sahibine devredildi (<tarih>)` yazılır. Bu adım commit üretmez.

---

## Başarı ölçütleri

- Bhava tablosu haritanın ev sisteminde on iki bhavanın başlangıcını, ortasını ve bitişini (burç, derece, nakshatra, pada, iki lord) ve dokuz grahanın bhavasını ve uzaklığını verir; graha tablosunun "ev" sütunuyla aynı evi gösterir.
- Sripati'nin ortaları Porphyry noktalarıdır ve sandhiler onların ikişer ikişer ortasıdır (gerçek motorla doğrulanır); Placidus'ta orta yoktur; tam burçta hiçbir graha yer değiştirmez.
- Harita paneli chalit görünümünde her grahayı bhavasının hücresine çizer, yer değiştireni işaretler ve yeni bir istek yapmaz; chalit kapalıyken çizim değişmez.
- Sudarshana chakra üç halkayı tek bir çizimde ve aynı veriyi bir tabloda gösterir; tablo klavyeyle ortak vurguyu sürer; saat bilinmezken lagna halkası çizilmez.
- Doğum saati bilinmeyen haritada lagnaya bağlı hiçbir değer üretilmez (Rust'ta) ve arayüz nedenini söyler.
- Veritabanı şeması, `Chart`, hata türleri ve bağımlılıklar değişmez; iki komutun girdisi loglanmaz.
- Sayaçlar: `jyotish-core` lib 415, `jyotish-desk` `bhava` 1, Vitest 92 dosya / 930 test, e2e 30, görsel 15.

## Bu planın kapsamadıkları

- KP alt lordu (nakshatra alt lordu) ve bhava başlangıçlarının alt lord dizisi (D19; sahibi isterse ayrı kalem).
- Sudarshana dashası; chakrada retro, yanma ve vargottama işaretleri; chakrada geçiş grahaları.
- Bölüm haritalarında chalit; geçiş ve eş halkasının chalit yerleşimi; arudha padaların chalit çiziminde gösterimi.
- Bhava başlangıç derecelerinin harita çizimine yazılması; bhava ortasının burcunu hücreye yazan ikinci bir chalit biçimi (Y5: ikinci okuma ancak uygulanınca ayar olur).
- Placidus'ta yay ortası; tam burçta burcun 15°'sini orta sayan okuma.
- Baskı belgesinde bhava tablosu, chalit ve chakra (K10); CSV dışa aktarımı.
- Bhava tablosunun harita aramasında koşul olması (Plan 22 "kapsamadıkları": bhava evi koşulu ayrı bir karardır).
- `nakshatra_lord`'un `dasha.rs` ile ortaklaştırılması (paralel planlar bittikten sonra bir temizlik işi).
- Yorum metni, AI paneli; Windows ve Android.

## Yol haritasına geri bildirim

1. **(D9) Görev sırası.** Taslak sekiz görevi (1) çekirdek, (2) komut ve tablo paneli, (3) chalit, (4) chalit görsel temelleri ve ev sistemi durumları, (5) Sudarshana çekirdeği ve geometrisi, (6) Sudarshana paneli, (7) i18n ve "saat bilinmiyor", (8) kapılar diye sayar. Bu plan iki çekirdeği öne (Task 1–2), iki komutu ve üretilen dosyaları tek göreve (Task 3) alır; her arayüz görevi kendi üç dildeki metinlerini ve "saat bilinmiyor" durumunu taşır; görsel temeller uçtan uca akışlarla birlikte Task 7'dedir. Gerekçe: üretilen dosyaların bir kez yeniden üretilmesi ve her görevin kendi başına yeşil olması.
2. **(D12) Chalit "adaptör ve geometri".** Geometri değişmedi: bhava, numarasını taşıyan rasi evinin hücresine çizilir. Bhava ortasının burcunu hücreye yazan çizim ikinci okumadır ve `REVIEW.md` 5. satırdadır.
3. **(D2) "Porphyry noktaları da gerekir".** Yeni ephemeris çağrısı türü gerekmedi: noktalar lagna ile MC'den aritmetikle kurulur ve Sripati sandhileriyle ikinci yazım olarak doğrulanır.
4. **(D16) Fikstür ölçümü.** Delhi fikstüründe Sripati hiçbir grahayı yer değiştirmez; chalit görünümünün fikstürü bu yüzden Placidus haritasıdır. Sripati'de yer değiştiren grahası olan ikinci bir fikstür haritası istenirse eş haritanın anı (İstanbul 1987) uygundur (ölçüm: Sripati'de sekiz graha yer değiştirir).
5. **(D6) Nakshatra lordu.** Üç plan (23, 25, 26) aynı küçük fonksiyonu ister; yol haritasına ortak bir yer (`dasha.rs` ya da `zodiac.rs`) kararı eklenmelidir.
6. **(D20) Ek A.** Türkçe "halka" sözcüğünün çoğulu yasak diziyi üretir; Ek A'nın "dikkat edilecek yerler" listesine eklenmelidir (Plan 31'in ızgara ve çerçeve metinlerinde de geçecektir).

## Self-Review

1. **Kapsam:** yol haritası §4.5 ve §4.6'nın her maddesi "Kapsam" tablosunda bir göreve bağlıdır. Taslaktan bilerek ayrılan yerler Kararlar'da "Yol haritası farkı" diye işaretlidir (D9, D12) ve geri bildirim listesindedir. Kapsam dışı bırakılanlar (KP alt lordu, Sudarshana dashası) yol haritasının kendi kapsam dışı listesindedir.
2. **Yer tutucu taraması:** planda "TBD", "TODO", "sonra doldurulur" ya da kodu olmayan kod adımı yoktur; her kod adımı tam dosya ya da uygulanabilir `diff` taşır. `<tarih>` yürütme anında yazılan değerdir (Task 8). Üretilen iki dosyanın ve beş PNG'nin içeriği planda yoktur: komutla üretilir.
3. **Tip ve ad tutarlılığı:** `BhavaTable`, `BhavaRow`, `BhavaPoint`, `GrahaBhava` Task 1'de; `SudarshanaChakra`, `SudarshanaRing`, `SudarshanaHouse`, `SudarshanaRef` Task 2'de; `compute_bhava_table_blocking`, `compute_sudarshana_blocking` Task 3'te tanımlanır ve sonraki görevlerde aynı adlarla kullanılır. Komut adları Rust'ta `snake_case`, arayüzde `camelCase`'tir (`compute_bhava_table` / `computeBhavaTable`). Panel türleri `bhava_table` ve `sudarshana`'dır (katalog, kayıt, i18n anahtarı ve testlerde aynı). `SHIFT_MARK` Task 4'te panelde tanımlanır ve Task 5'te `chart/types.ts`'e taşınır (Task 5'in `diff`'i yerel sabiti siler).
4. **Dersler (önceki planlardan):** hata türü eklenmedi ve hata parametreleri değer taşımıyor (D8); girdi loglanmıyor (Task 8 Step 4 taramaları); tercihlere yazılmıyor, panel ayarı düzenle birlikte kaydediliyor (C3); kullanılamayan denetim `aria-disabled` ve nedeni görünür (D12); yeniden denemede odak gövdeye düşmüyor (`BhavaTablePanel.test.tsx`); renk tek başına anlam taşımıyor (C9); üretilen dosyalar yalnızca komutla değişiyor (C10); var olan görsel temeller yeniden yazılmıyor (D17); yasak dizi Türkçe çoğul ekinde bulundu ve metinler yeniden yazıldı (D20).
5. **Koşulmuş kod:** bu plandaki her kod bloğu, `7270550`'nin bir kopyasında sekiz ayrı commit olarak uygulanmış ağaçtan alınmıştır. Son ağaçta `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (başarısız sonuç satırı 0; `jyotish-core` lib 415; `bhava` 1), `cargo deny check`, 1.93 ile `cargo check`, Vitest (92 / 930), `pnpm build`, e2e (30), görsel (15; var olan on temel o kopyada değişmeden geçti) ve yayın betiği testleri (14) koşulmuş ve geçmiştir. Ara sayaçlar (412, 888, 899, 914) görev commit'lerinde ölçülmüştür. Yeni beş görsel temel kopyada üretilmiş ve gözle denetlenmiştir; sahibin makinesinde Task 7'de yeniden üretilir. **[insan]** maddeleri koşulmamıştır; gerçek uygulama hiç çalıştırılmamıştır.
6. **Yasaklı ad:** bu belge ve bütün kod blokları `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile taranmıştır; çıktı boştur.
