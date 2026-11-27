# Varshaphala el örnekleri: okuma kuralları, tablolar, onay bekleyen noktalar ve gözden geçirme kaydı

Kaynak: eksikler yol haritası §4.14 (ilk yarı) ve Plan 27'nin kararları; klasik dayanak Tajika
geleneğinin yıllık harita (varshaphala) bölümleridir (genel gönderme; alıntı yapılmaz). Bu
dizinde iki dizi durur:

- `muntha/`: iki lagna boylamı (doğum haritasının ve yıllık haritanın lagnası; sidereal sayılır)
  ve tamamlanan yıl. Her dosya munthanın burcunu, lordunu, iki haritadaki evini ve iki lagnanın
  lordunu yazar.
- `mudda/`: Ay'ın sidereal boylamı, tamamlanan yıl ve yılın gün olarak uzunluğu. Her dosya doğum
  nakshatrasını, ilk lord kuralının bölümden artanını, ilk lordu ve dokuz dönemin bitişini (yılın
  başından gün sayısı) yazar; üç dosya ilk dönemin dokuz alt dönemini de yazar.

Dönüş anı bir tablo değildir ve burada örneği yoktur: gerçek motorla
`app/src-tauri/tests/varsha.rs`'te denetlenir (yıllık haritanın Güneş'i doğumdaki sidereal
boylamından bir yay saniyesinden az ayrışır; aynı dönüşü olay araması da bir saniye içinde bulur).

Ephemeris yoktur. Değerler kuraldan türetilmiştir ve `derivation` alanında adım adım yazılıdır;
motor çıktısı buraya kopyalanmaz (K9). Bir değer motorla uyuşmazsa önce kural yeniden okunur.
Testler: `crates/jyotish-core/tests/varsha_golden.rs`.

## Okuma kuralları

1. **Tamamlanan yıl.** Yıllar doğumdan sayılır: tamamlanan yıl 0 doğum yılıdır (doğumdan ilk
   dönüşe), 1 ilk dönüşle başlayan yıldır. "N. yıl" diye yazılan sayı tamamlanan yılın bir
   fazlasıdır. En büyük tamamlanan yıl 120'dir.
2. **Dönüş anı.** Yıl, Güneş'in doğumdaki boylamına döndüğü anda başlar. Boylam her zodyak kipinde
   sidereal boylamdır (haritanın seçili ayanamsasıyla). Yılın sonu sonraki dönüştür; yılın
   uzunluğu iki dönüşün arasıdır (yaklaşık 365,2564 gün).
3. **Yıllık harita.** Dönüş anının haritasıdır: doğum haritasının ayarlarıyla (ayanamsa, düğüm, ev
   sistemi, zodyak kipi) ve seçilen yerde hesaplanır. Yer doğum yeridir; astrolog isterse bulunulan
   yeri seçer. Yer dönüş anını değiştirmez, yalnızca lagnayı ve evleri değiştirir.
4. **Muntha.** Doğumda lagnanın burcundadır ve tamamlanan her yılda bir burç ilerler: burcu, doğum
   lagnasının burcundan tamamlanan yıl kadar burç ötesidir (on ikide bir başa döner). Lordu o
   burcun lordudur. Evi burç evidir: doğum lagnasından ve yıllık lagnadan ayrı ayrı sayılır.
   Burçlar rasi çerçevesindedir. Munthanın yıl içindeki derece ilerleyişi hesaplanmaz.
5. **Lagna lordları.** Yıllık lagnanın ve doğum lagnasının lordu, burçlarının lordudur (aşağıdaki
   tablo). Yıl lordu seçilmez (Plan 28).
