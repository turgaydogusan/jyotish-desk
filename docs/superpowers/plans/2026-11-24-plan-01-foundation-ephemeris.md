# Plan 01 — Repo İskeleti ve `ephemeris` Crate'i

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Swiss Ephemeris'i vendored C kaynağından derleyen, güvenli bir Rust API sunan ve `swetest` ile ≤ 0.001″ uyumu kanıtlanmış `ephemeris` crate'ini içeren monorepo iskeletini kurmak.

**Architecture:** Cargo workspace. `crates/ephemeris` Swiss Ephemeris'in C dosyalarını `cc` ile derler ve elle yazılmış FFI bildirimleri kullanır, bu yüzden bindgen ya da libclang gerekmez. Tek bir global `Mutex`, aynı anda yalnızca tek bir `SwissEph` örneğinin var olmasını sağlar, çünkü SE'nin global durumu thread-safe değil. `Ephemeris` trait'i, üst katmanların sahte (fake) motorla test edilmesine izin verir.

**Tech Stack:** Rust (stable, MSRV 1.80), `cc` 1.x, `thiserror` 2.x, Swiss Ephemeris 2.10.03 (AGPL), GitHub Actions, cargo-deny.

**Spec:** `docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md`

## Global Constraints

- Lisans: AGPL-3.0-or-later (tüm crate'ler, `license` alanında).
- Ephemeris: Swiss Ephemeris; kaynak vendored, `cc` ile derlenir, build sırasında libclang gerekmez.
- SE global durumu: tek `SwissEph` örneği, `Mutex` arkasında.
- Veri yolu parametredir (sabit kodlanmış yol yok; Android'e hazırlık).
- Eksik ephemeris dosyası: açılış self-check'i ölümcül hata verir, sessiz yedeğe düşme yok.
- Paketlenen aralık dışındaki tarih: Moshier'e düşülür ve `Precision::Moshier` olarak işaretlenir.
- Ephemeris sarmalayıcısının `swetest`'e toleransı ≤ 0.001″.
- FFI sınırında panik yok.
- CI: `macos-14` tam set; `ubuntu` ve `windows` yalnızca çekirdek crate'ler; clippy `-D warnings`; `cargo deny`.
- Kod adı `jyotish-desk`. "Kala" ve türevleri hiçbir yerde kullanılmaz.

## v1 Plan Yol Haritası

Her plan bir öncekinin bitmesinin ardından, o anki koda göre yazılır.

| Plan | İçerik | Spec kilometre taşı |
|---|---|---|
| **01** | **Repo iskeleti, CI, `ephemeris` crate'i, x86_64 derleme kontrolü** | M0 + M1 (kısmen) |
| 02 | Spike'lar: Tauri 2 + tauri-specta iskeleti, WKWebView print → PDF | M0 |
| 03 | `atlas`: GeoNames pipeline'ı, tz çözümü, LMT, DST boşluk/örtüşme | M1 |
| 04 | `jyotish-core` temel: ayanamsa, grahalar, lagna, evler, nakshatra, dignity, yanma, golden altyapısı | M2 |
| 05 | `jyotish-core` ileri: 16 varga, Vimshottari, Panchang, Ashtakavarga, transit, tropikal zodyak | M3 |
| 06 | `store`: şema, migration, FTS, `open_client`, JSON/CSV/JHD | M4 |
| 07 | Arayüz iskeleti: giriş, SVG haritalar, tablolar, dasha ağacı, i18n, `ViewState` | M5 |
| 08 | Danışan arayüzü: `Cmd+K`, notlar, fark rozeti, oturum geri yükleme, transit, ayarlar | M6 |
| 09 | Print ve PDF düzenleri | M7 |
| 10 | İmza, notarizasyon, universal DMG, updater, beta | M8 |

## Dosya Yapısı (bu plan)

```
Cargo.toml                         workspace tanımı
.gitignore
LICENSE                            AGPL-3.0 metni
NOTICE                             üçüncü taraf bileşenler
README.md
deny.toml                          cargo-deny lisans/advisory kuralları
.github/workflows/ci.yml
scripts/vendor-swisseph.sh         SE kaynağını ve veri dosyalarını getirir
scripts/gen-swetest-fixtures.sh    swetest'i derler, golden CSV üretir
data/ephe/                         sepl_18.se1, semo_18.se1, sefstars.txt
crates/ephemeris/
  Cargo.toml
  build.rs                         SE C kaynağını derler
  vendor/swisseph/                 SE .c/.h dosyaları (+ swetest.c, yalnızca fixture için)
  src/lib.rs                       genel API, Ephemeris trait'i, julday_gregorian
  src/ffi.rs                       extern "C" bildirimleri ve sabitler
  src/types.rs                     Body, SiderealMode, Zodiac, Precision, BodyPosition
  src/error.rs                     EphError
  src/swiss.rs                     SwissEph (Mutex sahipliği, self-check, hesaplar)
  tests/common/mod.rs              ephe_dir() yardımcısı
  tests/ffi_smoke.rs
  tests/sanity.rs
  tests/golden.rs
  tests/fixtures/swetest.csv       üretilen golden veri
```

---

### Task 1: Repo iskeleti

**Files:**
- Create: `Cargo.toml`, `.gitignore`, `LICENSE`, `NOTICE`, `README.md`

**Interfaces:**
- Consumes: yok
- Produces: workspace kökü; `[workspace.package]` alanları (`version`, `edition`, `license`, `rust-version`) ve `[workspace.dependencies]` içinde `thiserror`. Sonraki crate'ler bunları `*.workspace = true` ile kullanır.

- [ ] **Step 1: Git deposunu başlat**

```bash
git init -b main
```

- [ ] **Step 2: Workspace `Cargo.toml` yaz**

```toml
[workspace]
resolver = "2"
members = []

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "AGPL-3.0-or-later"
rust-version = "1.80"
repository = "https://github.com/OWNER/jyotish-desk"

[workspace.dependencies]
thiserror = "2"
```

Not: `OWNER` yerine gerçek GitHub hesabı repo oluşturulunca yazılır. Bu yer tutucu yalnızca metadata, derlemeyi etkilemez.

- [ ] **Step 3: `.gitignore` yaz**

```gitignore
/target
**/node_modules
.DS_Store
```

- [ ] **Step 4: LICENSE indir, NOTICE ve README yaz**

```bash
curl -fsSL -o LICENSE https://www.gnu.org/licenses/agpl-3.0.txt
head -3 LICENSE
```

Beklenen: `GNU AFFERO GENERAL PUBLIC LICENSE` satırı.

`NOTICE`:

```text
jyotish-desk
Copyright (C) 2026 jyotish-desk contributors
Licensed under the GNU Affero General Public License v3.0 or later.

This product includes third-party components:

- Swiss Ephemeris, Copyright (C) 1997-2021 Astrodienst AG, Switzerland.
  Used under the GNU Affero General Public License v3.0.
  https://www.astro.com/swisseph/
```

`README.md`:

```markdown
# jyotish-desk (kod adı)

Profesyonel astrologlar için açık kaynak Jyotish (Vedik astroloji) masaüstü yazılımı.
Lisans: AGPL-3.0-or-later. Ayrıntılar: `docs/superpowers/specs/`.

## Geliştirme

    ./scripts/vendor-swisseph.sh   # bir kez
    cargo test --workspace
```

- [ ] **Step 5: Workspace'in geçerli olduğunu doğrula**

Run: `cargo metadata --format-version 1 --no-deps > /dev/null && echo OK`
Expected: `OK`

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml .gitignore LICENSE NOTICE README.md docs/
git commit -m "chore: initialize workspace with AGPL license and design docs"
```

---

### Task 2: Swiss Ephemeris'i vendor et ve derle

**Files:**
- Create: `scripts/vendor-swisseph.sh`, `crates/ephemeris/Cargo.toml`, `crates/ephemeris/build.rs`, `crates/ephemeris/src/lib.rs`, `crates/ephemeris/src/ffi.rs`, `crates/ephemeris/tests/ffi_smoke.rs`
- Create (script çıktısı): `crates/ephemeris/vendor/swisseph/*`, `data/ephe/{sepl_18.se1,semo_18.se1,sefstars.txt}`
- Modify: `Cargo.toml` (`members`)

**Interfaces:**
- Consumes: Task 1 workspace alanları
- Produces: `ephemeris::julday_gregorian(year: i32, month: u32, day: u32, hour_ut: f64) -> f64`; `pub(crate) mod ffi` (Task 3 kullanır); derlenmiş `libswe` statik kütüphanesi.

- [ ] **Step 1: Vendor script'ini yaz**

`scripts/vendor-swisseph.sh`:

```bash
#!/usr/bin/env bash
# Swiss Ephemeris kaynağını ve veri dosyalarını sabit bir sürümden getirir.
set -euo pipefail
SWE_TAG="v2.10.03"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$(mktemp -d)/swisseph"
DEST="$ROOT/crates/ephemeris/vendor/swisseph"
EPHE="$ROOT/data/ephe"

git clone --depth 1 --branch "$SWE_TAG" https://github.com/aloistr/swisseph.git "$SRC"

mkdir -p "$DEST" "$EPHE"
cp "$SRC"/*.c "$SRC"/*.h "$DEST"/
# Kütüphanede kullanılmayan örnek ve araç programları; swetest.c fixture üretimi için kalır.
rm -f "$DEST"/swemini.c "$DEST"/swevents.c "$DEST"/swephgen4.c "$DEST"/sweasp.c "$DEST"/swemptab.c.orig
cp "$SRC"/LICENSE "$DEST"/LICENSE 2>/dev/null || true

cp "$SRC"/ephe/sepl_18.se1 "$SRC"/ephe/semo_18.se1 "$SRC"/ephe/sefstars.txt "$EPHE"/
echo "$SWE_TAG" > "$DEST"/VERSION
ls -la "$DEST" "$EPHE"
```

- [ ] **Step 2: Script'i çalıştır ve dosyaları doğrula**

```bash
chmod +x scripts/vendor-swisseph.sh
./scripts/vendor-swisseph.sh
ls crates/ephemeris/vendor/swisseph/{sweph.c,swephlib.c,swedate.c,swejpl.c,swemmoon.c,swemplan.c,swehouse.c,swecl.c,swehel.c,swetest.c,swephexp.h}
ls -la data/ephe
```

Beklenen: Listelenen tüm `.c` ve `.h` dosyaları var. `data/ephe` içinde üç dosya var: `sepl_18.se1` yaklaşık 470 KB, `semo_18.se1` yaklaşık 1,3 MB.

Tag bulunamazsa `git ls-remote --tags https://github.com/aloistr/swisseph.git` ile en yakın 2.10.x tag'ini seç, `SWE_TAG`'i güncelle ve NOTICE'e not düş. Repoda `ephe/` klasörü yoksa üç dosyayı https://www.astro.com/ftp/swisseph/ephe/ adresinden indir.

- [ ] **Step 3: Crate `Cargo.toml` yaz ve workspace'e ekle**

`crates/ephemeris/Cargo.toml`:

```toml
[package]
name = "ephemeris"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
links = "swe"
build = "build.rs"

[dependencies]
thiserror.workspace = true

[build-dependencies]
cc = "1"
```

Kök `Cargo.toml` içinde `members = []` satırını şununla değiştir:

```toml
members = ["crates/ephemeris"]
```

- [ ] **Step 4: Failing smoke testi yaz**

`crates/ephemeris/tests/ffi_smoke.rs`:

```rust
#[test]
fn julday_of_j2000_epoch_is_2451545() {
    let jd = ephemeris::julday_gregorian(2000, 1, 1, 12.0);
    assert_eq!(jd, 2_451_545.0);
}
```

- [ ] **Step 5: Testi çalıştır, derleme hatası ver**

Run: `cargo test -p ephemeris --test ffi_smoke`
Expected: FAIL. `build.rs` ya da `src/lib.rs` bulunamadığı için derleme hatası.

- [ ] **Step 6: `build.rs` yaz**

`crates/ephemeris/build.rs`:

```rust
fn main() {
    let dir = "vendor/swisseph";
    let files = [
        "swedate.c",
        "swehouse.c",
        "swejpl.c",
        "swemmoon.c",
        "swemplan.c",
        "sweph.c",
        "swephlib.c",
        "swecl.c",
        "swehel.c",
    ];

    let mut build = cc::Build::new();
    build.include(dir).warnings(false).opt_level(2);
    for f in files {
        let path = format!("{dir}/{f}");
        println!("cargo:rerun-if-changed={path}");
        build.file(path);
    }
    build.compile("swe");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "linux" || target_os == "android" {
        println!("cargo:rustc-link-lib=m");
    }
}
```

- [ ] **Step 7: `ffi.rs` ve `lib.rs` yaz**

`crates/ephemeris/src/ffi.rs`:

```rust
//! Swiss Ephemeris C API'sinin kullandığımız alt kümesi.
//! İmzalar `swephexp.h` (2.10.x) ile birebir eşleşir.

use std::os::raw::{c_char, c_double, c_int};

pub const SE_GREG_CAL: c_int = 1;

pub const SEFLG_SWIEPH: i32 = 2;
pub const SEFLG_MOSEPH: i32 = 4;
pub const SEFLG_SPEED: i32 = 256;
pub const SEFLG_SIDEREAL: i32 = 64 * 1024;

pub const SE_SIDM_LAHIRI: i32 = 1;
pub const SE_SIDM_RAMAN: i32 = 3;
pub const SE_SIDM_KRISHNAMURTI: i32 = 5;
pub const SE_SIDM_TRUE_CITRA: i32 = 27;
pub const SE_SIDM_USER: i32 = 255;

/// `serr` tamponları için SE'nin `AS_MAXCH` değeri.
pub const AS_MAXCH: usize = 256;

extern "C" {
    pub fn swe_set_ephe_path(path: *const c_char);
    pub fn swe_close();
    pub fn swe_version(s: *mut c_char) -> *mut c_char;
    pub fn swe_julday(
        year: c_int,
        month: c_int,
        day: c_int,
        hour: c_double,
        gregflag: c_int,
    ) -> c_double;
    pub fn swe_calc_ut(
        tjd_ut: c_double,
        ipl: i32,
        iflag: i32,
        xx: *mut c_double,
        serr: *mut c_char,
    ) -> i32;
    pub fn swe_set_sid_mode(sid_mode: i32, t0: c_double, ayan_t0: c_double);
    pub fn swe_get_ayanamsa_ex_ut(
        tjd_ut: c_double,
        iflag: i32,
        daya: *mut c_double,
        serr: *mut c_char,
    ) -> i32;
}
```

`crates/ephemeris/src/lib.rs`:

```rust
//! Swiss Ephemeris için güvenli Rust sarmalayıcı.

#![allow(clippy::upper_case_acronyms)]

pub(crate) mod ffi;

/// Gregoryen takvim tarihini ve UT saatini (ondalık) Jülyen gününe çevirir.
/// Saf bir hesaptır, SE global durumuna dokunmaz; kilit gerekmez.
pub fn julday_gregorian(year: i32, month: u32, day: u32, hour_ut: f64) -> f64 {
    // SAFETY: swe_julday yalnızca argümanlarını okur, global durum kullanmaz.
    unsafe { ffi::swe_julday(year, month as i32, day as i32, hour_ut, ffi::SE_GREG_CAL) }
}
```

- [ ] **Step 8: Testi çalıştır, geçtiğini gör**

Run: `cargo test -p ephemeris --test ffi_smoke`
Expected: PASS (`julday_of_j2000_epoch_is_2451545 ... ok`).

`ffi.rs` içinde kullanılmayan sabitler için `dead_code` uyarısı çıkabilir. Task 3'te hepsi kullanılacak. Şimdilik `lib.rs`'teki `pub(crate) mod ffi;` satırının üstüne `#[allow(dead_code)]` ekle; Task 3'ün son adımında bu satırı kaldır.

- [ ] **Step 9: Commit**

```bash
git add Cargo.toml scripts/vendor-swisseph.sh crates/ephemeris data/ephe
git commit -m "feat(ephemeris): vendor Swiss Ephemeris 2.10.03 and compile via cc"
```

---

### Task 3: Güvenli API (`SwissEph`, tipler, hatalar, self-check)

**Files:**
- Create: `crates/ephemeris/src/types.rs`, `crates/ephemeris/src/error.rs`, `crates/ephemeris/src/swiss.rs`, `crates/ephemeris/tests/common/mod.rs`, `crates/ephemeris/tests/sanity.rs`
- Modify: `crates/ephemeris/src/lib.rs`

**Interfaces:**
- Consumes: `ffi` modülü, `julday_gregorian` (Task 2)
- Produces (sonraki planlar bunlara dayanır):
  - `pub enum Body { Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, MeanNode, TrueNode }`
  - `pub enum SiderealMode { Lahiri, Raman, Krishnamurti, TrueChitra, Custom { t0_jd_ut: f64, ayan_t0_deg: f64 } }`
  - `pub enum Zodiac { Tropical, Sidereal(SiderealMode) }`
  - `pub enum Precision { SwissEph, Moshier }`
  - `pub struct BodyPosition { lon, lat, distance_au, lon_speed, lat_speed, dist_speed: f64, precision: Precision }` ve `fn is_retrograde(&self) -> bool`
  - `pub enum EphError { InvalidPath, DataFilesMissing { path: String, detail: String }, Calc(String) }`
  - `pub trait Ephemeris { fn body(&mut self, jd_ut: f64, body: Body, zodiac: Zodiac) -> Result<BodyPosition, EphError>; fn ayanamsa(&mut self, jd_ut: f64, mode: SiderealMode) -> Result<f64, EphError>; }`
  - `pub struct SwissEph`: `fn open(path: &Path) -> Result<SwissEph, EphError>`, `fn version() -> String`, `impl Ephemeris`

- [ ] **Step 1: Test yardımcısını yaz**

`crates/ephemeris/tests/common/mod.rs`:

```rust
use std::path::PathBuf;

pub fn ephe_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/ephe")
}

/// İki açı arasındaki en kısa fark, derece.
pub fn angle_diff_deg(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}
```

- [ ] **Step 2: Failing sanity testlerini yaz**

`crates/ephemeris/tests/sanity.rs`:

```rust
mod common;

use common::{angle_diff_deg, ephe_dir};
use ephemeris::*;

const J2000: f64 = 2_451_545.0;

#[test]
fn sun_at_j2000_is_near_280_37_tropical_with_swisseph_precision() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let sun = eph.body(J2000, Body::Sun, Zodiac::Tropical).unwrap();
    assert!(angle_diff_deg(sun.lon, 280.37) < 0.05, "sun lon = {}", sun.lon);
    assert_eq!(sun.precision, Precision::SwissEph);
    assert!(!sun.is_retrograde());
}

#[test]
fn lahiri_ayanamsa_at_j2000_is_near_23_85() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let ayan = eph.ayanamsa(J2000, SiderealMode::Lahiri).unwrap();
    assert!((ayan - 23.85).abs() < 0.02, "ayanamsa = {ayan}");
}

#[test]
fn sidereal_longitude_equals_tropical_minus_ayanamsa() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    for body in [Body::Sun, Body::Moon, Body::Mars, Body::Saturn] {
        let trop = eph.body(J2000, body, Zodiac::Tropical).unwrap().lon;
        let sid = eph
            .body(J2000, body, Zodiac::Sidereal(SiderealMode::Lahiri))
            .unwrap()
            .lon;
        let ayan = eph.ayanamsa(J2000, SiderealMode::Lahiri).unwrap();
        let expected = (trop - ayan).rem_euclid(360.0);
        assert!(angle_diff_deg(sid, expected) < 0.001, "{body:?}: {sid} vs {expected}");
    }
}

#[test]
fn true_chitra_is_within_a_tenth_degree_of_lahiri() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let lahiri = eph.ayanamsa(J2000, SiderealMode::Lahiri).unwrap();
    let chitra = eph.ayanamsa(J2000, SiderealMode::TrueChitra).unwrap();
    assert!((lahiri - chitra).abs() < 0.1, "lahiri {lahiri}, chitra {chitra}");
}

#[test]
fn mean_node_is_always_retrograde() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    for jd in [J2000, J2000 + 1000.0, J2000 - 5000.0] {
        let rahu = eph.body(jd, Body::MeanNode, Zodiac::Tropical).unwrap();
        assert!(rahu.is_retrograde(), "jd {jd}: speed {}", rahu.lon_speed);
    }
}

#[test]
fn longitudes_are_normalized_to_0_360() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    for body in [Body::Sun, Body::Moon, Body::Mercury, Body::Venus, Body::Mars,
                 Body::Jupiter, Body::Saturn, Body::MeanNode, Body::TrueNode] {
        let p = eph.body(J2000, body, Zodiac::Sidereal(SiderealMode::Lahiri)).unwrap();
        assert!((0.0..360.0).contains(&p.lon), "{body:?}: {}", p.lon);
    }
}

#[test]
fn missing_data_directory_is_a_fatal_error() {
    let missing = std::env::temp_dir().join("jyotish-desk-no-such-ephe-dir");
    let err = SwissEph::open(&missing).err().expect("must fail");
    assert!(matches!(err, EphError::DataFilesMissing { .. }), "{err:?}");
}

#[test]
fn reopen_after_drop_works() {
    drop(SwissEph::open(&ephe_dir()).unwrap());
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    assert!(eph.body(J2000, Body::Moon, Zodiac::Tropical).is_ok());
}

#[test]
fn version_reports_2_10() {
    assert!(SwissEph::version().starts_with("2.10"), "{}", SwissEph::version());
}
```

- [ ] **Step 3: Testleri çalıştır, derleme hatası ver**

Run: `cargo test -p ephemeris --test sanity`
Expected: FAIL. `SwissEph`, `Body` ve diğer tipler tanımsız olduğu için derleme hatası.

- [ ] **Step 4: `types.rs` yaz**

```rust
use crate::ffi;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Body {
    Sun,
    Moon,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    MeanNode,
    TrueNode,
}

impl Body {
    pub(crate) fn se_id(self) -> i32 {
        match self {
            Self::Sun => 0,
            Self::Moon => 1,
            Self::Mercury => 2,
            Self::Venus => 3,
            Self::Mars => 4,
            Self::Jupiter => 5,
            Self::Saturn => 6,
            Self::MeanNode => 10,
            Self::TrueNode => 11,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SiderealMode {
    Lahiri,
    Raman,
    Krishnamurti,
    TrueChitra,
    /// Kullanıcı tanımlı: `t0_jd_ut` anında ayanamsa `ayan_t0_deg` derecedir.
    Custom { t0_jd_ut: f64, ayan_t0_deg: f64 },
}

impl SiderealMode {
    pub(crate) fn se_params(self) -> (i32, f64, f64) {
        match self {
            Self::Lahiri => (ffi::SE_SIDM_LAHIRI, 0.0, 0.0),
            Self::Raman => (ffi::SE_SIDM_RAMAN, 0.0, 0.0),
            Self::Krishnamurti => (ffi::SE_SIDM_KRISHNAMURTI, 0.0, 0.0),
            Self::TrueChitra => (ffi::SE_SIDM_TRUE_CITRA, 0.0, 0.0),
            Self::Custom { t0_jd_ut, ayan_t0_deg } => (ffi::SE_SIDM_USER, t0_jd_ut, ayan_t0_deg),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Zodiac {
    Tropical,
    Sidereal(SiderealMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    /// Swiss Ephemeris veri dosyalarıyla tam hassasiyet.
    SwissEph,
    /// Dosya aralığı dışında Moshier analitik yedeği (düşük hassasiyet).
    Moshier,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyPosition {
    pub lon: f64,
    pub lat: f64,
    pub distance_au: f64,
    pub lon_speed: f64,
    pub lat_speed: f64,
    pub dist_speed: f64,
    pub precision: Precision,
}

impl BodyPosition {
    pub fn is_retrograde(&self) -> bool {
        self.lon_speed < 0.0
    }
}
```

- [ ] **Step 5: `error.rs` yaz**

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EphError {
    #[error("ephemeris path is not valid UTF-8 or contains a NUL byte")]
    InvalidPath,
    #[error("ephemeris data files missing in {path}: {detail}")]
    DataFilesMissing { path: String, detail: String },
    #[error("swiss ephemeris error: {0}")]
    Calc(String),
}
```

- [ ] **Step 6: `swiss.rs` yaz**

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use crate::{ffi, Body, BodyPosition, EphError, Ephemeris, Precision, SiderealMode, Zodiac};

/// SE global durumunu koruyan süreç çapında kilit.
static SWE_LOCK: Mutex<()> = Mutex::new(());

const J2000: f64 = 2_451_545.0;

/// Swiss Ephemeris'in tek sahibi. Var olduğu sürece başka örnek açılamaz
/// (açmaya çalışan iş parçacığı bekler).
pub struct SwissEph {
    path: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

fn read_cstr(buf: &[c_char]) -> String {
    // SAFETY: SE tamponu her zaman NUL ile sonlandırır; tampon AS_MAXCH uzunluğunda.
    unsafe { CStr::from_ptr(buf.as_ptr()) }.to_string_lossy().into_owned()
}

impl SwissEph {
    /// Veri dizinini ayarlar ve self-check yapar. Güneş ya da Ay dosyası
    /// eksikse `DataFilesMissing` döner; sessiz Moshier yedeği yoktur.
    pub fn open(path: &Path) -> Result<Self, EphError> {
        let guard = SWE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let s = path.to_str().ok_or(EphError::InvalidPath)?;
        let c_path = CString::new(s).map_err(|_| EphError::InvalidPath)?;
        // SAFETY: kilit tutuluyor; SE yolu kendi tamponuna kopyalar.
        unsafe { ffi::swe_set_ephe_path(c_path.as_ptr()) };

        let mut eph = SwissEph { path: path.to_path_buf(), _guard: guard };
        eph.self_check()?;
        Ok(eph)
    }

    fn self_check(&mut self) -> Result<(), EphError> {
        for body in [Body::Sun, Body::Moon] {
            let pos = self.body(J2000, body, Zodiac::Tropical)?;
            if pos.precision != Precision::SwissEph {
                return Err(EphError::DataFilesMissing {
                    path: self.path.display().to_string(),
                    detail: format!("{body:?} at J2000 fell back to Moshier"),
                });
            }
        }
        Ok(())
    }

    pub fn version() -> String {
        let mut buf: [c_char; ffi::AS_MAXCH] = [0; ffi::AS_MAXCH];
        // SAFETY: swe_version sabit bir sürüm dizgisini tampona yazar.
        unsafe { ffi::swe_version(buf.as_mut_ptr()) };
        read_cstr(&buf)
    }

    fn set_sid_mode(mode: SiderealMode) {
        let (m, t0, ayan_t0) = mode.se_params();
        // SAFETY: yalnızca kilit sahibi çağırır (tüm çağıranlar &mut self).
        unsafe { ffi::swe_set_sid_mode(m, t0, ayan_t0) };
    }
}

impl Ephemeris for SwissEph {
    fn body(&mut self, jd_ut: f64, body: Body, zodiac: Zodiac) -> Result<BodyPosition, EphError> {
        let mut iflag = ffi::SEFLG_SWIEPH | ffi::SEFLG_SPEED;
        if let Zodiac::Sidereal(mode) = zodiac {
            Self::set_sid_mode(mode);
            iflag |= ffi::SEFLG_SIDEREAL;
        }
        let mut xx = [0.0_f64; 6];
        let mut serr: [c_char; ffi::AS_MAXCH] = [0; ffi::AS_MAXCH];
        // SAFETY: xx 6 double, serr AS_MAXCH; kilit tutuluyor.
        let ret = unsafe {
            ffi::swe_calc_ut(jd_ut, body.se_id(), iflag, xx.as_mut_ptr(), serr.as_mut_ptr())
        };
        if ret < 0 {
            return Err(EphError::Calc(read_cstr(&serr)));
        }
        let precision = if ret & ffi::SEFLG_MOSEPH != 0 {
            Precision::Moshier
        } else {
            Precision::SwissEph
        };
        Ok(BodyPosition {
            lon: xx[0].rem_euclid(360.0),
            lat: xx[1],
            distance_au: xx[2],
            lon_speed: xx[3],
            lat_speed: xx[4],
            dist_speed: xx[5],
            precision,
        })
    }

    fn ayanamsa(&mut self, jd_ut: f64, mode: SiderealMode) -> Result<f64, EphError> {
        Self::set_sid_mode(mode);
        let mut daya = 0.0_f64;
        let mut serr: [c_char; ffi::AS_MAXCH] = [0; ffi::AS_MAXCH];
        // SAFETY: daya tek double, serr AS_MAXCH; kilit tutuluyor.
        let ret = unsafe {
            ffi::swe_get_ayanamsa_ex_ut(jd_ut, ffi::SEFLG_SWIEPH, &mut daya, serr.as_mut_ptr())
        };
        if ret < 0 {
            return Err(EphError::Calc(read_cstr(&serr)));
        }
        Ok(daya)
    }
}

impl Drop for SwissEph {
    fn drop(&mut self) {
        // SAFETY: kilit hâlâ tutuluyor (_guard bu gövdeden sonra düşer).
        unsafe { ffi::swe_close() };
    }
}
```

- [ ] **Step 7: `lib.rs`'i güncelle**

`crates/ephemeris/src/lib.rs` dosyasının tamamını şununla değiştir:

```rust
//! Swiss Ephemeris için güvenli Rust sarmalayıcı.

pub(crate) mod ffi;
mod error;
mod swiss;
mod types;

pub use error::EphError;
pub use swiss::SwissEph;
pub use types::{Body, BodyPosition, Precision, SiderealMode, Zodiac};

/// Gök cismi konumu sağlayan motor. Üst katmanlar testte sahte motor takar.
pub trait Ephemeris {
    /// Jeosantrik, görünür konum; boylam [0, 360).
    fn body(&mut self, jd_ut: f64, body: Body, zodiac: Zodiac) -> Result<BodyPosition, EphError>;
    /// Nütasyon dahil (gerçek) ayanamsa, derece.
    fn ayanamsa(&mut self, jd_ut: f64, mode: SiderealMode) -> Result<f64, EphError>;
}

/// Gregoryen takvim tarihini ve UT saatini (ondalık) Jülyen gününe çevirir.
/// Saf bir hesaptır, SE global durumuna dokunmaz; kilit gerekmez.
pub fn julday_gregorian(year: i32, month: u32, day: u32, hour_ut: f64) -> f64 {
    // SAFETY: swe_julday yalnızca argümanlarını okur, global durum kullanmaz.
    unsafe { ffi::swe_julday(year, month as i32, day as i32, hour_ut, ffi::SE_GREG_CAL) }
}
```

Bu değişiklik Task 2'de eklenen `#[allow(dead_code)]` satırını da kaldırır.

- [ ] **Step 8: Testleri çalıştır, geçtiğini gör**

Run: `cargo test -p ephemeris`
Expected: PASS. `ffi_smoke` 1, `sanity` 9 test.

Bir testin başarısız olması durumları:
- **`sun_at_j2000...`:** Önce `data/ephe` dosyalarını kontrol et.
- **`missing_data_directory...` yanlışlıkla geçiyorsa:** SE varsayılan yolu (`/users/ephe`) kullanıyor olabilir. Makinede böyle bir dizin varsa testi o dizini geçici olarak yeniden adlandırıp tekrar çalıştır.

- [ ] **Step 9: Lint**

Run: `cargo fmt --all && cargo clippy -p ephemeris --all-targets -- -D warnings`
Expected: uyarı yok.

- [ ] **Step 10: Commit**

```bash
git add crates/ephemeris
git commit -m "feat(ephemeris): safe SwissEph API with single-owner lock and self-check"
```

---

### Task 4: `swetest` golden testi (≤ 0.001″)

**Files:**
- Create: `scripts/gen-swetest-fixtures.sh`, `crates/ephemeris/tests/fixtures/swetest.csv` (üretilir), `crates/ephemeris/tests/golden.rs`

**Interfaces:**
- Consumes: `SwissEph`, `Body`, `Zodiac`, `SiderealMode`, `julday_gregorian` (Task 3); vendored `swetest.c` (Task 2)
- Produces: Plan 04'teki golden altyapısının kullanacağı fixture biçimi: `date(d.m.yyyy),time(h:mm),body,mode(trop|lahiri),lon`

- [ ] **Step 1: Fixture üretim script'ini yaz**

`scripts/gen-swetest-fixtures.sh`:

```bash
#!/usr/bin/env bash
# Vendored kaynaktan swetest derler ve ephemeris golden fixture'ını üretir.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
V="$ROOT/crates/ephemeris/vendor/swisseph"
OUT="$ROOT/crates/ephemeris/tests/fixtures/swetest.csv"
BIN="$ROOT/target/swetest"
mkdir -p "$ROOT/target" "$(dirname "$OUT")"

cc -O2 -w -o "$BIN" "$V/swetest.c" \
  "$V/swedate.c" "$V/swehouse.c" "$V/swejpl.c" "$V/swemmoon.c" "$V/swemplan.c" \
  "$V/sweph.c" "$V/swephlib.c" "$V/swecl.c" "$V/swehel.c" -lm

DATES=(
  "1.1.1850 6:30"
  "15.6.1923 0:00"
  "29.10.1923 12:00"
  "17.3.1987 23:59"
  "1.1.2000 12:00"
  "21.12.2050 18:15"
)

echo "date,time,body,mode,lon" > "$OUT"
for dt in "${DATES[@]}"; do
  d="${dt% *}"; t="${dt#* }"
  for mode in trop lahiri; do
    sid=""; [ "$mode" = lahiri ] && sid="-sid1"
    raw="$("$BIN" -b"$d" -ut"$t" -p0123456mt -fPl -head -eswe -edir"$ROOT/data/ephe" $sid)"
    if echo "$raw" | grep -qi -e warning -e error; then
      echo "swetest reported a problem for $dt $mode:" >&2; echo "$raw" >&2; exit 1
    fi
    echo "$raw" | while IFS= read -r line; do
      [ -z "$line" ] && continue
      lon="$(echo "$line" | awk '{print $NF}')"
      name="$(echo "$line" | awk '{$NF=""; sub(/[ \t]+$/, ""); print}')"
      echo "$d,$t,$name,$mode,$lon" >> "$OUT"
    done
  done
done
wc -l "$OUT"
```

- [ ] **Step 2: Fixture'ı üret ve incele**

```bash
chmod +x scripts/gen-swetest-fixtures.sh
./scripts/gen-swetest-fixtures.sh
head -12 crates/ephemeris/tests/fixtures/swetest.csv
```

Beklenen:
- `109` satır: başlık + 6 tarih × 2 mod × 9 cisim.
- Gövde adları: `Sun`, `Moon`, `Mercury`, `Venus`, `Mars`, `Jupiter`, `Saturn`, `mean Node`, `true Node`.
- `1.1.2000,12:00,Sun,trop,...` satırında boylam yaklaşık `280.37`.

Adlar farklı yazılmışsa (ör. `Mean Node`), Step 3'teki `body_from_name` eşlemesini çıktıya göre düzelt.

- [ ] **Step 3: Golden testi yaz**

`crates/ephemeris/tests/golden.rs`:

```rust
mod common;

use common::{angle_diff_deg, ephe_dir};
use ephemeris::*;

const TOLERANCE_ARCSEC: f64 = 0.001;

fn body_from_name(name: &str) -> Body {
    match name {
        "Sun" => Body::Sun,
        "Moon" => Body::Moon,
        "Mercury" => Body::Mercury,
        "Venus" => Body::Venus,
        "Mars" => Body::Mars,
        "Jupiter" => Body::Jupiter,
        "Saturn" => Body::Saturn,
        "mean Node" => Body::MeanNode,
        "true Node" => Body::TrueNode,
        other => panic!("unknown body name in fixture: {other:?}"),
    }
}

fn parse_jd(date: &str, time: &str) -> f64 {
    let dmy: Vec<i32> = date.split('.').map(|x| x.parse().unwrap()).collect();
    let hm: Vec<f64> = time.split(':').map(|x| x.parse().unwrap()).collect();
    julday_gregorian(dmy[2], dmy[1] as u32, dmy[0] as u32, hm[0] + hm[1] / 60.0)
}

#[test]
fn matches_swetest_within_one_milliarcsecond() {
    let csv = include_str!("fixtures/swetest.csv");
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let mut checked = 0;

    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        assert_eq!(f.len(), 5, "bad fixture line: {line}");
        let jd = parse_jd(f[0], f[1]);
        let zodiac = match f[3] {
            "trop" => Zodiac::Tropical,
            "lahiri" => Zodiac::Sidereal(SiderealMode::Lahiri),
            other => panic!("unknown mode {other:?}"),
        };
        let expected: f64 = f[4].parse().unwrap();
        let pos = eph.body(jd, body_from_name(f[2]), zodiac).unwrap();
        assert_eq!(pos.precision, Precision::SwissEph, "{line}");
        let diff_arcsec = angle_diff_deg(pos.lon, expected) * 3600.0;
        assert!(diff_arcsec <= TOLERANCE_ARCSEC, "{line}: got {}, diff {diff_arcsec}\"", pos.lon);
        checked += 1;
    }

    assert_eq!(checked, 6 * 2 * 9);
}
```

- [ ] **Step 4: Testi çalıştır**

Run: `cargo test -p ephemeris --test golden`
Expected: PASS.

Test başarısız olursa önce hata mesajındaki farka bak:
- **Fark yaklaşık 0,0000003° (swetest'in 7 ondalık yuvarlaması) civarındaysa:** Toleransı gevşetme. `-fPl` yerine daha fazla ondalık veren biçimi (`-fPl -roundsec` kullanma) araştır ve fixture'ı yeniden üret.
- **Fark saniyeler mertebesindeyse:** `swetest`'in `-ut` saatini nasıl yorumladığını kontrol et (UT ile ET farkı).

- [ ] **Step 5: Commit**

```bash
git add scripts/gen-swetest-fixtures.sh crates/ephemeris/tests
git commit -m "test(ephemeris): golden comparison against swetest within 0.001 arcsec"
```

---

### Task 5: CI, cargo-deny ve x86_64 derleme kontrolü

**Files:**
- Create: `.github/workflows/ci.yml`, `deny.toml`

**Interfaces:**
- Consumes: workspace (Task 1–4)
- Produces: tüm sonraki planların PR'larda geçmesi gereken CI kapısı.

- [ ] **Step 1: Intel (x86_64) derlemesini yerelde doğrula**

Universal binary spike'ının ilk yarısı:

```bash
rustup target add x86_64-apple-darwin
cargo build -p ephemeris --target x86_64-apple-darwin
cargo test -p ephemeris --target x86_64-apple-darwin
```

Beklenen: Derleme başarılı. Apple Silicon'da testler Rosetta 2 ile çalışır ve PASS verir. Rosetta yüklü değilse test adımı "Bad CPU type" hatası verir; bu durumda yalnızca derlemenin başarılı olması yeterli.

- [ ] **Step 2: `deny.toml` yaz**

```toml
[graph]
all-features = true

[licenses]
allow = [
  "AGPL-3.0-or-later",
  "AGPL-3.0",
  "MIT",
  "Apache-2.0",
  "Apache-2.0 WITH LLVM-exception",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "ISC",
  "Unicode-3.0",
  "Zlib",
]
confidence-threshold = 0.9

[advisories]
yanked = "deny"

[bans]
multiple-versions = "warn"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

- [ ] **Step 3: cargo-deny'ı yerelde çalıştır**

```bash
cargo install cargo-deny --locked
cargo deny check
```

Beklenen: `advisories ok, bans ok, licenses ok, sources ok`.

Bir bağımlılık listede olmayan bir lisans kullanıyorsa, lisansı AGPL ile uyumluysa `allow` listesine ekle. Uyumsuzsa bağımlılığı değiştir.

- [ ] **Step 4: CI workflow'unu yaz**

`.github/workflows/ci.yml`:

```yaml
name: ci

on:
  push:
    branches: [main]
  pull_request:

jobs:
  core:
    name: core (${{ matrix.os }})
    strategy:
      fail-fast: false
      matrix:
        os: [macos-14, ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace

  macos-x86_64:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-apple-darwin
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --workspace --target x86_64-apple-darwin

  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2
```

Not: Bu planda workspace yalnızca çekirdek crate'lerden oluşuyor, bu yüzden üç işletim sisteminde de `--workspace` kullanılıyor. Plan 02 Tauri uygulamasını eklediğinde `ubuntu` ve `windows` işleri `-p ephemeris` (ve sonraki çekirdek crate'ler) ile sınırlandırılacak.

- [ ] **Step 5: Tüm yerel kontrolleri çalıştır**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Beklenen: Üçü de hatasız. `cargo test` 11 test geçer: smoke 1, sanity 9, golden 1.

- [ ] **Step 6: Commit**

```bash
git add .github/workflows/ci.yml deny.toml
git commit -m "ci: add fmt/clippy/test matrix, x86_64 macOS build and cargo-deny"
```

- [ ] **Step 7: GitHub'a gönder ve CI'ı doğrula (kullanıcı onayıyla)**

Repo oluşturmak ve push etmek dışa dönük işlemler, bu yüzden kullanıcıya sor. Onay gelirse:

```bash
gh repo create jyotish-desk --public --source . --push
gh run watch
```

Beklenen: `core (macos-14)`, `core (ubuntu-latest)`, `core (windows-latest)`, `macos-x86_64` ve `deny` işlerinin hepsi yeşil.

Windows'ta MSVC derleme hatası çıkarsa: SE kaynağı MSVC'yi resmen destekliyor. Hata mesajına göre `build.rs`'e `build.define("_CRT_SECURE_NO_WARNINGS", None)` gibi yalnızca gereken tanımı ekle.

---

## Bu planın tamamlanma ölçütü

- `cargo test --workspace` yeşil: 11 test, `swetest` ile ≤ 0.001″ uyum dahil.
- Eksik veri dizini ölümcül `DataFilesMissing` hatası veriyor.
- Hem arm64 hem x86_64 macOS için derleniyor.
- CI üç işletim sisteminde ve cargo-deny'da yeşil.
- `Ephemeris` trait'i ve tipleri Plan 04 (`jyotish-core`) için hazır.
