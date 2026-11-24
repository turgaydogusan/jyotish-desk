# Golden harita fixture'ları

`crates/jyotish-core/tests/golden.rs` bu dizindeki her `*.json` dosyasını yükler,
`compute_chart` ile hesaplar ve karşılaştırır. Her fixture geçmek zorundadır; atlama yoktur.

## Toleranslar (spec §8)

- Graha boylamı ≤ 2″
- Ayanamsa ≤ 1″
- Lagna ≤ 5″
- İlk mahadasha başlangıcı ≤ 1 gün; D9 burçları birebir

Bir fixture tolerans dışına çıkarsa tolerans gevşetilmez. Önce ayar farkı aranır:
ayanamsa tanımı, nütasyon, mean/true düğüm, UT/ET.

## Şema

```json
{
  "id": "dosya-adiyla-ayni",
  "source": "swetest | jhora",
  "source_version": "2.10.03",
  "notes": "yerel saat, saat dilimi, neden seçildiği",
  "settings": { "...": "CalcSettings, tam döküm (schema_version 1)" },
  "input": { "jd_ut": 2451545.0, "lat": 28.6139, "lon": 77.2090 },
  "expected": {
    "ayanamsa_deg": 23.8532225,
    "lagna_lon": 76.3380905,
    "grahas": { "Sun": 0.0, "Moon": 0.0, "Mars": 0.0, "Mercury": 0.0, "Jupiter": 0.0,
                "Venus": 0.0, "Saturn": 0.0, "Rahu": 0.0, "Ketu": 0.0 },
    "d9_signs": { "Lagna": "Aquarius", "Sun": "Leo", "...": "dokuz grahanın hepsi" },
    "dasha": { "first_lord": "Rahu", "first_start_jd": 2445231.575532 }
  }
}
```

- Bilinmeyen alanlar reddedilir. Dokuz grahanın hepsi zorunludur.
- `lat` kuzey +, `lon` doğu +, ondalık derece.
- `jd_ut`, UT Jülyen günüdür. Yerel saatten `atlas::resolve_moment` ile hesaplanır; yerel saat ve saat dilimi `notes`'a yazılır.
- Boylamlar ondalık derecedir ve `settings.zodiac`'ın rasi çerçevesindedir: `Sidereal` ise sidereal, `TropicalRasiSiderealNakshatra` ise tropikal.
- `d9_signs` ve `dasha` isteğe bağlıdır. `d9_signs` varsa `Lagna` ve dokuz graha zorunludur; burçlar birebir karşılaştırılır.
- `dasha.first_start_jd`, ilk mahadashanın **teorik** başlangıcıdır (UT JD, doğumdan önce): `jd − geçen_kesir × yıl(lord) × dasha_year`. Tolerans ≤ 1 gün.
- `swetest` fixture'larında bu iki alan `scripts/gen-golden-swetest.sh` içinde awk ile türetilir (Rust motorundan bağımsız aritmetik, aynı kural). Bağımsız referans değildir; kuralın kendisini JHora seti doğrular.
- JHora'dan eklerken: D9 burçlarını Navamsa haritasından, ilk mahadasha lordunu ve başlangıç tarihini Vimshottari listesinden oku. JHora'nın dasha yılı ayarını fixture'ın `settings.dasha_year`'ıyla eşleştir; başlangıç tarihini UT JD'ye çevirip `notes`'a yerel tarihi yaz.
- En az 5 fixture `d9_signs` ve `dasha` taşımalıdır (`MIN_DERIVED_FIXTURES`).

## Kaynaklar

- `swetest`: `./scripts/gen-golden-swetest.sh` ile üretilir (vendored SE 2.10.03). Yalnızca tesisatı kanıtlar, çünkü motor aynı kütüphanedir.
- `jhora`: Jagannatha Hora çıktısı. Bağımsız referanstır. 25 haritalık set kullanıcı tarafından eklenir.

## JHora haritası ekleme

1. JHora'da hesap ayarlarını fixture'ın `settings`'iyle eşleştir:
   - Ayanamsa: Lahiri / Chitrapaksha
   - Düğüm: mean veya true
   - Zodyak: sidereal veya tropikal
2. Haritayı gir. Yerel saati, saat dilimini ve yeri `notes`'a yaz.
3. JHora'nın gösterdiği değerleri saniye hassasiyetinde oku: ayanamsa, lagna ve dokuz graha.
4. Değerleri ondalık dereceye çevir: `burç_index × 30 + derece + dakika/60 + saniye/3600`. Koç = 0.
5. `jd_ut`'yi aynı doğum verisinden `atlas::resolve_moment` ile hesapla. JHora'nın gösterdiği JD varsa onu kullan ve farkı `notes`'a yaz.
6. Fixture'ı doldur: `"source": "jhora"`, `"source_version"` JHora sürümüdür (ör. "8.0"). Dosya adı `id` ile aynı olmalıdır.
7. Çalıştır: `cargo test -p jyotish-core --test golden`.

## 25 haritalık JHora setinin kapsamı (spec §8)

- [ ] 1850–2050 arası tarihler (en az bir 19. yüzyıl, bir 2030 sonrası)
- [ ] İki yarıküre (güney enlemi, batı boylamı)
- [ ] Yüksek enlem (≥ 60°)
- [ ] Yerel gece yarısı doğumu
- [ ] Burç sınırında graha ve lagna (< 1′)
- [ ] Nakshatra sınırında Ay (< 1′)
- [ ] Türkiye yaz saati vakaları (1940–2016 geçişleri ve kalıcı +03)