6. **Mudda dasanın ilk lordu.** Doğum nakshatrasının sırası (Ay'ın sidereal boylamından; Ashwini 1,
   Revati 27) ile tamamlanan yıl toplanır, 2 çıkarılır ve 9'a bölünür. Bölümden artan sayı mudda
   sırasındaki lordu verir: 1 Güneş, 2 Ay, 3 Mars, 4 Rahu, 5 Jüpiter, 6 Satürn, 7 Merkür, 8 Ketu,
   0 Venüs. Toplam eksiye düşerse (Ashwini ve doğum yılı: −1) artan yine 0–8 arasında okunur
   (−1 = −1 × 9 + 8).
7. **Dönemler.** Dokuz dönem ilk lorddan başlar ve mudda sırasını çevrimsel izler. İlk dönem yılın
   başında (dönüş anında) başlar ve tamdır: Ay'ın nakshatrada geçtiği yay için kesinti yapılmaz.
   Her dönem yılın `gün(lord) / 360` kadarını alır; yıl dönüşten dönüşe sayıldığı için dokuz dönem
   yılı boşluksuz doldurur ve son dönem sonraki dönüşte biter.
8. **Alt dönemler.** Her dönemin dokuz alt dönemi o dönemin lordundan başlar, aynı sırayı izler ve
   dönemi aynı paylarla böler.
9. **Doğum saati bilinmiyorsa** yıllık harita, muntha ve lagna lordları verilmez (hepsi lagna
   ister). Dönüş anı ve mudda dasa öğle konumlarından hesaplanır ve yaklaşık sayılır.
10. **Sayı prashnası.** Doğum lagnası sayıdan gelir ve muntha o lagnadan sayılır; yıllık harita
    dönüş anının kendi lagnasıyla hesaplanır.

## Tablolar

Burç lordları (uygulamanın ortak tablosu; düğümler burç lordu değildir):

| Burç | Koç | Boğa | İkizler | Yengeç | Aslan | Başak | Terazi | Akrep | Yay | Oğlak | Kova | Balık |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Lord | Mars | Venüs | Merkür | Ay | Güneş | Merkür | Venüs | Mars | Jüpiter | Satürn | Satürn | Jüpiter |

Mudda dasa: sıra, 360 günlük yıldaki gün ve ilk lord kuralının artanı. Günler Vimshottari
yıllarının üç katıdır (120 yıl → 360 gün).

| Sıra | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|---|
| Lord | Güneş | Ay | Mars | Rahu | Jüpiter | Satürn | Merkür | Ketu | Venüs |
| Gün | 18 | 30 | 21 | 54 | 48 | 57 | 51 | 21 | 60 |
| Artan | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 0 |

