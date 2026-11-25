# Plan 07 — Gün doğumu, Panchang, Ashtakavarga ve Transit (M3, 2. kısım)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

> **Uygulama sonrası düzeltmeler (kod geçerlidir, bu plandaki ilgili bloklar değil):**
> - Task 4 `day_sunrise`: arama önce doğumdan 24 saat, sonra 30 saat öncesinden başlar (`SUNRISE_NEAR_START_DAYS`; bu plandaki `SUNRISE_RETRY_DAYS` ve başlangıç sırası hatalıdır, commit `fccede0`). Test sayıları bu yüzden plandan fazladır: `jyotish-core` lib +3, golden +1.
> - Task 2 `sun_rise_set`: 50° enlemin üstünde `SE_BIT_FORCE_SLOW_METHOD` eklenir; SE'nin hızlı yöntemi Hint tanımında 55°'nin üstünde 60 sn toleransını aşar ve `swetest -rise` de aynı hatayı taşır.

**Goal:** M3'ü tamamlamak: `Ephemeris` trait'ine `swe_rise_trans` tabanlı gün doğumu/batımı eklemek; `Chart`'a doğum anının panchangını (tithi, vara, nakshatra, yoga, karana, bitiş zamanları, gün doğumu ve batımı) ve ashtakavargayı (7 BAV + SAV) eklemek; saf `compute_transits` fonksiyonu ve Tauri komutu sağlamak; golden fixture'ları `swetest`'ten türetilmiş panchang ve SAV beklentileriyle genişletmek; Plan 06 son incelemesinin iki giriş koşulunu (D1, D2) kapatmak.

