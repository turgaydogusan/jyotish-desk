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
