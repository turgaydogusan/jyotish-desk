//! Avastha raporu ve toplayıcısı (K1, K3): saf hesap `avasthas` konumları ve ghati sayısını
//! görür; toplayıcı `compute_avasthas` temel haritayı hesaplar, gün doğumundan geçen ghatiyi
//! bulur ve saf hesaba verir.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{compute_chart_for, ChartWarning};
use crate::graha::Graha;
use crate::positions::{ChartInput, Positions};
use crate::CoreError;

use super::baladi::{baladi, jagradadi, Baladi, Jagradadi};
use super::lajjitadi::{lajjitadi, LajjitadiHit};
use super::shayanadi::{elapsed_ghatis, shayanadi, shayanadi_basis, ShayanadiBasis, ShayanadiHit};
use super::{AVASTHA_GRAHAS, AVASTHA_METHOD};

/// Bir grahanın dört dizideki durumu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GrahaAvastha {
    pub graha: Graha,
    pub baladi: Baladi,
    pub jagradadi: Jagradadi,
    /// Kural sırasında; boş olabilir. Lagna yoksa Lajjita değerlendirilmemiştir.
    pub lajjitadi: Vec<LajjitadiHit>,
    /// Lagna ya da gün doğumu yoksa `None`.
    pub shayanadi: Option<ShayanadiHit>,
}

/// `compute_avasthas` sonucu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AvasthaReport {
    /// Yedi graha, `AVASTHA_GRAHAS` sırasında.
    pub grahas: Vec<GrahaAvastha>,
    /// Shayanadi formülünün haritaya ait terimleri; Shayanadi hesaplanmadıysa `None`.
    pub shayanadi_basis: Option<ShayanadiBasis>,
    /// Yanlışsa lagna yoktur: Lajjita değerlendirilmemiş, Shayanadi hesaplanmamıştır ve Ay'a
    /// bağlı değerler yaklaşıktır (Ay'ın derecesi gün içinde değişir).
    pub time_known: bool,
    /// Temel haritanın konum duyarlılığı uyarısı (`LowPrecision`).
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

/// Saf hesap: konumlardan avastha raporu. `ghatis`, gün doğumundan doğuma tamamlanmış ghati
/// sayısıdır; `None` ise (gün doğumu yok) ya da lagna yoksa Shayanadi hesaplanmaz.
pub fn avasthas(positions: &Positions, ghatis: Option<u32>) -> AvasthaReport {
    let basis = shayanadi_basis(positions, ghatis);
    let grahas = AVASTHA_GRAHAS
        .into_iter()
        .filter_map(|graha| {
            let point = positions.point(graha);
            Some(GrahaAvastha {
                graha,
                baladi: baladi(point.rasi_sign, point.deg_in_sign),
                jagradadi: jagradadi(graha, point.rasi_sign)?,
                lajjitadi: lajjitadi(graha, positions),
                shayanadi: basis.as_ref().and_then(|b| shayanadi(graha, positions, b)),
            })
        })
        .collect();
    AvasthaReport {
        grahas,
        shayanadi_basis: basis,
        time_known: positions.lagna_sign.is_some(),
        warnings: Vec::new(),
        method: AVASTHA_METHOD.to_owned(),
    }
}

