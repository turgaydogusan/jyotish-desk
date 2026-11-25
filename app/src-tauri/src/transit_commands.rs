//! Transit anı (UI spec §3.3 `transit_table`, harita panelinde transit halkası, dasa şeridi).
//! "Şimdi" Rust'ta okunur; seçilen yerel tarih-saat tarayıcının IANA diliminde çözülür
//! (boşlukta hata, örtüşmede önceki an; Plan 11 D7). Arayüz tarih hesabı yapmaz (C4).
//! Tarih ve ayarlar loglanmaz (C1).

use std::sync::Arc;

use atlas::Timestamp;
use ephemeris::Engine;
use jyotish_core::{dasha_path_at, Bhavas, CalcSettings, DashaSeed, Graha, TransitSnapshot};
use serde::{Deserialize, Serialize};
use specta::Type;
use store::{BirthData, FoldChoice, Place, TimeCertainty, TzChoice};
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::{compute_transits_blocking, CommandError};

/// Şeritte ve tabloda gösterilen dasa derinliği: maha, antar, pratyantar.
pub const TRANSIT_DASHA_DEPTH: u8 = 3;

/// Transit anı. `Local`: duvar saati (`YYYY-MM-DD`, `HH:MM`) ve IANA dilimi.
#[derive(Debug, Clone, PartialEq, Deserialize, Type)]
pub enum TransitAt {
    Now,
    Local {
        date: String,
        time: String,
        tz: String,
    },
    Jd(f64),
}

/// Transit konumları, anın UTC'si (RFC 3339) ve o andaki dasa yolu.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct TransitView {
    pub snapshot: TransitSnapshot,
    pub utc: String,
    pub dasha_path: Vec<Graha>,
}

/// Duvar saatini çözer. Koordinat ve doğum notu anı etkilemez; doğrulama ve hata eşlemesi
/// doğum verisininkiyle aynıdır (`birth.date`, `birth.time`, `nonexistent_local_time` …).
fn resolve_local(date: &str, time: &str, tz: TzChoice) -> Result<(f64, String), CommandError> {
    let birth = BirthData {
        date: date.to_owned(),
        time: Some(time.to_owned()),
        time_certainty: TimeCertainty::Exact,
        place: Place {
            name: String::new(),
            country: String::new(),
            admin1: String::new(),
            lat: 0.0,
            lon: 0.0,
            geoname_id: None,
        },
        tz,
        fold: Some(FoldChoice::Earlier),
        notes: String::new(),
    };
    let (_, r) = store::resolve_birth(&birth)?;
    Ok((r.jd_ut, r.utc))
}

