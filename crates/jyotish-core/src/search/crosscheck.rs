//! Çapraz denetim (ileri modüller spec'i §12 "Test", asıl güvence): hafif yol (`search_facts` +
//! `evaluate`) ile tam yol (`compute_chart_for` ve modül toplayıcıları) aynı haritada her koşul
//! için aynı sonucu verir. Rastgele gök, rastgele an, rastgele ayar ve rastgele sorgu.

use ephemeris::Body;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

use crate::chart::{compute_chart_for, Chart};
use crate::dignity::Dignity;
use crate::drishti::compute_drishti;
use crate::fake::FakeEph;
use crate::graha::Graha;
use crate::houses::whole_sign_house;
use crate::jaimini::{compute_jaimini, EIGHT_KARAKA_ROLES};
use crate::lordship::house_lord;
use crate::panchang::Paksha;
use crate::positions::ChartInput;
use crate::settings::{AdvancedSettings, CalcSettings, HouseSystem, KarakaScheme, ZodiacMode};
use crate::varga::Varga;
use crate::yogas::{compute_yogas, YOGA_CATALOG};
use crate::zodiac::Sign;

use super::eval::{evaluate, search_facts, MatchFact, Outcome, SearchFacts};
use super::query::{date_noon_jd, ChartQuery, Condition, GrahaFlag, QueryCondition, QueryScope};

const JD: f64 = 2_451_545.0;
const PLANETS: [Body; 7] = [
    Body::Sun,
    Body::Moon,
    Body::Mars,
    Body::Mercury,
    Body::Jupiter,
    Body::Venus,
    Body::Saturn,
];
const DIGNITIES: [Dignity; 9] = [
    Dignity::Exalted,
    Dignity::Moolatrikona,
    Dignity::OwnSign,
    Dignity::GreatFriend,
    Dignity::Friend,
    Dignity::Neutral,
    Dignity::Enemy,
    Dignity::GreatEnemy,
    Dignity::Debilitated,
];
const DATES: [&str; 4] = ["1990-06-15", "2005-03-01", "2030-11-27", "2090-01-01"];

fn sky(lons: [f64; 7], speeds: [f64; 7], asc: f64, node: f64, ayanamsa: f64) -> FakeEph {
    let mut eph = FakeEph {
        asc,
        ayanamsa,
        ..FakeEph::default()
    };
    for ((body, lon), speed) in PLANETS.iter().zip(lons).zip(speeds) {
        eph = eph.with(*body, lon, speed);
    }
    eph.with(Body::MeanNode, node, -0.053)
}

fn graha() -> impl Strategy<Value = Graha> {
    proptest::sample::select(Graha::ALL.to_vec())
}

fn subset<T: Clone + std::fmt::Debug + 'static>(
    all: Vec<T>,
    min: usize,
) -> impl Strategy<Value = Vec<T>> {
    let max = all.len();
    proptest::sample::subsequence(all, min..=max)
}

fn houses() -> impl Strategy<Value = Vec<u8>> {
    subset((1..=12).collect(), 1)
}

fn signs() -> impl Strategy<Value = Vec<Sign>> {
    subset(Sign::ALL.to_vec(), 1)
}

fn varga() -> impl Strategy<Value = Varga> {
    proptest::sample::select(Varga::ALL.to_vec())
}

