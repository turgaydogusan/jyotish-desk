use atlas::{
    is_known_time_zone, resolve_moment, Fold, MomentError, MomentWarning, Timestamp, TzSpec,
};
use jiff::civil::{date, time};

fn ts(s: &str) -> Timestamp {
    s.parse().unwrap()
}
fn iana(id: &str) -> TzSpec {
    TzSpec::Iana(id.to_owned())
}

/// (zone, yıl, ay, gün, saat, dakika, beklenen ofset, beklenen kısaltma, beklenen UTC).
/// Değerler jiff 1.0.3 / gömülü tzdb 2026f ile doğrulandı (chrono-tz ile aynı).
#[allow(clippy::type_complexity)]
const TABLE: &[(&str, i16, i8, i8, i8, i8, i32, &str, &str)] = &[
    (
        "Europe/Istanbul",
        1925,
        1,
        1,
        12,
        0,
        7200,
        "EET",
        "1925-01-01T10:00:00Z",
    ),
    (
        "Europe/Istanbul",
        1942,
        6,
        1,
        12,
        0,
        10800,
        "EEST",
        "1942-06-01T09:00:00Z",
    ),
    (
        "Europe/Istanbul",
        1960,
        1,
        1,
        12,
        0,
        7200,
        "EET",
        "1960-01-01T10:00:00Z",
    ),
    (
        "Europe/Istanbul",
        1978,
        7,
        1,
        12,
        0,
        10800,
        "+03",
        "1978-07-01T09:00:00Z",
    ),
    (
        "Europe/Istanbul",
        1985,
        1,
        1,
        12,
        0,
        7200,
        "EET",
        "1985-01-01T10:00:00Z",
    ),
    (
        "Europe/Istanbul",
        2011,
        3,
        27,
        12,
        0,
        7200,
        "EET",
        "2011-03-27T10:00:00Z",
    ),
    (
        "Europe/Istanbul",
        2011,
        3,
        28,
        12,
        0,
        10800,
        "EEST",
        "2011-03-28T09:00:00Z",
    ),
    (
        "Europe/Istanbul",
        2014,
        3,
        30,
        12,
        0,
        7200,
        "EET",
        "2014-03-30T10:00:00Z",
    ),
    (
        "Europe/Istanbul",
        2015,
        11,
        1,
        12,
        0,
        10800,
        "EEST",
        "2015-11-01T09:00:00Z",
    ),
    (
        "Europe/Istanbul",
        2015,
        11,
        10,
        12,
        0,
        7200,
        "EET",
        "2015-11-10T10:00:00Z",
    ),
    (
        "Europe/Istanbul",
        2016,
        9,
        7,
        12,
        0,
        10800,
        "+03",
        "2016-09-07T09:00:00Z",
    ),
    (
        "Europe/Istanbul",
        2016,
        12,
        1,
        12,
        0,
        10800,
        "+03",
        "2016-12-01T09:00:00Z",
    ),
    (
        "Asia/Kolkata",
        1990,
        6,
        15,
        10,
        0,
        19800,
        "IST",
        "1990-06-15T04:30:00Z",
    ),
];

#[test]
fn offset_table() {
    for &(zone, y, mo, day, h, mi, off, abbr, utc) in TABLE {
        let m = resolve_moment(
            date(y, mo, day),
            Some(time(h, mi, 0, 0)),
            &iana(zone),
            30.0,
            None,
        )
        .unwrap_or_else(|e| panic!("{zone} {y}-{mo}-{day}: {e}"));
        assert_eq!(m.offset_secs, off, "{zone} {y}-{mo}-{day} offset");
        assert_eq!(m.abbrev, abbr, "{zone} {y}-{mo}-{day} abbrev");
        assert_eq!(m.utc, ts(utc), "{zone} {y}-{mo}-{day} utc");
        let expected_warn = if y < 1970 {
            vec![MomentWarning::PreTzdbReliability]
        } else {
            vec![]
        };
        assert_eq!(m.warnings, expected_warn, "{zone} {y}-{mo}-{day} warnings");
    }
}

