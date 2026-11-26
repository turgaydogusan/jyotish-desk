//! Günün dilimleri ve muhurta sınıfları (ileri modüller spec'i §10): gündüzün sekizde bir
//! dilimleri ve varaya göre işaretli üçü, Abhijit, hora tablosu; tithi grupları, nakshatra
//! doğaları, tara bala ve chandra bala. Saf fonksiyonlardır; ephemeris görmezler (K3).
//!
//! Buradaki tablolar ve listeler klasik muhurta derlemelerinin yaygın biçimidir ve hüküm
//! taşımaz: bir dilimin ya da sınıfın "uygun" olup olmadığına süzgeci kuran astrolog karar
//! verir. "Elverişli" kümeleri (tara ve chandra bala) spec'in yazdığı kümelerdir.

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::kuta::tara_number;
use crate::panchang::{karana_name, KaranaName, Vara};
use crate::shadbala::{hora_lord, vara_lord};
use crate::timeline::{Interval, NakshatraPada, SunDay};
use crate::zodiac::Sign;

/// Gündüz (gün doğumu → batımı) bu kadar eşit dilime bölünür.
pub const DAY_EIGHTHS: u8 = 8;
/// Gündüzün eşit muhurta sayısı.
pub const DAY_MUHURTAS: u8 = 15;
/// Abhijit, gündüzün on beş muhurtasının sekizincisidir.
pub const ABHIJIT_MUHURTA: u8 = 8;
/// Gün doğumundan sonraki gün doğumuna saat dilimi (hora) sayısı.
pub const HORAS_PER_DAY: u8 = 24;

/// Varaya göre Rahu dilimi (1..=8), Pazar'dan Cumartesi'ye.
pub const RAHU_SEGMENTS: [u8; 7] = [8, 2, 7, 5, 6, 4, 3];

fn weekday_index(vara: Vara) -> u8 {
    vara as u8
}

/// Rahu'ya ait sekizde bir dilim, 1..=8.
pub fn rahu_segment(vara: Vara) -> u8 {
    RAHU_SEGMENTS[usize::from(weekday_index(vara))]
}

/// Yamaganda dilimi, 1..=7: Perşembe 1, Çarşamba 2, Salı 3, Pazartesi 4, Pazar 5, Cumartesi 6,
/// Cuma 7 (hafta geriye doğru yürür).
pub fn yamaganda_segment(vara: Vara) -> u8 {
    (4 + 7 - weekday_index(vara)) % 7 + 1
}

/// Gulika dilimi, 1..=7: Cumartesi 1, Cuma 2 … Pazar 7.
pub fn gulika_segment(vara: Vara) -> u8 {
    7 - weekday_index(vara)
}

/// `[start, end)` aralığının `parts` eşit dilimi; değer dilimin sırasıdır (1'den). Son dilim
/// tam `end`'de biter.
pub fn equal_parts(start_jd_ut: f64, end_jd_ut: f64, parts: u8) -> Vec<Interval<u8>> {
    let at = |k: u8| {
        if k == parts {
            end_jd_ut
        } else {
            start_jd_ut + (end_jd_ut - start_jd_ut) * f64::from(k) / f64::from(parts)
        }
    };
    (1..=parts)
        .map(|k| Interval {
            value: k,
            start_jd_ut: at(k - 1),
            end_jd_ut: at(k),
        })
        .collect()
}

/// Günün dilimleri. `rahu`, `yamaganda` ve `gulika`'nın değeri dilimin sırasıdır (1..=8),
/// `abhijit`'in değeri muhurtanın sırasıdır (8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DaySegments {
    /// Gündüzün sekiz eşit dilimi, sırayla.
    pub eighths: Vec<Interval<u8>>,
    pub rahu: Interval<u8>,
    pub yamaganda: Interval<u8>,
    pub gulika: Interval<u8>,
    pub abhijit: Interval<u8>,
    /// Çarşamba: bazı derlemeler o gün Abhijit'i kullanmaz. Olgudur; dilim yine verilir.
    pub abhijit_weekday_exception: bool,
    /// 24 saat dilimi ve lordları: gün doğumundan sonraki gün doğumuna.
    pub horas: Vec<Interval<Graha>>,
}

