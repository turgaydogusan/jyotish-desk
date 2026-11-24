mod common;

use common::{angle_diff_deg, ephe_dir};
use ephemeris::*;

/// Plan 01 ile aynı tolerans; swetest 7 ondalık basar (≈ 0.00036″ yuvarlama).
const TOLERANCE_ARCSEC: f64 = 0.001;

// İstanbul, 17.3.1987 23:59 UT.
const IST_LAT: f64 = 41.0082;
const IST_LON: f64 = 28.9784;

fn ist_jd() -> f64 {
    julday_gregorian(1987, 3, 17, 23.0 + 59.0 / 60.0)
}

// target/swetest -b17.3.1987 -ut23:59 -p -house28.9784,41.0082,P -fPl -head -eswe -edirdata/ephe -sid1
const IST_PLACIDUS_LAHIRI: [f64; 12] = [
    247.7970775,
    286.9650884,
    328.5411204,
    1.9451259,
    27.0589213,
    47.7845930,
    67.7970775,
    106.9650884,
    148.5411204,
    181.9451259,
    207.0589213,
    227.7845930,
];
const IST_LAHIRI_ASC: f64 = 247.7970775;
const IST_LAHIRI_MC: f64 = 181.9451259;

// target/swetest -b17.3.1987 -ut23:59 -p -house28.9784,41.0082,P -fPl -head -eswe -edirdata/ephe
const IST_PLACIDUS_TROPICAL: [f64; 12] = [
    271.4744747,
    310.6424856,
    352.2185176,
    25.6225231,
    50.7363186,
    71.4619903,
    91.4744747,
    130.6424856,
    172.2185176,
    205.6225231,
    230.7363186,
    251.4619903,
];
const IST_TROPICAL_ASC: f64 = 271.4744747;
const IST_TROPICAL_MC: f64 = 205.6225231;

// target/swetest -b17.3.1987 -ut23:59 -p -house28.9784,41.0082,S -fPl -head -eswe -edirdata/ephe -sid1
// Sripati: dönen değerler bhava başlangıcıdır (sandhi).
const IST_SRIPATI_LAHIRI: [f64; 12] = [
    236.8217522,
    266.8217522,
    304.8711017,
    342.9204512,
    12.9204512,
    34.8711017,
    56.8217522,
    86.8217522,
    124.8711017,
    162.9204512,
    192.9204512,
    214.8711017,
];

// Tromsø, 21.12.2050 18:15 UT, kutup dairesi içinde. swetest "Placidus failed,
// Porphyry calculated instead" der:
// target/swetest -b21.12.2050 -ut18:15 -p -house18.9553,69.6492,P -fPl -head -eswe -edirdata/ephe -sid1
const TROMSO_LAT: f64 = 69.6492;
const TROMSO_LON: f64 = 18.9553;
const TROMSO_PORPHYRY_LAHIRI: [f64; 12] = [
    122.7704081,
    142.0099788,
    161.2495496,
    180.4891203,
    221.2495496,
    262.0099788,
    302.7704081,
    322.0099788,
    341.2495496,
    0.4891203,
    41.2495496,
    82.0099788,
];
const TROMSO_LAHIRI_ASC: f64 = 122.7704081;
const TROMSO_LAHIRI_MC: f64 = 0.4891203;

fn tromso_jd() -> f64 {
    julday_gregorian(2050, 12, 21, 18.25)
}

fn lahiri() -> Zodiac {
    Zodiac::Sidereal(SiderealMode::Lahiri)
}

fn assert_close(label: &str, got: f64, want: f64) {
    let diff = angle_diff_deg(got, want) * 3600.0;
    assert!(
        diff <= TOLERANCE_ARCSEC,
        "{label}: got {got}, want {want}, diff {diff}\""
    );
}

fn assert_cusps(label: &str, got: &[f64; 12], want: &[f64; 12]) {
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        assert_close(&format!("{label} cusp {}", i + 1), *g, *w);
    }
}

