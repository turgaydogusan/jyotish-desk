# Transit takvimi: okuma kuralları, sahibin kararını bekleyen noktalar ve gözden geçirme kaydı

Kaynak: yol haritası (`docs/superpowers/specs/2026-11-27-eksikler-yol-haritasi.md`) §4.11 ve §5
"Plan 29", Plan 29'un kararları (D1–D22) ve ileri modüller tasarım belgesinin K1–K5 kuralları.
Bu modül bellekten aktarılmış klasik tablo taşımaz (Y4'ün "doğrulanmamış tablo" notu gerekmez)
ve yeni astronomi içermez: olayları olay aramasının çekirdeği bulur, tithi ve nakshatra v1'in
panchang fonksiyonlarıyla okunur. Taşıdığı şey gün tanımı ve varsayılan seçimlerdir; bunlar
aşağıda sahibin kararını bekler. Bu dizinde JSON örneği yoktur ve hiçbir test bu klasörü
yüklemez: el örnekleri Rust birim testleridir ve türetmeleri test yorumlarında yazılıdır
(`crates/jyotish-core/src/calendar/grid.rs`, `crates/jyotish-core/src/calendar/events.rs`).
Beklenen değerler kuraldan elle türetilmiştir; motor çıktısı kopyalanmaz (K9). Hiçbir hücre
hüküm değildir: takvim olguları sayar, "uygun gün" demez.

## Okuma kuralları

1. **Hücre takvim günüdür.** Gün, seçilen yerin duvar saatiyle gece yarısından sonraki gece
   yarısınadır (`[başlangıç, bitiş)`). Yaz saati geçişinde gün 23 ya da 25 saat sürer. Saat
   diliminin takvimden atladığı gün (örnek: Samoa, 30 Aralık 2011) ızgarada yoktur.
2. **Günün olguları gün doğumundadır.** Hücrenin tithisi ve Ay nakshatrası, o takvim gününün
   içindeki gün doğumunda geçerli olan değerdir: o gün başlayan panchang gününün ilk değeri.
   Gün içinde değişen tithi ya da nakshatra hücrede yazılmaz; gün panchangı paneli bütün
   değişimleri saatleriyle verir. Bir tithi iki gün doğumunda birden geçerliyse iki hücrede de
   yazılır; hiçbir gün doğumuna rastlamayan tithi hiçbir hücrede yazılmaz.
3. **Tithi ve nakshatra.** Tithi, Ay − Güneş uzanımının 12°'lik dilimidir (1–30; 1–15 Shukla,
   16–30 Krishna). Nakshatra, Ay'ın sidereal boylamının 13°20′'lık dilimidir (1–27). İkisi de
   her zodyak kipinde sidereal boylamdan okunur. Dilim sınırı sonraki dilime aittir.
4. **Gün doğumu.** Gün doğumu hesap ayarındaki tanımla (disk kenarı ya da merkezi, kırılma)
   ve seçilen yerin koordinatıyla bulunur. Günün içinde iki doğuş varsa ilki alınır; tam gece
   yarısındaki doğuş başlayan günündür.
5. **Gün doğumu olmayan gün.** O takvim gününde Güneş doğmuyorsa (kutup gecesi ya da gündüzü)
   olgular 06:00 yerel ortalama saatte okunur: günün ortasının düştüğü yerel ortalama tarihin
   06:00'sı. Hücre `*` işareti ve "gün doğumu yok" sözü taşır. Kural gün panchangı panelinin
   kuralıdır: o panel aynı günü gösterir.
6. **Olayın günü.** Olay, anını içeren takvim günündedir; tam gece yarısındaki olay başlayan
   günündür. Günün olayları zaman sırasındadır; saatler olayın kendi anındaki ofsetle yazılır.
7. **Olay türleri.** Takvim yedi türü gösterebilir: burç girişi, nakshatra girişi, R
   başlangıcı, R bitişi, yeni ay, dolunay ve doğum boylamına geliş. Kavuşum ve doğum Ay'ından
   ev değişimi takvimde yoktur (olay araması panelindedir). Burç rasi çerçevesindedir (zodyak
   kipini izler); nakshatra siderealdir.
