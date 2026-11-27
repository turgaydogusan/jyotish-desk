//! Vimshopaka bala: yedi grahanın varga kümelerindeki konum gücü, yirmi üzerinden (yol
//! haritası §4.3). Kaynak: BPHS, vargaların değerlendirilmesi bölümü; özgün olarak uygulanır.
//!
//! Dört küme vardır (shadvarga, saptavarga, dashavarga, shodashavarga); her küme vargalarına
//! toplamı 20 olan ağırlıklar verir. Graha her vargada, o vargadaki burcuna göre altı
//! basamaktan birini alır: kendi burcu 20, çok dost 18, dost 15, nötr 10, düşman 7, çok düşman
//! 5. Kümenin değeri `Σ ağırlık × basamak / 20`'dir (5–20).
//!
//! Okuma (tek okuma; `VIMSHOPAKA_METHOD`): yücelme ve moolatrikona ayrı basamak değildir; kendi
//! burcunda olmayan graha, varga burcunun lorduyla bileşik ilişkisinden okunur. Geçici dostluk
//! her vargada D1 burçlarından alınır (Shadbala'nın saptavargaja kalemindeki okuma). Vargalar
//! rasi çerçevesindeki boylamdan hesaplanır (K5). Düğümlere değer verilmez. Ağırlık tabloları
//! ve basamaklar bellekten yazılmıştır ve sahibinin onayını bekler (Y4):
//! `tests/golden/modules/natal_extras/REVIEW.md`.

use serde::{Deserialize, Serialize};

use crate::dignity::{
    compound_relation, natural_relation, own_signs, sign_lord, temporal_friend, Dignity,
};
use crate::graha::Graha;
use crate::positions::Positions;
use crate::shadbala::{rasi_lon, SHADBALA_GRAHAS};
use crate::varga::{varga_position, Varga};
use crate::zodiac::Sign;

/// Yöntem kimliği (Y5): BPHS ağırlıkları, altı basamak, geçici dostluk D1'den.
pub const VIMSHOPAKA_METHOD: &str = "vimshopaka.bphs_v1";
/// Bir kümenin en büyük değeri ve ağırlıklarının toplamı.
pub const VIMSHOPAKA_MAX: f64 = 20.0;
/// Değer verilen grahalar: Güneş … Satürn.
pub const VIMSHOPAKA_GRAHAS: [Graha; 7] = SHADBALA_GRAHAS;

/// Dört varga kümesi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum VargaScheme {
    /// Altı varga.
    Shadvarga,
    /// Yedi varga.
    Saptavarga,
    /// On varga.
    Dashavarga,
    /// On altı varga.
    Shodashavarga,
}

const SHADVARGA: [(Varga, f64); 6] = [
    (Varga::D1, 6.0),
    (Varga::D2, 2.0),
    (Varga::D3, 4.0),
    (Varga::D9, 5.0),
    (Varga::D12, 2.0),
    (Varga::D30, 1.0),
];

const SAPTAVARGA: [(Varga, f64); 7] = [
    (Varga::D1, 5.0),
    (Varga::D2, 2.0),
    (Varga::D3, 3.0),
    (Varga::D7, 2.5),
    (Varga::D9, 4.5),
    (Varga::D12, 2.0),
    (Varga::D30, 1.0),
];

const DASHAVARGA: [(Varga, f64); 10] = [
    (Varga::D1, 3.0),
    (Varga::D2, 1.5),
    (Varga::D3, 1.5),
    (Varga::D7, 1.5),
    (Varga::D9, 1.5),
    (Varga::D10, 1.5),
    (Varga::D12, 1.5),
    (Varga::D16, 1.5),
    (Varga::D30, 1.5),
    (Varga::D60, 5.0),
];

