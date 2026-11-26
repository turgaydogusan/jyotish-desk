//! Yogalar (ileri modüller spec'i §8): veriyle tanımlı kural kataloğu. Her kural konumlardan
//! eşleşme listesi üreten saf bir yüklemdir; sonuç bir olgu listesidir (hangi yoga, hangi
//! grahalar, hangi evler). Güç derecelendirmesi ve hüküm yoktur; yalnızca D1 değerlendirilir.
//!
//! Kaynak: BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana
//! yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; kurallar özgün olarak uygulanır.
//! Evler rasi evidir (K4), burçlar rasi çerçevesindedir (K5), doğal sınıf `nature`'dır (K6).
//! Lagnası olmayan konumlarda (doğum saati bilinmiyor) lagna isteyen kurallar değerlendirilmez.

mod luminaries;
mod placement;

pub use placement::occupied_sign_count;

use serde::{Deserialize, Serialize};

use crate::dignity::own_signs;
use crate::drishti::{graha_aspect_signs, ASPECTING_GRAHAS};
use crate::graha::Graha;
use crate::positions::Positions;
use crate::zodiac::Sign;

/// Kural kataloğunun sürümü: bir kural eklenince, çıkınca ya da koşulu değişince artar.
pub const YOGA_CATALOG_VERSION: u32 = 1;

/// Kendra evleri.
pub const KENDRA_HOUSES: [u8; 4] = [1, 4, 7, 10];
/// Trikona evleri.
pub const TRIKONA_HOUSES: [u8; 3] = [1, 5, 9];
/// 6., 8. ve 12. ev.
pub const DUSTHANA_HOUSES: [u8; 3] = [6, 8, 12];
/// Upachaya evleri.
pub const UPACHAYA_HOUSES: [u8; 4] = [3, 6, 10, 11];
/// Yedi graha (düğümler dışında): lord olabilenler ve nabhasa yogalarında sayılanlar.
pub const YOGA_GRAHAS: [Graha; 7] = ASPECTING_GRAHAS;
/// Ay ve Güneş yogalarında sayılan grahalar.
pub const COUNTED_GRAHAS: [Graha; 5] = [
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// Alt koşul kimlikleri (`YogaHit::conditions`). Görünen adları i18n anahtarıdır.
pub const COND_OWN_SIGN: &str = "own_sign";
pub const COND_EXALTATION: &str = "exaltation";
pub const COND_FROM_LAGNA: &str = "from_lagna";
pub const COND_FROM_MOON: &str = "from_moon";
pub const COND_FROM_VENUS: &str = "from_venus";
pub const COND_SIGN_LORD_IN_KENDRA: &str = "sign_lord_in_kendra";
pub const COND_EXALTED_GRAHA_IN_KENDRA: &str = "exalted_graha_in_kendra";
pub const COND_DISPOSITOR_ASPECT: &str = "dispositor_aspect";
/// Bütün alt koşul kimlikleri.
pub const YOGA_CONDITIONS: [&str; 8] = [
    COND_OWN_SIGN,
    COND_EXALTATION,
    COND_FROM_LAGNA,
    COND_FROM_MOON,
    COND_FROM_VENUS,
    COND_SIGN_LORD_IN_KENDRA,
    COND_EXALTED_GRAHA_IN_KENDRA,
    COND_DISPOSITOR_ASPECT,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum YogaGroup {
    Mahapurusha,
    Lunar,
    Solar,
    Raja,
    Dhana,
    Exchange,
    ViparitaRaja,
    NeechaBhanga,
    NabhasaCount,
    NabhasaAshraya,
    Dosha,
}

impl YogaGroup {
    /// Katalog sırası.
    pub const ALL: [YogaGroup; 11] = [
        YogaGroup::Mahapurusha,
        YogaGroup::Lunar,
        YogaGroup::Solar,
        YogaGroup::Raja,
        YogaGroup::Dhana,
        YogaGroup::Exchange,
        YogaGroup::ViparitaRaja,
        YogaGroup::NeechaBhanga,
        YogaGroup::NabhasaCount,
        YogaGroup::NabhasaAshraya,
        YogaGroup::Dosha,
    ];
}

/// İki grahanın ilişkisi (spec §8 "ilişkili"). v1'deki `dignity::Relation` (doğal dostluk)
/// başka bir kavramdır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum YogaRelation {
    /// Aynı burç.
    Conjunction,
    /// İki grahanın birbirine burç temelli tam bakışı.
    MutualAspect,
    /// İki grahanın birbirinin burcunda olması (parivartana).
    Exchange,
}

/// Bir kuralın tek eşleşmesi: kural kimliği ve grubu katalogdan gelir.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct YogaMatch {
    /// `Graha::ALL` sırasında, yinelemesiz.
    pub grahas: Vec<Graha>,
    /// Lagnadan sayılan evler, artan sırada, yinelemesiz; lagna yoksa boş.
    pub houses: Vec<u8>,
    pub relations: Vec<YogaRelation>,
    pub conditions: Vec<&'static str>,
}

