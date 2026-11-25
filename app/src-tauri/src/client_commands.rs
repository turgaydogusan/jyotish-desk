//! Danışan veritabanı komutları (spec §4 "Danışan açma", M4). İnce katman: kayıt işleri
//! `store`'da, harita hesabı `jyotish-core`'da; burada yalnızca birleştirme ve hata eşleme.

use std::sync::{Arc, Mutex};

use atlas::MomentError;
use ephemeris::Engine;
use jyotish_core::{CalcSettings, Chart};
use serde::Serialize;
use specta::Type;
use store::{
    ChartRecord, Client, NewChart, NewClient, Note, Store, StoreError, StoreWarning, ViewState,
};
use tauri::State;

use crate::commands::{compute_chart_blocking, CommandError};

/// Tauri durumunda tek store örneği.
pub type StoreState = Arc<Mutex<Store>>;

/// Veritabanı dosyasının adı (uygulama veri dizininde).
pub const DB_FILE_NAME: &str = "jyotish-desk.sqlite";

/// `open_client` dönüşü (spec §4: `{client, birth, calc_settings, view_state, chart,
/// recent_notes}`; doğum verisi `chart_record.birth`'tedir).
#[derive(Debug, Clone, Serialize, Type)]
pub struct OpenedClient {
    pub client: Client,
    pub chart_record: ChartRecord,
    pub calc_settings: CalcSettings,
    pub view_state: ViewState,
    pub chart: Chart,
    pub recent_notes: Vec<Note>,
    pub warnings: Vec<StoreWarning>,
}

impl From<StoreError> for CommandError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::Sqlite(e) => Self::new("store_sqlite", &[("detail", e.to_string())]),
            StoreError::Migration(e) => Self::new("store_migration", &[("detail", e.to_string())]),
            StoreError::Json(e) => Self::new("store_json", &[("detail", e.to_string())]),
            StoreError::DatabaseTooNew { found, supported } => Self::new(
                "store_too_new",
                &[
                    ("found", found.to_string()),
                    ("supported", supported.to_string()),
                ],
            ),
            StoreError::NotFound { what, id } => Self::new(
                "not_found",
                &[("what", what.to_owned()), ("id", id.to_string())],
            ),
            StoreError::InvalidInput(field) => {
                Self::new("invalid_input", &[("field", field.to_owned())])
            }
            StoreError::Moment(m) => match m {
                MomentError::UnknownTimeZone(tz) => Self::new("unknown_time_zone", &[("tz", tz)]),
                MomentError::NonexistentLocalTime { local, suggestion } => Self::new(
                    "nonexistent_local_time",
                    &[
                        ("local", local.to_string()),
                        ("suggestion", suggestion.to_string()),
                    ],
                ),
                MomentError::AmbiguousLocalTime {
                    local,
                    earlier_offset_secs,
                    later_offset_secs,
                } => Self::new(
                    "ambiguous_local_time",
                    &[
                        ("local", local.to_string()),
                        ("earlier_offset_secs", earlier_offset_secs.to_string()),
                        ("later_offset_secs", later_offset_secs.to_string()),
                    ],
                ),
                MomentError::OffsetOutOfRange(secs) => {
                    Self::new("offset_out_of_range", &[("secs", secs.to_string())])
                }
                MomentError::LongitudeOutOfRange(lon) => {
                    Self::new("longitude_out_of_range", &[("lon", lon.to_string())])
                }
                MomentError::DateOutOfRange => Self::new("date_out_of_range", &[]),
            },
            StoreError::Settings(e) => e.into(),
            StoreError::LastChart => Self::new("last_chart", &[]),
            StoreError::ImportFormat(detail) => Self::new("import_format", &[("detail", detail)]),
            StoreError::ImportVersion { found, supported } => Self::new(
                "import_version",
                &[
                    ("found", found.to_string()),
                    ("supported", supported.to_string()),
                ],
            ),
            StoreError::ImportRecord { item, field } => Self::new(
                "import_record",
                &[("item", item), ("field", field.to_owned())],
            ),
            StoreError::Jhd { line, field } => Self::new(
                "jhd_invalid",
                &[("line", line.to_string()), ("field", field.to_owned())],
            ),
        }
    }
}

pub(crate) fn lock(store: &Mutex<Store>) -> Result<std::sync::MutexGuard<'_, Store>, CommandError> {
    store
        .lock()
        .map_err(|_| CommandError::new("store_unavailable", &[]))
}

