# Plan 20 — Alt proje 3.7: Muhurta (zaman çizelgesi çekirdeği, gün panchangı, pencere araması; iki panel ve bulunulan yer)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Muhurta için olgu üreten üç parça: `jyotish-core`'da bir zaman aralığını boşluksuz aralık dizilerine bölen zaman çizelgesi çekirdeği (`timeline`: tithi, karana, nakshatra ve pada, nitya yoga, vara, Ay ve Güneş burcu, lagna burcu, graha burç ve nakshatra girişleri, retro ve düz dönüşler, yanma); seçilen anı içeren günün panchangı (`day_panchang`: beş uzvun gün içindeki değerleri, gündüzün sekiz dilimi ve işaretli üçü, Abhijit, hora tablosu, işaretler, tara ve chandra bala); ve astrologun kurduğu süzgece uyan zaman pencerelerinin aranması (`muhurta_search`: 366 güne dek, dilim dilim, ilerleme ve durdurma ile). Ayrıca bulunulan yer (`AppPrefs.home_place`), kayıtlı süzgeçler, `muhurta_day` ve `muhurta_search` panelleri. Etkinlik şablonu, "uygun/uygunsuz" hükmü ve açıklama metni yoktur.

**Architecture:**
- **Tek yürüyücü, çok dizi (spec §10, K3).** `timeline::walk` bir değeri sabit adımlarla örnekler ve değişim anını ikiye bölmeyle (0,05 sn) bulur; her dizi, v1 panchangının kullandığı fonksiyonlarla (`tithi_no`, `karana_no`, `nakshatra_pada`, `nitya_yoga_no`, `Sign::of`, `vara_at`) aynı ephemeris sorgularından hesaplanan bir değerin yürüyüşüdür. Bir aralığın içindeki her anda `compute_panchang` aynı değeri verir: çapraz denetim (§13 katman 4) yapıdan gelir ve gerçek ephemeris'le ayrıca sınanır.
- **Saf çekirdek, ince toplayıcılar.** `muhurta::segments` ephemeris görmez (dilim sıraları, eşit bölme, sınıflar, tara ve chandra bala): el örnekleri sentetik bir günden kurulur. `muhurta::day_panchang` ve `muhurta::muhurta_search_page` dizileri toplar. Aramada her etkin koşul bir dizi ister; pencereler dizilerin kesişimidir; en pahalı dizi (lagna) en sona bırakılır.
- **Durumsuz, imleçli arama.** `muhurta_search_page(filter, cursor)` 31 günlük tek bir dilimi hesaplar ve sonraki dilimin imlecini döner. Dilim sınırını aşan pencere bölünmez: açık pencerenin başı imleçte taşınır. Rust arama durumu tutmaz; durdurmak, sonraki dilimi istememektir. Motor iş parçacığı her dilimden sonra bırakılır.
- **İki komut, Plan 14'ün kalıbıyla (K1, K2).** `day_panchang(at, place, settings, natal)` ve `muhurta_search(query, settings, cursor)`: motor iş parçacığı, `blocking` havuzu, `CommandError`. "Şimdi" ve "bugün" Rust'ta okunur; tarihler yerin saat diliminde Rust'ta çözülür; her sonuç, saatlerin gösterileceği UTC ofsetini taşır (`atlas::offset_at`). `Chart` ve `open_client` değişmez; sonuç saklanmaz.
- **Kalıcı veri iki yerde.** `AppPrefs.home_place: Option<HomePlace>` (yer ve saat dilimi; şema sürümü değişmez) ve `app_state` altında `muhurta_filters` (ad ve koşullar; yer, tarih ve kişi taşımaz). İkisi de dışa aktarıma yazılmaz. SQLite şeması, `CalcSettings`, `ViewState` ve bütün şema sürümleri değişmez.
- **Arayüz hesap yapmaz.** Dilimler, sınıflar, işaretler, elverişli kümeler, pencereler ve ofsetler Rust'tan gelir; paneller çizer. İki panel paletten eklenir (K12). Yer seçimi (bulunulan yer / haritanın doğum yeri / özel yer), tarih bağı ve etkin süzgeç panelin ayarıdır.
- **Testler:** sahte ephemeris'le dizi testleri (analitik sınırlar), v1 panchangıyla çapraz denetim, elle yazılmış tablolar (dilim sıraları, tithi grupları, nakshatra doğaları; 27 × 27 tara ve 12 × 12 chandra bala), yedi el fikstürü (`tests/golden/modules/muhurta/`, kaynak yalnızca `hand`), proptest değişmezleri (aralıklar boşluksuz; pencerenin her anı süzgeci sağlar, pencere arası sağlamaz; dilim uzunluğu sonucu değiştirmez), gerçek motorla entegrasyon (çapraz denetim, giriş anında boylam ± 1″, dönüşte hız işareti, bütçeler), `store` testleri (bulunulan yer, kayıtlı süzgeçler, dışa aktarımda yokluk), Vitest (iki panel, pencere, kancalar), Playwright e2e.

**Tech Stack:** Rust 1.93+ (stable; yerelde ve CI'da 1.100), Tauri 2.12, tauri-specta 2.0.0-rc.29, serde, rusqlite, jiff (atlas üzerinden), proptest, React 19, TypeScript 7, Vitest 5, Playwright 1.65.1. **Yeni Rust ya da npm bağımlılığı yok.**

**Spec:** `docs/superpowers/specs/2026-11-26-alt-proje-3-ileri-moduller-design.md` (Plan 19'un geri bildirimi işlenmiş hâli): §2 (K1–K12), §4 (`ChartInput`, panchang paneli), §7 (hora kuralı), §10 (Alt proje 3.7), §12.2 (`timeline`'ı kullanacak olay araması), §13 (ortak test stratejisi), §14 (gizlilik), §15 (bütçeler), §17 (riskler), Ek A (adlandırma). UI spec: `2026-11-24-ui-design.md` §2.1, §3.2–§3.4, §4 (palet), §5, §6, §8.

**Ön koşul:** Taban, Plan 19'un son inceleme düzeltmelerini taşıyan `main` ucudur: commit `ff3da00` (Plan 19 Task 5 düzeltmeleri `b77e8d4` ve `7c1316c`, son inceleme testleri `6cdd6bf`, el örneklerinin yeniden türetilmesi `baa5198`, spec'e işlenen geri bildirim). `main` bu commit'ten ilerlediyse Task 1 adım 1 tabanları yeniden ölçer ve sayaç zinciri o ölçümden yürütülür (farklar aynıdır). Plan 19'un elle denetim kapısının yalnızca insana ait maddeleri ve kuta tablolarının basılı kaynak karşılaştırması sahibine devredilmiştir; bu planı engellemez.

## Kararlar

Sahibi kararları devretti; muhafazakâr seçenek seçildi ve her karar bir satır gerekçeyle yazıldı. "Spec boşluğu" ya da "Spec kusuru" diye işaretlenenler spec'te düzeltilmesi gereken yerlerdir ve belgenin sonundaki "Spec'e geri bildirim" bölümünde aynı numarayla toplanmıştır. D26, D27, D37 ve sayaç zinciri bu plan yazılırken deponun bir kopyasında ölçülmüştür (macOS 27.2, Rust 1.100).

- **D1 — Çekirdek iki modüldür: `crates/jyotish-core/src/timeline.rs` (aralık dizileri) ve `crates/jyotish-core/src/muhurta/` (`segments.rs` saf sınıflar ve dilimler, `day.rs` gün panchangı, `search.rs` pencere araması, `mod.rs` ortak tipler). Yöntem kimliği `MUHURTA_METHOD = "muhurta.v1"`'dir; `DayPanchang` ve `MuhurtaSearchPage` `method` taşır.** Gerekçe: spec `timeline` ve `muhurta` adlarını verir; §10'un iki sonuç tipinde `method` yoktur, K7 her modül sonucunun taşımasını ister. `timeline` ayrı durur, çünkü 3.9'un olay araması yalnızca onu kullanacaktır. **Spec kusuru 7.**
- **D2 — Bütün diziler tek yürüyücüden gelir: `walk(eph, from, to, step, value)`. Değer `step` adımlarla örneklenir; iki örnek arasında değer değişmişse değişim anı ikiye bölmeyle bulunur (`TIMELINE_RESOLUTION_DAYS` = 0,05 sn); bir adımın içinde birden çok değişim varsa hepsi sırayla bulunur. Sınır, yeni değerin geçerli olduğu ilk örnektir (gerçek sınır en çok bir çözünürlük adımı öncedir). Değer, v1 panchangının fonksiyonlarıyla aynı sorgulardan hesaplanır.** Gerekçe: spec "sınırlar v1'deki panchang bitiş aramasıyla aynı yöntemle (ikiye bölme) bulunur" der. v1'in `next_boundary`'si açının tekdüze arttığını varsayar; retro giden bir grahanın burç girişi ve lagnanın yüksek enlemdeki sıçraması bu varsayımı bozar. Ayrık değerin yürüyüşü tekdüzelik istemez, her diziye aynı kodla uygulanır ve "aralığın içinde `compute_panchang` ile aynı değer" özelliğini yapıdan verir. Adım yalnızca "değer değişip aynı değere dönmesin" diye seçilir (D3).
- **D3 — Adımlar: Ay'a bağlı diziler 0,5 gün (`MOON_STEP_DAYS`), Güneş burcu, graha burcu ve nakshatrası, hareket yönü ve yanma 1 gün (`GRAHA_STEP_DAYS`), lagna 30 dakika (`LAGNA_STEP_DAYS`), gün doğumu olmayan yerde vara 6 saat (`VARA_STEP_DAYS`). Bir adımın içinde değer değişip aynı değere dönerse (dönüş anının tam burç sınırına düşmesi; kutup dairesinin içinde lagnanın anlık sıçraması) o kısa gidiş görülmez.** Gerekçe: en kısa Ay dilimi (pada) ~5 saat sürer ve bir dizi aynı değere ancak tam bir çevrimden sonra döner (en kısası ~27 gün; lagna için 1 gün). Retro dönemleri haftalar sürer. Görülmeyen kısa gidiş bir günden kısa süren, boylamca birkaç yay saniyelik bir taşmadır; olgu olarak kaydı sonraki örnekte yine doğru değerdedir. **Spec boşluğu 23.**
- **D4 — Tithi dizisi karana dizisinden türetilir (`tithis_of_karanas`: ardışık iki karana bir tithidir). İki dizi aynı sınır anlarını bit bit paylaşır.** Gerekçe: her tithi sınırı bir karana sınırıdır; ayrı ikiye bölme aynı anı çözünürlük kadar farklı bulur ve gün panchangında tithi ile karana satırları birbirini tutmaz görünürdü. Hesap da yarıya iner.
- **D5 — Graha dizileri: `graha_sign_series` (rasi çerçevesinde burç; Ay ve Güneş burcu dizileri de budur), `graha_nakshatra_series` (sidereal nakshatra), `graha_motion_series` (`Motion { Direct, Retrograde }`; sınırlar dönüş anlarıdır) ve `combust_series` (v1 yanma kuralı). Olaylar ayrı bir tip değildir: bir giriş ya da dönüş, dizideki iki aralığın sınırıdır. Hareket yönü dizisi yalnızca retro olabilen beş graha için verilir (`STATION_GRAHAS`: Mars, Merkür, Jüpiter, Venüs, Satürn); diğerlerinde `InvalidMuhurtaInput("graha")`. Ketu, Rahu'nun 180° karşısıdır; düğüm türü ayardan gelir.** Gerekçe: spec "graha burç girişleri, nakshatra girişleri, retro ve düz dönüş anları" der; hangi grahalar için olduğunu yazmaz. Güneş ve Ay hiç geri gitmez; ortalama düğüm hep geri gider; gerçek düğüm birkaç günde bir yön değiştirir ve bu dönüşler klasik anlamda olay değildir (arayüz de düğümlerde retro göstermez). **Spec boşluğu 8.**
- **D6 — Vara dizisi `compute_panchang`'ın kuralıdır: vara gün doğumunda değişir; gün doğumu yoksa gün 06:00 yerel ortalama saatte başlar. `sun_days` aralıktaki doğuşu, batışı ve sonraki doğuşu bilinen günleri listeler (`SunDay`); günler aralığı boşluksuz örtüyorsa vara dizisi doğuş anlarından kurulur (hızlı yol), örtmüyorsa (ardışık iki doğuş arası 1,25 günden uzun ya da arada batış yok) v1'in `vara_at` fonksiyonu yürünür (yavaş yol). İki yol her anda aynı değeri verir.** Gerekçe: spec "vara (gün doğumu → gün doğumu)" der; kutup gecesi ve gündüzünde ne olacağını yazmaz. v1'in kuralını yeniden yazmak yerine aynı fonksiyonu yürümek, iki tanımın ayrışmasını olanaksız kılar; hızlı yol olağan enlemlerde yüzlerce doğuş aramasını tasarruf eder. **Spec boşluğu 9.**
- **D7 — Bir dizi `[from, to]` aralığına kırpılır: ilk aralık `from`'da başlar, sonuncusu `to`'da biter. Gün panchangında günün dışına taşan uç, günün başı ya da sonudur; arayüz bunu bir okla ve ekran okuyucu metniyle gösterir ("günün başından önce başlar", "günün sonundan sonra sürer").** Gerekçe: spec "boşluksuz ardışık aralıklar" der ve kırpmayı yazmaz. Kırpılmış dizi günü tam böler (property); bir uzvun gerçek bitişini isteyen, transit tarihini ertesi güne alır. **Spec boşluğu 10.**
- **D8 — Gün, verilen anı içeren gündür (gün doğumundan sonraki gün doğumuna): `day_panchang(eph, jd_ut, lat, lon, settings, natal)`. Komut bir takvim tarihi ve saat dilimi almaz; transit anını (`TransitAt`: "şimdi", duvar saati ya da JD) alır. Panelin "bugün" bağı "şimdi"yi gönderir. Gün doğumundan önceki an bir önceki güne aittir (vara ile aynı kural).** Gerekçe: spec `day_panchang(date, place, tz, …)` yazar. Bir takvim günü ve saat dilimi hangi gün doğumunun kastedildiğini tek başına söylemez (yüksek enlemde gün doğumu öğleden sonraya düşebilir; yaz saati geçişinde gece yarısı olmayabilir); panelin ayarı ise bir andır ("transit tarihi / bugün") ve "bugün" Rust'ta okunmalıdır (C4). v1'in `compute_panchang`'ı günü zaten andan seçer; aynı kural kullanıldı. **Spec kusuru 3.**
- **D9 — `DayPanchang`'ın gün doğumu alanları `Option`'dır ve tip iki alan kazanır: `day_start_jd_ut`, `day_end_jd_ut`. Doğuşu, batışı ve sonraki doğuşu bilinen günde üçü de doludur ve `segments` vardır. Değilse (kutup gecesi ya da gündüzü) üçü de boştur, `segments` `None`'dır, gün 06:00 yerel ortalama saatte başlar ve 24 saat sürer, `warnings` `SunriseUnavailable` taşır; uzuvlar, işaretler ve kişiye bağlı etkenler yine verilir.** Gerekçe: spec'in tipi `sunrise_jd_ut: f64`, `sunset_jd_ut: f64`, `next_sunrise_jd_ut: f64` ve `segments: DaySegments` yazar; aynı bölüm "kutup günü ya da gecesinde gündüz dilimleri hesaplanmaz; panel uyarı rozetiyle yalnızca uzuvları gösterir" der. Olmayan bir gün doğumu sayı olarak dönemez. **Spec kusuru 1.**
- **D10 — `DaySegments` sekiz dilimin tamamını taşır (`eighths: Vec<Interval<u8>>`) ve işaretli dilimler dilimin sırasını taşır (`rahu`, `yamaganda`, `gulika`: `Interval<u8>`, değer 1..=8; `abhijit`: değer 8, muhurtanın sırası). Dilimler gerçek gün doğumu ile batımı arasının eşit parçalarıdır (`equal_parts`; son parça tam bitişte biter).** Gerekçe: spec `rahu: Interval<()>` yazar ve sekiz dilimi tipe koymaz; panel ise "sekiz dilim ve işaretli üçü"nü çizer ve dilim sınırlarını kendisi hesaplayamaz (C4). `()` dilimin kaçıncı olduğunu da söylemez. **Spec kusuru 6.**
- **D11 — Dilim sıraları (Pazar'dan Cumartesi'ye): Rahu dilimi 8, 2, 7, 5, 6, 4, 3 (tablo); Yamaganda 5, 4, 3, 2, 1, 7, 6 (formül: `(4 − gün) mod 7 + 1`); Gulika 7, 6, 5, 4, 3, 2, 1 (formül: `7 − gün`). Abhijit gündüzün on beş eşit muhurtasının sekizincisidir; Çarşamba günü `abhijit_weekday_exception: true`'dur ve dilim yine verilir. Hora: gün doğumundan sonraki gün doğumuna 24 eşit dilim; ilk lord vara lordudur, sonrakiler Plan 17'nin `hora_lord` fonksiyonuyla. Bu tablolar spec'te yazılıdır, muhurta derlemelerinin yaygın biçimidir ve bellekten aktarılmıştır: basılı bir kaynakla karşılaştırılması sahibin onay maddesidir (Task 9 Step 6) ve bilinen varyant noktaları `tests/golden/modules/muhurta/REVIEW.md`'de sayılıdır.** Gerekçe: spec §10'un listeleri; §13 katman 3 formül ve elle yazılmış tablonun iki ayrı yazım olmasını ister (testte `HAND_RAHU_DAY_OF_SEGMENT`, `HAND_YAMAGANDA`, `HAND_GULIKA`). Yürüten ajanın elinde basılı kaynak yoktur; başka yazılımın çıktısı kaynak olamaz (K9).
- **D12 — Sınıflar değerin içindedir: `DayTithi { no, paksha, group }`, `DayNakshatra { no, pada, nature }`, `DayKarana { no, name }`; nitya yoga sıradır (`u8`). `TithiGroup { Nanda, Bhadra, Jaya, Rikta, Purna }` paksha içi sıranın beşli döngüsüdür (Purnima ve Amavasya Purna'dır); `NakshatraNature { Fixed, Movable, Fierce, Mixed, Swift, Soft, Sharp }` spec'in yedi listesidir (`NAKSHATRA_NATURES`).** Gerekçe: spec `tithis: Vec<Interval<u8>>` ve `nakshatras: Vec<Interval<(u8, u8)>>` yazar; aynı bölüm tithi grubunu ve nakshatra doğasını "olgu" sayar ve panelin göstermesini bekler. Sınıf bir astrolojik eşlemedir: arayüzde tablo tutulmaz (C4). Demet yerine adlı alanlar TS bağlamasını okunur kılar. **Spec boşluğu 11.**
- **D13 — İşaretler tek bir enum'dur: `MuhurtaFlag { Rikta, Vishti, Vyatipata, Vaidhriti, RahuSegment, Yamaganda, Gulika, Panchaka, JupiterCombust, VenusCombust }` (10). Gün panchangının `flags` listesi yedisini aralık olarak verir (`DAY_FLAG_KINDS`: üç gündüz dilimi `segments` içindedir); aramanın `exclude` listesi onunu da alır. Panchaka, Ay'ın nakshatra ve padasından okunur (Dhanishta'nın üçüncü padasından Revati'nin sonuna: sidereal 300°–360°); Vishti karananın adından; Rikta tithi grubundan; yanma v1'in `is_combust` kuralından ve haritanın orblarından.** Gerekçe: spec işaretleri iki ayrı listede sayar ("Vyatipata ve Vaidhriti", "yanık Jüpiter/Venüs" tek kalem olarak) ve `MuhurtaFlag`'i tanımlamaz. İkili kalemler ayrı değerlerdir: süzgeç birini dışlayıp diğerini bırakabilmelidir. **Spec boşluğu 12.**
- **D14 — Kişiye bağlı etkenler aralıktır: `PersonalFactors { tara: Vec<Interval<TaraBala>>, chandra: Vec<Interval<ChandraBala>> }`; `TaraBala { number, favourable }`, `ChandraBala { house, favourable }`. Tara numarası Plan 19'un `kuta::tara_number` fonksiyonudur (sayım başlangıcı da sayar); elverişli küme {2, 4, 6, 8, 9}'dur (Janma elverişli sayılmaz). Chandra bala geçiş Ay'ının rasi burcunun doğum Ay burcundan sayısıdır; elverişli küme {1, 3, 6, 7, 10, 11}. Girdi `NatalMoon { nakshatra, sign }`'dir; nakshatra 1..=27 değilse `InvalidMuhurtaInput("natal")`.** Gerekçe: spec `personal: Option<PersonalFactors>` yazar ve tipi tanımlamaz; Ay gün içinde nakshatra ya da burç değiştirir (Ay bir nakshatrada ~1 gün durur), tek bir değer günün yarısında yanlış olurdu. "Elverişli" spec'in adlandırdığı kümenin adıdır, hüküm değildir. **Spec boşluğu 12.**
- **D15 — Aramanın girdisi ikiye ayrılır: `MuhurtaCriteria` (saklanabilen koşullar: `weekdays`, `tithis`, `nakshatras`, `lagna_signs`, `daytime_only`, `min_minutes`, `exclude`, `require_tara`, `require_chandra`) ve `MuhurtaFilter { from_jd_ut, to_jd_ut, lat, lon, criteria, natal }`. Boş küme kısıt değildir. `tithis` 1..=30'dur (paksha ayrımıyla). `MuhurtaCriteria::normalized` kümeleri sıralar ve yinelenenleri atar.** Gerekçe: spec tek bir `MuhurtaFilter` yazar: `place: Place`, tarih aralığı ve `natal` koşullarla aynı tiptedir; aynı bölüm kayıtlı süzgecin "kişi verisi içermediğini" söyler. Yer (bulunulan yer dahil) kişisel veridir ve `Place` bir `store` tipidir (çekirdek `store`'a bağımlı olamaz). Koşulları ayırmak, saklanan şeyin tipini "kişisiz ve yersiz" kılar. **Spec kusuru 5.**
- **D16 — Arama durumsuzdur ve imleçle sayfalanır: `muhurta_search_page(eph, settings, filter, cursor) -> MuhurtaSearchPage { windows, scanned, chunks, next, warnings, method }`. Bir çağrı `MUHURTA_CHUNK_DAYS` = 31 günlük tek bir dilimi hesaplar; `next: Option<MuhurtaCursor { chunk, open_start_jd_ut }>` sonraki çağrıya olduğu gibi verilir. `muhurta_cancel` komutu ve `search_id` yoktur: durdurmak, sonraki dilimi istememektir; eski aramanın geç gelen sayfasını arayüz kendi sayacıyla düşürür. Aynı süzgeç ve imleç her zaman aynı sayfayı verir; geçersiz imleç `InvalidMuhurtaInput("cursor")`'dur.** Gerekçe: spec `muhurta_search(filter, search_id) -> MuhurtaSearchPage` ("ay ay sayfalanır") ve `muhurta_cancel(search_id)` yazar; imzada sayfanın yeri yoktur ve Rust'ta arama durumu tutmak (kilit, sızan durum, eski sayfaların atılması) yalnızca arayüzün "bir sonrakini isteme" ile yapabildiği işi yapar. Tek bir dilim en çok 37 ms sürer (D26): yoldaki dilimi kesmeye gerek yoktur. Dilimler takvim ayı değil 31 gündür: takvim ayı saat dilimi ister ve bir şey kazandırmaz; arayüz "taranan bölüm n / N" der. **Spec kusuru 4.**
- **D17 — Dilim sınırını aşan pencere bölünmez: bir dilimin sonuna değen aralık verilmez, başı imleçte taşınır (`open_start_jd_ut`) ve pencere kapandığı dilimde, tam uzunluğuyla verilir. "En kısa süre" birleşmiş pencereye uygulanır. 1 saniyeden kısa pencere hiç verilmez (`MUHURTA_MIN_WINDOW_DAYS`).** Gerekçe: spec sayfalamayı yazar, sınırı aşan pencereyi yazmaz; bölünmüş bir pencere "en kısa süre" koşulundan yanlışlıkla düşerdi. 1 saniye kuralı bir ölçüm bulgusudur: iki ayrı dizinin aynı ana düşen sınırları (Ay'ın burç girişi ile nakshatra girişi) ayrı ikiye bölmelerle bulunur ve çözünürlük kadar ayrışır; biri kapanırken diğeri açılan iki koşul arasında 0,05 saniyelik bir "pencere" oluşuyordu ve dilim uzunluğuna göre bir görünüp bir yok oluyordu (property testi buldu). **Spec boşluğu 13.**
- **D18 — Doğuşu ya da batışı olmayan günlerde arama: "yalnızca gündüz" o günlerde pencere vermez; üç gündüz dilimi dışlaması o günlerde uygulanmaz (dilim yoktur); iki durumda da sayfa `SunriseUnavailable` uyarısı taşır. Vara koşulu D6'nın kuralıyla çalışır ve uyarı vermez.** Gerekçe: spec aramanın kutup davranışını yazmaz. Gündüz olduğu bilinmeyen bir anı "gündüz" saymak ya da olmayan bir dilimi dışlamış görünmek yanlış olgu üretirdi; uyarı astrologa süzgecin o günlerde eksik uygulandığını söyler. **Spec boşluğu 14.**
- **D19 — `WindowSnapshot { vara, tithi, paksha, nakshatra, pada, yoga, karana, karana_name, lagna_sign }` pencerenin başlangıç anının noktasal hesabıdır (`window_snapshot`); dizilerden okunmaz.** Gerekçe: spec "her biri o andaki beş uzuv ve lagna burcuyla" der ve tipi tanımlamaz. Özet için bütün dizileri (özellikle lagnayı) hesaplamak, süzgeçte olmayan koşulların da bedelini ödetirdi; pencere başına birkaç ephemeris sorgusu yeter. **Spec boşluğu 12.**
- **D20 — Yeni tek hata türü `CoreError::InvalidMuhurtaInput(&'static str)` → `invalid_muhurta_input { field }`'dir (alanlar: `range`, `place`, `natal`, `graha`, `tithis`, `nakshatras`, `min_minutes`, `cursor`; komut katmanında ayrıca `from_date`, `to_date`). Muhurta fonksiyonları konumu kendileri denetler ve yalnızca alan adını döner; v1'in `invalid_location { lat, lon }` hatası bu yollarda üretilmez. Komut, saat dilimi çözülemeyen yeri de `place` alanıyla reddeder (`unknown_time_zone { tz }` üretilmez).** Gerekçe: spec §14 ve §10 "`home_place` hata parametresine girmez" der; v1'in konum hatası enlem ve boylamı, saat dilimi hatası dilimin adını parametreye yazar. Tek bir kod üç dilde tek bir ileti demektir (C12). **Spec boşluğu 22.**
- **D21 — Bulunulan yer `AppPrefs.home_place: Option<HomePlace>`'tir; `HomePlace { place: Place, tz: TzChoice }`. `APP_PREFS_SCHEMA_VERSION` 1'dir ve değişmez: alanı taşımayan eski kayıt yersiz okunur (Plan 16 ve 19'un `ViewState` kalıbı). Okuma toleranslıdır (okunamayan ya da geçersiz yer "seçilmemiş"tir; uyarı vermez); yazım denetimi koordinat aralığını, ad uzunluğunu ve saat dilimini (bilinen IANA kimliği ya da ± 18 saat içinde sabit ofset) denetler ve `invalid_input` alan `home_place` döner.** Gerekçe: spec `home_place: Option<Place>` yazar; `Place` saat dilimi taşımaz, oysa o yerin saatleri gösterilecektir ve doğum verisinde yer ile dilim zaten ayrı alanlardır. **Spec kusuru 2.**
- **D22 — Kayıtlı süzgeçler `app_state`'in `muhurta_filters` anahtarındadır: `{"schema_version": 1, "filters": [{"name", "criteria"}]}`. `Store::muhurta_filters()` toleranslı okur (okunamayan kayıt boş liste; eksik koşul alanı varsayılan; bozuk, adsız ve yinelenen öğe atılır); `Store::save_muhurta_filters(list)` listenin tamamını yazar ve kanonik listeyi döner. En çok 32 süzgeç; ad kırpılır, boş olamaz, 200 karakteri aşamaz ve büyük/küçük harf ayırmadan tektir. Daha yeni sürümün kaydı boş okunur ve üzerine yazılmaz.** Gerekçe: spec anahtarı ve "kişi verisi içermez" kuralını verir; biçimi, sınırları ve API'yi vermez. Listenin tamamını yazmak pano düzeninin kalıbıdır (`save_panel_layout`). **Spec boşluğu 15.**
- **D23 — İki komut: `day_panchang(at: TransitAt, place: MuhurtaPlace, settings, natal) -> DayPanchangView { day, offset_secs }` ve `muhurta_search(query: MuhurtaQuery, settings, cursor) -> MuhurtaSearchView`. `MuhurtaPlace { lat, lon, tz }` ad taşımaz. `MuhurtaQuery { from_date, to_date, place, criteria, natal }`: tarihler yerin takvim günleridir (`YYYY-MM-DD`, ikisi de dahil); `from_date` boşsa bugün, `to_date` boşsa başlangıçtan 30 gün sonrası (31 gün: tek dilim). Aralık ilk çağrıda çözülür ve imleçte taşınır (`MuhurtaPageCursor { from_jd_ut, to_jd_ut, cursor }`): gece yarısını aşan bir arama aralığını değiştirmez. Günün başı yerin 00:00'ıdır; 00:00 yaz saati boşluğuna düşerse öğleden 12 saat öncesi alınır. Ayrıca iki store komutu: `load_muhurta_filters`, `save_muhurta_filters`.** Gerekçe: spec komutları adlandırır; tarihin nasıl verileceğini, "bugün"ün nerede okunacağını ve aralığın sayfalar boyunca nasıl sabit duracağını yazmaz. Arayüz saat okumaz (C4). **Spec boşluğu 16.**
- **D24 — Saatler seçilen yerin saatiyle gösterilir: `DayPanchangView.offset_secs` günün başındaki, `MuhurtaWindowView.start_offset_secs` ve `end_offset_secs` pencerenin iki ucundaki UTC ofsetidir; `atlas::offset_at(tz, lon, utc)` çözer (`resolve_moment` ile aynı kurallar). Gün içinde yaz saati değişirse gün panchangının saatleri o gün bir saat kayık gösterilir (yılda iki gün).** Gerekçe: spec saatlerin hangi dilimde gösterileceğini yazmaz. v1 paneli doğumun çözülmüş ofsetini kullanır (`formatJdMinute`); aynı yol izlendi. Tarayıcının dilim verisi atlasın gömülü tzdb'siyle aynı olmayabilir; ofset Rust'ta çözülür. **Spec boşluğu 17.**
- **D25 — Gizlilik: yer, sorgu ve doğum Ay'ı loglanmaz; komuta yerin adı gitmez; `home_place` ve kayıtlı süzgeçler JSON ve CSV dışa aktarımda yoktur (test sabitler) ve içe aktarma tercihlere dokunmaz. Özel yer (D29) panelin ayarında, yani kullanıcının pano düzeninde durur; düzen de dışa aktarılmaz.** Gerekçe: spec §10 ve §14.
- **D26 — Bütçeler entegrasyon testinde ölçülür (hata ayıklama derlemesi): `day_panchang` ≤ 50 ms; 366 günlük arama lagna koşulu olmadan ≤ 2 sn, lagna koşuluyla ≤ 5 sn; ilk sayfa ≤ 500 ms; her sayfa ≤ 300 ms (motor iş parçacığı dilim aralarında bırakılır: araya giren harita hesabı en çok bir dilim bekler).** Gerekçe: ölçüm: `day_panchang` ortanca 0,55 ms; diğer bütün koşullar etkinken 366 gün 0,20 sn (12 dilim, en yavaşı 18 ms); lagna koşulu eklenince 0,42 sn (en yavaş dilim 37 ms); yalnızca lagna koşuluyla 0,25 sn. Bütçelerin onda birinden azdır; 31 günlük dilim 300 ms sınırının sekizde biridir.
- **D27 — Kanca ve yenileme: `useDayPanchang(opened, place, at, epoch, scope)` sonucu `useModule` ile ister ve yeni gün okunurken aynı panelin son gününü göstermeyi sürdürür (`useLastData`): içerik ve odak yerinde durur, panel `aria-busy` olur. `Transit.epoch` eklenir ("Şimdi" her seçildiğinde ve her yeniden denemede artar): an "şimdi" olarak dursa da gün yeniden okunur. `useMuhurtaSearch(scope)` dilimleri sırayla ister, pencereleri biriktirir; `stop()` sonraki dilimi istemez ve gelmiş pencereler durur; başka harita açılınca arama bırakılır.** Gerekçe: Plan 16 ve Plan 19 Task 5'in dersi: yeniden okuma sırasında listenin DOM'dan kalkması odaktaki denetimi götürür. `useModule` anahtar değişince veriyi boşaltır; `useLastData` bunu panelin isteğine bırakır.
- **D28 — Panel türleri `muhurta_day` (7 × 18) ve `muhurta_search` (8 × 18); `MODULE_PANELS`'e girer ve paletten eklenip çıkarılır (K12). Paneller yalnızca harita açıkken panoda görünür; doğum Ay'ı her zaman açık haritanınkidir (kişi seçimi yoktur). Gün panelinin ayarı: `place` (`home` | `chart` | `custom`; varsayılan `home`), `link` (`transit` | `today`; varsayılan `transit`), `custom` (özel yer). Arama panelinin ayarı: `place`, `custom`, `criteria` (etkin süzgeç). Tarih aralığı ayar değildir (oturumluk).** Gerekçe: spec panelleri ve ayarlarını sayar. K12 modül panellerini harita açıkken sunar: "harita açıksa tara ve chandra bala" koşulu bu yüzden her zaman sağlanır; haritasız gün panchangı bu planda yoktur. **Spec boşluğu 18.**
- **D29 — Yer seçimi (`MuhurtaPlace.tsx`): "Bulunulan yer" (tercihlerden; seçilmemişse panel yer ister ve hiçbir komut çağırmaz), "Haritanın doğum yeri" (doğum verisinin yeri ve saat dilimi) ve "Özel yer" (atlastan aranır; panelin ayarında saklanır). Bulunulan yer bir pencereden değiştirilir (`HomePlaceDialog`, `shell/Dialog`): atlas araması, "açık haritanın doğum yerini kullan" ve "yeri kaldır". Bulunulan yer seçilmemişken doğum yerine sessizce düşülmez. Koordinatı elle yazmak için ayrı bir form yoktur: elle girilmiş koordinat, o koordinatla kurulmuş bir haritanın doğum yeri olarak alınır.** Gerekçe: spec "boşsa açık haritanın doğum yeri önerilir ve seçim istenir" ve "atlas'tan seçilir ya da elle girilir" der. Doğum formu elle koordinat ve saat dilimi girişini zaten çözer (yaz saati, LMT, sabit ofset); aynı alanları ikinci bir formda yinelemek yerine o yoldan gelen yer kullanılır. **Spec boşluğu 19.**
- **D30 — Gün paneli: araç çubuğu (yer, tarih bağı; transit bağında `TransitDate` seçicisi); başlık ("2026-11-25 · Budhavara · Çarşamba"); gün doğumu, batımı ve sonraki gün doğumu; şerit (sekiz dilimli sıralı liste; işaretli dilim tarama deseni ve kısa yazıyla ayrılır: "Rahu", "Yama", "Guli"; altında Abhijit çubuğu "Abhi" ve saatleri metin olarak); beş uzvun tablosu (uzuv, değer ve sınıf, başlangıç, bitiş); işaretler listesi; açık haritanın Ay'ına göre tara ve chandra bala tablosu; hora tablosu (lord ortak vurgu düğmesidir); okuma notu ve yöntem kimliği. Saat "SS:DD"dir; günün başladığı takvim gününden sonraki saat "(+1)" taşır. Transit seçicisinin duvar saati tarayıcının dilimindedir (v1 davranışı); gün, o anı içeren ve seçilen yerde yaşanan gündür.** Gerekçe: spec panelin öğelerini sayar; sütunları, saat biçimini ve şeridin erişilebilir biçimini vermez (UI spec §1: renk tek başına anlam taşımaz). **Spec boşluğu 20.**
- **D31 — Arama paneli: kayıtlı süzgeç satırı (seçici, ad, "Süzgeci kaydet", "Süzgeci sil"); yer; başlangıç ve bitiş tarihi (boş bırakılabilir; D23); beş onay kutusu kümesi (günler, tithiler, nakshatralar, lagna burçları, dışlananlar; uzun üçü açılır kapanır bölümdedir ve başlığı seçili sayıyı söyler); "yalnızca gündüz", "en kısa süre (dk)", "tara bala elverişli", "chandra bala elverişli"; "Ara". Sonuç: özet satırı ("30 pencere · 2026-11-26 – 2027-01-31"), başlangıç, bitiş, süre ve altı uzuv sütunlu tablo. Satırın başlangıcı bir düğmedir: transit tarihini o ana götürür (`setAt({ Jd })`). En çok 500 satır çizilir; fazlası sayılır ve süzgecin daraltılması söylenir.** Gerekçe: spec formu ve listeyi sayar; alanların denetimlerini, kümelerin gösterimini ve uzun sonucun sınırını vermez. 366 günlük gevşek bir süzgeç binlerce pencere verebilir. **Spec boşluğu 21.**
- **D32 — Durumlar ve odak: arama 100 ms'yi aşınca panel başlığında satır içi gösterge; 1 saniyeyi aşınca ilerleme satırı (`role="status"`) ve "Durdur". "Durdur" kalkınca (arama bitti ya da durduruldu) odak ondaysa "Ara"ya taşınır. Sonuç tablosu ilk pencereyle çizilir ve arama boyunca DOM'da durur: odaktaki satır odağını yitirmez. "Süzgeci sil" seçim boşalınca kullanılamaz olur: odak seçiciye taşınır. Bulunulan yer penceresinde "Yeri kaldır" kalkınca odak "doğum yerini kullan" düğmesine taşınır. Kullanılamayan düğmeler yerel `disabled` değildir (`aria-disabled`; odaklanır ve nedeni görünür bir satırdır: Plan 11 D22). Hata panelin içindedir; gelmiş pencereler durur; "Yeniden dene" aramayı yeniden başlatır.** Gerekçe: spec §10 "Arayüz" ve UI spec §5; Plan 16 ve Plan 19 Task 5'in odak dersleri.
- **D33 — Doğum saati bilinmiyorsa günün panchangı değişmez (lagnaya ve doğum saatine bağlı değildir); doğum Ay'ı öğle konumundadır: gün panelinde kişiye bağlı tablo, arama panelinde iki kişi koşulu "≈" ve açıklamasıyla işaretlenir (`Approx`). Rust'ta `None` dönen bir alan yoktur.** Gerekçe: K2 "lagnaya bağlı her çıktı `None`" der; muhurtanın lagnası geçiş lagnasıdır, doğum lagnası hiç kullanılmaz. §13 her panel için "saat bilinmiyor" durumunu ister: o durum burada yaklaşık doğum Ay'ıdır. **Spec boşluğu 24.**
- **D34 — Görünen adlar: klasik adlar üç dilde aynı yazılır (K11): tithi grupları (enum adı), nakshatra doğaları (`NAKSHATRA_NATURE_NAMES`: Dhruva, Chara, Ugra, Mishra, Kshipra, Mridu, Tikshna; yanında çevrilen kısa ek), taralar (`TARA_NAMES`), karana adları (enum adı), "Tara bala", "Chandra bala", "Vyatipata", "Vaidhriti", "Yamaganda", "Gulika", "Panchaka" ve şeridin kısa yazıları. Çevrilenler: "Rahu dilimi", "Rikta tithi", "Vishti karana", "Jüpiter yanık", "Venüs yanık" ve doğa ekleri. `ui_fixture.rs` Rust'ın `MuhurtaFlag::ALL`, `TithiGroup::ALL`, `NakshatraNature::ALL` listelerini ve `MUHURTA_MAX_MIN_MINUTES`'ı fikstüre yazar (`muhurta_ids`); Vitest her kimliğin üç dilde adı olduğunu oradan denetler.** Gerekçe: kimlikler Rust'ta, adlar TS'tedir (Plan 18 D17 ve Plan 19 D28'in yolu).
- **D35 — Adlandırma (Ek A'nın uzantısı): gündüzün üç işaretli diliminin klasik adlarında ikinci sözcük yasak diziyi içerir; bu planda dilimler "Rahu dilimi" (Ek A; kod `rahu`, `RahuSegment`, `rahu_segment`), "Yamaganda" (`yamaganda`, `Yamaganda`) ve "Gulika" (`gulika`, `Gulika`) diye tek sözcükle anılır. Hora tablosunun, tithi ve nakshatra temelli "zehir" ve "nektar" sürelerinin, choghadiya dilimlerinin ve günün otuz muhurtasının yasak diziyi içeren klasik adları hiçbir yere yazılmaz (bu sürelerin kendileri de kapsam dışıdır; hora tablosu "Hora tablosu"dur). "Panchaka", "Gulika" ve "dakika" sözcüklerine `-lar` eki ya da `L` ile başlayan bir ad bitişik yazılınca (camelCase'te de) yasak dizi oluşur; böyle birleşimler kurulmaz ("panchaka dönemleri", "dakika", `flagLabel`).** Gerekçe: v1 spec §10 ve Ek A; yasak harf dizisi düzeyindedir.
- **D36 — Arayüz fikstürü `muhurta` ve `muhurta_ids` kazanır: açık haritanın doğum yeri ve Ay'ı, aynı yerin `HomePlace` biçimi, fikstürün sabit saatini içeren gün (`day`: 2026-11-25, Çarşamba, Delhi), kutup gecesinde bir gün (`day_polar`: Tromsø, 2050-12-21) ve 67 günlük bir aramanın üç dilimi (`query`, `pages`: Perşembe gündüzleri, üç işaretli dilim dışlanmış, en az 30 dakika; 30 pencere). E2e taklidi günü ve sayfaları fikstürden, tercihleri ve kayıtlı süzgeçleri bellekten verir.** Gerekçe: arayüz hesap yapmaz; testler gerçek motorun ürettiği sonucu çizer.
- **D37 — Yeni görsel temel (PNG) ve yeni baskı düzeni yoktur (K10). `pnpm build` Vite'ın 500 kB parça uyarısını basmayı sürdürür (ölçüm: 608,6 kB) ve kapılar `✓ built` satırını arar (Plan 15 D22).** Gerekçe: on görsel temel modül paneli taşımaz (ölçüm: `e2e:visual` 10 geçti, temel dosyalar değişmedi).

## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni bağımlılık yok: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `app/package.json`, `app/pnpm-lock.yaml` ve `app/vite.config.ts` değişmez (`jiff` `atlas`'ın, `atlas` ve `jyotish-core` `store`'un mevcut bağımlılıklarıdır); `cargo deny check` Task 9'da `advisories ok, bans ok, licenses ok, sources ok` verir.
- **Yasaklı ad kuralı (v1 spec §10 "İsim kuralları", ilk madde; ileri modüller spec'i Ek A):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Kural harf dizisi düzeyindedir: Türkçe sözcüklerin içinde de geçemez (yerine ör. "diğer", "ötekiler", "durur", "geri", "sürer", "tutar", "bulur", "çıkar" yazılır). **Bu planda özel dikkat (D35):** muhurta derlemelerinde gündüz dilimlerinin, hora tablosunun ve bazı sürelerin klasik adları yasak diziyi içerir; bu planda yalnızca "Rahu dilimi", "Yamaganda", "Gulika" ve "Hora tablosu" adları kullanılır ve diğerleri hiçbir yere yazılmaz. `-ka` ile biten bir sözcüğe (`Panchaka`, `Gulika`, `karaka`, "dakika") `-lar` çoğul eki ya da `L` ile başlayan bir ad bitişik yazılınca (camelCase'te de) yasak dizi oluşur; böyle birleşimler kurulmaz. Her commit adımı eklenen satırları, dosya adlarını ve commit mesajını `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile tarar; çıktı boş olmalıdır (C13). Bu belgede de eşleşme yoktur.
- **Clean-room ve yorum metni yok:** kurallar ve tablolar klasik tanımdan özgün olarak uygulanır; başka ürünün adı, düzeni, metni ya da çıktısı kullanılmaz; kaynak gösterimi geneldir ("muhurta derlemeleri") ve alıntı yapılmaz. Paneller hesaplanmış olguları ve klasik etiketleri gösterir (uzuvlar, dilimler, sınıflar, işaretler, tara ve chandra bala, pencereler). **Hazır etkinlik şablonu, "uygun/uygunsuz/uğurlu" hükmü, renkli derecelendirme ve bir dilimin ya da sınıfın ne anlama geldiğini anlatan metin eklenmez.** "Elverişli", spec'in adlandırdığı iki kümenin (tara {2, 4, 6, 8, 9}; chandra bala {1, 3, 6, 7, 10, 11}) adıdır ve yalnızca o bağlamda geçer. Süzgeci kuran astrologdur.
- **Çerçeveler (K5):** Ay burcu, Güneş burcu, lagna burcu, graha burcu ve chandra bala rasi çerçevesindedir (`settings.zodiac`'ı izler); tithi, karana, nakshatra, pada, nitya yoga, panchaka ve tara bala her kipte sidereal boylamdandır (v1 panchangıyla aynı). Yanma v1'in kuralı ve haritanın orblarıdır. Vara gün doğumunda değişir; gün doğumu tanımı `settings.sunrise_def`'tir.
- **Doğum saati bilinmiyorsa (K2, D33):** muhurta doğum lagnasını kullanmaz; hiçbir alan `None` olmaz. Doğum Ay'ı öğle konumundadır ve arayüz kişiye bağlı değerleri "≈" ile işaretler.
- **Yer ve kişi verisi (spec §14, §10 "Gizlilik"; D20, D25):** bulunulan yer kişisel veridir. (1) Yer, sorgu, doğum Ay'ı ve kayıtlı süzgeç adları hiçbir yerde loglanmaz. (2) Muhurta yollarının hataları yalnızca alan adı taşır (`invalid_muhurta_input { field }`); enlem, boylam, saat dilimi ve tarih hata parametresine girmez. (3) Komutlara yerin adı gönderilmez (`MuhurtaPlace { lat, lon, tz }`). (4) `home_place` ve `muhurta_filters` dışa aktarıma yazılmaz. (5) Kayıtlı süzgeç yer, tarih ve kişi taşımaz. (6) Ağ isteği yoktur; konum servisi kullanılmaz. (7) Sonuçlar veritabanına yazılmaz, oturumda saklanmaz. (8) Elle denetimde yalnızca deneme danışanları kullanılır.
- **El örnekleri (§13, K9):** `source` yalnızca `hand`'dir; beklenen değer kuraldan elle türetilir ve `derivation`'da adım adım yazılır. Bir el örneği motorla uyuşmazsa önce kural yeniden okunur; **beklenen değer motor çıktısına bakarak düzeltilmez.** Gözden geçirme döngüsünün zorunlu maddesi: motoru yazan dışında bir gözden geçiren yedi örneği kuraldan yeniden hesaplar ve `REVIEW.md`'deki tabloya adını yazar (Task 9 Step 5). **Bellekten aktarılan klasik tablolar** (dilim sıraları, tithi grupları, nakshatra doğaları, elverişli kümeler) sahibin onayını bekler: bilinen varyant noktalarıyla `REVIEW.md`'de sayılıdır (Task 9 Step 6); yürüten ajan tabloları değiştirmez ve başka bir yazılımın çıktısıyla ya da çevrimiçi bir tabloyla karşılaştırmaz.
- Kapılar: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (1.100), `cargo test --workspace`, `pnpm --dir app test`, `pnpm --dir app build` her görev sonunda yeşil; `pnpm --dir app e2e` Task 8 ve 9'da. `tsc` `strict`. `pnpm build` çıktısında aranan satır `✓ built`'tir (D37). Tek bir Vitest dosyası `pnpm --dir app exec vitest run ui/…` ile koşulur: yol `app/` dizinine göredir.
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez). Node 24+, pnpm 11.8.0. Depo kısa bir yolda durmalıdır: Swiss Ephemeris 242 bayttan uzun ephemeris yolunu reddeder (çalışma ağacı uzun bir yoldaysa `/tmp` altında kısa bir yola klonlanır).
- **Çalışan `pnpm tauri dev` durdurulmaz** (port 1420); e2e 1421'i kullanır. `pkill -f` kullanılmaz. Üretilen `.app` çalıştırılmaz. `~/Library/Application Support/io.github.turgaydogusan.jyotishdesk` dizinine dokunulmaz.
- Yalnızca macOS hedeflenir. Yorum metni ve AI paneli bu planın dışındadır. Prashna ve arama (3.8, 3.9) bu planın dışındadır; `timeline` ve `home_place` onlar için hazır bırakılır.
- **Tabanlar (`ff3da00`):** `jyotish-core` lib 307, golden 8, `modules_golden` 13; `atlas` `moment_fixed_lmt` 10, `moment_iana` 12 (diğerleri: `build` 6, `search` 8, `shipped` 3); `store` lib 96 (+1 `#[ignore]`); `jyotish-desk` lib 46, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `Test Files  75 passed (75)`, `Tests  738 passed (738)`; e2e 20; görsel 10 (yerel, darwin); yayın betiği testleri 14.
- **Sayaç zinciri** (her satır görev bitince; boş hücre "önceki satırla aynı"). Bu plan yazılırken `ff3da00`'ın bir kopyasında her görev ayrı commit olarak uygulanıp ölçülmüştür.

  | Görev | `jyotish-core` lib | `modules_golden` | `atlas` `moment_fixed_lmt` / `moment_iana` | `store` lib | `jyotish-desk` lib | Vitest dosya / test | e2e |
  |---|---|---|---|---|---|---|---|
  | Taban | 307 | 13 | 10 / 12 | 96 (+1 ign.) | 46 | 75 / 738 | 20 |
  | Task 1 | 321 | | | | | 75 / 738 | |
  | Task 2 | 331 | 15 | | | | | |
  | Task 3 | 351 | | | | | | |
  | Task 4 | | | 11 / 13 | 103 (+1 ign.) | | 75 / 738 | |
  | Task 5 | | | | | 49 | 75 / 738 | |
  | Task 6 | | | | | | 78 / 771 | |
  | Task 7 | | | | | | 80 / 795 | |
  | Task 8 | | | | | | | 22 |

  Değişmeyenler: `jyotish-core` golden 8; `atlas` `build` 6, `search` 8, `shipped` 3; `jyotish-desk` `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; görsel 10; yayın betiği testleri 14. `jyotish-core` lib Task 1'de 14 test (`timeline.rs` 13, proptest 1), Task 2'de 10 test (`segments.rs` 8, proptest 2), Task 3'te 20 test (`day.rs` 6, `search.rs` 11, proptest 3) kazanır; `modules_golden` iki test kazanır (`muhurta_hand_fixtures`, `muhurta_fixtures_cover_the_required_cases`). `atlas` iki test (her dosyada bir), `store` lib yedi test (`settings.rs` 2, `muhurta.rs` 4, `exchange.rs` 1), `jyotish-desk` lib üç test (`muhurta_commands.rs`) kazanır. `jyotish-desk` `modules` tek test olarak durur (süreçte tek SE örneği); Task 5 ona `timeline_checks` ve `muhurta_checks`'i ekler. Vitest: Task 1, 4 ve 5'te sayı değişmez (`i18n.test.ts` ve `api.test.ts`'in tabloları büyür); Task 6 üç dosya ve 33 test (`modules/muhurta.test.ts` 8, `HomePlaceDialog.test.tsx` 6, `MuhurtaDayPanel.test.tsx` 15, `useModule.test.tsx` 1, `format.test.ts` 3), Task 7 iki dosya ve 24 test (`useMuhurtaSearch.test.tsx` 6, `MuhurtaSearchPanel.test.tsx` 18) ekler. Task 8 iki e2e testi ekler.
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir:**
  - **C1** Gizlilik (spec §14): danışan adları, doğum verisi, not gövdeleri, yer bilgisi, dosya yolları ve `CommandError.params` loglanmaz; `no-console.test.ts` durur ve izin listesi büyümez. Bulunulan yer, özel yer, sorgu, doğum Ay'ı ve kayıtlı süzgeç adları kişisel veridir: loglanmaz, hata parametresine girmez. Hiçbir modül ağ isteği yapmaz. Yeni tek `CommandError` kodu `invalid_muhurta_input`'tur (parametresi yalnızca `field`); `invalid_input` dört yeni alan adı taşır (`home_place`, `muhurta_filters`, `muhurta_filters.name`, `muhurta_filters.criteria`).
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu ya da adres almaz. Bu planın dört komutu doğrudan dizgi bağımsız değişken almaz (tarihler `MuhurtaQuery` nesnesinin alanlarıdır ve Rust'ta takvim günü olarak ayrıştırılır); `api.test.ts`'in dizgi bağımsız değişken listesi değişmez.
  - **C3** Göç V1 donmuştur; `crates/store/src/schema.rs` değişmez. `CalcSettings` ve `ViewState` tipleri ve bütün şema sürümleri (`APP_PREFS_SCHEMA_VERSION` dahil) değişmez. `AppPrefs` bir alan kazanır (D21); eski kayıtlar göçsüz okunur.
  - **C4** Arayüz astrolojik hesap yapmaz: dilimler, sınıflar, işaretler, kümeler, pencereler ve ofsetler Rust'tan gelir; "şimdi" ve "bugün" Rust'ta okunur. Bir anın sabit ofsette saat olarak yazılması, sürenin "2 sa 05 dk" diye yazılması, Abhijit çubuğunun gündüz içindeki payı, kimliğin adına çevrilmesi ve koşul kümelerinin sıralanması gösterimdir.
  - **C5/C17** Otomatik kayıtlar ve boşaltılmaları değişmez: panelin yeri, tarih bağı, özel yeri ve etkin süzgeci panel ayarıdır ve mevcut düzen kaydıyla yazılır. **C6** Düzen ayrıştırıcıları (`parseLayoutState`, Rust `check_panel_layout`) değişmez; iki yeni panel türü kataloğa eklenir.
  - **C7** Kısayollar yalnızca `shortcuts.ts`'tedir; bu plan kısayol eklemez (K12). **C8** Her pencere `shell/Dialog`'u kullanır (bulunulan yer penceresi dahil).
  - **C9** Renk yalnızca token'lardan gelir (hex yok) ve tek başına anlam taşımaz: işaretli dilim tarama deseni ve kısa yazıyla, uyarı kenar çizgisi, rozet ve metinle, yaklaşık değer "≈" ve sözle, vurgu zemin ve yazı ağırlığıyla, sabitleme `aria-pressed` ile gösterilir.
  - **C10** Üretilen dosyalar elle düzenlenmez: `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` Task 4 ve Task 5'te komutla yeniden üretilir.
  - **C11** Görsel PNG temelleri yereldir; bu planda değişmeleri beklenmez (D37).
  - **C12** Her `CommandError.code` üç dilde çevrilidir (bu plan birini ekler: Task 1). Her yeni i18n anahtarı üç dilde vardır (`i18n.test.ts` anahtar kümelerini ve yer tutucuları karşılaştırır).
  - **C13** Yasaklı ad taraması her commit'te; depo dışındaki ilerleme notları ve `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** Plan 07'nin gün doğumu toleransı ve `compute_panchang` değişmez: bu plan `panchang.rs`'te yalnızca iki öğenin görünürlüğünü `pub(crate)` yapar. **C15** `alert`, `confirm`, `prompt`, `window.print()` ve `window.open()` çağrılmaz. **C16** ⌘ kısayolları değişmez.
- **Retro** her yerde düz `R`'dir ve Rahu ile Ketu'da gösterilmez (UI spec §2.3); bu planın panelleri retro göstermez.
- **Erişilebilirlik:** WCAG AA. Tablolar `caption`, `scope="col"` ve `scope="row"` taşır ve klavye odağı alan kaydırma kaplarındadır (`tabIndex=0`, `role="group"`, adları başlıklarıdır); gün şeridi adlı bir sıralı listedir ve Abhijit çubuğu yalnızca çizimdir (`aria-hidden`; saatleri metin olarak da yazılıdır); işaretler adlı bir listedir; kırpılmış uçların ve "≈" işaretinin sözcük karşılığı ekran okuyucuya okunur; form alanlarının görünür etiketi vardır ve onay kutusu kümeleri `fieldset` ve `legend` taşır; ilerleme `role="status"`, hata `role="alert"`'tir. **Yeniden okuma içeriği DOM'dan çıkarmaz ve odağı düşürmez; odaktaki denetimi kaldıran her eylem odağı bilerek taşır (D27, D32).** Kullanılamayan düğme yerel `disabled` değildir: odaklanır ve nedeni görünür bir satırdır (Plan 11 D22).
- Commit mesajları İngilizcedir ve deponun `Co-Authored-By:` satırıyla biter (`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`). Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, `git tag`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Kod blokları `ff3da00` ağacına göre yazılmış ve bu plan yazılırken deponun bir kopyasında koşulmuştur (Self-Review madde 5). Yeni dosyalar tam içerikle verilir ("**Yeni dosya:**"). Değişen dosyalar `diff` bloğuyla verilir: blok bir dosyaya konup depo kökünde `git apply <dosya>` ile ya da elle uygulanır; bağlam tutmazsa önceki görevin adımı denetlenir, elle uydurma yapılmaz. Bir dosyanın test ve uygulama değişiklikleri ayrı adımlarda ayrı bloklardır; yeni bir Rust dosyasının `mod tests` bölümü dosyayla birlikte gelir (dosyayı yazarken önce testleri okuyun). Dosya ve diff blokları dört ters tırnakla çevrilidir (içlerinde üç ters tırnak geçebilir); komut blokları üç ters tırnaklıdır. `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` diff olarak verilmez; adımlardaki komutlar üretir.

## Sonraki alt projelere verilen API

Bu planın ürettiği ve sonraki işlerin tüketeceği adlar. Rust adları yıldızlıysa `jyotish_core` kökünden, değilse `jyotish_core::timeline` ya da `jyotish_core::muhurta`'dan dışa aktarılır; imzalar görevlerde tam olarak verilir.

| API | Tüketen | Not |
|---|---|---|
| `Interval<T>`\* (`contains`, `days`), `NakshatraPada`\*, `Motion`\*, `SunDay`\*, `STATION_GRAHAS`\*, `TIMELINE_RESOLUTION_DAYS`\* | 3.9 olay araması; AI paketleyicisi | Aralıklar `[start, end)`; dizi verilen aralığa kırpılır |
| `graha_sign_series`\*, `graha_nakshatra_series`\*, `graha_motion_series`\*, `combust_series`\* | 3.9 (burç ve nakshatra girişi, dönüş anları; doğum Ay burcundan Satürn ve Jüpiter ev değişimi) | Giriş ve dönüş, iki aralığın sınırıdır; retro girişler dahil |
| `tithi_series`\*, `karana_series`\*, `nakshatra_series`\*, `nitya_yoga_series`\*, `vara_series`\*, `lagna_sign_series`\*, `lagna_sign_at`\*, `sun_days`\*; `timeline::{map_merge, value_at, tithis_of_karanas, sun_days_cover, vara_of_sun_days}` | 3.9 (yeni ay ve dolunay: tithi 30 → 1 ve 15 → 16 sınırı), 3.8 Prashna (anın panchangı) | `compute_panchang` ile aynı değerler |
| `day_panchang`\*, `DayPanchang`\*, `DaySegments`\*, `DayTithi`\*, `DayNakshatra`\*, `DayKarana`\*, `PersonalFactors`\*; Tauri komutu `day_panchang` | Arayüz; AI paneli; 3.8 | Anı içeren gün |
| `day_segments(&SunDay)`\*, `muhurta::{rahu_segment, yamaganda_segment, gulika_segment, equal_parts}`, `tithi_group`\*, `nakshatra_nature`\*, `tara_bala`\*, `chandra_bala`\*, `muhurta::{is_rikta, is_vishti, is_panchaka}`, `TithiGroup`\*, `NakshatraNature`\*, `TaraBala`\*, `ChandraBala`\*, `MuhurtaFlag`\*, `NatalMoon`\* | 3.8 (soru anının sınıfları), 3.9 `search_facts` | Saf; ephemeris görmez |
| `muhurta_search_page`\*, `muhurta_chunks`\*, `window_snapshot`\*, `MuhurtaCriteria`\*, `MuhurtaFilter`\*, `MuhurtaCursor`\*, `MuhurtaSearchPage`\*, `MuhurtaWindow`\*, `WindowSnapshot`\*; Tauri komutu `muhurta_search` | Arayüz; 3.9'un olay aramasının sayfalama kalıbı | Durumsuz; imleçli |
| `CoreError::InvalidMuhurtaInput`; `CommandError` kodu `invalid_muhurta_input` | 3.8, 3.9 | Yalnızca alan adı |
| `atlas::offset_at(tz, lon, utc)` | 3.8 (soru anının yerel saati), 3.9 | Bir andaki UTC ofseti |
| `store`: `HomePlace`, `AppPrefs.home_place`, `SavedMuhurtaFilter`, `Store::muhurta_filters`, `Store::save_muhurta_filters`; Tauri komutları `load_muhurta_filters`, `save_muhurta_filters` | 3.8 Prashna (varsayılan yer), 3.9 (`saved_searches` aynı kalıbı izler) | Dışa aktarılmaz |
| `muhurta_commands::{MuhurtaPlace, DayPanchangView, MuhurtaQuery, MuhurtaPageCursor, MuhurtaSearchView, MuhurtaWindowView}`; `transit_commands::transit_moment` | 3.8, 3.9 | Yer ad taşımaz |
| Arayüz: `modules/muhurta.ts` (`resolvePlace`, `muhurtaPlace`, `chartPlace`, `hitPlace`, `placeLabel`, `natalMoon`, `parseCriteria`, `toggleMember`, `VARAS`, `MUHURTA_FLAGS`); `modules/useModule.ts` (`useLastData`); `modules/useDayPanchang.ts`; `modules/useMuhurtaSearch.ts`; `panels/MuhurtaPlace.tsx` (`usePanelPlace`, `PlaceControls`, `PlaceMissing`); `panels/HomePlaceDialog.tsx`; `panels/format.ts` (`formatJdClock`, `formatJdDate`, `formatJdSpan`); `panels/labels.ts` (`TARA_NAMES`, `taraName`, `NAKSHATRA_NATURE_NAMES`, `tithiShort`); `Transit.epoch`; fikstürde `muhurta`, `muhurta_ids` | 3.8 (yer seçimi ve bulunulan yer penceresi), 3.9 (dilimli arama kancası, sonuç listesi) | i18n: `muhurta.*`, `muhurtaDay.*`, `muhurtaSearch.*` |

## Dosya Yapısı (bu plan)

```
crates/jyotish-core/
  src/timeline.rs                 yürüyücü ve aralık dizileri (yeni)
  src/muhurta/mod.rs              MuhurtaFlag, NatalMoon, yöntem kimliği (yeni)
  src/muhurta/segments.rs         dilimler, hora, sınıflar, tara ve chandra bala (yeni)
  src/muhurta/day.rs              day_panchang (yeni)
  src/muhurta/search.rs           süzgeç, dilimli arama (yeni)
  src/error.rs                    InvalidMuhurtaInput
  src/fake.rs                     sahte ephemeris'te dönen lagna
  src/panchang.rs                 iki öğenin görünürlüğü
  src/lib.rs                      modül kaydı ve dışa aktarımlar
  src/proptests.rs                timeline_properties, muhurta_properties
  tests/modules_golden.rs         muhurta fikstür düzeneği ve kapsam testi
tests/golden/modules/
  README.md                       muhurta satırı
  muhurta/REVIEW.md               okuma kuralları, tablolar, onay bekleyen noktalar, kayıtlar (yeni)
  muhurta/*.json                  7 el örneği (yeni)
crates/atlas/
  src/moment.rs, src/lib.rs       offset_at
  tests/moment_iana.rs, tests/moment_fixed_lmt.rs
crates/store/
  src/model.rs                    HomePlace
  src/settings.rs                 AppPrefs.home_place, toleranslı okuma, yazım denetimi
  src/muhurta.rs                  kayıtlı süzgeçler (yeni)
  src/exchange.rs                 dışa aktarımda yokluk testi
  src/lib.rs                      modül kaydı ve dışa aktarımlar
app/src-tauri/
  src/commands.rs                 invalid_muhurta_input eşlemesi
  src/muhurta_commands.rs         day_panchang, muhurta_search (yeni)
  src/store_commands.rs           load_muhurta_filters, save_muhurta_filters
  src/transit_commands.rs         transit_moment görünürlüğü
  src/lib.rs                      komut kaydı
  tests/modules.rs                timeline ve muhurta entegrasyonu, çapraz denetim, bütçeler
  tests/ui_fixture.rs             fikstüre muhurta ve muhurta_ids
app/ui/
  bindings.ts                     üretilir
  test/fixtures/delhi.json        üretilir
  test/fixture.ts                 UiFixture'ın iki yeni alanı
  api.ts, api.test.ts             dört sarmalayıcı
  transit/store.tsx (+test)       Transit.epoch
  modules/useModule.ts (+test)    useLastData
  modules/muhurta.ts (+test)      yer, doğum Ay'ı ve koşul yardımcıları (yeni)
  modules/useDayPanchang.ts       kanca (yeni)
  modules/useMuhurtaSearch.ts (+test)   dilimli arama kancası (yeni)
  panels/format.ts (+test)        saat, gün ve süre biçimleri
  panels/labels.ts                tara adları, doğa adları, kısa tithi
  panels/HomePlaceDialog.tsx (+test)    bulunulan yer penceresi (yeni)
  panels/MuhurtaPlace.tsx         yer seçimi (yeni)
  panels/MuhurtaDayPanel.tsx (+test)    (yeni)
  panels/MuhurtaSearchPanel.tsx (+test) (yeni)
  panels/registry.ts, panels.test.tsx   iki panel türü
  panels/panels.css
  layout/model.ts (+test)         PANEL_KINDS, MODULE_PANELS
  i18n/{tr,en,fr}.ts, i18n.test.ts      errors.invalid_muhurta_input, panel.*, muhurta, muhurtaDay, muhurtaSearch
app/e2e/{ipc.ts,desk.spec.ts}     dört komutun taklidi; iki panelin akışı
```

### Kapsam

| Spec maddesi | Yer |
|---|---|
| §10 Zaman çizelgesi çekirdeği: tithi, nakshatra (pada ile), nitya yoga, karana, vara, Ay burcu, Güneş burcu | Task 1 (`timeline.rs`; D2–D4, D6, D7) |
| §10 Lagna burcu dizisi | Task 1 (`lagna_sign_series`; D3) |
| §10 Graha burç ve nakshatra girişleri, retro ve düz dönüşler | Task 1 (`graha_sign_series`, `graha_nakshatra_series`, `graha_motion_series`; D5) |
| §10 Gün panchangı: beş uzvun gün içindeki değerleri ve değişim saatleri | Task 3 (`day.rs`; D8, D9, D12) |
| §10 Sekizde bir dilimler; Rahu dilimi, Yamaganda, Gulika; Abhijit; hora tablosu | Task 2 (`segments.rs`; D10, D11); Task 9 Step 6 |
| §10 Sınıflar: tithi grubu, nakshatra doğası | Task 2 (D12) |
| §10 İşaretler: Vishti, Vyatipata, Vaidhriti, panchaka, yanık Jüpiter ya da Venüs | Task 1 (`combust_series`), Task 2 (`is_*`), Task 3 (`flags`; D13) |
| §10 Kişiye bağlı etkenler: tara bala, chandra bala | Task 2 (`tara_bala`, `chandra_bala`), Task 3 (`PersonalFactors`; D14) |
| §10 Pencere araması: süzgeç alanları, kesişim, sonuç | Task 3 (`search.rs`; D15–D19) |
| §10 "Hazır etkinlik şablonu yoktur; astrolog süzgecini kurar, adlandırıp saklar" | Task 4 (`store/muhurta.rs`; D22), Task 7 (kayıtlı süzgeçler) |
| §10 Bulunulan yer: `AppPrefs.home_place`; konum servisi yok | Task 4 (D21), Task 6 (`HomePlaceDialog`; D29) |
| §10 Veri modeli: `Interval`, `DaySegments`, `DayPanchang`, `MuhurtaFilter`, `MuhurtaWindow` | Task 1–3 (D9, D10, D12, D15 farklarıyla) |
| §10 Komutlar: `day_panchang`, `muhurta_search`, `muhurta_cancel`; `NatalMoon` | Task 5 (D8, D16, D23) |
| §10 Arayüz: `muhurta_day` paneli (şerit, uzuv listesi, işaretler, hora tablosu, tara ve chandra bala) | Task 6 (D28–D30, D33, D34) |
| §10 Arayüz: `muhurta_search` paneli (form, "Ara", sonuç listesi, transit tarihine gitme) | Task 7 (D31, D32) |
| §10 Arayüz: dilimler desen ve yazıyla ayrılır | Task 6 (C9) |
| §10 Arayüz: 100 ms ve 1 sn eşikleri, ilerleme, "Durdur" | Task 7 (D32) |
| §10 Arayüz: kutup günü ya da gecesinde yalnızca uzuvlar ve uyarı rozeti | Task 3 (D9), Task 6 |
| §10 Test: çapraz denetim (`timeline` ↔ `compute_panchang`; sınır ≤ 1 sn) | Task 1 (sahte ephemeris), Task 3 (`day.rs`), Task 5 (`timeline_checks`: gerçek ephemeris, beş golden an) |
| §10 Test: el örnekleri (yedi vara, 06:00–18:00 günü, Abhijit); 27 × 27 ve 12 × 12 tabloların ikinci yazımı | Task 2 (yedi fikstür; `segments.rs` testleri; D11) |
| §10 Test: property'ler | Task 1 (`timeline_properties`), Task 2 ve 3 (`muhurta_properties`) |
| §10 Test: astronomik (giriş anında boylam ± 1″; dönüşte hız işareti) | Task 1 (sahte ephemeris), Task 5 (`timeline_checks`) |
| §10 Gizlilik: `home_place` loglanmaz, hata parametresine girmez, dışa aktarılmaz | Task 4 (`exchange.rs` testi), D20, D25; Task 9 Step 3 |
| §10, §15 Performans: `day_panchang` ≤ 50 ms; arama ≤ 2 sn ve ≤ 5 sn; ilk sayfa ≤ 500 ms; 300 ms bekleme | Task 5 (`muhurta_checks`; D26) |
| §13 Panel durumları: boş, yükleniyor, hata, dolu, "saat bilinmiyor"; ortak seçim; i18n | Task 6, 7 |
| K12 Paletten ekleme | Task 6, 7; Task 8 |
| Uçtan uca akış | Task 8 |
| Son doğrulama, elle denetim, el örneklerinin gözden geçirilmesi, klasik tabloların onayı | Task 9 |

---


### Task 1: Çekirdek: zaman çizelgesi (`timeline`) ve yeni hata türü

Spec §10 "Zaman çizelgesi çekirdeği" (üç madde), "Test" (çapraz denetim; property "aralıklar boşluksuz ve örtüşmesiz"; astronomik); §13 katman 2 ve 4; K3, K5, K8; D1–D7, D20. Hiçbir mevcut davranış değişmez: yeni bir modül eklenir, `panchang.rs`'te bir sabitin görünürlüğü `pub(crate)` olur, sahte ephemeris dönen bir lagna kazanır (varsayılanı sabit lagnadır). Yeni hata türü çekirdeğe eklendiği için uygulamanın hata eşlemesi ve üç dildeki iletisi de bu görevdedir (derleme ve C12 bunu ister). Gün panchangı ve arama Task 3'tedir.

**Files:**
- Create: `crates/jyotish-core/src/timeline.rs`
- Modify: `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/error.rs`, `crates/jyotish-core/src/fake.rs`, `crates/jyotish-core/src/panchang.rs`, `crates/jyotish-core/src/proptests.rs`, `app/src-tauri/src/commands.rs`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`, `app/ui/i18n/i18n.test.ts`
- Test: `timeline.rs`'in `mod tests`'i (13 test), `proptests.rs` `timeline_properties` (1 test); `commands.rs` ve `i18n.test.ts`'in mevcut testleri büyür

**Interfaces:**
- Consumes (v1, Plan 14): `ephemeris::{Ephemeris, Body, RiseSet, SunDisc, Zodiac}`, `CalcSettings` (`validate`, `ayanamsa.sidereal_mode`, `node.body`, `sunrise_def.sun_disc`, `house_system.ephemeris_system`, `combustion_orbs`), `JD_UT_RANGE`, `chart::{finite, rasi_frame}`, `combustion::is_combust`, `panchang::{day_sunrise, vara_at, local_weekday, karana_no, nitya_yoga_no, Vara, SUNRISE_WINDOW_DAYS}`, `zodiac::{nakshatra_pada, normalize, Sign}`, `Graha`, `CoreError`.
- Produces (`jyotish_core::timeline`; yıldızlılar kökten de dışa aktarılır):
  - Tipler: `Interval<T> { value, start_jd_ut, end_jd_ut }`\* (`contains`, `days`), `NakshatraPada { nakshatra, pada }`\*, `Motion { Direct, Retrograde }`\*, `SunDay { sunrise_jd_ut, sunset_jd_ut, next_sunrise_jd_ut, vara }`\*.
  - Sabitler: `TIMELINE_RESOLUTION_DAYS`\*, `MOON_STEP_DAYS`, `GRAHA_STEP_DAYS`, `LAGNA_STEP_DAYS`, `VARA_STEP_DAYS`, `STATION_GRAHAS`\*.
  - Diziler (hepsi `(eph, settings, …, from_jd_ut, to_jd_ut) -> Result<Vec<Interval<_>>, CoreError>`): `karana_series`\*, `tithi_series`\*, `nakshatra_series`\*, `nitya_yoga_series`\*, `graha_sign_series(…, graha, …)`\*, `graha_nakshatra_series`\*, `graha_motion_series`\*, `combust_series`\*, `lagna_sign_series(…, lat, lon, …)`\*, `vara_series(…, lat, lon, …)`\*, `sun_days(…, lat, lon, …) -> Vec<SunDay>`\*.
  - Yardımcılar: `lagna_sign_at(eph, settings, jd_ut, lat, lon) -> Sign`\*, `tithi_of_karana`, `tithis_of_karanas`, `map_merge`, `value_at`, `sun_days_cover`, `vara_of_sun_days`; crate içi `walk`, `check_span`.
  - `CoreError::InvalidMuhurtaInput(&'static str)`; `CommandError` kodu `invalid_muhurta_input { field }`; `errors.invalid_muhurta_input` (üç dil).

- [ ] **Step 1: Tabanları ölç**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
git log -1 --format=%h
git status --short
rustc --version
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `git status --short` boş; `rustc 1.100.0` ya da yenisi; `jyotish-core` lib `307 passed`, golden `8 passed`, `modules_golden` `13 passed`; `atlas` `moment_fixed_lmt` `10 passed`, `moment_iana` `12 passed`; `store` lib `96 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `46 passed`, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `75 passed (75)`, `738 passed (738)`. Commit `ff3da00` değilse ölçülen sayılar ilerleme notlarına taban olarak yazılır ve zincir oradan yürütülür (farklar aynıdır). (Vitest'te `NotesPanel.test.tsx › New note adds an empty note…` yük altında seyrek olarak düşebilir; yeniden koşunca geçer ve bu planın dosyalarına dokunmaz.)

- [ ] **Step 2: Testler: modül kaydı, dizi değişmezi ve yeni hata kodu**

Dört değişiklik: modül kaydedilir (dosyası henüz yoktur), `timeline_properties` her dizinin aralığı boşluksuz böldüğünü ister, uygulamanın hata testi ve arayüzün hata kodu listesi yeni kodu bekler.

````diff
diff --git a/app/src-tauri/src/commands.rs b/app/src-tauri/src/commands.rs
index b79d449..a60407b 100644
--- a/app/src-tauri/src/commands.rs
+++ b/app/src-tauri/src/commands.rs
@@ -240,6 +240,11 @@ mod tests {
         let e = CommandError::from(CoreError::InvalidTransitInput("natal_bhavas"));
         assert_eq!(e.code, "invalid_transit_input");
         assert_eq!(e.params["field"], "natal_bhavas");
+        let e = CommandError::from(CoreError::InvalidMuhurtaInput("place"));
+        assert_eq!(e.code, "invalid_muhurta_input");
+        // Yalnızca alan adı: yer ve tarih hata parametresine girmez.
+        assert_eq!(e.params.len(), 1);
+        assert_eq!(e.params["field"], "place");
     }
 
     #[test]
diff --git a/app/ui/i18n/i18n.test.ts b/app/ui/i18n/i18n.test.ts
index fa02a78..b2925d0 100644
--- a/app/ui/i18n/i18n.test.ts
+++ b/app/ui/i18n/i18n.test.ts
@@ -74,7 +74,7 @@ const BACKEND_CODES = [
   "store_sqlite", "store_migration", "store_json", "store_too_new", "store_unavailable", "not_found", "invalid_input",
   "unknown_time_zone", "nonexistent_local_time", "ambiguous_local_time", "offset_out_of_range", "longitude_out_of_range",
   "date_out_of_range", "last_chart", "unsupported_schema_version", "invalid_settings", "invalid_location", "invalid_julian_day",
-  "non_finite_result", "invalid_dasha_input", "panchang_search", "invalid_transit_input",
+  "non_finite_result", "invalid_dasha_input", "panchang_search", "invalid_transit_input", "invalid_muhurta_input",
   "atlas_unavailable", "atlas_sqlite", "atlas_io", "atlas_schema",
   "file_io", "dialog_path", "import_too_large", "import_format", "import_version", "import_record",
   "prefs_too_new",
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 6269ae9..8f26241 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -28,6 +28,7 @@ pub mod positions;
 mod proptests;
 pub mod settings;
 pub mod shadbala;
+pub mod timeline;
 pub mod transit;
 pub mod varga;
 pub mod yogas;
diff --git a/crates/jyotish-core/src/proptests.rs b/crates/jyotish-core/src/proptests.rs
index 7602b8b..8db4517 100644
--- a/crates/jyotish-core/src/proptests.rs
+++ b/crates/jyotish-core/src/proptests.rs
@@ -1457,3 +1457,106 @@ mod kuta_properties {
         }
     }
 }
+
+mod timeline_properties {
+    use ephemeris::Body;
+    use proptest::prelude::*;
+
+    use crate::fake::FakeEph;
+    use crate::graha::Graha;
+    use crate::settings::CalcSettings;
+    use crate::timeline::{
+        graha_sign_series, karana_series, lagna_sign_series, nakshatra_series, nitya_yoga_series,
+        tithi_series, vara_series, Interval,
+    };
+
+    const JD: f64 = 2_451_545.0;
+
+    /// Gerçekçi hızlarla sahte gök: Güneş, Ay, Jüpiter, Venüs ve dönen lagna rastgele yerlerde
+    /// başlar (tropikal boylamlar; hızlar °/gün).
+    #[derive(Debug, Clone)]
+    pub(super) struct Sky {
+        sun: (f64, f64),
+        moon: (f64, f64),
+        asc: f64,
+        jupiter: f64,
+        venus: f64,
+    }
+
+    impl Sky {
+        pub(super) fn eph(&self) -> FakeEph {
+            FakeEph {
+                asc: self.asc,
+                asc_speed: 360.98,
+                ..FakeEph::default()
+            }
+            .with(Body::Sun, self.sun.0, self.sun.1)
+            .with(Body::Moon, self.moon.0, self.moon.1)
+            .with(Body::Jupiter, self.jupiter, 0.08)
+            .with(Body::Venus, self.venus, 1.2)
+        }
+    }
+
+    pub(super) fn sky() -> impl Strategy<Value = Sky> {
+        (
+            (0.0f64..360.0, 0.95f64..1.02),
+            (0.0f64..360.0, 11.8f64..15.4),
+            0.0f64..360.0,
+            0.0f64..360.0,
+            0.0f64..360.0,
+        )
+            .prop_map(|(sun, moon, asc, jupiter, venus)| Sky {
+                sun,
+                moon,
+                asc,
+                jupiter,
+                venus,
+            })
+    }
+
+    fn check_partition<T: PartialEq + std::fmt::Debug>(
+        series: &[Interval<T>],
+        from: f64,
+        to: f64,
+    ) -> Result<(), TestCaseError> {
+        prop_assert!(!series.is_empty());
+        prop_assert_eq!(series[0].start_jd_ut, from);
+        prop_assert_eq!(series[series.len() - 1].end_jd_ut, to);
+        for w in series.windows(2) {
+            prop_assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
+            prop_assert!(w[0].value != w[1].value);
+        }
+        for i in series {
+            prop_assert!(i.start_jd_ut < i.end_jd_ut);
+        }
+        Ok(())
+    }
+
+    proptest! {
+        #![proptest_config(ProptestConfig::with_cases(48))]
+
+        #[test]
+        fn every_series_splits_the_span_without_gaps(
+            sky in sky(),
+            start in 0.0f64..400.0,
+            days in 0.01f64..12.0,
+        ) {
+            let mut eph = sky.eph();
+            let s = CalcSettings::default();
+            let (from, to) = (JD + start, JD + start + days);
+            check_partition(&tithi_series(&mut eph, &s, from, to).unwrap(), from, to)?;
+            check_partition(&karana_series(&mut eph, &s, from, to).unwrap(), from, to)?;
+            check_partition(&nakshatra_series(&mut eph, &s, from, to).unwrap(), from, to)?;
+            check_partition(&nitya_yoga_series(&mut eph, &s, from, to).unwrap(), from, to)?;
+            check_partition(&vara_series(&mut eph, &s, 41.0, 29.0, from, to).unwrap(), from, to)?;
+            check_partition(
+                &lagna_sign_series(&mut eph, &s, 41.0, 29.0, from, to).unwrap(),
+                from,
+                to,
+            )?;
+            for g in Graha::ALL {
+                check_partition(&graha_sign_series(&mut eph, &s, g, from, to).unwrap(), from, to)?;
+            }
+        }
+    }
+}
````


- [ ] **Step 3: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-core --lib timeline 2>&1 | grep -E "^error" | head -3
pnpm --dir app exec vitest run ui/i18n/i18n.test.ts 2>&1 | grep -E "Tests |FAIL"
```
Expected: `error[E0583]: file not found for module `timeline``, `error[E0560]: struct `FakeEph` has no field named `asc_speed`` (ve ardından gelen tip hataları); Vitest `FAIL  ui/i18n/i18n.test.ts > every backend error code reachable from the UI has a message` ve `1 failed | 12 passed (13)`. Sonraki üç adım bunları giderir. `timeline.rs`'i yazarken önce dosyanın sonundaki testleri okuyun: bir test düşerse önce spec §10 ve D2–D7 yeniden okunur; beklenen değer motor çıktısına bakarak değiştirilmez.

- [ ] **Step 4: Hata türü, dönen sahte lagna ve bir sabitin görünürlüğü**

`InvalidMuhurtaInput` yalnızca alan adını taşır (D20). Sahte ephemeris'in lagnası `asc_speed` ile döner (varsayılan 0: mevcut testler değişmez). `RISE_STEP_DAYS` `timeline`'ın gün listesinde kullanılır.

````diff
diff --git a/crates/jyotish-core/src/error.rs b/crates/jyotish-core/src/error.rs
index f19b1b6..f874f81 100644
--- a/crates/jyotish-core/src/error.rs
+++ b/crates/jyotish-core/src/error.rs
@@ -23,4 +23,8 @@ pub enum CoreError {
     PanchangSearch(&'static str),
     #[error("invalid transit input: {0}")]
     InvalidTransitInput(&'static str),
+    /// Zaman çizelgesi, gün panchangı ya da muhurta araması girdisi geçersiz; alan adını taşır
+    /// (değer taşımaz).
+    #[error("invalid muhurta input: {0}")]
+    InvalidMuhurtaInput(&'static str),
 }
diff --git a/crates/jyotish-core/src/fake.rs b/crates/jyotish-core/src/fake.rs
index 864f23c..6ea6b4a 100644
--- a/crates/jyotish-core/src/fake.rs
+++ b/crates/jyotish-core/src/fake.rs
@@ -16,8 +16,10 @@ pub(crate) struct FakeEph {
     /// (cisim, tropikal boylam, boylam hızı °/gün)
     pub positions: Vec<(Body, f64, f64)>,
     pub ayanamsa: f64,
-    /// Tropikal lagna.
+    /// Tropikal lagna (`epoch` anında).
     pub asc: f64,
+    /// Lagnanın hızı, °/gün; 0 ise lagna sabittir. Zaman çizelgesi testleri 360 verir.
+    pub asc_speed: f64,
     pub precision: Precision,
     /// Yalnızca bu cisimler `Moshier` hassasiyetinde döner (`precision` ne olursa olsun).
     pub moshier_bodies: Vec<Body>,
@@ -56,6 +58,7 @@ impl Default for FakeEph {
             ],
             ayanamsa: 24.0,
             asc: 70.0,
+            asc_speed: 0.0,
             precision: Precision::SwissEph,
             moshier_bodies: Vec::new(),
             porphyry_fallback: false,
@@ -122,14 +125,14 @@ impl Ephemeris for FakeEph {
 
     fn houses(
         &mut self,
-        _jd_ut: f64,
+        jd_ut: f64,
         _geo_lat: f64,
         _geo_lon: f64,
         system: HouseSystem,
         zodiac: Zodiac,
     ) -> Result<Houses, EphError> {
         self.last_house_system = Some(system);
-        let asc = self.in_frame(self.asc, zodiac);
+        let asc = self.in_frame(self.asc + self.asc_speed * (jd_ut - self.epoch), zodiac);
         let cusps: [f64; 12] = std::array::from_fn(|i| normalize(asc + 30.0 * i as f64));
         Ok(Houses {
             cusps,
diff --git a/crates/jyotish-core/src/panchang.rs b/crates/jyotish-core/src/panchang.rs
index 2ef78fe..1cfb376 100644
--- a/crates/jyotish-core/src/panchang.rs
+++ b/crates/jyotish-core/src/panchang.rs
@@ -207,7 +207,7 @@ const SUNRISE_NEAR_START_DAYS: f64 = 1.0;
 /// Pencerede taranan en çok doğuş (güvenlik sınırı).
 const MAX_SUNRISE_STEPS: usize = 4;
 /// Bulunan doğuşun ardından bir sonrakini aramak için atlanan süre (≈ 14 dk).
-const RISE_STEP_DAYS: f64 = 0.01;
+pub(crate) const RISE_STEP_DAYS: f64 = 0.01;
 /// Gün batımı gün doğumundan sonra bu süre içinde olmalıdır; değilse yok sayılır.
 const MAX_DAYLIGHT_DAYS: f64 = 1.0;
 /// Gün doğumu yoksa günün başladığı yerel ortalama saat: 06:00 = 0.25 gün.
````


- [ ] **Step 5: Zaman çizelgesi**

**Yeni dosya:** `crates/jyotish-core/src/timeline.rs`

````rust
//! Zaman çizelgesi çekirdeği (ileri modüller spec'i §10): bir zaman aralığı için ardışık,
//! boşluksuz `[start, end)` aralık dizileri. Gün panchangı, pencere araması (Alt proje 3.7) ve
//! olay araması (3.9) bu dizileri kullanır.
//!
//! Yöntem tek bir yürüyücüdür (`walk`): değer sabit adımlarla örneklenir; iki örnek arasında
//! değer değişmişse değişim anı ikiye bölmeyle (`TIMELINE_RESOLUTION_DAYS`) bulunur. Değer
//! v1'in panchang fonksiyonlarıyla (`tithi_no`, `karana_no`, `nakshatra_pada`, `nitya_yoga_no`,
//! `Sign::of`) aynı ephemeris sorgularından hesaplanır: bir aralığın içindeki her anda
//! `compute_panchang` aynı değeri verir. Sınır, değerin değiştiği ilk örnektir (yeni değer
//! aralığın başında geçerlidir); gerçek sınır en çok bir çözünürlük adımı öncedir.
//!
//! Dizi `[from, to]` aralığına kırpılır: ilk aralık `from`'da başlar, sonuncusu `to`'da biter.
//! Zamanlar UT Jülyen günüdür. Burç dizileri rasi çerçevesindedir (K5), nakshatra ve yoga
//! dizileri her kipte sidereal boylamdandır.

use ephemeris::{Body, Ephemeris, RiseSet, SunDisc, Zodiac};
use serde::{Deserialize, Serialize};

use crate::chart::{finite, rasi_frame};
use crate::combustion::is_combust;
use crate::graha::Graha;
use crate::panchang::{
    day_sunrise, karana_no, local_weekday, nitya_yoga_no, vara_at, Vara, RISE_STEP_DAYS,
    SUNRISE_WINDOW_DAYS,
};
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::zodiac::{nakshatra_pada, normalize, Sign};
use crate::CoreError;

/// Sınır aramasının çözünürlüğü: 0,05 saniye.
pub const TIMELINE_RESOLUTION_DAYS: f64 = 0.05 / 86_400.0;
/// Ay'a bağlı dizilerin örnekleme adımı. En kısa dilim (pada, 3°20′) ~5 saat sürer; adım
/// yalnızca tam bir çevrimden (en kısası ~27 gün) kısa olmak zorundadır.
pub const MOON_STEP_DAYS: f64 = 0.5;
/// Güneş burcu, graha burcu, graha nakshatrası, hareket yönü ve yanma dizilerinin adımı.
pub const GRAHA_STEP_DAYS: f64 = 1.0;
/// Lagna burcu dizisinin adımı (30 dakika): lagna bir günde on iki burcu dolaşır.
pub const LAGNA_STEP_DAYS: f64 = 1.0 / 48.0;
/// Vara dizisinin yavaş yolundaki adım (gün doğumu olmayan günler).
pub const VARA_STEP_DAYS: f64 = 0.25;
/// Tek bir sınır için en çok ikiye bölme adımı (1 gün / 2^21 ≈ 0,04 sn; pay bırakılmıştır).
const MAX_BISECTIONS: u32 = 40;
/// Bir örnekleme adımının içinde bulunabilecek en çok değişim (güvenlik sınırı).
const MAX_CHANGES_PER_STEP: usize = 64;

/// `[start_jd_ut, end_jd_ut)` boyunca geçerli değer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Interval<T> {
    pub value: T,
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
}

impl<T> Interval<T> {
    pub fn contains(&self, jd_ut: f64) -> bool {
        self.start_jd_ut <= jd_ut && jd_ut < self.end_jd_ut
    }

    pub fn days(&self) -> f64 {
        self.end_jd_ut - self.start_jd_ut
    }
}

/// Ay'ın nakshatrası (1..=27) ve padası (1..=4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct NakshatraPada {
    pub nakshatra: u8,
    pub pada: u8,
}

/// Grahanın boylamca hareket yönü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Motion {
    Direct,
    Retrograde,
}

/// Hareket yönü dizisi verilen grahalar: retro olabilen beş graha. Güneş ve Ay hep düzdür;
/// düğümlerin yönü olay sayılmaz (ortalama düğüm hep geri gider, gerçek düğüm birkaç günde
/// bir yön değiştirir).
pub const STATION_GRAHAS: [Graha; 5] = [
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// Güneş'in doğuşu, batışı ve sonraki doğuşu bilinen gün (gün doğumu → gün doğumu).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SunDay {
    pub sunrise_jd_ut: f64,
    pub sunset_jd_ut: f64,
    pub next_sunrise_jd_ut: f64,
    /// Gün doğumunun yerel ortalama tarihinin hafta günü (`compute_panchang` ile aynı kural).
    pub vara: Vara,
}

/// Aralık denetimi: iki uç da hesap aralığında ve `from < to`.
pub(crate) fn check_span(from_jd_ut: f64, to_jd_ut: f64) -> Result<(), CoreError> {
    for jd in [from_jd_ut, to_jd_ut] {
        // NaN ve sonsuz da aralık dışındadır.
        if !JD_UT_RANGE.contains(&jd) {
            return Err(CoreError::InvalidJulianDay(jd));
        }
    }
    if from_jd_ut >= to_jd_ut {
        return Err(CoreError::InvalidMuhurtaInput("range"));
    }
    Ok(())
}

fn check_place(geo_lat: f64, geo_lon: f64) -> Result<(), CoreError> {
    if (-90.0..=90.0).contains(&geo_lat) && (-180.0..=180.0).contains(&geo_lon) {
        Ok(())
    } else {
        Err(CoreError::InvalidMuhurtaInput("place"))
    }
}

/// Yürüyücü: `value`'nun `[from, to]` boyunca aldığı değerler, değişim anlarıyla. `step`
/// örnekleme adımıdır; bir adımın içinde değer değişip aynı değere dönerse o kısa gidiş
/// görülmez (çağıran adımı buna göre seçer).
pub(crate) fn walk<E, V, F>(
    eph: &mut E,
    from: f64,
    to: f64,
    step: f64,
    mut value: F,
) -> Result<Vec<Interval<V>>, CoreError>
where
    E: Ephemeris + ?Sized,
    V: Copy + PartialEq,
    F: FnMut(&mut E, f64) -> Result<V, CoreError>,
{
    check_span(from, to)?;
    let mut out = Vec::new();
    let mut start = from;
    let mut current = value(eph, from)?;
    let mut t = from;
    while t < to {
        let next = (t + step).min(to);
        let at_next = value(eph, next)?;
        let mut lo = t;
        let mut changes = 0;
        while current != at_next {
            changes += 1;
            if changes > MAX_CHANGES_PER_STEP {
                return Err(CoreError::PanchangSearch("timeline"));
            }
            // `lo`'da değer `current`, `hi`'de `at_hi` (başka bir değer).
            let (mut hi, mut at_hi) = (next, at_next);
            let mut steps = 0;
            while hi - lo > TIMELINE_RESOLUTION_DAYS && steps < MAX_BISECTIONS {
                let mid = 0.5 * (lo + hi);
                let at_mid = value(eph, mid)?;
                if at_mid == current {
                    lo = mid;
                } else {
                    hi = mid;
                    at_hi = at_mid;
                }
                steps += 1;
            }
            out.push(Interval {
                value: current,
                start_jd_ut: start,
                end_jd_ut: hi,
            });
            current = at_hi;
            start = hi;
            lo = hi;
        }
        t = next;
    }
    out.push(Interval {
        value: current,
        start_jd_ut: start,
        end_jd_ut: to,
    });
    Ok(out)
}

/// Değerleri `f` ile çevirir ve aynı değerli ardışık aralıkları birleştirir.
pub fn map_merge<T, U: PartialEq>(
    series: &[Interval<T>],
    mut f: impl FnMut(&T) -> U,
) -> Vec<Interval<U>> {
    let mut out: Vec<Interval<U>> = Vec::new();
    for i in series {
        let value = f(&i.value);
        match out.last_mut() {
            Some(last) if last.value == value && last.end_jd_ut == i.start_jd_ut => {
                last.end_jd_ut = i.end_jd_ut;
            }
            _ => out.push(Interval {
                value,
                start_jd_ut: i.start_jd_ut,
                end_jd_ut: i.end_jd_ut,
            }),
        }
    }
    out
}

/// `jd_ut` anını içeren aralığın değeri; an dizinin dışındaysa `None`.
pub fn value_at<T: Copy>(series: &[Interval<T>], jd_ut: f64) -> Option<T> {
    let i = series.partition_point(|x| x.end_jd_ut <= jd_ut);
    series.get(i).filter(|x| x.contains(jd_ut)).map(|x| x.value)
}

fn sidereal(settings: &CalcSettings) -> Zodiac {
    Zodiac::Sidereal(settings.ayanamsa.sidereal_mode())
}

fn lon_of<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    body: Body,
    zodiac: Zodiac,
) -> Result<f64, CoreError> {
    finite(eph.body(jd_ut, body, zodiac)?.lon, "timeline")
}

/// Karana (1..=60) dizisi: Ay − Güneş uzanımının 6°'lik dilimleri.
pub fn karana_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<u8>>, CoreError> {
    settings.validate()?;
    let zodiac = sidereal(settings);
    walk(eph, from_jd_ut, to_jd_ut, MOON_STEP_DAYS, |eph, jd| {
        let sun = lon_of(eph, jd, Body::Sun, zodiac)?;
        let moon = lon_of(eph, jd, Body::Moon, zodiac)?;
        Ok(karana_no(normalize(moon - sun)))
    })
}

/// Karananın ait olduğu tithi (1..=30): her tithi iki karanadır.
pub fn tithi_of_karana(karana: u8) -> u8 {
    karana.div_ceil(2).clamp(1, 30)
}

/// Bir karana dizisinden tithi dizisi (ardışık iki karana birleşir). Tithi sınırları karana
/// sınırlarının bir alt kümesidir; iki dizi aynı anları paylaşır.
pub fn tithis_of_karanas(karanas: &[Interval<u8>]) -> Vec<Interval<u8>> {
    map_merge(karanas, |k| tithi_of_karana(*k))
}

/// Tithi (1..=30) dizisi.
pub fn tithi_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<u8>>, CoreError> {
    Ok(tithis_of_karanas(&karana_series(
        eph, settings, from_jd_ut, to_jd_ut,
    )?))
}

/// Ay'ın nakshatra ve pada dizisi (sidereal boylam, her zodyak kipinde).
pub fn nakshatra_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<NakshatraPada>>, CoreError> {
    settings.validate()?;
    let zodiac = sidereal(settings);
    walk(eph, from_jd_ut, to_jd_ut, MOON_STEP_DAYS, |eph, jd| {
        let (nakshatra, pada) = nakshatra_pada(lon_of(eph, jd, Body::Moon, zodiac)?);
        Ok(NakshatraPada { nakshatra, pada })
    })
}

/// Nitya yoga (1..=27) dizisi: sidereal Güneş + Ay toplamı.
pub fn nitya_yoga_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<u8>>, CoreError> {
    settings.validate()?;
    let zodiac = sidereal(settings);
    walk(eph, from_jd_ut, to_jd_ut, MOON_STEP_DAYS, |eph, jd| {
        let sun = lon_of(eph, jd, Body::Sun, zodiac)?;
        let moon = lon_of(eph, jd, Body::Moon, zodiac)?;
        Ok(nitya_yoga_no(sun, moon))
    })
}

/// Grahanın ephemeris cismi ve boylamına eklenen açı (Ketu, Rahu'nun karşısıdır).
fn graha_body(graha: Graha, settings: &CalcSettings) -> (Body, f64) {
    match graha {
        Graha::Sun => (Body::Sun, 0.0),
        Graha::Moon => (Body::Moon, 0.0),
        Graha::Mars => (Body::Mars, 0.0),
        Graha::Mercury => (Body::Mercury, 0.0),
        Graha::Jupiter => (Body::Jupiter, 0.0),
        Graha::Venus => (Body::Venus, 0.0),
        Graha::Saturn => (Body::Saturn, 0.0),
        Graha::Rahu => (settings.node.body(), 0.0),
        Graha::Ketu => (settings.node.body(), 180.0),
    }
}

fn graha_step(graha: Graha) -> f64 {
    if graha == Graha::Moon {
        MOON_STEP_DAYS
    } else {
        GRAHA_STEP_DAYS
    }
}

/// Grahanın rasi çerçevesindeki burç dizisi; sınırlar burç girişleridir (retro girişler dahil).
/// `Graha::Moon` Ay burcu, `Graha::Sun` Güneş burcu dizisidir.
pub fn graha_sign_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    graha: Graha,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<Sign>>, CoreError> {
    settings.validate()?;
    let frame = rasi_frame(settings);
    let (body, shift) = graha_body(graha, settings);
    walk(eph, from_jd_ut, to_jd_ut, graha_step(graha), |eph, jd| {
        Ok(Sign::of(lon_of(eph, jd, body, frame)? + shift))
    })
}

/// Grahanın nakshatra (1..=27) dizisi (sidereal boylam); sınırlar nakshatra girişleridir.
pub fn graha_nakshatra_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    graha: Graha,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<u8>>, CoreError> {
    settings.validate()?;
    let zodiac = sidereal(settings);
    let (body, shift) = graha_body(graha, settings);
    walk(eph, from_jd_ut, to_jd_ut, graha_step(graha), |eph, jd| {
        Ok(nakshatra_pada(lon_of(eph, jd, body, zodiac)? + shift).0)
    })
}

/// Grahanın hareket yönü dizisi; sınırlar retro ve düz dönüş anlarıdır (boylam hızının sıfır
/// geçişi). Yalnızca `STATION_GRAHAS` için; diğerlerinde `InvalidMuhurtaInput("graha")`.
/// Retro tanımı v1 ile aynıdır: hız < 0.
pub fn graha_motion_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    graha: Graha,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<Motion>>, CoreError> {
    settings.validate()?;
    if !STATION_GRAHAS.contains(&graha) {
        return Err(CoreError::InvalidMuhurtaInput("graha"));
    }
    let frame = rasi_frame(settings);
    let (body, _) = graha_body(graha, settings);
    walk(eph, from_jd_ut, to_jd_ut, GRAHA_STEP_DAYS, |eph, jd| {
        let speed = finite(eph.body(jd, body, frame)?.lon_speed, "timeline")?;
        Ok(if speed < 0.0 {
            Motion::Retrograde
        } else {
            Motion::Direct
        })
    })
}

/// Grahanın yanık olup olmadığının dizisi (v1 yanma kuralı: `is_combust`, haritanın orblarıyla;
/// retro Merkür ve Venüs'ün orbu küçüktür, dönüş anında dizi değişebilir).
pub fn combust_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    graha: Graha,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<bool>>, CoreError> {
    settings.validate()?;
    let frame = rasi_frame(settings);
    let (body, shift) = graha_body(graha, settings);
    walk(eph, from_jd_ut, to_jd_ut, GRAHA_STEP_DAYS, |eph, jd| {
        let p = eph.body(jd, body, frame)?;
        let lon = finite(p.lon, "timeline")? + shift;
        let speed = finite(p.lon_speed, "timeline")?;
        let sun = lon_of(eph, jd, Body::Sun, frame)?;
        Ok(is_combust(
            graha,
            lon,
            sun,
            speed < 0.0,
            &settings.combustion_orbs,
        ))
    })
}

/// `jd_ut` anında yükselen burç (rasi çerçevesinde; `compute_chart`'ın lagnasıyla aynı sorgu).
pub fn lagna_sign_at<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    jd_ut: f64,
    geo_lat: f64,
    geo_lon: f64,
) -> Result<Sign, CoreError> {
    let houses = eph.houses(
        jd_ut,
        geo_lat,
        geo_lon,
        settings.house_system.ephemeris_system(),
        rasi_frame(settings),
    )?;
    Ok(Sign::of(finite(houses.asc, "lagna")?))
}

/// Lagna burcu dizisi: yükselen burcun değiştiği anlar.
pub fn lagna_sign_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    geo_lat: f64,
    geo_lon: f64,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<Sign>>, CoreError> {
    settings.validate()?;
    check_place(geo_lat, geo_lon)?;
    walk(eph, from_jd_ut, to_jd_ut, LAGNA_STEP_DAYS, |eph, jd| {
        lagna_sign_at(eph, settings, jd, geo_lat, geo_lon)
    })
}

fn rise_set<E: Ephemeris + ?Sized>(
    eph: &mut E,
    after: f64,
    event: RiseSet,
    geo_lat: f64,
    geo_lon: f64,
    disc: SunDisc,
) -> Result<Option<f64>, CoreError> {
    match eph.sun_rise_set(after, event, geo_lat, geo_lon, disc)? {
        Some(t) if !t.is_finite() => Err(CoreError::NonFiniteResult("sunrise")),
        Some(t) if t > after => Ok(Some(t)),
        _ => Ok(None),
    }
}

/// `[from, to]` ile kesişen, doğuşu, batışı ve sonraki doğuşu bilinen günler; sırayla. Ardışık
/// iki doğuş arası `SUNRISE_WINDOW_DAYS`'ten uzunsa ya da arada batış yoksa (kutup gecesi ya
/// da gündüzü) o gün listede yer almaz. İlk gün `from`'dan önce başlayabilir, sonuncusu
/// `to`'dan sonra bitebilir.
pub fn sun_days<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    geo_lat: f64,
    geo_lon: f64,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<SunDay>, CoreError> {
    settings.validate()?;
    check_span(from_jd_ut, to_jd_ut)?;
    check_place(geo_lat, geo_lon)?;
    let disc = settings.sunrise_def.sun_disc();
    let mut out = Vec::new();
    let mut rise = day_sunrise(eph, from_jd_ut, geo_lat, geo_lon, disc)?;
    // Bilinen doğuş yokken ileriye doğru aramanın başladığı an.
    let mut cursor = from_jd_ut;
    loop {
        let Some(r) = rise else {
            if cursor >= to_jd_ut {
                break;
            }
            // SE'nin yüksek enlem araması ~1 gün ileriye bakar: doğuş yoksa bir gün atlanır.
            rise = rise_set(eph, cursor, RiseSet::Rise, geo_lat, geo_lon, disc)?;
            cursor += 1.0;
            continue;
        };
        if r >= to_jd_ut {
            break;
        }
        let from = r + RISE_STEP_DAYS;
        match rise_set(eph, from, RiseSet::Rise, geo_lat, geo_lon, disc)? {
            Some(n) if n - r <= SUNRISE_WINDOW_DAYS => {
                let set = rise_set(eph, r, RiseSet::Set, geo_lat, geo_lon, disc)?;
                if let Some(s) = set.filter(|s| *s < n) {
                    out.push(SunDay {
                        sunrise_jd_ut: r,
                        sunset_jd_ut: s,
                        next_sunrise_jd_ut: n,
                        vara: local_weekday(r, geo_lon),
                    });
                }
                rise = Some(n);
            }
            // Sonraki doğuş uzakta: `r`'de başlayan gün atlanır.
            Some(n) => rise = Some(n),
            None => {
                rise = None;
                cursor = r + 1.0;
            }
        }
    }
    Ok(out)
}

/// `days` listesi `[from, to]` aralığını boşluksuz örtüyor mu (her günün sonraki doğuşu bir
/// sonraki günün doğuşu, ilk doğuş `from`'dan önce, son gün `to`'dan sonra bitiyor)?
pub fn sun_days_cover(days: &[SunDay], from_jd_ut: f64, to_jd_ut: f64) -> bool {
    let (Some(first), Some(last)) = (days.first(), days.last()) else {
        return false;
    };
    first.sunrise_jd_ut <= from_jd_ut
        && last.next_sunrise_jd_ut >= to_jd_ut
        && days
            .windows(2)
            .all(|w| w[0].next_sunrise_jd_ut == w[1].sunrise_jd_ut)
}

/// Vara dizisi: gün doğumundan gün doğumuna. Değer her anda `compute_panchang`'ın varasıdır
/// (gün doğumu yoksa gün 06:00 yerel ortalama saatte başlar). Günler aralığı boşluksuz
/// örtüyorsa dizi doğuş anlarından kurulur; değilse (kutup gecesi ya da gündüzü) v1 kuralı
/// adım adım yürünür.
pub fn vara_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    geo_lat: f64,
    geo_lon: f64,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<Vara>>, CoreError> {
    let days = sun_days(eph, settings, geo_lat, geo_lon, from_jd_ut, to_jd_ut)?;
    if sun_days_cover(&days, from_jd_ut, to_jd_ut) {
        return Ok(vara_of_sun_days(&days, from_jd_ut, to_jd_ut));
    }
    let disc = settings.sunrise_def.sun_disc();
    walk(eph, from_jd_ut, to_jd_ut, VARA_STEP_DAYS, |eph, jd| {
        vara_at(eph, jd, geo_lat, geo_lon, disc)
    })
}

/// Aralığı boşluksuz örten günlerden vara dizisi, `[from, to]`'ya kırpılmış.
pub fn vara_of_sun_days(days: &[SunDay], from_jd_ut: f64, to_jd_ut: f64) -> Vec<Interval<Vara>> {
    days.iter()
        .map(|d| Interval {
            value: d.vara,
            start_jd_ut: d.sunrise_jd_ut.max(from_jd_ut),
            end_jd_ut: d.next_sunrise_jd_ut.min(to_jd_ut),
        })
        .filter(|i| i.start_jd_ut < i.end_jd_ut)
        .collect()
}

#[cfg(test)]
mod tests {
    use ephemeris::{
        Ayanamsa, BodyPosition, EphError, HouseSystem, Houses, Precision, SiderealMode,
    };

    use super::*;
    use crate::fake::FakeEph;
    use crate::panchang::{compute_panchang, karana_name, tithi_no, KaranaName};
    use crate::settings::{NodeKind, ZodiacMode};

    const JD: f64 = 2_451_545.0;
    /// Bir sınırın analitik değerine izin verilen uzaklık: 0,2 sn.
    const TOL: f64 = 0.2 / 86_400.0;

    fn settings() -> CalcSettings {
        CalcSettings::default()
    }

    fn assert_partition<T: PartialEq + std::fmt::Debug>(s: &[Interval<T>], from: f64, to: f64) {
        assert!(!s.is_empty());
        assert_eq!(s[0].start_jd_ut, from);
        assert_eq!(s[s.len() - 1].end_jd_ut, to);
        for w in s.windows(2) {
            assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut, "no gap, no overlap");
            assert_ne!(w[0].value, w[1].value, "adjacent values differ");
        }
        assert!(s.iter().all(|i| i.start_jd_ut < i.end_jd_ut));
    }

    #[test]
    fn walker_finds_every_change_and_keeps_the_span() {
        // Değer günün tam sayı kısmıdır: sınırlar tam günlerdedir; adım 2,5 gün (bir adımda
        // iki ya da üç değişim).
        let mut eph = FakeEph::default();
        let s = walk(&mut eph, JD + 0.3, JD + 7.6, 2.5, |_, jd| {
            Ok((jd - JD).floor() as i64)
        })
        .unwrap();
        assert_partition(&s, JD + 0.3, JD + 7.6);
        assert_eq!(
            s.iter().map(|i| i.value).collect::<Vec<_>>(),
            (0..=7).collect::<Vec<_>>()
        );
        for (k, i) in s.iter().enumerate().skip(1) {
            let boundary = JD + k as f64;
            // Sınır, yeni değerin geçerli olduğu ilk örnektir: gerçek sınırdan önce değildir.
            assert!(
                i.start_jd_ut >= boundary && i.start_jd_ut - boundary <= TIMELINE_RESOLUTION_DAYS
            );
        }
        // Değişmeyen değer tek aralıktır.
        let flat = walk(&mut eph, JD, JD + 3.0, 1.0, |_, _| Ok(7u8)).unwrap();
        assert_eq!(
            flat,
            [Interval {
                value: 7,
                start_jd_ut: JD,
                end_jd_ut: JD + 3.0
            }]
        );
    }

    #[test]
    fn walker_rejects_bad_spans_and_reports_value_errors() {
        let mut eph = FakeEph::default();
        let ok = |_: &mut FakeEph, _: f64| Ok(0u8);
        assert_eq!(
            walk(&mut eph, JD, JD, 1.0, ok),
            Err(CoreError::InvalidMuhurtaInput("range"))
        );
        assert_eq!(
            walk(&mut eph, JD + 1.0, JD, 1.0, ok),
            Err(CoreError::InvalidMuhurtaInput("range"))
        );
        assert!(matches!(
            walk(&mut eph, f64::NAN, JD, 1.0, ok),
            Err(CoreError::InvalidJulianDay(_))
        ));
        assert_eq!(
            walk(&mut eph, 1e9, 1e9 + 1.0, 1.0, ok),
            Err(CoreError::InvalidJulianDay(1e9))
        );
        let failing = walk(&mut eph, JD, JD + 1.0, 1.0, |_, jd| {
            if jd > JD {
                Err(CoreError::NonFiniteResult("timeline"))
            } else {
                Ok(0u8)
            }
        });
        assert_eq!(failing, Err(CoreError::NonFiniteResult("timeline")));
    }

    #[test]
    fn map_merge_and_value_at() {
        let s = [
            Interval {
                value: 1u8,
                start_jd_ut: 0.0,
                end_jd_ut: 1.0,
            },
            Interval {
                value: 2,
                start_jd_ut: 1.0,
                end_jd_ut: 2.0,
            },
            Interval {
                value: 3,
                start_jd_ut: 2.0,
                end_jd_ut: 3.0,
            },
        ];
        let merged = map_merge(&s, |v| *v >= 2);
        assert_eq!(merged.len(), 2);
        assert_eq!((merged[1].start_jd_ut, merged[1].end_jd_ut), (1.0, 3.0));
        assert_eq!(value_at(&s, 0.0), Some(1));
        assert_eq!(value_at(&s, 1.0), Some(2));
        assert_eq!(value_at(&s, 2.999), Some(3));
        // Uçlar yarı açıktır; dizinin dışı boştur.
        assert_eq!(value_at(&s, 3.0), None);
        assert_eq!(value_at(&s, -0.1), None);
        assert_eq!(value_at::<u8>(&[], 1.0), None);
    }

    #[test]
    fn karana_and_tithi_series_follow_the_linear_elongation() {
        // Sahte sidereal: Güneş 346° + 0,98°/gün, Ay 71° + 13,2°/gün; uzanım 85° + 12,22°/gün.
        let mut eph = FakeEph::default();
        let (from, to) = (JD, JD + 3.0);
        let karanas = karana_series(&mut eph, &settings(), from, to).unwrap();
        assert_partition(&karanas, from, to);
        // 85° → 15. karana (Vishti); 90°'de 16., sonra her 6°'de bir.
        assert_eq!(karanas[0].value, 15);
        assert_eq!(karana_name(karanas[0].value), KaranaName::Vishti);
        for (k, i) in karanas.iter().enumerate().skip(1) {
            let boundary = JD + (90.0 + 6.0 * (k as f64 - 1.0) - 85.0) / 12.22;
            assert!((i.start_jd_ut - boundary).abs() < TOL, "karana {k}");
            assert_eq!(i.value, 15 + k as u8);
        }
        // 3 günde uzanım 36,66° ilerler: 85° → 121,66°; 90, 96, …, 120: altı sınır.
        assert_eq!(karanas.len(), 7);

        let tithis = tithi_series(&mut eph, &settings(), from, to).unwrap();
        assert_partition(&tithis, from, to);
        // 85° → 8. tithi; 96°, 108°, 120°'de değişir.
        assert_eq!(
            tithis.iter().map(|i| i.value).collect::<Vec<_>>(),
            [8, 9, 10, 11]
        );
        for (k, i) in tithis.iter().enumerate().skip(1) {
            let boundary = JD + (96.0 + 12.0 * (k as f64 - 1.0) - 85.0) / 12.22;
            assert!((i.start_jd_ut - boundary).abs() < TOL, "tithi {k}");
        }
        // Her tithi sınırı bir karana sınırıdır (aynı an, bit bit).
        for t in &tithis[1..] {
            assert!(karanas.iter().any(|k| k.start_jd_ut == t.start_jd_ut));
        }
        for k in 1..=60u8 {
            assert_eq!(
                tithi_of_karana(k),
                tithi_no((f64::from(k) - 0.5) * 6.0),
                "karana {k}"
            );
        }
    }

    #[test]
    fn nakshatra_and_yoga_series_follow_the_linear_motion() {
        let mut eph = FakeEph::default();
        let (from, to) = (JD, JD + 2.0);
        let n = nakshatra_series(&mut eph, &settings(), from, to).unwrap();
        assert_partition(&n, from, to);
        // Ay 71°: Ardra (6.) 2. pada; padalar 73°20′, 76°40′, 80° (Punarvasu), … 'de değişir.
        assert_eq!(
            n[0].value,
            NakshatraPada {
                nakshatra: 6,
                pada: 2
            }
        );
        let expected = [(6, 3), (6, 4), (7, 1), (7, 2), (7, 3), (7, 4), (8, 1)];
        for (k, (nak, pada)) in expected.into_iter().enumerate() {
            let i = &n[k + 1];
            assert_eq!((i.value.nakshatra, i.value.pada), (nak, pada));
            let boundary = JD + (220.0 / 3.0 + 10.0 / 3.0 * k as f64 - 71.0) / 13.2;
            assert!((i.start_jd_ut - boundary).abs() < TOL, "pada {k}");
        }
        // 2 günde 26,4°: 71° → 97,4°; son sınır 96°40′ (Pushya 2. pada).
        assert_eq!(n.len(), 9);

        let y = nitya_yoga_series(&mut eph, &settings(), from, to).unwrap();
        assert_partition(&y, from, to);
        // Toplam 57° + 14,18°/gün: 5. yoga; 66°40′ ve 80°'de değişir.
        assert_eq!(y.iter().map(|i| i.value).collect::<Vec<_>>(), [5, 6, 7]);
        assert!((y[1].start_jd_ut - (JD + (200.0 / 3.0 - 57.0) / 14.18)).abs() < TOL);
        assert!((y[2].start_jd_ut - (JD + (80.0 - 57.0) / 14.18)).abs() < TOL);
    }

    #[test]
    fn series_agree_with_the_birth_panchang_at_sampled_moments() {
        // Çapraz denetim (ileri modüller spec'i §10 "Test"): aralıkların içinden örneklenen
        // anlarda `compute_panchang` aynı değerleri verir; aralığın sonu, `compute_panchang`'ın
        // bitiş anıyla 1 saniye içinde örtüşür.
        let s = settings();
        let (from, to) = (JD - 5.0, JD + 5.0);
        let mut eph = FakeEph::default();
        let tithis = tithi_series(&mut eph, &s, from, to).unwrap();
        let karanas = karana_series(&mut eph, &s, from, to).unwrap();
        let nakshatras = nakshatra_series(&mut eph, &s, from, to).unwrap();
        let yogas = nitya_yoga_series(&mut eph, &s, from, to).unwrap();
        let varas = vara_series(&mut eph, &s, 41.0, 29.0, from, to).unwrap();
        assert_partition(&varas, from, to);
        let second = 1.0 / 86_400.0;
        let mut checked = 0;
        for k in 0..200 {
            let jd = from + 0.013 + 0.0497 * f64::from(k);
            let p = compute_panchang(&mut eph, jd, 41.0, 29.0, &s).unwrap();
            let (Some(t), Some(ka), Some(n), Some(y), Some(v)) = (
                tithis.iter().find(|i| i.contains(jd)),
                karanas.iter().find(|i| i.contains(jd)),
                nakshatras.iter().find(|i| i.contains(jd)),
                yogas.iter().find(|i| i.contains(jd)),
                varas.iter().find(|i| i.contains(jd)),
            ) else {
                panic!("sample {k} outside the series");
            };
            assert_eq!(t.value, p.tithi.no, "tithi at {jd}");
            assert_eq!(ka.value, p.karana.no, "karana at {jd}");
            assert_eq!(
                (n.value.nakshatra, n.value.pada),
                (p.nakshatra.no, p.nakshatra.pada)
            );
            assert_eq!(y.value, p.yoga.no, "yoga at {jd}");
            assert_eq!(v.value, p.vara, "vara at {jd}");
            // Kırpılmamış uçlar bitiş anıyla örtüşür.
            if t.end_jd_ut < to {
                assert!((t.end_jd_ut - p.tithi.ends_jd_ut).abs() <= second);
                checked += 1;
            }
            if ka.end_jd_ut < to {
                assert!((ka.end_jd_ut - p.karana.ends_jd_ut).abs() <= second);
            }
            if y.end_jd_ut < to {
                assert!((y.end_jd_ut - p.yoga.ends_jd_ut).abs() <= second);
            }
            // `compute_panchang` nakshatranın bitişini verir: son padanın sonu.
            let nak_end = nakshatras
                .iter()
                .rfind(|i| i.value.nakshatra == n.value.nakshatra)
                .map(|i| i.end_jd_ut)
                .unwrap();
            if nak_end < to {
                assert!((nak_end - p.nakshatra.ends_jd_ut).abs() <= second);
            }
        }
        assert!(checked > 150, "{checked}");
    }

    #[test]
    fn graha_sign_series_follow_the_rasi_frame() {
        let (from, to) = (JD, JD + 40.0);
        // Sidereal kip: Güneş 346° + 0,98°/gün; 360°'de (14,29. gün) Koç'a, 30°'de Boğa'ya.
        let mut eph = FakeEph::default();
        let sun = graha_sign_series(&mut eph, &settings(), Graha::Sun, from, to).unwrap();
        assert_partition(&sun, from, to);
        assert_eq!(
            sun.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Sign::Pisces, Sign::Aries]
        );
        assert!((sun[1].start_jd_ut - (JD + 14.0 / 0.98)).abs() < TOL);
        // Tropikal rasi kipinde burç tropikal boylamdandır: Güneş 10° (Koç), 30°'de Boğa'ya.
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..settings()
        };
        let sun = graha_sign_series(&mut eph, &tropical, Graha::Sun, from, to).unwrap();
        assert_eq!(
            sun.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Sign::Aries, Sign::Taurus]
        );
        assert!((sun[1].start_jd_ut - (JD + 20.0 / 0.98)).abs() < TOL);
        // Nakshatra iki kipte de sidereal'dir.
        let a = graha_nakshatra_series(&mut eph, &settings(), Graha::Moon, from, to).unwrap();
        let b = graha_nakshatra_series(&mut eph, &tropical, Graha::Moon, from, to).unwrap();
        assert_eq!(a, b);
        assert_partition(&a, from, to);
        // Ay 71° + 13,2°/gün: 40 günde 528° → 39 nakshatra sınırı ve bir fazlası kadar aralık.
        assert_eq!(a.len(), 40);
        assert_eq!(a[0].value, 6);
        assert_eq!(a[1].value, 7);

        // Geri giden graha burca geriden girer: Jüpiter sidereal 226° − 0,1°/gün; 210°'de
        // (160. gün) Akrep'ten Terazi'ye.
        let jup = graha_sign_series(&mut eph, &settings(), Graha::Jupiter, JD, JD + 200.0).unwrap();
        assert_eq!(
            jup.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Sign::Scorpio, Sign::Libra]
        );
        assert!((jup[1].start_jd_ut - (JD + 160.0)).abs() < TOL);

        // Ketu, Rahu'nun karşısıdır; düğüm türü ayardan gelir. Ortalama düğüm sidereal 96° −
        // 0,053°/gün: Rahu 90°'de (113,2. gün) Yengeç'ten İkizler'e, Ketu aynı anda Oğlak'tan
        // Yay'a.
        let rahu = graha_sign_series(&mut eph, &settings(), Graha::Rahu, JD, JD + 150.0).unwrap();
        let ketu = graha_sign_series(&mut eph, &settings(), Graha::Ketu, JD, JD + 150.0).unwrap();
        assert_eq!(
            rahu.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Sign::Cancer, Sign::Gemini]
        );
        assert_eq!(
            ketu.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Sign::Capricorn, Sign::Sagittarius]
        );
        assert_eq!(rahu[1].start_jd_ut, ketu[1].start_jd_ut);
        assert!((rahu[1].start_jd_ut - (JD + 6.0 / 0.053)).abs() < TOL);
        // Gerçek düğüm ayarı başka cismi sorar (sahte: 101° sidereal, ileri gider).
        let true_node = CalcSettings {
            node: NodeKind::True,
            ..settings()
        };
        let rahu = graha_sign_series(&mut eph, &true_node, Graha::Rahu, JD, JD + 150.0).unwrap();
        assert_eq!(rahu.len(), 1);
        assert_eq!(rahu[0].value, Sign::Cancer);
    }

    /// Boylamı `lon0 + hız × t + genlik × sin(2π t / dönem)` olan tek cisimli sahte ephemeris
    /// (retro dönemleri olan bir graha); Güneş sabit hızla ilerler.
    struct WobbleEph {
        lon0: f64,
        speed: f64,
        amplitude: f64,
        period: f64,
        sun0: f64,
    }

    impl WobbleEph {
        fn phase(&self, jd: f64) -> f64 {
            std::f64::consts::TAU * (jd - JD) / self.period
        }
    }

    impl Ephemeris for WobbleEph {
        fn body(&mut self, jd: f64, body: Body, _: Zodiac) -> Result<BodyPosition, EphError> {
            let (lon, lon_speed) = if body == Body::Sun {
                (self.sun0 + (jd - JD), 1.0)
            } else {
                (
                    self.lon0 + self.speed * (jd - JD) + self.amplitude * self.phase(jd).sin(),
                    self.speed
                        + self.amplitude * std::f64::consts::TAU / self.period
                            * self.phase(jd).cos(),
                )
            };
            Ok(BodyPosition {
                lon: normalize(lon),
                lat: 0.0,
                distance_au: 1.0,
                lon_speed,
                lat_speed: 0.0,
                dist_speed: 0.0,
                precision: Precision::SwissEph,
            })
        }

        fn ayanamsa(&mut self, _: f64, _: SiderealMode) -> Result<Ayanamsa, EphError> {
            Ok(Ayanamsa {
                deg: 0.0,
                precision: Precision::SwissEph,
            })
        }

        fn houses(
            &mut self,
            _: f64,
            _: f64,
            _: f64,
            _: HouseSystem,
            _: Zodiac,
        ) -> Result<Houses, EphError> {
            Err(EphError::Calc("wobble: no houses".into()))
        }

        fn sun_rise_set(
            &mut self,
            _: f64,
            _: RiseSet,
            _: f64,
            _: f64,
            _: SunDisc,
        ) -> Result<Option<f64>, EphError> {
            Ok(None)
        }

        fn swe_version(&self) -> String {
            "wobble".into()
        }
    }

    #[test]
    fn motion_series_finds_the_stations_where_the_speed_changes_sign() {
        // Hız 0,5 + 1,0 × cos(2π t / 100): cos = −0,5 olduğunda sıfır; t = 100/3 ve 200/3.
        let mut eph = WobbleEph {
            lon0: 100.0,
            speed: 0.5,
            amplitude: 100.0 / std::f64::consts::TAU,
            period: 100.0,
            sun0: 0.0,
        };
        let s = settings();
        let m = graha_motion_series(&mut eph, &s, Graha::Mars, JD, JD + 100.0).unwrap();
        assert_partition(&m, JD, JD + 100.0);
        assert_eq!(
            m.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Motion::Direct, Motion::Retrograde, Motion::Direct]
        );
        assert!((m[1].start_jd_ut - (JD + 100.0 / 3.0)).abs() < TOL);
        assert!((m[2].start_jd_ut - (JD + 200.0 / 3.0)).abs() < TOL);
        // Dönüş anında hız işaret değiştirir.
        for station in [m[1].start_jd_ut, m[2].start_jd_ut] {
            let before = eph
                .body(station - 0.01, Body::Mars, Zodiac::Tropical)
                .unwrap();
            let after = eph
                .body(station + 0.01, Body::Mars, Zodiac::Tropical)
                .unwrap();
            assert!(before.lon_speed * after.lon_speed < 0.0);
        }
        // Retro graha burç sınırını üç kez geçer: 100° → 130,4° (retro dönüşü) → 119,6° (düz
        // dönüş) → 148,5°; 120° sınırı ileri, geri ve yeniden ileri geçilir.
        let signs = graha_sign_series(&mut eph, &s, Graha::Mars, JD, JD + 99.0).unwrap();
        assert_eq!(
            signs.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Sign::Cancer, Sign::Leo, Sign::Cancer, Sign::Leo]
        );
        for i in &signs[1..] {
            let lon = eph
                .body(i.start_jd_ut, Body::Mars, Zodiac::Tropical)
                .unwrap()
                .lon;
            // Giriş anında boylam sınırın 1″ içindedir.
            assert!((lon - 120.0).abs() < 1.0 / 3600.0, "{lon}");
        }
        // Yalnızca retro olabilen beş graha.
        for g in [Graha::Sun, Graha::Moon, Graha::Rahu, Graha::Ketu] {
            assert_eq!(
                graha_motion_series(&mut eph, &s, g, JD, JD + 1.0),
                Err(CoreError::InvalidMuhurtaInput("graha"))
            );
        }
    }

    #[test]
    fn combust_series_uses_the_chart_orbs_and_the_retrograde_orb() {
        // Güneş 0° + 1°/gün; Venüs 20° sabit (hız 0, düz): fark 20 − t. Venüs'ün orbu 10°:
        // t = 10'da yanar, t = 30'da (fark −10°) çıkar.
        let mut eph = WobbleEph {
            lon0: 20.0,
            speed: 0.0,
            amplitude: 0.0,
            period: 100.0,
            sun0: 0.0,
        };
        let s = settings();
        let c = combust_series(&mut eph, &s, Graha::Venus, JD, JD + 40.0).unwrap();
        assert_partition(&c, JD, JD + 40.0);
        assert_eq!(
            c.iter().map(|i| i.value).collect::<Vec<_>>(),
            [false, true, false]
        );
        assert!((c[1].start_jd_ut - (JD + 10.0)).abs() < TOL);
        assert!((c[2].start_jd_ut - (JD + 30.0)).abs() < TOL);
        // Geri giden Venüs'ün orbu 8°'dir: hız −0,001°/gün; fark 20 − 1,001 t.
        let mut retro = WobbleEph {
            lon0: 20.0,
            speed: -0.001,
            amplitude: 0.0,
            period: 100.0,
            sun0: 0.0,
        };
        let c = combust_series(&mut retro, &s, Graha::Venus, JD, JD + 40.0).unwrap();
        assert!((c[1].start_jd_ut - (JD + 12.0 / 1.001)).abs() < TOL);
        assert!((c[2].start_jd_ut - (JD + 28.0 / 1.001)).abs() < TOL);
        // Güneş ve düğümler yanmaz.
        let sun = combust_series(&mut eph, &s, Graha::Sun, JD, JD + 40.0).unwrap();
        assert_eq!(sun.len(), 1);
        assert!(!sun[0].value);
    }

    #[test]
    fn lagna_series_turns_through_twelve_signs_a_day() {
        // Sahte lagna tropikal 70° (sidereal 46°, Boğa), günde 360°: her 2 saatte bir burç.
        let mut eph = FakeEph {
            asc_speed: 360.0,
            ..FakeEph::default()
        };
        let s = settings();
        let l = lagna_sign_series(&mut eph, &s, 41.0, 29.0, JD, JD + 1.0).unwrap();
        assert_partition(&l, JD, JD + 1.0);
        assert_eq!(l.len(), 13);
        assert_eq!(l[0].value, Sign::Taurus);
        assert_eq!(l[12].value, Sign::Taurus);
        for (k, i) in l.iter().enumerate().skip(1) {
            assert_eq!(i.value, Sign::from_index(1 + k));
            // İlk sınır 60°'de: 14° / 360 gün; sonrakiler 30°'de bir.
            let boundary = JD + (14.0 + 30.0 * (k as f64 - 1.0)) / 360.0;
            assert!((i.start_jd_ut - boundary).abs() < TOL, "lagna {k}");
            assert_eq!(
                lagna_sign_at(&mut eph, &s, i.start_jd_ut, 41.0, 29.0).unwrap(),
                i.value
            );
        }
        // Tropikal rasi kipinde lagna burcu tropikal boylamdandır (70°: İkizler).
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..settings()
        };
        let l = lagna_sign_series(&mut eph, &tropical, 41.0, 29.0, JD, JD + 0.05).unwrap();
        assert_eq!(l[0].value, Sign::Gemini);
        assert_eq!(
            lagna_sign_series(&mut eph, &s, 91.0, 29.0, JD, JD + 1.0),
            Err(CoreError::InvalidMuhurtaInput("place"))
        );
    }

    #[test]
    fn sun_days_and_the_vara_series_run_from_sunrise_to_sunrise() {
        // Sahte gün doğumu her gün n + 0,67, batımı n + 0,15.
        let mut eph = FakeEph::default();
        let s = settings();
        let (from, to) = (JD, JD + 3.0);
        let days = sun_days(&mut eph, &s, 41.0, 29.0, from, to).unwrap();
        assert_eq!(days.len(), 4);
        assert!(sun_days_cover(&days, from, to));
        for (k, d) in days.iter().enumerate() {
            let rise = 2_451_544.67 + k as f64;
            assert!((d.sunrise_jd_ut - rise).abs() < 1e-9);
            assert!((d.sunset_jd_ut - (rise + 0.48)).abs() < 1e-9);
            assert!((d.next_sunrise_jd_ut - (rise + 1.0)).abs() < 1e-9);
        }
        // 2451544,67 (1 Ocak 2000, yerel ortalama) Cumartesi.
        assert_eq!(
            days.iter().map(|d| d.vara).collect::<Vec<_>>(),
            [Vara::Saturday, Vara::Sunday, Vara::Monday, Vara::Tuesday]
        );
        let varas = vara_series(&mut eph, &s, 41.0, 29.0, from, to).unwrap();
        assert_partition(&varas, from, to);
        assert_eq!(varas.len(), 4);
        assert!((varas[1].start_jd_ut - 2_451_545.67).abs() < 1e-9);
        assert_eq!(varas, vara_of_sun_days(&days, from, to));
        // Aralık tam bir doğuşta biterse sonraki gün listeye girmez.
        let exact = sun_days(&mut eph, &s, 41.0, 29.0, JD, days[1].next_sunrise_jd_ut).unwrap();
        assert_eq!(exact.len(), 2);
    }

    #[test]
    fn without_a_sunrise_the_vara_changes_at_six_local_mean_time() {
        let mut eph = FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        let s = settings();
        let (from, to) = (JD, JD + 2.0);
        assert!(sun_days(&mut eph, &s, 41.0, 29.0, from, to)
            .unwrap()
            .is_empty());
        let varas = vara_series(&mut eph, &s, 41.0, 29.0, from, to).unwrap();
        assert_partition(&varas, from, to);
        // Boylam 29°: 06:00 yerel ortalama saat = 04:04 UT = n + 0,6694.
        assert_eq!(
            varas.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Vara::Saturday, Vara::Sunday, Vara::Monday]
        );
        let six = 2_451_545.75 - 29.0 / 360.0;
        assert!((varas[1].start_jd_ut - six).abs() < TOL);
        assert!((varas[2].start_jd_ut - (six + 1.0)).abs() < TOL);
        for i in &varas {
            let mid = 0.5 * (i.start_jd_ut + i.end_jd_ut);
            assert_eq!(
                compute_panchang(&mut eph, mid, 41.0, 29.0, &s)
                    .unwrap()
                    .vara,
                i.value
            );
        }
    }

    #[test]
    fn a_gap_between_sunrises_leaves_the_day_out_and_falls_back_to_the_slow_path() {
        // Doğuşlar: gün 0, 1, 2 ve sonra 5 (üç günlük boşluk: kutup gecesi). Batışlar her gün
        // n + 0,15'tedir; sahte ephemeris doğuş listesi verilince 26 saat ileriye bakar.
        let rises = vec![JD - 0.3, JD + 0.7, JD + 1.7, JD + 4.7, JD + 5.7];
        let mut eph = FakeEph {
            sunrises: Some(rises),
            ..FakeEph::default()
        };
        let s = settings();
        let (from, to) = (JD, JD + 5.5);
        let days = sun_days(&mut eph, &s, 41.0, 29.0, from, to).unwrap();
        // [−0,3; 0,7), [0,7; 1,7) ve boşluktan sonra [4,7; 5,7): 1,7'de başlayan günün sonraki
        // doğuşu yoktur.
        assert_eq!(
            days.iter()
                .map(|d| ((d.sunrise_jd_ut - JD) * 10.0).round() as i64)
                .collect::<Vec<_>>(),
            [-3, 7, 47]
        );
        assert!(!sun_days_cover(&days, from, to));
        let varas = vara_series(&mut eph, &s, 41.0, 29.0, from, to).unwrap();
        assert_partition(&varas, from, to);
        for k in 0..110 {
            let jd = from + 0.01 + 0.05 * f64::from(k);
            let v = varas.iter().find(|i| i.contains(jd)).unwrap().value;
            assert_eq!(
                v,
                compute_panchang(&mut eph, jd, 41.0, 29.0, &s).unwrap().vara,
                "vara at +{}",
                jd - JD
            );
        }
    }
}
````


- [ ] **Step 6: Dışa aktarımlar, hata eşlemesi ve üç dilde ileti**

````diff
diff --git a/app/src-tauri/src/commands.rs b/app/src-tauri/src/commands.rs
index a60407b..91ca7bd 100644
--- a/app/src-tauri/src/commands.rs
+++ b/app/src-tauri/src/commands.rs
@@ -85,6 +85,9 @@ impl From<CoreError> for CommandError {
             CoreError::InvalidTransitInput(field) => {
                 Self::new("invalid_transit_input", &[("field", field.to_owned())])
             }
+            CoreError::InvalidMuhurtaInput(field) => {
+                Self::new("invalid_muhurta_input", &[("field", field.to_owned())])
+            }
         }
     }
 }
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 2efd535..76cfeff 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -443,6 +443,7 @@ export const en: Messages = {
     invalid_dasha_input: "Invalid dasha request: {{field}}.",
     panchang_search: "Panchang boundary not found ({{what}}).",
     invalid_transit_input: "Invalid transit request: {{field}}.",
+    invalid_muhurta_input: "Invalid muhurta request: {{field}}.",
     atlas_unavailable: "Place search is unavailable ({{reason}}). You can enter latitude, longitude and time zone by hand.",
     atlas_sqlite: "The place atlas could not be read ({{detail}}).",
     atlas_io: "The place atlas file could not be read ({{detail}}).",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index b9ff315..dd0bd05 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -443,6 +443,7 @@ export const fr: Messages = {
     invalid_dasha_input: "Requête de dasha invalide : {{field}}.",
     panchang_search: "Limite du panchang introuvable ({{what}}).",
     invalid_transit_input: "Requête de transit invalide : {{field}}.",
+    invalid_muhurta_input: "Requête de muhurta invalide : {{field}}.",
     atlas_unavailable: "La recherche de lieux est indisponible ({{reason}}). Vous pouvez saisir la latitude, la longitude et le fuseau à la main.",
     atlas_sqlite: "L’atlas des lieux est illisible ({{detail}}).",
     atlas_io: "Le fichier de l’atlas est illisible ({{detail}}).",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 372980b..393b509 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -442,6 +442,7 @@ export const tr = {
     invalid_dasha_input: "Dasa isteği geçersiz: {{field}}.",
     panchang_search: "Panchang sınırı bulunamadı ({{what}}).",
     invalid_transit_input: "Transit isteği geçersiz: {{field}}.",
+    invalid_muhurta_input: "Muhurta isteği geçersiz: {{field}}.",
     atlas_unavailable: "Yer araması kullanılamıyor ({{reason}}). Enlem, boylam ve saat dilimini elle girebilirsiniz.",
     atlas_sqlite: "Yer atlası okunamadı ({{detail}}).",
     atlas_io: "Yer atlası dosyası okunamadı ({{detail}}).",
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 8f26241..fb3518a 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -6,6 +6,7 @@
 //! (Alt proje 3.1): `positions` (ortak girdi), `lordship`, `nature` ve `drishti`. Jaimini
 //! çekirdeği (Alt proje 3.2) ve chara dasha (Alt proje 3.3): `jaimini`. Shadbala ve Bhava bala
 //! (Alt proje 3.4): `shadbala`. Yogalar (Alt proje 3.5): `yogas`. Uyum (Alt proje 3.6): `kuta`.
+//! Muhurta (Alt proje 3.7): `timeline` (aralık dizileri).
 
 pub mod ashtakavarga;
 mod chart;
@@ -93,6 +94,11 @@ pub use shadbala::{
     GRAHA_WAR_ORB_DEG, HORA_CYCLE, MAX_SUNRISE_GAP_DAYS, SAPTAVARGA, SHADBALA_GRAHAS,
     SHADBALA_METHOD, VIRUPAS_PER_RUPA, WAR_GRAHAS,
 };
+pub use timeline::{
+    combust_series, graha_motion_series, graha_nakshatra_series, graha_sign_series, karana_series,
+    lagna_sign_at, lagna_sign_series, nakshatra_series, nitya_yoga_series, sun_days, tithi_series,
+    vara_series, Interval, Motion, NakshatraPada, SunDay, STATION_GRAHAS, TIMELINE_RESOLUTION_DAYS,
+};
 pub use transit::{compute_transits, TransitGraha, TransitSnapshot};
 pub use varga::{
     all_vargas, is_vargottama, varga_chart, varga_position, Varga, VargaChart, VargaPlacement,
````


- [ ] **Step 7: Testleri koş**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -1
cargo test -p jyotish-core --lib timeline 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --lib 2>&1 | grep -E "^test result"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: fmt çıktısız; clippy `Finished`; `14 passed; 0 failed; 0 ignored; 0 measured; 307 filtered out` (13 birim testi ve `timeline_properties`); sonra `321 passed` (lib), `8 passed` (golden), `13 passed` (`modules_golden`); `jyotish-desk` lib `46 passed`; Vitest `75 passed (75)`, `738 passed (738)`. `series_agree_with_the_birth_panchang_at_sampled_moments` düşerse dizinin değeri `compute_panchang`'ınkinden başka bir sorgudan hesaplanıyordur (D2): beklenen değer değil, dizinin değer fonksiyonu düzeltilir.

- [ ] **Step 8: Commit**

```bash
git add crates/jyotish-core/src app/src-tauri/src/commands.rs app/ui/i18n
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): timeline core with interval series for the panchang limbs, signs, lagna, graha ingresses, stations and sun days; invalid_muhurta_input error" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: iki `grep` çıktısız (çıktısız `grep` 1 ile çıkar; beklenen budur).

---


### Task 2: Çekirdek: günün dilimleri, hora tablosu, sınıflar, tara ve chandra bala; el fikstürleri

Spec §10 "Gün panchangı" (sekizde bir dilimler, Abhijit, hora tablosu), "Sınıflar ve işaretler", "Kişiye bağlı etkenler", "Test" (el örnekleri; ikinci bağımsız yazım; property'ler); §13 katman 1–3; K3, K9; D10–D14. Hiçbir mevcut davranış değişmez. Bu görevin fonksiyonları ephemeris görmez; gün panchangı ve arama onları Task 3'te kullanır.

Dilim sıraları, tithi grupları, nakshatra doğaları ve iki elverişli küme aşağıdaki dosyalarda yazılıdır; yürüten ajan bunları **değiştirmez ve başka bir yazılımın çıktısıyla ya da çevrimiçi bir tabloyla karşılaştırmaz** (K9). Basılı bir kaynakla karşılaştırma Task 9 Step 6'nın **[insan]** maddesidir (D11).

**Files:**
- Create: `crates/jyotish-core/src/muhurta/mod.rs`, `crates/jyotish-core/src/muhurta/segments.rs`, `tests/golden/modules/muhurta/REVIEW.md`, `tests/golden/modules/muhurta/*.json` (7 dosya)
- Modify: `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/proptests.rs`, `crates/jyotish-core/tests/modules_golden.rs`, `tests/golden/modules/README.md`
- Test: `segments.rs`'in `mod tests`'i (8 test), `proptests.rs` `muhurta_properties` (2 test), `modules_golden.rs` (2 test)

**Interfaces:**
- Consumes (v1, Plan 17, Plan 19, Task 1): `Vara`, `KaranaName`, `karana_name`, `Sign` (`house_from`), `Graha`, `shadbala::{hora_lord, vara_lord, HORA_CYCLE}`, `kuta::tara_number`, `timeline::{Interval, NakshatraPada, SunDay}`, `CoreError::InvalidMuhurtaInput`.
- Produces (`jyotish_core::muhurta`; yıldızlılar kökten de dışa aktarılır):
  - Tipler: `MuhurtaFlag`\* (10 değer; `ALL`, `is_day_segment()`), `NatalMoon { nakshatra, sign }`\* (`validate`), `DaySegments { eighths, rahu, yamaganda, gulika, abhijit, abhijit_weekday_exception, horas }`\*, `TithiGroup`\* (5; `ALL`), `NakshatraNature`\* (7; `ALL`), `TaraBala { number, favourable }`\*, `ChandraBala { house, favourable }`\*.
  - Sabitler: `MUHURTA_METHOD`\*, `DAY_EIGHTHS`, `DAY_MUHURTAS`, `ABHIJIT_MUHURTA`, `HORAS_PER_DAY`, `RAHU_SEGMENTS`, `NAKSHATRA_NATURES`, `VYATIPATA_YOGA`, `VAIDHRITI_YOGA`, `PANCHAKA_START_DEG`, `TARA_FAVOURABLE`, `CHANDRA_FAVOURABLE`.
  - Fonksiyonlar: `rahu_segment(Vara) -> u8`, `yamaganda_segment`, `gulika_segment`, `equal_parts(start, end, parts) -> Vec<Interval<u8>>`, `day_segments(&SunDay) -> DaySegments`\*, `tithi_group(u8) -> TithiGroup`\*, `nakshatra_nature(u8) -> NakshatraNature`\*, `is_rikta(tithi)`, `is_vishti(karana)`, `is_panchaka(NakshatraPada)`, `tara_bala(natal_nakshatra, day_nakshatra) -> TaraBala`\*, `chandra_bala(natal_sign, moon_sign) -> ChandraBala`\*.

- [ ] **Step 1: El örnekleri (yedi dosya), okuma kuralları ve README**

Her örnek sentetik bir gündür: `input.day` gün doğumunu, gün batımını ve sonraki gün doğumunu dakika olarak (aynı gece yarısından) ve varayı verir; `input.tithi`, `input.moon` ve `input.natal` sınıfların ve kişiye bağlı etkenlerin girdisidir. Değerler kuraldan elle türetilmiştir (Self-Review madde 5): **dosyalar olduğu gibi yazılır; bir değer motorla uyuşmazsa motor düzeltilir, dosya değil.**

**Yeni dosya:** `tests/golden/modules/muhurta/REVIEW.md`

````markdown
# Muhurta el örnekleri: okuma kuralları, tablolar, onay bekleyen noktalar ve gözden geçirme kaydı

Kaynak: ileri modüller tasarım belgesi §10 (gün panchangı, sınıflar ve işaretler, kişiye bağlı
etkenler) ve §7'nin hora kuralı. Bu dizindeki her JSON dosyası sentetik bir gündür: gün doğumu,
gün batımı ve sonraki gün doğumu dakika olarak (aynı gece yarısından sayılır; sonraki gün doğumu
1440'tan büyüktür), günün varası, o günün tithisi ve Ay'ı (nakshatra, burç) ve bir doğum Ay'ı.
Ephemeris ve konum yoktur. `derivation` dilimleri, Abhijit'i, hora düzenini, tithi grubunu,
nakshatra doğasını, tara ve chandra balayı sırayla yazar. Değerler kuraldan elle türetilmiştir;
motor çıktısı buraya kopyalanmaz (K9). Bir değer motorla uyuşmazsa önce kural yeniden okunur.

## Okuma kuralları

1. **Sekizde bir dilimler.** Gündüz (gün doğumu → gün batımı) sekiz eşit dilime bölünür; dilimler
   1'den 8'e sayılır. Dilimin uzunluğu gündüzün sekizde biridir (saat sabit değildir).
2. **İşaretli üç dilim** varaya göre aşağıdaki tablodan okunur. Üçü her gün üç ayrı dilimdir.
3. **Abhijit.** Gündüz on beş eşit muhurtaya bölünür; Abhijit sekizincisidir (yedinci muhurtanın
   bitişinden sekizincinin bitişine). Çarşamba günü `abhijit_weekday_exception` `true`'dur; dilim
   yine verilir.
4. **Hora.** Gün doğumundan sonraki gün doğumuna yirmi dört eşit dilim. İlk horanın lordu günün
   lordudur (Pazar Güneş, Pazartesi Ay, Salı Mars, Çarşamba Merkür, Perşembe Jüpiter, Cuma Venüs,
   Cumartesi Satürn); sonrakiler Satürn, Jüpiter, Mars, Güneş, Venüs, Merkür, Ay döngüsünde
   sırayla gelir (döngü Ay'dan sonra Satürn'e döner).
5. **Tithi grubu.** Tithinin paksha içi sırası (16–30 için 15 çıkarılır) aşağıdaki tablodan.
6. **Nakshatra doğası** aşağıdaki listeden.
7. **Tara bala.** Doğum nakshatrasından günün nakshatrasına sayım başlangıcı da sayar (aynı
   nakshatra 1); numara sayımın dokuzlu döngüdeki yeridir (10 → 1, 19 → 1). Adlar: 1 Janma,
   2 Sampat, 3 Vipat, 4 Kshema, 5 Pratyak, 6 Sadhana, 7 Naidhana, 8 Mitra, 9 Parama Mitra.
   Elverişli küme {2, 4, 6, 8, 9}'dur; Janma (1) bu kümede değildir.
8. **Chandra bala.** Geçiş Ay'ının burcunun doğum Ay burcundan sayısı (aynı burç 1). Elverişli
   küme {1, 3, 6, 7, 10, 11}'dir.

## Tablolar

Motorun kullandığı tablolar (`crates/jyotish-core/src/muhurta/segments.rs`).

**İşaretli dilimler** (dilim numarası, 1–8)

| Vara | Rahu dilimi | Yamaganda | Gulika |
|---|---|---|---|
| Pazar | 8 | 5 | 7 |
| Pazartesi | 2 | 4 | 6 |
| Salı | 7 | 3 | 5 |
| Çarşamba | 5 | 2 | 4 |
| Perşembe | 6 | 1 | 3 |
| Cuma | 4 | 7 | 2 |
| Cumartesi | 3 | 6 | 1 |

**Tithi grupları** (paksha içi sıra)

| Grup | Tithiler |
|---|---|
| Nanda | 1, 6, 11 |
| Bhadra | 2, 7, 12 |
| Jaya | 3, 8, 13 |
| Rikta | 4, 9, 14 |
| Purna | 5, 10, 15 |

**Nakshatra doğaları**

| Doğa | Kimlik | Nakshatralar |
|---|---|---|
| Dhruva (sabit) | `Fixed` | Rohini, Uttara Phalguni, Uttara Ashadha, Uttara Bhadrapada |
| Chara (yürük) | `Movable` | Punarvasu, Swati, Shravana, Dhanishta, Shatabhisha |
| Ugra (sert) | `Fierce` | Bharani, Magha, Purva Phalguni, Purva Ashadha, Purva Bhadrapada |
| Mishra (karışık) | `Mixed` | Krittika, Vishakha |
| Kshipra (çevik) | `Swift` | Ashwini, Pushya, Hasta |
| Mridu (yumuşak) | `Soft` | Mrigashira, Chitra, Anuradha, Revati |
| Tikshna (keskin) | `Sharp` | Ardra, Ashlesha, Jyeshtha, Mula |

## Sahibin onayını bekleyen klasik tablolar ve bilinen varyant noktaları

Yukarıdaki tablolar ve kümeler tasarım belgesinde yazılıdır ve muhurta derlemelerinin yaygın
biçimidir; bu depoya bellekten aktarılmıştır, basılı bir kaynakla karşılaştırılmamıştır.
Aşağıdaki noktalarda gelenekler ayrışır. Karşılaştırma bir insanın işidir: elinde basılı bir
panchang ya da muhurta derlemesi bulunan kişi her satırı denetler, kaynağı genel adıyla (alıntı
yapmadan) ve sonucu aşağıdaki tabloya yazar. Fark bulunursa tablo ayrı bir commit'le düzeltilir
ve bu dizindeki örnekler kuraldan yeniden türetilir.

| No | Nokta | Bu depodaki okuma | Bilinen diğer okuma |
|---|---|---|---|
| 1 | Rahu dilimi, Yamaganda ve Gulika sıraları | Yukarıdaki tablo; dilimler gerçek gün doğumu ile batımı arasının sekizde biridir | Sabit 06:00–18:00 günü ve 90 dakikalık dilimler kullanan takvimler; Gulika için dilimin başı yerine ortasını ya da sonunu alan ve gece Gulika'sını da veren gelenekler |
| 2 | Abhijit | Gündüzün on beş muhurtasının sekizincisi | Yerel öğlenin iki yanında sabit 24'er dakika; Çarşamba günü hiç verilmemesi |
| 3 | Hora uzunluğu | Gün doğumundan sonraki gün doğumuna 24 eşit dilim (Shadbala'nın hora lordu ile aynı kural) | Gündüzün ve gecenin ayrı ayrı on ikiye bölünmesi (gündüz ve gece horaları eşit değildir) |
| 4 | Nakshatra doğaları | Yukarıdaki yedi liste (27 nakshatra) | Abhijit'i 28. nakshatra olarak çevik sınıfa ekleyen listeler |
| 5 | Tara bala | Elverişli {2, 4, 6, 8, 9}; Janma elverişli sayılmaz | Janma'yı nötr ya da işe göre kabul edilir sayan okumalar; yalnızca ilk dokuzluğu ölçü alanlar |
| 6 | Chandra bala | Elverişli {1, 3, 6, 7, 10, 11} | Shukla pakshada 2, 5 ve 9'u da ekleyen okumalar |
| 7 | Panchaka | Ay sidereal 300°–360° (Dhanishta'nın üçüncü padasından Revati'nin sonuna) | Dhanishta'nın başından (293°20′) başlatan takvimler |
| 8 | Vishti | Yalnızca karana (sekiz yarım tithi) | Vishti'nin "bulunduğu yer" kuralıyla bir bölümünü dışarıda tutan okumalar |

| Denetleyen | Tarih | Kaynak (genel ad) | Sonuç |
|---|---|---|---|
| — | — | — | Bekliyor: sahibine devredildi (Plan 20 Task 9) |

## Gözden geçirme kaydı

Gözden geçiren (motoru yazan dışında biri) her örnekte yalnızca `input`'tan başlar, motoru
çalıştırmadan ve Rust kaynağını okumadan sekizde biri, üç işaretli dilimi, Abhijit'i, hora
uzunluğunu ve yirmi dört lordu, tithi grubunu, nakshatra doğasını, tara ve chandra balayı
tasarım belgesinden, yukarıdaki okuma kurallarından ve tablolardan yeniden hesaplar; sonucunu
`expected` ile karşılaştırır ve aşağıya adını, tarihini ve bulduğu farkları yazar.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
| — | — | Bekliyor (Plan 20 Task 9 Step 5) |
````

**Yeni dosya:** `tests/golden/modules/muhurta/sunday-twelve-hour-day-parama-mitra.json`

````json
{
  "id": "sunday-twelve-hour-day-parama-mitra",
  "source": "hand",
  "rule_ref": "Muhurta derlemelerinin gün dilimleri, hora düzeni, tithi grupları, nakshatra doğaları, tara ve chandra bala (alıntı yapılmaz); ileri modüller spec'i §10",
  "derivation": [
    "Gün: Pazar. Gün doğumu 06:00, batımı 18:00, sonraki gün doğumu 06:00 (dakika: 360, 1080, 1800).",
    "Gündüz 1080 − 360 = 720 dakika; sekizde biri 720 / 8 = 90 dakika.",
    "Rahu dilimi: Pazar günü 8. dilim → 360 + 7 × 90 = 990 … 1080 (16:30–18:00).",
    "Yamaganda: Pazar günü 5. dilim → 720 … 810 (12:00–13:30).",
    "Gulika: Pazar günü 7. dilim → 900 … 990 (15:00–16:30).",
    "Abhijit: gündüzün on beşte biri 720 / 15 = 48 dakika; sekizinci muhurta 360 + 7 × 48 = 696 … 744 (11:36–12:24). Çarşamba istisnası: yok.",
    "Hora: gün doğumundan sonraki gün doğumuna 1800 − 360 = 1440 dakika; 24'te biri 60 dakika. İlk hora günün lordunundur (Güneş); sonrakiler Satürn, Jüpiter, Mars, Güneş, Venüs, Merkür, Ay döngüsünde ilerler: Güneş, Venüs, Merkür, Ay, Satürn, Jüpiter, Mars, Güneş ….",
    "Tithi 1: Shukla pakshanın 1. tithisi → Nanda grubu.",
    "Günün nakshatrası 1. (Ashwini): Swift (çevik).",
    "Tara bala: doğum nakshatrası 2. (Bharani); ondan günün nakshatrasına sayım (başlangıç dahil) dokuzlu döngüde 9 → Parama Mitra; elverişli küme Sampat, Kshema, Sadhana, Mitra, Parama Mitra → elverişli.",
    "Chandra bala: doğum Ay burcu Koç, geçiş Ay'ı Koç → 1. ev; elverişli evler 1, 3, 6, 7, 10, 11 → elverişli."
  ],
  "input": {
    "day": {"vara": "Sunday", "sunrise_min": 360.0, "sunset_min": 1080.0, "next_sunrise_min": 1800.0},
    "tithi": 1,
    "moon": {"nakshatra": 1, "sign": "Aries"},
    "natal": {"nakshatra": 2, "sign": "Aries"}
  },
  "expected": {
    "eighth_min": 90.0,
    "rahu": {"segment": 8, "from_min": 990.0, "to_min": 1080.0},
    "yamaganda": {"segment": 5, "from_min": 720.0, "to_min": 810.0},
    "gulika": {"segment": 7, "from_min": 900.0, "to_min": 990.0},
    "abhijit": {"from_min": 696.0, "to_min": 744.0},
    "abhijit_weekday_exception": false,
    "hora_min": 60.0,
    "hora_lords": ["Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury"],
    "tithi_group": "Nanda",
    "nakshatra_nature": "Swift",
    "tara": {"number": 9, "favourable": true},
    "chandra": {"house": 1, "favourable": true}
  }
}
````

**Yeni dosya:** `tests/golden/modules/muhurta/monday-six-to-six-rahu-second-segment.json`

````json
{
  "id": "monday-six-to-six-rahu-second-segment",
  "source": "hand",
  "rule_ref": "Muhurta derlemelerinin gün dilimleri, hora düzeni, tithi grupları, nakshatra doğaları, tara ve chandra bala (alıntı yapılmaz); ileri modüller spec'i §10",
  "derivation": [
    "Gün: Pazartesi. Gün doğumu 06:00, batımı 18:00, sonraki gün doğumu 06:00 (dakika: 360, 1080, 1800).",
    "Gündüz 1080 − 360 = 720 dakika; sekizde biri 720 / 8 = 90 dakika.",
    "Rahu dilimi: Pazartesi günü 2. dilim → 360 + 1 × 90 = 450 … 540 (07:30–09:00).",
    "Yamaganda: Pazartesi günü 4. dilim → 630 … 720 (10:30–12:00).",
    "Gulika: Pazartesi günü 6. dilim → 810 … 900 (13:30–15:00).",
    "Abhijit: gündüzün on beşte biri 720 / 15 = 48 dakika; sekizinci muhurta 360 + 7 × 48 = 696 … 744 (11:36–12:24). Çarşamba istisnası: yok.",
    "Hora: gün doğumundan sonraki gün doğumuna 1800 − 360 = 1440 dakika; 24'te biri 60 dakika. İlk hora günün lordunundur (Ay); sonrakiler Satürn, Jüpiter, Mars, Güneş, Venüs, Merkür, Ay döngüsünde ilerler: Ay, Satürn, Jüpiter, Mars, Güneş, Venüs, Merkür, Ay ….",
    "Tithi 9: Shukla pakshanın 9. tithisi → Rikta grubu.",
    "Günün nakshatrası 5. (Mrigashira): Soft (yumuşak).",
    "Tara bala: doğum nakshatrası 4. (Rohini); ondan günün nakshatrasına sayım (başlangıç dahil) dokuzlu döngüde 2 → Sampat; elverişli küme Sampat, Kshema, Sadhana, Mitra, Parama Mitra → elverişli.",
    "Chandra bala: doğum Ay burcu Akrep, geçiş Ay'ı Boğa → 7. ev; elverişli evler 1, 3, 6, 7, 10, 11 → elverişli."
  ],
  "input": {
    "day": {"vara": "Monday", "sunrise_min": 360.0, "sunset_min": 1080.0, "next_sunrise_min": 1800.0},
    "tithi": 9,
    "moon": {"nakshatra": 5, "sign": "Taurus"},
    "natal": {"nakshatra": 4, "sign": "Scorpio"}
  },
  "expected": {
    "eighth_min": 90.0,
    "rahu": {"segment": 2, "from_min": 450.0, "to_min": 540.0},
    "yamaganda": {"segment": 4, "from_min": 630.0, "to_min": 720.0},
    "gulika": {"segment": 6, "from_min": 810.0, "to_min": 900.0},
    "abhijit": {"from_min": 696.0, "to_min": 744.0},
    "abhijit_weekday_exception": false,
    "hora_min": 60.0,
    "hora_lords": ["Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter"],
    "tithi_group": "Rikta",
    "nakshatra_nature": "Soft",
    "tara": {"number": 2, "favourable": true},
    "chandra": {"house": 7, "favourable": true}
  }
}
````

**Yeni dosya:** `tests/golden/modules/muhurta/tuesday-fourteen-hour-day-short-horas.json`

````json
{
  "id": "tuesday-fourteen-hour-day-short-horas",
  "source": "hand",
  "rule_ref": "Muhurta derlemelerinin gün dilimleri, hora düzeni, tithi grupları, nakshatra doğaları, tara ve chandra bala (alıntı yapılmaz); ileri modüller spec'i §10",
  "derivation": [
    "Gün: Salı. Gün doğumu 05:00, batımı 19:00, sonraki gün doğumu 04:36 (dakika: 300, 1140, 1716).",
    "Gündüz 1140 − 300 = 840 dakika; sekizde biri 840 / 8 = 105 dakika.",
    "Rahu dilimi: Salı günü 7. dilim → 300 + 6 × 105 = 930 … 1035 (15:30–17:15).",
    "Yamaganda: Salı günü 3. dilim → 510 … 615 (08:30–10:15).",
    "Gulika: Salı günü 5. dilim → 720 … 825 (12:00–13:45).",
    "Abhijit: gündüzün on beşte biri 840 / 15 = 56 dakika; sekizinci muhurta 300 + 7 × 56 = 692 … 748 (11:32–12:28). Çarşamba istisnası: yok.",
    "Hora: gün doğumundan sonraki gün doğumuna 1716 − 300 = 1416 dakika; 24'te biri 59 dakika. İlk hora günün lordunundur (Mars); sonrakiler Satürn, Jüpiter, Mars, Güneş, Venüs, Merkür, Ay döngüsünde ilerler: Mars, Güneş, Venüs, Merkür, Ay, Satürn, Jüpiter, Mars ….",
    "Tithi 18: Krishna pakshanın 3. tithisi → Jaya grubu.",
    "Günün nakshatrası 19. (Mula): Sharp (keskin).",
    "Tara bala: doğum nakshatrası 26. (Uttara Bhadrapada); ondan günün nakshatrasına sayım (başlangıç dahil) dokuzlu döngüde 3 → Vipat; elverişli küme Sampat, Kshema, Sadhana, Mitra, Parama Mitra → elverişli değil.",
    "Chandra bala: doğum Ay burcu Aslan, geçiş Ay'ı Yay → 5. ev; elverişli evler 1, 3, 6, 7, 10, 11 → elverişli değil."
  ],
  "input": {
    "day": {"vara": "Tuesday", "sunrise_min": 300.0, "sunset_min": 1140.0, "next_sunrise_min": 1716.0},
    "tithi": 18,
    "moon": {"nakshatra": 19, "sign": "Sagittarius"},
    "natal": {"nakshatra": 26, "sign": "Leo"}
  },
  "expected": {
    "eighth_min": 105.0,
    "rahu": {"segment": 7, "from_min": 930.0, "to_min": 1035.0},
    "yamaganda": {"segment": 3, "from_min": 510.0, "to_min": 615.0},
    "gulika": {"segment": 5, "from_min": 720.0, "to_min": 825.0},
    "abhijit": {"from_min": 692.0, "to_min": 748.0},
    "abhijit_weekday_exception": false,
    "hora_min": 59.0,
    "hora_lords": ["Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus"],
    "tithi_group": "Jaya",
    "nakshatra_nature": "Sharp",
    "tara": {"number": 3, "favourable": false},
    "chandra": {"house": 5, "favourable": false}
  }
}
````

**Yeni dosya:** `tests/golden/modules/muhurta/wednesday-ten-hour-day-abhijit-exception.json`

````json
{
  "id": "wednesday-ten-hour-day-abhijit-exception",
  "source": "hand",
  "rule_ref": "Muhurta derlemelerinin gün dilimleri, hora düzeni, tithi grupları, nakshatra doğaları, tara ve chandra bala (alıntı yapılmaz); ileri modüller spec'i §10",
  "derivation": [
    "Gün: Çarşamba. Gün doğumu 07:00, batımı 17:00, sonraki gün doğumu 07:24 (dakika: 420, 1020, 1884).",
    "Gündüz 1020 − 420 = 600 dakika; sekizde biri 600 / 8 = 75 dakika.",
    "Rahu dilimi: Çarşamba günü 5. dilim → 420 + 4 × 75 = 720 … 795 (12:00–13:15).",
    "Yamaganda: Çarşamba günü 2. dilim → 495 … 570 (08:15–09:30).",
    "Gulika: Çarşamba günü 4. dilim → 645 … 720 (10:45–12:00).",
    "Abhijit: gündüzün on beşte biri 600 / 15 = 40 dakika; sekizinci muhurta 420 + 7 × 40 = 700 … 740 (11:40–12:20). Çarşamba istisnası: var.",
    "Hora: gün doğumundan sonraki gün doğumuna 1884 − 420 = 1464 dakika; 24'te biri 61 dakika. İlk hora günün lordunundur (Merkür); sonrakiler Satürn, Jüpiter, Mars, Güneş, Venüs, Merkür, Ay döngüsünde ilerler: Merkür, Ay, Satürn, Jüpiter, Mars, Güneş, Venüs, Merkür ….",
    "Tithi 30: Krishna pakshanın 15. tithisi → Purna grubu.",
    "Günün nakshatrası 26. (Uttara Bhadrapada): Fixed (sabit).",
    "Tara bala: doğum nakshatrası 23. (Dhanishta); ondan günün nakshatrasına sayım (başlangıç dahil) dokuzlu döngüde 4 → Kshema; elverişli küme Sampat, Kshema, Sadhana, Mitra, Parama Mitra → elverişli.",
    "Chandra bala: doğum Ay burcu Oğlak, geçiş Ay'ı Balık → 3. ev; elverişli evler 1, 3, 6, 7, 10, 11 → elverişli."
  ],
  "input": {
    "day": {"vara": "Wednesday", "sunrise_min": 420.0, "sunset_min": 1020.0, "next_sunrise_min": 1884.0},
    "tithi": 30,
    "moon": {"nakshatra": 26, "sign": "Pisces"},
    "natal": {"nakshatra": 23, "sign": "Capricorn"}
  },
  "expected": {
    "eighth_min": 75.0,
    "rahu": {"segment": 5, "from_min": 720.0, "to_min": 795.0},
    "yamaganda": {"segment": 2, "from_min": 495.0, "to_min": 570.0},
    "gulika": {"segment": 4, "from_min": 645.0, "to_min": 720.0},
    "abhijit": {"from_min": 700.0, "to_min": 740.0},
    "abhijit_weekday_exception": true,
    "hora_min": 61.0,
    "hora_lords": ["Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn"],
    "tithi_group": "Purna",
    "nakshatra_nature": "Fixed",
    "tara": {"number": 4, "favourable": true},
    "chandra": {"house": 3, "favourable": true}
  }
}
````

**Yeni dosya:** `tests/golden/modules/muhurta/thursday-sixteen-hour-day-yamaganda-first.json`

````json
{
  "id": "thursday-sixteen-hour-day-yamaganda-first",
  "source": "hand",
  "rule_ref": "Muhurta derlemelerinin gün dilimleri, hora düzeni, tithi grupları, nakshatra doğaları, tara ve chandra bala (alıntı yapılmaz); ileri modüller spec'i §10",
  "derivation": [
    "Gün: Perşembe. Gün doğumu 04:00, batımı 20:00, sonraki gün doğumu 04:00 (dakika: 240, 1200, 1680).",
    "Gündüz 1200 − 240 = 960 dakika; sekizde biri 960 / 8 = 120 dakika.",
    "Rahu dilimi: Perşembe günü 6. dilim → 240 + 5 × 120 = 840 … 960 (14:00–16:00).",
    "Yamaganda: Perşembe günü 1. dilim → 240 … 360 (04:00–06:00).",
    "Gulika: Perşembe günü 3. dilim → 480 … 600 (08:00–10:00).",
    "Abhijit: gündüzün on beşte biri 960 / 15 = 64 dakika; sekizinci muhurta 240 + 7 × 64 = 688 … 752 (11:28–12:32). Çarşamba istisnası: yok.",
    "Hora: gün doğumundan sonraki gün doğumuna 1680 − 240 = 1440 dakika; 24'te biri 60 dakika. İlk hora günün lordunundur (Jüpiter); sonrakiler Satürn, Jüpiter, Mars, Güneş, Venüs, Merkür, Ay döngüsünde ilerler: Jüpiter, Mars, Güneş, Venüs, Merkür, Ay, Satürn, Jüpiter ….",
    "Tithi 12: Shukla pakshanın 12. tithisi → Bhadra grubu.",
    "Günün nakshatrası 15. (Swati): Movable (yürük).",
    "Tara bala: doğum nakshatrası 8. (Pushya); ondan günün nakshatrasına sayım (başlangıç dahil) dokuzlu döngüde 8 → Mitra; elverişli küme Sampat, Kshema, Sadhana, Mitra, Parama Mitra → elverişli.",
    "Chandra bala: doğum Ay burcu Boğa, geçiş Ay'ı Terazi → 6. ev; elverişli evler 1, 3, 6, 7, 10, 11 → elverişli."
  ],
  "input": {
    "day": {"vara": "Thursday", "sunrise_min": 240.0, "sunset_min": 1200.0, "next_sunrise_min": 1680.0},
    "tithi": 12,
    "moon": {"nakshatra": 15, "sign": "Libra"},
    "natal": {"nakshatra": 8, "sign": "Taurus"}
  },
  "expected": {
    "eighth_min": 120.0,
    "rahu": {"segment": 6, "from_min": 840.0, "to_min": 960.0},
    "yamaganda": {"segment": 1, "from_min": 240.0, "to_min": 360.0},
    "gulika": {"segment": 3, "from_min": 480.0, "to_min": 600.0},
    "abhijit": {"from_min": 688.0, "to_min": 752.0},
    "abhijit_weekday_exception": false,
    "hora_min": 60.0,
    "hora_lords": ["Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun"],
    "tithi_group": "Bhadra",
    "nakshatra_nature": "Movable",
    "tara": {"number": 8, "favourable": true},
    "chandra": {"house": 6, "favourable": true}
  }
}
````

**Yeni dosya:** `tests/golden/modules/muhurta/friday-eight-hour-day-full-moon.json`

````json
{
  "id": "friday-eight-hour-day-full-moon",
  "source": "hand",
  "rule_ref": "Muhurta derlemelerinin gün dilimleri, hora düzeni, tithi grupları, nakshatra doğaları, tara ve chandra bala (alıntı yapılmaz); ileri modüller spec'i §10",
  "derivation": [
    "Gün: Cuma. Gün doğumu 08:00, batımı 16:00, sonraki gün doğumu 08:00 (dakika: 480, 960, 1920).",
    "Gündüz 960 − 480 = 480 dakika; sekizde biri 480 / 8 = 60 dakika.",
    "Rahu dilimi: Cuma günü 4. dilim → 480 + 3 × 60 = 660 … 720 (11:00–12:00).",
    "Yamaganda: Cuma günü 7. dilim → 840 … 900 (14:00–15:00).",
    "Gulika: Cuma günü 2. dilim → 540 … 600 (09:00–10:00).",
    "Abhijit: gündüzün on beşte biri 480 / 15 = 32 dakika; sekizinci muhurta 480 + 7 × 32 = 704 … 736 (11:44–12:16). Çarşamba istisnası: yok.",
    "Hora: gün doğumundan sonraki gün doğumuna 1920 − 480 = 1440 dakika; 24'te biri 60 dakika. İlk hora günün lordunundur (Venüs); sonrakiler Satürn, Jüpiter, Mars, Güneş, Venüs, Merkür, Ay döngüsünde ilerler: Venüs, Merkür, Ay, Satürn, Jüpiter, Mars, Güneş, Venüs ….",
    "Tithi 15: Shukla pakshanın 15. tithisi → Purna grubu.",
    "Günün nakshatrası 3. (Krittika): Mixed (karışık).",
    "Tara bala: doğum nakshatrası 7. (Punarvasu); ondan günün nakshatrasına sayım (başlangıç dahil) dokuzlu döngüde 6 → Sadhana; elverişli küme Sampat, Kshema, Sadhana, Mitra, Parama Mitra → elverişli.",
    "Chandra bala: doğum Ay burcu İkizler, geçiş Ay'ı Boğa → 12. ev; elverişli evler 1, 3, 6, 7, 10, 11 → elverişli değil."
  ],
  "input": {
    "day": {"vara": "Friday", "sunrise_min": 480.0, "sunset_min": 960.0, "next_sunrise_min": 1920.0},
    "tithi": 15,
    "moon": {"nakshatra": 3, "sign": "Taurus"},
    "natal": {"nakshatra": 7, "sign": "Gemini"}
  },
  "expected": {
    "eighth_min": 60.0,
    "rahu": {"segment": 4, "from_min": 660.0, "to_min": 720.0},
    "yamaganda": {"segment": 7, "from_min": 840.0, "to_min": 900.0},
    "gulika": {"segment": 2, "from_min": 540.0, "to_min": 600.0},
    "abhijit": {"from_min": 704.0, "to_min": 736.0},
    "abhijit_weekday_exception": false,
    "hora_min": 60.0,
    "hora_lords": ["Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon"],
    "tithi_group": "Purna",
    "nakshatra_nature": "Mixed",
    "tara": {"number": 6, "favourable": true},
    "chandra": {"house": 12, "favourable": false}
  }
}
````

**Yeni dosya:** `tests/golden/modules/muhurta/saturday-eleven-hour-day-gulika-first.json`

````json
{
  "id": "saturday-eleven-hour-day-gulika-first",
  "source": "hand",
  "rule_ref": "Muhurta derlemelerinin gün dilimleri, hora düzeni, tithi grupları, nakshatra doğaları, tara ve chandra bala (alıntı yapılmaz); ileri modüller spec'i §10",
  "derivation": [
    "Gün: Cumartesi. Gün doğumu 06:30, batımı 17:30, sonraki gün doğumu 06:30 (dakika: 390, 1050, 1830).",
    "Gündüz 1050 − 390 = 660 dakika; sekizde biri 660 / 8 = 82,5 dakika.",
    "Rahu dilimi: Cumartesi günü 3. dilim → 390 + 2 × 82,5 = 555 … 637,5 (09:15–10:37:30).",
    "Yamaganda: Cumartesi günü 6. dilim → 802,5 … 885 (13:22:30–14:45).",
    "Gulika: Cumartesi günü 1. dilim → 390 … 472,5 (06:30–07:52:30).",
    "Abhijit: gündüzün on beşte biri 660 / 15 = 44 dakika; sekizinci muhurta 390 + 7 × 44 = 698 … 742 (11:38–12:22). Çarşamba istisnası: yok.",
    "Hora: gün doğumundan sonraki gün doğumuna 1830 − 390 = 1440 dakika; 24'te biri 60 dakika. İlk hora günün lordunundur (Satürn); sonrakiler Satürn, Jüpiter, Mars, Güneş, Venüs, Merkür, Ay döngüsünde ilerler: Satürn, Jüpiter, Mars, Güneş, Venüs, Merkür, Ay, Satürn ….",
    "Tithi 24: Krishna pakshanın 9. tithisi → Rikta grubu.",
    "Günün nakshatrası 10. (Magha): Fierce (sert).",
    "Tara bala: doğum nakshatrası 4. (Rohini); ondan günün nakshatrasına sayım (başlangıç dahil) dokuzlu döngüde 7 → Naidhana; elverişli küme Sampat, Kshema, Sadhana, Mitra, Parama Mitra → elverişli değil.",
    "Chandra bala: doğum Ay burcu Kova, geçiş Ay'ı Aslan → 7. ev; elverişli evler 1, 3, 6, 7, 10, 11 → elverişli."
  ],
  "input": {
    "day": {"vara": "Saturday", "sunrise_min": 390.0, "sunset_min": 1050.0, "next_sunrise_min": 1830.0},
    "tithi": 24,
    "moon": {"nakshatra": 10, "sign": "Leo"},
    "natal": {"nakshatra": 4, "sign": "Aquarius"}
  },
  "expected": {
    "eighth_min": 82.5,
    "rahu": {"segment": 3, "from_min": 555.0, "to_min": 637.5},
    "yamaganda": {"segment": 6, "from_min": 802.5, "to_min": 885.0},
    "gulika": {"segment": 1, "from_min": 390.0, "to_min": 472.5},
    "abhijit": {"from_min": 698.0, "to_min": 742.0},
    "abhijit_weekday_exception": false,
    "hora_min": 60.0,
    "hora_lords": ["Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars"],
    "tithi_group": "Rikta",
    "nakshatra_nature": "Fierce",
    "tara": {"number": 7, "favourable": false},
    "chandra": {"house": 7, "favourable": true}
  }
}
````

````diff
diff --git a/tests/golden/modules/README.md b/tests/golden/modules/README.md
index 623b5ba..06889a2 100644
--- a/tests/golden/modules/README.md
+++ b/tests/golden/modules/README.md
@@ -54,6 +54,7 @@ doğum anı ve 365,25 günlük yıl kullanır, tarihleri el yıllarının biriki
 | `shadbala` | `grahas` (yedi graha: sthana balanın beş kalemi ve `sthana`, `dig`, zaman gücünün sekiz kalemi ve `temporal`, `cheshta`, `naisargika`, `drik`, `total`, `rank`, `war`), `time_lords`, `bhavas` (on iki bhava: `house`, `lord`, `lord_bala`, `dig`, `drishti`, `total`, `rank`); saat bilinmiyorsa `expected` `null` | 4 |
 | `yogas` | `hits` (raporun bütün kayıtları, katalog sırasıyla: `id`, `grahas`, `houses`, `relations`, `conditions`), `absent` (bu örneğin "oluşmayan durum" olarak gösterdiği kurallar), `cancelled` (iptal örnekleri), `evaluated`, `skipped_time_unknown` | 15 |
 | `kuta` | `scores` (sekiz kuta, tablo sırasıyla: `kuta`, `score`, `groom_class`, `bride_class`), `total`, `bhakoot_exception`, `nadi_exception`, `kuja_dosha` (`[damat rolü, gelin rolü]`), `overlay` (`groom` ve `bride`: dokuzar satır `[diğer tarafın lagnasından ev ya da null, Ay'ından ev]`), `approximate` | 8 |
+| `muhurta` | `eighth_min`, `rahu`, `yamaganda`, `gulika` (`segment`, `from_min`, `to_min`), `abhijit` (`from_min`, `to_min`), `abhijit_weekday_exception`, `hora_min`, `hora_lords` (24), `tithi_group`, `nakshatra_nature`, `tara` (`number`, `favourable`), `chandra` (`house`, `favourable`) | 7 |
 
 Shadbala örnekleri çalışma kâğıdıdır (tasarım belgesi §7 "Test"): `shadbala/WORKSHEET.md` her
 kalemin aritmetiğini adım adım yazar, JSON dosyaları aynı girdiyi ve sonuçları taşır. Tolerans
@@ -78,4 +79,15 @@ haritanın iki rolde de örneklerde bulunduğunu sabitler. Okuma kuralları, mot
 tabloların basılı kaynaklarla karşılaştırma kaydı ve bağımsız gözden geçirmenin kaydı
 `kuta/REVIEW.md`'dedir.
 
+Muhurta örnekleri (tasarım belgesi §10 "Test") konum ve graha taşımaz: girdileri sentetik bir
+gündür. `input.day` gün doğumunu, gün batımını ve sonraki gün doğumunu dakika olarak (aynı gece
+yarısından) ve varayı; `input.tithi` günün tithisini (1–30); `input.moon` günün Ay'ını;
+`input.natal` doğum Ay'ını (ikisi de `nakshatra` ve `sign`) verir. Diğer modüllerin `lagna_lon` ve
+`grahas` alanları bu örneklerde yazılmaz. Beklenen saatler de dakikadır. Ayrı bir test örneklerin
+yedi varanın her birini, beş tithi grubunu, yedi nakshatra doğasını, elverişli ve elverişsiz tara
+ile chandra balayı, Çarşamba istisnasını ve 59, 60 ve 61 dakikalık horaları kapsadığını, ve
+tasarım belgesinin örneğinin (06:00–18:00 Pazartesi) aralarında bulunduğunu sabitler. Okuma
+kuralları, motorun tabloları, sahibin onayını bekleyen varyant noktaları ve bağımsız gözden
+geçirmenin kaydı `muhurta/REVIEW.md`'dedir.
+
 Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
````


- [ ] **Step 2: Fikstür düzeneği, değişmezler ve modül kaydı (testler)**

`modules_golden.rs`'te ortak denetimler `load_raw`'a taşınır (girdinin biçimi tip parametresidir; mevcut `load` onu çağırır ve davranışı değişmez); muhurta örnekleri kendi girdi tipiyle yüklenir. `muhurta_properties` sekiz dilimin gündüzü, yirmi dört horanın günü tam böldüğünü ister.

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index fb3518a..4866496 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -22,6 +22,7 @@ mod houses;
 pub mod jaimini;
 pub mod kuta;
 pub mod lordship;
+pub mod muhurta;
 pub mod nature;
 pub mod panchang;
 pub mod positions;
diff --git a/crates/jyotish-core/src/proptests.rs b/crates/jyotish-core/src/proptests.rs
index 8db4517..c44308f 100644
--- a/crates/jyotish-core/src/proptests.rs
+++ b/crates/jyotish-core/src/proptests.rs
@@ -1560,3 +1560,79 @@ mod timeline_properties {
         }
     }
 }
+
+mod muhurta_properties {
+    use proptest::prelude::*;
+
+    use crate::muhurta::day_segments;
+    use crate::panchang::Vara;
+    use crate::shadbala::vara_lord;
+    use crate::timeline::SunDay;
+
+    fn vara() -> impl Strategy<Value = Vara> {
+        (0usize..7).prop_map(|i| Vara::ALL[i])
+    }
+
+    /// Gün: doğuş, 0,2–0,8 gün süren gündüz ve 0,9–1,1 gün sonraki doğuş.
+    fn sun_day() -> impl Strategy<Value = SunDay> {
+        (
+            2_400_000.0f64..2_500_000.0,
+            0.2f64..0.8,
+            0.9f64..1.1,
+            vara(),
+        )
+            .prop_map(|(rise, daylight, length, vara)| SunDay {
+                sunrise_jd_ut: rise,
+                sunset_jd_ut: rise + daylight,
+                next_sunrise_jd_ut: rise + length,
+                vara,
+            })
+    }
+
+    proptest! {
+        #[test]
+        fn eight_segments_split_the_daytime_and_the_three_marked_ones_differ(day in sun_day()) {
+            let s = day_segments(&day);
+            prop_assert_eq!(s.eighths.len(), 8);
+            prop_assert_eq!(s.eighths[0].start_jd_ut, day.sunrise_jd_ut);
+            prop_assert_eq!(s.eighths[7].end_jd_ut, day.sunset_jd_ut);
+            for w in s.eighths.windows(2) {
+                prop_assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
+                prop_assert!(w[0].start_jd_ut < w[0].end_jd_ut);
+            }
+            let marked = [s.rahu, s.yamaganda, s.gulika];
+            for m in marked {
+                prop_assert_eq!(m, s.eighths[usize::from(m.value) - 1]);
+            }
+            prop_assert!(marked[0].value != marked[1].value);
+            prop_assert!(marked[1].value != marked[2].value);
+            prop_assert!(marked[0].value != marked[2].value);
+            // Abhijit gündüzün tam ortasını içerir ve on beşte biri kadar sürer.
+            let noon = 0.5 * (day.sunrise_jd_ut + day.sunset_jd_ut);
+            prop_assert!(s.abhijit.contains(noon));
+            let daylight = day.sunset_jd_ut - day.sunrise_jd_ut;
+            prop_assert!((s.abhijit.days() - daylight / 15.0).abs() < 1e-9);
+            prop_assert_eq!(s.abhijit_weekday_exception, day.vara == Vara::Wednesday);
+        }
+
+        #[test]
+        fn twenty_four_horas_split_the_day_and_start_with_the_day_lord(day in sun_day()) {
+            let s = day_segments(&day);
+            prop_assert_eq!(s.horas.len(), 24);
+            prop_assert_eq!(s.horas[0].start_jd_ut, day.sunrise_jd_ut);
+            prop_assert_eq!(s.horas[23].end_jd_ut, day.next_sunrise_jd_ut);
+            prop_assert_eq!(s.horas[0].value, vara_lord(day.vara));
+            let length = (day.next_sunrise_jd_ut - day.sunrise_jd_ut) / 24.0;
+            for (k, h) in s.horas.iter().enumerate() {
+                prop_assert!((h.days() - length).abs() < 1e-9);
+                if k > 0 {
+                    prop_assert_eq!(s.horas[k - 1].end_jd_ut, h.start_jd_ut);
+                }
+                // Yedi saatte bir aynı lord.
+                if k >= 7 {
+                    prop_assert_eq!(h.value, s.horas[k - 7].value);
+                }
+            }
+        }
+    }
+}
diff --git a/crates/jyotish-core/tests/modules_golden.rs b/crates/jyotish-core/tests/modules_golden.rs
index c2eba4d..b7f64a0 100644
--- a/crates/jyotish-core/tests/modules_golden.rs
+++ b/crates/jyotish-core/tests/modules_golden.rs
@@ -6,13 +6,16 @@
 use std::collections::BTreeMap;
 use std::path::PathBuf;
 
+use jyotish_core::muhurta::{rahu_segment, yamaganda_segment};
 use jyotish_core::{
-    argala_rows, arudha_padas, chara_dasha, chara_dasha_order, chara_dasha_sign_years,
-    chara_karakas, co_lord_choice, drishti_table, house_lord, houses_owned, jaimini_lord,
-    jaimini_report, karakamsha, kuja_dosha, kuta_class_ids, kuta_report, moon_is_waxing, nature,
-    normalize, shadbala, sign_dasha_children, yoga_rule, yogas, AdvancedSettings, ArgalaStatus,
-    Graha, GrahaWar, KarakaRole, KujaDosha, Kuta, KutaRole, MoonDayChange, MoonPoint, Nature,
-    Positions, ShadbalaAstro, Sign, SignDashaPeriod, TimeLords, YogaRelation, ASPECTING_GRAHAS,
+    argala_rows, arudha_padas, chandra_bala, chara_dasha, chara_dasha_order,
+    chara_dasha_sign_years, chara_karakas, co_lord_choice, day_segments, drishti_table, house_lord,
+    houses_owned, jaimini_lord, jaimini_report, karakamsha, kuja_dosha, kuta_class_ids,
+    kuta_report, moon_is_waxing, nakshatra_nature, nature, normalize, shadbala,
+    sign_dasha_children, tara_bala, tithi_group, yoga_rule, yogas, AdvancedSettings, ArgalaStatus,
+    ChandraBala, Graha, GrahaWar, Interval, KarakaRole, KujaDosha, Kuta, KutaRole, MoonDayChange,
+    MoonPoint, NakshatraNature, NatalMoon, Nature, Positions, ShadbalaAstro, Sign, SignDashaPeriod,
+    SunDay, TaraBala, TimeLords, TithiGroup, Vara, YogaRelation, ASPECTING_GRAHAS,
     CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD, KUTA_METHOD, SHADBALA_GRAHAS,
     SHADBALA_METHOD, YOGA_CATALOG, YOGA_CATALOG_VERSION, YOGA_CONDITIONS,
 };
@@ -27,6 +30,7 @@ const MIN_CHARA_DASHA: usize = 7;
 const MIN_SHADBALA: usize = 4;
 const MIN_YOGAS: usize = 15;
 const MIN_KUTA: usize = 8;
+const MIN_MUHURTA: usize = 7;
 /// El hesabı ile motor arasındaki en büyük fark, virupa.
 const VIRUPA_TOL: f64 = 1e-9;
 /// Shadbala çalışma kâğıdının toleransı, virupa (ileri modüller spec'i §7 "Test").
@@ -34,12 +38,12 @@ const WORKSHEET_TOL: f64 = 0.01;
 
 #[derive(Debug, Deserialize)]
 #[serde(deny_unknown_fields)]
-struct Fixture<X> {
+struct Fixture<X, I = Input> {
     id: String,
     source: String,
     rule_ref: String,
     derivation: Vec<String>,
-    input: Input,
+    input: I,
     expected: X,
 }
 
@@ -95,7 +99,12 @@ impl Input {
     }
 }
 
-fn load<X: DeserializeOwned>(module: &str, min: usize) -> Vec<(Fixture<X>, Positions)> {
+/// Bir modülün el örnekleri, ortak denetimleriyle: en az sayı, dosya adı, kaynak, gönderme ve
+/// türetme adımları. `I` girdinin biçimidir (çoğu modülde sentetik konumlar: `Input`).
+fn load_raw<X: DeserializeOwned, I: DeserializeOwned>(
+    module: &str,
+    min: usize,
+) -> Vec<Fixture<X, I>> {
     let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
         .join("../../tests/golden/modules")
         .join(module);
@@ -115,7 +124,7 @@ fn load<X: DeserializeOwned>(module: &str, min: usize) -> Vec<(Fixture<X>, Posit
         .map(|p| {
             let text =
                 std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
-            let f: Fixture<X> =
+            let f: Fixture<X, I> =
                 serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
             assert_eq!(
                 p.file_stem().and_then(|s| s.to_str()),
@@ -130,6 +139,15 @@ fn load<X: DeserializeOwned>(module: &str, min: usize) -> Vec<(Fixture<X>, Posit
                 "{}: derivation steps",
                 f.id
             );
+            f
+        })
+        .collect()
+}
+
+fn load<X: DeserializeOwned>(module: &str, min: usize) -> Vec<(Fixture<X>, Positions)> {
+    load_raw::<X, Input>(module, min)
+        .into_iter()
+        .map(|f| {
             let positions = f.input.positions(&f.id);
             (f, positions)
         })
@@ -1483,3 +1501,205 @@ fn kuta_fixtures_cover_the_required_cases() {
         }
     }
 }
+
+/// Muhurta örneklerinin girdisi: sentetik bir gün (dakika olarak gün doğumu, batımı ve sonraki
+/// gün doğumu; varası), o günün tithisi ve Ay'ı, ve doğum Ay'ı. Konum ve ephemeris yoktur.
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct MuhurtaInput {
+    day: MuhurtaDay,
+    tithi: u8,
+    moon: NatalMoon,
+    natal: NatalMoon,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct MuhurtaDay {
+    vara: Vara,
+    sunrise_min: f64,
+    sunset_min: f64,
+    next_sunrise_min: f64,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ExpectedSegment {
+    segment: u8,
+    from_min: f64,
+    to_min: f64,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ExpectedSpan {
+    from_min: f64,
+    to_min: f64,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct MuhurtaExpected {
+    eighth_min: f64,
+    rahu: ExpectedSegment,
+    yamaganda: ExpectedSegment,
+    gulika: ExpectedSegment,
+    abhijit: ExpectedSpan,
+    abhijit_weekday_exception: bool,
+    hora_min: f64,
+    hora_lords: Vec<Graha>,
+    tithi_group: TithiGroup,
+    nakshatra_nature: NakshatraNature,
+    tara: TaraBala,
+    chandra: ChandraBala,
+}
+
+const MINUTES_PER_DAY: f64 = 1440.0;
+/// El hesabı ile motor arasındaki en büyük fark, dakika.
+const MINUTE_TOL: f64 = 1e-6;
+
+fn muhurta_fixtures() -> Vec<Fixture<MuhurtaExpected, MuhurtaInput>> {
+    load_raw("muhurta", MIN_MUHURTA)
+}
+
+#[test]
+fn muhurta_hand_fixtures() {
+    for f in muhurta_fixtures() {
+        let id = &f.id;
+        let (i, x) = (&f.input, &f.expected);
+        // Dakika değerleri gün kesrine çevrilir (sentetik gün; Jülyen günü değildir).
+        let s = day_segments(&SunDay {
+            sunrise_jd_ut: i.day.sunrise_min / MINUTES_PER_DAY,
+            sunset_jd_ut: i.day.sunset_min / MINUTES_PER_DAY,
+            next_sunrise_jd_ut: i.day.next_sunrise_min / MINUTES_PER_DAY,
+            vara: i.day.vara,
+        });
+        let minutes = |t: f64| t * MINUTES_PER_DAY;
+        let close = |got: f64, want: f64, what: &str| {
+            assert!(
+                (minutes(got) - want).abs() < MINUTE_TOL,
+                "{id}: {what}: {} vs {want}",
+                minutes(got)
+            );
+        };
+        assert_eq!(s.eighths.len(), 8, "{id}");
+        for e in &s.eighths {
+            close(e.end_jd_ut - e.start_jd_ut, x.eighth_min, "eighth");
+        }
+        let segment = |got: &Interval<u8>, want: &ExpectedSegment, what: &str| {
+            assert_eq!(got.value, want.segment, "{id}: {what} segment");
+            close(got.start_jd_ut, want.from_min, what);
+            close(got.end_jd_ut, want.to_min, what);
+        };
+        segment(&s.rahu, &x.rahu, "rahu");
+        segment(&s.yamaganda, &x.yamaganda, "yamaganda");
+        segment(&s.gulika, &x.gulika, "gulika");
+        assert_eq!(s.abhijit.value, 8, "{id}: abhijit is the eighth muhurta");
+        close(s.abhijit.start_jd_ut, x.abhijit.from_min, "abhijit");
+        close(s.abhijit.end_jd_ut, x.abhijit.to_min, "abhijit");
+        assert_eq!(
+            s.abhijit_weekday_exception, x.abhijit_weekday_exception,
+            "{id}: abhijit weekday exception"
+        );
+        assert_eq!(x.hora_lords.len(), 24, "{id}: twenty-four hora lords");
+        let lords: Vec<Graha> = s.horas.iter().map(|h| h.value).collect();
+        assert_eq!(lords, x.hora_lords, "{id}: hora lords");
+        for h in &s.horas {
+            close(h.end_jd_ut - h.start_jd_ut, x.hora_min, "hora");
+        }
+        close(s.horas[0].start_jd_ut, i.day.sunrise_min, "first hora");
+        close(s.horas[23].end_jd_ut, i.day.next_sunrise_min, "last hora");
+        assert_eq!(tithi_group(i.tithi), x.tithi_group, "{id}: tithi group");
+        assert_eq!(
+            nakshatra_nature(i.moon.nakshatra),
+            x.nakshatra_nature,
+            "{id}: nakshatra nature"
+        );
+        assert_eq!(
+            tara_bala(i.natal.nakshatra, i.moon.nakshatra),
+            x.tara,
+            "{id}: tara bala"
+        );
+        assert_eq!(
+            chandra_bala(i.natal.sign, i.moon.sign),
+            x.chandra,
+            "{id}: chandra bala"
+        );
+    }
+}
+
+/// Örnekler yedi varanın her birini, beş tithi grubunu, yedi nakshatra doğasını, elverişli ve
+/// elverişsiz tara ile chandra balayı, Çarşamba istisnasını ve 24 saatten kısa, 24 saate eşit ve
+/// 24 saatten uzun günü kapsar; spec §10'un örneği (06:00–18:00 Pazartesi: Rahu 07:30–09:00,
+/// Abhijit 11:36–12:24) aralarındadır.
+#[test]
+fn muhurta_fixtures_cover_the_required_cases() {
+    let fixtures = muhurta_fixtures();
+    for vara in Vara::ALL {
+        let of_day: Vec<_> = fixtures
+            .iter()
+            .filter(|f| f.input.day.vara == vara)
+            .collect();
+        assert!(!of_day.is_empty(), "no fixture for {vara:?}");
+        for f in of_day {
+            assert_eq!(f.expected.rahu.segment, rahu_segment(vara), "{}", f.id);
+            assert_eq!(
+                f.expected.yamaganda.segment,
+                yamaganda_segment(vara),
+                "{}",
+                f.id
+            );
+        }
+    }
+    for group in TithiGroup::ALL {
+        assert!(
+            fixtures.iter().any(|f| f.expected.tithi_group == group),
+            "no fixture with {group:?}"
+        );
+    }
+    for nature in NakshatraNature::ALL {
+        assert!(
+            fixtures
+                .iter()
+                .any(|f| f.expected.nakshatra_nature == nature),
+            "no fixture with {nature:?}"
+        );
+    }
+    for favourable in [true, false] {
+        assert!(fixtures
+            .iter()
+            .any(|f| f.expected.tara.favourable == favourable));
+        assert!(fixtures
+            .iter()
+            .any(|f| f.expected.chandra.favourable == favourable));
+    }
+    assert!(fixtures
+        .iter()
+        .any(|f| f.expected.abhijit_weekday_exception));
+    assert!(
+        fixtures.iter().any(|f| f.input.tithi > 15),
+        "a krishna tithi"
+    );
+    for want in [59.0, 60.0, 61.0] {
+        assert!(
+            fixtures.iter().any(|f| f.expected.hora_min == want),
+            "no fixture with {want}-minute horas"
+        );
+    }
+    let spec = fixtures
+        .iter()
+        .find(|f| {
+            f.input.day.vara == Vara::Monday
+                && f.input.day.sunrise_min == 360.0
+                && f.input.day.sunset_min == 1080.0
+        })
+        .expect("the six-to-six Monday of the design document");
+    assert_eq!(
+        (spec.expected.rahu.from_min, spec.expected.rahu.to_min),
+        (450.0, 540.0)
+    );
+    assert_eq!(
+        (spec.expected.abhijit.from_min, spec.expected.abhijit.to_min),
+        (696.0, 744.0)
+    );
+}
````


- [ ] **Step 3: Testlerin derlenmediğini doğrula**

```bash
cargo test -p jyotish-core --lib muhurta 2>&1 | grep -E "^error" | head -3
```
Expected: `error[E0583]: file not found for module `muhurta`` ve `could not compile`.

- [ ] **Step 4: Modülün kökü**

**Yeni dosya:** `crates/jyotish-core/src/muhurta/mod.rs`

````rust
//! Muhurta (ileri modüller spec'i §10, Alt proje 3.7).
//!
//! - `segments`: günün dilimleri ve sınıflar; saf fonksiyonlar, ephemeris görmez (K3).
//!
//! Hepsi olgudur: "uygun" ya da "uygunsuz" hükmü, etkinlik şablonu ve açıklama metni yoktur.

pub mod segments;

use serde::{Deserialize, Serialize};

use crate::zodiac::Sign;
use crate::CoreError;

pub use segments::{
    chandra_bala, day_segments, equal_parts, gulika_segment, is_panchaka, is_rikta, is_vishti,
    nakshatra_nature, rahu_segment, tara_bala, tithi_group, yamaganda_segment, ChandraBala,
    DaySegments, NakshatraNature, TaraBala, TithiGroup, ABHIJIT_MUHURTA, CHANDRA_FAVOURABLE,
    DAY_EIGHTHS, DAY_MUHURTAS, HORAS_PER_DAY, NAKSHATRA_NATURES, PANCHAKA_START_DEG, RAHU_SEGMENTS,
    TARA_FAVOURABLE, VAIDHRITI_YOGA, VYATIPATA_YOGA,
};

/// Yöntem kimliği (K7): gün panchangı ve arama sonucu taşır.
pub const MUHURTA_METHOD: &str = "muhurta.v1";

/// Muhurta işaretleri. Gün panchangında aralık olarak listelenir, aramada dışlanabilir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum MuhurtaFlag {
    /// Rikta tithi (4, 9, 14; her iki pakshada).
    Rikta,
    /// Vishti karana.
    Vishti,
    /// Vyatipata nitya yogası.
    Vyatipata,
    /// Vaidhriti nitya yogası.
    Vaidhriti,
    /// Gündüzün varaya göre Rahu'ya ait sekizde bir dilimi.
    RahuSegment,
    Yamaganda,
    Gulika,
    /// Ay sidereal 300°–360° arasında.
    Panchaka,
    /// Jüpiter yanık (v1 yanma kuralı).
    JupiterCombust,
    /// Venüs yanık.
    VenusCombust,
}

impl MuhurtaFlag {
    pub const ALL: [MuhurtaFlag; 10] = [
        MuhurtaFlag::Rikta,
        MuhurtaFlag::Vishti,
        MuhurtaFlag::Vyatipata,
        MuhurtaFlag::Vaidhriti,
        MuhurtaFlag::RahuSegment,
        MuhurtaFlag::Yamaganda,
        MuhurtaFlag::Gulika,
        MuhurtaFlag::Panchaka,
        MuhurtaFlag::JupiterCombust,
        MuhurtaFlag::VenusCombust,
    ];

    /// Gündüz dilimi işaretleri: yalnızca doğuşu ve batışı bilinen günde tanımlıdır.
    pub fn is_day_segment(self) -> bool {
        matches!(self, Self::RahuSegment | Self::Yamaganda | Self::Gulika)
    }
}

/// Kişiye bağlı etkenlerin girdisi: doğum haritasının Ay'ı. Yalnızca iki sayı taşır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct NatalMoon {
    /// Doğum nakshatrası, 1..=27 (sidereal).
    pub nakshatra: u8,
    /// Doğum Ay burcu (rasi çerçevesinde).
    pub sign: Sign,
}

impl NatalMoon {
    pub fn validate(&self) -> Result<(), CoreError> {
        if (1..=27).contains(&self.nakshatra) {
            Ok(())
        } else {
            Err(CoreError::InvalidMuhurtaInput("natal"))
        }
    }
}
````


- [ ] **Step 5: Dilimler, hora tablosu, sınıflar, tara ve chandra bala**

Dosyanın `mod tests` bölümü spec §10'un listelerini elle yazar (dilimden varaya Rahu tablosu, Yamaganda ve Gulika dizileri, yedi doğa listesi, dokuz taranın adı ve elverişli olanlar, 12 × 12 chandra bala tablosu) ve tara sayımını adım adım yürür. Bir liste testi düşerse önce spec §10 ile `RAHU_SEGMENTS` ve `NAKSHATRA_NATURES` satırları karşılaştırılır (27 satırın sırası Ashwini'den Revati'yedir); **tablo hücreleri testi geçirmek için değiştirilmez.**

**Yeni dosya:** `crates/jyotish-core/src/muhurta/segments.rs`

````rust
//! Günün dilimleri ve muhurta sınıfları (ileri modüller spec'i §10): gündüzün sekizde bir
//! dilimleri ve varaya göre işaretli üçü, Abhijit, hora tablosu; tithi grupları, nakshatra
//! doğaları, tara bala ve chandra bala. Saf fonksiyonlardır; ephemeris görmezler (K3).
//!
//! Buradaki tablolar ve listeler klasik muhurta derlemelerinin yaygın biçimidir ve hüküm
//! taşımaz: bir dilimin ya da sınıfın "uygun" olup olmadığına süzgeci kuran astrolog karar
//! verir. "Elverişli" kümeleri (tara ve chandra bala) spec'in yazdığı kümelerdir.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::kuta::tara_number;
use crate::panchang::{karana_name, KaranaName, Vara};
use crate::shadbala::{hora_lord, vara_lord};
use crate::timeline::{Interval, NakshatraPada, SunDay};
use crate::zodiac::Sign;

/// Gündüz (gün doğumu → batımı) bu kadar eşit dilime bölünür.
pub const DAY_EIGHTHS: u8 = 8;
/// Gündüzün eşit muhurta sayısı.
pub const DAY_MUHURTAS: u8 = 15;
/// Abhijit, gündüzün on beş muhurtasının sekizincisidir.
pub const ABHIJIT_MUHURTA: u8 = 8;
/// Gün doğumundan sonraki gün doğumuna saat dilimi (hora) sayısı.
pub const HORAS_PER_DAY: u8 = 24;

/// Varaya göre Rahu dilimi (1..=8), Pazar'dan Cumartesi'ye.
pub const RAHU_SEGMENTS: [u8; 7] = [8, 2, 7, 5, 6, 4, 3];

fn weekday_index(vara: Vara) -> u8 {
    vara as u8
}

/// Rahu'ya ait sekizde bir dilim, 1..=8.
pub fn rahu_segment(vara: Vara) -> u8 {
    RAHU_SEGMENTS[usize::from(weekday_index(vara))]
}

/// Yamaganda dilimi, 1..=7: Perşembe 1, Çarşamba 2, Salı 3, Pazartesi 4, Pazar 5, Cumartesi 6,
/// Cuma 7 (hafta geriye doğru yürür).
pub fn yamaganda_segment(vara: Vara) -> u8 {
    (4 + 7 - weekday_index(vara)) % 7 + 1
}

/// Gulika dilimi, 1..=7: Cumartesi 1, Cuma 2 … Pazar 7.
pub fn gulika_segment(vara: Vara) -> u8 {
    7 - weekday_index(vara)
}

/// `[start, end)` aralığının `parts` eşit dilimi; değer dilimin sırasıdır (1'den). Son dilim
/// tam `end`'de biter.
pub fn equal_parts(start_jd_ut: f64, end_jd_ut: f64, parts: u8) -> Vec<Interval<u8>> {
    let at = |k: u8| {
        if k == parts {
            end_jd_ut
        } else {
            start_jd_ut + (end_jd_ut - start_jd_ut) * f64::from(k) / f64::from(parts)
        }
    };
    (1..=parts)
        .map(|k| Interval {
            value: k,
            start_jd_ut: at(k - 1),
            end_jd_ut: at(k),
        })
        .collect()
}

/// Günün dilimleri. `rahu`, `yamaganda` ve `gulika`'nın değeri dilimin sırasıdır (1..=8),
/// `abhijit`'in değeri muhurtanın sırasıdır (8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DaySegments {
    /// Gündüzün sekiz eşit dilimi, sırayla.
    pub eighths: Vec<Interval<u8>>,
    pub rahu: Interval<u8>,
    pub yamaganda: Interval<u8>,
    pub gulika: Interval<u8>,
    pub abhijit: Interval<u8>,
    /// Çarşamba: bazı derlemeler o gün Abhijit'i kullanmaz. Olgudur; dilim yine verilir.
    pub abhijit_weekday_exception: bool,
    /// 24 saat dilimi ve lordları: gün doğumundan sonraki gün doğumuna.
    pub horas: Vec<Interval<Graha>>,
}

/// Doğuşu, batışı ve sonraki doğuşu bilinen günün dilimleri.
pub fn day_segments(day: &SunDay) -> DaySegments {
    let eighths = equal_parts(day.sunrise_jd_ut, day.sunset_jd_ut, DAY_EIGHTHS);
    let pick = |segment: u8| eighths[usize::from(segment) - 1];
    let muhurtas = equal_parts(day.sunrise_jd_ut, day.sunset_jd_ut, DAY_MUHURTAS);
    let lord = vara_lord(day.vara);
    let horas = equal_parts(day.sunrise_jd_ut, day.next_sunrise_jd_ut, HORAS_PER_DAY)
        .into_iter()
        .map(|h| Interval {
            value: hora_lord(lord, h.value - 1),
            start_jd_ut: h.start_jd_ut,
            end_jd_ut: h.end_jd_ut,
        })
        .collect();
    DaySegments {
        rahu: pick(rahu_segment(day.vara)),
        yamaganda: pick(yamaganda_segment(day.vara)),
        gulika: pick(gulika_segment(day.vara)),
        abhijit: muhurtas[usize::from(ABHIJIT_MUHURTA) - 1],
        abhijit_weekday_exception: day.vara == Vara::Wednesday,
        eighths,
        horas,
    }
}

/// Tithi grubu: paksha içi sıranın beşli döngüsü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum TithiGroup {
    /// 1, 6, 11.
    Nanda,
    /// 2, 7, 12.
    Bhadra,
    /// 3, 8, 13.
    Jaya,
    /// 4, 9, 14.
    Rikta,
    /// 5, 10, 15 (Purnima ve Amavasya dahil).
    Purna,
}

impl TithiGroup {
    pub const ALL: [TithiGroup; 5] = [
        TithiGroup::Nanda,
        TithiGroup::Bhadra,
        TithiGroup::Jaya,
        TithiGroup::Rikta,
        TithiGroup::Purna,
    ];
}

/// Tithinin (1..=30) grubu; iki pakshada aynıdır. Aralık dışı değer panik.
pub fn tithi_group(tithi: u8) -> TithiGroup {
    assert!((1..=30).contains(&tithi), "tithi {tithi} outside 1..=30");
    TithiGroup::ALL[usize::from((tithi - 1) % 5)]
}

/// Nakshatranın doğası (muhurta sınıfı).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum NakshatraNature {
    /// Dhruva: sabit.
    Fixed,
    /// Chara: yürük.
    Movable,
    /// Ugra: sert.
    Fierce,
    /// Mishra: karışık.
    Mixed,
    /// Kshipra: çevik.
    Swift,
    /// Mridu: yumuşak.
    Soft,
    /// Tikshna: keskin.
    Sharp,
}

impl NakshatraNature {
    pub const ALL: [NakshatraNature; 7] = [
        NakshatraNature::Fixed,
        NakshatraNature::Movable,
        NakshatraNature::Fierce,
        NakshatraNature::Mixed,
        NakshatraNature::Swift,
        NakshatraNature::Soft,
        NakshatraNature::Sharp,
    ];
}

/// Nakshatra doğaları, Ashwini'den Revati'ye.
pub const NAKSHATRA_NATURES: [NakshatraNature; 27] = {
    use NakshatraNature::{Fierce, Fixed, Mixed, Movable, Sharp, Soft, Swift};
    [
        Swift, Fierce, Mixed, Fixed, Soft, Sharp, Movable, Swift, Sharp, Fierce, Fierce, Fixed,
        Swift, Soft, Movable, Mixed, Soft, Sharp, Sharp, Fierce, Fixed, Movable, Movable, Movable,
        Fierce, Fixed, Soft,
    ]
};

/// Nakshatranın (1..=27) doğası. Aralık dışı değer panik.
pub fn nakshatra_nature(nakshatra: u8) -> NakshatraNature {
    assert!(
        (1..=27).contains(&nakshatra),
        "nakshatra {nakshatra} outside 1..=27"
    );
    NAKSHATRA_NATURES[usize::from(nakshatra) - 1]
}

/// Rikta tithi: her iki pakshanın 4., 9. ve 14. tithisi.
pub fn is_rikta(tithi: u8) -> bool {
    tithi_group(tithi) == TithiGroup::Rikta
}

/// Vishti karana (1..=60 sırasından).
pub fn is_vishti(karana: u8) -> bool {
    karana_name(karana) == KaranaName::Vishti
}

/// Vyatipata nitya yogasının sırası.
pub const VYATIPATA_YOGA: u8 = 17;
/// Vaidhriti nitya yogasının sırası.
pub const VAIDHRITI_YOGA: u8 = 27;
/// Panchaka: Ay'ın sidereal boylamı bu değerden büyük ya da eşit (300°–360°).
pub const PANCHAKA_START_DEG: f64 = 300.0;

/// Panchaka: Ay Dhanishta'nın üçüncü padasından Revati'nin sonuna (sidereal 300°–360°).
pub fn is_panchaka(moon: NakshatraPada) -> bool {
    moon.nakshatra > 23 || (moon.nakshatra == 23 && moon.pada >= 3)
}

/// Elverişli taralar: Sampat (2), Kshema (4), Sadhana (6), Mitra (8), Parama Mitra (9).
pub const TARA_FAVOURABLE: [u8; 5] = [2, 4, 6, 8, 9];
/// Elverişli chandra bala evleri: geçiş Ay'ının doğum Ay burcundan sayısı.
pub const CHANDRA_FAVOURABLE: [u8; 6] = [1, 3, 6, 7, 10, 11];

/// Tara bala: doğum nakshatrasından günün nakshatrasına sayımın dokuzlu döngüdeki yeri
/// (1 Janma … 9 Parama Mitra) ve elverişli kümede olup olmadığı.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TaraBala {
    pub number: u8,
    pub favourable: bool,
}

/// İki nakshatra da 1..=27 olmalıdır (aralık dışı değer panik; toplayıcılar girdiyi denetler).
pub fn tara_bala(natal_nakshatra: u8, day_nakshatra: u8) -> TaraBala {
    let number = tara_number(natal_nakshatra, day_nakshatra);
    TaraBala {
        number,
        favourable: TARA_FAVOURABLE.contains(&number),
    }
}

/// Chandra bala: geçiş Ay'ının doğum Ay burcundan sayılan evi ve elverişli kümede olup
/// olmadığı. İki burç da rasi çerçevesindedir (K5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ChandraBala {
    pub house: u8,
    pub favourable: bool,
}

pub fn chandra_bala(natal_sign: Sign, moon_sign: Sign) -> ChandraBala {
    let house = moon_sign.house_from(natal_sign);
    ChandraBala {
        house,
        favourable: CHANDRA_FAVOURABLE.contains(&house),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shadbala::HORA_CYCLE;

    /// Elle yazılmış ikinci tablo (§13 katman 3): dilimden varaya. Rahu'nun ilk dilimi hiçbir
    /// güne düşmez.
    const HAND_RAHU_DAY_OF_SEGMENT: [Option<Vara>; 8] = [
        None,
        Some(Vara::Monday),
        Some(Vara::Saturday),
        Some(Vara::Friday),
        Some(Vara::Wednesday),
        Some(Vara::Thursday),
        Some(Vara::Tuesday),
        Some(Vara::Sunday),
    ];
    /// Yamaganda ve Gulika: Pazar'dan Cumartesi'ye, elle.
    const HAND_YAMAGANDA: [u8; 7] = [5, 4, 3, 2, 1, 7, 6];
    const HAND_GULIKA: [u8; 7] = [7, 6, 5, 4, 3, 2, 1];

    #[test]
    fn segment_orders_match_the_hand_tables() {
        for (i, vara) in Vara::ALL.into_iter().enumerate() {
            let rahu = rahu_segment(vara);
            assert_eq!(
                HAND_RAHU_DAY_OF_SEGMENT[usize::from(rahu) - 1],
                Some(vara),
                "{vara:?}"
            );
            assert_eq!(yamaganda_segment(vara), HAND_YAMAGANDA[i], "{vara:?}");
            assert_eq!(gulika_segment(vara), HAND_GULIKA[i], "{vara:?}");
            // Üçü her varada üç ayrı dilimdir.
            let three = [rahu, yamaganda_segment(vara), gulika_segment(vara)];
            assert!(three[0] != three[1] && three[1] != three[2] && three[0] != three[2]);
            assert!(three.iter().all(|s| (1..=8).contains(s)));
        }
        // Rahu yedi günde yedi ayrı dilim alır: 2..=8.
        let mut rahu: Vec<u8> = Vara::ALL.into_iter().map(rahu_segment).collect();
        rahu.sort_unstable();
        assert_eq!(rahu, [2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(
            HAND_RAHU_DAY_OF_SEGMENT.iter().flatten().count(),
            Vara::ALL.len()
        );
    }

    fn minutes(i: &Interval<u8>) -> (f64, f64) {
        (i.start_jd_ut * 1440.0, i.end_jd_ut * 1440.0)
    }

    fn close(a: (f64, f64), b: (f64, f64)) -> bool {
        (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6
    }

    #[test]
    fn a_six_to_six_monday_has_the_textbook_segments() {
        // Gün doğumu 06:00, batımı 18:00, sonraki doğuş ertesi 06:00; gün kesri olarak.
        let day = SunDay {
            sunrise_jd_ut: 0.25,
            sunset_jd_ut: 0.75,
            next_sunrise_jd_ut: 1.25,
            vara: Vara::Monday,
        };
        let s = day_segments(&day);
        assert_eq!(s.eighths.len(), 8);
        // Sekizde bir: 90 dakika. Pazartesi Rahu 2. dilim: 07:30–09:00.
        assert_eq!(s.rahu.value, 2);
        assert!(close(minutes(&s.rahu), (450.0, 540.0)));
        // Yamaganda 4. dilim: 10:30–12:00; Gulika 6. dilim: 13:30–15:00.
        assert!(close(minutes(&s.yamaganda), (630.0, 720.0)));
        assert!(close(minutes(&s.gulika), (810.0, 900.0)));
        // Muhurta 48 dakika; sekizincisi 11:36–12:24.
        assert_eq!(s.abhijit.value, 8);
        assert!(close(minutes(&s.abhijit), (696.0, 744.0)));
        assert!(!s.abhijit_weekday_exception);
        // 24 hora birer saattir; ilki Ay'ındır (Pazartesi), sonra Satürn, Jüpiter, Mars…
        assert_eq!(s.horas.len(), 24);
        assert_eq!(
            s.horas[..8].iter().map(|h| h.value).collect::<Vec<_>>(),
            [
                Graha::Moon,
                Graha::Saturn,
                Graha::Jupiter,
                Graha::Mars,
                Graha::Sun,
                Graha::Venus,
                Graha::Mercury,
                Graha::Moon
            ]
        );
        assert!((s.horas[1].start_jd_ut * 1440.0 - 420.0).abs() < 1e-6);
        assert_eq!(s.horas[23].end_jd_ut, 1.25);
        assert_eq!(s.eighths[7].end_jd_ut, 0.75);
        let wednesday = day_segments(&SunDay {
            vara: Vara::Wednesday,
            ..day
        });
        assert!(wednesday.abhijit_weekday_exception);
    }

    #[test]
    fn hora_lords_start_with_the_day_lord_and_the_next_day_continues_the_cycle() {
        for vara in Vara::ALL {
            let s = day_segments(&SunDay {
                sunrise_jd_ut: 10.25,
                sunset_jd_ut: 10.8,
                next_sunrise_jd_ut: 11.26,
                vara,
            });
            assert_eq!(s.horas[0].value, vara_lord(vara), "{vara:?}");
            for w in s.horas.windows(2) {
                assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
                let at = HORA_CYCLE.iter().position(|g| *g == w[0].value).unwrap();
                assert_eq!(w[1].value, HORA_CYCLE[(at + 1) % 7]);
            }
            // 25. hora, ertesi günün lordudur.
            let next_day = Vara::ALL[(vara as usize + 1) % 7];
            let at = HORA_CYCLE
                .iter()
                .position(|g| *g == s.horas[23].value)
                .unwrap();
            assert_eq!(HORA_CYCLE[(at + 1) % 7], vara_lord(next_day), "{vara:?}");
        }
    }

    /// Spec §10'un listeleri, elle (nakshatra sırası; Ashwini = 1).
    const HAND_GROUPS: [(TithiGroup, [u8; 3]); 5] = [
        (TithiGroup::Nanda, [1, 6, 11]),
        (TithiGroup::Bhadra, [2, 7, 12]),
        (TithiGroup::Jaya, [3, 8, 13]),
        (TithiGroup::Rikta, [4, 9, 14]),
        (TithiGroup::Purna, [5, 10, 15]),
    ];
    const HAND_FIXED: [u8; 4] = [4, 12, 21, 26];
    const HAND_MOVABLE: [u8; 5] = [7, 15, 22, 23, 24];
    const HAND_FIERCE: [u8; 5] = [2, 10, 11, 20, 25];
    const HAND_MIXED: [u8; 2] = [3, 16];
    const HAND_SWIFT: [u8; 3] = [1, 8, 13];
    const HAND_SOFT: [u8; 4] = [5, 14, 17, 27];
    const HAND_SHARP: [u8; 4] = [6, 9, 18, 19];

    #[test]
    fn tithi_groups_and_nakshatra_natures_match_the_hand_lists() {
        for (group, in_paksha) in HAND_GROUPS {
            for t in in_paksha {
                assert_eq!(tithi_group(t), group, "shukla {t}");
                assert_eq!(tithi_group(t + 15), group, "krishna {t}");
            }
        }
        assert_eq!(
            (1..=30).filter(|t| is_rikta(*t)).collect::<Vec<u8>>(),
            [4, 9, 14, 19, 24, 29]
        );
        let lists: [(NakshatraNature, &[u8]); 7] = [
            (NakshatraNature::Fixed, &HAND_FIXED),
            (NakshatraNature::Movable, &HAND_MOVABLE),
            (NakshatraNature::Fierce, &HAND_FIERCE),
            (NakshatraNature::Mixed, &HAND_MIXED),
            (NakshatraNature::Swift, &HAND_SWIFT),
            (NakshatraNature::Soft, &HAND_SOFT),
            (NakshatraNature::Sharp, &HAND_SHARP),
        ];
        let mut seen = 0;
        for (nature, list) in lists {
            for n in list {
                assert_eq!(nakshatra_nature(*n), nature, "nakshatra {n}");
                seen += 1;
            }
        }
        // Yirmi yedi nakshatranın her biri tam bir listededir.
        assert_eq!(seen, 27);
    }

    #[test]
    fn flags_follow_the_karana_the_yoga_and_the_moon() {
        // Vishti sekiz yarım tithide: 8, 15, 22, 29, 36, 43, 50, 57.
        assert_eq!(
            (1..=60).filter(|k| is_vishti(*k)).collect::<Vec<u8>>(),
            [8, 15, 22, 29, 36, 43, 50, 57]
        );
        // Panchaka 300°'de başlar: Dhanishta'nın (23.) üçüncü padası.
        for idx in 0..108u8 {
            let moon = NakshatraPada {
                nakshatra: idx / 4 + 1,
                pada: idx % 4 + 1,
            };
            let start_deg = f64::from(idx) * 10.0 / 3.0;
            assert_eq!(
                is_panchaka(moon),
                start_deg >= PANCHAKA_START_DEG - 1e-9,
                "{moon:?}"
            );
        }
        assert_eq!((VYATIPATA_YOGA, VAIDHRITI_YOGA), (17, 27));
    }

    /// Dokuz taranın adı ve elverişli olanlar, elle.
    const HAND_TARAS: [(&str, bool); 9] = [
        ("Janma", false),
        ("Sampat", true),
        ("Vipat", false),
        ("Kshema", true),
        ("Pratyak", false),
        ("Sadhana", true),
        ("Naidhana", false),
        ("Mitra", true),
        ("Parama Mitra", true),
    ];

    /// Adım adım sayım: `from`'dan başlayıp `to`'ya varana dek nakshatra nakshatra yürür ve
    /// dokuzlu döngüde sayar (formül kullanmaz).
    fn hand_tara(from: u8, to: u8) -> u8 {
        let (mut at, mut count) = (from, 1);
        while at != to {
            at = if at == 27 { 1 } else { at + 1 };
            count = if count == 9 { 1 } else { count + 1 };
        }
        count
    }

    /// Chandra bala evleri, elle: satır doğum Ay burcu, sütun geçiş Ay'ının burcu (Koç'tan).
    const HAND_CHANDRA_HOUSES: [[u8; 12]; 12] = [
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        [12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        [11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        [10, 11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        [9, 10, 11, 12, 1, 2, 3, 4, 5, 6, 7, 8],
        [8, 9, 10, 11, 12, 1, 2, 3, 4, 5, 6, 7],
        [7, 8, 9, 10, 11, 12, 1, 2, 3, 4, 5, 6],
        [6, 7, 8, 9, 10, 11, 12, 1, 2, 3, 4, 5],
        [5, 6, 7, 8, 9, 10, 11, 12, 1, 2, 3, 4],
        [4, 5, 6, 7, 8, 9, 10, 11, 12, 1, 2, 3],
        [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 1, 2],
        [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 1],
    ];
    /// Elverişli evler, elle (sıra: ev 1…12).
    const HAND_CHANDRA_FAVOURABLE: [bool; 12] = [
        true, false, true, false, false, true, true, false, false, true, true, false,
    ];

    #[test]
    fn tara_and_chandra_bala_match_the_hand_tables_on_every_pair() {
        // 27 × 27 nakshatra çifti.
        for from in 1..=27u8 {
            for to in 1..=27u8 {
                let t = tara_bala(from, to);
                assert_eq!(t.number, hand_tara(from, to), "{from} → {to}");
                assert_eq!(
                    t.favourable,
                    HAND_TARAS[usize::from(t.number) - 1].1,
                    "{from} → {to}: {}",
                    HAND_TARAS[usize::from(t.number) - 1].0
                );
            }
        }
        // 12 × 12 burç çifti.
        for (i, natal) in Sign::ALL.into_iter().enumerate() {
            for (j, moon) in Sign::ALL.into_iter().enumerate() {
                let c = chandra_bala(natal, moon);
                assert_eq!(c.house, HAND_CHANDRA_HOUSES[i][j], "{natal:?} → {moon:?}");
                assert_eq!(
                    c.favourable,
                    HAND_CHANDRA_FAVOURABLE[usize::from(c.house) - 1]
                );
            }
        }
    }

    #[test]
    fn equal_parts_end_exactly_on_the_span() {
        let parts = equal_parts(2_451_545.123, 2_451_545.777, 15);
        assert_eq!(parts.len(), 15);
        assert_eq!(parts[0].start_jd_ut, 2_451_545.123);
        assert_eq!(parts[14].end_jd_ut, 2_451_545.777);
        for w in parts.windows(2) {
            assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
            assert_eq!(w[0].value + 1, w[1].value);
        }
    }

    #[test]
    #[should_panic(expected = "outside 1..=30")]
    fn a_tithi_outside_the_range_is_a_programming_error() {
        tithi_group(31);
    }
}
````


- [ ] **Step 6: Dışa aktarımlar**

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 4866496..d61d8d8 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -6,7 +6,7 @@
 //! (Alt proje 3.1): `positions` (ortak girdi), `lordship`, `nature` ve `drishti`. Jaimini
 //! çekirdeği (Alt proje 3.2) ve chara dasha (Alt proje 3.3): `jaimini`. Shadbala ve Bhava bala
 //! (Alt proje 3.4): `shadbala`. Yogalar (Alt proje 3.5): `yogas`. Uyum (Alt proje 3.6): `kuta`.
-//! Muhurta (Alt proje 3.7): `timeline` (aralık dizileri).
+//! Muhurta (Alt proje 3.7): `timeline` (aralık dizileri) ve `muhurta` (gün panchangı, arama).
 
 pub mod ashtakavarga;
 mod chart;
@@ -72,6 +72,10 @@ pub use kuta::{
     KutaScore, MoonDayChange, MoonPoint, OverlayRow, KUTA_METHOD, KUTA_TOTAL_MAX,
 };
 pub use lordship::{house_lord, house_sign, houses_owned};
+pub use muhurta::{
+    chandra_bala, day_segments, nakshatra_nature, tara_bala, tithi_group, ChandraBala, DaySegments,
+    MuhurtaFlag, NakshatraNature, NatalMoon, TaraBala, TithiGroup, MUHURTA_METHOD,
+};
 pub use nature::{kuja_dosha, moon_is_waxing, nature, KujaDosha, Nature, KUJA_DOSHA_HOUSES};
 pub use panchang::{
     compute_panchang, elongation, karana_name, karana_no, local_weekday, nitya_yoga_no, paksha,
````


- [ ] **Step 7: Testleri koş**

```bash
cargo fmt --all --check
cargo clippy -p jyotish-core --all-targets -- -D warnings 2>&1 | tail -1
cargo test -p jyotish-core --lib muhurta 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
grep -rn '"source"' tests/golden/modules/muhurta | grep -vc '"source": "hand"'
```
Expected: fmt çıktısız; clippy `Finished`; `10 passed; 0 failed; 0 ignored; 0 measured; 321 filtered out`; sonra `331 passed` (lib), `8 passed` (golden), `15 passed` (`modules_golden`); son komut `0` yazar. `muhurta_hand_fixtures` bir örnekte düşerse önce o örneğin `derivation` adımı ile spec §10 ve `REVIEW.md`'deki okuma kuralları karşılaştırılır; beklenen değer motor çıktısına bakarak değiştirilmez.

- [ ] **Step 8: Commit**

```bash
git add crates/jyotish-core/src crates/jyotish-core/tests/modules_golden.rs tests/golden/modules
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): muhurta day segments, hora table, tithi groups, nakshatra natures, tara and chandra bala; seven hand fixtures" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: iki `grep` çıktısız.

---


### Task 3: Çekirdek: gün panchangı ve dilimli pencere araması

Spec §10 "Gün panchangı", "Sınıflar ve işaretler", "Kişiye bağlı etkenler", "Pencere araması", "Veri modeli" (`DayPanchang`, `MuhurtaFilter`, `MuhurtaWindow`), "Test" (çapraz denetim; property "arama sonucu her pencerenin her anında süzgeci sağlar … en kısa süreden kısa değildir"); §13 katman 2 ve 4; K1–K3, K8; D8, D9, D12–D20. Hiçbir mevcut davranış değişmez. Bu görevin sonunda `day_panchang` ve `muhurta_search_page` çekirdekte hazırdır; komutlar Task 5'tedir.

**Files:**
- Create: `crates/jyotish-core/src/muhurta/day.rs`, `crates/jyotish-core/src/muhurta/search.rs`
- Modify: `crates/jyotish-core/src/muhurta/mod.rs`, `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/panchang.rs`, `crates/jyotish-core/src/proptests.rs`
- Test: `day.rs` (6 test), `search.rs` (11 test), `proptests.rs` `muhurta_properties` (3 test daha)

**Interfaces:**
- Consumes (v1, Task 1, Task 2): `timeline`'ın dizileri, `Interval`, `SunDay`, `map_merge`, `sun_days`, `sun_days_cover`, `vara_of_sun_days`, `lagna_sign_at`, `check_span`; `panchang::{day_sunrise, day_sunset, following_sunrise, vara_at, local_weekday, tithi_no, karana_no, karana_name, nitya_yoga_no, paksha, Paksha, KaranaName, Vara, SUNRISE_WINDOW_DAYS}`; `chart::{finite, rasi_frame, ChartWarning}`; Task 2'nin dilimleri, sınıfları ve `MuhurtaFlag`, `NatalMoon`, `MUHURTA_METHOD`.
- Produces (`jyotish_core::muhurta`; yıldızlılar kökten de dışa aktarılır):
  - Gün: `DayTithi { no, paksha, group }`\*, `DayNakshatra { no, pada, nature }`\*, `DayKarana { no, name }`\*, `PersonalFactors { tara, chandra }`\*, `DayPanchang { day_start_jd_ut, day_end_jd_ut, sunrise_jd_ut, sunset_jd_ut, next_sunrise_jd_ut, vara, tithis, nakshatras, yogas, karanas, segments, flags, personal, warnings, method }`\*, `DAY_FLAG_KINDS`, `day_panchang(eph, jd_ut, lat, lon, settings, natal) -> Result<DayPanchang, CoreError>`\*; crate içi `day::sun_day_at`.
  - Arama: `MuhurtaCriteria`\* (`validate`, `normalized`; `Default`), `MuhurtaFilter { from_jd_ut, to_jd_ut, lat, lon, criteria, natal }`\* (`validate`), `WindowSnapshot`\*, `MuhurtaWindow { start_jd_ut, end_jd_ut, snapshot }`\*, `MuhurtaCursor { chunk, open_start_jd_ut }`\*, `MuhurtaSearchPage { windows, scanned, chunks, next, warnings, method }`\*, `MUHURTA_MAX_RANGE_DAYS`\*, `MUHURTA_CHUNK_DAYS`\*, `MUHURTA_MAX_MIN_MINUTES`, `MUHURTA_MIN_WINDOW_DAYS`, `muhurta_chunks(from, to) -> u32`\*, `window_snapshot(eph, settings, jd_ut, lat, lon)`\*, `muhurta_search_page(eph, settings, filter, cursor)`\*; crate içi `search::search_page_with` (dilim uzunluğu parametreli; testler için).

- [ ] **Step 1: Testler: alt modüllerin kaydı ve değişmezler**

`muhurta_properties` üç test kazanır: gün panchangı gününü tam böler; pencerenin içinden örneklenen her an süzgeci sağlar ve iki pencerenin arası sağlamaz (süzgeç, aramadan bağımsız noktasal bir hesapla yeniden değerlendirilir: `holds_at`); dilim uzunluğu pencereleri değiştirmez.

````diff
diff --git a/crates/jyotish-core/src/muhurta/mod.rs b/crates/jyotish-core/src/muhurta/mod.rs
index 57e6235..f3927da 100644
--- a/crates/jyotish-core/src/muhurta/mod.rs
+++ b/crates/jyotish-core/src/muhurta/mod.rs
@@ -4,6 +4,8 @@
 //!
 //! Hepsi olgudur: "uygun" ya da "uygunsuz" hükmü, etkinlik şablonu ve açıklama metni yoktur.
 
+pub mod day;
+pub mod search;
 pub mod segments;
 
 use serde::{Deserialize, Serialize};
diff --git a/crates/jyotish-core/src/proptests.rs b/crates/jyotish-core/src/proptests.rs
index c44308f..f67d66a 100644
--- a/crates/jyotish-core/src/proptests.rs
+++ b/crates/jyotish-core/src/proptests.rs
@@ -1562,12 +1562,29 @@ mod timeline_properties {
 }
 
 mod muhurta_properties {
+    use ephemeris::{Body, Ephemeris};
     use proptest::prelude::*;
 
-    use crate::muhurta::day_segments;
+    use super::timeline_properties::sky;
+    use crate::chart::rasi_frame;
+    use crate::combustion::is_combust;
+    use crate::fake::FakeEph;
+    use crate::graha::Graha;
+    use crate::muhurta::day::sun_day_at;
+    use crate::muhurta::search::search_page_with;
+    use crate::muhurta::{
+        chandra_bala, day_panchang, day_segments, is_panchaka, is_rikta, is_vishti, tara_bala,
+        window_snapshot, MuhurtaCriteria, MuhurtaFilter, MuhurtaFlag, MuhurtaWindow, NatalMoon,
+        MUHURTA_MIN_WINDOW_DAYS, VAIDHRITI_YOGA, VYATIPATA_YOGA,
+    };
     use crate::panchang::Vara;
+    use crate::settings::CalcSettings;
     use crate::shadbala::vara_lord;
-    use crate::timeline::SunDay;
+    use crate::timeline::{NakshatraPada, SunDay};
+    use crate::zodiac::Sign;
+
+    const JD: f64 = 2_451_545.0;
+    const SECOND: f64 = 1.0 / 86_400.0;
 
     fn vara() -> impl Strategy<Value = Vara> {
         (0usize..7).prop_map(|i| Vara::ALL[i])
@@ -1589,6 +1606,137 @@ mod muhurta_properties {
             })
     }
 
+    fn subset<T: Clone + std::fmt::Debug + 'static>(all: Vec<T>) -> impl Strategy<Value = Vec<T>> {
+        let n = all.len();
+        // Çoğu durumda boş küme (kısıt yok); yoksa rastgele bir alt küme.
+        prop_oneof![
+            3 => Just(Vec::new()),
+            2 => proptest::sample::subsequence(all, 1..=n),
+        ]
+    }
+
+    fn criteria() -> impl Strategy<Value = MuhurtaCriteria> {
+        (
+            subset(Vara::ALL.to_vec()),
+            subset((1..=30u8).collect()),
+            subset((1..=27u8).collect()),
+            subset(Sign::ALL.to_vec()),
+            any::<bool>(),
+            prop_oneof![3 => Just(0u32), 1 => 1u32..180],
+            subset(MuhurtaFlag::ALL.to_vec()),
+            any::<bool>(),
+            any::<bool>(),
+        )
+            .prop_map(
+                |(
+                    weekdays,
+                    tithis,
+                    nakshatras,
+                    lagna_signs,
+                    daytime_only,
+                    min_minutes,
+                    exclude,
+                    require_tara,
+                    require_chandra,
+                )| MuhurtaCriteria {
+                    weekdays,
+                    tithis,
+                    nakshatras,
+                    lagna_signs,
+                    daytime_only,
+                    min_minutes,
+                    exclude,
+                    require_tara,
+                    require_chandra,
+                },
+            )
+    }
+
+    /// Süzgecin `jd` anında sağlanıp sağlanmadığı, aramadan bağımsız noktasal hesapla.
+    fn holds_at(eph: &mut FakeEph, s: &CalcSettings, f: &MuhurtaFilter, jd: f64) -> bool {
+        let c = &f.criteria;
+        let snap = window_snapshot(eph, s, jd, f.lat, f.lon).unwrap();
+        let excluded = |flag| c.exclude.contains(&flag);
+        if !c.weekdays.is_empty() && !c.weekdays.contains(&snap.vara) {
+            return false;
+        }
+        if !c.tithis.is_empty() && !c.tithis.contains(&snap.tithi) {
+            return false;
+        }
+        if !c.nakshatras.is_empty() && !c.nakshatras.contains(&snap.nakshatra) {
+            return false;
+        }
+        if !c.lagna_signs.is_empty() && !c.lagna_signs.contains(&snap.lagna_sign) {
+            return false;
+        }
+        let moon = NakshatraPada {
+            nakshatra: snap.nakshatra,
+            pada: snap.pada,
+        };
+        if (excluded(MuhurtaFlag::Rikta) && is_rikta(snap.tithi))
+            || (excluded(MuhurtaFlag::Vishti) && is_vishti(snap.karana))
+            || (excluded(MuhurtaFlag::Vyatipata) && snap.yoga == VYATIPATA_YOGA)
+            || (excluded(MuhurtaFlag::Vaidhriti) && snap.yoga == VAIDHRITI_YOGA)
+            || (excluded(MuhurtaFlag::Panchaka) && is_panchaka(moon))
+        {
+            return false;
+        }
+        let frame = rasi_frame(s);
+        let sun = eph.body(jd, Body::Sun, frame).unwrap().lon;
+        for (flag, graha, body) in [
+            (MuhurtaFlag::JupiterCombust, Graha::Jupiter, Body::Jupiter),
+            (MuhurtaFlag::VenusCombust, Graha::Venus, Body::Venus),
+        ] {
+            let p = eph.body(jd, body, frame).unwrap();
+            if excluded(flag)
+                && is_combust(graha, p.lon, sun, p.lon_speed < 0.0, &s.combustion_orbs)
+            {
+                return false;
+            }
+        }
+        let day = sun_day_at(eph, s, jd, f.lat, f.lon).unwrap();
+        if c.daytime_only && !day.is_some_and(|d| jd < d.sunset_jd_ut) {
+            return false;
+        }
+        if let Some(d) = day {
+            let seg = day_segments(&d);
+            if (excluded(MuhurtaFlag::RahuSegment) && seg.rahu.contains(jd))
+                || (excluded(MuhurtaFlag::Yamaganda) && seg.yamaganda.contains(jd))
+                || (excluded(MuhurtaFlag::Gulika) && seg.gulika.contains(jd))
+            {
+                return false;
+            }
+        }
+        if let Some(natal) = &f.natal {
+            if c.require_tara && !tara_bala(natal.nakshatra, snap.nakshatra).favourable {
+                return false;
+            }
+            let moon_sign = Sign::of(eph.body(jd, Body::Moon, frame).unwrap().lon);
+            if c.require_chandra && !chandra_bala(natal.sign, moon_sign).favourable {
+                return false;
+            }
+        }
+        true
+    }
+
+    fn search_all(
+        eph: &mut FakeEph,
+        s: &CalcSettings,
+        f: &MuhurtaFilter,
+        chunk_days: f64,
+    ) -> Vec<MuhurtaWindow> {
+        let mut out = Vec::new();
+        let mut cursor = None;
+        loop {
+            let page = search_page_with(eph, s, f, cursor, chunk_days).unwrap();
+            out.extend(page.windows);
+            match page.next {
+                Some(next) => cursor = Some(next),
+                None => return out,
+            }
+        }
+    }
+
     proptest! {
         #[test]
         fn eight_segments_split_the_daytime_and_the_three_marked_ones_differ(day in sun_day()) {
@@ -1635,4 +1783,117 @@ mod muhurta_properties {
             }
         }
     }
+
+    proptest! {
+        #![proptest_config(ProptestConfig::with_cases(48))]
+
+        #[test]
+        fn the_day_panchang_splits_its_day_and_keeps_one_vara(
+            sky in sky(),
+            offset in 0.0f64..400.0,
+            nakshatra in 1u8..=27,
+            sign in 0usize..12,
+        ) {
+            let mut eph = sky.eph();
+            let s = CalcSettings::default();
+            let natal = NatalMoon { nakshatra, sign: Sign::ALL[sign] };
+            let jd = JD + offset;
+            let d = day_panchang(&mut eph, jd, 41.0, 29.0, &s, Some(natal)).unwrap();
+            prop_assert!(d.day_start_jd_ut <= jd && jd < d.day_end_jd_ut);
+            let (a, b) = (d.day_start_jd_ut, d.day_end_jd_ut);
+            for (first, last) in [
+                (d.tithis[0].start_jd_ut, d.tithis[d.tithis.len() - 1].end_jd_ut),
+                (d.nakshatras[0].start_jd_ut, d.nakshatras[d.nakshatras.len() - 1].end_jd_ut),
+                (d.yogas[0].start_jd_ut, d.yogas[d.yogas.len() - 1].end_jd_ut),
+                (d.karanas[0].start_jd_ut, d.karanas[d.karanas.len() - 1].end_jd_ut),
+            ] {
+                prop_assert_eq!((first, last), (a, b));
+            }
+            let p = d.personal.as_ref().unwrap();
+            prop_assert_eq!(p.tara[0].start_jd_ut, a);
+            prop_assert_eq!(p.chandra[p.chandra.len() - 1].end_jd_ut, b);
+            for f in &d.flags {
+                prop_assert!(a <= f.start_jd_ut && f.start_jd_ut < f.end_jd_ut && f.end_jd_ut <= b);
+            }
+            let seg = d.segments.as_ref().unwrap();
+            prop_assert_eq!(seg.horas[0].value, vara_lord(d.vara));
+            prop_assert_eq!(seg.horas[23].end_jd_ut, b);
+        }
+
+        #[test]
+        fn every_moment_of_a_window_passes_the_filter_and_gaps_do_not(
+            sky in sky(),
+            start in 0.0f64..300.0,
+            days in 1.0f64..25.0,
+            criteria in criteria(),
+            nakshatra in 1u8..=27,
+            sign in 0usize..12,
+            samples in proptest::collection::vec(0.0f64..1.0, 3),
+        ) {
+            let mut eph = sky.eph();
+            let s = CalcSettings::default();
+            let f = MuhurtaFilter {
+                from_jd_ut: JD + start,
+                to_jd_ut: JD + start + days,
+                lat: 41.0,
+                lon: 29.0,
+                criteria,
+                natal: Some(NatalMoon { nakshatra, sign: Sign::ALL[sign] }),
+            };
+            let windows = search_all(&mut eph, &s, &f, 31.0);
+            let min_days = (f64::from(f.criteria.min_minutes) / 1440.0).max(MUHURTA_MIN_WINDOW_DAYS);
+            let mut previous_end = f.from_jd_ut;
+            for w in &windows {
+                prop_assert!(f.from_jd_ut <= w.start_jd_ut && w.end_jd_ut <= f.to_jd_ut);
+                prop_assert!(w.start_jd_ut < w.end_jd_ut);
+                prop_assert!(w.end_jd_ut - w.start_jd_ut >= min_days);
+                prop_assert!(previous_end <= w.start_jd_ut, "ordered and disjoint");
+                // Pencerenin içinden rastgele anlar (uçlardan 1 sn içeride).
+                let inner = w.end_jd_ut - w.start_jd_ut - 2.0 * SECOND;
+                if inner > 0.0 {
+                    for x in &samples {
+                        let jd = w.start_jd_ut + SECOND + inner * x;
+                        prop_assert!(holds_at(&mut eph, &s, &f, jd), "inside a window at {}", jd);
+                    }
+                }
+                // En kısa süre yokken iki pencerenin arası süzgeci sağlamaz.
+                let gap = w.start_jd_ut - previous_end;
+                if f.criteria.min_minutes == 0 && gap > 4.0 * SECOND {
+                    for x in &samples {
+                        let jd = previous_end + SECOND + (gap - 2.0 * SECOND) * x;
+                        prop_assert!(!holds_at(&mut eph, &s, &f, jd), "inside a gap at {}", jd);
+                    }
+                }
+                previous_end = w.end_jd_ut;
+            }
+        }
+
+        #[test]
+        fn chunk_length_does_not_change_the_windows(
+            sky in sky(),
+            start in 0.0f64..300.0,
+            days in 1.0f64..40.0,
+            criteria in criteria(),
+            chunk in 0.7f64..9.0,
+        ) {
+            let mut eph = sky.eph();
+            let s = CalcSettings::default();
+            let f = MuhurtaFilter {
+                from_jd_ut: JD + start,
+                to_jd_ut: JD + start + days,
+                lat: 41.0,
+                lon: 29.0,
+                criteria,
+                natal: Some(NatalMoon { nakshatra: 4, sign: Sign::Taurus }),
+            };
+            let whole = search_all(&mut eph, &s, &f, 400.0);
+            let pieces = search_all(&mut eph, &s, &f, chunk);
+            prop_assert_eq!(whole.len(), pieces.len());
+            for (a, b) in whole.iter().zip(&pieces) {
+                // Sınırlar aynı ikiye bölmeden gelmez: çözünürlük kadar oynayabilir.
+                prop_assert!((a.start_jd_ut - b.start_jd_ut).abs() < SECOND);
+                prop_assert!((a.end_jd_ut - b.end_jd_ut).abs() < SECOND);
+            }
+        }
+    }
 }
````


- [ ] **Step 2: Testlerin derlenmediğini doğrula**

```bash
cargo test -p jyotish-core --lib muhurta 2>&1 | grep -E "^error" | head -3
```
Expected: `error[E0583]: file not found for module `day``, `error[E0583]: file not found for module `search`` ve çözülemeyen içe aktarımlar (`crate::muhurta::day_panchang` …).

- [ ] **Step 3: Gün panchangı**

`day_sunset` `day.rs`'te kullanılır (görünürlük); gün, andan önceki son gün doğumunda başlar (D8) ve doğuşu, batışı ya da sonraki doğuşu yoksa 06:00 yerel ortalama saat gününe düşer (D9).

````diff
diff --git a/crates/jyotish-core/src/panchang.rs b/crates/jyotish-core/src/panchang.rs
index 1cfb376..b5ec493 100644
--- a/crates/jyotish-core/src/panchang.rs
+++ b/crates/jyotish-core/src/panchang.rs
@@ -315,7 +315,7 @@ pub(crate) fn day_sunrise<E: Ephemeris + ?Sized>(
 }
 
 /// `sunrise`'tan sonraki ilk gün batımı; 1 gün içinde değilse `None`.
-fn day_sunset<E: Ephemeris + ?Sized>(
+pub(crate) fn day_sunset<E: Ephemeris + ?Sized>(
     eph: &mut E,
     sunrise: f64,
     geo_lat: f64,
````

**Yeni dosya:** `crates/jyotish-core/src/muhurta/day.rs`

````rust
//! Gün panchangı (ileri modüller spec'i §10): verilen anı içeren gün, gün doğumundan sonraki
//! gün doğumuna. Beş uzvun gün içindeki bütün değerleri ve değişim anları, günün dilimleri,
//! işaretler ve (doğum Ay'ı verilmişse) tara ve chandra bala.
//!
//! Gün, `compute_panchang`'ın kuralıyla seçilir: andan önceki son gün doğumunda başlar. Doğuşu,
//! batışı ya da sonraki doğuşu olmayan günde (kutup gecesi ya da gündüzü) gün 06:00 yerel
//! ortalama saatte başlar ve 24 saat sürer; dilimler hesaplanmaz, uzuvlar yine verilir.

use ephemeris::{Body, Ephemeris, Precision};
use serde::{Deserialize, Serialize};

use crate::chart::{rasi_frame, ChartWarning};
use crate::graha::Graha;
use crate::panchang::{
    day_sunrise, day_sunset, following_sunrise, karana_name, local_weekday, paksha, vara_at,
    KaranaName, Paksha, Vara, SUNRISE_WINDOW_DAYS,
};
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::timeline::{
    combust_series, graha_sign_series, karana_series, map_merge, nakshatra_series,
    nitya_yoga_series, tithis_of_karanas, Interval, SunDay,
};
use crate::CoreError;

use super::segments::{
    chandra_bala, day_segments, is_panchaka, is_rikta, is_vishti, nakshatra_nature, tara_bala,
    tithi_group, ChandraBala, DaySegments, NakshatraNature, TaraBala, TithiGroup, VAIDHRITI_YOGA,
    VYATIPATA_YOGA,
};
use super::{MuhurtaFlag, NatalMoon, MUHURTA_METHOD};

/// Gün doğumu yoksa günün başladığı yerel ortalama saat: 06:00 = 0,25 gün (v1 kuralı).
const FALLBACK_DAY_START: f64 = 0.25;

/// Gün panchangının `flags` listesinde aralık olarak verilen işaretler, bu sırayla. Üç gündüz
/// dilimi (`RahuSegment`, `Yamaganda`, `Gulika`) `segments` içindedir.
pub const DAY_FLAG_KINDS: [MuhurtaFlag; 7] = [
    MuhurtaFlag::Rikta,
    MuhurtaFlag::Vishti,
    MuhurtaFlag::Vyatipata,
    MuhurtaFlag::Vaidhriti,
    MuhurtaFlag::Panchaka,
    MuhurtaFlag::JupiterCombust,
    MuhurtaFlag::VenusCombust,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DayTithi {
    /// 1..=30.
    pub no: u8,
    pub paksha: Paksha,
    pub group: TithiGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DayNakshatra {
    /// Ay'ın nakshatrası, 1..=27.
    pub no: u8,
    /// 1..=4.
    pub pada: u8,
    pub nature: NakshatraNature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DayKarana {
    /// 1..=60.
    pub no: u8,
    pub name: KaranaName,
}

/// Kişiye bağlı etkenler, gün içindeki aralıklarıyla: Ay gün içinde nakshatra ya da burç
/// değiştirirse değer de değişir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PersonalFactors {
    pub tara: Vec<Interval<TaraBala>>,
    pub chandra: Vec<Interval<ChandraBala>>,
}

/// Bir günün panchangı. Bütün aralık dizileri `[day_start_jd_ut, day_end_jd_ut]`'ye kırpılmıştır
/// ve onu boşluksuz böler.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DayPanchang {
    /// Günün başı: gün doğumu; yoksa 06:00 yerel ortalama saat.
    pub day_start_jd_ut: f64,
    /// Günün sonu: sonraki gün doğumu; yoksa başından 24 saat sonrası.
    pub day_end_jd_ut: f64,
    /// Üçü birlikte dolu ya da birlikte boştur.
    pub sunrise_jd_ut: Option<f64>,
    pub sunset_jd_ut: Option<f64>,
    pub next_sunrise_jd_ut: Option<f64>,
    pub vara: Vara,
    pub tithis: Vec<Interval<DayTithi>>,
    pub nakshatras: Vec<Interval<DayNakshatra>>,
    pub yogas: Vec<Interval<u8>>,
    pub karanas: Vec<Interval<DayKarana>>,
    /// Doğuşu ve batışı olmayan günde `None`.
    pub segments: Option<DaySegments>,
    /// `DAY_FLAG_KINDS` sırasında, her işaretin gün içindeki aralıkları.
    pub flags: Vec<Interval<MuhurtaFlag>>,
    /// Doğum Ay'ı verilmediyse `None`.
    pub personal: Option<PersonalFactors>,
    /// `SunriseUnavailable` ya da `LowPrecision`.
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

/// `jd_ut` anını içeren, doğuşu, batışı ve sonraki doğuşu bilinen gün; yoksa `None`.
pub(crate) fn sun_day_at<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    jd_ut: f64,
    geo_lat: f64,
    geo_lon: f64,
) -> Result<Option<SunDay>, CoreError> {
    let disc = settings.sunrise_def.sun_disc();
    let Some(sunrise) = day_sunrise(eph, jd_ut, geo_lat, geo_lon, disc)? else {
        return Ok(None);
    };
    let Some(sunset) = day_sunset(eph, sunrise, geo_lat, geo_lon, disc)? else {
        return Ok(None);
    };
    let Some(next) = following_sunrise(eph, sunrise, geo_lat, geo_lon, disc)? else {
        return Ok(None);
    };
    if next <= jd_ut || next - sunrise > SUNRISE_WINDOW_DAYS || sunset >= next {
        return Ok(None);
    }
    Ok(Some(SunDay {
        sunrise_jd_ut: sunrise,
        sunset_jd_ut: sunset,
        next_sunrise_jd_ut: next,
        vara: local_weekday(sunrise, geo_lon),
    }))
}

/// `value`'ların işaret taşıyan aralıkları, birleştirilmiş.
fn flagged<T>(
    series: &[Interval<T>],
    flag: MuhurtaFlag,
    holds: impl Fn(&T) -> bool,
) -> Vec<Interval<MuhurtaFlag>> {
    map_merge(series, |v| holds(v))
        .into_iter()
        .filter(|i| i.value)
        .map(|i| Interval {
            value: flag,
            start_jd_ut: i.start_jd_ut,
            end_jd_ut: i.end_jd_ut,
        })
        .collect()
}

/// `jd_ut` anını içeren günün panchangı. Konum denetimi alan adıyla döner
/// (`InvalidMuhurtaInput("place")`); konum hata parametresine girmez (bulunulan yer kişisel
/// veridir). `natal` verilirse tara ve chandra bala eklenir.
pub fn day_panchang<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    geo_lat: f64,
    geo_lon: f64,
    settings: &CalcSettings,
    natal: Option<NatalMoon>,
) -> Result<DayPanchang, CoreError> {
    settings.validate()?;
    // NaN ve sonsuz da aralık dışındadır.
    if !JD_UT_RANGE.contains(&jd_ut) {
        return Err(CoreError::InvalidJulianDay(jd_ut));
    }
    if !((-90.0..=90.0).contains(&geo_lat) && (-180.0..=180.0).contains(&geo_lon)) {
        return Err(CoreError::InvalidMuhurtaInput("place"));
    }
    if let Some(n) = &natal {
        n.validate()?;
    }
    let mut warnings = Vec::new();
    let day = sun_day_at(eph, settings, jd_ut, geo_lat, geo_lon)?;
    let (start, end, vara) = match &day {
        Some(d) => (d.sunrise_jd_ut, d.next_sunrise_jd_ut, d.vara),
        None => {
            warnings.push(ChartWarning::SunriseUnavailable);
            // 06:00 yerel ortalama saat: yerel gün sayısı `jd + 0,5 + boylam / 360`'tır.
            let local = jd_ut + 0.5 + geo_lon / 360.0 - FALLBACK_DAY_START;
            let start = local.floor() - 0.5 - geo_lon / 360.0 + FALLBACK_DAY_START;
            let disc = settings.sunrise_def.sun_disc();
            (
                start,
                start + 1.0,
                vara_at(eph, jd_ut, geo_lat, geo_lon, disc)?,
            )
        }
    };
    if !(JD_UT_RANGE.contains(&start) && JD_UT_RANGE.contains(&end)) {
        return Err(CoreError::InvalidJulianDay(jd_ut));
    }
    // Hassasiyet Güneş'ten okunur (`compute_chart` ile aynı kural).
    if eph.body(start, Body::Sun, rasi_frame(settings))?.precision == Precision::Moshier {
        warnings.push(ChartWarning::LowPrecision);
    }

    let karanas = karana_series(eph, settings, start, end)?;
    let tithis = tithis_of_karanas(&karanas);
    let nakshatras = nakshatra_series(eph, settings, start, end)?;
    let yogas = nitya_yoga_series(eph, settings, start, end)?;
    let jupiter = combust_series(eph, settings, Graha::Jupiter, start, end)?;
    let venus = combust_series(eph, settings, Graha::Venus, start, end)?;

    let mut flags = flagged(&tithis, MuhurtaFlag::Rikta, |t| is_rikta(*t));
    flags.extend(flagged(&karanas, MuhurtaFlag::Vishti, |k| is_vishti(*k)));
    flags.extend(flagged(&yogas, MuhurtaFlag::Vyatipata, |y| {
        *y == VYATIPATA_YOGA
    }));
    flags.extend(flagged(&yogas, MuhurtaFlag::Vaidhriti, |y| {
        *y == VAIDHRITI_YOGA
    }));
    flags.extend(flagged(&nakshatras, MuhurtaFlag::Panchaka, |n| {
        is_panchaka(*n)
    }));
    flags.extend(flagged(&jupiter, MuhurtaFlag::JupiterCombust, |c| *c));
    flags.extend(flagged(&venus, MuhurtaFlag::VenusCombust, |c| *c));

    let personal = match natal {
        Some(natal) => {
            let moon_signs = graha_sign_series(eph, settings, Graha::Moon, start, end)?;
            Some(PersonalFactors {
                tara: map_merge(&nakshatras, |n| tara_bala(natal.nakshatra, n.nakshatra)),
                chandra: map_merge(&moon_signs, |s| chandra_bala(natal.sign, *s)),
            })
        }
        None => None,
    };

    Ok(DayPanchang {
        day_start_jd_ut: start,
        day_end_jd_ut: end,
        sunrise_jd_ut: day.map(|d| d.sunrise_jd_ut),
        sunset_jd_ut: day.map(|d| d.sunset_jd_ut),
        next_sunrise_jd_ut: day.map(|d| d.next_sunrise_jd_ut),
        vara,
        tithis: map_merge(&tithis, |t| DayTithi {
            no: *t,
            paksha: paksha(*t),
            group: tithi_group(*t),
        }),
        nakshatras: map_merge(&nakshatras, |n| DayNakshatra {
            no: n.nakshatra,
            pada: n.pada,
            nature: nakshatra_nature(n.nakshatra),
        }),
        yogas,
        karanas: map_merge(&karanas, |k| DayKarana {
            no: *k,
            name: karana_name(*k),
        }),
        segments: day.as_ref().map(day_segments),
        flags,
        personal,
        warnings,
        method: MUHURTA_METHOD.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use ephemeris::Precision;

    use super::*;
    use crate::fake::FakeEph;
    use crate::panchang::compute_panchang;
    use crate::zodiac::Sign;

    const JD: f64 = 2_451_545.0;
    const TOL: f64 = 0.2 / 86_400.0;

    fn day(eph: &mut FakeEph, jd: f64, natal: Option<NatalMoon>) -> DayPanchang {
        day_panchang(eph, jd, 41.0, 29.0, &CalcSettings::default(), natal).unwrap()
    }

    fn partitions<T>(s: &[Interval<T>], d: &DayPanchang) {
        assert_eq!(s.first().unwrap().start_jd_ut, d.day_start_jd_ut);
        assert_eq!(s.last().unwrap().end_jd_ut, d.day_end_jd_ut);
        for w in s.windows(2) {
            assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
        }
    }

    #[test]
    fn the_day_runs_from_the_last_sunrise_to_the_next_one() {
        // Sahte gün doğumu n + 0,67, batımı n + 0,15. 12:00 UT'yi içeren gün 2451544,67'de
        // başlar (yerel ortalama 1 Ocak 2000, Cumartesi).
        let mut eph = FakeEph::default();
        let d = day(&mut eph, JD, None);
        assert!((d.day_start_jd_ut - 2_451_544.67).abs() < 1e-9);
        assert!((d.day_end_jd_ut - 2_451_545.67).abs() < 1e-9);
        assert_eq!(d.sunrise_jd_ut, Some(d.day_start_jd_ut));
        assert_eq!(d.next_sunrise_jd_ut, Some(d.day_end_jd_ut));
        assert!((d.sunset_jd_ut.unwrap() - 2_451_545.15).abs() < 1e-9);
        assert_eq!(d.vara, Vara::Saturday);
        assert!(d.warnings.is_empty());
        assert_eq!(d.method, MUHURTA_METHOD);
        assert_eq!(d.personal, None);
        partitions(&d.tithis, &d);
        partitions(&d.nakshatras, &d);
        partitions(&d.yogas, &d);
        partitions(&d.karanas, &d);
        // Gün doğumunda uzanım 85° − 0,33 × 12,22° = 80,97°: 7. tithi (Bhadra); 84°'de 8.
        // (Jaya). Gün 93,19°'de biter.
        assert_eq!(
            d.tithis
                .iter()
                .map(|t| (t.value.no, t.value.group))
                .collect::<Vec<_>>(),
            [(7, TithiGroup::Bhadra), (8, TithiGroup::Jaya)]
        );
        assert!(d.tithis.iter().all(|t| t.value.paksha == Paksha::Shukla));
        assert!((d.tithis[1].start_jd_ut - (JD - 1.0 / 12.22)).abs() < TOL);
        // Karana: 14 (Vanija), 84°'de 15 (Vishti), 90°'de 16 (Bava).
        assert_eq!(
            d.karanas
                .iter()
                .map(|k| (k.value.no, k.value.name))
                .collect::<Vec<_>>(),
            [
                (14, KaranaName::Vanija),
                (15, KaranaName::Vishti),
                (16, KaranaName::Bava)
            ]
        );
        // Ay gün doğumunda 66,64°: Mrigashira'nın son padası (yumuşak), sonra Ardra (keskin).
        assert_eq!(
            (d.nakshatras[0].value.no, d.nakshatras[0].value.pada),
            (5, 4)
        );
        assert_eq!(d.nakshatras[0].value.nature, NakshatraNature::Soft);
        assert_eq!(d.nakshatras[1].value.nature, NakshatraNature::Sharp);
        // Dilimler günün doğuşundan ve batışından.
        let s = d.segments.as_ref().unwrap();
        assert_eq!(s.eighths.len(), 8);
        assert_eq!(s.eighths[0].start_jd_ut, d.day_start_jd_ut);
        assert_eq!(s.eighths[7].end_jd_ut, d.sunset_jd_ut.unwrap());
        // Cumartesi: Rahu 3., Yamaganda 6., Gulika 1. dilim.
        assert_eq!((s.rahu.value, s.yamaganda.value, s.gulika.value), (3, 6, 1));
        assert_eq!(s.horas.len(), 24);
        assert_eq!(s.horas[0].value, Graha::Saturn);
        assert_eq!(s.horas[23].end_jd_ut, d.day_end_jd_ut);
    }

    #[test]
    fn the_day_agrees_with_the_birth_panchang_inside_every_interval() {
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        for offset in [0.0, 0.3, 0.68, 1.4, 7.9, 15.2, 29.5] {
            let jd = JD + offset;
            let d = day(&mut eph, jd, None);
            assert!(d.day_start_jd_ut <= jd && jd < d.day_end_jd_ut, "{offset}");
            let p = compute_panchang(&mut eph, jd, 41.0, 29.0, &s).unwrap();
            assert_eq!(d.vara, p.vara);
            assert_eq!(d.sunrise_jd_ut, p.sunrise_jd_ut);
            assert_eq!(d.sunset_jd_ut, p.sunset_jd_ut);
            for t in &d.tithis {
                let mid = 0.5 * (t.start_jd_ut + t.end_jd_ut);
                let p = compute_panchang(&mut eph, mid, 41.0, 29.0, &s).unwrap();
                assert_eq!((t.value.no, t.value.paksha), (p.tithi.no, p.tithi.paksha));
                assert_eq!(p.vara, d.vara, "the whole day has one vara");
            }
            for n in &d.nakshatras {
                let mid = 0.5 * (n.start_jd_ut + n.end_jd_ut);
                let p = compute_panchang(&mut eph, mid, 41.0, 29.0, &s).unwrap();
                assert_eq!(
                    (n.value.no, n.value.pada),
                    (p.nakshatra.no, p.nakshatra.pada)
                );
            }
            for y in &d.yogas {
                let mid = 0.5 * (y.start_jd_ut + y.end_jd_ut);
                let p = compute_panchang(&mut eph, mid, 41.0, 29.0, &s).unwrap();
                assert_eq!(y.value, p.yoga.no);
            }
            for k in &d.karanas {
                let mid = 0.5 * (k.start_jd_ut + k.end_jd_ut);
                let p = compute_panchang(&mut eph, mid, 41.0, 29.0, &s).unwrap();
                assert_eq!((k.value.no, k.value.name), (p.karana.no, p.karana.name));
            }
        }
    }

    #[test]
    fn flags_are_intervals_inside_the_day_in_a_fixed_order() {
        let mut eph = FakeEph::default();
        let d = day(&mut eph, JD, None);
        // Vishti (15. karana) uzanım 84°–90° arasındadır.
        assert_eq!(d.flags.len(), 1);
        let vishti = &d.flags[0];
        assert_eq!(vishti.value, MuhurtaFlag::Vishti);
        assert!((vishti.start_jd_ut - (JD - 1.0 / 12.22)).abs() < TOL);
        assert!((vishti.end_jd_ut - (JD + 5.0 / 12.22)).abs() < TOL);
        // Ertesi gün (93,19° → 105,41°): 96°'de 9. tithi (Rikta) başlar ve gün sonuna sürer.
        let d = day(&mut eph, JD + 1.0, None);
        assert_eq!(d.flags.len(), 1);
        let rikta = &d.flags[0];
        assert_eq!(rikta.value, MuhurtaFlag::Rikta);
        assert!((rikta.start_jd_ut - (JD + 11.0 / 12.22)).abs() < TOL);
        assert_eq!(rikta.end_jd_ut, d.day_end_jd_ut);

        // Ay 300°'yi geçerken panchaka başlar. Sahte Ay sidereal 71° + 13,2°/gün: 300°'ye 17,35. günde varır.
        let d = day(&mut eph, JD + 17.5, None);
        let panchaka: Vec<_> = d
            .flags
            .iter()
            .filter(|f| f.value == MuhurtaFlag::Panchaka)
            .collect();
        assert_eq!(panchaka.len(), 1);
        assert!((panchaka[0].start_jd_ut - (JD + 229.0 / 13.2)).abs() < TOL);
        assert_eq!(panchaka[0].end_jd_ut, d.day_end_jd_ut);
        // Sıra `DAY_FLAG_KINDS` sırasıdır.
        let order: Vec<usize> = d
            .flags
            .iter()
            .map(|f| DAY_FLAG_KINDS.iter().position(|k| *k == f.value).unwrap())
            .collect();
        assert!(order.windows(2).all(|w| w[0] <= w[1]));
        assert!(d.flags.iter().all(|f| !f.value.is_day_segment()));

        // Yanık Jüpiter: Güneş'i Jüpiter'in 5° yanına koy (orb 11°).
        let mut near = FakeEph::default().with(Body::Sun, 245.0, 0.98);
        let d = day(&mut near, JD, None);
        let combust: Vec<_> = d
            .flags
            .iter()
            .filter(|f| f.value == MuhurtaFlag::JupiterCombust)
            .collect();
        assert_eq!(combust.len(), 1);
        assert_eq!(
            (combust[0].start_jd_ut, combust[0].end_jd_ut),
            (d.day_start_jd_ut, d.day_end_jd_ut)
        );
    }

    #[test]
    fn personal_factors_follow_the_moon_through_the_day() {
        let mut eph = FakeEph::default();
        // Doğum Ay'ı: Rohini (4.), Boğa.
        let natal = NatalMoon {
            nakshatra: 4,
            sign: Sign::Taurus,
        };
        let d = day(&mut eph, JD, Some(natal));
        let p = d.personal.as_ref().unwrap();
        partitions(&p.tara, &d);
        partitions(&p.chandra, &d);
        // Gün içinde Ay Mrigashira (5.) → Ardra (6.): Rohini'den sayım 2 (Sampat) → 3 (Vipat).
        assert_eq!(
            p.tara.iter().map(|t| t.value).collect::<Vec<_>>(),
            [
                TaraBala {
                    number: 2,
                    favourable: true
                },
                TaraBala {
                    number: 3,
                    favourable: false
                }
            ]
        );
        assert!((p.tara[1].start_jd_ut - (JD - (71.0 - 200.0 / 3.0) / 13.2)).abs() < TOL);
        // Ay İkizler'de (60°–90°): Boğa'dan 2. ev, elverişli değil; gün boyu değişmez.
        assert_eq!(
            p.chandra.iter().map(|c| c.value).collect::<Vec<_>>(),
            [ChandraBala {
                house: 2,
                favourable: false
            }]
        );
        assert_eq!(
            day_panchang(
                &mut eph,
                JD,
                41.0,
                29.0,
                &CalcSettings::default(),
                Some(NatalMoon {
                    nakshatra: 28,
                    sign: Sign::Aries
                })
            ),
            Err(CoreError::InvalidMuhurtaInput("natal"))
        );
    }

    #[test]
    fn without_a_sunrise_the_day_starts_at_six_local_mean_time_and_has_no_segments() {
        let mut eph = FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        let d = day(&mut eph, JD, None);
        assert_eq!(d.warnings, [ChartWarning::SunriseUnavailable]);
        assert_eq!(
            (d.sunrise_jd_ut, d.sunset_jd_ut, d.next_sunrise_jd_ut),
            (None, None, None)
        );
        assert_eq!(d.segments, None);
        // Boylam 29°: 06:00 yerel ortalama saat = n + 0,6694 UT.
        let six = 2_451_544.75 - 29.0 / 360.0;
        assert!((d.day_start_jd_ut - six).abs() < 1e-9);
        assert!((d.day_end_jd_ut - (six + 1.0)).abs() < 1e-9);
        assert_eq!(d.vara, Vara::Saturday);
        partitions(&d.tithis, &d);
        partitions(&d.karanas, &d);
        // 06:00'dan önceki an önceki güne aittir.
        let before = day(&mut eph, JD - 0.4, None);
        assert!((before.day_end_jd_ut - six).abs() < 1e-9);
        assert_eq!(before.vara, Vara::Friday);
    }

    #[test]
    fn low_precision_and_invalid_input_are_reported_without_the_place() {
        let mut eph = FakeEph {
            precision: Precision::Moshier,
            ..FakeEph::default()
        };
        assert_eq!(
            day(&mut eph, JD, None).warnings,
            [ChartWarning::LowPrecision]
        );
        let s = CalcSettings::default();
        let mut eph = FakeEph::default();
        for (lat, lon) in [(91.0, 29.0), (41.0, 181.0), (f64::NAN, 29.0)] {
            assert_eq!(
                day_panchang(&mut eph, JD, lat, lon, &s, None),
                Err(CoreError::InvalidMuhurtaInput("place"))
            );
        }
        assert_eq!(
            day_panchang(&mut eph, 1e9, 41.0, 29.0, &s, None),
            Err(CoreError::InvalidJulianDay(1e9))
        );
        let old = CalcSettings {
            schema_version: 1,
            ..CalcSettings::default()
        };
        assert!(matches!(
            day_panchang(&mut eph, JD, 41.0, 29.0, &old, None),
            Err(CoreError::UnsupportedSchemaVersion { .. })
        ));
    }
}
````


- [ ] **Step 4: Pencere araması**

Her etkin koşul bir dizi ister; pencereler kesişimdir. Dilim sınırına değen pencere verilmez, başı imleçte taşınır (D17). `boundaries_of_two_series_that_fall_on_the_same_moment_leave_no_sliver_window` testi 1 saniye kuralının nedenidir: kural kaldırılırsa 0,7 günlük dilimlerle bir pencere fazla çıkar.

**Yeni dosya:** `crates/jyotish-core/src/muhurta/search.rs`

````rust
//! Pencere araması (ileri modüller spec'i §10): süzgece uyan zaman pencereleri. Her etkin
//! koşul `timeline`'dan bir aralık dizisi ister; pencereler bu dizilerin kesişimidir.
//!
//! Arama dilim dilim yürür (`MUHURTA_CHUNK_DAYS`): her çağrı bir dilimi hesaplar ve sonraki
//! çağrının imlecini döner. Dilim sınırını aşan pencere bölünmez: sınırda açık duran pencerenin
//! başı imleçte taşınır ve pencere kapandığı dilimde verilir. Çekirdek durum tutmaz; aramayı
//! durdurmak, sonraki dilimi istememektir.

use ephemeris::{Body, Ephemeris, Precision, Zodiac};
use serde::{Deserialize, Serialize};

use crate::chart::{finite, rasi_frame, ChartWarning};
use crate::graha::Graha;
use crate::panchang::{
    karana_name, karana_no, nitya_yoga_no, paksha, tithi_no, vara_at, KaranaName, Paksha, Vara,
};
use crate::settings::CalcSettings;
use crate::timeline::{
    check_span, combust_series, graha_sign_series, karana_series, lagna_sign_at, lagna_sign_series,
    nakshatra_series, nitya_yoga_series, sun_days, sun_days_cover, tithis_of_karanas,
    vara_of_sun_days, vara_series, Interval,
};
use crate::zodiac::{nakshatra_pada, normalize, Sign};
use crate::CoreError;

use super::segments::{
    chandra_bala, day_segments, is_panchaka, is_rikta, is_vishti, tara_bala, VAIDHRITI_YOGA,
    VYATIPATA_YOGA,
};
use super::{MuhurtaFlag, NatalMoon, MUHURTA_METHOD};

/// Aramanın en uzun tarih aralığı, gün.
pub const MUHURTA_MAX_RANGE_DAYS: f64 = 366.0;
/// Bir çağrının hesapladığı dilim, gün. Motor iş parçacığı dilim aralarında bırakılır.
pub const MUHURTA_CHUNK_DAYS: f64 = 31.0;
/// "En kısa süre" süzgecinin üst sınırı, dakika (bir hafta).
pub const MUHURTA_MAX_MIN_MINUTES: u32 = 7 * 24 * 60;
/// Bundan kısa pencere verilmez (1 saniye). İki ayrı dizinin aynı ana düşen sınırları (ör. Ay'ın
/// burç girişi ile nakshatra girişi) ayrı ikiye bölmelerle bulunur ve çözünürlük kadar
/// ayrışabilir; biri kapanırken diğeri açılan iki koşul arasında oluşan bu kıymık pencere değildir.
pub const MUHURTA_MIN_WINDOW_DAYS: f64 = 1.0 / 86_400.0;

/// Süzgecin saklanabilen kısmı: koşullar. Boş küme "kısıt yok" demektir. Tarih aralığı, yer ve
/// kişi burada değildir (kayıtlı süzgeç kişi ve yer verisi taşımaz).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MuhurtaCriteria {
    pub weekdays: Vec<Vara>,
    /// 1..=30 (1–15 Shukla, 16–30 Krishna).
    pub tithis: Vec<u8>,
    /// 1..=27.
    pub nakshatras: Vec<u8>,
    pub lagna_signs: Vec<Sign>,
    /// Yalnızca gün doğumu ile batımı arası.
    pub daytime_only: bool,
    /// Pencerenin en kısa süresi, dakika; 0 kısıt yok.
    pub min_minutes: u32,
    pub exclude: Vec<MuhurtaFlag>,
    /// Tara bala elverişli (doğum Ay'ı gerekir).
    pub require_tara: bool,
    /// Chandra bala elverişli (doğum Ay'ı gerekir).
    pub require_chandra: bool,
}

impl MuhurtaCriteria {
    /// Değer aralıklarını denetler; hata alan adını taşır.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.tithis.iter().any(|t| !(1..=30).contains(t)) {
            return Err(CoreError::InvalidMuhurtaInput("tithis"));
        }
        if self.nakshatras.iter().any(|n| !(1..=27).contains(n)) {
            return Err(CoreError::InvalidMuhurtaInput("nakshatras"));
        }
        if self.min_minutes > MUHURTA_MAX_MIN_MINUTES {
            return Err(CoreError::InvalidMuhurtaInput("min_minutes"));
        }
        Ok(())
    }

    /// Kümeleri sıralar ve yinelenenleri atar (kayıt ve karşılaştırma için kanonik biçim).
    pub fn normalized(mut self) -> Self {
        fn tidy<T: Ord>(v: &mut Vec<T>) {
            v.sort();
            v.dedup();
        }
        self.weekdays.sort_by_key(|v| *v as u8);
        self.weekdays.dedup();
        tidy(&mut self.tithis);
        tidy(&mut self.nakshatras);
        tidy(&mut self.lagna_signs);
        tidy(&mut self.exclude);
        self
    }

    fn excludes(&self, flag: MuhurtaFlag) -> bool {
        self.exclude.contains(&flag)
    }
}

/// Aramanın tam girdisi: tarih aralığı, yer, koşullar ve (kişiye bağlı koşullar için) doğum
/// Ay'ı.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MuhurtaFilter {
    pub from_jd_ut: f64,
    pub to_jd_ut: f64,
    /// Kuzey +.
    pub lat: f64,
    /// Doğu +.
    pub lon: f64,
    pub criteria: MuhurtaCriteria,
    pub natal: Option<NatalMoon>,
}

impl MuhurtaFilter {
    pub fn validate(&self) -> Result<(), CoreError> {
        check_span(self.from_jd_ut, self.to_jd_ut)?;
        if self.to_jd_ut - self.from_jd_ut > MUHURTA_MAX_RANGE_DAYS {
            return Err(CoreError::InvalidMuhurtaInput("range"));
        }
        if !((-90.0..=90.0).contains(&self.lat) && (-180.0..=180.0).contains(&self.lon)) {
            return Err(CoreError::InvalidMuhurtaInput("place"));
        }
        self.criteria.validate()?;
        match &self.natal {
            Some(n) => n.validate(),
            None if self.criteria.require_tara || self.criteria.require_chandra => {
                Err(CoreError::InvalidMuhurtaInput("natal"))
            }
            None => Ok(()),
        }
    }
}

/// Pencerenin başındaki beş uzuv ve lagna burcu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct WindowSnapshot {
    pub vara: Vara,
    pub tithi: u8,
    pub paksha: Paksha,
    pub nakshatra: u8,
    pub pada: u8,
    pub yoga: u8,
    pub karana: u8,
    pub karana_name: KaranaName,
    pub lagna_sign: Sign,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MuhurtaWindow {
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
    pub snapshot: WindowSnapshot,
}

/// Sonraki dilimin imleci. Arayüz bunu olduğu gibi geri gönderir.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MuhurtaCursor {
    /// Hesaplanacak dilimin sırası, 0'dan.
    pub chunk: u32,
    /// Önceki dilimin sonunda açık duran pencerenin başı.
    pub open_start_jd_ut: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MuhurtaSearchPage {
    /// Bu dilimde kapanan pencereler, başlangıç sırasıyla.
    pub windows: Vec<MuhurtaWindow>,
    /// Taranan dilim sayısı (bu dilim dahil) ve toplam.
    pub scanned: u32,
    pub chunks: u32,
    /// Sonraki dilimin imleci; arama bittiyse `None`.
    pub next: Option<MuhurtaCursor>,
    /// `SunriseUnavailable`: dilimde doğuşu ya da batışı olmayan gün var ve süzgeç gündüze ya da
    /// gündüz dilimlerine bağlı. `LowPrecision`: Moshier yedeği.
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

/// Aralığın dilim sayısı.
pub fn muhurta_chunks(from_jd_ut: f64, to_jd_ut: f64) -> u32 {
    chunk_count(from_jd_ut, to_jd_ut, MUHURTA_CHUNK_DAYS)
}

fn chunk_count(from: f64, to: f64, chunk_days: f64) -> u32 {
    (((to - from) / chunk_days).ceil() as u32).max(1)
}

type Spans = Vec<(f64, f64)>;

/// Koşulu sağlayan aralıklar, birleştirilmiş.
fn spans_where<T>(series: &[Interval<T>], holds: impl Fn(&T) -> bool) -> Spans {
    let mut out: Spans = Vec::new();
    for i in series.iter().filter(|i| holds(&i.value)) {
        match out.last_mut() {
            Some(last) if last.1 == i.start_jd_ut => last.1 = i.end_jd_ut,
            _ => out.push((i.start_jd_ut, i.end_jd_ut)),
        }
    }
    out
}

/// İki sıralı, ayrık aralık listesinin kesişimi.
fn intersect(a: &Spans, b: &Spans) -> Spans {
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < a.len() && j < b.len() {
        let start = a[i].0.max(b[j].0);
        let end = a[i].1.min(b[j].1);
        if start < end {
            out.push((start, end));
        }
        if a[i].1 <= b[j].1 {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

/// `a`'dan `cut` aralıklarının çıkarılması (`cut` sıralı ve ayrık).
fn subtract(a: &Spans, cut: &Spans) -> Spans {
    let mut out = Vec::new();
    for &(mut start, end) in a {
        for &(c0, c1) in cut {
            if c1 <= start || c0 >= end {
                continue;
            }
            if c0 > start {
                out.push((start, c0));
            }
            start = start.max(c1);
        }
        if start < end {
            out.push((start, end));
        }
    }
    out
}

/// `jd_ut` anının beş uzvu ve lagna burcu (noktasal; `compute_panchang` ve `compute_chart`
/// ile aynı sorgular).
pub fn window_snapshot<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    jd_ut: f64,
    geo_lat: f64,
    geo_lon: f64,
) -> Result<WindowSnapshot, CoreError> {
    let zodiac = Zodiac::Sidereal(settings.ayanamsa.sidereal_mode());
    let sun = finite(eph.body(jd_ut, Body::Sun, zodiac)?.lon, "panchang")?;
    let moon = finite(eph.body(jd_ut, Body::Moon, zodiac)?.lon, "panchang")?;
    let elongation = normalize(moon - sun);
    let tithi = tithi_no(elongation);
    let (nakshatra, pada) = nakshatra_pada(moon);
    let karana = karana_no(elongation);
    Ok(WindowSnapshot {
        vara: vara_at(
            eph,
            jd_ut,
            geo_lat,
            geo_lon,
            settings.sunrise_def.sun_disc(),
        )?,
        tithi,
        paksha: paksha(tithi),
        nakshatra,
        pada,
        yoga: nitya_yoga_no(sun, moon),
        karana,
        karana_name: karana_name(karana),
        lagna_sign: lagna_sign_at(eph, settings, jd_ut, geo_lat, geo_lon)?,
    })
}

/// Bir dilimde süzgeci sağlayan aralıklar ve uyarılar.
fn allowed_spans<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    filter: &MuhurtaFilter,
    a: f64,
    b: f64,
) -> Result<(Spans, Vec<ChartWarning>), CoreError> {
    let c = &filter.criteria;
    let (lat, lon) = (filter.lat, filter.lon);
    let mut allowed: Spans = vec![(a, b)];
    let mut warnings = Vec::new();
    if eph.body(a, Body::Sun, rasi_frame(settings))?.precision == Precision::Moshier {
        warnings.push(ChartWarning::LowPrecision);
    }

    let segment_flags: Vec<MuhurtaFlag> = MuhurtaFlag::ALL
        .into_iter()
        .filter(|f| f.is_day_segment() && c.excludes(*f))
        .collect();
    let needs_days = c.daytime_only || !segment_flags.is_empty();
    if needs_days || !c.weekdays.is_empty() {
        let days = sun_days(eph, settings, lat, lon, a, b)?;
        let covered = sun_days_cover(&days, a, b);
        if !c.weekdays.is_empty() {
            let varas = if covered {
                vara_of_sun_days(&days, a, b)
            } else {
                vara_series(eph, settings, lat, lon, a, b)?
            };
            allowed = intersect(&allowed, &spans_where(&varas, |v| c.weekdays.contains(v)));
        }
        if needs_days && !covered {
            warnings.push(ChartWarning::SunriseUnavailable);
        }
        if c.daytime_only {
            let daylight: Spans = days
                .iter()
                .map(|d| (d.sunrise_jd_ut, d.sunset_jd_ut))
                .collect();
            allowed = intersect(&allowed, &daylight);
        }
        if !segment_flags.is_empty() {
            let mut cut: Spans = Vec::new();
            for d in &days {
                let s = day_segments(d);
                let mut of_day: Spans = segment_flags
                    .iter()
                    .map(|f| match f {
                        MuhurtaFlag::RahuSegment => s.rahu,
                        MuhurtaFlag::Yamaganda => s.yamaganda,
                        _ => s.gulika,
                    })
                    .map(|i| (i.start_jd_ut, i.end_jd_ut))
                    .collect();
                of_day.sort_by(|x, y| x.0.total_cmp(&y.0));
                cut.extend(of_day);
            }
            allowed = subtract(&allowed, &cut);
        }
    }

    let (rikta, vishti) = (
        c.excludes(MuhurtaFlag::Rikta),
        c.excludes(MuhurtaFlag::Vishti),
    );
    if !c.tithis.is_empty() || rikta || vishti {
        let karanas = karana_series(eph, settings, a, b)?;
        if vishti {
            allowed = intersect(&allowed, &spans_where(&karanas, |k| !is_vishti(*k)));
        }
        if !c.tithis.is_empty() || rikta {
            let tithis = tithis_of_karanas(&karanas);
            allowed = intersect(
                &allowed,
                &spans_where(&tithis, |t| {
                    (c.tithis.is_empty() || c.tithis.contains(t)) && !(rikta && is_rikta(*t))
                }),
            );
        }
    }

    let panchaka = c.excludes(MuhurtaFlag::Panchaka);
    if !c.nakshatras.is_empty() || panchaka || c.require_tara {
        let series = nakshatra_series(eph, settings, a, b)?;
        allowed = intersect(
            &allowed,
            &spans_where(&series, |n| {
                (c.nakshatras.is_empty() || c.nakshatras.contains(&n.nakshatra))
                    && !(panchaka && is_panchaka(*n))
                    && match (&filter.natal, c.require_tara) {
                        (Some(natal), true) => tara_bala(natal.nakshatra, n.nakshatra).favourable,
                        _ => true,
                    }
            }),
        );
    }

    let (vyatipata, vaidhriti) = (
        c.excludes(MuhurtaFlag::Vyatipata),
        c.excludes(MuhurtaFlag::Vaidhriti),
    );
    if vyatipata || vaidhriti {
        let yogas = nitya_yoga_series(eph, settings, a, b)?;
        allowed = intersect(
            &allowed,
            &spans_where(&yogas, |y| {
                !(vyatipata && *y == VYATIPATA_YOGA) && !(vaidhriti && *y == VAIDHRITI_YOGA)
            }),
        );
    }

    if let (Some(natal), true) = (&filter.natal, c.require_chandra) {
        let signs = graha_sign_series(eph, settings, Graha::Moon, a, b)?;
        allowed = intersect(
            &allowed,
            &spans_where(&signs, |s| chandra_bala(natal.sign, *s).favourable),
        );
    }

    for (flag, graha) in [
        (MuhurtaFlag::JupiterCombust, Graha::Jupiter),
        (MuhurtaFlag::VenusCombust, Graha::Venus),
    ] {
        if c.excludes(flag) {
            let series = combust_series(eph, settings, graha, a, b)?;
            allowed = intersect(&allowed, &spans_where(&series, |combust| !combust));
        }
    }

    // En pahalı dizi sona kalır: diğer koşullar hiçbir an bırakmadıysa hesaplanmaz.
    if !c.lagna_signs.is_empty() && !allowed.is_empty() {
        let series = lagna_sign_series(eph, settings, lat, lon, a, b)?;
        allowed = intersect(
            &allowed,
            &spans_where(&series, |s| c.lagna_signs.contains(s)),
        );
    }
    Ok((allowed, warnings))
}

/// Aramanın bir dilimi. `cursor` yoksa ilk dilim hesaplanır; dönen `next` bir sonraki çağrıya
/// olduğu gibi verilir. Aynı süzgeç ve imleç her zaman aynı sayfayı verir.
pub fn muhurta_search_page<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    filter: &MuhurtaFilter,
    cursor: Option<MuhurtaCursor>,
) -> Result<MuhurtaSearchPage, CoreError> {
    search_page_with(eph, settings, filter, cursor, MUHURTA_CHUNK_DAYS)
}

pub(crate) fn search_page_with<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    filter: &MuhurtaFilter,
    cursor: Option<MuhurtaCursor>,
    chunk_days: f64,
) -> Result<MuhurtaSearchPage, CoreError> {
    settings.validate()?;
    filter.validate()?;
    let (from, to) = (filter.from_jd_ut, filter.to_jd_ut);
    let chunks = chunk_count(from, to, chunk_days);
    let cursor = cursor.unwrap_or(MuhurtaCursor {
        chunk: 0,
        open_start_jd_ut: None,
    });
    if cursor.chunk >= chunks {
        return Err(CoreError::InvalidMuhurtaInput("cursor"));
    }
    let a = from + f64::from(cursor.chunk) * chunk_days;
    let last = cursor.chunk + 1 == chunks;
    let b = if last {
        to
    } else {
        from + f64::from(cursor.chunk + 1) * chunk_days
    };
    // Açık pencerenin başı aralığın içinde ve bu dilimden önce olmalıdır.
    if cursor
        .open_start_jd_ut
        .is_some_and(|s| !(s >= from && s < a))
    {
        return Err(CoreError::InvalidMuhurtaInput("cursor"));
    }

    let (allowed, warnings) = allowed_spans(eph, settings, filter, a, b)?;
    let min_days = (f64::from(filter.criteria.min_minutes) / 1440.0).max(MUHURTA_MIN_WINDOW_DAYS);
    let mut closed: Spans = Vec::new();
    let mut open = cursor.open_start_jd_ut;
    let mut carry = None;
    for (start, end) in allowed {
        let start = match open.take() {
            // Önceki dilimde açık duran pencere bu dilimin başında sürüyor.
            Some(o) if start == a => o,
            // Sürmüyor: tam dilim sınırında kapanmış.
            Some(o) => {
                closed.push((o, a));
                start
            }
            None => start,
        };
        if end == b && !last {
            carry = Some(start);
        } else {
            closed.push((start, end));
        }
    }
    if let Some(o) = open {
        closed.push((o, a));
    }

    let mut windows = Vec::new();
    for (start, end) in closed {
        if end - start >= min_days {
            windows.push(MuhurtaWindow {
                start_jd_ut: start,
                end_jd_ut: end,
                snapshot: window_snapshot(eph, settings, start, filter.lat, filter.lon)?,
            });
        }
    }
    Ok(MuhurtaSearchPage {
        windows,
        scanned: cursor.chunk + 1,
        chunks,
        next: (!last).then_some(MuhurtaCursor {
            chunk: cursor.chunk + 1,
            open_start_jd_ut: carry,
        }),
        warnings,
        method: MUHURTA_METHOD.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::panchang::compute_panchang;

    const JD: f64 = 2_451_545.0;
    const TOL: f64 = 0.2 / 86_400.0;

    fn eph() -> FakeEph {
        FakeEph {
            asc_speed: 360.0,
            ..FakeEph::default()
        }
    }

    fn filter(days: f64, criteria: MuhurtaCriteria) -> MuhurtaFilter {
        MuhurtaFilter {
            from_jd_ut: JD,
            to_jd_ut: JD + days,
            lat: 41.0,
            lon: 29.0,
            criteria,
            natal: None,
        }
    }

    /// Bütün dilimleri sırayla ister.
    fn run(eph: &mut FakeEph, f: &MuhurtaFilter, chunk_days: f64) -> Vec<MuhurtaWindow> {
        let s = CalcSettings::default();
        let mut out = Vec::new();
        let mut cursor = None;
        let mut pages = 0;
        loop {
            let page = search_page_with(eph, &s, f, cursor, chunk_days).unwrap();
            pages += 1;
            assert_eq!(page.scanned, pages);
            assert_eq!(page.method, MUHURTA_METHOD);
            out.extend(page.windows);
            match page.next {
                Some(next) => cursor = Some(next),
                None => {
                    assert_eq!(page.scanned, page.chunks);
                    return out;
                }
            }
        }
    }

    #[test]
    fn span_sets_intersect_and_subtract() {
        let a: Spans = vec![(0.0, 4.0), (6.0, 10.0)];
        let b: Spans = vec![(1.0, 2.0), (3.0, 7.0), (9.0, 12.0)];
        assert_eq!(
            intersect(&a, &b),
            [(1.0, 2.0), (3.0, 4.0), (6.0, 7.0), (9.0, 10.0)]
        );
        assert_eq!(intersect(&a, &Vec::new()), []);
        assert_eq!(subtract(&a, &b), [(0.0, 1.0), (2.0, 3.0), (7.0, 9.0)]);
        assert_eq!(subtract(&a, &vec![(-1.0, 20.0)]), []);
        assert_eq!(subtract(&a, &Vec::new()), a);
        // Uçları değen aralıklar birleşir; koşulu sağlamayan araya girerse birleşmez.
        let series = [
            Interval {
                value: 1u8,
                start_jd_ut: 0.0,
                end_jd_ut: 1.0,
            },
            Interval {
                value: 2,
                start_jd_ut: 1.0,
                end_jd_ut: 2.0,
            },
            Interval {
                value: 9,
                start_jd_ut: 2.0,
                end_jd_ut: 3.0,
            },
            Interval {
                value: 3,
                start_jd_ut: 3.0,
                end_jd_ut: 4.0,
            },
        ];
        assert_eq!(spans_where(&series, |v| *v < 5), [(0.0, 2.0), (3.0, 4.0)]);
    }

    #[test]
    fn no_condition_gives_one_window_over_the_whole_range_across_chunks() {
        let mut e = eph();
        let f = filter(100.0, MuhurtaCriteria::default());
        for chunk in [31.0, 7.0, 100.0, 250.0] {
            let w = run(&mut e, &f, chunk);
            assert_eq!(w.len(), 1, "chunk {chunk}");
            assert_eq!((w[0].start_jd_ut, w[0].end_jd_ut), (JD, JD + 100.0));
        }
        assert_eq!(muhurta_chunks(JD, JD + 100.0), 4);
        assert_eq!(muhurta_chunks(JD, JD + 31.0), 1);
        assert_eq!(muhurta_chunks(JD, JD + 366.0), 12);
        // İlk sayfa pencereyi vermez: pencere son dilimde kapanır.
        let s = CalcSettings::default();
        let first = muhurta_search_page(&mut e, &s, &f, None).unwrap();
        assert!(first.windows.is_empty());
        assert_eq!((first.scanned, first.chunks), (1, 4));
        assert_eq!(
            first.next,
            Some(MuhurtaCursor {
                chunk: 1,
                open_start_jd_ut: Some(JD)
            })
        );
    }

    #[test]
    fn a_tithi_filter_finds_the_tithi_and_the_snapshot_describes_its_start() {
        // Uzanım 85° + 12,22°/gün: 9. tithi 96°–108°; 29,46 gün sonra yeniden.
        let mut e = eph();
        let f = filter(
            40.0,
            MuhurtaCriteria {
                tithis: vec![9],
                ..MuhurtaCriteria::default()
            },
        );
        let w = run(&mut e, &f, MUHURTA_CHUNK_DAYS);
        assert_eq!(w.len(), 2);
        assert!((w[0].start_jd_ut - (JD + 11.0 / 12.22)).abs() < TOL);
        assert!((w[0].end_jd_ut - (JD + 23.0 / 12.22)).abs() < TOL);
        assert!((w[1].start_jd_ut - (JD + 371.0 / 12.22)).abs() < TOL);
        let s = CalcSettings::default();
        for win in &w {
            let snap = win.snapshot;
            assert_eq!((snap.tithi, snap.paksha), (9, Paksha::Shukla));
            // Karana 17: (17 − 2) mod 7 = 1 → Balava.
            assert_eq!((snap.karana, snap.karana_name), (17, KaranaName::Balava));
            let p = compute_panchang(&mut e, win.start_jd_ut, 41.0, 29.0, &s).unwrap();
            assert_eq!(snap.vara, p.vara);
            assert_eq!(
                (snap.nakshatra, snap.pada),
                (p.nakshatra.no, p.nakshatra.pada)
            );
            assert_eq!(snap.yoga, p.yoga.no);
            assert_eq!(
                snap.lagna_sign,
                lagna_sign_at(&mut e, &s, win.start_jd_ut, 41.0, 29.0).unwrap()
            );
        }
        // İkinci pencere iki dilimi aşar (31. gün sınırı 30,36–31,34 arasındadır); dilim
        // uzunluğu sonucu değiştirmez.
        let small = run(&mut e, &f, 3.0);
        assert_eq!(small.len(), 2);
        for (x, y) in w.iter().zip(&small) {
            assert_eq!(x, y);
        }
    }

    #[test]
    fn daytime_and_segment_exclusions_cut_the_day() {
        // Sahte gün: doğuş n + 0,67, batış n + 1,15 (gündüz 0,48 gün; sekizde bir 0,06 gün).
        let mut e = eph();
        let day_only = MuhurtaCriteria {
            daytime_only: true,
            ..MuhurtaCriteria::default()
        };
        let w = run(&mut e, &filter(3.0, day_only.clone()), 31.0);
        // JD'de gündüzün içindeyiz: [JD, +0,15), sonra üç tam gündüz… sonuncusu aralıkta kesilir.
        assert_eq!(w.len(), 4);
        assert_eq!(w[0].start_jd_ut, JD);
        assert!((w[0].end_jd_ut - (JD + 0.15)).abs() < 1e-9);
        assert!((w[1].start_jd_ut - (JD + 0.67)).abs() < 1e-9);
        assert!((w[1].end_jd_ut - (JD + 1.15)).abs() < 1e-9);
        assert_eq!(w[3].end_jd_ut, JD + 3.0);

        // JD + 0,67'de başlayan gün Pazar'dır: Rahu 8., Yamaganda 5., Gulika 7. dilim.
        let cut = MuhurtaCriteria {
            exclude: vec![
                MuhurtaFlag::Gulika,
                MuhurtaFlag::RahuSegment,
                MuhurtaFlag::Yamaganda,
            ],
            ..day_only
        };
        let f = MuhurtaFilter {
            from_jd_ut: JD + 0.67,
            to_jd_ut: JD + 1.15,
            ..filter(1.0, cut)
        };
        let w = run(&mut e, &f, 31.0);
        let rise = JD + 0.67;
        let eighth = 0.06;
        // Geriye 1.–4. dilim ve 6. dilim durur.
        assert_eq!(w.len(), 2);
        assert!((w[0].start_jd_ut - rise).abs() < 1e-9);
        assert!((w[0].end_jd_ut - (rise + 4.0 * eighth)).abs() < 1e-9);
        assert!((w[1].start_jd_ut - (rise + 5.0 * eighth)).abs() < 1e-9);
        assert!((w[1].end_jd_ut - (rise + 6.0 * eighth)).abs() < 1e-9);
        assert_eq!(w[0].snapshot.vara, Vara::Sunday);
    }

    #[test]
    fn weekday_lagna_and_minimum_length_combine() {
        // Lagna sidereal 46° + 360°/gün: Yengeç 90°–120°, her gün 44/360'ta başlar ve 2 saat
        // sürer. Pazartesi günü JD + 1,67'de başlar.
        let mut e = eph();
        let criteria = MuhurtaCriteria {
            weekdays: vec![Vara::Monday],
            lagna_signs: vec![Sign::Cancer],
            ..MuhurtaCriteria::default()
        };
        let w = run(&mut e, &filter(10.0, criteria.clone()), 31.0);
        // 10 günde iki Pazartesi: JD + 1,67 … 2,67 ve JD + 8,67 … 9,67.
        assert_eq!(w.len(), 2);
        assert!((w[0].start_jd_ut - (JD + 2.0 + 44.0 / 360.0)).abs() < TOL);
        assert!((w[0].end_jd_ut - (JD + 2.0 + 74.0 / 360.0)).abs() < TOL);
        assert!((w[1].start_jd_ut - (JD + 9.0 + 44.0 / 360.0)).abs() < TOL);
        for win in &w {
            assert_eq!(win.snapshot.vara, Vara::Monday);
            assert_eq!(win.snapshot.lagna_sign, Sign::Cancer);
        }
        // Pencere 120 dakikadır: 120 dakikalık alt sınır geçer, 121 dakikalık geçmez.
        let at_least = |min_minutes| MuhurtaCriteria {
            min_minutes,
            ..criteria.clone()
        };
        assert_eq!(run(&mut e, &filter(10.0, at_least(119)), 31.0).len(), 2);
        assert_eq!(run(&mut e, &filter(10.0, at_least(121)), 31.0).len(), 0);
    }

    #[test]
    fn boundaries_of_two_series_that_fall_on_the_same_moment_leave_no_sliver_window() {
        // Panchaka Ay 360°'ye varınca biter; aynı anda Ay Balık'tan Koç'a geçer. Doğum Ay burcu
        // Boğa'ysa Balık elverişli (11. ev), Koç elverişsizdir (12. ev): biri açılırken diğeri
        // kapanır, pencere yoktur. İki sınır ayrı dizilerden gelir (nakshatra ve burç) ve
        // çözünürlük kadar ayrışabilir; dilim uzunluğu ne olursa olsun kıymık pencere verilmez.
        let mut e = eph();
        let f = MuhurtaFilter {
            natal: Some(NatalMoon {
                nakshatra: 4,
                sign: Sign::Taurus,
            }),
            ..filter(
                40.0,
                MuhurtaCriteria {
                    exclude: vec![MuhurtaFlag::Panchaka],
                    require_chandra: true,
                    ..MuhurtaCriteria::default()
                },
            )
        };
        let whole = run(&mut e, &f, 400.0);
        for chunk in [0.7, 1.3, 2.9, 7.0, 31.0] {
            let pieces = run(&mut e, &f, chunk);
            assert_eq!(pieces.len(), whole.len(), "chunk {chunk}");
        }
        for w in &whole {
            assert!(w.end_jd_ut - w.start_jd_ut >= MUHURTA_MIN_WINDOW_DAYS);
            // Hiçbir pencere Kova ya da Balık'ta (panchaka) değildir.
            assert!(!matches!(
                Sign::of(
                    e.body(
                        0.5 * (w.start_jd_ut + w.end_jd_ut),
                        Body::Moon,
                        Zodiac::Sidereal(ephemeris::SiderealMode::Lahiri)
                    )
                    .unwrap()
                    .lon
                ),
                Sign::Aquarius | Sign::Pisces
            ));
        }
        // Sahte Ay 27,27 günde bir döner: 40 günde elverişli altı burcun dördü (Boğa, Yengeç,
        // Terazi, Akrep) en az birer kez geçilir.
        assert!(whole.len() >= 4, "{}", whole.len());
    }

    #[test]
    fn personal_conditions_need_the_natal_moon() {
        let mut e = eph();
        let s = CalcSettings::default();
        let criteria = MuhurtaCriteria {
            require_tara: true,
            require_chandra: true,
            ..MuhurtaCriteria::default()
        };
        let without = filter(30.0, criteria.clone());
        assert_eq!(
            muhurta_search_page(&mut e, &s, &without, None),
            Err(CoreError::InvalidMuhurtaInput("natal"))
        );
        let natal = NatalMoon {
            nakshatra: 4,
            sign: Sign::Taurus,
        };
        let with = MuhurtaFilter {
            natal: Some(natal),
            ..without
        };
        let w = run(&mut e, &with, 31.0);
        assert!(!w.is_empty());
        for win in &w {
            for jd in [
                win.start_jd_ut,
                0.5 * (win.start_jd_ut + win.end_jd_ut),
                win.end_jd_ut - 2.0 * TOL,
            ] {
                let snap = window_snapshot(&mut e, &s, jd, 41.0, 29.0).unwrap();
                assert!(tara_bala(natal.nakshatra, snap.nakshatra).favourable);
                let moon = graha_sign_series(&mut e, &s, Graha::Moon, jd, jd + TOL).unwrap();
                assert!(chandra_bala(natal.sign, moon[0].value).favourable);
            }
        }
    }

    #[test]
    fn excluded_flags_never_hold_inside_a_window() {
        let mut e = eph();
        let s = CalcSettings::default();
        let criteria = MuhurtaCriteria {
            exclude: vec![
                MuhurtaFlag::Rikta,
                MuhurtaFlag::Vishti,
                MuhurtaFlag::Vyatipata,
                MuhurtaFlag::Vaidhriti,
                MuhurtaFlag::Panchaka,
            ],
            ..MuhurtaCriteria::default()
        };
        let w = run(&mut e, &filter(60.0, criteria), 31.0);
        assert!(w.len() > 10);
        let mut covered = 0.0;
        for win in &w {
            covered += win.end_jd_ut - win.start_jd_ut;
            for k in 0..5 {
                let jd = win.start_jd_ut
                    + (win.end_jd_ut - win.start_jd_ut - 2.0 * TOL) * f64::from(k) / 4.0;
                let snap = window_snapshot(&mut e, &s, jd, 41.0, 29.0).unwrap();
                assert!(!is_rikta(snap.tithi), "rikta at {jd}");
                assert_ne!(snap.karana_name, KaranaName::Vishti);
                assert!(snap.yoga != VYATIPATA_YOGA && snap.yoga != VAIDHRITI_YOGA);
                assert!(!is_panchaka(crate::timeline::NakshatraPada {
                    nakshatra: snap.nakshatra,
                    pada: snap.pada
                }));
            }
        }
        for pair in w.windows(2) {
            assert!(pair[0].end_jd_ut < pair[1].start_jd_ut, "disjoint, ordered");
        }
        // Bir şey dışlandı, her şey değil.
        assert!(covered > 20.0 && covered < 55.0, "{covered}");
    }

    #[test]
    fn days_without_a_sunrise_give_no_daytime_windows_and_a_warning() {
        let mut polar = FakeEph {
            polar: true,
            ..eph()
        };
        let s = CalcSettings::default();
        let day_only = filter(
            5.0,
            MuhurtaCriteria {
                daytime_only: true,
                ..MuhurtaCriteria::default()
            },
        );
        let page = muhurta_search_page(&mut polar, &s, &day_only, None).unwrap();
        assert!(page.windows.is_empty());
        assert_eq!(page.warnings, [ChartWarning::SunriseUnavailable]);
        // Dilim dışlaması o günlerde uygulanamaz: pencere kesilmez, uyarı verilir.
        let rahu = filter(
            5.0,
            MuhurtaCriteria {
                exclude: vec![MuhurtaFlag::RahuSegment],
                ..MuhurtaCriteria::default()
            },
        );
        let page = muhurta_search_page(&mut polar, &s, &rahu, None).unwrap();
        assert_eq!(page.windows.len(), 1);
        assert_eq!(page.warnings, [ChartWarning::SunriseUnavailable]);
        // Vara koşulu gün doğumu olmadan da çalışır (06:00 yerel ortalama saat) ve uyarı vermez.
        let monday = filter(
            7.0,
            MuhurtaCriteria {
                weekdays: vec![Vara::Monday],
                ..MuhurtaCriteria::default()
            },
        );
        let page = muhurta_search_page(&mut polar, &s, &monday, None).unwrap();
        assert_eq!(page.windows.len(), 1);
        assert!(page.warnings.is_empty());
        assert!((page.windows[0].end_jd_ut - page.windows[0].start_jd_ut - 1.0).abs() < TOL);
    }

    #[test]
    fn invalid_filters_and_cursors_are_rejected_by_field() {
        let mut e = eph();
        let s = CalcSettings::default();
        let bad =
            |f: MuhurtaFilter, e: &mut FakeEph| muhurta_search_page(e, &s, &f, None).unwrap_err();
        let ok = MuhurtaCriteria::default();
        assert_eq!(
            bad(filter(366.5, ok.clone()), &mut e),
            CoreError::InvalidMuhurtaInput("range")
        );
        assert_eq!(
            bad(filter(-1.0, ok.clone()), &mut e),
            CoreError::InvalidMuhurtaInput("range")
        );
        assert_eq!(
            bad(
                MuhurtaFilter {
                    lat: 95.0,
                    ..filter(1.0, ok.clone())
                },
                &mut e
            ),
            CoreError::InvalidMuhurtaInput("place")
        );
        let with = |criteria| filter(1.0, criteria);
        assert_eq!(
            bad(
                with(MuhurtaCriteria {
                    tithis: vec![0],
                    ..ok.clone()
                }),
                &mut e
            ),
            CoreError::InvalidMuhurtaInput("tithis")
        );
        assert_eq!(
            bad(
                with(MuhurtaCriteria {
                    nakshatras: vec![28],
                    ..ok.clone()
                }),
                &mut e
            ),
            CoreError::InvalidMuhurtaInput("nakshatras")
        );
        assert_eq!(
            bad(
                with(MuhurtaCriteria {
                    min_minutes: MUHURTA_MAX_MIN_MINUTES + 1,
                    ..ok.clone()
                }),
                &mut e
            ),
            CoreError::InvalidMuhurtaInput("min_minutes")
        );
        let f = filter(100.0, ok);
        for cursor in [
            MuhurtaCursor {
                chunk: 4,
                open_start_jd_ut: None,
            },
            // Açık pencere bu dilimden sonra başlayamaz.
            MuhurtaCursor {
                chunk: 1,
                open_start_jd_ut: Some(JD + 40.0),
            },
            MuhurtaCursor {
                chunk: 1,
                open_start_jd_ut: Some(JD - 1.0),
            },
            MuhurtaCursor {
                chunk: 0,
                open_start_jd_ut: Some(JD),
            },
            MuhurtaCursor {
                chunk: 1,
                open_start_jd_ut: Some(f64::NAN),
            },
        ] {
            assert_eq!(
                muhurta_search_page(&mut e, &s, &f, Some(cursor)),
                Err(CoreError::InvalidMuhurtaInput("cursor")),
                "{cursor:?}"
            );
        }
    }

    #[test]
    fn criteria_normalize_to_a_canonical_form() {
        let c = MuhurtaCriteria {
            weekdays: vec![Vara::Friday, Vara::Sunday, Vara::Friday],
            tithis: vec![9, 2, 9],
            nakshatras: vec![27, 1],
            lagna_signs: vec![Sign::Leo, Sign::Aries, Sign::Leo],
            exclude: vec![MuhurtaFlag::Gulika, MuhurtaFlag::Rikta, MuhurtaFlag::Gulika],
            ..MuhurtaCriteria::default()
        }
        .normalized();
        assert_eq!(c.weekdays, [Vara::Sunday, Vara::Friday]);
        assert_eq!(c.tithis, [2, 9]);
        assert_eq!(c.nakshatras, [1, 27]);
        assert_eq!(c.lagna_signs, [Sign::Aries, Sign::Leo]);
        assert_eq!(c.exclude, [MuhurtaFlag::Rikta, MuhurtaFlag::Gulika]);
        assert_eq!(c.clone().normalized(), c);
    }
}
````


- [ ] **Step 5: Modülün kökü ve dışa aktarımlar**

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index d61d8d8..e7498b6 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -73,8 +73,12 @@ pub use kuta::{
 };
 pub use lordship::{house_lord, house_sign, houses_owned};
 pub use muhurta::{
-    chandra_bala, day_segments, nakshatra_nature, tara_bala, tithi_group, ChandraBala, DaySegments,
-    MuhurtaFlag, NakshatraNature, NatalMoon, TaraBala, TithiGroup, MUHURTA_METHOD,
+    chandra_bala, day_panchang, day_segments, muhurta_chunks, muhurta_search_page,
+    nakshatra_nature, tara_bala, tithi_group, window_snapshot, ChandraBala, DayKarana,
+    DayNakshatra, DayPanchang, DaySegments, DayTithi, MuhurtaCriteria, MuhurtaCursor,
+    MuhurtaFilter, MuhurtaFlag, MuhurtaSearchPage, MuhurtaWindow, NakshatraNature, NatalMoon,
+    PersonalFactors, TaraBala, TithiGroup, WindowSnapshot, MUHURTA_CHUNK_DAYS,
+    MUHURTA_MAX_RANGE_DAYS, MUHURTA_METHOD,
 };
 pub use nature::{kuja_dosha, moon_is_waxing, nature, KujaDosha, Nature, KUJA_DOSHA_HOUSES};
 pub use panchang::{
diff --git a/crates/jyotish-core/src/muhurta/mod.rs b/crates/jyotish-core/src/muhurta/mod.rs
index f3927da..1da186e 100644
--- a/crates/jyotish-core/src/muhurta/mod.rs
+++ b/crates/jyotish-core/src/muhurta/mod.rs
@@ -1,8 +1,11 @@
-//! Muhurta (ileri modüller spec'i §10, Alt proje 3.7).
+//! Muhurta (ileri modüller spec'i §10, Alt proje 3.7): gün panchangı ve pencere araması.
 //!
 //! - `segments`: günün dilimleri ve sınıflar; saf fonksiyonlar, ephemeris görmez (K3).
+//! - `day`: seçilen anı içeren günün panchangı (`day_panchang`).
+//! - `search`: süzgece uyan zaman pencereleri, dilim dilim (`muhurta_search_page`).
 //!
 //! Hepsi olgudur: "uygun" ya da "uygunsuz" hükmü, etkinlik şablonu ve açıklama metni yoktur.
+//! Aralıklar `timeline` çekirdeğinden gelir. Sonuçlar saklanmaz (K1).
 
 pub mod day;
 pub mod search;
@@ -13,6 +16,14 @@ use serde::{Deserialize, Serialize};
 use crate::zodiac::Sign;
 use crate::CoreError;
 
+pub use day::{
+    day_panchang, DayKarana, DayNakshatra, DayPanchang, DayTithi, PersonalFactors, DAY_FLAG_KINDS,
+};
+pub use search::{
+    muhurta_chunks, muhurta_search_page, window_snapshot, MuhurtaCriteria, MuhurtaCursor,
+    MuhurtaFilter, MuhurtaSearchPage, MuhurtaWindow, WindowSnapshot, MUHURTA_CHUNK_DAYS,
+    MUHURTA_MAX_MIN_MINUTES, MUHURTA_MAX_RANGE_DAYS, MUHURTA_MIN_WINDOW_DAYS,
+};
 pub use segments::{
     chandra_bala, day_segments, equal_parts, gulika_segment, is_panchaka, is_rikta, is_vishti,
     nakshatra_nature, rahu_segment, tara_bala, tithi_group, yamaganda_segment, ChandraBala,
````


- [ ] **Step 6: Testleri koş**

```bash
cargo fmt --all --check
cargo clippy -p jyotish-core --all-targets -- -D warnings 2>&1 | tail -1
cargo test -p jyotish-core --lib muhurta 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
ls crates/jyotish-core/proptest-regressions 2>/dev/null
```
Expected: fmt çıktısız; clippy `Finished`; `30 passed; 0 failed; 0 ignored; 0 measured; 321 filtered out`; sonra `351 passed` (lib), `8 passed` (golden), `15 passed` (`modules_golden`); `ls` çıktısız (proptest düşen bir durum kaydetmedi). Bir property düşerse proptest küçültülmüş girdiyi `crates/jyotish-core/proptest-regressions/` altına yazar: girdi ilerleme notlarına kopyalanır, neden bulunur ve düzeltilir; dosya commit'lenmez. Düşen bir örneklemede önce `holds_at` ile `allowed_spans`'ın aynı kuralı okuyup okumadığına bakılır; test gevşetilmez.

- [ ] **Step 7: Commit**

```bash
git add crates/jyotish-core/src
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): day panchang and the muhurta window search in chunks; properties" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: iki `grep` çıktısız.

---


### Task 4: Atlas ve store: bir andaki ofset, bulunulan yer, kayıtlı süzgeçler

Spec §10 "Bulunulan yer", "Veri modeli ve IPC" (son cümle: `muhurta_filters`), "Gizlilik"; §14; D21, D22, D24, D25. `SQLite` şeması ve şema sürümleri değişmez (C3): `AppPrefs` bir alan kazanır ve eski kayıt yersiz okunur; süzgeçler `app_state`'in yeni bir anahtarıdır. `AppPrefs` bir IPC tipidir: bağlamalar ve fikstür bu görevde yenilenir.

**Files:**
- Create: `crates/store/src/muhurta.rs`
- Modify: `crates/atlas/src/moment.rs`, `crates/atlas/src/lib.rs`, `crates/atlas/tests/moment_iana.rs`, `crates/atlas/tests/moment_fixed_lmt.rs`, `crates/store/src/model.rs`, `crates/store/src/settings.rs`, `crates/store/src/exchange.rs`, `crates/store/src/lib.rs`; üretilen: `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`
- Test: `atlas` (2 test), `store` `settings.rs` (2 test), `muhurta.rs` (4 test), `exchange.rs` (1 test)

**Interfaces:**
- Consumes (v1, Plan 08, Task 3): `atlas::{TzSpec, MomentError, Timestamp, is_known_time_zone}`, `store::{Place, TzChoice, Store, StoreError}`, `Store::{app_state_get, app_state_set}`, `charts::MAX_NAME_CHARS`, `jyotish_core::MuhurtaCriteria` (`validate`, `normalized`).
- Produces:
  - `atlas::offset_at(tz: &TzSpec, lon_deg: f64, utc: Timestamp) -> Result<i32, MomentError>`.
  - `store::HomePlace { place: Place, tz: TzChoice }`; `AppPrefs.home_place: Option<HomePlace>`; `StoreError::InvalidInput("home_place")`.
  - `store::SavedMuhurtaFilter { name, criteria }`, `MAX_MUHURTA_FILTERS`, `MUHURTA_FILTERS_SCHEMA_VERSION`; `Store::muhurta_filters() -> Result<Vec<SavedMuhurtaFilter>, StoreError>`, `Store::save_muhurta_filters(&[SavedMuhurtaFilter]) -> Result<Vec<SavedMuhurtaFilter>, StoreError>`; hata alanları `muhurta_filters`, `muhurta_filters.name`, `muhurta_filters.criteria`.

- [ ] **Step 1: Testler (mevcut dosyalarda)**

Atlas: bir andaki ofset, `resolve_moment`'in çözdüğü ofsetle aynıdır (mevcut tablonun her satırında); yaz saati dönüşünde an belirsiz değildir. Store: bulunulan yer gidip gelir, eski ve bozuk kayıt yersiz okunur, geçersiz yer yalnızca alan adıyla reddedilir; bulunulan yer ve kayıtlı süzgeçler dışa aktarımın hiçbir yerinde geçmez.

````diff
diff --git a/crates/atlas/tests/moment_fixed_lmt.rs b/crates/atlas/tests/moment_fixed_lmt.rs
index 0700f02..a57478c 100644
--- a/crates/atlas/tests/moment_fixed_lmt.rs
+++ b/crates/atlas/tests/moment_fixed_lmt.rs
@@ -159,3 +159,29 @@ fn tzdb_version_is_exposed() {
     assert_eq!(v.len(), 5, "{v}");
     assert!(v.starts_with("20"), "{v}");
 }
+
+#[test]
+fn offset_at_for_fixed_offsets_and_local_mean_time() {
+    let utc: atlas::Timestamp = "2026-11-26T12:00:00Z".parse().unwrap();
+    assert_eq!(
+        atlas::offset_at(&TzSpec::Fixed(19_800), 77.2, utc),
+        Ok(19_800)
+    );
+    assert_eq!(
+        atlas::offset_at(&TzSpec::Fixed(-3_600), 0.0, utc),
+        Ok(-3_600)
+    );
+    // 77,209° × 240 = 18 530,16 sn.
+    assert_eq!(atlas::offset_at(&TzSpec::Lmt, 77.209, utc), Ok(18_530));
+    assert_eq!(atlas::offset_at(&TzSpec::Lmt, -58.3816, utc), Ok(-14_012));
+    assert_eq!(
+        atlas::offset_at(&TzSpec::Fixed(18 * 3600 + 1), 0.0, utc),
+        Err(MomentError::OffsetOutOfRange(18 * 3600 + 1))
+    );
+    for lon in [f64::NAN, 180.5, -181.0] {
+        assert!(matches!(
+            atlas::offset_at(&TzSpec::Lmt, lon, utc),
+            Err(MomentError::LongitudeOutOfRange(_))
+        ));
+    }
+}
diff --git a/crates/atlas/tests/moment_iana.rs b/crates/atlas/tests/moment_iana.rs
index c987390..0eb5e2a 100644
--- a/crates/atlas/tests/moment_iana.rs
+++ b/crates/atlas/tests/moment_iana.rs
@@ -1,5 +1,6 @@
 use atlas::{
-    is_known_time_zone, resolve_moment, Fold, MomentError, MomentWarning, Timestamp, TzSpec,
+    is_known_time_zone, offset_at, resolve_moment, Fold, MomentError, MomentWarning, Timestamp,
+    TzSpec,
 };
 use jiff::civil::{date, time};
 
@@ -395,3 +396,32 @@ fn unknown_time_with_iana_zone_keeps_both_warnings() {
     );
     assert_eq!(m.utc, ts("1960-01-01T10:00:00Z"));
 }
+
+#[test]
+fn offset_at_an_instant_follows_the_zone_rules_and_agrees_with_resolve_moment() {
+    // Tablodaki her an: çözülen UTC anındaki ofset, çözümün ofsetidir.
+    for &(zone, y, mo, d, h, mi, offset, _, utc) in TABLE {
+        let got = offset_at(&iana(zone), 0.0, ts(utc)).unwrap();
+        assert_eq!(got, offset, "{zone} {y}-{mo}-{d} {h}:{mi}");
+    }
+    // Yaz saati geçişi: Berlin 2014-10-26 01:00 UTC'de +02:00'den +01:00'e döner; bir an hiçbir
+    // zaman belirsiz değildir.
+    let berlin = iana("Europe/Berlin");
+    assert_eq!(
+        offset_at(&berlin, 13.4, ts("2014-10-26T00:59:59Z")).unwrap(),
+        7200
+    );
+    assert_eq!(
+        offset_at(&berlin, 13.4, ts("2014-10-26T01:00:00Z")).unwrap(),
+        3600
+    );
+    // tzdb'nin LMT kaydı yerin boylamıyla değiştirilir (1° = 240 sn).
+    assert_eq!(
+        offset_at(&iana("Europe/Istanbul"), 29.0, ts("1850-01-01T00:00:00Z")).unwrap(),
+        6960
+    );
+    assert_eq!(
+        offset_at(&iana("Mars/Olympus"), 0.0, ts("2026-01-01T00:00:00Z")),
+        Err(MomentError::UnknownTimeZone("Mars/Olympus".into()))
+    );
+}
diff --git a/crates/store/src/exchange.rs b/crates/store/src/exchange.rs
index 15c9a49..0762aaf 100644
--- a/crates/store/src/exchange.rs
+++ b/crates/store/src/exchange.rs
@@ -870,4 +870,59 @@ mod tests {
             assert_eq!(link_of(&dst, &own), (None, CompatRoles::OpenIsGroom));
         }
     }
+
+    #[test]
+    fn the_home_place_and_saved_muhurta_filters_are_never_exported() {
+        use crate::model::{HomePlace, Place, TzChoice};
+        use crate::settings::AppPrefs;
+        use crate::SavedMuhurtaFilter;
+
+        let (mut s, _) = populated();
+        s.set_app_prefs(&AppPrefs {
+            home_place: Some(HomePlace {
+                place: Place {
+                    name: "Bulunulan Kasaba".into(),
+                    country: "Ülke".into(),
+                    admin1: "Bölge".into(),
+                    lat: 12.345_678,
+                    lon: 98.765_432,
+                    geoname_id: Some(424_242),
+                },
+                tz: TzChoice::Iana("Asia/Tokyo".into()),
+            }),
+            ..AppPrefs::default()
+        })
+        .unwrap();
+        s.save_muhurta_filters(&[SavedMuhurtaFilter {
+            name: "Saklı süzgeç".into(),
+            criteria: jyotish_core::MuhurtaCriteria::default(),
+        }])
+        .unwrap();
+        let json = s
+            .export_json(&ExportOptions {
+                include_notes: true,
+                client_ids: None,
+            })
+            .unwrap();
+        let csv = s.export_csv().unwrap();
+        for text in [&json, &csv] {
+            for secret in [
+                "Bulunulan Kasaba",
+                "12.345678",
+                "98.765432",
+                "424242",
+                "Asia/Tokyo",
+                "home_place",
+                "Saklı süzgeç",
+                "muhurta_filters",
+            ] {
+                assert!(!text.contains(secret), "{secret} in the export");
+            }
+        }
+        // İçe aktarma tercihlere dokunmaz.
+        let mut other = store();
+        other.import_json(&json).unwrap();
+        assert_eq!(other.app_prefs().unwrap().home_place, None);
+        assert_eq!(other.muhurta_filters().unwrap(), []);
+    }
 }
diff --git a/crates/store/src/settings.rs b/crates/store/src/settings.rs
index 512ee5f..23511ea 100644
--- a/crates/store/src/settings.rs
+++ b/crates/store/src/settings.rs
@@ -476,6 +476,7 @@ mod tests {
             schema_version: 1,
             locale: "fr".into(),
             calc_settings: raman(),
+            home_place: None,
         };
         s.set_app_prefs(&p).unwrap();
         assert_eq!(s.app_prefs().unwrap(), p);
@@ -490,6 +491,104 @@ mod tests {
         assert_eq!(s.app_prefs().unwrap(), AppPrefs::default());
     }
 
+    fn istanbul() -> HomePlace {
+        HomePlace {
+            place: crate::model::Place {
+                name: "İstanbul".into(),
+                country: "Türkiye".into(),
+                admin1: "İstanbul".into(),
+                lat: 41.0138,
+                lon: 28.9497,
+                geoname_id: Some(745_044),
+            },
+            tz: TzChoice::Iana("Europe/Istanbul".into()),
+        }
+    }
+
+    #[test]
+    fn home_place_round_trips_and_old_or_broken_records_read_as_unset() {
+        let s = store();
+        let with_home = AppPrefs {
+            home_place: Some(istanbul()),
+            ..AppPrefs::default()
+        };
+        s.set_app_prefs(&with_home).unwrap();
+        assert_eq!(s.app_prefs().unwrap(), with_home);
+        // Sabit ofset ve yerel ortalama saat de saklanır; ad boş olabilir (elle girilen yer).
+        for tz in [TzChoice::Fixed(19_800), TzChoice::Lmt] {
+            let mut h = istanbul();
+            h.tz = tz;
+            h.place.name = String::new();
+            h.place.geoname_id = None;
+            let p = AppPrefs {
+                home_place: Some(h),
+                ..AppPrefs::default()
+            };
+            s.set_app_prefs(&p).unwrap();
+            assert_eq!(s.app_prefs().unwrap(), p);
+        }
+        // Yer kaldırılır.
+        s.set_app_prefs(&AppPrefs::default()).unwrap();
+        assert_eq!(s.app_prefs().unwrap().home_place, None);
+        // Alanı taşımayan eski kayıt (Plan 03–19) yersiz okunur; uyarı yoktur.
+        s.app_state_set(
+            APP_PREFS_KEY,
+            r#"{"schema_version":1,"locale":"en","calc_settings":{"schema_version":9}}"#,
+        )
+        .unwrap();
+        assert_eq!(s.app_prefs().unwrap().home_place, None);
+        // Bozuk ya da geçersiz yer "seçilmemiş" sayılır; diğer alanlar okunur.
+        let stored = |home: serde_json::Value| {
+            let mut v = serde_json::to_value(with_home.clone()).unwrap();
+            v["locale"] = json!("fr");
+            v["home_place"] = home;
+            s.app_state_set(APP_PREFS_KEY, &v.to_string()).unwrap();
+            s.app_prefs_load().unwrap()
+        };
+        let mut far = serde_json::to_value(istanbul()).unwrap();
+        far["place"]["lat"] = json!(123.0);
+        let mut zone = serde_json::to_value(istanbul()).unwrap();
+        zone["tz"] = json!({"Iana": "Mars/Olympus"});
+        for broken in [json!("İstanbul"), json!({"place": 1}), far, zone] {
+            let load = stored(broken);
+            assert_eq!(load.prefs.home_place, None);
+            assert_eq!(load.prefs.locale, "fr");
+            assert!(load.warnings.is_empty());
+        }
+        assert_eq!(
+            stored(serde_json::to_value(istanbul()).unwrap())
+                .prefs
+                .home_place,
+            Some(istanbul())
+        );
+    }
+
+    #[test]
+    fn an_invalid_home_place_is_refused_by_field_name_only() {
+        let s = store();
+        let refused = |edit: &dyn Fn(&mut HomePlace)| {
+            let mut h = istanbul();
+            edit(&mut h);
+            let e = s
+                .set_app_prefs(&AppPrefs {
+                    home_place: Some(h),
+                    ..AppPrefs::default()
+                })
+                .unwrap_err();
+            assert!(matches!(e, StoreError::InvalidInput("home_place")), "{e}");
+            // Hata metni yerin adını ya da koordinatını taşımaz.
+            let text = e.to_string();
+            assert!(!text.contains("stanbul") && !text.contains("41"), "{text}");
+        };
+        refused(&|h| h.place.lat = 90.5);
+        refused(&|h| h.place.lon = -180.5);
+        refused(&|h| h.place.lat = f64::NAN);
+        refused(&|h| h.place.name = "x".repeat(MAX_NAME_CHARS + 1));
+        refused(&|h| h.tz = TzChoice::Iana("Mars/Olympus".into()));
+        refused(&|h| h.tz = TzChoice::Fixed(18 * 3600 + 1));
+        assert_eq!(s.app_prefs().unwrap(), AppPrefs::default());
+    }
+
     #[test]
     fn set_app_prefs_validates() {
         let s = store();
````


- [ ] **Step 2: Testlerin derlenmediğini doğrula**

```bash
cargo test -p atlas 2>&1 | grep -E "^error\[" | sort | uniq -c
cargo test -p store --lib 2>&1 | grep -E "^error\[" | sort | uniq -c
```
Expected: atlas: `E0425` (`cannot find function `offset_at` in crate `atlas``) ve `E0432` (`unresolved import `atlas::offset_at``); store: `E0560` (`struct `settings::AppPrefs` has no field named `home_place``), `E0432` (`crate::model::HomePlace`, `crate::SavedMuhurtaFilter`) ve bunlardan doğan tip hataları.

- [ ] **Step 3: Atlas: bir andaki ofset**

````diff
diff --git a/crates/atlas/src/lib.rs b/crates/atlas/src/lib.rs
index 56c898b..c5ca314 100644
--- a/crates/atlas/src/lib.rs
+++ b/crates/atlas/src/lib.rs
@@ -9,6 +9,6 @@ pub use db::{Atlas, AtlasError, Place};
 pub use jiff::civil::{Date, DateTime, Time};
 pub use jiff::Timestamp;
 pub use moment::{
-    is_known_time_zone, resolve_moment, tzdb_version, Fold, MomentError, MomentWarning,
+    is_known_time_zone, offset_at, resolve_moment, tzdb_version, Fold, MomentError, MomentWarning,
     ResolvedMoment, TzSpec,
 };
diff --git a/crates/atlas/src/moment.rs b/crates/atlas/src/moment.rs
index e5a074b..dc6a1b4 100644
--- a/crates/atlas/src/moment.rs
+++ b/crates/atlas/src/moment.rs
@@ -201,6 +201,35 @@ fn lmt(
     Ok(moment(utc, offset_secs, "LMT".to_owned(), warnings))
 }
 
+/// `utc` anında `tz`'nin UTC ofseti, saniye (doğuda pozitif); gösterim içindir (bir anı o yerin
+/// duvar saatiyle yazmak). `resolve_moment` ile aynı kurallar: sabit ofset aralık denetiminden
+/// geçer; `Lmt` boylamdan hesaplanır; IANA bölgesinde tzdb'nin "LMT" kaydı yerin boylamıyla
+/// değiştirilir. Bir an hiçbir zaman belirsiz değildir: boşluk ve örtüşme burada oluşmaz.
+pub fn offset_at(tz: &TzSpec, lon_deg: f64, utc: Timestamp) -> Result<i32, MomentError> {
+    if !lon_deg.is_finite() || !(-180.0..=180.0).contains(&lon_deg) {
+        return Err(MomentError::LongitudeOutOfRange(lon_deg));
+    }
+    let lmt_secs = (lon_deg * 240.0).round() as i32;
+    match tz {
+        TzSpec::Fixed(secs) if secs.abs() > MAX_FIXED_OFFSET_SECS => {
+            Err(MomentError::OffsetOutOfRange(*secs))
+        }
+        TzSpec::Fixed(secs) => Ok(*secs),
+        TzSpec::Lmt => Ok(lmt_secs),
+        TzSpec::Iana(id) => {
+            let zone = tzdb()
+                .get(id)
+                .map_err(|_| MomentError::UnknownTimeZone(id.to_owned()))?;
+            let info = zone.to_offset_info(utc);
+            if info.abbreviation() == "LMT" && !info.dst().is_dst() {
+                Ok(lmt_secs)
+            } else {
+                Ok(info.offset().seconds())
+            }
+        }
+    }
+}
+
 fn moment(
     utc: Timestamp,
     offset_secs: i32,
````


- [ ] **Step 4: Store: bulunulan yer ve kayıtlı süzgeçler**

`home_place` şema sürümünü değiştirmez; okuma toleranslıdır, yazım denetler (D21). Süzgeç kaydı yalnızca ad ve koşuldur (D22); `muhurta.rs`'in testleri saklanan JSON'un anahtarlarını sayar.

````diff
diff --git a/crates/store/src/lib.rs b/crates/store/src/lib.rs
index d532d4c..868c7ca 100644
--- a/crates/store/src/lib.rs
+++ b/crates/store/src/lib.rs
@@ -12,6 +12,7 @@ pub mod exchange;
 pub mod jhd;
 mod layout;
 pub mod model;
+mod muhurta;
 mod notes;
 mod open;
 pub mod schema;
@@ -31,9 +32,10 @@ pub use exchange::{
 };
 pub use jhd::{parse_jhd, JhdChart, JhdImport};
 pub use model::{
-    BirthData, ChartKind, ChartRecord, Client, FoldChoice, Gender, MomentWarning, NewChart,
-    NewClient, Note, Place, ResolvedRecord, TimeCertainty, TzChoice,
+    BirthData, ChartKind, ChartRecord, Client, FoldChoice, Gender, HomePlace, MomentWarning,
+    NewChart, NewClient, Note, Place, ResolvedRecord, TimeCertainty, TzChoice,
 };
+pub use muhurta::{SavedMuhurtaFilter, MAX_MUHURTA_FILTERS, MUHURTA_FILTERS_SCHEMA_VERSION};
 pub use notes::{normalize_tags, TagCount, MAX_NOTE_BYTES, MAX_TAG_CHARS, RECENT_NOTES};
 pub use open::{OpenRecord, StoreWarning};
 pub use schema::SCHEMA_VERSION;
diff --git a/crates/store/src/model.rs b/crates/store/src/model.rs
index a426557..8e6cfd3 100644
--- a/crates/store/src/model.rs
+++ b/crates/store/src/model.rs
@@ -66,6 +66,16 @@ pub struct Place {
     pub geoname_id: Option<u32>,
 }
 
+/// Bulunulan yer (ileri modüller spec'i §10): gün panchangının ve prashnanın varsayılan yeri.
+/// Kişisel veridir: loglanmaz, hata parametresine girmez, dışa aktarıma yazılmaz. Saat dilimi
+/// yerin duvar saatini göstermek içindir (doğum verisindeki `tz` ile aynı seçim).
+#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
+#[cfg_attr(feature = "specta", derive(specta::Type))]
+pub struct HomePlace {
+    pub place: Place,
+    pub tz: TzChoice,
+}
+
 #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
 #[cfg_attr(feature = "specta", derive(specta::Type))]
 pub struct BirthData {
diff --git a/crates/store/src/settings.rs b/crates/store/src/settings.rs
index 23511ea..8436dc7 100644
--- a/crates/store/src/settings.rs
+++ b/crates/store/src/settings.rs
@@ -7,6 +7,8 @@ use serde::de::DeserializeOwned;
 use serde::{Deserialize, Serialize};
 use serde_json::Value;
 
+use crate::charts::MAX_NAME_CHARS;
+use crate::model::{HomePlace, TzChoice};
 use crate::{Store, StoreError};
 
 /// Bir şema adımı: `v` sürümlü JSON'u `v + 1`'e çevirir. `schema_version` alanını
@@ -175,6 +177,29 @@ pub struct AppPrefs {
     pub locale: String,
     /// Yeni haritalara tam kopyası yazılan genel varsayılan (spec §4 "Ayar katmanları").
     pub calc_settings: CalcSettings,
+    /// Bulunulan yer (ileri modüller spec'i §10); seçilmediyse `None`. Alan şema sürümünü
+    /// değiştirmez: bu alanı taşımayan eski kayıt yersiz okunur.
+    pub home_place: Option<HomePlace>,
+}
+
+/// Sabit ofsetin sınırı (atlas ile aynı: ±18 saat).
+const MAX_FIXED_OFFSET_SECS: i32 = 18 * 3600;
+
+/// Bulunulan yerin denetimi: koordinatlar aralıkta, adlar `MAX_NAME_CHARS`'ı aşmıyor, saat
+/// dilimi çözülebilir (bilinen IANA kimliği ya da ±18 saat içinde sabit ofset).
+pub(crate) fn home_place_is_valid(h: &HomePlace) -> bool {
+    let p = &h.place;
+    // NaN ve sonsuz aralık dışındadır.
+    (-90.0..=90.0).contains(&p.lat)
+        && (-180.0..=180.0).contains(&p.lon)
+        && [&p.name, &p.country, &p.admin1]
+            .iter()
+            .all(|t| t.chars().count() <= MAX_NAME_CHARS)
+        && match &h.tz {
+            TzChoice::Iana(id) => atlas::is_known_time_zone(id),
+            TzChoice::Fixed(secs) => secs.abs() <= MAX_FIXED_OFFSET_SECS,
+            TzChoice::Lmt => true,
+        }
 }
 
 impl Default for AppPrefs {
@@ -183,6 +208,7 @@ impl Default for AppPrefs {
             schema_version: APP_PREFS_SCHEMA_VERSION,
             locale: "tr".into(),
             calc_settings: CalcSettings::default(),
+            home_place: None,
         }
     }
 }
@@ -223,6 +249,11 @@ impl Store {
             Some(load) => out.prefs.calc_settings = load.settings().clone(),
             None => {}
         }
+        // Okunamayan ya da geçersiz yer "seçilmemiş" sayılır; açılışı engellemez, uyarı vermez.
+        out.prefs.home_place = obj
+            .remove("home_place")
+            .and_then(|v| serde_json::from_value::<HomePlace>(v).ok())
+            .filter(home_place_is_valid);
         Ok(out)
     }
 
@@ -237,6 +268,14 @@ impl Store {
             return Err(StoreError::InvalidInput("locale"));
         }
         prefs.calc_settings.validate()?;
+        // Hata yalnızca alan adını taşır; yerin kendisi hata parametresine girmez.
+        if prefs
+            .home_place
+            .as_ref()
+            .is_some_and(|h| !home_place_is_valid(h))
+        {
+            return Err(StoreError::InvalidInput("home_place"));
+        }
         if let Some(raw) = self.app_state_get(APP_PREFS_KEY)? {
             if let Ok(Value::Object(obj)) = serde_json::from_str::<Value>(&raw) {
                 let found = stored_prefs_version(&obj);
````

**Yeni dosya:** `crates/store/src/muhurta.rs`

````rust
//! Kayıtlı muhurta süzgeçleri (ileri modüller spec'i §10): `app_state` altında `muhurta_filters`
//! anahtarında JSON. Bir kayıt yalnızca bir ad ve koşullardır (`MuhurtaCriteria`); tarih aralığı,
//! yer ve kişi kaydedilmez. Okuma toleranslıdır: okunamayan kayıt boş liste, okunamayan öğe
//! yok sayılır; eksik koşul alanları varsayılanla dolar.

use jyotish_core::MuhurtaCriteria;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::charts::MAX_NAME_CHARS;
use crate::{Store, StoreError};

pub const MUHURTA_FILTERS_SCHEMA_VERSION: u32 = 1;
/// En çok kayıtlı süzgeç.
pub const MAX_MUHURTA_FILTERS: usize = 32;
const MUHURTA_FILTERS_KEY: &str = "muhurta_filters";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SavedMuhurtaFilter {
    pub name: String,
    pub criteria: MuhurtaCriteria,
}

/// Adların karşılaştırma anahtarı: büyük/küçük harf duyarsız.
fn name_key(name: &str) -> String {
    name.trim().to_lowercase()
}

/// Kanonik biçim: ad kırpılır, koşul kümeleri sıralanır. Geçersizse alan adıyla hata.
fn canonical(filter: &SavedMuhurtaFilter) -> Result<SavedMuhurtaFilter, StoreError> {
    let name = filter.name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
        return Err(StoreError::InvalidInput("muhurta_filters.name"));
    }
    filter
        .criteria
        .validate()
        .map_err(|_| StoreError::InvalidInput("muhurta_filters.criteria"))?;
    Ok(SavedMuhurtaFilter {
        name: name.to_owned(),
        criteria: filter.criteria.clone().normalized(),
    })
}

/// Saklanan bir öğeyi okur: eksik koşul alanları varsayılanla dolar, bilinmeyenler yok sayılır.
fn read_entry(mut entry: Value) -> Option<SavedMuhurtaFilter> {
    let Ok(Value::Object(defaults)) = serde_json::to_value(MuhurtaCriteria::default()) else {
        return None;
    };
    let criteria = entry.get_mut("criteria")?.as_object_mut()?;
    for (key, value) in defaults {
        criteria.entry(key).or_insert(value);
    }
    let filter: SavedMuhurtaFilter = serde_json::from_value(entry).ok()?;
    canonical(&filter).ok()
}

fn stored_version(value: &Value) -> u32 {
    value
        .get("schema_version")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(MUHURTA_FILTERS_SCHEMA_VERSION)
}

impl Store {
    /// Kayıtlı süzgeçler, kaydedildikleri sırayla. Kayıt yoksa, okunamıyorsa ya da daha yeni bir
    /// sürümle yazılmışsa liste boştur.
    pub fn muhurta_filters(&self) -> Result<Vec<SavedMuhurtaFilter>, StoreError> {
        let Some(raw) = self.app_state_get(MUHURTA_FILTERS_KEY)? else {
            return Ok(Vec::new());
        };
        let Ok(value) = serde_json::from_str::<Value>(&raw) else {
            return Ok(Vec::new());
        };
        if stored_version(&value) > MUHURTA_FILTERS_SCHEMA_VERSION {
            return Ok(Vec::new());
        }
        let Some(Value::Array(entries)) = value.get("filters").cloned() else {
            return Ok(Vec::new());
        };
        let mut out: Vec<SavedMuhurtaFilter> = Vec::new();
        for filter in entries.into_iter().filter_map(read_entry) {
            let key = name_key(&filter.name);
            if out.len() < MAX_MUHURTA_FILTERS && out.iter().all(|f| name_key(&f.name) != key) {
                out.push(filter);
            }
        }
        Ok(out)
    }

    /// Listenin tamamını yazar ve saklanan (kanonik) listeyi döner. Adlar boş olamaz ve
    /// büyük/küçük harf ayırmadan tek olmalıdır. Daha yeni sürümlü kaydın üzerine yazmaz.
    pub fn save_muhurta_filters(
        &self,
        filters: &[SavedMuhurtaFilter],
    ) -> Result<Vec<SavedMuhurtaFilter>, StoreError> {
        if filters.len() > MAX_MUHURTA_FILTERS {
            return Err(StoreError::InvalidInput("muhurta_filters"));
        }
        let mut stored: Vec<SavedMuhurtaFilter> = Vec::with_capacity(filters.len());
        for filter in filters {
            let filter = canonical(filter)?;
            let key = name_key(&filter.name);
            if stored.iter().any(|f| name_key(&f.name) == key) {
                return Err(StoreError::InvalidInput("muhurta_filters.name"));
            }
            stored.push(filter);
        }
        if let Some(raw) = self.app_state_get(MUHURTA_FILTERS_KEY)? {
            if let Ok(value) = serde_json::from_str::<Value>(&raw) {
                if stored_version(&value) > MUHURTA_FILTERS_SCHEMA_VERSION {
                    return Err(StoreError::InvalidInput("muhurta_filters"));
                }
            }
        }
        let value = json!({
            "schema_version": MUHURTA_FILTERS_SCHEMA_VERSION,
            "filters": stored,
        });
        self.app_state_set(MUHURTA_FILTERS_KEY, &value.to_string())?;
        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use jyotish_core::{MuhurtaFlag, Sign, Vara};

    use super::*;
    use crate::db::testutil::store;

    fn filter(name: &str) -> SavedMuhurtaFilter {
        SavedMuhurtaFilter {
            name: name.into(),
            criteria: MuhurtaCriteria {
                weekdays: vec![Vara::Thursday, Vara::Monday],
                tithis: vec![11, 2],
                nakshatras: vec![4],
                lagna_signs: vec![Sign::Taurus],
                daytime_only: true,
                min_minutes: 45,
                exclude: vec![MuhurtaFlag::RahuSegment, MuhurtaFlag::Rikta],
                require_tara: true,
                require_chandra: false,
            },
        }
    }

    #[test]
    fn filters_round_trip_in_canonical_form_and_keep_their_order() {
        let s = store();
        assert_eq!(s.muhurta_filters().unwrap(), []);
        let saved = s
            .save_muhurta_filters(&[filter("  Yolculuk "), filter("Açılış")])
            .unwrap();
        assert_eq!(saved.len(), 2);
        assert_eq!(saved[0].name, "Yolculuk");
        assert_eq!(saved[0].criteria.weekdays, [Vara::Monday, Vara::Thursday]);
        assert_eq!(saved[0].criteria.tithis, [2, 11]);
        assert_eq!(
            saved[0].criteria.exclude,
            [MuhurtaFlag::Rikta, MuhurtaFlag::RahuSegment]
        );
        assert_eq!(s.muhurta_filters().unwrap(), saved);
        // Liste bütünüyle değişir; boş liste hepsini siler.
        let one = s.save_muhurta_filters(&[filter("Açılış")]).unwrap();
        assert_eq!(s.muhurta_filters().unwrap(), one);
        assert_eq!(s.save_muhurta_filters(&[]).unwrap(), []);
        assert_eq!(s.muhurta_filters().unwrap(), []);
    }

    #[test]
    fn a_saved_filter_holds_a_name_and_criteria_and_nothing_personal() {
        let s = store();
        s.save_muhurta_filters(&[filter("Yolculuk")]).unwrap();
        let raw = s.app_state_get(MUHURTA_FILTERS_KEY).unwrap().unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["schema_version"], 1);
        let entry = v["filters"][0].as_object().unwrap();
        assert_eq!(entry.keys().collect::<Vec<_>>(), ["criteria", "name"]);
        let mut keys: Vec<&String> = entry["criteria"].as_object().unwrap().keys().collect();
        keys.sort();
        assert_eq!(
            keys,
            [
                "daytime_only",
                "exclude",
                "lagna_signs",
                "min_minutes",
                "nakshatras",
                "require_chandra",
                "require_tara",
                "tithis",
                "weekdays"
            ]
        );
        // Yer, tarih ve kişi alanı yoktur.
        for word in ["lat", "lon", "place", "natal", "jd", "date"] {
            assert!(!raw.contains(&format!("\"{word}")), "{word} in {raw}");
        }
    }

    #[test]
    fn invalid_lists_are_refused_by_field_name() {
        let s = store();
        let bad = |filters: &[SavedMuhurtaFilter], field: &'static str| {
            let e = s.save_muhurta_filters(filters).unwrap_err();
            assert!(
                matches!(e, StoreError::InvalidInput(f) if f == field),
                "{e}"
            );
        };
        bad(&[filter("   ")], "muhurta_filters.name");
        bad(
            &[filter(&"x".repeat(MAX_NAME_CHARS + 1))],
            "muhurta_filters.name",
        );
        // Adlar büyük/küçük harf ayırmadan tek olmalıdır.
        bad(
            &[filter("Yolculuk"), filter("yolculuk ")],
            "muhurta_filters.name",
        );
        let mut tithi = filter("a");
        tithi.criteria.tithis = vec![31];
        bad(&[tithi], "muhurta_filters.criteria");
        let mut nakshatra = filter("a");
        nakshatra.criteria.nakshatras = vec![0];
        bad(&[nakshatra], "muhurta_filters.criteria");
        let many: Vec<_> = (0..=MAX_MUHURTA_FILTERS)
            .map(|i| filter(&format!("f{i}")))
            .collect();
        bad(&many, "muhurta_filters");
        assert_eq!(s.muhurta_filters().unwrap(), []);
        assert_eq!(
            s.save_muhurta_filters(&many[..MAX_MUHURTA_FILTERS])
                .unwrap()
                .len(),
            MAX_MUHURTA_FILTERS
        );
    }

    #[test]
    fn reading_is_tolerant_and_a_newer_record_is_not_overwritten() {
        let s = store();
        for raw in [
            "not json",
            "[1,2]",
            r#"{"filters":7}"#,
            r#"{"schema_version":1}"#,
        ] {
            s.app_state_set(MUHURTA_FILTERS_KEY, raw).unwrap();
            assert_eq!(s.muhurta_filters().unwrap(), [], "{raw}");
        }
        // Eksik koşul alanları varsayılanla dolar; bilinmeyen alanlar yok sayılır; bozuk, adsız
        // ve yinelenen öğeler atılır.
        s.app_state_set(
            MUHURTA_FILTERS_KEY,
            r#"{"schema_version":1,"filters":[
                {"name":"Kısa","criteria":{"tithis":[5,5,2],"future":true}},
                {"name":"kısa","criteria":{}},
                {"name":"","criteria":{}},
                {"name":"Bozuk","criteria":{"tithis":[99]}},
                {"name":"Koşulsuz"},
                7
            ]}"#,
        )
        .unwrap();
        let read = s.muhurta_filters().unwrap();
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].name, "Kısa");
        assert_eq!(
            read[0].criteria,
            MuhurtaCriteria {
                tithis: vec![2, 5],
                ..MuhurtaCriteria::default()
            }
        );
        // Okunamayan eski kaydın üzerine yazılabilir.
        s.app_state_set(MUHURTA_FILTERS_KEY, "[1,2]").unwrap();
        s.save_muhurta_filters(&[filter("Yeni")]).unwrap();
        // Daha yeni sürümün kaydı boş okunur ve üzerine yazılmaz.
        let newer = r#"{"schema_version":2,"filters":[{"name":"Yarın","criteria":{}}]}"#;
        s.app_state_set(MUHURTA_FILTERS_KEY, newer).unwrap();
        assert_eq!(s.muhurta_filters().unwrap(), []);
        assert!(matches!(
            s.save_muhurta_filters(&[filter("Yeni")]),
            Err(StoreError::InvalidInput("muhurta_filters"))
        ));
        assert_eq!(
            s.app_state_get(MUHURTA_FILTERS_KEY).unwrap().as_deref(),
            Some(newer)
        );
    }
}
````


- [ ] **Step 5: Testleri koş**

```bash
cargo fmt --all --check
cargo clippy -p atlas -p store --all-targets -- -D warnings 2>&1 | tail -1
cargo test -p atlas 2>&1 | grep -E "^test result" | grep -v " 0 passed"
cargo test -p store 2>&1 | grep -E "^test result" | head -1
git diff --stat -- crates/store/src/schema.rs crates/store/Cargo.toml crates/atlas/Cargo.toml
```
Expected: fmt çıktısız; clippy `Finished`; atlas `6 passed`, `11 passed`, `13 passed`, `8 passed`, `3 passed`; store `103 passed; 0 failed; 1 ignored`; son komut çıktısız (şema ve bağımlılıklar değişmedi: C3).

- [ ] **Step 6: Üretilen dosyaları yenile ve kapıları koş**

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
git diff --numstat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
grep -c "home_place" app/ui/bindings.ts
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: iki `ok`; `bindings.ts` 15 satır kazanır (`HomePlace` tipi ve `AppPrefs`'in alanı); `delhi.json` tek satır değişir (`prefs.prefs.home_place: null`); `grep -c` `1` yazar; `cargo test --workspace` için sayaç `0` yazar (`ok` olmayan sonuç satırı yok); Vitest `75 passed (75)`, `738 passed (738)`; `✓ built`. (Arayüzde `AppPrefs` her yerde fikstürden ya da okunan değerden yayılarak kurulur: TypeScript değişikliği gerekmez.)

- [ ] **Step 7: Commit**

```bash
git add crates/atlas crates/store/src app/ui/bindings.ts app/ui/test/fixtures/delhi.json
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(atlas,store): offset at an instant; home place in the app prefs; saved muhurta filters" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: iki `grep` çıktısız.

---


### Task 5: Uygulama: `day_panchang` ve `muhurta_search` komutları, süzgeç komutları; bağlamalar, sarmalayıcılar ve fikstür; gerçek ephemeris'le çapraz denetim

Spec §10 "Veri modeli ve IPC" (komutlar), "Test" (çapraz denetim, astronomik), "Performans"; §13 katman 4 ve "komut katmanı"; §15; K1, K2, K8; D8, D16, D20, D23–D26, D36. Komutlar Plan 14'ün kalıbını izler. `Chart` ve `open_client` değişmez.

**Files:**
- Create: `app/src-tauri/src/muhurta_commands.rs`
- Modify: `app/src-tauri/src/lib.rs`, `app/src-tauri/src/store_commands.rs`, `app/src-tauri/src/transit_commands.rs`, `app/src-tauri/tests/modules.rs`, `app/src-tauri/tests/ui_fixture.rs`, `app/ui/api.ts`, `app/ui/api.test.ts`, `app/ui/test/fixture.ts`, `app/e2e/ipc.ts`; üretilen: `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`
- Test: `muhurta_commands.rs` (3 test); `tests/modules.rs` (`timeline_checks`, `muhurta_checks`: mevcut tek testin içinde); `api.test.ts`'in tabloları

**Interfaces:**
- Consumes (Task 1–4, Plan 11): çekirdeğin `day_panchang`, `muhurta_search_page` ve tipleri; `atlas::{offset_at, Timestamp, TzSpec, Date}`; `store::{resolve_birth, BirthData, Place, TzChoice, FoldChoice, TimeCertainty, SavedMuhurtaFilter}`; `transit_commands::{TransitAt, transit_moment}`; `client_commands::{blocking, with_store}`; `Engine::call`.
- Produces:
  - `muhurta_commands::{MuhurtaPlace { lat, lon, tz }, DayPanchangView { day, offset_secs }, MuhurtaQuery { from_date, to_date, place, criteria, natal }, MuhurtaPageCursor { from_jd_ut, to_jd_ut, cursor }, MuhurtaWindowView { start_jd_ut, end_jd_ut, start_offset_secs, end_offset_secs, snapshot }, MuhurtaSearchView { windows, scanned, chunks, from_jd_ut, to_jd_ut, from_offset_secs, to_offset_secs, next, warnings, method }, DEFAULT_SEARCH_DAYS}`.
  - `day_panchang_blocking(engine, at, now, place, settings, natal)`, `muhurta_search_blocking(engine, query, settings, cursor, now)`; Tauri komutları `day_panchang(at, place, settings, natal)`, `muhurta_search(query, settings, cursor)`, `load_muhurta_filters()`, `save_muhurta_filters(filters)`.
  - TS: `api.dayPanchang`, `api.muhurtaSearch`, `api.loadMuhurtaFilters`, `api.saveMuhurtaFilters`; `UiFixture.muhurta`, `UiFixture.muhurta_ids`; e2e taklidinde dört komut ve bellek içi tercihler.

- [ ] **Step 1: Entegrasyon testleri, fikstür üreticisi ve sarmalayıcı tablosu (testler)**

`timeline_checks` çapraz denetimi gerçek ephemeris'le koşar: beş golden anın çevresindeki altı günde, 120'şer örnek anda dizilerin değeri `compute_panchang`'ınkiyle aynıdır ve kırpılmamış her bitiş onun bitiş anıyla 1 saniye içinde örtüşür (Tromsø örneği kutup gecesindedir: vara yavaş yoldan gelir). Ardından astronomik olgular: 800 günde dokuz grahanın burç ve nakshatra girişlerinde boylam sınırın 1″ içindedir; beş grahanın her dönüşünde hız işaret değiştirir; lagna sınırının bir saniye öncesinde önceki burç yükselir. `muhurta_checks` gün panchangını v1 haritasıyla karşılaştırır, hataların yalnızca alan adı taşıdığını ve bütçeleri sınar; aramanın her penceresi noktasal hesapla yeniden doğrulanır.

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index cfe7898..1152a17 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -3,6 +3,7 @@ pub mod client_commands;
 pub mod commands;
 pub mod data_dir;
 pub mod module_commands;
+pub mod muhurta_commands;
 pub mod store_commands;
 pub mod transit_commands;
 pub mod update_commands;
diff --git a/app/src-tauri/tests/modules.rs b/app/src-tauri/tests/modules.rs
index a6dfcd5..22cd167 100644
--- a/app/src-tauri/tests/modules.rs
+++ b/app/src-tauri/tests/modules.rs
@@ -4,17 +4,26 @@
 use std::path::PathBuf;
 use std::time::{Duration, Instant};
 
+use atlas::Timestamp;
 use ephemeris::{Body, Engine, Ephemeris, SwissEph, Zodiac};
 use jyotish_core::kuta::{gana_of, nadi_of, varna_of, yoni_of};
+use jyotish_core::muhurta::{is_panchaka, is_rikta, is_vishti};
+use jyotish_core::timeline::{
+    graha_motion_series, graha_nakshatra_series, graha_sign_series, karana_series, lagna_sign_at,
+    lagna_sign_series, nakshatra_series, nitya_yoga_series, tithi_series, vara_series, Motion,
+    NakshatraPada, STATION_GRAHAS,
+};
 use jyotish_core::{
     chara_dasha, cheshta_from, co_lords, compute_panchang, compute_shadbala_astro,
     graha_aspect_signs, house_lord, house_sign, jaimini_report, kuja_dosha, kuta_report,
     mean_longitudes, rashi_drishti_signs, shadbala, sign_dasha_children, sign_dasha_path_at,
-    sphuta_drishti, sun_entry_before, vara_lord, yoga_rule, yogas, AdvancedSettings, CalcSettings,
-    Chart, ChartInput, ChartWarning, Dignity, Graha, HouseSystem, KarakaRole, KarakaScheme, Kuta,
-    KutaRole, MoonDayChange, Positions, Sign, SignDashaPeriod, Varga, YogaGroup, ASPECTING_GRAHAS,
-    CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD, KUTA_METHOD, MAX_SUNRISE_GAP_DAYS,
-    SEVEN_KARAKA_ROLES, SHADBALA_GRAHAS, SHADBALA_METHOD, YOGA_CATALOG, YOGA_CATALOG_VERSION,
+    sphuta_drishti, sun_entry_before, vara_lord, window_snapshot, yoga_rule, yogas,
+    AdvancedSettings, CalcSettings, Chart, ChartInput, ChartWarning, Dignity, Graha, HouseSystem,
+    KarakaRole, KarakaScheme, Kuta, KutaRole, MoonDayChange, MuhurtaCriteria, MuhurtaFlag,
+    NatalMoon, NodeKind, Positions, Sign, SignDashaPeriod, Vara, Varga, YogaGroup,
+    ASPECTING_GRAHAS, CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD, KUTA_METHOD,
+    MAX_SUNRISE_GAP_DAYS, MUHURTA_METHOD, SEVEN_KARAKA_ROLES, SHADBALA_GRAHAS, SHADBALA_METHOD,
+    YOGA_CATALOG, YOGA_CATALOG_VERSION,
 };
 use jyotish_desk_lib::commands::compute_chart_blocking;
 use jyotish_desk_lib::module_commands::{
@@ -22,7 +31,13 @@ use jyotish_desk_lib::module_commands::{
     compute_compat_blocking, compute_drishti_blocking, compute_jaimini_blocking,
     compute_shadbala_blocking, compute_yogas_blocking,
 };
+use jyotish_desk_lib::muhurta_commands::{
+    day_panchang_blocking, muhurta_search_blocking, MuhurtaPlace, MuhurtaQuery, MuhurtaSearchView,
+    MuhurtaWindowView,
+};
+use jyotish_desk_lib::transit_commands::TransitAt;
 use serde_json::Value;
+use store::TzChoice;
 
 fn root() -> PathBuf {
     PathBuf::from(env!("CARGO_MANIFEST_DIR"))
@@ -66,6 +81,8 @@ fn module_commands_run_on_the_engine_thread_and_map_errors() {
     shadbala_checks(&engine, &inputs);
     yogas_checks(&engine, &inputs);
     compat_checks(&engine, &inputs);
+    timeline_checks(&engine, &inputs);
+    muhurta_checks(&engine, &inputs);
 }
 
 fn drishti_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
@@ -1618,3 +1635,660 @@ fn compat_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
         "compute_compat (both times unknown) median {median:?}"
     );
 }
+
+const SECOND: f64 = 1.0 / 86_400.0;
+
+/// Zaman çizelgesi çekirdeği, gerçek ephemeris'le (ileri modüller spec'i §10 "Test"): v1
+/// panchangıyla çapraz denetim ve astronomik olgular.
+fn timeline_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
+    // Çapraz denetim (asıl güvence): beş golden anın çevresindeki altı günde, aralıkların
+    // içinden örneklenen anlarda `compute_panchang` aynı beş uzvu verir; aralığın sonu onun
+    // bitiş anıyla 1 saniye içinde örtüşür. Tromsø örneği kutup gecesindedir: vara dizisi
+    // 06:00 yerel ortalama saat kuralını yürür.
+    for (id, input) in inputs {
+        let (id, input) = (id.clone(), input.clone());
+        let checked = engine
+            .call(move |se: &mut SwissEph| {
+                let s = &input.settings;
+                let (lat, lon) = (input.lat, input.lon);
+                let (from, to) = (input.jd_ut - 3.0, input.jd_ut + 3.0);
+                let tithis = tithi_series(se, s, from, to).expect("tithis");
+                let karanas = karana_series(se, s, from, to).expect("karanas");
+                let nakshatras = nakshatra_series(se, s, from, to).expect("nakshatras");
+                let yogas = nitya_yoga_series(se, s, from, to).expect("yogas");
+                let varas = vara_series(se, s, lat, lon, from, to).expect("varas");
+                for series_ends in [
+                    (tithis[0].start_jd_ut, tithis[tithis.len() - 1].end_jd_ut),
+                    (karanas[0].start_jd_ut, karanas[karanas.len() - 1].end_jd_ut),
+                    (
+                        nakshatras[0].start_jd_ut,
+                        nakshatras[nakshatras.len() - 1].end_jd_ut,
+                    ),
+                    (yogas[0].start_jd_ut, yogas[yogas.len() - 1].end_jd_ut),
+                    (varas[0].start_jd_ut, varas[varas.len() - 1].end_jd_ut),
+                ] {
+                    assert_eq!(series_ends, (from, to), "{id}: series cover the span");
+                }
+                let mut ends = 0;
+                for k in 0..120 {
+                    let jd = from + 0.011 + 0.0499 * f64::from(k);
+                    let p = compute_panchang(se, jd, lat, lon, s).expect("panchang");
+                    let at = |what: &str| format!("{id}: {what} at +{:.4}", jd - from);
+                    let t = tithis.iter().find(|i| i.contains(jd)).expect("tithi");
+                    let ka = karanas.iter().find(|i| i.contains(jd)).expect("karana");
+                    let n = nakshatras
+                        .iter()
+                        .find(|i| i.contains(jd))
+                        .expect("nakshatra");
+                    let y = yogas.iter().find(|i| i.contains(jd)).expect("yoga");
+                    let v = varas.iter().find(|i| i.contains(jd)).expect("vara");
+                    assert_eq!(t.value, p.tithi.no, "{}", at("tithi"));
+                    assert_eq!(ka.value, p.karana.no, "{}", at("karana"));
+                    assert_eq!(
+                        (n.value.nakshatra, n.value.pada),
+                        (p.nakshatra.no, p.nakshatra.pada),
+                        "{}",
+                        at("nakshatra")
+                    );
+                    assert_eq!(y.value, p.yoga.no, "{}", at("yoga"));
+                    assert_eq!(v.value, p.vara, "{}", at("vara"));
+                    let nakshatra_end = nakshatras
+                        .iter()
+                        .rfind(|i| i.value.nakshatra == n.value.nakshatra)
+                        .expect("nakshatra end")
+                        .end_jd_ut;
+                    for (end, want, what) in [
+                        (t.end_jd_ut, p.tithi.ends_jd_ut, "tithi end"),
+                        (ka.end_jd_ut, p.karana.ends_jd_ut, "karana end"),
+                        (y.end_jd_ut, p.yoga.ends_jd_ut, "yoga end"),
+                        (nakshatra_end, p.nakshatra.ends_jd_ut, "nakshatra end"),
+                    ] {
+                        // Aralığın kırpılan ucu gerçek bir sınır değildir.
+                        if end < to {
+                            assert!((end - want).abs() <= SECOND, "{}", at(what));
+                            ends += 1;
+                        }
+                    }
+                }
+                ends
+            })
+            .expect("engine");
+        assert!(checked > 300, "{checked} boundaries compared");
+    }
+
+    // Astronomik olgular: bulunan burç ve nakshatra girişinde boylam sınırın 1″ içindedir;
+    // dönüş anında hız işaret değiştirir; lagna burcu sınırın iki yanında farklıdır.
+    let delhi = inputs
+        .iter()
+        .find(|(id, _)| id == "new-delhi-2000-01-01")
+        .expect("delhi fixture")
+        .1
+        .clone();
+    engine
+        .call(move |se: &mut SwissEph| {
+            let arcsecond = 1.0 / 3600.0;
+            let off_grid = |lon: f64, span: f64| {
+                let r = lon.rem_euclid(span);
+                r.min(span - r)
+            };
+            let (from, to) = (delhi.jd_ut, delhi.jd_ut + 800.0);
+            for node in [NodeKind::Mean, NodeKind::True] {
+                let s = CalcSettings {
+                    node,
+                    ..delhi.settings.clone()
+                };
+                let sidereal = Zodiac::Sidereal(s.ayanamsa.sidereal_mode());
+                let bodies = [
+                    (Graha::Sun, Body::Sun, 0.0),
+                    (Graha::Moon, Body::Moon, 0.0),
+                    (Graha::Mars, Body::Mars, 0.0),
+                    (Graha::Mercury, Body::Mercury, 0.0),
+                    (Graha::Jupiter, Body::Jupiter, 0.0),
+                    (Graha::Venus, Body::Venus, 0.0),
+                    (Graha::Saturn, Body::Saturn, 0.0),
+                    (Graha::Rahu, s.node.body(), 0.0),
+                    (Graha::Ketu, s.node.body(), 180.0),
+                ];
+                for (graha, body, shift) in bodies {
+                    // Gerçek düğüm 800 günde yüzlerce kez yön değiştirir; ortalama düğümle
+                    // denetlemek yeterlidir.
+                    if node == NodeKind::True && !graha.is_node() {
+                        continue;
+                    }
+                    let span_days = if graha == Graha::Moon { 60.0 } else { 800.0 };
+                    let to = from + span_days;
+                    let signs = graha_sign_series(se, &s, graha, from, to).expect("signs");
+                    assert!(signs.len() > 1, "{graha:?}: at least one ingress");
+                    for i in &signs[1..] {
+                        let lon = se.body(i.start_jd_ut, body, sidereal).expect("lon").lon + shift;
+                        assert!(
+                            off_grid(lon, 30.0) < arcsecond,
+                            "{graha:?} sign ingress at {lon}"
+                        );
+                        assert_eq!(Sign::of(lon), i.value, "{graha:?}");
+                    }
+                    let naks = graha_nakshatra_series(se, &s, graha, from, to).expect("naks");
+                    assert!(naks.len() >= signs.len(), "{graha:?}");
+                    for i in &naks[1..] {
+                        let lon = se.body(i.start_jd_ut, body, sidereal).expect("lon").lon + shift;
+                        assert!(
+                            off_grid(lon, 40.0 / 3.0) < arcsecond,
+                            "{graha:?} nakshatra ingress at {lon}"
+                        );
+                    }
+                }
+            }
+            let s = delhi.settings.clone();
+            let sidereal = Zodiac::Sidereal(s.ayanamsa.sidereal_mode());
+            // 800 günde her grahanın en az bir retro dönemi vardır (Mars ~26 ayda bir).
+            for (graha, body) in [
+                (Graha::Mars, Body::Mars),
+                (Graha::Mercury, Body::Mercury),
+                (Graha::Jupiter, Body::Jupiter),
+                (Graha::Venus, Body::Venus),
+                (Graha::Saturn, Body::Saturn),
+            ] {
+                assert!(STATION_GRAHAS.contains(&graha));
+                let motion = graha_motion_series(se, &s, graha, from, to).expect("motion");
+                assert!(motion.len() >= 3, "{graha:?}: {} intervals", motion.len());
+                for w in motion.windows(2) {
+                    assert_ne!(w[0].value, w[1].value);
+                    let station = w[1].start_jd_ut;
+                    let before = se.body(station - 0.02, body, sidereal).expect("speed");
+                    let after = se.body(station + 0.02, body, sidereal).expect("speed");
+                    assert!(
+                        before.lon_speed * after.lon_speed < 0.0,
+                        "{graha:?}: speed changes sign at the station"
+                    );
+                    assert_eq!(after.lon_speed < 0.0, w[1].value == Motion::Retrograde);
+                }
+            }
+            // Merkür yılda üç kez geri gider: 800 günde en az altı retro dönemi.
+            let mercury = graha_motion_series(se, &s, Graha::Mercury, from, to).expect("motion");
+            let retro = mercury
+                .iter()
+                .filter(|i| i.value == Motion::Retrograde)
+                .count();
+            assert!((6..=8).contains(&retro), "{retro} mercury retrogrades");
+            assert_eq!(
+                graha_motion_series(se, &s, Graha::Rahu, from, to)
+                    .unwrap_err()
+                    .to_string(),
+                "invalid muhurta input: graha"
+            );
+
+            // Lagna: 3 günde 36 burç değişimi; sınırın 1 saniye öncesinde önceki burç.
+            let (lat, lon) = (delhi.lat, delhi.lon);
+            let lagnas = lagna_sign_series(se, &s, lat, lon, from, from + 3.0).expect("lagna");
+            assert!((36..=38).contains(&lagnas.len()), "{}", lagnas.len());
+            for w in lagnas.windows(2) {
+                let at = w[1].start_jd_ut;
+                assert_eq!(lagna_sign_at(se, &s, at, lat, lon).unwrap(), w[1].value);
+                assert_eq!(
+                    lagna_sign_at(se, &s, at - SECOND, lat, lon).unwrap(),
+                    w[0].value
+                );
+                assert_eq!(w[1].value.index(), (w[0].value.index() + 1) % 12);
+            }
+        })
+        .expect("engine");
+}
+
+fn fixed_place(input: &ChartInput) -> MuhurtaPlace {
+    MuhurtaPlace {
+        lat: input.lat,
+        lon: input.lon,
+        tz: TzChoice::Fixed(0),
+    }
+}
+
+/// Bir aramanın bütün dilimleri; her dilimin süresiyle.
+fn search_all(
+    engine: &Engine,
+    query: &MuhurtaQuery,
+    settings: &CalcSettings,
+    now: Timestamp,
+) -> (Vec<MuhurtaWindowView>, Vec<Duration>, MuhurtaSearchView) {
+    let mut windows = Vec::new();
+    let mut times = Vec::new();
+    let mut cursor = None;
+    loop {
+        let t = Instant::now();
+        let page = muhurta_search_blocking(engine, query.clone(), settings.clone(), cursor, now)
+            .unwrap_or_else(|e| panic!("search: {} {:?}", e.code, e.params));
+        times.push(t.elapsed());
+        assert_eq!(page.scanned as usize, times.len());
+        assert_eq!(page.method, MUHURTA_METHOD);
+        windows.extend(page.windows.iter().copied());
+        match page.next {
+            Some(next) => cursor = Some(next),
+            None => {
+                assert_eq!(page.scanned, page.chunks);
+                return (windows, times, page);
+            }
+        }
+    }
+}
+
+fn muhurta_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
+    let now: Timestamp = "2026-11-26T12:00:00Z".parse().expect("timestamp");
+
+    // Gün panchangı: golden anı içeren gün. Vara, gün doğumu ve batımı v1 haritasının
+    // panchangıyla aynıdır; doğum Ay'ı o anın Ay'ıysa tara Janma (1), chandra bala 1. evdir.
+    for (id, input) in inputs {
+        let chart = compute_chart_blocking(
+            engine,
+            input.jd_ut,
+            input.lat,
+            input.lon,
+            input.settings.clone(),
+        )
+        .expect("chart");
+        let moon = &chart.grahas[Graha::Moon as usize];
+        let natal = NatalMoon {
+            nakshatra: moon.nakshatra,
+            sign: moon.sign,
+        };
+        let view = day_panchang_blocking(
+            engine,
+            TransitAt::Jd(input.jd_ut),
+            now,
+            fixed_place(input),
+            input.settings.clone(),
+            Some(natal),
+        )
+        .unwrap_or_else(|e| panic!("{id}: {}", e.code));
+        let d = &view.day;
+        assert_eq!(view.offset_secs, 0, "{id}");
+        assert_eq!(d.method, MUHURTA_METHOD, "{id}");
+        assert!(
+            d.day_start_jd_ut <= input.jd_ut && input.jd_ut < d.day_end_jd_ut,
+            "{id}: the day contains the moment"
+        );
+        assert_eq!(d.vara, chart.panchang.vara, "{id}: vara");
+        let polar = chart
+            .meta
+            .warnings
+            .contains(&ChartWarning::SunriseUnavailable);
+        assert_eq!(
+            d.warnings.contains(&ChartWarning::SunriseUnavailable),
+            polar,
+            "{id}"
+        );
+        if polar {
+            // Kutup gecesi: dilimler yok, uzuvlar var; gün 24 saattir.
+            assert!(d.segments.is_none() && d.sunrise_jd_ut.is_none(), "{id}");
+            assert!(
+                (d.day_end_jd_ut - d.day_start_jd_ut - 1.0).abs() < 1e-9,
+                "{id}"
+            );
+        } else {
+            assert_eq!(d.sunrise_jd_ut, chart.panchang.sunrise_jd_ut, "{id}");
+            assert_eq!(d.sunset_jd_ut, chart.panchang.sunset_jd_ut, "{id}");
+            let seg = d.segments.as_ref().expect("segments");
+            assert_eq!(seg.eighths.len(), 8, "{id}");
+            assert_eq!(seg.horas.len(), 24, "{id}");
+            assert_eq!(seg.horas[0].value, vara_lord(d.vara), "{id}");
+            assert_eq!(Some(seg.eighths[7].end_jd_ut), d.sunset_jd_ut, "{id}");
+            assert_eq!(seg.horas[23].end_jd_ut, d.day_end_jd_ut, "{id}");
+        }
+        let at = |jd: f64| move |i: &&jyotish_core::Interval<_>| i.contains(jd);
+        let tithi = d.tithis.iter().find(at(input.jd_ut)).expect("tithi").value;
+        assert_eq!(
+            (tithi.no, tithi.paksha),
+            (chart.panchang.tithi.no, chart.panchang.tithi.paksha),
+            "{id}"
+        );
+        let nakshatra = d.nakshatras.iter().find(|i| i.contains(input.jd_ut));
+        let nakshatra = nakshatra.expect("nakshatra").value;
+        assert_eq!((nakshatra.no, nakshatra.pada), (moon.nakshatra, moon.pada));
+        let karana = d.karanas.iter().find(|i| i.contains(input.jd_ut));
+        assert_eq!(
+            karana.expect("karana").value.name,
+            chart.panchang.karana.name,
+            "{id}"
+        );
+        let yoga = d.yogas.iter().find(|i| i.contains(input.jd_ut));
+        assert_eq!(yoga.expect("yoga").value, chart.panchang.yoga.no, "{id}");
+        let personal = d.personal.as_ref().expect("personal");
+        let tara = personal.tara.iter().find(|i| i.contains(input.jd_ut));
+        assert_eq!(tara.expect("tara").value.number, 1, "{id}: janma tara");
+        let chandra = personal.chandra.iter().find(|i| i.contains(input.jd_ut));
+        assert_eq!(chandra.expect("chandra").value.house, 1, "{id}");
+        // Yanık Jüpiter ya da Venüs işareti, haritanın yanma olgusuyla aynıdır.
+        for (flag, graha) in [
+            (MuhurtaFlag::JupiterCombust, Graha::Jupiter),
+            (MuhurtaFlag::VenusCombust, Graha::Venus),
+        ] {
+            let flagged = d
+                .flags
+                .iter()
+                .any(|f| f.value == flag && f.contains(input.jd_ut));
+            assert_eq!(
+                flagged, chart.grahas[graha as usize].combust,
+                "{id}: {flag:?}"
+            );
+        }
+        // Doğum Ay'ı verilmezse kişiye bağlı etken yoktur.
+        let plain = day_panchang_blocking(
+            engine,
+            TransitAt::Jd(input.jd_ut),
+            now,
+            fixed_place(input),
+            input.settings.clone(),
+            None,
+        )
+        .expect("day");
+        assert_eq!(plain.day.personal, None, "{id}");
+        assert_eq!(plain.day.tithis, d.tithis, "{id}");
+    }
+
+    let delhi = inputs
+        .iter()
+        .find(|(id, _)| id == "new-delhi-2000-01-01")
+        .expect("delhi fixture")
+        .1
+        .clone();
+    let kolkata_time = MuhurtaPlace {
+        lat: delhi.lat,
+        lon: delhi.lon,
+        tz: TzChoice::Iana("Asia/Kolkata".into()),
+    };
+    // "Şimdi" Rust'ta okunur; saatler yerin ofsetiyle gösterilir (+05:30).
+    let today = day_panchang_blocking(
+        engine,
+        TransitAt::Now,
+        now,
+        kolkata_time.clone(),
+        delhi.settings.clone(),
+        None,
+    )
+    .expect("today");
+    assert_eq!(today.offset_secs, 19_800);
+    // 2026-11-26 12:00 UTC = JD 2461371.0; Delhi'de Perşembe, gün doğumundan sonra.
+    assert!(today.day.day_start_jd_ut <= 2_461_371.0 && 2_461_371.0 < today.day.day_end_jd_ut);
+    assert_eq!(today.day.vara, Vara::Thursday);
+
+    // Hatalar alan adıyla döner; yer ve tarih hata parametresine girmez.
+    let bad_place = MuhurtaPlace {
+        lat: 95.0,
+        ..kolkata_time.clone()
+    };
+    let e = day_panchang_blocking(
+        engine,
+        TransitAt::Now,
+        now,
+        bad_place,
+        delhi.settings.clone(),
+        None,
+    )
+    .unwrap_err();
+    assert_eq!(e.code, "invalid_muhurta_input");
+    assert_eq!(e.params.len(), 1);
+    assert_eq!(e.params["field"], "place");
+    let bad_natal = NatalMoon {
+        nakshatra: 0,
+        sign: Sign::Aries,
+    };
+    let e = day_panchang_blocking(
+        engine,
+        TransitAt::Now,
+        now,
+        kolkata_time.clone(),
+        delhi.settings.clone(),
+        Some(bad_natal),
+    )
+    .unwrap_err();
+    assert_eq!(
+        (e.code.as_str(), e.params["field"].as_str()),
+        ("invalid_muhurta_input", "natal")
+    );
+    let old = CalcSettings {
+        schema_version: 1,
+        ..CalcSettings::default()
+    };
+    assert_eq!(
+        day_panchang_blocking(engine, TransitAt::Now, now, kolkata_time.clone(), old, None)
+            .unwrap_err()
+            .code,
+        "unsupported_schema_version"
+    );
+
+    // Performans: spec §15 "day_panchang ≤ 50 ms".
+    let (median, max) = median_of_15(|| {
+        day_panchang_blocking(
+            engine,
+            TransitAt::Jd(delhi.jd_ut),
+            now,
+            kolkata_time.clone(),
+            delhi.settings.clone(),
+            Some(NatalMoon {
+                nakshatra: 15,
+                sign: Sign::Libra,
+            }),
+        )
+        .expect("day");
+    });
+    eprintln!("day_panchang median {median:?}, max {max:?}");
+    assert!(
+        median <= Duration::from_millis(50),
+        "day_panchang median {median:?}"
+    );
+
+    // Arama: 366 gün, lagna süzgeci olmadan. Bütün diğer koşullar etkin (her dizi hesaplanır).
+    let natal = NatalMoon {
+        nakshatra: 15,
+        sign: Sign::Libra,
+    };
+    let criteria = MuhurtaCriteria {
+        weekdays: vec![Vara::Monday, Vara::Wednesday, Vara::Thursday, Vara::Friday],
+        tithis: (1..=30).filter(|t| t % 15 != 0).collect(),
+        nakshatras: (1..=27).filter(|n| n % 3 != 0).collect(),
+        lagna_signs: Vec::new(),
+        daytime_only: true,
+        min_minutes: 30,
+        exclude: MuhurtaFlag::ALL.to_vec(),
+        require_tara: true,
+        require_chandra: true,
+    };
+    let query = MuhurtaQuery {
+        from_date: Some("2026-01-01".into()),
+        to_date: Some("2027-01-01".into()),
+        place: kolkata_time.clone(),
+        criteria: criteria.clone(),
+        natal: Some(natal),
+    };
+    let s = delhi.settings.clone();
+    let started = Instant::now();
+    let (windows, times, last) = search_all(engine, &query, &s, now);
+    let total = started.elapsed();
+    let slowest = times.iter().max().copied().unwrap_or_default();
+    eprintln!(
+        "muhurta_search (366 days) total {total:?}, first page {:?}, slowest page {slowest:?}, {} windows",
+        times[0],
+        windows.len()
+    );
+    assert_eq!(times.len(), 12, "twelve chunks for 366 days");
+    assert_eq!(last.chunks, 12);
+    // 2026-01-01 00:00 +05:30 = 2025-12-31 18:30 UTC.
+    assert!((last.from_jd_ut - 2_461_041.270_833_333).abs() < 1e-6);
+    assert!((last.to_jd_ut - last.from_jd_ut - 366.0).abs() < 1e-9);
+    assert_eq!(
+        (last.from_offset_secs, last.to_offset_secs),
+        (19_800, 19_800)
+    );
+    assert!(total <= Duration::from_secs(2), "366 days {total:?}");
+    assert!(
+        times[0] <= Duration::from_millis(500),
+        "first page {:?}",
+        times[0]
+    );
+    // Motor iş parçacığı dilim aralarında bırakılır: araya giren hesap en çok bir dilim bekler.
+    assert!(
+        slowest <= Duration::from_millis(300),
+        "slowest page {slowest:?}"
+    );
+    assert!(windows.len() > 20, "{} windows", windows.len());
+    let check = |windows: &[MuhurtaWindowView], lagna: &[Sign]| {
+        let windows = windows.to_vec();
+        let (lat, lon) = (delhi.lat, delhi.lon);
+        let (s, c) = (delhi.settings.clone(), criteria.clone());
+        let lagna = lagna.to_vec();
+        engine
+            .call(move |se: &mut SwissEph| {
+                let mut previous_end = f64::MIN;
+                for w in &windows {
+                    assert!(previous_end <= w.start_jd_ut, "ordered and disjoint");
+                    previous_end = w.end_jd_ut;
+                    assert!(
+                        w.end_jd_ut - w.start_jd_ut >= 30.0 / 1440.0,
+                        "minimum length"
+                    );
+                    assert_eq!(w.start_offset_secs, 19_800);
+                    // Başlangıç anının özeti, noktasal hesapla aynıdır.
+                    let at_start = window_snapshot(se, &s, w.start_jd_ut, lat, lon).unwrap();
+                    assert_eq!(w.snapshot, at_start);
+                    for jd in [
+                        w.start_jd_ut + SECOND,
+                        0.5 * (w.start_jd_ut + w.end_jd_ut),
+                        w.end_jd_ut - SECOND,
+                    ] {
+                        let x = window_snapshot(se, &s, jd, lat, lon).unwrap();
+                        let p = compute_panchang(se, jd, lat, lon, &s).unwrap();
+                        assert_eq!((x.vara, x.tithi, x.yoga), (p.vara, p.tithi.no, p.yoga.no));
+                        assert!(c.weekdays.contains(&x.vara), "weekday at {jd}");
+                        assert!(c.tithis.contains(&x.tithi) && !is_rikta(x.tithi));
+                        assert!(c.nakshatras.contains(&x.nakshatra));
+                        assert!(!is_vishti(x.karana) && x.yoga != 17 && x.yoga != 27);
+                        assert!(!is_panchaka(NakshatraPada {
+                            nakshatra: x.nakshatra,
+                            pada: x.pada
+                        }));
+                        let (rise, set) = (p.sunrise_jd_ut.unwrap(), p.sunset_jd_ut.unwrap());
+                        assert!(rise <= jd && jd < set, "daytime at {jd}");
+                        // Rahu dilimi: Pazartesi 2., Çarşamba 5., Perşembe 6., Cuma 4.
+                        let eighth = ((jd - rise) / (set - rise) * 8.0).floor() as u8 + 1;
+                        let rahu = match x.vara {
+                            Vara::Monday => 2,
+                            Vara::Wednesday => 5,
+                            Vara::Thursday => 6,
+                            _ => 4,
+                        };
+                        assert_ne!(eighth, rahu, "rahu segment at {jd}");
+                        if !lagna.is_empty() {
+                            assert!(lagna.contains(&x.lagna_sign), "lagna at {jd}");
+                        }
+                    }
+                }
+            })
+            .expect("engine");
+    };
+    check(&windows, &[]);
+
+    // Lagna süzgeciyle: bütçe ≤ 5 sn; pencereler ilk aramanınkilerin içindedir.
+    let lagna = vec![Sign::Taurus, Sign::Leo, Sign::Scorpio, Sign::Aquarius];
+    let with_lagna = MuhurtaQuery {
+        criteria: MuhurtaCriteria {
+            lagna_signs: lagna.clone(),
+            ..criteria.clone()
+        },
+        ..query.clone()
+    };
+    let started = Instant::now();
+    let (lagna_windows, times, _) = search_all(engine, &with_lagna, &s, now);
+    let total = started.elapsed();
+    let slowest = times.iter().max().copied().unwrap_or_default();
+    eprintln!(
+        "muhurta_search (366 days, lagna) total {total:?}, first page {:?}, slowest page {slowest:?}, {} windows",
+        times[0],
+        lagna_windows.len()
+    );
+    assert!(
+        total <= Duration::from_secs(5),
+        "366 days with lagna {total:?}"
+    );
+    assert!(
+        times[0] <= Duration::from_millis(500),
+        "first page {:?}",
+        times[0]
+    );
+    assert!(
+        slowest <= Duration::from_millis(300),
+        "slowest page {slowest:?}"
+    );
+    assert!(!lagna_windows.is_empty());
+    check(&lagna_windows, &lagna);
+    for w in &lagna_windows {
+        assert!(
+            windows
+                .iter()
+                .any(|outer| outer.start_jd_ut <= w.start_jd_ut && w.end_jd_ut <= outer.end_jd_ut),
+            "a lagna window lies inside a window of the wider search"
+        );
+    }
+    // En pahalı durum: yalnızca lagna koşulu (hiçbir koşul aralığı daraltmaz).
+    let only_lagna = MuhurtaQuery {
+        criteria: MuhurtaCriteria {
+            lagna_signs: vec![Sign::Leo],
+            ..MuhurtaCriteria::default()
+        },
+        natal: None,
+        ..query.clone()
+    };
+    let started = Instant::now();
+    let (leo, times, _) = search_all(engine, &only_lagna, &s, now);
+    let total = started.elapsed();
+    let slowest = times.iter().max().copied().unwrap_or_default();
+    eprintln!(
+        "muhurta_search (366 days, lagna only) total {total:?}, slowest page {slowest:?}, {} windows",
+        leo.len()
+    );
+    // Yıldız günü 23 sa 56 dk: 366 günde lagna Aslan'a 367 kez girer.
+    assert!((366..=368).contains(&leo.len()), "{}", leo.len());
+    assert!(total <= Duration::from_secs(5), "lagna only {total:?}");
+    assert!(
+        slowest <= Duration::from_millis(300),
+        "slowest page {slowest:?}"
+    );
+
+    // Tarihsiz arama bugünden başlar ve 31 gün sürer (tek dilim).
+    let default_range = MuhurtaQuery {
+        from_date: None,
+        to_date: None,
+        ..only_lagna.clone()
+    };
+    let page = muhurta_search_blocking(engine, default_range, s.clone(), None, now).expect("page");
+    assert_eq!((page.scanned, page.chunks, page.next), (1, 1, None));
+    // 2026-11-26 00:00 +05:30.
+    assert!((page.from_jd_ut - 2_461_370.270_833_333).abs() < 1e-6);
+    assert!((page.to_jd_ut - page.from_jd_ut - 31.0).abs() < 1e-9);
+
+    // Hatalar: 366 günü aşan aralık, ters aralık, kişisiz kişi koşulu.
+    let field = |q: MuhurtaQuery| {
+        let e = muhurta_search_blocking(engine, q, s.clone(), None, now).unwrap_err();
+        assert_eq!(e.code, "invalid_muhurta_input");
+        assert_eq!(e.params.len(), 1, "{:?}", e.params);
+        e.params["field"].clone()
+    };
+    assert_eq!(
+        field(MuhurtaQuery {
+            to_date: Some("2027-01-02".into()),
+            ..query.clone()
+        }),
+        "range"
+    );
+    assert_eq!(
+        field(MuhurtaQuery {
+            to_date: Some("2025-12-30".into()),
+            ..query.clone()
+        }),
+        "range"
+    );
+    assert_eq!(
+        field(MuhurtaQuery {
+            natal: None,
+            ..query.clone()
+        }),
+        "natal"
+    );
+}
diff --git a/app/src-tauri/tests/ui_fixture.rs b/app/src-tauri/tests/ui_fixture.rs
index 9988b14..0917910 100644
--- a/app/src-tauri/tests/ui_fixture.rs
+++ b/app/src-tauri/tests/ui_fixture.rs
@@ -11,7 +11,11 @@ use std::sync::Mutex;
 
 use atlas::Timestamp;
 use ephemeris::Engine;
-use jyotish_core::{kuta_class_ids, ChartInput, Kuta, YOGA_CATALOG, YOGA_CONDITIONS};
+use jyotish_core::muhurta::{NakshatraNature, TithiGroup, MUHURTA_MAX_MIN_MINUTES};
+use jyotish_core::{
+    kuta_class_ids, ChartInput, Graha, Kuta, MuhurtaCriteria, MuhurtaFlag, NatalMoon, Vara,
+    YOGA_CATALOG, YOGA_CONDITIONS,
+};
 use jyotish_desk_lib::atlas_commands::{preview_moment_body, search_places_blocking, AtlasState};
 use jyotish_desk_lib::client_commands::create_client_blocking;
 use jyotish_desk_lib::commands::dasha_children;
@@ -20,10 +24,14 @@ use jyotish_desk_lib::module_commands::{
     compute_compat_blocking, compute_drishti_blocking, compute_jaimini_blocking,
     compute_shadbala_blocking, compute_yogas_blocking,
 };
+use jyotish_desk_lib::muhurta_commands::{
+    day_panchang_blocking, muhurta_search_blocking, MuhurtaPlace, MuhurtaQuery,
+};
 use jyotish_desk_lib::transit_commands::{transits_at_blocking, TransitAt};
 use serde_json::{json, Value};
 use store::{
-    BirthData, ChartKind, Gender, NewChart, NewClient, Place, Store, TimeCertainty, TzChoice,
+    BirthData, ChartKind, Gender, HomePlace, NewChart, NewClient, Place, Store, TimeCertainty,
+    TzChoice,
 };
 
 const CLIENT_UUID: &str = "00000000-0000-4000-8000-000000000001";
@@ -221,6 +229,84 @@ fn ui_fixture_is_up_to_date() {
         .into_iter()
         .map(|k| (format!("{k:?}"), json!(kuta_class_ids(k))))
         .collect();
+    // Muhurta: açık haritanın doğum yeri için "şimdi"yi içeren gün (doğum Ay'ıyla), kutup
+    // gecesinde bir gün (dilimler hesaplanmaz) ve üç dilimlik bir arama (67 gün; Perşembe
+    // gündüzleri, üç işaretli dilim dışlanmış). İşaret ve sınıf kimlikleri: arayüz testleri her
+    // birinin üç dilde adı olduğunu bu listeden denetler.
+    let fixture_now = Timestamp::from_second(1_795_600_800).expect("timestamp");
+    let moon = &opened.chart.grahas[Graha::Moon as usize];
+    let natal = NatalMoon {
+        nakshatra: moon.nakshatra,
+        sign: moon.sign,
+    };
+    let birth = &opened.chart_record.birth;
+    let muhurta_place = MuhurtaPlace {
+        lat: birth.place.lat,
+        lon: birth.place.lon,
+        tz: birth.tz.clone(),
+    };
+    let muhurta_day = day_panchang_blocking(
+        &engine,
+        TransitAt::Now,
+        fixture_now,
+        muhurta_place.clone(),
+        opened.calc_settings.clone(),
+        Some(natal),
+    )
+    .expect("day panchang");
+    // tests/golden/tromso-2050-12-21.json anı: Güneş doğmaz.
+    let muhurta_day_polar = day_panchang_blocking(
+        &engine,
+        TransitAt::Jd(2_470_162.260_416_666_5),
+        fixture_now,
+        MuhurtaPlace {
+            lat: 69.6492,
+            lon: 18.9553,
+            tz: TzChoice::Iana("Europe/Oslo".into()),
+        },
+        opened.calc_settings.clone(),
+        None,
+    )
+    .expect("day panchang in the polar night");
+    let muhurta_query = MuhurtaQuery {
+        from_date: Some("2026-11-26".into()),
+        to_date: Some("2027-01-31".into()),
+        place: muhurta_place.clone(),
+        criteria: MuhurtaCriteria {
+            weekdays: vec![Vara::Thursday],
+            daytime_only: true,
+            min_minutes: 30,
+            exclude: vec![
+                MuhurtaFlag::RahuSegment,
+                MuhurtaFlag::Yamaganda,
+                MuhurtaFlag::Gulika,
+            ],
+            ..MuhurtaCriteria::default()
+        },
+        natal: Some(natal),
+    };
+    let mut muhurta_pages = Vec::new();
+    let mut cursor = None;
+    loop {
+        let page = muhurta_search_blocking(
+            &engine,
+            muhurta_query.clone(),
+            opened.calc_settings.clone(),
+            cursor,
+            fixture_now,
+        )
+        .expect("muhurta search");
+        cursor = page.next;
+        muhurta_pages.push(page);
+        if cursor.is_none() {
+            break;
+        }
+    }
+    assert_eq!(muhurta_pages.len(), 3, "sixty-seven days are three chunks");
+    let home_place = HomePlace {
+        place: birth.place.clone(),
+        tz: birth.tz.clone(),
+    };
     // Kaydedilen doğum verisi girdinin kanonik biçimidir (Plan 10 kapsam dışı 21).
     assert_eq!(opened.chart_record.birth, delhi());
     let prefs = store
@@ -256,6 +342,21 @@ fn ui_fixture_is_up_to_date() {
             "report_time_unknown": compat_time_unknown,
         },
         "kuta_classes": kuta_classes,
+        "muhurta": {
+            "natal": natal,
+            "place": muhurta_place,
+            "home_place": home_place,
+            "day": muhurta_day,
+            "day_polar": muhurta_day_polar,
+            "query": muhurta_query,
+            "pages": muhurta_pages,
+        },
+        "muhurta_ids": {
+            "flags": MuhurtaFlag::ALL,
+            "tithi_groups": TithiGroup::ALL,
+            "nakshatra_natures": NakshatraNature::ALL,
+            "max_min_minutes": MUHURTA_MAX_MIN_MINUTES,
+        },
     });
     let text = serde_json::to_string(&value)
         .expect("serialize")
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index 8500a09..7c75efd 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -38,6 +38,10 @@ vi.mock("./bindings", () => ({
     computeCompat: vi.fn(),
     chartInput: vi.fn(),
     chartBrief: vi.fn(),
+    dayPanchang: vi.fn(),
+    muhurtaSearch: vi.fn(),
+    loadMuhurtaFilters: vi.fn(),
+    saveMuhurtaFilters: vi.fn(),
     searchPlaces: vi.fn(),
     previewMoment: vi.fn(),
     getAppPrefs: vi.fn(),
@@ -127,6 +131,10 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["computeCompat", [{ jd_ut: 1, time_known: true }, { jd_ut: 2, time_known: false }]],
     ["chartInput", ["00000000-0000-4000-8000-000000000012"]],
     ["chartBrief", ["00000000-0000-4000-8000-000000000012"]],
+    ["dayPanchang", ["Now", { lat: 28.6, lon: 77.2, tz: "Lmt" }, { ayanamsa: "Lahiri" }, { nakshatra: 15, sign: "Libra" }]],
+    ["muhurtaSearch", [{ from_date: null, to_date: null }, { ayanamsa: "Lahiri" }, { cursor: { chunk: 1 } }]],
+    ["loadMuhurtaFilters", []],
+    ["saveMuhurtaFilters", [[{ name: "Yolculuk", criteria: {} }]]],
     ["searchPlaces", ["delhi", 8]],
     ["previewMoment", [{ date: "2000-01-01" }]],
     ["getAppPrefs", []],
````


- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-desk --lib muhurta 2>&1 | grep -E "^error" | head -2
pnpm --dir app exec vitest run ui/api.test.ts 2>&1 | grep -E "Tests |FAIL|TypeError"
```
Expected: `error[E0583]: file not found for module `muhurta_commands``; Vitest `FAIL  ui/api.test.ts > every wrapper forwards its arguments to the command of the same name`, `TypeError: call is not a function` ve `1 failed | 6 passed (7)`.

- [ ] **Step 3: Muhurta komutları**

Yer motoru meşgul etmeden önce denetlenir; hatalar yalnızca alan adı taşır (D20). Aralık ilk çağrıda çözülür ve imleçte taşınır (D23). Dosyanın testleri motor açmaz.

````diff
diff --git a/app/src-tauri/src/transit_commands.rs b/app/src-tauri/src/transit_commands.rs
index e205f29..5065973 100644
--- a/app/src-tauri/src/transit_commands.rs
+++ b/app/src-tauri/src/transit_commands.rs
@@ -62,7 +62,11 @@ fn resolve_local(date: &str, time: &str, tz: TzChoice) -> Result<(f64, String),
     Ok((r.jd_ut, r.utc))
 }
 
-fn transit_moment(at: &TransitAt, now: Timestamp) -> Result<(f64, String), CommandError> {
+/// Transit anının Jülyen günü ve UTC yazımı. Gün panchangı da aynı anı kullanır.
+pub(crate) fn transit_moment(
+    at: &TransitAt,
+    now: Timestamp,
+) -> Result<(f64, String), CommandError> {
     match at {
         TransitAt::Now => {
             // UTC duvar saati, saniye çözünürlüklü; JD dönüşümü atlas'ta.
````

**Yeni dosya:** `app/src-tauri/src/muhurta_commands.rs`

````rust
//! Muhurta komutları (ileri modüller spec'i §10): gün panchangı ve pencere araması. Ortak kalıp
//! (K1–K3): hesap motor iş parçacığında, komut gövdesi `blocking` havuzunda, hata
//! `CommandError`; sonuç saklanmaz. "Şimdi" ve "bugün" Rust'ta okunur; arayüz tarih hesabı
//! yapmaz (C4). Arama dilim dilim yürür: her çağrı bir dilimi hesaplar ve motor iş parçacığını
//! bırakır; aramayı durdurmak, sonraki dilimi istememektir (durum tutulmaz).
//!
//! Yer (bulunulan yer dahil) kişisel veridir: loglanmaz ve hata parametresine girmez. Bu
//! dosyanın ürettiği hatalar yalnızca alan adı taşır (`invalid_muhurta_input`).

use std::sync::Arc;

use atlas::{Timestamp, TzSpec};
use ephemeris::{Engine, SwissEph};
use jyotish_core::{
    CalcSettings, ChartWarning, DayPanchang, MuhurtaCriteria, MuhurtaCursor, MuhurtaFilter,
    NatalMoon, WindowSnapshot,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use store::{BirthData, FoldChoice, Place, TimeCertainty, TzChoice};
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::CommandError;
use crate::transit_commands::{transit_moment, TransitAt};

/// Tarih verilmeyen aramanın uzunluğu, gün: başlangıç günü ve sonraki 30 gün.
pub const DEFAULT_SEARCH_DAYS: f64 = 31.0;
const UNIX_EPOCH_JD: f64 = 2_440_587.5;

/// Hesabın yapılacağı yer ve saatlerin gösterileceği dilim. Ad taşımaz.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct MuhurtaPlace {
    /// Kuzey +.
    pub lat: f64,
    /// Doğu +.
    pub lon: f64,
    pub tz: TzChoice,
}

/// Gün panchangı ve günün başındaki UTC ofseti (saatler bu ofsetle gösterilir).
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct DayPanchangView {
    pub day: DayPanchang,
    pub offset_secs: i32,
}

/// Aramanın girdisi. `from_date` ve `to_date` yerin takvim günleridir (`YYYY-MM-DD`, ikisi de
/// dahil); `from_date` boşsa bugün, `to_date` boşsa başlangıçtan 30 gün sonrası.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct MuhurtaQuery {
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub place: MuhurtaPlace,
    pub criteria: MuhurtaCriteria,
    pub natal: Option<NatalMoon>,
}

/// Sonraki dilimin imleci: ilk çağrıda çözülen aralık ve çekirdeğin imleci. Arayüz olduğu gibi
/// geri gönderir; böylece gece yarısını aşan bir arama aralığını değiştirmez.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
pub struct MuhurtaPageCursor {
    pub from_jd_ut: f64,
    pub to_jd_ut: f64,
    pub cursor: MuhurtaCursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Type)]
pub struct MuhurtaWindowView {
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
    /// Başlangıç ve bitiş anlarındaki UTC ofseti (yaz saati pencerenin içinde değişebilir).
    pub start_offset_secs: i32,
    pub end_offset_secs: i32,
    pub snapshot: WindowSnapshot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct MuhurtaSearchView {
    /// Bu dilimde kapanan pencereler.
    pub windows: Vec<MuhurtaWindowView>,
    pub scanned: u32,
    pub chunks: u32,
    /// Taranan aralığın tamamı ve uçlarındaki ofset.
    pub from_jd_ut: f64,
    pub to_jd_ut: f64,
    pub from_offset_secs: i32,
    pub to_offset_secs: i32,
    /// Sonraki dilimin imleci; arama bittiyse `null`.
    pub next: Option<MuhurtaPageCursor>,
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

fn invalid(field: &str) -> CommandError {
    CommandError::new("invalid_muhurta_input", &[("field", field.to_owned())])
}

fn tz_spec(tz: &TzChoice) -> TzSpec {
    match tz {
        TzChoice::Iana(id) => TzSpec::Iana(id.trim().to_owned()),
        TzChoice::Fixed(secs) => TzSpec::Fixed(*secs),
        TzChoice::Lmt => TzSpec::Lmt,
    }
}

fn timestamp_of(jd_ut: f64) -> Option<Timestamp> {
    let secs = (jd_ut - UNIX_EPOCH_JD) * 86_400.0;
    if secs.is_finite() && secs.abs() < 1e13 {
        Timestamp::from_second(secs.floor() as i64).ok()
    } else {
        None
    }
}

/// Yerin denetimi ve bir andaki ofseti. Geçersiz koordinat ya da çözülemeyen saat dilimi
/// yalnızca alan adıyla döner.
fn offset_at(place: &MuhurtaPlace, jd_ut: f64) -> Result<i32, CommandError> {
    if !((-90.0..=90.0).contains(&place.lat) && (-180.0..=180.0).contains(&place.lon)) {
        return Err(invalid("place"));
    }
    let utc = timestamp_of(jd_ut).ok_or_else(|| invalid("range"))?;
    atlas::offset_at(&tz_spec(&place.tz), place.lon, utc).map_err(|_| invalid("place"))
}

/// `day_panchang` gövdesi; `now` testte sabitlenir. Bloklar.
pub fn day_panchang_blocking(
    engine: &Engine,
    at: TransitAt,
    now: Timestamp,
    place: MuhurtaPlace,
    settings: CalcSettings,
    natal: Option<NatalMoon>,
) -> Result<DayPanchangView, CommandError> {
    let (jd_ut, _) = transit_moment(&at, now)?;
    // Yer, motoru meşgul etmeden önce denetlenir.
    offset_at(&place, jd_ut)?;
    let (lat, lon) = (place.lat, place.lon);
    let day = engine.call(move |se: &mut SwissEph| {
        jyotish_core::day_panchang(se, jd_ut, lat, lon, &settings, natal)
    })??;
    let offset_secs = offset_at(&place, day.day_start_jd_ut)?;
    Ok(DayPanchangView { day, offset_secs })
}

/// Gün panchangı: `at` anını içeren gün (gün doğumundan sonraki gün doğumuna) için beş uzvun
/// gün içindeki değerleri, günün dilimleri, işaretler ve (doğum Ay'ı verilirse) tara ve chandra
/// bala. `at` transit anıdır ("şimdi" Rust'ta okunur). Yer kişisel veridir: loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn day_panchang(
    engine: State<'_, Arc<Engine>>,
    at: TransitAt,
    place: MuhurtaPlace,
    settings: CalcSettings,
    natal: Option<NatalMoon>,
) -> Result<DayPanchangView, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || day_panchang_blocking(&engine, at, Timestamp::now(), place, settings, natal))
        .await
}

/// Yerin takvim gününün başı (00:00 duvar saati). Gece yarısı yaz saati boşluğuna düşerse
/// öğleden 12 saat öncesi alınır.
fn day_start_jd(date: &str, place: &MuhurtaPlace, field: &str) -> Result<f64, CommandError> {
    let date: atlas::Date = date.trim().parse().map_err(|_| invalid(field))?;
    let resolve = |time: &str| {
        store::resolve_birth(&BirthData {
            date: date.to_string(),
            time: Some(time.to_owned()),
            time_certainty: TimeCertainty::Exact,
            place: Place {
                name: String::new(),
                country: String::new(),
                admin1: String::new(),
                lat: place.lat,
                lon: place.lon,
                geoname_id: None,
            },
            tz: place.tz.clone(),
            fold: Some(FoldChoice::Earlier),
            notes: String::new(),
        })
        .map(|(_, r)| r.jd_ut)
    };
    resolve("00:00")
        .or_else(|_| resolve("12:00").map(|noon| noon - 0.5))
        .map_err(|_| invalid(field))
}

/// Yerin bugünkü takvim günü (`YYYY-MM-DD`).
fn today(place: &MuhurtaPlace, now: Timestamp) -> Result<String, CommandError> {
    let offset =
        atlas::offset_at(&tz_spec(&place.tz), place.lon, now).map_err(|_| invalid("place"))?;
    let local = Timestamp::from_second(now.as_second() + i64::from(offset))
        .map_err(|_| invalid("range"))?;
    Ok(local.strftime("%Y-%m-%d").to_string())
}

/// Aramanın aralığı: imleç varsa onunki, yoksa tarihlerden çözülür.
fn search_range(
    query: &MuhurtaQuery,
    cursor: Option<&MuhurtaPageCursor>,
    now: Timestamp,
) -> Result<(f64, f64), CommandError> {
    if let Some(c) = cursor {
        return Ok((c.from_jd_ut, c.to_jd_ut));
    }
    let from_date = match &query.from_date {
        Some(d) => d.clone(),
        None => today(&query.place, now)?,
    };
    let from = day_start_jd(&from_date, &query.place, "from_date")?;
    let to = match &query.to_date {
        Some(d) => day_start_jd(d, &query.place, "to_date")? + 1.0,
        None => from + DEFAULT_SEARCH_DAYS,
    };
    Ok((from, to))
}

/// `muhurta_search` gövdesi; `now` testte sabitlenir. Bloklar ve tek bir dilim hesaplar.
pub fn muhurta_search_blocking(
    engine: &Engine,
    query: MuhurtaQuery,
    settings: CalcSettings,
    cursor: Option<MuhurtaPageCursor>,
    now: Timestamp,
) -> Result<MuhurtaSearchView, CommandError> {
    let (from_jd_ut, to_jd_ut) = search_range(&query, cursor.as_ref(), now)?;
    let from_offset_secs = offset_at(&query.place, from_jd_ut)?;
    let to_offset_secs = offset_at(&query.place, to_jd_ut)?;
    let filter = MuhurtaFilter {
        from_jd_ut,
        to_jd_ut,
        lat: query.place.lat,
        lon: query.place.lon,
        criteria: query.criteria.clone(),
        natal: query.natal,
    };
    let core_cursor = cursor.map(|c| c.cursor);
    let page = engine.call(move |se: &mut SwissEph| {
        jyotish_core::muhurta_search_page(se, &settings, &filter, core_cursor)
    })??;
    let windows = page
        .windows
        .iter()
        .map(|w| {
            Ok(MuhurtaWindowView {
                start_jd_ut: w.start_jd_ut,
                end_jd_ut: w.end_jd_ut,
                start_offset_secs: offset_at(&query.place, w.start_jd_ut)?,
                end_offset_secs: offset_at(&query.place, w.end_jd_ut)?,
                snapshot: w.snapshot,
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;
    Ok(MuhurtaSearchView {
        windows,
        scanned: page.scanned,
        chunks: page.chunks,
        from_jd_ut,
        to_jd_ut,
        from_offset_secs,
        to_offset_secs,
        next: page.next.map(|cursor| MuhurtaPageCursor {
            from_jd_ut,
            to_jd_ut,
            cursor,
        }),
        warnings: page.warnings,
        method: page.method,
    })
}

/// Muhurta araması, bir dilim: süzgece uyan zaman pencereleri. İlk çağrıda `cursor` boştur;
/// dönen `next` bir sonraki çağrıya olduğu gibi verilir, `null` ise arama bitmiştir. Pencere
/// dilim sınırını aşarsa kapandığı dilimde verilir. Yer ve doğum Ay'ı loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn muhurta_search(
    engine: State<'_, Arc<Engine>>,
    query: MuhurtaQuery,
    settings: CalcSettings,
    cursor: Option<MuhurtaPageCursor>,
) -> Result<MuhurtaSearchView, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || muhurta_search_blocking(&engine, query, settings, cursor, Timestamp::now()))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(tz: TzChoice) -> MuhurtaPlace {
        MuhurtaPlace {
            lat: 41.0138,
            lon: 28.9497,
            tz,
        }
    }

    fn istanbul() -> MuhurtaPlace {
        place(TzChoice::Iana("Europe/Istanbul".into()))
    }

    fn query(from: Option<&str>, to: Option<&str>, place: MuhurtaPlace) -> MuhurtaQuery {
        MuhurtaQuery {
            from_date: from.map(str::to_owned),
            to_date: to.map(str::to_owned),
            place,
            criteria: MuhurtaCriteria::default(),
            natal: None,
        }
    }

    /// 2026-11-26T22:30:00Z: İstanbul'da 27 Kasım 01:30.
    fn now() -> Timestamp {
        "2026-11-26T22:30:00Z".parse().unwrap()
    }

    /// 2026-11-27T00:00 İstanbul (+03:00) = 2026-11-26T21:00:00Z.
    const NOV_27_ISTANBUL: f64 = 2_461_371.375;

    #[test]
    fn ranges_are_whole_local_days_and_default_to_today_plus_thirty() {
        let r = search_range(&query(None, None, istanbul()), None, now()).unwrap();
        assert_eq!(r, (NOV_27_ISTANBUL, NOV_27_ISTANBUL + 31.0));
        // İki tarih de dahildir.
        let q = query(Some("2026-11-27"), Some("2026-11-27"), istanbul());
        assert_eq!(
            search_range(&q, None, now()).unwrap(),
            (NOV_27_ISTANBUL, NOV_27_ISTANBUL + 1.0)
        );
        // Sabit ofset ve yerel ortalama saat (28,9497° × 240 = 6948 sn).
        let q = query(Some("2026-11-27"), None, place(TzChoice::Fixed(0)));
        assert_eq!(search_range(&q, None, now()).unwrap().0, 2_461_371.5);
        let q = query(Some("2026-11-27"), None, place(TzChoice::Lmt));
        let lmt = search_range(&q, None, now()).unwrap().0;
        assert!((lmt - (2_461_371.5 - 6_947.928 / 86_400.0)).abs() < 1e-6);
        // UTC'de hâlâ 26 Kasım'dır.
        let r = search_range(&query(None, None, place(TzChoice::Fixed(0))), None, now()).unwrap();
        assert_eq!(r.0, 2_461_370.5);
        // İmleç varsa aralık ondan okunur; tarihler ve saat yeniden çözülmez.
        let cursor = MuhurtaPageCursor {
            from_jd_ut: 10.0,
            to_jd_ut: 20.0,
            cursor: MuhurtaCursor {
                chunk: 1,
                open_start_jd_ut: None,
            },
        };
        let q = query(Some("bozuk"), None, istanbul());
        assert_eq!(
            search_range(&q, Some(&cursor), now()).unwrap(),
            (10.0, 20.0)
        );
    }

    #[test]
    fn a_midnight_inside_a_dst_gap_falls_back_to_twelve_hours_before_noon() {
        // Asunción 2010-10-03: saatler 00:00'da 01:00'e alındı; 00:00 yoktur.
        let asuncion = MuhurtaPlace {
            lat: -25.3,
            lon: -57.64,
            tz: TzChoice::Iana("America/Asuncion".into()),
        };
        let start = day_start_jd("2010-10-03", &asuncion, "from_date").unwrap();
        // Öğle 12:00 −03:00 = 15:00 UTC; 12 saat öncesi 03:00 UTC.
        assert!((start - 2_455_472.625).abs() < 1e-9, "{start}");
    }

    #[test]
    fn errors_name_the_field_and_never_carry_the_place() {
        let field = |e: CommandError| {
            assert_eq!(e.code, "invalid_muhurta_input");
            assert_eq!(e.params.len(), 1, "{:?}", e.params);
            e.params["field"].clone()
        };
        let bad_date = query(Some("27.11.2026"), None, istanbul());
        assert_eq!(
            field(search_range(&bad_date, None, now()).unwrap_err()),
            "from_date"
        );
        let bad_to = query(Some("2026-11-27"), Some("2026-13-01"), istanbul());
        assert_eq!(
            field(search_range(&bad_to, None, now()).unwrap_err()),
            "to_date"
        );
        for bad in [
            place(TzChoice::Iana("Mars/Olympus".into())),
            place(TzChoice::Fixed(19 * 3600)),
            MuhurtaPlace {
                lat: 91.0,
                ..istanbul()
            },
            MuhurtaPlace {
                lon: f64::NAN,
                ..istanbul()
            },
        ] {
            assert_eq!(
                field(offset_at(&bad, NOV_27_ISTANBUL).unwrap_err()),
                "place"
            );
        }
        assert_eq!(
            field(offset_at(&istanbul(), f64::INFINITY).unwrap_err()),
            "range"
        );
        assert_eq!(offset_at(&istanbul(), NOV_27_ISTANBUL).unwrap(), 10_800);
        assert_eq!(
            offset_at(&place(TzChoice::Lmt), NOV_27_ISTANBUL).unwrap(),
            6_948
        );
    }
}
````


- [ ] **Step 4: Süzgeç komutları ve komut kaydı**

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index 1152a17..fa5b6d7 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -38,6 +38,8 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             module_commands::compute_shadbala,
             module_commands::compute_yogas,
             module_commands::compute_compat,
+            muhurta_commands::day_panchang,
+            muhurta_commands::muhurta_search,
             client_commands::open_client,
             client_commands::restore_session,
             client_commands::create_client,
@@ -63,6 +65,8 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             store_commands::list_tags,
             store_commands::get_app_prefs,
             store_commands::set_app_prefs,
+            store_commands::load_muhurta_filters,
+            store_commands::save_muhurta_filters,
             store_commands::load_panel_layout,
             store_commands::save_panel_layout,
             store_commands::export_clients_json,
diff --git a/app/src-tauri/src/store_commands.rs b/app/src-tauri/src/store_commands.rs
index 40a77fa..6d14b52 100644
--- a/app/src-tauri/src/store_commands.rs
+++ b/app/src-tauri/src/store_commands.rs
@@ -12,7 +12,7 @@ use serde::Serialize;
 use specta::Type;
 use store::{
     AppPrefs, ChartBrief, ChartRecord, Client, ExportOptions, Gender, ImportReport, Note,
-    PanelLayout, PrefsLoad, SearchHit, Store, TagCount,
+    PanelLayout, PrefsLoad, SavedMuhurtaFilter, SearchHit, Store, TagCount,
 };
 use tauri::{AppHandle, State, WebviewWindow};
 use tauri_plugin_dialog::{DialogExt, FilePath};
@@ -434,6 +434,26 @@ pub async fn set_app_prefs(
     with_store(&store, move |s| s.set_app_prefs(&prefs)).await
 }
 
+/// Kayıtlı muhurta süzgeçleri (ad ve koşullar; yer, tarih ve kişi kaydedilmez). Kayıt yoksa ya
+/// da okunamıyorsa liste boştur.
+#[tauri::command]
+#[specta::specta]
+pub async fn load_muhurta_filters(
+    store: State<'_, StoreState>,
+) -> Result<Vec<SavedMuhurtaFilter>, CommandError> {
+    with_store(&store, |s| s.muhurta_filters()).await
+}
+
+/// Kayıtlı süzgeç listesinin tamamını yazar ve saklanan (kanonik) listeyi döner.
+#[tauri::command]
+#[specta::specta]
+pub async fn save_muhurta_filters(
+    store: State<'_, StoreState>,
+    filters: Vec<SavedMuhurtaFilter>,
+) -> Result<Vec<SavedMuhurtaFilter>, CommandError> {
+    with_store(&store, move |s| s.save_muhurta_filters(&filters)).await
+}
+
 /// Kullanıcının pano düzeni (`app_state`); kayıt yoksa ya da okunamazsa `None`.
 #[tauri::command]
 #[specta::specta]
````


- [ ] **Step 5: Sarmalayıcılar, fikstür tipi ve e2e taklidi**

E2e taklidinde tercihler ve kayıtlı süzgeçler bellekte tutulur: bulunulan yer penceresi ve süzgeç kaydı gerçek bir gidiş dönüş görür.

````diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index 26931a2..0df1043 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -40,6 +40,10 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
       let noteSeq = 0;
       // Etiketler bellek içi: harita kimliği → etiketler (Rust normalleştirmesi taklit edilmez).
       const tags: Record<number, string[]> = {};
+      // Tercihler ve kayıtlı muhurta süzgeçleri bellek içi: bulunulan yer penceresi ve süzgeç
+      // kaydı gerçek bir gidiş dönüş görür.
+      let prefs = fixture.prefs.prefs;
+      let muhurtaFilters: unknown[] = [];
       const chartOf = (clientId: number) => opened[clientId].chart_record.id;
       const stamp = () => new Date(Date.UTC(2026, 10, 25, 10, 0, noteSeq)).toISOString();
       const respond: Record<string, (args: Record<string, unknown>) => unknown> = {
@@ -83,8 +87,11 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
         },
         restore_session: () => (session ? fixture.opened : null),
         load_panel_layout: () => null,
-        get_app_prefs: () => fixture.prefs,
-        set_app_prefs: () => null,
+        get_app_prefs: () => ({ ...fixture.prefs, prefs }),
+        set_app_prefs: (args) => {
+          prefs = args.prefs as typeof prefs;
+          return null;
+        },
         // Yeniden hesap taklit edilmez: aynı harita, yeni ayarların kopyasıyla döner.
         save_calc_settings: (args) => ({ ...fixture.opened.chart, meta: { ...fixture.opened.chart.meta, settings: args.settings } }),
         save_panel_layout: () => null,
@@ -136,6 +143,18 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
           }
           return bride.time_known ? fixture.compat.report : fixture.compat.report_time_unknown;
         },
+        // Muhurta: gün yeniden hesaplanmaz, fikstürdeki gerçek gün döner. Arama fikstürün üç
+        // dilimini imlecin sırasıyla verir (ilk çağrıda imleç yoktur).
+        day_panchang: () => fixture.muhurta.day,
+        muhurta_search: (args) => {
+          const cursor = args.cursor as { cursor: { chunk: number } } | null;
+          return fixture.muhurta.pages[cursor === null ? 0 : cursor.cursor.chunk];
+        },
+        load_muhurta_filters: () => muhurtaFilters,
+        save_muhurta_filters: (args) => {
+          muhurtaFilters = args.filters as unknown[];
+          return muhurtaFilters;
+        },
         print_window: () => null,
         // Güncelleme: sahte yayın; ağ yoktur. Kurulum yalnızca denetimin bildirdiği sürümü kabul eder.
         app_info: () => ({ version: "0.9.0", updaterConfigured: true }),
diff --git a/app/ui/api.ts b/app/ui/api.ts
index 016e00f..be3b6ff 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -15,10 +15,15 @@ import {
   type DashaSeed,
   type Gender,
   type Graha,
+  type MuhurtaPageCursor,
+  type MuhurtaPlace,
+  type MuhurtaQuery,
+  type NatalMoon,
   type NewChart,
   type NewClient,
   type BirthData,
   type PanelLayout,
+  type SavedMuhurtaFilter,
   type Sign,
   type TransitAt,
   type ViewState,
@@ -98,6 +103,14 @@ export const api = {
   computeCompat: (groom: ChartInput, bride: ChartInput) => wrap(() => commands.computeCompat(groom, bride)),
   chartInput: (chartUuid: string) => wrap(() => commands.chartInput(chartUuid)),
   chartBrief: (chartUuid: string) => wrap(() => commands.chartBrief(chartUuid)),
+  // Muhurta: gün panchangı transit anını içeren gündür ("şimdi" Rust'ta okunur). Arama dilim
+  // dilim yürür: ilk çağrıda `cursor` boştur, dönen `next` sonraki çağrıya olduğu gibi verilir;
+  // aramayı durdurmak sonraki dilimi istememektir. Yer ve doğum Ay'ı kişisel veridir: loglanmaz.
+  dayPanchang: (at: TransitAt, place: MuhurtaPlace, settings: CalcSettings, natal: NatalMoon | null) => wrap(() => commands.dayPanchang(at, place, settings, natal)),
+  muhurtaSearch: (query: MuhurtaQuery, settings: CalcSettings, cursor: MuhurtaPageCursor | null) => wrap(() => commands.muhurtaSearch(query, settings, cursor)),
+  // Kayıtlı süzgeçler: ad ve koşullar; yer, tarih ve kişi kaydedilmez. Kayıt listenin tamamını yazar.
+  loadMuhurtaFilters: () => wrap(() => commands.loadMuhurtaFilters()),
+  saveMuhurtaFilters: (filters: SavedMuhurtaFilter[]) => wrap(() => commands.saveMuhurtaFilters(filters)),
   searchPlaces: (query: string, limit: number) => wrap(() => commands.searchPlaces(query, limit)),
   previewMoment: (birth: BirthData) => wrap(() => commands.previewMoment(birth)),
   appInfo: () => wrap(() => commands.appInfo()),
diff --git a/app/ui/test/fixture.ts b/app/ui/test/fixture.ts
index e391c4a..17c0f3b 100644
--- a/app/ui/test/fixture.ts
+++ b/app/ui/test/fixture.ts
@@ -1,20 +1,29 @@
 // Gerçek veri fikstürü (app/src-tauri/tests/ui_fixture.rs üretir): Delhi 2000-01-01 17:30
 // için `create_client` dönüşü, ilk 3. seviye dönemin alt dönemleri, "new delhi" atlas araması,
 // an önizlemesi, bir DST boşluğu hatası, aynı sabit saatte transit, bakış tablosu, Jaimini raporu,
-// chara dasha, Shadbala raporu, yoga raporu (kataloğuyla) ve ikinci bir danışanla uyum raporları.
-// Arayüz hesap yapmadığı için testler bunu çizer.
+// chara dasha, Shadbala raporu, yoga raporu (kataloğuyla), ikinci bir danışanla uyum raporları,
+// gün panchangı (iki gün) ve üç dilimlik bir muhurta araması. Arayüz hesap yapmadığı için testler
+// bunu çizer.
 import type {
   CharaDasha,
   ChartBrief,
   ChartInput,
   CommandError,
   DashaPeriod,
+  DayPanchangView,
   DrishtiTable,
   Graha,
+  HomePlace,
   JaiminiReport,
   Kuta,
   KutaReport,
   MomentPreview,
+  MuhurtaFlag,
+  MuhurtaPlace,
+  MuhurtaQuery,
+  MuhurtaSearchView,
+  NakshatraNature,
+  NatalMoon,
   OpenedClient,
   PlaceHit,
   PrefsLoad,
@@ -22,6 +31,7 @@ import type {
   Sign,
   SignDashaPath,
   SignDashaPeriod,
+  TithiGroup,
   TransitView,
   YogaGroup,
   YogaReport,
@@ -76,6 +86,24 @@ export interface UiFixture {
   };
   /** Rust `kuta_class_ids`: her kutanın alabileceği sınıf kimlikleri. */
   kuta_classes: Record<Kuta, string[]>;
+  /**
+   * Muhurta: `place` açık haritanın doğum yeridir (Delhi, Asia/Kolkata), `natal` açık haritanın
+   * Ay'ı, `home_place` aynı yerin `AppPrefs.home_place` biçimi. `day`, fikstürün sabit saatini
+   * ("şimdi") içeren günün `day_panchang` dönüşüdür; `day_polar` kutup gecesinde bir gündür
+   * (Tromsø, 2050-12-21: dilimler hesaplanmaz). `pages`, `query` için `muhurta_search`'ün üç
+   * diliminin dönüşüdür (67 gün; Perşembe gündüzleri, üç işaretli dilim dışlanmış).
+   */
+  muhurta: {
+    natal: NatalMoon;
+    place: MuhurtaPlace;
+    home_place: HomePlace;
+    day: DayPanchangView;
+    day_polar: DayPanchangView;
+    query: MuhurtaQuery;
+    pages: MuhurtaSearchView[];
+  };
+  /** Rust `MuhurtaFlag::ALL`, `TithiGroup::ALL`, `NakshatraNature::ALL` ve `MUHURTA_MAX_MIN_MINUTES`. */
+  muhurta_ids: { flags: MuhurtaFlag[]; tithi_groups: TithiGroup[]; nakshatra_natures: NakshatraNature[]; max_min_minutes: number };
 }
 
 const FIXTURE = JSON.parse(raw) as UiFixture;
````


- [ ] **Step 6: Üretilen dosyaları yenile**

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
git diff --numstat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
grep -cE "dayPanchang|muhurtaSearch|loadMuhurtaFilters|saveMuhurtaFilters" app/ui/bindings.ts
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
```
Expected: iki `ok`; `bindings.ts` 287 satır kazanır (dört komut ve muhurta tipleri: `Interval<T>`, `DayPanchang`, `DaySegments`, `MuhurtaCriteria`, `MuhurtaFlag` …); `delhi.json` tek satır değişir; ilk `grep -c` `4`, ikincisi `0` yazar. Fikstürdeki değerler sonraki görevlerin panel testlerinin beklediği değerlerdir: `muhurta.day` 2026-11-25 Çarşamba'dır (Delhi; gün doğumu 06:55, batımı 17:20; Yamaganda 2., Gulika 4., Rahu 5. dilim; iki tithi, beş pada, iki yoga, üç karana; işaret yok; tara 8 → 9, chandra bala 8. ev); `muhurta.day_polar` 2050-12-21'dir (gün doğumu yok; üç işaret: Rikta, Vyatipata, Panchaka); `muhurta.pages` üç dilimdir (15, 12 ve 3 pencere; aralık 2026-11-26 – 2027-01-31). Bu değerler çıkmazsa Task 1–3'ün motoru plandakinden farklıdır; fikstür testlere uydurulmaz.

- [ ] **Step 7: Kapıları koş**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -1
cargo test -p jyotish-desk 2>&1 | grep -E "^test result|Running"
cargo test -p jyotish-desk --test modules -- --nocapture 2>&1 | grep -E "median|muhurta_search"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: fmt çıktısız; clippy `Finished`; `jyotish-desk` lib `49 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `modules` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; on `median` satırı (önceki dokuz ölçüm ve `day_panchang`; ölçüm: 0,55 ms, sınır 50 ms) ve üç `muhurta_search` satırı: 366 gün toplam 2 sn'nin, lagna koşuluyla 5 sn'nin, her dilim 300 ms'nin, ilk sayfa 500 ms'nin altında (ölçüm: 0,20 sn ve 77 pencere; 0,42 sn ve 36 pencere; yalnızca lagna 0,25 sn ve 367 pencere; en yavaş dilim 37 ms); Vitest `75 passed (75)`, `738 passed (738)`; `✓ built` (`tsc -p e2e` hatasız).

- [ ] **Step 8: Commit**

```bash
git add app/src-tauri/src app/src-tauri/tests app/ui/bindings.ts app/ui/test app/ui/api.ts app/ui/api.test.ts app/e2e/ipc.ts
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(app): day_panchang and muhurta_search commands, saved filter commands; bindings, wrappers and fixture; timeline cross-checks on the engine" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: `grep` çıktısız.

---


### Task 6: Arayüz: Gün panchangı paneli ve bulunulan yer penceresi

Spec §10 "Arayüz" (`muhurta_day` satırı ve üç madde), "Bulunulan yer"; §13 "Arayüz" (panel durumları, ortak seçim, klavye, i18n); K11, K12; UI spec §3.3, §3.4, §5, §6; D27–D30, D32–D35. Arayüz hesap yapmaz (C4). Muhurta panellerinin ortak parçaları (yer seçimi, doğum Ay'ı, koşul yardımcıları, saat ve süre biçimleri, klasik adlar) bu görevdedir; arama paneli Task 7'dedir.

**Files:**
- Create: `app/ui/modules/muhurta.ts`, `app/ui/modules/muhurta.test.ts`, `app/ui/modules/useDayPanchang.ts`, `app/ui/panels/HomePlaceDialog.tsx`, `app/ui/panels/HomePlaceDialog.test.tsx`, `app/ui/panels/MuhurtaPlace.tsx`, `app/ui/panels/MuhurtaDayPanel.tsx`, `app/ui/panels/MuhurtaDayPanel.test.tsx`
- Modify: `app/ui/transit/store.tsx`, `app/ui/transit/store.test.tsx`, `app/ui/modules/useModule.ts`, `app/ui/modules/useModule.test.tsx`, `app/ui/panels/format.ts`, `app/ui/panels/format.test.ts`, `app/ui/panels/labels.ts`, `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx`, `app/ui/panels/panels.css`, `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: `modules/muhurta.test.ts` (8), `HomePlaceDialog.test.tsx` (6), `MuhurtaDayPanel.test.tsx` (15), `useModule.test.tsx` (+1), `format.test.ts` (+3); `transit/store.test.tsx`, `layout/model.test.ts` ve `panels.test.tsx`'in mevcut testleri büyür

**Interfaces:**
- Consumes (Task 5, önceki planlar): `api.dayPanchang`, `api.setAppPrefs`, `api.searchPlaces`; bağlama tipleri `DayPanchangView`, `DayPanchang`, `Interval`, `HomePlace`, `MuhurtaPlace`, `MuhurtaCriteria`, `MuhurtaFlag`, `NatalMoon`, `TransitAt`, `PlaceHit`, `AppPrefs`; `useModule`, `usePanelLoadPhase`, `PanelError`, `PanelSkeleton`, `BalaScroll`, `Approx`, `PinButton`, `useHighlight`, `usePrefs`, `useLayoutStore`, `useTransitControl`, `TransitDate`, `PlaceSearch`, `Dialog`, `formatJdMinute`, `timeUnknown`; fikstürde `muhurta`, `muhurta_ids`.
- Produces:
  - `Transit.epoch: number`; `useLastData(state, scope)`.
  - `modules/muhurta.ts`: `PLACE_CHOICES`, `PlaceChoice`, `VARAS`, `MUHURTA_FLAGS`, `MAX_MIN_MINUTES`, `placeChoice`, `parseHomePlace`, `hitPlace`, `chartPlace`, `resolvePlace`, `muhurtaPlace`, `placeLabel`, `natalMoon`, `EMPTY_CRITERIA`, `parseCriteria`, `toggleMember`.
  - `useDayPanchang(opened, place, at, epoch, scope) -> DayPanchangState` (`shown`).
  - `formatJdClock`, `formatJdDate`, `formatJdSpan`; `TARA_NAMES`, `taraName`, `NAKSHATRA_NATURE_NAMES`, `tithiShort`.
  - `HomePlaceDialog`, `tzLabel`; `usePanelPlace`, `PlaceControls`, `PlaceMissing`; `MuhurtaDayPanel`, `DAY_LINKS`, `dayLink`.
  - Panel türü `muhurta_day`; i18n `panel.muhurta_day`, `muhurta.*`, `muhurtaDay.*`.

- [ ] **Step 1: Testler**

Üç yeni dosya ve beş mevcut dosyanın büyüyen testleri. Panel testleri fikstürün gerçek gününü çizer (D36): beklenen saatler ve adlar fikstürden okunmuştur; **bir beklenti tutmazsa fikstür değil panel düzeltilir.** Odak testleri D27 ve D32'nin kurallarını sabitler.

**Yeni dosya:** `app/ui/modules/muhurta.test.ts`

````ts
import { expect, test } from "vitest";
import type { HomePlace } from "../bindings";
import { fixture, unknownTimeOpened } from "../test/fixture";
import {
  chartPlace,
  EMPTY_CRITERIA,
  hitPlace,
  MAX_MIN_MINUTES,
  MUHURTA_FLAGS,
  muhurtaPlace,
  natalMoon,
  parseCriteria,
  parseHomePlace,
  placeChoice,
  placeLabel,
  resolvePlace,
  toggleMember,
  VARAS,
} from "./muhurta";

const FX = fixture();
const DELHI: HomePlace = FX.muhurta.home_place;

test("the place choice defaults to the current place and unknown values fall back to it", () => {
  expect(placeChoice({})).toBe("home");
  expect(placeChoice({ place: "chart" })).toBe("chart");
  expect(placeChoice({ place: "custom" })).toBe("custom");
  expect(placeChoice({ place: "moon" })).toBe("home");
  expect(placeChoice({ place: 3 })).toBe("home");
});

test("the place comes from the preferences, the open chart or the panel's own setting", () => {
  const opened = FX.opened;
  const prefs = { ...FX.prefs.prefs, home_place: DELHI };
  expect(chartPlace(opened)).toEqual({ place: opened.chart_record.birth.place, tz: { Iana: "Asia/Kolkata" } });
  expect(resolvePlace("chart", opened, null, null)).toEqual(chartPlace(opened));
  expect(resolvePlace("home", opened, prefs, null)).toEqual(DELHI);
  // Bulunulan yer seçilmemiş ya da tercihler okunamamış: yer yok (doğum yerine sessizce düşülmez).
  expect(resolvePlace("home", opened, FX.prefs.prefs, null)).toBeNull();
  expect(resolvePlace("home", opened, null, DELHI)).toBeNull();
  expect(resolvePlace("custom", opened, prefs, null)).toBeNull();
  const tokyo: HomePlace = { place: { name: "Tokyo", country: "Japan", admin1: "Tokyo", lat: 35.69, lon: 139.69, geoname_id: 1850147 }, tz: { Iana: "Asia/Tokyo" } };
  expect(resolvePlace("custom", opened, prefs, tokyo)).toEqual(tokyo);
  // Komuta yalnızca koordinat ve saat dilimi gider; ad gitmez.
  expect(muhurtaPlace(tokyo)).toEqual({ lat: 35.69, lon: 139.69, tz: { Iana: "Asia/Tokyo" } });
  expect(muhurtaPlace(DELHI)).toEqual(FX.muhurta.place);
});

test("a stored custom place is read tolerantly", () => {
  expect(parseHomePlace(DELHI)).toEqual(DELHI);
  expect(parseHomePlace(JSON.parse(JSON.stringify(DELHI)))).toEqual(DELHI);
  const manual = { place: { lat: -33.5, lon: 151.2 }, tz: { Fixed: 36000 }, extra: 1 };
  expect(parseHomePlace(manual)).toEqual({ place: { name: "", country: "", admin1: "", lat: -33.5, lon: 151.2, geoname_id: null }, tz: { Fixed: 36000 } });
  expect(parseHomePlace({ place: { lat: 1, lon: 2 }, tz: "Lmt" })?.tz).toBe("Lmt");
  for (const broken of [
    null,
    "Delhi",
    [],
    {},
    { place: DELHI.place },
    { place: { ...DELHI.place, lat: 91 }, tz: DELHI.tz },
    { place: { ...DELHI.place, lon: "77" }, tz: DELHI.tz },
    { place: { ...DELHI.place, lat: Number.NaN }, tz: DELHI.tz },
    { place: DELHI.place, tz: { Iana: " " } },
    { place: DELHI.place, tz: { Fixed: 1.5 } },
    { place: DELHI.place, tz: "Utc" },
  ]) {
    expect(parseHomePlace(broken), JSON.stringify(broken)).toBeNull();
  }
});

test("an atlas hit becomes a place with its zone, and a place is named by its parts or its coordinates", () => {
  const hit = FX.places[0];
  const place = hitPlace(hit);
  expect(place).toEqual({ place: { name: "New Delhi", country: hit.country, admin1: hit.admin1, lat: hit.lat, lon: hit.lon, geoname_id: hit.geoname_id }, tz: { Iana: "Asia/Kolkata" } });
  expect(hitPlace({ ...hit, lat: null })).toBeNull();
  expect(placeLabel(DELHI)).toBe("New Delhi, Delhi, India");
  // Yinelenen parça bir kez yazılır; adı olmayan yer koordinatlarıyla anılır.
  expect(placeLabel({ ...DELHI, place: { ...DELHI.place, name: "İstanbul", admin1: "İstanbul", country: "Türkiye" } })).toBe("İstanbul, Türkiye");
  expect(placeLabel({ ...DELHI, place: { ...DELHI.place, name: " ", admin1: "", country: "" } })).toBe("28.61°, 77.21°");
});

test("the natal Moon is read from the computed chart, also when the birth time is unknown", () => {
  const moon = FX.opened.chart.grahas[1];
  expect(moon.graha).toBe("Moon");
  expect(natalMoon(FX.opened)).toEqual({ nakshatra: moon.nakshatra, sign: moon.sign });
  expect(natalMoon(FX.opened)).toEqual(FX.muhurta.natal);
  expect(natalMoon(unknownTimeOpened())).toEqual(FX.muhurta.natal);
});

test("stored criteria are read tolerantly into the canonical order", () => {
  expect(parseCriteria(undefined)).toEqual(EMPTY_CRITERIA);
  expect(parseCriteria("x")).toEqual(EMPTY_CRITERIA);
  expect(parseCriteria(FX.muhurta.query.criteria)).toEqual(FX.muhurta.query.criteria);
  expect(
    parseCriteria({
      weekdays: ["Friday", "Funday", "Sunday", "Friday"],
      tithis: [30, 2, 0, 31, 2.5, "3"],
      nakshatras: [27, 28, 1],
      lagna_signs: ["Leo", "Aries", "Ophiuchus"],
      daytime_only: "yes",
      min_minutes: 99999,
      exclude: ["Gulika", "Rikta", "Eclipse"],
      require_tara: true,
      require_chandra: 1,
      future: true,
    }),
  ).toEqual({
    weekdays: ["Sunday", "Friday"],
    tithis: [2, 30],
    nakshatras: [1, 27],
    lagna_signs: ["Aries", "Leo"],
    daytime_only: false,
    min_minutes: 10080,
    exclude: ["Rikta", "Gulika"],
    require_tara: true,
    require_chandra: false,
  });
  expect(parseCriteria({ min_minutes: -5 }).min_minutes).toBe(0);
  expect(parseCriteria({ min_minutes: 12.5 }).min_minutes).toBe(0);
  // Okunan nesne her çağrıda yenidir.
  expect(parseCriteria(null)).not.toBe(EMPTY_CRITERIA);
});

test("toggling a member keeps the set in the canonical order", () => {
  expect(toggleMember([], "Friday", VARAS)).toEqual(["Friday"]);
  expect(toggleMember(["Friday"], "Monday", VARAS)).toEqual(["Monday", "Friday"]);
  expect(toggleMember(["Monday", "Friday"], "Monday", VARAS)).toEqual(["Friday"]);
  expect(toggleMember([9, 2], 5, [1, 2, 3, 4, 5, 6, 7, 8, 9])).toEqual([2, 5, 9]);
});

test("the weekday and flag lists and the length limit are the Rust ones", () => {
  expect([...MUHURTA_FLAGS]).toEqual(FX.muhurta_ids.flags);
  expect(MAX_MIN_MINUTES).toBe(FX.muhurta_ids.max_min_minutes);
  expect(VARAS).toHaveLength(7);
  expect(VARAS[0]).toBe("Sunday");
});
````

**Yeni dosya:** `app/ui/panels/HomePlaceDialog.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { setAppPrefs: vi.fn(), searchPlaces: vi.fn() } }));

import { api } from "../api";
import type { AppPrefs, HomePlace } from "../bindings";
import { SEARCH_DEBOUNCE_MS } from "../birth/PlaceSearch";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { usePrefs } from "../settings/prefs";
import { HomePlaceDialog, tzLabel } from "./HomePlaceDialog";

const FX = fixture();
const DELHI: HomePlace = FX.muhurta.home_place;
const withHome = (home: HomePlace | null): AppPrefs => ({ ...FX.prefs.prefs, home_place: home });

let prefs: AppPrefs | null;
function Probe() {
  prefs = usePrefs().prefs;
  return null;
}

/** Seçili yeri söyleyen satır (yer arama kutusunun da bir durum satırı vardır). */
const current = () => document.querySelector(".home-place-current");

function setup(initial: AppPrefs | null = withHome(null)) {
  const onClose = vi.fn();
  renderWithStores(
    <>
      <HomePlaceDialog opened={FX.opened} onClose={onClose} />
      <Probe />
    </>,
    { prefs: initial },
  );
  return { onClose };
}

beforeEach(() => {
  vi.mocked(api.setAppPrefs).mockReset();
  vi.mocked(api.searchPlaces).mockReset();
  vi.mocked(api.setAppPrefs).mockResolvedValue({ status: "ok", data: null });
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: FX.places });
});

test("with no place set the dialog offers the birth place of the open chart and saves it", async () => {
  const { onClose } = setup();
  expect(screen.getByRole("dialog", { name: "Bulunulan yer" })).toBeInTheDocument();
  expect(current()).toHaveAttribute("role", "status");
  expect(current()).toHaveTextContent("Bulunulan yer seçilmedi.");
  expect(screen.getByText(/Konum servisi kullanılmaz/)).toBeInTheDocument();
  // Seçilmemişken kaldırma eylemi yoktur; odak önerilen eylemdedir.
  expect(screen.queryByRole("button", { name: "Yeri kaldır" })).toBeNull();
  const use = screen.getByRole("button", { name: "Açık haritanın doğum yerini kullan (New Delhi, Delhi, India)" });
  expect(use).toHaveFocus();
  fireEvent.click(use);
  await waitFor(() => expect(prefs?.home_place).toEqual(DELHI));
  // Diğer tercihler aynen yazılır; yer doğum verisindeki saat dilimiyle saklanır.
  expect(api.setAppPrefs).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.setAppPrefs).mock.calls[0][0]).toEqual(withHome(DELHI));
  expect(current()).toHaveTextContent("Seçili yer: New Delhi, Delhi, India · Asia/Kolkata");
  expect(screen.getByRole("button", { name: "Yeri kaldır" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Kapat" }));
  expect(onClose).toHaveBeenCalledTimes(1);
});

test("a place picked from the atlas is saved with its coordinates and its IANA zone", async () => {
  vi.useFakeTimers();
  try {
    setup();
    const input = screen.getByRole("combobox", { name: "Yer ara" });
    fireEvent.change(input, { target: { value: "delhi" } });
    await act(async () => vi.advanceTimersByTime(SEARCH_DEBOUNCE_MS));
    expect(api.searchPlaces).toHaveBeenCalledWith("delhi", 8);
    fireEvent.keyDown(input, { key: "Enter" });
    await act(async () => vi.advanceTimersByTime(0));
    const hit = FX.places[0];
    expect(vi.mocked(api.setAppPrefs).mock.calls[0][0].home_place).toEqual({
      place: { name: "New Delhi", country: "India", admin1: "Delhi", lat: hit.lat, lon: hit.lon, geoname_id: hit.geoname_id },
      tz: { Iana: "Asia/Kolkata" },
    });
    expect(input).toHaveValue("");
    expect(current()).toHaveTextContent("Seçili yer: New Delhi, Delhi, India · Asia/Kolkata");
  } finally {
    vi.useRealTimers();
  }
});

test("removing the place clears it and moves the focus to the next action instead of dropping it", async () => {
  setup(withHome(DELHI));
  const remove = screen.getByRole("button", { name: "Yeri kaldır" });
  remove.focus();
  fireEvent.click(remove);
  await waitFor(() => expect(prefs?.home_place).toBeNull());
  expect(vi.mocked(api.setAppPrefs).mock.calls[0][0]).toEqual(withHome(null));
  expect(remove).not.toBeInTheDocument();
  expect(screen.getByRole("button", { name: /^Açık haritanın doğum yerini kullan/ })).toHaveFocus();
  expect(document.activeElement).not.toBe(document.body);
  expect(current()).toHaveTextContent("Bulunulan yer seçilmedi.");
});

test("a failed save is reported in the dialog and the place does not change", async () => {
  vi.mocked(api.setAppPrefs).mockResolvedValueOnce({ status: "error", error: { code: "invalid_input", params: { field: "home_place" } } });
  setup(withHome(DELHI));
  fireEvent.click(screen.getByRole("button", { name: "Yeri kaldır" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Geçersiz değer: home_place.");
  expect(prefs?.home_place).toEqual(DELHI);
  expect(screen.getByRole("button", { name: "Yeri kaldır" })).toBeInTheDocument();
  // Sonraki başarılı kayıt hatayı kaldırır.
  fireEvent.click(screen.getByRole("button", { name: "Yeri kaldır" }));
  await waitFor(() => expect(prefs?.home_place).toBeNull());
  expect(screen.queryByRole("alert")).toBeNull();
});

test("without readable preferences the place cannot be changed, and Esc closes the dialog", () => {
  const { onClose } = setup(null);
  expect(screen.getByRole("note")).toHaveTextContent("Tercihler okunamadığı için bulunulan yer değiştirilemiyor.");
  expect(screen.queryByRole("combobox")).toBeNull();
  expect(screen.queryByRole("button", { name: /doğum yerini kullan/ })).toBeNull();
  fireEvent.keyDown(document, { key: "Escape" });
  expect(onClose).toHaveBeenCalledTimes(1);
  expect(api.setAppPrefs).not.toHaveBeenCalled();
});

test("the zone is written as an IANA id, a signed offset or local mean time", () => {
  const at = (tz: HomePlace["tz"]) => tzLabel({ ...DELHI, tz }, "yerel ortalama saat");
  expect(at({ Iana: "Asia/Kolkata" })).toBe("Asia/Kolkata");
  expect(at({ Fixed: 19800 })).toBe("UTC+05:30");
  expect(at({ Fixed: -12600 })).toBe("UTC−03:30");
  expect(at({ Fixed: 0 })).toBe("UTC+00:00");
  expect(at("Lmt")).toBe("yerel ortalama saat");
});
````

**Yeni dosya:** `app/ui/panels/MuhurtaDayPanel.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { dayPanchang: vi.fn(), setAppPrefs: vi.fn(), searchPlaces: vi.fn() } }));

import { api, type Result } from "../api";
import type { AppPrefs, DayPanchangView } from "../bindings";
import { SEARCH_DEBOUNCE_MS } from "../birth/PlaceSearch";
import { useHighlight } from "../highlight/store";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { browserTimeZone, useTransitControl, type Transit } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { NAKSHATRA_NATURE_NAMES, TARA_NAMES } from "./labels";
import { dayLink, MuhurtaDayPanel } from "./MuhurtaDayPanel";

const FX = fixture();
const ok = (view: DayPanchangView = fixture().muhurta.day): Result<DayPanchangView> => ({ status: "ok", data: view });
const failure = { status: "error" as const, error: { code: "calc", params: { detail: "x" } } };
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "md", kind: "muhurta_day", x: 0, y: 0, w: 7, h: 18, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};
const withHome = (): AppPrefs => ({ ...FX.prefs.prefs, home_place: FX.muhurta.home_place });

let transit: Transit;
/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  transit = useTransitControl();
  return <MuhurtaDayPanel panel={active.panels[0]} />;
}

interface Options {
  opened?: ReturnType<typeof fixture>["opened"] | null;
  prefs?: AppPrefs | null;
}
function renderPanel(settings: Record<string, unknown> = { place: "chart" }, { opened = fixture().opened, prefs = FX.prefs.prefs }: Options = {}) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened, prefs });
}

const limbs = () => screen.findByRole("table", { name: "Beş uzuv ve değişim saatleri" });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const rows = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);
const savedSettings = (persistence: ReturnType<typeof renderPanel>["persistence"]) =>
  persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings;

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.dayPanchang).mockImplementation(async () => ok());
  vi.mocked(api.setAppPrefs).mockResolvedValue({ status: "ok", data: null });
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: FX.places });
});

test("draws the day of the transit moment for the birth place: strip, limbs, factors and horas", async () => {
  renderPanel();
  const table = await limbs();
  // Bir istek: transit anı ("şimdi" Rust'ta okunur), yerin koordinatı ve dilimi (ad gitmez),
  // haritanın hesap ayarı ve doğum Ay'ı.
  expect(api.dayPanchang).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.dayPanchang).mock.calls[0]).toEqual(["Now", FX.muhurta.place, FX.opened.chart.meta.settings, FX.muhurta.natal]);
  expect(screen.getByText("New Delhi, Delhi, India")).toBeInTheDocument();
  expect(screen.getByText("2026-11-25 · Budhavara · Çarşamba")).toBeInTheDocument();
  expect(screen.getByText("Gün doğumu 06:55 · gün batımı 17:20 · sonraki gün doğumu 06:56 (+1)")).toBeInTheDocument();

  // Şerit: sekiz dilim; Çarşamba günü Yamaganda 2., Gulika 4., Rahu 5. dilimdir. İşaret yazıdır.
  const strip = screen.getByRole("list", { name: "Gündüzün sekiz dilimi" });
  const segments = within(strip).getAllByRole("listitem");
  expect(segments.map((s) => s.getAttribute("data-mark"))).toEqual([null, "yamaganda", null, "gulika", "rahu", null, null, null]);
  expect(segments[0]).toHaveTextContent("106:55–08:13");
  expect(segments[1]).toHaveTextContent("2Yama08:13–09:31");
  expect(segments[3]).toHaveTextContent("4Guli10:49–12:07");
  expect(segments[4]).toHaveTextContent("5Rahu12:07–13:26");
  expect(segments[7]).toHaveTextContent("816:02–17:20");
  expect(segments.filter((s) => s.classList.contains("is-marked"))).toHaveLength(3);
  expect(screen.getByText("Abhijit 11:47–12:28 (Çarşamba: bazı derlemeler bu gün Abhijit'i kullanmaz.)")).toBeInTheDocument();

  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Öğe", "Değer", "Başlangıç", "Bitiş"]);
  const body = rows(table);
  expect(body.map((r) => r[0])).toEqual(["Tithi", "Tithi", "Nakshatra", "Nakshatra", "Nakshatra", "Nakshatra", "Nakshatra", "Nitya yoga", "Nitya yoga", "Karana", "Karana", "Karana"]);
  expect(body.map((r) => r[1])).toEqual([
    "Krishna 1 · Pratipada · Nanda grubu",
    "Krishna 2 · Dwitiya · Bhadra grubu",
    "Rohini · pada 2 · Dhruva (sabit)",
    "Rohini · pada 3 · Dhruva (sabit)",
    "Rohini · pada 4 · Dhruva (sabit)",
    "Mrigashira · pada 1 · Mridu (yumuşak)",
    "Mrigashira · pada 2 · Mridu (yumuşak)",
    "Shiva",
    "Siddha",
    "Kaulava",
    "Taitila",
    "Garaja",
  ]);
  // Günün dışına taşan uçlar okla ve ekran okuyucu metniyle işaretlidir.
  expect(body[0].slice(2)).toEqual(["06:55 ←günün başından önce başlar", "16:50"]);
  expect(body[1].slice(2)).toEqual(["16:50", "06:56 (+1) →günün sonundan sonra sürer"]);
  expect(body[5].slice(2)).toEqual(["20:36", "01:53 (+1)"]);

  expect(screen.getByText("Bu gün işaret yok.")).toBeInTheDocument();
  const personal = screen.getByRole("table", { name: "Açık haritanın Ay'ına göre" });
  expect(rows(personal)).toEqual([
    ["Tara bala", "8 · Mitra", "elverişli kümede", "06:55 ←günün başından önce başlar", "20:36"],
    ["Tara bala", "9 · Parama Mitra", "elverişli kümede", "20:36", "06:56 (+1) →günün sonundan sonra sürer"],
    ["Chandra bala", "doğum Ay burcundan 8. ev", "elverişli kümede değil", "06:55 ←günün başından önce başlar", "06:56 (+1) →günün sonundan sonra sürer"],
  ]);
  const horas = rows(screen.getByRole("table", { name: "Hora tablosu" }));
  expect(horas).toHaveLength(24);
  expect(horas[0]).toEqual(["1", "Merkür", "06:55", "07:55"]);
  expect(horas.slice(0, 8).map((r) => r[1])).toEqual(["Merkür", "Ay", "Satürn", "Jüpiter", "Mars", "Güneş", "Venüs", "Merkür"]);
  expect(horas[23]).toEqual(["24", "Satürn", "05:56 (+1)", "06:56 (+1)"]);
  expect(screen.getByText("Yöntem: muhurta.v1")).toBeInTheDocument();
  expect(screen.getByText(/olgudur; uygunluk hükmü değildir/)).toBeInTheDocument();
  expect(screen.queryByText("≈")).toBeNull();
});

test("without a current place the panel asks for one, and choosing it in the dialog loads the day", async () => {
  renderPanel({});
  expect(screen.getByRole("combobox", { name: "Yer" })).toHaveValue("home");
  expect(screen.getByRole("note")).toHaveTextContent(/^Bulunulan yer seçilmedi\./);
  await act(() => Promise.resolve());
  expect(api.dayPanchang).not.toHaveBeenCalled();
  const pick = screen.getByRole("button", { name: "Bulunulan yeri seç" });
  pick.focus();
  fireEvent.click(pick);
  const dialog = screen.getByRole("dialog", { name: "Bulunulan yer" });
  fireEvent.click(within(dialog).getByRole("button", { name: /^Açık haritanın doğum yerini kullan/ }));
  await limbs();
  expect(vi.mocked(api.setAppPrefs).mock.calls[0][0].home_place).toEqual(FX.muhurta.home_place);
  expect(vi.mocked(api.dayPanchang).mock.calls[0][1]).toEqual(FX.muhurta.place);
  fireEvent.click(within(dialog).getByRole("button", { name: "Kapat" }));
  // Pencere kapanınca odak açan düğmeye döner; düğme artık "değiştir"dir.
  expect(screen.getByRole("button", { name: "Bulunulan yeri değiştir" })).toHaveFocus();
  expect(screen.queryByRole("note")).toBeNull();
});

test("a saved current place is used, and the place choice is saved with the layout", async () => {
  const { persistence } = renderPanel({}, { prefs: withHome() });
  await limbs();
  expect(vi.mocked(api.dayPanchang).mock.calls[0][1]).toEqual(FX.muhurta.place);
  expect(within(screen.getByRole("combobox", { name: "Yer" })).getAllByRole("option").map((o) => o.textContent)).toEqual(["Bulunulan yer", "Haritanın doğum yeri", "Özel yer"]);
  fireEvent.change(screen.getByRole("combobox", { name: "Yer" }), { target: { value: "custom" } });
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ place: "custom" }));
  // Özel yer seçilene dek gün gösterilmez ve yeni istek gitmez.
  expect(screen.getByRole("note")).toHaveTextContent("Özel yer seçilmedi. Arama kutusundan bir yer seçin.");
  expect(screen.queryByRole("table")).toBeNull();
  expect(api.dayPanchang).toHaveBeenCalledTimes(1);
});

test("a custom place is picked from the atlas and kept in the panel's settings", async () => {
  vi.useFakeTimers();
  try {
    const { persistence } = renderPanel({ place: "custom" });
    const input = screen.getByRole("combobox", { name: "Özel yer ara" });
    fireEvent.change(input, { target: { value: "delhi" } });
    await act(async () => vi.advanceTimersByTime(SEARCH_DEBOUNCE_MS));
    fireEvent.keyDown(input, { key: "Enter" });
    await act(async () => vi.advanceTimersByTime(0));
    const hit = FX.places[0];
    expect(vi.mocked(api.dayPanchang).mock.calls[0][1]).toEqual({ lat: hit.lat, lon: hit.lon, tz: { Iana: "Asia/Kolkata" } });
    expect(savedSettings(persistence)).toEqual({
      place: "custom",
      custom: { place: { name: "New Delhi", country: "India", admin1: "Delhi", lat: hit.lat, lon: hit.lon, geoname_id: hit.geoname_id }, tz: { Iana: "Asia/Kolkata" } },
    });
    expect(input).toHaveValue("");
    expect(screen.getByRole("table", { name: "Hora tablosu" })).toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
});

test("the date follows the transit moment or stays on today, and the choice is saved", async () => {
  expect(dayLink({})).toBe("transit");
  expect(dayLink({ link: "today" })).toBe("today");
  expect(dayLink({ link: "tomorrow" })).toBe("transit");
  const { persistence } = renderPanel();
  await limbs();
  // Transit bağında panel tarih seçicisini de taşır.
  const form = screen.getByRole("form", { name: "Transit anı" });
  fireEvent.change(within(form).getByLabelText("Tarih"), { target: { value: "2030-01-01" } });
  fireEvent.change(within(form).getByLabelText("Saat"), { target: { value: "03:00" } });
  fireEvent.click(within(form).getByRole("button", { name: "Uygula" }));
  const local = { Local: { date: "2030-01-01", time: "03:00", tz: browserTimeZone() } };
  await waitFor(() => expect(vi.mocked(api.dayPanchang).mock.calls.at(-1)?.[0]).toEqual(local));
  expect(api.dayPanchang).toHaveBeenCalledTimes(2);
  // "Bugün": transit anı ne olursa olsun "şimdi" gönderilir ve tarih seçici gösterilmez.
  fireEvent.change(screen.getByRole("combobox", { name: "Tarih bağı" }), { target: { value: "today" } });
  await waitFor(() => expect(vi.mocked(api.dayPanchang).mock.calls.at(-1)?.[0]).toBe("Now"));
  expect(screen.queryByRole("form", { name: "Transit anı" })).toBeNull();
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ place: "chart", link: "today" }));
  // Başka bir panelden gelen transit anı (ör. arama sonucu) "bugün" bağını etkilemez.
  const calls = vi.mocked(api.dayPanchang).mock.calls.length;
  act(() => transit.setAt({ Jd: 2_461_400.5 }));
  await act(() => Promise.resolve());
  expect(api.dayPanchang).toHaveBeenCalledTimes(calls);
});

test("choosing now again reads the day again", async () => {
  renderPanel();
  await limbs();
  expect(api.dayPanchang).toHaveBeenCalledTimes(1);
  act(() => transit.setAt("Now"));
  await waitFor(() => expect(api.dayPanchang).toHaveBeenCalledTimes(2));
  expect(vi.mocked(api.dayPanchang).mock.calls[1][0]).toBe("Now");
});

test("a day without a sunrise shows the limbs and the flags, with a badge and no segments", async () => {
  vi.mocked(api.dayPanchang).mockImplementation(async () => ok(fixture().muhurta.day_polar));
  renderPanel();
  const table = await limbs();
  const note = screen.getByRole("note");
  expect(note).toHaveTextContent("Gün doğumu yok");
  expect(note).toHaveTextContent("gündüz dilimleri ve hora tablosu hesaplanmaz. Gün 06:00 yerel ortalama saatte (05:44) başlar.");
  expect(screen.queryByRole("list", { name: "Gündüzün sekiz dilimi" })).toBeNull();
  expect(screen.queryByRole("table", { name: "Hora tablosu" })).toBeNull();
  expect(screen.queryByText(/^Gün doğumu \d/)).toBeNull();
  expect(rows(table)).toHaveLength(12);
  const flags = within(screen.getByRole("list", { name: "İşaretler" })).getAllByRole("listitem");
  expect(flags.map((f) => f.getAttribute("data-flag"))).toEqual(["Rikta", "Vyatipata", "Panchaka"]);
  expect(flags[0]).toHaveTextContent("Rikta tithi 16:15–05:44 (+1) →günün sonundan sonra sürer");
  expect(flags[1]).toHaveTextContent("Vyatipata 05:44 ←günün başından önce başlar–16:08");
  expect(screen.queryByText("Bu gün işaret yok.")).toBeNull();
  // Kişiye bağlı etken istenmediyse (Rust `null`) tablo yoktur.
  expect(screen.queryByRole("table", { name: "Açık haritanın Ay'ına göre" })).toBeNull();
});

test("an unknown birth time marks the personal factors as approximate", async () => {
  renderPanel({ place: "chart" }, { opened: unknownTimeOpened() });
  await limbs();
  // Doğum Ay'ı yine gönderilir (öğle konumu); tablo "≈" ve açıklamasıyla işaretlenir.
  expect(vi.mocked(api.dayPanchang).mock.calls[0][3]).toEqual(FX.muhurta.natal);
  const caption = screen.getByRole("table", { name: /Açık haritanın Ay'ına göre/ }).querySelector("caption")!;
  expect(caption).toHaveTextContent("≈yaklaşık: doğum saati bilinmiyor, doğum Ay'ı öğlen 12:00 içindir Açık haritanın Ay'ına göre");
  // Günün uzuvları doğum saatine bağlı değildir: işaretlenmez.
  expect(within(screen.getByRole("table", { name: "Beş uzuv ve değişim saatleri" })).queryByText("≈")).toBeNull();
});

test("loading: nothing under a second, then a skeleton, then the day", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<DayPanchangView>) => void;
    vi.mocked(api.dayPanchang).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    await act(async () => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    await act(async () => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    // Yer ve tarih bağı yüklenirken de kullanılabilir.
    expect(screen.getByRole("combobox", { name: "Yer" })).toBeEnabled();
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getAllByRole("table")).toHaveLength(3);
  } finally {
    vi.useRealTimers();
  }
});

test("an error keeps the toolbar, and retry from the keyboard keeps the focus in the panel", async () => {
  vi.mocked(api.dayPanchang).mockResolvedValueOnce(failure);
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Gün panchangı">
      <LivePanel />
    </section>,
    { layout: LAYOUT({ place: "chart" }) },
  );
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  expect(screen.getByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.getByRole("combobox", { name: "Yer" })).toBeInTheDocument();
  expect(screen.getByRole("combobox", { name: "Tarih bağı" })).toBeInTheDocument();
  retry.focus();
  fireEvent.click(retry);
  await limbs();
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Gün panchangı" }));
  expect(api.dayPanchang).toHaveBeenCalledTimes(2);
});

test("while another day is read the old one stays on screen and the focus stays where it was", async () => {
  renderPanel();
  await limbs();
  const lord = within(screen.getByRole("table", { name: "Hora tablosu" })).getAllByRole("button", { name: "Merkür" })[0];
  lord.focus();
  let resolve!: (r: Result<DayPanchangView>) => void;
  vi.mocked(api.dayPanchang).mockReturnValueOnce(new Promise((r) => (resolve = r)));
  act(() => transit.setAt({ Jd: 2_470_162.26 }));
  await waitFor(() => expect(api.dayPanchang).toHaveBeenCalledTimes(2));
  // Eski gün yerinde durur; panel meşgul olduğunu bildirir; odaktaki düğme kaldırılmaz.
  expect(screen.getByText("2026-11-25 · Budhavara · Çarşamba")).toBeInTheDocument();
  expect(document.querySelector(".muhurta-day")).toHaveAttribute("aria-busy", "true");
  expect(lord).toBeInTheDocument();
  expect(lord).toHaveFocus();
  await act(async () => resolve(ok(fixture().muhurta.day_polar)));
  expect(screen.getByText("2050-12-21 · Budhavara · Çarşamba")).toBeInTheDocument();
  expect(document.querySelector(".muhurta-day")).not.toHaveAttribute("aria-busy");
});

test("shared highlight: a hora lord is pinned and every hora of that graha is marked", async () => {
  let highlight!: ReturnType<typeof useHighlight>;
  function Probe() {
    highlight = useHighlight();
    return null;
  }
  renderWithStores(
    <>
      <LivePanel />
      <Probe />
    </>,
    { layout: LAYOUT({ place: "chart" }) },
  );
  const horas = await screen.findByRole("table", { name: "Hora tablosu" });
  const mercury = within(horas).getAllByRole("button", { name: "Merkür" })[0];
  expect(mercury).toHaveAttribute("aria-pressed", "false");
  fireEvent.click(mercury);
  expect(highlight.pinned).toEqual({ type: "graha", id: "Mercury" });
  expect(mercury).toHaveAttribute("aria-pressed", "true");
  const marked = [...horas.querySelectorAll("tr.is-highlighted")].map((r) => r.getAttribute("data-hora"));
  expect(marked).toEqual(["1", "8", "15", "22"]);
  // Başka bir panelden gelen vurgu da satırları işaretler.
  act(() => highlight.togglePin({ type: "graha", id: "Moon" }));
  expect([...horas.querySelectorAll("tr.is-highlighted")].map((r) => r.getAttribute("data-hora"))).toEqual(["2", "9", "16", "23"]);
});

test("the tables sit in keyboard-focusable scroll groups named by their captions", async () => {
  renderPanel();
  await limbs();
  for (const name of ["Beş uzuv ve değişim saatleri", "Açık haritanın Ay'ına göre", "Hora tablosu"]) {
    const group = screen.getByRole("group", { name });
    expect(group).toHaveAttribute("tabindex", "0");
    expect(within(group).getByRole("table", { name })).toBeInTheDocument();
  }
  // Abhijit çubuğu yalnızca çizimdir; saatleri metin olarak da yazılıdır.
  expect(document.querySelector(".day-strip-abhijit")).toHaveAttribute("aria-hidden", "true");
});

test("without an open chart the panel says so and asks for nothing", async () => {
  renderPanel({ place: "chart" }, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  await act(() => Promise.resolve());
  expect(api.dayPanchang).not.toHaveBeenCalled();
});

test("every flag, nature and tara has a name in three languages; classical names are the same in all", () => {
  for (const lang of [tr, en, fr]) {
    expect(Object.keys(lang.muhurta.flag).sort()).toEqual([...FX.muhurta_ids.flags].sort());
    expect(Object.keys(lang.muhurta.nature).sort()).toEqual([...FX.muhurta_ids.nakshatra_natures].sort());
    expect(Object.keys(lang.muhurtaDay.mark).sort()).toEqual(["abhijit", "gulika", "rahu", "yamaganda"]);
    // Şeritteki kısa yazılar ve klasik adlar üç dilde aynıdır (K11).
    expect(lang.muhurtaDay.mark).toEqual(tr.muhurtaDay.mark);
    for (const id of ["Vyatipata", "Vaidhriti", "Yamaganda", "Gulika", "Panchaka"] as const) expect(lang.muhurta.flag[id]).toBe(id);
    expect([lang.muhurta.tara, lang.muhurta.chandra]).toEqual(["Tara bala", "Chandra bala"]);
  }
  expect(Object.keys(NAKSHATRA_NATURE_NAMES).sort()).toEqual([...FX.muhurta_ids.nakshatra_natures].sort());
  expect(TARA_NAMES).toHaveLength(9);
  expect(FX.muhurta_ids.tithi_groups).toEqual(["Nanda", "Bhadra", "Jaya", "Rikta", "Purna"]);
});
````

````diff
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index 7faade4..1ccad01 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -43,6 +43,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "bhava_bala",
     "yogas",
     "compat",
+    "muhurta_day",
   ]);
   expect(isKnownPanelKind("chart")).toBe(true);
   expect(isKnownPanelKind("drishti")).toBe(true);
@@ -50,7 +51,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas", "compat"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas", "compat", "muhurta_day"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
diff --git a/app/ui/modules/useModule.test.tsx b/app/ui/modules/useModule.test.tsx
index 0aaf99c..58d6694 100644
--- a/app/ui/modules/useModule.test.tsx
+++ b/app/ui/modules/useModule.test.tsx
@@ -3,7 +3,7 @@ import { StrictMode } from "react";
 import { expect, test, vi } from "vitest";
 import type { Result } from "../api";
 import type { CommandError } from "../bindings";
-import { useModule } from "./useModule";
+import { useLastData, useModule } from "./useModule";
 
 const failure: CommandError = { code: "calc", params: { detail: "x" } };
 
@@ -112,3 +112,42 @@ test("a fetch that rejects ends as an error instead of loading forever, and retr
   await act(async () => screen.getByRole("button", { name: "retry" }).click());
   expect(state()).toBe("B");
 });
+
+function KeptProbe({ k, scope, fetch }: { k: string | null; scope: string | null; fetch: () => Promise<Result<string>> }) {
+  const m = useModule(k, fetch);
+  const shown = useLastData(m, scope);
+  return <span data-testid="kept">{`${shown ?? "none"}${m.loading ? " (loading)" : ""}${m.error ? " (error)" : ""}`}</span>;
+}
+
+test("the last data stays on screen while a new key loads, but not across scopes or after an error", async () => {
+  const kept = () => screen.getByTestId("kept").textContent;
+  const [a, b, c, d] = [deferred<string>(), deferred<string>(), deferred<string>(), deferred<string>()];
+  const fetch = vi
+    .fn<() => Promise<Result<string>>>()
+    .mockReturnValueOnce(a.promise)
+    .mockReturnValueOnce(b.promise)
+    .mockReturnValueOnce(c.promise)
+    .mockReturnValueOnce(d.promise);
+  const { rerender } = render(<KeptProbe k="a" scope="panel" fetch={fetch} />);
+  expect(kept()).toBe("none (loading)");
+  await act(async () => a.resolve({ status: "ok", data: "A" }));
+  expect(kept()).toBe("A");
+  // Yeni anahtar: yanıt gelene dek önceki veri gösterilir.
+  rerender(<KeptProbe k="b" scope="panel" fetch={fetch} />);
+  expect(kept()).toBe("A (loading)");
+  await act(async () => b.resolve({ status: "ok", data: "B" }));
+  expect(kept()).toBe("B");
+  // Hata eski veriyi göstermez.
+  rerender(<KeptProbe k="c" scope="panel" fetch={fetch} />);
+  expect(kept()).toBe("B (loading)");
+  await act(async () => c.resolve({ status: "error", error: failure }));
+  expect(kept()).toBe("none (error)");
+  // Başka bir kapsam (başka harita) öncekinin verisini göstermez.
+  rerender(<KeptProbe k="d" scope="other" fetch={fetch} />);
+  expect(kept()).toBe("none (loading)");
+  await act(async () => d.resolve({ status: "ok", data: "D" }));
+  expect(kept()).toBe("D");
+  // Anahtar ve kapsam yokken hiçbir şey gösterilmez.
+  rerender(<KeptProbe k={null} scope={null} fetch={fetch} />);
+  expect(kept()).toBe("none");
+});
diff --git a/app/ui/panels/format.test.ts b/app/ui/panels/format.test.ts
index f197cf4..d9509d0 100644
--- a/app/ui/panels/format.test.ts
+++ b/app/ui/panels/format.test.ts
@@ -1,4 +1,4 @@
-import { formatDegree, formatJdMinute, formatSignedDegree, formatSpeed } from "./format";
+import { formatDegree, formatJdClock, formatJdDate, formatJdMinute, formatJdSpan, formatSignedDegree, formatSpeed } from "./format";
 
 test("formatDegree writes degrees and zero-padded arc minutes, rounding down", () => {
   expect(formatDegree(0)).toBe("0°00′");
@@ -29,3 +29,34 @@ test("a julian day is written as local date and minute at a fixed offset, floore
   expect(formatJdMinute(Number.NaN, 0)).toBe("—");
   expect(formatJdMinute(1e300, 0)).toBe("—");
 });
+
+test("a clock time carries a day offset when it falls on another local day than the base", () => {
+  // JD 2451545.0 = 2000-01-01 12:00 UT; +05:30'da 17:30.
+  expect(formatJdClock(2451545, 19800, 2451545)).toBe("17:30");
+  expect(formatJdClock(2451545, 0, null)).toBe("12:00");
+  // 9 saat sonra: 21:00 UT, +05:30'da ertesi gün 02:30.
+  expect(formatJdClock(2451545.375, 19800, 2451545)).toBe("02:30 (+1)");
+  expect(formatJdClock(2451545.375, 0, 2451545)).toBe("21:00");
+  // Tabandan önceki gün: 2000-01-01 01:00:30 UT, −05:00'da önceki gün 20:00.
+  expect(formatJdClock(2451544.5 + 3630 / 86400, -18000, 2451545)).toBe("20:00 (−1)");
+  expect(formatJdClock(2451547.5, 0, 2451545)).toBe("00:00 (+3)");
+  expect(formatJdClock(null, 0, 2451545)).toBe("—");
+  expect(formatJdClock(Number.NaN, 0, 2451545)).toBe("—");
+});
+
+test("a julian day is written as a local date at a fixed offset", () => {
+  expect(formatJdDate(2451545, 0)).toBe("2000-01-01");
+  expect(formatJdDate(2451545.375, 19800)).toBe("2000-01-02");
+  expect(formatJdDate(null, 0)).toBe("—");
+});
+
+test("a span is written in days, hours and minutes, rounding the minute down", () => {
+  const units = { day: "g", hour: "sa", minute: "dk" };
+  expect(formatJdSpan(10, 10 + 45 / 1440, units)).toBe("45 dk");
+  expect(formatJdSpan(10, 10 + 125 / 1440, units)).toBe("2 sa 05 dk");
+  expect(formatJdSpan(10, 10 + 125.9 / 1440, units)).toBe("2 sa 05 dk");
+  expect(formatJdSpan(10, 13 + 4.5 / 24, units)).toBe("3 g 4 sa");
+  expect(formatJdSpan(10, 10, units)).toBe("0 dk");
+  expect(formatJdSpan(10, 9, units)).toBe("—");
+  expect(formatJdSpan(null, 9, units)).toBe("—");
+});
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index 9729df5..d73e465 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -18,6 +18,7 @@ import { CompatPanel } from "./CompatPanel";
 import { DrishtiPanel } from "./DrishtiPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
 import { JaiminiKarakasPanel } from "./JaiminiKarakasPanel";
+import { MuhurtaDayPanel } from "./MuhurtaDayPanel";
 import { PanchangPanel } from "./PanchangPanel";
 import { panelDefinition, PANEL_REGISTRY } from "./registry";
 import { ShadbalaPanel } from "./ShadbalaPanel";
@@ -54,6 +55,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.bhava_bala.component).toBe(BhavaBalaPanel);
   expect(PANEL_REGISTRY.yogas.component).toBe(YogasPanel);
   expect(PANEL_REGISTRY.compat.component).toBe(CompatPanel);
+  expect(PANEL_REGISTRY.muhurta_day.component).toBe(MuhurtaDayPanel);
   // Kataloğun her türü uygulanmıştır: yer tutucu yoktur.
   for (const kind of PANEL_KINDS) expect(PANEL_REGISTRY[kind].component, kind).not.toBe(UnavailablePanel);
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
diff --git a/app/ui/transit/store.test.tsx b/app/ui/transit/store.test.tsx
index 8654da0..723ade8 100644
--- a/app/ui/transit/store.test.tsx
+++ b/app/ui/transit/store.test.tsx
@@ -84,7 +84,10 @@ test("choosing now again while already at now asks Rust again, so a stale now ca
   await waitFor(() => expect(screen.getByTestId("utc")).toHaveTextContent(fixture().transit.utc));
   expect(api.transitsAt).toHaveBeenCalledTimes(1);
   vi.mocked(api.transitsAt).mockResolvedValueOnce(ok({ ...fixture().transit, utc: "2030-01-01T00:00:00Z" }));
+  const epoch = control.epoch;
   act(() => control.setAt("Now"));
+  // Anı kendisi okuyan bileşenler (gün panchangı) bu sayaçla yeniden okur.
+  expect(control.epoch).toBe(epoch + 1);
   await waitFor(() => expect(screen.getByTestId("utc")).toHaveTextContent("2030-01-01T00:00:00Z"));
   expect(api.transitsAt).toHaveBeenCalledTimes(2);
   expect(vi.mocked(api.transitsAt).mock.calls[1][0]).toBe("Now");
````


- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
pnpm --dir app exec vitest run ui/transit/store.test.tsx ui/modules/useModule.test.tsx ui/modules/muhurta.test.ts ui/panels/format.test.ts ui/panels/HomePlaceDialog.test.tsx ui/panels/MuhurtaDayPanel.test.tsx ui/layout/model.test.ts ui/panels/panels.test.tsx 2>&1 | grep -E "Test Files|Tests "
```
Expected: `Test Files  8 failed (8)`, `Tests  7 failed | 35 passed (42)`: üç yeni dosya ve `panels.test.tsx` içe aktarım hatasıyla düşer; diğer dört dosyada yedi test düşer.

- [ ] **Step 3: Ortak parçalar: transit sayacı, son veri, biçimler, adlar, yer ve koşul yardımcıları, kanca**

````diff
diff --git a/app/ui/modules/useModule.ts b/app/ui/modules/useModule.ts
index 6856031..eac44d8 100644
--- a/app/ui/modules/useModule.ts
+++ b/app/ui/modules/useModule.ts
@@ -51,3 +51,16 @@ export function useModule<T>(key: string | null, fetch: () => Promise<Result<T>>
     retry,
   };
 }
+
+/**
+ * Yeniden okuma sürerken gösterilecek veri: `state.data` yoksa ve istek sürüyorsa aynı `scope`
+ * için son okunan veri. Böylece içerik (liste, tablo, odaktaki denetim) yeni yanıt gelene dek
+ * yerinde durur; hata ya da başka bir `scope` eski veriyi göstermez. `scope === null` unutur.
+ */
+export function useLastData<T>(state: ModuleState<T>, scope: string | null): T | null {
+  const last = useRef<{ scope: string; data: T } | null>(null);
+  if (scope === null || (last.current !== null && last.current.scope !== scope)) last.current = null;
+  if (scope !== null && state.data !== null) last.current = { scope, data: state.data };
+  if (state.data !== null) return state.data;
+  return state.loading ? (last.current?.data ?? null) : null;
+}
diff --git a/app/ui/panels/format.ts b/app/ui/panels/format.ts
index 0bad911..0c797de 100644
--- a/app/ui/panels/format.ts
+++ b/app/ui/panels/format.ts
@@ -34,3 +34,36 @@ export function formatJdMinute(jd: number | null, offsetSecs: number): string {
 export function formatSpeed(degPerDay: number): string {
   return `${degPerDay < 0 ? "−" : "+"}${Math.abs(degPerDay).toFixed(3)}`;
 }
+
+/**
+ * JD (UT) → sabit ofsette saat, "SS:DD" (dakika aşağı yuvarlanır). An, `baseJd`'nin o ofsetteki
+ * takvim gününde değilse gün farkı eklenir: "06:57 (+1)", "23:10 (−1)". Yalnızca gösterim.
+ */
+export function formatJdClock(jd: number | null, offsetSecs: number, baseJd: number | null): string {
+  const full = formatJdMinute(jd, offsetSecs);
+  if (full === "—" || jd === null) return full;
+  const clock = full.slice(-5);
+  if (baseJd === null || !Number.isFinite(baseJd)) return clock;
+  const localDay = (x: number) => Math.floor((x - UNIX_EPOCH_JD) + offsetSecs / 86_400);
+  const days = localDay(jd) - localDay(baseJd);
+  if (days === 0) return clock;
+  return `${clock} (${days > 0 ? "+" : "−"}${Math.abs(days)})`;
+}
+
+/** JD (UT) → sabit ofsette takvim günü, "YYYY-AA-GG". */
+export function formatJdDate(jd: number | null, offsetSecs: number): string {
+  const full = formatJdMinute(jd, offsetSecs);
+  return full === "—" ? full : full.slice(0, -6);
+}
+
+/** İki an arasındaki süre: "2 sa 05 dk", "45 dk"; bir günden uzunsa "3 g 4 sa". Dakika aşağı yuvarlanır. */
+export function formatJdSpan(startJd: number | null, endJd: number | null, units: { day: string; hour: string; minute: string }): string {
+  if (startJd === null || endJd === null || !Number.isFinite(startJd) || !Number.isFinite(endJd) || endJd < startJd) return "—";
+  const minutes = Math.floor((endJd - startJd) * 1440 + 1e-6);
+  const days = Math.floor(minutes / 1440);
+  const hours = Math.floor((minutes % 1440) / 60);
+  const rest = minutes % 60;
+  if (days > 0) return `${days} ${units.day} ${hours} ${units.hour}`;
+  if (hours > 0) return `${hours} ${units.hour} ${String(rest).padStart(2, "0")} ${units.minute}`;
+  return `${rest} ${units.minute}`;
+}
diff --git a/app/ui/panels/labels.ts b/app/ui/panels/labels.ts
index e963dc3..9f31ab9 100644
--- a/app/ui/panels/labels.ts
+++ b/app/ui/panels/labels.ts
@@ -152,3 +152,27 @@ export const KARAKA_ROLES: Record<KarakaRole, { short: string; name: string }> =
 export function arudhaLabel(house: number): string {
   return house === 1 ? "AL" : house === 12 ? "UL" : `A${house}`;
 }
+
+/** Dokuz taranın adı, 1 = Janma … 9 = Parama Mitra (Sanskritçe terim; dile göre değişmez). */
+export const TARA_NAMES = ["Janma", "Sampat", "Vipat", "Kshema", "Pratyak", "Sadhana", "Naidhana", "Mitra", "Parama Mitra"] as const;
+
+/** 1 tabanlı tara numarasının adı; aralık dışıysa numaranın kendisi. */
+export function taraName(n: number): string {
+  return TARA_NAMES[n - 1] ?? String(n);
+}
+
+/** Nakshatra doğalarının Sanskritçe adı (dile göre değişmez); kısa çevirisi i18n'dedir. */
+export const NAKSHATRA_NATURE_NAMES = {
+  Fixed: "Dhruva",
+  Movable: "Chara",
+  Fierce: "Ugra",
+  Mixed: "Mishra",
+  Swift: "Kshipra",
+  Soft: "Mridu",
+  Sharp: "Tikshna",
+} as const;
+
+/** Tithinin (1–30) paksha kısaltmasıyla kısa yazımı: "S9" (Shukla 9), "K14" (Krishna 14). */
+export function tithiShort(no: number): string {
+  return `${no <= 15 ? "S" : "K"}${tithiInPaksha(no)}`;
+}
diff --git a/app/ui/transit/store.tsx b/app/ui/transit/store.tsx
index 3b6a67d..f132252 100644
--- a/app/ui/transit/store.tsx
+++ b/app/ui/transit/store.tsx
@@ -12,6 +12,11 @@ export interface Transit {
   view: TransitView | null;
   error: CommandError | null;
   loading: boolean;
+  /**
+   * "Şimdi" her seçildiğinde ve her yeniden denemede artar. Anı kendisi okuyan bileşenler (gün
+   * panchangı) bunu anahtarlarına katar: an "şimdi" olarak dursa da yeniden okurlar.
+   */
+  epoch: number;
   setAt(at: TransitAt): void;
   retry(): void;
 }
@@ -76,11 +81,12 @@ export function TransitProvider({ children }: { children: ReactNode }) {
       view: mine?.view ?? null,
       error: mine && mine.key === key ? mine.error : null,
       loading: key !== null && mine?.key !== key,
+      epoch: nonce,
       setAt,
       retry,
       register,
     }),
-    [at, mine, key, setAt, retry, register],
+    [at, mine, key, nonce, setAt, retry, register],
   );
   return <TransitContext.Provider value={value}>{children}</TransitContext.Provider>;
 }
````

**Yeni dosya:** `app/ui/modules/muhurta.ts`

````ts
// Muhurta panellerinin ortak girdileri (ileri modüller spec'i §10): hesabın yapılacağı yer, doğum
// Ay'ı ve süzgeç koşulları. Burada hesap yoktur (C4): değerler açık haritadan, tercihlerden ve
// panel ayarlarından okunur. Yer ve doğum Ay'ı kişisel veridir: loglanmaz.
import type { AppPrefs, HomePlace, MuhurtaCriteria, MuhurtaFlag, MuhurtaPlace, NatalMoon, OpenedClient, PlaceHit, Sign, TzChoice, Vara } from "../bindings";
import { SIGNS } from "../chart/adapter";

/** Yer seçimi: bulunulan yer (tercihlerden), açık haritanın doğum yeri ya da panele özel bir yer. */
export const PLACE_CHOICES = ["home", "chart", "custom"] as const;
export type PlaceChoice = (typeof PLACE_CHOICES)[number];

/** Rust `Vara::ALL`. */
export const VARAS = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"] as const satisfies readonly Vara[];
/** Rust `MuhurtaFlag::ALL`. */
export const MUHURTA_FLAGS = [
  "Rikta",
  "Vishti",
  "Vyatipata",
  "Vaidhriti",
  "RahuSegment",
  "Yamaganda",
  "Gulika",
  "Panchaka",
  "JupiterCombust",
  "VenusCombust",
] as const satisfies readonly MuhurtaFlag[];
/** Rust `MUHURTA_MAX_MIN_MINUTES`: "en kısa süre" alanının üst sınırı. */
export const MAX_MIN_MINUTES = 10_080;

/** Panelin yer seçimi; tanınmayan değer ve eksik anahtar bulunulan yerdir. */
export function placeChoice(settings: Record<string, unknown>): PlaceChoice {
  return (PLACE_CHOICES as readonly unknown[]).includes(settings.place) ? (settings.place as PlaceChoice) : "home";
}

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

function inRange(v: unknown, limit: number): v is number {
  return typeof v === "number" && Number.isFinite(v) && Math.abs(v) <= limit;
}

function parseTz(raw: unknown): TzChoice | null {
  if (raw === "Lmt") return "Lmt";
  if (!isRecord(raw)) return null;
  if (typeof raw.Iana === "string" && raw.Iana.trim() !== "") return { Iana: raw.Iana };
  if (typeof raw.Fixed === "number" && Number.isInteger(raw.Fixed)) return { Fixed: raw.Fixed };
  return null;
}

/** Panel ayarında saklanan özel yerin toleranslı okuması; okunamıyorsa `null`. */
export function parseHomePlace(raw: unknown): HomePlace | null {
  if (!isRecord(raw) || !isRecord(raw.place)) return null;
  const { name, country, admin1, lat, lon, geoname_id } = raw.place;
  const tz = parseTz(raw.tz);
  if (tz === null || !inRange(lat, 90) || !inRange(lon, 180)) return null;
  const text = (v: unknown) => (typeof v === "string" ? v : "");
  return {
    place: { name: text(name), country: text(country), admin1: text(admin1), lat, lon, geoname_id: typeof geoname_id === "number" ? geoname_id : null },
    tz,
  };
}

/** Atlas sonucu → yer; koordinatı olmayan sonuç `null`. */
export function hitPlace(hit: PlaceHit): HomePlace | null {
  if (hit.lat === null || hit.lon === null) return null;
  return {
    place: { name: hit.name, country: hit.country, admin1: hit.admin1, lat: hit.lat, lon: hit.lon, geoname_id: hit.geoname_id },
    tz: { Iana: hit.timezone },
  };
}

/** Açık haritanın doğum yeri ve doğum verisindeki saat dilimi. */
export function chartPlace(opened: OpenedClient): HomePlace {
  const { place, tz } = opened.chart_record.birth;
  return { place, tz };
}

/** Seçime göre yer; bulunulan yer seçilmemişse ya da özel yer girilmemişse `null`. */
export function resolvePlace(choice: PlaceChoice, opened: OpenedClient, prefs: AppPrefs | null, custom: HomePlace | null): HomePlace | null {
  if (choice === "chart") return chartPlace(opened);
  if (choice === "custom") return custom;
  return prefs?.home_place ?? null;
}

/** Komuta giden yer: koordinat ve saat dilimi; ad gönderilmez. */
export function muhurtaPlace(home: HomePlace): MuhurtaPlace {
  return { lat: home.place.lat, lon: home.place.lon, tz: home.tz };
}

/** Yerin görünen adı; adı yoksa (elle girilen koordinat) koordinatları. */
export function placeLabel(home: HomePlace): string {
  const { name, admin1, country, lat, lon } = home.place;
  const parts = [name, admin1, country].map((s) => s.trim()).filter((s, i, all) => s !== "" && all.indexOf(s) === i);
  if (parts.length > 0) return parts.join(", ");
  return `${(lat ?? 0).toFixed(2)}°, ${(lon ?? 0).toFixed(2)}°`;
}

/** Doğum Ay'ı: açık haritanın Ay'ının nakshatrası ve rasi burcu (hesaplanmış haritadan okunur). */
export function natalMoon(opened: OpenedClient): NatalMoon {
  const moon = opened.chart.grahas.find((g) => g.graha === "Moon") ?? opened.chart.grahas[1];
  return { nakshatra: moon.nakshatra, sign: moon.sign };
}

export const EMPTY_CRITERIA: MuhurtaCriteria = {
  weekdays: [],
  tithis: [],
  nakshatras: [],
  lagna_signs: [],
  daytime_only: false,
  min_minutes: 0,
  exclude: [],
  require_tara: false,
  require_chandra: false,
};

function members<T>(raw: unknown, allowed: readonly T[]): T[] {
  if (!Array.isArray(raw)) return [];
  return allowed.filter((a) => raw.includes(a));
}

function numbers(raw: unknown, max: number): number[] {
  if (!Array.isArray(raw)) return [];
  const out: number[] = [];
  for (let n = 1; n <= max; n++) if (raw.includes(n)) out.push(n);
  return out;
}

/**
 * Panel ayarında saklanan koşulların toleranslı okuması: tanınmayan değerler atılır, kümeler
 * Rust'ın kanonik sırasına girer, eksik alanlar "kısıt yok"tur.
 */
export function parseCriteria(raw: unknown): MuhurtaCriteria {
  if (!isRecord(raw)) return { ...EMPTY_CRITERIA };
  const minutes = raw.min_minutes;
  return {
    weekdays: members<Vara>(raw.weekdays, VARAS),
    tithis: numbers(raw.tithis, 30),
    nakshatras: numbers(raw.nakshatras, 27),
    lagna_signs: members<Sign>(raw.lagna_signs, SIGNS),
    daytime_only: raw.daytime_only === true,
    min_minutes: typeof minutes === "number" && Number.isInteger(minutes) && minutes >= 0 ? Math.min(minutes, MAX_MIN_MINUTES) : 0,
    exclude: members<MuhurtaFlag>(raw.exclude, MUHURTA_FLAGS),
    require_tara: raw.require_tara === true,
    require_chandra: raw.require_chandra === true,
  };
}

/** Kümede `value` varsa çıkarır, yoksa ekler; sonuç `order` sırasındadır. */
export function toggleMember<T>(list: readonly T[], value: T, order: readonly T[]): T[] {
  const next = list.includes(value) ? list.filter((v) => v !== value) : [...list, value];
  return order.filter((o) => next.includes(o));
}
````

**Yeni dosya:** `app/ui/modules/useDayPanchang.ts`

````ts
// Gün panchangını (`day_panchang`) isteyen kanca (ileri modüller spec'i §10). Sonuç saklanmaz ve
// arayüzde hesap yoktur (K1, C4); panel açıkken istenir. Gün, verilen anı içeren gündür: "şimdi"
// Rust'ta okunur. Yeni bir gün okunurken aynı panelin son günü gösterilmeyi sürdürür (içerik ve
// odak yerinde durur). Yer ve doğum Ay'ı kişisel veridir: anahtar ve girdiler loglanmaz.
import { api } from "../api";
import type { DayPanchangView, HomePlace, OpenedClient, TransitAt } from "../bindings";
import { muhurtaPlace, natalMoon } from "./muhurta";
import { useLastData, useModule, type ModuleState } from "./useModule";

export interface DayPanchangState extends ModuleState<DayPanchangView> {
  /** Gösterilecek gün: yeni yanıt gelene dek bir önceki. */
  shown: DayPanchangView | null;
}

/**
 * `place === null`: yer seçilmemiş, komut çağrılmaz. `epoch` aynı anın yeniden okunmasını ister
 * ("Şimdi" yeniden seçildi). `scope` panelin kimliğidir.
 */
export function useDayPanchang(opened: OpenedClient, place: HomePlace | null, at: TransitAt, epoch: number, scope: string): DayPanchangState {
  const settings = opened.chart.meta.settings;
  const natal = natalMoon(opened);
  const key = place !== null ? JSON.stringify([opened.chart_record.id, muhurtaPlace(place), at, epoch, settings, natal]) : null;
  const state = useModule(key, () => api.dayPanchang(at, muhurtaPlace(place as HomePlace), settings, natal));
  const shown = useLastData(state, key === null ? null : `${opened.chart_record.id}:${scope}`);
  return { ...state, shown };
}
````


- [ ] **Step 4: Panel türü**

````diff
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index 9aa4868..9dba4e7 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -1,7 +1,7 @@
 // Pano düzeni veri modeli (spec §3.2). Kalıcı biçim `ViewState.panel_layout` ile aynıdır;
 // M4'teki store komutu gelene kadar localStorage'da tutulur (bkz. persistence.ts).
 
-/** Panel kataloğu: v1 (UI spec §3.3) ve ileri modül panelleri (ileri modüller spec'i §4, §5, §7, §8, §9). */
+/** Panel kataloğu: v1 (UI spec §3.3) ve ileri modül panelleri (ileri modüller spec'i §4, §5, §7, §8, §9, §10). */
 export const PANEL_KINDS = [
   "chart",
   "graha_table",
@@ -20,6 +20,7 @@ export const PANEL_KINDS = [
   "bhava_bala",
   "yogas",
   "compat",
+  "muhurta_day",
 ] as const;
 export type KnownPanelKind = (typeof PANEL_KINDS)[number];
 
@@ -42,6 +43,8 @@ export const MODULE_PANELS = [
   { kind: "yogas", w: 7, h: 14 },
   // Uyum (Plan 19).
   { kind: "compat", w: 7, h: 16 },
+  // Muhurta (Plan 20).
+  { kind: "muhurta_day", w: 7, h: 18 },
 ] as const satisfies readonly { kind: KnownPanelKind; w: number; h: number }[];
 export type ModulePanelKind = (typeof MODULE_PANELS)[number]["kind"];
 
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index faacd6a..ae97960 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -14,6 +14,7 @@ import { TransitTablePanel } from "../transit/TransitTablePanel";
 import { DrishtiPanel } from "./DrishtiPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
 import { JaiminiKarakasPanel } from "./JaiminiKarakasPanel";
+import { MuhurtaDayPanel } from "./MuhurtaDayPanel";
 import { PanchangPanel } from "./PanchangPanel";
 import { ShadbalaPanel } from "./ShadbalaPanel";
 import type { PanelDefinition } from "./types";
@@ -43,6 +44,8 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   yogas: { titleKey: "panel.yogas", component: YogasPanel },
   // Uyum (Plan 19).
   compat: { titleKey: "panel.compat", component: CompatPanel },
+  // Muhurta (Plan 20).
+  muhurta_day: { titleKey: "panel.muhurta_day", component: MuhurtaDayPanel },
 };
 
 const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };
````


- [ ] **Step 5: Bulunulan yer penceresi ve yer seçimi**

**Yeni dosya:** `app/ui/panels/HomePlaceDialog.tsx`

````tsx
// Bulunulan yer penceresi (ileri modüller spec'i §10): `AppPrefs.home_place`'i atlastan seçer,
// açık haritanın doğum yerinden alır ya da kaldırır. Gün panchangı ve muhurta araması bunu
// varsayılan yer olarak kullanır. Konum servisi kullanılmaz; yer kişisel veridir ve loglanmaz.
import { useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError, HomePlace, OpenedClient } from "../bindings";
import { PlaceSearch } from "../birth/PlaceSearch";
import { errorMessage } from "../i18n/errors";
import { chartPlace, hitPlace, placeLabel } from "../modules/muhurta";
import { usePrefs } from "../settings/prefs";
import { Dialog } from "../shell/Dialog";

/** Saat diliminin kısa yazımı: IANA kimliği, sabit ofset (saniye) ya da yerel ortalama saat. */
export function tzLabel(home: HomePlace, lmt: string): string {
  if (home.tz === "Lmt") return lmt;
  // `{ Fixed } & { Iana?: never }` biçiminde `in` daraltmaz; alan değerine bakılır.
  if (home.tz.Iana !== undefined) return home.tz.Iana;
  const secs = home.tz.Fixed;
  const abs = Math.abs(secs);
  const two = (n: number) => String(n).padStart(2, "0");
  return `UTC${secs < 0 ? "−" : "+"}${two(Math.floor(abs / 3600))}:${two(Math.floor((abs % 3600) / 60))}`;
}

export function HomePlaceDialog({ opened, onClose }: { opened: OpenedClient; onClose(): void }) {
  const { t, i18n } = useTranslation();
  const { prefs, save } = usePrefs();
  const id = useId();
  const [text, setText] = useState("");
  const [error, setError] = useState<CommandError | null>(null);
  const [saving, setSaving] = useState(false);
  const useChart = useRef<HTMLButtonElement>(null);
  const home = prefs?.home_place ?? null;
  const birth = chartPlace(opened);

  const store = async (next: HomePlace | null) => {
    if (prefs === null || saving) return;
    setSaving(true);
    const failed = await save({ ...prefs, home_place: next });
    setSaving(false);
    setError(failed);
    if (failed === null) setText("");
    // "Yeri kaldır" düğmesi yer kalkınca DOM'dan gider: odak pencerede bir sonraki eyleme taşınır.
    if (failed === null && next === null) useChart.current?.focus();
  };

  return (
    <Dialog title={t("muhurta.home.title")} onClose={onClose}>
      <div className="home-place">
        <p className="home-place-current" role="status">
          {home ? t("muhurta.home.current", { place: placeLabel(home), zone: tzLabel(home, t("muhurta.home.lmt")) }) : t("muhurta.home.unset")}
        </p>
        <p className="field-hint">{t("muhurta.home.hint")}</p>
        {prefs === null ? (
          <p className="panel-notice" role="note">
            {t("muhurta.home.noPrefs")}
          </p>
        ) : (
          <>
            <label htmlFor={`${id}-search`}>{t("muhurta.home.search")}</label>
            <PlaceSearch
              inputId={`${id}-search`}
              value={text}
              onChange={setText}
              onSelect={(hit) => {
                const place = hitPlace(hit);
                if (place) void store(place);
              }}
            />
            <div className="home-place-actions">
              <button ref={useChart} type="button" data-autofocus onClick={() => void store(birth)}>
                {t("muhurta.home.useChart", { place: placeLabel(birth) })}
              </button>
              {home && (
                <button type="button" onClick={() => void store(null)}>
                  {t("muhurta.home.remove")}
                </button>
              )}
            </div>
          </>
        )}
        {error && (
          <p className="panel-error" role="alert">
            {errorMessage(t, i18n, error)}
          </p>
        )}
        <div className="dialog-actions">
          <button type="button" onClick={onClose}>
            {t("muhurta.home.close")}
          </button>
        </div>
      </div>
    </Dialog>
  );
}
````

**Yeni dosya:** `app/ui/panels/MuhurtaPlace.tsx`

````tsx
// Muhurta panellerinin yer seçimi (ileri modüller spec'i §10 "Bulunulan yer"): bulunulan yer
// (tercihlerden), açık haritanın doğum yeri ya da panele özel bir yer (atlastan). Seçim panelin
// ayarıdır ve düzenle birlikte saklanır; bulunulan yerin kendisi tercihlerdedir. Burada hesap
// yoktur (C4).
import { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import type { HomePlace, OpenedClient } from "../bindings";
import { PlaceSearch } from "../birth/PlaceSearch";
import type { Panel } from "../layout/model";
import { hitPlace, parseHomePlace, PLACE_CHOICES, placeChoice, placeLabel, resolvePlace, type PlaceChoice } from "../modules/muhurta";
import { usePrefs } from "../settings/prefs";
import { useLayoutStore } from "../view/store";
import { HomePlaceDialog } from "./HomePlaceDialog";

export interface PanelPlace {
  choice: PlaceChoice;
  /** Seçilen yer; bulunulan yer seçilmemişse ya da özel yer girilmemişse `null`. */
  place: HomePlace | null;
}

/** Panelin ayarından ve tercihlerden yer. */
export function usePanelPlace(panel: Panel, opened: OpenedClient): PanelPlace {
  const { prefs } = usePrefs();
  const choice = placeChoice(panel.settings);
  return { choice, place: resolvePlace(choice, opened, prefs, parseHomePlace(panel.settings.custom)) };
}

/** Yer seçicisi ve seçime bağlı denetimler: özel yer araması ya da bulunulan yer penceresi. */
export function PlaceControls({ panel, opened, at }: { panel: Panel; opened: OpenedClient; at: PanelPlace }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  const [text, setText] = useState("");
  const [editingHome, setEditingHome] = useState(false);
  return (
    <>
      <label htmlFor={`${id}-place`}>{t("muhurta.place.label")}</label>
      <select id={`${id}-place`} value={at.choice} onChange={(e) => setPanelSettings(panel.id, { place: e.target.value })}>
        {PLACE_CHOICES.map((c) => (
          <option key={c} value={c}>
            {t(`muhurta.place.${c}`)}
          </option>
        ))}
      </select>
      {at.place && <span className="muhurta-place-name">{placeLabel(at.place)}</span>}
      {at.choice === "home" && (
        <button type="button" onClick={() => setEditingHome(true)}>
          {t(at.place ? "muhurta.place.changeHome" : "muhurta.place.pickHome")}
        </button>
      )}
      {at.choice === "custom" && (
        <span className="muhurta-place-search">
          <label htmlFor={`${id}-search`}>{t("muhurta.place.search")}</label>
          <PlaceSearch
            inputId={`${id}-search`}
            value={text}
            onChange={setText}
            onSelect={(hit) => {
              const place = hitPlace(hit);
              if (place === null) return;
              setText("");
              setPanelSettings(panel.id, { custom: place });
            }}
          />
        </span>
      )}
      {editingHome && <HomePlaceDialog opened={opened} onClose={() => setEditingHome(false)} />}
    </>
  );
}

/** Yer seçilmemişken panelin gösterdiği açıklama. */
export function PlaceMissing({ choice }: { choice: PlaceChoice }) {
  const { t } = useTranslation();
  return (
    <p className="panel-notice" role="note">
      {t(choice === "custom" ? "muhurta.place.missingCustom" : "muhurta.place.missingHome")}
    </p>
  );
}
````


- [ ] **Step 6: Panel ve stiller**

**Yeni dosya:** `app/ui/panels/MuhurtaDayPanel.tsx`

````tsx
// Gün panchangı paneli (ileri modüller spec'i §10): seçilen anı içeren günün (gün doğumundan
// sonraki gün doğumuna) şeridi (sekiz dilim, işaretli üçü ve Abhijit), beş uzvun değişim saatli
// listesi, işaretler, hora tablosu ve açık haritanın Ay'ına göre tara ve chandra bala. Değerler
// `day_panchang`'tan gelir; burada hesap yoktur (C4). "Uygun" ya da "uygunsuz" hükmü ve
// açıklama metni yoktur; adlar etikettir. Saatler seçilen yerin saatiyle gösterilir.
import { useId, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { DayPanchang, DayPanchangView, Interval, OpenedClient, TransitAt } from "../bindings";
import { useHighlight } from "../highlight/store";
import type { Panel } from "../layout/model";
import { useDayPanchang } from "../modules/useDayPanchang";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useTransitControl } from "../transit/store";
import { TransitDate } from "../transit/TransitDate";
import "../transit/transit.css";
import { useLayoutStore } from "../view/store";
import { Approx } from "./Approx";
import { BalaScroll } from "./bala";
import { formatJdClock, formatJdDate } from "./format";
import { NAKSHATRA_NATURE_NAMES, nakshatraName, nityaYogaName, taraName, tithiInPaksha, tithiName, VARA_NAMES } from "./labels";
import { PlaceControls, PlaceMissing, usePanelPlace } from "./MuhurtaPlace";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export const DAY_LINKS = ["transit", "today"] as const;
export type DayLink = (typeof DAY_LINKS)[number];

/** Panelin tarih bağı; tanınmayan değer ve eksik anahtar transit tarihidir. */
export function dayLink(settings: Record<string, unknown>): DayLink {
  return settings.link === "today" ? "today" : "transit";
}

/** Dilimin şeritteki kısa yazısı (renk tek başına anlam taşımaz). */
const MARKS = ["rahu", "yamaganda", "gulika"] as const;

export function MuhurtaDayPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Day panel={panel} opened={opened} />;
}

function Day({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const transit = useTransitControl();
  const id = useId();
  const link = dayLink(panel.settings);
  const place = usePanelPlace(panel, opened);
  const at: TransitAt = link === "today" ? "Now" : transit.at;
  const module = useDayPanchang(opened, place.place, at, transit.epoch, panel.id);
  const phase = usePanelLoadPhase(module.loading);
  const view = module.shown;
  return (
    <div className="module-panel muhurta-day" aria-busy={module.loading && view !== null ? true : undefined}>
      {/* Yer ve tarih bağı yükleme ve hata durumlarında da kullanılabilir. */}
      <div className="module-toolbar">
        <PlaceControls panel={panel} opened={opened} at={place} />
        <label htmlFor={`${id}-link`}>{t("muhurtaDay.linkLabel")}</label>
        <select id={`${id}-link`} value={link} onChange={(e) => setPanelSettings(panel.id, { link: e.target.value })}>
          {DAY_LINKS.map((l) => (
            <option key={l} value={l}>
              {t(`muhurtaDay.link.${l}`)}
            </option>
          ))}
        </select>
      </div>
      {link === "transit" && <TransitDate />}
      {place.place === null ? (
        <PlaceMissing choice={place.choice} />
      ) : module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : view === null ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : (
        <DayBody view={view} unknown={timeUnknown(opened)} />
      )}
    </div>
  );
}

/** Kırpılan uç: aralık günün dışında başlıyor ya da sürüyor. İşaret ve ekran okuyucu metni. */
function Clipped({ on, mark, label, children }: { on: boolean; mark: string; label: string; children: ReactNode }) {
  if (!on) return <>{children}</>;
  return (
    <span>
      {children}{" "}
      <span aria-hidden="true" title={label}>
        {mark}
      </span>
      <span className="visually-hidden">{label}</span>
    </span>
  );
}

function DayBody({ view, unknown }: { view: DayPanchangView; unknown: boolean }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const limbsId = useId();
  const personalId = useId();
  const horasId = useId();
  const day = view.day;
  const clock = (jd: number | null) => formatJdClock(jd, view.offset_secs, day.day_start_jd_ut);
  const span = <T,>(i: Interval<T>) => ({
    start: (
      <Clipped on={i.start_jd_ut === day.day_start_jd_ut} mark="←" label={t("muhurtaDay.fromBefore")}>
        {clock(i.start_jd_ut)}
      </Clipped>
    ),
    end: (
      <Clipped on={i.end_jd_ut === day.day_end_jd_ut} mark="→" label={t("muhurtaDay.goesOn")}>
        {clock(i.end_jd_ut)}
      </Clipped>
    ),
  });
  const limbRow = <T,>(limb: string, i: Interval<T>, index: number, value: ReactNode) => {
    const { start, end } = span(i);
    return (
      <tr key={`${limb}:${index}`} data-limb={limb}>
        <th scope="row">{t(`panchang.${limb}`)}</th>
        <td>{value}</td>
        <td className="num">{start}</td>
        <td className="num">{end}</td>
      </tr>
    );
  };
  const noSunrise = day.warnings.includes("SunriseUnavailable");
  const personal = day.personal;
  return (
    <>
      <p className="muhurta-day-title">
        {t("muhurtaDay.title", { date: formatJdDate(day.day_start_jd_ut, view.offset_secs), vara: VARA_NAMES[day.vara], weekday: t(`panchang.weekday.${day.vara}`) })}
      </p>
      {noSunrise ? (
        <p className="panel-notice muhurta-warning" role="note">
          <span className="badge">
            <span className="badge-mark" aria-hidden="true">
              !
            </span>
            {t("badge.SunriseUnavailable")}
          </span>{" "}
          {t("muhurtaDay.noSunrise", { start: clock(day.day_start_jd_ut) })}
        </p>
      ) : (
        <p className="panel-message">
          {t("muhurtaDay.sun", { sunrise: clock(day.sunrise_jd_ut), sunset: clock(day.sunset_jd_ut), next: clock(day.next_sunrise_jd_ut) })}
        </p>
      )}
      {day.warnings.includes("LowPrecision") && (
        <p className="panel-notice" role="note">
          <strong>{t("badge.LowPrecision")}</strong>: {t("badgeHint.LowPrecision")}
        </p>
      )}
      {day.segments && <Strip day={day} clock={clock} />}
      <BalaScroll labelledBy={limbsId}>
        <table className="module-table muhurta-table">
          <caption id={limbsId}>{t("muhurtaDay.limbs")}</caption>
          <thead>
            <tr>
              <th scope="col">{t("panchang.item")}</th>
              <th scope="col">{t("panchang.value")}</th>
              <th scope="col" className="num">
                {t("muhurtaDay.from")}
              </th>
              <th scope="col" className="num">
                {t("muhurtaDay.to")}
              </th>
            </tr>
          </thead>
          <tbody>
            {day.tithis.map((i, n) =>
              limbRow("tithi", i, n, `${i.value.paksha} ${tithiInPaksha(i.value.no)} · ${tithiName(i.value.no)} · ${t("muhurtaDay.group", { group: i.value.group })}`),
            )}
            {day.nakshatras.map((i, n) =>
              limbRow(
                "nakshatra",
                i,
                n,
                `${nakshatraName(i.value.no)} · ${t("panchang.pada", { pada: i.value.pada })} · ${NAKSHATRA_NATURE_NAMES[i.value.nature]} (${t(`muhurta.nature.${i.value.nature}`)})`,
              ),
            )}
            {day.yogas.map((i, n) => limbRow("yoga", i, n, nityaYogaName(i.value)))}
            {day.karanas.map((i, n) => limbRow("karana", i, n, i.value.name))}
          </tbody>
        </table>
      </BalaScroll>
      {day.flags.length === 0 ? (
        <p className="panel-message">{t("muhurtaDay.noFlags")}</p>
      ) : (
        <ul className="muhurta-flags" aria-label={t("muhurtaDay.flags")}>
          {day.flags.map((f, n) => {
            const { start, end } = span(f);
            return (
              <li key={`${f.value}:${n}`} data-flag={f.value}>
                <span className="muhurta-flag-name">{t(`muhurta.flag.${f.value}`)}</span> {start}–{end}
              </li>
            );
          })}
        </ul>
      )}
      {personal && (
        <BalaScroll labelledBy={personalId}>
          <table className="module-table muhurta-table">
            <caption id={personalId}>
              <Approx on={unknown} label={t("muhurtaDay.natalApprox")}>
                {t("muhurtaDay.personal")}
              </Approx>
            </caption>
            <thead>
              <tr>
                <th scope="col">{t("panchang.item")}</th>
                <th scope="col">{t("panchang.value")}</th>
                <th scope="col">{t("muhurtaDay.favourableColumn")}</th>
                <th scope="col" className="num">
                  {t("muhurtaDay.from")}
                </th>
                <th scope="col" className="num">
                  {t("muhurtaDay.to")}
                </th>
              </tr>
            </thead>
            <tbody>
              {personal.tara.map((i, n) => {
                const { start, end } = span(i);
                return (
                  <tr key={`tara:${n}`} data-personal="tara">
                    <th scope="row">{t("muhurta.tara")}</th>
                    <td>{t("muhurtaDay.taraValue", { number: i.value.number, name: taraName(i.value.number) })}</td>
                    <td>{t(i.value.favourable ? "muhurtaDay.favourable" : "muhurtaDay.notFavourable")}</td>
                    <td className="num">{start}</td>
                    <td className="num">{end}</td>
                  </tr>
                );
              })}
              {personal.chandra.map((i, n) => {
                const { start, end } = span(i);
                return (
                  <tr key={`chandra:${n}`} data-personal="chandra">
                    <th scope="row">{t("muhurta.chandra")}</th>
                    <td>{t("muhurtaDay.chandraValue", { house: i.value.house })}</td>
                    <td>{t(i.value.favourable ? "muhurtaDay.favourable" : "muhurtaDay.notFavourable")}</td>
                    <td className="num">{start}</td>
                    <td className="num">{end}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </BalaScroll>
      )}
      {day.segments && (
        <BalaScroll labelledBy={horasId}>
          <table className="module-table muhurta-table">
            <caption id={horasId}>{t("muhurtaDay.horas")}</caption>
            <thead>
              <tr>
                <th scope="col" className="num">
                  {t("muhurtaDay.horaNo")}
                </th>
                <th scope="col">{t("muhurtaDay.horaLord")}</th>
                <th scope="col" className="num">
                  {t("muhurtaDay.from")}
                </th>
                <th scope="col" className="num">
                  {t("muhurtaDay.to")}
                </th>
              </tr>
            </thead>
            <tbody>
              {day.segments.horas.map((h, n) => (
                <tr key={n} data-hora={n + 1} className={active?.type === "graha" && active.id === h.value ? "is-highlighted" : undefined}>
                  <td className="num">{n + 1}</td>
                  <th scope="row">
                    <PinButton target={{ type: "graha", id: h.value }}>{t(`grahaName.${h.value}`)}</PinButton>
                  </th>
                  <td className="num">{clock(h.start_jd_ut)}</td>
                  <td className="num">{clock(h.end_jd_ut)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </BalaScroll>
      )}
      <p className="panel-message">{t("muhurtaDay.note")}</p>
      <p className="panel-message">{t("muhurta.method", { method: day.method })}</p>
    </>
  );
}

/** Gün şeridi: gündüzün sekiz dilimi, işaretli üçü ve altında Abhijit. */
function Strip({ day, clock }: { day: DayPanchang; clock: (jd: number | null) => string }) {
  const { t } = useTranslation();
  const segments = day.segments;
  if (segments === null) return null;
  const markOf = (no: number) => MARKS.find((m) => segments[m].value === no) ?? null;
  // Abhijit çubuğunun yeri gündüzün içindeki payıdır (yalnızca çizim).
  const daylight = (day.sunset_jd_ut ?? 0) - (day.sunrise_jd_ut ?? 0);
  const share = (jd: number | null) => (daylight > 0 && jd !== null ? Math.min(100, Math.max(0, (((jd - (day.sunrise_jd_ut ?? 0)) / daylight) * 100))) : 0);
  const left = share(segments.abhijit.start_jd_ut);
  const width = share(segments.abhijit.end_jd_ut) - left;
  return (
    <div className="day-strip-box">
      <ol className="day-strip" aria-label={t("muhurtaDay.strip")}>
        {segments.eighths.map((e) => {
          const mark = markOf(e.value);
          return (
            <li key={e.value} data-segment={e.value} data-mark={mark ?? undefined} className={mark ? "is-marked" : undefined}>
              <span className="day-strip-no">{e.value}</span>
              {mark && <strong className="day-strip-mark">{t(`muhurtaDay.mark.${mark}`)}</strong>}
              <span className="day-strip-time">
                {clock(e.start_jd_ut)}–{clock(e.end_jd_ut)}
              </span>
            </li>
          );
        })}
      </ol>
      <div className="day-strip-abhijit" aria-hidden="true">
        <span style={{ marginLeft: `${left}%`, width: `${width}%` }}>{t("muhurtaDay.mark.abhijit")}</span>
      </div>
      <p className="panel-message" data-abhijit>
        {t("muhurtaDay.abhijit", { start: clock(segments.abhijit.start_jd_ut), end: clock(segments.abhijit.end_jd_ut) })}
        {segments.abhijit_weekday_exception && ` ${t("muhurtaDay.abhijitWednesday")}`}
      </p>
    </div>
  );
}
````

````diff
diff --git a/app/ui/panels/panels.css b/app/ui/panels/panels.css
index f02cee8..d6f6188 100644
--- a/app/ui/panels/panels.css
+++ b/app/ui/panels/panels.css
@@ -290,3 +290,108 @@
   border-left: 2px solid var(--warn);
   color: var(--text);
 }
+
+/* Muhurta panelleri (ileri modüller spec'i §10). */
+.muhurta-place-name {
+  color: var(--text);
+  font-weight: var(--fw-medium);
+}
+.muhurta-place-search {
+  display: inline-flex;
+  align-items: center;
+  gap: 6px;
+}
+.muhurta-day-title {
+  margin: 0;
+  font-weight: var(--fw-medium);
+}
+/* Uyarı yalnızca renkle ayrılmaz: kenar çizgisi, rozet ve metin. */
+.muhurta-warning {
+  padding-left: 6px;
+  border-left: 2px solid var(--warn);
+  color: var(--text);
+}
+.muhurta-table caption {
+  text-align: left;
+  padding: 6px 6px 2px;
+  font-weight: var(--fw-medium);
+}
+.day-strip-box {
+  display: flex;
+  flex-direction: column;
+  gap: 2px;
+}
+.day-strip {
+  display: grid;
+  grid-template-columns: repeat(8, minmax(0, 1fr));
+  margin: 0;
+  padding: 0;
+  list-style: none;
+  border: 1px solid var(--line);
+}
+.day-strip li {
+  display: flex;
+  flex-direction: column;
+  align-items: center;
+  gap: 1px;
+  min-width: 0;
+  padding: 2px;
+  border-left: 1px solid var(--line);
+  font-size: var(--fs-table);
+  font-variant-numeric: tabular-nums;
+}
+.day-strip li:first-child {
+  border-left: 0;
+}
+/* İşaretli dilim renkle değil, taramayla ve yazıyla ayrılır. */
+.day-strip li.is-marked {
+  background-image: repeating-linear-gradient(45deg, var(--line) 0 1px, transparent 1px 6px);
+}
+.day-strip-no,
+.day-strip-time {
+  color: var(--text-2);
+}
+.day-strip-time {
+  overflow-wrap: anywhere;
+  text-align: center;
+}
+.day-strip-mark {
+  font-weight: var(--fw-medium);
+  color: var(--ink-strong);
+}
+.day-strip-abhijit {
+  display: flex;
+  font-size: var(--fs-table);
+}
+.day-strip-abhijit span {
+  box-sizing: border-box;
+  border: 1px solid var(--ink);
+  border-top: 0;
+  text-align: center;
+  color: var(--ink-strong);
+  overflow: hidden;
+}
+.muhurta-flags {
+  margin: 4px 0;
+  padding: 0;
+  list-style: none;
+  font-variant-numeric: tabular-nums;
+}
+.muhurta-flag-name {
+  font-weight: var(--fw-medium);
+}
+.home-place {
+  display: flex;
+  flex-direction: column;
+  gap: 6px;
+  min-width: 22em;
+}
+.home-place-current {
+  margin: 0;
+  font-weight: var(--fw-medium);
+}
+.home-place-actions {
+  display: flex;
+  flex-wrap: wrap;
+  gap: 6px;
+}
````


- [ ] **Step 7: Metinler (üç dil)**

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 76cfeff..ed167f8 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -73,6 +73,7 @@ export const en: Messages = {
     bhava_bala: "Bhava bala",
     yogas: "Yogas",
     compat: "Compatibility",
+    muhurta_day: "Day panchang",
     unknown: "Unknown panel",
     loading: "Loading",
     unavailable: "This panel arrives in a later version.",
@@ -373,6 +374,85 @@ export const en: Messages = {
     pickBack: "Back to clients",
     pickCount: "{{n}} options",
   },
+  muhurta: {
+    tara: "Tara bala",
+    chandra: "Chandra bala",
+    method: "Method: {{method}}",
+    place: {
+      label: "Place",
+      home: "Current place",
+      chart: "Birth place of the chart",
+      custom: "Custom place",
+      pickHome: "Choose the current place",
+      changeHome: "Change the current place",
+      search: "Search a custom place",
+      missingHome: "No current place is set. Choose one with “Choose the current place”, or use the birth place of the chart.",
+      missingCustom: "No custom place is set. Pick a place from the search box.",
+    },
+    home: {
+      title: "Current place",
+      current: "Selected place: {{place}} · {{zone}}",
+      unset: "No current place is set.",
+      hint: "The day panchang and the muhurta search use this place by default. No location service is used; the place stays on this computer and is never exported.",
+      noPrefs: "The current place cannot be changed because the preferences could not be read.",
+      search: "Search a place",
+      useChart: "Use the birth place of the open chart ({{place}})",
+      remove: "Remove the place",
+      close: "Close",
+      lmt: "local mean time",
+    },
+    nature: {
+      Fixed: "fixed",
+      Movable: "movable",
+      Fierce: "fierce",
+      Mixed: "mixed",
+      Swift: "swift",
+      Soft: "soft",
+      Sharp: "sharp",
+    },
+    flag: {
+      Rikta: "Rikta tithi",
+      Vishti: "Vishti karana",
+      Vyatipata: "Vyatipata",
+      Vaidhriti: "Vaidhriti",
+      RahuSegment: "Rahu segment",
+      Yamaganda: "Yamaganda",
+      Gulika: "Gulika",
+      Panchaka: "Panchaka",
+      JupiterCombust: "Jupiter combust",
+      VenusCombust: "Venus combust",
+    },
+  },
+  muhurtaDay: {
+    linkLabel: "Date link",
+    link: { transit: "Transit date", today: "Today" },
+    title: "{{date}} · {{vara}} · {{weekday}}",
+    sun: "Sunrise {{sunrise}} · sunset {{sunset}} · next sunrise {{next}}",
+    noSunrise: "The Sun does not rise or set at this place on this day: the daytime segments and the hora table are not computed. The day starts at 06:00 local mean time ({{start}}).",
+    strip: "The eight segments of the daytime",
+    mark: { rahu: "Rahu", yamaganda: "Yama", gulika: "Guli", abhijit: "Abhi" },
+    abhijit: "Abhijit {{start}}–{{end}}",
+    abhijitWednesday: "(Wednesday: some compendia do not use Abhijit on this day.)",
+    limbs: "The five limbs and their changes",
+    from: "From",
+    to: "To",
+    fromBefore: "starts before the day begins",
+    goesOn: "goes on after the day ends",
+    group: "{{group}} group",
+    flags: "Flags",
+    noFlags: "No flag on this day.",
+    personal: "From the Moon of the open chart",
+    natalApprox: "approximate: the birth time is unknown, the natal Moon is for 12:00 noon",
+    favourableColumn: "Set",
+    favourable: "in the favourable set",
+    notFavourable: "not in the favourable set",
+    taraValue: "{{number}} · {{name}}",
+    chandraValue: "house {{house}} from the natal Moon sign",
+    horas: "Hora table",
+    horaNo: "No",
+    horaLord: "Lord",
+    note: "Segments, classes and flags are facts, not a verdict on suitability. Times are in the time of the selected place.",
+  },
   arudha: {
     caption: "Arudha padas: the pada of each of the twelve houses, its sign and the lord of the house",
     pada: "Pada",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index dd0bd05..6a7b7f8 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -73,6 +73,7 @@ export const fr: Messages = {
     bhava_bala: "Bhava bala",
     yogas: "Yogas",
     compat: "Compatibilité",
+    muhurta_day: "Panchang du jour",
     unknown: "Panneau inconnu",
     loading: "Chargement",
     unavailable: "Ce panneau arrivera dans une version ultérieure.",
@@ -373,6 +374,85 @@ export const fr: Messages = {
     pickBack: "Retour aux clients",
     pickCount: "{{n}} options",
   },
+  muhurta: {
+    tara: "Tara bala",
+    chandra: "Chandra bala",
+    method: "Méthode : {{method}}",
+    place: {
+      label: "Lieu",
+      home: "Lieu actuel",
+      chart: "Lieu de naissance du thème",
+      custom: "Autre lieu",
+      pickHome: "Choisir le lieu actuel",
+      changeHome: "Changer le lieu actuel",
+      search: "Chercher un autre lieu",
+      missingHome: "Aucun lieu actuel n'est défini. Choisissez-en un avec « Choisir le lieu actuel », ou utilisez le lieu de naissance du thème.",
+      missingCustom: "Aucun autre lieu n'est défini. Choisissez un lieu dans la zone de recherche.",
+    },
+    home: {
+      title: "Lieu actuel",
+      current: "Lieu choisi : {{place}} · {{zone}}",
+      unset: "Aucun lieu actuel n'est défini.",
+      hint: "Le panchang du jour et la recherche de muhurta utilisent ce lieu par défaut. Aucun service de localisation n'est utilisé ; le lieu reste sur cet ordinateur et n'est jamais exporté.",
+      noPrefs: "Le lieu actuel ne peut pas être modifié, car les préférences n'ont pas pu être lues.",
+      search: "Chercher un lieu",
+      useChart: "Utiliser le lieu de naissance du thème ouvert ({{place}})",
+      remove: "Retirer le lieu",
+      close: "Fermer",
+      lmt: "temps moyen local",
+    },
+    nature: {
+      Fixed: "fixe",
+      Movable: "mobile",
+      Fierce: "féroce",
+      Mixed: "mixte",
+      Swift: "vif",
+      Soft: "doux",
+      Sharp: "tranchant",
+    },
+    flag: {
+      Rikta: "Tithi Rikta",
+      Vishti: "Karana Vishti",
+      Vyatipata: "Vyatipata",
+      Vaidhriti: "Vaidhriti",
+      RahuSegment: "Tranche de Rahu",
+      Yamaganda: "Yamaganda",
+      Gulika: "Gulika",
+      Panchaka: "Panchaka",
+      JupiterCombust: "Jupiter combuste",
+      VenusCombust: "Vénus combuste",
+    },
+  },
+  muhurtaDay: {
+    linkLabel: "Lien de date",
+    link: { transit: "Date du transit", today: "Aujourd'hui" },
+    title: "{{date}} · {{vara}} · {{weekday}}",
+    sun: "Lever {{sunrise}} · coucher {{sunset}} · lever suivant {{next}}",
+    noSunrise: "Le Soleil ne se lève pas ou ne se couche pas en ce lieu ce jour-là : les tranches du jour et la table des horas ne sont pas calculées. Le jour commence à 06:00 en temps moyen local ({{start}}).",
+    strip: "Les huit tranches du jour",
+    mark: { rahu: "Rahu", yamaganda: "Yama", gulika: "Guli", abhijit: "Abhi" },
+    abhijit: "Abhijit {{start}}–{{end}}",
+    abhijitWednesday: "(Mercredi : certains recueils n'utilisent pas Abhijit ce jour-là.)",
+    limbs: "Les cinq membres et leurs changements",
+    from: "Début",
+    to: "Fin",
+    fromBefore: "commence avant le début du jour",
+    goesOn: "se poursuit après la fin du jour",
+    group: "groupe {{group}}",
+    flags: "Marques",
+    noFlags: "Aucune marque ce jour-là.",
+    personal: "D'après la Lune du thème ouvert",
+    natalApprox: "approximatif : l'heure de naissance est inconnue, la Lune natale est celle de midi",
+    favourableColumn: "Ensemble",
+    favourable: "dans l'ensemble favorable",
+    notFavourable: "hors de l'ensemble favorable",
+    taraValue: "{{number}} · {{name}}",
+    chandraValue: "maison {{house}} depuis le signe de la Lune natale",
+    horas: "Table des horas",
+    horaNo: "N°",
+    horaLord: "Maître",
+    note: "Les tranches, les classes et les marques sont des faits, non un jugement de convenance. Les heures sont celles du lieu choisi.",
+  },
   arudha: {
     caption: "Arudha padas : le pada de chacune des douze maisons, son signe et le maître de la maison",
     pada: "Pada",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 393b509..aaa756d 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -72,6 +72,7 @@ export const tr = {
     bhava_bala: "Bhava bala",
     yogas: "Yogalar",
     compat: "Uyum",
+    muhurta_day: "Gün panchangı",
     unknown: "Bilinmeyen panel",
     loading: "Yükleniyor",
     unavailable: "Bu panel sonraki bir sürümde gelecek.",
@@ -372,6 +373,85 @@ export const tr = {
     pickBack: "Danışanlara dön",
     pickCount: "{{n}} seçenek",
   },
+  muhurta: {
+    tara: "Tara bala",
+    chandra: "Chandra bala",
+    method: "Yöntem: {{method}}",
+    place: {
+      label: "Yer",
+      home: "Bulunulan yer",
+      chart: "Haritanın doğum yeri",
+      custom: "Özel yer",
+      pickHome: "Bulunulan yeri seç",
+      changeHome: "Bulunulan yeri değiştir",
+      search: "Özel yer ara",
+      missingHome: "Bulunulan yer seçilmedi. “Bulunulan yeri seç” ile bir yer seçin ya da yer olarak haritanın doğum yerini kullanın.",
+      missingCustom: "Özel yer seçilmedi. Arama kutusundan bir yer seçin.",
+    },
+    home: {
+      title: "Bulunulan yer",
+      current: "Seçili yer: {{place}} · {{zone}}",
+      unset: "Bulunulan yer seçilmedi.",
+      hint: "Gün panchangı ve muhurta araması bu yeri varsayılan olarak kullanır. Konum servisi kullanılmaz; yer bu bilgisayarda durur ve dışa aktarıma yazılmaz.",
+      noPrefs: "Tercihler okunamadığı için bulunulan yer değiştirilemiyor.",
+      search: "Yer ara",
+      useChart: "Açık haritanın doğum yerini kullan ({{place}})",
+      remove: "Yeri kaldır",
+      close: "Kapat",
+      lmt: "yerel ortalama saat",
+    },
+    nature: {
+      Fixed: "sabit",
+      Movable: "yürük",
+      Fierce: "sert",
+      Mixed: "karışık",
+      Swift: "çevik",
+      Soft: "yumuşak",
+      Sharp: "keskin",
+    },
+    flag: {
+      Rikta: "Rikta tithi",
+      Vishti: "Vishti karana",
+      Vyatipata: "Vyatipata",
+      Vaidhriti: "Vaidhriti",
+      RahuSegment: "Rahu dilimi",
+      Yamaganda: "Yamaganda",
+      Gulika: "Gulika",
+      Panchaka: "Panchaka",
+      JupiterCombust: "Jüpiter yanık",
+      VenusCombust: "Venüs yanık",
+    },
+  },
+  muhurtaDay: {
+    linkLabel: "Tarih bağı",
+    link: { transit: "Transit tarihi", today: "Bugün" },
+    title: "{{date}} · {{vara}} · {{weekday}}",
+    sun: "Gün doğumu {{sunrise}} · gün batımı {{sunset}} · sonraki gün doğumu {{next}}",
+    noSunrise: "Bu yerde bu gün Güneş doğmuyor ya da batmıyor: gündüz dilimleri ve hora tablosu hesaplanmaz. Gün 06:00 yerel ortalama saatte ({{start}}) başlar.",
+    strip: "Gündüzün sekiz dilimi",
+    mark: { rahu: "Rahu", yamaganda: "Yama", gulika: "Guli", abhijit: "Abhi" },
+    abhijit: "Abhijit {{start}}–{{end}}",
+    abhijitWednesday: "(Çarşamba: bazı derlemeler bu gün Abhijit'i kullanmaz.)",
+    limbs: "Beş uzuv ve değişim saatleri",
+    from: "Başlangıç",
+    to: "Bitiş",
+    fromBefore: "günün başından önce başlar",
+    goesOn: "günün sonundan sonra sürer",
+    group: "{{group}} grubu",
+    flags: "İşaretler",
+    noFlags: "Bu gün işaret yok.",
+    personal: "Açık haritanın Ay'ına göre",
+    natalApprox: "yaklaşık: doğum saati bilinmiyor, doğum Ay'ı öğlen 12:00 içindir",
+    favourableColumn: "Küme",
+    favourable: "elverişli kümede",
+    notFavourable: "elverişli kümede değil",
+    taraValue: "{{number}} · {{name}}",
+    chandraValue: "doğum Ay burcundan {{house}}. ev",
+    horas: "Hora tablosu",
+    horaNo: "No",
+    horaLord: "Lord",
+    note: "Dilimler, sınıflar ve işaretler olgudur; uygunluk hükmü değildir. Saatler seçilen yerin saatiyledir.",
+  },
   arudha: {
     caption: "Arudha padalar: on iki evin padası, burcu ve evin lordu",
     pada: "Pada",
````


- [ ] **Step 8: Kapıları koş**

```bash
pnpm --dir app exec vitest run ui/panels/MuhurtaDayPanel.test.tsx ui/panels/HomePlaceDialog.test.tsx ui/modules/muhurta.test.ts 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
git status --short | grep -vE '^( M|\?\?) app/ui/'
```
Expected: `3 passed (3)`, `29 passed (29)`; sonra `78 passed (78)`, `771 passed (771)`; `✓ built`; son komut çıktısız (yalnızca `app/ui` değişti).

- [ ] **Step 9: Commit**

```bash
git add app/ui
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): day panchang panel with the day strip, limbs, flags, personal factors and horas; home place dialog; names in three languages" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: iki `grep` çıktısız.

---


### Task 7: Arayüz: Muhurta araması paneli

Spec §10 "Arayüz" (`muhurta_search` satırı; "Arama 100 ms'yi aşarsa …"), "Pencere araması" (süzgeç alanları; "astrolog kendi süzgecini kurar ve adlandırıp saklar"); §13 "Arayüz"; K12; UI spec §5, §6; D27, D28, D31–D34. Arayüz hesap yapmaz (C4): pencereler, özetleri ve ofsetleri Rust'tan gelir.

**Files:**
- Create: `app/ui/modules/useMuhurtaSearch.ts`, `app/ui/modules/useMuhurtaSearch.test.tsx`, `app/ui/panels/MuhurtaSearchPanel.tsx`, `app/ui/panels/MuhurtaSearchPanel.test.tsx`
- Modify: `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx`, `app/ui/panels/panels.css`, `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: `useMuhurtaSearch.test.tsx` (6), `MuhurtaSearchPanel.test.tsx` (18)

**Interfaces:**
- Consumes (Task 5, Task 6): `api.muhurtaSearch`, `api.loadMuhurtaFilters`, `api.saveMuhurtaFilters`; bağlama tipleri `MuhurtaQuery`, `MuhurtaPageCursor`, `MuhurtaSearchView`, `MuhurtaWindowView`, `SavedMuhurtaFilter`, `MuhurtaCriteria`; `modules/muhurta.ts`; `usePanelPlace`, `PlaceControls`, `PlaceMissing`; `useModule`, `usePanelLoadPhase`, `PanelError`, `Approx`, `useTransitControl`, `formatJdClock`, `formatJdDate`, `formatJdSpan`, `tithiShort`.
- Produces:
  - `useMuhurtaSearch(scope) -> MuhurtaSearch` (`status`, `windows`, `scanned`, `chunks`, `range`, `warnings`, `method`, `error`, `start(query, settings)`, `stop()`); `SearchStatus`.
  - `MuhurtaSearchPanel`, `MAX_SHOWN_WINDOWS`.
  - Panel türü `muhurta_search`; i18n `panel.muhurta_search`, `muhurtaSearch.*`.

- [ ] **Step 1: Testler**

**Yeni dosya:** `app/ui/modules/useMuhurtaSearch.test.tsx`

````tsx
import { act, render, screen } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { muhurtaSearch: vi.fn() } }));

import { api, type Result } from "../api";
import type { MuhurtaSearchView } from "../bindings";
import { fixture } from "../test/fixture";
import { useMuhurtaSearch, type MuhurtaSearch } from "./useMuhurtaSearch";

const FX = fixture();
const PAGES = FX.muhurta.pages;
const QUERY = FX.muhurta.query;
const SETTINGS = FX.opened.chart.meta.settings;

/** Elle çözülen yanıt: sayfaların geliş sırasını test belirler. */
function deferred() {
  let resolve!: (r: Result<MuhurtaSearchView>) => void;
  const promise = new Promise<Result<MuhurtaSearchView>>((r) => (resolve = r));
  return { promise, resolve };
}
const ok = (page: MuhurtaSearchView): Result<MuhurtaSearchView> => ({ status: "ok", data: page });

let search: MuhurtaSearch;
function Probe({ scope = "1" }: { scope?: string }) {
  search = useMuhurtaSearch(scope);
  return <span data-testid="state">{`${search.status} ${search.windows.length} ${search.scanned}/${search.chunks}`}</span>;
}
const state = () => screen.getByTestId("state").textContent;

beforeEach(() => vi.mocked(api.muhurtaSearch).mockReset());

test("pages are asked one after another with the cursor of the page before, and windows add up", async () => {
  const replies = [deferred(), deferred(), deferred()];
  replies.forEach((r) => vi.mocked(api.muhurtaSearch).mockReturnValueOnce(r.promise));
  render(<Probe />);
  expect(state()).toBe("idle 0 0/0");
  act(() => search.start(QUERY, SETTINGS));
  expect(state()).toBe("running 0 0/0");
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.muhurtaSearch).mock.calls[0]).toEqual([QUERY, SETTINGS, null]);
  await act(async () => replies[0].resolve(ok(PAGES[0])));
  expect(state()).toBe("running 15 1/3");
  // İkinci istek ilk yanıtın imleciyle gider (aralık imleçte taşınır).
  expect(vi.mocked(api.muhurtaSearch).mock.calls[1]).toEqual([QUERY, SETTINGS, PAGES[0].next]);
  await act(async () => replies[1].resolve(ok(PAGES[1])));
  expect(state()).toBe("running 27 2/3");
  await act(async () => replies[2].resolve(ok(PAGES[2])));
  expect(state()).toBe("done 30 3/3");
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(3);
  expect(search.range).toEqual({ from_jd_ut: PAGES[0].from_jd_ut, to_jd_ut: PAGES[0].to_jd_ut, from_offset_secs: 19800, to_offset_secs: 19800 });
  expect(search.method).toBe("muhurta.v1");
  expect(search.windows).toEqual(PAGES.flatMap((p) => p.windows));
});

test("stopping keeps the windows so far and asks for no further page", async () => {
  const replies = [deferred(), deferred()];
  replies.forEach((r) => vi.mocked(api.muhurtaSearch).mockReturnValueOnce(r.promise));
  render(<Probe />);
  act(() => search.start(QUERY, SETTINGS));
  await act(async () => replies[0].resolve(ok(PAGES[0])));
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(2);
  act(() => search.stop());
  expect(state()).toBe("stopped 15 1/3");
  // Yoldaki sayfanın yanıtı düşer; üçüncü sayfa hiç istenmez.
  await act(async () => replies[1].resolve(ok(PAGES[1])));
  expect(state()).toBe("stopped 15 1/3");
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(2);
  // Durmuş aramada `stop` bir şey değiştirmez.
  act(() => search.stop());
  expect(state()).toBe("stopped 15 1/3");
});

test("a new search replaces the old one and the old one's late page is dropped", async () => {
  const [old, fresh] = [deferred(), deferred()];
  vi.mocked(api.muhurtaSearch).mockReturnValueOnce(old.promise).mockReturnValueOnce(fresh.promise);
  render(<Probe />);
  act(() => search.start(QUERY, SETTINGS));
  act(() => search.start({ ...QUERY, to_date: "2026-12-01" }, SETTINGS));
  expect(state()).toBe("running 0 0/0");
  await act(async () => old.resolve(ok(PAGES[0])));
  expect(state()).toBe("running 0 0/0");
  await act(async () => fresh.resolve(ok({ ...PAGES[2], scanned: 1, chunks: 1 })));
  expect(state()).toBe("done 3 1/1");
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(2);
});

test("an error ends the search and keeps the windows that arrived", async () => {
  vi.mocked(api.muhurtaSearch)
    .mockResolvedValueOnce(ok(PAGES[0]))
    .mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  render(<Probe />);
  await act(async () => search.start(QUERY, SETTINGS));
  expect(state()).toBe("error 15 1/3");
  expect(search.error?.code).toBe("calc");
  // Yeni arama hatayı ve eski pencereleri siler.
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok({ ...PAGES[2], scanned: 1, chunks: 1 }));
  await act(async () => search.start(QUERY, SETTINGS));
  expect(state()).toBe("done 3 1/1");
  expect(search.error).toBeNull();
});

test("warnings of every page are gathered once", async () => {
  vi.mocked(api.muhurtaSearch)
    .mockResolvedValueOnce(ok({ ...PAGES[0], warnings: ["SunriseUnavailable"] }))
    .mockResolvedValueOnce(ok({ ...PAGES[1], warnings: ["SunriseUnavailable", "LowPrecision"] }))
    .mockResolvedValueOnce(ok(PAGES[2]));
  render(<Probe />);
  await act(async () => search.start(QUERY, SETTINGS));
  expect(state()).toBe("done 30 3/3");
  expect(search.warnings).toEqual(["SunriseUnavailable", "LowPrecision"]);
});

test("another chart drops the running search and its results; unmounting stops asking", async () => {
  const [first, second, third] = [deferred(), deferred(), deferred()];
  vi.mocked(api.muhurtaSearch).mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise).mockReturnValueOnce(third.promise);
  const { rerender, unmount } = render(<Probe scope="1" />);
  act(() => search.start(QUERY, SETTINGS));
  await act(async () => first.resolve(ok(PAGES[0])));
  expect(state()).toBe("running 15 1/3");
  rerender(<Probe scope="2" />);
  expect(state()).toBe("idle 0 0/0");
  await act(async () => second.resolve(ok(PAGES[1])));
  expect(state()).toBe("idle 0 0/0");
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(2);
  act(() => search.start(QUERY, SETTINGS));
  unmount();
  await act(async () => third.resolve(ok(PAGES[0])));
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(3);
});
````

**Yeni dosya:** `app/ui/panels/MuhurtaSearchPanel.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { muhurtaSearch: vi.fn(), loadMuhurtaFilters: vi.fn(), saveMuhurtaFilters: vi.fn(), setAppPrefs: vi.fn(), searchPlaces: vi.fn() } }));

import { api, type Result } from "../api";
import type { AppPrefs, MuhurtaCriteria, MuhurtaSearchView, MuhurtaWindowView, SavedMuhurtaFilter } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { EMPTY_CRITERIA } from "../modules/muhurta";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useTransitControl, type Transit } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { MAX_SHOWN_WINDOWS, MuhurtaSearchPanel } from "./MuhurtaSearchPanel";

const FX = fixture();
const PAGES = FX.muhurta.pages;
const ok = <T,>(data: T): Result<T> => ({ status: "ok", data });
const failure = { status: "error" as const, error: { code: "calc", params: { detail: "x" } } };
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "ms", kind: "muhurta_search", x: 0, y: 0, w: 8, h: 18, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};
const criteria = (patch: Partial<MuhurtaCriteria> = {}): MuhurtaCriteria => ({ ...EMPTY_CRITERIA, ...patch });
const saved = (name: string, patch: Partial<MuhurtaCriteria> = {}): SavedMuhurtaFilter => ({ name, criteria: criteria(patch) });

let transit: Transit;
/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  transit = useTransitControl();
  return <MuhurtaSearchPanel panel={active.panels[0]} />;
}

interface Options {
  opened?: ReturnType<typeof fixture>["opened"] | null;
  prefs?: AppPrefs | null;
}
function renderPanel(settings: Record<string, unknown> = { place: "chart" }, { opened = fixture().opened, prefs = FX.prefs.prefs }: Options = {}) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened, prefs });
}

/** Elle çözülen sayfa: geliş sırasını test belirler. */
function deferred() {
  let resolve!: (r: Result<MuhurtaSearchView>) => void;
  const promise = new Promise<Result<MuhurtaSearchView>>((r) => (resolve = r));
  return { promise, resolve };
}
/** Fikstürün üç sayfasını sırayla döner. */
function threePages() {
  for (const page of fixture().muhurta.pages) vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok(page));
}
const run = () => screen.getByRole("button", { name: "Ara" });
const summary = () => document.querySelector("[data-summary]")?.textContent;
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const resultRows = () => within(screen.getByRole("table", { name: "Bulunan pencereler" })).getAllByRole("row").slice(1);
const savedSettings = (persistence: ReturnType<typeof renderPanel>["persistence"]) =>
  persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings;
const box = (name: string) => screen.getByRole("checkbox", { name });

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.loadMuhurtaFilters).mockResolvedValue(ok([]));
  vi.mocked(api.saveMuhurtaFilters).mockImplementation(async (filters) => ok(filters));
  vi.mocked(api.setAppPrefs).mockResolvedValue(ok(null));
});

test("the form has visibly labelled fields and nothing is searched until asked", async () => {
  renderPanel();
  await waitFor(() => expect(api.loadMuhurtaFilters).toHaveBeenCalledTimes(1));
  const form = screen.getByRole("form", { name: "Muhurta süzgeci" });
  for (const name of ["Kayıtlı süzgeç", "Yer"]) expect(within(form).getByRole("combobox", { name })).toBeInTheDocument();
  for (const name of ["Başlangıç tarihi", "Bitiş tarihi"]) expect(within(form).getByLabelText(name)).toHaveAttribute("type", "date");
  expect(within(form).getByRole("spinbutton", { name: "En kısa süre (dk)" })).toHaveValue(0);
  expect(within(form).getByRole("textbox", { name: "Süzgeç adı" })).toHaveValue("");
  // Beş küme: her biri başlıklı bir gruptur; kutuların etiketi görünür.
  expect([...form.querySelectorAll("fieldset > legend")].map((l) => l.textContent)).toEqual(["Günler", "Tithiler", "Nakshatralar", "Lagna burçları", "Dışlananlar"]);
  expect(within(screen.getByRole("group", { name: "Günler" })).getAllByRole("checkbox").map((c) => c.closest("label")?.textContent)).toEqual([
    "Pazar",
    "Pazartesi",
    "Salı",
    "Çarşamba",
    "Perşembe",
    "Cuma",
    "Cumartesi",
  ]);
  expect(within(screen.getByRole("group", { name: "Tithiler" })).getAllByRole("checkbox")).toHaveLength(30);
  expect(within(screen.getByRole("group", { name: "Nakshatralar" })).getAllByRole("checkbox")).toHaveLength(27);
  expect(within(screen.getByRole("group", { name: "Lagna burçları" })).getAllByRole("checkbox")).toHaveLength(12);
  expect(within(screen.getByRole("group", { name: "Dışlananlar" })).getAllByRole("checkbox").map((c) => c.closest("label")?.textContent)).toEqual([
    "Rikta tithi",
    "Vishti karana",
    "Vyatipata",
    "Vaidhriti",
    "Rahu dilimi",
    "Yamaganda",
    "Gulika",
    "Panchaka",
    "Jüpiter yanık",
    "Venüs yanık",
  ]);
  expect(box("S9 Navami")).not.toBeChecked();
  expect(box("K15 Amavasya")).toBeInTheDocument();
  expect(screen.getByText("Tithiler · kısıt yok")).toBeInTheDocument();
  expect(screen.getByText("Hiçbir kutusu işaretli olmayan küme kısıt değildir.")).toBeInTheDocument();
  expect(screen.getByText(/^Süzgeci kurup “Ara”ya basın\./)).toBeInTheDocument();
  expect(api.muhurtaSearch).not.toHaveBeenCalled();
  expect(screen.queryByRole("table")).toBeNull();
});

test("a search sends the filter, the place and the natal Moon, then lists the windows of every page", async () => {
  threePages();
  renderPanel({ place: "chart", criteria: FX.muhurta.query.criteria });
  fireEvent.click(run());
  await waitFor(() => expect(summary()).toBe("30 pencere · 2026-11-26 – 2027-01-31"));
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(3);
  // Tarihler boş: Rust bugünden başlar. Yerin adı gitmez.
  expect(vi.mocked(api.muhurtaSearch).mock.calls[0]).toEqual([
    { from_date: null, to_date: null, place: FX.muhurta.place, criteria: FX.muhurta.query.criteria, natal: FX.muhurta.natal },
    FX.opened.chart.meta.settings,
    null,
  ]);
  expect(vi.mocked(api.muhurtaSearch).mock.calls[1][2]).toEqual(PAGES[0].next);
  expect(vi.mocked(api.muhurtaSearch).mock.calls[2][2]).toEqual(PAGES[1].next);
  const table = screen.getByRole("table", { name: "Bulunan pencereler" });
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Başlangıç", "Bitiş", "Süre", "Vara", "Tithi", "Nakshatra", "Nitya yoga", "Karana", "Lagna"]);
  const rows = resultRows();
  expect(rows).toHaveLength(30);
  expect(cells(rows[0])).toEqual(["2026-11-26 08:14", "2026-11-26 09:32", "1 sa 17 dk", "Perşembe", "K2 Dwitiya", "Mrigashira 3", "Sadhya", "Garaja", "Akr"]);
  expect(cells(rows[1])).toEqual(["2026-11-26 10:50", "2026-11-26 13:26", "2 sa 35 dk", "Perşembe", "K2 Dwitiya", "Mrigashira 3", "Sadhya", "Garaja", "Oğl"]);
  // Süzgeç Perşembe gündüzleriydi; her pencere en az 30 dakikadır.
  expect(new Set(rows.map((r) => cells(r)[3]))).toEqual(new Set(["Perşembe"]));
  expect(screen.getByText("Yöntem: muhurta.v1")).toBeInTheDocument();
  expect(screen.getByText(/Pencereler olgudur; uygunluk hükmü değildir\.$/)).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Durdur" })).toBeNull();
});

test("the criteria are edited with checkboxes and saved with the layout", async () => {
  const { persistence } = renderPanel();
  fireEvent.click(box("Perşembe"));
  fireEvent.click(box("Pazartesi"));
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ place: "chart", criteria: criteria({ weekdays: ["Monday", "Thursday"] }) }));
  fireEvent.click(box("K14 Chaturdashi"));
  fireEvent.click(box("S2 Dwitiya"));
  fireEvent.click(box("Rohini"));
  fireEvent.click(box("Asl"));
  fireEvent.click(box("Rahu dilimi"));
  fireEvent.click(box("Rikta tithi"));
  fireEvent.click(box("Yalnızca gündüz"));
  fireEvent.click(box("Tara bala elverişli"));
  fireEvent.change(screen.getByRole("spinbutton", { name: "En kısa süre (dk)" }), { target: { value: "45" } });
  const want = criteria({
    weekdays: ["Monday", "Thursday"],
    tithis: [2, 29],
    nakshatras: [4],
    lagna_signs: ["Leo"],
    daytime_only: true,
    min_minutes: 45,
    exclude: ["Rikta", "RahuSegment"],
    require_tara: true,
  });
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ place: "chart", criteria: want }));
  expect(box("Perşembe")).toBeChecked();
  expect(screen.getByText("Tithiler · 2 seçili")).toBeInTheDocument();
  expect(screen.getByText("Nakshatralar · 1 seçili")).toBeInTheDocument();
  // Kutuyu yeniden işaretlemek kümeden çıkarır; aralık dışı süre sınıra çekilir.
  fireEvent.click(box("Pazartesi"));
  fireEvent.change(screen.getByRole("spinbutton", { name: "En kısa süre (dk)" }), { target: { value: "99999" } });
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ place: "chart", criteria: { ...want, weekdays: ["Thursday"], min_minutes: 10080 } }));
  // Arama, saklanan koşullarla gider.
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok({ ...PAGES[2], scanned: 1, chunks: 1 }));
  fireEvent.click(run());
  await waitFor(() => expect(api.muhurtaSearch).toHaveBeenCalledTimes(1));
  expect(vi.mocked(api.muhurtaSearch).mock.calls[0][0].criteria).toEqual({ ...want, weekdays: ["Thursday"], min_minutes: 10080 });
});

test("typed dates are sent as they are and broken stored criteria fall back to no constraint", async () => {
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok({ ...PAGES[2], scanned: 1, chunks: 1 }));
  renderPanel({ place: "chart", criteria: { weekdays: "Friday", tithis: [99], min_minutes: -1 } });
  fireEvent.change(screen.getByLabelText("Başlangıç tarihi"), { target: { value: "2027-01-10" } });
  fireEvent.change(screen.getByLabelText("Bitiş tarihi"), { target: { value: "2027-01-31" } });
  // Enter da aramayı başlatır (form gönderimi).
  fireEvent.submit(screen.getByRole("form", { name: "Muhurta süzgeci" }));
  await waitFor(() => expect(summary()).toBe("3 pencere · 2026-11-26 – 2027-01-31"));
  expect(vi.mocked(api.muhurtaSearch).mock.calls[0][0]).toMatchObject({ from_date: "2027-01-10", to_date: "2027-01-31", criteria: EMPTY_CRITERIA });
});

test("the start of a row moves the transit date to that moment", async () => {
  threePages();
  renderPanel();
  fireEvent.click(run());
  await waitFor(() => expect(resultRows()).toHaveLength(30));
  expect(transit.at).toBe("Now");
  const first = PAGES[0].windows[0];
  const button = within(resultRows()[0]).getByRole("button", { name: "Transit tarihini 2026-11-26 08:14 anına götür" });
  expect(button).toHaveTextContent("2026-11-26 08:14");
  fireEvent.click(button);
  expect(transit.at).toEqual({ Jd: first.start_jd_ut });
  // Sonuç listesi ve odak yerinde durur.
  expect(resultRows()).toHaveLength(30);
});

test("past a second the search shows its progress and a stop button; stopping keeps the windows and the focus", async () => {
  vi.useFakeTimers();
  try {
    const replies = [deferred(), deferred()];
    replies.forEach((r) => vi.mocked(api.muhurtaSearch).mockReturnValueOnce(r.promise));
    renderPanel();
    fireEvent.click(run());
    await act(async () => vi.advanceTimersByTime(999));
    // Bir saniyeden kısa aramada ilerleme ve "Durdur" çıkmaz.
    expect(screen.queryByRole("button", { name: "Durdur" })).toBeNull();
    expect(screen.getByRole("status")).toHaveTextContent("");
    expect(summary()).toBe("Aranıyor…");
    await act(async () => vi.advanceTimersByTime(1));
    const stop = screen.getByRole("button", { name: "Durdur" });
    await act(async () => replies[0].resolve(ok(fixture().muhurta.pages[0])));
    expect(screen.getByRole("status")).toHaveTextContent("Taranan bölüm: 1 / 3");
    expect(summary()).toBe("Şimdiye dek 15 pencere · 2026-11-26 – 2027-01-31");
    // Arama sürerken "Ara" ikinci bir arama başlatmaz.
    expect(run()).toHaveAttribute("aria-disabled", "true");
    fireEvent.click(run());
    expect(api.muhurtaSearch).toHaveBeenCalledTimes(2);
    stop.focus();
    fireEvent.click(stop);
    // "Durdur" kalkar; odak gövdeye düşmez, "Ara"ya geçer.
    expect(stop).not.toBeInTheDocument();
    expect(run()).toHaveFocus();
    expect(run()).not.toHaveAttribute("aria-disabled");
    expect(summary()).toBe("Arama durduruldu: 15 pencere · 2026-11-26 – 2027-01-31");
    expect(resultRows()).toHaveLength(15);
    // Yoldaki sayfa gelir: eklenmez, yeni sayfa istenmez.
    await act(async () => replies[1].resolve(ok(fixture().muhurta.pages[1])));
    expect(resultRows()).toHaveLength(15);
    expect(api.muhurtaSearch).toHaveBeenCalledTimes(2);
    expect(screen.getByRole("status")).toHaveTextContent("");
  } finally {
    vi.useRealTimers();
  }
});

test("the list does not unmount between pages: a focused row keeps the focus while more windows arrive", async () => {
  const replies = [deferred(), deferred(), deferred()];
  replies.forEach((r) => vi.mocked(api.muhurtaSearch).mockReturnValueOnce(r.promise));
  renderPanel();
  fireEvent.click(run());
  await act(async () => replies[0].resolve(ok(fixture().muhurta.pages[0])));
  const table = screen.getByRole("table", { name: "Bulunan pencereler" });
  const button = within(resultRows()[3]).getByRole("button");
  button.focus();
  await act(async () => replies[1].resolve(ok(fixture().muhurta.pages[1])));
  expect(resultRows()).toHaveLength(27);
  await act(async () => replies[2].resolve(ok(fixture().muhurta.pages[2])));
  expect(resultRows()).toHaveLength(30);
  expect(screen.getByRole("table", { name: "Bulunan pencereler" })).toBe(table);
  expect(button).toBeInTheDocument();
  expect(button).toHaveFocus();
  // Arama bittiğinde odak gövdeye düşmemiştir.
  expect(document.activeElement).not.toBe(document.body);
});

test("an error is shown with the windows that arrived, and retry searches again", async () => {
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok(fixture().muhurta.pages[0])).mockResolvedValueOnce(failure);
  renderPanel();
  fireEvent.click(run());
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(summary()).toBe("Arama tamamlanamadı: 15 pencere · 2026-11-26 – 2027-01-31");
  expect(resultRows()).toHaveLength(15);
  // Form hata durumunda da kullanılabilir.
  expect(box("Perşembe")).toBeEnabled();
  threePages();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(summary()).toBe("30 pencere · 2026-11-26 – 2027-01-31"));
  expect(screen.queryByRole("alert")).toBeNull();
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(5);
});

test("an invalid range is reported by the field Rust names", async () => {
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce({ status: "error", error: { code: "invalid_muhurta_input", params: { field: "range" } } });
  renderPanel();
  fireEvent.click(run());
  expect(await screen.findByRole("alert")).toHaveTextContent("Muhurta isteği geçersiz: range.");
  // Hiç yanıt gelmedi: özet yerine arama satırı durur, tablo yoktur.
  expect(screen.queryByRole("table")).toBeNull();
});

test("without a place the search button stays focusable, says why and does nothing", async () => {
  renderPanel({});
  const button = run();
  expect(button).toHaveAttribute("aria-disabled", "true");
  expect(button).not.toBeDisabled();
  const why = document.getElementById(button.getAttribute("aria-describedby")!);
  expect(why).toHaveTextContent(/^Bulunulan yer seçilmedi\./);
  fireEvent.click(button);
  await act(() => Promise.resolve());
  expect(api.muhurtaSearch).not.toHaveBeenCalled();
  // Yer seçilince düğme kullanılabilir olur.
  fireEvent.change(screen.getByRole("combobox", { name: "Yer" }), { target: { value: "chart" } });
  await waitFor(() => expect(run()).not.toHaveAttribute("aria-disabled"));
  expect(run()).not.toHaveAttribute("aria-describedby");
});

test("a saved filter is loaded into the form; saving adds or overwrites by name", async () => {
  vi.mocked(api.loadMuhurtaFilters).mockResolvedValue(ok([saved("Yolculuk", { weekdays: ["Thursday"], exclude: ["Vishti"] }), saved("Açılış", { daytime_only: true })]));
  const { persistence } = renderPanel();
  const select = screen.getByRole("combobox", { name: "Kayıtlı süzgeç" });
  await waitFor(() => expect(within(select).getAllByRole("option").map((o) => o.textContent)).toEqual(["—", "Yolculuk", "Açılış"]));
  const name = screen.getByRole("textbox", { name: "Süzgeç adı" });
  const save = screen.getByRole("button", { name: "Süzgeci kaydet" });
  const remove = screen.getByRole("button", { name: "Süzgeci sil" });
  // Ad ve seçim yokken iki eylem de kullanılamaz ama odaklanabilir.
  expect(save).toHaveAttribute("aria-disabled", "true");
  expect(remove).toHaveAttribute("aria-disabled", "true");
  fireEvent.click(save);
  fireEvent.click(remove);
  expect(api.saveMuhurtaFilters).not.toHaveBeenCalled();

  fireEvent.change(select, { target: { value: "Yolculuk" } });
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ place: "chart", criteria: criteria({ weekdays: ["Thursday"], exclude: ["Vishti"] }) }));
  expect(name).toHaveValue("Yolculuk");
  expect(box("Perşembe")).toBeChecked();
  expect(box("Vishti karana")).toBeChecked();

  // Koşulu değiştirip aynı adla kaydetmek üzerine yazar (büyük/küçük harf ayırmadan).
  fireEvent.click(box("Cuma"));
  fireEvent.change(name, { target: { value: " yolculuk " } });
  fireEvent.click(save);
  await waitFor(() => expect(api.saveMuhurtaFilters).toHaveBeenCalledTimes(1));
  expect(vi.mocked(api.saveMuhurtaFilters).mock.calls[0][0]).toEqual([saved("yolculuk", { weekdays: ["Thursday", "Friday"], exclude: ["Vishti"] }), saved("Açılış", { daytime_only: true })]);
  await waitFor(() => expect(within(select).getAllByRole("option").map((o) => o.textContent)).toEqual(["—", "yolculuk", "Açılış"]));
  expect(select).toHaveValue("yolculuk");

  // Yeni ad listeye eklenir. Kayıt yalnızca ad ve koşul taşır.
  fireEvent.change(name, { target: { value: "Hasat" } });
  fireEvent.click(save);
  await waitFor(() => expect(api.saveMuhurtaFilters).toHaveBeenCalledTimes(2));
  const sent = vi.mocked(api.saveMuhurtaFilters).mock.calls[1][0];
  expect(sent.map((f) => f.name)).toEqual(["yolculuk", "Açılış", "Hasat"]);
  expect(Object.keys(sent[2]).sort()).toEqual(["criteria", "name"]);
  expect(Object.keys(sent[2].criteria).sort()).toEqual(Object.keys(EMPTY_CRITERIA).sort());
  await waitFor(() => expect(select).toHaveValue("Hasat"));
});

test("deleting the selected filter moves the focus to the picker; a failed save is reported", async () => {
  vi.mocked(api.loadMuhurtaFilters).mockResolvedValue(ok([saved("Yolculuk"), saved("Açılış")]));
  renderPanel();
  const select = screen.getByRole("combobox", { name: "Kayıtlı süzgeç" });
  await waitFor(() => expect(within(select).getAllByRole("option")).toHaveLength(3));
  fireEvent.change(select, { target: { value: "Açılış" } });
  const remove = screen.getByRole("button", { name: "Süzgeci sil" });
  expect(remove).not.toHaveAttribute("aria-disabled");
  // Kayıt düşerse liste ve seçim yerinde durur.
  vi.mocked(api.saveMuhurtaFilters).mockResolvedValueOnce({ status: "error", error: { code: "invalid_input", params: { field: "muhurta_filters" } } });
  remove.focus();
  fireEvent.click(remove);
  expect(await screen.findByRole("alert")).toHaveTextContent("Geçersiz değer: muhurta_filters.");
  expect(select).toHaveValue("Açılış");
  expect(within(select).getAllByRole("option")).toHaveLength(3);
  fireEvent.click(remove);
  await waitFor(() => expect(within(select).getAllByRole("option").map((o) => o.textContent)).toEqual(["—", "Yolculuk"]));
  expect(vi.mocked(api.saveMuhurtaFilters).mock.calls[1][0]).toEqual([saved("Yolculuk")]);
  // Silinen süzgecin düğmesi kullanılamaz olur: odak seçiciye taşınmıştır.
  expect(select).toHaveFocus();
  expect(select).toHaveValue("");
  expect(screen.getByRole("textbox", { name: "Süzgeç adı" })).toHaveValue("");
  expect(remove).toHaveAttribute("aria-disabled", "true");
  expect(screen.queryByRole("alert")).toBeNull();
});

test("a long result is cut at the display limit and says so", async () => {
  const window = PAGES[0].windows[0];
  const many: MuhurtaWindowView[] = Array.from({ length: MAX_SHOWN_WINDOWS + 100 }, (_, i) => ({
    ...window,
    start_jd_ut: (window.start_jd_ut ?? 0) + i,
    end_jd_ut: (window.end_jd_ut ?? 0) + i,
  }));
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok({ ...PAGES[2], windows: many, scanned: 1, chunks: 1 }));
  renderPanel();
  fireEvent.click(run());
  await waitFor(() => expect(summary()).toBe("600 pencere · 2026-11-26 – 2027-01-31"));
  expect(resultRows()).toHaveLength(500);
  expect(screen.getByText("İlk 500 pencere gösteriliyor (toplam 600). Süzgeci daraltın ya da aralığı kısaltın.")).toBeInTheDocument();
});

test("no window, a day without a sunrise and a fallback ephemeris are said in words", async () => {
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok({ ...PAGES[2], windows: [], scanned: 1, chunks: 1, warnings: ["SunriseUnavailable", "LowPrecision"] }));
  renderPanel();
  fireEvent.click(run());
  await waitFor(() => expect(summary()).toBe("0 pencere · 2026-11-26 – 2027-01-31"));
  expect(screen.queryByRole("table")).toBeNull();
  const notes = screen.getAllByRole("note").map((n) => n.textContent);
  expect(notes[0]).toMatch(/^Aralığın bir bölümünde Güneş doğmuyor ya da batmıyor/);
  expect(notes[1]).toBe("Düşük hassasiyet: Güneş veya Ay yedek ephemeris ile hesaplandı.");
});

test("the result list is a keyboard-focusable scroll group named by its caption", async () => {
  threePages();
  renderPanel();
  fireEvent.click(run());
  await waitFor(() => expect(resultRows()).toHaveLength(30));
  const group = screen.getByRole("group", { name: "Bulunan pencereler" });
  expect(group).toHaveAttribute("tabindex", "0");
  expect(within(group).getByRole("table", { name: "Bulunan pencereler" })).toBeInTheDocument();
});

test("an unknown birth time marks the two personal conditions as approximate", async () => {
  vi.mocked(api.muhurtaSearch).mockResolvedValueOnce(ok({ ...PAGES[2], scanned: 1, chunks: 1 }));
  renderPanel({ place: "chart" }, { opened: unknownTimeOpened() });
  for (const name of ["Tara bala elverişli", "Chandra bala elverişli"]) {
    const label = screen.getByText(name).closest("label")!;
    expect(label).toHaveTextContent(`≈yaklaşık: doğum saati bilinmiyor, doğum Ay'ı öğlen 12:00 içindir ${name}`);
    expect(within(label).getByRole("checkbox")).toBeEnabled();
  }
  // Doğum Ay'ı yine gönderilir (öğle konumu).
  fireEvent.click(run());
  await waitFor(() => expect(api.muhurtaSearch).toHaveBeenCalledTimes(1));
  expect(vi.mocked(api.muhurtaSearch).mock.calls[0][0].natal).toEqual(FX.muhurta.natal);
});

test("another chart drops the results, and without a chart the panel asks for nothing", async () => {
  threePages();
  const { unmount } = renderPanel();
  fireEvent.click(run());
  await waitFor(() => expect(resultRows()).toHaveLength(30));
  unmount();
  vi.mocked(api.loadMuhurtaFilters).mockClear();
  renderPanel({ place: "chart" }, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  await act(() => Promise.resolve());
  expect(api.loadMuhurtaFilters).not.toHaveBeenCalled();
  expect(api.muhurtaSearch).toHaveBeenCalledTimes(3);
});

test("the search texts exist in three languages and carry no verdict words", () => {
  for (const lang of [tr, en, fr]) {
    expect(Object.keys(lang.muhurtaSearch.summary).sort()).toEqual(["done", "error", "running", "stopped"]);
    expect(Object.keys(lang.muhurtaSearch.unit).sort()).toEqual(["day", "hour", "minute"]);
  }
  // Panel olgu gösterir: "uygun", "uğurlu", "auspicious", "favorable jour" gibi hüküm sözleri yoktur
  // ("elverişli" yalnızca spec'in adlandırdığı tara ve chandra bala kümesinin adıdır).
  const words = /uğur|iyi gün|kötü gün|auspicious|inauspicious|good day|bad day|faste|néfaste/i;
  for (const lang of [tr, en, fr]) {
    for (const group of [lang.muhurtaSearch, lang.muhurtaDay, lang.muhurta]) expect(JSON.stringify(group)).not.toMatch(words);
  }
});
````

````diff
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index 1ccad01..8489460 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -44,6 +44,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "yogas",
     "compat",
     "muhurta_day",
+    "muhurta_search",
   ]);
   expect(isKnownPanelKind("chart")).toBe(true);
   expect(isKnownPanelKind("drishti")).toBe(true);
@@ -51,7 +52,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas", "compat", "muhurta_day"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas", "compat", "muhurta_day", "muhurta_search"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index d73e465..6b30153 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -19,6 +19,7 @@ import { DrishtiPanel } from "./DrishtiPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
 import { JaiminiKarakasPanel } from "./JaiminiKarakasPanel";
 import { MuhurtaDayPanel } from "./MuhurtaDayPanel";
+import { MuhurtaSearchPanel } from "./MuhurtaSearchPanel";
 import { PanchangPanel } from "./PanchangPanel";
 import { panelDefinition, PANEL_REGISTRY } from "./registry";
 import { ShadbalaPanel } from "./ShadbalaPanel";
@@ -56,6 +57,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.yogas.component).toBe(YogasPanel);
   expect(PANEL_REGISTRY.compat.component).toBe(CompatPanel);
   expect(PANEL_REGISTRY.muhurta_day.component).toBe(MuhurtaDayPanel);
+  expect(PANEL_REGISTRY.muhurta_search.component).toBe(MuhurtaSearchPanel);
   // Kataloğun her türü uygulanmıştır: yer tutucu yoktur.
   for (const kind of PANEL_KINDS) expect(PANEL_REGISTRY[kind].component, kind).not.toBe(UnavailablePanel);
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
````


- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
pnpm --dir app exec vitest run ui/modules/useMuhurtaSearch.test.tsx ui/panels/MuhurtaSearchPanel.test.tsx ui/layout/model.test.ts ui/panels/panels.test.tsx 2>&1 | grep -E "Test Files|Tests "
```
Expected: `Test Files  4 failed (4)`, `Tests  2 failed | 22 passed (24)`: iki yeni dosya ve `panels.test.tsx` içe aktarım hatasıyla, `layout/model.test.ts` iki testle düşer.

- [ ] **Step 3: Panel türü**

````diff
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index 9dba4e7..518980d 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -21,6 +21,7 @@ export const PANEL_KINDS = [
   "yogas",
   "compat",
   "muhurta_day",
+  "muhurta_search",
 ] as const;
 export type KnownPanelKind = (typeof PANEL_KINDS)[number];
 
@@ -45,6 +46,7 @@ export const MODULE_PANELS = [
   { kind: "compat", w: 7, h: 16 },
   // Muhurta (Plan 20).
   { kind: "muhurta_day", w: 7, h: 18 },
+  { kind: "muhurta_search", w: 8, h: 18 },
 ] as const satisfies readonly { kind: KnownPanelKind; w: number; h: number }[];
 export type ModulePanelKind = (typeof MODULE_PANELS)[number]["kind"];
 
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index ae97960..a271209 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -15,6 +15,7 @@ import { DrishtiPanel } from "./DrishtiPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
 import { JaiminiKarakasPanel } from "./JaiminiKarakasPanel";
 import { MuhurtaDayPanel } from "./MuhurtaDayPanel";
+import { MuhurtaSearchPanel } from "./MuhurtaSearchPanel";
 import { PanchangPanel } from "./PanchangPanel";
 import { ShadbalaPanel } from "./ShadbalaPanel";
 import type { PanelDefinition } from "./types";
@@ -46,6 +47,7 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   compat: { titleKey: "panel.compat", component: CompatPanel },
   // Muhurta (Plan 20).
   muhurta_day: { titleKey: "panel.muhurta_day", component: MuhurtaDayPanel },
+  muhurta_search: { titleKey: "panel.muhurta_search", component: MuhurtaSearchPanel },
 };
 
 const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };
````


- [ ] **Step 4: Arama kancası**

**Yeni dosya:** `app/ui/modules/useMuhurtaSearch.ts`

````ts
// Muhurta aramasını (`muhurta_search`) yürüten kanca (ileri modüller spec'i §10). Arama dilim
// dilim ilerler: her yanıtın imleci bir sonraki isteğe verilir; aramayı durdurmak sonraki dilimi
// istememektir (Rust durum tutmaz). Pencereler geldikçe listeye eklenir ve yeni arama başlayana
// dek yerinde durur. Arayüzde hesap yoktur (C4); sonuç saklanmaz (K1). Sorgu yer ve doğum Ay'ı
// taşır: loglanmaz.
import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api";
import type { CalcSettings, ChartWarning, CommandError, MuhurtaPageCursor, MuhurtaQuery, MuhurtaSearchView, MuhurtaWindowView } from "../bindings";

export type SearchStatus = "idle" | "running" | "done" | "stopped" | "error";

export interface MuhurtaSearchState {
  status: SearchStatus;
  windows: MuhurtaWindowView[];
  /** Taranan dilim ve toplam; ilk yanıttan önce 0. */
  scanned: number;
  chunks: number;
  /** Taranan aralık (ilk yanıtla gelir). */
  range: Pick<MuhurtaSearchView, "from_jd_ut" | "to_jd_ut" | "from_offset_secs" | "to_offset_secs"> | null;
  warnings: ChartWarning[];
  method: string | null;
  error: CommandError | null;
}

export interface MuhurtaSearch extends MuhurtaSearchState {
  start(query: MuhurtaQuery, settings: CalcSettings): void;
  /** Sonraki dilimi istemez; gelmiş pencereler durur. */
  stop(): void;
}

const IDLE: MuhurtaSearchState = { status: "idle", windows: [], scanned: 0, chunks: 0, range: null, warnings: [], method: null, error: null };

/** `scope` değişince (başka harita) süren arama bırakılır ve sonuçlar silinir. */
export function useMuhurtaSearch(scope: string): MuhurtaSearch {
  const [state, setState] = useState<MuhurtaSearchState>(IDLE);
  // Her arama bir numara alır; eski aramanın geç gelen yanıtı ve sonraki isteği düşer.
  const run = useRef(0);

  useEffect(() => {
    return () => {
      run.current += 1;
    };
  }, []);
  const [shownScope, setShownScope] = useState(scope);
  if (shownScope !== scope) {
    setShownScope(scope);
    run.current += 1;
    setState(IDLE);
  }

  const start = useCallback((query: MuhurtaQuery, settings: CalcSettings) => {
    const mine = ++run.current;
    setState({ ...IDLE, status: "running" });
    void (async () => {
      let cursor: MuhurtaPageCursor | null = null;
      for (;;) {
        const r = await api.muhurtaSearch(query, settings, cursor);
        if (mine !== run.current) return;
        if (r.status === "error") {
          // Gelmiş pencereler durur; hata ayrıca gösterilir.
          setState((s) => ({ ...s, status: "error", error: r.error }));
          return;
        }
        const page = r.data;
        cursor = page.next;
        setState((s) => ({
          status: page.next === null ? "done" : "running",
          windows: [...s.windows, ...page.windows],
          scanned: page.scanned,
          chunks: page.chunks,
          range: { from_jd_ut: page.from_jd_ut, to_jd_ut: page.to_jd_ut, from_offset_secs: page.from_offset_secs, to_offset_secs: page.to_offset_secs },
          warnings: [...new Set([...s.warnings, ...page.warnings])],
          method: page.method,
          error: null,
        }));
        if (cursor === null) return;
      }
    })();
  }, []);

  const stop = useCallback(() => {
    run.current += 1;
    setState((s) => (s.status === "running" ? { ...s, status: "stopped" } : s));
  }, []);

  return { ...state, start, stop };
}
````


- [ ] **Step 5: Panel ve stiller**

Kayıtlı süzgeçler `useModule` ile bir kez okunur: geliştirme kipinde (`StrictMode`) effect iki kez koşar ve çıplak bir `useEffect` listeyi iki kez isterdi (e2e bunu bulmuştu).

**Yeni dosya:** `app/ui/panels/MuhurtaSearchPanel.tsx`

````tsx
// Muhurta araması paneli (ileri modüller spec'i §10): süzgeç formu, "Ara" ve sonuç listesi.
// Süzgeci astrolog kurar; hazır etkinlik şablonu, "uygun" ya da "uygunsuz" hükmü ve açıklama
// metni yoktur. Koşullar panelin ayarıdır (düzenle birlikte saklanır) ve adlandırılıp
// kaydedilebilir; kayıt yer, tarih ve kişi taşımaz. Pencereler `muhurta_search`'ten gelir;
// burada hesap yoktur (C4). Satırdaki başlangıç, transit tarihini o ana götürür.
import { useId, useLayoutEffect, useRef, useState, type FormEvent, type RefObject } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError, MuhurtaCriteria, MuhurtaWindowView, OpenedClient, SavedMuhurtaFilter } from "../bindings";
import { signIndex, SIGNS } from "../chart/adapter";
import { errorMessage } from "../i18n/errors";
import type { Panel } from "../layout/model";
import { MAX_MIN_MINUTES, MUHURTA_FLAGS, muhurtaPlace, natalMoon, parseCriteria, toggleMember, VARAS } from "../modules/muhurta";
import { useModule } from "../modules/useModule";
import { useMuhurtaSearch, type MuhurtaSearch } from "../modules/useMuhurtaSearch";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useTransitControl } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { Approx } from "./Approx";
import { formatJdClock, formatJdDate, formatJdSpan } from "./format";
import { nakshatraName, nityaYogaName, tithiName, tithiShort } from "./labels";
import { PlaceControls, PlaceMissing, usePanelPlace } from "./MuhurtaPlace";
import "./panels.css";
import { PanelError, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Listede çizilen en çok pencere; fazlası sayılır ve süzgecin daraltılması önerilir. */
export const MAX_SHOWN_WINDOWS = 500;
const TITHIS = Array.from({ length: 30 }, (_, i) => i + 1);
const NAKSHATRAS = Array.from({ length: 27 }, (_, i) => i + 1);

export function MuhurtaSearchPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Search panel={panel} opened={opened} />;
}

/** Onay kutusu kümesi: görünür başlık ve her değer için etiketli bir kutu. */
function CheckGroup<T extends string | number>({
  legend,
  options,
  selected,
  onToggle,
  fold,
}: {
  legend: string;
  options: readonly { value: T; label: string }[];
  selected: readonly T[];
  onToggle(value: T): void;
  /** Uzun kümeler açılır kapanır bir bölümde durur; başlık seçili sayıyı söyler. */
  fold?: string;
}) {
  const boxes = (
    <fieldset className="muhurta-checks">
      <legend className={fold ? "visually-hidden" : undefined}>{legend}</legend>
      {options.map((o) => (
        <label key={o.value}>
          <input type="checkbox" checked={selected.includes(o.value)} onChange={() => onToggle(o.value)} />
          {o.label}
        </label>
      ))}
    </fieldset>
  );
  if (!fold) return boxes;
  return (
    <details className="muhurta-fold">
      <summary>{fold}</summary>
      {boxes}
    </details>
  );
}

function Search({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  const place = usePanelPlace(panel, opened);
  const criteria = parseCriteria(panel.settings.criteria);
  const search = useMuhurtaSearch(String(opened.chart_record.id));
  const running = search.status === "running";
  const phase = usePanelLoadPhase(running);
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const searchButton = useRef<HTMLButtonElement>(null);
  const setCriteria = (patch: Partial<MuhurtaCriteria>) => setPanelSettings(panel.id, { criteria: { ...criteria, ...patch } });
  const unknown = timeUnknown(opened);
  const ready = place.place !== null;

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (place.place === null || running) return;
    search.start(
      { from_date: from === "" ? null : from, to_date: to === "" ? null : to, place: muhurtaPlace(place.place), criteria, natal: natalMoon(opened) },
      opened.chart.meta.settings,
    );
  };

  const count = (n: number) => t(n === 0 ? "muhurtaSearch.any" : "muhurtaSearch.selected", { n });
  return (
    <div className="module-panel muhurta-search">
      <form className="muhurta-form" aria-label={t("muhurtaSearch.form")} onSubmit={submit}>
        <SavedFilters criteria={criteria} onLoad={(c) => setPanelSettings(panel.id, { criteria: c })} />
        <div className="module-toolbar">
          <PlaceControls panel={panel} opened={opened} at={place} />
        </div>
        <div className="module-toolbar">
          <label htmlFor={`${id}-from`}>{t("muhurtaSearch.from")}</label>
          <input id={`${id}-from`} type="date" value={from} aria-describedby={`${id}-range`} onChange={(e) => setFrom(e.target.value)} />
          <label htmlFor={`${id}-to`}>{t("muhurtaSearch.to")}</label>
          <input id={`${id}-to`} type="date" value={to} aria-describedby={`${id}-range`} onChange={(e) => setTo(e.target.value)} />
          <span id={`${id}-range`} className="chart-toolbar-hint">
            {t("muhurtaSearch.rangeHint")}
          </span>
        </div>
        <p className="chart-toolbar-hint">{t("muhurtaSearch.emptySetHint")}</p>
        <CheckGroup
          legend={t("muhurtaSearch.weekdays")}
          options={VARAS.map((v) => ({ value: v, label: t(`panchang.weekday.${v}`) }))}
          selected={criteria.weekdays}
          onToggle={(v) => setCriteria({ weekdays: toggleMember(criteria.weekdays, v, VARAS) })}
        />
        <CheckGroup
          legend={t("muhurtaSearch.tithis")}
          fold={`${t("muhurtaSearch.tithis")} · ${count(criteria.tithis.length)}`}
          options={TITHIS.map((n) => ({ value: n, label: `${tithiShort(n)} ${tithiName(n)}` }))}
          selected={criteria.tithis}
          onToggle={(n) => setCriteria({ tithis: toggleMember(criteria.tithis, n, TITHIS) })}
        />
        <CheckGroup
          legend={t("muhurtaSearch.nakshatras")}
          fold={`${t("muhurtaSearch.nakshatras")} · ${count(criteria.nakshatras.length)}`}
          options={NAKSHATRAS.map((n) => ({ value: n, label: nakshatraName(n) }))}
          selected={criteria.nakshatras}
          onToggle={(n) => setCriteria({ nakshatras: toggleMember(criteria.nakshatras, n, NAKSHATRAS) })}
        />
        <CheckGroup
          legend={t("muhurtaSearch.lagnaSigns")}
          fold={`${t("muhurtaSearch.lagnaSigns")} · ${count(criteria.lagna_signs.length)}`}
          options={SIGNS.map((s) => ({ value: s, label: t(`sign.${signIndex(s)}`) }))}
          selected={criteria.lagna_signs}
          onToggle={(s) => setCriteria({ lagna_signs: toggleMember(criteria.lagna_signs, s, SIGNS) })}
        />
        <CheckGroup
          legend={t("muhurtaSearch.exclude")}
          options={MUHURTA_FLAGS.map((f) => ({ value: f, label: t(`muhurta.flag.${f}`) }))}
          selected={criteria.exclude}
          onToggle={(f) => setCriteria({ exclude: toggleMember(criteria.exclude, f, MUHURTA_FLAGS) })}
        />
        <div className="module-toolbar">
          <label>
            <input type="checkbox" checked={criteria.daytime_only} onChange={(e) => setCriteria({ daytime_only: e.target.checked })} />
            {t("muhurtaSearch.daytimeOnly")}
          </label>
          <label htmlFor={`${id}-min`}>{t("muhurtaSearch.minMinutes")}</label>
          <input
            id={`${id}-min`}
            type="number"
            min={0}
            max={MAX_MIN_MINUTES}
            step={1}
            value={criteria.min_minutes}
            onChange={(e) => {
              const n = Math.floor(Number(e.target.value));
              setCriteria({ min_minutes: Number.isFinite(n) ? Math.min(Math.max(n, 0), MAX_MIN_MINUTES) : 0 });
            }}
          />
          <label>
            <input type="checkbox" checked={criteria.require_tara} onChange={(e) => setCriteria({ require_tara: e.target.checked })} />
            <Approx on={unknown} label={t("muhurtaSearch.natalApprox")}>
              {t("muhurtaSearch.requireTara")}
            </Approx>
          </label>
          <label>
            <input type="checkbox" checked={criteria.require_chandra} onChange={(e) => setCriteria({ require_chandra: e.target.checked })} />
            <Approx on={unknown} label={t("muhurtaSearch.natalApprox")}>
              {t("muhurtaSearch.requireChandra")}
            </Approx>
          </label>
        </div>
        <div className="module-toolbar">
          {/* Kullanılamazken yerel `disabled` değildir: odaklanır ve nedeni görünür bir satırdır. */}
          <button ref={searchButton} type="submit" aria-disabled={!ready || running ? true : undefined} aria-describedby={ready ? undefined : `${id}-why`}>
            {t("muhurtaSearch.run")}
          </button>
          {running && phase === "skeleton" && <StopButton onStop={search.stop} fallback={searchButton} />}
          <span className="muhurta-progress" role="status">
            {running && phase === "skeleton" && search.chunks > 0 ? t("muhurtaSearch.progress", { scanned: search.scanned, chunks: search.chunks }) : ""}
          </span>
        </div>
        {!ready && (
          <div id={`${id}-why`}>
            <PlaceMissing choice={place.choice} />
          </div>
        )}
      </form>
      {search.error && <PanelError error={search.error} onRetry={() => searchButton.current?.click()} />}
      <Results search={search} />
    </div>
  );
}

/**
 * "Durdur": arama bitince ya da durunca DOM'dan gider. Odak ondaysa gövdeye düşmesin diye
 * "Ara" düğmesine taşınır (layout effect'in temizliği düğüm kaldırılmadan önce koşar).
 */
function StopButton({ onStop, fallback }: { onStop(): void; fallback: RefObject<HTMLButtonElement | null> }) {
  const { t } = useTranslation();
  const ref = useRef<HTMLButtonElement>(null);
  useLayoutEffect(() => {
    const button = ref.current;
    return () => {
      if (button !== null && document.activeElement === button) fallback.current?.focus({ preventScroll: true });
    };
  }, [fallback]);
  return (
    <button ref={ref} type="button" onClick={onStop}>
      {t("muhurtaSearch.stop")}
    </button>
  );
}

/** Kayıtlı süzgeçler: seç, adlandırıp kaydet, sil. Liste yeniden yazılırken yerinde durur. */
function SavedFilters({ criteria, onLoad }: { criteria: MuhurtaCriteria; onLoad(criteria: MuhurtaCriteria): void }) {
  const { t, i18n } = useTranslation();
  const id = useId();
  // Liste bir kez okunur (`useModule` geliştirme kipindeki ikinci effect koşusunda yeniden
  // istemez); kayıttan sonra Rust'ın döndürdüğü kanonik liste gösterilir.
  const loaded = useModule("muhurta-filters", () => api.loadMuhurtaFilters());
  const [written, setWritten] = useState<SavedMuhurtaFilter[] | null>(null);
  const filters = written ?? loaded.data ?? [];
  const [selected, setSelected] = useState("");
  const [name, setName] = useState("");
  const [failed, setFailed] = useState<CommandError | null>(null);
  const error = failed ?? loaded.error;
  const [busy, setBusy] = useState(false);
  const select = useRef<HTMLSelectElement>(null);

  const same = (a: string, b: string) => a.trim().toLocaleLowerCase(i18n.language) === b.trim().toLocaleLowerCase(i18n.language);
  const write = async (next: SavedMuhurtaFilter[], select_: string) => {
    setBusy(true);
    const r = await api.saveMuhurtaFilters(next);
    setBusy(false);
    if (r.status === "error") {
      setFailed(r.error);
      return false;
    }
    setFailed(null);
    setWritten(r.data);
    setSelected(r.data.find((f) => same(f.name, select_))?.name ?? "");
    return true;
  };
  const save = () => {
    const trimmed = name.trim();
    if (trimmed === "" || busy) return;
    const entry = { name: trimmed, criteria };
    const at = filters.findIndex((f) => same(f.name, trimmed));
    void write(at < 0 ? [...filters, entry] : filters.map((f, i) => (i === at ? entry : f)), trimmed);
  };
  const remove = async () => {
    if (selected === "" || busy) return;
    const done = await write(
      filters.filter((f) => f.name !== selected),
      "",
    );
    // "Sil" seçim boşalınca kullanılamaz olur: odak seçiciye taşınır.
    if (done) {
      setName("");
      select.current?.focus();
    }
  };

  return (
    <div className="module-toolbar muhurta-saved">
      <label htmlFor={`${id}-saved`}>{t("muhurtaSearch.saved")}</label>
      <select
        ref={select}
        id={`${id}-saved`}
        value={selected}
        onChange={(e) => {
          const filter = filters.find((f) => f.name === e.target.value);
          setSelected(filter?.name ?? "");
          setName(filter?.name ?? "");
          if (filter) onLoad(parseCriteria(filter.criteria));
        }}
      >
        <option value="">{t("muhurtaSearch.savedNone")}</option>
        {filters.map((f) => (
          <option key={f.name} value={f.name}>
            {f.name}
          </option>
        ))}
      </select>
      <label htmlFor={`${id}-name`}>{t("muhurtaSearch.name")}</label>
      <input id={`${id}-name`} type="text" value={name} maxLength={200} onChange={(e) => setName(e.target.value)} />
      <button type="button" aria-disabled={name.trim() === "" || busy ? true : undefined} onClick={save}>
        {t("muhurtaSearch.save")}
      </button>
      <button type="button" aria-disabled={selected === "" || busy ? true : undefined} onClick={() => void remove()}>
        {t("muhurtaSearch.remove")}
      </button>
      {error && (
        <span className="panel-error" role="alert">
          {errorMessage(t, i18n, error)}
        </span>
      )}
    </div>
  );
}

function Results({ search }: { search: MuhurtaSearch }) {
  const { t } = useTranslation();
  const transit = useTransitControl();
  const captionId = useId();
  if (search.status === "idle") return <p className="panel-message">{t("muhurtaSearch.idle")}</p>;
  const { windows, range } = search;
  const shown = windows.slice(0, MAX_SHOWN_WINDOWS);
  const units = { day: t("muhurtaSearch.unit.day"), hour: t("muhurtaSearch.unit.hour"), minute: t("muhurtaSearch.unit.minute") };
  const moment = (jd: number | null, offset: number) => `${formatJdDate(jd, offset)} ${formatJdClock(jd, offset, jd)}`;
  const row = (w: MuhurtaWindowView, n: number) => {
    const s = w.snapshot;
    const start = moment(w.start_jd_ut, w.start_offset_secs);
    return (
      <tr key={`${n}:${w.start_jd_ut}`} data-window={n}>
        <th scope="row">
          <button
            type="button"
            className="link-button"
            aria-label={t("muhurtaSearch.goTo", { when: start })}
            onClick={() => {
              if (w.start_jd_ut !== null) transit.setAt({ Jd: w.start_jd_ut });
            }}
          >
            {start}
          </button>
        </th>
        <td>{moment(w.end_jd_ut, w.end_offset_secs)}</td>
        <td className="num">{formatJdSpan(w.start_jd_ut, w.end_jd_ut, units)}</td>
        <td>{t(`panchang.weekday.${s.vara}`)}</td>
        <td>{`${tithiShort(s.tithi)} ${tithiName(s.tithi)}`}</td>
        <td>{`${nakshatraName(s.nakshatra)} ${s.pada}`}</td>
        <td>{nityaYogaName(s.yoga)}</td>
        <td>{s.karana_name}</td>
        <td>{t(`sign.${signIndex(s.lagna_sign)}`)}</td>
      </tr>
    );
  };
  return (
    <>
      <p className="panel-message" data-summary>
        {range
          ? t(`muhurtaSearch.summary.${search.status}`, {
              n: windows.length,
              from: formatJdDate(range.from_jd_ut, range.from_offset_secs),
              // Aralığın sonu son günün bitişidir: bir saniye öncesi o günün tarihini verir.
              to: formatJdDate(range.to_jd_ut === null ? null : range.to_jd_ut - 1 / 86_400, range.to_offset_secs),
            })
          : t("muhurtaSearch.searching")}
      </p>
      {search.warnings.includes("SunriseUnavailable") && (
        <p className="panel-notice muhurta-warning" role="note">
          {t("muhurtaSearch.noSunrise")}
        </p>
      )}
      {search.warnings.includes("LowPrecision") && (
        <p className="panel-notice" role="note">
          <strong>{t("badge.LowPrecision")}</strong>: {t("badgeHint.LowPrecision")}
        </p>
      )}
      {shown.length > 0 && (
        // Satırların çoğu hücresinde odaklanabilir denetim yoktur: kaydırma kabı klavye odağı alır
        // ve adını tablonun başlığından okur.
        <div className="bala-scroll muhurta-results" role="group" tabIndex={0} aria-labelledby={captionId}>
          <table className="module-table muhurta-table">
            <caption id={captionId}>{t("muhurtaSearch.results")}</caption>
            <thead>
              <tr>
                <th scope="col">{t("muhurtaSearch.col.start")}</th>
                <th scope="col">{t("muhurtaSearch.col.end")}</th>
                <th scope="col" className="num">
                  {t("muhurtaSearch.col.length")}
                </th>
                <th scope="col">{t("panchang.vara")}</th>
                <th scope="col">{t("panchang.tithi")}</th>
                <th scope="col">{t("panchang.nakshatra")}</th>
                <th scope="col">{t("panchang.yoga")}</th>
                <th scope="col">{t("panchang.karana")}</th>
                <th scope="col">{t("muhurtaSearch.col.lagna")}</th>
              </tr>
            </thead>
            <tbody>{shown.map(row)}</tbody>
          </table>
        </div>
      )}
      {windows.length > shown.length && <p className="panel-notice">{t("muhurtaSearch.capped", { shown: shown.length, n: windows.length })}</p>}
      <p className="panel-message">{t("muhurtaSearch.note")}</p>
      {search.method && <p className="panel-message">{t("muhurta.method", { method: search.method })}</p>}
    </>
  );
}
````

````diff
diff --git a/app/ui/panels/panels.css b/app/ui/panels/panels.css
index d6f6188..a4d89e7 100644
--- a/app/ui/panels/panels.css
+++ b/app/ui/panels/panels.css
@@ -395,3 +395,38 @@
   flex-wrap: wrap;
   gap: 6px;
 }
+/* Muhurta araması: süzgeç formu ve sonuç listesi. */
+.muhurta-form {
+  display: flex;
+  flex-direction: column;
+  gap: 4px;
+}
+.muhurta-checks {
+  display: flex;
+  flex-wrap: wrap;
+  gap: 2px 12px;
+  margin: 0;
+  padding: 4px;
+  border: 1px solid var(--line);
+}
+.muhurta-checks label,
+.muhurta-form .module-toolbar label {
+  display: inline-flex;
+  align-items: center;
+  gap: 4px;
+}
+.muhurta-fold summary {
+  cursor: default;
+  color: var(--text-2);
+}
+.muhurta-progress {
+  color: var(--text-2);
+  font-variant-numeric: tabular-nums;
+}
+.muhurta-form button[aria-disabled="true"] {
+  opacity: 0.6;
+}
+.muhurta-results {
+  max-height: 24em;
+  overflow: auto;
+}
````


- [ ] **Step 6: Metinler (üç dil)**

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index ed167f8..adf9afc 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -74,6 +74,7 @@ export const en: Messages = {
     yogas: "Yogas",
     compat: "Compatibility",
     muhurta_day: "Day panchang",
+    muhurta_search: "Muhurta search",
     unknown: "Unknown panel",
     loading: "Loading",
     unavailable: "This panel arrives in a later version.",
@@ -453,6 +454,48 @@ export const en: Messages = {
     horaLord: "Lord",
     note: "Segments, classes and flags are facts, not a verdict on suitability. Times are in the time of the selected place.",
   },
+  muhurtaSearch: {
+    form: "Muhurta filter",
+    saved: "Saved filter",
+    savedNone: "—",
+    name: "Filter name",
+    save: "Save the filter",
+    remove: "Delete the filter",
+    from: "Start date",
+    to: "End date",
+    rangeHint: "An empty start is today; an empty end is 30 days after the start. At most 366 days.",
+    emptySetHint: "A set with no box ticked is not a constraint.",
+    any: "no constraint",
+    selected: "{{n}} selected",
+    weekdays: "Weekdays",
+    tithis: "Tithis",
+    nakshatras: "Nakshatras",
+    lagnaSigns: "Lagna signs",
+    exclude: "Excluded",
+    daytimeOnly: "Daytime only",
+    minMinutes: "Shortest length (min)",
+    requireTara: "Tara bala favourable",
+    requireChandra: "Chandra bala favourable",
+    natalApprox: "approximate: the birth time is unknown, the natal Moon is for 12:00 noon",
+    run: "Search",
+    stop: "Stop",
+    progress: "Scanned part: {{scanned}} / {{chunks}}",
+    idle: "Build the filter and press “Search”. Windows are the spans of time that meet every condition of the filter.",
+    searching: "Searching…",
+    summary: {
+      running: "{{n}} windows so far · {{from}} – {{to}}",
+      done: "{{n}} windows · {{from}} – {{to}}",
+      stopped: "Search stopped: {{n}} windows · {{from}} – {{to}}",
+      error: "The search did not finish: {{n}} windows · {{from}} – {{to}}",
+    },
+    noSunrise: "The Sun does not rise or set during part of the range: on those days the daytime and daytime segment conditions could not be applied.",
+    results: "Windows found",
+    col: { start: "Start", end: "End", length: "Length", lagna: "Lagna" },
+    unit: { day: "d", hour: "h", minute: "min" },
+    goTo: "Move the transit date to {{when}}",
+    capped: "Showing the first {{shown}} windows (of {{n}}). Narrow the filter or shorten the range.",
+    note: "The limbs and the lagna are for the moment the window starts. Windows are facts, not a verdict on suitability.",
+  },
   arudha: {
     caption: "Arudha padas: the pada of each of the twelve houses, its sign and the lord of the house",
     pada: "Pada",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 6a7b7f8..5a6dd73 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -74,6 +74,7 @@ export const fr: Messages = {
     yogas: "Yogas",
     compat: "Compatibilité",
     muhurta_day: "Panchang du jour",
+    muhurta_search: "Recherche de muhurta",
     unknown: "Panneau inconnu",
     loading: "Chargement",
     unavailable: "Ce panneau arrivera dans une version ultérieure.",
@@ -453,6 +454,48 @@ export const fr: Messages = {
     horaLord: "Maître",
     note: "Les tranches, les classes et les marques sont des faits, non un jugement de convenance. Les heures sont celles du lieu choisi.",
   },
+  muhurtaSearch: {
+    form: "Filtre de muhurta",
+    saved: "Filtre enregistré",
+    savedNone: "—",
+    name: "Nom du filtre",
+    save: "Enregistrer le filtre",
+    remove: "Supprimer le filtre",
+    from: "Date de début",
+    to: "Date de fin",
+    rangeHint: "Un début vide est aujourd'hui ; une fin vide est 30 jours après le début. Au plus 366 jours.",
+    emptySetHint: "Un ensemble sans case cochée n'est pas une contrainte.",
+    any: "aucune contrainte",
+    selected: "{{n}} choisis",
+    weekdays: "Jours",
+    tithis: "Tithis",
+    nakshatras: "Nakshatras",
+    lagnaSigns: "Signes du lagna",
+    exclude: "Exclus",
+    daytimeOnly: "De jour seulement",
+    minMinutes: "Durée minimale (min)",
+    requireTara: "Tara bala favorable",
+    requireChandra: "Chandra bala favorable",
+    natalApprox: "approximatif : l'heure de naissance est inconnue, la Lune natale est celle de midi",
+    run: "Chercher",
+    stop: "Arrêter",
+    progress: "Partie parcourue : {{scanned}} / {{chunks}}",
+    idle: "Composez le filtre puis appuyez sur « Chercher ». Les fenêtres sont les intervalles de temps qui remplissent chaque condition du filtre.",
+    searching: "Recherche…",
+    summary: {
+      running: "{{n}} fenêtres jusqu'ici · {{from}} – {{to}}",
+      done: "{{n}} fenêtres · {{from}} – {{to}}",
+      stopped: "Recherche arrêtée : {{n}} fenêtres · {{from}} – {{to}}",
+      error: "La recherche n'a pas abouti : {{n}} fenêtres · {{from}} – {{to}}",
+    },
+    noSunrise: "Le Soleil ne se lève pas ou ne se couche pas pendant une partie de l'intervalle : ces jours-là, les conditions de jour et de tranche du jour n'ont pas pu être appliquées.",
+    results: "Fenêtres trouvées",
+    col: { start: "Début", end: "Fin", length: "Durée", lagna: "Lagna" },
+    unit: { day: "j", hour: "h", minute: "min" },
+    goTo: "Placer la date du transit à {{when}}",
+    capped: "Les {{shown}} premières fenêtres sont affichées (sur {{n}}). Resserrez le filtre ou raccourcissez l'intervalle.",
+    note: "Les membres et le lagna sont ceux de l'instant où la fenêtre commence. Les fenêtres sont des faits, non un jugement de convenance.",
+  },
   arudha: {
     caption: "Arudha padas : le pada de chacune des douze maisons, son signe et le maître de la maison",
     pada: "Pada",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index aaa756d..0cf440e 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -73,6 +73,7 @@ export const tr = {
     yogas: "Yogalar",
     compat: "Uyum",
     muhurta_day: "Gün panchangı",
+    muhurta_search: "Muhurta araması",
     unknown: "Bilinmeyen panel",
     loading: "Yükleniyor",
     unavailable: "Bu panel sonraki bir sürümde gelecek.",
@@ -452,6 +453,48 @@ export const tr = {
     horaLord: "Lord",
     note: "Dilimler, sınıflar ve işaretler olgudur; uygunluk hükmü değildir. Saatler seçilen yerin saatiyledir.",
   },
+  muhurtaSearch: {
+    form: "Muhurta süzgeci",
+    saved: "Kayıtlı süzgeç",
+    savedNone: "—",
+    name: "Süzgeç adı",
+    save: "Süzgeci kaydet",
+    remove: "Süzgeci sil",
+    from: "Başlangıç tarihi",
+    to: "Bitiş tarihi",
+    rangeHint: "Boş başlangıç bugündür; boş bitiş başlangıçtan 30 gün sonrasıdır. En çok 366 gün.",
+    emptySetHint: "Hiçbir kutusu işaretli olmayan küme kısıt değildir.",
+    any: "kısıt yok",
+    selected: "{{n}} seçili",
+    weekdays: "Günler",
+    tithis: "Tithiler",
+    nakshatras: "Nakshatralar",
+    lagnaSigns: "Lagna burçları",
+    exclude: "Dışlananlar",
+    daytimeOnly: "Yalnızca gündüz",
+    minMinutes: "En kısa süre (dk)",
+    requireTara: "Tara bala elverişli",
+    requireChandra: "Chandra bala elverişli",
+    natalApprox: "yaklaşık: doğum saati bilinmiyor, doğum Ay'ı öğlen 12:00 içindir",
+    run: "Ara",
+    stop: "Durdur",
+    progress: "Taranan bölüm: {{scanned}} / {{chunks}}",
+    idle: "Süzgeci kurup “Ara”ya basın. Pencereler süzgecin her koşulunu sağlayan zaman aralıklarıdır.",
+    searching: "Aranıyor…",
+    summary: {
+      running: "Şimdiye dek {{n}} pencere · {{from}} – {{to}}",
+      done: "{{n}} pencere · {{from}} – {{to}}",
+      stopped: "Arama durduruldu: {{n}} pencere · {{from}} – {{to}}",
+      error: "Arama tamamlanamadı: {{n}} pencere · {{from}} – {{to}}",
+    },
+    noSunrise: "Aralığın bir bölümünde Güneş doğmuyor ya da batmıyor: o günlerde gündüz ve gündüz dilimi koşulları uygulanamadı.",
+    results: "Bulunan pencereler",
+    col: { start: "Başlangıç", end: "Bitiş", length: "Süre", lagna: "Lagna" },
+    unit: { day: "g", hour: "sa", minute: "dk" },
+    goTo: "Transit tarihini {{when}} anına götür",
+    capped: "İlk {{shown}} pencere gösteriliyor (toplam {{n}}). Süzgeci daraltın ya da aralığı kısaltın.",
+    note: "Uzuvlar ve lagna pencerenin başlangıç anı içindir. Pencereler olgudur; uygunluk hükmü değildir.",
+  },
   arudha: {
     caption: "Arudha padalar: on iki evin padası, burcu ve evin lordu",
     pada: "Pada",
````


- [ ] **Step 7: Kapıları koş**

```bash
pnpm --dir app exec vitest run ui/panels/MuhurtaSearchPanel.test.tsx ui/modules/useMuhurtaSearch.test.tsx 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
git status --short | grep -vE '^( M|\?\?) app/ui/'
```
Expected: `2 passed (2)`, `24 passed (24)`; sonra `80 passed (80)`, `795 passed (795)`; `✓ built`; son komut çıktısız.

- [ ] **Step 8: Commit**

```bash
git add app/ui
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): muhurta search panel with the filter form, saved filters, paged results and a stop button" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: iki `grep` çıktısız.

---


### Task 8: Uçtan uca: Gün panchangı ve Muhurta araması panelleri

Spec §10 "Arayüz"; K12; UI spec §8 (uçtan uca). İki test WebKit'te, Tauri IPC taklidiyle koşar (Task 5'in `e2e/ipc.ts` eklemeleri): gün paneli paletten eklenir, bulunulan yer pencereden seçilir, şerit ve tablolar çizilir, hora lordu haritayı vurgular; arama paneli süzgeci kurar, üç dilimi sırayla ister, süzgeci kaydeder ve bir satır gün panelini o ana götürür.

**Files:**
- Modify: `app/e2e/desk.spec.ts`

**Interfaces:**
- Consumes: Task 5–7'nin tamamı; `mockIpc`, `ipcCalls`, `FIXTURE`.
- Produces: iki e2e testi (toplam 22).

- [ ] **Step 1: Testleri yaz**

WebKit tıklanan düğmeye odak vermez: odağın açan düğmeye dönüşünü sınayan adımlar düğmeyi klavyeyle çalıştırır.

````diff
diff --git a/app/e2e/desk.spec.ts b/app/e2e/desk.spec.ts
index 31b419a..7f3f919 100644
--- a/app/e2e/desk.spec.ts
+++ b/app/e2e/desk.spec.ts
@@ -801,3 +801,145 @@ test("compat with a partner whose birth time is unknown: the report is marked ap
   await expect(sun.locator("th, td")).toHaveText(["Gü", "lagnadan — · Ay'dan 4", "lagnadan 10 · Ay'dan 6"]);
   await expect(compat.locator('.compat-facts li[data-fact="kuja-Bride"]')).toHaveText("Kuja dosha, Ayşe Yılmaz: lagnadan bilinmiyor, Ay'dan var, Venüs'ten var.");
 });
+
+test("muhurta day: a palette panel; the current place from the dialog, the day strip, the limbs and the hora table", async ({ page }) => {
+  await mockIpc(page, { session: true });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  const toggle = async () => {
+    await page.keyboard.press("Meta+k");
+    const palette = page.getByRole("dialog", { name: "Komut paleti" });
+    await palette.getByRole("combobox").fill("gün panchang");
+    await palette.getByRole("option", { name: /Gün panchangı panelini aç\/kapat/ }).click();
+    await expect(page.getByRole("dialog")).toHaveCount(0);
+  };
+  type DayCall = { at: unknown; place: { lat: number; lon: number; tz: unknown }; natal: unknown };
+  const dayCalls = async () => (await ipcCalls(page, "day_panchang")) as DayCall[];
+  type Saved = { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: Record<string, unknown> }[] }[] } } };
+  const savedPanel = async () => {
+    const layout = ((await ipcCalls(page, "save_view_state")).at(-1) as Saved | undefined)?.viewState.panel_layout;
+    return layout?.layouts.find((l) => l.id === layout.active_layout_id)?.panels.find((p) => p.kind === "muhurta_day");
+  };
+
+  await toggle();
+  const day = page.getByRole("region", { name: "Gün panchangı" });
+  await expect(day).toBeFocused();
+  await expect(day).toBeInViewport();
+  // Bulunulan yer seçilmemiş: panel yer ister, gün hesaplanmaz (ileri modüller spec'i K1).
+  await expect(day.getByRole("note")).toContainText("Bulunulan yer seçilmedi.");
+  expect(await dayCalls()).toHaveLength(0);
+
+  // Bulunulan yer penceresi: açık haritanın doğum yeri önerilir; seçim tercihlere yazılır.
+  // Klavyeyle açılır (WebKit tıklanan düğmeye odak vermez): pencere kapanınca odak düğmeye döner.
+  await day.getByRole("button", { name: "Bulunulan yeri seç" }).focus();
+  await page.keyboard.press("Enter");
+  const dialog = page.getByRole("dialog", { name: "Bulunulan yer" });
+  await dialog.getByRole("button", { name: /^Açık haritanın doğum yerini kullan/ }).click();
+  await expect(dialog.locator(".home-place-current")).toHaveText("Seçili yer: New Delhi, Delhi, India · Asia/Kolkata");
+  const prefs = (await ipcCalls(page, "set_app_prefs")).at(-1) as { prefs: { home_place: unknown } };
+  expect(prefs.prefs.home_place).toEqual(FIXTURE.muhurta.home_place);
+  await page.keyboard.press("Escape");
+  await expect(page.getByRole("dialog")).toHaveCount(0);
+  await expect(day.getByRole("button", { name: "Bulunulan yeri değiştir" })).toBeFocused();
+
+  // Gün: şerit, uzuvlar, kişiye bağlı etkenler ve hora tablosu; komuta yerin adı gitmez.
+  await expect(day.getByText("2026-11-25 · Budhavara · Çarşamba")).toBeVisible();
+  const strip = day.getByRole("list", { name: "Gündüzün sekiz dilimi" });
+  await expect(strip.getByRole("listitem")).toHaveCount(8);
+  await expect(strip.locator("li.is-marked")).toHaveText(["2Yama08:13–09:31", "4Guli10:49–12:07", "5Rahu12:07–13:26"]);
+  await expect(day.getByText(/^Abhijit 11:47–12:28/)).toBeVisible();
+  await expect(day.getByRole("table", { name: "Beş uzuv ve değişim saatleri" }).locator("tbody tr")).toHaveCount(12);
+  await expect(day.getByRole("table", { name: "Açık haritanın Ay'ına göre" }).locator("tbody tr")).toHaveCount(3);
+  const horas = day.getByRole("table", { name: "Hora tablosu" });
+  await expect(horas.locator("tbody tr")).toHaveCount(24);
+  const calls = await dayCalls();
+  expect(calls).toHaveLength(1);
+  expect(calls[0].at).toBe("Now");
+  expect(calls[0].place).toEqual(FIXTURE.muhurta.place);
+  expect(calls[0].natal).toEqual(FIXTURE.muhurta.natal);
+
+  // Ortak vurgu: hora lordu haritadaki grahayı vurgular.
+  await horas.getByRole("button", { name: "Jüpiter" }).first().click();
+  await expect(page.locator('svg [data-graha="Jupiter"]').first()).toHaveClass(/chart-planet--highlight/);
+  await expect(horas.locator("tr.is-highlighted")).toHaveCount(3);
+  await page.keyboard.press("Escape");
+
+  // Yer seçimi ve tarih bağı panelin ayarıdır: düzenle birlikte kaydedilir.
+  await day.getByRole("combobox", { name: "Yer" }).selectOption("chart");
+  await day.getByRole("combobox", { name: "Tarih bağı" }).selectOption("today");
+  await expect(day.getByRole("form", { name: "Transit anı" })).toHaveCount(0);
+  await expect.poll(async () => (await savedPanel())?.settings).toEqual({ place: "chart", link: "today" });
+
+  // Aynı komut paneli kaldırır ve odak gövdeye düşmez.
+  await toggle();
+  await expect(day).toHaveCount(0);
+  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
+});
+
+test("muhurta search: the filter form, paged windows, a saved filter, and a row that moves the day panel to that moment", async ({ page }) => {
+  await mockIpc(page, { session: true });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  const open = async (query: string, name: RegExp) => {
+    await page.keyboard.press("Meta+k");
+    const palette = page.getByRole("dialog", { name: "Komut paleti" });
+    await palette.getByRole("combobox").fill(query);
+    await palette.getByRole("option", { name }).click();
+    await expect(page.getByRole("dialog")).toHaveCount(0);
+  };
+  type SearchCall = { query: { from_date: string | null; to_date: string | null; place: unknown; criteria: Record<string, unknown>; natal: unknown }; cursor: { cursor: { chunk: number } } | null };
+  const searchCalls = async () => (await ipcCalls(page, "muhurta_search")) as SearchCall[];
+
+  await open("gün panchang", /Gün panchangı panelini aç\/kapat/);
+  const day = page.getByRole("region", { name: "Gün panchangı" });
+  await day.getByRole("combobox", { name: "Yer" }).selectOption("chart");
+  await expect(day.getByText("2026-11-25 · Budhavara · Çarşamba")).toBeVisible();
+  await open("muhurta ara", /Muhurta araması panelini aç\/kapat/);
+  const search = page.getByRole("region", { name: "Muhurta araması" });
+  await expect(search).toBeFocused();
+  // Panel açılınca arama yapılmaz; kayıtlı süzgeçler bir kez okunur.
+  expect(await searchCalls()).toHaveLength(0);
+  expect(await ipcCalls(page, "load_muhurta_filters")).toHaveLength(1);
+  await expect(search.getByText(/^Süzgeci kurup “Ara”ya basın\./)).toBeVisible();
+
+  // Süzgeç: yer, gün, gündüz, dışlanan dilim; koşullar görünür etiketli kutulardır.
+  await search.getByRole("combobox", { name: "Yer" }).selectOption("chart");
+  await search.getByRole("checkbox", { name: "Perşembe" }).check();
+  await search.getByRole("checkbox", { name: "Yalnızca gündüz" }).check();
+  await search.getByRole("checkbox", { name: "Rahu dilimi" }).check();
+  await search.getByText("Tithiler · kısıt yok").click();
+  await search.getByRole("checkbox", { name: "S2 Dwitiya" }).check();
+  await expect(search.getByText("Tithiler · 1 seçili")).toBeVisible();
+  await search.getByRole("checkbox", { name: "S2 Dwitiya" }).uncheck();
+
+  await search.getByRole("button", { name: "Ara" }).click();
+  await expect(search.locator("[data-summary]")).toHaveText("30 pencere · 2026-11-26 – 2027-01-31");
+  const results = search.getByRole("table", { name: "Bulunan pencereler" });
+  await expect(results.locator("tbody tr")).toHaveCount(30);
+  await expect(results.locator("tbody tr").first().locator("th, td")).toHaveText(["2026-11-26 08:14", "2026-11-26 09:32", "1 sa 17 dk", "Perşembe", "K2 Dwitiya", "Mrigashira 3", "Sadhya", "Garaja", "Akr"]);
+  // Üç dilim, her biri bir öncekinin imleciyle; tarihler boş (bugünden başlar), yerin adı gitmez.
+  const calls = await searchCalls();
+  expect(calls.map((c) => c.cursor?.cursor.chunk ?? null)).toEqual([null, 1, 2]);
+  expect(calls[0].query).toMatchObject({ from_date: null, to_date: null, place: FIXTURE.muhurta.place, natal: FIXTURE.muhurta.natal });
+  expect(calls[0].query.criteria).toMatchObject({ weekdays: ["Thursday"], daytime_only: true, exclude: ["RahuSegment"], tithis: [] });
+
+  // Süzgeç adlandırılıp kaydedilir: kayıt yalnızca ad ve koşul taşır.
+  await search.getByRole("textbox", { name: "Süzgeç adı" }).fill("Perşembe gündüzü");
+  await search.getByRole("button", { name: "Süzgeci kaydet" }).click();
+  await expect(search.getByRole("combobox", { name: "Kayıtlı süzgeç" })).toHaveValue("Perşembe gündüzü");
+  const savedFilters = (await ipcCalls(page, "save_muhurta_filters")).at(-1) as { filters: Record<string, unknown>[] };
+  expect(savedFilters.filters).toHaveLength(1);
+  expect(Object.keys(savedFilters.filters[0]).sort()).toEqual(["criteria", "name"]);
+
+  // Satırın başlangıcı transit tarihini o ana götürür: gün paneli o anı ister.
+  const before = (await ipcCalls(page, "day_panchang")).length;
+  const first = results.locator("tbody tr").first().getByRole("button");
+  await first.focus();
+  await page.keyboard.press("Enter");
+  await expect.poll(async () => (await ipcCalls(page, "day_panchang")).length).toBe(before + 1);
+  const moved = (await ipcCalls(page, "day_panchang")).at(-1) as { at: { Jd: number } };
+  expect(moved.at).toEqual({ Jd: FIXTURE.muhurta.pages[0].windows[0].start_jd_ut });
+  // Sonuç listesi yerinde durur ve odak satırdadır.
+  await expect(results.locator("tbody tr")).toHaveCount(30);
+  await expect(first).toBeFocused();
+});
````


- [ ] **Step 2: Testleri koş**

```bash
lsof -i :1421
pnpm --dir app build 2>&1 | grep "✓ built"
pnpm --dir app e2e 2>&1 | tail -1
rm -rf app/test-results
```
Expected: `lsof` çıktısız; `✓ built` (`tsc -p e2e` hatasız); `22 passed`.

- [ ] **Step 3: Commit**

```bash
git add app/e2e/desk.spec.ts
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(e2e): day panchang and muhurta search panels from the palette; home place, paged windows and the transit link" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: `grep` çıktısız.

---


### Task 9: Son doğrulama, elle denetim kapısı, el örneklerinin gözden geçirilmesi ve klasik tabloların onayı

Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle çözülür ve bu görev baştan koşulur. Tek dosya değişikliği, `REVIEW.md`'deki iki kayıt tablosuna yazılan satırlardır (Step 5 ve Step 6).

**Files:**
- Modify: `tests/golden/modules/muhurta/REVIEW.md` (yalnızca iki kayıt tablosu). İlerleme notları depo dışıdır; `git add` edilmez.

**Interfaces:**
- Consumes: Task 1–8'in tamamı.
- Produces: yeşil kapılar, kural ve gizlilik taramaları, elle denetim sonucu, sahibine devredilen maddeler, el örneklerinin bağımsız gözden geçirmesi ve klasik tabloların onay kaydı (`REVIEW.md`'ye ve ilerleme notlarına yazılır).

- [ ] **Step 1: Rust kapıları**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test -p jyotish-desk --test modules -- --nocapture 2>&1 | grep -E "median|muhurta_search"
cargo test -p jyotish-desk --test open_client -- --nocapture 2>&1 | grep -E "median"
cargo deny check 2>&1 | tail -1
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p jyotish-core -p atlas -p store -p jyotish-desk --all-targets 2>&1 | tail -1
git diff ff3da00..HEAD --stat -- deny.toml Cargo.toml Cargo.lock app/package.json app/pnpm-lock.yaml app/vite.config.ts app/src-tauri/capabilities app/src-tauri/tauri.conf.json app/src-tauri/Cargo.toml crates/store/src/schema.rs crates/store/Cargo.toml crates/atlas/Cargo.toml crates/jyotish-core/Cargo.toml crates/jyotish-core/src/settings.rs crates/jyotish-core/src/chart.rs crates/jyotish-core/src/positions.rs crates/jyotish-core/src/kuta crates/jyotish-core/src/yogas crates/jyotish-core/src/shadbala crates/jyotish-core/src/jaimini crates/jyotish-core/src/dasha.rs crates/store/src/view_state.rs crates/store/src/charts.rs
git diff ff3da00..HEAD --numstat -- crates/jyotish-core/src/panchang.rs
ls crates/jyotish-core/proptest-regressions 2>/dev/null
```
Expected: fmt ve clippy çıktısız; testler: `jyotish-core` lib `351 passed`, golden `8 passed`, `modules_golden` `15 passed`; `atlas` `moment_fixed_lmt` `11 passed`, `moment_iana` `13 passed`; `store` lib `103 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `49 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `modules` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; on `median` satırı ve üç `muhurta_search` satırı, hepsi bütçesinin içinde (D26); `open_client median` 50 ms'nin altında (K1: `open_client` değişmedi); deny `advisories ok, bans ok, licenses ok, sources ok`; MSRV `Finished`; ilk `git diff --stat` çıktısız (bağımlılık, yetenek, yapılandırma, SQLite şeması, ayar tipi, `Chart`, `Positions`, önceki modüller ve `ViewState` değişmedi: K1, C3); `panchang.rs` `2	2` yazar (iki görünürlük satırı: C14); `ls` çıktısız. Taban `ff3da00` değilse komutlardaki commit, Task 1 adım 1'de not edilen tabandır.

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
Expected: `lsof` çıktısız; `Test Files  80 passed (80)`, `Tests  795 passed (795)`; `✓ built`; e2e `22 passed`; görsel `10 passed` (temel PNG'ler değişmedi: D37); betik testleri `pass 14`, `fail 0`; `git status --short` boş.

- [ ] **Step 3: Kural ve gizlilik taramaları (`ff3da00`'dan bu yana)**

```bash
git diff ff3da00..HEAD -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log ff3da00..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff ff3da00..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
git diff ff3da00..HEAD --name-only | grep -vE '^(app|crates|scripts|tests|docs)/'
echo "trailers: $(git log ff3da00..HEAD --format=%B | grep -c '^Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>$') commits: $(git rev-list --count ff3da00..HEAD)"
git diff ff3da00..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
git diff ff3da00..HEAD -U0 -- crates app/src-tauri/src | grep '^+' | grep -E 'println!|eprintln!|dbg!|log::|tracing::'
git diff ff3da00..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\(|window\.(print|open)\('
git diff ff3da00..HEAD -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
grep -rn '"source"' tests/golden/modules | grep -v '"source": "hand"'
ls tests/golden/modules/muhurta/*.json | wc -l
grep -rnE '#\[serde\(default' crates/jyotish-core/src crates/store/src/muhurta.rs crates/store/src/model.rs app/src-tauri/src/muhurta_commands.rs
grep -rnE 'XMLHttpRequest|WebSocket|\bfetch\(' app/ui/modules app/ui/panels
grep -rnE 'localStorage|sessionStorage|new Date\(|Date\.now' app/ui/modules/muhurta.ts app/ui/modules/useDayPanchang.ts app/ui/modules/useMuhurtaSearch.ts app/ui/panels/MuhurtaDayPanel.tsx app/ui/panels/MuhurtaSearchPanel.tsx app/ui/panels/MuhurtaPlace.tsx app/ui/panels/HomePlaceDialog.tsx
grep -cE 'muhurta_commands::(day_panchang|muhurta_search)|store_commands::(load|save)_muhurta_filters' app/src-tauri/src/lib.rs
grep -rnE 'InvalidLocation|unknown_time_zone' crates/jyotish-core/src/timeline.rs crates/jyotish-core/src/muhurta app/src-tauri/src/muhurta_commands.rs
git diff ff3da00..HEAD --stat -- app/ui/chart app/ui/print app/e2e/visual.spec.ts-snapshots
git tag --points-at HEAD
```
Expected: `trailers:` satırındaki iki sayı eşit; `grep -ciE` satırı `0`, `ls … | wc -l` satırı `7`, `grep -cE` satırı `4` yazar (dört komut kayıtlıdır); bunların dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur): yasaklı ad yok (üretilen fikstürde de), beklenmeyen yerde dosya yok, arayüzde `console` ve Rust'ta log çağrısı yok (yer, sorgu ve doğum Ay'ı loglanmaz: C1), yasak çağrı yok, yeni stilde hex ya da `rgb(` yok (C9), el örneklerinde `hand` dışında kaynak yok (K9), IPC tiplerinde `serde(default)` yok, muhurta kodunda ağ çağrısı, tarayıcı deposu ve saat okuması yok ("şimdi" Rust'ta okunur: C4), muhurta yolları konumu ya da saat dilimini hata parametresine yazan türleri üretmez (D20), `ChartSvg`, baskı ve görsel temeller değişmedi (K10, D37), ajan etiket atmamış.

- [ ] **Step 4: Elle denetim kapısı (`pnpm --dir app tauri dev`)**

Gerçek uygulamada, **yalnızca deneme danışanlarıyla** (gerçek danışan verisi ve gerçek ev adresi kullanılmaz; bulunulan yer olarak bir kent seçilir). Her madde ilerleme notlarına "geçti" ya da bulgu olarak yazılır; bulgu varsa düzeltme commit'i ve bu görev baştan. Her maddenin başında, otomatik bir karşılığı olup olmadığı yazılıdır. **[otomatik]** maddelerin karşılığı testlerde vardır ve yürütücü ajan bunları testin adını göstererek "geçti" sayabilir; **[insan]** maddeler yalnızca gerçek uygulamaya bakan bir insan tarafından doğrulanabilir ve sahibi yoksa ona devredilir (bu devir `git push`'u engellemez; hiçbir şey yayımlanmaz).

- [ ] **[otomatik]** Komut paletinde "Gün panchangı panelini aç/kapat" ve "Muhurta araması panelini aç/kapat" komutları yalnızca harita açıkken vardır; panel etkin düzene eklenir ve odak ona geçer; aynı komut paneli çıkarır ve odak gövdeye düşmez; panel açılınca arama yapılmaz, yer seçilmemişken gün hesaplanmaz. (e2e `muhurta day: …`, `muhurta search: …`; `layout/model.test.ts`)
- [ ] **[otomatik]** Bulunulan yer seçilmemişse panel yer ister; pencere atlastan arar, açık haritanın doğum yerini önerir, yeri kaldırır; kaldırınca ve pencere kapanınca odak düşmez; yer tercihlere yazılır ve dışa aktarımda yoktur. (`HomePlaceDialog.test.tsx`, `MuhurtaDayPanel.test.tsx › without a current place …`, `settings.rs › home_place_round_trips_…`, `exchange.rs › the_home_place_and_saved_muhurta_filters_are_never_exported`, e2e)
- [ ] **[otomatik]** Gün paneli şeridi (sekiz dilim; işaretli üçü yazı ve desenle), Abhijit'i (Çarşamba notuyla), beş uzvu sınıflarıyla, işaretleri, tara ve chandra balayı ve 24 horayı çizer; günün dışına taşan uçlar işaretlidir; hüküm sözü yoktur. (`MuhurtaDayPanel.test.tsx › draws the day …`, e2e)
- [ ] **[otomatik]** Tarih bağı transit anını izler ya da bugünde durur; "Şimdi" yeniden seçilince gün yeniden okunur; yeni gün okunurken eski gün ve odak yerinde durur. (`MuhurtaDayPanel.test.tsx › the date follows …`, `› choosing now again …`, `› while another day is read …`)
- [ ] **[otomatik]** Gün doğumu olmayan günde rozet çıkar, dilimler ve hora tablosu çizilmez, uzuvlar ve işaretler çizilir. (`MuhurtaDayPanel.test.tsx › a day without a sunrise …`; çekirdek `without_a_sunrise_the_day_starts_at_six_…`; Rust `modules` testi, Tromsø)
- [ ] **[otomatik]** Arama paneli süzgeci görünür etiketli kutularla kurar, koşulları düzenle saklar, dilimleri sırayla ister, pencereleri geldikçe listeler; bir saniyeden sonra ilerleme ve "Durdur" çıkar; durdurunca pencereler ve odak durur; satırın başlangıcı transit tarihini o ana götürür. (`MuhurtaSearchPanel.test.tsx`, `useMuhurtaSearch.test.tsx`, e2e)
- [ ] **[otomatik]** Süzgeç adlandırılıp kaydedilir, üzerine yazılır, silinir (odak seçiciye geçer); kayıt yalnızca ad ve koşul taşır. (`MuhurtaSearchPanel.test.tsx › a saved filter …`, `› deleting the selected filter …`; `store/muhurta.rs › a_saved_filter_holds_a_name_and_criteria_and_nothing_personal`)
- [ ] **[otomatik]** Aramanın her penceresi her anında süzgeci sağlar, pencere arası sağlamaz, dilim uzunluğu sonucu değiştirmez; diziler v1 panchangıyla aynı değeri verir; girişlerde boylam sınırın 1″ içindedir; bütçeler tutar. (çekirdek `muhurta_properties`, `timeline_properties`; Rust `modules` testi)
- [ ] **[otomatik]** Doğum saati bilinmeyen haritada kişiye bağlı değerler "≈" ile işaretlidir; günün panchangı değişmez. (`MuhurtaDayPanel.test.tsx › an unknown birth time …`, `MuhurtaSearchPanel.test.tsx › an unknown birth time …`)
- [ ] **[otomatik]** Hatalar yalnızca alan adı taşır; yer ve tarih hata parametresine girmez. (`muhurta_commands.rs › errors_name_the_field_…`, Rust `modules` testi, `commands.rs`)
- [ ] **[otomatik]** Her işaretin, doğanın ve taranın üç dilde adı vardır; klasik adlar üç dilde aynıdır. (`MuhurtaDayPanel.test.tsx › every flag, nature and tara …`, `MuhurtaSearchPanel.test.tsx › the search texts …`, `i18n.test.ts`)
- [ ] **[insan]** Gerçek veritabanıyla tam akış: bulunulan yer seçilir, uygulama yeniden başlatılınca yer ve iki panelin ayarları (yer seçimi, tarih bağı, etkin süzgeç) geri gelir; kayıtlı süzgeç yeniden başlatmadan sonra listededir; JSON dışa aktarım dosyasında yerin adı ve `home_place` geçmez.
- [ ] **[insan]** Gerçek motorla bugünün panchangı, astrologun elindeki basılı ya da güvendiği bir panchangla (bulunulan yer için) karşılaştırılır: gün doğumu ve batımı, tithi, nakshatra, yoga ve karana bitiş saatleri (± 1–2 dakika; gün doğumu tanımı ayardadır), Rahu dilimi, Yamaganda, Gulika ve Abhijit saatleri. Uyuşmazlık bulgu olarak yazılır; tablo farkıysa Step 6'ya, okuma farkıysa (gerçek gün doğumuna göre orantılı dilim, 24 eşit hora, kırpılmış uçlar) Kararlar'a göre sınıflanır.
- [ ] **[insan]** Gerçek motorla arama: 366 günlük, lagna koşullu bir arama akıcıdır (ilerleme görünmeden biter ya da ilerler ve durdurulabilir); arama sürerken başka bir danışan açılabilir ve arama bırakılır; bir satıra basınca transit tablosu ve gün paneli o ana gider.
- [ ] **[insan]** Yaz saati uygulayan bir yerde (ör. Berlin) geçiş gününü içeren bir aramada pencere saatleri geçişin iki yanında doğru ofsetle yazılır; gün panelinde geçiş gününün saatleri en çok bir saat kayıktır (D24) ve bu kabul edilebilir bulunur ya da bulgu yazılır.
- [ ] **[insan]** Paneller açık ve koyu temada, Türkçe, İngilizce ve Fransızca dilde okunur; dar panelde tablolar yatay kaydırılır ve şerit taşmaz; işaretli dilimler ve uyarılar renk olmadan da ayırt edilir.
- [ ] **[insan]** VoiceOver: şerit "Gündüzün sekiz dilimi" listesi olarak ve dilimlerin işaretiyle okunur; tabloların başlığı ve sütun adları okunur; kırpılmış uçların ve "≈" işaretinin sözcük karşılığı okunur; ilerleme satırı duyurulur; onay kutusu kümeleri başlıklarıyla okunur.
- [ ] **[insan]** Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum verisi, bulunulan yer, koordinat ya da dosya yolu görünmez.

Kapı geçtiyse ilerleme notlarına `Plan 20 Task 9 elle denetim: geçti (<tarih>)` yazılır; **[insan]** maddeleri devredildiyse `Plan 20 Task 9 elle denetim: otomatik maddeler geçti, insan maddeleri sahibine devredildi (<tarih>)` yazılır. Bu adım commit üretmez.

- [ ] **Step 5: El örneklerinin bağımsız gözden geçirilmesi (gözden geçirme maddesi)**

Spec §13: "`derivation` ayrı bir gözden geçirmede kuraldan yeniden hesaplanır." Gözden geçiren (motoru yazan dışında bir ajan ya da insan), `tests/golden/modules/muhurta/` altındaki yedi örneğin her birinde yalnızca `input`'tan başlar ve motoru çalıştırmadan, Rust kaynağını okumadan, spec §10'dan ve `REVIEW.md`'deki okuma kuralları ile tablolardan yeniden hesaplar: (1) gündüzün uzunluğu ve sekizde biri; (2) varaya göre üç işaretli dilimin sırası ve saatleri; (3) on beşte bir ve Abhijit'in saatleri; Çarşamba istisnası; (4) hora uzunluğu ve yirmi dört lord; (5) tithi grubu; (6) nakshatra doğası; (7) tara numarası ve kümede olup olmadığı; (8) chandra bala evi ve kümede olup olmadığı. Sonra kendi sonucunu `expected` ile karşılaştırır; farklar yazılır. Fark varsa önce kural yeniden okunur; düzeltme hangi tarafta olursa olsun ayrı commit'tir ve beklenen değer motor çıktısına bakarak değiştirilmez.

Gözden geçiren, `REVIEW.md`'nin sonundaki "Gözden geçirme kaydı" tablosuna adını (ya da ajan kimliğini), tarihi ve sonucu yazar:

```bash
git add tests/golden/modules/muhurta/REVIEW.md
git diff --cached --stat
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "docs(golden): muhurta hand fixtures re-derived by an independent reviewer" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
cargo test -p jyotish-core --test modules_golden 2>&1 | grep -E "^test result"
```
Expected: yalnızca `REVIEW.md` değişir (tablo satırı; fark bulunduysa ilgili örnek ayrı commit'tedir); `grep` çıktısız; `15 passed`. Sonuç ilerleme notlarına `Plan 20 el örnekleri yeniden türetildi: 7/7 (<tarih>, <gözden geçiren>)` olarak yazılır.

- [ ] **Step 6: [insan] Bellekten aktarılan klasik tabloların onayı**

Dilim sıraları, tithi grupları, nakshatra doğaları ve iki elverişli küme spec'te yazılıdır ve bu depoya bellekten aktarılmıştır (D11). Bu adım yalnızca elinde basılı bir panchang ya da muhurta derlemesi bulunan bir insan tarafından yapılabilir; yürütücü ajan bu adımı **yapmaz, başka bir yazılımın çıktısıyla ya da çevrimiçi bir tabloyla ikame etmez** ve `REVIEW.md`'deki "Bekliyor: sahibine devredildi" satırını olduğu gibi bırakır. Denetleyen kişi `REVIEW.md`'deki üç tabloyu ve "bilinen varyant noktaları" listesinin sekiz satırını (dilim sıraları ve orantılı dilim; Abhijit; hora uzunluğu; nakshatra doğaları; tara kümesi; chandra bala kümesi; panchakanın başlangıcı; Vishti) kaynakla karşılaştırır; kaynağı genel adıyla (alıntı yapmadan) ve sonucu tabloya yazar. Fark bulunursa: tablo ayrı bir commit'le düzeltilir, `REVIEW.md` güncellenir, etkilenen el örnekleri kuraldan yeniden türetilir (motor çıktısından değil) ve Step 1 ile Step 5 yeniden koşulur; ikinci bir okuma istenirse (ör. gündüz ve gece ayrı bölünen horalar) K7'ye göre ancak ikinci seçenek uygulanınca ayar olur. Bu madde devredildiyse ilerleme notlarına `Plan 20 muhurta tabloları: onay sahibine devredildi (<tarih>)` yazılır; devir `git push`'u engellemez, ama **tablolar onaylanana dek gün dilimleri ve sınıflar beta astrologlarına "doğrulanmamış tablo" notuyla sunulur** (sürüm notu sahibinin işidir).

---


## Başarı ölçütleri

- `jyotish-core` bir zaman aralığını boşluksuz aralık dizilerine böler (tithi, karana, nakshatra ve pada, nitya yoga, vara, Ay ve Güneş burcu, lagna burcu, graha burç ve nakshatra girişleri, dönüşler, yanma). Diziler aralığın içinde `compute_panchang` ile aynı değeri verir ve sınırları onun bitiş anıyla 1 saniye içinde örtüşür (sahte ve gerçek ephemeris'le); girişlerde boylam sınırın 1″ içindedir; dönüşlerde hız işaret değiştirir.
- Gün panchangı anı içeren günü verir: beş uzvun değerleri sınıflarıyla, sekiz dilim ve işaretli üçü, Abhijit, 24 hora, işaretler ve (doğum Ay'ı verilirse) tara ve chandra bala. Gün doğumu olmayan günde dilimler boş, uyarı dolu, uzuvlar tamdır. Dilimler ve sınıflar elle yazılmış tablolar, yedi el fikstürü (kaynak `hand`) ve değişmezlerle doğrulanır.
- Pencere araması süzgecin her koşulunu aralıkların kesişimiyle uygular; pencerenin her anı süzgeci sağlar, pencere arası sağlamaz; dilim uzunluğu sonucu değiştirmez; arama durumsuzdur ve dilim aralarında motoru bırakır. Bütçeler ölçülür ve tutar (`day_panchang` ≤ 50 ms; 366 gün ≤ 2 sn, lagna ile ≤ 5 sn; dilim ≤ 300 ms).
- Bulunulan yer tercihlerde durur, geçersizse yalnızca alan adıyla reddedilir, dışa aktarılmaz. Kayıtlı süzgeç yalnızca ad ve koşuldur. Yer, sorgu ve doğum Ay'ı hiçbir yerde loglanmaz; muhurta hataları yalnızca alan adı taşır.
- İki panel paletten eklenir. Gün paneli şeridi, uzuvları, işaretleri, kişiye bağlı etkenleri ve horaları çizer; arama paneli süzgeci kurar, kaydeder, dilim dilim arar, durdurur ve bir satırla transit tarihini o ana götürür. Boş, yükleniyor, hata, dolu ve "saat bilinmiyor" durumları testlidir; yeniden okuma içeriği kaldırmaz ve odak hiçbir eylemde gövdeye düşmez; renk tek başına anlam taşımaz; hüküm ve şablon yoktur; her kimliğin üç dilde adı vardır.
- `Chart`, `open_client`, `CalcSettings`, `ViewState`, SQLite şeması, bütün şema sürümleri, önceki modüller, `ChartSvg`, baskı düzenleri ve görsel temeller değişmemiştir; v1 golden testleri değişmeden geçer.
- Kapılar: `jyotish-core` lib 351, `modules_golden` 15, `atlas` 11 / 13, `store` lib 103 (+1 ign.), `jyotish-desk` lib 49; Vitest 80/795; e2e 22; görsel 10; `cargo deny` temiz; diğer sayılar tabandaki gibi. Elle denetim kapısının **[otomatik]** maddeleri testlerle doğrulanmış, **[insan]** maddeleri yapılmış ya da sahibine devredilmiştir; el örnekleri bağımsız olarak yeniden türetilmiş ve `REVIEW.md`'ye yazılmıştır; klasik tabloların onayı yapılmış ya da sahibine devredilmiştir.

## Bu planın kapsamadıkları

- **Spec'in 3.7 için kapsam dışı saydıkları:** etkinlik şablonları, vara–nakshatra birleşim yogaları, otuz muhurtanın adları, choghadiya ve benzeri bölgesel gün bölümlemeleri, tithi–vara–nakshatra dosha tablolarının tamamı, tutulmalar, takvim görünümünde aylık panchang, bildirim ve hatırlatma.
- **Sonraki alt projeler (3.8, 3.9):** Prashna ve arama. `timeline`'ın graha dizilerinin bir arayüzü bu planda yoktur (olay araması 3.9'dadır); iki grahanın kavuşumu, yeni ay ve dolunay olayları ve doğum grahasına geçiş 3.9'un işidir.
- **Klasik tabloların basılı kaynakla onayı** (D11; Task 9 Step 6) ve ikinci okumalar: gündüz ve gece ayrı bölünen horalar, sabit 06:00–18:00 günü, öğlenin iki yanında sabit Abhijit, Janma tarasının elverişli sayılması, shukla pakshada genişleyen chandra bala kümesi, 293°20′'de başlayan panchaka. K7'ye göre ancak ikinci seçenek uygulanınca ayar olur; bu planda ayar eklenmez ve yöntem kimliği `muhurta.v1`'dir.
- Bulunulan yer için ayrı bir elle koordinat formu (D29: elle girilmiş koordinat bir haritanın doğum yeri olarak alınır); haritasız (danışan açık değilken) gün panchangı (D28); arama için açık haritadan başka bir kişinin seçilmesi.
- Günün dışına taşan uzuvların gerçek başlangıç ve bitiş anları (D7); yaz saati geçiş gününde gün panchangının saat başına ofseti (D24); gün doğumu olmayan günlerde gündüz dilimleri (D9, D18).
- Bir adımın içinde değişip aynı değere dönen kısa gidişler (D3); gerçek düğümün yön değişimleri (D5); `muhurta_cancel` komutu ve Rust'ta arama durumu (D16); takvim ayı sınırlı dilimler.
- 500'den fazla pencerenin listelenmesi ya da sanal kaydırma (D31); sonuçların dışa aktarılması, takvime yazılması ya da saklanması (spec §16); kayıtlı süzgeçlerin dışa ve içe aktarılması.
- Gün panchangının ve arama sonucunun baskı düzeni (K10); gün şeridinin gece yarısını da gösteren 24 saatlik biçimi; gece dilimleri.
- Paket parçalama (D37: 500 kB uyarısı).
- Yorum metinleri ve AI yorum paneli; Windows ve Android.
- v1'den taşınan seyrek test dalgalanması (`NotesPanel.test.tsx › New note adds an empty note…`, yük altında): bu planın dosyalarına dokunmaz; ayrı bir bakım işidir.

## Spec'e geri bildirim (spec kusurları ve boşlukları)

Plan yazılırken bulunan ve Kararlar'da çözülen maddeler. "Kusur" spec'in kendi içinde çelişen, harfiyen uygulanamayan ya da mevcut kodla tutmayan yeridir; "boşluk" yazılmamış ayrıntıdır.

1. **Kusur — §10 Veri modeli: `DayPanchang`'ın `sunrise_jd_ut: f64`, `sunset_jd_ut: f64`, `next_sunrise_jd_ut: f64` ve `segments: DaySegments` alanları.** Aynı bölüm "kutup günü ya da gecesinde gündüz dilimleri hesaplanmaz; panel … yalnızca uzuvları gösterir" der: olmayan bir gün doğumu sayı olarak dönemez ve günün sınırları tipte yoktur. Çözüm (D9): üç alan ve `segments` `Option`; `day_start_jd_ut` ve `day_end_jd_ut` eklenir (gün doğumu yoksa 06:00 yerel ortalama saat, v1 kuralı). Spec tipi böyle yazmalı.
2. **Kusur — §10 "Bulunulan yer": `AppPrefs.home_place: Option<Place>`.** `Place` saat dilimi taşımaz; aynı bölümün komutu `tz` ister ve o yerin saatleri gösterilecektir. Çözüm (D21): `HomePlace { place, tz }`; şema sürümü değişmez. Spec tipi ve "eski kayıt yersiz okunur" kuralını yazmalı.
3. **Kusur — §10 Komut: `day_panchang(date, place, tz, settings, natal)`.** Bir takvim günü ve saat dilimi hangi gün doğumunun kastedildiğini tek başına söylemez; panelin ayarı bir andır ("transit tarihi / bugün") ve "bugün" Rust'ta okunmalıdır. Çözüm (D8, D23): `day_panchang(at: TransitAt, place: MuhurtaPlace, settings, natal)`; gün, anı içeren gündür (`compute_panchang`'ın kuralı). Spec imzayı ve "gün doğumundan önceki an önceki güne aittir" kuralını yazmalı.
4. **Kusur — §10 Komutlar: `muhurta_search(filter, search_id) -> MuhurtaSearchPage` ("ay ay sayfalanır") ve `muhurta_cancel(search_id)`.** İmzada sayfanın yeri yoktur; Rust'ta arama durumu, arayüzün "sonrakini isteme" ile yaptığı işten fazlasını yapmaz ve bir dilim en çok 37 ms sürer. Çözüm (D16): durumsuz, imleçli arama; `muhurta_cancel` ve `search_id` yok; dilimler 31 gündür (takvim ayı değil). Spec komutu imleçle yazmalı ve iptal komutunu kaldırmalı.
5. **Kusur — §10 `MuhurtaFilter { …, place: Place, …, natal }` ile "kayıtlı süzgeçler … kişi verisi içermez".** Süzgeç tipi yeri, tarih aralığını ve doğum Ay'ını koşullarla aynı tipte taşır; `Place` ayrıca bir `store` tipidir. Çözüm (D15): `MuhurtaCriteria` (saklanan) ve `MuhurtaFilter { from, to, lat, lon, criteria, natal }`. Spec iki tipi ayırmalı.
6. **Kusur — §10 `DaySegments { rahu: Interval<()>, … }`.** Sekiz dilim tipte yoktur, oysa panel onları çizer ve sınırlarını hesaplayamaz; `()` dilimin sırasını söylemez. Çözüm (D10): `eighths: Vec<Interval<u8>>` ve işaretli dilimlerde `Interval<u8>`. Spec tipi böyle yazmalı.
7. **Kusur — K7 ile §10 tipleri.** K7 "her modül sonucu `method` kimliği taşır" der; `DayPanchang` ve `MuhurtaSearchPage` taşımaz. Çözüm (D1): iki tipe `method` (`"muhurta.v1"`). Spec alanı eklemeli.
8. **Boşluk — §10 graha olayları.** Hangi grahalar için ve hangi biçimde olduğu yazılmamış. Çözüm (D5): diziler (sınır = olay); burç ve nakshatra dokuz graha için; dönüşler yalnızca beş graha için (Güneş ve Ay geri gitmez; düğümlerin yönü olay sayılmaz).
9. **Boşluk — §10 vara dizisi ve gün doğumu olmayan yer.** Çözüm (D6): v1 kuralı (06:00 yerel ortalama saat) yürünür; olağan enlemde doğuş anlarından kurulur.
10. **Boşluk — §10 dizilerin uçları.** Çözüm (D7): dizi verilen aralığa kırpılır; gün panchangında taşan uç arayüzde işaretlenir. Spec kırpmayı yazmalı.
11. **Boşluk — §10 sınıfların veri modelindeki yeri.** `tithis: Vec<Interval<u8>>` tithi grubunu, `nakshatras: Vec<Interval<(u8, u8)>>` doğayı taşımaz. Çözüm (D12): `DayTithi`, `DayNakshatra`, `DayKarana`.
12. **Boşluk — §10 tanımsız tipler: `MuhurtaFlag`, `PersonalFactors`, `WindowSnapshot`.** Çözüm (D13, D14, D19): on değerli enum (ikili kalemler ayrı); kişiye bağlı etkenler aralık dizisi; özet pencerenin başlangıcının noktasal hesabı. Janma tarasının elverişli kümede olmadığı açıkça yazılmalı.
13. **Boşluk — §10 sayfa sınırını aşan pencere ve kıymık pencere.** Çözüm (D17): açık pencere imleçte taşınır; 1 saniyeden kısa pencere verilmez (iki ayrı dizinin aynı ana düşen sınırları çözünürlük kadar ayrışır).
14. **Boşluk — §10 aramanın kutup davranışı.** Çözüm (D18): "yalnızca gündüz" o günlerde pencere vermez; dilim dışlaması uygulanmaz; sayfa uyarı taşır.
15. **Boşluk — §10 kayıtlı süzgeçlerin biçimi.** Çözüm (D22): `{schema_version, filters: [{name, criteria}]}`; en çok 32; ad tek; listenin tamamı yazılır.
16. **Boşluk — §10 tarih aralığının verilişi.** Çözüm (D23): yerin takvim günleri (ikisi de dahil); boş başlangıç bugün, boş bitiş 30 gün sonrası; aralık ilk çağrıda çözülür ve imleçte taşınır.
17. **Boşluk — §10 saatlerin gösterildiği dilim.** Çözüm (D24): seçilen yerin saati; ofset Rust'ta çözülür ve sonuçla gelir (`atlas::offset_at`).
18. **Boşluk — K12 ile §10 "harita açıksa tara ve chandra bala".** Modül panelleri yalnızca harita açıkken sunulur: koşul her zaman sağlanır ve haritasız gün panchangı yoktur. Çözüm (D28). Spec haritasız kullanımı isteyip istemediğini yazmalı (isterse K12'den ayrı bir yüzey gerekir).
19. **Boşluk — §10 "atlas'tan seçilir ya da elle girilir".** Elle girişin biçimi yazılmamış. Çözüm (D29): atlas araması ve "açık haritanın doğum yerini kullan"; ayrı koordinat formu yok. Spec ayrı bir form isteyip istemediğini yazmalı.
20. **Boşluk — §10 gün panelinin düzeni.** Sütunlar, saat biçimi, şeridin erişilebilir biçimi ve transit seçicisinin dilimi yazılmamış. Çözüm (D30).
21. **Boşluk — §10 arama panelinin düzeni.** Kümelerin gösterimi, tarih alanları, uzun sonucun sınırı ve odak kuralları yazılmamış. Çözüm (D31, D32).
22. **Boşluk — §14 ile v1 hataları.** "`home_place` hata parametresine girmez" kuralı, konumu ve saat dilimini parametreye yazan v1 hatalarıyla (`invalid_location`, `unknown_time_zone`) birlikte düşünülmemiş. Çözüm (D20): muhurta yolları kendi alan adlı hatasını üretir. Spec kuralı "muhurta ve prashna yolları konumu kendileri denetler" diye yazmalı.
23. **Boşluk — §10 "v1'deki panchang bitiş aramasıyla aynı yöntem".** v1'in araması tekdüze artan açı ister; retro girişler ve lagna bunu sağlamaz. Çözüm (D2, D3): ayrık değerin yürüyüşü; adım içinde gidip dönen kısa gidiş görülmez. Spec yöntemi ve bu sınırı yazmalı.
24. **Boşluk — K2 ve §13 "saat bilinmiyor" durumu.** Muhurta doğum lagnasını kullanmaz; `None` dönecek alan yoktur. Çözüm (D33): durum, yaklaşık doğum Ay'ıdır ve arayüzde "≈" ile işaretlenir. Spec §10'a bu cümleyi eklemeli.

## Self-Review

**1. Spec kapsamı.**

| Gereksinim (ileri modüller spec'i) | Görev |
|---|---|
| §10 `timeline`: tithi, nakshatra (pada ile), nitya yoga, karana, vara, Ay burcu, Güneş burcu; boşluksuz `[start, end)` aralıkları; ikiye bölme | Task 1 (D2–D4, D6, D7) |
| §10 `timeline`: lagna burcu | Task 1 |
| §10 `timeline`: graha burç girişleri, nakshatra girişleri, retro ve düz dönüşler | Task 1 (D5); Task 5 (gerçek ephemeris) |
| §10 Gün panchangı: beş uzvun değerleri ve değişim saatleri | Task 3 |
| §10 Sekizde bir dilimler ve varaya göre sıralar | Task 2 (D10, D11); Task 9 Step 6 |
| §10 Abhijit; `abhijit_weekday_exception` | Task 2 |
| §10 Hora tablosu (3.4'teki kural) | Task 2 (`hora_lord`, `vara_lord`) |
| §10 Tithi grubu, nakshatra doğası | Task 2 (D12), Task 3 (değerin içinde) |
| §10 İşaretler | Task 1–3 (D13) |
| §10 Tara bala, chandra bala; elverişli kümeler | Task 2, 3 (D14) |
| §10 Pencere araması: süzgeç alanları (tarih aralığı ≤ 366 gün, yer, varalar, tithiler, nakshatralar, lagna burçları, yalnızca gündüz, en kısa süre, dışlanan işaretler, tara ve chandra bala) | Task 3 (D15–D18), Task 5 (D23), Task 7 |
| §10 Sonuç: pencereler, o andaki beş uzuv ve lagna | Task 3 (D19) |
| §10 Şablon yok; süzgeç adlandırılıp saklanır | Task 4, 7 (D22) |
| §10 `home_place`; boşsa doğum yeri önerilir; konum servisi yok | Task 4, 6 (D21, D29) |
| §10 Komutlar | Task 5 (D8, D16, D23 farklarıyla) |
| §10 `muhurta_filters` `app_state`'te; kişi verisi içermez | Task 4 (D22) |
| §10 Arayüz: iki panel ve ayarları | Task 6, 7 (D28) |
| §10 Arayüz: desen ve yazı; 100 ms / 1 sn; kutup günü | Task 6, 7 |
| §10 Test: çapraz denetim | Task 1, 3, 5 |
| §10 Test: el örnekleri ve ikinci yazım | Task 2 |
| §10 Test: property'ler | Task 1–3 |
| §10 Test: astronomik | Task 1, 5 |
| §10 Gizlilik | Task 4, 5; Task 9 Step 3 |
| §10, §15 Performans | Task 5 (D26) |
| K1, K2, K3 | Task 1–5 (D33) |
| K4, K5 | Global Constraints "Çerçeveler"; Task 1 (tropikal rasi testleri) |
| K7 (`method`; ayar yok) | D1; "Bu planın kapsamadıkları" |
| K8, §14 | D20; C1 |
| K9, §13 | Task 2; Task 9 Step 5, 6 |
| K10, K11, K12 | D37; D34; Task 6–8 |
| §17 "Tek ephemeris iş parçacığı … dilimleme ve 300 ms sınırı" | Task 5 (dilim başına ölçüm) |
| Ek A | D35; Global Constraints; C13 taramaları |

Boşluk: yok. Spec'in istediği ama planın başka biçimde verdiği parçalar: `muhurta_cancel` (D16: yok), takvim tarihli `day_panchang` (D8: an), elle koordinat formu (D29). Spec'in istemediği ama planın eklediği parçalar: `sun_days` ve `SunDay`, `combust_series`, `lagna_sign_at`, `window_snapshot`, `atlas::offset_at`, `Transit.epoch`, `useLastData`, 1 saniye kuralı, `REVIEW.md`'deki varyant listesi, fikstürdeki `muhurta_ids`.

**2. Yer tutucu taraması.** "TODO", "TBD", "daha sonra", "uygun hata işleme" yok. Task 9'daki `<tarih>` ve `<gözden geçiren>` denetimin yapıldığı günün ve yapanın yazılacağı yerdir; `REVIEW.md`'deki "Bekliyor" satırları Step 5 ve Step 6'nın kaydıdır. Her kod adımı tam dosya ya da `ff3da00` ağacına (ve önceki görevlere) karşı üretilmiş diff taşır.

**3. Tip tutarlılığı.**
- `Interval<T> { value, start_jd_ut, end_jd_ut }`, `NakshatraPada`, `Motion`, `SunDay` Task 1'de; `MuhurtaFlag` (10), `NatalMoon`, `DaySegments`, `TithiGroup` (5), `NakshatraNature` (7), `TaraBala`, `ChandraBala` Task 2'de; `DayPanchang`, `DayTithi`, `DayNakshatra`, `DayKarana`, `PersonalFactors`, `MuhurtaCriteria`, `MuhurtaFilter`, `MuhurtaCursor`, `MuhurtaWindow`, `WindowSnapshot`, `MuhurtaSearchPage` Task 3'te Rust'ta tanımlanır; TS bağlaması Task 5'te üretilir; `ui_fixture.rs`, `tests/modules.rs`, paneller ve e2e aynı adları okur. Spec §10'dan farklar D8–D10, D12, D15 ve D16'dadır.
- `MuhurtaCriteria`'nın dokuz alanı (`weekdays`, `tithis`, `nakshatras`, `lagna_signs`, `daytime_only`, `min_minutes`, `exclude`, `require_tara`, `require_chandra`) Rust'ta, `store/muhurta.rs`'in saklanan JSON'unda, `parseCriteria` ve `EMPTY_CRITERIA`'da, arama panelinin kutularında ve testlerde aynıdır.
- `MuhurtaFlag` değerleri Rust'ta, `MUHURTA_FLAGS`'te, fikstürün `muhurta_ids.flags` listesinde ve üç dil dosyasının `muhurta.flag` ağacında aynıdır; Vitest son üçünü karşılaştırır. `NakshatraNature` için `NAKSHATRA_NATURE_NAMES` ve `muhurta.nature` aynı biçimde denetlenir.
- El fikstürü alanları (`input.{day.{vara, sunrise_min, sunset_min, next_sunrise_min}, tithi, moon, natal}`; `expected.{eighth_min, rahu, yamaganda, gulika, abhijit, abhijit_weekday_exception, hora_min, hora_lords, tithi_group, nakshatra_nature, tara, chandra}`) düzenekteki `MuhurtaInput`, `MuhurtaDay`, `MuhurtaExpected`, README satırı ve yedi JSON dosyasında aynıdır.
- Komut adları: Rust `day_panchang`, `muhurta_search`, `load_muhurta_filters`, `save_muhurta_filters`; gövdeler `day_panchang_blocking`, `muhurta_search_blocking`; TS `dayPanchang`, `muhurtaSearch`, `loadMuhurtaFilters`, `saveMuhurtaFilters`; bağımsız değişken adları `at`, `place`, `settings`, `natal`, `query`, `cursor`, `filters`; e2e taklidindeki anahtarlar aynıdır.
- Fikstür anahtarları `muhurta.{natal, place, home_place, day, day_polar, query, pages}` ve `muhurta_ids.{flags, tithi_groups, nakshatra_natures, max_min_minutes}` `ui_fixture.rs`, `test/fixture.ts`, panel testleri ve `e2e/ipc.ts`'te aynıdır.
- Panel türleri `muhurta_day` ve `muhurta_search`: `PANEL_KINDS`, `MODULE_PANELS`, `PANEL_REGISTRY`, `panel.*` i18n anahtarları ve e2e'de kaydedilen düzen aynı adı taşır. Panel ayar anahtarları `place`, `link`, `custom`, `criteria`'dır.
- Hata alanları: çekirdek `range`, `place`, `natal`, `graha`, `tithis`, `nakshatras`, `min_minutes`, `cursor`; komut `from_date`, `to_date`, `place`, `range`; store `home_place`, `muhurta_filters`, `muhurta_filters.name`, `muhurta_filters.criteria`.

**4. Kural denetimi.** C1 (yer, sorgu ve doğum Ay'ı loglanmaz; tek yeni kod yalnızca alan adı taşır), C2′ (yeni dizgi bağımsız değişkeni yok), C3 (SQLite şeması ve şema sürümleri değişmez; `AppPrefs` bir alan kazanır), C4 (bütün değerler ve "şimdi" Rust'tan; saat, süre ve çubuk payı gösterim), C5/C6 (ayarlar mevcut düzen kaydıyla; ayrıştırıcılar değişmez), C7 (kısayol eklenmez), C8 (bulunulan yer penceresi `Dialog`), C9 (desen ve yazı; kenar çizgisi ve metin; yeni stilde hex yok), C10 (iki üretilen dosya komutla, iki görevde), C12 (yeni kod ve her yeni anahtar üç dilde), C13 (her commit'te tarama), C14 (`panchang.rs`'te yalnızca iki görünürlük), K9 (yalnızca `hand`; tablolar başka yazılımla karşılaştırılmaz), K10 (baskı ve görsel temeller değişmez) görevlerin adımlarında ve Task 9 Step 3'te denetlenir. Önceki planların dersleri: yeniden okuma içeriği kaldırmaz (`useLastData`; arama listesi), odaktaki denetimi kaldıran eylem odağı taşır ("Durdur", "Süzgeci sil", "Yeri kaldır"), kaydırma kapları odak alır ve adlıdır, Vitest yolları `app/` dizinine göredir, bellekten aktarılan tablolar varyant noktalarıyla sahibin onayına işaretlidir.

**5. Koşulan doğrulama.** Bu plandaki kod, deponun `ff3da00` üzerindeki bir kopyasında (kısa bir yolda, Rust 1.100) sekiz commit olarak uygulanmış ve her commit'te ölçülmüştür: `jyotish-core` lib 307 → 321 → 331 → 351; `modules_golden` 13 → 15; `atlas` 10 / 12 → 11 / 13; `store` lib 96 → 103 (+1 ign.); `jyotish-desk` lib 46 → 49; `modules` 1 (ortancalar: `day_panchang` 0,55 ms; 366 günlük arama 0,20 sn, lagna koşuluyla 0,42 sn, yalnızca lagna 0,25 sn; en yavaş dilim 37 ms; önceki dokuz ölçüm 0,57–3,40 ms); Vitest 738 → 738 → 738 → 771 → 795 (dosya 75 → 78 → 80); e2e 20 → 22. Her commit'te `cargo fmt --check` ve `clippy -D warnings` (1.100) çıktısızdır ve `pnpm build` `✓ built` verir. Son durumda `cargo test --workspace`, `cargo deny check` (dört `ok`), 1.93 ile `cargo check`, `pnpm build` (608,6 kB; D37), görsel 10 (temel dosyalar değişmeden) ve yayın betiği testleri 14 yeşildir; Task 9 Step 3'ün taramaları beklendiği gibidir. "Testlerin başarısız olduğunu doğrula" adımlarındaki çıktılar da aynı kopyada, bir önceki görevin ağacına yalnızca o adımın test değişiklikleri konarak ölçülmüştür. Ayrıca bu belgedeki bloklar (27 yeni dosya ve 26 diff bloğu) belgeden okunup sırayla temiz bir `ff3da00` ağacına uygulanmış ve her görevin sonunda o commit'in ağacıyla birebir aynı sonucu vermiştir (üretilen iki dosyayı Task 4 ve Task 5'in adımlarındaki komutlar üretir). Plan yazılırken `main` bir kez ilerledi (`b77e8d4` → `ff3da00`: Plan 19'un son inceleme commit'leri); kopya yeni uca taşındı ve bütün ölçümler `ff3da00` üzerinde yinelendi.

El fikstürlerinin değerleri bu plan yazılırken motordan bağımsız bir hesapla türetilmiştir: kural metninden (spec §10 ve Kararlar) başka bir dilde, sözlükler ve ad listeleriyle yazılmış ayrı bir hesap yedi günün `derivation` satırlarını ve `expected` değerlerini üretmiştir (dilim sıraları üç ayrı sözlük, hora döngüsü ad listesi, tara ve chandra bala adım adım sayım). Motor yedi örnekte ilk koşuda birebir uyuşmuştur. Hiçbir beklenen değer motor çıktısına bakılarak değiştirilmemiştir. İki türetmeyi de aynı yazar yazmıştır ve **tablolar iki hesapta da aynı kaynaktan (spec'in listeleri ve yazarın belleği) gelir**: bu uyum aritmetiği ve okumayı doğrular, tabloların kendisini değil. Task 9 Step 5 (bağımsız gözden geçirme) ve Step 6 (basılı kaynakla onay) bunun için vardır ve bu belgedeki doğrulamayla yerine getirilmiş sayılmaz.

Yazım sırasında düşenler ve düzeltmeleri (hiçbirinde beklenen değer motor çıktısından kopyalanmadı): (1) `chunk_length_does_not_change_the_windows` property'si altı koşunun dördünde düştü: panchaka dışlanmış ve chandra bala isteniyorken, Ay'ın Balık'tan Koç'a geçtiği anda iki ayrı dizinin sınırları 0,05 saniye ayrışıyor ve dilim uzunluğuna göre bir görünüp bir yok olan kıymık bir pencere çıkıyordu. 1 saniye kuralı eklendi (D17), durum bir birim testine yazıldı ve altı property 3 × 4000 durumla yeniden koşuldu. (2) Kayıtlı süzgeçler ilk taslakta çıplak bir `useEffect` ile okunuyordu; geliştirme kipinde liste iki kez isteniyordu (e2e buldu). `useModule` kullanıldı. (3) İlk e2e taslağı odağın açan düğmeye dönüşünü fare tıklamasıyla sınıyordu; WebKit tıklanan düğmeye odak vermez. Adımlar klavyeyle çalıştırıldı. (4) Gün panelinin tarih bağı seçicisinin etiketi ilk taslakta "Tarih"ti ve transit seçicisinin "Tarih" alanıyla çakışıyordu; "Tarih bağı" yapıldı. (5) Bulunulan yer penceresinde iki `role="status"` satırı vardı (yer arama kutusununki ve seçili yer); testler seçili yer satırını sınıfıyla okur. (6) `formatJdClock` testinin iki beklentisi yanlış yazılmıştı (dakikanın aşağı yuvarlanması; üç günlük fark); testler düzeltildi, kod değişmedi. (7) İlk gün panchangı testinde elle hesaplanan tithi ve karana listeleri günün sonunu aşıyordu (gün uzanım 93,19°'de biter); beklentiler elle yeniden hesaplandı. (8) İki Türkçe yorumda yasak dizi oluşmuştu (bir çoğul ek ve bir sözcük); tarama buldu, yeniden yazıldı. **Koşulmayanlar:** `pnpm tauri dev` ile gerçek uygulama ve gerçek veritabanı (Task 9 Step 4'ün **[insan]** maddeleri; özellikle uygulama yeniden başlatılınca bulunulan yerin geri gelmesi ve basılı bir panchangla karşılaştırma), VoiceOver, ve klasik tabloların basılı kaynakla onayı (Task 9 Step 6): bunlar yürütme sırasında yapılır ya da sahibine devredilir.
