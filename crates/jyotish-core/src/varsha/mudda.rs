//! Mudda dasa: Vimshottari paylarının bir yıla sığdırılması. Dokuz dönem yılın başında (dönüş
//! anında) başlar ve sonraki dönüşte biter; her dönem yılın `gün(lord) / 360` kadarını alır.
//!
//! İlk dönemin lordu doğum nakshatrasından ve tamamlanan yıldan bulunur: doğum nakshatrasının
//! sırası (Ashwini 1) ile tamamlanan yıl toplanır, 2 çıkarılır ve 9'a bölünür; bölümden artan
//! sayı `MUDDA_ORDER` içindeki sırayı verir (1 Güneş … 8 Ketu, 0 Venüs). Sonraki dönemler aynı
//! sırayla, çevrimsel olarak yürür. İlk dönem tamdır: Ay'ın nakshatrada geçtiği yay için kesinti
//! yapılmaz.
//!
//! Alt dönemler (ikinci seviye) üst dönemin lordundan başlar ve üst dönemi aynı paylarla böler.

use serde::{Deserialize, Serialize};

use crate::dasha::DashaPeriod;
use crate::graha::Graha;
use crate::CoreError;

/// Mudda dasa lordlarının sırası: Vimshottari sırasının Güneş'ten başlayan yazımı.
pub const MUDDA_ORDER: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Rahu,
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Mercury,
    Graha::Ketu,
    Graha::Venus,
];

/// Dokuz dönemin gün toplamı: payların paydası.
pub const MUDDA_TOTAL_DAYS: f64 = 360.0;
/// Raporun taşıdığı seviye sayısı: dönem ve alt dönem.
pub const MUDDA_LEVELS: u8 = 2;
/// İlk lord kuralında toplamdan çıkarılan sayı.
pub const MUDDA_OFFSET: u32 = 2;

/// Lordun 360 günlük yıldaki günü (Vimshottari yılının üç katı).
pub fn mudda_days(lord: Graha) -> f64 {
    match lord {
        Graha::Sun => 18.0,
        Graha::Moon => 30.0,
        Graha::Mars => 21.0,
        Graha::Rahu => 54.0,
        Graha::Jupiter => 48.0,
        Graha::Saturn => 57.0,
        Graha::Mercury => 51.0,
        Graha::Ketu => 21.0,
        Graha::Venus => 60.0,
    }
}

fn order_index(lord: Graha) -> usize {
    match lord {
        Graha::Sun => 0,
        Graha::Moon => 1,
        Graha::Mars => 2,
        Graha::Rahu => 3,
        Graha::Jupiter => 4,
        Graha::Saturn => 5,
        Graha::Mercury => 6,
        Graha::Ketu => 7,
        Graha::Venus => 8,
    }
}

/// Bölümden artan: `(doğum nakshatrası + tamamlanan yıl − 2)`'nin 9'a bölümünden artan, 0..=8.
/// `birth_nakshatra` 1..=27 olmalıdır (çağıran denetler).
pub fn mudda_remainder(birth_nakshatra: u8, age: u8) -> u8 {
    // En küçük toplam 1 + 0 = 1'dir; 9 eklemek çıkarmayı eksiye düşürmez ve artanı değiştirmez.
    ((u32::from(birth_nakshatra) + u32::from(age) + 9 - MUDDA_OFFSET) % 9) as u8
}

/// Yılın ilk mudda dasa lordu: artan 1 Güneş, 2 Ay, 3 Mars, 4 Rahu, 5 Jüpiter, 6 Satürn,
/// 7 Merkür, 8 Ketu, 0 Venüs.
pub fn mudda_first_lord(birth_nakshatra: u8, age: u8) -> Graha {
    let remainder = usize::from(mudda_remainder(birth_nakshatra, age));
    MUDDA_ORDER[(remainder + 8) % 9]
}

