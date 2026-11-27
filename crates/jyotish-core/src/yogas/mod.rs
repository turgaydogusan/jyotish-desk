//! Yogalar (ileri modüller spec'i §8): veriyle tanımlı kural kataloğu. Her kural konumlardan
//! eşleşme listesi üreten saf bir yüklemdir; sonuç bir olgu listesidir (hangi yoga, hangi
//! grahalar, hangi evler). Güç derecelendirmesi ve hüküm yoktur; yalnızca D1 değerlendirilir.
//!
//! Kaynak: BPHS'nin yoga bölümleri (Pancha Mahapurusha, Ay ve Güneş yogaları, raja ve dhana
//! yogaları, nabhasa yogaları) ve klasik yoga derlemeleri; kurallar özgün olarak uygulanır.
//! Evler rasi evidir (K4), burçlar rasi çerçevesindedir (K5), doğal sınıf `nature`'dır (K6).
//! Lagnası olmayan konumlarda (doğum saati bilinmiyor) lagna isteyen kurallar değerlendirilmez.

mod lords;
mod luminaries;
mod placement;

pub use lords::{exchange_kind, ExchangeKind, DHANA_HOUSES};
pub use placement::occupied_sign_count;

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::dignity::own_signs;
use crate::drishti::{graha_aspect_signs, ASPECTING_GRAHAS};
use crate::graha::Graha;
use crate::positions::{compute_positions, ChartInput, Positions};
use crate::zodiac::Sign;
use crate::CoreError;

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
        "raja.kendra_trikona",
        YogaGroup::Raja,
        true,
        lords::kendra_trikona,
    ),
    rule(
        "raja.dharma_karmadhipati",
        YogaGroup::Raja,
        true,
        lords::dharma_karmadhipati,
    ),
    rule("raja.yogakaraka", YogaGroup::Raja, true, lords::yogakaraka),
    rule("dhana.dhana", YogaGroup::Dhana, true, lords::dhana),
    rule("exchange.maha", YogaGroup::Exchange, true, |p| {
        lords::exchange(p, ExchangeKind::Maha)
    }),
    rule("exchange.third_lord", YogaGroup::Exchange, true, |p| {
        lords::exchange(p, ExchangeKind::ThirdLord)
    }),
    rule("exchange.dainya", YogaGroup::Exchange, true, |p| {
        lords::exchange(p, ExchangeKind::Dainya)
    }),
    rule("viparita.harsha", YogaGroup::ViparitaRaja, true, |p| {
        lords::viparita(p, 6)
    }),
    rule("viparita.sarala", YogaGroup::ViparitaRaja, true, |p| {
        lords::viparita(p, 8)
    }),
    rule("viparita.vimala", YogaGroup::ViparitaRaja, true, |p| {
        lords::viparita(p, 12)
    }),
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

/// Oluşan bir yoga: bir kuralın bir eşleşmesi.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct YogaHit {
    /// Kararlı kimlik (`"mahapurusha.hamsa"`); görünen ad i18n anahtarıdır.
    pub id: String,
    pub group: YogaGroup,
    /// Yogayı oluşturan grahalar, `Graha::ALL` sırasında.
    pub grahas: Vec<Graha>,
    /// Lagnadan sayılan ilgili evler, artan sırada; doğum saati bilinmiyorsa boş.
    pub houses: Vec<u8>,
    /// İki grahanın ilişkisine dayanan kurallarda ilişki türleri; diğerlerinde boş.
    pub relations: Vec<YogaRelation>,
    /// Sağlanan alt koşul kimlikleri (`YOGA_CONDITIONS`); alt koşulu olmayan kurallarda boş.
    pub conditions: Vec<String>,
    /// Doğum saati bilinmiyor: konumlar yaklaşıktır (Ay'ın burcu gün içinde değişebilir) ve
    /// lagnaya bağlı seçenekler değerlendirilmemiştir.
    pub approximate: bool,
}

