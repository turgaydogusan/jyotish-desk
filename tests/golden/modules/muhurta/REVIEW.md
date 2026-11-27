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
| Plan 20 Task 2'nin gözden geçiren ajanı (motoru ve örnekleri yazan dışında); son incelemede yeniden koşuldu | 2026-11-27 | 7/7 uyuştu: yedi örnekte 140 denetim, fark yok. Hesap yalnızca `input`'tan başlar, motoru çalıştırmaz ve kendi yazdığı listeleri kullanır (varaya göre üç dilim sırası, gün lordları ve hora döngüsü, tithi grupları, yedi doğa listesi, iki elverişli küme, sayım başlangıcını sayan tara ve burç sayımı); bu listeler son incelemede yukarıdaki tablolar ve okuma kurallarıyla satır satır karşılaştırıldı. Sınır: gözden geçiren aynı görevde motorun kaynağını da okudu ve tablolar iki hesapta da aynı aktarımdan gelir; bu kayıt kuralların uygulanışını ve aritmetiği doğrular, tabloların kendisini (yukarıdaki onay tablosu) ve okuma seçimlerini değil. |
