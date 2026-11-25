//! Swiss Ephemeris C API'sinin kullandığımız alt kümesi.
//! İmzalar `swephexp.h` (2.10.x) ile birebir eşleşir.

use std::os::raw::{c_char, c_double, c_int};

pub const SE_GREG_CAL: c_int = 1;

pub const SEFLG_SWIEPH: i32 = 2;
pub const SEFLG_MOSEPH: i32 = 4;
pub const SEFLG_SPEED: i32 = 256;
pub const SEFLG_SIDEREAL: i32 = 64 * 1024;

pub const SE_SIDM_LAHIRI: i32 = 1;
pub const SE_SIDM_RAMAN: i32 = 3;
pub const SE_SIDM_KRISHNAMURTI: i32 = 5;
pub const SE_SIDM_TRUE_CITRA: i32 = 27;
pub const SE_SIDM_USER: i32 = 255;
/// `SE_SIDM_USER` ile birlikte: `t0` UT olarak okunur (swephexp.h; varsayılan TT).
pub const SE_SIDBIT_USER_UT: i32 = 1024;

/// `ascmc` dizisinde lagna ve MC indeksleri (swephexp.h).
pub const SE_ASC: usize = 0;
pub const SE_MC: usize = 1;

/// `serr` tamponları için SE'nin `AS_MAXCH` değeri.
pub const AS_MAXCH: usize = 256;

/// `swe_rise_trans` olayları ve bayrakları (swephexp.h).
pub const SE_CALC_RISE: i32 = 1;
pub const SE_CALC_SET: i32 = 2;
pub const SE_BIT_DISC_CENTER: i32 = 256;
pub const SE_BIT_NO_REFRACTION: i32 = 512;
pub const SE_BIT_GEOCTR_NO_ECL_LAT: i32 = 128;
/// Hint gün doğumu: disk merkezi, kırılmasız, jeosantrik ve ekliptik enlemsiz (swephexp.h).
pub const SE_BIT_HINDU_RISING: i32 =
    SE_BIT_DISC_CENTER | SE_BIT_NO_REFRACTION | SE_BIT_GEOCTR_NO_ECL_LAT;
/// `swe_rise_trans`'ı hızlı yöntem yerine tam (yavaş) aramaya zorlar (swephexp.h).
pub const SE_BIT_FORCE_SLOW_METHOD: i32 = 32768;
/// `swe_rise_trans` dönüşü: cisim bu aramada doğmuyor ya da batmıyor (swecl.c,
/// `swe_rise_trans_true_hor`: "rise or set not found"). Hata değildir.
pub const RISE_TRANS_NOT_FOUND: i32 = -2;

extern "C" {
    pub fn swe_set_ephe_path(path: *const c_char);
    pub fn swe_close();
    pub fn swe_version(s: *mut c_char) -> *mut c_char;
    pub fn swe_julday(
        year: c_int,
        month: c_int,
        day: c_int,
        hour: c_double,
        gregflag: c_int,
    ) -> c_double;
    pub fn swe_calc_ut(
        tjd_ut: c_double,
        ipl: i32,
        iflag: i32,
        xx: *mut c_double,
        serr: *mut c_char,
    ) -> i32;
    pub fn swe_set_sid_mode(sid_mode: i32, t0: c_double, ayan_t0: c_double);
    pub fn swe_get_ayanamsa_ex_ut(
        tjd_ut: c_double,
        iflag: i32,
        daya: *mut c_double,
        serr: *mut c_char,
    ) -> i32;
    /// `cusps` 13 double (1..=12 kullanılır), `ascmc` 10 double; hız dizileri NULL olabilir.
    pub fn swe_houses_ex2(
        tjd_ut: c_double,
        iflag: i32,
        geolat: c_double,
        geolon: c_double,
        hsys: c_int,
        cusps: *mut c_double,
        ascmc: *mut c_double,
        cusp_speed: *mut c_double,
        ascmc_speed: *mut c_double,
        serr: *mut c_char,
    ) -> c_int;
    /// `starname` gezegenler için NULL. `geopos` 3 double: boylam (doğu +), enlem, yükseklik (m).
    /// Sonuç `*tret`'e yazılır; swetest.c `call_rise_set` de tek bir double geçirir.
    pub fn swe_rise_trans(
        tjd_ut: c_double,
        ipl: i32,
        starname: *mut c_char,
        epheflag: i32,
        rsmi: i32,
        geopos: *mut c_double,
        atpress: c_double,
        attemp: c_double,
        tret: *mut c_double,
        serr: *mut c_char,
    ) -> i32;
}
