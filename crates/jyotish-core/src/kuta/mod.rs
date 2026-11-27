//! Uyum (ileri modüller spec'i §9): Ashtakuta (Kuzey Hint geleneğinin 36 puanlık sistemi) ve
//! karşılıklı yerleşim. Girdi iki haritanın konumlarıdır; sistem yönlüdür ve roller ("damat
//! rolü", "gelin rolü") çağıran tarafından açıkça atanır. Sonuç bir olgu tablosudur: sekiz
//! kuta puanı, iki tarafın sınıfları, toplam, iki istisna olgusu, iki haritanın Kuja dosha
//! olgusu ve on sekiz satırlık karşılıklı yerleşim. Uyum yüzdesi ve hüküm yoktur.
//!
//! Kaynak: muhurta ve evlilik uyumu derlemelerindeki Ashtakuta kuralları; özgün olarak
//! uygulanır. Burçlar rasi çerçevesindedir (K5), nakshatra sidereal boylamdandır, evler rasi
//! evidir (K4). Lagnası olmayan konumlarda (doğum saati bilinmiyor) lagnaya bağlı değerler
//! `None` döner ve rapor `approximate` işaretlenir.

pub mod kuta_tables;
pub mod score;

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::dignity::sign_lord;
use crate::graha::Graha;
use crate::nature::{kuja_dosha, KujaDosha};
use crate::positions::{compute_positions, ChartInput, Positions};
use crate::settings::JD_UT_RANGE;
use crate::zodiac::Sign;
use crate::CoreError;

pub use kuta_tables::{
    gana_of, gana_points, nadi_of, varna_of, vashya_of, vashya_points, yoni_of, yoni_points, Gana,
    Nadi, Varna, Vashya, Yoni, GANA_POINTS, NADI_CYCLE, NAKSHATRA_GANA, NAKSHATRA_YONI,
    VASHYA_HALF_DEG, VASHYA_POINTS, YONI_ENEMIES, YONI_POINTS,
};
pub use score::{
    bhakoot_score, gana_score, lords_same_or_mutual_friends, maitri_score, nadi_score, tara_number,
    tara_score, varna_score, vashya_score, yoni_score, Kuta, MoonPoint, BHAKOOT_ZERO_COUNTS,
    KUTA_TOTAL_MAX, TARA_UNFAVOURABLE,
};

/// Yöntem kimliği (K7): Kuzey Hint Ashtakuta'sı, bu dosyadaki tablolar ve okuma kurallarıyla.
pub const KUTA_METHOD: &str = "kuta.ashtakuta_v1";

/// İki rol; rapordaki ikili alanların sırası da budur (damat rolü önce).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum KutaRole {
    Groom,
    Bride,
}

/// Bir kutanın puanı ve iki tarafın sınıfı.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct KutaScore {
    pub kuta: Kuta,
    pub score: f64,
    pub max: f64,
    /// Damat rolünün sınıf kimliği (`kuta_class_ids`); görünen ad i18n anahtarıdır.
    pub groom_class: String,
    /// Gelin rolünün sınıf kimliği.
    pub bride_class: String,
}

/// Karşılıklı yerleşimin bir satırı: bir tarafın bir grahası ve diğer tarafın haritasındaki
/// rasi evleri.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct OverlayRow {
    /// Grahanın sahibi.
    pub owner: KutaRole,
    pub graha: Graha,
    /// Grahanın kendi haritasının rasi çerçevesindeki burcu ve burç içi derecesi.
    pub sign: Sign,
    pub deg_in_sign: f64,
    pub retrograde: bool,
    /// Diğer tarafın lagnasından sayılan rasi evi; onun doğum saati bilinmiyorsa `None`.
    pub house_from_lagna: Option<u8>,
    /// Diğer tarafın Ay'ından sayılan rasi evi.
    pub house_from_moon: u8,
}

/// Doğum saati bilinmeyen haritada Ay'ın o yerel gün içinde (öğle ± 12 saat) nakshatra ya da
/// burç değiştirip değiştirmediği. Doğum saati bilinen haritada ikisi de `false`'tur.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MoonDayChange {
    pub nakshatra: bool,
    pub sign: bool,
}

