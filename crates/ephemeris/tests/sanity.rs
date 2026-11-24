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
        assert!(
            angle_diff_deg(sid, expected) < 0.001,
            "{body:?}: {sid} vs {expected}"
        );
    }
}

#[test]
fn true_chitra_is_within_a_tenth_degree_of_lahiri() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let lahiri = eph.ayanamsa(J2000, SiderealMode::Lahiri).unwrap();
    let chitra = eph.ayanamsa(J2000, SiderealMode::TrueChitra).unwrap();
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