8. **Ay ayrı bir seçimdir.** Ay kapalıyken aranan grahalar Ay dışındaki sekiz grahadır; Ay
   açılınca Ay'ın burç ve nakshatra girişleri ile doğum boylamlarına gelişleri de aranır. Yeni
   ay ve dolunay bu seçime bağlı değildir.
9. **Doğum boylamına geliş** açık haritayı ister: geçiş grahasının, dokuz doğum grahasından
   birinin sidereal boylamına gelişidir (kendi boylamına dönüşü dahil). Tür seçili değilse
   doğum verisi komuta gönderilmez. Doğum saati bilinmiyorsa doğum boylamları öğlen içindir ve
   panel bunu söyler.
10. **Sınırlar.** Bir ay en çok 300 olay taşır; fazlası kesilir ve panel bunu söyler. Bir hücre
    en çok üç olay yazar ve gerisini sayar; gün ayrıntısı günün bütün olaylarını listeler.
11. **Dilimler.** Ay en çok yedi günlük dilimlerle hesaplanır; dilim sınırı olay kaybettirmez
    ve çoğaltmaz (sınırdaki değişim içinde bulunduğu dilime aittir).
12. **Yıl aralığı** 1–2999'dur. Paketlenen ephemeris dosyalarının dışındaki aylar (1800'den
    önce, 2399'dan sonra) düşük hassasiyetle hesaplanır ve panel uyarıyı taşır.
13. **Hafta başlangıcı** arayüz diline bağlıdır: Türkçe ve Fransızcada Pazartesi, İngilizcede
    Pazar. Günün sütununu Rust'ın verdiği hafta günü belirler.
14. **"Bugün"** seçilen yerin saat dilimindeki bugündür ve Rust'ta okunur.

## El örnekleri (Rust birim testleri)

Sahte gökyüzü (`crates/jyotish-core/src/fake.rs`): sidereal boylamlar J2000'de (JD 2451545,0)
Güneş 346° (0,98°/gün), Ay 71° (13,2°/gün), Mars 176° (0,6), Merkür 356° (1,2), Jüpiter 226°
(−0,1), Venüs 16° (1,1), Satürn 276° (0,05); gün doğumu her gün JD'nin 0,67 kesrindedir. Test
günleri 2000-01-01 00:00 UT'de (JD 2451544,5) başlayan ardışık UT günleridir; gün sırası 0'dan
başlar.