/// `compute_compat` sonucu. İkili alanlar `[damat rolü, gelin rolü]` sırasındadır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct KutaReport {
    /// `Kuta::ALL` sırasında.
    pub scores: [KutaScore; 8],
    /// Sekiz puanın toplamı, 0..=36.
    pub total: f64,
    /// Bhakoot 0 iken iki Ay burcunun lordu aynı ya da iki yönlü dost. Puanı değiştirmez.
    pub bhakoot_exception: bool,
    /// Nadi 0 iken iki Ay aynı nakshatranın farklı padalarında ya da aynı burcun farklı
    /// nakshatralarında. Puanı değiştirmez.
    pub nadi_exception: bool,
    pub kuja_dosha: [KujaDosha; 2],
    /// Önce damat rolünün dokuz grahası (gelin rolünün haritasındaki evleriyle), sonra gelin
    /// rolünün dokuz grahası; her yarı `Graha::ALL` sırasında.
    pub overlay: [OverlayRow; 18],
    /// İki haritadan en az birinin doğum saati bilinmiyor: o haritanın Ay'ı öğle konumundadır.
    pub approximate: bool,
    pub moon_day_change: [MoonDayChange; 2],
    pub method: String,
}

/// Bir kutanın alabileceği sınıf kimlikleri (i18n anahtarlarının tek kaynağı). Tara'da dokuz
/// tara numarası, graha maitride yedi lord, Bhakoot'ta on iki burç.
pub fn kuta_class_ids(kuta: Kuta) -> Vec<String> {
    let owned = |ids: &[&str]| ids.iter().map(|s| (*s).to_owned()).collect();
    match kuta {
        Kuta::Varna => owned(&Varna::ALL.map(Varna::id)),
        Kuta::Vashya => owned(&Vashya::ALL.map(Vashya::id)),
        Kuta::Tara => (1..=9).map(|n: u8| n.to_string()).collect(),
        Kuta::Yoni => owned(&Yoni::ALL.map(Yoni::id)),
        Kuta::GrahaMaitri => Graha::ALL
            .into_iter()
            .filter(|g| !g.is_node())
            .map(variant_name)
            .collect(),
        Kuta::Gana => owned(&Gana::ALL.map(Gana::id)),
        Kuta::Bhakoot => Sign::ALL.into_iter().map(variant_name).collect(),
        Kuta::Nadi => owned(&Nadi::ALL.map(Nadi::id)),
    }
}

/// Birim varyantın adı; serde'nin yazdığı dizgiyle aynıdır (`"Mars"`, `"Aries"`).
fn variant_name<T: std::fmt::Debug>(value: T) -> String {
    format!("{value:?}")
}

/// Sekiz kutanın puanı ve sınıfları, `Kuta::ALL` sırasında.
pub fn kuta_scores(groom: &MoonPoint, bride: &MoonPoint) -> [KutaScore; 8] {
    let (g_lord, b_lord) = (sign_lord(groom.sign), sign_lord(bride.sign));
    Kuta::ALL.map(|kuta| {
        let (score, groom_class, bride_class) = match kuta {
            Kuta::Varna => {
                let (g, b) = (varna_of(groom.sign), varna_of(bride.sign));
                (varna_score(g, b), g.id().to_owned(), b.id().to_owned())
            }
            Kuta::Vashya => {
                let g = vashya_of(groom.sign, groom.deg_in_sign);
                let b = vashya_of(bride.sign, bride.deg_in_sign);
                (vashya_score(g, b), g.id().to_owned(), b.id().to_owned())
            }
            // Sınıf, her tarafın nakshatrasının diğerinden sayılan tara numarasıdır.
            Kuta::Tara => (
                tara_score(groom.nakshatra, bride.nakshatra),
                tara_number(bride.nakshatra, groom.nakshatra).to_string(),
                tara_number(groom.nakshatra, bride.nakshatra).to_string(),
            ),
            Kuta::Yoni => {
                let (g, b) = (yoni_of(groom.nakshatra), yoni_of(bride.nakshatra));
                (yoni_score(g, b), g.id().to_owned(), b.id().to_owned())
            }
            Kuta::GrahaMaitri => (
                maitri_score(g_lord, b_lord),
                variant_name(g_lord),
                variant_name(b_lord),
            ),
            Kuta::Gana => {
                let (g, b) = (gana_of(groom.nakshatra), gana_of(bride.nakshatra));
                (gana_score(g, b), g.id().to_owned(), b.id().to_owned())
            }
            Kuta::Bhakoot => (
                bhakoot_score(groom.sign, bride.sign),
                variant_name(groom.sign),
                variant_name(bride.sign),
            ),
            Kuta::Nadi => {
                let (g, b) = (nadi_of(groom.nakshatra), nadi_of(bride.nakshatra));
                (nadi_score(g, b), g.id().to_owned(), b.id().to_owned())
            }
        };
        KutaScore {
            kuta,
            score,
            max: kuta.max(),
            groom_class,
            bride_class,
        }
    })
}

