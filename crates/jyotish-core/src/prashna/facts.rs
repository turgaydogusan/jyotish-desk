//! Soru haritasının olguları (ileri modüller spec'i §11 "Olgular"). Saf hesap: ephemeris
//! görmez, `Positions` okur (K3). Hüküm, konu çıkarımı ve evet/hayır sonucu yoktur; her alan
//! haritadan okunan bir olgudur. Evler lagnadan sayılan burç evleridir (K4); burçlar rasi
//! çerçevesindedir, nakshatra ve tithi sidereal boylamdandır (K5).

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{compute_chart_for, ChartWarning};
use crate::combustion::is_combust;
use crate::dignity::{dignity, sign_lord, Dignity};
use crate::drishti::{graha_aspect_signs, ASPECTING_GRAHAS};
use crate::graha::Graha;
use crate::jaimini::{sign_kind, SignKind};
use crate::lordship::{house_lord, house_sign};
use crate::nature::{nature, Nature};
use crate::panchang::{elongation, paksha, tithi_no, Paksha};
use crate::positions::{ChartInput, Positions};
use crate::settings::CombustionOrbs;
use crate::yogas::{is_exchange, is_mutual_aspect, yogas, YogaReport, KENDRA_HOUSES};
use crate::zodiac::{nakshatra_pada, Sign};
use crate::CoreError;

/// Yöntem kimliği (K7): Parashari motorunun olguları; ayrı bir prashna ekolü uygulanmaz.
pub const PRASHNA_METHOD: &str = "prashna.parashari_v1";

/// Burcun doğuş biçimi (BPHS, burç nitelikleri bölümü): başla doğan (shirshodaya), sırtla
/// doğan (prishthodaya), iki yönlü (ubhayodaya).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum RisingMode {
    HeadFirst,
    BackFirst,
    BothWays,
}

impl RisingMode {
    pub const ALL: [RisingMode; 3] = [
        RisingMode::HeadFirst,
        RisingMode::BackFirst,
        RisingMode::BothWays,
    ];
}

/// Başla doğan: İkizler, Aslan, Başak, Terazi, Akrep, Kova. Sırtla doğan: Koç, Boğa, Yengeç,
/// Yay, Oğlak. İki yönlü: Balık.
pub fn rising_mode(sign: Sign) -> RisingMode {
    match sign {
        Sign::Gemini | Sign::Leo | Sign::Virgo | Sign::Libra | Sign::Scorpio | Sign::Aquarius => {
            RisingMode::HeadFirst
        }
        Sign::Aries | Sign::Taurus | Sign::Cancer | Sign::Sagittarius | Sign::Capricorn => {
            RisingMode::BackFirst
        }
        Sign::Pisces => RisingMode::BothWays,
    }
}

/// Bir ev lordunun haritadaki durumu.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct LordFacts {
    pub graha: Graha,
    pub sign: Sign,
    /// Lagnadan sayılan burç evi, 1..=12.
    pub house: u8,
    pub dignity: Option<Dignity>,
    pub retrograde: bool,
    pub combust: bool,
}

/// Lagna lordu ile odak evinin lordu arasındaki bağ. Birden çoğu birlikte geçerli olabilir
/// (ör. değişim ve karşılıklı bakış); liste bu enum'un sırasındadır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum LordLink {
    /// Birlikte: aynı burçtalar.
    Conjunct,
    /// Değişim: her biri diğerinin burcunda.
    Exchange,
    /// Karşılıklı bakış: her biri diğerinin burcuna tam bakışla bakar.
    MutualAspect,
    /// Tek yönlü bakış: yalnızca lagna lordu odak lordunun burcuna bakar.
    LagnaLordAspects,
    /// Tek yönlü bakış: yalnızca odak lordu lagna lordunun burcuna bakar.
    FocusLordAspects,
}

impl LordLink {
    pub const ALL: [LordLink; 5] = [
        LordLink::Conjunct,
        LordLink::Exchange,
        LordLink::MutualAspect,
        LordLink::LagnaLordAspects,
        LordLink::FocusLordAspects,
    ];
}

