mod common;

use common::{angle_diff_deg, ephe_dir};
use ephemeris::*;

const J2000: f64 = 2_451_545.0;

#[test]
fn sun_at_j2000_is_near_280_37_tropical_with_swisseph_precision() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let sun = eph.body(J2000, Body::Sun, Zodiac::Tropical).unwrap();
    assert!(
        angle_diff_deg(sun.lon, 280.37) < 0.05,
        "sun lon = {}",
        sun.lon
    );
    assert_eq!(sun.precision, Precision::SwissEph);
    assert!(!sun.is_retrograde());
}

#[test]
fn lahiri_ayanamsa_at_j2000_is_near_23_85() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let ayan = eph.ayanamsa(J2000, SiderealMode::Lahiri).unwrap().deg;
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
        let ayan = eph.ayanamsa(J2000, SiderealMode::Lahiri).unwrap().deg;
        let expected = (trop - ayan).rem_euclid(360.0);
        assert!(
            angle_diff_deg(sid, expected) < 0.001,
            "{body:?}: {sid} vs {expected}"
        );
    }
}

#[test]
fn true_chitra_is_within_a_tenth_degree_of_lahiri() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let lahiri = eph.ayanamsa(J2000, SiderealMode::Lahiri).unwrap().deg;
    let chitra = eph.ayanamsa(J2000, SiderealMode::TrueChitra).unwrap().deg;
    assert!(
        (lahiri - chitra).abs() < 0.1,
        "lahiri {lahiri}, chitra {chitra}"
    );
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
    for body in [
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::MeanNode,
        Body::TrueNode,
    ] {
        let p = eph
            .body(J2000, body, Zodiac::Sidereal(SiderealMode::Lahiri))
            .unwrap();
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
    assert!(
        SwissEph::version().starts_with("2.10"),
        "{}",
        SwissEph::version()
    );
}

#[test]
fn over_long_path_is_rejected() {
    let long = std::env::temp_dir().join("a".repeat(300));
    assert!(matches!(SwissEph::open(&long), Err(EphError::InvalidPath)));
}

#[cfg(unix)]
#[test]
fn path_with_nul_byte_is_rejected() {
    use std::os::unix::ffi::OsStrExt;
    let p = std::path::Path::new(std::ffi::OsStr::from_bytes(b"/tmp/a\0b"));
    assert!(matches!(SwissEph::open(p), Err(EphError::InvalidPath)));
}

#[test]
fn out_of_range_date_falls_back_to_flagged_moshier() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let jd = julday_gregorian(3000, 1, 1, 12.0);
    let sun = eph.body(jd, Body::Sun, Zodiac::Tropical).unwrap();
    assert_eq!(sun.precision, Precision::Moshier);
}

#[test]
fn ayanamsa_reports_precision() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let in_range = eph.ayanamsa(J2000, SiderealMode::Lahiri).unwrap();
    assert_eq!(in_range.precision, Precision::SwissEph);
    // Gözlem (SE 2.10.03 ile doğrulandı): swe_get_ayanamsa_ex_ut, dosya aralığı
    // dışında (yıl 3000) bile MOSEPH bayrağı döndürmez; ayanamsa Spica'nın
    // (swe_fixstar) konumundan gelir ve dönen bayrak istenen SWIEPH kalır.
    // Yani `precision` ayanamsa için SE'nin bildirdiği bayraktır; gezegen
    // konumunun hassasiyeti için `body().precision` esas alınmalıdır.
    let jd = julday_gregorian(3000, 1, 1, 12.0);
    let out = eph.ayanamsa(jd, SiderealMode::TrueChitra).unwrap();
    assert_eq!(out.precision, Precision::SwissEph);
    let sun = eph
        .body(jd, Body::Sun, Zodiac::Sidereal(SiderealMode::TrueChitra))
        .unwrap();
    assert_eq!(sun.precision, Precision::Moshier);
}

#[test]
fn custom_ayanamsa_epoch_is_interpreted_as_ut() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let mode = SiderealMode::Custom {
        t0_jd_ut: J2000,
        ayan_t0_deg: 23.5,
    };
    let a = eph.ayanamsa(J2000, mode).unwrap();
    // swe_get_ayanamsa_ex_ut nütasyonu (Δψ ≈ -13.93" = -0.0038694° @ J2000)
    // sonuca ekler (SEFLG_NONUT verilmedikçe); trait sözleşmesi de "nütasyon dahil".
    let expected = 23.5 - 13.93 / 3600.0;
    assert!((a.deg - expected).abs() < 1e-5, "ayanamsa = {}", a.deg);
}

#[cfg(unix)]
#[test]
fn path_with_colon_is_rejected() {
    let p = std::env::temp_dir().join("a:b");
    assert!(matches!(SwissEph::open(&p), Err(EphError::InvalidPath)));
}
