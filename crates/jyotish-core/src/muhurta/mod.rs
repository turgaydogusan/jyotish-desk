//! Muhurta (ileri modüller spec'i §10, Alt proje 3.7).
//!
//! - `segments`: günün dilimleri ve sınıflar; saf fonksiyonlar, ephemeris görmez (K3).
//!
//! Hepsi olgudur: "uygun" ya da "uygunsuz" hükmü, etkinlik şablonu ve açıklama metni yoktur.

pub mod segments;

use serde::{Deserialize, Serialize};

use crate::zodiac::Sign;
use crate::CoreError;

pub use segments::{
    chandra_bala, day_segments, equal_parts, gulika_segment, is_panchaka, is_rikta, is_vishti,
    nakshatra_nature, rahu_segment, tara_bala, tithi_group, yamaganda_segment, ChandraBala,
    DaySegments, NakshatraNature, TaraBala, TithiGroup, ABHIJIT_MUHURTA, CHANDRA_FAVOURABLE,
    DAY_EIGHTHS, DAY_MUHURTAS, HORAS_PER_DAY, NAKSHATRA_NATURES, PANCHAKA_START_DEG, RAHU_SEGMENTS,
    TARA_FAVOURABLE, VAIDHRITI_YOGA, VYATIPATA_YOGA,
};

/// Yöntem kimliği (K7): gün panchangı ve arama sonucu taşır.
pub const MUHURTA_METHOD: &str = "muhurta.v1";

/// Muhurta işaretleri. Gün panchangında aralık olarak listelenir, aramada dışlanabilir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum MuhurtaFlag {
    /// Rikta tithi (4, 9, 14; her iki pakshada).
    Rikta,
    /// Vishti karana.
    Vishti,
    /// Vyatipata nitya yogası.
    Vyatipata,
    /// Vaidhriti nitya yogası.
    Vaidhriti,
    /// Gündüzün varaya göre Rahu'ya ait sekizde bir dilimi.
    RahuSegment,
    Yamaganda,
    Gulika,
    /// Ay sidereal 300°–360° arasında.
    Panchaka,
    /// Jüpiter yanık (v1 yanma kuralı).
    JupiterCombust,
    /// Venüs yanık.
    VenusCombust,
}

impl MuhurtaFlag {
    pub const ALL: [MuhurtaFlag; 10] = [
        MuhurtaFlag::Rikta,
        MuhurtaFlag::Vishti,
        MuhurtaFlag::Vyatipata,
        MuhurtaFlag::Vaidhriti,
        MuhurtaFlag::RahuSegment,
        MuhurtaFlag::Yamaganda,
        MuhurtaFlag::Gulika,
        MuhurtaFlag::Panchaka,
        MuhurtaFlag::JupiterCombust,
        MuhurtaFlag::VenusCombust,
    ];

    /// Gündüz dilimi işaretleri: yalnızca doğuşu ve batışı bilinen günde tanımlıdır.
    pub fn is_day_segment(self) -> bool {
        matches!(self, Self::RahuSegment | Self::Yamaganda | Self::Gulika)
    }
}

/// Kişiye bağlı etkenlerin girdisi: doğum haritasının Ay'ı. Yalnızca iki sayı taşır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct NatalMoon {
    /// Doğum nakshatrası, 1..=27 (sidereal).
    pub nakshatra: u8,
    /// Doğum Ay burcu (rasi çerçevesinde).
    pub sign: Sign,
}

impl NatalMoon {
    pub fn validate(&self) -> Result<(), CoreError> {
        if (1..=27).contains(&self.nakshatra) {
            Ok(())
        } else {
            Err(CoreError::InvalidMuhurtaInput("natal"))
        }
    }
}
