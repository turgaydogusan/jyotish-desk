# Plan 24 — Avasthalar (Baladi ve Jagradadi; Lajjitadi kuralları ve küme sonucu; Shayanadi formülü; `compute_avasthas` komutu, Avastha paneli ve graha tablosunun Baladi sütunu)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Bu plan Claude dışı bir kodlama ajanı tarafından, ayrı bir git çalışma ağacında da yürütülebilir: gereken her yol, kod ve komut bu belgededir; başka bir oturumun bağlamına gönderme yoktur.

**Goal:** Grahanın dört "durum" dizisini eklemek (yol haritası §4.4). (1) **Baladi:** burç içi dereceden beş yaş basamağı; tek burçta düz, çift burçta ters sıra. (2) **Jagradadi:** burçtan üç uyanıklık durumu. (3) **Lajjitadi:** yerleşim, birliktelik ve bakıştan altı durum; bir graha aynı anda birden çok durumda olabilir, sonuç kümedir ve her kayıt kuralı sağlayan grahaları taşır. (4) **Shayanadi:** nakshatra, graha sayısı, navamsa dilimi, doğum nakshatrası, gün doğumundan geçen ghati ve lagnadan tek bir sayma formülüyle on iki durum. Üç katman: `jyotish-core`'da saf `avastha` modülü ve ince toplayıcı; uygulamada tek komut (`compute_avasthas`); arayüzde paletten açılan `avastha` paneli ve graha tablosunda isteğe bağlı Baladi sütunu. Yorum, puan ve hüküm yoktur: modül yalnızca sınıfı ve kuralı sağlayan grahaları döner.

**Architecture:**
- **Saf çekirdek ve ince toplayıcı (K3).** `avasthas(&Positions, ghatis)` ephemeris görmez; `compute_avasthas(eph, &ChartInput)` temel haritayı (`compute_chart_for`) hesaplar, doğumdan önceki son gün doğumundan (`chart.panchang.sunrise_jd_ut`) tamamlanmış ghati sayısını bulur ve saf hesaba verir. Var olan hiçbir hesabın davranışı değişmez: `dignity`, `drishti`, `nature`, `positions`, `panchang` ve `chart` yalnızca okunur.
- **Modül `Chart`'a eklenmez (K1).** Rapor istek üzerine hesaplanır ve saklanmaz; veritabanı şeması değişmez (Y6).
- **Tek okuma, `method` kimliğiyle (Y5).** Lajjitadi kural metni ve Shayanadi formülü birden çok okumaya açıktır; her yerde tek okuma seçilir (Kararlar D3–D9), `AVASTHA_METHOD = "avastha.bphs_v1"` kimliğine bağlanır ve `tests/golden/modules/avastha/REVIEW.md`'de varyantlarıyla yazılır. Kurallar bellekten aktarılmıştır: sahibin basılı kaynak onayına dek panel "doğrulanmamış tablo" notunu taşır (Y4).
- **Lagna yoksa lagnaya bağlı çıktı yoktur (K2).** Doğum saati bilinmiyorsa Lajjita (5. ev ister) değerlendirilmez ve Shayanadi (lagna ve ghati ister) hesaplanmaz; Baladi, Jagradadi ve Lajjitadi'nin öteki beş kuralı döner ve Ay'ın satırı yaklaşık sayılır. Gün doğumu olmayan yerde (kutup) yalnızca Shayanadi boştur.
- **Arayüz hesap yapmaz (C4).** Panel Rust'tan gelen kimlikleri adlarına çevirir ve yazar; Shayanadi'nin toplamı ve artanı da Rust alanıdır.
- **Paralel planlarla çatışmayı azaltan yerleşim.** Plan 23 ve Plan 25 başka çalışma ağaçlarında aynı anda yürür. Bu plan yeni kodu yeni dosyalara koyar (çekirdekte `avastha/`, uygulamada `avastha_commands.rs` ve `tests/avastha.rs`, el örneklerinde `tests/avastha_golden.rs`, arayüzde kendi panel, test ve e2e dosyaları) ve ortak dosyalara yalnızca küçük, ekleme türünde dokunur; bunların tam listesi "Paylaşılan dosyalar" bölümündedir.
- **Testler:** çekirdekte her dizi için birim testleri (sınırlar, her kuralın tutan ve tutmayan durumu, küme sırası, formülün ikinci yazımı, toplayıcının sahte ephemeris testleri); el örnekleri (yedi Lajjitadi ve altı Shayanadi dosyası; kapsam testleri); uygulamada gerçek motorla çapraz denetim, değişmezler, hata eşlemesi ve bütçe; arayüzde Vitest (panel, sütun, adlar) ve iki uçtan uca akış.

**Tech Stack:** Rust 1.93+ (stable; yerelde ve CI'da 1.100), Tauri 2.12, tauri-specta 2.0.0-rc.29, serde, React 19, TypeScript 7, Vitest 5, Playwright 1.65.1. **Yeni Rust ya da npm bağımlılığı yok.**

**Spec:** `docs/superpowers/specs/2026-11-27-eksikler-yol-haritasi.md`: §1 (değişmeyen kurallar), §2 (Y1–Y8), §3 (avasthalar satırı), §4.4, §5 "Plan 24", §7, §8, Ek A. Taşınan kurallar: `2026-11-26-alt-proje-3-ileri-moduller-design.md` §2 (K1–K12), §4 (`ChartInput`, `Positions`, `nature`, bakışlar), §13 (ortak test stratejisi), §14 (gizlilik), §15 (bütçeler), Ek A. UI spec: `2026-11-24-ui-design.md` §3.3–§3.4 (paneller, ortak vurgu), §5 (yükleme evreleri), §8 (panel durumları).

**Ön koşul:** Taban, yol haritasının yazıldığı uçtur. Bu plan yazılırken `main` commit `7270550`'deydi; kod blokları ve sayaçlar o ağaca göre yazılmış ve ölçülmüştür. `main` bu commit'ten ilerlediyse Task 1 Step 1 tabanları yeniden ölçer ve sayaç zinciri o ölçümden yürütülür (farklar aynıdır); bir `diff` bloğunun bağlamı tutmazsa önce o satırlara dokunan commit okunur. Bu plan teslim edilirken `main` `e1269ae`'ye ilerlemişti: aradaki tek commit Plan 25'in plan belgesidir (`docs/` altında tek dosya); kod, `diff` bağlamları ve sayaçlar değişmez. `BASE` her zaman Task 1 Step 1'de okunan commit'tir. Plan 23 ve Plan 25 bu planın ön koşulu değildir ve bu planı beklemez.

## Kararlar

Sahibi kararları devretti; muhafazakâr seçenek seçildi ve her karar bir satır gerekçeyle yazıldı. "Yol haritası boşluğu" ya da "Yol haritası farkı" diye işaretlenenler yol haritasında düzeltilmesi gereken yerlerdir ve belgenin sonundaki "Spec'e geri bildirim" bölümünde aynı numarayla toplanmıştır. D13'teki ölçüm ve sayaç zinciri bu plan yazılırken deponun bir kopyasında, her görev ayrı commit olarak uygulanarak alınmıştır (macOS 27.2, Rust 1.100, hata ayıklama derlemesi).

- **D1 — Çekirdek modülü `crates/jyotish-core/src/avastha/`'dır: `mod.rs` (kayıt, yöntem kimliği), `baladi.rs` (Baladi ve Jagradadi), `lajjitadi.rs`, `shayanadi.rs`, `collect.rs` (rapor ve toplayıcı). Yöntem kimliği `AVASTHA_METHOD = "avastha.bphs_v1"`'dir. Adlar `jyotish_core` kökünden yeniden dışa aktarılmaz; tüketiciler `jyotish_core::avastha::…` yolunu kullanır.** Gerekçe: yol haritası modülü `avastha` diye adlandırır; K7 her sonucun yöntem kimliği taşımasını ister. Kökte yeniden dışa aktarım `lib.rs`'in sonundaki uzun `pub use` bloğuna satır ekler ve paralel planlarla çatışır; `muhurta` modülünün bir bölümü de alt yoldan kullanılır.
- **D2 — Avastha yedi grahaya atanır (`AVASTHA_GRAHAS`: Güneş … Satürn). Rahu ve Ketu'ya hiçbir dizide durum atanmaz; Lajjita'da eşlikçi olarak sayılırlar.** Gerekçe: v1 düğümlere dignity atamaz, doğal ilişki tablosu düğümleri taşımaz ve Shayanadi formülünün graha sayıları yedi graha için kesindir (düğümlere 8 ve 9 veren okuma REVIEW'de varyanttır).
- **D3 — Baladi: burç altışar derecelik beş dilimdir; tek burçlarda sıra Bala, Kumara, Yuva, Vriddha, Mrita, çift burçlarda terstir. Dilim sınırı sonraki dilime aittir. Derece rasi çerçevesindeki burç içi derecedir.** Gerekçe: yol haritası §4.4; tablo riski taşımayan tek dizidir.
- **D4 — Jagradadi eklenir. Graha yücelme burcunda ya da kendi burcundaysa Jagrat, düşüş burcundaysa Sushupti; öteki burçlarda burç lordu doğal dostu ya da nötrüyse Swapna, doğal düşmanıysa Sushupti'dir.** Gerekçe: yol haritası "Baladi kadar ucuzdur ve aynı plana eklenebilir" der ve kararı sahibine bırakır; maliyeti bir fonksiyon ve bir sütundur. Doğal ilişki seçilir, çünkü bu modülün bütün dostluk okuması doğaldır (D5) ve sonuç başka grahaların yerine bağlı olmaz. Graha tablosunun "Durum" (dignity) sütunu bileşik ilişkiyi gösterir ve Jagradadi ile ayrışabilir; fark REVIEW'de 2. noktadır. **Yol haritası boşluğu 1.**
- **D5 — Lajjitadi'nin ortak tanımları: birliktelik aynı rasi burcunda bulunmaktır; bakış burçtan burca tam graha bakışıdır (`graha_aspect_signs`; düğümler bakmaz); dost ve düşman doğal (naisargika) ilişkidir ve durumu sorulan grahanın gözünden okunur (`natural_relation(graha, öteki)`); iyicil ve kötücül `nature` sınıfıdır (K6).** Gerekçe: yol haritası üç noktayı da açık bırakır. Bileşik ilişkide aynı burçtaki graha her zaman geçici düşmandır (1. ev geçici dost evlerinden değildir); bileşik okuma her nötr eşlikçiyi düşmana çevirir ve "düşmanla birlikte" kuralını anlamsızlaştırırdı. Burç bakışı uygulamanın yoga ve prashna modülleriyle aynı bakıştır. **Yol haritası boşluğu 2.**
- **D6 — Altı kural (kol birleştirici "ya da"dır; yalnızca Lajjita, Trushita ve Kshobhita'da "ve" vardır): (1) Lajjita: lagnadan 5. burç evinde **ve** Güneş, Mars, Satürn, Rahu ya da Ketu ile birlikte. (2) Garvita: yücelme burcunda (burcun tamamı) ya da moolatrikona aralığında. (3) Kshudhita: düşmanının burcunda, ya da bir düşmanıyla birlikte, ya da bir düşmanının bakışı altında, ya da Satürn ile birlikte. (4) Trushita: su burcunda (Yengeç, Akrep, Balık) **ve** bir düşmanının bakışı altında **ve** ona bakan iyicil yok. (5) Mudita: dostunun burcunda, ya da bir dostuyla birlikte, ya da bir dostunun bakışı altında, ya da Jüpiter ile birlikte. (6) Kshobhita: Güneş ile birlikte **ve** bir kötücülün ya da bir düşmanının bakışı altında.** Gerekçe: BPHS'nin avasthalar bölümünün yaygın okuması; alıntı yapılmaz. Her kolun bilinen öteki okuması REVIEW'de numaralıdır (4–11) ve sahibin onayını bekler.
- **D7 — Lajjitadi sonucu kümedir: `Vec<LajjitadiHit>`, kural sırasında, boş olabilir. `LajjitadiHit { state, grahas }`: `grahas` kuralı sağlayan öteki grahalardır (birlikte olan ya da bakan), `Graha::ALL` sırasında ve tekrarsız; kural yalnızca burçtan sağlanıyorsa boştur. Öncelik sırasıyla tek durum seçilmez.** Gerekçe: yol haritası "bir graha aynı anda birden çok durumda olabilir: sonuç kümedir" der. Kuralı sağlayan grahalar yorum değil olgudur ve astrologun kuralı elle sağlamasını sağlar.
- **D8 — Shayanadi: `toplam = N × G × A + J + Gh + L`; durum toplamın 12'ye bölümünden artanın gösterdiği durumdur ve artan 0 on ikinci durumdur. `N` grahanın nakshatrası (1–27); `G` grahanın sayısı (Güneş 1, Ay 2, Mars 3, Merkür 4, Jüpiter 5, Venüs 6, Satürn 7); `A` grahanın navamsa diliminin burç içindeki sırası (1–9); `J` doğum nakshatrası (Ay); `Gh` doğumdan önceki son gün doğumundan doğuma tamamlanmış ghati (bir ghati 24 dakika: geçen gün kesri × 60, aşağı yuvarlanır); `L` lagna burcunun sırası (Koç 1). Sıra: Shayana, Upaveshana, Netrapani, Prakashana, Gamana, Agamana, Sabha, Agama, Bhojana, Nritya Lipsa, Kautuka, Nidra. `ShayanadiHit` durumla birlikte `nakshatra`, `graha_number`, `navamsa`, `total` ve `remainder` taşır; `ShayanadiBasis` üç ortak terimi taşır.** Gerekçe: yol haritası §4.4'ün saydığı dört risk noktasının (graha sayıları, ghati sayımı, artanın sıfır olduğu durum, on iki adın sırası) her biri için tek okuma seçildi ve REVIEW'de 13–18. noktalar olarak yazıldı. Terimler raporda durur, çünkü sahibin basılı kaynakla denetimi (ve astrologun elle sağlaması) ara değerleri ister; artan Rust alanıdır, çünkü arayüz bölme yapmaz (C4). Üç alt durum (cheshta) ve Deeptadi kapsam dışıdır.
- **D9 — Çerçeve (K5): burç, burç içi derece, ev ve lagna burcu rasi çerçevesindedir (`settings.zodiac`'ı izler); nakshatra her kipte sidereal boylamdandır. Tropikal-rasi kipinde Baladi dilimi, navamsa dilimi ve lagna terimi tropikal çerçeveden, `N` ve `J` sidereal boylamdan okunur.** Gerekçe: uygulamanın bütün modüllerindeki ayrım; tek bir çerçeveye zorlamak haritanın gösterdiği burçla çelişirdi. REVIEW'de 18. nokta.
- **D10 — Rapor her zaman döner (`AvasthaReport`, `Option` değil) ve `time_known` taşır. Doğum saati bilinmiyorsa (`input.time_known == false`) lagna boştur: Lajjita değerlendirilmez, `shayanadi_basis` ve her grahanın `shayanadi` alanı `None`'dır. Doğum yerinde o gün gün doğumu yoksa (`sunrise_jd_ut == None`) lagna vardır ve yalnızca Shayanadi boştur. Sayı prashnasında lagna sayıdan gelir (`compute_chart_for`) ve formülün lagna terimi o burçtur. Temel haritanın `LowPrecision` uyarısı rapora taşınır.** Gerekçe: K2 (arayüzün gizlemesine güvenilmez); Baladi ve Jagradadi lagnasız da anlamlıdır, raporun tümünü boşaltmak gereksiz olurdu. Shadbala sayı prashnasında meridyen istediği için rapor vermez; Shayanadi yalnızca lagna burcunu ister.
- **D11 — Yeni hata türü yoktur. `CoreError` ve `CommandError` kodları değişmez; `compute_avasthas` yalnızca `compute_chart`'ın hatalarını (konum, an, ayar sürümü, sayı prashnasının sayısı) taşır.** Gerekçe: komutun kendine ait girdisi yoktur; `commands.rs`, `error.rs` ve `i18n.test.ts`'in hata listesi ortak dosyalardır ve dokunulmaz.
- **D12 — El örnekleri `tests/golden/modules/avastha/lajjitadi/` (yedi dosya) ve `tests/golden/modules/avastha/shayanadi/` (altı dosya) altındadır ve ayrı bir test dosyasıyla yüklenir: `crates/jyotish-core/tests/avastha_golden.rs`. `modules_golden.rs`'e dokunulmaz. Lajjitadi örnekleri yedi grahanın Baladi, Jagradadi ve Lajjitadi durumlarını ve örneğin "oluşmayan durum" olarak gösterdiği çiftleri (`absent`) yazar; Shayanadi örneklerinin girdisi konumlar ve gün doğumundan geçen dakikadır. İkinci yazımlar: Baladi'de ayna (çift burçtaki basamak, tek burçta 30° − derecenin basamağıdır), Shayanadi'de bölmesiz yürüme (durum çemberinde adım adım), örneklerde örneğin kendi aritmetiğinin motordan bağımsız sağlaması.** Gerekçe: yol haritasının (3) ve (5) görevleri; `modules_golden.rs` Plan 23 ve 25'in de ekleme yapacağı ortak dosyadır. Beklenen değerler bu plan yazılırken motorun kaynağından ayrı yazılmış bir türetmeyle kural metninden üretilmiş ve motorla karşılaştırılmıştır (K9: başka yazılımın çıktısı yoktur).
- **D13 — Komut `compute_avasthas(input: ChartInput) -> AvasthaReport`'tur ve yeni bir dosyadadır: `app/src-tauri/src/avastha_commands.rs`. Komut listesine `commands::print_window`'dan hemen sonra eklenir. Gerçek motorla entegrasyon testi de ayrı dosyadır: `app/src-tauri/tests/avastha.rs`. Bütçe: temel harita dahil ortanca ≤ 10 ms (ölçüm: 0,6 ms) ve testte `assert!` ile doğrulanır.** Gerekçe: `module_commands.rs`, `tests/modules.rs` ve komut listesinin `module_commands::compute_compat` satırının altı paralel planların doğal ekleme yeridir; ayrı dosya ve ayrı satır çatışmayı önler. Her entegrasyon testi dosyası ayrı süreçtir, dolayısıyla "süreçte tek motor" kuralı bozulmaz.
- **D14 — Panel `avastha` türüdür (`MODULE_PANELS`: 7 × 16; paletten eklenir ve çıkarılır, K12) ve hazır düzenlere eklenmez. İki tablo çizer: yedi grahanın dört dizisi; altında (Shayanadi hesaplandıysa) terim tablosu, ortak terimler ve formül satırı. Panelin ayarı yoktur. Katalogda `yogas`'tan hemen sonra durur.** Gerekçe: yol haritası "ortak panel" der. Terim tablosu sahibin denetimi ve D8 içindir. Katalog sırasındaki yer, paralel planların listenin sonuna yapacağı eklemeyle aynı satırlara düşmemek içindir.
- **D15 — Durum adları Sanskritçe transliterasyondur ve dile göre değişmez (K11): `app/ui/panels/avasthaLabels.ts`'te sabit tablolarda durur, i18n kaynağında değil; `NrityaLipsa` kimliğinin adı "Nritya Lipsa"dır. Dizi adları (Baladi, Jagradadi, Lajjitadi, Shayanadi) i18n'dedir ve üç dilde aynıdır; bir test tabloları fikstürdeki Rust kimlikleriyle, bir başkası üç dilin eşitliğini karşılaştırır.** Gerekçe: nakshatra ve tithi adlarının kalıbı (`labels.ts`); otuz altı satırlık üç kopya yerine tek tablo. Ortak dosya `labels.ts`'e eklenmez.
- **D16 — Panel her zaman "Doğrulanmamış tablo: Lajjitadi kuralları ve Shayanadi formülü basılı kaynakla karşılaştırılmayı bekliyor." notunu taşır (`avastha.unverified`). Not, sahibi REVIEW'deki onay tablosunu doldurduktan sonra ayrı bir commit'le çıkarılır.** Gerekçe: Y4.
- **D17 — Doğum saati bilinmiyorsa panelde Ay'ın satırı (Baladi, Jagradadi ve doluysa Lajjitadi hücresi) "≈" ve "yaklaşık" metniyle işaretlenir; üstte tek bir açıklama satırı durur. Öteki grahaların hücreleri işaretlenmez.** Gerekçe: Ay günde ~13° yürür ve basamağı ya da burcu gün içinde değişebilir; öteki grahaların günlük yolu bir Baladi diliminden küçüktür. Graha tablosunun ve Jaimini panelinin kalıbı.
- **D18 — Graha tablosunun Baladi sütunu isteğe bağlıdır ve `settings.baladi` ile açılır (Karaka sütununun kalıbı): `settings.columns` listesine girmez, açık değilken komut çağrılmaz, Rahu, Ketu ve lagna satırında boştur, saat bilinmiyorsa Ay'ın hücresi "≈" taşır. Baskı tablosu `columns` listesini kullanır ve değişmez (K10).** Gerekçe: yol haritasının (7) görevi; `Chart` dondurulmuştur (K1), sütun değerini modül komutundan okur.
- **D19 — Arayüz fikstürü üç alan kazanır: `avasthas` (açık haritanın girdisiyle), `avasthas_time_unknown` (`time_known: false` ile) ve `avastha_ids` (Rust'ın `Baladi::ALL`, `Jagradadi::ALL`, `Lajjitadi::ALL`, `Shayanadi::ALL` listeleri). E2e taklidi iki raporu `time_known`'a göre döner.** Gerekçe: arayüz hesap yapmaz; testler gerçek motorun ürettiği veriyi çizer.
- **D20 — Uçtan uca akışlar yeni bir dosyadadır: `app/e2e/avastha.desk.spec.ts`. Playwright'ın "e2e" projesi adı `desk.spec.ts` ile biten her dosyayı koşar; yapılandırma ve `desk.spec.ts` değişmez.** Gerekçe: `desk.spec.ts` paralel planların sonuna test ekleyeceği ortak dosyadır.
- **D21 — Görev sırası yol haritasının taslağından iki yerde ayrılır: panelin i18n metinleri ve "saat bilinmiyor" durumu panelle birlikte Task 6'dadır (panel metinsiz ve durumsuz commit edilemez); Task 7 graha tablosunun Baladi sütunu ile uçtan uca akışlardır. REVIEW.md Task 8'de, kapılarla birlikte yazılır.** Gerekçe: her görev yeşil kapılarla tek commit'tir. **Yol haritası farkı 3.**
- **D22 — Adlandırma: bu planın getirdiği adların hiçbiri yasak diziyi içermez (Baladi, Jagradadi, Lajjitadi, Shayanadi ve otuz altı durum adı tarandı). Shayanadi'nin kapsam dışı üç alt durumunun adı koda ve metinlere girmez. Türkçe metinlerde bölümden artan için "artan" yazılır; dizi şu sözcüklerle de oluşur ve yerlerine ötekiler yazılır: "… durur", "geri", "öteki", "sürer", "çıkarılır", "yerinde durur". `karaka` sözcüğüne `-lar` eki bitişik yazılmaz.** Gerekçe: Y8, v1 spec §10 ve Ek A; yasak harf dizisi düzeyindedir.
- **D23 — Yeni görsel temel (PNG) ve yeni baskı düzeni yoktur (K10). `pnpm build` Vite'ın 500 kB parça uyarısını basmayı sürdürür ve kapılar `✓ built` satırını arar.** Gerekçe: on görsel temel Avastha panelini taşımaz; graha tablosunun temeli sütun kapalıyken değişmez.

## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni bağımlılık yok: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `app/package.json`, `app/pnpm-lock.yaml` ve `app/vite.config.ts` değişmez; `cargo deny check` Task 8'de `advisories ok, bans ok, licenses ok, sources ok` verir.
- **Yasaklı ad kuralı (v1 spec §10 "İsim kuralları", ilk madde; ileri modüller spec'i Ek A; yol haritası Y8):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Kural harf dizisi düzeyindedir: Türkçe sözcüklerin içinde de geçemez (D22). Her commit'ten önce eklenen satırlar şu komutla taranır ve çıktı boş olmalıdır: `git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'`.
- **Clean-room ve yorum metni yok:** kurallar klasik tanımdan özgün olarak uygulanır; başka ürünün adı, düzeni, metni ya da çıktısı kullanılmaz; kaynak gösterimi geneldir ve alıntı yapılmaz. Modül olgu döner: **"iyi/kötü/güçlü/zayıf" hükmü, puan, renkli derecelendirme ve bir durumun ne anlama geldiğini anlatan metin eklenmez.** Durum adları çevrilmez ve açıklanmaz.
- **Doğrulanmamış tablo (Y4):** Lajjitadi kuralları ve Shayanadi formülü bellekten aktarılmıştır. Yürüten ajan bunları başka bir yazılımın çıktısıyla ya da çevrimiçi bir kaynakla **karşılaştırmaz** ve bu planın okumasını "düzeltmez"; şüphe duyduğu yeri ilerleme notlarına yazar. Onay sahibinindir (Task 8).
- **Çerçeveler (K4, K5; D9):** evler lagnadan sayılan burç evleridir ve ev sistemi ayarından bağımsızdır; burçlar rasi çerçevesindedir; nakshatra sidereal boylamdandır.
- **Doğum saati bilinmiyorsa (K2, D10):** `time_known` yanlışsa lagna hesaba girmez; Lajjita ve Shayanadi Rust'ta boş döner; arayüzün gizlemesine güvenilmez.
- **Gizlilik (ileri modüller spec'i §14):** (1) Girdi (an, konum, ayar) ve rapor hiçbir yerde loglanmaz. (2) Yeni hata kodu yoktur; var olan hatalar değişmez. (3) Rapor saklanmaz ve dışa aktarıma girmez. (4) Hiçbir modül ağ isteği yapmaz. (5) Testler ve fikstürler yalnızca deneme adları ve sentetik konumlar kullanır ("Örnek Danışan"); gerçek kişi verisi yoktur.
- **Bütçe testte doğrulanır** (D13): `assert!` ile; `eprintln!` yalnızca ölçümü görünür kılar.
- **El örnekleri (§13, K9):** beklenen değer kuraldan türetilir ve `derivation` alanında ya da test yorumunda yazılıdır. Bir el örneği motorla uyuşmazsa önce kural yeniden okunur; **beklenen değer motor çıktısına bakarak düzeltilmez.** (Bu plan yazılırken iki kez oldu: bir Trushita testinde Satürn'ün Boğa'dan Yengeç'e 3. bakışı, bir Kshudhita testinde Mars'ın Kova'dan Boğa'ya 4. bakışı türetmede atlanmıştı; kural yeniden okundu ve senaryo ile beklenti türetmeden düzeltildi.)
- Kapılar: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (1.100), `cargo test --workspace`, `pnpm --dir app test`, `pnpm --dir app build` her görev sonunda yeşil; `pnpm --dir app e2e` Task 7 ve 8'de; `cargo deny check` ve `pnpm --dir app e2e:visual` Task 8'de. `tsc` `strict`. `pnpm build` çıktısında aranan satır `✓ built`'tir (D23). Tek bir Vitest dosyası `pnpm --dir app exec vitest run ui/…` ile koşulur: yol `app/` dizinine göredir. `cargo test --workspace` çıktısı uzun olduğu için başarısızlık şu sayaçla denetlenir ve `0` yazmalıdır: `cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."`.
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez; her yeni kabukta yeniden). Node 24+, pnpm 11.8.0. Depo ya da çalışma ağacı kısa bir yolda durmalıdır: Swiss Ephemeris 242 bayttan uzun ephemeris yolunu reddeder.
- **Çalışma ağacı.** Yeni bir git çalışma ağacında `app/node_modules` ve `target/` yoktur: Task 1 Step 1 bağımlılıkları kilit dosyasından kurar (`pnpm --dir app install --frozen-lockfile`; kilit dosyası değişmez) ve ilk `cargo test` her şeyi derler (birkaç dakika). Bütün komutlar çalışma ağacının kökünden verilir.
- **Üretilen dosyalar (C10) elle düzenlenmez** ve yalnızca şu iki komutla güncellenir: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings` (`app/ui/bindings.ts`) ve `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture` (`app/ui/test/fixtures/delhi.json`). İkisi de Task 6 Step 4'te koşulur.
- **Gerçek uygulama çalıştırılmaz.** Yürüten ajan `pnpm tauri dev` ya da `pnpm --dir app tauri dev` **koşmaz**, üretilen `.app`'i açmaz ve `~/Library/Application Support/io.github.turgaydogusan.jyotishdesk` dizinine dokunmaz. Kapı testlerdir (Y3); gerçek uygulamaya bakan maddeler Task 8'de **[insan]** diye işaretlidir ve sahibine devredilir. Başkasının çalışan `tauri dev`'i (port 1420) durdurulmaz; `pkill -f` kullanılmaz.
- **Port 1421 ve paralel çalışma ağaçları.** E2e, Vite sunucusunu 1421'de açar ve açık bir sunucu varsa onu kullanır. Plan 23 ve 25 aynı makinede koşuyorsa 1421 başka bir çalışma ağacının sunucusu olabilir ve testler yanlış arayüzü görür. Her e2e koşusundan önce `lsof -i :1421` bakılır: çıktı varsa öteki koşunun bitmesi beklenir (bir-iki dakika; sunucu öldürülmez) ve yeniden bakılır.
- Yalnızca macOS hedeflenir. Yorum metni ve AI paneli bu planın dışındadır (Y7).
- **Tabanlar (`7270550`):** `jyotish-core` lib 404, golden 8, `modules_golden` 17; `atlas` `build` 6, `moment_fixed_lmt` 11, `moment_iana` 13, `search` 8, `shipped` 3; `ephemeris` lib 1, `engine` 7, `env_override` 1, `ffi_smoke` 1, `golden` 1, `houses` 7, `rise_set` 5, `sanity` 15; `store` lib 122 (+1 `#[ignore]`); `jyotish-desk` lib 58, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `Test Files  88 passed (88)`, `Tests  887 passed (887)`; e2e 27; görsel 10 (yerel, darwin); yayın betiği testleri 14.
- **Sayaç zinciri** (her satır görev bitince; boş hücre "önceki satırla aynı"). Bu plan yazılırken `7270550`'nin bir kopyasında her görev ayrı commit olarak uygulanıp ölçülmüştür.

  | Görev | `jyotish-core` lib | `avastha_golden` | `jyotish-desk` `avastha` | Vitest dosya / test | e2e |
  |---|---|---|---|---|---|
  | Taban | 404 | yok | yok | 88 / 887 | 27 |
  | Task 1 | 408 | | | | |
  | Task 2 | 417 | | | | |
  | Task 3 | | 3 | | | |
  | Task 4 | 429 | | | | |
  | Task 5 | | 5 | | | |
  | Task 6 | | | 1 | 89 / 901 | |
  | Task 7 | | | | 90 / 907 | 29 |
  | Task 8 | | | | | |

  Değişmeyenler: `jyotish-core` golden 8, `modules_golden` 17; `atlas`, `ephemeris` ve `store` sayaçları; `jyotish-desk` lib 58, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; görsel 10; yayın betiği testleri 14. `jyotish-core` lib Task 1'de 4 test (`avastha/baladi.rs`), Task 2'de 9 test (`avastha/lajjitadi.rs`), Task 4'te 12 test (`avastha/shayanadi.rs` 7, `avastha/collect.rs` 5) kazanır. `avastha_golden` Task 3'te 3, Task 5'te 2 test kazanır. Vitest Task 6'da bir dosya ve 14 test (`panels/AvasthaPanel.test.tsx` 13; `api.test.ts`'in tablosunda bir satır), Task 7'de bir dosya ve 6 test (`panels/GrahaTableBaladi.test.tsx`) kazanır. E2e Task 7'de iki akış kazanır (`avastha.desk.spec.ts`). Paralel bir plan önce birleştiyse tabanlar onun eklediği kadar büyüktür; farklar aynıdır.
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir; bu planda değişen okumalar açıkça yazılıdır:**
  - **C1** Gizlilik: yukarıdaki "Gizlilik" maddesi; `no-console.test.ts` durur ve izin listesi büyümez. Yeni `CommandError` kodu yoktur (D11).
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu ya da adres almaz. `compute_avasthas` dizgi bağımsız değişken almaz; `api.test.ts`'in dizgi bağımsız değişken listesi değişmez, yalnızca sarmalayıcı tablosu bir satır kazanır.
  - **C3** Var olan göçler ve şema sürümleri değişmez; `crates/store` bu planda hiç değişmez (Y6). `CalcSettings`, `ViewState` ve `AppPrefs` tipleri değişmez.
  - **C4** Arayüz astrolojik hesap yapmaz: basamak, durum, kuralı sağlayan grahalar, formülün terimleri, toplamı ve artanı Rust'tan gelir. Kimliği adına çevirmek, bir listenin boş olup olmadığına bakmak, nakshatra numarasından adını ve burç sırasından kısaltmasını okumak gösterimdir.
  - **C5/C17** Otomatik kayıtlar değişmez; graha tablosunun `baladi` ayarı düzenle birlikte var olan yoldan kaydedilir. **C6** Düzen ayrıştırıcıları (`parseLayoutState`, Rust `check_panel_layout`) değişmez; bir panel türü kataloğa eklenir.
  - **C7, C16** Kısayol eklenmez; `shortcuts.ts` değişmez. **C8** Yeni pencere yoktur.
  - **C9** Renk yalnızca token'lardan gelir (hex yok) ve tek başına anlam taşımaz: yaklaşık değer "≈" ve sözle, boş değer çizgi ve sözle ("hiçbiri", "hesaplanmadı") yazılır.
  - **C10** Üretilen dosyalar elle düzenlenmez (yukarıda). **C11** Görsel PNG temelleri yereldir; bu planda değişmeleri beklenmez (D23).
  - **C12** Her yeni i18n anahtarı üç dilde vardır (`i18n.test.ts` anahtar kümelerini ve yer tutucuları karşılaştırır). Fransızca metinler tipografik kesme işareti (’) kullanır; panel testi `avastha` bloğunda düz kesme işaretinin yokluğunu sabitler.
  - **C13** Yasaklı ad taraması her commit'te; depo dışındaki ilerleme notları ve `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** Plan 07'nin gün doğumu toleransı ve `compute_panchang` değişmez (`panchang.rs`'e dokunulmaz): ghati, haritanın zaten hesapladığı gün doğumundan okunur. **C15** `alert`, `confirm`, `prompt`, `window.print()` ve `window.open()` çağrılmaz.
- **Erişilebilirlik:** WCAG AA. Tablolar `caption`, `scope="col"` ve `scope="row"` taşır; kaydırma kabı klavye odağı alır ve tablosunun adını taşır; Lajjitadi hücresindeki durumlar gerçek bir listedir (`ul`/`li`); boş hücre ekran okuyucuya sözcükle okunur; hata `role="alert"`'tir ve "Yeniden dene"den sonra odak gövdeye düşmez; her işlem klavyeyle yapılır.
- Commit mesajları İngilizcedir ve tam olarak şu satırla biter: `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` (komutlarda ikinci `-m` olarak verilir; yürüten araç kendi imza satırını eklemez). Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, `git tag`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Kod blokları `7270550` ağacına göre yazılmış ve bu plan yazılırken deponun bir kopyasında koşulmuştur (Self-Review madde 5). Yeni dosyalar tam içerikle verilir ("**Yeni dosya:**"). Değişen dosyalar `diff` bloğuyla verilir: blok bir dosyaya konup çalışma ağacının kökünde `git apply <dosya>` ile ya da elle uygulanır; bağlam tutmazsa önceki görevin adımı denetlenir, elle uydurma yapılmaz. Yeni bir Rust dosyasının testleri ve uygulaması ayrı adımlarda verilir: önce dosya yalnızca `#[cfg(test)] mod tests` bloğuyla oluşturulur, sonra uygulama **o bloğun üstüne**, arada bir boş satırla eklenir; `cargo fmt --all --check` birleştirmeyi doğrular. Kod blokları dört ters tırnakla çevrilidir; çevreleyen satırlar dosyaya yazılmaz.

## Paylaşılan dosyalar

Plan 23 (panchang ekleri, navatara, Vimshopaka) ve Plan 25 (bhava chalit, Sudarshana) başka çalışma ağaçlarında aynı anda yürür. Aşağıdaki liste bu planın dokunduğu ve öteki planların da dokunması beklenen **bütün** dosyalardır; her satır değişikliğin boyutunu, yerini ve birleştirme çatışmasında ne yapılacağını söyler. Bu listenin dışındaki her dosya ya yenidir ya da yalnızca bu plana aittir.

| Dosya | Görev | Değişiklik | Çatışmada |
|---|---|---|---|
| `crates/jyotish-core/src/lib.rs` | 1 | Tek satır: `pub mod avastha;` (`ashtakavarga`'dan sonra, abecesel yerinde). `pub use` bloğuna dokunulmaz. | İki tarafın satırları korunur. Plan 25 kendi bloğunu `pub mod ashtakavarga;` satırının önüne koyar; bu planın satırı o satırın arkasındadır. |
| `app/src-tauri/src/lib.rs` | 6 | İki satır: `pub mod avastha_commands;` (`atlas_commands`'tan sonra) ve komut listesinde `commands::print_window,`'dan sonra `avastha_commands::compute_avasthas,`. | `pub mod` satırı Plan 25'in `pub mod bhava_commands;` satırıyla aynı yere düşer: ikisi de korunur, abecesel sırayla (`avastha_commands`, sonra `bhava_commands`; `rustfmt` bunu ister). Komut satırları ayrı yerlerdedir. Sonra bağlamalar yeniden üretilir. |
| `app/src-tauri/tests/ui_fixture.rs` | 6 | İki `use` satırı; yoga raporlarından sonra on yedi satırlık hesap bloğu; `json!` içinde `"yoga_conditions"`'tan sonra üç anahtar (sekiz satır). | İki tarafın blokları ve anahtarları korunur. Sonra fikstür yeniden üretilir. |
| `app/ui/bindings.ts` | 6 | Üretilir (79 satır ekleme). | Elle çözülmez: bir taraf alınır, sonra `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings`. |
| `app/ui/test/fixtures/delhi.json` | 6 | Üretilir (tek satırlık dosya). | Elle çözülmez: bir taraf alınır, sonra `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture`. |
| `app/ui/test/fixture.ts` | 6 | Tip içe aktarımına beş ad (abecesel yerlerinde); `UiFixture`'a `yoga_conditions`'tan sonra üç alan. | İki tarafın adları ve alanları korunur. İçe aktarım listesinin başı (`AvasthaReport`, `Baladi`) Plan 25'in eklediği adlarla komşudur: liste abecesel sırayla birleştirilir. |
| `app/ui/api.ts` | 6 | `computeYogas`'tan sonra bir sarmalayıcı (üç satır). | İki tarafın sarmalayıcıları korunur. |
| `app/ui/api.test.ts` | 6 | İki satır: taklit listesinde ve sarmalayıcı tablosunda `computeAvasthas`. | İki tarafın satırları korunur. |
| `app/ui/layout/model.ts` | 6 | `PANEL_KINDS`'e `"yogas"`'tan sonra `"avastha"`; `MODULE_PANELS`'e `yogas`'tan sonra iki satır. | İki tarafın türleri korunur. |
| `app/ui/layout/model.test.ts` | 6 | Katalog listesine bir satır; **tek satırlık** `MODULE_PANELS` beklentisine `"avastha"` (`"yogas"`'tan sonra). | Tek satırlık beklenti kesin çatışır: sonuç, `model.ts`'teki `MODULE_PANELS` sırasının aynısıdır (iki tarafın türleri birlikte). |
| `app/ui/panels/registry.ts` | 6 | Bir içe aktarım (`AshtakavargaPanel`'den sonra) ve `yogas`'tan sonra iki satır. | İki tarafın satırları korunur. |
| `app/ui/i18n/tr.ts`, `en.ts`, `fr.ts` | 6, 7 | Üç ekleme: `panel` bloğunda `yogas`'tan sonra `avastha`; `compat` bloğundan önce yeni `avastha` bloğu (22 satır); `table` bloğunun sonunda `baladi` (Task 7). Var olan hiçbir satır değişmez. | İki tarafın anahtarları korunur; `i18n.test.ts` üç dilin eşitliğini denetler. |
| `app/ui/panels/GrahaTablePanel.tsx` | 7 | Baladi sütunu: iki içe aktarım, bir kanca çağrısı, bir onay kutusu, bir başlık ve bir hücre (Karaka sütununun yanına). | İki tarafın sütunları korunur; Baladi, Karaka'dan sonra durur. |
| `app/e2e/ipc.ts` | 7 | `compute_yogas` taklidinden sonra iki satır. | İki tarafın taklitleri korunur. |
| `tests/golden/modules/README.md` | 8 | Tabloya bir satır; son cümleden önce bir paragraf. | İki tarafın satır ve paragrafları korunur. |

Plan 25'in yazılı planıyla (`docs/superpowers/plans/2026-11-27-plan-25-bhava-sudarshana.md`, `e1269ae`) karşılaştırıldı: öteki bütün eklemeler ayrı çapa satırlarındadır (Plan 25 kataloğa `bhava_bala`'dan, bu plan `yogas`'tan sonra ekler; i18n'de Plan 25 `argala` ve `arudha` çevresine, bu plan `yogas` ve `compat` çevresine ekler). Metin çatışması beklenen yerler yalnızca şunlardır: `app/src-tauri/src/lib.rs`'in `pub mod` satırı, `fixture.ts`'in içe aktarım listesi, `model.test.ts`'in tek satırlık beklentisi ve üretilen iki dosya. Plan 23'ün planı bu belge yazılırken depoda değildi; aynı kurallar geçerlidir. Birleştirmeden sonra üretilen iki dosya iki komutla yeniden üretilir ve bütün kapılar yeniden koşulur (birleştirenin işidir, bu planın değil).

Bilerek dokunulmayan ortak dosyalar: `crates/jyotish-core/src/error.rs`, `crates/jyotish-core/tests/modules_golden.rs`, `app/src-tauri/src/module_commands.rs`, `app/src-tauri/src/commands.rs`, `app/src-tauri/tests/modules.rs`, `app/ui/panels/panels.test.tsx`, `app/ui/panels/panels.css`, `app/ui/panels/labels.ts`, `app/ui/panels/GrahaTablePanel.test.tsx`, `app/ui/panels/grahaColumns.ts`, `app/ui/i18n/i18n.test.ts`, `app/e2e/desk.spec.ts`, `app/playwright.config.ts`.

## Sonraki işlere verilen API

Bu planın ürettiği ve sonraki işlerin tüketeceği adlar. Rust adları `jyotish_core::avastha` yolundadır (D1); imzalar görevlerde tam olarak verilir.

| API | Tüketen | Not |
|---|---|---|
| `Baladi`, `Jagradadi`, `Lajjitadi`, `Shayanadi` (her biri `ALL` ile), `baladi`, `jagradadi`, `BALADI_SPAN_DEG` | harita araması (yeni koşul türü), AI paneli | Saf; konumdan |
| `lajjitadi`, `LajjitadiHit`, `conjunct_grahas`, `aspecting_grahas`, `is_garvita`, `LAJJITA_HOUSE`, `LAJJITA_COMPANIONS`, `WATER_SIGNS` | yorum katmanı, AI paneli | Küme sonucu; kuralı sağlayan grahalarla |
| `shayanadi`, `shayanadi_basis`, `shayanadi_total`, `shayanadi_state`, `graha_number`, `navamsa_number`, `elapsed_ghatis`, `ShayanadiBasis`, `ShayanadiHit`, `GHATIS_PER_DAY`, `NAVAMSA_SPAN_DEG` | AI paneli; formülü başka bir an için kullanacak sonraki modüller | Terimler raporda |
| `avasthas`, `compute_avasthas`, `AvasthaReport`, `GrahaAvastha`, `AVASTHA_METHOD`, `AVASTHA_GRAHAS` | yıllık harita ve prashna panelleri | `time_known` ve `method` taşır |
| `avastha_commands::{compute_avasthas, compute_avasthas_blocking}` | AI paneli | Yeni hata kodu yok |
| Arayüz: `modules/useAvasthas.ts`, `panels/avasthaLabels.ts`, `panels/AvasthaPanel.tsx`; fikstürde `avasthas`, `avasthas_time_unknown`, `avastha_ids` | sonraki paneller | i18n: `avastha.*`, `panel.avastha`, `table.baladi` |

## Dosya Yapısı (bu plan)

```
crates/jyotish-core/
  src/avastha/mod.rs              modül kaydı, AVASTHA_METHOD, AVASTHA_GRAHAS (yeni)
  src/avastha/baladi.rs           Baladi, Jagradadi (yeni)
  src/avastha/lajjitadi.rs        Lajjitadi, LajjitadiHit, altı kural (yeni)
  src/avastha/shayanadi.rs        Shayanadi, formül, ghati (yeni)
  src/avastha/collect.rs          AvasthaReport, avasthas, compute_avasthas (yeni)
  src/lib.rs                      pub mod avastha;
  tests/avastha_golden.rs         el örneklerinin testi (yeni)
tests/golden/modules/
  avastha/lajjitadi/*.json        yedi el örneği (yeni)
  avastha/shayanadi/*.json        altı el örneği (yeni)
  avastha/REVIEW.md               okuma kuralları, tablolar, onay bekleyen noktalar (yeni)
  README.md                       bir satır ve bir paragraf
app/src-tauri/
  src/avastha_commands.rs         compute_avasthas (yeni)
  src/lib.rs                      modül ve komut kaydı
  tests/avastha.rs                gerçek motorla çapraz denetim, hatalar, bütçe (yeni)
  tests/ui_fixture.rs             fikstüre avasthas, avasthas_time_unknown, avastha_ids
app/ui/
  bindings.ts                     üretilir
  test/fixtures/delhi.json        üretilir
  test/fixture.ts                 UiFixture'ın üç yeni alanı
  api.ts, api.test.ts             bir sarmalayıcı
  modules/useAvasthas.ts          rapor kancası (yeni)
  panels/avasthaLabels.ts         durum adları (yeni)
  panels/avastha.css              (yeni)
  panels/AvasthaPanel.tsx (+test) (yeni)
  panels/registry.ts
  layout/model.ts (+test)         avastha türü, MODULE_PANELS
  panels/GrahaTablePanel.tsx      Baladi sütunu
  panels/GrahaTableBaladi.test.tsx  (yeni)
  i18n/{tr,en,fr}.ts              panel.avastha, avastha, table.baladi
app/e2e/
  ipc.ts                          compute_avasthas taklidi
  avastha.desk.spec.ts            iki akış (yeni)
```

### Kapsam

| Yol haritası maddesi | Yer |
|---|---|
| §4.4 Baladi: beş basamak; tek burçta düz, çift burçta ters | Task 1 (D3) |
| §4.4 Jagradadi (üç durum; sahibin kararına bırakılmıştı) | Task 1 (D4) |
| §4.4 Lajjitadi: altı durum, küme sonucu | Task 2 (D5–D7) |
| §4.4 Lajjitadi'nin açık noktaları: düşmanlığın türü, bakışın türü, su burcu kümesi | Task 2 (D5, D6); REVIEW 4–12 |
| §5 (3) Lajjitadi el örnekleri: her durum için tutan ve tutmayan | Task 2 (birim testleri), Task 3 (`absent`) |
| §4.4 Shayanadi: formül, on iki durum; lagna ve saat ister | Task 4 (D8, D10) |
| §4.4 Shayanadi'nin açık noktaları: graha sayıları, ghati, artan sıfır, adların sırası | Task 4 (D8); REVIEW 13–18 |
| §5 (4) Toplayıcı: gün doğumu, ghati | Task 4 (`collect.rs`) |
| §5 (5) Shayanadi el örnekleri ve ikinci yazım | Task 4 (yürüme testi), Task 5 |
| §4.4 Saat bilinmiyorsa `None` (K2) | Task 4, Task 6 (D10, D17) |
| §5 (6) Komut ve Avastha paneli | Task 6 (D13, D14) |
| §5 (7) i18n, "saat bilinmiyor" durumu | Task 6 (D15–D17, D21) |
| §5 (7) Graha tablosuna isteğe bağlı Baladi sütunu | Task 7 (D18) |
| K12 Paletten ekleme; uçtan uca akış | Task 6, Task 7 (D20) |
| Y4 REVIEW kaydı, "doğrulanmamış tablo" notu | Task 6 (D16), Task 8 |
| §5 (8) Kapılar ve elle denetim; sahibin denetimi | Task 8 |
| §4.4 Kapsam dışı: Shayanadi alt durumları, Deeptadi | "Bu planın kapsamadıkları" |

---

### Task 1: Çekirdek: `avastha` modülü, Baladi ve Jagradadi

Yol haritası §4.4 ("Baladi", "Jagradadi") ve §5 Plan 24 görev (1); K5; D1–D4. Var olan hiçbir hesabın davranışı değişmez: `dignity.rs` yalnızca okunur (`exaltation_point`, `own_signs`, `sign_lord`, `natural_relation`).

**Files:**
- Create: `crates/jyotish-core/src/avastha/mod.rs`, `crates/jyotish-core/src/avastha/baladi.rs`
- Modify: `crates/jyotish-core/src/lib.rs` (tek satır; paylaşılan dosya)
- Test: `baladi.rs` `mod tests` (4)

**Interfaces:**
- Consumes (v1, Plan 14; hepsi var): `jyotish_core::dignity::{exaltation_point, natural_relation, own_signs, sign_lord, Relation}`; `drishti::ASPECTING_GRAHAS`; `Graha`; `zodiac::Sign`.
- Produces (`jyotish_core::avastha` yolunda):
  - `AVASTHA_METHOD: &str = "avastha.bphs_v1"`, `AVASTHA_GRAHAS: [Graha; 7]` (Güneş … Satürn).
  - `Baladi { Bala, Kumara, Yuva, Vriddha, Mrita }` (`Baladi::ALL`: tek burçtaki dilim sırası), `BALADI_SPAN_DEG: f64 = 6.0`, `baladi(sign: Sign, deg_in_sign: f64) -> Baladi`.
  - `Jagradadi { Jagrat, Swapna, Sushupti }` (`Jagradadi::ALL`), `jagradadi(graha: Graha, sign: Sign) -> Option<Jagradadi>` (düğümler `None`).

- [ ] **Step 1: Tabanları ölç**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
git log -1 --format=%h
git status --short
rustc --version
[ -d app/node_modules ] || pnpm --dir app install --frozen-lockfile
git status --short
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
lsof -i :1421
pnpm --dir app e2e 2>&1 | tail -1
rm -rf app/test-results
```
Expected: iki `git status --short` de boş (bağımlılık kurulumu kilit dosyasını değiştirmez; `app/node_modules` izlenmez); `rustc 1.100.0` ya da yenisi; `jyotish-core` lib `404 passed`, golden `8 passed`, `modules_golden` `17 passed`; `atlas` `build` 6, `moment_fixed_lmt` 11, `moment_iana` 13, `search` 8, `shipped` 3; `store` lib `122 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `58 passed`, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `88 passed (88)`, `887 passed (887)`; `lsof` çıktısız (çıktı varsa "Port 1421" kuralı: beklenir); e2e `27 passed`. `git log` çıktısı `7270550` değilse (`main` ilerlediyse ya da paralel bir plan birleştiyse) ölçülen değerler ilerleme notlarına yazılır ve sayaç zinciri o tabandan, aynı farklarla yürütülür. Okunan commit Task 8'de `BASE` olarak kullanılır: not edilir. Playwright WebKit'i bulamazsa kurulum komutunu kendisi söyler (`pnpm --dir app exec playwright install webkit`).

- [ ] **Step 2: Testler: Baladi sınırları ve ayna yazımı, Jagradadi; modül kaydı**

Önce modül kaydı (testlerin derlenebilmesi için), sonra test gövdesi. Testler: Baladi'nin on dilim sınırı tek ve çift burçta; ikinci yazım (çift burçtaki basamak, tek burçta 30° − derecenin basamağıdır) on iki burçta 300'er derecede; Jagradadi'nin el örnekleri (türetmeler yorumlarda) ve her grahanın üç durumu da aldığı değişmezi.

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 61db775..e78ad43 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -11,6 +11,7 @@
 //! Arama (Alt proje 3.9): `search` (harita araması sorgusu ve hafif yol, olay araması).
 
 pub mod ashtakavarga;
+pub mod avastha;
 mod chart;
 mod combustion;
 pub mod dasha;
````

**Yeni dosya:** `crates/jyotish-core/src/avastha/mod.rs`

````rust
//! Avasthalar (yol haritası §4.4, Plan 24): grahanın "durum" sınıfları. Baladi (burç içi
//! dereceden beş yaş basamağı), Jagradadi (burçtan üç uyanıklık durumu), Lajjitadi (yerleşim,
//! birliktelik ve bakıştan altı durum; sonuç kümedir) ve Shayanadi (nakshatra, graha sayısı,
//! navamsa, doğum nakshatrası, ghati ve lagnadan on iki durum).
//!
//! Kaynak: BPHS, avasthalar bölümü; kurallar özgün olarak uygulanır, alıntı yapılmaz. Okuma
//! seçimleri ve sahibin onayını bekleyen varyant noktaları
//! `tests/golden/modules/avastha/REVIEW.md`'dedir. Burçlar ve evler rasi çerçevesindedir
//! (K4, K5); nakshatra sidereal boylamdandır. Rahu ve Ketu'ya avastha atanmaz. Yorum ve hüküm
//! yoktur: modül yalnızca sınıfı ve kuralı sağlayan grahaları döner.

mod baladi;

pub use baladi::{baladi, jagradadi, Baladi, Jagradadi, BALADI_SPAN_DEG};

use crate::drishti::ASPECTING_GRAHAS;
use crate::graha::Graha;

/// `AvasthaReport::method`: bu modüldeki okumaların kimliği.
pub const AVASTHA_METHOD: &str = "avastha.bphs_v1";

/// Avastha atanan grahalar: rapor satırlarının sırası.
pub const AVASTHA_GRAHAS: [Graha; 7] = ASPECTING_GRAHAS;
````

**Yeni dosya:** `crates/jyotish-core/src/avastha/baladi.rs` (şimdilik yalnızca test modülü; uygulama sonraki adımda üstüne eklenir)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;
    use Sign::*;

    #[test]
    fn baladi_runs_forward_in_odd_signs_and_backward_in_even_signs() {
        use Baladi::*;
        // Tek burç (Koç): 0–6 Bala, 6–12 Kumara, 12–18 Yuva, 18–24 Vriddha, 24–30 Mrita.
        let odd = [
            (0.0, Bala),
            (5.999, Bala),
            (6.0, Kumara),
            (11.999, Kumara),
            (12.0, Yuva),
            (17.999, Yuva),
            (18.0, Vriddha),
            (23.999, Vriddha),
            (24.0, Mrita),
            (29.999, Mrita),
        ];
        for (deg, want) in odd {
            assert_eq!(baladi(Aries, deg), want, "Aries {deg}");
            assert_eq!(baladi(Libra, deg), want, "Libra {deg}");
        }
        // Çift burç (Boğa): sıra terstir.
        let even = [
            (0.0, Mrita),
            (5.999, Mrita),
            (6.0, Vriddha),
            (12.0, Yuva),
            (17.999, Yuva),
            (18.0, Kumara),
            (24.0, Bala),
            (29.999, Bala),
        ];
        for (deg, want) in even {
            assert_eq!(baladi(Taurus, deg), want, "Taurus {deg}");
            assert_eq!(baladi(Pisces, deg), want, "Pisces {deg}");
        }
    }

    #[test]
    fn baladi_is_the_mirror_image_between_an_odd_and_an_even_sign() {
        // İkinci yazım: çift burçtaki basamak, tek burçta 30° − derecenin basamağıdır.
        for sign in Sign::ALL {
            let odd = sign.index().is_multiple_of(2);
            for tenth in 0..300 {
                let deg = f64::from(tenth) / 10.0 + 0.05;
                let forward = Baladi::ALL[(deg / 6.0) as usize];
                let mirrored = Baladi::ALL[((30.0 - deg) / 6.0) as usize];
                let want = if odd { forward } else { mirrored };
                assert_eq!(baladi(sign, deg), want, "{sign:?} {deg}");
            }
        }
        // Aralık dışı derece taşmaz.
        assert_eq!(baladi(Aries, 30.0), Baladi::Mrita);
        assert_eq!(baladi(Aries, -0.1), Baladi::Bala);
        assert_eq!(baladi(Taurus, 30.0), Baladi::Bala);
    }

    #[test]
    fn jagradadi_follows_the_sign_and_the_natural_relation_to_its_lord() {
        use Jagradadi::*;
        // Güneş: yücelme Koç, kendi burcu Aslan, düşüş Terazi.
        assert_eq!(jagradadi(Sun, Aries), Some(Jagrat));
        assert_eq!(jagradadi(Sun, Leo), Some(Jagrat));
        assert_eq!(jagradadi(Sun, Libra), Some(Sushupti));
        // Yengeç'in lordu Ay dost, İkizler'in lordu Merkür nötr: Swapna.
        assert_eq!(jagradadi(Sun, Cancer), Some(Swapna));
        assert_eq!(jagradadi(Sun, Gemini), Some(Swapna));
        // Boğa'nın lordu Venüs ve Kova'nın lordu Satürn düşman: Sushupti.
        assert_eq!(jagradadi(Sun, Taurus), Some(Sushupti));
        assert_eq!(jagradadi(Sun, Aquarius), Some(Sushupti));
        // Merkür Başak'ta hem yücelir hem kendi burcundadır; Balık düşüş burcudur.
        assert_eq!(jagradadi(Mercury, Virgo), Some(Jagrat));
        assert_eq!(jagradadi(Mercury, Pisces), Some(Sushupti));
        // Mars'ın düşüş burcu Yengeç'in lordu Ay dosttur; düşüş önce gelir.
        assert_eq!(jagradadi(Mars, Cancer), Some(Sushupti));
        // Ay kimseyi düşman saymaz: düşüş burcu Akrep dışında Sushupti olmaz.
        let moon: Vec<_> = Sign::ALL
            .into_iter()
            .filter(|&s| jagradadi(Moon, s) == Some(Sushupti))
            .collect();
        assert_eq!(moon, [Scorpio]);
        // Düğümlere avastha atanmaz.
        for sign in Sign::ALL {
            assert_eq!(jagradadi(Rahu, sign), None);
            assert_eq!(jagradadi(Ketu, sign), None);
        }
    }

    #[test]
    fn every_graha_takes_each_jagradadi_state_somewhere() {
        // Değişmez: yedi grahanın her biri on iki burçta bir durum alır; Jagrat en az iki
        // burçtadır (yücelme ve kendi burcu; Merkür'de ikisi Başak'ta çakışır ama İkizler var).
        for graha in [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn] {
            let states: Vec<_> = Sign::ALL
                .into_iter()
                .map(|s| jagradadi(graha, s).expect("seven grahas"))
                .collect();
            let count = |j| states.iter().filter(|&&s| s == j).count();
            assert!(count(Jagradadi::Jagrat) >= 2, "{graha:?}");
            assert!(count(Jagradadi::Swapna) >= 1, "{graha:?}");
            assert!(count(Jagradadi::Sushupti) >= 1, "{graha:?}");
            assert_eq!(states.len(), 12);
        }
    }
}
````

- [ ] **Step 3: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-core --lib avastha 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası (`error[E0432]: unresolved imports` `baladi::baladi`, `baladi::jagradadi`, `baladi::Baladi`, `baladi::Jagradadi`, `baladi::BALADI_SPAN_DEG`: henüz yoklar). Test koşmaz.

- [ ] **Step 4: Baladi ve Jagradadi**

`baladi.rs`'in **başına**, test modülünün üstüne eklenir (arada bir boş satır):

````rust
//! Baladi ve Jagradadi avasthaları. İkisi de tek grahanın kendi yerleşiminden okunur; başka
//! grahaya, lagnaya ve saate bağlı değildir.
//!
//! Baladi: burç altışar derecelik beş dilime bölünür. Tek burçlarda (Koç, İkizler, …) dilimler
//! sırayla Bala, Kumara, Yuva, Vriddha, Mrita'dır; çift burçlarda sıra terstir.
//!
//! Jagradadi: yücelme burcunda ya da kendi burcunda Jagrat; dost ya da nötr grahanın burcunda
//! Swapna; düşman grahanın burcunda ya da düşüş burcunda Sushupti. Dostluk doğal (naisargika)
//! ilişkidir: grahanın, bulunduğu burcun lorduna ilişkisi.

use serde::{Deserialize, Serialize};

use crate::dignity::{exaltation_point, natural_relation, own_signs, sign_lord, Relation};
use crate::graha::Graha;
use crate::zodiac::Sign;

/// Baladi diliminin genişliği, derece.
pub const BALADI_SPAN_DEG: f64 = 6.0;

/// Yaş basamağı.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Baladi {
    Bala,
    Kumara,
    Yuva,
    Vriddha,
    Mrita,
}

impl Baladi {
    /// Tek burçtaki dilim sırası.
    pub const ALL: [Baladi; 5] = [
        Baladi::Bala,
        Baladi::Kumara,
        Baladi::Yuva,
        Baladi::Vriddha,
        Baladi::Mrita,
    ];
}

/// `sign` burcunda `deg_in_sign` derecedeki grahanın Baladi avasthası. Dilim sınırı sonraki
/// dilime aittir (6° Kumara'dır); 30° ve üstü son dilim sayılır.
pub fn baladi(sign: Sign, deg_in_sign: f64) -> Baladi {
    let step = (deg_in_sign / BALADI_SPAN_DEG).floor().clamp(0.0, 4.0) as usize;
    let odd_sign = sign.index().is_multiple_of(2);
    Baladi::ALL[if odd_sign { step } else { 4 - step }]
}

/// Uyanıklık durumu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Jagradadi {
    Jagrat,
    Swapna,
    Sushupti,
}

impl Jagradadi {
    pub const ALL: [Jagradadi; 3] = [Jagradadi::Jagrat, Jagradadi::Swapna, Jagradadi::Sushupti];
}

/// `sign` burcundaki grahanın Jagradadi avasthası; düğümler için `None`. Sıra: yücelme burcu ve
/// kendi burcu, düşüş burcu, sonra burç lorduna doğal ilişki.
pub fn jagradadi(graha: Graha, sign: Sign) -> Option<Jagradadi> {
    let (exaltation, _) = exaltation_point(graha)?;
    if sign == exaltation || own_signs(graha).contains(&sign) {
        return Some(Jagradadi::Jagrat);
    }
    if sign == Sign::from_index(exaltation.index() + 6) {
        return Some(Jagradadi::Sushupti);
    }
    Some(match natural_relation(graha, sign_lord(sign)) {
        Relation::Friend | Relation::Neutral => Jagradadi::Swapna,
        Relation::Enemy => Jagradadi::Sushupti,
    })
}
````

- [ ] **Step 5: Testler ve kapılar**

```bash
cargo fmt --all
cargo test -p jyotish-core --lib avastha 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: `4 passed`; lib `408 passed`, golden `8 passed`, `modules_golden` `17 passed`; fmt ve clippy çıktısız; sayaç `0`. Bir el örneği tutmazsa önce yorumdaki türetme ve D3/D4 yeniden okunur; beklenti motor çıktısına bakarak değiştirilmez. (clippy `is_multiple_of` yerine `% 2 == 0` yazılmasını reddeder: kod bloğundaki yazım korunur.)

- [ ] **Step 6: Commit**

```bash
git add crates/jyotish-core/src/avastha crates/jyotish-core/src/lib.rs
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): avastha module with the Baladi and Jagradadi series" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: üç dosya (`212 insertions`); yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 2: Çekirdek: Lajjitadi kuralları ve küme sonucu

Yol haritası §4.4 ("Lajjitadi") ve §5 Plan 24 görev (2); K4–K6; D5–D7. Altı kuralın her biri için tutan ve tutmayan durum bu görevin birim testlerindedir; harita düzeyindeki el örnekleri Task 3'tedir. `drishti.rs`, `nature.rs`, `dignity.rs` ve `positions.rs` yalnızca okunur.

**Files:**
- Create: `crates/jyotish-core/src/avastha/lajjitadi.rs`
- Modify: `crates/jyotish-core/src/avastha/mod.rs`
- Test: `lajjitadi.rs` `mod tests` (9)

**Interfaces:**
- Consumes (hepsi var): `dignity::{exaltation_point, moolatrikona, natural_relation, sign_lord, Relation}`; `drishti::{graha_aspect_signs, ASPECTING_GRAHAS}`; `nature::{nature, Nature}`; `positions::Positions` (`sign_of`, `point`, `occupants`, `house_of`); Task 1'in `AVASTHA_GRAHAS`'ı (yalnızca testte).
- Produces (`jyotish_core::avastha`):
  - `Lajjitadi { Lajjita, Garvita, Kshudhita, Trushita, Mudita, Kshobhita }` (`Lajjitadi::ALL`: kural sırası; `Ord` bu sıradır).
  - `LajjitadiHit { state: Lajjitadi, grahas: Vec<Graha> }`.
  - `lajjitadi(graha: Graha, positions: &Positions) -> Vec<LajjitadiHit>` (kural sırasında; düğümler için boş; lagna yoksa Lajjita değerlendirilmez).
  - `conjunct_grahas(graha, &Positions) -> Vec<Graha>` (aynı burçtaki öteki grahalar, düğümler dahil), `aspecting_grahas(graha, &Positions) -> Vec<Graha>` (burcuna tam bakışla bakan grahalar), `is_garvita(graha, sign, deg_in_sign) -> bool`.
  - Sabitler: `LAJJITA_HOUSE: u8 = 5`, `LAJJITA_COMPANIONS: [Graha; 5]`, `WATER_SIGNS: [Sign; 3]`.

- [ ] **Step 1: Testler: birliktelik ve bakış, altı kural, küme sırası, düğümler; modül kaydı**

Testler tek bir taban harita üzerindedir (`BASE`: Güneş İkizler'de yalnız) ve her test yalnızca sorduğu kuralın kaydına bakar (`found`): kayıt yoksa `None`, varsa kuralı sağlayan grahalar. Türetmeler yorumlardadır. Son iki test küme sırasını (beş durum birden) ve düğümleri sabitler.

````diff
diff --git a/crates/jyotish-core/src/avastha/mod.rs b/crates/jyotish-core/src/avastha/mod.rs
index 7ef2dce..8a7eb40 100644
--- a/crates/jyotish-core/src/avastha/mod.rs
+++ b/crates/jyotish-core/src/avastha/mod.rs
@@ -10,8 +10,13 @@
 //! yoktur: modül yalnızca sınıfı ve kuralı sağlayan grahaları döner.
 
 mod baladi;
+mod lajjitadi;
 
 pub use baladi::{baladi, jagradadi, Baladi, Jagradadi, BALADI_SPAN_DEG};
+pub use lajjitadi::{
+    aspecting_grahas, conjunct_grahas, is_garvita, lajjitadi, Lajjitadi, LajjitadiHit,
+    LAJJITA_COMPANIONS, LAJJITA_HOUSE, WATER_SIGNS,
+};
 
 use crate::drishti::ASPECTING_GRAHAS;
 use crate::graha::Graha;
````

**Yeni dosya:** `crates/jyotish-core/src/avastha/lajjitadi.rs` (şimdilik yalnızca test modülü; uygulama sonraki adımda üstüne eklenir)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;
    use Lajjitadi::*;
    use Sign::*;

    /// `Graha::ALL` sırasında (burç, burç içi derece).
    type Places = [(Sign, f64); 9];

    fn chart(lagna: Option<Sign>, places: Places) -> Positions {
        Positions::synthetic(
            lagna.map(|s| s.index() as f64 * 30.0 + 15.0),
            places.map(|(sign, deg)| sign.index() as f64 * 30.0 + deg),
        )
    }

    fn with(base: Places, changes: &[(Graha, Sign, f64)]) -> Places {
        let mut out = base;
        for &(g, sign, deg) in changes {
            out[g as usize] = (sign, deg);
        }
        out
    }

    /// Durumun kaydı: yoksa `None`, varsa kuralı sağlayan grahalar.
    fn found(graha: Graha, state: Lajjitadi, p: &Positions) -> Option<Vec<Graha>> {
        lajjitadi(graha, p)
            .into_iter()
            .find(|h| h.state == state)
            .map(|h| h.grahas)
    }

    /// Taban: Güneş İkizler'de yalnız; hiçbir graha İkizler'e bakmıyor.
    /// Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    const BASE: Places = [
        (Gemini, 5.0),
        (Taurus, 10.0),
        (Aquarius, 3.0),
        (Taurus, 20.0),
        (Aries, 10.0),
        (Aries, 15.0),
        (Aries, 12.0),
        (Virgo, 25.0),
        (Pisces, 25.0),
    ];

    #[test]
    fn conjunction_is_the_same_sign_and_aspect_is_the_full_sign_aspect() {
        let p = chart(None, BASE);
        assert_eq!(conjunct_grahas(Jupiter, &p), [Venus, Saturn]);
        assert_eq!(conjunct_grahas(Sun, &p), []);
        assert_eq!(conjunct_grahas(Ketu, &p), []);
        // Koç'taki Satürn 3. (İkizler), 7. (Terazi) ve 10. (Oğlak) burca bakar; Jüpiter 5., 7.
        // ve 9. burca; Venüs yalnız 7. burca. İkizler'e yalnız Satürn bakar.
        assert_eq!(aspecting_grahas(Sun, &p), [Saturn]);
        // Kova'daki Mars 4. burca (Boğa) bakar: Ay ve Merkür.
        assert_eq!(aspecting_grahas(Moon, &p), [Mars]);
        assert_eq!(aspecting_grahas(Mercury, &p), [Mars]);
        // Aynı burçtaki graha bakmaz; düğümler bakmaz ama bakılır.
        assert_eq!(aspecting_grahas(Jupiter, &p), []);
        let q = chart(None, with(BASE, &[(Rahu, Libra, 1.0)]));
        assert_eq!(aspecting_grahas(Rahu, &q), [Jupiter, Venus, Saturn]);
    }

    #[test]
    fn lajjita_needs_the_fifth_house_and_one_of_the_five_companions() {
        // Lagna Koç: 5. ev Aslan. Jüpiter Aslan'da Rahu ile.
        let places = with(BASE, &[(Jupiter, Leo, 10.0), (Rahu, Leo, 25.0)]);
        let p = chart(Some(Aries), places);
        assert_eq!(found(Jupiter, Lajjita, &p), Some(vec![Rahu]));
        // Birden çok eşlikçi `Graha::ALL` sırasında yazılır.
        let crowd = with(places, &[(Saturn, Leo, 1.0), (Sun, Leo, 2.0)]);
        assert_eq!(
            found(Jupiter, Lajjita, &chart(Some(Aries), crowd)),
            Some(vec![Sun, Saturn, Rahu])
        );
        // Güneş'in kendisi de 5. evde Lajjita olur; kendi kendisinin eşlikçisi değildir.
        assert_eq!(
            found(Sun, Lajjita, &chart(Some(Aries), crowd)),
            Some(vec![Saturn, Rahu])
        );
        // Ev 5 değilse yok (lagna Boğa: Aslan 4. ev).
        assert_eq!(found(Jupiter, Lajjita, &chart(Some(Taurus), places)), None);
        // Eşlikçi beş grahadan biri değilse yok (Ay, Merkür ve Venüs sayılmaz).
        let mild = with(
            BASE,
            &[
                (Jupiter, Leo, 10.0),
                (Moon, Leo, 1.0),
                (Mercury, Leo, 2.0),
                (Venus, Leo, 3.0),
            ],
        );
        assert_eq!(found(Jupiter, Lajjita, &chart(Some(Aries), mild)), None);
        // Lagna yoksa değerlendirilmez.
        assert_eq!(found(Jupiter, Lajjita, &chart(None, places)), None);
    }

    #[test]
    fn garvita_is_the_exaltation_sign_or_the_moolatrikona_range() {
        for (graha, sign, deg, want) in [
            (Sun, Aries, 29.0, true),      // yücelme burcunun tamamı
            (Sun, Leo, 10.0, true),        // moolatrikona 0–20°
            (Sun, Leo, 20.0, false),       // aralığın bitişi dışarıdadır: kendi burcu
            (Moon, Taurus, 1.0, true),     // yücelme
            (Moon, Taurus, 20.0, true),    // moolatrikona 3–30°
            (Mars, Aries, 11.9, true),     // moolatrikona 0–12°
            (Mars, Aries, 12.0, false),    // kendi burcu
            (Mars, Scorpio, 5.0, false),   // kendi burcu
            (Mercury, Virgo, 25.0, true),  // yücelme burcu (moolatrikona 15–20° dışında da)
            (Mercury, Gemini, 5.0, false), // kendi burcu
            (Jupiter, Sagittarius, 9.0, true),
            (Jupiter, Pisces, 9.0, false),
            (Venus, Libra, 14.0, true),
            (Venus, Taurus, 14.0, false),
            (Saturn, Aquarius, 10.0, true),
            (Saturn, Aquarius, 25.0, false),
            (Saturn, Libra, 1.0, true),
            (Rahu, Taurus, 1.0, false),
        ] {
            assert_eq!(
                is_garvita(graha, sign, deg),
                want,
                "{graha:?} {sign:?} {deg}"
            );
        }
        let p = chart(None, with(BASE, &[(Sun, Aries, 29.0)]));
        assert_eq!(found(Sun, Garvita, &p), Some(vec![]));
        assert_eq!(found(Sun, Garvita, &chart(None, BASE)), None);
    }

    #[test]
    fn kshudhita_is_an_enemy_sign_an_enemy_companion_an_enemy_aspect_or_saturn() {
        // Tabanda İkizler'deki Güneş'e düşmanı Satürn (Koç'tan 3. bakış) bakar.
        assert_eq!(
            found(Sun, Kshudhita, &chart(None, BASE)),
            Some(vec![Saturn])
        );
        // Satürn Boğa'ya alınınca kimse bakmaz; İkizler'in lordu Merkür nötrdür: yok.
        let calm = with(BASE, &[(Saturn, Taurus, 12.0)]);
        assert_eq!(found(Sun, Kshudhita, &chart(None, calm)), None);
        // Düşman burcu tek başına yeter: Güneş Terazi'de (lord Venüs). Koç'taki Venüs karşıdan
        // bakar; Jüpiter de bakar ama düşman değildir.
        let libra = with(calm, &[(Sun, Libra, 5.0)]);
        assert_eq!(
            found(Sun, Kshudhita, &chart(None, libra)),
            Some(vec![Venus])
        );
        // Venüs de Boğa'ya alınınca kayıt yalnız burçtandır: graha listesi boş.
        let sign_only = with(libra, &[(Venus, Taurus, 1.0)]);
        assert_eq!(found(Sun, Kshudhita, &chart(None, sign_only)), Some(vec![]));
        // Düşmanla birliktelik: Güneş İkizler'de Venüs ile.
        let together = with(calm, &[(Venus, Gemini, 9.0)]);
        assert_eq!(
            found(Sun, Kshudhita, &chart(None, together)),
            Some(vec![Venus])
        );
        // Satürn ile birliktelik, Satürn düşman olmasa da yeter: tabanda Venüs (Satürn dostu)
        // ve Jüpiter (Satürn nötr) Koç'ta Satürn iledir. Jüpiter'in düşmanı Venüs de oradadır.
        assert_eq!(
            found(Venus, Kshudhita, &chart(None, BASE)),
            Some(vec![Saturn])
        );
        assert_eq!(
            found(Jupiter, Kshudhita, &chart(None, BASE)),
            Some(vec![Venus, Saturn])
        );
        // Satürn kendi kendisinin eşlikçisi değildir: Boğa'da (lord Venüs dost) Ay ve Merkür
        // iledir; Ay düşmanıdır. Kova'daki Mars da düşmanıdır ve 4. bakışla Boğa'ya bakar.
        assert_eq!(
            found(Saturn, Kshudhita, &chart(None, calm)),
            Some(vec![Moon, Mars])
        );
        // Ay'ın doğal düşmanı yoktur: Satürn ile birlikte olmadıkça Kshudhita olmaz.
        for sign in Sign::ALL {
            let p = chart(None, with(BASE, &[(Moon, sign, 10.0)]));
            let want = (sign == Aries).then(|| vec![Saturn]);
            assert_eq!(found(Moon, Kshudhita, &p), want, "{sign:?}");
        }
    }

    #[test]
    fn trushita_needs_a_water_sign_an_enemy_aspect_and_no_benefic_aspect() {
        // Güneş Yengeç'te; Satürn Oğlak'tan karşıya bakar. İyicil bakış yok.
        let places = with(
            BASE,
            &[
                (Sun, Cancer, 5.0),
                (Saturn, Capricorn, 12.0),
                (Jupiter, Taurus, 1.0),
                (Venus, Taurus, 2.0),
            ],
        );
        assert_eq!(
            found(Sun, Trushita, &chart(None, places)),
            Some(vec![Saturn])
        );
        // Jüpiter Balık'tan 5. bakışla Yengeç'e bakarsa iyicil bakış vardır: yok.
        let helped = with(places, &[(Jupiter, Pisces, 1.0)]);
        assert_eq!(found(Sun, Trushita, &chart(None, helped)), None);
        // Bakan düşman kendisi iyicilse (Venüs) yine yok; Kshudhita ise vardır. Satürn Aslan'a
        // alınır: oradan 3., 7. ve 10. bakışı Terazi, Kova ve Boğa'ya düşer, Yengeç'e değil.
        let venus = with(places, &[(Saturn, Leo, 12.0), (Venus, Capricorn, 2.0)]);
        assert_eq!(found(Sun, Trushita, &chart(None, venus)), None);
        assert_eq!(
            found(Sun, Kshudhita, &chart(None, venus)),
            Some(vec![Venus])
        );
        // Su burcu değilse yok: Güneş Aslan'da, Satürn Kova'dan bakar.
        let fire = with(places, &[(Sun, Leo, 5.0), (Saturn, Aquarius, 12.0)]);
        assert_eq!(found(Sun, Trushita, &chart(None, fire)), None);
        // Düşman bakışı yoksa yok.
        let alone = with(places, &[(Saturn, Leo, 12.0)]);
        assert_eq!(found(Sun, Trushita, &chart(None, alone)), None);
    }

    #[test]
    fn mudita_is_a_friend_sign_a_friend_companion_a_friend_aspect_or_jupiter() {
        let calm = with(BASE, &[(Saturn, Taurus, 12.0)]);
        // İkizler'in lordu Merkür Güneş'e nötrdür; yalnız ve bakışsız: yok.
        assert_eq!(found(Sun, Mudita, &chart(None, calm)), None);
        // Dost burcu: Güneş Yengeç'te (lord Ay). Graha listesi boş.
        let cancer = with(calm, &[(Sun, Cancer, 5.0)]);
        assert_eq!(found(Sun, Mudita, &chart(None, cancer)), Some(vec![]));
        // Dostla birliktelik: Mars İkizler'e gelir.
        let together = with(calm, &[(Mars, Gemini, 3.0)]);
        assert_eq!(found(Sun, Mudita, &chart(None, together)), Some(vec![Mars]));
        // Dost bakışı: Jüpiter Yay'dan karşıya bakar.
        let aspect = with(calm, &[(Jupiter, Sagittarius, 10.0)]);
        assert_eq!(
            found(Sun, Mudita, &chart(None, aspect)),
            Some(vec![Jupiter])
        );
        // Jüpiter ile birliktelik, Jüpiter dost olmasa da yeter: tabanda Venüs Koç'ta Jüpiter
        // ile (Jüpiter Venüs'e nötr), Satürn de oradadır (dost).
        assert_eq!(
            found(Venus, Mudita, &chart(None, BASE)),
            Some(vec![Jupiter, Saturn])
        );
        let only_jupiter = with(BASE, &[(Saturn, Virgo, 12.0)]);
        assert_eq!(
            found(Venus, Mudita, &chart(None, only_jupiter)),
            Some(vec![Jupiter])
        );
        // Jüpiter de giderse Koç'ta (lord Mars nötr) yalnız Venüs için yok.
        let none = with(only_jupiter, &[(Jupiter, Virgo, 10.0)]);
        assert_eq!(found(Venus, Mudita, &chart(None, none)), None);
    }

    #[test]
    fn kshobhita_needs_the_sun_and_a_malefic_or_enemy_aspect() {
        // Merkür İkizler'de Güneş ile; Satürn (kötücül, Merkür'e nötr) Yay'dan karşıya bakar.
        let places = with(
            BASE,
            &[
                (Mercury, Gemini, 20.0),
                (Saturn, Sagittarius, 12.0),
                (Moon, Aries, 10.0),
            ],
        );
        assert_eq!(
            found(Mercury, Kshobhita, &chart(None, places)),
            Some(vec![Sun, Saturn])
        );
        // Güneş'in kendisi Kshobhita olmaz.
        assert_eq!(found(Sun, Kshobhita, &chart(None, places)), None);
        // Bakan yalnız iyicil ve düşman olmayan bir grahaysa (Jüpiter) yok.
        let mild = with(
            places,
            &[(Saturn, Taurus, 12.0), (Jupiter, Sagittarius, 10.0)],
        );
        assert_eq!(found(Mercury, Kshobhita, &chart(None, mild)), None);
        // Bakan iyicil ama düşmansa var: Venüs İkizler'de Güneş ile; Ay Yay 3°'den karşıya bakar
        // ve Venüs'ün düşmanıdır. Güneş 65°, Ay 243°: uzanım 178°, Ay büyüyor (iyicil).
        let moon = with(
            mild,
            &[
                (Venus, Gemini, 9.0),
                (Jupiter, Taurus, 1.0),
                (Moon, Sagittarius, 3.0),
            ],
        );
        let p = chart(None, moon);
        assert_eq!(nature(Moon, &p), Nature::Benefic);
        assert_eq!(found(Venus, Kshobhita, &p), Some(vec![Sun, Moon]));
        // Güneş ile birlikte değilse yok.
        let apart = with(places, &[(Mercury, Cancer, 20.0)]);
        assert_eq!(found(Mercury, Kshobhita, &chart(None, apart)), None);
    }

    #[test]
    fn one_graha_can_be_in_five_states_at_once_in_rule_order() {
        // Lagna Akrep: 5. ev Balık. Venüs Balık'ta (yücelme) Güneş ile; Satürn Başak'tan karşıya
        // bakar. Balık'a başka bakan yok.
        let places: Places = [
            (Pisces, 5.0),
            (Aries, 10.0),
            (Aries, 3.0),
            (Aquarius, 20.0),
            (Taurus, 10.0),
            (Pisces, 15.0),
            (Virgo, 12.0),
            (Gemini, 25.0),
            (Sagittarius, 25.0),
        ];
        let p = chart(Some(Scorpio), places);
        let hits = lajjitadi(Venus, &p);
        let states: Vec<_> = hits.iter().map(|h| h.state).collect();
        assert_eq!(states, [Lajjita, Garvita, Kshudhita, Mudita, Kshobhita]);
        let grahas: Vec<_> = hits.iter().map(|h| h.grahas.clone()).collect();
        assert_eq!(
            grahas,
            [
                vec![Sun],         // 5. evde Güneş ile
                vec![],            // yücelme burcu
                vec![Sun],         // düşmanı Güneş ile
                vec![Saturn],      // dostu Satürn'ün bakışı
                vec![Sun, Saturn]  // Güneş ile, kötücül Satürn'ün bakışı
            ]
        );
        // Trushita yok: su burcunda ama bakan (Satürn) düşman değil.
        assert_eq!(found(Venus, Trushita, &p), None);
        // Sıra her zaman kural sırasıdır ve bir durum iki kez yazılmaz.
        for graha in Graha::ALL {
            let states: Vec<_> = lajjitadi(graha, &p).iter().map(|h| h.state).collect();
            let mut sorted = states.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(states, sorted, "{graha:?}");
        }
    }

    #[test]
    fn nodes_have_no_state_and_no_list_names_the_graha_itself() {
        let p = chart(Some(Aries), BASE);
        assert_eq!(lajjitadi(Rahu, &p), []);
        assert_eq!(lajjitadi(Ketu, &p), []);
        for graha in crate::avastha::AVASTHA_GRAHAS {
            for h in lajjitadi(graha, &p) {
                assert!(!h.grahas.contains(&graha), "{graha:?} {:?}", h.state);
            }
        }
    }
}
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-core --lib avastha 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası (`error[E0432]: unresolved imports` `lajjitadi::lajjitadi`, `lajjitadi::Lajjitadi`, `lajjitadi::LajjitadiHit` ve ötekiler). Test koşmaz.

- [ ] **Step 3: Altı kural**

`lajjitadi.rs`'in **başına**, test modülünün üstüne eklenir (arada bir boş satır). Kural sırası `Lajjitadi::ALL` sırasıdır ve `hit` çağrılarının sırasıyla sağlanır; `grahas` her kayıtta `in_graha_order` ile sıralanır ve tekrarsızdır.

````rust
//! Lajjitadi avasthaları: yerleşim, birliktelik ve bakıştan altı durum. Bir graha aynı anda
//! birden çok durumda olabilir; sonuç, kural sırasında bir listedir ve boş olabilir.
//!
//! Bu dosyadaki okuma (varyant noktaları `tests/golden/modules/avastha/REVIEW.md`'dedir):
//! - **Birliktelik** aynı rasi burcunda bulunmaktır; **bakış** burçtan burca tam graha
//!   bakışıdır (`graha_aspect_signs`). Rahu ve Ketu bakmaz.
//! - **Dost** ve **düşman** doğal (naisargika) ilişkidir: durumu sorulan grahanın ötekine
//!   ilişkisi. Düğümler kimsenin dostu ya da düşmanı değildir.
//! - **İyicil** ve **kötücül** `nature` sınıfıdır (K6).
//!
//! Kurallar:
//! 1. Lajjita: graha lagnadan 5. evde ve Güneş, Mars, Satürn, Rahu ya da Ketu ile birlikte.
//! 2. Garvita: graha yücelme burcunda ya da moolatrikona aralığında.
//! 3. Kshudhita: graha düşmanının burcunda, ya da bir düşmanıyla birlikte, ya da bir düşmanının
//!    bakışı altında, ya da Satürn ile birlikte.
//! 4. Trushita: graha su burcunda (Yengeç, Akrep, Balık), bir düşmanının bakışı altında ve
//!    hiçbir iyicil ona bakmıyor.
//! 5. Mudita: graha dostunun burcunda, ya da bir dostuyla birlikte, ya da bir dostunun bakışı
//!    altında, ya da Jüpiter ile birlikte.
//! 6. Kshobhita: graha Güneş ile birlikte ve bir kötücülün ya da bir düşmanının bakışı altında.

use serde::{Deserialize, Serialize};

use crate::dignity::{exaltation_point, moolatrikona, natural_relation, sign_lord, Relation};
use crate::drishti::{graha_aspect_signs, ASPECTING_GRAHAS};
use crate::graha::Graha;
use crate::nature::{nature, Nature};
use crate::positions::Positions;
use crate::zodiac::Sign;

/// Lajjita'nın evi (lagnadan).
pub const LAJJITA_HOUSE: u8 = 5;

/// Lajjita'yı oluşturan birliktelikler.
pub const LAJJITA_COMPANIONS: [Graha; 5] = [
    Graha::Sun,
    Graha::Mars,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// Trushita'nın su burçları.
pub const WATER_SIGNS: [Sign; 3] = [Sign::Cancer, Sign::Scorpio, Sign::Pisces];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Lajjitadi {
    Lajjita,
    Garvita,
    Kshudhita,
    Trushita,
    Mudita,
    Kshobhita,
}

impl Lajjitadi {
    /// Kural sırası: raporun liste sırası.
    pub const ALL: [Lajjitadi; 6] = [
        Lajjitadi::Lajjita,
        Lajjitadi::Garvita,
        Lajjitadi::Kshudhita,
        Lajjitadi::Trushita,
        Lajjitadi::Mudita,
        Lajjitadi::Kshobhita,
    ];
}

/// Grahanın içinde bulunduğu bir Lajjitadi durumu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct LajjitadiHit {
    pub state: Lajjitadi,
    /// Kuralı sağlayan öteki grahalar (birlikte olan ya da bakan), `Graha::ALL` sırasında.
    /// Kural yalnızca burçtan sağlanıyorsa boştur.
    pub grahas: Vec<Graha>,
}

/// `graha` ile aynı rasi burcundaki öteki grahalar, `Graha::ALL` sırasında (düğümler dahil).
pub fn conjunct_grahas(graha: Graha, positions: &Positions) -> Vec<Graha> {
    positions
        .occupants(positions.sign_of(graha))
        .into_iter()
        .filter(|&g| g != graha)
        .collect()
}

/// `graha`'nın burcuna tam bakışla bakan grahalar, `ASPECTING_GRAHAS` sırasında.
pub fn aspecting_grahas(graha: Graha, positions: &Positions) -> Vec<Graha> {
    let sign = positions.sign_of(graha);
    ASPECTING_GRAHAS
        .into_iter()
        .filter(|&g| g != graha && graha_aspect_signs(g, positions.sign_of(g)).contains(&sign))
        .collect()
}

/// Graha yücelme burcunda mı ya da moolatrikona aralığında mı. Düğümler için yanlıştır.
pub fn is_garvita(graha: Graha, sign: Sign, deg_in_sign: f64) -> bool {
    let exalted = exaltation_point(graha).is_some_and(|(s, _)| s == sign);
    let in_moolatrikona = moolatrikona(graha)
        .is_some_and(|(s, from, to)| s == sign && deg_in_sign >= from && deg_in_sign < to);
    exalted || in_moolatrikona
}

/// Tekrarsız ve `Graha::ALL` sırasında.
fn in_graha_order(grahas: impl IntoIterator<Item = Graha>) -> Vec<Graha> {
    let picked: Vec<Graha> = grahas.into_iter().collect();
    Graha::ALL
        .into_iter()
        .filter(|g| picked.contains(g))
        .collect()
}

/// `graha`'nın Lajjitadi durumları, kural sırasında. Düğümler için boştur. Lagna yoksa (doğum
/// saati bilinmiyor) Lajjita değerlendirilmez; öteki beş kural lagnaya bağlı değildir.
pub fn lajjitadi(graha: Graha, positions: &Positions) -> Vec<LajjitadiHit> {
    if graha.is_node() {
        return Vec::new();
    }
    let point = positions.point(graha);
    let sign = point.rasi_sign;
    let lord = sign_lord(sign);
    let together = conjunct_grahas(graha, positions);
    let aspecting = aspecting_grahas(graha, positions);
    let relation = |other: Graha| natural_relation(graha, other);
    let pick = |from: &[Graha], wanted: Relation| -> Vec<Graha> {
        from.iter()
            .copied()
            .filter(|&g| relation(g) == wanted)
            .collect()
    };
    let enemies_together = pick(&together, Relation::Enemy);
    let enemies_aspecting = pick(&aspecting, Relation::Enemy);
    let friends_together = pick(&together, Relation::Friend);
    let friends_aspecting = pick(&aspecting, Relation::Friend);

    let mut hits = Vec::new();
    let mut hit = |state: Lajjitadi, grahas: Vec<Graha>| {
        hits.push(LajjitadiHit {
            state,
            grahas: in_graha_order(grahas),
        });
    };

    if positions.house_of(graha) == Some(LAJJITA_HOUSE) {
        let companions: Vec<Graha> = together
            .iter()
            .copied()
            .filter(|g| LAJJITA_COMPANIONS.contains(g))
            .collect();
        if !companions.is_empty() {
            hit(Lajjitadi::Lajjita, companions);
        }
    }

    if is_garvita(graha, sign, point.deg_in_sign) {
        hit(Lajjitadi::Garvita, Vec::new());
    }

    let with_saturn = together.contains(&Graha::Saturn);
    if relation(lord) == Relation::Enemy
        || !enemies_together.is_empty()
        || !enemies_aspecting.is_empty()
        || with_saturn
    {
        let mut grahas = enemies_together.clone();
        grahas.extend(&enemies_aspecting);
        if with_saturn {
            grahas.push(Graha::Saturn);
        }
        hit(Lajjitadi::Kshudhita, grahas);
    }

    let benefic_aspect = aspecting
        .iter()
        .any(|&g| nature(g, positions) == Nature::Benefic);
    if WATER_SIGNS.contains(&sign) && !enemies_aspecting.is_empty() && !benefic_aspect {
        hit(Lajjitadi::Trushita, enemies_aspecting.clone());
    }

    let with_jupiter = together.contains(&Graha::Jupiter);
    if relation(lord) == Relation::Friend
        || !friends_together.is_empty()
        || !friends_aspecting.is_empty()
        || with_jupiter
    {
        let mut grahas = friends_together;
        grahas.extend(&friends_aspecting);
        if with_jupiter {
            grahas.push(Graha::Jupiter);
        }
        hit(Lajjitadi::Mudita, grahas);
    }

    if together.contains(&Graha::Sun) {
        let pressing: Vec<Graha> = aspecting
            .iter()
            .copied()
            .filter(|&g| relation(g) == Relation::Enemy || nature(g, positions) == Nature::Malefic)
            .collect();
        if !pressing.is_empty() {
            let mut grahas = vec![Graha::Sun];
            grahas.extend(pressing);
            hit(Lajjitadi::Kshobhita, grahas);
        }
    }

    hits
}
````

- [ ] **Step 4: Testler ve kapılar**

```bash
cargo fmt --all
cargo test -p jyotish-core --lib avastha::lajjitadi 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: `9 passed`; lib `417 passed`, golden `8 passed`, `modules_golden` `17 passed`; fmt ve clippy çıktısız; sayaç `0`. Bir test tutmazsa önce yorumdaki türetme yeniden yapılır: en sık atlanan şey özel bakışlardır (Mars 4. ve 8., Jüpiter 5. ve 9., Satürn 3. ve 10. burca da bakar). Beklenti motor çıktısına bakarak değiştirilmez.

- [ ] **Step 5: Commit**

```bash
git add crates/jyotish-core/src/avastha
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): Lajjitadi rules; the result is a set with the grahas that satisfy each rule" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: iki dosya (`560 insertions`); tarama çıktısız; `git status --short` boş.

---

### Task 3: Lajjitadi el örnekleri (Baladi ve Jagradadi ile)

Yol haritası §5 Plan 24 görev (3): "her durum için tutan ve tutmayan"; ileri modüller spec'i §13 (el örnekleri), K9; D12. Yedi sentetik harita; her dosya yedi grahanın Baladi, Jagradadi ve Lajjitadi durumlarını ve örneğin "oluşmayan durum" olarak gösterdiği çiftleri (`absent`) taşır. Çekirdek kod değişmez. Dosyalardaki `rule_ref`, Task 8'de yazılan `avastha/REVIEW.md`'ye gönderme yapar.

**Files:**
- Create: `crates/jyotish-core/tests/avastha_golden.rs`; `tests/golden/modules/avastha/lajjitadi/` altında yedi dosya: `jupiter-in-fifth-with-rahu.json`, `mercury-with-saturn-kshudhita-by-company.json`, `mercury-with-sun-kshobhita-venus-in-no-state.json`, `sun-in-cancer-trushita-by-saturn.json`, `time-unknown-fifth-house-not-evaluated.json`, `trushita-cancelled-by-jupiter-aspect.json`, `venus-exalted-with-sun-five-states.json`
- Test: `avastha_golden.rs` (3): `lajjitadi_hand_fixtures`, `lajjitadi_fixtures_cover_the_required_cases`, `unknown_fields_are_rejected`

**Interfaces:**
- Consumes: Task 1–2'nin `baladi`, `jagradadi`, `lajjitadi`, `Baladi`, `Jagradadi`, `Lajjitadi`, `LajjitadiHit`, `AVASTHA_GRAHAS`; `jyotish_core::{Graha, Positions}` (`Positions::synthetic`); `serde`, `serde_json` (çekirdeğin var olan geliştirme bağımlılıkları).
- Produces: el örneği biçimi (`id`, `source: "hand"`, `rule_ref`, `derivation`, `input { lagna_lon, grahas }`, `expected { baladi, jagradadi, lajjitadi, absent }`); `MIN_LAJJITADI = 7`.

- [ ] **Step 1: Test dosyası**

Yükleyici `modules_golden.rs`'inkiyle aynı denetimleri yapar (en az sayı, dosya adı = `id`, `source == "hand"`, boş olmayan `rule_ref` ve `derivation`, bilinmeyen alanın reddi). Kapsam testi şunları sabitler: altı durumun her biri en az bir örnekte oluşur ve en az birinde `absent` sayılır; boş küme, tek durum ve en az dört durum birlikte; yalnızca burçtan sağlanan kayıt ve birden çok grahalı kayıt; Kshudhita'nın dördüncü kolu (düşmanı olmayan Satürn ile birliktelik); doğum saati bilinmeyen harita; beş Baladi basamağının her biri hem tek hem çift burçta; üç Jagradadi durumu.

**Yeni dosya:** `crates/jyotish-core/tests/avastha_golden.rs`

````rust
//! Avastha el örnekleri (yol haritası §4.4; ileri modüller spec'i §13): beklenen değerler
//! kuraldan elle türetilir (`derivation`), başka yazılımın çıktısı olamaz (K9). Her alt klasör
//! bir dizidir: `tests/golden/modules/avastha/lajjitadi/` (Baladi, Jagradadi ve Lajjitadi).
//! Okuma kuralları ve onay bekleyen varyant noktaları `avastha/REVIEW.md`'dedir.

use std::collections::BTreeMap;
use std::path::PathBuf;

use jyotish_core::avastha::{
    baladi, jagradadi, lajjitadi, Baladi, Jagradadi, Lajjitadi, LajjitadiHit, AVASTHA_GRAHAS,
};
use jyotish_core::{Graha, Positions};
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// Örnek silmek testi düşürür.
const MIN_LAJJITADI: usize = 7;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture<X, I> {
    id: String,
    source: String,
    rule_ref: String,
    derivation: Vec<String>,
    input: I,
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

/// Bir dizinin el örnekleri, ortak denetimleriyle: en az sayı, dosya adı, kaynak, gönderme ve
/// türetme adımları.
fn load<X: DeserializeOwned, I: DeserializeOwned>(series: &str, min: usize) -> Vec<Fixture<X, I>> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/golden/modules/avastha")
        .join(series);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    assert!(
        paths.len() >= min,
        "{series}: {} hand fixtures, at least {min} required",
        paths.len()
    );
    paths
        .into_iter()
        .map(|p| {
            let text =
                std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            let f: Fixture<X, I> =
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
            f
        })
        .collect()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Absent {
    graha: Graha,
    state: Lajjitadi,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LajjitadiExpected {
    baladi: BTreeMap<Graha, Baladi>,
    jagradadi: BTreeMap<Graha, Jagradadi>,
    /// Yedi grahanın durumları, kural sırasında; durumu olmayan graha boş liste taşır.
    lajjitadi: BTreeMap<Graha, Vec<LajjitadiHit>>,
    /// Bu örneğin "oluşmayan durum" olarak gösterdiği çiftler.
    absent: Vec<Absent>,
}

fn lajjitadi_fixtures() -> Vec<(Fixture<LajjitadiExpected, Input>, Positions)> {
    load::<LajjitadiExpected, Input>("lajjitadi", MIN_LAJJITADI)
        .into_iter()
        .map(|f| {
            let positions = f.input.positions(&f.id);
            (f, positions)
        })
        .collect()
}

#[test]
fn lajjitadi_hand_fixtures() {
    for (f, p) in lajjitadi_fixtures() {
        let x = &f.expected;
        let seven: Vec<Graha> = AVASTHA_GRAHAS.to_vec();
        for (what, keys) in [
            ("baladi", x.baladi.keys().copied().collect::<Vec<_>>()),
            ("jagradadi", x.jagradadi.keys().copied().collect()),
            ("lajjitadi", x.lajjitadi.keys().copied().collect()),
        ] {
            assert_eq!(keys, seven, "{}: {what} lists the seven grahas", f.id);
        }
        for graha in AVASTHA_GRAHAS {
            let point = p.point(graha);
            assert_eq!(
                baladi(point.rasi_sign, point.deg_in_sign),
                x.baladi[&graha],
                "{}: baladi of {graha:?}",
                f.id
            );
            assert_eq!(
                jagradadi(graha, point.rasi_sign),
                Some(x.jagradadi[&graha]),
                "{}: jagradadi of {graha:?}",
                f.id
            );
            assert_eq!(
                lajjitadi(graha, &p),
                x.lajjitadi[&graha],
                "{}: lajjitadi of {graha:?}",
                f.id
            );
        }
        // "Oluşmayan" çiftler beklenen listede de motorda da yoktur.
        assert!(!x.absent.is_empty(), "{}: absent pairs", f.id);
        for a in &x.absent {
            assert!(
                x.lajjitadi[&a.graha].iter().all(|h| h.state != a.state),
                "{}: {:?} {:?} is listed both as present and absent",
                f.id,
                a.graha,
                a.state
            );
        }
        // Lagna yoksa Lajjita hiçbir grahada beklenmez.
        if f.input.lagna_lon.is_none() {
            assert!(
                x.lajjitadi
                    .values()
                    .flatten()
                    .all(|h| h.state != Lajjitadi::Lajjita),
                "{}",
                f.id
            );
        }
    }
}

#[test]
fn lajjitadi_fixtures_cover_the_required_cases() {
    let fixtures = lajjitadi_fixtures();
    let hits = || {
        fixtures
            .iter()
            .flat_map(|(f, _)| f.expected.lajjitadi.values())
    };
    // Her durum en az bir örnekte oluşur ve en az bir örnekte "oluşmayan" olarak gösterilir.
    for state in Lajjitadi::ALL {
        assert!(
            hits().flatten().any(|h| h.state == state),
            "{state:?} holds in no fixture"
        );
        assert!(
            fixtures
                .iter()
                .flat_map(|(f, _)| &f.expected.absent)
                .any(|a| a.state == state),
            "{state:?} is shown absent in no fixture"
        );
    }
    // Küme sonucu: hiçbir durumda olmayan graha, tek durum ve en az dört durum birlikte.
    assert!(hits().any(|list| list.is_empty()), "an empty set");
    assert!(hits().any(|list| list.len() == 1), "a single state");
    assert!(hits().any(|list| list.len() >= 4), "four or more states");
    // Kuralı sağlayan grahalar: boş liste (yalnızca burçtan) ve birden çok graha.
    assert!(hits().flatten().any(|h| h.grahas.is_empty()));
    assert!(hits().flatten().any(|h| h.grahas.len() >= 2));
    // Kshudhita'nın dördüncü kolu: düşmanı olmayan Satürn ile birliktelik tek başına yeter.
    assert!(
        fixtures.iter().any(|(f, _)| {
            [Graha::Mercury, Graha::Jupiter, Graha::Venus]
                .iter()
                .any(|g| {
                    f.expected.lajjitadi[g]
                        .iter()
                        .any(|h| h.state == Lajjitadi::Kshudhita && h.grahas == [Graha::Saturn])
                })
        }),
        "Kshudhita by Saturn's company alone"
    );
    // Doğum saati bilinmeyen harita, 5. evde eşlikçisi olacak grahayla.
    assert!(
        fixtures.iter().any(|(f, _)| f.input.lagna_lon.is_none()
            && f.expected
                .absent
                .iter()
                .any(|a| a.state == Lajjitadi::Lajjita)),
        "a time-unknown fixture that shows Lajjita is not evaluated"
    );
    // Baladi: beş basamağın her biri hem tek hem çift burçta; Jagradadi: üç durum.
    for state in Baladi::ALL {
        for odd in [true, false] {
            assert!(
                fixtures
                    .iter()
                    .any(|(f, p)| AVASTHA_GRAHAS.iter().any(|&g| {
                        f.expected.baladi[&g] == state
                            && p.sign_of(g).index().is_multiple_of(2) == odd
                    })),
                "{state:?} in an {} sign",
                if odd { "odd" } else { "even" }
            );
        }
    }
    for state in Jagradadi::ALL {
        assert!(
            fixtures
                .iter()
                .any(|(f, _)| f.expected.jagradadi.values().any(|&j| j == state)),
            "{state:?}"
        );
    }
}

#[test]
fn unknown_fields_are_rejected() {
    let text = r#"{"lagna_lon": null, "grahas": {}, "extra": 1}"#;
    assert!(serde_json::from_str::<Input>(text).is_err());
    let text = r#"{"graha": "Sun", "state": "Lajjita", "why": "x"}"#;
    assert!(serde_json::from_str::<Absent>(text).is_err());
}
````

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `cargo test -p jyotish-core --test avastha_golden 2>&1 | grep -E "panicked|cannot read|^test result"`
Expected: `lajjitadi_hand_fixtures` ve `lajjitadi_fixtures_cover_the_required_cases` `cannot read …/tests/golden/modules/avastha/lajjitadi` ile düşer; `test result: FAILED. 1 passed; 2 failed`.

- [ ] **Step 3: Yedi el örneği**

Her dosyada `derivation` önce örneğin gösterdiği durumu, sonra Ay'ın sınıfını (uzanım), sonra her graha için iki satırı yazar: yerleşim satırı (burç ve derece, Baladi dilimi, Jagradadi'nin gerekçesi, ev, birlikte olanlar, bakanlar, burç lordunun ilişkisi) ve durum satırı. Son satırlar `absent` çiftlerinin gerekçesidir. Değerler kuraldan türetilmiştir (D12).

**Yeni dosya:** `tests/golden/modules/avastha/lajjitadi/jupiter-in-fifth-with-rahu.json`

````json
{
  "id": "jupiter-in-fifth-with-rahu",
  "source": "hand",
  "rule_ref": "BPHS, avasthalar bölümü: Baladi, Jagradadi ve Lajjitadi (alıntı yapılmaz); okuma kuralları avastha/REVIEW.md",
  "derivation": [
    "Lagna Koç: 5. ev Aslan. Jüpiter Aslan'da Rahu ile: Lajjita. Aslan'ın lordu Güneş dostu: Mudita.",
    "Ay'ın sınıfı: uzanım = Ay − Güneş = 40.0 − 65.0 ≡ 335.0°; 180° ya da daha çok: küçülüyor, kötücül.",
    "Güneş: İkizler 5°00′ (tek burç, 1. dilim → Bala); lord Merkür nötr → Swapna; 3. ev; birlikte: Merkür; bakan: yok; burç lordu Merkür (nötr).",
    "Güneş durumları: hiçbiri.",
    "Ay: Boğa 10°00′ (çift burç, 2. dilim → Vriddha); yücelme burcu → Jagrat; 2. ev; birlikte: yok; bakan: yok; burç lordu Venüs (nötr).",
    "Ay durumları: Garvita.",
    "Mars: Başak 3°00′ (çift burç, 1. dilim → Mrita); lord Merkür düşman → Sushupti; 6. ev; birlikte: yok; bakan: yok; burç lordu Merkür (düşman).",
    "Mars durumları: Kshudhita.",
    "Merkür: İkizler 20°00′ (tek burç, 4. dilim → Vriddha); kendi burcu → Jagrat; 3. ev; birlikte: Güneş; bakan: yok; burç lordu Merkür (kendisi).",
    "Merkür durumları: Mudita (Güneş).",
    "Jüpiter: Aslan 10°00′ (tek burç, 2. dilim → Kumara); lord Güneş dost → Swapna; 5. ev; birlikte: Rahu; bakan: yok; burç lordu Güneş (dost).",
    "Jüpiter durumları: Lajjita (Rahu); Mudita.",
    "Venüs: Yengeç 15°00′ (çift burç, 3. dilim → Yuva); lord Ay düşman → Sushupti; 4. ev; birlikte: yok; bakan: Satürn; burç lordu Ay (düşman).",
    "Venüs durumları: Kshudhita; Mudita (Satürn).",
    "Satürn: Oğlak 12°00′ (çift burç, 3. dilim → Yuva); kendi burcu → Jagrat; 10. ev; birlikte: yok; bakan: Venüs; burç lordu Satürn (kendisi).",
    "Satürn durumları: Mudita (Venüs).",
    "Oluşmayan: Güneş Lajjita değil (Güneş Merkür ile birlikte ama 3. evde).",
    "Oluşmayan: Merkür Kshobhita değil (Merkür Güneş ile birlikte ama ona bakan yok).",
    "Oluşmayan: Venüs Trushita değil (Venüs su burcunda ve Satürn bakıyor, ama Satürn Venüs'ün dostudur).",
    "Oluşmayan: Satürn Garvita değil (Satürn kendi burcu Oğlak'ta; yücelme burcu da moolatrikona aralığı da değil)."
  ],
  "input": {
    "lagna_lon": 15.0,
    "grahas": {"Sun": 65.0, "Moon": 40.0, "Mars": 153.0, "Mercury": 80.0, "Jupiter": 130.0, "Venus": 105.0, "Saturn": 282.0, "Rahu": 145.0, "Ketu": 325.0}
  },
  "expected": {
    "baladi": {"Sun": "Bala", "Moon": "Vriddha", "Mars": "Mrita", "Mercury": "Vriddha", "Jupiter": "Kumara", "Venus": "Yuva", "Saturn": "Yuva"},
    "jagradadi": {"Sun": "Swapna", "Moon": "Jagrat", "Mars": "Sushupti", "Mercury": "Jagrat", "Jupiter": "Swapna", "Venus": "Sushupti", "Saturn": "Jagrat"},
    "lajjitadi": {
      "Sun": [],
      "Moon": [{"state": "Garvita", "grahas": []}],
      "Mars": [{"state": "Kshudhita", "grahas": []}],
      "Mercury": [{"state": "Mudita", "grahas": ["Sun"]}],
      "Jupiter": [{"state": "Lajjita", "grahas": ["Rahu"]}, {"state": "Mudita", "grahas": []}],
      "Venus": [{"state": "Kshudhita", "grahas": []}, {"state": "Mudita", "grahas": ["Saturn"]}],
      "Saturn": [{"state": "Mudita", "grahas": ["Venus"]}]
    },
    "absent": [{"graha": "Sun", "state": "Lajjita"}, {"graha": "Mercury", "state": "Kshobhita"}, {"graha": "Venus", "state": "Trushita"}, {"graha": "Saturn", "state": "Garvita"}]
  }
}
````

**Yeni dosya:** `tests/golden/modules/avastha/lajjitadi/venus-exalted-with-sun-five-states.json`

````json
{
  "id": "venus-exalted-with-sun-five-states",
  "source": "hand",
  "rule_ref": "BPHS, avasthalar bölümü: Baladi, Jagradadi ve Lajjitadi (alıntı yapılmaz); okuma kuralları avastha/REVIEW.md",
  "derivation": [
    "Lagna Akrep: 5. ev Balık. Venüs Balık'ta (yücelme) Güneş ile; Satürn Başak'tan karşıya bakar: Venüs beş durumdadır.",
    "Ay'ın sınıfı: uzanım = Ay − Güneş = 10.0 − 335.0 ≡ 35.0°; 180°'den küçük: büyüyor, iyicil.",
    "Güneş: Balık 5°00′ (çift burç, 1. dilim → Mrita); lord Jüpiter dost → Swapna; 5. ev; birlikte: Venüs; bakan: Satürn; burç lordu Jüpiter (dost).",
    "Güneş durumları: Kshudhita (Venüs, Satürn); Trushita (Satürn); Mudita.",
    "Ay: Koç 10°00′ (tek burç, 2. dilim → Kumara); lord Mars nötr → Swapna; 6. ev; birlikte: Mars; bakan: yok; burç lordu Mars (nötr).",
    "Ay durumları: hiçbiri.",
    "Mars: Koç 3°00′ (tek burç, 1. dilim → Bala); kendi burcu → Jagrat; 6. ev; birlikte: Ay; bakan: yok; burç lordu Mars (kendisi).",
    "Mars durumları: Garvita; Mudita (Ay).",
    "Merkür: Kova 20°00′ (tek burç, 4. dilim → Vriddha); lord Satürn nötr → Swapna; 4. ev; birlikte: yok; bakan: yok; burç lordu Satürn (nötr).",
    "Merkür durumları: hiçbiri.",
    "Jüpiter: Boğa 10°00′ (çift burç, 2. dilim → Vriddha); lord Venüs düşman → Sushupti; 7. ev; birlikte: yok; bakan: yok; burç lordu Venüs (düşman).",
    "Jüpiter durumları: Kshudhita.",
    "Venüs: Balık 15°00′ (çift burç, 3. dilim → Yuva); yücelme burcu → Jagrat; 5. ev; birlikte: Güneş; bakan: Satürn; burç lordu Jüpiter (nötr).",
    "Venüs durumları: Lajjita (Güneş); Garvita; Kshudhita (Güneş); Mudita (Satürn); Kshobhita (Güneş, Satürn).",
    "Satürn: Başak 12°00′ (çift burç, 3. dilim → Yuva); lord Merkür dost → Swapna; 11. ev; birlikte: yok; bakan: Güneş, Jüpiter, Venüs; burç lordu Merkür (dost).",
    "Satürn durumları: Kshudhita (Güneş); Mudita (Venüs).",
    "Oluşmayan: Venüs Trushita değil (Venüs su burcunda, bakan Satürn düşman değil).",
    "Oluşmayan: Ay Kshudhita değil (Ay'ın doğal düşmanı yok).",
    "Oluşmayan: Merkür Mudita değil (Kova'nın lordu Satürn Merkür'e nötr; yalnız ve bakışsız)."
  ],
  "input": {
    "lagna_lon": 225.0,
    "grahas": {"Sun": 335.0, "Moon": 10.0, "Mars": 3.0, "Mercury": 320.0, "Jupiter": 40.0, "Venus": 345.0, "Saturn": 162.0, "Rahu": 85.0, "Ketu": 265.0}
  },
  "expected": {
    "baladi": {"Sun": "Mrita", "Moon": "Kumara", "Mars": "Bala", "Mercury": "Vriddha", "Jupiter": "Vriddha", "Venus": "Yuva", "Saturn": "Yuva"},
    "jagradadi": {"Sun": "Swapna", "Moon": "Swapna", "Mars": "Jagrat", "Mercury": "Swapna", "Jupiter": "Sushupti", "Venus": "Jagrat", "Saturn": "Swapna"},
    "lajjitadi": {
      "Sun": [{"state": "Kshudhita", "grahas": ["Venus", "Saturn"]}, {"state": "Trushita", "grahas": ["Saturn"]}, {"state": "Mudita", "grahas": []}],
      "Moon": [],
      "Mars": [{"state": "Garvita", "grahas": []}, {"state": "Mudita", "grahas": ["Moon"]}],
      "Mercury": [],
      "Jupiter": [{"state": "Kshudhita", "grahas": []}],
      "Venus": [{"state": "Lajjita", "grahas": ["Sun"]}, {"state": "Garvita", "grahas": []}, {"state": "Kshudhita", "grahas": ["Sun"]}, {"state": "Mudita", "grahas": ["Saturn"]}, {"state": "Kshobhita", "grahas": ["Sun", "Saturn"]}],
      "Saturn": [{"state": "Kshudhita", "grahas": ["Sun"]}, {"state": "Mudita", "grahas": ["Venus"]}]
    },
    "absent": [{"graha": "Venus", "state": "Trushita"}, {"graha": "Moon", "state": "Kshudhita"}, {"graha": "Mercury", "state": "Mudita"}]
  }
}
````

**Yeni dosya:** `tests/golden/modules/avastha/lajjitadi/sun-in-cancer-trushita-by-saturn.json`

````json
{
  "id": "sun-in-cancer-trushita-by-saturn",
  "source": "hand",
  "rule_ref": "BPHS, avasthalar bölümü: Baladi, Jagradadi ve Lajjitadi (alıntı yapılmaz); okuma kuralları avastha/REVIEW.md",
  "derivation": [
    "Güneş Yengeç'te (su burcu); düşmanı Satürn Oğlak'tan karşıya bakar ve Yengeç'e bakan iyicil yok: Trushita.",
    "Ay'ın sınıfı: uzanım = Ay − Güneş = 40.0 − 95.0 ≡ 305.0°; 180° ya da daha çok: küçülüyor, kötücül.",
    "Güneş: Yengeç 5°00′ (çift burç, 1. dilim → Mrita); lord Ay dost → Swapna; 5. ev; birlikte: yok; bakan: Satürn; burç lordu Ay (dost).",
    "Güneş durumları: Kshudhita (Satürn); Trushita (Satürn); Mudita.",
    "Ay: Boğa 10°00′ (çift burç, 2. dilim → Vriddha); yücelme burcu → Jagrat; 3. ev; birlikte: Merkür, Jüpiter, Venüs; bakan: Mars; burç lordu Venüs (nötr).",
    "Ay durumları: Garvita; Mudita (Merkür, Jüpiter).",
    "Mars: Kova 3°00′ (tek burç, 1. dilim → Bala); lord Satürn nötr → Swapna; 12. ev; birlikte: yok; bakan: yok; burç lordu Satürn (nötr).",
    "Mars durumları: hiçbiri.",
    "Merkür: Boğa 20°00′ (çift burç, 4. dilim → Kumara); lord Venüs dost → Swapna; 3. ev; birlikte: Ay, Jüpiter, Venüs; bakan: Mars; burç lordu Venüs (dost).",
    "Merkür durumları: Kshudhita (Ay); Mudita (Jüpiter, Venüs).",
    "Jüpiter: Boğa 1°00′ (çift burç, 1. dilim → Mrita); lord Venüs düşman → Sushupti; 3. ev; birlikte: Ay, Merkür, Venüs; bakan: Mars; burç lordu Venüs (düşman).",
    "Jüpiter durumları: Kshudhita (Merkür, Venüs); Mudita (Ay, Mars).",
    "Venüs: Boğa 2°00′ (çift burç, 1. dilim → Mrita); kendi burcu → Jagrat; 3. ev; birlikte: Ay, Merkür, Jüpiter; bakan: Mars; burç lordu Venüs (kendisi).",
    "Venüs durumları: Kshudhita (Ay); Mudita (Merkür, Jüpiter).",
    "Satürn: Oğlak 12°00′ (çift burç, 3. dilim → Yuva); kendi burcu → Jagrat; 11. ev; birlikte: yok; bakan: Güneş, Jüpiter; burç lordu Satürn (kendisi).",
    "Satürn durumları: Kshudhita (Güneş).",
    "Oluşmayan: Güneş Lajjita değil (Güneş 5. evde ama yalnız).",
    "Oluşmayan: Güneş Kshobhita değil (Güneş'in kendisi Güneş ile birlikte sayılmaz).",
    "Oluşmayan: Jüpiter Kshobhita değil (Jüpiter Güneş ile birlikte değil)."
  ],
  "input": {
    "lagna_lon": 345.0,
    "grahas": {"Sun": 95.0, "Moon": 40.0, "Mars": 303.0, "Mercury": 50.0, "Jupiter": 31.0, "Venus": 32.0, "Saturn": 282.0, "Rahu": 175.0, "Ketu": 355.0}
  },
  "expected": {
    "baladi": {"Sun": "Mrita", "Moon": "Vriddha", "Mars": "Bala", "Mercury": "Kumara", "Jupiter": "Mrita", "Venus": "Mrita", "Saturn": "Yuva"},
    "jagradadi": {"Sun": "Swapna", "Moon": "Jagrat", "Mars": "Swapna", "Mercury": "Swapna", "Jupiter": "Sushupti", "Venus": "Jagrat", "Saturn": "Jagrat"},
    "lajjitadi": {
      "Sun": [{"state": "Kshudhita", "grahas": ["Saturn"]}, {"state": "Trushita", "grahas": ["Saturn"]}, {"state": "Mudita", "grahas": []}],
      "Moon": [{"state": "Garvita", "grahas": []}, {"state": "Mudita", "grahas": ["Mercury", "Jupiter"]}],
      "Mars": [],
      "Mercury": [{"state": "Kshudhita", "grahas": ["Moon"]}, {"state": "Mudita", "grahas": ["Jupiter", "Venus"]}],
      "Jupiter": [{"state": "Kshudhita", "grahas": ["Mercury", "Venus"]}, {"state": "Mudita", "grahas": ["Moon", "Mars"]}],
      "Venus": [{"state": "Kshudhita", "grahas": ["Moon"]}, {"state": "Mudita", "grahas": ["Mercury", "Jupiter"]}],
      "Saturn": [{"state": "Kshudhita", "grahas": ["Sun"]}]
    },
    "absent": [{"graha": "Sun", "state": "Lajjita"}, {"graha": "Sun", "state": "Kshobhita"}, {"graha": "Jupiter", "state": "Kshobhita"}]
  }
}
````

**Yeni dosya:** `tests/golden/modules/avastha/lajjitadi/trushita-cancelled-by-jupiter-aspect.json`

````json
{
  "id": "trushita-cancelled-by-jupiter-aspect",
  "source": "hand",
  "rule_ref": "BPHS, avasthalar bölümü: Baladi, Jagradadi ve Lajjitadi (alıntı yapılmaz); okuma kuralları avastha/REVIEW.md",
  "derivation": [
    "Önceki örneğin konumları, Jüpiter Balık'ta: 5. bakışı Yengeç'e düşer ve Trushita oluşmaz.",
    "Ay'ın sınıfı: uzanım = Ay − Güneş = 40.0 − 95.0 ≡ 305.0°; 180° ya da daha çok: küçülüyor, kötücül.",
    "Güneş: Yengeç 5°00′ (çift burç, 1. dilim → Mrita); lord Ay dost → Swapna; 8. ev; birlikte: yok; bakan: Jüpiter, Satürn; burç lordu Ay (dost).",
    "Güneş durumları: Kshudhita (Satürn); Mudita (Jüpiter).",
    "Ay: Boğa 10°00′ (çift burç, 2. dilim → Vriddha); yücelme burcu → Jagrat; 6. ev; birlikte: Merkür, Venüs; bakan: Mars; burç lordu Venüs (nötr).",
    "Ay durumları: Garvita; Mudita (Merkür).",
    "Mars: Kova 3°00′ (tek burç, 1. dilim → Bala); lord Satürn nötr → Swapna; 3. ev; birlikte: yok; bakan: yok; burç lordu Satürn (nötr).",
    "Mars durumları: hiçbiri.",
    "Merkür: Boğa 20°00′ (çift burç, 4. dilim → Kumara); lord Venüs dost → Swapna; 6. ev; birlikte: Ay, Venüs; bakan: Mars; burç lordu Venüs (dost).",
    "Merkür durumları: Kshudhita (Ay); Mudita (Venüs).",
    "Jüpiter: Balık 1°00′ (çift burç, 1. dilim → Mrita); kendi burcu → Jagrat; 4. ev; birlikte: Ketu; bakan: Satürn; burç lordu Jüpiter (kendisi).",
    "Jüpiter durumları: hiçbiri.",
    "Venüs: Boğa 2°00′ (çift burç, 1. dilim → Mrita); kendi burcu → Jagrat; 6. ev; birlikte: Ay, Merkür; bakan: Mars; burç lordu Venüs (kendisi).",
    "Venüs durumları: Kshudhita (Ay); Mudita (Merkür).",
    "Satürn: Oğlak 12°00′ (çift burç, 3. dilim → Yuva); kendi burcu → Jagrat; 2. ev; birlikte: yok; bakan: Güneş; burç lordu Satürn (kendisi).",
    "Satürn durumları: Kshudhita (Güneş).",
    "Oluşmayan: Güneş Trushita değil (Jüpiter Balık'tan 5. bakışla Yengeç'e bakar: iyicil bakış var).",
    "Oluşmayan: Jüpiter Lajjita değil (Jüpiter Ketu ile birlikte ama 4. evde).",
    "Oluşmayan: Jüpiter Kshudhita değil (bakan Satürn Jüpiter'e nötr; Ketu kimsenin düşmanı değil).",
    "Oluşmayan: Jüpiter Garvita değil (Balık kendi burcu; yücelme Yengeç, moolatrikona Yay 0–10°)."
  ],
  "input": {
    "lagna_lon": 255.0,
    "grahas": {"Sun": 95.0, "Moon": 40.0, "Mars": 303.0, "Mercury": 50.0, "Jupiter": 331.0, "Venus": 32.0, "Saturn": 282.0, "Rahu": 175.0, "Ketu": 355.0}
  },
  "expected": {
    "baladi": {"Sun": "Mrita", "Moon": "Vriddha", "Mars": "Bala", "Mercury": "Kumara", "Jupiter": "Mrita", "Venus": "Mrita", "Saturn": "Yuva"},
    "jagradadi": {"Sun": "Swapna", "Moon": "Jagrat", "Mars": "Swapna", "Mercury": "Swapna", "Jupiter": "Jagrat", "Venus": "Jagrat", "Saturn": "Jagrat"},
    "lajjitadi": {
      "Sun": [{"state": "Kshudhita", "grahas": ["Saturn"]}, {"state": "Mudita", "grahas": ["Jupiter"]}],
      "Moon": [{"state": "Garvita", "grahas": []}, {"state": "Mudita", "grahas": ["Mercury"]}],
      "Mars": [],
      "Mercury": [{"state": "Kshudhita", "grahas": ["Moon"]}, {"state": "Mudita", "grahas": ["Venus"]}],
      "Jupiter": [],
      "Venus": [{"state": "Kshudhita", "grahas": ["Moon"]}, {"state": "Mudita", "grahas": ["Mercury"]}],
      "Saturn": [{"state": "Kshudhita", "grahas": ["Sun"]}]
    },
    "absent": [{"graha": "Sun", "state": "Trushita"}, {"graha": "Jupiter", "state": "Lajjita"}, {"graha": "Jupiter", "state": "Kshudhita"}, {"graha": "Jupiter", "state": "Garvita"}]
  }
}
````

**Yeni dosya:** `tests/golden/modules/avastha/lajjitadi/mercury-with-sun-kshobhita-venus-in-no-state.json`

````json
{
  "id": "mercury-with-sun-kshobhita-venus-in-no-state",
  "source": "hand",
  "rule_ref": "BPHS, avasthalar bölümü: Baladi, Jagradadi ve Lajjitadi (alıntı yapılmaz); okuma kuralları avastha/REVIEW.md",
  "derivation": [
    "Merkür İkizler'de Güneş ile; Satürn ve Jüpiter Yay'dan karşıya bakar: Kshobhita. Venüs Akrep'te yalnız ve bakışsız: hiçbir durumda değil.",
    "Ay'ın sınıfı: uzanım = Ay − Güneş = 10.0 − 65.0 ≡ 305.0°; 180° ya da daha çok: küçülüyor, kötücül.",
    "Güneş: İkizler 5°00′ (tek burç, 1. dilim → Bala); lord Merkür nötr → Swapna; 5. ev; birlikte: Merkür; bakan: Jüpiter, Satürn; burç lordu Merkür (nötr).",
    "Güneş durumları: Kshudhita (Satürn); Mudita (Jüpiter).",
    "Ay: Koç 10°00′ (tek burç, 2. dilim → Kumara); lord Mars nötr → Swapna; 3. ev; birlikte: yok; bakan: Mars, Jüpiter; burç lordu Mars (nötr).",
    "Ay durumları: hiçbiri.",
    "Mars: Oğlak 28°00′ (çift burç, 5. dilim → Bala); yücelme burcu → Jagrat; 12. ev; birlikte: yok; bakan: yok; burç lordu Satürn (nötr).",
    "Mars durumları: Garvita.",
    "Merkür: İkizler 20°00′ (tek burç, 4. dilim → Vriddha); kendi burcu → Jagrat; 5. ev; birlikte: Güneş; bakan: Jüpiter, Satürn; burç lordu Merkür (kendisi).",
    "Merkür durumları: Lajjita (Güneş); Mudita (Güneş); Kshobhita (Güneş, Satürn).",
    "Jüpiter: Yay 5°00′ (tek burç, 1. dilim → Bala); kendi burcu → Jagrat; 11. ev; birlikte: Satürn; bakan: Güneş, Merkür; burç lordu Jüpiter (kendisi).",
    "Jüpiter durumları: Garvita; Kshudhita (Merkür, Satürn); Mudita (Güneş).",
    "Venüs: Akrep 9°00′ (çift burç, 2. dilim → Vriddha); lord Mars nötr → Swapna; 10. ev; birlikte: yok; bakan: yok; burç lordu Mars (nötr).",
    "Venüs durumları: hiçbiri.",
    "Satürn: Yay 12°00′ (tek burç, 3. dilim → Yuva); lord Jüpiter nötr → Swapna; 11. ev; birlikte: Jüpiter; bakan: Güneş, Merkür; burç lordu Jüpiter (nötr).",
    "Satürn durumları: Kshudhita (Güneş); Mudita (Merkür, Jüpiter).",
    "Oluşmayan: Venüs Trushita değil (Venüs su burcunda ama ona bakan yok).",
    "Oluşmayan: Venüs Mudita değil (Akrep'in lordu Mars Venüs'e nötr).",
    "Oluşmayan: Venüs Kshudhita değil (Akrep'in lordu Mars Venüs'e nötr; birlikte ve bakan yok).",
    "Oluşmayan: Ay Kshobhita değil (Ay Güneş ile birlikte değil).",
    "Oluşmayan: Güneş Kshobhita değil (Güneş'in kendisi)."
  ],
  "input": {
    "lagna_lon": 315.0,
    "grahas": {"Sun": 65.0, "Moon": 10.0, "Mars": 298.0, "Mercury": 80.0, "Jupiter": 245.0, "Venus": 219.0, "Saturn": 252.0, "Rahu": 145.0, "Ketu": 325.0}
  },
  "expected": {
    "baladi": {"Sun": "Bala", "Moon": "Kumara", "Mars": "Bala", "Mercury": "Vriddha", "Jupiter": "Bala", "Venus": "Vriddha", "Saturn": "Yuva"},
    "jagradadi": {"Sun": "Swapna", "Moon": "Swapna", "Mars": "Jagrat", "Mercury": "Jagrat", "Jupiter": "Jagrat", "Venus": "Swapna", "Saturn": "Swapna"},
    "lajjitadi": {
      "Sun": [{"state": "Kshudhita", "grahas": ["Saturn"]}, {"state": "Mudita", "grahas": ["Jupiter"]}],
      "Moon": [],
      "Mars": [{"state": "Garvita", "grahas": []}],
      "Mercury": [{"state": "Lajjita", "grahas": ["Sun"]}, {"state": "Mudita", "grahas": ["Sun"]}, {"state": "Kshobhita", "grahas": ["Sun", "Saturn"]}],
      "Jupiter": [{"state": "Garvita", "grahas": []}, {"state": "Kshudhita", "grahas": ["Mercury", "Saturn"]}, {"state": "Mudita", "grahas": ["Sun"]}],
      "Venus": [],
      "Saturn": [{"state": "Kshudhita", "grahas": ["Sun"]}, {"state": "Mudita", "grahas": ["Mercury", "Jupiter"]}]
    },
    "absent": [{"graha": "Venus", "state": "Trushita"}, {"graha": "Venus", "state": "Mudita"}, {"graha": "Venus", "state": "Kshudhita"}, {"graha": "Moon", "state": "Kshobhita"}, {"graha": "Sun", "state": "Kshobhita"}]
  }
}
````

**Yeni dosya:** `tests/golden/modules/avastha/lajjitadi/mercury-with-saturn-kshudhita-by-company.json`

````json
{
  "id": "mercury-with-saturn-kshudhita-by-company",
  "source": "hand",
  "rule_ref": "BPHS, avasthalar bölümü: Baladi, Jagradadi ve Lajjitadi (alıntı yapılmaz); okuma kuralları avastha/REVIEW.md",
  "derivation": [
    "Merkür Oğlak'ta Satürn ile birlikte: Satürn Merkür'e nötrdür ve Oğlak düşman burcu değildir, ama Satürn ile birliktelik Kshudhita'nın dördüncü koludur.",
    "Ay'ın sınıfı: uzanım = Ay − Güneş = 140.0 − 130.0 ≡ 10.0°; 180°'den küçük: büyüyor, iyicil.",
    "Güneş: Aslan 10°00′ (tek burç, 2. dilim → Kumara); kendi burcu → Jagrat; 1. ev; birlikte: Ay; bakan: yok; burç lordu Güneş (kendisi).",
    "Güneş durumları: Garvita; Mudita (Ay).",
    "Ay: Aslan 20°00′ (tek burç, 4. dilim → Vriddha); lord Güneş dost → Swapna; 1. ev; birlikte: Güneş; bakan: yok; burç lordu Güneş (dost).",
    "Ay durumları: Mudita (Güneş).",
    "Mars: Koç 20°00′ (tek burç, 4. dilim → Vriddha); kendi burcu → Jagrat; 9. ev; birlikte: yok; bakan: Venüs; burç lordu Mars (kendisi).",
    "Mars durumları: hiçbiri.",
    "Merkür: Oğlak 10°00′ (çift burç, 2. dilim → Vriddha); lord Satürn nötr → Swapna; 6. ev; birlikte: Satürn; bakan: yok; burç lordu Satürn (nötr).",
    "Merkür durumları: Kshudhita (Satürn).",
    "Jüpiter: Balık 20°00′ (çift burç, 4. dilim → Kumara); kendi burcu → Jagrat; 8. ev; birlikte: yok; bakan: Satürn; burç lordu Jüpiter (kendisi).",
    "Jüpiter durumları: hiçbiri.",
    "Venüs: Terazi 20°00′ (tek burç, 4. dilim → Vriddha); kendi burcu → Jagrat; 3. ev; birlikte: yok; bakan: Mars, Satürn; burç lordu Venüs (kendisi).",
    "Venüs durumları: Mudita (Satürn).",
    "Satürn: Oğlak 25°00′ (çift burç, 5. dilim → Bala); kendi burcu → Jagrat; 6. ev; birlikte: Merkür; bakan: yok; burç lordu Satürn (kendisi).",
    "Satürn durumları: Mudita (Merkür).",
    "Oluşmayan: Satürn Trushita değil (Satürn su burcunda değil).",
    "Oluşmayan: Venüs Garvita değil (Venüs Terazi 20°'de: kendi burcu, moolatrikona aralığı (0°–15°) dışında).",
    "Oluşmayan: Mars Garvita değil (Mars Koç 20°'de: kendi burcu, moolatrikona aralığı (0°–12°) dışında).",
    "Oluşmayan: Jüpiter Kshudhita değil (bakan Satürn Jüpiter'e nötr ve birlikte değil)."
  ],
  "input": {
    "lagna_lon": 135.0,
    "grahas": {"Sun": 130.0, "Moon": 140.0, "Mars": 20.0, "Mercury": 280.0, "Jupiter": 350.0, "Venus": 200.0, "Saturn": 295.0, "Rahu": 33.0, "Ketu": 213.0}
  },
  "expected": {
    "baladi": {"Sun": "Kumara", "Moon": "Vriddha", "Mars": "Vriddha", "Mercury": "Vriddha", "Jupiter": "Kumara", "Venus": "Vriddha", "Saturn": "Bala"},
    "jagradadi": {"Sun": "Jagrat", "Moon": "Swapna", "Mars": "Jagrat", "Mercury": "Swapna", "Jupiter": "Jagrat", "Venus": "Jagrat", "Saturn": "Jagrat"},
    "lajjitadi": {
      "Sun": [{"state": "Garvita", "grahas": []}, {"state": "Mudita", "grahas": ["Moon"]}],
      "Moon": [{"state": "Mudita", "grahas": ["Sun"]}],
      "Mars": [],
      "Mercury": [{"state": "Kshudhita", "grahas": ["Saturn"]}],
      "Jupiter": [],
      "Venus": [{"state": "Mudita", "grahas": ["Saturn"]}],
      "Saturn": [{"state": "Mudita", "grahas": ["Mercury"]}]
    },
    "absent": [{"graha": "Saturn", "state": "Trushita"}, {"graha": "Venus", "state": "Garvita"}, {"graha": "Mars", "state": "Garvita"}, {"graha": "Jupiter", "state": "Kshudhita"}]
  }
}
````

**Yeni dosya:** `tests/golden/modules/avastha/lajjitadi/time-unknown-fifth-house-not-evaluated.json`

````json
{
  "id": "time-unknown-fifth-house-not-evaluated",
  "source": "hand",
  "rule_ref": "BPHS, avasthalar bölümü: Baladi, Jagradadi ve Lajjitadi (alıntı yapılmaz); okuma kuralları avastha/REVIEW.md",
  "derivation": [
    "Doğum saati bilinmiyor: lagna yok. Lajjita değerlendirilmez; öteki beş kural ve Baladi ile Jagradadi lagnaya bağlı değildir.",
    "Ay'ın sınıfı: uzanım = Ay − Güneş = 56.0 − 87.0 ≡ 329.0°; 180° ya da daha çok: küçülüyor, kötücül.",
    "Güneş: İkizler 27°00′ (tek burç, 5. dilim → Mrita); lord Merkür nötr → Swapna; ev yok (lagna yok); birlikte: Merkür; bakan: yok; burç lordu Merkür (nötr).",
    "Güneş durumları: hiçbiri.",
    "Ay: Boğa 26°00′ (çift burç, 5. dilim → Bala); yücelme burcu → Jagrat; ev yok (lagna yok); birlikte: yok; bakan: yok; burç lordu Venüs (nötr).",
    "Ay durumları: Garvita.",
    "Mars: Başak 20°00′ (çift burç, 4. dilim → Kumara); lord Merkür düşman → Sushupti; ev yok (lagna yok); birlikte: yok; bakan: yok; burç lordu Merkür (düşman).",
    "Mars durumları: Kshudhita.",
    "Merkür: İkizler 14°00′ (tek burç, 3. dilim → Yuva); kendi burcu → Jagrat; ev yok (lagna yok); birlikte: Güneş; bakan: yok; burç lordu Merkür (kendisi).",
    "Merkür durumları: Mudita (Güneş).",
    "Jüpiter: Aslan 10°00′ (tek burç, 2. dilim → Kumara); lord Güneş dost → Swapna; ev yok (lagna yok); birlikte: Rahu; bakan: yok; burç lordu Güneş (dost).",
    "Jüpiter durumları: Mudita.",
    "Venüs: Yengeç 8°00′ (çift burç, 2. dilim → Vriddha); lord Ay düşman → Sushupti; ev yok (lagna yok); birlikte: yok; bakan: Satürn; burç lordu Ay (düşman).",
    "Venüs durumları: Kshudhita; Mudita (Satürn).",
    "Satürn: Oğlak 12°00′ (çift burç, 3. dilim → Yuva); kendi burcu → Jagrat; ev yok (lagna yok); birlikte: yok; bakan: Venüs; burç lordu Satürn (kendisi).",
    "Satürn durumları: Mudita (Venüs).",
    "Oluşmayan: Jüpiter Lajjita değil (lagna yok: ev bilinmiyor, Rahu ile birliktelik tek başına yetmez).",
    "Oluşmayan: Mars Mudita değil (Başak'ın lordu Merkür Mars'ın düşmanı)."
  ],
  "input": {
    "lagna_lon": null,
    "grahas": {"Sun": 87.0, "Moon": 56.0, "Mars": 170.0, "Mercury": 74.0, "Jupiter": 130.0, "Venus": 98.0, "Saturn": 282.0, "Rahu": 145.0, "Ketu": 325.0}
  },
  "expected": {
    "baladi": {"Sun": "Mrita", "Moon": "Bala", "Mars": "Kumara", "Mercury": "Yuva", "Jupiter": "Kumara", "Venus": "Vriddha", "Saturn": "Yuva"},
    "jagradadi": {"Sun": "Swapna", "Moon": "Jagrat", "Mars": "Sushupti", "Mercury": "Jagrat", "Jupiter": "Swapna", "Venus": "Sushupti", "Saturn": "Jagrat"},
    "lajjitadi": {
      "Sun": [],
      "Moon": [{"state": "Garvita", "grahas": []}],
      "Mars": [{"state": "Kshudhita", "grahas": []}],
      "Mercury": [{"state": "Mudita", "grahas": ["Sun"]}],
      "Jupiter": [{"state": "Mudita", "grahas": []}],
      "Venus": [{"state": "Kshudhita", "grahas": []}, {"state": "Mudita", "grahas": ["Saturn"]}],
      "Saturn": [{"state": "Mudita", "grahas": ["Venus"]}]
    },
    "absent": [{"graha": "Jupiter", "state": "Lajjita"}, {"graha": "Mars", "state": "Mudita"}]
  }
}
````

- [ ] **Step 4: Testler ve kapılar**

```bash
ls tests/golden/modules/avastha/lajjitadi | wc -l
cargo fmt --all --check
cargo test -p jyotish-core --test avastha_golden 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: `7`; fmt çıktısız; `avastha_golden` `3 passed`; lib `417 passed`, golden `8 passed`, `modules_golden` `17 passed`; clippy çıktısız; sayaç `0`. **Bir örnek motorla uyuşmazsa** ne JSON ne motor hemen değiştirilir: önce `derivation`'ın o satırı D5–D7'den yeniden türetilir. Dosya bu plandaki metinden farklı yazılmışsa (kopyalama hatası) dosya düzeltilir; metin aynıysa ve türetme motoru haklı çıkarıyorsa bulgu ilerleme notlarına yazılır ve görev durur (planın örneği yanlıştır: sahibine bildirilir). Beklenen değer motor çıktısına bakarak değiştirilmez.

- [ ] **Step 5: Commit**

```bash
git add crates/jyotish-core/tests/avastha_golden.rs tests/golden/modules/avastha/lajjitadi
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(core): Lajjitadi hand examples with present and absent states" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: sekiz dosya (`571 insertions`); tarama çıktısız; `git status --short` boş.

---

### Task 4: Çekirdek: Shayanadi formülü, avastha raporu ve toplayıcı

Yol haritası §4.4 ("Shayanadi") ve §5 Plan 24 görev (4): "Shayanadi formülü ve toplayıcı (gün doğumu, ghati)"; K1–K3, K5; D8–D10. Formülün ikinci yazımı (bölmesiz yürüme) bu görevin birim testidir; harita düzeyindeki el örnekleri Task 5'tedir. `chart.rs`, `panchang.rs` ve `positions.rs` yalnızca okunur: gün doğumu haritanın zaten hesapladığı `panchang.sunrise_jd_ut`'tur.

**Files:**
- Create: `crates/jyotish-core/src/avastha/shayanadi.rs`, `crates/jyotish-core/src/avastha/collect.rs`
- Modify: `crates/jyotish-core/src/avastha/mod.rs`
- Test: `shayanadi.rs` `mod tests` (7), `collect.rs` `mod tests` (5)

**Interfaces:**
- Consumes (hepsi var): crate içi `chart::{compute_chart_for, ChartWarning}` (`Chart.panchang.sunrise_jd_ut`, `Chart.meta.warnings`); `positions::{ChartInput, Positions}`; `zodiac::nakshatra_pada`; `ephemeris::Ephemeris`; testte `fake::FakeEph`, `chart::compute_chart`, `settings::{CalcSettings, ZodiacMode}`. Task 1–2'nin `baladi`, `jagradadi`, `lajjitadi`, `LajjitadiHit`, `AVASTHA_GRAHAS`, `AVASTHA_METHOD`'u.
- Produces (`jyotish_core::avastha`):
  - `Shayanadi` (on iki tür; `Shayanadi::ALL` artan sırasıdır: artan 1 `Shayana` … artan 11 `Kautuka`, artan 0 `Nidra`).
  - `ShayanadiBasis { birth_nakshatra: u8, ghatis: u32, lagna_sign_number: u8 }`, `ShayanadiHit { state, nakshatra: u8, graha_number: u8, navamsa: u8, total: u32, remainder: u8 }`.
  - `graha_number(graha) -> Option<u8>`, `navamsa_number(deg_in_sign: f64) -> u8`, `elapsed_ghatis(jd_ut: f64, sunrise_jd_ut: f64) -> Option<u32>`, `shayanadi_total(nakshatra, graha_number, navamsa, &ShayanadiBasis) -> u32`, `shayanadi_state(total: u32) -> Shayanadi`, `shayanadi_basis(&Positions, ghatis: Option<u32>) -> Option<ShayanadiBasis>`, `shayanadi(graha, &Positions, &ShayanadiBasis) -> Option<ShayanadiHit>`.
  - Sabitler: `GHATIS_PER_DAY: f64 = 60.0`, `NAVAMSA_SPAN_DEG: f64 = 30.0 / 9.0`.
  - `GrahaAvastha { graha, baladi, jagradadi, lajjitadi: Vec<LajjitadiHit>, shayanadi: Option<ShayanadiHit> }`, `AvasthaReport { grahas: Vec<GrahaAvastha>, shayanadi_basis: Option<ShayanadiBasis>, time_known: bool, warnings: Vec<ChartWarning>, method: String }`.
  - `avasthas(positions: &Positions, ghatis: Option<u32>) -> AvasthaReport` (saf), `compute_avasthas<E: Ephemeris + ?Sized>(eph: &mut E, input: &ChartInput) -> Result<AvasthaReport, CoreError>` (toplayıcı).

- [ ] **Step 1: Testler: terimler, ghati, formül ve ikinci yazım; rapor ve toplayıcı; modül kaydı**

`shayanadi.rs` testleri: graha sayıları; navamsa dilimi sınırları; tamamlanmış ghati (24 dakikanın altı 0, gün doğumundan önce `None`); formülün el örneği ve artan sırası; **ikinci yazım** (bölme yok: durum çemberinde, çarpım tekrarlı toplama olarak, adım adım yürünür; 27 × 7 × 9 çarpım ve 98 toplanan değeri için formülle aynı durum); ortak terimlerin lagna ve ghati istemesi; her grahanın kendi üç teriminden durumu. `collect.rs` testleri: raporun yedi satırı ve dört dizisi; lagnasız ve gün doğumsuz raporda Shayanadi'nin yokluğu ve öteki dizilerin aynılığı; toplayıcının sahte ephemeris'te son gün doğumundan ghati sayması (türetme yorumda); `time_known`, tropikal-rasi kipi ve kutup günü; sayı prashnası, düşük hassasiyet uyarısı ve hatalar.

````diff
diff --git a/crates/jyotish-core/src/avastha/mod.rs b/crates/jyotish-core/src/avastha/mod.rs
index 8a7eb40..d992946 100644
--- a/crates/jyotish-core/src/avastha/mod.rs
+++ b/crates/jyotish-core/src/avastha/mod.rs
@@ -10,13 +10,20 @@
 //! yoktur: modül yalnızca sınıfı ve kuralı sağlayan grahaları döner.
 
 mod baladi;
+mod collect;
 mod lajjitadi;
+mod shayanadi;
 
 pub use baladi::{baladi, jagradadi, Baladi, Jagradadi, BALADI_SPAN_DEG};
+pub use collect::{avasthas, compute_avasthas, AvasthaReport, GrahaAvastha};
 pub use lajjitadi::{
     aspecting_grahas, conjunct_grahas, is_garvita, lajjitadi, Lajjitadi, LajjitadiHit,
     LAJJITA_COMPANIONS, LAJJITA_HOUSE, WATER_SIGNS,
 };
+pub use shayanadi::{
+    elapsed_ghatis, graha_number, navamsa_number, shayanadi, shayanadi_basis, shayanadi_state,
+    shayanadi_total, Shayanadi, ShayanadiBasis, ShayanadiHit, GHATIS_PER_DAY, NAVAMSA_SPAN_DEG,
+};
 
 use crate::drishti::ASPECTING_GRAHAS;
 use crate::graha::Graha;
````

**Yeni dosya:** `crates/jyotish-core/src/avastha/shayanadi.rs` (şimdilik yalnızca test modülü; uygulama sonraki adımda üstüne eklenir)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::avastha::AVASTHA_GRAHAS;
    use Graha::*;
    use Shayanadi::*;

    fn basis(birth_nakshatra: u8, ghatis: u32, lagna_sign_number: u8) -> ShayanadiBasis {
        ShayanadiBasis {
            birth_nakshatra,
            ghatis,
            lagna_sign_number,
        }
    }

    #[test]
    fn graha_numbers_follow_the_weekday_order_and_nodes_have_none() {
        let numbers: Vec<_> = Graha::ALL.into_iter().map(graha_number).collect();
        assert_eq!(
            numbers,
            [
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(6),
                Some(7),
                None,
                None
            ]
        );
    }

    #[test]
    fn navamsa_number_is_the_ninth_of_the_sign() {
        for (deg, want) in [
            (0.0, 1),
            (3.3, 1),
            (10.0 / 3.0, 2),
            (6.6, 2),
            (20.0 / 3.0, 3),
            (10.0, 4),
            (15.0, 5),
            (16.7, 6),
            (20.0, 7),
            (23.4, 8),
            (26.7, 9),
            (29.999, 9),
            (30.0, 9),
            (-0.1, 1),
        ] {
            assert_eq!(navamsa_number(deg), want, "{deg}");
        }
    }

    #[test]
    fn elapsed_ghatis_are_completed_ghatis_of_twenty_four_minutes() {
        let rise = 2_451_544.75; // 06:00 UT
        let minutes = |m: f64| rise + m / 1440.0;
        assert_eq!(elapsed_ghatis(rise, rise), Some(0));
        assert_eq!(elapsed_ghatis(minutes(23.9), rise), Some(0));
        assert_eq!(elapsed_ghatis(minutes(24.1), rise), Some(1));
        // 06:00'dan 17:30'a 690 dk = 28,75 ghati: tamamlanan 28.
        assert_eq!(elapsed_ghatis(minutes(690.0), rise), Some(28));
        // Gün doğumundan hemen önce doğan, önceki gün doğumundan sayar: 59 tamamlanmış ghati.
        assert_eq!(elapsed_ghatis(minutes(1439.0), rise), Some(59));
        assert_eq!(elapsed_ghatis(minutes(-1.0), rise), None);
        assert_eq!(elapsed_ghatis(f64::NAN, rise), None);
        assert_eq!(elapsed_ghatis(rise, f64::INFINITY), None);
    }

    #[test]
    fn the_formula_multiplies_three_terms_and_adds_three() {
        // El örneği: N = 4, G = 1, A = 5 → 20; J = 10, Gh = 28, L = 3 → 41; toplam 61.
        // 61 = 5 × 12 + 1: artan 1, ilk durum.
        let b = basis(10, 28, 3);
        assert_eq!(shayanadi_total(4, 1, 5, &b), 61);
        assert_eq!(shayanadi_state(61), Shayana);
        // Artan sırası: 1 Shayana, 2 Upaveshana, …, 11 Kautuka, 0 Nidra.
        let by_remainder: Vec<_> = (1..=12).map(shayanadi_state).collect();
        assert_eq!(by_remainder, Shayanadi::ALL);
        assert_eq!(shayanadi_state(12), Nidra);
        assert_eq!(shayanadi_state(24), Nidra);
        assert_eq!(shayanadi_state(0), Nidra);
        assert_eq!(shayanadi_state(13), Shayana);
        assert_eq!(shayanadi_state(71), Kautuka); // 71 = 5 × 12 + 11
        assert_eq!(shayanadi_state(57), Bhojana); // 57 = 4 × 12 + 9
    }

    #[test]
    fn second_writing_counting_through_the_cycle_agrees_for_every_input() {
        // İkinci yazım: bölme yok. Durum çemberinde her terim için adım adım yüründüğünde
        // (çarpım, tekrarlı toplama olarak) varılan yer formülün verdiği durumdur. İlk adım
        // ilk duruma varır: başlangıç on ikinci durumdur.
        fn walk(steps: [u32; 4], product: [u32; 3]) -> Shayanadi {
            let mut at = 11_usize; // Nidra: bir adım sonrası Shayana
            let mut step = |n: u32| {
                for _ in 0..n {
                    at = if at == 11 { 0 } else { at + 1 };
                }
            };
            for _ in 0..product[1] {
                for _ in 0..product[2] {
                    step(product[0]);
                }
            }
            for n in steps {
                step(n);
            }
            Shayanadi::ALL[at]
        }
        for nakshatra in 1..=27_u8 {
            for graha in 1..=7_u8 {
                for navamsa in 1..=9_u8 {
                    // Üç toplanan yalnız toplamıyla etki eder: 1 + 0 + 1 = 2 … 27 + 60 + 12 = 99.
                    for addend in 2..=99_u32 {
                        let b = basis(1, addend - 2, 1);
                        let total = shayanadi_total(nakshatra, graha, navamsa, &b);
                        assert_eq!(
                            shayanadi_state(total),
                            walk(
                                [1, addend - 2, 1, 0],
                                [nakshatra.into(), graha.into(), navamsa.into()]
                            ),
                            "{nakshatra} {graha} {navamsa} {addend}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn basis_needs_the_lagna_and_the_ghatis() {
        // Ay 100° → nakshatra 8 (93°20′–106°40′); lagna 70° İkizler → 3.
        let lons = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];
        let p = Positions::synthetic(Some(70.0), lons);
        assert_eq!(shayanadi_basis(&p, Some(28)), Some(basis(8, 28, 3)));
        assert_eq!(shayanadi_basis(&p, None), None);
        let unknown = p.clone().without_lagna();
        assert_eq!(shayanadi_basis(&unknown, Some(28)), None);
    }

    #[test]
    fn each_graha_gets_its_state_from_its_own_three_terms() {
        // Güneş 10° Koç: nakshatra 1 (0°–13°20′), navamsa 4 (10°–13°20′), G = 1 → 4.
        // Jüpiter 250° Yay 10°: nakshatra 19 (240°–253°20′), navamsa 4, G = 5 → 380.
        // Ay 100° Yengeç 10°: nakshatra 8, navamsa 4, G = 2 → 64.
        let lons = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];
        let p = Positions::synthetic(Some(70.0), lons);
        let b = basis(8, 28, 3); // J + Gh + L = 39
        let sun = shayanadi(Sun, &p, &b).expect("sun");
        assert_eq!(
            (sun.nakshatra, sun.graha_number, sun.navamsa, sun.total),
            (1, 1, 4, 43)
        );
        assert_eq!((sun.state, sun.remainder), (Sabha, 7)); // 43 = 3 × 12 + 7
        let jupiter = shayanadi(Jupiter, &p, &b).expect("jupiter");
        assert_eq!(
            (jupiter.nakshatra, jupiter.navamsa, jupiter.total),
            (19, 4, 419)
        );
        assert_eq!((jupiter.state, jupiter.remainder), (Kautuka, 11)); // 419 = 34 × 12 + 11
        let moon = shayanadi(Moon, &p, &b).expect("moon");
        assert_eq!((moon.nakshatra, moon.navamsa, moon.total), (8, 4, 103));
        assert_eq!(moon.state, Sabha); // 103 = 8 × 12 + 7
        assert_eq!(shayanadi(Rahu, &p, &b), None);
        assert_eq!(shayanadi(Ketu, &p, &b), None);
        for graha in AVASTHA_GRAHAS {
            let hit = shayanadi(graha, &p, &b).expect("seven grahas");
            assert_eq!(hit.state, shayanadi_state(hit.total), "{graha:?}");
            assert_eq!(u32::from(hit.remainder), hit.total % 12, "{graha:?}");
        }
    }
}
````

**Yeni dosya:** `crates/jyotish-core/src/avastha/collect.rs` (şimdilik yalnızca test modülü; uygulama Step 4'te üstüne eklenir)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::avastha::{Lajjitadi, Shayanadi};
    use crate::chart::compute_chart;
    use crate::fake::FakeEph;
    use crate::settings::{CalcSettings, ZodiacMode};
    use crate::zodiac::Sign;
    use ephemeris::{Body, Precision};

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

    #[test]
    fn the_report_lists_the_seven_grahas_with_all_four_series() {
        // Lagna 70° İkizler; Ay 100° (nakshatra 8).
        let lons = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];
        let p = Positions::synthetic(Some(70.0), lons);
        let report = avasthas(&p, Some(28));
        assert_eq!(report.method, AVASTHA_METHOD);
        assert!(report.time_known && report.warnings.is_empty());
        let order: Vec<_> = report.grahas.iter().map(|g| g.graha).collect();
        assert_eq!(order, AVASTHA_GRAHAS);
        let basis = report.shayanadi_basis.expect("basis");
        assert_eq!(
            (basis.birth_nakshatra, basis.ghatis, basis.lagna_sign_number),
            (8, 28, 3)
        );
        // Güneş Koç 10°: tek burç, ikinci dilim; yücelme burcu.
        let sun = &report.grahas[0];
        assert_eq!(
            (sun.baladi, sun.jagradadi),
            (Baladi::Kumara, Jagradadi::Jagrat)
        );
        assert!(sun.lajjitadi.iter().any(|h| h.state == Lajjitadi::Garvita));
        assert_eq!(sun.shayanadi.map(|s| s.state), Some(Shayanadi::Sabha));
        for row in &report.grahas {
            assert!(row.shayanadi.is_some(), "{:?}", row.graha);
        }
    }

    #[test]
    fn without_a_lagna_or_a_sunrise_there_is_no_shayanadi_and_the_rest_stands() {
        let lons = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];
        let full = avasthas(&Positions::synthetic(Some(70.0), lons), Some(28));
        // Gün doğumu yok: lagna var, Shayanadi yok.
        let polar = avasthas(&Positions::synthetic(Some(70.0), lons), None);
        assert!(polar.time_known);
        assert_eq!(polar.shayanadi_basis, None);
        // Doğum saati bilinmiyor: ghati verilse de Shayanadi yok.
        let unknown = avasthas(&Positions::synthetic(None, lons), Some(28));
        assert!(!unknown.time_known);
        assert_eq!(unknown.shayanadi_basis, None);
        for (other, name) in [(&polar, "polar"), (&unknown, "unknown")] {
            for (row, base) in other.grahas.iter().zip(&full.grahas) {
                assert_eq!(row.shayanadi, None, "{name} {:?}", row.graha);
                assert_eq!(
                    (row.baladi, row.jagradadi),
                    (base.baladi, base.jagradadi),
                    "{name}"
                );
            }
        }
        // Lajjita lagnaya bağlıdır; öteki beş durum değildir.
        for (row, base) in unknown.grahas.iter().zip(&full.grahas) {
            let rest: Vec<_> = base
                .lajjitadi
                .iter()
                .filter(|h| h.state != Lajjitadi::Lajjita)
                .cloned()
                .collect();
            assert_eq!(row.lajjitadi, rest, "{:?}", row.graha);
        }
        assert_eq!(
            polar
                .grahas
                .iter()
                .map(|g| &g.lajjitadi)
                .collect::<Vec<_>>(),
            full.grahas.iter().map(|g| &g.lajjitadi).collect::<Vec<_>>()
        );
    }

    #[test]
    fn collector_reads_the_chart_and_counts_ghatis_from_the_last_sunrise() {
        // FakeEph: gün doğumu her gün n + 0,67; doğum 2 451 545,0 → son doğuş 2 451 544,67.
        // Geçen süre 0,33 gün = 19,8 ghati: tamamlanan 19. Lagna tropikal 70°, sidereal 46°
        // (Boğa: 2). Ay tropikal 95°, sidereal 71°: nakshatra 6 (66°40′–80°).
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        let report = compute_avasthas(&mut eph, &input(s.clone(), true)).unwrap();
        let basis = report.shayanadi_basis.expect("basis");
        assert_eq!(
            (basis.birth_nakshatra, basis.ghatis, basis.lagna_sign_number),
            (6, 19, 2)
        );
        let chart = compute_chart(&mut eph, JD, 41.0, 29.0, &s).unwrap();
        assert_eq!(report, avasthas(&Positions::from(&chart), Some(19)));
        // Güneş sidereal 346° (Balık 16°): nakshatra 26 (333°20′–346°40′), navamsa 5.
        // 26 × 1 × 5 + 6 + 19 + 2 = 157 = 13 × 12 + 1.
        let sun = report.grahas[0].shayanadi.expect("sun");
        assert_eq!((sun.nakshatra, sun.navamsa, sun.total), (26, 5, 157));
        assert_eq!((sun.state, sun.remainder), (Shayanadi::Shayana, 1));
    }

    #[test]
    fn collector_follows_time_known_the_rasi_frame_and_the_polar_day() {
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        let known = compute_avasthas(&mut eph, &input(s.clone(), true)).unwrap();
        let unknown = compute_avasthas(&mut eph, &input(s.clone(), false)).unwrap();
        assert!(!unknown.time_known);
        assert_eq!(unknown.shayanadi_basis, None);
        assert!(unknown.grahas.iter().all(|g| g.shayanadi.is_none()));
        assert_eq!(unknown.grahas.len(), 7);
        for (a, b) in unknown.grahas.iter().zip(&known.grahas) {
            assert_eq!((a.baladi, a.jagradadi), (b.baladi, b.jagradadi));
        }
        // Kutup günü: gün doğumu yok; lagna var, Shayanadi yok.
        let mut polar = FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        let report = compute_avasthas(&mut polar, &input(s.clone(), true)).unwrap();
        assert!(report.time_known);
        assert_eq!(report.shayanadi_basis, None);
        assert_eq!(
            report
                .grahas
                .iter()
                .map(|g| &g.lajjitadi)
                .collect::<Vec<_>>(),
            known
                .grahas
                .iter()
                .map(|g| &g.lajjitadi)
                .collect::<Vec<_>>()
        );
        // Tropikal-rasi kipi: burç ve burç içi derece tropikal çerçevedendir (Güneş Koç 10°:
        // yücelme burcu, ikinci dilim); nakshatra sidereal boylamdandır (346° → 26).
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        let report = compute_avasthas(&mut eph, &input(tropical, true)).unwrap();
        let sun = &report.grahas[0];
        assert_eq!(
            (sun.baladi, sun.jagradadi),
            (Baladi::Kumara, Jagradadi::Jagrat)
        );
        let hit = sun.shayanadi.expect("sun");
        assert_eq!((hit.nakshatra, hit.navamsa), (26, 4));
        // Lagna tropikal 70°: İkizler (3).
        assert_eq!(
            report.shayanadi_basis.map(|b| b.lagna_sign_number),
            Some(Sign::Gemini.index() as u8 + 1)
        );
    }

    #[test]
    fn a_lagna_number_gives_the_lagna_term_and_errors_are_reported_without_a_birth_time() {
        let mut eph = FakeEph::default();
        // 30. sayı: lagna Yengeç (4).
        let mut numbered = input(CalcSettings::default(), true);
        numbered.lagna_number = Some(30);
        let report = compute_avasthas(&mut eph, &numbered).unwrap();
        assert_eq!(report.shayanadi_basis.map(|b| b.lagna_sign_number), Some(4));
        // Düşük hassasiyet uyarısı rapora taşınır.
        let mut low = FakeEph {
            precision: Precision::Moshier,
            ..FakeEph::default()
        };
        let report = compute_avasthas(&mut low, &input(CalcSettings::default(), true)).unwrap();
        assert_eq!(report.warnings, [ChartWarning::LowPrecision]);
        // Hatalar saat bilinmese de bildirilir.
        let mut bad = input(CalcSettings::default(), false);
        bad.lat = 91.0;
        assert_eq!(
            compute_avasthas(&mut eph, &bad),
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
            compute_avasthas(&mut eph, &input(old, false)),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 1,
                supported: 2
            })
        );
        // Sahte ephemeris'te cisim yoksa hata olduğu gibi döner.
        let mut missing = FakeEph::default();
        missing.positions.retain(|p| p.0 != Body::Moon);
        assert!(compute_avasthas(&mut missing, &input(CalcSettings::default(), true)).is_err());
    }
}
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-core --lib avastha 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası (`error[E0432]: unresolved imports` `collect::…` ve `shayanadi::…`). Test koşmaz.

- [ ] **Step 3: Shayanadi: terimler, ghati ve formül**

`shayanadi.rs`'in **başına**, test modülünün üstüne eklenir (arada bir boş satır):

````rust
//! Shayanadi avasthaları: on iki durum, tek bir sayma formülünden.
//!
//! Bu dosyadaki okuma (varyant noktaları `tests/golden/modules/avastha/REVIEW.md`'dedir):
//! `toplam = N × G × A + J + Gh + L`. Durum, toplamın 12'ye bölümünden artanın gösterdiği
//! durumdur; artan 0 ise on ikinci durumdur.
//!
//! - `N`: grahanın nakshatrası, Ashwini'den 1..=27 (sidereal boylamdan).
//! - `G`: grahanın sayısı: Güneş 1, Ay 2, Mars 3, Merkür 4, Jüpiter 5, Venüs 6, Satürn 7.
//! - `A`: grahanın bulunduğu navamsa diliminin burç içindeki sırası, 1..=9 (rasi çerçevesindeki
//!   burç içi dereceden; dilim 3°20′).
//! - `J`: doğum nakshatrası (Ay'ın nakshatrası), 1..=27.
//! - `Gh`: gün doğumundan doğuma **tamamlanmış** ghati sayısı (bir ghati 24 dakikadır).
//! - `L`: lagnanın rasi burcunun sırası, Koç 1 … Balık 12.
//!
//! Lagna ya da gün doğumu yoksa durum hesaplanmaz. Üç alt durum (cheshta) kapsam dışıdır.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::positions::Positions;
use crate::zodiac::nakshatra_pada;

/// Bir günde ghati sayısı: bir ghati 24 dakikadır.
pub const GHATIS_PER_DAY: f64 = 60.0;

/// Navamsa diliminin genişliği, derece (3°20′).
pub const NAVAMSA_SPAN_DEG: f64 = 30.0 / 9.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Shayanadi {
    Shayana,
    Upaveshana,
    Netrapani,
    Prakashana,
    Gamana,
    Agamana,
    Sabha,
    Agama,
    Bhojana,
    NrityaLipsa,
    Kautuka,
    Nidra,
}

impl Shayanadi {
    /// Artan sırası: artan 1 `Shayana`, …, artan 11 `Kautuka`, artan 0 `Nidra`.
    pub const ALL: [Shayanadi; 12] = [
        Shayanadi::Shayana,
        Shayanadi::Upaveshana,
        Shayanadi::Netrapani,
        Shayanadi::Prakashana,
        Shayanadi::Gamana,
        Shayanadi::Agamana,
        Shayanadi::Sabha,
        Shayanadi::Agama,
        Shayanadi::Bhojana,
        Shayanadi::NrityaLipsa,
        Shayanadi::Kautuka,
        Shayanadi::Nidra,
    ];
}

/// Formülün haritaya ait üç terimi: her graha için aynıdır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ShayanadiBasis {
    /// Doğum nakshatrası (Ay), 1..=27.
    pub birth_nakshatra: u8,
    /// Gün doğumundan doğuma tamamlanmış ghati.
    pub ghatis: u32,
    /// Lagnanın rasi burcunun sırası, 1..=12.
    pub lagna_sign_number: u8,
}

/// Bir grahanın Shayanadi durumu ve formülün o grahaya ait terimleri.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ShayanadiHit {
    pub state: Shayanadi,
    /// Grahanın nakshatrası, 1..=27.
    pub nakshatra: u8,
    /// Grahanın sayısı, 1..=7.
    pub graha_number: u8,
    /// Navamsa diliminin burç içindeki sırası, 1..=9.
    pub navamsa: u8,
    /// `N × G × A + J + Gh + L`.
    pub total: u32,
    /// Toplamın 12'ye bölümünden artan, 0..=11; 0 on ikinci durumdur.
    pub remainder: u8,
}

/// Grahanın formüldeki sayısı; düğümler için `None`.
pub fn graha_number(graha: Graha) -> Option<u8> {
    match graha {
        Graha::Sun => Some(1),
        Graha::Moon => Some(2),
        Graha::Mars => Some(3),
        Graha::Mercury => Some(4),
        Graha::Jupiter => Some(5),
        Graha::Venus => Some(6),
        Graha::Saturn => Some(7),
        Graha::Rahu | Graha::Ketu => None,
    }
}

/// Burç içi derecenin navamsa dilimi, 1..=9. Dilim sınırı sonraki dilime aittir.
pub fn navamsa_number(deg_in_sign: f64) -> u8 {
    (deg_in_sign / NAVAMSA_SPAN_DEG).floor().clamp(0.0, 8.0) as u8 + 1
}

/// Gün doğumundan `jd_ut` anına tamamlanmış ghati sayısı. An gün doğumundan önceyse ya da
/// değerlerden biri sonlu değilse `None`.
pub fn elapsed_ghatis(jd_ut: f64, sunrise_jd_ut: f64) -> Option<u32> {
    let ghatis = ((jd_ut - sunrise_jd_ut) * GHATIS_PER_DAY).floor();
    // Bir günden çok uzun aralık bir gün doğumu değildir (kutup); çağıran `None` verir.
    (ghatis.is_finite() && (0.0..=f64::from(u16::MAX)).contains(&ghatis)).then_some(ghatis as u32)
}

/// Formülün toplamı.
pub fn shayanadi_total(
    nakshatra: u8,
    graha_number: u8,
    navamsa: u8,
    basis: &ShayanadiBasis,
) -> u32 {
    u32::from(nakshatra) * u32::from(graha_number) * u32::from(navamsa)
        + u32::from(basis.birth_nakshatra)
        + basis.ghatis
        + u32::from(basis.lagna_sign_number)
}

/// Toplamın durumu: 12'ye bölümden artan 1..=11 ilk on bir durum, artan 0 on ikinci durum.
pub fn shayanadi_state(total: u32) -> Shayanadi {
    Shayanadi::ALL[((total + 11) % 12) as usize]
}

/// Haritaya ait terimler; lagna yoksa (doğum saati bilinmiyor) ya da ghati yoksa (gün doğumu
/// yok) `None`.
pub fn shayanadi_basis(positions: &Positions, ghatis: Option<u32>) -> Option<ShayanadiBasis> {
    let lagna = positions.lagna_sign?;
    Some(ShayanadiBasis {
        birth_nakshatra: nakshatra_pada(positions.point(Graha::Moon).sidereal_lon).0,
        ghatis: ghatis?,
        lagna_sign_number: lagna.index() as u8 + 1,
    })
}

/// `graha`'nın Shayanadi durumu; düğümler için `None`.
pub fn shayanadi(
    graha: Graha,
    positions: &Positions,
    basis: &ShayanadiBasis,
) -> Option<ShayanadiHit> {
    let graha_number = graha_number(graha)?;
    let point = positions.point(graha);
    let nakshatra = nakshatra_pada(point.sidereal_lon).0;
    let navamsa = navamsa_number(point.deg_in_sign);
    let total = shayanadi_total(nakshatra, graha_number, navamsa, basis);
    Some(ShayanadiHit {
        state: shayanadi_state(total),
        nakshatra,
        graha_number,
        navamsa,
        total,
        remainder: (total % 12) as u8,
    })
}
````

- [ ] **Step 4: Rapor ve toplayıcı**

`collect.rs`'in **başına**, test modülünün üstüne eklenir (arada bir boş satır). Toplayıcı ayar, an ve konum denetimini `compute_chart_for`'a bırakır; `time_known` yanlışsa lagnayı siler ve ghatiyi hiç hesaplamaz.

````rust
//! Avastha raporu ve toplayıcısı (K1, K3): saf hesap `avasthas` konumları ve ghati sayısını
//! görür; toplayıcı `compute_avasthas` temel haritayı hesaplar, gün doğumundan geçen ghatiyi
//! bulur ve saf hesaba verir.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{compute_chart_for, ChartWarning};
use crate::graha::Graha;
use crate::positions::{ChartInput, Positions};
use crate::CoreError;

use super::baladi::{baladi, jagradadi, Baladi, Jagradadi};
use super::lajjitadi::{lajjitadi, LajjitadiHit};
use super::shayanadi::{elapsed_ghatis, shayanadi, shayanadi_basis, ShayanadiBasis, ShayanadiHit};
use super::{AVASTHA_GRAHAS, AVASTHA_METHOD};

/// Bir grahanın dört dizideki durumu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GrahaAvastha {
    pub graha: Graha,
    pub baladi: Baladi,
    pub jagradadi: Jagradadi,
    /// Kural sırasında; boş olabilir. Lagna yoksa Lajjita değerlendirilmemiştir.
    pub lajjitadi: Vec<LajjitadiHit>,
    /// Lagna ya da gün doğumu yoksa `None`.
    pub shayanadi: Option<ShayanadiHit>,
}

/// `compute_avasthas` sonucu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AvasthaReport {
    /// Yedi graha, `AVASTHA_GRAHAS` sırasında.
    pub grahas: Vec<GrahaAvastha>,
    /// Shayanadi formülünün haritaya ait terimleri; Shayanadi hesaplanmadıysa `None`.
    pub shayanadi_basis: Option<ShayanadiBasis>,
    /// Yanlışsa lagna yoktur: Lajjita değerlendirilmemiş, Shayanadi hesaplanmamıştır ve Ay'a
    /// bağlı değerler yaklaşıktır (Ay'ın derecesi gün içinde değişir).
    pub time_known: bool,
    /// Temel haritanın konum duyarlılığı uyarısı (`LowPrecision`).
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

/// Saf hesap: konumlardan avastha raporu. `ghatis`, gün doğumundan doğuma tamamlanmış ghati
/// sayısıdır; `None` ise (gün doğumu yok) ya da lagna yoksa Shayanadi hesaplanmaz.
pub fn avasthas(positions: &Positions, ghatis: Option<u32>) -> AvasthaReport {
    let basis = shayanadi_basis(positions, ghatis);
    let grahas = AVASTHA_GRAHAS
        .into_iter()
        .filter_map(|graha| {
            let point = positions.point(graha);
            Some(GrahaAvastha {
                graha,
                baladi: baladi(point.rasi_sign, point.deg_in_sign),
                jagradadi: jagradadi(graha, point.rasi_sign)?,
                lajjitadi: lajjitadi(graha, positions),
                shayanadi: basis.as_ref().and_then(|b| shayanadi(graha, positions, b)),
            })
        })
        .collect();
    AvasthaReport {
        grahas,
        shayanadi_basis: basis,
        time_known: positions.lagna_sign.is_some(),
        warnings: Vec::new(),
        method: AVASTHA_METHOD.to_owned(),
    }
}

/// `compute_avasthas` komutunun hesabı: temel harita, konumlar, gün doğumundan geçen ghati,
/// avastha raporu. Ayar, an ve konum denetimi `compute_chart`'ınkidir. `input.time_known`
/// yanlışsa lagna boştur: Lajjita değerlendirilmez ve Shayanadi hesaplanmaz. Doğum yerinde o
/// gün gün doğumu yoksa (kutup) yalnızca Shayanadi boştur. Sayı prashnasında lagna sayıdan
/// gelir (`compute_chart_for`).
pub fn compute_avasthas<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<AvasthaReport, CoreError> {
    let chart = compute_chart_for(eph, input)?;
    let positions = Positions::from(&chart);
    let (positions, ghatis) = if input.time_known {
        let ghatis = chart
            .panchang
            .sunrise_jd_ut
            .and_then(|sunrise| elapsed_ghatis(input.jd_ut, sunrise));
        (positions, ghatis)
    } else {
        (positions.without_lagna(), None)
    };
    let mut report = avasthas(&positions, ghatis);
    report.warnings = chart
        .meta
        .warnings
        .iter()
        .copied()
        .filter(|w| *w == ChartWarning::LowPrecision)
        .collect();
    Ok(report)
}
````

- [ ] **Step 5: Testler ve kapılar**

```bash
cargo fmt --all
cargo test -p jyotish-core --lib avastha::shayanadi 2>&1 | grep -E "^test result"
cargo test -p jyotish-core --lib avastha::collect 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: `7 passed`, `5 passed`; lib `429 passed`, `avastha_golden` `3 passed`, golden `8 passed`, `modules_golden` `17 passed`; fmt ve clippy çıktısız; sayaç `0`. Bir el örneği tutmazsa önce yorumdaki aritmetik ve D8 yeniden okunur; beklenti motor çıktısına bakarak değiştirilmez.

- [ ] **Step 6: Commit**

```bash
git add crates/jyotish-core/src/avastha
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): Shayanadi formula with its terms; avastha report and collector from the last sunrise" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: üç dosya (`666 insertions`); tarama çıktısız; `git status --short` boş.

---

### Task 5: Shayanadi el örnekleri ve ikinci yazım

Yol haritası §5 Plan 24 görev (5); ileri modüller spec'i §13, K9; D8, D12. Altı sentetik harita: dördü hesaplı (ghati 0, 1, 28 ve 59), ikisi hesapsız (lagna yok; gün doğumu yok). Girdi, konumlara ek olarak gün doğumundan doğuma geçen dakikayı taşır; test ghatiyi motorun `elapsed_ghatis` fonksiyonuyla bulur ve örneğin kendi aritmetiğini (çarpım, toplam, yürüyerek bulunan durum) motordan bağımsız sağlar. Çekirdek kod değişmez.

**Files:**
- Create: `tests/golden/modules/avastha/shayanadi/` altında altı dosya: `birth-at-sunrise-zero-ghatis.json`, `birth-just-before-the-next-sunrise.json`, `evening-birth-twenty-eight-ghatis.json`, `ghati-not-yet-complete-counts-the-lower.json`, `no-sunrise-no-shayanadi.json`, `time-unknown-no-shayanadi.json`
- Modify: `crates/jyotish-core/tests/avastha_golden.rs`
- Test: `avastha_golden.rs` iki test kazanır (`shayanadi_hand_fixtures`, `shayanadi_fixtures_cover_the_required_cases`); `unknown_fields_are_rejected` bir beklenti daha

**Interfaces:**
- Consumes: Task 4'ün `avasthas`, `elapsed_ghatis`, `graha_number`, `Shayanadi`, `ShayanadiBasis`'i; Task 3'ün yükleyicisi (`load`, `Fixture`).
- Produces: Shayanadi el örneği biçimi (`input { lagna_lon, minutes_since_sunrise, grahas }`, `expected { basis, grahas { state, nakshatra, navamsa, total } }` ya da `expected: null`); `MIN_SHAYANADI = 6`.

- [ ] **Step 1: Test dosyasına Shayanadi testleri**

Kapsam testi şunları sabitler: on iki durumun her biri en az bir örnekte; artanın sıfır olduğu durum (on ikinci durum); ghati 0, 1, 28 ve 59; bir ghatinin son onda birinde doğum (aşağı yuvarlama); hesaplanmayan iki durum (lagna yok; gün doğumu yok).

````diff
diff --git a/crates/jyotish-core/tests/avastha_golden.rs b/crates/jyotish-core/tests/avastha_golden.rs
index eca28f3..73bed5d 100644
--- a/crates/jyotish-core/tests/avastha_golden.rs
+++ b/crates/jyotish-core/tests/avastha_golden.rs
@@ -1,13 +1,15 @@
 //! Avastha el örnekleri (yol haritası §4.4; ileri modüller spec'i §13): beklenen değerler
 //! kuraldan elle türetilir (`derivation`), başka yazılımın çıktısı olamaz (K9). Her alt klasör
-//! bir dizidir: `tests/golden/modules/avastha/lajjitadi/` (Baladi, Jagradadi ve Lajjitadi).
+//! bir dizidir: `tests/golden/modules/avastha/lajjitadi/` (Baladi, Jagradadi ve Lajjitadi) ve
+//! `tests/golden/modules/avastha/shayanadi/` (Shayanadi; girdide gün doğumundan geçen dakika).
 //! Okuma kuralları ve onay bekleyen varyant noktaları `avastha/REVIEW.md`'dedir.
 
 use std::collections::BTreeMap;
 use std::path::PathBuf;
 
 use jyotish_core::avastha::{
-    baladi, jagradadi, lajjitadi, Baladi, Jagradadi, Lajjitadi, LajjitadiHit, AVASTHA_GRAHAS,
+    avasthas, baladi, elapsed_ghatis, graha_number, jagradadi, lajjitadi, Baladi, Jagradadi,
+    Lajjitadi, LajjitadiHit, Shayanadi, ShayanadiBasis, AVASTHA_GRAHAS,
 };
 use jyotish_core::{Graha, Positions};
 use serde::de::DeserializeOwned;
@@ -15,6 +17,9 @@ use serde::Deserialize;
 
 /// Örnek silmek testi düşürür.
 const MIN_LAJJITADI: usize = 7;
+const MIN_SHAYANADI: usize = 6;
+/// Bir gün, dakika.
+const MINUTES_PER_DAY: f64 = 1440.0;
 
 #[derive(Debug, Deserialize)]
 #[serde(deny_unknown_fields)]
@@ -250,10 +255,185 @@ fn lajjitadi_fixtures_cover_the_required_cases() {
     }
 }
 
+/// Shayanadi örneğinin girdisi: konumlar ve gün doğumundan doğuma geçen dakika. `null` dakika
+/// o gün gün doğumu olmayan yerdir.
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ShayanadiInput {
+    lagna_lon: Option<f64>,
+    minutes_since_sunrise: Option<f64>,
+    grahas: BTreeMap<Graha, f64>,
+}
+
+impl ShayanadiInput {
+    fn positions(&self, id: &str) -> Positions {
+        assert_eq!(self.grahas.len(), 9, "{id}: all nine grahas are required");
+        Positions::synthetic(
+            self.lagna_lon,
+            std::array::from_fn(|i| self.grahas[&Graha::ALL[i]]),
+        )
+    }
+
+    /// Tamamlanmış ghati: gün doğumu 0 anı sayılır.
+    fn ghatis(&self) -> Option<u32> {
+        self.minutes_since_sunrise
+            .and_then(|m| elapsed_ghatis(m / MINUTES_PER_DAY, 0.0))
+    }
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ShayanadiRow {
+    state: Shayanadi,
+    nakshatra: u8,
+    navamsa: u8,
+    total: u32,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ShayanadiExpected {
+    basis: ShayanadiBasis,
+    grahas: BTreeMap<Graha, ShayanadiRow>,
+}
+
+type ShayanadiFixture = Fixture<Option<ShayanadiExpected>, ShayanadiInput>;
+
+fn shayanadi_fixtures() -> Vec<(ShayanadiFixture, Positions)> {
+    load::<Option<ShayanadiExpected>, ShayanadiInput>("shayanadi", MIN_SHAYANADI)
+        .into_iter()
+        .map(|f| {
+            let positions = f.input.positions(&f.id);
+            (f, positions)
+        })
+        .collect()
+}
+
+/// İkinci yazım: bölme yok; durum çemberinde `total` adım yürünür (ilk adım ilk duruma varır).
+fn walked_state(total: u32) -> Shayanadi {
+    let mut at = Shayanadi::ALL.len() - 1;
+    for _ in 0..total {
+        at = if at + 1 == Shayanadi::ALL.len() {
+            0
+        } else {
+            at + 1
+        };
+    }
+    Shayanadi::ALL[at]
+}
+
+#[test]
+fn shayanadi_hand_fixtures() {
+    for (f, p) in shayanadi_fixtures() {
+        let report = avasthas(&p, f.input.ghatis());
+        assert_eq!(report.grahas.len(), 7, "{}", f.id);
+        let Some(x) = &f.expected else {
+            // Lagna ya da gün doğumu yok: Shayanadi hesaplanmaz.
+            assert!(
+                f.input.lagna_lon.is_none() || f.input.minutes_since_sunrise.is_none(),
+                "{}: a null expectation needs a missing lagna or sunrise",
+                f.id
+            );
+            assert_eq!(report.shayanadi_basis, None, "{}", f.id);
+            assert!(
+                report.grahas.iter().all(|g| g.shayanadi.is_none()),
+                "{}",
+                f.id
+            );
+            continue;
+        };
+        assert_eq!(report.shayanadi_basis, Some(x.basis), "{}: basis", f.id);
+        assert_eq!(
+            x.grahas.keys().copied().collect::<Vec<_>>(),
+            AVASTHA_GRAHAS,
+            "{}: the seven grahas",
+            f.id
+        );
+        let addend = u32::from(x.basis.birth_nakshatra)
+            + x.basis.ghatis
+            + u32::from(x.basis.lagna_sign_number);
+        for row in &report.grahas {
+            let want = &x.grahas[&row.graha];
+            let got = row.shayanadi.expect("a state for each of the seven grahas");
+            assert_eq!(
+                (got.state, got.nakshatra, got.navamsa, got.total),
+                (want.state, want.nakshatra, want.navamsa, want.total),
+                "{}: {:?}",
+                f.id,
+                row.graha
+            );
+            // Örneğin kendi aritmetiği tutarlıdır (motordan bağımsız): toplam ve durum.
+            let number = u32::from(graha_number(row.graha).expect("seven grahas"));
+            assert_eq!(
+                u32::from(want.nakshatra) * number * u32::from(want.navamsa) + addend,
+                want.total,
+                "{}: {:?} total",
+                f.id,
+                row.graha
+            );
+            assert_eq!(
+                walked_state(want.total),
+                want.state,
+                "{}: {:?} state",
+                f.id,
+                row.graha
+            );
+            assert_eq!(
+                u32::from(got.remainder),
+                want.total % 12,
+                "{}: {:?} remainder",
+                f.id,
+                row.graha
+            );
+        }
+    }
+}
+
+#[test]
+fn shayanadi_fixtures_cover_the_required_cases() {
+    let fixtures = shayanadi_fixtures();
+    let rows = || {
+        fixtures
+            .iter()
+            .filter_map(|(f, _)| f.expected.as_ref())
+            .flat_map(|x| x.grahas.values())
+    };
+    // On iki durumun her biri en az bir örnekte.
+    for state in Shayanadi::ALL {
+        assert!(rows().any(|r| r.state == state), "{state:?} in no fixture");
+    }
+    // Artanın sıfır olduğu durum: on ikinci durum.
+    assert!(rows().any(|r| r.total.is_multiple_of(12) && r.state == Shayanadi::Nidra));
+    // Ghati sayımı: tam doğuşta 0, yarım ghati aşağı yuvarlanır, sonraki doğuştan önce 59.
+    let ghatis: Vec<u32> = fixtures
+        .iter()
+        .filter_map(|(f, _)| f.expected.as_ref().map(|x| x.basis.ghatis))
+        .collect();
+    for wanted in [0, 1, 28, 59] {
+        assert!(ghatis.contains(&wanted), "{wanted} ghatis in no fixture");
+    }
+    assert!(
+        fixtures.iter().any(|(f, _)| f
+            .input
+            .minutes_since_sunrise
+            .is_some_and(|m| (m / 24.0).fract() > 0.9)),
+        "a birth late in a ghati"
+    );
+    // Hesaplanmayan iki durum: lagna yok; gün doğumu yok.
+    assert!(fixtures
+        .iter()
+        .any(|(f, _)| f.expected.is_none() && f.input.lagna_lon.is_none()));
+    assert!(fixtures.iter().any(|(f, _)| f.expected.is_none()
+        && f.input.lagna_lon.is_some()
+        && f.input.minutes_since_sunrise.is_none()));
+}
+
 #[test]
 fn unknown_fields_are_rejected() {
     let text = r#"{"lagna_lon": null, "grahas": {}, "extra": 1}"#;
     assert!(serde_json::from_str::<Input>(text).is_err());
     let text = r#"{"graha": "Sun", "state": "Lajjita", "why": "x"}"#;
     assert!(serde_json::from_str::<Absent>(text).is_err());
+    let text = r#"{"state": "Nidra", "nakshatra": 1, "navamsa": 1, "total": 12, "note": "x"}"#;
+    assert!(serde_json::from_str::<ShayanadiRow>(text).is_err());
 }
````

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `cargo test -p jyotish-core --test avastha_golden 2>&1 | grep -E "cannot read|^test result"`
Expected: iki yeni test `cannot read …/tests/golden/modules/avastha/shayanadi` ile düşer; `test result: FAILED. 3 passed; 2 failed`.

- [ ] **Step 3: Altı el örneği**

Hesaplı dosyalarda `derivation` sırayla ghatiyi (dakika ÷ 24, tamamlanan), doğum nakshatrasını ve lagna sırasını, ortak toplananı (`J + Gh + L`) ve her graha için nakshatrayı, navamsa dilimini, çarpımı, toplamı ve 12'ye bölümünü yazar.

**Yeni dosya:** `tests/golden/modules/avastha/shayanadi/evening-birth-twenty-eight-ghatis.json`

````json
{
  "id": "evening-birth-twenty-eight-ghatis",
  "source": "hand",
  "rule_ref": "BPHS, avasthalar bölümü: Shayanadi (alıntı yapılmaz); okuma kuralları avastha/REVIEW.md",
  "derivation": [
    "Gün doğumundan 690 dakika sonra doğum (ör. 06:00 doğuş, 17:30 doğum).",
    "Ghati: 690.0 dk ÷ 24 dk = 28.7500; tamamlanan 28.",
    "Doğum nakshatrası: Ay 199.5° ÷ 13°20′ → 15. nakshatra (Swati). Lagna 70.0° İkizler: 3.",
    "Ortak toplanan: J + Gh + L = 15 + 28 + 3 = 46.",
    "Güneş: 256.5° = Yay 16°30′; nakshatra 20 (Purva Ashadha), navamsa dilimi 5, sayı 1. 20 × 1 × 5 = 100; + 46 = 146 = 12 × 12 + 2: artan 2 → Upaveshana.",
    "Ay: 199.5° = Terazi 19°30′; nakshatra 15 (Swati), navamsa dilimi 6, sayı 2. 15 × 2 × 6 = 180; + 46 = 226 = 18 × 12 + 10: artan 10 → NrityaLipsa.",
    "Mars: 304.0° = Kova 4°00′; nakshatra 23 (Dhanishta), navamsa dilimi 2, sayı 3. 23 × 3 × 2 = 138; + 46 = 184 = 15 × 12 + 4: artan 4 → Prakashana.",
    "Merkür: 248.0° = Yay 8°00′; nakshatra 19 (Mula), navamsa dilimi 3, sayı 4. 19 × 4 × 3 = 228; + 46 = 274 = 22 × 12 + 10: artan 10 → NrityaLipsa.",
    "Jüpiter: 1.5° = Koç 1°30′; nakshatra 1 (Ashwini), navamsa dilimi 1, sayı 5. 1 × 5 × 1 = 5; + 46 = 51 = 4 × 12 + 3: artan 3 → Netrapani.",
    "Venüs: 217.75° = Akrep 7°45′; nakshatra 17 (Anuradha), navamsa dilimi 3, sayı 6. 17 × 6 × 3 = 306; + 46 = 352 = 29 × 12 + 4: artan 4 → Prakashana.",
    "Satürn: 16.5° = Koç 16°30′; nakshatra 2 (Bharani), navamsa dilimi 5, sayı 7. 2 × 7 × 5 = 70; + 46 = 116 = 9 × 12 + 8: artan 8 → Agama."
  ],
  "input": {
    "lagna_lon": 70.0,
    "minutes_since_sunrise": 690.0,
    "grahas": {"Sun": 256.5, "Moon": 199.5, "Mars": 304.0, "Mercury": 248.0, "Jupiter": 1.5, "Venus": 217.75, "Saturn": 16.5, "Rahu": 101.0, "Ketu": 281.0}
  },
  "expected": {
    "basis": {"birth_nakshatra": 15, "ghatis": 28, "lagna_sign_number": 3},
    "grahas": {
      "Sun": {"state": "Upaveshana", "nakshatra": 20, "navamsa": 5, "total": 146},
      "Moon": {"state": "NrityaLipsa", "nakshatra": 15, "navamsa": 6, "total": 226},
      "Mars": {"state": "Prakashana", "nakshatra": 23, "navamsa": 2, "total": 184},
      "Mercury": {"state": "NrityaLipsa", "nakshatra": 19, "navamsa": 3, "total": 274},
      "Jupiter": {"state": "Netrapani", "nakshatra": 1, "navamsa": 1, "total": 51},
      "Venus": {"state": "Prakashana", "nakshatra": 17, "navamsa": 3, "total": 352},
      "Saturn": {"state": "Agama", "nakshatra": 2, "navamsa": 5, "total": 116}
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/avastha/shayanadi/ghati-not-yet-complete-counts-the-lower.json`

````json
{
  "id": "ghati-not-yet-complete-counts-the-lower",
  "source": "hand",
  "rule_ref": "BPHS, avasthalar bölümü: Shayanadi (alıntı yapılmaz); okuma kuralları avastha/REVIEW.md",
  "derivation": [
    "Gün doğumundan 47,9 dakika sonra doğum: ikinci ghati bitmemiştir (48 dakika), tamamlanan ghati 1'dir.",
    "Ghati: 47.9 dk ÷ 24 dk = 1.9958; tamamlanan 1.",
    "Doğum nakshatrası: Ay 115.0° ÷ 13°20′ → 9. nakshatra (Ashlesha). Lagna 122.0° Aslan: 5.",
    "Ortak toplanan: J + Gh + L = 9 + 1 + 5 = 15.",
    "Güneş: 33.0° = Boğa 3°00′; nakshatra 3 (Krittika), navamsa dilimi 1, sayı 1. 3 × 1 × 1 = 3; + 15 = 18 = 1 × 12 + 6: artan 6 → Agamana.",
    "Ay: 115.0° = Yengeç 25°00′; nakshatra 9 (Ashlesha), navamsa dilimi 8, sayı 2. 9 × 2 × 8 = 144; + 15 = 159 = 13 × 12 + 3: artan 3 → Netrapani.",
    "Mars: 134.0° = Aslan 14°00′; nakshatra 11 (Purva Phalguni), navamsa dilimi 5, sayı 3. 11 × 3 × 5 = 165; + 15 = 180 = 15 × 12 + 0: artan 0 → on ikinci durum Nidra.",
    "Merkür: 51.0° = Boğa 21°00′; nakshatra 4 (Rohini), navamsa dilimi 7, sayı 4. 4 × 4 × 7 = 112; + 15 = 127 = 10 × 12 + 7: artan 7 → Sabha.",
    "Jüpiter: 159.0° = Başak 9°00′; nakshatra 12 (Uttara Phalguni), navamsa dilimi 3, sayı 5. 12 × 5 × 3 = 180; + 15 = 195 = 16 × 12 + 3: artan 3 → Netrapani.",
    "Venüs: 88.0° = İkizler 28°00′; nakshatra 7 (Punarvasu), navamsa dilimi 9, sayı 6. 7 × 6 × 9 = 378; + 15 = 393 = 32 × 12 + 9: artan 9 → Bhojana.",
    "Satürn: 347.0° = Balık 17°00′; nakshatra 27 (Revati), navamsa dilimi 6, sayı 7. 27 × 7 × 6 = 1134; + 15 = 1149 = 95 × 12 + 9: artan 9 → Bhojana."
  ],
  "input": {
    "lagna_lon": 122.0,
    "minutes_since_sunrise": 47.9,
    "grahas": {"Sun": 33.0, "Moon": 115.0, "Mars": 134.0, "Mercury": 51.0, "Jupiter": 159.0, "Venus": 88.0, "Saturn": 347.0, "Rahu": 215.0, "Ketu": 35.0}
  },
  "expected": {
    "basis": {"birth_nakshatra": 9, "ghatis": 1, "lagna_sign_number": 5},
    "grahas": {
      "Sun": {"state": "Agamana", "nakshatra": 3, "navamsa": 1, "total": 18},
      "Moon": {"state": "Netrapani", "nakshatra": 9, "navamsa": 8, "total": 159},
      "Mars": {"state": "Nidra", "nakshatra": 11, "navamsa": 5, "total": 180},
      "Mercury": {"state": "Sabha", "nakshatra": 4, "navamsa": 7, "total": 127},
      "Jupiter": {"state": "Netrapani", "nakshatra": 12, "navamsa": 3, "total": 195},
      "Venus": {"state": "Bhojana", "nakshatra": 7, "navamsa": 9, "total": 393},
      "Saturn": {"state": "Bhojana", "nakshatra": 27, "navamsa": 6, "total": 1149}
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/avastha/shayanadi/birth-just-before-the-next-sunrise.json`

````json
{
  "id": "birth-just-before-the-next-sunrise",
  "source": "hand",
  "rule_ref": "BPHS, avasthalar bölümü: Shayanadi (alıntı yapılmaz); okuma kuralları avastha/REVIEW.md",
  "derivation": [
    "Gün doğumundan 1439 dakika sonra doğum (sonraki doğuştan bir dakika önce): gün hâlâ önceki doğuştan sayılır.",
    "Ghati: 1439.0 dk ÷ 24 dk = 59.9583; tamamlanan 59.",
    "Doğum nakshatrası: Ay 331.0° ÷ 13°20′ → 25. nakshatra (Purva Bhadrapada). Lagna 359.0° Balık: 12.",
    "Ortak toplanan: J + Gh + L = 25 + 59 + 12 = 96.",
    "Güneş: 282.0° = Oğlak 12°00′; nakshatra 22 (Shravana), navamsa dilimi 4, sayı 1. 22 × 1 × 4 = 88; + 96 = 184 = 15 × 12 + 4: artan 4 → Prakashana.",
    "Ay: 331.0° = Balık 1°00′; nakshatra 25 (Purva Bhadrapada), navamsa dilimi 1, sayı 2. 25 × 2 × 1 = 50; + 96 = 146 = 12 × 12 + 2: artan 2 → Upaveshana.",
    "Mars: 94.0° = Yengeç 4°00′; nakshatra 8 (Pushya), navamsa dilimi 2, sayı 3. 8 × 3 × 2 = 48; + 96 = 144 = 12 × 12 + 0: artan 0 → on ikinci durum Nidra.",
    "Merkür: 297.0° = Oğlak 27°00′; nakshatra 23 (Dhanishta), navamsa dilimi 9, sayı 4. 23 × 4 × 9 = 828; + 96 = 924 = 77 × 12 + 0: artan 0 → on ikinci durum Nidra.",
    "Jüpiter: 262.0° = Yay 22°00′; nakshatra 20 (Purva Ashadha), navamsa dilimi 7, sayı 5. 20 × 5 × 7 = 700; + 96 = 796 = 66 × 12 + 4: artan 4 → Prakashana.",
    "Venüs: 313.0° = Kova 13°00′; nakshatra 24 (Shatabhisha), navamsa dilimi 4, sayı 6. 24 × 6 × 4 = 576; + 96 = 672 = 56 × 12 + 0: artan 0 → on ikinci durum Nidra.",
    "Satürn: 301.0° = Kova 1°00′; nakshatra 23 (Dhanishta), navamsa dilimi 1, sayı 7. 23 × 7 × 1 = 161; + 96 = 257 = 21 × 12 + 5: artan 5 → Gamana."
  ],
  "input": {
    "lagna_lon": 359.0,
    "minutes_since_sunrise": 1439.0,
    "grahas": {"Sun": 282.0, "Moon": 331.0, "Mars": 94.0, "Mercury": 297.0, "Jupiter": 262.0, "Venus": 313.0, "Saturn": 301.0, "Rahu": 20.0, "Ketu": 200.0}
  },
  "expected": {
    "basis": {"birth_nakshatra": 25, "ghatis": 59, "lagna_sign_number": 12},
    "grahas": {
      "Sun": {"state": "Prakashana", "nakshatra": 22, "navamsa": 4, "total": 184},
      "Moon": {"state": "Upaveshana", "nakshatra": 25, "navamsa": 1, "total": 146},
      "Mars": {"state": "Nidra", "nakshatra": 8, "navamsa": 2, "total": 144},
      "Mercury": {"state": "Nidra", "nakshatra": 23, "navamsa": 9, "total": 924},
      "Jupiter": {"state": "Prakashana", "nakshatra": 20, "navamsa": 7, "total": 796},
      "Venus": {"state": "Nidra", "nakshatra": 24, "navamsa": 4, "total": 672},
      "Saturn": {"state": "Gamana", "nakshatra": 23, "navamsa": 1, "total": 257}
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/avastha/shayanadi/birth-at-sunrise-zero-ghatis.json`

````json
{
  "id": "birth-at-sunrise-zero-ghatis",
  "source": "hand",
  "rule_ref": "BPHS, avasthalar bölümü: Shayanadi (alıntı yapılmaz); okuma kuralları avastha/REVIEW.md",
  "derivation": [
    "Doğum tam gün doğumunda: tamamlanan ghati 0'dır.",
    "Ghati: 0.0 dk ÷ 24 dk = 0.0000; tamamlanan 0.",
    "Doğum nakshatrası: Ay 165.0° ÷ 13°20′ → 13. nakshatra (Hasta). Lagna 30.5° Boğa: 2.",
    "Ortak toplanan: J + Gh + L = 13 + 0 + 2 = 15.",
    "Güneş: 0.5° = Koç 0°30′; nakshatra 1 (Ashwini), navamsa dilimi 1, sayı 1. 1 × 1 × 1 = 1; + 15 = 16 = 1 × 12 + 4: artan 4 → Prakashana.",
    "Ay: 165.0° = Başak 15°00′; nakshatra 13 (Hasta), navamsa dilimi 5, sayı 2. 13 × 2 × 5 = 130; + 15 = 145 = 12 × 12 + 1: artan 1 → Shayana.",
    "Mars: 239.5° = Akrep 29°30′; nakshatra 18 (Jyeshtha), navamsa dilimi 9, sayı 3. 18 × 3 × 9 = 486; + 15 = 501 = 41 × 12 + 9: artan 9 → Bhojana.",
    "Merkür: 342.0° = Balık 12°00′; nakshatra 26 (Uttara Bhadrapada), navamsa dilimi 4, sayı 4. 26 × 4 × 4 = 416; + 15 = 431 = 35 × 12 + 11: artan 11 → Kautuka.",
    "Jüpiter: 138.0° = Aslan 18°00′; nakshatra 11 (Purva Phalguni), navamsa dilimi 6, sayı 5. 11 × 5 × 6 = 330; + 15 = 345 = 28 × 12 + 9: artan 9 → Bhojana.",
    "Venüs: 10.0° = Koç 10°00′; nakshatra 1 (Ashwini), navamsa dilimi 4, sayı 6. 1 × 6 × 4 = 24; + 15 = 39 = 3 × 12 + 3: artan 3 → Netrapani.",
    "Satürn: 272.0° = Oğlak 2°00′; nakshatra 21 (Uttara Ashadha), navamsa dilimi 1, sayı 7. 21 × 7 × 1 = 147; + 15 = 162 = 13 × 12 + 6: artan 6 → Agamana."
  ],
  "input": {
    "lagna_lon": 30.5,
    "minutes_since_sunrise": 0.0,
    "grahas": {"Sun": 0.5, "Moon": 165.0, "Mars": 239.5, "Mercury": 342.0, "Jupiter": 138.0, "Venus": 10.0, "Saturn": 272.0, "Rahu": 69.0, "Ketu": 249.0}
  },
  "expected": {
    "basis": {"birth_nakshatra": 13, "ghatis": 0, "lagna_sign_number": 2},
    "grahas": {
      "Sun": {"state": "Prakashana", "nakshatra": 1, "navamsa": 1, "total": 16},
      "Moon": {"state": "Shayana", "nakshatra": 13, "navamsa": 5, "total": 145},
      "Mars": {"state": "Bhojana", "nakshatra": 18, "navamsa": 9, "total": 501},
      "Mercury": {"state": "Kautuka", "nakshatra": 26, "navamsa": 4, "total": 431},
      "Jupiter": {"state": "Bhojana", "nakshatra": 11, "navamsa": 6, "total": 345},
      "Venus": {"state": "Netrapani", "nakshatra": 1, "navamsa": 4, "total": 39},
      "Saturn": {"state": "Agamana", "nakshatra": 21, "navamsa": 1, "total": 162}
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/avastha/shayanadi/time-unknown-no-shayanadi.json`

````json
{
  "id": "time-unknown-no-shayanadi",
  "source": "hand",
  "rule_ref": "BPHS, avasthalar bölümü: Shayanadi (alıntı yapılmaz); okuma kuralları avastha/REVIEW.md",
  "derivation": [
    "Doğum saati bilinmiyor: lagna yok; formülün lagna ve ghati terimleri tanımsızdır, Shayanadi hesaplanmaz.",
    "Beklenen: rapor Shayanadi taşımaz (her graha için boş); Baladi, Jagradadi ve Lajjitadi bundan etkilenmez."
  ],
  "input": {
    "lagna_lon": null,
    "minutes_since_sunrise": 690.0,
    "grahas": {"Sun": 256.5, "Moon": 199.5, "Mars": 304.0, "Mercury": 248.0, "Jupiter": 1.5, "Venus": 217.75, "Saturn": 16.5, "Rahu": 101.0, "Ketu": 281.0}
  },
  "expected": null
}
````

**Yeni dosya:** `tests/golden/modules/avastha/shayanadi/no-sunrise-no-shayanadi.json`

````json
{
  "id": "no-sunrise-no-shayanadi",
  "source": "hand",
  "rule_ref": "BPHS, avasthalar bölümü: Shayanadi (alıntı yapılmaz); okuma kuralları avastha/REVIEW.md",
  "derivation": [
    "Doğum yerinde o gün gün doğumu yok (kutup günü ya da gecesi): ghati terimi tanımsızdır, Shayanadi hesaplanmaz.",
    "Beklenen: rapor Shayanadi taşımaz (her graha için boş); Baladi, Jagradadi ve Lajjitadi bundan etkilenmez."
  ],
  "input": {
    "lagna_lon": 70.0,
    "minutes_since_sunrise": null,
    "grahas": {"Sun": 256.5, "Moon": 199.5, "Mars": 304.0, "Mercury": 248.0, "Jupiter": 1.5, "Venus": 217.75, "Saturn": 16.5, "Rahu": 101.0, "Ketu": 281.0}
  },
  "expected": null
}
````

- [ ] **Step 4: Testler ve kapılar**

```bash
ls tests/golden/modules/avastha/shayanadi | wc -l
cargo fmt --all --check
cargo test -p jyotish-core --test avastha_golden 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: `6`; fmt çıktısız; `avastha_golden` `5 passed`; lib `429 passed`, golden `8 passed`, `modules_golden` `17 passed`; clippy çıktısız; sayaç `0`. Bir örnek motorla uyuşmazsa Task 3 Step 4'teki kural geçerlidir: önce `derivation`'ın aritmetiği D8'den yeniden yapılır; beklenen değer motor çıktısına bakarak değiştirilmez.

- [ ] **Step 5: Commit**

```bash
git add crates/jyotish-core/tests/avastha_golden.rs tests/golden/modules/avastha/shayanadi
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(core): Shayanadi hand examples and a second writing of the formula" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: yedi dosya (`352 insertions`, `2 deletions`); tarama çıktısız; `git status --short` boş.

---

### Task 6: Uygulama: `compute_avasthas` komutu, bağlamalar, fikstür ve Avastha paneli

Yol haritası §5 Plan 24 görev (6) ve (7)'nin panel bölümü ("komut ve Avastha paneli", "i18n", "saat bilinmiyor durumu"); K1, K2, K11, K12; D10, D13–D17, D19, D21. Komut yeni bir dosyadadır ve var olan komut dosyalarına dokunulmaz. Paylaşılan dosyalardaki her değişiklik ekleme türündedir ("Paylaşılan dosyalar" tablosu).

**Files:**
- Create: `app/src-tauri/src/avastha_commands.rs`, `app/src-tauri/tests/avastha.rs`, `app/ui/modules/useAvasthas.ts`, `app/ui/panels/avasthaLabels.ts`, `app/ui/panels/avastha.css`, `app/ui/panels/AvasthaPanel.tsx`, `app/ui/panels/AvasthaPanel.test.tsx`
- Modify: `app/src-tauri/src/lib.rs`, `app/src-tauri/tests/ui_fixture.rs`, `app/ui/bindings.ts` (üretilir), `app/ui/test/fixtures/delhi.json` (üretilir), `app/ui/test/fixture.ts`, `app/ui/api.ts`, `app/ui/api.test.ts`, `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/panels/registry.ts`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: `tests/avastha.rs` (1); `ui_fixture` (var olan iki test, yeni alanlarla); `AvasthaPanel.test.tsx` (13); `api.test.ts` (tabloda bir satır); `model.test.ts` (iki beklenti)

**Interfaces:**
- Consumes: Task 4'ün `jyotish_core::avastha::{compute_avasthas, avasthas, elapsed_ghatis, shayanadi_state, AvasthaReport, Lajjitadi, AVASTHA_GRAHAS, AVASTHA_METHOD}`'u ve dört `ALL` listesi; `ephemeris::{Engine, SwissEph}`; `client_commands::blocking`; `commands::{CommandError, compute_chart_blocking}`; arayüzde `modules/{useModule, input}`, `panels/{Approx, PinButton, bala (BalaScroll), labels (NAKSHATRA_NAMES), states}`, `highlight/store`, `session/store`.
- Produces:
  - Rust: `avastha_commands::compute_avasthas_blocking(engine: &Engine, input: ChartInput) -> Result<AvasthaReport, CommandError>`; Tauri komutu `compute_avasthas(input) -> AvasthaReport`.
  - Bağlamalar: `commands.computeAvasthas`; tipler `AvasthaReport`, `GrahaAvastha`, `Baladi`, `Jagradadi`, `Lajjitadi`, `LajjitadiHit`, `Shayanadi`, `ShayanadiBasis`, `ShayanadiHit`.
  - Fikstür: `avasthas`, `avasthas_time_unknown`, `avastha_ids { baladi, jagradadi, lajjitadi, shayanadi }`.
  - Arayüz: `api.computeAvasthas(input)`; `useAvasthas(opened, enabled = true): ModuleState<AvasthaReport>`; `BALADI_NAMES`, `JAGRADADI_NAMES`, `LAJJITADI_NAMES`, `SHAYANADI_NAMES`; `AvasthaPanel`; panel türü `avastha` (7 × 16); i18n `panel.avastha` ve `avastha.*` (yirmi anahtar).

- [ ] **Step 1: Rust testleri: gerçek motorla çapraz denetim, değişmezler, hatalar ve bütçe; fikstür**

`tests/avastha.rs` beş golden anın her birinde raporu v1'in doğrulanmış haritasından yeniden kurar (konumlar ve panchangın gün doğumu) ve eşitliği ister; değişmezleri (yedi satır, kural sırası, nakshatra ve lagna teriminin haritayla aynılığı), ev sistemi ayarından bağımsızlığı, doğum saati bilinmeyen girdiyi, gün doğumsuz haritayı (Tromsø), hata eşlemesini, sayı prashnasını ve bütçeyi denetler.

**Yeni dosya:** `app/src-tauri/tests/avastha.rs`

````rust
//! Avastha komut gövdesi, gerçek `Engine` ile uçtan uca (yol haritası §4.4; ileri modüller
//! spec'i §13 "Komut katmanı", §15 bütçeleri). Tek test: süreçte yalnızca bir SE örneği
//! açılabilir. Ayrı dosyadır: `tests/modules.rs` paralel planlarla paylaşılmaz.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use ephemeris::Engine;
use jyotish_core::avastha::{
    avasthas, elapsed_ghatis, shayanadi_state, Lajjitadi, AVASTHA_GRAHAS, AVASTHA_METHOD,
};
use jyotish_core::{CalcSettings, ChartInput, ChartWarning, HouseSystem, Positions};
use jyotish_desk_lib::avastha_commands::compute_avasthas_blocking;
use jyotish_desk_lib::commands::compute_chart_blocking;
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

#[test]
fn avastha_command_runs_on_the_engine_thread_and_maps_errors() {
    let engine = Engine::spawn(root().join("../../data/ephe")).expect("spawn engine");
    let inputs = golden_inputs();
    assert!(inputs.len() >= 5, "{} golden moments", inputs.len());
    // Aşağıdaki kollar boş geçmemelidir: golden anlarda en az bir Shayanadi'li ve en az bir
    // gün doğumsuz harita, en az bir Lajjita dışı durum bulunur.
    let (mut with_shayanadi, mut without_sunrise, mut states) = (0, 0, 0);
    for (id, input) in &inputs {
        let report = compute_avasthas_blocking(&engine, input.clone())
            .unwrap_or_else(|e| panic!("{id}: {}", e.code));
        assert_eq!(report.method, AVASTHA_METHOD, "{id}");
        assert!(report.time_known, "{id}");
        let order: Vec<_> = report.grahas.iter().map(|g| g.graha).collect();
        assert_eq!(order, AVASTHA_GRAHAS, "{id}");

        // Çapraz denetim: v1'de golden fixture'larla doğrulanmış haritanın konumlarından ve
        // panchangının gün doğumundan aynı rapor çıkar.
        let chart = compute_chart_blocking(
            &engine,
            input.jd_ut,
            input.lat,
            input.lon,
            input.settings.clone(),
        )
        .expect("chart");
        let positions = Positions::from(&chart);
        let ghatis = chart
            .panchang
            .sunrise_jd_ut
            .and_then(|rise| elapsed_ghatis(input.jd_ut, rise));
        let mut want = avasthas(&positions, ghatis);
        want.warnings = chart
            .meta
            .warnings
            .iter()
            .copied()
            .filter(|w| *w == ChartWarning::LowPrecision)
            .collect();
        assert_eq!(report, want, "{id}");

        // Değişmezler gerçek haritalarda da sağlanır.
        match report.shayanadi_basis {
            Some(basis) => {
                with_shayanadi += 1;
                assert!((1..=27).contains(&basis.birth_nakshatra), "{id}");
                assert!((1..=12).contains(&basis.lagna_sign_number), "{id}");
                // Doğum, son gün doğumundan en çok bir gün ve çeyrek sonradır (v1 penceresi).
                assert!(basis.ghatis <= 75, "{id}: {} ghatis", basis.ghatis);
                assert_eq!(
                    basis.birth_nakshatra, chart.grahas[1].nakshatra,
                    "{id}: the Moon's nakshatra is the chart's"
                );
                assert_eq!(
                    usize::from(basis.lagna_sign_number),
                    chart.lagna.sign.index() + 1,
                    "{id}"
                );
                for row in &report.grahas {
                    let hit = row.shayanadi.expect("a state for each graha");
                    assert_eq!(hit.state, shayanadi_state(hit.total), "{id}");
                    assert_eq!(
                        hit.nakshatra, chart.grahas[row.graha as usize].nakshatra,
                        "{id}: {:?}",
                        row.graha
                    );
                    assert!((1..=9).contains(&hit.navamsa), "{id}");
                }
            }
            None => {
                without_sunrise += 1;
                assert!(
                    chart.panchang.sunrise_jd_ut.is_none(),
                    "{id}: no Shayanadi only without a sunrise"
                );
                assert!(chart
                    .meta
                    .warnings
                    .contains(&ChartWarning::SunriseUnavailable));
                assert!(report.grahas.iter().all(|g| g.shayanadi.is_none()), "{id}");
            }
        }
        for row in &report.grahas {
            let listed: Vec<_> = row.lajjitadi.iter().map(|h| h.state).collect();
            let mut sorted = listed.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(listed, sorted, "{id}: {:?} rule order", row.graha);
            states += listed.len();
            for h in &row.lajjitadi {
                assert!(!h.grahas.contains(&row.graha), "{id}: {:?}", row.graha);
            }
        }

        // Evler burç evidir (K4): ev sistemi ayarı raporu değiştirmez.
        for house_system in [HouseSystem::Sripati, HouseSystem::Placidus] {
            let other = ChartInput {
                settings: CalcSettings {
                    house_system,
                    ..input.settings.clone()
                },
                ..input.clone()
            };
            assert_eq!(
                compute_avasthas_blocking(&engine, other).expect("house system"),
                report,
                "{id}: {house_system:?}"
            );
        }

        // Doğum saati bilinmiyorsa lagna yoktur: Lajjita değerlendirilmez, Shayanadi yoktur;
        // öteki dizi ve durumlar aynıdır.
        let unknown = compute_avasthas_blocking(
            &engine,
            ChartInput {
                time_known: false,
                lagna_number: None,
                ..input.clone()
            },
        )
        .expect("time unknown");
        assert_eq!(
            unknown,
            {
                let mut w = avasthas(&positions.clone().without_lagna(), None);
                w.warnings.clone_from(&report.warnings);
                w
            },
            "{id}"
        );
        assert!(
            !unknown.time_known && unknown.shayanadi_basis.is_none(),
            "{id}"
        );
        for (a, b) in unknown.grahas.iter().zip(&report.grahas) {
            assert_eq!((a.baladi, a.jagradadi), (b.baladi, b.jagradadi), "{id}");
            assert!(a.shayanadi.is_none(), "{id}");
            let rest: Vec<_> = b
                .lajjitadi
                .iter()
                .filter(|h| h.state != Lajjitadi::Lajjita)
                .cloned()
                .collect();
            assert_eq!(a.lajjitadi, rest, "{id}: {:?}", a.graha);
        }
    }
    assert!(
        with_shayanadi >= 3,
        "{with_shayanadi} charts with a sunrise"
    );
    assert!(without_sunrise >= 1, "no polar chart in the golden moments");
    assert!(
        states >= 10,
        "{states} lajjitadi states in the golden moments"
    );

    // Hatalar `compute_chart` ile aynı kodlara eşlenir; saat bilinmese de denetlenir.
    let delhi = inputs
        .iter()
        .find(|(id, _)| id == "new-delhi-2000-01-01")
        .expect("delhi fixture")
        .1
        .clone();
    let bad_place = ChartInput {
        lat: 91.0,
        time_known: false,
        lagna_number: None,
        ..delhi.clone()
    };
    assert_eq!(
        compute_avasthas_blocking(&engine, bad_place)
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
    assert_eq!(
        compute_avasthas_blocking(&engine, old_settings)
            .unwrap_err()
            .code,
        "unsupported_schema_version"
    );
    let bad_number = ChartInput {
        lagna_number: Some(0),
        ..delhi.clone()
    };
    assert_eq!(
        compute_avasthas_blocking(&engine, bad_number)
            .unwrap_err()
            .code,
        "invalid_prashna_input"
    );
    // Sayı prashnası: lagna sayıdan gelir (30 → Yengeç, 4); öteki terimler aynıdır.
    let numbered = compute_avasthas_blocking(
        &engine,
        ChartInput {
            lagna_number: Some(30),
            ..delhi.clone()
        },
    )
    .expect("number prashna");
    let plain = compute_avasthas_blocking(&engine, delhi.clone()).expect("delhi");
    let (n, p) = (
        numbered.shayanadi_basis.expect("basis"),
        plain.shayanadi_basis.expect("basis"),
    );
    assert_eq!(n.lagna_sign_number, 4);
    assert_eq!((n.birth_nakshatra, n.ghatis), (p.birth_nakshatra, p.ghatis));

    // Performans: temel harita hesabı dahil ≤ 10 ms (yogalarla aynı bütçe; spec §15).
    let (median, max) = median_of_15(|| {
        compute_avasthas_blocking(&engine, delhi.clone()).expect("avasthas");
    });
    eprintln!("compute_avasthas median {median:?}, max {max:?}");
    assert!(
        median <= Duration::from_millis(10),
        "compute_avasthas median {median:?}"
    );
}
````

Fikstür üreticisi (paylaşılan dosya; üç ekleme):

````diff
diff --git a/app/src-tauri/tests/ui_fixture.rs b/app/src-tauri/tests/ui_fixture.rs
index 9e77aa0..9fad20f 100644
--- a/app/src-tauri/tests/ui_fixture.rs
+++ b/app/src-tauri/tests/ui_fixture.rs
@@ -11,6 +11,7 @@ use std::sync::Mutex;
 
 use atlas::Timestamp;
 use ephemeris::Engine;
+use jyotish_core::avastha::{Baladi, Jagradadi, Lajjitadi, Shayanadi};
 use jyotish_core::muhurta::{NakshatraNature, TithiGroup, MUHURTA_MAX_MIN_MINUTES};
 use jyotish_core::{
     kuta_class_ids, ChartInput, ChartQuery, Condition, Graha, GrahaFlag, Kuta, LordLink,
@@ -19,6 +20,7 @@ use jyotish_core::{
     MAX_SKY_EVENTS, MAX_YOGA_IDS, PRASHNA_NUMBER_MAX, YOGA_CATALOG, YOGA_CONDITIONS,
 };
 use jyotish_desk_lib::atlas_commands::{preview_moment_body, search_places_blocking, AtlasState};
+use jyotish_desk_lib::avastha_commands::compute_avasthas_blocking;
 use jyotish_desk_lib::client_commands::create_client_blocking;
 use jyotish_desk_lib::commands::dasha_children;
 use jyotish_desk_lib::module_commands::{
@@ -191,6 +193,23 @@ fn ui_fixture_is_up_to_date() {
         },
     )
     .expect("yogas without a birth time");
+    // Avasthalar (Plan 24): aynı girdiyle ve doğum saati bilinmeyen girdiyle (Lajjita
+    // değerlendirilmez, Shayanadi yoktur). Dört dizinin kimlikleri: arayüz testleri her birinin
+    // adını bu listelerden denetler.
+    let avasthas = compute_avasthas_blocking(&engine, module_input.clone()).expect("avasthas");
+    assert!(
+        avasthas.shayanadi_basis.is_some(),
+        "the fixture has a sunrise"
+    );
+    let avasthas_time_unknown = compute_avasthas_blocking(
+        &engine,
+        ChartInput {
+            time_known: false,
+            lagna_number: None,
+            ..module_input.clone()
+        },
+    )
+    .expect("avasthas without a birth time");
     let yoga_catalog: Vec<Value> = YOGA_CATALOG
         .iter()
         .map(|r| json!({ "id": r.id, "group": r.group }))
@@ -485,6 +504,14 @@ fn ui_fixture_is_up_to_date() {
         "yogas_time_unknown": yogas_time_unknown,
         "yoga_catalog": yoga_catalog,
         "yoga_conditions": YOGA_CONDITIONS,
+        "avasthas": avasthas,
+        "avasthas_time_unknown": avasthas_time_unknown,
+        "avastha_ids": {
+            "baladi": Baladi::ALL,
+            "jagradadi": Jagradadi::ALL,
+            "lajjitadi": Lajjitadi::ALL,
+            "shayanadi": Shayanadi::ALL,
+        },
         "compat": {
             "partner_uuid": partner.chart_record.uuid,
             "brief": partner_brief,
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-desk --test avastha --test ui_fixture 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası (`error[E0432]: unresolved import jyotish_desk_lib::avastha_commands`). Test koşmaz.

- [ ] **Step 3: Komut ve kayıt**

**Yeni dosya:** `app/src-tauri/src/avastha_commands.rs`

````rust
//! Avastha komutu (yol haritası §4.4, Plan 24). İleri modül komutlarının kalıbı: tek
//! `ChartInput`, hesap motor iş parçacığında, komut gövdesi `blocking` havuzunda, hata
//! `CommandError`. Rapor `Chart`'a eklenmez ve saklanmaz; panel açıkken istenir (K1).

use std::sync::Arc;

use ephemeris::{Engine, SwissEph};
use jyotish_core::avastha::AvasthaReport;
use jyotish_core::ChartInput;
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::CommandError;

/// `compute_avasthas` komutunun gövdesi; bloklar.
pub fn compute_avasthas_blocking(
    engine: &Engine,
    input: ChartInput,
) -> Result<AvasthaReport, CommandError> {
    let report = engine
        .call(move |se: &mut SwissEph| jyotish_core::avastha::compute_avasthas(se, &input))??;
    Ok(report)
}

/// Avasthalar: yedi grahanın Baladi, Jagradadi, Lajjitadi (küme) ve Shayanadi durumu. Doğum
/// saati bilinmiyorsa (`time_known == false`) Lajjita değerlendirilmez, Shayanadi `null` döner
/// ve rapor `time_known: false` taşır; doğum yerinde o gün gün doğumu yoksa yalnızca Shayanadi
/// `null`'dır. Girdi doğum verisidir: loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn compute_avasthas(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
) -> Result<AvasthaReport, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_avasthas_blocking(&engine, input)).await
}
````

Kayıt (paylaşılan dosya; iki satır). Komut satırı `commands::print_window,`'dan hemen sonra durur (D13):

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index a18fd91..4ee1dff 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -1,4 +1,5 @@
 pub mod atlas_commands;
+pub mod avastha_commands;
 pub mod client_commands;
 pub mod commands;
 pub mod data_dir;
@@ -32,6 +33,7 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             transit_commands::transits_at,
             commands::dasha_children,
             commands::print_window,
+            avastha_commands::compute_avasthas,
             module_commands::compute_drishti,
             module_commands::compute_jaimini,
             module_commands::compute_chara_dasha,
````

- [ ] **Step 4: Rust testleri; bağlamaları ve fikstürü yeniden üret (C10)**

```bash
cargo fmt --all
cargo test -p jyotish-desk --test avastha -- --nocapture 2>&1 | grep -E "compute_avasthas median|^test result"
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --test bindings --test ui_fixture 2>&1 | grep -E "^test result"
git diff --stat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
grep -c "compute_avasthas\|computeAvasthas" app/ui/bindings.ts
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
node -e 'const f=require("./app/ui/test/fixtures/delhi.json"); console.log(f.avasthas.grahas.length, f.avasthas.shayanadi_basis.ghatis, f.avasthas_time_unknown.shayanadi_basis, f.avastha_ids.shayanadi.length)'
```
Expected: ölçüm satırı (`compute_avasthas median …`; bu plan yazılırken 0,6 ms, bütçe 10 ms) ve `1 passed`; sonra üç `ok` satırı (`1 passed`, `2 passed`, sonra `1 passed` ve `2 passed`); `bindings.ts` 79 satır kazanır, `delhi.json` tek satırdır ve değişir; komut adı sayacı `2` (komut satırı ve bir belge yorumu); yasaklı ad sayacı `0`; son komut `7 25 null 12` yazar (yedi graha; Delhi fikstüründe 25 tamamlanmış ghati; saati bilinmeyen raporda ortak terim yok; on iki Shayanadi kimliği). İki dosya elle düzenlenmez. Entegrasyon testindeki `assert!(without_sunrise >= 1 …)` düşerse golden anlar değişmiştir: test gevşetilmez, bulgu notlara yazılır.

- [ ] **Step 5: Arayüz testleri: panel, sarmalayıcı, katalog; fikstür tipi**

`AvasthaPanel.test.tsx` on üç testtir: isteğin girdisi ve yedi satırın dört dizisi (Delhi fikstürü; beklenen değerlerin türetmesi yorumlarda); Shayanadi terim tablosu ve ortak terimler; "doğrulanmamış tablo" ve okuma notu; beş durumlu bir graha (liste öğeleri ve kuralı sağlayan grahalar); doğum saati bilinmeyen harita; gün doğumsuz rapor; düşük hassasiyet uyarısı; açık harita yokken istek yapılmaması; hata ve yeniden deneme; klavyeyle yeniden denemede odağın panelde durması; yükleme evreleri; ortak vurgu ve kaydırma kapları; katalog (kayıt, palet boyutu, üç dilde ad, Fransızca kesme işareti); adların Rust kimlikleriyle eşleşmesi ve üç dilde aynılığı.

**Yeni dosya:** `app/ui/panels/AvasthaPanel.test.tsx`

````tsx
import { act, fireEvent, screen, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeAvasthas: vi.fn() } }));

import { api, type Result } from "../api";
import type { AvasthaReport } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import { MODULE_PANELS, type Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { BALADI_NAMES, JAGRADADI_NAMES, LAJJITADI_NAMES, SHAYANADI_NAMES } from "./avasthaLabels";
import { AvasthaPanel } from "./AvasthaPanel";
import { PANEL_REGISTRY } from "./registry";

const ok = (report: AvasthaReport = fixture().avasthas): Result<AvasthaReport> => ({ status: "ok", data: report });

function renderPanel(opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  const panel: Panel = { id: "av", kind: "avastha", x: 0, y: 0, w: 7, h: 16, settings: {} };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<AvasthaPanel panel={panel} />, { layout, opened });
}

const statesTable = () => screen.findByRole("table", { name: "Avasthalar: yedi grahanın Baladi, Jagradadi, Lajjitadi ve Shayanadi durumu" });
const termsTable = () => screen.getByRole("table", { name: "Shayanadi terimleri" });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
/** Lajjitadi hücresi: liste öğeleri ayrı ayrı (ayraç CSS'tedir). */
const states = (row: HTMLElement) => [...row.querySelectorAll('td[data-series="lajjitadi"] li')].map((li) => li.textContent);

beforeEach(() => {
  vi.mocked(api.computeAvasthas).mockReset();
  vi.mocked(api.computeAvasthas).mockImplementation(async () => ok());
});

test("asks for the open chart's report and lists the four series of the seven grahas", async () => {
  renderPanel();
  const table = await statesTable();
  const opened = fixture().opened;
  expect(api.computeAvasthas).toHaveBeenCalledTimes(1);
  expect(api.computeAvasthas).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null });
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Graha", "Baladi", "Jagradadi", "Lajjitadi", "Shayanadi"]);
  const rows = within(table).getAllByRole("row").slice(1);
  // Delhi fikstürü. Lajjitadi sütunu aşağıda liste öğeleriyle ayrıca denetlenir.
  expect(rows.map((r) => [cells(r)[0], cells(r)[1], cells(r)[2], cells(r)[4]])).toEqual([
    ["Güneş", "Yuva", "Swapna", "Kautuka"],
    ["Ay", "Vriddha", "Swapna", "Sabha"],
    ["Mars", "Bala", "Swapna", "Shayana"],
    ["Merkür", "Kumara", "Swapna", "Sabha"],
    ["Jüpiter", "Bala", "Swapna", "Nidra"],
    ["Venüs", "Vriddha", "Swapna", "Shayana"],
    ["Satürn", "Yuva", "Sushupti", "Gamana"],
  ]);
  expect(rows.map(states)).toEqual([["Mudita (Jüpiter)"], [], [], ["Mudita (Güneş)"], ["Kshudhita (Satürn)", "Mudita (Ay)"], [], ["Kshudhita (Ay)", "Mudita (Jüpiter)"]]);
  // Hiçbir durumda olmayan graha: görünen çizgi, okunan sözcük.
  const moon = rows[1].querySelector('td[data-series="lajjitadi"]')!;
  expect(moon).toHaveTextContent("—hiçbiri");
  expect(moon.querySelector('[aria-hidden="true"]')).toHaveTextContent("—");
  expect(document.querySelector('[title="yaklaşık"]')).toBeNull();
  expect(screen.queryByText(/Doğum saati bilinmiyor/)).toBeNull();
  expect(screen.queryByText(/gün doğumu yok/)).toBeNull();
});

test("the Shayanadi terms come from Rust: three terms, the total and the remainder for each graha", async () => {
  renderPanel();
  await statesTable();
  const rows = within(termsTable()).getAllByRole("row");
  expect(cells(rows[0])).toEqual(["Graha", "Nakshatra (N)", "Graha sayısı (G)", "Navamsa dilimi (A)", "Toplam", "Artan"]);
  // Güneş: 20 × 1 × 5 + 15 + 25 + 3 = 143 = 11 × 12 + 11. Jüpiter: 1 × 5 × 1 + 43 = 48, artan 0.
  expect(rows.slice(1).map(cells)).toEqual([
    ["Güneş", "20", "1", "5", "143", "11"],
    ["Ay", "15", "2", "6", "223", "7"],
    ["Mars", "23", "3", "2", "181", "1"],
    ["Merkür", "19", "4", "3", "271", "7"],
    ["Jüpiter", "1", "5", "1", "48", "0"],
    ["Venüs", "17", "6", "3", "349", "1"],
    ["Satürn", "2", "7", "5", "113", "5"],
  ]);
  expect(screen.getByText("Ortak terimler: doğum nakshatrası 15 (Swati), ghati 25, lagna 3 (İki).")).toBeInTheDocument();
  expect(screen.getByText(/^Toplam = N × G × A \+ doğum nakshatrası \+ ghati \+ lagna\./)).toBeInTheDocument();
  expect(screen.getByText("Yöntem: avastha.bphs_v1")).toBeInTheDocument();
});

test("the unverified-table note and the reading note are always shown", async () => {
  renderPanel();
  await statesTable();
  expect(screen.getByText("Doğrulanmamış tablo: Lajjitadi kuralları ve Shayanadi formülü basılı kaynakla karşılaştırılmayı bekliyor.")).toBeInTheDocument();
  expect(screen.getByText(/^Burçlar ve evler rasi çerçevesindedir; dostluk ve düşmanlık doğal ilişkidir\./)).toBeInTheDocument();
});

test("a graha in several states lists them in rule order with the grahas that satisfy each rule", async () => {
  const report = fixture().avasthas;
  report.grahas[5].lajjitadi = [
    { state: "Lajjita", grahas: ["Sun"] },
    { state: "Garvita", grahas: [] },
    { state: "Kshudhita", grahas: ["Sun"] },
    { state: "Mudita", grahas: ["Saturn"] },
    { state: "Kshobhita", grahas: ["Sun", "Saturn"] },
  ];
  vi.mocked(api.computeAvasthas).mockResolvedValue(ok(report));
  renderPanel();
  const venus = within(await statesTable()).getAllByRole("row")[6];
  expect(states(venus)).toEqual(["Lajjita (Güneş)", "Garvita", "Kshudhita (Güneş)", "Mudita (Satürn)", "Kshobhita (Güneş, Satürn)"]);
  expect(within(venus).getByRole("list")).toBeInTheDocument();
  expect(within(venus).getAllByRole("listitem")).toHaveLength(5);
});

test("unknown birth time is sent to Rust; Lajjita and Shayanadi are absent and the Moon's row is approximate", async () => {
  vi.mocked(api.computeAvasthas).mockResolvedValue(ok(fixture().avasthas_time_unknown));
  renderPanel(unknownTimeOpened());
  const table = await statesTable();
  expect(vi.mocked(api.computeAvasthas).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByText("Doğum saati bilinmiyor: Lajjita değerlendirilmedi ve Shayanadi hesaplanmadı (ikisi de lagna ister). Ay'ın durumları yaklaşıktır (≈).")).toBeInTheDocument();
  expect(screen.queryByText(/gün doğumu yok/)).toBeNull();
  const rows = within(table).getAllByRole("row").slice(1);
  // Shayanadi sütunu durur ama boştur: çizgi ve "hesaplanmadı".
  for (const row of rows) expect(row.querySelector('td[data-series="shayanadi"]')).toHaveTextContent("—hesaplanmadı");
  expect(rows.flatMap(states).some((s) => s.startsWith("Lajjita"))).toBe(false);
  // Ay'ın Baladi ve Jagradadi hücreleri yaklaşıktır; öteki satırlar değildir.
  expect(rows[1].querySelector('td[data-series="baladi"]')).toHaveTextContent("≈yaklaşık Vriddha");
  expect(rows[1].querySelector('td[data-series="jagradadi"]')).toHaveTextContent("≈yaklaşık Swapna");
  expect(rows[0].querySelector('[title="yaklaşık"]')).toBeNull();
  expect(document.querySelectorAll('[title="yaklaşık"]')).toHaveLength(2);
  // Terim tablosu ve ortak terimler yoktur.
  expect(screen.getAllByRole("table")).toHaveLength(1);
  expect(screen.queryByText(/^Ortak terimler/)).toBeNull();
  expect(screen.queryByText(/^Toplam = /)).toBeNull();
});

test("no sunrise on the birth day: only Shayanadi is missing and the panel says why", async () => {
  const report = fixture().avasthas;
  report.shayanadi_basis = null;
  for (const row of report.grahas) row.shayanadi = null;
  vi.mocked(api.computeAvasthas).mockResolvedValue(ok(report));
  renderPanel();
  const table = await statesTable();
  expect(screen.getByText("Doğum yerinde o gün gün doğumu yok: Shayanadi hesaplanmadı (formül gün doğumundan geçen ghatiyi ister).")).toBeInTheDocument();
  expect(screen.queryByText(/Doğum saati bilinmiyor/)).toBeNull();
  const rows = within(table).getAllByRole("row").slice(1);
  expect(states(rows[6])).toEqual(["Kshudhita (Ay)", "Mudita (Jüpiter)"]);
  expect(rows[6].querySelector('td[data-series="shayanadi"]')).toHaveTextContent("—hesaplanmadı");
  expect(document.querySelector('[title="yaklaşık"]')).toBeNull();
  expect(screen.getAllByRole("table")).toHaveLength(1);
});

test("a low-precision warning from the report is shown as text", async () => {
  const report = fixture().avasthas;
  report.warnings = ["LowPrecision"];
  vi.mocked(api.computeAvasthas).mockResolvedValue(ok(report));
  renderPanel();
  await statesTable();
  expect(screen.getByText("Düşük hassasiyet")).toBeInTheDocument();
  expect(screen.getByText(/Güneş veya Ay yedek ephemeris ile hesaplandı\./)).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderPanel(null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeAvasthas).not.toHaveBeenCalled();
});

test("an error is shown in the panel and retry asks again", async () => {
  vi.mocked(api.computeAvasthas).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.queryByRole("table")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await statesTable();
  expect(api.computeAvasthas).toHaveBeenCalledTimes(2);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("retry from the keyboard keeps the focus in the panel while the error box is replaced", async () => {
  vi.mocked(api.computeAvasthas).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  const panel: Panel = { id: "av", kind: "avastha", x: 0, y: 0, w: 7, h: 16, settings: {} };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Avasthalar">
      <AvasthaPanel panel={panel} />
    </section>,
    { layout, opened: fixture().opened },
  );
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  await statesTable();
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).not.toBe(document.body);
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Avasthalar" }));
});

test("loading: nothing under a second, then a skeleton, then the tables", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<AvasthaReport>) => void;
    vi.mocked(api.computeAvasthas).mockReturnValueOnce(new Promise((r) => (resolve = r)));
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

test("shared highlight: a pinned graha marks its row in both tables; both tables scroll by keyboard", async () => {
  renderPanel();
  const table = await statesTable();
  const saturn = within(table).getByRole("button", { name: "Satürn" });
  fireEvent.click(saturn);
  expect(saturn).toHaveAttribute("aria-pressed", "true");
  expect(document.querySelectorAll("tr.is-highlighted")).toHaveLength(2);
  expect(document.querySelector('tr[data-graha="Saturn"]')).toHaveClass("is-highlighted");
  expect(document.querySelector('tr[data-terms-of="Saturn"]')).toHaveClass("is-highlighted");
  fireEvent.keyDown(window, { key: "Escape" });
  expect(saturn).toHaveAttribute("aria-pressed", "false");
  expect(document.querySelectorAll("tr.is-highlighted")).toHaveLength(0);
  // Kaydırma kapları klavye odağı alır ve tablolarının adını taşır.
  const groups = screen.getAllByRole("group");
  expect(groups.map((g) => g.getAttribute("tabindex"))).toEqual(["0", "0"]);
  expect(groups[1]).toHaveAccessibleName("Shayanadi terimleri");
});

test("catalog: the panel is registered, opens from the palette at 7 × 16 and is named in three languages", () => {
  expect(PANEL_REGISTRY.avastha.component).toBe(AvasthaPanel);
  expect(PANEL_REGISTRY.avastha.titleKey).toBe("panel.avastha");
  expect(MODULE_PANELS.find((m) => m.kind === "avastha")).toEqual({ kind: "avastha", w: 7, h: 16 });
  expect([tr.panel.avastha, en.panel.avastha, fr.panel.avastha]).toEqual(["Avasthalar", "Avasthas", "Avasthas"]);
  // Fransızca metinlerde düz kesme işareti yoktur (C12).
  for (const text of Object.values(fr.avastha)) expect(text).not.toContain("'");
});

test("every avastha id that Rust can return has a name, and the names are the same in every language", () => {
  const ids = fixture().avastha_ids;
  expect(Object.keys(BALADI_NAMES)).toEqual(ids.baladi);
  expect(Object.keys(JAGRADADI_NAMES)).toEqual(ids.jagradadi);
  expect(Object.keys(LAJJITADI_NAMES)).toEqual(ids.lajjitadi);
  expect(Object.keys(SHAYANADI_NAMES)).toEqual(ids.shayanadi);
  const names = [BALADI_NAMES, JAGRADADI_NAMES, LAJJITADI_NAMES, SHAYANADI_NAMES].flatMap((m) => Object.values(m));
  expect(new Set(names).size).toBe(5 + 3 + 6 + 12);
  expect(SHAYANADI_NAMES.NrityaLipsa).toBe("Nritya Lipsa");
  // Dizi adları transliterasyondur: üç dilde aynıdır (K11).
  for (const key of ["baladi", "jagradadi", "lajjitadi", "shayanadi"] as const) {
    expect([en.avastha[key], fr.avastha[key]]).toEqual([tr.avastha[key], tr.avastha[key]]);
  }
});
````

Paylaşılan test ve fikstür dosyaları (ekleme):

````diff
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index abf51e5..7ad8eae 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -35,6 +35,7 @@ vi.mock("./bindings", () => ({
     charaDashaAt: vi.fn(),
     computeShadbala: vi.fn(),
     computeYogas: vi.fn(),
+    computeAvasthas: vi.fn(),
     computeCompat: vi.fn(),
     chartInput: vi.fn(),
     chartBrief: vi.fn(),
@@ -139,6 +140,7 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["charaDashaAt", [{ jd_ut: 1, time_known: true }, 2451545.5]],
     ["computeShadbala", [{ jd_ut: 1, time_known: true }]],
     ["computeYogas", [{ jd_ut: 1, time_known: false }]],
+    ["computeAvasthas", [{ jd_ut: 1, time_known: false }]],
     ["computeCompat", [{ jd_ut: 1, time_known: true }, { jd_ut: 2, time_known: false }]],
     ["chartInput", ["00000000-0000-4000-8000-000000000012"]],
     ["chartBrief", ["00000000-0000-4000-8000-000000000012"]],
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index 08fa4b3..54dce66 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -48,6 +48,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "shadbala",
     "bhava_bala",
     "yogas",
+    "avastha",
     "compat",
     "muhurta_day",
     "muhurta_search",
@@ -60,7 +61,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas", "avastha", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
diff --git a/app/ui/test/fixture.ts b/app/ui/test/fixture.ts
index 62b0474..21f3a06 100644
--- a/app/ui/test/fixture.ts
+++ b/app/ui/test/fixture.ts
@@ -5,6 +5,8 @@
 // gün panchangı (iki gün), üç dilimlik bir muhurta araması ve iki prashna haritası (olgularıyla).
 // Arayüz hesap yapmadığı için testler bunu çizer.
 import type {
+  AvasthaReport,
+  Baladi,
   CharaDasha,
   ChartBrief,
   ChartInput,
@@ -19,10 +21,12 @@ import type {
   Graha,
   GrahaFlag,
   HomePlace,
+  Jagradadi,
   JaiminiReport,
   KarakaRole,
   Kuta,
   KutaReport,
+  Lajjitadi,
   LordLink,
   MomentPreview,
   MuhurtaFlag,
@@ -38,6 +42,7 @@ import type {
   RisingMode,
   SavedSearch,
   ShadbalaReport,
+  Shayanadi,
   Sign,
   SignDashaPath,
   SignDashaPeriod,
@@ -82,6 +87,12 @@ export interface UiFixture {
   yoga_catalog: { id: string; group: YogaGroup }[];
   /** Rust `YOGA_CONDITIONS`: alt koşul kimlikleri. */
   yoga_conditions: string[];
+  /** `compute_avasthas`, açık haritanın girdisiyle: yedi graha, dört dizi, Shayanadi terimleri. */
+  avasthas: AvasthaReport;
+  /** `compute_avasthas`, `time_known: false` ile: Lajjita değerlendirilmez, Shayanadi yoktur. */
+  avasthas_time_unknown: AvasthaReport;
+  /** Rust `Baladi::ALL`, `Jagradadi::ALL`, `Lajjitadi::ALL` ve `Shayanadi::ALL`. */
+  avastha_ids: { baladi: Baladi[]; jagradadi: Jagradadi[]; lajjitadi: Lajjitadi[]; shayanadi: Shayanadi[] };
   /**
    * Uyum: ikinci danışan ("Eş Danışan", İstanbul 1987-03-17) eş haritadır. `input` ve `brief`
    * store'un `chart_input` ve `chart_brief` dönüşleridir. `report` açık harita damat rolündeyken,
````

- [ ] **Step 6: Testlerin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/panels/AvasthaPanel.test.tsx ui/api.test.ts ui/layout/model.test.ts 2>&1 | grep -E "FAIL|Test Files|Tests "`
Expected: FAIL: `AvasthaPanel.test.tsx` `Failed to resolve import "./AvasthaPanel"` ile yüklenmez; `api.test.ts`'in sarmalayıcı testi (`call is not a function`) ve `model.test.ts`'in iki katalog testi düşer. `Test Files  3 failed (3)`, `Tests  3 failed | 29 passed (32)`.

- [ ] **Step 7: Sarmalayıcı, kanca, adlar, panel, katalog ve metinler**

````diff
diff --git a/app/ui/api.ts b/app/ui/api.ts
index b3b76a1..139179a 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -104,6 +104,9 @@ export const api = {
   // Yogalar: doğum saati bilinmiyorsa rapor yine döner; lagna isteyen kurallar atlanmış,
   // diğer kayıtlar `approximate` işaretlidir.
   computeYogas: (input: ChartInput) => wrap(() => commands.computeYogas(input)),
+  // Avasthalar: doğum saati bilinmiyorsa rapor yine döner; Lajjita değerlendirilmemiş, Shayanadi
+  // hesaplanmamıştır (`time_known: false`). O gün gün doğumu yoksa yalnızca Shayanadi boştur.
+  computeAvasthas: (input: ChartInput) => wrap(() => commands.computeAvasthas(input)),
   // Uyum: `groom` damat rolünün, `bride` gelin rolünün girdisidir. Eş haritanın girdisi ve kısa
   // kimliği store'dan okunur; harita yoksa (silinmiş eş) ikisi de `null` döner. Girdiler doğum
   // verisidir, eş bağı ilişki bilgisidir: loglanmaz.
````

**Yeni dosya:** `app/ui/modules/useAvasthas.ts`

````ts
// Avastha raporunu (`compute_avasthas`) isteyen ortak kanca. Avastha paneli ve graha tablosunun
// Baladi sütunu aynı raporu okur. Rapor saklanmaz ve arayüzde hesap yoktur (K1, C4); her
// tüketici kendi isteğini yapar (komut 1 ms'nin altındadır). Doğum saati bilinmiyorsa rapor
// yine döner: Lajjita değerlendirilmemiş, Shayanadi hesaplanmamıştır (`time_known: false`).
import { api } from "../api";
import type { AvasthaReport, OpenedClient } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

/** `enabled === false`: rapor gerekmiyor (ör. sütun kapalı), komut çağrılmaz. */
export function useAvasthas(opened: OpenedClient | null, enabled = true): ModuleState<AvasthaReport> {
  return useModule(opened !== null && enabled ? inputKey(opened) : null, () => api.computeAvasthas(chartInput(opened as OpenedClient)));
}
````

**Yeni dosya:** `app/ui/panels/avasthaLabels.ts`

````ts
// Avastha adları: Sanskritçe terimlerdir ve dile göre değişmez (K11). Kimlikler Rust
// enum'larıdır; bir test bu tabloları fikstürdeki `avastha_ids` ile karşılaştırır.
import type { Baladi, Jagradadi, Lajjitadi, Shayanadi } from "../bindings";

export const BALADI_NAMES: Record<Baladi, string> = {
  Bala: "Bala",
  Kumara: "Kumara",
  Yuva: "Yuva",
  Vriddha: "Vriddha",
  Mrita: "Mrita",
};

export const JAGRADADI_NAMES: Record<Jagradadi, string> = {
  Jagrat: "Jagrat",
  Swapna: "Swapna",
  Sushupti: "Sushupti",
};

export const LAJJITADI_NAMES: Record<Lajjitadi, string> = {
  Lajjita: "Lajjita",
  Garvita: "Garvita",
  Kshudhita: "Kshudhita",
  Trushita: "Trushita",
  Mudita: "Mudita",
  Kshobhita: "Kshobhita",
};

export const SHAYANADI_NAMES: Record<Shayanadi, string> = {
  Shayana: "Shayana",
  Upaveshana: "Upaveshana",
  Netrapani: "Netrapani",
  Prakashana: "Prakashana",
  Gamana: "Gamana",
  Agamana: "Agamana",
  Sabha: "Sabha",
  Agama: "Agama",
  Bhojana: "Bhojana",
  NrityaLipsa: "Nritya Lipsa",
  Kautuka: "Kautuka",
  Nidra: "Nidra",
};
````

**Yeni dosya:** `app/ui/panels/avastha.css`

````css
/* Avastha paneli: Lajjitadi hücresindeki durum listesi. Renk token'lardan gelir (C9). */
.avastha-states {
  list-style: none;
  margin: 0;
  padding: 0;
}
.avastha-states li {
  display: inline;
}
.avastha-states li + li::before {
  content: "; ";
}
.avastha-by {
  color: var(--text-2);
}
````

**Yeni dosya:** `app/ui/panels/AvasthaPanel.tsx`

````tsx
// Avastha paneli (yol haritası §4.4): yedi grahanın Baladi, Jagradadi, Lajjitadi ve Shayanadi
// durumu; altında Shayanadi formülünün terimleri. Değerler `compute_avasthas`'tan gelir ve panel
// açıkken istenir (K1); burada hesap yoktur (C4): artan da Rust'ın alanıdır. Lajjitadi bir
// kümedir: bir graha birden çok durumda olabilir ya da hiçbirinde olmayabilir. Yaklaşık değer
// `≈` ve metinle, boş değer çizgi ve metinle gösterilir; renk anlam taşımaz (C9). Yorum ve
// hüküm yoktur.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { AvasthaReport, Graha, GrahaAvastha } from "../bindings";
import { useHighlight } from "../highlight/store";
import { useAvasthas } from "../modules/useAvasthas";
import { useSession } from "../session/store";
import { Approx } from "./Approx";
import "./avastha.css";
import { BALADI_NAMES, JAGRADADI_NAMES, LAJJITADI_NAMES, SHAYANADI_NAMES } from "./avasthaLabels";
import { BalaScroll } from "./bala";
import { NAKSHATRA_NAMES } from "./labels";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

const DASH = "—";

export function AvasthaPanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const module = useAvasthas(opened);
  const phase = usePanelLoadPhase(module.loading);
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (module.data === null) return phase === "skeleton" ? <PanelSkeleton /> : null;
  return <Avasthas report={module.data} />;
}

/** Boş değer: görünen çizgi ve ekran okuyucu için sözcük. */
function Empty({ label }: { label: string }) {
  return (
    <>
      <span aria-hidden="true">{DASH}</span>
      <span className="visually-hidden">{label}</span>
    </>
  );
}

function Avasthas({ report }: { report: AvasthaReport }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const id = useId();
  const on = (g: Graha) => active?.type === "graha" && active.id === g;
  // Saat bilinmiyorsa Ay'ın derecesi ve burcu gün içinde değişebilir: Ay'ın satırı yaklaşıktır.
  const approx = (row: GrahaAvastha) => !report.time_known && row.graha === "Moon";
  const basis = report.shayanadi_basis;

  return (
    <div className="module-panel">
      {!report.time_known && <p className="panel-notice">{t("avastha.timeUnknown")}</p>}
      {report.time_known && basis === null && <p className="panel-notice">{t("avastha.noSunrise")}</p>}
      {report.warnings.map((w) => (
        <p key={w} className="panel-notice">
          <strong>{t(`badge.${w}`)}</strong>: {t(`badgeHint.${w}`)}
        </p>
      ))}
      <p className="panel-notice">{t("avastha.unverified")}</p>
      <BalaScroll labelledBy={`${id}-states`}>
        <table className="module-table">
          <caption id={`${id}-states`} className="visually-hidden">
            {t("avastha.caption")}
          </caption>
          <thead>
            <tr>
              <th scope="col">{t("table.graha")}</th>
              <th scope="col">{t("avastha.baladi")}</th>
              <th scope="col">{t("avastha.jagradadi")}</th>
              <th scope="col">{t("avastha.lajjitadi")}</th>
              <th scope="col">{t("avastha.shayanadi")}</th>
            </tr>
          </thead>
          <tbody>
            {report.grahas.map((row) => (
              <tr key={row.graha} data-graha={row.graha} className={on(row.graha) ? "is-highlighted" : undefined}>
                <th scope="row">
                  <PinButton target={{ type: "graha", id: row.graha }}>{t(`grahaName.${row.graha}`)}</PinButton>
                </th>
                <td data-series="baladi">
                  <Approx on={approx(row)} label={t("table.approx")}>
                    {BALADI_NAMES[row.baladi]}
                  </Approx>
                </td>
                <td data-series="jagradadi">
                  <Approx on={approx(row)} label={t("table.approx")}>
                    {JAGRADADI_NAMES[row.jagradadi]}
                  </Approx>
                </td>
                <td data-series="lajjitadi">
                  {row.lajjitadi.length === 0 ? (
                    <Empty label={t("avastha.none")} />
                  ) : (
                    <Approx on={approx(row)} label={t("table.approx")}>
                      <ul className="avastha-states">
                        {row.lajjitadi.map((hit) => (
                          <li key={hit.state} data-state={hit.state}>
                            {LAJJITADI_NAMES[hit.state]}
                            {hit.grahas.length > 0 && <span className="avastha-by"> ({hit.grahas.map((g) => t(`grahaName.${g}`)).join(", ")})</span>}
                          </li>
                        ))}
                      </ul>
                    </Approx>
                  )}
                </td>
                <td data-series="shayanadi">{row.shayanadi ? SHAYANADI_NAMES[row.shayanadi.state] : <Empty label={t("avastha.notComputed")} />}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </BalaScroll>
      {basis !== null && (
        <>
          <BalaScroll labelledBy={`${id}-terms`}>
            <table className="module-table">
              <caption id={`${id}-terms`}>{t("avastha.termsCaption")}</caption>
              <thead>
                <tr>
                  <th scope="col">{t("table.graha")}</th>
                  {(["nakshatra", "grahaNumber", "navamsa", "total", "remainder"] as const).map((k) => (
                    <th key={k} scope="col" className="num">
                      {t(`avastha.${k}`)}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {report.grahas.map(
                  (row) =>
                    row.shayanadi && (
                      <tr key={row.graha} data-terms-of={row.graha} className={on(row.graha) ? "is-highlighted" : undefined}>
                        <th scope="row">{t(`grahaName.${row.graha}`)}</th>
                        <td className="num">{row.shayanadi.nakshatra}</td>
                        <td className="num">{row.shayanadi.graha_number}</td>
                        <td className="num">{row.shayanadi.navamsa}</td>
                        <td className="num">{row.shayanadi.total}</td>
                        <td className="num">{row.shayanadi.remainder}</td>
                      </tr>
                    ),
                )}
              </tbody>
            </table>
          </BalaScroll>
          <p className="panel-message">
            {t("avastha.basis", {
              nakshatra: basis.birth_nakshatra,
              name: NAKSHATRA_NAMES[basis.birth_nakshatra - 1] ?? DASH,
              ghatis: basis.ghatis,
              lagna: basis.lagna_sign_number,
              sign: t(`sign.${basis.lagna_sign_number - 1}`),
            })}
          </p>
          <p className="panel-message">{t("avastha.formula")}</p>
        </>
      )}
      <p className="panel-message">{t("avastha.note")}</p>
      <p className="panel-message">{t("avastha.method", { method: report.method })}</p>
    </div>
  );
}
````

Katalog (paylaşılan dosyalar; ekleme). `avastha`, `yogas`'tan hemen sonra durur (D14):

````diff
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index c3629ce..f7ba657 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -19,6 +19,7 @@ export const PANEL_KINDS = [
   "shadbala",
   "bhava_bala",
   "yogas",
+  "avastha",
   "compat",
   "muhurta_day",
   "muhurta_search",
@@ -44,6 +45,8 @@ export const MODULE_PANELS = [
   { kind: "bhava_bala", w: 6, h: 14 },
   // Yogalar (Plan 18).
   { kind: "yogas", w: 7, h: 14 },
+  // Avasthalar (Plan 24).
+  { kind: "avastha", w: 7, h: 16 },
   // Uyum (Plan 19).
   { kind: "compat", w: 7, h: 16 },
   // Muhurta (Plan 20).
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index cee04ca..cf12cc8 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -7,6 +7,7 @@ import { NotesPanel } from "../notes/NotesPanel";
 import { ArgalaPanel } from "./ArgalaPanel";
 import { ArudhaPanel } from "./ArudhaPanel";
 import { AshtakavargaPanel } from "./AshtakavargaPanel";
+import { AvasthaPanel } from "./AvasthaPanel";
 import { BhavaBalaPanel } from "./BhavaBalaPanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel, chartVarga } from "./ChartPanel";
@@ -50,6 +51,8 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   bhava_bala: { titleKey: "panel.bhava_bala", component: BhavaBalaPanel },
   // Yogalar (Plan 18).
   yogas: { titleKey: "panel.yogas", component: YogasPanel },
+  // Avasthalar (Plan 24).
+  avastha: { titleKey: "panel.avastha", component: AvasthaPanel },
   // Uyum (Plan 19).
   compat: { titleKey: "panel.compat", component: CompatPanel },
   // Muhurta (Plan 20).
````

Metinler (paylaşılan dosyalar; üç dilde aynı iki ekleme: `panel.avastha` ve `avastha` bloğu). Fransızca metinlerde kesme işareti tipografiktir (’):

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 17db154..f5bb3f5 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -72,6 +72,7 @@ export const en: Messages = {
     shadbala: "Shadbala",
     bhava_bala: "Bhava bala",
     yogas: "Yogas",
+    avastha: "Avasthas",
     compat: "Compatibility",
     muhurta_day: "Day panchang",
     muhurta_search: "Muhurta search",
@@ -292,6 +293,28 @@ export const en: Messages = {
     note: "Only D1 is evaluated. Houses are rasi houses counted from the lagna sign and do not depend on the house system setting.",
     catalog: "Catalog version: {{version}}",
   },
+  avastha: {
+    caption: "Avasthas: the Baladi, Jagradadi, Lajjitadi and Shayanadi state of the seven grahas",
+    baladi: "Baladi",
+    jagradadi: "Jagradadi",
+    lajjitadi: "Lajjitadi",
+    shayanadi: "Shayanadi",
+    none: "none",
+    notComputed: "not computed",
+    termsCaption: "Shayanadi terms",
+    nakshatra: "Nakshatra (N)",
+    grahaNumber: "Graha number (G)",
+    navamsa: "Navamsa part (A)",
+    total: "Total",
+    remainder: "Remainder",
+    basis: "Shared terms: birth nakshatra {{nakshatra}} ({{name}}), ghatis {{ghatis}}, lagna {{lagna}} ({{sign}}).",
+    formula: "Total = N × G × A + birth nakshatra + ghatis + lagna. The state is the remainder of the total divided by 12; remainder 0 is the twelfth state.",
+    timeUnknown: "Birth time unknown: Lajjita was not evaluated and Shayanadi was not computed (both need the lagna). The Moon's states are approximate (≈).",
+    noSunrise: "There is no sunrise at the birth place on that day: Shayanadi was not computed (the formula needs the ghatis since sunrise).",
+    unverified: "Unverified table: the Lajjitadi rules and the Shayanadi formula are waiting to be checked against a printed source.",
+    note: "Signs and houses are in the rasi frame; friendship and enmity are the natural relation. Lajjitadi is a set: the grahas in brackets are the ones that satisfy the rule. Rahu and Ketu get no avastha.",
+    method: "Method: {{method}}",
+  },
   compat: {
     empty: "Choose a partner chart for compatibility.",
     pick: "Choose a partner chart",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 9b6621f..3f91bde 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -72,6 +72,7 @@ export const fr: Messages = {
     shadbala: "Shadbala",
     bhava_bala: "Bhava bala",
     yogas: "Yogas",
+    avastha: "Avasthas",
     compat: "Compatibilité",
     muhurta_day: "Panchang du jour",
     muhurta_search: "Recherche de muhurta",
@@ -292,6 +293,28 @@ export const fr: Messages = {
     note: "Seul le D1 est évalué. Les maisons sont les maisons rasi comptées depuis le signe du lagna et ne dépendent pas du système de maisons choisi.",
     catalog: "Version du catalogue : {{version}}",
   },
+  avastha: {
+    caption: "Avasthas : état Baladi, Jagradadi, Lajjitadi et Shayanadi des sept grahas",
+    baladi: "Baladi",
+    jagradadi: "Jagradadi",
+    lajjitadi: "Lajjitadi",
+    shayanadi: "Shayanadi",
+    none: "aucun",
+    notComputed: "non déterminé",
+    termsCaption: "Termes du Shayanadi",
+    nakshatra: "Nakshatra (N)",
+    grahaNumber: "Nombre du graha (G)",
+    navamsa: "Partie de navamsa (A)",
+    total: "Total",
+    remainder: "Reste",
+    basis: "Termes communs : nakshatra de naissance {{nakshatra}} ({{name}}), ghatis {{ghatis}}, lagna {{lagna}} ({{sign}}).",
+    formula: "Total = N × G × A + nakshatra de naissance + ghatis + lagna. L’état est le reste de la division du total par 12 ; le reste 0 est le douzième état.",
+    timeUnknown: "Heure de naissance inconnue : Lajjita n’a pas été évalué et le Shayanadi n’a pas été déterminé (les deux demandent le lagna). Les états de la Lune sont approximatifs (≈).",
+    noSunrise: "Pas de lever du Soleil ce jour-là au lieu de naissance : le Shayanadi n’a pas été déterminé (la formule demande les ghatis écoulés depuis le lever).",
+    unverified: "Table non vérifiée : les règles du Lajjitadi et la formule du Shayanadi attendent d’être comparées à une source imprimée.",
+    note: "Les signes et les maisons sont dans le cadre rasi ; l’amitié et l’inimitié sont la relation naturelle. Le Lajjitadi est un ensemble : les grahas entre parenthèses sont ceux qui remplissent la règle. Rahu et Ketu ne reçoivent pas d’avastha.",
+    method: "Méthode : {{method}}",
+  },
   compat: {
     empty: "Choisissez une carte partenaire pour la compatibilité.",
     pick: "Choisir une carte partenaire",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 8b5784d..9acf8b2 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -71,6 +71,7 @@ export const tr = {
     shadbala: "Shadbala",
     bhava_bala: "Bhava bala",
     yogas: "Yogalar",
+    avastha: "Avasthalar",
     compat: "Uyum",
     muhurta_day: "Gün panchangı",
     muhurta_search: "Muhurta araması",
@@ -291,6 +292,28 @@ export const tr = {
     note: "Yalnızca D1 değerlendirilir. Evler lagna burcundan sayılan rasi evleridir ve ev sistemi ayarından bağımsızdır.",
     catalog: "Katalog sürümü: {{version}}",
   },
+  avastha: {
+    caption: "Avasthalar: yedi grahanın Baladi, Jagradadi, Lajjitadi ve Shayanadi durumu",
+    baladi: "Baladi",
+    jagradadi: "Jagradadi",
+    lajjitadi: "Lajjitadi",
+    shayanadi: "Shayanadi",
+    none: "hiçbiri",
+    notComputed: "hesaplanmadı",
+    termsCaption: "Shayanadi terimleri",
+    nakshatra: "Nakshatra (N)",
+    grahaNumber: "Graha sayısı (G)",
+    navamsa: "Navamsa dilimi (A)",
+    total: "Toplam",
+    remainder: "Artan",
+    basis: "Ortak terimler: doğum nakshatrası {{nakshatra}} ({{name}}), ghati {{ghatis}}, lagna {{lagna}} ({{sign}}).",
+    formula: "Toplam = N × G × A + doğum nakshatrası + ghati + lagna. Durum, toplamın 12'ye bölümünden artandır; artan 0 on ikinci durumdur.",
+    timeUnknown: "Doğum saati bilinmiyor: Lajjita değerlendirilmedi ve Shayanadi hesaplanmadı (ikisi de lagna ister). Ay'ın durumları yaklaşıktır (≈).",
+    noSunrise: "Doğum yerinde o gün gün doğumu yok: Shayanadi hesaplanmadı (formül gün doğumundan geçen ghatiyi ister).",
+    unverified: "Doğrulanmamış tablo: Lajjitadi kuralları ve Shayanadi formülü basılı kaynakla karşılaştırılmayı bekliyor.",
+    note: "Burçlar ve evler rasi çerçevesindedir; dostluk ve düşmanlık doğal ilişkidir. Lajjitadi bir kümedir: parantezdeki grahalar kuralı sağlayanlardır. Rahu ve Ketu'ya avastha atanmaz.",
+    method: "Yöntem: {{method}}",
+  },
   compat: {
     empty: "Uyum için bir eş harita seçin.",
     pick: "Eş harita seç",
````

- [ ] **Step 8: Testler ve kapılar**

```bash
pnpm --dir app exec vitest run ui/panels/AvasthaPanel.test.tsx ui/api.test.ts ui/layout/model.test.ts ui/i18n ui/panels/panels.test.tsx 2>&1 | grep -E "FAIL|Test Files|Tests "
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: `5 passed (5)`, `70 passed (70)`; Vitest `89 passed (89)`, `901 passed (901)`; `✓ built`; fmt ve clippy çıktısız; sayaç `0`. Panel testindeki bir beklenti tutmazsa önce fikstürün o satırı D3–D8'den elle türetilir (testin yorumlarındaki aritmetik); beklenti ekrana bakarak değiştirilmez.

- [ ] **Step 9: Commit**

```bash
git add app/src-tauri/src/avastha_commands.rs app/src-tauri/src/lib.rs app/src-tauri/tests/avastha.rs app/src-tauri/tests/ui_fixture.rs app/ui/bindings.ts app/ui/test/fixtures/delhi.json app/ui/test/fixture.ts app/ui/api.ts app/ui/api.test.ts app/ui/modules/useAvasthas.ts app/ui/panels/avasthaLabels.ts app/ui/panels/avastha.css app/ui/panels/AvasthaPanel.tsx app/ui/panels/AvasthaPanel.test.tsx app/ui/panels/registry.ts app/ui/layout/model.ts app/ui/layout/model.test.ts app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(app): compute_avasthas command, fixture and the Avastha panel" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: yirmi dosya; tarama çıktısız; `git status --short` boş.

---

### Task 7: Arayüz: graha tablosunun Baladi sütunu; uçtan uca akışlar

Yol haritası §5 Plan 24 görev (7): "graha tablosuna isteğe bağlı Baladi sütunu"; K1, K10, K12; D18, D20. Sütunun testi ve uçtan uca akışlar yeni dosyalardadır; `GrahaTablePanel.test.tsx` ve `desk.spec.ts` değişmez.

**Files:**
- Create: `app/ui/panels/GrahaTableBaladi.test.tsx`, `app/e2e/avastha.desk.spec.ts`
- Modify: `app/ui/panels/GrahaTablePanel.tsx`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`, `app/e2e/ipc.ts`
- Test: `GrahaTableBaladi.test.tsx` (6); `avastha.desk.spec.ts` (2 akış)

**Interfaces:**
- Consumes: Task 6'nın `useAvasthas`, `BALADI_NAMES`, `api.computeAvasthas`, fikstür alanları ve `avastha` paneli; graha tablosunun var olan Karaka sütunu kalıbı (`settings.karaka`).
- Produces: `graha_table.settings.baladi: boolean` (varsayılan kapalı); i18n `table.baladi`; e2e taklidinde `compute_avasthas`.

- [ ] **Step 1: Testler: sütun ve iki uçtan uca akış; komutun taklidi**

`GrahaTableBaladi.test.tsx`: sütun varsayılan olarak kapalıdır ve istek yapılmaz; açıkken yedi grahanın basamağı raporlardan gelir, düğümler ve lagna satırı boştur ve Karaka raporu istenmez; iki sütun birlikte açıkken Baladi sondadır; doğum saati bilinmiyorsa yalnız Ay'ın hücresi "≈" taşır; onay kutusu yalnızca `settings.baladi`'yi yazar; başarısız istek tabloyu bozmaz ve yeniden denenir.

**Yeni dosya:** `app/ui/panels/GrahaTableBaladi.test.tsx`

````tsx
// Graha tablosunun isteğe bağlı Baladi sütunu (Plan 24). Ayrı dosyadır: `GrahaTablePanel.test.tsx`
// paralel planlarla paylaşılır.
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeJaimini: vi.fn(), computeAvasthas: vi.fn() } }));

import { api } from "../api";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { GrahaTablePanel } from "./GrahaTablePanel";

beforeEach(() => {
  vi.mocked(api.computeJaimini).mockReset();
  vi.mocked(api.computeJaimini).mockImplementation(async () => ({ status: "ok", data: fixture().jaimini }));
  vi.mocked(api.computeAvasthas).mockReset();
  vi.mocked(api.computeAvasthas).mockImplementation(async () => ({ status: "ok", data: fixture().avasthas }));
});

function renderTable(settings: Record<string, unknown> = {}, opened = fixture().opened) {
  const panel: Panel = { id: "t", kind: "graha_table", x: 0, y: 0, w: 6, h: 8, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<GrahaTablePanel panel={panel} />, { layout, opened });
}

const headers = () => screen.getAllByRole("columnheader").map((h) => h.textContent);
const baladi = (g: string) => document.querySelector(`[data-baladi="${g}"]`)!;

test("the Baladi column is off by default and asks for nothing", () => {
  renderTable();
  expect(headers()).not.toContain("Baladi");
  expect(api.computeAvasthas).not.toHaveBeenCalled();
  expect(document.querySelector("[data-baladi]")).toBeNull();
});

test("the Baladi column shows each graha's stage from the avastha report; the nodes and the lagna have none", async () => {
  renderTable({ columns: ["sign"], baladi: true });
  expect(headers()).toEqual(["Graha", "Burç", "Baladi"]);
  await waitFor(() => expect(baladi("Sun")).toHaveTextContent("Yuva"));
  expect(api.computeAvasthas).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.computeAvasthas).mock.calls[0][0]).toMatchObject({ jd_ut: 2451545, time_known: true, lagna_number: null });
  expect(["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"].map((g) => baladi(g).textContent)).toEqual([
    "Yuva",
    "Vriddha",
    "Bala",
    "Kumara",
    "Bala",
    "Vriddha",
    "Yuva",
    "",
    "",
  ]);
  const lagna = screen.getByRole("rowheader", { name: "Lagna" }).closest("tr")!;
  expect([...lagna.querySelectorAll("td")].map((c) => c.textContent)).toEqual(["İki", ""]);
  // Karaka raporu istenmez: iki sütun birbirinden bağımsızdır.
  expect(api.computeJaimini).not.toHaveBeenCalled();
});

test("Baladi sits after the karaka column when both are on", async () => {
  renderTable({ columns: ["sign"], karaka: true, baladi: true });
  expect(headers()).toEqual(["Graha", "Burç", "Karaka", "Baladi"]);
  await waitFor(() => expect(baladi("Saturn")).toHaveTextContent("Yuva"));
  await waitFor(() => expect(document.querySelector('[data-karaka="Moon"]')).toHaveTextContent("AK Atmakaraka"));
  const saturn = screen.getByRole("button", { name: "Satürn" }).closest("tr")!;
  expect([...saturn.querySelectorAll("td")].map((c) => c.textContent)).toEqual(["Koç", "BK Bhratrikaraka", "Yuva"]);
});

test("unknown birth time: the Moon's stage is marked approximate, the other stages are not", async () => {
  vi.mocked(api.computeAvasthas).mockImplementation(async () => ({ status: "ok", data: fixture().avasthas_time_unknown }));
  renderTable({ columns: ["sign"], baladi: true }, unknownTimeOpened());
  await waitFor(() => expect(baladi("Moon")).toHaveTextContent("≈yaklaşık Vriddha"));
  expect(vi.mocked(api.computeAvasthas).mock.calls[0][0].time_known).toBe(false);
  expect(baladi("Sun")).toHaveTextContent(/^Yuva$/);
  expect(baladi("Sun").querySelector('[title="yaklaşık"]')).toBeNull();
  expect(screen.queryByRole("rowheader", { name: "Lagna" })).toBeNull();
});

test("the Baladi checkbox writes settings.baladi and leaves the other settings alone", async () => {
  const { persistence } = renderTable({ columns: ["sign", "house"], karaka: true });
  const summary = screen.getByText("Sütunlar", { selector: "summary" });
  fireEvent.click(summary);
  fireEvent.click(within(summary.closest("details")!).getByRole("checkbox", { name: "Baladi" }));
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!;
  expect(saved.panels[0].settings).toEqual({ columns: ["sign", "house"], karaka: true, baladi: true });
});

test("a failed avastha request keeps the table and offers a retry", async () => {
  vi.mocked(api.computeAvasthas).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderTable({ baladi: true });
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.getByRole("table")).toBeInTheDocument();
  expect(baladi("Moon")).toHaveTextContent(/^$/);
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(baladi("Moon")).toHaveTextContent("Vriddha"));
  expect(screen.queryByRole("alert")).toBeNull();
});
````

Uçtan uca akışlar: (1) panel paletten açılır, rapor bir kez istenir, Satürn'ün dört dizisi ve terim satırı fikstürdeki değerlerdir, ortak vurgu çalışır, aynı komut paneli çıkarır; sonra Araştırma düzeninde graha tablosunun Baladi sütunu açılır ve ayar düzenle birlikte kaydedilir. (2) Doğum saati bilinmeyen haritada Lajjita ve Shayanadi yoktur ve Ay'ın satırı yaklaşıktır.

**Yeni dosya:** `app/e2e/avastha.desk.spec.ts`

````ts
// Avastha akışları (Plan 24). Ayrı dosyadır: `desk.spec.ts` paralel planlarla paylaşılır; "e2e"
// projesi adı `desk.spec.ts` ile biten her dosyayı koşar.
import { expect, test } from "@playwright/test";
import { ipcCalls, mockIpc } from "./ipc";

type AvasthaCall = { input: { jd_ut: number; time_known: boolean; settings: { schema_version: number } } };

test("avasthas: a palette panel with the four series and the Shayanadi terms; the graha table's Baladi column", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const toggle = async () => {
    await page.keyboard.press("Meta+k");
    const palette = page.getByRole("dialog", { name: "Komut paleti" });
    await palette.getByRole("combobox").fill("avastha");
    await palette.getByRole("option", { name: /Avasthalar panelini aç\/kapat/ }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
  };
  const calls = async () => (await ipcCalls(page, "compute_avasthas")) as AvasthaCall[];
  // Rapor panel açılmadan istenmez (ileri modüller spec'i K1).
  expect(await calls()).toHaveLength(0);

  await toggle();
  const panel = page.getByRole("region", { name: "Avasthalar" });
  await expect(panel).toBeFocused();
  await expect(panel).toBeInViewport();
  await expect(panel.getByRole("table")).toHaveCount(2);
  expect(await calls()).toHaveLength(1);
  expect((await calls())[0].input).toMatchObject({ jd_ut: 2451545, time_known: true, settings: { schema_version: 2 } });

  // Delhi fikstürü: Satürn Koç'ta (düşüş burcu); düşmanı Ay karşıdan bakar, Jüpiter yanındadır.
  const states = panel.getByRole("table", { name: /^Avasthalar: yedi grahanın/ });
  await expect(states.locator("tbody tr")).toHaveCount(7);
  const saturn = states.locator('tbody tr[data-graha="Saturn"]');
  await expect(saturn.locator('td[data-series="baladi"]')).toHaveText("Yuva");
  await expect(saturn.locator('td[data-series="jagradadi"]')).toHaveText("Sushupti");
  await expect(saturn.getByRole("listitem")).toHaveText(["Kshudhita (Ay)", "Mudita (Jüpiter)"]);
  await expect(saturn.locator('td[data-series="shayanadi"]')).toHaveText("Gamana");
  // Hiçbir Lajjitadi durumunda olmayan graha çizgi ve sözcük taşır.
  await expect(states.locator('tbody tr[data-graha="Moon"] td[data-series="lajjitadi"]')).toHaveText("—hiçbiri");

  // Shayanadi terimleri Rust'tan gelir: Satürn 2 × 7 × 5 + 15 + 25 + 3 = 113, artan 5.
  const terms = panel.getByRole("table", { name: "Shayanadi terimleri" });
  await expect(terms.locator('tbody tr[data-terms-of="Saturn"]').locator("th, td")).toHaveText(["Satürn", "2", "7", "5", "113", "5"]);
  await expect(panel.getByText("Ortak terimler: doğum nakshatrası 15 (Swati), ghati 25, lagna 3 (İki).")).toBeVisible();
  await expect(panel.getByText(/^Doğrulanmamış tablo:/)).toBeVisible();
  await expect(panel.getByText("Yöntem: avastha.bphs_v1")).toBeVisible();
  // Tabloda denetim az olabilir: kaydırma kabı klavye odağı alır.
  const scroll = panel.getByRole("group", { name: "Shayanadi terimleri" });
  await scroll.focus();
  await expect(scroll).toBeFocused();

  // Ortak vurgu: satırdaki graha haritadaki grahayı ve iki tablodaki satırını vurgular.
  await saturn.getByRole("button", { name: "Satürn" }).hover();
  await expect(page.locator('svg [data-graha="Saturn"]').first()).toHaveClass(/chart-planet--highlight/);
  await expect(panel.locator("tr.is-highlighted")).toHaveCount(2);

  // Aynı komut paneli kaldırır; odak gövdeye düşmez.
  await toggle();
  await expect(panel).toHaveCount(0);
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);

  // Graha tablosunun Baladi sütunu aynı komutu okur ve ayarı düzenle birlikte kaydedilir.
  await page.keyboard.press("Meta+Digit2");
  const table = page.getByRole("region", { name: "Graha tablosu" });
  await expect(table.getByRole("columnheader", { name: "Baladi" })).toHaveCount(0);
  const before = (await calls()).length;
  await table.locator("summary", { hasText: "Sütunlar" }).click();
  await table.getByRole("checkbox", { name: "Baladi" }).check();
  await expect(table.getByRole("columnheader", { name: "Baladi" })).toBeVisible();
  await expect(table.locator('td[data-baladi="Saturn"]')).toHaveText("Yuva");
  await expect(table.locator('td[data-baladi="Rahu"]')).toHaveText("");
  expect((await calls()).length).toBe(before + 1);
  await expect
    .poll(async () => {
      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: { baladi?: boolean } }[] }[] } } } | undefined;
      const layout = saved?.viewState.panel_layout;
      return layout?.layouts
        .find((l) => l.id === layout.active_layout_id)
        ?.panels.filter((p) => p.kind === "graha_table")
        .map((p) => p.settings.baladi);
    })
    .toEqual([true]);
});

test("avasthas with an unknown birth time: Lajjita and Shayanadi are not given and the Moon's row is approximate", async ({ page }) => {
  await mockIpc(page, { session: true, unknownTime: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("avastha");
  await palette.getByRole("option", { name: /Avasthalar panelini aç\/kapat/ }).click();
  const panel = page.getByRole("region", { name: "Avasthalar" });
  await expect(panel.getByRole("table")).toHaveCount(1);
  const calls = (await ipcCalls(page, "compute_avasthas")) as AvasthaCall[];
  expect(calls.map((c) => c.input.time_known)).toEqual([false]);
  await expect(panel.getByText(/^Doğum saati bilinmiyor: Lajjita değerlendirilmedi ve Shayanadi hesaplanmadı/)).toBeVisible();
  await expect(panel.locator('td[data-series="shayanadi"]')).toHaveText(Array.from({ length: 7 }, () => "—hesaplanmadı"));
  await expect(panel.locator('tr[data-graha="Moon"] td[data-series="baladi"]')).toHaveText("≈yaklaşık Vriddha");
  await expect(panel.locator('tr[data-graha="Sun"] td[data-series="baladi"]')).toHaveText("Yuva");
  await expect(panel.getByRole("listitem").filter({ hasText: "Lajjita" })).toHaveCount(0);
  await expect(panel.getByText(/^Ortak terimler/)).toHaveCount(0);
});
````

Komutun taklidi (paylaşılan dosya; iki satır):

````diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index 0b96d84..17562bc 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -130,6 +130,8 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
         compute_shadbala: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.shadbala : null),
         // Yogalar: fikstürdeki iki gerçek sonuç (doğum saati bilinen ve bilinmeyen girdi).
         compute_yogas: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.yogas : fixture.yogas_time_unknown),
+        // Avasthalar: fikstürdeki iki gerçek sonuç (doğum saati bilinen ve bilinmeyen girdi).
+        compute_avasthas: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.avasthas : fixture.avasthas_time_unknown),
         // Uyum: eş haritanın girdisi ve kısa kimliği; başka kimlik `null` döner (harita yok).
         // Rapor yeniden hesaplanmaz, rollerden seçilir: açık harita damat rolündeyse `report`,
         // gelin rolündeyse `report_swapped`; gelin rolündeki eşin doğum saati bilinmiyorsa
````

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/panels/GrahaTableBaladi.test.tsx 2>&1 | grep -E "FAIL|Test Files|Tests "
lsof -i :1421
pnpm --dir app exec tsc --noEmit -p e2e
pnpm --dir app exec playwright test --project=e2e -g "avasthas" 2>&1 | tail -4
rm -rf app/test-results
```
Expected: Vitest `Tests  5 failed | 1 passed (6)` (sütun henüz yok; "varsayılan olarak kapalı" testi geçer); `lsof` çıktısız (çıktı varsa beklenir); `tsc` çıktısız; Playwright `1 failed` (ilk akış, Baladi onay kutusunda) ve `1 passed` (saati bilinmeyen harita: panel Task 6'da kuruludur).

- [ ] **Step 3: Baladi sütunu ve metni**

````diff
diff --git a/app/ui/panels/GrahaTablePanel.tsx b/app/ui/panels/GrahaTablePanel.tsx
index cf4ef49..8f5e89c 100644
--- a/app/ui/panels/GrahaTablePanel.tsx
+++ b/app/ui/panels/GrahaTablePanel.tsx
@@ -4,11 +4,13 @@ import { useTranslation } from "react-i18next";
 import type { GrahaPosition } from "../bindings";
 import { useHighlight } from "../highlight/store";
 import { sameTarget, type HighlightTarget } from "../highlight/target";
+import { useAvasthas } from "../modules/useAvasthas";
 import { useJaimini } from "../modules/useJaimini";
 import { useSession } from "../session/store";
 import { useLayoutStore } from "../view/store";
 import { timeUnknown } from "../session/birth";
 import { Approx } from "./Approx";
+import { BALADI_NAMES } from "./avasthaLabels";
 import { APPROX_COLUMNS, grahaCellText, lagnaCellText, NUMERIC_COLUMNS as NUMERIC } from "./grahaCells";
 import { GRAHA_COLUMNS, grahaColumns, toggleColumn, type GrahaColumn } from "./grahaColumns";
 import { KARAKA_ROLES } from "./labels";
@@ -40,7 +42,9 @@ function grahaCell(t: TFunction, g: GrahaPosition, column: GrahaColumn, approx:
  * `Chart.lagna`'dan. Saat bilinmiyorsa lagna satırı ve ev sütunu gizlenir, Ay'ın saate
  * bağlı değerleri "≈" ile işaretlenir. İsteğe bağlı "Karaka" sütunu (`settings.karaka`)
  * chara karaka rolünü `compute_jaimini`'den okur; açık değilken komut çağrılmaz. Sütun
- * `columns` listesinde değildir: baskı tablosu o listeyi kullanır ve değişmez (K10).
+ * `columns` listesinde değildir: baskı tablosu o listeyi kullanır ve değişmez (K10). İsteğe
+ * bağlı "Baladi" sütunu (`settings.baladi`) aynı kalıptır: değer `compute_avasthas`'tan okunur,
+ * açık değilken komut çağrılmaz; Rahu, Ketu ve lagna satırında boştur.
  */
 export function GrahaTablePanel({ panel }: PanelProps) {
   const { t } = useTranslation();
@@ -49,10 +53,13 @@ export function GrahaTablePanel({ panel }: PanelProps) {
   const highlight = useHighlight();
   const karakaOn = panel.settings.karaka === true;
   const jaimini = useJaimini(opened, karakaOn);
+  const baladiOn = panel.settings.baladi === true;
+  const avasthas = useAvasthas(opened, baladiOn);
 
   if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
   if (opened.chart.grahas.length === 0) return <p className="panel-message">{t("table.empty")}</p>;
   const roles = new Map((jaimini.data?.karakas ?? []).map((k) => [k.graha, KARAKA_ROLES[k.role]]));
+  const baladi = new Map((avasthas.data?.grahas ?? []).map((a) => [a.graha, BALADI_NAMES[a.baladi]]));
 
   const unknown = timeUnknown(opened);
   const chosen = grahaColumns(panel.settings);
@@ -80,9 +87,14 @@ export function GrahaTablePanel({ panel }: PanelProps) {
             <input type="checkbox" checked={karakaOn} onChange={(e) => setPanelSettings(panel.id, { karaka: e.target.checked })} />
             {t("table.karaka")}
           </label>
+          <label>
+            <input type="checkbox" checked={baladiOn} onChange={(e) => setPanelSettings(panel.id, { baladi: e.target.checked })} />
+            {t("table.baladi")}
+          </label>
         </fieldset>
       </details>
       {jaimini.error && <PanelError error={jaimini.error} onRetry={jaimini.retry} />}
+      {avasthas.error && <PanelError error={avasthas.error} onRetry={avasthas.retry} />}
       <table className="graha-table">
         <thead>
           <tr>
@@ -93,6 +105,7 @@ export function GrahaTablePanel({ panel }: PanelProps) {
               </th>
             ))}
             {karakaOn && <th scope="col">{t("table.karaka")}</th>}
+            {baladiOn && <th scope="col">{t("table.baladi")}</th>}
           </tr>
         </thead>
         <tbody>
@@ -105,6 +118,7 @@ export function GrahaTablePanel({ panel }: PanelProps) {
                 </td>
               ))}
               {karakaOn && <td />}
+              {baladiOn && <td />}
             </tr>
           )}
           {opened.chart.grahas.map((g) => {
@@ -148,6 +162,16 @@ export function GrahaTablePanel({ panel }: PanelProps) {
                     )}
                   </td>
                 )}
+                {baladiOn && (
+                  <td data-baladi={g.graha}>
+                    {/* Ay'ın basamağı yaklaşık dereceden çıkar: "≈" taşır (K2). Düğümlerde boştur. */}
+                    {baladi.has(g.graha) && (
+                      <Approx on={approx} label={t("table.approx")}>
+                        {baladi.get(g.graha)}
+                      </Approx>
+                    )}
+                  </td>
+                )}
               </tr>
             );
           })}
````

`table.baladi` (paylaşılan dosyalar; `table` bloğunun sonuna birer satır):

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index f5bb3f5..b3a0f3a 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -829,6 +829,7 @@ export const en: Messages = {
     lagnaRow: "Lagna",
     approx: "approximate",
     karaka: "Karaka",
+    baladi: "Baladi",
   },
   errors: {
     dasha_empty: "The sub-periods of this period could not be loaded.",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 3f91bde..fe4a4c8 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -829,6 +829,7 @@ export const fr: Messages = {
     lagnaRow: "Lagna",
     approx: "approximatif",
     karaka: "Karaka",
+    baladi: "Baladi",
   },
   errors: {
     dasha_empty: "Les sous-périodes de cette période n’ont pas pu être chargées.",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 9acf8b2..de270fd 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -828,6 +828,7 @@ export const tr = {
     lagnaRow: "Lagna",
     approx: "yaklaşık",
     karaka: "Karaka",
+    baladi: "Baladi",
   },
   errors: {
     dasha_empty: "Bu dönemin alt dönemleri alınamadı.",
````

- [ ] **Step 4: Testler, akışlar ve kapılar**

```bash
pnpm --dir app exec vitest run ui/panels/GrahaTableBaladi.test.tsx ui/panels/GrahaTablePanel.test.tsx 2>&1 | grep -E "FAIL|Test Files|Tests "
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
lsof -i :1421
pnpm --dir app exec playwright test --project=e2e -g "avasthas" 2>&1 | tail -1
pnpm --dir app e2e 2>&1 | tail -1
rm -rf app/test-results
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: `2 passed (2)`, `16 passed (16)` (var olan on graha tablosu testi değişmeden geçer); Vitest `90 passed (90)`, `907 passed (907)`; `✓ built`; `lsof` çıktısız; `2 passed`; `29 passed`; sayaç `0`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/panels/GrahaTablePanel.tsx app/ui/panels/GrahaTableBaladi.test.tsx app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts app/e2e/ipc.ts app/e2e/avastha.desk.spec.ts
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): optional Baladi column in the graha table; avastha end-to-end flows" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: yedi dosya; tarama çıktısız; `git status --short` boş.

---

### Task 8: Kapılar, REVIEW.md ve elle denetim kapısı

Yol haritası §5 Plan 24 görev (8) ve "Sahibin denetimi"; Y3, Y4. Bu görev iki belge dosyası üretir (REVIEW kaydı ve el örnekleri kılavuzunun satırı), bütün kapıları koşar ve el örneklerini bağımsız olarak yeniden türetir. Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle çözülür ve bu görev baştan koşulur. Aşağıda `BASE`, Task 1 Step 1'de `git log -1 --format=%h` ile okunan commit'tir (bu plan yazılırken `7270550`).

**Files:**
- Create: `tests/golden/modules/avastha/REVIEW.md`
- Modify: `tests/golden/modules/README.md` (paylaşılan dosya; bir satır ve bir paragraf)

**Interfaces:**
- Consumes: Task 1–7'nin tamamı.
- Produces: okuma kurallarının, tabloların ve sahibin onayını bekleyen on dokuz varyant noktasının kaydı; yeşil kapılar; bağımsız yeniden türetmenin sonucu; kural ve gizlilik taramaları; sahibine devredilen **[insan]** maddeleri (ilerleme notlarına yazılır).

- [ ] **Step 1: REVIEW.md ve kılavuz satırı**

**Yeni dosya:** `tests/golden/modules/avastha/REVIEW.md`

````markdown
# Avastha el örnekleri: okuma kuralları, tablolar, onay bekleyen noktalar ve gözden geçirme kaydı

Kaynak: eksikler yol haritası §4.4 ve Plan 24'ün kararları; klasik dayanak BPHS'nin avasthalar
bölümüdür (genel gönderme; alıntı yapılmaz). Bu dizinde iki dizi durur:

- `lajjitadi/`: sentetik konumlar (dokuz grahanın ve varsa lagnanın sidereal boylamı). Her dosya
  yedi grahanın Baladi, Jagradadi ve Lajjitadi durumlarını ve o örneğin "oluşmayan durum" olarak
  gösterdiği çiftleri (`absent`) yazar.
- `shayanadi/`: aynı konumlar ve `minutes_since_sunrise` (gün doğumundan doğuma geçen dakika;
  `null` o gün gün doğumu olmayan yerdir). Her dosya formülün ortak terimlerini (`basis`) ve yedi
  grahanın nakshatrasını, navamsa dilimini, toplamını ve durumunu yazar; Shayanadi hesaplanmayan
  haritada `expected` `null`'dır.

Ephemeris yoktur; rasi çerçevesi sidereal sayılır. Değerler kuraldan türetilmiştir ve
`derivation` alanında adım adım yazılıdır; motor çıktısı buraya kopyalanmaz (K9). Bir değer
motorla uyuşmazsa önce kural yeniden okunur. Testler: `crates/jyotish-core/tests/avastha_golden.rs`.

## Okuma kuralları

Ortak tanımlar:

- **Kapsam.** Yedi graha (Güneş … Satürn). Rahu ve Ketu'ya avastha atanmaz; Lajjita'da eşlikçi
  olarak sayılırlar.
- **Burç ve ev** rasi çerçevesindedir; ev, lagna burcundan sayılan burç evidir ve ev sistemi
  ayarından bağımsızdır. **Nakshatra** her zodyak kipinde sidereal boylamdandır.
- **Birliktelik** aynı burçta bulunmaktır. **Bakış** burçtan burca tam graha bakışıdır: her graha
  bulunduğu burçtan 7. burca; ayrıca Mars 4. ve 8., Jüpiter 5. ve 9., Satürn 3. ve 10. burca bakar.
  Rahu ve Ketu bakmaz. Aynı burçtaki graha bakmış sayılmaz.
- **Dost** ve **düşman** doğal (naisargika) ilişkidir ve durumu sorulan grahanın gözünden okunur
  (aşağıdaki tablo). Düğümler kimsenin dostu ya da düşmanı değildir. Bir burcun lordu grahanın
  kendisiyse o burç ne dost ne düşman burcudur.
- **İyicil** ve **kötücül** uygulamanın ortak doğal sınıfıdır: Jüpiter ve Venüs iyicil; Güneş, Mars
  ve Satürn kötücül; Ay büyürken (uzanım 0°–180°) iyicil, küçülürken kötücül; Merkür aynı burçta
  kötücül varken iyicil yoksa kötücül, değilse iyicil.

1. **Baladi.** Burç altışar derecelik beş dilime bölünür. Tek burçlarda (Koç, İkizler, Aslan,
   Terazi, Yay, Kova) dilimler sırayla Bala, Kumara, Yuva, Vriddha, Mrita'dır; çift burçlarda sıra
   terstir (ilk dilim Mrita, son dilim Bala). Dilim sınırı sonraki dilime aittir (6°00′ ikinci
   dilimdir).
2. **Jagradadi.** Graha yücelme burcunda ya da kendi burcundaysa Jagrat; düşüş burcundaysa
   Sushupti; öteki burçlarda burç lordu dostu ya da nötrüyse Swapna, düşmanıysa Sushupti. Sıra
   budur: yücelme ve kendi burcu önce, düşüş sonra, ilişki en son okunur.
3. **Lajjita.** Graha lagnadan 5. evde **ve** Güneş, Mars, Satürn, Rahu ya da Ketu'dan en az biriyle
   birlikte. Lagna yoksa (doğum saati bilinmiyor) değerlendirilmez.
4. **Garvita.** Graha yücelme burcunda (burcun tamamı) **ya da** moolatrikona aralığında.
5. **Kshudhita.** Graha düşmanının burcunda, **ya da** bir düşmanıyla birlikte, **ya da** bir
   düşmanının bakışı altında, **ya da** Satürn ile birlikte (Satürn o grahanın düşmanı olmasa da).
6. **Trushita.** Graha su burcunda (Yengeç, Akrep, Balık) **ve** bir düşmanının bakışı altında
   **ve** ona bakan hiçbir iyicil yok.
7. **Mudita.** Graha dostunun burcunda, **ya da** bir dostuyla birlikte, **ya da** bir dostunun
   bakışı altında, **ya da** Jüpiter ile birlikte.
8. **Kshobhita.** Graha Güneş ile birlikte **ve** bir kötücülün ya da bir düşmanının bakışı
   altında. Güneş'in kendisi Kshobhita olmaz.
9. **Küme.** Lajjitadi'de bir graha aynı anda birden çok durumda olabilir ya da hiçbirinde
   olmayabilir. Liste yukarıdaki kural sırasındadır. Her kayıt, kuralı sağlayan öteki grahaları
   (birlikte olan ya da bakan) graha sırasıyla taşır; kural yalnızca burçtan sağlanıyorsa liste
   boştur.
10. **Shayanadi.** `toplam = N × G × A + J + Gh + L`; durum, toplamın 12'ye bölümünden artanın
    gösterdiği durumdur ve artan 0 ise on ikinci durumdur.
    - `N`: grahanın nakshatrası, Ashwini'den 1–27.
    - `G`: grahanın sayısı (aşağıdaki tablo).
    - `A`: grahanın bulunduğu navamsa diliminin burç içindeki sırası, 1–9 (dilim 3°20′; sınır
      sonraki dilime aittir).
    - `J`: doğum nakshatrası (Ay'ın nakshatrası), 1–27.
    - `Gh`: doğumdan önceki son gün doğumundan doğuma **tamamlanmış** ghati sayısı; bir ghati 24
      dakikadır (gün doğumundan geçen gün kesri × 60, aşağı yuvarlanır).
    - `L`: lagna burcunun sırası, Koç 1 … Balık 12.
    Lagna yoksa (doğum saati bilinmiyor) ya da o gün gün doğumu yoksa Shayanadi hesaplanmaz.

## Tablolar

Motorun kullandığı tablolar (`crates/jyotish-core/src/avastha/` ve v1'in `dignity.rs` dosyası).

**Doğal ilişki** (satırdaki grahanın sütundakine ilişkisi)

| Graha | Dost | Nötr | Düşman |
|---|---|---|---|
| Güneş | Ay, Mars, Jüpiter | Merkür | Venüs, Satürn |
| Ay | Güneş, Merkür | Mars, Jüpiter, Venüs, Satürn | — |
| Mars | Güneş, Ay, Jüpiter | Venüs, Satürn | Merkür |
| Merkür | Güneş, Venüs | Mars, Jüpiter, Satürn | Ay |
| Jüpiter | Güneş, Ay, Mars | Satürn | Merkür, Venüs |
| Venüs | Merkür, Satürn | Mars, Jüpiter | Güneş, Ay |
| Satürn | Merkür, Venüs | Jüpiter | Güneş, Ay, Mars |

**Burçlar** (Jagradadi ve Garvita)

| Graha | Yücelme burcu | Düşüş burcu | Kendi burçları | Moolatrikona aralığı |
|---|---|---|---|---|
| Güneş | Koç | Terazi | Aslan | Aslan 0°–20° |
| Ay | Boğa | Akrep | Yengeç | Boğa 3°–30° |
| Mars | Oğlak | Yengeç | Koç, Akrep | Koç 0°–12° |
| Merkür | Başak | Balık | İkizler, Başak | Başak 15°–20° |
| Jüpiter | Yengeç | Oğlak | Yay, Balık | Yay 0°–10° |
| Venüs | Balık | Başak | Boğa, Terazi | Terazi 0°–15° |
| Satürn | Terazi | Koç | Oğlak, Kova | Kova 0°–20° |

Aralığın başı içeride, sonu dışarıdadır.

**Shayanadi: graha sayıları ve durum sırası**

| Graha | Güneş | Ay | Mars | Merkür | Jüpiter | Venüs | Satürn |
|---|---|---|---|---|---|---|---|
| `G` | 1 | 2 | 3 | 4 | 5 | 6 | 7 |

| Artan | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 0 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Durum | Shayana | Upaveshana | Netrapani | Prakashana | Gamana | Agamana | Sabha | Agama | Bhojana | Nritya Lipsa | Kautuka | Nidra |

## Sahibin onayını bekleyen kurallar ve bilinen varyant noktaları

Yukarıdaki kurallar ve tablolar bu depoya bellekten aktarılmıştır; basılı bir kaynakla
karşılaştırılmamıştır (yol haritası Y4). Aşağıdaki noktalarda çeviriler ve gelenekler ayrışır.
Karşılaştırma bir insanın işidir: elinde basılı bir BPHS çevirisi ya da avasthaları anlatan bir
derleme bulunan kişi her satırı denetler, kaynağı genel adıyla (alıntı yapmadan) ve sonucu
aşağıdaki tabloya yazar. Fark bulunursa kural ayrı bir commit'le düzeltilir, `AVASTHA_METHOD`
kimliği değişir ve bu dizindeki örnekler kuraldan yeniden türetilir. Onaya dek Avastha paneli
"doğrulanmamış tablo" notunu taşır (`avastha.unverified`).

| No | Nokta | Bu depodaki okuma | Bilinen diğer okuma |
|---|---|---|---|
| 1 | Jagradadi'nin eklenmesi | Eklendi (yol haritası sahibin kararına bırakmıştı; maliyeti Baladi kadardır) | Hiç gösterilmemesi |
| 2 | Jagradadi'de dostluk | Doğal ilişki; moolatrikona ayrı basamak değildir (kendi burcu sayılır) | Bileşik (panchadha) ilişki; moolatrikonanın Jagrat'a ayrıca yazılması |
| 3 | Baladi'nin kapsamı | Yedi graha; düğümlere atanmaz | Dokuz grahaya ve lagnaya da basamak veren tablolar |
| 4 | Lajjita'nın evi ve eşlikçileri | Lagnadan 5. burç evi; Güneş, Mars, Satürn, Rahu, Ketu'dan biri | Yalnızca Rahu ya da Ketu ile; evin Ay'dan da sayılması; bhava (ev sistemi) evi |
| 5 | Garvita | Yücelme burcunun tamamı ya da moolatrikona aralığı | Moolatrikona burcunun tamamı; yalnızca derin yücelme derecesine dek |
| 6 | Düşmanlığın türü (Kshudhita, Trushita, Kshobhita) | Doğal ilişki | Bileşik (doğal + geçici) ilişki |
| 7 | Kshudhita'nın kolları | Dört kol: düşman burcu, düşmanla birliktelik, düşman bakışı, Satürn ile birliktelik | Satürn kolunu saymayan üç kollu okuma |
| 8 | Bakışın türü | Burçtan burca tam graha bakışı | Dereceden bakış (sphuta drishti, bir eşikle); rashi drishti |
| 9 | Trushita'nın su burcu | Yengeç, Akrep, Balık | Lagnadan 4. ev ("su evi") okuması; düşman yerine kötücül bakışı arayan okuma |
| 10 | Mudita | Dört kolun herhangi biri ("ya da") | Kolların birlikte aranması ("ve"); Jüpiter kolu yerine iyicil bakışı |
| 11 | Kshobhita | Güneş ile aynı burçta ve kötücül ya da düşman bakışı altında | Yanıklık orbuyla birliktelik; kötücülle birlikteliğin de sayılması |
| 12 | Birden çok durum | Küme: bütün tutan kurallar, kural sırasında | Öncelik sırasıyla tek durum |
| 13 | Shayanadi: graha sayıları | Güneş 1 … Satürn 7; düğümlere durum atanmaz | Rahu 8 ve Ketu 9 ile dokuz graha |
| 14 | Shayanadi: navamsa terimi | Burç içindeki dilim sırası, 1–9 | Navamsa burcunun Koç'tan sırası, 1–12 |
| 15 | Shayanadi: ghati | Tamamlanmış ghati; bir ghati 24 dakika | Süren ghati (tamamlanan + 1); gün doğumundan sonraki gün doğumuna altmış eşit dilim |
| 16 | Shayanadi: artan 0 | On ikinci durum (Nidra) | — (okuma ortaktır; kayıt için yazıldı) |
| 17 | Shayanadi: on iki adın sırası | Yukarıdaki tablo (bellekten aktarıldı) | Belirli bir varyant bilinmiyor; sıra basılı kaynakla hücre hücre denetlenmelidir |
| 18 | Shayanadi: lagna terimi ve çerçeve | Lagnanın rasi burcunun sırası; tropikal-rasi kipinde navamsa dilimi ve lagna rasi çerçevesinden, nakshatra sidereal | Her terimin sidereal çerçeveden okunması |
| 19 | Kapsam dışı bırakılanlar | Shayanadi'nin üç alt durumu (cheshta) ve Deeptadi yoktur | — |

| Denetleyen | Tarih | Kaynak (genel ad) | Sonuç |
|---|---|---|---|
| — | — | — | Bekliyor: sahibine devredildi (Plan 24 Task 8) |

## Gözden geçirme kaydı

Gözden geçiren (örnekleri yazan dışında biri) her örnekte yalnızca `input`'tan başlar, motoru
çalıştırmadan ve Rust kaynağını okumadan, yukarıdaki okuma kurallarından ve tablolardan yedi
grahanın Baladi, Jagradadi ve Lajjitadi durumlarını (`lajjitadi/`) ya da ghatiyi, ortak terimleri,
toplamı ve durumu (`shayanadi/`) yeniden hesaplar; sonucunu `expected` ile karşılaştırır ve aşağıya
adını, tarihini ve bulduğu farkları yazar.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
| Plan 24'ü yazan oturum (örneklerin yazarı) | 2026-11-27 | Örnekler, motorun kaynağından ayrı yazılmış bir türetmeyle kural metninden üretildi ve motorla karşılaştırıldı: `lajjitadi/` yedi örnek, 49 graha satırı; `shayanadi/` dört hesaplı örnek, 28 graha satırı; fark yok. Sınır: türetmeyi ve motoru aynı yazar yazdı; kuralın okunuşundaki ortak bir yanlış ikisinde de aynıdır. Bu kayıt aritmetiği ve kuralların uygulanışını doğrular, okuma seçimlerini (yukarıdaki onay tablosu) değil. |
| — | — | Bekliyor: Plan 24 Task 8 Step 4 (yürüten ajanın bağımsız yeniden türetmesi) |
````

Kılavuz (paylaşılan dosya; tablonun sonuna bir satır, son cümleden önce bir paragraf):

````diff
diff --git a/tests/golden/modules/README.md b/tests/golden/modules/README.md
index 268e1ad..0b6aec0 100644
--- a/tests/golden/modules/README.md
+++ b/tests/golden/modules/README.md
@@ -56,6 +56,7 @@ doğum anı ve 365,25 günlük yıl kullanır, tarihleri el yıllarının biriki
 | `kuta` | `scores` (sekiz kuta, tablo sırasıyla: `kuta`, `score`, `groom_class`, `bride_class`), `total`, `bhakoot_exception`, `nadi_exception`, `kuja_dosha` (`[damat rolü, gelin rolü]`), `overlay` (`groom` ve `bride`: dokuzar satır `[diğer tarafın lagnasından ev ya da null, Ay'ından ev]`), `approximate` | 8 |
 | `muhurta` | `eighth_min`, `rahu`, `yamaganda`, `gulika` (`segment`, `from_min`, `to_min`), `abhijit` (`from_min`, `to_min`), `abhijit_weekday_exception`, `hora_min`, `hora_lords` (24), `tithi_group`, `nakshatra_nature`, `tara` (`number`, `favourable`), `chandra` (`house`, `favourable`) | 7 |
 | `prashna` | `lagna` (`sign`, `deg_in_sign`, `kind`, `rising`, `lord`, dört `kendras`; lagna yoksa `null`), `focus` (`house`, `sign`, `lord`, `same_graha`, `links`, iki ev sayısı, `occupants`; odak evi ya da lagna yoksa `null`), `moon` (`sign`, `house`, `nakshatra`, `pada`, `tithi`, `paksha`, `aspected_by`) | 14 |
+| `avastha` (alt klasörler: `lajjitadi`, `shayanadi`) | `lajjitadi/`: `baladi`, `jagradadi`, `lajjitadi` (yedi graha; durum ve kuralı sağlayan grahalar), `absent`; `shayanadi/`: `basis` (`birth_nakshatra`, `ghatis`, `lagna_sign_number`), `grahas` (yedi graha: `state`, `nakshatra`, `navamsa`, `total`); Shayanadi hesaplanmıyorsa `expected` `null` | 7 + 6 |
 
 Shadbala örnekleri çalışma kâğıdıdır (tasarım belgesi §7 "Test"): `shadbala/WORKSHEET.md` her
 kalemin aritmetiğini adım adım yazar, JSON dosyaları aynı girdiyi ve sonuçları taşır. Tolerans
@@ -101,4 +102,14 @@ beş bağını, "hiçbiri"ni ve aynı grahayı, odak evi seçilmemiş ve lagnas
 lordu ve iki pakshayı kapsadığını sabitler. Okuma kuralları, motorun tabloları, sahibin onayını
 bekleyen varyant noktaları ve bağımsız gözden geçirmenin kaydı `prashna/REVIEW.md`'dedir.
 
+Avastha örnekleri (yol haritası §4.4, Plan 24) ayrı bir test dosyasıyla yüklenir
+(`crates/jyotish-core/tests/avastha_golden.rs`) ve iki alt klasörde durur. `lajjitadi/` örnekleri
+ortak girdiyi (`lagna_lon`, `grahas`) kullanır; `shayanadi/` örnekleri girdiye
+`minutes_since_sunrise` ekler (gün doğumundan doğuma geçen dakika; `null` o gün gün doğumu olmayan
+yerdir). Ayrı testler altı Lajjitadi durumunun her birinin en az bir örnekte oluştuğunu ve en az
+birinde "oluşmayan" olarak gösterildiğini, beş Baladi basamağının tek ve çift burçta, on iki
+Shayanadi durumunun ve artanın sıfır olduğu durumun örneklerde bulunduğunu sabitler. Okuma
+kuralları, motorun tabloları, sahibin onayını bekleyen varyant noktaları ve gözden geçirme kaydı
+`avastha/REVIEW.md`'dedir.
+
 Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
````

- [ ] **Step 2: Rust kapıları**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
BASE=7270550
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
cargo test -p jyotish-desk --test avastha -- --nocapture 2>&1 | grep -E "compute_avasthas median"
cargo deny check 2>&1 | tail -1
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p jyotish-core -p atlas -p store -p jyotish-desk --all-targets 2>&1 | tail -1
git diff $BASE..HEAD --stat -- deny.toml Cargo.toml Cargo.lock app/package.json app/pnpm-lock.yaml app/vite.config.ts app/playwright.config.ts app/src-tauri/capabilities app/src-tauri/tauri.conf.json app/src-tauri/Cargo.toml crates/jyotish-core/Cargo.toml crates/store crates/atlas crates/ephemeris crates/jyotish-core/src/chart.rs crates/jyotish-core/src/positions.rs crates/jyotish-core/src/settings.rs crates/jyotish-core/src/panchang.rs crates/jyotish-core/src/dignity.rs crates/jyotish-core/src/drishti.rs crates/jyotish-core/src/nature.rs crates/jyotish-core/src/error.rs crates/jyotish-core/tests/modules_golden.rs app/src-tauri/src/commands.rs app/src-tauri/src/module_commands.rs app/src-tauri/tests/modules.rs
git diff $BASE..HEAD --stat -- crates/jyotish-core/src/lib.rs app/src-tauri/src/lib.rs | tail -1
ls crates/jyotish-core/proptest-regressions 2>/dev/null
```
Expected: fmt ve clippy çıktısız; testler: `jyotish-core` lib `429 passed`, `avastha_golden` `5 passed`, golden `8 passed`, `modules_golden` `17 passed`; `atlas` `build` 6, `moment_fixed_lmt` 11, `moment_iana` 13, `search` 8, `shipped` 3; `store` lib `122 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `58 passed`, `avastha` `1 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `modules` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; sayaç `0`; ölçüm satırı bütçenin içinde (D13; aşan değer testi zaten düşürür); deny `advisories ok, bans ok, licenses ok, sources ok`; MSRV `Finished` (1.93 araç zinciri kurulu değilse bu satır atlanır ve notlara yazılır); ilk `git diff --stat` çıktısız (bağımlılıklar, yetenek dosyası, veritabanı, var olan hesap modülleri, ortak komut ve test dosyaları değişmedi); iki `lib.rs` birlikte `2 files changed, 3 insertions(+)`; `ls` çıktısız. `BASE` Task 1 Step 1'de başka bir commit okunduysa o yazılır ve taban sayaçları o ölçümdendir.

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
Expected: `lsof` çıktısız (çıktı varsa beklenir); `Test Files  90 passed (90)`, `Tests  907 passed (907)`; `✓ built`; e2e `29 passed`; görsel `10 passed` (temel PNG'ler değişmedi: D23); betik testleri `pass 14`, `fail 0`; `git status --short` yalnızca Step 1'in iki dosyasını gösterir.

- [ ] **Step 4: Bağımsız yeniden türetme**

Yol haritası §8: "el örneğini yazanın kuralı yanlış anlaması motoru da yanlış doğrular; ayrı gözden geçirmede yeniden türetme zorunludur". Yürüten ajan örnekleri yazan oturum değildir ve bu denetimi yapabilir. Kural: **yalnızca** `tests/golden/modules/avastha/REVIEW.md`'nin "Okuma kuralları" ve "Tablolar" bölümleri ile her JSON'un `input` alanı okunur; `expected` ve `derivation` karşılaştırmadan önce okunmaz, Rust kaynağına bakılmaz, motor çalıştırılmaz.

1. Depo **dışında** (ör. `/tmp` altında; `git add` edilmez) atılacak bir betik yazılır: tabloları REVIEW'den kendi yazar, on üç dosyanın `input`'unu okur ve `lajjitadi/` için yedi grahanın Baladi, Jagradadi ve Lajjitadi durumlarını (kuralı sağlayan grahalarla), `shayanadi/` için tamamlanmış ghatiyi, üç ortak terimi ve yedi grahanın nakshatrasını, navamsa dilimini, toplamını ve durumunu hesaplar.
2. Sonuç `expected` ile karşılaştırılır: `lajjitadi/` 7 dosya × 7 graha = 49 satır; `shayanadi/` 4 hesaplı dosya × 7 graha = 28 satır ve iki `null` dosya.
3. Hepsi uyuşuyorsa REVIEW.md'nin "Gözden geçirme kaydı" tablosundaki son satır (`| — | — | Bekliyor: Plan 24 Task 8 Step 4 …`) şu satırla değiştirilir (tarih koşunun günüdür):

   ````markdown
   | Plan 24 Task 8'i yürüten ajan (örnekleri yazan oturum dışında) | <tarih> | 13/13 uyuştu: 49 + 28 graha satırı ve iki hesapsız örnek, fark yok. Hesap yalnızca `input`'tan ve bu dosyanın kural ve tablolarından başlar, motoru çalıştırmaz. Sınır: yürüten ajan aynı planın kodunu da yazdı; bu kayıt kuralların uygulanışını ve aritmetiği doğrular, okuma seçimlerini (onay tablosu) değil. |
   ````

4. Bir fark varsa **hiçbir beklenen değer ve hiçbir kural değiştirilmez**: fark (dosya, graha, iki değer ve kuralın hangi cümlesinden çıktığı) ilerleme notlarına yazılır, tablodaki satır `Fark bulundu: <kısa özet>; sahibine devredildi` diye doldurulur ve görev kapıların sonucuyla birlikte sahibine bildirilir. REVIEW'deki kural metni belirsiz bulunduysa bu da bir bulgudur.

```bash
grep -c "Bekliyor: Plan 24 Task 8 Step 4" tests/golden/modules/avastha/REVIEW.md
grep -inE 'k(a|ā|aa)la|\bcala\b' tests/golden/modules/avastha/REVIEW.md tests/golden/modules/README.md
git status --short
```
Expected: sayaç `0` (satır dolduruldu); tarama çıktısız; `git status --short` yalnızca iki belge dosyasını gösterir (betik depoda değildir).

- [ ] **Step 5: Commit**

```bash
git add tests/golden/modules/avastha/REVIEW.md tests/golden/modules/README.md
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "docs(golden): avastha reading rules, variant points awaiting approval and the review record" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: iki dosya; tarama çıktısız; `git status --short` boş.

- [ ] **Step 6: Kural ve gizlilik taramaları (tabandan bu yana)**

```bash
BASE=7270550
git diff $BASE..HEAD -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log $BASE..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff $BASE..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
git diff $BASE..HEAD --name-only | grep -vE '^(app|crates|tests)/'
echo "trailers: $(git log $BASE..HEAD --format=%B | grep -c '^Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>$') commits: $(git rev-list --count $BASE..HEAD) other: $(git log $BASE..HEAD --format=%B | grep '^Co-Authored-By: ' | grep -vc 'Claude Opus 5.5 <noreply@anthropic.com>$')"
git diff $BASE..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
git diff $BASE..HEAD -U0 -- crates app/src-tauri/src | grep '^+' | grep -E 'println!|eprintln!|dbg!|log::|tracing::'
git diff $BASE..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\(|window\.(print|open)\('
git diff $BASE..HEAD -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
grep -rnE '#\[serde\(default' crates/jyotish-core/src/avastha app/src-tauri/src/avastha_commands.rs
grep -rnE 'XMLHttpRequest|WebSocket|\bfetch\(' app/ui/panels/AvasthaPanel.tsx app/ui/panels/avasthaLabels.ts app/ui/modules/useAvasthas.ts
grep -rnE 'localStorage|sessionStorage|new Date\(|Date\.now' app/ui/panels/AvasthaPanel.tsx app/ui/panels/avasthaLabels.ts app/ui/modules/useAvasthas.ts
grep -cE 'avastha_commands::compute_avasthas,' app/src-tauri/src/lib.rs
grep -cE 'CommandError::new\(' app/src-tauri/src/avastha_commands.rs
grep -rn 'avastha' crates/store/src | head -1
git diff $BASE..HEAD --stat -- app/ui/chart app/ui/print app/ui/shortcuts.ts app/e2e/visual.spec.ts-snapshots app/e2e/desk.spec.ts app/ui/panels/panels.test.tsx app/ui/panels/GrahaTablePanel.test.tsx app/ui/panels/panels.css app/ui/panels/labels.ts app/ui/i18n/i18n.test.ts
git ls-files | grep -c '^\.superpowers/'
git tag --points-at HEAD
```
Expected (`BASE` Task 1 Step 1'de okunan commit'tir; `7270550` değilse o yazılır, yoksa aradaki başka commit'lerin dosyaları da taranır): `trailers:` satırında ilk iki sayı eşit (sekiz ve sekiz; inceleme düzeltmesi commit'i varsa ikisi birlikte büyür) ve `other: 0` (başka bir imza satırı yok); `grep -ciE` satırı `0`; komut sayacı `1`; `CommandError::new` sayacı `0`; `.superpowers` sayacı `0`. Bunların dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur): yasaklı ad yok (üretilen fikstürde de), `app/`, `crates/` ve `tests/` dışında dosya yok, arayüzde `console` ve Rust kaynağında log çağrısı yok (girdi ve rapor loglanmaz: C1; `tests/avastha.rs`'teki ölçüm satırı test dosyasıdır ve bu taramanın dışındadır), yasak çağrı yok, yeni stilde hex ya da `rgb(` yok (C9), IPC tiplerinde `serde(default)` yok, panelde ağ, tarayıcı deposu ve saat çağrısı yok (C4), veritabanı katmanında avastha yok (Y6), harita çizimi, baskı, kısayollar, görsel temeller ve bilerek dokunulmayan ortak dosyalar değişmedi, etiket (git tag) yok.

- [ ] **Step 7: Elle denetim kapısı**

Yürüten ajan gerçek uygulamayı **çalıştırmaz** (Global Constraints). Her madde ilerleme notlarına "geçti", "devredildi" ya da bulgu olarak yazılır; bulgu varsa düzeltme commit'i ve bu görev baştan. Her maddenin başında, otomatik bir karşılığı olup olmadığı yazılıdır. **[otomatik]** maddelerin karşılığı testlerde vardır ve yürüten ajan bunları testin adını göstererek "geçti" sayar; **[insan]** maddeler yalnızca basılı kaynağa ya da gerçek uygulamaya bakan bir insan tarafından doğrulanabilir ve sahibine devredilir (bu devir sonraki planı engellemez: Y3; hiçbir şey yayımlanmaz).

- [ ] **[otomatik]** Baladi: beş basamak, tek burçta düz ve çift burçta ters sıra, dilim sınırı sonraki dilime aittir; ikinci yazım (ayna) on iki burçta tutar. (`baladi.rs › baladi_runs_forward_…`, `› baladi_is_the_mirror_image_…`; `avastha_golden › lajjitadi_fixtures_cover_…`: her basamak tek ve çift burçta)
- [ ] **[otomatik]** Jagradadi: yücelme ve kendi burcu, düşüş burcu, sonra burç lorduna doğal ilişki; düğümlere durum atanmaz. (`baladi.rs › jagradadi_follows_…`, `› every_graha_takes_…`)
- [ ] **[otomatik]** Lajjitadi'nin altı kuralının her biri için tutan ve tutmayan durum; Kshudhita'nın dört kolu; Trushita'nın iyicil bakışla bozulması; Güneş'in Kshobhita olmaması; lagna yokken Lajjita'nın değerlendirilmemesi. (`lajjitadi.rs` dokuz test; `avastha_golden › lajjitadi_hand_fixtures`: yedi örnekte 49 graha satırı ve `absent` çiftleri)
- [ ] **[otomatik]** Küme sonucu: bir graha beş durumda birden olabilir, liste kural sırasındadır, bir durum iki kez yazılmaz, kayıt grahanın kendisini saymaz. (`lajjitadi.rs › one_graha_can_be_in_five_states_…`, `› nodes_have_no_state_…`; `tests/avastha.rs`: golden anlarda aynı değişmezler)
- [ ] **[otomatik]** Shayanadi: graha sayıları, navamsa dilimi, tamamlanmış ghati, formülün toplamı, artan sırası ve artan 0'ın on ikinci durum olması; bölmesiz ikinci yazım her girdide formülle aynıdır. (`shayanadi.rs` yedi test; `avastha_golden › shayanadi_hand_fixtures`, `› shayanadi_fixtures_cover_…`: on iki durum, ghati 0, 1, 28, 59)
- [ ] **[otomatik]** Doğum saati bilinmiyorsa Lajjita ve Shayanadi yoktur, öteki diziler aynıdır; gün doğumu yoksa yalnızca Shayanadi yoktur; sayı prashnasında lagna terimi sayıdan gelir; tropikal-rasi kipinde burç tropikal, nakshatra sidereal okunur. (`collect.rs` beş test; `tests/avastha.rs`: beş golden an, Tromsø dahil)
- [ ] **[otomatik]** Gerçek motorla rapor, v1'in doğrulanmış haritasından yeniden kurulan raporla aynıdır; ev sistemi ayarı raporu değiştirmez; hatalar `compute_chart` ile aynı kodlara eşlenir; ortanca süre ≤ 10 ms. (`tests/avastha.rs`: `assert!`)
- [ ] **[otomatik]** Panel: istek açık haritanın girdisiyle bir kez yapılır; yedi satır ve dört dizi; Lajjitadi hücresi gerçek bir listedir ve kuralı sağlayan grahaları yazar; boş değer çizgi ve sözcüktür; terim tablosu, toplam ve artan Rust'tan gelir; "doğrulanmamış tablo" notu, okuma notu ve yöntem kimliği yazılıdır. (`AvasthaPanel.test.tsx` ilk dört test; e2e ilk akış)
- [ ] **[otomatik]** Panel durumları: doğum saati bilinmeyen harita (açıklama satırı, Ay'ın satırında "≈", terim tablosu yok); gün doğumsuz rapor (ayrı açıklama); düşük hassasiyet uyarısı; açık harita yok; hata ve yeniden deneme; klavyeyle yeniden denemede odak panelde durur; yükleme evreleri. (`AvasthaPanel.test.tsx` yedi test; e2e ikinci akış)
- [ ] **[otomatik]** Ortak vurgu grahanın iki tablodaki satırını ve haritadaki grahayı işaretler; kaydırma kapları klavye odağı alır; panel paletten açılır, odağı alır, görünür alana gelir ve aynı komutla çıkarılır; odak gövdeye düşmez. (`AvasthaPanel.test.tsx › shared highlight …`; e2e ilk akış)
- [ ] **[otomatik]** Graha tablosunun Baladi sütunu: varsayılan olarak kapalıdır ve istek yapmaz; açıkken yedi grahanın basamağını yazar, düğümlerde ve lagna satırında boştur; Karaka sütunundan sonra durur; saat bilinmiyorsa yalnız Ay "≈" taşır; ayar düzenle birlikte kaydedilir; var olan on graha tablosu testi değişmeden geçer. (`GrahaTableBaladi.test.tsx` altı test; e2e ilk akış)
- [ ] **[otomatik]** Her durum kimliğinin adı vardır ve adlar tektir; dizi adları ve panel adı üç dilde vardır; Fransızca metinlerde düz kesme işareti yoktur; üç dilin anahtar kümeleri ve yer tutucuları eşittir. (`AvasthaPanel.test.tsx` son iki test; `i18n.test.ts`)
- [ ] **[otomatik]** Baskı sayfaları ve görsel temeller değişmedi; bağımlılıklar, veritabanı ve var olan hesap modülleri değişmedi. (`e2e:visual` on test; Step 2 ve Step 6'nın `git diff --stat` satırları)
- [ ] **[otomatik]** El örnekleri motorun kaynağına bakılmadan yeniden türetildi ve uyuştu. (Step 4; REVIEW.md "Gözden geçirme kaydı")
- [ ] **[insan]** **Basılı kaynak denetimi (asıl kapı).** `tests/golden/modules/avastha/REVIEW.md`'nin onay tablosundaki on dokuz nokta basılı bir BPHS çevirisiyle ya da avasthaları anlatan bir derlemeyle karşılaştırılır: Lajjitadi'nin altı kural metni (düşmanlığın doğal mı bileşik mi okunduğu, bakışın türü, su burcu kümesi, Kshudhita'nın Satürn kolu, Mudita'da "ya da", birden çok durumun birlikte gösterilmesi) ve Shayanadi formülünün graha sayıları, navamsa terimi, ghati sayımı, artanın sıfır olduğu durum ve on iki adın sırası. Sonuç tablonun altındaki "Denetleyen" satırına yazılır. Onaydan sonra `avastha.unverified` notu ayrı bir commit'le çıkarılır.
- [ ] **[insan]** Jagradadi'nin panelde durup durmayacağına ve dostluğun doğal okunmasına (D4; REVIEW 1–2) sahibi karar verir.
- [ ] **[insan]** Gerçek uygulamada, **yalnızca deneme danışanlarıyla**: palette "Avasthalar panelini aç/kapat" paneli açar; yedi satır okunur; bir grahanın Baladi basamağı graha tablosundaki derecesiyle elle sağlanır (ör. tek burçta 14° üçüncü dilimdir: Yuva).
- [ ] **[insan]** Shayanadi terimleri haritanın öteki panelleriyle tutarlıdır: terim tablosundaki nakshatra numarası graha tablosundaki nakshatranın sırasıdır; ortak terimlerdeki doğum nakshatrası Ay'ın nakshatrasıdır; ghati, Panchang panelindeki gün doğumu ile doğum saati arasındaki dakikanın 24'e bölümünün tam kısmıdır; lagna sırası haritadaki lagna burcudur.
- [ ] **[insan]** Doğum saati bilinmeyen bir deneme haritasında açıklama satırı görünür, Shayanadi sütunu çizgidir ve Ay'ın satırı "≈" taşır. Kutup dairesinin ötesinde, gün doğumu olmayan bir günde kurulan deneme haritasında (ör. Tromsø, 21 Aralık) "gün doğumu yok" açıklaması görünür ve öteki üç dizi doludur.
- [ ] **[insan]** Sayı prashnası haritasında panel açılır ve ortak terimlerdeki lagna, haritanın gösterdiği (sayıdan gelen) lagna burcudur.
- [ ] **[insan]** Graha tablosunda "Sütunlar" altındaki "Baladi" kutusu sütunu açar; ayar uygulama yeniden başlatılınca yerindedir; baskı önizlemesindeki graha tablosunda Baladi sütunu yoktur.
- [ ] **[insan]** Panel açık ve koyu temada, Türkçe, İngilizce ve Fransızca dilde okunur; dar panelde tablolar yatay kayar ve Lajjitadi hücresindeki durumlar noktalı virgülle ayrılır; yaklaşık ve boş değerler renk olmadan da ayırt edilir.
- [ ] **[insan]** VoiceOver: tablo başlığıyla duyurulur; Lajjitadi hücresi liste olarak, boş hücre "hiçbiri" ya da "hesaplanmadı" diye, Ay'ın yaklaşık hücresi "yaklaşık" diye okunur; kaydırma kapları "Shayanadi terimleri" adıyla odak alır.
- [ ] **[insan]** Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum anı, yer, koordinat ya da rapor içeriği görünmez.

Kapı geçtiyse ilerleme notlarına `Plan 24 Task 8 elle denetim: otomatik maddeler geçti, insan maddeleri sahibine devredildi (<tarih>)` yazılır. Bu adım commit üretmez.

---

## Başarı ölçütleri

- Yedi grahanın Baladi, Jagradadi, Lajjitadi ve Shayanadi durumu tek komutla, saf bir çekirdekten ve haritanın kendi ayarıyla hesaplanır; sonuç saklanmaz ve `Chart`'a eklenmez.
- Lajjitadi bir kümedir: her tutan kural, kuralı sağlayan grahalarla birlikte döner; altı kuralın her birinin tutan ve tutmayan durumu birim testlerinde ve el örneklerinde vardır.
- Shayanadi formülünün her terimi, toplamı ve artanı raporda durur; formülün bölmesiz ikinci yazımı her girdide aynı durumu verir; on iki durumun hepsi el örneklerindedir.
- Doğum saati bilinmiyorsa Lajjita ve Shayanadi Rust'ta boş döner; gün doğumu yoksa yalnızca Shayanadi boştur; panel ikisini ayrı ayrı söyler.
- Okuma seçimleri `avastha.bphs_v1` kimliğine bağlıdır ve REVIEW.md'de on dokuz varyant noktasıyla yazılıdır; onaya dek panel "doğrulanmamış tablo" notunu taşır.
- Paralel planlarla paylaşılan her dosyadaki değişiklik ekleme türündedir ve "Paylaşılan dosyalar" tablosundadır; ortak komut, test ve stil dosyalarına dokunulmamıştır.
- Sayaçlar: `jyotish-core` lib 429, `avastha_golden` 5, `jyotish-desk` `avastha` 1, Vitest 90 dosya / 907 test, e2e 29; öteki bütün sayaçlar tabandaki gibidir.

## Bu planın kapsamadıkları

- Shayanadi'nin üç alt durumu (cheshta; ad harfine dayalı sabitler ister) ve Deeptadi (dokuz durum): yol haritası §4.4 ve §7.
- İkinci okumalar ve bunların ayarı: bileşik dostluk, dereceden bakış, öncelik sırasıyla tek Lajjitadi durumu, düğümlere Shayanadi, navamsa burcunun sırası, süren ghati (Y5: tek seçenekli okuma ayar olmaz). Sahibi REVIEW'deki bir noktayı değiştirirse kural ayrı bir commit'le değişir ve yöntem kimliği `avastha.bphs_v2` olur.
- Avasthaların vargalarda (ör. navamsada) ya da geçiş anında hesaplanması; avastha koşuluyla harita araması (çekirdek fonksiyonlar saftır ve sonradan koşul olarak eklenebilir).
- Avasthaya dayalı güç, puan ya da yüzde; durumların ne anlama geldiğini anlatan metin; renkli derecelendirme.
- Panelin hazır düzenlere eklenmesi; harita çiziminde avastha işareti; baskı düzeni (K10).
- Dasa ya da geçiş panellerinde avastha sütunu; lagnaya ve düğümlere Baladi.
- Yorum metni, AI paneli; Windows ve Android.

## Spec'e geri bildirim (yol haritası farkları ve boşlukları)

Kararlar bölümünde işaretlenen maddeler; yol haritası bir sonraki düzenlemede bunlara göre güncellenmelidir.

1. **(D4) Jagradadi.** Yol haritası eklenmesini sahibine bırakır; eklendi. Kuralı (yücelme ve kendi burcu Jagrat; dost ve nötr burç Swapna; düşman burcu ve düşüş Sushupti), dostluğun doğal okunduğu ve moolatrikonanın ayrı basamak olmadığı §4.4'e yazılmalıdır.
2. **(D5, D6) Lajjitadi'nin okuması.** §4.4 açık noktaları sayar ama seçimi yazmaz: doğal ilişki, burçtan tam graha bakışı, su burcu kümesi (Yengeç, Akrep, Balık), Kshudhita'nın dördüncü kolu (Satürn ile birliktelik), Mudita'da kolların "ya da" ile bağlandığı ve Jüpiter kolu, Kshobhita'da "kötücül ya da düşman bakışı". Gerekçesiyle (bileşik ilişkide aynı burçtaki graha hep geçici düşmandır) yazılmalıdır.
3. **(D21) Görev taslağı.** §5 Plan 24'ün (6) ve (7) maddeleri: panelin i18n metinleri ve "saat bilinmiyor" durumu panelle aynı görevdedir; (7) Baladi sütunu ve uçtan uca akışlardır.
4. **(D7) `LajjitadiHit`.** Sonucun yalnızca durum kümesi değil, her durum için kuralı sağlayan grahaların listesi olduğu yazılmalıdır.
5. **(D8) Shayanadi'nin terimleri.** Graha sayıları (yedi graha), navamsa teriminin burç içi dilim sırası olduğu, ghatinin tamamlanmış ve 24 dakikalık sayıldığı, artan 0'ın on ikinci durum olduğu, on iki adın sırası ve raporun terimleri, toplamı ve artanı taşıdığı yazılmalıdır.
6. **(D10) Rapor biçimi.** §4.4 "saat bilinmiyorsa `None`" der; rapor `None` olmaz, `time_known: false` ile döner ve yalnızca lagnaya bağlı parçalar (Lajjita, Shayanadi) boştur. Gün doğumu olmayan yer ve sayı prashnası durumları eklenmelidir.
7. **(D2) Düğümler.** Hiçbir dizide Rahu ve Ketu'ya durum atanmadığı (Baladi dahil) ve Lajjita'da eşlikçi sayıldıkları yazılmalıdır.
8. **(D12) El örneklerinin yeri.** `tests/golden/modules/avastha/` iki alt klasörlüdür ve ayrı bir test dosyasıyla yüklenir; Y4'ün `tests/golden/modules/<modül>/REVIEW.md` yolu aynıdır.
9. **(D13) Bütçe.** `compute_avasthas` için hedef yazılmamıştır: ortanca ≤ 10 ms (ölçüm 0,6 ms).
10. **Y4'ün notu.** "Doğrulanmamış tablo" notunun metni ve yeri (panelin üstünde, her zaman) ile onaydan sonra nasıl çıkarılacağı (ayrı commit) yol haritasında tek bir yerde tanımlanmalıdır; Plan 23, 28 ve 31 aynı notu kullanacaktır.

## Self-Review

1. **Kapsam:** yol haritası §4.4'ün ve §5 Plan 24'ün her maddesi "Kapsam" tablosunda bir göreve bağlıdır. Taslaktan bilerek ayrılan yerler Kararlar'da işaretlidir (D4, D5, D21) ve geri bildirim listesindedir. Sekiz görev vardır; son görev kapılar, REVIEW.md ve **[otomatik]**/**[insan]** maddeleridir.
2. **Yer tutucu taraması:** planda "TBD", "TODO", "sonra doldurulur", "uygun hata işleme ekle" ya da kodu olmayan kod adımı yoktur; her kod adımı tam dosya ya da uygulanabilir `diff` taşır. `<tarih>` ve `<kısa özet>` yürütme anında yazılan değerlerdir (Task 8 Step 4 ve Step 7); `BASE` Task 1 Step 1'de okunur.
3. **Tip ve ad tutarlılığı:** `Baladi`, `Jagradadi` Task 1'de; `Lajjitadi`, `LajjitadiHit` Task 2'de; `Shayanadi`, `ShayanadiBasis`, `ShayanadiHit`, `GrahaAvastha`, `AvasthaReport` Task 4'te tanımlanır ve sonraki görevlerde aynı adlarla kullanılır. Komut adı Rust'ta `compute_avasthas`, arayüzde `computeAvasthas`'tır; panel türü `avastha`, i18n bloğu `avastha`, fikstür alanları `avasthas`, `avasthas_time_unknown`, `avastha_ids`'tir. Arayüzün ad tabloları fikstürdeki Rust kimlikleriyle testte karşılaştırılır.
4. **Dersler:** tek okuma `method` kimliğine bağlıdır ve varyantlar REVIEW'dedir (Y4, Y5); el örneği motorla uyuşmazsa kural yeniden okunur (bu plan yazılırken iki kez; Global Constraints); arayüz bölme yapmaz, artan Rust alanıdır (C4); lagnaya bağlı çıktı Rust'ta boşaltılır (K2); yeniden denemede odak gövdeye düşmez (panel testi); bütçe `assert!` ile doğrulanır (D13); paylaşılan dosyalara yalnızca ekleme yapılır ve liste tamdır ("Paylaşılan dosyalar"); üretilen dosyalar yalnızca iki komutla güncellenir (C10); gerçek uygulama çalıştırılmaz ve 1421 başka bir koşuya aitse beklenir.
5. **Koşulmuş kod:** bu plandaki her kod bloğu, `7270550`'nin bir kopyasında sekiz ayrı commit olarak uygulanmış ağaçtan alınmıştır. Task 1–5'in commit'lerinde `cargo test -p jyotish-core`, Task 6'nın commit'inde `cargo test --workspace` (başarısız sonuç satırı 0), `cargo clippy --workspace --all-targets -- -D warnings`, Vitest (89 / 901) ve `pnpm build`; son ağaçta `cargo fmt --all --check`, clippy, `cargo test --workspace` (0), Vitest (90 / 907), `pnpm build`, e2e (29), görsel (10), `cargo deny check`, 1.93 ile `cargo check` ve yayın betiği testleri (14) koşulmuş ve geçmiştir. "Testlerin düştüğünü gör" adımlarından Task 6 Step 6 ile Task 7 Step 2 kopyada koşulmuş ve beklenen sayılar oradan alınmıştır; öteki kırmızı adımların beklentisi derleme hatasının türüdür. Görsel karşılaştırma kopyadaki temellerle yapılmıştır; sahibin yerel temelleriyle karşılaştırma Task 8 Step 3'tedir. **[insan]** maddeleri, Task 8 Step 4'ün bağımsız yeniden türetmesi ve yeni bir çalışma ağacındaki bağımlılık kurulumu koşulmamıştır. Lajjitadi kuralları ve Shayanadi formülü basılı kaynakla karşılaştırılmamıştır (Y4).
6. **Yasaklı ad:** bu belge ve bütün kod blokları `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile taranmıştır; çıktı boştur.