**Architecture:**
- `crates/ephemeris`: elle yazılmış FFI'ye `swe_rise_trans` eklenir. Trait yöntemi `sun_rise_set(after_jd_ut, RiseSet, lat, lon, SunDisc) -> Result<Option<f64>, EphError>`. SE'nin `-2` dönüşü (cisim doğmuyor/batmıyor) `Ok(None)` olur; hata değildir.
- `crates/jyotish-core/src/panchang.rs`: saf panchang. Numaralar doğum anındaki sidereal Güneş/Ay boylamlarından gelir. Bitiş zamanları ephemeris boylamları üzerinde 1.5 günlük aralıkta 20 adım ikiye bölmeyle (≈ 0.12 sn) bulunur. Gün doğumu doğumdan önceki son doğuştur; vara onun yerel ortalama tarihinden gelir.
- `crates/jyotish-core/src/ashtakavarga.rs`: BPHS böl. 66 tabloları, saf aritmetik. `Chart.ashtakavarga` rasi çerçevesindeki burçlardan hesaplanır.
- `crates/jyotish-core/src/transit.rs`: `compute_transits(eph, jd_ut, settings, natal_bhavas)`. Dokuz grahanın o andaki konumu natal bhava başlangıçlarına yerleştirilir. `compute_chart` ile ortak ham konum kodu `chart.rs`'ten paylaşılır.
- `app/src-tauri`: `compute_chart` yeni alanları döndürür; `compute_transits` yeni komuttur. `serde_json` `float_roundtrip` özelliği açılır (IPC float'ları bit düzeyinde korunur).
- Golden: `expected.panchang` ve `expected.sav` isteğe bağlıdır. 5 `swetest` fixture'ına, üretici betikte Rust motorundan bağımsız bir bash/awk uygulamasıyla eklenir (`swetest -rise -hindu` ve `swetest` boylamları üzerinde ikiye bölme).

**Tech Stack:** Rust 1.93, Swiss Ephemeris 2.10.03 (vendored, `swecl.c` zaten derleniyor), `serde` 1, `thiserror` 2, `serde_json` 1 (core'da dev; uygulamada normal bağımlılık, `float_roundtrip`), `proptest` 1 (dev), `specta` `=2.0.0-rc.28`, `tauri-specta` `=2.0.0-rc.29`, Tauri 2. **Yeni crate yok:** `serde_json` 1.0.154 zaten `Cargo.lock`'ta (Tauri üzerinden); yalnızca `jyotish-desk`'in doğrudan bağımlılığı olur.

**Spec:** `docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md` §3 (panchang, ashtakavarga, transit bindirme), §4 (core: panchang, ashtakavarga, transit), §5 (`Panchang`, `Ashtakavarga`, `Chart`), §6 ("Vara gün doğumunda değişir", "Gün doğumu: disk merkezi, kırılmasız (`SE_BIT_HINDU_RISING`). Ayarlanabilir."), §8 (panchang bitiş zamanları ve gün doğumu ≤ 60 sn; BAV/SAV birebir; SAV 337, BAV 48/49/39/54/56/52/39), §11 (M3). UI: `docs/superpowers/specs/2026-11-24-ui-design.md` §3.3 (transit halkası D1'de, transit tablosu, tarih varsayılanı "şimdi"), §3.4 (dasa dönemi seçilince transit tarihi). Önceki plan: `docs/superpowers/plans/2026-11-25-plan-06-vargas-dasha.md`. Ertelenenler: `.superpowers/sdd/plan06-final-review.md` D1, D2; `docs/superpowers/plans/notes-from-plan-01.md` ("`swe_rise_trans` + `SE_BIT_HINDU_RISING`").

## Global Constraints

- Lisans: AGPL-3.0-or-later (`license.workspace = true`). Yeni bağımlılıklar `deny.toml` lisans listesine uymalı ve `cargo deny check` temiz olmalı.
- "Kala", "Kāla", "Kaala", "Cala" ve türevleri hiçbir isimde, yorumda ya da dosya adında kullanılmaz. Bu, içinde bu harf dizisi geçen Türkçe ve Sanskritçe sözcükleri de kapsar (ör. "kalan", "kalabilir", "kalacak"): eklenen satırlarda `k(a|ā|aa)la` alt dizgisi bulunmaz. Her görevin commit adımında eklenen satırlar taranır.
- MSRV 1.93 (workspace). CI'da `core` (ubuntu, windows) ve `msrv` işleri çekirdek crate'leri kapsar; `jyotish-core` ve `ephemeris` çekirdek crate'tir.
- `jyotish-core` saftır: dosya, ağ, saat ve Tauri yok (testler hariç). `atlas`'a bağımlı değildir. Ephemeris'e yalnızca `ephemeris::Ephemeris` trait'i ile erişir. "Şimdi" anını çekirdek bilmez; transit tarihini arayüz verir.
- Çekirdekte panik yok: tüm hatalar `Result<_, CoreError>` olarak döner. FFI sınırında panik olmaz. Döngülerin hepsi sabit üst sınırlıdır.
- **Varsayılanlar (spec §6):**
  - Düğüm: Mean Rahu. True seçenek olarak var.
  - Ev sistemi: Whole Sign. Bhava chalit için Sripati, KP için Placidus seçenek olarak var.
  - Dasha yılı: 365.25 gün. Seçenekler: 360 ve 365.2422.
  - Yanma orbları (BPHS): Ay 12°, Mars 17°, Merkür 14° (retroda 12°), Jüpiter 11°, Venüs 10° (retroda 8°), Satürn 15°. Düğümler yanmaz.
  - Dignity: BPHS. Yücelme ve düşüş dereceleri, moolatrikona, kendi burcu ve panchadha maitri. Düğümlere dignity atanmaz.
  - Konumlar: Jeosantrik, görünür konum. **Vara gün doğumunda değişir.**
  - **Gün doğumu: Disk merkezi, kırılmasız (`SE_BIT_HINDU_RISING`). Ayarlanabilir** (`SunriseDef::UpperLimbRefracted`).
- **Toleranslar (spec §8):** Ephemeris sarmalayıcısı `swetest`'e karşı ≤ 0.001″ (konum); gün doğumu sarmalayıcısı `swetest`'e karşı ≤ 0.1 sn (`swetest` saati 0.1 sn'ye yuvarlar). Golden haritalarda graha boylamı ≤ 2″, ayanamsa ≤ 1″, lagna ≤ 5″, mahadasha başlangıcı ≤ 1 gün, varga burçları birebir. **Panchang bitiş zamanları ve gün doğumu ≤ 60 sn. BAV/SAV birebir.** Toleranslar gevşetilmez.
- **Plan 01 notları:**
  - Sidereal mod SE'de global durumdur, her çağrıda yeniden ayarlanır.
  - Ayanamsa nütasyon dahildir (`SEFLG_NONUT` yok).
  - Ketu = Rahu + 180°, tam olarak.
  - Harita hassasiyeti Güneş/Ay konumlarından alınır, düğümlerden ya da ayanamsadan değil.
  - `swe_rise_trans` hızlı yöntemi `swe_set_topo`'yu çağırır (global topo durumu). Hiçbir çağrımız `SEFLG_TOPOCTR` kullanmadığı için etkisizdir.
- **Serde:** IPC'yi geçecek tüm public tiplerde `Serialize` + `Deserialize` olur. JSON alan adları snake_case, enum varyantları PascalCase dizgidir (`"Shukla"`, `"Saturday"`, `"Vishti"`). IPC tiplerinde `#[serde(default)]`, `skip_serializing_if` ya da tek yönlü `rename` **yok**: `specta_builder()`'daki `disable_serde_phases()` buna dayanır. `Option<T>` alanları özniteliksiz kalır.
- **Zaman alanları** UT Jülyen günüdür ve adları `_jd_ut` ile biter (Plan 06 dasha kararıyla aynı). Spec §5'teki `ends_at`, `sunrise`, `sunset` adları bu yüzden `ends_jd_ut`, `sunrise_jd_ut`, `sunset_jd_ut` olur.
- **Kapsam dışı (sonraki planlar):** Panchang, ashtakavarga ve transit **arayüzü** (paneller, transit halkası çizimi): UI planı. Ay doğuşu/batışı, tithi/yoga/nakshatra **adları** (numara → ad eşlemesi i18n'de), muhurta, kaksha/prastara ashtakavarga, shodhana (trikona/ekadhipatya) indirgemesi, transit için vedha ve murti. `Chart.moment` (`ResolvedMoment`) hâlâ yoktur.
- Commit mesajları İngilizce olur ve `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` satırıyla biter. `.superpowers/` asla `git add` edilmez.
- Plan kod blokları rustfmt-temiz olmayabilir: her görevde doğrulamadan ve commit'ten önce `cargo fmt --all` çalıştır; `cargo fmt --all --check` CI'da zorunlu.
- Tamsayı bölünebilirliği `x % n == 0` ile değil `x.is_multiple_of(n)` ile yazılır (stable clippy `manual_is_multiple_of`; Rust 1.87'de kararlı).
- **Toolchain:** Bu makinede `cargo +1.93` kısayolu (rustup proxy) **yok**. Komutlar doğrudan toolchain dizininden çalışır:
  - Stable (fmt, clippy, test, deny): `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"` (her terminal oturumunda bir kez).
  - MSRV denetimi (1.93 kurulumunda clippy/rustfmt yok): `PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check …`. CI'daki `cargo +1.93 check` ile aynıdır.
- Plan 05 ve 06'nın astrolojik tanımları (dignity, yanma, ev sistemleri, vargalar, dasha) değişmez.

### Bu planın kararları

| Konu | Karar | Gerekçe |
|---|---|---|
| D1: IPC float gidiş-dönüşü | **Yapılır.** `app/src-tauri/Cargo.toml`'a `serde_json = { version = "1", features = ["float_roundtrip"] }`. | Ucuz (yeni crate yok). Özellik birleşimiyle Tauri'nin komut argümanı ayrıştırması da tam olur; `chart.dasha.seed` JS'ten döndüğünde dönem sınırları kaymaz. Varsayılan ayrıştırıcının `2459475.0167725882`'yi `2459475.016772588`'e kaydırdığı doğrulandı. |
| D2: `DashaSeed.birth_jd_ut` aralığı | **Yapılır**, ve aynı aralık `compute_chart`, `compute_panchang`, `compute_transits`'e de uygulanır: `JD_UT_RANGE = 625_002.0..=2_817_997.0`. | `compute_chart` bugün yalnızca `is_finite` denetler; bir aralık yoktur. SE Moshier aralığı `625000.5 … 2818000.5` ET'dir (`sweph.h` `MOSHPLEPH_START/END`). Panchang aramaları anın 1.25 gün öncesine ve 1.5 gün sonrasına uzanır; ΔT sınırlarda < 1 gündür. Ölçüldü: `swe_rise_trans` JD 2818000.0 UT'de `sepl_30.se1 not found` hatası verir. 2 ve 3 günlük paylar her çağrıyı aralıkta tutar. |
| Panchang hangi an için | Tithi, nakshatra, yoga ve karana **doğum anındaki** (`jd_ut`) değerlerdir; bitişleri o andan sonraki ilk sınırdır. Vara **doğum gününün gün doğumundan** gelir. | Doğum haritası panchangı (JHora "Panchanga" görünümü) doğum anını kullanır. Spec §6 vara için gün doğumunu ister. |
| "Doğum gününün gün doğumu" | Doğum yerinde, doğumdan önceki **en son** gün doğumu. `[jd − 1.25 g, jd]` penceresinde aranır; en çok 4 adım. SE'nin yüksek enlem araması başlangıçtan ~26 saat ileriye baktığından pencere iki başlangıçtan taranır (doğumdan 30 sa ve 24 sa önce). Gün batımı o gün doğumundan sonraki ilk batıştır ve 1 gün içinde olmalıdır. | Hint günü gün doğumundan gün doğumuna sürer. Ardışık gün doğumları 65° enleme kadar ~24 sa ± 15 dk arayla gelir; 1.25 günlük pencere en çok iki doğuş içerir, 4 adım sınırı güvenlik payıdır. |
| Konum | `compute_chart`'ın `geo_lat`/`geo_lon`'u (doğum yeri). Gözlemci deniz seviyesinde (yükseklik 0). | Atlas yükseklik vermez. Hindu tanımında (`SE_BIT_HINDU_RISING`) kırılma ve basınç kullanılmaz. |
| Gün doğumu tanımı | `SunriseDef::DiskCenterNoRefraction` → `SE_BIT_HINDU_RISING` (`DISC_CENTER \| NO_REFRACTION \| GEOCTR_NO_ECL_LAT`). `SunriseDef::UpperLimbRefracted` → bayraksız SE varsayılanı (üst kenar, kırılmalı), atmosfer 1013.25 hPa / 15 °C (`swetest.c` `datm` varsayılanı). | `settings.rs`'te zaten tanımlı iki seçenek; yenisi eklenmez. |
| Kutup gecesi/gündüzü | Pencerede gün doğumu yoksa `sunrise_jd_ut = None`, `sunset_jd_ut = None`. Vara, gün **06:00 yerel ortalama saatte** başlıyor sayılarak bulunur. `Chart.meta.warnings`'e `ChartWarning::SunriseUnavailable` eklenir. Panik ya da hata yok. | Spec bu durumu tanımlamaz; 06:00 LMT yaygın bir yedektir ve açıkça uyarılır. Tromsø fixture'ı (21.12.2050, kutup gecesi) bunu sınar. |
| Vara günü | Gün başlangıcı anının **yerel ortalama tarihi**: `floor(jd + 0.5 + boylam/360)`, hafta günü `(gün + 1) mod 7`, 0 = Pazar. | UT tarihi büyük boylamlarda gün doğumunu önceki güne atardı (ör. +170° boylamda 06:00 yerel = önceki gün 18:40 UT). |
| Bitiş zamanı arama | İkiye bölme, `[jd, jd + 1.5 g]`, **20 sabit adım** (1.5 g / 2²⁰ ≈ 0.12 sn), sonuç aralık ortası. Aralık sonunda sınıra ulaşılmadıysa `CoreError::PanchangSearch(öğe)`. | Ay'ın jeosantrik hızı ≈ 11.8–15.4°/gün, Güneş'inki 0.95–1.02°/gün: uzanım ≥ 10.7°/gün, Ay ≥ 11.8°/gün, toplam ≥ 12.7°/gün. En uzun dilim 13°20′/11.8 ≈ 1.13 gün < 1.5 gün. Aralıkta açı en çok ~25° ilerler; hedefe işaretli uzaklık (±180° sarmalı) sürekli ve artandır, ikiye bölme geçerlidir. Gereken adım `ceil(log2(1.5 × 86400 / 0.5)) = 18`; 20 paydır. Maliyet: 4 öğe × 22 değerlendirme × 2 SE çağrısı ≈ 180 çağrı ≈ 3 ms (ölçüldü: `swe_rise_trans` ≈ 40 µs, `swe_calc_ut` daha hızlı). |
| Panchang çerçevesi | Her zodyak kipinde **sidereal** Güneş/Ay (`Zodiac::Sidereal(ayanamsa)`). | Nakshatra ve yoga sidereal tanımlıdır; tithi/karana uzanımı ayanamsadan bağımsızdır. |
| Yoga tipinin adı | `NityaYoga` (27 panchang yogası). | İleride gelecek graha yogalarıyla (alt proje 3) karışmasın. |
| Adlar | Karana adı çekirdekte (`KaranaName`), çünkü eşleme kuralı (4 sabit + 7 hareketli × 8) hesaptır. Tithi, nakshatra ve yoga numaradır; ad i18n'dedir. | Numara → ad eşlemesi dil işidir; karana eşlemesi değildir. |
| Ashtakavarga çerçevesi | Rasi çerçevesindeki burçlar (`GrahaPosition.sign`, `Lagna.sign`): `Sidereal` kipinde sidereal, tropikal-rasi kipinde tropikal. Rahu/Ketu katılmaz. `bav: [[u8; 12]; 7]` (`Graha::ALL[..7]` sırası, Koç 0), `sav: [u8; 12]`. | Ashtakavarga D1 burçlarından sayılır; D1 her kipte `sign` ile aynıdır (Plan 06 varga kararıyla tutarlı). Spec §5 `bav[7][12]`, `sav[12]`. |
| Transit kapsamı | v1: `compute_transits(eph, jd_ut, settings, natal_bhavas) -> TransitSnapshot`. Dokuz graha: boylam, sidereal boylam, hız, retro, burç, derece, nakshatra, pada, `natal_house`. Lagna, dignity, yanma ve vedha yok. Tauri komutu `compute_transits(jd_ut, settings, natal_bhavas)`; arayüz `chart.meta.settings` ve `chart.bhavas`'ı geri gönderir. | UI spec §3.3: transit halkası yalnızca D1'de, transit tablosu tarih alır (varsayılan "şimdi", arayüz hesaplar). Natal haritanın tamamını (≈ 115 KB) IPC'den geri göndermek gereksiz; ev yerleşimi için bhava başlangıçları yeter (Whole Sign'da da `house_from_cusps` doğru sonucu verir, Plan 05 proptest'i). |

### Astrolojik tanımlar: Panchang

Kaynaklar: *Surya Siddhanta* böl. 2 (E. Burgess çevirisi: tithi, karana, nitya yoga ve nakshatra tanımları); *Muhurta Chintamani* böl. 1 (Shubhashubha prakarana: karanalar ve Vishti/Bhadra'nın yeri). Gösterim: `S`, `M` doğum anındaki sidereal Güneş ve Ay boylamı; `e = (M − S) mod 360`.

| Öğe | Tanım | Numara | Bitiş |
|---|---|---|---|
| Tithi | `e`'nin 12°'lik dilimi | `floor(e / 12) + 1`, 1..=30. 1–15 **Shukla**, 16–30 **Krishna** paksha. 15 Purnima, 30 Amavasya. | `e = no × 12°` (30. tithi 360°'de biter). |
| Karana | Yarım tithi, `e`'nin 6°'lik dilimi | `k = floor(e / 6) + 1`, 1..=60. **1 Kimstughna** (sabit); **2–57** hareketli yedili `[Bava, Balava, Kaulava, Taitila, Garaja, Vanija, Vishti]`, ad `HAREKETLİ[(k − 2) mod 7]` (8 tur); **58 Shakuni, 59 Chatushpada, 60 Naga** (sabit). | `e = k × 6°`. |
| Nakshatra | `M`'nin 13°20′'lik dilimi; pada 3°20′ | `nakshatra_pada(M)` (Plan 05). | `M = no × 13°20′`. |
| Yoga (nitya) | `(S + M) mod 360`'ın 13°20′'lik dilimi | `floor((S + M) mod 360 × 3 / 40) + 1`, 1..=27 (1 Vishkambha … 27 Vaidhriti). | `(S + M) mod 360 = no × 13°20′`. |
| Vara | Hafta günü (Ravi … Shani = Pazar … Cumartesi) | Gün doğumunun yerel ortalama tarihinden; doğum gün doğumundan önceyse **önceki günün** varası. | — |

Karana doğrulaması: Vishti (Bhadra) geleneksel olarak sekiz yarım tithiye düşer: Shukla 4. tithinin 2. yarısı, 8.'in 1., 11.'in 2., 15.'in 1. yarısı; Krishna 3.'nün 2., 7.'nin 1., 10.'un 2., 14.'ün 1. yarısı. Tithi `t`'nin 1. yarısı karana `2t − 1`, 2. yarısı `2t`'dir: karana 8, 15, 22, 29, 36, 43, 50, 57. Formül tam bu sekizini verir (Task 3 testi).

### Astrolojik tanımlar: Ashtakavarga (kaynak: BPHS böl. 66 "Ashtakavarga", R. Santhanam çevirisi; B. V. Raman, *Ashtakavarga System of Prediction* ile aynı tablolar)

Her hedef graha için sekiz referans (yedi graha ve lagna), **kendi burcundan sayınca** (1 = aynı burç) aşağıdaki evlere birer bindu verir. `BAV[hedef][burç]` = o burca bindu veren referans sayısı. `SAV[burç] = Σ BAV[hedef][burç]`. Tablo haritadan bağımsız olduğundan satır toplamları sabittir.

| Hedef ↓ / Referans → | Güneş | Ay | Mars | Merkür | Jüpiter | Venüs | Satürn | Lagna | Toplam |
|---|---|---|---|---|---|---|---|---|---|
| Güneş | 1,2,4,7,8,9,10,11 | 3,6,10,11 | 1,2,4,7,8,9,10,11 | 3,5,6,9,10,11,12 | 5,6,9,11 | 6,7,12 | 1,2,4,7,8,9,10,11 | 3,4,6,10,11,12 | **48** |
| Ay | 3,6,7,8,10,11 | 1,3,6,7,10,11 | 2,3,5,6,9,10,11 | 1,3,4,5,7,8,10,11 | 1,4,7,8,10,11,12 | 3,4,5,7,9,10,11 | 3,5,6,11 | 3,6,10,11 | **49** |
| Mars | 3,5,6,10,11 | 3,6,11 | 1,2,4,7,8,10,11 | 3,5,6,11 | 6,10,11,12 | 6,8,11,12 | 1,4,7,8,9,10,11 | 1,3,6,10,11 | **39** |
| Merkür | 5,6,9,11,12 | 2,4,6,8,10,11 | 1,2,4,7,8,9,10,11 | 1,3,5,6,9,10,11,12 | 6,8,11,12 | 1,2,3,4,5,8,9,11 | 1,2,4,7,8,9,10,11 | 1,2,4,6,8,10,11 | **54** |
| Jüpiter | 1,2,3,4,7,8,9,10,11 | 2,5,7,9,11 | 1,2,4,7,8,10,11 | 1,2,4,5,6,9,10,11 | 1,2,3,4,7,8,10,11 | 2,5,6,9,10,11 | 3,5,6,12 | 1,2,4,5,6,7,9,10,11 | **56** |
| Venüs | 8,11,12 | 1,2,3,4,5,8,9,11,12 | 3,5,6,9,11,12 | 3,5,6,9,11 | 5,8,9,10,11 | 1,2,3,4,5,8,9,10,11 | 3,4,5,8,9,10,11 | 1,2,3,4,5,8,9,11 | **52** |
| Satürn | 1,2,4,7,8,10,11 | 3,6,11 | 3,5,6,10,11,12 | 6,8,9,10,11,12 | 5,6,11,12 | 6,11,12 | 3,5,6,11 | 1,3,4,6,10,11 | **39** |

Toplam: 48 + 49 + 39 + 54 + 56 + 52 + 39 = **337** (spec §8). Satır toplamları plan yazılırken ayrı bir betikle yeniden sayıldı; sahte haritanın elle doğrulanmış BAV'ı Task 5'tedir.

### Bu planın gün doğumu ve panchang referans değerleri

`target/swetest` (vendored SE 2.10.03) ve doğrudan `swe_rise_trans` çağıran bir C programıyla plan yazılırken ölçüldü:

| Yer, başlangıç (UT) | Tanım | Doğuş | Batış |
|---|---|---|---|
| New Delhi, 1.1.2000 00:00 | `-hindu` | 01:47:58.7 (JD 2451544.57498462) | 12:00:48.4 (JD 2451545.00055998) |
| New Delhi, 1.1.2000 00:00 | kırılmalı üst kenar | 01:43:46.0 | 12:05:01.0 |
| İstanbul, 17.3.1987 00:00 | `-hindu` | 04:18:10.6 | 16:07:45.6 |
| Tromsø, 21.12.2050 00:00 | ikisi de | yok (`ret = -2`, "rise or set not found") | yok |

---

## Dosya Yapısı (bu plan)

```
crates/ephemeris/
  src/ffi.rs            # swe_rise_trans, SE_CALC_*, SE_BIT_HINDU_RISING, RISE_TRANS_NOT_FOUND (Task 2)
  src/types.rs          # RiseSet, SunDisc (Task 2)
  src/lib.rs            # Ephemeris::sun_rise_set, yeniden dışa aktarımlar (Task 2)
  src/swiss.rs          # SwissEph::sun_rise_set (Task 2)
  tests/rise_set.rs     # swetest karşılaştırması (Task 2)
crates/jyotish-core/
  src/settings.rs       # JD_UT_RANGE (Task 1), SunriseDef::sun_disc (Task 4), belge (Task 6)
  src/chart.rs          # check_moment (Task 1); panchang, ashtakavarga, uyarı (Task 6); raw_grahas, rasi_frame, to_sidereal (Task 7)
  src/dasha.rs          # birth_jd_ut aralığı (Task 1)
  src/panchang.rs       # tipler ve saf kurallar (Task 3), compute_panchang (Task 4)
  src/ashtakavarga.rs   # BPHS tabloları, compute_ashtakavarga (Task 5)
  src/transit.rs        # TransitGraha, TransitSnapshot, compute_transits (Task 7)
  src/error.rs          # PanchangSearch (Task 4), InvalidTransitInput (Task 7)
  src/fake.rs           # sahte gün doğumu (Task 2), doğrusal hareket ve disk kaydı (Task 4)
  src/proptests.rs      # panchang (Task 4), ashtakavarga (Task 5), harita (Task 6) özellikleri
  src/lib.rs            # modüller ve yeniden dışa aktarımlar
  tests/golden.rs       # panchang ve SAV karşılaştırması (Task 8)
scripts/gen-golden-swetest.sh   # panchang ve SAV türetmesi (Task 8)
tests/golden/README.md, *.json  # şema ve 5 fixture (Task 8)
app/src-tauri/
  Cargo.toml            # serde_json float_roundtrip (Task 1)
  src/commands.rs       # IPC float testi (Task 1), hata eşlemeleri (Task 4, 7), compute_transits (Task 9)
  src/lib.rs            # compute_transits kaydı (Task 9)
  tests/compute_chart.rs# panchang, ashtakavarga, transit (Task 9)
app/ui/bindings.ts      # yeniden üretilir (Task 6, 9)
```

`bindings.ts` her IPC tipi değişikliğinde yeniden üretilir (Task 6, 9), çünkü `bindings_ts_is_up_to_date` testi `cargo test --workspace`'in parçasıdır; her görev sonunda iş alanı yeşil olur.

Her görevin başında: `cd /Users/ttd/jyotish-desk && export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"`.

---
### Task 1: Giriş koşulları: hesap anı aralığı (D2) ve IPC float gidiş-dönüşü (D1)

**Files:**
- Modify: `crates/jyotish-core/src/settings.rs` (`JD_UT_RANGE`)
- Modify: `crates/jyotish-core/src/chart.rs` (`check_moment`, `compute_chart` girişi, `rejects_invalid_input`)
- Modify: `crates/jyotish-core/src/dasha.rs` (`DashaSeed::validate`, `invalid_seed_is_rejected`)
- Modify: `crates/jyotish-core/src/lib.rs` (yeniden dışa aktarım)
- Modify: `app/src-tauri/Cargo.toml`, `Cargo.lock` (yalnızca `jyotish-desk` bağımlılık listesi)
- Test: `app/src-tauri/src/commands.rs` içindeki `mod tests`

**Interfaces:**
- Consumes: `CoreError::{InvalidJulianDay, InvalidLocation, InvalidDashaInput}` (Plan 05–06).
- Produces:
  - `pub const JD_UT_RANGE: std::ops::RangeInclusive<f64> = 625_002.0..=2_817_997.0;` (`jyotish_core::settings`, kökten de dışa aktarılır).
  - `pub(crate) fn check_moment(jd_ut: f64, geo_lat: f64, geo_lon: f64) -> Result<(), CoreError>` (`crate::chart`): aralık dışı JD → `InvalidJulianDay(jd_ut)`, aralık dışı konum → `InvalidLocation`. Task 4 (`compute_panchang`) kullanır.
  - `DashaSeed::validate` `birth_jd_ut` için `JD_UT_RANGE` uygular.
  - Uygulama crate'inde `serde_json` `float_roundtrip` ile derlenir (Tauri IPC ayrıştırması dahil).

- [ ] **Step 1: Başarısız testleri yaz**

`crates/jyotish-core/src/chart.rs` içindeki `rejects_invalid_input` testinde `let bad = CalcSettings {` satırından **önce** ekle:

```rust
        // JD_UT_RANGE (D2): sonlu ama SE aralığı dışındaki anlar reddedilir.
        for jd in [1e300, -1e300, 625_001.0, 2_817_998.0] {
            assert_eq!(
                compute_chart(&mut eph, jd, 0.0, 0.0, &s),
                Err(CoreError::InvalidJulianDay(jd)),
                "{jd}"
            );
        }
        assert!(compute_chart(&mut eph, 625_002.0, 0.0, 0.0, &s).is_ok());
        assert!(compute_chart(&mut eph, 2_817_997.0, 0.0, 0.0, &s).is_ok());
```

`crates/jyotish-core/src/dasha.rs` içindeki `invalid_seed_is_rejected` testinde `for y in [0.0, 299.0, 1e308, f64::NAN] {` satırından **önce** ekle:

```rust
        // D2: sonlu ama hesap aralığı dışındaki doğum anı (ör. IPC'den gelen 1e300).
        for jd in [1e300, 625_001.0, 2_817_998.0] {
            assert_eq!(
                bad(DashaSeed {
                    birth_jd_ut: jd,
                    ..seed(0.0)
                }),
                CoreError::InvalidDashaInput("birth_jd_ut"),
                "{jd}"
            );
        }
```

`app/src-tauri/src/commands.rs` içindeki `mod tests`'in sonuna ekle:

```rust
    #[test]
    fn ipc_floats_round_trip_exactly() {
        // serde_json'un varsayılan ayrıştırıcısı bu değeri 1 ulp kaydırır (2459475.016772588).
        // `float_roundtrip` uygulama crate'inde açıktır; özellik birleşimiyle Tauri'nin komut
        // argümanı ayrıştırması da aynı yolu kullanır (Plan 06 incelemesi D1).
        let jd = 2_459_475.016_772_588_2_f64;
        let seed = DashaSeed {
            birth_jd_ut: jd,
            moon_sidereal_lon: 199.470_529,
            year_days: 365.25,
        };
        let json = serde_json::to_string(&seed).expect("serialize");
        assert!(json.contains("2459475.0167725882"), "{json}");
        let back: DashaSeed = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.birth_jd_ut.to_bits(), jd.to_bits());
        assert_eq!(back, seed);
    }
```

- [ ] **Step 2: Testlerin düştüğünü doğrula**

Run: `cargo test -p jyotish-core --lib -- rejects_invalid_input invalid_seed_is_rejected`
Expected: FAIL. `rejects_invalid_input`: `assertion `left == right` failed: 1e300` (sol taraf `Ok(Chart { … })`). `invalid_seed_is_rejected`: `called Result::unwrap_err() on an Ok value` (1e300 kabul edilir).

Run: `cargo test -p jyotish-desk --lib ipc_floats_round_trip_exactly`
Expected: FAIL, derleme hatası: `failed to resolve: use of unresolved module or unlinked crate 'serde_json'`.

- [ ] **Step 3: Aralığı ve denetimi yaz**

`crates/jyotish-core/src/settings.rs` içinde `pub const DASHA_YEAR_RANGE …;` satırından sonra ekle:

```rust

/// Kabul edilen hesap anı, UT Jülyen günü (yaklaşık −3000 … +3000 yılları). SE'nin Moshier
/// aralığı `625000.5 … 2818000.5` ET'dir (`sweph.h`, `MOSHPLEPH_START/END`). Panchang
/// aramaları anın 1.25 gün öncesine ve 1.5 gün sonrasına uzanır; ΔT sınırlarda 1 günden
/// azdır. 2 ve 3 günlük paylar her SE çağrısını aralıkta tutar. NaN ve sonsuz dışarıdadır.
pub const JD_UT_RANGE: std::ops::RangeInclusive<f64> = 625_002.0..=2_817_997.0;
```

`crates/jyotish-core/src/chart.rs` içinde `use crate::settings::{CalcSettings, HouseSystem, ZodiacMode};` satırını şununla değiştir:

```rust
use crate::settings::{CalcSettings, HouseSystem, ZodiacMode, JD_UT_RANGE};
```

`fn finite_body` fonksiyonundan sonra ekle:

```rust

/// Hesap anı ve konum denetimi; `compute_chart` ve `compute_panchang` ortak kullanır.
pub(crate) fn check_moment(jd_ut: f64, geo_lat: f64, geo_lon: f64) -> Result<(), CoreError> {
    // NaN ve sonsuz da aralık dışındadır.
    if !JD_UT_RANGE.contains(&jd_ut) {
        return Err(CoreError::InvalidJulianDay(jd_ut));
    }
    if !((-90.0..=90.0).contains(&geo_lat) && (-180.0..=180.0).contains(&geo_lon)) {
        return Err(CoreError::InvalidLocation {
            lat: geo_lat,
            lon: geo_lon,
        });
    }
    Ok(())
}
```

`compute_chart` içinde şu bloğu:

```rust
    settings.validate()?;
    if !jd_ut.is_finite() {
        return Err(CoreError::InvalidJulianDay(jd_ut));
    }
    if !((-90.0..=90.0).contains(&geo_lat) && (-180.0..=180.0).contains(&geo_lon)) {
        return Err(CoreError::InvalidLocation {
            lat: geo_lat,
            lon: geo_lon,
        });
    }
```

şununla değiştir:

```rust
    settings.validate()?;
    check_moment(jd_ut, geo_lat, geo_lon)?;
```

`crates/jyotish-core/src/dasha.rs` içinde `use crate::settings::DASHA_YEAR_RANGE;` satırını şununla değiştir:

```rust
use crate::settings::{DASHA_YEAR_RANGE, JD_UT_RANGE};
```

`DashaSeed::validate` içinde:

```rust
        if !self.birth_jd_ut.is_finite() {
            return Err(CoreError::InvalidDashaInput("birth_jd_ut"));
        }
```

şununla değiştir:

```rust
        // `compute_chart` ile aynı aralık; NaN ve sonsuz da dışarıdadır.
        if !JD_UT_RANGE.contains(&self.birth_jd_ut) {
            return Err(CoreError::InvalidDashaInput("birth_jd_ut"));
        }
```

`crates/jyotish-core/src/lib.rs` içindeki `pub use settings::{ … };` bloğunu şununla değiştir:

```rust
pub use settings::{
    AyanamsaSystem, CalcSettings, CombustionOrbs, HouseSystem, NodeKind, SunriseDef, ZodiacMode,
    CALC_SETTINGS_SCHEMA_VERSION, JD_UT_RANGE,
};
```

`app/src-tauri/Cargo.toml` içinde `[dependencies]` altında `serde = { version = "1", features = ["derive"] }` satırından sonra ekle:

```toml
# Normal bağımlılık (dev değil): özellik birleşimiyle Tauri'nin IPC ayrıştırması da float'ları
# bit düzeyinde korur (Plan 06 incelemesi D1). serde_json zaten Tauri üzerinden ağaçtadır.
serde_json = { version = "1", features = ["float_roundtrip"] }
```

- [ ] **Step 4: Testlerin geçtiğini doğrula**

Run: `cargo test -p jyotish-core && cargo test -p jyotish-desk --lib`
Expected: `jyotish-core` lib `83 passed`, golden `3 passed`; `jyotish-desk` lib `8 passed`.

Run: `git diff --stat Cargo.lock`
Expected: `Cargo.lock | 1 +` (yalnızca `jyotish-desk`'in `dependencies` listesine `"serde_json",`). Yeni `[[package]]` yok.

- [ ] **Step 5: Biçim, lint, deny ve commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
git add crates/jyotish-core/src/settings.rs crates/jyotish-core/src/chart.rs crates/jyotish-core/src/dasha.rs crates/jyotish-core/src/lib.rs app/src-tauri/Cargo.toml app/src-tauri/src/commands.rs Cargo.lock
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b' || true
git commit -m "fix(core): bound chart and dasha moments to the SE range; exact IPC floats" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: clippy temiz; `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`; yasaklı dizgi taraması boş çıktı verir.

---

### Task 2: Ephemeris: `swe_rise_trans` ile gün doğumu ve batımı

**Files:**
- Modify: `crates/ephemeris/src/ffi.rs`, `crates/ephemeris/src/types.rs`, `crates/ephemeris/src/lib.rs`, `crates/ephemeris/src/swiss.rs`
- Create: `crates/ephemeris/tests/rise_set.rs`
- Modify: `crates/jyotish-core/src/fake.rs` (sahte gün doğumu)
- Modify: `app/src-tauri/src/commands.rs` (test içindeki `Fake`)

**Interfaces:**
- Consumes: `swe_rise_trans` (`swephexp.h` 2.10.03; `swecl.c` zaten `build.rs`'te derleniyor), `Body::se_id`, `read_cstr`.
- Produces:
  - `pub enum RiseSet { Rise, Set }` ve `pub enum SunDisc { CenterNoRefraction, UpperLimbRefracted }` (`ephemeris` kökünden).
  - `Ephemeris::sun_rise_set(&mut self, after_jd_ut: f64, event: RiseSet, geo_lat: f64, geo_lon: f64, disc: SunDisc) -> Result<Option<f64>, EphError>`: `after_jd_ut`'den sonraki ilk olay (UT JD); olay yoksa `Ok(None)`; SE hatası `Err(EphError::Calc)`.
  - `FakeEph` (core, test): `sunrise_offset: f64` (0.67), `sunset_offset: f64` (0.15), `polar: bool` (false). Olaylar her gün `n + offset` (n tamsayı JD); `polar` ise `None`.

- [ ] **Step 1: Başarısız testi yaz**

`crates/ephemeris/tests/rise_set.rs` dosyasının tamamı:

```rust
//! `Ephemeris::sun_rise_set`'i `swetest -rise` çıktısıyla karşılaştırır.

mod common;

use common::ephe_dir;
use ephemeris::*;

/// `swetest` saati 0.1 sn'ye yuvarlar; aynı SE fonksiyonu çağrıldığından fark yalnızca
/// yuvarlamadır (plan yazılırken ölçülen fark < 0.05 sn).
const TOLERANCE_SECONDS: f64 = 0.1;

const DELHI: (f64, f64) = (28.6139, 77.2090);
const ISTANBUL: (f64, f64) = (41.0082, 28.9784);
const TROMSO: (f64, f64) = (69.6492, 18.9553);

fn ut(y: i32, mo: u32, d: u32, h: f64, mi: f64, s: f64) -> f64 {
    julday_gregorian(y, mo, d, h + mi / 60.0 + s / 3600.0)
}

fn event(
    eph: &mut SwissEph,
    start: f64,
    ev: RiseSet,
    (lat, lon): (f64, f64),
    disc: SunDisc,
) -> Option<f64> {
    eph.sun_rise_set(start, ev, lat, lon, disc).expect("rise_trans")
}

fn assert_at(label: &str, got: Option<f64>, want: f64) {
    let got = got.unwrap_or_else(|| panic!("{label}: no event"));
    let diff = (got - want).abs() * 86_400.0;
    assert!(
        diff <= TOLERANCE_SECONDS,
        "{label}: got {got:.8}, want {want:.8}, diff {diff:.3} s"
    );
}

#[test]
fn hindu_sunrise_and_sunset_match_swetest() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let hindu = SunDisc::CenterNoRefraction;

    // target/swetest -rise -hindu -b1.1.2000 -ut0:00 -p0 -geopos77.2090,28.6139,0 -head -eswe -edirdata/ephe
    // rise 1.01.2000 01:47:58.7   set 1.01.2000 12:00:48.4
    let start = ut(2000, 1, 1, 0.0, 0.0, 0.0);
    let rise = event(&mut eph, start, RiseSet::Rise, DELHI, hindu);
    assert_at("delhi rise", rise, ut(2000, 1, 1, 1.0, 47.0, 58.7));
    let set = event(&mut eph, start, RiseSet::Set, DELHI, hindu);
    assert_at("delhi set", set, ut(2000, 1, 1, 12.0, 0.0, 48.4));

    // target/swetest -rise -hindu -b17.3.1987 -ut0:00 -p0 -geopos28.9784,41.0082,0 -head -eswe -edirdata/ephe
    // rise 17.03.1987 04:18:10.6   set 17.03.1987 16:07:45.6
    let start = ut(1987, 3, 17, 0.0, 0.0, 0.0);
    let rise = event(&mut eph, start, RiseSet::Rise, ISTANBUL, hindu);
    assert_at("istanbul rise", rise, ut(1987, 3, 17, 4.0, 18.0, 10.6));
    let set = event(&mut eph, start, RiseSet::Set, ISTANBUL, hindu);
    assert_at("istanbul set", set, ut(1987, 3, 17, 16.0, 7.0, 45.6));
}

#[test]
fn refracted_upper_limb_matches_swetest_and_lengthens_the_day() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    // target/swetest -rise -b1.1.2000 -ut0:00 -p0 -geopos77.2090,28.6139,0 -head -eswe -edirdata/ephe
    // rise 1.01.2000 01:43:46.0   set 1.01.2000 12:05:01.0
    let start = ut(2000, 1, 1, 0.0, 0.0, 0.0);
    let upper = SunDisc::UpperLimbRefracted;
    let rise = event(&mut eph, start, RiseSet::Rise, DELHI, upper);
    assert_at("delhi upper rise", rise, ut(2000, 1, 1, 1.0, 43.0, 46.0));
    let set = event(&mut eph, start, RiseSet::Set, DELHI, upper);
    assert_at("delhi upper set", set, ut(2000, 1, 1, 12.0, 5.0, 1.0));
    // Üst kenar ve kırılma günü iki uçtan uzatır.
    let hindu = SunDisc::CenterNoRefraction;
    let hindu_rise = event(&mut eph, start, RiseSet::Rise, DELHI, hindu).unwrap();
    let hindu_set = event(&mut eph, start, RiseSet::Set, DELHI, hindu).unwrap();
    assert!(rise.unwrap() < hindu_rise && set.unwrap() > hindu_set);
}

#[test]
fn polar_night_has_no_sunrise_or_sunset() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    // target/swetest -rise -hindu -b21.12.2050 -ut0:00 -p0 -geopos18.9553,69.6492,0 -head -eswe -edirdata/ephe
    // rise - set -   (SE ret -2: "rise or set not found for planet 0")
    let start = ut(2050, 12, 21, 0.0, 0.0, 0.0);
    for disc in [SunDisc::CenterNoRefraction, SunDisc::UpperLimbRefracted] {
        for ev in [RiseSet::Rise, RiseSet::Set] {
            assert_eq!(event(&mut eph, start, ev, TROMSO, disc), None, "{disc:?} {ev:?}");
        }
    }
}

#[test]
fn next_event_comes_after_the_start() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let hindu = SunDisc::CenterNoRefraction;
    let first = event(&mut eph, ut(2000, 1, 1, 0.0, 0.0, 0.0), RiseSet::Rise, DELHI, hindu)
        .unwrap();
    // Bulunan doğuşun hemen sonrasından aramak ertesi günün doğuşunu verir (≈ 2451545.575167).
    let next = event(&mut eph, first + 0.01, RiseSet::Rise, DELHI, hindu).unwrap();
    assert!((0.99..1.01).contains(&(next - first)), "{}", next - first);
}
```

- [ ] **Step 2: Testin derlenmediğini doğrula**

Run: `cargo test -p ephemeris --test rise_set`
Expected: FAIL, derleme hatası: `cannot find type 'RiseSet' in this scope` ve `no method named 'sun_rise_set' found`.

- [ ] **Step 3: FFI, tipler ve trait yöntemi**

`crates/ephemeris/src/ffi.rs` içinde `pub const AS_MAXCH: usize = 256;` satırından sonra ekle:

```rust

/// `swe_rise_trans` olayları ve bayrakları (swephexp.h).
pub const SE_CALC_RISE: i32 = 1;
pub const SE_CALC_SET: i32 = 2;
pub const SE_BIT_DISC_CENTER: i32 = 256;
pub const SE_BIT_NO_REFRACTION: i32 = 512;
pub const SE_BIT_GEOCTR_NO_ECL_LAT: i32 = 128;
/// Hint gün doğumu: disk merkezi, kırılmasız, jeosantrik ve ekliptik enlemsiz (swephexp.h).
pub const SE_BIT_HINDU_RISING: i32 =
    SE_BIT_DISC_CENTER | SE_BIT_NO_REFRACTION | SE_BIT_GEOCTR_NO_ECL_LAT;
/// `swe_rise_trans` dönüşü: cisim bu aramada doğmuyor ya da batmıyor (swecl.c,
/// `swe_rise_trans_true_hor`: "rise or set not found"). Hata değildir.
pub const RISE_TRANS_NOT_FOUND: i32 = -2;
```

Aynı dosyada `extern "C"` bloğunun sonuna (`swe_houses_ex2` bildiriminden sonra) ekle:

```rust
    /// `starname` gezegenler için NULL. `geopos` 3 double: boylam (doğu +), enlem, yükseklik (m).
    /// Sonuç `*tret`'e yazılır; swetest.c `call_rise_set` de tek bir double geçirir.
    pub fn swe_rise_trans(
        tjd_ut: c_double,
        ipl: i32,
        starname: *mut c_char,
        epheflag: i32,
        rsmi: i32,
        geopos: *mut c_double,
        atpress: c_double,
        attemp: c_double,
        tret: *mut c_double,
        serr: *mut c_char,
    ) -> i32;
```

`crates/ephemeris/src/types.rs` dosyasının sonuna ekle:

```rust

/// Güneş olayı.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RiseSet {
    Rise,
    Set,
}

impl RiseSet {
    pub(crate) fn se_code(self) -> i32 {
        match self {
            Self::Rise => ffi::SE_CALC_RISE,
            Self::Set => ffi::SE_CALC_SET,
        }
    }
}

/// Doğuş ve batışta ufka değen nokta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SunDisc {
    /// Disk merkezi, kırılmasız, jeosantrik (`SE_BIT_HINDU_RISING`).
    CenterNoRefraction,
    /// Üst kenar, standart atmosfer kırılmasıyla (SE varsayılanı).
    UpperLimbRefracted,
}

impl SunDisc {
    pub(crate) fn rsmi_bits(self) -> i32 {
        match self {
            Self::CenterNoRefraction => ffi::SE_BIT_HINDU_RISING,
            Self::UpperLimbRefracted => 0,
        }
    }
}
```

`crates/ephemeris/src/lib.rs` içinde `pub use types::{ … };` bloğunu şununla değiştir:

```rust
pub use types::{
    Ayanamsa, Body, BodyPosition, HouseSystem, Houses, Precision, RiseSet, SiderealMode, SunDisc,
    Zodiac,
};
```

Aynı dosyada `Ephemeris` trait'inde `fn swe_version(&self) -> String;` satırından **önce** ekle:

```rust
    /// Güneş'in `after_jd_ut`'den sonraki ilk doğuşu ya da batışı, UT JD. Güneş bu aramada
    /// ufku geçmiyorsa (kutup gecesi ya da gündüzü) `Ok(None)`. `geo_lat` kuzey +,
    /// `geo_lon` doğu +, derece; gözlemci deniz seviyesinde.
    fn sun_rise_set(
        &mut self,
        after_jd_ut: f64,
        event: RiseSet,
        geo_lat: f64,
        geo_lon: f64,
        disc: SunDisc,
    ) -> Result<Option<f64>, EphError>;
```

`crates/ephemeris/src/swiss.rs` içinde `use crate::{ … };` bloğunu şununla değiştir:

```rust
use crate::{
    ffi, Ayanamsa, Body, BodyPosition, EphError, Ephemeris, HouseSystem, Houses, Precision,
    RiseSet, SiderealMode, SunDisc, Zodiac,
};
```

`const MAX_PATH_BYTES …;` satırından sonra ekle:

```rust

/// Kırılmalı tanım için standart atmosfer (swetest.c `datm` varsayılanı). Hint tanımında
/// (`SE_BIT_NO_REFRACTION`) SE bu değerleri kullanmaz.
const STD_PRESSURE_HPA: f64 = 1013.25;
const STD_TEMPERATURE_C: f64 = 15.0;
```

`impl Ephemeris for SwissEph` içinde `fn swe_version(&self) -> String {` satırından **önce** ekle:

```rust
    fn sun_rise_set(
        &mut self,
        after_jd_ut: f64,
        event: RiseSet,
        geo_lat: f64,
        geo_lon: f64,
        disc: SunDisc,
    ) -> Result<Option<f64>, EphError> {
        let mut geopos = [geo_lon, geo_lat, 0.0_f64];
        let mut tret = 0.0_f64;
        let mut serr: [c_char; ffi::AS_MAXCH] = [0; ffi::AS_MAXCH];
        // SAFETY: geopos 3 double; tret tek double (swecl.c yalnızca *tret yazar, swetest.c
        // de tek double geçirir); starname NULL (gezegen); serr AS_MAXCH; kilit tutuluyor.
        let ret = unsafe {
            ffi::swe_rise_trans(
                after_jd_ut,
                Body::Sun.se_id(),
                std::ptr::null_mut(),
                ffi::SEFLG_SWIEPH,
                event.se_code() | disc.rsmi_bits(),
                geopos.as_mut_ptr(),
                STD_PRESSURE_HPA,
                STD_TEMPERATURE_C,
                &mut tret,
                serr.as_mut_ptr(),
            )
        };
        if ret == ffi::RISE_TRANS_NOT_FOUND {
            return Ok(None);
        }
        if ret < 0 {
            return Err(EphError::Calc(read_cstr(&serr)));
        }
        // ret ≥ 0 iken serr Moshier uyarısı içerebilir; hassasiyet Güneş/Ay konumundan alınır.
        Ok(Some(tret))
    }
```

`crates/jyotish-core/src/fake.rs` içinde `use ephemeris::{ … };` bloğunu şununla değiştir:

```rust
use ephemeris::{
    Ayanamsa, Body, BodyPosition, EphError, Ephemeris, HouseSystem, Houses, Precision, RiseSet,
    SiderealMode, SunDisc, Zodiac,
};
```

`pub(crate) struct FakeEph` içinde `pub last_house_system: Option<HouseSystem>,` satırından sonra ekle:

```rust
    /// Sahte gün doğumu ve batımı her gün `n + offset` anındadır (n tamsayı JD; JD n + 0.5
    /// UT gece yarısıdır). 0.67 ≈ 04:05 UT, 0.15 ≈ 15:36 UT.
    pub sunrise_offset: f64,
    pub sunset_offset: f64,
    /// Güneş hiç doğup batmıyor (kutup gecesi ya da gündüzü).
    pub polar: bool,
```

`impl Default for FakeEph` içinde `last_house_system: None,` satırından sonra ekle:

```rust
            sunrise_offset: 0.67,
            sunset_offset: 0.15,
            polar: false,
```

`impl Ephemeris for FakeEph` içinde `fn swe_version(&self) -> String {` satırından **önce** ekle:

```rust
    fn sun_rise_set(
        &mut self,
        after_jd_ut: f64,
        event: RiseSet,
        _geo_lat: f64,
        _geo_lon: f64,
        _disc: SunDisc,
    ) -> Result<Option<f64>, EphError> {
        if self.polar {
            return Ok(None);
        }
        let offset = match event {
            RiseSet::Rise => self.sunrise_offset,
            RiseSet::Set => self.sunset_offset,
        };
        // `after_jd_ut`'den kesinlikle sonraki ilk `n + offset`.
        Ok(Some((after_jd_ut - offset).floor() + 1.0 + offset))
    }
```

`app/src-tauri/src/commands.rs` içindeki `mod tests`'te `use ephemeris::{Ayanamsa, BodyPosition, HouseSystem, Houses, Precision};` satırını şununla değiştir:

```rust
    use ephemeris::{Ayanamsa, BodyPosition, HouseSystem, Houses, Precision, RiseSet, SunDisc};
```

ve `impl Ephemeris for Fake` içinde `fn swe_version(&self) -> String {` satırından **önce** ekle:

```rust
        fn sun_rise_set(
            &mut self,
            _after: f64,
            _event: RiseSet,
            _lat: f64,
            _lon: f64,
            _disc: SunDisc,
        ) -> Result<Option<f64>, EphError> {
            Ok(None)
        }
```

- [ ] **Step 4: Testlerin geçtiğini doğrula**

Run: `cargo test -p ephemeris --test rise_set`
Expected: `4 passed`.

Run: `cargo test --workspace`
Expected: Hepsi geçer: `ephemeris` 37 (33 + 4), `jyotish-core` lib 83, golden 3, `jyotish-desk` lib 8, `bindings` 1, `compute_chart` 1, `atlas` 39.

- [ ] **Step 5: MSRV, biçim, lint ve commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p ephemeris -p jyotish-core --all-targets
git add crates/ephemeris/src crates/ephemeris/tests/rise_set.rs crates/jyotish-core/src/fake.rs app/src-tauri/src/commands.rs
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b' || true
git commit -m "feat(ephemeris): sunrise and sunset via swe_rise_trans with Hindu rising" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: clippy temiz, MSRV `cargo check` geçer, tarama boş.

---
### Task 3: `panchang.rs`: tipler ve saf kurallar (tithi, paksha, karana, yoga, vara)

**Files:**
- Create: `crates/jyotish-core/src/panchang.rs`
- Modify: `crates/jyotish-core/src/lib.rs`

**Interfaces:**
- Consumes: `crate::zodiac::normalize`.
- Produces (`jyotish_core::panchang`, kökten de dışa aktarılır):
  - `TITHI_SPAN_DEG = 12.0`, `KARANA_SPAN_DEG = 6.0`.
  - `enum Paksha { Shukla, Krishna }`; `enum Vara { Sunday, Monday, Tuesday, Wednesday, Thursday, Friday, Saturday }` ve `Vara::ALL: [Vara; 7]`; `enum KaranaName { Kimstughna, Bava, Balava, Kaulava, Taitila, Garaja, Vanija, Vishti, Shakuni, Chatushpada, Naga }`.
  - `struct Tithi { no: u8, paksha: Paksha, ends_jd_ut: f64 }`, `struct PanchangNakshatra { no: u8, pada: u8, ends_jd_ut: f64 }`, `struct NityaYoga { no: u8, ends_jd_ut: f64 }`, `struct Karana { no: u8, name: KaranaName, ends_jd_ut: f64 }`, `struct Panchang { tithi, vara, nakshatra, yoga, karana, sunrise_jd_ut: Option<f64>, sunset_jd_ut: Option<f64> }`.
  - `fn elongation(sun: f64, moon: f64) -> f64`, `fn tithi_no(elongation: f64) -> u8`, `fn paksha(tithi_no: u8) -> Paksha`, `fn karana_no(elongation: f64) -> u8`, `fn karana_name(no: u8) -> KaranaName`, `fn nitya_yoga_no(sun_sidereal: f64, moon_sidereal: f64) -> u8`, `fn local_weekday(jd_ut: f64, geo_lon: f64) -> Vara`.

- [ ] **Step 1: Başarısız testleri yaz**

`crates/jyotish-core/src/panchang.rs` dosyasını yalnızca testlerle oluştur:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tithi_and_paksha_follow_twelve_degree_steps() {
        assert_eq!(tithi_no(0.0), 1);
        assert_eq!(tithi_no(11.999_999), 1);
        assert_eq!(tithi_no(12.0), 2);
        assert_eq!(tithi_no(179.999_999), 15);
        assert_eq!(tithi_no(180.0), 16);
        assert_eq!(tithi_no(359.999_999), 30);
        assert_eq!(tithi_no(-1.0), 30);
        assert_eq!(tithi_no(372.0), 2);
        assert_eq!(paksha(1), Paksha::Shukla);
        assert_eq!(paksha(15), Paksha::Shukla);
        assert_eq!(paksha(16), Paksha::Krishna);
        assert_eq!(paksha(30), Paksha::Krishna);
        // Ay − Güneş: 71 − 346 = −275 → 85°.
        assert_eq!(elongation(346.0, 71.0), 85.0);
    }

    #[test]
    fn karana_has_four_fixed_and_seven_movable() {
        use KaranaName::*;
        assert_eq!(karana_no(0.0), 1);
        assert_eq!(karana_no(5.999_999), 1);
        assert_eq!(karana_no(6.0), 2);
        assert_eq!(karana_no(359.999_999), 60);
        let names: Vec<KaranaName> = (1..=60u8).map(karana_name).collect();
        assert_eq!(names[0], Kimstughna);
        let movable = [Bava, Balava, Kaulava, Taitila, Garaja, Vanija, Vishti];
        assert_eq!(names[1..8], movable);
        assert_eq!(names[8..15], movable);
        assert_eq!(names[50..57], movable);
        assert_eq!(names[57..], [Shakuni, Chatushpada, Naga]);
        for m in movable {
            assert_eq!(names.iter().filter(|&&n| n == m).count(), 8, "{m:?}");
        }
        for f in [Kimstughna, Shakuni, Chatushpada, Naga] {
            assert_eq!(names.iter().filter(|&&n| n == f).count(), 1, "{f:?}");
        }
        // Geçersiz numaralar panik yapmaz.
        assert_eq!(karana_name(0), Kimstughna);
        assert_eq!(karana_name(255), Naga);
    }

    #[test]
    fn vishti_falls_on_the_eight_traditional_half_tithis() {
        // Muhurta Chintamani: Vishti (Bhadra) Shukla 4. tithinin 2. yarısı, 8.'in 1., 11.'in 2.,
        // 15.'in 1. yarısı; Krishna 3.'nün 2., 7.'nin 1., 10.'un 2., 14.'ün 1. yarısı.
        // Tithi t'nin 1. yarısı karana 2t − 1, 2. yarısı 2t (Krishna için t = 15 + gün).
        let vishti: Vec<u8> = (1..=60u8)
            .filter(|&k| karana_name(k) == KaranaName::Vishti)
            .collect();
        assert_eq!(vishti, [8, 15, 22, 29, 36, 43, 50, 57]);
    }

    #[test]
    fn nitya_yoga_uses_the_sidereal_sum() {
        assert_eq!(nitya_yoga_no(0.0, 0.0), 1);
        assert_eq!(nitya_yoga_no(6.0, 7.333_334), 2); // 13.333334° > 13°20′
        assert_eq!(nitya_yoga_no(346.0, 71.0), 5); // 417° → 57°: Shobhana
        assert_eq!(nitya_yoga_no(180.0, 179.999_999), 27);
    }

    #[test]
    fn weekday_uses_the_local_mean_date() {
        // JD 2451545.0 = 1 Ocak 2000 12:00 UT, Cumartesi.
        assert_eq!(local_weekday(2_451_545.0, 0.0), Vara::Saturday);
        // 31 Aralık 1999 21:36 UT: Greenwich'te Cuma, New Delhi'de (≈ +5 sa 9 dk) 1 Ocak.
        assert_eq!(local_weekday(2_451_544.4, 0.0), Vara::Friday);
        assert_eq!(local_weekday(2_451_544.4, 77.209), Vara::Saturday);
        // Golden gün doğumları (Task 8): Londra 31.12.1849 Pazartesi, Buenos Aires 14.6.1923
        // Perşembe. İkisinde de doğum UT tarihinden farklı bir yerel güne düşer.
        assert_eq!(local_weekday(2_396_757.842_784, -0.1278), Vara::Monday);
        assert_eq!(local_weekday(2_423_584.959_912, -58.3816), Vara::Thursday);
        assert_eq!(Vara::ALL[0], Vara::Sunday);
        assert_eq!(Vara::ALL[6], Vara::Saturday);
    }
}
```

`crates/jyotish-core/src/lib.rs` içinde `mod houses;` satırından sonra ekle:

```rust
pub mod panchang;
```

- [ ] **Step 2: Testlerin derlenmediğini doğrula**

Run: `cargo test -p jyotish-core --lib panchang`
Expected: FAIL, derleme hatası: `cannot find function 'tithi_no' in this scope` (ve diğer adlar).

- [ ] **Step 3: Tipleri ve kuralları yaz**

`crates/jyotish-core/src/panchang.rs` dosyasının **başına** (testlerin üstüne) ekle:

```rust
//! Panchang: tithi, vara, nakshatra, yoga ve karana; gün doğumu ve batımı (spec §3, §5, §6).
//!
//! Tanımlar (Surya Siddhanta böl. 2, Burgess çevirisi; Muhurta Chintamani böl. 1):
//! - Tithi: Ay − Güneş uzanımının 12°'lik dilimi, 1..=30. 1–15 Shukla, 16–30 Krishna paksha.
//! - Karana: yarım tithi (6°), 1..=60. 1 Kimstughna (sabit); 2–57 yedi hareketli karananın
//!   (Bava, Balava, Kaulava, Taitila, Garaja, Vanija, Vishti) sekiz turu; 58 Shakuni,
//!   59 Chatushpada, 60 Naga (sabit).
//! - Nakshatra: Ay'ın sidereal boylamının 13°20′'lik dilimi; pada 3°20′.
//! - Nitya yoga: Güneş ve Ay sidereal boylamları toplamının 13°20′'lik dilimi, 1..=27.
//! - Vara: hafta günü; gün doğumunda değişir (spec §6).
//!
//! Uzanım iki boylamın farkı olduğu için ayanamsadan bağımsızdır; yoga toplamı değildir.
//! Zamanlar UT Jülyen günüdür.

use serde::{Deserialize, Serialize};

use crate::zodiac::normalize;

pub const TITHI_SPAN_DEG: f64 = 12.0;
pub const KARANA_SPAN_DEG: f64 = 6.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Paksha {
    /// Büyüyen Ay: tithi 1–15.
    Shukla,
    /// Küçülen Ay: tithi 16–30.
    Krishna,
}

/// Hafta günü: Ravi, Soma, Mangala, Budha, Guru, Shukra, Shani.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Vara {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Vara {
    /// 0 = Pazar.
    pub const ALL: [Vara; 7] = [
        Vara::Sunday,
        Vara::Monday,
        Vara::Tuesday,
        Vara::Wednesday,
        Vara::Thursday,
        Vara::Friday,
        Vara::Saturday,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum KaranaName {
    /// Sabit: Shukla 1. tithinin 1. yarısı (karana 1).
    Kimstughna,
    Bava,
    Balava,
    Kaulava,
    Taitila,
    Garaja,
    Vanija,
    /// Bhadra olarak da bilinir.
    Vishti,
    /// Sabit: Krishna 14. tithinin 2. yarısı (karana 58).
    Shakuni,
    /// Sabit: Amavasya'nın 1. yarısı (karana 59).
    Chatushpada,
    /// Sabit: Amavasya'nın 2. yarısı (karana 60).
    Naga,
}

const MOVABLE_KARANAS: [KaranaName; 7] = [
    KaranaName::Bava,
    KaranaName::Balava,
    KaranaName::Kaulava,
    KaranaName::Taitila,
    KaranaName::Garaja,
    KaranaName::Vanija,
    KaranaName::Vishti,
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Tithi {
    /// 1..=30.
    pub no: u8,
    pub paksha: Paksha,
    pub ends_jd_ut: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PanchangNakshatra {
    /// Ay'ın nakshatrası, 1..=27.
    pub no: u8,
    /// 1..=4.
    pub pada: u8,
    pub ends_jd_ut: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct NityaYoga {
    /// 1 Vishkambha … 27 Vaidhriti.
    pub no: u8,
    pub ends_jd_ut: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Karana {
    /// 1..=60 (yarım tithi sırası).
    pub no: u8,
    pub name: KaranaName,
    pub ends_jd_ut: f64,
}

/// Doğum anının panchangı (spec §5 `Panchang`). Tithi, nakshatra, yoga ve karana doğum
/// anındaki değerdir; `ends_jd_ut` o andan sonraki ilk sınırdır. Vara doğumdan önceki son
/// gün doğumundan gelir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Panchang {
    pub tithi: Tithi,
    pub vara: Vara,
    pub nakshatra: PanchangNakshatra,
    pub yoga: NityaYoga,
    pub karana: Karana,
    /// Doğumdan önceki son gün doğumu; kutup gecesi/gündüzünde `None`.
    pub sunrise_jd_ut: Option<f64>,
    /// O gün doğumundan sonraki ilk gün batımı (1 gün içinde); yoksa `None`.
    pub sunset_jd_ut: Option<f64>,
}

/// Ay − Güneş uzanımı, [0, 360).
pub fn elongation(sun: f64, moon: f64) -> f64 {
    normalize(moon - sun)
}

/// Uzanımdan tithi, 1..=30.
pub fn tithi_no(elongation: f64) -> u8 {
    ((normalize(elongation) / TITHI_SPAN_DEG) as u8).min(29) + 1
}

pub fn paksha(tithi_no: u8) -> Paksha {
    if tithi_no <= 15 {
        Paksha::Shukla
    } else {
        Paksha::Krishna
    }
}

/// Uzanımdan karana sırası, 1..=60.
pub fn karana_no(elongation: f64) -> u8 {
    ((normalize(elongation) / KARANA_SPAN_DEG) as u8).min(59) + 1
}

/// Karana sırasından ad. 1..=60 dışı değerler en yakın uca düşer (panik yok).
pub fn karana_name(no: u8) -> KaranaName {
    match no {
        0 | 1 => KaranaName::Kimstughna,
        2..=57 => MOVABLE_KARANAS[usize::from(no - 2) % 7],
        58 => KaranaName::Shakuni,
        59 => KaranaName::Chatushpada,
        _ => KaranaName::Naga,
    }
}

/// Sidereal Güneş + Ay toplamından nitya yoga, 1..=27.
pub fn nitya_yoga_no(sun_sidereal: f64, moon_sidereal: f64) -> u8 {
    ((normalize(sun_sidereal + moon_sidereal) * 3.0 / 40.0) as u8).min(26) + 1
}

/// `jd_ut` anının `geo_lon`'daki yerel ortalama tarihinin hafta günü: gün numarası
/// `floor(jd + 0.5 + boylam / 360)`, hafta günü `(gün + 1) mod 7`, 0 = Pazar.
pub fn local_weekday(jd_ut: f64, geo_lon: f64) -> Vara {
    let day = (jd_ut + 0.5 + geo_lon / 360.0).floor();
    // rem_euclid [0, 7) verir; NaN `as usize` ile 0'a düşer, min(6) sınırı korur.
    let i = (day + 1.0).rem_euclid(7.0) as usize;
    Vara::ALL[i.min(6)]
}
```

`crates/jyotish-core/src/lib.rs` içinde `pub use houses::{…};` satırından sonra ekle:

```rust
pub use panchang::{
    elongation, karana_name, karana_no, local_weekday, nitya_yoga_no, paksha, tithi_no, Karana,
    KaranaName, NityaYoga, Paksha, Panchang, PanchangNakshatra, Tithi, Vara, KARANA_SPAN_DEG,
    TITHI_SPAN_DEG,
};
```

- [ ] **Step 4: Testlerin geçtiğini doğrula**

Run: `cargo test -p jyotish-core --lib panchang`
Expected: `5 passed`.

Run: `cargo test -p jyotish-core`
Expected: lib `88 passed`, golden `3 passed`.

- [ ] **Step 5: Biçim, lint ve commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
git add crates/jyotish-core/src/panchang.rs crates/jyotish-core/src/lib.rs
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b' || true
git commit -m "feat(core): panchang element rules for tithi, karana, yoga and vara" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: clippy temiz, tarama boş.

---

### Task 4: `compute_panchang`: gün doğumu seçimi, vara kuralı ve bitiş zamanları

**Files:**
- Modify: `crates/jyotish-core/src/panchang.rs` (hesap bölümü ve `compute_tests`)
- Modify: `crates/jyotish-core/src/settings.rs` (`SunriseDef::sun_disc`)
- Modify: `crates/jyotish-core/src/error.rs` (`PanchangSearch`)
- Modify: `crates/jyotish-core/src/fake.rs` (doğrusal hareket, disk kaydı)
- Modify: `crates/jyotish-core/src/proptests.rs` (`mod panchang_properties`)
- Modify: `crates/jyotish-core/src/lib.rs`
- Modify: `app/src-tauri/src/commands.rs` (hata eşlemesi ve testi)

**Interfaces:**
- Consumes: `Ephemeris::{body, sun_rise_set}`, `RiseSet`, `SunDisc` (Task 2); `check_moment` (Task 1); Task 3'ün tipleri ve kuralları; `nakshatra_pada`, `NAKSHATRA_SPAN_DEG`.
- Produces:
  - `pub fn compute_panchang<E: Ephemeris + ?Sized>(eph: &mut E, jd_ut: f64, geo_lat: f64, geo_lon: f64, settings: &CalcSettings) -> Result<Panchang, CoreError>`.
  - `pub const END_SEARCH_DAYS: f64 = 1.5`, `pub const END_BISECTIONS: u32 = 20`, `pub const SUNRISE_WINDOW_DAYS: f64 = 1.25`.
  - `SunriseDef::sun_disc(self) -> ephemeris::SunDisc`.
  - `CoreError::PanchangSearch(&'static str)` (`"tithi"`, `"nakshatra"`, `"yoga"`, `"karana"`); IPC kodu `"panchang_search"`, parametre `what`.
  - `FakeEph`: `epoch: f64` (2_451_545.0; boylam `lon + hız × (jd − epoch)`), `last_sun_disc: Option<SunDisc>`.

- [ ] **Step 1: Başarısız testleri yaz**

`crates/jyotish-core/src/panchang.rs` dosyasının **sonuna** ekle:

```rust

#[cfg(test)]
mod compute_tests {
    use ephemeris::{Body, SunDisc};

    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::SunriseDef;

    const JD: f64 = 2_451_545.0;
    /// İkiye bölme 1.5 gün / 2^20 ≈ 0.12 sn verir; 0.2 sn pay.
    const TOL_DAYS: f64 = 0.2 / 86_400.0;

    fn panchang(eph: &mut FakeEph, jd: f64) -> Panchang {
        compute_panchang(eph, jd, 41.0, 29.0, &CalcSettings::default()).unwrap()
    }

    #[test]
    fn fake_panchang_follows_linear_motion() {
        // Sahte sidereal (tropikal − 24°): Güneş 346° + 0.98°/gün, Ay 71° + 13.2°/gün.
        let p = panchang(&mut FakeEph::default(), JD);
        // Uzanım 85°: 8. tithi (Shukla Ashtami), 96°'de biter; uzanım hızı 12.22°/gün.
        assert_eq!((p.tithi.no, p.tithi.paksha), (8, Paksha::Shukla));
        assert!((p.tithi.ends_jd_ut - (JD + 11.0 / 12.22)).abs() < TOL_DAYS);
        // Ay 71°: Ardra (6.), 2. pada; 80°'de biter.
        assert_eq!((p.nakshatra.no, p.nakshatra.pada), (6, 2));
        assert!((p.nakshatra.ends_jd_ut - (JD + 9.0 / 13.2)).abs() < TOL_DAYS);
        // Toplam 417° → 57°: 5. yoga (Shobhana), 66°40′'ta biter; toplam hız 14.18°/gün.
        assert_eq!(p.yoga.no, 5);
        assert!((p.yoga.ends_jd_ut - (JD + (200.0 / 3.0 - 57.0) / 14.18)).abs() < TOL_DAYS);
        // 85° / 6 = 14.17: 15. karana, (15 − 2) mod 7 = 6 → Vishti; 90°'de biter.
        assert_eq!((p.karana.no, p.karana.name), (15, KaranaName::Vishti));
        assert!((p.karana.ends_jd_ut - (JD + 5.0 / 12.22)).abs() < TOL_DAYS);
        // Sahte gün doğumu her gün n + 0.67 (04:05 UT), batımı n + 0.15 (15:36 UT).
        assert!((p.sunrise_jd_ut.unwrap() - 2_451_544.67).abs() < 1e-9);
        assert!((p.sunset_jd_ut.unwrap() - 2_451_545.15).abs() < 1e-9);
        // 2451544.67 + 29°/360 → yerel ortalama 1 Ocak 2000: Cumartesi.
        assert_eq!(p.vara, Vara::Saturday);
    }

    #[test]
    fn birth_before_sunrise_belongs_to_the_previous_vara() {
        // 1 Ocak 2000 02:24 UT: o günün sahte gün doğumu (04:05 UT) henüz olmadı.
        let p = panchang(&mut FakeEph::default(), 2_451_544.6);
        assert!((p.sunrise_jd_ut.unwrap() - 2_451_543.67).abs() < 1e-9);
        assert_eq!(p.vara, Vara::Friday);
        // 04:19 UT: gün doğumundan sonra.
        let p = panchang(&mut FakeEph::default(), 2_451_544.68);
        assert!((p.sunrise_jd_ut.unwrap() - 2_451_544.67).abs() < 1e-9);
        assert_eq!(p.vara, Vara::Saturday);
    }

    #[test]
    fn polar_day_starts_at_six_local_mean_time() {
        let polar = || FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        let p = panchang(&mut polar(), JD);
        assert_eq!((p.sunrise_jd_ut, p.sunset_jd_ut), (None, None));
        // 12:00 UT, boylam 29° → 13:56 yerel ortalama saat: 1 Ocak, Cumartesi.
        assert_eq!(p.vara, Vara::Saturday);
        // 02:24 UT → 04:20 yerel ortalama saat, 06:00'dan önce: önceki gün, Cuma.
        assert_eq!(panchang(&mut polar(), 2_451_544.6).vara, Vara::Friday);
    }

    #[test]
    fn sunrise_definition_selects_the_sun_disc() {
        let mut eph = FakeEph::default();
        panchang(&mut eph, JD);
        assert_eq!(eph.last_sun_disc, Some(SunDisc::CenterNoRefraction));
        let s = CalcSettings {
            sunrise_def: SunriseDef::UpperLimbRefracted,
            ..CalcSettings::default()
        };
        compute_panchang(&mut eph, JD, 41.0, 29.0, &s).unwrap();
        assert_eq!(eph.last_sun_disc, Some(SunDisc::UpperLimbRefracted));
    }

    #[test]
    fn invalid_input_and_a_stalled_moon_are_errors() {
        let s = CalcSettings::default();
        assert_eq!(
            compute_panchang(&mut FakeEph::default(), JD, 91.0, 0.0, &s),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 0.0
            })
        );
        assert_eq!(
            compute_panchang(&mut FakeEph::default(), 1e300, 0.0, 0.0, &s),
            Err(CoreError::InvalidJulianDay(1e300))
        );
        // Ay Güneş hızında: uzanım ilerlemez, sınır bulunamaz (gerçek ephemeris'te olmaz).
        let mut eph = FakeEph::default().with(Body::Moon, 95.0, 0.98);
        assert_eq!(
            compute_panchang(&mut eph, JD, 41.0, 29.0, &s),
            Err(CoreError::PanchangSearch("tithi"))
        );
    }
}
```

`crates/jyotish-core/src/settings.rs` içindeki `mod tests`'in sonuna ekle:

```rust

    #[test]
    fn sunrise_def_maps_to_sun_disc() {
        use ephemeris::SunDisc;
        assert_eq!(
            SunriseDef::DiskCenterNoRefraction.sun_disc(),
            SunDisc::CenterNoRefraction
        );
        assert_eq!(
            SunriseDef::UpperLimbRefracted.sun_disc(),
            SunDisc::UpperLimbRefracted
        );
    }
```

`crates/jyotish-core/src/proptests.rs` dosyasının sonuna ekle:

```rust

mod panchang_properties {
    use ephemeris::Body;
    use proptest::prelude::*;

    use crate::fake::FakeEph;
    use crate::panchang::{
        compute_panchang, elongation, karana_no, nitya_yoga_no, tithi_no, END_SEARCH_DAYS,
    };
    use crate::settings::CalcSettings;
    use crate::zodiac::{nakshatra_pada, normalize};

    const JD: f64 = 2_451_545.0;
    const ONE_SECOND: f64 = 1.0 / 86_400.0;
    /// FakeEph varsayılan ayanamsası.
    const AYANAMSA: f64 = 24.0;

    proptest! {
        #[test]
        fn end_times_close_the_current_element(
            sun in 0.0f64..360.0,
            moon in 0.0f64..360.0,
            sun_speed in 0.95f64..1.02,
            moon_speed in 11.8f64..15.4,
        ) {
            let mut eph = FakeEph::default()
                .with(Body::Sun, sun, sun_speed)
                .with(Body::Moon, moon, moon_speed);
            let p = compute_panchang(&mut eph, JD, 41.0, 29.0, &CalcSettings::default()).unwrap();
            let ends = [
                p.tithi.ends_jd_ut,
                p.nakshatra.ends_jd_ut,
                p.yoga.ends_jd_ut,
                p.karana.ends_jd_ut,
            ];
            for end in ends {
                prop_assert!((JD..=JD + END_SEARCH_DAYS).contains(&end), "{end}");
            }
            prop_assert!(p.karana.ends_jd_ut <= p.tithi.ends_jd_ut + ONE_SECOND);
            // Bitişten 1 sn sonra her öğe bir sonraki numaradadır (sahte hareket doğrusal).
            let after = |end: f64| {
                let dt = end + ONE_SECOND - JD;
                let s = normalize(sun - AYANAMSA + sun_speed * dt);
                let m = normalize(moon - AYANAMSA + moon_speed * dt);
                (s, m)
            };
            let (s, m) = after(p.tithi.ends_jd_ut);
            prop_assert_eq!(tithi_no(elongation(s, m)), p.tithi.no % 30 + 1);
            let (_, m) = after(p.nakshatra.ends_jd_ut);
            prop_assert_eq!(nakshatra_pada(m).0, p.nakshatra.no % 27 + 1);
            let (s, m) = after(p.yoga.ends_jd_ut);
            prop_assert_eq!(nitya_yoga_no(s, m), p.yoga.no % 27 + 1);
            let (s, m) = after(p.karana.ends_jd_ut);
            prop_assert_eq!(karana_no(elongation(s, m)), p.karana.no % 60 + 1);
        }
    }
}
```

`app/src-tauri/src/commands.rs` içindeki `core_error_codes_are_stable` testinin sonuna (son `assert_eq!`'ten sonra) ekle:

```rust
        let e = CommandError::from(CoreError::PanchangSearch("tithi"));
        assert_eq!(e.code, "panchang_search");
        assert_eq!(e.params["what"], "tithi");
```

- [ ] **Step 2: Testlerin derlenmediğini doğrula**

Run: `cargo test -p jyotish-core --lib`
Expected: FAIL, derleme hatası: `cannot find function 'compute_panchang' in this scope`, `no variant named 'PanchangSearch'`, `no method named 'sun_disc'`, `no field 'last_sun_disc'`.

- [ ] **Step 3: Hesabı yaz**

`crates/jyotish-core/src/error.rs` içinde `InvalidDashaInput` varyantından sonra ekle:

```rust
    /// Panchang öğesinin sonraki sınırı arama aralığında bulunamadı (yalnızca bozuk ya da
    /// sahte ephemeris'te: gerçek Ay her zaman Güneş'ten hızlıdır).
    #[error("no {0} boundary within the panchang search window")]
    PanchangSearch(&'static str),
```

`app/src-tauri/src/commands.rs` içinde `impl From<CoreError> for CommandError` eşlemesinde `CoreError::InvalidDashaInput(field) => { … }` kolundan sonra ekle:

```rust
            CoreError::PanchangSearch(what) => {
                Self::new("panchang_search", &[("what", what.to_owned())])
            }
```

`crates/jyotish-core/src/settings.rs` içinde `use ephemeris::{Body, SiderealMode};` satırını şununla değiştir:

```rust
use ephemeris::{Body, SiderealMode, SunDisc};
```

ve `pub enum SunriseDef { … }` tanımından sonra ekle:

```rust

impl SunriseDef {
    pub fn sun_disc(self) -> SunDisc {
        match self {
            Self::DiskCenterNoRefraction => SunDisc::CenterNoRefraction,
            Self::UpperLimbRefracted => SunDisc::UpperLimbRefracted,
        }
    }
}
```

`crates/jyotish-core/src/fake.rs` başındaki modül belgesini:

```rust
//! Birim testleri için sahte ephemeris. Tropikal boylamlar sabittir;
//! sidereal = tropikal − `ayanamsa`. Evler lagnadan eşit 30°'lik cusp'lardır.
```

şununla değiştir:

```rust
//! Birim testleri için sahte ephemeris. Tropikal boylamlar `epoch` anında verilir ve
//! sabit hızla doğrusal ilerler; sidereal = tropikal − `ayanamsa`. Evler lagnadan eşit
//! 30°'lik cusp'lardır. Gün doğumu/batımı her gün aynı UT saatindedir.
```

Aynı dosyada `pub polar: bool,` satırından sonra ekle:

```rust
    /// Konumların tanımlandığı an; boylam `lon + hız × (jd − epoch)` olarak ilerler.
    pub epoch: f64,
    /// Son `sun_rise_set` çağrısında istenen disk tanımı.
    pub last_sun_disc: Option<SunDisc>,
```

`impl Default for FakeEph` içinde `polar: false,` satırından sonra ekle:

```rust
            epoch: 2_451_545.0,
            last_sun_disc: None,
```

`impl Ephemeris for FakeEph` içindeki `body` yönteminin imzasını ve ilk satırlarını:

```rust
    fn body(&mut self, _jd_ut: f64, body: Body, zodiac: Zodiac) -> Result<BodyPosition, EphError> {
        let &(_, lon, speed) = self
            .positions
            .iter()
            .find(|p| p.0 == body)
            .ok_or_else(|| EphError::Calc(format!("fake: no {body:?}")))?;
        Ok(BodyPosition {
            lon: self.in_frame(lon, zodiac),
```

şununla değiştir:

```rust
    fn body(&mut self, jd_ut: f64, body: Body, zodiac: Zodiac) -> Result<BodyPosition, EphError> {
        let &(_, lon, speed) = self
            .positions
            .iter()
            .find(|p| p.0 == body)
            .ok_or_else(|| EphError::Calc(format!("fake: no {body:?}")))?;
        Ok(BodyPosition {
            lon: self.in_frame(lon + speed * (jd_ut - self.epoch), zodiac),
```

ve `sun_rise_set` içinde `_disc: SunDisc,` parametresini `disc: SunDisc,` yapıp gövdenin ilk satırı olarak ekle:

```rust
        self.last_sun_disc = Some(disc);
```

`crates/jyotish-core/src/panchang.rs` içinde Task 3'te yazılan `use` satırlarını:

```rust
use serde::{Deserialize, Serialize};

use crate::zodiac::normalize;
```

şununla değiştir:

```rust
use ephemeris::{Body, Ephemeris, RiseSet, SunDisc, Zodiac};
use serde::{Deserialize, Serialize};

use crate::chart::check_moment;
use crate::settings::CalcSettings;
use crate::zodiac::{nakshatra_pada, normalize, NAKSHATRA_SPAN_DEG};
use crate::CoreError;
```

`pub fn local_weekday` fonksiyonundan sonra (ilk `#[cfg(test)]`'ten önce) ekle:

```rust

/// Bitiş araması: sonraki sınır en geç bu kadar gün sonradır. Ay'ın jeosantrik boylam hızı
/// ≈ 11.8–15.4°/gün, Güneş'inki 0.95–1.02°/gün: uzanım ≥ 10.7°/gün, Ay ≥ 11.8°/gün, toplam
/// ≥ 12.7°/gün. En uzun dilim 13°20′ / 11.8 ≈ 1.13 gündür. Aralıkta açı en çok ~25° ilerler,
/// yani hedefe ±180° sarmalı uzaklık aralık boyunca sürekli ve artandır.
pub const END_SEARCH_DAYS: f64 = 1.5;
/// Sabit adım sayısı: 1.5 gün / 2^20 ≈ 0.12 sn (gereken 18 adım ≈ 0.5 sn).
pub const END_BISECTIONS: u32 = 20;
/// Doğum gününün gün doğumu, doğumdan en çok bu kadar önce aranır. Ardışık gün doğumları
/// 65° enleme kadar ~24 sa ± 15 dk arayla gelir; pencerede en çok iki doğuş olur.
pub const SUNRISE_WINDOW_DAYS: f64 = 1.25;
/// İlk aramada doğuş bulunamazsa ikinci aramanın başlangıcı (doğumdan bu kadar önce).
const SUNRISE_RETRY_DAYS: f64 = 1.0;
/// Pencerede taranan en çok doğuş (güvenlik sınırı).
const MAX_SUNRISE_STEPS: usize = 4;
/// Bulunan doğuşun ardından bir sonrakini aramak için atlanan süre (≈ 14 dk).
const RISE_STEP_DAYS: f64 = 0.01;
/// Gün batımı gün doğumundan sonra bu süre içinde olmalıdır; değilse yok sayılır.
const MAX_DAYLIGHT_DAYS: f64 = 1.0;
/// Gün doğumu yoksa günün başladığı yerel ortalama saat: 06:00 = 0.25 gün.
const FALLBACK_DAY_START: f64 = 0.25;

/// Panchang açısı.
#[derive(Clone, Copy)]
enum Angle {
    /// Ay − Güneş (tithi, karana).
    Elongation,
    /// Ay (nakshatra).
    Moon,
    /// Güneş + Ay (nitya yoga).
    Sum,
}

impl Angle {
    fn of(self, sun: f64, moon: f64) -> f64 {
        match self {
            Self::Elongation => elongation(sun, moon),
            Self::Moon => normalize(moon),
            Self::Sum => normalize(sun + moon),
        }
    }
}

/// Sidereal Güneş ve Ay boylamı.
fn sun_moon<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    zodiac: Zodiac,
) -> Result<(f64, f64), CoreError> {
    let sun = eph.body(jd_ut, Body::Sun, zodiac)?.lon;
    let moon = eph.body(jd_ut, Body::Moon, zodiac)?.lon;
    if sun.is_finite() && moon.is_finite() {
        Ok((sun, moon))
    } else {
        Err(CoreError::NonFiniteResult("panchang"))
    }
}

/// `angle`'ın `jd_ut`'den sonra `target` dereceye (0..=360) ulaştığı ilk an. İkiye bölme,
/// `[jd_ut, jd_ut + END_SEARCH_DAYS]`, `END_BISECTIONS` sabit adım; sonuç aralık ortası.
fn next_boundary<E: Ephemeris + ?Sized>(
    eph: &mut E,
    zodiac: Zodiac,
    jd_ut: f64,
    angle: Angle,
    target: f64,
    what: &'static str,
) -> Result<f64, CoreError> {
    // Hedefe işaretli uzaklık, [-180, 180): başta negatif, sınırdan sonra pozitif.
    let offset = |a: f64| normalize(a - target + 180.0) - 180.0;
    let mut lo = jd_ut;
    let mut hi = jd_ut + END_SEARCH_DAYS;
    let (s, m) = sun_moon(eph, hi, zodiac)?;
    if offset(angle.of(s, m)) < 0.0 {
        return Err(CoreError::PanchangSearch(what));
    }
    for _ in 0..END_BISECTIONS {
        let mid = 0.5 * (lo + hi);
        let (s, m) = sun_moon(eph, mid, zodiac)?;
        if offset(angle.of(s, m)) >= 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Ok(0.5 * (lo + hi))
}

/// Doğumdan önceki son gün doğumu, `[jd_ut − SUNRISE_WINDOW_DAYS, jd_ut]` içinde.
fn day_sunrise<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    geo_lat: f64,
    geo_lon: f64,
    disc: SunDisc,
) -> Result<Option<f64>, CoreError> {
    let mut found = None;
    // SE'nin yüksek enlem araması (|enlem| > 65°) başlangıçtan yalnızca ~26 saat ileriye
    // bakar (swecl.c `swe_rise_trans_true_hor`); 30 saatlik pencerenin son saatleri ikinci
    // başlangıçla taranır.
    for start in [jd_ut - SUNRISE_WINDOW_DAYS, jd_ut - SUNRISE_RETRY_DAYS] {
        let mut from = start;
        for _ in 0..MAX_SUNRISE_STEPS {
            match eph.sun_rise_set(from, RiseSet::Rise, geo_lat, geo_lon, disc)? {
                Some(t) if !t.is_finite() => return Err(CoreError::NonFiniteResult("sunrise")),
                Some(t) if t > from && t <= jd_ut => {
                    found = Some(t);
                    from = t + RISE_STEP_DAYS;
                }
                _ => break,
            }
        }
        if found.is_some() {
            break;
        }
    }
    Ok(found)
}

/// `sunrise`'tan sonraki ilk gün batımı; 1 gün içinde değilse `None`.
fn day_sunset<E: Ephemeris + ?Sized>(
    eph: &mut E,
    sunrise: f64,
    geo_lat: f64,
    geo_lon: f64,
    disc: SunDisc,
) -> Result<Option<f64>, CoreError> {
    match eph.sun_rise_set(sunrise, RiseSet::Set, geo_lat, geo_lon, disc)? {
        Some(t) if !t.is_finite() => Err(CoreError::NonFiniteResult("sunset")),
        Some(t) if t > sunrise && t - sunrise < MAX_DAYLIGHT_DAYS => Ok(Some(t)),
        _ => Ok(None),
    }
}

/// Doğum anının panchangı (spec §5). Güneş ve Ay her zodyak kipinde sidereal sorulur.
/// Gün doğumu `settings.sunrise_def` tanımıyla, doğum yerinde (`geo_lat` kuzey +, `geo_lon`
/// doğu +) ve deniz seviyesinde hesaplanır.
pub fn compute_panchang<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    geo_lat: f64,
    geo_lon: f64,
    settings: &CalcSettings,
) -> Result<Panchang, CoreError> {
    settings.validate()?;
    check_moment(jd_ut, geo_lat, geo_lon)?;
    let zodiac = Zodiac::Sidereal(settings.ayanamsa.sidereal_mode());
    let disc = settings.sunrise_def.sun_disc();

    let sunrise_jd_ut = day_sunrise(eph, jd_ut, geo_lat, geo_lon, disc)?;
    let sunset_jd_ut = match sunrise_jd_ut {
        Some(r) => day_sunset(eph, r, geo_lat, geo_lon, disc)?,
        None => None,
    };
    // Vara gün doğumunda değişir; gün doğumu yoksa gün 06:00 yerel ortalama saatte başlar.
    let vara = match sunrise_jd_ut {
        Some(r) => local_weekday(r, geo_lon),
        None => local_weekday(jd_ut - FALLBACK_DAY_START, geo_lon),
    };

    let (sun, moon) = sun_moon(eph, jd_ut, zodiac)?;
    let e = elongation(sun, moon);

    let t_no = tithi_no(e);
    let tithi = Tithi {
        no: t_no,
        paksha: paksha(t_no),
        ends_jd_ut: next_boundary(
            eph,
            zodiac,
            jd_ut,
            Angle::Elongation,
            f64::from(t_no) * TITHI_SPAN_DEG,
            "tithi",
        )?,
    };
    let (n_no, pada) = nakshatra_pada(moon);
    let nakshatra = PanchangNakshatra {
        no: n_no,
        pada,
        ends_jd_ut: next_boundary(
            eph,
            zodiac,
            jd_ut,
            Angle::Moon,
            f64::from(n_no) * NAKSHATRA_SPAN_DEG,
            "nakshatra",
        )?,
    };
    let y_no = nitya_yoga_no(sun, moon);
    let yoga = NityaYoga {
        no: y_no,
        ends_jd_ut: next_boundary(
            eph,
            zodiac,
            jd_ut,
            Angle::Sum,
            f64::from(y_no) * NAKSHATRA_SPAN_DEG,
            "yoga",
        )?,
    };
    let k_no = karana_no(e);
    let karana = Karana {
        no: k_no,
        name: karana_name(k_no),
        ends_jd_ut: next_boundary(
            eph,
            zodiac,
            jd_ut,
            Angle::Elongation,
            f64::from(k_no) * KARANA_SPAN_DEG,
            "karana",
        )?,
    };
    Ok(Panchang {
        tithi,
        vara,
        nakshatra,
        yoga,
        karana,
        sunrise_jd_ut,
        sunset_jd_ut,
    })
}
```

`crates/jyotish-core/src/lib.rs` içinde Task 3'te eklenen `pub use panchang::{ … };` bloğunu şununla değiştir:

```rust
pub use panchang::{
    compute_panchang, elongation, karana_name, karana_no, local_weekday, nitya_yoga_no, paksha,
    tithi_no, Karana, KaranaName, NityaYoga, Paksha, Panchang, PanchangNakshatra, Tithi, Vara,
    END_BISECTIONS, END_SEARCH_DAYS, KARANA_SPAN_DEG, SUNRISE_WINDOW_DAYS, TITHI_SPAN_DEG,
};
```

- [ ] **Step 4: Testlerin geçtiğini doğrula**

Run: `cargo test -p jyotish-core --lib panchang`
Expected: `11 passed` (Task 3'ün 5'i, `compute_tests` 5, `panchang_properties` 1).

Run: `cargo test --workspace`
Expected: Hepsi geçer: `jyotish-core` lib `95 passed` (88 + 5 + 1 + `sunrise_def_maps_to_sun_disc`), golden 3, `jyotish-desk` lib 8, `ephemeris` 37. Sahte konumların zamanla ilerlemesi mevcut testleri etkilemez: hepsi `epoch` anında (JD 2451545.0) çalışır.

- [ ] **Step 5: Biçim, lint ve commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
git add crates/jyotish-core/src app/src-tauri/src/commands.rs
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b' || true
git commit -m "feat(core): compute_panchang with sunrise-based vara and bisected end times" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: clippy temiz, tarama boş.

---
### Task 5: `ashtakavarga.rs`: BPHS tabloları, BAV ve SAV

**Files:**
- Create: `crates/jyotish-core/src/ashtakavarga.rs`
- Modify: `crates/jyotish-core/src/lib.rs`
- Modify: `crates/jyotish-core/src/proptests.rs` (`mod ashtakavarga_properties`)

**Interfaces:**
- Consumes: `Graha`, `Sign::{index, from_index}`.
- Produces (`jyotish_core::ashtakavarga`, kökten de dışa aktarılır):
  - `pub const ASHTAKAVARGA_GRAHAS: [Graha; 7]` (Güneş … Satürn, `Graha::ALL[..7]`).
  - `pub const BAV_TOTALS: [u8; 7] = [48, 49, 39, 54, 56, 52, 39]`, `pub const SAV_TOTAL: u16 = 337`.
  - `pub struct Ashtakavarga { pub bav: [[u8; 12]; 7], pub sav: [u8; 12] }` (Serialize, Deserialize, specta).
  - `pub fn compute_ashtakavarga(signs: [Sign; 7], lagna: Sign) -> Ashtakavarga` (toplam fonksiyon, hata yok).

- [ ] **Step 1: Başarısız testleri yaz**

`crates/jyotish-core/src/ashtakavarga.rs` dosyasını yalnızca testlerle oluştur:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::zodiac::Sign::*;

    #[test]
    fn tables_have_the_bphs_totals() {
        let mut all = 0u16;
        for (g, table) in BENEFIC_PLACES.iter().enumerate() {
            let n: usize = table.iter().map(|places| places.len()).sum();
            assert_eq!(n, usize::from(BAV_TOTALS[g]), "{:?}", ASHTAKAVARGA_GRAHAS[g]);
            for places in table {
                assert!(places.windows(2).all(|w| w[0] < w[1]), "{places:?}");
                assert!(places.iter().all(|p| (1..=12).contains(p)), "{places:?}");
            }
            all += u16::from(BAV_TOTALS[g]);
        }
        assert_eq!(all, SAV_TOTAL);
        assert_eq!(ASHTAKAVARGA_GRAHAS[..], Graha::ALL[..7]);
    }

    #[test]
    fn hand_checked_chart() {
        // FakeEph varsayılan haritası (sidereal): Güneş Balık, Ay İkizler, Mars Başak,
        // Merkür Balık, Jüpiter Akrep, Venüs Koç, Satürn Oğlak; lagna Boğa.
        let a = compute_ashtakavarga(
            [Pisces, Gemini, Virgo, Pisces, Scorpio, Aries, Capricorn],
            Taurus,
        );
        // Güneş BAV'ı elle, Koç sütunu: Güneş'ten (Balık) 2., Ay'dan (İkizler) 11., Mars'tan
        // (Başak) 8., Jüpiter'den (Akrep) 6., Satürn'den (Oğlak) 4., lagnadan (Boğa) 12. ev:
        // altısı da Güneş tablosunda → 6. Merkür'den (Balık) 2. ve Venüs'ten (Koç) 1. ev
        // tabloda yok. Balık sütunu: Güneş 1., Ay 10., Mars 7., Jüpiter 5., Venüs 12.,
        // lagna 11. → 6.
        assert_eq!(a.bav[0], [6, 2, 2, 5, 4, 5, 5, 4, 3, 3, 3, 6]);
        assert_eq!(a.bav[1], [1, 5, 6, 4, 4, 3, 6, 4, 4, 4, 4, 4]);
        assert_eq!(a.bav[2], [4, 3, 1, 5, 5, 4, 4, 3, 2, 3, 2, 3]);
        assert_eq!(a.bav[3], [5, 4, 4, 6, 5, 4, 4, 5, 4, 4, 5, 4]);
        assert_eq!(a.bav[4], [4, 5, 6, 3, 4, 5, 4, 4, 7, 5, 4, 5]);
        assert_eq!(a.bav[5], [3, 6, 4, 6, 7, 4, 3, 4, 2, 6, 4, 3]);
        assert_eq!(a.bav[6], [3, 2, 3, 2, 4, 3, 4, 4, 2, 3, 4, 5]);
        assert_eq!(a.sav, [26, 27, 26, 31, 33, 28, 30, 28, 24, 28, 26, 30]);
    }

    #[test]
    fn shifting_every_sign_rotates_the_tables() {
        let signs = [Pisces, Gemini, Virgo, Pisces, Scorpio, Aries, Capricorn];
        let base = compute_ashtakavarga(signs, Taurus);
        for k in 1..12 {
            let shifted = compute_ashtakavarga(
                signs.map(|s| Sign::from_index(s.index() + k)),
                Sign::from_index(Taurus.index() + k),
            );
            for s in 0..12 {
                let from = (s + 12 - k) % 12;
                assert_eq!(shifted.sav[s], base.sav[from], "k {k}, sign {s}");
                for g in 0..7 {
                    assert_eq!(shifted.bav[g][s], base.bav[g][from], "k {k}, graha {g}");
                }
            }
        }
    }
}
```

`crates/jyotish-core/src/lib.rs` içinde `mod chart;` satırından **önce** ekle:

```rust
pub mod ashtakavarga;
```

`crates/jyotish-core/src/proptests.rs` dosyasının sonuna ekle:

```rust

mod ashtakavarga_properties {
    use proptest::prelude::*;

    use crate::ashtakavarga::{compute_ashtakavarga, BAV_TOTALS, SAV_TOTAL};
    use crate::zodiac::Sign;

    proptest! {
        #[test]
        fn bav_and_sav_totals_are_fixed(
            idx in proptest::array::uniform7(0usize..12),
            lagna in 0usize..12,
        ) {
            let a = compute_ashtakavarga(idx.map(Sign::from_index), Sign::from_index(lagna));
            for (row, total) in a.bav.iter().zip(BAV_TOTALS) {
                prop_assert_eq!(row.iter().map(|&v| u16::from(v)).sum::<u16>(), u16::from(total));
                // Bir burca en çok sekiz referans bindu verir.
                prop_assert!(row.iter().all(|&v| v <= 8));
            }
            prop_assert_eq!(a.sav.iter().map(|&v| u16::from(v)).sum::<u16>(), SAV_TOTAL);
            for s in 0..12 {
                prop_assert_eq!(a.sav[s], a.bav.iter().map(|row| row[s]).sum::<u8>());
            }
        }
    }
}
```

- [ ] **Step 2: Testlerin derlenmediğini doğrula**

Run: `cargo test -p jyotish-core --lib ashtakavarga`
Expected: FAIL, derleme hatası: `cannot find value 'BENEFIC_PLACES' in this scope`, `cannot find function 'compute_ashtakavarga'`.

- [ ] **Step 3: Tabloyu ve hesabı yaz**

`crates/jyotish-core/src/ashtakavarga.rs` dosyasının **başına** (testlerin üstüne) ekle:

```rust
//! Ashtakavarga: yedi grahanın bhinnashtakavargası (BAV) ve sarvashtakavarga (SAV)
//! (spec §3, §5).
//!
//! Kaynak: BPHS böl. 66 (R. Santhanam çevirisi); B. V. Raman, *Ashtakavarga System of
//! Prediction* ile aynı tablolar. Her hedef graha için sekiz referansın (yedi graha ve
//! lagna) her biri, kendi burcundan sayılan belirli evlere birer bindu verir; aynı burç
//! 1. evdir. Rahu ve Ketu katılmaz. Trikona/ekadhipatya shodhana kapsam dışıdır.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::zodiac::Sign;

/// BAV satır sırası ve referans sırasının ilk yedisi.
pub const ASHTAKAVARGA_GRAHAS: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// BPHS toplamları: her BAV'ın bindu sayısı burç yerleşiminden bağımsızdır.
pub const BAV_TOTALS: [u8; 7] = [48, 49, 39, 54, 56, 52, 39];
pub const SAV_TOTAL: u16 = 337;

/// `BENEFIC_PLACES[hedef][referans]`: referansın burcundan sayılan bindu evleri.
/// Referans sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Lagna.
const BENEFIC_PLACES: [[&[u8]; 8]; 7] = [
    // Güneş
    [
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 6, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 5, 6, 9, 10, 11, 12],
        &[5, 6, 9, 11],
        &[6, 7, 12],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 4, 6, 10, 11, 12],
    ],
    // Ay
    [
        &[3, 6, 7, 8, 10, 11],
        &[1, 3, 6, 7, 10, 11],
        &[2, 3, 5, 6, 9, 10, 11],
        &[1, 3, 4, 5, 7, 8, 10, 11],
        &[1, 4, 7, 8, 10, 11, 12],
        &[3, 4, 5, 7, 9, 10, 11],
        &[3, 5, 6, 11],
        &[3, 6, 10, 11],
    ],
    // Mars
    [
        &[3, 5, 6, 10, 11],
        &[3, 6, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 5, 6, 11],
        &[6, 10, 11, 12],
        &[6, 8, 11, 12],
        &[1, 4, 7, 8, 9, 10, 11],
        &[1, 3, 6, 10, 11],
    ],
    // Merkür
    [
        &[5, 6, 9, 11, 12],
        &[2, 4, 6, 8, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[1, 3, 5, 6, 9, 10, 11, 12],
        &[6, 8, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[1, 2, 4, 6, 8, 10, 11],
    ],
    // Jüpiter
    [
        &[1, 2, 3, 4, 7, 8, 9, 10, 11],
        &[2, 5, 7, 9, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[1, 2, 4, 5, 6, 9, 10, 11],
        &[1, 2, 3, 4, 7, 8, 10, 11],
        &[2, 5, 6, 9, 10, 11],
        &[3, 5, 6, 12],
        &[1, 2, 4, 5, 6, 7, 9, 10, 11],
    ],
    // Venüs
    [
        &[8, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 11, 12],
        &[3, 5, 6, 9, 11, 12],
        &[3, 5, 6, 9, 11],
        &[5, 8, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9, 10, 11],
        &[3, 4, 5, 8, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9, 11],
    ],
    // Satürn
    [
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 6, 11],
        &[3, 5, 6, 10, 11, 12],
        &[6, 8, 9, 10, 11, 12],
        &[5, 6, 11, 12],
        &[6, 11, 12],
        &[3, 5, 6, 11],
        &[1, 3, 4, 6, 10, 11],
    ],
];

/// Spec §5 `Ashtakavarga`. Burç sütunları Koç'tan (0) Balık'a (11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Ashtakavarga {
    /// `ASHTAKAVARGA_GRAHAS` sırasında yedi BAV.
    pub bav: [[u8; 12]; 7],
    /// Burç başına yedi BAV'ın toplamı; toplamı her zaman `SAV_TOTAL`.
    pub sav: [u8; 12],
}

/// `signs`: `ASHTAKAVARGA_GRAHAS` sırasında yedi grahanın burcu; `lagna`: lagna burcu.
/// İkisi de aynı (rasi) çerçeveden gelmelidir.
pub fn compute_ashtakavarga(signs: [Sign; 7], lagna: Sign) -> Ashtakavarga {
    let refs = [
        signs[0], signs[1], signs[2], signs[3], signs[4], signs[5], signs[6], lagna,
    ];
    let mut bav = [[0u8; 12]; 7];
    for (row, table) in bav.iter_mut().zip(&BENEFIC_PLACES) {
        for (r, places) in refs.iter().zip(table) {
            for &p in *places {
                // p 1..=12: referans burcunun kendisi 1. evdir.
                row[(r.index() + usize::from(p) + 11) % 12] += 1;
            }
        }
    }
    let mut sav = [0u8; 12];
    for row in &bav {
        for (s, v) in sav.iter_mut().zip(row) {
            *s += v;
        }
    }
    Ashtakavarga { bav, sav }
}
```

`crates/jyotish-core/src/lib.rs` içinde `pub use chart::{…};` satırından **önce** ekle:

```rust
pub use ashtakavarga::{
    compute_ashtakavarga, Ashtakavarga, ASHTAKAVARGA_GRAHAS, BAV_TOTALS, SAV_TOTAL,
};
```

Taşma: bir hücre en çok 8 (sekiz referans), SAV hücresi en çok 7 × 8 = 56; `u8` yeter. Döngüler sabit sınırlıdır (7 × 8 × ≤ 9).

- [ ] **Step 4: Testlerin geçtiğini doğrula**

Run: `cargo test -p jyotish-core --lib ashtakavarga`
Expected: `4 passed` (3 birim, 1 proptest).

Run: `cargo test -p jyotish-core`
Expected: lib `99 passed`, golden `3 passed`.

- [ ] **Step 5: Biçim, lint ve commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
git add crates/jyotish-core/src/ashtakavarga.rs crates/jyotish-core/src/lib.rs crates/jyotish-core/src/proptests.rs
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b' || true
git commit -m "feat(core): ashtakavarga BAV and SAV from BPHS chapter 66 tables" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: clippy temiz, tarama boş.

---

### Task 6: `Chart`'a panchang ve ashtakavarga; `SunriseUnavailable` uyarısı; bağlamalar

**Files:**
- Modify: `crates/jyotish-core/src/chart.rs` (`Chart` alanları, `ChartWarning::SunriseUnavailable`, hesap, testler)
- Modify: `crates/jyotish-core/src/settings.rs` (`SunriseDef` belgesi)
- Modify: `crates/jyotish-core/src/proptests.rs` (gerçekçi hızlar, harita SAV özelliği)
- Modify: `crates/jyotish-core/src/lib.rs` (crate belgesi)
- Modify: `app/ui/bindings.ts` (yeniden üretilir)

**Interfaces:**
- Consumes: `compute_panchang` (Task 4), `compute_ashtakavarga` (Task 5).
- Produces:
  - `Chart.panchang: Panchang`, `Chart.ashtakavarga: Ashtakavarga` (`dasha`'dan sonra).
  - `ChartWarning::SunriseUnavailable`: gün doğumu ya da batımı bulunamadı (kutup gecesi/gündüzü); vara 06:00 yerel ortalama saatten.
  - TS: `Chart.panchang: Panchang`, `Chart.ashtakavarga: Ashtakavarga`, `ChartWarning` birliğine `"SunriseUnavailable"`.

- [ ] **Step 1: Başarısız testleri yaz**

`crates/jyotish-core/src/chart.rs` içindeki `mod tests`'in sonuna ekle:

```rust

    #[test]
    fn panchang_and_ashtakavarga_follow_their_frames() {
        let c = chart(&mut FakeEph::default(), &CalcSettings::default());
        let p = compute_panchang(&mut FakeEph::default(), JD, 41.0, 29.0, &CalcSettings::default())
            .unwrap();
        assert_eq!(c.panchang, p);
        assert_eq!(c.panchang.tithi.no, 8);
        assert_eq!(c.ashtakavarga.sav, [26, 27, 26, 31, 33, 28, 30, 28, 24, 28, 26, 30]);
        assert!(c.meta.warnings.is_empty());

        // Tropikal-rasi kipinde panchang yine sidereal; ashtakavarga tropikal burçlardan.
        // Sahte haritada her burç bir ileri kayar (ayanamsa 24°), SAV bir sütun döner.
        let t = chart(&mut FakeEph::default(), &tropical_rasi());
        assert_eq!(t.panchang, p);
        assert_eq!(t.ashtakavarga.sav, [30, 26, 27, 26, 31, 33, 28, 30, 28, 24, 28, 26]);
    }

    #[test]
    fn missing_sunrise_becomes_a_warning() {
        let mut eph = FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        let c = chart(&mut eph, &CalcSettings::default());
        assert_eq!(c.meta.warnings, [ChartWarning::SunriseUnavailable]);
        assert_eq!(c.panchang.sunrise_jd_ut, None);
        assert_eq!(c.panchang.vara, crate::panchang::Vara::Saturday);
    }
```

Aynı modülün `use` satırlarına ekle (`use crate::varga::Varga;` satırından sonra):

```rust
    use crate::panchang::compute_panchang;
```

`crates/jyotish-core/src/proptests.rs` içindeki `mod chart_properties`'te `const PLANETS: [Body; 7] = [ … ];` tanımından sonra ekle:

```rust
    /// Gerçekçi boylam hızları (°/gün), `PLANETS` sırasında. Panchang bitiş araması Ay'ın
    /// Güneş'ten hızlı olmasını ister; eşit hızda harita `PanchangSearch` döner.
    const SPEEDS: [f64; 7] = [0.98, 13.2, 0.6, 1.2, -0.1, 1.1, 0.05];
```

`whole_sign_house_is_sign_offset_from_lagna` içinde:

```rust
            for (body, lon) in PLANETS.iter().zip(lons) {
                eph = eph.with(*body, lon, 1.0);
            }
```

şununla değiştir:

```rust
            for ((body, lon), speed) in PLANETS.iter().zip(lons).zip(SPEEDS) {
                eph = eph.with(*body, lon, speed);
            }
```

ve aynı `proptest!` bloğunun sonuna (son testin kapanışından sonra, bloğun `}`'sinden önce) ekle:

```rust

        #[test]
        fn chart_ashtakavarga_uses_rasi_signs(
            lons in proptest::array::uniform7(0.0f64..360.0),
            asc in 0.0f64..360.0,
            tropical in any::<bool>(),
        ) {
            let mut eph = FakeEph { asc, ..FakeEph::default() };
            for ((body, lon), speed) in PLANETS.iter().zip(lons).zip(SPEEDS) {
                eph = eph.with(*body, lon, speed);
            }
            let mut s = CalcSettings::default();
            if tropical {
                s.zodiac = ZodiacMode::TropicalRasiSiderealNakshatra;
            }
            let c = compute_chart(&mut eph, 2_451_545.0, 0.0, 0.0, &s).unwrap();
            let signs = std::array::from_fn(|i| c.grahas[i].sign);
            prop_assert_eq!(&c.ashtakavarga, &compute_ashtakavarga(signs, c.lagna.sign));
            prop_assert_eq!(c.ashtakavarga.sav.iter().map(|&v| u16::from(v)).sum::<u16>(), SAV_TOTAL);
        }
```

ve `mod chart_properties`'in `use` satırlarına ekle:

```rust
    use crate::ashtakavarga::{compute_ashtakavarga, SAV_TOTAL};
```

- [ ] **Step 2: Testlerin derlenmediğini doğrula**

Run: `cargo test -p jyotish-core --lib`
Expected: FAIL, derleme hatası: `no field 'panchang' on type 'Chart'`, `no field 'ashtakavarga' on type 'Chart'`, `no variant named 'SunriseUnavailable'`.

- [ ] **Step 3: Haritayı genişlet**

`crates/jyotish-core/src/chart.rs` içinde `use` bloğuna ekle (`use crate::combustion::is_combust;` satırından önce):

```rust
use crate::ashtakavarga::{compute_ashtakavarga, Ashtakavarga};
```

ve `use crate::houses::{…};` satırından sonra:

```rust
use crate::panchang::{compute_panchang, Panchang};
```

`ChartWarning` içinde `PlacidusPolarFallback,` satırından sonra ekle:

```rust
    /// Doğum yerinde gün doğumu ya da batımı yok (kutup gecesi/gündüzü). Panchang'ın
    /// `sunrise_jd_ut`/`sunset_jd_ut`'si boştur; vara gün 06:00 yerel ortalama saatte
    /// başlıyor sayılarak bulunur.
    SunriseUnavailable,
```

`Chart`'ın belge yorumunu:

```rust
/// M3 haritası (Plan 06): vargalar ve Vimshottari dahil. Panchang ve ashtakavarga
/// Plan 07'de eklenir.
```

şununla değiştir:

```rust
/// M3 haritası (spec §5): vargalar, Vimshottari, panchang ve ashtakavarga dahil.
```

`Chart` içinde `pub dasha: Dasha,` satırından sonra ekle:

```rust
    /// Doğum anının panchangı; Güneş ve Ay her kipte sidereal.
    pub panchang: Panchang,
    /// Rasi çerçevesindeki burçlardan (`grahas[..7].sign`, `lagna.sign`).
    pub ashtakavarga: Ashtakavarga,
```

`compute_chart` içinde `let dasha = vimshottari(DashaSeed { … })?;` ifadesinden sonra ekle:

```rust
    let panchang = compute_panchang(eph, jd_ut, geo_lat, geo_lon, settings)?;
    if panchang.sunrise_jd_ut.is_none() || panchang.sunset_jd_ut.is_none() {
        warnings.push(ChartWarning::SunriseUnavailable);
    }
    // grahas her zaman 9 elemandır, ilk yedisi ASHTAKAVARGA_GRAHAS sırasındadır.
    let signs: [Sign; 7] = std::array::from_fn(|i| grahas[i].sign);
    let ashtakavarga = compute_ashtakavarga(signs, lagna.sign);
```

ve `Ok(Chart { … })` içinde `dasha,` satırından sonra ekle:

```rust
        panchang,
        ashtakavarga,
```

`grahas[i]` indeksi `i < 7 ≤ grahas.len() = 9` olduğundan panik yapamaz (`raw` her zaman 7 + 2 eleman iter eder).

`crates/jyotish-core/src/settings.rs` içinde:

```rust
/// Gün doğumu tanımı. Bu planda kullanılmaz (M3 panchang); ayar olarak saklanır.
```

satırını şununla değiştir:

```rust
/// Panchang gün doğumu ve batımı tanımı (spec §6). Gözlemci doğum yerinde, deniz
/// seviyesindedir.
```

`crates/jyotish-core/src/lib.rs` başındaki crate belgesinde:

```rust
//! Kapsam: ayanamsa, grahalar, lagna, bhavalar, nakshatra, dignity, yanma (M2), 16 varga
//! ve Vimshottari dasha (M3, Plan 06). Panchang ve ashtakavarga Plan 07'de eklenir.
```

satırlarını şununla değiştir:

```rust
//! Kapsam: ayanamsa, grahalar, lagna, bhavalar, nakshatra, dignity, yanma (M2), 16 varga,
//! Vimshottari dasha, panchang, ashtakavarga ve transit (M3).
```

- [ ] **Step 4: Bağlamaları yeniden üret ve doğrula**

Run: `cargo test -p jyotish-core`
Expected: lib `102 passed` (99 + 2 birim + 1 proptest), golden `3 passed`. Golden haritaların panchang hesabı gerçek SE ile çalışır; Tromsø için `SunriseUnavailable` uyarısı eklenir, `compare` uyarıları denetlemez.

Run: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings && cargo test --workspace`
Expected: Hepsi geçer. `git diff --stat app/ui/bindings.ts` yalnızca `bindings.ts`'i gösterir.

Run: `grep -n -e 'panchang: Panchang' -e 'ashtakavarga: Ashtakavarga' -e '"SunriseUnavailable"' -e 'export type KaranaName' -e 'export type Vara' app/ui/bindings.ts`
Expected: beş eşleşme. `bav`/`sav` dizilerinin TS biçimi specta'nın sabit dizi eşlemesine bağlıdır (`number[][]`/`number[]` ya da demet); hangisi çıkarsa commit'lenir, arayüz henüz okumaz.

Run: `pnpm --dir app test && pnpm --dir app build`
Expected: geçer (arayüzde `Chart` örneği oluşturan kod yok; yeni alanlar yalnızca eklenir).

- [ ] **Step 5: Biçim, lint ve commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
git add crates/jyotish-core/src app/ui/bindings.ts
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b' || true
git commit -m "feat(core): chart carries panchang and ashtakavarga; sunrise warning" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: clippy temiz, tarama boş.

---
### Task 7: `compute_transits` ve `chart.rs`'in ortak ham konum kodu

**Files:**
- Create: `crates/jyotish-core/src/transit.rs`
- Modify: `crates/jyotish-core/src/chart.rs` (`Raw`, `rasi_frame`, `to_sidereal`, `raw_grahas`, `finite` crate içine açılır; `compute_chart` bunları kullanır)
- Modify: `crates/jyotish-core/src/error.rs` (`InvalidTransitInput`)
- Modify: `crates/jyotish-core/src/lib.rs`
- Modify: `app/src-tauri/src/commands.rs` (hata eşlemesi ve testi)

**Interfaces:**
- Consumes: `check_moment`'in aralığı (`JD_UT_RANGE`), `house_from_cusps`, `Bhavas`, `ChartWarning`.
- Produces:
  - `pub fn compute_transits<E: Ephemeris + ?Sized>(eph: &mut E, jd_ut: f64, settings: &CalcSettings, natal_bhavas: &Bhavas) -> Result<TransitSnapshot, CoreError>`.
  - `pub struct TransitGraha { graha, lon, sidereal_lon, speed, retrograde, sign, deg_in_sign, nakshatra, pada, natal_house: u8 }`.
  - `pub struct TransitSnapshot { jd_ut: f64, ayanamsa_deg: f64, grahas: Vec<TransitGraha>, warnings: Vec<ChartWarning> }`.
  - `CoreError::InvalidTransitInput(&'static str)` (`"natal_bhavas"`: 12 sonlu başlangıç değil); IPC kodu `"invalid_transit_input"`, parametre `field`.
  - `crate::chart` içinde (crate'e açık): `struct Raw`, `fn rasi_frame(&CalcSettings) -> Zodiac`, `fn to_sidereal(ZodiacMode, f64, f64) -> f64`, `fn raw_grahas(eph, jd_ut, Zodiac, NodeKind) -> Result<(Vec<Raw>, bool), CoreError>`, `fn finite`.

`compute_transits` yalnızca yeni çağıran ekler; `compute_chart`'ın davranışı değişmez (Task 6'nın tüm `chart.rs` testleri değişmeden geçer).

- [ ] **Step 1: Başarısız testleri yaz**

`crates/jyotish-core/src/transit.rs` dosyasını yalnızca testlerle oluştur:

```rust
#[cfg(test)]
mod tests {
    use ephemeris::Precision;

    use super::*;
    use crate::chart::compute_chart;
    use crate::fake::FakeEph;
    use crate::settings::{HouseSystem, ZodiacMode};
    use crate::zodiac::normalize;

    const JD: f64 = 2_451_545.0;

    fn natal(s: &CalcSettings) -> crate::chart::Chart {
        compute_chart(&mut FakeEph::default(), JD, 41.0, 29.0, s).unwrap()
    }

    #[test]
    fn transit_at_birth_matches_the_chart() {
        for zodiac in [
            ZodiacMode::Sidereal,
            ZodiacMode::TropicalRasiSiderealNakshatra,
        ] {
            for house_system in [HouseSystem::WholeSign, HouseSystem::Placidus] {
                let s = CalcSettings {
                    zodiac,
                    house_system,
                    ..CalcSettings::default()
                };
                let c = natal(&s);
                let t = compute_transits(&mut FakeEph::default(), JD, &s, &c.bhavas).unwrap();
                assert_eq!((t.jd_ut, t.ayanamsa_deg), (JD, c.ayanamsa_deg));
                assert!(t.warnings.is_empty());
                assert_eq!(t.grahas.len(), 9);
                for (tg, cg) in t.grahas.iter().zip(&c.grahas) {
                    let ctx = format!("{:?} {zodiac:?} {house_system:?}", tg.graha);
                    assert_eq!(tg.graha, cg.graha, "{ctx}");
                    assert_eq!((tg.lon, tg.sidereal_lon), (cg.lon, cg.sidereal_lon), "{ctx}");
                    assert_eq!((tg.speed, tg.retrograde), (cg.speed, cg.retrograde), "{ctx}");
                    assert_eq!((tg.sign, tg.deg_in_sign), (cg.sign, cg.deg_in_sign), "{ctx}");
                    assert_eq!((tg.nakshatra, tg.pada), (cg.nakshatra, cg.pada), "{ctx}");
                    assert_eq!(tg.natal_house, cg.house, "{ctx}");
                }
            }
        }
    }

    #[test]
    fn transits_move_and_fall_in_natal_houses() {
        let s = CalcSettings::default();
        let c = natal(&s);
        // 10 gün sonra (sahte hareket doğrusal): Ay sidereal 71 + 132 = 203° (Terazi),
        // Güneş 346 + 9.8 = 355.8° (Balık). Natal lagna Boğa.
        let t = compute_transits(&mut FakeEph::default(), JD + 10.0, &s, &c.bhavas).unwrap();
        let (sun, moon) = (&t.grahas[0], &t.grahas[1]);
        assert!((moon.lon - 203.0).abs() < 1e-9, "{}", moon.lon);
        assert_eq!((moon.sign, moon.natal_house), (Sign::Libra, 6));
        assert!((sun.lon - 355.8).abs() < 1e-9, "{}", sun.lon);
        assert_eq!((sun.sign, sun.natal_house), (Sign::Pisces, 11));
        assert_eq!(t.grahas[8].lon, normalize(t.grahas[7].lon + 180.0));

        let mut eph = FakeEph {
            precision: Precision::Moshier,
            ..FakeEph::default()
        };
        let t = compute_transits(&mut eph, JD, &s, &c.bhavas).unwrap();
        assert_eq!(t.warnings, [ChartWarning::LowPrecision]);
    }

    #[test]
    fn invalid_input_is_rejected() {
        let s = CalcSettings::default();
        let c = natal(&s);
        let mut eph = FakeEph::default();
        let bad_bhavas = CoreError::InvalidTransitInput("natal_bhavas");
        let short = Bhavas {
            cusps: c.bhavas.cusps[..11].to_vec(),
            ..c.bhavas.clone()
        };
        assert_eq!(
            compute_transits(&mut eph, JD, &s, &short),
            Err(bad_bhavas.clone())
        );
        let mut nan = c.bhavas.clone();
        nan.cusps[3] = f64::NAN;
        assert_eq!(compute_transits(&mut eph, JD, &s, &nan), Err(bad_bhavas));
        assert_eq!(
            compute_transits(&mut eph, 1e300, &s, &c.bhavas),
            Err(CoreError::InvalidJulianDay(1e300))
        );
        let old = CalcSettings {
            schema_version: 2,
            ..CalcSettings::default()
        };
        assert_eq!(
            compute_transits(&mut eph, JD, &old, &c.bhavas),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 2,
                supported: 1
            })
        );
    }
}
```

`crates/jyotish-core/src/lib.rs` içinde `pub mod settings;` satırından sonra ekle:

```rust
pub mod transit;
```

`app/src-tauri/src/commands.rs` içindeki `core_error_codes_are_stable` testinin sonuna ekle:

```rust
        let e = CommandError::from(CoreError::InvalidTransitInput("natal_bhavas"));
        assert_eq!(e.code, "invalid_transit_input");
        assert_eq!(e.params["field"], "natal_bhavas");
```

- [ ] **Step 2: Testlerin derlenmediğini doğrula**

Run: `cargo test -p jyotish-core --lib transit`
Expected: FAIL, derleme hatası: `cannot find function 'compute_transits' in this scope`, `no variant named 'InvalidTransitInput'`.

- [ ] **Step 3: Ortak kodu ayır ve transiti yaz**

`crates/jyotish-core/src/error.rs` içinde `PanchangSearch` varyantından sonra ekle:

```rust
    #[error("invalid transit input: {0}")]
    InvalidTransitInput(&'static str),
```

`app/src-tauri/src/commands.rs` içinde `CoreError::PanchangSearch(what) => { … }` kolundan sonra ekle:

```rust
            CoreError::InvalidTransitInput(field) => {
                Self::new("invalid_transit_input", &[("field", field.to_owned())])
            }
```

`crates/jyotish-core/src/chart.rs` içinde `use crate::settings::{CalcSettings, HouseSystem, ZodiacMode, JD_UT_RANGE};` satırını şununla değiştir:

```rust
use crate::settings::{CalcSettings, HouseSystem, NodeKind, ZodiacMode, JD_UT_RANGE};
```

`fn finite(` satırını `pub(crate) fn finite(` yap.

Şu bloğu (Raw yapısı ve `compute_chart`'ın tamamı, belge yorumuyla):

```rust
/// Rasi çerçevesinde ham konum.
struct Raw {
```

ile başlayıp `compute_chart`'ın kapanış `}`'siyle (`#[cfg(test)]`'ten önceki son satır) biten bölümü şununla değiştir:

```rust
/// Rasi çerçevesinde ham konum.
pub(crate) struct Raw {
    pub graha: Graha,
    pub lon: f64,
    pub lat: f64,
    pub speed: f64,
}

/// Ephemeris'in sorulacağı rasi çerçevesi: `Sidereal` kipinde sidereal, tropikal-rasi
/// kipinde tropikal.
pub(crate) fn rasi_frame(settings: &CalcSettings) -> Zodiac {
    match settings.zodiac {
        ZodiacMode::Sidereal => Zodiac::Sidereal(settings.ayanamsa.sidereal_mode()),
        ZodiacMode::TropicalRasiSiderealNakshatra => Zodiac::Tropical,
    }
}

/// Rasi çerçevesindeki boylamın sidereal karşılığı. SE'nin sidereal hesabı da aynı
/// çıkarmayı yapar.
pub(crate) fn to_sidereal(zodiac: ZodiacMode, frame_lon: f64, ayanamsa_deg: f64) -> f64 {
    match zodiac {
        ZodiacMode::Sidereal => frame_lon,
        ZodiacMode::TropicalRasiSiderealNakshatra => normalize(frame_lon - ayanamsa_deg),
    }
}

/// Dokuz grahanın rasi çerçevesindeki ham konumu (`Graha::ALL` sırasında) ve Güneş ya da
/// Ay'ın Moshier yedeğiyle hesaplanıp hesaplanmadığı. `compute_chart` ve `compute_transits`
/// ortak kullanır.
pub(crate) fn raw_grahas<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    frame: Zodiac,
    node: NodeKind,
) -> Result<(Vec<Raw>, bool), CoreError> {
    let mut low_precision = false;
    let mut raw = Vec::with_capacity(9);
    for (graha, body) in PLANETS {
        let p = eph.body(jd_ut, body, frame)?;
        finite_body(&p)?;
        // Hassasiyet Güneş/Ay'dan alınır (Plan 01 notu: ayanamsa bayrağı güvenilmez).
        if matches!(graha, Graha::Sun | Graha::Moon) && p.precision == Precision::Moshier {
            low_precision = true;
        }
        raw.push(Raw {
            graha,
            lon: p.lon,
            lat: p.lat,
            speed: p.lon_speed,
        });
    }
    let rahu = eph.body(jd_ut, node.body(), frame)?;
    finite_body(&rahu)?;
    raw.push(Raw {
        graha: Graha::Rahu,
        lon: rahu.lon,
        lat: rahu.lat,
        speed: rahu.lon_speed,
    });
    // Ketu tanım gereği Rahu'nun tam karşısıdır; hareketi Rahu ile aynıdır.
    raw.push(Raw {
        graha: Graha::Ketu,
        lon: normalize(rahu.lon + 180.0),
        lat: -rahu.lat,
        speed: rahu.lon_speed,
    });
    Ok((raw, low_precision))
}

/// `jd_ut` anı ve (`geo_lat` kuzey +, `geo_lon` doğu +) konumu için haritayı hesaplar.
///
/// Rasi çerçevesi `settings.zodiac`'a göre seçilir: `Sidereal` kipinde ephemeris sidereal
/// sorulur, `TropicalRasiSiderealNakshatra` kipinde tropikal sorulur ve sidereal boylam
/// `tropikal − ayanamsa` ile bulunur.
pub fn compute_chart<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    geo_lat: f64,
    geo_lon: f64,
    settings: &CalcSettings,
) -> Result<Chart, CoreError> {
    settings.validate()?;
    check_moment(jd_ut, geo_lat, geo_lon)?;

    let ayanamsa = eph.ayanamsa(jd_ut, settings.ayanamsa.sidereal_mode())?;
    finite(ayanamsa.deg, "ayanamsa")?;
    let frame = rasi_frame(settings);
    let sidereal = |frame_lon: f64| to_sidereal(settings.zodiac, frame_lon, ayanamsa.deg);

    let mut warnings = Vec::new();
    let (raw, low_precision) = raw_grahas(eph, jd_ut, frame, settings.node)?;
    if low_precision {
        warnings.push(ChartWarning::LowPrecision);
    }

    let houses = eph.houses(
        jd_ut,
        geo_lat,
        geo_lon,
        settings.house_system.ephemeris_system(),
        frame,
    )?;
    finite(houses.asc, "lagna")?;
    for c in houses.cusps {
        finite(c, "bhavas")?;
    }
    if houses.porphyry_fallback {
        warnings.push(ChartWarning::PlacidusPolarFallback);
    }
    let lagna = Lagna::new(houses.asc, sidereal(houses.asc));
    let cusps = match settings.house_system {
        HouseSystem::WholeSign => whole_sign_cusps(lagna.sign),
        HouseSystem::Sripati | HouseSystem::Placidus => houses.cusps.to_vec(),
    };

    let sun_lon = raw[0].lon;
    // Burç lordları her zaman ilk 7 grahadan biridir; bulunamaması mümkün değildir.
    let sign_of = |g: Graha| raw.iter().find(|r| r.graha == g).map(|r| Sign::of(r.lon));
    let grahas: Vec<GrahaPosition> = raw
        .iter()
        .map(|r| {
            let sign = Sign::of(r.lon);
            let deg = deg_in_sign(r.lon);
            let sidereal_lon = sidereal(r.lon);
            let (nakshatra, pada) = nakshatra_pada(sidereal_lon);
            let retrograde = r.speed < 0.0;
            let lord_sign = sign_of(sign_lord(sign)).unwrap_or(sign);
            GrahaPosition {
                graha: r.graha,
                lon: r.lon,
                sidereal_lon,
                lat: r.lat,
                speed: r.speed,
                retrograde,
                sign,
                deg_in_sign: deg,
                nakshatra,
                pada,
                combust: is_combust(
                    r.graha,
                    r.lon,
                    sun_lon,
                    retrograde,
                    &settings.combustion_orbs,
                ),
                dignity: dignity(r.graha, sign, deg, lord_sign),
                house: match settings.house_system {
                    HouseSystem::WholeSign => whole_sign_house(sign, lagna.sign),
                    HouseSystem::Sripati | HouseSystem::Placidus => house_from_cusps(r.lon, &cusps),
                },
                vargottama: is_vargottama(r.lon),
            }
        })
        .collect();

    let points: Vec<(Graha, f64)> = raw.iter().map(|r| (r.graha, r.lon)).collect();
    let vargas = all_vargas(lagna.lon, &points);
    // raw[1] her zaman Ay'dır (PLANETS sırası); nakshatra her kipte sidereal'dir.
    let dasha = vimshottari(DashaSeed {
        birth_jd_ut: jd_ut,
        moon_sidereal_lon: sidereal(raw[1].lon),
        year_days: settings.dasha_year,
    })?;
    let panchang = compute_panchang(eph, jd_ut, geo_lat, geo_lon, settings)?;
    if panchang.sunrise_jd_ut.is_none() || panchang.sunset_jd_ut.is_none() {
        warnings.push(ChartWarning::SunriseUnavailable);
    }
    // grahas her zaman 9 elemandır, ilk yedisi ASHTAKAVARGA_GRAHAS sırasındadır.
    let signs: [Sign; 7] = std::array::from_fn(|i| grahas[i].sign);
    let ashtakavarga = compute_ashtakavarga(signs, lagna.sign);
    Ok(Chart {
        meta: ChartMeta {
            engine_version: ENGINE_VERSION.to_owned(),
            swe_version: eph.swe_version(),
            settings: settings.clone(),
            warnings,
        },
        jd_ut,
        geo_lat,
        geo_lon,
        ayanamsa_deg: ayanamsa.deg,
        lagna,
        grahas,
        bhavas: Bhavas {
            system: settings.house_system,
            cusps,
        },
        vargas,
        dasha,
        panchang,
        ashtakavarga,
    })
}
```

Davranış aynıdır: `LowPrecision` eskiden de en fazla bir kez ve `PlacidusPolarFallback`'ten önce eklenirdi; `to_sidereal` aynı formüldür.

`crates/jyotish-core/src/transit.rs` dosyasının **başına** (testlerin üstüne) ekle:

```rust
//! Transit: verilen andaki dokuz grahanın konumu ve natal bhavalardaki yeri (spec §3, §4).
//!
//! v1 kapsamı (UI spec §3.3): transit halkası ve tablosu. Lagna, dignity, yanma, vedha
//! yoktur. "Şimdi" anını çekirdek bilmez; `jd_ut`'yi arayüz verir.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{finite, raw_grahas, rasi_frame, to_sidereal, ChartWarning};
use crate::graha::Graha;
use crate::houses::{house_from_cusps, Bhavas};
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::zodiac::{deg_in_sign, nakshatra_pada, Sign};
use crate::CoreError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TransitGraha {
    pub graha: Graha,
    /// Rasi çerçevesinde boylam (natal haritayla aynı ayarlar verilmelidir).
    pub lon: f64,
    pub sidereal_lon: f64,
    pub speed: f64,
    pub retrograde: bool,
    pub sign: Sign,
    pub deg_in_sign: f64,
    pub nakshatra: u8,
    pub pada: u8,
    /// Natal bhava başlangıçlarına göre ev, 1..=12.
    pub natal_house: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TransitSnapshot {
    pub jd_ut: f64,
    pub ayanamsa_deg: f64,
    /// Her zaman 9 eleman, `Graha::ALL` sırasında.
    pub grahas: Vec<TransitGraha>,
    /// Yalnızca `LowPrecision` olabilir.
    pub warnings: Vec<ChartWarning>,
}

/// `jd_ut` anının transit konumları. `settings` ve `natal_bhavas` natal haritadan gelir
/// (`chart.meta.settings`, `chart.bhavas`); evler natal bhava başlangıçlarına göre bulunur
/// (Whole Sign başlangıçları burç başlarıdır, sonuç burç sayımıyla aynıdır).
pub fn compute_transits<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    settings: &CalcSettings,
    natal_bhavas: &Bhavas,
) -> Result<TransitSnapshot, CoreError> {
    settings.validate()?;
    // NaN ve sonsuz da aralık dışındadır.
    if !JD_UT_RANGE.contains(&jd_ut) {
        return Err(CoreError::InvalidJulianDay(jd_ut));
    }
    if natal_bhavas.cusps.len() != 12 || !natal_bhavas.cusps.iter().all(|c| c.is_finite()) {
        return Err(CoreError::InvalidTransitInput("natal_bhavas"));
    }
    let ayanamsa = eph.ayanamsa(jd_ut, settings.ayanamsa.sidereal_mode())?;
    finite(ayanamsa.deg, "ayanamsa")?;
    let (raw, low_precision) = raw_grahas(eph, jd_ut, rasi_frame(settings), settings.node)?;
    let grahas = raw
        .iter()
        .map(|r| {
            let sidereal_lon = to_sidereal(settings.zodiac, r.lon, ayanamsa.deg);
            let (nakshatra, pada) = nakshatra_pada(sidereal_lon);
            TransitGraha {
                graha: r.graha,
                lon: r.lon,
                sidereal_lon,
                speed: r.speed,
                retrograde: r.speed < 0.0,
                sign: Sign::of(r.lon),
                deg_in_sign: deg_in_sign(r.lon),
                nakshatra,
                pada,
                natal_house: house_from_cusps(r.lon, &natal_bhavas.cusps),
            }
        })
        .collect();
    let warnings = if low_precision {
        vec![ChartWarning::LowPrecision]
    } else {
        Vec::new()
    };
    Ok(TransitSnapshot {
        jd_ut,
        ayanamsa_deg: ayanamsa.deg,
        grahas,
        warnings,
    })
}
```

`crates/jyotish-core/src/lib.rs` içinde `pub use settings::{ … };` bloğundan sonra ekle:

```rust
pub use transit::{compute_transits, TransitGraha, TransitSnapshot};
```

- [ ] **Step 4: Testlerin geçtiğini doğrula**

Run: `cargo test -p jyotish-core --lib transit`
Expected: `3 passed`.

Run: `cargo test --workspace`
Expected: Hepsi geçer: `jyotish-core` lib `105 passed`, golden 3, `jyotish-desk` lib 8, `bindings` 1 (`compute_transits` henüz komut değil; IPC tipleri değişmedi).

- [ ] **Step 5: Biçim, lint ve commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p ephemeris -p atlas -p jyotish-core --all-targets
git add crates/jyotish-core/src app/src-tauri/src/commands.rs
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b' || true
git commit -m "feat(core): compute_transits over natal bhavas with shared raw positions" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: clippy temiz, MSRV `cargo check` geçer, tarama boş.

---

### Task 8: Golden fixture'lar: panchang ve SAV beklentileri

**Files:**
- Modify: `scripts/gen-golden-swetest.sh` (panchang ve SAV türetmesi)
- Modify: `crates/jyotish-core/tests/golden.rs` (`ExpectedPanchang`, `sav`, karşılaştırma, negatif test)
- Modify: `tests/golden/README.md` (şema, toleranslar)
- Modify: `tests/golden/*.json` (5 dosya, betikle yeniden üretilir)

**Interfaces:**
- Consumes: `Chart.panchang`, `Chart.ashtakavarga` (Task 6), `Vara` (Task 3).
- Produces: fixture şemasında isteğe bağlı `expected.panchang { sunrise_jd, sunset_jd (null olabilir), vara, tithi, tithi_ends_jd, nakshatra, nakshatra_ends_jd, yoga, yoga_ends_jd, karana, karana_ends_jd }` ve `expected.sav: [u8; 12]`. `PANCHANG_TOL_SECONDS = 60.0`, `MIN_PANCHANG_FIXTURES = 5`.

Betik bu alanları Rust motorundan bağımsız üretir: gün doğumu/batımı `swetest -rise -hindu`'dan (Meeus tarih → JD), tithi/nakshatra/yoga/karana `swetest -sid1` Güneş/Ay boylamlarından awk ile, bitişler aynı boylamlar üzerinde 24 adım ikiye bölmeyle (≈ 0.008 sn), SAV ise Rust tablosundan ayrı yazılmış bir awk tablosuyla. Aynı SE kütüphanesi kullanıldığı için bu bir **tesisat** kanıtıdır (`README.md` "Kaynaklar"); bağımsız referans JHora setidir.

- [ ] **Step 1: Başarısız karşılaştırıcıyı yaz**

`crates/jyotish-core/tests/golden.rs` içinde:

```rust
use jyotish_core::{
    angular_distance, compute_chart, CalcSettings, Chart, ChartWarning, Graha, Sign, Varga,
};
```

şununla değiştir:

```rust
use jyotish_core::{
    angular_distance, compute_chart, CalcSettings, Chart, ChartWarning, Graha, Sign, Vara,
    Varga,
};
```

`const MIN_DERIVED_FIXTURES: usize = 5;` satırından sonra ekle:

```rust
/// Panchang bitiş zamanları ve gün doğumu/batımı toleransı (spec §8).
const PANCHANG_TOL_SECONDS: f64 = 60.0;
/// `panchang` ve `sav` taşıyan fixture sayısı bunun altına düşerse test düşer.
const MIN_PANCHANG_FIXTURES: usize = 5;
```

`struct Expected` içinde `dasha: Option<ExpectedDasha>,` satırından sonra ekle:

```rust
    /// İsteğe bağlı: doğum anının panchangı (Plan 07). Zamanlar UT JD; gün doğumu yoksa null.
    #[serde(default)]
    panchang: Option<ExpectedPanchang>,
    /// İsteğe bağlı: Koç'tan Balık'a sarvashtakavarga.
    #[serde(default)]
    sav: Option<[u8; 12]>,
```

`struct ExpectedDasha { … }` tanımından sonra ekle:

```rust

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedPanchang {
    sunrise_jd: Option<f64>,
    sunset_jd: Option<f64>,
    vara: Vara,
    tithi: u8,
    tithi_ends_jd: f64,
    nakshatra: u8,
    nakshatra_ends_jd: f64,
    yoga: u8,
    yoga_ends_jd: f64,
    karana: u8,
    karana_ends_jd: f64,
}
```

`compare_derived` fonksiyonundan sonra ekle:

```rust

/// İki zaman (UT JD) tolerans dışındaysa, biri yoksa ya da NaN ise açıklama döner.
fn time_mismatch(what: &str, got: Option<f64>, want: Option<f64>) -> Option<String> {
    match (got, want) {
        (None, None) => None,
        (Some(g), Some(w)) => {
            let diff = (g - w).abs() * 86_400.0;
            if diff.is_nan() || diff > PANCHANG_TOL_SECONDS {
                Some(format!(
                    "{what}: got {g:.6}, want {w:.6}, diff {diff:.1} s > {PANCHANG_TOL_SECONDS} s"
                ))
            } else {
                None
            }
        }
        _ => Some(format!("{what}: got {got:?}, want {want:?}")),
    }
}

/// Plan 07 beklentileri: panchang numaraları ve vara birebir, zamanlar ≤ 60 sn; SAV birebir.
fn compare_panchang(f: &Fixture, chart: &Chart) -> Vec<String> {
    let mut errs = Vec::new();
    if let Some(want) = &f.expected.panchang {
        let p = &chart.panchang;
        errs.extend(time_mismatch("sunrise", p.sunrise_jd_ut, want.sunrise_jd));
        errs.extend(time_mismatch("sunset", p.sunset_jd_ut, want.sunset_jd));
        if p.vara != want.vara {
            errs.push(format!("vara: got {:?}, want {:?}", p.vara, want.vara));
        }
        let elements = [
            ("tithi", p.tithi.no, want.tithi, p.tithi.ends_jd_ut, want.tithi_ends_jd),
            (
                "nakshatra",
                p.nakshatra.no,
                want.nakshatra,
                p.nakshatra.ends_jd_ut,
                want.nakshatra_ends_jd,
            ),
            ("yoga", p.yoga.no, want.yoga, p.yoga.ends_jd_ut, want.yoga_ends_jd),
            ("karana", p.karana.no, want.karana, p.karana.ends_jd_ut, want.karana_ends_jd),
        ];
        for (what, got_no, want_no, got_end, want_end) in elements {
            if got_no != want_no {
                errs.push(format!("{what}: got {got_no}, want {want_no}"));
            }
            errs.extend(time_mismatch(
                &format!("{what} end"),
                Some(got_end),
                Some(want_end),
            ));
        }
    }
    if let Some(want) = &f.expected.sav {
        if &chart.ashtakavarga.sav != want {
            errs.push(format!(
                "sav: got {:?}, want {want:?}",
                chart.ashtakavarga.sav
            ));
        }
    }
    errs
}

#[test]
fn time_tolerance_flags_out_of_range_nan_and_missing() {
    let second = 1.0 / 86_400.0;
    let jd = 2_451_545.0;
    assert!(time_mismatch("t", Some(jd), Some(jd + 59.0 * second)).is_none());
    assert!(time_mismatch("t", Some(jd), Some(jd + 61.0 * second)).is_some());
    assert!(time_mismatch("t", Some(f64::NAN), Some(jd)).is_some());
    assert!(time_mismatch("t", None, None).is_none());
    assert!(time_mismatch("t", None, Some(jd)).is_some());
    assert!(time_mismatch("t", Some(jd), None).is_some());
}
```

`golden_charts_match_references` içinde:

```rust
    let mut derived = 0;
    let mut sample: Option<(Fixture, Chart)> = None;
```

şununla değiştir:

```rust
    let mut derived = 0;
    let mut sample: Option<(Fixture, Chart)> = None;
    let mut with_panchang = 0;
    let mut panchang_sample: Option<(Fixture, Chart)> = None;
```

döngüdeki:

```rust
        for e in compare(f, &chart)
            .into_iter()
            .chain(compare_derived(f, &chart))
        {
```

şununla değiştir:

```rust
        for e in compare(f, &chart)
            .into_iter()
            .chain(compare_derived(f, &chart))
            .chain(compare_panchang(f, &chart))
        {
```

döngüde `if f.expected.d9_signs.is_some() && f.expected.dasha.is_some() { … }` bloğunun **önüne** (döngü içinde; o blok `sample = Some((f.clone(), chart));` ile `chart`'ı taşıdığı için yeni blok ondan önce gelir) ekle:

```rust
        if let (Some(p), Some(_)) = (&f.expected.panchang, &f.expected.sav) {
            with_panchang += 1;
            if panchang_sample.is_none() && p.sunrise_jd.is_some() {
                panchang_sample = Some((f.clone(), chart.clone()));
            }
        }
```

Fonksiyonun sonuna (son `assert_eq!(errs.len(), 3, "{errs:?}");` satırından sonra) ekle:

```rust
    assert!(
        with_panchang >= MIN_PANCHANG_FIXTURES,
        "found {with_panchang} fixtures with panchang and sav, want >= {MIN_PANCHANG_FIXTURES}"
    );

    // Panchang karşılaştırıcısı da ısırır: tithi, gün doğumu ve SAV'ı boz, üç hata bekle.
    let (mut bad, chart) = panchang_sample.expect("a fixture with panchang and a sunrise");
    if let Some(p) = bad.expected.panchang.as_mut() {
        p.tithi = p.tithi % 30 + 1;
        p.sunrise_jd = p.sunrise_jd.map(|t| t + 0.01); // 864 sn
    }
    if let Some(s) = bad.expected.sav.as_mut() {
        s[0] += 1;
    }
    let errs = compare_panchang(&bad, &chart);
    assert_eq!(errs.len(), 3, "{errs:?}");
```

Aynı dosyaya (`crates/jyotish-core/tests/golden.rs`, dosyanın sonuna) şu testi de ekle (gerçek SE ile; yukarıdaki değişiklik olmadan `sunrise` `None` olur ve test düşer):

```rust
#[test]
fn first_sunrise_after_the_midnight_sun_is_found() {
    let mut eph = SwissEph::open(&ephe_dir()).expect("open ephemeris");
    // 67.179 K, 90.14 B; 5 Temmuz 2000 07:36 UT (yerel ortalama 01:36, Çarşamba). Gece yarısı
    // güneşinden sonraki ilk doğuş doğumdan 1.2 saat önce (yerel 00:24); bir önceki doğuş
    // haftalar öncedir. SE'nin yüksek enlem araması başlangıçtan ~26 saat ileriye bakar.
    let p = jyotish_core::compute_panchang(
        &mut eph,
        2_451_730.817,
        67.179,
        -90.14,
        &CalcSettings::default(),
    )
    .expect("panchang");
    let rise = p.sunrise_jd_ut.expect("sunrise");
    assert!(
        (rise - 2_451_730.766_847).abs() * 86_400.0 <= 60.0,
        "{rise}"
    );
    assert_eq!(p.vara, Vara::Wednesday);
}
```

- [ ] **Step 2: Testin düştüğünü doğrula**

Run: `cargo test -p jyotish-core --test golden`
Expected: FAIL: `golden_charts_match_references` panikler: `found 0 fixtures with panchang and sav, want >= 5`. Diğer dört test geçer (`time_tolerance_flags_out_of_range_nan_and_missing` dahil).

- [ ] **Step 3: Üretici betiği genişlet**

`scripts/gen-golden-swetest.sh` içinde `check() { … }` satırından sonra şu bloğu ekle:

```bash
# --- Plan 07: panchang ve SAV (Rust motorundan bağımsız; swetest + awk) ---
# Lahiri sidereal Güneş ve Ay boylamı, tek satır: "güneş ay".
sun_moon() {
  "$BIN" -bj"$1" -ut -p01 -fl -head -eswe -edir"$EPHE" -sid1 |
    awk '{ v[NR] = $1 } END { printf "%s %s", v[1], v[2] }'
}
# Panchang açısı, [0, 360): tithi/karana Ay − Güneş, nakshatra Ay, yoga Güneş + Ay.
angle() {
  sun_moon "$1" | awk -v q="$2" '{
    if (q == "elong") a = $2 - $1; else if (q == "moon") a = $2; else a = $1 + $2;
    a -= 360 * int(a / 360); if (a < 0) a += 360; printf "%.9f", a }'
}
# Açının `jd`'den sonra `target` dereceye ulaştığı an: [jd, jd + 1.5] aralığında 24 adım
# ikiye bölme (≈ 0.008 sn). Hedefe işaretli uzaklık [-180, 180) aralığına sarılır.
ends_at() {
  local jd="$1" q="$2" target="$3" lo hi mid up
  lo="$jd"; hi="$(awk -v j="$jd" 'BEGIN { printf "%.9f", j + 1.5 }')"
  for _ in $(seq 1 24); do
    mid="$(awk -v a="$lo" -v b="$hi" 'BEGIN { printf "%.9f", (a + b) / 2 }')"
    up="$(awk -v a="$(angle "$mid" "$q")" -v t="$target" 'BEGIN {
      d = a - t + 180; d -= 360 * int(d / 360); if (d < 0) d += 360; print (d - 180 >= 0) }')"
    if [ "$up" = 1 ]; then hi="$mid"; else lo="$mid"; fi
  done
  awk -v a="$lo" -v b="$hi" 'BEGIN { printf "%.6f", (a + b) / 2 }'
}
# `swetest -rise -hindu` satırlarından "rise JD" / "set JD" (UT; Meeus tarih → JD).
rise_set_jds() {
  "$BIN" -rise -hindu -bj"$1" -ut -p0 -geopos"$2","$3",0 -n4 -head -eswe -edir"$EPHE" | awk '
    function jd(d, t,   a, b, D, M, Y, h, A, B) {
      split(d, a, "."); split(t, b, ":"); D = a[1]; M = a[2]; Y = a[3];
      h = b[1] + b[2] / 60.0 + b[3] / 3600.0;
      if (M <= 2) { Y -= 1; M += 12 }
      A = int(Y / 100); B = 2 - A + int(A / 4);
      return int(365.25 * (Y + 4716)) + int(30.6001 * (M + 1)) + D + B - 1524.5 + h / 24.0 }
    $1 == "rise" {
      if ($2 != "-") printf "rise %.6f\n", jd($2, $3);
      for (i = 3; i < NF; i++) if ($i == "set" && $(i + 1) != "-") printf "set %.6f\n", jd($(i + 1), $(i + 2)) }'
}
# Doğum anının panchangı (Plan 07 tanımları): gün doğumu doğumdan önceki 1.25 gündeki son
# doğuş, batış ondan sonraki ilk batış (1 gün içinde). Vara gün doğumunun yerel ortalama
# tarihinden; gün doğumu yoksa 06:00 yerel ortalama saatten.
panchang_json() {
  local jd="$1" lat="$2" lon="$3" ev rise set vara sm t n y k
  ev="$(rise_set_jds "$(awk -v j="$jd" 'BEGIN { printf "%.6f", j - 1.25 }')" "$lon" "$lat")"
  rise="$(echo "$ev" | awk -v j="$jd" '$1 == "rise" && $2 <= j { r = $2 } END { print (r == "" ? "null" : r) }')"
  set="null"
  if [ "$rise" != null ]; then
    set="$(echo "$ev" | awk -v r="$rise" '$1 == "set" && $2 > r && $2 - r < 1 { print $2; exit }')"
    [ -z "$set" ] && set=null
  fi
  vara="$(awk -v r="$rise" -v j="$jd" -v l="$lon" 'BEGIN {
    split("Sunday Monday Tuesday Wednesday Thursday Friday Saturday", W, " ");
    x = (r == "null") ? j - 0.25 : r; x += 0.5 + l / 360;
    print W[(int(x) + 1) % 7 + 1] }')"
  sm="$(sun_moon "$jd")"
  read -r t n y k <<< "$(echo "$sm" | awk '{
    e = $2 - $1; e -= 360 * int(e / 360); if (e < 0) e += 360;
    s = $1 + $2; s -= 360 * int(s / 360);
    printf "%d %d %d %d", int(e / 12) + 1, int($2 * 3 / 40) + 1, int(s * 3 / 40) + 1, int(e / 6) + 1 }')"
  printf '{\n'
  printf '      "sunrise_jd": %s,\n      "sunset_jd": %s,\n      "vara": "%s",\n' "$rise" "$set" "$vara"
  printf '      "tithi": %s,\n      "tithi_ends_jd": %s,\n' "$t" "$(ends_at "$jd" elong "$((t * 12))")"
  printf '      "nakshatra": %s,\n      "nakshatra_ends_jd": %s,\n' "$n" \
    "$(ends_at "$jd" moon "$(awk -v n="$n" 'BEGIN { printf "%.12f", n * 40 / 3 }')")"
  printf '      "yoga": %s,\n      "yoga_ends_jd": %s,\n' "$y" \
    "$(ends_at "$jd" sum "$(awk -v n="$y" 'BEGIN { printf "%.12f", n * 40 / 3 }')")"
  printf '      "karana": %s,\n      "karana_ends_jd": %s\n    }' "$k" "$(ends_at "$jd" elong "$((k * 6))")"
}
# Sarvashtakavarga, Koç'tan 12 burç. Girdi: "Ad boylam" satırları (Lagna ve grahalar).
# Tablolar BPHS böl. 66; Rust tablosundan (crates/jyotish-core/src/ashtakavarga.rs) ayrı yazılmıştır.
sav_json() {
  awk '
    BEGIN {
      split("Sun Moon Mars Mercury Jupiter Venus Saturn Lagna", R, " ");
      T["Sun"]     = "1,2,4,7,8,9,10,11|3,6,10,11|1,2,4,7,8,9,10,11|3,5,6,9,10,11,12|5,6,9,11|6,7,12|1,2,4,7,8,9,10,11|3,4,6,10,11,12";
      T["Moon"]    = "3,6,7,8,10,11|1,3,6,7,10,11|2,3,5,6,9,10,11|1,3,4,5,7,8,10,11|1,4,7,8,10,11,12|3,4,5,7,9,10,11|3,5,6,11|3,6,10,11";
      T["Mars"]    = "3,5,6,10,11|3,6,11|1,2,4,7,8,10,11|3,5,6,11|6,10,11,12|6,8,11,12|1,4,7,8,9,10,11|1,3,6,10,11";
      T["Mercury"] = "5,6,9,11,12|2,4,6,8,10,11|1,2,4,7,8,9,10,11|1,3,5,6,9,10,11,12|6,8,11,12|1,2,3,4,5,8,9,11|1,2,4,7,8,9,10,11|1,2,4,6,8,10,11";
      T["Jupiter"] = "1,2,3,4,7,8,9,10,11|2,5,7,9,11|1,2,4,7,8,10,11|1,2,4,5,6,9,10,11|1,2,3,4,7,8,10,11|2,5,6,9,10,11|3,5,6,12|1,2,4,5,6,7,9,10,11";
      T["Venus"]   = "8,11,12|1,2,3,4,5,8,9,11,12|3,5,6,9,11,12|3,5,6,9,11|5,8,9,10,11|1,2,3,4,5,8,9,10,11|3,4,5,8,9,10,11|1,2,3,4,5,8,9,11";
      T["Saturn"]  = "1,2,4,7,8,10,11|3,6,11|3,5,6,10,11,12|6,8,9,10,11,12|5,6,11,12|6,11,12|3,5,6,11|1,3,4,6,10,11";
    }
    { sign[$1] = int($2 / 30) % 12 }
    END {
      for (g in T) {
        n = split(T[g], refs, "|");
        for (r = 1; r <= n; r++) {
          m = split(refs[r], places, ",");
          for (p = 1; p <= m; p++) sav[(sign[R[r]] + places[p] - 1) % 12]++;
        }
      }
      for (s = 0; s < 12; s++) printf "%s%d", (s ? ", " : "[ "), sav[s];
      printf " ]" }'
}
```

Döngüde `dasha="$(…)"` atamasından sonra (`cat > "$OUT_DIR/$id.json" <<JSON` satırından önce) ekle:

```bash
  pan="$(panchang_json "$jd" "$lat" "$lon")"
  sav="$(echo "$points" | sav_json)"
```

JSON şablonunda `notes` satırını:

```bash
  "notes": "$note; swetest -sid1 (Lahiri), mean node, UT $d $t; d9_signs ve dasha awk ile türetildi (Plan 06)",
```

şununla değiştir:

```bash
  "notes": "$note; swetest -sid1 (Lahiri), mean node, UT $d $t; d9_signs ve dasha awk ile türetildi (Plan 06); panchang swetest -rise -hindu ve ikiye bölmeyle, sav awk ile türetildi (Plan 07)",
```

ve şablonun sonundaki:

```bash
    "dasha": $dasha
  }
}
JSON
```

şununla değiştir:

```bash
    "dasha": $dasha,
    "panchang": $pan,
    "sav": $sav
  }
}
JSON
```

Notlar: `rise_set_jds` doğumdan 1.25 gün önceden başlayarak 4 doğuş/batış çifti ister; `panchang_json` doğumdan önceki son doğuşu ve ondan sonraki 1 gün içindeki ilk batışı seçer (Rust `day_sunrise`/`day_sunset` kuralıyla aynı). `swetest` saatleri 0.1 sn'ye yuvarladığından gün doğumu beklentisi ≈ 0.05 sn hassasiyetindedir.

- [ ] **Step 4: Fixture'ları yeniden üret ve beklentileri denetle**

Run: `./scripts/gen-golden-swetest.sh`
Expected: beş `wrote …/tests/golden/<id>.json` satırı; stderr'de `swetest problem` yok.

Run: `git diff --stat tests/golden`
Expected: `5 files changed, 80 insertions(+), 10 deletions(-)`: her dosyada yalnızca `notes` satırı, `"dasha": …` satırının sonuna virgül, 13 satırlık `panchang` nesnesi ve `sav` satırı. Mevcut alanların değerleri değişmez.

Run: `grep -h -A13 '"panchang"' tests/golden/*.json | grep -e sunrise_jd -e vara -e '"tithi"'; grep -h '"sav"' tests/golden/*.json`
Expected (sıra dosya adına göre: Buenos Aires, İstanbul, Londra, New Delhi, Tromsø):

| Fixture | Gün doğumu (JD UT) | Gün batımı | Vara | Tithi / bitiş | Nakshatra / bitiş | Yoga / bitiş | Karana / bitiş | SAV (Koç → Balık) |
|---|---|---|---|---|---|---|---|---|
| buenos-aires-1923-06-15 | 2423584.959912 | 2423585.364238 | Thursday | 1 / 2423586.058925 | 5 / 2423585.588332 | 10 / 2423586.064967 | 1 / 2423585.546226 | 38 20 25 30 26 28 27 26 21 31 26 39 |
| istanbul-1987-03-17 | 2446871.679289 | 2446872.172056 | Tuesday | 18 / 2446872.995923 | 14 / 2446872.773372 | 12 / 2446872.765163 | 35 / 2446872.515545 | 23 26 25 26 27 28 31 27 35 34 30 25 |
| london-1850-01-01 | 2396757.842784 | 2396758.162728 | Monday | 19 / 2396759.523273 | 9 / 2396759.000288 | 2 / 2396759.492441 | 37 / 2396759.076838 | 29 31 23 28 32 33 31 24 26 27 29 24 |
| new-delhi-2000-01-01 | 2451544.574985 | 2451545.000560 | Saturday | 26 / 2451545.826235 | 15 / 2451545.044056 | 8 / 2451545.822425 | 51 / 2451545.277284 | 29 21 23 26 34 33 30 30 28 21 32 30 |
| tromso-2050-12-21 | null | null | Wednesday | 9 / 2470163.054546 | 26 / 2470162.511436 | 18 / 2470163.014137 | 17 / 2470162.594498 | 26 29 33 26 33 29 28 27 22 38 17 29 |

Her SAV satırının toplamı 337'dir. Denetimler: New Delhi gün doğumu ve batımı bu planın referans tablosundaki (`-hindu`) değerlerle aynıdır. Londra ve Buenos Aires doğumları UT tarihinden farklı yerel güne düşer (Pazartesi 31.12.1849 ve Perşembe 14.6.1923 gün doğumları), bu Task 3'ün `weekday_uses_the_local_mean_date` testindeki değerlerdir. New Delhi karana 51: `(51 − 2) mod 7 = 0` → Bava. Tromsø'da 21.12.2050 kutup gecesidir: gün doğumu yok, vara 06:00 yerel ortalama saatten.

Değerler bu tablodan saparsa (ör. bir satır eksik ya da numara farklı) fixture commit'lenmez; önce betik çıktısı `swetest -rise -hindu -bj<jd> -ut -geopos<lon>,<lat>,0 -n4` ile elle karşılaştırılır.

- [ ] **Step 5: README'yi güncelle**

`tests/golden/README.md` içinde `## Toleranslar (spec §8)` listesine son madde olarak ekle:

```markdown
- Panchang: tithi, nakshatra, yoga, karana numaraları ve vara birebir; bitiş zamanları, gün doğumu ve batımı ≤ 60 sn. SAV birebir.
```

Şema bloğunda `"dasha": { "first_lord": "Rahu", "first_start_jd": 2445231.575532 }` satırını şununla değiştir:

```markdown
    "dasha": { "first_lord": "Rahu", "first_start_jd": 2445231.575532 },
    "panchang": { "sunrise_jd": 2451544.574985, "sunset_jd": 2451545.000560, "vara": "Saturday",
                  "tithi": 26, "tithi_ends_jd": 2451545.826235, "nakshatra": 15,
                  "nakshatra_ends_jd": 2451545.044056, "yoga": 8, "yoga_ends_jd": 2451545.822425,
                  "karana": 51, "karana_ends_jd": 2451545.277284 },
    "sav": [29, 21, 23, 26, 34, 33, 30, 30, 28, 21, 32, 30]
```

`- En az 5 fixture `d9_signs` ve `dasha` taşımalıdır (`MIN_DERIVED_FIXTURES`).` maddesinden sonra ekle:

```markdown
- `panchang` ve `sav` isteğe bağlıdır. `panchang` doğum anının değerleridir: tithi 1–30, nakshatra 1–27, nitya yoga 1–27, karana 1–60 (yarım tithi sırası), vara İngilizce gün adı. `*_ends_jd` o andan sonraki ilk sınırdır (UT JD). `sunrise_jd` doğum yerinde doğumdan önceki son gün doğumudur (`settings.sunrise_def`; varsayılan disk merkezi, kırılmasız), `sunset_jd` ondan sonraki ilk batıştır; kutup gecesi/gündüzünde ikisi de `null` ve vara 06:00 yerel ortalama saatten bulunur. `sav` Koç'tan Balık'a 12 sayıdır, toplamı 337.
- `swetest` fixture'larında bu iki alan betikte türetilir: gün doğumu `swetest -rise -hindu`'dan, bitişler `swetest -sid1` boylamları üzerinde ikiye bölmeyle, SAV ayrı yazılmış bir BPHS tablosuyla (awk). JHora'dan eklerken: "Panchanga" görünümünden tithi, nakshatra, yoga, karana ve bitiş saatlerini; gün doğumunu ve batımını; Ashtakavarga görünümünden SAV'ı oku. JHora'nın gün doğumu ayarını fixture'ın `sunrise_def`'iyle eşleştir.
- En az 5 fixture `panchang` ve `sav` taşımalıdır (`MIN_PANCHANG_FIXTURES`).
```

`## 25 haritalık JHora setinin kapsamı (spec §8)` listesine son madde olarak ekle:

```markdown
- [ ] Gün doğumundan önce doğum (vara önceki gün) ve kutup dairesinde kutup gecesi
```

- [ ] **Step 6: Testlerin geçtiğini doğrula**

Run: `cargo test -p jyotish-core --test golden`
Expected: `5 passed`. Hata olursa tolerans gevşetilmez; önce `sunrise_def` ve gün doğumu penceresi (doğumdan önceki son doğuş) karşılaştırılır.

Run: `cargo test --workspace`
Expected: Hepsi geçer.

- [ ] **Step 7: Biçim, lint ve commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
shellcheck scripts/gen-golden-swetest.sh || true
git add scripts/gen-golden-swetest.sh crates/jyotish-core/tests/golden.rs tests/golden
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b' || true
git commit -m "test(golden): panchang and SAV expectations derived from swetest" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: clippy temiz, tarama boş. (`shellcheck` kuruluysa uyarıları bilgi amaçlıdır; betik `set -euo pipefail` ile çalışır.)

---

### Task 9: `compute_transits` Tauri komutu, uçtan uca test ve son doğrulama

**Files:**
- Modify: `app/src-tauri/src/commands.rs` (`compute_transits_blocking`, `compute_transits`)
- Modify: `app/src-tauri/src/lib.rs` (komut kaydı)
- Modify: `app/src-tauri/tests/compute_chart.rs` (panchang, ashtakavarga, transit)
- Modify: `app/ui/bindings.ts` (yeniden üretilir)

**Interfaces:**
- Consumes: `jyotish_core::{compute_transits, Bhavas, TransitSnapshot}` (Task 7), `Chart.panchang`, `Chart.ashtakavarga` (Task 6).
- Produces:
  - `pub fn compute_transits_blocking(engine: &Engine, jd_ut: f64, settings: CalcSettings, natal_bhavas: Bhavas) -> Result<TransitSnapshot, CommandError>`.
  - Tauri komutu `compute_transits(jd_ut: f64, settings: CalcSettings, natal_bhavas: Bhavas) -> Result<TransitSnapshot, CommandError>`; TS: `commands.computeTransits(jdUt: number | null, settings: CalcSettings, natalBhavas: Bhavas, options?)`.

- [ ] **Step 1: Başarısız uçtan uca testi yaz**

`app/src-tauri/tests/compute_chart.rs` içinde:

```rust
use jyotish_core::{AyanamsaSystem, CalcSettings, Graha, Sign, Varga};
use jyotish_desk_lib::commands::compute_chart_blocking;
```

şununla değiştir:

```rust
use jyotish_core::{AyanamsaSystem, Bhavas, CalcSettings, Graha, KaranaName, Sign, Vara, Varga};
use jyotish_desk_lib::commands::{compute_chart_blocking, compute_transits_blocking};
```

ve `let err = compute_chart_blocking(&engine, 2_451_545.0, 91.0, 0.0, CalcSettings::default())` satırından **önce** ekle:

```rust
    // Plan 07: panchang ve SAV (tests/golden/new-delhi-2000-01-01.json, swetest türetmesi).
    let p = &chart.panchang;
    let sec = |a: f64, b: f64| (a - b).abs() * 86_400.0;
    assert!(sec(p.sunrise_jd_ut.unwrap(), 2_451_544.574_985) <= 60.0);
    assert!(sec(p.sunset_jd_ut.unwrap(), 2_451_545.000_560) <= 60.0);
    assert_eq!(p.vara, Vara::Saturday);
    assert_eq!(p.tithi.no, 26);
    assert!(sec(p.tithi.ends_jd_ut, 2_451_545.826_235) <= 60.0);
    assert_eq!((p.nakshatra.no, p.yoga.no), (15, 8));
    assert_eq!((p.karana.no, p.karana.name), (51, KaranaName::Bava));
    assert_eq!(
        chart.ashtakavarga.sav,
        [29, 21, 23, 26, 34, 33, 30, 30, 28, 21, 32, 30]
    );

    // Transit: doğum anında haritayla aynı; bir gün sonra Ay ≈ 12–15° ilerler.
    let settings = chart.meta.settings.clone();
    let at_birth =
        compute_transits_blocking(&engine, 2_451_545.0, settings.clone(), chart.bhavas.clone())
            .expect("transits at birth");
    assert_eq!(at_birth.grahas.len(), 9);
    for (t, g) in at_birth.grahas.iter().zip(&chart.grahas) {
        assert_eq!((t.graha, t.sign, t.natal_house), (g.graha, g.sign, g.house));
        assert!((t.lon - g.lon).abs() < 1e-9, "{:?}", t.graha);
    }
    let next = compute_transits_blocking(&engine, 2_451_546.0, settings.clone(), chart.bhavas.clone())
        .expect("transits next day");
    let step = (next.grahas[1].lon - at_birth.grahas[1].lon).rem_euclid(360.0);
    assert!((11.0..16.0).contains(&step), "moon step {step}");
    let broken = Bhavas {
        cusps: vec![0.0; 3],
        ..chart.bhavas.clone()
    };
    let err = compute_transits_blocking(&engine, 2_451_545.0, settings, broken).unwrap_err();
    assert_eq!(err.code, "invalid_transit_input");
    assert_eq!(err.params["field"], "natal_bhavas");
```

- [ ] **Step 2: Testin derlenmediğini doğrula**

Run: `cargo test -p jyotish-desk --test compute_chart`
Expected: FAIL, derleme hatası: `unresolved import 'jyotish_desk_lib::commands::compute_transits_blocking'`.

- [ ] **Step 3: Komutu yaz ve kaydet**

`app/src-tauri/src/commands.rs` içinde:

```rust
use jyotish_core::{CalcSettings, Chart, CoreError, DashaPeriod, DashaSeed, Graha};
```

şununla değiştir:

```rust
use jyotish_core::{
    Bhavas, CalcSettings, Chart, CoreError, DashaPeriod, DashaSeed, Graha, TransitSnapshot,
};
```

`pub async fn compute_chart(…) { … }` komutundan sonra ekle:

```rust

/// `compute_transits` komutunun gövdesi; `compute_chart_blocking` gibi bloklar.
pub fn compute_transits_blocking(
    engine: &Engine,
    jd_ut: f64,
    settings: CalcSettings,
    natal_bhavas: Bhavas,
) -> Result<TransitSnapshot, CommandError> {
    let snapshot = engine.call(move |se: &mut SwissEph| {
        jyotish_core::compute_transits(se, jd_ut, &settings, &natal_bhavas)
    })??;
    Ok(snapshot)
}

/// Transit konumları (UI spec §3.3). `jd_ut` arayüzden gelir (varsayılan "şimdi" ya da
/// seçilen dasha dönemi); `settings` ve `natal_bhavas` natal haritanın
/// `chart.meta.settings` ve `chart.bhavas` alanlarıdır.
#[tauri::command]
#[specta::specta]
pub async fn compute_transits(
    engine: State<'_, Arc<Engine>>,
    jd_ut: f64,
    settings: CalcSettings,
    natal_bhavas: Bhavas,
) -> Result<TransitSnapshot, CommandError> {
    let engine = Arc::clone(&engine);
    tauri::async_runtime::spawn_blocking(move || {
        compute_transits_blocking(&engine, jd_ut, settings, natal_bhavas)
    })
    .await
    .map_err(|e| CommandError::new("engine_stopped", &[("detail", e.to_string())]))?
}
```

`app/src-tauri/src/lib.rs` içinde `commands::compute_chart,` satırından sonra ekle:

```rust
            commands::compute_transits,
```

- [ ] **Step 4: Bağlamaları yeniden üret ve doğrula**

Run: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings`
Expected: `1 passed`.

Run: `grep -n -e 'computeTransits:' -e 'export type TransitSnapshot' -e 'export type TransitGraha' app/ui/bindings.ts`
Expected: üç eşleşme; `computeTransits: (jdUt: number | null, settings: CalcSettings, natalBhavas: Bhavas, options?: InvokeOptions)`.

Run: `cargo test -p jyotish-desk --test compute_chart`
Expected: `1 passed`.

- [ ] **Step 5: Son doğrulama**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p ephemeris -p atlas -p jyotish-core --all-targets
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p jyotish-desk
pnpm --dir app test
pnpm --dir app build
git diff d35da05..HEAD -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b' || true
git diff -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b' || true
```

Expected:
- fmt ve clippy temiz.
- `cargo test --workspace`: `ephemeris` 37, `atlas` 39, `jyotish-core` lib 105, golden 5, `jyotish-desk` lib 8, `bindings` 1, `compute_chart` 1; hata yok.
- `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
- İki MSRV `cargo check` geçer.
- `pnpm` test ve build geçer.
- İki yasaklı dizgi taraması boş.

- [ ] **Step 6: Commit**

```bash
git add app/src-tauri/src/commands.rs app/src-tauri/src/lib.rs app/src-tauri/tests/compute_chart.rs app/ui/bindings.ts
git diff --cached -U0 | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b' || true
git commit -m "feat(app): compute_transits command; end-to-end panchang and transit test" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Expected: tarama boş; `git status` temiz (`.superpowers/` izlenmez).

---

## Bu planın tamamlanma ölçütü

- `Ephemeris::sun_rise_set` `swetest -rise`'a ≤ 0.1 sn uyar; kutup gecesi `Ok(None)` döner (Task 2).
- `Chart.panchang` ve `Chart.ashtakavarga` her haritada doludur; kutup gecesinde `SunriseUnavailable` uyarısı vardır (Task 6).
- 5 golden fixture'da panchang numaraları ve vara birebir, zamanlar ≤ 60 sn, SAV birebir (Task 8).
- BAV toplamları 48/49/39/54/56/52/39 ve SAV 337 proptest ile, elle sayılmış harita birim testiyle doğrulanır (Task 5–6).
- `compute_transits` saf çekirdek fonksiyonu ve Tauri komutu vardır; doğum anında harita ile özdeştir (Task 7, 9).
- D1 (IPC float gidiş-dönüşü) ve D2 (`JD_UT_RANGE`) kapanmıştır (Task 1).
- `cargo test --workspace`, clippy, fmt, deny, MSRV ve `pnpm` test/build yeşildir.

## Bu planın kapsamadıkları

- Panchang, ashtakavarga ve transit **arayüzü** (paneller, D1 transit halkası, transit tablosu, dasha seçiminden transit tarihi): UI planı. Bu plan yalnızca IPC tiplerini ve komutları sağlar.
- Tithi, nakshatra, nitya yoga ve vara **adlarının** i18n'i (çekirdek numara verir; karana adı istisnadır).
- Ay doğuşu/batışı, gün doğumu anının panchangı (muhurta görünümü), gün içi Rahu, Yamaganda ve Gulika dilimleri, hora.
- Ashtakavarga shodhana (trikona, ekadhipatya), prastara/kaksha, BAV'a göre transit puanlaması.
- Transit için lagna, dignity, yanma, vedha, murti; transit olay araması (giriş/çıkış tarihleri).
- JHora'dan bağımsız panchang/SAV fixture'ları (kullanıcı ekler; README yönergesi Task 8'de).

## Self-Review

**Kapsam (istek → görev):**

| İstek | Görev |
|---|---|
| `swe_rise_trans` FFI, trait `sun_rise_set`, kutupta açık sonuç (`Ok(None)`), `swetest`'e karşı test | 2 |
| Panchang: tithi, vara, nakshatra, yoga, karana, gün doğumu/batımı; bitişler ikiye bölmeyle ≤ 1 sn, sınırların gerekçesi | 3, 4 (karar tablosu "Bitiş zamanı arama") |
| Ashtakavarga: BPHS böl. 66 tam tablolar, BAV, SAV; toplam proptest'i (48/49/39/54/56/52/39, 337); elle sayılmış harita | 5, 6 |
| Asgari transit: `compute_transits` + Tauri komutu | 7, 9 |
| `Chart`'a panchang ve ashtakavarga; golden fixture'lar isteğe bağlı alanlarla; bağlamalar | 6, 8, 9 |
| Plan 06 incelemesi D1 (`float_roundtrip`) ve D2 (`birth_jd_ut` aralığı) | 1 |
| Çekirdek saf, `Result<_, CoreError>`, panik yok, serde snake_case, yeni crate yok | Global Constraints; her görevin Interfaces'i |

**Yer tutucu taraması:** Her kod adımı tam koddur. "…" yalnızca düzyazıda mevcut bir bloğun yerini tarif ederken geçer (ör. `pub use houses::{…};`). Beklenen değerler sayıyla verilmiştir (Task 8 tablosu, Task 3–7 testleri).

**Tip ve ad tutarlılığı:**
- `sun_rise_set(after_jd_ut, RiseSet, geo_lat, geo_lon, SunDisc) -> Result<Option<f64>, EphError>`: Task 2'de tanımlanır; `FakeEph` (Task 2, 4), uygulama `Fake`'i (Task 2) ve `day_sunrise`/`day_sunset` (Task 4) aynı imzayı kullanır.
- `check_moment` (Task 1) → `compute_panchang` (Task 4). `JD_UT_RANGE` (Task 1) → `DashaSeed::validate`, `compute_transits` (Task 7).
- `Panchang` alanları (`tithi`, `vara`, `nakshatra`, `yoga`, `karana`, `sunrise_jd_ut`, `sunset_jd_ut`) Task 3'te tanımlanır; Task 6 (`Chart.panchang`), Task 8 (`compare_panchang`) ve Task 9 (uçtan uca test) aynı adları okur.
- `Ashtakavarga { bav: [[u8; 12]; 7], sav: [u8; 12] }` (Task 5) → `Chart.ashtakavarga` (Task 6), golden `sav: Option<[u8; 12]>` (Task 8).
- `CoreError::PanchangSearch` (Task 4) ve `InvalidTransitInput` (Task 7) için `commands.rs` eşlemesi aynı görevde eklenir; `match` her görev sonunda kapsayıcıdır.
- `raw_grahas`, `rasi_frame`, `to_sidereal`, `finite` Task 7'de `pub(crate)` olur; yalnızca `transit.rs` kullanır.

**Test sayıları (görev sonunda, `cargo test --workspace`):**

| Görev | `ephemeris` | `jyotish-core` lib | golden | `jyotish-desk` lib | bindings | compute_chart |
|---|---|---|---|---|---|---|
| Başlangıç (d35da05) | 33 | 83 | 3 | 7 | 1 | 1 |
| 1 | 33 | 83 | 3 | 8 | 1 | 1 |
| 2 | 37 | 83 | 3 | 8 | 1 | 1 |
| 3 | 37 | 88 | 3 | 8 | 1 | 1 |
| 4 | 37 | 95 | 3 | 8 | 1 | 1 |
| 5 | 37 | 99 | 3 | 8 | 1 | 1 |
| 6 | 37 | 102 | 3 | 8 | 1 | 1 |
| 7 | 37 | 105 | 3 | 8 | 1 | 1 |
| 8 | 37 | 105 | 5 | 8 | 1 | 1 |
| 9 | 37 | 105 | 5 | 8 | 1 | 1 |

`atlas` her görevde 39'dur. Task 1 mevcut testlere satır ekler (`rejects_invalid_input`, `invalid_seed_is_rejected`); yeni çekirdek testi yoktur.

**Bilinen riskler:**
- Ashtakavarga tabloları Rust'ta ve betikte aynı yazar tarafından aktarıldı; toplamlar (BPHS) doğrular ama tek bir hücre kayması toplamı korumaz. JHora SAV'ıyla karşılaştırma (README yönergesi) bunu kapatır.
- Panchang golden'ı aynı SE'den türetilir (tesisat kanıtı). Doğum anı mı gün doğumu anı mı sorusu bir konvansiyondur; karar tablosunda JHora "Panchanga" görünümüne göre seçildi.
- Kutupta 06:00 yerel ortalama saat yedeği bir konvansiyondur ve `SunriseUnavailable` ile açıkça işaretlenir.
- specta'nın sabit dizileri (`[[u8; 12]; 7]`) TS'e nasıl yazdığı bağlamalar üretilince görülür; arayüz henüz okumadığı için iki biçim de kabul edilir.
- `JD_UT_RANGE` daha önce kabul edilen sonlu ama aşırı JD'leri reddeder (davranış değişikliği, `invalid_julian_day`).
- Harita başına ≈ 180 ek SE çağrısı (≈ 3 ms).
