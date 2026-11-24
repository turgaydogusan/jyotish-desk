//! Doğum zamanı çözümü (yerel saat → UTC, JD) ve çevrimdışı GeoNames atlası.

pub mod moment;

pub use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
pub use moment::{
    resolve_moment, Fold, MomentError, MomentWarning, ResolvedMoment, TzSpec, TZDB_VERSION,
};