Nakshatra bölümü: her nakshatra 13°20′'dır; sıra `tam kısım(boylam ÷ 13°20′) + 1`. Sınır sonraki
nakshatraya aittir (120° tam Magha'nın, 10. nakshatranın başıdır).

## Sahibin onayını bekleyen kurallar ve bilinen varyant noktaları

Kurallar bellekten aktarılmıştır (yol haritası Y4). Her satır basılı bir Tajika kaynağıyla
karşılaştırılmayı bekler; onaya dek dasa ağacının mudda görünümü "doğrulanmamış tablo" notunu
taşır. Yürüten ajan bu noktaları başka bir yazılımla ya da çevrimiçi bir kaynakla karşılaştırmaz.

| No | Nokta | Bu plandaki okuma | Bilinen öteki okuma | Durum |
|---|---|---|---|---|
| 1 | Dönüşün çerçevesi | Sidereal boylama dönüş, seçili ayanamsayla; her zodyak kipinde | Tropikal boylama dönüş (her yıl yaklaşık yirmi dakika erken gelir; lagnayı değiştirir) | Bekliyor |
| 2 | Yıllık haritanın yeri | Doğum yeri; astrolog bulunulan yeri seçebilir | Yalnızca doğum yeri; yalnızca o yıl yaşanan yer | Bekliyor |
| 3 | Munthanın sayımı | Doğum lagnasının burcu + tamamlanan yıl | Süren yılın sayısı (bir burç ileri) | Bekliyor |
| 4 | Munthanın çerçevesi | Rasi çerçevesi (tropikal-rasi kipinde tropikal lagna burcu) | Her kipte sidereal lagna burcu | Bekliyor |
| 5 | Munthanın derecesi | Hesaplanmaz; yalnızca burç | Lagnanın derecesiyle başlar, ayda 2°30′ ilerler | Bekliyor |
| 6 | İlk lord kuralının terimleri | Doğum nakshatrası + tamamlanan yıl − 2 | Süren yılın sayısı ile (çıkarılan sayı 3 olur; sonuç aynıdır) ya da çıkarmadan (iki lord kayar) | Bekliyor |
| 7 | Artanın lorda eşlenmesi | 1 Güneş … 8 Ketu, 0 Venüs | Artan 0 ilk lord sayılır (sıra bir kayar) | Bekliyor |
| 8 | Mudda sırası ve günler | Güneş 18, Ay 30, Mars 21, Rahu 54, Jüpiter 48, Satürn 57, Merkür 51, Ketu 21, Venüs 60 | — (toplam 360; sıra Vimshottari sırasıdır) | Bekliyor |
| 9 | İlk dönemin kesintisi | Yok: ilk dönem tamdır | Ay'ın doğum nakshatrasında geçtiği yay oranında ilk dönem kısalır | Bekliyor |
| 10 | Yılın uzunluğu | Dönüşten dönüşe; dönemler bu sürenin payıdır | 360 takvim günü (yılın sonunda beş gün boş); Güneş'in her 1° yolu bir gün | Bekliyor |
| 11 | Alt dönemler | Üst dönemin lordundan başlar; aynı paylar | Alt dönem verilmez | Bekliyor |
| 12 | Doğum saati bilinmeyen harita | Yıllık harita ve muntha verilmez; dönüş anı ve mudda dasa yaklaşık | Hiçbiri verilmez | Bekliyor |

Denetleyen (sahibi), tarih ve kullanılan basılı kaynak:

| Denetleyen | Tarih | Kaynak | Sonuç |
|---|---|---|---|
| — | — | — | Bekliyor: sahibine devredildi (Plan 27 Task 8) |

## Gözden geçirme kaydı

Gözden geçiren (örnekleri yazan dışında biri) her örnekte yalnızca `input`'tan başlar, motoru
çalıştırmadan ve Rust kaynağını okumadan, yukarıdaki okuma kurallarından ve tablolardan munthayı
ve lordları (`muntha/`) ya da doğum nakshatrasını, artanı, ilk lordu ve dönem bitişlerini
(`mudda/`) yeniden hesaplar; sonucunu `expected` ile karşılaştırır ve aşağıya adını, tarihini ve
bulduğu farkları yazar.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
| Plan 27'yi yazan oturum (örneklerin yazarı) | 2026-11-28 | Örnekler, motorun kaynağından ayrı yazılmış bir türetmeyle kural metninden üretildi ve motorla karşılaştırıldı: `muntha/` beş örnek; `mudda/` dokuz örnek, 81 dönem bitişi ve üç örnekte 27 alt dönem bitişi; fark yok. Sınır: türetmeyi ve motoru aynı yazar yazdı; kuralın okunuşundaki ortak bir yanlış ikisinde de aynıdır. Bu kayıt aritmetiği ve kuralların uygulanışını doğrular, okuma seçimlerini (yukarıdaki onay tablosu) değil. |
| Plan 27 Task 8'i yürüten ajan (örnekleri yazan oturum dışında) | 2026-11-28 | 14/14 uyuştu: beş muntha örneği; dokuz mudda örneğinde 81 dönem ve 27 alt dönem bitişi, fark yok. Hesap yalnızca `input`'tan ve bu dosyanın kural ve tablolarından başlar, motoru çalıştırmaz. Sınır: yürüten ajan aynı planın kodunu da yazdı; bu kayıt kuralların uygulanışını ve aritmetiği doğrular, okuma seçimlerini (onay tablosu) değil. |
