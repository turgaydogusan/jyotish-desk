//! Gün panchangı (ileri modüller spec'i §10): verilen anı içeren gün, gün doğumundan sonraki
//! gün doğumuna. Beş uzvun gün içindeki bütün değerleri ve değişim anları, günün dilimleri,
//! işaretler ve (doğum Ay'ı verilmişse) tara ve chandra bala.
//!
//! Gün, `compute_panchang`'ın kuralıyla seçilir: andan önceki son gün doğumunda başlar. Doğuşu,
//! batışı ya da sonraki doğuşu olmayan günde (kutup gecesi ya da gündüzü) gün 06:00 yerel
//! ortalama saatte başlar ve 24 saat sürer; dilimler hesaplanmaz, uzuvlar yine verilir.

use ephemeris::{Body, Ephemeris, Precision};
use serde::{Deserialize, Serialize};

use crate::chart::{rasi_frame, ChartWarning};
use crate::graha::Graha;
use crate::panchang::{
    day_sunrise, day_sunset, following_sunrise, karana_name, local_weekday, paksha, vara_at,
    KaranaName, Paksha, Vara, SUNRISE_WINDOW_DAYS,
};
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::timeline::{
    combust_series, graha_sign_series, karana_series, map_merge, nakshatra_series,
    nitya_yoga_series, tithis_of_karanas, Interval, SunDay,
};
use crate::CoreError;

use super::segments::{
    chandra_bala, day_segments, is_panchaka, is_rikta, is_vishti, nakshatra_nature, tara_bala,
    tithi_group, ChandraBala, DaySegments, NakshatraNature, TaraBala, TithiGroup, VAIDHRITI_YOGA,
    VYATIPATA_YOGA,
};
use super::{MuhurtaFlag, NatalMoon, MUHURTA_METHOD};

/// Gün doğumu yoksa günün başladığı yerel ortalama saat: 06:00 = 0,25 gün (v1 kuralı).
const FALLBACK_DAY_START: f64 = 0.25;

