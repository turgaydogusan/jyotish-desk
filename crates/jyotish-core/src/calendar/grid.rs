//! Takvim günleri ve günün gün doğumu olguları (Plan 29 D3–D5).
//!
//! Hücre takvim günüdür (yerel gece yarısından gece yarısına); panchang günü ise gün doğumunda
//! başlar. İkisi şöyle bağlanır: hücrenin tithisi ve Ay nakshatrası, o takvim gününün içindeki
//! gün doğumunda geçerli olan değerdir (o gün başlayan panchang gününün ilk değeri). Gün içinde
//! değişen tithi hücrede yazılmaz; gün panchangı paneli bütün değişimleri saatleriyle verir.

use ephemeris::{Body, Ephemeris, RiseSet};
use serde::{Deserialize, Serialize};

use crate::panchang::{elongation, paksha, tithi_no, Paksha};
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::timeline::{lon_of, sidereal, TIMELINE_RESOLUTION_DAYS};
use crate::zodiac::nakshatra_pada;
use crate::CoreError;

/// Bir istekteki en çok takvim günü: bir ay.
pub const CALENDAR_MAX_DAYS: usize = 31;
/// Bir motor işinde hesaplanan en çok gün: çağıran dilimler arasında motor iş parçacığını
/// bırakır.
pub const CALENDAR_SLICE_DAYS: usize = 7;
/// Bir takvim gününün en uzun süresi, gün: 24 saat ve saat dilimi geçişi için 6 saat pay.
pub const MAX_DAY_SPAN_DAYS: f64 = 1.25;
/// Gün doğumu yoksa olguların okunduğu yerel ortalama saat: 06:00 = 0,25 gün (v1 kuralı).
const FALLBACK_DAY_START: f64 = 0.25;

/// Bir takvim günü: `[start_jd_ut, end_jd_ut)`. Uçlar yerin duvar saatiyle gece yarılarıdır.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DaySpan {
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
}

impl DaySpan {
    pub fn contains(&self, jd_ut: f64) -> bool {
        self.start_jd_ut <= jd_ut && jd_ut < self.end_jd_ut
    }
}

/// Bir takvim gününün gün doğumu olguları.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DayFacts {
    /// Olguların okunduğu an: günün içindeki gün doğumu; yoksa 06:00 yerel ortalama saat.
    pub jd_ut: f64,
    /// Günün içindeki (ilk) gün doğumu; o takvim gününde Güneş doğmuyorsa `None`.
    pub sunrise_jd_ut: Option<f64>,
    /// 1..=30.
    pub tithi: u8,
    pub paksha: Paksha,
    /// Ay'ın nakshatrası, 1..=27 (sidereal boylamdan, her zodyak kipinde).
    pub nakshatra: u8,
}

fn invalid(field: &'static str) -> CoreError {
    CoreError::InvalidSearchInput(field)
}

pub(crate) fn check_place(geo_lat: f64, geo_lon: f64) -> Result<(), CoreError> {
    if (-90.0..=90.0).contains(&geo_lat) && (-180.0..=180.0).contains(&geo_lon) {
        Ok(())
    } else {
        Err(invalid("place"))
    }
}

/// Gün listesinin denetimi: 1–31 gün, her gün hesap aralığında ve en çok `MAX_DAY_SPAN_DAYS`
/// uzunluğunda, günler boşluksuz ardışık. Hata yalnızca alan adını taşır (`range`).
pub fn check_spans(spans: &[DaySpan]) -> Result<(), CoreError> {
    if spans.is_empty() || spans.len() > CALENDAR_MAX_DAYS {
        return Err(invalid("range"));
    }
    for s in spans {
        // NaN ve sonsuz da aralık dışındadır.
        let in_range = JD_UT_RANGE.contains(&s.start_jd_ut) && JD_UT_RANGE.contains(&s.end_jd_ut);
        let days = s.end_jd_ut - s.start_jd_ut;
        if !in_range || days <= 0.0 || days > MAX_DAY_SPAN_DAYS {
            return Err(invalid("range"));
        }
    }
    if spans.windows(2).any(|w| w[0].end_jd_ut != w[1].start_jd_ut) {
        return Err(invalid("range"));
    }
    Ok(())
}

