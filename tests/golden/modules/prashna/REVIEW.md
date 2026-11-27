# Prashna el örnekleri: okuma kuralları, tablolar, onay bekleyen noktalar ve gözden geçirme kaydı

Kaynak: ileri modüller tasarım belgesi §11 (sayı prashnasının lagnası ve soru haritasının
olguları), K4–K6 (burç evleri, rasi çerçevesi, doğal sınıf) ve v1'in dignity ile yanma
kuralları. Bu dizindeki her JSON dosyası sentetik bir haritadır: dokuz grahanın sidereal
boylamı, lagna (boylam olarak ya da 1–108 arası bir sayı olarak; `null` ise saat bilinmiyor),
retro sayılan grahalar ve astrologun seçtiği odak evi. Ephemeris yoktur; rasi çerçevesi sidereal
sayılır. `derivation` lagnayı, burcun türünü ve doğuş biçimini, lagna lordunu, kendraları, odak
evini ve lordunu, iki lordun bağını ve ev sayılarını, Ay'ı ve Ay'ın aldığı bakışları sırayla
yazar. Değerler kuraldan elle türetilmiştir; motor çıktısı buraya kopyalanmaz (K9). Bir değer
motorla uyuşmazsa önce kural yeniden okunur. Hiçbir satır hüküm değildir: örnekler olguları
sayar, sorunun yanıtını söylemez.

## Okuma kuralları

1. **Sayı prashnasının lagnası.** Sayı `n` (1–108) zodyağın `n`. navamsasını gösterir; lagna o
   navamsanın ortasıdır: `(n − 1) × 3°20′ + 1°40′`, sidereal boylam. 0 ve 109 geçersizdir.
   Burç başına dokuz sayı düşer (1–9 Koç, 10–18 Boğa … 100–108 Balık). Grahalar anın
   grahalarıdır; evler bu lagnadan sayılan burç evleridir.
2. **Burç türü.** Koç, Yengeç, Terazi, Oğlak yürük (`Movable`); Boğa, Aslan, Akrep, Kova sabit
   (`Fixed`); İkizler, Başak, Yay, Balık çift doğalı (`Dual`).
3. **Doğuş biçimi** aşağıdaki tablodan.
4. **Ev ve lord.** Ev, lagna burcundan sayılan burçtur (lagna burcu 1). Bir evin lordu, o
   burcun lordudur (tablo aşağıda). Lordun evi, bulunduğu burcun lagnadan sayısıdır.
5. **Dignity** (v1 kuralı, sırayla): yücelme burcu (Ay ve Merkür'de yalnızca moolatrikona
   aralığından önceki dereceler); düşüş burcu (yücelmenin karşısı); moolatrikona aralığı; kendi
   burcu; değilse beşli ilişki: burcun lorduyla doğal ilişki (dost, nötr, düşman) ile geçici
   ilişkinin (lordun bulunduğu burç, grahanın burcundan 2, 3, 4, 10, 11 ya da 12. ise dost,
   değilse düşman) birleşimi: dost + dost `GreatFriend`, dost + düşman `Neutral`, nötr + dost
   `Friend`, nötr + düşman `Enemy`, düşman + dost `Neutral`, düşman + düşman `GreatEnemy`.
6. **Retro** girdideki `retrograde` listesidir. **Yanma:** grahanın Güneş'e açısal uzaklığı orbun
   içindeyse (sınır dahil) yanıktır. Orblar: Ay 12°, Mars 17°, Merkür 14° (retro 12°),
   Jüpiter 11°, Venüs 10° (retro 8°), Satürn 15°. Güneş ve düğümler yanmaz.
7. **İki lordun bağı.** Lagna lordu ile odak evinin lordu aynı grahaysa `same_graha` doğrudur ve
   bağ aranmaz. Değilse sırayla: aynı burçtalarsa `Conjunct`; her biri diğerinin sahip olduğu bir
   burçtaysa `Exchange`; bakış için her grahanın tam bakışla baktığı burçlar bulunur (kendi
   burcundan sayılır: herkes 7.; Mars ayrıca 4. ve 8.; Jüpiter 5. ve 9.; Satürn 3. ve 10.):
   ikisi de diğerinin burcuna bakıyorsa `MutualAspect`, yalnızca lagna lordu bakıyorsa
   `LagnaLordAspects`, yalnızca odak lordu bakıyorsa `FocusLordAspects`. Birden çok bağ birlikte
   geçerli olabilir; hiçbiri yoksa liste boştur.