/// Katalog satırı.
#[derive(Debug, Clone, Copy)]
pub struct YogaRule {
    /// Kararlı kimlik: `"<grup>.<ad>"`.
    pub id: &'static str,
    pub group: YogaGroup,
    /// Kural lagnasız değerlendirilemez (doğum saati bilinmiyorsa atlanır).
    pub needs_lagna: bool,
    eval: fn(&Positions) -> Vec<YogaMatch>,
}

impl YogaRule {
    /// Kuralın bu konumlardaki eşleşmeleri. Lagna isteyen kural lagnasız konumda boş döner.
    pub fn matches(&self, positions: &Positions) -> Vec<YogaMatch> {
        if self.needs_lagna && positions.lagna_sign.is_none() {
            return Vec::new();
        }
        (self.eval)(positions)
    }
}

const fn rule(
    id: &'static str,
    group: YogaGroup,
    needs_lagna: bool,
    eval: fn(&Positions) -> Vec<YogaMatch>,
) -> YogaRule {
    YogaRule {
        id,
        group,
        needs_lagna,
        eval,
    }
}

/// Kural kataloğu, spec §8 tablosunun sırasıyla. Rapor bu sırayla dolar.
pub static YOGA_CATALOG: &[YogaRule] = &[
    rule("mahapurusha.ruchaka", YogaGroup::Mahapurusha, true, |p| {
        placement::mahapurusha(p, Graha::Mars)
    }),
    rule("mahapurusha.bhadra", YogaGroup::Mahapurusha, true, |p| {
        placement::mahapurusha(p, Graha::Mercury)
    }),
    rule("mahapurusha.hamsa", YogaGroup::Mahapurusha, true, |p| {
        placement::mahapurusha(p, Graha::Jupiter)
    }),
    rule("mahapurusha.malavya", YogaGroup::Mahapurusha, true, |p| {
        placement::mahapurusha(p, Graha::Venus)
    }),
    rule("mahapurusha.shasha", YogaGroup::Mahapurusha, true, |p| {
        placement::mahapurusha(p, Graha::Saturn)
    }),
    rule("lunar.sunapha", YogaGroup::Lunar, false, |p| {
        luminaries::flank(p, Graha::Moon, luminaries::Flank::SecondOnly)
    }),
    rule("lunar.anapha", YogaGroup::Lunar, false, |p| {
        luminaries::flank(p, Graha::Moon, luminaries::Flank::TwelfthOnly)
    }),
    rule("lunar.durudhara", YogaGroup::Lunar, false, |p| {
        luminaries::flank(p, Graha::Moon, luminaries::Flank::Both)
    }),
    rule(
        "lunar.kemadruma",
        YogaGroup::Lunar,
        true,
        luminaries::kemadruma,
    ),
    rule(
        "lunar.gajakesari",
        YogaGroup::Lunar,
        false,
        luminaries::gajakesari,
    ),
    rule(
        "lunar.chandra_mangala",
        YogaGroup::Lunar,
        false,
        luminaries::chandra_mangala,
    ),
    rule("lunar.adhi", YogaGroup::Lunar, false, luminaries::adhi),
    rule("lunar.shakata", YogaGroup::Lunar, true, luminaries::shakata),
    rule("lunar.amala", YogaGroup::Lunar, false, luminaries::amala),
    rule(
        "lunar.vasumati",
        YogaGroup::Lunar,
        false,
        luminaries::vasumati,
    ),
    rule("solar.vesi", YogaGroup::Solar, false, |p| {
        luminaries::flank(p, Graha::Sun, luminaries::Flank::SecondOnly)
    }),
    rule("solar.vasi", YogaGroup::Solar, false, |p| {
        luminaries::flank(p, Graha::Sun, luminaries::Flank::TwelfthOnly)
    }),
    rule("solar.ubhayachari", YogaGroup::Solar, false, |p| {
        luminaries::flank(p, Graha::Sun, luminaries::Flank::Both)
    }),
    rule(
        "solar.budha_aditya",
        YogaGroup::Solar,
        false,
        luminaries::budha_aditya,
    ),
    rule(
        "neecha_bhanga.neecha_bhanga",
        YogaGroup::NeechaBhanga,
        false,
        placement::neecha_bhanga,
    ),
    rule(
        "nabhasa_count.vallaki",
        YogaGroup::NabhasaCount,
        false,
        |p| placement::nabhasa_count(p, 7),
    ),
    rule("nabhasa_count.dama", YogaGroup::NabhasaCount, false, |p| {
        placement::nabhasa_count(p, 6)
    }),
    rule("nabhasa_count.pasha", YogaGroup::NabhasaCount, false, |p| {
        placement::nabhasa_count(p, 5)
    }),
    rule(
        "nabhasa_count.kedara",
        YogaGroup::NabhasaCount,
        false,
        |p| placement::nabhasa_count(p, 4),
    ),
    rule("nabhasa_count.shula", YogaGroup::NabhasaCount, false, |p| {
        placement::nabhasa_count(p, 3)
    }),
    rule("nabhasa_count.yuga", YogaGroup::NabhasaCount, false, |p| {
        placement::nabhasa_count(p, 2)
    }),
    rule("nabhasa_count.gola", YogaGroup::NabhasaCount, false, |p| {
        placement::nabhasa_count(p, 1)
    }),
    rule(
        "nabhasa_ashraya.rajju",
        YogaGroup::NabhasaAshraya,
        false,
        |p| placement::nabhasa_ashraya(p, crate::jaimini::SignKind::Movable),
    ),
    rule(
        "nabhasa_ashraya.musala",
        YogaGroup::NabhasaAshraya,
        false,
        |p| placement::nabhasa_ashraya(p, crate::jaimini::SignKind::Fixed),
    ),
    rule(
        "nabhasa_ashraya.nala",
        YogaGroup::NabhasaAshraya,
        false,
        |p| placement::nabhasa_ashraya(p, crate::jaimini::SignKind::Dual),
    ),
    rule("dosha.kuja", YogaGroup::Dosha, false, placement::kuja),
];

