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
        assert!(
            diff_arcsec <= TOLERANCE_ARCSEC,
            "{line}: got {}, diff {diff_arcsec}\"",
            pos.lon
        );
        checked += 1;
    }

    assert_eq!(checked, 6 * 2 * 9);
}
