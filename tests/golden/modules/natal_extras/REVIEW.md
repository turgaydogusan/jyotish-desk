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

## 4. Elle denetim kaydı (Plan 23 Task 8)

Her maddenin başında, otomatik bir karşılığı olup olmadığı yazılıdır. **[otomatik]** maddelerin
karşılığı testlerdedir; yürüten ajan testin adını göstererek işaretler. **[insan]** maddeler
yalnızca basılı kaynağa ya da gerçek uygulamaya bakan bir insan tarafından doğrulanır ve sahibine
devredilir; devir sonraki planı engellemez (Y3). Gerçek uygulamada yalnızca deneme danışanları
kullanılır.

- [x] **[otomatik]** Yogi noktası Güneş + Ay + 93°20′'dir ve 360°'de sarılır; yogi nakshatrası
  nitya yogadan yedi ileridedir; üç el örneği (A, B, C) noktayı, yogiyi, eş yogiyi ve avayogiyi
  verir. (`yogi.rs › the_arc_is_seven_nakshatras_and_the_point_wraps`,
  `› hand_examples_give_the_point_the_yogi_the_co_yogi_and_the_avayogi`,
  `› the_yogi_nakshatra_is_the_nitya_yoga_shifted_by_seven`; `tests/natal_extras.rs`)
- [x] **[otomatik]** Avayogi altıncı nakshatranın lordudur; ikinci yazım (nokta + 186°40′) aynı
  lordu verir; dokuz yogi–avayogi çifti 1.2'deki tabloyla aynıdır.
  (`yogi.rs › avayogi_is_the_sixth_nakshatra_and_agrees_with_the_added_arc`,
  `› nakshatra_lords_repeat_every_nine`)
- [x] **[otomatik]** Eş yoginin burcu rasi çerçevesindedir; `Sidereal` kipinde öteleme sıfırdır;
  nakshatra ve lordlar kipten bağımsızdır. (`yogi.rs › the_point_sign_follows_the_rasi_frame`;
  `collect.rs › panchang_extras_of_the_fake_sky`; `tests/natal_extras.rs`, tropikal-rasi kipindeki örnekler)
- [x] **[otomatik]** Dagdha rasi tablosu burçtan yazılan ikinci tabloyla aynıdır (30 hücre); iki
  paksha aynı satırı okur; dolunay ve yeni ayda boştur; aralık dışı tithi boş liste verir.
  (`dagdha.rs › two_signs_on_thirteen_tithis_four_on_the_fourteenth_none_on_the_fifteenth`,
  `› the_table_agrees_with_its_second_writing_by_sign`, `› out_of_range_tithis_have_no_dagdha_sign`)
- [x] **[otomatik]** Raporun tithisi v1'in panchangındaki tithidir; Delhi 2000-01-01 için yogi
  Rahu (Swati), avayogi Venüs (Purva Ashadha), dagdha Yay ve Balık.
  (`dagdha.rs › extras_join_the_yogi_facts_the_tithi_and_the_dagdha_signs`; `tests/natal_extras.rs`)
- [x] **[otomatik]** Navatara 27 satırdır, doğum nakshatrasından başlar, üç tur yürür; tara ve tur
  adım adım sayımla aynıdır (729 çift); her graha kendi nakshatrasının satırındadır; geçiş taraları
  transit komutunun nakshatralarından sayılır. (`navatara.rs` beş test;
  `collect.rs › navatara_of_the_fake_sky_with_and_without_a_transit_moment`; `tests/natal_extras.rs`)
- [x] **[otomatik]** Vimshopaka: dört kümenin ağırlıkları 20'ye toplanır; altı basamak 20, 18, 15,
  10, 7, 5'tir; yücelme ve moolatrikona ayrı basamak değildir; geçici dostluk D1'den okunur; iki el
  örneği (Güneş Aslan 0°, Ay Boğa 0°) dört değeri verir; her değer 5 ile 20 arasındadır.
  (`vimshopaka.rs` altı test)
- [x] **[otomatik]** Vimshopaka'nın varga burçları v1'in on altı varga haritasıyla, basamakları
  Shadbala'nın saptavargaja kalemiyle (yedi vargada) aynıdır. (`tests/natal_extras.rs`)
- [x] **[otomatik]** Doğum saati bilinmiyorsa üç rapor da döner ve `approximate` işaretlidir;
  sayı prashnasında raporlar sayısız haritayla aynıdır. (`collect.rs` dört test;
  `PanchangExtras.test.tsx › unknown birth time …`, `NavataraPanel.test.tsx › unknown birth time …`,
  `VimshopakaPanel.test.tsx › unknown and approximate birth times …`)
- [x] **[otomatik]** Geçiş anı haritadan önce denetlenir; hatası yalnızca alan adını taşır
  (`invalid_transit_input { field: transit_jd_ut }`). (`collect.rs ›
  collectors_check_the_input_before_and_through_the_chart`; `tests/natal_extras.rs`)
