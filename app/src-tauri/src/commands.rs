//! İnce komut katmanı: girdiyi motora iletir, sonucu DTO'ya çevirir.

use std::collections::BTreeMap;
use std::sync::Arc;

use ephemeris::{Body, Engine, EphError, Ephemeris, SiderealMode, SwissEph, Zodiac};
use serde::Serialize;
use specta::Type;
use tauri::State;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub se_version: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PlanetRow {
    pub body: String,
    pub lon: f64,
    pub retrograde: bool,
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

const GRAHAS: [(Body, &str); 8] = [
    (Body::Sun, "Sun"),
    (Body::Moon, "Moon"),
    (Body::Mars, "Mars"),
    (Body::Mercury, "Mercury"),
    (Body::Jupiter, "Jupiter"),
    (Body::Venus, "Venus"),
    (Body::Saturn, "Saturn"),
    (Body::MeanNode, "Rahu"),
];

/// Dokuz grahanın Lahiri sidereal boylamları. Ketu = Rahu + 180°.
/// Spike amaçlıdır; gerçek hesap Plan 04'te `jyotish-core`'a taşınır.
pub fn planet_rows(eph: &mut dyn Ephemeris, jd_ut: f64) -> Result<Vec<PlanetRow>, EphError> {
    let zodiac = Zodiac::Sidereal(SiderealMode::Lahiri);
    let mut rows = Vec::with_capacity(9);
    for (body, name) in GRAHAS {
        let p = eph.body(jd_ut, body, zodiac)?;
        rows.push(PlanetRow {
            body: name.to_owned(),
            lon: p.lon,
            retrograde: p.is_retrograde(),
        });
    }
    let rahu = rows
        .iter()
        .zip(GRAHAS)
        .find(|(_, (body, _))| *body == Body::MeanNode)
        .map(|(row, _)| row.clone())
        .ok_or_else(|| EphError::Calc("rahu missing".into()))?;
    rows.push(PlanetRow {
        body: "Ketu".to_owned(),
        lon: (rahu.lon + 180.0).rem_euclid(360.0),
        retrograde: rahu.retrograde,
    });
    Ok(rows)
}

#[tauri::command]
#[specta::specta]
pub async fn engine_status() -> Result<EngineStatus, CommandError> {
    Ok(EngineStatus {
        se_version: SwissEph::version(),
    })
}

/// Motor çağrısı milisaniyeler sürer ve `engine.call` kanal üzerinde bloklar. Bu yüzden
/// çağrı `spawn_blocking` havuzunda yapılır; async çalışma zamanı iş parçacıkları serbest kalır.
/// `State<'_, _>` `'static` olmadığından motor `Arc` olarak yönetilir ve kapanışa klonlanır.
#[tauri::command]
#[specta::specta]
pub async fn planet_longitudes(
    engine: State<'_, Arc<Engine>>,
    jd_ut: f64,
) -> Result<Vec<PlanetRow>, CommandError> {
    let engine = Arc::clone(&engine);
    let rows = tauri::async_runtime::spawn_blocking(move || {
        engine.call(move |se: &mut SwissEph| planet_rows(se, jd_ut))
    })
    .await
    .map_err(|e| CommandError::new("engine_stopped", &[("detail", e.to_string())]))???;
    Ok(rows)
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
    use ephemeris::{Ayanamsa, BodyPosition, HouseSystem, Houses, Precision};

    /// Her cisme sabit boylam veren sahte motor; Rahu 100°.
    struct Fake;
    impl Ephemeris for Fake {
        fn body(&mut self, _jd: f64, body: Body, _z: Zodiac) -> Result<BodyPosition, EphError> {
            let lon = match body {
                Body::MeanNode => 100.0,
                Body::Saturn => 300.0,
                _ => 10.0,
            };
            Ok(BodyPosition {
                lon,
                lat: 0.0,
                distance_au: 1.0,
                // Mean Node gerçekte hep geri gider (≈ -0.053°/gün).
                lon_speed: match body {
                    Body::Saturn => -0.05,
                    Body::MeanNode => -0.053,
                    _ => 1.0,
                },
                lat_speed: 0.0,
                dist_speed: 0.0,
                precision: Precision::SwissEph,
            })
        }
        fn ayanamsa(&mut self, _jd: f64, _m: SiderealMode) -> Result<Ayanamsa, EphError> {
            Ok(Ayanamsa {
                deg: 24.0,
                precision: Precision::SwissEph,
            })
        }
        fn houses(
            &mut self,
            _jd: f64,
            _lat: f64,
            _lon: f64,
            _s: HouseSystem,
            _z: Zodiac,
        ) -> Result<Houses, EphError> {
            Err(EphError::Calc("fake: houses unused".into()))
        }
        fn swe_version(&self) -> String {
            "fake".into()
        }
    }

    #[test]
    fn rows_are_nine_grahas_in_order() {
        let rows = planet_rows(&mut Fake, 2451545.0).unwrap();
        let names: Vec<_> = rows.iter().map(|r| r.body.as_str()).collect();
        assert_eq!(
            names,
            ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"]
        );
    }

    #[test]
    fn ketu_is_opposite_rahu_and_both_retrograde() {
        let rows = planet_rows(&mut Fake, 2451545.0).unwrap();
        let rahu = rows.iter().find(|r| r.body == "Rahu").unwrap();
        let ketu = rows.iter().find(|r| r.body == "Ketu").unwrap();
        assert_eq!(rahu.lon, 100.0);
        assert_eq!(ketu.lon, 280.0);
        assert!(rahu.retrograde && ketu.retrograde);
    }

    #[test]
    fn retrograde_follows_speed_sign() {
        let rows = planet_rows(&mut Fake, 2451545.0).unwrap();
        assert!(rows.iter().find(|r| r.body == "Saturn").unwrap().retrograde);
        assert!(!rows.iter().find(|r| r.body == "Sun").unwrap().retrograde);
    }

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
    fn real_engine_produces_sidereal_rows() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/ephe");
        let engine = ephemeris::Engine::spawn(dir).expect("spawn");
        let rows = engine
            .call(|se| planet_rows(se, 2451545.0))
            .expect("alive")
            .expect("calc");
        let sun = rows.iter().find(|r| r.body == "Sun").unwrap();
        // J2000 tropikal Güneş ≈ 280.37°, Lahiri ≈ 23.86° → sidereal ≈ 256.5°.
        assert!((256.0..257.0).contains(&sun.lon), "sun {}", sun.lon);
    }
}
