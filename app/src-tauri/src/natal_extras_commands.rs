//! Doğum anı eklerinin komutları (yol haritası Plan 23): panchang ekleri, navatara ve
//! Vimshopaka bala. Kalıp `module_commands` ile aynıdır: tek `ChartInput`, hesap motor iş
//! parçacığında, komut gövdesi `blocking` havuzunda, hata `CommandError`. Sonuçlar `Chart`'a
//! eklenmez ve saklanmaz; panel açıkken istenir (K1). Girdiler doğum verisidir: loglanmaz.

use std::sync::Arc;

use ephemeris::{Engine, SwissEph};
use jyotish_core::{ChartInput, Navatara, PanchangExtras, VimshopakaReport};
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::CommandError;

/// `compute_panchang_extras` komutunun gövdesi; bloklar.
pub fn compute_panchang_extras_blocking(
    engine: &Engine,
    input: ChartInput,
) -> Result<PanchangExtras, CommandError> {
    let extras = engine
        .call(move |se: &mut SwissEph| jyotish_core::compute_panchang_extras(se, &input))??;
    Ok(extras)
}

/// Panchang ekleri: yogi noktası, yogi, eş yogi, avayogi ve doğum tithisinin dagdha burçları.
/// Lagna gerekmez; doğum saati bilinmiyorsa (`time_known == false`) değerler öğlen
/// konumlarındandır ve sonuç `approximate` işaretlidir.
#[tauri::command]
#[specta::specta]
pub async fn compute_panchang_extras(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
) -> Result<PanchangExtras, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_panchang_extras_blocking(&engine, input)).await
}

/// `compute_navatara` komutunun gövdesi; bloklar.
pub fn compute_navatara_blocking(
    engine: &Engine,
    input: ChartInput,
    transit_jd_ut: Option<f64>,
) -> Result<Navatara, CommandError> {
    let table = engine.call(move |se: &mut SwissEph| {
        jyotish_core::compute_navatara(se, &input, transit_jd_ut)
    })??;
    Ok(table)
}

/// Navatara: doğum Ay'ının nakshatrasından başlayan 27 satırlık tara tablosu (nakshatra, tara,
/// tur, lord, o nakshatradaki doğum grahaları). `transit_jd_ut` verilirse (transit anı:
/// `TransitView.snapshot.jd_ut`) o andaki dokuz grahanın tarası da döner; arayüz an hesabı
/// yapmaz. Doğum saati bilinmiyorsa sonuç `approximate` işaretlidir.
#[tauri::command]
#[specta::specta]
pub async fn compute_navatara(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
    transit_jd_ut: Option<f64>,
) -> Result<Navatara, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_navatara_blocking(&engine, input, transit_jd_ut)).await
}

/// `compute_vimshopaka` komutunun gövdesi; bloklar.
pub fn compute_vimshopaka_blocking(
    engine: &Engine,
    input: ChartInput,
) -> Result<VimshopakaReport, CommandError> {
    let report =
        engine.call(move |se: &mut SwissEph| jyotish_core::compute_vimshopaka(se, &input))??;
    Ok(report)
}

/// Vimshopaka bala: yedi grahanın on altı vargadaki burcu ve basamağı ile dört kümedeki
/// (shadvarga, saptavarga, dashavarga, shodashavarga) yirmi üzerinden değeri. Lagna gerekmez;
/// doğum saati bilinmiyorsa sonuç `approximate` işaretlidir.
#[tauri::command]
#[specta::specta]
pub async fn compute_vimshopaka(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
) -> Result<VimshopakaReport, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_vimshopaka_blocking(&engine, input)).await
}
