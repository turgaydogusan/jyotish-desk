//! Bhava bala (ileri modüller spec'i §7, "Yöntem — Bhava bala"): bhavadhipati bala, bhava dig
//! bala ve bhava drishti bala. Kaynak: BPHS, bhava gücü bölümü.
//!
//! Bhavalar eşittir: bhava ortası `lagna + 30° × (h − 1)`; `CalcSettings.house_system`
//! bunları etkilemez. Bhavanın burcu rasi çerçevesindedir (K5) ve lagna burcundan sayılan
//! rasi evinin burcuyla aynıdır (K4).

use crate::graha::Graha;
use crate::lordship::{house_lord, house_sign};
use crate::positions::Positions;
use crate::zodiac::{angular_distance, deg_in_sign, normalize, Sign};

use super::sthana::rasi_lon;
use super::{drik_bala_at, ranks, BhavaBala, GrahaShadbala};

/// Bhava dig bala için burç türü.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BhavaSignKind {
    /// İnsan burçları: İkizler, Başak, Terazi, Kova, Yay'ın ilk yarısı.
    Human,
    /// Dört ayaklılar: Koç, Boğa, Aslan, Yay'ın ikinci yarısı, Oğlak'ın ilk yarısı.
    Quadruped,
    /// Su burçları: Yengeç, Balık, Oğlak'ın ikinci yarısı.
    Water,
    /// Akrep.
    Insect,
}

impl BhavaSignKind {
    /// Türün güçsüz olduğu bhava: bu bhavanın ortasında dig bala 0'dır.
    pub fn powerless_house(self) -> u8 {
        match self {
            Self::Human => 7,
            Self::Quadruped => 4,
            Self::Water => 10,
            Self::Insect => 1,
        }
    }
}

/// `sign` burcunun `deg` derecesindeki noktanın türü. Yay ve Oğlak 15°'de bölünür (ilk yarı
/// `[0°, 15°)`).
pub fn bhava_sign_kind(sign: Sign, deg: f64) -> BhavaSignKind {
    let first_half = deg < 15.0;
    match sign {
        Sign::Gemini | Sign::Virgo | Sign::Libra | Sign::Aquarius => BhavaSignKind::Human,
        Sign::Aries | Sign::Taurus | Sign::Leo => BhavaSignKind::Quadruped,
        Sign::Cancer | Sign::Pisces => BhavaSignKind::Water,
        Sign::Scorpio => BhavaSignKind::Insect,
        Sign::Sagittarius => {
            if first_half {
                BhavaSignKind::Human
            } else {
                BhavaSignKind::Quadruped
            }
        }
        Sign::Capricorn => {
            if first_half {
                BhavaSignKind::Quadruped
            } else {
                BhavaSignKind::Water
            }
        }
    }
}

/// `house`. bhavanın ortası: `lagna_lon + 30° × (house − 1)`, [0, 360). `lagna_lon` hangi
/// çerçevedeyse sonuç da o çerçevededir.
pub fn bhava_midpoint(lagna_lon: f64, house: u8) -> f64 {
    normalize(lagna_lon + 30.0 * (f64::from(house) - 1.0))
}

/// Bhava dig bala: bhava ortasının, burç türünün güçsüz olduğu bhavanın ortasına açısal
/// uzaklığı / 3. Bhava ortaları 30° arayla durduğu için değer 0, 10, …, 60'tır.
pub fn bhava_dig_bala(house: u8, kind: BhavaSignKind) -> f64 {
    angular_distance(
        bhava_midpoint(0.0, house),
        bhava_midpoint(0.0, kind.powerless_house()),
    ) / 3.0
}

/// Rasi çerçevesi ile sidereal boylam arasındaki fark, derece: `Sidereal` kipinde ve sentetik
/// konumlarda tam 0, tropikal-rasi kipinde ayanamsadır. Bütün grahalarda aynıdır; Güneş'ten
/// okunur.
fn frame_offset(positions: &Positions) -> f64 {
    let sun = positions.point(Graha::Sun);
    normalize(rasi_lon(sun) - sun.sidereal_lon)
}

