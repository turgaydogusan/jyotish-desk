# jyotish-desk v1 — Tasarım Belgesi

Tarih: 2026-11-24 · Durum: Onay bekliyor · Kod adı: `jyotish-desk` (geçici)

## 1. Amaç

Profesyonel Jyotish (Vedik astroloji) yazılımı. İşlevsel hedef: piyasadaki profesyonel Jyotish yazılımları seviyesi. Başka ürünlerin adı, arayüz düzeni ve yorum metinleri kopyalanmaz. Klasik hesap yöntemleri (Parashara, Jaimini, dasha) özgün olarak uygulanır (clean-room).

- **Kitle:** Profesyonel astrologlar. Yoğun tablolar, danışan veritabanı, raporlar, klavye ağırlıklı masaüstü kullanımı.
- **Lisans:** AGPL-3.0-or-later. Ephemeris: Swiss Ephemeris (AGPL). Katkı DCO ile, CLA yok.
- **Platform yol haritası:** v1 macOS. Sonra Windows, ardından Android tablet. iPad, AGPL ile App Store arasındaki uyumsuzluk nedeniyle kapsam dışı.
- **Dağıtım (v1):** Notarize edilmiş DMG, App Store dışı.
- **Arayüz dilleri:** Türkçe, İngilizce, Fransızca (i18n ilk günden).

## 2. Alt projeler

| # | Alt proje | v1 |
|---|---|---|
| 1 | `jyotish-core` Rust hesap kütüphanesi | Evet |
| 2 | macOS Tauri uygulaması | Evet |
| 3 | İleri modüller: Shadbala, yogalar, Jaimini, uyum, Muhurta, Prashna, arama | Hayır |
| 4 | Windows sürümü | Hayır |
| 5 | Android tablet sürümü | Hayır |
| 6 | Özgün yorum metinleri (TR/EN/FR) | Hayır |

Bu belge yalnızca 1 ve 2 numaralı alt projeleri kapsar.

## 3. v1 kapsamı

**Kapsama girenler:**
- Doğum verisi girişi. Çevrimdışı atlas (GeoNames `cities1000`), geçmişe dönük saat dilimleri (IANA tzdb), LMT, manuel koordinat ve manuel UTC ofseti.
- Zodyak: Sidereal (varsayılan) ve tropikal rasi + sidereal nakshatra (karma yaklaşım).
- Ayanamsa: Lahiri (varsayılan), Raman, KP, True Chitrapaksha ve özel değer.
- Rasi ve 16 varga (Shodasavarga, Parashara tanımları). Çizim stili Kuzey, Güney veya Doğu Hint.
- Graha tablosu: boylam, enlem, hız, retro, burç, burç içi derece, nakshatra, pada, yanma (combust), dignity ve ev.
- Vimshottari dasha, 5 seviyeye kadar (4. ve 5. seviyeler tembel yüklenir).
- Panchang: tithi, vara, nakshatra, yoga, karana, gün doğumu ve gün batımı.
- Ashtakavarga: her gezegen için ayrı tablolar (BAV) ve toplam tablo (SAV).
- Transit (gochara) bindirme.
- Danışan veritabanı: arama, etiketler, notlar, JSON/CSV dışa aktarım, JSON ve Jagannatha Hora `.jhd` içe aktarım.
- Print ve PDF.
- Otomatik güncelleme (kullanıcı onaylı).

**Kapsam dışı:** Alt proje 3, Yogini ve Chara dasha, yorum metinleri, bulut senkronizasyonu.

## 4. Mimari

Tek bir Cargo workspace ve monorepo.

```
crates/
  ephemeris/     Swiss Ephemeris C kaynağı vendored, `cc` ile derlenir.
                 Hazır binding'ler repoda (build'de libclang yok).
                 Tek `SwissEph` örneği Mutex arkasında (SE global durumu thread-safe değil).
                 `Ephemeris` trait'i; veri yolu parametre (Android için).
  jyotish-core/  Saf, yan etkisiz hesap: time, ayanamsa, grahas, houses,
                 vargas, dasha, panchang, ashtakavarga, transit.
  atlas/         GeoNames SQLite + FTS5; yerel saat → ResolvedMoment (jiff (bundled tzdb), LMT).
  store/         rusqlite + rusqlite_migration; danışan ve harita kayıtları.
app/
  src-tauri/     İnce komut katmanı, iş mantığı yok. TS tipleri tauri-specta ile üretilir.
  ui/            React + TypeScript + Vite. SVG harita çizimi, tablolar,
                 dasha ağacı, i18next.
data/            Atlas ve ephemeris build betikleri.
tests/golden/    Referans harita fixture'ları.
docs/
LICENSE, NOTICE
```