/// `compute_yogas` sonucu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct YogaReport {
    pub catalog_version: u32,
    /// Katalog sırasıyla; bir kuralın birden çok eşleşmesi graha sırasıyla.
    pub hits: Vec<YogaHit>,
    /// Değerlendirilen kural sayısı.
    pub evaluated: u32,
    /// Doğum saati bilinmediği için değerlendirilmeyen (lagna isteyen) kural sayısı.
    pub skipped_time_unknown: u32,
}

/// Saf hesap: konumlardan yoga raporu. Lagna yoksa (doğum saati bilinmiyor) lagna isteyen
/// kurallar atlanır ve sayılır; diğer kuralların eşleşmeleri `approximate` işaretiyle döner.
pub fn yogas(positions: &Positions) -> YogaReport {
    let has_lagna = positions.lagna_sign.is_some();
    let mut hits = Vec::new();
    let (mut evaluated, mut skipped) = (0, 0);
    for rule in YOGA_CATALOG {
        if rule.needs_lagna && !has_lagna {
            skipped += 1;
            continue;
        }
        evaluated += 1;
        hits.extend(rule.matches(positions).into_iter().map(|m| YogaHit {
            id: rule.id.to_owned(),
            group: rule.group,
            grahas: m.grahas,
            houses: m.houses,
            relations: m.relations,
            conditions: m.conditions.into_iter().map(str::to_owned).collect(),
            approximate: !has_lagna,
        }));
    }
    YogaReport {
        catalog_version: YOGA_CATALOG_VERSION,
        hits,
        evaluated,
        skipped_time_unknown: skipped,
    }
}

