//! Lajjitadi avasthaları: yerleşim, birliktelik ve bakıştan altı durum. Bir graha aynı anda
//! birden çok durumda olabilir; sonuç, kural sırasında bir listedir ve boş olabilir.
//!
//! Bu dosyadaki okuma (varyant noktaları `tests/golden/modules/avastha/REVIEW.md`'dedir):
//! - **Birliktelik** aynı rasi burcunda bulunmaktır; **bakış** burçtan burca tam graha
//!   bakışıdır (`graha_aspect_signs`). Rahu ve Ketu bakmaz.
//! - **Dost** ve **düşman** doğal (naisargika) ilişkidir: durumu sorulan grahanın ötekine
//!   ilişkisi. Düğümler kimsenin dostu ya da düşmanı değildir.
//! - **İyicil** ve **kötücül** `nature` sınıfıdır (K6).
//!
//! Kurallar:
//! 1. Lajjita: graha lagnadan 5. evde ve Güneş, Mars, Satürn, Rahu ya da Ketu ile birlikte.
//! 2. Garvita: graha yücelme burcunda ya da moolatrikona aralığında.
//! 3. Kshudhita: graha düşmanının burcunda, ya da bir düşmanıyla birlikte, ya da bir düşmanının
//!    bakışı altında, ya da Satürn ile birlikte.
//! 4. Trushita: graha su burcunda (Yengeç, Akrep, Balık), bir düşmanının bakışı altında ve
//!    hiçbir iyicil ona bakmıyor.
//! 5. Mudita: graha dostunun burcunda, ya da bir dostuyla birlikte, ya da bir dostunun bakışı
//!    altında, ya da Jüpiter ile birlikte.
//! 6. Kshobhita: graha Güneş ile birlikte ve bir kötücülün ya da bir düşmanının bakışı altında.

use serde::{Deserialize, Serialize};

use crate::dignity::{exaltation_point, moolatrikona, natural_relation, sign_lord, Relation};
use crate::drishti::{graha_aspect_signs, ASPECTING_GRAHAS};
use crate::graha::Graha;
use crate::nature::{nature, Nature};
use crate::positions::Positions;
use crate::zodiac::Sign;

/// Lajjita'nın evi (lagnadan).
pub const LAJJITA_HOUSE: u8 = 5;

/// Lajjita'yı oluşturan birliktelikler.
pub const LAJJITA_COMPANIONS: [Graha; 5] = [
    Graha::Sun,
    Graha::Mars,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// Trushita'nın su burçları.
pub const WATER_SIGNS: [Sign; 3] = [Sign::Cancer, Sign::Scorpio, Sign::Pisces];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Lajjitadi {
    Lajjita,
    Garvita,
    Kshudhita,
    Trushita,
    Mudita,
    Kshobhita,
}

impl Lajjitadi {
    /// Kural sırası: raporun liste sırası.
    pub const ALL: [Lajjitadi; 6] = [
        Lajjitadi::Lajjita,
        Lajjitadi::Garvita,
        Lajjitadi::Kshudhita,
        Lajjitadi::Trushita,
        Lajjitadi::Mudita,
        Lajjitadi::Kshobhita,
    ];
}

/// Grahanın içinde bulunduğu bir Lajjitadi durumu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct LajjitadiHit {
    pub state: Lajjitadi,
    /// Kuralı sağlayan öteki grahalar (birlikte olan ya da bakan), `Graha::ALL` sırasında.
    /// Kural yalnızca burçtan sağlanıyorsa boştur.
    pub grahas: Vec<Graha>,
}

/// `graha` ile aynı rasi burcundaki öteki grahalar, `Graha::ALL` sırasında (düğümler dahil).
pub fn conjunct_grahas(graha: Graha, positions: &Positions) -> Vec<Graha> {
    positions
        .occupants(positions.sign_of(graha))
        .into_iter()
        .filter(|&g| g != graha)
        .collect()
}

