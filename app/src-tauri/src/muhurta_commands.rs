//! Muhurta komutları (ileri modüller spec'i §10): gün panchangı ve pencere araması. Ortak kalıp
//! (K1–K3): hesap motor iş parçacığında, komut gövdesi `blocking` havuzunda, hata
//! `CommandError`; sonuç saklanmaz. "Şimdi" ve "bugün" Rust'ta okunur; arayüz tarih hesabı
//! yapmaz (C4). Arama dilim dilim yürür: her çağrı bir dilimi hesaplar ve motor iş parçacığını
//! bırakır; aramayı durdurmak, sonraki dilimi istememektir (durum tutulmaz).
//!
//! Yer (bulunulan yer dahil) kişisel veridir: loglanmaz ve hata parametresine girmez. Bu
//! dosyanın ürettiği hatalar yalnızca alan adı taşır (`invalid_muhurta_input`).

use std::sync::Arc;

use atlas::{Timestamp, TzSpec};
use ephemeris::{Engine, SwissEph};
use jyotish_core::{
    CalcSettings, ChartWarning, DayPanchang, MuhurtaCriteria, MuhurtaCursor, MuhurtaFilter,
    NatalMoon, WindowSnapshot,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use store::{BirthData, FoldChoice, Place, TimeCertainty, TzChoice};
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::CommandError;
use crate::transit_commands::{transit_moment, TransitAt};

/// Tarih verilmeyen aramanın uzunluğu, gün: başlangıç günü ve sonraki 30 gün.
pub const DEFAULT_SEARCH_DAYS: f64 = 31.0;
const UNIX_EPOCH_JD: f64 = 2_440_587.5;

/// Hesabın yapılacağı yer ve saatlerin gösterileceği dilim. Ad taşımaz.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct MuhurtaPlace {
    /// Kuzey +.
    pub lat: f64,
    /// Doğu +.
    pub lon: f64,
    pub tz: TzChoice,
}

/// Gün panchangı ve günün başındaki UTC ofseti (saatler bu ofsetle gösterilir).
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct DayPanchangView {
    pub day: DayPanchang,
    pub offset_secs: i32,
}

/// Aramanın girdisi. `from_date` ve `to_date` yerin takvim günleridir (`YYYY-MM-DD`, ikisi de
/// dahil); `from_date` boşsa bugün, `to_date` boşsa başlangıçtan 30 gün sonrası.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct MuhurtaQuery {
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub place: MuhurtaPlace,
    pub criteria: MuhurtaCriteria,
    pub natal: Option<NatalMoon>,
}

/// Sonraki dilimin imleci: ilk çağrıda çözülen aralık ve çekirdeğin imleci. Arayüz olduğu gibi
/// geri gönderir; böylece gece yarısını aşan bir arama aralığını değiştirmez.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
pub struct MuhurtaPageCursor {
    pub from_jd_ut: f64,
    pub to_jd_ut: f64,
    pub cursor: MuhurtaCursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Type)]
