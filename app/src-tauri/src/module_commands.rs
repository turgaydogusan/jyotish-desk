//! İleri modül komutları (ileri modüller spec'i K1–K3). Ortak kalıp: tek `ChartInput`, hesap
//! motor iş parçacığında, komut gövdesi `blocking` havuzunda, hata `CommandError`. Modül
//! sonuçları `Chart`'a eklenmez ve saklanmaz; panel açıkken istenir. Sonraki alt projelerin
//! komutları bu dosyaya aynı kalıpla eklenir.

use std::sync::Arc;

use ephemeris::{Engine, SwissEph};
use jyotish_core::{ChartInput, DrishtiTable};
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::CommandError;

/// `compute_drishti` komutunun gövdesi; bloklar.
pub fn compute_drishti_blocking(
    engine: &Engine,
    input: ChartInput,
) -> Result<DrishtiTable, CommandError> {
    let table =
        engine.call(move |se: &mut SwissEph| jyotish_core::compute_drishti(se, &input))??;
    Ok(table)
}

/// Graha bakışları: on üç tam bakış ve 7 × 9 sphuta drishti matrisi. `input`, açık haritanın
/// `jd_ut`, konum ve ayarlarıdır; `time_known` doğum saatinin bilinip bilinmediğidir.
#[tauri::command]
#[specta::specta]
pub async fn compute_drishti(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
) -> Result<DrishtiTable, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_drishti_blocking(&engine, input)).await
}
