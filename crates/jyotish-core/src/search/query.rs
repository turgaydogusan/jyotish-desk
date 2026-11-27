//! Harita aramasının sorgusu (ileri modüller spec'i §12.1): hepsi sağlanması gereken koşulların
//! listesi (`all_of`). Her koşulun değer kümesi kendi içinde "ya da"dır ve her koşul `negate`
//! alabilir; serbest VE/VEYA ağacı yoktur. Sorgu kişi verisi taşımaz; yine de loglanmaz.
//!
//! Denetim hataları yalnızca alan adı taşır (`InvalidSearchInput`); değer taşımaz.

use serde::{Deserialize, Serialize};

use crate::dignity::Dignity;
use crate::drishti::ASPECTING_GRAHAS;
use crate::graha::Graha;
use crate::jaimini::KarakaRole;
use crate::panchang::Paksha;
use crate::settings::JD_UT_RANGE;
use crate::varga::Varga;
use crate::yogas::yoga_rule;
use crate::zodiac::Sign;
use crate::CoreError;

/// Yöntem kimliği (K7): arama sayfası taşır.
pub const SEARCH_METHOD: &str = "chart_search.v1";
/// Bir sorgudaki en çok koşul.
pub const MAX_CONDITIONS: usize = 12;
/// Kapsam süzgecindeki en çok etiket.
pub const MAX_SCOPE_TAGS: usize = 16;
/// Kapsamdaki bir etiketin en çok karakter sayısı (`store::MAX_TAG_CHARS` ile aynı).
pub const MAX_SCOPE_TAG_CHARS: usize = 64;
/// Bir `YogaPresent` koşulundaki en çok yoga kimliği.
pub const MAX_YOGA_IDS: usize = 16;
/// Bir koşulun değer kümesindeki en çok öğe (en geniş küme 30 tithidir). Yinelenen değer
/// zararsızdır ama bozuk bir kayıt kümeyi sınırsız uzatamaz.
pub const MAX_SET_VALUES: usize = 32;
/// `DashaAt` koşulunun en derin seviyesi (1 maha, 2 antar, 3 pratyantar).
pub const MAX_SEARCH_DASHA_LEVEL: u8 = 3;

/// Grahanın aranabilen ikili durumları. Retro, Rahu ve Ketu için sunulmaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum GrahaFlag {
    Retrograde,
    Combust,
    Vargottama,
}

impl GrahaFlag {
    pub const ALL: [GrahaFlag; 3] = [
        GrahaFlag::Retrograde,
        GrahaFlag::Combust,
        GrahaFlag::Vargottama,
    ];
}

/// Bir koşul. Değer kümeleri (`signs`, `houses`, …) kendi içinde "ya da"dır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Condition {
    /// Graha, `varga`'da bu burçlardan birindedir.
    GrahaInSign {
        graha: Graha,
        signs: Vec<Sign>,
        varga: Varga,
    },
    /// Graha, lagnadan sayılan bu burç evlerinden birindedir (K4).
    GrahaInHouse { graha: Graha, houses: Vec<u8> },
    /// Graha bu nakshatralardan birindedir; `padas` boşsa her pada.
    GrahaInNakshatra {
        graha: Graha,
        nakshatras: Vec<u8>,
        padas: Vec<u8>,
    },
    /// Graha `flags`'teki durumlardan birindedir ya da dignity'si `dignities`'tedir.
    GrahaState {
        graha: Graha,
        flags: Vec<GrahaFlag>,
        dignities: Vec<Dignity>,
    },
    /// Lagna, `varga`'da bu burçlardan birindedir.
    LagnaInSign { signs: Vec<Sign>, varga: Varga },
    /// Grahaların hepsi aynı burçtadır (rasi).
    Conjunction { grahas: Vec<Graha> },
    /// `from`, `to`'nun burcuna burç temelli tam bakışla bakar.
    GrahaAspects { from: Graha, to: Graha },
    /// `lord_of` evinin lordu bu evlerden birindedir.
    HouseLordInHouse { lord_of: u8, houses: Vec<u8> },
    /// Tithi `tithis`'tedir ya da paksha `paksha`'dır.
    MoonPhase {
        tithis: Vec<u8>,
        paksha: Option<Paksha>,
    },
    /// `date` gününde (`YYYY-MM-DD`, 12:00 UT; boşsa aramanın başladığı an) Vimshottari'nin
    /// `level`. seviyesinin lordu `lords`'tan biridir.
    DashaAt {
        date: Option<String>,
        level: u8,
        lords: Vec<Graha>,
    },
    /// Chara karaka rolünü bu grahalardan biri taşır (haritanın karaka şemasıyla).
    CharaKaraka {
        role: KarakaRole,
        grahas: Vec<Graha>,
    },
    /// `pada` evinin arudha padası (1 = AL, 12 = UL) bu burçlardan birindedir.
    ArudhaInSign { pada: u8, signs: Vec<Sign> },
    /// Bu yogalardan en az biri oluşur (katalog kimlikleri).
    YogaPresent { ids: Vec<String> },
}

