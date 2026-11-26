# Uyum el örnekleri: okuma kuralları, tablolar ve gözden geçirme kaydı

Kaynak: ileri modüller tasarım belgesi §9 (kuta tablosu, sabit eşlemeler, ek olgular, karşılıklı
yerleşim), §2 (K4 rasi evleri, K5 rasi çerçevesi) ve §4 (`kuja_dosha`). Bu dizindeki her JSON
dosyası iki sentetik haritadır: üst düzeydeki `lagna_lon` ve `grahas` damat rolünün, `bride`
bloğu gelin rolünün haritasıdır. `derivation` sekiz kutayı, toplamı, iki istisnayı, iki Kuja
dosha olgusunu ve karşılıklı yerleşimi sırayla yazar. Değerler kuraldan elle türetilmiştir; motor
çıktısı buraya kopyalanmaz (K9). Bir değer motorla uyuşmazsa önce kural yeniden okunur.

## Okuma kuralları

Tasarım belgesinin açık bıraktığı yerlerde örnekler şu okumayı kullanır:

1. **Ay.** Burç ve burç içi derece boylamdan okunur (sentetik örneklerde rasi çerçevesi
   siderealdir). Nakshatra ve pada: boylam × 3/10'un tam kısmı pada dizinidir (0–107); nakshatra
   dizinin 4'e bölümünün tam kısmı + 1, pada dizinin 4'e bölümünden artan + 1.
2. **Varna.** Brahmin: Yengeç, Akrep, Balık. Kshatriya: Koç, Aslan, Yay. Vaishya: Boğa, Başak,
   Oğlak. Shudra: İkizler, Terazi, Kova. Damat rolünün sınıfı eşit ya da üstünse 1, değilse 0.
3. **Vashya.** Dört ayaklı (`quadruped`): Koç, Boğa, Yay'ın ikinci yarısı, Oğlak'ın ilk yarısı.
   İnsan (`human`): İkizler, Başak, Terazi, Kova, Yay'ın ilk yarısı. Su (`water`): Yengeç, Balık,
   Oğlak'ın ikinci yarısı. Yabanıl (`wild`): Aslan. Böcek (`insect`): Akrep. İlk yarı `[0°, 15°)`
   aralığıdır; tam 15° ikinci yarıdadır. Puan aşağıdaki tablodan, satır damat rolü.
4. **Tara.** Sayım başlangıç nakshatrasını da sayar (aynı nakshatra 1). Tara numarası sayımın
   dokuzlu döngüdeki yeridir (sayım 10 → 1, 18 → 9). 3., 5. ve 7. taralar uygunsuzdur. İki yön
   ayrı sayılır (gelininkinden damadınkine; damadınkinden gelininkine); uygun her yön 1½ verir.
   Damat rolünün "sınıfı" kendi nakshatrasının gelininkinden sayılan tara numarasıdır; gelin
   rolününki tersidir. İki yön birden uygunsuz olamaz: en düşük puan 1½'dir.
5. **Yoni.** Nakshatranın hayvanı tasarım belgesindeki listeden; puan aşağıdaki tablodan
   (bakışımlı).
