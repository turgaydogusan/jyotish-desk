//! Avastha el örnekleri (yol haritası §4.4; ileri modüller spec'i §13): beklenen değerler
//! kuraldan elle türetilir (`derivation`), başka yazılımın çıktısı olamaz (K9). Her alt klasör
//! bir dizidir: `tests/golden/modules/avastha/lajjitadi/` (Baladi, Jagradadi ve Lajjitadi) ve
//! `tests/golden/modules/avastha/shayanadi/` (Shayanadi; girdide gün doğumundan geçen dakika).
//! Okuma kuralları ve onay bekleyen varyant noktaları `avastha/REVIEW.md`'dedir.

use std::collections::BTreeMap;
use std::path::PathBuf;

use jyotish_core::avastha::{
    avasthas, baladi, elapsed_ghatis, graha_number, jagradadi, lajjitadi, Baladi, Jagradadi,
    Lajjitadi, LajjitadiHit, Shayanadi, ShayanadiBasis, AVASTHA_GRAHAS,
};
use jyotish_core::{Graha, Positions};
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// Örnek silmek testi düşürür.
const MIN_LAJJITADI: usize = 7;
const MIN_SHAYANADI: usize = 6;
/// Bir gün, dakika.
const MINUTES_PER_DAY: f64 = 1440.0;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture<X, I> {
    id: String,
    source: String,
    rule_ref: String,
    derivation: Vec<String>,
    input: I,
    expected: X,
}

/// Sentetik konumlar: sidereal boylamlar, derece. `lagna_lon: null` doğum saati bilinmeyen
/// haritadır.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    lagna_lon: Option<f64>,
    grahas: BTreeMap<Graha, f64>,
}

impl Input {
    fn positions(&self, id: &str) -> Positions {
        assert_eq!(self.grahas.len(), 9, "{id}: all nine grahas are required");
        Positions::synthetic(
            self.lagna_lon,
            std::array::from_fn(|i| self.grahas[&Graha::ALL[i]]),
        )
    }
}

