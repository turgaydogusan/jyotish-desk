//! Vimshottari dasha (BPHS, nakshatra dashaları; Santhanam çevirisinde böl. 46).
//!
//! Doğumdaki Ay'ın sidereal nakshatrası ilk mahadashanın lordunu ve geçen kesrini
//! belirler. Her dönemin dokuz alt dönemi ebeveynin lordundan başlar ve ebeveyn süresini
//! `yıl(L) / 120` oranıyla böler. Zamanlar UT Jülyen günüdür.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::settings::{DASHA_YEAR_RANGE, JD_UT_RANGE};
use crate::zodiac::{nakshatra_pada, normalize, NAKSHATRA_SPAN_DEG};
use crate::CoreError;

pub const VIMSHOTTARI_TOTAL_YEARS: f64 = 120.0;
/// 1 mahadasha, 2 antardasha, 3 pratyantardasha, 4 sookshma, 5 prana.
pub const MAX_DASHA_LEVEL: u8 = 5;
/// `Chart.dasha`'nın taşıdığı seviye sayısı (spec §4: "Dasha 3 seviye gelir").
pub const CHART_DASHA_LEVELS: u8 = 3;

/// Ashwini'den başlayarak nakshatra lordları; sıra her 9 nakshatrada tekrar eder.
pub const VIMSHOTTARI_ORDER: [Graha; 9] = [
    Graha::Ketu,
    Graha::Venus,
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Rahu,
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Mercury,
];

/// Mahadasha süresi, yıl.
pub fn vimshottari_years(lord: Graha) -> f64 {
    match lord {
        Graha::Ketu => 7.0,
        Graha::Venus => 20.0,
        Graha::Sun => 6.0,
        Graha::Moon => 10.0,
        Graha::Mars => 7.0,
        Graha::Rahu => 18.0,
        Graha::Jupiter => 16.0,
        Graha::Saturn => 19.0,
        Graha::Mercury => 17.0,
    }
}

fn order_index(lord: Graha) -> usize {
    match lord {
        Graha::Ketu => 0,
        Graha::Venus => 1,
        Graha::Sun => 2,
        Graha::Moon => 3,
        Graha::Mars => 4,
        Graha::Rahu => 5,
        Graha::Jupiter => 6,
        Graha::Saturn => 7,
        Graha::Mercury => 8,
    }
}

/// Dasha ağacını tek başına belirleyen girdiler.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DashaSeed {
    /// Doğum anı, UT Jülyen günü.
    pub birth_jd_ut: f64,
    /// Doğumdaki Ay'ın sidereal boylamı (her zodyak kipinde sidereal).
    pub moon_sidereal_lon: f64,
    /// Yıl uzunluğu, gün (`CalcSettings::dasha_year`).
    pub year_days: f64,
}

impl DashaSeed {
    pub fn validate(&self) -> Result<(), CoreError> {
        // `compute_chart` ile aynı aralık; NaN ve sonsuz da dışarıdadır.
        if !JD_UT_RANGE.contains(&self.birth_jd_ut) {
            return Err(CoreError::InvalidDashaInput("birth_jd_ut"));
        }
        if !self.moon_sidereal_lon.is_finite() {
            return Err(CoreError::InvalidDashaInput("moon_sidereal_lon"));
        }
        // NaN ve sonsuz aralık dışında kalır.
        if !DASHA_YEAR_RANGE.contains(&self.year_days) {
            return Err(CoreError::InvalidDashaInput("year_days"));
        }
        Ok(())
    }
}

/// Bir dasha dönemi (spec §5 `DashaPeriod`). Zamanlar UT Jülyen günüdür.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DashaPeriod {
    /// Kökten bu döneme lordlar: `[maha]`, `[maha, antar]`, …
    pub lords: Vec<Graha>,
    /// 1 mahadasha … 5 prana; `lords.len()` ile aynı.
    pub level: u8,
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
    /// `dasha_children` ile alt dönem istenebilir (`level < MAX_DASHA_LEVEL`).
    pub has_children: bool,
}