/// On iki bhavanın gücü ve sırası. `grahas` yedi grahanın Shadbala sonucudur
/// (`SHADBALA_GRAHAS` sırasında). Doğum saati bilinmiyorsa `None`.
pub fn bhava_balas(positions: &Positions, grahas: &[GrahaShadbala; 7]) -> Option<[BhavaBala; 12]> {
    let lagna_lon = positions.lagna_lon?;
    let lagna_sign = positions.lagna_sign?;
    // Lagnanın rasi çerçevesindeki burç içi derecesi: her bhava ortasının derecesi de budur.
    let deg = deg_in_sign(lagna_lon + frame_offset(positions));
    let mut bhavas: [BhavaBala; 12] = std::array::from_fn(|i| {
        let house = i as u8 + 1;
        let lord = house_lord(lagna_sign, house);
        // Burç lordu her zaman yedi grahadan biridir.
        let lord_bala = grahas[lord as usize].total;
        let dig = bhava_dig_bala(house, bhava_sign_kind(house_sign(lagna_sign, house), deg));
        let drishti = drik_bala_at(bhava_midpoint(lagna_lon, house), None, positions);
        BhavaBala {
            house,
            lord,
            lord_bala,
            dig,
            drishti,
            total: lord_bala + dig + drishti,
            rank: 0,
        }
    });
    let order = ranks(std::array::from_fn::<f64, 12, _>(|i| bhavas[i].total));
    for (b, rank) in bhavas.iter_mut().zip(order) {
        b.rank = rank;
    }
    Some(bhavas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use BhavaSignKind::*;

    #[test]
    fn sign_kinds_with_the_two_split_signs() {
        // Elle yazılmış tablo: (burç, ilk yarının türü, ikinci yarının türü).
        let table = [
            (Sign::Aries, Quadruped, Quadruped),
            (Sign::Taurus, Quadruped, Quadruped),
            (Sign::Gemini, Human, Human),
            (Sign::Cancer, Water, Water),
            (Sign::Leo, Quadruped, Quadruped),
            (Sign::Virgo, Human, Human),
            (Sign::Libra, Human, Human),
            (Sign::Scorpio, Insect, Insect),
            (Sign::Sagittarius, Human, Quadruped),
            (Sign::Capricorn, Quadruped, Water),
            (Sign::Aquarius, Human, Human),
            (Sign::Pisces, Water, Water),
        ];
        for (sign, first, second) in table {
            assert_eq!(bhava_sign_kind(sign, 0.0), first, "{sign:?}");
            assert_eq!(bhava_sign_kind(sign, 14.999), first, "{sign:?}");
            assert_eq!(bhava_sign_kind(sign, 15.0), second, "{sign:?}");
            assert_eq!(bhava_sign_kind(sign, 29.999), second, "{sign:?}");
        }
        assert_eq!(
            [Human, Quadruped, Water, Insect].map(BhavaSignKind::powerless_house),
            [7, 4, 10, 1]
        );
    }

    #[test]
    fn bhava_midpoints_step_by_thirty_degrees() {
        assert_eq!(bhava_midpoint(100.0, 1), 100.0);
        assert_eq!(bhava_midpoint(100.0, 4), 190.0);
        assert_eq!(bhava_midpoint(100.0, 10), 10.0);
        assert_eq!(bhava_midpoint(350.0, 2), 20.0);
    }

    #[test]
    fn bhava_dig_is_zero_at_the_powerless_house_and_sixty_opposite() {
        // İnsan burcu: 7. bhavada 0, 1. bhavada 60; 4. ve 10. bhavada 30.
        assert_eq!(bhava_dig_bala(7, Human), 0.0);
        assert_eq!(bhava_dig_bala(1, Human), 60.0);
        assert_eq!(bhava_dig_bala(4, Human), 30.0);
        assert_eq!(bhava_dig_bala(10, Human), 30.0);
        assert_eq!(bhava_dig_bala(8, Human), 10.0);
        // Dört ayaklı: 4. bhavada 0, 10. bhavada 60.
        assert_eq!(bhava_dig_bala(4, Quadruped), 0.0);
        assert_eq!(bhava_dig_bala(10, Quadruped), 60.0);
        assert_eq!(bhava_dig_bala(1, Quadruped), 30.0);
        assert_eq!(bhava_dig_bala(12, Quadruped), 40.0);
        // Su burcu: 10. bhavada 0, 4. bhavada 60.
        assert_eq!(bhava_dig_bala(10, Water), 0.0);
        assert_eq!(bhava_dig_bala(4, Water), 60.0);
        assert_eq!(bhava_dig_bala(12, Water), 20.0);
        // Akrep: 1. bhavada 0, 7. bhavada 60.
        assert_eq!(bhava_dig_bala(1, Insect), 0.0);
        assert_eq!(bhava_dig_bala(7, Insect), 60.0);
        assert_eq!(bhava_dig_bala(3, Insect), 20.0);
        // Her tür için on iki değer 0, 10, …, 60 kümesindedir ve karşıt bhavaların toplamı 60'tır.
        for kind in [Human, Quadruped, Water, Insect] {
            for h in 1..=6u8 {
                let (a, b) = (bhava_dig_bala(h, kind), bhava_dig_bala(h + 6, kind));
                assert_eq!(a + b, 60.0, "{kind:?} {h}");
                assert_eq!(a % 10.0, 0.0, "{kind:?} {h}");
            }
        }
    }
}
