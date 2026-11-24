use ephemeris::EphError;

/// Çekirdek hesap hataları. Arayüze `{code, params}` olarak uygulama katmanında çevrilir.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CoreError {
    #[error(transparent)]
    Ephemeris(#[from] EphError),
    #[error("unsupported calc settings schema version {found} (supported: {supported})")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    #[error("invalid calc settings field: {0}")]
    InvalidSettings(&'static str),
    #[error("invalid location: lat {lat}, lon {lon}")]
    InvalidLocation { lat: f64, lon: f64 },
    #[error("invalid julian day: {0}")]
    InvalidJulianDay(f64),
}
