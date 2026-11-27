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