/// Doğuşu, batışı ve sonraki doğuşu bilinen günün dilimleri.
pub fn day_segments(day: &SunDay) -> DaySegments {
    let eighths = equal_parts(day.sunrise_jd_ut, day.sunset_jd_ut, DAY_EIGHTHS);
    let pick = |segment: u8| eighths[usize::from(segment) - 1];
    let muhurtas = equal_parts(day.sunrise_jd_ut, day.sunset_jd_ut, DAY_MUHURTAS);
    let lord = vara_lord(day.vara);
    let horas = equal_parts(day.sunrise_jd_ut, day.next_sunrise_jd_ut, HORAS_PER_DAY)
        .into_iter()
        .map(|h| Interval {
            value: hora_lord(lord, h.value - 1),
            start_jd_ut: h.start_jd_ut,
            end_jd_ut: h.end_jd_ut,
        })
        .collect();
    DaySegments {
        rahu: pick(rahu_segment(day.vara)),
        yamaganda: pick(yamaganda_segment(day.vara)),
        gulika: pick(gulika_segment(day.vara)),
        abhijit: muhurtas[usize::from(ABHIJIT_MUHURTA) - 1],
        abhijit_weekday_exception: day.vara == Vara::Wednesday,
        eighths,
        horas,
    }
}

/// Tithi grubu: paksha içi sıranın beşli döngüsü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum TithiGroup {
    /// 1, 6, 11.
    Nanda,
    /// 2, 7, 12.
    Bhadra,
    /// 3, 8, 13.
    Jaya,
    /// 4, 9, 14.
    Rikta,
    /// 5, 10, 15 (Purnima ve Amavasya dahil).
    Purna,
}

impl TithiGroup {
    pub const ALL: [TithiGroup; 5] = [
        TithiGroup::Nanda,
        TithiGroup::Bhadra,
        TithiGroup::Jaya,
        TithiGroup::Rikta,
        TithiGroup::Purna,
    ];
}

/// Tithinin (1..=30) grubu; iki pakshada aynıdır. Aralık dışı değer panik.
pub fn tithi_group(tithi: u8) -> TithiGroup {
    assert!((1..=30).contains(&tithi), "tithi {tithi} outside 1..=30");
    TithiGroup::ALL[usize::from((tithi - 1) % 5)]
}

/// Nakshatranın doğası (muhurta sınıfı).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum NakshatraNature {
    /// Dhruva: sabit.
    Fixed,
    /// Chara: yürük.
    Movable,
    /// Ugra: sert.
    Fierce,
    /// Mishra: karışık.
    Mixed,
    /// Kshipra: çevik.
    Swift,
    /// Mridu: yumuşak.
    Soft,
    /// Tikshna: keskin.
    Sharp,
}

impl NakshatraNature {
    pub const ALL: [NakshatraNature; 7] = [
        NakshatraNature::Fixed,
        NakshatraNature::Movable,
        NakshatraNature::Fierce,
        NakshatraNature::Mixed,
        NakshatraNature::Swift,
        NakshatraNature::Soft,
        NakshatraNature::Sharp,
    ];
}

/// Nakshatra doğaları, Ashwini'den Revati'ye.
pub const NAKSHATRA_NATURES: [NakshatraNature; 27] = {
    use NakshatraNature::{Fierce, Fixed, Mixed, Movable, Sharp, Soft, Swift};
    [
        Swift, Fierce, Mixed, Fixed, Soft, Sharp, Movable, Swift, Sharp, Fierce, Fierce, Fixed,
        Swift, Soft, Movable, Mixed, Soft, Sharp, Sharp, Fierce, Fixed, Movable, Movable, Movable,
        Fierce, Fixed, Soft,
    ]
};

