//! Cheshta bala (hareket gücü) ve ortalama boylamlar (ileri modüller spec'i §7, "4. Cheshta
//! bala"). Kaynak: BPHS, graha gücü bölümü; hesap adımlarında yaygın hesap geleneği.
//!
//! Ortalama boylamlar J2000 çağlı doğrusal ortalama öğelerdir (`L0 + L1 × T`; tropikal,
//! tarihin ekinoksu). Katsayılar: J. Meeus, *Astronomical Algorithms* (2. baskı), Tablo 31.A
//! (VSOP87 ortalama öğeleri, tarihin ortalama ekinoksu), yalnızca sabit ve doğrusal terim.
//! Gerçek boylam tropikal jeosantrik boylamdır (`ShadbalaAstro::tropical_lons`).

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::positions::Positions;
use crate::zodiac::normalize;

use super::temporal::{ayana_bala, paksha_elongation};
use super::ShadbalaAstro;

/// J2000.0 (JD 2451545,0) ve bir Jülyen yüzyılı, gün.
const J2000: f64 = 2_451_545.0;
const JULIAN_CENTURY_DAYS: f64 = 36_525.0;

/// Ortalama boylam öğeleri: (J2000'deki değer, yüzyıl başına ilerleme), derece.
pub const MEAN_SUN: (f64, f64) = (280.466_457, 36_000.769_827_8);
pub const MEAN_MERCURY: (f64, f64) = (252.250_906, 149_474.072_249_1);
pub const MEAN_VENUS: (f64, f64) = (181.979_801, 58_519.213_030_2);
pub const MEAN_MARS: (f64, f64) = (355.433_000, 19_141.696_447_1);
pub const MEAN_JUPITER: (f64, f64) = (34.351_519, 3_036.302_774_8);
pub const MEAN_SATURN: (f64, f64) = (50.077_444, 1_223.511_068_6);

/// Tropikal ortalama boylamlar, derece, [0, 360). `sun` jeosantrik ortalama Güneş'tir (Dünya'nın
/// güneş merkezli ortalama boylamı + 180°); diğerleri güneş merkezli ortalama boylamlardır.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeanLongitudes {
    pub sun: f64,
    pub mercury: f64,
    pub venus: f64,
    pub mars: f64,
    pub jupiter: f64,
    pub saturn: f64,
}

/// `jd_ut` anındaki ortalama boylamlar. UT ile dinamik zaman arasındaki fark (≈ 1 dk) en hızlı
/// öğede (Merkür, 4,1°/gün) 0,003°'den küçüktür ve yok sayılır.
pub fn mean_longitudes(jd_ut: f64) -> MeanLongitudes {
    let t = (jd_ut - J2000) / JULIAN_CENTURY_DAYS;
    let at = |(l0, l1): (f64, f64)| normalize(l0 + l1 * t);
    MeanLongitudes {
        sun: at(MEAN_SUN),
        mercury: at(MEAN_MERCURY),
        venus: at(MEAN_VENUS),
        mars: at(MEAN_MARS),
        jupiter: at(MEAN_JUPITER),
        saturn: at(MEAN_SATURN),
    }
}

/// İki boylamın dairesel orta noktası: `a`'dan `b`'ye kısa yayın ortası, [0, 360). Tam karşıt
/// iki boylamda yay `−180°` sayılır ve `a − 90°` döner (gerçek gökyüzünde ortalama ve gerçek
/// boylam hiçbir zaman karşıt olmaz).
pub fn circular_midpoint(a: f64, b: f64) -> f64 {
    // `a`'dan `b`'ye işaretli kısa yay, [-180, 180).
    let arc = normalize(b - a + 180.0) - 180.0;
    normalize(a + arc / 2.0)
}