/// İki lordun ilişkisi (spec §11: birlikte, karşılıklı bakış, tek yönlü bakış, değişim,
/// hiçbiri) ve birbirinden ev sayıları.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct LordRelation {
    /// İki ev aynı grahanındır (ör. odak evi 1; Koç lagnasında 8. ev): bağ aranmaz.
    pub same_graha: bool,
    /// Geçerli bağlar, `LordLink::ALL` sırasında; boş liste "hiçbiri"dir.
    pub links: Vec<LordLink>,
    /// Odak lordunun burcunun, lagna lordunun burcundan sayısı (1..=12).
    pub focus_lord_from_lagna_lord: u8,
    /// Lagna lordunun burcunun, odak lordunun burcundan sayısı (1..=12).
    pub lagna_lord_from_focus_lord: u8,
}

/// Bir evdeki graha ve doğal sınıfı (K6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Occupant {
    pub graha: Graha,
    pub nature: Nature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct HouseOccupants {
    pub house: u8,
    pub sign: Sign,
    /// `Graha::ALL` sırasında.
    pub grahas: Vec<Occupant>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct LagnaFacts {
    pub sign: Sign,
    /// Burç türü: yürük, sabit, çift doğalı.
    pub kind: SignKind,
    pub rising: RisingMode,
    pub lord: LordFacts,
    /// Dört kendra (1, 4, 7, 10), ev sırasıyla; boş evler de listededir.
    pub kendras: Vec<HouseOccupants>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct FocusFacts {
    /// Astrologun seçtiği ev, 1..=12.
    pub house: u8,
    pub sign: Sign,
    pub lord: LordFacts,
    pub relation: LordRelation,
    /// Odak evindeki grahalar, `Graha::ALL` sırasında.
    pub occupants: Vec<Occupant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MoonFacts {
    pub sign: Sign,
    /// Lagnadan sayılan burç evi; lagna yoksa `None`.
    pub house: Option<u8>,
    pub nakshatra: u8,
    pub pada: u8,
    /// 1..=30.
    pub tithi: u8,
    pub paksha: Paksha,
    /// Ay'ın burcuna tam bakışla bakan grahalar (burç temelli graha drishti),
    /// `ASPECTING_GRAHAS` sırasında.
    pub aspected_by: Vec<Graha>,
}

/// `compute_prashna_facts` sonucu. Lagnaya bağlı iki bölüm (`lagna`, `focus`) lagna yoksa
/// `None`'dır (K2); `focus` odak evi seçilmemişse de `None`'dır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PrashnaFacts {
    pub lagna: Option<LagnaFacts>,
    pub focus: Option<FocusFacts>,
    pub moon: MoonFacts,
    /// Soru haritasının yoga listesi (Alt proje 3.5'in kataloğu).
    pub yogas: YogaReport,
    /// Temel haritanın konum duyarlılığı uyarısı (`LowPrecision`).
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

fn lord_facts(
    graha: Graha,
    lagna_sign: Sign,
    positions: &Positions,
    orbs: &CombustionOrbs,
) -> LordFacts {
    let p = positions.point(graha);
    let sign = p.rasi_sign;
    let retrograde = p.speed < 0.0;
    LordFacts {
        graha,
        sign,
        house: sign.house_from(lagna_sign),
        dignity: dignity(
            graha,
            sign,
            p.deg_in_sign,
            positions.sign_of(sign_lord(sign)),
        ),
        retrograde,
        // Fark iki çerçevede de aynıdır: sidereal boylamlar yeter.
        combust: is_combust(
            graha,
            p.sidereal_lon,
            positions.point(Graha::Sun).sidereal_lon,
            retrograde,
            orbs,
        ),
    }
}

/// Lagna lordu (`lagna_lord`) ile odak lordunun (`focus_lord`) ilişkisi.
pub fn lord_relation(lagna_lord: Graha, focus_lord: Graha, positions: &Positions) -> LordRelation {
    let (a, b) = (positions.sign_of(lagna_lord), positions.sign_of(focus_lord));
    let mut links = Vec::new();
    if lagna_lord != focus_lord {
        if a == b {
            links.push(LordLink::Conjunct);
        }
        if is_exchange(lagna_lord, focus_lord, positions) {
            links.push(LordLink::Exchange);
        }
        let forward = graha_aspect_signs(lagna_lord, a).contains(&b);
        let backward = graha_aspect_signs(focus_lord, b).contains(&a);
        debug_assert_eq!(
            forward && backward,
            is_mutual_aspect(lagna_lord, focus_lord, positions)
        );
        match (forward, backward) {
            (true, true) => links.push(LordLink::MutualAspect),
            (true, false) => links.push(LordLink::LagnaLordAspects),
            (false, true) => links.push(LordLink::FocusLordAspects),
            (false, false) => {}
        }
    }
    LordRelation {
        same_graha: lagna_lord == focus_lord,
        links,
        focus_lord_from_lagna_lord: b.house_from(a),
        lagna_lord_from_focus_lord: a.house_from(b),
    }
}

fn occupants(sign: Sign, positions: &Positions) -> Vec<Occupant> {
    positions
        .occupants(sign)
        .into_iter()
        .map(|graha| Occupant {
            graha,
            nature: nature(graha, positions),
        })
        .collect()
}

/// Ay'ın olguları. Tithi ve nakshatra sidereal boylamdan, burç rasi çerçevesindendir.
pub fn moon_facts(positions: &Positions) -> MoonFacts {
    let moon = positions.point(Graha::Moon);
    let sun = positions.point(Graha::Sun);
    let (nakshatra, pada) = nakshatra_pada(moon.sidereal_lon);
    let tithi = tithi_no(elongation(sun.sidereal_lon, moon.sidereal_lon));
    MoonFacts {
        sign: moon.rasi_sign,
        house: positions.house_of(Graha::Moon),
        nakshatra,
        pada,
        tithi,
        paksha: paksha(tithi),
        aspected_by: ASPECTING_GRAHAS
            .into_iter()
            .filter(|&g| {
                g != Graha::Moon
                    && graha_aspect_signs(g, positions.sign_of(g)).contains(&moon.rasi_sign)
            })
            .collect(),
    }
}

/// Saf hesap: konumlardan prashna olguları. `focus_house` astrologun seçtiği evdir (1..=12;
/// değilse `InvalidPrashnaInput("focus_house")`); modül konu çıkarmaz. Lagna yoksa `lagna` ve
/// `focus` boştur; Ay ve yoga listesi yine verilir.
pub fn prashna_facts(
    positions: &Positions,
    orbs: &CombustionOrbs,
    focus_house: Option<u8>,
) -> Result<PrashnaFacts, CoreError> {
    if focus_house.is_some_and(|h| !(1..=12).contains(&h)) {
        return Err(CoreError::InvalidPrashnaInput("focus_house"));
    }
    let lagna = positions.lagna_sign.map(|sign| LagnaFacts {
        sign,
        kind: sign_kind(sign),
        rising: rising_mode(sign),
        lord: lord_facts(house_lord(sign, 1), sign, positions, orbs),
        kendras: KENDRA_HOUSES
            .into_iter()
            .map(|house| {
                let of_house = house_sign(sign, house);
                HouseOccupants {
                    house,
                    sign: of_house,
                    grahas: occupants(of_house, positions),
                }
            })
            .collect(),
    });
    let focus = match (&lagna, focus_house) {
        (Some(l), Some(house)) => {
            let sign = house_sign(l.sign, house);
            let lord = lord_facts(house_lord(l.sign, house), l.sign, positions, orbs);
            Some(FocusFacts {
                house,
                sign,
                relation: lord_relation(l.lord.graha, lord.graha, positions),
                lord,
                occupants: occupants(sign, positions),
            })
        }
        _ => None,
    };
    Ok(PrashnaFacts {
        lagna,
        focus,
        moon: moon_facts(positions),
        yogas: yogas(positions),
        warnings: Vec::new(),
        method: PRASHNA_METHOD.to_owned(),
    })
}

/// `compute_prashna_facts` komutunun hesabı: temel harita (sayı prashnasında sayının
/// lagnasıyla: `input.lagna_number`), konumlar, olgular. Odak evi ephemeris sorulmadan önce
/// denetlenir. Ayar, an ve konum denetimi `compute_chart`'ınkidir. `input.time_known` yanlışsa
/// lagnaya bağlı bölümler boştur.
pub fn compute_prashna_facts<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
    focus_house: Option<u8>,
) -> Result<PrashnaFacts, CoreError> {
    if focus_house.is_some_and(|h| !(1..=12).contains(&h)) {
        return Err(CoreError::InvalidPrashnaInput("focus_house"));
    }
    let chart = compute_chart_for(eph, input)?;
    let positions = Positions::from(&chart);
    let positions = if input.time_known {
        positions
    } else {
        positions.without_lagna()
    };
    let mut facts = prashna_facts(&positions, &input.settings.combustion_orbs, focus_house)?;
    facts.warnings = chart
        .meta
        .warnings
        .iter()
        .copied()
        .filter(|w| *w == ChartWarning::LowPrecision)
        .collect();
    Ok(facts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::{CalcSettings, ZodiacMode};

    /// Elle yazılmış ikinci tablo (§13 katman 3), `Sign::ALL` sırasında.
    const HAND_RISING: [RisingMode; 12] = [
        RisingMode::BackFirst, // Koç
        RisingMode::BackFirst, // Boğa
        RisingMode::HeadFirst, // İkizler
        RisingMode::BackFirst, // Yengeç
        RisingMode::HeadFirst, // Aslan
        RisingMode::HeadFirst, // Başak
        RisingMode::HeadFirst, // Terazi
        RisingMode::HeadFirst, // Akrep
        RisingMode::BackFirst, // Yay
        RisingMode::BackFirst, // Oğlak
        RisingMode::HeadFirst, // Kova
        RisingMode::BothWays,  // Balık
    ];

    fn orbs() -> CombustionOrbs {
        CombustionOrbs::default()
    }

    /// Koç lagnası; Güneş Koç 10°, Ay İkizler 5°, Mars Oğlak 10°, Merkür Koç 25°,
    /// Jüpiter Yengeç 5°, Venüs Balık 20°, Satürn Terazi 10°, Rahu Boğa, Ketu Akrep.
    fn sample() -> Positions {
        Positions::synthetic(
            Some(15.0),
            [10.0, 65.0, 280.0, 25.0, 95.0, 350.0, 190.0, 40.0, 220.0],
        )
    }

    #[test]
    fn rising_modes_match_the_hand_table() {
        for (sign, want) in Sign::ALL.into_iter().zip(HAND_RISING) {
            assert_eq!(rising_mode(sign), want, "{sign:?}");
        }
        let count = |m| HAND_RISING.iter().filter(|&&x| x == m).count();
        assert_eq!(
            RisingMode::ALL.map(count),
            [6, 5, 1],
            "six head-first, five back-first, one both ways"
        );
    }

    #[test]
    fn facts_describe_the_lagna_its_lord_the_kendras_and_the_moon() {
        let p = sample();
        let f = prashna_facts(&p, &orbs(), None).unwrap();
        assert_eq!(f.method, PRASHNA_METHOD);
        assert!(f.focus.is_none() && f.warnings.is_empty());
        assert_eq!(f.yogas, yogas(&p));
        let l = f.lagna.unwrap();
        assert_eq!(
            (l.sign, l.kind, l.rising),
            (Sign::Aries, SignKind::Movable, RisingMode::BackFirst)
        );
        assert_eq!(
            l.lord,
            LordFacts {
                graha: Graha::Mars,
                sign: Sign::Capricorn,
                house: 10,
                dignity: Some(Dignity::Exalted),
                retrograde: false,
                combust: false,
            }
        );
        let houses: Vec<u8> = l.kendras.iter().map(|k| k.house).collect();
        assert_eq!(houses, [1, 4, 7, 10]);
        let signs: Vec<Sign> = l.kendras.iter().map(|k| k.sign).collect();
        assert_eq!(
            signs,
            [Sign::Aries, Sign::Cancer, Sign::Libra, Sign::Capricorn]
        );
        // Merkür Güneş'le (kötücül) aynı burçta, iyicil yok: kötücül (K6).
        assert_eq!(
            l.kendras[0].grahas,
            [
                Occupant {
                    graha: Graha::Sun,
                    nature: Nature::Malefic
                },
                Occupant {
                    graha: Graha::Mercury,
                    nature: Nature::Malefic
                },
            ]
        );
        assert_eq!(
            l.kendras[1].grahas,
            [Occupant {
                graha: Graha::Jupiter,
                nature: Nature::Benefic
            }]
        );
        // Ay İkizler 5°: Mrigashira'nın 4. padası; uzanım 55° → 5. tithi.
        assert_eq!(
            f.moon,
            MoonFacts {
                sign: Sign::Gemini,
                house: Some(3),
                nakshatra: 5,
                pada: 4,
                tithi: 5,
                paksha: Paksha::Shukla,
                aspected_by: vec![],
            }
        );
    }

    #[test]
    fn the_focus_house_names_its_lord_and_the_relation() {
        let p = sample();
        // 7. ev Terazi; lordu Venüs Balık'ta (yücelmiş), 12. evde. Mars Oğlak'tan 4, 7, 8.
        // burçlara bakar: Koç, Yengeç, Aslan → Balık'a bakmaz. Venüs Balık'tan Başak'a bakar.
        let f = prashna_facts(&p, &orbs(), Some(7)).unwrap().focus.unwrap();
        assert_eq!((f.house, f.sign), (7, Sign::Libra));
        assert_eq!(
            (f.lord.graha, f.lord.sign, f.lord.house, f.lord.dignity),
            (Graha::Venus, Sign::Pisces, 12, Some(Dignity::Exalted))
        );
        assert_eq!(
            f.relation,
            LordRelation {
                same_graha: false,
                links: vec![],
                focus_lord_from_lagna_lord: 3,
                lagna_lord_from_focus_lord: 11,
            }
        );
        assert_eq!(
            f.occupants,
            [Occupant {
                graha: Graha::Saturn,
                nature: Nature::Malefic
            }]
        );
        // 1. ve 8. ev aynı grahanındır.
        for house in [1, 8] {
            let f = prashna_facts(&p, &orbs(), Some(house))
                .unwrap()
                .focus
                .unwrap();
            assert!(f.relation.same_graha && f.relation.links.is_empty());
            assert_eq!(
                (
                    f.relation.focus_lord_from_lagna_lord,
                    f.relation.lagna_lord_from_focus_lord
                ),
                (1, 1)
            );
        }
    }

    #[test]
    fn the_five_relations_between_two_lords() {
        // Mars (a) ile Venüs (b); diğer grahalar önemsizdir.
        let at = |mars: f64, venus: f64| {
            let p = Positions::synthetic(
                Some(15.0),
                [100.0, 65.0, mars, 130.0, 95.0, venus, 190.0, 40.0, 220.0],
            );
            lord_relation(Graha::Mars, Graha::Venus, &p).links
        };
        // Birlikte: ikisi de Yay'da.
        assert_eq!(at(250.0, 260.0), [LordLink::Conjunct]);
        // Değişim ve karşılıklı bakış: Mars Terazi'de, Venüs Koç'ta.
        assert_eq!(
            at(190.0, 10.0),
            [LordLink::Exchange, LordLink::MutualAspect]
        );
        // Yalnızca değişim: Mars Boğa'da, Venüs Koç'ta.
        assert_eq!(at(40.0, 10.0), [LordLink::Exchange]);
        // Yalnızca karşılıklı bakış: Mars İkizler'de, Venüs Yay'da.
        assert_eq!(at(70.0, 250.0), [LordLink::MutualAspect]);
        // Tek yönlü: Mars İkizler'den 4. burca (Başak) bakar; Venüs Başak'tan Balık'a bakar.
        assert_eq!(at(70.0, 160.0), [LordLink::LagnaLordAspects]);
        // Ters yön: roller değişince bakan odak lordudur.
        let p = Positions::synthetic(
            Some(15.0),
            [100.0, 65.0, 70.0, 130.0, 95.0, 160.0, 190.0, 40.0, 220.0],
        );
        assert_eq!(
            lord_relation(Graha::Venus, Graha::Mars, &p).links,
            [LordLink::FocusLordAspects]
        );
        // Hiçbiri: Mars İkizler'de, Venüs Aslan'da.
        assert!(at(70.0, 130.0).is_empty());
    }

    #[test]
    fn retrograde_and_combust_lords_are_marked() {
        let mut p = Positions::synthetic(
            Some(190.0),
            [200.0, 95.0, 280.0, 185.0, 5.0, 194.0, 340.0, 130.0, 310.0],
        );
        p.grahas[Graha::Venus as usize].speed = -0.3;
        let l = prashna_facts(&p, &orbs(), None).unwrap().lagna.unwrap();
        // Venüs Terazi 14°: moolatrikona; Güneş'e 6°, retro orbu 8° → yanık.
        assert_eq!(
            (
                l.lord.graha,
                l.lord.dignity,
                l.lord.retrograde,
                l.lord.combust
            ),
            (Graha::Venus, Some(Dignity::Moolatrikona), true, true)
        );
        // Orb ayardan gelir.
        let tight = CombustionOrbs {
            venus_retro: 5.0,
            ..orbs()
        };
        assert!(
            !prashna_facts(&p, &tight, None)
                .unwrap()
                .lagna
                .unwrap()
                .lord
                .combust
        );
    }

    #[test]
    fn without_a_lagna_only_the_moon_and_the_yogas_remain() {
        let p = sample().without_lagna();
        let f = prashna_facts(&p, &orbs(), Some(7)).unwrap();
        assert!(f.lagna.is_none() && f.focus.is_none());
        assert_eq!(f.moon.house, None);
        assert_eq!((f.moon.sign, f.moon.tithi), (Sign::Gemini, 5));
        assert_eq!(f.yogas, yogas(&p));
    }

    #[test]
    fn the_moon_takes_sign_aspects_and_a_full_moon_is_the_sixteenth_tithi() {
        // Güneş Koç 5°, Ay Terazi 5°: uzanım tam 180° → 16. tithi, Krishna (K6 sınırı).
        // Güneş karşıdan bakar; Satürn Koç'tan 7. burca, Jüpiter Kova'dan 9. burca bakar.
        let p = Positions::synthetic(
            Some(280.0),
            [5.0, 185.0, 220.0, 65.0, 310.0, 40.0, 15.0, 100.0, 280.0],
        );
        let m = moon_facts(&p);
        assert_eq!((m.tithi, m.paksha), (16, Paksha::Krishna));
        assert_eq!(m.aspected_by, [Graha::Sun, Graha::Jupiter, Graha::Saturn]);
        assert_eq!(m.house, Some(10));
    }

    #[test]
    fn a_focus_house_outside_one_to_twelve_is_rejected_by_field_name() {
        let p = sample();
        for bad in [0, 13, 255] {
            assert_eq!(
                prashna_facts(&p, &orbs(), Some(bad)),
                Err(CoreError::InvalidPrashnaInput("focus_house"))
            );
        }
    }

    fn input(settings: CalcSettings) -> ChartInput {
        ChartInput {
            jd_ut: 2_451_545.0,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known: true,
            lagna_number: None,
        }
    }

    #[test]
    fn collector_follows_the_number_the_time_and_the_rasi_frame() {
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        // FakeEph: lagna tropikal 70°, ayanamsa 24° → sidereal 46° (Boğa).
        let time = compute_prashna_facts(&mut eph, &input(s.clone()), Some(4)).unwrap();
        assert_eq!(time.lagna.as_ref().unwrap().sign, Sign::Taurus);
        assert_eq!(time.focus.as_ref().unwrap().sign, Sign::Leo);
        // Sayı 30: Yengeç 8°20′; grahalar aynı an, yalnızca lagna ve evler değişir.
        let mut numbered = input(s.clone());
        numbered.lagna_number = Some(30);
        let number = compute_prashna_facts(&mut eph, &numbered, Some(4)).unwrap();
        assert_eq!(number.lagna.as_ref().unwrap().sign, Sign::Cancer);
        assert_eq!(number.focus.as_ref().unwrap().sign, Sign::Libra);
        assert_eq!(
            (number.moon.sign, number.moon.nakshatra, number.moon.tithi),
            (time.moon.sign, time.moon.nakshatra, time.moon.tithi)
        );
        assert_ne!(number.moon.house, time.moon.house);
        // Saat bilinmiyorsa lagnaya bağlı bölümler boştur (K2).
        let mut unknown = input(s);
        unknown.time_known = false;
        let u = compute_prashna_facts(&mut eph, &unknown, Some(4)).unwrap();
        assert!(u.lagna.is_none() && u.focus.is_none() && u.moon.house.is_none());
        // Tropikal-rasi kipinde lagna burcu tropikaldir (İkizler); nakshatra sidereal.
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        let t = compute_prashna_facts(&mut eph, &input(tropical), None).unwrap();
        assert_eq!(t.lagna.unwrap().sign, Sign::Gemini);
        assert_eq!(t.moon.nakshatra, time.moon.nakshatra);
    }

    #[test]
    fn collector_checks_the_focus_house_and_the_number_before_the_chart() {
        let mut eph = FakeEph::default();
        let mut bad = input(CalcSettings::default());
        bad.lat = 91.0;
        // Odak evi ve sayı konumdan önce denetlenir: hata yalnızca alan adı taşır.
        assert_eq!(
            compute_prashna_facts(&mut eph, &bad, Some(13)),
            Err(CoreError::InvalidPrashnaInput("focus_house"))
        );
        bad.lagna_number = Some(109);
        assert_eq!(
            compute_prashna_facts(&mut eph, &bad, Some(3)),
            Err(CoreError::InvalidPrashnaInput("number"))
        );
        bad.lagna_number = None;
        assert_eq!(
            compute_prashna_facts(&mut eph, &bad, Some(3)),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0
            })
        );
    }
}
