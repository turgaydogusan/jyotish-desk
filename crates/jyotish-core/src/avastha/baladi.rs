//! Baladi ve Jagradadi avasthaları. İkisi de tek grahanın kendi yerleşiminden okunur; başka
//! grahaya, lagnaya ve saate bağlı değildir.
//!
//! Baladi: burç altışar derecelik beş dilime bölünür. Tek burçlarda (Koç, İkizler, …) dilimler
//! sırayla Bala, Kumara, Yuva, Vriddha, Mrita'dır; çift burçlarda sıra terstir.
//!
//! Jagradadi: yücelme burcunda ya da kendi burcunda Jagrat; dost ya da nötr grahanın burcunda
//! Swapna; düşman grahanın burcunda ya da düşüş burcunda Sushupti. Dostluk doğal (naisargika)
//! ilişkidir: grahanın, bulunduğu burcun lorduna ilişkisi.

use serde::{Deserialize, Serialize};

use crate::dignity::{exaltation_point, natural_relation, own_signs, sign_lord, Relation};
use crate::graha::Graha;
use crate::zodiac::Sign;

/// Baladi diliminin genişliği, derece.
pub const BALADI_SPAN_DEG: f64 = 6.0;

/// Yaş basamağı.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Baladi {
    Bala,
    Kumara,
    Yuva,
    Vriddha,
    Mrita,
}

impl Baladi {
    /// Tek burçtaki dilim sırası.
    pub const ALL: [Baladi; 5] = [
        Baladi::Bala,
        Baladi::Kumara,
        Baladi::Yuva,
        Baladi::Vriddha,
        Baladi::Mrita,
    ];
}

/// `sign` burcunda `deg_in_sign` derecedeki grahanın Baladi avasthası. Dilim sınırı sonraki
/// dilime aittir (6° Kumara'dır); 30° ve üstü son dilim sayılır.
pub fn baladi(sign: Sign, deg_in_sign: f64) -> Baladi {
    let step = (deg_in_sign / BALADI_SPAN_DEG).floor().clamp(0.0, 4.0) as usize;
    let odd_sign = sign.index().is_multiple_of(2);
    Baladi::ALL[if odd_sign { step } else { 4 - step }]
}

/// Uyanıklık durumu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Jagradadi {
    Jagrat,
    Swapna,
    Sushupti,
}

impl Jagradadi {
    pub const ALL: [Jagradadi; 3] = [Jagradadi::Jagrat, Jagradadi::Swapna, Jagradadi::Sushupti];
}