/// Nakshatranın (1..=27) doğası. Aralık dışı değer panik.
pub fn nakshatra_nature(nakshatra: u8) -> NakshatraNature {
    assert!(
        (1..=27).contains(&nakshatra),
        "nakshatra {nakshatra} outside 1..=27"
    );
    NAKSHATRA_NATURES[usize::from(nakshatra) - 1]
}

/// Rikta tithi: her iki pakshanın 4., 9. ve 14. tithisi.
pub fn is_rikta(tithi: u8) -> bool {
    tithi_group(tithi) == TithiGroup::Rikta
}

/// Vishti karana (1..=60 sırasından).
pub fn is_vishti(karana: u8) -> bool {
    karana_name(karana) == KaranaName::Vishti
}

/// Vyatipata nitya yogasının sırası.
pub const VYATIPATA_YOGA: u8 = 17;
/// Vaidhriti nitya yogasının sırası.
pub const VAIDHRITI_YOGA: u8 = 27;
/// Panchaka: Ay'ın sidereal boylamı bu değerden büyük ya da eşit (300°–360°).
pub const PANCHAKA_START_DEG: f64 = 300.0;

/// Panchaka: Ay Dhanishta'nın üçüncü padasından Revati'nin sonuna (sidereal 300°–360°).
pub fn is_panchaka(moon: NakshatraPada) -> bool {
    moon.nakshatra > 23 || (moon.nakshatra == 23 && moon.pada >= 3)
}

/// Elverişli taralar: Sampat (2), Kshema (4), Sadhana (6), Mitra (8), Parama Mitra (9).
pub const TARA_FAVOURABLE: [u8; 5] = [2, 4, 6, 8, 9];
/// Elverişli chandra bala evleri: geçiş Ay'ının doğum Ay burcundan sayısı.
pub const CHANDRA_FAVOURABLE: [u8; 6] = [1, 3, 6, 7, 10, 11];

/// Tara bala: doğum nakshatrasından günün nakshatrasına sayımın dokuzlu döngüdeki yeri
/// (1 Janma … 9 Parama Mitra) ve elverişli kümede olup olmadığı.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TaraBala {
    pub number: u8,
    pub favourable: bool,
}

/// İki nakshatra da 1..=27 olmalıdır (aralık dışı değer panik; toplayıcılar girdiyi denetler).
pub fn tara_bala(natal_nakshatra: u8, day_nakshatra: u8) -> TaraBala {
    let number = tara_number(natal_nakshatra, day_nakshatra);
    TaraBala {
        number,
        favourable: TARA_FAVOURABLE.contains(&number),
    }
}

/// Chandra bala: geçiş Ay'ının doğum Ay burcundan sayılan evi ve elverişli kümede olup
/// olmadığı. İki burç da rasi çerçevesindedir (K5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ChandraBala {
    pub house: u8,
    pub favourable: bool,
}