const SHODASHAVARGA: [(Varga, f64); 16] = [
    (Varga::D1, 3.5),
    (Varga::D2, 1.0),
    (Varga::D3, 1.0),
    (Varga::D4, 0.5),
    (Varga::D7, 0.5),
    (Varga::D9, 3.0),
    (Varga::D10, 0.5),
    (Varga::D12, 0.5),
    (Varga::D16, 2.0),
    (Varga::D20, 0.5),
    (Varga::D24, 0.5),
    (Varga::D27, 0.5),
    (Varga::D30, 1.0),
    (Varga::D40, 0.5),
    (Varga::D45, 0.5),
    (Varga::D60, 4.0),
];

impl VargaScheme {
    pub const ALL: [VargaScheme; 4] = [
        VargaScheme::Shadvarga,
        VargaScheme::Saptavarga,
        VargaScheme::Dashavarga,
        VargaScheme::Shodashavarga,
    ];

    /// Kümenin vargaları ve ağırlıkları, `Varga::ALL` sırasında; ağırlıkların toplamı 20'dir.
    pub fn weights(self) -> &'static [(Varga, f64)] {
        match self {
            Self::Shadvarga => &SHADVARGA,
            Self::Saptavarga => &SAPTAVARGA,
            Self::Dashavarga => &DASHAVARGA,
            Self::Shodashavarga => &SHODASHAVARGA,
        }
    }
}

/// Basamağın değeri. `varga_relation` yalnızca ilk altı değeri üretir; ötekiler 0 verir.
pub fn rung_points(relation: Dignity) -> f64 {
    match relation {
        Dignity::OwnSign => 20.0,
        Dignity::GreatFriend => 18.0,
        Dignity::Friend => 15.0,
        Dignity::Neutral => 10.0,
        Dignity::Enemy => 7.0,
        Dignity::GreatEnemy => 5.0,
        Dignity::Exalted | Dignity::Moolatrikona | Dignity::Debilitated => 0.0,
    }
}

/// Grahanın `varga`'daki burcu ve basamağı: kendi burcundaysa `OwnSign`, değilse varga burcunun
/// lorduyla bileşik ilişki (doğal ilişki + D1 burçlarından geçici dostluk). Düğümler `None`.
pub fn varga_relation(
    graha: Graha,
    varga: Varga,
    positions: &Positions,
) -> Option<(Sign, Dignity)> {
    if graha.is_node() {
        return None;
    }
    let point = positions.point(graha);
    let (sign, _) = varga_position(varga, rasi_lon(point));
    if own_signs(graha).contains(&sign) {
        return Some((sign, Dignity::OwnSign));
    }
    let lord = sign_lord(sign);
    let relation = compound_relation(
        natural_relation(graha, lord),
        temporal_friend(point.rasi_sign, positions.sign_of(lord)),
    );
    Some((sign, relation))
}

/// Grahanın bir kümedeki değeri, 5–20. Düğümler 0 verir.
pub fn vimshopaka_score(graha: Graha, scheme: VargaScheme, positions: &Positions) -> f64 {
    scheme
        .weights()
        .iter()
        .map(|&(varga, weight)| {
            varga_relation(graha, varga, positions)
                .map_or(0.0, |(_, relation)| weight * rung_points(relation))
        })
        .sum::<f64>()
        / VIMSHOPAKA_MAX
}

/// Grahanın bir vargadaki yeri ve basamağı.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct VimshopakaVarga {
    pub varga: Varga,
    /// Varga burcu.
    pub sign: Sign,
    /// Basamak: `OwnSign`, `GreatFriend`, `Friend`, `Neutral`, `Enemy` ya da `GreatEnemy`.
    pub relation: Dignity,
    /// Basamağın değeri: 20, 18, 15, 10, 7 ya da 5.
    pub points: f64,
}

/// Bir grahanın on altı vargası ve dört kümedeki değeri.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GrahaVimshopaka {
    pub graha: Graha,
    /// On altı varga, `Varga::ALL` sırasında.
    pub vargas: Vec<VimshopakaVarga>,
    pub shadvarga: f64,
    pub saptavarga: f64,
    pub dashavarga: f64,
    pub shodashavarga: f64,
}

