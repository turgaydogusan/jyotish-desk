use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use crate::{ffi, Body, BodyPosition, EphError, Ephemeris, Precision, SiderealMode, Zodiac};

/// SE global durumunu koruyan süreç çapında kilit.
static SWE_LOCK: Mutex<()> = Mutex::new(());

const J2000: f64 = 2_451_545.0;

/// Swiss Ephemeris'in tek sahibi. Var olduğu sürece başka örnek açılamaz
/// (açmaya çalışan iş parçacığı bekler).
pub struct SwissEph {
    path: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

fn read_cstr(buf: &[c_char]) -> String {
    // SAFETY: SE tamponu her zaman NUL ile sonlandırır; tampon AS_MAXCH uzunluğunda.
    unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

impl SwissEph {
    /// Veri dizinini ayarlar ve self-check yapar. Güneş ya da Ay dosyası
    /// eksikse `DataFilesMissing` döner; sessiz Moshier yedeği yoktur.
    pub fn open(path: &Path) -> Result<Self, EphError> {
        let guard = SWE_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let s = path.to_str().ok_or(EphError::InvalidPath)?;
        let c_path = CString::new(s).map_err(|_| EphError::InvalidPath)?;
        // SAFETY: kilit tutuluyor; SE yolu kendi tamponuna kopyalar.
        unsafe { ffi::swe_set_ephe_path(c_path.as_ptr()) };

        let mut eph = SwissEph {
            path: path.to_path_buf(),
            _guard: guard,
        };
        eph.self_check()?;
        Ok(eph)
    }

    fn self_check(&mut self) -> Result<(), EphError> {
        for body in [Body::Sun, Body::Moon] {
            let pos = self.body(J2000, body, Zodiac::Tropical)?;
            if pos.precision != Precision::SwissEph {
                return Err(EphError::DataFilesMissing {
                    path: self.path.display().to_string(),
                    detail: format!("{body:?} at J2000 fell back to Moshier"),
                });
            }
        }
        Ok(())
    }

    pub fn version() -> String {
        let mut buf: [c_char; ffi::AS_MAXCH] = [0; ffi::AS_MAXCH];
        // SAFETY: swe_version sabit bir sürüm dizgisini tampona yazar.
        unsafe { ffi::swe_version(buf.as_mut_ptr()) };
        read_cstr(&buf)
    }

    fn set_sid_mode(mode: SiderealMode) {
        let (m, t0, ayan_t0) = mode.se_params();
        // SAFETY: yalnızca kilit sahibi çağırır (tüm çağıranlar &mut self).
        unsafe { ffi::swe_set_sid_mode(m, t0, ayan_t0) };
    }
}

impl Ephemeris for SwissEph {
    fn body(&mut self, jd_ut: f64, body: Body, zodiac: Zodiac) -> Result<BodyPosition, EphError> {
        let mut iflag = ffi::SEFLG_SWIEPH | ffi::SEFLG_SPEED;
        if let Zodiac::Sidereal(mode) = zodiac {
            Self::set_sid_mode(mode);
            iflag |= ffi::SEFLG_SIDEREAL;
        }
        let mut xx = [0.0_f64; 6];
        let mut serr: [c_char; ffi::AS_MAXCH] = [0; ffi::AS_MAXCH];
        // SAFETY: xx 6 double, serr AS_MAXCH; kilit tutuluyor.
        let ret = unsafe {
            ffi::swe_calc_ut(
                jd_ut,
                body.se_id(),
                iflag,
                xx.as_mut_ptr(),
                serr.as_mut_ptr(),
            )
        };
        if ret < 0 {
            return Err(EphError::Calc(read_cstr(&serr)));
        }
        let precision = if ret & ffi::SEFLG_MOSEPH != 0 {
            Precision::Moshier
        } else {
            Precision::SwissEph
        };
        Ok(BodyPosition {
            lon: xx[0].rem_euclid(360.0),
            lat: xx[1],
            distance_au: xx[2],
            lon_speed: xx[3],
            lat_speed: xx[4],
            dist_speed: xx[5],
            precision,
        })
    }

    fn ayanamsa(&mut self, jd_ut: f64, mode: SiderealMode) -> Result<f64, EphError> {
        Self::set_sid_mode(mode);
        let mut daya = 0.0_f64;
        let mut serr: [c_char; ffi::AS_MAXCH] = [0; ffi::AS_MAXCH];
        // SAFETY: daya tek double, serr AS_MAXCH; kilit tutuluyor.
        let ret = unsafe {
            ffi::swe_get_ayanamsa_ex_ut(jd_ut, ffi::SEFLG_SWIEPH, &mut daya, serr.as_mut_ptr())
        };
        if ret < 0 {
            return Err(EphError::Calc(read_cstr(&serr)));
        }
        Ok(daya)
    }
}

impl Drop for SwissEph {
    fn drop(&mut self) {
        // SAFETY: kilit hâlâ tutuluyor (_guard bu gövdeden sonra düşer).
        unsafe { ffi::swe_close() };
    }
}