**Veri akışı:**
1. `resolve_moment(birth)` → `ResolvedMoment`. Saat dilimi belirsizliği hesaptan önce çözülür.
2. `compute_chart(moment, settings)` → `Chart`. Dasha 3 seviye gelir.
3. `dasha_children(path)` → alt dönemler, isteğe bağlı.
4. Arayüz yalnızca çizer, hesap yapmaz.

**Kalıcılık:** Veritabanında doğum verisi, çözülmüş UTC zamanı, hesap ayarları, görünüm durumu ve notlar saklanır; hesaplanmış harita saklanmaz. Harita her açılışta yeniden hesaplanır (deterministik, milisaniyeler sürer). Böylece eskimiş kayıt oluşmaz.

**Danışan açma:**
- Kayıtlı bir danışan seçilince tek IPC çağrısı yapılır: `open_client(client_id)`. Dönüş: `{client, birth, calc_settings, view_state, chart, recent_notes}`.
- Seçimden çizilmiş haritaya ≤ 200 ms. Rust tarafında ≤ 50 ms. Yükleme göstergesi yok; form ya da onay penceresi çıkmaz.
- `Cmd+K` danışan arama paleti: isim, etiket ve not araması. `Enter` haritayı açar, `Cmd+Enter` danışanı açar ve kenar çubuğunu gösterir (sekme yoktur; UI spec §4).
- Tek harita açıktır (UI spec §4 "Sekme yoktur"); her harita kendi `ViewState`'ini saklar. Uygulama kapanıp açılınca son harita ve düzen geri yüklenir.
- Notlar paneli haritanın yanında sürekli açılabilir (`Cmd+Shift+N`). Notlar tarih damgalıdır ve otomatik kaydedilir.
- "Kaydet" düğmesi yok. Görünüm ve ayar değişiklikleri yaklaşık 500 ms debounce ile kaydedilir. Doğum verisi yalnızca açık bir "Düzenle" kipinde değiştirilir.

**Ayar katmanları:**
- `CalcSettings` hesap sonucunu değiştirir: zodyak, ayanamsa, düğüm, ev sistemi, dasha yılı, yanma orbları, gün doğumu tanımı.
- `ViewState` yalnızca arayüzü etkiler. İkisi ayrı tutulur.
- Harita oluşturulurken genel varsayılanların tam kopyası haritaya yazılır, miras alınmaz. Genel varsayılan sonradan değişse de mevcut haritalar sessizce değişmez.
- Ayarlar harita başına tutulur; danışan düzeyinde yalnızca `last_chart_id` saklanır.
- Haritanın hesap ayarı genel varsayılandan farklıysa başlıkta bir fark rozeti görünür (ör. "Raman · True Rahu"). Rozette "varsayılana döndür" ve "bunu varsayılan yap" seçenekleri var.
- Transit bindirmenin açık/kapalı durumu saklanır, tarihi saklanmaz. Her açılışta tarih "şimdi" olur.

**PDF:** Print'e özel HTML düzeni ve macOS'un yerleşik "PDF olarak kaydet" özelliği. M0'da spike yapılır. WKWebView `@page` desteği yetersiz çıkarsa yedek plan Rust tarafında Typst.

## 5. Veri modeli

### Rust tipleri

- **`BirthData`**: `name`, `gender?`, `date`, `time: Option`, `time_certainty {Exact, Approx, Unknown}`, `place {name, country, admin1, lat, lon, geoname_id?}`, `tz {Iana(id) | Fixed(secs) | Lmt}`, `fold: Option<Earlier|Later>`, `notes`
- **`ResolvedMoment`**: `utc`, `jd_ut`, `offset_secs`, `abbrev`, `warnings[]`
- **`CalcSettings`** (sürümlü): `schema_version`, `zodiac {Sidereal, TropicalRasiSiderealNakshatra}`, `ayanamsa {Lahiri, Raman, KP, TrueChitra, Custom{t0_jd, deg}}`, `node {Mean, True}`, `house_system`, `dasha_year`, `combustion_orbs`, `sunrise_def`
- **`ViewState`** (sürümlü, toleranslı ayrıştırma): `schema_version`, `panel_layout` (düzenler, paneller ve panel başına varga/stil/transit ayarları; bkz. `2026-11-24-ui-design.md` §3.2), `dasha_expanded_path[]`
- **`AppPrefs`** (genel): `locale`, varsayılan `CalcSettings`, varsayılan `ViewState`
- **`GrahaPosition`**: `graha`, `lon`, `lat`, `speed`, `retrograde`, `sign`, `deg_in_sign`, `nakshatra`, `pada`, `combust`, `dignity`, `house`
- **`VargaChart`**: `varga (D1..D60)`, `lagna_sign`, `placements[(graha, sign, deg)]`
- **`DashaPeriod`**: `lords[]` (yol), `level`, `start_utc`, `end_utc`, `has_children`
- **`Panchang`**: `tithi {no, paksha, ends_at}`, `vara`, `nakshatra {no, pada, ends_at}`, `yoga`, `karana`, `sunrise`, `sunset`
- **`Ashtakavarga`**: `bav[7][12]`, `sav[12]`
- **`Chart`**: `meta {engine_version, swe_version, settings, warnings}`, `moment`, `ayanamsa_deg`, `lagna`, `grahas[9]`, `bhavas`, `vargas`, `dasha`, `panchang`, `ashtakavarga`