/// Katalogdaki kural; kimlik bilinmiyorsa `None`.
pub fn yoga_rule(id: &str) -> Option<&'static YogaRule> {
    YOGA_CATALOG.iter().find(|r| r.id == id)
}

/// "Değişim" (parivartana): iki farklı graha birbirinin burcundadır.
pub fn is_exchange(a: Graha, b: Graha, positions: &Positions) -> bool {
    a != b
        && own_signs(b).contains(&positions.sign_of(a))
        && own_signs(a).contains(&positions.sign_of(b))
}

/// "Karşılıklı bakış": iki graha birbirinin burcuna tam bakışla bakar (burç temelli graha
/// drishti). Düğümler bakış atmaz.
pub fn is_mutual_aspect(a: Graha, b: Graha, positions: &Positions) -> bool {
    let (sa, sb) = (positions.sign_of(a), positions.sign_of(b));
    a != b && graha_aspect_signs(a, sa).contains(&sb) && graha_aspect_signs(b, sb).contains(&sa)
}

/// İki grahanın taşıdığı ilişkiler, `YogaRelation` sırasında. Boş liste: ilişkili değiller.
pub fn yoga_relations(a: Graha, b: Graha, positions: &Positions) -> Vec<YogaRelation> {
    let mut out = Vec::new();
    if a != b && positions.sign_of(a) == positions.sign_of(b) {
        out.push(YogaRelation::Conjunction);
    }
    if is_mutual_aspect(a, b, positions) {
        out.push(YogaRelation::MutualAspect);
    }
    if is_exchange(a, b, positions) {
        out.push(YogaRelation::Exchange);
    }
    out
}