#[test]
fn overlap_requires_fold() {
    let r = resolve_moment(
        date(2010, 10, 31),
        Some(time(3, 30, 0, 0)),
        &iana("Europe/Istanbul"),
        29.0,
        None,
    );
    assert!(matches!(
        r,
        Err(MomentError::AmbiguousLocalTime {
            earlier_offset_secs: 10800,
            later_offset_secs: 7200,
            ..
        })
    ));
}

#[test]
fn overlap_fold_selects_instant() {
    let z = iana("Europe/Istanbul");
    let at = Some(time(3, 30, 0, 0));
    let e = resolve_moment(date(2010, 10, 31), at, &z, 29.0, Some(Fold::Earlier)).unwrap();
    let l = resolve_moment(date(2010, 10, 31), at, &z, 29.0, Some(Fold::Later)).unwrap();
    assert_eq!(e.utc, ts("2010-10-31T00:30:00Z"));
    assert_eq!(l.utc, ts("2010-10-31T01:30:00Z"));

    let z = iana("America/New_York");
    let at = Some(time(1, 30, 0, 0));
    let e = resolve_moment(date(2021, 11, 7), at, &z, -74.0, Some(Fold::Earlier)).unwrap();
    let l = resolve_moment(date(2021, 11, 7), at, &z, -74.0, Some(Fold::Later)).unwrap();
    assert!((e.jd_ut - 2459525.7291666665).abs() < 1e-9);
    assert!((l.jd_ut - 2459525.7708333335).abs() < 1e-9);
}

#[test]
fn fold_is_ignored_when_unambiguous() {
    let z = iana("Europe/Istanbul");
    let at = Some(time(12, 0, 0, 0));
    let a = resolve_moment(date(2016, 12, 1), at, &z, 29.0, None).unwrap();
    let b = resolve_moment(date(2016, 12, 1), at, &z, 29.0, Some(Fold::Later)).unwrap();
    assert_eq!(a, b);
}

#[test]
fn gap_is_error_with_forward_suggestion() {
    let r = resolve_moment(
        date(2014, 3, 31),
        Some(time(3, 30, 0, 0)),
        &iana("Europe/Istanbul"),
        29.0,
        None,
    );
    match r {
        Err(MomentError::NonexistentLocalTime { suggestion, .. }) => {
            assert_eq!(suggestion, date(2014, 3, 31).at(4, 30, 0, 0));
        }
        other => panic!("expected gap error, got {other:?}"),
    }
    let r = resolve_moment(
        date(2021, 3, 14),
        Some(time(2, 30, 0, 0)),
        &iana("America/New_York"),
        -74.0,
        None,
    );
    match r {
        Err(MomentError::NonexistentLocalTime { suggestion, .. }) => {
            assert_eq!(suggestion, date(2021, 3, 14).at(3, 30, 0, 0));
        }
        other => panic!("expected gap error, got {other:?}"),
    }
}

#[test]
fn tzdb_lmt_is_replaced_by_birthplace_lmt() {
    // tzdb: İstanbul 1870 LMT +1:55:52; doğum yeri boylamı 28.9784° → 6954.816 s.
    let m = resolve_moment(
        date(1870, 5, 1),
        Some(time(12, 0, 0, 0)),
        &iana("Europe/Istanbul"),
        28.9784,
        None,
    )
    .unwrap();
    assert_eq!(m.offset_secs, 6955);
    assert_eq!(m.abbrev, "LMT");
    assert_eq!(m.warnings, vec![MomentWarning::LmtFromLongitude]);
    assert!((m.jd_ut - 2404183.9195044446).abs() < 1e-9);

    // tzdb: New York 1880 LMT -4:56:02; -74.006° → -17761.44 s.
    let m = resolve_moment(
        date(1880, 1, 1),
        Some(time(12, 0, 0, 0)),
        &iana("America/New_York"),
        -74.006,
        None,
    )
    .unwrap();
    assert_eq!(m.offset_secs, -17_761);
    assert_eq!(m.abbrev, "LMT");
    assert_eq!(m.warnings, vec![MomentWarning::LmtFromLongitude]);
}

