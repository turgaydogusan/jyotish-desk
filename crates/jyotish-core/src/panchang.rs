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

use ephemeris::{Body, Ephemeris, RiseSet, SunDisc, Zodiac};
use serde::{Deserialize, Serialize};

use crate::chart::check_moment;
use crate::settings::CalcSettings;
use crate::zodiac::{nakshatra_pada, normalize, NAKSHATRA_SPAN_DEG};
use crate::CoreError;

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

/// Bitiş araması: sonraki sınır en geç bu kadar gün sonradır. Ay'ın jeosantrik boylam hızı
/// ≈ 11.8–15.4°/gün, Güneş'inki 0.95–1.02°/gün: uzanım ≥ 10.7°/gün, Ay ≥ 11.8°/gün, toplam
/// ≥ 12.7°/gün. En uzun dilim 13°20′ / 11.8 ≈ 1.13 gündür. Aralıkta açı en çok ~25° ilerler,
/// yani hedefe ±180° sarmalı uzaklık aralık boyunca sürekli ve artandır.
pub const END_SEARCH_DAYS: f64 = 1.5;
/// Sabit adım sayısı: 1.5 gün / 2^20 ≈ 0.12 sn (gereken 18 adım ≈ 0.5 sn).
pub const END_BISECTIONS: u32 = 20;
/// Doğum gününün gün doğumu, doğumdan en çok bu kadar önce aranır. Ardışık gün doğumları
/// 65° enleme kadar ~24 sa ± 15 dk arayla gelir; pencerede en çok iki doğuş olur.
pub const SUNRISE_WINDOW_DAYS: f64 = 1.25;
/// İlk aramada doğuş bulunamazsa ikinci aramanın başlangıcı (doğumdan bu kadar önce).
const SUNRISE_RETRY_DAYS: f64 = 1.0;
/// Pencerede taranan en çok doğuş (güvenlik sınırı).
const MAX_SUNRISE_STEPS: usize = 4;
/// Bulunan doğuşun ardından bir sonrakini aramak için atlanan süre (≈ 14 dk).
const RISE_STEP_DAYS: f64 = 0.01;
/// Gün batımı gün doğumundan sonra bu süre içinde olmalıdır; değilse yok sayılır.
const MAX_DAYLIGHT_DAYS: f64 = 1.0;
/// Gün doğumu yoksa günün başladığı yerel ortalama saat: 06:00 = 0.25 gün.
const FALLBACK_DAY_START: f64 = 0.25;

/// Panchang açısı.
#[derive(Clone, Copy)]
enum Angle {
    /// Ay − Güneş (tithi, karana).
    Elongation,
    /// Ay (nakshatra).
    Moon,
    /// Güneş + Ay (nitya yoga).
    Sum,
}

impl Angle {
    fn of(self, sun: f64, moon: f64) -> f64 {
        match self {
            Self::Elongation => elongation(sun, moon),
            Self::Moon => normalize(moon),
            Self::Sum => normalize(sun + moon),
        }
    }
}

/// Sidereal Güneş ve Ay boylamı.
fn sun_moon<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    zodiac: Zodiac,
) -> Result<(f64, f64), CoreError> {
    let sun = eph.body(jd_ut, Body::Sun, zodiac)?.lon;
    let moon = eph.body(jd_ut, Body::Moon, zodiac)?.lon;
    if sun.is_finite() && moon.is_finite() {
        Ok((sun, moon))
    } else {
        Err(CoreError::NonFiniteResult("panchang"))
    }
}

/// `angle`'ın `jd_ut`'den sonra `target` dereceye (0..=360) ulaştığı ilk an. İkiye bölme,
/// `[jd_ut, jd_ut + END_SEARCH_DAYS]`, `END_BISECTIONS` sabit adım; sonuç aralık ortası.
fn next_boundary<E: Ephemeris + ?Sized>(
    eph: &mut E,
    zodiac: Zodiac,
    jd_ut: f64,
    angle: Angle,
    target: f64,
    what: &'static str,
) -> Result<f64, CoreError> {
    // Hedefe işaretli uzaklık, [-180, 180): başta negatif, sınırdan sonra pozitif.
    let offset = |a: f64| normalize(a - target + 180.0) - 180.0;
    let mut lo = jd_ut;
    let mut hi = jd_ut + END_SEARCH_DAYS;
    let (s, m) = sun_moon(eph, hi, zodiac)?;
    if offset(angle.of(s, m)) < 0.0 {
        return Err(CoreError::PanchangSearch(what));
    }
    for _ in 0..END_BISECTIONS {
        let mid = 0.5 * (lo + hi);
        let (s, m) = sun_moon(eph, mid, zodiac)?;
        if offset(angle.of(s, m)) >= 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Ok(0.5 * (lo + hi))
}

/// Doğumdan önceki son gün doğumu, `[jd_ut − SUNRISE_WINDOW_DAYS, jd_ut]` içinde.
fn day_sunrise<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    geo_lat: f64,
    geo_lon: f64,
    disc: SunDisc,
) -> Result<Option<f64>, CoreError> {
    let mut found = None;
    // SE'nin yüksek enlem araması (|enlem| > 65°) başlangıçtan yalnızca ~26 saat ileriye
    // bakar (swecl.c `swe_rise_trans_true_hor`); 30 saatlik pencerenin son saatleri ikinci
    // başlangıçla taranır.
    for start in [jd_ut - SUNRISE_WINDOW_DAYS, jd_ut - SUNRISE_RETRY_DAYS] {
        let mut from = start;
        for _ in 0..MAX_SUNRISE_STEPS {
            match eph.sun_rise_set(from, RiseSet::Rise, geo_lat, geo_lon, disc)? {
                Some(t) if !t.is_finite() => return Err(CoreError::NonFiniteResult("sunrise")),
                Some(t) if t > from && t <= jd_ut => {
                    found = Some(t);
                    from = t + RISE_STEP_DAYS;
                }
                _ => break,
            }
        }
        if found.is_some() {
            break;
        }
    }
    Ok(found)
}

