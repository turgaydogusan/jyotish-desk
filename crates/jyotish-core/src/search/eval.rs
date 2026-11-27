//! Harita aramasının hafif yolu ve değerlendirmesi (ileri modüller spec'i §12.1).
//!
//! `search_facts` bir haritanın yalnızca konumlarını ve lagnasını hesaplar: ayanamsa, dokuz
//! graha ve (doğum saati biliniyorsa) lagna. Vargalar, Vimshottari ağacı, panchang bitişleri,
//! gün doğumu ve ashtakavarga hesaplanmaz. Bir koşulun istediği türetilmiş değer (varga burcu,
//! dasha yolu, karaka, arudha, yoga) `evaluate` içinde, yalnızca o koşul varsa ve saf
//! fonksiyonlarla hesaplanır. Sonuç, tam yolun (`compute_chart_for` ve modül fonksiyonları)
//! verdiğiyle aynıdır; bunu `crosscheck` testleri sabitler.
//!
//! Evler lagnadan sayılan burç evleridir (K4); burçlar rasi çerçevesindedir (K5); nakshatra ve
//! tithi sidereal boylamdandır.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{check_moment, finite, from_sidereal, rasi_frame, raw_grahas, to_sidereal};
use crate::combustion::is_combust;
use crate::dasha::{dasha_path_at, DashaSeed};
use crate::dignity::{dignity, sign_lord, Dignity};
use crate::drishti::graha_aspect_signs;
use crate::graha::Graha;
use crate::jaimini::{arudha_padas, chara_karakas};
use crate::lordship::house_lord;
use crate::panchang::{elongation, paksha, tithi_no};
use crate::positions::{ChartInput, GrahaPoint, Positions};
use crate::prashna::number_lagna_lon;
use crate::settings::{CalcSettings, CombustionOrbs, KarakaScheme};
use crate::varga::{is_vargottama, varga_position, Varga};
use crate::yogas::yoga_rule;
use crate::zodiac::{deg_in_sign, nakshatra_pada, normalize, Sign};
use crate::CoreError;

use super::query::{date_noon_jd, ChartQuery, Condition, GrahaFlag};

/// Bir haritanın arama olguları: konumlar ve koşulların türetme için istediği girdiler.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchFacts {
    /// Doğum saati bilinmiyorsa lagnasız.
    pub positions: Positions,
    /// Rasi çerçevesindeki boylamlar, `Graha::ALL` sırasında (varga, yanma ve vargottama
    /// `compute_chart` ile aynı sayıdan hesaplanır).
    pub frame_lons: [f64; 9],
    /// Lagnanın rasi çerçevesindeki boylamı; `positions.lagna_lon` ile birlikte dolu ya da boş.
    pub lagna_frame_lon: Option<f64>,
    pub seed: DashaSeed,
    pub orbs: CombustionOrbs,
    pub karaka_scheme: KarakaScheme,
    pub co_lords: bool,
}

impl SearchFacts {
    /// Hazır konumlardan olgular (el örnekleri): rasi çerçevesi boylamı burç ve burç içi
    /// dereceden kurulur.
    pub fn from_positions(
        positions: Positions,
        birth_jd_ut: f64,
        settings: &CalcSettings,
    ) -> SearchFacts {
        let frame = |sign: Sign, deg: f64| sign.index() as f64 * 30.0 + deg;
        let frame_lons = std::array::from_fn(|i| {
            frame(
                positions.grahas[i].rasi_sign,
                positions.grahas[i].deg_in_sign,
            )
        });
        let lagna_frame_lon = match (positions.lagna_lon, positions.lagna_sign) {
            // Sentetik konumlarda çerçeve sidereal'dir; burç içi derece sidereal boylamdan gelir.
            (Some(lon), Some(sign)) => Some(frame(sign, deg_in_sign(lon))),
            _ => None,
        };
        SearchFacts {
            seed: DashaSeed {
                birth_jd_ut,
                moon_sidereal_lon: positions.point(Graha::Moon).sidereal_lon,
                year_days: settings.dasha_year,
            },
            orbs: settings.combustion_orbs,
            karaka_scheme: settings.advanced.karaka_scheme,
            co_lords: settings.advanced.jaimini_co_lords,
            positions,
            frame_lons,
            lagna_frame_lon,
        }
    }

    fn frame_lon(&self, graha: Graha) -> f64 {
        self.frame_lons[graha as usize]
    }

