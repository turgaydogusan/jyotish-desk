//! Chara karaka ve karakamsha (ileri modüller spec'i §5).
//!
//! Kaynak: BPHS, karaka bölümü; Jaimini Upadesha Sutraları. Grahalar burç içi derecelerine göre
//! büyükten küçüğe sıralanır. Sekizli şemada Rahu da sıralanır ve derecesi burcun sonundan
//! ölçülür (30 − derece); yedili şemada Rahu yoktur. Ketu karaka olmaz. Karaka birleşmesi
//! kuralları uygulanmaz. Derece rasi çerçevesindedir (K5).

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::positions::{GrahaPoint, Positions};
use crate::settings::KarakaScheme;
use crate::varga::{varga_position, Varga};
use crate::zodiac::Sign;

/// Eşitlik eşiği: iki etkin derece bu fark içindeyse (1″) ikisi de `tie: true` taşır.
pub const KARAKA_TIE_DEG: f64 = 1.0 / 3600.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum KarakaRole {
    Atma,
    Amatya,
    Bhratri,
    Matri,
    Pitri,
    Putra,
    Gnati,
    Dara,
}

/// Sekizli şemanın rolleri, sırayla.
pub const EIGHT_KARAKA_ROLES: [KarakaRole; 8] = [
    KarakaRole::Atma,
    KarakaRole::Amatya,
    KarakaRole::Bhratri,
    KarakaRole::Matri,
    KarakaRole::Pitri,
    KarakaRole::Putra,
    KarakaRole::Gnati,
    KarakaRole::Dara,
];

/// Yedili şemanın rolleri, sırayla (Pitri yoktur).
pub const SEVEN_KARAKA_ROLES: [KarakaRole; 7] = [
    KarakaRole::Atma,
    KarakaRole::Amatya,
    KarakaRole::Bhratri,
    KarakaRole::Matri,
    KarakaRole::Putra,
    KarakaRole::Gnati,
    KarakaRole::Dara,
];

/// Şemanın rolleri, sırayla.
pub fn karaka_roles(scheme: KarakaScheme) -> &'static [KarakaRole] {
    match scheme {
        KarakaScheme::Eight => &EIGHT_KARAKA_ROLES,
        KarakaScheme::Seven => &SEVEN_KARAKA_ROLES,
    }
}

/// Bir rolün grahası.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CharaKaraka {
    pub role: KarakaRole,
    pub graha: Graha,
    /// Sıralamada kullanılan derece: burç içi derece, [0, 30); Rahu için 30 − derece, (0, 30].
    pub effective_deg: f64,
    /// Etkin derecesi sıradaki komşusuyla 1″ içinde eşit.
    pub tie: bool,
}

/// Grahanın sıralamada kullanılan derecesi.
fn effective_deg(point: &GrahaPoint) -> f64 {
    if point.graha == Graha::Rahu {
        30.0 - point.deg_in_sign
    } else {
        point.deg_in_sign
    }
}

/// Chara karaka sırası: `scheme`'e göre 8 ya da 7 kayıt, rol sırasında. Eşit derecede sıra tam
/// `f64` değeriyle, o da eşitse Güneş…Rahu doğal sırasıyla belirlenir.
pub fn chara_karakas(positions: &Positions, scheme: KarakaScheme) -> Vec<CharaKaraka> {
    let roles = karaka_roles(scheme);
    // `Graha::ALL`'un ilk yedisi Güneş…Satürn, sekizincisi Rahu'dur.
    let mut ranked: Vec<(Graha, f64)> = positions.grahas[..roles.len()]
        .iter()
        .map(|p| (p.graha, effective_deg(p)))
        .collect();
    // Kararlı sıralama: eşit derecede doğal sıra korunur.
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    let near = |i: usize, j: usize| (ranked[i].1 - ranked[j].1).abs() <= KARAKA_TIE_DEG;
    (0..ranked.len())
        .map(|i| CharaKaraka {
            role: roles[i],
            graha: ranked[i].0,
            effective_deg: ranked[i].1,
            tie: (i > 0 && near(i, i - 1)) || (i + 1 < ranked.len() && near(i, i + 1)),
        })
        .collect()
}

/// Karakamsha: Atmakaraka'nın navamsa burcu ve grahaların ondan ev sayıları.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Karakamsha {
    pub atmakaraka: Graha,
    /// Atmakaraka'nın navamsa (D9) burcu.
    pub sign: Sign,
    /// Karakamsha rasi haritasına taşınmış: her grahanın D1 burcunun karakamshadan ev sayısı,
    /// `Graha::ALL` sırasında.
    pub houses_in_d1: [u8; 9],
    /// Swamsha: her grahanın D9 burcunun karakamshadan ev sayısı, `Graha::ALL` sırasında.
    pub houses_in_d9: [u8; 9],
}

/// Grahanın navamsa burcu, rasi çerçevesindeki konumundan (haritanın D9'uyla aynı kural).
pub fn navamsa_sign(point: &GrahaPoint) -> Sign {
    let lon = point.rasi_sign.index() as f64 * 30.0 + point.deg_in_sign;
    varga_position(Varga::D9, lon).0
}

