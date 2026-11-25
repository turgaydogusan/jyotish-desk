//! Doğum verisi denetimi, kanonik biçim ve atlas ile an çözümü (spec §7: DST boşluğu ve
//! örtüşmesi hatadır, sessizce kaydırılmaz).

use atlas::{Date, Fold, Time, TzSpec};
use jyotish_core::JD_UT_RANGE;

use crate::model::{BirthData, FoldChoice, MomentWarning, ResolvedRecord, TimeCertainty, TzChoice};
use crate::StoreError;

/// Denetlenmiş doğum verisi: kanonik `BirthData` ve atlas girdileri.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedBirth {
    /// Tarih `YYYY-MM-DD`, saat jiff'in kanonik biçimi (`10:30:00`).
    pub canonical: BirthData,
    pub date: Date,
    pub time: Option<Time>,
    pub tz: TzSpec,
    pub fold: Option<Fold>,
}

/// Alanları denetler; hata `InvalidInput(alan)` olarak döner. Saat bilinmiyorsa
/// (`TimeCertainty::Unknown`) `time` boş olmalıdır, ve tersi.
pub fn parse_birth(b: &BirthData) -> Result<ParsedBirth, StoreError> {
    let date: Date = b
        .date
        .trim()
        .parse()
        .map_err(|_| StoreError::InvalidInput("birth.date"))?;
    let time: Option<Time> = match &b.time {
        Some(t) => Some(
            t.trim()
                .parse()
                .map_err(|_| StoreError::InvalidInput("birth.time"))?,
        ),
        None => None,
    };
    if time.is_none() != (b.time_certainty == TimeCertainty::Unknown) {
        return Err(StoreError::InvalidInput("birth.time_certainty"));
    }
    // NaN ve sonsuz aralık dışında kalır.
    if !(-90.0..=90.0).contains(&b.place.lat) {
        return Err(StoreError::InvalidInput("birth.place.lat"));
    }
    if !(-180.0..=180.0).contains(&b.place.lon) {
        return Err(StoreError::InvalidInput("birth.place.lon"));
    }
    let tz = match &b.tz {
        TzChoice::Iana(id) => TzSpec::Iana(id.trim().to_owned()),
        TzChoice::Fixed(secs) => TzSpec::Fixed(*secs),
        TzChoice::Lmt => TzSpec::Lmt,
    };
    let fold = b.fold.map(|f| match f {
        FoldChoice::Earlier => Fold::Earlier,
        FoldChoice::Later => Fold::Later,
    });
    let canonical = BirthData {
        date: date.to_string(),
        time: time.map(|t| t.to_string()),
        tz: match &tz {
            TzSpec::Iana(id) => TzChoice::Iana(id.clone()),
            TzSpec::Fixed(secs) => TzChoice::Fixed(*secs),
            TzSpec::Lmt => TzChoice::Lmt,
        },
        ..b.clone()
    };
    Ok(ParsedBirth {
        canonical,
        date,
        time,
        tz,
        fold,
    })
}

/// Atlas ile çözer. Hesap aralığı dışındaki an (`JD_UT_RANGE`) `InvalidInput("birth.date")`
/// olur; böylece kayıtlı her harita açılışta hesaplanabilir.
pub fn resolve_parsed(p: &ParsedBirth) -> Result<ResolvedRecord, StoreError> {
    let m = atlas::resolve_moment(p.date, p.time, &p.tz, p.canonical.place.lon, p.fold)?;
    if !JD_UT_RANGE.contains(&m.jd_ut) {
        return Err(StoreError::InvalidInput("birth.date"));
    }
    Ok(ResolvedRecord {
        utc: m.utc.to_string(),
        jd_ut: m.jd_ut,
        offset_secs: m.offset_secs,
        abbrev: m.abbrev,
        warnings: m
            .warnings
            .iter()
            .map(|w| match w {
                atlas::MomentWarning::TimeUnknownNoon => MomentWarning::TimeUnknownNoon,
                atlas::MomentWarning::LmtFromLongitude => MomentWarning::LmtFromLongitude,
                atlas::MomentWarning::PreTzdbReliability => MomentWarning::PreTzdbReliability,
            })
            .collect(),
        tzdb_version: atlas::tzdb_version().to_owned(),
    })
}

/// `parse_birth` + `resolve_parsed`: kanonik doğum verisi ve çözülmüş an.
pub fn resolve_birth(b: &BirthData) -> Result<(BirthData, ResolvedRecord), StoreError> {
    let p = parse_birth(b)?;
    let r = resolve_parsed(&p)?;
    Ok((p.canonical, r))
}

#[cfg(test)]
pub(crate) mod fixtures {
    use crate::model::{BirthData, Place, TimeCertainty, TzChoice};

    /// tests/golden/new-delhi-2000-01-01.json anı: 17:30 IST = 12:00 UT, JD 2451545.0.
    pub fn delhi() -> BirthData {
        BirthData {
            date: "2000-01-01".into(),
            time: Some("17:30".into()),
            time_certainty: TimeCertainty::Exact,
            place: Place {
                name: "New Delhi".into(),
                country: "India".into(),
                admin1: "NCT".into(),
                lat: 28.6139,
                lon: 77.2090,
                geoname_id: Some(1_261_481),
            },
            tz: TzChoice::Iana("Asia/Kolkata".into()),
            fold: None,
            notes: String::new(),
        }
    }