/// Grahaların lagnadan sayılan evleri: artan sırada, yinelemesiz. Lagna yoksa boş.
pub(crate) fn occupied_houses(positions: &Positions, grahas: &[Graha]) -> Vec<u8> {
    let mut houses: Vec<u8> = grahas
        .iter()
        .filter_map(|&g| positions.house_of(g))
        .collect();
    houses.sort_unstable();
    houses.dedup();
    houses
}

/// `grahas` `Graha::ALL` sırasına dizilir ve yinelemeler atılır.
pub(crate) fn in_graha_order(mut grahas: Vec<Graha>) -> Vec<Graha> {
    grahas.sort_unstable();
    grahas.dedup();
    grahas
}

/// `graha`, `reference` burcundan sayılan `houses` evlerinden birinde mi.
pub(crate) fn in_houses_from(
    positions: &Positions,
    graha: Graha,
    reference: Sign,
    houses: &[u8],
) -> bool {
    houses.contains(&positions.house_from(graha, reference))
}

#[cfg(test)]
pub(crate) mod testkit {
    use super::*;

    /// Burç ortasında duran sentetik konumlar: `signs`, `Graha::ALL` sırasında burç dizinidir
    /// (Koç 0 … Balık 11). Ay'ın büyüyüp küçülmesi burçların sırasından çıkar.
    pub(crate) fn chart(lagna: Option<usize>, signs: [usize; 9]) -> Positions {
        Positions::synthetic(
            lagna.map(|s| s as f64 * 30.0 + 15.0),
            signs.map(|s| s as f64 * 30.0 + 15.0),
        )
    }

    /// Kuralın eşleşmeleri; kimlik katalogda yoksa test düşer.
    pub(crate) fn matches(id: &str, positions: &Positions) -> Vec<YogaMatch> {
        yoga_rule(id)
            .unwrap_or_else(|| panic!("{id} is not in the catalog"))
            .matches(positions)
    }

