//! Yerel doğum zamanı → UTC ve Jülyen günü (UT).
//!
//! Belirsizlik sessizce düzeltilmez: DST boşluğu ve örtüşmesi hata olarak döner,
//! karar kullanıcıya bırakılır (spec §7). tzdb binary'ye gömülüdür; sistem
//! zoneinfo'su kullanılmaz, sonuç her makinede aynıdır.
//!
//! IANA bölgelerinde LMT kuralı: tzdb'nin seçilen ofsetinin kısaltması "LMT" ise ve
//! yaz saati değilse, bu değer bölgenin referans şehrinin boylamıdır; yerine doğum
//! yerinin boylamından LMT hesaplanır (`LmtFromLongitude`). Diğer tüm ofsetler —
//! adlandırılmış ortalama saatler (IMT, PMT, DMT, MMT, NST…) ve dakikaya tam
//! bölünmeyen yaz saatleri (Dublin 1916 IST, St. John's NDT) dahil — tzdb'deki ofset
//! ve kısaltmayla aynen kullanılır. Ofset dakikaya tam bölünmüyorsa (adlandırılmış
//! ortalama saat) ya da yerel yıl 1970'ten önceyse `PreTzdbReliability` eklenir.
//! Doğum yerinin LMT'sini isteyen kullanıcı `TzSpec::Lmt` seçer.

use std::sync::OnceLock;

use jiff::civil::{Date, DateTime, Time};
use jiff::tz::{AmbiguousOffset, Offset, TimeZoneDatabase};
use jiff::{SignedDuration, Timestamp};

const MAX_FIXED_OFFSET_SECS: i32 = 18 * 3600;
const UNIX_EPOCH_JD: f64 = 2440587.5;

fn tzdb() -> &'static TimeZoneDatabase {
    static DB: OnceLock<TimeZoneDatabase> = OnceLock::new();
    DB.get_or_init(TimeZoneDatabase::bundled)
}

/// Gömülü IANA tzdb sürümü (ör. "2026f"); haritayla birlikte saklanır.
pub fn tzdb_version() -> &'static str {
    jiff_tzdb::VERSION
}

/// IANA kimliği gömülü tzdb'de var mı?
pub fn is_known_time_zone(id: &str) -> bool {
    tzdb().get(id).is_ok()
}

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
    /// tzdb referans şehrin LMT'sini veriyordu; doğum yerinin boylamından LMT hesaplandı.
    LmtFromLongitude,
    /// 1970 öncesi tzdb verisi garanti değil; manuel ofset önerilir.
    PreTzdbReliability,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedMoment {
    pub utc: Timestamp,
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
        local: DateTime,
        suggestion: DateTime,
    },
    #[error("local time {local} occurs twice (DST overlap): earlier {earlier_offset_secs}s, later {later_offset_secs}s")]
    AmbiguousLocalTime {
        local: DateTime,
        earlier_offset_secs: i32,
        later_offset_secs: i32,
    },
    #[error("UTC offset out of range: {0} s")]
    OffsetOutOfRange(i32),
    #[error("longitude out of range: {0}")]
    LongitudeOutOfRange(f64),
    #[error("date out of supported range")]
    DateOutOfRange,
}

pub fn resolve_moment(
    date: Date,
    time: Option<Time>,
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
            Time::constant(12, 0, 0, 0)
        }
    };
    let local = date.to_datetime(time);
    match tz {
        TzSpec::Fixed(secs) => {
            if secs.abs() > MAX_FIXED_OFFSET_SECS {
                return Err(MomentError::OffsetOutOfRange(*secs));
            }
            let offset =
                Offset::from_seconds(*secs).map_err(|_| MomentError::OffsetOutOfRange(*secs))?;
            let utc = offset
                .to_timestamp(local)
                .map_err(|_| MomentError::DateOutOfRange)?;
            Ok(moment(utc, *secs, fmt_utc_offset(*secs), warnings))
        }
        TzSpec::Lmt => lmt(local, lon_deg, warnings),
        TzSpec::Iana(id) => resolve_iana(local, id, lon_deg, fold, warnings),
    }
}

