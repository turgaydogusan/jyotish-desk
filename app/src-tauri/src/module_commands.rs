//! İleri modül komutları (ileri modüller spec'i K1–K3). Ortak kalıp: tek `ChartInput`, hesap
//! motor iş parçacığında, komut gövdesi `blocking` havuzunda, hata `CommandError`. Modül
//! sonuçları `Chart`'a eklenmez ve saklanmaz; panel açıkken istenir. Sonraki alt projelerin
//! komutları bu dosyaya aynı kalıpla eklenir.

use std::sync::Arc;

use ephemeris::{Engine, SwissEph};
use jyotish_core::{
    CharaDasha, ChartInput, DrishtiTable, JaiminiReport, Sign, SignDashaPath, SignDashaPeriod,
};
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

/// `compute_jaimini` komutunun gövdesi; bloklar.
pub fn compute_jaimini_blocking(
    engine: &Engine,
    input: ChartInput,
) -> Result<JaiminiReport, CommandError> {
    let report =
        engine.call(move |se: &mut SwissEph| jyotish_core::compute_jaimini(se, &input))??;
    Ok(report)
}

/// Jaimini çekirdeği: chara karaka, karakamsha, rashi drishti, arudha padalar ve argala.
/// Karaka şeması ve eş lord kuralı `input.settings.advanced`'ten okunur. Doğum saati
/// bilinmiyorsa (`time_known == false`) padalar `null` döner.
#[tauri::command]
#[specta::specta]
pub async fn compute_jaimini(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
) -> Result<JaiminiReport, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_jaimini_blocking(&engine, input)).await
}

/// `compute_chara_dasha` komutunun gövdesi; bloklar.
pub fn compute_chara_dasha_blocking(
    engine: &Engine,
    input: ChartInput,
) -> Result<Option<CharaDasha>, CommandError> {
    let dasha =
        engine.call(move |se: &mut SwissEph| jyotish_core::compute_chara_dasha(se, &input))??;
    Ok(dasha)
}

/// Chara dasha (padakrama düzeni): yön, on iki burcun yılları ve iki seviye dönem. Eş lord
/// kuralı `input.settings.advanced`'ten, yıl uzunluğu `input.settings.dasha_year`'dan okunur.
/// Doğum saati bilinmiyorsa (`time_known == false`) `null` döner: sıra lagnaya bağlıdır.
#[tauri::command]
#[specta::specta]
pub async fn compute_chara_dasha(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
) -> Result<Option<CharaDasha>, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_chara_dasha_blocking(&engine, input)).await
}

/// `chara_dasha_children` komutunun gövdesi; bloklar.
pub fn chara_dasha_children_blocking(
    engine: &Engine,
    input: ChartInput,
    path: Vec<Sign>,
    cycle: u8,
) -> Result<Vec<SignDashaPeriod>, CommandError> {
    let periods = engine.call(move |se: &mut SwissEph| {
        jyotish_core::compute_chara_dasha_children(se, &input, &path, cycle)
    })??;
    Ok(periods)
}

/// Bir chara dasha döneminin alt dönemleri. `path` dönemin `signs`'ı, `cycle` çevrimidir (1 ya
/// da 2); boş yol o çevrimin dashalarını verir. Doğum saati bilinmiyorsa liste boştur.
#[tauri::command]
#[specta::specta]
pub async fn chara_dasha_children(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
    path: Vec<Sign>,
    cycle: u8,
) -> Result<Vec<SignDashaPeriod>, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || chara_dasha_children_blocking(&engine, input, path, cycle)).await
}

/// `chara_dasha_at` komutunun gövdesi; bloklar.
pub fn chara_dasha_at_blocking(
    engine: &Engine,
    input: ChartInput,
    jd_ut: f64,
) -> Result<Option<SignDashaPath>, CommandError> {
    let path = engine
        .call(move |se: &mut SwissEph| jyotish_core::compute_chara_dasha_at(se, &input, jd_ut))??;
    Ok(path)
}

/// `jd_ut` anını içeren chara dasha dönemlerinin yolu (üç seviye). Dasa şeridinin "güncel
/// dönem"i buradan gelir; arayüz tarih karşılaştırmaz. `jd_ut` transit anıdır
/// (`TransitView.snapshot.jd_ut`). Doğum saati bilinmiyorsa ya da an dönemlerin dışındaysa
/// `null` döner.
#[tauri::command]
#[specta::specta]
pub async fn chara_dasha_at(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
    jd_ut: f64,
) -> Result<Option<SignDashaPath>, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || chara_dasha_at_blocking(&engine, input, jd_ut)).await
}