    pub(crate) fn forms(id: &str, positions: &Positions) -> bool {
        !matches(id, positions).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::testkit::chart;
    use super::*;
    use Graha::*;

    /// Elle yazılmış tablo (bağımsız ikinci yazım): `a` grahası `b`'den kaç burç ilerideyken
    /// (0 = aynı burç) karşılıklı tam bakış vardır. 7. burç bakışı her çiftte karşılıklıdır;
    /// özel bakışlardan yalnızca Mars'ın 4. ve Satürn'ün 10. bakışı birbirini karşılar
    /// (Satürn Mars'tan 4. burçtayken Mars da Satürn'den 10. burçtadır).
    fn hand_mutual(a: Graha, b: Graha, a_ahead_of_b: usize) -> bool {
        if a.is_node() || b.is_node() || a == b {
            return false;
        }
        matches!(
            (a, b, a_ahead_of_b),
            (_, _, 6) | (Saturn, Mars, 3) | (Mars, Saturn, 9)
        )
    }

    #[test]
    fn mutual_aspect_matches_the_hand_table_for_every_pair_and_offset() {
        for a in Graha::ALL {
            for b in Graha::ALL {
                for offset in 0..12usize {
                    let mut signs = [0usize; 9];
                    // Diğer grahalar sonucu etkilemez; `b` Yengeç'te, `a` ondan `offset` ileride.
                    signs[b as usize] = 3;
                    signs[a as usize] = 3 + offset;
                    if a == b {
                        continue;
                    }
                    let p = chart(None, signs.map(|s| s % 12));
                    assert_eq!(
                        is_mutual_aspect(a, b, &p),
                        hand_mutual(a, b, offset),
                        "{a:?} {offset} signs ahead of {b:?}"
                    );
                    assert_eq!(is_mutual_aspect(a, b, &p), is_mutual_aspect(b, a, &p));
                }
            }
        }
    }

    #[test]
    fn exchange_needs_each_graha_in_a_sign_of_the_other() {
        // Mars Boğa'da, Venüs Koç'ta: değişim. Diğerleri Yengeç'te.
        let p = chart(None, [3, 3, 1, 3, 3, 0, 3, 3, 3]);
        assert!(is_exchange(Mars, Venus, &p) && is_exchange(Venus, Mars, &p));
        assert_eq!(yoga_relations(Mars, Venus, &p), [YogaRelation::Exchange]);
        // Mars Terazi'de, Venüs Koç'ta: değişim ve karşılıklı 7. bakış birlikte.
        let p = chart(None, [3, 3, 6, 3, 3, 0, 3, 3, 3]);
        assert_eq!(
            yoga_relations(Mars, Venus, &p),
            [YogaRelation::MutualAspect, YogaRelation::Exchange]
        );
        // Satürn Koç'ta, Mars Oğlak'ta: değişim; Koç Oğlak'tan 4. burçtur (Mars'ın 4. bakışı),
        // Oğlak Koç'tan 10. burçtur (Satürn'ün 10. bakışı).
        let p = chart(None, [3, 3, 9, 3, 3, 3, 0, 3, 3]);
        assert_eq!(
            yoga_relations(Mars, Saturn, &p),
            [YogaRelation::MutualAspect, YogaRelation::Exchange]
        );
        // Yalnızca biri diğerinin burcunda: değişim değil.
        let p = chart(None, [3, 3, 1, 3, 3, 5, 3, 3, 3]);
        assert!(!is_exchange(Mars, Venus, &p));
        assert!(yoga_relations(Mars, Venus, &p).is_empty());
        // Aynı burç: birlikte. Bir graha kendisiyle ilişkili sayılmaz.
        let p = chart(None, [4, 3, 1, 4, 3, 5, 3, 3, 3]);
        assert_eq!(
            yoga_relations(Sun, Mercury, &p),
            [YogaRelation::Conjunction]
        );
        assert!(yoga_relations(Sun, Sun, &p).is_empty());
        // Düğümlerin burcu yoktur: değişim kuramazlar.
        assert!(!is_exchange(Rahu, Moon, &p));
    }

    #[test]
    fn catalog_ids_are_unique_grouped_and_prefixed_by_their_group() {
        let prefix = |g: YogaGroup| match g {
            YogaGroup::Mahapurusha => "mahapurusha.",
            YogaGroup::Lunar => "lunar.",
            YogaGroup::Solar => "solar.",
            YogaGroup::Raja => "raja.",
            YogaGroup::Dhana => "dhana.",
            YogaGroup::Exchange => "exchange.",
            YogaGroup::ViparitaRaja => "viparita.",
            YogaGroup::NeechaBhanga => "neecha_bhanga.",
            YogaGroup::NabhasaCount => "nabhasa_count.",
            YogaGroup::NabhasaAshraya => "nabhasa_ashraya.",
            YogaGroup::Dosha => "dosha.",
        };
        let mut ids: Vec<&str> = YOGA_CATALOG.iter().map(|r| r.id).collect();
        for r in YOGA_CATALOG {
            assert!(r.id.starts_with(prefix(r.group)), "{}", r.id);
            assert_eq!(yoga_rule(r.id).map(|x| x.id), Some(r.id));
        }
        // Gruplar katalogda `YogaGroup::ALL` sırasıyla ve bitişik durur.
        let groups: Vec<YogaGroup> = YOGA_CATALOG.iter().map(|r| r.group).collect();
        assert!(groups.windows(2).all(|w| w[0] <= w[1]), "{groups:?}");
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), YOGA_CATALOG.len());
        assert_eq!(yoga_rule("lunar.unknown").map(|r| r.id), None);
        assert_eq!(YOGA_CATALOG_VERSION, 1);
    }

    #[test]
    fn helpers_order_grahas_and_houses() {
        // Koç lagna; Güneş 5. evde, Ay 2. evde, Mars 5. evde.
        let p = chart(Some(0), [4, 1, 4, 0, 0, 0, 0, 0, 0]);
        assert_eq!(occupied_houses(&p, &[Mars, Moon, Sun]), [2, 5]);
        assert!(occupied_houses(&p.clone().without_lagna(), &[Mars, Moon]).is_empty());
        assert_eq!(
            in_graha_order(vec![Venus, Moon, Venus, Sun]),
            [Sun, Moon, Venus]
        );
        assert!(in_houses_from(&p, Sun, Sign::Taurus, &KENDRA_HOUSES));
        assert!(!in_houses_from(&p, Sun, Sign::Taurus, &TRIKONA_HOUSES));
    }
}