### SQLite şeması

```
clients(id, uuid, name, gender, last_chart_id, last_opened_at,
        created_at, updated_at)
charts(id, uuid, client_id, label, kind, date, time, time_certainty,
       place_name, country, lat, lon, tz_id, offset_override, fold,
       resolved_utc, resolved_offset_secs, tzdb_version,
       calc_settings_json NOT NULL, view_state_json NOT NULL,
       last_opened_at, created_at, updated_at)
client_notes(id, uuid, client_id, chart_id?, body, created_at, updated_at)
tags(id, name UNIQUE)
chart_tags(chart_id, tag_id)
app_state(key PRIMARY KEY, value_json)   -- son açık danışanlar, pencere düzeni
FTS5: clients.name, client_notes.body
```

- UUID alanları ileride senkronizasyon ve içe aktarma sırasında kayıt birleştirme içindir.
- Etiketler harita düzeyindedir, böylece araştırma koleksiyonları kurulabilir.
- Notlar tarihli seans kayıtları olarak ayrı tabloda tutulur; isteğe bağlı olarak bir haritaya bağlanabilir.
- Çözülmüş zaman (`resolved_*`, `tzdb_version`) saklanır. Açılışta atlas sorgusu gerekmez. tzdb güncellemesi bir haritanın zamanını değiştirecekse uyarı gösterilir, karar kullanıcıya bırakılır.

### İçe ve dışa aktarma

- Sürümlü JSON (`schema_version`): içe ve dışa aktarma. `calc_settings` ve `view_state` içerir. Notlar yalnızca "notları dahil et" kutusu işaretlenirse eklenir; kutu varsayılan olarak kapalıdır, çünkü notlar hassas veridir.
- CSV: yalnızca dışa aktarma.
- JHD (Jagannatha Hora): yalnızca içe aktarma. İçe alınan haritalar o anki genel varsayılanların kopyasını alır. İşaret konvansiyonları gerçek dosyalarla doğrulanacak.
- Sınırlar: içe aktarılan JSON en çok 32 MiB (okunmadan önce denetlenir); not gövdesi en çok 64 KiB (65 536 bayt UTF-8, arayüzde 16 384 karakter). Dışa aktarma sınırlanmaz; dosya içe aktarma sınırını aşarsa kullanıcı uyarılır. Biçimin ayrıntısı: `docs/export-format.md`.

## 6. Varsayılanlar

- **Düğüm:** Mean Rahu. True seçenek olarak var.
- **Ev sistemi:** Whole Sign. Bhava chalit için Sripati, KP için Placidus seçenek olarak var.
- **Dasha yılı:** 365.25 gün. Seçenekler: 360 ve 365.2422.
- **Yanma orbları (BPHS):** Ay 12°, Mars 17°, Merkür 14° (retroda 12°), Jüpiter 11°, Venüs 10° (retroda 8°), Satürn 15°. Düğümler yanmaz.
- **Dignity:** BPHS. Yücelme ve düşüş dereceleri, moolatrikona, kendi burcu ve panchadha maitri. Düğümlere dignity atanmaz.
- **Konumlar:** Jeosantrik, görünür konum. Vara gün doğumunda değişir.
- **Gün doğumu:** Disk merkezi, kırılmasız (`SE_BIT_HINDU_RISING`). Ayarlanabilir.

## 7. Hata yönetimi