/// `sunrise`'tan sonraki ilk gün batımı; 1 gün içinde değilse `None`.
fn day_sunset<E: Ephemeris + ?Sized>(
    eph: &mut E,
    sunrise: f64,
    geo_lat: f64,
    geo_lon: f64,
    disc: SunDisc,
) -> Result<Option<f64>, CoreError> {
    match eph.sun_rise_set(sunrise, RiseSet::Set, geo_lat, geo_lon, disc)? {
        Some(t) if !t.is_finite() => Err(CoreError::NonFiniteResult("sunset")),
        Some(t) if t > sunrise && t - sunrise < MAX_DAYLIGHT_DAYS => Ok(Some(t)),
        _ => Ok(None),
    }
}

/// Doğum anının panchangı (spec §5). Güneş ve Ay her zodyak kipinde sidereal sorulur.
/// Gün doğumu `settings.sunrise_def` tanımıyla, doğum yerinde (`geo_lat` kuzey +, `geo_lon`
/// doğu +) ve deniz seviyesinde hesaplanır.
pub fn compute_panchang<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    geo_lat: f64,
    geo_lon: f64,
    settings: &CalcSettings,
) -> Result<Panchang, CoreError> {
    settings.validate()?;
    check_moment(jd_ut, geo_lat, geo_lon)?;
    let zodiac = Zodiac::Sidereal(settings.ayanamsa.sidereal_mode());
    let disc = settings.sunrise_def.sun_disc();

    let sunrise_jd_ut = day_sunrise(eph, jd_ut, geo_lat, geo_lon, disc)?;
    let sunset_jd_ut = match sunrise_jd_ut {
        Some(r) => day_sunset(eph, r, geo_lat, geo_lon, disc)?,
        None => None,
    };
    // Vara gün doğumunda değişir; gün doğumu yoksa gün 06:00 yerel ortalama saatte başlar.
    let vara = match sunrise_jd_ut {
        Some(r) => local_weekday(r, geo_lon),
        None => local_weekday(jd_ut - FALLBACK_DAY_START, geo_lon),
    };

    let (sun, moon) = sun_moon(eph, jd_ut, zodiac)?;
    let e = elongation(sun, moon);

    let t_no = tithi_no(e);
    let tithi = Tithi {
        no: t_no,
        paksha: paksha(t_no),
        ends_jd_ut: next_boundary(
            eph,
            zodiac,
            jd_ut,
            Angle::Elongation,
            f64::from(t_no) * TITHI_SPAN_DEG,
            "tithi",
        )?,
    };
    let (n_no, pada) = nakshatra_pada(moon);
    let nakshatra = PanchangNakshatra {
        no: n_no,
        pada,
        ends_jd_ut: next_boundary(
            eph,
            zodiac,
            jd_ut,
            Angle::Moon,
            f64::from(n_no) * NAKSHATRA_SPAN_DEG,
            "nakshatra",
        )?,
    };
    let y_no = nitya_yoga_no(sun, moon);
    let yoga = NityaYoga {
        no: y_no,
        ends_jd_ut: next_boundary(
            eph,
            zodiac,
            jd_ut,
            Angle::Sum,
            f64::from(y_no) * NAKSHATRA_SPAN_DEG,
            "yoga",
        )?,
    };
    let k_no = karana_no(e);
    let karana = Karana {
        no: k_no,
        name: karana_name(k_no),
        ends_jd_ut: next_boundary(
            eph,
            zodiac,
            jd_ut,
            Angle::Elongation,
            f64::from(k_no) * KARANA_SPAN_DEG,
            "karana",
        )?,
    };
    Ok(Panchang {
        tithi,
        vara,
        nakshatra,
        yoga,
        karana,
        sunrise_jd_ut,
        sunset_jd_ut,
    })
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

#[cfg(test)]
mod compute_tests {
    use ephemeris::{Body, SunDisc};

    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::SunriseDef;

    const JD: f64 = 2_451_545.0;
    /// İkiye bölme 1.5 gün / 2^20 ≈ 0.12 sn verir; 0.2 sn pay.
    const TOL_DAYS: f64 = 0.2 / 86_400.0;

    fn panchang(eph: &mut FakeEph, jd: f64) -> Panchang {
        compute_panchang(eph, jd, 41.0, 29.0, &CalcSettings::default()).unwrap()
    }

    #[test]
    fn fake_panchang_follows_linear_motion() {
        // Sahte sidereal (tropikal − 24°): Güneş 346° + 0.98°/gün, Ay 71° + 13.2°/gün.
        let p = panchang(&mut FakeEph::default(), JD);
        // Uzanım 85°: 8. tithi (Shukla Ashtami), 96°'de biter; uzanım hızı 12.22°/gün.
        assert_eq!((p.tithi.no, p.tithi.paksha), (8, Paksha::Shukla));
        assert!((p.tithi.ends_jd_ut - (JD + 11.0 / 12.22)).abs() < TOL_DAYS);
        // Ay 71°: Ardra (6.), 2. pada; 80°'de biter.
        assert_eq!((p.nakshatra.no, p.nakshatra.pada), (6, 2));
        assert!((p.nakshatra.ends_jd_ut - (JD + 9.0 / 13.2)).abs() < TOL_DAYS);
        // Toplam 417° → 57°: 5. yoga (Shobhana), 66°40′'ta biter; toplam hız 14.18°/gün.
        assert_eq!(p.yoga.no, 5);
        assert!((p.yoga.ends_jd_ut - (JD + (200.0 / 3.0 - 57.0) / 14.18)).abs() < TOL_DAYS);
        // 85° / 6 = 14.17: 15. karana, (15 − 2) mod 7 = 6 → Vishti; 90°'de biter.
        assert_eq!((p.karana.no, p.karana.name), (15, KaranaName::Vishti));
        assert!((p.karana.ends_jd_ut - (JD + 5.0 / 12.22)).abs() < TOL_DAYS);
        // Sahte gün doğumu her gün n + 0.67 (04:05 UT), batımı n + 0.15 (15:36 UT).
        assert!((p.sunrise_jd_ut.unwrap() - 2_451_544.67).abs() < 1e-9);
        assert!((p.sunset_jd_ut.unwrap() - 2_451_545.15).abs() < 1e-9);
        // 2451544.67 + 29°/360 → yerel ortalama 1 Ocak 2000: Cumartesi.
        assert_eq!(p.vara, Vara::Saturday);
    }

    #[test]
    fn birth_before_sunrise_belongs_to_the_previous_vara() {
        // 1 Ocak 2000 02:24 UT: o günün sahte gün doğumu (04:05 UT) henüz olmadı.
        let p = panchang(&mut FakeEph::default(), 2_451_544.6);
        assert!((p.sunrise_jd_ut.unwrap() - 2_451_543.67).abs() < 1e-9);
        assert_eq!(p.vara, Vara::Friday);
        // 04:19 UT: gün doğumundan sonra.
        let p = panchang(&mut FakeEph::default(), 2_451_544.68);
        assert!((p.sunrise_jd_ut.unwrap() - 2_451_544.67).abs() < 1e-9);
        assert_eq!(p.vara, Vara::Saturday);
    }

    #[test]
    fn polar_day_starts_at_six_local_mean_time() {
        let polar = || FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        let p = panchang(&mut polar(), JD);
        assert_eq!((p.sunrise_jd_ut, p.sunset_jd_ut), (None, None));
        // 12:00 UT, boylam 29° → 13:56 yerel ortalama saat: 1 Ocak, Cumartesi.
        assert_eq!(p.vara, Vara::Saturday);
        // 02:24 UT → 04:20 yerel ortalama saat, 06:00'dan önce: önceki gün, Cuma.
        assert_eq!(panchang(&mut polar(), 2_451_544.6).vara, Vara::Friday);
    }

    #[test]
    fn sunrise_definition_selects_the_sun_disc() {
        let mut eph = FakeEph::default();
        panchang(&mut eph, JD);
        assert_eq!(eph.last_sun_disc, Some(SunDisc::CenterNoRefraction));
        let s = CalcSettings {
            sunrise_def: SunriseDef::UpperLimbRefracted,
            ..CalcSettings::default()
        };
        compute_panchang(&mut eph, JD, 41.0, 29.0, &s).unwrap();
        assert_eq!(eph.last_sun_disc, Some(SunDisc::UpperLimbRefracted));
    }

    #[test]
    fn invalid_input_and_a_stalled_moon_are_errors() {
        let s = CalcSettings::default();
        assert_eq!(
            compute_panchang(&mut FakeEph::default(), JD, 91.0, 0.0, &s),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 0.0
            })
        );
        assert_eq!(
            compute_panchang(&mut FakeEph::default(), 1e300, 0.0, 0.0, &s),
            Err(CoreError::InvalidJulianDay(1e300))
        );
        // Ay Güneş hızında: uzanım ilerlemez, sınır bulunamaz (gerçek ephemeris'te olmaz).
        let mut eph = FakeEph::default().with(Body::Moon, 95.0, 0.98);
        assert_eq!(
            compute_panchang(&mut eph, JD, 41.0, 29.0, &s),
            Err(CoreError::PanchangSearch("tithi"))
        );
    }
}