/// Gün panchangının `flags` listesinde aralık olarak verilen işaretler, bu sırayla. Üç gündüz
/// dilimi (`RahuSegment`, `Yamaganda`, `Gulika`) `segments` içindedir.
pub const DAY_FLAG_KINDS: [MuhurtaFlag; 7] = [
    MuhurtaFlag::Rikta,
    MuhurtaFlag::Vishti,
    MuhurtaFlag::Vyatipata,
    MuhurtaFlag::Vaidhriti,
    MuhurtaFlag::Panchaka,
    MuhurtaFlag::JupiterCombust,
    MuhurtaFlag::VenusCombust,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DayTithi {
    /// 1..=30.
    pub no: u8,
    pub paksha: Paksha,
    pub group: TithiGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DayNakshatra {
    /// Ay'ın nakshatrası, 1..=27.
    pub no: u8,
    /// 1..=4.
    pub pada: u8,
    pub nature: NakshatraNature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DayKarana {
    /// 1..=60.
    pub no: u8,
    pub name: KaranaName,
}

/// Kişiye bağlı etkenler, gün içindeki aralıklarıyla: Ay gün içinde nakshatra ya da burç
/// değiştirirse değer de değişir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PersonalFactors {
    pub tara: Vec<Interval<TaraBala>>,
    pub chandra: Vec<Interval<ChandraBala>>,
}

/// Bir günün panchangı. Bütün aralık dizileri `[day_start_jd_ut, day_end_jd_ut]`'ye kırpılmıştır
/// ve onu boşluksuz böler.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DayPanchang {
    /// Günün başı: gün doğumu; yoksa 06:00 yerel ortalama saat.
    pub day_start_jd_ut: f64,
    /// Günün sonu: sonraki gün doğumu; yoksa başından 24 saat sonrası.
    pub day_end_jd_ut: f64,
    /// Üçü birlikte dolu ya da birlikte boştur.
    pub sunrise_jd_ut: Option<f64>,
    pub sunset_jd_ut: Option<f64>,
    pub next_sunrise_jd_ut: Option<f64>,
    pub vara: Vara,
    pub tithis: Vec<Interval<DayTithi>>,
    pub nakshatras: Vec<Interval<DayNakshatra>>,
    pub yogas: Vec<Interval<u8>>,
    pub karanas: Vec<Interval<DayKarana>>,
    /// Doğuşu ve batışı olmayan günde `None`.
    pub segments: Option<DaySegments>,
    /// `DAY_FLAG_KINDS` sırasında, her işaretin gün içindeki aralıkları.
    pub flags: Vec<Interval<MuhurtaFlag>>,
    /// Doğum Ay'ı verilmediyse `None`.
    pub personal: Option<PersonalFactors>,
    /// `SunriseUnavailable` ya da `LowPrecision`.
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

/// `jd_ut` anını içeren, doğuşu, batışı ve sonraki doğuşu bilinen gün; yoksa `None`.
pub(crate) fn sun_day_at<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    jd_ut: f64,
    geo_lat: f64,
    geo_lon: f64,
) -> Result<Option<SunDay>, CoreError> {
    let disc = settings.sunrise_def.sun_disc();
    let Some(sunrise) = day_sunrise(eph, jd_ut, geo_lat, geo_lon, disc)? else {
        return Ok(None);
    };
    let Some(sunset) = day_sunset(eph, sunrise, geo_lat, geo_lon, disc)? else {
        return Ok(None);
    };
    let Some(next) = following_sunrise(eph, sunrise, geo_lat, geo_lon, disc)? else {
        return Ok(None);
    };
    if next <= jd_ut || next - sunrise > SUNRISE_WINDOW_DAYS || sunset >= next {
        return Ok(None);
    }
    Ok(Some(SunDay {
        sunrise_jd_ut: sunrise,
        sunset_jd_ut: sunset,
        next_sunrise_jd_ut: next,
        vara: local_weekday(sunrise, geo_lon),
    }))
}

/// `value`'ların işaret taşıyan aralıkları, birleştirilmiş.
fn flagged<T>(
    series: &[Interval<T>],
    flag: MuhurtaFlag,
    holds: impl Fn(&T) -> bool,
) -> Vec<Interval<MuhurtaFlag>> {
    map_merge(series, |v| holds(v))
        .into_iter()
        .filter(|i| i.value)
        .map(|i| Interval {
            value: flag,
            start_jd_ut: i.start_jd_ut,
            end_jd_ut: i.end_jd_ut,
        })
        .collect()
}

/// `jd_ut` anını içeren günün panchangı. Konum denetimi alan adıyla döner
/// (`InvalidMuhurtaInput("place")`); konum hata parametresine girmez (bulunulan yer kişisel
/// veridir). `natal` verilirse tara ve chandra bala eklenir.
pub fn day_panchang<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    geo_lat: f64,
    geo_lon: f64,
    settings: &CalcSettings,
    natal: Option<NatalMoon>,
) -> Result<DayPanchang, CoreError> {
    settings.validate()?;
    // NaN ve sonsuz da aralık dışındadır.
    if !JD_UT_RANGE.contains(&jd_ut) {
        return Err(CoreError::InvalidJulianDay(jd_ut));
    }
    if !((-90.0..=90.0).contains(&geo_lat) && (-180.0..=180.0).contains(&geo_lon)) {
        return Err(CoreError::InvalidMuhurtaInput("place"));
    }
    if let Some(n) = &natal {
        n.validate()?;
    }
    let mut warnings = Vec::new();
    let day = sun_day_at(eph, settings, jd_ut, geo_lat, geo_lon)?;
    let (start, end, vara) = match &day {
        Some(d) => (d.sunrise_jd_ut, d.next_sunrise_jd_ut, d.vara),
        None => {
            warnings.push(ChartWarning::SunriseUnavailable);
            // 06:00 yerel ortalama saat: yerel gün sayısı `jd + 0,5 + boylam / 360`'tır.
            let local = jd_ut + 0.5 + geo_lon / 360.0 - FALLBACK_DAY_START;
            let start = local.floor() - 0.5 - geo_lon / 360.0 + FALLBACK_DAY_START;
            let disc = settings.sunrise_def.sun_disc();
            (
                start,
                start + 1.0,
                vara_at(eph, jd_ut, geo_lat, geo_lon, disc)?,
            )
        }
    };
    if !(JD_UT_RANGE.contains(&start) && JD_UT_RANGE.contains(&end)) {
        return Err(CoreError::InvalidJulianDay(jd_ut));
    }
    // Hassasiyet Güneş'ten okunur (`compute_chart` ile aynı kural).
    if eph.body(start, Body::Sun, rasi_frame(settings))?.precision == Precision::Moshier {
        warnings.push(ChartWarning::LowPrecision);
    }

    let karanas = karana_series(eph, settings, start, end)?;
    let tithis = tithis_of_karanas(&karanas);
    let nakshatras = nakshatra_series(eph, settings, start, end)?;
    let yogas = nitya_yoga_series(eph, settings, start, end)?;
    let jupiter = combust_series(eph, settings, Graha::Jupiter, start, end)?;
    let venus = combust_series(eph, settings, Graha::Venus, start, end)?;

    let mut flags = flagged(&tithis, MuhurtaFlag::Rikta, |t| is_rikta(*t));
    flags.extend(flagged(&karanas, MuhurtaFlag::Vishti, |k| is_vishti(*k)));
    flags.extend(flagged(&yogas, MuhurtaFlag::Vyatipata, |y| {
        *y == VYATIPATA_YOGA
    }));
    flags.extend(flagged(&yogas, MuhurtaFlag::Vaidhriti, |y| {
        *y == VAIDHRITI_YOGA
    }));
    flags.extend(flagged(&nakshatras, MuhurtaFlag::Panchaka, |n| {
        is_panchaka(*n)
    }));
    flags.extend(flagged(&jupiter, MuhurtaFlag::JupiterCombust, |c| *c));
    flags.extend(flagged(&venus, MuhurtaFlag::VenusCombust, |c| *c));

    let personal = match natal {
        Some(natal) => {
            let moon_signs = graha_sign_series(eph, settings, Graha::Moon, start, end)?;
            Some(PersonalFactors {
                tara: map_merge(&nakshatras, |n| tara_bala(natal.nakshatra, n.nakshatra)),
                chandra: map_merge(&moon_signs, |s| chandra_bala(natal.sign, *s)),
            })
        }
        None => None,
    };

    Ok(DayPanchang {
        day_start_jd_ut: start,
        day_end_jd_ut: end,
        sunrise_jd_ut: day.map(|d| d.sunrise_jd_ut),
        sunset_jd_ut: day.map(|d| d.sunset_jd_ut),
        next_sunrise_jd_ut: day.map(|d| d.next_sunrise_jd_ut),
        vara,
        tithis: map_merge(&tithis, |t| DayTithi {
            no: *t,
            paksha: paksha(*t),
            group: tithi_group(*t),
        }),
        nakshatras: map_merge(&nakshatras, |n| DayNakshatra {
            no: n.nakshatra,
            pada: n.pada,
            nature: nakshatra_nature(n.nakshatra),
        }),
        yogas,
        karanas: map_merge(&karanas, |k| DayKarana {
            no: *k,
            name: karana_name(*k),
        }),
        segments: day.as_ref().map(day_segments),
        flags,
        personal,
        warnings,
        method: MUHURTA_METHOD.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use ephemeris::Precision;

    use super::*;
    use crate::fake::FakeEph;
    use crate::panchang::compute_panchang;
    use crate::zodiac::Sign;

    const JD: f64 = 2_451_545.0;
    const TOL: f64 = 0.2 / 86_400.0;

    fn day(eph: &mut FakeEph, jd: f64, natal: Option<NatalMoon>) -> DayPanchang {
        day_panchang(eph, jd, 41.0, 29.0, &CalcSettings::default(), natal).unwrap()
    }

    fn partitions<T>(s: &[Interval<T>], d: &DayPanchang) {
        assert_eq!(s.first().unwrap().start_jd_ut, d.day_start_jd_ut);
        assert_eq!(s.last().unwrap().end_jd_ut, d.day_end_jd_ut);
        for w in s.windows(2) {
            assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
        }
    }

    #[test]
    fn the_day_runs_from_the_last_sunrise_to_the_next_one() {
        // Sahte gün doğumu n + 0,67, batımı n + 0,15. 12:00 UT'yi içeren gün 2451544,67'de
        // başlar (yerel ortalama 1 Ocak 2000, Cumartesi).
        let mut eph = FakeEph::default();
        let d = day(&mut eph, JD, None);
        assert!((d.day_start_jd_ut - 2_451_544.67).abs() < 1e-9);
        assert!((d.day_end_jd_ut - 2_451_545.67).abs() < 1e-9);
        assert_eq!(d.sunrise_jd_ut, Some(d.day_start_jd_ut));
        assert_eq!(d.next_sunrise_jd_ut, Some(d.day_end_jd_ut));
        assert!((d.sunset_jd_ut.unwrap() - 2_451_545.15).abs() < 1e-9);
        assert_eq!(d.vara, Vara::Saturday);
        assert!(d.warnings.is_empty());
        assert_eq!(d.method, MUHURTA_METHOD);
        assert_eq!(d.personal, None);
        partitions(&d.tithis, &d);
        partitions(&d.nakshatras, &d);
        partitions(&d.yogas, &d);
        partitions(&d.karanas, &d);
        // Gün doğumunda uzanım 85° − 0,33 × 12,22° = 80,97°: 7. tithi (Bhadra); 84°'de 8.
        // (Jaya). Gün 93,19°'de biter.
        assert_eq!(
            d.tithis
                .iter()
                .map(|t| (t.value.no, t.value.group))
                .collect::<Vec<_>>(),
            [(7, TithiGroup::Bhadra), (8, TithiGroup::Jaya)]
        );
        assert!(d.tithis.iter().all(|t| t.value.paksha == Paksha::Shukla));
        assert!((d.tithis[1].start_jd_ut - (JD - 1.0 / 12.22)).abs() < TOL);
        // Karana: 14 (Vanija), 84°'de 15 (Vishti), 90°'de 16 (Bava).
        assert_eq!(
            d.karanas
                .iter()
                .map(|k| (k.value.no, k.value.name))
                .collect::<Vec<_>>(),
            [
                (14, KaranaName::Vanija),
                (15, KaranaName::Vishti),
                (16, KaranaName::Bava)
            ]
        );
        // Ay gün doğumunda 66,64°: Mrigashira'nın son padası (yumuşak), sonra Ardra (keskin).
        assert_eq!(
            (d.nakshatras[0].value.no, d.nakshatras[0].value.pada),
            (5, 4)
        );
        assert_eq!(d.nakshatras[0].value.nature, NakshatraNature::Soft);
        assert_eq!(d.nakshatras[1].value.nature, NakshatraNature::Sharp);
        // Dilimler günün doğuşundan ve batışından.
        let s = d.segments.as_ref().unwrap();
        assert_eq!(s.eighths.len(), 8);
        assert_eq!(s.eighths[0].start_jd_ut, d.day_start_jd_ut);
        assert_eq!(s.eighths[7].end_jd_ut, d.sunset_jd_ut.unwrap());
        // Cumartesi: Rahu 3., Yamaganda 6., Gulika 1. dilim.
        assert_eq!((s.rahu.value, s.yamaganda.value, s.gulika.value), (3, 6, 1));
        assert_eq!(s.horas.len(), 24);
        assert_eq!(s.horas[0].value, Graha::Saturn);
        assert_eq!(s.horas[23].end_jd_ut, d.day_end_jd_ut);
    }

    #[test]
    fn the_day_agrees_with_the_birth_panchang_inside_every_interval() {
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        for offset in [0.0, 0.3, 0.68, 1.4, 7.9, 15.2, 29.5] {
            let jd = JD + offset;
            let d = day(&mut eph, jd, None);
            assert!(d.day_start_jd_ut <= jd && jd < d.day_end_jd_ut, "{offset}");
            let p = compute_panchang(&mut eph, jd, 41.0, 29.0, &s).unwrap();
            assert_eq!(d.vara, p.vara);
            assert_eq!(d.sunrise_jd_ut, p.sunrise_jd_ut);
            assert_eq!(d.sunset_jd_ut, p.sunset_jd_ut);
            for t in &d.tithis {
                let mid = 0.5 * (t.start_jd_ut + t.end_jd_ut);
                let p = compute_panchang(&mut eph, mid, 41.0, 29.0, &s).unwrap();
                assert_eq!((t.value.no, t.value.paksha), (p.tithi.no, p.tithi.paksha));
                assert_eq!(p.vara, d.vara, "the whole day has one vara");
            }
            for n in &d.nakshatras {
                let mid = 0.5 * (n.start_jd_ut + n.end_jd_ut);
                let p = compute_panchang(&mut eph, mid, 41.0, 29.0, &s).unwrap();
                assert_eq!(
                    (n.value.no, n.value.pada),
                    (p.nakshatra.no, p.nakshatra.pada)
                );
            }
            for y in &d.yogas {
                let mid = 0.5 * (y.start_jd_ut + y.end_jd_ut);
                let p = compute_panchang(&mut eph, mid, 41.0, 29.0, &s).unwrap();
                assert_eq!(y.value, p.yoga.no);
            }
            for k in &d.karanas {
                let mid = 0.5 * (k.start_jd_ut + k.end_jd_ut);
                let p = compute_panchang(&mut eph, mid, 41.0, 29.0, &s).unwrap();
                assert_eq!((k.value.no, k.value.name), (p.karana.no, p.karana.name));
            }
        }
    }

    #[test]
    fn flags_are_intervals_inside_the_day_in_a_fixed_order() {
        let mut eph = FakeEph::default();
        let d = day(&mut eph, JD, None);
        // Vishti (15. karana) uzanım 84°–90° arasındadır.
        assert_eq!(d.flags.len(), 1);
        let vishti = &d.flags[0];
        assert_eq!(vishti.value, MuhurtaFlag::Vishti);
        assert!((vishti.start_jd_ut - (JD - 1.0 / 12.22)).abs() < TOL);
        assert!((vishti.end_jd_ut - (JD + 5.0 / 12.22)).abs() < TOL);
        // Ertesi gün (93,19° → 105,41°): 96°'de 9. tithi (Rikta) başlar ve gün sonuna sürer.
        let d = day(&mut eph, JD + 1.0, None);
        assert_eq!(d.flags.len(), 1);
        let rikta = &d.flags[0];
        assert_eq!(rikta.value, MuhurtaFlag::Rikta);
        assert!((rikta.start_jd_ut - (JD + 11.0 / 12.22)).abs() < TOL);
        assert_eq!(rikta.end_jd_ut, d.day_end_jd_ut);

        // Ay 300°'yi geçerken panchaka başlar. Sahte Ay sidereal 71° + 13,2°/gün: 300°'ye 17,35. günde varır.
        let d = day(&mut eph, JD + 17.5, None);
        let panchaka: Vec<_> = d
            .flags
            .iter()
            .filter(|f| f.value == MuhurtaFlag::Panchaka)
            .collect();
        assert_eq!(panchaka.len(), 1);
        assert!((panchaka[0].start_jd_ut - (JD + 229.0 / 13.2)).abs() < TOL);
        assert_eq!(panchaka[0].end_jd_ut, d.day_end_jd_ut);
        // Sıra `DAY_FLAG_KINDS` sırasıdır.
        let order: Vec<usize> = d
            .flags
            .iter()
            .map(|f| DAY_FLAG_KINDS.iter().position(|k| *k == f.value).unwrap())
            .collect();
        assert!(order.windows(2).all(|w| w[0] <= w[1]));
        assert!(d.flags.iter().all(|f| !f.value.is_day_segment()));

        // Yanık Jüpiter: Güneş'i Jüpiter'in 5° yanına koy (orb 11°).
        let mut near = FakeEph::default().with(Body::Sun, 245.0, 0.98);
        let d = day(&mut near, JD, None);
        let combust: Vec<_> = d
            .flags
            .iter()
            .filter(|f| f.value == MuhurtaFlag::JupiterCombust)
            .collect();
        assert_eq!(combust.len(), 1);
        assert_eq!(
            (combust[0].start_jd_ut, combust[0].end_jd_ut),
            (d.day_start_jd_ut, d.day_end_jd_ut)
        );
    }

    #[test]
    fn personal_factors_follow_the_moon_through_the_day() {
        let mut eph = FakeEph::default();
        // Doğum Ay'ı: Rohini (4.), Boğa.
        let natal = NatalMoon {
            nakshatra: 4,
            sign: Sign::Taurus,
        };
        let d = day(&mut eph, JD, Some(natal));
        let p = d.personal.as_ref().unwrap();
        partitions(&p.tara, &d);
        partitions(&p.chandra, &d);
        // Gün içinde Ay Mrigashira (5.) → Ardra (6.): Rohini'den sayım 2 (Sampat) → 3 (Vipat).
        assert_eq!(
            p.tara.iter().map(|t| t.value).collect::<Vec<_>>(),
            [
                TaraBala {
                    number: 2,
                    favourable: true
                },
                TaraBala {
                    number: 3,
                    favourable: false
                }
            ]
        );
        assert!((p.tara[1].start_jd_ut - (JD - (71.0 - 200.0 / 3.0) / 13.2)).abs() < TOL);
        // Ay İkizler'de (60°–90°): Boğa'dan 2. ev, elverişli değil; gün boyu değişmez.
        assert_eq!(
            p.chandra.iter().map(|c| c.value).collect::<Vec<_>>(),
            [ChandraBala {
                house: 2,
                favourable: false
            }]
        );
        assert_eq!(
            day_panchang(
                &mut eph,
                JD,
                41.0,
                29.0,
                &CalcSettings::default(),
                Some(NatalMoon {
                    nakshatra: 28,
                    sign: Sign::Aries
                })
            ),
            Err(CoreError::InvalidMuhurtaInput("natal"))
        );
    }

    #[test]
    fn without_a_sunrise_the_day_starts_at_six_local_mean_time_and_has_no_segments() {
        let mut eph = FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        let d = day(&mut eph, JD, None);
        assert_eq!(d.warnings, [ChartWarning::SunriseUnavailable]);
        assert_eq!(
            (d.sunrise_jd_ut, d.sunset_jd_ut, d.next_sunrise_jd_ut),
            (None, None, None)
        );
        assert_eq!(d.segments, None);
        // Boylam 29°: 06:00 yerel ortalama saat = n + 0,6694 UT.
        let six = 2_451_544.75 - 29.0 / 360.0;
        assert!((d.day_start_jd_ut - six).abs() < 1e-9);
        assert!((d.day_end_jd_ut - (six + 1.0)).abs() < 1e-9);
        assert_eq!(d.vara, Vara::Saturday);
        partitions(&d.tithis, &d);
        partitions(&d.karanas, &d);
        // 06:00'dan önceki an önceki güne aittir.
        let before = day(&mut eph, JD - 0.4, None);
        assert!((before.day_end_jd_ut - six).abs() < 1e-9);
        assert_eq!(before.vara, Vara::Friday);
    }

    #[test]
    fn low_precision_and_invalid_input_are_reported_without_the_place() {
        let mut eph = FakeEph {
            precision: Precision::Moshier,
            ..FakeEph::default()
        };
        assert_eq!(
            day(&mut eph, JD, None).warnings,
            [ChartWarning::LowPrecision]
        );
        let s = CalcSettings::default();
        let mut eph = FakeEph::default();
        for (lat, lon) in [(91.0, 29.0), (41.0, 181.0), (f64::NAN, 29.0)] {
            assert_eq!(
                day_panchang(&mut eph, JD, lat, lon, &s, None),
                Err(CoreError::InvalidMuhurtaInput("place"))
            );
        }
        assert_eq!(
            day_panchang(&mut eph, 1e9, 41.0, 29.0, &s, None),
            Err(CoreError::InvalidJulianDay(1e9))
        );
        let old = CalcSettings {
            schema_version: 1,
            ..CalcSettings::default()
        };
        assert!(matches!(
            day_panchang(&mut eph, JD, 41.0, 29.0, &old, None),
            Err(CoreError::UnsupportedSchemaVersion { .. })
        ));
    }
}
