//! Navatara: doğum Ay'ının nakshatrasından başlayarak 27 nakshatranın dokuz taraya üç turda
//! dağılımı ve seçilen geçiş anında dokuz grahanın tarası (yol haritası §4.2).
//!
//! Sayım doğum nakshatrasını da sayar (aynı nakshatra 1, Janma) ve dokuzda bir yinelenir; tur
//! sayımın hangi dokuzluda olduğudur (1..=3). Taranın numarası ve elverişli kümesi muhurta
//! modülününkidir (`tara_bala`); bu dosya yeni tablo getirmez. Sayım yalnızca Ay'dandır
//! (`NAVATARA_METHOD`); lagna nakshatrasından sayım uygulanmamıştır.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::muhurta::{tara_bala, TaraBala};
use crate::positions::Positions;
use crate::zodiac::nakshatra_pada;

use super::yogi::nakshatra_lord;

/// Yöntem kimliği (Y5): sayım doğum Ay'ının nakshatrasından.
pub const NAVATARA_METHOD: &str = "navatara.moon_v1";

/// Tablonun bir satırı: bir nakshatra ve doğum nakshatrasına göre tarası.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct NavataraRow {
    /// 1..=27.
    pub nakshatra: u8,
    pub tara: TaraBala,
    /// Sayımın turu, 1..=3.
    pub cycle: u8,
    /// Nakshatranın Vimshottari lordu.
    pub lord: Graha,
    /// Bu nakshatrada duran doğum grahaları, `Graha::ALL` sırasında.
    pub grahas: Vec<Graha>,
}

/// Geçiş anındaki bir grahanın tarası.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TransitTara {
    pub graha: Graha,
    /// Grahanın geçiş anındaki nakshatrası, 1..=27.
    pub nakshatra: u8,
    /// 1..=4.
    pub pada: u8,
    pub tara: TaraBala,
    /// Sayımın turu, 1..=3.
    pub cycle: u8,
}

/// `compute_navatara` sonucu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Navatara {
    /// Doğum Ay'ının nakshatrası, 1..=27.
    pub janma_nakshatra: u8,
    /// 27 satır; ilk satır doğum nakshatrasıdır, sonrakiler zodyak sırasıyla.
    pub rows: Vec<NavataraRow>,
    /// Geçiş anı verildiyse dokuz graha, `Graha::ALL` sırasında.
    pub transit: Option<Vec<TransitTara>>,
    /// Geçiş anı, UT Jülyen günü.
    pub transit_jd_ut: Option<f64>,
    /// Doğum saati bilinmiyor: Ay'ın nakshatrası öğlen konumundandır.
    pub approximate: bool,
    pub method: String,
}

/// `from` nakshatrasından `to` nakshatrasına sayımın turu, 1..=3: sayım 1–9 ise 1, 10–18 ise
/// 2, 19–27 ise 3. İkisi de 1..=27 olmalıdır (aralık dışı değer 27'ye göre sarılır).
pub fn tara_cycle(from: u8, to: u8) -> u8 {
    ((usize::from(to) + 27 - usize::from(from) % 27) % 27 / 9 + 1) as u8
}

/// Doğum tablosu: 27 satır, doğum Ay'ının nakshatrasından başlayarak.
pub fn navatara_rows(positions: &Positions) -> Vec<NavataraRow> {
    let (janma, _) = nakshatra_pada(positions.point(Graha::Moon).sidereal_lon);
    (0..27u8)
        .map(|step| {
            let nakshatra = (janma - 1 + step) % 27 + 1;
            NavataraRow {
                nakshatra,
                tara: tara_bala(janma, nakshatra),
                cycle: step / 9 + 1,
                lord: nakshatra_lord(nakshatra),
                grahas: positions
                    .grahas
                    .iter()
                    .filter(|p| nakshatra_pada(p.sidereal_lon).0 == nakshatra)
                    .map(|p| p.graha)
                    .collect(),
            }
        })
        .collect()
}

/// Geçiş grahalarının tarası. `lons` (graha, sidereal boylam) çiftleridir; sıra korunur.
/// `janma_nakshatra` 1..=27 olmalıdır.
pub fn transit_taras(janma_nakshatra: u8, lons: &[(Graha, f64)]) -> Vec<TransitTara> {
    lons.iter()
        .map(|&(graha, lon)| {
            let (nakshatra, pada) = nakshatra_pada(lon);
            TransitTara {
                graha,
                nakshatra,
                pada,
                tara: tara_bala(janma_nakshatra, nakshatra),
                cycle: tara_cycle(janma_nakshatra, nakshatra),
            }
        })
        .collect()
}

