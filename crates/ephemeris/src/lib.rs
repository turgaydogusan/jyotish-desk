//! Swiss Ephemeris için güvenli Rust sarmalayıcı.

mod engine;
mod error;
pub(crate) mod ffi;
mod swiss;
mod types;

pub use engine::Engine;
pub use error::EphError;
pub use swiss::SwissEph;
pub use types::{Ayanamsa, Body, BodyPosition, Precision, SiderealMode, Zodiac};

/// Gök cismi konumu sağlayan motor. Üst katmanlar testte sahte motor takar.
pub trait Ephemeris {
    /// Jeosantrik, görünür konum; boylam [0, 360).
    fn body(&mut self, jd_ut: f64, body: Body, zodiac: Zodiac) -> Result<BodyPosition, EphError>;
    /// Nütasyon dahil (gerçek) ayanamsa, derece; hassasiyet bayrağıyla.
    fn ayanamsa(&mut self, jd_ut: f64, mode: SiderealMode) -> Result<Ayanamsa, EphError>;
}

/// Gregoryen takvim tarihini ve UT saatini (ondalık) Jülyen gününe çevirir.
/// Saf bir hesaptır, SE global durumuna dokunmaz; kilit gerekmez.
pub fn julday_gregorian(year: i32, month: u32, day: u32, hour_ut: f64) -> f64 {
    // SAFETY: swe_julday yalnızca argümanlarını okur, global durum kullanmaz.
    unsafe { ffi::swe_julday(year, month as i32, day as i32, hour_ut, ffi::SE_GREG_CAL) }
}