/// `open_client` gövdesi: kayıt (store kilidi kısa tutulur), sonra motor iş parçacığında
/// harita. Bloklar; async bağlamdan `spawn_blocking` içinde çağrılmalıdır.
pub fn open_client_blocking(
    engine: &Engine,
    store: &Mutex<Store>,
    client_id: u32,
    chart_id: Option<u32>,
) -> Result<OpenedClient, CommandError> {
    let rec = lock(store)?.open_client(client_id, chart_id)?;
    let place = &rec.chart.birth.place;
    let chart = compute_chart_blocking(
        engine,
        rec.chart.resolved.jd_ut,
        place.lat,
        place.lon,
        rec.calc_settings.clone(),
    )?;
    Ok(OpenedClient {
        client: rec.client,
        chart_record: rec.chart,
        calc_settings: rec.calc_settings,
        view_state: rec.view_state,
        chart,
        recent_notes: rec.recent_notes,
        warnings: rec.warnings,
    })
}

/// Oturum geri yükleme: en son açılan harita; hiç açılmadıysa `None`.
pub fn restore_session_blocking(
    engine: &Engine,
    store: &Mutex<Store>,
) -> Result<Option<OpenedClient>, CommandError> {
    let last = lock(store)?.last_opened()?;
    match last {
        Some((client_id, chart_id)) => {
            open_client_blocking(engine, store, client_id, Some(chart_id)).map(Some)
        }
        None => Ok(None),
    }
}

pub fn create_client_blocking(
    engine: &Engine,
    store: &Mutex<Store>,
    input: NewClient,
) -> Result<OpenedClient, CommandError> {
    let (client, chart) = lock(store)?.create_client(&input)?;
    open_client_blocking(engine, store, client.id, Some(chart.id))
}

pub fn add_chart_blocking(
    engine: &Engine,
    store: &Mutex<Store>,
    client_id: u32,
    chart: NewChart,
) -> Result<OpenedClient, CommandError> {
    let rec = lock(store)?.add_chart(client_id, &chart)?;
    open_client_blocking(engine, store, client_id, Some(rec.id))
}

/// "Düzenle" kipinin kaydı: an yeniden çözülür, harita yeniden hesaplanır. tzdb farkı
/// uyarısını kabul etmek için aynı doğum verisiyle çağrılır.
pub fn update_chart_blocking(
    engine: &Engine,
    store: &Mutex<Store>,
    chart_id: u32,
    chart: NewChart,
) -> Result<OpenedClient, CommandError> {
    let rec = lock(store)?.update_chart(chart_id, &chart)?;
    open_client_blocking(engine, store, rec.client_id, Some(chart_id))
}

/// Ayar değişikliği (otomatik kayıt, "varsayılana döndür"): kaydeder ve yeniden hesaplar.
pub fn save_calc_settings_blocking(
    engine: &Engine,
    store: &Mutex<Store>,
    chart_id: u32,
    settings: CalcSettings,
) -> Result<Chart, CommandError> {
    let rec = {
        let mut s = lock(store)?;
        s.save_calc_settings(chart_id, &settings)?;
        s.chart(chart_id)?
    };
    let place = &rec.birth.place;
    compute_chart_blocking(engine, rec.resolved.jd_ut, place.lat, place.lon, settings)
}

/// Bloklayan gövdeyi havuzda çalıştırır. Görev paniklerse (`JoinError`) `task_failed` döner
/// (Plan 08 ertelenen madde 11); panik iletisi parametreye konmaz, kişisel veri taşıyabilir.
pub(crate) async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, CommandError> + Send + 'static,
) -> Result<T, CommandError> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|_| CommandError::new("task_failed", &[]))?
}

/// Yalnızca store işi yapan komutlar için: kilit, çağrı, hata eşleme.
pub(crate) async fn with_store<T: Send + 'static>(
    store: &StoreState,
    f: impl FnOnce(&mut Store) -> Result<T, StoreError> + Send + 'static,
) -> Result<T, CommandError> {
    let store = Arc::clone(store);
    blocking(move || Ok(f(&mut *lock(&store)?)?)).await
}

#[tauri::command]
#[specta::specta]
pub async fn open_client(
    engine: State<'_, Arc<Engine>>,
    store: State<'_, StoreState>,
    client_id: u32,
    chart_id: Option<u32>,
) -> Result<OpenedClient, CommandError> {
    let (engine, store) = (Arc::clone(&engine), Arc::clone(&store));
    blocking(move || open_client_blocking(&engine, &store, client_id, chart_id)).await
}

#[tauri::command]
#[specta::specta]
pub async fn restore_session(
    engine: State<'_, Arc<Engine>>,
    store: State<'_, StoreState>,
) -> Result<Option<OpenedClient>, CommandError> {
    let (engine, store) = (Arc::clone(&engine), Arc::clone(&store));
    blocking(move || restore_session_blocking(&engine, &store)).await
}

