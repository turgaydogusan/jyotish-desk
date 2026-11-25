# Golden harita fixture'ları

`crates/jyotish-core/tests/golden.rs` bu dizindeki her `*.json` dosyasını yükler,
`compute_chart` ile hesaplar ve karşılaştırır. Her fixture geçmek zorundadır; atlama yoktur.

## Toleranslar (spec §8)

- Graha boylamı ≤ 2″
- Ayanamsa ≤ 1″
- Lagna ≤ 5″
- İlk mahadasha başlangıcı ≤ 1 gün; D9 burçları birebir
- Panchang: tithi, nakshatra, yoga, karana numaraları ve vara birebir; bitiş zamanları, gün doğumu ve batımı ≤ 60 sn. SAV birebir.

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
    "dasha": { "first_lord": "Rahu", "first_start_jd": 2445231.575532 },
    "panchang": { "sunrise_jd": 2451544.574985, "sunset_jd": 2451545.000560, "vara": "Saturday",
                  "tithi": 26, "tithi_ends_jd": 2451545.826235, "nakshatra": 15,
                  "nakshatra_ends_jd": 2451545.044056, "yoga": 8, "yoga_ends_jd": 2451545.822425,
                  "karana": 51, "karana_ends_jd": 2451545.277284 },
    "sav": [29, 21, 23, 26, 34, 33, 30, 30, 28, 21, 32, 30]
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
- `panchang` ve `sav` isteğe bağlıdır. `panchang` doğum anının değerleridir: tithi 1–30, nakshatra 1–27, nitya yoga 1–27, karana 1–60 (yarım tithi sırası), vara İngilizce gün adı. `*_ends_jd` o andan sonraki ilk sınırdır (UT JD). `sunrise_jd` doğum yerinde doğumdan önceki son gün doğumudur (`settings.sunrise_def`; varsayılan disk merkezi, kırılmasız), `sunset_jd` ondan sonraki ilk batıştır; kutup gecesi/gündüzünde ikisi de `null` ve vara 06:00 yerel ortalama saatten bulunur. `sav` Koç'tan Balık'a 12 sayıdır, toplamı 337.
- `swetest` fixture'larında bu iki alan betikte türetilir: gün doğumu `swetest -rise -hindu`'dan, bitişler `swetest -sid1` boylamları üzerinde ikiye bölmeyle, SAV ayrı yazılmış bir BPHS tablosuyla (awk). JHora'dan eklerken: "Panchanga" görünümünden tithi, nakshatra, yoga, karana ve bitiş saatlerini; gün doğumunu ve batımını; Ashtakavarga görünümünden SAV'ı oku. JHora'nın gün doğumu ayarını fixture'ın `sunrise_def`'iyle eşleştir. `swetest -rise -hindu` SE'nin hızlı yöntemini kullanır ve 55° enlemin üstünde 60 sn'den fazla sapabilir; bu enlemlerde gün doğumu beklentisi motorun tam aramasıyla ya da bağımsız bir ufuk geçişi hesabıyla doğrulanmadan fixture'a yazılmaz.
- En az 5 fixture `panchang` ve `sav` taşımalıdır (`MIN_PANCHANG_FIXTURES`).

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
- [ ] Gün doğumundan önce doğum (vara önceki gün) ve kutup dairesinde kutup gecesi
