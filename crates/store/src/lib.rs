//! Danışan ve harita kayıtları (spec §4 "Kalıcılık", §5 SQLite şeması). Hesaplanmış harita
//! saklanmaz; doğum verisi, çözülmüş UTC anı, hesap ayarları, görünüm durumu ve notlar
//! saklanır. Hesap yapmaz: `jyotish-core`'dan yalnızca tipleri kullanır.

pub mod birth;
mod chart_search;
mod charts;
mod compat;
mod csv;
mod db;
mod error;
pub mod exchange;
pub mod jhd;
mod layout;
pub mod model;
mod muhurta;
mod notes;
mod open;
mod prashna;
pub mod schema;
mod search;
pub mod settings;
pub mod view_state;

pub use birth::{parse_birth, resolve_birth, resolve_parsed, ParsedBirth};
pub use chart_search::{
    SavedSearch, SearchRow, MAX_SAVED_SEARCHES, MAX_TAG_CHARTS, SEARCH_PAGE_CHARTS,
};
pub use charts::{MAX_LIST, MAX_NAME_CHARS};
pub use compat::ChartBrief;
pub use csv::CSV_COLUMNS;
pub use db::{Clock, Store};
pub use error::StoreError;
pub use exchange::{
    ExportOptions, ImportReport, ImportWarning, ImportWarningCode, EXPORT_FORMAT,
    EXPORT_SCHEMA_VERSION, MAX_IMPORT_WARNINGS,
};
pub use jhd::{parse_jhd, JhdChart, JhdImport};
pub use model::{
    BirthData, ChartKind, ChartRecord, Client, FoldChoice, Gender, HomePlace, MomentWarning,
    NewChart, NewClient, NewPrashna, Note, Place, PrashnaMeta, ResolvedRecord, TimeCertainty,
    TzChoice,
};
pub use muhurta::{SavedMuhurtaFilter, MAX_MUHURTA_FILTERS, MUHURTA_FILTERS_SCHEMA_VERSION};
pub use notes::{normalize_tags, TagCount, MAX_NOTE_BYTES, MAX_TAG_CHARS, RECENT_NOTES};
pub use open::{OpenRecord, StoreWarning};
pub use prashna::{MAX_PRASHNA_NUMBER, MAX_QUESTION_CHARS, PRASHNA_CLIENT_NAME};
pub use schema::SCHEMA_VERSION;
pub use search::{SearchHit, SearchMatch};
pub use settings::{
    load_calc_settings, AppPrefs, PrefsLoad, PrefsWarning, SettingsLoad, ValueMigration,
    APP_PREFS_SCHEMA_VERSION, CALC_SETTINGS_MIGRATIONS, LOCALES,
};
pub use view_state::{
    canonical_uuid, chart_view_chara_path, chart_view_compat, chart_view_json, chart_view_path,
    check_panel_layout, check_view_state, empty_chara_path, empty_chart_view_json,
    parse_chara_path, parse_compat, parse_dasha_path, parse_panel_layout, parse_view_state,
    CompatRoles, Layout, Panel, PanelLayout, ViewState, MAX_LAYOUTS, MAX_PANELS,
    MAX_VIEW_STATE_BYTES, VIEW_STATE_SCHEMA_VERSION,
};
