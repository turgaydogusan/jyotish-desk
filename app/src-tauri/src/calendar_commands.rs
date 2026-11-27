//! Geçiş takvimi komutu (yol haritası §4.11, Plan 29): bir ayın takvim günleri, her günün gün
//! doğumundaki tithisi ve Ay nakshatrası ile o güne düşen gök olayları. Ortak kalıp (K1–K3):
//! hesap motor iş parçacığında, komut gövdesi `blocking` havuzunda, hata `CommandError`; sonuç
//! saklanmaz. "Bugün" Rust'ta okunur; arayüz tarih hesabı yapmaz (C4): önceki ve sonraki ay da
//! buradan döner.
//!
//! Ay dilim dilim hesaplanır (en çok yedi gün): her dilim ayrı bir motor işidir ve aralarında
//! motor iş parçacığı bırakılır; araya giren harita hesabı bir dilimden uzun beklemez.
//!
//! Gizlilik: yer (bulunulan yer dahil) ve açık haritanın doğum verisi kişisel veridir;
//! loglanmaz ve hata parametresine girmez. Bu dosyanın ürettiği girdi hataları yalnızca alan
//! adı taşır (`invalid_calendar_input`).

use std::sync::Arc;

use atlas::Timestamp;
use ephemeris::{Engine, SwissEph};
use jyotish_core::calendar::{
    calendar_slice, calendar_slices, CalendarFilter, CalendarSpec, DayFacts, DaySpan,
    CALENDAR_MAX_EVENTS, CALENDAR_METHOD,
};
use jyotish_core::{CalcSettings, ChartInput, ChartWarning, Positions, Vara};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::CommandError;
use crate::muhurta_commands::{day_start_jd, offset_at, today, MuhurtaPlace, J2000_JD};
use crate::search_commands::SkyEventView;

/// Takvimin kabul ettiği yıllar. Paketlenen ephemeris dosyaları 1800–2399'u kapsar; dışındaki
/// aylar Moshier hassasiyetindedir ve sonuç `LowPrecision` uyarısını taşır.
pub const CALENDAR_YEARS: std::ops::RangeInclusive<i32> = 1..=2999;
/// Gün panchangı paneline verilen an, olguların okunduğu andan bu kadar sonradır (bir dakika):
/// an kesinlikle o gün doğumuyla başlayan günün içindedir.
const PANCHANG_ANCHOR_DAYS: f64 = 60.0 / 86_400.0;

/// Bir takvim ayı.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct YearMonth {
    pub year: i32,
    /// 1..=12.
    pub month: u8,
}

/// Takvimin girdisi. `month` boşsa yerin bugünkü ayıdır. `place` günlerin sınırını (yerel gece
/// yarısı), gün doğumunu ve saatlerin gösterileceği dilimi belirler. `natal` açık haritanın
/// modül girdisidir (doğum boylamına gelişler için; o tür seçili değilse gönderilmez);
/// `settings` açık haritanın hesap ayarıdır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct CalendarRequest {
    pub month: Option<YearMonth>,
    pub place: MuhurtaPlace,
    pub filter: CalendarFilter,
    pub settings: CalcSettings,
    pub natal: Option<ChartInput>,
}