/// `n` günün dilimleri, sırayla: `[from, to)` gün sıraları, her biri en çok
/// `CALENDAR_SLICE_DAYS` gün.
pub fn calendar_slices(n: usize) -> Vec<(usize, usize)> {
    (0..n)
        .step_by(CALENDAR_SLICE_DAYS)
        .map(|from| (from, (from + CALENDAR_SLICE_DAYS).min(n)))
        .collect()
}

/// Takvim gününün içindeki ilk gün doğumu; o gün Güneş doğmuyorsa `None`.
pub fn sunrise_in<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    geo_lat: f64,
    geo_lon: f64,
    span: DaySpan,
) -> Result<Option<f64>, CoreError> {
    let disc = settings.sunrise_def.sun_disc();
    // Arama günün başından hemen önce başlar: tam gece yarısındaki doğuş o güne aittir.
    let from = span.start_jd_ut - TIMELINE_RESOLUTION_DAYS;
    match eph.sun_rise_set(from, RiseSet::Rise, geo_lat, geo_lon, disc)? {
        Some(t) if !t.is_finite() => Err(CoreError::NonFiniteResult("sunrise")),
        Some(t) if span.contains(t) => Ok(Some(t)),
        _ => Ok(None),
    }
}

/// Gün doğumu olmayan günde olguların okunduğu an: günün ortasının düştüğü yerel ortalama
/// tarihte 06:00 (gün panchangının kuralı), günün içine sıkıştırılmış.
pub fn fallback_moment(span: DaySpan, geo_lon: f64) -> f64 {
    let mid = 0.5 * (span.start_jd_ut + span.end_jd_ut);
    // Yerel gün sayısı `jd + 0,5 + boylam / 360`'tır.
    let local = mid + 0.5 + geo_lon / 360.0;
    let six = local.floor() - 0.5 - geo_lon / 360.0 + FALLBACK_DAY_START;
    six.clamp(span.start_jd_ut, span.end_jd_ut - TIMELINE_RESOLUTION_DAYS)
}

