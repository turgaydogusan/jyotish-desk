//! Swiss Ephemeris için güvenli Rust sarmalayıcı.

mod engine;
mod error;
pub(crate) mod ffi;
mod swiss;
mod types;

pub use engine::Engine;
pub use error::EphError;
pub use swiss::SwissEph;
pub use types::{
    Ayanamsa, Body, BodyPosition, HouseSystem, Houses, Precision, RiseSet, SiderealMode, SunDisc,
    Zodiac,
};

/// Gök cismi konumu sağlayan motor. Üst katmanlar testte sahte motor takar.
pub trait Ephemeris {
    /// Jeosantrik, görünür konum; boylam [0, 360).
    fn body(&mut self, jd_ut: f64, body: Body, zodiac: Zodiac) -> Result<BodyPosition, EphError>;
    /// Nütasyon dahil (gerçek) ayanamsa, derece; hassasiyet bayrağıyla.
    fn ayanamsa(&mut self, jd_ut: f64, mode: SiderealMode) -> Result<Ayanamsa, EphError>;
    /// Lagna, MC ve 12 ev başlangıcı. `zodiac` sidereal ise ayanamsa (nütasyon dahil)
    /// düşülmüştür. `geo_lat` kuzey +, `geo_lon` doğu +, derece.
    fn houses(
        &mut self,
        jd_ut: f64,
        geo_lat: f64,
        geo_lon: f64,
        system: HouseSystem,
        zodiac: Zodiac,
    ) -> Result<Houses, EphError>;
    /// Güneş'in `after_jd_ut`'den sonraki ilk doğuşu ya da batışı, UT JD. Güneş bu aramada
    /// ufku geçmiyorsa (kutup gecesi ya da gündüzü) `Ok(None)`. `geo_lat` kuzey +,
    /// `geo_lon` doğu +, derece; gözlemci deniz seviyesinde.
    fn sun_rise_set(
        &mut self,
        after_jd_ut: f64,
        event: RiseSet,
        geo_lat: f64,
        geo_lon: f64,
        disc: SunDisc,
    ) -> Result<Option<f64>, EphError>;
    /// Swiss Ephemeris sürüm dizgisi (ör. "2.10.03").
    fn swe_version(&self) -> String;
}

/// Gregoryen takvim tarihini ve UT saatini (ondalık) Jülyen gününe çevirir.
/// Saf bir hesaptır, SE global durumuna dokunmaz; kilit gerekmez.
pub fn julday_gregorian(year: i32, month: u32, day: u32, hour_ut: f64) -> f64 {
    // SAFETY: swe_julday yalnızca argümanlarını okur, global durum kullanmaz.
    unsafe { ffi::swe_julday(year, month as i32, day as i32, hour_ut, ffi::SE_GREG_CAL) }
}
