//! Argala ve virodhargala (ileri modüller spec'i §5).
//!
//! Kaynak: BPHS, argala bölümü; Jaimini Upadesha Sutraları. Referans burçtan 2., 4. ve 11.
//! evdeki grahalar argala verir; 12., 10. ve 3. evdekiler sırasıyla engeller. İkincil çift:
//! 5. ev argala, 9. ev engel. Durum yalnızca sayıya bakar; güç karşılaştırması yapılmaz ve
//! Ketu için ters sayım uygulanmaz. Referanstan 3. evde üç ya da daha çok doğal kötücül varsa
//! bu ayrıca bildirilir. Evler referans burçtan sayılan rasi evleridir (K4).

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::nature::{nature, Nature};
use crate::positions::Positions;
use crate::zodiac::Sign;

/// Argala veren ev ve engelleyeni: 2 ↔ 12, 4 ↔ 10, 11 ↔ 3 ve ikincil 5 ↔ 9.
pub const ARGALA_PAIRS: [(u8, u8); 4] = [(2, 12), (4, 10), (11, 3), (5, 9)];

/// 3. evdeki kötücül argala için gereken en az doğal kötücül sayısı.
pub const THIRD_HOUSE_MALEFICS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum ArgalaStatus {
    /// Argala evinde graha yok.
    None,
    /// Argala tarafında daha çok graha var.
    Stands,
    /// İki tarafta eşit sayıda graha var.
    Contested,
    /// Engel tarafında daha çok graha var.
    Obstructed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ArgalaPair {
    pub argala_house: u8,
    pub virodha_house: u8,
    /// Argala evindeki grahalar, `Graha::ALL` sırasında (dokuz graha da sayılır).
    pub argala_grahas: Vec<Graha>,
    /// Engel evindeki grahalar, `Graha::ALL` sırasında.
    pub virodha_grahas: Vec<Graha>,
    pub status: ArgalaStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ArgalaRow {
    pub reference: Sign,
    /// `ARGALA_PAIRS` sırasında.
    pub pairs: [ArgalaPair; 4],
    pub third_house_malefic_argala: bool,
}

/// Sayıya göre durum.
pub fn argala_status(argala_count: usize, virodha_count: usize) -> ArgalaStatus {
    if argala_count == 0 {
        ArgalaStatus::None
    } else {
        match argala_count.cmp(&virodha_count) {
            std::cmp::Ordering::Greater => ArgalaStatus::Stands,
            std::cmp::Ordering::Equal => ArgalaStatus::Contested,
            std::cmp::Ordering::Less => ArgalaStatus::Obstructed,
        }
    }
}

/// `reference` burcundan sayılan `house`. evdeki grahalar.
fn in_house(positions: &Positions, reference: Sign, house: u8) -> Vec<Graha> {
    positions.occupants(Sign::from_index(reference.index() + usize::from(house) - 1))
}

/// `reference` burcu için dört çift ve 3. ev kötücül argalası.
pub fn argala_row(reference: Sign, positions: &Positions) -> ArgalaRow {
    let pairs = ARGALA_PAIRS.map(|(argala_house, virodha_house)| {
        let argala_grahas = in_house(positions, reference, argala_house);
        let virodha_grahas = in_house(positions, reference, virodha_house);
        ArgalaPair {
            argala_house,
            virodha_house,
            status: argala_status(argala_grahas.len(), virodha_grahas.len()),
            argala_grahas,
            virodha_grahas,
        }
    });
    let third_house_malefics = in_house(positions, reference, 3)
        .into_iter()
        .filter(|&g| nature(g, positions) == Nature::Malefic)
        .count();
    ArgalaRow {
        reference,
        pairs,
        third_house_malefic_argala: third_house_malefics >= THIRD_HOUSE_MALEFICS,
    }
}

/// On iki referans burç için argala: lagna burcundan başlayarak zodyak sırasıyla; doğum saati
/// bilinmiyorsa Koç'tan. Argala lagnaya bağlı değildir; lagna yalnızca satır sırasını belirler.
pub fn argala_rows(positions: &Positions) -> [ArgalaRow; 12] {
    let first = positions.lagna_sign.unwrap_or(Sign::Aries);
    std::array::from_fn(|i| argala_row(Sign::from_index(first.index() + i), positions))
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;
    use Sign::*;

    #[test]
    fn status_looks_at_the_counts_only() {
        assert_eq!(argala_status(0, 0), ArgalaStatus::None);
        assert_eq!(argala_status(0, 3), ArgalaStatus::None);
        assert_eq!(argala_status(1, 0), ArgalaStatus::Stands);
        assert_eq!(argala_status(2, 1), ArgalaStatus::Stands);
        assert_eq!(argala_status(1, 1), ArgalaStatus::Contested);
        assert_eq!(argala_status(1, 2), ArgalaStatus::Obstructed);
    }

    #[test]
    fn four_pairs_for_a_reference_sign() {
        // Güneş Boğa, Ay Yengeç, Mars İkizler, Merkür Boğa, Jüpiter Balık, Venüs Oğlak,
        // Satürn İkizler, Rahu İkizler, Ketu Yay.
        let p = Positions::synthetic(
            None,
            [40.0, 100.0, 65.0, 50.0, 340.0, 280.0, 75.0, 80.0, 260.0],
        );
        let row = argala_row(Aries, &p);
        assert_eq!(row.reference, Aries);
        let houses: Vec<(u8, u8)> = row
            .pairs
            .iter()
            .map(|x| (x.argala_house, x.virodha_house))
            .collect();
        assert_eq!(houses, ARGALA_PAIRS);
        // 2 (Boğa: Güneş, Merkür) ↔ 12 (Balık: Jüpiter): 2 > 1.
        assert_eq!(row.pairs[0].argala_grahas, [Sun, Mercury]);
        assert_eq!(row.pairs[0].virodha_grahas, [Jupiter]);
        assert_eq!(row.pairs[0].status, ArgalaStatus::Stands);
        // 4 (Yengeç: Ay) ↔ 10 (Oğlak: Venüs): eşit.
        assert_eq!(row.pairs[1].argala_grahas, [Moon]);
        assert_eq!(row.pairs[1].virodha_grahas, [Venus]);
        assert_eq!(row.pairs[1].status, ArgalaStatus::Contested);
        // 11 (Kova: boş) ↔ 3 (İkizler: Mars, Satürn, Rahu): argala tarafı boş.
        assert!(row.pairs[2].argala_grahas.is_empty());
        assert_eq!(row.pairs[2].virodha_grahas, [Mars, Saturn, Rahu]);
        assert_eq!(row.pairs[2].status, ArgalaStatus::None);
        // 5 (Aslan: boş) ↔ 9 (Yay: Ketu).
        assert_eq!(row.pairs[3].status, ArgalaStatus::None);
        assert_eq!(row.pairs[3].virodha_grahas, [Ketu]);
        // 3. evde üç doğal kötücül (Mars, Satürn, Rahu).
        assert!(row.third_house_malefic_argala);
        // Boğa'dan 3. ev Yengeç: yalnızca Ay.
        assert!(!argala_row(Taurus, &p).third_house_malefic_argala);
        // Koç'tan 12. evdeki graha sayısı artınca 2 ↔ 12 engellenir.
        let q = Positions::synthetic(
            None,
            [40.0, 345.0, 65.0, 350.0, 340.0, 280.0, 75.0, 80.0, 260.0],
        );
        let row = argala_row(Aries, &q);
        assert_eq!(row.pairs[0].argala_grahas, [Sun]);
        assert_eq!(row.pairs[0].virodha_grahas, [Moon, Mercury, Jupiter]);
        assert_eq!(row.pairs[0].status, ArgalaStatus::Obstructed);
    }

    #[test]
    fn third_house_needs_three_natural_malefics() {
        // İkizler'de Mars, Satürn ve Jüpiter: iki kötücül, bir iyicil.
        let p = Positions::synthetic(
            None,
            [40.0, 100.0, 65.0, 50.0, 70.0, 280.0, 75.0, 200.0, 20.0],
        );
        assert!(!argala_row(Aries, &p).third_house_malefic_argala);
        // Küçülen Ay kötücüldür (K6): Güneş Aslan 10°, Ay İkizler 5° (uzanım 295°).
        let q = Positions::synthetic(
            None,
            [130.0, 65.0, 70.0, 250.0, 340.0, 280.0, 75.0, 200.0, 20.0],
        );
        assert!(argala_row(Aries, &q).third_house_malefic_argala);
        // Aynı yerleşimde büyüyen Ay iyicildir: Güneş Boğa 10°, Ay İkizler 5° (uzanım 25°).
        let r = Positions::synthetic(
            None,
            [40.0, 65.0, 70.0, 250.0, 340.0, 280.0, 75.0, 200.0, 20.0],
        );
        assert!(!argala_row(Aries, &r).third_house_malefic_argala);
    }

    #[test]
    fn rows_start_at_the_lagna_sign_or_at_aries() {
        let lons = [40.0, 100.0, 65.0, 50.0, 340.0, 280.0, 75.0, 80.0, 260.0];
        let with_lagna = argala_rows(&Positions::synthetic(Some(215.0), lons));
        let refs: Vec<Sign> = with_lagna.iter().map(|r| r.reference).collect();
        assert_eq!(refs[0], Scorpio);
        assert_eq!(refs[11], Libra);
        let without = argala_rows(&Positions::synthetic(None, lons));
        let refs: Vec<Sign> = without.iter().map(|r| r.reference).collect();
        assert_eq!(refs, Sign::ALL);
        // Satırların içeriği aynıdır; yalnızca sıra değişir.
        for row in &with_lagna {
            assert_eq!(row, &without[row.reference.index()]);
        }
        // Aynı graha bir çiftin iki tarafında görünmez.
        for row in &without {
            for pair in &row.pairs {
                assert!(pair
                    .argala_grahas
                    .iter()
                    .all(|g| !pair.virodha_grahas.contains(g)));
            }
        }
    }
}
