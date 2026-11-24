# Plan 06 — `jyotish-core`: 16 Varga (Shodashavarga) ve Vimshottari Dasha (M3, 1. kısım)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `Chart`'a BPHS'nin 16 vargasını (D1–D60, Parashara tanımları) ve 3 seviyeli Vimshottari dashayı eklemek; 4. ve 5. seviyeler için saf `dasha_children` fonksiyonu ve Tauri komutu sağlamak; golden şemayı D9 burçları ve ilk mahadasha ile genişletmek; `bindings.ts`'deki `Chart_Serialize | Chart_Deserialize` ayrımını kaldırmak.

**Architecture:**
- `crates/jyotish-core/src/varga.rs`: saf, tablo güdümlü bölüm kuralları. Girdi rasi çerçevesindeki boylamdır; çıktı varga burcu ve varga içi derecedir. `compute_chart` 16 vargayı her zaman hesaplar (≈ 160 aritmetik işlem).
- `crates/jyotish-core/src/dasha.rs`: saf Vimshottari. Tohum (`DashaSeed`: doğum JD'si, Ay'ın sidereal boylamı, yıl uzunluğu) tüm ağacı belirler. `compute_chart` 1–3. seviyeleri düz, ön-sıralı (preorder) bir listede taşır (9 + 81 + 729 = 819 dönem). Daha derin seviyeler `dasha_children(seed, path)` ile istenir; ephemeris gerekmez.
- `app/src-tauri`: `compute_chart` değişmeden yeni alanları döndürür. `dasha_children` motor kullanmayan senkron bir komuttur. `specta_builder()` serde faz ayrımını kapatır; her IPC tipi tek bir TS tipidir.
- Golden fixture'lar geriye uyumlu genişler: `expected.d9_signs` ve `expected.dasha` isteğe bağlıdır. Mevcut 5 `swetest` fixture'ına üretici betikteki bağımsız bir awk uygulamasıyla eklenir.

**Tech Stack:** Rust 1.93, Swiss Ephemeris 2.10.03 (vendored), `serde` 1, `thiserror` 2, `serde_json` 1 (dev), `proptest` 1 (dev, `default-features = false`), `specta` `=2.0.0-rc.28` (isteğe bağlı özellik), `tauri-specta` `=2.0.0-rc.29`, Tauri 2. **Yeni bağımlılık yok.**

**Spec:** `docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md` §3 (16 varga, Vimshottari 5 seviye, 4–5 tembel), §4 veri akışı ("Dasha 3 seviye gelir", `dasha_children(path)`), §5 (`VargaChart`, `DashaPeriod`, `Chart`), §6 (dasha yılı), §8 (mahadasha ≤ 1 gün, varga burçları birebir, dasha proptest'leri), §11 (M3). Önceki plan: `docs/superpowers/plans/2026-11-25-plan-05-core.md`. Ertelenen madde: `.superpowers/sdd/plan05-final-review.md` T8a.

## Global Constraints

- Lisans: AGPL-3.0-or-later (`license.workspace = true`). Yeni bağımlılıklar `deny.toml` lisans listesine uymalı ve `cargo deny check` temiz kalmalı.
- "Kala", "Kāla", "Kaala", "Cala" ve türevleri hiçbir isimde, yorumda ya da dosya adında kullanılmaz.
- MSRV 1.93 (workspace). CI'da `core` (ubuntu, windows) ve `msrv` işleri çekirdek crate'leri kapsar; `jyotish-core` da bir çekirdek crate'tir.
- `jyotish-core` saftır: dosya, ağ, saat ve Tauri yok (testler hariç). `atlas`'a bağımlı değildir. Ephemeris'e yalnızca `ephemeris::Ephemeris` trait'i ile erişir. `Body`, `Zodiac`, `SiderealMode` gibi değer tiplerini kullanabilir.
- Çekirdekte panik yok: tüm hatalar `Result<_, CoreError>` olarak döner. FFI sınırında panik olmaz.
- **Varsayılanlar (spec §6):**
  - Düğüm: Mean Rahu. True seçenek olarak var.
  - Ev sistemi: Whole Sign. Bhava chalit için Sripati, KP için Placidus seçenek olarak var.
  - Dasha yılı: 365.25 gün. Seçenekler: 360 ve 365.2422.
  - Yanma orbları (BPHS): Ay 12°, Mars 17°, Merkür 14° (retroda 12°), Jüpiter 11°, Venüs 10° (retroda 8°), Satürn 15°. Düğümler yanmaz.
  - Dignity: BPHS. Yücelme ve düşüş dereceleri, moolatrikona, kendi burcu ve panchadha maitri. Düğümlere dignity atanmaz.
  - Konumlar: Jeosantrik, görünür konum.
  - Gün doğumu: Disk merkezi, kırılmasız (`SE_BIT_HINDU_RISING`). Ayarlanabilir.
- **Toleranslar (spec §8):** Ephemeris sarmalayıcısı `swetest`'e karşı ≤ 0.001″. Golden haritalarda graha boylamı ≤ 2″, ayanamsa ≤ 1″, lagna ≤ 5″. **Mahadasha başlangıcı ≤ 1 gün. Varga burçları birebir aynı.** Toleranslar gevşetilmez.
- **Plan 01 notları:**
  - Sidereal mod SE'de global durumdur, her çağrıda yeniden ayarlanır.
  - Ayanamsa nütasyon dahildir (`SEFLG_NONUT` yok).
  - Ketu = Rahu + 180°, tam olarak.
  - Harita hassasiyeti Güneş/Ay konumlarından alınır, düğümlerden ya da ayanamsadan değil.
- **Serde:** IPC'yi geçecek tüm public tiplerde `Serialize` + `Deserialize` olur. JSON alan adları snake_case, enum varyantları PascalCase dizgidir (`"Sidereal"`, `{"Custom": {"t0_jd": …, "deg": …}}`). Varga adları `"D1"` … `"D60"`.
- **Kapsam dışı (Plan 07 ve sonrası):** panchang, gün doğumu hesabı, ashtakavarga ve transit. `Chart` bu alanları **içermez**; boş alan ya da taslak (stub) eklenmez. `Chart.moment` (`ResolvedMoment`) da yoktur; `Chart` yalnızca `jd_ut`, `geo_lat` ve `geo_lon` taşır. Bilinmeyen doğum saatinde lagnaya bağlı vargaları gizlemek ve dashayı "yaklaşık" işaretlemek (spec §7) arayüz/`store` işidir; çekirdek `time_certainty` bilmez.
- Commit mesajları İngilizce olur ve `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` satırıyla biter. `.superpowers/` asla `git add` edilmez.
- Plan kod blokları rustfmt-temiz olmayabilir: her görevde doğrulamadan ve commit'ten önce `cargo fmt --all` çalıştır; `cargo fmt --all --check` CI'da zorunlu.
- **Toolchain:** `cargo fmt`, `cargo clippy` ve `cargo test` stable toolchain ile çalışır (1.93 kurulumunda clippy/rustfmt yok). MSRV denetimi yalnızca `cargo +1.93 check …` ile yapılır. Kodda `f64::next_down` kullanılır (Rust 1.86'da kararlı, MSRV 1.93 içinde).
- Plan 05'in astrolojik tanımları (dignity tabloları, yanma, ev sistemleri, tropikal-rasi kipi) değişmez; bu plan onlara dokunmaz.

### Bu planın kararları

| Konu | Karar | Gerekçe |
|---|---|---|
| Vargalar ne zaman hesaplanır | **Her zaman, 16'sı birden.** `Chart.vargas: Vec<VargaChart>`, `Varga::ALL` sırasında. | 16 × 10 nokta saf aritmetik, mikrosaniye düzeyi. Harita süresine SE çağrıları (~10) hâkimdir; ≤ 50 ms bütçesi tehlikede değil. İsteğe bağlı hesap API'yi ve IPC'yi karmaşıklaştırır. |
| Vargaların çerçevesi | Rasi çerçevesindeki `lon` (sidereal kipte sidereal, `TropicalRasiSiderealNakshatra` kipinde tropikal). | Varga, rasi burcunun bölümüdür; D1 her kipte `GrahaPosition.sign` ile aynı olur. |
| Varga içi derece (`deg`) | Grahanın düştüğü parçadaki konumu 30°'ye ölçeklenir: parça `[a, b)` ise `deg = (d − a) / (b − a) × 30`. Eşit parçalı vargalarda bu `(d × N) mod 30`'dur; D1'de `deg_in_sign`. D30'da eşit olmayan parçaya göre orantılıdır. Aralık `[0, 30)`. | JHora ve yaygın yazılımların "varga boylamı = boylam × N" gösterimiyle eşit parçalarda aynıdır. |
| `VargaChart.placements` | Spec §5'teki `(graha, sign, deg)` üçlüsü bir struct olur: `VargaPlacement { graha, sign, deg }`. Lagna için yalnızca `lagna_sign` (spec). | TS'de adlandırılmış alanlar demetten (tuple) okunaklıdır. |
| Vargottama | `GrahaPosition.vargottama: bool` (D1 burcu == D9 burcu). Lagna için alan yok; arayüz `vargas` içinden D1/D9 `lagna_sign`'ı karşılaştırabilir. | Vargottama bir graha niteliğidir ve graha tablosunda gösterilir; D9 haritasına bağlamak yanlış yer olurdu. |
| Dasha zaman gösterimi | `start_jd_ut` / `end_jd_ut: f64` (UT Jülyen günü). Spec'teki `start_utc`/`end_utc` adları bu yüzden değişir. | Çekirdek saat/takvim bağımlılığı taşımaz; `Chart.jd_ut` ile aynı birim. Arayüz biçimlendirir. |
| Dasha yapısı | `Chart.dasha: Dasha { seed: DashaSeed, periods: Vec<DashaPeriod> }`. `periods` 1–3. seviyeleri **ön-sıra** düz listede taşır: her mahadasha, ardından antardashaları, her antardashanın ardından pratyantarları. `has_children` = `level < 5`. | Spec `DashaPeriod`'da çocuk alanı yok; düz liste `lords` önekleriyle ağaca çevrilir. ≈ 115 KB JSON. |
| `dasha_children` girdisi | `dasha_children(seed: &DashaSeed, path: &[Graha])`. `path = []` mahadashaları, `path.len() == 4` 5. seviyeyi verir; daha uzunu `InvalidDashaInput("path")`. Tauri komutu `dasha_children(seed, path)`; motor kullanmaz. | Dasha yalnızca tohuma bağlıdır; haritayı yeniden hesaplamak gereksiz. Arayüz tohumu `chart.dasha.seed`'den alır. |
| Dasha yılı aralığı | `CalcSettings::validate` ve `DashaSeed::validate` `dasha_year`/`year_days`'i `DASHA_YEAR_RANGE = 300.0..=400.0` ile sınırlar. | Önceki denetim yalnızca `> 0` idi; `1e308` gibi bir değer sonsuz bitiş tarihleri üretirdi (sessiz yanlış sonuç). Spec seçenekleri 360, 365.2422, 365.25 bu aralıktadır. |
| `Chart_Serialize \| Chart_Deserialize` | `specta_builder()` içinde `.disable_serde_phases()`. | `specta-serde 0.0.15` her `Option<T>` alanını deserialize fazında isteğe bağlı sayar (`lib.rs`, `field_has_local_difference` → `declared_option_is_deserialize_optional`) ve tipi ikiye böler. Alan düzeyinde `deserialize_with`/`with` de faz farkı sayılır ve `#[specta(type)]` ister (`validate.rs`, `ensure_codec_override`). Birleşik kipte `dignity: Dignity \| null` olur; IPC tiplerimizde `default`, `skip_serializing_if` ya da asimetrik `rename` yok, yani birleşik tip iki yön için de doğrudur. |

### Astrolojik tanımlar: Shodashavarga (kaynak: BPHS böl. 6 "Shodasavargadhyaya", R. Santhanam çevirisi)

Gösterim: `s` burç indeksi (Koç 0 … Balık 11), `d` burç içi derece `[0, 30)`. **Tek (oja) burç** = Koç, İkizler, Aslan, Terazi, Yay, Kova (`s` çift). **Hareketli (chara)** = Koç, Yengeç, Terazi, Oğlak (`s % 3 == 0`); **sabit (sthira)** = Boğa, Aslan, Akrep, Kova (`s % 3 == 1`); **çift doğalı (dvisvabhava)** = İkizler, Başak, Yay, Balık (`s % 3 == 2`). Ateş/toprak/hava/su = `s % 4` = 0/1/2/3. Eşit parçalı vargada parça `p = floor(d × N / 30)`, varga burcu `(başlangıç + p) mod 12`. Parça sınırı sonraki parçaya aittir.

| Varga | Ad | Parça | Kural (başlangıç burcu) |
|---|---|---|---|
| D1 | Rasi | 30° | Burcun kendisi. |
| D2 | Hora | 15° | Parashara horası: tek burçta 1. yarı Güneş (Aslan), 2. yarı Ay (Yengeç); çift burçta 1. yarı Ay (Yengeç), 2. yarı Güneş (Aslan). |
| D3 | Drekkana | 10° | Parashara drekkanası: 1. parça burcun kendisi, 2. parça 5. burç, 3. parça 9. burç (`s + 4p`). |
| D4 | Chaturthamsa | 7°30′ | Kendisi, 4., 7., 10. burç (`s + 3p`). |
| D7 | Saptamsa | 30/7° | Tek burçta kendisinden, çift burçta 7. burçtan. |
| D9 | Navamsa | 3°20′ | Hareketlide kendisinden, sabitte 9.'dan, çift doğalıda 5.'den. Eşdeğeri: ateş Koç'tan, toprak Oğlak'tan, hava Terazi'den, su Yengeç'ten; ya da `floor(boylam × 9 / 30) mod 12`. |
| D10 | Dasamsa | 3° | Tek burçta kendisinden, çift burçta 9.'dan. |
| D12 | Dwadasamsa | 2°30′ | Kendisinden. |
| D16 | Shodasamsa | 1°52′30″ | Hareketlide Koç'tan, sabitte Aslan'dan, çift doğalıda Yay'dan. |
| D20 | Vimsamsa | 1°30′ | Hareketlide Koç'tan, sabitte Yay'dan, çift doğalıda Aslan'dan. |
| D24 | Chaturvimsamsa (Siddhamsa) | 1°15′ | Tek burçta Aslan'dan, çift burçta Yengeç'ten. |
| D27 | Saptavimsamsa (Bhamsa) | 30/27° | Ateşte Koç'tan, toprakta Yengeç'ten, havada Terazi'den, suda Oğlak'tan. Eşdeğeri `floor(boylam × 27 / 30) mod 12`. |
| D30 | Trimsamsa | eşit değil | Tek burç: 0–5° Mars → Koç, 5–10° Satürn → Kova, 10–18° Jüpiter → Yay, 18–25° Merkür → İkizler, 25–30° Venüs → Terazi. Çift burç (ters sıra): 0–5° Venüs → Boğa, 5–12° Merkür → Başak, 12–20° Jüpiter → Balık, 20–25° Satürn → Oğlak, 25–30° Mars → Akrep. Güneş ve Ay'ın trimsamsası yoktur (D30 burcu asla Yengeç/Aslan değildir). |
| D40 | Khavedamsa | 0°45′ | Tek burçta Koç'tan, çift burçta Terazi'den. |
| D45 | Akshavedamsa | 0°40′ | Hareketlide Koç'tan, sabitte Aslan'dan, çift doğalıda Yay'dan. |
| D60 | Shashtyamsa | 0°30′ | Burcun kendisinden (`s + floor(2d)`): BPHS "dereceyi 2 ile çarp, 12'ye böl, kalana 1 ekle, burcun kendisinden say". |

Kullanılmayan alternatifler (bilinçli): Jagannatha/Somanatha drekkanaları, Kashinatha horası, parivritti trimsamsa. Spec "Parashara tanımları" der.

### Astrolojik tanımlar: Vimshottari (kaynak: BPHS, nakshatra dashaları bölümü; Santhanam çevirisinde böl. 46)

- Lord sırası ve yılları (toplam 120): Ketu 7, Venüs 20, Güneş 6, Ay 10, Mars 7, Rahu 18, Jüpiter 16, Satürn 19, Merkür 17. Ashwini (1.) Ketu ile başlar ve sıra her 9 nakshatrada tekrar eder: nakshatra `n` (1..=27) için lord `SIRA[(n − 1) mod 9]`.
- Doğum mahadashası: doğumdaki Ay'ın **sidereal** boylamının nakshatrasının lordu (her zodyak kipinde sidereal; nakshatra her zaman sidereal'dir). Geçen kesir `f = (λ − (n − 1) × 13°20′) / 13°20′`, `[0, 1]`'e kırpılır.
- Birinci mahadasha **teorik başlangıcı** `başlangıç = jd_doğum − f × yıl(lord) × dasha_yılı` (doğumdan önce). Bakiye `= (1 − f) × yıl(lord)`. Dokuz mahadasha bu başlangıçtan ardışık dizilir ve 120 yılı kaplar.
- Alt dönem: süresi `D` olan bir dönemin dokuz alt dönemi, **ebeveynin lordundan başlayarak** aynı sırayla gelir; lord `L`'nin payı `D × yıl(L) / 120`. Aynı kural 2. (antar), 3. (pratyantar), 4. (sookshma) ve 5. (prana) seviyelere uygulanır.
- Sayısal kural: alt dönem bitişleri birikimli yıldan hesaplanır (`başlangıç + D × Σyıl / 120`); sonuncusu ebeveyn bitişine tam eşitlenir. Böylece ardışıklık (`bitiş_i == başlangıç_{i+1}`) bit düzeyinde tamdır ve kayma birikmez.

---

## Dosya Yapısı (bu plan)

```
crates/jyotish-core/
  src/varga.rs          # Varga, varga_position, VargaPlacement, VargaChart, varga_chart, all_vargas, is_vargottama (Task 2–4)
  src/dasha.rs          # DashaSeed, DashaPeriod, Dasha, birth_dasha, mahadashas, dasha_children, vimshottari (Task 5–6)
  src/lib.rs            # modüller ve yeniden dışa aktarımlar
  src/error.rs          # CoreError::InvalidDashaInput (Task 5)
  src/settings.rs       # DASHA_YEAR_RANGE, dasha_year denetimi (Task 5), belge yorumu (Task 6)
  src/graha.rs          # GrahaPosition.vargottama (Task 4)
  src/chart.rs          # Chart.vargas (Task 4), Chart.dasha (Task 6)
  src/proptests.rs      # varga ve dasha özellikleri (Task 2, 3, 6)
  tests/golden.rs       # d9_signs ve dasha karşılaştırması (Task 7)
scripts/gen-golden-swetest.sh   # türetilmiş beklentiler (awk) (Task 7)
tests/golden/
  README.md             # şema ve kaynak notu (Task 7)
  *.json                # 5 fixture, yeniden üretilir (Task 7)
app/src-tauri/
  src/lib.rs            # disable_serde_phases (Task 1), dasha_children kaydı (Task 8)
  src/commands.rs       # InvalidDashaInput eşlemesi (Task 5), dasha_children komutu (Task 8)
  tests/bindings.rs     # faz ayrımı koruması (Task 1)
  tests/compute_chart.rs# vargalar ve dasha (Task 8)
app/ui/bindings.ts      # yeniden üretilir (Task 1, 4, 6, 8)
```

`bindings.ts` her `Chart` değişikliğinde yeniden üretilir (Task 4, 6), çünkü `bindings_ts_is_up_to_date` testi `cargo test --workspace`'in parçasıdır; her görev sonunda iş alanı yeşil kalır.

---

### Task 1: Tek `Chart` tipi: serde faz ayrımını kapat

**Files:**
- Modify: `app/src-tauri/src/lib.rs` (`specta_builder`)
- Modify (üretilir): `app/ui/bindings.ts`
- Test: `app/src-tauri/tests/bindings.rs`

**Interfaces:**
- Consumes: `tauri_specta::Builder::disable_serde_phases` (tauri-specta rc.29, `builder.rs`).
- Produces: `bindings.ts`'de `export type Chart = { … }` ve `export type GrahaPosition = { …, dignity: Dignity | null, … }`; `_Serialize`/`_Deserialize` ekli tip yok. `computeChart` `typedError<Chart, CommandError>` döner. Sonraki görevlerin yeni tipleri de tek tip olarak çıkar.

- [ ] **Step 1: Başarısız testi yaz**

`app/src-tauri/tests/bindings.rs` dosyasının tamamı:

```rust
//! `app/ui/bindings.ts` dosyasının komut tanımlarıyla aynı olduğunu doğrular.
//! Güncellemek için: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings`

use std::path::PathBuf;

#[test]
fn bindings_ts_is_up_to_date() {
    let committed = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ui/bindings.ts");
    let fresh = std::env::temp_dir().join(format!("jd-bindings-{}.ts", std::process::id()));
    jyotish_desk_lib::specta_builder()
        .export(specta_typescript::Typescript::default(), &fresh)
        .expect("export bindings");
    let fresh_text = std::fs::read_to_string(&fresh).expect("read fresh");
    let _ = std::fs::remove_file(&fresh);
    // Her IPC tipi tek bir TS tipidir: `X_Serialize | X_Deserialize` ayrımı yok.
    assert!(
        !fresh_text.contains("_Serialize") && !fresh_text.contains("_Deserialize"),
        "bindings split a type into serde phases; keep disable_serde_phases() in specta_builder"
    );
    if std::env::var_os("UPDATE_BINDINGS").is_some() {
        std::fs::write(&committed, &fresh_text).expect("write bindings.ts");
        return;
    }
    let committed_text = std::fs::read_to_string(&committed).unwrap_or_default();
    assert!(
        committed_text == fresh_text,
        "app/ui/bindings.ts is stale; run: UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings"
    );
}
```

- [ ] **Step 2: Testin düştüğünü doğrula**

Run: `cargo test -p jyotish-desk --test bindings`
Expected: FAIL, `bindings split a type into serde phases; keep disable_serde_phases() in specta_builder`.

- [ ] **Step 3: Faz ayrımını kapat**

`app/src-tauri/src/lib.rs` içinde `specta_builder` fonksiyonunun tamamı:

```rust
/// Komut listesi tek yerde: hem uygulama hem bağlama testi bunu kullanır.
pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
    // rc.29 varsayılanı `Throw`; arayüz `{ status, data | error }` sonucunu bekler.
    // Serde fazları kapalı: specta-serde her `Option<T>` alanında tipi
    // `X_Serialize | X_Deserialize` olarak ikiye böler. IPC tiplerimizde faz farkı yaratan
    // serde özniteliği (default, skip_serializing_if, asimetrik rename) yok; tek tip iki
    // yön için de doğrudur.
    tauri_specta::Builder::<tauri::Wry>::new()
        .error_handling(tauri_specta::ErrorHandlingMode::Result)
        .disable_serde_phases()
        .commands(tauri_specta::collect_commands![
            commands::engine_status,
            commands::planet_longitudes,
            commands::compute_chart,
            commands::print_window,
        ])
}
```

- [ ] **Step 4: Bağlamaları yeniden üret ve doğrula**

Run:

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
cargo test -p jyotish-desk --test bindings
grep -c "_Serialize\|_Deserialize" app/ui/bindings.ts
grep -n "export type Chart = {\|export type GrahaPosition = {\|dignity: Dignity | null\|typedError<Chart, CommandError>" app/ui/bindings.ts
```