fn condition() -> impl Strategy<Value = Condition> {
    let yoga_ids: Vec<String> = YOGA_CATALOG.iter().map(|r| r.id.to_owned()).collect();
    prop_oneof![
        (graha(), signs(), varga()).prop_map(|(graha, signs, varga)| Condition::GrahaInSign {
            graha,
            signs,
            varga
        }),
        (graha(), houses()).prop_map(|(graha, houses)| Condition::GrahaInHouse { graha, houses }),
        (
            graha(),
            subset((1..=27).collect(), 1),
            subset((1..=4).collect(), 0)
        )
            .prop_map(|(graha, nakshatras, padas)| Condition::GrahaInNakshatra {
                graha,
                nakshatras,
                padas
            }),
        (
            graha(),
            subset(GrahaFlag::ALL.to_vec(), 0),
            subset(DIGNITIES.to_vec(), 0)
        )
            .prop_filter_map("needs a state", |(graha, mut flags, dignities)| {
                if graha.is_node() {
                    flags.retain(|f| *f != GrahaFlag::Retrograde);
                }
                (!flags.is_empty() || !dignities.is_empty()).then_some(Condition::GrahaState {
                    graha,
                    flags,
                    dignities,
                })
            }),
        (signs(), varga()).prop_map(|(signs, varga)| Condition::LagnaInSign { signs, varga }),
        subset(Graha::ALL.to_vec(), 2).prop_map(|grahas| Condition::Conjunction {
            grahas: grahas.into_iter().take(3).collect()
        }),
        (proptest::sample::select(Graha::ALL[..7].to_vec()), graha())
            .prop_filter_map("two grahas", |(from, to)| (from != to)
                .then_some(Condition::GrahaAspects { from, to })),
        (1u8..=12, houses())
            .prop_map(|(lord_of, houses)| Condition::HouseLordInHouse { lord_of, houses }),
        (
            subset((1..=30).collect(), 0),
            proptest::option::of(prop_oneof![Just(Paksha::Shukla), Just(Paksha::Krishna)])
        )
            .prop_filter_map("needs a phase", |(tithis, paksha)| {
                (!tithis.is_empty() || paksha.is_some())
                    .then_some(Condition::MoonPhase { tithis, paksha })
            }),
        (
            proptest::option::of(proptest::sample::select(DATES.to_vec())),
            1u8..=3,
            subset(Graha::ALL.to_vec(), 1)
        )
            .prop_map(|(date, level, lords)| Condition::DashaAt {
                date: date.map(str::to_owned),
                level,
                lords
            }),
        (
            proptest::sample::select(EIGHT_KARAKA_ROLES.to_vec()),
            subset(Graha::ALL.to_vec(), 1)
        )
            .prop_map(|(role, grahas)| Condition::CharaKaraka { role, grahas }),
        (1u8..=12, signs()).prop_map(|(pada, signs)| Condition::ArudhaInSign { pada, signs }),
        proptest::sample::subsequence(yoga_ids, 1..=4)
            .prop_map(|ids| Condition::YogaPresent { ids }),
    ]
}

