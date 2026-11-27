//! Avastha el örnekleri (yol haritası §4.4; ileri modüller spec'i §13): beklenen değerler
//! kuraldan elle türetilir (`derivation`), başka yazılımın çıktısı olamaz (K9). Her alt klasör
//! bir dizidir: `tests/golden/modules/avastha/lajjitadi/` (Baladi, Jagradadi ve Lajjitadi).
//! Okuma kuralları ve onay bekleyen varyant noktaları `avastha/REVIEW.md`'dedir.

use std::collections::BTreeMap;
use std::path::PathBuf;

use jyotish_core::avastha::{
    baladi, jagradadi, lajjitadi, Baladi, Jagradadi, Lajjitadi, LajjitadiHit, AVASTHA_GRAHAS,
};
use jyotish_core::{Graha, Positions};
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// Örnek silmek testi düşürür.
const MIN_LAJJITADI: usize = 7;

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

#[test]
fn unknown_fields_are_rejected() {
    let text = r#"{"lagna_lon": null, "grahas": {}, "extra": 1}"#;
    assert!(serde_json::from_str::<Input>(text).is_err());
    let text = r#"{"graha": "Sun", "state": "Lajjita", "why": "x"}"#;
    assert!(serde_json::from_str::<Absent>(text).is_err());
}
