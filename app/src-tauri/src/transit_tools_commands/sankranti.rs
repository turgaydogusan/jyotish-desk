//! Sankranti listesi komutu (yol haritası §4.8): yerin bir takvim yılındaki güneş girişleri.
//! Yıl, yerin saat dilimindeki 1 Ocak 00:00'dan sonraki 1 Ocak 00:00'a dektir; saatler yerin
//! diliminde gösterilsin diye her satır o andaki UTC ofsetini taşır.

use std::sync::Arc;

use atlas::Timestamp;
use ephemeris::{Engine, SwissEph};
use jyotish_core::{CalcSettings, ChartWarning, Sankranti};
use serde::Serialize;
use specta::Type;
use tauri::State;

use super::{invalid, TOOLS_YEAR_RANGE};
use crate::client_commands::blocking;
use crate::commands::CommandError;
use crate::muhurta_commands::{day_start_jd, offset_at, today, MuhurtaPlace, J2000_JD};

/// Bir Sankranti ve o andaki UTC ofseti (saat, yerin diliminde gösterilir).
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct SankrantiView {
    pub sankranti: Sankranti,
    pub offset_secs: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct SankrantiListView {
    /// Listenin yılı (yıl verilmediyse yerin bugünkü yılı).
    pub year: i32,
    /// Zaman sırasıyla; olağan bir yılda on iki satır.
    pub rows: Vec<SankrantiView>,
    /// Yılın uçları: yerin 1 Ocak 00:00'ı ve sonraki 1 Ocak 00:00'ı.
    pub from_jd_ut: f64,
    pub to_jd_ut: f64,
    /// `LowPrecision` ya da `SunriseUnavailable`.
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

/// Yerin takvim yılının başı (1 Ocak 00:00 duvar saati).
fn year_start_jd(year: i32, place: &MuhurtaPlace) -> Result<f64, CommandError> {
    day_start_jd(&format!("{year:04}-01-01"), place, "year").map_err(|_| invalid("year"))
}

/// `sankranti_list` gövdesi; `now` testte sabitlenir. Bloklar.
pub fn sankranti_list_blocking(
    engine: &Engine,
    year: Option<i32>,
    place: MuhurtaPlace,
    settings: CalcSettings,
    now: Timestamp,
) -> Result<SankrantiListView, CommandError> {
    // Yer, yıldan ve motordan önce denetlenir: geçersiz yer hangi yılla sorulursa sorulsun
    // `place`'tir.
    offset_at(&place, J2000_JD).map_err(|_| invalid("place"))?;
    let year = match year {
        Some(y) => y,
        None => {
            let date: atlas::Date = today(&place, now)
                .map_err(|_| invalid("place"))?
                .parse()
                .map_err(|_| invalid("year"))?;
            i32::from(date.year())
        }
    };
    if !TOOLS_YEAR_RANGE.contains(&year) {
        return Err(invalid("year"));
    }
    let from_jd_ut = year_start_jd(year, &place)?;
    let to_jd_ut = year_start_jd(year + 1, &place)?;
    let (lat, lon) = (place.lat, place.lon);
    let list = engine.call(move |se: &mut SwissEph| {
        jyotish_core::sankranti_list(se, &settings, lat, lon, from_jd_ut, to_jd_ut)
    })??;
    let rows = list
        .rows
        .into_iter()
        .map(|sankranti| {
            Ok(SankrantiView {
                offset_secs: offset_at(&place, sankranti.jd_ut).map_err(|_| invalid("place"))?,
                sankranti,
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;
    Ok(SankrantiListView {
        year,
        rows,
        from_jd_ut,
        to_jd_ut,
        warnings: list.warnings,
        method: list.method,
    })
}

/// Sankranti listesi: yerin `year` takvim yılında Güneş'in sidereal burç girişleri (haritanın
/// zodyak kipi ne olursa olsun seçili ayanamsayla sidereal) ve her giriş anının tithisi, varası
/// ve Ay nakshatrası. `year` boşsa yerin bugünkü yılı ("bugün" Rust'ta okunur). Yer loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn sankranti_list(
    engine: State<'_, Arc<Engine>>,
    year: Option<i32>,
    place: MuhurtaPlace,
    settings: CalcSettings,
) -> Result<SankrantiListView, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || sankranti_list_blocking(&engine, year, place, settings, Timestamp::now()))
        .await
}