Expected:
- İki test çalışması da geçer.
- İlk `grep` `0` basar.
- İkinci `grep` dört satır basar: `export type Chart = {`, `export type GrahaPosition = {`, `dignity: Dignity | null,` ve `computeChart` satırı (`typedError<Chart, CommandError>`).

> Not: Export bir specta doğrulama hatasıyla düşerse (birleşik kip bir tipi reddederse) hata mesajındaki tipi raporla ve dur. Beklenen durum bu değildir: dışa aktarılan tiplerde birleşik kipin reddettiği öznitelik yok.

- [ ] **Step 5: Uygulama testleri ve TS derlemesi**

Run: `cargo test -p jyotish-desk && pnpm --dir app test && pnpm --dir app build`
Expected: Tümü geçer. Arayüz `Chart`/`GrahaPosition` tiplerini henüz kullanmadığından TS derlemesi etkilenmez.

- [ ] **Step 6: Biçim ve lint**

Run: `cargo fmt --all && cargo fmt --all --check && cargo clippy -p jyotish-desk --all-targets -- -D warnings`
Expected: Uyarısız geçer.

- [ ] **Step 7: Commit**

```bash
git add app/src-tauri/src/lib.rs app/src-tauri/tests/bindings.rs app/ui/bindings.ts
git commit -m "fix(app): export each IPC type once (disable serde phase split)" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: `varga.rs`: `Varga` ve D1–D12 kuralları

**Files:**
- Create: `crates/jyotish-core/src/varga.rs`
- Modify: `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/proptests.rs`
- Test: `varga.rs` içindeki `mod tests`, `proptests.rs` içindeki `mod varga_properties`

**Interfaces:**
- Consumes: `Sign` (`Sign::of`, `Sign::from_index`, `Sign::index`), `deg_in_sign`, `normalize` (Plan 05 `zodiac.rs`).
- Produces:
  - `pub enum Varga { D1, D2, D3, D4, D7, D9, D10, D12 }` (Task 3'te 16 varyanta çıkar), `Varga::ALL`, `pub fn divisions(self) -> u8`
  - `pub fn varga_position(varga: Varga, lon: f64) -> (Sign, f64)`: rasi çerçevesindeki `lon` için varga burcu ve varga içi derece `[0, 30)`.

- [ ] **Step 1: Başarısız testleri yaz**

`crates/jyotish-core/src/varga.rs` (önce yalnızca test modülü; Step 3'te üstüne kod eklenir):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn check(cases: &[(Varga, f64, Sign)]) {
        for &(v, lon, want) in cases {
            assert_eq!(varga_position(v, lon).0, want, "{v:?} at {lon}°");
        }
    }

    fn deg(v: Varga, lon: f64) -> f64 {
        varga_position(v, lon).1
    }

    #[test]
    fn all_in_order_with_divisions() {
        let n: Vec<u8> = Varga::ALL.iter().map(|v| v.divisions()).collect();
        assert_eq!(n, [1, 2, 3, 4, 7, 9, 10, 12]);
    }

    #[test]
    fn lower_vargas_follow_parashari_rules() {
        use Sign::*;
        use Varga::*;
        check(&[
            // D1: burcun kendisi.
            (D1, 0.0, Aries),
            (D1, 29.999, Aries),
            (D1, 30.0, Taurus),
            (D1, 359.99, Pisces),
            // D2: tek burç Aslan → Yengeç, çift burç Yengeç → Aslan.
            (D2, 0.0, Leo),
            (D2, 14.999, Leo),
            (D2, 15.0, Cancer),
            (D2, 30.0, Cancer),
            (D2, 45.0, Leo),
            (D2, 359.0, Leo),
            // D3: kendisi, 5., 9.
            (D3, 0.0, Aries),
            (D3, 9.999, Aries),
            (D3, 10.0, Leo),
            (D3, 20.0, Sagittarius),
            (D3, 40.0, Virgo),
            (D3, 355.0, Scorpio),
            // D4: kendisi, 4., 7., 10.
            (D4, 0.0, Aries),
            (D4, 7.5, Cancer),
            (D4, 15.0, Libra),
            (D4, 22.5, Capricorn),
            (D4, 37.5, Leo),
            // D7: tek burç kendisinden, çift burç 7.'den. 30/7 = 4.2857°.
            (D7, 4.28, Aries),
            (D7, 4.29, Taurus),
            (D7, 29.0, Libra),
            (D7, 30.0, Scorpio),
            (D7, 59.9, Taurus),
            // D9: hareketli kendisinden, sabit 9.'dan, çift doğalı 5.'den.
            (D9, 0.0, Aries),
            (D9, 3.33, Aries),
            (D9, 3.34, Taurus),
            (D9, 30.0, Capricorn),
            (D9, 60.0, Libra),
            (D9, 90.0, Cancer),
            (D9, 120.0, Aries),
            (D9, 359.9, Pisces),
            // D10: tek burç kendisinden, çift burç 9.'dan.
            (D10, 0.0, Aries),
            (D10, 3.0, Taurus),
            (D10, 30.0, Capricorn),
            (D10, 59.0, Libra),
            // D12: kendisinden.
            (D12, 0.0, Aries),
            (D12, 2.5, Taurus),
            (D12, 29.9, Pisces),
            (D12, 120.0, Leo),
        ]);
    }

    #[test]
    fn degree_in_varga_is_position_in_part_scaled_to_30() {
        assert!((deg(Varga::D1, 46.5) - 16.5).abs() < 1e-9);
        assert!((deg(Varga::D2, 20.0) - 10.0).abs() < 1e-9); // 20 × 2 = 40 → 40 − 30
        assert!((deg(Varga::D3, 25.0) - 15.0).abs() < 1e-9); // 25 × 3 = 75 → 75 − 60
        assert!((deg(Varga::D9, 1.0) - 9.0).abs() < 1e-9);
        assert!((deg(Varga::D12, 31.25) - 15.0).abs() < 1e-9); // Boğa 1.25 × 12
    }

    #[test]
    fn top_of_sign_stays_in_last_part() {
        let top = 30.0_f64.next_down();
        for v in Varga::ALL {
            let (sign, d) = varga_position(v, top);
            assert_eq!(sign, varga_position(v, 29.999_999).0, "{v:?}");
            assert!((0.0..30.0).contains(&d), "{v:?}: {d}");
        }
    }

    #[test]
    fn serde_uses_division_names() {
        assert_eq!(serde_json::to_string(&Varga::D9).unwrap(), r#""D9""#);
        assert_eq!(
            serde_json::from_str::<Varga>(r#""D12""#).unwrap(),
            Varga::D12
        );
    }
}
```

`crates/jyotish-core/src/lib.rs` içinde `mod proptests;` satırından sonra, `pub mod settings;` satırından önce ekle:

```rust
pub mod varga;
```

ve `pub use zodiac::{…};` satırından önce ekle:

```rust
pub use varga::{varga_position, Varga};
```

`crates/jyotish-core/src/proptests.rs` dosyasının sonuna ekle:

```rust
mod varga_properties {
    use proptest::prelude::*;

    use crate::varga::{varga_position, Varga};
    use crate::zodiac::{normalize, Sign};

    proptest! {
        #[test]
        fn varga_degree_is_in_range(lon in -720.0f64..720.0) {
            for v in Varga::ALL {
                let (_, deg) = varga_position(v, lon);
                prop_assert!((0.0..30.0).contains(&deg), "{v:?} at {lon}: {deg}");
            }
        }

        #[test]
        fn navamsa_matches_the_continuous_formula(lon in 0.0f64..360.0) {
            // BPHS kuralı (hareketli/sabit/çift doğalı başlangıcı) Koç'tan sürekli sayımla
            // aynıdır. Parça sınırında ulp düzeyinde fark olasılığı ihmal edilebilir.
            let continuous = Sign::from_index((normalize(lon) * 9.0 / 30.0) as usize);
            prop_assert_eq!(varga_position(Varga::D9, lon).0, continuous);
        }
    }
}
```

- [ ] **Step 2: Testlerin derlenmediğini doğrula**

Run: `cargo test -p jyotish-core varga`
Expected: FAIL, derleme hatası: `cannot find type 'Varga'`, `cannot find function 'varga_position'`.

- [ ] **Step 3: Uygulamayı yaz**

`crates/jyotish-core/src/varga.rs` dosyasının başına (test modülünün üstüne) ekle:

```rust
//! Shodashavarga: BPHS böl. 6'nın bölümlü haritaları (Parashara tanımları).
//!
//! Her varga, rasi çerçevesindeki boylamı (burç `s`, burç içi derece `d`) bir varga burcuna
//! ve varga içi dereceye eşler. Varga içi derece, grahanın düştüğü parçadaki konumunun
//! 30°'ye ölçeklenmiş hâlidir: parça `[a, b)` ise `(d − a) / (b − a) × 30`. Eşit parçalı
//! vargalarda bu `(d × N) mod 30`'dur. Parça sınırı sonraki parçaya aittir.

use serde::{Deserialize, Serialize};

use crate::zodiac::{deg_in_sign, Sign};

const CANCER: usize = Sign::Cancer as usize;
const LEO: usize = Sign::Leo as usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Varga {
    /// Rasi.
    D1,
    /// Hora (Parashara).
    D2,
    /// Drekkana (Parashara).
    D3,
    /// Chaturthamsa.
    D4,
    /// Saptamsa.
    D7,
    /// Navamsa.
    D9,
    /// Dasamsa.
    D10,
    /// Dwadasamsa.
    D12,
}

impl Varga {
    pub const ALL: [Varga; 8] = [
        Varga::D1,
        Varga::D2,
        Varga::D3,
        Varga::D4,
        Varga::D7,
        Varga::D9,
        Varga::D10,
        Varga::D12,
    ];

    /// Bir burcun kaç parçaya bölündüğü (N).
    pub fn divisions(self) -> u8 {
        match self {
            Self::D1 => 1,
            Self::D2 => 2,
            Self::D3 => 3,
            Self::D4 => 4,
            Self::D7 => 7,
            Self::D9 => 9,
            Self::D10 => 10,
            Self::D12 => 12,
        }
    }
}

/// Tek (oja) burç: Koç, İkizler, Aslan, Terazi, Yay, Kova.
fn is_odd_sign(s: Sign) -> bool {
    s.index().is_multiple_of(2)
}

/// Hareketli, sabit ve çift doğalı burçlar için ayrı başlangıç.
fn by_modality(s: Sign, movable: usize, fixed: usize, dual: usize) -> usize {
    match s.index() % 3 {
        0 => movable,
        1 => fixed,
        _ => dual,
    }
}

/// Eşit parçalı bölüm: `d`'nin düştüğü parça (0..n) ve parça içi derece, [0, 30).
fn equal_part(d: f64, n: u8) -> (usize, f64) {
    let scaled = d * f64::from(n);
    let last = usize::from(n.max(1)) - 1;
    let part = ((scaled / 30.0) as usize).min(last);
    // Yuvarlama 30'a ya da sıfırın altına taşırmasın.
    let within = (scaled - part as f64 * 30.0).clamp(0.0, 30.0_f64.next_down());
    (part, within)
}

/// `lon` (rasi çerçevesi) boylamının `varga`'daki burcu ve varga içi derecesi.
pub fn varga_position(varga: Varga, lon: f64) -> (Sign, f64) {
    let s = Sign::of(lon);
    let d = deg_in_sign(lon);
    let i = s.index();
    let odd = is_odd_sign(s);
    let (part, within) = equal_part(d, varga.divisions());
    let sign = match varga {
        Varga::D1 => i,
        Varga::D2 => {
            if (part == 0) == odd {
                LEO
            } else {
                CANCER
            }
        }
        Varga::D3 => i + 4 * part,
        Varga::D4 => i + 3 * part,
        Varga::D7 => (if odd { i } else { i + 6 }) + part,
        Varga::D9 => by_modality(s, i, i + 8, i + 4) + part,
        Varga::D10 => (if odd { i } else { i + 8 }) + part,
        Varga::D12 => i + part,
    };
    (Sign::from_index(sign), within)
}
```

- [ ] **Step 4: Testlerin geçtiğini doğrula**

Run: `cargo fmt --all && cargo test -p jyotish-core && cargo clippy -p jyotish-core --all-targets -- -D warnings && cargo fmt --all --check`
Expected: `test result: ok. 62 passed` (55 mevcut + 5 varga + 2 proptest). Golden testleri ayrıca `3 passed`. Clippy uyarısız.

- [ ] **Step 5: Commit**

```bash
git add crates/jyotish-core/src
git commit -m "feat(core): Parashari vargas D1-D12" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: D16–D60 kuralları ve eşit olmayan Trimsamsa (D30)

**Files:**
- Modify: `crates/jyotish-core/src/varga.rs`, `crates/jyotish-core/src/proptests.rs`
- Test: `varga.rs` içindeki `mod tests`, `proptests.rs` içindeki `mod varga_properties`

**Interfaces:**
- Consumes: Task 2'deki `varga.rs` yardımcıları (`is_odd_sign`, `by_modality`, `equal_part`).
- Produces: `Varga` 16 varyanta çıkar: `D1, D2, D3, D4, D7, D9, D10, D12, D16, D20, D24, D27, D30, D40, D45, D60`. `Varga::ALL: [Varga; 16]` bu sıradadır. `varga_position` imzası değişmez.

- [ ] **Step 1: Başarısız testleri yaz**

`varga.rs` test modülünde `all_in_order_with_divisions` testini şununla değiştir:

```rust
    #[test]
    fn all_in_order_with_divisions() {
        let n: Vec<u8> = Varga::ALL.iter().map(|v| v.divisions()).collect();
        assert_eq!(
            n,
            [1, 2, 3, 4, 7, 9, 10, 12, 16, 20, 24, 27, 30, 40, 45, 60]
        );
    }
```

ve test modülünün sonuna ekle:

```rust
    #[test]
    fn upper_vargas_follow_parashari_rules() {
        use Sign::*;
        use Varga::*;
        check(&[
            // D16: hareketli Koç'tan, sabit Aslan'dan, çift doğalı Yay'dan. Parça 1.875°.
            (D16, 0.0, Aries),
            (D16, 1.875, Taurus),
            (D16, 30.0, Leo),
            (D16, 60.0, Sagittarius),
            (D16, 29.99, Cancer),
            // D20: hareketli Koç'tan, sabit Yay'dan, çift doğalı Aslan'dan. Parça 1.5°.
            (D20, 0.0, Aries),
            (D20, 1.5, Taurus),
            (D20, 30.0, Sagittarius),
            (D20, 60.0, Leo),
            (D20, 61.5, Virgo),
            // D24: tek burç Aslan'dan, çift burç Yengeç'ten. Parça 1.25°.
            (D24, 0.0, Leo),
            (D24, 1.25, Virgo),
            (D24, 30.0, Cancer),
            (D24, 29.99, Cancer),
            // D27: ateş Koç, toprak Yengeç, hava Terazi, su Oğlak.
            (D27, 0.0, Aries),
            (D27, 1.12, Taurus),
            (D27, 30.0, Cancer),
            (D27, 60.0, Libra),
            (D27, 90.0, Capricorn),
            (D27, 120.0, Aries),
            // D30 tek burç (Koç): Mars 0–5, Satürn 5–10, Jüpiter 10–18, Merkür 18–25, Venüs 25–30.
            (D30, 0.0, Aries),
            (D30, 4.999, Aries),
            (D30, 5.0, Aquarius),
            (D30, 10.0, Sagittarius),
            (D30, 17.999, Sagittarius),
            (D30, 18.0, Gemini),
            (D30, 25.0, Libra),
            (D30, 29.99, Libra),
            // D30 çift burç (Boğa): Venüs 0–5, Merkür 5–12, Jüpiter 12–20, Satürn 20–25, Mars 25–30.
            (D30, 30.0, Taurus),
            (D30, 34.999, Taurus),
            (D30, 35.0, Virgo),
            (D30, 42.0, Pisces),
            (D30, 50.0, Capricorn),
            (D30, 55.0, Scorpio),
            (D30, 59.99, Scorpio),
            // D30 burç değil, teklik/çiftlik izler: Aslan 18° → İkizler.
            (D30, 138.0, Gemini),
            // D40: tek burç Koç'tan, çift burç Terazi'den. Parça 0.75°.
            (D40, 0.0, Aries),
            (D40, 0.75, Taurus),
            (D40, 30.0, Libra),
            (D40, 30.75, Scorpio),
            (D40, 29.9, Cancer),
            // D45: hareketli Koç'tan, sabit Aslan'dan, çift doğalı Yay'dan.
            (D45, 0.0, Aries),
            (D45, 30.0, Leo),
            (D45, 60.0, Sagittarius),
            (D45, 29.9, Sagittarius),
            // D60: burcun kendisinden, s + floor(2d).
            (D60, 0.0, Aries),
            (D60, 0.5, Taurus),
            (D60, 30.0, Taurus),
            (D60, 59.9, Aries),
            (D60, 359.99, Aquarius),
        ]);
    }

    #[test]
    fn trimsamsa_degree_is_proportional_to_its_unequal_part() {
        // Koç 14°: Jüpiter parçası 10–18 → (14 − 10) / 8 × 30 = 15.
        assert!((deg(Varga::D30, 14.0) - 15.0).abs() < 1e-9);
        // Boğa 8.5°: Merkür parçası 5–12 → (8.5 − 5) / 7 × 30 = 15.
        assert!((deg(Varga::D30, 38.5) - 15.0).abs() < 1e-9);
        // D60 eşit parça: 0.25 × 60 = 15.
        assert!((deg(Varga::D60, 0.25) - 15.0).abs() < 1e-9);
    }
```

`proptests.rs` içindeki `mod varga_properties` `proptest! { … }` bloğunun içine, `navamsa_matches_the_continuous_formula`'dan sonra ekle:

```rust
        #[test]
        fn bhamsa_matches_the_continuous_formula(lon in 0.0f64..360.0) {
            let continuous = Sign::from_index((normalize(lon) * 27.0 / 30.0) as usize);
            prop_assert_eq!(varga_position(Varga::D27, lon).0, continuous);
        }

        #[test]
        fn trimsamsa_never_falls_in_luminary_signs(lon in 0.0f64..360.0) {
            // Güneş ve Ay'ın trimsamsası yoktur. Tek burçta lordların tek burcu, çift burçta
            // çift burcu gelir.
            let (sign, _) = varga_position(Varga::D30, lon);
            prop_assert!(sign != Sign::Cancer && sign != Sign::Leo, "{sign:?}");
            let odd = Sign::of(lon).index().is_multiple_of(2);
            prop_assert_eq!(sign.index().is_multiple_of(2), odd);
        }
```

- [ ] **Step 2: Testlerin derlenmediğini doğrula**

Run: `cargo test -p jyotish-core varga`
Expected: FAIL, derleme hatası: `no variant named 'D16'` (ve diğer yeni varyantlar).

- [ ] **Step 3: Uygulamayı yaz**

`varga.rs` içinde `use crate::zodiac::{deg_in_sign, Sign};` satırından `#[cfg(test)]` satırına kadar olan her şeyi şununla değiştir:

```rust
use crate::zodiac::{deg_in_sign, Sign};

const ARIES: usize = Sign::Aries as usize;
const CANCER: usize = Sign::Cancer as usize;
const LEO: usize = Sign::Leo as usize;
const LIBRA: usize = Sign::Libra as usize;
const SAGITTARIUS: usize = Sign::Sagittarius as usize;
const CAPRICORN: usize = Sign::Capricorn as usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Varga {
    /// Rasi.
    D1,
    /// Hora (Parashara).
    D2,
    /// Drekkana (Parashara).
    D3,
    /// Chaturthamsa.
    D4,
    /// Saptamsa.
    D7,
    /// Navamsa.
    D9,
    /// Dasamsa.
    D10,
    /// Dwadasamsa.
    D12,
    /// Shodasamsa.
    D16,
    /// Vimsamsa.
    D20,
    /// Chaturvimsamsa (Siddhamsa).
    D24,
    /// Saptavimsamsa (Bhamsa).
    D27,
    /// Trimsamsa (eşit olmayan parçalar).
    D30,
    /// Khavedamsa.
    D40,
    /// Akshavedamsa.
    D45,
    /// Shashtyamsa.
    D60,
}

impl Varga {
    /// BPHS'nin Shodashavarga sırası.
    pub const ALL: [Varga; 16] = [
        Varga::D1,
        Varga::D2,
        Varga::D3,
        Varga::D4,
        Varga::D7,
        Varga::D9,
        Varga::D10,
        Varga::D12,
        Varga::D16,
        Varga::D20,
        Varga::D24,
        Varga::D27,
        Varga::D30,
        Varga::D40,
        Varga::D45,
        Varga::D60,
    ];

    /// Bir burcun kaç parçaya bölündüğü (N). D30'da parçalar eşit değildir.
    pub fn divisions(self) -> u8 {
        match self {
            Self::D1 => 1,
            Self::D2 => 2,
            Self::D3 => 3,
            Self::D4 => 4,
            Self::D7 => 7,
            Self::D9 => 9,
            Self::D10 => 10,
            Self::D12 => 12,
            Self::D16 => 16,
            Self::D20 => 20,
            Self::D24 => 24,
            Self::D27 => 27,
            Self::D30 => 30,
            Self::D40 => 40,
            Self::D45 => 45,
            Self::D60 => 60,
        }
    }
}

/// Tek (oja) burç: Koç, İkizler, Aslan, Terazi, Yay, Kova.
fn is_odd_sign(s: Sign) -> bool {
    s.index().is_multiple_of(2)
}

/// Hareketli, sabit ve çift doğalı burçlar için ayrı başlangıç.
fn by_modality(s: Sign, movable: usize, fixed: usize, dual: usize) -> usize {
    match s.index() % 3 {
        0 => movable,
        1 => fixed,
        _ => dual,
    }
}

/// D27 başlangıcı: ateş Koç, toprak Yengeç, hava Terazi, su Oğlak.
fn by_element(s: Sign) -> usize {
    match s.index() % 4 {
        0 => ARIES,
        1 => CANCER,
        2 => LIBRA,
        _ => CAPRICORN,
    }
}

/// Eşit parçalı bölüm: `d`'nin düştüğü parça (0..n) ve parça içi derece, [0, 30).
fn equal_part(d: f64, n: u8) -> (usize, f64) {
    let scaled = d * f64::from(n);
    let last = usize::from(n.max(1)) - 1;
    let part = ((scaled / 30.0) as usize).min(last);
    // Yuvarlama 30'a ya da sıfırın altına taşırmasın.
    let within = (scaled - part as f64 * 30.0).clamp(0.0, 30.0_f64.next_down());
    (part, within)
}

/// Trimsamsa, tek burç: (parça bitişi, varga burcu). Mars Koç, Satürn Kova, Jüpiter Yay,
/// Merkür İkizler, Venüs Terazi.
const TRIMSAMSA_ODD: [(f64, Sign); 5] = [
    (5.0, Sign::Aries),
    (10.0, Sign::Aquarius),
    (18.0, Sign::Sagittarius),
    (25.0, Sign::Gemini),
    (30.0, Sign::Libra),
];

/// Trimsamsa, çift burç (ters sıra): Venüs Boğa, Merkür Başak, Jüpiter Balık, Satürn Oğlak,
/// Mars Akrep.
const TRIMSAMSA_EVEN: [(f64, Sign); 5] = [
    (5.0, Sign::Taurus),
    (12.0, Sign::Virgo),
    (20.0, Sign::Pisces),
    (25.0, Sign::Capricorn),
    (30.0, Sign::Scorpio),
];

fn trimsamsa(d: f64, odd: bool) -> (Sign, f64) {
    let table = if odd { &TRIMSAMSA_ODD } else { &TRIMSAMSA_EVEN };
    let mut from = 0.0;
    for &(to, sign) in table {
        if d < to {
            let within = ((d - from) / (to - from) * 30.0).clamp(0.0, 30.0_f64.next_down());
            return (sign, within);
        }
        from = to;
    }
    // `d` her zaman 30'dan küçüktür; NaN gibi durumlar son parçaya düşer.
    (table[4].1, 30.0_f64.next_down())
}

/// `lon` (rasi çerçevesi) boylamının `varga`'daki burcu ve varga içi derecesi.
pub fn varga_position(varga: Varga, lon: f64) -> (Sign, f64) {
    let s = Sign::of(lon);
    let d = deg_in_sign(lon);
    let i = s.index();
    let odd = is_odd_sign(s);
    let (part, within) = equal_part(d, varga.divisions());
    let sign = match varga {
        Varga::D1 => i,
        Varga::D2 => {
            if (part == 0) == odd {
                LEO
            } else {
                CANCER
            }
        }
        Varga::D3 => i + 4 * part,
        Varga::D4 => i + 3 * part,
        Varga::D7 => (if odd { i } else { i + 6 }) + part,
        Varga::D9 => by_modality(s, i, i + 8, i + 4) + part,
        Varga::D10 => (if odd { i } else { i + 8 }) + part,
        Varga::D12 => i + part,
        Varga::D16 => by_modality(s, ARIES, LEO, SAGITTARIUS) + part,
        Varga::D20 => by_modality(s, ARIES, SAGITTARIUS, LEO) + part,
        Varga::D24 => (if odd { LEO } else { CANCER }) + part,
        Varga::D27 => by_element(s) + part,
        Varga::D30 => return trimsamsa(d, odd),
        Varga::D40 => (if odd { ARIES } else { LIBRA }) + part,
        Varga::D45 => by_modality(s, ARIES, LEO, SAGITTARIUS) + part,
        Varga::D60 => i + part,
    };
    (Sign::from_index(sign), within)
}

```