- **DST boşluğu (var olmayan saat):** Kayıt engellenir, tek tıkla düzeltme önerilir. Saat sessizce kaydırılmaz.
- **DST örtüşmesi (iki kez yaşanan saat):** Kullanıcı Earlier ya da Later seçmek zorundadır. Seçim `fold` alanında saklanır.
- **Standart saat öncesi tarihler:** tzdb ofsetinin kısaltması "LMT" ise (yaz saati değil), LMT referans şehirden değil doğum yerinin boylamından hesaplanır. Adlandırılmış ortalama saatler (IMT, MMT…) ve diğer tüm ofsetler tzdb'deki gibi kullanılır; ofset dakikaya tam bölünmüyorsa veya yıl 1970'ten önceyse güvenilirlik uyarısı verilir.
- **Manuel UTC ofseti:** Her zaman girilebilir. tzdb'nin 1970 öncesi verisi garanti değil.
- **Bilinmeyen doğum saati:** Yerel 12:00 kullanılır. Lagna, evler ve lagnaya bağlı vargalar gizlenir. Ay, nakshatra ve dasha "yaklaşık" olarak işaretlenir. Ay o gün içinde nakshatra değiştiriyorsa uyarı verilir.
- **Paketlenen aralık dışındaki tarih:** Moshier yedeğine geçilir ve görünür bir "düşük hassasiyet" uyarısı gösterilir.
- **Eksik ephemeris dosyası:** Açılıştaki self-check ölümcül hata verir. Sessiz yedeğe düşme yok.
- **Bozuk ya da eski `view_state_json`:** Açılışı asla engellemez. Bilinmeyen anahtarlar yok sayılır, geçersiz değerde varsayılan görünüm kullanılır.
- **Eski `calc_settings_json`:** Sürüm göçü uygulanır. Göç başarısız olursa genel varsayılan kullanılır ve görünür bir uyarı verilir.
- **Silinmiş haritayı gösteren `last_chart_id`:** Danışanın en son güncellenen haritası açılır.
- **Arayüze yansıma:** `thiserror` enum'u `{code, params}` olarak gelir, çeviri i18n katmanında yapılır.
  - Alan hataları formda satır içinde gösterilir.
  - Hesap hataları modal olmayan bir uyarı bandında gösterilir.
  - Ölümcül olmayan durumlar `Chart.meta.warnings` içinde taşınır.
  - FFI sınırında panik olmaz.

## 8. Doğruluk ve test

- **Ephemeris sarmalayıcı:** `swetest` çıktısıyla karşılaştırılır, tolerans ≤ 0.001″.
- **Golden set:** 25 harita. Kapsamı:
  - 1850–2050 arası tarihler, iki yarıküre, yüksek enlem
  - Gece yarısı, burç ve nakshatra sınırları
  - Türkiye'nin yaz saati vakaları

  Referans olarak JHora çıktısı, sürüm ve ayar dökümüyle birlikte JSON fixture olarak saklanır. İkinci kaynak `swetest`.
- **Toleranslar:**
  - Graha boylamı ≤ 2″, ayanamsa ≤ 1″, lagna ≤ 5″
  - Panchang bitiş zamanları ve gün doğumu ≤ 60 sn
  - Mahadasha başlangıcı ≤ 1 gün
  - Varga burçları ve BAV/SAV değerleri birebir aynı
- **Property testleri (proptest):**
  - Boylam [0, 360) aralığında, nakshatra ve pada tutarlı
  - Rahu ile Ketu arası 180°
  - Alt dashalar boşluksuz ardışık ve toplamları ebeveyne eşit; tüm dönemler toplam 120 yıl
  - SAV toplamı 337; BAV toplamları 48/49/39/54/56/52/39
  - Yerel saat → UTC → yerel saat dönüşümü tutarlı
- **Saat dilimi:** Türkiye'nin 1940–2016 geçişleri ve kalıcı +03 için tablo testleri.
- **Danışan açma:**
  - Gidiş-dönüş: Kaydet → uygulamayı kapat → danışanı aç. Ayarlar, sekmeler, vargalar ve notlar birebir aynı gelmeli.
  - Performans: `open_client` Rust tarafında ≤ 50 ms. CI'da regresyon eşiği olarak ölçülür.
  - Property: Rastgele ya da bozuk `view_state_json` ile `open_client` asla hata dönmez.
  - Playwright: `Cmd+K` → isim → `Enter` akışında doğum formunun hiç görünmediği doğrulanır.
