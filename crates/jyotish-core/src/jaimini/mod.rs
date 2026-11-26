//! Jaimini çekirdeği (ileri modüller spec'i §5): chara karaka, karakamsha, rashi drishti, eş
//! lordlar, arudha padalar ve argala. Hesap fonksiyonları yalnızca `Positions` görür (K3);
//! `compute_jaimini` temel haritayı hesaplayıp onlara veren ince toplayıcıdır. Chara dasha
//! Alt proje 3.3'tedir ve bu modülün `jaimini_lord` ile `counts_forward` fonksiyonlarını
//! kullanır.

mod argala;
mod arudha;
mod karaka;
mod lords;
mod rashi;

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{compute_chart, ChartWarning};
use crate::drishti::SignAspect;
use crate::positions::{ChartInput, Positions};
use crate::settings::{AdvancedSettings, KarakaScheme};
use crate::CoreError;

pub use argala::{
    argala_row, argala_rows, argala_status, ArgalaPair, ArgalaRow, ArgalaStatus, ARGALA_PAIRS,
    THIRD_HOUSE_MALEFICS,
};
pub use arudha::{arudha_of, arudha_padas, ArudhaPada};
pub use karaka::{
    chara_karakas, karaka_roles, karakamsha, navamsa_sign, CharaKaraka, KarakaRole, Karakamsha,
    EIGHT_KARAKA_ROLES, KARAKA_TIE_DEG, SEVEN_KARAKA_ROLES,
};
pub use lords::{co_lord_choice, co_lords, jaimini_lord, stronger_co_lord, CoLordChoice};
pub use rashi::{
    counts_forward, directed_count, graha_rashi_drishti, rashi_drishti_matrix, rashi_drishti_signs,
    sign_kind, SignKind,
};

/// `JaiminiReport::method`: bu modüldeki yöntemin kimliği.
pub const JAIMINI_METHOD: &str = "jaimini.core_v1";

/// `compute_jaimini` sonucu. AI yorum paneli için sözleşmedir: alan adları değişirse AI
/// paketleyicisi de değişir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct JaiminiReport {
    /// Kullanılan karaka şeması (`settings.advanced.karaka_scheme`).
    pub scheme: KarakaScheme,
    /// Eş lordlar kullanıldı mı (`settings.advanced.jaimini_co_lords`).
    pub co_lords: bool,
    pub method: String,
    /// 8 (`Eight`) ya da 7 (`Seven`) kayıt, rol sırasında.
    pub karakas: Vec<CharaKaraka>,
    pub karakamsha: Karakamsha,
    /// `rashi_drishti[from][to]`, `Sign::ALL` sırasında.
    pub rashi_drishti: [[bool; 12]; 12],
    /// 27 kayıt: `Graha::ALL` sırasında, her graha için üç hedef burç ve oradaki grahalar.
    pub graha_rashi_drishti: Vec<SignAspect>,
    /// A1…A12; doğum saati bilinmiyorsa `None`.
    pub arudhas: Option<[ArudhaPada; 12]>,
    /// On iki referans burç: lagna burcundan sıralı; doğum saati bilinmiyorsa Koç'tan.
    pub argala: [ArgalaRow; 12],
    pub warnings: Vec<ChartWarning>,
}

/// Saf hesap: konumlardan ve iki ayardan rapor. `warnings` boştur; toplayıcı doldurur.
pub fn jaimini_report(positions: &Positions, advanced: &AdvancedSettings) -> JaiminiReport {
    let karakas = chara_karakas(positions, advanced.karaka_scheme);
    // Sıra hiçbir şemada boş değildir: ilk kayıt Atmakaraka'dır.
    let atmakaraka = karakas[0].graha;
    JaiminiReport {
        scheme: advanced.karaka_scheme,
        co_lords: advanced.jaimini_co_lords,
        method: JAIMINI_METHOD.to_owned(),
        karakamsha: karakamsha(positions, atmakaraka),
        karakas,
        rashi_drishti: rashi_drishti_matrix(),
        graha_rashi_drishti: graha_rashi_drishti(positions),
        arudhas: arudha_padas(positions, advanced.jaimini_co_lords),
        argala: argala_rows(positions),
        warnings: Vec::new(),
    }
}