/// Bir dizinin el örnekleri, ortak denetimleriyle: en az sayı, dosya adı, kaynak, gönderme ve
/// türetme adımları.
fn load<X: DeserializeOwned, I: DeserializeOwned>(series: &str, min: usize) -> Vec<Fixture<X, I>> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/golden/modules/avastha")
        .join(series);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    assert!(
        paths.len() >= min,
        "{series}: {} hand fixtures, at least {min} required",
        paths.len()
    );
    paths
        .into_iter()
        .map(|p| {
            let text =
                std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            let f: Fixture<X, I> =
                serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            assert_eq!(
                p.file_stem().and_then(|s| s.to_str()),
                Some(f.id.as_str()),
                "id must equal the file name"
            );
            // Başka yazılımın çıktısı fixture olamaz (K9).
            assert_eq!(f.source, "hand", "{}: only hand-derived fixtures", f.id);
            assert!(!f.rule_ref.trim().is_empty(), "{}: rule_ref", f.id);
            assert!(
                !f.derivation.is_empty() && f.derivation.iter().all(|s| !s.trim().is_empty()),
                "{}: derivation steps",
                f.id
            );
            f
        })
        .collect()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Absent {
    graha: Graha,
    state: Lajjitadi,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LajjitadiExpected {
    baladi: BTreeMap<Graha, Baladi>,
    jagradadi: BTreeMap<Graha, Jagradadi>,
    /// Yedi grahanın durumları, kural sırasında; durumu olmayan graha boş liste taşır.
    lajjitadi: BTreeMap<Graha, Vec<LajjitadiHit>>,
    /// Bu örneğin "oluşmayan durum" olarak gösterdiği çiftler.
    absent: Vec<Absent>,
}

fn lajjitadi_fixtures() -> Vec<(Fixture<LajjitadiExpected, Input>, Positions)> {
    load::<LajjitadiExpected, Input>("lajjitadi", MIN_LAJJITADI)
        .into_iter()
        .map(|f| {
            let positions = f.input.positions(&f.id);
            (f, positions)
        })
        .collect()
}

#[test]
fn lajjitadi_hand_fixtures() {
    for (f, p) in lajjitadi_fixtures() {
        let x = &f.expected;
        let seven: Vec<Graha> = AVASTHA_GRAHAS.to_vec();
        for (what, keys) in [
            ("baladi", x.baladi.keys().copied().collect::<Vec<_>>()),
            ("jagradadi", x.jagradadi.keys().copied().collect()),
            ("lajjitadi", x.lajjitadi.keys().copied().collect()),
        ] {
            assert_eq!(keys, seven, "{}: {what} lists the seven grahas", f.id);
        }
        for graha in AVASTHA_GRAHAS {
            let point = p.point(graha);
            assert_eq!(
                baladi(point.rasi_sign, point.deg_in_sign),
                x.baladi[&graha],
                "{}: baladi of {graha:?}",
                f.id
            );
            assert_eq!(
                jagradadi(graha, point.rasi_sign),
                Some(x.jagradadi[&graha]),
                "{}: jagradadi of {graha:?}",
                f.id
            );
            assert_eq!(
                lajjitadi(graha, &p),
                x.lajjitadi[&graha],
                "{}: lajjitadi of {graha:?}",
                f.id
            );
        }
        // "Oluşmayan" çiftler beklenen listede de motorda da yoktur.
        assert!(!x.absent.is_empty(), "{}: absent pairs", f.id);
        for a in &x.absent {
            assert!(
                x.lajjitadi[&a.graha].iter().all(|h| h.state != a.state),
                "{}: {:?} {:?} is listed both as present and absent",
                f.id,
                a.graha,
                a.state
            );
        }
        // Lagna yoksa Lajjita hiçbir grahada beklenmez.
        if f.input.lagna_lon.is_none() {
            assert!(
                x.lajjitadi
                    .values()
                    .flatten()
                    .all(|h| h.state != Lajjitadi::Lajjita),
                "{}",
                f.id
            );
        }
    }
}

#[test]
fn lajjitadi_fixtures_cover_the_required_cases() {
    let fixtures = lajjitadi_fixtures();
    let hits = || {
        fixtures
            .iter()
            .flat_map(|(f, _)| f.expected.lajjitadi.values())
    };
    // Her durum en az bir örnekte oluşur ve en az bir örnekte "oluşmayan" olarak gösterilir.
    for state in Lajjitadi::ALL {
        assert!(
            hits().flatten().any(|h| h.state == state),
            "{state:?} holds in no fixture"
        );
        assert!(
            fixtures
                .iter()
                .flat_map(|(f, _)| &f.expected.absent)
                .any(|a| a.state == state),
            "{state:?} is shown absent in no fixture"
        );
    }
    // Küme sonucu: hiçbir durumda olmayan graha, tek durum ve en az dört durum birlikte.
    assert!(hits().any(|list| list.is_empty()), "an empty set");
    assert!(hits().any(|list| list.len() == 1), "a single state");
    assert!(hits().any(|list| list.len() >= 4), "four or more states");
    // Kuralı sağlayan grahalar: boş liste (yalnızca burçtan) ve birden çok graha.
    assert!(hits().flatten().any(|h| h.grahas.is_empty()));
    assert!(hits().flatten().any(|h| h.grahas.len() >= 2));
    // Kshudhita'nın dördüncü kolu: düşmanı olmayan Satürn ile birliktelik tek başına yeter.
    assert!(
        fixtures.iter().any(|(f, _)| {
            [Graha::Mercury, Graha::Jupiter, Graha::Venus]
                .iter()
                .any(|g| {
                    f.expected.lajjitadi[g]
                        .iter()
                        .any(|h| h.state == Lajjitadi::Kshudhita && h.grahas == [Graha::Saturn])
                })
        }),
        "Kshudhita by Saturn's company alone"
    );
    // Doğum saati bilinmeyen harita, 5. evde eşlikçisi olacak grahayla.
    assert!(
        fixtures.iter().any(|(f, _)| f.input.lagna_lon.is_none()
            && f.expected
                .absent
                .iter()
                .any(|a| a.state == Lajjitadi::Lajjita)),
        "a time-unknown fixture that shows Lajjita is not evaluated"
    );
    // Baladi: beş basamağın her biri hem tek hem çift burçta; Jagradadi: üç durum.
    for state in Baladi::ALL {
        for odd in [true, false] {
            assert!(
                fixtures
                    .iter()
                    .any(|(f, p)| AVASTHA_GRAHAS.iter().any(|&g| {
                        f.expected.baladi[&g] == state
                            && p.sign_of(g).index().is_multiple_of(2) == odd
                    })),
                "{state:?} in an {} sign",
                if odd { "odd" } else { "even" }
            );
        }
    }
    for state in Jagradadi::ALL {
        assert!(
            fixtures
                .iter()
                .any(|(f, _)| f.expected.jagradadi.values().any(|&j| j == state)),
            "{state:?}"
        );
    }
}

/// Shayanadi örneğinin girdisi: konumlar ve gün doğumundan doğuma geçen dakika. `null` dakika
/// o gün gün doğumu olmayan yerdir.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShayanadiInput {
    lagna_lon: Option<f64>,
    minutes_since_sunrise: Option<f64>,
    grahas: BTreeMap<Graha, f64>,
}

