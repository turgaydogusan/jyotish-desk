//! Saf, yan etkisiz Jyotish hesabı. Ephemeris'e yalnızca `ephemeris::Ephemeris`
//! trait'i üzerinden erişir; I/O, Tauri ve atlas bağımlılığı yoktur.
//!
//! Kapsam: ayanamsa, grahalar, lagna, bhavalar, nakshatra, dignity, yanma (M2), 16 varga,
//! Vimshottari dasha, panchang, ashtakavarga ve transit (M3). İleri modüllerin ortak temeli
//! (Alt proje 3.1): `positions` (ortak girdi), `lordship`, `nature` ve `drishti`.

pub mod ashtakavarga;
mod chart;
mod combustion;
pub mod dasha;
pub mod dignity;
pub mod drishti;
mod error;
#[cfg(test)]
mod fake;
mod graha;
mod houses;
pub mod lordship;
pub mod nature;
pub mod panchang;
pub mod positions;
#[cfg(test)]
mod proptests;
pub mod settings;
pub mod transit;
pub mod varga;
pub mod zodiac;

pub use ashtakavarga::{
    compute_ashtakavarga, Ashtakavarga, ASHTAKAVARGA_GRAHAS, BAV_TOTALS, SAV_TOTAL,
};
pub use chart::{compute_chart, Chart, ChartMeta, ChartWarning, ENGINE_VERSION};
pub use combustion::is_combust;
pub use dasha::{
    birth_dasha, dasha_children, dasha_path_at, mahadashas, vimshottari, vimshottari_years, Dasha,
    DashaPeriod, DashaSeed, CHART_DASHA_LEVELS, MAX_DASHA_LEVEL, VIMSHOTTARI_ORDER,
    VIMSHOTTARI_TOTAL_YEARS,
};
pub use dignity::{dignity, Dignity, Relation};
pub use drishti::{
    compute_drishti, drishti_table, graha_aspect_houses, graha_aspect_signs, sphuta_drishti,
    DrishtiTable, SignAspect, ASPECTING_GRAHAS, DRISHTI_METHOD, FULL_DRISHTI,
};
pub use error::CoreError;
pub use graha::{Graha, GrahaPosition};
pub use houses::{house_from_cusps, whole_sign_cusps, whole_sign_house, Bhavas, Lagna};
pub use lordship::{house_lord, house_sign, houses_owned};
pub use nature::{kuja_dosha, moon_is_waxing, nature, KujaDosha, Nature, KUJA_DOSHA_HOUSES};
pub use panchang::{
    compute_panchang, elongation, karana_name, karana_no, local_weekday, nitya_yoga_no, paksha,
    tithi_no, Karana, KaranaName, NityaYoga, Paksha, Panchang, PanchangNakshatra, Tithi, Vara,
    END_BISECTIONS, END_SEARCH_DAYS, KARANA_SPAN_DEG, SUNRISE_WINDOW_DAYS, TITHI_SPAN_DEG,
};
pub use positions::{compute_positions, ChartInput, GrahaPoint, Positions};
pub use settings::{
    AdvancedSettings, AyanamsaSystem, CalcSettings, CombustionOrbs, HouseSystem, KarakaScheme,
    NodeKind, SunriseDef, ZodiacMode, CALC_SETTINGS_SCHEMA_VERSION, JD_UT_RANGE,
};
pub use transit::{compute_transits, TransitGraha, TransitSnapshot};
pub use varga::{
    all_vargas, is_vargottama, varga_chart, varga_position, Varga, VargaChart, VargaPlacement,
};
pub use zodiac::{angular_distance, deg_in_sign, nakshatra_pada, normalize, Sign};
