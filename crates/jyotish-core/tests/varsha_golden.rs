//! Varsha el örnekleri (yol haritası §4.14; ileri modüller spec'i §13): beklenen değerler
//! kuraldan elle türetilir (`derivation`), başka yazılımın çıktısı olamaz (K9). İki alt klasör:
//! `tests/golden/modules/varsha/muntha/` (muntha ve lagna lordları) ve
//! `tests/golden/modules/varsha/mudda/` (ilk lord kuralı ve dönem bitişleri). Dönüş anı bir
//! tablo değildir: gerçek motorla `app/src-tauri/tests/varsha.rs`'te denetlenir. Okuma kuralları
//! ve onay bekleyen varyant noktaları `varsha/REVIEW.md`'dedir.

use std::path::PathBuf;

use jyotish_core::varsha::{
    mudda_dasha, muntha_sign, year_facts, MuddaDasha, Muntha, YearFacts, MUDDA_ORDER,
};
use jyotish_core::{nakshatra_pada, DashaPeriod, Graha, Sign};
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// Örnek silmek testi düşürür.
const MIN_MUNTHA: usize = 5;
const MIN_MUDDA: usize = 9;
/// Örneklerdeki gün değerleri altı ondalığa yuvarlanmıştır.
const DAY_TOLERANCE: f64 = 1e-6;

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

/// Bir dizinin el örnekleri, ortak denetimleriyle: en az sayı, dosya adı, kaynak, gönderme ve
/// türetme adımları.
fn load<X: DeserializeOwned, I: DeserializeOwned>(series: &str, min: usize) -> Vec<Fixture<X, I>> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/golden/modules/varsha")
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

/// Sidereal sayılan iki lagna boylamı (derece) ve tamamlanan yıl.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MunthaInput {
    natal_lagna_lon: f64,
    annual_lagna_lon: f64,
    age: u8,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MunthaExpected {
    sign: Sign,
    lord: Graha,
    natal_house: u8,
    annual_house: u8,
    lagna_sign: Sign,
    lagna_lord: Graha,
    natal_lagna_sign: Sign,
    natal_lagna_lord: Graha,
}

/// Ay'ın sidereal boylamı (derece), tamamlanan yıl ve yılın uzunluğu (gün). Yıl 0. günde
/// başlar: beklenen bitişler yılın başından gün sayısıdır.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MuddaInput {
    moon_lon: f64,
    age: u8,
    year_days: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MuddaExpected {
    birth_nakshatra: u8,
    remainder: u8,
    first_lord: Graha,
    /// Dokuz dönem: lord ve bitiş günü.
    periods: Vec<(Graha, f64)>,
    /// İlk dönemin dokuz alt dönemi: alt lord ve bitiş günü; yazılmadıysa `null`.
    first_sub_periods: Option<Vec<(Graha, f64)>>,
}

fn muntha_fixtures() -> Vec<Fixture<MunthaExpected, MunthaInput>> {
    load("muntha", MIN_MUNTHA)
}

fn mudda_fixtures() -> Vec<Fixture<MuddaExpected, MuddaInput>> {
    load("mudda", MIN_MUDDA)
}

/// Dönemlerin son lordu ve bitişi, sırayla.
fn ends(periods: &[&DashaPeriod]) -> Vec<(Graha, f64)> {
    periods
        .iter()
        .map(|p| (*p.lords.last().expect("a lord"), p.end_jd_ut))
        .collect()
}

fn assert_ends(id: &str, what: &str, got: &[(Graha, f64)], want: &[(Graha, f64)]) {
    assert_eq!(got.len(), 9, "{id}: {what} has nine periods");
    assert_eq!(want.len(), 9, "{id}: {what} lists nine periods");
    for (k, (g, w)) in got.iter().zip(want).enumerate() {
        assert_eq!(g.0, w.0, "{id}: {what} lord {k}");
        assert!(
            (g.1 - w.1).abs() <= DAY_TOLERANCE,
            "{id}: {what} {:?} ends at {} (expected {})",
            w.0,
            g.1,
            w.1
        );
    }
}

#[test]
fn muntha_hand_fixtures() {
    for f in muntha_fixtures() {
        let (i, x) = (&f.input, &f.expected);
        let (natal, annual) = (Sign::of(i.natal_lagna_lon), Sign::of(i.annual_lagna_lon));
        assert_eq!(muntha_sign(natal, i.age), x.sign, "{}", f.id);
        assert_eq!(
            year_facts(natal, annual, i.age),
            YearFacts {
                lagna_sign: x.lagna_sign,
                lagna_lord: x.lagna_lord,
                natal_lagna_sign: x.natal_lagna_sign,
                natal_lagna_lord: x.natal_lagna_lord,
                muntha: Muntha {
                    sign: x.sign,
                    lord: x.lord,
                    natal_house: x.natal_house,
                    annual_house: x.annual_house,
                },
            },
            "{}",
            f.id
        );
        // Örneğin kendi aritmetiği, motordan bağımsız: doğum lagnasından sayılan ev, tamamlanan
        // yılın 12'ye bölümünden artanın bir fazlasıdır.
        assert_eq!(x.natal_house, i.age % 12 + 1, "{}: natal house", f.id);
        assert_eq!(x.sign.house_from(natal), x.natal_house, "{}", f.id);
    }
}

