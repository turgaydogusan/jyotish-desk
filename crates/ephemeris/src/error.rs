#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EphError {
    #[error("ephemeris path is not valid UTF-8 or contains a NUL byte")]
    InvalidPath,
    #[error("ephemeris data files missing in {path}: {detail}")]
    DataFilesMissing { path: String, detail: String },
    #[error("swiss ephemeris error: {0}")]
    Calc(String),
}
