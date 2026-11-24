mod common;

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use ephemeris::{julday_gregorian, Body, Engine, EphError, Ephemeris, SwissEph, Zodiac};

// SE kilidi süreç çapında; bu dosyadaki testler paralel koşarsa birbirinin
// `try_open`'ını `AlreadyOpen` ile düşürür. Testleri sıraya sokar.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|p| p.into_inner())
}

fn j2000() -> f64 {
    julday_gregorian(2000, 1, 1, 12.0)
}

#[test]
fn engine_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Engine>();
}

#[test]
fn call_returns_sun_position() {
    let _s = serial();
    let engine = Engine::spawn(common::ephe_dir()).expect("spawn");
    let pos = engine
        .call(|se| se.body(j2000(), Body::Sun, Zodiac::Tropical))
        .expect("engine alive")
        .expect("calc ok");
    assert!((280.0..281.0).contains(&pos.lon), "sun lon {}", pos.lon);
}

#[test]
fn second_instance_while_engine_alive_is_already_open() {
    let _s = serial();
    let _engine = Engine::spawn(common::ephe_dir()).expect("spawn");
    assert!(matches!(
        Engine::spawn(common::ephe_dir()),
        Err(EphError::AlreadyOpen)
    ));
    assert!(matches!(
        SwissEph::try_open(&common::ephe_dir()),
        Err(EphError::AlreadyOpen)
    ));
}

#[test]
fn drop_releases_the_instance() {
    let _s = serial();
    let engine = Engine::spawn(common::ephe_dir()).expect("first spawn");
    drop(engine);
    let again = Engine::spawn(common::ephe_dir()).expect("respawn after drop");
    drop(again);
}

#[test]
fn missing_data_dir_fails_spawn() {
    let _s = serial();
    let bogus = PathBuf::from("/nonexistent-jyotish-desk-ephe");
    assert!(matches!(
        Engine::spawn(bogus),
        Err(EphError::DataFilesMissing { .. })
    ));
}

#[test]
fn panicking_job_reports_error_and_engine_survives() {
    let _s = serial();
    let engine = Engine::spawn(common::ephe_dir()).expect("spawn");
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let r: Result<(), EphError> = engine.call(|_| panic!("boom"));
    std::panic::set_hook(prev_hook);
    assert!(matches!(r, Err(EphError::JobPanicked)));
    let pos = engine
        .call(|se| se.body(j2000(), Body::Moon, Zodiac::Tropical))
        .expect("engine still alive")
        .expect("calc ok");
    assert!((0.0..360.0).contains(&pos.lon));
}

#[test]
fn concurrent_callers_are_serialized() {
    let _s = serial();
    let engine = Arc::new(Engine::spawn(common::ephe_dir()).expect("spawn"));
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let e = Arc::clone(&engine);
            std::thread::spawn(move || {
                for k in 0..10 {
                    let jd = j2000() + f64::from(i * 10 + k);
                    let pos = e
                        .call(move |se| se.body(jd, Body::Mars, Zodiac::Tropical))
                        .expect("alive")
                        .expect("calc");
                    assert!((0.0..360.0).contains(&pos.lon));
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("caller thread");
    }
}