/// Koşul ve "değil" işareti.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct QueryCondition {
    pub cond: Condition,
    pub negate: bool,
}

/// Kapsamdaki harita türü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum ScopeKind {
    Natal,
    Event,
}

/// Kapsam süzgeci: hangi haritaların taranacağı.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct QueryScope {
    /// Boş değilse yalnızca bu etiketlerden en az birini taşıyan haritalar.
    pub tags: Vec<String>,
    /// `None`: her tür.
    pub kind: Option<ScopeKind>,
    /// Prashna haritaları taranmaz (varsayılan açık).
    pub exclude_prashna: bool,
}

impl Default for QueryScope {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            kind: None,
            exclude_prashna: true,
        }
    }
}

/// Harita araması sorgusu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ChartQuery {
    pub all_of: Vec<QueryCondition>,
    pub scope: QueryScope,
}

fn invalid(field: &'static str) -> CoreError {
    CoreError::InvalidSearchInput(field)
}

fn check_set<T: PartialEq>(
    values: &[T],
    field: &'static str,
    ok: impl Fn(&T) -> bool,
) -> Result<(), CoreError> {
    if values.is_empty() || values.len() > MAX_SET_VALUES || !values.iter().all(ok) {
        return Err(invalid(field));
    }
    Ok(())
}

fn distinct<T: PartialEq>(values: &[T]) -> bool {
    values
        .iter()
        .enumerate()
        .all(|(i, v)| !values[..i].contains(v))
}

/// `YYYY-MM-DD` (Gregoryen) gününün 12:00 UT anı, Jülyen günü. Biçim ya da takvim günü
/// geçersizse ya da an hesap aralığının dışındaysa `None`.
pub fn date_noon_jd(date: &str) -> Option<f64> {
    let b = date.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let digits = |s: &str| -> Option<u32> {
        if s.bytes().all(|c| c.is_ascii_digit()) {
            s.parse().ok()
        } else {
            None
        }
    };
    let year = digits(&date[0..4])?;
    let month = digits(&date[5..7])?;
    let day = digits(&date[8..10])?;
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    if day == 0 || day > days {
        return None;
    }
    let jd = ephemeris::julday_gregorian(i32::try_from(year).ok()?, month, day, 12.0);
    JD_UT_RANGE.contains(&jd).then_some(jd)
}