    /// Düğümlerde retro gösterilmez; diğerlerinde hız < 0 (v1 tanımı).
    fn retrograde(&self, graha: Graha) -> bool {
        !graha.is_node() && self.positions.point(graha).speed < 0.0
    }
}

/// Hafif yol: `input` haritasının arama olguları. Ayar, an, konum ve sayı denetimi
/// `compute_chart_for`'unkidir. `input.time_known` yanlışsa lagna hesaplanmaz; sayı
/// prashnasında (`input.lagna_number`) lagna sayıdan gelir ve ephemeris'ten ev istenmez.
pub fn search_facts<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<SearchFacts, CoreError> {
    let settings = &input.settings;
    settings.validate()?;
    check_moment(input.jd_ut, input.lat, input.lon)?;
    // Sayı, ephemeris sorulmadan önce denetlenir.
    let number_lagna = input.lagna_number.map(number_lagna_lon).transpose()?;

    let ayanamsa = eph.ayanamsa(input.jd_ut, settings.ayanamsa.sidereal_mode())?;
    let ayanamsa = finite(ayanamsa.deg, "ayanamsa")?;
    let frame = rasi_frame(settings);
    let (raw, _) = raw_grahas(eph, input.jd_ut, frame, settings.node)?;

    // (rasi çerçevesindeki boylam, sidereal boylam)
    let lagna: Option<(f64, f64)> = if !input.time_known {
        None
    } else if let Some(sidereal_lon) = number_lagna {
        let sidereal_lon = normalize(sidereal_lon);
        Some((
            normalize(from_sidereal(settings.zodiac, sidereal_lon, ayanamsa)),
            sidereal_lon,
        ))
    } else {
        let houses = eph.houses(
            input.jd_ut,
            input.lat,
            input.lon,
            settings.house_system.ephemeris_system(),
            frame,
        )?;
        let asc = finite(houses.asc, "lagna")?;
        Some((
            normalize(asc),
            normalize(to_sidereal(settings.zodiac, asc, ayanamsa)),
        ))
    };

    let grahas: [GrahaPoint; 9] = std::array::from_fn(|i| {
        let r = &raw[i];
        GrahaPoint {
            graha: r.graha,
            sidereal_lon: to_sidereal(settings.zodiac, r.lon, ayanamsa),
            rasi_sign: Sign::of(r.lon),
            deg_in_sign: deg_in_sign(r.lon),
            lat: r.lat,
            speed: r.speed,
        }
    });
    let positions = Positions {
        lagna_lon: lagna.map(|l| l.1),
        lagna_sign: lagna.map(|l| Sign::of(l.0)),
        grahas,
    };
    Ok(SearchFacts {
        seed: DashaSeed {
            birth_jd_ut: input.jd_ut,
            moon_sidereal_lon: positions.point(Graha::Moon).sidereal_lon,
            year_days: settings.dasha_year,
        },
        orbs: settings.combustion_orbs,
        karaka_scheme: settings.advanced.karaka_scheme,
        co_lords: settings.advanced.jaimini_co_lords,
        frame_lons: std::array::from_fn(|i| raw[i].lon),
        lagna_frame_lon: lagna.map(|l| l.0),
        positions,
    })
}

/// Eşleşen bir koşulun olgusu: sonuç satırındaki kısa özetin verisi. Hangi alanların dolu
/// olduğu koşulun türüne bağlıdır; arayüz türü sorgudan (`condition` sırası) bilir.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MatchFact {
    /// Koşulun `all_of` içindeki sırası (0'dan).
    pub condition: u32,
    /// "Değil" koşulu sağlandı: koşulun kendisi tutmuyor; diğer alanlar boştur.
    pub negated: bool,
    pub grahas: Vec<Graha>,
    pub sign: Option<Sign>,
    /// Lagnadan sayılan burç evi.
    pub house: Option<u8>,
    pub nakshatra: Option<u8>,
    pub pada: Option<u8>,
    /// Tek grahalı olguda grahanın retro olup olmadığı (düğümlerde hep yanlış).
    pub retrograde: bool,
    /// `GrahaState`: tutan durumlar.
    pub flags: Vec<GrahaFlag>,
    /// `GrahaState`: grahanın dignity'si (kümedeyse).
    pub dignity: Option<Dignity>,
    pub tithi: Option<u8>,
    /// `YogaPresent`: oluşan yogaların kimlikleri, sorgudaki sırayla.
    pub yogas: Vec<String>,
}

