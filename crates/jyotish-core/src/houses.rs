//! Lagna ve bhavalar. Bhava başlangıçları rasi çerçevesindedir (sidereal ya da tropikal).

use serde::{Deserialize, Serialize};

use crate::settings::HouseSystem;
use crate::zodiac::{deg_in_sign, nakshatra_pada, normalize, Sign};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lagna {
    /// Rasi çerçevesinde boylam.
    pub lon: f64,
    /// Sidereal boylam; nakshatra ve pada buradan.
    pub sidereal_lon: f64,
    pub sign: Sign,
    pub deg_in_sign: f64,
    pub nakshatra: u8,
    pub pada: u8,
}

impl Lagna {
    pub fn new(lon: f64, sidereal_lon: f64) -> Lagna {
        let lon = normalize(lon);
        let sidereal_lon = normalize(sidereal_lon);
        let (nakshatra, pada) = nakshatra_pada(sidereal_lon);
        Lagna {
            lon,
            sidereal_lon,
            sign: Sign::of(lon),
            deg_in_sign: deg_in_sign(lon),
            nakshatra,
            pada,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bhavas {
    pub system: HouseSystem,
    /// İlk bhavadan başlayarak 12 bhavanın başlangıç boylamı (rasi çerçevesi).
    /// Whole Sign: burç başları. Sripati: sandhi. Placidus: cusp.
    pub cusps: Vec<f64>,
}

pub fn whole_sign_cusps(lagna_sign: Sign) -> Vec<f64> {
    (0..12)
        .map(|i| Sign::from_index(lagna_sign.index() + i).index() as f64 * 30.0)
        .collect()
}

/// Whole Sign: lagna burcundan sayınca grahanın burcu.
pub fn whole_sign_house(sign: Sign, lagna_sign: Sign) -> u8 {
    sign.house_from(lagna_sign)
}

/// Sıralı bhava başlangıçlarına göre ev, 1..=12. Boylamdan geriye en yakın başlangıç,
/// içinde bulunulan bhavanın başlangıcıdır; başlangıç noktası o bhavaya dahildir.
pub fn house_from_cusps(lon: f64, cusps: &[f64]) -> u8 {
    let mut best = 0usize;
    let mut best_d = f64::INFINITY;
    for (i, c) in cusps.iter().enumerate() {
        // normalize() değil: bir ulp altındaki boylam 360.0'a yuvarlanıp 0'a sarmamalı.
        let d = (lon - c).rem_euclid(360.0);
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    (best + 1) as u8
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_sign_cusps_start_at_lagna_sign() {
        assert_eq!(
            whole_sign_cusps(Sign::Taurus),
            [30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0, 0.0]
        );
    }

    #[test]
    fn whole_sign_house_counts_from_lagna() {
        assert_eq!(whole_sign_house(Sign::Taurus, Sign::Taurus), 1);
        assert_eq!(whole_sign_house(Sign::Pisces, Sign::Taurus), 11);
        assert_eq!(whole_sign_house(Sign::Scorpio, Sign::Taurus), 7);
    }

    #[test]
    fn house_from_cusps_handles_uneven_and_wrapping_cusps() {
        // İstanbul 1987 sidereal Placidus (Task 1 swetest referansı).
        let cusps = [
            247.7970775,
            286.9650884,
            328.5411204,
            1.9451259,
            27.0589213,
            47.7845930,
            67.7970775,
            106.9650884,
            148.5411204,
            181.9451259,
            207.0589213,
            227.7845930,
        ];
        assert_eq!(house_from_cusps(250.0, &cusps), 1);
        assert_eq!(house_from_cusps(247.7970775, &cusps), 1);
        assert_eq!(house_from_cusps(247.0, &cusps), 12);
        assert_eq!(house_from_cusps(0.5, &cusps), 3);
        assert_eq!(house_from_cusps(1.95, &cusps), 4);
        assert_eq!(house_from_cusps(200.0, &cusps), 10);

        // Bir ulp altı: Koç whole-sign başlangıçlarında 29.999999999999996 → 1. ev.
        let ws: Vec<f64> = (0..12).map(|i| i as f64 * 30.0).collect();
        assert_eq!(house_from_cusps(29.999999999999996, &ws), 1);
        assert_eq!(house_from_cusps(30.0, &ws), 2);
    }

    #[test]
    fn lagna_new_fills_sign_and_nakshatra() {
        let l = Lagna::new(70.0, 46.0);
        assert_eq!(l.lon, 70.0);
        assert_eq!(l.sidereal_lon, 46.0);
        assert_eq!(l.sign, Sign::Gemini);
        assert!((l.deg_in_sign - 10.0).abs() < 1e-12);
        assert_eq!((l.nakshatra, l.pada), (4, 2));
        let wrapped = Lagna::new(-10.0, 370.0);
        assert_eq!(wrapped.lon, 350.0);
        assert_eq!(wrapped.sidereal_lon, 10.0);
    }
}