/// `graha`'nın burcuna tam bakışla bakan grahalar, `ASPECTING_GRAHAS` sırasında.
pub fn aspecting_grahas(graha: Graha, positions: &Positions) -> Vec<Graha> {
    let sign = positions.sign_of(graha);
    ASPECTING_GRAHAS
        .into_iter()
        .filter(|&g| g != graha && graha_aspect_signs(g, positions.sign_of(g)).contains(&sign))
        .collect()
}

/// Graha yücelme burcunda mı ya da moolatrikona aralığında mı. Düğümler için yanlıştır.
pub fn is_garvita(graha: Graha, sign: Sign, deg_in_sign: f64) -> bool {
    let exalted = exaltation_point(graha).is_some_and(|(s, _)| s == sign);
    let in_moolatrikona = moolatrikona(graha)
        .is_some_and(|(s, from, to)| s == sign && deg_in_sign >= from && deg_in_sign < to);
    exalted || in_moolatrikona
}

/// Tekrarsız ve `Graha::ALL` sırasında.
fn in_graha_order(grahas: impl IntoIterator<Item = Graha>) -> Vec<Graha> {
    let picked: Vec<Graha> = grahas.into_iter().collect();
    Graha::ALL
        .into_iter()
        .filter(|g| picked.contains(g))
        .collect()
}

