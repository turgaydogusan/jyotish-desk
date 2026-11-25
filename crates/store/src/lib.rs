//! Danışan ve harita kayıtları (spec §4 "Kalıcılık", §5 SQLite şeması). Hesaplanmış harita
//! saklanmaz; doğum verisi, çözülmüş UTC anı, hesap ayarları, görünüm durumu ve notlar
//! saklanır. Hesap yapmaz: `jyotish-core`'dan yalnızca tipleri kullanır.

pub mod birth;
mod charts;
mod csv;
mod db;
mod error;
pub mod exchange;
pub mod model;
mod notes;
mod open;
pub mod schema;
mod search;
pub mod settings;
pub mod view_state;

pub use birth::{parse_birth, resolve_birth, resolve_parsed, ParsedBirth};
pub use charts::{MAX_LIST, MAX_NAME_CHARS};
pub use csv::CSV_COLUMNS;
pub use db::{Clock, Store};
pub use error::StoreError;
pub use exchange::{
    ExportOptions, ImportReport, ImportWarning, ImportWarningCode, EXPORT_FORMAT,
    EXPORT_SCHEMA_VERSION,
};
pub use model::{
    BirthData, ChartKind, ChartRecord, Client, FoldChoice, Gender, MomentWarning, NewChart,
    NewClient, Note, Place, ResolvedRecord, TimeCertainty, TzChoice,
};
pub use notes::{normalize_tags, TagCount, MAX_TAG_CHARS, RECENT_NOTES};
pub use open::{OpenRecord, StoreWarning};
pub use schema::SCHEMA_VERSION;
pub use search::{SearchHit, SearchMatch};
pub use settings::{
    load_calc_settings, AppPrefs, SettingsLoad, ValueMigration, APP_PREFS_SCHEMA_VERSION,
    CALC_SETTINGS_MIGRATIONS, LOCALES,
};
pub use view_state::{
    chart_view_json, chart_view_path, parse_dasha_path, parse_panel_layout, parse_view_state,
    Layout, Panel, PanelLayout, ViewState, VIEW_STATE_SCHEMA_VERSION,
};