pub struct MuhurtaWindowView {
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
    /// Başlangıç ve bitiş anlarındaki UTC ofseti (yaz saati pencerenin içinde değişebilir).
    pub start_offset_secs: i32,
    pub end_offset_secs: i32,
    pub snapshot: WindowSnapshot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct MuhurtaSearchView {
    /// Bu dilimde kapanan pencereler.
    pub windows: Vec<MuhurtaWindowView>,
    pub scanned: u32,
    pub chunks: u32,
    /// Taranan aralığın tamamı ve uçlarındaki ofset.
    pub from_jd_ut: f64,
    pub to_jd_ut: f64,
    pub from_offset_secs: i32,
    pub to_offset_secs: i32,
    /// Sonraki dilimin imleci; arama bittiyse `null`.
    pub next: Option<MuhurtaPageCursor>,
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

fn invalid(field: &str) -> CommandError {
    CommandError::new("invalid_muhurta_input", &[("field", field.to_owned())])
}

fn tz_spec(tz: &TzChoice) -> TzSpec {
    match tz {
        TzChoice::Iana(id) => TzSpec::Iana(id.trim().to_owned()),
        TzChoice::Fixed(secs) => TzSpec::Fixed(*secs),
        TzChoice::Lmt => TzSpec::Lmt,
    }
}

fn timestamp_of(jd_ut: f64) -> Option<Timestamp> {
    let secs = (jd_ut - UNIX_EPOCH_JD) * 86_400.0;
    if secs.is_finite() && secs.abs() < 1e13 {
        Timestamp::from_second(secs.floor() as i64).ok()
    } else {
        None
    }
}

/// Yerin denetimi ve bir andaki ofseti. Geçersiz koordinat ya da çözülemeyen saat dilimi
/// yalnızca alan adıyla döner.
fn offset_at(place: &MuhurtaPlace, jd_ut: f64) -> Result<i32, CommandError> {
    if !((-90.0..=90.0).contains(&place.lat) && (-180.0..=180.0).contains(&place.lon)) {
        return Err(invalid("place"));
    }
    let utc = timestamp_of(jd_ut).ok_or_else(|| invalid("range"))?;
    atlas::offset_at(&tz_spec(&place.tz), place.lon, utc).map_err(|_| invalid("place"))
}

/// `day_panchang` gövdesi; `now` testte sabitlenir. Bloklar.
pub fn day_panchang_blocking(
    engine: &Engine,
    at: TransitAt,
    now: Timestamp,
    place: MuhurtaPlace,
    settings: CalcSettings,
    natal: Option<NatalMoon>,
) -> Result<DayPanchangView, CommandError> {
    let (jd_ut, _) = transit_moment(&at, now)?;
    // Yer, motoru meşgul etmeden önce denetlenir.
    offset_at(&place, jd_ut)?;
    let (lat, lon) = (place.lat, place.lon);
    let day = engine.call(move |se: &mut SwissEph| {
        jyotish_core::day_panchang(se, jd_ut, lat, lon, &settings, natal)
    })??;
    let offset_secs = offset_at(&place, day.day_start_jd_ut)?;
    Ok(DayPanchangView { day, offset_secs })
}

/// Gün panchangı: `at` anını içeren gün (gün doğumundan sonraki gün doğumuna) için beş uzvun
/// gün içindeki değerleri, günün dilimleri, işaretler ve (doğum Ay'ı verilirse) tara ve chandra
/// bala. `at` transit anıdır ("şimdi" Rust'ta okunur). Yer kişisel veridir: loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn day_panchang(
    engine: State<'_, Arc<Engine>>,
    at: TransitAt,
    place: MuhurtaPlace,
    settings: CalcSettings,
    natal: Option<NatalMoon>,
) -> Result<DayPanchangView, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || day_panchang_blocking(&engine, at, Timestamp::now(), place, settings, natal))
        .await
}

/// Yerin takvim gününün başı (00:00 duvar saati). Gece yarısı yaz saati boşluğuna düşerse
/// öğleden 12 saat öncesi alınır.
fn day_start_jd(date: &str, place: &MuhurtaPlace, field: &str) -> Result<f64, CommandError> {
    let date: atlas::Date = date.trim().parse().map_err(|_| invalid(field))?;
    let resolve = |time: &str| {
        store::resolve_birth(&BirthData {
            date: date.to_string(),
            time: Some(time.to_owned()),
            time_certainty: TimeCertainty::Exact,
            place: Place {
                name: String::new(),
                country: String::new(),
                admin1: String::new(),
                lat: place.lat,
                lon: place.lon,
                geoname_id: None,
            },
            tz: place.tz.clone(),
            fold: Some(FoldChoice::Earlier),
            notes: String::new(),
        })
        .map(|(_, r)| r.jd_ut)
    };
    resolve("00:00")
        .or_else(|_| resolve("12:00").map(|noon| noon - 0.5))
        .map_err(|_| invalid(field))
}

/// Yerin bugünkü takvim günü (`YYYY-MM-DD`).
fn today(place: &MuhurtaPlace, now: Timestamp) -> Result<String, CommandError> {
    let offset =
        atlas::offset_at(&tz_spec(&place.tz), place.lon, now).map_err(|_| invalid("place"))?;
    let local = Timestamp::from_second(now.as_second() + i64::from(offset))
        .map_err(|_| invalid("range"))?;
    Ok(local.strftime("%Y-%m-%d").to_string())
}