impl ShayanadiInput {
    fn positions(&self, id: &str) -> Positions {
        assert_eq!(self.grahas.len(), 9, "{id}: all nine grahas are required");
        Positions::synthetic(
            self.lagna_lon,
            std::array::from_fn(|i| self.grahas[&Graha::ALL[i]]),
        )
    }

    /// Tamamlanmış ghati: gün doğumu 0 anı sayılır.
    fn ghatis(&self) -> Option<u32> {
        self.minutes_since_sunrise
            .and_then(|m| elapsed_ghatis(m / MINUTES_PER_DAY, 0.0))
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShayanadiRow {
    state: Shayanadi,
    nakshatra: u8,
    navamsa: u8,
    total: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShayanadiExpected {
    basis: ShayanadiBasis,
    grahas: BTreeMap<Graha, ShayanadiRow>,
}

type ShayanadiFixture = Fixture<Option<ShayanadiExpected>, ShayanadiInput>;

fn shayanadi_fixtures() -> Vec<(ShayanadiFixture, Positions)> {
    load::<Option<ShayanadiExpected>, ShayanadiInput>("shayanadi", MIN_SHAYANADI)
        .into_iter()
        .map(|f| {
            let positions = f.input.positions(&f.id);
            (f, positions)
        })
        .collect()
}

/// İkinci yazım: bölme yok; durum çemberinde `total` adım yürünür (ilk adım ilk duruma varır).
fn walked_state(total: u32) -> Shayanadi {
    let mut at = Shayanadi::ALL.len() - 1;
    for _ in 0..total {
        at = if at + 1 == Shayanadi::ALL.len() {
            0
        } else {
            at + 1
        };
    }
    Shayanadi::ALL[at]
}

#[test]
fn shayanadi_hand_fixtures() {
    for (f, p) in shayanadi_fixtures() {
        let report = avasthas(&p, f.input.ghatis());
        assert_eq!(report.grahas.len(), 7, "{}", f.id);
        let Some(x) = &f.expected else {
            // Lagna ya da gün doğumu yok: Shayanadi hesaplanmaz.
            assert!(
                f.input.lagna_lon.is_none() || f.input.minutes_since_sunrise.is_none(),
                "{}: a null expectation needs a missing lagna or sunrise",
                f.id
            );
            assert_eq!(report.shayanadi_basis, None, "{}", f.id);
            assert!(
                report.grahas.iter().all(|g| g.shayanadi.is_none()),
                "{}",
                f.id
            );
            continue;
        };
        assert_eq!(report.shayanadi_basis, Some(x.basis), "{}: basis", f.id);
        assert_eq!(
            x.grahas.keys().copied().collect::<Vec<_>>(),
            AVASTHA_GRAHAS,
            "{}: the seven grahas",
            f.id
        );
        let addend = u32::from(x.basis.birth_nakshatra)
            + x.basis.ghatis
            + u32::from(x.basis.lagna_sign_number);
        for row in &report.grahas {
            let want = &x.grahas[&row.graha];
            let got = row.shayanadi.expect("a state for each of the seven grahas");
            assert_eq!(
                (got.state, got.nakshatra, got.navamsa, got.total),
                (want.state, want.nakshatra, want.navamsa, want.total),
                "{}: {:?}",
                f.id,
                row.graha
            );
            // Örneğin kendi aritmetiği tutarlıdır (motordan bağımsız): toplam ve durum.
            let number = u32::from(graha_number(row.graha).expect("seven grahas"));
            assert_eq!(
                u32::from(want.nakshatra) * number * u32::from(want.navamsa) + addend,
                want.total,
                "{}: {:?} total",
                f.id,
                row.graha
            );
            assert_eq!(
                walked_state(want.total),
                want.state,
                "{}: {:?} state",
                f.id,
                row.graha
            );
            assert_eq!(
                u32::from(got.remainder),
                want.total % 12,
                "{}: {:?} remainder",
                f.id,
                row.graha
            );
        }
    }
}

#[test]
fn shayanadi_fixtures_cover_the_required_cases() {
    let fixtures = shayanadi_fixtures();
    let rows = || {
        fixtures
            .iter()
            .filter_map(|(f, _)| f.expected.as_ref())
            .flat_map(|x| x.grahas.values())
    };
    // On iki durumun her biri en az bir örnekte.
    for state in Shayanadi::ALL {
        assert!(rows().any(|r| r.state == state), "{state:?} in no fixture");
    }
    // Artanın sıfır olduğu durum: on ikinci durum.
    assert!(rows().any(|r| r.total.is_multiple_of(12) && r.state == Shayanadi::Nidra));
    // Ghati sayımı: tam doğuşta 0, yarım ghati aşağı yuvarlanır, sonraki doğuştan önce 59.
    let ghatis: Vec<u32> = fixtures
        .iter()
        .filter_map(|(f, _)| f.expected.as_ref().map(|x| x.basis.ghatis))
        .collect();
    for wanted in [0, 1, 28, 59] {
        assert!(ghatis.contains(&wanted), "{wanted} ghatis in no fixture");
    }
    assert!(
        fixtures.iter().any(|(f, _)| f
            .input
            .minutes_since_sunrise
            .is_some_and(|m| (m / 24.0).fract() > 0.9)),
        "a birth late in a ghati"
    );
    // Hesaplanmayan iki durum: lagna yok; gün doğumu yok.
    assert!(fixtures
        .iter()
        .any(|(f, _)| f.expected.is_none() && f.input.lagna_lon.is_none()));
    assert!(fixtures.iter().any(|(f, _)| f.expected.is_none()
        && f.input.lagna_lon.is_some()
        && f.input.minutes_since_sunrise.is_none()));
}

#[test]
fn unknown_fields_are_rejected() {
    let text = r#"{"lagna_lon": null, "grahas": {}, "extra": 1}"#;
    assert!(serde_json::from_str::<Input>(text).is_err());
    let text = r#"{"graha": "Sun", "state": "Lajjita", "why": "x"}"#;
    assert!(serde_json::from_str::<Absent>(text).is_err());
    let text = r#"{"state": "Nidra", "nakshatra": 1, "navamsa": 1, "total": 12, "note": "x"}"#;
    assert!(serde_json::from_str::<ShayanadiRow>(text).is_err());
}