/// `graha`'nın Lajjitadi durumları, kural sırasında. Düğümler için boştur. Lagna yoksa (doğum
/// saati bilinmiyor) Lajjita değerlendirilmez; öteki beş kural lagnaya bağlı değildir.
pub fn lajjitadi(graha: Graha, positions: &Positions) -> Vec<LajjitadiHit> {
    if graha.is_node() {
        return Vec::new();
    }
    let point = positions.point(graha);
    let sign = point.rasi_sign;
    let lord = sign_lord(sign);
    let together = conjunct_grahas(graha, positions);
    let aspecting = aspecting_grahas(graha, positions);
    let relation = |other: Graha| natural_relation(graha, other);
    let pick = |from: &[Graha], wanted: Relation| -> Vec<Graha> {
        from.iter()
            .copied()
            .filter(|&g| relation(g) == wanted)
            .collect()
    };
    let enemies_together = pick(&together, Relation::Enemy);
    let enemies_aspecting = pick(&aspecting, Relation::Enemy);
    let friends_together = pick(&together, Relation::Friend);
    let friends_aspecting = pick(&aspecting, Relation::Friend);

    let mut hits = Vec::new();
    let mut hit = |state: Lajjitadi, grahas: Vec<Graha>| {
        hits.push(LajjitadiHit {
            state,
            grahas: in_graha_order(grahas),
        });
    };

    if positions.house_of(graha) == Some(LAJJITA_HOUSE) {
        let companions: Vec<Graha> = together
            .iter()
            .copied()
            .filter(|g| LAJJITA_COMPANIONS.contains(g))
            .collect();
        if !companions.is_empty() {
            hit(Lajjitadi::Lajjita, companions);
        }
    }

    if is_garvita(graha, sign, point.deg_in_sign) {
        hit(Lajjitadi::Garvita, Vec::new());
    }

    let with_saturn = together.contains(&Graha::Saturn);
    if relation(lord) == Relation::Enemy
        || !enemies_together.is_empty()
        || !enemies_aspecting.is_empty()
        || with_saturn
    {
        let mut grahas = enemies_together.clone();
        grahas.extend(&enemies_aspecting);
        if with_saturn {
            grahas.push(Graha::Saturn);
        }
        hit(Lajjitadi::Kshudhita, grahas);
    }

    let benefic_aspect = aspecting
        .iter()
        .any(|&g| nature(g, positions) == Nature::Benefic);
    if WATER_SIGNS.contains(&sign) && !enemies_aspecting.is_empty() && !benefic_aspect {
        hit(Lajjitadi::Trushita, enemies_aspecting.clone());
    }

    let with_jupiter = together.contains(&Graha::Jupiter);
    if relation(lord) == Relation::Friend
        || !friends_together.is_empty()
        || !friends_aspecting.is_empty()
        || with_jupiter
    {
        let mut grahas = friends_together;
        grahas.extend(&friends_aspecting);
        if with_jupiter {
            grahas.push(Graha::Jupiter);
        }
        hit(Lajjitadi::Mudita, grahas);
    }

    if together.contains(&Graha::Sun) {
        let pressing: Vec<Graha> = aspecting
            .iter()
            .copied()
            .filter(|&g| relation(g) == Relation::Enemy || nature(g, positions) == Nature::Malefic)
            .collect();
        if !pressing.is_empty() {
            let mut grahas = vec![Graha::Sun];
            grahas.extend(pressing);
            hit(Lajjitadi::Kshobhita, grahas);
        }
    }

    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graha::Graha;
    use crate::graha::Graha::*;
    use crate::nature::{nature, Nature};
    use crate::positions::Positions;
    use crate::zodiac::Sign;
    use crate::zodiac::Sign::*;
    use Lajjitadi::*;

    /// `Graha::ALL` sırasında (burç, burç içi derece).
    type Places = [(Sign, f64); 9];

    fn chart(lagna: Option<Sign>, places: Places) -> Positions {
        Positions::synthetic(
            lagna.map(|s| s.index() as f64 * 30.0 + 15.0),
            places.map(|(sign, deg)| sign.index() as f64 * 30.0 + deg),
        )
    }

    fn with(base: Places, changes: &[(Graha, Sign, f64)]) -> Places {
        let mut out = base;
        for &(g, sign, deg) in changes {
            out[g as usize] = (sign, deg);
        }
        out
    }

    /// Durumun kaydı: yoksa `None`, varsa kuralı sağlayan grahalar.
    fn found(graha: Graha, state: Lajjitadi, p: &Positions) -> Option<Vec<Graha>> {
        lajjitadi(graha, p)
            .into_iter()
            .find(|h| h.state == state)
            .map(|h| h.grahas)
    }

    /// Taban: Güneş İkizler'de yalnız; hiçbir graha İkizler'e bakmıyor.
    /// Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    const BASE: Places = [
        (Gemini, 5.0),
        (Taurus, 10.0),
        (Aquarius, 3.0),
        (Taurus, 20.0),
        (Aries, 10.0),
        (Aries, 15.0),
        (Aries, 12.0),
        (Virgo, 25.0),
        (Pisces, 25.0),
    ];

    #[test]
    fn conjunction_is_the_same_sign_and_aspect_is_the_full_sign_aspect() {
        let p = chart(None, BASE);
        assert_eq!(conjunct_grahas(Jupiter, &p), [Venus, Saturn]);
        assert_eq!(conjunct_grahas(Sun, &p), []);
        assert_eq!(conjunct_grahas(Ketu, &p), []);
        // Koç'taki Satürn 3. (İkizler), 7. (Terazi) ve 10. (Oğlak) burca bakar; Jüpiter 5., 7.
        // ve 9. burca; Venüs yalnız 7. burca. İkizler'e yalnız Satürn bakar.
        assert_eq!(aspecting_grahas(Sun, &p), [Saturn]);
        // Kova'daki Mars 4. burca (Boğa) bakar: Ay ve Merkür.
        assert_eq!(aspecting_grahas(Moon, &p), [Mars]);
        assert_eq!(aspecting_grahas(Mercury, &p), [Mars]);
        // Aynı burçtaki graha bakmaz; düğümler bakmaz ama bakılır.
        assert_eq!(aspecting_grahas(Jupiter, &p), []);
        let q = chart(None, with(BASE, &[(Rahu, Libra, 1.0)]));
        assert_eq!(aspecting_grahas(Rahu, &q), [Jupiter, Venus, Saturn]);
    }

    #[test]
    fn lajjita_needs_the_fifth_house_and_one_of_the_five_companions() {
        // Lagna Koç: 5. ev Aslan. Jüpiter Aslan'da Rahu ile.
        let places = with(BASE, &[(Jupiter, Leo, 10.0), (Rahu, Leo, 25.0)]);
        let p = chart(Some(Aries), places);
        assert_eq!(found(Jupiter, Lajjita, &p), Some(vec![Rahu]));
        // Birden çok eşlikçi `Graha::ALL` sırasında yazılır.
        let crowd = with(places, &[(Saturn, Leo, 1.0), (Sun, Leo, 2.0)]);
        assert_eq!(
            found(Jupiter, Lajjita, &chart(Some(Aries), crowd)),
            Some(vec![Sun, Saturn, Rahu])
        );
        // Güneş'in kendisi de 5. evde Lajjita olur; kendi kendisinin eşlikçisi değildir.
        assert_eq!(
            found(Sun, Lajjita, &chart(Some(Aries), crowd)),
            Some(vec![Saturn, Rahu])
        );
        // Ev 5 değilse yok (lagna Boğa: Aslan 4. ev).
        assert_eq!(found(Jupiter, Lajjita, &chart(Some(Taurus), places)), None);
        // Eşlikçi beş grahadan biri değilse yok (Ay, Merkür ve Venüs sayılmaz).
        let mild = with(
            BASE,
            &[
                (Jupiter, Leo, 10.0),
                (Moon, Leo, 1.0),
                (Mercury, Leo, 2.0),
                (Venus, Leo, 3.0),
            ],
        );
        assert_eq!(found(Jupiter, Lajjita, &chart(Some(Aries), mild)), None);
        // Lagna yoksa değerlendirilmez.
        assert_eq!(found(Jupiter, Lajjita, &chart(None, places)), None);
    }

    #[test]
    fn garvita_is_the_exaltation_sign_or_the_moolatrikona_range() {
        for (graha, sign, deg, want) in [
            (Sun, Aries, 29.0, true),      // yücelme burcunun tamamı
            (Sun, Leo, 10.0, true),        // moolatrikona 0–20°
            (Sun, Leo, 20.0, false),       // aralığın bitişi dışarıdadır: kendi burcu
            (Moon, Taurus, 1.0, true),     // yücelme
            (Moon, Taurus, 20.0, true),    // moolatrikona 3–30°
            (Mars, Aries, 11.9, true),     // moolatrikona 0–12°
            (Mars, Aries, 12.0, false),    // kendi burcu
            (Mars, Scorpio, 5.0, false),   // kendi burcu
            (Mercury, Virgo, 25.0, true),  // yücelme burcu (moolatrikona 15–20° dışında da)
            (Mercury, Gemini, 5.0, false), // kendi burcu
            (Jupiter, Sagittarius, 9.0, true),
            (Jupiter, Pisces, 9.0, false),
            (Venus, Libra, 14.0, true),
            (Venus, Taurus, 14.0, false),
            (Saturn, Aquarius, 10.0, true),
            (Saturn, Aquarius, 25.0, false),
            (Saturn, Libra, 1.0, true),
            (Rahu, Taurus, 1.0, false),
        ] {
            assert_eq!(
                is_garvita(graha, sign, deg),
                want,
                "{graha:?} {sign:?} {deg}"
            );
        }
        let p = chart(None, with(BASE, &[(Sun, Aries, 29.0)]));
        assert_eq!(found(Sun, Garvita, &p), Some(vec![]));
        assert_eq!(found(Sun, Garvita, &chart(None, BASE)), None);
    }

    #[test]
    fn kshudhita_is_an_enemy_sign_an_enemy_companion_an_enemy_aspect_or_saturn() {
        // Tabanda İkizler'deki Güneş'e düşmanı Satürn (Koç'tan 3. bakış) bakar.
        assert_eq!(
            found(Sun, Kshudhita, &chart(None, BASE)),
            Some(vec![Saturn])
        );
        // Satürn Boğa'ya alınınca kimse bakmaz; İkizler'in lordu Merkür nötrdür: yok.
        let calm = with(BASE, &[(Saturn, Taurus, 12.0)]);
        assert_eq!(found(Sun, Kshudhita, &chart(None, calm)), None);
        // Düşman burcu tek başına yeter: Güneş Terazi'de (lord Venüs). Koç'taki Venüs karşıdan
        // bakar; Jüpiter de bakar ama düşman değildir.
        let libra = with(calm, &[(Sun, Libra, 5.0)]);
        assert_eq!(
            found(Sun, Kshudhita, &chart(None, libra)),
            Some(vec![Venus])
        );
        // Venüs de Boğa'ya alınınca kayıt yalnız burçtandır: graha listesi boş.
        let sign_only = with(libra, &[(Venus, Taurus, 1.0)]);
        assert_eq!(found(Sun, Kshudhita, &chart(None, sign_only)), Some(vec![]));
        // Düşmanla birliktelik: Güneş İkizler'de Venüs ile.
        let together = with(calm, &[(Venus, Gemini, 9.0)]);
        assert_eq!(
            found(Sun, Kshudhita, &chart(None, together)),
            Some(vec![Venus])
        );
        // Satürn ile birliktelik, Satürn düşman olmasa da yeter: tabanda Venüs (Satürn dostu)
        // ve Jüpiter (Satürn nötr) Koç'ta Satürn iledir. Jüpiter'in düşmanı Venüs de oradadır.
        assert_eq!(
            found(Venus, Kshudhita, &chart(None, BASE)),
            Some(vec![Saturn])
        );
        assert_eq!(
            found(Jupiter, Kshudhita, &chart(None, BASE)),
            Some(vec![Venus, Saturn])
        );
        // Satürn kendi kendisinin eşlikçisi değildir: Boğa'da (lord Venüs dost) Ay ve Merkür
        // iledir; Ay düşmanıdır. Kova'daki Mars da düşmanıdır ve 4. bakışla Boğa'ya bakar.
        assert_eq!(
            found(Saturn, Kshudhita, &chart(None, calm)),
            Some(vec![Moon, Mars])
        );
        // Ay'ın doğal düşmanı yoktur: Satürn ile birlikte olmadıkça Kshudhita olmaz.
        for sign in Sign::ALL {
            let p = chart(None, with(BASE, &[(Moon, sign, 10.0)]));
            let want = (sign == Aries).then(|| vec![Saturn]);
            assert_eq!(found(Moon, Kshudhita, &p), want, "{sign:?}");
        }
    }

    #[test]
    fn trushita_needs_a_water_sign_an_enemy_aspect_and_no_benefic_aspect() {
        // Güneş Yengeç'te; Satürn Oğlak'tan karşıya bakar. İyicil bakış yok.
        let places = with(
            BASE,
            &[
                (Sun, Cancer, 5.0),
                (Saturn, Capricorn, 12.0),
                (Jupiter, Taurus, 1.0),
                (Venus, Taurus, 2.0),
            ],
        );
        assert_eq!(
            found(Sun, Trushita, &chart(None, places)),
            Some(vec![Saturn])
        );
        // Jüpiter Balık'tan 5. bakışla Yengeç'e bakarsa iyicil bakış vardır: yok.
        let helped = with(places, &[(Jupiter, Pisces, 1.0)]);
        assert_eq!(found(Sun, Trushita, &chart(None, helped)), None);
        // Bakan düşman kendisi iyicilse (Venüs) yine yok; Kshudhita ise vardır. Satürn Aslan'a
        // alınır: oradan 3., 7. ve 10. bakışı Terazi, Kova ve Boğa'ya düşer, Yengeç'e değil.
        let venus = with(places, &[(Saturn, Leo, 12.0), (Venus, Capricorn, 2.0)]);
        assert_eq!(found(Sun, Trushita, &chart(None, venus)), None);
        assert_eq!(
            found(Sun, Kshudhita, &chart(None, venus)),
            Some(vec![Venus])
        );
        // Su burcu değilse yok: Güneş Aslan'da, Satürn Kova'dan bakar.
        let fire = with(places, &[(Sun, Leo, 5.0), (Saturn, Aquarius, 12.0)]);
        assert_eq!(found(Sun, Trushita, &chart(None, fire)), None);
        // Düşman bakışı yoksa yok.
        let alone = with(places, &[(Saturn, Leo, 12.0)]);
        assert_eq!(found(Sun, Trushita, &chart(None, alone)), None);
    }

    #[test]
    fn mudita_is_a_friend_sign_a_friend_companion_a_friend_aspect_or_jupiter() {
        let calm = with(BASE, &[(Saturn, Taurus, 12.0)]);
        // İkizler'in lordu Merkür Güneş'e nötrdür; yalnız ve bakışsız: yok.
        assert_eq!(found(Sun, Mudita, &chart(None, calm)), None);
        // Dost burcu: Güneş Yengeç'te (lord Ay). Graha listesi boş.
        let cancer = with(calm, &[(Sun, Cancer, 5.0)]);
        assert_eq!(found(Sun, Mudita, &chart(None, cancer)), Some(vec![]));
        // Dostla birliktelik: Mars İkizler'e gelir.
        let together = with(calm, &[(Mars, Gemini, 3.0)]);
        assert_eq!(found(Sun, Mudita, &chart(None, together)), Some(vec![Mars]));
        // Dost bakışı: Jüpiter Yay'dan karşıya bakar.
        let aspect = with(calm, &[(Jupiter, Sagittarius, 10.0)]);
        assert_eq!(
            found(Sun, Mudita, &chart(None, aspect)),
            Some(vec![Jupiter])
        );
        // Jüpiter ile birliktelik, Jüpiter dost olmasa da yeter: tabanda Venüs Koç'ta Jüpiter
        // ile (Jüpiter Venüs'e nötr), Satürn de oradadır (dost).
        assert_eq!(
            found(Venus, Mudita, &chart(None, BASE)),
            Some(vec![Jupiter, Saturn])
        );
        let only_jupiter = with(BASE, &[(Saturn, Virgo, 12.0)]);
        assert_eq!(
            found(Venus, Mudita, &chart(None, only_jupiter)),
            Some(vec![Jupiter])
        );
        // Jüpiter de giderse Koç'ta (lord Mars nötr) yalnız Venüs için yok.
        let none = with(only_jupiter, &[(Jupiter, Virgo, 10.0)]);
        assert_eq!(found(Venus, Mudita, &chart(None, none)), None);
    }

    #[test]
    fn kshobhita_needs_the_sun_and_a_malefic_or_enemy_aspect() {
        // Merkür İkizler'de Güneş ile; Satürn (kötücül, Merkür'e nötr) Yay'dan karşıya bakar.
        let places = with(
            BASE,
            &[
                (Mercury, Gemini, 20.0),
                (Saturn, Sagittarius, 12.0),
                (Moon, Aries, 10.0),
            ],
        );
        assert_eq!(
            found(Mercury, Kshobhita, &chart(None, places)),
            Some(vec![Sun, Saturn])
        );
        // Güneş'in kendisi Kshobhita olmaz.
        assert_eq!(found(Sun, Kshobhita, &chart(None, places)), None);
        // Bakan yalnız iyicil ve düşman olmayan bir grahaysa (Jüpiter) yok.
        let mild = with(
            places,
            &[(Saturn, Taurus, 12.0), (Jupiter, Sagittarius, 10.0)],
        );
        assert_eq!(found(Mercury, Kshobhita, &chart(None, mild)), None);
        // Bakan iyicil ama düşmansa var: Venüs İkizler'de Güneş ile; Ay Yay 3°'den karşıya bakar
        // ve Venüs'ün düşmanıdır. Güneş 65°, Ay 243°: uzanım 178°, Ay büyüyor (iyicil).
        let moon = with(
            mild,
            &[
                (Venus, Gemini, 9.0),
                (Jupiter, Taurus, 1.0),
                (Moon, Sagittarius, 3.0),
            ],
        );
        let p = chart(None, moon);
        assert_eq!(nature(Moon, &p), Nature::Benefic);
        assert_eq!(found(Venus, Kshobhita, &p), Some(vec![Sun, Moon]));
        // Güneş ile birlikte değilse yok.
        let apart = with(places, &[(Mercury, Cancer, 20.0)]);
        assert_eq!(found(Mercury, Kshobhita, &chart(None, apart)), None);
    }

    #[test]
    fn mercury_alone_with_a_node_is_malefic_for_kshobhita_and_trushita() {
        // `nature` düğümleri kötücül sayar: Merkür aynı burçta yalnız Rahu ile ise kötücüldür.
        // Venüs İkizler'de Güneş ile; İkizler'e yalnız Yay'daki Merkür (Venüs'ün dostu) bakar.
        // Merkür Rahu ile: kötücül bakış, Kshobhita var. Rahu Başak'a dönünce Merkür yalnız ve
        // iyicildir: yok.
        let places = with(
            BASE,
            &[
                (Venus, Gemini, 9.0),
                (Mercury, Sagittarius, 10.0),
                (Rahu, Sagittarius, 25.0),
                (Saturn, Taurus, 12.0),
                (Jupiter, Taurus, 1.0),
            ],
        );
        let p = chart(None, places);
        assert_eq!(aspecting_grahas(Venus, &p), [Mercury]);
        assert_eq!(nature(Mercury, &p), Nature::Malefic);
        assert_eq!(found(Venus, Kshobhita, &p), Some(vec![Sun, Mercury]));
        let alone = chart(None, with(places, &[(Rahu, Virgo, 25.0)]));
        assert_eq!(nature(Mercury, &alone), Nature::Benefic);
        assert_eq!(found(Venus, Kshobhita, &alone), None);
        // Güneş Yengeç'te; düşmanı Satürn Boğa'dan 3. bakışla, Merkür Oğlak'tan karşıdan bakar.
        // Merkür Rahu ile kötücüldür: iyicil bakış yok, Trushita var. Rahu Başak'a dönünce
        // Merkür iyicildir ve bakışı Trushita'yı bozar.
        let places = with(
            BASE,
            &[
                (Sun, Cancer, 5.0),
                (Saturn, Taurus, 12.0),
                (Mercury, Capricorn, 10.0),
                (Rahu, Capricorn, 25.0),
            ],
        );
        let p = chart(None, places);
        assert_eq!(aspecting_grahas(Sun, &p), [Mercury, Saturn]);
        assert_eq!(found(Sun, Trushita, &p), Some(vec![Saturn]));
        let alone = chart(None, with(places, &[(Rahu, Virgo, 25.0)]));
        assert_eq!(found(Sun, Trushita, &alone), None);
    }

    #[test]
    fn one_graha_can_be_in_five_states_at_once_in_rule_order() {
        // Lagna Akrep: 5. ev Balık. Venüs Balık'ta (yücelme) Güneş ile; Satürn Başak'tan karşıya
        // bakar. Balık'a başka bakan yok.
        let places: Places = [
            (Pisces, 5.0),
            (Aries, 10.0),
            (Aries, 3.0),
            (Aquarius, 20.0),
            (Taurus, 10.0),
            (Pisces, 15.0),
            (Virgo, 12.0),
            (Gemini, 25.0),
            (Sagittarius, 25.0),
        ];
        let p = chart(Some(Scorpio), places);
        let hits = lajjitadi(Venus, &p);
        let states: Vec<_> = hits.iter().map(|h| h.state).collect();
        assert_eq!(states, [Lajjita, Garvita, Kshudhita, Mudita, Kshobhita]);
        let grahas: Vec<_> = hits.iter().map(|h| h.grahas.clone()).collect();
        assert_eq!(
            grahas,
            [
                vec![Sun],         // 5. evde Güneş ile
                vec![],            // yücelme burcu
                vec![Sun],         // düşmanı Güneş ile
                vec![Saturn],      // dostu Satürn'ün bakışı
                vec![Sun, Saturn]  // Güneş ile, kötücül Satürn'ün bakışı
            ]
        );
        // Trushita yok: su burcunda ama bakan (Satürn) düşman değil.
        assert_eq!(found(Venus, Trushita, &p), None);
        // Sıra her zaman kural sırasıdır ve bir durum iki kez yazılmaz.
        for graha in Graha::ALL {
            let states: Vec<_> = lajjitadi(graha, &p).iter().map(|h| h.state).collect();
            let mut sorted = states.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(states, sorted, "{graha:?}");
        }
    }

    #[test]
    fn nodes_have_no_state_and_no_list_names_the_graha_itself() {
        let p = chart(Some(Aries), BASE);
        assert_eq!(lajjitadi(Rahu, &p), []);
        assert_eq!(lajjitadi(Ketu, &p), []);
        for graha in crate::avastha::AVASTHA_GRAHAS {
            for h in lajjitadi(graha, &p) {
                assert!(!h.grahas.contains(&graha), "{graha:?} {:?}", h.state);
            }
        }
    }
}
