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

/// `serr` tamponları için SE'nin `AS_MAXCH` değeri.
pub const AS_MAXCH: usize = 256;

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
}
