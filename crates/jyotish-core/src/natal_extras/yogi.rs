//! Yogi noktası, yogi, eş yogi ve avayogi (yol haritası §4.1).
//!
//! Tanım (nitya yoga geleneği): yogi noktası, Güneş ve Ay'ın sidereal boylamları toplamına
//! 93°20′ (yedi nakshatra; Pushya'nın başı) eklenerek bulunur. Yogi o noktanın nakshatra
//! lordudur (Vimshottari sırası); eş yogi noktanın burcunun lordudur; avayogi, yogi
//! nakshatrasından altıncı nakshatranın lordudur (yogi nakshatrası 1 sayılır).
//!
//! Nokta ve nakshatraları her zodyak kipinde sidereal boylamdandır; noktanın burcu rasi
//! çerçevesindedir (K5): tropikal-rasi kipinde nokta, grahalarla aynı yay kadar ötelenir.

use serde::{Deserialize, Serialize};

use crate::dasha::VIMSHOTTARI_ORDER;
use crate::dignity::sign_lord;
use crate::graha::Graha;
use crate::positions::Positions;
use crate::shadbala::rasi_lon;
use crate::zodiac::{deg_in_sign, nakshatra_pada, normalize, Sign};

/// Yogi noktasının sabit yayı: 93°20′ = 7 × 13°20′.
pub const YOGI_ARC_DEG: f64 = 280.0 / 3.0;
/// Avayogi nakshatrası yogi nakshatrasından kaçıncıdır (yogi nakshatrası 1 sayılır).
pub const AVAYOGI_COUNT: u8 = 6;
/// Rasi çerçevesi ile sidereal çerçeve arasındaki yay bundan küçükse sıfır sayılır (`Sidereal`
/// kipinde iki boylam aynı sayının iki yazımıdır ve son basamakta ayrışabilir).
const FRAME_EPS_DEG: f64 = 1e-9;

/// Nakshatranın (1..=27) Vimshottari lordu: Ashwini Ketu, Bharani Venüs … sıra dokuzda bir
/// yinelenir. Aralık dışı numara 27'ye göre sarılır (panik yok).
pub fn nakshatra_lord(nakshatra: u8) -> Graha {
    VIMSHOTTARI_ORDER[(usize::from(nakshatra) + 26) % 27 % 9]
}

/// Yogi noktasının sidereal boylamı, [0, 360).
pub fn yogi_point(sun_sidereal: f64, moon_sidereal: f64) -> f64 {
    normalize(normalize(sun_sidereal + moon_sidereal) + YOGI_ARC_DEG)
}

/// Yogi nakshatrasından (1..=27) altıncı nakshatra, 1..=27.
pub fn avayogi_nakshatra(yogi_nakshatra: u8) -> u8 {
    ((usize::from(yogi_nakshatra) + 26 + usize::from(AVAYOGI_COUNT) - 1) % 27 + 1) as u8
}

/// Rasi çerçevesinin sidereal çerçeveye göre ötelenmesi, derece: `Sidereal` kipinde 0,
/// tropikal-rasi kipinde ayanamsa. Güneş'in iki boylamının farkından okunur (`Positions`
/// ayanamsayı taşımaz).
pub fn rasi_frame_offset(positions: &Positions) -> f64 {
    let sun = positions.point(Graha::Sun);
    let offset = normalize(rasi_lon(sun) - sun.sidereal_lon);
    if (FRAME_EPS_DEG..=360.0 - FRAME_EPS_DEG).contains(&offset) {
        offset
    } else {
        0.0
    }
}