impl Condition {
    /// Değer aralıklarını denetler; hata alan adını taşır.
    pub fn validate(&self) -> Result<(), CoreError> {
        let house = |h: &u8| (1..=12).contains(h);
        match self {
            Condition::GrahaInSign { signs, .. } => check_set(signs, "signs", |_| true),
            Condition::GrahaInHouse { houses, .. } => check_set(houses, "houses", house),
            Condition::GrahaInNakshatra {
                nakshatras, padas, ..
            } => {
                check_set(nakshatras, "nakshatras", |n| (1..=27).contains(n))?;
                if padas.len() > MAX_SET_VALUES || padas.iter().any(|p| !(1..=4).contains(p)) {
                    return Err(invalid("padas"));
                }
                Ok(())
            }
            Condition::GrahaState {
                graha,
                flags,
                dignities,
            } => {
                let empty = flags.is_empty() && dignities.is_empty();
                // Retro, Rahu ve Ketu için sunulmaz (UI spec §2.3).
                let node_retro = graha.is_node() && flags.contains(&GrahaFlag::Retrograde);
                let long = flags.len().max(dignities.len()) > MAX_SET_VALUES;
                if empty || long || node_retro {
                    return Err(invalid("states"));
                }
                Ok(())
            }
            Condition::LagnaInSign { signs, .. } => check_set(signs, "signs", |_| true),
            Condition::Conjunction { grahas } => {
                if grahas.len() < 2 || !distinct(grahas) {
                    return Err(invalid("grahas"));
                }
                Ok(())
            }
            Condition::GrahaAspects { from, to } => {
                // Düğümlere graha bakışı atanmaz (spec §4).
                if from == to || !ASPECTING_GRAHAS.contains(from) {
                    return Err(invalid("grahas"));
                }
                Ok(())
            }
            Condition::HouseLordInHouse { lord_of, houses } => {
                if !house(lord_of) {
                    return Err(invalid("lord_of"));
                }
                check_set(houses, "houses", house)
            }
            Condition::MoonPhase { tithis, paksha } => {
                let empty = tithis.is_empty() && paksha.is_none();
                let long = tithis.len() > MAX_SET_VALUES;
                if empty || long || tithis.iter().any(|t| !(1..=30).contains(t)) {
                    return Err(invalid("tithis"));
                }
                Ok(())
            }
            Condition::DashaAt { date, level, lords } => {
                if !(1..=MAX_SEARCH_DASHA_LEVEL).contains(level) {
                    return Err(invalid("level"));
                }
                check_set(lords, "lords", |_| true)?;
                if let Some(d) = date {
                    date_noon_jd(d).ok_or(invalid("date"))?;
                }
                Ok(())
            }
            Condition::CharaKaraka { grahas, .. } => check_set(grahas, "grahas", |_| true),
            Condition::ArudhaInSign { pada, signs } => {
                if !house(pada) {
                    return Err(invalid("pada"));
                }
                check_set(signs, "signs", |_| true)
            }
            Condition::YogaPresent { ids } => {
                if ids.len() > MAX_YOGA_IDS {
                    return Err(invalid("yogas"));
                }
                check_set(ids, "yogas", |id| yoga_rule(id).is_some())
            }
        }
    }

    /// Koşul lagnasız değerlendirilemez (doğum saati bilinmeyen haritada).
    pub fn needs_lagna(&self) -> bool {
        match self {
            Condition::GrahaInHouse { .. }
            | Condition::LagnaInSign { .. }
            | Condition::HouseLordInHouse { .. }
            | Condition::ArudhaInSign { .. } => true,
            Condition::YogaPresent { ids } => ids
                .iter()
                .any(|id| yoga_rule(id).is_some_and(|r| r.needs_lagna)),
            _ => false,
        }
    }
}

impl QueryScope {
    pub fn validate(&self) -> Result<(), CoreError> {
        let ok = |t: &String| {
            let n = t.trim().chars().count();
            (1..=MAX_SCOPE_TAG_CHARS).contains(&n)
        };
        if self.tags.len() > MAX_SCOPE_TAGS || !self.tags.iter().all(ok) {
            return Err(invalid("tags"));
        }
        Ok(())
    }
}

