//! İnce komut katmanı: girdiyi motora iletir, sonucu DTO'ya çevirir.

use std::collections::BTreeMap;
use std::sync::Arc;

use ephemeris::{Engine, EphError, SwissEph};
use jyotish_core::{
    Bhavas, CalcSettings, Chart, CoreError, DashaPeriod, DashaSeed, Graha, TransitSnapshot,
};
use serde::Serialize;
use specta::Type;
use tauri::State;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub se_version: String,
}

/// Arayüze giden hata: `code` sabit bir anahtar, çeviri i18n katmanında yapılır.
#[derive(Debug, Clone, Serialize, Type)]
pub struct CommandError {
    pub code: String,
    pub params: BTreeMap<String, String>,
}

impl CommandError {
    pub fn new(code: &str, params: &[(&str, String)]) -> Self {
        Self {
            code: code.to_owned(),
            params: params
                .iter()
                .map(|(k, v)| ((*k).to_owned(), v.clone()))
                .collect(),
        }
    }
}

impl From<EphError> for CommandError {
    fn from(e: EphError) -> Self {
        match e {
            EphError::InvalidPath => Self::new("invalid_path", &[]),
            EphError::EnvOverride => Self::new("env_override", &[]),
            EphError::DataFilesMissing { path, detail } => {
                Self::new("data_files_missing", &[("path", path), ("detail", detail)])
            }
            EphError::Calc(detail) => Self::new("calc", &[("detail", detail)]),
            EphError::AlreadyOpen => Self::new("already_open", &[]),
            EphError::EngineStopped => Self::new("engine_stopped", &[]),
            EphError::JobPanicked => Self::new("job_panicked", &[]),
        }
    }
}

impl From<CoreError> for CommandError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::Ephemeris(e) => e.into(),
            CoreError::UnsupportedSchemaVersion { found, supported } => Self::new(
                "unsupported_schema_version",
                &[
                    ("found", found.to_string()),
                    ("supported", supported.to_string()),
                ],
            ),
            CoreError::InvalidSettings(field) => {
                Self::new("invalid_settings", &[("field", field.to_owned())])
            }
            CoreError::InvalidLocation { lat, lon } => Self::new(
                "invalid_location",
                &[("lat", lat.to_string()), ("lon", lon.to_string())],
            ),
            CoreError::InvalidJulianDay(jd) => {
                Self::new("invalid_julian_day", &[("jd", jd.to_string())])
            }
            CoreError::NonFiniteResult(what) => {
                Self::new("non_finite_result", &[("what", what.to_owned())])
            }
            CoreError::InvalidDashaInput(field) => {
                Self::new("invalid_dasha_input", &[("field", field.to_owned())])
            }
            CoreError::PanchangSearch(what) => {
                Self::new("panchang_search", &[("what", what.to_owned())])
            }
            CoreError::InvalidTransitInput(field) => {
                Self::new("invalid_transit_input", &[("field", field.to_owned())])
            }
            CoreError::InvalidMuhurtaInput(field) => {
                Self::new("invalid_muhurta_input", &[("field", field.to_owned())])
            }
            CoreError::InvalidPrashnaInput(field) => {
                Self::new("invalid_prashna_input", &[("field", field.to_owned())])
            }
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn engine_status() -> Result<EngineStatus, CommandError> {
    Ok(EngineStatus {
        se_version: SwissEph::version(),
    })
}

/// `compute_chart` komutunun gövdesi: hesap motor iş parçacığında yapılır, burada iş
/// mantığı yoktur. Bloklar; async bağlamdan `spawn_blocking` içinde çağrılmalıdır.
pub fn compute_chart_blocking(
    engine: &Engine,
    jd_ut: f64,
    lat: f64,
    lon: f64,
    settings: CalcSettings,
) -> Result<Chart, CommandError> {
    let chart = engine.call(move |se: &mut SwissEph| {
        jyotish_core::compute_chart(se, jd_ut, lat, lon, &settings)
    })??;
    Ok(chart)
}