/// Bhakoot istisnası: puan 0 iken iki Ay burcunun lordu aynı ya da iki yönlü dost.
pub fn bhakoot_exception(groom: &MoonPoint, bride: &MoonPoint) -> bool {
    bhakoot_score(groom.sign, bride.sign) == 0.0
        && lords_same_or_mutual_friends(sign_lord(groom.sign), sign_lord(bride.sign))
}

/// Nadi istisnası: puan 0 iken iki Ay aynı nakshatranın farklı padalarında ya da aynı burcun
/// farklı nakshatralarında.
pub fn nadi_exception(groom: &MoonPoint, bride: &MoonPoint) -> bool {
    nadi_score(nadi_of(groom.nakshatra), nadi_of(bride.nakshatra)) == 0.0
        && if groom.nakshatra == bride.nakshatra {
            groom.pada != bride.pada
        } else {
            groom.sign == bride.sign
        }
}

/// Karşılıklı yerleşim: önce damat rolünün dokuz grahası gelin rolünün lagnasından ve
/// Ay'ından, sonra tersi.
pub fn overlay(groom: &Positions, bride: &Positions) -> [OverlayRow; 18] {
    std::array::from_fn(|i| {
        let (owner, own, other) = if i < 9 {
            (KutaRole::Groom, groom, bride)
        } else {
            (KutaRole::Bride, bride, groom)
        };
        let point = own.point(Graha::ALL[i % 9]);
        OverlayRow {
            owner,
            graha: point.graha,
            sign: point.rasi_sign,
            deg_in_sign: point.deg_in_sign,
            retrograde: point.speed < 0.0,
            house_from_lagna: other.lagna_sign.map(|l| point.rasi_sign.house_from(l)),
            house_from_moon: point.rasi_sign.house_from(other.sign_of(Graha::Moon)),
        }
    })
}

/// Saf hesap: iki haritanın konumlarından uyum raporu. `moon_day_change` burada her zaman
/// boştur; onu ephemeris gören toplayıcı (`compute_compat`) doldurur.
pub fn kuta_report(groom: &Positions, bride: &Positions) -> KutaReport {
    let (g, b) = (MoonPoint::of(groom), MoonPoint::of(bride));
    let scores = kuta_scores(&g, &b);
    let total = scores.iter().map(|s| s.score).sum();
    KutaReport {
        scores,
        total,
        bhakoot_exception: bhakoot_exception(&g, &b),
        nadi_exception: nadi_exception(&g, &b),
        kuja_dosha: [kuja_dosha(groom), kuja_dosha(bride)],
        overlay: overlay(groom, bride),
        approximate: groom.lagna_sign.is_none() || bride.lagna_sign.is_none(),
        moon_day_change: [MoonDayChange::default(); 2],
        method: KUTA_METHOD.to_owned(),
    }
}

/// Doğum saati bilinmeyen haritanın yerel günü: an yerel 12:00'dir, gün öğle ± 12 saattir.
pub const UNKNOWN_TIME_HALF_DAY: f64 = 0.5;

