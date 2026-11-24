use atlas::{resolve_moment, MomentError, MomentWarning, NaiveDate, NaiveTime, TzSpec};
use chrono::{TimeZone, Timelike, Utc};

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}
fn t(h: u32, mi: u32) -> Option<NaiveTime> {
    Some(NaiveTime::from_hms_opt(h, mi, 0).unwrap())
}

#[test]
fn unix_epoch_is_jd_2440587_5() {
    let m = resolve_moment(d(1970, 1, 1), t(0, 0), &TzSpec::Fixed(0), 0.0, None).unwrap();
    assert_eq!(m.jd_ut, 2440587.5);
    assert_eq!(m.utc, Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap());
    assert_eq!(m.abbrev, "UTC+00:00");
    assert!(m.warnings.is_empty());
}

#[test]
fn j2000_noon_is_jd_2451545() {
    let m = resolve_moment(d(2000, 1, 1), t(12, 0), &TzSpec::Fixed(0), 0.0, None).unwrap();
    assert_eq!(m.jd_ut, 2451545.0);
}

#[test]
fn fixed_positive_half_hour_offset() {
    let m = resolve_moment(d(1990, 6, 15), t(10, 0), &TzSpec::Fixed(19_800), 77.2, None).unwrap();
    assert_eq!(m.utc, Utc.with_ymd_and_hms(1990, 6, 15, 4, 30, 0).unwrap());
    assert_eq!(m.jd_ut, 2448057.6875);
    assert_eq!(m.offset_secs, 19_800);
    assert_eq!(m.abbrev, "UTC+05:30");
}

#[test]
fn fixed_negative_offset() {
    let m = resolve_moment(d(2000, 1, 1), t(0, 0), &TzSpec::Fixed(-12_600), -52.7, None).unwrap();
    assert_eq!(m.utc, Utc.with_ymd_and_hms(2000, 1, 1, 3, 30, 0).unwrap());
    assert_eq!(m.abbrev, "UTC-03:30");
}

#[test]
fn fixed_offset_out_of_range_is_error() {
    let r = resolve_moment(
        d(2000, 1, 1),
        t(0, 0),
        &TzSpec::Fixed(18 * 3600 + 1),
        0.0,
        None,
    );
    assert_eq!(r, Err(MomentError::OffsetOutOfRange(18 * 3600 + 1)));
    assert!(resolve_moment(
        d(2000, 1, 1),
        t(0, 0),
        &TzSpec::Fixed(-18 * 3600),
        0.0,
        None
    )
    .is_ok());
}

#[test]
fn lmt_east_keeps_fractional_seconds() {
    // İstanbul boylamı 28.9784° → 6954.816 s.
    let m = resolve_moment(d(1870, 5, 1), t(12, 0), &TzSpec::Lmt, 28.9784, None).unwrap();
    assert_eq!(m.offset_secs, 6955);
    assert_eq!(m.abbrev, "LMT");
    assert_eq!((m.utc.hour(), m.utc.minute(), m.utc.second()), (10, 4, 5));
    assert_eq!(m.utc.nanosecond(), 184_000_000);
    assert!(
        (m.jd_ut - 2404183.9195044446).abs() < 1e-9,
        "jd {}",
        m.jd_ut
    );
}

#[test]
fn lmt_west_is_negative() {
    // -74.006° → -17761.44 s.
    let m = resolve_moment(d(1880, 1, 1), t(12, 0), &TzSpec::Lmt, -74.006, None).unwrap();
    assert_eq!(m.offset_secs, -17_761);
    assert_eq!((m.utc.hour(), m.utc.minute(), m.utc.second()), (16, 56, 1));
    assert_eq!(m.utc.nanosecond(), 440_000_000);
}

#[test]
fn unknown_time_uses_local_noon_with_warning() {
    let m = resolve_moment(d(2000, 1, 1), None, &TzSpec::Fixed(0), 0.0, None).unwrap();
    assert_eq!(m.jd_ut, 2451545.0);
    assert_eq!(m.warnings, vec![MomentWarning::TimeUnknownNoon]);
}

#[test]
fn longitude_must_be_finite_and_in_range() {
    for lon in [f64::NAN, f64::INFINITY, 180.5, -181.0] {
        let r = resolve_moment(d(2000, 1, 1), t(0, 0), &TzSpec::Lmt, lon, None);
        assert!(
            matches!(r, Err(MomentError::LongitudeOutOfRange(_))),
            "lon {lon}"
        );
    }
}

#[test]
fn tzdb_version_is_exposed() {
    assert_eq!(atlas::TZDB_VERSION.len(), 5);
}