#[tauri::command]
#[specta::specta]
pub async fn create_client(
    engine: State<'_, Arc<Engine>>,
    store: State<'_, StoreState>,
    input: NewClient,
) -> Result<OpenedClient, CommandError> {
    let (engine, store) = (Arc::clone(&engine), Arc::clone(&store));
    blocking(move || create_client_blocking(&engine, &store, input)).await
}

#[tauri::command]
#[specta::specta]
pub async fn add_chart(
    engine: State<'_, Arc<Engine>>,
    store: State<'_, StoreState>,
    client_id: u32,
    chart: NewChart,
) -> Result<OpenedClient, CommandError> {
    let (engine, store) = (Arc::clone(&engine), Arc::clone(&store));
    blocking(move || add_chart_blocking(&engine, &store, client_id, chart)).await
}

#[tauri::command]
#[specta::specta]
pub async fn update_chart(
    engine: State<'_, Arc<Engine>>,
    store: State<'_, StoreState>,
    chart_id: u32,
    chart: NewChart,
) -> Result<OpenedClient, CommandError> {
    let (engine, store) = (Arc::clone(&engine), Arc::clone(&store));
    blocking(move || update_chart_blocking(&engine, &store, chart_id, chart)).await
}

#[tauri::command]
#[specta::specta]
pub async fn save_calc_settings(
    engine: State<'_, Arc<Engine>>,
    store: State<'_, StoreState>,
    chart_id: u32,
    settings: CalcSettings,
) -> Result<Chart, CommandError> {
    let (engine, store) = (Arc::clone(&engine), Arc::clone(&store));
    blocking(move || save_calc_settings_blocking(&engine, &store, chart_id, settings)).await
}

#[tauri::command]
#[specta::specta]
pub async fn save_view_state(
    store: State<'_, StoreState>,
    chart_id: u32,
    view_state: ViewState,
) -> Result<(), CommandError> {
    with_store(&store, move |s| s.save_view_state(chart_id, &view_state)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas::Date;

    #[test]
    fn a_panicking_blocking_task_is_task_failed_without_its_message() {
        let e = tauri::async_runtime::block_on(blocking::<()>(|| panic!("Ayşe"))).unwrap_err();
        assert_eq!(e.code, "task_failed");
        assert!(e.params.is_empty());
    }

    #[test]
    fn store_error_codes_are_stable() {
        let e = CommandError::from(StoreError::NotFound {
            what: "chart",
            id: 7,
        });
        assert_eq!(e.code, "not_found");
        assert_eq!(
            (e.params["what"].as_str(), e.params["id"].as_str()),
            ("chart", "7")
        );
        let e = CommandError::from(StoreError::InvalidInput("birth.date"));
        assert_eq!(
            (e.code.as_str(), e.params["field"].as_str()),
            ("invalid_input", "birth.date")
        );
        let local = Date::constant(2014, 3, 31).at(3, 30, 0, 0);
        let e = CommandError::from(StoreError::Moment(MomentError::NonexistentLocalTime {
            local,
            suggestion: Date::constant(2014, 3, 31).at(4, 30, 0, 0),
        }));
        assert_eq!(e.code, "nonexistent_local_time");
        assert_eq!(e.params["suggestion"], "2014-03-31T04:30:00");
        let e = CommandError::from(StoreError::Moment(MomentError::AmbiguousLocalTime {
            local,
            earlier_offset_secs: 10_800,
            later_offset_secs: 7_200,
        }));
        assert_eq!(e.code, "ambiguous_local_time");
        assert_eq!(e.params["later_offset_secs"], "7200");
        assert_eq!(CommandError::from(StoreError::LastChart).code, "last_chart");
        let e = CommandError::from(StoreError::Jhd {
            line: 5,
            field: "timezone",
        });
        assert_eq!(
            (e.code.as_str(), e.params["line"].as_str()),
            ("jhd_invalid", "5")
        );
        let e = CommandError::from(StoreError::DatabaseTooNew {
            found: 3,
            supported: 1,
        });
        assert_eq!(
            (e.code.as_str(), e.params["found"].as_str()),
            ("store_too_new", "3")
        );
        let e = CommandError::from(StoreError::Settings(
            jyotish_core::CoreError::InvalidSettings("dasha_year"),
        ));
        assert_eq!(e.code, "invalid_settings");
        let e = CommandError::from(StoreError::ImportVersion {
            found: 2,
            supported: 1,
        });
        assert_eq!(e.code, "import_version");
        let e = CommandError::from(StoreError::ImportRecord {
            item: "u".into(),
            field: "uuid",
        });
        assert_eq!(
            (e.code.as_str(), e.params["item"].as_str()),
            ("import_record", "u")
        );
    }
}
