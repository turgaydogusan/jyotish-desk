# İleri modüllerin el örnekleri

`crates/jyotish-core/tests/modules_golden.rs` bu dizindeki her modül klasörünü yükler. Kaynak:
ileri modüller tasarım belgesi §13 (dört katmanlı test stratejisinin ilk katmanı).

- Girdi sentetik konumlardır: dokuz grahanın sidereal boylamı ve (varsa) lagnanın sidereal
  boylamı. Ephemeris gerekmez; rasi çerçevesi sidereal sayılır.
- Beklenen değer **klasik kuraldan elle türetilir** ve `derivation` alanında adım adım yazılır.
  Başka bir yazılımın çıktısı buraya girmez; `source` yalnızca `hand` olabilir.
- Bir örnek motorla uyuşmazsa önce kural metni yeniden okunur. Beklenen değer motor çıktısına
  bakarak düzeltilmez.
- `derivation`, örneği yazan dışında bir gözden geçirmede kuraldan yeniden hesaplanır.
- Bilinmeyen alan reddedilir. Dosya adı `id` ile aynıdır. Modül başına en az örnek sayısı test
  dosyasında sabittir (`MIN_*`); örnek silmek testi düşürür.

## Şema

```json
{
  "id": "dosya-adiyla-ayni",
  "source": "hand",
  "rule_ref": "genel gönderme, ör. BPHS, bakışlar bölümü (alıntı yapılmaz)",
  "derivation": ["adım 1", "adım 2"],
  "input": {
    "lagna_lon": 15.0,
    "grahas": { "Sun": 0.0, "Moon": 0.0, "Mars": 0.0, "Mercury": 0.0, "Jupiter": 0.0,
                "Venus": 0.0, "Saturn": 0.0, "Rahu": 0.0, "Ketu": 0.0 }
  },
  "expected": { "...": "modüle göre" }
}
```

`lagna_lon: null` doğum saati bilinmeyen haritadır: lagnaya bağlı her beklenen değer `null` olur.

