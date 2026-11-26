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

Ayara bağlı modüller (Jaimini) girdiye `advanced` bloğunu ekler:
`"advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}`. Diğer modüllerde bu alan
yazılmaz.

## Modüller

| Klasör | `expected` | En az |
|---|---|---|
| `foundation` | `waxing`, `nature` (dokuz graha), `kuja_dosha`, `house_lords` (12), `houses`, `houses_owned` | 6 |
| `drishti` | `sign_aspects` (on üç tam bakışın tamamı, tablo sırasıyla), `sphuta` (en az on değer: `from`, `to`, `angle`, `value`) | 4 |
| `jaimini` | `karakas` (rol sırasında: `role`, `graha`, `effective_deg`, `tie`), `karakamsha` (`atmakaraka`, `sign`, dokuz grahanın `houses_in_d1` ve `houses_in_d9` değeri) | 8 |

Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