/// `atmakaraka`'nın karakamshası. Lagnaya bağlı değildir.
pub fn karakamsha(positions: &Positions, atmakaraka: Graha) -> Karakamsha {
    let sign = navamsa_sign(positions.point(atmakaraka));
    Karakamsha {
        atmakaraka,
        sign,
        houses_in_d1: std::array::from_fn(|i| positions.grahas[i].rasi_sign.house_from(sign)),
        houses_in_d9: std::array::from_fn(|i| navamsa_sign(&positions.grahas[i]).house_from(sign)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;

    fn order(k: &[CharaKaraka]) -> Vec<Graha> {
        k.iter().map(|c| c.graha).collect()
    }

    #[test]
    fn eight_scheme_ranks_rahu_from_the_end_of_its_sign() {
        // Burç içi dereceler: Güneş 10, Ay 5, Mars 25, Merkür 20, Jüpiter 15, Venüs 12,
        // Satürn 2, Rahu 4 (etkin 26), Ketu 4.
        let p = Positions::synthetic(
            None,
            [10.0, 95.0, 145.0, 200.0, 255.0, 42.0, 302.0, 124.0, 304.0],
        );
        let k = chara_karakas(&p, KarakaScheme::Eight);
        assert_eq!(
            order(&k),
            [Rahu, Mars, Mercury, Jupiter, Venus, Sun, Moon, Saturn]
        );
        assert_eq!(
            k.iter().map(|c| c.role).collect::<Vec<_>>(),
            EIGHT_KARAKA_ROLES
        );
        assert_eq!(k[0].effective_deg, 26.0);
        assert_eq!(k[1].effective_deg, 25.0);
        assert!(k.iter().all(|c| !c.tie));
        // Yedili şema: Rahu yok, Pitri rolü yok.
        let k7 = chara_karakas(&p, KarakaScheme::Seven);
        assert_eq!(
            order(&k7),
            [Mars, Mercury, Jupiter, Venus, Sun, Moon, Saturn]
        );
        assert_eq!(
            k7.iter().map(|c| c.role).collect::<Vec<_>>(),
            SEVEN_KARAKA_ROLES
        );
        assert!(!SEVEN_KARAKA_ROLES.contains(&KarakaRole::Pitri));
    }

    #[test]
    fn ties_within_one_arcsecond_are_flagged_and_ordered() {
        let sec = 1.0 / 3600.0;
        // Mars 20°, Jüpiter 20° − 0,5″: 1″ içinde. Venüs 20° − 2,5″: Jüpiter'e 2″ uzak.
        let p = Positions::synthetic(
            None,
            [
                5.0,
                40.0,
                80.0,
                101.0,
                140.0 - 0.5 * sec,
                170.0 - 2.5 * sec,
                212.0,
                273.0,
                93.0,
            ],
        );
        let k = chara_karakas(&p, KarakaScheme::Eight);
        // Rahu etkin 27; Mars 20; Jüpiter; Venüs; Satürn 2; ...
        assert_eq!(order(&k)[..4], [Rahu, Mars, Jupiter, Venus]);
        assert_eq!(
            k.iter().map(|c| c.tie).collect::<Vec<_>>(),
            [false, true, true, false, false, false, false, false]
        );
        // Tam eşit derecede doğal sıra: Güneş, Ay'dan önce.
        let q = Positions::synthetic(
            None,
            [48.0, 18.0, 80.0, 101.0, 125.0, 157.0, 212.0, 273.0, 93.0],
        );
        let k = chara_karakas(&q, KarakaScheme::Seven);
        assert_eq!(order(&k)[..2], [Mars, Sun]);
        assert_eq!(order(&k)[2], Moon);
        assert_eq!((k[1].tie, k[2].tie, k[0].tie), (true, true, false));
    }

    #[test]
    fn rahu_at_the_start_of_a_sign_has_thirty_effective_degrees() {
        let p = Positions::synthetic(
            None,
            [29.5, 40.0, 80.0, 101.0, 142.0, 173.0, 212.0, 270.0, 90.0],
        );
        let k = chara_karakas(&p, KarakaScheme::Eight);
        assert_eq!((k[0].graha, k[0].effective_deg), (Rahu, 30.0));
        assert_eq!(k[1].graha, Sun);
    }

    #[test]
    fn karakamsha_counts_houses_from_the_navamsa_sign_of_the_atmakaraka() {
        // Mars Aslan 25°: ateş burcu Koç'tan başlar; 25° sekizinci parça (23°20′–26°40′):
        // Koç'tan 8. burç Akrep. Güneş Koç 11°: dördüncü parça (10°–13°20′): Yengeç.
        let p = Positions::synthetic(
            None,
            [11.0, 95.0, 145.0, 200.0, 255.0, 42.0, 302.0, 124.0, 304.0],
        );
        assert_eq!(navamsa_sign(p.point(Mars)), Sign::Scorpio);
        assert_eq!(navamsa_sign(p.point(Sun)), Sign::Cancer);
        let k = karakamsha(&p, Mars);
        assert_eq!((k.atmakaraka, k.sign), (Mars, Sign::Scorpio));
        // D1: Güneş Koç, Akrep'ten 6. burç; Mars Aslan, Akrep'ten 10.
        assert_eq!(k.houses_in_d1[Sun as usize], 6);
        assert_eq!(k.houses_in_d1[Mars as usize], 10);
        // D9: Mars'ın navamsa burcu karakamshanın kendisidir (1); Güneş Yengeç, Akrep'ten 9.
        assert_eq!(k.houses_in_d9[Mars as usize], 1);
        assert_eq!(k.houses_in_d9[Sun as usize], 9);
        for h in k.houses_in_d1.iter().chain(&k.houses_in_d9) {
            assert!((1..=12).contains(h));
        }
    }
}