/// Cheshta formülü: `ort` = `slow` ile `true_lon`'un dairesel orta noktası; `K` = (`apex` −
/// `ort`) mod 360, 180°'den büyükse `360 − K`; güç `K / 3` (0–60).
///
/// Dış gezegenlerde `slow` gezegenin ortalama boylamı, `apex` ortalama Güneş'tir; Merkür ve
/// Venüs'te `slow` ortalama Güneş, `apex` gezegenin güneş merkezli ortalama boylamıdır.
pub fn cheshta_from(slow: f64, true_lon: f64, apex: f64) -> f64 {
    let k = normalize(apex - circular_midpoint(slow, true_lon));
    (if k > 180.0 { 360.0 - k } else { k }) / 3.0
}

/// Grahanın cheshta balası. Güneş: ayana bala (iki katı alınmamış). Ay: paksha balanın iki
/// katı alınmamış hâli (`uzanım / 3`). Düğümler 0.
pub fn cheshta_bala(graha: Graha, positions: &Positions, astro: &ShadbalaAstro) -> f64 {
    let mean = &astro.mean;
    let true_lon = astro
        .tropical_lons
        .get(graha as usize)
        .copied()
        .unwrap_or(0.0);
    match graha {
        Graha::Sun => ayana_bala(Graha::Sun, true_lon),
        Graha::Moon => paksha_elongation(positions) / 3.0,
        Graha::Mars => cheshta_from(mean.mars, true_lon, mean.sun),
        Graha::Jupiter => cheshta_from(mean.jupiter, true_lon, mean.sun),
        Graha::Saturn => cheshta_from(mean.saturn, true_lon, mean.sun),
        Graha::Mercury => cheshta_from(mean.sun, true_lon, mean.mercury),
        Graha::Venus => cheshta_from(mean.sun, true_lon, mean.venus),
        Graha::Rahu | Graha::Ketu => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panchang::Vara;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn mean_longitudes_at_the_epoch_and_one_century_later() {
        let m = mean_longitudes(J2000);
        assert_eq!(
            (m.sun, m.mercury, m.venus, m.mars, m.jupiter, m.saturn),
            (
                280.466_457,
                252.250_906,
                181.979_801,
                355.433,
                34.351_519,
                50.077_444
            )
        );
        // Bir yüzyıl sonra: L0 + L1, tam turlar atılır (elle: 36000,7698278 = 100 tur + 0,7698278).
        let c = mean_longitudes(J2000 + JULIAN_CENTURY_DAYS);
        assert!((c.sun - 281.236_284_8).abs() < 1e-6, "{}", c.sun);
        // Mars: 355,433 + 19141,6964471 = 19497,1294471 = 54 × 360 + 57,1294471.
        assert!((c.mars - 57.129_447_1).abs() < 1e-6, "{}", c.mars);
        // Satürn: 50,077444 + 1223,5110686 = 1273,5885126 = 3 × 360 + 193,5885126.
        assert!((c.saturn - 193.588_512_6).abs() < 1e-6, "{}", c.saturn);
        // Günlük ilerleme: Güneş 0,98565°, Merkür 4,09234°, Venüs 1,60215°.
        let d = mean_longitudes(J2000 + 1.0);
        assert!((normalize(d.sun - m.sun) - 0.985_647).abs() < 1e-6);
        assert!((normalize(d.mercury - m.mercury) - 4.092_377).abs() < 1e-6);
        assert!((normalize(d.venus - m.venus) - 1.602_169).abs() < 1e-6);
        // J2000'den önce de [0, 360) içindedir.
        let before = mean_longitudes(J2000 - 400.0 * 365.25);
        for v in [
            before.sun,
            before.mercury,
            before.venus,
            before.mars,
            before.jupiter,
            before.saturn,
        ] {
            assert!((0.0..360.0).contains(&v), "{v}");
        }
    }

    #[test]
    fn circular_midpoint_takes_the_short_arc() {
        assert!(close(circular_midpoint(10.0, 50.0), 30.0));
        assert!(close(circular_midpoint(50.0, 10.0), 30.0));
        assert!(close(circular_midpoint(350.0, 30.0), 10.0));
        assert!(close(circular_midpoint(30.0, 350.0), 10.0));
        assert!(close(circular_midpoint(340.0, 350.0), 345.0));
        assert!(close(circular_midpoint(77.0, 77.0), 77.0));
        // Tam karşıt: yay −180° sayılır, orta nokta a − 90°.
        assert!(close(circular_midpoint(0.0, 180.0), 270.0));
    }

    #[test]
    fn cheshta_is_full_at_opposition_and_zero_at_conjunction() {
        // Dış gezegen: ortalama ve gerçek boylam 100°. Güneş karşıda (280°): K = 180 → 60.
        assert!(close(cheshta_from(100.0, 100.0, 280.0), 60.0));
        // Güneş'le kavuşumda (100°): K = 0 → 0.
        assert!(close(cheshta_from(100.0, 100.0, 100.0), 0.0));
        // Dörtlükte (Güneş 190° ya da 10°): K = 90 → 30.
        assert!(close(cheshta_from(100.0, 100.0, 190.0), 30.0));
        assert!(close(cheshta_from(100.0, 100.0, 10.0), 30.0));
        // Ara değer: ortalama 100°, gerçek 110° → orta 105°; Güneş 225° → K = 120 → 40.
        assert!(close(cheshta_from(100.0, 110.0, 225.0), 40.0));
        // 0°'yi geçen orta nokta: ortalama 355°, gerçek 5° → orta 0°; Güneş 300° → K = 300 →
        // 360 − 300 = 60 → 20.
        assert!(close(cheshta_from(355.0, 5.0, 300.0), 20.0));
        // İç gezegen: ortalama Güneş 100°, gerçek boylam 100°. Güneş merkezli ortalama boylam
        // 280° (Dünya ile Güneş arasında, alt kavuşum): 60. 100° (üst kavuşum): 0.
        assert!(close(cheshta_from(100.0, 100.0, 280.0), 60.0));
        assert!(close(cheshta_from(100.0, 100.0, 100.0), 0.0));
    }

    #[test]
    fn cheshta_by_graha() {
        let p = Positions::synthetic(
            Some(0.0),
            [10.0, 70.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0],
        );
        let astro = ShadbalaAstro {
            jd_ut: 0.5,
            sunrise_jd_ut: 0.25,
            sunset_jd_ut: 0.75,
            next_sunrise_jd_ut: 1.25,
            mc_lon: 270.0,
            // Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn (tropikal).
            tropical_lons: [90.0, 150.0, 200.0, 80.0, 280.0, 130.0, 10.0],
            mean: MeanLongitudes {
                sun: 100.0,
                mercury: 250.0,
                venus: 100.0,
                mars: 220.0,
                jupiter: 280.0,
                saturn: 350.0,
            },
            weekday: Vara::Sunday,
            year_entry_weekday: Vara::Sunday,
            month_entry_weekday: Vara::Sunday,
        };
        // Güneş: ayana bala, tropikal 90° → 59,3125 (iki katı alınmaz).
        assert!(close(cheshta_bala(Graha::Sun, &p, &astro), 59.3125));
        // Ay: uzanım 60° → 20.
        assert!(close(cheshta_bala(Graha::Moon, &p, &astro), 20.0));
        // Mars: orta(220, 200) = 210; K = (100 − 210) mod 360 = 250 → 110 → 36,666…
        assert!(close(cheshta_bala(Graha::Mars, &p, &astro), 110.0 / 3.0));
        // Jüpiter: orta(280, 280) = 280; K = (100 − 280) mod 360 = 180 → 60.
        assert!(close(cheshta_bala(Graha::Jupiter, &p, &astro), 60.0));
        // Satürn: orta(350, 10) = 0; K = 100 → 33,333…
        assert!(close(cheshta_bala(Graha::Saturn, &p, &astro), 100.0 / 3.0));
        // Merkür: orta(ortalama Güneş 100, gerçek 80) = 90; K = (250 − 90) = 160 → 53,333…
        assert!(close(cheshta_bala(Graha::Mercury, &p, &astro), 160.0 / 3.0));
        // Venüs: orta(100, 130) = 115; K = (100 − 115) mod 360 = 345 → 15 → 5.
        assert!(close(cheshta_bala(Graha::Venus, &p, &astro), 5.0));
        assert_eq!(cheshta_bala(Graha::Rahu, &p, &astro), 0.0);
    }
}
