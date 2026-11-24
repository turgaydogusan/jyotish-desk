use crate::ffi;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Body {
    Sun,
    Moon,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
    MeanNode,
    TrueNode,
}

impl Body {
    pub(crate) fn se_id(self) -> i32 {
        match self {
            Self::Sun => 0,
            Self::Moon => 1,
            Self::Mercury => 2,
            Self::Venus => 3,
            Self::Mars => 4,
            Self::Jupiter => 5,
            Self::Saturn => 6,
            Self::MeanNode => 10,
            Self::TrueNode => 11,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SiderealMode {
    Lahiri,
    Raman,
    Krishnamurti,
    TrueChitra,
    /// Kullanıcı tanımlı: `t0_jd_ut` anında ayanamsa `ayan_t0_deg` derecedir.
    Custom {
        t0_jd_ut: f64,
        ayan_t0_deg: f64,
    },
}

impl SiderealMode {
    pub(crate) fn se_params(self) -> (i32, f64, f64) {
        match self {
            Self::Lahiri => (ffi::SE_SIDM_LAHIRI, 0.0, 0.0),
            Self::Raman => (ffi::SE_SIDM_RAMAN, 0.0, 0.0),
            Self::Krishnamurti => (ffi::SE_SIDM_KRISHNAMURTI, 0.0, 0.0),
            Self::TrueChitra => (ffi::SE_SIDM_TRUE_CITRA, 0.0, 0.0),
            Self::Custom {
                t0_jd_ut,
                ayan_t0_deg,
            } => (
                ffi::SE_SIDM_USER | ffi::SE_SIDBIT_USER_UT,
                t0_jd_ut,
                ayan_t0_deg,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Zodiac {
    Tropical,
    Sidereal(SiderealMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    /// Swiss Ephemeris veri dosyalarıyla tam hassasiyet.
    SwissEph,
    /// Dosya aralığı dışında Moshier analitik yedeği (düşük hassasiyet).
    Moshier,
}

/// Ayanamsa değeri ve hesabın hassasiyeti.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ayanamsa {
    pub deg: f64,
    pub precision: Precision,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyPosition {
    pub lon: f64,
    pub lat: f64,
    pub distance_au: f64,
    pub lon_speed: f64,
    pub lat_speed: f64,
    pub dist_speed: f64,
    pub precision: Precision,
}

impl BodyPosition {
    pub fn is_retrograde(&self) -> bool {
        self.lon_speed < 0.0
    }
}
