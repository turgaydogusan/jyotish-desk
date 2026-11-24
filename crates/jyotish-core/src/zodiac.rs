//! Zodyak aritmetiği: normalizasyon, burç, burç içi derece, nakshatra ve pada.

use serde::{Deserialize, Serialize};

/// Bir nakshatranın genişliği: 360° / 27 = 13°20′.
pub const NAKSHATRA_SPAN_DEG: f64 = 40.0 / 3.0;
/// Bir padanın genişliği: 360° / 108 = 3°20′.
pub const PADA_SPAN_DEG: f64 = 10.0 / 3.0;

/// [0, 360) aralığına indirger; `rem_euclid` küçük negatiflerde 360.0 verebilir.
pub fn normalize(deg: f64) -> f64 {
    let r = deg.rem_euclid(360.0);
    if r >= 360.0 {
        0.0
    } else {
        r
    }
}

/// İki boylam arasındaki en kısa yay, [0, 180].
pub fn angular_distance(a: f64, b: f64) -> f64 {
    let d = normalize(a - b);
    d.min(360.0 - d)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Sign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

impl Sign {
    pub const ALL: [Sign; 12] = [
        Sign::Aries,
        Sign::Taurus,
        Sign::Gemini,
        Sign::Cancer,
        Sign::Leo,
        Sign::Virgo,
        Sign::Libra,
        Sign::Scorpio,
        Sign::Sagittarius,
        Sign::Capricorn,
        Sign::Aquarius,
        Sign::Pisces,
    ];

    /// 12'ye göre mod alınır.
    pub fn from_index(i: usize) -> Sign {
        Self::ALL[i % 12]
    }

    /// Koç 0, Balık 11.
    pub fn index(self) -> usize {
        self as usize
    }

    /// Boylamın düştüğü burç.
    pub fn of(lon: f64) -> Sign {
        let i = (normalize(lon) / 30.0) as usize;
        Self::ALL[i.min(11)]
    }

    /// `from` burcundan sayınca bu burç kaçıncıdır: aynı burç 1, karşı burç 7.
    pub fn house_from(self, from: Sign) -> u8 {
        ((self.index() + 12 - from.index()) % 12 + 1) as u8
    }
}

/// Burç içi derece, [0, 30).
pub fn deg_in_sign(lon: f64) -> f64 {
    let l = normalize(lon);
    l - Sign::of(l).index() as f64 * 30.0
}

/// Sidereal boylamdan (nakshatra 1..=27, pada 1..=4). İkisi de aynı pada indeksinden
/// (0..108) türetilir, böylece her zaman tutarlıdır. `×3/10` çarpımı 3°20′'nin tüm
/// katlarında tam sınır verir (k·10/3 için k = 1..107 doğrulandı).
pub fn nakshatra_pada(sidereal_lon: f64) -> (u8, u8) {
    let idx = ((normalize(sidereal_lon) * 3.0 / 10.0) as usize).min(107);
    ((idx / 4 + 1) as u8, (idx % 4 + 1) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_wraps_into_range() {
        assert_eq!(normalize(370.0), 10.0);
        assert_eq!(normalize(-10.0), 350.0);
        assert_eq!(normalize(360.0), 0.0);
        assert_eq!(normalize(-1e-15), 0.0);
    }

    #[test]
    fn angular_distance_is_shortest_arc() {
        assert_eq!(angular_distance(355.0, 5.0), 10.0);
        assert_eq!(angular_distance(5.0, 355.0), 10.0);
        assert_eq!(angular_distance(0.0, 180.0), 180.0);
    }

    #[test]
    fn sign_of_boundaries() {
        assert_eq!(Sign::of(0.0), Sign::Aries);
        assert_eq!(Sign::of(29.999_999), Sign::Aries);
        assert_eq!(Sign::of(30.0), Sign::Taurus);
        assert_eq!(Sign::of(359.999_999), Sign::Pisces);
        assert_eq!(Sign::of(-1.0), Sign::Pisces);
        assert_eq!(Sign::from_index(13), Sign::Taurus);
        assert_eq!(Sign::Leo.index(), 4);
    }

    #[test]
    fn deg_in_sign_basic() {
        assert!((deg_in_sign(46.5) - 16.5).abs() < 1e-12);
        assert_eq!(deg_in_sign(30.0), 0.0);
        assert!((deg_in_sign(-0.5) - 29.5).abs() < 1e-12);
    }

    #[test]
    fn house_from_counts_inclusively() {
        assert_eq!(Sign::Aries.house_from(Sign::Aries), 1);
        assert_eq!(Sign::Libra.house_from(Sign::Aries), 7);
        assert_eq!(Sign::Pisces.house_from(Sign::Taurus), 11);
        assert_eq!(Sign::Aries.house_from(Sign::Gemini), 11);
    }

    #[test]
    fn nakshatra_pada_boundaries() {
        assert_eq!(nakshatra_pada(0.0), (1, 1));
        assert_eq!(nakshatra_pada(3.3333), (1, 1));
        assert_eq!(nakshatra_pada(3.334), (1, 2));
        assert_eq!(nakshatra_pada(13.3333), (1, 4));
        // 13°20′ tam sınırı 2. nakshatranın 1. padasıdır.
        assert_eq!(nakshatra_pada(NAKSHATRA_SPAN_DEG), (2, 1));
        // Rohini (40°–53°20′), 2. pada 43°20′–46°40′.
        assert_eq!(nakshatra_pada(46.0), (4, 2));
        // Uttara Bhadrapada (333°20′–346°40′), 4. pada.
        assert_eq!(nakshatra_pada(346.0), (26, 4));
        assert_eq!(nakshatra_pada(359.999_999), (27, 4));
    }
}
