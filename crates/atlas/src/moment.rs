//! Yerel doğum zamanı → UTC ve Jülyen günü (UT).
//!
//! Belirsizlik sessizce düzeltilmez: DST boşluğu ve örtüşmesi hata olarak döner,
//! karar kullanıcıya bırakılır (spec §7).

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};

/// Paketlenen IANA tzdb sürümü (ör. "2025b"); haritayla birlikte saklanır.
pub const TZDB_VERSION: &str = chrono_tz::IANA_TZDB_VERSION;

const MAX_FIXED_OFFSET_SECS: i32 = 18 * 3600;
const UNIX_EPOCH_JD: f64 = 2440587.5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TzSpec {
    /// IANA kimliği, ör. "Europe/Istanbul".
    Iana(String),
    /// Manuel UTC ofseti, saniye (doğuda pozitif).
    Fixed(i32),
    /// Doğum yerinin boylamından yerel ortalama saat.
    Lmt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fold {
    /// İki kez yaşanan saatin ilki (genellikle yaz saati).
    Earlier,
    /// İkincisi.
    Later,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MomentWarning {
    /// Doğum saati bilinmiyor; yerel 12:00 kullanıldı.
    TimeUnknownNoon,
    /// tzdb referans şehrin ortalama saatini veriyordu; doğum yerinin boylamından LMT hesaplandı.
    LmtFromLongitude,
    /// 1970 öncesi tzdb verisi garanti değil; manuel ofset önerilir.
    PreTzdbReliability,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedMoment {
    pub utc: DateTime<Utc>,
    pub jd_ut: f64,
    /// Gösterim için saniyeye yuvarlanmış ofset; hesap `utc`/`jd_ut` ile yapılır.
    pub offset_secs: i32,
    pub abbrev: String,
    pub warnings: Vec<MomentWarning>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum MomentError {
    #[error("unknown IANA time zone: {0}")]
    UnknownTimeZone(String),
    #[error("local time {local} does not exist (DST gap); nearest valid: {suggestion}")]
    NonexistentLocalTime {
        local: NaiveDateTime,
        suggestion: NaiveDateTime,
    },
    #[error("local time {local} occurs twice (DST overlap): earlier {earlier_offset_secs}s, later {later_offset_secs}s")]
    AmbiguousLocalTime {
        local: NaiveDateTime,
        earlier_offset_secs: i32,
        later_offset_secs: i32,
    },
    #[error("UTC offset out of range: {0} s")]
    OffsetOutOfRange(i32),
    #[error("longitude out of range: {0}")]
    LongitudeOutOfRange(f64),
}

pub fn resolve_moment(
    date: NaiveDate,
    time: Option<NaiveTime>,
    tz: &TzSpec,
    lon_deg: f64,
    fold: Option<Fold>,
) -> Result<ResolvedMoment, MomentError> {
    if !lon_deg.is_finite() || !(-180.0..=180.0).contains(&lon_deg) {
        return Err(MomentError::LongitudeOutOfRange(lon_deg));
    }
    let mut warnings = Vec::new();
    let time = match time {
        Some(t) => t,
        None => {
            warnings.push(MomentWarning::TimeUnknownNoon);
            NaiveTime::from_hms_opt(12, 0, 0).expect("12:00:00 is a valid time")
        }
    };
    let local = date.and_time(time);
    match tz {
        TzSpec::Fixed(secs) => {
            if secs.abs() > MAX_FIXED_OFFSET_SECS {
                return Err(MomentError::OffsetOutOfRange(*secs));
            }
            let utc = local.and_utc() - Duration::seconds(i64::from(*secs));
            Ok(moment(utc, *secs, fmt_utc_offset(*secs), warnings))
        }
        TzSpec::Lmt => Ok(lmt(local, lon_deg, warnings)),
        TzSpec::Iana(id) => resolve_iana(local, id, lon_deg, fold, warnings),
    }
}

fn resolve_iana(
    _local: NaiveDateTime,
    id: &str,
    _lon_deg: f64,
    _fold: Option<Fold>,
    _warnings: Vec<MomentWarning>,
) -> Result<ResolvedMoment, MomentError> {
    // Task 2'de gerçek çözüm gelir.
    Err(MomentError::UnknownTimeZone(id.to_owned()))
}

/// Boylamdan yerel ortalama saat: 1° = 240 s. Saniye kesirleri korunur.
fn lmt(local: NaiveDateTime, lon_deg: f64, warnings: Vec<MomentWarning>) -> ResolvedMoment {
    let offset_ns = (lon_deg * 240.0 * 1e9).round() as i64;
    let utc = local.and_utc() - Duration::nanoseconds(offset_ns);
    let offset_secs = (lon_deg * 240.0).round() as i32;
    moment(utc, offset_secs, "LMT".to_owned(), warnings)
}

fn moment(
    utc: DateTime<Utc>,
    offset_secs: i32,
    abbrev: String,
    warnings: Vec<MomentWarning>,
) -> ResolvedMoment {
    let secs = utc.timestamp() as f64 + f64::from(utc.timestamp_subsec_nanos()) / 1e9;
    ResolvedMoment {
        utc,
        jd_ut: secs / 86_400.0 + UNIX_EPOCH_JD,
        offset_secs,
        abbrev,
        warnings,
    }
}

/// "UTC+05:30", saniye varsa "UTC+01:56:56".
fn fmt_utc_offset(secs: i32) -> String {
    let sign = if secs < 0 { '-' } else { '+' };
    let a = secs.unsigned_abs();
    let (h, m, s) = (a / 3600, (a % 3600) / 60, a % 60);
    if s == 0 {
        format!("UTC{sign}{h:02}:{m:02}")
    } else {
        format!("UTC{sign}{h:02}:{m:02}:{s:02}")
    }
}