/// `compute_vimshopaka` sonucu. AI yorum paneli için sözleşmedir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct VimshopakaReport {
    /// `VIMSHOPAKA_GRAHAS` sırasında yedi graha.
    pub grahas: Vec<GrahaVimshopaka>,
    /// Doğum saati bilinmiyor: yüksek bölümlü vargalar (D16 ve üstü) birkaç dakikada burç
    /// değiştirir; değerler öğlen konumlarındandır.
    pub approximate: bool,
    pub method: String,
}

/// Konumlardan rapor. Lagna gerekmez.
pub fn vimshopaka(positions: &Positions, approximate: bool) -> VimshopakaReport {
    let grahas = VIMSHOPAKA_GRAHAS
        .into_iter()
        .map(|graha| GrahaVimshopaka {
            graha,
            vargas: Varga::ALL
                .into_iter()
                .filter_map(|varga| {
                    varga_relation(graha, varga, positions).map(|(sign, relation)| {
                        VimshopakaVarga {
                            varga,
                            sign,
                            relation,
                            points: rung_points(relation),
                        }
                    })
                })
                .collect(),
            shadvarga: vimshopaka_score(graha, VargaScheme::Shadvarga, positions),
            saptavarga: vimshopaka_score(graha, VargaScheme::Saptavarga, positions),
            dashavarga: vimshopaka_score(graha, VargaScheme::Dashavarga, positions),
            shodashavarga: vimshopaka_score(graha, VargaScheme::Shodashavarga, positions),
        })
        .collect();
    VimshopakaReport {
        grahas,
        approximate,
        method: VIMSHOPAKA_METHOD.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zodiac::{deg_in_sign, normalize};
    use Graha::*;
    use Varga::*;

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    /// `sthana.rs` testlerinin tabanıdır.
    const BASE: [f64; 9] = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];

    fn with(changes: &[(Graha, f64)]) -> Positions {
        let mut lons = BASE;
        for &(g, lon) in changes {
            lons[g as usize] = lon;
        }
        Positions::synthetic(None, lons)
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn points_of(row: &GrahaVimshopaka) -> Vec<f64> {
        row.vargas.iter().map(|v| v.points).collect()
    }

    #[test]
    fn each_scheme_weighs_its_vargas_to_twenty() {
        // Kümelerin vargaları, elle (BPHS sırası).
        let hand: [(VargaScheme, &[Varga]); 4] = [
            (VargaScheme::Shadvarga, &[D1, D2, D3, D9, D12, D30]),
            (VargaScheme::Saptavarga, &[D1, D2, D3, D7, D9, D12, D30]),
            (
                VargaScheme::Dashavarga,
                &[D1, D2, D3, D7, D9, D10, D12, D16, D30, D60],
            ),
            (VargaScheme::Shodashavarga, &Varga::ALL),
        ];
        for (scheme, vargas) in hand {
            let weights = scheme.weights();
            let listed: Vec<Varga> = weights.iter().map(|w| w.0).collect();
            assert_eq!(listed, vargas, "{scheme:?}");
            // Değişmez: ağırlıkların toplamı 20'dir ve hiçbiri sıfır ya da eksi değildir.
            assert_eq!(weights.iter().map(|w| w.1).sum::<f64>(), 20.0, "{scheme:?}");
            assert!(weights.iter().all(|w| w.1 > 0.0), "{scheme:?}");
            // Ağırlıklar yarımın katıdır: çarpımlar ve toplamlar ikili kayan noktada tamdır.
            assert!(
                weights.iter().all(|w| (w.1 * 2.0).fract() == 0.0),
                "{scheme:?}"
            );
        }
        assert_eq!(VargaScheme::ALL.len(), 4);
        // Tek tek hücreler, elle: D1 6 / 5 / 3 / 3½; D9 5 / 4½ / 1½ / 3; D60 – / – / 5 / 4.
        let weight = |scheme: VargaScheme, varga: Varga| {
            scheme.weights().iter().find(|w| w.0 == varga).map(|w| w.1)
        };
        let d1: Vec<_> = VargaScheme::ALL
            .into_iter()
            .map(|s| weight(s, D1))
            .collect();
        assert_eq!(d1, [Some(6.0), Some(5.0), Some(3.0), Some(3.5)]);
        let d9: Vec<_> = VargaScheme::ALL
            .into_iter()
            .map(|s| weight(s, D9))
            .collect();
        assert_eq!(d9, [Some(5.0), Some(4.5), Some(1.5), Some(3.0)]);
        let d60: Vec<_> = VargaScheme::ALL
            .into_iter()
            .map(|s| weight(s, D60))
            .collect();
        assert_eq!(d60, [None, None, Some(5.0), Some(4.0)]);
        assert_eq!(weight(VargaScheme::Saptavarga, D7), Some(2.5));
        assert_eq!(weight(VargaScheme::Shodashavarga, D16), Some(2.0));
    }

    #[test]
    fn the_six_rungs_in_d1_and_no_separate_rung_for_exaltation_or_moolatrikona() {
        let rung = |g: Graha, p: &Positions| varga_relation(g, D1, p).expect("not a node");
        // Kendi burcu: Mars Akrep'te (220°).
        assert_eq!(rung(Mars, &with(&[])), (Sign::Scorpio, Dignity::OwnSign));
        // Güneş Koç 10°'de (yücelme derecesi): ayrı basamak yoktur, Koç'un lordu Mars'a göre
        // okunur. Doğal: Güneş → Mars dost. Geçici: Mars Akrep'te, Koç'tan 8. burç → düşman.
        // Dost + düşman = nötr.
        assert_eq!(rung(Sun, &with(&[])), (Sign::Aries, Dignity::Neutral));
        // Mars Boğa'ya (2. burç) alınırsa geçici dost: dost + dost = çok dost.
        assert_eq!(rung(Sun, &with(&[(Mars, 40.0)])).1, Dignity::GreatFriend);
        // Ay Boğa 10°'de (moolatrikona aralığı): ayrı basamak yoktur. Boğa'nın lordu Venüs de
        // Boğa'da (aynı burç → geçici düşman); doğal Ay → Venüs nötr: düşman.
        assert_eq!(rung(Moon, &with(&[(Moon, 40.0)])).1, Dignity::Enemy);
        // Venüs İkizler'e (2. burç) alınırsa nötr + dost = dost.
        assert_eq!(
            rung(Moon, &with(&[(Moon, 40.0), (Venus, 70.0)])).1,
            Dignity::Friend
        );
        // Merkür Yengeç 5°'de: lord Ay da Yengeç'te (geçici düşman); doğal Merkür → Ay düşman:
        // çok düşman.
        assert_eq!(
            rung(Mercury, &with(&[(Mercury, 95.0)])).1,
            Dignity::GreatEnemy
        );
        // Güneş Aslan 5°'te (moolatrikona aralığı) kendi burcundadır: 20.
        assert_eq!(rung(Sun, &with(&[(Sun, 125.0)])).1, Dignity::OwnSign);
        // Basamakların değeri.
        let points: Vec<f64> = [
            Dignity::OwnSign,
            Dignity::GreatFriend,
            Dignity::Friend,
            Dignity::Neutral,
            Dignity::Enemy,
            Dignity::GreatEnemy,
        ]
        .into_iter()
        .map(rung_points)
        .collect();
        assert_eq!(points, [20.0, 18.0, 15.0, 10.0, 7.0, 5.0]);
        // Düğümlere değer verilmez.
        for node in [Rahu, Ketu] {
            assert_eq!(varga_relation(node, D1, &with(&[])), None);
            assert_eq!(
                vimshopaka_score(node, VargaScheme::Shodashavarga, &with(&[])),
                0.0
            );
        }
    }

    #[test]
    fn hand_example_sun_at_the_start_of_leo() {
        // Güneş Aslan 0° (120°): tek, sabit, ateş burcu; her vargada ilk parça. Mars Başak'ta
        // (160°), Jüpiter Aslan'da (130°).
        // D1 Aslan. D2 tek burcun ilk yarısı: Güneş'in horası, Aslan. D3, D4, D12, D60 ilk
        // parça: aynı burç, Aslan. D7 ve D10 tek burçta aynı burçtan sayılır: Aslan. D9 ateş
        // burcu Koç'tan sayılır: Koç. D16 ve D45 sabit burçta Aslan'dan: Aslan. D20 sabit burçta
        // Yay'dan: Yay. D24 tek burçta Aslan'dan: Aslan. D27 ateş burcu Koç'tan: Koç. D30 tek
        // burcun ilk 5°'si Mars'ındır: Koç. D40 tek burçta Koç'tan: Koç.
        // Basamaklar: Aslan kendi burcu → 20. Koç'un lordu Mars: doğal dost; Mars Başak'ta,
        // Aslan'dan 2. burç → geçici dost; çok dost → 18. Yay'ın lordu Jüpiter: doğal dost;
        // Jüpiter Aslan'da, aynı burç → geçici düşman; nötr → 10.
        let p = with(&[(Sun, 120.0), (Mars, 160.0), (Jupiter, 130.0)]);
        let report = vimshopaka(&p, false);
        let sun = &report.grahas[0];
        assert_eq!(sun.graha, Sun);
        let signs: Vec<Sign> = sun.vargas.iter().map(|v| v.sign).collect();
        use Sign::{Aries, Leo, Sagittarius};
        assert_eq!(
            signs,
            [
                Leo,
                Leo,
                Leo,
                Leo,
                Leo,
                Aries,
                Leo,
                Leo,
                Leo,
                Sagittarius,
                Leo,
                Aries,
                Aries,
                Aries,
                Leo,
                Leo
            ]
        );
        assert_eq!(
            points_of(sun),
            [
                20.0, 20.0, 20.0, 20.0, 20.0, 18.0, 20.0, 20.0, 20.0, 10.0, 20.0, 18.0, 18.0, 18.0,
                20.0, 20.0
            ]
        );
        // Shadvarga: 6·20 + 2·20 + 4·20 + 5·18 + 2·20 + 1·18 = 388 → 19,4.
        assert!(close(sun.shadvarga, 19.4));
        // Saptavarga: 5·20 + 2·20 + 3·20 + 2½·20 + 4½·18 + 2·20 + 1·18 = 389 → 19,45.
        assert!(close(sun.saptavarga, 19.45));
        // Dashavarga: 3·20 + 1½·(20 + 20 + 20 + 18 + 20 + 20 + 20 + 18) + 5·20 = 394 → 19,7.
        assert!(close(sun.dashavarga, 19.7));
        // Shodashavarga: 70 + 20 + 20 + 10 + 10 + 54 + 10 + 10 + 40 + 5 + 10 + 9 + 18 + 9 + 10 +
        // 80 = 385 → 19,25.
        assert!(close(sun.shodashavarga, 19.25));
    }

    #[test]
    fn hand_example_moon_at_the_start_of_taurus_scores_by_the_sign_lords() {
        // Ay Boğa 0° (30°; yücelme burcu): çift, sabit, toprak burcu; her vargada ilk parça.
        // Güneş Balık'ta (340°), Mars Boğa'da (45°), Jüpiter Aslan'da (125°), Venüs İkizler'de
        // (65°), Satürn Akrep'te (220°).
        // D1, D3, D4, D12, D60 Boğa. D2 çift burcun ilk yarısı: Ay'ın horası, Yengeç. D7 çift
        // burçta yedinci burçtan: Akrep. D9 toprak burcu Oğlak'tan: Oğlak. D10 çift burçta
        // dokuzuncu burçtan: Oğlak. D16 ve D45 sabit burçta Aslan'dan: Aslan. D20 sabit burçta
        // Yay'dan: Yay. D24 çift burçta Yengeç'ten: Yengeç. D27 toprak burcu Yengeç'ten: Yengeç.
        // D30 çift burcun ilk 5°'si Venüs'ündür: Boğa. D40 çift burçta Terazi'den: Terazi.
        // Basamaklar (geçici dostluk Boğa'dan sayılır): Venüs doğal nötr, İkizler 2. burç → dost
        // → 15. Yengeç kendi burcu → 20. Mars doğal nötr, Boğa'da (aynı burç) → düşman → 7.
        // Satürn doğal nötr, Akrep 7. burç → düşman → 7. Güneş doğal dost, Balık 11. burç → çok
        // dost → 18. Jüpiter doğal nötr, Aslan 4. burç → dost → 15.
        let p = with(&[
            (Sun, 340.0),
            (Moon, 30.0),
            (Mars, 45.0),
            (Jupiter, 125.0),
            (Venus, 65.0),
            (Saturn, 220.0),
        ]);
        let report = vimshopaka(&p, false);
        let moon = &report.grahas[1];
        assert_eq!(moon.graha, Moon);
        use Sign::{Cancer, Capricorn, Leo, Libra, Sagittarius, Scorpio, Taurus};
        let signs: Vec<Sign> = moon.vargas.iter().map(|v| v.sign).collect();
        assert_eq!(
            signs,
            [
                Taurus,
                Cancer,
                Taurus,
                Taurus,
                Scorpio,
                Capricorn,
                Capricorn,
                Taurus,
                Leo,
                Sagittarius,
                Cancer,
                Cancer,
                Taurus,
                Libra,
                Leo,
                Taurus
            ]
        );
        assert_eq!(
            points_of(moon),
            [
                15.0, 20.0, 15.0, 15.0, 7.0, 7.0, 7.0, 15.0, 18.0, 15.0, 20.0, 20.0, 15.0, 15.0,
                18.0, 15.0
            ]
        );
        assert_eq!(moon.vargas[0].relation, Dignity::Friend);
        assert_eq!(moon.vargas[1].relation, Dignity::OwnSign);
        // Shadvarga: 6·15 + 2·20 + 4·15 + 5·7 + 2·15 + 1·15 = 270 → 13,5.
        assert!(close(moon.shadvarga, 13.5));
        // Saptavarga: 75 + 40 + 45 + 17,5 + 31,5 + 30 + 15 = 254 → 12,7.
        assert!(close(moon.saptavarga, 12.7));
        // Dashavarga: 3·15 + 1½·(20 + 15 + 7 + 7 + 7 + 15 + 18 + 15) + 5·15 = 276 → 13,8.
        assert!(close(moon.dashavarga, 13.8));
        // Shodashavarga: 52,5 + 20 + 15 + 7,5 + 3,5 + 21 + 3,5 + 7,5 + 36 + 7,5 + 10 + 10 + 15 +
        // 7,5 + 9 + 60 = 285,5 → 14,275.
        assert!(close(moon.shodashavarga, 14.275));
    }

    #[test]
    fn the_report_lists_seven_grahas_and_every_score_stays_between_five_and_twenty() {
        let mut lowest = f64::MAX;
        let mut highest = f64::MIN;
        for k in 0..240 {
            // Dokuz graha ayrı hızlarla yürür: ızgara bütün burçları ve parçaları gezer.
            let lons: [f64; 9] =
                std::array::from_fn(|i| f64::from(k) * (7.3 + 3.1 * i as f64) + 11.0 * i as f64);
            let p = Positions::synthetic(None, lons);
            let report = vimshopaka(&p, k % 2 == 0);
            assert_eq!(report.approximate, k % 2 == 0);
            assert_eq!(report.method, "vimshopaka.bphs_v1");
            let order: Vec<Graha> = report.grahas.iter().map(|g| g.graha).collect();
            assert_eq!(order, VIMSHOPAKA_GRAHAS);
            for row in &report.grahas {
                let vargas: Vec<Varga> = row.vargas.iter().map(|v| v.varga).collect();
                assert_eq!(vargas, Varga::ALL);
                for v in &row.vargas {
                    assert_eq!(v.points, rung_points(v.relation));
                    assert!([20.0, 18.0, 15.0, 10.0, 7.0, 5.0].contains(&v.points));
                    assert_eq!(
                        v.relation == Dignity::OwnSign,
                        own_signs(row.graha).contains(&v.sign)
                    );
                }
                let scores = [
                    row.shadvarga,
                    row.saptavarga,
                    row.dashavarga,
                    row.shodashavarga,
                ];
                for (scheme, score) in VargaScheme::ALL.into_iter().zip(scores) {
                    // Değer, raporun kendi satırlarından yeniden toplanır.
                    let again: f64 = scheme
                        .weights()
                        .iter()
                        .map(|&(varga, w)| {
                            w * row
                                .vargas
                                .iter()
                                .find(|v| v.varga == varga)
                                .expect("varga")
                                .points
                        })
                        .sum::<f64>()
                        / 20.0;
                    assert!(close(score, again), "{:?} {scheme:?}", row.graha);
                    assert!((5.0..=20.0).contains(&score), "{:?} {scheme:?}", row.graha);
                    lowest = lowest.min(score);
                    highest = highest.max(score);
                }
            }
        }
        // Izgara aralığın iki yarısını da görür.
        assert!(lowest < 10.0 && highest > 15.0, "{lowest} {highest}");
    }

    #[test]
    fn the_temporal_relation_is_read_from_d1_and_the_vargas_from_the_rasi_frame() {
        // Güneş Koç 10°'de: D9 burcu Yengeç'tir (Koç'un dördüncü navamsası: 10°–13°20′), lordu
        // Ay. Ay'ın D1 burcu Yengeç (100°), Koç'tan 4. burç → geçici dost; doğal dost: çok dost.
        assert_eq!(
            varga_relation(Sun, D9, &with(&[])),
            Some((Sign::Cancer, Dignity::GreatFriend))
        );
        // Ay Koç'a (5°) alınırsa D1'de Güneş'le aynı burçtadır → geçici düşman; dost + düşman =
        // nötr. Geçici dostluk D9 burçlarından okunsaydı sonuç değişirdi: Ay'ın D9 burcu Boğa
        // (Koç'un ikinci navamsası), Güneş'in D9 burcu Yengeç'ten 11. burçtur → geçici dost →
        // çok dost.
        assert_eq!(
            varga_relation(Sun, D9, &with(&[(Moon, 5.0)])),
            Some((Sign::Cancer, Dignity::Neutral))
        );
        // Tropikal-rasi kipi: vargalar rasi çerçevesindeki boylamdan hesaplanır. Sidereal 346°
        // (Balık 16°) + 24° = Koç 10°.
        let mut frame = with(&[(Sun, 346.0)]);
        for g in &mut frame.grahas {
            let lon = normalize(g.sidereal_lon + 24.0);
            g.rasi_sign = Sign::of(lon);
            g.deg_in_sign = deg_in_sign(lon);
        }
        assert_eq!(varga_relation(Sun, D1, &frame).expect("sun").0, Sign::Aries);
        assert_eq!(
            varga_relation(Sun, D9, &frame).expect("sun").0,
            Sign::Cancer
        );
    }
}
