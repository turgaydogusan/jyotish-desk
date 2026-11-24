//! Saf, yan etkisiz Jyotish hesabı. Ephemeris'e yalnızca `ephemeris::Ephemeris`
//! trait'i üzerinden erişir; I/O, Tauri ve atlas bağımlılığı yoktur.

mod error;
#[cfg(test)]
mod proptests;
pub mod settings;
pub mod zodiac;

pub use error::CoreError;
pub use settings::{
    AyanamsaSystem, CalcSettings, CombustionOrbs, HouseSystem, NodeKind, SunriseDef, ZodiacMode,
    CALC_SETTINGS_SCHEMA_VERSION,
};
pub use zodiac::{angular_distance, deg_in_sign, nakshatra_pada, normalize, Sign};
