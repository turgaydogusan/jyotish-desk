//! Bhava tablosu ve Sudarshana chakra komutları (yol haritası Plan 25). Kalıp
//! `module_commands` ile aynıdır: tek `ChartInput`, hesap motor iş parçacığında, komut gövdesi
//! `blocking` havuzunda, hata `CommandError`. Sonuçlar `Chart`'a eklenmez ve saklanmaz; panel
//! açıkken istenir (K1). Girdi doğum verisidir: loglanmaz.

use std::sync::Arc;

use ephemeris::{Engine, SwissEph};
use jyotish_core::{BhavaTable, ChartInput, SudarshanaChakra};
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::CommandError;

/// `compute_bhava_table` komutunun gövdesi; bloklar.
pub fn compute_bhava_table_blocking(
    engine: &Engine,
    input: ChartInput,
) -> Result<Option<BhavaTable>, CommandError> {
    let table =
        engine.call(move |se: &mut SwissEph| jyotish_core::compute_bhava_table(se, &input))??;
    Ok(table)
}

/// Bhava tablosu: haritanın ev sisteminde on iki bhavanın başlangıcı, ortası ve bitişi (burç,
/// derece, nakshatra, pada, burç lordu, nakshatra lordu) ve dokuz grahanın bhavası ile bhava
/// başlangıcına ve ortasına uzaklığı. Doğum saati bilinmiyorsa (`time_known == false`) `null`
/// döner. Placidus'ta bhava ortası yoktur; sayı prashnasında evler tam burçtur.
#[tauri::command]
#[specta::specta]
pub async fn compute_bhava_table(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
) -> Result<Option<BhavaTable>, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_bhava_table_blocking(&engine, input)).await
}

/// `compute_sudarshana` komutunun gövdesi; bloklar.
pub fn compute_sudarshana_blocking(
    engine: &Engine,
    input: ChartInput,
) -> Result<SudarshanaChakra, CommandError> {
    let chakra =
        engine.call(move |se: &mut SwissEph| jyotish_core::compute_sudarshana(se, &input))??;
    Ok(chakra)
}

/// Sudarshana chakra: lagna, Ay ve Güneş'i birinci ev sayan üç halka (içten dışa), her halkada
/// on iki ev ve içindeki grahalar. Doğum saati bilinmiyorsa lagna halkasının evleri `null` döner
/// ve sonuç `approximate` işaretlidir.
#[tauri::command]
#[specta::specta]
pub async fn compute_sudarshana(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
) -> Result<SudarshanaChakra, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_sudarshana_blocking(&engine, input)).await
}