/// Haritanın Vimshottari ağacı: 1–3. seviyeler ön-sırada (her dönemin ardından alt
/// dönemleri). 4. ve 5. seviyeler `dasha_children(seed, path)` ile istenir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Dasha {
    pub seed: DashaSeed,
    pub periods: Vec<DashaPeriod>,
}

/// Doğum mahadashasının lordu ve geçen kesri, [0, 1]. Lord `nakshatra_pada` ile aynı
/// nakshatradan seçilir; gösterilen nakshatra ile dasha lordu asla ayrışmaz.
pub fn birth_dasha(moon_sidereal_lon: f64) -> (Graha, f64) {
    let lon = normalize(moon_sidereal_lon);
    let (nakshatra, _) = nakshatra_pada(lon);
    let index = usize::from(nakshatra).saturating_sub(1);
    let elapsed = (lon - index as f64 * NAKSHATRA_SPAN_DEG) / NAKSHATRA_SPAN_DEG;
    (VIMSHOTTARI_ORDER[index % 9], elapsed.clamp(0.0, 1.0))
}

/// `[start, end)` aralığını `first`'ten başlayan dokuz alt döneme böler. Bitişler
/// birikimli yıldan hesaplanır; sonuncusu `end`'e tam eşittir.
fn subdivide(path: &[Graha], first: Graha, start: f64, end: f64) -> Vec<DashaPeriod> {
    let level = (path.len() + 1) as u8;
    let span = end - start;
    let first = order_index(first);
    let mut periods = Vec::with_capacity(9);
    let mut from = start;
    let mut years = 0.0;
    for k in 0..9 {
        let lord = VIMSHOTTARI_ORDER[(first + k) % 9];
        years += vimshottari_years(lord);
        let to = if k == 8 {
            end
        } else {
            start + span * years / VIMSHOTTARI_TOTAL_YEARS
        };
        let mut lords = path.to_vec();
        lords.push(lord);
        periods.push(DashaPeriod {
            lords,
            level,
            start_jd_ut: from,
            end_jd_ut: to,
            has_children: level < MAX_DASHA_LEVEL,
        });
        from = to;
    }
    periods
}

/// Dönemin dokuz alt dönemi; 5. seviyede boş.
fn children_of(period: &DashaPeriod) -> Vec<DashaPeriod> {
    match period.lords.last() {
        Some(&lord) if period.has_children => {
            subdivide(&period.lords, lord, period.start_jd_ut, period.end_jd_ut)
        }
        _ => Vec::new(),
    }
}

/// Dokuz mahadasha. İlki doğumdan `f × yıl(lord) × year_days` gün önce başlar; toplam
/// 120 yıl.
pub fn mahadashas(seed: &DashaSeed) -> Result<Vec<DashaPeriod>, CoreError> {
    seed.validate()?;
    let (lord, elapsed) = birth_dasha(seed.moon_sidereal_lon);
    let start = seed.birth_jd_ut - elapsed * vimshottari_years(lord) * seed.year_days;
    let end = start + VIMSHOTTARI_TOTAL_YEARS * seed.year_days;
    Ok(subdivide(&[], lord, start, end))
}

/// `path` ile seçilen dönemin alt dönemleri. `[]` mahadashaları, `[maha]` antardashaları,
/// `[maha, antar, pratyantar, sookshma]` prana dönemlerini verir. Daha uzun yol hatadır.
pub fn dasha_children(seed: &DashaSeed, path: &[Graha]) -> Result<Vec<DashaPeriod>, CoreError> {
    if path.len() >= usize::from(MAX_DASHA_LEVEL) {
        return Err(CoreError::InvalidDashaInput("path"));
    }
    let mut periods = mahadashas(seed)?;
    for lord in path {
        // Her seviyede dokuz lordun hepsi bulunur; yine de panik yerine hata.
        let parent = periods
            .into_iter()
            .find(|p| p.lords.last() == Some(lord))
            .ok_or(CoreError::InvalidDashaInput("path"))?;
        periods = children_of(&parent);
    }
    Ok(periods)
}