#[test]
fn whole_minute_tzdb_lmt_is_replaced_by_birthplace_lmt() {
    // tzdb: Varşova 1870 LMT +1:24:00 (tam dakika); Kraków 19.94° → 4785.6 s.
    let m = resolve_moment(
        date(1870, 6, 1),
        Some(time(12, 0, 0, 0)),
        &iana("Europe/Warsaw"),
        19.94,
        None,
    )
    .unwrap();
    assert_eq!(m.offset_secs, 4786);
    assert_eq!(m.abbrev, "LMT");
    assert_eq!(m.warnings, vec![MomentWarning::LmtFromLongitude]);
}

#[test]
fn named_mean_time_keeps_tzdb_offset() {
    // tzdb: 1900'de IMT +1:56:56 (İstanbul ortalama saati). Ankara boylamı verilse de
    // tzdb ofseti korunur; doğum yeri LMT'si isteyen TzSpec::Lmt kullanır.
    let m = resolve_moment(
        date(1900, 1, 1),
        Some(time(12, 0, 0, 0)),
        &iana("Europe/Istanbul"),
        32.8597,
        None,
    )
    .unwrap();
    assert_eq!(m.offset_secs, 7016);
    assert_eq!(m.abbrev, "IMT");
    assert_eq!(m.utc, ts("1900-01-01T10:03:04Z"));
    assert_eq!(m.warnings, vec![MomentWarning::PreTzdbReliability]);
}

#[test]
fn non_minute_dst_offset_is_kept() {
    // tzdb: Dublin 1916 yazı IST +0:34:39 (Dunsink ortalama saati + 1 saat).
    let m = resolve_moment(
        date(1916, 7, 1),
        Some(time(12, 0, 0, 0)),
        &iana("Europe/Dublin"),
        -6.26,
        None,
    )
    .unwrap();
    assert_eq!(m.offset_secs, 2079);
    assert_eq!(m.abbrev, "IST");
    assert_eq!(m.warnings, vec![MomentWarning::PreTzdbReliability]);

    // tzdb: St. John's 1925 yazı NDT -2:30:52.
    let m = resolve_moment(
        date(1925, 7, 1),
        Some(time(12, 0, 0, 0)),
        &iana("America/St_Johns"),
        -52.71,
        None,
    )
    .unwrap();
    assert_eq!(m.offset_secs, -9052);
    assert_eq!(m.abbrev, "NDT");
    assert_eq!(m.warnings, vec![MomentWarning::PreTzdbReliability]);
}

#[test]
fn named_mean_time_after_1970_warns_reliability() {
    // tzdb: Monrovia 1972'ye kadar MMT -0:44:30.
    let m = resolve_moment(
        date(1971, 6, 1),
        Some(time(12, 0, 0, 0)),
        &iana("Africa/Monrovia"),
        -10.8,
        None,
    )
    .unwrap();
    assert_eq!(m.offset_secs, -2670);
    assert_eq!(m.utc, ts("1971-06-01T12:44:30Z"));
    assert_eq!(m.warnings, vec![MomentWarning::PreTzdbReliability]);
}

#[test]
fn unknown_zone_is_error() {
    let r = resolve_moment(
        date(2000, 1, 1),
        Some(time(0, 0, 0, 0)),
        &iana("Mars/Olympus"),
        0.0,
        None,
    );
    assert_eq!(
        r,
        Err(MomentError::UnknownTimeZone("Mars/Olympus".to_owned()))
    );
    assert!(!is_known_time_zone("Mars/Olympus"));
    assert!(is_known_time_zone("Europe/Istanbul"));
}

#[test]
fn unknown_time_with_iana_zone_keeps_both_warnings() {
    let m = resolve_moment(date(1960, 1, 1), None, &iana("Europe/Istanbul"), 29.0, None).unwrap();
    assert_eq!(
        m.warnings,
        vec![
            MomentWarning::TimeUnknownNoon,
            MomentWarning::PreTzdbReliability
        ]
    );
    assert_eq!(m.utc, ts("1960-01-01T10:00:00Z"));
}
