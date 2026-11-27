# Geçiş araçları: okuma kuralları, onay bekleyen seçimler ve gözden geçirme kaydı

Kaynak: yol haritası (`docs/superpowers/specs/2026-11-27-eksikler-yol-haritasi.md`) §4.7–§4.10 ve
Plan 26'nın kararları. Çekirdek: `crates/jyotish-core/src/transit_tools/`; komutlar:
`app/src-tauri/src/transit_tools_commands/`.

Bu dizinde JSON örnek yoktur. El örnekleri birim testleridir (`lords.rs`, `sankranti.rs`,
`ephemeris_table.rs` ve `transit_tools_commands/step.rs` içindeki `mod tests`); her beklenen
değerin türetmesi testin yorumunda adım adım yazılıdır. Değerler kuraldan elle türetilmiştir;
motor çıktısı ve başka bir yazılımın çıktısı buraya ve testlere kopyalanmaz (K9). Bir değer
motorla uyuşmazsa önce kural yeniden okunur; beklenen değer motor çıktısına bakarak düzeltilmez.

**Bu planda bellekten yazılan klasik tablo yoktur.** Tek tablo (nakshatra lordları) v1'in
Vimshottari sırasının üç kez yazılmış biçimidir ve testte o sırayla karşılaştırılır; paneller
"doğrulanmamış tablo" notu taşımaz. Aşağıdaki "onay bekleyen" maddeler okuma seçimleridir (Y5).

## 1. Nakshatra lordları

1. **Tablo.** `NAKSHATRA_LORDS`: yirmi yedi nakshatranın lordu, nakshatra sırasıyla. Dokuzlu
   sıra (Ketu, Venüs, Güneş, Ay, Mars, Rahu, Jüpiter, Satürn, Merkür) üç kez yinelenir: n.
   nakshatranın lordu sıranın ((n − 1) mod 9). adıdır.
2. **Arayüz lordu türetmez.** Tablo bağlama dosyasına sabit olarak çıkar (`bindings.ts`,
   `NAKSHATRA_LORDS`); arayüz nakshatra numarasının satırını okur (`panels/nakshatraLord.ts`).
   `GrahaPosition`, `TransitGraha` ve `SkyEvent` tiplerine alan eklenmez.
3. **Üç yüzey.** Geçiş tablosunda "Nakshatra lordu" sütunu her zaman görünür. Graha tablosunda
   sütun isteğe bağlıdır (`settings.nakshatraLord`), varsayılan sütunlar ve baskı tablosu değişmez;
   lagna satırı da lordunu yazar; doğum saati bilinmiyorsa Ay'ın lordu "≈" taşır. Olay aramasında
   yalnızca nakshatra girişi satırı, girilen nakshatranın lordunu aynı hücrede yazar.
4. **Ay'ın önümüzdeki nakshatra girişleri** olay aramasının hazır seçimidir (tür: nakshatra
   girişi; graha: Ay); ayrı komut ve ayrı liste yoktur.

### 1.1 Onay bekleyen noktalar (sahibi)

