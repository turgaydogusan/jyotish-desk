//! Bhava tablosu: on iki bhavanın başlangıcı, ortası ve bitişi; her grahanın bhavası ve bhava
//! başlangıcına ve ortasına uzaklığı. Boylamlar rasi çerçevesindedir (K5); nakshatra ve pada
//! sidereal boylamdan okunur.
//!
//! Okuma (yol haritası §4.5; `tests/golden/modules/bhava/REVIEW.md`):
//! - **Tam burç:** bhava burcun kendisidir (başlangıç 0°, bitiş 30°). Orta, lagnanın derecesinin
//!   o burçtaki karşılığıdır (lagna + 30° × (n − 1)); Bhava bala da aynı noktayı kullanır.
//! - **Sripati:** ephemeris'in döndürdüğü noktalar bhava başlangıcıdır (sandhi). Orta, Porphyry
//!   noktasıdır: lagna ile MC arasındaki çeyrekler üçe bölünür; 1. bhavanın ortası lagna, 10.
//!   bhavanın ortası MC'dir. Her sandhi iki komşu ortanın tam ortasıdır.
//! - **Placidus:** nokta bhavanın başlangıcıdır (KP okuması); bhava ortası tanımlanmaz (`None`).

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{compute_chart_for, finite, rasi_frame, to_sidereal, ChartWarning};
use crate::dasha::VIMSHOTTARI_ORDER;
use crate::dignity::sign_lord;
use crate::graha::Graha;
use crate::houses::house_from_cusps;
use crate::positions::ChartInput;
use crate::settings::{HouseSystem, ZodiacMode};
use crate::zodiac::{deg_in_sign, nakshatra_pada, normalize, Sign};
use crate::CoreError;

/// Sonucun yöntem kimliği (K7).
pub const BHAVA_METHOD: &str = "bhava_table.v1";

/// Bir bhava noktası (başlangıç, orta ya da bitiş).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct BhavaPoint {
    /// Rasi çerçevesinde boylam, [0, 360).
    pub lon: f64,
    pub sign: Sign,
    /// Burç içi derece, [0, 30).
    pub deg_in_sign: f64,
    /// 1..=27; sidereal boylamdan.
    pub nakshatra: u8,
    /// 1..=4.
    pub pada: u8,
    pub sign_lord: Graha,
    pub nakshatra_lord: Graha,
}

/// Bir bhava: başlangıç, orta ve bitiş. Bitiş, sonraki bhavanın başlangıcıdır.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct BhavaRow {
    /// 1..=12.
    pub house: u8,
    pub start: BhavaPoint,
    /// Placidus'ta `None`.
    pub mid: Option<BhavaPoint>,
    pub end: BhavaPoint,
    /// Başlangıçtan bitişe yay, derece, (0, 360).
    pub span_deg: f64,
}

/// Bir grahanın bhavası.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GrahaBhava {
    pub graha: Graha,
    /// Lagna burcundan sayılan burç evi (K4), 1..=12.
    pub sign_house: u8,
    /// Seçili ev sistemindeki bhava, 1..=12.
    pub bhava: u8,
    /// Bhava başlangıcından grahaya yay, derece, [0, bhavanın yayı).
    pub from_start_deg: f64,
    /// Bhava ortasından grahaya işaretli yay, derece, (−180, 180]: eksi, graha ortadan öncedir.
    /// Placidus'ta `None`.
    pub from_mid_deg: Option<f64>,
    /// Bhava, burç evinden farklı.
    pub shifted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct BhavaTable {
    pub method: String,
    /// Tablonun ev sistemi (`Chart.bhavas.system`): sayı prashnasında ayar ne olursa olsun
    /// `WholeSign`.
    pub system: HouseSystem,
    /// Placidus bu enlemde hesaplanamadı; başlangıç noktaları Porphyry'dir.
    pub porphyry_fallback: bool,
    /// On iki bhava, 1. bhavadan başlayarak.
    pub rows: Vec<BhavaRow>,
    /// Dokuz graha, `Graha::ALL` sırasında.
    pub grahas: Vec<GrahaBhava>,
}