/// Haritayı hesaplar. `jd_ut` ve konum atlas katmanında çözülmüş olarak gelir.
#[tauri::command]
#[specta::specta]
pub async fn compute_chart(
    engine: State<'_, Arc<Engine>>,
    jd_ut: f64,
    lat: f64,
    lon: f64,
    settings: CalcSettings,
) -> Result<Chart, CommandError> {
    let engine = Arc::clone(&engine);
    tauri::async_runtime::spawn_blocking(move || {
        compute_chart_blocking(&engine, jd_ut, lat, lon, settings)
    })
    .await
    .map_err(|e| CommandError::new("engine_stopped", &[("detail", e.to_string())]))?
}

/// `compute_transits` komutunun gövdesi; `compute_chart_blocking` gibi bloklar.
pub fn compute_transits_blocking(
    engine: &Engine,
    jd_ut: f64,
    settings: CalcSettings,
    natal_bhavas: Bhavas,
) -> Result<TransitSnapshot, CommandError> {
    let snapshot = engine.call(move |se: &mut SwissEph| {
        jyotish_core::compute_transits(se, jd_ut, &settings, &natal_bhavas)
    })??;
    Ok(snapshot)
}

/// Transit konumları (UI spec §3.3). `jd_ut` arayüzden gelir (varsayılan "şimdi" ya da
/// seçilen dasha dönemi); `settings` ve `natal_bhavas` natal haritanın
/// `chart.meta.settings` ve `chart.bhavas` alanlarıdır.
#[tauri::command]
#[specta::specta]
pub async fn compute_transits(
    engine: State<'_, Arc<Engine>>,
    jd_ut: f64,
    settings: CalcSettings,
    natal_bhavas: Bhavas,
) -> Result<TransitSnapshot, CommandError> {
    let engine = Arc::clone(&engine);
    tauri::async_runtime::spawn_blocking(move || {
        compute_transits_blocking(&engine, jd_ut, settings, natal_bhavas)
    })
    .await
    .map_err(|e| CommandError::new("engine_stopped", &[("detail", e.to_string())]))?
}

/// Bir dasha döneminin alt dönemleri (spec §4 `dasha_children(path)`). `seed`
/// `chart.dasha.seed`'den, `path` dönemin `lords`'undan gelir. Saf hesaptır, motora gitmez.
#[tauri::command]
#[specta::specta]
pub fn dasha_children(seed: DashaSeed, path: Vec<Graha>) -> Result<Vec<DashaPeriod>, CommandError> {
    Ok(jyotish_core::dasha_children(&seed, &path)?)
}

