use atlas::MomentError;
use jyotish_core::CoreError;

/// Store hataları. Arayüze `{code, params}` olarak uygulama katmanında çevrilir (spec §7).
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("schema migration: {0}")]
    Migration(#[from] rusqlite_migration::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// Veritabanı bu sürümden yeni bir uygulamayla göç ettirilmiş; açılmaz, değiştirilmez.
    #[error("database schema version {found} is newer than supported {supported}")]
    DatabaseTooNew { found: usize, supported: usize },
    #[error("{what} {id} not found")]
    NotFound { what: &'static str, id: u32 },
    /// Alan hatası; `field` formdaki alanın anahtarıdır (ör. `"birth.date"`).
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
    #[error(transparent)]
    Moment(#[from] MomentError),
    #[error(transparent)]
    Settings(#[from] CoreError),
    /// Danışanın son haritası silinemez; danışan silinir.
    #[error("a client must keep at least one chart")]
    LastChart,
    #[error("unsupported import file: {0}")]
    ImportFormat(String),
    #[error("unsupported import schema version {found} (supported: {supported})")]
    ImportVersion { found: u32, supported: u32 },
    #[error("invalid import record {item}: {field}")]
    ImportRecord { item: String, field: &'static str },
    /// JHD satırı okunamadı. `line` 1'den sayılır; dosya adından gelen alanlarda 0'dır.
    #[error("invalid JHD line {line}: {field}")]
    Jhd { line: usize, field: &'static str },
}