- [ ] **Step 4: Testlerin geçtiğini doğrula**

Run: `cargo fmt --all && cargo test -p jyotish-core && cargo clippy -p jyotish-core --all-targets -- -D warnings && cargo fmt --all --check`
Expected: `test result: ok. 66 passed` (62 + 2 varga + 2 proptest). Golden `3 passed`. Clippy uyarısız.

- [ ] **Step 5: Commit**

```bash
git add crates/jyotish-core/src
git commit -m "feat(core): Parashari vargas D16-D60 with unequal trimsamsa" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: `VargaChart`, `Chart.vargas` ve vargottama

**Files:**
- Modify: `crates/jyotish-core/src/varga.rs`, `crates/jyotish-core/src/graha.rs`, `crates/jyotish-core/src/chart.rs`, `crates/jyotish-core/src/lib.rs`
- Modify (üretilir): `app/ui/bindings.ts`
- Test: `varga.rs` ve `chart.rs` içindeki `mod tests`

**Interfaces:**
- Consumes: `varga_position`, `Varga::ALL` (Task 3); `Graha` (Plan 05).
- Produces:
  - `pub struct VargaPlacement { pub graha: Graha, pub sign: Sign, pub deg: f64 }`
  - `pub struct VargaChart { pub varga: Varga, pub lagna_sign: Sign, pub placements: Vec<VargaPlacement> }`
  - `pub fn varga_chart(varga: Varga, lagna_lon: f64, grahas: &[(Graha, f64)]) -> VargaChart`
  - `pub fn all_vargas(lagna_lon: f64, grahas: &[(Graha, f64)]) -> Vec<VargaChart>` (`Varga::ALL` sırası)
  - `pub fn is_vargottama(lon: f64) -> bool`
  - `GrahaPosition.vargottama: bool` (son alan)
  - `Chart.vargas: Vec<VargaChart>` (`bhavas`'tan sonra), her zaman 16 eleman, rasi çerçevesinden

- [ ] **Step 1: Başarısız testleri yaz**

`varga.rs` test modülünün sonuna ekle:

```rust
    #[test]
    fn varga_chart_places_every_graha_in_order() {
        let grahas = [(Graha::Sun, 346.0), (Graha::Moon, 71.0)];
        let c = varga_chart(Varga::D9, 46.0, &grahas);
        assert_eq!(c.varga, Varga::D9);
        // Lagna Boğa 16°: floor(46 × 9 / 30) = 13 → Boğa.
        assert_eq!(c.lagna_sign, Sign::Taurus);
        assert_eq!(c.placements.len(), 2);
        assert_eq!(
            (c.placements[0].graha, c.placements[0].sign),
            (Graha::Sun, Sign::Scorpio)
        );
        assert_eq!(
            (c.placements[1].graha, c.placements[1].sign),
            (Graha::Moon, Sign::Capricorn)
        );
        // Güneş Balık 16°: 16 × 9 = 144 → 4. parça, parça içi 144 − 120 = 24°.
        assert!((c.placements[0].deg - 24.0).abs() < 1e-9);
        let all = all_vargas(46.0, &grahas);
        assert_eq!(all.iter().map(|v| v.varga).collect::<Vec<_>>(), Varga::ALL);
    }

    #[test]
    fn vargottama_compares_d1_and_d9() {
        assert!(is_vargottama(0.0)); // Koç 0°: D9 Koç
        assert!(is_vargottama(226.0)); // Akrep 16°: floor(67.8) = 67 → Akrep
        assert!(is_vargottama(359.9)); // Balık'ın son navamsası Balık
        assert!(!is_vargottama(346.0)); // Balık 16°: D9 Akrep
    }
```

`chart.rs` içindeki `mod tests`'e, `use crate::settings::NodeKind;` satırından sonra ekle:

```rust
    use crate::varga::Varga;
```

ve test modülünün sonuna ekle:

```rust
    #[test]
    fn vargas_cover_all_sixteen_from_the_rasi_frame() {
        let c = chart(&mut FakeEph::default(), &CalcSettings::default());
        let order: Vec<Varga> = c.vargas.iter().map(|v| v.varga).collect();
        assert_eq!(order, Varga::ALL);
        let d1 = &c.vargas[0];
        assert_eq!(d1.lagna_sign, c.lagna.sign);
        for (p, g) in d1.placements.iter().zip(&c.grahas) {
            assert_eq!((p.graha, p.sign), (g.graha, g.sign));
            assert!((p.deg - g.deg_in_sign).abs() < 1e-12);
        }
        let d9 = c.vargas.iter().find(|v| v.varga == Varga::D9).unwrap();
        assert_eq!(d9.lagna_sign, Sign::Taurus);
        // Güneş sidereal 346°: floor(103.8) = 103 → Akrep.
        assert_eq!(d9.placements[0].sign, Sign::Scorpio);

        // Tropikal-rasi kipinde vargalar tropikal boylamdan: Güneş 10° → floor(3.0) = 3 → Yengeç.
        let t = chart(&mut FakeEph::default(), &tropical_rasi());
        let d9 = t.vargas.iter().find(|v| v.varga == Varga::D9).unwrap();
        assert_eq!(d9.placements[0].sign, Sign::Cancer);
    }

    #[test]
    fn vargottama_flag_compares_d1_and_d9() {
        let c = chart(&mut FakeEph::default(), &CalcSettings::default());
        // Sidereal: Güneş 346, Ay 71, Mars 176, Merkür 356, Jüpiter 226, Venüs 16, Satürn 276,
        // Rahu 96, Ketu 276. Yalnızca Jüpiter (Akrep 16°) D9'da da Akrep'tir.
        let vargottama: Vec<Graha> = c
            .grahas
            .iter()
            .filter(|p| p.vargottama)
            .map(|p| p.graha)
            .collect();
        assert_eq!(vargottama, [Graha::Jupiter]);
    }
```

- [ ] **Step 2: Testlerin derlenmediğini doğrula**

Run: `cargo test -p jyotish-core varga`
Expected: FAIL, derleme hatası: `cannot find function 'varga_chart'`, `no field 'vargas' on type 'Chart'`, `no field 'vargottama'`.

- [ ] **Step 3: Uygulamayı yaz**

`varga.rs` içinde `use serde::{Deserialize, Serialize};` satırından sonra ekle:

```rust

use crate::graha::Graha;
```

`varga_position` fonksiyonundan sonra (test modülünden önce) ekle:

```rust
/// Bir grahanın varga burcu ve varga içi derecesi (spec §5 `(graha, sign, deg)`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct VargaPlacement {
    pub graha: Graha,
    pub sign: Sign,
    /// Varga içi derece, [0, 30): parçadaki konum 30°'ye ölçeklenmiş (modül belgesi).
    pub deg: f64,
}

/// Bir varga haritası (spec §5 `VargaChart`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct VargaChart {
    pub varga: Varga,
    pub lagna_sign: Sign,
    /// Girdi sırasında; haritada `Graha::ALL` sırası.
    pub placements: Vec<VargaPlacement>,
}

/// `lagna_lon` ve `grahas` (rasi çerçevesi boylamları) için tek bir varga haritası.
pub fn varga_chart(varga: Varga, lagna_lon: f64, grahas: &[(Graha, f64)]) -> VargaChart {
    VargaChart {
        varga,
        lagna_sign: varga_position(varga, lagna_lon).0,
        placements: grahas
            .iter()
            .map(|&(graha, lon)| {
                let (sign, deg) = varga_position(varga, lon);
                VargaPlacement { graha, sign, deg }
            })
            .collect(),
    }
}

/// 16 varganın tamamı, `Varga::ALL` sırasında.
pub fn all_vargas(lagna_lon: f64, grahas: &[(Graha, f64)]) -> Vec<VargaChart> {
    Varga::ALL
        .iter()
        .map(|&v| varga_chart(v, lagna_lon, grahas))
        .collect()
}

/// Vargottama: D1 ve D9 burcu aynı.
pub fn is_vargottama(lon: f64) -> bool {
    varga_position(Varga::D1, lon).0 == varga_position(Varga::D9, lon).0
}
```

`graha.rs` içinde `GrahaPosition`'ın `pub house: u8,` alanından sonra ekle:

```rust
    /// D1 ve D9 burcu aynı (rasi çerçevesinde).
    pub vargottama: bool,
```

`chart.rs`:

1. `use crate::settings::{CalcSettings, HouseSystem, ZodiacMode};` satırından sonra ekle:

```rust
use crate::varga::{all_vargas, is_vargottama, VargaChart};
```

2. `Chart` struct'ının belge yorumunu ve `bhavas` alanını şununla değiştir:

```rust
/// M3 haritası (Plan 06): vargalar ve Vimshottari dahil. Panchang ve ashtakavarga
/// Plan 07'de eklenir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Chart {
    pub meta: ChartMeta,
    pub jd_ut: f64,
    pub geo_lat: f64,
    pub geo_lon: f64,
    /// Nütasyon dahil (gerçek) ayanamsa, derece.
    pub ayanamsa_deg: f64,
    pub lagna: Lagna,
    /// Her zaman 9 eleman, `Graha::ALL` sırasında.
    pub grahas: Vec<GrahaPosition>,
    pub bhavas: Bhavas,
    /// Her zaman 16 eleman, `Varga::ALL` sırasında; rasi çerçevesindeki boylamlardan.
    pub vargas: Vec<VargaChart>,
}
```

3. `GrahaPosition { … }` kurulumunda `house: match … },` ifadesinden sonra ekle:

```rust
                vargottama: is_vargottama(r.lon),
