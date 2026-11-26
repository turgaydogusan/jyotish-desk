# Plan 18 — Alt proje 3.5: Yogalar (41 kurallık katalog, olgu listesi; yoga paneli)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** D1 haritasında oluşan klasik yogaları olgu listesi olarak vermek (hangi yoga, hangi grahalar, hangi evler, hangi ilişki, hangi alt koşul): `jyotish-core`'da veriyle tanımlı 41 kurallık `yogas` kataloğu, `compute_yogas` toplayıcısı ve komutu, grup süzgeçli `yogas` paneli. Güç derecelendirmesi, hüküm ve açıklama metni yoktur.

**Architecture:**
- **Saf çekirdek, ince toplayıcı (spec K3).** `jyotish-core`'a `yogas` modülü eklenir: `mod.rs` (tipler, ilişki yardımcıları, katalog, rapor, toplayıcı), `luminaries.rs` (Ay ve Güneş yogaları), `placement.rs` (mahapurusha, neecha bhanga, nabhasa, kuja dosha), `lords.rs` (raja, dhana, değişim, viparita). Her kural yalnızca `Positions` gören saf bir yüklemdir; el örnekleri ephemeris olmadan kurulur. Katalog bir veri tablosudur (`YOGA_CATALOG`: kimlik, grup, lagna gereksinimi, yüklem).
- **Tek komut, Plan 14'ün kalıbıyla (spec K1, K2).** `compute_yogas(input: ChartInput) -> YogaReport`; motor iş parçacığı, `blocking` havuzu, `CommandError`. `Chart` ve `open_client` değişmez; sonuç saklanmaz. Doğum saati bilinmiyorsa lagna isteyen 17 kural Rust'ta atlanır ve sayılır; diğer kayıtlar `approximate` işaretiyle döner.
- **Arayüz hesap yapmaz.** Kayıtlar, sayaçlar ve yaklaşık işareti Rust'tan gelir; panel gruplar, süzer ve çizer. Panel paletten eklenir (K12). Yoga adları ve koşul etiketleri i18n anahtarıdır; Rust kataloğu fikstürle arayüz testine taşınır ve her kimliğin üç dilde adı olduğu orada denetlenir.
- **Kalıcı biçim:** yalnızca panel ayarı (`group`). Şema sürümleri, SQLite şeması, `CalcSettings` ve `ViewState` değişmez.
- **Yogalar Shadbala kullanmaz.** Spec §8 güç derecelendirmesini kapsam dışı bırakır; Plan 17'nin `shadbala` API'si bu planda tüketilmez (güç koşulu isteyen bir kural eklenirse `jyotish_core::shadbala`'dan okunur).
- **Testler:** kural düzeyinde el örnekleri (her dosyanın `mod tests`'i), elle yazılmış tablolar (bağımsız ikinci yazım), raporun tamamını yazan on beş el fikstürü (`tests/golden/modules/yogas/`, kaynak yalnızca `hand`; her kuralın oluşan ve oluşmayan örneği, üç iptal durumu), proptest değişmezleri, sahte ephemeris'le toplayıcı testi, gerçek motorla entegrasyon testi (v1 haritasıyla çapraz denetim, bütçe), Vitest (panel), Playwright e2e.

**Tech Stack:** Rust 1.93+ (stable; yerelde ve CI'da 1.100), Tauri 2.12, tauri-specta 2.0.0-rc.29, serde, proptest, React 19, TypeScript 7, Vitest 5, Playwright 1.65.1. **Yeni Rust ya da npm bağımlılığı yok.**

**Spec:** `docs/superpowers/specs/2026-11-26-alt-proje-3-ileri-moduller-design.md` (Plan 17'nin geri bildirimi işlenmiş hâli): §2 (K1–K12), §4 (`Positions`, burç temelli bakış, `kuja_dosha`), §8 (Alt proje 3.5), §13 (ortak test stratejisi), §14 (gizlilik), §15 (bütçeler), Ek A (adlandırma). UI spec: `2026-11-24-ui-design.md` §3.3, §3.4, §5, §6, §8.

**Ön koşul:** Taban, Plan 17'nin son inceleme düzeltmelerini taşıyan `main` ucudur: commit `9362280` (Plan 17 son inceleme düzeltmeleri `92a37a5` ve `780f28a`, çalışma kâğıdının yeniden türetilmesi `1558abd`, spec'e işlenen geri bildirim). `main` bu commit'ten ilerlediyse Task 1 adım 1 tabanları yeniden ölçer ve sayaç zinciri o ölçümden yürütülür (farklar aynıdır). Plan 17'nin elle denetim kapısının yalnızca insana ait maddeleri sahibine devredilmiştir; bu planı engellemez.

## Kararlar

Sahibi kararları devretti; muhafazakâr seçenek seçildi ve her karar bir satır gerekçeyle yazıldı. "Spec boşluğu" ya da "Spec kusuru" diye işaretlenenler spec'te düzeltilmesi gereken yerlerdir ve belgenin sonundaki "Spec'e geri bildirim" bölümünde aynı numarayla toplanmıştır. D18, D24 ve sayaç zinciri bu plan yazılırken deponun bir kopyasında ölçülmüştür (macOS 27.2, Rust 1.100).

- **D1 — Modül `crates/jyotish-core/src/yogas/` dizinidir: `mod.rs`, `luminaries.rs`, `placement.rs`, `lords.rs`. Katalog `YOGA_CATALOG: &[YogaRule]`'dur (41 satır, spec §8 tablosunun sırasıyla); `YogaRule { id, group, needs_lagna, eval }`, `eval` konumlardan `Vec<YogaMatch>` üreten saf bir fonksiyondur. `YOGA_CATALOG_VERSION = 1`. Kimlikler `"<grup>.<ad>"` biçimindedir; grup önekleri `mahapurusha`, `lunar`, `solar`, `raja`, `dhana`, `exchange`, `viparita`, `neecha_bhanga`, `nabhasa_count`, `nabhasa_ashraya`, `dosha`'dır.** Gerekçe: spec "veriyle tanımlı kural kataloğu; her kural saf bir yüklemdir" der; satır başına kimlik, 3.9'un arama koşullarının ve i18n adlarının tek kaynağıdır.
- **D2 — İlişki tipi `YogaRelation { Conjunction, MutualAspect, Exchange }` adını taşır.** Gerekçe: spec `Relation` yazar; v1'in `dignity::Relation` tipi (doğal dostluk: `Friend`, `Neutral`, `Enemy`) aynı adla `jyotish_core` kökünden dışa aktarılır ve specta TS adlarını tek ad uzayında üretir. **Spec kusuru 1.**
- **D3 — `YogaReport` spec'teki dört alanı taşır (`catalog_version`, `hits`, `evaluated`, `skipped_time_unknown`); `method` ve `warnings` alanı yoktur. Yöntem kimliği `catalog_version`'dır; arayüz "Katalog sürümü: 1" satırını gösterir. Temel haritanın `LowPrecision` uyarısı rapora taşınmaz (harita başlığındaki rozette görünür). Kayıtlar katalog sırasıyladır; bir kuralın birden çok eşleşmesi graha sırasıyla (`Graha::ALL`; çiftlerde önce ilk, sonra ikinci graha) döner. `evaluated + skipped_time_unknown` her zaman 41'dir.** Gerekçe: K7 "her modül sonucu `method` kimliği taşır" ve K8 "ölümcül olmayan durumlar modül sonucunun `warnings` alanındadır" der; §8'in tipi ikisini de taşımaz. Tip AI paketleyicisinin sözleşmesidir; §8 esas alındı. **Spec kusuru 4.**
- **D4 — Doğum saati bilinmiyorsa (`Positions.lagna_sign == None`): koşulunda lagna zorunlu olan 17 kural değerlendirilmez ve `skipped_time_unknown`'da sayılır (beş mahapurusha, Kemadruma, Shakata, üç raja, dhana, üç değişim, üç viparita). Diğer 24 kural lagna seçenekleri olmadan değerlendirilir (Amala ve Vasumati yalnızca Ay'dan, Neecha bhanga'nın kendra koşulları yalnızca Ay'dan, Kuja dosha Ay'dan ve Venüs'ten); bunların her kaydı `approximate = true` taşır ve `houses` boştur.** Gerekçe: spec "lagnaya bağlı yogalar değerlendirilmez; Ay ve Güneş yogaları yaklaşık işaretiyle döner" der; ama Ay grubundaki Kemadruma ile Shakata'nın iptal kuralı lagna ister (iptali denetlenemeyen yoga bildirilmez), ve Neecha bhanga, nabhasa ve dosha grupları için hiçbir şey yazılmamıştır. Öğle konumunda Ay'ın burcu gün içinde değişebildiği için Ay'ı içeren her kural (nabhasa dahil) yaklaşıktır; tek kural: lagna yoksa her kayıt yaklaşıktır. **Spec kusuru 7.**
- **D5 — `grahas` yogayı oluşturan grahalardır (`Graha::ALL` sırasında, yinelemesiz): Ay ve Güneş yogalarında ışık da listelenir; Neecha bhanga'da yalnızca düşüşteki graha (koşulu sağlayan grahalar listelenmez); nabhasa yogalarında yedi graha; Kuja dosha'da Mars. `houses` lagnadan sayılan evlerdir (artan, yinelemesiz): raja ve dhana kayıtlarında çiftin lordu olduğu ilgili evler (kendra ve trikonalar; {1, 2, 5, 9, 11}); değişimde değişen iki ev; viparita'da lordu olunan ve bulunulan ev; diğer bütün kurallarda listelenen grahaların bulunduğu evler. Düğümler hiçbir kayıtta yer almaz.** Gerekçe: spec alanları "hangi grahalar, hangi evler" diye tanımlar; kural başına içerik yazılmamış. Neecha bhanga'da tek graha, "aynı `id` ve aynı graha kümesi iki kez dönmez" property'sini karşılıklı düşüşte de korur. **Spec boşluğu 8.**
- **D6 — Ay ve Güneş yogaları: "sayılan grahalar" Mars, Merkür, Jüpiter, Venüs, Satürn'dür (`COUNTED_GRAHAS`); Güneş, Ay ve düğümler sayılmaz. Kemadruma'da "Ay'dan 2. ve 12. boş" ve "Ay yalnız" bu beş grahaya göre okunur (Ay'ın burcunda sayılan graha yok); "Ay'dan kendralar" 1., 4., 7. ve 10. evlerdir. Gajakesari'de Ay'la aynı burç (1. ev) kendradır. Adhi'de üç graha aynı evde de olabilir. Amala'da "o evde kötücül yok", evdeki her grahanın K6'ya göre iyicil olmasıdır (Merkür'ün sınıfı `nature`'dandır; düğümler kötücüldür).** Gerekçe: spec bu ayrıntıları yazmaz; hepsi tablodaki cümlenin en dar okumasıdır. **Spec boşluğu 9.**
- **D7 — Alt koşul kimlikleri sabit bir kümedir (`YOGA_CONDITIONS`, 8 kimlik) ve i18n anahtarıdır: `own_sign`, `exaltation` (mahapurusha), `from_lagna`, `from_moon`, `from_venus` (Amala, Vasumati, Kuja dosha), `sign_lord_in_kendra`, `exalted_graha_in_kendra`, `dispositor_aspect` (Neecha bhanga). Mahapurusha burç düzeyindedir: kendi burcu ve yücelme burcu derecesinden bağımsızdır; Merkür Başak'ta iki koşulu da taşır. İki referanslı kurallar (Amala, Vasumati, Kuja dosha) tek kayıt verir; sağlayan referanslar koşuldur.** Gerekçe: spec `conditions: Vec<String>` ve "sağlanan alt koşul kimlikleri" der, kimlikleri saymaz; Kuja dosha için "üç referans ayrı alanlarda" der ama `YogaHit`'te ayrı alan yoktur. v1'in dignity sırası Merkür'ü Başak'ın 15°'sinden sonra moolatrikona ya da kendi burcunda sayar; kural burç adıyla yazıldığı için derece ayrımı yapılmaz. **Spec boşluğu 10.**
- **D8 — İlişkiler: "birlikte" aynı rasi burcudur; "karşılıklı bakış" iki grahanın birbirinin burcuna burç temelli tam bakışıdır (`graha_aspect_signs`): 7. burç her çiftte karşılıklıdır, özel bakışlardan yalnızca Mars'ın 4. ve Satürn'ün 10. bakışı birbirini karşılar; "değişim" iki farklı grahanın birbirinin kendi burcunda olmasıdır. Bir çift birden çok ilişki taşıyabilir (karşılıklı bakış ve değişim); kayıt tektir ve `relations` listesi `YogaRelation` sırasındadır. Değişim kayıtları (`exchange.*`) yalnızca `Exchange` taşır. Chandra-Mangala ve Budha-Aditya `Conjunction` taşır; diğer kurallarda `relations` boştur.** Gerekçe: spec §8 ortak tanımları; hangi kaydın `relations` taşıdığı yazılmamış. **Spec boşluğu 11.**
- **D9 — Raja yogaları: lordlar `lordship::house_lord`'dur (Akrep: Mars, Kova: Satürn). Lagna lordu hem kendra hem trikona lordudur; yalnızca kendra lordu olan iki graha (ör. 4. ve 7. lord) yoga vermez. Kendra–trikona kaydı çift başınadır. Dharma-Karmadhipati, 9. ve 10. lord farklı grahalarsa ve ilişkiliyse ek bir kayıttır: aynı çift `raja.kendra_trikona` olarak da döner. Yogakaraka yalnızca lagna burcuna bağlıdır (grahanın konumuna değil): Boğa ve Terazi'de Satürn, Yengeç ve Aslan'da Mars, Oğlak ve Kova'da Venüs; diğer altı lagnada yoktur.** Gerekçe: spec "9. ve 10. lord özel durumu" der, genel kaydın yerine mi yanına mı döndüğünü yazmaz; genel kayıt listede durursa 3.9'un "kendra–trikona var" koşulu özel durumu da bulur. **Spec boşluğu 12.**
- **D10 — Dhana: çiftin iki grahası da {1, 2, 5, 9, 11} evlerinden en az birinin lordudur, ikisinden en az biri 2. ya da 11. evin lordudur ve ilişkilidirler. Çift başına tek kayıt.** Gerekçe: spec cümlesinin harfiyen okuması. **Spec boşluğu 12.**
- **D11 — Değişim sınıfını değişen iki ev belirler (iki grahanın bulunduğu evler; her graha diğerinin lordu olduğu evde durur): biri 6, 8 ya da 12 ise Dainya; değilse biri 3 ise "3. ev lordu değişimi"; değilse Maha (`exchange_kind`). Üç sınıf bütün ev çiftlerini böler.** Gerekçe: spec "lordlar {1, 2, 4, 5, 7, 9, 10, 11} içindeyse" der; beş graha iki evin lordudur ve lordluk üzerinden okuma aynı değişimi iki sınıfa sokar (Koç lagnada Mars 1 ve 8'in lordudur: 1. ve 2. evlerin değişimi hem Maha hem Dainya olurdu). **Spec kusuru 5.**
- **D12 — Viparita raja harfiyen uygulanır: 6., 8. ya da 12. lord bu üç evden birindeyse (kendi evi dahil) kayıt döner. Hiçbir graha iki dusthananın birden lordu değildir (6, 8 ve 12. evlerin aralarındaki uzaklık çift, bir grahanın iki burcu arasındaki uzaklık tek sayıdır); üç kural üç ayrı grahaya bakar.** Gerekçe: spec cümlesi; `houses` D5'te.
- **D13 — Neecha bhanga: düşüş burcun tamamıdır (derece ayrımı yok); düşüşteki her graha için ayrı kayıt; koşullar: (1) düşüş burcunun lordu lagnadan ya da Ay'dan kendrada, (2) o burçta yücelen graha lagnadan ya da Ay'dan kendrada (Akrep'te yoktur), (3) dispozitörün (aynı lord) burç temelli tam bakışı. Ay kendisinden sayılmaz: lord ya da yücelen graha Ay ise yalnızca lagna referansı geçerlidir. Hiçbir koşul sağlanmıyorsa kayıt yoktur.** Gerekçe: Mars Yengeç'te düşüşteyken burcun lordu Ay'dır ve Ay kendisinden 1. evdedir; harfiyen okumada Mars'ın düşüşü her haritada iptal olurdu. **Spec kusuru 3.**
- **D14 — Nabhasa: yedi graha (düğümler dışında) sayılır. Sayı yogalarından tam biri her haritada oluşur; ashraya yogası en çok bir tanedir. Sayı ve ashraya yogaları birbirini dışlamaz (tek burçtaki yedi graha Gola ile birlikte bir ashraya yogası da verir).** Gerekçe: spec iki grubu ayrı satırlarda verir ve dışlama yazmaz; "sayı yogalarından tam biri oluşur" property'si bunu gerektirir.
- **D15 — El örnekleri raporun tamamını yazar: `tests/golden/modules/yogas/` altında on beş JSON (on üçü lagnalı, ikisi doğum saati bilinmeyen) ve `REVIEW.md` (okuma kuralları ve gözden geçirme kaydı). `expected`: `hits` (bütün kayıtlar, katalog sırasıyla), `absent` (örneğin "oluşmayan durum" olarak gösterdiği kurallar), `cancelled` (iptal örnekleri), `evaluated`, `skipped_time_unknown`. Ayrı bir test 41 kuralın her birinin en az bir örnekte oluştuğunu, en az bir örnekte `absent` sayıldığını, üç iptal durumunun, sekiz koşul kimliğinin ve üç ilişki türünün bulunduğunu sabitler. "İptal edilen durum": temel koşulu sağlandığı hâlde kendradaki grahayla oluşmayan Kemadruma; Ay'ı lagnadan kendrada olan Shakata; düşüşte olup hiçbir koşulu sağlanmayan graha (Neecha bhanga kaydı yok).** Gerekçe: spec §8 "her yoga için en az bir oluşan ve bir oluşmayan sentetik konum; iptal kuralı olanlarda ayrıca iptal edilen durum" der; §13 şeması `expected`'ı modüle bırakır. Neecha bhanga'nın kendisi bir iptaldir; onun "iptal edilen durumu" ancak bhanganın oluşmaması olabilir. **Spec boşluğu 13.**
- **D16 — Bağımsız ikinci yazım (§13 katman 3), testte elle yazılmış tablolarla: karşılıklı bakışın çift ve burç uzaklığına göre tablosu (`hand_mutual`), beş mahapurushanın burçları (`HAND_MAHAPURUSHA`), düşüş burcu → lord ve yücelen graha (`HAND_FALL`), dolu burç sayısı → sayı yogası (`HAND_COUNT`), lagna → yogakaraka (`HAND_YOGAKARAKA`, 12 satır), ev çifti → değişim sınıfı (`HAND_EXCHANGE`, 66 hücre), lagna isteyen kurallar (`HAND_NEEDS_LAGNA`, 17 kimlik).** Gerekçe: §13 "tablo olarak yazılabilen kurallar testte elle girilmiş tabloyla, motorda formülle durur".
- **D17 — Görünen adlar i18n anahtarıdır: `yogas.name.<grup>.<ad>`, `yogas.group.<YogaGroup>`, `yogas.condition.<kimlik>`, `yogas.relation.<YogaRelation>`. Klasik adlar üç dilde aynı yazılır (K11); yalnızca üç değişim sınıfı çevrilir ("Maha değişimi", "3. ev lordu değişimi", "Dainya değişimi"; Ek A). Rust kataloğu (`id`, `group`) ve koşul kimlikleri `ui_fixture.rs` ile fikstüre yazılır (`yoga_catalog`, `yoga_conditions`); Vitest her kimliğin, grubun, koşulun ve ilişkinin üç dilde adı olduğunu ve katalogda olmayan bir adın durmadığını bu listeden denetler.** Gerekçe: spec "her `id` katalogdadır ve üç dilde adı vardır" property'sini ister; katalog Rust'ta, adlar TS'te durur ve aralarında tip bağı yoktur (`id: String`). Yeni komut eklemek yerine mevcut fikstür yolu kullanıldı. **Spec boşluğu 14.**
- **D18 — `compute_yogas` her girdide rapor döner (`Option` değildir); ayar, an ve konum denetimi `compute_chart`'ınkidir, yeni hata kodu yoktur. Bütçe entegrasyon testinde ölçülür: ≤ 10 ms (§15; temel harita dahil).** Gerekçe: ölçüm (hata ayıklama derlemesi): ortanca 0,63–0,64 ms.
- **D19 — "Uçtan uca: beş `swetest` fixture anı" denetimi beş anın beşinde de rapor verir (yogalar gün doğumuna bağlı değildir). Test v1 haritasıyla çapraz denetler: rapor `yogas(&Positions::from(&chart))` ile aynıdır; mahapurusha veren graha v1'in dignity sırasında yücelmiş, moolatrikonada ya da kendi burcundadır; Neecha bhanga kaydındaki graha `Debilitated`'dir; Budha-Aditya Güneş ile Merkür'ün burç eşitliğidir; Kuja dosha 3.1'in olgusudur; ev sistemi ayarı (Sripati, Placidus) raporu değiştirmez (K4).** Gerekçe: §13 katman 4.
- **D20 — Panel türü `yogas` (7 × 14) eklenir; `MODULE_PANELS`'e girer ve paletten eklenip çıkarılır (K12). Kanca `useYogas`'tır; rapor oturumda paylaşılmaz (Plan 15 D16 sürer). Panel ayarı `group`: `"all"` ya da bir `YogaGroup`; tanınmayan değer ve eksik anahtar "tümü"dür.** Gerekçe: spec "Ayar: grup süzgeci"; komut 1 ms'nin altındadır.
- **D21 — Panel düzeni: grup başına bir tablo (görünür başlık grup adıdır), yalnızca kaydı olan gruplar; sütunlar Yoga, Grahalar, Evler, İlişki, Koşullar; boş hücre "—". Graha kısaltmayla yazılır (tam adı okunur), ev numarası "N. ev" diye okunur. Yaklaşık kayıtlar `Approx` bileşeniyle "≈" işaretini ve ekran okuyucu için "yaklaşık" sözünü taşır. Altta: doğum saati bilinmiyorsa "lagnaya bağlı 17 kural değerlendirilmedi" açıklaması, "Değerlendirilen kural: N. Oluşan kayıt: M." satırı, yalnızca D1'in değerlendirildiğini ve evlerin rasi evi olduğunu söyleyen satır ve "Katalog sürümü: 1". Tablolar tek bir kaydırma kabındadır (`BalaScroll`; klavye odağı alır, adı "Oluşan yogalar"): ilişki ve koşul sütunlarında odaklanabilir denetim yoktur. Süzgeç yükleme ve hata durumlarında da görünür.** Gerekçe: spec "gruplara ayrılmış liste; her satır: yoga adı, oluşturan grahalar, evler, ilişki türü, varsa sağlanan alt koşullar" der; düzeni, sayaçları ve yaklaşık işaretinin gösterimini vermez. **Spec boşluğu 15.**
- **D22 — Ortak vurgu: satırdaki her graha ve her ev ayrı bir ortak vurgu düğmesidir (`PinButton`); üzerine gelince ya da odaklanınca o öğe bütün panellerde vurgulanır, tıklayınca sabitlenir. Ters yönde, ortak vurgudaki grahayı ya da evi (burç vurgusunda: lagnadan sayılan evini) içeren bütün yoga satırları işaretlenir.** Gerekçe: spec "satıra gelince grahalar ve evler ortak seçimle tüm panellerde vurgulanır" der; ortak seçim deposu (UI spec §3.4, `highlight/store`) tek hedef taşır (bir graha, bir ev ya da bir burç). Çoklu hedef, bütün panellerin vurgu okuyuşunu değiştirirdi. **Spec kusuru 6.**
- **D23 — Boş durumlar: süzülen grupta kayıt yoksa "Bu grupta oluşan yoga yok: <grup>."; rapor hiç kayıt taşımıyorsa spec'in metni ("Katalogdaki yogalardan hiçbiri oluşmuyor. Değerlendirilen kural: N."). İkincisi Rust'tan gelen raporda görülemez (savunma amaçlı çizilir ve taklit raporla test edilir).** Gerekçe: spec'in kendi property'si "sayı yogalarından tam biri oluşur" der; rapor her zaman en az bir kayıt taşır ve spec'in boş durumu erişilemezdir. **Spec kusuru 2.**
- **D24 — Yeni görsel temel (PNG) ve yeni baskı düzeni yoktur (K10). `pnpm build` Vite'ın 500 kB parça uyarısını basmayı sürdürür (ölçüm: 543,2 kB) ve kapılar `✓ built` satırını arar (Plan 15 D22).** Gerekçe: on görsel temel harita öğelerini ve baskı sayfalarını çeker; modül panelleri bu görüntülerde yoktur (ölçüm: `e2e:visual` 10 geçti, temel dosyalar değişmedi).

## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni bağımlılık yok: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `app/package.json`, `app/pnpm-lock.yaml` ve `app/vite.config.ts` değişmez; `cargo deny check` Task 7'de `advisories ok, bans ok, licenses ok, sources ok` verir.
- **Yasaklı ad kuralı (v1 spec §10 "İsim kuralları", ilk madde; ileri modüller spec'i Ek A):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Kural harf dizisi düzeyindedir: Türkçe sözcüklerin içinde de geçemez (yerine ör. "diğer", "ötekiler", "durur", "geri", "sürer", "tutar" yazılır). **Bu planda özel dikkat:** katalogdaki 41 kimliğin ve görünen adın hiçbiri yasak diziyi içermez (taranmıştır). Düğüm eksenine dayalı modern yoga kataloğa alınmaz (spec §8, Ek A) ve adı hiçbir yere yazılmaz. `-ka` ile biten bir sözcüğe (`yogakaraka`, `karaka`, `naisargika`) `-lar` çoğul eki ya da `L` ile başlayan bir ad bitişik yazılınca (camelCase'te de) yasak dizi oluşur; böyle birleşimler kurulmaz ("yogakaraka grahaları" yazılır). Her commit adımı eklenen satırları, dosya adlarını ve commit mesajını `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile tarar; çıktı boş olmalıdır (C13). Bu belgede de eşleşme yoktur.
- **Clean-room ve yorum metni yok:** kurallar klasik tanımdan özgün olarak uygulanır; başka ürünün adı, düzeni, metni ya da çıktısı kullanılmaz; kaynak gösterimi geneldir ("BPHS'nin yoga bölümleri") ve alıntı yapılmaz. Panel hesaplanmış olguları ve klasik etiketleri gösterir (yoga adı, grahalar, evler, ilişki, koşul); bir yoganın ne anlama geldiğini, iyi mi kötü mü olduğunu anlatan metin eklenmez. Koşul etiketleri ("kendi burcunda", "Ay'dan") olgudur, yorum değildir.
- **Evler rasi evidir (K4), rasi çerçevesi `settings.zodiac`'ı izler (K5), doğal sınıf `nature`'dır (K6).** Lordlar ve evler lagna burcundan sayılan bütün burç evleridir; `CalcSettings.house_system` hiçbir kuralı etkilemez. Modül kendi iyicil/kötücül listesini tutmaz. Yalnızca D1 değerlendirilir.
- **Doğum saati bilinmiyorsa lagna isteyen kurallar Rust'ta atlanır (K2, D4).** Arayüzün gizlemesine güvenilmez.
- **El örnekleri (§13, K9):** `source` yalnızca `hand`'dir; beklenen değer kuraldan elle türetilir ve `derivation`'da adım adım yazılır. Bir el örneği motorla uyuşmazsa önce kural yeniden okunur; **beklenen değer motor çıktısına bakarak düzeltilmez.** Gözden geçirme döngüsünün zorunlu maddesi: motoru yazan dışında bir gözden geçiren on beş örneği kuraldan yeniden değerlendirir ve `REVIEW.md`'deki tabloya adını yazar (Task 7 Step 5).
- Kapılar: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (1.100), `cargo test --workspace`, `pnpm --dir app test`, `pnpm --dir app build` her görev sonunda yeşil; `pnpm --dir app e2e` Task 6 ve 7'de. `tsc` `strict`. `pnpm build` çıktısında aranan satır `✓ built`'tir (D24).
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez). Node 24+, pnpm 11.8.0. Depo kısa bir yolda durmalıdır: Swiss Ephemeris 242 bayttan uzun ephemeris yolunu reddeder (çalışma ağacı uzun bir yoldaysa `/tmp` altında kısa bir yola klonlanır).
- **Çalışan `pnpm tauri dev` durdurulmaz** (port 1420); e2e 1421'i kullanır. `pkill -f` kullanılmaz. Üretilen `.app` çalıştırılmaz. `~/Library/Application Support/io.github.turgaydogusan.jyotishdesk` dizinine dokunulmaz.
- Yalnızca macOS hedeflenir. Yorum metni ve AI paneli bu planın dışındadır. Uyum ve sonrası (3.6+) bu planın dışındadır.
- **Tabanlar (`9362280`):** `jyotish-core` lib 239, golden 8, `modules_golden` 9; `store` lib 86 (+1 `#[ignore]`); `jyotish-desk` lib 46, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `Test Files  72 passed (72)`, `Tests  677 passed (677)`; e2e 16; görsel 10 (yerel, darwin); yayın betiği testleri 14.
- **Sayaç zinciri** (her satır görev bitince; boş hücre "önceki satırla aynı"). Bu plan yazılırken `9362280`'in bir kopyasında her görev ayrı commit olarak uygulanıp ölçülmüştür.

  | Görev | `jyotish-core` lib | `modules_golden` | `jyotish-desk` `modules` | Vitest dosya / test | e2e |
  |---|---|---|---|---|---|
  | Taban | 239 | 9 | 1 | 72 / 677 | 16 |
  | Task 1 | 259 | | | | |
  | Task 2 | 267 | | | | |
  | Task 3 | 276 | 11 | | | |
  | Task 4 | | | 1 | 72 / 677 | |
  | Task 5 | | | | 73 / 689 | |
  | Task 6 | | | | | 18 |

  Değişmeyenler: `jyotish-core` golden 8; `store` lib 86 (+1 ign.); `jyotish-desk` lib 46, `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2; görsel 10; yayın betiği testleri 14. `jyotish-core` lib Task 1'de 20 birim testi (`mod.rs` 4, `luminaries.rs` 9, `placement.rs` 7), Task 2'de 8 test (`lords.rs`), Task 3'te 9 test (`mod.rs` 3: rapor ve toplayıcı; proptest 6) kazanır. `modules_golden` iki test kazanır (`yogas_hand_fixtures`, `yogas_fixtures_cover_every_rule`). `jyotish-desk` `modules` tek test olarak durur (süreçte tek SE örneği); Task 4 ona `yogas_checks`'i ekler. Task 4'te Vitest sayısı değişmez (`api.test.ts`'in tablosu büyür). Task 5 bir dosya ve 12 test ekler (`YogasPanel.test.tsx`). Task 6 iki e2e testi ekler.
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir:**
  - **C1** Gizlilik (spec §14): danışan adları, doğum verisi, not gövdeleri, yer bilgisi, dosya yolları ve `CommandError.params` loglanmaz; `no-console.test.ts` durur ve izin listesi büyümez. `ChartInput` ve `inputKey` doğum verisinden türer: loglanmaz, hata parametresine girmez. Hiçbir modül ağ isteği yapmaz; `YogaReport` veritabanına yazılmaz. Yeni hata kodu yoktur.
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu ya da adres almaz. Yeni komutun tek bağımsız değişkeni `ChartInput`'tur; `api.test.ts`'in dizgi bağımsız değişken listesi değişmez.
  - **C3** Göç V1 donmuştur; `crates/store/src/schema.rs` değişmez. `CalcSettings`, `AppPrefs` ve `ViewState` tipleri ve şema sürümleri değişmez.
  - **C4** Arayüz astrolojik hesap yapmaz: kayıtlar, koşullar, sayaçlar ve yaklaşık işareti Rust'tan gelir. Gruplama, süzme, adların çevirisi ve burç vurgusunun lagnadan sayılan eve eşlenmesi gösterimdir (ikincisi Plan 17'nin Bhava bala panelindeki eşlemeyle aynıdır).
  - **C5/C17** Otomatik kayıtlar ve boşaltılmaları değişmez: panel ayarı düzenle birlikte, mevcut debounce ve `save_view_state` kaydıyla yazılır. **C6** Düzen ayrıştırıcıları (`parseLayoutState`, Rust `check_panel_layout`) değişmez; yeni panel türü kataloğa eklenir.
  - **C7** Kısayollar yalnızca `shortcuts.ts`'tedir; bu plan kısayol eklemez (K12). **C8** Her pencere `shell/Dialog`'u kullanır (bu plan pencere eklemez).
  - **C9** Renk yalnızca token'lardan gelir (hex yok) ve tek başına anlam taşımaz: yaklaşık kayıt "≈" işareti ve "yaklaşık" sözüyle, vurgu zemin ve yazı ağırlığıyla, sabitleme `aria-pressed` ile gösterilir.
  - **C10** Üretilen dosyalar elle düzenlenmez: `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` Task 4'te komutla yeniden üretilir.
  - **C11** Görsel PNG temelleri yereldir; bu planda değişmeleri beklenmez (D24).
  - **C12** Her `CommandError.code` üç dilde çevrilidir; bu plan kod eklemez. Her yeni i18n anahtarı üç dilde vardır (`i18n.test.ts` anahtar kümelerini ve yer tutucuları karşılaştırır).
  - **C13** Yasaklı ad taraması her commit'te; depo dışındaki ilerleme notları ve `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** Plan 07'nin gün doğumu toleransı değişmez (bu plan gün doğumu okumaz). **C15** `alert`, `confirm`, `prompt`, `window.print()` ve `window.open()` çağrılmaz. **C16** ⌘ kısayolları değişmez.
- **Retro** her yerde düz `R`'dir ve Rahu ile Ketu'da gösterilmez (UI spec §2.3). Yoga paneli retro göstermez; Rahu ve Ketu hiçbir yoga kaydında yer almaz (D5).
- **Erişilebilirlik:** WCAG AA. Tablolar `caption` (grup adı), `scope="col"` ve `scope="row"` taşır; süzgecin görünür etiketi vardır; "≈" işaretinin sözcük karşılığı ekran okuyucuya okunur; kaydırma kabı klavye odağı alır ve adlıdır; yükleme iskeleti `aria-busy`, hata `role="alert"` ve "Yeniden dene" düğmesi taşır; "Yeniden dene" hata kutusunu değiştirirken odak panelde durur; graha ve ev adları ortak vurgu düğmesidir (`PinButton`, `aria-pressed`).
- Commit mesajları İngilizcedir ve deponun `Co-Authored-By:` satırıyla biter (`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`). Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, `git tag`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Kod blokları `9362280` ağacına göre yazılmış ve bu plan yazılırken deponun bir kopyasında koşulmuştur (Self-Review madde 5). Yeni dosyalar tam içerikle verilir ("**Yeni dosya:**"). Değişen dosyalar `diff` bloğuyla verilir: blok bir dosyaya konup depo kökünde `git apply <dosya>` ile ya da elle uygulanır; bağlam tutmazsa önceki görevin adımı denetlenir, elle uydurma yapılmaz. Bir dosyanın test ve uygulama değişiklikleri ayrı adımlarda ayrı bloklardır. Dosya ve diff blokları dört ters tırnakla çevrilidir (içlerinde üç ters tırnak geçebilir); komut blokları üç ters tırnaklıdır. `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` diff olarak verilmez; adımlardaki komutlar üretir.

## Sonraki alt projelere verilen API

Bu planın ürettiği ve sonraki işlerin tüketeceği adlar. Rust adları `jyotish_core`'dan dışa aktarılır; imzalar görevlerde tam olarak verilir.

| API | Tüketen | Not |
|---|---|---|
| `yogas(positions) -> YogaReport`, `YogaReport`, `YogaHit`, `YogaGroup`, `YogaRelation`, `YOGA_CATALOG_VERSION` | AI paketleyicisi; 3.9'un yoga koşulları (`hits[].id`) | Lagna yoksa 17 kural atlanır |
| `compute_yogas(eph, input)`; Tauri komutu `compute_yogas` | Arayüz; AI paneli | Her girdide rapor |
| `YOGA_CATALOG`, `YogaRule { id, group, needs_lagna }`, `yoga_rule(id)`, `YogaRule::matches(positions)`, `YogaMatch` | 3.9 `chart_search` (koşul seçici: kimlik ve grup listesi; tek kuralın ucuz değerlendirmesi) | Kimlikler kararlıdır |
| `yoga_relations(a, b, positions)`, `is_exchange`, `is_mutual_aspect` | 3.6 uyum (karşılıklı yerleşim), 3.8 Prashna olguları | Burç temelli; düğümler ilişki kurmaz |
| `exchange_kind(a, b)`, `ExchangeKind`, `occupied_sign_count(positions)` | 3.9 arama koşulları | |
| `KENDRA_HOUSES`, `TRIKONA_HOUSES`, `DUSTHANA_HOUSES`, `UPACHAYA_HOUSES`, `DHANA_HOUSES`, `COUNTED_GRAHAS`, `YOGA_GRAHAS`, `YOGA_CONDITIONS` | 3.6, 3.8, 3.9 | Ev kümeleri tek yerde |
| Arayüz: `modules/useYogas.ts`; `panels/YogasPanel.tsx` (`YOGA_GROUPS`, `yogaGroupFilter`); fikstürde `yogas`, `yogas_time_unknown`, `yoga_catalog`, `yoga_conditions` | 3.9 arama yüzeyi (yoga seçici, adlar) | i18n: `yogas.name.*`, `yogas.group.*` |

## Dosya Yapısı (bu plan)

```
crates/jyotish-core/
  src/yogas/mod.rs                tipler, ilişkiler, katalog, rapor, compute_yogas (yeni)
  src/yogas/luminaries.rs         Ay ve Güneş yogaları (yeni)
  src/yogas/placement.rs          mahapurusha, neecha bhanga, nabhasa, kuja dosha (yeni)
  src/yogas/lords.rs              raja, dhana, değişim, viparita raja (yeni)
  src/lib.rs                      modül kaydı ve dışa aktarımlar
  src/proptests.rs                yoga_properties
  tests/modules_golden.rs         yoga fikstür düzeneği ve kapsam testi
tests/golden/modules/
  README.md                       yogas satırı
  yogas/REVIEW.md                 okuma kuralları ve gözden geçirme kaydı (yeni)
  yogas/*.json                    15 el örneği (yeni)
app/src-tauri/
  src/module_commands.rs          compute_yogas
  src/lib.rs                      komut kaydı
  tests/modules.rs                yoga entegrasyonu, v1 çapraz denetimi, bütçe
  tests/ui_fixture.rs             fikstüre yogas, yogas_time_unknown, yoga_catalog, yoga_conditions
app/ui/
  bindings.ts                     üretilir
  test/fixtures/delhi.json        üretilir
  test/fixture.ts                 UiFixture'ın dört yeni alanı
  api.ts, api.test.ts             computeYogas
  modules/useYogas.ts             kanca (yeni)
  panels/YogasPanel.tsx (+test)   (yeni)
  panels/registry.ts, panels.test.tsx        panel türü
  panels/panels.css
  layout/model.ts (+test)         PANEL_KINDS, MODULE_PANELS
  i18n/{tr,en,fr}.ts              panel.yogas, yogas
app/e2e/{ipc.ts,desk.spec.ts}     komutun taklidi; panelin akışı
```

### Kapsam

| Spec maddesi | Yer |
|---|---|
| §8 Ortak tanımlar: birlikte, karşılıklı bakış, değişim, ilişkili; sayılan grahalar | Task 1 (`yoga_relations`, `COUNTED_GRAHAS`; D6, D8) |
| §8 Pancha Mahapurusha | Task 1 (`placement::mahapurusha`; D7) |
| §8 Ay: Sunapha, Anapha, Durudhara, Kemadruma, Gajakesari, Chandra-Mangala, Adhi, Shakata, Amala, Vasumati | Task 1 (`luminaries.rs`; D6) |
| §8 Güneş: Vesi, Vasi, Ubhayachari, Budha-Aditya | Task 1 (`luminaries.rs`) |
| §8 Neecha bhanga | Task 1 (`placement::neecha_bhanga`; D13) |
| §8 Nabhasa: sayı ve ashraya | Task 1 (`placement::nabhasa_count`, `nabhasa_ashraya`; D14) |
| §8 Dosha: Kuja dosha | Task 1 (`placement::kuja`) |
| §8 Raja: kendra–trikona, Dharma-Karmadhipati, yogakaraka | Task 2 (`lords.rs`; D9) |
| §8 Dhana | Task 2 (D10) |
| §8 Değişim: Maha, 3. ev lordu, Dainya | Task 2 (`exchange_kind`; D11) |
| §8 Viparita raja: Harsha, Sarala, Vimala | Task 2 (D12) |
| §8 Veri modeli; `YOGA_CATALOG_VERSION`; doğum saati bilinmiyorsa | Task 1 (tipler), Task 3 (`yogas`, `YogaHit`, `YogaReport`; D3, D4) |
| §8 Test: kural başına el örneği (oluşan, oluşmayan, iptal) | Task 3 (D15); Task 7 Step 5 |
| §8 Test: property'ler | Task 3 (`yoga_properties`); Task 5 (üç dilde ad) |
| §8 Test: uçtan uca beş golden anı | Task 4 (D19) |
| §8 IPC: `compute_yogas`; K1, K2, K8 | Task 3 (toplayıcı), Task 4 (komut; D18) |
| §8 Arayüz: `yogas` paneli (grup süzgeci, liste, vurgu, boş durum) | Task 5 (D20–D23) |
| §13 Bağımsız ikinci yazım | Task 1, 2 (D16) |
| §13 Komut katmanı; §15 bütçe (≤ 10 ms) | Task 4 (D18) |
| §13 Panel durumları: boş, yükleniyor, hata, dolu, "saat bilinmiyor" | Task 5 |
| §14 Gizlilik | C1; Task 7 taramaları |
| K12 Paletten ekleme | Task 5; Task 6 |
| Uçtan uca akış | Task 6 |
| Son doğrulama, elle denetim, el örneklerinin gözden geçirilmesi | Task 7 |

---


### Task 1: Çekirdek: katalog tipleri, ilişkiler; mahapurusha, Ay ve Güneş yogaları, neecha bhanga, nabhasa ve kuja dosha

Spec §8 "Kapsam ve yöntem" (tablonun Pancha Mahapurusha, Ay, Güneş, Neecha bhanga, Nabhasa ve Dosha satırları; ortak tanımlar), "Veri modeli" (`YogaGroup`, ilişki tipi); §13 katman 1 (kural düzeyinde el örnekleri) ve katman 3 (elle yazılmış tablolar); D1, D2, D5–D8, D13, D14, D16. Hiçbir mevcut davranış değişmez; yalnızca yeni modül eklenir. Lord kuralları Task 2'dedir; rapor (`yogas`), toplayıcı, el fikstürleri ve değişmezler Task 3'tedir. Bu görevin sonunda katalog 31 kural taşır.

**Files:**
- Create: `crates/jyotish-core/src/yogas/mod.rs`, `crates/jyotish-core/src/yogas/luminaries.rs`, `crates/jyotish-core/src/yogas/placement.rs`
- Modify: `crates/jyotish-core/src/lib.rs`
- Test: üç dosyanın `mod tests`'i (20 test)

**Interfaces:**
- Consumes (v1, Plan 14, Plan 15): `Positions` (`sign_of`, `house_of`, `house_from`, `occupants`, `synthetic`, `without_lagna`), `Graha`, `Sign`, `dignity::{own_signs, exaltation_point, debilitation_point, sign_lord}`, `drishti::{graha_aspect_signs, ASPECTING_GRAHAS}`, `lordship::house_sign`, `nature::{nature, kuja_dosha, Nature}`, `jaimini::{sign_kind, SignKind}`.
- Produces (yıldızlılar `jyotish_core` kökünden de dışa aktarılır):
  - Tipler: `YogaGroup`\* (11 değer; `YogaGroup::ALL`), `YogaRelation`\* (`Conjunction`, `MutualAspect`, `Exchange`), `YogaMatch { grahas, houses, relations, conditions }`\*, `YogaRule { id, group, needs_lagna, .. }`\* (`matches(&Positions) -> Vec<YogaMatch>`).
  - Sabitler: `YOGA_CATALOG: &[YogaRule]`\*, `YOGA_CATALOG_VERSION: u32`\*, `YOGA_CONDITIONS: [&str; 8]`\*, `COND_OWN_SIGN` … `COND_DISPOSITOR_ASPECT`, `KENDRA_HOUSES`\*, `TRIKONA_HOUSES`\*, `DUSTHANA_HOUSES`\*, `UPACHAYA_HOUSES`\*, `YOGA_GRAHAS: [Graha; 7]`\*, `COUNTED_GRAHAS: [Graha; 5]`\*.
  - Fonksiyonlar: `yoga_rule(id) -> Option<&'static YogaRule>`\*, `is_exchange(a, b, &Positions) -> bool`\*, `is_mutual_aspect(a, b, &Positions) -> bool`\*, `yoga_relations(a, b, &Positions) -> Vec<YogaRelation>`\*, `occupied_sign_count(&Positions) -> u8`\*; modül içi yardımcılar `occupied_houses`, `in_graha_order`, `in_houses_from`, `placement::exalted_in`; test yardımcıları `testkit::{chart, matches, forms}`.

- [ ] **Step 1: Tabanları ölç**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
git log -1 --format=%h
git status --short
rustc --version
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `git status --short` boş; `rustc 1.100.0` ya da yenisi; `jyotish-core` lib `239 passed`, golden `8 passed`, `modules_golden` `9 passed`; `store` lib `86 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `46 passed`, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `72 passed (72)`, `677 passed (677)`. Commit `9362280` değilse ölçülen sayılar ilerleme notlarına taban olarak yazılır ve zincir oradan yürütülür (farklar aynıdır). (Vitest'te `NotesPanel.test.tsx › New note adds an empty note…` yük altında seyrek olarak düşebilir; bu plandan bağımsızdır, yeniden koşulur ve ilerleme notlarına yazılır.)

- [ ] **Step 2: Modülü kaydet (derleme başarısız olur)**

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 747d7f6..cfb92c1 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -5,7 +5,7 @@
 //! Vimshottari dasha, panchang, ashtakavarga ve transit (M3). İleri modüllerin ortak temeli
 //! (Alt proje 3.1): `positions` (ortak girdi), `lordship`, `nature` ve `drishti`. Jaimini
 //! çekirdeği (Alt proje 3.2) ve chara dasha (Alt proje 3.3): `jaimini`. Shadbala ve Bhava bala
-//! (Alt proje 3.4): `shadbala`.
+//! (Alt proje 3.4): `shadbala`. Yogalar (Alt proje 3.5): `yogas`.
 
 pub mod ashtakavarga;
 mod chart;
@@ -29,6 +29,7 @@ pub mod settings;
 pub mod shadbala;
 pub mod transit;
 pub mod varga;
+pub mod yogas;
 pub mod zodiac;
 
 pub use ashtakavarga::{
@@ -90,4 +91,10 @@ pub use transit::{compute_transits, TransitGraha, TransitSnapshot};
 pub use varga::{
     all_vargas, is_vargottama, varga_chart, varga_position, Varga, VargaChart, VargaPlacement,
 };
+pub use yogas::{
+    is_exchange, is_mutual_aspect, occupied_sign_count, yoga_relations, yoga_rule, YogaGroup,
+    YogaMatch, YogaRelation, YogaRule, COUNTED_GRAHAS, DUSTHANA_HOUSES, KENDRA_HOUSES,
+    TRIKONA_HOUSES, UPACHAYA_HOUSES, YOGA_CATALOG, YOGA_CATALOG_VERSION, YOGA_CONDITIONS,
+    YOGA_GRAHAS,
+};
 pub use zodiac::{angular_distance, deg_in_sign, nakshatra_pada, normalize, Sign};
````

- [ ] **Step 3: Derlemenin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-core --lib yogas 2>&1 | grep -E "^error"
```
Expected: `error[E0583]: file not found for module `yogas`` ve `could not compile`. Sonraki üç adım modülü yazar; her dosyanın `mod tests` bölümü o dosyadaki kuralların el örnekleridir (kuraldan elle türetilmiştir; her beklenen değerin gerekçesi yanındaki yorumdadır). Dosyayı yazarken önce testleri okuyun: bir test düşerse önce kural (spec §8 ve ilgili D-kararı) yeniden okunur, beklenen değer motor çıktısına bakarak değiştirilmez.

- [ ] **Step 4: Tipler, ilişkiler ve katalog**

Katalog bu görevde 31 satırdır: 5 mahapurusha, 10 Ay, 4 Güneş, 1 neecha bhanga, 7 sayı, 3 ashraya, 1 dosha. Satırların yüklemleri sonraki iki adımın dosyalarındadır. `testkit::chart` burç ortasında duran sentetik konumlar kurar (burç dizini: Koç 0 … Balık 11).

**Yeni dosya:** `crates/jyotish-core/src/yogas/mod.rs`

````rust
//! Yogalar (ileri modüller spec'i §8): veriyle tanımlı kural kataloğu. Her kural konumlardan
//! eşleşme listesi üreten saf bir yüklemdir; sonuç bir olgu listesidir (hangi yoga, hangi
//! grahalar, hangi evler). Güç derecelendirmesi ve hüküm yoktur; yalnızca D1 değerlendirilir.
//!
//! Kaynak: BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana
//! yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; kurallar özgün olarak uygulanır.
//! Evler rasi evidir (K4), burçlar rasi çerçevesindedir (K5), doğal sınıf `nature`'dır (K6).
//! Lagnası olmayan konumlarda (doğum saati bilinmiyor) lagna isteyen kurallar değerlendirilmez.

mod luminaries;
mod placement;

pub use placement::occupied_sign_count;

use serde::{Deserialize, Serialize};

use crate::dignity::own_signs;
use crate::drishti::{graha_aspect_signs, ASPECTING_GRAHAS};
use crate::graha::Graha;
use crate::positions::Positions;
use crate::zodiac::Sign;

/// Kural kataloğunun sürümü: bir kural eklenince, çıkınca ya da koşulu değişince artar.
pub const YOGA_CATALOG_VERSION: u32 = 1;

/// Kendra evleri.
pub const KENDRA_HOUSES: [u8; 4] = [1, 4, 7, 10];
/// Trikona evleri.
pub const TRIKONA_HOUSES: [u8; 3] = [1, 5, 9];
/// 6., 8. ve 12. ev.
pub const DUSTHANA_HOUSES: [u8; 3] = [6, 8, 12];
/// Upachaya evleri.
pub const UPACHAYA_HOUSES: [u8; 4] = [3, 6, 10, 11];
/// Yedi graha (düğümler dışında): lord olabilenler ve nabhasa yogalarında sayılanlar.
pub const YOGA_GRAHAS: [Graha; 7] = ASPECTING_GRAHAS;
/// Ay ve Güneş yogalarında sayılan grahalar.
pub const COUNTED_GRAHAS: [Graha; 5] = [
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// Alt koşul kimlikleri (`YogaHit::conditions`). Görünen adları i18n anahtarıdır.
pub const COND_OWN_SIGN: &str = "own_sign";
pub const COND_EXALTATION: &str = "exaltation";
pub const COND_FROM_LAGNA: &str = "from_lagna";
pub const COND_FROM_MOON: &str = "from_moon";
pub const COND_FROM_VENUS: &str = "from_venus";
pub const COND_SIGN_LORD_IN_KENDRA: &str = "sign_lord_in_kendra";
pub const COND_EXALTED_GRAHA_IN_KENDRA: &str = "exalted_graha_in_kendra";
pub const COND_DISPOSITOR_ASPECT: &str = "dispositor_aspect";
/// Bütün alt koşul kimlikleri.
pub const YOGA_CONDITIONS: [&str; 8] = [
    COND_OWN_SIGN,
    COND_EXALTATION,
    COND_FROM_LAGNA,
    COND_FROM_MOON,
    COND_FROM_VENUS,
    COND_SIGN_LORD_IN_KENDRA,
    COND_EXALTED_GRAHA_IN_KENDRA,
    COND_DISPOSITOR_ASPECT,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum YogaGroup {
    Mahapurusha,
    Lunar,
    Solar,
    Raja,
    Dhana,
    Exchange,
    ViparitaRaja,
    NeechaBhanga,
    NabhasaCount,
    NabhasaAshraya,
    Dosha,
}

impl YogaGroup {
    /// Katalog sırası.
    pub const ALL: [YogaGroup; 11] = [
        YogaGroup::Mahapurusha,
        YogaGroup::Lunar,
        YogaGroup::Solar,
        YogaGroup::Raja,
        YogaGroup::Dhana,
        YogaGroup::Exchange,
        YogaGroup::ViparitaRaja,
        YogaGroup::NeechaBhanga,
        YogaGroup::NabhasaCount,
        YogaGroup::NabhasaAshraya,
        YogaGroup::Dosha,
    ];
}

/// İki grahanın ilişkisi (spec §8 "ilişkili"). v1'deki `dignity::Relation` (doğal dostluk)
/// başka bir kavramdır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum YogaRelation {
    /// Aynı burç.
    Conjunction,
    /// İki grahanın birbirine burç temelli tam bakışı.
    MutualAspect,
    /// İki grahanın birbirinin burcunda olması (parivartana).
    Exchange,
}

/// Bir kuralın tek eşleşmesi: kural kimliği ve grubu katalogdan gelir.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct YogaMatch {
    /// `Graha::ALL` sırasında, yinelemesiz.
    pub grahas: Vec<Graha>,
    /// Lagnadan sayılan evler, artan sırada, yinelemesiz; lagna yoksa boş.
    pub houses: Vec<u8>,
    pub relations: Vec<YogaRelation>,
    pub conditions: Vec<&'static str>,
}

/// Katalog satırı.
#[derive(Debug, Clone, Copy)]
pub struct YogaRule {
    /// Kararlı kimlik: `"<grup>.<ad>"`.
    pub id: &'static str,
    pub group: YogaGroup,
    /// Kural lagnasız değerlendirilemez (doğum saati bilinmiyorsa atlanır).
    pub needs_lagna: bool,
    eval: fn(&Positions) -> Vec<YogaMatch>,
}

impl YogaRule {
    /// Kuralın bu konumlardaki eşleşmeleri. Lagna isteyen kural lagnasız konumda boş döner.
    pub fn matches(&self, positions: &Positions) -> Vec<YogaMatch> {
        if self.needs_lagna && positions.lagna_sign.is_none() {
            return Vec::new();
        }
        (self.eval)(positions)
    }
}

const fn rule(
    id: &'static str,
    group: YogaGroup,
    needs_lagna: bool,
    eval: fn(&Positions) -> Vec<YogaMatch>,
) -> YogaRule {
    YogaRule {
        id,
        group,
        needs_lagna,
        eval,
    }
}

/// Kural kataloğu, spec §8 tablosunun sırasıyla. Rapor bu sırayla dolar.
pub static YOGA_CATALOG: &[YogaRule] = &[
    rule("mahapurusha.ruchaka", YogaGroup::Mahapurusha, true, |p| {
        placement::mahapurusha(p, Graha::Mars)
    }),
    rule("mahapurusha.bhadra", YogaGroup::Mahapurusha, true, |p| {
        placement::mahapurusha(p, Graha::Mercury)
    }),
    rule("mahapurusha.hamsa", YogaGroup::Mahapurusha, true, |p| {
        placement::mahapurusha(p, Graha::Jupiter)
    }),
    rule("mahapurusha.malavya", YogaGroup::Mahapurusha, true, |p| {
        placement::mahapurusha(p, Graha::Venus)
    }),
    rule("mahapurusha.shasha", YogaGroup::Mahapurusha, true, |p| {
        placement::mahapurusha(p, Graha::Saturn)
    }),
    rule("lunar.sunapha", YogaGroup::Lunar, false, |p| {
        luminaries::flank(p, Graha::Moon, luminaries::Flank::SecondOnly)
    }),
    rule("lunar.anapha", YogaGroup::Lunar, false, |p| {
        luminaries::flank(p, Graha::Moon, luminaries::Flank::TwelfthOnly)
    }),
    rule("lunar.durudhara", YogaGroup::Lunar, false, |p| {
        luminaries::flank(p, Graha::Moon, luminaries::Flank::Both)
    }),
    rule(
        "lunar.kemadruma",
        YogaGroup::Lunar,
        true,
        luminaries::kemadruma,
    ),
    rule(
        "lunar.gajakesari",
        YogaGroup::Lunar,
        false,
        luminaries::gajakesari,
    ),
    rule(
        "lunar.chandra_mangala",
        YogaGroup::Lunar,
        false,
        luminaries::chandra_mangala,
    ),
    rule("lunar.adhi", YogaGroup::Lunar, false, luminaries::adhi),
    rule("lunar.shakata", YogaGroup::Lunar, true, luminaries::shakata),
    rule("lunar.amala", YogaGroup::Lunar, false, luminaries::amala),
    rule(
        "lunar.vasumati",
        YogaGroup::Lunar,
        false,
        luminaries::vasumati,
    ),
    rule("solar.vesi", YogaGroup::Solar, false, |p| {
        luminaries::flank(p, Graha::Sun, luminaries::Flank::SecondOnly)
    }),
    rule("solar.vasi", YogaGroup::Solar, false, |p| {
        luminaries::flank(p, Graha::Sun, luminaries::Flank::TwelfthOnly)
    }),
    rule("solar.ubhayachari", YogaGroup::Solar, false, |p| {
        luminaries::flank(p, Graha::Sun, luminaries::Flank::Both)
    }),
    rule(
        "solar.budha_aditya",
        YogaGroup::Solar,
        false,
        luminaries::budha_aditya,
    ),
    rule(
        "neecha_bhanga.neecha_bhanga",
        YogaGroup::NeechaBhanga,
        false,
        placement::neecha_bhanga,
    ),
    rule(
        "nabhasa_count.vallaki",
        YogaGroup::NabhasaCount,
        false,
        |p| placement::nabhasa_count(p, 7),
    ),
    rule("nabhasa_count.dama", YogaGroup::NabhasaCount, false, |p| {
        placement::nabhasa_count(p, 6)
    }),
    rule("nabhasa_count.pasha", YogaGroup::NabhasaCount, false, |p| {
        placement::nabhasa_count(p, 5)
    }),
    rule(
        "nabhasa_count.kedara",
        YogaGroup::NabhasaCount,
        false,
        |p| placement::nabhasa_count(p, 4),
    ),
    rule("nabhasa_count.shula", YogaGroup::NabhasaCount, false, |p| {
        placement::nabhasa_count(p, 3)
    }),
    rule("nabhasa_count.yuga", YogaGroup::NabhasaCount, false, |p| {
        placement::nabhasa_count(p, 2)
    }),
    rule("nabhasa_count.gola", YogaGroup::NabhasaCount, false, |p| {
        placement::nabhasa_count(p, 1)
    }),
    rule(
        "nabhasa_ashraya.rajju",
        YogaGroup::NabhasaAshraya,
        false,
        |p| placement::nabhasa_ashraya(p, crate::jaimini::SignKind::Movable),
    ),
    rule(
        "nabhasa_ashraya.musala",
        YogaGroup::NabhasaAshraya,
        false,
        |p| placement::nabhasa_ashraya(p, crate::jaimini::SignKind::Fixed),
    ),
    rule(
        "nabhasa_ashraya.nala",
        YogaGroup::NabhasaAshraya,
        false,
        |p| placement::nabhasa_ashraya(p, crate::jaimini::SignKind::Dual),
    ),
    rule("dosha.kuja", YogaGroup::Dosha, false, placement::kuja),
];

/// Katalogdaki kural; kimlik bilinmiyorsa `None`.
pub fn yoga_rule(id: &str) -> Option<&'static YogaRule> {
    YOGA_CATALOG.iter().find(|r| r.id == id)
}

/// "Değişim" (parivartana): iki farklı graha birbirinin burcundadır.
pub fn is_exchange(a: Graha, b: Graha, positions: &Positions) -> bool {
    a != b
        && own_signs(b).contains(&positions.sign_of(a))
        && own_signs(a).contains(&positions.sign_of(b))
}

/// "Karşılıklı bakış": iki graha birbirinin burcuna tam bakışla bakar (burç temelli graha
/// drishti). Düğümler bakış atmaz.
pub fn is_mutual_aspect(a: Graha, b: Graha, positions: &Positions) -> bool {
    let (sa, sb) = (positions.sign_of(a), positions.sign_of(b));
    a != b && graha_aspect_signs(a, sa).contains(&sb) && graha_aspect_signs(b, sb).contains(&sa)
}

/// İki grahanın taşıdığı ilişkiler, `YogaRelation` sırasında. Boş liste: ilişkili değiller.
pub fn yoga_relations(a: Graha, b: Graha, positions: &Positions) -> Vec<YogaRelation> {
    let mut out = Vec::new();
    if a != b && positions.sign_of(a) == positions.sign_of(b) {
        out.push(YogaRelation::Conjunction);
    }
    if is_mutual_aspect(a, b, positions) {
        out.push(YogaRelation::MutualAspect);
    }
    if is_exchange(a, b, positions) {
        out.push(YogaRelation::Exchange);
    }
    out
}

/// Grahaların lagnadan sayılan evleri: artan sırada, yinelemesiz. Lagna yoksa boş.
pub(crate) fn occupied_houses(positions: &Positions, grahas: &[Graha]) -> Vec<u8> {
    let mut houses: Vec<u8> = grahas
        .iter()
        .filter_map(|&g| positions.house_of(g))
        .collect();
    houses.sort_unstable();
    houses.dedup();
    houses
}

/// `grahas` `Graha::ALL` sırasına dizilir ve yinelemeler atılır.
pub(crate) fn in_graha_order(mut grahas: Vec<Graha>) -> Vec<Graha> {
    grahas.sort_unstable();
    grahas.dedup();
    grahas
}

/// `graha`, `reference` burcundan sayılan `houses` evlerinden birinde mi.
pub(crate) fn in_houses_from(
    positions: &Positions,
    graha: Graha,
    reference: Sign,
    houses: &[u8],
) -> bool {
    houses.contains(&positions.house_from(graha, reference))
}

#[cfg(test)]
pub(crate) mod testkit {
    use super::*;

    /// Burç ortasında duran sentetik konumlar: `signs`, `Graha::ALL` sırasında burç dizinidir
    /// (Koç 0 … Balık 11). Ay'ın büyüyüp küçülmesi burçların sırasından çıkar.
    pub(crate) fn chart(lagna: Option<usize>, signs: [usize; 9]) -> Positions {
        Positions::synthetic(
            lagna.map(|s| s as f64 * 30.0 + 15.0),
            signs.map(|s| s as f64 * 30.0 + 15.0),
        )
    }

    /// Kuralın eşleşmeleri; kimlik katalogda yoksa test düşer.
    pub(crate) fn matches(id: &str, positions: &Positions) -> Vec<YogaMatch> {
        yoga_rule(id)
            .unwrap_or_else(|| panic!("{id} is not in the catalog"))
            .matches(positions)
    }

    pub(crate) fn forms(id: &str, positions: &Positions) -> bool {
        !matches(id, positions).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::testkit::chart;
    use super::*;
    use Graha::*;

    /// Elle yazılmış tablo (bağımsız ikinci yazım): `a` grahası `b`'den kaç burç ilerideyken
    /// (0 = aynı burç) karşılıklı tam bakış vardır. 7. burç bakışı her çiftte karşılıklıdır;
    /// özel bakışlardan yalnızca Mars'ın 4. ve Satürn'ün 10. bakışı birbirini karşılar
    /// (Satürn Mars'tan 4. burçtayken Mars da Satürn'den 10. burçtadır).
    fn hand_mutual(a: Graha, b: Graha, a_ahead_of_b: usize) -> bool {
        if a.is_node() || b.is_node() || a == b {
            return false;
        }
        matches!(
            (a, b, a_ahead_of_b),
            (_, _, 6) | (Saturn, Mars, 3) | (Mars, Saturn, 9)
        )
    }

    #[test]
    fn mutual_aspect_matches_the_hand_table_for_every_pair_and_offset() {
        for a in Graha::ALL {
            for b in Graha::ALL {
                for offset in 0..12usize {
                    let mut signs = [0usize; 9];
                    // Diğer grahalar sonucu etkilemez; `b` Yengeç'te, `a` ondan `offset` ileride.
                    signs[b as usize] = 3;
                    signs[a as usize] = 3 + offset;
                    if a == b {
                        continue;
                    }
                    let p = chart(None, signs.map(|s| s % 12));
                    assert_eq!(
                        is_mutual_aspect(a, b, &p),
                        hand_mutual(a, b, offset),
                        "{a:?} {offset} signs ahead of {b:?}"
                    );
                    assert_eq!(is_mutual_aspect(a, b, &p), is_mutual_aspect(b, a, &p));
                }
            }
        }
    }

    #[test]
    fn exchange_needs_each_graha_in_a_sign_of_the_other() {
        // Mars Boğa'da, Venüs Koç'ta: değişim. Diğerleri Yengeç'te.
        let p = chart(None, [3, 3, 1, 3, 3, 0, 3, 3, 3]);
        assert!(is_exchange(Mars, Venus, &p) && is_exchange(Venus, Mars, &p));
        assert_eq!(yoga_relations(Mars, Venus, &p), [YogaRelation::Exchange]);
        // Mars Terazi'de, Venüs Koç'ta: değişim ve karşılıklı 7. bakış birlikte.
        let p = chart(None, [3, 3, 6, 3, 3, 0, 3, 3, 3]);
        assert_eq!(
            yoga_relations(Mars, Venus, &p),
            [YogaRelation::MutualAspect, YogaRelation::Exchange]
        );
        // Satürn Koç'ta, Mars Oğlak'ta: değişim; Koç Oğlak'tan 4. burçtur (Mars'ın 4. bakışı),
        // Oğlak Koç'tan 10. burçtur (Satürn'ün 10. bakışı).
        let p = chart(None, [3, 3, 9, 3, 3, 3, 0, 3, 3]);
        assert_eq!(
            yoga_relations(Mars, Saturn, &p),
            [YogaRelation::MutualAspect, YogaRelation::Exchange]
        );
        // Yalnızca biri diğerinin burcunda: değişim değil.
        let p = chart(None, [3, 3, 1, 3, 3, 5, 3, 3, 3]);
        assert!(!is_exchange(Mars, Venus, &p));
        assert!(yoga_relations(Mars, Venus, &p).is_empty());
        // Aynı burç: birlikte. Bir graha kendisiyle ilişkili sayılmaz.
        let p = chart(None, [4, 3, 1, 4, 3, 5, 3, 3, 3]);
        assert_eq!(
            yoga_relations(Sun, Mercury, &p),
            [YogaRelation::Conjunction]
        );
        assert!(yoga_relations(Sun, Sun, &p).is_empty());
        // Düğümlerin burcu yoktur: değişim kuramazlar.
        assert!(!is_exchange(Rahu, Moon, &p));
    }

    #[test]
    fn catalog_ids_are_unique_grouped_and_prefixed_by_their_group() {
        let prefix = |g: YogaGroup| match g {
            YogaGroup::Mahapurusha => "mahapurusha.",
            YogaGroup::Lunar => "lunar.",
            YogaGroup::Solar => "solar.",
            YogaGroup::Raja => "raja.",
            YogaGroup::Dhana => "dhana.",
            YogaGroup::Exchange => "exchange.",
            YogaGroup::ViparitaRaja => "viparita.",
            YogaGroup::NeechaBhanga => "neecha_bhanga.",
            YogaGroup::NabhasaCount => "nabhasa_count.",
            YogaGroup::NabhasaAshraya => "nabhasa_ashraya.",
            YogaGroup::Dosha => "dosha.",
        };
        let mut ids: Vec<&str> = YOGA_CATALOG.iter().map(|r| r.id).collect();
        for r in YOGA_CATALOG {
            assert!(r.id.starts_with(prefix(r.group)), "{}", r.id);
            assert_eq!(yoga_rule(r.id).map(|x| x.id), Some(r.id));
        }
        // Gruplar katalogda `YogaGroup::ALL` sırasıyla ve bitişik durur.
        let groups: Vec<YogaGroup> = YOGA_CATALOG.iter().map(|r| r.group).collect();
        assert!(groups.windows(2).all(|w| w[0] <= w[1]), "{groups:?}");
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), YOGA_CATALOG.len());
        assert_eq!(yoga_rule("lunar.unknown").map(|r| r.id), None);
        assert_eq!(YOGA_CATALOG_VERSION, 1);
    }

    #[test]
    fn helpers_order_grahas_and_houses() {
        // Koç lagna; Güneş 5. evde, Ay 2. evde, Mars 5. evde.
        let p = chart(Some(0), [4, 1, 4, 0, 0, 0, 0, 0, 0]);
        assert_eq!(occupied_houses(&p, &[Mars, Moon, Sun]), [2, 5]);
        assert!(occupied_houses(&p.clone().without_lagna(), &[Mars, Moon]).is_empty());
        assert_eq!(
            in_graha_order(vec![Venus, Moon, Venus, Sun]),
            [Sun, Moon, Venus]
        );
        assert!(in_houses_from(&p, Sun, Sign::Taurus, &KENDRA_HOUSES));
        assert!(!in_houses_from(&p, Sun, Sign::Taurus, &TRIKONA_HOUSES));
    }
}
````

- [ ] **Step 5: Ay ve Güneş yogaları**

**Yeni dosya:** `crates/jyotish-core/src/yogas/luminaries.rs`

````rust
//! Ay ve Güneş yogaları (spec §8 "Ay" ve "Güneş" satırları). Sayılan grahalar Mars, Merkür,
//! Jüpiter, Venüs ve Satürn'dür (`COUNTED_GRAHAS`); Güneş, Ay ve düğümler sayılmaz. Evler
//! Ay'ın ya da Güneş'in burcundan sayılan rasi evleridir.

use super::{
    in_graha_order, in_houses_from, occupied_houses, YogaMatch, YogaRelation, COND_FROM_LAGNA,
    COND_FROM_MOON, COUNTED_GRAHAS, KENDRA_HOUSES, UPACHAYA_HOUSES,
};
use crate::graha::Graha;
use crate::lordship::house_sign;
use crate::nature::{nature, Nature};
use crate::positions::Positions;
use crate::zodiac::Sign;

/// Bir ışığın (Ay ya da Güneş) iki yanındaki evlerin doluluğu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Flank {
    /// Sayılan graha yalnızca 2. evde.
    SecondOnly,
    /// Sayılan graha yalnızca 12. evde.
    TwelfthOnly,
    /// İki evde de sayılan graha var.
    Both,
}

/// `center`'ın burcundan sayılan `house`. evdeki sayılan grahalar.
fn counted_in(positions: &Positions, center: Graha, house: u8) -> Vec<Graha> {
    let reference = positions.sign_of(center);
    COUNTED_GRAHAS
        .into_iter()
        .filter(|&g| positions.house_from(g, reference) == house)
        .collect()
}

fn listed(positions: &Positions, grahas: Vec<Graha>) -> YogaMatch {
    let grahas = in_graha_order(grahas);
    YogaMatch {
        houses: occupied_houses(positions, &grahas),
        grahas,
        ..YogaMatch::default()
    }
}

/// Sunapha / Anapha / Durudhara (Ay) ve Vesi / Vasi / Ubhayachari (Güneş): üçü birbirini
/// dışlar. Eşleşme ışığı ve iki yandaki sayılan grahaları listeler.
pub(super) fn flank(positions: &Positions, center: Graha, want: Flank) -> Vec<YogaMatch> {
    let second = counted_in(positions, center, 2);
    let twelfth = counted_in(positions, center, 12);
    let found = match (second.is_empty(), twelfth.is_empty()) {
        (false, true) => Flank::SecondOnly,
        (true, false) => Flank::TwelfthOnly,
        (false, false) => Flank::Both,
        (true, true) => return Vec::new(),
    };
    if found != want {
        return Vec::new();
    }
    let mut grahas = vec![center];
    grahas.extend(second);
    grahas.extend(twelfth);
    vec![listed(positions, grahas)]
}

/// Kemadruma: Ay'dan 2. ve 12. evde sayılan graha yok, Ay'ın burcunda sayılan graha yok (Ay
/// yalnız) ve Ay'dan ve lagnadan kendralarda sayılan graha yok. Son koşul iptal kuralıdır;
/// lagna ister.
pub(super) fn kemadruma(positions: &Positions) -> Vec<YogaMatch> {
    let Some(lagna) = positions.lagna_sign else {
        return Vec::new();
    };
    let moon = positions.sign_of(Graha::Moon);
    let near_moon = COUNTED_GRAHAS
        .into_iter()
        .any(|g| in_houses_from(positions, g, moon, &[12, 1, 2]));
    let in_a_kendra = COUNTED_GRAHAS.into_iter().any(|g| {
        in_houses_from(positions, g, moon, &KENDRA_HOUSES)
            || in_houses_from(positions, g, lagna, &KENDRA_HOUSES)
    });
    if near_moon || in_a_kendra {
        return Vec::new();
    }
    vec![listed(positions, vec![Graha::Moon])]
}

/// Gajakesari: Jüpiter Ay'dan kendrada (Ay'la aynı burç dahil).
pub(super) fn gajakesari(positions: &Positions) -> Vec<YogaMatch> {
    let moon = positions.sign_of(Graha::Moon);
    if !in_houses_from(positions, Graha::Jupiter, moon, &KENDRA_HOUSES) {
        return Vec::new();
    }
    vec![listed(positions, vec![Graha::Moon, Graha::Jupiter])]
}

fn together(positions: &Positions, a: Graha, b: Graha) -> Vec<YogaMatch> {
    if positions.sign_of(a) != positions.sign_of(b) {
        return Vec::new();
    }
    vec![YogaMatch {
        relations: vec![YogaRelation::Conjunction],
        ..listed(positions, vec![a, b])
    }]
}

/// Chandra-Mangala: Ay ile Mars birlikte.
pub(super) fn chandra_mangala(positions: &Positions) -> Vec<YogaMatch> {
    together(positions, Graha::Moon, Graha::Mars)
}

/// Budha-Aditya: Güneş ile Merkür birlikte.
pub(super) fn budha_aditya(positions: &Positions) -> Vec<YogaMatch> {
    together(positions, Graha::Sun, Graha::Mercury)
}

const NATURAL_BENEFIC_TRIO: [Graha; 3] = [Graha::Mercury, Graha::Jupiter, Graha::Venus];

/// Adhi: Merkür, Jüpiter ve Venüs'ün üçü de Ay'dan 6., 7. ya da 8. evde.
pub(super) fn adhi(positions: &Positions) -> Vec<YogaMatch> {
    let moon = positions.sign_of(Graha::Moon);
    if !NATURAL_BENEFIC_TRIO
        .into_iter()
        .all(|g| in_houses_from(positions, g, moon, &[6, 7, 8]))
    {
        return Vec::new();
    }
    let mut grahas = vec![Graha::Moon];
    grahas.extend(NATURAL_BENEFIC_TRIO);
    vec![listed(positions, grahas)]
}

/// Shakata: Ay Jüpiter'den 6., 8. ya da 12. evde. Ay lagnadan kendradaysa oluşmaz (iptal);
/// lagna ister.
pub(super) fn shakata(positions: &Positions) -> Vec<YogaMatch> {
    let Some(lagna) = positions.lagna_sign else {
        return Vec::new();
    };
    let jupiter = positions.sign_of(Graha::Jupiter);
    if !in_houses_from(positions, Graha::Moon, jupiter, &[6, 8, 12])
        || in_houses_from(positions, Graha::Moon, lagna, &KENDRA_HOUSES)
    {
        return Vec::new();
    }
    vec![listed(positions, vec![Graha::Moon, Graha::Jupiter])]
}

/// Lagna (varsa) ve Ay: iki referanslı kuralların referansları, koşul kimlikleriyle.
fn references(positions: &Positions) -> Vec<(&'static str, Sign)> {
    let mut out = Vec::new();
    if let Some(lagna) = positions.lagna_sign {
        out.push((COND_FROM_LAGNA, lagna));
    }
    out.push((COND_FROM_MOON, positions.sign_of(Graha::Moon)));
    out
}

/// Amala: lagnadan ya da Ay'dan 10. evde en az bir doğal iyicil (K6) var ve o evde kötücül
/// yok. Sağlayan referanslar koşul olarak, o evlerdeki grahalar eşleşmede listelenir.
pub(super) fn amala(positions: &Positions) -> Vec<YogaMatch> {
    let mut conditions = Vec::new();
    let mut grahas = Vec::new();
    for (condition, reference) in references(positions) {
        let occupants = positions.occupants(house_sign(reference, 10));
        let all_benefic = occupants
            .iter()
            .all(|&g| nature(g, positions) == Nature::Benefic);
        if !occupants.is_empty() && all_benefic {
            conditions.push(condition);
            grahas.extend(occupants);
        }
    }
    if conditions.is_empty() {
        return Vec::new();
    }
    vec![YogaMatch {
        conditions,
        ..listed(positions, grahas)
    }]
}

/// Vasumati: Merkür, Jüpiter ve Venüs'ün üçü de lagnadan upachaya evlerinde ya da üçü de
/// Ay'dan upachaya evlerinde.
pub(super) fn vasumati(positions: &Positions) -> Vec<YogaMatch> {
    let conditions: Vec<&'static str> = references(positions)
        .into_iter()
        .filter(|&(_, reference)| {
            NATURAL_BENEFIC_TRIO
                .into_iter()
                .all(|g| in_houses_from(positions, g, reference, &UPACHAYA_HOUSES))
        })
        .map(|(condition, _)| condition)
        .collect();
    if conditions.is_empty() {
        return Vec::new();
    }
    vec![YogaMatch {
        conditions,
        ..listed(positions, NATURAL_BENEFIC_TRIO.to_vec())
    }]
}

#[cfg(test)]
mod tests {
    use super::super::testkit::{chart, forms, matches};
    use super::*;
    use Graha::*;

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    /// Taban: Koç lagna; Ay Yengeç'te (4. ev), sayılan grahalar Ay'dan 3., 5., 6., 9. ve 11.
    /// evlerde, Güneş'ten 5., 7., 8., 11. ve 1. evlerde; hiçbiri lagnadan kendrada değil.
    /// Ay'dan: Mars Başak (3), Merkür Akrep (5), Jüpiter Yay (6), Venüs Balık (9), Satürn
    /// Boğa (11). Güneş Boğa'da.
    const BASE: [usize; 9] = [1, 3, 5, 7, 8, 11, 1, 2, 8];

    fn with(lagna: Option<usize>, changes: &[(Graha, usize)]) -> Positions {
        let mut signs = BASE;
        for &(g, s) in changes {
            signs[g as usize] = s;
        }
        chart(lagna, signs)
    }

    #[test]
    fn moon_flank_yogas_exclude_each_other() {
        let ids = ["lunar.sunapha", "lunar.anapha", "lunar.durudhara"];
        let formed =
            |p: &Positions| -> Vec<&str> { ids.into_iter().filter(|id| forms(id, p)).collect() };
        // Taban: Ay'dan 2. (Aslan) ve 12. (İkizler) boş.
        assert!(formed(&with(Some(0), &[])).is_empty());
        // Mars Aslan'da (Ay'dan 2.): Sunapha.
        let p = with(Some(0), &[(Mars, 4)]);
        assert_eq!(formed(&p), ["lunar.sunapha"]);
        let m = &matches("lunar.sunapha", &p)[0];
        assert_eq!(m.grahas, [Moon, Mars]);
        assert_eq!(m.houses, [4, 5]);
        assert!(m.relations.is_empty() && m.conditions.is_empty());
        // Venüs İkizler'de (Ay'dan 12.): Anapha.
        let p = with(Some(0), &[(Venus, 2)]);
        assert_eq!(formed(&p), ["lunar.anapha"]);
        assert_eq!(matches("lunar.anapha", &p)[0].grahas, [Moon, Venus]);
        // İkisi birden: yalnızca Durudhara.
        let p = with(Some(0), &[(Mars, 4), (Venus, 2), (Saturn, 4)]);
        assert_eq!(formed(&p), ["lunar.durudhara"]);
        let m = &matches("lunar.durudhara", &p)[0];
        assert_eq!(m.grahas, [Moon, Mars, Venus, Saturn]);
        assert_eq!(m.houses, [3, 4, 5]);
        // Güneş ve düğümler sayılmaz: Güneş Aslan'da, Rahu İkizler'de.
        assert!(formed(&with(Some(0), &[(Sun, 4), (Rahu, 2)])).is_empty());
        // Lagnasız konumda evler boştur; yoga yine oluşur.
        let p = with(None, &[(Mars, 4)]);
        assert_eq!(matches("lunar.sunapha", &p)[0].houses, [] as [u8; 0]);
    }

    #[test]
    fn sun_flank_yogas_exclude_each_other() {
        let ids = ["solar.vesi", "solar.vasi", "solar.ubhayachari"];
        let formed =
            |p: &Positions| -> Vec<&str> { ids.into_iter().filter(|id| forms(id, p)).collect() };
        // Taban: Satürn Güneş'le aynı burçta (Boğa); Güneş'ten 2. (İkizler) ve 12. (Koç) boş.
        assert!(formed(&with(Some(0), &[])).is_empty());
        let p = with(Some(0), &[(Jupiter, 2)]);
        assert_eq!(formed(&p), ["solar.vesi"]);
        assert_eq!(matches("solar.vesi", &p)[0].grahas, [Sun, Jupiter]);
        let p = with(Some(0), &[(Mercury, 0)]);
        assert_eq!(formed(&p), ["solar.vasi"]);
        let p = with(Some(0), &[(Jupiter, 2), (Mercury, 0)]);
        assert_eq!(formed(&p), ["solar.ubhayachari"]);
        assert_eq!(
            matches("solar.ubhayachari", &p)[0].grahas,
            [Sun, Mercury, Jupiter]
        );
        // Ay sayılmaz: Ay İkizler'de.
        assert!(formed(&with(Some(0), &[(Moon, 2)])).is_empty());
    }

    #[test]
    fn kemadruma_needs_an_isolated_moon_and_empty_kendras() {
        // Taban: Ay'dan 2. ve 12. boş, Ay yalnız; Ay'dan kendralar (Yengeç, Terazi, Oğlak,
        // Koç) ve lagnadan kendralar (Koç, Yengeç, Terazi, Oğlak) sayılan graha taşımıyor.
        let p = with(Some(0), &[]);
        let m = matches("lunar.kemadruma", &p);
        assert_eq!(m.len(), 1);
        assert_eq!(
            (m[0].grahas.as_slice(), m[0].houses.as_slice()),
            (&[Moon][..], &[4][..])
        );
        // Güneş ve düğümler yalnızlığı bozmaz.
        assert!(forms(
            "lunar.kemadruma",
            &with(Some(0), &[(Sun, 3), (Rahu, 4), (Ketu, 2)])
        ));
        // Oluşmaz: Ay'dan 2. evde graha (Sunapha oluşur).
        assert!(!forms("lunar.kemadruma", &with(Some(0), &[(Mars, 4)])));
        // Oluşmaz: Ay yalnız değil.
        assert!(!forms("lunar.kemadruma", &with(Some(0), &[(Venus, 3)])));
        // İptal: Ay'dan kendrada graha (Jüpiter Terazi'de, Ay'dan 4.).
        assert!(!forms("lunar.kemadruma", &with(Some(0), &[(Jupiter, 6)])));
        // İptal: lagnadan kendrada graha. İkizler lagna: Başak'taki Mars lagnadan 4. evdedir
        // ama Ay'dan 3. evdedir.
        assert!(!forms("lunar.kemadruma", &with(Some(2), &[])));
        // Lagna yoksa değerlendirilmez.
        assert!(!forms("lunar.kemadruma", &with(None, &[])));
    }

    #[test]
    fn gajakesari_counts_the_four_kendras_from_the_moon() {
        for (jupiter, want) in [
            (3, true),   // aynı burç
            (6, true),   // 4.
            (9, true),   // 7.
            (0, true),   // 10.
            (4, false),  // 2.
            (8, false),  // 6.
            (11, false), // 9.
        ] {
            let p = with(Some(0), &[(Jupiter, jupiter)]);
            assert_eq!(forms("lunar.gajakesari", &p), want, "Jupiter in {jupiter}");
        }
        let p = with(Some(0), &[(Jupiter, 9)]);
        let m = &matches("lunar.gajakesari", &p)[0];
        assert_eq!(
            (m.grahas.as_slice(), m.houses.as_slice()),
            (&[Moon, Jupiter][..], &[4, 10][..])
        );
    }

    #[test]
    fn conjunction_yogas_carry_the_relation() {
        assert!(!forms("lunar.chandra_mangala", &with(Some(0), &[])));
        let p = with(Some(0), &[(Mars, 3)]);
        let m = &matches("lunar.chandra_mangala", &p)[0];
        assert_eq!(m.grahas, [Moon, Mars]);
        assert_eq!(m.houses, [4]);
        assert_eq!(m.relations, [YogaRelation::Conjunction]);
        assert!(!forms("solar.budha_aditya", &with(Some(0), &[])));
        let p = with(Some(0), &[(Mercury, 1)]);
        let m = &matches("solar.budha_aditya", &p)[0];
        assert_eq!(m.grahas, [Sun, Mercury]);
        assert_eq!(m.relations, [YogaRelation::Conjunction]);
    }

    #[test]
    fn adhi_needs_all_three_in_the_sixth_seventh_or_eighth_from_the_moon() {
        // Ay Yengeç'te: 6. Yay, 7. Oğlak, 8. Kova.
        let p = with(Some(0), &[(Mercury, 8), (Jupiter, 9), (Venus, 10)]);
        let m = &matches("lunar.adhi", &p)[0];
        assert_eq!(m.grahas, [Moon, Mercury, Jupiter, Venus]);
        assert_eq!(m.houses, [4, 9, 10, 11]);
        // Üçü aynı evde de olabilir.
        assert!(forms(
            "lunar.adhi",
            &with(Some(0), &[(Mercury, 9), (Jupiter, 9), (Venus, 9)])
        ));
        // Biri dışarıda (Venüs Balık'ta, 9. ev): oluşmaz.
        assert!(!forms(
            "lunar.adhi",
            &with(Some(0), &[(Mercury, 8), (Jupiter, 9)])
        ));
    }

    #[test]
    fn shakata_is_cancelled_by_a_moon_in_a_kendra_from_the_lagna() {
        // Taban: Jüpiter Yay'da, Ay Yengeç'te: Ay Jüpiter'den 8. evde. Boğa lagna: Ay 3. evde.
        let p = with(Some(1), &[]);
        let m = &matches("lunar.shakata", &p)[0];
        assert_eq!(
            (m.grahas.as_slice(), m.houses.as_slice()),
            (&[Moon, Jupiter][..], &[3, 8][..])
        );
        // 6. ve 12. de sayılır: Jüpiter Kova'da (Ay ondan 6.), Jüpiter Aslan'da (Ay ondan 12.).
        assert!(forms("lunar.shakata", &with(Some(1), &[(Jupiter, 10)])));
        assert!(forms("lunar.shakata", &with(Some(1), &[(Jupiter, 4)])));
        // Oluşmaz: Ay Jüpiter'den 7. evde.
        assert!(!forms("lunar.shakata", &with(Some(1), &[(Jupiter, 9)])));
        // İptal: Koç lagna, Ay 4. evde (kendra).
        assert!(!forms("lunar.shakata", &with(Some(0), &[])));
        assert!(!forms("lunar.shakata", &with(None, &[])));
    }

    #[test]
    fn amala_needs_a_benefic_and_no_malefic_in_the_tenth() {
        // Taban, Koç lagna: lagnadan 10. Oğlak (boş); Ay'dan 10. Koç (boş).
        assert!(!forms("lunar.amala", &with(Some(0), &[])));
        // Jüpiter Oğlak'ta: lagnadan.
        let p = with(Some(0), &[(Jupiter, 9)]);
        let m = &matches("lunar.amala", &p)[0];
        assert_eq!(m.grahas, [Jupiter]);
        assert_eq!(m.houses, [10]);
        assert_eq!(m.conditions, [COND_FROM_LAGNA]);
        // Venüs Koç'ta: Ay'dan. Lagnasız konumda da bulunur.
        let p = with(None, &[(Venus, 0)]);
        let m = &matches("lunar.amala", &p)[0];
        assert_eq!(
            (m.grahas.as_slice(), m.conditions.as_slice()),
            (&[Venus][..], &[COND_FROM_MOON][..])
        );
        // İki referans birden; grahalar birleşir.
        let p = with(Some(0), &[(Jupiter, 9), (Venus, 0)]);
        let m = &matches("lunar.amala", &p)[0];
        assert_eq!(m.grahas, [Jupiter, Venus]);
        assert_eq!(m.conditions, [COND_FROM_LAGNA, COND_FROM_MOON]);
        // Aynı evde kötücül varsa oluşmaz: Jüpiter ve Ketu Oğlak'ta.
        assert!(!forms(
            "lunar.amala",
            &with(Some(0), &[(Jupiter, 9), (Ketu, 9)])
        ));
        // Kötücülle tek başına duran Merkür kötücüldür (K6): Merkür ve Güneş Oğlak'ta.
        assert!(!forms(
            "lunar.amala",
            &with(Some(0), &[(Mercury, 9), (Sun, 9)])
        ));
        // Tek başına Merkür iyicildir.
        assert!(forms("lunar.amala", &with(Some(0), &[(Mercury, 9)])));
        // Büyüyen Ay iyicildir: Terazi lagna, Ay Yengeç'te (lagnadan 10.), Güneş Boğa'da.
        let p = with(Some(6), &[]);
        let m = &matches("lunar.amala", &p)[0];
        assert_eq!(
            (m.grahas.as_slice(), m.conditions.as_slice()),
            (&[Moon][..], &[COND_FROM_LAGNA][..])
        );
        // Küçülen Ay kötücüldür: Güneş Akrep'te (uzanım 240°); Merkür de Akrep'tedir.
        assert!(!forms("lunar.amala", &with(Some(6), &[(Sun, 7)])));
    }

    #[test]
    fn vasumati_needs_all_three_in_upachayas_from_one_reference() {
        assert!(!forms("lunar.vasumati", &with(Some(0), &[])));
        // Koç lagna: 3. İkizler, 6. Başak, 10. Oğlak, 11. Kova.
        let p = with(Some(0), &[(Mercury, 2), (Jupiter, 5), (Venus, 10)]);
        let m = &matches("lunar.vasumati", &p)[0];
        assert_eq!(m.grahas, [Mercury, Jupiter, Venus]);
        assert_eq!(m.houses, [3, 6, 11]);
        assert_eq!(m.conditions, [COND_FROM_LAGNA]);
        // Ay Yengeç'te: 3. Başak, 6. Yay, 10. Koç, 11. Boğa.
        let p = with(Some(0), &[(Mercury, 5), (Jupiter, 8), (Venus, 1)]);
        assert_eq!(
            matches("lunar.vasumati", &p)[0].conditions,
            [COND_FROM_MOON]
        );
        // Karışık referans sayılmaz: Merkür ve Jüpiter lagnadan, Venüs yalnızca Ay'dan.
        assert!(!forms(
            "lunar.vasumati",
            &with(Some(0), &[(Mercury, 2), (Jupiter, 10), (Venus, 1)])
        ));
        // Lagnasız konumda yalnızca Ay referansı.
        let p = with(None, &[(Mercury, 5), (Jupiter, 8), (Venus, 1)]);
        assert_eq!(
            matches("lunar.vasumati", &p)[0].conditions,
            [COND_FROM_MOON]
        );
        assert!(!forms(
            "lunar.vasumati",
            &with(None, &[(Mercury, 2), (Jupiter, 5), (Venus, 10)])
        ));
    }
}
````

- [ ] **Step 6: Mahapurusha, neecha bhanga, nabhasa ve kuja dosha**

**Yeni dosya:** `crates/jyotish-core/src/yogas/placement.rs`

````rust
//! Yerleşime dayalı yogalar (spec §8): Pancha Mahapurusha, Neecha bhanga, nabhasa sayı ve
//! ashraya yogaları ve Kuja dosha.

use super::{
    in_houses_from, occupied_houses, YogaMatch, COND_DISPOSITOR_ASPECT, COND_EXALTATION,
    COND_EXALTED_GRAHA_IN_KENDRA, COND_FROM_LAGNA, COND_FROM_MOON, COND_FROM_VENUS, COND_OWN_SIGN,
    COND_SIGN_LORD_IN_KENDRA, KENDRA_HOUSES, YOGA_GRAHAS,
};
use crate::dignity::{debilitation_point, exaltation_point, own_signs, sign_lord};
use crate::drishti::graha_aspect_signs;
use crate::graha::Graha;
use crate::jaimini::{sign_kind, SignKind};
use crate::nature::kuja_dosha;
use crate::positions::Positions;
use crate::zodiac::Sign;

/// Pancha Mahapurusha: graha kendi ya da yücelme burcunda ve lagnadan kendrada. Yalnızca
/// lagnadan sayılır. Koşullar grahanın burcunun niteliğini yazar (Merkür Başak'ta ikisini de).
pub(super) fn mahapurusha(positions: &Positions, graha: Graha) -> Vec<YogaMatch> {
    let Some(house) = positions.house_of(graha) else {
        return Vec::new();
    };
    if !KENDRA_HOUSES.contains(&house) {
        return Vec::new();
    }
    let sign = positions.sign_of(graha);
    let mut conditions = Vec::new();
    if own_signs(graha).contains(&sign) {
        conditions.push(COND_OWN_SIGN);
    }
    if exaltation_point(graha).is_some_and(|(s, _)| s == sign) {
        conditions.push(COND_EXALTATION);
    }
    if conditions.is_empty() {
        return Vec::new();
    }
    vec![YogaMatch {
        grahas: vec![graha],
        houses: vec![house],
        conditions,
        ..YogaMatch::default()
    }]
}

/// `sign` burcunda yücelen graha (yedi graha içinde); Akrep'te yoktur.
pub(crate) fn exalted_in(sign: Sign) -> Option<Graha> {
    YOGA_GRAHAS
        .into_iter()
        .find(|&g| exaltation_point(g).is_some_and(|(s, _)| s == sign))
}

/// `graha` lagnadan ya da Ay'dan kendrada mı. Ay kendisinden sayılmaz: Ay için yalnızca lagna
/// referansı geçerlidir.
fn in_kendra_from_lagna_or_moon(positions: &Positions, graha: Graha) -> bool {
    let from_lagna = positions
        .house_of(graha)
        .is_some_and(|h| KENDRA_HOUSES.contains(&h));
    let from_moon = graha != Graha::Moon
        && in_houses_from(
            positions,
            graha,
            positions.sign_of(Graha::Moon),
            &KENDRA_HOUSES,
        );
    from_lagna || from_moon
}

/// Neecha bhanga: düşüş burcundaki her graha için sağlanan koşullar. Hiçbir koşul sağlanmıyorsa
/// eşleşme yoktur. Düşüş burcu derecesinden bağımsızdır (burcun tamamı).
pub(super) fn neecha_bhanga(positions: &Positions) -> Vec<YogaMatch> {
    let mut out = Vec::new();
    for graha in YOGA_GRAHAS {
        let Some((fall, _)) = debilitation_point(graha) else {
            continue;
        };
        if positions.sign_of(graha) != fall {
            continue;
        }
        // Grahanın dispozitörü, bulunduğu (düşüş) burcun lordudur.
        let lord = sign_lord(fall);
        let mut conditions = Vec::new();
        if in_kendra_from_lagna_or_moon(positions, lord) {
            conditions.push(COND_SIGN_LORD_IN_KENDRA);
        }
        if exalted_in(fall).is_some_and(|g| in_kendra_from_lagna_or_moon(positions, g)) {
            conditions.push(COND_EXALTED_GRAHA_IN_KENDRA);
        }
        if graha_aspect_signs(lord, positions.sign_of(lord)).contains(&fall) {
            conditions.push(COND_DISPOSITOR_ASPECT);
        }
        if !conditions.is_empty() {
            out.push(YogaMatch {
                grahas: vec![graha],
                houses: occupied_houses(positions, &[graha]),
                conditions,
                ..YogaMatch::default()
            });
        }
    }
    out
}

/// Yedi grahanın bulunduğu farklı burç sayısı, 1..=7.
pub fn occupied_sign_count(positions: &Positions) -> u8 {
    let mut signs: Vec<Sign> = YOGA_GRAHAS
        .into_iter()
        .map(|g| positions.sign_of(g))
        .collect();
    signs.sort_unstable();
    signs.dedup();
    signs.len() as u8
}

fn all_seven(positions: &Positions) -> Vec<YogaMatch> {
    vec![YogaMatch {
        grahas: YOGA_GRAHAS.to_vec(),
        houses: occupied_houses(positions, &YOGA_GRAHAS),
        ..YogaMatch::default()
    }]
}

/// Nabhasa sayı yogası: yedi graha tam `count` burca dağılmış.
pub(super) fn nabhasa_count(positions: &Positions, count: u8) -> Vec<YogaMatch> {
    if occupied_sign_count(positions) != count {
        return Vec::new();
    }
    all_seven(positions)
}

/// Nabhasa ashraya yogası: yedi grahanın hepsi `kind` türünden burçlarda.
pub(super) fn nabhasa_ashraya(positions: &Positions, kind: SignKind) -> Vec<YogaMatch> {
    if !YOGA_GRAHAS
        .into_iter()
        .all(|g| sign_kind(positions.sign_of(g)) == kind)
    {
        return Vec::new();
    }
    all_seven(positions)
}

/// Kuja dosha: 3.1'deki `kuja_dosha` olgusu; sağlayan referanslar koşul olarak listelenir.
/// İptal kuralı uygulanmaz.
pub(super) fn kuja(positions: &Positions) -> Vec<YogaMatch> {
    let dosha = kuja_dosha(positions);
    let mut conditions = Vec::new();
    if dosha.from_lagna == Some(true) {
        conditions.push(COND_FROM_LAGNA);
    }
    if dosha.from_moon {
        conditions.push(COND_FROM_MOON);
    }
    if dosha.from_venus {
        conditions.push(COND_FROM_VENUS);
    }
    if conditions.is_empty() {
        return Vec::new();
    }
    vec![YogaMatch {
        grahas: vec![Graha::Mars],
        houses: occupied_houses(positions, &[Graha::Mars]),
        conditions,
        ..YogaMatch::default()
    }]
}

#[cfg(test)]
mod tests {
    use super::super::testkit::{chart, forms, matches};
    use super::*;
    use Graha::*;
    use Sign::*;

    /// Elle yazılmış tablo (bağımsız ikinci yazım): mahapurusha yogası, grahası, kendi
    /// burçları ve yücelme burcu.
    const HAND_MAHAPURUSHA: [(&str, Graha, &[Sign], Sign); 5] = [
        ("mahapurusha.ruchaka", Mars, &[Aries, Scorpio], Capricorn),
        ("mahapurusha.bhadra", Mercury, &[Gemini, Virgo], Virgo),
        ("mahapurusha.hamsa", Jupiter, &[Sagittarius, Pisces], Cancer),
        ("mahapurusha.malavya", Venus, &[Taurus, Libra], Pisces),
        ("mahapurusha.shasha", Saturn, &[Capricorn, Aquarius], Libra),
    ];

    #[test]
    fn mahapurusha_matches_the_hand_table_for_every_lagna_and_sign() {
        for (id, graha, own, exalted) in HAND_MAHAPURUSHA {
            for lagna in 0..12usize {
                for sign in 0..12usize {
                    let mut signs = [0usize; 9];
                    signs[graha as usize] = sign;
                    let p = chart(Some(lagna), signs);
                    let house = (sign + 12 - lagna) % 12 + 1;
                    let s = Sign::from_index(sign);
                    let strong = own.contains(&s) || s == exalted;
                    let want = strong && [1, 4, 7, 10].contains(&house);
                    let got = matches(id, &p);
                    assert_eq!(!got.is_empty(), want, "{id}: lagna {lagna}, sign {sign}");
                    if let Some(m) = got.first() {
                        assert_eq!(m.grahas, [graha]);
                        assert_eq!(m.houses, [house as u8]);
                        let mut conditions = Vec::new();
                        if own.contains(&s) {
                            conditions.push(COND_OWN_SIGN);
                        }
                        if s == exalted {
                            conditions.push(COND_EXALTATION);
                        }
                        assert_eq!(m.conditions, conditions, "{id}");
                    }
                    // Lagna yoksa değerlendirilmez (yalnızca lagnadan sayılır).
                    assert!(!forms(id, &p.clone().without_lagna()));
                }
            }
        }
    }

    /// Elle yazılmış tablo: graha, düşüş burcu, o burcun lordu, o burçta yücelen graha.
    const HAND_FALL: [(Graha, Sign, Graha, Option<Graha>); 7] = [
        (Sun, Libra, Venus, Some(Saturn)),
        (Moon, Scorpio, Mars, None),
        (Mars, Cancer, Moon, Some(Jupiter)),
        (Mercury, Pisces, Jupiter, Some(Venus)),
        (Jupiter, Capricorn, Saturn, Some(Mars)),
        (Venus, Virgo, Mercury, Some(Mercury)),
        (Saturn, Aries, Mars, Some(Sun)),
    ];

    #[test]
    fn fall_signs_lords_and_exalted_grahas_match_the_hand_table() {
        for (graha, fall, lord, exalted) in HAND_FALL {
            assert_eq!(debilitation_point(graha).map(|(s, _)| s), Some(fall));
            assert_eq!(sign_lord(fall), lord, "{graha:?}");
            assert_eq!(exalted_in(fall), exalted, "{graha:?}");
        }
        // Yedi yücelme burcu dışındaki burçlarda yücelen graha yoktur.
        let exalting: Vec<Sign> = Sign::ALL
            .into_iter()
            .filter(|&s| exalted_in(s).is_some())
            .collect();
        assert_eq!(
            exalting,
            [Aries, Taurus, Cancer, Virgo, Libra, Capricorn, Pisces]
        );
    }

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    /// Taban: hiçbir graha düşüş burcunda değil. Güneş Aslan, Ay Boğa, Mars İkizler, Merkür
    /// Aslan, Jüpiter Kova, Venüs Yengeç, Satürn Yay.
    const BASE: [usize; 9] = [4, 1, 2, 4, 10, 3, 8, 5, 11];

    fn with(lagna: Option<usize>, changes: &[(Graha, usize)]) -> Positions {
        let mut signs = BASE;
        for &(g, s) in changes {
            signs[g as usize] = s;
        }
        chart(lagna, signs)
    }

    const NB: &str = "neecha_bhanga.neecha_bhanga";

    #[test]
    fn neecha_bhanga_lists_the_conditions_that_hold() {
        assert!(!forms(NB, &with(Some(0), &[])));
        // Güneş Terazi'de (düşüş). Lord Venüs Yengeç'te: Koç lagnadan 4. ev (kendra). Terazi'de
        // yücelen Satürn Yay'da: lagnadan 9., Ay'dan (Boğa) 8. Venüs Yengeç'ten Terazi'ye
        // bakmaz (4. burç).
        let p = with(Some(0), &[(Sun, 6)]);
        let m = matches(NB, &p);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].grahas, [Sun]);
        assert_eq!(m[0].houses, [7]);
        assert_eq!(m[0].conditions, [COND_SIGN_LORD_IN_KENDRA]);
        // Satürn Aslan'a alınır: Ay'dan (Boğa) 4. ev. Venüs Koç'a alınır: lagnadan 1. ev ve
        // Terazi'ye 7. bakış. Üç koşul birden.
        let p = with(Some(0), &[(Sun, 6), (Saturn, 4), (Venus, 0)]);
        assert_eq!(
            matches(NB, &p)[0].conditions,
            [
                COND_SIGN_LORD_IN_KENDRA,
                COND_EXALTED_GRAHA_IN_KENDRA,
                COND_DISPOSITOR_ASPECT
            ]
        );
        // Venüs Başak'a alınır (lagnadan 6., Ay'dan 5. ev). Güneş için hiçbir koşul yok: lord
        // Venüs kendrada değil ve Başak'tan Terazi'ye bakmıyor (2. burç); Satürn Yay'da kendrada
        // değil. Güneş listelenmez. Venüs ise Başak'ta düşüştedir ve kendi koşulları sağlanır.
        let p = with(Some(0), &[(Sun, 6), (Venus, 5)]);
        let m = matches(NB, &p);
        assert_eq!(m.len(), 1, "only Venus has a cancelling condition");
        assert_eq!(m[0].grahas, [Venus]);
        // Başak'ın lordu da Başak'ta yücelen graha da Merkür'dür: Aslan'da, Ay'dan 4. evde.
        // Merkür Aslan'dan Başak'a bakmaz (2. burç).
        assert_eq!(
            m[0].conditions,
            [COND_SIGN_LORD_IN_KENDRA, COND_EXALTED_GRAHA_IN_KENDRA]
        );
    }

    #[test]
    fn neecha_bhanga_special_cases() {
        // Ay Akrep'te: Akrep'te yücelen graha yok, ikinci koşul uygulanmaz. Lord Mars İkizler'de:
        // Koç lagnadan 3., Ay'dan 8.; Akrep'e bakmaz (6. burç). Koşul yok.
        assert!(!forms(NB, &with(Some(0), &[(Moon, 7)])));
        // Mars Aslan'a alınır: Akrep Aslan'dan 4. burçtur (Mars'ın özel bakışı); Aslan Ay'dan
        // 10. evdir.
        let p = with(Some(0), &[(Moon, 7), (Mars, 4)]);
        assert_eq!(
            matches(NB, &p)[0].conditions,
            [COND_SIGN_LORD_IN_KENDRA, COND_DISPOSITOR_ASPECT]
        );
        // Mars Yengeç'te: lord Ay'dır. Ay kendisinden sayılmaz; Ay Boğa'da, Koç lagnadan 2.
        // evde → ilk koşul yok. Yengeç'te yücelen Jüpiter Kova'da: lagnadan 11., Ay'dan 10. →
        // ikinci koşul. Ay Boğa'dan Yengeç'e bakmaz.
        let p = with(Some(0), &[(Mars, 3)]);
        assert_eq!(
            matches(NB, &p)[0].conditions,
            [COND_EXALTED_GRAHA_IN_KENDRA]
        );
        // Ay lagnadan kendradaysa ilk koşul sağlanır: Boğa lagna, Ay 1. evde.
        let p = with(Some(1), &[(Mars, 3)]);
        assert_eq!(
            matches(NB, &p)[0].conditions,
            [COND_SIGN_LORD_IN_KENDRA, COND_EXALTED_GRAHA_IN_KENDRA]
        );
        // Lagnasız konumda yalnızca Ay referansı ve bakış: aynı konumda ilk koşul düşer.
        let p = with(None, &[(Mars, 3)]);
        let m = &matches(NB, &p)[0];
        assert_eq!(m.conditions, [COND_EXALTED_GRAHA_IN_KENDRA]);
        assert!(m.houses.is_empty());
        // İki graha düşüşteyse iki ayrı eşleşme: Güneş Terazi'de (lord Venüs Koç'ta, bakıyor),
        // Satürn Koç'ta (lord Mars İkizler'de: lagnadan 3., Ay'dan 2.; Koç'ta yücelen Güneş
        // Terazi'de: lagnadan 7.).
        let p = with(Some(0), &[(Sun, 6), (Venus, 0), (Saturn, 0)]);
        let m = matches(NB, &p);
        assert_eq!(m.len(), 2);
        assert_eq!(
            (m[0].grahas.as_slice(), m[1].grahas.as_slice()),
            (&[Sun][..], &[Saturn][..])
        );
        assert_eq!(m[1].conditions, [COND_EXALTED_GRAHA_IN_KENDRA]);
    }

    /// Elle yazılmış tablo: dolu burç sayısı → sayı yogası.
    const HAND_COUNT: [(u8, &str); 7] = [
        (7, "nabhasa_count.vallaki"),
        (6, "nabhasa_count.dama"),
        (5, "nabhasa_count.pasha"),
        (4, "nabhasa_count.kedara"),
        (3, "nabhasa_count.shula"),
        (2, "nabhasa_count.yuga"),
        (1, "nabhasa_count.gola"),
    ];

    #[test]
    fn exactly_one_count_yoga_forms_and_nodes_do_not_count() {
        // Yedi graha ilk `n` burca dağıtılır; düğümler başka burçlardadır ve sayılmaz.
        for (n, id) in HAND_COUNT {
            let mut signs = [0usize; 9];
            for (i, s) in signs.iter_mut().take(7).enumerate() {
                *s = i.min(usize::from(n) - 1);
            }
            signs[7] = 10;
            signs[8] = 4;
            let p = chart(Some(2), signs);
            assert_eq!(occupied_sign_count(&p), n);
            let formed: Vec<&str> = HAND_COUNT
                .into_iter()
                .map(|(_, other)| other)
                .filter(|other| forms(other, &p))
                .collect();
            assert_eq!(formed, [id]);
            let m = &matches(id, &p)[0];
            assert_eq!(m.grahas, YOGA_GRAHAS);
            assert_eq!(m.houses.len(), usize::from(n));
            assert!(matches(id, &p.clone().without_lagna())[0].houses.is_empty());
        }
    }

    #[test]
    fn ashraya_yogas_need_all_seven_in_one_kind_of_sign() {
        let ids = [
            "nabhasa_ashraya.rajju",
            "nabhasa_ashraya.musala",
            "nabhasa_ashraya.nala",
        ];
        let formed =
            |p: &Positions| -> Vec<&str> { ids.into_iter().filter(|id| forms(id, p)).collect() };
        // Yürük burçlar: Koç, Yengeç, Terazi, Oğlak. Düğümler başka türden burçlarda.
        let p = chart(None, [0, 3, 6, 9, 0, 3, 6, 1, 7]);
        assert_eq!(formed(&p), ["nabhasa_ashraya.rajju"]);
        // Sabit burçlar: Boğa, Aslan, Akrep, Kova.
        let p = chart(None, [1, 4, 7, 10, 1, 4, 7, 0, 6]);
        assert_eq!(formed(&p), ["nabhasa_ashraya.musala"]);
        // Çift doğalı burçlar: İkizler, Başak, Yay, Balık.
        let p = chart(None, [2, 5, 8, 11, 2, 5, 8, 0, 6]);
        assert_eq!(formed(&p), ["nabhasa_ashraya.nala"]);
        // Tek graha başka türden burçtaysa hiçbiri oluşmaz.
        let p = chart(None, [2, 5, 8, 11, 2, 5, 7, 0, 6]);
        assert!(formed(&p).is_empty());
    }

    #[test]
    fn kuja_dosha_lists_the_references_that_hold() {
        // Mars İkizler'de. Koç lagnadan 3., Ay'dan (Boğa) 2., Venüs'ten (Yengeç) 12.
        let p = with(Some(0), &[]);
        let m = &matches("dosha.kuja", &p)[0];
        assert_eq!(m.grahas, [Mars]);
        assert_eq!(m.houses, [3]);
        assert_eq!(m.conditions, [COND_FROM_MOON, COND_FROM_VENUS]);
        // İkizler lagna: lagnadan 1. ev de.
        let p = with(Some(2), &[]);
        assert_eq!(
            matches("dosha.kuja", &p)[0].conditions,
            [COND_FROM_LAGNA, COND_FROM_MOON, COND_FROM_VENUS]
        );
        // Lagnasız konumda lagna referansı yoktur.
        let p = with(None, &[]);
        assert_eq!(
            matches("dosha.kuja", &p)[0].conditions,
            [COND_FROM_MOON, COND_FROM_VENUS]
        );
        // Oluşmaz: Mars Terazi'de, Venüs Aslan'da, Boğa lagna. Mars lagnadan 6., Ay'dan (Boğa)
        // 6., Venüs'ten 3. evdedir; hiçbiri 1, 2, 4, 7, 8, 12 değildir.
        let p = with(Some(1), &[(Mars, 6), (Venus, 4)]);
        assert!(!forms("dosha.kuja", &p));
    }
}
````

- [ ] **Step 7: Testleri koş**

```bash
cargo test -p jyotish-core --lib yogas 2>&1 | grep -E "^test result"
cargo test -p jyotish-core --lib 2>&1 | grep -E "^test result"
```
Expected: `20 passed` (239 süzülmüş); sonra `259 passed`.

- [ ] **Step 8: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
```
Expected: fmt ve clippy çıktısız; `jyotish-core` lib `259 passed`; diğer satırlar tabandaki gibi.

- [ ] **Step 9: Commit**

```bash
git add crates/jyotish-core/src/lib.rs crates/jyotish-core/src/yogas
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): yoga catalog types and relations; mahapurusha, Moon and Sun yogas, neecha bhanga, nabhasa and kuja dosha rules" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: iki `grep` çıktısız.

---

### Task 2: Çekirdek: lord yogaları (raja, dhana, değişim, viparita raja)

Spec §8 tablosunun Raja, Dhana, Değişim ve Viparita raja satırları; §13 katman 1 ve 3; D8–D12, D16. On kural eklenir; katalog 41 kurala tamamlanır. Hepsi lagna ister.

**Files:**
- Create: `crates/jyotish-core/src/yogas/lords.rs`
- Modify: `crates/jyotish-core/src/yogas/mod.rs`, `crates/jyotish-core/src/lib.rs`
- Test: `lords.rs` `mod tests` (8 test)

**Interfaces:**
- Consumes: Task 1'in tipleri ve yardımcıları (`YogaMatch`, `YogaRelation`, `yoga_relations`, `is_exchange`, `in_graha_order`, `KENDRA_HOUSES`, `TRIKONA_HOUSES`, `DUSTHANA_HOUSES`, `YOGA_GRAHAS`, `testkit`); `lordship::{house_lord, houses_owned}`.
- Produces (`jyotish_core`'dan dışa aktarılır): `ExchangeKind { Maha, ThirdLord, Dainya }`, `exchange_kind(a: u8, b: u8) -> ExchangeKind`, `DHANA_HOUSES: [u8; 5]`; katalogda on yeni kimlik: `raja.kendra_trikona`, `raja.dharma_karmadhipati`, `raja.yogakaraka`, `dhana.dhana`, `exchange.maha`, `exchange.third_lord`, `exchange.dainya`, `viparita.harsha`, `viparita.sarala`, `viparita.vimala`.

- [ ] **Step 1: Kuralları kataloğa kaydet (derleme başarısız olur)**

````diff
diff --git a/crates/jyotish-core/src/yogas/mod.rs b/crates/jyotish-core/src/yogas/mod.rs
index 8cbd972..c0e082a 100644
--- a/crates/jyotish-core/src/yogas/mod.rs
+++ b/crates/jyotish-core/src/yogas/mod.rs
@@ -7,9 +7,11 @@
 //! Evler rasi evidir (K4), burçlar rasi çerçevesindedir (K5), doğal sınıf `nature`'dır (K6).
 //! Lagnası olmayan konumlarda (doğum saati bilinmiyor) lagna isteyen kurallar değerlendirilmez.
 
+mod lords;
 mod luminaries;
 mod placement;
 
+pub use lords::{exchange_kind, ExchangeKind, DHANA_HOUSES};
 pub use placement::occupied_sign_count;
 
 use serde::{Deserialize, Serialize};
@@ -223,6 +225,38 @@ pub static YOGA_CATALOG: &[YogaRule] = &[
         false,
         luminaries::budha_aditya,
     ),
+    rule(
+        "raja.kendra_trikona",
+        YogaGroup::Raja,
+        true,
+        lords::kendra_trikona,
+    ),
+    rule(
+        "raja.dharma_karmadhipati",
+        YogaGroup::Raja,
+        true,
+        lords::dharma_karmadhipati,
+    ),
+    rule("raja.yogakaraka", YogaGroup::Raja, true, lords::yogakaraka),
+    rule("dhana.dhana", YogaGroup::Dhana, true, lords::dhana),
+    rule("exchange.maha", YogaGroup::Exchange, true, |p| {
+        lords::exchange(p, ExchangeKind::Maha)
+    }),
+    rule("exchange.third_lord", YogaGroup::Exchange, true, |p| {
+        lords::exchange(p, ExchangeKind::ThirdLord)
+    }),
+    rule("exchange.dainya", YogaGroup::Exchange, true, |p| {
+        lords::exchange(p, ExchangeKind::Dainya)
+    }),
+    rule("viparita.harsha", YogaGroup::ViparitaRaja, true, |p| {
+        lords::viparita(p, 6)
+    }),
+    rule("viparita.sarala", YogaGroup::ViparitaRaja, true, |p| {
+        lords::viparita(p, 8)
+    }),
+    rule("viparita.vimala", YogaGroup::ViparitaRaja, true, |p| {
+        lords::viparita(p, 12)
+    }),
     rule(
         "neecha_bhanga.neecha_bhanga",
         YogaGroup::NeechaBhanga,
````

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index cfb92c1..3718726 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -92,9 +92,9 @@ pub use varga::{
     all_vargas, is_vargottama, varga_chart, varga_position, Varga, VargaChart, VargaPlacement,
 };
 pub use yogas::{
-    is_exchange, is_mutual_aspect, occupied_sign_count, yoga_relations, yoga_rule, YogaGroup,
-    YogaMatch, YogaRelation, YogaRule, COUNTED_GRAHAS, DUSTHANA_HOUSES, KENDRA_HOUSES,
-    TRIKONA_HOUSES, UPACHAYA_HOUSES, YOGA_CATALOG, YOGA_CATALOG_VERSION, YOGA_CONDITIONS,
-    YOGA_GRAHAS,
+    exchange_kind, is_exchange, is_mutual_aspect, occupied_sign_count, yoga_relations, yoga_rule,
+    ExchangeKind, YogaGroup, YogaMatch, YogaRelation, YogaRule, COUNTED_GRAHAS, DHANA_HOUSES,
+    DUSTHANA_HOUSES, KENDRA_HOUSES, TRIKONA_HOUSES, UPACHAYA_HOUSES, YOGA_CATALOG,
+    YOGA_CATALOG_VERSION, YOGA_CONDITIONS, YOGA_GRAHAS,
 };
 pub use zodiac::{angular_distance, deg_in_sign, nakshatra_pada, normalize, Sign};
````

- [ ] **Step 2: Derlemenin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-core --lib yogas 2>&1 | grep -E "^error"
```
Expected: `error[E0583]: file not found for module `lords`` ve `could not compile`.

- [ ] **Step 3: Lord yogaları**

Testlerin taban haritası (Koç lagna) hiçbir lord çiftini ilişkilendirmeyecek biçimde kurulmuştur: yedi graha altı burca dağılır, hiçbir dolu burç diğerinin karşısında değildir, Satürn Mars'tan 4. burçta değildir, değişim yoktur; tek ilişkili çift Güneş–Merkür'dür ve Merkür (3. ve 6. lord) hiçbir kuralın ev kümesinde değildir. Her örnek tabandan bir ya da iki grahayı oynatır.

**Yeni dosya:** `crates/jyotish-core/src/yogas/lords.rs`

````rust
//! Ev lordlarına dayalı yogalar (spec §8): raja (kendra–trikona, dharma-karmadhipati,
//! yogakaraka), dhana, değişim sınıfları ve viparita raja. Hepsi lagna ister: lordlar lagna
//! burcundan sayılan rasi evlerinin lordlarıdır (K4; `lordship::house_lord`).

use super::{
    is_exchange, yoga_relations, YogaMatch, YogaRelation, DUSTHANA_HOUSES, KENDRA_HOUSES,
    TRIKONA_HOUSES, YOGA_GRAHAS,
};
use crate::graha::Graha;
use crate::lordship::{house_lord, houses_owned};
use crate::positions::Positions;
use crate::zodiac::Sign;

/// Dhana yogasında sayılan evler.
pub const DHANA_HOUSES: [u8; 5] = [1, 2, 5, 9, 11];

/// Değişim sınıfı; kural kimlikleri `exchange.maha`, `exchange.third_lord`, `exchange.dainya`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExchangeKind {
    Maha,
    ThirdLord,
    Dainya,
}

/// Değişen iki evin sınıfı: biri 6., 8. ya da 12. evse Dainya; değilse biri 3. evse "3. ev
/// lordu değişimi"; değilse Maha. `a` ve `b` farklı evlerdir.
pub fn exchange_kind(a: u8, b: u8) -> ExchangeKind {
    if DUSTHANA_HOUSES.contains(&a) || DUSTHANA_HOUSES.contains(&b) {
        ExchangeKind::Dainya
    } else if a == 3 || b == 3 {
        ExchangeKind::ThirdLord
    } else {
        ExchangeKind::Maha
    }
}

/// `set` içinden grahanın lordu olduğu evler, artan sırada.
fn owned_in(lagna: Sign, graha: Graha, set: &[u8]) -> Vec<u8> {
    houses_owned(lagna, graha)
        .into_iter()
        .filter(|h| set.contains(h))
        .collect()
}

fn merged(mut a: Vec<u8>, b: Vec<u8>) -> Vec<u8> {
    a.extend(b);
    a.sort_unstable();
    a.dedup();
    a
}

/// Yedi grahanın sırasız çiftleri, `Graha::ALL` sırasında.
fn pairs() -> impl Iterator<Item = (Graha, Graha)> {
    YOGA_GRAHAS
        .into_iter()
        .enumerate()
        .flat_map(|(i, a)| YOGA_GRAHAS[i + 1..].iter().map(move |&b| (a, b)))
}

/// Kendra–trikona raja yogası: bir kendra lordu ile bir trikona lordu farklı grahalardır ve
/// ilişkilidir. Çift başına tek eşleşme; evler iki grahanın lordu olduğu kendra ve trikonalardır.
pub(super) fn kendra_trikona(positions: &Positions) -> Vec<YogaMatch> {
    let Some(lagna) = positions.lagna_sign else {
        return Vec::new();
    };
    pairs()
        .filter_map(|(a, b)| {
            let (a_kendra, a_trikona) = (
                owned_in(lagna, a, &KENDRA_HOUSES),
                owned_in(lagna, a, &TRIKONA_HOUSES),
            );
            let (b_kendra, b_trikona) = (
                owned_in(lagna, b, &KENDRA_HOUSES),
                owned_in(lagna, b, &TRIKONA_HOUSES),
            );
            let pairs_up = (!a_kendra.is_empty() && !b_trikona.is_empty())
                || (!a_trikona.is_empty() && !b_kendra.is_empty());
            let relations = yoga_relations(a, b, positions);
            (pairs_up && !relations.is_empty()).then(|| YogaMatch {
                grahas: vec![a, b],
                houses: merged(merged(a_kendra, a_trikona), merged(b_kendra, b_trikona)),
                relations,
                conditions: Vec::new(),
            })
        })
        .collect()
}

/// Dharma-Karmadhipati: 9. ve 10. lord farklı grahalardır ve ilişkilidir.
pub(super) fn dharma_karmadhipati(positions: &Positions) -> Vec<YogaMatch> {
    let Some(lagna) = positions.lagna_sign else {
        return Vec::new();
    };
    let (ninth, tenth) = (house_lord(lagna, 9), house_lord(lagna, 10));
    let relations = yoga_relations(ninth, tenth, positions);
    if relations.is_empty() {
        return Vec::new();
    }
    vec![YogaMatch {
        grahas: super::in_graha_order(vec![ninth, tenth]),
        houses: vec![9, 10],
        relations,
        conditions: Vec::new(),
    }]
}

/// Yogakaraka: tek graha, lagna dışında hem bir kendranın (4, 7, 10) hem bir trikonanın (5, 9)
/// lordu. Yalnızca lagna burcuna bağlıdır.
pub(super) fn yogakaraka(positions: &Positions) -> Vec<YogaMatch> {
    let Some(lagna) = positions.lagna_sign else {
        return Vec::new();
    };
    YOGA_GRAHAS
        .into_iter()
        .filter_map(|g| {
            let kendra = owned_in(lagna, g, &[4, 7, 10]);
            let trikona = owned_in(lagna, g, &[5, 9]);
            (!kendra.is_empty() && !trikona.is_empty()).then(|| YogaMatch {
                grahas: vec![g],
                houses: merged(kendra, trikona),
                ..YogaMatch::default()
            })
        })
        .collect()
}

/// Dhana: {1, 2, 5, 9, 11} lordlarından, en az biri 2. ya da 11. lord olan iki farklı graha
/// ilişkili. Çift başına tek eşleşme; evler iki grahanın bu kümeden lordu olduğu evlerdir.
pub(super) fn dhana(positions: &Positions) -> Vec<YogaMatch> {
    let Some(lagna) = positions.lagna_sign else {
        return Vec::new();
    };
    pairs()
        .filter_map(|(a, b)| {
            let (wa, wb) = (
                owned_in(lagna, a, &DHANA_HOUSES),
                owned_in(lagna, b, &DHANA_HOUSES),
            );
            if wa.is_empty() || wb.is_empty() {
                return None;
            }
            let houses = merged(wa, wb);
            let relations = yoga_relations(a, b, positions);
            (houses.iter().any(|h| [2, 11].contains(h)) && !relations.is_empty()).then(|| {
                YogaMatch {
                    grahas: vec![a, b],
                    houses,
                    relations,
                    conditions: Vec::new(),
                }
            })
        })
        .collect()
}

/// Değişim yogaları: birbirinin burcundaki iki graha; evler değişen iki evdir (grahaların
/// bulunduğu evler). Eşleşme yalnızca `kind` sınıfındaki çiftleri verir.
pub(super) fn exchange(positions: &Positions, kind: ExchangeKind) -> Vec<YogaMatch> {
    if positions.lagna_sign.is_none() {
        return Vec::new();
    }
    pairs()
        .filter(|&(a, b)| is_exchange(a, b, positions))
        .filter_map(|(a, b)| {
            let (ha, hb) = (positions.house_of(a)?, positions.house_of(b)?);
            (exchange_kind(ha, hb) == kind).then(|| YogaMatch {
                grahas: vec![a, b],
                houses: merged(vec![ha], vec![hb]),
                relations: yoga_relations(a, b, positions)
                    .into_iter()
                    .filter(|r| *r == YogaRelation::Exchange)
                    .collect(),
                conditions: Vec::new(),
            })
        })
        .collect()
}

/// Viparita raja: `house` (6, 8 ya da 12) lordu 6., 8. ya da 12. evde. Evler lordu olduğu ev ve
/// bulunduğu evdir.
pub(super) fn viparita(positions: &Positions, house: u8) -> Vec<YogaMatch> {
    let Some(lagna) = positions.lagna_sign else {
        return Vec::new();
    };
    let lord = house_lord(lagna, house);
    match positions.house_of(lord) {
        Some(at) if DUSTHANA_HOUSES.contains(&at) => vec![YogaMatch {
            grahas: vec![lord],
            houses: merged(vec![house], vec![at]),
            ..YogaMatch::default()
        }],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::testkit::{chart, forms, matches};
    use super::super::YOGA_CATALOG;
    use super::*;
    use Graha::*;
    use YogaRelation::*;

    /// Elle yazılmış tablo (bağımsız ikinci yazım): lagna burcu (Koç 0 … Balık 11) →
    /// yogakaraka ve lordu olduğu kendra ile trikona. Altı lagnada yogakaraka yoktur.
    const HAND_YOGAKARAKA: [Option<(Graha, [u8; 2])>; 12] = [
        None,                    // Koç
        Some((Saturn, [9, 10])), // Boğa: Oğlak 9., Kova 10.
        None,                    // İkizler
        Some((Mars, [5, 10])),   // Yengeç: Akrep 5., Koç 10.
        Some((Mars, [4, 9])),    // Aslan: Akrep 4., Koç 9.
        None,                    // Başak
        Some((Saturn, [4, 5])),  // Terazi: Oğlak 4., Kova 5.
        None,                    // Akrep
        None,                    // Yay
        Some((Venus, [5, 10])),  // Oğlak: Boğa 5., Terazi 10.
        Some((Venus, [4, 9])),   // Kova: Boğa 4., Terazi 9.
        None,                    // Balık
    ];

    #[test]
    fn yogakaraka_matches_the_hand_table_whatever_the_placements() {
        for (lagna, want) in HAND_YOGAKARAKA.into_iter().enumerate() {
            for shift in [0usize, 5] {
                let p = chart(Some(lagna), [0, 1, 2, 3, 4, 5, 6, 7, 8].map(|s| s + shift));
                let got = matches("raja.yogakaraka", &p);
                match want {
                    Some((graha, houses)) => {
                        assert_eq!(got.len(), 1, "lagna {lagna}");
                        assert_eq!(got[0].grahas, [graha], "lagna {lagna}");
                        assert_eq!(got[0].houses, houses, "lagna {lagna}");
                        assert!(got[0].relations.is_empty() && got[0].conditions.is_empty());
                    }
                    None => assert!(got.is_empty(), "lagna {lagna}"),
                }
            }
        }
    }

    /// Elle yazılmış tablo: değişen iki evin sınıfı. Satır küçük ev, sütun büyük ev; `M` Maha,
    /// `3` 3. ev lordu değişimi, `D` Dainya. Köşegen ve altı kullanılmaz (`.`).
    const HAND_EXCHANGE: [&str; 12] = [
        ".M3MMDMDMMMD", // 1
        "..3MMDMDMMMD", // 2
        "...33D3D333D", // 3
        "....MDMDMMMD", // 4
        ".....DMDMMMD", // 5
        "......DDDDDD", // 6
        ".......DMMMD", // 7
        "........DDDD", // 8
        ".........MMD", // 9
        "..........MD", // 10
        "...........D", // 11
        "............", // 12
    ];

    #[test]
    fn exchange_kind_matches_the_hand_table_and_is_symmetric() {
        for a in 1..=12u8 {
            for b in (a + 1)..=12 {
                let cell = HAND_EXCHANGE[usize::from(a) - 1].as_bytes()[usize::from(b) - 1];
                let want = match cell {
                    b'M' => ExchangeKind::Maha,
                    b'3' => ExchangeKind::ThirdLord,
                    b'D' => ExchangeKind::Dainya,
                    other => panic!("bad cell {other} at {a},{b}"),
                };
                assert_eq!(exchange_kind(a, b), want, "{a} and {b}");
                assert_eq!(exchange_kind(b, a), want, "{b} and {a}");
            }
        }
    }

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    /// Taban, Koç lagna: Güneş ve Merkür Koç'ta (1. ev), Ay Boğa'da (2), Satürn Yengeç'te (4),
    /// Jüpiter Aslan'da (5), Mars Yay'da (9), Venüs Balık'ta (12). Altı dolu burcun hiçbiri
    /// diğerinin karşısında değildir; Satürn Mars'tan 4. burçta değildir; değişim yoktur. Tek
    /// ilişkili çift Güneş–Merkür'dür (birlikte) ve Merkür (3. ve 6. lord) kendra, trikona ya
    /// da dhana evlerinin lordu değildir.
    /// Koç lagnanın lordları: 1 Mars, 2 Venüs, 3 Merkür, 4 Ay, 5 Güneş, 6 Merkür, 7 Venüs,
    /// 8 Mars, 9 Jüpiter, 10 Satürn, 11 Satürn, 12 Jüpiter.
    const BASE: [usize; 9] = [0, 1, 8, 0, 4, 11, 3, 5, 11];

    fn with(lagna: usize, changes: &[(Graha, usize)]) -> Positions {
        let mut signs = BASE;
        for &(g, s) in changes {
            signs[g as usize] = s;
        }
        chart(Some(lagna), signs)
    }

    fn rows(id: &str, p: &Positions) -> Vec<(Vec<Graha>, Vec<u8>, Vec<YogaRelation>)> {
        matches(id, p)
            .into_iter()
            .map(|m| {
                assert!(m.conditions.is_empty(), "{id}");
                (m.grahas, m.houses, m.relations)
            })
            .collect()
    }

    #[test]
    fn kendra_trikona_pairs_a_kendra_lord_with_a_trikona_lord() {
        assert!(rows("raja.kendra_trikona", &with(0, &[])).is_empty());
        // Ay (4. lord) Aslan'a, Jüpiter'in (9. lord) yanına: birlikte.
        assert_eq!(
            rows("raja.kendra_trikona", &with(0, &[(Moon, 4)])),
            [(vec![Moon, Jupiter], vec![4, 9], vec![Conjunction])]
        );
        // Lagna lordu hem kendra hem trikona lordudur: Mars (1) Yay'da, Venüs (7) İkizler'e
        // alınır: karşılıklı 7. bakış. Venüs yalnızca kendra lordudur; Mars trikona lordu sayılır.
        assert_eq!(
            rows("raja.kendra_trikona", &with(0, &[(Venus, 2)])),
            [(vec![Mars, Venus], vec![1, 7], vec![MutualAspect])]
        );
        // İki kendra lordu (Ay 4, Venüs 7) birlikte olsa da yoga oluşmaz: Ay Balık'a alınır.
        assert!(rows("raja.kendra_trikona", &with(0, &[(Moon, 11)])).is_empty());
        // Değişim: Güneş (5) Yengeç'e, Ay (4) Aslan'a. Güneş orada Satürn'le (10), Ay Jüpiter'le
        // (9) birliktedir: üç çift.
        assert_eq!(
            rows("raja.kendra_trikona", &with(0, &[(Sun, 3), (Moon, 4)])),
            [
                (vec![Sun, Moon], vec![4, 5], vec![Exchange]),
                (vec![Sun, Saturn], vec![5, 10], vec![Conjunction]),
                (vec![Moon, Jupiter], vec![4, 9], vec![Conjunction]),
            ]
        );
        // Birden çok ilişki tek eşleşmede: Mars (1) Oğlak'a, Satürn (10) Koç'a: değişim ve
        // karşılıklı bakış (Koç Oğlak'tan 4., Oğlak Koç'tan 10. burç). Satürn Koç'ta Güneş'le
        // (5) birliktedir: o çift de oluşur.
        assert_eq!(
            rows("raja.kendra_trikona", &with(0, &[(Mars, 9), (Saturn, 0)])),
            [
                (vec![Sun, Saturn], vec![5, 10], vec![Conjunction]),
                (
                    vec![Mars, Saturn],
                    vec![1, 10],
                    vec![MutualAspect, Exchange]
                ),
            ]
        );
        assert!(!forms(
            "raja.kendra_trikona",
            &with(0, &[(Moon, 4)]).without_lagna()
        ));
    }

    #[test]
    fn dharma_karmadhipati_is_the_ninth_and_tenth_lord_case() {
        // Koç lagna: 9. lord Jüpiter (Aslan), 10. lord Satürn (Yengeç): ilişkili değiller.
        assert!(!forms("raja.dharma_karmadhipati", &with(0, &[])));
        // Satürn Kova'ya: Jüpiter'le karşılıklı 7. bakış.
        let p = with(0, &[(Saturn, 10)]);
        assert_eq!(
            rows("raja.dharma_karmadhipati", &p),
            [(vec![Jupiter, Saturn], vec![9, 10], vec![MutualAspect])]
        );
        // Aynı çift kendra–trikona kaydı olarak da döner.
        assert_eq!(
            rows("raja.kendra_trikona", &p),
            [(vec![Jupiter, Saturn], vec![9, 10], vec![MutualAspect])]
        );
        // Boğa lagnada 9. ve 10. lord aynı grahadır (Satürn): yoga oluşmaz, yogakaraka oluşur.
        for shift in 0..12usize {
            let p = chart(Some(1), BASE.map(|s| s + shift));
            assert!(!forms("raja.dharma_karmadhipati", &p), "shift {shift}");
            assert!(forms("raja.yogakaraka", &p));
        }
        // Grahalar `Graha::ALL` sırasındadır: Kova lagnada 9. lord Venüs, 10. lord Mars; ikisi
        // İkizler'de.
        let p = with(10, &[(Mars, 2), (Venus, 2)]);
        assert_eq!(
            rows("raja.dharma_karmadhipati", &p),
            [(vec![Mars, Venus], vec![9, 10], vec![Conjunction])]
        );
    }

    #[test]
    fn dhana_needs_the_second_or_eleventh_lord_in_the_pair() {
        // Koç lagna: 1 Mars, 2 Venüs, 5 Güneş, 9 Jüpiter, 11 Satürn.
        assert!(rows("dhana.dhana", &with(0, &[])).is_empty());
        // Güneş (5) Aslan'a, Jüpiter'in (9) yanına: ikisi de kümede ama 2. ya da 11. lord yok.
        assert!(rows("dhana.dhana", &with(0, &[(Sun, 4)])).is_empty());
        // Venüs (2) Aslan'a, Jüpiter'in (9) yanına.
        assert_eq!(
            rows("dhana.dhana", &with(0, &[(Venus, 4)])),
            [(vec![Jupiter, Venus], vec![2, 9], vec![Conjunction])]
        );
        // Satürn (11) Koç'a, Güneş'in (5) yanına; Merkür de oradadır ama kümede değildir.
        assert_eq!(
            rows("dhana.dhana", &with(0, &[(Saturn, 0)])),
            [(vec![Sun, Saturn], vec![5, 11], vec![Conjunction])]
        );
        // 2. ve 11. lord birbiriyle: Venüs (2) Yengeç'e, Satürn'ün (11) yanına.
        assert_eq!(
            rows("dhana.dhana", &with(0, &[(Venus, 3)])),
            [(vec![Venus, Saturn], vec![2, 11], vec![Conjunction])]
        );
        // Bir graha kümeden iki evin lorduysa ikisi de yazılır: Boğa lagnada Merkür 2 ve 5'in,
        // Jüpiter 11'in lordudur. Merkür Kova'ya alınır: Aslan'daki Jüpiter'le karşılıklı bakış.
        assert_eq!(
            rows("dhana.dhana", &with(1, &[(Mercury, 10)])),
            [(vec![Mercury, Jupiter], vec![2, 5, 11], vec![MutualAspect])]
        );
    }

    #[test]
    fn exchanges_are_classified_by_the_two_houses() {
        let ids = ["exchange.maha", "exchange.third_lord", "exchange.dainya"];
        let all = |p: &Positions| -> Vec<(&str, Vec<Graha>, Vec<u8>)> {
            ids.into_iter()
                .flat_map(|id| {
                    rows(id, p).into_iter().map(move |(g, h, r)| {
                        assert_eq!(r, [Exchange], "{id}");
                        (id, g, h)
                    })
                })
                .collect()
        };
        assert!(all(&with(0, &[])).is_empty());
        // Koç lagna. Maha: Güneş Yengeç'te (4. ev), Ay Aslan'da (5. ev).
        assert_eq!(
            all(&with(0, &[(Sun, 3), (Moon, 4)])),
            [("exchange.maha", vec![Sun, Moon], vec![4, 5])]
        );
        // 3. ev lordu değişimi: Merkür Boğa'da (2. ev), Venüs İkizler'de (3. ev).
        assert_eq!(
            all(&with(0, &[(Mercury, 1), (Venus, 2)])),
            [("exchange.third_lord", vec![Mercury, Venus], vec![2, 3])]
        );
        // Dainya: Mars Başak'ta (6. ev), Merkür Koç'ta (1. ev). Mars 1. evin de lordudur; sınıfı
        // değişen evler (1 ve 6) belirler.
        assert_eq!(
            all(&with(0, &[(Mars, 5)])),
            [("exchange.dainya", vec![Mars, Mercury], vec![1, 6])]
        );
        // Aynı iki graha, başka evler: Mars İkizler'de (3), Merkür Akrep'te (8). 6, 8 ve 12
        // önce gelir: Dainya.
        assert_eq!(
            all(&with(0, &[(Mars, 2), (Mercury, 7)])),
            [("exchange.dainya", vec![Mars, Mercury], vec![3, 8])]
        );
        // İki değişim birden: Güneş–Ay (4 ve 5) ve Mars Terazi'de, Venüs Koç'ta (7 ve 1).
        // İkinci çift karşılıklı bakışta da olsa değişim kaydı yalnızca değişimi taşır.
        assert_eq!(
            all(&with(0, &[(Sun, 3), (Moon, 4), (Mars, 6), (Venus, 0)])),
            [
                ("exchange.maha", vec![Sun, Moon], vec![4, 5]),
                ("exchange.maha", vec![Mars, Venus], vec![1, 7]),
            ]
        );
        assert!(!forms(
            "exchange.maha",
            &with(0, &[(Sun, 3), (Moon, 4)]).without_lagna()
        ));
    }

    #[test]
    fn viparita_names_follow_the_lorded_house() {
        // Koç lagna: 6 Merkür, 8 Mars, 12 Jüpiter. Taban: Merkür 1., Mars 9., Jüpiter 5. evde.
        for id in ["viparita.harsha", "viparita.sarala", "viparita.vimala"] {
            assert!(!forms(id, &with(0, &[])), "{id}");
        }
        // Harsha: 6. lord Merkür 8. evde (Akrep).
        let p = with(0, &[(Mercury, 7)]);
        assert_eq!(
            rows("viparita.harsha", &p),
            [(vec![Mercury], vec![6, 8], vec![])]
        );
        // Kendi evinde: 6. lord 6. evde → tek ev.
        let p = with(0, &[(Mercury, 5)]);
        assert_eq!(
            rows("viparita.harsha", &p),
            [(vec![Mercury], vec![6], vec![])]
        );
        // Sarala: 8. lord Mars 12. evde (Balık).
        let p = with(0, &[(Mars, 11)]);
        assert_eq!(
            rows("viparita.sarala", &p),
            [(vec![Mars], vec![8, 12], vec![])]
        );
        assert!(!forms("viparita.harsha", &p) && !forms("viparita.vimala", &p));
        // Vimala: 12. lord Jüpiter 6. evde (Başak).
        let p = with(0, &[(Jupiter, 5)]);
        assert_eq!(
            rows("viparita.vimala", &p),
            [(vec![Jupiter], vec![6, 12], vec![])]
        );
        assert!(!forms("viparita.vimala", &p.without_lagna()));
    }

    /// Elle yazılmış liste: lagna isteyen kurallar (doğum saati bilinmiyorsa atlananlar).
    const HAND_NEEDS_LAGNA: [&str; 17] = [
        "mahapurusha.ruchaka",
        "mahapurusha.bhadra",
        "mahapurusha.hamsa",
        "mahapurusha.malavya",
        "mahapurusha.shasha",
        "lunar.kemadruma",
        "lunar.shakata",
        "raja.kendra_trikona",
        "raja.dharma_karmadhipati",
        "raja.yogakaraka",
        "dhana.dhana",
        "exchange.maha",
        "exchange.third_lord",
        "exchange.dainya",
        "viparita.harsha",
        "viparita.sarala",
        "viparita.vimala",
    ];

    #[test]
    fn the_catalog_has_forty_one_rules_and_seventeen_need_the_lagna() {
        assert_eq!(YOGA_CATALOG.len(), 41);
        let needing: Vec<&str> = YOGA_CATALOG
            .iter()
            .filter(|r| r.needs_lagna)
            .map(|r| r.id)
            .collect();
        assert_eq!(needing, HAND_NEEDS_LAGNA);
    }
}
````

- [ ] **Step 4: Testleri koş**

```bash
cargo test -p jyotish-core --lib yogas::lords 2>&1 | grep -E "^test result"
cargo test -p jyotish-core --lib 2>&1 | grep -E "^test result"
```
Expected: `8 passed`; sonra `267 passed`.

- [ ] **Step 5: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
```
Expected: fmt ve clippy çıktısız; `jyotish-core` lib `267 passed`; diğer satırlar tabandaki gibi.

- [ ] **Step 6: Commit**

```bash
git add crates/jyotish-core/src/lib.rs crates/jyotish-core/src/yogas
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): lordship yogas: kendra-trikona, dharma-karmadhipati, yogakaraka, dhana, the three exchange classes and viparita raja" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: `grep` çıktısız.

---


### Task 3: Çekirdek: rapor, toplayıcı, el fikstürleri ve değişmezler

Spec §8 "Veri modeli ve IPC" (`YogaHit`, `YogaReport`), "Kararlar" (doğum saati bilinmiyorsa), "Test" (kural başına el örneği; property'ler); §13 katman 1 ve 2; K2, K3; D3, D4, D15.

**Files:**
- Create: `tests/golden/modules/yogas/REVIEW.md` ve 15 JSON
- Modify: `crates/jyotish-core/src/yogas/mod.rs`, `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/proptests.rs`, `crates/jyotish-core/tests/modules_golden.rs`, `tests/golden/modules/README.md`
- Test: `mod.rs` (3 test), `proptests.rs` `yoga_properties` (6 test), `modules_golden.rs` (2 test)

**Interfaces:**
- Consumes: Task 1 ve 2'nin kataloğu; `positions::{compute_positions, ChartInput}`, `CoreError`, `ephemeris::Ephemeris`; testlerde `fake::FakeEph`, `chart::compute_chart`, `proptests::foundation_properties::positions`.
- Produces (`jyotish_core`'dan dışa aktarılır):
  - `pub struct YogaHit { pub id: String, pub group: YogaGroup, pub grahas: Vec<Graha>, pub houses: Vec<u8>, pub relations: Vec<YogaRelation>, pub conditions: Vec<String>, pub approximate: bool }`
  - `pub struct YogaReport { pub catalog_version: u32, pub hits: Vec<YogaHit>, pub evaluated: u32, pub skipped_time_unknown: u32 }`
  - `pub fn yogas(positions: &Positions) -> YogaReport`
  - `pub fn compute_yogas<E: Ephemeris + ?Sized>(eph: &mut E, input: &ChartInput) -> Result<YogaReport, CoreError>`
  - El fikstürü şeması: `expected { hits: [{ id, grahas, houses, relations, conditions }], absent: [id], cancelled: [id], evaluated, skipped_time_unknown }`.

- [ ] **Step 1: El örneklerini yaz (kaynak: `hand`)**

On beş sentetik harita. Her dosyanın `derivation`'ı katalogdaki 41 kuralı sırayla değerlendirir (burçlar ve evler, lordlar, K6 sınıfı, sonra grup grup); `expected.hits` raporun tamamıdır. Beklenen değerler kuraldan (spec §8 ve `REVIEW.md`'deki okuma kuralları) türetilmiştir; **motor çıktısına bakarak yazılmaz ve düzeltilmez.** Örneklerin birlikte kapsadıkları: 41 kuralın her biri en az bir örnekte oluşur ve en az bir örnekte `absent` sayılır; üç iptal durumu; sekiz koşul kimliği; üç ilişki türü; bir kayıtta iki ilişki; iki koşullu Bhadra; yedi sayı yogasının yedisi; iki doğum saati bilinmeyen harita.

| Dosya | Lagna | Öne çıkan |
|---|---|---|
| `three-exalted-mahapurushas-seven-signs` | Koç | Ruchaka, Hamsa, Shasha (yücelme); Bhadra yok (6. ev); Vallaki; Anapha, Shakata |
| `bhadra-with-both-conditions-malavya-and-hamsa` | İkizler | Bhadra (iki koşul), Malavya, Hamsa (kendi burcu); Amala lagnadan; Neecha bhanga iki koşulla |
| `sunapha-vesi-and-shakata-cancelled-by-kendra-moon` | Yengeç | Sunapha, Vesi; Shakata iptal (Ay 4. evde); yogakaraka Mars |
| `anapha-vasi-and-dharma-karmadhipati` | Başak | Anapha, Vasi; Dharma-Karmadhipati birlikte; Harsha, Sarala; düşüşteki Satürn için koşul yok |
| `durudhara-ubhayachari-and-maha-exchange-with-mutual-aspect` | Terazi | Durudhara, Ubhayachari; Maha değişimi ve karşılıklı bakış aynı çiftte; Neecha bhanga üç koşulla |
| `kemadruma-kedara-and-debilitated-venus-without-bhanga` | Boğa | Kemadruma; Kedara; düşüşteki Venüs için koşul yok; Vimala |
| `kemadruma-cancelled-by-a-graha-in-a-lagna-kendra` | Boğa | Kemadruma iptal (Jüpiter 10. evde); Dainya değişimi |
| `yuga-and-musala-two-fixed-signs` | Balık | Yuga, Musala |
| `adhi-and-chandra-mangala-ruchaka-in-own-sign` | Oğlak | Adhi, Chandra-Mangala; Ruchaka ve Malavya (kendi burcu); Kuja dosha üç referanstan |
| `shula-and-nala-three-dual-signs` | Balık | Shula, Nala |
| `three-exchanges-one-of-each-class` | Aslan | Maha, 3. ev lordu ve Dainya değişimi; üç viparita |
| `vasumati-and-amala-from-both-references-no-kuja-dosha` | Koç | Vasumati ve Amala iki referanstan; Kuja dosha yok; raja ve dhana yok |
| `neecha-bhanga-by-dispositor-aspect-only` | Boğa | Neecha bhanga yalnızca dispozitör bakışıyla |
| `time-unknown-gola-and-rajju-in-capricorn` | yok | Gola, Rajju; 17 kural atlanır |
| `time-unknown-moon-references-only` | yok | Amala ve Vasumati yalnızca Ay'dan |

**Yeni dosya:** `tests/golden/modules/yogas/REVIEW.md`

````markdown
# Yoga el örnekleri: okuma kuralları ve gözden geçirme kaydı

Kaynak: ileri modüller tasarım belgesi §8 (kural tablosu ve kararlar), §2 (K4 rasi evleri, K5 rasi
çerçevesi, K6 doğal sınıf) ve §4 (burç temelli tam bakış). Bu dizindeki her JSON dosyası bir
sentetik haritadır; `derivation` katalogdaki 41 kuralı sırayla değerlendirir ve `expected.hits`
raporun bütün kayıtlarını yazar. Değerler kuraldan elle türetilmiştir; motor çıktısı buraya
kopyalanmaz (K9). Bir kayıt motorla uyuşmazsa önce kural yeniden okunur.

## Okuma kuralları

Tasarım belgesinin açık bıraktığı yerlerde örnekler şu okumayı kullanır:

1. **Evler ve ilişkiler.** Ev, ilgili burçtan (lagna, Ay, Güneş, Jüpiter ya da Venüs) sayılan
   bütün burç evidir; aynı burç 1. evdir. "Birlikte" aynı burçtur. "Karşılıklı bakış" iki yönlü
   burç temelli tam bakıştır: 7. burç her çiftte karşılıklıdır; özel bakışlardan yalnızca Mars'ın
   4. ve Satürn'ün 10. bakışı birbirini karşılar. "Değişim" iki grahanın birbirinin burcunda
   olmasıdır. Düğümler bakış atmaz, burç lordu değildir ve hiçbir kayıtta yer almaz.
2. **Sayılan grahalar** Mars, Merkür, Jüpiter, Venüs ve Satürn'dür. Kemadruma'da "boş" ve "yalnız"
   bu beş grahaya göre okunur (Güneş ve düğümler Ay'ın yanında durabilir); Ay'dan kendralar 1.,
   4., 7. ve 10. evlerdir.
3. **Mahapurusha** burç düzeyindedir: kendi burcu `own_sign`, yücelme burcu `exaltation` koşulunu
   verir (Merkür Başak'ta ikisini de). Yalnızca lagnadan sayılır.
4. **Lordlar** lagna burcundan sayılan evlerin burç lordlarıdır (Akrep: Mars, Kova: Satürn). Lagna
   lordu hem kendra hem trikona lordudur. Kendra–trikona ve dhana kayıtları çift başınadır; 9. ve
   10. lord çifti iki kayıt verir (kendra–trikona ve Dharma-Karmadhipati). Yogakaraka yalnızca
   lagna burcuna bağlıdır.
5. **Değişim sınıfını** değişen iki ev (grahaların bulunduğu evler) belirler: biri 6, 8 ya da 12
   ise Dainya; değilse biri 3 ise 3. ev lordu değişimi; değilse Maha.
6. **Neecha bhanga** düşüş burcundaki her graha için ayrı kayıttır; düşüş burcun tamamıdır. Koşul
   kimlikleri: `sign_lord_in_kendra`, `exalted_graha_in_kendra`, `dispositor_aspect` (dispozitörün
   burç temelli tam bakışı). Ay kendisinden sayılmaz: lord Ay ise yalnızca lagna referansı
   geçerlidir. Hiçbir koşul sağlanmıyorsa kayıt yoktur.
7. **İki referanslı kurallar** (Amala, Vasumati, Kuja dosha) tek kayıt verir; sağlayan referanslar
   koşuldur (`from_lagna`, `from_moon`, `from_venus`). Amala'da 10. evdeki her graha K6'ya göre
   iyicil olmalıdır.
8. **`grahas` ve `houses`.** `grahas` yogayı oluşturan grahalardır (`Graha::ALL` sırasında); Ay ve
   Güneş yogalarında ışık da listelenir, Neecha bhanga'da yalnızca düşüşteki graha, nabhasa
   yogalarında yedi graha. `houses` lagnadan sayılır: raja ve dhana kayıtlarında çiftin lordu
   olduğu ilgili evler; değişimde değişen iki ev; viparita'da lordu olunan ve bulunulan ev;
   diğerlerinde listelenen grahaların bulunduğu evler.
9. **Doğum saati bilinmiyorsa** (`lagna_lon: null`) lagna isteyen 17 kural değerlendirilmez (beş
   mahapurusha, Kemadruma, Shakata, üç raja, dhana, üç değişim, üç viparita); diğer 24 kural lagna
   seçenekleri olmadan değerlendirilir, `houses` boştur.

`absent` o örneğin "oluşmayan durum" olarak gösterdiği kurallardır. `cancelled` iptal örnekleridir:
temel koşulu sağlandığı hâlde kendradaki grahayla oluşmayan Kemadruma, Ay'ı lagnadan kendrada olan
Shakata ve düşüşte olup hiçbir koşulu sağlanmayan graha (Neecha bhanga kaydı yok).

## Gözden geçirme kaydı

Gözden geçiren (motoru yazan dışında biri) her örnekte yalnızca `input`'tan başlar, motoru
çalıştırmadan ve Rust kaynağını okumadan 41 kuralı tasarım belgesinden ve yukarıdaki okuma
kurallarından yeniden değerlendirir; sonucunu `expected` ile karşılaştırır ve aşağıya adını,
tarihini ve bulduğu farkları yazar.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
| | | |
````

**Yeni dosya:** `tests/golden/modules/yogas/three-exalted-mahapurushas-seven-signs.json`

````json
{
  "id": "three-exalted-mahapurushas-seven-signs",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Lagna 10° Koç. Burçlar ve lagna burcundan sayılan evler: Güneş 125° Aslan (5. ev); Ay 320° Kova (11. ev); Mars 298° Oğlak (10. ev); Merkür 165° Başak (6. ev); Jüpiter 95° Yengeç (4. ev); Venüs 72° İkizler (3. ev); Satürn 200° Terazi (7. ev); Rahu 33° Boğa (2. ev); Ketu 213° Akrep (8. ev).",
    "Ev lordları (Koç lagna): 1 Mars, 2 Venüs, 3 Merkür, 4 Ay, 5 Güneş, 6 Merkür, 7 Venüs, 8 Mars, 9 Jüpiter, 10 Satürn, 11 Satürn, 12 Jüpiter.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 195°: küçülüyor, kötücül. Merkür'ün burcundaki diğer grahalar: boş; Merkür iyicil.",
    "Mahapurusha (lagnadan kendra 1, 4, 7, 10). Ruchaka: Mars Oğlak (yücelme burcu), 10. ev (kendra) → oluşur. Bhadra: Merkür Başak (kendi burcu ve yücelme burcu), 6. ev (kendra değil) → yok. Hamsa: Jüpiter Yengeç (yücelme burcu), 4. ev (kendra) → oluşur. Malavya: Venüs İkizler (kendi ya da yücelme burcu değil), 3. ev (kendra değil) → yok. Shasha: Satürn Terazi (yücelme burcu), 7. ev (kendra) → oluşur.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Balık: boş; 12. burç Oğlak: Mars → Anapha oluşur; diğer ikisi yok.",
    "Kemadruma: Ay'ın burcunda sayılan graha: boş; Ay'dan 4., 7. ve 10. evlerde: boş; lagnadan kendralarda: Mars, Jüpiter, Satürn → Kemadruma yok.",
    "Gajakesari: Jüpiter Ay'dan 6. evde (kendra değil) → yok. Chandra-Mangala: Ay Kova, Mars Oğlak → yok.",
    "Adhi: Ay'dan evler: Merkür 8, Jüpiter 6, Venüs 5 → üçü birden 6, 7 ya da 8 değil: yok.",
    "Shakata: Ay Jüpiter'den 8. evde (6, 8 ya da 12); Ay lagnadan 11. evde (kendra değil) → oluşur.",
    "Amala: lagnadan 10. burç Oğlak: Mars kötücül (kötücül var, sağlamaz); Ay'dan 10. burç Akrep: Ketu kötücül (kötücül var, sağlamaz) → yok.",
    "Vasumati (upachaya 3, 6, 10, 11): lagnadan Merkür 6, Jüpiter 4, Venüs 3 (üçü birden upachaya değil); Ay'dan Merkür 8, Jüpiter 6, Venüs 5 (üçü birden upachaya değil) → yok.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Başak: Merkür; 12. burç Yengeç: Jüpiter → Ubhayachari oluşur; diğer ikisi yok. Budha-Aditya: Güneş Aslan, Merkür Başak → yok.",
    "İlişkili çiftler (yedi graha; birlikte = aynı burç, karşılıklı bakış = iki yönlü tam bakış, değişim = birbirinin burcunda): Güneş–Ay (karşılıklı bakış); Mars–Jüpiter (karşılıklı bakış).",
    "Kendra–trikona: kendra lordları 1 Mars, 4 Ay, 7 Venüs, 10 Satürn; trikona lordları 1 Mars, 5 Güneş, 9 Jüpiter. İlişkili çiftlerden bir kendra lordu ile bir trikona lordu olanlar: Güneş–Ay (evler 4, 5); Mars–Jüpiter (evler 1, 9).",
    "Dharma-Karmadhipati: 9. lord Jüpiter, 10. lord Satürn: ilişkili değil → yok. Yogakaraka (lagna dışında bir kendranın ve bir trikonanın lordu): Koç lagnada yok.",
    "Dhana: 1 Mars, 2 Venüs, 5 Güneş, 9 Jüpiter, 11 Satürn. İlişkili çiftlerden iki lordu da bu kümeden olan ve 2. ya da 11. lordu içerenler: yok.",
    "Değişim (sınıfı değişen iki ev belirler: 6, 8 ya da 12 varsa Dainya; yoksa 3 varsa 3. ev lordu değişimi; yoksa Maha): değişen çift yok.",
    "Viparita raja (lord 6., 8. ya da 12. evde): 6. lord Merkür 6. evde → Harsha oluşur; 8. lord Mars 10. evde → Sarala yok; 12. lord Jüpiter 4. evde → Vimala yok.",
    "Neecha bhanga: düşüş burcunda graha yok → yok.",
    "Nabhasa: yedi grahanın burçları İkizler, Yengeç, Aslan, Başak, Terazi, Oğlak, Kova: 7 farklı burç → Vallaki. Burç türleri: yürük, sabit, çift doğalı → tek tür değil: ashraya yogası yok.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars lagnadan 10., Ay'dan 12., Venüs'ten 8. evde → oluşur: from_moon, from_venus.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): mahapurusha.bhadra, mahapurusha.malavya, lunar.sunapha, lunar.durudhara, lunar.kemadruma, lunar.gajakesari, lunar.chandra_mangala, solar.budha_aditya, raja.dharma_karmadhipati, raja.yogakaraka, dhana.dhana, exchange.maha, exchange.third_lord, exchange.dainya, viparita.sarala, viparita.vimala, neecha_bhanga.neecha_bhanga, nabhasa_count.dama, nabhasa_ashraya.rajju, nabhasa_ashraya.musala, nabhasa_ashraya.nala."
  ],
  "input": {
    "lagna_lon": 10.0,
    "grahas": {"Sun": 125.0, "Moon": 320.0, "Mars": 298.0, "Mercury": 165.0, "Jupiter": 95.0, "Venus": 72.0, "Saturn": 200.0, "Rahu": 33.0, "Ketu": 213.0}
  },
  "expected": {
    "hits": [
      {"id": "mahapurusha.ruchaka", "grahas": ["Mars"], "houses": [10], "relations": [], "conditions": ["exaltation"]},
      {"id": "mahapurusha.hamsa", "grahas": ["Jupiter"], "houses": [4], "relations": [], "conditions": ["exaltation"]},
      {"id": "mahapurusha.shasha", "grahas": ["Saturn"], "houses": [7], "relations": [], "conditions": ["exaltation"]},
      {"id": "lunar.anapha", "grahas": ["Moon", "Mars"], "houses": [10, 11], "relations": [], "conditions": []},
      {"id": "lunar.shakata", "grahas": ["Moon", "Jupiter"], "houses": [4, 11], "relations": [], "conditions": []},
      {"id": "solar.ubhayachari", "grahas": ["Sun", "Mercury", "Jupiter"], "houses": [4, 5, 6], "relations": [], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Sun", "Moon"], "houses": [4, 5], "relations": ["MutualAspect"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mars", "Jupiter"], "houses": [1, 9], "relations": ["MutualAspect"], "conditions": []},
      {"id": "viparita.harsha", "grahas": ["Mercury"], "houses": [6], "relations": [], "conditions": []},
      {"id": "nabhasa_count.vallaki", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [3, 4, 5, 6, 7, 10, 11], "relations": [], "conditions": []},
      {"id": "dosha.kuja", "grahas": ["Mars"], "houses": [10], "relations": [], "conditions": ["from_moon", "from_venus"]}
    ],
    "absent": ["mahapurusha.bhadra", "mahapurusha.malavya", "lunar.sunapha", "lunar.durudhara", "lunar.kemadruma", "lunar.gajakesari", "lunar.chandra_mangala", "solar.budha_aditya", "raja.dharma_karmadhipati", "raja.yogakaraka", "dhana.dhana", "exchange.maha", "exchange.third_lord", "exchange.dainya", "viparita.sarala", "viparita.vimala", "neecha_bhanga.neecha_bhanga", "nabhasa_count.dama", "nabhasa_ashraya.rajju", "nabhasa_ashraya.musala", "nabhasa_ashraya.nala"],
    "cancelled": [],
    "evaluated": 41,
    "skipped_time_unknown": 0
  }
}
````

**Yeni dosya:** `tests/golden/modules/yogas/bhadra-with-both-conditions-malavya-and-hamsa.json`

````json
{
  "id": "bhadra-with-both-conditions-malavya-and-hamsa",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Lagna 76° İkizler. Burçlar ve lagna burcundan sayılan evler: Güneş 182° Terazi (5. ev); Ay 129° Aslan (3. ev); Mars 48° Boğa (12. ev); Merkür 179.5° Başak (4. ev); Jüpiter 240° Yay (7. ev); Venüs 357° Balık (10. ev); Satürn 142° Aslan (3. ev); Rahu 101° Yengeç (2. ev); Ketu 281° Oğlak (8. ev).",
    "Ev lordları (İkizler lagna): 1 Merkür, 2 Ay, 3 Güneş, 4 Merkür, 5 Venüs, 6 Mars, 7 Jüpiter, 8 Satürn, 9 Satürn, 10 Jüpiter, 11 Mars, 12 Venüs.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 307°: küçülüyor, kötücül. Merkür'ün burcundaki diğer grahalar: boş; Merkür iyicil.",
    "Mahapurusha (lagnadan kendra 1, 4, 7, 10). Ruchaka: Mars Boğa (kendi ya da yücelme burcu değil), 12. ev (kendra değil) → yok. Bhadra: Merkür Başak (kendi burcu ve yücelme burcu), 4. ev (kendra) → oluşur. Hamsa: Jüpiter Yay (kendi burcu), 7. ev (kendra) → oluşur. Malavya: Venüs Balık (yücelme burcu), 10. ev (kendra) → oluşur. Shasha: Satürn Aslan (kendi ya da yücelme burcu değil), 3. ev (kendra değil) → yok.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Başak: Merkür; 12. burç Yengeç: boş → Sunapha oluşur; diğer ikisi yok.",
    "Kemadruma: Ay'ın burcunda sayılan graha: Satürn; Ay'dan 4., 7. ve 10. evlerde: Mars; lagnadan kendralarda: Merkür, Jüpiter, Venüs → Kemadruma yok.",
    "Gajakesari: Jüpiter Ay'dan 5. evde (kendra değil) → yok. Chandra-Mangala: Ay Aslan, Mars Boğa → yok.",
    "Adhi: Ay'dan evler: Merkür 2, Jüpiter 5, Venüs 8 → üçü birden 6, 7 ya da 8 değil: yok.",
    "Shakata: Ay Jüpiter'den 9. evde (6, 8 ya da 12 değil); Ay lagnadan 3. evde (kendra değil) → yok.",
    "Amala: lagnadan 10. burç Balık: Venüs iyicil (sağlar); Ay'dan 10. burç Boğa: Mars kötücül (kötücül var, sağlamaz) → oluşur.",
    "Vasumati (upachaya 3, 6, 10, 11): lagnadan Merkür 4, Jüpiter 7, Venüs 10 (üçü birden upachaya değil); Ay'dan Merkür 2, Jüpiter 5, Venüs 8 (üçü birden upachaya değil) → yok.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Akrep: boş; 12. burç Başak: Merkür → Vasi oluşur; diğer ikisi yok. Budha-Aditya: Güneş Terazi, Merkür Başak → yok.",
    "İlişkili çiftler (yedi graha; birlikte = aynı burç, karşılıklı bakış = iki yönlü tam bakış, değişim = birbirinin burcunda): Ay–Satürn (birlikte); Mars–Satürn (karşılıklı bakış); Merkür–Venüs (karşılıklı bakış).",
    "Kendra–trikona: kendra lordları 1 Merkür, 4 Merkür, 7 Jüpiter, 10 Jüpiter; trikona lordları 1 Merkür, 5 Venüs, 9 Satürn. İlişkili çiftlerden bir kendra lordu ile bir trikona lordu olanlar: Merkür–Venüs (evler 1, 4, 5).",
    "Dharma-Karmadhipati: 9. lord Satürn, 10. lord Jüpiter: ilişkili değil → yok. Yogakaraka (lagna dışında bir kendranın ve bir trikonanın lordu): İkizler lagnada yok.",
    "Dhana: 1 Merkür, 2 Ay, 5 Venüs, 9 Satürn, 11 Mars. İlişkili çiftlerden iki lordu da bu kümeden olan ve 2. ya da 11. lordu içerenler: Ay–Satürn (evler 2, 9); Mars–Satürn (evler 9, 11).",
    "Değişim (sınıfı değişen iki ev belirler: 6, 8 ya da 12 varsa Dainya; yoksa 3 varsa 3. ev lordu değişimi; yoksa Maha): değişen çift yok.",
    "Viparita raja (lord 6., 8. ya da 12. evde): 6. lord Mars 12. evde → Harsha oluşur; 8. lord Satürn 3. evde → Sarala yok; 12. lord Venüs 10. evde → Vimala yok.",
    "Neecha bhanga: Güneş düşüş burcunda (Terazi). (1) Burcun lordu Venüs: lagnadan 10., Ay'dan 8. → kendrada. (2) O burçta yücelen Satürn: lagnadan 3., Ay'dan 1. → kendrada. (3) Dispozitör Venüs (Balık): Terazi oradan 8. burç, tam bakış yok → oluşur: sign_lord_in_kendra, exalted_graha_in_kendra.",
    "Nabhasa: yedi grahanın burçları Boğa, Aslan, Başak, Terazi, Yay, Balık: 6 farklı burç → Dama. Burç türleri: yürük, sabit, çift doğalı → tek tür değil: ashraya yogası yok.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars lagnadan 12., Ay'dan 10., Venüs'ten 3. evde → oluşur: from_lagna.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): mahapurusha.ruchaka, mahapurusha.shasha, lunar.anapha, lunar.shakata, lunar.vasumati, solar.vesi, solar.ubhayachari, raja.yogakaraka, nabhasa_count.vallaki."
  ],
  "input": {
    "lagna_lon": 76.0,
    "grahas": {"Sun": 182.0, "Moon": 129.0, "Mars": 48.0, "Mercury": 179.5, "Jupiter": 240.0, "Venus": 357.0, "Saturn": 142.0, "Rahu": 101.0, "Ketu": 281.0}
  },
  "expected": {
    "hits": [
      {"id": "mahapurusha.bhadra", "grahas": ["Mercury"], "houses": [4], "relations": [], "conditions": ["own_sign", "exaltation"]},
      {"id": "mahapurusha.hamsa", "grahas": ["Jupiter"], "houses": [7], "relations": [], "conditions": ["own_sign"]},
      {"id": "mahapurusha.malavya", "grahas": ["Venus"], "houses": [10], "relations": [], "conditions": ["exaltation"]},
      {"id": "lunar.sunapha", "grahas": ["Moon", "Mercury"], "houses": [3, 4], "relations": [], "conditions": []},
      {"id": "lunar.amala", "grahas": ["Venus"], "houses": [10], "relations": [], "conditions": ["from_lagna"]},
      {"id": "solar.vasi", "grahas": ["Sun", "Mercury"], "houses": [4, 5], "relations": [], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mercury", "Venus"], "houses": [1, 4, 5], "relations": ["MutualAspect"], "conditions": []},
      {"id": "dhana.dhana", "grahas": ["Moon", "Saturn"], "houses": [2, 9], "relations": ["Conjunction"], "conditions": []},
      {"id": "dhana.dhana", "grahas": ["Mars", "Saturn"], "houses": [9, 11], "relations": ["MutualAspect"], "conditions": []},
      {"id": "viparita.harsha", "grahas": ["Mars"], "houses": [6, 12], "relations": [], "conditions": []},
      {"id": "neecha_bhanga.neecha_bhanga", "grahas": ["Sun"], "houses": [5], "relations": [], "conditions": ["sign_lord_in_kendra", "exalted_graha_in_kendra"]},
      {"id": "nabhasa_count.dama", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [3, 4, 5, 7, 10, 12], "relations": [], "conditions": []},
      {"id": "dosha.kuja", "grahas": ["Mars"], "houses": [12], "relations": [], "conditions": ["from_lagna"]}
    ],
    "absent": ["mahapurusha.ruchaka", "mahapurusha.shasha", "lunar.anapha", "lunar.shakata", "lunar.vasumati", "solar.vesi", "solar.ubhayachari", "raja.yogakaraka", "nabhasa_count.vallaki"],
    "cancelled": [],
    "evaluated": 41,
    "skipped_time_unknown": 0
  }
}
````

**Yeni dosya:** `tests/golden/modules/yogas/sunapha-vesi-and-shakata-cancelled-by-kendra-moon.json`

````json
{
  "id": "sunapha-vesi-and-shakata-cancelled-by-kendra-moon",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Lagna 93° Yengeç. Burçlar ve lagna burcundan sayılan evler: Güneş 14° Koç (10. ev); Ay 188° Terazi (4. ev); Mars 222° Akrep (5. ev); Merkür 31° Boğa (11. ev); Jüpiter 55° Boğa (11. ev); Venüs 319° Kova (8. ev); Satürn 239° Akrep (5. ev); Rahu 127° Aslan (2. ev); Ketu 307° Kova (8. ev).",
    "Ev lordları (Yengeç lagna): 1 Ay, 2 Güneş, 3 Merkür, 4 Venüs, 5 Mars, 6 Jüpiter, 7 Satürn, 8 Satürn, 9 Jüpiter, 10 Mars, 11 Venüs, 12 Merkür.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 174°: büyüyor, iyicil. Merkür'ün burcundaki diğer grahalar: Jüpiter; Merkür iyicil.",
    "Mahapurusha (lagnadan kendra 1, 4, 7, 10). Ruchaka: Mars Akrep (kendi burcu), 5. ev (kendra değil) → yok. Bhadra: Merkür Boğa (kendi ya da yücelme burcu değil), 11. ev (kendra değil) → yok. Hamsa: Jüpiter Boğa (kendi ya da yücelme burcu değil), 11. ev (kendra değil) → yok. Malavya: Venüs Kova (kendi ya da yücelme burcu değil), 8. ev (kendra değil) → yok. Shasha: Satürn Akrep (kendi ya da yücelme burcu değil), 5. ev (kendra değil) → yok.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Akrep: Mars, Satürn; 12. burç Başak: boş → Sunapha oluşur; diğer ikisi yok.",
    "Kemadruma: Ay'ın burcunda sayılan graha: boş; Ay'dan 4., 7. ve 10. evlerde: boş; lagnadan kendralarda: boş → Kemadruma yok.",
    "Gajakesari: Jüpiter Ay'dan 8. evde (kendra değil) → yok. Chandra-Mangala: Ay Terazi, Mars Akrep → yok.",
    "Adhi: Ay'dan evler: Merkür 8, Jüpiter 8, Venüs 5 → üçü birden 6, 7 ya da 8 değil: yok.",
    "Shakata: Ay Jüpiter'den 6. evde (6, 8 ya da 12); Ay lagnadan 4. evde (kendra) → temel koşul sağlanıyor ama Ay lagnadan kendrada: iptal → yok.",
    "Amala: lagnadan 10. burç Koç: Güneş kötücül (kötücül var, sağlamaz); Ay'dan 10. burç Yengeç: boş → yok.",
    "Vasumati (upachaya 3, 6, 10, 11): lagnadan Merkür 11, Jüpiter 11, Venüs 8 (üçü birden upachaya değil); Ay'dan Merkür 8, Jüpiter 8, Venüs 5 (üçü birden upachaya değil) → yok.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Boğa: Merkür, Jüpiter; 12. burç Balık: boş → Vesi oluşur; diğer ikisi yok. Budha-Aditya: Güneş Koç, Merkür Boğa → yok.",
    "İlişkili çiftler (yedi graha; birlikte = aynı burç, karşılıklı bakış = iki yönlü tam bakış, değişim = birbirinin burcunda): Güneş–Ay (karşılıklı bakış); Mars–Merkür (karşılıklı bakış); Mars–Jüpiter (karşılıklı bakış); Mars–Satürn (birlikte); Merkür–Jüpiter (birlikte); Merkür–Satürn (karşılıklı bakış); Jüpiter–Satürn (karşılıklı bakış).",
    "Kendra–trikona: kendra lordları 1 Ay, 4 Venüs, 7 Satürn, 10 Mars; trikona lordları 1 Ay, 5 Mars, 9 Jüpiter. İlişkili çiftlerden bir kendra lordu ile bir trikona lordu olanlar: Mars–Jüpiter (evler 5, 9, 10); Mars–Satürn (evler 5, 7, 10); Jüpiter–Satürn (evler 7, 9).",
    "Dharma-Karmadhipati: 9. lord Jüpiter, 10. lord Mars: ilişkili (karşılıklı bakış) → oluşur. Yogakaraka (lagna dışında bir kendranın ve bir trikonanın lordu): Mars (5 ve 10. evlerin lordu).",
    "Dhana: 1 Ay, 2 Güneş, 5 Mars, 9 Jüpiter, 11 Venüs. İlişkili çiftlerden iki lordu da bu kümeden olan ve 2. ya da 11. lordu içerenler: Güneş–Ay (evler 1, 2).",
    "Değişim (sınıfı değişen iki ev belirler: 6, 8 ya da 12 varsa Dainya; yoksa 3 varsa 3. ev lordu değişimi; yoksa Maha): değişen çift yok.",
    "Viparita raja (lord 6., 8. ya da 12. evde): 6. lord Jüpiter 11. evde → Harsha yok; 8. lord Satürn 5. evde → Sarala yok; 12. lord Merkür 11. evde → Vimala yok.",
    "Neecha bhanga: düşüş burcunda graha yok → yok.",
    "Nabhasa: yedi grahanın burçları Koç, Boğa, Terazi, Akrep, Kova: 5 farklı burç → Pasha. Burç türleri: yürük, sabit → tek tür değil: ashraya yogası yok.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars lagnadan 5., Ay'dan 2., Venüs'ten 10. evde → oluşur: from_moon.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): mahapurusha.hamsa, lunar.anapha, lunar.adhi, lunar.amala, solar.vasi, solar.ubhayachari, viparita.harsha, nabhasa_count.dama.",
    "İptal örneği olarak işaretlenenler (`cancelled`): lunar.shakata."
  ],
  "input": {
    "lagna_lon": 93.0,
    "grahas": {"Sun": 14.0, "Moon": 188.0, "Mars": 222.0, "Mercury": 31.0, "Jupiter": 55.0, "Venus": 319.0, "Saturn": 239.0, "Rahu": 127.0, "Ketu": 307.0}
  },
  "expected": {
    "hits": [
      {"id": "lunar.sunapha", "grahas": ["Moon", "Mars", "Saturn"], "houses": [4, 5], "relations": [], "conditions": []},
      {"id": "solar.vesi", "grahas": ["Sun", "Mercury", "Jupiter"], "houses": [10, 11], "relations": [], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mars", "Jupiter"], "houses": [5, 9, 10], "relations": ["MutualAspect"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mars", "Saturn"], "houses": [5, 7, 10], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Jupiter", "Saturn"], "houses": [7, 9], "relations": ["MutualAspect"], "conditions": []},
      {"id": "raja.dharma_karmadhipati", "grahas": ["Mars", "Jupiter"], "houses": [9, 10], "relations": ["MutualAspect"], "conditions": []},
      {"id": "raja.yogakaraka", "grahas": ["Mars"], "houses": [5, 10], "relations": [], "conditions": []},
      {"id": "dhana.dhana", "grahas": ["Sun", "Moon"], "houses": [1, 2], "relations": ["MutualAspect"], "conditions": []},
      {"id": "nabhasa_count.pasha", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [4, 5, 8, 10, 11], "relations": [], "conditions": []},
      {"id": "dosha.kuja", "grahas": ["Mars"], "houses": [5], "relations": [], "conditions": ["from_moon"]}
    ],
    "absent": ["mahapurusha.hamsa", "lunar.anapha", "lunar.adhi", "lunar.amala", "solar.vasi", "solar.ubhayachari", "viparita.harsha", "nabhasa_count.dama"],
    "cancelled": ["lunar.shakata"],
    "evaluated": 41,
    "skipped_time_unknown": 0
  }
}
````

**Yeni dosya:** `tests/golden/modules/yogas/anapha-vasi-and-dharma-karmadhipati.json`

````json
{
  "id": "anapha-vasi-and-dharma-karmadhipati",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Lagna 171° Başak. Burçlar ve lagna burcundan sayılan evler: Güneş 280° Oğlak (5. ev); Ay 46° Boğa (9. ev); Mars 4° Koç (8. ev); Merkür 262° Yay (4. ev); Jüpiter 133° Aslan (12. ev); Venüs 242° Yay (4. ev); Satürn 27° Koç (8. ev); Rahu 78° İkizler (10. ev); Ketu 258° Yay (4. ev).",
    "Ev lordları (Başak lagna): 1 Merkür, 2 Venüs, 3 Mars, 4 Jüpiter, 5 Satürn, 6 Satürn, 7 Jüpiter, 8 Mars, 9 Venüs, 10 Merkür, 11 Ay, 12 Güneş.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 126°: büyüyor, iyicil. Merkür'ün burcundaki diğer grahalar: Venüs, Ketu; Merkür iyicil.",
    "Mahapurusha (lagnadan kendra 1, 4, 7, 10). Ruchaka: Mars Koç (kendi burcu), 8. ev (kendra değil) → yok. Bhadra: Merkür Yay (kendi ya da yücelme burcu değil), 4. ev (kendra) → yok. Hamsa: Jüpiter Aslan (kendi ya da yücelme burcu değil), 12. ev (kendra değil) → yok. Malavya: Venüs Yay (kendi ya da yücelme burcu değil), 4. ev (kendra) → yok. Shasha: Satürn Koç (kendi ya da yücelme burcu değil), 8. ev (kendra değil) → yok.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç İkizler: boş; 12. burç Koç: Mars, Satürn → Anapha oluşur; diğer ikisi yok.",
    "Kemadruma: Ay'ın burcunda sayılan graha: boş; Ay'dan 4., 7. ve 10. evlerde: Jüpiter; lagnadan kendralarda: Merkür, Venüs → Kemadruma yok.",
    "Gajakesari: Jüpiter Ay'dan 4. evde (kendra) → oluşur. Chandra-Mangala: Ay Boğa, Mars Koç → yok.",
    "Adhi: Ay'dan evler: Merkür 8, Jüpiter 4, Venüs 8 → üçü birden 6, 7 ya da 8 değil: yok.",
    "Shakata: Ay Jüpiter'den 10. evde (6, 8 ya da 12 değil); Ay lagnadan 9. evde (kendra değil) → yok.",
    "Amala: lagnadan 10. burç İkizler: Rahu kötücül (kötücül var, sağlamaz); Ay'dan 10. burç Kova: boş → yok.",
    "Vasumati (upachaya 3, 6, 10, 11): lagnadan Merkür 4, Jüpiter 12, Venüs 4 (üçü birden upachaya değil); Ay'dan Merkür 8, Jüpiter 4, Venüs 8 (üçü birden upachaya değil) → yok.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Kova: boş; 12. burç Yay: Merkür, Venüs → Vasi oluşur; diğer ikisi yok. Budha-Aditya: Güneş Oğlak, Merkür Yay → yok.",
    "İlişkili çiftler (yedi graha; birlikte = aynı burç, karşılıklı bakış = iki yönlü tam bakış, değişim = birbirinin burcunda): Mars–Satürn (birlikte); Merkür–Venüs (birlikte).",
    "Kendra–trikona: kendra lordları 1 Merkür, 4 Jüpiter, 7 Jüpiter, 10 Merkür; trikona lordları 1 Merkür, 5 Satürn, 9 Venüs. İlişkili çiftlerden bir kendra lordu ile bir trikona lordu olanlar: Merkür–Venüs (evler 1, 9, 10).",
    "Dharma-Karmadhipati: 9. lord Venüs, 10. lord Merkür: ilişkili (birlikte) → oluşur. Yogakaraka (lagna dışında bir kendranın ve bir trikonanın lordu): Başak lagnada yok.",
    "Dhana: 1 Merkür, 2 Venüs, 5 Satürn, 9 Venüs, 11 Ay. İlişkili çiftlerden iki lordu da bu kümeden olan ve 2. ya da 11. lordu içerenler: Merkür–Venüs (evler 1, 2, 9).",
    "Değişim (sınıfı değişen iki ev belirler: 6, 8 ya da 12 varsa Dainya; yoksa 3 varsa 3. ev lordu değişimi; yoksa Maha): değişen çift yok.",
    "Viparita raja (lord 6., 8. ya da 12. evde): 6. lord Satürn 8. evde → Harsha oluşur; 8. lord Mars 8. evde → Sarala oluşur; 12. lord Güneş 5. evde → Vimala yok.",
    "Neecha bhanga: Satürn düşüş burcunda (Koç). (1) Burcun lordu Mars: lagnadan 8., Ay'dan 12. → kendrada değil. (2) O burçta yücelen Güneş: lagnadan 5., Ay'dan 9. → kendrada değil. (3) Dispozitör Mars (Koç): Koç oradan 1. burç, tam bakış yok → hiçbir koşul sağlanmıyor: düşüş iptal olmaz, kayıt yok.",
    "Nabhasa: yedi grahanın burçları Koç, Boğa, Aslan, Yay, Oğlak: 5 farklı burç → Pasha. Burç türleri: yürük, sabit, çift doğalı → tek tür değil: ashraya yogası yok.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars lagnadan 8., Ay'dan 12., Venüs'ten 5. evde → oluşur: from_lagna, from_moon.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): lunar.sunapha, lunar.durudhara, solar.vesi, raja.yogakaraka, viparita.vimala, nabhasa_count.kedara."
  ],
  "input": {
    "lagna_lon": 171.0,
    "grahas": {"Sun": 280.0, "Moon": 46.0, "Mars": 4.0, "Mercury": 262.0, "Jupiter": 133.0, "Venus": 242.0, "Saturn": 27.0, "Rahu": 78.0, "Ketu": 258.0}
  },
  "expected": {
    "hits": [
      {"id": "lunar.anapha", "grahas": ["Moon", "Mars", "Saturn"], "houses": [8, 9], "relations": [], "conditions": []},
      {"id": "lunar.gajakesari", "grahas": ["Moon", "Jupiter"], "houses": [9, 12], "relations": [], "conditions": []},
      {"id": "solar.vasi", "grahas": ["Sun", "Mercury", "Venus"], "houses": [4, 5], "relations": [], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mercury", "Venus"], "houses": [1, 9, 10], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.dharma_karmadhipati", "grahas": ["Mercury", "Venus"], "houses": [9, 10], "relations": ["Conjunction"], "conditions": []},
      {"id": "dhana.dhana", "grahas": ["Mercury", "Venus"], "houses": [1, 2, 9], "relations": ["Conjunction"], "conditions": []},
      {"id": "viparita.harsha", "grahas": ["Saturn"], "houses": [6, 8], "relations": [], "conditions": []},
      {"id": "viparita.sarala", "grahas": ["Mars"], "houses": [8], "relations": [], "conditions": []},
      {"id": "nabhasa_count.pasha", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [4, 5, 8, 9, 12], "relations": [], "conditions": []},
      {"id": "dosha.kuja", "grahas": ["Mars"], "houses": [8], "relations": [], "conditions": ["from_lagna", "from_moon"]}
    ],
    "absent": ["lunar.sunapha", "lunar.durudhara", "solar.vesi", "raja.yogakaraka", "viparita.vimala", "nabhasa_count.kedara"],
    "cancelled": [],
    "evaluated": 41,
    "skipped_time_unknown": 0
  }
}
````

**Yeni dosya:** `tests/golden/modules/yogas/durudhara-ubhayachari-and-maha-exchange-with-mutual-aspect.json`

````json
{
  "id": "durudhara-ubhayachari-and-maha-exchange-with-mutual-aspect",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Lagna 192° Terazi. Burçlar ve lagna burcundan sayılan evler: Güneş 255° Yay (3. ev); Ay 336° Balık (6. ev); Mars 273° Oğlak (4. ev); Merkür 234° Akrep (2. ev); Jüpiter 10° Koç (7. ev); Venüs 317° Kova (5. ev); Satürn 8° Koç (7. ev); Rahu 151° Başak (12. ev); Ketu 331° Balık (6. ev).",
    "Ev lordları (Terazi lagna): 1 Venüs, 2 Mars, 3 Jüpiter, 4 Satürn, 5 Satürn, 6 Jüpiter, 7 Mars, 8 Venüs, 9 Merkür, 10 Ay, 11 Güneş, 12 Merkür.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 81°: büyüyor, iyicil. Merkür'ün burcundaki diğer grahalar: boş; Merkür iyicil.",
    "Mahapurusha (lagnadan kendra 1, 4, 7, 10). Ruchaka: Mars Oğlak (yücelme burcu), 4. ev (kendra) → oluşur. Bhadra: Merkür Akrep (kendi ya da yücelme burcu değil), 2. ev (kendra değil) → yok. Hamsa: Jüpiter Koç (kendi ya da yücelme burcu değil), 7. ev (kendra) → yok. Malavya: Venüs Kova (kendi ya da yücelme burcu değil), 5. ev (kendra değil) → yok. Shasha: Satürn Koç (kendi ya da yücelme burcu değil), 7. ev (kendra) → yok.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Koç: Jüpiter, Satürn; 12. burç Kova: Venüs → Durudhara oluşur; diğer ikisi yok.",
    "Kemadruma: Ay'ın burcunda sayılan graha: boş; Ay'dan 4., 7. ve 10. evlerde: boş; lagnadan kendralarda: Mars, Jüpiter, Satürn → Kemadruma yok.",
    "Gajakesari: Jüpiter Ay'dan 2. evde (kendra değil) → yok. Chandra-Mangala: Ay Balık, Mars Oğlak → yok.",
    "Adhi: Ay'dan evler: Merkür 9, Jüpiter 2, Venüs 12 → üçü birden 6, 7 ya da 8 değil: yok.",
    "Shakata: Ay Jüpiter'den 12. evde (6, 8 ya da 12); Ay lagnadan 6. evde (kendra değil) → oluşur.",
    "Amala: lagnadan 10. burç Yengeç: boş; Ay'dan 10. burç Yay: Güneş kötücül (kötücül var, sağlamaz) → yok.",
    "Vasumati (upachaya 3, 6, 10, 11): lagnadan Merkür 2, Jüpiter 7, Venüs 5 (üçü birden upachaya değil); Ay'dan Merkür 9, Jüpiter 2, Venüs 12 (üçü birden upachaya değil) → yok.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Oğlak: Mars; 12. burç Akrep: Merkür → Ubhayachari oluşur; diğer ikisi yok. Budha-Aditya: Güneş Yay, Merkür Akrep → yok.",
    "İlişkili çiftler (yedi graha; birlikte = aynı burç, karşılıklı bakış = iki yönlü tam bakış, değişim = birbirinin burcunda): Mars–Satürn (karşılıklı bakış, değişim); Jüpiter–Satürn (birlikte).",
    "Kendra–trikona: kendra lordları 1 Venüs, 4 Satürn, 7 Mars, 10 Ay; trikona lordları 1 Venüs, 5 Satürn, 9 Merkür. İlişkili çiftlerden bir kendra lordu ile bir trikona lordu olanlar: Mars–Satürn (evler 4, 5, 7).",
    "Dharma-Karmadhipati: 9. lord Merkür, 10. lord Ay: ilişkili değil → yok. Yogakaraka (lagna dışında bir kendranın ve bir trikonanın lordu): Satürn (4 ve 5. evlerin lordu).",
    "Dhana: 1 Venüs, 2 Mars, 5 Satürn, 9 Merkür, 11 Güneş. İlişkili çiftlerden iki lordu da bu kümeden olan ve 2. ya da 11. lordu içerenler: Mars–Satürn (evler 2, 5).",
    "Değişim (sınıfı değişen iki ev belirler: 6, 8 ya da 12 varsa Dainya; yoksa 3 varsa 3. ev lordu değişimi; yoksa Maha): Mars (Oğlak, 4. ev) ↔ Satürn (Koç, 7. ev) → Maha.",
    "Viparita raja (lord 6., 8. ya da 12. evde): 6. lord Jüpiter 7. evde → Harsha yok; 8. lord Venüs 5. evde → Sarala yok; 12. lord Merkür 2. evde → Vimala yok.",
    "Neecha bhanga: Satürn düşüş burcunda (Koç). (1) Burcun lordu Mars: lagnadan 4., Ay'dan 11. → kendrada. (2) O burçta yücelen Güneş: lagnadan 3., Ay'dan 10. → kendrada. (3) Dispozitör Mars (Oğlak): Koç oradan 4. burç, tam bakış var → oluşur: sign_lord_in_kendra, exalted_graha_in_kendra, dispositor_aspect.",
    "Nabhasa: yedi grahanın burçları Koç, Akrep, Yay, Oğlak, Kova, Balık: 6 farklı burç → Dama. Burç türleri: yürük, sabit, çift doğalı → tek tür değil: ashraya yogası yok.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars lagnadan 4., Ay'dan 11., Venüs'ten 12. evde → oluşur: from_lagna, from_venus.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): lunar.sunapha, lunar.anapha, lunar.gajakesari, solar.vesi, solar.vasi, raja.dharma_karmadhipati, exchange.third_lord, exchange.dainya, nabhasa_count.pasha."
  ],
  "input": {
    "lagna_lon": 192.0,
    "grahas": {"Sun": 255.0, "Moon": 336.0, "Mars": 273.0, "Mercury": 234.0, "Jupiter": 10.0, "Venus": 317.0, "Saturn": 8.0, "Rahu": 151.0, "Ketu": 331.0}
  },
  "expected": {
    "hits": [
      {"id": "mahapurusha.ruchaka", "grahas": ["Mars"], "houses": [4], "relations": [], "conditions": ["exaltation"]},
      {"id": "lunar.durudhara", "grahas": ["Moon", "Jupiter", "Venus", "Saturn"], "houses": [5, 6, 7], "relations": [], "conditions": []},
      {"id": "lunar.shakata", "grahas": ["Moon", "Jupiter"], "houses": [6, 7], "relations": [], "conditions": []},
      {"id": "solar.ubhayachari", "grahas": ["Sun", "Mars", "Mercury"], "houses": [2, 3, 4], "relations": [], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mars", "Saturn"], "houses": [4, 5, 7], "relations": ["MutualAspect", "Exchange"], "conditions": []},
      {"id": "raja.yogakaraka", "grahas": ["Saturn"], "houses": [4, 5], "relations": [], "conditions": []},
      {"id": "dhana.dhana", "grahas": ["Mars", "Saturn"], "houses": [2, 5], "relations": ["MutualAspect", "Exchange"], "conditions": []},
      {"id": "exchange.maha", "grahas": ["Mars", "Saturn"], "houses": [4, 7], "relations": ["Exchange"], "conditions": []},
      {"id": "neecha_bhanga.neecha_bhanga", "grahas": ["Saturn"], "houses": [7], "relations": [], "conditions": ["sign_lord_in_kendra", "exalted_graha_in_kendra", "dispositor_aspect"]},
      {"id": "nabhasa_count.dama", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [2, 3, 4, 5, 6, 7], "relations": [], "conditions": []},
      {"id": "dosha.kuja", "grahas": ["Mars"], "houses": [4], "relations": [], "conditions": ["from_lagna", "from_venus"]}
    ],
    "absent": ["lunar.sunapha", "lunar.anapha", "lunar.gajakesari", "solar.vesi", "solar.vasi", "raja.dharma_karmadhipati", "exchange.third_lord", "exchange.dainya", "nabhasa_count.pasha"],
    "cancelled": [],
    "evaluated": 41,
    "skipped_time_unknown": 0
  }
}
````

**Yeni dosya:** `tests/golden/modules/yogas/kemadruma-kedara-and-debilitated-venus-without-bhanga.json`

````json
{
  "id": "kemadruma-kedara-and-debilitated-venus-without-bhanga",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Lagna 37° Boğa. Burçlar ve lagna burcundan sayılan evler: Güneş 159° Başak (5. ev); Ay 104° Yengeç (3. ev); Mars 266° Yay (8. ev); Merkür 173° Başak (5. ev); Jüpiter 341° Balık (11. ev); Venüs 152° Başak (5. ev); Satürn 245° Yay (8. ev); Rahu 20° Koç (12. ev); Ketu 200° Terazi (6. ev).",
    "Ev lordları (Boğa lagna): 1 Venüs, 2 Merkür, 3 Ay, 4 Güneş, 5 Merkür, 6 Venüs, 7 Mars, 8 Jüpiter, 9 Satürn, 10 Satürn, 11 Jüpiter, 12 Mars.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 305°: küçülüyor, kötücül. Merkür'ün burcundaki diğer grahalar: Güneş, Venüs; Merkür iyicil.",
    "Mahapurusha (lagnadan kendra 1, 4, 7, 10). Ruchaka: Mars Yay (kendi ya da yücelme burcu değil), 8. ev (kendra değil) → yok. Bhadra: Merkür Başak (kendi burcu ve yücelme burcu), 5. ev (kendra değil) → yok. Hamsa: Jüpiter Balık (kendi burcu), 11. ev (kendra değil) → yok. Malavya: Venüs Başak (kendi ya da yücelme burcu değil), 5. ev (kendra değil) → yok. Shasha: Satürn Yay (kendi ya da yücelme burcu değil), 8. ev (kendra değil) → yok.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Aslan: boş; 12. burç İkizler: boş → üçü de yok.",
    "Kemadruma: Ay'ın burcunda sayılan graha: boş; Ay'dan 4., 7. ve 10. evlerde: boş; lagnadan kendralarda: boş → Kemadruma oluşur.",
    "Gajakesari: Jüpiter Ay'dan 9. evde (kendra değil) → yok. Chandra-Mangala: Ay Yengeç, Mars Yay → yok.",
    "Adhi: Ay'dan evler: Merkür 3, Jüpiter 9, Venüs 3 → üçü birden 6, 7 ya da 8 değil: yok.",
    "Shakata: Ay Jüpiter'den 5. evde (6, 8 ya da 12 değil); Ay lagnadan 3. evde (kendra değil) → yok.",
    "Amala: lagnadan 10. burç Kova: boş; Ay'dan 10. burç Koç: Rahu kötücül (kötücül var, sağlamaz) → yok.",
    "Vasumati (upachaya 3, 6, 10, 11): lagnadan Merkür 5, Jüpiter 11, Venüs 5 (üçü birden upachaya değil); Ay'dan Merkür 3, Jüpiter 9, Venüs 3 (üçü birden upachaya değil) → yok.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Terazi: boş; 12. burç Aslan: boş → üçü de yok. Budha-Aditya: Güneş Başak, Merkür Başak → birlikte, oluşur.",
    "İlişkili çiftler (yedi graha; birlikte = aynı burç, karşılıklı bakış = iki yönlü tam bakış, değişim = birbirinin burcunda): Güneş–Merkür (birlikte); Güneş–Jüpiter (karşılıklı bakış); Güneş–Venüs (birlikte); Mars–Satürn (birlikte); Merkür–Jüpiter (karşılıklı bakış); Merkür–Venüs (birlikte); Jüpiter–Venüs (karşılıklı bakış).",
    "Kendra–trikona: kendra lordları 1 Venüs, 4 Güneş, 7 Mars, 10 Satürn; trikona lordları 1 Venüs, 5 Merkür, 9 Satürn. İlişkili çiftlerden bir kendra lordu ile bir trikona lordu olanlar: Güneş–Merkür (evler 4, 5); Güneş–Venüs (evler 1, 4); Mars–Satürn (evler 7, 9, 10); Merkür–Venüs (evler 1, 5).",
    "Dharma-Karmadhipati: 9. ve 10. lord aynı graha (Satürn) → yok. Yogakaraka (lagna dışında bir kendranın ve bir trikonanın lordu): Satürn (9 ve 10. evlerin lordu).",
    "Dhana: 1 Venüs, 2 Merkür, 5 Merkür, 9 Satürn, 11 Jüpiter. İlişkili çiftlerden iki lordu da bu kümeden olan ve 2. ya da 11. lordu içerenler: Merkür–Jüpiter (evler 2, 5, 11); Merkür–Venüs (evler 1, 2, 5); Jüpiter–Venüs (evler 1, 11).",
    "Değişim (sınıfı değişen iki ev belirler: 6, 8 ya da 12 varsa Dainya; yoksa 3 varsa 3. ev lordu değişimi; yoksa Maha): değişen çift yok.",
    "Viparita raja (lord 6., 8. ya da 12. evde): 6. lord Venüs 5. evde → Harsha yok; 8. lord Jüpiter 11. evde → Sarala yok; 12. lord Mars 8. evde → Vimala oluşur.",
    "Neecha bhanga: Venüs düşüş burcunda (Başak). (1) Burcun lordu Merkür: lagnadan 5., Ay'dan 3. → kendrada değil. (2) O burçta yücelen Merkür: lagnadan 5., Ay'dan 3. → kendrada değil. (3) Dispozitör Merkür (Başak): Başak oradan 1. burç, tam bakış yok → hiçbir koşul sağlanmıyor: düşüş iptal olmaz, kayıt yok.",
    "Nabhasa: yedi grahanın burçları Yengeç, Başak, Yay, Balık: 4 farklı burç → Kedara. Burç türleri: yürük, çift doğalı → tek tür değil: ashraya yogası yok.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars lagnadan 8., Ay'dan 6., Venüs'ten 4. evde → oluşur: from_lagna, from_venus.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): lunar.sunapha, lunar.anapha, lunar.durudhara, lunar.shakata, lunar.amala, solar.vesi, solar.vasi, solar.ubhayachari, viparita.harsha, viparita.sarala, nabhasa_count.shula, nabhasa_ashraya.nala.",
    "İptal örneği olarak işaretlenenler (`cancelled`): neecha_bhanga.neecha_bhanga."
  ],
  "input": {
    "lagna_lon": 37.0,
    "grahas": {"Sun": 159.0, "Moon": 104.0, "Mars": 266.0, "Mercury": 173.0, "Jupiter": 341.0, "Venus": 152.0, "Saturn": 245.0, "Rahu": 20.0, "Ketu": 200.0}
  },
  "expected": {
    "hits": [
      {"id": "lunar.kemadruma", "grahas": ["Moon"], "houses": [3], "relations": [], "conditions": []},
      {"id": "solar.budha_aditya", "grahas": ["Sun", "Mercury"], "houses": [5], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Sun", "Mercury"], "houses": [4, 5], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Sun", "Venus"], "houses": [1, 4], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mars", "Saturn"], "houses": [7, 9, 10], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mercury", "Venus"], "houses": [1, 5], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.yogakaraka", "grahas": ["Saturn"], "houses": [9, 10], "relations": [], "conditions": []},
      {"id": "dhana.dhana", "grahas": ["Mercury", "Jupiter"], "houses": [2, 5, 11], "relations": ["MutualAspect"], "conditions": []},
      {"id": "dhana.dhana", "grahas": ["Mercury", "Venus"], "houses": [1, 2, 5], "relations": ["Conjunction"], "conditions": []},
      {"id": "dhana.dhana", "grahas": ["Jupiter", "Venus"], "houses": [1, 11], "relations": ["MutualAspect"], "conditions": []},
      {"id": "viparita.vimala", "grahas": ["Mars"], "houses": [8, 12], "relations": [], "conditions": []},
      {"id": "nabhasa_count.kedara", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [3, 5, 8, 11], "relations": [], "conditions": []},
      {"id": "dosha.kuja", "grahas": ["Mars"], "houses": [8], "relations": [], "conditions": ["from_lagna", "from_venus"]}
    ],
    "absent": ["lunar.sunapha", "lunar.anapha", "lunar.durudhara", "lunar.shakata", "lunar.amala", "solar.vesi", "solar.vasi", "solar.ubhayachari", "viparita.harsha", "viparita.sarala", "nabhasa_count.shula", "nabhasa_ashraya.nala"],
    "cancelled": ["neecha_bhanga.neecha_bhanga"],
    "evaluated": 41,
    "skipped_time_unknown": 0
  }
}
````

**Yeni dosya:** `tests/golden/modules/yogas/kemadruma-cancelled-by-a-graha-in-a-lagna-kendra.json`

````json
{
  "id": "kemadruma-cancelled-by-a-graha-in-a-lagna-kendra",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Lagna 37° Boğa. Burçlar ve lagna burcundan sayılan evler: Güneş 159° Başak (5. ev); Ay 104° Yengeç (3. ev); Mars 266° Yay (8. ev); Merkür 173° Başak (5. ev); Jüpiter 311° Kova (10. ev); Venüs 152° Başak (5. ev); Satürn 245° Yay (8. ev); Rahu 20° Koç (12. ev); Ketu 200° Terazi (6. ev).",
    "Ev lordları (Boğa lagna): 1 Venüs, 2 Merkür, 3 Ay, 4 Güneş, 5 Merkür, 6 Venüs, 7 Mars, 8 Jüpiter, 9 Satürn, 10 Satürn, 11 Jüpiter, 12 Mars.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 305°: küçülüyor, kötücül. Merkür'ün burcundaki diğer grahalar: Güneş, Venüs; Merkür iyicil.",
    "Mahapurusha (lagnadan kendra 1, 4, 7, 10). Ruchaka: Mars Yay (kendi ya da yücelme burcu değil), 8. ev (kendra değil) → yok. Bhadra: Merkür Başak (kendi burcu ve yücelme burcu), 5. ev (kendra değil) → yok. Hamsa: Jüpiter Kova (kendi ya da yücelme burcu değil), 10. ev (kendra) → yok. Malavya: Venüs Başak (kendi ya da yücelme burcu değil), 5. ev (kendra değil) → yok. Shasha: Satürn Yay (kendi ya da yücelme burcu değil), 8. ev (kendra değil) → yok.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Aslan: boş; 12. burç İkizler: boş → üçü de yok.",
    "Kemadruma: Ay'ın burcunda sayılan graha: boş; Ay'dan 4., 7. ve 10. evlerde: boş; lagnadan kendralarda: Jüpiter → temel koşul sağlanıyor ama kendradaki graha iptal eder → Kemadruma yok.",
    "Gajakesari: Jüpiter Ay'dan 8. evde (kendra değil) → yok. Chandra-Mangala: Ay Yengeç, Mars Yay → yok.",
    "Adhi: Ay'dan evler: Merkür 3, Jüpiter 8, Venüs 3 → üçü birden 6, 7 ya da 8 değil: yok.",
    "Shakata: Ay Jüpiter'den 6. evde (6, 8 ya da 12); Ay lagnadan 3. evde (kendra değil) → oluşur.",
    "Amala: lagnadan 10. burç Kova: Jüpiter iyicil (sağlar); Ay'dan 10. burç Koç: Rahu kötücül (kötücül var, sağlamaz) → oluşur.",
    "Vasumati (upachaya 3, 6, 10, 11): lagnadan Merkür 5, Jüpiter 10, Venüs 5 (üçü birden upachaya değil); Ay'dan Merkür 3, Jüpiter 8, Venüs 3 (üçü birden upachaya değil) → yok.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Terazi: boş; 12. burç Aslan: boş → üçü de yok. Budha-Aditya: Güneş Başak, Merkür Başak → birlikte, oluşur.",
    "İlişkili çiftler (yedi graha; birlikte = aynı burç, karşılıklı bakış = iki yönlü tam bakış, değişim = birbirinin burcunda): Güneş–Merkür (birlikte); Güneş–Venüs (birlikte); Mars–Satürn (birlikte); Merkür–Venüs (birlikte); Jüpiter–Satürn (değişim).",
    "Kendra–trikona: kendra lordları 1 Venüs, 4 Güneş, 7 Mars, 10 Satürn; trikona lordları 1 Venüs, 5 Merkür, 9 Satürn. İlişkili çiftlerden bir kendra lordu ile bir trikona lordu olanlar: Güneş–Merkür (evler 4, 5); Güneş–Venüs (evler 1, 4); Mars–Satürn (evler 7, 9, 10); Merkür–Venüs (evler 1, 5).",
    "Dharma-Karmadhipati: 9. ve 10. lord aynı graha (Satürn) → yok. Yogakaraka (lagna dışında bir kendranın ve bir trikonanın lordu): Satürn (9 ve 10. evlerin lordu).",
    "Dhana: 1 Venüs, 2 Merkür, 5 Merkür, 9 Satürn, 11 Jüpiter. İlişkili çiftlerden iki lordu da bu kümeden olan ve 2. ya da 11. lordu içerenler: Merkür–Venüs (evler 1, 2, 5); Jüpiter–Satürn (evler 9, 11).",
    "Değişim (sınıfı değişen iki ev belirler: 6, 8 ya da 12 varsa Dainya; yoksa 3 varsa 3. ev lordu değişimi; yoksa Maha): Jüpiter (Kova, 10. ev) ↔ Satürn (Yay, 8. ev) → Dainya.",
    "Viparita raja (lord 6., 8. ya da 12. evde): 6. lord Venüs 5. evde → Harsha yok; 8. lord Jüpiter 10. evde → Sarala yok; 12. lord Mars 8. evde → Vimala oluşur.",
    "Neecha bhanga: Venüs düşüş burcunda (Başak). (1) Burcun lordu Merkür: lagnadan 5., Ay'dan 3. → kendrada değil. (2) O burçta yücelen Merkür: lagnadan 5., Ay'dan 3. → kendrada değil. (3) Dispozitör Merkür (Başak): Başak oradan 1. burç, tam bakış yok → hiçbir koşul sağlanmıyor: düşüş iptal olmaz, kayıt yok.",
    "Nabhasa: yedi grahanın burçları Yengeç, Başak, Yay, Kova: 4 farklı burç → Kedara. Burç türleri: yürük, sabit, çift doğalı → tek tür değil: ashraya yogası yok.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars lagnadan 8., Ay'dan 6., Venüs'ten 4. evde → oluşur: from_lagna, from_venus.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): lunar.gajakesari, exchange.maha, exchange.third_lord, nabhasa_count.pasha.",
    "İptal örneği olarak işaretlenenler (`cancelled`): lunar.kemadruma, neecha_bhanga.neecha_bhanga."
  ],
  "input": {
    "lagna_lon": 37.0,
    "grahas": {"Sun": 159.0, "Moon": 104.0, "Mars": 266.0, "Mercury": 173.0, "Jupiter": 311.0, "Venus": 152.0, "Saturn": 245.0, "Rahu": 20.0, "Ketu": 200.0}
  },
  "expected": {
    "hits": [
      {"id": "lunar.shakata", "grahas": ["Moon", "Jupiter"], "houses": [3, 10], "relations": [], "conditions": []},
      {"id": "lunar.amala", "grahas": ["Jupiter"], "houses": [10], "relations": [], "conditions": ["from_lagna"]},
      {"id": "solar.budha_aditya", "grahas": ["Sun", "Mercury"], "houses": [5], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Sun", "Mercury"], "houses": [4, 5], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Sun", "Venus"], "houses": [1, 4], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mars", "Saturn"], "houses": [7, 9, 10], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mercury", "Venus"], "houses": [1, 5], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.yogakaraka", "grahas": ["Saturn"], "houses": [9, 10], "relations": [], "conditions": []},
      {"id": "dhana.dhana", "grahas": ["Mercury", "Venus"], "houses": [1, 2, 5], "relations": ["Conjunction"], "conditions": []},
      {"id": "dhana.dhana", "grahas": ["Jupiter", "Saturn"], "houses": [9, 11], "relations": ["Exchange"], "conditions": []},
      {"id": "exchange.dainya", "grahas": ["Jupiter", "Saturn"], "houses": [8, 10], "relations": ["Exchange"], "conditions": []},
      {"id": "viparita.vimala", "grahas": ["Mars"], "houses": [8, 12], "relations": [], "conditions": []},
      {"id": "nabhasa_count.kedara", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [3, 5, 8, 10], "relations": [], "conditions": []},
      {"id": "dosha.kuja", "grahas": ["Mars"], "houses": [8], "relations": [], "conditions": ["from_lagna", "from_venus"]}
    ],
    "absent": ["lunar.gajakesari", "exchange.maha", "exchange.third_lord", "nabhasa_count.pasha"],
    "cancelled": ["lunar.kemadruma", "neecha_bhanga.neecha_bhanga"],
    "evaluated": 41,
    "skipped_time_unknown": 0
  }
}
````

**Yeni dosya:** `tests/golden/modules/yogas/yuga-and-musala-two-fixed-signs.json`

````json
{
  "id": "yuga-and-musala-two-fixed-signs",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Lagna 339° Balık. Burçlar ve lagna burcundan sayılan evler: Güneş 48° Boğa (3. ev); Ay 304° Kova (12. ev); Mars 51° Boğa (3. ev); Merkür 33° Boğa (3. ev); Jüpiter 327° Kova (12. ev); Venüs 40° Boğa (3. ev); Satürn 43° Boğa (3. ev); Rahu 69° İkizler (4. ev); Ketu 249° Yay (10. ev).",
    "Ev lordları (Balık lagna): 1 Jüpiter, 2 Mars, 3 Venüs, 4 Merkür, 5 Ay, 6 Güneş, 7 Merkür, 8 Venüs, 9 Mars, 10 Jüpiter, 11 Satürn, 12 Satürn.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 256°: küçülüyor, kötücül. Merkür'ün burcundaki diğer grahalar: Güneş, Mars, Venüs, Satürn; Merkür iyicil.",
    "Mahapurusha (lagnadan kendra 1, 4, 7, 10). Ruchaka: Mars Boğa (kendi ya da yücelme burcu değil), 3. ev (kendra değil) → yok. Bhadra: Merkür Boğa (kendi ya da yücelme burcu değil), 3. ev (kendra değil) → yok. Hamsa: Jüpiter Kova (kendi ya da yücelme burcu değil), 12. ev (kendra değil) → yok. Malavya: Venüs Boğa (kendi burcu), 3. ev (kendra değil) → yok. Shasha: Satürn Boğa (kendi ya da yücelme burcu değil), 3. ev (kendra değil) → yok.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Balık: boş; 12. burç Oğlak: boş → üçü de yok.",
    "Kemadruma: Ay'ın burcunda sayılan graha: Jüpiter; Ay'dan 4., 7. ve 10. evlerde: Mars, Merkür, Venüs, Satürn; lagnadan kendralarda: boş → Kemadruma yok.",
    "Gajakesari: Jüpiter Ay'dan 1. evde (kendra) → oluşur. Chandra-Mangala: Ay Kova, Mars Boğa → yok.",
    "Adhi: Ay'dan evler: Merkür 4, Jüpiter 1, Venüs 4 → üçü birden 6, 7 ya da 8 değil: yok.",
    "Shakata: Ay Jüpiter'den 1. evde (6, 8 ya da 12 değil); Ay lagnadan 12. evde (kendra değil) → yok.",
    "Amala: lagnadan 10. burç Yay: Ketu kötücül (kötücül var, sağlamaz); Ay'dan 10. burç Akrep: boş → yok.",
    "Vasumati (upachaya 3, 6, 10, 11): lagnadan Merkür 3, Jüpiter 12, Venüs 3 (üçü birden upachaya değil); Ay'dan Merkür 4, Jüpiter 1, Venüs 4 (üçü birden upachaya değil) → yok.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç İkizler: boş; 12. burç Koç: boş → üçü de yok. Budha-Aditya: Güneş Boğa, Merkür Boğa → birlikte, oluşur.",
    "İlişkili çiftler (yedi graha; birlikte = aynı burç, karşılıklı bakış = iki yönlü tam bakış, değişim = birbirinin burcunda): Güneş–Mars (birlikte); Güneş–Merkür (birlikte); Güneş–Venüs (birlikte); Güneş–Satürn (birlikte); Ay–Jüpiter (birlikte); Mars–Merkür (birlikte); Mars–Venüs (birlikte); Mars–Satürn (birlikte); Merkür–Venüs (birlikte); Merkür–Satürn (birlikte); Venüs–Satürn (birlikte).",
    "Kendra–trikona: kendra lordları 1 Jüpiter, 4 Merkür, 7 Merkür, 10 Jüpiter; trikona lordları 1 Jüpiter, 5 Ay, 9 Mars. İlişkili çiftlerden bir kendra lordu ile bir trikona lordu olanlar: Ay–Jüpiter (evler 1, 5, 10); Mars–Merkür (evler 4, 7, 9).",
    "Dharma-Karmadhipati: 9. lord Mars, 10. lord Jüpiter: ilişkili değil → yok. Yogakaraka (lagna dışında bir kendranın ve bir trikonanın lordu): Balık lagnada yok.",
    "Dhana: 1 Jüpiter, 2 Mars, 5 Ay, 9 Mars, 11 Satürn. İlişkili çiftlerden iki lordu da bu kümeden olan ve 2. ya da 11. lordu içerenler: Mars–Satürn (evler 2, 9, 11).",
    "Değişim (sınıfı değişen iki ev belirler: 6, 8 ya da 12 varsa Dainya; yoksa 3 varsa 3. ev lordu değişimi; yoksa Maha): değişen çift yok.",
    "Viparita raja (lord 6., 8. ya da 12. evde): 6. lord Güneş 3. evde → Harsha yok; 8. lord Venüs 3. evde → Sarala yok; 12. lord Satürn 3. evde → Vimala yok.",
    "Neecha bhanga: düşüş burcunda graha yok → yok.",
    "Nabhasa: yedi grahanın burçları Boğa, Kova: 2 farklı burç → Yuga. Burç türleri: sabit → hepsi sabit: Musala oluşur.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars lagnadan 3., Ay'dan 4., Venüs'ten 1. evde → oluşur: from_moon, from_venus.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): lunar.chandra_mangala, lunar.adhi, lunar.shakata, raja.dharma_karmadhipati, raja.yogakaraka, exchange.maha, viparita.harsha, nabhasa_count.shula, nabhasa_count.gola, nabhasa_ashraya.rajju, nabhasa_ashraya.nala."
  ],
  "input": {
    "lagna_lon": 339.0,
    "grahas": {"Sun": 48.0, "Moon": 304.0, "Mars": 51.0, "Mercury": 33.0, "Jupiter": 327.0, "Venus": 40.0, "Saturn": 43.0, "Rahu": 69.0, "Ketu": 249.0}
  },
  "expected": {
    "hits": [
      {"id": "lunar.gajakesari", "grahas": ["Moon", "Jupiter"], "houses": [12], "relations": [], "conditions": []},
      {"id": "solar.budha_aditya", "grahas": ["Sun", "Mercury"], "houses": [3], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Moon", "Jupiter"], "houses": [1, 5, 10], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mars", "Mercury"], "houses": [4, 7, 9], "relations": ["Conjunction"], "conditions": []},
      {"id": "dhana.dhana", "grahas": ["Mars", "Saturn"], "houses": [2, 9, 11], "relations": ["Conjunction"], "conditions": []},
      {"id": "nabhasa_count.yuga", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [3, 12], "relations": [], "conditions": []},
      {"id": "nabhasa_ashraya.musala", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [3, 12], "relations": [], "conditions": []},
      {"id": "dosha.kuja", "grahas": ["Mars"], "houses": [3], "relations": [], "conditions": ["from_moon", "from_venus"]}
    ],
    "absent": ["lunar.chandra_mangala", "lunar.adhi", "lunar.shakata", "raja.dharma_karmadhipati", "raja.yogakaraka", "exchange.maha", "viparita.harsha", "nabhasa_count.shula", "nabhasa_count.gola", "nabhasa_ashraya.rajju", "nabhasa_ashraya.nala"],
    "cancelled": [],
    "evaluated": 41,
    "skipped_time_unknown": 0
  }
}
````

**Yeni dosya:** `tests/golden/modules/yogas/adhi-and-chandra-mangala-ruchaka-in-own-sign.json`

````json
{
  "id": "adhi-and-chandra-mangala-ruchaka-in-own-sign",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Lagna 292° Oğlak. Burçlar ve lagna burcundan sayılan evler: Güneş 181° Terazi (10. ev); Ay 20° Koç (4. ev); Mars 7° Koç (4. ev); Merkür 166° Başak (9. ev); Jüpiter 223° Akrep (11. ev); Venüs 204° Terazi (10. ev); Satürn 65° İkizler (6. ev); Rahu 148° Aslan (8. ev); Ketu 328° Kova (2. ev).",
    "Ev lordları (Oğlak lagna): 1 Satürn, 2 Satürn, 3 Jüpiter, 4 Mars, 5 Venüs, 6 Merkür, 7 Ay, 8 Güneş, 9 Merkür, 10 Venüs, 11 Mars, 12 Jüpiter.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 199°: küçülüyor, kötücül. Merkür'ün burcundaki diğer grahalar: boş; Merkür iyicil.",
    "Mahapurusha (lagnadan kendra 1, 4, 7, 10). Ruchaka: Mars Koç (kendi burcu), 4. ev (kendra) → oluşur. Bhadra: Merkür Başak (kendi burcu ve yücelme burcu), 9. ev (kendra değil) → yok. Hamsa: Jüpiter Akrep (kendi ya da yücelme burcu değil), 11. ev (kendra değil) → yok. Malavya: Venüs Terazi (kendi burcu), 10. ev (kendra) → oluşur. Shasha: Satürn İkizler (kendi ya da yücelme burcu değil), 6. ev (kendra değil) → yok.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Boğa: boş; 12. burç Balık: boş → üçü de yok.",
    "Kemadruma: Ay'ın burcunda sayılan graha: Mars; Ay'dan 4., 7. ve 10. evlerde: Venüs; lagnadan kendralarda: Mars, Venüs → Kemadruma yok.",
    "Gajakesari: Jüpiter Ay'dan 8. evde (kendra değil) → yok. Chandra-Mangala: Ay Koç, Mars Koç → birlikte, oluşur.",
    "Adhi: Ay'dan evler: Merkür 6, Jüpiter 8, Venüs 7 → üçü de 6, 7 ya da 8: oluşur.",
    "Shakata: Ay Jüpiter'den 6. evde (6, 8 ya da 12); Ay lagnadan 4. evde (kendra) → temel koşul sağlanıyor ama Ay lagnadan kendrada: iptal → yok.",
    "Amala: lagnadan 10. burç Terazi: Güneş kötücül, Venüs iyicil (kötücül var, sağlamaz); Ay'dan 10. burç Oğlak: boş → yok.",
    "Vasumati (upachaya 3, 6, 10, 11): lagnadan Merkür 9, Jüpiter 11, Venüs 10 (üçü birden upachaya değil); Ay'dan Merkür 6, Jüpiter 8, Venüs 7 (üçü birden upachaya değil) → yok.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Akrep: Jüpiter; 12. burç Başak: Merkür → Ubhayachari oluşur; diğer ikisi yok. Budha-Aditya: Güneş Terazi, Merkür Başak → yok.",
    "İlişkili çiftler (yedi graha; birlikte = aynı burç, karşılıklı bakış = iki yönlü tam bakış, değişim = birbirinin burcunda): Güneş–Ay (karşılıklı bakış); Güneş–Mars (karşılıklı bakış); Güneş–Venüs (birlikte); Ay–Mars (birlikte); Ay–Venüs (karşılıklı bakış); Mars–Venüs (karşılıklı bakış).",
    "Kendra–trikona: kendra lordları 1 Satürn, 4 Mars, 7 Ay, 10 Venüs; trikona lordları 1 Satürn, 5 Venüs, 9 Merkür. İlişkili çiftlerden bir kendra lordu ile bir trikona lordu olanlar: Ay–Venüs (evler 5, 7, 10); Mars–Venüs (evler 4, 5, 10).",
    "Dharma-Karmadhipati: 9. lord Merkür, 10. lord Venüs: ilişkili değil → yok. Yogakaraka (lagna dışında bir kendranın ve bir trikonanın lordu): Venüs (5 ve 10. evlerin lordu).",
    "Dhana: 1 Satürn, 2 Satürn, 5 Venüs, 9 Merkür, 11 Mars. İlişkili çiftlerden iki lordu da bu kümeden olan ve 2. ya da 11. lordu içerenler: Mars–Venüs (evler 5, 11).",
    "Değişim (sınıfı değişen iki ev belirler: 6, 8 ya da 12 varsa Dainya; yoksa 3 varsa 3. ev lordu değişimi; yoksa Maha): değişen çift yok.",
    "Viparita raja (lord 6., 8. ya da 12. evde): 6. lord Merkür 9. evde → Harsha yok; 8. lord Güneş 10. evde → Sarala yok; 12. lord Jüpiter 11. evde → Vimala yok.",
    "Neecha bhanga: Güneş düşüş burcunda (Terazi). (1) Burcun lordu Venüs: lagnadan 10., Ay'dan 7. → kendrada. (2) O burçta yücelen Satürn: lagnadan 6., Ay'dan 3. → kendrada değil. (3) Dispozitör Venüs (Terazi): Terazi oradan 1. burç, tam bakış yok → oluşur: sign_lord_in_kendra.",
    "Nabhasa: yedi grahanın burçları Koç, İkizler, Başak, Terazi, Akrep: 5 farklı burç → Pasha. Burç türleri: yürük, sabit, çift doğalı → tek tür değil: ashraya yogası yok.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars lagnadan 4., Ay'dan 1., Venüs'ten 7. evde → oluşur: from_lagna, from_moon, from_venus.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): mahapurusha.hamsa, mahapurusha.shasha, lunar.gajakesari, lunar.vasumati, solar.budha_aditya, nabhasa_count.kedara."
  ],
  "input": {
    "lagna_lon": 292.0,
    "grahas": {"Sun": 181.0, "Moon": 20.0, "Mars": 7.0, "Mercury": 166.0, "Jupiter": 223.0, "Venus": 204.0, "Saturn": 65.0, "Rahu": 148.0, "Ketu": 328.0}
  },
  "expected": {
    "hits": [
      {"id": "mahapurusha.ruchaka", "grahas": ["Mars"], "houses": [4], "relations": [], "conditions": ["own_sign"]},
      {"id": "mahapurusha.malavya", "grahas": ["Venus"], "houses": [10], "relations": [], "conditions": ["own_sign"]},
      {"id": "lunar.chandra_mangala", "grahas": ["Moon", "Mars"], "houses": [4], "relations": ["Conjunction"], "conditions": []},
      {"id": "lunar.adhi", "grahas": ["Moon", "Mercury", "Jupiter", "Venus"], "houses": [4, 9, 10, 11], "relations": [], "conditions": []},
      {"id": "solar.ubhayachari", "grahas": ["Sun", "Mercury", "Jupiter"], "houses": [9, 10, 11], "relations": [], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Moon", "Venus"], "houses": [5, 7, 10], "relations": ["MutualAspect"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mars", "Venus"], "houses": [4, 5, 10], "relations": ["MutualAspect"], "conditions": []},
      {"id": "raja.yogakaraka", "grahas": ["Venus"], "houses": [5, 10], "relations": [], "conditions": []},
      {"id": "dhana.dhana", "grahas": ["Mars", "Venus"], "houses": [5, 11], "relations": ["MutualAspect"], "conditions": []},
      {"id": "neecha_bhanga.neecha_bhanga", "grahas": ["Sun"], "houses": [10], "relations": [], "conditions": ["sign_lord_in_kendra"]},
      {"id": "nabhasa_count.pasha", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [4, 6, 9, 10, 11], "relations": [], "conditions": []},
      {"id": "dosha.kuja", "grahas": ["Mars"], "houses": [4], "relations": [], "conditions": ["from_lagna", "from_moon", "from_venus"]}
    ],
    "absent": ["mahapurusha.hamsa", "mahapurusha.shasha", "lunar.gajakesari", "lunar.vasumati", "solar.budha_aditya", "nabhasa_count.kedara"],
    "cancelled": [],
    "evaluated": 41,
    "skipped_time_unknown": 0
  }
}
````

**Yeni dosya:** `tests/golden/modules/yogas/shula-and-nala-three-dual-signs.json`

````json
{
  "id": "shula-and-nala-three-dual-signs",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Lagna 348° Balık. Burçlar ve lagna burcundan sayılan evler: Güneş 68° İkizler (4. ev); Ay 252° Yay (10. ev); Mars 156° Başak (7. ev); Merkür 79° İkizler (4. ev); Jüpiter 268° Yay (10. ev); Venüs 61° İkizler (4. ev); Satürn 174° Başak (7. ev); Rahu 315° Kova (12. ev); Ketu 135° Aslan (6. ev).",
    "Ev lordları (Balık lagna): 1 Jüpiter, 2 Mars, 3 Venüs, 4 Merkür, 5 Ay, 6 Güneş, 7 Merkür, 8 Venüs, 9 Mars, 10 Jüpiter, 11 Satürn, 12 Satürn.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 184°: küçülüyor, kötücül. Merkür'ün burcundaki diğer grahalar: Güneş, Venüs; Merkür iyicil.",
    "Mahapurusha (lagnadan kendra 1, 4, 7, 10). Ruchaka: Mars Başak (kendi ya da yücelme burcu değil), 7. ev (kendra) → yok. Bhadra: Merkür İkizler (kendi burcu), 4. ev (kendra) → oluşur. Hamsa: Jüpiter Yay (kendi burcu), 10. ev (kendra) → oluşur. Malavya: Venüs İkizler (kendi ya da yücelme burcu değil), 4. ev (kendra) → yok. Shasha: Satürn Başak (kendi ya da yücelme burcu değil), 7. ev (kendra) → yok.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Oğlak: boş; 12. burç Akrep: boş → üçü de yok.",
    "Kemadruma: Ay'ın burcunda sayılan graha: Jüpiter; Ay'dan 4., 7. ve 10. evlerde: Mars, Merkür, Venüs, Satürn; lagnadan kendralarda: Mars, Merkür, Jüpiter, Venüs, Satürn → Kemadruma yok.",
    "Gajakesari: Jüpiter Ay'dan 1. evde (kendra) → oluşur. Chandra-Mangala: Ay Yay, Mars Başak → yok.",
    "Adhi: Ay'dan evler: Merkür 7, Jüpiter 1, Venüs 7 → üçü birden 6, 7 ya da 8 değil: yok.",
    "Shakata: Ay Jüpiter'den 1. evde (6, 8 ya da 12 değil); Ay lagnadan 10. evde (kendra) → yok.",
    "Amala: lagnadan 10. burç Yay: Ay kötücül, Jüpiter iyicil (kötücül var, sağlamaz); Ay'dan 10. burç Başak: Mars kötücül, Satürn kötücül (kötücül var, sağlamaz) → yok.",
    "Vasumati (upachaya 3, 6, 10, 11): lagnadan Merkür 4, Jüpiter 10, Venüs 4 (üçü birden upachaya değil); Ay'dan Merkür 7, Jüpiter 1, Venüs 7 (üçü birden upachaya değil) → yok.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Yengeç: boş; 12. burç Boğa: boş → üçü de yok. Budha-Aditya: Güneş İkizler, Merkür İkizler → birlikte, oluşur.",
    "İlişkili çiftler (yedi graha; birlikte = aynı burç, karşılıklı bakış = iki yönlü tam bakış, değişim = birbirinin burcunda): Güneş–Ay (karşılıklı bakış); Güneş–Merkür (birlikte); Güneş–Jüpiter (karşılıklı bakış); Güneş–Venüs (birlikte); Ay–Merkür (karşılıklı bakış); Ay–Jüpiter (birlikte); Ay–Venüs (karşılıklı bakış); Mars–Satürn (birlikte); Merkür–Jüpiter (karşılıklı bakış); Merkür–Venüs (birlikte); Jüpiter–Venüs (karşılıklı bakış).",
    "Kendra–trikona: kendra lordları 1 Jüpiter, 4 Merkür, 7 Merkür, 10 Jüpiter; trikona lordları 1 Jüpiter, 5 Ay, 9 Mars. İlişkili çiftlerden bir kendra lordu ile bir trikona lordu olanlar: Ay–Merkür (evler 4, 5, 7); Ay–Jüpiter (evler 1, 5, 10); Merkür–Jüpiter (evler 1, 4, 7, 10).",
    "Dharma-Karmadhipati: 9. lord Mars, 10. lord Jüpiter: ilişkili değil → yok. Yogakaraka (lagna dışında bir kendranın ve bir trikonanın lordu): Balık lagnada yok.",
    "Dhana: 1 Jüpiter, 2 Mars, 5 Ay, 9 Mars, 11 Satürn. İlişkili çiftlerden iki lordu da bu kümeden olan ve 2. ya da 11. lordu içerenler: Mars–Satürn (evler 2, 9, 11).",
    "Değişim (sınıfı değişen iki ev belirler: 6, 8 ya da 12 varsa Dainya; yoksa 3 varsa 3. ev lordu değişimi; yoksa Maha): değişen çift yok.",
    "Viparita raja (lord 6., 8. ya da 12. evde): 6. lord Güneş 4. evde → Harsha yok; 8. lord Venüs 4. evde → Sarala yok; 12. lord Satürn 7. evde → Vimala yok.",
    "Neecha bhanga: düşüş burcunda graha yok → yok.",
    "Nabhasa: yedi grahanın burçları İkizler, Başak, Yay: 3 farklı burç → Shula. Burç türleri: çift doğalı → hepsi çift doğalı: Nala oluşur.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars lagnadan 7., Ay'dan 10., Venüs'ten 4. evde → oluşur: from_lagna, from_venus.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): mahapurusha.malavya, lunar.chandra_mangala, lunar.adhi, nabhasa_count.kedara, nabhasa_count.yuga, nabhasa_ashraya.rajju, nabhasa_ashraya.musala."
  ],
  "input": {
    "lagna_lon": 348.0,
    "grahas": {"Sun": 68.0, "Moon": 252.0, "Mars": 156.0, "Mercury": 79.0, "Jupiter": 268.0, "Venus": 61.0, "Saturn": 174.0, "Rahu": 315.0, "Ketu": 135.0}
  },
  "expected": {
    "hits": [
      {"id": "mahapurusha.bhadra", "grahas": ["Mercury"], "houses": [4], "relations": [], "conditions": ["own_sign"]},
      {"id": "mahapurusha.hamsa", "grahas": ["Jupiter"], "houses": [10], "relations": [], "conditions": ["own_sign"]},
      {"id": "lunar.gajakesari", "grahas": ["Moon", "Jupiter"], "houses": [10], "relations": [], "conditions": []},
      {"id": "solar.budha_aditya", "grahas": ["Sun", "Mercury"], "houses": [4], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Moon", "Mercury"], "houses": [4, 5, 7], "relations": ["MutualAspect"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Moon", "Jupiter"], "houses": [1, 5, 10], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mercury", "Jupiter"], "houses": [1, 4, 7, 10], "relations": ["MutualAspect"], "conditions": []},
      {"id": "dhana.dhana", "grahas": ["Mars", "Saturn"], "houses": [2, 9, 11], "relations": ["Conjunction"], "conditions": []},
      {"id": "nabhasa_count.shula", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [4, 7, 10], "relations": [], "conditions": []},
      {"id": "nabhasa_ashraya.nala", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [4, 7, 10], "relations": [], "conditions": []},
      {"id": "dosha.kuja", "grahas": ["Mars"], "houses": [7], "relations": [], "conditions": ["from_lagna", "from_venus"]}
    ],
    "absent": ["mahapurusha.malavya", "lunar.chandra_mangala", "lunar.adhi", "nabhasa_count.kedara", "nabhasa_count.yuga", "nabhasa_ashraya.rajju", "nabhasa_ashraya.musala"],
    "cancelled": [],
    "evaluated": 41,
    "skipped_time_unknown": 0
  }
}
````

**Yeni dosya:** `tests/golden/modules/yogas/three-exchanges-one-of-each-class.json`

````json
{
  "id": "three-exchanges-one-of-each-class",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Lagna 131° Aslan. Burçlar ve lagna burcundan sayılan evler: Güneş 179° Başak (2. ev); Ay 287° Oğlak (6. ev); Mars 189° Terazi (3. ev); Merkür 140° Aslan (1. ev); Jüpiter 334° Balık (8. ev); Venüs 215° Akrep (4. ev); Satürn 113° Yengeç (12. ev); Rahu 66° İkizler (11. ev); Ketu 246° Yay (5. ev).",
    "Ev lordları (Aslan lagna): 1 Güneş, 2 Merkür, 3 Venüs, 4 Mars, 5 Jüpiter, 6 Satürn, 7 Satürn, 8 Jüpiter, 9 Mars, 10 Venüs, 11 Merkür, 12 Ay.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 108°: büyüyor, iyicil. Merkür'ün burcundaki diğer grahalar: boş; Merkür iyicil.",
    "Mahapurusha (lagnadan kendra 1, 4, 7, 10). Ruchaka: Mars Terazi (kendi ya da yücelme burcu değil), 3. ev (kendra değil) → yok. Bhadra: Merkür Aslan (kendi ya da yücelme burcu değil), 1. ev (kendra) → yok. Hamsa: Jüpiter Balık (kendi burcu), 8. ev (kendra değil) → yok. Malavya: Venüs Akrep (kendi ya da yücelme burcu değil), 4. ev (kendra) → yok. Shasha: Satürn Yengeç (kendi ya da yücelme burcu değil), 12. ev (kendra değil) → yok.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Kova: boş; 12. burç Yay: boş → üçü de yok.",
    "Kemadruma: Ay'ın burcunda sayılan graha: boş; Ay'dan 4., 7. ve 10. evlerde: Mars, Satürn; lagnadan kendralarda: Merkür, Venüs → temel koşul sağlanıyor ama kendradaki graha iptal eder → Kemadruma yok.",
    "Gajakesari: Jüpiter Ay'dan 3. evde (kendra değil) → yok. Chandra-Mangala: Ay Oğlak, Mars Terazi → yok.",
    "Adhi: Ay'dan evler: Merkür 8, Jüpiter 3, Venüs 11 → üçü birden 6, 7 ya da 8 değil: yok.",
    "Shakata: Ay Jüpiter'den 11. evde (6, 8 ya da 12 değil); Ay lagnadan 6. evde (kendra değil) → yok.",
    "Amala: lagnadan 10. burç Boğa: boş; Ay'dan 10. burç Terazi: Mars kötücül (kötücül var, sağlamaz) → yok.",
    "Vasumati (upachaya 3, 6, 10, 11): lagnadan Merkür 1, Jüpiter 8, Venüs 4 (üçü birden upachaya değil); Ay'dan Merkür 8, Jüpiter 3, Venüs 11 (üçü birden upachaya değil) → yok.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Terazi: Mars; 12. burç Aslan: Merkür → Ubhayachari oluşur; diğer ikisi yok. Budha-Aditya: Güneş Başak, Merkür Aslan → yok.",
    "İlişkili çiftler (yedi graha; birlikte = aynı burç, karşılıklı bakış = iki yönlü tam bakış, değişim = birbirinin burcunda): Güneş–Merkür (değişim); Güneş–Jüpiter (karşılıklı bakış); Ay–Satürn (karşılıklı bakış, değişim); Mars–Venüs (değişim).",
    "Kendra–trikona: kendra lordları 1 Güneş, 4 Mars, 7 Satürn, 10 Venüs; trikona lordları 1 Güneş, 5 Jüpiter, 9 Mars. İlişkili çiftlerden bir kendra lordu ile bir trikona lordu olanlar: Güneş–Jüpiter (evler 1, 5); Mars–Venüs (evler 4, 9, 10).",
    "Dharma-Karmadhipati: 9. lord Mars, 10. lord Venüs: ilişkili (değişim) → oluşur. Yogakaraka (lagna dışında bir kendranın ve bir trikonanın lordu): Mars (4 ve 9. evlerin lordu).",
    "Dhana: 1 Güneş, 2 Merkür, 5 Jüpiter, 9 Mars, 11 Merkür. İlişkili çiftlerden iki lordu da bu kümeden olan ve 2. ya da 11. lordu içerenler: Güneş–Merkür (evler 1, 2, 11).",
    "Değişim (sınıfı değişen iki ev belirler: 6, 8 ya da 12 varsa Dainya; yoksa 3 varsa 3. ev lordu değişimi; yoksa Maha): Güneş (Başak, 2. ev) ↔ Merkür (Aslan, 1. ev) → Maha; Mars (Terazi, 3. ev) ↔ Venüs (Akrep, 4. ev) → 3. ev lordu değişimi; Ay (Oğlak, 6. ev) ↔ Satürn (Yengeç, 12. ev) → Dainya.",
    "Viparita raja (lord 6., 8. ya da 12. evde): 6. lord Satürn 12. evde → Harsha oluşur; 8. lord Jüpiter 8. evde → Sarala oluşur; 12. lord Ay 6. evde → Vimala oluşur.",
    "Neecha bhanga: düşüş burcunda graha yok → yok.",
    "Nabhasa: yedi grahanın burçları Yengeç, Aslan, Başak, Terazi, Akrep, Oğlak, Balık: 7 farklı burç → Vallaki. Burç türleri: yürük, sabit, çift doğalı → tek tür değil: ashraya yogası yok.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars lagnadan 3., Ay'dan 10., Venüs'ten 12. evde → oluşur: from_venus.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): mahapurusha.ruchaka, lunar.gajakesari, lunar.amala, solar.budha_aditya, nabhasa_count.dama."
  ],
  "input": {
    "lagna_lon": 131.0,
    "grahas": {"Sun": 179.0, "Moon": 287.0, "Mars": 189.0, "Mercury": 140.0, "Jupiter": 334.0, "Venus": 215.0, "Saturn": 113.0, "Rahu": 66.0, "Ketu": 246.0}
  },
  "expected": {
    "hits": [
      {"id": "solar.ubhayachari", "grahas": ["Sun", "Mars", "Mercury"], "houses": [1, 2, 3], "relations": [], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Sun", "Jupiter"], "houses": [1, 5], "relations": ["MutualAspect"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mars", "Venus"], "houses": [4, 9, 10], "relations": ["Exchange"], "conditions": []},
      {"id": "raja.dharma_karmadhipati", "grahas": ["Mars", "Venus"], "houses": [9, 10], "relations": ["Exchange"], "conditions": []},
      {"id": "raja.yogakaraka", "grahas": ["Mars"], "houses": [4, 9], "relations": [], "conditions": []},
      {"id": "dhana.dhana", "grahas": ["Sun", "Mercury"], "houses": [1, 2, 11], "relations": ["Exchange"], "conditions": []},
      {"id": "exchange.maha", "grahas": ["Sun", "Mercury"], "houses": [1, 2], "relations": ["Exchange"], "conditions": []},
      {"id": "exchange.third_lord", "grahas": ["Mars", "Venus"], "houses": [3, 4], "relations": ["Exchange"], "conditions": []},
      {"id": "exchange.dainya", "grahas": ["Moon", "Saturn"], "houses": [6, 12], "relations": ["Exchange"], "conditions": []},
      {"id": "viparita.harsha", "grahas": ["Saturn"], "houses": [6, 12], "relations": [], "conditions": []},
      {"id": "viparita.sarala", "grahas": ["Jupiter"], "houses": [8], "relations": [], "conditions": []},
      {"id": "viparita.vimala", "grahas": ["Moon"], "houses": [6, 12], "relations": [], "conditions": []},
      {"id": "nabhasa_count.vallaki", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [1, 2, 3, 4, 6, 8, 12], "relations": [], "conditions": []},
      {"id": "dosha.kuja", "grahas": ["Mars"], "houses": [3], "relations": [], "conditions": ["from_venus"]}
    ],
    "absent": ["mahapurusha.ruchaka", "lunar.gajakesari", "lunar.amala", "solar.budha_aditya", "nabhasa_count.dama"],
    "cancelled": [],
    "evaluated": 41,
    "skipped_time_unknown": 0
  }
}
````

**Yeni dosya:** `tests/golden/modules/yogas/vasumati-and-amala-from-both-references-no-kuja-dosha.json`

````json
{
  "id": "vasumati-and-amala-from-both-references-no-kuja-dosha",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Lagna 25° Koç. Burçlar ve lagna burcundan sayılan evler: Güneş 312° Kova (11. ev); Ay 3° Koç (1. ev); Mars 166° Başak (6. ev); Merkür 328° Kova (11. ev); Jüpiter 67° İkizler (3. ev); Venüs 290° Oğlak (10. ev); Satürn 219° Akrep (8. ev); Rahu 212° Akrep (8. ev); Ketu 32° Boğa (2. ev).",
    "Ev lordları (Koç lagna): 1 Mars, 2 Venüs, 3 Merkür, 4 Ay, 5 Güneş, 6 Merkür, 7 Venüs, 8 Mars, 9 Jüpiter, 10 Satürn, 11 Satürn, 12 Jüpiter.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 51°: büyüyor, iyicil. Merkür'ün burcundaki diğer grahalar: Güneş; Merkür kötücül.",
    "Mahapurusha (lagnadan kendra 1, 4, 7, 10). Ruchaka: Mars Başak (kendi ya da yücelme burcu değil), 6. ev (kendra değil) → yok. Bhadra: Merkür Kova (kendi ya da yücelme burcu değil), 11. ev (kendra değil) → yok. Hamsa: Jüpiter İkizler (kendi ya da yücelme burcu değil), 3. ev (kendra değil) → yok. Malavya: Venüs Oğlak (kendi ya da yücelme burcu değil), 10. ev (kendra) → yok. Shasha: Satürn Akrep (kendi ya da yücelme burcu değil), 8. ev (kendra değil) → yok.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Boğa: boş; 12. burç Balık: boş → üçü de yok.",
    "Kemadruma: Ay'ın burcunda sayılan graha: boş; Ay'dan 4., 7. ve 10. evlerde: Venüs; lagnadan kendralarda: Venüs → temel koşul sağlanıyor ama kendradaki graha iptal eder → Kemadruma yok.",
    "Gajakesari: Jüpiter Ay'dan 3. evde (kendra değil) → yok. Chandra-Mangala: Ay Koç, Mars Başak → yok.",
    "Adhi: Ay'dan evler: Merkür 11, Jüpiter 3, Venüs 10 → üçü birden 6, 7 ya da 8 değil: yok.",
    "Shakata: Ay Jüpiter'den 11. evde (6, 8 ya da 12 değil); Ay lagnadan 1. evde (kendra) → yok.",
    "Amala: lagnadan 10. burç Oğlak: Venüs iyicil (sağlar); Ay'dan 10. burç Oğlak: Venüs iyicil (sağlar) → oluşur.",
    "Vasumati (upachaya 3, 6, 10, 11): lagnadan Merkür 11, Jüpiter 3, Venüs 10 (üçü de upachaya); Ay'dan Merkür 11, Jüpiter 3, Venüs 10 (üçü de upachaya) → oluşur.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Balık: boş; 12. burç Oğlak: Venüs → Vasi oluşur; diğer ikisi yok. Budha-Aditya: Güneş Kova, Merkür Kova → birlikte, oluşur.",
    "İlişkili çiftler (yedi graha; birlikte = aynı burç, karşılıklı bakış = iki yönlü tam bakış, değişim = birbirinin burcunda): Güneş–Merkür (birlikte).",
    "Kendra–trikona: kendra lordları 1 Mars, 4 Ay, 7 Venüs, 10 Satürn; trikona lordları 1 Mars, 5 Güneş, 9 Jüpiter. İlişkili çiftlerden bir kendra lordu ile bir trikona lordu olanlar: yok.",
    "Dharma-Karmadhipati: 9. lord Jüpiter, 10. lord Satürn: ilişkili değil → yok. Yogakaraka (lagna dışında bir kendranın ve bir trikonanın lordu): Koç lagnada yok.",
    "Dhana: 1 Mars, 2 Venüs, 5 Güneş, 9 Jüpiter, 11 Satürn. İlişkili çiftlerden iki lordu da bu kümeden olan ve 2. ya da 11. lordu içerenler: yok.",
    "Değişim (sınıfı değişen iki ev belirler: 6, 8 ya da 12 varsa Dainya; yoksa 3 varsa 3. ev lordu değişimi; yoksa Maha): değişen çift yok.",
    "Viparita raja (lord 6., 8. ya da 12. evde): 6. lord Merkür 11. evde → Harsha yok; 8. lord Mars 6. evde → Sarala oluşur; 12. lord Jüpiter 3. evde → Vimala yok.",
    "Neecha bhanga: düşüş burcunda graha yok → yok.",
    "Nabhasa: yedi grahanın burçları Koç, İkizler, Başak, Akrep, Oğlak, Kova: 6 farklı burç → Dama. Burç türleri: yürük, sabit, çift doğalı → tek tür değil: ashraya yogası yok.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars lagnadan 6., Ay'dan 6., Venüs'ten 9. evde → üç referanstan da yok.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): lunar.gajakesari, lunar.adhi, solar.vesi, raja.kendra_trikona, raja.dharma_karmadhipati, raja.yogakaraka, dhana.dhana, viparita.harsha, neecha_bhanga.neecha_bhanga, dosha.kuja."
  ],
  "input": {
    "lagna_lon": 25.0,
    "grahas": {"Sun": 312.0, "Moon": 3.0, "Mars": 166.0, "Mercury": 328.0, "Jupiter": 67.0, "Venus": 290.0, "Saturn": 219.0, "Rahu": 212.0, "Ketu": 32.0}
  },
  "expected": {
    "hits": [
      {"id": "lunar.amala", "grahas": ["Venus"], "houses": [10], "relations": [], "conditions": ["from_lagna", "from_moon"]},
      {"id": "lunar.vasumati", "grahas": ["Mercury", "Jupiter", "Venus"], "houses": [3, 10, 11], "relations": [], "conditions": ["from_lagna", "from_moon"]},
      {"id": "solar.vasi", "grahas": ["Sun", "Venus"], "houses": [10, 11], "relations": [], "conditions": []},
      {"id": "solar.budha_aditya", "grahas": ["Sun", "Mercury"], "houses": [11], "relations": ["Conjunction"], "conditions": []},
      {"id": "viparita.sarala", "grahas": ["Mars"], "houses": [6, 8], "relations": [], "conditions": []},
      {"id": "nabhasa_count.dama", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [1, 3, 6, 8, 10, 11], "relations": [], "conditions": []}
    ],
    "absent": ["lunar.gajakesari", "lunar.adhi", "solar.vesi", "raja.kendra_trikona", "raja.dharma_karmadhipati", "raja.yogakaraka", "dhana.dhana", "viparita.harsha", "neecha_bhanga.neecha_bhanga", "dosha.kuja"],
    "cancelled": [],
    "evaluated": 41,
    "skipped_time_unknown": 0
  }
}
````

**Yeni dosya:** `tests/golden/modules/yogas/neecha-bhanga-by-dispositor-aspect-only.json`

````json
{
  "id": "neecha-bhanga-by-dispositor-aspect-only",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Lagna 45° Boğa. Burçlar ve lagna burcundan sayılan evler: Güneş 177° Başak (5. ev); Ay 96° Yengeç (3. ev); Mars 161° Başak (5. ev); Merkür 153° Başak (5. ev); Jüpiter 229° Akrep (7. ev); Venüs 188° Terazi (6. ev); Satürn 16° Koç (12. ev); Rahu 343° Balık (11. ev); Ketu 163° Başak (5. ev).",
    "Ev lordları (Boğa lagna): 1 Venüs, 2 Merkür, 3 Ay, 4 Güneş, 5 Merkür, 6 Venüs, 7 Mars, 8 Jüpiter, 9 Satürn, 10 Satürn, 11 Jüpiter, 12 Mars.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 279°: küçülüyor, kötücül. Merkür'ün burcundaki diğer grahalar: Güneş, Mars, Ketu; Merkür kötücül.",
    "Mahapurusha (lagnadan kendra 1, 4, 7, 10). Ruchaka: Mars Başak (kendi ya da yücelme burcu değil), 5. ev (kendra değil) → yok. Bhadra: Merkür Başak (kendi burcu ve yücelme burcu), 5. ev (kendra değil) → yok. Hamsa: Jüpiter Akrep (kendi ya da yücelme burcu değil), 7. ev (kendra) → yok. Malavya: Venüs Terazi (kendi burcu), 6. ev (kendra değil) → yok. Shasha: Satürn Koç (kendi ya da yücelme burcu değil), 12. ev (kendra değil) → yok.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Aslan: boş; 12. burç İkizler: boş → üçü de yok.",
    "Kemadruma: Ay'ın burcunda sayılan graha: boş; Ay'dan 4., 7. ve 10. evlerde: Venüs, Satürn; lagnadan kendralarda: Jüpiter → temel koşul sağlanıyor ama kendradaki graha iptal eder → Kemadruma yok.",
    "Gajakesari: Jüpiter Ay'dan 5. evde (kendra değil) → yok. Chandra-Mangala: Ay Yengeç, Mars Başak → yok.",
    "Adhi: Ay'dan evler: Merkür 3, Jüpiter 5, Venüs 4 → üçü birden 6, 7 ya da 8 değil: yok.",
    "Shakata: Ay Jüpiter'den 9. evde (6, 8 ya da 12 değil); Ay lagnadan 3. evde (kendra değil) → yok.",
    "Amala: lagnadan 10. burç Kova: boş; Ay'dan 10. burç Koç: Satürn kötücül (kötücül var, sağlamaz) → yok.",
    "Vasumati (upachaya 3, 6, 10, 11): lagnadan Merkür 5, Jüpiter 7, Venüs 6 (üçü birden upachaya değil); Ay'dan Merkür 3, Jüpiter 5, Venüs 4 (üçü birden upachaya değil) → yok.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Terazi: Venüs; 12. burç Aslan: boş → Vesi oluşur; diğer ikisi yok. Budha-Aditya: Güneş Başak, Merkür Başak → birlikte, oluşur.",
    "İlişkili çiftler (yedi graha; birlikte = aynı burç, karşılıklı bakış = iki yönlü tam bakış, değişim = birbirinin burcunda): Güneş–Mars (birlikte); Güneş–Merkür (birlikte); Mars–Merkür (birlikte); Venüs–Satürn (karşılıklı bakış).",
    "Kendra–trikona: kendra lordları 1 Venüs, 4 Güneş, 7 Mars, 10 Satürn; trikona lordları 1 Venüs, 5 Merkür, 9 Satürn. İlişkili çiftlerden bir kendra lordu ile bir trikona lordu olanlar: Güneş–Merkür (evler 4, 5); Mars–Merkür (evler 5, 7); Venüs–Satürn (evler 1, 9, 10).",
    "Dharma-Karmadhipati: 9. ve 10. lord aynı graha (Satürn) → yok. Yogakaraka (lagna dışında bir kendranın ve bir trikonanın lordu): Satürn (9 ve 10. evlerin lordu).",
    "Dhana: 1 Venüs, 2 Merkür, 5 Merkür, 9 Satürn, 11 Jüpiter. İlişkili çiftlerden iki lordu da bu kümeden olan ve 2. ya da 11. lordu içerenler: yok.",
    "Değişim (sınıfı değişen iki ev belirler: 6, 8 ya da 12 varsa Dainya; yoksa 3 varsa 3. ev lordu değişimi; yoksa Maha): değişen çift yok.",
    "Viparita raja (lord 6., 8. ya da 12. evde): 6. lord Venüs 6. evde → Harsha oluşur; 8. lord Jüpiter 7. evde → Sarala yok; 12. lord Mars 5. evde → Vimala yok.",
    "Neecha bhanga: Satürn düşüş burcunda (Koç). (1) Burcun lordu Mars: lagnadan 5., Ay'dan 3. → kendrada değil. (2) O burçta yücelen Güneş: lagnadan 5., Ay'dan 3. → kendrada değil. (3) Dispozitör Mars (Başak): Koç oradan 8. burç, tam bakış var → oluşur: dispositor_aspect.",
    "Nabhasa: yedi grahanın burçları Koç, Yengeç, Başak, Terazi, Akrep: 5 farklı burç → Pasha. Burç türleri: yürük, sabit, çift doğalı → tek tür değil: ashraya yogası yok.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars lagnadan 5., Ay'dan 3., Venüs'ten 12. evde → oluşur: from_venus.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): lunar.amala, lunar.vasumati, raja.dharma_karmadhipati, dhana.dhana, exchange.dainya, viparita.sarala."
  ],
  "input": {
    "lagna_lon": 45.0,
    "grahas": {"Sun": 177.0, "Moon": 96.0, "Mars": 161.0, "Mercury": 153.0, "Jupiter": 229.0, "Venus": 188.0, "Saturn": 16.0, "Rahu": 343.0, "Ketu": 163.0}
  },
  "expected": {
    "hits": [
      {"id": "solar.vesi", "grahas": ["Sun", "Venus"], "houses": [5, 6], "relations": [], "conditions": []},
      {"id": "solar.budha_aditya", "grahas": ["Sun", "Mercury"], "houses": [5], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Sun", "Mercury"], "houses": [4, 5], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Mars", "Mercury"], "houses": [5, 7], "relations": ["Conjunction"], "conditions": []},
      {"id": "raja.kendra_trikona", "grahas": ["Venus", "Saturn"], "houses": [1, 9, 10], "relations": ["MutualAspect"], "conditions": []},
      {"id": "raja.yogakaraka", "grahas": ["Saturn"], "houses": [9, 10], "relations": [], "conditions": []},
      {"id": "viparita.harsha", "grahas": ["Venus"], "houses": [6], "relations": [], "conditions": []},
      {"id": "neecha_bhanga.neecha_bhanga", "grahas": ["Saturn"], "houses": [12], "relations": [], "conditions": ["dispositor_aspect"]},
      {"id": "nabhasa_count.pasha", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [3, 5, 6, 7, 12], "relations": [], "conditions": []},
      {"id": "dosha.kuja", "grahas": ["Mars"], "houses": [5], "relations": [], "conditions": ["from_venus"]}
    ],
    "absent": ["lunar.amala", "lunar.vasumati", "raja.dharma_karmadhipati", "dhana.dhana", "exchange.dainya", "viparita.sarala"],
    "cancelled": [],
    "evaluated": 41,
    "skipped_time_unknown": 0
  }
}
````

**Yeni dosya:** `tests/golden/modules/yogas/time-unknown-gola-and-rajju-in-capricorn.json`

````json
{
  "id": "time-unknown-gola-and-rajju-in-capricorn",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Doğum saati bilinmiyor: lagna yok, evler sayılmaz. Burçlar: Güneş 282° Oğlak; Ay 295° Oğlak; Mars 273° Oğlak; Merkür 288° Oğlak; Jüpiter 277° Oğlak; Venüs 299° Oğlak; Satürn 291° Oğlak; Rahu 10° Koç; Ketu 190° Terazi.",
    "Lagna isteyen 17 kural değerlendirilmez (beş mahapurusha, Kemadruma, Shakata, üç raja, dhana, üç değişim, üç viparita); diğer 24 kural değerlendirilir ve sonuçları yaklaşıktır.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 13°: büyüyor, iyicil. Merkür'ün burcundaki diğer grahalar: Güneş, Ay, Mars, Jüpiter, Venüs, Satürn; Merkür iyicil.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Kova: boş; 12. burç Yay: boş → üçü de yok.",
    "Gajakesari: Jüpiter Ay'dan 1. evde (kendra) → oluşur. Chandra-Mangala: Ay Oğlak, Mars Oğlak → birlikte, oluşur.",
    "Adhi: Ay'dan evler: Merkür 1, Jüpiter 1, Venüs 1 → üçü birden 6, 7 ya da 8 değil: yok.",
    "Amala: Ay'dan 10. burç Terazi: Ketu kötücül (kötücül var, sağlamaz) → yok.",
    "Vasumati (upachaya 3, 6, 10, 11): Ay'dan Merkür 1, Jüpiter 1, Venüs 1 (üçü birden upachaya değil) → yok.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Kova: boş; 12. burç Yay: boş → üçü de yok. Budha-Aditya: Güneş Oğlak, Merkür Oğlak → birlikte, oluşur.",
    "Neecha bhanga: Jüpiter düşüş burcunda (Oğlak). (1) Burcun lordu Satürn: Ay'dan 1. → kendrada. (2) O burçta yücelen Mars: Ay'dan 1. → kendrada. (3) Dispozitör Satürn (Oğlak): Oğlak oradan 1. burç, tam bakış yok → oluşur: sign_lord_in_kendra, exalted_graha_in_kendra.",
    "Nabhasa: yedi grahanın burçları Oğlak: 1 farklı burç → Gola. Burç türleri: yürük → hepsi yürük: Rajju oluşur.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars Ay'dan 1., Venüs'ten 1. evde → oluşur: from_moon, from_venus.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): lunar.sunapha, lunar.adhi, lunar.amala, lunar.vasumati, solar.vesi, nabhasa_count.yuga, nabhasa_ashraya.musala, nabhasa_ashraya.nala."
  ],
  "input": {
    "lagna_lon": null,
    "grahas": {"Sun": 282.0, "Moon": 295.0, "Mars": 273.0, "Mercury": 288.0, "Jupiter": 277.0, "Venus": 299.0, "Saturn": 291.0, "Rahu": 10.0, "Ketu": 190.0}
  },
  "expected": {
    "hits": [
      {"id": "lunar.gajakesari", "grahas": ["Moon", "Jupiter"], "houses": [], "relations": [], "conditions": []},
      {"id": "lunar.chandra_mangala", "grahas": ["Moon", "Mars"], "houses": [], "relations": ["Conjunction"], "conditions": []},
      {"id": "solar.budha_aditya", "grahas": ["Sun", "Mercury"], "houses": [], "relations": ["Conjunction"], "conditions": []},
      {"id": "neecha_bhanga.neecha_bhanga", "grahas": ["Jupiter"], "houses": [], "relations": [], "conditions": ["sign_lord_in_kendra", "exalted_graha_in_kendra"]},
      {"id": "nabhasa_count.gola", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [], "relations": [], "conditions": []},
      {"id": "nabhasa_ashraya.rajju", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [], "relations": [], "conditions": []},
      {"id": "dosha.kuja", "grahas": ["Mars"], "houses": [], "relations": [], "conditions": ["from_moon", "from_venus"]}
    ],
    "absent": ["lunar.sunapha", "lunar.adhi", "lunar.amala", "lunar.vasumati", "solar.vesi", "nabhasa_count.yuga", "nabhasa_ashraya.musala", "nabhasa_ashraya.nala"],
    "cancelled": [],
    "evaluated": 24,
    "skipped_time_unknown": 17
  }
}
````

**Yeni dosya:** `tests/golden/modules/yogas/time-unknown-moon-references-only.json`

````json
{
  "id": "time-unknown-moon-references-only",
  "source": "hand",
  "rule_ref": "BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; ileri modüller tasarım belgesi §8 (alıntı yapılmaz)",
  "derivation": [
    "Doğum saati bilinmiyor: lagna yok, evler sayılmaz. Burçlar: Güneş 312° Kova; Ay 3° Koç; Mars 166° Başak; Merkür 328° Kova; Jüpiter 67° İkizler; Venüs 290° Oğlak; Satürn 219° Akrep; Rahu 212° Akrep; Ketu 32° Boğa.",
    "Lagna isteyen 17 kural değerlendirilmez (beş mahapurusha, Kemadruma, Shakata, üç raja, dhana, üç değişim, üç viparita); diğer 24 kural değerlendirilir ve sonuçları yaklaşıktır.",
    "Doğal sınıf (K6): Ay'ın Güneş'ten uzanımı 51°: büyüyor, iyicil. Merkür'ün burcundaki diğer grahalar: Güneş; Merkür kötücül.",
    "Ay'ın yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Boğa: boş; 12. burç Balık: boş → üçü de yok.",
    "Gajakesari: Jüpiter Ay'dan 3. evde (kendra değil) → yok. Chandra-Mangala: Ay Koç, Mars Başak → yok.",
    "Adhi: Ay'dan evler: Merkür 11, Jüpiter 3, Venüs 10 → üçü birden 6, 7 ya da 8 değil: yok.",
    "Amala: Ay'dan 10. burç Oğlak: Venüs iyicil (sağlar) → oluşur.",
    "Vasumati (upachaya 3, 6, 10, 11): Ay'dan Merkür 11, Jüpiter 3, Venüs 10 (üçü de upachaya) → oluşur.",
    "Güneş'in yanları (sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn): 2. burç Balık: boş; 12. burç Oğlak: Venüs → Vasi oluşur; diğer ikisi yok. Budha-Aditya: Güneş Kova, Merkür Kova → birlikte, oluşur.",
    "Neecha bhanga: düşüş burcunda graha yok → yok.",
    "Nabhasa: yedi grahanın burçları Koç, İkizler, Başak, Akrep, Oğlak, Kova: 6 farklı burç → Dama. Burç türleri: yürük, sabit, çift doğalı → tek tür değil: ashraya yogası yok.",
    "Kuja dosha (1, 2, 4, 7, 8, 12): Mars Ay'dan 6., Venüs'ten 9. evde → üç referanstan da yok.",
    "Oluşmayan örnek olarak işaretlenenler (`absent`): lunar.durudhara, lunar.gajakesari, lunar.chandra_mangala, neecha_bhanga.neecha_bhanga, nabhasa_count.vallaki, nabhasa_ashraya.rajju, dosha.kuja."
  ],
  "input": {
    "lagna_lon": null,
    "grahas": {"Sun": 312.0, "Moon": 3.0, "Mars": 166.0, "Mercury": 328.0, "Jupiter": 67.0, "Venus": 290.0, "Saturn": 219.0, "Rahu": 212.0, "Ketu": 32.0}
  },
  "expected": {
    "hits": [
      {"id": "lunar.amala", "grahas": ["Venus"], "houses": [], "relations": [], "conditions": ["from_moon"]},
      {"id": "lunar.vasumati", "grahas": ["Mercury", "Jupiter", "Venus"], "houses": [], "relations": [], "conditions": ["from_moon"]},
      {"id": "solar.vasi", "grahas": ["Sun", "Venus"], "houses": [], "relations": [], "conditions": []},
      {"id": "solar.budha_aditya", "grahas": ["Sun", "Mercury"], "houses": [], "relations": ["Conjunction"], "conditions": []},
      {"id": "nabhasa_count.dama", "grahas": ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"], "houses": [], "relations": [], "conditions": []}
    ],
    "absent": ["lunar.durudhara", "lunar.gajakesari", "lunar.chandra_mangala", "neecha_bhanga.neecha_bhanga", "nabhasa_count.vallaki", "nabhasa_ashraya.rajju", "dosha.kuja"],
    "cancelled": [],
    "evaluated": 24,
    "skipped_time_unknown": 17
  }
}
````

````diff
diff --git a/tests/golden/modules/README.md b/tests/golden/modules/README.md
index 4b0b377..3779fb6 100644
--- a/tests/golden/modules/README.md
+++ b/tests/golden/modules/README.md
@@ -58,4 +58,15 @@ kalemin aritmetiğini adım adım yazar, JSON dosyaları aynı girdiyi ve sonuç
 0,01 virupadır. Kâğıdı, motoru yazan dışında bir gözden geçiren kuraldan yeniden hesaplar ve
 kâğıdın başındaki tabloya adını yazar.
 
+| `yogas` | `hits` (raporun bütün kayıtları, katalog sırasıyla: `id`, `grahas`, `houses`, `relations`, `conditions`), `absent` (bu örneğin "oluşmayan durum" olarak gösterdiği kurallar), `cancelled` (iptal örnekleri), `evaluated`, `skipped_time_unknown` | 15 |
+
+Yoga örnekleri (tasarım belgesi §8 "Test") raporun tamamını yazar: her örnekte katalogdaki 41
+kuralın hepsi `derivation`'da sırayla değerlendirilir. `absent`, o örneğin "oluşmayan durum"
+olarak gösterdiği kuralları sayar; `cancelled`, temel koşulu sağlandığı hâlde iptal kuralıyla
+oluşmayan Kemadruma ve Shakata'yı, bir de düşüş burcunda olup hiçbir koşulu sağlanmayan grahayı
+(Neecha bhanga kaydı yok) gösterir. Ayrı bir test her kuralın en az bir örnekte oluştuğunu, en
+az bir örnekte `absent` sayıldığını ve üç iptal durumunun bulunduğunu sabitler. `lagna_lon: null`
+örneklerinde lagna isteyen 17 kural değerlendirilmez ve `houses` boştur. Tasarım belgesinin açık
+bıraktığı yerlerdeki okuma kuralları ve bağımsız gözden geçirmenin kaydı `yogas/REVIEW.md`'dedir.
+
 Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
````

- [ ] **Step 2: Başarısız testleri yaz (fikstür düzeneği, değişmezler, rapor ve toplayıcı)**

`yogas_hand_fixtures` raporun bütün kayıtlarını örnekle karşılaştırır; `yogas_fixtures_cover_every_rule` kapsamı sabitler (bir örnek silinir ya da zayıflatılırsa düşer).

````diff
diff --git a/crates/jyotish-core/tests/modules_golden.rs b/crates/jyotish-core/tests/modules_golden.rs
index 487851b..2c8e92b 100644
--- a/crates/jyotish-core/tests/modules_golden.rs
+++ b/crates/jyotish-core/tests/modules_golden.rs
@@ -10,9 +10,10 @@ use jyotish_core::{
     argala_rows, arudha_padas, chara_dasha, chara_dasha_order, chara_dasha_sign_years,
     chara_karakas, co_lord_choice, drishti_table, house_lord, houses_owned, jaimini_lord,
     jaimini_report, karakamsha, kuja_dosha, moon_is_waxing, nature, normalize, shadbala,
-    sign_dasha_children, AdvancedSettings, ArgalaStatus, Graha, GrahaWar, KarakaRole, KujaDosha,
-    Nature, Positions, ShadbalaAstro, Sign, SignDashaPeriod, TimeLords, ASPECTING_GRAHAS,
-    CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD, SHADBALA_GRAHAS, SHADBALA_METHOD,
+    sign_dasha_children, yoga_rule, yogas, AdvancedSettings, ArgalaStatus, Graha, GrahaWar,
+    KarakaRole, KujaDosha, Nature, Positions, ShadbalaAstro, Sign, SignDashaPeriod, TimeLords,
+    YogaRelation, ASPECTING_GRAHAS, CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD,
+    SHADBALA_GRAHAS, SHADBALA_METHOD, YOGA_CATALOG, YOGA_CATALOG_VERSION, YOGA_CONDITIONS,
 };
 use serde::de::DeserializeOwned;
 use serde::Deserialize;
@@ -23,6 +24,7 @@ const MIN_DRISHTI: usize = 4;
 const MIN_JAIMINI: usize = 8;
 const MIN_CHARA_DASHA: usize = 7;
 const MIN_SHADBALA: usize = 4;
+const MIN_YOGAS: usize = 15;
 /// El hesabı ile motor arasındaki en büyük fark, virupa.
 const VIRUPA_TOL: f64 = 1e-9;
 /// Shadbala çalışma kâğıdının toleransı, virupa (ileri modüller spec'i §7 "Test").
@@ -1031,3 +1033,167 @@ fn shadbala_fixtures_cover_the_required_cases() {
     };
     assert!(ratio(true) && ratio(false), "ratios on both sides of one");
 }
+
+/// Bir yoga kaydı: `YogaHit`'in `group` ve `approximate` dışındaki alanları (grup katalogdan,
+/// yaklaşık işareti lagnanın yokluğundan gelir; test ikisini de ayrıca denetler).
+#[derive(Debug, Deserialize, PartialEq)]
+#[serde(deny_unknown_fields)]
+struct ExpectedYoga {
+    id: String,
+    grahas: Vec<Graha>,
+    houses: Vec<u8>,
+    relations: Vec<YogaRelation>,
+    conditions: Vec<String>,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct YogasExpected {
+    /// Raporun bütün kayıtları, katalog sırasıyla.
+    hits: Vec<ExpectedYoga>,
+    /// Bu örneğin "oluşmayan durum" olarak gösterdiği kurallar (gerekçesi `derivation`'da).
+    absent: Vec<String>,
+    /// İptal örnekleri: temel koşulu sağlanan ama iptal kuralıyla oluşmayan Kemadruma ve
+    /// Shakata; hiçbir koşulu sağlanmayan düşüşteki graha (Neecha bhanga kaydı yok).
+    cancelled: Vec<String>,
+    evaluated: u32,
+    skipped_time_unknown: u32,
+}
+
+/// İptal kuralı olan yogalar (ileri modüller spec'i §8 "Kararlar").
+const CANCEL_RULES: [&str; 3] = [
+    "lunar.kemadruma",
+    "lunar.shakata",
+    "neecha_bhanga.neecha_bhanga",
+];
+
+#[test]
+fn yogas_hand_fixtures() {
+    for (f, p) in load::<YogasExpected>("yogas", MIN_YOGAS) {
+        let id = &f.id;
+        let x = &f.expected;
+        let report = yogas(&p);
+        assert_eq!(report.catalog_version, YOGA_CATALOG_VERSION, "{id}");
+        assert_eq!(report.evaluated, x.evaluated, "{id}: evaluated");
+        assert_eq!(
+            report.skipped_time_unknown, x.skipped_time_unknown,
+            "{id}: skipped"
+        );
+        let got: Vec<ExpectedYoga> = report
+            .hits
+            .iter()
+            .map(|h| ExpectedYoga {
+                id: h.id.clone(),
+                grahas: h.grahas.clone(),
+                houses: h.houses.clone(),
+                relations: h.relations.clone(),
+                conditions: h.conditions.clone(),
+            })
+            .collect();
+        assert_eq!(got.len(), x.hits.len(), "{id}: hit count, got {got:#?}");
+        for (g, want) in got.iter().zip(&x.hits) {
+            assert_eq!(g, want, "{id}");
+        }
+        for h in &report.hits {
+            let rule = yoga_rule(&h.id).unwrap_or_else(|| panic!("{id}: {} unknown", h.id));
+            assert_eq!(h.group, rule.group, "{id}: {}", h.id);
+            // Doğum saati bilinmiyorsa her kayıt yaklaşıktır ve ev taşımaz.
+            assert_eq!(h.approximate, p.lagna_sign.is_none(), "{id}: {}", h.id);
+            assert!(p.lagna_sign.is_some() || h.houses.is_empty(), "{id}");
+        }
+        // "Oluşmayan" ve "iptal" örnekleri katalogdadır, değerlendirilmiştir ve raporda yoktur.
+        for absent in x.absent.iter().chain(&x.cancelled) {
+            let rule = yoga_rule(absent).unwrap_or_else(|| panic!("{id}: {absent} unknown"));
+            assert!(
+                p.lagna_sign.is_some() || !rule.needs_lagna,
+                "{id}: {absent} is skipped, not absent, without a lagna"
+            );
+            assert!(
+                report.hits.iter().all(|h| &h.id != absent),
+                "{id}: {absent} is marked absent but the report lists it"
+            );
+        }
+        for cancelled in &x.cancelled {
+            assert!(
+                CANCEL_RULES.contains(&cancelled.as_str()),
+                "{id}: {cancelled} has no cancellation rule"
+            );
+        }
+    }
+}
+
+/// Spec §8 "Test": her yoga için en az bir oluşan ve bir oluşmayan örnek; iptal kuralı olanlarda
+/// ayrıca iptal edilen durum. Örnek silinirse ya da zayıflatılırsa bu test düşer.
+#[test]
+fn yogas_fixtures_cover_every_rule() {
+    let all = load::<YogasExpected>("yogas", MIN_YOGAS);
+    let formed = |rule: &str| {
+        all.iter()
+            .any(|(f, _)| f.expected.hits.iter().any(|h| h.id == rule))
+    };
+    let absent = |rule: &str| {
+        all.iter()
+            .any(|(f, _)| f.expected.absent.iter().any(|a| a == rule))
+    };
+    let cancelled = |rule: &str| {
+        all.iter()
+            .any(|(f, _)| f.expected.cancelled.iter().any(|a| a == rule))
+    };
+    assert_eq!(YOGA_CATALOG.len(), 41);
+    for rule in YOGA_CATALOG {
+        assert!(formed(rule.id), "{}: no fixture where it forms", rule.id);
+        assert!(
+            absent(rule.id),
+            "{}: no fixture where it is absent",
+            rule.id
+        );
+    }
+    for rule in CANCEL_RULES {
+        assert!(cancelled(rule), "{rule}: no cancelled case");
+    }
+    // Her alt koşul ve her ilişki türü en az bir örnekte geçer.
+    for condition in YOGA_CONDITIONS {
+        assert!(
+            all.iter().any(|(f, _)| f
+                .expected
+                .hits
+                .iter()
+                .any(|h| h.conditions.iter().any(|c| c == condition))),
+            "condition {condition}"
+        );
+    }
+    for relation in [
+        YogaRelation::Conjunction,
+        YogaRelation::MutualAspect,
+        YogaRelation::Exchange,
+    ] {
+        assert!(
+            all.iter().any(|(f, _)| f
+                .expected
+                .hits
+                .iter()
+                .any(|h| h.relations.contains(&relation))),
+            "relation {relation:?}"
+        );
+    }
+    // Bir kayıtta birden çok ilişki; iki koşulu birden taşıyan mahapurusha (Merkür Başak'ta).
+    assert!(all
+        .iter()
+        .any(|(f, _)| f.expected.hits.iter().any(|h| h.relations.len() > 1)));
+    assert!(all.iter().any(|(f, _)| f
+        .expected
+        .hits
+        .iter()
+        .any(|h| h.id == "mahapurusha.bhadra" && h.conditions.len() == 2)));
+    // Doğum saati bilinmeyen örnekler: 17 kural atlanır, 24 kural değerlendirilir.
+    let unknown: Vec<_> = all.iter().filter(|(_, p)| p.lagna_sign.is_none()).collect();
+    assert!(unknown.len() >= 2, "time-unknown fixtures");
+    for (f, _) in unknown {
+        assert_eq!(
+            (f.expected.evaluated, f.expected.skipped_time_unknown),
+            (24, 17),
+            "{}",
+            f.id
+        );
+    }
+}
````

````diff
diff --git a/crates/jyotish-core/src/proptests.rs b/crates/jyotish-core/src/proptests.rs
index f4ea42d..ff006b8 100644
--- a/crates/jyotish-core/src/proptests.rs
+++ b/crates/jyotish-core/src/proptests.rs
@@ -1135,3 +1135,188 @@ mod shadbala_properties {
         }
     }
 }
+
+mod yoga_properties {
+    use proptest::prelude::*;
+
+    use super::foundation_properties::positions;
+    use crate::graha::Graha;
+    use crate::positions::Positions;
+    use crate::yogas::{
+        is_exchange, yoga_relations, yoga_rule, yogas, YogaGroup, YogaHit, YogaRelation,
+        KENDRA_HOUSES, YOGA_CATALOG, YOGA_CATALOG_VERSION, YOGA_CONDITIONS, YOGA_GRAHAS,
+    };
+
+    fn ids(hits: &[YogaHit], prefix: &str) -> Vec<String> {
+        hits.iter()
+            .filter(|h| h.id.starts_with(prefix))
+            .map(|h| h.id.clone())
+            .collect()
+    }
+
+    fn has(hits: &[YogaHit], id: &str) -> bool {
+        hits.iter().any(|h| h.id == id)
+    }
+
+    /// Kimliğin katalogdaki sırası.
+    fn catalog_index(id: &str) -> usize {
+        YOGA_CATALOG
+            .iter()
+            .position(|r| r.id == id)
+            .unwrap_or_else(|| panic!("{id} is not in the catalog"))
+    }
+
+    proptest! {
+        #[test]
+        fn exactly_one_count_yoga_and_at_most_one_ashraya_yoga(p in positions()) {
+            let report = yogas(&p);
+            prop_assert_eq!(ids(&report.hits, "nabhasa_count.").len(), 1);
+            prop_assert!(ids(&report.hits, "nabhasa_ashraya.").len() <= 1);
+            // Tek burçta toplanan yedi graha (Gola) her zaman bir ashraya yogası da verir.
+            if has(&report.hits, "nabhasa_count.gola") {
+                prop_assert_eq!(ids(&report.hits, "nabhasa_ashraya.").len(), 1);
+            }
+        }
+
+        #[test]
+        fn mahapurusha_needs_a_kendra_and_a_strong_sign(p in positions()) {
+            for h in yogas(&p).hits.iter().filter(|h| h.group == YogaGroup::Mahapurusha) {
+                prop_assert_eq!(h.grahas.len(), 1);
+                let house = p.house_of(h.grahas[0]);
+                prop_assert!(house.is_some_and(|x| KENDRA_HOUSES.contains(&x)));
+                prop_assert_eq!(&h.houses, &vec![house.unwrap()]);
+                prop_assert!(!h.conditions.is_empty());
+            }
+        }
+
+        #[test]
+        fn flank_yogas_exclude_each_other_and_kemadruma(p in positions()) {
+            let report = yogas(&p);
+            let moon: Vec<String> = ["lunar.sunapha", "lunar.anapha", "lunar.durudhara"]
+                .into_iter()
+                .filter(|id| has(&report.hits, id))
+                .map(str::to_owned)
+                .collect();
+            prop_assert!(moon.len() <= 1, "{:?}", moon);
+            let sun = ["solar.vesi", "solar.vasi", "solar.ubhayachari"]
+                .into_iter()
+                .filter(|id| has(&report.hits, id))
+                .count();
+            prop_assert!(sun <= 1);
+            if has(&report.hits, "lunar.kemadruma") {
+                prop_assert!(moon.is_empty(), "{:?}", moon);
+                // Jüpiter Ay'dan kendradaysa Kemadruma oluşamaz.
+                prop_assert!(!has(&report.hits, "lunar.gajakesari"));
+            }
+        }
+
+        #[test]
+        fn hits_are_unique_ordered_and_well_formed(p in positions()) {
+            let report = yogas(&p);
+            prop_assert_eq!(report.catalog_version, YOGA_CATALOG_VERSION);
+            prop_assert_eq!(
+                (report.evaluated + report.skipped_time_unknown) as usize,
+                YOGA_CATALOG.len()
+            );
+            let lagna = p.lagna_sign.is_some();
+            prop_assert_eq!(report.skipped_time_unknown, if lagna { 0 } else { 17 });
+            let mut seen: Vec<(&str, &[Graha])> = Vec::new();
+            let mut last = 0;
+            for h in &report.hits {
+                // Her kimlik katalogdadır; kayıtlar katalog sırasındadır.
+                let rule = yoga_rule(&h.id);
+                prop_assert!(rule.is_some(), "{} is not in the catalog", h.id);
+                let rule = rule.unwrap();
+                prop_assert_eq!(h.group, rule.group);
+                let index = catalog_index(&h.id);
+                prop_assert!(index >= last);
+                last = index;
+                // Aynı kimlik ve aynı graha kümesi iki kez dönmez.
+                let key = (h.id.as_str(), h.grahas.as_slice());
+                prop_assert!(!seen.contains(&key), "{:?} twice", key);
+                seen.push(key);
+                // Grahalar ve evler sıralı ve yinelemesizdir; düğümler hiçbir kayıtta yoktur.
+                prop_assert!(!h.grahas.is_empty());
+                prop_assert!(h.grahas.windows(2).all(|w| w[0] < w[1]));
+                prop_assert!(h.grahas.iter().all(|g| !g.is_node()));
+                prop_assert!(h.houses.windows(2).all(|w| w[0] < w[1]));
+                prop_assert!(h.houses.iter().all(|x| (1..=12).contains(x)));
+                prop_assert!(h.conditions.iter().all(|c| YOGA_CONDITIONS.contains(&c.as_str())));
+                // Lagnasız konumda lagna isteyen kural dönmez; her kayıt yaklaşıktır, ev taşımaz.
+                prop_assert_eq!(h.approximate, !lagna);
+                if !lagna {
+                    prop_assert!(!rule.needs_lagna);
+                    prop_assert!(h.houses.is_empty());
+                    prop_assert!(!h.conditions.iter().any(|c| c == "from_lagna"));
+                }
+            }
+        }
+
+        #[test]
+        fn relation_based_hits_carry_the_relations_of_their_pair(p in positions()) {
+            let report = yogas(&p);
+            for h in &report.hits {
+                match h.group {
+                    YogaGroup::Raja if h.id != "raja.yogakaraka" => {
+                        prop_assert_eq!(h.grahas.len(), 2);
+                        let relations = yoga_relations(h.grahas[0], h.grahas[1], &p);
+                        prop_assert!(!relations.is_empty());
+                        prop_assert_eq!(&h.relations, &relations);
+                    }
+                    YogaGroup::Dhana => {
+                        prop_assert_eq!(&h.relations, &yoga_relations(h.grahas[0], h.grahas[1], &p));
+                        prop_assert!(h.houses.iter().any(|x| [2, 11].contains(x)));
+                    }
+                    YogaGroup::Exchange => {
+                        prop_assert!(is_exchange(h.grahas[0], h.grahas[1], &p));
+                        prop_assert_eq!(&h.relations, &vec![YogaRelation::Exchange]);
+                        prop_assert_eq!(h.houses.len(), 2);
+                    }
+                    _ => {}
+                }
+            }
+            // Dharma-Karmadhipati kaydı varsa aynı çift kendra–trikona kaydı olarak da döner.
+            for h in report.hits.iter().filter(|h| h.id == "raja.dharma_karmadhipati") {
+                prop_assert!(report
+                    .hits
+                    .iter()
+                    .any(|k| k.id == "raja.kendra_trikona" && k.grahas == h.grahas));
+            }
+            // Değişen her çift tam bir değişim sınıfında döner.
+            if p.lagna_sign.is_some() {
+                let exchanging = YOGA_GRAHAS
+                    .into_iter()
+                    .enumerate()
+                    .flat_map(|(i, a)| YOGA_GRAHAS[i + 1..].iter().map(move |&b| (a, b)))
+                    .filter(|&(a, b)| is_exchange(a, b, &p))
+                    .count();
+                let listed = report
+                    .hits
+                    .iter()
+                    .filter(|h| h.group == YogaGroup::Exchange)
+                    .count();
+                prop_assert_eq!(listed, exchanging);
+            }
+        }
+
+        #[test]
+        fn dropping_the_lagna_keeps_the_lagna_free_rules(p in positions()) {
+            // Lagnasız rapor, lagnalı rapordaki lagnadan bağımsız kayıtları aynı grahalarla taşır.
+            let without = yogas(&p.clone().without_lagna());
+            let with: Positions = p;
+            for h in &yogas(&with).hits {
+                let free = [YogaGroup::Solar, YogaGroup::NabhasaCount, YogaGroup::NabhasaAshraya]
+                    .contains(&h.group)
+                    || ["lunar.sunapha", "lunar.anapha", "lunar.durudhara", "lunar.gajakesari",
+                        "lunar.chandra_mangala", "lunar.adhi"].contains(&h.id.as_str());
+                if free {
+                    prop_assert!(
+                        without.hits.iter().any(|w| w.id == h.id && w.grahas == h.grahas),
+                        "{} is missing without a lagna",
+                        h.id
+                    );
+                }
+            }
+        }
+    }
+}
````

````diff
diff --git a/crates/jyotish-core/src/yogas/mod.rs b/crates/jyotish-core/src/yogas/mod.rs
index c0e082a..e3ff912 100644
--- a/crates/jyotish-core/src/yogas/mod.rs
+++ b/crates/jyotish-core/src/yogas/mod.rs
@@ -519,4 +519,134 @@ mod tests {
         assert!(in_houses_from(&p, Sun, Sign::Taurus, &KENDRA_HOUSES));
         assert!(!in_houses_from(&p, Sun, Sign::Taurus, &TRIKONA_HOUSES));
     }
+
+    /// Koç lagna; Güneş Aslan (5), Ay Kova (11), Mars Oğlak (10), Merkür Başak (6), Jüpiter
+    /// Yengeç (4), Venüs İkizler (3), Satürn Terazi (7), Rahu Boğa, Ketu Akrep. El örneği
+    /// `three-exalted-mahapurushas-seven-signs` ile aynı burçlar.
+    const SEVEN_SIGNS: [usize; 9] = [4, 10, 9, 5, 3, 2, 6, 1, 7];
+
+    #[test]
+    fn report_lists_hits_in_catalog_order_and_counts_the_rules() {
+        let report = yogas(&chart(Some(0), SEVEN_SIGNS));
+        assert_eq!(report.catalog_version, YOGA_CATALOG_VERSION);
+        assert_eq!((report.evaluated, report.skipped_time_unknown), (41, 0));
+        let ids: Vec<&str> = report.hits.iter().map(|h| h.id.as_str()).collect();
+        assert_eq!(
+            ids,
+            [
+                "mahapurusha.ruchaka",
+                "mahapurusha.hamsa",
+                "mahapurusha.shasha",
+                "lunar.anapha",
+                "lunar.shakata",
+                "solar.ubhayachari",
+                "raja.kendra_trikona",
+                "raja.kendra_trikona",
+                "viparita.harsha",
+                "nabhasa_count.vallaki",
+                "dosha.kuja",
+            ]
+        );
+        let ruchaka = &report.hits[0];
+        assert_eq!(ruchaka.group, YogaGroup::Mahapurusha);
+        assert_eq!(ruchaka.grahas, [Mars]);
+        assert_eq!(ruchaka.houses, [10]);
+        assert_eq!(ruchaka.conditions, [COND_EXALTATION]);
+        assert!(ruchaka.relations.is_empty());
+        assert!(report.hits.iter().all(|h| !h.approximate));
+        // Bir kuralın iki eşleşmesi graha sırasıyla: Güneş–Ay, sonra Mars–Jüpiter.
+        assert_eq!(report.hits[6].grahas, [Sun, Moon]);
+        assert_eq!(report.hits[7].grahas, [Mars, Jupiter]);
+        assert_eq!(report.hits[7].relations, [YogaRelation::MutualAspect]);
+    }
+
+    #[test]
+    fn without_a_lagna_seventeen_rules_are_skipped_and_the_rest_is_approximate() {
+        let report = yogas(&chart(None, SEVEN_SIGNS));
+        assert_eq!((report.evaluated, report.skipped_time_unknown), (24, 17));
+        let ids: Vec<&str> = report.hits.iter().map(|h| h.id.as_str()).collect();
+        assert_eq!(
+            ids,
+            [
+                "lunar.anapha",
+                "solar.ubhayachari",
+                "nabhasa_count.vallaki",
+                "dosha.kuja"
+            ]
+        );
+        for h in &report.hits {
+            assert!(h.approximate, "{}", h.id);
+            assert!(h.houses.is_empty(), "{}", h.id);
+        }
+        // Kuja dosha lagnasız konumda yalnızca Ay ve Venüs referanslarını taşır.
+        assert_eq!(report.hits[3].conditions, [COND_FROM_MOON, COND_FROM_VENUS]);
+        // Lagna isteyen kural lagnasız konumda eşleşme vermez.
+        for r in YOGA_CATALOG.iter().filter(|r| r.needs_lagna) {
+            assert!(r.matches(&chart(None, SEVEN_SIGNS)).is_empty(), "{}", r.id);
+        }
+    }
+
+    #[test]
+    fn collector_follows_time_known_the_rasi_frame_and_the_chart_errors() {
+        use crate::chart::compute_chart;
+        use crate::fake::FakeEph;
+        use crate::settings::{CalcSettings, ZodiacMode};
+
+        let input = |settings: CalcSettings, time_known: bool| ChartInput {
+            jd_ut: 2_451_545.0,
+            lat: 41.0,
+            lon: 29.0,
+            settings,
+            time_known,
+        };
+        let mut eph = FakeEph::default();
+        let s = CalcSettings::default();
+        let chart = compute_chart(&mut eph, 2_451_545.0, 41.0, 29.0, &s).unwrap();
+        let known = compute_yogas(&mut eph, &input(s.clone(), true)).unwrap();
+        assert_eq!(known, yogas(&Positions::from(&chart)));
+        assert_eq!((known.evaluated, known.skipped_time_unknown), (41, 0));
+        let unknown = compute_yogas(&mut eph, &input(s.clone(), false)).unwrap();
+        assert_eq!(unknown, yogas(&Positions::from(&chart).without_lagna()));
+        assert_eq!((unknown.evaluated, unknown.skipped_time_unknown), (24, 17));
+        assert!(unknown.hits.iter().all(|h| h.approximate));
+        // FakeEph: Güneş ve Merkür tropikal 10° ve 20°, ayanamsa 24° → ikisi de sidereal Balık.
+        let budha_aditya = |r: &YogaReport| r.hits.iter().any(|h| h.id == "solar.budha_aditya");
+        assert!(budha_aditya(&known) && budha_aditya(&unknown));
+
+        // Rasi çerçevesi ayarı izler (K5). Sidereal: Jüpiter 226° Akrep, lagna 46° Boğa: 7. ev
+        // ama kendi burcu değil → Hamsa yok. Tropikal rasi: Jüpiter 250° Yay (kendi burcu),
+        // lagna 70° İkizler: 7. ev → Hamsa.
+        let hamsa = |r: &YogaReport| r.hits.iter().find(|h| h.id == "mahapurusha.hamsa").cloned();
+        assert_eq!(hamsa(&known), None);
+        let tropical = CalcSettings {
+            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
+            ..CalcSettings::default()
+        };
+        let report = compute_yogas(&mut eph, &input(tropical, true)).unwrap();
+        let hit = hamsa(&report).expect("Hamsa in the tropical rasi frame");
+        assert_eq!(hit.houses, [7]);
+        assert_eq!(hit.conditions, [COND_OWN_SIGN]);
+
+        // Hatalar temel haritanınkidir; saat bilinmese de denetlenir.
+        let mut bad = input(s, false);
+        bad.lat = 91.0;
+        assert_eq!(
+            compute_yogas(&mut eph, &bad),
+            Err(CoreError::InvalidLocation {
+                lat: 91.0,
+                lon: 29.0
+            })
+        );
+        let old = CalcSettings {
+            schema_version: 1,
+            ..CalcSettings::default()
+        };
+        assert_eq!(
+            compute_yogas(&mut eph, &input(old, true)),
+            Err(CoreError::UnsupportedSchemaVersion {
+                found: 1,
+                supported: 2
+            })
+        );
+    }
 }
````

- [ ] **Step 3: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-core 2>&1 | grep -E "^error" | sort | uniq -c
```
Expected: `could not compile `jyotish-core` (lib test)` ve `(test "modules_golden")`; hatalar `yogas`, `compute_yogas`, `YogaHit`, `YogaReport`, `ChartInput` ve `CoreError` adlarının bulunamamasıdır (`E0425`, `E0432`, `E0422`, `E0433`).

- [ ] **Step 4: Rapor ve toplayıcı**

````diff
diff --git a/crates/jyotish-core/src/yogas/mod.rs b/crates/jyotish-core/src/yogas/mod.rs
index e3ff912..245f382 100644
--- a/crates/jyotish-core/src/yogas/mod.rs
+++ b/crates/jyotish-core/src/yogas/mod.rs
@@ -14,13 +14,15 @@ mod placement;
 pub use lords::{exchange_kind, ExchangeKind, DHANA_HOUSES};
 pub use placement::occupied_sign_count;
 
+use ephemeris::Ephemeris;
 use serde::{Deserialize, Serialize};
 
 use crate::dignity::own_signs;
 use crate::drishti::{graha_aspect_signs, ASPECTING_GRAHAS};
 use crate::graha::Graha;
-use crate::positions::Positions;
+use crate::positions::{compute_positions, ChartInput, Positions};
 use crate::zodiac::Sign;
+use crate::CoreError;
 
 /// Kural kataloğunun sürümü: bir kural eklenince, çıkınca ya da koşulu değişince artar.
 pub const YOGA_CATALOG_VERSION: u32 = 1;
@@ -311,6 +313,78 @@ pub static YOGA_CATALOG: &[YogaRule] = &[
     rule("dosha.kuja", YogaGroup::Dosha, false, placement::kuja),
 ];
 
+/// Oluşan bir yoga: bir kuralın bir eşleşmesi.
+#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
+#[cfg_attr(feature = "specta", derive(specta::Type))]
+pub struct YogaHit {
+    /// Kararlı kimlik (`"mahapurusha.hamsa"`); görünen ad i18n anahtarıdır.
+    pub id: String,
+    pub group: YogaGroup,
+    /// Yogayı oluşturan grahalar, `Graha::ALL` sırasında.
+    pub grahas: Vec<Graha>,
+    /// Lagnadan sayılan ilgili evler, artan sırada; doğum saati bilinmiyorsa boş.
+    pub houses: Vec<u8>,
+    /// İki grahanın ilişkisine dayanan kurallarda ilişki türleri; diğerlerinde boş.
+    pub relations: Vec<YogaRelation>,
+    /// Sağlanan alt koşul kimlikleri (`YOGA_CONDITIONS`); alt koşulu olmayan kurallarda boş.
+    pub conditions: Vec<String>,
+    /// Doğum saati bilinmiyor: konumlar yaklaşıktır (Ay'ın burcu gün içinde değişebilir) ve
+    /// lagnaya bağlı seçenekler değerlendirilmemiştir.
+    pub approximate: bool,
+}
+
+/// `compute_yogas` sonucu.
+#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
+#[cfg_attr(feature = "specta", derive(specta::Type))]
+pub struct YogaReport {
+    pub catalog_version: u32,
+    /// Katalog sırasıyla; bir kuralın birden çok eşleşmesi graha sırasıyla.
+    pub hits: Vec<YogaHit>,
+    /// Değerlendirilen kural sayısı.
+    pub evaluated: u32,
+    /// Doğum saati bilinmediği için değerlendirilmeyen (lagna isteyen) kural sayısı.
+    pub skipped_time_unknown: u32,
+}
+
+/// Saf hesap: konumlardan yoga raporu. Lagna yoksa (doğum saati bilinmiyor) lagna isteyen
+/// kurallar atlanır ve sayılır; diğer kuralların eşleşmeleri `approximate` işaretiyle döner.
+pub fn yogas(positions: &Positions) -> YogaReport {
+    let has_lagna = positions.lagna_sign.is_some();
+    let mut hits = Vec::new();
+    let (mut evaluated, mut skipped) = (0, 0);
+    for rule in YOGA_CATALOG {
+        if rule.needs_lagna && !has_lagna {
+            skipped += 1;
+            continue;
+        }
+        evaluated += 1;
+        hits.extend(rule.matches(positions).into_iter().map(|m| YogaHit {
+            id: rule.id.to_owned(),
+            group: rule.group,
+            grahas: m.grahas,
+            houses: m.houses,
+            relations: m.relations,
+            conditions: m.conditions.into_iter().map(str::to_owned).collect(),
+            approximate: !has_lagna,
+        }));
+    }
+    YogaReport {
+        catalog_version: YOGA_CATALOG_VERSION,
+        hits,
+        evaluated,
+        skipped_time_unknown: skipped,
+    }
+}
+
+/// `compute_yogas` komutunun hesabı: temel harita, konumlar, yoga raporu. Ayar, an ve konum
+/// denetimi `compute_chart`'ınkidir; `input.time_known` yanlışsa lagna isteyen kurallar atlanır.
+pub fn compute_yogas<E: Ephemeris + ?Sized>(
+    eph: &mut E,
+    input: &ChartInput,
+) -> Result<YogaReport, CoreError> {
+    Ok(yogas(&compute_positions(eph, input)?))
+}
+
 /// Katalogdaki kural; kimlik bilinmiyorsa `None`.
 pub fn yoga_rule(id: &str) -> Option<&'static YogaRule> {
     YOGA_CATALOG.iter().find(|r| r.id == id)
````

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 3718726..1cf3187 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -92,9 +92,10 @@ pub use varga::{
     all_vargas, is_vargottama, varga_chart, varga_position, Varga, VargaChart, VargaPlacement,
 };
 pub use yogas::{
-    exchange_kind, is_exchange, is_mutual_aspect, occupied_sign_count, yoga_relations, yoga_rule,
-    ExchangeKind, YogaGroup, YogaMatch, YogaRelation, YogaRule, COUNTED_GRAHAS, DHANA_HOUSES,
-    DUSTHANA_HOUSES, KENDRA_HOUSES, TRIKONA_HOUSES, UPACHAYA_HOUSES, YOGA_CATALOG,
-    YOGA_CATALOG_VERSION, YOGA_CONDITIONS, YOGA_GRAHAS,
+    compute_yogas, exchange_kind, is_exchange, is_mutual_aspect, occupied_sign_count,
+    yoga_relations, yoga_rule, yogas, ExchangeKind, YogaGroup, YogaHit, YogaMatch, YogaRelation,
+    YogaReport, YogaRule, COUNTED_GRAHAS, DHANA_HOUSES, DUSTHANA_HOUSES, KENDRA_HOUSES,
+    TRIKONA_HOUSES, UPACHAYA_HOUSES, YOGA_CATALOG, YOGA_CATALOG_VERSION, YOGA_CONDITIONS,
+    YOGA_GRAHAS,
 };
 pub use zodiac::{angular_distance, deg_in_sign, nakshatra_pada, normalize, Sign};
````

- [ ] **Step 5: Testleri koş**

```bash
cargo test -p jyotish-core --lib yogas:: 2>&1 | grep -E "^test result"
cargo test -p jyotish-core --lib yoga_properties 2>&1 | grep -E "^test result"
cargo test -p jyotish-core --test modules_golden 2>&1 | grep -E "^test result"
```
Expected: `31 passed`; `6 passed`; `11 passed`. `yogas_hand_fixtures` düşerse önce örneğin `derivation`'ı ve kural yeniden okunur; düzeltme hangi tarafta olursa olsun gerekçesiyle ilerleme notlarına yazılır ve beklenen değer motor çıktısından kopyalanmaz.

- [ ] **Step 6: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
grep -rn '"source"' tests/golden/modules/yogas | grep -vc '"source": "hand"'
ls tests/golden/modules/yogas/*.json | wc -l
```
Expected: fmt ve clippy çıktısız; `jyotish-core` lib `276 passed`, golden `8 passed`, `modules_golden` `11 passed`; diğer satırlar tabandaki gibi; `grep -vc` `0`, `wc -l` `15` yazar.

- [ ] **Step 7: Commit**

```bash
git add crates/jyotish-core tests/golden/modules
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): yoga report and compute_yogas collector; fifteen hand fixtures covering every rule, properties" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: iki `grep` çıktısız.

---


### Task 4: Uygulama: `compute_yogas` komutu; bağlamalar, sarmalayıcı ve fikstür; v1 ile çapraz denetim

Spec §8 "Veri modeli ve IPC", "Test" (uçtan uca), K1, K2, K4, K8; §13 "Komut katmanı" ve katman 4; §15 bütçesi; D17–D19. Komut Plan 14'ün ortak kalıbını izler: motor iş parçacığı, `blocking` havuzu, `CommandError`.

**Files:**
- Modify: `app/src-tauri/src/module_commands.rs`, `app/src-tauri/src/lib.rs`, `app/src-tauri/tests/modules.rs`, `app/src-tauri/tests/ui_fixture.rs`, `app/ui/api.ts`, `app/ui/api.test.ts`, `app/ui/test/fixture.ts`, `app/e2e/ipc.ts`
- Regenerate: `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`
- Test: `app/src-tauri/tests/modules.rs` (`yogas_checks`), `app/ui/api.test.ts`

**Interfaces:**
- Consumes: `jyotish_core::{compute_yogas, yogas, yoga_rule, kuja_dosha, YogaReport, YogaGroup, Dignity, HouseSystem, Positions, YOGA_CATALOG, YOGA_CATALOG_VERSION, YOGA_CONDITIONS}`; `ephemeris::{Engine, SwissEph}`; `client_commands::blocking`; `commands::{CommandError, compute_chart_blocking}`; `tests/modules.rs`'teki `golden_inputs`, `median_of_15`.
- Produces:
  - `pub fn compute_yogas_blocking(engine: &Engine, input: ChartInput) -> Result<YogaReport, CommandError>`
  - Tauri komutu `compute_yogas(input: ChartInput) -> YogaReport`; TS `commands.computeYogas`, `api.computeYogas(input: ChartInput): Promise<Result<YogaReport>>`
  - TS tipleri (üretilir): `YogaReport`, `YogaHit`, `YogaGroup`, `YogaRelation`.
  - `UiFixture.yogas: YogaReport`, `UiFixture.yogas_time_unknown: YogaReport`, `UiFixture.yoga_catalog: { id: string; group: YogaGroup }[]`, `UiFixture.yoga_conditions: string[]`; e2e taklidinde `compute_yogas`.

- [ ] **Step 1: Başarısız testleri yaz**

`yogas_checks` beş golden anında komutu çağırır ve v1 haritasıyla çapraz denetler (rapor aynı konumlardan aynı çıkar; mahapurusha ve Neecha bhanga kayıtları v1'in dignity alanıyla, Budha-Aditya burç eşitliğiyle, Kuja dosha 3.1'in olgusuyla tutarlıdır); ev sistemi ayarının raporu değiştirmediğini, doğum saati bilinmeyen girdide 17 kuralın atlandığını, hata eşlemesini ve bütçeyi denetler.

````diff
diff --git a/app/src-tauri/tests/modules.rs b/app/src-tauri/tests/modules.rs
index a1b0fed..85acaf6 100644
--- a/app/src-tauri/tests/modules.rs
+++ b/app/src-tauri/tests/modules.rs
@@ -7,17 +7,19 @@ use std::time::{Duration, Instant};
 use ephemeris::{Body, Engine, Ephemeris, SwissEph, Zodiac};
 use jyotish_core::{
     chara_dasha, cheshta_from, co_lords, compute_panchang, compute_shadbala_astro,
-    graha_aspect_signs, house_lord, house_sign, jaimini_report, mean_longitudes,
+    graha_aspect_signs, house_lord, house_sign, jaimini_report, kuja_dosha, mean_longitudes,
     rashi_drishti_signs, shadbala, sign_dasha_children, sign_dasha_path_at, sphuta_drishti,
-    sun_entry_before, vara_lord, AdvancedSettings, CalcSettings, ChartInput, ChartWarning, Graha,
-    KarakaRole, KarakaScheme, Positions, Sign, SignDashaPeriod, Varga, ASPECTING_GRAHAS,
-    CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD, MAX_SUNRISE_GAP_DAYS, SEVEN_KARAKA_ROLES,
-    SHADBALA_GRAHAS, SHADBALA_METHOD,
+    sun_entry_before, vara_lord, yoga_rule, yogas, AdvancedSettings, CalcSettings, ChartInput,
+    ChartWarning, Dignity, Graha, HouseSystem, KarakaRole, KarakaScheme, Positions, Sign,
+    SignDashaPeriod, Varga, YogaGroup, ASPECTING_GRAHAS, CHARA_DASHA_METHOD, DRISHTI_METHOD,
+    JAIMINI_METHOD, MAX_SUNRISE_GAP_DAYS, SEVEN_KARAKA_ROLES, SHADBALA_GRAHAS, SHADBALA_METHOD,
+    YOGA_CATALOG, YOGA_CATALOG_VERSION,
 };
 use jyotish_desk_lib::commands::compute_chart_blocking;
 use jyotish_desk_lib::module_commands::{
     chara_dasha_at_blocking, chara_dasha_children_blocking, compute_chara_dasha_blocking,
     compute_drishti_blocking, compute_jaimini_blocking, compute_shadbala_blocking,
+    compute_yogas_blocking,
 };
 use serde_json::Value;
 
@@ -61,6 +63,7 @@ fn module_commands_run_on_the_engine_thread_and_map_errors() {
     jaimini_checks(&engine, &inputs);
     chara_dasha_checks(&engine, &inputs);
     shadbala_checks(&engine, &inputs);
+    yogas_checks(&engine, &inputs);
 }
 
 fn drishti_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
@@ -1084,3 +1087,164 @@ fn shadbala_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
         "compute_shadbala median {median:?}"
     );
 }
+
+fn yogas_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
+    for (id, input) in inputs {
+        let report = compute_yogas_blocking(engine, input.clone())
+            .unwrap_or_else(|e| panic!("{id}: {}", e.code));
+        assert_eq!(report.catalog_version, YOGA_CATALOG_VERSION, "{id}");
+        assert_eq!(
+            (report.evaluated, report.skipped_time_unknown),
+            (41, 0),
+            "{id}"
+        );
+        // Çapraz denetim: v1'de golden fixture'larla doğrulanmış haritanın konumlarından aynı
+        // rapor çıkar.
+        let chart = compute_chart_blocking(
+            engine,
+            input.jd_ut,
+            input.lat,
+            input.lon,
+            input.settings.clone(),
+        )
+        .expect("chart");
+        let positions = Positions::from(&chart);
+        assert_eq!(report, yogas(&positions), "{id}");
+
+        // Değişmezler gerçek haritalarda da sağlanır: her kimlik katalogdadır, aynı kimlik ve
+        // graha kümesi iki kez dönmez, sayı yogalarından tam biri oluşur.
+        let mut seen: Vec<(&str, &[Graha])> = Vec::new();
+        for h in &report.hits {
+            let rule = yoga_rule(&h.id).unwrap_or_else(|| panic!("{id}: {} unknown", h.id));
+            assert_eq!(h.group, rule.group, "{id}: {}", h.id);
+            assert!(!h.approximate, "{id}: {}", h.id);
+            let key = (h.id.as_str(), h.grahas.as_slice());
+            assert!(!seen.contains(&key), "{id}: {key:?} twice");
+            seen.push(key);
+        }
+        let count = |group: YogaGroup| report.hits.iter().filter(|h| h.group == group).count();
+        assert_eq!(count(YogaGroup::NabhasaCount), 1, "{id}");
+
+        // v1 olgularıyla çapraz denetim: mahapurusha veren graha v1'in dignity sırasında
+        // yücelmiş, moolatrikonada ya da kendi burcundadır ve rasi evi kendradır; Neecha bhanga
+        // kaydındaki graha düşüştedir; Budha-Aditya Güneş ile Merkür'ün aynı burcudur; Kuja
+        // dosha 3.1'deki olgudur.
+        let v1 = |g: Graha| &chart.grahas[g as usize];
+        for h in &report.hits {
+            match h.group {
+                YogaGroup::Mahapurusha => {
+                    let g = v1(h.grahas[0]);
+                    assert!(
+                        matches!(
+                            g.dignity,
+                            Some(Dignity::Exalted | Dignity::Moolatrikona | Dignity::OwnSign)
+                        ),
+                        "{id}: {} with dignity {:?}",
+                        h.id,
+                        g.dignity
+                    );
+                    let house = g.sign.house_from(chart.lagna.sign);
+                    assert!([1, 4, 7, 10].contains(&house), "{id}: {}", h.id);
+                }
+                YogaGroup::NeechaBhanga => {
+                    assert_eq!(v1(h.grahas[0]).dignity, Some(Dignity::Debilitated), "{id}");
+                }
+                _ => {}
+            }
+        }
+        let has = |yoga: &str| report.hits.iter().any(|h| h.id == yoga);
+        assert_eq!(
+            has("solar.budha_aditya"),
+            v1(Graha::Sun).sign == v1(Graha::Mercury).sign,
+            "{id}"
+        );
+        let dosha = kuja_dosha(&positions);
+        assert_eq!(
+            has("dosha.kuja"),
+            dosha.from_lagna == Some(true) || dosha.from_moon || dosha.from_venus,
+            "{id}"
+        );
+
+        // Evler rasi evidir (K4): ev sistemi ayarı raporu değiştirmez.
+        for house_system in [HouseSystem::Sripati, HouseSystem::Placidus] {
+            let other = ChartInput {
+                settings: CalcSettings {
+                    house_system,
+                    ..input.settings.clone()
+                },
+                ..input.clone()
+            };
+            // Kutup dairesinin ötesinde Placidus tanımsızdır; harita uyarıyla hesaplanır ve
+            // yogalar yine aynıdır.
+            assert_eq!(
+                compute_yogas_blocking(engine, other).expect("house system"),
+                report,
+                "{id}: {house_system:?}"
+            );
+        }
+
+        // Doğum saati bilinmiyorsa lagna isteyen 17 kural atlanır; diğer kayıtlar yaklaşıktır.
+        let unknown = compute_yogas_blocking(
+            engine,
+            ChartInput {
+                time_known: false,
+                ..input.clone()
+            },
+        )
+        .expect("time unknown");
+        assert_eq!(unknown, yogas(&positions.clone().without_lagna()), "{id}");
+        assert_eq!(
+            (unknown.evaluated, unknown.skipped_time_unknown),
+            (24, 17),
+            "{id}"
+        );
+        assert_eq!(
+            (unknown.evaluated + unknown.skipped_time_unknown) as usize,
+            YOGA_CATALOG.len()
+        );
+        for h in &unknown.hits {
+            assert!(h.approximate && h.houses.is_empty(), "{id}: {}", h.id);
+            assert!(!yoga_rule(&h.id).expect("catalog").needs_lagna, "{id}");
+        }
+    }
+
+    // Hatalar `compute_chart` ile aynı kodlara eşlenir; saat bilinmese de denetlenir.
+    let delhi = inputs
+        .iter()
+        .find(|(id, _)| id == "new-delhi-2000-01-01")
+        .expect("delhi fixture")
+        .1
+        .clone();
+    let bad_place = ChartInput {
+        lat: 91.0,
+        time_known: false,
+        ..delhi.clone()
+    };
+    assert_eq!(
+        compute_yogas_blocking(engine, bad_place).unwrap_err().code,
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
+        compute_yogas_blocking(engine, old_settings)
+            .unwrap_err()
+            .code,
+        "unsupported_schema_version"
+    );
+
+    // Performans: spec §15 "compute_yogas ≤ 10 ms" (temel harita hesabı dahil).
+    let (median, max) = median_of_15(|| {
+        compute_yogas_blocking(engine, delhi.clone()).expect("yogas");
+    });
+    eprintln!("compute_yogas median {median:?}, max {max:?}");
+    assert!(
+        median <= Duration::from_millis(10),
+        "compute_yogas median {median:?}"
+    );
+}
````

````diff
diff --git a/app/src-tauri/tests/ui_fixture.rs b/app/src-tauri/tests/ui_fixture.rs
index 58afd56..4e242a3 100644
--- a/app/src-tauri/tests/ui_fixture.rs
+++ b/app/src-tauri/tests/ui_fixture.rs
@@ -11,13 +11,14 @@ use std::sync::Mutex;
 
 use atlas::Timestamp;
 use ephemeris::Engine;
-use jyotish_core::ChartInput;
+use jyotish_core::{ChartInput, YOGA_CATALOG, YOGA_CONDITIONS};
 use jyotish_desk_lib::atlas_commands::{preview_moment_body, search_places_blocking, AtlasState};
 use jyotish_desk_lib::client_commands::create_client_blocking;
 use jyotish_desk_lib::commands::dasha_children;
 use jyotish_desk_lib::module_commands::{
     chara_dasha_at_blocking, chara_dasha_children_blocking, compute_chara_dasha_blocking,
     compute_drishti_blocking, compute_jaimini_blocking, compute_shadbala_blocking,
+    compute_yogas_blocking,
 };
 use jyotish_desk_lib::transit_commands::{transits_at_blocking, TransitAt};
 use serde_json::{json, Value};
@@ -129,9 +130,25 @@ fn ui_fixture_is_up_to_date() {
     )
     .expect("chara dasha children at the transit moment");
     // Shadbala ve Bhava bala: aynı girdiyle (doğum saati biliniyor, Güneş doğup batıyor).
-    let shadbala = compute_shadbala_blocking(&engine, module_input)
+    let shadbala = compute_shadbala_blocking(&engine, module_input.clone())
         .expect("shadbala")
         .expect("time known and a sunrise");
+    // Yogalar: aynı girdiyle ve doğum saati bilinmeyen girdiyle (lagna isteyen kurallar atlanır).
+    // Katalog (kimlik ve grup) ve alt koşul kimlikleri: arayüz testleri her birinin üç dilde
+    // adı olduğunu bu listeden denetler.
+    let yogas = compute_yogas_blocking(&engine, module_input.clone()).expect("yogas");
+    let yogas_time_unknown = compute_yogas_blocking(
+        &engine,
+        ChartInput {
+            time_known: false,
+            ..module_input
+        },
+    )
+    .expect("yogas without a birth time");
+    let yoga_catalog: Vec<Value> = YOGA_CATALOG
+        .iter()
+        .map(|r| json!({ "id": r.id, "group": r.group }))
+        .collect();
     // Kaydedilen doğum verisi girdinin kanonik biçimidir (Plan 10 kapsam dışı 21).
     assert_eq!(opened.chart_record.birth, delhi());
     let prefs = store
@@ -154,6 +171,10 @@ fn ui_fixture_is_up_to_date() {
         "chara_dasha_at": chara_at,
         "chara_dasha_at_children": chara_at_children,
         "shadbala": shadbala,
+        "yogas": yogas,
+        "yogas_time_unknown": yogas_time_unknown,
+        "yoga_catalog": yoga_catalog,
+        "yoga_conditions": YOGA_CONDITIONS,
     });
     let text = serde_json::to_string(&value)
         .expect("serialize")
````

````diff
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index a3206e3..eca596d 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -34,6 +34,7 @@ vi.mock("./bindings", () => ({
     charaDashaChildren: vi.fn(),
     charaDashaAt: vi.fn(),
     computeShadbala: vi.fn(),
+    computeYogas: vi.fn(),
     searchPlaces: vi.fn(),
     previewMoment: vi.fn(),
     getAppPrefs: vi.fn(),
@@ -119,6 +120,7 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["charaDashaChildren", [{ jd_ut: 1, time_known: true }, ["Aries", "Taurus"], 2]],
     ["charaDashaAt", [{ jd_ut: 1, time_known: true }, 2451545.5]],
     ["computeShadbala", [{ jd_ut: 1, time_known: true }]],
+    ["computeYogas", [{ jd_ut: 1, time_known: false }]],
     ["searchPlaces", ["delhi", 8]],
     ["previewMoment", [{ date: "2000-01-01" }]],
     ["getAppPrefs", []],
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-desk --test modules --test ui_fixture 2>&1 | grep -E "^error" | sort | uniq -c
pnpm --dir app test 2>&1 | grep -E "Tests |TypeError"
```
Expected: ilk komutta iki `error[E0432]: unresolved import `jyotish_desk_lib::module_commands::compute_yogas_blocking`` ve iki `could not compile`; ikincide `TypeError: call is not a function` ve `1 failed | 676 passed (677)`.

- [ ] **Step 3: Komutu yaz ve kaydet**

````diff
diff --git a/app/src-tauri/src/module_commands.rs b/app/src-tauri/src/module_commands.rs
index bed1562..1697125 100644
--- a/app/src-tauri/src/module_commands.rs
+++ b/app/src-tauri/src/module_commands.rs
@@ -8,7 +8,7 @@ use std::sync::Arc;
 use ephemeris::{Engine, SwissEph};
 use jyotish_core::{
     CharaDasha, ChartInput, DrishtiTable, JaiminiReport, ShadbalaReport, Sign, SignDashaPath,
-    SignDashaPeriod,
+    SignDashaPeriod, YogaReport,
 };
 use tauri::State;
 
@@ -158,3 +158,27 @@ pub async fn compute_shadbala(
     let engine = Arc::clone(&engine);
     blocking(move || compute_shadbala_blocking(&engine, input)).await
 }
+
+/// `compute_yogas` komutunun gövdesi; bloklar.
+pub fn compute_yogas_blocking(
+    engine: &Engine,
+    input: ChartInput,
+) -> Result<YogaReport, CommandError> {
+    let report =
+        engine.call(move |se: &mut SwissEph| jyotish_core::compute_yogas(se, &input))??;
+    Ok(report)
+}
+
+/// Yogalar: kural kataloğunun D1'de oluşan kayıtları (yoga, grahalar, evler, ilişki türü,
+/// sağlanan alt koşullar) ve değerlendirilen kural sayısı. Doğum saati bilinmiyorsa
+/// (`time_known == false`) lagna isteyen kurallar atlanır ve sayılır; diğer kayıtlar
+/// `approximate` işaretiyle döner.
+#[tauri::command]
+#[specta::specta]
+pub async fn compute_yogas(
+    engine: State<'_, Arc<Engine>>,
+    input: ChartInput,
+) -> Result<YogaReport, CommandError> {
+    let engine = Arc::clone(&engine);
+    blocking(move || compute_yogas_blocking(&engine, input)).await
+}
````

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index d347d44..ba41b4a 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -35,6 +35,7 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             module_commands::chara_dasha_children,
             module_commands::chara_dasha_at,
             module_commands::compute_shadbala,
+            module_commands::compute_yogas,
             client_commands::open_client,
             client_commands::restore_session,
             client_commands::create_client,
````

- [ ] **Step 4: Sarmalayıcı, fikstür tipi ve e2e taklidi; üretilen dosyaları yeniden üret**

````diff
diff --git a/app/ui/api.ts b/app/ui/api.ts
index 7c4f834..4806b8e 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -89,6 +89,9 @@ export const api = {
   // Shadbala ve Bhava bala: `null` doğum saati bilinmeyen ya da o gün Güneş'in doğmadığı ya da
   // batmadığı haritadır.
   computeShadbala: (input: ChartInput) => wrap(() => commands.computeShadbala(input)),
+  // Yogalar: doğum saati bilinmiyorsa rapor yine döner; lagna isteyen kurallar atlanmış,
+  // diğer kayıtlar `approximate` işaretlidir.
+  computeYogas: (input: ChartInput) => wrap(() => commands.computeYogas(input)),
   searchPlaces: (query: string, limit: number) => wrap(() => commands.searchPlaces(query, limit)),
   previewMoment: (birth: BirthData) => wrap(() => commands.previewMoment(birth)),
   appInfo: () => wrap(() => commands.appInfo()),
````

````diff
diff --git a/app/ui/test/fixture.ts b/app/ui/test/fixture.ts
index 65129e4..f5aecf9 100644
--- a/app/ui/test/fixture.ts
+++ b/app/ui/test/fixture.ts
@@ -1,7 +1,8 @@
 // Gerçek veri fikstürü (app/src-tauri/tests/ui_fixture.rs üretir): Delhi 2000-01-01 17:30
 // için `create_client` dönüşü, ilk 3. seviye dönemin alt dönemleri, "new delhi" atlas araması,
 // an önizlemesi, bir DST boşluğu hatası, aynı sabit saatte transit, bakış tablosu, Jaimini raporu,
-// chara dasha ve Shadbala raporu. Arayüz hesap yapmadığı için testler bunu çizer.
+// chara dasha, Shadbala raporu ve yoga raporu (kataloğuyla). Arayüz hesap yapmadığı için testler
+// bunu çizer.
 import type {
   CharaDasha,
   CommandError,
@@ -18,6 +19,8 @@ import type {
   SignDashaPath,
   SignDashaPeriod,
   TransitView,
+  YogaGroup,
+  YogaReport,
 } from "../bindings";
 import raw from "./fixtures/delhi.json?raw";
 
@@ -45,6 +48,14 @@ export interface UiFixture {
   chara_dasha_at_children: SignDashaPeriod[];
   /** `compute_shadbala`, aynı girdiyle: yedi graha, zaman lordları, on iki bhava. */
   shadbala: ShadbalaReport;
+  /** `compute_yogas`, aynı girdiyle: 41 kural değerlendirilir. */
+  yogas: YogaReport;
+  /** `compute_yogas`, `time_known: false` ile: lagna isteyen 17 kural atlanır, kayıtlar yaklaşıktır. */
+  yogas_time_unknown: YogaReport;
+  /** Rust `YOGA_CATALOG`: kural kimlikleri ve grupları, katalog sırasıyla. */
+  yoga_catalog: { id: string; group: YogaGroup }[];
+  /** Rust `YOGA_CONDITIONS`: alt koşul kimlikleri. */
+  yoga_conditions: string[];
 }
 
 const FIXTURE = JSON.parse(raw) as UiFixture;
````

````diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index d823b43..d713888 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -112,6 +112,8 @@ export async function mockIpc(page: Page, { session, unknownTime = false }: IpcO
         chara_dasha_at: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.chara_dasha_at : null),
         // Shadbala: fikstürdeki gerçek sonuç; saat bilinmiyorsa Rust `null` döner.
         compute_shadbala: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.shadbala : null),
+        // Yogalar: fikstürdeki iki gerçek sonuç (doğum saati bilinen ve bilinmeyen girdi).
+        compute_yogas: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.yogas : fixture.yogas_time_unknown),
         print_window: () => null,
         // Güncelleme: sahte yayın; ağ yoktur. Kurulum yalnızca denetimin bildirdiği sürümü kabul eder.
         app_info: () => ({ version: "0.9.0", updaterConfigured: true }),
````

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
git diff --numstat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
grep -c "computeYogas" app/ui/bindings.ts
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
```
Expected: iki `ok`; `bindings.ts` 52 satır kazanır (bir komut ve dört tip); `delhi.json` tek satır değişir; `grep -c` `1`, son komut `0` yazar. Fikstürdeki rapor (Delhi 2000-01-01, İkizler lagna, varsayılan ayarlar) sonraki görevlerin panel testlerinin beklediği değerleri taşır: 41 kural değerlendirilir, 13 kayıt döner. Sırasıyla: Sunapha (Ay, Venüs; evler 5, 6), Gajakesari (Ay, Jüpiter; 5, 11), Vasi (Güneş, Venüs; 6, 7), Budha-Aditya (Güneş, Merkür; 7; birlikte), kendra–trikona (Jüpiter, Satürn; 7, 9, 10; birlikte), Dharma-Karmadhipati (Jüpiter, Satürn; 9, 10; birlikte), Dhana (Ay, Satürn; 2, 9; karşılıklı bakış), Dhana (Mars, Satürn; 9, 11; değişim), Maha değişimi (Mars, Satürn; 9, 11), Vimala (Venüs; 6, 12), Neecha bhanga (Satürn; 11; `exalted_graha_in_kendra`), Pasha (yedi graha; 5, 6, 7, 9, 11), Kuja dosha (Mars; 9; `from_venus`). Mahapurusha ve ashraya yogası yoktur. Doğum saati bilinmeyen rapor: 24 kural değerlendirilir, 17 atlanır, 6 kayıt döner (Sunapha, Gajakesari, Vasi, Budha-Aditya, Pasha, Kuja dosha), hepsi yaklaşıktır ve ev taşımaz.

- [ ] **Step 5: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test -p jyotish-desk --test modules -- --nocapture 2>&1 | grep -E "median"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: fmt ve clippy çıktısız; `jyotish-desk` `modules` `1 passed`, `bindings` `1 passed`, `ui_fixture` `2 passed`; diğer Rust satırları Task 3'teki gibi; yedi `median` satırı (`compute_drishti`, `compute_jaimini`, `compute_chara_dasha`, `chara_dasha_children`, `chara_dasha_at`, `compute_shadbala`, `compute_yogas`), hepsi bütçesinin altında (ölçüm: 0,56–0,79 ms; `compute_yogas` 0,63–0,64 ms); Vitest `72 passed (72)`, `677 passed (677)` (`api.test.ts`'in tablosu büyür, test sayısı değişmez; "dizgi bağımsız değişken" testi değişmeden geçer: C2′); `✓ built`.

- [ ] **Step 6: Commit**

```bash
git add app/src-tauri/src app/src-tauri/tests app/ui/bindings.ts app/ui/test app/ui/api.ts app/ui/api.test.ts app/e2e/ipc.ts
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(app): compute_yogas command; bindings, wrapper and fixture with the catalog; cross-checks on the engine" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: iki `grep` çıktısız.

---

### Task 5: Arayüz: Yogalar paneli

Spec §8 "Arayüz", "Test" (üç dilde ad), K11, K12; §13 "Arayüz"; UI spec §3.4, §5, §6; D17, D20–D23.

**Files:**
- Create: `app/ui/modules/useYogas.ts`, `app/ui/panels/YogasPanel.tsx`, `app/ui/panels/YogasPanel.test.tsx`
- Modify: `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx`, `app/ui/panels/panels.css`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: `YogasPanel.test.tsx` (12 test), `model.test.ts`, `panels.test.tsx`

**Interfaces:**
- Consumes: `api.computeYogas`, `UiFixture` (Task 4); `modules/useModule.ts`, `modules/input.ts`; `panels/bala.tsx` (`BalaScroll`), `Approx`, `PinButton`, `states.tsx`; `chart/adapter.ts` (`signIndex`); `highlight/store` (`graha`, `house`, `sign` hedefleri); `session/birth.ts` (`timeUnknown`).
- Produces: `useYogas(opened): ModuleState<YogaReport>`; `YogasPanel`, `YOGA_GROUPS`, `YogaGroupFilter`, `yogaGroupFilter(settings)`; panel türü `yogas` (`MODULE_PANELS` `{ w: 7, h: 14 }`); panel ayarı `group`; i18n `panel.yogas`, `yogas.*`.

- [ ] **Step 1: Başarısız testleri yaz**

**Yeni dosya:** `app/ui/panels/YogasPanel.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeYogas: vi.fn() } }));

import { api, type Result } from "../api";
import type { YogaReport } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { YOGA_GROUPS, yogaGroupFilter, YogasPanel } from "./YogasPanel";

const ok = (report: YogaReport = fixture().yogas): Result<YogaReport> => ({ status: "ok", data: report });
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "yg", kind: "yogas", x: 0, y: 0, w: 7, h: 14, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};

/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  return <YogasPanel panel={active.panels[0]} />;
}

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened });
}

const tables = () => screen.getAllByRole("table");
const caption = (table: HTMLElement) => table.querySelector("caption")?.textContent;
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const bodyRows = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);

beforeEach(() => {
  vi.mocked(api.computeYogas).mockReset();
  vi.mocked(api.computeYogas).mockImplementation(async () => ok());
});

test("lists the yogas that form, grouped in catalog order, with grahas, houses, relation and conditions", async () => {
  renderPanel();
  await screen.findByRole("table", { name: "Ay yogaları" });
  expect(api.computeYogas).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.computeYogas).mock.calls[0][0].time_known).toBe(true);
  // Delhi fikstürü, İkizler lagna: dokuz grupta on üç kayıt; Mahapurusha ve ashraya yogası yok.
  expect(tables().map(caption)).toEqual([
    "Ay yogaları",
    "Güneş yogaları",
    "Raja yogaları",
    "Dhana yogaları",
    "Değişim yogaları",
    "Viparita raja yogaları",
    "Neecha bhanga",
    "Nabhasa: sayı yogaları",
    "Dosha",
  ]);
  expect(cells(within(tables()[0]).getAllByRole("row")[0])).toEqual(["Yoga", "Grahalar", "Evler", "İlişki", "Koşullar"]);
  expect(tables().map(bodyRows)).toEqual([
    [
      ["Sunapha", "Ay Ve", "5 6", "—", "—"],
      ["Gajakesari", "Ay Jü", "5 11", "—", "—"],
    ],
    [
      ["Vasi", "Gü Ve", "6 7", "—", "—"],
      ["Budha-Aditya", "Gü Me", "7", "birlikte", "—"],
    ],
    [
      ["Kendra–trikona", "Jü Sa", "7 9 10", "birlikte", "—"],
      ["Dharma-Karmadhipati", "Jü Sa", "9 10", "birlikte", "—"],
    ],
    [
      ["Dhana", "Ay Sa", "2 9", "karşılıklı bakış", "—"],
      ["Dhana", "Ma Sa", "9 11", "değişim", "—"],
    ],
    [["Maha değişimi", "Ma Sa", "9 11", "değişim", "—"]],
    [["Vimala", "Ve", "6 12", "—", "—"]],
    [["Neecha bhanga", "Sa", "11", "—", "o burçta yücelen graha kendrada"]],
    [["Pasha", "Gü Ay Ma Me Jü Ve Sa", "5 6 7 9 11", "—", "—"]],
    [["Kuja dosha", "Ma", "9", "—", "Venüs'ten"]],
  ]);
  // Kısaltmalar tam adla, evler "N. ev" diye okunur.
  const moon = within(tables()[0]).getAllByRole("button", { name: "Ay" })[0];
  expect(moon).toHaveTextContent("Ay");
  expect(within(tables()[8]).getByRole("button", { name: "Mars" })).toHaveTextContent("Ma");
  expect(within(tables()[8]).getByRole("button", { name: "9. ev" })).toHaveTextContent("9");
  expect(screen.getByText("Değerlendirilen kural: 41. Oluşan kayıt: 13.")).toBeInTheDocument();
  expect(screen.getByText(/^Yalnızca D1 değerlendirilir\./)).toBeInTheDocument();
  expect(screen.getByText("Katalog sürümü: 1")).toBeInTheDocument();
  // Doğum saati biliniyor: yaklaşık işareti ve "değerlendirilmedi" açıklaması yok.
  expect(screen.queryByText(/Doğum saati bilinmiyor/)).toBeNull();
  expect(screen.queryByText("≈")).toBeNull();
});

test("the group filter is saved with the layout and narrows the list without a new request", async () => {
  const { persistence } = renderPanel();
  await screen.findByRole("table", { name: "Ay yogaları" });
  const select = screen.getByRole("combobox", { name: "Grup" });
  expect(within(select).getAllByRole("option").map((o) => o.textContent)).toEqual([
    "Tümü",
    "Pancha Mahapurusha",
    "Ay yogaları",
    "Güneş yogaları",
    "Raja yogaları",
    "Dhana yogaları",
    "Değişim yogaları",
    "Viparita raja yogaları",
    "Neecha bhanga",
    "Nabhasa: sayı yogaları",
    "Nabhasa: ashraya yogaları",
    "Dosha",
  ]);
  fireEvent.change(select, { target: { value: "Dhana" } });
  await waitFor(() => expect(tables().map(caption)).toEqual(["Dhana yogaları"]));
  expect(bodyRows(tables()[0])).toHaveLength(2);
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ group: "Dhana" }));
  // Oluşan yogası olmayan grup: tablo yerine açıklama; sayaç satırı raporun tamamını sayar.
  fireEvent.change(select, { target: { value: "Mahapurusha" } });
  expect(await screen.findByText("Bu grupta oluşan yoga yok: Pancha Mahapurusha.")).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.getByText("Değerlendirilen kural: 41. Oluşan kayıt: 13.")).toBeInTheDocument();
  expect(api.computeYogas).toHaveBeenCalledTimes(1);
});

test("the tables sit in one keyboard-focusable scroll group", async () => {
  renderPanel();
  const table = await screen.findByRole("table", { name: "Dosha" });
  const group = screen.getByRole("group", { name: "Oluşan yogalar" });
  expect(group).toHaveClass("bala-scroll");
  expect(group).toHaveAttribute("tabindex", "0");
  expect(group).toContainElement(table);
  expect(within(group).getAllByRole("table")).toHaveLength(9);
});

test("the stored filter is read tolerantly", () => {
  expect(yogaGroupFilter({})).toBe("all");
  expect(yogaGroupFilter({ group: "Raja" })).toBe("Raja");
  expect(yogaGroupFilter({ group: "raja" })).toBe("all");
  expect(yogaGroupFilter({ group: 3 })).toBe("all");
});

test("unknown birth time: lagna rules are skipped in Rust, the rest is marked approximate and carries no houses", async () => {
  vi.mocked(api.computeYogas).mockResolvedValue(ok(fixture().yogas_time_unknown));
  renderPanel({}, unknownTimeOpened());
  await screen.findByRole("table", { name: "Ay yogaları" });
  expect(vi.mocked(api.computeYogas).mock.calls[0][0].time_known).toBe(false);
  expect(tables().map(caption)).toEqual(["Ay yogaları", "Güneş yogaları", "Nabhasa: sayı yogaları", "Dosha"]);
  // "≈" işareti ekran okuyucuya "yaklaşık" diye okunur; evler çizgidir.
  expect(bodyRows(tables()[0])).toEqual([
    ["≈yaklaşık Sunapha", "Ay Ve", "—", "—", "—"],
    ["≈yaklaşık Gajakesari", "Ay Jü", "—", "—", "—"],
  ]);
  expect(bodyRows(tables()[3])).toEqual([["≈yaklaşık Kuja dosha", "Ma", "—", "—", "Venüs'ten"]]);
  expect(screen.getAllByText("≈")).toHaveLength(6);
  expect(
    screen.getByText("Doğum saati bilinmiyor: lagnaya bağlı 17 kural değerlendirilmedi. Listelenen yogalar yaklaşıktır (≈) ve ev taşımaz."),
  ).toBeInTheDocument();
  expect(screen.getByText("Değerlendirilen kural: 24. Oluşan kayıt: 6.")).toBeInTheDocument();
});

test("an empty report says so and gives the number of rules evaluated", async () => {
  vi.mocked(api.computeYogas).mockResolvedValue(ok({ catalog_version: 1, hits: [], evaluated: 41, skipped_time_unknown: 0 }));
  renderPanel();
  expect(await screen.findByText("Katalogdaki yogalardan hiçbiri oluşmuyor. Değerlendirilen kural: 41.")).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.getByRole("combobox", { name: "Grup" })).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeYogas).not.toHaveBeenCalled();
});

test("an error is shown in the panel, the filter stays and retry asks again", async () => {
  vi.mocked(api.computeYogas).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.getByRole("combobox", { name: "Grup" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await screen.findByRole("table", { name: "Ay yogaları" });
  expect(api.computeYogas).toHaveBeenCalledTimes(2);
});

test("retry from the keyboard keeps the focus in the panel while the error box is replaced", async () => {
  vi.mocked(api.computeYogas).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Yogalar">
      <LivePanel />
    </section>,
    { layout: LAYOUT({}), opened: fixture().opened },
  );
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  await screen.findByRole("table", { name: "Ay yogaları" });
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Yogalar" }));
});

test("loading: nothing under a second, then a skeleton, then the list", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<YogaReport>) => void;
    vi.mocked(api.computeYogas).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(container.querySelector(".panel-notice")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getAllByRole("table")).toHaveLength(9);
  } finally {
    vi.useRealTimers();
  }
});

test("shared highlight: a graha marks every yoga it takes part in, a house every yoga that names it", async () => {
  renderPanel();
  await screen.findByRole("table", { name: "Ay yogaları" });
  const marked = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.yoga);
  const venus = within(screen.getByRole("table", { name: "Viparita raja yogaları" })).getByRole("button", { name: "Venüs" });
  fireEvent.click(venus);
  expect(venus).toHaveAttribute("aria-pressed", "true");
  expect(marked()).toEqual(["lunar.sunapha", "solar.vasi", "viparita.vimala", "nabhasa_count.pasha"]);
  // Üzerine gelinen öğe sabitlenenin önüne geçer: 11. ev.
  fireEvent.mouseEnter(within(screen.getByRole("table", { name: "Neecha bhanga" })).getByRole("button", { name: "11. ev" }));
  expect(marked()).toEqual(["lunar.gajakesari", "dhana.dhana", "exchange.maha", "neecha_bhanga.neecha_bhanga", "nabhasa_count.pasha"]);
  fireEvent.keyDown(window, { key: "Escape" });
  fireEvent.mouseLeave(within(screen.getByRole("table", { name: "Neecha bhanga" })).getByRole("button", { name: "11. ev" }));
  expect(marked()).toEqual([]);
});

test("every catalog rule, group, relation and condition has a name in the three languages", () => {
  const { yoga_catalog, yoga_conditions } = fixture();
  expect(yoga_catalog).toHaveLength(41);
  // Arayüzün grup listesi Rust kataloğunun grup sırasıdır.
  expect([...new Set(yoga_catalog.map((r) => r.group))]).toEqual([...YOGA_GROUPS]);
  type Names = Record<string, Record<string, string>>;
  for (const messages of [tr, en, fr]) {
    const names = messages.yogas.name as Names;
    for (const { id } of yoga_catalog) {
      const [group, name] = id.split(".");
      expect(names[group]?.[name], id).toEqual(expect.stringMatching(/\S/));
    }
    // Katalogda olmayan bir kuralın adı da durmaz.
    const named = Object.entries(names).flatMap(([group, rules]) => Object.keys(rules).map((name) => `${group}.${name}`));
    expect(named.sort()).toEqual(yoga_catalog.map((r) => r.id).sort());
    expect(Object.keys(messages.yogas.group).sort()).toEqual([...YOGA_GROUPS].sort());
    expect(Object.keys(messages.yogas.condition).sort()).toEqual([...yoga_conditions].sort());
    expect(Object.keys(messages.yogas.relation).sort()).toEqual(["Conjunction", "Exchange", "MutualAspect"]);
  }
  // Klasik adlar üç dilde aynı yazılır (K11); çevrilenler yalnızca değişim sınıflarıdır.
  const flat = (m: typeof tr) => Object.entries(m.yogas.name as Names).flatMap(([g, rules]) => Object.entries(rules).map(([n, v]) => [`${g}.${n}`, v] as const));
  const differing = flat(tr)
    .filter(([id, v]) => new Map(flat(en)).get(id) !== v || new Map(flat(fr)).get(id) !== v)
    .map(([id]) => id);
  expect(differing).toEqual(["exchange.maha", "exchange.third_lord", "exchange.dainya"]);
});
````

````diff
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index 888282a..11ea65b 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -41,6 +41,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "argala",
     "shadbala",
     "bhava_bala",
+    "yogas",
   ]);
   expect(isKnownPanelKind("chart")).toBe(true);
   expect(isKnownPanelKind("drishti")).toBe(true);
@@ -48,7 +49,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
````

````diff
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index 6a2a807..d7e4e54 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -21,6 +21,7 @@ import { PanchangPanel } from "./PanchangPanel";
 import { panelDefinition, PANEL_REGISTRY } from "./registry";
 import { ShadbalaPanel } from "./ShadbalaPanel";
 import { UnavailablePanel } from "./UnavailablePanel";
+import { YogasPanel } from "./YogasPanel";
 
 const chartPanel = (settings: Record<string, unknown> = { varga: "D1", style: "north" }): Panel => ({
   id: "c",
@@ -50,6 +51,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.argala.component).toBe(ArgalaPanel);
   expect(PANEL_REGISTRY.shadbala.component).toBe(ShadbalaPanel);
   expect(PANEL_REGISTRY.bhava_bala.component).toBe(BhavaBalaPanel);
+  expect(PANEL_REGISTRY.yogas.component).toBe(YogasPanel);
   // Kataloğun her türü uygulanmıştır: yer tutucu yoktur.
   for (const kind of PANEL_KINDS) expect(PANEL_REGISTRY[kind].component, kind).not.toBe(UnavailablePanel);
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests |Failed to resolve"
```
Expected: iki `Failed to resolve import "./YogasPanel"`; `Test Files  3 failed | 70 passed (73)`, `Tests  2 failed | 664 passed (666)`.

- [ ] **Step 3: Kanca**

**Yeni dosya:** `app/ui/modules/useYogas.ts`

````ts
// Yoga raporunu (`compute_yogas`) isteyen kanca. Rapor saklanmaz ve arayüzde hesap yoktur
// (K1, C4); panel açıkken istenir. Doğum saati bilinmiyorsa rapor yine döner: lagna isteyen
// kurallar Rust'ta atlanmış, diğer kayıtlar `approximate` işaretlidir.
import { api } from "../api";
import type { OpenedClient, YogaReport } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

export function useYogas(opened: OpenedClient | null): ModuleState<YogaReport> {
  return useModule(opened !== null ? inputKey(opened) : null, () => api.computeYogas(chartInput(opened as OpenedClient)));
}
````

- [ ] **Step 4: Panel, tür, kayıt, stil ve metinler**

**Yeni dosya:** `app/ui/panels/YogasPanel.tsx`

````tsx
// Yogalar paneli (ileri modüller spec'i §8): kural kataloğunun D1'de oluşan kayıtları, gruplara
// ayrılmış listeyle. Her satır yoganın adını, oluşturan grahaları, lagnadan sayılan evleri,
// ilişki türünü ve sağlanan alt koşulları yazar. Değerler `compute_yogas`'tan gelir; burada hesap
// yoktur (C4). Yoganın ne anlama geldiğini anlatan metin yoktur; adlar ve koşullar etikettir.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { OpenedClient, YogaGroup, YogaHit, YogaReport } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import type { HighlightTarget } from "../highlight/target";
import type { Panel } from "../layout/model";
import { useYogas } from "../modules/useYogas";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import { Approx } from "./Approx";
import { BalaScroll } from "./bala";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Rust `YogaGroup::ALL`: katalog sırası. */
export const YOGA_GROUPS = [
  "Mahapurusha",
  "Lunar",
  "Solar",
  "Raja",
  "Dhana",
  "Exchange",
  "ViparitaRaja",
  "NeechaBhanga",
  "NabhasaCount",
  "NabhasaAshraya",
  "Dosha",
] as const satisfies readonly YogaGroup[];
export type YogaGroupFilter = "all" | YogaGroup;
const DASH = "—";

/** Panelin grup süzgeci; tanınmayan değer ve eksik anahtar "tümü"dür. */
export function yogaGroupFilter(settings: Record<string, unknown>): YogaGroupFilter {
  return (YOGA_GROUPS as readonly unknown[]).includes(settings.group) ? (settings.group as YogaGroup) : "all";
}

/** Ortak vurgu bu kaydın bir grahasını ya da evini gösteriyor mu (burç, lagnadan sayılan evidir). */
function hitIsActive(hit: YogaHit, active: HighlightTarget | null, lagna: number | null): boolean {
  if (active === null) return false;
  if (active.type === "graha") return hit.grahas.some((g) => g === active.id);
  if (active.type === "house") return hit.houses.includes(active.house);
  return lagna !== null && hit.houses.includes(((active.sign - lagna + 12) % 12) + 1);
}

export function YogasPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Yogas panel={panel} opened={opened} />;
}

function Yogas({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const module = useYogas(opened);
  const phase = usePanelLoadPhase(module.loading);
  const id = useId();
  const filter = yogaGroupFilter(panel.settings);
  return (
    <div className="module-panel">
      {/* Süzgeç yükleme ve hata durumlarında da kullanılabilir. */}
      <div className="module-toolbar">
        <label htmlFor={`${id}-group`}>{t("yogas.groupLabel")}</label>
        <select id={`${id}-group`} value={filter} onChange={(e) => setPanelSettings(panel.id, { group: e.target.value })}>
          <option value="all">{t("yogas.allGroups")}</option>
          {YOGA_GROUPS.map((g) => (
            <option key={g} value={g}>
              {t(`yogas.group.${g}`)}
            </option>
          ))}
        </select>
      </div>
      {module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : module.data === null ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : (
        <YogaList report={module.data} filter={filter} opened={opened} />
      )}
    </div>
  );
}

function YogaList({ report, filter, opened }: { report: YogaReport; filter: YogaGroupFilter; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const listId = useId();
  const lagna = timeUnknown(opened) ? null : signIndex(opened.chart.lagna.sign);
  const groups = (filter === "all" ? YOGA_GROUPS : [filter])
    .map((group) => ({ group, hits: report.hits.filter((h) => h.group === group) }))
    .filter((g) => g.hits.length > 0);

  const row = (hit: YogaHit, index: number) => (
    <tr key={`${hit.id}:${index}`} data-yoga={hit.id} className={hitIsActive(hit, active, lagna) ? "is-highlighted" : undefined}>
      <th scope="row">
        <Approx on={hit.approximate} label={t("yogas.approximate")}>
          {t(`yogas.name.${hit.id}`)}
        </Approx>
      </th>
      <td>
        {hit.grahas.map((g, i) => (
          <span key={g}>
            {i > 0 && " "}
            <PinButton target={{ type: "graha", id: g }} label={t(`grahaName.${g}`)}>
              {t(`graha.${g}`)}
            </PinButton>
          </span>
        ))}
      </td>
      <td className="num">
        {hit.houses.length === 0
          ? DASH
          : hit.houses.map((h, i) => (
              <span key={h}>
                {i > 0 && " "}
                <PinButton target={{ type: "house", house: h }} label={t("yogas.houseLabel", { house: h })}>
                  {h}
                </PinButton>
              </span>
            ))}
      </td>
      <td>{hit.relations.length === 0 ? DASH : hit.relations.map((r) => t(`yogas.relation.${r}`)).join(", ")}</td>
      <td>{hit.conditions.length === 0 ? DASH : hit.conditions.map((c) => t(`yogas.condition.${c}`)).join("; ")}</td>
    </tr>
  );

  return (
    <>
      {report.hits.length === 0 ? (
        <p className="panel-message">{t("yogas.empty", { n: report.evaluated })}</p>
      ) : groups.length === 0 ? (
        <p className="panel-message">{t("yogas.emptyGroup", { group: t(`yogas.group.${filter}`) })}</p>
      ) : (
        // İlişki ve koşul sütunlarında odaklanabilir denetim yoktur: dar panelde kaydırma kabı
        // klavye odağı alır ve adını gizli başlıktan okur.
        <BalaScroll labelledBy={listId}>
          <p id={listId} className="visually-hidden">
            {t("yogas.listLabel")}
          </p>
          {groups.map(({ group, hits }) => (
            <table key={group} className="module-table yoga-table" data-group={group}>
              <caption>{t(`yogas.group.${group}`)}</caption>
              <thead>
                <tr>
                  <th scope="col">{t("yogas.yoga")}</th>
                  <th scope="col">{t("yogas.grahas")}</th>
                  <th scope="col" className="num">
                    {t("yogas.houses")}
                  </th>
                  <th scope="col">{t("yogas.relationColumn")}</th>
                  <th scope="col">{t("yogas.conditions")}</th>
                </tr>
              </thead>
              <tbody>{hits.map(row)}</tbody>
            </table>
          ))}
        </BalaScroll>
      )}
      {report.skipped_time_unknown > 0 && <p className="panel-notice">{t("yogas.timeUnknown", { n: report.skipped_time_unknown })}</p>}
      <p className="panel-message">{t("yogas.evaluated", { n: report.evaluated, hits: report.hits.length })}</p>
      <p className="panel-message">{t("yogas.note")}</p>
      <p className="panel-message">{t("yogas.catalog", { version: report.catalog_version })}</p>
    </>
  );
}
````

````diff
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index 96e1b99..ac178c6 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -1,7 +1,7 @@
 // Pano düzeni veri modeli (spec §3.2). Kalıcı biçim `ViewState.panel_layout` ile aynıdır;
 // M4'teki store komutu gelene kadar localStorage'da tutulur (bkz. persistence.ts).
 
-/** Panel kataloğu: v1 (UI spec §3.3) ve ileri modül panelleri (ileri modüller spec'i §4, §5, §7). */
+/** Panel kataloğu: v1 (UI spec §3.3) ve ileri modül panelleri (ileri modüller spec'i §4, §5, §7, §8). */
 export const PANEL_KINDS = [
   "chart",
   "graha_table",
@@ -18,6 +18,7 @@ export const PANEL_KINDS = [
   "argala",
   "shadbala",
   "bhava_bala",
+  "yogas",
 ] as const;
 export type KnownPanelKind = (typeof PANEL_KINDS)[number];
 
@@ -36,6 +37,8 @@ export const MODULE_PANELS = [
   // Shadbala ve Bhava bala (Plan 17).
   { kind: "shadbala", w: 8, h: 12 },
   { kind: "bhava_bala", w: 6, h: 14 },
+  // Yogalar (Plan 18).
+  { kind: "yogas", w: 7, h: 14 },
 ] as const satisfies readonly { kind: KnownPanelKind; w: number; h: number }[];
 export type ModulePanelKind = (typeof MODULE_PANELS)[number]["kind"];
 
````

````diff
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index 006dcd9..2f4beb9 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -17,6 +17,7 @@ import { PanchangPanel } from "./PanchangPanel";
 import { ShadbalaPanel } from "./ShadbalaPanel";
 import type { PanelDefinition } from "./types";
 import { UnavailablePanel } from "./UnavailablePanel";
+import { YogasPanel } from "./YogasPanel";
 
 export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   chart: { titleKey: "panel.chart", subtitle: (p) => chartVarga(p.settings), component: ChartPanel },
@@ -37,6 +38,8 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   // Shadbala ve Bhava bala (Plan 17).
   shadbala: { titleKey: "panel.shadbala", component: ShadbalaPanel },
   bhava_bala: { titleKey: "panel.bhava_bala", component: BhavaBalaPanel },
+  // Yogalar (Plan 18).
+  yogas: { titleKey: "panel.yogas", component: YogasPanel },
 };
 
 const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };
````

````diff
diff --git a/app/ui/panels/panels.css b/app/ui/panels/panels.css
index 4564395..0fdc133 100644
--- a/app/ui/panels/panels.css
+++ b/app/ui/panels/panels.css
@@ -242,3 +242,11 @@
   padding: 0;
   list-style: none;
 }
+.yoga-table + .yoga-table {
+  margin-top: 8px;
+}
+.yoga-table caption {
+  text-align: left;
+  padding: 6px 6px 2px;
+  font-weight: var(--fw-medium);
+}
````

````diff
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 952cd1b..a96b961 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -67,6 +67,7 @@ export const tr = {
     argala: "Argala",
     shadbala: "Shadbala",
     bhava_bala: "Bhava bala",
+    yogas: "Yogalar",
     unknown: "Bilinmeyen panel",
     loading: "Yükleniyor",
     unavailable: "Bu panel sonraki bir sürümde gelecek.",
@@ -215,6 +216,72 @@ export const tr = {
     drishti: "Drishti",
     note: "Bhavalar eşittir: bhava ortası lagnadan 30°'lik adımlarla sayılır ve ev sistemi ayarından bağımsızdır. Bhavadhipati, lordun Shadbala toplamıdır.",
   },
+  yogas: {
+    listLabel: "Oluşan yogalar",
+    groupLabel: "Grup",
+    allGroups: "Tümü",
+    group: {
+      Mahapurusha: "Pancha Mahapurusha",
+      Lunar: "Ay yogaları",
+      Solar: "Güneş yogaları",
+      Raja: "Raja yogaları",
+      Dhana: "Dhana yogaları",
+      Exchange: "Değişim yogaları",
+      ViparitaRaja: "Viparita raja yogaları",
+      NeechaBhanga: "Neecha bhanga",
+      NabhasaCount: "Nabhasa: sayı yogaları",
+      NabhasaAshraya: "Nabhasa: ashraya yogaları",
+      Dosha: "Dosha",
+    },
+    yoga: "Yoga",
+    grahas: "Grahalar",
+    houses: "Evler",
+    houseLabel: "{{house}}. ev",
+    relationColumn: "İlişki",
+    conditions: "Koşullar",
+    relation: { Conjunction: "birlikte", MutualAspect: "karşılıklı bakış", Exchange: "değişim" },
+    condition: {
+      own_sign: "kendi burcunda",
+      exaltation: "yücelme burcunda",
+      from_lagna: "lagnadan",
+      from_moon: "Ay'dan",
+      from_venus: "Venüs'ten",
+      sign_lord_in_kendra: "düşüş burcunun lordu kendrada",
+      exalted_graha_in_kendra: "o burçta yücelen graha kendrada",
+      dispositor_aspect: "dispozitörün bakışı",
+    },
+    name: {
+      mahapurusha: { ruchaka: "Ruchaka", bhadra: "Bhadra", hamsa: "Hamsa", malavya: "Malavya", shasha: "Shasha" },
+      lunar: {
+        sunapha: "Sunapha",
+        anapha: "Anapha",
+        durudhara: "Durudhara",
+        kemadruma: "Kemadruma",
+        gajakesari: "Gajakesari",
+        chandra_mangala: "Chandra-Mangala",
+        adhi: "Adhi",
+        shakata: "Shakata",
+        amala: "Amala",
+        vasumati: "Vasumati",
+      },
+      solar: { vesi: "Vesi", vasi: "Vasi", ubhayachari: "Ubhayachari", budha_aditya: "Budha-Aditya" },
+      raja: { kendra_trikona: "Kendra–trikona", dharma_karmadhipati: "Dharma-Karmadhipati", yogakaraka: "Yogakaraka" },
+      dhana: { dhana: "Dhana" },
+      exchange: { maha: "Maha değişimi", third_lord: "3. ev lordu değişimi", dainya: "Dainya değişimi" },
+      viparita: { harsha: "Harsha", sarala: "Sarala", vimala: "Vimala" },
+      neecha_bhanga: { neecha_bhanga: "Neecha bhanga" },
+      nabhasa_count: { vallaki: "Vallaki", dama: "Dama", pasha: "Pasha", kedara: "Kedara", shula: "Shula", yuga: "Yuga", gola: "Gola" },
+      nabhasa_ashraya: { rajju: "Rajju", musala: "Musala", nala: "Nala" },
+      dosha: { kuja: "Kuja dosha" },
+    },
+    approximate: "yaklaşık",
+    empty: "Katalogdaki yogalardan hiçbiri oluşmuyor. Değerlendirilen kural: {{n}}.",
+    emptyGroup: "Bu grupta oluşan yoga yok: {{group}}.",
+    evaluated: "Değerlendirilen kural: {{n}}. Oluşan kayıt: {{hits}}.",
+    timeUnknown: "Doğum saati bilinmiyor: lagnaya bağlı {{n}} kural değerlendirilmedi. Listelenen yogalar yaklaşıktır (≈) ve ev taşımaz.",
+    note: "Yalnızca D1 değerlendirilir. Evler lagna burcundan sayılan rasi evleridir ve ev sistemi ayarından bağımsızdır.",
+    catalog: "Katalog sürümü: {{version}}",
+  },
   arudha: {
     caption: "Arudha padalar: on iki evin padası, burcu ve evin lordu",
     pada: "Pada",
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index a31c4c4..0917159 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -68,6 +68,7 @@ export const en: Messages = {
     argala: "Argala",
     shadbala: "Shadbala",
     bhava_bala: "Bhava bala",
+    yogas: "Yogas",
     unknown: "Unknown panel",
     loading: "Loading",
     unavailable: "This panel arrives in a later version.",
@@ -216,6 +217,72 @@ export const en: Messages = {
     drishti: "Drishti",
     note: "The bhavas are equal: each midpoint is counted from the lagna in steps of 30° and does not depend on the house system setting. Bhavadhipati is the Shadbala total of the lord.",
   },
+  yogas: {
+    listLabel: "Yogas that form",
+    groupLabel: "Group",
+    allGroups: "All",
+    group: {
+      Mahapurusha: "Pancha Mahapurusha",
+      Lunar: "Moon yogas",
+      Solar: "Sun yogas",
+      Raja: "Raja yogas",
+      Dhana: "Dhana yogas",
+      Exchange: "Exchange yogas",
+      ViparitaRaja: "Viparita raja yogas",
+      NeechaBhanga: "Neecha bhanga",
+      NabhasaCount: "Nabhasa: count yogas",
+      NabhasaAshraya: "Nabhasa: ashraya yogas",
+      Dosha: "Dosha",
+    },
+    yoga: "Yoga",
+    grahas: "Grahas",
+    houses: "Houses",
+    houseLabel: "House {{house}}",
+    relationColumn: "Relation",
+    conditions: "Conditions",
+    relation: { Conjunction: "conjunction", MutualAspect: "mutual aspect", Exchange: "exchange" },
+    condition: {
+      own_sign: "in its own sign",
+      exaltation: "in its exaltation sign",
+      from_lagna: "from the lagna",
+      from_moon: "from the Moon",
+      from_venus: "from Venus",
+      sign_lord_in_kendra: "lord of the debilitation sign in a kendra",
+      exalted_graha_in_kendra: "graha exalted in that sign in a kendra",
+      dispositor_aspect: "aspect of its dispositor",
+    },
+    name: {
+      mahapurusha: { ruchaka: "Ruchaka", bhadra: "Bhadra", hamsa: "Hamsa", malavya: "Malavya", shasha: "Shasha" },
+      lunar: {
+        sunapha: "Sunapha",
+        anapha: "Anapha",
+        durudhara: "Durudhara",
+        kemadruma: "Kemadruma",
+        gajakesari: "Gajakesari",
+        chandra_mangala: "Chandra-Mangala",
+        adhi: "Adhi",
+        shakata: "Shakata",
+        amala: "Amala",
+        vasumati: "Vasumati",
+      },
+      solar: { vesi: "Vesi", vasi: "Vasi", ubhayachari: "Ubhayachari", budha_aditya: "Budha-Aditya" },
+      raja: { kendra_trikona: "Kendra–trikona", dharma_karmadhipati: "Dharma-Karmadhipati", yogakaraka: "Yogakaraka" },
+      dhana: { dhana: "Dhana" },
+      exchange: { maha: "Maha exchange", third_lord: "Third-lord exchange", dainya: "Dainya exchange" },
+      viparita: { harsha: "Harsha", sarala: "Sarala", vimala: "Vimala" },
+      neecha_bhanga: { neecha_bhanga: "Neecha bhanga" },
+      nabhasa_count: { vallaki: "Vallaki", dama: "Dama", pasha: "Pasha", kedara: "Kedara", shula: "Shula", yuga: "Yuga", gola: "Gola" },
+      nabhasa_ashraya: { rajju: "Rajju", musala: "Musala", nala: "Nala" },
+      dosha: { kuja: "Kuja dosha" },
+    },
+    approximate: "approximate",
+    empty: "None of the yogas in the catalog forms. Rules evaluated: {{n}}.",
+    emptyGroup: "No yoga forms in this group: {{group}}.",
+    evaluated: "Rules evaluated: {{n}}. Records found: {{hits}}.",
+    timeUnknown: "Birth time unknown: {{n}} rules that depend on the lagna were not evaluated. The listed yogas are approximate (≈) and carry no houses.",
+    note: "Only D1 is evaluated. Houses are rasi houses counted from the lagna sign and do not depend on the house system setting.",
+    catalog: "Catalog version: {{version}}",
+  },
   arudha: {
     caption: "Arudha padas: the pada of each of the twelve houses, its sign and the lord of the house",
     pada: "Pada",
````

````diff
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index a433953..33c834d 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -68,6 +68,7 @@ export const fr: Messages = {
     argala: "Argala",
     shadbala: "Shadbala",
     bhava_bala: "Bhava bala",
+    yogas: "Yogas",
     unknown: "Panneau inconnu",
     loading: "Chargement",
     unavailable: "Ce panneau arrivera dans une version ultérieure.",
@@ -216,6 +217,72 @@ export const fr: Messages = {
     drishti: "Drishti",
     note: "Les bhavas sont égaux : chaque milieu est compté depuis le lagna par pas de 30° et ne dépend pas du système de maisons choisi. Bhavadhipati est le total Shadbala du maître.",
   },
+  yogas: {
+    listLabel: "Yogas formés",
+    groupLabel: "Groupe",
+    allGroups: "Tous",
+    group: {
+      Mahapurusha: "Pancha Mahapurusha",
+      Lunar: "Yogas de la Lune",
+      Solar: "Yogas du Soleil",
+      Raja: "Raja yogas",
+      Dhana: "Dhana yogas",
+      Exchange: "Yogas d'échange",
+      ViparitaRaja: "Viparita raja yogas",
+      NeechaBhanga: "Neecha bhanga",
+      NabhasaCount: "Nabhasa : yogas de nombre",
+      NabhasaAshraya: "Nabhasa : yogas ashraya",
+      Dosha: "Dosha",
+    },
+    yoga: "Yoga",
+    grahas: "Grahas",
+    houses: "Maisons",
+    houseLabel: "Maison {{house}}",
+    relationColumn: "Relation",
+    conditions: "Conditions",
+    relation: { Conjunction: "conjonction", MutualAspect: "aspect mutuel", Exchange: "échange" },
+    condition: {
+      own_sign: "dans son propre signe",
+      exaltation: "dans son signe d'exaltation",
+      from_lagna: "depuis le lagna",
+      from_moon: "depuis la Lune",
+      from_venus: "depuis Vénus",
+      sign_lord_in_kendra: "maître du signe de chute en kendra",
+      exalted_graha_in_kendra: "graha exalté dans ce signe en kendra",
+      dispositor_aspect: "aspect de son dispositeur",
+    },
+    name: {
+      mahapurusha: { ruchaka: "Ruchaka", bhadra: "Bhadra", hamsa: "Hamsa", malavya: "Malavya", shasha: "Shasha" },
+      lunar: {
+        sunapha: "Sunapha",
+        anapha: "Anapha",
+        durudhara: "Durudhara",
+        kemadruma: "Kemadruma",
+        gajakesari: "Gajakesari",
+        chandra_mangala: "Chandra-Mangala",
+        adhi: "Adhi",
+        shakata: "Shakata",
+        amala: "Amala",
+        vasumati: "Vasumati",
+      },
+      solar: { vesi: "Vesi", vasi: "Vasi", ubhayachari: "Ubhayachari", budha_aditya: "Budha-Aditya" },
+      raja: { kendra_trikona: "Kendra–trikona", dharma_karmadhipati: "Dharma-Karmadhipati", yogakaraka: "Yogakaraka" },
+      dhana: { dhana: "Dhana" },
+      exchange: { maha: "Échange Maha", third_lord: "Échange du maître de la 3e maison", dainya: "Échange Dainya" },
+      viparita: { harsha: "Harsha", sarala: "Sarala", vimala: "Vimala" },
+      neecha_bhanga: { neecha_bhanga: "Neecha bhanga" },
+      nabhasa_count: { vallaki: "Vallaki", dama: "Dama", pasha: "Pasha", kedara: "Kedara", shula: "Shula", yuga: "Yuga", gola: "Gola" },
+      nabhasa_ashraya: { rajju: "Rajju", musala: "Musala", nala: "Nala" },
+      dosha: { kuja: "Kuja dosha" },
+    },
+    approximate: "approximatif",
+    empty: "Aucun yoga du catalogue ne se forme. Règles évaluées : {{n}}.",
+    emptyGroup: "Aucun yoga ne se forme dans ce groupe : {{group}}.",
+    evaluated: "Règles évaluées : {{n}}. Entrées trouvées : {{hits}}.",
+    timeUnknown: "Heure de naissance inconnue : {{n}} règles qui dépendent du lagna n'ont pas été évaluées. Les yogas listés sont approximatifs (≈) et ne portent pas de maisons.",
+    note: "Seul le D1 est évalué. Les maisons sont les maisons rasi comptées depuis le signe du lagna et ne dépendent pas du système de maisons choisi.",
+    catalog: "Version du catalogue : {{version}}",
+  },
   arudha: {
     caption: "Arudha padas : le pada de chacune des douze maisons, son signe et le maître de la maison",
     pada: "Pada",
````

- [ ] **Step 5: Testleri koş**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `Test Files  73 passed (73)`, `Tests  689 passed (689)`.

- [ ] **Step 6: Kapılar**

```bash
pnpm --dir app build 2>&1 | grep "✓ built"
git diff -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
cargo test -p jyotish-desk --test bindings --test ui_fixture 2>&1 | grep -E "^test result"
```
Expected: `✓ built` (`tsc` hatasız; 500 kB uyarısı sürer: D24); `grep` çıktısız (C9); `1 passed` ve `2 passed` (üretilen dosyalar güncel).

- [ ] **Step 7: Commit**

```bash
git add app/ui
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): yogas panel with the grouped list, group filter and shared highlight; names in three languages" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: iki `grep` çıktısız.

---

### Task 6: Uçtan uca: Yogalar paneli

Spec K1 (panel açılmadan istek yok), K2 (doğum saati bilinmiyorsa), K12 (paletten ekleme ve çıkarma; odak), §13 (ortak seçimin yayılması). IPC taklidi Task 4'te eklenmiştir (`compute_yogas`).

**Files:**
- Modify: `app/e2e/desk.spec.ts`

**Interfaces:**
- Consumes: `mockIpc`, `ipcCalls` (`app/e2e/ipc.ts`); fikstürdeki `yogas` ve `yogas_time_unknown` raporları; Task 5'in paneli.
- Produces: iki e2e testi: `yogas: a palette panel with the grouped list; the group filter and the shared highlight` ve `yogas with an unknown birth time: lagna rules are skipped and the rest is marked approximate`.

- [ ] **Step 1: Testleri yaz**

````diff
diff --git a/app/e2e/desk.spec.ts b/app/e2e/desk.spec.ts
index 3c4d714..02746b9 100644
--- a/app/e2e/desk.spec.ts
+++ b/app/e2e/desk.spec.ts
@@ -600,3 +600,87 @@ test("shadbala and bhava bala: two palette panels on one command; units, items v
   expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
   expect(await calls()).toHaveLength(2);
 });
+
+test("yogas: a palette panel with the grouped list; the group filter and the shared highlight", async ({ page }) => {
+  await mockIpc(page, { session: true });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  const toggle = async () => {
+    await page.keyboard.press("Meta+k");
+    const palette = page.getByRole("dialog", { name: "Komut paleti" });
+    await palette.getByRole("combobox").fill("yoga");
+    await palette.getByRole("option", { name: /Yogalar panelini aç\/kapat/ }).click();
+    await expect(page.getByRole("dialog")).toHaveCount(0);
+  };
+  type YogasCall = { input: { jd_ut: number; time_known: boolean; settings: { schema_version: number } } };
+  const calls = async () => (await ipcCalls(page, "compute_yogas")) as YogasCall[];
+  // Rapor panel açılmadan istenmez (ileri modüller spec'i K1).
+  expect(await calls()).toHaveLength(0);
+
+  await toggle();
+  const yogas = page.getByRole("region", { name: "Yogalar" });
+  await expect(yogas).toBeFocused();
+  await expect(yogas).toBeInViewport();
+  await expect(yogas.getByRole("table")).toHaveCount(9);
+  expect(await calls()).toHaveLength(1);
+  expect((await calls())[0].input).toMatchObject({ jd_ut: 2451545, time_known: true, settings: { schema_version: 2 } });
+  // Delhi fikstürü: 9. ve 10. lord (Satürn ve Jüpiter) Koç'ta birlikte.
+  const raja = yogas.getByRole("table", { name: "Raja yogaları" });
+  const dharma = raja.locator('tbody tr[data-yoga="raja.dharma_karmadhipati"]');
+  await expect(dharma.locator("th, td")).toHaveText(["Dharma-Karmadhipati", "Jü Sa", "9 10", "birlikte", "—"]);
+  await expect(yogas.getByRole("table", { name: "Dosha" }).locator("tbody tr").locator("th, td")).toHaveText(["Kuja dosha", "Ma", "9", "—", "Venüs'ten"]);
+  await expect(yogas.getByText("Değerlendirilen kural: 41. Oluşan kayıt: 13.")).toBeVisible();
+  // İlişki ve koşul sütunlarında denetim yoktur: kaydırma kabı klavye odağı alır.
+  const list = yogas.getByRole("group", { name: "Oluşan yogalar" });
+  await list.focus();
+  await expect(list).toBeFocused();
+
+  // Ortak vurgu: satırdaki graha haritadaki grahayı ve onu içeren bütün yoga satırlarını vurgular.
+  await dharma.getByRole("button", { name: "Jüpiter" }).hover();
+  await expect(page.locator('svg [data-graha="Jupiter"]').first()).toHaveClass(/chart-planet--highlight/);
+  await expect(yogas.locator("tr.is-highlighted")).toHaveCount(4);
+  await expect(yogas.locator('tr[data-yoga="lunar.gajakesari"]')).toHaveClass(/is-highlighted/);
+
+  // Grup süzgeci listeyi daraltır; yeniden hesap istenmez ve ayar düzenle birlikte kaydedilir.
+  await yogas.getByRole("combobox", { name: "Grup" }).selectOption("Raja");
+  await expect(yogas.getByRole("table")).toHaveCount(1);
+  await expect(raja.locator("tbody tr")).toHaveCount(2);
+  await yogas.getByRole("combobox", { name: "Grup" }).selectOption("NabhasaAshraya");
+  await expect(yogas.getByText("Bu grupta oluşan yoga yok: Nabhasa: ashraya yogaları.")).toBeVisible();
+  expect(await calls()).toHaveLength(1);
+  await expect
+    .poll(async () => {
+      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: unknown }[] }[] } } } | undefined;
+      const layout = saved?.viewState.panel_layout;
+      return layout?.layouts
+        .find((l) => l.id === layout.active_layout_id)
+        ?.panels.filter((p) => p.kind === "yogas")
+        .map((p) => p.settings);
+    })
+    .toEqual([{ group: "NabhasaAshraya" }]);
+
+  // Aynı komut paneli kaldırır; odak gövdeye düşmez.
+  await toggle();
+  await expect(yogas).toHaveCount(0);
+  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
+});
+
+test("yogas with an unknown birth time: lagna rules are skipped and the rest is marked approximate", async ({ page }) => {
+  await mockIpc(page, { session: true, unknownTime: true });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  await page.keyboard.press("Meta+k");
+  const palette = page.getByRole("dialog", { name: "Komut paleti" });
+  await palette.getByRole("combobox").fill("yoga");
+  await palette.getByRole("option", { name: /Yogalar panelini aç\/kapat/ }).click();
+  const yogas = page.getByRole("region", { name: "Yogalar" });
+  await expect(yogas.getByRole("table")).toHaveCount(4);
+  const calls = (await ipcCalls(page, "compute_yogas")) as { input: { time_known: boolean } }[];
+  expect(calls.map((c) => c.input.time_known)).toEqual([false]);
+  await expect(yogas.getByText(/^Doğum saati bilinmiyor: lagnaya bağlı 17 kural değerlendirilmedi\./)).toBeVisible();
+  await expect(yogas.getByText("Değerlendirilen kural: 24. Oluşan kayıt: 6.")).toBeVisible();
+  // Her satır "≈" işaretini taşır; evler çizgidir.
+  await expect(yogas.locator("tbody tr")).toHaveCount(6);
+  await expect(yogas.locator('tbody tr th span[aria-hidden="true"]')).toHaveText(["≈", "≈", "≈", "≈", "≈", "≈"]);
+  await expect(yogas.locator('tr[data-yoga="dosha.kuja"]').locator("th, td")).toHaveText(["≈yaklaşık Kuja dosha", "Ma", "—", "—", "Venüs'ten"]);
+});
````

- [ ] **Step 2: Koş**

```bash
lsof -i :1421
pnpm --dir app build 2>&1 | grep "✓ built"
pnpm --dir app e2e 2>&1 | tail -1
rm -rf app/test-results
```
Expected: `lsof` çıktısız (port boş; 1420'deki `tauri dev` durdurulmaz); `✓ built` (`tsc -p e2e` hatasız); `18 passed`.

- [ ] **Step 3: Commit**

```bash
git add app/e2e/desk.spec.ts
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(e2e): yogas panel from the palette; group filter, shared highlight and the unknown birth time" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: `grep` çıktısız.

---


### Task 7: Son doğrulama, elle denetim kapısı ve el örneklerinin gözden geçirilmesi

Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle çözülür ve bu görev baştan koşulur. Tek dosya değişikliği, gözden geçirenin `REVIEW.md`'deki tabloya yazdığı satırdır (Step 5).

**Files:**
- Modify: `tests/golden/modules/yogas/REVIEW.md` (yalnızca gözden geçirme tablosu; Step 5). İlerleme notları depo dışıdır; `git add` edilmez.

**Interfaces:**
- Consumes: Task 1–6'nın tamamı.
- Produces: yeşil kapılar, kural taramaları, elle denetim sonucu, sahibine devredilen maddeler ve el örneklerinin bağımsız gözden geçirmesi (`REVIEW.md`'ye ve ilerleme notlarına yazılır).

- [ ] **Step 1: Rust kapıları**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test -p jyotish-desk --test modules -- --nocapture 2>&1 | grep -E "median"
cargo deny check 2>&1 | tail -1
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p jyotish-core -p store -p jyotish-desk --all-targets 2>&1 | tail -1
git diff 9362280..HEAD --stat -- deny.toml Cargo.toml Cargo.lock app/package.json app/pnpm-lock.yaml app/vite.config.ts app/src-tauri/capabilities app/src-tauri/tauri.conf.json crates/store crates/jyotish-core/src/settings.rs crates/jyotish-core/src/chart.rs crates/jyotish-core/src/positions.rs crates/jyotish-core/src/nature.rs crates/jyotish-core/src/shadbala
```
Expected: fmt ve clippy çıktısız; testler: `jyotish-core` lib `276 passed`, golden `8 passed`, `modules_golden` `11 passed`; `store` lib `86 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `46 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `modules` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; yedi `median` satırı, `compute_yogas` 10 ms'nin altında; deny `advisories ok, bans ok, licenses ok, sources ok`; MSRV `Finished`; son `git diff --stat` çıktısız (bağımlılık, yetenek, yapılandırma, `store`, ayar tipi, `Chart`, `Positions`, `nature` ve Shadbala değişmedi: K1, C3). Taban `9362280` değilse komutlardaki commit, Task 1 adım 1'de not edilen tabandır.

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
Expected: `lsof` çıktısız; `Test Files  73 passed (73)`, `Tests  689 passed (689)`; `✓ built`; e2e `18 passed`; görsel `10 passed` (temel PNG'ler değişmedi: D24); betik testleri `pass 14`, `fail 0`; `git status --short` boş.

- [ ] **Step 3: Kural taramaları (`9362280`'den bu yana)**

```bash
git diff 9362280..HEAD -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log 9362280..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff 9362280..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
git diff 9362280..HEAD --name-only | grep -vE '^(app|crates|scripts|tests|docs)/'
echo "trailers: $(git log 9362280..HEAD --format=%B | grep -c '^Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>$') commits: $(git rev-list --count 9362280..HEAD)"
git diff 9362280..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
git diff 9362280..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\(|window\.(print|open)\('
git diff 9362280..HEAD -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
grep -rn '"source"' tests/golden/modules | grep -v '"source": "hand"'
ls tests/golden/modules/yogas/*.json | wc -l
grep -rnE 'serde\(default' crates/jyotish-core/src
grep -rnE 'XMLHttpRequest|WebSocket|\bfetch\(' app/ui/modules app/ui/panels
git diff 9362280..HEAD --stat -- app/ui/print app/e2e/visual.spec.ts-snapshots
git tag --points-at HEAD
```
Expected: `trailers:` satırındaki iki sayı eşit; `grep -ciE` satırı `0`, `ls … | wc -l` satırı `15` yazar; bunların dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur): yasaklı ad yok (üretilen fikstürde de; düğüm eksenine dayalı modern yoganın adı hiçbir yerde geçmez), beklenmeyen yerde dosya yok, `console` ve yasak çağrı yok, yeni stilde hex ya da `rgb(` yok (C9), el örneklerinde `hand` dışında kaynak yok (K9), çekirdek tiplerinde `serde(default)` yok, modül ve panel kodunda ağ çağrısı yok (C1), baskı ve görsel temeller değişmedi (K10, D24), ajan etiket atmamış.

- [ ] **Step 4: Elle denetim kapısı (`pnpm --dir app tauri dev`)**

Gerçek uygulamada, deneme danışanlarıyla (gerçek danışan verisi kullanılmaz). Her madde ilerleme notlarına "geçti" ya da bulgu olarak yazılır; bulgu varsa düzeltme commit'i ve bu görev baştan. Her maddenin başında, otomatik bir karşılığı olup olmadığı yazılıdır. **[otomatik]** maddelerin karşılığı testlerde vardır ve yürütücü ajan bunları testin adını göstererek "geçti" sayabilir; **[insan]** maddeler yalnızca gerçek uygulamaya bakan bir insan tarafından doğrulanabilir ve sahibi yoksa ona devredilir (bu devir `git push`'u engellemez; hiçbir şey yayımlanmaz).

- [ ] **[otomatik]** Komut paletinde "Yogalar panelini aç/kapat" komutu yalnızca harita açıkken vardır; panel etkin düzene eklenir ve odak ona geçer; aynı komut paneli çıkarır ve odak gövdeye düşmez; rapor panel açılmadan istenmez. (e2e `yogas: a palette panel …`; `Palette.test.tsx`, `layout/model.test.ts`)
- [ ] **[otomatik]** Panel oluşan yogaları katalog sırasıyla, grup başına bir tabloda gösterir; her satır yoganın adını, grahaları, evleri, ilişki türünü ve sağlanan alt koşulları taşır; altta değerlendirilen kural ve kayıt sayısı, D1 notu ve katalog sürümü durur. (`YogasPanel.test.tsx › lists the yogas that form …`, e2e)
- [ ] **[otomatik]** "Grup" süzgeci listeyi daraltır, düzenle birlikte saklanır ve yeniden hesap istemez; kaydı olmayan grupta tablo yerine açıklama çıkar; hiç kayıt taşımayan raporda spec'in boş durum metni ve kural sayısı çıkar. (`YogasPanel.test.tsx`, e2e)
- [ ] **[otomatik]** Doğum saati "bilinmiyor" olan haritada lagna isteyen 17 kural Rust'ta atlanır; panel bunu söyler, listelenen her yoga "≈" işaretini ve "yaklaşık" sözünü taşır, evler çizgidir. (`YogasPanel.test.tsx › unknown birth time …`; e2e `yogas with an unknown birth time …`; çekirdek `without_a_lagna_seventeen_rules_are_skipped_…`; Rust `modules` testi)
- [ ] **[otomatik]** Hata durumunda süzgeç görünür durur; "Yeniden dene" klavyeyle kullanılınca odak panelde durur; yükleme 1 saniyeyi geçerse iskelet çıkar; tablolar klavye odağı alan tek bir kaydırma kabındadır. (`YogasPanel.test.tsx`)
- [ ] **[otomatik]** Ortak vurgu: satırdaki bir graha haritadaki grahayı ve onu içeren bütün yoga satırlarını vurgular; bir ev, onu taşıyan satırları vurgular; tıklama sabitler, `Esc` kaldırır. (`YogasPanel.test.tsx › shared highlight …`, e2e)
- [ ] **[otomatik]** Katalogdaki 41 kuralın, 11 grubun, 8 koşulun ve 3 ilişki türünün üç dilde adı vardır; klasik adlar üç dilde aynıdır. (`YogasPanel.test.tsx › every catalog rule …`, `i18n.test.ts`)
- [ ] **[insan]** Gerçek motorla ayarlar: ayanamsa değişince ya da zodyak kipi tropikal rasiye alınınca panel yeniden hesaplanır (burç değiştiren graha yoga listesini değiştirebilir); ev sistemi Placidus'a alınınca liste değişmez (K4). (e2e taklidi raporu yeniden hesaplamaz; bu yalnızca gerçek motorla görülür.)
- [ ] **[insan]** Panel açık ve koyu temada, Türkçe, İngilizce ve Fransızca dilde okunur; dar panelde tablolar yatay kaydırılır ve taşmaz; grup başlıkları ayırt edilir; uzun koşul metinleri satır kırar.
- [ ] **[insan]** VoiceOver: her tablonun başlığı (grup adı) ve sütun adları okunur; graha kısaltması tam adıyla, ev numarası "9. ev" diye okunur; "≈" işareti "yaklaşık" diye okunur; "Grup" seçicisi adıyla duyurulur; kaydırma kabı "Oluşan yogalar" diye duyurulur.
- [ ] **[insan]** Bilinen bir haritada (astrologun kendi elindeki bir örnek) liste astrologun beklediğiyle karşılaştırılır. Uyuşmazlık bulgu olarak yazılır; okuma farkıysa (Kemadruma'nın iptalleri, Neecha bhanga'nın koşulları ve Ay'ın kendisinden sayılmaması, değişim sınıfının evlerle belirlenmesi, Dharma-Karmadhipati'nin ek kayıt olması, mahapurushanın yalnızca lagnadan sayılması, kendra–trikonada lagna lordunun iki kümede de sayılması) "Bu planın kapsamadıkları"na ve Kararlar'a göre sınıflanır.
- [ ] **[insan]** Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum verisi, konum ya da dosya yolu görünmez.

Kapı geçtiyse ilerleme notlarına `Plan 18 Task 7 elle denetim: geçti (<tarih>)` yazılır; **[insan]** maddeleri devredildiyse `Plan 18 Task 7 elle denetim: otomatik maddeler geçti, insan maddeleri sahibine devredildi (<tarih>)` yazılır. Bu adım commit üretmez.

- [ ] **Step 5: El örneklerinin bağımsız gözden geçirilmesi (gözden geçirme maddesi)**

Spec §13: "`derivation` ayrı bir gözden geçirmede kuraldan yeniden hesaplanır." Gözden geçiren (motoru yazan dışında bir ajan ya da insan), `tests/golden/modules/yogas/` altındaki on beş örneğin her birinde yalnızca `input`'tan başlar ve motoru çalıştırmadan, Rust kaynağını okumadan 41 kuralı spec §8'den ve `REVIEW.md`'deki okuma kurallarından yeniden değerlendirir: (1) dokuz grahanın burcu ve (lagna varsa) lagnadan evi; (2) lagnanın on iki ev lordu; (3) K6 sınıfı (Ay'ın uzanımı, Merkür'ün burcundaki grahalar); (4) beş mahapurusha; (5) Ay'ın iki yanı, Kemadruma'nın dört koşulu, Gajakesari, Chandra-Mangala, Adhi, Shakata ve iptali, Amala ve Vasumati (iki referans); (6) Güneş'in iki yanı ve Budha-Aditya; (7) yedi grahanın 21 çiftinin ilişkileri (aynı burç; 7. burç; Mars'tan 4. burçtaki Satürn; birbirinin burcu); (8) kendra–trikona çiftleri, Dharma-Karmadhipati, yogakaraka, dhana çiftleri; (9) değişen çiftler ve değişen evlerin sınıfı; (10) üç viparita; (11) düşüşteki grahalar ve üç koşul; (12) dolu burç sayısı ve burç türleri; (13) Mars'ın üç referanstan evi. Sonra kendi kayıt listesini `expected.hits` ile (kimlik, grahalar, evler, ilişkiler, koşullar ve sıra), `absent` ve `cancelled` listelerini kendi sonucuyla karşılaştırır; farklar yazılır. Fark varsa önce kural yeniden okunur; düzeltme hangi tarafta olursa olsun ayrı commit'tir ve beklenen değer motor çıktısına bakarak değiştirilmez. Ayrıca testlerdeki elle yazılmış tablolar (`hand_mutual`, `HAND_MAHAPURUSHA`, `HAND_FALL`, `HAND_COUNT`, `HAND_YOGAKARAKA`, `HAND_EXCHANGE`, `HAND_NEEDS_LAGNA`) aynı kuraldan satır satır denetlenir.

Gözden geçiren, `REVIEW.md`'nin sonundaki tabloya adını (ya da ajan kimliğini), tarihi ve sonucu yazar:

```bash
git add tests/golden/modules/yogas/REVIEW.md
git diff --cached --stat
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "docs(golden): yoga hand fixtures re-derived by an independent reviewer" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
cargo test -p jyotish-core --test modules_golden 2>&1 | grep -E "^test result"
```
Expected: yalnızca `REVIEW.md` değişir (tablo satırı; fark bulunduysa ilgili örnek ayrı commit'tedir); `grep` çıktısız; `11 passed`. Sonuç ilerleme notlarına `Plan 18 el örnekleri yeniden türetildi: 15/15 (<tarih>, <gözden geçiren>)` olarak yazılır.

---

## Başarı ölçütleri

- `jyotish-core` 41 kurallık yoga kataloğunu verir: beş mahapurusha, on Ay ve dört Güneş yogası, üç raja, dhana, üç değişim sınıfı, üç viparita, Neecha bhanga, yedi sayı ve üç ashraya yogası, Kuja dosha. Her kayıt kimliğini, grubunu, grahalarını, evlerini, ilişkilerini ve sağlanan alt koşulları taşır; güç puanı ve hüküm yoktur. Kurallar kural düzeyinde el örnekleri, elle yazılmış yedi tablo, raporun tamamını yazan on beş el fikstürü (kaynak `hand`; her kuralın oluşan ve oluşmayan örneği, üç iptal durumu) ve proptest değişmezleriyle doğrulanır.
- Lagna yoksa lagna isteyen 17 kural değerlendirilmez ve sayılır; diğer kayıtlar yaklaşık işaretlidir ve ev taşımaz.
- `compute_yogas` ortak kalıpla çalışır (motor iş parçacığı, `blocking`, `CommandError`), beş golden anında v1 haritasıyla çapraz denetlenir, ev sistemi ayarından bağımsızdır ve bütçesinin içindedir (≤ 10 ms).
- Yogalar paneli paletten eklenir; kayıtları grup başına tabloda gösterir, grup süzgecini saklar, ortak vurguya katılır. Boş, yükleniyor, hata, dolu ve "saat bilinmiyor" durumları testlidir; renk tek başına anlam taşımaz; yorum metni yoktur; her kimliğin üç dilde adı vardır.
- `Chart`, `open_client`, `CalcSettings`, `ViewState`, `store`, `Positions`, Shadbala, baskı düzenleri ve görsel temeller değişmemiştir; v1 golden testleri değişmeden geçer.
- Kapılar: `jyotish-core` lib 276, `modules_golden` 11, `store` lib 86 (+1 ign.), `jyotish-desk` `modules` 1; Vitest 73/689; e2e 18; görsel 10; `cargo deny` temiz; diğer sayılar tabandaki gibi. Elle denetim kapısının **[otomatik]** maddeleri testlerle doğrulanmış, **[insan]** maddeleri yapılmış ya da sahibine devredilmiştir; el örnekleri bağımsız olarak yeniden türetilmiş ve `REVIEW.md`'ye yazılmıştır.

## Bu planın kapsamadıkları

- **Spec'in 3.5 için kapsam dışı saydıkları:** akriti ve dala nabhasa yogaları, bölümsel haritalarda yoga, yoga gücü puanı, kullanıcı tanımlı kural, arishta ve ayur yogaları, Jaimini raja yogaları, dosha iptalleri; düğüm eksenine dayalı modern yoga (Ek A).
- **Sonraki alt projeler (3.6+):** uyum, Muhurta, Prashna, arama. Yoga koşulunun harita aramasında kullanılması 3.9'un, Kuja dosha'nın uyum raporunda gösterimi 3.6'nın işidir.
- İkinci bir okuma (ör. mahapurushanın Ay'dan da sayılması, değişim sınıfının lordlukla belirlenmesi, Dharma-Karmadhipati'nin genel kaydın yerine dönmesi, Neecha bhanga'nın başka koşulları): K7'ye göre ancak ikinci seçenek uygulanınca ayar olur; bu planda ayar eklenmez ve katalog sürümü 1'dir.
- Satır düzeyinde çoklu vurgu (D22: ortak seçim deposu tek hedef taşır) ve yoga satırından haritada ev ve grahaların birlikte işaretlenmesi.
- Raporda `method` ve `warnings` alanı (D3); "doğum saati bilinmiyor" dışındaki atlama nedenleri.
- Kataloğun IPC ile okunması (D17: fikstür yolu) ve oluşmayan yogaların listelenmesi.
- Yogaların baskı düzeni (K10); raporun oturum içi paylaşılan önbelleği (D20) ve veritabanında saklanması (spec §16).
- Paket parçalama (D24: 500 kB uyarısı).
- Yorum metinleri ve AI yorum paneli; Windows ve Android.
- v1'den taşınan seyrek test dalgalanması (`NotesPanel.test.tsx › New note adds an empty note…`, yük altında): bu planın dosyalarına dokunmaz; ayrı bir bakım işidir.

## Spec'e geri bildirim (spec kusurları ve boşlukları)

Plan yazılırken bulunan ve Kararlar'da çözülen maddeler. "Kusur" spec'in kendi içinde çelişen, harfiyen uygulanamayan ya da kendi property'siyle tutmayan yeridir; "boşluk" yazılmamış ayrıntıdır.

1. **Kusur — §8 Veri modeli: `pub enum Relation`.** v1'in `dignity::Relation` tipi (doğal dostluk: `Friend`, `Neutral`, `Enemy`) aynı adla `jyotish_core` kökünden dışa aktarılır; iki `Relation` aynı crate kökünde ve aynı TS bağlama dosyasında duramaz. Çözüm (D2): `YogaRelation`. Spec tipin adını düzeltmeli.
2. **Kusur — §8 Arayüz: "Boş durum: Katalogdaki yogalardan hiçbiri oluşmuyor".** §8'in kendi property'si "sayı yogalarından tam biri oluşur" der: yedi graha her zaman 1–7 burca dağılır ve rapor en az bir kayıt taşır. Boş durum erişilemezdir. Çözüm (D23): boş durum süzülen grup için gösterilir; spec'in metni savunma amaçlı çizilir. Spec boş durumu "seçili grupta oluşan yoga yok" diye tanımlamalı.
3. **Kusur — §8 Neecha bhanga: "düşüş burcunun lordu lagnadan ya da Ay'dan kendrada".** Mars Yengeç'te düşüşteyken burcun lordu Ay'dır ve Ay kendisinden 1. evdedir (kendra): koşul her haritada sağlanır ve Mars'ın düşüşü her zaman iptal görünür. Çözüm (D13): Ay kendisinden sayılmaz; lord Ay ise yalnızca lagna referansı geçerlidir. Spec bu istisnayı yazmalı.
4. **Kusur — K7 ve K8 ile §8 `YogaReport`.** K7 "her modül sonucu `method` kimliği taşır", K8 "ölümcül olmayan durumlar modül sonucunun `warnings` alanındadır" der; §8'in tipinde ikisi de yoktur. Çözüm (D3): §8'in tipi korunur; yöntem kimliği `catalog_version`'dır; `LowPrecision` uyarısı rapora taşınmaz. Spec ya K7 ve K8'e "yoga raporu `catalog_version` taşır, uyarı taşımaz" istisnasını eklemeli ya da tipe iki alanı koymalı.
5. **Kusur — §8 Değişim: "lordlar {1, 2, 4, 5, 7, 9, 10, 11} içindeyse Maha; … biri 6., 8. ya da 12. lordsa Dainya".** Beş graha iki evin lordudur; lordluk üzerinden okuma aynı değişimi iki sınıfa sokar (Koç lagnada 1. ve 2. evlerin değişiminde Mars 8. lord olduğu için Dainya da olurdu). Çözüm (D11): sınıfı değişen iki ev belirler. Spec "değişen evler" demeli.
6. **Kusur — §8 Arayüz: "Satıra gelince grahalar ve evler ortak seçimle tüm panellerde vurgulanır".** Ortak seçim (UI spec §3.4; `highlight/store`) tek hedef taşır: bir graha, bir ev ya da bir burç. Bir satırın bütün grahaları ve evleri aynı anda vurgulanamaz. Çözüm (D22): satırdaki her graha ve ev ayrı vurgu düğmesidir; ters yönde vurgudaki öğeyi içeren bütün satırlar işaretlenir. Spec ya bu davranışı yazmalı ya da UI spec §3.4'e çoklu hedefi eklemeli (bütün panelleri etkiler).
7. **Kusur — §8 Kararlar: "Lagnaya bağlı yogalar doğum saati bilinmiyorsa değerlendirilmez; Ay ve Güneş yogaları 'yaklaşık' işaretiyle döner".** Ay grubundaki Kemadruma ve Shakata'nın tablodaki koşulu lagna içerir (iptal kuralları); ikisi hem "Ay yogası" hem "lagnaya bağlı"dır. Amala, Vasumati, Neecha bhanga ve Kuja dosha lagnayı seçeneklerden biri olarak kullanır; nabhasa yogaları Ay'ın burcuna bağlıdır. Bunların doğum saati bilinmeyen haritadaki davranışı yazılmamış ya da çelişkilidir. Çözüm (D4): koşulunda lagna zorunlu olan 17 kural atlanır; diğer 24 kural lagna seçenekleri olmadan değerlendirilir ve her kayıt yaklaşıktır.
8. **Boşluk — §8 `grahas` ve `houses`.** Alanların kural başına içeriği yazılmamış: ışığın (Ay, Güneş) listelenip listelenmediği; Neecha bhanga'da koşulu sağlayan grahaların durumu; `houses`'ın lordu olunan ev mi bulunulan ev mi olduğu. Çözüm (D5).
9. **Boşluk — §8 Ay yogaları.** Kemadruma'da "boş" ve "yalnız"ın hangi grahalara göre okunduğu; "Ay'dan kendralar"ın 1. evi içerip içermediği (Gajakesari'de birlikte duran Jüpiter); Amala'da Merkür'ün ve düğümlerin sınıfı. Çözüm (D6).
10. **Boşluk — §8 `conditions`.** Koşul kimlikleri sayılmamış; mahapurushanın burç mu derece mi düzeyinde olduğu (v1'in dignity sırası Merkür'ü Başak'ın 15°'sinden sonra moolatrikona ya da kendi burcunda sayar); Kuja dosha için "üç referans ayrı alanlarda" denmiş ama `YogaHit`'te ayrı alan yok. Çözüm (D7): sekiz kimlik; referanslar koşuldur.
11. **Boşluk — §8 `relations`.** Hangi kaydın ilişki taşıdığı (yalnızca çift kuralları ve iki "birlikte" yogası), bir çiftin iki ilişkiyi birlikte taşıyabildiği (karşılıklı bakış ve değişim) ve karşılıklı bakışın hangi burç uzaklıklarında oluştuğu (7. burç; Mars'ın 4. ile Satürn'ün 10. bakışı) yazılmamış. Çözüm (D8).
12. **Boşluk — §8 Raja ve Dhana.** Lagna lordunun hem kendra hem trikona lordu sayılması; Dharma-Karmadhipati'nin genel kaydın yerine mi yanına mı döndüğü; yogakaraka'nın konumdan bağımsız olması; dhana cümlesinin okunuşu. Çözüm (D9, D10).
13. **Boşluk — §8 Test ve §13 şeması.** Kural başına "oluşan, oluşmayan, iptal" örneklerinin fikstürde nasıl gösterileceği ve Neecha bhanga'nın "iptal edilen durumu"nun ne olduğu yazılmamış. Çözüm (D15): `hits`, `absent`, `cancelled`; okuma kuralları ve gözden geçirme kaydı `REVIEW.md`'de.
14. **Boşluk — §8 Test: "her `id` katalogdadır ve üç dilde adı vardır".** Katalog Rust'ta, adlar arayüzdedir ve `id` dizgidir; ikisini bağlayan yol yazılmamış. Çözüm (D17): katalog fikstüre yazılır ve Vitest oradan denetler. 3.9 arama yüzeyi kataloğu çalışma zamanında isteyecekse spec bir `yoga_catalog` komutu tanımlamalı.
15. **Boşluk — §8 Arayüz: düzen ve durumlar.** Tablo düzeni, sütunlar, sayaç satırı, "yaklaşık" işaretinin gösterimi, doğum saati bilinmeyen haritadaki açıklama ve katalog sürümünün gösterimi yazılmamış. Çözüm (D21).

## Self-Review

**1. Spec kapsamı.**

| Gereksinim (ileri modüller spec'i) | Görev |
|---|---|
| §8 Veriyle tanımlı katalog; saf yüklemler; `YOGA_CATALOG_VERSION = 1`; yalnızca D1 | Task 1, 2 (`YOGA_CATALOG`; D1) |
| §8 Ortak tanımlar (birlikte, karşılıklı bakış, değişim, ilişkili; sayılan grahalar) | Task 1 (`yoga_relations`, `is_mutual_aspect`, `is_exchange`, `COUNTED_GRAHAS`) |
| §8 Pancha Mahapurusha; yalnızca lagnadan | Task 1 (`mahapurusha`; `HAND_MAHAPURUSHA`) |
| §8 Sunapha, Anapha, Durudhara (birbirini dışlar) | Task 1 (`flank`) |
| §8 Kemadruma (iptal), Gajakesari, Chandra-Mangala, Adhi, Shakata (iptal), Amala, Vasumati | Task 1 (`luminaries.rs`) |
| §8 Vesi, Vasi, Ubhayachari (birbirini dışlar), Budha-Aditya | Task 1 (`flank`, `budha_aditya`) |
| §8 Kendra–trikona, Dharma-Karmadhipati, Yogakaraka | Task 2 (`HAND_YOGAKARAKA`) |
| §8 Dhana | Task 2 |
| §8 Değişim: Maha, 3. ev lordu, Dainya | Task 2 (`exchange_kind`; `HAND_EXCHANGE`) |
| §8 Viparita raja | Task 2 (`viparita`) |
| §8 Neecha bhanga (üç koşul; Akrep'te ikinci koşul yok; koşullar listelenir) | Task 1 (`neecha_bhanga`; `HAND_FALL`) |
| §8 Nabhasa sayı ve ashraya | Task 1 (`HAND_COUNT`) |
| §8 Kuja dosha (3.1'in olgusu; iptal yok) | Task 1 (`kuja`) |
| §8 Karar: bir çift birden çok ilişki taşıyorsa tek kayıt | Task 2 (testler), Task 3 (fikstür) |
| §8 Karar: iptal kuralları yalnızca tablodakiler | Task 1; Task 3 (`cancelled`) |
| §8 Karar: doğum saati bilinmiyorsa | Task 3 (`yogas`; D4) |
| §8 Karar: modern düğüm ekseni yogası kataloğa alınmaz | Katalogda yok; Global Constraints |
| §8 Veri modeli (`YogaGroup`, ilişki tipi, `YogaHit`, `YogaReport`); kararlı `id` | Task 1, 3 (D2, D3) |
| §8 Komut `compute_yogas` | Task 3 (toplayıcı), Task 4 (komut) |
| §8 Arayüz: `yogas` paneli, grup süzgeci, gruplu liste, vurgu, boş durum, açıklama metni yok | Task 5 |
| §8 Test: kural başına el örneği | Task 3; Task 7 Step 5 |
| §8 Test: property'ler (altı madde) | Task 3 (`yoga_properties`), Task 5 (üç dilde ad) |
| §8 Test: uçtan uca | Task 4 (D19) |
| §8 Gizlilik, performans (≤ 10 ms) | C1; Task 4 (ölçüm), Task 7 |
| K1, K2, K3 | Task 1–4 |
| K4, K5 | Task 3 (`collector_follows_…the_rasi_frame…` testi), Task 4 (ev sistemi denetimi) |
| K6 | Task 1 (`nature` çağrısı; kendi listesi yok) |
| K7 (ayar yok) | D3; "Bu planın kapsamadıkları" |
| K8, §14 | D18 (yeni kod yok), C1 |
| K9, §13 (el örnekleri, değişmezler, ikinci yazım, çapraz denetim, komut katmanı) | Task 1–4; Task 7 Step 5 |
| K10, K11, K12 | D24; klasik adlar üç dilde aynı (D17); Task 5, 6 |
| Ek A (`exchange.third_lord`; modern yoga uygulanmaz) | Task 2; C13 taramaları |

Boşluk: yok. Spec'in istemediği ama planın eklediği parçalar: koşul ve ilişki kapsamının fikstür testinde sabitlenmesi, `REVIEW.md` (okuma kuralları), `YogaRule::matches` ve `yoga_rule` (3.9 için), fikstürdeki katalog (D17), v1 dignity alanıyla çapraz denetim (D19), kaydırma kabı (D21), sayaç ve katalog sürümü satırları.

**2. Yer tutucu taraması.** "TODO", "TBD", "daha sonra", "uygun hata işleme" yok. Task 7'deki `<tarih>` ve `<gözden geçiren>` denetimin yapıldığı günün ve yapanın yazılacağı yerdir. Her kod adımı tam dosya ya da `9362280` ağacına (ve önceki görevlere) karşı üretilmiş diff taşır.

**3. Tip tutarlılığı.**
- `YogaGroup` (11 değer), `YogaRelation` (3 değer), `YogaMatch`, `YogaRule` Task 1'de; `YogaHit { id, group, grahas, houses, relations, conditions, approximate }` ve `YogaReport { catalog_version, hits, evaluated, skipped_time_unknown }` Task 3'te Rust'ta tanımlanır (alan adları spec §8'dekiyle aynıdır; ilişki tipinin adı D2'ye göre `YogaRelation`'dır); TS bağlaması Task 4'te üretilir; `ui_fixture.rs`, `tests/modules.rs`, panel ve e2e aynı adları okur.
- Kural kimlikleri (41) `YOGA_CATALOG`'da, `HAND_NEEDS_LAGNA`'da, on beş JSON dosyasında, üç dil dosyasının `yogas.name` ağacında (`<grup>.<ad>` iç içe anahtar) ve panel testlerindeki `data-yoga` değerlerinde aynıdır.
- Koşul kimlikleri (8) `YOGA_CONDITIONS`'ta, kuralların `COND_*` sabitlerinde, JSON dosyalarında, `yogas.condition` anahtarlarında ve `REVIEW.md`'de aynıdır.
- El fikstürü alanları (`hits[].{id, grahas, houses, relations, conditions}`, `absent`, `cancelled`, `evaluated`, `skipped_time_unknown`) düzenekteki `YogasExpected` ve `ExpectedYoga`, README satırı ve on beş JSON dosyasında aynıdır.
- Komut adları: Rust `compute_yogas`; gövde `compute_yogas_blocking`; TS `computeYogas`; e2e taklidindeki anahtar `compute_yogas`; bağımsız değişken adı `input`.
- Fikstür anahtarları `yogas`, `yogas_time_unknown`, `yoga_catalog`, `yoga_conditions` `ui_fixture.rs`, `test/fixture.ts`, panel testi ve `e2e/ipc.ts`'te aynıdır.
- Panel türü `yogas`: `PANEL_KINDS`, `MODULE_PANELS`, `PANEL_REGISTRY`, `panel.yogas` i18n anahtarı, testlerdeki `Panel.kind` değeri ve e2e'de kaydedilen düzen aynı adı taşır. Panel ayar anahtarı `group`'tur; değerleri `"all"` ve `YOGA_GROUPS`'un on bir değeridir (Rust `YogaGroup::ALL` sırası; panel testi sırayı fikstürdeki katalogla karşılaştırır).
- `useYogas` panelin tek veri kaynağıdır; `BalaScroll` Plan 17'nin son incelemesinde `bala.tsx`'e eklenen bileşendir.

**4. Kural denetimi.** C1 (`ChartInput` loglanmaz; yeni hata kodu yok), C2′ (komut dizgi almaz), C3 (`store`, ayar tipi ve görünüm kaydı değişmez), C4 (bütün kayıtlar Rust'tan; gruplama ve süzme gösterim), C5 (panel ayarı mevcut kayıt yoluyla), C7 (kısayol eklenmez), C9 ("≈" ve "yaklaşık"; yeni stilde hex yok), C10 (iki üretilen dosya komutla), C12 (her yeni anahtar üç dilde; yeni hata kodu yok), C13 (her commit'te tarama), K9 (yalnızca `hand`), K10 (baskı ve görsel temeller değişmez) görevlerin adımlarında ve Task 7 Step 3'te denetlenir.

**5. Koşulan doğrulama.** Bu plandaki kod, deponun `9362280` üzerindeki bir kopyasında (kısa bir yolda, Rust 1.100) altı commit olarak uygulanmış ve her commit'te ölçülmüştür: `jyotish-core` lib 239 → 259 → 267 → 276; `modules_golden` 9 → 11; `jyotish-desk` `modules` 1 (ortancalar: `compute_yogas` 0,63–0,64 ms; diğer altı komut 0,56–0,79 ms); Vitest 677 → 677 → 689 (dosya 72 → 73); e2e 16 → 18. Her commit'te `cargo fmt --check` ve `clippy -D warnings` (1.100) çıktısızdır. Son durumda `cargo test --workspace`, `cargo deny check` (dört `ok`), 1.93 ile `cargo check`, `pnpm build` (543,2 kB; D24), görsel 10 (temel dosyalar değişmeden) ve yayın betiği testleri 14 yeşildir; Task 7 Step 3'ün taramaları beklendiği gibidir. "Testlerin başarısız olduğunu doğrula" ve "derlemenin başarısız olduğunu doğrula" adımlarındaki çıktılar da aynı kopyada, bir önceki görevin ağacına yalnızca o adımın değişiklikleri konarak ölçülmüştür. Ayrıca bu belgedeki bloklar (23 yeni dosya ve 26 diff) belgeden okunup sırayla temiz bir `9362280` ağacına uygulanmış ve her görevin sonunda o commit'in ağacıyla birebir aynı sonucu vermiştir (üretilen iki dosyayı adımlardaki komutlar üretir). Prototip önce `dedfb4b` üzerinde yazılmış, Plan 17'nin son inceleme commit'leri gelince `9362280`'e taşınmıştır; taşımada tek çakışma `tests/modules.rs`'in içe aktarma listesindeydi ve panel, son incelemenin getirdiği `BalaScroll` kalıbına uyduruldu.

El fikstürlerinin değerleri bu plan yazılırken motordan bağımsız bir hesapla türetilmiştir: kural metninden (spec §8 ve Kararlar) başka bir dilde, kümeler ve tablolarla yazılmış ayrı bir hesap her haritada 41 kuralı değerlendirmiş, `derivation` satırlarını ve `expected` listelerini o hesap üretmiştir. Motor, on beş örneğin 155 kaydında (kimlik, grahalar, evler, ilişkiler, koşullar ve sıra) ilk koşuda birebir uyuşmuştur; iki hesap ayrıca 20 000 rastgele haritada (yarısı birkaç burca toplanmış, yüzde on beşi lagnasız) kayıt listesinin tamamında uyuşmuştur. Hiçbir beklenen değer motor çıktısına bakılarak değiştirilmemiştir. İki türetmeyi de aynı yazar yazmıştır: kuralın aynı yanlış okunuşu ikisine de girmiş olabilir. Task 7 Step 5 bunun için vardır ve bu belgedeki doğrulamayla yerine getirilmiş sayılmaz.

Yazım sırasında düşenler ve düzeltmeleri (hiçbirinde beklenen değer motor çıktısından kopyalanmadı): (1) `lords.rs` testlerinin ilk taban haritası yanlış kurulmuştu: "hiçbir lord çifti ilişkili değil" diye yazılan haritada Mars İkizler'de ve Merkür Koç'taydı (değişim), Satürn Mars'tan 4. burçtaydı (karşılıklı bakış) ve Venüs ile Satürn karşı karşıyaydı; üç test düştü. Kural doğruydu, örnek yanlıştı; taban, altı eksenin her birinden tek burç seçilerek yeniden kuruldu (yedi graha yedi ayrı burca dağılırsa en az bir karşıtlık kaçınılmazdır; bu yüzden zararsız bir çift, Güneş–Merkür, aynı burca kondu). (2) Fikstür kapsam testi ilk koşuda düştü: Yuga örneği sadeleştirilince "bir kayıtta iki ilişki" durumu örneklerden çıkmıştı; Durudhara örneğinde Satürn Koç'a alınarak (Mars'la değişim ve karşılıklı bakış) geri kondu. (3) İlk fikstür taslaklarında iki harita amaçlanan yogayı vermedi (Vesi yerine Ubhayachari; Ketu'nun bozduğu Amala) ve bir harita Neecha bhanga'yı yalnızca bakışla değil iki koşulla veriyordu; haritalar bağımsız hesabın çıktısına bakılarak yeniden kuruldu. (4) clippy 1.100 elle yazılmış karşılıklı bakış tablosunda `matches!` istedi ve kullanılmayan bir sabiti (`dead_code`) bildirdi. **Koşulmayanlar:** `pnpm tauri dev` ile gerçek uygulama ve gerçek veritabanı (Task 7 Step 4'ün **[insan]** maddeleri; özellikle gerçek motorla ayar değişiminin panele yansıması); bir astrologun elindeki örnekle karşılaştırma; el örneklerinin ikinci bir kişi tarafından yeniden türetilmesi (Task 7 Step 5); uzak CI (hiçbir şey gönderilmedi).