/// macOS'ta sistem print penceresini açar; kullanıcı oradan "PDF olarak kaydet"i seçer.
/// Senkron komut: print işlemi ana iş parçacığında çalışmalıdır.
#[tauri::command]
#[specta::specta]
pub fn print_window(window: tauri::WebviewWindow) -> Result<(), CommandError> {
    window
        .print()
        .map_err(|e| CommandError::new("print_failed", &[("detail", e.to_string())]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_are_stable() {
        assert_eq!(
            CommandError::from(EphError::AlreadyOpen).code,
            "already_open"
        );
        let e = CommandError::from(EphError::DataFilesMissing {
            path: "/p".into(),
            detail: "d".into(),
        });
        assert_eq!(e.code, "data_files_missing");
        assert_eq!(e.params.get("path").map(String::as_str), Some("/p"));
        assert_eq!(
            CommandError::from(EphError::Calc("x".into())).params["detail"],
            "x"
        );
    }

    #[test]
    fn core_error_codes_are_stable() {
        use jyotish_core::CoreError;
        let e = CommandError::from(CoreError::InvalidLocation {
            lat: 91.0,
            lon: 0.0,
        });
        assert_eq!(e.code, "invalid_location");
        assert_eq!(e.params["lon"], "0");
        let e = CommandError::from(CoreError::UnsupportedSchemaVersion {
            found: 2,
            supported: 1,
        });
        assert_eq!(e.code, "unsupported_schema_version");
        assert_eq!(e.params["supported"], "1");
        let e = CommandError::from(CoreError::InvalidSettings("dasha_year"));
        assert_eq!(e.code, "invalid_settings");
        assert_eq!(e.params["field"], "dasha_year");
        let e = CommandError::from(CoreError::InvalidDashaInput("path"));
        assert_eq!(e.code, "invalid_dasha_input");
        assert_eq!(e.params["field"], "path");
        let e = CommandError::from(CoreError::NonFiniteResult("ayanamsa"));
        assert_eq!(e.code, "non_finite_result");
        assert_eq!(e.params["what"], "ayanamsa");
        assert_eq!(
            CommandError::from(CoreError::InvalidJulianDay(f64::NAN)).code,
            "invalid_julian_day"
        );
        assert_eq!(
            CommandError::from(CoreError::Ephemeris(EphError::EngineStopped)).code,
            "engine_stopped"
        );
        let e = CommandError::from(CoreError::PanchangSearch("tithi"));
        assert_eq!(e.code, "panchang_search");
        assert_eq!(e.params["what"], "tithi");
        let e = CommandError::from(CoreError::InvalidTransitInput("natal_bhavas"));
        assert_eq!(e.code, "invalid_transit_input");
        assert_eq!(e.params["field"], "natal_bhavas");
        let e = CommandError::from(CoreError::InvalidMuhurtaInput("place"));
        assert_eq!(e.code, "invalid_muhurta_input");
        // Yalnızca alan adı: yer ve tarih hata parametresine girmez.
        assert_eq!(e.params.len(), 1);
        assert_eq!(e.params["field"], "place");
        let e = CommandError::from(CoreError::InvalidPrashnaInput("number"));
        assert_eq!(e.code, "invalid_prashna_input");
        // Yalnızca alan adı: sayı, soru, yer ve an hata parametresine girmez.
        assert_eq!(e.params.len(), 1);
        assert_eq!(e.params["field"], "number");
    }

    #[test]
    fn dasha_children_command_returns_sub_periods_and_maps_errors() {
        use jyotish_core::{DashaSeed, Graha};
        // tests/golden/new-delhi-2000-01-01.json Ay'ı: Rahu mahadashası.
        let seed = DashaSeed {
            birth_jd_ut: 2_451_545.0,
            moon_sidereal_lon: 199.470_529,
            year_days: 365.25,
        };
        let path = vec![Graha::Rahu, Graha::Rahu, Graha::Rahu];
        let kids = dasha_children(seed, path.clone()).expect("children");
        assert_eq!(kids.len(), 9);
        assert!(kids
            .iter()
            .all(|p| p.level == 4 && p.lords[..3] == path[..] && p.has_children));
        let err = dasha_children(seed, vec![Graha::Sun; 5]).unwrap_err();
        assert_eq!(err.code, "invalid_dasha_input");
        assert_eq!(err.params["field"], "path");
    }

    #[test]
    fn ipc_floats_round_trip_exactly() {
        // serde_json'un varsayılan ayrıştırıcısı bu değeri 1 ulp kaydırır (2459475.016772588).
        // `float_roundtrip` uygulama crate'inde açıktır; özellik birleşimiyle Tauri'nin komut
        // argümanı ayrıştırması da aynı yolu kullanır (Plan 06 incelemesi D1).
        let jd = 2_459_475.016_772_588_2_f64;
        let seed = DashaSeed {
            birth_jd_ut: jd,
            moon_sidereal_lon: 199.470_529,
            year_days: 365.25,
        };
        let json = serde_json::to_string(&seed).expect("serialize");
        assert!(json.contains("2459475.0167725882"), "{json}");
        let back: DashaSeed = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.birth_jd_ut.to_bits(), jd.to_bits());
        assert_eq!(back, seed);
    }
}