/// Nakshatranın (1..=27) Vimshottari lordu: Ashwini'den başlayan dokuzlu sıra üç kez yinelenir.
pub fn nakshatra_lord(nakshatra: u8) -> Graha {
    VIMSHOTTARI_ORDER[(usize::from(nakshatra) + 8) % 9]
}

/// `from` boylamından `to` boylamına işaretli kısa yay, derece, (−180, 180].
pub fn signed_arc(from: f64, to: f64) -> f64 {
    let d = (to - from).rem_euclid(360.0);
    if d > 180.0 {
        d - 360.0
    } else {
        d
    }
}

/// Rasi çerçevesindeki bir boylamın bhava noktası. Nakshatra ve pada sidereal boylamdandır
/// (`zodiac` tropikal-rasi ise `ayanamsa_deg` düşülür).
pub fn bhava_point(lon: f64, zodiac: ZodiacMode, ayanamsa_deg: f64) -> BhavaPoint {
    let lon = normalize(lon);
    let sign = Sign::of(lon);
    let (nakshatra, pada) = nakshatra_pada(to_sidereal(zodiac, lon, ayanamsa_deg));
    BhavaPoint {
        lon,
        sign,
        deg_in_sign: deg_in_sign(lon),
        nakshatra,
        pada,
        sign_lord: sign_lord(sign),
        nakshatra_lord: nakshatra_lord(nakshatra),
    }
}

/// Tam burç evlerinde bhava ortaları: lagna + 30° × (n − 1).
pub fn whole_sign_mids(lagna_lon: f64) -> [f64; 12] {
    std::array::from_fn(|i| normalize(lagna_lon + 30.0 * i as f64))
}

/// Porphyry noktaları (Sripati'de bhava ortaları): MC'den lagnaya ve lagnadan IC'ye uzanan iki
/// çeyrek üçe bölünür; öteki altı nokta bunların karşısıdır. `[0]` lagna, `[3]` IC, `[6]`
/// batı ufku, `[9]` MC.
pub fn porphyry_points(asc: f64, mc: f64) -> [f64; 12] {
    // MC → lagna yayı (10., 11. ve 12. bhavaların ortaları) ve lagna → IC yayı.
    let upper = normalize(asc - mc);
    let lower = 180.0 - upper;
    let mut p = [0.0; 12];
    for k in 0..3 {
        let thirds = k as f64;
        p[k] = normalize(asc + lower * thirds / 3.0);
        p[9 + k] = normalize(mc + upper * thirds / 3.0);
    }
    for k in 0..6 {
        let opposite = (k + 9) % 12;
        // p[3..=8]: p[9..=11] ve p[0..=2] noktalarının karşısı.
        p[3 + k] = normalize(p[opposite] + 180.0);
    }
    p
}

/// Bhava ortalarından sandhiler: n. bhavanın başlangıcı, (n − 1). ve n. ortanın tam ortasıdır.
pub fn sandhis_from_mids(mids: &[f64; 12]) -> [f64; 12] {
    std::array::from_fn(|i| {
        let prev = mids[(i + 11) % 12];
        normalize(prev + (mids[i] - prev).rem_euclid(360.0) / 2.0)
    })
}

/// Saf tablo. `cusps` on iki bhavanın başlangıcı, `mids` (varsa) ortasıdır; ikisi de rasi
/// çerçevesindedir. `grahas`, `Graha::ALL` sırasında rasi çerçevesindeki boylamlardır.
pub fn bhava_table(
    system: HouseSystem,
    cusps: &[f64; 12],
    mids: Option<&[f64; 12]>,
    lagna_sign: Sign,
    grahas: &[(Graha, f64); 9],
    zodiac: ZodiacMode,
    ayanamsa_deg: f64,
) -> BhavaTable {
    let point = |lon: f64| bhava_point(lon, zodiac, ayanamsa_deg);
    let rows = (0..12)
        .map(|i| {
            let end = cusps[(i + 1) % 12];
            BhavaRow {
                house: (i + 1) as u8,
                start: point(cusps[i]),
                mid: mids.map(|m| point(m[i])),
                end: point(end),
                span_deg: (end - cusps[i]).rem_euclid(360.0),
            }
        })
        .collect();
    let grahas = grahas
        .iter()
        .map(|&(graha, lon)| {
            let bhava = house_from_cusps(lon, cusps);
            let at = usize::from(bhava) - 1;
            let sign_house = Sign::of(lon).house_from(lagna_sign);
            GrahaBhava {
                graha,
                sign_house,
                bhava,
                from_start_deg: (lon - cusps[at]).rem_euclid(360.0),
                from_mid_deg: mids.map(|m| signed_arc(m[at], lon)),
                shifted: bhava != sign_house,
            }
        })
        .collect();
    BhavaTable {
        method: BHAVA_METHOD.to_owned(),
        system,
        porphyry_fallback: false,
        rows,
        grahas,
    }
}