impl ChartQuery {
    /// Sorgunun tamamını denetler: en az bir, en çok `MAX_CONDITIONS` koşul.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.all_of.is_empty() || self.all_of.len() > MAX_CONDITIONS {
            return Err(invalid("conditions"));
        }
        for c in &self.all_of {
            c.cond.validate()?;
        }
        self.scope.validate()
    }

    /// En az bir koşul lagna ister: doğum saati bilinmeyen harita değerlendirilemez.
    pub fn needs_lagna(&self) -> bool {
        self.all_of.iter().any(|c| c.cond.needs_lagna())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(r: Result<(), CoreError>) -> &'static str {
        match r {
            Err(CoreError::InvalidSearchInput(f)) => f,
            other => panic!("expected an invalid search input, got {other:?}"),
        }
    }

    fn one(cond: Condition) -> ChartQuery {
        ChartQuery {
            all_of: vec![QueryCondition {
                cond,
                negate: false,
            }],
            scope: QueryScope::default(),
        }
    }

    fn mars_in(signs: Vec<Sign>) -> Condition {
        Condition::GrahaInSign {
            graha: Graha::Mars,
            signs,
            varga: Varga::D1,
        }
    }

    #[test]
    fn a_query_needs_one_to_twelve_conditions() {
        let empty = ChartQuery {
            all_of: Vec::new(),
            scope: QueryScope::default(),
        };
        assert_eq!(field(empty.validate()), "conditions");
        let mut q = one(mars_in(vec![Sign::Aries]));
        assert_eq!(q.validate(), Ok(()));
        while q.all_of.len() < MAX_CONDITIONS {
            q.all_of.push(q.all_of[0].clone());
        }
        assert_eq!(q.validate(), Ok(()));
        q.all_of.push(q.all_of[0].clone());
        assert_eq!(field(q.validate()), "conditions");
    }

    #[test]
    fn every_condition_names_its_bad_field_and_never_a_value() {
        let cases: Vec<(Condition, &str)> = vec![
            (mars_in(Vec::new()), "signs"),
            (
                Condition::GrahaInHouse {
                    graha: Graha::Sun,
                    houses: vec![13],
                },
                "houses",
            ),
            (
                Condition::GrahaInHouse {
                    graha: Graha::Sun,
                    houses: Vec::new(),
                },
                "houses",
            ),
            (
                Condition::GrahaInNakshatra {
                    graha: Graha::Moon,
                    nakshatras: vec![28],
                    padas: Vec::new(),
                },
                "nakshatras",
            ),
            (
                Condition::GrahaInNakshatra {
                    graha: Graha::Moon,
                    nakshatras: vec![4],
                    padas: vec![5],
                },
                "padas",
            ),
            (
                Condition::GrahaState {
                    graha: Graha::Mars,
                    flags: Vec::new(),
                    dignities: Vec::new(),
                },
                "states",
            ),
            (
                Condition::GrahaState {
                    graha: Graha::Rahu,
                    flags: vec![GrahaFlag::Retrograde],
                    dignities: Vec::new(),
                },
                "states",
            ),
            (
                Condition::LagnaInSign {
                    signs: Vec::new(),
                    varga: Varga::D9,
                },
                "signs",
            ),
            (
                Condition::Conjunction {
                    grahas: vec![Graha::Sun],
                },
                "grahas",
            ),
            (
                Condition::Conjunction {
                    grahas: vec![Graha::Sun, Graha::Sun],
                },
                "grahas",
            ),
            (
                Condition::GrahaAspects {
                    from: Graha::Rahu,
                    to: Graha::Sun,
                },
                "grahas",
            ),
            (
                Condition::GrahaAspects {
                    from: Graha::Mars,
                    to: Graha::Mars,
                },
                "grahas",
            ),
            (
                Condition::HouseLordInHouse {
                    lord_of: 0,
                    houses: vec![1],
                },
                "lord_of",
            ),
            (
                Condition::HouseLordInHouse {
                    lord_of: 7,
                    houses: vec![0],
                },
                "houses",
            ),
            (
                Condition::MoonPhase {
                    tithis: Vec::new(),
                    paksha: None,
                },
                "tithis",
            ),
            (
                Condition::MoonPhase {
                    tithis: vec![31],
                    paksha: None,
                },
                "tithis",
            ),
            (
                Condition::DashaAt {
                    date: None,
                    level: 4,
                    lords: vec![Graha::Venus],
                },
                "level",
            ),
            (
                Condition::DashaAt {
                    date: None,
                    level: 1,
                    lords: Vec::new(),
                },
                "lords",
            ),
            (
                Condition::DashaAt {
                    date: Some("2026-02-30".into()),
                    level: 1,
                    lords: vec![Graha::Venus],
                },
                "date",
            ),
            (
                Condition::CharaKaraka {
                    role: KarakaRole::Atma,
                    grahas: Vec::new(),
                },
                "grahas",
            ),
            (
                Condition::ArudhaInSign {
                    pada: 13,
                    signs: vec![Sign::Leo],
                },
                "pada",
            ),
            (
                Condition::ArudhaInSign {
                    pada: 1,
                    signs: Vec::new(),
                },
                "signs",
            ),
            (Condition::YogaPresent { ids: Vec::new() }, "yogas"),
            // Değer kümesi sınırsız uzayamaz.
            (mars_in(vec![Sign::Aries; MAX_SET_VALUES + 1]), "signs"),
            (
                Condition::MoonPhase {
                    tithis: vec![1; MAX_SET_VALUES + 1],
                    paksha: None,
                },
                "tithis",
            ),
            (
                Condition::YogaPresent {
                    ids: vec!["no.such_rule".into()],
                },
                "yogas",
            ),
        ];
        for (cond, want) in cases {
            let err = one(cond.clone()).validate().unwrap_err();
            assert_eq!(field(Err(err.clone())), want, "{cond:?}");
            // İleti yalnızca alan adını taşır.
            assert_eq!(err.to_string(), format!("invalid search input: {want}"));
        }
    }

    #[test]
    fn the_scope_limits_its_tags() {
        let mut q = one(mars_in(vec![Sign::Aries]));
        q.scope.tags = vec!["araştırma".into()];
        assert_eq!(q.validate(), Ok(()));
        q.scope.tags = vec!["  ".into()];
        assert_eq!(field(q.validate()), "tags");
        q.scope.tags = vec!["x".repeat(MAX_SCOPE_TAG_CHARS + 1)];
        assert_eq!(field(q.validate()), "tags");
        q.scope.tags = vec!["t".to_owned(); MAX_SCOPE_TAGS + 1];
        assert_eq!(field(q.validate()), "tags");
        assert!(QueryScope::default().exclude_prashna);
    }

    #[test]
    fn dates_are_gregorian_days_at_noon_ut() {
        assert_eq!(date_noon_jd("2000-01-01"), Some(2_451_545.0));
        assert_eq!(date_noon_jd("2024-02-29"), Some(2_460_370.0));
        for bad in [
            "2023-02-29",
            "2000-13-01",
            "2000-00-10",
            "2000-01-00",
            "2000-1-01",
            "20000101",
            "2000-01-01T12",
            "+200-01-01",
            "",
        ] {
            assert_eq!(date_noon_jd(bad), None, "{bad}");
        }
    }

    #[test]
    fn lagna_dependence_is_per_condition() {
        assert!(!mars_in(vec![Sign::Aries]).needs_lagna());
        assert!(Condition::GrahaInHouse {
            graha: Graha::Sun,
            houses: vec![10]
        }
        .needs_lagna());
        assert!(Condition::LagnaInSign {
            signs: vec![Sign::Leo],
            varga: Varga::D1
        }
        .needs_lagna());
        assert!(Condition::ArudhaInSign {
            pada: 1,
            signs: vec![Sign::Leo]
        }
        .needs_lagna());
        // Lagna isteyen ve istemeyen yoga kuralları (Plan 18 kataloğu).
        let yoga = |id: &str| Condition::YogaPresent {
            ids: vec![id.into()],
        };
        let needs: Vec<&str> = crate::yogas::YOGA_CATALOG
            .iter()
            .filter(|r| r.needs_lagna)
            .map(|r| r.id)
            .collect();
        let free: Vec<&str> = crate::yogas::YOGA_CATALOG
            .iter()
            .filter(|r| !r.needs_lagna)
            .map(|r| r.id)
            .collect();
        assert!(!needs.is_empty() && !free.is_empty());
        assert!(yoga(needs[0]).needs_lagna());
        assert!(!yoga(free[0]).needs_lagna());
        let q = ChartQuery {
            all_of: vec![
                QueryCondition {
                    cond: mars_in(vec![Sign::Aries]),
                    negate: false,
                },
                QueryCondition {
                    cond: yoga(needs[0]),
                    negate: true,
                },
            ],
            scope: QueryScope::default(),
        };
        assert!(q.needs_lagna());
    }
}
