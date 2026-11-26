//! Sekiz kutanın puanı (ileri modüller spec'i §9 tablosu). Her fonksiyon iki tarafın
//! sınıfından puan üretir; ilk bağımsız değişken damat rolünün, ikincisi gelin rolünün
//! değeridir. Varna, vashya, yoni, gana ve nadi sınıfları `kuta_tables`'tadır.

use serde::{Deserialize, Serialize};

use crate::dignity::{natural_relation, Relation};
use crate::graha::Graha;
use crate::positions::Positions;
use crate::zodiac::{deg_in_sign, nakshatra_pada, Sign};

use super::kuta_tables::{
    gana_points, vashya_points, yoni_points, Gana, Nadi, Varna, Vashya, Yoni,
};

/// Sekiz kuta, puan tablosunun sırasıyla (en çok puan 1'den 8'e artar).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Kuta {
    Varna,
    Vashya,
    Tara,
    Yoni,
    GrahaMaitri,
    Gana,
    Bhakoot,
    Nadi,
}

impl Kuta {
    pub const ALL: [Kuta; 8] = [
        Kuta::Varna,
        Kuta::Vashya,
        Kuta::Tara,
        Kuta::Yoni,
        Kuta::GrahaMaitri,
        Kuta::Gana,
        Kuta::Bhakoot,
        Kuta::Nadi,
    ];

    /// Kutanın en çok puanı: sırasıyla 1, 2, … 8 (toplam 36).
    pub fn max(self) -> f64 {
        f64::from(self as u8 + 1)
    }
}

/// Ashtakuta'nın en çok toplam puanı.
pub const KUTA_TOTAL_MAX: f64 = 36.0;

/// Ashtakuta'nın bir haritadan okuduğu her şey: Ay'ın rasi çerçevesindeki burcu ve burç içi
/// derecesi (K5), sidereal nakshatrası ve padası.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoonPoint {
    pub sign: Sign,
    /// Rasi çerçevesinde burç içi derece, [0, 30); vashya yarılarını belirler.
    pub deg_in_sign: f64,
    /// 1 = Ashwini … 27 = Revati.
    pub nakshatra: u8,
    /// 1..=4.
    pub pada: u8,
}

impl MoonPoint {
    /// Haritanın Ay'ı: burç ve derece rasi çerçevesinden, nakshatra sidereal boylamdan.
    pub fn of(positions: &Positions) -> MoonPoint {
        let moon = positions.point(Graha::Moon);
        let (nakshatra, pada) = nakshatra_pada(moon.sidereal_lon);
        MoonPoint {
            sign: moon.rasi_sign,
            deg_in_sign: moon.deg_in_sign,
            nakshatra,
            pada,
        }
    }

    /// Sidereal boylamdaki Ay (rasi çerçevesi sidereal sayılır); el örnekleri ve testler.
    pub fn at(sidereal_lon: f64) -> MoonPoint {
        let (nakshatra, pada) = nakshatra_pada(sidereal_lon);
        MoonPoint {
            sign: Sign::of(sidereal_lon),
            deg_in_sign: deg_in_sign(sidereal_lon),
            nakshatra,
            pada,
        }
    }
}

/// Varna (1): damat rolünün sınıfı eşit ya da üstünse 1, değilse 0.
pub fn varna_score(groom: Varna, bride: Varna) -> f64 {
    if groom.rank() >= bride.rank() {
        1.0
    } else {
        0.0
    }
}

/// Vashya (2): 5 × 5 tablodan.
pub fn vashya_score(groom: Vashya, bride: Vashya) -> f64 {
    vashya_points(groom, bride)
}

/// `from` nakshatrasından `to` nakshatrasına (ikisi de 1..=27) sayımın dokuzlu döngüdeki yeri,
/// 1..=9: aynı nakshatra 1 (Janma), bir sonraki 2 (Sampat) … Sayım `from`'u da sayar.
pub fn tara_number(from: u8, to: u8) -> u8 {
    assert!(
        (1..=27).contains(&from) && (1..=27).contains(&to),
        "nakshatras {from} and {to} must be inside 1..=27"
    );
    (to + 27 - from) % 9 + 1
}

/// Uygunsuz taralar: 3. (Vipat), 5. (Pratyak) ve 7. (Naidhana).
pub const TARA_UNFAVOURABLE: [u8; 3] = [3, 5, 7];

/// Tara (3): iki yön ayrı sayılır; uygun olan her yön 1½ verir. Rol değişiminde değişmez.
pub fn tara_score(groom_nakshatra: u8, bride_nakshatra: u8) -> f64 {
    let favourable = |from, to| !TARA_UNFAVOURABLE.contains(&tara_number(from, to));
    let directions = [
        favourable(bride_nakshatra, groom_nakshatra),
        favourable(groom_nakshatra, bride_nakshatra),
    ];
    1.5 * directions.into_iter().filter(|f| *f).count() as f64
}

