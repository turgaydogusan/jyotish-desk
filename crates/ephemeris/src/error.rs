#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EphError {
    #[error("ephemeris path is not valid UTF-8, contains a NUL byte, contains a path separator (';', or ':' on Unix) that Swiss Ephemeris would split on, or is too long (> 242 bytes)")]
    InvalidPath,
    #[error(
        "SE_EPHE_PATH environment variable is set and would override the given ephemeris path"
    )]
    EnvOverride,
    #[error("ephemeris data files missing in {path}: {detail}")]
    DataFilesMissing { path: String, detail: String },
    #[error("swiss ephemeris error: {0}")]
    Calc(String),
}