/// Bir takvim günü. Saatler `offset_secs` ile gösterilir; olaylar kendi ofsetlerini taşır (yaz
/// saati gün içinde değişebilir).
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct CalendarDayView {
    /// `YYYY-MM-DD`.
    pub date: String,
    /// Ayın günü, 1..=31.
    pub day: u8,
    /// Takvim gününün hafta günü.
    pub weekday: Vara,
    pub span: DaySpan,
    /// Gün doğumundaki tithi ve Ay nakshatrası (gün doğumu yoksa 06:00 yerel ortalama saat).
    pub facts: DayFacts,
    /// `facts.jd_ut` anındaki UTC ofseti.
    pub offset_secs: i32,
    /// Gün panchangı paneline verilecek an: o gün doğumuyla başlayan günün içindedir.
    pub panchang_jd_ut: f64,
    /// Zaman sırasıyla.
    pub events: Vec<SkyEventView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct CalendarView {
    pub month: YearMonth,
    /// Önceki ve sonraki ay; yıl aralığının ucunda `None`.
    pub prev: Option<YearMonth>,
    pub next: Option<YearMonth>,
    /// Yerin bugünkü günü bu aydaysa ayın günü.
    pub today: Option<u8>,
    pub days: Vec<CalendarDayView>,
    /// Olay sayısı ayın sınırını (`CALENDAR_MAX_EVENTS`) aştı: sonraki olaylar listede yoktur.
    pub truncated: bool,
    /// Doğum boylamına gelişler arandı (tür seçili ve açık haritanın girdisi verildi).
    pub natal: bool,
    /// `LowPrecision` ya da boş.
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

fn invalid(field: &str) -> CommandError {
    CommandError::new("invalid_calendar_input", &[("field", field.to_owned())])
}

/// Çekirdeğin ve muhurta yardımcılarının alan adlı hatasını takvimin koduna çevirir; alan adı
/// sabit bir kümeden seçilir (değer taşınmaz). Öteki hatalar (hesap, motor) olduğu gibi geçer.
fn calendar_error(e: CommandError) -> CommandError {
    if e.code != "invalid_search_input" && e.code != "invalid_muhurta_input" {
        return e;
    }
    let field = match e.params.get("field").map(String::as_str) {
        Some("place") => "place",
        Some("kinds") => "kinds",
        Some("month") => "month",
        _ => "range",
    };
    invalid(field)
}

fn civil(year: i32, month: u8, day: u8) -> Option<atlas::Date> {
    atlas::Date::new(
        i16::try_from(year).ok()?,
        i8::try_from(month).ok()?,
        i8::try_from(day).ok()?,
    )
    .ok()
}

impl YearMonth {
    fn is_valid(self) -> bool {
        CALENDAR_YEARS.contains(&self.year) && (1..=12).contains(&self.month)
    }

    /// `step` ay sonrası (−1 ya da +1); yıl aralığının dışına çıkıyorsa `None`.
    fn shifted(self, step: i32) -> Option<YearMonth> {
        let index = self.year * 12 + i32::from(self.month) - 1 + step;
        let next = YearMonth {
            year: index.div_euclid(12),
            month: u8::try_from(index.rem_euclid(12)).ok()? + 1,
        };
        next.is_valid().then_some(next)
    }
}

/// Bir ayın takvim günleri: (tarih, günün başı). Yerin saat diliminde var olmayan gün (takvimden
/// atlanan gün) listede yer almaz. Son öğe sonraki ayın ilk günüdür (son günün bitişi).
fn month_days(
    month: YearMonth,
    place: &MuhurtaPlace,
) -> Result<Vec<(atlas::Date, f64)>, CommandError> {
    let first = civil(month.year, month.month, 1).ok_or_else(|| invalid("month"))?;
    let mut out = Vec::new();
    let mut date = first;
    for _ in 0..first.days_in_month() {
        // Çözülemeyen gün başı: o gün yerin takviminde yoktur (saat dilimi bir günü atlamıştır).
        if let Ok(start) = day_start_jd(&date.to_string(), place, "month") {
            out.push((date, start));
        }
        date = date.tomorrow().map_err(|_| invalid("month"))?;
    }
    if out.is_empty() {
        return Err(invalid("month"));
    }
    let end = day_start_jd(&date.to_string(), place, "month").map_err(calendar_error)?;
    out.push((date, end));
    Ok(out)
}

/// Takvim tarihinin hafta günü.
fn weekday(date: atlas::Date) -> Vara {
    Vara::ALL[usize::from(date.weekday().to_sunday_zero_offset().unsigned_abs())]
}

/// `month_calendar` gövdesi; `now` testte sabitlenir. Bloklar; dilimleri sırayla yürütür ve
/// her dilimden sonra motor iş parçacığını bırakır.
pub fn month_calendar_blocking(
    engine: &Engine,
    request: CalendarRequest,
    now: Timestamp,
) -> Result<CalendarView, CommandError> {
    // Yer, aydan ve motordan önce denetlenir: geçersiz yer hangi ayla sorulursa sorulsun
    // `place`'tir.
    offset_at(&request.place, J2000_JD).map_err(calendar_error)?;
    let spec = CalendarSpec {
        lat: request.place.lat,
        lon: request.place.lon,
        settings: request.settings.clone(),
        filter: request.filter.clone(),
    };
    // Süzgeç ve ayar denetimi motordan önce.
    spec.validate()
        .map_err(|e| calendar_error(CommandError::from(e)))?;
    let today = today(&request.place, now).map_err(calendar_error)?;
    let today: atlas::Date = today.parse().map_err(|_| invalid("range"))?;
    let month = match request.month {
        Some(m) => m,
        None => YearMonth {
            year: i32::from(today.year()),
            month: today.month().unsigned_abs(),
        },
    };
    if !month.is_valid() {
        return Err(invalid("month"));
    }
    let days = month_days(month, &request.place)?;
    let spans: Vec<DaySpan> = days
        .windows(2)
        .map(|w| DaySpan {
            start_jd_ut: w[0].1,
            end_jd_ut: w[1].1,
        })
        .collect();

    let natal: Option<Arc<Positions>> = match request.natal.clone() {
        Some(input) => Some(Arc::new(engine.call(move |se: &mut SwissEph| {
            jyotish_core::compute_positions(se, &input)
        })??)),
        None => None,
    };
    let searched_natal = natal.is_some()
        && request
            .filter
            .event_spec(0.0, 1.0, true)
            .is_some_and(|s| s.kinds.iter().any(|k| k.needs_natal()));
    let spec = Arc::new(spec);
    let spans = Arc::new(spans);
    let mut found = Vec::with_capacity(spans.len());
    let mut truncated = false;
    let mut low_precision = false;
    let mut left = CALENDAR_MAX_EVENTS;
    for (from, to) in calendar_slices(spans.len()) {
        let (c, s, n) = (Arc::clone(&spec), Arc::clone(&spans), natal.clone());
        let slice = engine
            .call(move |se: &mut SwissEph| {
                calendar_slice(se, &c, n.as_deref(), &s[from..to], left)
            })?
            .map_err(|e| calendar_error(CommandError::from(e)))?;
        left -= slice.days.iter().map(|d| d.events.len()).sum::<usize>();
        truncated |= slice.truncated;
        low_precision |= slice.low_precision;
        found.extend(slice.days);
    }

    let days = days
        .iter()
        .zip(found)
        .map(|((date, _), day)| {
            let events = day
                .events
                .into_iter()
                .map(|event| {
                    Ok(SkyEventView {
                        offset_secs: offset_at(&request.place, event.jd_ut)
                            .map_err(calendar_error)?,
                        event,
                    })
                })
                .collect::<Result<Vec<_>, CommandError>>()?;
            Ok(CalendarDayView {
                date: date.to_string(),
                day: date.day().unsigned_abs(),
                weekday: weekday(*date),
                span: day.span,
                facts: day.facts,
                offset_secs: offset_at(&request.place, day.facts.jd_ut).map_err(calendar_error)?,
                panchang_jd_ut: (day.facts.jd_ut + PANCHANG_ANCHOR_DAYS)
                    .min(0.5 * (day.facts.jd_ut + day.span.end_jd_ut)),
                events,
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;
    let in_month =
        i32::from(today.year()) == month.year && today.month().unsigned_abs() == month.month;
    Ok(CalendarView {
        month,
        prev: month.shifted(-1),
        next: month.shifted(1),
        today: in_month.then(|| today.day().unsigned_abs()),
        days,
        truncated,
        natal: searched_natal,
        warnings: if low_precision {
            vec![ChartWarning::LowPrecision]
        } else {
            Vec::new()
        },
        method: CALENDAR_METHOD.to_owned(),
    })
}

/// Geçiş takvimi: bir ayın takvim günleri (yerin duvar saatiyle gece yarısından gece yarısına),
/// her günün gün doğumundaki tithisi ve Ay nakshatrası ile o güne düşen gök olayları. `month`
/// boşsa yerin bugünkü ayı döner ("bugün" Rust'ta okunur). Yer ve girdi loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn month_calendar(
    engine: State<'_, Arc<Engine>>,
    request: CalendarRequest,
) -> Result<CalendarView, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || month_calendar_blocking(&engine, request, Timestamp::now())).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn months_step_across_years_and_stop_at_the_ends_of_the_range() {
        let ym = |year: i32, month: u8| YearMonth { year, month };
        assert_eq!(ym(2026, 11).shifted(1), Some(ym(2026, 12)));
        assert_eq!(ym(2026, 12).shifted(1), Some(ym(2027, 1)));
        assert_eq!(ym(2027, 1).shifted(-1), Some(ym(2026, 12)));
        assert_eq!(ym(2026, 3).shifted(-1), Some(ym(2026, 2)));
        // Aralığın uçları.
        assert_eq!(ym(1, 1).shifted(-1), None);
        assert_eq!(ym(1, 1).shifted(1), Some(ym(1, 2)));
        assert_eq!(ym(2999, 12).shifted(1), None);
        assert_eq!(ym(2999, 12).shifted(-1), Some(ym(2999, 11)));
        assert!(!ym(0, 5).is_valid() && !ym(3000, 1).is_valid());
        assert!(!ym(2026, 0).is_valid() && !ym(2026, 13).is_valid());
        assert!(ym(1, 1).is_valid() && ym(2999, 12).is_valid());
    }

    #[test]
    fn the_weekday_of_a_civil_date_is_its_calendar_weekday() {
        // 2000-01-01 Cumartesi; 2026-11-01 Pazar; 2026-11-25 Çarşamba.
        let day = |y, m, d| weekday(civil(y, m, d).unwrap());
        assert_eq!(day(2000, 1, 1), Vara::Saturday);
        assert_eq!(day(2026, 11, 1), Vara::Sunday);
        assert_eq!(day(2026, 11, 25), Vara::Wednesday);
        // Var olmayan tarih kurulmaz.
        assert!(civil(2026, 2, 30).is_none());
        assert!(civil(40_000, 1, 1).is_none());
    }

    #[test]
    fn errors_name_the_field_and_carry_nothing_else() {
        let e = invalid("place");
        assert_eq!(e.code, "invalid_calendar_input");
        assert_eq!(e.params.len(), 1);
        assert_eq!(e.params["field"], "place");
        // Çekirdeğin ve muhurta yardımcılarının hatası takvimin koduna, sabit bir alan adıyla
        // çevrilir.
        let from = |code: &str, field: &str| {
            calendar_error(CommandError::new(code, &[("field", field.to_owned())]))
        };
        for (code, given, want) in [
            ("invalid_search_input", "kinds", "kinds"),
            ("invalid_search_input", "place", "place"),
            ("invalid_search_input", "range", "range"),
            ("invalid_search_input", "grahas", "range"),
            ("invalid_muhurta_input", "place", "place"),
            ("invalid_muhurta_input", "month", "month"),
            ("invalid_muhurta_input", "28.6139, 77.2090", "range"),
        ] {
            let e = from(code, given);
            assert_eq!(
                (e.code.as_str(), e.params["field"].as_str(), e.params.len()),
                ("invalid_calendar_input", want, 1)
            );
        }
        // Hesap ve motor hataları olduğu gibi geçer.
        let calc = calendar_error(CommandError::new("calc", &[("detail", "x".to_owned())]));
        assert_eq!(
            (calc.code.as_str(), calc.params["detail"].as_str()),
            ("calc", "x")
        );
    }
}