/// Bir haritanın sorguya karşı sonucu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Bütün koşullar sağlandı; koşul başına bir olgu, sorgu sırasıyla.
    Match(Vec<MatchFact>),
    NoMatch,
    /// Sorgu lagna istiyor ve haritanın doğum saati bilinmiyor: harita eşleşmez ve ayrıca
    /// sayılır.
    TimeUnknown,
}

fn placement(facts: &SearchFacts, graha: Graha) -> MatchFact {
    MatchFact {
        grahas: vec![graha],
        sign: Some(facts.positions.sign_of(graha)),
        house: facts.positions.house_of(graha),
        retrograde: facts.retrograde(graha),
        ..MatchFact::default()
    }
}

/// Koşul tutuyorsa olgusu; tutmuyorsa `None`. Lagna isteyen koşul lagnasız konumda tutmaz
/// (`evaluate` o durumu önceden `TimeUnknown` olarak ayırır).
fn holds(
    cond: &Condition,
    facts: &SearchFacts,
    now_jd_ut: f64,
) -> Result<Option<MatchFact>, CoreError> {
    let p = &facts.positions;
    Ok(match cond {
        Condition::GrahaInSign {
            graha,
            signs,
            varga,
        } => {
            let sign = varga_position(*varga, facts.frame_lon(*graha)).0;
            signs.contains(&sign).then(|| {
                if *varga == Varga::D1 {
                    placement(facts, *graha)
                } else {
                    MatchFact {
                        grahas: vec![*graha],
                        sign: Some(sign),
                        ..MatchFact::default()
                    }
                }
            })
        }
        Condition::GrahaInHouse { graha, houses } => p
            .house_of(*graha)
            .filter(|h| houses.contains(h))
            .map(|_| placement(facts, *graha)),
        Condition::GrahaInNakshatra {
            graha,
            nakshatras,
            padas,
        } => {
            let (n, pada) = nakshatra_pada(p.point(*graha).sidereal_lon);
            (nakshatras.contains(&n) && (padas.is_empty() || padas.contains(&pada))).then(|| {
                MatchFact {
                    grahas: vec![*graha],
                    nakshatra: Some(n),
                    pada: Some(pada),
                    ..MatchFact::default()
                }
            })
        }
        Condition::GrahaState {
            graha,
            flags,
            dignities,
        } => {
            let point = p.point(*graha);
            let lon = facts.frame_lon(*graha);
            let retro = point.speed < 0.0;
            let state = |flag: &GrahaFlag| match flag {
                GrahaFlag::Retrograde => facts.retrograde(*graha),
                GrahaFlag::Combust => {
                    is_combust(*graha, lon, facts.frame_lon(Graha::Sun), retro, &facts.orbs)
                }
                GrahaFlag::Vargottama => is_vargottama(lon),
            };
            let held: Vec<GrahaFlag> = flags.iter().copied().filter(|f| state(f)).collect();
            let lord_sign = p.sign_of(sign_lord(point.rasi_sign));
            let own = dignity(*graha, point.rasi_sign, point.deg_in_sign, lord_sign)
                .filter(|d| dignities.contains(d));
            (!held.is_empty() || own.is_some()).then(|| MatchFact {
                flags: held,
                dignity: own,
                ..placement(facts, *graha)
            })
        }
        Condition::LagnaInSign { signs, varga } => facts
            .lagna_frame_lon
            .map(|lon| varga_position(*varga, lon).0)
            .filter(|s| signs.contains(s))
            .map(|sign| MatchFact {
                sign: Some(sign),
                ..MatchFact::default()
            }),
        Condition::Conjunction { grahas } => {
            let sign = p.sign_of(grahas[0]);
            grahas
                .iter()
                .all(|g| p.sign_of(*g) == sign)
                .then(|| MatchFact {
                    grahas: grahas.clone(),
                    sign: Some(sign),
                    house: p.lagna_sign.map(|l| sign.house_from(l)),
                    ..MatchFact::default()
                })
        }
        Condition::GrahaAspects { from, to } => graha_aspect_signs(*from, p.sign_of(*from))
            .contains(&p.sign_of(*to))
            .then(|| MatchFact {
                grahas: vec![*from, *to],
                ..MatchFact::default()
            }),
        Condition::HouseLordInHouse { lord_of, houses } => p.lagna_sign.and_then(|lagna| {
            let lord = house_lord(lagna, *lord_of);
            p.house_of(lord)
                .filter(|h| houses.contains(h))
                .map(|_| placement(facts, lord))
        }),
        Condition::MoonPhase {
            tithis,
            paksha: wanted,
        } => {
            let tithi = tithi_no(elongation(
                p.point(Graha::Sun).sidereal_lon,
                p.point(Graha::Moon).sidereal_lon,
            ));
            (tithis.contains(&tithi) || *wanted == Some(paksha(tithi))).then(|| MatchFact {
                tithi: Some(tithi),
                ..MatchFact::default()
            })
        }
        Condition::DashaAt { date, level, lords } => {
            let at = match date {
                Some(d) => date_noon_jd(d).ok_or(CoreError::InvalidSearchInput("date"))?,
                None => now_jd_ut,
            };
            let path = dasha_path_at(&facts.seed, at, *level)?;
            // An 120 yıllık döngünün dışındaysa yol kısadır: koşul tutmaz. Doğumdan önceki an
            // da tutmaz: ilk dönem doğumdan önce başlar ama harita o gün henüz yoktur.
            (at >= facts.seed.birth_jd_ut
                && path.len() == usize::from(*level)
                && lords.contains(&path[path.len() - 1]))
            .then(|| MatchFact {
                grahas: path,
                ..MatchFact::default()
            })
        }
        Condition::CharaKaraka { role, grahas } => chara_karakas(p, facts.karaka_scheme)
            .into_iter()
            .find(|k| k.role == *role)
            .filter(|k| grahas.contains(&k.graha))
            .map(|k| MatchFact {
                grahas: vec![k.graha],
                ..MatchFact::default()
            }),
        Condition::ArudhaInSign { pada, signs } => arudha_padas(p, facts.co_lords)
            .map(|padas| padas[usize::from(*pada) - 1].sign)
            .filter(|s| signs.contains(s))
            .map(|sign| MatchFact {
                sign: Some(sign),
                ..MatchFact::default()
            }),
        Condition::YogaPresent { ids } => {
            let formed: Vec<String> = ids
                .iter()
                .filter(|id| yoga_rule(id).is_some_and(|r| !r.matches(p).is_empty()))
                .cloned()
                .collect();
            (!formed.is_empty()).then(|| MatchFact {
                yogas: formed,
                ..MatchFact::default()
            })
        }
    })
}