/// Yogi noktası ve ondan okunan üç graha.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct YogiFacts {
    /// Yogi noktasının sidereal boylamı, [0, 360).
    pub point_lon: f64,
    /// Noktanın rasi çerçevesindeki burcu (K5).
    pub sign: Sign,
    /// Rasi çerçevesinde burç içi derece, [0, 30).
    pub deg_in_sign: f64,
    /// Noktanın nakshatrası, 1..=27 (sidereal).
    pub nakshatra: u8,
    /// 1..=4.
    pub pada: u8,
    /// Noktanın nakshatra lordu.
    pub yogi: Graha,
    /// Eş yogi: noktanın burcunun lordu.
    pub co_yogi: Graha,
    /// Yogi nakshatrasından altıncı nakshatra, 1..=27.
    pub avayogi_nakshatra: u8,
    /// O nakshatranın lordu.
    pub avayogi: Graha,
}

/// Konumlardan yogi olguları. Lagna gerekmez.
pub fn yogi_facts(positions: &Positions) -> YogiFacts {
    let sun = positions.point(Graha::Sun).sidereal_lon;
    let moon = positions.point(Graha::Moon).sidereal_lon;
    let point_lon = yogi_point(sun, moon);
    let frame_lon = normalize(point_lon + rasi_frame_offset(positions));
    let sign = Sign::of(frame_lon);
    let (nakshatra, pada) = nakshatra_pada(point_lon);
    let avayogi_nakshatra = avayogi_nakshatra(nakshatra);
    YogiFacts {
        point_lon,
        sign,
        deg_in_sign: deg_in_sign(frame_lon),
        nakshatra,
        pada,
        yogi: nakshatra_lord(nakshatra),
        co_yogi: sign_lord(sign),
        avayogi_nakshatra,
        avayogi: nakshatra_lord(avayogi_nakshatra),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panchang::nitya_yoga_no;
    use crate::zodiac::NAKSHATRA_SPAN_DEG;
    use Graha::*;

    /// Yalnızca Güneş ve Ay'ı verilen sentetik konumlar; öteki grahalar Koç 5°'dedir.
    fn sky(sun: f64, moon: f64) -> Positions {
        let mut lons = [5.0; 9];
        lons[Sun as usize] = sun;
        lons[Moon as usize] = moon;
        Positions::synthetic(None, lons)
    }

    /// Tropikal-rasi kipinin konumları: sidereal boylamlar yerinde durur, burç ve burç içi
    /// derece `ayanamsa` kadar ötelenmiş boylamdan okunur.
    fn tropical(mut p: Positions, ayanamsa: f64) -> Positions {
        for g in &mut p.grahas {
            let frame = normalize(g.sidereal_lon + ayanamsa);
            g.rasi_sign = Sign::of(frame);
            g.deg_in_sign = deg_in_sign(frame);
        }
        p
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn the_arc_is_seven_nakshatras_and_the_point_wraps() {
        assert!(close(YOGI_ARC_DEG, 7.0 * NAKSHATRA_SPAN_DEG));
        assert!(close(YOGI_ARC_DEG, 93.0 + 20.0 / 60.0));
        // 10° + 101° + 93°20′ = 204°20′.
        assert!(close(yogi_point(10.0, 101.0), 204.0 + 20.0 / 60.0));
        // Toplam 360°'yi aşar: 346° + 71° = 417° → 57°; + 93°20′ = 150°20′.
        assert!(close(yogi_point(346.0, 71.0), 150.0 + 20.0 / 60.0));
        // Toplamın kendisi 360°'yi aşmaz ama yay ekleyince aşar: 300° + 10° + 93°20′ = 43°20′.
        assert!(close(yogi_point(300.0, 10.0), 43.0 + 20.0 / 60.0));
        // Girdi sarılmamış olabilir.
        assert!(close(yogi_point(370.0, -259.0), yogi_point(10.0, 101.0)));
    }

    #[test]
    fn nakshatra_lords_repeat_every_nine() {
        // Ashwini'den: Ketu, Venüs, Güneş, Ay, Mars, Rahu, Jüpiter, Satürn, Merkür; üç tur.
        let hand = [Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury];
        for n in 1..=27u8 {
            assert_eq!(nakshatra_lord(n), hand[usize::from(n - 1) % 9], "{n}");
        }
        // Aralık dışı numara sarılır.
        assert_eq!(nakshatra_lord(28), Ketu);
        assert_eq!(nakshatra_lord(0), Mercury);
    }

    #[test]
    fn hand_examples_give_the_point_the_yogi_the_co_yogi_and_the_avayogi() {
        // A: Güneş 10°, Ay 101° → nokta 204°20′. Vishakha (16: 200°–213°20′), 2. pada
        // (203°20′–206°40′); lordu Jüpiter. Burç Terazi (180°–210°), 24°20′; lordu Venüs.
        // Altıncı nakshatra: 16, 17, 18, 19, 20, 21 → Uttara Ashadha (21); lordu Güneş.
        let a = yogi_facts(&sky(10.0, 101.0));
        assert!(close(a.point_lon, 204.0 + 20.0 / 60.0));
        assert_eq!((a.nakshatra, a.pada, a.yogi), (16, 2, Jupiter));
        assert_eq!((a.sign, a.co_yogi), (Sign::Libra, Venus));
        assert!(close(a.deg_in_sign, 24.0 + 20.0 / 60.0));
        assert_eq!((a.avayogi_nakshatra, a.avayogi), (21, Sun));

        // B: Güneş 346°, Ay 71° → nokta 150°20′. Uttara Phalguni (12: 146°40′–160°), 2. pada
        // (150°–153°20′); lordu Güneş. Burç Başak (150°–180°), 0°20′; lordu Merkür. Altıncı
        // nakshatra: 12 … 17 → Anuradha (17); lordu Satürn.
        let b = yogi_facts(&sky(346.0, 71.0));
        assert_eq!((b.nakshatra, b.pada, b.yogi), (12, 2, Sun));
        assert_eq!((b.sign, b.co_yogi), (Sign::Virgo, Mercury));
        assert!(close(b.deg_in_sign, 20.0 / 60.0));
        assert_eq!((b.avayogi_nakshatra, b.avayogi), (17, Saturn));

        // C: Güneş 200°, Ay 31°40′ → nokta 325°. Purva Bhadrapada (25: 320°–333°20′), 2. pada
        // (323°20′–326°40′); lordu Jüpiter. Burç Kova (300°–330°), 25°; lordu Satürn. Altıncı
        // nakshatra Revati'yi aşar: 25, 26, 27, 1, 2, 3 → Krittika (3); lordu Güneş.
        let c = yogi_facts(&sky(200.0, 31.0 + 40.0 / 60.0));
        assert!(close(c.point_lon, 325.0));
        assert_eq!((c.nakshatra, c.pada, c.yogi), (25, 2, Jupiter));
        assert_eq!((c.sign, c.co_yogi), (Sign::Aquarius, Saturn));
        assert_eq!((c.avayogi_nakshatra, c.avayogi), (3, Sun));
        // Lagna sonucu etkilemez.
        let mut with_lagna = sky(200.0, 31.0 + 40.0 / 60.0);
        with_lagna.lagna_lon = Some(15.0);
        with_lagna.lagna_sign = Some(Sign::Aries);
        assert_eq!(yogi_facts(&with_lagna), c);
    }

    #[test]
    fn the_yogi_nakshatra_is_the_nitya_yoga_shifted_by_seven() {
        // İkinci yazım: nitya yoga, toplamın 13°20′'lik dilimidir (1..=27); nokta toplamdan tam
        // yedi dilim ileridedir. Yoga k ise yogi nakshatrası (k + 7), 27'yi aşınca sarılır.
        // Izgara dilim sınırlarına düşmez (sınırda iki yazım son basamakta ayrışabilir).
        let mut seen = [false; 27];
        for i in 0..72 {
            for j in 0..36 {
                let sun = f64::from(i) * 5.0 + 0.11;
                let moon = f64::from(j) * 10.0 + 0.07;
                let yoga = nitya_yoga_no(sun, moon);
                let facts = yogi_facts(&sky(sun, moon));
                assert_eq!(facts.nakshatra, (yoga + 6) % 27 + 1, "{sun} {moon}");
                seen[usize::from(facts.nakshatra) - 1] = true;
            }
        }
        assert!(seen.iter().all(|s| *s), "the grid visits every nakshatra");
    }

    #[test]
    fn avayogi_is_the_sixth_nakshatra_and_agrees_with_the_added_arc() {
        // Adım adım sayım: yogi nakshatrası 1, sonra beş adım.
        for n in 1..=27u8 {
            let mut at = n;
            for _ in 1..AVAYOGI_COUNT {
                at = if at == 27 { 1 } else { at + 1 };
            }
            assert_eq!(avayogi_nakshatra(n), at, "{n}");
        }
        // Yogi → avayogi çiftleri, elle (lord sırasında beş adım ileri).
        let pairs = [
            (Ketu, Rahu),
            (Venus, Jupiter),
            (Sun, Saturn),
            (Moon, Mercury),
            (Mars, Ketu),
            (Rahu, Venus),
            (Jupiter, Sun),
            (Saturn, Moon),
            (Mercury, Mars),
        ];
        for n in 1..=27u8 {
            let yogi = nakshatra_lord(n);
            let expected = pairs.iter().find(|(y, _)| *y == yogi).expect("pair").1;
            assert_eq!(nakshatra_lord(avayogi_nakshatra(n)), expected, "{n}");
        }
        // İkinci yazım: noktaya 186°40′ (on dört nakshatra) eklenir ve varılan nakshatranın
        // lorduna bakılır. Nakshatra aynı değildir (6. yerine 15.), lord aynıdır: 14 = 9 + 5.
        for i in 0..72 {
            for j in 0..36 {
                let facts = yogi_facts(&sky(f64::from(i) * 5.0 + 0.11, f64::from(j) * 10.0 + 0.07));
                let (other, _) = nakshatra_pada(facts.point_lon + 14.0 * NAKSHATRA_SPAN_DEG);
                assert_eq!(nakshatra_lord(other), facts.avayogi, "{i} {j}");
                assert_eq!((other + 27 - facts.nakshatra) % 27, 14);
            }
        }
    }

    #[test]
    fn the_point_sign_follows_the_rasi_frame() {
        // Güneş 10°, Ay 68°40′ → nokta sidereal 172°: Hasta (13: 160°–173°20′), lordu Ay.
        // Sidereal çerçevede burç Başak (lordu Merkür).
        let sidereal = sky(10.0, 68.0 + 40.0 / 60.0);
        assert_eq!(rasi_frame_offset(&sidereal), 0.0);
        let s = yogi_facts(&sidereal);
        assert_eq!((s.nakshatra, s.yogi), (13, Moon));
        assert_eq!((s.sign, s.co_yogi), (Sign::Virgo, Mercury));
        // Tropikal-rasi kipi, ayanamsa 24°: nokta rasi çerçevesinde 196°, Terazi 16° (lordu
        // Venüs). Nokta, nakshatra, yogi ve avayogi değişmez.
        let frame = tropical(sidereal, 24.0);
        assert!(close(rasi_frame_offset(&frame), 24.0));
        let t = yogi_facts(&frame);
        assert_eq!((t.sign, t.co_yogi), (Sign::Libra, Venus));
        assert!(close(t.deg_in_sign, 16.0));
        assert_eq!(
            (t.point_lon, t.nakshatra, t.pada, t.yogi, t.avayogi),
            (s.point_lon, s.nakshatra, s.pada, s.yogi, s.avayogi)
        );
        // Güneş 350°'deyken öteleme 360°'yi aşar; fark yine 24°'dir.
        let wrapped = tropical(sky(350.0, 20.0), 24.0);
        assert!(close(rasi_frame_offset(&wrapped), 24.0));
        // `Sidereal` kipinde son basamak gürültüsü öteleme sayılmaz.
        for k in 0..360 {
            let p = sky(f64::from(k) + 1.0 / 3.0, 0.0);
            assert_eq!(rasi_frame_offset(&p), 0.0, "{k}");
        }
    }
}
