//! İleri modüllerin el örnekleri (ileri modüller spec'i §13): `tests/golden/modules/<modül>/*.json`.
//! Girdi sentetik konumlardır (ephemeris gerekmez); beklenen değer klasik kuraldan elle
//! türetilmiştir ve `derivation` alanında adım adım yazılıdır. Bir örnek motorla uyuşmazsa
//! önce kural yeniden okunur; beklenen değer motor çıktısına bakarak düzeltilmez.

use std::collections::BTreeMap;
use std::path::PathBuf;

use jyotish_core::muhurta::{rahu_segment, yamaganda_segment};
use jyotish_core::{
    argala_rows, arudha_padas, chandra_bala, chara_dasha, chara_dasha_order,
    chara_dasha_sign_years, chara_karakas, co_lord_choice, day_segments, drishti_table, house_lord,
    houses_owned, jaimini_lord, jaimini_report, karakamsha, kuja_dosha, kuta_class_ids,
    kuta_report, moon_is_waxing, nakshatra_nature, nature, normalize, shadbala,
    sign_dasha_children, tara_bala, tithi_group, yoga_rule, yogas, AdvancedSettings, ArgalaStatus,
    ChandraBala, Graha, GrahaWar, Interval, KarakaRole, KujaDosha, Kuta, KutaRole, MoonDayChange,
    MoonPoint, NakshatraNature, NatalMoon, Nature, Positions, ShadbalaAstro, Sign, SignDashaPeriod,
    SunDay, TaraBala, TimeLords, TithiGroup, Vara, YogaRelation, ASPECTING_GRAHAS,
    CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD, KUTA_METHOD, SHADBALA_GRAHAS,
    SHADBALA_METHOD, YOGA_CATALOG, YOGA_CATALOG_VERSION, YOGA_CONDITIONS,
};
use jyotish_core::{
    number_lagna_lon, prashna_facts, rising_mode, CombustionOrbs, CoreError, Dignity, LordLink,
    Paksha, RisingMode, SignKind, PRASHNA_METHOD,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// Modül başına en az örnek sayısı; altına düşerse test düşer (sessiz silinmeye karşı).
const MIN_FOUNDATION: usize = 6;
const MIN_DRISHTI: usize = 4;
const MIN_JAIMINI: usize = 8;
const MIN_CHARA_DASHA: usize = 7;
const MIN_SHADBALA: usize = 4;
const MIN_YOGAS: usize = 15;
const MIN_KUTA: usize = 8;
const MIN_MUHURTA: usize = 7;
const MIN_PRASHNA: usize = 14;
/// El hesabı ile motor arasındaki en büyük fark, virupa.
const VIRUPA_TOL: f64 = 1e-9;
/// Shadbala çalışma kâğıdının toleransı, virupa (ileri modüller spec'i §7 "Test").
const WORKSHEET_TOL: f64 = 0.01;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture<X, I = Input> {
    id: String,
    source: String,
    rule_ref: String,
    derivation: Vec<String>,
    input: I,
    expected: X,
}

/// Sentetik konumlar: sidereal boylamlar, derece. `lagna_lon: null` doğum saati bilinmeyen
/// haritadır. `advanced` yalnızca ayara bağlı modüllerde (Jaimini, chara dasha) verilir;
/// `astro` (saf hesabın astronomik girdisi) ve `lats` (ekliptik enlemler; yazılmayan graha 0°)
/// yalnızca Shadbala örneklerinde; `bride` (gelin rolünün haritası) yalnızca uyum örneklerinde:
/// orada üst düzeydeki `lagna_lon` ve `grahas` damat rolünün haritasıdır.
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
    #[serde(default)]
    bride: Option<PartnerInput>,
}

/// Uyum örneklerinde ikinci harita (gelin rolü): aynı biçimde sentetik konumlar.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PartnerInput {
    lagna_lon: Option<f64>,
    grahas: BTreeMap<Graha, f64>,
}

