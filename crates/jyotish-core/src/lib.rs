//! Saf, yan etkisiz Jyotish hesabı. Ephemeris'e yalnızca `ephemeris::Ephemeris`
//! trait'i üzerinden erişir; I/O, Tauri ve atlas bağımlılığı yoktur.

mod error;
pub mod settings;

pub use error::CoreError;
pub use settings::{
    AyanamsaSystem, CalcSettings, CombustionOrbs, HouseSystem, NodeKind, SunriseDef, ZodiacMode,
    CALC_SETTINGS_SCHEMA_VERSION,
};