- **Arayüz:**
  - Vitest + Testing Library, SVG snapshot testleri
  - Playwright (Tauri IPC mock'lanır)
  - Komut katmanı Rust entegrasyon testleriyle kapsanır, çünkü macOS'ta `tauri-driver` yok
- **CI (GitHub Actions):**
  - `macos-14`: fmt, clippy `-D warnings`, test, `cargo deny`, vitest, playwright, build
  - `ubuntu` ve `windows`: yalnızca çekirdek crate'ler (taşınabilirlik kontrolü)
  - Tag atılınca imzalı ve notarize edilmiş DMG üretilir

## 9. Paket verisi

- **Ephemeris dosyaları:** `sepl_12`, `semo_12`, `sepl_18`, `semo_18` (.se1). MS 1200–2399 aralığı, yaklaşık 3,6 MB (doğrulanacak).
- **`sefstars.txt`:** True Chitrapaksha ayanamsası için gerekli (Spica).
- **Asteroit dosyaları yok.**
- **Atlas:** GeoNames `cities1000`, kırpılmış SQLite + FTS5, yaklaşık 15–25 MB. CC-BY kaynak gösterimi About ekranında.

## 10. Paketleme

- **İmza:** Developer ID Application, hardened runtime, `notarytool` + staple. Universal binary (arm64 + x86_64).
- **Minimum macOS:** 13 (Ventura).
- **Güncelleme:** Tauri updater, imzalı, GitHub Releases üzerinden. Kontrol kullanıcı onayıyla yapılır.
- **İsim kuralları:**
  - Kala, Kāla, Kaala, Cala ve türevleri kullanılmaz.
  - Ernst Wilhelm'e ya da JHora, Parashara's Light, Shri Jyoti Star gibi yazılımlara benzer isim kullanılmaz.
  - Çıkıştan önce marka taraması yapılır.
- **AGPL:** About ekranında kaynak kod bağlantısı.

## 11. Kilometre taşları

- **M0:** Repo ve CI. Üç spike: SE universal derleme, WKWebView print → PDF, tauri-specta.
- **M1:** `ephemeris` crate'i. `atlas`: GeoNames pipeline'ı, tz çözümü, LMT, DST hataları.
- **M2:** Çekirdek temel: ayanamsa, grahalar, lagna, evler, nakshatra, dignity, yanma. Golden test altyapısı.
- **M3:** 16 varga, Vimshottari, Panchang, Ashtakavarga, transit.
- **M4:** `store`: şema, migration, FTS, `open_client` komutu, ayar kopyası ve sürüm göçü, `client_notes`, JSON/CSV, JHD içe aktarma.
- **M5:** Arayüz iskeleti: doğum girişi, üç stilde SVG harita, graha tablosu, dasha ağacı, i18n, klavye kısayolları. `ViewState` baştan tek merkezden yönetilir ve serileştirilebilir.
- **M6:** Danışan veritabanı arayüzü, `Cmd+K` paleti, notlar paneli, fark rozeti, oturum geri yükleme, etiketler, transit bindirme, ayarlar.
- **M7:** Print ve PDF düzenleri.
- **M8:** İmza, notarizasyon, updater. 3–5 astrologla kapalı beta.

## 12. Riskler

- **CLA olmadığı için lisans sonradan değiştirilemez.** iPad kapısı kalıcı olarak kapanır. Bu bilinçli bir kabul; fikir değişirse CLA'ya şimdi geçilmeli.
- **"Tam eşdeğer" iddiası v1'i aşıyor.** v1 sonrası için profesyonel Jyotish özellik envanteri çıkarılacak (clean-room, yalnızca klasik kaynaklardan).
- **JHora kapalı kaynak.** Ayar uyumsuzluğu sahte farklar üretebilir. Bu yüzden her fixture'a ayar dökümü eklenir ve `swetest` ikinci kaynak olarak kullanılır.
- **tzdb bayatlayabilir ve 1970 öncesinde hatalı olabilir.** Updater ve manuel ofset bu riski hafifletir.
- **GeoNames küçük köyleri kapsamıyor.** Manuel koordinat girişi birinci sınıf bir özellik olmalı.
- **WKWebView ile WebView2 farklı davranıyor.** Windows portunda print ve SVG farkları çıkabilir; print düzenleri soyut tutulmalı.
- **Android'de Swiss Ephemeris için NDK ile derleme gerekiyor.** Dosya erişimi de sorun çıkarabilir; veri yolunun parametre olması bunu karşılıyor.
- **Danışan verisi şifrelenmemiş SQLite'ta duruyor.** v1 FileVault'a güveniyor; bu bilinçli bir kabul ve kullanıcı belgelerinde yazılacak. SQLCipher ile şifreleme v1 sonrasına kalıyor.
- **Ayarların haritaya kopyalanması toplu değişikliği zorlaştırıyor** ("tüm haritaları Lahiri'ye çevir" gibi). v1'de toplu ayar değişikliği yok; ihtiyaç olursa ayrı bir komut olarak eklenir.