/// Yoni (4): 14 × 14 tablodan.
pub fn yoni_score(groom: Yoni, bride: Yoni) -> f64 {
    f64::from(yoni_points(groom, bride))
}

/// Graha maitri (5): iki Ay burcunun lordları arasındaki doğal ilişki. Aynı lord ya da iki
/// yönlü dost 5; dost + nötr 4; iki yönlü nötr 3; dost + düşman 1; nötr + düşman ½; iki yönlü
/// düşman 0. Rol değişiminde değişmez.
pub fn maitri_score(groom_lord: Graha, bride_lord: Graha) -> f64 {
    if groom_lord == bride_lord {
        return 5.0;
    }
    use Relation::{Enemy, Friend, Neutral};
    match (
        natural_relation(groom_lord, bride_lord),
        natural_relation(bride_lord, groom_lord),
    ) {
        (Friend, Friend) => 5.0,
        (Friend, Neutral) | (Neutral, Friend) => 4.0,
        (Neutral, Neutral) => 3.0,
        (Friend, Enemy) | (Enemy, Friend) => 1.0,
        (Neutral, Enemy) | (Enemy, Neutral) => 0.5,
        (Enemy, Enemy) => 0.0,
    }
}

/// İki lord aynı ya da iki yönlü dost mu (Bhakoot istisnasının koşulu).
pub fn lords_same_or_mutual_friends(a: Graha, b: Graha) -> bool {
    a == b
        || (natural_relation(a, b) == Relation::Friend
            && natural_relation(b, a) == Relation::Friend)
}

/// Gana (6): 3 × 3 tablodan.
pub fn gana_score(groom: Gana, bride: Gana) -> f64 {
    f64::from(gana_points(groom, bride))
}

/// Bhakoot'u sıfırlayan karşılıklı sayımlar: 2/12, 5/9 ve 6/8.
pub const BHAKOOT_ZERO_COUNTS: [u8; 6] = [2, 12, 5, 9, 6, 8];

/// Bhakoot (7): iki Ay burcunun karşılıklı sayımı 2/12, 5/9 ya da 6/8 ise 0, değilse 7.
/// Rol değişiminde değişmez (sayımlardan biri n ise diğeri 14 − n'dir).
pub fn bhakoot_score(groom: Sign, bride: Sign) -> f64 {
    if BHAKOOT_ZERO_COUNTS.contains(&bride.house_from(groom)) {
        0.0
    } else {
        7.0
    }
}

