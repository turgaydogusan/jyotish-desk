//! Panchang: tithi, vara, nakshatra, yoga ve karana; gün doğumu ve batımı (spec §3, §5, §6).
//!
//! Tanımlar (Surya Siddhanta böl. 2, Burgess çevirisi; Muhurta Chintamani böl. 1):
//! - Tithi: Ay − Güneş uzanımının 12°'lik dilimi, 1..=30. 1–15 Shukla, 16–30 Krishna paksha.
//! - Karana: yarım tithi (6°), 1..=60. 1 Kimstughna (sabit); 2–57 yedi hareketli karananın
//!   (Bava, Balava, Kaulava, Taitila, Garaja, Vanija, Vishti) sekiz turu; 58 Shakuni,
//!   59 Chatushpada, 60 Naga (sabit).
//! - Nakshatra: Ay'ın sidereal boylamının 13°20′'lik dilimi; pada 3°20′.
//! - Nitya yoga: Güneş ve Ay sidereal boylamları toplamının 13°20′'lik dilimi, 1..=27.
//! - Vara: hafta günü; gün doğumunda değişir (spec §6).
//!
//! Uzanım iki boylamın farkı olduğu için ayanamsadan bağımsızdır; yoga toplamı değildir.
//! Zamanlar UT Jülyen günüdür.

use serde::{Deserialize, Serialize};

use crate::zodiac::normalize;

pub const TITHI_SPAN_DEG: f64 = 12.0;
pub const KARANA_SPAN_DEG: f64 = 6.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Paksha {
    /// Büyüyen Ay: tithi 1–15.
    Shukla,
    /// Küçülen Ay: tithi 16–30.
    Krishna,
}