/// Bir yılın mudda dasası.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MuddaDasha {
    /// Doğum nakshatrası (Ay), 1..=27.
    pub birth_nakshatra: u8,
    /// Tamamlanan yıl.
    pub age: u8,
    /// İlk lord kuralının bölümden artanı, 0..=8.
    pub remainder: u8,
    pub first_lord: Graha,
    /// Dönemler ve alt dönemleri, ön-sırada (her dönemin ardından dokuz alt dönemi): 9 + 81.
    /// `lords` kökten yoldur; zamanlar UT Jülyen günüdür.
    pub periods: Vec<DashaPeriod>,
}

/// `[start, end)` aralığını `first`'ten başlayan dokuz döneme böler. Bitişler birikimli günden
/// hesaplanır; sonuncusu `end`'e tam eşittir.
fn subdivide(path: &[Graha], first: Graha, start: f64, end: f64) -> Vec<DashaPeriod> {
    let level = (path.len() + 1) as u8;
    let span = end - start;
    let first = order_index(first);
    let mut periods = Vec::with_capacity(9);
    let mut from = start;
    let mut days = 0.0;
    for k in 0..9 {
        let lord = MUDDA_ORDER[(first + k) % 9];
        days += mudda_days(lord);
        let to = if k == 8 {
            end
        } else {
            start + span * days / MUDDA_TOTAL_DAYS
        };
        let mut lords = path.to_vec();
        lords.push(lord);
        periods.push(DashaPeriod {
            lords,
            level,
            start_jd_ut: from,
            end_jd_ut: to,
            has_children: level < MUDDA_LEVELS,
        });
        from = to;
    }
    periods
}

/// `[start_jd_ut, end_jd_ut)` yılının mudda dasası: dokuz dönem ve seksen bir alt dönem.
/// `birth_nakshatra` 1..=27 değilse `InvalidDashaInput("nakshatra")`; uçlar sonlu ve sıralı
/// değilse `InvalidDashaInput("range")`.
pub fn mudda_dasha(
    birth_nakshatra: u8,
    age: u8,
    start_jd_ut: f64,
    end_jd_ut: f64,
) -> Result<MuddaDasha, CoreError> {
    if !(1..=27).contains(&birth_nakshatra) {
        return Err(CoreError::InvalidDashaInput("nakshatra"));
    }
    // NaN karşılaştırması yanlıştır: sonlu olmayan uçlar da burada düşer.
    if !(start_jd_ut.is_finite() && end_jd_ut.is_finite() && start_jd_ut < end_jd_ut) {
        return Err(CoreError::InvalidDashaInput("range"));
    }
    let first_lord = mudda_first_lord(birth_nakshatra, age);
    let mut periods = Vec::with_capacity(9 + 81);
    for period in subdivide(&[], first_lord, start_jd_ut, end_jd_ut) {
        let children = subdivide(
            &period.lords,
            first_lord_of(&period),
            period.start_jd_ut,
            period.end_jd_ut,
        );
        periods.push(period);
        periods.extend(children);
    }
    Ok(MuddaDasha {
        birth_nakshatra,
        age,
        remainder: mudda_remainder(birth_nakshatra, age),
        first_lord,
        periods,
    })
}