fn settings() -> impl Strategy<Value = CalcSettings> {
    (
        any::<bool>(),
        prop_oneof![
            Just(HouseSystem::WholeSign),
            Just(HouseSystem::Sripati),
            Just(HouseSystem::Placidus),
        ],
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(|(tropical, house_system, eight, co_lords)| CalcSettings {
            zodiac: if tropical {
                ZodiacMode::TropicalRasiSiderealNakshatra
            } else {
                ZodiacMode::Sidereal
            },
            house_system,
            advanced: AdvancedSettings {
                karaka_scheme: if eight {
                    KarakaScheme::Eight
                } else {
                    KarakaScheme::Seven
                },
                jaimini_co_lords: co_lords,
            },
            ..CalcSettings::default()
        })
}

fn pos(chart: &Chart, graha: Graha) -> &crate::graha::GrahaPosition {
    &chart.grahas[graha as usize]
}

/// Tam yol: koşul haritada tutuyorsa beklenen olgu (`condition` 0, `negated` yanlış), tutmuyorsa
/// `None`. Her alan v1 haritasından ve modül toplayıcılarından okunur; evler lagnadan burç
/// evidir (K4).
fn full_fact(
    eph: &mut FakeEph,
    cond: &Condition,
    input: &ChartInput,
    now: f64,
) -> Option<MatchFact> {
    let chart = compute_chart_for(eph, input).unwrap();
    let lagna = input.time_known.then_some(chart.lagna.sign);
    let house = |g: Graha| lagna.map(|l| whole_sign_house(pos(&chart, g).sign, l));
    let varga_chart = |v: Varga| chart.vargas.iter().find(|c| c.varga == v).unwrap();
    let placed = |g: Graha| MatchFact {
        grahas: vec![g],
        sign: Some(pos(&chart, g).sign),
        house: house(g),
        retrograde: pos(&chart, g).retrograde && !g.is_node(),
        ..MatchFact::default()
    };
    match cond {
        Condition::GrahaInSign {
            graha,
            signs,
            varga,
        } => {
            let sign = varga_chart(*varga)
                .placements
                .iter()
                .find(|p| p.graha == *graha)
                .unwrap()
                .sign;
            signs.contains(&sign).then(|| {
                if *varga == Varga::D1 {
                    placed(*graha)
                } else {
                    MatchFact {
                        grahas: vec![*graha],
                        sign: Some(sign),
                        ..MatchFact::default()
                    }
                }
            })
        }
        Condition::GrahaInHouse { graha, houses } => house(*graha)
            .is_some_and(|h| houses.contains(&h))
            .then(|| placed(*graha)),
        Condition::GrahaInNakshatra {
            graha,
            nakshatras,
            padas,
        } => {
            let g = pos(&chart, *graha);
            (nakshatras.contains(&g.nakshatra) && (padas.is_empty() || padas.contains(&g.pada)))
                .then(|| MatchFact {
                    grahas: vec![*graha],
                    nakshatra: Some(g.nakshatra),
                    pada: Some(g.pada),
                    ..MatchFact::default()
                })
        }
        Condition::GrahaState {
            graha,
            flags,
            dignities,
        } => {
            let g = pos(&chart, *graha);
            let held: Vec<GrahaFlag> = flags
                .iter()
                .copied()
                .filter(|f| match f {
                    GrahaFlag::Retrograde => g.retrograde && !graha.is_node(),
                    GrahaFlag::Combust => g.combust,
                    GrahaFlag::Vargottama => g.vargottama,
                })
                .collect();
            let own = g.dignity.filter(|d| dignities.contains(d));
            (!held.is_empty() || own.is_some()).then(|| MatchFact {
                flags: held,
                dignity: own,
                ..placed(*graha)
            })
        }
        Condition::LagnaInSign { signs, varga } => {
            let sign = varga_chart(*varga).lagna_sign;
            (input.time_known && signs.contains(&sign)).then(|| MatchFact {
                sign: Some(sign),
                ..MatchFact::default()
            })
        }
        Condition::Conjunction { grahas } => {
            let sign = pos(&chart, grahas[0]).sign;
            grahas
                .iter()
                .all(|g| pos(&chart, *g).sign == sign)
                .then(|| MatchFact {
                    grahas: grahas.clone(),
                    sign: Some(sign),
                    house: lagna.map(|l| whole_sign_house(sign, l)),
                    ..MatchFact::default()
                })
        }
        Condition::GrahaAspects { from, to } => compute_drishti(eph, input)
            .unwrap()
            .sign_aspects
            .iter()
            .any(|a| a.graha == *from && a.occupants.contains(to))
            .then(|| MatchFact {
                grahas: vec![*from, *to],
                ..MatchFact::default()
            }),
        Condition::HouseLordInHouse { lord_of, houses } => lagna.and_then(|l| {
            let lord = house_lord(l, *lord_of);
            house(lord)
                .is_some_and(|h| houses.contains(&h))
                .then(|| placed(lord))
        }),
        Condition::MoonPhase { tithis, paksha } => {
            let t = &chart.panchang.tithi;
            (tithis.contains(&t.no) || *paksha == Some(t.paksha)).then(|| MatchFact {
                tithi: Some(t.no),
                ..MatchFact::default()
            })
        }
        Condition::DashaAt { date, level, lords } => {
            let at = date.as_deref().map_or(now, |d| date_noon_jd(d).unwrap());
            // Doğumdan önceki an dönem sayılmaz: ağacın ilk dönemi doğumdan önce başlar.
            chart
                .dasha
                .periods
                .iter()
                .find(|p| p.level == *level && p.start_jd_ut <= at && at < p.end_jd_ut)
                .filter(|p| at >= input.jd_ut && lords.contains(p.lords.last().unwrap()))
                .map(|p| MatchFact {
                    grahas: p.lords.clone(),
                    ..MatchFact::default()
                })
        }
        Condition::CharaKaraka { role, grahas } => compute_jaimini(eph, input)
            .unwrap()
            .karakas
            .iter()
            .find(|k| k.role == *role && grahas.contains(&k.graha))
            .map(|k| MatchFact {
                grahas: vec![k.graha],
                ..MatchFact::default()
            }),
        Condition::ArudhaInSign { pada, signs } => compute_jaimini(eph, input)
            .unwrap()
            .arudhas
            .map(|a| a[usize::from(*pada) - 1].sign)
            .filter(|s| signs.contains(s))
            .map(|sign| MatchFact {
                sign: Some(sign),
                ..MatchFact::default()
            }),
        Condition::YogaPresent { ids } => {
            let hits = compute_yogas(eph, input).unwrap().hits;
            let formed: Vec<String> = ids
                .iter()
                .filter(|id| hits.iter().any(|h| h.id == **id))
                .cloned()
                .collect();
            (!formed.is_empty()).then(|| MatchFact {
                yogas: formed,
                ..MatchFact::default()
            })
        }
    }
}

fn full_holds(eph: &mut FakeEph, cond: &Condition, input: &ChartInput, now: f64) -> bool {
    full_fact(eph, cond, input, now).is_some()
}

fn single(cond: &Condition, negate: bool) -> ChartQuery {
    ChartQuery {
        all_of: vec![QueryCondition {
            cond: cond.clone(),
            negate,
        }],
        scope: QueryScope::default(),
    }
}

/// Tek koşullu sorgunun iki kutbu da tam yolla aynıdır ve tutan koşulun olgusu, alan alan,
/// haritanın gösterdiğidir.
fn check_single(
    eph: &mut FakeEph,
    cond: &Condition,
    input: &ChartInput,
    facts: &SearchFacts,
    now: f64,
) -> Result<(), TestCaseError> {
    let plain = evaluate(&single(cond, false), facts, now).unwrap();
    let negated = evaluate(&single(cond, true), facts, now).unwrap();
    if !input.time_known && cond.needs_lagna() {
        prop_assert_eq!(&plain, &Outcome::TimeUnknown, "{:?}", cond);
        prop_assert_eq!(&negated, &Outcome::TimeUnknown, "{:?}", cond);
        return Ok(());
    }
    match full_fact(eph, cond, input, now) {
        Some(fact) => {
            prop_assert_eq!(&plain, &Outcome::Match(vec![fact]), "{:?}", cond);
            prop_assert_eq!(&negated, &Outcome::NoMatch, "{:?}", cond);
        }
        None => {
            prop_assert_eq!(&plain, &Outcome::NoMatch, "{:?}", cond);
            let only_negated = MatchFact {
                negated: true,
                ..MatchFact::default()
            };
            prop_assert_eq!(&negated, &Outcome::Match(vec![only_negated]), "{:?}", cond);
        }
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(192))]

    #[test]
    fn the_light_path_agrees_with_the_full_path_on_every_condition(
        lons in proptest::array::uniform7(0.0f64..360.0),
        speeds in proptest::array::uniform7(-1.0f64..1.0),
        asc in 0.0f64..360.0,
        node in 0.0f64..360.0,
        ayanamsa in 20.0f64..26.0,
        days in -20_000.0f64..20_000.0,
        now_years in -10.0f64..110.0,
        settings in settings(),
        time_known in any::<bool>(),
        number in proptest::option::weighted(0.2, 1u8..=108),
        conds in proptest::collection::vec((condition(), any::<bool>()), 1..=4),
    ) {
        // Ay hep ileri gider (panchang bitiş araması bunu ister); diğer hızlar iki yönlüdür.
        let mut speeds = speeds;
        speeds[1] = 13.2;
        let mut eph = sky(lons, speeds, asc, node, ayanamsa);
        let input = ChartInput {
            jd_ut: JD + days,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known,
            lagna_number: number,
        };
        let now = input.jd_ut + now_years * 365.25;
        let query = ChartQuery {
            all_of: conds
                .iter()
                .cloned()
                .map(|(cond, negate)| QueryCondition { cond, negate })
                .collect(),
            scope: QueryScope::default(),
        };
        prop_assert_eq!(query.validate(), Ok(()));

        let facts = search_facts(&mut eph, &input).unwrap();
        let light = evaluate(&query, &facts, now).unwrap();

        let expected: Option<bool> = if !time_known && query.needs_lagna() {
            None
        } else {
            Some(conds.iter().all(|(cond, negate)| {
                full_holds(&mut eph, cond, &input, now) != *negate
            }))
        };
        match (expected, &light) {
            (None, Outcome::TimeUnknown) | (Some(false), Outcome::NoMatch) => {}
            (Some(true), Outcome::Match(found)) => {
                prop_assert_eq!(found.len(), conds.len());
                for (i, (fact, (_, negate))) in found.iter().zip(&conds).enumerate() {
                    prop_assert_eq!(fact.condition as usize, i);
                    prop_assert_eq!(fact.negated, *negate);
                }
            }
            other => prop_assert!(false, "light and full paths disagree: {:?}", other),
        }
    }

    /// Her koşul tek başına, iki kutbuyla ve olgusunun bütün alanlarıyla denetlenir: birleşik
    /// sorgu bir koşulun yanlışını bir başka koşulun arkasında saklayabilir.
    #[test]
    fn each_condition_alone_agrees_in_both_polarities_and_reports_the_chart_facts(
        lons in proptest::array::uniform7(0.0f64..360.0),
        speeds in proptest::array::uniform7(-1.0f64..1.0),
        asc in 0.0f64..360.0,
        node in 0.0f64..360.0,
        ayanamsa in 20.0f64..26.0,
        days in -20_000.0f64..20_000.0,
        now_years in -10.0f64..110.0,
        settings in settings(),
        time_known in any::<bool>(),
        number in proptest::option::weighted(0.2, 1u8..=108),
        conds in proptest::collection::vec(condition(), 6),
    ) {
        let mut speeds = speeds;
        speeds[1] = 13.2;
        let mut eph = sky(lons, speeds, asc, node, ayanamsa);
        let input = ChartInput {
            jd_ut: JD + days,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known,
            lagna_number: number,
        };
        let now = input.jd_ut + now_years * 365.25;
        let facts = search_facts(&mut eph, &input).unwrap();
        for cond in &conds {
            check_single(&mut eph, cond, &input, &facts, now)?;
        }
    }

    /// Ayara ve Güneş'e yakınlığa bağlı olgular her haritada eksiksiz sorulur: her grahanın
    /// durumu (retro, yanık, vargottama; grahalar Güneş'in ±20° yakınında ve iki yönde, yani
    /// yanma orblarının iki yanında), her karaka rolü ve her arudha padası. Rastgele sorgu bu
    /// köşelere seyrek düşer.
    #[test]
    fn states_karakas_and_arudhas_agree_on_every_graha_role_and_pada(
        sun in 0.0f64..360.0,
        offsets in proptest::array::uniform6(-20.0f64..20.0),
        speeds in proptest::array::uniform7(-1.0f64..1.0),
        asc in 0.0f64..360.0,
        node in 0.0f64..360.0,
        ayanamsa in 20.0f64..26.0,
        settings in settings(),
        time_known in any::<bool>(),
    ) {
        let mut lons = [sun; 7];
        for (lon, offset) in lons[1..].iter_mut().zip(offsets) {
            *lon = (sun + offset).rem_euclid(360.0);
        }
        let mut speeds = speeds;
        speeds[1] = 13.2;
        let mut eph = sky(lons, speeds, asc, node, ayanamsa);
        let input = ChartInput {
            jd_ut: JD,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known,
            lagna_number: None,
        };
        let facts = search_facts(&mut eph, &input).unwrap();
        for graha in Graha::ALL {
            let mut flags = GrahaFlag::ALL.to_vec();
            if graha.is_node() {
                flags.retain(|f| *f != GrahaFlag::Retrograde);
            }
            let cond = Condition::GrahaState { graha, flags, dignities: DIGNITIES.to_vec() };
            check_single(&mut eph, &cond, &input, &facts, JD)?;
            for role in EIGHT_KARAKA_ROLES {
                let cond = Condition::CharaKaraka { role, grahas: vec![graha] };
                check_single(&mut eph, &cond, &input, &facts, JD)?;
            }
        }
        for pada in 1u8..=12 {
            for half in [&Sign::ALL[..6], &Sign::ALL[6..]] {
                let cond = Condition::ArudhaInSign { pada, signs: half.to_vec() };
                check_single(&mut eph, &cond, &input, &facts, JD)?;
            }
        }
    }

    /// Hafif yolun konumları tam yolun konumlarıyla birebir aynıdır.
    #[test]
    fn the_light_positions_are_the_chart_positions(
        lons in proptest::array::uniform7(0.0f64..360.0),
        asc in 0.0f64..360.0,
        node in 0.0f64..360.0,
        ayanamsa in 20.0f64..26.0,
        settings in settings(),
        time_known in any::<bool>(),
        number in proptest::option::weighted(0.2, 1u8..=108),
    ) {
        let speeds = [0.98, 13.2, 0.6, 1.2, -0.1, 1.1, 0.05];
        let mut eph = sky(lons, speeds, asc, node, ayanamsa);
        let input = ChartInput {
            jd_ut: JD,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known,
            lagna_number: number,
        };
        let facts = search_facts(&mut eph, &input).unwrap();
        let full = crate::positions::compute_positions(&mut eph, &input).unwrap();
        prop_assert_eq!(&facts.positions, &full);
        let chart = compute_chart_for(&mut eph, &input).unwrap();
        for g in Graha::ALL {
            prop_assert_eq!(facts.frame_lons[g as usize], pos(&chart, g).lon);
        }
        prop_assert_eq!(facts.lagna_frame_lon, time_known.then_some(chart.lagna.lon));
        prop_assert_eq!(facts.seed, chart.dasha.seed);
    }
}

/// Strateji her koşul türünü üretir (bir tür sessizce düşmesin).
#[test]
fn the_strategy_produces_every_condition_kind() {
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;
    let mut runner = TestRunner::deterministic();
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..2_000 {
        let cond = condition().new_tree(&mut runner).unwrap().current();
        let debug = format!("{cond:?}");
        let name = debug.split([' ', '{']).next().unwrap().to_owned();
        seen.insert(name);
    }
    assert_eq!(seen.len(), 13, "{seen:?}");
}