/// Hafta günü: Ravi, Soma, Mangala, Budha, Guru, Shukra, Shani.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Vara {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl Vara {
    /// 0 = Pazar.
    pub const ALL: [Vara; 7] = [
        Vara::Sunday,
        Vara::Monday,
        Vara::Tuesday,
        Vara::Wednesday,
        Vara::Thursday,
        Vara::Friday,
        Vara::Saturday,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum KaranaName {
    /// Sabit: Shukla 1. tithinin 1. yarısı (karana 1).
    Kimstughna,
    Bava,
    Balava,
    Kaulava,
    Taitila,
    Garaja,
    Vanija,
    /// Bhadra olarak da bilinir.
    Vishti,
    /// Sabit: Krishna 14. tithinin 2. yarısı (karana 58).
    Shakuni,
    /// Sabit: Amavasya'nın 1. yarısı (karana 59).
    Chatushpada,
    /// Sabit: Amavasya'nın 2. yarısı (karana 60).
    Naga,
}

const MOVABLE_KARANAS: [KaranaName; 7] = [
    KaranaName::Bava,
    KaranaName::Balava,
    KaranaName::Kaulava,
    KaranaName::Taitila,
    KaranaName::Garaja,
    KaranaName::Vanija,
    KaranaName::Vishti,
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Tithi {
    /// 1..=30.
    pub no: u8,
    pub paksha: Paksha,
    pub ends_jd_ut: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PanchangNakshatra {
    /// Ay'ın nakshatrası, 1..=27.
    pub no: u8,
    /// 1..=4.
    pub pada: u8,
    pub ends_jd_ut: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct NityaYoga {
    /// 1 Vishkambha … 27 Vaidhriti.
    pub no: u8,
    pub ends_jd_ut: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Karana {
    /// 1..=60 (yarım tithi sırası).
    pub no: u8,
    pub name: KaranaName,
    pub ends_jd_ut: f64,
}

/// Doğum anının panchangı (spec §5 `Panchang`). Tithi, nakshatra, yoga ve karana doğum
/// anındaki değerdir; `ends_jd_ut` o andan sonraki ilk sınırdır. Vara doğumdan önceki son
/// gün doğumundan gelir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Panchang {
    pub tithi: Tithi,
    pub vara: Vara,
    pub nakshatra: PanchangNakshatra,
    pub yoga: NityaYoga,
    pub karana: Karana,
    /// Doğumdan önceki son gün doğumu; kutup gecesi/gündüzünde `None`.
    pub sunrise_jd_ut: Option<f64>,
    /// O gün doğumundan sonraki ilk gün batımı (1 gün içinde); yoksa `None`.
    pub sunset_jd_ut: Option<f64>,
}

/// Ay − Güneş uzanımı, [0, 360).
pub fn elongation(sun: f64, moon: f64) -> f64 {
    normalize(moon - sun)
}

/// Uzanımdan tithi, 1..=30.
pub fn tithi_no(elongation: f64) -> u8 {
    ((normalize(elongation) / TITHI_SPAN_DEG) as u8).min(29) + 1
}

pub fn paksha(tithi_no: u8) -> Paksha {
    if tithi_no <= 15 {
        Paksha::Shukla
    } else {
        Paksha::Krishna
    }
}

/// Uzanımdan karana sırası, 1..=60.
pub fn karana_no(elongation: f64) -> u8 {
    ((normalize(elongation) / KARANA_SPAN_DEG) as u8).min(59) + 1
}

/// Karana sırasından ad. 1..=60 dışı değerler en yakın uca düşer (panik yok).
pub fn karana_name(no: u8) -> KaranaName {
    match no {
        0 | 1 => KaranaName::Kimstughna,
        2..=57 => MOVABLE_KARANAS[usize::from(no - 2) % 7],
        58 => KaranaName::Shakuni,
        59 => KaranaName::Chatushpada,
        _ => KaranaName::Naga,
    }
}

/// Sidereal Güneş + Ay toplamından nitya yoga, 1..=27.
pub fn nitya_yoga_no(sun_sidereal: f64, moon_sidereal: f64) -> u8 {
    ((normalize(sun_sidereal + moon_sidereal) * 3.0 / 40.0) as u8).min(26) + 1
}

/// `jd_ut` anının `geo_lon`'daki yerel ortalama tarihinin hafta günü: gün numarası
/// `floor(jd + 0.5 + boylam / 360)`, hafta günü `(gün + 1) mod 7`, 0 = Pazar.
pub fn local_weekday(jd_ut: f64, geo_lon: f64) -> Vara {
    let day = (jd_ut + 0.5 + geo_lon / 360.0).floor();
    // rem_euclid [0, 7) verir; NaN `as usize` ile 0'a düşer, min(6) sınırı korur.
    let i = (day + 1.0).rem_euclid(7.0) as usize;
    Vara::ALL[i.min(6)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tithi_and_paksha_follow_twelve_degree_steps() {
        assert_eq!(tithi_no(0.0), 1);
        assert_eq!(tithi_no(11.999_999), 1);
        assert_eq!(tithi_no(12.0), 2);
        assert_eq!(tithi_no(179.999_999), 15);
        assert_eq!(tithi_no(180.0), 16);
        assert_eq!(tithi_no(359.999_999), 30);
        assert_eq!(tithi_no(-1.0), 30);
        assert_eq!(tithi_no(372.0), 2);
        assert_eq!(paksha(1), Paksha::Shukla);
        assert_eq!(paksha(15), Paksha::Shukla);
        assert_eq!(paksha(16), Paksha::Krishna);
        assert_eq!(paksha(30), Paksha::Krishna);
        // Ay − Güneş: 71 − 346 = −275 → 85°.
        assert_eq!(elongation(346.0, 71.0), 85.0);
    }

    #[test]
    fn karana_has_four_fixed_and_seven_movable() {
        use KaranaName::*;
        assert_eq!(karana_no(0.0), 1);
        assert_eq!(karana_no(5.999_999), 1);
        assert_eq!(karana_no(6.0), 2);
        assert_eq!(karana_no(359.999_999), 60);
        let names: Vec<KaranaName> = (1..=60u8).map(karana_name).collect();
        assert_eq!(names[0], Kimstughna);
        let movable = [Bava, Balava, Kaulava, Taitila, Garaja, Vanija, Vishti];
        assert_eq!(names[1..8], movable);
        assert_eq!(names[8..15], movable);
        assert_eq!(names[50..57], movable);
        assert_eq!(names[57..], [Shakuni, Chatushpada, Naga]);
        for m in movable {
            assert_eq!(names.iter().filter(|&&n| n == m).count(), 8, "{m:?}");
        }
        for f in [Kimstughna, Shakuni, Chatushpada, Naga] {
            assert_eq!(names.iter().filter(|&&n| n == f).count(), 1, "{f:?}");
        }
        // Geçersiz numaralar panik yapmaz.
        assert_eq!(karana_name(0), Kimstughna);
        assert_eq!(karana_name(255), Naga);
    }

    #[test]
    fn vishti_falls_on_the_eight_traditional_half_tithis() {
        // Muhurta Chintamani: Vishti (Bhadra) Shukla 4. tithinin 2. yarısı, 8.'in 1., 11.'in 2.,
        // 15.'in 1. yarısı; Krishna 3.'nün 2., 7.'nin 1., 10.'un 2., 14.'ün 1. yarısı.
        // Tithi t'nin 1. yarısı karana 2t − 1, 2. yarısı 2t (Krishna için t = 15 + gün).
        let vishti: Vec<u8> = (1..=60u8)
            .filter(|&k| karana_name(k) == KaranaName::Vishti)
            .collect();
        assert_eq!(vishti, [8, 15, 22, 29, 36, 43, 50, 57]);
    }

    #[test]
    fn nitya_yoga_uses_the_sidereal_sum() {
        assert_eq!(nitya_yoga_no(0.0, 0.0), 1);
        assert_eq!(nitya_yoga_no(6.0, 7.333_334), 2); // 13.333334° > 13°20′
        assert_eq!(nitya_yoga_no(346.0, 71.0), 5); // 417° → 57°: Shobhana
        assert_eq!(nitya_yoga_no(180.0, 179.999_999), 27);
    }

    #[test]
    fn weekday_uses_the_local_mean_date() {
        // JD 2451545.0 = 1 Ocak 2000 12:00 UT, Cumartesi.
        assert_eq!(local_weekday(2_451_545.0, 0.0), Vara::Saturday);
        // 31 Aralık 1999 21:36 UT: Greenwich'te Cuma, New Delhi'de (≈ +5 sa 9 dk) 1 Ocak.
        assert_eq!(local_weekday(2_451_544.4, 0.0), Vara::Friday);
        assert_eq!(local_weekday(2_451_544.4, 77.209), Vara::Saturday);
        // Golden gün doğumları (Task 8): Londra 31.12.1849 Pazartesi, Buenos Aires 14.6.1923
        // Perşembe. İkisinde de doğum UT tarihinden farklı bir yerel güne düşer.
        assert_eq!(local_weekday(2_396_757.842_784, -0.1278), Vara::Monday);
        assert_eq!(local_weekday(2_423_584.959_912, -58.3816), Vara::Thursday);
        assert_eq!(Vara::ALL[0], Vara::Sunday);
        assert_eq!(Vara::ALL[6], Vara::Saturday);
    }
}