#[test]
fn placidus_sidereal_matches_swetest() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let h = eph
        .houses(ist_jd(), IST_LAT, IST_LON, HouseSystem::Placidus, lahiri())
        .unwrap();
    assert!(!h.porphyry_fallback);
    assert_cusps("placidus lahiri", &h.cusps, &IST_PLACIDUS_LAHIRI);
    assert_close("asc", h.asc, IST_LAHIRI_ASC);
    assert_close("mc", h.mc, IST_LAHIRI_MC);
}

#[test]
fn placidus_tropical_matches_swetest() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let h = eph
        .houses(
            ist_jd(),
            IST_LAT,
            IST_LON,
            HouseSystem::Placidus,
            Zodiac::Tropical,
        )
        .unwrap();
    assert_cusps("placidus tropical", &h.cusps, &IST_PLACIDUS_TROPICAL);
    assert_close("asc", h.asc, IST_TROPICAL_ASC);
    assert_close("mc", h.mc, IST_TROPICAL_MC);
}

#[test]
fn sripati_sidereal_matches_swetest_sandhi_cusps() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let h = eph
        .houses(ist_jd(), IST_LAT, IST_LON, HouseSystem::Sripati, lahiri())
        .unwrap();
    assert!(!h.porphyry_fallback);
    assert_cusps("sripati lahiri", &h.cusps, &IST_SRIPATI_LAHIRI);
    assert_close("asc", h.asc, IST_LAHIRI_ASC);
}

#[test]
fn whole_sign_cusps_start_at_lagna_sign() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let h = eph
        .houses(ist_jd(), IST_LAT, IST_LON, HouseSystem::WholeSign, lahiri())
        .unwrap();
    // Lagna 247.797° (Yay); Whole Sign cusp'ları 240°'den 30'ar derece.
    let want: [f64; 12] = std::array::from_fn(|i| (240.0 + 30.0 * i as f64) % 360.0);
    assert_cusps("whole sign", &h.cusps, &want);
    assert_close("asc", h.asc, IST_LAHIRI_ASC);
}

#[test]
fn sidereal_ascendant_is_tropical_minus_ayanamsa() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let trop = eph
        .houses(
            ist_jd(),
            IST_LAT,
            IST_LON,
            HouseSystem::WholeSign,
            Zodiac::Tropical,
        )
        .unwrap();
    let sid = eph
        .houses(ist_jd(), IST_LAT, IST_LON, HouseSystem::WholeSign, lahiri())
        .unwrap();
    let ayan = eph.ayanamsa(ist_jd(), SiderealMode::Lahiri).unwrap().deg;
    assert_close("asc", sid.asc, (trop.asc - ayan).rem_euclid(360.0));
}

#[test]
fn placidus_inside_polar_circle_falls_back_to_porphyry() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let h = eph
        .houses(
            tromso_jd(),
            TROMSO_LAT,
            TROMSO_LON,
            HouseSystem::Placidus,
            lahiri(),
        )
        .unwrap();
    assert!(h.porphyry_fallback);
    assert_cusps("tromso porphyry", &h.cusps, &TROMSO_PORPHYRY_LAHIRI);
    assert_close("asc", h.asc, TROMSO_LAHIRI_ASC);
    assert_close("mc", h.mc, TROMSO_LAHIRI_MC);

    let s = eph
        .houses(
            tromso_jd(),
            TROMSO_LAT,
            TROMSO_LON,
            HouseSystem::Sripati,
            lahiri(),
        )
        .unwrap();
    assert!(!s.porphyry_fallback, "Sripati never falls back");
}

#[test]
fn swe_version_is_available_through_the_trait() {
    let eph = SwissEph::open(&ephe_dir()).unwrap();
    let v = Ephemeris::swe_version(&eph);
    assert_eq!(v, SwissEph::version());
    assert!(v.starts_with("2.10"), "version {v}");
}
