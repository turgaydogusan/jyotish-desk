//! Avastha komutu (yol haritası §4.4, Plan 24). İleri modül komutlarının kalıbı: tek
//! `ChartInput`, hesap motor iş parçacığında, komut gövdesi `blocking` havuzunda, hata
//! `CommandError`. Rapor `Chart`'a eklenmez ve saklanmaz; panel açıkken istenir (K1).

use std::sync::Arc;

use ephemeris::{Engine, SwissEph};
use jyotish_core::avastha::AvasthaReport;
use jyotish_core::ChartInput;
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::CommandError;

/// `compute_avasthas` komutunun gövdesi; bloklar.
pub fn compute_avasthas_blocking(
    engine: &Engine,
    input: ChartInput,
) -> Result<AvasthaReport, CommandError> {
    let report = engine
        .call(move |se: &mut SwissEph| jyotish_core::avastha::compute_avasthas(se, &input))??;
    Ok(report)
}

/// Avasthalar: yedi grahanın Baladi, Jagradadi, Lajjitadi (küme) ve Shayanadi durumu. Doğum
/// saati bilinmiyorsa (`time_known == false`) Lajjita değerlendirilmez, Shayanadi `null` döner
/// ve rapor `time_known: false` taşır; doğum yerinde o gün gün doğumu yoksa yalnızca Shayanadi
/// `null`'dır. Girdi doğum verisidir: loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn compute_avasthas(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
) -> Result<AvasthaReport, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_avasthas_blocking(&engine, input)).await
}
