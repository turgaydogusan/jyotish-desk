//! İleri modüllerin el örnekleri (ileri modüller spec'i §13): `tests/golden/modules/<modül>/*.json`.
//! Girdi sentetik konumlardır (ephemeris gerekmez); beklenen değer klasik kuraldan elle
//! türetilmiştir ve `derivation` alanında adım adım yazılıdır. Bir örnek motorla uyuşmazsa
//! önce kural yeniden okunur; beklenen değer motor çıktısına bakarak düzeltilmez.

use std::collections::BTreeMap;
use std::path::PathBuf;

use jyotish_core::{
    arudha_padas, chara_karakas, co_lord_choice, drishti_table, house_lord, houses_owned,
    jaimini_lord, karakamsha, kuja_dosha, moon_is_waxing, nature, normalize, AdvancedSettings,
    Graha, KarakaRole, KujaDosha, Nature, Positions, Sign, ASPECTING_GRAHAS, DRISHTI_METHOD,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// Modül başına en az örnek sayısı; altına düşerse test düşer (sessiz silinmeye karşı).
const MIN_FOUNDATION: usize = 6;
const MIN_DRISHTI: usize = 4;
const MIN_JAIMINI: usize = 8;
/// El hesabı ile motor arasındaki en büyük fark, virupa.
const VIRUPA_TOL: f64 = 1e-9;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture<X> {
    id: String,
    source: String,
    rule_ref: String,
    derivation: Vec<String>,
    input: Input,
    expected: X,
}

/// Sentetik konumlar: sidereal boylamlar, derece. `lagna_lon: null` doğum saati bilinmeyen
/// haritadır. `advanced` yalnızca ayara bağlı modüllerde (Jaimini) verilir.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    lagna_lon: Option<f64>,
    grahas: BTreeMap<Graha, f64>,
    #[serde(default)]
    advanced: Option<AdvancedSettings>,
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

fn load<X: DeserializeOwned>(module: &str, min: usize) -> Vec<(Fixture<X>, Positions)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/golden/modules")
        .join(module);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    assert!(
        paths.len() >= min,
        "{module}: {} hand fixtures, at least {min} required",
        paths.len()
    );
    paths
        .into_iter()
        .map(|p| {
            let text =
                std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            let f: Fixture<X> =
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
            let positions = f.input.positions(&f.id);
            (f, positions)
        })
        .collect()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FoundationExpected {
    waxing: bool,
    nature: BTreeMap<Graha, Nature>,
    kuja_dosha: KujaDosha,
    /// 1.–12. evin lordu; doğum saati bilinmiyorsa `null`.
    house_lords: Option<[Graha; 12]>,
    /// Her grahanın lagnadan evi; doğum saati bilinmiyorsa `null`.
    houses: Option<BTreeMap<Graha, u8>>,
    /// Her grahanın lordu olduğu evler; doğum saati bilinmiyorsa `null`.
    houses_owned: Option<BTreeMap<Graha, Vec<u8>>>,
}

#[test]
fn foundation_hand_fixtures() {
    for (f, p) in load::<FoundationExpected>("foundation", MIN_FOUNDATION) {
        let id = &f.id;
        let x = &f.expected;
        assert_eq!(moon_is_waxing(&p), x.waxing, "{id}: waxing");
        assert_eq!(x.nature.len(), 9, "{id}: nature lists nine grahas");
        for g in Graha::ALL {
            assert_eq!(nature(g, &p), x.nature[&g], "{id}: nature of {g:?}");
        }
        assert_eq!(kuja_dosha(&p), x.kuja_dosha, "{id}: kuja dosha");
        match p.lagna_sign {
            Some(lagna) => {
                let lords: [Graha; 12] = std::array::from_fn(|i| house_lord(lagna, i as u8 + 1));
                assert_eq!(Some(lords), x.house_lords, "{id}: house lords");
                let houses = x.houses.as_ref().expect("houses with a lagna");
                let owned = x.houses_owned.as_ref().expect("houses_owned with a lagna");
                assert_eq!((houses.len(), owned.len()), (9, 9), "{id}: nine grahas");
                for g in Graha::ALL {
                    assert_eq!(p.house_of(g), Some(houses[&g]), "{id}: house of {g:?}");
                    assert_eq!(houses_owned(lagna, g), owned[&g], "{id}: owned by {g:?}");
                }
            }
            None => {
                assert!(
                    x.house_lords.is_none() && x.houses.is_none() && x.houses_owned.is_none(),
                    "{id}: lagna-dependent values must be null without a lagna"
                );
                assert_eq!(x.kuja_dosha.from_lagna, None, "{id}");
                for g in Graha::ALL {
                    assert_eq!(p.house_of(g), None, "{id}");
                }
            }
        }
    }
}

#[test]
fn unknown_fields_are_rejected() {
    let base = r#"{"id":"x","source":"hand","rule_ref":"r","derivation":["d"],
        "input":{"lagna_lon":null,"grahas":{}},
        "expected":{"waxing":true,"nature":{},
            "kuja_dosha":{"from_lagna":null,"from_moon":false,"from_venus":false},
            "house_lords":null,"houses":null,"houses_owned":null}}"#;
    assert!(serde_json::from_str::<Fixture<FoundationExpected>>(base).is_ok());
    let extra = base.replace(r#""id":"x","#, r#""id":"x","note":"n","#);
    assert!(serde_json::from_str::<Fixture<FoundationExpected>>(&extra).is_err());
    let extra_input = base.replace(r#""grahas":{}"#, r#""grahas":{},"lagna":1"#);
    assert!(serde_json::from_str::<Fixture<FoundationExpected>>(&extra_input).is_err());
    let extra_expected = base.replace(r#""waxing":true,"#, r#""waxing":true,"more":1,"#);
    assert!(serde_json::from_str::<Fixture<FoundationExpected>>(&extra_expected).is_err());
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct ExpectedAspect {
    graha: Graha,
    target: Sign,
    occupants: Vec<Graha>,
}

/// Bir sphuta drishti değeri: `angle` = (bakılan − bakan) mod 360, `value` virupa.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedSphuta {
    from: Graha,
    to: Graha,
    angle: f64,
    value: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DrishtiExpected {
    /// On üç tam bakışın tamamı, tablo sırasıyla.
    sign_aspects: Vec<ExpectedAspect>,
    sphuta: Vec<ExpectedSphuta>,
}

#[test]
fn drishti_hand_fixtures() {
    for (f, p) in load::<DrishtiExpected>("drishti", MIN_DRISHTI) {
        let id = &f.id;
        let table = drishti_table(&p);
        assert_eq!(table.method, DRISHTI_METHOD, "{id}");
        let got: Vec<ExpectedAspect> = table
            .sign_aspects
            .iter()
            .map(|a| ExpectedAspect {
                graha: a.graha,
                target: a.target,
                occupants: a.occupants.clone(),
            })
            .collect();
        assert_eq!(got, f.expected.sign_aspects, "{id}: sign aspects");
        assert!(f.expected.sphuta.len() >= 10, "{id}: at least ten values");
        for s in &f.expected.sphuta {
            let row = ASPECTING_GRAHAS
                .iter()
                .position(|&g| g == s.from)
                .unwrap_or_else(|| panic!("{id}: {:?} casts no graha aspect", s.from));
            // Türetmedeki açı girdiyle tutarlı olmalı.
            let angle = normalize(p.point(s.to).sidereal_lon - p.point(s.from).sidereal_lon);
            assert!(
                (angle - s.angle).abs() < 1e-9,
                "{id}: {:?} → {:?} angle {angle}, fixture says {}",
                s.from,
                s.to,
                s.angle
            );
            let got = table.sphuta[row][s.to as usize];
            assert!(
                (got - s.value).abs() <= VIRUPA_TOL,
                "{id}: {:?} → {:?} at {}°: got {got}, hand value {}",
                s.from,
                s.to,
                s.angle,
                s.value
            );
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedKaraka {
    role: KarakaRole,
    graha: Graha,
    effective_deg: f64,
    tie: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedKarakamsha {
    atmakaraka: Graha,
    sign: Sign,
    houses_in_d1: BTreeMap<Graha, u8>,
    houses_in_d9: BTreeMap<Graha, u8>,
}

/// Eş lord seçimi: seçilen lord ve belirleyici ölçütün numarası.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedCoLord {
    lord: Graha,
    step: u8,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct ExpectedPada {
    house: u8,
    sign: Sign,
    lord: Graha,
    exception: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct JaiminiExpected {
    /// Rol sırasında: sekizli şemada 8, yedili şemada 7 kayıt.
    karakas: Vec<ExpectedKaraka>,
    karakamsha: ExpectedKarakamsha,
    /// Akrep ve Kova için seçim kuralının sonucu (ayar devre dışıyken de izlenir).
    co_lords: BTreeMap<Sign, ExpectedCoLord>,
    /// A1…A12; doğum saati bilinmiyorsa `null`.
    arudhas: Option<Vec<ExpectedPada>>,
}

#[test]
fn jaimini_hand_fixtures() {
    for (f, p) in load::<JaiminiExpected>("jaimini", MIN_JAIMINI) {
        let id = &f.id;
        let x = &f.expected;
        let advanced = f
            .input
            .advanced
            .unwrap_or_else(|| panic!("{id}: jaimini fixtures state the advanced settings"));

        // Chara karaka.
        let karakas = chara_karakas(&p, advanced.karaka_scheme);
        assert_eq!(karakas.len(), x.karakas.len(), "{id}: karaka count");
        for (got, want) in karakas.iter().zip(&x.karakas) {
            assert_eq!(
                (got.role, got.graha, got.tie),
                (want.role, want.graha, want.tie),
                "{id}: {:?}",
                want.role
            );
            assert!(
                (got.effective_deg - want.effective_deg).abs() < 1e-9,
                "{id}: {:?} effective degree {}, hand value {}",
                want.role,
                got.effective_deg,
                want.effective_deg
            );
        }

        // Karakamsha.
        assert_eq!(x.karakamsha.atmakaraka, x.karakas[0].graha, "{id}");
        let k = karakamsha(&p, x.karakamsha.atmakaraka);
        assert_eq!(k.sign, x.karakamsha.sign, "{id}: karakamsha");
        assert_eq!(
            (
                x.karakamsha.houses_in_d1.len(),
                x.karakamsha.houses_in_d9.len()
            ),
            (9, 9),
            "{id}: nine grahas"
        );
        for g in Graha::ALL {
            let i = g as usize;
            assert_eq!(
                k.houses_in_d1[i], x.karakamsha.houses_in_d1[&g],
                "{id}: D1 house of {g:?}"
            );
            assert_eq!(
                k.houses_in_d9[i], x.karakamsha.houses_in_d9[&g],
                "{id}: D9 house of {g:?}"
            );
        }

        // Eş lordlar.
        assert_eq!(
            x.co_lords.keys().copied().collect::<Vec<_>>(),
            [Sign::Scorpio, Sign::Aquarius],
            "{id}: co-lorded signs"
        );
        for (&sign, want) in &x.co_lords {
            let got = co_lord_choice(sign, &p).expect("co-lorded sign");
            assert_eq!(
                (got.lord, got.step),
                (want.lord, want.step),
                "{id}: co-lord of {sign:?}"
            );
            assert_eq!(jaimini_lord(sign, &p, true), want.lord, "{id}: {sign:?}");
        }

        // Arudha padalar.
        let padas = arudha_padas(&p, advanced.jaimini_co_lords).map(|a| {
            a.iter()
                .map(|x| ExpectedPada {
                    house: x.house,
                    sign: x.sign,
                    lord: x.lord,
                    exception: x.exception,
                })
                .collect::<Vec<_>>()
        });
        assert_eq!(padas, x.arudhas, "{id}: arudha padas");
        assert_eq!(x.arudhas.is_some(), p.lagna_sign.is_some(), "{id}");
        if let Some(a) = &x.arudhas {
            assert_eq!(a.len(), 12, "{id}: twelve padas");
        }
    }
}

/// Zorunlu durumlar (ileri modüller spec'i §5 "Test"): örnekler silinir ya da değiştirilirse
/// kapsam sessizce daralmasın.
#[test]
fn jaimini_fixtures_cover_the_required_cases() {
    let all = load::<JaiminiExpected>("jaimini", MIN_JAIMINI);
    let any = |pred: &dyn Fn(&Fixture<JaiminiExpected>, &Positions) -> bool| {
        all.iter().any(|(f, p)| pred(f, p))
    };
    assert!(
        any(&|f, _| f.expected.karakas[0].graha == Graha::Rahu),
        "Rahu as Atmakaraka"
    );
    assert!(
        any(&|f, _| f.expected.karakas.iter().any(|k| k.tie)),
        "a tie within one arcsecond"
    );
    assert!(
        any(&|f, _| f.expected.karakas.len() == 7),
        "the seven-karaka scheme"
    );
    assert!(
        any(&|f, p| p.sign_of(f.expected.karakamsha.atmakaraka) == f.expected.karakamsha.sign),
        "a vargottama Atmakaraka"
    );
    assert!(
        any(&|f, _| f.input.advanced.is_some_and(|a| !a.jaimini_co_lords)),
        "co-lords switched off"
    );
    assert!(
        any(&|f, _| f.expected.arudhas.is_none()),
        "birth time unknown"
    );
    // Lordun evden 1., 4., 7. ve 10. burçta olduğu istisnalar.
    for n in [1u8, 4, 7, 10] {
        assert!(
            any(&|f, p| {
                let (Some(lagna), Some(padas)) = (p.lagna_sign, &f.expected.arudhas) else {
                    return false;
                };
                padas.iter().any(|x| {
                    x.exception && p.sign_of(x.lord).house_from(house_sign_of(lagna, x.house)) == n
                })
            }),
            "arudha exception with the lord in house {n}"
        );
    }
    // Eş lord seçiminin altı ölçütünden her biri en az bir örnekte belirleyicidir.
    for step in 1..=6u8 {
        assert!(
            any(&|f, _| f.expected.co_lords.values().any(|c| c.step == step)),
            "co-lord step {step}"
        );
    }
}

/// `lagna` burcundan sayılan `house`. evin burcu (test yardımcısı; motorun `house_sign`'ı
/// yerine elle sayım).
fn house_sign_of(lagna: Sign, house: u8) -> Sign {
    Sign::ALL[(lagna as usize + usize::from(house) - 1) % 12]
}