fn transit_moment(at: &TransitAt, now: Timestamp) -> Result<(f64, String), CommandError> {
    match at {
        TransitAt::Now => {
            // UTC duvar saati, saniye çözünürlüklü; JD dönüşümü atlas'ta.
            let date = now.strftime("%Y-%m-%d").to_string();
            let time = now.strftime("%H:%M:%S").to_string();
            resolve_local(&date, &time, TzChoice::Fixed(0))
        }
        TransitAt::Local { date, time, tz } => {
            resolve_local(date, time, TzChoice::Iana(tz.clone()))
        }
        TransitAt::Jd(jd) => {
            let secs = (jd - 2_440_587.5) * 86_400.0;
            let utc = if secs.is_finite() && secs.abs() < 1e13 {
                Timestamp::from_second(secs.round() as i64)
                    .map(|t| t.to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            };
            Ok((*jd, utc))
        }
    }
}

/// `transits_at` gövdesi; `now` testte sabitlenir. Bloklar: `spawn_blocking` içinde çağrılır.
pub fn transits_at_blocking(
    engine: &Engine,
    at: TransitAt,
    now: Timestamp,
    settings: CalcSettings,
    natal_bhavas: Bhavas,
    seed: DashaSeed,
) -> Result<TransitView, CommandError> {
    let (jd_ut, utc) = transit_moment(&at, now)?;
    let snapshot = compute_transits_blocking(engine, jd_ut, settings, natal_bhavas)?;
    let dasha_path = dasha_path_at(&seed, jd_ut, TRANSIT_DASHA_DEPTH)?;
    Ok(TransitView {
        snapshot,
        utc,
        dasha_path,
    })
}

/// Açık haritanın transitleri. `settings`, `natal_bhavas` ve `seed` açık haritadan gelir.
#[tauri::command]
#[specta::specta]
pub async fn transits_at(
    engine: State<'_, Arc<Engine>>,
    at: TransitAt,
    settings: CalcSettings,
    natal_bhavas: Bhavas,
    seed: DashaSeed,
) -> Result<TransitView, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || {
        transits_at_blocking(&engine, at, Timestamp::now(), settings, natal_bhavas, seed)
    })
    .await
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// Süreçte yalnızca bir SE örneği açılabilir (`EphError::AlreadyOpen`) ve testler paralel
    /// koşar: bu modülün testleri tek motoru paylaşır (Plan 11 D20). Başka bir lib testi motor
    /// açmaz; açması gerekirse bu işlevi kullanır.
    fn engine() -> &'static Engine {
        static ENGINE: std::sync::OnceLock<Engine> = std::sync::OnceLock::new();
        ENGINE.get_or_init(|| {
            let ephe = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/ephe");
            Engine::spawn(ephe).expect("spawn engine")
        })
    }

    /// tests/golden/new-delhi-2000-01-01.json: JD 2451545.0, Ay 199.470529°.
    fn seed() -> DashaSeed {
        DashaSeed {
            birth_jd_ut: 2_451_545.0,
            moon_sidereal_lon: 199.470_529,
            year_days: 365.25,
        }
    }

    fn natal(engine: &Engine) -> Bhavas {
        crate::commands::compute_chart_blocking(
            engine,
            2_451_545.0,
            28.6139,
            77.2090,
            CalcSettings::default(),
        )
        .expect("chart")
        .bhavas
    }

    #[test]
    fn now_jd_local_and_jd_resolve_to_the_same_moment() {
        let engine = engine();
        let bhavas = natal(engine);
        // 2026-11-24T00:00:00Z.
        let now = Timestamp::from_second(1_795_478_400).unwrap();
        let a = transits_at_blocking(
            engine,
            TransitAt::Now,
            now,
            CalcSettings::default(),
            bhavas.clone(),
            seed(),
        )
        .unwrap();
        assert_eq!(a.utc, "2026-11-24T00:00:00Z");
        let b = transits_at_blocking(
            engine,
            TransitAt::Local {
                date: "2026-11-24".into(),
                time: "03:00".into(),
                tz: "Europe/Istanbul".into(),
            },
            now,
            CalcSettings::default(),
            bhavas.clone(),
            seed(),
        )
        .unwrap();
        assert_eq!(b.utc, a.utc);
        assert_eq!(b.snapshot.jd_ut, a.snapshot.jd_ut);
        let c = transits_at_blocking(
            engine,
            TransitAt::Jd(a.snapshot.jd_ut),
            now,
            CalcSettings::default(),
            bhavas,
            seed(),
        )
        .unwrap();
        assert_eq!(c.snapshot, a.snapshot);
        assert_eq!(a.dasha_path.len(), usize::from(TRANSIT_DASHA_DEPTH));
        assert_eq!(a.dasha_path, c.dasha_path);
    }

    #[test]
    fn local_moments_follow_the_dst_rules() {
        let engine = engine();
        let bhavas = natal(engine);
        let now = Timestamp::from_second(1_795_478_400).unwrap();
        let local = |date: &str, time: &str| TransitAt::Local {
            date: date.into(),
            time: time.into(),
            tz: "Europe/Istanbul".into(),
        };
        // Boşluk: hata ve öneri, sessiz kaydırma yok.
        let gap = transits_at_blocking(
            engine,
            local("2014-03-31", "03:30"),
            now,
            CalcSettings::default(),
            bhavas.clone(),
            seed(),
        )
        .unwrap_err();
        assert_eq!(gap.code, "nonexistent_local_time");
        assert_eq!(gap.params["suggestion"], "2014-03-31T04:30:00");
        // Örtüşme: önceki an (yaz saati, +02:00).
        let berlin = TransitAt::Local {
            date: "2014-10-26".into(),
            time: "02:30".into(),
            tz: "Europe/Berlin".into(),
        };
        let overlap = transits_at_blocking(
            engine,
            berlin,
            now,
            CalcSettings::default(),
            bhavas.clone(),
            seed(),
        )
        .unwrap();
        assert_eq!(overlap.utc, "2014-10-26T00:30:00Z");
        // Bilinmeyen dilim ve bozuk tarih.
        let bad_tz = TransitAt::Local {
            date: "2026-11-24".into(),
            time: "03:00".into(),
            tz: "Mars/Olympus".into(),
        };
        assert_eq!(
            transits_at_blocking(
                engine,
                bad_tz,
                now,
                CalcSettings::default(),
                bhavas.clone(),
                seed()
            )
            .unwrap_err()
            .code,
            "unknown_time_zone"
        );
        assert_eq!(
            transits_at_blocking(
                engine,
                local("2026-13-01", "03:00"),
                now,
                CalcSettings::default(),
                bhavas,
                seed()
            )
            .unwrap_err()
            .params["field"],
            "birth.date"
        );
    }

    #[test]
    fn bad_jd_is_an_error_not_a_panic() {
        let engine = engine();
        let bhavas = natal(engine);
        let now = Timestamp::from_second(1_795_478_400).unwrap();
        let e = transits_at_blocking(
            engine,
            TransitAt::Jd(f64::INFINITY),
            now,
            CalcSettings::default(),
            bhavas,
            seed(),
        )
        .unwrap_err();
        assert!(
            ["invalid_julian_day", "invalid_transit_input"].contains(&e.code.as_str()),
            "{}",
            e.code
        );
    }
}