- [ ] Graha tablosunda sütunun varsayılan olarak gizli olması (öteki okuma: varsayılan sütun).
- [ ] Olay aramasında lordun yalnızca nakshatra girişi satırında yazılması (öteki okuma: yeni ay
      ve dolunay satırında Ay'ın nakshatra lordu da yazılır).

## 2. Sankranti listesi (yöntem `sankranti.sidereal_v1`)

### 2.1 Okuma kuralları

1. **Giriş.** Sankranti, Güneş'in sidereal bir burca girdiği andır: sidereal boylamın 30°'nin
   katından geçişi. An `timeline` yürüyücüsüyle 0,05 saniye çözünürlükte bulunur.
2. **Çerçeve.** Liste her zodyak kipinde seçili ayanamsayla sidereal boylamdan kurulur.
   Tropikal-rasi kipinde haritanın burçları tropikaldir (K5) ve olay aramasının Güneş burç
   girişleri tropikal girişlerdir; Sankranti listesi bundan bilerek ayrılır.
3. **Yıl.** Yerin takvim yılı: yerin saat diliminde 1 Ocak 00:00'dan sonraki 1 Ocak 00:00'a.
   Olağan bir yılda on iki giriş vardır; yılın ilki Oğlak (Makara) girişidir.
4. **Giriş anının olguları.** Tithi (Ay − Güneş uzanımının 12°'lik dilimi) ve paksha, Ay'ın
   nakshatrası ve padası, vara. Üçü de v1'in `compute_panchang` hesabıdır. Vara gün doğumunda
   değişir ve seçilen yerin gün doğumuna göre okunur: gün doğumundan önceki bir giriş önceki
   günün varasını taşır.
5. **Gün doğumu olmayan gün.** Vara, günü 06:00 yerel ortalama saatte başlatarak okunur ve liste
   `SunriseUnavailable` uyarısını taşır (v1'in panchang kuralı).
6. **Saat.** Anlar seçilen yerin saat diliminde gösterilir; her satır kendi anının UTC ofsetini
   taşır (yaz saati yıl içinde değişebilir).
7. **Kapsam dışı.** Girişin kutlu süresi (punya dilimi), girişin yönü ve adı (ör. burca göre
   özel adlar) hesaplanmaz ve yazılmaz.

### 2.2 Onay bekleyen noktalar (sahibi)

- [ ] Listenin tropikal-rasi kipinde de sidereal verilmesi (K5'ten bilinçli ayrılma).
- [ ] Varanın gün doğumuna göre okunması (öteki okuma: yerin takvim günü, gece yarısından).
- [ ] Yılın, seçilen yerin takvim yılı olması (öteki okuma: Mesha girişinden Mesha girişine).

## 3. Efemeris tablosu (yöntem `ephemeris_table.v1`)

### 3.1 Okuma kuralları

1. **Satır.** Seçilen yerin takvim ayının her günü için bir satır; satırın anı o günün başıdır:
   yerin duvar saatiyle 00:00. Yaz saati gece yarısında başlıyorsa (00:00 yoksa) satır günün
   ilk geçerli anındadır, yani geçiş anında (São Paulo 2017-10-15: 03:00 UT = 01:00 yerel);
   o gün önceki satırdan 24, sonraki satıra 23 saat uzaktadır. Saat geri alınırken 00:00 iki
   kez yaşanıyorsa önceki an alınır. Saat dilimi bir günü tümden atlıyorsa (Pacific/Apia
   2011-12-30, Pacific/Kiritimati 1994-12-31) o gün yerde yoktur: satırı yazılmaz, ay hata
   vermez ve 29–30 satır döner; işaretler atlanan günün üstünden ardışık satırları karşılaştırır.
   Ayı kapatan an, sonraki ayın ilk var olan gününün başıdır.
2. **Konum.** Dokuz grahanın boylamı transit hesabıyla aynıdır: burç ve burç içi derece rasi
   çerçevesindedir (haritanın zodyak kipini izler, K5); nakshatra ve pada sidereal boylamdandır.
   Ayanamsa ve düğüm türü açık haritanın ayarıdır. Retro, boylam hızının eksi olmasıdır; Rahu ve
   Ketu'da retro işareti yazılmaz.
3. **Giriş işareti.** Graha, satırın anı ile bir sonraki satırın anı arasında başka bir burca
   giriyorsa satır girilen burcu taşır. Ayın son gününün işareti, sonraki ayın ilk gününün
   başındaki konumdan okunur.
4. **Dönüş işareti.** Mars, Merkür, Jüpiter, Venüs ve Satürn için: graha aynı süre içinde retroya
   ya da düze dönüyorsa satır dönülen yönü taşır. Düğümlerin yönü işaretlenmez.
5. **İşaretlerin sınırı.** İşaret, ardışık iki satırın karşılaştırmasıdır: kesin an olay
   aramasındadır. Bir gün içinde burç sınırını geçip geri dönen durağan bir graha işaret almaz.
6. **Aralık.** Bir ay (28–31 satır); yıl 1–2999. Ay verilmezse yerin bugünkü ayıdır. Ephemeris
   dosyalarının dışındaki yıllarda (1800 öncesi, 2400 ve sonrası) tablo `LowPrecision` uyarısını
   taşır.
7. **Kapsam dışı.** CSV ve baskı (K10); enlem, hız ve deklinasyon sütunları; yıl görünümü;
   00:00 UT satırı.

### 3.2 Onay bekleyen noktalar (sahibi)

- [ ] Satır saatinin seçilen yerin yerel gece yarısı olması (öteki okuma: 00:00 UT).
- [ ] Varsayılan yerin bulunulan yer olması (öteki okuma: haritanın doğum yeri).
- [ ] Burç ve derecenin haritanın zodyak kipini izlemesi (tropikal-rasi kipinde tropikal).

## 4. Geçiş anının adımı ve oynatma

### 4.1 Okuma kuralları

1. **Birimler.** Saat, gün, hafta, ay, yıl. Varsayılan birim gündür. Her tıklama bir adımdır.
2. **Saat adımı mutlaktır:** tam 3 600 saniye; yaz saati geçişinde de böyledir.
3. **Gün, hafta, ay ve yıl adımı duvar saatini korur.** An, tarayıcının saat dilimindeki takvim
   tarihine ve saatine çevrilir; takvimde yürünür; aynı duvar saati yeniden çözülür. Yaz saati
   geçişinden geçen bir gün adımı 23 ya da 25 saat sürer.
4. **Ay sonu.** Varılan ayda o gün yoksa ayın son günü alınır: 31 Ocak + 1 ay = 28 Şubat (artık
   yılda 29 Şubat); 29 Şubat ± 1 yıl = 28 Şubat. Bu yüzden ay adımı her zaman geri alınamaz:
   31 Ocak → 28 Şubat → 28 Ocak.
5. **Yaz saati boşluğu.** Varılan duvar saati yoksa (ör. 02:30, saatlerin 02:00'den 03:00'e
   atladığı gün) boşluk kadar ileri alınır: 03:30. Örtüşmede (aynı duvar saati iki kez) önceki an
   alınır. İkisi de transit anının v1 kuralıdır.
6. **Aralık.** 1–2999 yılları; dışına çıkan adım `invalid_transit_input { field: "range" }` verir
   ve an değişmez.
7. **Çıkış anı.** Transit deposunun anıdır: an bir Jülyen günüyse o, "şimdi" ya da yazılmış bir
   duvar saatiyse gösterilen yanıtın anı. İlk adımdan sonra an "şimdi" olmaktan çıkar.
8. **Eşzamanlılık.** Bir adım isteği sürerken yeni adım başlatılmaz. İstek sürerken an başka bir
   yoldan değişirse, harita değişirse ya da panel kapanırsa geç gelen yanıt düşürülür.

### 4.2 Onay bekleyen noktalar (sahibi)

- [ ] Gün ve hafta adımının duvar saatini koruması (öteki okuma: tam 24 ve 168 saat).
- [ ] Duvar saatinin tarayıcının diliminde okunması (öteki okuma: haritanın ya da bulunulan
      yerin dilimi).
- [ ] Ay sonunda ayın son gününe düşme ve boşlukta ileri alma.
