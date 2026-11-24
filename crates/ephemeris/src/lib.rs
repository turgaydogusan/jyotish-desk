//! Swiss Ephemeris için güvenli Rust sarmalayıcı.

#![allow(clippy::upper_case_acronyms)]

#[allow(dead_code)]
pub(crate) mod ffi;

/// Gregoryen takvim tarihini ve UT saatini (ondalık) Jülyen gününe çevirir.
/// Saf bir hesaptır, SE global durumuna dokunmaz; kilit gerekmez.
pub fn julday_gregorian(year: i32, month: u32, day: u32, hour_ut: f64) -> f64 {
    // SAFETY: swe_julday yalnızca argümanlarını okur, global durum kullanmaz.
    unsafe { ffi::swe_julday(year, month as i32, day as i32, hour_ut, ffi::SE_GREG_CAL) }
}
