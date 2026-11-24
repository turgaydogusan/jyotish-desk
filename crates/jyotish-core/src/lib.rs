//! Saf, yan etkisiz Jyotish hesabı. Ephemeris'e yalnızca `ephemeris::Ephemeris`
//! trait'i üzerinden erişir; I/O, Tauri ve atlas bağımlılığı yoktur.
//!
//! Bu sürüm M2 kapsamıdır: ayanamsa, grahalar, lagna, bhavalar, nakshatra, dignity, yanma.
//! Vargalar, dasha, panchang ve ashtakavarga sonraki planlarda `Chart`'a eklenir.

mod chart;
mod combustion;
pub mod dignity;
mod error;
#[cfg(test)]
mod fake;
mod graha;
mod houses;
#[cfg(test)]
mod proptests;
pub mod settings;
pub mod varga;
pub mod zodiac;

pub use chart::{compute_chart, Chart, ChartMeta, ChartWarning, ENGINE_VERSION};
pub use combustion::is_combust;
pub use dignity::{dignity, Dignity, Relation};
pub use error::CoreError;
pub use graha::{Graha, GrahaPosition};
pub use houses::{house_from_cusps, whole_sign_cusps, whole_sign_house, Bhavas, Lagna};
pub use settings::{
    AyanamsaSystem, CalcSettings, CombustionOrbs, HouseSystem, NodeKind, SunriseDef, ZodiacMode,
    CALC_SETTINGS_SCHEMA_VERSION,
};
pub use varga::{varga_position, Varga};
pub use zodiac::{angular_distance, deg_in_sign, nakshatra_pada, normalize, Sign};