/// `compute_bhava_table` komutunun hesabı. Doğum saati bilinmiyorsa (`input.time_known` yanlış)
/// `None` döner (K2); ayar, an ve konum denetimi (`compute_chart`) her durumda yapılır. Ev
/// sistemi haritanınkidir (`Chart.bhavas.system`). Sripati'de MC için ephemeris'ten evler bir kez
/// daha istenir.
pub fn compute_bhava_table<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<Option<BhavaTable>, CoreError> {
    let chart = compute_chart_for(eph, input)?;
    if !input.time_known {
        return Ok(None);
    }
    let cusps: [f64; 12] = chart
        .bhavas
        .cusps
        .as_slice()
        .try_into()
        .map_err(|_| CoreError::NonFiniteResult("bhavas"))?;
    let mids = match chart.bhavas.system {
        HouseSystem::WholeSign => Some(whole_sign_mids(chart.lagna.lon)),
        HouseSystem::Sripati => {
            let houses = eph.houses(
                input.jd_ut,
                input.lat,
                input.lon,
                HouseSystem::Sripati.ephemeris_system(),
                rasi_frame(&input.settings),
            )?;
            finite(houses.mc, "bhavas")?;
            Some(porphyry_points(chart.lagna.lon, houses.mc))
        }
        HouseSystem::Placidus => None,
    };
    // `Chart::grahas` her zaman 9 elemandır ve `Graha::ALL` sırasındadır.
    let grahas: [(Graha, f64); 9] =
        std::array::from_fn(|i| (chart.grahas[i].graha, chart.grahas[i].lon));
    let mut table = bhava_table(
        chart.bhavas.system,
        &cusps,
        mids.as_ref(),
        chart.lagna.sign,
        &grahas,
        input.settings.zodiac,
        chart.ayanamsa_deg,
    );
    table.porphyry_fallback = chart
        .meta
        .warnings
        .contains(&ChartWarning::PlacidusPolarFallback);
    Ok(Some(table))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dasha::birth_dasha;
    use crate::fake::FakeEph;
    use crate::houses::whole_sign_cusps;
    use crate::settings::CalcSettings;

    const JD: f64 = 2_451_545.0;

    fn input(settings: CalcSettings, time_known: bool) -> ChartInput {
        ChartInput {
            jd_ut: JD,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known,
            lagna_number: None,
        }
    }

    fn close(a: f64, b: f64) -> bool {
        signed_arc(a, b).abs() < 1e-9
    }

    /// El örneğinin grahaları (`Graha::ALL` sırasında, sidereal).
    const LONS: [f64; 9] = [82.0, 80.5, 101.0, 110.0, 359.5, 1.0, 46.0, 201.0, 21.0];

    fn grahas() -> [(Graha, f64); 9] {
        std::array::from_fn(|i| (Graha::ALL[i], LONS[i]))
    }

    #[test]
    fn nakshatra_lords_repeat_every_nine_and_agree_with_the_dasha_seed() {
        assert_eq!(nakshatra_lord(1), Graha::Ketu);
        assert_eq!(nakshatra_lord(4), Graha::Moon);
        assert_eq!(nakshatra_lord(9), Graha::Mercury);
        assert_eq!(nakshatra_lord(10), Graha::Ketu);
        assert_eq!(nakshatra_lord(27), Graha::Mercury);
        // İkinci yazım: Vimshottari'nin doğum lordu, Ay'ın nakshatrasının lordudur.
        for n in 1..=27u8 {
            let mid = (f64::from(n) - 0.5) * 40.0 / 3.0;
            assert_eq!(nakshatra_lord(n), birth_dasha(mid).0, "nakshatra {n}");
        }
    }

    #[test]
    fn signed_arcs_are_short_and_signed() {
        assert_eq!(signed_arc(100.0, 110.0), 10.0);
        assert_eq!(signed_arc(110.0, 100.0), -10.0);
        assert_eq!(signed_arc(350.0, 10.0), 20.0);
        assert_eq!(signed_arc(10.0, 350.0), -20.0);
        assert_eq!(signed_arc(0.0, 180.0), 180.0);
        assert_eq!(signed_arc(180.0, 0.0), 180.0);
        assert_eq!(signed_arc(45.0, 45.0), 0.0);
    }

    #[test]
    fn a_point_reads_sign_nakshatra_and_both_lords() {
        // 81° sidereal: İkizler 21°; pada dizini 81 × 0,3 = 24,3 → 24: 7. nakshatra
        // (Punarvasu), 1. pada. Burç lordu Merkür; 7. nakshatranın lordu Jüpiter.
        let p = bhava_point(81.0, ZodiacMode::Sidereal, 24.0);
        assert_eq!((p.lon, p.sign, p.deg_in_sign), (81.0, Sign::Gemini, 21.0));
        assert_eq!((p.nakshatra, p.pada), (7, 1));
        assert_eq!(
            (p.sign_lord, p.nakshatra_lord),
            (Graha::Mercury, Graha::Jupiter)
        );
        // Tropikal-rasi kipinde burç tropikal boylamdan, nakshatra sidereal boylamdandır:
        // 81° tropikal − 24° = 57° sidereal → dizin 17: 5. nakshatra (Mrigashira), 2. pada.
        let t = bhava_point(81.0, ZodiacMode::TropicalRasiSiderealNakshatra, 24.0);
        assert_eq!((t.sign, t.nakshatra, t.pada), (Sign::Gemini, 5, 2));
        assert_eq!(
            (t.sign_lord, t.nakshatra_lord),
            (Graha::Mercury, Graha::Mars)
        );
        // Boylam [0, 360) aralığına iner.
        assert_eq!(
            bhava_point(-30.0, ZodiacMode::Sidereal, 0.0).sign,
            Sign::Pisces
        );
    }

    #[test]
    fn porphyry_points_trisect_the_quadrants_and_sandhis_bisect_the_points() {
        // Lagna 101°, MC 341°: MC → lagna yayı 120° (üçte biri 40°), lagna → IC yayı 60° (üçte
        // biri 20°). IC 161°, batı ufku 281°.
        let mids = porphyry_points(101.0, 341.0);
        let expected = [
            101.0, 121.0, 141.0, 161.0, 201.0, 241.0, 281.0, 301.0, 321.0, 341.0, 21.0, 61.0,
        ];
        for (i, (got, want)) in mids.iter().zip(expected).enumerate() {
            assert!(close(*got, want), "mid {} = {got}", i + 1);
        }
        // Sandhi iki komşu ortanın ortasıdır: 1. bhava 61° ile 101°'nin ortasında başlar.
        let sandhis = sandhis_from_mids(&mids);
        let expected = [
            81.0, 111.0, 131.0, 151.0, 181.0, 221.0, 261.0, 291.0, 311.0, 331.0, 1.0, 41.0,
        ];
        for (i, (got, want)) in sandhis.iter().zip(expected).enumerate() {
            assert!(close(*got, want), "sandhi {} = {got}", i + 1);
        }
        // Eşit çeyrekler (MC lagnadan 90° geride): noktalar 30° arayla durur.
        let equal = porphyry_points(70.0, 340.0);
        for (i, m) in equal.iter().enumerate() {
            assert!(close(*m, 70.0 + 30.0 * i as f64), "equal {}", i + 1);
        }
        // Tam burç: orta, lagnanın derecesinin o burçtaki karşılığıdır.
        assert_eq!(whole_sign_mids(346.0)[..3], [346.0, 16.0, 46.0]);
    }

    #[test]
    fn sripati_table_by_hand() {
        // Yukarıdaki örnek: lagna 101° (Yengeç 11°), MC 341°. Sandhiler ve ortalar elle
        // türetildi. Grahalar: Güneş 82°, Ay 80,5°, Mars 101°, Merkür 110°, Jüpiter 359,5°,
        // Venüs 1°, Satürn 46°, Rahu 201°, Ketu 21°.
        let mids = porphyry_points(101.0, 341.0);
        let cusps = sandhis_from_mids(&mids);
        let t = bhava_table(
            HouseSystem::Sripati,
            &cusps,
            Some(&mids),
            Sign::Cancer,
            &grahas(),
            ZodiacMode::Sidereal,
            24.0,
        );
        assert_eq!(t.method, BHAVA_METHOD);
        assert_eq!(t.system, HouseSystem::Sripati);
        assert!(!t.porphyry_fallback);
        assert_eq!((t.rows.len(), t.grahas.len()), (12, 9));
        // 1. bhava: 81° (İkizler 21°) → orta 101° (Yengeç 11°) → 111° (Yengeç 21°); yay 30°.
        let first = &t.rows[0];
        assert_eq!(first.house, 1);
        assert_eq!(
            (first.start.sign, first.end.sign),
            (Sign::Gemini, Sign::Cancer)
        );
        assert!(close(first.start.lon, 81.0) && close(first.end.lon, 111.0));
        assert!(close(first.mid.unwrap().lon, 101.0));
        assert!((first.span_deg - 30.0).abs() < 1e-9);
        // 81° sidereal: pada dizini 81 × 0,3 = 24,3 → 24: 7. nakshatra (Punarvasu, lord
        // Jüpiter), 1. pada. 101°: 30,3 → 30: 8. nakshatra (Pushya, lord Satürn), 3. pada.
        assert_eq!((first.start.nakshatra, first.start.pada), (7, 1));
        assert_eq!(first.start.nakshatra_lord, Graha::Jupiter);
        let mid = first.mid.unwrap();
        assert_eq!(
            (mid.nakshatra, mid.pada, mid.nakshatra_lord),
            (8, 3, Graha::Saturn)
        );
        // Yaylar: 30, 20, 20, 30, 40, 40, 30, 20, 20, 30, 40, 40 (toplam 360).
        let spans: Vec<f64> = t.rows.iter().map(|r| r.span_deg).collect();
        let expected = [
            30.0, 20.0, 20.0, 30.0, 40.0, 40.0, 30.0, 20.0, 20.0, 30.0, 40.0, 40.0,
        ];
        for (got, want) in spans.iter().zip(expected) {
            assert!((got - want).abs() < 1e-9, "span {got}");
        }
        assert!((spans.iter().sum::<f64>() - 360.0).abs() < 1e-9);
        // Her bhavanın bitişi sonrakinin başlangıcıdır.
        for i in 0..12 {
            assert_eq!(t.rows[i].end, t.rows[(i + 1) % 12].start);
        }
        // Grahalar: (burç evi, bhava, başlangıçtan, ortadan).
        //   Güneş 82° İkizler: burç evi 12; 81° ≤ 82° < 111° → 1. bhava; +1°; ortadan −19°.
        //   Ay 80,5° İkizler: burç evi 12; 41° ≤ 80,5° < 81° → 12. bhava; +39,5°; orta 61° →
        //     +19,5°.
        //   Mars 101° Yengeç: burç evi 1; 1. bhava; +20°; ortada (0°).
        //   Merkür 110° Yengeç: burç evi 1; 1. bhava; +29°; +9°.
        //   Jüpiter 359,5° Balık: burç evi 9; 331° ≤ 359,5° → 10. bhava; +28,5°; orta 341° →
        //     +18,5°.
        //   Venüs 1° Koç: burç evi 10; başlangıç noktası bhavaya dahildir → 11. bhava; 0°;
        //     orta 21° → −20°.
        //   Satürn 46° Boğa: burç evi 11; 41° ≤ 46° → 12. bhava; +5°; orta 61° → −15°.
        //   Rahu 201° Terazi: burç evi 4; 181° ≤ 201° < 221° → 5. bhava; +20°; ortada (0°).
        //   Ketu 21° Koç: burç evi 10; 1° ≤ 21° < 41° → 11. bhava; +20°; ortada (0°).
        let expected: [(u8, u8, f64, f64); 9] = [
            (12, 1, 1.0, -19.0),
            (12, 12, 39.5, 19.5),
            (1, 1, 20.0, 0.0),
            (1, 1, 29.0, 9.0),
            (9, 10, 28.5, 18.5),
            (10, 11, 0.0, -20.0),
            (11, 12, 5.0, -15.0),
            (4, 5, 20.0, 0.0),
            (10, 11, 20.0, 0.0),
        ];
        for (g, (sign_house, bhava, from_start, from_mid)) in t.grahas.iter().zip(expected) {
            assert_eq!(
                (g.sign_house, g.bhava),
                (sign_house, bhava),
                "{:?}",
                g.graha
            );
            assert!(
                (g.from_start_deg - from_start).abs() < 1e-9,
                "{:?}",
                g.graha
            );
            assert!(
                (g.from_mid_deg.unwrap() - from_mid).abs() < 1e-9,
                "{:?}",
                g.graha
            );
            assert_eq!(g.shifted, sign_house != bhava, "{:?}", g.graha);
        }
        let shifted: Vec<Graha> = t
            .grahas
            .iter()
            .filter(|g| g.shifted)
            .map(|g| g.graha)
            .collect();
        assert_eq!(
            shifted,
            [
                Graha::Sun,
                Graha::Jupiter,
                Graha::Venus,
                Graha::Saturn,
                Graha::Rahu,
                Graha::Ketu
            ]
        );
    }

    #[test]
    fn whole_sign_table_never_shifts_and_placidus_has_no_middle() {
        // Tam burç, lagna Yengeç 10°: bhava burcun kendisidir; orta burcun 10°'sidir.
        let cusps: [f64; 12] = whole_sign_cusps(Sign::Cancer).try_into().unwrap();
        let mids = whole_sign_mids(100.0);
        let t = bhava_table(
            HouseSystem::WholeSign,
            &cusps,
            Some(&mids),
            Sign::Cancer,
            &grahas(),
            ZodiacMode::Sidereal,
            24.0,
        );
        for (i, row) in t.rows.iter().enumerate() {
            assert_eq!(row.start.sign, Sign::from_index(3 + i));
            assert_eq!(row.start.deg_in_sign, 0.0);
            assert_eq!(row.span_deg, 30.0);
            assert!((row.mid.unwrap().deg_in_sign - 10.0).abs() < 1e-9);
        }
        assert!(t
            .grahas
            .iter()
            .all(|g| !g.shifted && g.bhava == g.sign_house));
        // Güneş İkizler 22°: 12. bhava; başlangıçtan 22°, ortadan (İkizler 10°) +12°.
        let sun = t.grahas[0];
        assert_eq!((sun.bhava, sun.from_start_deg), (12, 22.0));
        assert!((sun.from_mid_deg.unwrap() - 12.0).abs() < 1e-9);
        // Placidus: aynı başlangıçlar, orta yok.
        let p = bhava_table(
            HouseSystem::Placidus,
            &cusps,
            None,
            Sign::Cancer,
            &grahas(),
            ZodiacMode::Sidereal,
            24.0,
        );
        assert!(p.rows.iter().all(|r| r.mid.is_none()));
        assert!(p.grahas.iter().all(|g| g.from_mid_deg.is_none()));
        assert_eq!(p.grahas[0].from_start_deg, 22.0);
    }

    #[test]
    fn collector_follows_the_chart_and_hides_the_table_when_time_is_unknown() {
        let mut eph = FakeEph::default();
        // Tam burç (varsayılan): FakeEph lagnası sidereal 46° (Boğa 16°).
        let i = input(CalcSettings::default(), true);
        let t = compute_bhava_table(&mut eph, &i).unwrap().expect("table");
        let chart = compute_chart_for(&mut eph, &i).unwrap();
        assert_eq!(t.system, HouseSystem::WholeSign);
        assert_eq!(t.rows[0].start.sign, Sign::Taurus);
        assert!((t.rows[0].mid.unwrap().deg_in_sign - 16.0).abs() < 1e-9);
        for (g, c) in t.grahas.iter().zip(&chart.grahas) {
            assert_eq!((g.graha, g.bhava), (c.graha, c.house));
            assert!(!g.shifted);
        }
        // Sripati: tablo haritanın başlangıçlarını ve graha evlerini izler. FakeEph eşit
        // evler verir (MC lagnadan 90° geride): ortalar 30° arayla, lagnadan başlar.
        let sripati = CalcSettings {
            house_system: HouseSystem::Sripati,
            ..CalcSettings::default()
        };
        let i = input(sripati, true);
        let t = compute_bhava_table(&mut eph, &i).unwrap().expect("table");
        let chart = compute_chart_for(&mut eph, &i).unwrap();
        assert_eq!(t.system, HouseSystem::Sripati);
        for (row, cusp) in t.rows.iter().zip(&chart.bhavas.cusps) {
            assert_eq!(row.start.lon, *cusp);
        }
        for (k, row) in t.rows.iter().enumerate() {
            assert!(close(row.mid.unwrap().lon, 46.0 + 30.0 * k as f64));
        }
        for (g, c) in t.grahas.iter().zip(&chart.grahas) {
            assert_eq!(g.bhava, c.house);
        }
        // Placidus kutup dairesinde Porphyry'ye düşerse tablo bunu söyler; orta yoktur.
        let mut polar = FakeEph {
            porphyry_fallback: true,
            ..FakeEph::default()
        };
        let placidus = CalcSettings {
            house_system: HouseSystem::Placidus,
            ..CalcSettings::default()
        };
        let t = compute_bhava_table(&mut polar, &input(placidus.clone(), true))
            .unwrap()
            .expect("table");
        assert!(t.porphyry_fallback);
        assert_eq!(t.system, HouseSystem::Placidus);
        assert!(t.rows.iter().all(|r| r.mid.is_none()));
        // Saat bilinmiyor: tablo yok; girdi yine de denetlenir.
        assert_eq!(
            compute_bhava_table(&mut eph, &input(placidus, false)),
            Ok(None)
        );
        let mut bad = input(CalcSettings::default(), false);
        bad.lat = 91.0;
        assert_eq!(
            compute_bhava_table(&mut eph, &bad),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0
            })
        );
    }

    #[test]
    fn tropical_rasi_mode_and_number_prashna_follow_the_chart() {
        let mut eph = FakeEph::default();
        // Tropikal-rasi: burçlar tropikal (lagna 70°, İkizler 10°), nakshatra sidereal (46°:
        // dizin 13 → 4. nakshatra, Rohini, 2. pada; lord Ay).
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        let t = compute_bhava_table(&mut eph, &input(tropical, true))
            .unwrap()
            .expect("table");
        let mid = t.rows[0].mid.unwrap();
        assert_eq!(mid.sign, Sign::Gemini);
        assert!((mid.deg_in_sign - 10.0).abs() < 1e-9);
        assert_eq!(
            (mid.nakshatra, mid.pada, mid.nakshatra_lord),
            (4, 2, Graha::Moon)
        );
        // Sayı prashnası: ev sistemi ayarı ne olursa olsun tam burç, lagna sayıdan.
        let mut numbered = input(
            CalcSettings {
                house_system: HouseSystem::Placidus,
                ..CalcSettings::default()
            },
            true,
        );
        numbered.lagna_number = Some(30);
        let t = compute_bhava_table(&mut eph, &numbered)
            .unwrap()
            .expect("table");
        assert_eq!(t.system, HouseSystem::WholeSign);
        assert_eq!(t.rows[0].start.sign, Sign::Cancer);
        assert!(t.rows[0].mid.is_some());
    }
}