impl PartnerInput {
    fn positions(&self, id: &str) -> Positions {
        assert_eq!(self.grahas.len(), 9, "{id}: all nine grahas of the bride");
        Positions::synthetic(
            self.lagna_lon,
            std::array::from_fn(|i| self.grahas[&Graha::ALL[i]]),
        )
    }
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

/// Bir modülün el örnekleri, ortak denetimleriyle: en az sayı, dosya adı, kaynak, gönderme ve
/// türetme adımları. `I` girdinin biçimidir (çoğu modülde sentetik konumlar: `Input`).
fn load_raw<X: DeserializeOwned, I: DeserializeOwned>(
    module: &str,
    min: usize,
) -> Vec<Fixture<X, I>> {
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

fn load<X: DeserializeOwned>(module: &str, min: usize) -> Vec<(Fixture<X>, Positions)> {
    load_raw::<X, Input>(module, min)
        .into_iter()
        .map(|f| {
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
    // Moolatrikona (D1'de 45 puan): en az bir graha moolatrikona derece aralığındadır. Aralıklar
    // burada elle yazılıdır (sidereal boylam; sentetik konumlarda rasi çerçevesiyle aynıdır):
    // Güneş Aslan 0°–20°, Ay Boğa 3°–30°, Mars Koç 0°–12°, Merkür Başak 15°–20°, Jüpiter Yay
    // 0°–10°, Venüs Terazi 0°–15°, Satürn Kova 0°–20°.
    let moolatrikona = [
        (Graha::Sun, 120.0, 140.0),
        (Graha::Moon, 33.0, 60.0),
        (Graha::Mars, 0.0, 12.0),
        (Graha::Mercury, 165.0, 170.0),
        (Graha::Jupiter, 240.0, 250.0),
        (Graha::Venus, 180.0, 195.0),
        (Graha::Saturn, 300.0, 320.0),
    ];
    assert!(
        any(&|_, p, _| moolatrikona
            .iter()
            .any(|(g, from, to)| (*from..*to).contains(&p.point(*g).sidereal_lon))),
        "a graha inside its moolatrikona range"
    );
    // Negatif drik bala; her kendradi değeri; bhava dig balanın uç değerleri.
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

/// Bir yoga kaydı: `YogaHit`'in `group` ve `approximate` dışındaki alanları (grup katalogdan,
/// yaklaşık işareti lagnanın yokluğundan gelir; test ikisini de ayrıca denetler).
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct ExpectedYoga {
    id: String,
    grahas: Vec<Graha>,
    houses: Vec<u8>,
    relations: Vec<YogaRelation>,
    conditions: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct YogasExpected {
    /// Raporun bütün kayıtları, katalog sırasıyla.
    hits: Vec<ExpectedYoga>,
    /// Bu örneğin "oluşmayan durum" olarak gösterdiği kurallar (gerekçesi `derivation`'da).
    absent: Vec<String>,
    /// İptal örnekleri: temel koşulu sağlanan ama iptal kuralıyla oluşmayan Kemadruma ve
    /// Shakata; hiçbir koşulu sağlanmayan düşüşteki graha (Neecha bhanga kaydı yok).
    cancelled: Vec<String>,
    evaluated: u32,
    skipped_time_unknown: u32,
}

/// İptal kuralı olan yogalar (ileri modüller spec'i §8 "Kararlar").
const CANCEL_RULES: [&str; 3] = [
    "lunar.kemadruma",
    "lunar.shakata",
    "neecha_bhanga.neecha_bhanga",
];

/// Hand-written debilitation signs of the seven grahas (whole sign).
const HAND_FALL_SIGNS: [(Graha, Sign); 7] = [
    (Graha::Sun, Sign::Libra),
    (Graha::Moon, Sign::Scorpio),
    (Graha::Mars, Sign::Cancer),
    (Graha::Mercury, Sign::Pisces),
    (Graha::Jupiter, Sign::Capricorn),
    (Graha::Venus, Sign::Virgo),
    (Graha::Saturn, Sign::Aries),
];
const HAND_COUNTED: [Graha; 5] = [
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// The base condition of a cancelled example, and the condition that cancels it:
/// Kemadruma: no counted graha in the Moon's sign or in the 2nd and 12th from it, and a counted
/// graha in a kendra from the Moon or from the lagna; Shakata: the Moon in the 6th, 8th or 12th
/// from Jupiter and in a kendra from the lagna; Neecha bhanga: a graha in its debilitation sign
/// (the report lists no record at all, which the caller asserts).
fn cancelled_base_holds(rule: &str, p: &Positions) -> bool {
    let moon = p.sign_of(Graha::Moon);
    match rule {
        "lunar.kemadruma" => {
            let from_moon = |g: Graha| p.house_from(g, moon);
            let alone = HAND_COUNTED
                .iter()
                .all(|&g| ![12, 1, 2].contains(&from_moon(g)));
            let in_a_kendra = HAND_COUNTED.iter().any(|&g| {
                [4, 7, 10].contains(&from_moon(g))
                    || p.house_of(g).is_some_and(|h| [1, 4, 7, 10].contains(&h))
            });
            alone && in_a_kendra
        }
        "lunar.shakata" => {
            [6, 8, 12].contains(&p.house_from(Graha::Moon, p.sign_of(Graha::Jupiter)))
                && p.house_of(Graha::Moon)
                    .is_some_and(|h| [1, 4, 7, 10].contains(&h))
        }
        "neecha_bhanga.neecha_bhanga" => HAND_FALL_SIGNS.iter().any(|&(g, s)| p.sign_of(g) == s),
        _ => false,
    }
}

#[test]
fn yogas_hand_fixtures() {
    for (f, p) in load::<YogasExpected>("yogas", MIN_YOGAS) {
        let id = &f.id;
        let x = &f.expected;
        let report = yogas(&p);
        assert_eq!(report.catalog_version, YOGA_CATALOG_VERSION, "{id}");
        assert_eq!(report.evaluated, x.evaluated, "{id}: evaluated");
        assert_eq!(
            report.skipped_time_unknown, x.skipped_time_unknown,
            "{id}: skipped"
        );
        let got: Vec<ExpectedYoga> = report
            .hits
            .iter()
            .map(|h| ExpectedYoga {
                id: h.id.clone(),
                grahas: h.grahas.clone(),
                houses: h.houses.clone(),
                relations: h.relations.clone(),
                conditions: h.conditions.clone(),
            })
            .collect();
        assert_eq!(got.len(), x.hits.len(), "{id}: hit count, got {got:#?}");
        for (g, want) in got.iter().zip(&x.hits) {
            assert_eq!(g, want, "{id}");
        }
        for h in &report.hits {
            let rule = yoga_rule(&h.id).unwrap_or_else(|| panic!("{id}: {} unknown", h.id));
            assert_eq!(h.group, rule.group, "{id}: {}", h.id);
            // Doğum saati bilinmiyorsa her kayıt yaklaşıktır ve ev taşımaz.
            assert_eq!(h.approximate, p.lagna_sign.is_none(), "{id}: {}", h.id);
            assert!(p.lagna_sign.is_some() || h.houses.is_empty(), "{id}");
        }
        // "Oluşmayan" ve "iptal" örnekleri katalogdadır, değerlendirilmiştir ve raporda yoktur.
        for absent in x.absent.iter().chain(&x.cancelled) {
            let rule = yoga_rule(absent).unwrap_or_else(|| panic!("{id}: {absent} unknown"));
            assert!(
                p.lagna_sign.is_some() || !rule.needs_lagna,
                "{id}: {absent} is skipped, not absent, without a lagna"
            );
            assert!(
                report.hits.iter().all(|h| &h.id != absent),
                "{id}: {absent} is marked absent but the report lists it"
            );
        }
        for cancelled in &x.cancelled {
            assert!(
                CANCEL_RULES.contains(&cancelled.as_str()),
                "{id}: {cancelled} has no cancellation rule"
            );
            // A cancelled case is only an example if the base condition holds in the input and
            // the cancelling condition is what removes it. Second writing of the reading rules
            // in `yogas/REVIEW.md`, from sign positions only.
            assert!(
                cancelled_base_holds(cancelled, &p),
                "{id}: {cancelled} is marked cancelled but its base condition does not hold"
            );
        }
    }
}

/// Spec §8 "Test": her yoga için en az bir oluşan ve bir oluşmayan örnek; iptal kuralı olanlarda
/// ayrıca iptal edilen durum. Örnek silinirse ya da zayıflatılırsa bu test düşer.
#[test]
fn yogas_fixtures_cover_every_rule() {
    let all = load::<YogasExpected>("yogas", MIN_YOGAS);
    let formed = |rule: &str| {
        all.iter()
            .any(|(f, _)| f.expected.hits.iter().any(|h| h.id == rule))
    };
    let absent = |rule: &str| {
        all.iter()
            .any(|(f, _)| f.expected.absent.iter().any(|a| a == rule))
    };
    let cancelled = |rule: &str| {
        all.iter()
            .any(|(f, _)| f.expected.cancelled.iter().any(|a| a == rule))
    };
    assert_eq!(YOGA_CATALOG.len(), 41);
    for rule in YOGA_CATALOG {
        assert!(formed(rule.id), "{}: no fixture where it forms", rule.id);
        assert!(
            absent(rule.id),
            "{}: no fixture where it is absent",
            rule.id
        );
    }
    for rule in CANCEL_RULES {
        assert!(cancelled(rule), "{rule}: no cancelled case");
    }
    // Her alt koşul ve her ilişki türü en az bir örnekte geçer.
    for condition in YOGA_CONDITIONS {
        assert!(
            all.iter().any(|(f, _)| f
                .expected
                .hits
                .iter()
                .any(|h| h.conditions.iter().any(|c| c == condition))),
            "condition {condition}"
        );
    }
    for relation in [
        YogaRelation::Conjunction,
        YogaRelation::MutualAspect,
        YogaRelation::Exchange,
    ] {
        assert!(
            all.iter().any(|(f, _)| f
                .expected
                .hits
                .iter()
                .any(|h| h.relations.contains(&relation))),
            "relation {relation:?}"
        );
    }
    // Bir kayıtta birden çok ilişki; iki koşulu birden taşıyan mahapurusha (Merkür Başak'ta).
    assert!(all
        .iter()
        .any(|(f, _)| f.expected.hits.iter().any(|h| h.relations.len() > 1)));
    assert!(all.iter().any(|(f, _)| f
        .expected
        .hits
        .iter()
        .any(|h| h.id == "mahapurusha.bhadra" && h.conditions.len() == 2)));
    // Doğum saati bilinmeyen örnekler: 17 kural atlanır, 24 kural değerlendirilir.
    let unknown: Vec<_> = all.iter().filter(|(_, p)| p.lagna_sign.is_none()).collect();
    assert!(unknown.len() >= 2, "time-unknown fixtures");
    for (f, _) in unknown {
        assert_eq!(
            (f.expected.evaluated, f.expected.skipped_time_unknown),
            (24, 17),
            "{}",
            f.id
        );
    }
}

/// Bir kutanın beklenen puanı ve iki tarafın sınıf kimliği (`max` kutanın sırasından gelir).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedKuta {
    kuta: Kuta,
    score: f64,
    groom_class: String,
    bride_class: String,
}

/// Karşılıklı yerleşim: her tarafın dokuz grahası (`Graha::ALL` sırasında) için
/// `[diğer tarafın lagnasından ev | null, diğer tarafın Ay'ından ev]`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedOverlay {
    groom: Vec<(Option<u8>, u8)>,
    bride: Vec<(Option<u8>, u8)>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct KutaExpected {
    /// Sekiz kuta, tablo sırasıyla.
    scores: Vec<ExpectedKuta>,
    total: f64,
    bhakoot_exception: bool,
    nadi_exception: bool,
    /// `[damat rolü, gelin rolü]`.
    kuja_dosha: [KujaDosha; 2],
    overlay: ExpectedOverlay,
    approximate: bool,
}

/// Uyum örnekleri: damat rolünün konumları, gelin rolünün konumları.
fn kuta_fixtures() -> Vec<(Fixture<KutaExpected>, Positions, Positions)> {
    load::<KutaExpected>("kuta", MIN_KUTA)
        .into_iter()
        .map(|(f, groom)| {
            let bride = f
                .input
                .bride
                .as_ref()
                .unwrap_or_else(|| panic!("{}: kuta fixtures need the bride block", f.id))
                .positions(&f.id);
            (f, groom, bride)
        })
        .collect()
}

#[test]
fn kuta_hand_fixtures() {
    for (f, groom, bride) in kuta_fixtures() {
        let id = &f.id;
        let x = &f.expected;
        let r = kuta_report(&groom, &bride);
        assert_eq!(r.method, KUTA_METHOD, "{id}");
        assert_eq!(x.scores.len(), 8, "{id}: eight kutas");
        for (i, (got, want)) in r.scores.iter().zip(&x.scores).enumerate() {
            assert_eq!(want.kuta, Kuta::ALL[i], "{id}: fixture order");
            assert_eq!(got.kuta, want.kuta, "{id}");
            assert_eq!(got.score, want.score, "{id}: {:?}", want.kuta);
            assert_eq!(got.max, (i + 1) as f64, "{id}: {:?} max", want.kuta);
            assert_eq!(got.groom_class, want.groom_class, "{id}: {:?}", want.kuta);
            assert_eq!(got.bride_class, want.bride_class, "{id}: {:?}", want.kuta);
            for class in [&want.groom_class, &want.bride_class] {
                assert!(
                    kuta_class_ids(want.kuta).contains(class),
                    "{id}: {:?} class {class}",
                    want.kuta
                );
            }
        }
        // Toplam, el örneğindeki sekiz kalemin toplamıdır (½ katları: tam eşitlik).
        assert_eq!(
            x.total,
            x.scores.iter().map(|s| s.score).sum::<f64>(),
            "{id}"
        );
        assert_eq!(r.total, x.total, "{id}: total");
        assert_eq!(r.bhakoot_exception, x.bhakoot_exception, "{id}: bhakoot");
        assert_eq!(r.nadi_exception, x.nadi_exception, "{id}: nadi");
        assert_eq!(r.kuja_dosha, x.kuja_dosha, "{id}: kuja dosha");
        assert_eq!(r.approximate, x.approximate, "{id}: approximate");
        assert_eq!(
            x.approximate,
            groom.lagna_sign.is_none() || bride.lagna_sign.is_none(),
            "{id}"
        );
        // Saf hesap Ay'ın gün içindeki değişimini bilmez; onu toplayıcı doldurur.
        assert_eq!(r.moon_day_change, [MoonDayChange::default(); 2], "{id}");
        // Karşılıklı yerleşim: 9 + 9 satır.
        assert_eq!(
            (x.overlay.groom.len(), x.overlay.bride.len()),
            (9, 9),
            "{id}"
        );
        let want_rows = x.overlay.groom.iter().chain(&x.overlay.bride);
        for (i, (row, want)) in r.overlay.iter().zip(want_rows).enumerate() {
            let (owner, own) = if i < 9 {
                (KutaRole::Groom, &groom)
            } else {
                (KutaRole::Bride, &bride)
            };
            let graha = Graha::ALL[i % 9];
            assert_eq!((row.owner, row.graha), (owner, graha), "{id}");
            assert_eq!(row.sign, own.sign_of(graha), "{id}: {graha:?}");
            assert_eq!(
                (row.house_from_lagna, row.house_from_moon),
                *want,
                "{id}: {owner:?} {graha:?}"
            );
        }
    }
}

#[test]
fn kuta_fixtures_cover_the_required_cases() {
    let all = kuta_fixtures();
    let score = |f: &Fixture<KutaExpected>, kuta: Kuta| f.expected.scores[kuta as usize].score;
    // Her kuta en az bir örnekte en çok puanını, en az bir örnekte en düşük puanını alır.
    // Tara'nın en düşük puanı 1½'dir: iki yön birden uygunsuz olamaz.
    for kuta in Kuta::ALL {
        let min = if kuta == Kuta::Tara { 1.5 } else { 0.0 };
        assert!(
            all.iter().any(|(f, ..)| score(f, kuta) == kuta.max()),
            "{kuta:?}: no fixture with the maximum"
        );
        assert!(
            all.iter().any(|(f, ..)| score(f, kuta) == min),
            "{kuta:?}: no fixture with the minimum"
        );
    }
    // Yarım puanlı kalemler: vashya ½, graha maitri ½.
    assert!(all.iter().any(|(f, ..)| score(f, Kuta::Vashya) == 0.5));
    assert!(all.iter().any(|(f, ..)| score(f, Kuta::GrahaMaitri) == 0.5));
    // Toplamın iki ucu: 36 ve 2½ (108 × 108 pada çiftinin en düşüğü).
    assert!(all.iter().any(|(f, ..)| f.expected.total == 36.0));
    assert!(all.iter().any(|(f, ..)| f.expected.total == 2.5));
    // İstisnalar: var; puan 0 iken yok.
    let x = |f: &Fixture<KutaExpected>| (f.expected.bhakoot_exception, f.expected.nadi_exception);
    assert!(all.iter().any(|(f, ..)| x(f).0), "bhakoot exception");
    assert!(all.iter().any(|(f, ..)| x(f).1), "nadi exception");
    assert!(all
        .iter()
        .any(|(f, ..)| score(f, Kuta::Bhakoot) == 0.0 && !x(f).0));
    assert!(all
        .iter()
        .any(|(f, ..)| score(f, Kuta::Nadi) == 0.0 && !x(f).1));
    // Nadi istisnasının iki yolu: aynı nakshatranın farklı padaları; aynı burcun farklı
    // nakshatraları.
    let moons = |g: &Positions, b: &Positions| (MoonPoint::of(g), MoonPoint::of(b));
    assert!(all.iter().any(|(f, g, b)| {
        let (g, b) = moons(g, b);
        x(f).1 && g.nakshatra == b.nakshatra && g.pada != b.pada
    }));
    assert!(all.iter().any(|(f, g, b)| {
        let (g, b) = moons(g, b);
        x(f).1 && g.nakshatra != b.nakshatra && g.sign == b.sign
    }));
    // Aynı Ay (aynı nakshatra, aynı pada): nadi 0, yoni 4, bhakoot 7, maitri 5; istisna yok.
    assert!(all.iter().any(|(f, g, b)| {
        let (g, b) = moons(g, b);
        (g.nakshatra, g.pada) == (b.nakshatra, b.pada)
            && score(f, Kuta::Nadi) == 0.0
            && score(f, Kuta::Yoni) == 4.0
            && score(f, Kuta::Bhakoot) == 7.0
            && score(f, Kuta::GrahaMaitri) == 5.0
            && !x(f).1
    }));
    // Vashya'nın bölünen iki burcunun dört yarısı da örneklerde geçer.
    let half = |sign: Sign, first: bool| {
        all.iter().any(|(_, g, b)| {
            [MoonPoint::of(g), MoonPoint::of(b)]
                .iter()
                .any(|m| m.sign == sign && (m.deg_in_sign < 15.0) == first)
        })
    };
    for sign in [Sign::Sagittarius, Sign::Capricorn] {
        assert!(half(sign, true) && half(sign, false), "{sign:?} halves");
    }
    // Doğum saati bilinmeyen harita: bir örnekte damat rolünde, bir örnekte gelin rolünde.
    assert!(all
        .iter()
        .any(|(_, g, b)| g.lagna_sign.is_none() && b.lagna_sign.is_some()));
    assert!(all
        .iter()
        .any(|(_, g, b)| g.lagna_sign.is_some() && b.lagna_sign.is_none()));
    // Kuja dosha: üç referansın her biri en az bir haritada var ve en az birinde yok.
    let doshas: Vec<KujaDosha> = all
        .iter()
        .flat_map(|(f, ..)| f.expected.kuja_dosha)
        .collect();
    for (name, read) in [
        (
            "lagna",
            (|d: &KujaDosha| d.from_lagna) as fn(&KujaDosha) -> Option<bool>,
        ),
        ("moon", |d: &KujaDosha| Some(d.from_moon)),
        ("venus", |d: &KujaDosha| Some(d.from_venus)),
    ] {
        for want in [true, false] {
            assert!(
                doshas.iter().any(|d| read(d) == Some(want)),
                "kuja dosha from {name}: {want}"
            );
        }
    }
}

/// Muhurta örneklerinin girdisi: sentetik bir gün (dakika olarak gün doğumu, batımı ve sonraki
/// gün doğumu; varası), o günün tithisi ve Ay'ı, ve doğum Ay'ı. Konum ve ephemeris yoktur.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MuhurtaInput {
    day: MuhurtaDay,
    tithi: u8,
    moon: NatalMoon,
    natal: NatalMoon,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MuhurtaDay {
    vara: Vara,
    sunrise_min: f64,
    sunset_min: f64,
    next_sunrise_min: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedSegment {
    segment: u8,
    from_min: f64,
    to_min: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedSpan {
    from_min: f64,
    to_min: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MuhurtaExpected {
    eighth_min: f64,
    rahu: ExpectedSegment,
    yamaganda: ExpectedSegment,
    gulika: ExpectedSegment,
    abhijit: ExpectedSpan,
    abhijit_weekday_exception: bool,
    hora_min: f64,
    hora_lords: Vec<Graha>,
    tithi_group: TithiGroup,
    nakshatra_nature: NakshatraNature,
    tara: TaraBala,
    chandra: ChandraBala,
}

const MINUTES_PER_DAY: f64 = 1440.0;
/// El hesabı ile motor arasındaki en büyük fark, dakika.
const MINUTE_TOL: f64 = 1e-6;

fn muhurta_fixtures() -> Vec<Fixture<MuhurtaExpected, MuhurtaInput>> {
    load_raw("muhurta", MIN_MUHURTA)
}

#[test]
fn muhurta_hand_fixtures() {
    for f in muhurta_fixtures() {
        let id = &f.id;
        let (i, x) = (&f.input, &f.expected);
        // Dakika değerleri gün kesrine çevrilir (sentetik gün; Jülyen günü değildir).
        let s = day_segments(&SunDay {
            sunrise_jd_ut: i.day.sunrise_min / MINUTES_PER_DAY,
            sunset_jd_ut: i.day.sunset_min / MINUTES_PER_DAY,
            next_sunrise_jd_ut: i.day.next_sunrise_min / MINUTES_PER_DAY,
            vara: i.day.vara,
        });
        let minutes = |t: f64| t * MINUTES_PER_DAY;
        let close = |got: f64, want: f64, what: &str| {
            assert!(
                (minutes(got) - want).abs() < MINUTE_TOL,
                "{id}: {what}: {} vs {want}",
                minutes(got)
            );
        };
        assert_eq!(s.eighths.len(), 8, "{id}");
        for e in &s.eighths {
            close(e.end_jd_ut - e.start_jd_ut, x.eighth_min, "eighth");
        }
        let segment = |got: &Interval<u8>, want: &ExpectedSegment, what: &str| {
            assert_eq!(got.value, want.segment, "{id}: {what} segment");
            close(got.start_jd_ut, want.from_min, what);
            close(got.end_jd_ut, want.to_min, what);
        };
        segment(&s.rahu, &x.rahu, "rahu");
        segment(&s.yamaganda, &x.yamaganda, "yamaganda");
        segment(&s.gulika, &x.gulika, "gulika");
        assert_eq!(s.abhijit.value, 8, "{id}: abhijit is the eighth muhurta");
        close(s.abhijit.start_jd_ut, x.abhijit.from_min, "abhijit");
        close(s.abhijit.end_jd_ut, x.abhijit.to_min, "abhijit");
        assert_eq!(
            s.abhijit_weekday_exception, x.abhijit_weekday_exception,
            "{id}: abhijit weekday exception"
        );
        assert_eq!(x.hora_lords.len(), 24, "{id}: twenty-four hora lords");
        let lords: Vec<Graha> = s.horas.iter().map(|h| h.value).collect();
        assert_eq!(lords, x.hora_lords, "{id}: hora lords");
        for h in &s.horas {
            close(h.end_jd_ut - h.start_jd_ut, x.hora_min, "hora");
        }
        close(s.horas[0].start_jd_ut, i.day.sunrise_min, "first hora");
        close(s.horas[23].end_jd_ut, i.day.next_sunrise_min, "last hora");
        assert_eq!(tithi_group(i.tithi), x.tithi_group, "{id}: tithi group");
        assert_eq!(
            nakshatra_nature(i.moon.nakshatra),
            x.nakshatra_nature,
            "{id}: nakshatra nature"
        );
        assert_eq!(
            tara_bala(i.natal.nakshatra, i.moon.nakshatra),
            x.tara,
            "{id}: tara bala"
        );
        assert_eq!(
            chandra_bala(i.natal.sign, i.moon.sign),
            x.chandra,
            "{id}: chandra bala"
        );
    }
}

/// Örnekler yedi varanın her birini, beş tithi grubunu, yedi nakshatra doğasını, elverişli ve
/// elverişsiz tara ile chandra balayı, Çarşamba istisnasını ve 24 saatten kısa, 24 saate eşit ve
/// 24 saatten uzun günü kapsar; spec §10'un örneği (06:00–18:00 Pazartesi: Rahu 07:30–09:00,
/// Abhijit 11:36–12:24) aralarındadır.
#[test]
fn muhurta_fixtures_cover_the_required_cases() {
    let fixtures = muhurta_fixtures();
    for vara in Vara::ALL {
        let of_day: Vec<_> = fixtures
            .iter()
            .filter(|f| f.input.day.vara == vara)
            .collect();
        assert!(!of_day.is_empty(), "no fixture for {vara:?}");
        for f in of_day {
            assert_eq!(f.expected.rahu.segment, rahu_segment(vara), "{}", f.id);
            assert_eq!(
                f.expected.yamaganda.segment,
                yamaganda_segment(vara),
                "{}",
                f.id
            );
        }
    }
    for group in TithiGroup::ALL {
        assert!(
            fixtures.iter().any(|f| f.expected.tithi_group == group),
            "no fixture with {group:?}"
        );
    }
    for nature in NakshatraNature::ALL {
        assert!(
            fixtures
                .iter()
                .any(|f| f.expected.nakshatra_nature == nature),
            "no fixture with {nature:?}"
        );
    }
    for favourable in [true, false] {
        assert!(fixtures
            .iter()
            .any(|f| f.expected.tara.favourable == favourable));
        assert!(fixtures
            .iter()
            .any(|f| f.expected.chandra.favourable == favourable));
    }
    assert!(fixtures
        .iter()
        .any(|f| f.expected.abhijit_weekday_exception));
    assert!(
        fixtures.iter().any(|f| f.input.tithi > 15),
        "a krishna tithi"
    );
    for want in [59.0, 60.0, 61.0] {
        assert!(
            fixtures.iter().any(|f| f.expected.hora_min == want),
            "no fixture with {want}-minute horas"
        );
    }
    let spec = fixtures
        .iter()
        .find(|f| {
            f.input.day.vara == Vara::Monday
                && f.input.day.sunrise_min == 360.0
                && f.input.day.sunset_min == 1080.0
        })
        .expect("the six-to-six Monday of the design document");
    assert_eq!(
        (spec.expected.rahu.from_min, spec.expected.rahu.to_min),
        (450.0, 540.0)
    );
    assert_eq!(
        (spec.expected.abhijit.from_min, spec.expected.abhijit.to_min),
        (696.0, 744.0)
    );
}

/// Prashna örneklerinin girdisi: sentetik konumlar ve sorunun seçimleri. Lagna ya `lagna_lon`
/// ile verilir ya da `number` ile (sayı prashnası: lagna sayıdan türetilir); ikisi birlikte
/// yazılmaz. `retrograde` listesindeki grahaların hızı eksidir.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrashnaInput {
    lagna_lon: Option<f64>,
    number: Option<u8>,
    focus_house: Option<u8>,
    grahas: BTreeMap<Graha, f64>,
    retrograde: Vec<Graha>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct ExpectedLord {
    graha: Graha,
    sign: Sign,
    house: u8,
    dignity: Option<Dignity>,
    retrograde: bool,
    combust: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedKendra {
    house: u8,
    sign: Sign,
    grahas: Vec<(Graha, Nature)>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedLagna {
    sign: Sign,
    /// Lagnanın burç içi derecesi (sayı prashnasında navamsanın ortası).
    deg_in_sign: f64,
    kind: SignKind,
    rising: RisingMode,
    lord: ExpectedLord,
    kendras: Vec<ExpectedKendra>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedFocus {
    house: u8,
    sign: Sign,
    lord: ExpectedLord,
    same_graha: bool,
    links: Vec<LordLink>,
    focus_lord_from_lagna_lord: u8,
    lagna_lord_from_focus_lord: u8,
    occupants: Vec<(Graha, Nature)>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedMoon {
    sign: Sign,
    house: Option<u8>,
    nakshatra: u8,
    pada: u8,
    tithi: u8,
    paksha: Paksha,
    aspected_by: Vec<Graha>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrashnaExpected {
    lagna: Option<ExpectedLagna>,
    focus: Option<ExpectedFocus>,
    moon: ExpectedMoon,
}

/// Lagnanın burç içi derecesinde el hesabı ile motor arasındaki en büyük fark.
const LAGNA_DEG_TOL: f64 = 1e-9;

fn prashna_fixtures() -> Vec<(Fixture<PrashnaExpected, PrashnaInput>, Positions)> {
    load_raw::<PrashnaExpected, PrashnaInput>("prashna", MIN_PRASHNA)
        .into_iter()
        .map(|f| {
            let id = &f.id;
            let i = &f.input;
            assert_eq!(i.grahas.len(), 9, "{id}: all nine grahas are required");
            assert!(
                i.lagna_lon.is_none() || i.number.is_none(),
                "{id}: the lagna comes from the longitude or from the number, not both"
            );
            let lagna = match i.number {
                Some(n) => Some(number_lagna_lon(n).unwrap_or_else(|e| panic!("{id}: {e}"))),
                None => i.lagna_lon,
            };
            let mut positions =
                Positions::synthetic(lagna, std::array::from_fn(|k| i.grahas[&Graha::ALL[k]]));
            for g in &i.retrograde {
                positions.grahas[*g as usize].speed = -1.0;
            }
            (f, positions)
        })
        .collect()
}

fn lord_of(l: &jyotish_core::LordFacts) -> ExpectedLord {
    ExpectedLord {
        graha: l.graha,
        sign: l.sign,
        house: l.house,
        dignity: l.dignity,
        retrograde: l.retrograde,
        combust: l.combust,
    }
}

fn occupants_of(list: &[jyotish_core::Occupant]) -> Vec<(Graha, Nature)> {
    list.iter().map(|o| (o.graha, o.nature)).collect()
}

#[test]
fn prashna_hand_fixtures() {
    // Yanma orbları v1'in varsayılanlarıdır (el örnekleri bu orblarla türetilmiştir).
    let orbs = CombustionOrbs::default();
    for (f, positions) in prashna_fixtures() {
        let id = &f.id;
        let got = prashna_facts(&positions, &orbs, f.input.focus_house)
            .unwrap_or_else(|e| panic!("{id}: {e}"));
        assert_eq!(got.method, PRASHNA_METHOD, "{id}");
        assert!(got.warnings.is_empty(), "{id}");
        assert_eq!(got.yogas, yogas(&positions), "{id}: yoga list");

        match (&f.expected.lagna, &got.lagna) {
            (None, None) => assert!(positions.lagna_lon.is_none(), "{id}"),
            (Some(want), Some(have)) => {
                assert_eq!(
                    (have.sign, have.kind, have.rising),
                    (want.sign, want.kind, want.rising),
                    "{id}: lagna"
                );
                let lon = positions.lagna_lon.expect("lagna");
                assert_eq!(Sign::of(lon), want.sign, "{id}: lagna sign");
                assert!(
                    (jyotish_core::deg_in_sign(lon) - want.deg_in_sign).abs() < LAGNA_DEG_TOL,
                    "{id}: lagna degree"
                );
                assert_eq!(lord_of(&have.lord), want.lord, "{id}: lagna lord");
                assert_eq!(have.kendras.len(), 4, "{id}");
                assert_eq!(want.kendras.len(), 4, "{id}: four kendras");
                for (h, w) in have.kendras.iter().zip(&want.kendras) {
                    assert_eq!((h.house, h.sign), (w.house, w.sign), "{id}: kendra");
                    assert_eq!(occupants_of(&h.grahas), w.grahas, "{id}: house {}", w.house);
                }
            }
            (want, have) => panic!(
                "{id}: lagna expected {}, got {}",
                want.is_some(),
                have.is_some()
            ),
        }

        match (&f.expected.focus, &got.focus) {
            (None, None) => {}
            (Some(want), Some(have)) => {
                assert_eq!(Some(want.house), f.input.focus_house, "{id}");
                assert_eq!(
                    (have.house, have.sign),
                    (want.house, want.sign),
                    "{id}: focus"
                );
                assert_eq!(lord_of(&have.lord), want.lord, "{id}: focus lord");
                assert_eq!(have.relation.same_graha, want.same_graha, "{id}");
                assert_eq!(have.relation.links, want.links, "{id}: links");
                assert_eq!(
                    (
                        have.relation.focus_lord_from_lagna_lord,
                        have.relation.lagna_lord_from_focus_lord
                    ),
                    (
                        want.focus_lord_from_lagna_lord,
                        want.lagna_lord_from_focus_lord
                    ),
                    "{id}: counts"
                );
                assert_eq!(occupants_of(&have.occupants), want.occupants, "{id}");
            }
            (want, have) => panic!(
                "{id}: focus expected {}, got {}",
                want.is_some(),
                have.is_some()
            ),
        }

        let (m, w) = (&got.moon, &f.expected.moon);
        assert_eq!(
            (m.sign, m.house, m.nakshatra, m.pada, m.tithi, m.paksha),
            (w.sign, w.house, w.nakshatra, w.pada, w.tithi, w.paksha),
            "{id}: moon"
        );
        assert_eq!(m.aspected_by, w.aspected_by, "{id}: aspects on the moon");
    }
}

/// Sayı → lagna eşlemesinin elle yazılmış ikinci tablosu (spec §11 "Test"): sayı, burç, derece,
/// dakika. Örneklerin `deg_in_sign` değeri de bu tabloyla karşılaştırılır.
const HAND_NUMBER_LAGNA: [(u8, Sign, f64, f64); 5] = [
    (1, Sign::Aries, 1.0, 40.0),
    (9, Sign::Aries, 28.0, 20.0),
    (10, Sign::Taurus, 1.0, 40.0),
    (54, Sign::Virgo, 28.0, 20.0),
    (108, Sign::Pisces, 28.0, 20.0),
];

/// On iki burcun doğuş biçiminin elle yazılmış ikinci tablosu (spec §11), burç sırasıyla.
const HAND_RISING: [(Sign, RisingMode); 12] = [
    (Sign::Aries, RisingMode::BackFirst),
    (Sign::Taurus, RisingMode::BackFirst),
    (Sign::Gemini, RisingMode::HeadFirst),
    (Sign::Cancer, RisingMode::BackFirst),
    (Sign::Leo, RisingMode::HeadFirst),
    (Sign::Virgo, RisingMode::HeadFirst),
    (Sign::Libra, RisingMode::HeadFirst),
    (Sign::Scorpio, RisingMode::HeadFirst),
    (Sign::Sagittarius, RisingMode::BackFirst),
    (Sign::Capricorn, RisingMode::BackFirst),
    (Sign::Aquarius, RisingMode::HeadFirst),
    (Sign::Pisces, RisingMode::BothWays),
];

#[test]
fn prashna_fixtures_cover_the_required_cases() {
    let fixtures = prashna_fixtures();
    // Sayı → lagna: 1, 9, 10, 54, 108; sınır dışı sayılar hatadır.
    for (n, sign, deg, min) in HAND_NUMBER_LAGNA {
        let (f, _) = fixtures
            .iter()
            .find(|(f, _)| f.input.number == Some(n))
            .unwrap_or_else(|| panic!("no fixture with the number {n}"));
        let lagna = f.expected.lagna.as_ref().expect("a number gives a lagna");
        assert_eq!(lagna.sign, sign, "{}", f.id);
        assert!(
            (lagna.deg_in_sign - (deg + min / 60.0)).abs() < LAGNA_DEG_TOL,
            "{}",
            f.id
        );
    }
    for bad in [0, 109] {
        assert_eq!(
            number_lagna_lon(bad),
            Err(CoreError::InvalidPrashnaInput("number"))
        );
    }
    // On iki burcun doğuş biçimi: her burç bir örneğin lagnasıdır ve el tablosuyla aynıdır.
    for (sign, rising) in HAND_RISING {
        assert_eq!(rising_mode(sign), rising, "{sign:?}");
        let (f, _) = fixtures
            .iter()
            .find(|(f, _)| f.expected.lagna.as_ref().is_some_and(|l| l.sign == sign))
            .unwrap_or_else(|| panic!("no fixture with the lagna {sign:?}"));
        assert_eq!(
            f.expected.lagna.as_ref().unwrap().rising,
            rising,
            "{}",
            f.id
        );
    }
    // Lord ilişkilerinin beş türü (tek yönlü bakışın iki yönü ayrı), "hiçbiri" ve aynı graha.
    let focuses: Vec<&ExpectedFocus> = fixtures
        .iter()
        .filter_map(|(f, _)| f.expected.focus.as_ref())
        .collect();
    for link in LordLink::ALL {
        assert!(
            focuses.iter().any(|f| f.links.contains(&link)),
            "no fixture with {link:?}"
        );
    }
    assert!(focuses.iter().any(|f| f.links.is_empty() && !f.same_graha));
    assert!(focuses.iter().any(|f| f.same_graha));
    assert!(
        focuses.iter().any(|f| f.links.len() > 1),
        "two links at once"
    );
    // Odak evi seçilmemiş ve lagnasız (saat bilinmiyor) örnekler.
    assert!(fixtures
        .iter()
        .any(|(f, _)| f.input.focus_house.is_none() && f.expected.lagna.is_some()));
    assert!(fixtures.iter().any(|(f, _)| {
        f.expected.lagna.is_none() && f.expected.focus.is_none() && f.expected.moon.house.is_none()
    }));
    // Retro ve yanık lord; iki paksha; kendrada iki sınıftan graha ve düğüm.
    let lords: Vec<&ExpectedLord> = fixtures
        .iter()
        .filter_map(|(f, _)| f.expected.lagna.as_ref())
        .map(|l| &l.lord)
        .chain(focuses.iter().map(|f| &f.lord))
        .collect();
    assert!(lords.iter().any(|l| l.retrograde));
    assert!(lords.iter().any(|l| l.combust));
    for want in [
        Dignity::Exalted,
        Dignity::Debilitated,
        Dignity::Moolatrikona,
        Dignity::OwnSign,
    ] {
        assert!(lords.iter().any(|l| l.dignity == Some(want)), "{want:?}");
    }
    for paksha in [Paksha::Shukla, Paksha::Krishna] {
        assert!(fixtures
            .iter()
            .any(|(f, _)| f.expected.moon.paksha == paksha));
    }
    assert!(fixtures
        .iter()
        .any(|(f, _)| !f.expected.moon.aspected_by.is_empty()));
    let in_kendras: Vec<(Graha, Nature)> = fixtures
        .iter()
        .filter_map(|(f, _)| f.expected.lagna.as_ref())
        .flat_map(|l| l.kendras.iter().flat_map(|k| k.grahas.clone()))
        .collect();
    for nature in [Nature::Benefic, Nature::Malefic] {
        assert!(in_kendras.iter().any(|(_, n)| *n == nature));
    }
    assert!(in_kendras.contains(&(Graha::Mercury, Nature::Malefic)));
    assert!(in_kendras.iter().any(|(g, _)| g.is_node()));
}
