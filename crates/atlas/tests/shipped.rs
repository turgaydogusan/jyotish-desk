//! Commit edilmiş data/atlas/atlas.sqlite için sağlamlık kontrolleri.

use std::path::PathBuf;

use atlas::{resolve_moment, Atlas, TzSpec};
use jiff::civil::{date, time};

fn shipped() -> Atlas {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/atlas/atlas.sqlite");
    Atlas::open(&p).expect("data/atlas/atlas.sqlite missing; run scripts/fetch-geonames.sh")
}

#[test]
fn has_expected_size() {
    let a = shipped();
    assert!(a.row_count().unwrap() > 100_000);
    assert_eq!(a.source_date().unwrap().len(), 10);
}

#[test]
fn major_turkish_cities_resolve() {
    let a = shipped();
    for (q, tz) in [
        ("istanbul", "Europe/Istanbul"),
        ("ankara", "Europe/Istanbul"),
        ("izmir", "Europe/Istanbul"),
    ] {
        let hits = a.search(q, 1).unwrap();
        let p = hits.first().unwrap_or_else(|| panic!("{q}: no result"));
        assert_eq!(p.country_code, "TR", "{q}");
        assert_eq!(p.timezone, tz, "{q}");
    }
}

#[test]
fn every_sampled_place_has_a_resolvable_time_zone() {
    let a = shipped();
    let (d, t) = (date(2000, 6, 1), Some(time(12, 0, 0, 0)));
    for q in [
        "paris", "london", "new york", "delhi", "tokyo", "sydney", "konya", "trabzon",
    ] {
        let hits = a.search(q, 1).unwrap();
        let p = hits.first().unwrap_or_else(|| panic!("{q}: no result"));
        resolve_moment(d, t, &TzSpec::Iana(p.timezone.clone()), p.lon, None)
            .unwrap_or_else(|e| panic!("{q}: {e}"));
    }
}