/// `jd_ut` anını içeren dönemlerin lordları, kökten `depth` seviyeye (1 maha … 5 prana).
/// An 120 yıllık Vimshottari döngüsünün dışındaysa yol orada biter (boş ya da kısa yol).
/// Arayüzün "güncel dönem"i buradan gelir; arayüz tarih karşılaştırması yapmaz (C4).
pub fn dasha_path_at(seed: &DashaSeed, jd_ut: f64, depth: u8) -> Result<Vec<Graha>, CoreError> {
    if depth == 0 || depth > MAX_DASHA_LEVEL {
        return Err(CoreError::InvalidDashaInput("depth"));
    }
    if !jd_ut.is_finite() {
        return Err(CoreError::InvalidDashaInput("jd_ut"));
    }
    let mut path = Vec::with_capacity(usize::from(depth));
    let mut periods = mahadashas(seed)?;
    loop {
        let Some(p) = periods
            .iter()
            .find(|p| p.start_jd_ut <= jd_ut && jd_ut < p.end_jd_ut)
        else {
            return Ok(path);
        };
        path.push(*p.lords.last().expect("a period has a lord"));
        if path.len() == usize::from(depth) {
            return Ok(path);
        }
        periods = dasha_children(seed, &path)?;
    }
}

/// Haritaya giden üç seviyeli ağaç (9 + 81 + 729 dönem).
pub fn vimshottari(seed: DashaSeed) -> Result<Dasha, CoreError> {
    let mut periods = Vec::with_capacity(9 + 81 + 729);
    for maha in mahadashas(&seed)? {
        push_preorder(&mut periods, maha, CHART_DASHA_LEVELS);
    }
    Ok(Dasha { seed, periods })
}