/// Rapor: doğum tablosu ve (verildiyse) geçiş anının taraları. `transit`, anın UT Jülyen günü
/// ve dokuz grahanın sidereal boylamıdır.
pub fn navatara(
    positions: &Positions,
    transit: Option<(f64, &[(Graha, f64)])>,
    approximate: bool,
) -> Navatara {
    let rows = navatara_rows(positions);
    let janma_nakshatra = rows[0].nakshatra;
    Navatara {
        janma_nakshatra,
        rows,
        transit: transit.map(|(_, lons)| transit_taras(janma_nakshatra, lons)),
        transit_jd_ut: transit.map(|(jd, _)| jd),
        approximate,
        method: NAVATARA_METHOD.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::muhurta::TARA_FAVOURABLE;
    use Graha::*;

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    /// Nakshatralar: Güneş 10° Ashwini (1); Ay 100° Pushya (8: 93°20′–106°40′); Mars 220°
    /// Anuradha (17: 213°20′–226°40′); Merkür 65° Mrigashira (5: 53°20′–66°40′); Jüpiter 250°
    /// Mula (19: 240°–253°20′); Venüs 40° Rohini (4: 40°–53°20′); Satürn 300° Dhanishta (23:
    /// 293°20′–306°40′); Rahu 95° Pushya (8); Ketu 275° Uttara Ashadha (21: 266°40′–280°).
    const BASE: [f64; 9] = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 95.0, 275.0];

    fn base() -> Positions {
        Positions::synthetic(Some(0.0), BASE)
    }

    /// Adım adım sayım: `from`'dan `to`'ya nakshatra nakshatra yürür; taranın numarasını ve
    /// turunu formülsüz bulur.
    fn hand_count(from: u8, to: u8) -> (u8, u8) {
        let (mut at, mut tara, mut cycle) = (from, 1, 1);
        while at != to {
            at = if at == 27 { 1 } else { at + 1 };
            if tara == 9 {
                tara = 1;
                cycle += 1;
            } else {
                tara += 1;
            }
        }
        (tara, cycle)
    }

    #[test]
    fn the_table_starts_at_the_birth_nakshatra_and_runs_three_rounds_of_nine() {
        let rows = navatara_rows(&base());
        assert_eq!(rows.len(), 27);
        // Doğum nakshatrası Pushya (8): satırlar 8, 9, … 27, 1, … 7.
        let order: Vec<u8> = rows.iter().map(|r| r.nakshatra).collect();
        let hand: Vec<u8> = (8..=27).chain(1..=7).collect();
        assert_eq!(order, hand);
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(usize::from(row.tara.number), i % 9 + 1, "row {i}");
            assert_eq!(usize::from(row.cycle), i / 9 + 1, "row {i}");
            assert_eq!(
                row.tara.favourable,
                TARA_FAVOURABLE.contains(&row.tara.number),
                "row {i}"
            );
            assert_eq!(row.lord, nakshatra_lord(row.nakshatra), "row {i}");
        }
        // Bir taranın üç nakshatrası aynı lordu taşır: Janma satırları Pushya (8), Anuradha (17)
        // ve Uttara Bhadrapada (26); üçünün lordu Satürn.
        let janma: Vec<(u8, Graha)> = rows
            .iter()
            .filter(|r| r.tara.number == 1)
            .map(|r| (r.nakshatra, r.lord))
            .collect();
        assert_eq!(janma, [(8, Saturn), (17, Saturn), (26, Saturn)]);
    }

    #[test]
    fn natal_grahas_sit_in_the_rows_of_their_nakshatras() {
        let rows = navatara_rows(&base());
        let at = |n: u8| rows.iter().find(|r| r.nakshatra == n).expect("row");
        // Pushya (8): Ay ve Rahu; 1. tara (Janma), 1. tur.
        assert_eq!(at(8).grahas, [Moon, Rahu]);
        assert_eq!((at(8).tara.number, at(8).cycle), (1, 1));
        // Anuradha (17): Mars. Pushya'dan sayım 8 → 17: on nakshatra → 1. tara, 2. tur.
        assert_eq!(at(17).grahas, [Mars]);
        assert_eq!((at(17).tara.number, at(17).cycle), (1, 2));
        // Mula (19): Jüpiter. Sayım 12 → 3. tara (Vipat), 2. tur.
        assert_eq!(at(19).grahas, [Jupiter]);
        assert_eq!((at(19).tara.number, at(19).cycle), (3, 2));
        // Uttara Ashadha (21): Ketu. Sayım 14 → 5. tara (Pratyak), 2. tur.
        assert_eq!(
            (at(21).grahas.as_slice(), at(21).tara.number),
            (&[Ketu][..], 5)
        );
        // Dhanishta (23): Satürn. Sayım 16 → 7. tara (Naidhana), 2. tur.
        assert_eq!(
            (at(23).grahas.as_slice(), at(23).tara.number),
            (&[Saturn][..], 7)
        );
        // Ashwini (1): Güneş. Sayım 8 → 27 yirmi, sonra 1: yirmi bir → 3. tara, 3. tur.
        assert_eq!(at(1).grahas, [Sun]);
        assert_eq!((at(1).tara.number, at(1).cycle), (3, 3));
        // Rohini (4): Venüs. Sayım 24 → 6. tara (Sadhana), 3. tur; elverişli kümede.
        assert_eq!(at(4).grahas, [Venus]);
        assert_eq!(
            (at(4).tara.number, at(4).cycle, at(4).tara.favourable),
            (6, 3, true)
        );
        // Mrigashira (5): Merkür. Sayım 25 → 7. tara, 3. tur.
        assert_eq!(
            (at(5).grahas.as_slice(), at(5).tara.number),
            (&[Mercury][..], 7)
        );
        // Her graha tam bir satırdadır; öteki satırlar boştur.
        assert_eq!(rows.iter().map(|r| r.grahas.len()).sum::<usize>(), 9);
        assert_eq!(rows.iter().filter(|r| r.grahas.is_empty()).count(), 19);
    }

    #[test]
    fn tara_and_cycle_match_the_step_by_step_count_on_every_pair() {
        for from in 1..=27u8 {
            for to in 1..=27u8 {
                let (tara, cycle) = hand_count(from, to);
                assert_eq!(tara_bala(from, to).number, tara, "{from} → {to}");
                assert_eq!(tara_cycle(from, to), cycle, "{from} → {to}");
            }
        }
    }

    #[test]
    fn transit_taras_count_each_graha_from_the_birth_nakshatra() {
        // Doğum nakshatrası Pushya (8). Geçiş: Güneş 100° (Pushya, 3. pada: 100°–103°20′) → 1.
        // tara, 1. tur; Ay 0° (Ashwini, 1. pada): sayım 21 → 3. tara, 3. tur; Jüpiter 359°
        // (Revati 27, 4. pada): sayım 20 → 2. tara (Sampat), 3. tur; Satürn 110°30′ (Ashlesha 9,
        // 2. pada: 110°–113°20′): sayım 2 → 2. tara, 1. tur.
        let lons = [(Sun, 100.0), (Moon, 0.0), (Jupiter, 359.0), (Saturn, 110.5)];
        let got: Vec<(Graha, u8, u8, u8, u8, bool)> = transit_taras(8, &lons)
            .into_iter()
            .map(|t| {
                (
                    t.graha,
                    t.nakshatra,
                    t.pada,
                    t.tara.number,
                    t.cycle,
                    t.tara.favourable,
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                (Sun, 8, 3, 1, 1, false),
                (Moon, 1, 1, 3, 3, false),
                (Jupiter, 27, 4, 2, 3, true),
                (Saturn, 9, 2, 2, 1, true),
            ]
        );
    }

    #[test]
    fn the_report_carries_the_transit_only_when_a_moment_is_given() {
        let natal = navatara(&base(), None, false);
        assert_eq!(natal.janma_nakshatra, 8);
        assert_eq!(natal.rows, navatara_rows(&base()));
        assert_eq!((natal.transit.as_ref(), natal.transit_jd_ut), (None, None));
        assert!(!natal.approximate);
        assert_eq!(natal.method, "navatara.moon_v1");
        let lons: Vec<(Graha, f64)> = Graha::ALL.into_iter().zip(BASE).collect();
        let with = navatara(&base(), Some((2_460_000.5, &lons)), true);
        assert_eq!(with.transit_jd_ut, Some(2_460_000.5));
        assert!(with.approximate);
        // Geçiş konumları doğum konumlarının aynısıysa her graha kendi satırının tarasını alır.
        let transit = with.transit.expect("transit");
        assert_eq!(transit.len(), 9);
        for t in &transit {
            let row = with
                .rows
                .iter()
                .find(|r| r.nakshatra == t.nakshatra)
                .expect("row");
            assert!(row.grahas.contains(&t.graha), "{:?}", t.graha);
            assert_eq!((t.tara, t.cycle), (row.tara, row.cycle), "{:?}", t.graha);
        }
        // Lagna tabloyu etkilemez.
        assert_eq!(navatara_rows(&base().without_lagna()), natal.rows);
    }
}