/// `sign` burcundaki grahanın Jagradadi avasthası; düğümler için `None`. Sıra: yücelme burcu ve
/// kendi burcu, düşüş burcu, sonra burç lorduna doğal ilişki.
pub fn jagradadi(graha: Graha, sign: Sign) -> Option<Jagradadi> {
    let (exaltation, _) = exaltation_point(graha)?;
    if sign == exaltation || own_signs(graha).contains(&sign) {
        return Some(Jagradadi::Jagrat);
    }
    if sign == Sign::from_index(exaltation.index() + 6) {
        return Some(Jagradadi::Sushupti);
    }
    Some(match natural_relation(graha, sign_lord(sign)) {
        Relation::Friend | Relation::Neutral => Jagradadi::Swapna,
        Relation::Enemy => Jagradadi::Sushupti,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graha::Graha::*;
    use crate::zodiac::Sign::*;

    #[test]
    fn baladi_runs_forward_in_odd_signs_and_backward_in_even_signs() {
        use Baladi::*;
        // Tek burç (Koç): 0–6 Bala, 6–12 Kumara, 12–18 Yuva, 18–24 Vriddha, 24–30 Mrita.
        let odd = [
            (0.0, Bala),
            (5.999, Bala),
            (6.0, Kumara),
            (11.999, Kumara),
            (12.0, Yuva),
            (17.999, Yuva),
            (18.0, Vriddha),
            (23.999, Vriddha),
            (24.0, Mrita),
            (29.999, Mrita),
        ];
        for (deg, want) in odd {
            assert_eq!(baladi(Aries, deg), want, "Aries {deg}");
            assert_eq!(baladi(Libra, deg), want, "Libra {deg}");
        }
        // Çift burç (Boğa): sıra terstir.
        let even = [
            (0.0, Mrita),
            (5.999, Mrita),
            (6.0, Vriddha),
            (12.0, Yuva),
            (17.999, Yuva),
            (18.0, Kumara),
            (24.0, Bala),
            (29.999, Bala),
        ];
        for (deg, want) in even {
            assert_eq!(baladi(Taurus, deg), want, "Taurus {deg}");
            assert_eq!(baladi(Pisces, deg), want, "Pisces {deg}");
        }
    }

    #[test]
    fn baladi_is_the_mirror_image_between_an_odd_and_an_even_sign() {
        // İkinci yazım: çift burçtaki basamak, tek burçta 30° − derecenin basamağıdır.
        for sign in crate::zodiac::Sign::ALL {
            let odd = sign.index().is_multiple_of(2);
            for tenth in 0..300 {
                let deg = f64::from(tenth) / 10.0 + 0.05;
                let forward = Baladi::ALL[(deg / 6.0) as usize];
                let mirrored = Baladi::ALL[((30.0 - deg) / 6.0) as usize];
                let want = if odd { forward } else { mirrored };
                assert_eq!(baladi(sign, deg), want, "{sign:?} {deg}");
            }
        }
        // Aralık dışı derece taşmaz.
        assert_eq!(baladi(Aries, 30.0), Baladi::Mrita);
        assert_eq!(baladi(Aries, -0.1), Baladi::Bala);
        assert_eq!(baladi(Taurus, 30.0), Baladi::Bala);
    }

    #[test]
    fn jagradadi_follows_the_sign_and_the_natural_relation_to_its_lord() {
        use Jagradadi::*;
        // Güneş: yücelme Koç, kendi burcu Aslan, düşüş Terazi.
        assert_eq!(jagradadi(Sun, Aries), Some(Jagrat));
        assert_eq!(jagradadi(Sun, Leo), Some(Jagrat));
        assert_eq!(jagradadi(Sun, Libra), Some(Sushupti));
        // Yengeç'in lordu Ay dost, İkizler'in lordu Merkür nötr: Swapna.
        assert_eq!(jagradadi(Sun, Cancer), Some(Swapna));
        assert_eq!(jagradadi(Sun, Gemini), Some(Swapna));
        // Boğa'nın lordu Venüs ve Kova'nın lordu Satürn düşman: Sushupti.
        assert_eq!(jagradadi(Sun, Taurus), Some(Sushupti));
        assert_eq!(jagradadi(Sun, Aquarius), Some(Sushupti));
        // Merkür Başak'ta hem yücelir hem kendi burcundadır; Balık düşüş burcudur.
        assert_eq!(jagradadi(Mercury, Virgo), Some(Jagrat));
        assert_eq!(jagradadi(Mercury, Pisces), Some(Sushupti));
        // Mars'ın düşüş burcu Yengeç'in lordu Ay dosttur; düşüş önce gelir.
        assert_eq!(jagradadi(Mars, Cancer), Some(Sushupti));
        // Ay kimseyi düşman saymaz: düşüş burcu Akrep dışında Sushupti olmaz.
        let moon: Vec<_> = crate::zodiac::Sign::ALL
            .into_iter()
            .filter(|&s| jagradadi(Moon, s) == Some(Sushupti))
            .collect();
        assert_eq!(moon, [Scorpio]);
        // Düğümlere avastha atanmaz.
        for sign in crate::zodiac::Sign::ALL {
            assert_eq!(jagradadi(Rahu, sign), None);
            assert_eq!(jagradadi(Ketu, sign), None);
        }
    }

    #[test]
    fn every_graha_takes_each_jagradadi_state_somewhere() {
        use Jagradadi::*;
        // Değişmez: yedi grahanın her biri on iki burçta bir durum alır; Jagrat en az iki
        // burçtadır (yücelme ve kendi burcu; Merkür'de ikisi Başak'ta çakışır ama İkizler var).
        for graha in [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn] {
            let states: Vec<_> = crate::zodiac::Sign::ALL
                .into_iter()
                .map(|s| jagradadi(graha, s).expect("seven grahas"))
                .collect();
            let count = |j| states.iter().filter(|&&s| s == j).count();
            assert!(count(Jagrat) >= 2, "{graha:?}");
            assert!(count(Swapna) >= 1, "{graha:?}");
            assert!(count(Sushupti) >= 1, "{graha:?}");
            assert_eq!(states.len(), 12);
        }
    }
}
