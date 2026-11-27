//! Efemeris tablosu komutu (yol haritası §4.9): yerin bir takvim ayında, her günün başında
//! (yerin duvar saatiyle 00:00) dokuz grahanın konumu. Ay verilmezse yerin bugünkü ayıdır;
//! önceki ve sonraki ay Rust'ta hesaplanır (arayüz tarih hesabı yapmaz, C4).

use std::sync::Arc;

use atlas::Timestamp;
use ephemeris::{Engine, SwissEph};
use jyotish_core::{CalcSettings, ChartWarning, EphemerisRow};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use super::{invalid, TOOLS_YEAR_RANGE};
use crate::client_commands::blocking;
use crate::commands::CommandError;
use crate::muhurta_commands::{day_start_jd, offset_at, today, MuhurtaPlace, J2000_JD};

/// Bir takvim ayı (yerin takvimi): `month` 1..=12.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct MonthId {
    pub year: i32,
    pub month: u8,
}

impl MonthId {
    fn is_valid(self) -> bool {
        TOOLS_YEAR_RANGE.contains(&self.year) && (1..=12).contains(&self.month)
    }

    /// Önceki ay; yıl aralığının dışına çıkıyorsa `None`.
    pub fn prev(self) -> Option<MonthId> {
        let m = if self.month == 1 {
            MonthId {
                year: self.year - 1,
                month: 12,
            }
        } else {
            MonthId {
                month: self.month - 1,
                ..self
            }
        };
        m.is_valid().then_some(m)
    }

    /// Sonraki ay; yıl aralığının dışına çıkıyorsa `None`.
    pub fn next(self) -> Option<MonthId> {
        let m = self.following();
        m.is_valid().then_some(m)
    }

    /// Sonraki ay, aralık denetimi olmadan (son satırı kapatan an için).
    fn following(self) -> MonthId {
        if self.month == 12 {
            MonthId {
                year: self.year + 1,
                month: 1,
            }
        } else {
            MonthId {
                month: self.month + 1,
                ..self
            }
        }
    }

    fn first_day(self) -> Result<atlas::Date, CommandError> {
        let (year, month) = (
            i16::try_from(self.year).map_err(|_| invalid("month"))?,
            i8::try_from(self.month).map_err(|_| invalid("month"))?,
        );
        atlas::Date::new(year, month, 1).map_err(|_| invalid("month"))
    }
}

/// Bir günün satırı: yerin takvim günü (`YYYY-MM-DD`), o günün başındaki UTC ofseti ve konumlar.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct EphemerisDayView {
    pub date: String,
    pub offset_secs: i32,
    pub row: EphemerisRow,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct EphemerisMonthView {
    /// Tablonun ayı (ay verilmediyse yerin bugünkü ayı).
    pub month: MonthId,
    pub prev: Option<MonthId>,
    pub next: Option<MonthId>,
    /// Ayın günleri, sırayla (28–31 satır).
    pub days: Vec<EphemerisDayView>,
    /// Yalnızca `LowPrecision` olabilir.
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

/// `ephemeris_month` gövdesi; `now` testte sabitlenir. Bloklar.
pub fn ephemeris_month_blocking(
    engine: &Engine,
    month: Option<MonthId>,
    place: MuhurtaPlace,
    settings: CalcSettings,
    now: Timestamp,
) -> Result<EphemerisMonthView, CommandError> {
    // Yer, aydan ve motordan önce denetlenir.
    offset_at(&place, J2000_JD).map_err(|_| invalid("place"))?;
    let month = match month {
        Some(m) => m,
        None => {
            let date: atlas::Date = today(&place, now)
                .map_err(|_| invalid("place"))?
                .parse()
                .map_err(|_| invalid("month"))?;
            MonthId {
                year: i32::from(date.year()),
                month: date.month().unsigned_abs(),
            }
        }
    };
    if !month.is_valid() {
        return Err(invalid("month"));
    }
    let first = month.first_day()?;
    let start_of = |date: atlas::Date| {
        day_start_jd(&date.to_string(), &place, "month").map_err(|_| invalid("month"))
    };
    let mut dates = Vec::with_capacity(31);
    let mut moments = Vec::with_capacity(32);
    let mut day = first;
    for _ in 0..first.days_in_month() {
        moments.push(start_of(day)?);
        dates.push(day);
        day = day.tomorrow().map_err(|_| invalid("month"))?;
    }
    // Son satırı kapatan an: sonraki ayın ilk gününün başı.
    moments.push(start_of(month.following().first_day()?)?);
    let table = {
        let moments = moments.clone();
        engine.call(move |se: &mut SwissEph| {
            jyotish_core::ephemeris_table(se, &settings, &moments)
        })??
    };
    let days = dates
        .into_iter()
        .zip(table.rows)
        .map(|(date, row)| {
            Ok(EphemerisDayView {
                date: date.to_string(),
                offset_secs: offset_at(&place, row.jd_ut).map_err(|_| invalid("place"))?,
                row,
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;
    Ok(EphemerisMonthView {
        month,
        prev: month.prev(),
        next: month.next(),
        days,
        warnings: table.warnings,
        method: table.method,
    })
}

/// Efemeris tablosu: yerin `month` takvim ayında her günün başındaki (00:00 duvar saati) dokuz
/// graha: burç, derece, retro, nakshatra ve pada; o gün içindeki burç girişi ve yön dönüşü
/// işaretli. `month` boşsa yerin bugünkü ayı ("bugün" Rust'ta okunur). `settings` açık haritanın
/// hesap ayarıdır (ayanamsa, düğüm türü, zodyak kipi). Yer loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn ephemeris_month(
    engine: State<'_, Arc<Engine>>,
    month: Option<MonthId>,
    place: MuhurtaPlace,
    settings: CalcSettings,
) -> Result<EphemerisMonthView, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || ephemeris_month_blocking(&engine, month, place, settings, Timestamp::now()))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn months_step_across_the_year_and_stop_at_the_range_ends() {
        let m = |year, month| MonthId { year, month };
        assert_eq!(m(2026, 11).prev(), Some(m(2026, 10)));
        assert_eq!(m(2026, 11).next(), Some(m(2026, 12)));
        assert_eq!(m(2026, 12).next(), Some(m(2027, 1)));
        assert_eq!(m(2027, 1).prev(), Some(m(2026, 12)));
        // Yıl aralığının uçları: 1 ve 2999.
        assert_eq!(m(1, 1).prev(), None);
        assert_eq!(m(1, 1).next(), Some(m(1, 2)));
        assert_eq!(m(2999, 12).next(), None);
        assert_eq!(m(2999, 12).prev(), Some(m(2999, 11)));
        for bad in [m(0, 5), m(3000, 1), m(2026, 0), m(2026, 13)] {
            assert!(!bad.is_valid(), "{bad:?}");
        }
        assert_eq!(m(2024, 2).first_day().unwrap().days_in_month(), 29);
        assert_eq!(m(2026, 2).first_day().unwrap().days_in_month(), 28);
    }
}