#[test]
fn mudda_hand_fixtures() {
    for f in mudda_fixtures() {
        let (i, x) = (&f.input, &f.expected);
        let nakshatra = nakshatra_pada(i.moon_lon).0;
        assert_eq!(nakshatra, x.birth_nakshatra, "{}: birth nakshatra", f.id);
        let d: MuddaDasha = mudda_dasha(nakshatra, i.age, 0.0, i.year_days)
            .unwrap_or_else(|e| panic!("{}: {e}", f.id));
        assert_eq!(
            (d.remainder, d.first_lord),
            (x.remainder, x.first_lord),
            "{}",
            f.id
        );
        let top: Vec<&DashaPeriod> = d.periods.iter().filter(|p| p.level == 1).collect();
        assert_ends(&f.id, "year", &ends(&top), &x.periods);
        if let Some(subs) = &x.first_sub_periods {
            let kids: Vec<&DashaPeriod> = d.periods[1..10].iter().collect();
            assert!(kids.iter().all(|k| k.level == 2), "{}", f.id);
            assert_ends(&f.id, "first period", &ends(&kids), subs);
            // Alt dönemler üst dönemin lorduyla başlar ve üst dönemin bitişinde biter.
            assert_eq!(subs[0].0, x.first_lord, "{}", f.id);
            assert!(
                (subs[8].1 - x.periods[0].1).abs() <= DAY_TOLERANCE,
                "{}",
                f.id
            );
        }
        // Örneğin kendi aritmetiği, motordan bağımsız: artan, toplamın 9'a bölümünden artandır;
        // ilk lord sıradaki o sayıdır (0 dokuzuncudur); dönemler sırayı çevrimsel izler ve son
        // dönem yılın sonunda biter.
        let total = i32::from(x.birth_nakshatra) + i32::from(i.age) - 2;
        assert_eq!(i32::from(x.remainder), total.rem_euclid(9), "{}", f.id);
        let first = (usize::from(x.remainder) + 8) % 9;
        assert_eq!(x.first_lord, MUDDA_ORDER[first], "{}", f.id);
        for (k, (lord, _)) in x.periods.iter().enumerate() {
            assert_eq!(*lord, MUDDA_ORDER[(first + k) % 9], "{}: order {k}", f.id);
        }
        assert!(
            (x.periods[8].1 - i.year_days).abs() <= DAY_TOLERANCE,
            "{}: the last period closes the year",
            f.id
        );
    }
}

#[test]
fn fixtures_cover_the_required_cases() {
    let mudda = mudda_fixtures();
    // Dokuz lordun her biri bir örnekte yılı açar; dokuz artan değerinin hepsi görülür.
    for (k, lord) in MUDDA_ORDER.into_iter().enumerate() {
        assert!(
            mudda.iter().any(|f| f.expected.first_lord == lord),
            "{lord:?} opens no fixture year"
        );
        assert!(
            mudda
                .iter()
                .any(|f| usize::from(f.expected.remainder) == (k + 1) % 9),
            "remainder {} is in no fixture",
            (k + 1) % 9
        );
    }
    // Toplamın eksiye düştüğü durum (Ashwini, doğum yılı), 360 günlük yıl, dönüşten dönüşe yıl
    // ve alt dönemler.
    assert!(mudda
        .iter()
        .any(|f| f.expected.birth_nakshatra == 1 && f.input.age == 0));
    assert!(mudda.iter().any(|f| f.input.year_days == 360.0));
    assert!(mudda.iter().any(|f| f.input.year_days != 360.0));
    assert!(
        mudda
            .iter()
            .filter(|f| f.expected.first_sub_periods.is_some())
            .count()
            >= 3
    );
    let muntha = muntha_fixtures();
    // Doğum yılı, tam on iki yıl, Balık'tan Koç'a dönen sayım ve on ikiden büyük yaş.
    assert!(muntha.iter().any(|f| f.input.age == 0));
    assert!(muntha
        .iter()
        .any(|f| f.input.age > 0 && f.input.age % 12 == 0));
    assert!(muntha
        .iter()
        .any(|f| f.expected.sign.index() < f.expected.natal_lagna_sign.index()));
    assert!(muntha.iter().any(|f| f.input.age > 100));
}
