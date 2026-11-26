//! İleri modüllerin el örnekleri (ileri modüller spec'i §13): `tests/golden/modules/<modül>/*.json`.
//! Girdi sentetik konumlardır (ephemeris gerekmez); beklenen değer klasik kuraldan elle
//! türetilmiştir ve `derivation` alanında adım adım yazılıdır. Bir örnek motorla uyuşmazsa
//! önce kural yeniden okunur; beklenen değer motor çıktısına bakarak düzeltilmez.

use std::collections::BTreeMap;
use std::path::PathBuf;

use jyotish_core::{
    argala_rows, arudha_padas, chara_dasha, chara_dasha_order, chara_dasha_sign_years,
    chara_karakas, co_lord_choice, drishti_table, house_lord, houses_owned, jaimini_lord,
    jaimini_report, karakamsha, kuja_dosha, moon_is_waxing, nature, normalize, shadbala,
    sign_dasha_children, AdvancedSettings, ArgalaStatus, Graha, GrahaWar, KarakaRole, KujaDosha,
    Nature, Positions, ShadbalaAstro, Sign, SignDashaPeriod, TimeLords, ASPECTING_GRAHAS,
    CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD, SHADBALA_GRAHAS, SHADBALA_METHOD,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// Modül başına en az örnek sayısı; altına düşerse test düşer (sessiz silinmeye karşı).
const MIN_FOUNDATION: usize = 6;
const MIN_DRISHTI: usize = 4;
const MIN_JAIMINI: usize = 8;
const MIN_CHARA_DASHA: usize = 7;
const MIN_SHADBALA: usize = 4;
/// El hesabı ile motor arasındaki en büyük fark, virupa.
const VIRUPA_TOL: f64 = 1e-9;
/// Shadbala çalışma kâğıdının toleransı, virupa (ileri modüller spec'i §7 "Test").
const WORKSHEET_TOL: f64 = 0.01;

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
/// haritadır. `advanced` yalnızca ayara bağlı modüllerde (Jaimini, chara dasha) verilir;
/// `astro` (saf hesabın astronomik girdisi) ve `lats` (ekliptik enlemler; yazılmayan graha 0°)
/// yalnızca Shadbala örneklerinde.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    lagna_lon: Option<f64>,
    grahas: BTreeMap<Graha, f64>,
    #[serde(default)]
    advanced: Option<AdvancedSettings>,
    #[serde(default)]
    lats: Option<BTreeMap<Graha, f64>>,
    #[serde(default)]
    astro: Option<ShadbalaAstro>,
}

