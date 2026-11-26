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
    #[error("non-finite ephemeris result: {0}")]
    NonFiniteResult(&'static str),
    #[error("invalid dasha input: {0}")]
    InvalidDashaInput(&'static str),
    /// Panchang öğesinin sonraki sınırı arama aralığında bulunamadı (yalnızca bozuk ya da
    /// sahte ephemeris'te: gerçek Ay her zaman Güneş'ten hızlıdır).
    #[error("no {0} boundary within the panchang search window")]
    PanchangSearch(&'static str),
    #[error("invalid transit input: {0}")]
    InvalidTransitInput(&'static str),
    /// Zaman çizelgesi, gün panchangı ya da muhurta araması girdisi geçersiz; alan adını taşır
    /// (değer taşımaz).
    #[error("invalid muhurta input: {0}")]
    InvalidMuhurtaInput(&'static str),
}
