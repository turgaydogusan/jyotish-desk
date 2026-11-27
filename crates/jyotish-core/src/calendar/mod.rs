//! Geçiş takvimi (yol haritası §4.11, Plan 29): bir ayın takvim günleri, her günün gün
//! doğumundaki tithisi ve Ay nakshatrası ile o güne düşen gök olayları.
//!
//! - `grid`: takvim günleri (`DaySpan`), günün gün doğumu olguları (`DayFacts`) ve dilimler.
//! - `events`: takvimin olay süzgeci (`CalendarFilter`), olayların günlere dağıtımı ve sınır.
//!
//! Gün, yerin takvim günüdür: sınırlarını (yerel gece yarısı) çağıran verir, çünkü saat dilimi
//! bu kasada çözülmez. Olgular o günün içindeki gün doğumunda okunur; gün doğumu yoksa (kutup
//! gecesi ya da gündüzü) 06:00 yerel ortalama saatte. Olaylar olay aramasının çekirdeğinden
//! (`search::events`) gelir: yeni astronomi yoktur. Hepsi olgudur: yorum, puan ya da "uygun gün"
//! hükmü yoktur. Sonuç saklanmaz (K1). Adlar `jyotish_core` kökünden yeniden dışa aktarılmaz;
//! tüketiciler `jyotish_core::calendar::…` yolunu kullanır.

mod grid;

pub use grid::{
    calendar_slices, check_spans, day_facts, fallback_moment, sunrise_in, DayFacts, DaySpan,
    CALENDAR_MAX_DAYS, CALENDAR_SLICE_DAYS, MAX_DAY_SPAN_DAYS,
};

/// Yöntem kimliği (K7): takvim günü yerel gece yarısından gece yarısınadır; olgular günün
/// içindeki gün doğumunda okunur.
pub const CALENDAR_METHOD: &str = "calendar.month_v1";