Shadbala örnekleri girdiye iki alan daha ekler: `astro` (saf hesabın astronomik girdisi:
doğum anı, gün doğumu, batımı ve sonraki gün doğumu gün kesri olarak; MC'nin sidereal boylamı;
yedi grahanın tropikal boylamı; ortalama boylamlar; doğum gününün ve iki güneş girişinin
varası) ve isteğe bağlı `lats` (ekliptik enlemler; yazılmayan graha 0°'dir). Diğer modüllerde
bu alanlar yazılmaz.

Ayara bağlı modüller (Jaimini, chara dasha) girdiye `advanced` bloğunu ekler:
`"advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}`. Diğer modüllerde bu alan
yazılmaz. Chara dasha örneklerinde doğum anı ve yıl uzunluğu girdide yoktur: test sabit bir
doğum anı ve 365,25 günlük yıl kullanır, tarihleri el yıllarının birikimli toplamından kurar.

## Modüller

| Klasör | `expected` | En az |
|---|---|---|
| `foundation` | `waxing`, `nature` (dokuz graha), `kuja_dosha`, `house_lords` (12), `houses`, `houses_owned` | 6 |
| `drishti` | `sign_aspects` (on üç tam bakışın tamamı, tablo sırasıyla), `sphuta` (en az on değer: `from`, `to`, `angle`, `value`) | 4 |
| `jaimini` | `karakas` (rol sırasında: `role`, `graha`, `effective_deg`, `tie`), `karakamsha` (`atmakaraka`, `sign`, dokuz grahanın `houses_in_d1` ve `houses_in_d9` değeri), `co_lords` (Akrep ve Kova: `lord`, belirleyici ölçüt `step`), `arudhas` (on iki pada: `house`, `sign`, `lord`, `exception`; saat bilinmiyorsa `null`), `argala` (en az iki referans burç: dört çiftin iki graha listesi ve durumu, `third_house_malefic_argala`) | 8 |
| `chara_dasha` | `forward`, `order` (dasha sırası, lagnadan), `years` (ilk çevrim, `order` sırasında), `second_cycle` (ikinci çevrimde üretilen dashalar: `sign`, `years`), `sub_periods` (ilk iki dashanın alt dönem sırası); saat bilinmiyorsa `expected` `null` | 7 |
| `shadbala` | `grahas` (yedi graha: sthana balanın beş kalemi ve `sthana`, `dig`, zaman gücünün sekiz kalemi ve `temporal`, `cheshta`, `naisargika`, `drik`, `total`, `rank`, `war`), `time_lords`, `bhavas` (on iki bhava: `house`, `lord`, `lord_bala`, `dig`, `drishti`, `total`, `rank`); saat bilinmiyorsa `expected` `null` | 4 |
| `yogas` | `hits` (raporun bütün kayıtları, katalog sırasıyla: `id`, `grahas`, `houses`, `relations`, `conditions`), `absent` (bu örneğin "oluşmayan durum" olarak gösterdiği kurallar), `cancelled` (iptal örnekleri), `evaluated`, `skipped_time_unknown` | 15 |
| `kuta` | `scores` (sekiz kuta, tablo sırasıyla: `kuta`, `score`, `groom_class`, `bride_class`), `total`, `bhakoot_exception`, `nadi_exception`, `kuja_dosha` (`[damat rolü, gelin rolü]`), `overlay` (`groom` ve `bride`: dokuzar satır `[diğer tarafın lagnasından ev ya da null, Ay'ından ev]`), `approximate` | 8 |
| `muhurta` | `eighth_min`, `rahu`, `yamaganda`, `gulika` (`segment`, `from_min`, `to_min`), `abhijit` (`from_min`, `to_min`), `abhijit_weekday_exception`, `hora_min`, `hora_lords` (24), `tithi_group`, `nakshatra_nature`, `tara` (`number`, `favourable`), `chandra` (`house`, `favourable`) | 7 |

Shadbala örnekleri çalışma kâğıdıdır (tasarım belgesi §7 "Test"): `shadbala/WORKSHEET.md` her
kalemin aritmetiğini adım adım yazar, JSON dosyaları aynı girdiyi ve sonuçları taşır. Tolerans
0,01 virupadır. Kâğıdı, motoru yazan dışında bir gözden geçiren kuraldan yeniden hesaplar ve
kâğıdın başındaki tabloya adını yazar.

Yoga örnekleri (tasarım belgesi §8 "Test") raporun tamamını yazar: her örnekte katalogdaki 41
kuralın hepsi `derivation`'da sırayla değerlendirilir. `absent`, o örneğin "oluşmayan durum"
olarak gösterdiği kuralları sayar; `cancelled`, temel koşulu sağlandığı hâlde iptal kuralıyla
oluşmayan Kemadruma ve Shakata'yı, bir de düşüş burcunda olup hiçbir koşulu sağlanmayan grahayı
(Neecha bhanga kaydı yok) gösterir. Ayrı bir test her kuralın en az bir örnekte oluştuğunu, en
az bir örnekte `absent` sayıldığını ve üç iptal durumunun bulunduğunu sabitler. `lagna_lon: null`
örneklerinde lagna isteyen 17 kural değerlendirilmez ve `houses` boştur. Tasarım belgesinin açık
bıraktığı yerlerdeki okuma kuralları ve bağımsız gözden geçirmenin kaydı `yogas/REVIEW.md`'dedir.

Uyum örnekleri (tasarım belgesi §9 "Test") iki harita taşır: üst düzeydeki `lagna_lon` ve
`grahas` damat rolünün haritasıdır, girdideki `bride` bloğu (aynı iki alan) gelin rolünün
haritasıdır; diğer modüllerde `bride` yazılmaz. Her örnek raporun tamamını yazar ve toplamı
`derivation`'da adım adım kurar. Ayrı bir test her kutanın en çok ve en düşük puanının, iki
istisnanın (var ve yok), vashyanın bölünen iki burcunun dört yarısının ve doğum saati bilinmeyen
haritanın iki rolde de örneklerde bulunduğunu sabitler. Okuma kuralları, motorun üç puan tablosu,
tabloların basılı kaynaklarla karşılaştırma kaydı ve bağımsız gözden geçirmenin kaydı
`kuta/REVIEW.md`'dedir.

Muhurta örnekleri (tasarım belgesi §10 "Test") konum ve graha taşımaz: girdileri sentetik bir
gündür. `input.day` gün doğumunu, gün batımını ve sonraki gün doğumunu dakika olarak (aynı gece
yarısından) ve varayı; `input.tithi` günün tithisini (1–30); `input.moon` günün Ay'ını;
`input.natal` doğum Ay'ını (ikisi de `nakshatra` ve `sign`) verir. Diğer modüllerin `lagna_lon` ve
`grahas` alanları bu örneklerde yazılmaz. Beklenen saatler de dakikadır. Ayrı bir test örneklerin
yedi varanın her birini, beş tithi grubunu, yedi nakshatra doğasını, elverişli ve elverişsiz tara
ile chandra balayı, Çarşamba istisnasını ve 59, 60 ve 61 dakikalık horaları kapsadığını, ve
tasarım belgesinin örneğinin (06:00–18:00 Pazartesi) aralarında bulunduğunu sabitler. Okuma
kuralları, motorun tabloları, sahibin onayını bekleyen varyant noktaları ve bağımsız gözden
geçirmenin kaydı `muhurta/REVIEW.md`'dedir.

Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