- [x] **[otomatik]** Üç komut hata ayıklama derlemesinde 20 ms'nin altındadır.
  (`tests/natal_extras.rs`: `assert!`)
- [x] **[otomatik]** Raporlar panel açılmadan istenmez; Panchang paneli beş uzvu komut beklemeden
  çizer ve ek tabloyu altına ekler; ek tablonun hatası beş uzvu kaldırmaz.
  (`PanchangExtras.test.tsx` yedi test; e2e `natal extras: …`)
- [x] **[otomatik]** Navatara paneli dokuz tarayı üç turda, lord ve kümeyle yazar; transit anını
  Rust'a olduğu gibi gönderir; an değişirken tablolar yerinde durur; transit hatasında doğum tablosu
  durur. (`NavataraPanel.test.tsx` sekiz test; e2e)
- [x] **[otomatik]** Vimshopaka paneli yedi grahanın dört değerini üç ondalıkla yazar; "Vargalar"
  görünümü on altı vargada burcu ve basamağı gösterir; ayar düzenle birlikte saklanır.
  (`VimshopakaPanel.test.tsx` sekiz test; e2e)
- [x] **[otomatik]** Her metin üç dilde vardır; klasik adlar (tara, nakshatra, küme adları) üç
  dilde aynıdır; Fransızca metinlerde düz kesme işareti yoktur. (`i18n.test.ts`; üç panel testinin
  dil testleri)
- [ ] **[insan]** (sahibine devredildi) Dagdha rasi tablosu basılı kaynakla hücre hücre karşılaştırılır (1.3, 1.5);
  paksha ayrımı olup olmadığına bakılır.
- [ ] **[insan]** (sahibine devredildi) Yogi noktasının yayı (93°20′) ve avayoginin sayımı basılı kaynakla doğrulanır
  (1.5).
- [ ] **[insan]** (sahibine devredildi) Vimshopaka'nın dört ağırlık sütunu ve altı basamağı basılı kaynakla
  karşılaştırılır (3.2, 3.3); yücelme ile moolatrikonanın yeri ve geçici dostluğun okunduğu varga
  için karar verilir (3.4).
- [ ] **[insan]** (sahibine devredildi) Navataranın lagna nakshatrasından da sayılıp sayılmayacağına karar verilir (2.2).
- [ ] **[insan]** (sahibine devredildi) Gerçek uygulamada Panchang paneli: ek tablo beş uzvun altındadır ve panelin
  varsayılan boyutunda kaydırmayla okunur; boyutun büyütülmesi gerekiyorsa karar olarak yazılır.
- [ ] **[insan]** (sahibine devredildi) Gerçek uygulamada Navatara paneli: transit anı değiştirilince geçiş tablosu
  değişir; "Şimdi" çalışır; nakshatralar transit tablosundakilerle aynıdır.
- [ ] **[insan]** (sahibine devredildi) Gerçek uygulamada Vimshopaka paneli: saati "yaklaşık" girilmiş bir deneme
  haritasında not görünür; "Vargalar" tablosu dar panelde yana kayar; açık ve koyu temada, üç
  dilde okunur.
- [ ] **[insan]** (sahibine devredildi) VoiceOver: üç tablo başlıklarıyla duyurulur; kısaltmayla yazılan grahalar tam
  adıyla okunur; "doğrulanmamış tablo" notları okunur.
- [ ] **[insan]** (sahibine devredildi) Denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum verisi,
  koordinat ya da dosya yolu görünmez.

## 5. Gözden geçirme kaydı

| Tarih | Gözden geçiren | Kapsam | Sonuç |
|---|---|---|---|
| 2026-11-27 | Plan 23–25'in son gözden geçirmesi (Claude; motoru ve örnekleri yazan oturumların dışında) | El örneklerinin kuraldan yeniden türetilmesi (yogi A–C, navatara tabanı, Vimshopaka'nın iki örneği) | Fark yok. Elle, yalnızca girdilerden ve bu dosyadaki kurallarla tablolardan, motor çalıştırılmadan yeniden hesaplandı: yogi A, B, C (nokta, nakshatra, pada, yogi, burç, eş yogi, avayogi); tropikal-rasi örneği; `collect.rs`'teki sahte gökyüzü (tithi 8, dagdha İkizler ve Başak; dokuz grahanın nakshatrası, tarası ve turu; on gün sonraki Ay ve Güneş); `navatara.rs` tabanı (dokuz grahanın satırı, tarası ve turu) ve dört grahalık geçiş örneği; Vimshopaka'nın iki örneği (on altı varga burcu, on altı basamak, dört küme toplamı: 388, 389, 394, 385 ve 270, 254, 276, 285,5). Sınır: beklenen değerler testte türetmenin yanında yazılıdır, türetme kör değildir; bu kayıt aritmetiği ve kuralların uygulanışını doğrular, okuma seçimlerini ve bellekten yazılan tabloları (1.5, 2.2, 3.4) değil. |
| | | Sahibinin basılı kaynak onayı (1.5, 2.2, 3.4) | |