/// Alt dönemler üst dönemin lordundan başlar.
fn first_lord_of(period: &DashaPeriod) -> Graha {
    *period.lords.last().expect("a period has a lord")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dasha::{vimshottari_years, VIMSHOTTARI_ORDER, VIMSHOTTARI_TOTAL_YEARS};

    fn level(d: &MuddaDasha, n: u8) -> Vec<&DashaPeriod> {
        d.periods.iter().filter(|p| p.level == n).collect()
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn the_nine_shares_total_360_days_and_are_three_days_per_vimshottari_year() {
        assert_eq!(
            MUDDA_ORDER.map(mudda_days),
            [18.0, 30.0, 21.0, 54.0, 48.0, 57.0, 51.0, 21.0, 60.0]
        );
        let total: f64 = MUDDA_ORDER.iter().map(|&g| mudda_days(g)).sum();
        assert_eq!(total, MUDDA_TOTAL_DAYS);
        // İkinci yazım: pay Vimshottari yılının 120'de biridir; 360 günde yıl başına üç gün.
        for lord in MUDDA_ORDER {
            assert_eq!(
                mudda_days(lord),
                vimshottari_years(lord) * MUDDA_TOTAL_DAYS / VIMSHOTTARI_TOTAL_YEARS,
                "{lord:?}"
            );
        }
        // Sıra, Vimshottari sırasının Güneş'ten başlayan yazımıdır.
        for (i, lord) in MUDDA_ORDER.into_iter().enumerate() {
            assert_eq!(lord, VIMSHOTTARI_ORDER[(i + 2) % 9]);
            assert_eq!(order_index(lord), i);
        }
    }

    #[test]
    fn the_first_lord_follows_the_remainder_rule() {
        // Krittika (3), tamamlanan yıl 0: 3 + 0 − 2 = 1 → artan 1 → Güneş.
        assert_eq!(mudda_remainder(3, 0), 1);
        assert_eq!(mudda_first_lord(3, 0), Graha::Sun);
        // Ashwini (1), yıl 0: 1 + 0 − 2 = −1; 9 eklenir: 8 → Ketu.
        assert_eq!(mudda_remainder(1, 0), 8);
        assert_eq!(mudda_first_lord(1, 0), Graha::Ketu);
        // Bharani (2), yıl 0: 0 → artan 0 dokuzuncu lorddur: Venüs.
        assert_eq!(mudda_remainder(2, 0), 0);
        assert_eq!(mudda_first_lord(2, 0), Graha::Venus);
        // Swati (15), yıl 26: 15 + 26 − 2 = 39 = 4 × 9 + 3 → Mars.
        assert_eq!(mudda_remainder(15, 26), 3);
        assert_eq!(mudda_first_lord(15, 26), Graha::Mars);
        // Revati (27), yıl 120: 27 + 120 − 2 = 145 = 16 × 9 + 1 → Güneş.
        assert_eq!(mudda_first_lord(27, 120), Graha::Sun);
        // Rohini (4), yıl 3: 4 + 3 − 2 = 5 → Jüpiter. Ardra (6), yıl 0: 4 → Rahu.
        assert_eq!(mudda_first_lord(4, 3), Graha::Jupiter);
        assert_eq!(mudda_first_lord(6, 0), Graha::Rahu);
        // Pushya (8), yıl 0: 6 → Satürn. Ashlesha (9), yıl 0: 7 → Merkür. Rohini, yıl 0: 2 → Ay.
        assert_eq!(mudda_first_lord(8, 0), Graha::Saturn);
        assert_eq!(mudda_first_lord(9, 0), Graha::Mercury);
        assert_eq!(mudda_first_lord(4, 0), Graha::Moon);
    }

    #[test]
    fn second_formulation_the_birth_nakshatra_lord_advanced_one_step_a_year() {
        // Yıl 0'da ilk lord doğum nakshatrasının Vimshottari lordudur ve her yıl sırada bir
        // adım ilerler. Kural ile bu yürüyüş her nakshatra ve her yaşta aynı lordu verir.
        for nakshatra in 1..=27_u8 {
            let mut walk = (usize::from(nakshatra) - 1) % 9;
            for age in 0..=120_u8 {
                assert_eq!(
                    mudda_first_lord(nakshatra, age),
                    VIMSHOTTARI_ORDER[walk],
                    "nakshatra {nakshatra}, age {age}"
                );
                walk = (walk + 1) % 9;
            }
        }
    }

    #[test]
    fn nine_periods_fill_the_year_in_order_from_the_first_lord() {
        // 360 günlük yıl: sınırlar tam gündür. Swati, yıl 26: ilk lord Mars.
        let d = mudda_dasha(15, 26, 1000.0, 1360.0).unwrap();
        assert_eq!(
            (d.birth_nakshatra, d.age, d.remainder, d.first_lord),
            (15, 26, 3, Graha::Mars)
        );
        assert_eq!(d.periods.len(), 9 + 81);
        let top = level(&d, 1);
        let lords: Vec<_> = top.iter().map(|p| p.lords[0]).collect();
        assert_eq!(
            lords,
            [
                Graha::Mars,
                Graha::Rahu,
                Graha::Jupiter,
                Graha::Saturn,
                Graha::Mercury,
                Graha::Ketu,
                Graha::Venus,
                Graha::Sun,
                Graha::Moon
            ]
        );
        // Mars 21, Rahu 54, Jüpiter 48, Satürn 57, Merkür 51, Ketu 21, Venüs 60, Güneş 18, Ay 30.
        let ends: Vec<_> = top.iter().map(|p| p.end_jd_ut).collect();
        assert_eq!(
            ends,
            [1021.0, 1075.0, 1123.0, 1180.0, 1231.0, 1252.0, 1312.0, 1330.0, 1360.0]
        );
        assert_eq!(top[0].start_jd_ut, 1000.0);
        for w in top.windows(2) {
            assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
        }
        assert!(top
            .iter()
            .all(|p| p.has_children && p.lords.len() == 1 && p.level == 1));
    }

    #[test]
    fn the_year_between_two_returns_is_split_in_the_same_shares() {
        // Dönüşten dönüşe yıl 365,2564 gündür: her dönem aynı oranda uzar, yıl boşluksuz dolar.
        let (start, end) = (2_461_000.25, 2_461_365.506_363);
        let d = mudda_dasha(3, 0, start, end).unwrap();
        let top = level(&d, 1);
        assert_eq!(top[0].lords, [Graha::Sun]);
        assert!(close(
            top[0].end_jd_ut - top[0].start_jd_ut,
            (end - start) * 18.0 / 360.0
        ));
        assert_eq!(top[0].start_jd_ut, start);
        assert_eq!(top[8].end_jd_ut, end);
        let sum: f64 = top.iter().map(|p| p.end_jd_ut - p.start_jd_ut).sum();
        assert!(close(sum, end - start));
    }

    #[test]
    fn sub_periods_start_from_the_parent_lord_and_fill_the_parent() {
        let d = mudda_dasha(15, 26, 1000.0, 1360.0).unwrap();
        // Ön-sıra: her dönemin ardından dokuz alt dönemi.
        for (i, chunk) in d.periods.chunks(10).enumerate() {
            let parent = &chunk[0];
            assert_eq!(parent.level, 1, "chunk {i}");
            let kids = &chunk[1..];
            assert!(kids
                .iter()
                .all(|k| k.level == 2 && !k.has_children && k.lords[0] == parent.lords[0]));
            assert_eq!(kids[0].lords[1], parent.lords[0], "starts from the parent");
            assert_eq!(kids[0].start_jd_ut, parent.start_jd_ut);
            assert_eq!(kids[8].end_jd_ut, parent.end_jd_ut);
            for w in kids.windows(2) {
                assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
            }
        }
        // Mars dönemi 21 gün (1000 → 1021): Mars–Mars 21 × 21 / 360 = 1,225 gün.
        let mars_mars = &d.periods[1];
        assert_eq!(mars_mars.lords, [Graha::Mars, Graha::Mars]);
        assert!(close(mars_mars.end_jd_ut, 1001.225));
        // Mars–Rahu 21 × 54 / 360 = 3,15 gün: 1001,225 → 1004,375.
        assert_eq!(d.periods[2].lords, [Graha::Mars, Graha::Rahu]);
        assert!(close(d.periods[2].end_jd_ut, 1004.375));
    }

    #[test]
    fn nakshatra_and_range_are_validated() {
        for bad in [0_u8, 28] {
            assert_eq!(
                mudda_dasha(bad, 0, 0.0, 360.0),
                Err(CoreError::InvalidDashaInput("nakshatra"))
            );
        }
        for (start, end) in [
            (10.0, 10.0),
            (10.0, 5.0),
            (f64::NAN, 5.0),
            (0.0, f64::INFINITY),
        ] {
            assert_eq!(
                mudda_dasha(1, 0, start, end),
                Err(CoreError::InvalidDashaInput("range"))
            );
        }
    }
}
