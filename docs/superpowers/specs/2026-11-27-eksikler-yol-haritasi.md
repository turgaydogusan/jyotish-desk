# jyotish-desk — Eksikler yol haritası (Plan 23 ve sonrası)

Tarih: 2026-11-27 · Durum: Ürün kararları proje sahibinin devriyle verildi (Fable); sahibin gözden geçirmesine açık. Bu belge bir tasarım belgesi değildir: her plan yazılmadan önce kendi tasarım bölümü (bu belgeye ek ya da ayrı spec) ve "Kararlar" listesi yazılır · İlgili belgeler: `2026-11-24-jyotish-desk-v1-design.md` (v1 spec), `2026-11-24-ui-design.md` (UI spec), `2026-11-26-alt-proje-3-ileri-moduller-design.md` (ileri modüller spec'i; aşağıda "İM spec")

## 1. Amaç

Sahibi, uygulamayı klasik bir masaüstü Jyotish programıyla yan yana koydu ve bizde bulunmayan on sekiz kalem saydı. Bu belge o listeyi koda bakarak doğrular (§3), her kalemi tanımlar, bağımlılığını, zor yerini, kaynak ve tablo riskini ve boyutunu yazar (§4), kalemleri en çok sekiz görevlik planlara böler ve sıraya koyar (§5, §6). Planlanmayan üç şey §7'dedir.

Karşılaştırma yalnızca "hangi teknik var, hangisi yok" düzeyindedir. Öteki programın adı, arayüz düzeni, çıktısı ve metni bu belgeye, koda ve fixture'lara girmez (v1 spec §1; İM spec K9).

Değişmeyen kurallar İM spec §1 ve §2'den aynen taşınır: clean-room; yorum metni yok; arayüz hesap yapmaz; yalnızca macOS; modüller `Chart`'a eklenmez, istek üzerine hesaplanır (K1); saf çekirdek ve ince toplayıcı (K3); saat bilinmiyorsa lagnaya bağlı çıktı `None` (K2); başka yazılımın çıktısı fixture olmaz (K9); baskı düzeni eklenmez (K10); panel paletten açılır (K12); üç dilde (tr, en, fr) aynı transliterasyon (K11); adlandırma yasağı (v1 spec §10, İM spec Ek A).

## 2. Kararlar

Sahibi kararları devretti; muhafazakâr seçenek seçildi ve her karar bir satır gerekçeyle yazıldı.

- **Y1 — Sıra ölçütü: uygulayan bir astrologa değer ÷ maliyet.** Değer, tekniğin danışan görüşmesinde ne sıklıkta açıldığıdır; maliyet, görev sayısı ile sahibin basılı kaynak denetimi yükünün toplamıdır. Gerekçe: sahibin istediği ölçüt; denetim yükü de maliyettir, çünkü onaylanmamış tablo "doğrulanmamış tablo" notuyla sunulur ve değeri düşer.
- **Y2 — Her plan en çok sekiz görevdir; son görev kapılar ve elle denetimdir.** Sekize sığmayan kalem ikiye bölünür (Varshaphala: Plan 27 ve 28). Gerekçe: Plan 14–22'nin ölçüsü.
- **Y3 — Kapı testlerdir, gerçek uygulama değil.** Her planın **[insan]** maddeleri sahibine devredilir ve sonraki planı engellemez. Gerekçe: Plan 19–22'deki uygulama.
- **Y4 — Bellekten aktarılan her klasik tablo sahibin onayını bekler.** Tablo `tests/golden/modules/<modül>/REVIEW.md`'ye bilinen varyant noktalarıyla yazılır; onaya dek panel "doğrulanmamış tablo" notu taşır. Yürüten ajan tabloyu başka yazılımla ya da çevrimiçi kaynakla karşılaştırmaz. Gerekçe: kuta ve muhurta tablolarındaki yol (İM spec §17).
- **Y5 — Tek seçenekli okuma ayar olmaz (K7).** Aşağıda "varyant" diye işaretlenen her yerde tek okuma seçilir, `method` kimliğine yazılır ve ikinci okuma ancak uygulanınca ayar olur.
- **Y6 — Yeni kalıcı veri eklenmez.** Bu yol haritasındaki hiçbir plan veritabanı şemasını değiştirmez; istisna çıkarsa o planın kendi kararıdır. Yıllık harita, ilerletilmiş harita ve olay haritası anlık hesaplanır; saklanmak istenirse var olan `ChartKind::Event` kaydı kullanılır. Gerekçe: K1 ve İM spec §14.
- **Y7 — Yorum metinleri, kart özelliği ve AI yorum paneli planlanmaz (§7).** Gerekçe: sahibin açık ertelemesi ve kapsam.
- **Y8 — Yasak harf dizisini içeren klasik adlar yeniden adlandırılır.** Bu belgede geçen tekniklerin hiçbirinin adı yasak diziyi içermez; alt kalemlerde çıkarsa (ör. sahamlardan biri, gün dilimi adları) İM spec Ek A'ya satır eklenir. Her plan commit öncesi depo taramasını sürdürür.

## 3. Doğrulama: "bizde yok" iddiaları ve koddaki durum

`crates/` ve `app/ui` tarandı (2026-11-27, `main` `8b69747`). Sonuç: on sekiz kalemin dokuzu hiç yok, yedisi kısmen var, ikisi kapsam dışı ya da ertelenmiş.

| Kalem | Durum | Kodda bulunan |
|---|---|---|
| Yogi noktası, yogi, avayogi, eş yogi, dagdha rasi | Yok | Girdilerin hepsi var: `panchang.rs` (tithi, nitya yoga numarası, Güneş ve Ay sidereal boylamı), `dasha.rs::birth_dasha` (nakshatra lordu), `dignity::sign_lord`. Panchang paneli bu beş değeri göstermez. |
| Avasthalar (Baladi, Lajjitadi, Shayanadi) | Yok | Hiçbir avastha kodu yok. Girdiler var: `dignity`, `drishti`, `nature`, `varga_position`, gün doğumu (`Panchang.sunrise_jd_ut`). |
| Vimshopaka bala | Yok | On altı varganın hepsi var (`Varga::ALL`). Vargada bileşik ilişki hesabı Shadbala'nın saptavargaja kaleminde yazılı (`shadbala/sthana.rs::saptavargaja_points`). |
| Tara tabloları | Kısmen | `muhurta/segments.rs::tara_bala` (doğum nakshatrasına göre günün tarası; gün panchangı panelinde, doğum Ay'ı verilince) ve `kuta/score.rs::tara_number` (uyumda). Doğum haritasının 27 nakshatralık navatara tablosu ve geçiş grahalarının tarası yok. |
| Sarvatobhadra chakra | Yok | Tithi grupları (`TithiGroup`) ve 27 nakshatra var. Abhijit nakshatrası, harf hücreleri ve vedha kuralları yok. |
| Sudarshana chakra | Yok | Veri tümüyle var (lagna, Ay ve Güneş burcu; burç evi). Eksik olan çizim ve paneldir. |
| Varshaphala | Yok | Güneş'in doğum boylamına dönüş anı olay aramasında bulunuyor (`SkyEventKind::TransitToNatal`). Muntha, yıl lordu, mudda dasa, sahamlar ve Tajika bakışları yok (Tajika bakışları İM spec §4'te kapsam dışı bırakılmıştı). |
| İlerletmeler | Yok | — |
| Sankranti listesi | Kısmen | Olay araması Güneş'in burç girişlerini veriyor (`SignIngress`, graha seçimi Güneş). Hazır "Sankranti" görünümü, giriş anının panchangı ve yıl listesi yok. |
| Tutulmalar | Yok | `ffi.rs` yalnızca `swe_calc_ut`, `swe_houses_ex2`, `swe_rise_trans` ve ayanamsa çağrılarını bağlıyor. Tutulma fonksiyonları depodaki ephemeris kaynağında derleniyor (`vendor/swisseph/swecl.c`) ama bağlanmamış. Arayüz testleri `"Eclipse"` değerini bilinmeyen tür olarak eliyor. |
| Geçiş takvimi | Kısmen | Olay araması düz liste veriyor (dokuz olay türü); gün panchangı tek günü veriyor. Ay ızgarası görünümü yok. |
| Animasyonlu geçişler | Kısmen | Geçiş anı seçilebiliyor (`TransitDate`: tarih, saat, "Şimdi"), D1 haritasında geçiş halkası ve geçiş tablosu var. Adım düğmeleri ve oynatma yok. |
| Efemeris tablosu | Yok | Tek anın konumları var (`compute_transits`); gün gün tablo yok. |
| Geçiş Ay'ının nakshatra lordları | Kısmen | Geçiş tablosu nakshatra ve padayı gösteriyor; olay araması nakshatra girişlerini veriyor. Nakshatra lordu hiçbir tabloda sütun değil (graha tablosunun on bir sütunu arasında da yok). |
| Bhava chalit ve graha–ev başlangıcı dizisi | Kısmen | Sripati ve Placidus bhava başlangıçları hesaplanıyor (`Bhavas.cusps`), graha tablosundaki "ev" sütunu bunlara göre (`house_from_cusps`), geçiş tablosu doğum bhavalarına göre ev veriyor. Bhava ortası (madhya), başlangıç tablosu ve chalit harita çizimi yok. |
| Mundane haritalar | Kısmen | Her an ve yer için olay haritası kaydedilebiliyor (`ChartKind::Event`). Giriş, yeni ay ve tutulma anından tek adımda harita açma yok. |
| Yorum metinleri | Ertelendi | v1 spec §2 satır 6; sahibi isteyince (§7). |
| Vedik olmayan kart özelliği | Kapsam dışı | §7. |

## 4. Eksikler

Boyut: **S** bir-iki görev, **M** üç-beş görev, **L** altı-sekiz görev (çekirdek, IPC, panel, i18n ve testler dahil). "Kaynak riski" sahibin basılı kaynakla denetlemesi gereken tabloyu ya da okuma seçimini söyler.

### 4.1 Panchang ekleri: yogi noktası, yogi, avayogi, eş yogi, dagdha rasi

- **Tanım:** Doğum anının beş ek panchang olgusu. Yogi noktası Güneş ve Ay sidereal boylamlarının toplamına sabit bir yay eklenerek bulunur; yogi o noktanın nakshatra lordu, eş yogi o noktanın burç lordu, avayogi yogi nakshatrasından altıncı nakshatranın lordudur; dagdha rasi doğum tithisine bağlı burç çiftidir.
- **Bağımlılık:** `panchang.rs`, `dasha.rs` (nakshatra lordu), `dignity::sign_lord`; Panchang paneli. Lagna gerekmez; saat bilinmiyorsa değerler "yaklaşık" işaretini alır (Ay günde ~13° yürür).
- **Zor yer ve kaynak riski:** Aritmetik basit. Sabit yay (93°20′) ve avayoginin sayımı (altıncı nakshatra; öteki yazım noktaya 186°40′ eklemek, ikisi aynı lordu vermeli: ikinci yazım testi). **Dagdha rasi tablosu** (on dört tithi × iki burç; chaturdashide dört burç, dolunay ve yeni ayda yok) bellekten yazılır ve basılı kaynakla hücre hücre denetlenmelidir; shukla ve krishna pakshanın aynı tabloyu paylaşıp paylaşmadığı varyant noktasıdır.
- **Boyut:** S.

### 4.2 Tara tabloları (navatara)

- **Tanım:** Doğum Ay'ının nakshatrasından başlayarak 27 nakshatranın dokuz taraya üç turda dağılımı; her satırda nakshatra, lordu ve orada duran doğum grahaları. İkinci görünüm: seçilen geçiş anında dokuz grahanın tarası.
- **Bağımlılık:** `tara_number` ve `TaraBala` (var), `nakshatra_pada`, geçiş deposu (`transits_at`).
- **Zor yer ve kaynak riski:** Düşük. Dokuz taranın adı ve elverişli kümesi muhurta tablolarında zaten onay bekliyor (Janma tarasının sınıfı dahil); yeni tablo getirmez. Karar noktası: tablo yalnızca Ay'dan mı, lagna nakshatrasından da mı sayılır (öneri: yalnızca Ay; lagna ikinci okuma olur).
- **Boyut:** S.

### 4.3 Vimshopaka bala

- **Tanım:** Yedi grahanın varga kümelerindeki konum gücü, yirmi üzerinden; dört küme: shadvarga, saptavarga, dashavarga, shodashavarga.
- **Bağımlılık:** `varga.rs` (on altı varga), `dignity` (bileşik ilişki), `shadbala/sthana.rs` (aynı vargada ilişki okuması yeniden kullanılır). Shadbala paneline üçüncü bir kardeş panel.
- **Zor yer ve kaynak riski:** İki tablo basılı kaynak ister: **(a) dört kümenin varga ağırlıkları** (toplamı 20 olmalı: değişmez testi), **(b) konum puanı basamakları** (kendi burcu, çok dost, dost, nötr, düşman, çok düşman için 20–18–15–10–7–5 dizisi yaygındır; yücelme ve moolatrikonanın ayrı basamak olup olmadığı, geçici dostluğun her vargada D1'den mi o vargadan mı okunduğu varyanttır; Shadbala'da D1 seçilmişti). Rahu ve Ketu'ya puan verilmez. Yüksek vargalar (D40, D45, D60) doğum saatine çok duyarlıdır: saat "yaklaşık" ise uyarı.
- **Boyut:** M.

### 4.4 Avasthalar: Baladi, Lajjitadi, Shayanadi

- **Tanım:** Grahanın "durum" sınıfları. Baladi: burç içi dereceye göre beş yaş basamağı (tek burçta düz, çift burçta ters sıra). Lajjitadi: yerleşim, kavuşum ve bakışa göre altı durum. Shayanadi: nakshatra, graha sırası, navamsa, doğum nakshatrası, gün doğumundan geçen ghati ve lagnadan üretilen on iki durum.
- **Bağımlılık:** `Positions`, `dignity`, `drishti` (burç bakışı), `nature`, `varga_position` (navamsa), gün doğumu, lagna. Shayanadi lagna ve saat ister: saat bilinmiyorsa `None`.
- **Zor yer ve kaynak riski:** Baladi tablo riski taşımaz ve bir görevin yarısıdır: **alınmaya değer**. **Lajjitadi** kural metni yoruma açıktır (bir graha aynı anda birden çok durumda olabilir: sonuç kümedir; "düşmanla birlikte" doğal mı bileşik mi; bakış burçtan mı dereceden mi; Trushita'nın "su burcu" kümesi). **Shayanadi** en riskli kalemdir: çarpım formülündeki graha sayıları, ghatinin tamamlanmış mı süren mi sayıldığı, bölümden artanın sıfır olduğu durum ve on iki adın sırası basılı kaynaktan denetlenmelidir; alt durum (üç cheshta) ad harfine dayalı sabitler ister ve **kapsam dışı** bırakılır. İki dizinin de el örnekleri kuraldan türetilir (İM spec §13). Jagradadi (üç durum) Baladi kadar ucuzdur ve aynı plana eklenebilir; Deeptadi (dokuz durum) eklenmez (varyantı çok, sahibi istemedi).
- **Boyut:** M (Baladi S, Lajjitadi M, Shayanadi M; ortak panel).

### 4.5 Bhava chalit ve graha–ev başlangıcı dizisi

- **Tanım:** Seçili ev sisteminde on iki bhavanın başlangıcı, ortası ve bitişi (burç, derece, nakshatra-pada, burç lordu ve nakshatra lordu); her grahanın bhavası ve bhava ortasına uzaklığı; grahaları burca göre değil bhavaya göre yerleştiren chalit harita görünümü.
- **Bağımlılık:** `Bhavas`, `house_from_cusps`, `ephemeris::houses` (Sripati'de bhava ortası için Porphyry noktaları da gerekir), `ChartSvg` ve harita panelinin görünüm ayarı.
- **Zor yer ve kaynak riski:** Tablo riski yok; okuma seçimleri var: Sripati'de dönen değerler bhava başlangıcıdır (sandhi), orta ayrıca hesaplanır; ev sistemi tam burç ise chalit görünümü rasi ile aynıdır ve panel bunu söyler; kutup dairesinde Placidus'un Porphyry'ye düşmesi (var olan uyarı) burada da gösterilir. `Chart` dondurulduğu için (K1) yeni komut gerekir. Çizimde zor yer: bir bhavanın iki burca yayıldığı durumda Kuzey ve Güney Hint çizimlerinin tutarlı gösterimi ve görsel temeller. Nakshatra alt lordu (KP) **kapsam dışı**; sahibi isterse ayrı kalemdir.
- **Boyut:** M.

### 4.6 Sudarshana chakra

- **Tanım:** Lagna, Ay ve Güneş'i birinci ev sayan üç iç içe halkalı tek çizim; her halkada on iki ev ve içindeki grahalar.
- **Bağımlılık:** `Positions` (burç evi, K4); yeni SVG geometrisi; ortak seçim (UI spec §3.4).
- **Zor yer ve kaynak riski:** Hesap yok denecek kadar az; iş çizimdedir: üç halkada 36 hücreye sığan etiketler, klavye erişimi, çizimin tablo karşılığı (erişilebilirlik), görsel temeller. Saat bilinmiyorsa lagna halkası çizilmez. Sudarshana dashası **kapsam dışı**.
- **Boyut:** M.

### 4.7 Geçiş Ay'ının nakshatra lordları

- **Tanım:** Geçiş tablosuna, graha tablosuna ve olay aramasının nakshatra girişi satırlarına "nakshatra lordu" sütunu; Ay'ın önümüzdeki nakshatra girişleri lordlarıyla.
- **Bağımlılık:** `dasha.rs` (Vimshottari lord sırası), `TransitGraha`, `GRAHA_COLUMNS`, `SkyEvent.nakshatra`.
- **Zor yer ve kaynak riski:** Yok; lord arayüzde türetilmez, Rust tipine alan olarak eklenir (C4). Bağlama türü değişimi görsel ve IPC temellerini oynatır.
- **Boyut:** S.

### 4.8 Sankranti listesi

- **Tanım:** Seçilen yılın on iki güneş girişi: yerel saat, girilen burç, giriş anının tithisi, varası ve Ay nakshatrası.
- **Bağımlılık:** `search/events.rs` (`SignIngress`, Güneş), `compute_panchang`, bulunulan yer (`home_place`).
- **Zor yer ve kaynak riski:** Düşük. Karar noktası: çerçeve. Sankranti sidereal bir kavramdır; harita tropikal çerçevedeyse bile liste seçili ayanamsayla sidereal verilir (K5'ten bilinçli ayrılma, o planın kararına yazılır). Girişin "kutlu süresi" (punya dilimi) tablo ister ve **kapsam dışı**.
- **Boyut:** S.

### 4.9 Efemeris tablosu

- **Tanım:** Seçilen ay için gün gün dokuz grahanın boylamı (burç, derece, retro işareti), isteğe bağlı nakshatra; sabit bir saatte.
- **Bağımlılık:** `Ephemeris::body`, `timeline` (dönüş ve giriş günlerini işaretlemek için), ayarlar (ayanamsa, düğüm türü).
- **Zor yer ve kaynak riski:** Tablo riski yok. Kararlar: satır saati (öneri: bulunulan yerin yerel gece yarısı; 00:00 UT ikinci okuma), harita açık değilken hangi ayarın kullanıldığı (genel varsayılan), aralık sınırı (bir ay; yıl görünümü ayrı). Bütçe: ay başına 31 × 9 çağrı, tek motor işi. Erişilebilir geniş tablo ve yatay kaydırma UI işidir. CSV ve baskı **kapsam dışı** (K10).
- **Boyut:** M.

### 4.10 Animasyonlu geçişler

- **Tanım:** Geçiş anını adım adım (saat, gün, hafta, ay, yıl) ileri-geri yürüten düğmeler ve oynat-durdur; geçiş halkası ve tablo birlikte güncellenir.
- **Bağımlılık:** `transit/store.tsx` (`setAt`), `transits_at`, harita panelinin geçiş halkası.
- **Zor yer ve kaynak riski:** Kaynak riski yok. Zor yer eşzamanlılıktır: tek ephemeris iş parçacığı (İM spec §17), eski yanıtın düşürülmesi, oynatma sürerken harita açmanın beklememesi, `prefers-reduced-motion` (oynatma yerine yalnızca adım), panel kapanınca zamanlayıcının durması. Adım aritmetiği (bir ay sonrası, yaz saati boşluğu) Rust'ta çözülür.
- **Boyut:** S–M.

### 4.11 Geçiş takvimi

- **Tanım:** Ay ızgarası: her günün hücresinde gün doğumundaki tithi ve Ay nakshatrası ile o güne düşen olaylar (burç ve nakshatra girişleri, dönüşler, yeni ay, dolunay; harita açıksa doğum boylamına gelişler).
- **Bağımlılık:** `event_search` ve dilimleme, `timeline::sun_days`, gün panchangı, `home_place`.
- **Zor yer ve kaynak riski:** Kaynak riski yok. Zor yerler: gün sınırı (takvim günü yerel gece yarısıyla, panchang günü gün doğumuyla başlar; hücre hangisini gösterir kararı), kutup dairesinde gün doğumu olmayan günler, ay başına olay sayısının sınırı, ızgaranın klavye ve ekran okuyucu erişimi, üç dilde hafta başlangıcı. Olay araması çekirdeği yeniden kullanılır; yeni astronomi yoktur.
- **Boyut:** M.

### 4.12 Tutulmalar

- **Tanım:** Bir tarih aralığındaki güneş ve ay tutulmaları: tür, en büyük evre anı, Güneş ve Ay'ın burcu ve nakshatrası; bulunulan yerden görünür mü; harita açıksa doğum grahalarına yakınlık.
- **Bağımlılık:** `ephemeris` kasasına yeni bağlama (kaynak depoda derleniyor), `Ephemeris` trait'ine yeni yöntem (sahte ephemeris de uygular), `SkyEventKind`'a iki tür, olay araması paneli.
- **Zor yer ve kaynak riski:** Tablo riski yok; doğrulama bilinen astronomik tutulma anlarıyla yapılır (yayımlanmış gökbilim verisi; başka astroloji yazılımı değil). Zor yerler: `unsafe` FFI yüzeyinin büyümesi ve gözden geçirmesi, yerel görünürlük hesabının bütçesi, sahte ephemeris'te tutulmanın temsili, `SkyEventKind` genişleyince kayıtlı aramaların ve arayüz ayrıştırıcısının geriye uyumu. Tutulmaya bağlı törensel süreler **kapsam dışı**.
- **Boyut:** M.

### 4.13 Mundane haritalar

- **Tanım:** Güneş girişi (özellikle Koç girişi), yeni ay, dolunay ve tutulma anı için seçilen yerde harita açma; istenirse olay haritası olarak kaydetme.
- **Bağımlılık:** Sankranti listesi (4.8), tutulmalar (4.12), olay araması, atlas yer araması, `ChartKind::Event`, prashnadaki "an ve yerden harita" akışı.
- **Zor yer ve kaynak riski:** Hesap riski yok; iş bir akıştır ("bu olayı harita olarak aç"). Ülke ve başkent listesi, kuruluş haritaları ve mundane kuralları tablo ve siyasi veri ister: **eklenmez**; yeri astrolog atlastan seçer.
- **Boyut:** S (4.8 ve 4.12'den sonra).

### 4.14 Varshaphala (Tajika yıllık harita)

- **Tanım:** Güneş'in doğum boylamına döndüğü anın haritası ve yıllık öğeler: muntha, yıl lordu, mudda dasa, sahamlar.
- **Bağımlılık:** `TransitToNatal` çekirdeği (dönüş anı), `compute_chart_for` (yıllık harita), `dasha.rs` (mudda dasa Vimshottari oranlarının bir yıla sıkıştırılmasıdır), `dignity`, `varga_position`, `hora_lord` (Plan 17), gün doğumu.
- **Zor yer ve kaynak riski:** Bu yol haritasının kaynak yükü en ağır kalemidir.
  - *Dönüş anı:* sidereal boylama dönüş (seçili ayanamsayla) alınır; tropikal dönüş her yıl yaklaşık yirmi dakikalık bir fark biriktirir ve lagnayı değiştirir. Yer: doğum yeri (öneri) ya da bulunulan yer. İkisi de karar ve `method`.
  - *Muntha:* aritmetik (lagna burcu + tamamlanan yıl); risk yok; saat bilinmiyorsa `None`.
  - *Mudda dasa:* başlangıç lordunun bulunuşu (doğum nakshatrası + yaş sayımı) ve yılın uzunluğu (360 gün mü, dönüşten dönüşe mi) varyanttır: basılı kaynak.
  - *Yıl lordu:* beş aday (muntha lordu, doğum lagnası lordu, yıl lagnası lordu, gündüz-gece üçlülük lordu, gün ya da gece lordu), panchavargiya bala (beş kalem; **hadda tablosu** ve kalem puanları basılı kaynak ister), adayın lagnaya **Tajika bakışı** (açı ve orb tablosu; İM spec'te kapsam dışıydı, burada gerekir) ve eşitlik bozma kuralları. Her adım varyant taşır.
  - *Sahamlar:* her biri üç noktalı bir formüldür; gündüz ve gece doğumunda terimlerin yer değiştirmesi ve "lagna iki terimin arasında değilse 30° ekle" kuralı saham başına denetlenmelidir. İlk sürüm sahibin seçeceği dar bir çekirdek kümeyle (öneri: on altı saham) çıkar; liste REVIEW dosyasında formülüyle durur.
  - Tajika yogaları (on altı yoga), tri-pataki chakra ve yıllık yorum **kapsam dışı**.
- **Boyut:** L; iki plana bölünür: yıllık harita + muntha + mudda dasa (M), yıl lordu + sahamlar (L).

### 4.15 Sarvatobhadra chakra

- **Tanım:** 9 × 9 ızgara: dış çerçevede 28 nakshatra, içeride harfler, on iki burç, tithi grupları ve varalar; doğum ve geçiş grahaları nakshatra hücrelerine yerleşir ve vedha (karşı ve çapraz etki) çizgileri gösterilir.
- **Bağımlılık:** `nakshatra_pada`, `TithiGroup`, geçiş deposu, yeni ızgara bileşeni.
- **Zor yer ve kaynak riski:** Yüksek. **(a) Izgaranın 81 hücrelik yerleşimi** basılı kaynaktan denetlenmelidir (harflerin ve ünlülerin yeri, köşeler). **(b) Abhijit:** motor 27 nakshatra bilir; 28'li bölümde Abhijit'in yayı ve komşularının kısalması yeni bir tablodur. **(c) Vedha yönleri:** grahanın hızına ve retrosuna göre sağ, sol ve karşı vedha; Güneş, Ay ve düğümlerin kuralı varyanttır. **(d) Harflerin** üç dilde transliterasyonu ve ad harfi eşlemesi; ad harfinden çıkarım **kapsam dışı**. İlk sürüm: ızgara, yerleşim ve seçili grahanın vedha çizgileri; iyi-kötü sınıfı ve puan yok.
- **Boyut:** L.

### 4.16 İlerletmeler

- **Tanım:** İkincil ilerletme (bir gün = bir yıl): seçilen yaş için ilerletilmiş konumlar ve harita.
- **Bağımlılık:** `compute_chart_for` (ilerletilmiş an doğumdan yaş kadar gün sonrasıdır), geçiş tarih seçicisi, harita panelinin dış halkası.
- **Zor yer ve kaynak riski:** Klasik Jyotish kaynağı yoktur; teknik batı geleneğinden gelir ve tanımı sahibin seçimidir: ilerletilmiş lagnanın bulunuşu (birkaç yöntem), yılın uzunluğu, sidereal çerçevede gösterim. Hesap ucuz, karar pahalı. Üçüncül ve güneş yayı ilerletmeleri **kapsam dışı**. Sahibi tanımı vermeden plan yazılmaz.
- **Boyut:** S–M.

### 4.17 ve 4.18 Yorum metinleri; Vedik olmayan kart özelliği

Planlanmaz; §7.

## 5. Planlar

| Sıra | Plan | Başlık | Kalemler | Boyut | Bağımlı olduğu |
|---|---|---|---|---|---|
| 1 | 23 | Doğum anı ekleri: panchang ekleri, navatara, Vimshopaka bala | 4.1, 4.2, 4.3 | M | Plan 14, 17, 20 |
| 2 | 24 | Avasthalar: Baladi, Lajjitadi, Shayanadi | 4.4 | M | Plan 14 |
| 3 | 25 | Bhava chalit, ev başlangıcı dizisi ve Sudarshana chakra | 4.5, 4.6 | M | v1 harita çizimi |
| 4 | 26 | Geçiş araçları: nakshatra lordları, Sankranti listesi, efemeris tablosu, adımlı ve oynatılan geçiş | 4.7, 4.8, 4.9, 4.10 | M | Plan 20, 22 |
| 5 | 27 | Varshaphala I: yıllık harita, muntha, mudda dasa | 4.14 (ilk yarı) | M | Plan 22 (dönüş anı) |
| 6 | 28 | Varshaphala II: Tajika bakışları, panchavargiya bala, yıl lordu, sahamlar | 4.14 (ikinci yarı) | L | Plan 27 |
| 7 | 29 | Geçiş takvimi | 4.11 | M | Plan 22, 26 |
| 8 | 30 | Tutulmalar ve mundane haritalar | 4.12, 4.13 | M | Plan 22, 26 |
| 9 | 31 | Sarvatobhadra chakra | 4.15 | L | Plan 23 (tara), geçiş deposu |
| 10 | 32 | İlerletmeler | 4.16 | S–M | sahibin tanımı |

Plan 23, 24, 25 ve 26 birbirinden bağımsızdır; sıraları değişebilir. Plan 29 ve 30 da kendi aralarında yer değiştirebilir.

### Plan 23 — Doğum anı ekleri (M)

Görev taslağı: (1) tabanlar ve `panchang_extras` çekirdeği (yogi noktası, yogi, eş yogi, avayogi); (2) dagdha rasi tablosu, ikinci yazım ve REVIEW kaydı; (3) navatara çekirdeği (doğum tablosu ve geçiş tarası); (4) Vimshopaka çekirdeği: ağırlık tabloları, basamaklar, değişmezler; (5) el örnekleri ve komutlar (`compute_panchang_extras`, `compute_navatara`, `compute_vimshopaka`); (6) Panchang paneline ek satırlar ve Navatara paneli; (7) Vimshopaka paneli, i18n üç dil; (8) kapılar ve elle denetim.

**Sahibin denetimi:** dagdha rasi tablosu (hücre hücre; paksha ayrımı); yogi noktasının sabit yayı ve avayoginin sayımı; Vimshopaka'nın dört ağırlık tablosu ve puan basamakları (yücelme ve moolatrikonanın yeri, geçici dostluğun okunduğu varga); navataranın lagnadan da sayılıp sayılmayacağı.

### Plan 24 — Avasthalar (M)

Görev taslağı: (1) tabanlar, `avastha` modülü ve Baladi (istenirse Jagradadi); (2) Lajjitadi kuralları ve küme sonucu; (3) Lajjitadi el örnekleri (her durum için tutan ve tutmayan); (4) Shayanadi formülü ve toplayıcı (gün doğumu, ghati); (5) Shayanadi el örnekleri ve ikinci yazım; (6) komut ve Avastha paneli; (7) i18n, "saat bilinmiyor" durumu, graha tablosuna isteğe bağlı Baladi sütunu; (8) kapılar ve elle denetim.

**Sahibin denetimi:** Lajjitadi'nin altı kural metni (düşmanlığın doğal mı bileşik mi okunduğu, bakışın türü, su burcu kümesi, birden çok durumun birlikte gösterilmesi); Shayanadi formülünün graha sayıları, ghati sayımı, artanın sıfır olduğu durum ve on iki adın sırası; Jagradadi'nin eklenip eklenmeyeceği.

### Plan 25 — Bhava chalit, ev başlangıcı dizisi ve Sudarshana chakra (M)

Görev taslağı: (1) tabanlar ve `bhava_table` çekirdeği (başlangıç, orta, bitiş; graha uzaklığı); (2) komut ve Bhava tablosu paneli; (3) harita paneline chalit görünümü (adaptör ve geometri); (4) chalit görsel temelleri ve ev sistemi durumları (tam burç, Placidus düşmesi); (5) Sudarshana çekirdeği ve SVG geometrisi; (6) Sudarshana paneli, ortak seçim, tablo karşılığı; (7) i18n ve "saat bilinmiyor"; (8) kapılar ve elle denetim.

**Sahibin denetimi:** Sripati'de bhava ortasının ve sınırının okunuşu; chalit çiziminin iki burca yayılan bhavayı gösterme biçimi (ekranda, **[insan]**); Sudarshana'da üç halkanın sırası (içten dışa lagna, Ay, Güneş önerilir); KP alt lordunun istenip istenmediği.

### Plan 26 — Geçiş araçları (M)

Görev taslağı: (1) tabanlar; nakshatra lordu alanı (`TransitGraha`, graha satırı, olay satırı) ve sütunlar; (2) Sankranti çekirdeği (yıl listesi + giriş anı panchangı); (3) Sankranti görünümü (olay araması panelinde hazır seçim ya da ayrı panel: o planın kararı); (4) efemeris çekirdeği ve komutu; (5) Efemeris paneli; (6) geçiş adımı aritmetiği (Rust) ve adım düğmeleri; (7) oynat-durdur, eski yanıtın düşürülmesi, azaltılmış hareket; (8) kapılar ve elle denetim.

**Sahibin denetimi:** Sankranti'nin tropikal haritada da sidereal verilmesi; efemeris satırının saati (yerel gece yarısı ya da 00:00 UT); oynatma hızları ve adım birimleri (ekranda, **[insan]**). Basılı tablo denetimi yok; isterse basılı bir efemerisle birkaç satırı karşılaştırabilir.

### Plan 27 — Varshaphala I (M)

Görev taslağı: (1) tabanlar ve dönüş anı çekirdeği (yaş → an; ±1″ testi); (2) yıllık harita toplayıcısı ve komutu (yer seçimiyle); (3) muntha ve yıl başlığı olguları; (4) mudda dasa çekirdeği ve ikinci yazım; (5) el örnekleri; (6) Varshaphala paneli (yıl seçici, harita, muntha) ve var olan dasa panellerinde mudda görünümü; (7) i18n, "saat bilinmiyor", olay haritası olarak kaydetme (var olan akış); (8) kapılar ve elle denetim.

**Sahibin denetimi:** dönüşün sidereal mi tropikal mi alındığı; yıllık haritanın yeri (doğum yeri ya da bulunulan yer); munthanın sayımı (tamamlanan yıl); mudda dasanın başlangıç lordu kuralı ve yıl uzunluğu.

### Plan 28 — Varshaphala II (L)

Görev taslağı: (1) tabanlar ve Tajika bakış tablosu (açı, orb, dost-düşman türü); (2) panchavargiya bala: hadda tablosu ve beş kalem; (3) beş aday ve yıl lordu seçimi, eşitlik kuralları; (4) el örnekleri (bala ve seçim); (5) saham çekirdeği: formül tablosu, gündüz-gece değişimi, 30° kuralı; (6) saham el örnekleri ve REVIEW kaydı; (7) panel ekleri (yıl lordu dökümü, saham tablosu), i18n; (8) kapılar ve elle denetim.

**Sahibin denetimi:** en ağır plan. Hadda tablosu (on iki burç × beş dilim, hücre hücre); panchavargiya kalem puanları ve bölme kuralı; Tajika bakış açıları ve orbları; beş adayın tanımı (özellikle üçlülük lordu tablosu ve gün-gece lordu); eşitlik bozma sırası; saham listesi (hangi sahamlar ilk sürümde) ve her birinin formülü ile gece değişimi. Onaya dek yıl lordu ve sahamlar "doğrulanmamış tablo" notuyla sunulur. Bu plan yazılmadan önce sahibin tabloları sağlaması önerilir; sağlanmazsa plan bellekten yazılan tablolarla ilerler ve not durur (Y4).

### Plan 29 — Geçiş takvimi (M)

Görev taslağı: (1) tabanlar ve ay ızgarası çekirdeği (günler, gün doğumu olguları); (2) olayların günlere dağıtımı ve sınırlar; (3) komut ve dilimli yükleme; (4) Takvim paneli: ızgara ve hücre içeriği; (5) klavye gezinme, gün ayrıntısı, gün panchangına geçiş; (6) harita açıkken doğuma göre olaylar ve süzgeç; (7) i18n (hafta başlangıcı, ay adları), boş ve hata durumları; (8) kapılar ve elle denetim.

**Sahibin denetimi:** hücrenin gösterdiği gün tanımı (takvim günü ile panchang günü); varsayılan olay türleri; ızgaranın okunurluğu (ekranda, **[insan]**). Tablo denetimi yok.

### Plan 30 — Tutulmalar ve mundane haritalar (M)

Görev taslağı: (1) tabanlar; ephemeris bağlaması ve `Ephemeris` yöntemi (gerçek ve sahte); (2) bilinen tutulma anlarıyla doğrulama testleri; (3) `SkyEventKind` genişlemesi, kayıtlı aramaların geriye uyumu; (4) yerel görünürlük ve doğum grahalarına yakınlık olguları; (5) olay araması panelinde tutulma satırları; (6) "olayı harita olarak aç" akışı (giriş, yeni ay, dolunay, tutulma; yer seçimi); (7) i18n, olay haritası olarak kaydetme, durum testleri; (8) kapılar ve elle denetim.

**Sahibin denetimi:** tutulma listesinin bir yıl için yayımlanmış bir gökbilim almanağıyla karşılaştırılması (isteğe bağlı; testte bilinen anlar zaten var); yakınlık için kullanılan orb; mundane harita için varsayılan yer (bulunulan yer önerilir). Klasik tablo yok.

### Plan 31 — Sarvatobhadra chakra (L)

Görev taslağı: (1) tabanlar; 28'li nakshatra bölümü ve Abhijit; (2) 81 hücrelik ızgara tablosu, ikinci yazım, REVIEW kaydı; (3) vedha çekirdeği (yönler, hız ve retro kuralı); (4) el örnekleri; (5) komut ve ızgara bileşeni; (6) doğum ve geçiş yerleşimi, seçili grahanın vedha çizgileri; (7) i18n (harf transliterasyonu), erişilebilirlik, görsel temeller; (8) kapılar ve elle denetim.

**Sahibin denetimi:** ızgaranın 81 hücresi; Abhijit'in yayı; vedha yönlerinin hız ve retroya göre kuralı; Güneş, Ay ve düğümlerin vedhası; harflerin Latin yazımı. Onaya dek panel "doğrulanmamış tablo" notu taşır.

### Plan 32 — İlerletmeler (S–M)

Görev taslağı (tanım gelince): ilerletilmiş an çekirdeği; komut; harita panelinde ilerletme halkası ve tablo; i18n; kapılar. Beş görevi geçmez.

**Sahibin denetimi:** planın ön koşuludur: hangi ilerletme (ikincil önerilir), ilerletilmiş lagnanın yöntemi, yıl uzunluğu. Tanım verilmezse plan yazılmaz.

## 6. Sıranın gerekçesi

- **Önce ucuz ve her gün açılanlar (Plan 23–25).** Yogi ve avayogi, navatara, Vimshopaka, avasthalar ve bhava chalit doğum haritası okumasının ilk on dakikasında bakılan tablolardır ve tümü var olan konumlar üzerinde saf aritmetiktir: yeni astronomi, yeni kalıcı veri ve yeni çizim altyapısı gerekmez (chalit ve Sudarshana dışında). Plan 23 en öne alınır, çünkü üç kalemi de birer-iki görevdir ve denetim yükü üç küçük tablodur. Avasthalar değerde eşit ama Shayanadi'nin kaynak riski yüzünden bir adım geridedir.
- **Geçiş araçları (Plan 26) dördüncüdür.** Dört kalemin üçü var olan çekirdeğin yeni görünümüdür (olay araması, geçiş deposu); basılı kaynak denetimi yoktur. Değer orta-yüksek, maliyet düşüktür. Takvim (29) ve tutulmalar (30) bu planın nakshatra lordu alanını ve Sankranti çekirdeğini kullanır.
- **Varshaphala (Plan 27, 28) ortadadır.** Yıllık okuma yapan astrolog için değeri çok yüksektir, ama toplam boyutu L + denetim yükü en ağırdır. Bölünmenin nedeni: ilk yarı (harita, muntha, mudda dasa) az tabloyla kendi başına işe yarar; ikinci yarı (yıl lordu, sahamlar) sahibin sağlayacağı tabloları bekleyebilir ve beklerken öteki planlar yürür.
- **Takvim ve tutulmalar (Plan 29, 30) Varshaphala I'den sonradır.** İkisi de kaynak riski taşımaz, ama bugünkü olay araması listesi aynı bilgiyi (tutulma dışında) zaten veriyor; kazanç sunumdadır. Tutulma tek yeni astronomidir ve `unsafe` yüzeyi büyütür; mundane harita onun ve Sankranti'nin üstüne küçük bir akıştır.
- **Sarvatobhadra (Plan 31) sona yakındır.** Uzman tekniğidir, boyutu L'dir ve dört ayrı tablo denetimi ister.
- **İlerletmeler (Plan 32) sondadır.** Jyotish uygulamasında yeri azdır, klasik kaynağı yoktur ve tanımı sahibin kararını bekler.

## 7. Planlanmayanlar

- **Yorum metinleri: sahibi isteyince.** v1 spec §2'nin 6. satırıdır ve sahibi açıkça erteledi. Takvime konmaz. Bu yol haritasındaki her modül yalnızca olgu ve klasik etiket gösterir; metin gelirse bu tipleri okur.
- **Vedik olmayan kart özelliği kapsam dışıdır;** Jyotish hesabı değildir ve ürün hedefine girmez.
- **AI yorum paneli beyin fırtınası ayrı bir kilometre taşıdır ve sahibin onayını bekler.** Bu belgede planlanmaz, numara almaz. Buradaki modüller o panelin okuyacağı tipleri üretir (`method` kimliğiyle); panel hesap yapmaz.
- Kalemlerin içinde kapsam dışı bırakılanlar (her biri kendi bölümünde yazılıdır): Shayanadi alt durumları, Deeptadi, KP alt lordu, Sudarshana dashası, Sankranti'nin kutlu süresi, efemerisin CSV ve baskısı, tutulmaya bağlı süreler, ülke ve kuruluş haritaları, Tajika yogaları ve tri-pataki, Sarvatobhadra'da ad harfi çıkarımı ve puanlama, üçüncül ve güneş yayı ilerletmeleri.

## 8. Riskler ve açık noktalar

- **Denetim birikimi.** Kuta ve muhurta tabloları hâlâ sahibin onayını bekliyor (İM spec §17). Bu yol haritası en az dokuz tablo daha ekler (dagdha rasi, Vimshopaka ağırlıkları ve basamakları, Lajjitadi kuralları, Shayanadi formülü, hadda, Tajika bakışları, sahamlar, Sarvatobhadra ızgarası, Abhijit yayı). Onaylanmamış tablo sayısı arttıkça "doğrulanmamış tablo" notu olağanlaşır ve anlamını yitirir. Öneri: Plan 28 ve 31 yazılmadan önce birikmiş onayların en azından kuta ve muhurta kısmı kapatılsın.
- **El örneğini yazanın kuralı yanlış anlaması** motoru da yanlış doğrular (İM spec §17). Avasthalar ve yıl lordu bu riskin en yüksek olduğu yerlerdir; ayrı gözden geçirmede yeniden türetme zorunludur.
- **Panel sayısı.** On plan yaklaşık on iki yeni panel getirir. Paletten açma (K12) ölçeklenir, ama genel bir panel kataloğu kararı (İM spec K12'de beta geri bildirimine bırakılmıştı) Plan 26 dolayında yeniden değerlendirilmelidir.
- **Tek ephemeris iş parçacığı.** Oynatılan geçiş, takvim ve efemeris tablosu motoru sık çağırır; dilimleme ve eski yanıtın düşürülmesi her birinde ayrı test ister.
- **`SkyEventKind` ve bağlama tiplerinin genişlemesi** kayıtlı aramaları ve arayüzün küme ayrıştırıcısını etkiler (Plan 30); geriye uyum testi o planın ilk görevidir.
- **Sahibin kararına bırakılanlar** (engelleyici değil; önerilen varsayılanla ilerlenir): navataranın lagnadan sayımı; Jagradadi; KP alt lordu; Sankranti çerçevesi; efemeris satır saati; yıllık haritanın yeri ve dönüş çerçevesi; ilk saham kümesi; ilerletmelerin tanımı (bu sonuncusu engelleyicidir, yalnızca Plan 32 için).

## Ek A — Adlandırma

Bu belgedeki teknik adlarının hiçbiri yasak harf dizisini içermez. Plan yazarken dikkat edilecek yerler: saham adları, Tajika yoga ve bala adları, gün dilimi adları ve Türkçe çoğul ekleri (klasik bir terim "-ka" ile bitiyorsa çoğul eki yasak diziyi üretir; terim tekil ya da "… tablosu", "… dizisi" biçiminde yazılır). Yeni bir çakışma çıkarsa İM spec Ek A'ya satır eklenir.

Bu belge için denetim: `grep -inE 'k(a|ā|aa)la|\bcala\b'` boş dönmelidir.