/// Bir takvim gününün gün doğumu olguları: tithi ve Ay nakshatrası, günün içindeki gün
/// doğumunda; gün doğumu yoksa `fallback_moment` anında. Güneş ve Ay her zodyak kipinde
/// sidereal sorulur. Konum denetimi alan adıyla döner (`place`); konum hata parametresine girmez.
pub fn day_facts<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    geo_lat: f64,
    geo_lon: f64,
    span: DaySpan,
) -> Result<DayFacts, CoreError> {
    check_place(geo_lat, geo_lon)?;
    let sunrise_jd_ut = sunrise_in(eph, settings, geo_lat, geo_lon, span)?;
    let jd_ut = sunrise_jd_ut.unwrap_or_else(|| fallback_moment(span, geo_lon));
    let zodiac = sidereal(settings);
    let sun = lon_of(eph, jd_ut, Body::Sun, zodiac)?;
    let moon = lon_of(eph, jd_ut, Body::Moon, zodiac)?;
    let tithi = tithi_no(elongation(sun, moon));
    Ok(DayFacts {
        jd_ut,
        sunrise_jd_ut,
        tithi,
        paksha: paksha(tithi),
        nakshatra: nakshatra_pada(moon).0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::panchang::Paksha;
    use crate::settings::CalcSettings;
    use crate::CoreError;
    use ephemeris::Body;

    /// 2000-01-01 00:00 UT.
    const MIDNIGHT: f64 = 2_451_544.5;
    const SECOND: f64 = 1.0 / 86_400.0;

    /// `n` ardışık UT günü.
    fn days(n: usize) -> Vec<DaySpan> {
        (0..n)
            .map(|i| DaySpan {
                start_jd_ut: MIDNIGHT + i as f64,
                end_jd_ut: MIDNIGHT + (i + 1) as f64,
            })
            .collect()
    }

    fn facts(eph: &mut FakeEph, span: DaySpan, lon: f64) -> DayFacts {
        day_facts(eph, &CalcSettings::default(), 28.6, lon, span).unwrap()
    }

    fn field(r: Result<(), CoreError>) -> &'static str {
        match r {
            Err(CoreError::InvalidSearchInput(f)) => f,
            other => panic!("expected an invalid search input, got {other:?}"),
        }
    }

    /// Sahte ephemeris: gün doğumu her gün `n + 0,67` anındadır (04:05 UT); sidereal Güneş
    /// J2000'de 346° (0,98°/gün), Ay 71° (13,2°/gün).
    #[test]
    fn the_facts_of_a_day_are_read_at_the_sunrise_inside_it() {
        let mut eph = FakeEph::default();
        let spans = days(3);
        // 1. gün: doğuş 2451544,67 (J2000'den 0,33 gün önce). Güneş 346 − 0,3234 = 345,6766°;
        // Ay 71 − 4,356 = 66,644°. Uzanım 80,9674°: 72°–84° dilimi 7. tithidir (Shukla).
        // Ay 66,644° < 66°40′ (= 66,667°): 5. nakshatranın son yayındadır.
        let first = facts(&mut eph, spans[0], 77.2);
        assert_eq!(first.sunrise_jd_ut, Some(2_451_544.67));
        assert_eq!(first.jd_ut, 2_451_544.67);
        assert_eq!(
            (first.tithi, first.paksha, first.nakshatra),
            (7, Paksha::Shukla, 5)
        );
        // 2. gün: doğuş J2000'den 0,67 gün sonra. Güneş 346,6566°, Ay 79,844°. Uzanım
        // 93,1874°: 84°–96° dilimi 8. tithidir. Ay 66°40′–80° arasında: 6. nakshatra.
        let second = facts(&mut eph, spans[1], 77.2);
        assert_eq!(second.sunrise_jd_ut, Some(2_451_545.67));
        assert_eq!((second.tithi, second.nakshatra), (8, 6));
        // 3. gün: Güneş 347,6366°, Ay 93,044°. Uzanım 105,4074°: 96°–108° dilimi 9. tithidir.
        // Ay 80°–93°20′ dilimini 0,29 gün önce geçmiştir: 7. nakshatra.
        let third = facts(&mut eph, spans[2], 77.2);
        assert_eq!((third.tithi, third.nakshatra), (9, 7));
        // Disk tanımı hesap ayarından gelir.
        assert_eq!(
            eph.last_sun_disc,
            Some(CalcSettings::default().sunrise_def.sun_disc())
        );
    }

    #[test]
    fn the_waning_half_starts_after_the_fifteenth_tithi() {
        // Ay'ı Güneş'in 200° önüne koy: uzanım 200° − (0,33 × 12,22) ≈ 195,97°; 192°–204°
        // dilimi 17. tithidir (Krishna 2).
        let mut eph = FakeEph::default().with(Body::Moon, 210.0, 13.2);
        let got = facts(&mut eph, days(1)[0], 77.2);
        assert_eq!((got.tithi, got.paksha), (17, Paksha::Krishna));
    }

    #[test]
    fn a_sunrise_belongs_to_the_day_that_contains_it_and_midnight_opens_the_day() {
        let settings = CalcSettings::default();
        let span = days(1)[0];
        // Tam gece yarısındaki doğuş o güne aittir; günün bitişindeki doğuş sonraki güne.
        for (rise, want) in [
            (span.start_jd_ut, Some(span.start_jd_ut)),
            (span.end_jd_ut - SECOND, Some(span.end_jd_ut - SECOND)),
            (span.end_jd_ut, None),
            (span.start_jd_ut - SECOND, None),
        ] {
            let mut eph = FakeEph {
                sunrises: Some(vec![rise]),
                ..FakeEph::default()
            };
            assert_eq!(
                sunrise_in(&mut eph, &settings, 69.6, 18.9, span).unwrap(),
                want
            );
        }
        // İki doğuşlu günde ilki alınır.
        let mut eph = FakeEph {
            sunrises: Some(vec![span.start_jd_ut + 0.01, span.start_jd_ut + 0.99]),
            ..FakeEph::default()
        };
        assert_eq!(
            sunrise_in(&mut eph, &settings, 69.6, 18.9, span).unwrap(),
            Some(span.start_jd_ut + 0.01)
        );
    }

    #[test]
    fn a_day_without_a_sunrise_is_read_at_six_local_mean_time() {
        let mut eph = FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        let span = days(1)[0];
        // Boylam 30° D: yerel ortalama saat UT + 2 saattir; 06:00 yerel 04:00 UT'dir.
        let got = facts(&mut eph, span, 30.0);
        assert_eq!(got.sunrise_jd_ut, None);
        assert!((got.jd_ut - (span.start_jd_ut + 4.0 / 24.0)).abs() < SECOND);
        // Güneş 346 − 0,98 / 3 = 345,6733°; Ay 71 − 4,4 = 66,6°: uzanım 80,93° (7. tithi),
        // 5. nakshatra.
        assert_eq!((got.tithi, got.nakshatra), (7, 5));
        // Boylam 0°: 06:00 UT. Batı boylamında (−90°: UT − 6 saat) 06:00 yerel 12:00 UT'dir.
        assert!((fallback_moment(span, 0.0) - (span.start_jd_ut + 0.25)).abs() < SECOND);
        assert!((fallback_moment(span, -90.0) - (span.start_jd_ut + 0.5)).abs() < SECOND);
        // An her durumda günün içindedir (duvar saati yerel ortalama saatten çok uzak olsa da).
        for lon in [-180.0, -135.0, 135.0, 180.0] {
            assert!(span.contains(fallback_moment(span, lon)), "{lon}");
        }
    }

    #[test]
    fn spans_are_one_to_thirty_one_gapless_days_of_plausible_length() {
        assert!(check_spans(&days(1)).is_ok());
        assert!(check_spans(&days(31)).is_ok());
        assert_eq!(field(check_spans(&[])), "range");
        assert_eq!(field(check_spans(&days(32))), "range");
        // 23 ve 25 saatlik günler (yaz saati geçişi) geçerlidir; 31 saatlik gün değildir.
        let day = |start: f64, hours: f64| DaySpan {
            start_jd_ut: start,
            end_jd_ut: start + hours / 24.0,
        };
        assert!(check_spans(&[day(MIDNIGHT, 23.0)]).is_ok());
        assert!(check_spans(&[day(MIDNIGHT, 25.0)]).is_ok());
        assert_eq!(field(check_spans(&[day(MIDNIGHT, 31.0)])), "range");
        assert_eq!(field(check_spans(&[day(MIDNIGHT, 0.0)])), "range");
        assert_eq!(field(check_spans(&[day(MIDNIGHT, -1.0)])), "range");
        // Günler boşluksuz ardışıktır.
        let mut gap = days(3);
        gap[2].start_jd_ut += SECOND;
        assert_eq!(field(check_spans(&gap)), "range");
        // Hesap aralığının dışı ve sonlu olmayan uçlar.
        assert_eq!(field(check_spans(&[day(600_000.0, 24.0)])), "range");
        assert_eq!(field(check_spans(&[day(f64::NAN, 24.0)])), "range");
        assert_eq!(field(check_spans(&[day(f64::INFINITY, 24.0)])), "range");
        // Yer denetimi alan adıyla döner.
        let bad = day_facts(
            &mut FakeEph::default(),
            &CalcSettings::default(),
            91.0,
            0.0,
            days(1)[0],
        );
        assert_eq!(field(bad.map(|_| ())), "place");
        assert_eq!(field(check_place(91.0, 0.0)), "place");
        assert_eq!(field(check_place(0.0, f64::NAN)), "place");
        assert!(check_place(-90.0, 180.0).is_ok());
    }

    #[test]
    fn a_month_is_cut_into_slices_of_at_most_seven_days() {
        assert_eq!(
            calendar_slices(31),
            [(0, 7), (7, 14), (14, 21), (21, 28), (28, 31)]
        );
        assert_eq!(calendar_slices(28), [(0, 7), (7, 14), (14, 21), (21, 28)]);
        assert_eq!(calendar_slices(1), [(0, 1)]);
        assert!(calendar_slices(0).is_empty());
    }
}
