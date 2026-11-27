use atlas::{resolve_moment, MomentError, MomentWarning, Timestamp, TzSpec};
use jiff::civil::{date, time, DateTime};
use jiff::tz::Offset;

fn ts(s: &str) -> Timestamp {
    s.parse().unwrap()
}
/// UTC takvim saati (saniye altı pozitif).
fn civil_utc(t: Timestamp) -> DateTime {
    Offset::UTC.to_datetime(t)
}

#[test]
fn unix_epoch_is_jd_2440587_5() {
    let m = resolve_moment(
        date(1970, 1, 1),
        Some(time(0, 0, 0, 0)),
        &TzSpec::Fixed(0),
        0.0,
        None,
    )
    .unwrap();
    assert_eq!(m.jd_ut, 2440587.5);
    assert_eq!(m.utc, ts("1970-01-01T00:00:00Z"));
    assert_eq!(m.abbrev, "UTC+00:00");
    assert!(m.warnings.is_empty());
}

#[test]
fn j2000_noon_is_jd_2451545() {
    let m = resolve_moment(
        date(2000, 1, 1),
        Some(time(12, 0, 0, 0)),
        &TzSpec::Fixed(0),
        0.0,
        None,
    )
    .unwrap();
    assert_eq!(m.jd_ut, 2451545.0);
}

#[test]
fn fixed_positive_half_hour_offset() {
    let m = resolve_moment(
        date(1990, 6, 15),
        Some(time(10, 0, 0, 0)),
        &TzSpec::Fixed(19_800),
        77.2,
        None,
    )
    .unwrap();
    assert_eq!(m.utc, ts("1990-06-15T04:30:00Z"));
    assert_eq!(m.jd_ut, 2448057.6875);
    assert_eq!(m.offset_secs, 19_800);
    assert_eq!(m.abbrev, "UTC+05:30");
}

#[test]
fn fixed_negative_offset() {
    let m = resolve_moment(
        date(2000, 1, 1),
        Some(time(0, 0, 0, 0)),
        &TzSpec::Fixed(-12_600),
        -52.7,
        None,
    )
    .unwrap();
    assert_eq!(m.utc, ts("2000-01-01T03:30:00Z"));
    assert_eq!(m.abbrev, "UTC-03:30");
}

#[test]
fn fixed_offset_out_of_range_is_error() {
    let r = resolve_moment(
        date(2000, 1, 1),
        Some(time(0, 0, 0, 0)),
        &TzSpec::Fixed(18 * 3600 + 1),
        0.0,
        None,
    );
    assert_eq!(r, Err(MomentError::OffsetOutOfRange(18 * 3600 + 1)));
    assert!(resolve_moment(
        date(2000, 1, 1),
        Some(time(0, 0, 0, 0)),
        &TzSpec::Fixed(-18 * 3600),
        0.0,
        None
    )
    .is_ok());
}

#[test]
fn lmt_east_keeps_fractional_seconds() {
    // İstanbul boylamı 28.9784° → 6954.816 s.
    let m = resolve_moment(
        date(1870, 5, 1),
        Some(time(12, 0, 0, 0)),
        &TzSpec::Lmt,
        28.9784,
        None,
    )
    .unwrap();
    assert_eq!(m.offset_secs, 6955);
    assert_eq!(m.abbrev, "LMT");
    let u = civil_utc(m.utc);
    assert_eq!((u.hour(), u.minute(), u.second()), (10, 4, 5));
    assert_eq!(u.subsec_nanosecond(), 184_000_000);
    assert!(
        (m.jd_ut - 2404183.9195044446).abs() < 1e-9,
        "jd {}",
        m.jd_ut
    );
}

#[test]
fn lmt_west_is_negative() {
    // -74.006° → -17761.44 s.
    let m = resolve_moment(
        date(1880, 1, 1),
        Some(time(12, 0, 0, 0)),
        &TzSpec::Lmt,
        -74.006,
        None,
    )
    .unwrap();
    assert_eq!(m.offset_secs, -17_761);
    let u = civil_utc(m.utc);
    assert_eq!((u.hour(), u.minute(), u.second()), (16, 56, 1));
    assert_eq!(u.subsec_nanosecond(), 440_000_000);
}

#[test]
fn unknown_time_uses_local_noon_with_warning() {
    let m = resolve_moment(date(2000, 1, 1), None, &TzSpec::Fixed(0), 0.0, None).unwrap();
    assert_eq!(m.jd_ut, 2451545.0);
    assert_eq!(m.warnings, vec![MomentWarning::TimeUnknownNoon]);
}

#[test]
fn longitude_must_be_finite_and_in_range() {
    for lon in [f64::NAN, f64::INFINITY, 180.5, -181.0] {
        let r = resolve_moment(
            date(2000, 1, 1),
            Some(time(0, 0, 0, 0)),
            &TzSpec::Lmt,
            lon,
            None,
        );
        assert!(
            matches!(r, Err(MomentError::LongitudeOutOfRange(_))),
            "lon {lon}"
        );
    }
}

#[test]
fn tzdb_version_is_exposed() {
    let v = atlas::tzdb_version();
    assert_eq!(v.len(), 5, "{v}");
    assert!(v.starts_with("20"), "{v}");
}

#[test]
fn offset_at_for_fixed_offsets_and_local_mean_time() {
    let utc: atlas::Timestamp = "2026-11-26T12:00:00Z".parse().unwrap();
    assert_eq!(
        atlas::offset_at(&TzSpec::Fixed(19_800), 77.2, utc),
        Ok(19_800)
    );
    assert_eq!(
        atlas::offset_at(&TzSpec::Fixed(-3_600), 0.0, utc),
        Ok(-3_600)
    );
    // 77,209° × 240 = 18 530,16 sn.
    assert_eq!(atlas::offset_at(&TzSpec::Lmt, 77.209, utc), Ok(18_530));
    assert_eq!(atlas::offset_at(&TzSpec::Lmt, -58.3816, utc), Ok(-14_012));
    assert_eq!(
        atlas::offset_at(&TzSpec::Fixed(18 * 3600 + 1), 0.0, utc),
        Err(MomentError::OffsetOutOfRange(18 * 3600 + 1))
    );
    for lon in [f64::NAN, 180.5, -181.0] {
        assert!(matches!(
            atlas::offset_at(&TzSpec::Lmt, lon, utc),
            Err(MomentError::LongitudeOutOfRange(_))
        ));
    }
}
