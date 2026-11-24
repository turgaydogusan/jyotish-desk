//! Dokuz graha.

use serde::{Deserialize, Serialize};

use crate::dignity::Dignity;
use crate::zodiac::Sign;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Graha {
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
    Rahu,
    Ketu,
}

impl Graha {
    /// Geleneksel sıra (gün sırası, sonra düğümler); `Chart::grahas` bu sıradadır.
    pub const ALL: [Graha; 9] = [
        Graha::Sun,
        Graha::Moon,
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Saturn,
        Graha::Rahu,
        Graha::Ketu,
    ];

    pub fn is_node(self) -> bool {
        matches!(self, Self::Rahu | Self::Ketu)
    }
}

/// Bir grahanın haritadaki konumu (spec §5 `GrahaPosition`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GrahaPosition {
    pub graha: Graha,
    /// Rasi çerçevesinde boylam, [0, 360): `Sidereal` kipinde sidereal,
    /// `TropicalRasiSiderealNakshatra` kipinde tropikal.
    pub lon: f64,
    /// Her kipte sidereal boylam; nakshatra ve pada buradan.
    pub sidereal_lon: f64,
    /// Ekliptik enlem, derece.
    pub lat: f64,
    /// Rasi çerçevesinde boylam hızı, derece/gün.
    pub speed: f64,
    pub retrograde: bool,
    pub sign: Sign,
    pub deg_in_sign: f64,
    pub nakshatra: u8,
    pub pada: u8,
    pub combust: bool,
    /// Düğümler için `None`.
    pub dignity: Option<Dignity>,
    pub house: u8,
    /// D1 ve D9 burcu aynı (rasi çerçevesinde).
    pub vargottama: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_in_traditional_order_and_nodes() {
        assert_eq!(
            Graha::ALL,
            [
                Graha::Sun,
                Graha::Moon,
                Graha::Mars,
                Graha::Mercury,
                Graha::Jupiter,
                Graha::Venus,
                Graha::Saturn,
                Graha::Rahu,
                Graha::Ketu
            ]
        );
        let nodes: Vec<_> = Graha::ALL.into_iter().filter(|g| g.is_node()).collect();
        assert_eq!(nodes, [Graha::Rahu, Graha::Ketu]);
    }
}