fn push_preorder(out: &mut Vec<DashaPeriod>, period: DashaPeriod, levels: u8) {
    let children = if period.level < levels {
        children_of(&period)
    } else {
        Vec::new()
    };
    out.push(period);
    for child in children {
        push_preorder(out, child, levels);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JD: f64 = 2_451_545.0;

    fn seed(moon: f64) -> DashaSeed {
        DashaSeed {
            birth_jd_ut: JD,
            moon_sidereal_lon: moon,
            year_days: 365.25,
        }
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn last_lords(periods: &[DashaPeriod]) -> Vec<Graha> {
        periods.iter().map(|p| *p.lords.last().unwrap()).collect()
    }

    #[test]
    fn order_and_years_total_120() {
        assert_eq!(
            VIMSHOTTARI_ORDER.map(vimshottari_years),
            [7.0, 20.0, 6.0, 10.0, 7.0, 18.0, 16.0, 19.0, 17.0]
        );
        let total: f64 = VIMSHOTTARI_ORDER
            .iter()
            .map(|&g| vimshottari_years(g))
            .sum();
        assert_eq!(total, VIMSHOTTARI_TOTAL_YEARS);
    }

    #[test]
    fn birth_dasha_follows_the_moon_nakshatra() {
        assert_eq!(birth_dasha(0.0), (Graha::Ketu, 0.0));
        assert_eq!(birth_dasha(NAKSHATRA_SPAN_DEG / 2.0), (Graha::Ketu, 0.5));
        assert_eq!(birth_dasha(NAKSHATRA_SPAN_DEG), (Graha::Venus, 0.0));
        // Magha (10.) ve Mula (19.) yine Ketu ile başlar.
        assert_eq!(birth_dasha(9.0 * NAKSHATRA_SPAN_DEG + 1.0).0, Graha::Ketu);
        assert_eq!(birth_dasha(18.0 * NAKSHATRA_SPAN_DEG + 1.0).0, Graha::Ketu);
        // Revati (27.) Merkür; sona çok yakın.
        let (lord, elapsed) = birth_dasha(359.999_999);
        assert_eq!(lord, Graha::Mercury);
        assert!(elapsed > 0.999_99 && elapsed <= 1.0, "{elapsed}");
        // Golden new-delhi Ay'ı: 199.470529 / 13.3̄ = 14.96029 → Swati (15.), Rahu,
        // kesir 0.9602897.
        let (lord, elapsed) = birth_dasha(199.470_529);
        assert_eq!(lord, Graha::Rahu);
        assert!((elapsed - 0.960_289_675).abs() < 1e-9, "{elapsed}");
    }

    #[test]
    fn mahadashas_start_before_birth_and_cover_120_years() {
        let m = mahadashas(&seed(NAKSHATRA_SPAN_DEG / 2.0)).unwrap();
        assert_eq!(last_lords(&m), VIMSHOTTARI_ORDER);
        assert!(m
            .iter()
            .all(|p| p.level == 1 && p.has_children && p.lords.len() == 1));
        // Ketu'nun yarısı geçmiş: 0.5 × 7 × 365.25 = 1278.375 gün önce başlar.
        assert!(close(m[0].start_jd_ut, 2_450_266.625));
        assert!(close(m[0].end_jd_ut, 2_452_823.375));
        // Venüs bitişi: başlangıç + 27 × 365.25.
        assert!(close(m[1].end_jd_ut, 2_460_128.375));
        assert!(close(m[8].end_jd_ut, 2_450_266.625 + 120.0 * 365.25));
    }

    #[test]
    fn sub_periods_start_from_the_parent_lord_and_split_by_years() {
        let s = seed(NAKSHATRA_SPAN_DEG / 2.0);
        let antar = dasha_children(&s, &[Graha::Ketu]).unwrap();
        assert_eq!(last_lords(&antar), VIMSHOTTARI_ORDER);
        assert!(antar
            .iter()
            .all(|p| p.level == 2 && p.lords[0] == Graha::Ketu));
        // Ketu mahadashası 7 × 365.25 = 2556.75 gün; Ke-Ke 2556.75 × 7 / 120 = 149.14375 gün.
        assert!(close(antar[0].start_jd_ut, 2_450_266.625));
        assert!(close(antar[0].end_jd_ut, 2_450_415.768_75));
        // Ke-Ve bitişi: 2556.75 × 27 / 120 = 575.26875 gün.
        assert!(close(antar[1].end_jd_ut, 2_450_841.893_75));
        let venus = dasha_children(&s, &[Graha::Venus]).unwrap();
        assert_eq!(
            last_lords(&venus),
            [
                Graha::Venus,
                Graha::Sun,
                Graha::Moon,
                Graha::Mars,
                Graha::Rahu,
                Graha::Jupiter,
                Graha::Saturn,
                Graha::Mercury,
                Graha::Ketu
            ]
        );
    }

    #[test]
    fn deeper_levels_and_path_limits() {
        let s = seed(100.0);
        assert_eq!(dasha_children(&s, &[]).unwrap(), mahadashas(&s).unwrap());
        let path = [Graha::Rahu, Graha::Jupiter, Graha::Saturn, Graha::Mercury];
        let prana = dasha_children(&s, &path).unwrap();
        assert_eq!(prana.len(), 9);
        assert!(prana
            .iter()
            .all(|p| p.level == 5 && !p.has_children && p.lords[..4] == path));
        assert_eq!(
            dasha_children(&s, &[Graha::Sun; 5]),
            Err(CoreError::InvalidDashaInput("path"))
        );
    }

    #[test]
    fn year_length_scales_every_period() {
        let m = mahadashas(&DashaSeed {
            year_days: 360.0,
            ..seed(0.0)
        })
        .unwrap();
        assert!(close(m[0].start_jd_ut, JD));
        assert!(close(m[8].end_jd_ut - m[0].start_jd_ut, 43_200.0));
        let m = mahadashas(&DashaSeed {
            year_days: 365.2422,
            ..seed(0.0)
        })
        .unwrap();
        assert!(close(m[0].end_jd_ut, JD + 7.0 * 365.2422));
    }

    #[test]
    fn invalid_seed_is_rejected() {
        let bad = |s: DashaSeed| mahadashas(&s).unwrap_err();
        assert_eq!(
            bad(DashaSeed {
                birth_jd_ut: f64::NAN,
                ..seed(0.0)
            }),
            CoreError::InvalidDashaInput("birth_jd_ut")
        );
        assert_eq!(
            bad(DashaSeed {
                moon_sidereal_lon: f64::INFINITY,
                ..seed(0.0)
            }),
            CoreError::InvalidDashaInput("moon_sidereal_lon")
        );
        // D2: sonlu ama hesap aralığı dışındaki doğum anı (ör. IPC'den gelen 1e300).
        for jd in [1e300, 625_001.0, 2_817_998.0] {
            assert_eq!(
                bad(DashaSeed {
                    birth_jd_ut: jd,
                    ..seed(0.0)
                }),
                CoreError::InvalidDashaInput("birth_jd_ut"),
                "{jd}"
            );
        }
        for y in [0.0, 299.0, 1e308, f64::NAN] {
            assert_eq!(
                bad(DashaSeed {
                    year_days: y,
                    ..seed(0.0)
                }),
                CoreError::InvalidDashaInput("year_days"),
                "{y}"
            );
        }
    }

    #[test]
    fn chart_dasha_has_three_levels_in_preorder() {
        let d = vimshottari(seed(NAKSHATRA_SPAN_DEG / 2.0)).unwrap();
        assert_eq!(d.periods.len(), 9 + 81 + 729);
        assert_eq!(d.periods[0].lords, [Graha::Ketu]);
        assert_eq!(d.periods[1].lords, [Graha::Ketu, Graha::Ketu]);
        assert_eq!(d.periods[2].lords, [Graha::Ketu, Graha::Ketu, Graha::Ketu]);
        // Ke, Ke-Ke, 9 × Ke-Ke-*, sonra Ke-Ve: indeks 11.
        assert_eq!(d.periods[11].lords, [Graha::Ketu, Graha::Venus]);
        // Ketu bloğu 1 + 9 × (1 + 9) = 91 dönem; ardından Venüs mahadashası.
        assert_eq!(d.periods[91].lords, [Graha::Venus]);
        assert!(d.periods.iter().all(|p| p.has_children && p.level <= 3));
        let level3: Vec<DashaPeriod> = d
            .periods
            .iter()
            .filter(|p| p.level == 3 && p.lords[..2] == [Graha::Moon, Graha::Mars])
            .cloned()
            .collect();
        assert_eq!(
            level3,
            dasha_children(&d.seed, &[Graha::Moon, Graha::Mars]).unwrap()
        );
    }

    #[test]
    fn dasha_path_at_follows_the_containing_periods() {
        let seed = DashaSeed {
            birth_jd_ut: 2_451_545.0,
            moon_sidereal_lon: 199.470_529,
            year_days: 365.25,
        };
        // Doğum anı: doğum mahadashası ve onun içindeki dönemler.
        let at_birth = dasha_path_at(&seed, seed.birth_jd_ut, 3).unwrap();
        assert_eq!(at_birth.len(), 3);
        assert_eq!(at_birth[0], birth_dasha(seed.moon_sidereal_lon).0);
        // Her seviyede seçilen dönem anı içerir.
        let jd = seed.birth_jd_ut + 9_000.0;
        let path = dasha_path_at(&seed, jd, 5).unwrap();
        assert_eq!(path.len(), 5);
        for depth in 1..=5usize {
            let parent = &path[..depth - 1];
            let p = dasha_children(&seed, parent)
                .unwrap()
                .into_iter()
                .find(|p| p.lords == path[..depth])
                .unwrap();
            assert!(p.start_jd_ut <= jd && jd < p.end_jd_ut, "level {depth}");
        }
        // 120 yıllık döngünün dışı: yol yok.
        assert!(dasha_path_at(&seed, seed.birth_jd_ut + 200.0 * 365.25, 3)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn dasha_path_at_rejects_bad_input() {
        let seed = DashaSeed {
            birth_jd_ut: 2_451_545.0,
            moon_sidereal_lon: 10.0,
            year_days: 365.25,
        };
        for depth in [0, 6] {
            assert!(matches!(
                dasha_path_at(&seed, 2_451_545.0, depth),
                Err(CoreError::InvalidDashaInput("depth"))
            ));
        }
        assert!(matches!(
            dasha_path_at(&seed, f64::NAN, 3),
            Err(CoreError::InvalidDashaInput("jd_ut"))
        ));
    }
}