/// `compute_jaimini` komutunun hesabı: temel harita, konumlar, rapor. `input.time_known`
/// yanlışsa lagna boşaltılır: padalar `None` olur ve argala satırları Koç'tan sıralanır.
/// Temel haritanın konum duyarlılığı uyarısı (`LowPrecision`) rapora taşınır; diğer harita
/// uyarıları (ev sistemi, gün doğumu) bu modülü ilgilendirmez.
pub fn compute_jaimini<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<JaiminiReport, CoreError> {
    let chart = compute_chart(eph, input.jd_ut, input.lat, input.lon, &input.settings)?;
    let positions = Positions::from(&chart);
    let positions = if input.time_known {
        positions
    } else {
        positions.without_lagna()
    };
    let mut report = jaimini_report(&positions, &input.settings.advanced);
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
    use crate::fake::FakeEph;
    use crate::graha::Graha;
    use crate::settings::CalcSettings;
    use crate::zodiac::Sign;
    use ephemeris::Precision;

    fn input(advanced: AdvancedSettings, time_known: bool) -> ChartInput {
        ChartInput {
            jd_ut: 2_451_545.0,
            lat: 41.0,
            lon: 29.0,
            settings: CalcSettings {
                advanced,
                ..CalcSettings::default()
            },
            time_known,
        }
    }

    #[test]
    fn report_follows_the_two_advanced_settings() {
        // Lagna Koç; Güneş Aslan 12°, Ay Boğa 7°, Mars Yengeç 22°, Merkür Başak 3°, Jüpiter
        // Yay 16°, Venüs Balık 25°, Satürn Terazi 9°, Rahu İkizler 2° (etkin 28°), Ketu Yay 2°.
        let p = Positions::synthetic(
            Some(5.0),
            [132.0, 37.0, 112.0, 153.0, 256.0, 355.0, 189.0, 62.0, 242.0],
        );
        let eight = jaimini_report(&p, &AdvancedSettings::default());
        assert_eq!(eight.method, JAIMINI_METHOD);
        assert_eq!((eight.scheme, eight.co_lords), (KarakaScheme::Eight, true));
        assert_eq!(eight.karakas.len(), 8);
        assert_eq!(eight.karakas[0].graha, Graha::Rahu);
        assert_eq!(eight.karakamsha.atmakaraka, Graha::Rahu);
        assert_eq!(eight.rashi_drishti, rashi_drishti_matrix());
        assert_eq!(eight.graha_rashi_drishti.len(), 27);
        assert_eq!(eight.argala[0].reference, Sign::Aries);
        assert!(eight.warnings.is_empty());
        // 8. ev Akrep: Mars Yengeç'te yalnız, Ketu Yay'da Jüpiter'le: Ketu.
        assert_eq!(eight.arudhas.expect("lagna")[7].lord, Graha::Ketu);

        let seven = jaimini_report(
            &p,
            &AdvancedSettings {
                karaka_scheme: KarakaScheme::Seven,
                jaimini_co_lords: false,
            },
        );
        assert_eq!((seven.scheme, seven.co_lords), (KarakaScheme::Seven, false));
        assert_eq!(seven.karakas.len(), 7);
        // Rahu sıralanmaz: en yüksek derece Venüs 25°.
        assert_eq!(seven.karakamsha.atmakaraka, Graha::Venus);
        assert_eq!(seven.arudhas.expect("lagna")[7].lord, Graha::Mars);
        // Argala ve rashi drishti ayarlardan bağımsızdır.
        assert_eq!(seven.argala, eight.argala);
        assert_eq!(seven.graha_rashi_drishti, eight.graha_rashi_drishti);
    }

    #[test]
    fn collector_hides_lagna_dependent_values_when_time_is_unknown() {
        let mut eph = FakeEph::default();
        let known = compute_jaimini(&mut eph, &input(AdvancedSettings::default(), true)).unwrap();
        let chart =
            compute_chart(&mut eph, 2_451_545.0, 41.0, 29.0, &CalcSettings::default()).unwrap();
        let positions = Positions::from(&chart);
        let mut want = jaimini_report(&positions, &AdvancedSettings::default());
        want.warnings = known.warnings.clone();
        assert_eq!(known, want);
        assert!(known.arudhas.is_some());
        assert_eq!(known.argala[0].reference, chart.lagna.sign);

        let unknown =
            compute_jaimini(&mut eph, &input(AdvancedSettings::default(), false)).unwrap();
        assert_eq!(unknown.arudhas, None);
        assert_eq!(unknown.argala[0].reference, Sign::Aries);
        // Lagnaya bağlı olmayan her şey aynıdır.
        assert_eq!(unknown.karakas, known.karakas);
        assert_eq!(unknown.karakamsha, known.karakamsha);
        assert_eq!(unknown.graha_rashi_drishti, known.graha_rashi_drishti);
        for row in &unknown.argala {
            let same = known
                .argala
                .iter()
                .find(|r| r.reference == row.reference)
                .expect("row");
            assert_eq!(row, same);
        }
    }

    #[test]
    fn collector_carries_the_low_precision_warning_and_nothing_else() {
        let mut eph = FakeEph::default();
        let clean = compute_jaimini(&mut eph, &input(AdvancedSettings::default(), true)).unwrap();
        assert!(clean.warnings.is_empty());

        let mut moshier = FakeEph {
            precision: Precision::Moshier,
            ..FakeEph::default()
        };
        for time_known in [true, false] {
            let report = compute_jaimini(
                &mut moshier,
                &input(AdvancedSettings::default(), time_known),
            )
            .unwrap();
            assert_eq!(report.warnings, [ChartWarning::LowPrecision]);
            // Uyarı yalnızca taşınır: hesap aynıdır.
            assert_eq!(report.karakas, clean.karakas);
        }
    }

    #[test]
    fn collector_reports_the_chart_errors() {
        let mut eph = FakeEph::default();
        let mut bad = input(AdvancedSettings::default(), true);
        bad.lat = 91.0;
        assert_eq!(
            compute_jaimini(&mut eph, &bad),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0
            })
        );
    }
}