fn resolve_iana(
    local: DateTime,
    id: &str,
    lon_deg: f64,
    fold: Option<Fold>,
    mut warnings: Vec<MomentWarning>,
) -> Result<ResolvedMoment, MomentError> {
    let tz = tzdb()
        .get(id)
        .map_err(|_| MomentError::UnknownTimeZone(id.to_owned()))?;
    let offset = match tz.to_ambiguous_timestamp(local).offset() {
        AmbiguousOffset::Unambiguous { offset } => offset,
        AmbiguousOffset::Fold { before, after } => match fold {
            Some(Fold::Earlier) => before,
            Some(Fold::Later) => after,
            None => {
                return Err(MomentError::AmbiguousLocalTime {
                    local,
                    earlier_offset_secs: before.seconds(),
                    later_offset_secs: after.seconds(),
                })
            }
        },
        AmbiguousOffset::Gap { before, after } => {
            // Boşluk kadar ileri kaydır: 02:30 + (after − before) → 03:30.
            let shift = SignedDuration::from_secs(i64::from(after.seconds() - before.seconds()));
            return Err(MomentError::NonexistentLocalTime {
                local,
                suggestion: local.checked_add(shift).unwrap_or(local),
            });
        }
    };
    let offset_secs = offset.seconds();
    let utc = offset
        .to_timestamp(local)
        .map_err(|_| MomentError::DateOutOfRange)?;
    let info = tz.to_offset_info(utc);
    // Yalnızca tzdb'nin "LMT" kaydı referans şehrin boylamına bağlıdır; doğum yerininkiyle değiştirilir.
    if info.abbreviation() == "LMT" && !info.dst().is_dst() {
        warnings.push(MomentWarning::LmtFromLongitude);
        return lmt(local, lon_deg, warnings);
    }
    // Adlandırılmış ortalama saatler (IMT, MMT…) dakikaya tam bölünmez; tzdb ofseti korunur.
    if (offset_secs % 60 != 0 || local.year() < 1970)
        && !warnings.contains(&MomentWarning::PreTzdbReliability)
    {
        warnings.push(MomentWarning::PreTzdbReliability);
    }
    let abbrev = info.abbreviation().to_owned();
    Ok(moment(utc, offset_secs, abbrev, warnings))
}

/// Boylamdan yerel ortalama saat: 1° = 240 s. Saniye kesirleri korunur.
fn lmt(
    local: DateTime,
    lon_deg: f64,
    warnings: Vec<MomentWarning>,
) -> Result<ResolvedMoment, MomentError> {
    let offset_ns = (lon_deg * 240.0 * 1e9).round() as i64;
    let utc = Offset::UTC
        .to_timestamp(local)
        .and_then(|t| t.checked_sub(SignedDuration::from_nanos(offset_ns)))
        .map_err(|_| MomentError::DateOutOfRange)?;
    let offset_secs = (lon_deg * 240.0).round() as i32;
    Ok(moment(utc, offset_secs, "LMT".to_owned(), warnings))
}

/// `utc` anında `tz`'nin UTC ofseti, saniye (doğuda pozitif); gösterim içindir (bir anı o yerin
/// duvar saatiyle yazmak). `resolve_moment` ile aynı kurallar: sabit ofset aralık denetiminden
/// geçer; `Lmt` boylamdan hesaplanır; IANA bölgesinde tzdb'nin "LMT" kaydı yerin boylamıyla
/// değiştirilir. Bir an hiçbir zaman belirsiz değildir: boşluk ve örtüşme burada oluşmaz.
pub fn offset_at(tz: &TzSpec, lon_deg: f64, utc: Timestamp) -> Result<i32, MomentError> {
    if !lon_deg.is_finite() || !(-180.0..=180.0).contains(&lon_deg) {
        return Err(MomentError::LongitudeOutOfRange(lon_deg));
    }
    let lmt_secs = (lon_deg * 240.0).round() as i32;
    match tz {
        TzSpec::Fixed(secs) if secs.abs() > MAX_FIXED_OFFSET_SECS => {
            Err(MomentError::OffsetOutOfRange(*secs))
        }
        TzSpec::Fixed(secs) => Ok(*secs),
        TzSpec::Lmt => Ok(lmt_secs),
        TzSpec::Iana(id) => {
            let zone = tzdb()
                .get(id)
                .map_err(|_| MomentError::UnknownTimeZone(id.to_owned()))?;
            let info = zone.to_offset_info(utc);
            if info.abbreviation() == "LMT" && !info.dst().is_dst() {
                Ok(lmt_secs)
            } else {
                Ok(info.offset().seconds())
            }
        }
    }
}

fn moment(
    utc: Timestamp,
    offset_secs: i32,
    abbrev: String,
    warnings: Vec<MomentWarning>,
) -> ResolvedMoment {
    // as_second sıfıra doğru keser, subsec işaretlidir; toplam her iki yönde de doğrudur.
    let secs = utc.as_second() as f64 + f64::from(utc.subsec_nanosecond()) / 1e9;
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