/// `compute_avasthas` komutunun hesabı: temel harita, konumlar, gün doğumundan geçen ghati,
/// avastha raporu. Ayar, an ve konum denetimi `compute_chart`'ınkidir. `input.time_known`
/// yanlışsa lagna boştur: Lajjita değerlendirilmez ve Shayanadi hesaplanmaz. Doğum yerinde o
/// gün gün doğumu yoksa (kutup) yalnızca Shayanadi boştur. Sayı prashnasında lagna sayıdan
/// gelir (`compute_chart_for`).
pub fn compute_avasthas<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<AvasthaReport, CoreError> {
    let chart = compute_chart_for(eph, input)?;
    let positions = Positions::from(&chart);
    let (positions, ghatis) = if input.time_known {
        let ghatis = chart
            .panchang
            .sunrise_jd_ut
            .and_then(|sunrise| elapsed_ghatis(input.jd_ut, sunrise));
        (positions, ghatis)
    } else {
        (positions.without_lagna(), None)
    };
    let mut report = avasthas(&positions, ghatis);
    report.warnings = chart
        .meta
        .warnings
        .iter()
        .copied()
        .filter(|w| *w == ChartWarning::LowPrecision)
        .collect();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avastha::{Lajjitadi, Shayanadi};
    use crate::chart::compute_chart;
    use crate::fake::FakeEph;
    use crate::settings::{CalcSettings, ZodiacMode};
    use crate::zodiac::Sign;
    use ephemeris::{Body, Precision};

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

    #[test]
    fn the_report_lists_the_seven_grahas_with_all_four_series() {
        // Lagna 70° İkizler; Ay 100° (nakshatra 8).
        let lons = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];
        let p = Positions::synthetic(Some(70.0), lons);
        let report = avasthas(&p, Some(28));
        assert_eq!(report.method, AVASTHA_METHOD);
        assert!(report.time_known && report.warnings.is_empty());
        let order: Vec<_> = report.grahas.iter().map(|g| g.graha).collect();
        assert_eq!(order, AVASTHA_GRAHAS);
        let basis = report.shayanadi_basis.expect("basis");
        assert_eq!(
            (basis.birth_nakshatra, basis.ghatis, basis.lagna_sign_number),
            (8, 28, 3)
        );
        // Güneş Koç 10°: tek burç, ikinci dilim; yücelme burcu.
        let sun = &report.grahas[0];
        assert_eq!(
            (sun.baladi, sun.jagradadi),
            (Baladi::Kumara, Jagradadi::Jagrat)
        );
        assert!(sun.lajjitadi.iter().any(|h| h.state == Lajjitadi::Garvita));
        assert_eq!(sun.shayanadi.map(|s| s.state), Some(Shayanadi::Sabha));
        for row in &report.grahas {
            assert!(row.shayanadi.is_some(), "{:?}", row.graha);
        }
    }

    #[test]
    fn without_a_lagna_or_a_sunrise_there_is_no_shayanadi_and_the_rest_stands() {
        let lons = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];
        let full = avasthas(&Positions::synthetic(Some(70.0), lons), Some(28));
        // Gün doğumu yok: lagna var, Shayanadi yok.
        let polar = avasthas(&Positions::synthetic(Some(70.0), lons), None);
        assert!(polar.time_known);
        assert_eq!(polar.shayanadi_basis, None);
        // Doğum saati bilinmiyor: ghati verilse de Shayanadi yok.
        let unknown = avasthas(&Positions::synthetic(None, lons), Some(28));
        assert!(!unknown.time_known);
        assert_eq!(unknown.shayanadi_basis, None);
        for (other, name) in [(&polar, "polar"), (&unknown, "unknown")] {
            for (row, base) in other.grahas.iter().zip(&full.grahas) {
                assert_eq!(row.shayanadi, None, "{name} {:?}", row.graha);
                assert_eq!(
                    (row.baladi, row.jagradadi),
                    (base.baladi, base.jagradadi),
                    "{name}"
                );
            }
        }
        // Lajjita lagnaya bağlıdır; öteki beş durum değildir.
        for (row, base) in unknown.grahas.iter().zip(&full.grahas) {
            let rest: Vec<_> = base
                .lajjitadi
                .iter()
                .filter(|h| h.state != Lajjitadi::Lajjita)
                .cloned()
                .collect();
            assert_eq!(row.lajjitadi, rest, "{:?}", row.graha);
        }
        assert_eq!(
            polar
                .grahas
                .iter()
                .map(|g| &g.lajjitadi)
                .collect::<Vec<_>>(),
            full.grahas.iter().map(|g| &g.lajjitadi).collect::<Vec<_>>()
        );
    }

    #[test]
    fn collector_reads_the_chart_and_counts_ghatis_from_the_last_sunrise() {
        // FakeEph: gün doğumu her gün n + 0,67; doğum 2 451 545,0 → son doğuş 2 451 544,67.
        // Geçen süre 0,33 gün = 19,8 ghati: tamamlanan 19. Lagna tropikal 70°, sidereal 46°
        // (Boğa: 2). Ay tropikal 95°, sidereal 71°: nakshatra 6 (66°40′–80°).
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        let report = compute_avasthas(&mut eph, &input(s.clone(), true)).unwrap();
        let basis = report.shayanadi_basis.expect("basis");
        assert_eq!(
            (basis.birth_nakshatra, basis.ghatis, basis.lagna_sign_number),
            (6, 19, 2)
        );
        let chart = compute_chart(&mut eph, JD, 41.0, 29.0, &s).unwrap();
        assert_eq!(report, avasthas(&Positions::from(&chart), Some(19)));
        // Güneş sidereal 346° (Balık 16°): nakshatra 26 (333°20′–346°40′), navamsa 5.
        // 26 × 1 × 5 + 6 + 19 + 2 = 157 = 13 × 12 + 1.
        let sun = report.grahas[0].shayanadi.expect("sun");
        assert_eq!((sun.nakshatra, sun.navamsa, sun.total), (26, 5, 157));
        assert_eq!((sun.state, sun.remainder), (Shayanadi::Shayana, 1));
    }

    #[test]
    fn collector_follows_time_known_the_rasi_frame_and_the_polar_day() {
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        let known = compute_avasthas(&mut eph, &input(s.clone(), true)).unwrap();
        let unknown = compute_avasthas(&mut eph, &input(s.clone(), false)).unwrap();
        assert!(!unknown.time_known);
        assert_eq!(unknown.shayanadi_basis, None);
        assert!(unknown.grahas.iter().all(|g| g.shayanadi.is_none()));
        assert_eq!(unknown.grahas.len(), 7);
        for (a, b) in unknown.grahas.iter().zip(&known.grahas) {
            assert_eq!((a.baladi, a.jagradadi), (b.baladi, b.jagradadi));
        }
        // Kutup günü: gün doğumu yok; lagna var, Shayanadi yok.
        let mut polar = FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        let report = compute_avasthas(&mut polar, &input(s.clone(), true)).unwrap();
        assert!(report.time_known);
        assert_eq!(report.shayanadi_basis, None);
        assert_eq!(
            report
                .grahas
                .iter()
                .map(|g| &g.lajjitadi)
                .collect::<Vec<_>>(),
            known
                .grahas
                .iter()
                .map(|g| &g.lajjitadi)
                .collect::<Vec<_>>()
        );
        // Tropikal-rasi kipi: burç ve burç içi derece tropikal çerçevedendir (Güneş Koç 10°:
        // yücelme burcu, ikinci dilim); nakshatra sidereal boylamdandır (346° → 26).
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        let report = compute_avasthas(&mut eph, &input(tropical, true)).unwrap();
        let sun = &report.grahas[0];
        assert_eq!(
            (sun.baladi, sun.jagradadi),
            (Baladi::Kumara, Jagradadi::Jagrat)
        );
        let hit = sun.shayanadi.expect("sun");
        assert_eq!((hit.nakshatra, hit.navamsa), (26, 4));
        // Lagna tropikal 70°: İkizler (3).
        assert_eq!(
            report.shayanadi_basis.map(|b| b.lagna_sign_number),
            Some(Sign::Gemini.index() as u8 + 1)
        );
    }

    #[test]
    fn a_lagna_number_gives_the_lagna_term_and_errors_are_reported_without_a_birth_time() {
        let mut eph = FakeEph::default();
        // 30. sayı: lagna Yengeç (4).
        let mut numbered = input(CalcSettings::default(), true);
        numbered.lagna_number = Some(30);
        let report = compute_avasthas(&mut eph, &numbered).unwrap();
        assert_eq!(report.shayanadi_basis.map(|b| b.lagna_sign_number), Some(4));
        // Düşük hassasiyet uyarısı rapora taşınır.
        let mut low = FakeEph {
            precision: Precision::Moshier,
            ..FakeEph::default()
        };
        let report = compute_avasthas(&mut low, &input(CalcSettings::default(), true)).unwrap();
        assert_eq!(report.warnings, [ChartWarning::LowPrecision]);
        // Hatalar saat bilinmese de bildirilir.
        let mut bad = input(CalcSettings::default(), false);
        bad.lat = 91.0;
        assert_eq!(
            compute_avasthas(&mut eph, &bad),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0
            })
        );
        let old = CalcSettings {
            schema_version: 1,
            ..CalcSettings::default()
        };
        assert_eq!(
            compute_avasthas(&mut eph, &input(old, false)),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 1,
                supported: 2
            })
        );
        // Sahte ephemeris'te cisim yoksa hata olduğu gibi döner.
        let mut missing = FakeEph::default();
        missing.positions.retain(|p| p.0 != Body::Moon);
        assert!(compute_avasthas(&mut missing, &input(CalcSettings::default(), true)).is_err());
    }
}