/// Sorguyu bir haritanın olgularına uygular. `now_jd_ut`, tarihi verilmeyen `DashaAt`
/// koşullarının anıdır (aramanın başladığı an; çağıran bir kez okur). Sorgu denetlenmiş
/// olmalıdır (`ChartQuery::validate`); denetlenmemiş sorguda alan hatası buradan da dönebilir.
pub fn evaluate(
    query: &ChartQuery,
    facts: &SearchFacts,
    now_jd_ut: f64,
) -> Result<Outcome, CoreError> {
    query.validate()?;
    if facts.positions.lagna_sign.is_none() && query.needs_lagna() {
        return Ok(Outcome::TimeUnknown);
    }
    let mut matched = Vec::with_capacity(query.all_of.len());
    for (i, c) in query.all_of.iter().enumerate() {
        let condition = i as u32;
        match (holds(&c.cond, facts, now_jd_ut)?, c.negate) {
            (Some(fact), false) => matched.push(MatchFact { condition, ..fact }),
            (None, true) => matched.push(MatchFact {
                condition,
                negated: true,
                ..MatchFact::default()
            }),
            _ => return Ok(Outcome::NoMatch),
        }
    }
    Ok(Outcome::Match(matched))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jaimini::KarakaRole;
    use crate::panchang::Paksha;
    use crate::search::query::{QueryCondition, QueryScope};

    const JD: f64 = 2_451_545.0;

    /// El haritası (sidereal; lagna Koç 5°):
    /// Güneş Koç 10° (yüceltilmiş), Ay Yengeç 5°, Mars Oğlak 28° (yüceltilmiş; Koç'tan 10. ev),
    /// Merkür Koç 20° (Güneş'ten 10°: yanık), Jüpiter Yengeç 5°, Venüs Balık 27°,
    /// Satürn Terazi 20°, Rahu İkizler 15°, Ketu Yay 15°.
    fn hand() -> SearchFacts {
        let mut p = Positions::synthetic(
            Some(5.0),
            [10.0, 95.0, 298.0, 20.0, 95.0, 357.0, 200.0, 75.0, 255.0],
        );
        p.grahas[Graha::Jupiter as usize].speed = -0.05;
        p.grahas[Graha::Rahu as usize].speed = -0.05;
        SearchFacts::from_positions(p, JD, &CalcSettings::default())
    }

    fn query(conds: Vec<(Condition, bool)>) -> ChartQuery {
        ChartQuery {
            all_of: conds
                .into_iter()
                .map(|(cond, negate)| QueryCondition { cond, negate })
                .collect(),
            scope: QueryScope::default(),
        }
    }

    fn run(cond: Condition) -> Outcome {
        evaluate(&query(vec![(cond, false)]), &hand(), JD).unwrap()
    }

    fn fact(cond: Condition) -> MatchFact {
        match run(cond.clone()) {
            Outcome::Match(mut f) => f.remove(0),
            other => panic!("{cond:?} should match, got {other:?}"),
        }
    }

    fn in_sign(graha: Graha, signs: &[Sign], varga: Varga) -> Condition {
        Condition::GrahaInSign {
            graha,
            signs: signs.to_vec(),
            varga,
        }
    }

    #[test]
    fn graha_in_sign_in_the_rasi_and_in_a_varga() {
        let f = fact(in_sign(
            Graha::Mars,
            &[Sign::Leo, Sign::Capricorn],
            Varga::D1,
        ));
        assert_eq!(f.grahas, [Graha::Mars]);
        assert_eq!((f.sign, f.house), (Some(Sign::Capricorn), Some(10)));
        assert!(!f.retrograde);
        assert_eq!(
            run(in_sign(Graha::Mars, &[Sign::Aries], Varga::D1)),
            Outcome::NoMatch
        );
        // Navamsa: Oğlak 28° dokuzuncu parçadır; toprak burçlarında sayım Oğlak'tan başlar:
        // Başak. Koç 10° dördüncü parçadır: Yengeç.
        let f = fact(in_sign(Graha::Mars, &[Sign::Virgo], Varga::D9));
        assert_eq!((f.sign, f.house), (Some(Sign::Virgo), None));
        assert_eq!(
            fact(in_sign(Graha::Sun, &[Sign::Cancer], Varga::D9)).sign,
            Some(Sign::Cancer)
        );
        assert_eq!(
            run(in_sign(Graha::Sun, &[Sign::Aries], Varga::D9)),
            Outcome::NoMatch
        );
    }

    #[test]
    fn graha_in_house_and_house_lord_in_house() {
        let house = |graha, houses: &[u8]| Condition::GrahaInHouse {
            graha,
            houses: houses.to_vec(),
        };
        assert_eq!(fact(house(Graha::Mars, &[1, 10])).house, Some(10));
        assert_eq!(fact(house(Graha::Moon, &[4])).sign, Some(Sign::Cancer));
        assert_eq!(run(house(Graha::Mars, &[9, 11])), Outcome::NoMatch);
        // Koç lagnasında 10. evin (Oğlak) lordu Satürn 7. evdedir (Terazi); 1. evin lordu Mars
        // 10. evdedir.
        let lord = |lord_of, houses: &[u8]| Condition::HouseLordInHouse {
            lord_of,
            houses: houses.to_vec(),
        };
        let f = fact(lord(10, &[7]));
        assert_eq!(
            (f.grahas.as_slice(), f.house),
            ([Graha::Saturn].as_slice(), Some(7))
        );
        assert_eq!(fact(lord(1, &[10])).grahas, [Graha::Mars]);
        assert_eq!(run(lord(10, &[1, 4, 10])), Outcome::NoMatch);
    }

    #[test]
    fn graha_in_nakshatra_with_and_without_padas() {
        let nak = |graha, nakshatras: &[u8], padas: &[u8]| Condition::GrahaInNakshatra {
            graha,
            nakshatras: nakshatras.to_vec(),
            padas: padas.to_vec(),
        };
        // Ay 95°: 95 / 13°20′ = 7,125 → 8. nakshatra (Pushya); içinde 1°40′ → 1. pada.
        let f = fact(nak(Graha::Moon, &[8], &[]));
        assert_eq!((f.nakshatra, f.pada), (Some(8), Some(1)));
        assert_eq!(fact(nak(Graha::Moon, &[7, 8], &[1, 2])).pada, Some(1));
        assert_eq!(run(nak(Graha::Moon, &[8], &[2, 3, 4])), Outcome::NoMatch);
        assert_eq!(run(nak(Graha::Moon, &[9], &[])), Outcome::NoMatch);
    }

    #[test]
    fn graha_state_flags_and_dignities() {
        let state = |graha, flags: &[GrahaFlag], dignities: &[Dignity]| Condition::GrahaState {
            graha,
            flags: flags.to_vec(),
            dignities: dignities.to_vec(),
        };
        // Jüpiter retro ve Yengeç'te yüceltilmiş.
        let f = fact(state(
            Graha::Jupiter,
            &[GrahaFlag::Retrograde, GrahaFlag::Combust],
            &[Dignity::Exalted],
        ));
        assert_eq!(f.flags, [GrahaFlag::Retrograde]);
        assert_eq!(f.dignity, Some(Dignity::Exalted));
        assert!(f.retrograde);
        // Merkür Güneş'ten 10° uzakta: yanık (varsayılan orb 14°); retro değil.
        assert_eq!(
            fact(state(Graha::Mercury, &[GrahaFlag::Combust], &[])).flags,
            [GrahaFlag::Combust]
        );
        assert_eq!(
            run(state(Graha::Mercury, &[GrahaFlag::Retrograde], &[])),
            Outcome::NoMatch
        );
        // Venüs Balık 27°: son navamsa Balık'tır (su burçlarında sayım Yengeç'ten): vargottama.
        assert_eq!(
            fact(state(Graha::Venus, &[GrahaFlag::Vargottama], &[])).flags,
            [GrahaFlag::Vargottama]
        );
        // Dignity kümesi "ya da"dır; kümede olmayan dignity eşleşmez.
        assert_eq!(
            fact(state(
                Graha::Sun,
                &[],
                &[Dignity::Debilitated, Dignity::Exalted]
            ))
            .dignity,
            Some(Dignity::Exalted)
        );
        assert_eq!(
            run(state(Graha::Sun, &[], &[Dignity::Debilitated])),
            Outcome::NoMatch
        );
        // Düğümler yanmaz ve dignity taşımaz.
        assert_eq!(
            run(state(
                Graha::Rahu,
                &[GrahaFlag::Combust],
                &[Dignity::Exalted]
            )),
            Outcome::NoMatch
        );
    }

    #[test]
    fn lagna_in_sign_in_the_rasi_and_in_a_varga() {
        let lagna = |signs: &[Sign], varga| Condition::LagnaInSign {
            signs: signs.to_vec(),
            varga,
        };
        assert_eq!(
            fact(lagna(&[Sign::Aries], Varga::D1)).sign,
            Some(Sign::Aries)
        );
        assert_eq!(run(lagna(&[Sign::Taurus], Varga::D1)), Outcome::NoMatch);
        // Koç 5°: ikinci navamsa, Boğa.
        assert_eq!(
            fact(lagna(&[Sign::Taurus], Varga::D9)).sign,
            Some(Sign::Taurus)
        );
    }

    #[test]
    fn conjunction_and_aspect() {
        let conj = |grahas: &[Graha]| Condition::Conjunction {
            grahas: grahas.to_vec(),
        };
        let f = fact(conj(&[Graha::Moon, Graha::Jupiter]));
        assert_eq!((f.sign, f.house), (Some(Sign::Cancer), Some(4)));
        assert_eq!(
            fact(conj(&[Graha::Sun, Graha::Mercury])).sign,
            Some(Sign::Aries)
        );
        assert_eq!(
            run(conj(&[Graha::Sun, Graha::Mercury, Graha::Moon])),
            Outcome::NoMatch
        );
        let aspect = |from, to| Condition::GrahaAspects { from, to };
        // Mars Oğlak'tan 4. (Koç), 7. (Yengeç) ve 8. (Aslan) burçlara bakar.
        assert_eq!(
            fact(aspect(Graha::Mars, Graha::Sun)).grahas,
            [Graha::Mars, Graha::Sun]
        );
        assert_eq!(
            fact(aspect(Graha::Mars, Graha::Moon)).grahas,
            [Graha::Mars, Graha::Moon]
        );
        // Güneş Koç'tan yalnızca Terazi'ye bakar: Satürn'e bakar, Mars'a bakmaz.
        assert!(matches!(
            run(aspect(Graha::Sun, Graha::Saturn)),
            Outcome::Match(_)
        ));
        assert_eq!(run(aspect(Graha::Sun, Graha::Mars)), Outcome::NoMatch);
        // Satürn Terazi'den 3. (Yay), 7. (Koç) ve 10. (Yengeç) burçlara bakar.
        assert!(matches!(
            run(aspect(Graha::Saturn, Graha::Ketu)),
            Outcome::Match(_)
        ));
        assert_eq!(run(aspect(Graha::Saturn, Graha::Venus)), Outcome::NoMatch);
    }

    #[test]
    fn moon_phase_by_tithi_or_paksha() {
        let phase = |tithis: &[u8], paksha| Condition::MoonPhase {
            tithis: tithis.to_vec(),
            paksha,
        };
        // Uzanım 95 − 10 = 85°: 85 / 12 = 7,08 → 8. tithi, Shukla.
        assert_eq!(fact(phase(&[8], None)).tithi, Some(8));
        assert_eq!(fact(phase(&[], Some(Paksha::Shukla))).tithi, Some(8));
        assert_eq!(fact(phase(&[1], Some(Paksha::Shukla))).tithi, Some(8));
        assert_eq!(run(phase(&[7, 9], None)), Outcome::NoMatch);
        assert_eq!(run(phase(&[], Some(Paksha::Krishna))), Outcome::NoMatch);
        assert_eq!(run(phase(&[23], Some(Paksha::Krishna))), Outcome::NoMatch);
    }

    #[test]
    fn dasha_at_a_date_and_at_the_search_moment() {
        let dasha = |date: Option<&str>, level, lords: &[Graha]| Condition::DashaAt {
            date: date.map(str::to_owned),
            level,
            lords: lords.to_vec(),
        };
        // Ay 95° Pushya'dadır: doğum mahadashası Satürn; nakshatranın 1°40′'sı geçmiştir
        // (13°20′'nın sekizde biri), 19 yılın 16,625 yılı durur. Sonraki mahadasha Merkür'dür.
        let f = fact(dasha(None, 1, &[Graha::Saturn]));
        assert_eq!(f.grahas, [Graha::Saturn]);
        assert_eq!(run(dasha(None, 1, &[Graha::Mercury])), Outcome::NoMatch);
        // 2020-01-01 doğumdan 20 yıl sonradır: Merkür mahadashası.
        assert_eq!(
            fact(dasha(Some("2020-01-01"), 1, &[Graha::Ketu, Graha::Mercury])).grahas,
            [Graha::Mercury]
        );
        // İkinci seviye: yol iki lord taşır ve koşul sonuncusuna bakar.
        let f = fact(dasha(Some("2020-01-01"), 2, &Graha::ALL));
        assert_eq!(f.grahas.len(), 2);
        assert_eq!(f.grahas[0], Graha::Mercury);
        // Doğumdan önceki an döngünün dışındadır: yol boştur, koşul tutmaz.
        assert_eq!(
            run(dasha(Some("1990-01-01"), 1, &Graha::ALL)),
            Outcome::NoMatch
        );
        // 1999-06-15 Satürn mahadashasının içindedir (dönem doğumdan 2,375 yıl önce başlar)
        // ama doğumdan öncedir: harita o gün yoktur, koşul tutmaz.
        assert_eq!(
            run(dasha(Some("1999-06-15"), 1, &Graha::ALL)),
            Outcome::NoMatch
        );
        // Aramanın anı `now_jd_ut`'dir: 20 yıl sonra Merkür.
        let q = query(vec![(dasha(None, 1, &[Graha::Mercury]), false)]);
        assert!(matches!(
            evaluate(&q, &hand(), JD + 20.0 * 365.25).unwrap(),
            Outcome::Match(_)
        ));
    }

    #[test]
    fn chara_karaka_arudha_and_yoga() {
        // Sekizli şema: etkin dereceler Güneş 10, Ay 5, Mars 28, Merkür 20, Jüpiter 5,
        // Venüs 27, Satürn 20, Rahu 30 − 15 = 15. En yüksek Mars: Atmakaraka; sonra Venüs.
        let karaka = |role, grahas: &[Graha]| Condition::CharaKaraka {
            role,
            grahas: grahas.to_vec(),
        };
        assert_eq!(
            fact(karaka(KarakaRole::Atma, &[Graha::Mars, Graha::Sun])).grahas,
            [Graha::Mars]
        );
        assert_eq!(
            fact(karaka(KarakaRole::Amatya, &[Graha::Venus])).grahas,
            [Graha::Venus]
        );
        assert_eq!(
            run(karaka(KarakaRole::Atma, &[Graha::Venus])),
            Outcome::NoMatch
        );
        // AL: lagna Koç, lordu Mars Oğlak'ta (10. burç); Oğlak'tan 10. burç Terazi'dir; Terazi
        // lagnadan 7. burçtur: istisna, 10. burcu alınır: Yengeç.
        let arudha = |pada, signs: &[Sign]| Condition::ArudhaInSign {
            pada,
            signs: signs.to_vec(),
        };
        assert_eq!(fact(arudha(1, &[Sign::Cancer])).sign, Some(Sign::Cancer));
        assert_eq!(run(arudha(1, &[Sign::Libra])), Outcome::NoMatch);
        // Yoga: Ay ve Jüpiter aynı burçta (kendra ilişkisi): Gajakesari; Mars yüceltilmiş ve
        // lagnadan kendrada: Ruchaka. Katalogda olup oluşmayan bir kural eşleşmez.
        let yoga = |ids: &[&str]| Condition::YogaPresent {
            ids: ids.iter().map(|s| (*s).to_owned()).collect(),
        };
        let formed: Vec<String> = crate::yogas::yogas(&hand().positions)
            .hits
            .into_iter()
            .map(|h| h.id)
            .collect();
        assert!(
            formed.contains(&"mahapurusha.ruchaka".to_owned()),
            "{formed:?}"
        );
        let absent = crate::yogas::YOGA_CATALOG
            .iter()
            .map(|r| r.id)
            .find(|id| !formed.iter().any(|f| f == id))
            .expect("some rule does not form");
        assert_eq!(
            fact(yoga(&[absent, "mahapurusha.ruchaka"])).yogas,
            ["mahapurusha.ruchaka"]
        );
        assert_eq!(run(yoga(&[absent])), Outcome::NoMatch);
    }

    #[test]
    fn negation_inverts_one_condition_and_all_conditions_must_hold() {
        let mars_capricorn = in_sign(Graha::Mars, &[Sign::Capricorn], Varga::D1);
        let mars_aries = in_sign(Graha::Mars, &[Sign::Aries], Varga::D1);
        let facts = hand();
        // "Değil": tutmayan koşul sağlanır ve olgusu yalnızca işaret taşır.
        let out = evaluate(
            &query(vec![
                (mars_capricorn.clone(), false),
                (mars_aries.clone(), true),
            ]),
            &facts,
            JD,
        )
        .unwrap();
        let Outcome::Match(found) = out else {
            panic!("should match");
        };
        assert_eq!(found.len(), 2);
        assert_eq!((found[0].condition, found[0].negated), (0, false));
        assert_eq!(
            found[1],
            MatchFact {
                condition: 1,
                negated: true,
                ..MatchFact::default()
            }
        );
        // Tutan koşulun "değil"i eşleşmez; koşullardan biri tutmazsa harita eşleşmez.
        for conds in [
            vec![(mars_capricorn.clone(), true)],
            vec![(mars_capricorn, false), (mars_aries, false)],
        ] {
            assert_eq!(
                evaluate(&query(conds), &facts, JD).unwrap(),
                Outcome::NoMatch
            );
        }
    }

    #[test]
    fn a_chart_without_a_birth_time_is_not_evaluated_against_lagna_conditions() {
        let mut facts = hand();
        facts.positions = facts.positions.without_lagna();
        facts.lagna_frame_lon = None;
        let lagna = Condition::LagnaInSign {
            signs: vec![Sign::Aries],
            varga: Varga::D1,
        };
        let mars = in_sign(Graha::Mars, &[Sign::Capricorn], Varga::D1);
        // Lagna koşulu, "değil" ile de, tutmayan başka bir koşulun yanında da değerlendirilemez.
        for conds in [
            vec![(lagna.clone(), false)],
            vec![(lagna.clone(), true)],
            vec![
                (in_sign(Graha::Mars, &[Sign::Aries], Varga::D1), false),
                (lagna, false),
            ],
        ] {
            assert_eq!(
                evaluate(&query(conds), &facts, JD).unwrap(),
                Outcome::TimeUnknown
            );
        }
        // Lagna istemeyen sorgu değerlendirilir; olguda ev yoktur.
        let Outcome::Match(found) = evaluate(&query(vec![(mars, false)]), &facts, JD).unwrap()
        else {
            panic!("should match");
        };
        assert_eq!(
            (found[0].sign, found[0].house),
            (Some(Sign::Capricorn), None)
        );
    }

    #[test]
    fn an_unchecked_query_is_rejected_by_field_name() {
        let bad = query(vec![(in_sign(Graha::Mars, &[], Varga::D1), false)]);
        assert_eq!(
            evaluate(&bad, &hand(), JD),
            Err(CoreError::InvalidSearchInput("signs"))
        );
    }
}