/// Doğum saati bilinmeyen haritada Ay'ın yerel günün başındaki ve sonundaki nakshatrası ve
/// burcu karşılaştırılır. Uçlar hesap aralığına kırpılır.
fn moon_day_change<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<MoonDayChange, CoreError> {
    if input.time_known {
        return Ok(MoonDayChange::default());
    }
    let mut at = |offset: f64| -> Result<MoonPoint, CoreError> {
        let jd_ut = (input.jd_ut + offset).clamp(*JD_UT_RANGE.start(), *JD_UT_RANGE.end());
        let positions = compute_positions(
            eph,
            &ChartInput {
                jd_ut,
                ..input.clone()
            },
        )?;
        Ok(MoonPoint::of(&positions))
    };
    let (start, end) = (at(-UNKNOWN_TIME_HALF_DAY)?, at(UNKNOWN_TIME_HALF_DAY)?);
    Ok(MoonDayChange {
        nakshatra: start.nakshatra != end.nakshatra,
        sign: start.sign != end.sign,
    })
}

/// `compute_compat` komutunun hesabı: iki temel harita, konumlar, rapor. Ayar, an ve konum
/// denetimi `compute_chart`'ınkidir; her harita kendi ayarıyla hesaplanır. Doğum saati
/// bilinmeyen haritanın lagnası yoktur ve Ay'ının gün içindeki değişimi ayrıca bildirilir.
pub fn compute_compat<E: Ephemeris + ?Sized>(
    eph: &mut E,
    groom: &ChartInput,
    bride: &ChartInput,
) -> Result<KutaReport, CoreError> {
    let groom_positions = compute_positions(eph, groom)?;
    let bride_positions = compute_positions(eph, bride)?;
    let mut report = kuta_report(&groom_positions, &bride_positions);
    report.moon_day_change = [moon_day_change(eph, groom)?, moon_day_change(eph, bride)?];
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::{CalcSettings, ZodiacMode};
    use ephemeris::Body;

    /// Yalnızca Ay'ı verilen sentetik harita; diğer grahalar Koç 5°'dedir.
    fn chart(lagna: Option<f64>, moon: f64) -> Positions {
        let mut lons = [5.0; 9];
        lons[Graha::Moon as usize] = moon;
        Positions::synthetic(lagna, lons)
    }

    fn score(report: &KutaReport, kuta: Kuta) -> &KutaScore {
        &report.scores[kuta as usize]
    }

    #[test]
    fn report_lists_eight_scores_in_order_with_classes_and_a_total() {
        // Damat Ay 5° (Koç, Ashwini 2. pada), gelin Ay 100° (Yengeç, Pushya 3. pada).
        // Varna: Kshatriya – Brahmin → 0. Vashya: dört ayaklı – su → 1. Tara: Pushya'dan (8)
        // Ashwini'ye (1) sayım 21 → 3. tara (uygunsuz); Ashwini'den Pushya'ya sayım 8 → 8.
        // tara (uygun) → 1½. Yoni: at – koyun → 2. Maitri: Mars – Ay: Mars→Ay dost, Ay→Mars
        // nötr → 4. Gana: Deva – Deva → 6. Bhakoot: Yengeç Koç'tan 4. → 7. Nadi: Adi –
        // Madhya → 8. Toplam 0 + 1 + 1½ + 2 + 4 + 6 + 7 + 8 = 29½.
        let r = kuta_report(&chart(Some(0.0), 5.0), &chart(Some(0.0), 100.0));
        let got: Vec<(Kuta, f64, f64, &str, &str)> = r
            .scores
            .iter()
            .map(|s| {
                (
                    s.kuta,
                    s.score,
                    s.max,
                    s.groom_class.as_str(),
                    s.bride_class.as_str(),
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                (Kuta::Varna, 0.0, 1.0, "kshatriya", "brahmin"),
                (Kuta::Vashya, 1.0, 2.0, "quadruped", "water"),
                (Kuta::Tara, 1.5, 3.0, "3", "8"),
                (Kuta::Yoni, 2.0, 4.0, "horse", "sheep"),
                (Kuta::GrahaMaitri, 4.0, 5.0, "Mars", "Moon"),
                (Kuta::Gana, 6.0, 6.0, "deva", "deva"),
                (Kuta::Bhakoot, 7.0, 7.0, "Aries", "Cancer"),
                (Kuta::Nadi, 8.0, 8.0, "adi", "madhya"),
            ]
        );
        assert_eq!(r.total, 29.5);
        assert!(!r.bhakoot_exception && !r.nadi_exception && !r.approximate);
        assert_eq!(r.method, KUTA_METHOD);
        assert_eq!(r.moon_day_change, [MoonDayChange::default(); 2]);
    }

    #[test]
    fn every_class_in_a_report_is_a_listed_class_id() {
        for groom in 0..108 {
            for bride in [0, 17, 35, 53, 71, 89, 107] {
                let at = |pada: i32| f64::from(pada) * 10.0 / 3.0 + 1.0;
                let r = kuta_report(&chart(None, at(groom)), &chart(None, at(bride)));
                for s in &r.scores {
                    let ids = kuta_class_ids(s.kuta);
                    assert!(
                        ids.contains(&s.groom_class),
                        "{:?} {}",
                        s.kuta,
                        s.groom_class
                    );
                    assert!(
                        ids.contains(&s.bride_class),
                        "{:?} {}",
                        s.kuta,
                        s.bride_class
                    );
                }
            }
        }
        let counts = Kuta::ALL.map(|k| kuta_class_ids(k).len());
        assert_eq!(counts, [4, 5, 9, 14, 7, 3, 12, 3]);
    }

    #[test]
    fn bhakoot_exception_needs_a_zero_and_same_or_mutually_friendly_lords() {
        let ex = |g: f64, b: f64| {
            let r = kuta_report(&chart(None, g), &chart(None, b));
            (score(&r, Kuta::Bhakoot).score, r.bhakoot_exception)
        };
        // Koç – Akrep: 6/8, iki lord da Mars → istisna.
        assert_eq!(ex(5.0, 215.0), (0.0, true));
        // Koç – Balık: 2/12, Mars ve Jüpiter iki yönlü dost → istisna.
        assert_eq!(ex(5.0, 335.0), (0.0, true));
        // Koç – Başak: 6/8, Mars→Merkür düşman → istisna yok.
        assert_eq!(ex(5.0, 155.0), (0.0, false));
        // Koç – Aslan: 5/9, Mars ve Güneş iki yönlü dost → istisna.
        assert_eq!(ex(5.0, 125.0), (0.0, true));
        // Koç – Yengeç: sayım 4/10, puan 7: Mars ve Ay'ın ilişkisi ne olursa olsun istisna yok.
        assert_eq!(ex(5.0, 100.0), (7.0, false));
        // Aynı burç: puan 7, istisna yok (lord aynı olsa da).
        assert_eq!(ex(5.0, 20.0), (7.0, false));
    }

    #[test]
    fn nadi_exception_needs_a_zero_and_the_same_nakshatra_or_sign() {
        let ex = |g: f64, b: f64| {
            let r = kuta_report(&chart(None, g), &chart(None, b));
            (score(&r, Kuta::Nadi).score, r.nadi_exception)
        };
        // Aynı nakshatra, aynı pada (Ashwini 1. pada: 0°–3°20′): 0, istisna yok.
        assert_eq!(ex(1.0, 2.0), (0.0, false));
        // Aynı nakshatra, farklı pada (Ashwini 1. ve 3. pada) → istisna.
        assert_eq!(ex(1.0, 8.0), (0.0, true));
        // Aynı burç, farklı nakshatra, aynı nadi: Boğa'da Krittika (Antya, 30°–40°) ve Rohini
        // (Antya, 40°–53°20′) → istisna.
        assert_eq!(ex(35.0, 45.0), (0.0, true));
        // Aynı nakshatra, farklı pada, farklı burç: Krittika 1. pada Koç'ta (27°), 2. pada
        // Boğa'da (31°) → istisna (aynı nakshatranın farklı padaları).
        assert_eq!(ex(27.0, 31.0), (0.0, true));
        // Aynı nadi, farklı burç ve nakshatra: Ashwini (Adi, Koç) ve Ardra (Adi, İkizler) →
        // istisna yok.
        assert_eq!(ex(1.0, 70.0), (0.0, false));
        // Farklı nadi: puan 8, istisna yok (aynı burçta olsalar da: Ashwini ve Bharani).
        assert_eq!(ex(1.0, 15.0), (8.0, false));
    }

    #[test]
    fn overlay_counts_each_graha_from_the_other_lagna_and_moon() {
        // Damat: lagna Koç, Ay Boğa (40°), Güneş Aslan (130°), diğerleri Koç 5°.
        // Gelin: lagna Terazi (185°), Ay Oğlak (280°), Güneş Koç 5°, diğerleri Koç 5°.
        let mut g = [5.0; 9];
        g[Graha::Sun as usize] = 130.0;
        g[Graha::Moon as usize] = 40.0;
        let groom = Positions::synthetic(Some(0.0), g);
        let bride = chart(Some(185.0), 280.0);
        let rows = overlay(&groom, &bride);
        assert_eq!(rows.len(), 18);
        for (i, row) in rows.iter().enumerate() {
            let owner = if i < 9 {
                KutaRole::Groom
            } else {
                KutaRole::Bride
            };
            assert_eq!((row.owner, row.graha), (owner, Graha::ALL[i % 9]));
            assert!(!row.retrograde);
        }
        // Damadın Güneş'i Aslan: gelinin lagnası Terazi'den 11., Ay'ı Oğlak'tan 8. ev.
        assert_eq!(
            (
                rows[0].sign,
                rows[0].house_from_lagna,
                rows[0].house_from_moon
            ),
            (Sign::Leo, Some(11), 8)
        );
        // Damadın Ay'ı Boğa: Terazi'den 8., Oğlak'tan 5. ev.
        assert_eq!(
            (rows[1].house_from_lagna, rows[1].house_from_moon),
            (Some(8), 5)
        );
        // Gelinin Ay'ı Oğlak: damadın lagnası Koç'tan 10., Ay'ı Boğa'dan 9. ev.
        assert_eq!(
            (
                rows[10].sign,
                rows[10].deg_in_sign,
                rows[10].house_from_lagna,
                rows[10].house_from_moon
            ),
            (Sign::Capricorn, 10.0, Some(10), 9)
        );
        // Gelinin Güneş'i Koç: Koç'tan 1., Boğa'dan 12. ev.
        assert_eq!(
            (rows[9].house_from_lagna, rows[9].house_from_moon),
            (Some(1), 12)
        );

        // Gelinin doğum saati bilinmiyor: damadın grahalarının gelin lagnasından evi yoktur;
        // gelinin grahalarının damat lagnasından evi durur.
        let rows = overlay(&groom, &bride.clone().without_lagna());
        assert!(rows[..9].iter().all(|r| r.house_from_lagna.is_none()));
        assert!(rows[9..].iter().all(|r| r.house_from_lagna.is_some()));
        assert_eq!(rows[0].house_from_moon, 8);

        // Geri giden graha: hız negatifse `retrograde`.
        let mut slow = groom.clone();
        slow.grahas[Graha::Saturn as usize].speed = -0.01;
        assert!(overlay(&slow, &bride)[Graha::Saturn as usize].retrograde);
    }

    #[test]
    fn kuja_dosha_and_approximate_follow_each_chart() {
        // Damat: lagna Koç, Mars Koç'ta (1. ev) → lagnadan dosha. Gelin: saat bilinmiyor.
        let groom = chart(Some(0.0), 100.0);
        let bride = chart(None, 200.0);
        let r = kuta_report(&groom, &bride);
        assert_eq!(r.kuja_dosha, [kuja_dosha(&groom), kuja_dosha(&bride)]);
        assert_eq!(r.kuja_dosha[0].from_lagna, Some(true));
        assert_eq!(r.kuja_dosha[1].from_lagna, None);
        assert!(r.approximate);
        assert!(kuta_report(&bride, &groom).approximate);
        assert!(!kuta_report(&groom, &groom).approximate);
    }

    fn input(jd_ut: f64, settings: CalcSettings, time_known: bool) -> ChartInput {
        ChartInput {
            jd_ut,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known,
            lagna_number: None,
        }
    }

    #[test]
    fn collector_uses_both_charts_the_rasi_frame_and_reports_the_chart_errors() {
        const JD: f64 = 2_451_545.0;
        let mut eph = FakeEph::default();
        let known = input(JD, CalcSettings::default(), true);
        // FakeEph: Ay tropikal 95°, günde 13,2°; ayanamsa 24° → sidereal 71° (İkizler, Ardra).
        // Bir gün sonra 84,2° (İkizler, Punarvasu).
        let later = input(JD + 1.0, CalcSettings::default(), true);
        let r = compute_compat(&mut eph, &known, &later).expect("report");
        let groom = compute_positions(&mut eph, &known).expect("groom");
        let bride = compute_positions(&mut eph, &later).expect("bride");
        assert_eq!(r, kuta_report(&groom, &bride));
        assert_eq!(score(&r, Kuta::Bhakoot).groom_class, "Gemini");
        assert_eq!(score(&r, Kuta::Nadi).groom_class, "adi");
        assert_eq!(score(&r, Kuta::Nadi).bride_class, "adi");
        // Ardra ve Punarvasu: aynı nadi (Adi), aynı burç, farklı nakshatra → istisna.
        assert!(r.nadi_exception);
        assert!(!r.approximate);

        // Tropikal rasi: burç tropikal boylamdan (95° → Yengeç), nakshatra yine siderealden.
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        let t = compute_compat(&mut eph, &input(JD, tropical, true), &known).expect("tropical");
        assert_eq!(score(&t, Kuta::Bhakoot).groom_class, "Cancer");
        assert_eq!(score(&t, Kuta::Bhakoot).bride_class, "Gemini");
        assert_eq!(score(&t, Kuta::Nadi).groom_class, "adi");

        // Doğum saati bilinmiyor: lagna yok, rapor yaklaşık. Ay gün içinde 71° ∓ 6,6° arasında
        // gezer: 64,4° (Mrigashira, 53°20′–66°40′) → 77,6° (Ardra): nakshatra değişir, burç
        // (İkizler) değişmez.
        let unknown = input(JD, CalcSettings::default(), false);
        let u = compute_compat(&mut eph, &known, &unknown).expect("unknown");
        assert!(u.approximate);
        assert_eq!(
            u.moon_day_change,
            [
                MoonDayChange::default(),
                MoonDayChange {
                    nakshatra: true,
                    sign: false
                }
            ]
        );
        assert!(u.overlay[..9].iter().all(|r| r.house_from_lagna.is_none()));
        assert!(u.overlay[9..].iter().all(|r| r.house_from_lagna.is_some()));
        assert_eq!(u.kuja_dosha[1].from_lagna, None);
        assert_eq!(
            u.scores,
            scores_from_the_two_moons(&mut eph, &known, &unknown)
        );

        // Ay gün içinde burç da değiştirir: Ay'ı sidereal 89°'ye al (tropikal 113°):
        // 82,4° (İkizler) → 95,6° (Yengeç).
        eph.positions
            .iter_mut()
            .filter(|p| p.0 == Body::Moon)
            .for_each(|p| p.1 = 113.0);
        let s = compute_compat(&mut eph, &unknown, &known).expect("sign change");
        assert_eq!(
            s.moon_day_change[0],
            MoonDayChange {
                nakshatra: true,
                sign: true
            }
        );

        // Hatalar `compute_chart`'ınkidir; hangi haritadan geldiği parametreye yazılmaz.
        let bad = ChartInput {
            lat: 91.0,
            ..known.clone()
        };
        assert!(matches!(
            compute_compat(&mut eph, &bad, &known),
            Err(CoreError::InvalidLocation { .. })
        ));
        assert!(matches!(
            compute_compat(&mut eph, &known, &bad),
            Err(CoreError::InvalidLocation { .. })
        ));
    }

    /// Puanlar yalnızca iki Ay'a bağlıdır: lagnasız konumlardan aynı puanlar çıkar.
    fn scores_from_the_two_moons(
        eph: &mut FakeEph,
        groom: &ChartInput,
        bride: &ChartInput,
    ) -> [KutaScore; 8] {
        let g = compute_positions(eph, groom).expect("groom");
        let b = compute_positions(eph, bride).expect("bride");
        kuta_scores(&MoonPoint::of(&g), &MoonPoint::of(&b))
    }

    #[test]
    fn the_day_window_is_clamped_to_the_calculation_range() {
        let mut eph = FakeEph::default();
        let edge = input(*JD_UT_RANGE.end(), CalcSettings::default(), false);
        let known = input(2_451_545.0, CalcSettings::default(), true);
        // Aralığın son anı: +12 saat aralığın dışındadır; kırpılmasa hata dönerdi.
        assert!(compute_compat(&mut eph, &edge, &known).is_ok());
    }
}