/// Nadi (8): farklıysa 8, aynıysa 0.
pub fn nadi_score(groom: Nadi, bride: Nadi) -> f64 {
    if groom == bride {
        0.0
    } else {
        8.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dignity::sign_lord;
    use crate::kuta::kuta_tables::{nadi_of, varna_of};
    use Graha::{Jupiter, Mars, Mercury, Moon, Saturn, Sun, Venus};

    #[test]
    fn kuta_maxima_add_up_to_thirty_six() {
        assert_eq!(
            Kuta::ALL.map(Kuta::max),
            [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]
        );
        assert_eq!(Kuta::ALL.map(Kuta::max).iter().sum::<f64>(), KUTA_TOTAL_MAX);
    }

    #[test]
    fn moon_point_reads_sign_degree_nakshatra_and_pada() {
        // 100°: Yengeç 10°; nakshatra dizini 100 / 13°20′ = 7,5 → 8. nakshatra (Pushya);
        // Pushya 93°20′'de başlar, 100° − 93°20′ = 6°40′ → 3. pada (6°40′–10°).
        let m = MoonPoint::at(100.0);
        assert_eq!(
            (m.sign, m.deg_in_sign, m.nakshatra, m.pada),
            (Sign::Cancer, 10.0, 8, 3)
        );
        let p = Positions::synthetic(None, [0.0, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(MoonPoint::of(&p), m);
        // Sınırlar: 0° Ashwini 1. pada; 359,999° Revati 4. pada.
        assert_eq!(
            (MoonPoint::at(0.0).nakshatra, MoonPoint::at(0.0).pada),
            (1, 1)
        );
        let last = MoonPoint::at(359.999);
        assert_eq!(
            (last.sign, last.nakshatra, last.pada),
            (Sign::Pisces, 27, 4)
        );
    }

    /// Varna: 16 sınıf çiftinin tamamı, elle. Satır damat rolü, sütun gelin rolü; sıra
    /// Brahmin, Kshatriya, Vaishya, Shudra. Damat rolünün sınıfı eşit ya da üstünse 1.
    const HAND_VARNA: [[u8; 4]; 4] = [
        [1, 1, 1, 1], // damat Brahmin
        [0, 1, 1, 1], // damat Kshatriya
        [0, 0, 1, 1], // damat Vaishya
        [0, 0, 0, 1], // damat Shudra
    ];

    #[test]
    fn varna_all_sixteen_class_pairs() {
        for (i, g) in Varna::ALL.into_iter().enumerate() {
            for (j, b) in Varna::ALL.into_iter().enumerate() {
                assert_eq!(
                    varna_score(g, b),
                    f64::from(HAND_VARNA[i][j]),
                    "{g:?} {b:?}"
                );
            }
        }
        // Burçlardan: Koç (Kshatriya) damat, Yengeç (Brahmin) gelin → 0; tersi → 1.
        assert_eq!(
            varna_score(varna_of(Sign::Aries), varna_of(Sign::Cancer)),
            0.0
        );
        assert_eq!(
            varna_score(varna_of(Sign::Cancer), varna_of(Sign::Aries)),
            1.0
        );
    }

    /// Tara'nın bağımsız ikinci yazımı: sayım (1..=27) → uygunsuz mu, elle yazılmış liste.
    /// Dokuzlu döngünün 3., 5. ve 7. yerleri: 3, 5, 7; 12, 14, 16; 21, 23, 25.
    const HAND_UNFAVOURABLE_COUNTS: [u8; 9] = [3, 5, 7, 12, 14, 16, 21, 23, 25];

    /// `from`'dan `to`'ya sayım, adım adım yürüyerek (mod aritmetiği kullanmadan).
    fn hand_count(from: u8, to: u8) -> u8 {
        let mut at = from;
        let mut count = 1;
        while at != to {
            at = if at == 27 { 1 } else { at + 1 };
            count += 1;
        }
        count
    }

    #[test]
    fn tara_all_729_nakshatra_pairs_against_the_hand_list() {
        let mut totals = [0u32; 3];
        for groom in 1..=27u8 {
            for bride in 1..=27u8 {
                let to_groom = !HAND_UNFAVOURABLE_COUNTS.contains(&hand_count(bride, groom));
                let to_bride = !HAND_UNFAVOURABLE_COUNTS.contains(&hand_count(groom, bride));
                let want = match (to_groom, to_bride) {
                    (true, true) => 3.0,
                    (false, false) => 0.0,
                    _ => 1.5,
                };
                assert_eq!(
                    tara_score(groom, bride),
                    want,
                    "groom {groom} bride {bride}"
                );
                // Rol değişiminde değişmez.
                assert_eq!(
                    tara_score(bride, groom),
                    want,
                    "bride {bride} groom {groom}"
                );
                totals[(want / 1.5) as usize] += 1;
            }
        }
        // Sayım c ise ters yönün sayımı 29 − c'dir (c = 1'de 1). Dokuzlu döngüde tara t'nin
        // tersi t = 1 için 1, diğerlerinde 11 − t'dir: 3 ↔ 8, 5 ↔ 6, 7 ↔ 4. Uygunsuz bir yönün
        // tersi hiçbir zaman uygunsuz değildir: 0 puan hiç oluşmaz. Her nakshatra için 27
        // sayımın 9'u uygunsuzdur; 27 × 9 × 2 = 486 çift 1½, 729 − 486 = 243 çift 3 alır.
        assert_eq!(totals, [0, 486, 243]);
    }

    #[test]
    fn tara_number_counts_inclusively_in_a_cycle_of_nine() {
        assert_eq!(tara_number(1, 1), 1);
        assert_eq!(tara_number(1, 2), 2);
        assert_eq!(tara_number(1, 9), 9);
        assert_eq!(tara_number(1, 10), 1);
        assert_eq!(tara_number(27, 1), 2);
        // Rohini'den (4) Ashwini'ye (1): sayım 25 → 25 − 18 = 7. tara.
        assert_eq!(tara_number(4, 1), 7);
        assert_eq!(hand_count(4, 1), 25);
    }

    /// Graha maitri: 49 lord çiftinin tamamı, elle. Sıra Güneş, Ay, Mars, Merkür, Jüpiter,
    /// Venüs, Satürn; puanın iki katı yazılıdır (½ puan 1 olarak).
    const HAND_MAITRI_X2: [[u8; 7]; 7] = [
        // Gü  Ay  Ma  Me  Jü  Ve  Sa
        [10, 10, 10, 8, 10, 0, 0], // Güneş
        [10, 10, 8, 2, 8, 1, 1],   // Ay
        [10, 8, 10, 1, 10, 6, 1],  // Mars
        [8, 2, 1, 10, 1, 10, 8],   // Merkür
        [10, 8, 10, 1, 10, 1, 6],  // Jüpiter
        [0, 1, 6, 10, 1, 10, 10],  // Venüs
        [0, 1, 1, 8, 6, 10, 10],   // Satürn
    ];
    const LORDS: [Graha; 7] = [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn];

    #[test]
    fn maitri_all_forty_nine_lord_pairs() {
        for (i, a) in LORDS.into_iter().enumerate() {
            for (j, b) in LORDS.into_iter().enumerate() {
                let want = f64::from(HAND_MAITRI_X2[i][j]) / 2.0;
                assert_eq!(maitri_score(a, b), want, "{a:?} {b:?}");
                assert_eq!(HAND_MAITRI_X2[i][j], HAND_MAITRI_X2[j][i], "{a:?} {b:?}");
            }
        }
        // İki yönlü dost çiftleri ve aynı lord: Bhakoot istisnasının koşulu, 5 puanla aynıdır.
        for a in LORDS {
            for b in LORDS {
                assert_eq!(
                    lords_same_or_mutual_friends(a, b),
                    maitri_score(a, b) == 5.0,
                    "{a:?} {b:?}"
                );
            }
        }
    }

    /// Bhakoot: 144 burç çiftinin tamamı, elle. Satır damat rolünün, sütun gelin rolünün Ay
    /// burcu (Koç … Balık); `7` yedi puan, `0` sıfır puan. Her satır bir öncekinin bir sağa
    /// kaymış hâlidir: sayım 1, 3, 4, 7, 10, 11 ise 7.
    const HAND_BHAKOOT: [&str; 12] = [
        "707700700770",
        "070770070077",
        "707077007007",
        "770707700700",
        "077070770070",
        "007707077007",
        "700770707700",
        "070077070770",
        "007007707077",
        "700700770707",
        "770070077070",
        "077007007707",
    ];

    #[test]
    fn bhakoot_all_144_sign_pairs() {
        let mut sevens = 0;
        for (i, row) in HAND_BHAKOOT.iter().enumerate() {
            assert_eq!(row.len(), 12);
            for (j, cell) in row.chars().enumerate() {
                let want = if cell == '7' { 7.0 } else { 0.0 };
                let (g, b) = (Sign::from_index(i), Sign::from_index(j));
                assert_eq!(bhakoot_score(g, b), want, "{g:?} {b:?}");
                assert_eq!(bhakoot_score(b, g), want, "{b:?} {g:?}");
                sevens += u32::from(cell == '7');
            }
        }
        // Her satırda altı sayım (1, 3, 4, 7, 10, 11) 7 verir.
        assert_eq!(sevens, 72);
        // Aynı lordlu sıfır çiftleri: Koç–Akrep (6/8, Mars), Boğa–Terazi (6/8, Venüs),
        // Oğlak–Kova (2/12, Satürn).
        for (a, b) in [
            (Sign::Aries, Sign::Scorpio),
            (Sign::Taurus, Sign::Libra),
            (Sign::Capricorn, Sign::Aquarius),
        ] {
            assert_eq!(bhakoot_score(a, b), 0.0);
            assert_eq!(sign_lord(a), sign_lord(b));
        }
    }

    #[test]
    fn nadi_all_nine_class_pairs() {
        for g in Nadi::ALL {
            for b in Nadi::ALL {
                assert_eq!(
                    nadi_score(g, b),
                    if g == b { 0.0 } else { 8.0 },
                    "{g:?} {b:?}"
                );
            }
        }
        // Ashwini (Adi) ile Ardra (Adi): 0; Ashwini ile Bharani (Madhya): 8.
        assert_eq!(nadi_score(nadi_of(1), nadi_of(6)), 0.0);
        assert_eq!(nadi_score(nadi_of(1), nadi_of(2)), 8.0);
    }

    #[test]
    fn table_scores_read_groom_first() {
        // Vashya: damat yabanıl, gelin dört ayaklı ½; tersi 0.
        assert_eq!(vashya_score(Vashya::Wild, Vashya::Quadruped), 0.5);
        assert_eq!(vashya_score(Vashya::Quadruped, Vashya::Wild), 0.0);
        // Gana: damat Manushya, gelin Deva 5; tersi 6.
        assert_eq!(gana_score(Gana::Manushya, Gana::Deva), 5.0);
        assert_eq!(gana_score(Gana::Deva, Gana::Manushya), 6.0);
        // Yoni: at–manda düşman; at–at 4.
        assert_eq!(yoni_score(Yoni::Horse, Yoni::Buffalo), 0.0);
        assert_eq!(yoni_score(Yoni::Horse, Yoni::Horse), 4.0);
    }
}