/// Aramanın aralığı: imleç varsa onunki, yoksa tarihlerden çözülür.
fn search_range(
    query: &MuhurtaQuery,
    cursor: Option<&MuhurtaPageCursor>,
    now: Timestamp,
) -> Result<(f64, f64), CommandError> {
    if let Some(c) = cursor {
        return Ok((c.from_jd_ut, c.to_jd_ut));
    }
    let from_date = match &query.from_date {
        Some(d) => d.clone(),
        None => today(&query.place, now)?,
    };
    let from = day_start_jd(&from_date, &query.place, "from_date")?;
    let to = match &query.to_date {
        Some(d) => day_start_jd(d, &query.place, "to_date")? + 1.0,
        None => from + DEFAULT_SEARCH_DAYS,
    };
    Ok((from, to))
}

/// `muhurta_search` gövdesi; `now` testte sabitlenir. Bloklar ve tek bir dilim hesaplar.
pub fn muhurta_search_blocking(
    engine: &Engine,
    query: MuhurtaQuery,
    settings: CalcSettings,
    cursor: Option<MuhurtaPageCursor>,
    now: Timestamp,
) -> Result<MuhurtaSearchView, CommandError> {
    let (from_jd_ut, to_jd_ut) = search_range(&query, cursor.as_ref(), now)?;
    let from_offset_secs = offset_at(&query.place, from_jd_ut)?;
    let to_offset_secs = offset_at(&query.place, to_jd_ut)?;
    let filter = MuhurtaFilter {
        from_jd_ut,
        to_jd_ut,
        lat: query.place.lat,
        lon: query.place.lon,
        criteria: query.criteria.clone(),
        natal: query.natal,
    };
    let core_cursor = cursor.map(|c| c.cursor);
    let page = engine.call(move |se: &mut SwissEph| {
        jyotish_core::muhurta_search_page(se, &settings, &filter, core_cursor)
    })??;
    let windows = page
        .windows
        .iter()
        .map(|w| {
            Ok(MuhurtaWindowView {
                start_jd_ut: w.start_jd_ut,
                end_jd_ut: w.end_jd_ut,
                start_offset_secs: offset_at(&query.place, w.start_jd_ut)?,
                end_offset_secs: offset_at(&query.place, w.end_jd_ut)?,
                snapshot: w.snapshot,
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;
    Ok(MuhurtaSearchView {
        windows,
        scanned: page.scanned,
        chunks: page.chunks,
        from_jd_ut,
        to_jd_ut,
        from_offset_secs,
        to_offset_secs,
        next: page.next.map(|cursor| MuhurtaPageCursor {
            from_jd_ut,
            to_jd_ut,
            cursor,
        }),
        warnings: page.warnings,
        method: page.method,
    })
}

/// Muhurta araması, bir dilim: süzgece uyan zaman pencereleri. İlk çağrıda `cursor` boştur;
/// dönen `next` bir sonraki çağrıya olduğu gibi verilir, `null` ise arama bitmiştir. Pencere
/// dilim sınırını aşarsa kapandığı dilimde verilir. Yer ve doğum Ay'ı loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn muhurta_search(
    engine: State<'_, Arc<Engine>>,
    query: MuhurtaQuery,
    settings: CalcSettings,
    cursor: Option<MuhurtaPageCursor>,
) -> Result<MuhurtaSearchView, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || muhurta_search_blocking(&engine, query, settings, cursor, Timestamp::now()))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(tz: TzChoice) -> MuhurtaPlace {
        MuhurtaPlace {
            lat: 41.0138,
            lon: 28.9497,
            tz,
        }
    }

    fn istanbul() -> MuhurtaPlace {
        place(TzChoice::Iana("Europe/Istanbul".into()))
    }

    fn query(from: Option<&str>, to: Option<&str>, place: MuhurtaPlace) -> MuhurtaQuery {
        MuhurtaQuery {
            from_date: from.map(str::to_owned),
            to_date: to.map(str::to_owned),
            place,
            criteria: MuhurtaCriteria::default(),
            natal: None,
        }
    }

    /// 2026-11-26T22:30:00Z: İstanbul'da 27 Kasım 01:30.
    fn now() -> Timestamp {
        "2026-11-26T22:30:00Z".parse().unwrap()
    }

    /// 2026-11-27T00:00 İstanbul (+03:00) = 2026-11-26T21:00:00Z.
    const NOV_27_ISTANBUL: f64 = 2_461_371.375;

    #[test]
    fn ranges_are_whole_local_days_and_default_to_today_plus_thirty() {
        let r = search_range(&query(None, None, istanbul()), None, now()).unwrap();
        assert_eq!(r, (NOV_27_ISTANBUL, NOV_27_ISTANBUL + 31.0));
        // İki tarih de dahildir.
        let q = query(Some("2026-11-27"), Some("2026-11-27"), istanbul());
        assert_eq!(
            search_range(&q, None, now()).unwrap(),
            (NOV_27_ISTANBUL, NOV_27_ISTANBUL + 1.0)
        );
        // Sabit ofset ve yerel ortalama saat (28,9497° × 240 = 6948 sn).
        let q = query(Some("2026-11-27"), None, place(TzChoice::Fixed(0)));
        assert_eq!(search_range(&q, None, now()).unwrap().0, 2_461_371.5);
        let q = query(Some("2026-11-27"), None, place(TzChoice::Lmt));
        let lmt = search_range(&q, None, now()).unwrap().0;
        assert!((lmt - (2_461_371.5 - 6_947.928 / 86_400.0)).abs() < 1e-6);
        // UTC'de hâlâ 26 Kasım'dır.
        let r = search_range(&query(None, None, place(TzChoice::Fixed(0))), None, now()).unwrap();
        assert_eq!(r.0, 2_461_370.5);
        // İmleç varsa aralık ondan okunur; tarihler ve saat yeniden çözülmez.
        let cursor = MuhurtaPageCursor {
            from_jd_ut: 10.0,
            to_jd_ut: 20.0,
            cursor: MuhurtaCursor {
                chunk: 1,
                open_start_jd_ut: None,
            },
        };
        let q = query(Some("bozuk"), None, istanbul());
        assert_eq!(
            search_range(&q, Some(&cursor), now()).unwrap(),
            (10.0, 20.0)
        );
    }

    #[test]
    fn a_midnight_inside_a_dst_gap_falls_back_to_twelve_hours_before_noon() {
        // Asunción 2010-10-03: saatler 00:00'da 01:00'e alındı; 00:00 yoktur.
        let asuncion = MuhurtaPlace {
            lat: -25.3,
            lon: -57.64,
            tz: TzChoice::Iana("America/Asuncion".into()),
        };
        let start = day_start_jd("2010-10-03", &asuncion, "from_date").unwrap();
        // Öğle 12:00 −03:00 = 15:00 UTC; 12 saat öncesi 03:00 UTC.
        assert!((start - 2_455_472.625).abs() < 1e-9, "{start}");
    }

    #[test]
    fn errors_name_the_field_and_never_carry_the_place() {
        let field = |e: CommandError| {
            assert_eq!(e.code, "invalid_muhurta_input");
            assert_eq!(e.params.len(), 1, "{:?}", e.params);
            e.params["field"].clone()
        };
        let bad_date = query(Some("27.11.2026"), None, istanbul());
        assert_eq!(
            field(search_range(&bad_date, None, now()).unwrap_err()),
            "from_date"
        );
        let bad_to = query(Some("2026-11-27"), Some("2026-13-01"), istanbul());
        assert_eq!(
            field(search_range(&bad_to, None, now()).unwrap_err()),
            "to_date"
        );
        for bad in [
            place(TzChoice::Iana("Mars/Olympus".into())),
            place(TzChoice::Fixed(19 * 3600)),
            MuhurtaPlace {
                lat: 91.0,
                ..istanbul()
            },
            MuhurtaPlace {
                lon: f64::NAN,
                ..istanbul()
            },
        ] {
            assert_eq!(
                field(offset_at(&bad, NOV_27_ISTANBUL).unwrap_err()),
                "place"
            );
        }
        assert_eq!(
            field(offset_at(&istanbul(), f64::INFINITY).unwrap_err()),
            "range"
        );
        assert_eq!(offset_at(&istanbul(), NOV_27_ISTANBUL).unwrap(), 10_800);
        assert_eq!(
            offset_at(&place(TzChoice::Lmt), NOV_27_ISTANBUL).unwrap(),
            6_948
        );
    }
}