pub fn chandra_bala(natal_sign: Sign, moon_sign: Sign) -> ChandraBala {
    let house = moon_sign.house_from(natal_sign);
    ChandraBala {
        house,
        favourable: CHANDRA_FAVOURABLE.contains(&house),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shadbala::HORA_CYCLE;

    /// Elle yazılmış ikinci tablo (§13 katman 3): dilimden varaya. Rahu'nun ilk dilimi hiçbir
    /// güne düşmez.
    const HAND_RAHU_DAY_OF_SEGMENT: [Option<Vara>; 8] = [
        None,
        Some(Vara::Monday),
        Some(Vara::Saturday),
        Some(Vara::Friday),
        Some(Vara::Wednesday),
        Some(Vara::Thursday),
        Some(Vara::Tuesday),
        Some(Vara::Sunday),
    ];
    /// Yamaganda ve Gulika: Pazar'dan Cumartesi'ye, elle.
    const HAND_YAMAGANDA: [u8; 7] = [5, 4, 3, 2, 1, 7, 6];
    const HAND_GULIKA: [u8; 7] = [7, 6, 5, 4, 3, 2, 1];

    #[test]
    fn segment_orders_match_the_hand_tables() {
        for (i, vara) in Vara::ALL.into_iter().enumerate() {
            let rahu = rahu_segment(vara);
            assert_eq!(
                HAND_RAHU_DAY_OF_SEGMENT[usize::from(rahu) - 1],
                Some(vara),
                "{vara:?}"
            );
            assert_eq!(yamaganda_segment(vara), HAND_YAMAGANDA[i], "{vara:?}");
            assert_eq!(gulika_segment(vara), HAND_GULIKA[i], "{vara:?}");
            // Üçü her varada üç ayrı dilimdir.
            let three = [rahu, yamaganda_segment(vara), gulika_segment(vara)];
            assert!(three[0] != three[1] && three[1] != three[2] && three[0] != three[2]);
            assert!(three.iter().all(|s| (1..=8).contains(s)));
        }
        // Rahu yedi günde yedi ayrı dilim alır: 2..=8.
        let mut rahu: Vec<u8> = Vara::ALL.into_iter().map(rahu_segment).collect();
        rahu.sort_unstable();
        assert_eq!(rahu, [2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(
            HAND_RAHU_DAY_OF_SEGMENT.iter().flatten().count(),
            Vara::ALL.len()
        );
    }

    fn minutes(i: &Interval<u8>) -> (f64, f64) {
        (i.start_jd_ut * 1440.0, i.end_jd_ut * 1440.0)
    }

    fn close(a: (f64, f64), b: (f64, f64)) -> bool {
        (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6
    }

    #[test]
    fn a_six_to_six_monday_has_the_textbook_segments() {
        // Gün doğumu 06:00, batımı 18:00, sonraki doğuş ertesi 06:00; gün kesri olarak.
        let day = SunDay {
            sunrise_jd_ut: 0.25,
            sunset_jd_ut: 0.75,
            next_sunrise_jd_ut: 1.25,
            vara: Vara::Monday,
        };
        let s = day_segments(&day);
        assert_eq!(s.eighths.len(), 8);
        // Sekizde bir: 90 dakika. Pazartesi Rahu 2. dilim: 07:30–09:00.
        assert_eq!(s.rahu.value, 2);
        assert!(close(minutes(&s.rahu), (450.0, 540.0)));
        // Yamaganda 4. dilim: 10:30–12:00; Gulika 6. dilim: 13:30–15:00.
        assert!(close(minutes(&s.yamaganda), (630.0, 720.0)));
        assert!(close(minutes(&s.gulika), (810.0, 900.0)));
        // Muhurta 48 dakika; sekizincisi 11:36–12:24.
        assert_eq!(s.abhijit.value, 8);
        assert!(close(minutes(&s.abhijit), (696.0, 744.0)));
        assert!(!s.abhijit_weekday_exception);
        // 24 hora birer saattir; ilki Ay'ındır (Pazartesi), sonra Satürn, Jüpiter, Mars…
        assert_eq!(s.horas.len(), 24);
        assert_eq!(
            s.horas[..8].iter().map(|h| h.value).collect::<Vec<_>>(),
            [
                Graha::Moon,
                Graha::Saturn,
                Graha::Jupiter,
                Graha::Mars,
                Graha::Sun,
                Graha::Venus,
                Graha::Mercury,
                Graha::Moon
            ]
        );
        assert!((s.horas[1].start_jd_ut * 1440.0 - 420.0).abs() < 1e-6);
        assert_eq!(s.horas[23].end_jd_ut, 1.25);
        assert_eq!(s.eighths[7].end_jd_ut, 0.75);
        let wednesday = day_segments(&SunDay {
            vara: Vara::Wednesday,
            ..day
        });
        assert!(wednesday.abhijit_weekday_exception);
    }

    #[test]
    fn hora_lords_start_with_the_day_lord_and_the_next_day_continues_the_cycle() {
        for vara in Vara::ALL {
            let s = day_segments(&SunDay {
                sunrise_jd_ut: 10.25,
                sunset_jd_ut: 10.8,
                next_sunrise_jd_ut: 11.26,
                vara,
            });
            assert_eq!(s.horas[0].value, vara_lord(vara), "{vara:?}");
            for w in s.horas.windows(2) {
                assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
                let at = HORA_CYCLE.iter().position(|g| *g == w[0].value).unwrap();
                assert_eq!(w[1].value, HORA_CYCLE[(at + 1) % 7]);
            }
            // 25. hora, ertesi günün lordudur.
            let next_day = Vara::ALL[(vara as usize + 1) % 7];
            let at = HORA_CYCLE
                .iter()
                .position(|g| *g == s.horas[23].value)
                .unwrap();
            assert_eq!(HORA_CYCLE[(at + 1) % 7], vara_lord(next_day), "{vara:?}");
        }
    }

    /// Spec §10'un listeleri, elle (nakshatra sırası; Ashwini = 1).
    const HAND_GROUPS: [(TithiGroup, [u8; 3]); 5] = [
        (TithiGroup::Nanda, [1, 6, 11]),
        (TithiGroup::Bhadra, [2, 7, 12]),
        (TithiGroup::Jaya, [3, 8, 13]),
        (TithiGroup::Rikta, [4, 9, 14]),
        (TithiGroup::Purna, [5, 10, 15]),
    ];
    const HAND_FIXED: [u8; 4] = [4, 12, 21, 26];
    const HAND_MOVABLE: [u8; 5] = [7, 15, 22, 23, 24];
    const HAND_FIERCE: [u8; 5] = [2, 10, 11, 20, 25];
    const HAND_MIXED: [u8; 2] = [3, 16];
    const HAND_SWIFT: [u8; 3] = [1, 8, 13];
    const HAND_SOFT: [u8; 4] = [5, 14, 17, 27];
    const HAND_SHARP: [u8; 4] = [6, 9, 18, 19];

    #[test]
    fn tithi_groups_and_nakshatra_natures_match_the_hand_lists() {
        for (group, in_paksha) in HAND_GROUPS {
            for t in in_paksha {
                assert_eq!(tithi_group(t), group, "shukla {t}");
                assert_eq!(tithi_group(t + 15), group, "krishna {t}");
            }
        }
        assert_eq!(
            (1..=30).filter(|t| is_rikta(*t)).collect::<Vec<u8>>(),
            [4, 9, 14, 19, 24, 29]
        );
        let lists: [(NakshatraNature, &[u8]); 7] = [
            (NakshatraNature::Fixed, &HAND_FIXED),
            (NakshatraNature::Movable, &HAND_MOVABLE),
            (NakshatraNature::Fierce, &HAND_FIERCE),
            (NakshatraNature::Mixed, &HAND_MIXED),
            (NakshatraNature::Swift, &HAND_SWIFT),
            (NakshatraNature::Soft, &HAND_SOFT),
            (NakshatraNature::Sharp, &HAND_SHARP),
        ];
        let mut seen = 0;
        for (nature, list) in lists {
            for n in list {
                assert_eq!(nakshatra_nature(*n), nature, "nakshatra {n}");
                seen += 1;
            }
        }
        // Yirmi yedi nakshatranın her biri tam bir listededir.
        assert_eq!(seen, 27);
    }

    #[test]
    fn flags_follow_the_karana_the_yoga_and_the_moon() {
        // Vishti sekiz yarım tithide: 8, 15, 22, 29, 36, 43, 50, 57.
        assert_eq!(
            (1..=60).filter(|k| is_vishti(*k)).collect::<Vec<u8>>(),
            [8, 15, 22, 29, 36, 43, 50, 57]
        );
        // Panchaka 300°'de başlar: Dhanishta'nın (23.) üçüncü padası.
        for idx in 0..108u8 {
            let moon = NakshatraPada {
                nakshatra: idx / 4 + 1,
                pada: idx % 4 + 1,
            };
            let start_deg = f64::from(idx) * 10.0 / 3.0;
            assert_eq!(
                is_panchaka(moon),
                start_deg >= PANCHAKA_START_DEG - 1e-9,
                "{moon:?}"
            );
        }
        assert_eq!((VYATIPATA_YOGA, VAIDHRITI_YOGA), (17, 27));
    }

    /// Dokuz taranın adı ve elverişli olanlar, elle.
    const HAND_TARAS: [(&str, bool); 9] = [
        ("Janma", false),
        ("Sampat", true),
        ("Vipat", false),
        ("Kshema", true),
        ("Pratyak", false),
        ("Sadhana", true),
        ("Naidhana", false),
        ("Mitra", true),
        ("Parama Mitra", true),
    ];

    /// Adım adım sayım: `from`'dan başlayıp `to`'ya varana dek nakshatra nakshatra yürür ve
    /// dokuzlu döngüde sayar (formül kullanmaz).
    fn hand_tara(from: u8, to: u8) -> u8 {
        let (mut at, mut count) = (from, 1);
        while at != to {
            at = if at == 27 { 1 } else { at + 1 };
            count = if count == 9 { 1 } else { count + 1 };
        }
        count
    }

    /// Chandra bala evleri, elle: satır doğum Ay burcu, sütun geçiş Ay'ının burcu (Koç'tan).
    const HAND_CHANDRA_HOUSES: [[u8; 12]; 12] = [
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        [12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        [11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        [10, 11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        [9, 10, 11, 12, 1, 2, 3, 4, 5, 6, 7, 8],
        [8, 9, 10, 11, 12, 1, 2, 3, 4, 5, 6, 7],
        [7, 8, 9, 10, 11, 12, 1, 2, 3, 4, 5, 6],
        [6, 7, 8, 9, 10, 11, 12, 1, 2, 3, 4, 5],
        [5, 6, 7, 8, 9, 10, 11, 12, 1, 2, 3, 4],
        [4, 5, 6, 7, 8, 9, 10, 11, 12, 1, 2, 3],
        [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 1, 2],
        [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 1],
    ];
    /// Elverişli evler, elle (sıra: ev 1…12).
    const HAND_CHANDRA_FAVOURABLE: [bool; 12] = [
        true, false, true, false, false, true, true, false, false, true, true, false,
    ];

    #[test]
    fn tara_and_chandra_bala_match_the_hand_tables_on_every_pair() {
        // 27 × 27 nakshatra çifti.
        for from in 1..=27u8 {
            for to in 1..=27u8 {
                let t = tara_bala(from, to);
                assert_eq!(t.number, hand_tara(from, to), "{from} → {to}");
                assert_eq!(
                    t.favourable,
                    HAND_TARAS[usize::from(t.number) - 1].1,
                    "{from} → {to}: {}",
                    HAND_TARAS[usize::from(t.number) - 1].0
                );
            }
        }
        // 12 × 12 burç çifti.
        for (i, natal) in Sign::ALL.into_iter().enumerate() {
            for (j, moon) in Sign::ALL.into_iter().enumerate() {
                let c = chandra_bala(natal, moon);
                assert_eq!(c.house, HAND_CHANDRA_HOUSES[i][j], "{natal:?} → {moon:?}");
                assert_eq!(
                    c.favourable,
                    HAND_CHANDRA_FAVOURABLE[usize::from(c.house) - 1]
                );
            }
        }
    }

    #[test]
    fn equal_parts_end_exactly_on_the_span() {
        let parts = equal_parts(2_451_545.123, 2_451_545.777, 15);
        assert_eq!(parts.len(), 15);
        assert_eq!(parts[0].start_jd_ut, 2_451_545.123);
        assert_eq!(parts[14].end_jd_ut, 2_451_545.777);
        for w in parts.windows(2) {
            assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
            assert_eq!(w[0].value + 1, w[1].value);
        }
    }

    #[test]
    #[should_panic(expected = "outside 1..=30")]
    fn a_tithi_outside_the_range_is_a_programming_error() {
        tithi_group(31);
    }
}