```

4. `let grahas = raw` satırını `let grahas: Vec<GrahaPosition> = raw` olarak değiştir. `Ok(Chart {` satırından hemen önce ekle:

```rust
    let points: Vec<(Graha, f64)> = raw.iter().map(|r| (r.graha, r.lon)).collect();
    let vargas = all_vargas(lagna.lon, &points);
```

ve `Chart { … }` içinde `bhavas: Bhavas { … },` alanından sonra ekle:

```rust
        vargas,
```

`lib.rs`:

- Modül belgesini şununla değiştir:

```rust
//! Saf, yan etkisiz Jyotish hesabı. Ephemeris'e yalnızca `ephemeris::Ephemeris`
//! trait'i üzerinden erişir; I/O, Tauri ve atlas bağımlılığı yoktur.
//!
//! Kapsam: ayanamsa, grahalar, lagna, bhavalar, nakshatra, dignity, yanma (M2), 16 varga
//! ve Vimshottari dasha (M3, Plan 06). Panchang ve ashtakavarga Plan 07'de eklenir.
```

- `pub use varga::{varga_position, Varga};` satırını şununla değiştir:

```rust
pub use varga::{
    all_vargas, is_vargottama, varga_chart, varga_position, Varga, VargaChart, VargaPlacement,
};
```

- [ ] **Step 4: Çekirdek testlerinin geçtiğini doğrula**

Run: `cargo fmt --all && cargo test -p jyotish-core && cargo clippy -p jyotish-core --all-targets -- -D warnings`
Expected: `test result: ok. 70 passed` (66 + 2 varga + 2 chart). Golden `3 passed`.

- [ ] **Step 5: Bağlamaları yeniden üret**

Run:

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
cargo test -p jyotish-desk --test bindings
grep -n "vargas: VargaChart\[\]\|vargottama: boolean\|export type VargaChart = {\|export type Varga = " app/ui/bindings.ts
```

Expected: İki test de geçer. `grep` dört satır basar.

- [ ] **Step 6: Tam doğrulama**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && pnpm --dir app build`
Expected: Hepsi geçer.

- [ ] **Step 7: Commit**

```bash
git add crates/jyotish-core/src app/ui/bindings.ts
git commit -m "feat(core): all 16 varga charts and vargottama on every chart" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: `dasha.rs`: Vimshottari dönemleri ve `dasha_children`

**Files:**
- Create: `crates/jyotish-core/src/dasha.rs`
- Modify: `crates/jyotish-core/src/error.rs`, `crates/jyotish-core/src/settings.rs`, `crates/jyotish-core/src/lib.rs`
- Modify: `app/src-tauri/src/commands.rs` (`From<CoreError>` eşlemesi; `match` tam olmalı)
- Test: `dasha.rs` ve `settings.rs` içindeki `mod tests`; `commands.rs` içindeki `core_error_codes_are_stable`

**Interfaces:**
- Consumes: `Graha` (Plan 05), `nakshatra_pada`, `normalize`, `NAKSHATRA_SPAN_DEG` (`zodiac.rs`).
- Produces:
  - `pub const DASHA_YEAR_RANGE: RangeInclusive<f64> = 300.0..=400.0` (`settings.rs`); `CalcSettings::validate` `dasha_year`'ı bununla denetler.
  - `CoreError::InvalidDashaInput(&'static str)`; alan adları `"birth_jd_ut"`, `"moon_sidereal_lon"`, `"year_days"`, `"path"`. Komut kodu `invalid_dasha_input {field}`.
  - `pub const VIMSHOTTARI_ORDER: [Graha; 9]`, `pub const VIMSHOTTARI_TOTAL_YEARS: f64 = 120.0`, `pub const MAX_DASHA_LEVEL: u8 = 5`, `pub fn vimshottari_years(lord: Graha) -> f64`
  - `pub struct DashaSeed { pub birth_jd_ut: f64, pub moon_sidereal_lon: f64, pub year_days: f64 }` (`Copy`), `pub fn validate(&self) -> Result<(), CoreError>`
  - `pub struct DashaPeriod { pub lords: Vec<Graha>, pub level: u8, pub start_jd_ut: f64, pub end_jd_ut: f64, pub has_children: bool }`
  - `pub fn birth_dasha(moon_sidereal_lon: f64) -> (Graha, f64)`: (doğum mahadashası lordu, geçen kesir [0, 1])
  - `pub fn mahadashas(seed: &DashaSeed) -> Result<Vec<DashaPeriod>, CoreError>`
  - `pub fn dasha_children(seed: &DashaSeed, path: &[Graha]) -> Result<Vec<DashaPeriod>, CoreError>`

- [ ] **Step 1: Başarısız testleri yaz**

`crates/jyotish-core/src/dasha.rs` (önce yalnızca test modülü):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const JD: f64 = 2_451_545.0;

    fn seed(moon: f64) -> DashaSeed {
        DashaSeed {
            birth_jd_ut: JD,
            moon_sidereal_lon: moon,
            year_days: 365.25,
        }
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn last_lords(periods: &[DashaPeriod]) -> Vec<Graha> {
        periods.iter().map(|p| *p.lords.last().unwrap()).collect()
    }

    #[test]
    fn order_and_years_total_120() {
        assert_eq!(
            VIMSHOTTARI_ORDER.map(vimshottari_years),
            [7.0, 20.0, 6.0, 10.0, 7.0, 18.0, 16.0, 19.0, 17.0]
        );
        let total: f64 = VIMSHOTTARI_ORDER.iter().map(|&g| vimshottari_years(g)).sum();
        assert_eq!(total, VIMSHOTTARI_TOTAL_YEARS);
    }

    #[test]
    fn birth_dasha_follows_the_moon_nakshatra() {
        assert_eq!(birth_dasha(0.0), (Graha::Ketu, 0.0));
        assert_eq!(birth_dasha(NAKSHATRA_SPAN_DEG / 2.0), (Graha::Ketu, 0.5));
        assert_eq!(birth_dasha(NAKSHATRA_SPAN_DEG), (Graha::Venus, 0.0));
        // Magha (10.) ve Mula (19.) yine Ketu ile başlar.
        assert_eq!(birth_dasha(9.0 * NAKSHATRA_SPAN_DEG + 1.0).0, Graha::Ketu);
        assert_eq!(birth_dasha(18.0 * NAKSHATRA_SPAN_DEG + 1.0).0, Graha::Ketu);
        // Revati (27.) Merkür; sona çok yakın.
        let (lord, elapsed) = birth_dasha(359.999_999);
        assert_eq!(lord, Graha::Mercury);
        assert!(elapsed > 0.999_99 && elapsed <= 1.0, "{elapsed}");
        // Golden new-delhi Ay'ı: 199.470529 / 13.3̄ = 14.96029 → Swati (15.), Rahu,
        // kesir 0.9602897.
        let (lord, elapsed) = birth_dasha(199.470_529);
        assert_eq!(lord, Graha::Rahu);
        assert!((elapsed - 0.960_289_675).abs() < 1e-9, "{elapsed}");
    }

    #[test]
    fn mahadashas_start_before_birth_and_cover_120_years() {
        let m = mahadashas(&seed(NAKSHATRA_SPAN_DEG / 2.0)).unwrap();
        assert_eq!(last_lords(&m), VIMSHOTTARI_ORDER);
        assert!(m
            .iter()
            .all(|p| p.level == 1 && p.has_children && p.lords.len() == 1));
        // Ketu'nun yarısı geçmiş: 0.5 × 7 × 365.25 = 1278.375 gün önce başlar.
        assert!(close(m[0].start_jd_ut, 2_450_266.625));
        assert!(close(m[0].end_jd_ut, 2_452_823.375));
        // Venüs bitişi: başlangıç + 27 × 365.25.
        assert!(close(m[1].end_jd_ut, 2_460_128.375));
        assert!(close(m[8].end_jd_ut, 2_450_266.625 + 120.0 * 365.25));
    }

    #[test]
    fn sub_periods_start_from_the_parent_lord_and_split_by_years() {
        let s = seed(NAKSHATRA_SPAN_DEG / 2.0);
        let antar = dasha_children(&s, &[Graha::Ketu]).unwrap();
        assert_eq!(last_lords(&antar), VIMSHOTTARI_ORDER);
        assert!(antar
            .iter()
            .all(|p| p.level == 2 && p.lords[0] == Graha::Ketu));
        // Ketu mahadashası 7 × 365.25 = 2556.75 gün; Ke-Ke 2556.75 × 7 / 120 = 149.14375 gün.
        assert!(close(antar[0].start_jd_ut, 2_450_266.625));
        assert!(close(antar[0].end_jd_ut, 2_450_415.768_75));
        // Ke-Ve bitişi: 2556.75 × 27 / 120 = 575.26875 gün.
        assert!(close(antar[1].end_jd_ut, 2_450_841.893_75));
        let venus = dasha_children(&s, &[Graha::Venus]).unwrap();
        assert_eq!(
            last_lords(&venus),
            [
                Graha::Venus,
                Graha::Sun,
                Graha::Moon,
                Graha::Mars,
                Graha::Rahu,
                Graha::Jupiter,
                Graha::Saturn,
                Graha::Mercury,
                Graha::Ketu
            ]
        );
    }

    #[test]
    fn deeper_levels_and_path_limits() {
        let s = seed(100.0);
        assert_eq!(dasha_children(&s, &[]).unwrap(), mahadashas(&s).unwrap());
        let path = [Graha::Rahu, Graha::Jupiter, Graha::Saturn, Graha::Mercury];
        let prana = dasha_children(&s, &path).unwrap();
        assert_eq!(prana.len(), 9);
        assert!(prana
            .iter()
            .all(|p| p.level == 5 && !p.has_children && p.lords[..4] == path));
        assert_eq!(
            dasha_children(&s, &[Graha::Sun; 5]),
            Err(CoreError::InvalidDashaInput("path"))
        );
    }

    #[test]
    fn year_length_scales_every_period() {
        let m = mahadashas(&DashaSeed {
            year_days: 360.0,
            ..seed(0.0)
        })
        .unwrap();
        assert!(close(m[0].start_jd_ut, JD));
        assert!(close(m[8].end_jd_ut - m[0].start_jd_ut, 43_200.0));
        let m = mahadashas(&DashaSeed {
            year_days: 365.2422,
            ..seed(0.0)
        })
        .unwrap();
        assert!(close(m[0].end_jd_ut, JD + 7.0 * 365.2422));
    }

    #[test]
    fn invalid_seed_is_rejected() {
        let bad = |s: DashaSeed| mahadashas(&s).unwrap_err();
        assert_eq!(
            bad(DashaSeed {
                birth_jd_ut: f64::NAN,
                ..seed(0.0)
            }),
            CoreError::InvalidDashaInput("birth_jd_ut")
        );
        assert_eq!(
            bad(DashaSeed {
                moon_sidereal_lon: f64::INFINITY,
                ..seed(0.0)
            }),
            CoreError::InvalidDashaInput("moon_sidereal_lon")
        );
        for y in [0.0, 299.0, 1e308, f64::NAN] {
            assert_eq!(
                bad(DashaSeed {
                    year_days: y,
                    ..seed(0.0)
                }),
                CoreError::InvalidDashaInput("year_days"),
                "{y}"
            );
        }
    }
}
```

`settings.rs` test modülünün sonuna ekle:

```rust
    #[test]
    fn validate_rejects_out_of_range_dasha_year() {
        let with = |dasha_year: f64| {
            CalcSettings {
                dasha_year,
                ..CalcSettings::default()
            }
            .validate()
        };
        for ok in [360.0, 365.2422, 365.25] {
            assert_eq!(with(ok), Ok(()), "{ok}");
        }
        for bad in [299.9, 400.1, 1e308, f64::NAN, f64::INFINITY] {
            assert_eq!(
                with(bad),
                Err(CoreError::InvalidSettings("dasha_year")),
                "{bad}"
            );
        }
    }
```

`app/src-tauri/src/commands.rs` içinde `core_error_codes_are_stable` testinde `let e = CommandError::from(CoreError::NonFiniteResult("ayanamsa"));` satırından önce ekle:

```rust
        let e = CommandError::from(CoreError::InvalidDashaInput("path"));
        assert_eq!(e.code, "invalid_dasha_input");
        assert_eq!(e.params["field"], "path");
```

`lib.rs` içinde `mod combustion;` satırından sonra ekle:

```rust
pub mod dasha;
```

ve `pub use combustion::is_combust;` satırından sonra ekle:

```rust
pub use dasha::{
    birth_dasha, dasha_children, mahadashas, vimshottari_years, DashaPeriod, DashaSeed,
    MAX_DASHA_LEVEL, VIMSHOTTARI_ORDER, VIMSHOTTARI_TOTAL_YEARS,
};
```

`lib.rs`'deki `pub use settings::{…}` listesi değişmez; `DASHA_YEAR_RANGE` `jyotish_core::settings::DASHA_YEAR_RANGE` yoluyla erişilir (`settings` modülü zaten `pub`).

- [ ] **Step 2: Testlerin derlenmediğini doğrula**

Run: `cargo test -p jyotish-core dasha`
Expected: FAIL, derleme hatası: `cannot find type 'DashaSeed'`, `no variant named 'InvalidDashaInput'`.

- [ ] **Step 3: Uygulamayı yaz**

`error.rs` içinde `NonFiniteResult` varyantından sonra ekle:

```rust
    #[error("invalid dasha input: {0}")]
    InvalidDashaInput(&'static str),
```

`settings.rs` içinde `pub const CUSTOM_T0_JD_RANGE …` satırından sonra ekle:

```rust

/// Kabul edilen Vimshottari yıl uzunluğu, gün. Spec §6 seçenekleri 360, 365.2422 ve
/// 365.25 bu aralıktadır; dışı sonsuz ya da anlamsız tarihler üretir.
pub const DASHA_YEAR_RANGE: std::ops::RangeInclusive<f64> = 300.0..=400.0;
```

ve `validate` içindeki `dasha_year` denetimini şununla değiştir:

```rust
        // NaN ve sonsuz aralık dışında kalır.
        if !DASHA_YEAR_RANGE.contains(&self.dasha_year) {
            return Err(CoreError::InvalidSettings("dasha_year"));
        }
```

`dasha.rs` dosyasının başına (test modülünün üstüne) ekle:

```rust
//! Vimshottari dasha (BPHS, nakshatra dashaları; Santhanam çevirisinde böl. 46).
//!
//! Doğumdaki Ay'ın sidereal nakshatrası ilk mahadashanın lordunu ve geçen kesrini
//! belirler. Her dönemin dokuz alt dönemi ebeveynin lordundan başlar ve ebeveyn süresini
//! `yıl(L) / 120` oranıyla böler. Zamanlar UT Jülyen günüdür.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::settings::DASHA_YEAR_RANGE;
use crate::zodiac::{nakshatra_pada, normalize, NAKSHATRA_SPAN_DEG};
use crate::CoreError;

pub const VIMSHOTTARI_TOTAL_YEARS: f64 = 120.0;
/// 1 mahadasha, 2 antardasha, 3 pratyantardasha, 4 sookshma, 5 prana.
pub const MAX_DASHA_LEVEL: u8 = 5;

/// Ashwini'den başlayarak nakshatra lordları; sıra her 9 nakshatrada tekrar eder.
pub const VIMSHOTTARI_ORDER: [Graha; 9] = [
    Graha::Ketu,
    Graha::Venus,
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Rahu,
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Mercury,
];

/// Mahadasha süresi, yıl.
pub fn vimshottari_years(lord: Graha) -> f64 {
    match lord {
        Graha::Ketu => 7.0,
        Graha::Venus => 20.0,
        Graha::Sun => 6.0,
        Graha::Moon => 10.0,
        Graha::Mars => 7.0,
        Graha::Rahu => 18.0,
        Graha::Jupiter => 16.0,
        Graha::Saturn => 19.0,
        Graha::Mercury => 17.0,
    }
}

fn order_index(lord: Graha) -> usize {
    match lord {
        Graha::Ketu => 0,
        Graha::Venus => 1,
        Graha::Sun => 2,
        Graha::Moon => 3,
        Graha::Mars => 4,
        Graha::Rahu => 5,
        Graha::Jupiter => 6,
        Graha::Saturn => 7,
        Graha::Mercury => 8,
    }
}

/// Dasha ağacını tek başına belirleyen girdiler.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DashaSeed {
    /// Doğum anı, UT Jülyen günü.
    pub birth_jd_ut: f64,
    /// Doğumdaki Ay'ın sidereal boylamı (her zodyak kipinde sidereal).
    pub moon_sidereal_lon: f64,
    /// Yıl uzunluğu, gün (`CalcSettings::dasha_year`).
    pub year_days: f64,
}

impl DashaSeed {
    pub fn validate(&self) -> Result<(), CoreError> {
        if !self.birth_jd_ut.is_finite() {
            return Err(CoreError::InvalidDashaInput("birth_jd_ut"));
        }
        if !self.moon_sidereal_lon.is_finite() {
            return Err(CoreError::InvalidDashaInput("moon_sidereal_lon"));
        }
        // NaN ve sonsuz aralık dışında kalır.
        if !DASHA_YEAR_RANGE.contains(&self.year_days) {
            return Err(CoreError::InvalidDashaInput("year_days"));
        }
        Ok(())
    }
}

/// Bir dasha dönemi (spec §5 `DashaPeriod`). Zamanlar UT Jülyen günüdür.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DashaPeriod {
    /// Kökten bu döneme lordlar: `[maha]`, `[maha, antar]`, …
    pub lords: Vec<Graha>,
    /// 1 mahadasha … 5 prana; `lords.len()` ile aynı.
    pub level: u8,
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
    /// `dasha_children` ile alt dönem istenebilir (`level < MAX_DASHA_LEVEL`).
    pub has_children: bool,
}

/// Doğum mahadashasının lordu ve geçen kesri, [0, 1]. Lord `nakshatra_pada` ile aynı
/// nakshatradan seçilir; gösterilen nakshatra ile dasha lordu asla ayrışmaz.
pub fn birth_dasha(moon_sidereal_lon: f64) -> (Graha, f64) {
    let lon = normalize(moon_sidereal_lon);
    let (nakshatra, _) = nakshatra_pada(lon);
    let index = usize::from(nakshatra).saturating_sub(1);
    let elapsed = (lon - index as f64 * NAKSHATRA_SPAN_DEG) / NAKSHATRA_SPAN_DEG;
    (VIMSHOTTARI_ORDER[index % 9], elapsed.clamp(0.0, 1.0))
}

/// `[start, end)` aralığını `first`'ten başlayan dokuz alt döneme böler. Bitişler
/// birikimli yıldan hesaplanır; sonuncusu `end`'e tam eşittir.
fn subdivide(path: &[Graha], first: Graha, start: f64, end: f64) -> Vec<DashaPeriod> {
    let level = (path.len() + 1) as u8;
    let span = end - start;
    let first = order_index(first);
    let mut periods = Vec::with_capacity(9);
    let mut from = start;
    let mut years = 0.0;
    for k in 0..9 {
        let lord = VIMSHOTTARI_ORDER[(first + k) % 9];
        years += vimshottari_years(lord);
        let to = if k == 8 {
            end
        } else {
            start + span * years / VIMSHOTTARI_TOTAL_YEARS
        };
        let mut lords = path.to_vec();
        lords.push(lord);
        periods.push(DashaPeriod {
            lords,
            level,
            start_jd_ut: from,
            end_jd_ut: to,
            has_children: level < MAX_DASHA_LEVEL,
        });
        from = to;
    }
    periods
}

/// Dönemin dokuz alt dönemi; 5. seviyede boş.
fn children_of(period: &DashaPeriod) -> Vec<DashaPeriod> {
    match period.lords.last() {
        Some(&lord) if period.has_children => {
            subdivide(&period.lords, lord, period.start_jd_ut, period.end_jd_ut)
        }
        _ => Vec::new(),
    }
}

/// Dokuz mahadasha. İlki doğumdan `f × yıl(lord) × year_days` gün önce başlar; toplam
/// 120 yıl.
pub fn mahadashas(seed: &DashaSeed) -> Result<Vec<DashaPeriod>, CoreError> {
    seed.validate()?;
    let (lord, elapsed) = birth_dasha(seed.moon_sidereal_lon);
    let start = seed.birth_jd_ut - elapsed * vimshottari_years(lord) * seed.year_days;
    let end = start + VIMSHOTTARI_TOTAL_YEARS * seed.year_days;
    Ok(subdivide(&[], lord, start, end))
}

/// `path` ile seçilen dönemin alt dönemleri. `[]` mahadashaları, `[maha]` antardashaları,
/// `[maha, antar, pratyantar, sookshma]` prana dönemlerini verir. Daha uzun yol hatadır.
pub fn dasha_children(seed: &DashaSeed, path: &[Graha]) -> Result<Vec<DashaPeriod>, CoreError> {
    if path.len() >= usize::from(MAX_DASHA_LEVEL) {
        return Err(CoreError::InvalidDashaInput("path"));
    }
    let mut periods = mahadashas(seed)?;
    for lord in path {
        // Her seviyede dokuz lordun hepsi bulunur; yine de panik yerine hata.
        let parent = periods
            .into_iter()
            .find(|p| p.lords.last() == Some(lord))
            .ok_or(CoreError::InvalidDashaInput("path"))?;
        periods = children_of(&parent);
    }
    Ok(periods)
}
```

`app/src-tauri/src/commands.rs` içindeki `impl From<CoreError> for CommandError` `match`'ine, `CoreError::NonFiniteResult(what) => …` kolundan sonra ekle:

```rust
            CoreError::InvalidDashaInput(field) => {
                Self::new("invalid_dasha_input", &[("field", field.to_owned())])
            }
```

- [ ] **Step 4: Testlerin geçtiğini doğrula**

Run: `cargo fmt --all && cargo test -p jyotish-core && cargo test -p jyotish-desk --lib && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all --check`
Expected: `jyotish-core`: `test result: ok. 78 passed` (70 + 7 dasha + 1 settings), golden `3 passed`. `jyotish-desk` birim testleri `6 passed`. Clippy uyarısız. `bindings.ts` değişmez (`DashaSeed`/`DashaPeriod` henüz dışa aktarılan bir tipte kullanılmıyor).

- [ ] **Step 5: Commit**

```bash
git add crates/jyotish-core/src app/src-tauri/src/commands.rs
git commit -m "feat(core): Vimshottari periods and dasha_children to five levels" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: `Chart.dasha` (3 seviye) ve dasha proptest'leri

**Files:**
- Modify: `crates/jyotish-core/src/dasha.rs`, `crates/jyotish-core/src/chart.rs`, `crates/jyotish-core/src/settings.rs` (belge yorumu), `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/proptests.rs`
- Modify: `crates/jyotish-core/Cargo.toml` (dev-dependency `serde_json = { version = "1", features = ["float_roundtrip"] }`; 819 dönemlik `chart_serde_round_trip` testi bu özellik olmadan 1 ulp farkla düşer)
- Modify (üretilir): `app/ui/bindings.ts`
- Test: `dasha.rs`, `chart.rs` içindeki `mod tests`; `proptests.rs` içindeki `mod dasha_properties`

**Interfaces:**
- Consumes: `mahadashas`, `dasha_children`, `children_of`, `DashaSeed`, `DashaPeriod` (Task 5).
- Produces:
  - `pub const CHART_DASHA_LEVELS: u8 = 3`
  - `pub struct Dasha { pub seed: DashaSeed, pub periods: Vec<DashaPeriod> }`: 1–3. seviyeler ön-sırada (819 dönem)
  - `pub fn vimshottari(seed: DashaSeed) -> Result<Dasha, CoreError>`
  - `Chart.dasha: Dasha` (`vargas`'tan sonra). Tohum: `birth_jd_ut = jd_ut`, `moon_sidereal_lon` = Ay'ın sidereal boylamı (her kipte), `year_days = settings.dasha_year`.

- [ ] **Step 0: `serde_json` float_roundtrip özelliğini aç**

`crates/jyotish-core/Cargo.toml` `[dev-dependencies]` içindeki `serde_json` satırını şöyle yap:

```toml
serde_json = { version = "1", features = ["float_roundtrip"] }
```

- [ ] **Step 1: Başarısız testleri yaz**

`dasha.rs` test modülünün sonuna ekle:

```rust
    #[test]
    fn chart_dasha_has_three_levels_in_preorder() {
        let d = vimshottari(seed(NAKSHATRA_SPAN_DEG / 2.0)).unwrap();
        assert_eq!(d.periods.len(), 9 + 81 + 729);
        assert_eq!(d.periods[0].lords, [Graha::Ketu]);
        assert_eq!(d.periods[1].lords, [Graha::Ketu, Graha::Ketu]);
        assert_eq!(d.periods[2].lords, [Graha::Ketu, Graha::Ketu, Graha::Ketu]);
        // Ke, Ke-Ke, 9 × Ke-Ke-*, sonra Ke-Ve: indeks 11.
        assert_eq!(d.periods[11].lords, [Graha::Ketu, Graha::Venus]);
        // Ketu bloğu 1 + 9 × (1 + 9) = 91 dönem; ardından Venüs mahadashası.
        assert_eq!(d.periods[91].lords, [Graha::Venus]);
        assert!(d.periods.iter().all(|p| p.has_children && p.level <= 3));
        let level3: Vec<DashaPeriod> = d
            .periods
            .iter()
            .filter(|p| p.level == 3 && p.lords[..2] == [Graha::Moon, Graha::Mars])
            .cloned()
            .collect();
        assert_eq!(
            level3,
            dasha_children(&d.seed, &[Graha::Moon, Graha::Mars]).unwrap()
        );
    }
```

`chart.rs` içindeki `mod tests`'e, `use crate::varga::Varga;` satırından sonra ekle:

```rust
    use crate::dasha::DashaSeed;
```

ve test modülünün sonuna ekle:

```rust
    #[test]
    fn dasha_comes_from_the_sidereal_moon_in_both_modes() {
        for s in [CalcSettings::default(), tropical_rasi()] {
            let c = chart(&mut FakeEph::default(), &s);
            // Ay sidereal 71°: Ardra (6.), lord Rahu; kesir (71 − 66.6̄) / 13.3̄ = 0.325.
            assert_eq!(
                c.dasha.seed,
                DashaSeed {
                    birth_jd_ut: JD,
                    moon_sidereal_lon: 71.0,
                    year_days: 365.25
                }
            );
            let first = &c.dasha.periods[0];
            assert_eq!(first.lords, [Graha::Rahu]);
            assert!((first.start_jd_ut - (JD - 0.325 * 18.0 * 365.25)).abs() < 1e-6);
            assert_eq!(c.dasha.periods.len(), 819);
        }
    }
```

`proptests.rs` dosyasının sonuna ekle:

```rust
mod dasha_properties {
    use proptest::prelude::*;

    use crate::dasha::{
        dasha_children, mahadashas, vimshottari, vimshottari_years, DashaPeriod, DashaSeed,
        VIMSHOTTARI_ORDER, VIMSHOTTARI_TOTAL_YEARS,
    };
    use crate::graha::Graha;

    /// JD ~2.4e6 civarında f64 çözünürlüğü ~5e-10 gün; 1e-6 gün (~0.09 sn) bol paydır.
    const TOL_DAYS: f64 = 1e-6;

    fn seed_strategy() -> impl Strategy<Value = DashaSeed> {
        (
            0.0f64..360.0,
            2_300_000.0f64..2_500_000.0,
            prop::sample::select(vec![360.0, 365.2422, 365.25]),
        )
            .prop_map(|(moon, jd, year)| DashaSeed {
                birth_jd_ut: jd,
                moon_sidereal_lon: moon,
                year_days: year,
            })
    }

    fn lord() -> impl Strategy<Value = Graha> {
        prop::sample::select(VIMSHOTTARI_ORDER.to_vec())
    }

    /// Alt dönemler ebeveyni boşluksuz ve örtüşmesiz böler; paylar yıl oranındadır.
    fn check_partition(
        start: f64,
        end: f64,
        children: &[DashaPeriod],
    ) -> Result<(), TestCaseError> {
        prop_assert_eq!(children.len(), 9);
        prop_assert_eq!(children[0].start_jd_ut, start);
        prop_assert_eq!(children[8].end_jd_ut, end);
        for w in children.windows(2) {
            prop_assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
        }
        let span = end - start;
        let mut sum = 0.0;
        for c in children {
            let d = c.end_jd_ut - c.start_jd_ut;
            prop_assert!(d > 0.0, "{:?}", c.lords);
            let lord = *c.lords.last().unwrap();
            let want = span * vimshottari_years(lord) / VIMSHOTTARI_TOTAL_YEARS;
            prop_assert!((d - want).abs() < TOL_DAYS, "{:?}: {d} vs {want}", c.lords);
            sum += d;
        }
        prop_assert!((sum - span).abs() < TOL_DAYS);
        Ok(())
    }

    proptest! {
        #[test]
        fn mahadashas_total_120_years_and_contain_birth(seed in seed_strategy()) {
            let m = mahadashas(&seed).unwrap();
            check_partition(m[0].start_jd_ut, m[8].end_jd_ut, &m)?;
            let total = m[8].end_jd_ut - m[0].start_jd_ut;
            prop_assert!((total - VIMSHOTTARI_TOTAL_YEARS * seed.year_days).abs() < TOL_DAYS);
            prop_assert!(m[0].start_jd_ut <= seed.birth_jd_ut);
            prop_assert!(seed.birth_jd_ut <= m[0].end_jd_ut + TOL_DAYS);
        }

        #[test]
        fn sub_periods_partition_their_parent(
            seed in seed_strategy(),
            path in proptest::collection::vec(lord(), 1..=4),
        ) {
            let (last, parent_path) = path.split_last().unwrap();
            let siblings = dasha_children(&seed, parent_path).unwrap();
            let parent = siblings.iter().find(|p| p.lords.last() == Some(last)).unwrap();
            let children = dasha_children(&seed, &path).unwrap();
            check_partition(parent.start_jd_ut, parent.end_jd_ut, &children)?;
            for c in &children {
                prop_assert_eq!(&c.lords[..path.len()], &path[..]);
                prop_assert_eq!(usize::from(c.level), path.len() + 1);
            }
        }
    }

    proptest! {
        // Her durumda 90 ebeveyn × 819 dönem taranır; debug derlemede süreyi sınırlı tut.
        #![proptest_config(ProptestConfig::with_cases(32))]
        #[test]
        fn chart_dasha_matches_dasha_children(seed in seed_strategy()) {
            let d = vimshottari(seed).unwrap();
            prop_assert_eq!(d.periods.len(), 9 + 81 + 729);
            for p in d.periods.iter().filter(|p| p.level < 3) {
                let kids: Vec<DashaPeriod> = d
                    .periods
                    .iter()
                    .filter(|c| {
                        c.level == p.level + 1 && c.lords[..p.lords.len()] == p.lords[..]
                    })
                    .cloned()
                    .collect();
                prop_assert_eq!(kids, dasha_children(&seed, &p.lords).unwrap());
            }
        }
    }
}
```

- [ ] **Step 2: Testlerin derlenmediğini doğrula**

Run: `cargo test -p jyotish-core dasha`
Expected: FAIL, derleme hatası: `cannot find function 'vimshottari'`, `no field 'dasha' on type 'Chart'`.

- [ ] **Step 3: Uygulamayı yaz**

`dasha.rs` içinde `pub const MAX_DASHA_LEVEL: u8 = 5;` satırından sonra ekle:

```rust
/// `Chart.dasha`'nın taşıdığı seviye sayısı (spec §4: "Dasha 3 seviye gelir").
pub const CHART_DASHA_LEVELS: u8 = 3;
```

`DashaPeriod` struct'ından sonra ekle:

```rust
/// Haritanın Vimshottari ağacı: 1–3. seviyeler ön-sırada (her dönemin ardından alt
/// dönemleri). 4. ve 5. seviyeler `dasha_children(seed, path)` ile istenir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Dasha {
    pub seed: DashaSeed,
    pub periods: Vec<DashaPeriod>,
}
```

`dasha_children` fonksiyonundan sonra (test modülünden önce) ekle:

```rust
/// Haritaya giden üç seviyeli ağaç (9 + 81 + 729 dönem).
pub fn vimshottari(seed: DashaSeed) -> Result<Dasha, CoreError> {
    let mut periods = Vec::with_capacity(9 + 81 + 729);
    for maha in mahadashas(&seed)? {
        push_preorder(&mut periods, maha, CHART_DASHA_LEVELS);
    }
    Ok(Dasha { seed, periods })
}

fn push_preorder(out: &mut Vec<DashaPeriod>, period: DashaPeriod, levels: u8) {
    let children = if period.level < levels {
        children_of(&period)
    } else {
        Vec::new()
    };
    out.push(period);
    for child in children {
        push_preorder(out, child, levels);
    }
}
```

`chart.rs`:

1. `use crate::combustion::is_combust;` satırından sonra ekle:

```rust
use crate::dasha::{vimshottari, Dasha, DashaSeed};
```

2. `Chart` struct'ında `pub vargas: Vec<VargaChart>,` alanından sonra ekle:

```rust
    /// Vimshottari, 1–3. seviyeler. Ay'ın sidereal boylamından (her zodyak kipinde).
    pub dasha: Dasha,
```

3. `let vargas = all_vargas(lagna.lon, &points);` satırından sonra ekle:

```rust
    // raw[1] her zaman Ay'dır (PLANETS sırası); nakshatra her kipte sidereal'dir.
    let dasha = vimshottari(DashaSeed {
        birth_jd_ut: jd_ut,
        moon_sidereal_lon: to_sidereal(raw[1].lon),
        year_days: settings.dasha_year,
    })?;
```

4. `Chart { … }` içinde `vargas,` satırından sonra ekle:

```rust
        dasha,
```

`settings.rs` içinde `CalcSettings`'in `dasha_year` alanının belge yorumunu şununla değiştir:

```rust
    /// Vimshottari yıl uzunluğu, gün; `DASHA_YEAR_RANGE` içinde.
```

`lib.rs` içinde `pub use dasha::{…};` satırını şununla değiştir:

```rust
pub use dasha::{
    birth_dasha, dasha_children, mahadashas, vimshottari, vimshottari_years, Dasha,
    DashaPeriod, DashaSeed, CHART_DASHA_LEVELS, MAX_DASHA_LEVEL, VIMSHOTTARI_ORDER,
    VIMSHOTTARI_TOTAL_YEARS,
};
```

- [ ] **Step 4: Çekirdek testlerinin geçtiğini doğrula**

Run: `cargo fmt --all && cargo test -p jyotish-core && cargo clippy -p jyotish-core --all-targets -- -D warnings`
Expected: `test result: ok. 83 passed` (78 + 1 dasha + 1 chart + 3 proptest). Golden `3 passed`.

- [ ] **Step 5: Bağlamaları yeniden üret**

Run:

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
cargo test -p jyotish-desk --test bindings
grep -n "dasha: Dasha,\|export type DashaPeriod = {\|export type DashaSeed = {\|start_jd_ut: number" app/ui/bindings.ts
```

Expected: İki test de geçer. `grep` en az dört satır basar.

- [ ] **Step 6: Tam doğrulama**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && pnpm --dir app build`
Expected: Hepsi geçer. `compute_chart` entegrasyon testi değişmeden geçer (yeni alanlar yalnızca eklenir).

- [ ] **Step 7: Commit**

```bash
git add crates/jyotish-core/src crates/jyotish-core/Cargo.toml Cargo.lock app/ui/bindings.ts
git commit -m "feat(core): three-level Vimshottari tree on every chart" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Golden şemasını genişlet: D9 burçları ve ilk mahadasha

> **Kaynak dürüstlüğü:** Yeni beklentiler bağımsız bir yazılımdan gelmez. `scripts/gen-golden-swetest.sh` bunları fixture'ın kendi `swetest` boylamlarından, Rust motorundan ayrı bir **awk** uygulamasıyla türetir. D9 için BPHS kuralının sürekli biçimini (`floor(boylam × 9 / 30) mod 12`), dasha için Vimshottari formülünü kullanır. Bu, Rust'taki hareketli/sabit/çift doğalı kural yazımını ve dasha tesisatını (sidereal Ay seçimi, yıl ayarı, ön-sıra) bağımsız bir aritmetiğe karşı denetler. Yöntem hatasını (kuralın kendisini yanlış anlamak) yakalamaz; onu 25 haritalık JHora seti yakalar. Fixture'lar `"source": "swetest"` olarak kalır ve `notes`'a türetme yazılır.
>
> **Elle doğrulama (new-delhi-2000-01-01):** Ay `199.4705290°`. `199.470529 × 3 / 40 = 14.960290` → `n = 14` (0 tabanlı), 15. nakshatra Swati; `14 mod 9 = 5` → Rahu (18 yıl). Kesir `f = 0.960290`; geçen süre `0.9602897 × 18 × 365.25 = 6313.424` gün; başlangıç `2451545.0 − 6313.424 = 2445231.5755`. D9: Güneş `256.515696 × 9 / 30 = 76.95` → `76 mod 12 = 4` → Aslan; lagna `76.3380905 × 0.3 = 22.90` → `22 mod 12 = 10` → Kova.
>
> **Elle doğrulama (london-1850-01-01):** Ay `116.6412478°` → `× 3 / 40 = 8.748094` → `n = 8`, 9. nakshatra Ashlesha → Merkür (17 yıl). `0.748094 × 17 × 365.25 = 4645.100` gün; başlangıç `2396758.770833 − 4645.100 = 2392113.6707`.
>
> **Sınır payı:** Beş fixture'da her noktanın en yakın navamsa sınırına uzaklığı ≥ 91″'dir (en küçük: london). Golden toleransları 2″ (graha) ve 5″ (lagna) olduğundan D9 burçları toleransa duyarsızdır. Ay'daki 2″ hata dasha başlangıcını en çok `2/48000 × 20 × 365.25 ≈ 0.3` gün kaydırır (< 1 gün).

**Files:**
- Modify: `scripts/gen-golden-swetest.sh`, `tests/golden/README.md`, `tests/golden/*.json` (yeniden üretilir)
- Test: `crates/jyotish-core/tests/golden.rs`

**Interfaces:**
- Consumes: `Chart.vargas`, `Varga::D9`, `VargaChart.lagna_sign`, `VargaPlacement` (Task 4); `Chart.dasha.periods`, `DashaPeriod.{lords, start_jd_ut}` (Task 6); `Sign`, `Graha` serde (Plan 05).
- Produces: fixture şeması (geriye uyumlu; yeni alanlar isteğe bağlı):
  ```
  expected: { ayanamsa_deg, lagna_lon, grahas: {…},
              d9_signs?: { "Lagna": Sign, "Sun": Sign, …, "Ketu": Sign },   // varsa 10 anahtar da
              dasha?: { first_lord: Graha, first_start_jd: f64 } }          // UT JD, ≤ 1 gün
  ```

- [ ] **Step 1: Başarısız testi yaz**

`crates/jyotish-core/tests/golden.rs` dosyasının tamamı:

```rust
//! `tests/golden/*.json` referans haritalarını `compute_chart` ile karşılaştırır.
//! Toleranslar spec §8: graha ≤ 2″, ayanamsa ≤ 1″, lagna ≤ 5″, mahadasha başlangıcı
//! ≤ 1 gün, varga burçları birebir.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ephemeris::SwissEph;
use jyotish_core::{
    angular_distance, compute_chart, CalcSettings, Chart, ChartWarning, Graha, Sign, Varga,
};
use serde::Deserialize;

const GRAHA_TOL_ARCSEC: f64 = 2.0;
const AYANAMSA_TOL_ARCSEC: f64 = 1.0;
const LAGNA_TOL_ARCSEC: f64 = 5.0;
const DASHA_START_TOL_DAYS: f64 = 1.0;
/// Bilinen kaynaklar; yeni kaynak eklemek bilinçli bir karar olmalıdır.
const SOURCES: [&str; 2] = ["swetest", "jhora"];
/// Commit'li fixture sayısı bunun altına düşerse test düşer (sessiz silinmeye karşı).
const MIN_FIXTURES: usize = 5;
/// `d9_signs` ve `dasha` taşıyan fixture sayısı bunun altına düşerse test düşer.
const MIN_DERIVED_FIXTURES: usize = 5;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    id: String,
    source: String,
    source_version: String,
    #[serde(default)]
    notes: String,
    settings: CalcSettings,
    input: Input,
    expected: Expected,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    jd_ut: f64,
    lat: f64,
    lon: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    ayanamsa_deg: f64,
    lagna_lon: f64,
    grahas: BTreeMap<String, f64>,
    /// İsteğe bağlı: "Lagna" ve dokuz graha için D9 burcu. Varsa on anahtar da zorunlu.
    #[serde(default)]
    d9_signs: Option<BTreeMap<String, Sign>>,
    /// İsteğe bağlı: ilk mahadashanın lordu ve teorik başlangıcı (UT JD).
    #[serde(default)]
    dasha: Option<ExpectedDasha>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedDasha {
    first_lord: Graha,
    first_start_jd: f64,
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden")
}

fn ephe_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/ephe")
}

fn load_fixtures(dir: &Path) -> Vec<(PathBuf, Fixture)> {
    let entries =
        std::fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut paths: Vec<PathBuf> = entries
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|p| {
            let text =
                std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            let f: Fixture =
                serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            (p, f)
        })
        .collect()
}

/// Tolerans dışındaysa (ya da değer NaN ise) açıklama döner.
fn out_of_tolerance(what: &str, got: f64, want: f64, tol_arcsec: f64) -> Option<String> {
    let diff = angular_distance(got, want) * 3600.0;
    if diff.is_nan() || diff > tol_arcsec {
        Some(format!(
            "{what}: got {got:.7}, want {want:.7}, diff {diff:.3}\" > {tol_arcsec}\""
        ))
    } else {
        None
    }
}

fn compare(f: &Fixture, chart: &Chart) -> Vec<String> {
    let mut errs: Vec<String> = [
        out_of_tolerance(
            "ayanamsa",
            chart.ayanamsa_deg,
            f.expected.ayanamsa_deg,
            AYANAMSA_TOL_ARCSEC,
        ),
        out_of_tolerance(
            "lagna",
            chart.lagna.lon,
            f.expected.lagna_lon,
            LAGNA_TOL_ARCSEC,
        ),
    ]
    .into_iter()
    .flatten()
    .collect();
    for g in Graha::ALL {
        let name = format!("{g:?}");
        let got = chart
            .grahas
            .iter()
            .find(|p| p.graha == g)
            .map_or(f64::NAN, |p| p.lon);
        match f.expected.grahas.get(&name) {
            Some(&want) => errs.extend(out_of_tolerance(&name, got, want, GRAHA_TOL_ARCSEC)),
            None => errs.push(format!("{name}: missing from fixture")),
        }
    }
    if f.expected.grahas.len() != Graha::ALL.len() {
        errs.push(format!(
            "expected.grahas has {} keys, want exactly {:?}",
            f.expected.grahas.len(),
            Graha::ALL
        ));
    }
    errs
}

/// Plan 06 beklentileri: D9 burçları birebir, ilk mahadasha lordu birebir ve başlangıcı
/// ≤ 1 gün. Alan yoksa denetim yapılmaz (eski ve JHora fixture'ları).
fn compare_derived(f: &Fixture, chart: &Chart) -> Vec<String> {
    let mut errs = Vec::new();
    if let Some(want) = &f.expected.d9_signs {
        match chart.vargas.iter().find(|v| v.varga == Varga::D9) {
            Some(d9) => {
                let mut got: BTreeMap<String, Sign> = d9
                    .placements
                    .iter()
                    .map(|p| (format!("{:?}", p.graha), p.sign))
                    .collect();
                got.insert("Lagna".to_owned(), d9.lagna_sign);
                if &got != want {
                    errs.push(format!("d9_signs: got {got:?}, want {want:?}"));
                }
            }
            None => errs.push("D9 missing from chart.vargas".to_owned()),
        }
    }
    if let Some(want) = &f.expected.dasha {
        match chart.dasha.periods.first() {
            Some(first) => {
                if first.lords != [want.first_lord] {
                    errs.push(format!(
                        "dasha first lord: got {:?}, want {:?}",
                        first.lords, want.first_lord
                    ));
                }
                let diff = (first.start_jd_ut - want.first_start_jd).abs();
                if diff.is_nan() || diff > DASHA_START_TOL_DAYS {
                    errs.push(format!(
                        "dasha first start: got {:.6}, want {:.6}, diff {diff:.3} d > {DASHA_START_TOL_DAYS} d",
                        first.start_jd_ut, want.first_start_jd
                    ));
                }
            }
            None => errs.push("dasha has no periods".to_owned()),
        }
    }
    errs
}

#[test]
fn tolerance_check_flags_out_of_range_nan_and_wraps() {
    let arcsec = 1.0 / 3600.0;
    assert!(out_of_tolerance("x", 10.0, 10.0 + 1.9 * arcsec, 2.0).is_none());
    assert!(out_of_tolerance("x", 10.0, 10.0 + 2.1 * arcsec, 2.0).is_some());
    assert!(out_of_tolerance("x", f64::NAN, 10.0, 2.0).is_some());
    assert!(out_of_tolerance("x", 359.9999, 0.0001, 1.0).is_none()); // 0.72″
}

#[test]
fn golden_charts_match_references() {
    let fixtures = load_fixtures(&golden_dir());
    assert!(
        fixtures.len() >= MIN_FIXTURES,
        "found {} golden fixtures in {}, want >= {MIN_FIXTURES}",
        fixtures.len(),
        golden_dir().display()
    );
    let mut eph = SwissEph::open(&ephe_dir()).expect("open ephemeris");
    let mut failures = Vec::new();
    let mut derived = 0;
    let mut sample: Option<(Fixture, Chart)> = None;
    for (path, f) in &fixtures {
        assert_eq!(
            path.file_stem().and_then(|s| s.to_str()),
            Some(f.id.as_str()),
            "file name must equal id"
        );
        assert!(
            SOURCES.contains(&f.source.as_str()),
            "{}: unknown source {:?}",
            f.id,
            f.source
        );
        assert!(
            !f.source_version.is_empty(),
            "{}: empty source_version",
            f.id
        );
        let chart = compute_chart(
            &mut eph,
            f.input.jd_ut,
            f.input.lat,
            f.input.lon,
            &f.settings,
        )
        .unwrap_or_else(|e| panic!("{}: {e}", f.id));
        for e in compare(f, &chart)
            .into_iter()
            .chain(compare_derived(f, &chart))
        {
            failures.push(format!(
                "{} [{} {}; {}]: {e}",
                f.id, f.source, f.source_version, f.notes
            ));
        }
        if f.expected.d9_signs.is_some() && f.expected.dasha.is_some() {
            derived += 1;
            if sample.is_none() {
                sample = Some((f.clone(), chart));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "golden mismatches:\n{}",
        failures.join("\n")
    );
    assert!(
        derived >= MIN_DERIVED_FIXTURES,
        "found {derived} fixtures with d9_signs and dasha, want >= {MIN_DERIVED_FIXTURES}"
    );

    // Karşılaştırıcı gerçekten ısırır: beklentiyi üç yerden boz, üç hata bekle.
    let (mut bad, chart) = sample.expect("a fixture with derived expectations");
    if let Some(d) = bad.expected.dasha.as_mut() {
        d.first_lord = if d.first_lord == Graha::Sun {
            Graha::Moon
        } else {
            Graha::Sun
        };
        d.first_start_jd += 2.0;
    }
    if let Some(s) = bad
        .expected
        .d9_signs
        .as_mut()
        .and_then(|m| m.get_mut("Lagna"))
    {
        *s = Sign::from_index(s.index() + 1);
    }
    let errs = compare_derived(&bad, &chart);
    assert_eq!(errs.len(), 3, "{errs:?}");
}

#[test]
fn out_of_file_range_date_warns_low_precision() {
    let mut eph = SwissEph::open(&ephe_dir()).expect("open ephemeris");
    let settings = CalcSettings::default();
    // 1700-01-01: ephemeris dosyalarının dışında, Moshier'e düşer.
    let old = compute_chart(&mut eph, 2_341_972.5, 41.0, 29.0, &settings).expect("1700 chart");
    assert_eq!(old.meta.warnings, [ChartWarning::LowPrecision]);
    let j2000 = compute_chart(&mut eph, 2_451_545.0, 41.0, 29.0, &settings).expect("J2000 chart");
    assert!(j2000.meta.warnings.is_empty());
}
```

- [ ] **Step 2: Testin düştüğünü doğrula**

Run: `cargo test -p jyotish-core --test golden`
Expected: FAIL, `golden_charts_match_references`: `found 0 fixtures with d9_signs and dasha, want >= 5`. Diğer iki test geçer (eski fixture'lar yeni şemayla hâlâ okunur).

- [ ] **Step 3: Üreticiyi genişlet**

`scripts/gen-golden-swetest.sh` içinde `grahas="$(…)"` atamasının bittiği satırdan (`printf "      \"Ketu\": %.7f\n", k }')"`) sonra, `cat > "$OUT_DIR/$id.json" <<JSON` satırından önce ekle:

```bash
  # Plan 06 türetilmiş beklentileri. Rust motorundan bağımsız awk uygulaması; girdi
  # yukarıdaki swetest boylamlarıdır.
  points="$(printf 'Lagna %s\n' "$asc"; echo "$grahas" | tr -d '":,' | awk '{print $1, $2}')"
  # D9: BPHS navamsa kuralının sürekli biçimi, Koç'tan floor(boylam / 3°20') mod 12.
  d9="$(echo "$points" | awk '
    BEGIN { split("Aries Taurus Gemini Cancer Leo Virgo Libra Scorpio Sagittarius Capricorn Aquarius Pisces", S, " ") }
    { line[NR] = sprintf("      \"%s\": \"%s\"", $1, S[int($2 * 3 / 10) % 12 + 1]) }
    END { for (i = 1; i <= NR; i++) printf "%s%s\n", line[i], (i < NR ? "," : "") }')"
  # İlk mahadasha: Ay'ın nakshatrası (13°20') lordu verir; başlangıç = jd − kesir × yıl × 365.25.
  dasha="$(echo "$points" | awk -v jd="$jd" '$1 == "Moon" {
    split("Ketu Venus Sun Moon Mars Rahu Jupiter Saturn Mercury", L, " ");
    split("7 20 6 10 7 18 16 19 17", Y, " ");
    x = $2 * 3 / 40; n = int(x); f = x - n; i = n % 9 + 1;
    printf "{ \"first_lord\": \"%s\", \"first_start_jd\": %.6f }", L[i], jd - f * Y[i] * 365.25 }')"
```

Aynı dosyada heredoc içindeki `"notes"` satırını şununla değiştir:

```
  "notes": "$note; swetest -sid1 (Lahiri), mean node, UT $d $t; d9_signs ve dasha awk ile türetildi (Plan 06)",
```

ve heredoc'taki `expected` bloğunun sonunu (`"grahas": {` satırından `JSON` satırına kadar) şununla değiştir:

```
    "grahas": {
$grahas
    },
    "d9_signs": {
$d9
    },
    "dasha": $dasha
  }
}
JSON
```

- [ ] **Step 4: Fixture'ları yeniden üret ve farkı denetle**

Run:

```bash
./scripts/gen-golden-swetest.sh
git diff --numstat tests/golden
grep -h '"dasha"' tests/golden/*.json
```

Expected:
- Betik beş kez `wrote …/tests/golden/<id>.json` basar.
- `git diff --numstat` her fixture için `15	2` basar: değişen satırlar yalnızca `notes` ve `grahas` bloğunun kapanışı (`}` → `},`); eklenenler 12 satırlık `d9_signs` bloğu ve `dasha` satırı. Boylam, ayanamsa ve lagna değerleri bayt düzeyinde aynı kalır.
- `dasha` satırları (alfabetik dosya sırasıyla; son ondalıkta ±1 fark kabul):

| Fixture | Ay (°) | Nakshatra | `first_lord` | `first_start_jd` |
|---|---|---|---|---|
| buenos-aires-1923-06-15 | 65.5530117 | 5 Mrigashira | Mars | 2421242.300300 |
| istanbul-1987-03-17 | 182.9775858 | 14 Chitra | Mars | 2445023.153618 |
| london-1850-01-01 | 116.6412478 | 9 Ashlesha | Mercury | 2392113.670741 |
| new-delhi-2000-01-01 | 199.4705290 | 15 Swati | Rahu | 2445231.575532 |
| tromso-2050-12-21 | 343.1305929 | 26 Uttara Bhadrapada | Saturn | 2465062.970511 |

- `d9_signs` blokları (Lagna, Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu sırasıyla):

| Fixture | D9 burçları |
|---|---|
| buenos-aires | Aries, Libra, Scorpio, Pisces, Aries, Pisces, Aquarius, Cancer, Libra, Aries |
| istanbul | Gemini, Cancer, Libra, Scorpio, Sagittarius, Virgo, Leo, Pisces, Sagittarius, Gemini |
| london | Aquarius, Virgo, Aquarius, Virgo, Scorpio, Capricorn, Taurus, Libra, Taurus, Scorpio |
| new-delhi | Aquarius, Leo, Pisces, Scorpio, Gemini, Aries, Virgo, Leo, Libra, Aries |
| tromso | Aries, Taurus, Libra, Leo, Sagittarius, Cancer, Pisces, Aries, Aquarius, Leo |

Değerler tabloyla uyuşmazsa dur ve raporla: betik ya da tablo hatalıdır; fixture elle düzeltilmez.

- [ ] **Step 5: README'yi güncelle**

`tests/golden/README.md` içinde `## Şema` bölümündeki JSON örneğini şununla değiştir:

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

ve örneğin altındaki madde listesinin sonuna ekle:

```markdown
- `d9_signs` ve `dasha` isteğe bağlıdır. `d9_signs` varsa `Lagna` ve dokuz graha zorunludur; burçlar birebir karşılaştırılır.
- `dasha.first_start_jd`, ilk mahadashanın **teorik** başlangıcıdır (UT JD, doğumdan önce): `jd − geçen_kesir × yıl(lord) × dasha_year`. Tolerans ≤ 1 gün.
- `swetest` fixture'larında bu iki alan `scripts/gen-golden-swetest.sh` içinde awk ile türetilir (Rust motorundan bağımsız aritmetik, aynı kural). Bağımsız referans değildir; kuralın kendisini JHora seti doğrular.
- JHora'dan eklerken: D9 burçlarını Navamsa haritasından, ilk mahadasha lordunu ve başlangıç tarihini Vimshottari listesinden oku. JHora'nın dasha yılı ayarını fixture'ın `settings.dasha_year`'ıyla eşleştir; başlangıç tarihini UT JD'ye çevirip `notes`'a yerel tarihi yaz.
- En az 5 fixture `d9_signs` ve `dasha` taşımalıdır (`MIN_DERIVED_FIXTURES`).
```

- [ ] **Step 6: Testlerin geçtiğini doğrula**

Run: `cargo fmt --all && cargo test -p jyotish-core --test golden && cargo clippy -p jyotish-core --all-targets -- -D warnings && cargo fmt --all --check`
Expected: `test result: ok. 3 passed`.

- [ ] **Step 7: Commit**

```bash
git add crates/jyotish-core/tests/golden.rs scripts/gen-golden-swetest.sh tests/golden
git commit -m "test(golden): D9 signs and first mahadasha in swetest fixtures" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: `dasha_children` Tauri komutu ve uçtan uca doğrulama

**Files:**
- Modify: `app/src-tauri/src/commands.rs`, `app/src-tauri/src/lib.rs`
- Modify (üretilir): `app/ui/bindings.ts`
- Test: `commands.rs` içindeki `mod tests`, `app/src-tauri/tests/compute_chart.rs`

**Interfaces:**
- Consumes: `jyotish_core::{dasha_children, DashaPeriod, DashaSeed, Graha, Sign, Varga}` (Task 4–6); `CommandError`, `From<CoreError>` (Plan 05, Task 5).
- Produces:
  - Komut: `#[tauri::command] pub fn dasha_children(seed: DashaSeed, path: Vec<Graha>) -> Result<Vec<DashaPeriod>, CommandError>`. TS'de `commands.dashaChildren(seed, path)`. Motor kullanmaz; senkron ve mikrosaniye düzeyindedir.
  - Arayüz sözleşmesi: tohum `chart.dasha.seed`'den, yol tıklanan dönemin `lords`'undan gelir. `path.length` 3 → 4. seviye, 4 → 5. seviye.

- [ ] **Step 1: Başarısız testleri yaz**

`commands.rs` içindeki `mod tests`'in sonuna ekle:

```rust
    #[test]
    fn dasha_children_command_returns_sub_periods_and_maps_errors() {
        use jyotish_core::{DashaSeed, Graha};
        // tests/golden/new-delhi-2000-01-01.json Ay'ı: Rahu mahadashası.
        let seed = DashaSeed {
            birth_jd_ut: 2_451_545.0,
            moon_sidereal_lon: 199.470_529,
            year_days: 365.25,
        };
        let path = vec![Graha::Rahu, Graha::Rahu, Graha::Rahu];
        let kids = dasha_children(seed, path.clone()).expect("children");
        assert_eq!(kids.len(), 9);
        assert!(kids
            .iter()
            .all(|p| p.level == 4 && p.lords[..3] == path[..] && p.has_children));
        let err = dasha_children(seed, vec![Graha::Sun; 5]).unwrap_err();
        assert_eq!(err.code, "invalid_dasha_input");
        assert_eq!(err.params["field"], "path");
    }
```

`app/src-tauri/tests/compute_chart.rs` içinde `use jyotish_core::{AyanamsaSystem, CalcSettings, Graha};` satırını şununla değiştir:

```rust
use jyotish_core::{AyanamsaSystem, CalcSettings, Graha, Sign, Varga};
```

ve `assert!(chart.meta.warnings.is_empty());` satırından sonra ekle:

```rust

    // Plan 06: vargalar ve dasha (tests/golden/new-delhi-2000-01-01.json türetilmiş beklentileri).
    assert_eq!(chart.vargas.len(), 16);
    let d9 = chart
        .vargas
        .iter()
        .find(|v| v.varga == Varga::D9)
        .expect("D9");
    assert_eq!(d9.lagna_sign, Sign::Aquarius);
    let sun_d9 = d9
        .placements
        .iter()
        .find(|p| p.graha == Graha::Sun)
        .unwrap();
    assert_eq!(sun_d9.sign, Sign::Leo);
    assert_eq!(chart.dasha.periods.len(), 9 + 81 + 729);
    let first = &chart.dasha.periods[0];
    assert_eq!(first.lords, [Graha::Rahu]);
    assert!(
        (first.start_jd_ut - 2_445_231.575_532).abs() <= 1.0,
        "maha start {}",
        first.start_jd_ut
    );
```

- [ ] **Step 2: Testlerin derlenmediğini doğrula**

Run: `cargo test -p jyotish-desk`
Expected: FAIL, derleme hatası: `cannot find function 'dasha_children' in this scope` (commands testleri). `compute_chart.rs` ayrı derlenir ve bu adımda geçebilir; çünkü vargalar ve dasha Task 4 ve 6'dan beri haritadadır.

- [ ] **Step 3: Komutu yaz**

`commands.rs` içinde `use jyotish_core::{CalcSettings, Chart, CoreError};` satırını şununla değiştir:

```rust
use jyotish_core::{CalcSettings, Chart, CoreError, DashaPeriod, DashaSeed, Graha};
```

`compute_chart` komutundan sonra ekle:

```rust
/// Bir dasha döneminin alt dönemleri (spec §4 `dasha_children(path)`). `seed`
/// `chart.dasha.seed`'den, `path` dönemin `lords`'undan gelir. Saf hesaptır, motora gitmez.
#[tauri::command]
#[specta::specta]
pub fn dasha_children(
    seed: DashaSeed,
    path: Vec<Graha>,
) -> Result<Vec<DashaPeriod>, CommandError> {
    Ok(jyotish_core::dasha_children(&seed, &path)?)
}
```

`app/src-tauri/src/lib.rs` içinde `collect_commands!` listesini şununla değiştir:

```rust
        .commands(tauri_specta::collect_commands![
            commands::engine_status,
            commands::planet_longitudes,
            commands::compute_chart,
            commands::dasha_children,
            commands::print_window,
        ])
```

- [ ] **Step 4: Bağlamaları yeniden üret**

Run:

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
cargo test -p jyotish-desk --test bindings
grep -n "dashaChildren: (seed: DashaSeed, path: Graha\[\]" app/ui/bindings.ts
grep -c "_Serialize\|_Deserialize" app/ui/bindings.ts
```

Expected: İki test de geçer. İlk `grep` bir satır basar (`typedError<DashaPeriod[], CommandError>`). İkinci `grep` `0` basar.

- [ ] **Step 5: Testlerin geçtiğini doğrula**

Run: `cargo test -p jyotish-desk && pnpm --dir app test && pnpm --dir app build`
Expected: `jyotish-desk` birim testleri `7 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`. Vitest ve `pnpm build` geçer.

- [ ] **Step 6: Tam doğrulama**

Run:

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo +1.93 check -p ephemeris -p atlas -p jyotish-core --all-targets
cargo +1.93 check -p jyotish-desk --all-targets
cargo deny check
grep -rniE "\bk(a|ā|aa)la\b|\bcala\b" crates/jyotish-core/src crates/jyotish-core/tests app/src-tauri/src app/src-tauri/tests scripts tests/golden || true
```

Expected: Hepsi geçer. `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`. `Cargo.lock` değişmez (yeni bağımlılık yok). Son `grep` yasaklı ad bulmaz (boş çıktı). Plan belgeleri taranmaz: yasak listesini kendileri içerir.

- [ ] **Step 7: Commit**

```bash
git add app/src-tauri/src app/src-tauri/tests/compute_chart.rs app/ui/bindings.ts
git commit -m "feat(app): dasha_children command for levels four and five" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Bu planın tamamlanma ölçütü

- `cargo test --workspace` geçer. Kapsananlar:
  - `jyotish-core`: 83 birim ve proptest (55 Plan 05 + 28 yeni), 3 golden testi
  - `jyotish-desk`: 7 birim testi, `bindings` ve `compute_chart` entegrasyon testleri
- `cargo +1.93 check` çekirdek crate'ler ve `jyotish-desk` için geçer. `cargo deny check` temiz; yeni bağımlılık yok.
- `Chart` 16 varga (`vargas`), vargottama bayrağı ve 3 seviyeli Vimshottari (`dasha`) taşır. `dasha_children` saf fonksiyon ve Tauri komutu olarak 4. ve 5. seviyeleri verir.
- 5 golden fixture D9 burçlarını ve ilk mahadashayı taşır; hepsi geçer.
- `bindings.ts`'de `_Serialize`/`_Deserialize` ayrımı yok; `Chart` tek tiptir.

## Bu planın kapsamadıkları (sonraki planlar)

- Panchang, gün doğumu ve batışı (`swe_rise_trans` + `SE_BIT_HINDU_RISING`), ashtakavarga (BAV/SAV ve 337/48… proptest'leri), transit: **Plan 07**.
- Bilinmeyen doğum saatinde lagnaya bağlı vargaların gizlenmesi ve dashanın "yaklaşık" işaretlenmesi (spec §7): arayüz ve `store` (`BirthData.time_certainty`).
- Dasha ağacının arayüzü, `ViewState.dasha_expanded_path`: arayüz planı (M5).
- 25 haritalık JHora golden seti; D9 ve dasha için gerçek bağımsız referans: kullanıcı ekler (`tests/golden/README.md`).
- Performans ölçümü (`open_client` Rust tarafında ≤ 50 ms): `store`/`open_client` planı. Bu plan haritaya ~160 varga konumu ve 819 dasha dönemi ekler (saf aritmetik, ~115 KB JSON); bütçe riski beklenmez.
- Lagna için vargottama alanı, varga haritalarında lagna derecesi: istenirse arayüz planında.

---

## Self-Review

**1. Spec kapsamı:**

| Gereksinim | Görev |
|---|---|
| 16 varga, Parashara tanımları (spec §3) | Task 2 (D1–D12), Task 3 (D16–D60) |
| D2 Parashara hora, D3 Parashara drekkana, D30 eşit olmayan trimsamsa | Task 2, Task 3; kurallar ve kaynak "Astrolojik tanımlar" tablosunda |
| Her varga için tablo testleri, sınır dereceleri dahil | Task 2 `lower_vargas_follow_parashari_rules`, Task 3 `upper_vargas_follow_parashari_rules`, `top_of_sign_stays_in_last_part` |
| `VargaChart { varga, lagna_sign, placements[(graha, sign, deg)] }` (spec §5), deg tanımı | Task 4; tanım "Bu planın kararları"nda |
| Vargottama kararı | Task 4 (`GrahaPosition.vargottama`) |
| `Chart.vargas`, her zaman mı isteğe bağlı mı | Kararlar tablosu (her zaman), Task 4 |
| Vimshottari: Ay nakshatrası, sıra, yıllar, 120 toplam | Task 5 |
| `dasha_year` (365.25, 360, 365.2422) | Task 5 (`year_days`, `DASHA_YEAR_RANGE`), `year_length_scales_every_period` |
| `DashaPeriod { lords, level, start, end, has_children }` (spec §5), zaman gösterimi | Task 5; JD kararı tabloda |
| Harita 3 seviye taşır (spec §4) | Task 6 |
| `dasha_children(path)` saf fonksiyon + Tauri komutu (4–5. seviye) | Task 5 (fonksiyon), Task 8 (komut) |
| Proptest: alt dashalar boşluksuz, toplam ebeveyne eşit, toplam 120 yıl (spec §8) | Task 6 `mod dasha_properties` |
| Golden: mahadasha başlangıcı ≤ 1 gün, varga burçları birebir (spec §8) | Task 7 |
| Golden şeması geriye uyumlu, kaynak dürüst | Task 7 (isteğe bağlı alanlar, awk türetmesi, `notes`) |
| Mevcut 5 fixture geçer | Task 7 Step 4 (`15 2` farkı), Step 6 |
| `compute_chart` vargas + dasha döner, bindings yeniden üretilir | Task 4, 6, 8 |
| Plan 05 T8a: `Chart_Serialize \| Chart_Deserialize` | Task 1 |
| Saf çekirdek, panik yok, `Result<_, CoreError>`, snake_case, yeni bağımlılık yok, yasaklı ad yok | Global Constraints; `equal_part`/`trimsamsa` kırpması, `saturating_sub`, `dasha_children`'da `ok_or`; Task 8 Step 6 |

Bilinçli olarak dışarıda bırakılanlar: panchang, gün doğumu, ashtakavarga, transit (Plan 07); bilinmeyen saat davranışı (arayüz/store).

**2. Yer tutucu taraması:** "TBD", "TODO", "sonra doldur" yok. Her kod adımında tam kod var. Fixture JSON'ları betikle üretilir; beklenen değerler Task 7 Step 4'teki tablolarda tam olarak verilmiştir. Task 1 Step 4'teki not bir durma koşuludur, yer tutucu değildir.

**3. Tip tutarlılığı:**
- `varga_position(varga: Varga, lon: f64) -> (Sign, f64)` Task 2'de tanımlanır; Task 3 imzayı korur; Task 4 (`varga_chart`, `is_vargottama`) ve proptest'ler aynı imzayı kullanır.
- `Varga::ALL` Task 2'de 8, Task 3'ten sonra 16 elemanlıdır; `all_vargas` ve `vargas_cover_all_sixteen_from_the_rasi_frame` 16'yı bekler (Task 4, Task 3'ten sonra gelir).
- `VargaPlacement.deg` (spec'teki `deg`) her yerde aynı adla geçer; `GrahaPosition.deg_in_sign` ile karıştırılmaz.
- `DashaSeed { birth_jd_ut, moon_sidereal_lon, year_days }` Task 5'te tanımlanır; Task 6 (`chart.rs`, `dasha_comes_from_the_sidereal_moon_in_both_modes`), Task 7 (dolaylı) ve Task 8 (komut, test) aynı alan adlarını kullanır.
- `DashaPeriod { lords, level, start_jd_ut, end_jd_ut, has_children }` Task 5–8'de aynıdır; golden `first.start_jd_ut` ve `first.lords` kullanır.
- `CoreError::InvalidDashaInput(&'static str)` Task 5'te eklenir ve aynı görevde `commands.rs` eşlemesi yapılır; `match` tam kalır. Alan adları `"birth_jd_ut"`, `"moon_sidereal_lon"`, `"year_days"`, `"path"` testlerle sabitlenir.
- `children_of` Task 5'te özeldir; Task 6'nın `push_preorder`'ı aynı modülde kullanır.
- `CHART_DASHA_LEVELS` Task 6'da, `MAX_DASHA_LEVEL` Task 5'te tanımlanır; `lib.rs` yeniden dışa aktarımı Task 6'da ikisini de içerir.
- Komut adı `dasha_children` ile çekirdek fonksiyonu çakışmaz: komut gövdesi `jyotish_core::dasha_children` tam yolunu kullanır.
- Test sayıları: 55 → 62 (Task 2) → 66 (Task 3) → 70 (Task 4) → 78 (Task 5) → 83 (Task 6); golden 3; `jyotish-desk` birim 6 → 7 (Task 8).