| Test | Sınadığı kural |
|---|---|
| `grid.rs › the_facts_of_a_day_are_read_at_the_sunrise_inside_it` (üç gün: tithi 7, 8, 9; nakshatra 5, 6, 7) | 2, 3, 4 |
| `grid.rs › the_waning_half_starts_after_the_fifteenth_tithi` | 3 |
| `grid.rs › a_sunrise_belongs_to_the_day_that_contains_it_and_midnight_opens_the_day` | 1, 4 |
| `grid.rs › a_day_without_a_sunrise_is_read_at_six_local_mean_time` (boylam 30° D: 04:00 UT) | 5 |
| `grid.rs › spans_are_one_to_thirty_one_gapless_days_of_plausible_length` | 1 |
| `grid.rs › a_month_is_cut_into_slices_of_at_most_seven_days` | 11 |
| `events.rs › the_filter_takes_calendar_kinds_only_and_the_moon_is_a_separate_choice` | 7, 8, 9 |
| `events.rs › an_event_belongs_to_the_day_that_contains_its_moment` | 6, 10 |
| `events.rs › sign_ingresses_and_lunations_land_on_their_days` (yedi olay: gün 3, 7, 8, 13, 14, 23, 28) | 6, 7 |
| `events.rs › the_moon_adds_its_own_ingresses_only_when_asked` (Ay: 13 burç, 31 nakshatra girişi) | 8 |
| `events.rs › the_sliced_month_finds_exactly_the_events_of_one_search` (ikinci yazım) | 9, 11 |
| `events.rs › the_event_limit_cuts_the_list_and_says_so` | 10 |
| `events.rs › bad_input_is_named_by_field_and_low_precision_is_reported` | 12 |
| `app/src-tauri/tests/calendar.rs` (gerçek motor: Delhi Kasım 2026 gün panchangı ve olay aramasıyla çapraz denetim; Berlin yaz saati günleri; Tromsø kutup gecesi ve gece yarısı güneşi; Samoa'nın atlanan günü; yıl aralığının uçları) | 1–12, 14 |
| `app/ui/calendar/model.test.ts`, `CalendarPanel.test.tsx` | 10, 13 |

## Sahibin kararını bekleyen noktalar

Aşağıdaki her satır tek seçimle uygulanmıştır (Y5); ikinci seçim ayar değildir. Sahibi kararını
alttaki tabloya yazar. Değişiklik ayrı bir commit'tir; gün tanımı değişirse yöntem kimliği
`calendar.month_v2` olur.

| # | Nokta | Uygulanan seçim | Öteki seçim |
|---|---|---|---|
| 1 | Hücrenin gösterdiği gün | Takvim günü (gece yarısından gece yarısına); olgular o günün gün doğumunda | Panchang günü (gün doğumundan gün doğumuna): olaylar da o güne yazılır, gece yarısından sonraki olay önceki günün hücresine düşer |
| 2 | Hücredeki tithi | Gün doğumundaki tithi (tek değer) | Günün bütün tithileri bitiş saatleriyle; gün içinde biten tithiyi de yazan düzen |
| 3 | Gün doğumu olmayan gün | 06:00 yerel ortalama saat, `*` işaretiyle | Hücreyi boş bırakmak; yerel öğleyi almak |
| 4 | Varsayılan olay türleri | Burç ve nakshatra girişleri, R başlangıcı ve bitişi, yeni ay, dolunay; Ay'ın kendi olayları ve doğum boylamına gelişler kapalı | Yalnızca burç girişleri ve Ay evreleri; ya da Ay'ın burç girişleri açık |
| 5 | Takvimde olmayan türler | Kavuşum ve doğum Ay'ından ev değişimi yok | İkisini de süzgece eklemek |
| 6 | Varsayılan yer | Bulunulan yer (gün panchangı gibi) | Açık haritanın doğum yeri (olay araması gibi) |
| 7 | Hafta başlangıcı | Türkçe ve Fransızcada Pazartesi, İngilizcede Pazar | Her dilde Pazartesi; ya da ayrı bir tercih |
| 8 | Hücredeki olay sayısı | En çok üç, gerisi sayılır | Hepsini yazan, satırı uzatan düzen |
| 9 | Panelin Türkçe adı | "Transit takvimi" (arayüzdeki "Transit tablosu" ve "transit tarihi" ile aynı sözcük) | "Geçiş takvimi" (yol haritasındaki ad) |

| Karar veren | Tarih | Sonuç |
|---|---|---|
| — | — | Bekliyor: sahibine devredildi (Plan 29 Task 8) |

## Denetim maddeleri

**[otomatik]** maddelerin karşılığı testlerde vardır; yürüten ajan testin adını göstererek
"geçti" sayar. **[insan]** maddeler yalnızca gerçek uygulamaya ya da basılı bir takvime bakan
biri tarafından doğrulanabilir ve sahibine devredilir (Y3: sonraki planı engellemez).

- [x] **[otomatik]** Hücrenin tithisi ve nakshatrası, o gün doğumuyla başlayan panchang gününün ilk değeridir (otuz gün, gerçek motor). (`tests/calendar.rs`; `grid.rs › the_facts_of_a_day_…`)
- [x] **[otomatik]** Takvimin olayları aynı ay için olay aramasının bulduklarıdır; dilimleme olay kaybettirmez ve çoğaltmaz. (`tests/calendar.rs`; `events.rs › the_sliced_month_…`)
- [x] **[otomatik]** 2026 Kasım'ında yeni ay 9'unda, dolunay 24'ündedir; Güneş ayda bir kez burç değiştirir. (`tests/calendar.rs`)
- [x] **[otomatik]** Yaz saati günleri 23 ve 25 saattir; kutup gecesinde ve gece yarısı güneşinde olgular 06:00 yerel ortalama saatte okunur ve gün panchangıyla aynı günü gösterir; takvimden atlanan gün ızgarada yoktur. (`tests/calendar.rs`)
- [x] **[otomatik]** Girdi hataları yalnızca alan adını taşır; doğum verisi yalnızca doğum türü seçiliyken gönderilir; bir ayın ortanca süresi bütçenin içindedir. (`tests/calendar.rs`; `calendar_commands.rs` testleri; `CalendarPanel.test.tsx`)
- [x] **[otomatik]** Panel: ay ızgarası, bugünün işareti, hücrede en çok üç olay, ay seçimi (önceki, sonraki, ay, yıl, "Bu ay"), yer seçimi, gün doğumu olmayan gün, düşük hassasiyet, hata ve yeniden deneme. (`CalendarPanel.test.tsx`)
- [x] **[otomatik]** Klavye: ızgara tek sekme durağıdır; oklar, Home, End, Page Up ve Page Down; Enter ve boşluk günü seçer; odak gövdeye düşmez. (`CalendarPanel.test.tsx`)
- [x] **[otomatik]** Gün ayrıntısı günün bütün olaylarını listeler; olayın saati transit tarihini değiştirir; "Gün panchangını aç" paneli ekler. (`CalendarPanel.test.tsx`)
- [x] **[otomatik]** Süzgeç panelin ayarıdır; hafta İngilizcede Pazar, Türkçe ve Fransızcada Pazartesi başlar; üç dilde ay ve gün adları vardır. (`CalendarPanel.test.tsx`, `model.test.ts`, `i18n.test.ts`)
- [x] **[otomatik]** Eski isteğin geç gelen yanıtı gösterilmez; yeni ay yüklenirken eski ızgara durur. (`useCalendar.test.tsx`)
- [ ] **[kontrolör]** Uçtan uca akışlar ve görsel temeller. (`app/e2e/calendar.desk.spec.ts`: üç akış; `app/e2e/calendar.visual.spec.ts`: üç temel)
- [ ] **[insan]** Gün tanımı (1. ve 2. satırlar): bir ayın hücreleri, sahibin kullandığı basılı bir panchang takvimiyle yan yana konur; gün doğumundaki tithi ve nakshatra uyuşur. Farklı bir gün tanımı isteniyorsa karar tablosuna yazılır.
- [ ] **[insan]** Varsayılan olay türleri (4. ve 5. satırlar) bir danışan görüşmesinde yeterlidir; hücreler ne boş ne de aşırı doludur.
- [ ] **[insan]** Izgara ekranda okunur: açık ve koyu temada, üç dilde; dar panelde yatay kayar; bugün ve seçili gün ayırt edilir (renk dışında sözle ve çerçeveyle); üç olaylı hücre taşmaz.
- [ ] **[insan]** Klavyeyle bir ay boyunca gezilir; Page Down ile ay değişince odak aynı günde durur; VoiceOver hücreyi gün, tithi, nakshatra ve olaylarıyla, seçili günü "seçili" diye okur.
- [ ] **[insan]** "Gün panchangını aç" doğru günü açar (gün panchangı panelinin tarih bağı "Transit tarihi" iken); gece yarısından sonra "Bu ay" bugünü yeni güne taşır.
- [ ] **[insan]** Kutup dairesinin ötesinde bir yerde kış ayı: `*` işaretli günler ve dipnot okunur.
- [ ] **[insan]** Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum anı, yer, koordinat ya da takvim içeriği görünmez.

## Gözden geçirme kaydı

Gözden geçiren (motoru yazan dışında biri) `grid.rs` ve `events.rs` testlerindeki el örneklerini
yalnızca yukarıdaki sahte gökyüzünden ve okuma kurallarından başlayarak, motoru çalıştırmadan
yeniden hesaplar: üç günün gün doğumundaki tithisi ve nakshatrası, gün doğumu olmayan günün anı,
bir aydaki yedi olayın günü ve Ay'ın giriş sayıları. Sonucunu testteki beklenen değerlerle
karşılaştırır ve aşağıya adını, tarihini ve bulduğu farkları yazar.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
| — | — | Bekliyor: Plan 29 Task 8 Step 4 |
