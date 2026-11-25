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
    eph.sun_rise_set(start, ev, lat, lon, disc)
        .expect("rise_trans")
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
            assert_eq!(
                event(&mut eph, start, ev, TROMSO, disc),
                None,
                "{disc:?} {ev:?}"
            );
        }
    }
}

#[test]
fn next_event_comes_after_the_start() {
    let mut eph = SwissEph::open(&ephe_dir()).unwrap();
    let hindu = SunDisc::CenterNoRefraction;
    let first = event(
        &mut eph,
        ut(2000, 1, 1, 0.0, 0.0, 0.0),
        RiseSet::Rise,
        DELHI,
        hindu,
    )
    .unwrap();
    // Bulunan doğuşun hemen sonrasından aramak ertesi günün doğuşunu verir (≈ 2451545.575167).
    let next = event(&mut eph, first + 0.01, RiseSet::Rise, DELHI, hindu).unwrap();
    assert!((0.99..1.01).contains(&(next - first)), "{}", next - first);
}