8. **Ev sayıları.** `focus_lord_from_lagna_lord`: odak lordunun burcunun lagna lordunun
   burcundan sayısı (aynı burç 1); `lagna_lord_from_focus_lord` tersidir.
9. **Doğal sınıf (K6).** Jüpiter ve Venüs iyicil; Güneş, Mars, Satürn, Rahu, Ketu kötücül; Ay,
   Güneş'ten uzanımı `[0°, 180°)` ise iyicil, değilse kötücül (tam 180° kötücül); Merkür aynı
   burçta kötücül varken iyicil yoksa kötücül, değilse iyicil.
10. **Kendralar** 1, 4, 7 ve 10. evlerdir; her birindeki grahalar Güneş, Ay, Mars, Merkür,
    Jüpiter, Venüs, Satürn, Rahu, Ketu sırasıyla ve sınıflarıyla yazılır. Odak evindeki grahalar
    aynı biçimdedir.
11. **Ay.** Burç ve lagnadan ev; nakshatra ve pada sidereal boylamdan (bir pada 3°20′, bir
    nakshatra dört pada); tithi, Güneş'ten uzanımın 12°'lik dilimidir (0°–12° 1. tithi; tam 180°
    16. tithi); 1–15 `Shukla`, 16–30 `Krishna`. `aspected_by`: Ay'ın burcuna tam bakışla bakan
    grahalar (madde 7'nin bakışları), Güneş, Mars, Merkür, Jüpiter, Venüs, Satürn sırasıyla.
12. **Lagna yoksa** (`lagna_lon` ve `number` ikisi de `null`) `lagna` ve `focus` `null`'dır ve
    Ay'ın evi `null`'dır. **Odak evi seçilmemişse** `focus` `null`'dır.

## Tablolar

Doğuş biçimi (BPHS, burç nitelikleri bölümü):

| Biçim | Burçlar |
|---|---|
| Başla doğan (`HeadFirst`) | İkizler, Aslan, Başak, Terazi, Akrep, Kova |
| Sırtla doğan (`BackFirst`) | Koç, Boğa, Yengeç, Yay, Oğlak |
| İki yönlü (`BothWays`) | Balık |

Burç lordları, yücelme ve moolatrikona (v1; BPHS, graha nitelikleri bölümü):

| Graha | Kendi burçları | Yücelme burcu | Moolatrikona |
|---|---|---|---|
| Güneş | Aslan | Koç | Aslan 0°–20° |
| Ay | Yengeç | Boğa | Boğa 3°–30° |
| Mars | Koç, Akrep | Oğlak | Koç 0°–12° |
| Merkür | İkizler, Başak | Başak | Başak 15°–20° |
| Jüpiter | Yay, Balık | Yengeç | Yay 0°–10° |
| Venüs | Boğa, Terazi | Balık | Terazi 0°–15° |
| Satürn | Oğlak, Kova | Terazi | Kova 0°–20° |

Doğal ilişkiler (yazılmayan nötrdür):

| Graha | Dostları | Düşmanları |
|---|---|---|
| Güneş | Ay, Mars, Jüpiter | Venüs, Satürn |
| Ay | Güneş, Merkür | — |
| Mars | Güneş, Ay, Jüpiter | Merkür |
| Merkür | Güneş, Venüs | Ay |
| Jüpiter | Güneş, Ay, Mars | Merkür, Venüs |
| Venüs | Merkür, Satürn | Güneş, Ay |
| Satürn | Merkür, Venüs | Güneş, Ay, Mars |

## Sahibin onayını bekleyen klasik tablolar ve bilinen varyant noktaları

Doğuş biçimi tablosu tasarım belgesinde yazılıdır ve klasik derlemelerin yaygın biçimidir; sayı
yöntemi de tasarım belgesindeki biçimdedir. İkisi bu depoya bellekten aktarılmıştır, basılı bir
kaynakla karşılaştırılmamıştır. Aşağıdaki noktalarda gelenekler ayrışır. Karşılaştırma bir
insanın işidir: elinde basılı bir kaynak bulunan kişi her satırı denetler, kaynağı genel adıyla
(alıntı yapmadan) ve sonucu aşağıdaki tabloya yazar. Fark bulunursa tablo ayrı bir commit'le
düzeltilir ve bu dizindeki örnekler kuraldan yeniden türetilir.

| No | Nokta | Bu depodaki okuma | Bilinen diğer okuma |
|---|---|---|---|
| 1 | İkizler'in doğuş biçimi | Başla doğan | İkizler'i gece burçlarıyla birlikte sırtla doğan sayan listeler (gece burçları Koç, Boğa, İkizler, Yengeç, Yay, Oğlak'tır; doğuş biçimi listesiyle bir burç ayrışır) |
| 2 | Balık | İki yönlü | Balık'ı başla doğanlara katan iki sınıflı listeler |
| 3 | Sayının lagnası | 1–108: `n`. navamsanın ortası (`(n − 1) × 3°20′ + 1°40′`) | Navamsanın başını alan okuma; sayıyı on ikiye bölüp yalnızca burç veren okuma; 1–249 sayı sistemi (tasarım belgesinde kapsam dışı) |
| 4 | Sayı haritasının evleri | Sayı lagnasından burç evleri; ev sistemi ayarı uygulanmaz | Anın gerçek lagnasını ikinci bir lagna olarak yanında gösteren uygulamalar |
| 5 | İki lordun bakışı | Burç temelli tam bakışlar (özel bakışlar dahil); kısmi bakış ve orb yok | Derece temelli (sphuta) bakış; Tajika bakışları ve yogaları (tasarım belgesinde kapsam dışı) |
| 6 | Değişim | Her lord diğerinin sahip olduğu herhangi bir burçta | Yalnızca ilgili iki evin burçları arasındaki değişimi sayan dar okuma |
| 7 | Ay'ın aldığı bakışlar | Düğümler bakış atmaz; Ay'la aynı burçtaki graha "bakan" sayılmaz | Rahu ve Ketu'ya 5., 7. ve 9. bakış veren okumalar |

| Denetleyen | Tarih | Kaynak (genel ad) | Sonuç |
|---|---|---|---|
| — | — | — | Bekliyor: sahibine devredildi (Plan 21 Task 8) |

## Gözden geçirme kaydı

Gözden geçiren (motoru yazan dışında biri) her örnekte yalnızca `input`'tan başlar, motoru
çalıştırmadan ve Rust kaynağını okumadan lagnayı (sayıdan ya da boylamdan), burç türünü ve
doğuş biçimini, lagna lordunun burcunu, evini, dignity'sini ve yanmasını, dört kendrayı, odak
evinin burcunu ve lordunu, iki lordun bağını ve ev sayılarını, Ay'ın burcunu, evini,
nakshatrasını, padasını, tithisini ve aldığı bakışları tasarım belgesinden, yukarıdaki okuma
kurallarından ve tablolardan yeniden hesaplar; sonucunu `expected` ile karşılaştırır ve aşağıya
adını, tarihini ve bulduğu farkları yazar.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
| Claude Opus 5.5 (Plan 21 Task 8 gözden geçiren ajanı; Rust kaynağı okunmadan, ayrı bir Python betiğiyle) | 2026-11-27 | 14/14 örnek `input`'tan yeniden türetildi ve `expected` ile birebir uyuştu (lagna, burç türü ve doğuş biçimi, iki lord ve dignity, retro, yanma, kendralar, odak evi, bağlar ve ev sayıları, odak evindeki grahalar, Ay ve aldığı bakışlar); fark yok. Tablolar aynı kaynaktan (spec ve bu belge) geldiği için bu uyum aritmetiği ve okumayı doğrular, tabloları değil; tablo onayı Step 6'dadır. |