    pub fn istanbul(date: &str, time: &str) -> BirthData {
        BirthData {
            date: date.into(),
            time: Some(time.into()),
            time_certainty: TimeCertainty::Exact,
            place: Place {
                name: "İstanbul".into(),
                country: "Türkiye".into(),
                admin1: "İstanbul".into(),
                lat: 41.01384,
                lon: 28.94966,
                geoname_id: Some(745_044),
            },
            tz: TzChoice::Iana("Europe/Istanbul".into()),
            fold: None,
            notes: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::{delhi, istanbul};
    use super::*;
    use crate::model::FoldChoice;
    use atlas::MomentError;

    #[test]
    fn delhi_resolves_to_j2000_noon() {
        let (canon, r) = resolve_birth(&delhi()).unwrap();
        assert_eq!(canon.time.as_deref(), Some("17:30:00"));
        assert_eq!(canon.date, "2000-01-01");
        assert_eq!(r.utc, "2000-01-01T12:00:00Z");
        assert_eq!(r.jd_ut, 2_451_545.0);
        assert_eq!((r.offset_secs, r.abbrev.as_str()), (19_800, "IST"));
        assert!(r.warnings.is_empty());
        assert_eq!(r.tzdb_version, atlas::tzdb_version());
    }

    #[test]
    fn unknown_time_uses_noon_and_warns() {
        let b = BirthData {
            time: None,
            time_certainty: TimeCertainty::Unknown,
            ..delhi()
        };
        let (canon, r) = resolve_birth(&b).unwrap();
        assert_eq!(canon.time, None);
        assert_eq!(r.utc, "2000-01-01T06:30:00Z");
        assert_eq!(r.warnings, [MomentWarning::TimeUnknownNoon]);
    }

    #[test]
    fn invalid_fields_are_reported_by_name() {
        let cases: Vec<(BirthData, &str)> = vec![
            (
                BirthData {
                    date: "2000-02-30".into(),
                    ..delhi()
                },
                "birth.date",
            ),
            (
                BirthData {
                    time: Some("25:00".into()),
                    ..delhi()
                },
                "birth.time",
            ),
            (
                BirthData {
                    time_certainty: TimeCertainty::Unknown,
                    ..delhi()
                },
                "birth.time_certainty",
            ),
            (
                BirthData {
                    time: None,
                    ..delhi()
                },
                "birth.time_certainty",
            ),
        ];
        for (b, field) in cases {
            match resolve_birth(&b) {
                Err(StoreError::InvalidInput(f)) => assert_eq!(f, field),
                other => panic!("{field}: {other:?}"),
            }
        }
        let mut b = delhi();
        b.place.lat = f64::NAN;
        assert!(matches!(
            resolve_birth(&b),
            Err(StoreError::InvalidInput("birth.place.lat"))
        ));
        let mut b = delhi();
        b.place.lon = 181.0;
        assert!(matches!(
            resolve_birth(&b),
            Err(StoreError::InvalidInput("birth.place.lon"))
        ));
        // Hesap aralığı dışı: 9999 yılı (jiff kabul eder, SE aralığı dışında).
        let b = BirthData {
            date: "9999-01-01".into(),
            ..delhi()
        };
        assert!(matches!(
            resolve_birth(&b),
            Err(StoreError::InvalidInput("birth.date"))
        ));
    }

    #[test]
    fn dst_gap_and_overlap_are_errors_not_shifts() {
        // atlas tests/moment_iana.rs ile aynı anlar.
        match resolve_birth(&istanbul("2014-03-31", "03:30")) {
            Err(StoreError::Moment(MomentError::NonexistentLocalTime { .. })) => {}
            other => panic!("gap: {other:?}"),
        }
        match resolve_birth(&istanbul("2010-10-31", "03:30")) {
            Err(StoreError::Moment(MomentError::AmbiguousLocalTime { .. })) => {}
            other => panic!("overlap: {other:?}"),
        }
        let b = BirthData {
            fold: Some(FoldChoice::Later),
            ..istanbul("2010-10-31", "03:30")
        };
        assert_eq!(resolve_birth(&b).unwrap().1.utc, "2010-10-31T01:30:00Z");
    }

    #[test]
    fn fixed_offset_and_lmt_round_trip_through_canonical_form() {
        let b = BirthData {
            tz: TzChoice::Fixed(19_800),
            ..delhi()
        };
        let (canon, r) = resolve_birth(&b).unwrap();
        assert_eq!(canon.tz, TzChoice::Fixed(19_800));
        assert_eq!(r.jd_ut, 2_451_545.0);
        let b = BirthData {
            tz: TzChoice::Lmt,
            ..delhi()
        };
        let (_, r) = resolve_birth(&b).unwrap();
        // 77.2090° × 240 s = 18 530.16 s.
        assert_eq!(r.offset_secs, 18_530);
        assert_eq!(r.abbrev, "LMT");
    }
}