/// `compute_yogas` komutunun hesabı: temel harita, konumlar, yoga raporu. Ayar, an ve konum
/// denetimi `compute_chart`'ınkidir; `input.time_known` yanlışsa lagna isteyen kurallar atlanır.
pub fn compute_yogas<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<YogaReport, CoreError> {
    Ok(yogas(&compute_positions(eph, input)?))
}

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

    /// Koç lagna; Güneş Aslan (5), Ay Kova (11), Mars Oğlak (10), Merkür Başak (6), Jüpiter
    /// Yengeç (4), Venüs İkizler (3), Satürn Terazi (7), Rahu Boğa, Ketu Akrep. El örneği
    /// `three-exalted-mahapurushas-seven-signs` ile aynı burçlar.
    const SEVEN_SIGNS: [usize; 9] = [4, 10, 9, 5, 3, 2, 6, 1, 7];

    #[test]
    fn report_lists_hits_in_catalog_order_and_counts_the_rules() {
        let report = yogas(&chart(Some(0), SEVEN_SIGNS));
        assert_eq!(report.catalog_version, YOGA_CATALOG_VERSION);
        assert_eq!((report.evaluated, report.skipped_time_unknown), (41, 0));
        let ids: Vec<&str> = report.hits.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "mahapurusha.ruchaka",
                "mahapurusha.hamsa",
                "mahapurusha.shasha",
                "lunar.anapha",
                "lunar.shakata",
                "solar.ubhayachari",
                "raja.kendra_trikona",
                "raja.kendra_trikona",
                "viparita.harsha",
                "nabhasa_count.vallaki",
                "dosha.kuja",
            ]
        );
        let ruchaka = &report.hits[0];
        assert_eq!(ruchaka.group, YogaGroup::Mahapurusha);
        assert_eq!(ruchaka.grahas, [Mars]);
        assert_eq!(ruchaka.houses, [10]);
        assert_eq!(ruchaka.conditions, [COND_EXALTATION]);
        assert!(ruchaka.relations.is_empty());
        assert!(report.hits.iter().all(|h| !h.approximate));
        // Bir kuralın iki eşleşmesi graha sırasıyla: Güneş–Ay, sonra Mars–Jüpiter.
        assert_eq!(report.hits[6].grahas, [Sun, Moon]);
        assert_eq!(report.hits[7].grahas, [Mars, Jupiter]);
        assert_eq!(report.hits[7].relations, [YogaRelation::MutualAspect]);
    }

    #[test]
    fn without_a_lagna_seventeen_rules_are_skipped_and_the_rest_is_approximate() {
        let report = yogas(&chart(None, SEVEN_SIGNS));
        assert_eq!((report.evaluated, report.skipped_time_unknown), (24, 17));
        let ids: Vec<&str> = report.hits.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "lunar.anapha",
                "solar.ubhayachari",
                "nabhasa_count.vallaki",
                "dosha.kuja"
            ]
        );
        for h in &report.hits {
            assert!(h.approximate, "{}", h.id);
            assert!(h.houses.is_empty(), "{}", h.id);
        }
        // Kuja dosha lagnasız konumda yalnızca Ay ve Venüs referanslarını taşır.
        assert_eq!(report.hits[3].conditions, [COND_FROM_MOON, COND_FROM_VENUS]);
        // Lagna isteyen kural lagnasız konumda eşleşme vermez.
        for r in YOGA_CATALOG.iter().filter(|r| r.needs_lagna) {
            assert!(r.matches(&chart(None, SEVEN_SIGNS)).is_empty(), "{}", r.id);
        }
    }

    #[test]
    fn collector_follows_time_known_the_rasi_frame_and_the_chart_errors() {
        use crate::chart::compute_chart;
        use crate::fake::FakeEph;
        use crate::settings::{CalcSettings, ZodiacMode};

        let input = |settings: CalcSettings, time_known: bool| ChartInput {
            jd_ut: 2_451_545.0,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known,
            lagna_number: None,
        };
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        let chart = compute_chart(&mut eph, 2_451_545.0, 41.0, 29.0, &s).unwrap();
        let known = compute_yogas(&mut eph, &input(s.clone(), true)).unwrap();
        assert_eq!(known, yogas(&Positions::from(&chart)));
        assert_eq!((known.evaluated, known.skipped_time_unknown), (41, 0));
        let unknown = compute_yogas(&mut eph, &input(s.clone(), false)).unwrap();
        assert_eq!(unknown, yogas(&Positions::from(&chart).without_lagna()));
        assert_eq!((unknown.evaluated, unknown.skipped_time_unknown), (24, 17));
        assert!(unknown.hits.iter().all(|h| h.approximate));
        // FakeEph: Güneş ve Merkür tropikal 10° ve 20°, ayanamsa 24° → ikisi de sidereal Balık.
        let budha_aditya = |r: &YogaReport| r.hits.iter().any(|h| h.id == "solar.budha_aditya");
        assert!(budha_aditya(&known) && budha_aditya(&unknown));

        // Rasi çerçevesi ayarı izler (K5). Sidereal: Jüpiter 226° Akrep, lagna 46° Boğa: 7. ev
        // ama kendi burcu değil → Hamsa yok. Tropikal rasi: Jüpiter 250° Yay (kendi burcu),
        // lagna 70° İkizler: 7. ev → Hamsa.
        let hamsa = |r: &YogaReport| r.hits.iter().find(|h| h.id == "mahapurusha.hamsa").cloned();
        assert_eq!(hamsa(&known), None);
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        let report = compute_yogas(&mut eph, &input(tropical, true)).unwrap();
        let hit = hamsa(&report).expect("Hamsa in the tropical rasi frame");
        assert_eq!(hit.houses, [7]);
        assert_eq!(hit.conditions, [COND_OWN_SIGN]);

        // Hatalar temel haritanınkidir; saat bilinmese de denetlenir.
        let mut bad = input(s, false);
        bad.lat = 91.0;
        assert_eq!(
            compute_yogas(&mut eph, &bad),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0
            })
        );
        let old = CalcSettings {
            schema_version: 1,
            ..CalcSettings::default()
        };
        assert_eq!(
            compute_yogas(&mut eph, &input(old, true)),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 1,
                supported: 2
            })
        );
    }
}