impl Input {
    fn positions(&self, id: &str) -> Positions {
        assert_eq!(self.grahas.len(), 9, "{id}: all nine grahas are required");
        let mut positions = Positions::synthetic(
            self.lagna_lon,
            std::array::from_fn(|i| self.grahas[&Graha::ALL[i]]),
        );
        for (graha, lat) in self.lats.iter().flatten() {
            positions.grahas[*graha as usize].lat = *lat;
        }
        positions
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

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct ExpectedArgalaPair {
    argala_house: u8,
    virodha_house: u8,
    argala_grahas: Vec<Graha>,
    virodha_grahas: Vec<Graha>,
    status: ArgalaStatus,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedArgalaRow {
    reference: Sign,
    pairs: Vec<ExpectedArgalaPair>,
    third_house_malefic_argala: bool,
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
    /// En az iki referans burç.
    argala: Vec<ExpectedArgalaRow>,
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

        // Argala.
        assert!(x.argala.len() >= 2, "{id}: at least two references");
        let rows = argala_rows(&p);
        assert_eq!(
            rows[0].reference,
            p.lagna_sign.unwrap_or(Sign::Aries),
            "{id}: first argala row"
        );
        for want in &x.argala {
            let got = rows
                .iter()
                .find(|r| r.reference == want.reference)
                .expect("twelve references");
            let pairs: Vec<ExpectedArgalaPair> = got
                .pairs
                .iter()
                .map(|x| ExpectedArgalaPair {
                    argala_house: x.argala_house,
                    virodha_house: x.virodha_house,
                    argala_grahas: x.argala_grahas.clone(),
                    virodha_grahas: x.virodha_grahas.clone(),
                    status: x.status,
                })
                .collect();
            assert_eq!(pairs, want.pairs, "{id}: argala from {:?}", want.reference);
            assert_eq!(
                got.third_house_malefic_argala, want.third_house_malefic_argala,
                "{id}: third house from {:?}",
                want.reference
            );
        }

        // Rapor aynı parçalardan kurulur.
        let report = jaimini_report(&p, &advanced);
        assert_eq!(report.method, JAIMINI_METHOD, "{id}");
        assert_eq!(report.karakas, karakas, "{id}");
        assert_eq!(report.karakamsha, k, "{id}");
        assert_eq!(report.argala, rows, "{id}");
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
    // Argalanın dört durumu ve 3. ev kötücül argalası.
    for status in [
        ArgalaStatus::None,
        ArgalaStatus::Stands,
        ArgalaStatus::Contested,
        ArgalaStatus::Obstructed,
    ] {
        assert!(
            any(&|f, _| f
                .expected
                .argala
                .iter()
                .any(|r| r.pairs.iter().any(|x| x.status == status))),
            "argala status {status:?}"
        );
    }
    assert!(
        any(&|f, _| f
            .expected
            .argala
            .iter()
            .any(|r| r.third_house_malefic_argala)),
        "three malefics in the third house"
    );
}

/// `lagna` burcundan sayılan `house`. evin burcu (test yardımcısı; motorun `house_sign`'ı
/// yerine elle sayım).
fn house_sign_of(lagna: Sign, house: u8) -> Sign {
    Sign::ALL[(lagna as usize + usize::from(house) - 1) % 12]
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedSecondCycle {
    sign: Sign,
    years: u8,
}

/// Chara dasha el örneği. Doğum saati bilinmeyen haritada `expected` `null`'dır.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CharaDashaExpected {
    /// Dashalar zodyak sırasıyla mı ilerliyor.
    forward: bool,
    /// Dasha sırası, lagna burcundan.
    order: [Sign; 12],
    /// İlk çevrimin yılları, `order` sırasında.
    years: [u8; 12],
    /// İkinci çevrimde üretilen dashalar, sırasıyla (0 yıllık burçlar ve 120. yıldan sonra
    /// başlayanlar yoktur).
    second_cycle: Vec<ExpectedSecondCycle>,
    /// İlk iki dashanın alt dönem sırası.
    sub_periods: [[Sign; 12]; 2],
}

/// El örneklerinde doğum anı ve yıl uzunluğu sabittir; tarihler el yıllarından kurulur.
const CHARA_BIRTH_JD: f64 = 2_451_545.0;
const CHARA_YEAR_DAYS: f64 = 365.25;

fn chara_fixtures() -> Vec<(
    Fixture<Option<CharaDashaExpected>>,
    Positions,
    AdvancedSettings,
)> {
    load::<Option<CharaDashaExpected>>("chara_dasha", MIN_CHARA_DASHA)
        .into_iter()
        .map(|(f, p)| {
            let advanced = f.input.advanced.unwrap_or_else(|| {
                panic!("{}: chara dasha fixtures state the advanced settings", f.id)
            });
            (f, p, advanced)
        })
        .collect()
}

#[test]
fn chara_dasha_hand_fixtures() {
    for (f, p, advanced) in chara_fixtures() {
        let id = &f.id;
        let co_lords = advanced.jaimini_co_lords;
        let got = chara_dasha(&p, co_lords, CHARA_BIRTH_JD, CHARA_YEAR_DAYS);
        let Some(x) = &f.expected else {
            assert!(p.lagna_sign.is_none(), "{id}: null only without a lagna");
            assert_eq!(got, None, "{id}: no dasha without a lagna");
            continue;
        };
        let lagna = p
            .lagna_sign
            .unwrap_or_else(|| panic!("{id}: a lagna is required"));
        let d = got.unwrap_or_else(|| panic!("{id}: a dasha with a lagna"));
        assert_eq!(d.method, CHARA_DASHA_METHOD, "{id}");

        // Yön, sıra ve yıllar.
        assert_eq!(d.forward, x.forward, "{id}: direction");
        assert_eq!(x.order[0], lagna, "{id}: the first dasha is the lagna sign");
        assert_eq!(chara_dasha_order(lagna), x.order, "{id}: order");
        assert_eq!(d.years, x.years, "{id}: years");
        for (&sign, &years) in x.order.iter().zip(&x.years) {
            assert_eq!(
                chara_dasha_sign_years(sign, &p, co_lords),
                years,
                "{id}: years of {sign:?}"
            );
        }

        // Dashalar: ilk çevrimin on iki burcu, sonra ikinci çevrimde üretilenler; tarihler el
        // yıllarının birikimli toplamından.
        let mut want: Vec<(u8, Sign, u8)> = x
            .order
            .iter()
            .zip(&x.years)
            .map(|(&sign, &years)| (1, sign, years))
            .collect();
        want.extend(x.second_cycle.iter().map(|c| (2, c.sign, c.years)));
        let dashas: Vec<&SignDashaPeriod> = d.periods.iter().filter(|q| q.level == 1).collect();
        assert_eq!(dashas.len(), want.len(), "{id}: dasha count");
        let mut elapsed = 0.0;
        for (got, &(cycle, sign, years)) in dashas.iter().zip(&want) {
            assert_eq!(
                (got.cycle, got.signs.as_slice()),
                (cycle, &[sign][..]),
                "{id}: dasha after {elapsed} years"
            );
            let start = CHARA_BIRTH_JD + elapsed * CHARA_YEAR_DAYS;
            elapsed += f64::from(years);
            let end = CHARA_BIRTH_JD + elapsed * CHARA_YEAR_DAYS;
            assert!(
                (got.start_jd_ut - start).abs() < 1e-6 && (got.end_jd_ut - end).abs() < 1e-6,
                "{id}: {sign:?} (cycle {cycle}) runs {} – {}, hand value {start} – {end}",
                got.start_jd_ut,
                got.end_jd_ut
            );
        }

        // İlk iki dashanın alt dönemleri: sıra, on iki eşit parça, 3. seviyede aynı kural.
        for (i, subs) in x.sub_periods.iter().enumerate() {
            let dasha_sign = x.order[i];
            assert_eq!(subs[11], dasha_sign, "{id}: last sub-period");
            let kids = sign_dasha_children(&d, &[dasha_sign], 1).expect("sub-periods");
            let signs: Vec<Sign> = kids.iter().map(|k| k.signs[1]).collect();
            assert_eq!(signs, subs, "{id}: sub-periods of {dasha_sign:?}");
            let part = f64::from(x.years[i]) * CHARA_YEAR_DAYS / 12.0;
            for k in &kids {
                assert!(
                    (k.end_jd_ut - k.start_jd_ut - part).abs() < 1e-6,
                    "{id}: {:?} is one twelfth of the dasha",
                    k.signs
                );
            }
            // 3. seviye: ilk alt dönemin alt dönemleri ondan sonraki burçla başlar (bu, 2.
            // seviyenin ikinci burcudur) ve kendi burcuyla biter.
            let third = sign_dasha_children(&d, &[dasha_sign, subs[0]], 1).expect("third level");
            assert_eq!(third.len(), 12, "{id}");
            assert_eq!(third[0].signs[2], subs[1], "{id}: third level starts");
            assert_eq!(third[11].signs[2], subs[0], "{id}: third level ends");
        }
    }
}

/// Elle yazılmış listeler (motorun `counts_forward` ve `sign_lord` fonksiyonları yerine).
const HAND_FORWARD_GROUP: [Sign; 6] = [
    Sign::Aries,
    Sign::Taurus,
    Sign::Gemini,
    Sign::Libra,
    Sign::Scorpio,
    Sign::Sagittarius,
];
/// Tek lordlu on burcun lordu (Akrep ve Kova eş lordludur, burada yoktur).
const HAND_SINGLE_LORDS: [(Sign, Graha); 10] = [
    (Sign::Aries, Graha::Mars),
    (Sign::Taurus, Graha::Venus),
    (Sign::Gemini, Graha::Mercury),
    (Sign::Cancer, Graha::Moon),
    (Sign::Leo, Graha::Sun),
    (Sign::Virgo, Graha::Mercury),
    (Sign::Libra, Graha::Venus),
    (Sign::Sagittarius, Graha::Jupiter),
    (Sign::Capricorn, Graha::Saturn),
    (Sign::Pisces, Graha::Jupiter),
];

/// Zorunlu durumlar (ileri modüller spec'i §6 "Test"): örnekler silinir ya da değiştirilirse
/// kapsam sessizce daralmasın.
#[test]
fn chara_dasha_fixtures_cover_the_required_cases() {
    let all = chara_fixtures();
    type Case<'a> = (&'a CharaDashaExpected, &'a Positions, &'a AdvancedSettings);
    let any = |pred: &dyn Fn(Case) -> bool| {
        all.iter()
            .any(|(f, p, a)| f.expected.as_ref().is_some_and(|x| pred((x, p, a))))
    };
    assert!(any(&|(x, _, _)| x.forward), "forward direction");
    assert!(any(&|(x, _, _)| !x.forward), "backward direction");
    assert!(
        all.iter().any(|(f, _, _)| f.expected.is_none()),
        "birth time unknown"
    );

    // Tek lordlu bir burçta lordun burçtan `offset` burç ötede olduğu ve yılın `years` olduğu
    // örnek. `offset` 0: kendi burcu; 1: sonraki burç (2. burç); 11: önceki burç (12. burç).
    let lord_case = |forward_group: bool, offset: usize, years: u8| {
        any(&|(x, p, _)| {
            x.order.iter().zip(&x.years).any(|(&sign, &y)| {
                let Some(&(_, lord)) = HAND_SINGLE_LORDS.iter().find(|(s, _)| *s == sign) else {
                    return false;
                };
                HAND_FORWARD_GROUP.contains(&sign) == forward_group
                    && p.sign_of(lord) == Sign::ALL[(sign as usize + offset) % 12]
                    && y == years
            })
        })
    };
    for forward_group in [true, false] {
        assert!(
            lord_case(forward_group, 0, 12),
            "lord in its own sign (forward group: {forward_group})"
        );
    }
    assert!(
        lord_case(true, 1, 1),
        "forward sign, lord in the second: 1 year"
    );
    assert!(
        lord_case(true, 11, 11),
        "forward sign, lord in the twelfth: 11 years"
    );
    assert!(
        lord_case(false, 1, 11),
        "reverse sign, lord in the second: 11 years"
    );
    assert!(
        lord_case(false, 11, 1),
        "reverse sign, lord in the twelfth: 1 year"
    );

    // Akrep ve Kova'da eş lord seçimi: düğümün seçildiği, graha lordun seçildiği, iki eş
    // lordun da burcun içinde olduğu (12 yıl) ve ayarın devre dışı olduğu örnekler.
    let years_of = |x: &CharaDashaExpected, sign: Sign| {
        x.years[x.order.iter().position(|s| *s == sign).expect("in order")]
    };
    for (sign, planet, node) in [
        (Sign::Scorpio, Graha::Mars, Graha::Ketu),
        (Sign::Aquarius, Graha::Saturn, Graha::Rahu),
    ] {
        for lord in [planet, node] {
            assert!(
                any(&|(_, p, a)| a.jaimini_co_lords
                    && p.sign_of(planet) != p.sign_of(node)
                    && co_lord_choice(sign, p).is_some_and(|c| c.lord == lord)),
                "{lord:?} chosen for {sign:?}"
            );
        }
        assert!(
            any(&|(x, p, a)| a.jaimini_co_lords
                && p.sign_of(planet) == sign
                && p.sign_of(node) == sign
                && years_of(x, sign) == 12),
            "both co-lords in {sign:?}: 12 years"
        );
    }
    assert!(
        any(&|(x, p, a)| !a.jaimini_co_lords
            && p.sign_of(Graha::Mars) == Sign::Scorpio
            && p.sign_of(Graha::Ketu) != Sign::Scorpio
            && years_of(x, Sign::Scorpio) == 12),
        "co-lords switched off: Mars in Scorpio gives 12 years"
    );

    // İkinci çevrim: 12 yıllık bir burç atlanır ve ondan sonraki bir burç üretilir; lagna
    // burcunun atlandığı örnek; çevrimin tam 120. yılda ve 120. yıldan sonra bittiği örnekler.
    assert!(
        any(&|(x, _, _)| {
            let skipped = x.years.iter().position(|y| *y == 12).expect("a 12");
            x.second_cycle
                .iter()
                .any(|c| x.order.iter().position(|s| *s == c.sign) > Some(skipped))
        }),
        "a sign skipped inside the second cycle"
    );
    assert!(
        any(&|(x, _, _)| x.years[0] == 12 && x.second_cycle[0].sign != x.order[0]),
        "the lagna sign skipped in the second cycle"
    );
    let total = |x: &CharaDashaExpected| {
        x.years.iter().map(|y| u32::from(*y)).sum::<u32>()
            + x.second_cycle
                .iter()
                .map(|c| u32::from(c.years))
                .sum::<u32>()
    };
    assert!(
        any(&|(x, _, _)| total(x) == 120),
        "ends exactly at 120 years"
    );
    assert!(any(&|(x, _, _)| total(x) > 120), "ends after 120 years");
}

/// Bir grahanın çalışma kâğıdındaki satırı: bütün kalemler, virupa.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedGrahaBala {
    uccha: f64,
    saptavargaja: f64,
    ojayugma: f64,
    kendradi: f64,
    drekkana: f64,
    sthana: f64,
    dig: f64,
    day_night: f64,
    paksha: f64,
    tribhaga: f64,
    year_lord: f64,
    month_lord: f64,
    weekday_lord: f64,
    hora_lord: f64,
    ayana: f64,
    temporal: f64,
    cheshta: f64,
    naisargika: f64,
    drik: f64,
    total: f64,
    rank: u8,
    war: Option<GrahaWar>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedBhavaBala {
    house: u8,
    lord: Graha,
    lord_bala: f64,
    dig: f64,
    drishti: f64,
    total: f64,
    rank: u8,
}

/// Shadbala el örneği (çalışma kâğıdı). Doğum saati bilinmeyen haritada `expected` `null`'dır.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShadbalaExpected {
    grahas: BTreeMap<Graha, ExpectedGrahaBala>,
    time_lords: TimeLords,
    bhavas: Vec<ExpectedBhavaBala>,
}

fn shadbala_fixtures() -> Vec<(Fixture<Option<ShadbalaExpected>>, Positions, ShadbalaAstro)> {
    load::<Option<ShadbalaExpected>>("shadbala", MIN_SHADBALA)
        .into_iter()
        .map(|(mut f, p)| {
            let astro = f
                .input
                .astro
                .take()
                .unwrap_or_else(|| panic!("{}: shadbala fixtures state the astro block", f.id));
            (f, p, astro)
        })
        .collect()
}

#[test]
fn shadbala_hand_fixtures() {
    for (f, p, astro) in shadbala_fixtures() {
        let id = &f.id;
        assert!(astro.is_consistent(), "{id}: astro block");
        let got = shadbala(&p, &astro);
        let Some(x) = &f.expected else {
            assert!(p.lagna_sign.is_none(), "{id}: null only without a lagna");
            assert_eq!(got, None, "{id}: no report without a lagna");
            continue;
        };
        let r = got.unwrap_or_else(|| panic!("{id}: a report with a lagna"));
        assert_eq!(r.method, SHADBALA_METHOD, "{id}");
        assert_eq!(r.time_lords, x.time_lords, "{id}: time lords");
        assert_eq!(x.grahas.len(), 7, "{id}: seven grahas");
        let near = |got: f64, want: f64, what: &str| {
            assert!(
                (got - want).abs() <= WORKSHEET_TOL,
                "{id}: {what}: engine {got}, worksheet {want}"
            );
        };
        for (row, g) in r.grahas.iter().zip(SHADBALA_GRAHAS) {
            let w = &x.grahas[&g];
            let item = |name: &str| format!("{g:?} {name}");
            near(row.sthana.uccha, w.uccha, &item("uccha"));
            near(
                row.sthana.saptavargaja,
                w.saptavargaja,
                &item("saptavargaja"),
            );
            near(row.sthana.ojayugma, w.ojayugma, &item("ojayugma"));
            near(row.sthana.kendradi, w.kendradi, &item("kendradi"));
            near(row.sthana.drekkana, w.drekkana, &item("drekkana"));
            near(row.sthana.total, w.sthana, &item("sthana"));
            near(row.dig, w.dig, &item("dig"));
            near(row.temporal.day_night, w.day_night, &item("day_night"));
            near(row.temporal.paksha, w.paksha, &item("paksha"));
            near(row.temporal.tribhaga, w.tribhaga, &item("tribhaga"));
            near(row.temporal.year_lord, w.year_lord, &item("year_lord"));
            near(row.temporal.month_lord, w.month_lord, &item("month_lord"));
            near(
                row.temporal.weekday_lord,
                w.weekday_lord,
                &item("weekday_lord"),
            );
            near(row.temporal.hora_lord, w.hora_lord, &item("hora_lord"));
            near(row.temporal.ayana, w.ayana, &item("ayana"));
            near(row.temporal.total, w.temporal, &item("temporal"));
            near(row.cheshta, w.cheshta, &item("cheshta"));
            near(row.naisargika, w.naisargika, &item("naisargika"));
            near(row.drik, w.drik, &item("drik"));
            near(row.total, w.total, &item("total"));
            assert_eq!(row.rank, w.rank, "{id}: rank of {g:?}");
            assert_eq!(row.war, w.war, "{id}: war of {g:?}");
        }
        assert_eq!(x.bhavas.len(), 12, "{id}: twelve bhavas");
        for (b, w) in r.bhavas.iter().zip(&x.bhavas) {
            let item = |name: &str| format!("bhava {} {name}", w.house);
            assert_eq!((b.house, b.lord), (w.house, w.lord), "{id}: bhava lord");
            near(b.lord_bala, w.lord_bala, &item("lord_bala"));
            near(b.dig, w.dig, &item("dig"));
            near(b.drishti, w.drishti, &item("drishti"));
            near(b.total, w.total, &item("total"));
            assert_eq!(b.rank, w.rank, "{id}: rank of bhava {}", w.house);
        }
    }
}

/// Çalışma kâğıdının kapsaması gereken durumlar (ileri modüller spec'i §7 "Test": en az üç
/// sentetik girdi): örnekler silinir ya da değiştirilirse kapsam sessizce daralmasın. Denetim
/// motorun fonksiyonlarını değil, örneklerin girdisini ve el değerlerini okur.
#[test]
fn shadbala_fixtures_cover_the_required_cases() {
    let all = shadbala_fixtures();
    let full: Vec<(&ShadbalaExpected, &Positions, &ShadbalaAstro)> = all
        .iter()
        .filter_map(|(f, p, a)| f.expected.as_ref().map(|x| (x, p, a)))
        .collect();
    assert!(full.len() >= 3, "{} worked examples", full.len());
    assert!(
        all.iter().any(|(f, _, _)| f.expected.is_none()),
        "birth time unknown"
    );
    let any = |pred: &dyn Fn(&ShadbalaExpected, &Positions, &ShadbalaAstro) -> bool| {
        full.iter().any(|(x, p, a)| pred(x, p, a))
    };
    // Gündüz ve gece doğumu; öğleye uzaklığın 12 saati aştığı (şafaktan önce) doğum.
    assert!(any(&|_, _, a| a.jd_ut < a.sunset_jd_ut), "day birth");
    assert!(any(&|_, _, a| a.jd_ut >= a.sunset_jd_ut), "night birth");
    assert!(
        any(&|_, _, a| {
            let noon = 0.5 * (a.sunrise_jd_ut + a.sunset_jd_ut);
            (a.jd_ut - noon) * 24.0 > 12.0
        }),
        "more than twelve hours after noon"
    );
    // Büyüyen ve küçülen Ay (Ay − Güneş uzanımı 180°'nin altında ve üstünde).
    let elongation = |p: &Positions| {
        normalize(p.point(Graha::Moon).sidereal_lon - p.point(Graha::Sun).sidereal_lon)
    };
    assert!(any(&|_, p, _| elongation(p) < 180.0), "waxing Moon");
    assert!(any(&|_, p, _| elongation(p) > 180.0), "waning Moon");
    // Graha savaşı: enlemle kazanılan ve eşit enlemli.
    let war = |equal_lat: bool| {
        any(&|x, p, _| {
            x.grahas.iter().any(|(g, row)| {
                row.war
                    .is_some_and(|w| (p.point(*g).lat == p.point(w.with).lat) == equal_lat)
            })
        })
    };
    assert!(war(false), "a war decided by latitude");
    assert!(war(true), "a war between equal latitudes");
    assert!(
        any(&|x, _, _| x.grahas.values().all(|row| row.war.is_none())),
        "an example without a war"
    );
    // Negatif drik bala; moolatrikona (D1'de 45 puan ancak toplam ≥ 45 ile görünür); her
    // kendradi değeri; bhava dig balanın uç değerleri.
    assert!(
        any(&|x, _, _| x.grahas.values().any(|row| row.drik < 0.0)),
        "a negative drik bala"
    );
    for value in [15.0, 30.0, 60.0] {
        assert!(
            any(&|x, _, _| x.grahas.values().any(|row| row.kendradi == value)),
            "kendradi {value}"
        );
    }
    for value in [0.0, 60.0] {
        assert!(
            any(&|x, _, _| x.bhavas.iter().any(|b| b.dig == value)),
            "bhava dig {value}"
        );
    }
    // Akrep lagna (güçsüz bhavası kendisi) ve Yay ya da Oğlak'ın bölünen yarıları.
    assert!(
        any(&|_, p, _| p.lagna_sign == Some(Sign::Scorpio)),
        "Scorpio lagna"
    );
    assert!(
        any(&|_, p, _| p.lagna_sign == Some(Sign::Sagittarius)),
        "Sagittarius lagna"
    );
    // Oranı 1'in altında ve üstünde olan grahalar (gerekli değer el tablosundan).
    let required = [390.0, 360.0, 300.0, 420.0, 390.0, 330.0, 300.0];
    let ratio = |above: bool| {
        any(&|x, _, _| {
            SHADBALA_GRAHAS
                .iter()
                .zip(required)
                .any(|(g, need)| (x.grahas[g].total >= need) == above)
        })
    };
    assert!(ratio(true) && ratio(false), "ratios on both sides of one");
}