6. **Graha maitri.** Ay burçlarının lordları (Akrep: Mars, Kova: Satürn) arasındaki doğal ilişki
   (v1'in doğal dostluk tablosu). Aynı lord ya da iki yönlü dost 5; dost + nötr 4; iki yönlü nötr
   3; dost + düşman 1; nötr + düşman ½; iki yönlü düşman 0.
7. **Gana.** Tasarım belgesindeki listeden; puan aşağıdaki tablodan, satır damat rolü.
8. **Bhakoot.** Gelin rolünün burcu damat rolününkinden sayılır (aynı burç 1); ters yönün sayımı
   14 eksi odur (1'de 1). Sayım 2, 12, 5, 9, 6 ya da 8 ise 0, değilse 7.
9. **Nadi.** Nakshatra sırası (Ashwini = 0) mod 6 üzerinden Adi, Madhya, Antya, Antya, Madhya,
   Adi. Farklıysa 8, aynıysa 0.
10. **İstisnalar** puanı değiştirmez. Bhakoot istisnası: puan 0 ve iki lord aynı ya da iki yönlü
    dost. Nadi istisnası: puan 0 ve iki Ay ya aynı nakshatranın farklı padalarında ya da (farklı
    nakshatralardaysa) aynı burçta. Aynı nakshatranın aynı padasında istisna yoktur.
11. **Kuja dosha** her harita için ayrı: Mars lagnadan, Ay'dan ve Venüs'ten 1, 2, 4, 7, 8 ya da
    12. evdeyse o referans `true`'dur; lagna yoksa `from_lagna` `null`'dır.
12. **Karşılıklı yerleşim.** Her tarafın dokuz grahası için, grahanın burcunun diğer tarafın lagna
    burcundan ve Ay burcundan sayısı (aynı burç 1). Diğer tarafın lagnası yoksa ilk değer
    `null`'dır. `expected.overlay.groom` damat rolünün grahalarını (gelin rolünün haritasında),
    `bride` tersini yazar; her satır `[lagnadan, Ay'dan]`, graha sırası Güneş, Ay, Mars, Merkür,
    Jüpiter, Venüs, Satürn, Rahu, Ketu.
13. **`approximate`** iki haritadan birinin lagnası yoksa `true`'dur.

## Puan tabloları

Motorun kullandığı üç tablo (`crates/jyotish-core/src/kuta/kuta_tables.rs`). Satır damat rolünün,
sütun gelin rolünün sınıfıdır.

**Vashya** (sıra: dört ayaklı, insan, su, yabanıl, böcek)

| damat \ gelin | dört ayaklı | insan | su | yabanıl | böcek |
|---|---|---|---|---|---|
| dört ayaklı | 2 | 1 | 1 | 0 | 1 |
| insan | 1 | 2 | ½ | 0 | 1 |
| su | 1 | ½ | 2 | 0 | 1 |
| yabanıl | ½ | 0 | 1 | 2 | 0 |
| böcek | 1 | 1 | 1 | 0 | 2 |

**Gana**

| damat \ gelin | Deva | Manushya | Rakshasa |
|---|---|---|---|
| Deva | 6 | 6 | 0 |
| Manushya | 5 | 6 | 0 |
| Rakshasa | 1 | 0 | 6 |

**Yoni** (bakışımlı; sıra: at, fil, koyun, yılan, köpek, kedi, sıçan, inek, manda, kaplan, geyik,
maymun, firavun faresi, aslan)

| | at | fil | koy | yıl | köp | ked | sıç | ine | man | kap | gey | may | fir | asl |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| at | 4 | 2 | 2 | 3 | 2 | 2 | 2 | 1 | 0 | 1 | 3 | 3 | 2 | 1 |
| fil | 2 | 4 | 3 | 3 | 2 | 2 | 2 | 2 | 3 | 1 | 2 | 3 | 2 | 0 |
| koyun | 2 | 3 | 4 | 2 | 1 | 2 | 1 | 3 | 3 | 1 | 2 | 0 | 3 | 1 |
| yılan | 3 | 3 | 2 | 4 | 2 | 1 | 1 | 1 | 1 | 2 | 2 | 2 | 0 | 2 |
| köpek | 2 | 2 | 1 | 2 | 4 | 2 | 1 | 2 | 2 | 1 | 0 | 2 | 1 | 1 |
| kedi | 2 | 2 | 2 | 1 | 2 | 4 | 0 | 2 | 2 | 1 | 3 | 3 | 2 | 1 |
| sıçan | 2 | 2 | 1 | 1 | 1 | 0 | 4 | 2 | 2 | 2 | 2 | 2 | 1 | 2 |
| inek | 1 | 2 | 3 | 1 | 2 | 2 | 2 | 4 | 3 | 0 | 3 | 2 | 2 | 1 |
| manda | 0 | 3 | 3 | 1 | 2 | 2 | 2 | 3 | 4 | 1 | 2 | 2 | 2 | 1 |
| kaplan | 1 | 1 | 1 | 2 | 1 | 1 | 2 | 0 | 1 | 4 | 1 | 1 | 2 | 1 |
| geyik | 3 | 2 | 2 | 2 | 0 | 3 | 2 | 3 | 2 | 1 | 4 | 2 | 2 | 1 |
| maymun | 3 | 3 | 0 | 2 | 2 | 3 | 2 | 2 | 2 | 1 | 2 | 4 | 3 | 2 |
| firavun faresi | 2 | 2 | 3 | 0 | 1 | 2 | 1 | 2 | 2 | 2 | 2 | 3 | 4 | 2 |
| aslan | 1 | 0 | 1 | 2 | 1 | 1 | 2 | 1 | 1 | 1 | 1 | 2 | 2 | 4 |

Sınıf kimlikleri (JSON'daki yazım): varna `brahmin`, `kshatriya`, `vaishya`, `shudra`; vashya
`quadruped`, `human`, `water`, `wild`, `insect`; tara `1` … `9`; yoni `horse`, `elephant`, `sheep`,
`serpent`, `dog`, `cat`, `rat`, `cow`, `buffalo`, `tiger`, `deer`, `monkey`, `mongoose`, `lion`;
graha maitri lordun adı (`Sun` … `Saturn`); gana `deva`, `manushya`, `rakshasa`; Bhakoot burcun adı
(`Aries` … `Pisces`); nadi `adi`, `madhya`, `antya`.

## Tabloların basılı kaynaklarla karşılaştırılması

Tasarım belgesi §9 üç puan tablosunun iki bağımsız basılı klasik kaynaktan aktarılmasını ister.
Tablolar bu depoya yaygın basılı biçimden aktarılmıştır; bilinen ayrışmalar ve seçimler
`kuta_tables.rs`'in başındaki nottadır. Hücre hücre karşılaştırma bir insanın işidir: iki basılı
kaynağı (ör. bir Kuzey Hint panchangının evlilik uyumu çizelgeleri ve bir muhurta derlemesi)
elinde tutan kişi üç tabloyu ve yukarıdaki satır/sütun yönünü denetler, kaynakları genel adıyla
(alıntı yapmadan) ve bulduğu farkları aşağıya yazar. Fark bulunursa tablo ayrı bir commit'le
düzeltilir, dosya başındaki not güncellenir ve bu dizindeki örnekler yeniden türetilir.

| Denetleyen | Tarih | Kaynaklar (genel ad) | Sonuç |
|---|---|---|---|
| — | — | — | Bekliyor: sahibine devredildi (Plan 19 Task 9) |

## Gözden geçirme kaydı

Gözden geçiren (motoru yazan dışında biri) her örnekte yalnızca `input`'tan başlar, motoru
çalıştırmadan ve Rust kaynağını okumadan sekiz kutayı, toplamı, iki istisnayı, iki Kuja dosha
olgusunu ve on sekiz yerleşim satırını tasarım belgesinden, yukarıdaki okuma kurallarından ve
tablolardan yeniden hesaplar; sonucunu `expected` ile karşılaştırır ve aşağıya adını, tarihini
ve bulduğu farkları yazar.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
