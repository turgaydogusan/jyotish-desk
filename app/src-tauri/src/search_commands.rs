//! Arama komutları (ileri modüller spec'i §12): danışan veritabanında harita araması, toplu
//! etiketleme, kayıtlı aramalar ve zaman içinde olay araması. Ortak kalıp (K1–K3): hesap motor
//! iş parçacığında, komut gövdesi `blocking` havuzunda, hata `CommandError`; sonuç saklanmaz.
//!
//! Her iki arama da dilim dilim yürür ve dilim sınırlarında motor iş parçacığını bırakır: araya
//! giren harita hesabı bir dilimden uzun beklemez. Harita aramasının sayfasını arayüz ister
//! (`chart_search`, `done` olana dek); olay aramasının dilimlerini komut kendisi yürütür. Her
//! aramanın bir kimliği vardır: durdurulan ya da yerine yenisi başlatılan aramanın sonraki
//! isteği `search_stale` ile reddedilir.
//!
//! Gizlilik: sorgu, sonuçlar (adlar, eşleşme sayıları, harita kimlikleri), yer ve açık haritanın
//! doğum boylamları loglanmaz ve hata parametresine girmez. Bu dosyanın ürettiği girdi hataları
//! yalnızca alan adı taşır (`invalid_search_input`).

use std::sync::{Arc, Mutex};

use atlas::Timestamp;
use ephemeris::{Engine, SwissEph};
use jyotish_core::{
    CalcSettings, ChartInput, ChartQuery, EventSpec, Graha, MatchFact, Outcome, Positions,
    SkyEvent, SkyEventKind, EVENT_METHOD, MAX_SKY_EVENTS, SEARCH_METHOD,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use store::{SavedSearch, SearchRow, Store, SEARCH_PAGE_CHARTS};
use tauri::State;

use crate::client_commands::{blocking, lock, with_store, StoreState};
use crate::commands::CommandError;
use crate::muhurta_commands::{day_start_jd, offset_at, today, MuhurtaPlace, J2000_JD};

/// Bir motor işinde hesaplanan en çok harita: sayfa (500 harita) beş işe bölünür ve aralarında
/// motor iş parçacığı bırakılır.
pub const SEARCH_SLICE_CHARTS: usize = 100;
/// Bitiş tarihi verilmeyen olay aramasının uzunluğu, gün.
pub const DEFAULT_EVENT_DAYS: f64 = 366.0;
const UNIX_EPOCH_JD: f64 = 2_440_587.5;

/// Bir arama türünün durumu: süren aramanın kimliği ve en son durdurulan kimlik.
#[derive(Debug, Default)]
struct Slot {
    live: Option<u32>,
    /// Durdurma, aramanın ilk isteği kaydolmadan önce gelebilir (istek store kilidini beklerken
    /// arayüz yeni bir arama başlatır): durdurulan kimlik sonradan başlayamaz. Aksi halde geç
    /// kaydolan eski arama yenisinin yerine geçerdi.
    cancelled: Option<u32>,
}

/// Süren aramaların kimlikleri. Bir türde aynı anda tek arama sürer: yenisi başlayınca ya da
/// arama durdurulunca eskisinin kimliği geçersiz olur.
#[derive(Debug, Default)]
pub struct SearchRegistry {
    chart: Mutex<Slot>,
    event: Mutex<Slot>,
}

fn slot(m: &Mutex<Slot>) -> std::sync::MutexGuard<'_, Slot> {
    // Kilit yalnızca iki sayı tutar; zehirlenmiş kilidin değeri de geçerlidir.
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Aramayı kaydeder; kimlik daha önce durdurulduysa arama başlamaz (`search_stale`).
fn begin(m: &Mutex<Slot>, id: u32) -> Result<(), CommandError> {
    let mut s = slot(m);
    if s.cancelled == Some(id) {
        return Err(stale());
    }
    s.live = Some(id);
    Ok(())
}

fn is_live(m: &Mutex<Slot>, id: u32) -> bool {
    slot(m).live == Some(id)
}

/// Biten ya da hatayla sonlanan aramanın kaydını siler; başka bir arama sürüyorsa dokunmaz.
fn end(m: &Mutex<Slot>, id: u32) {
    let mut s = slot(m);
    if s.live == Some(id) {
        s.live = None;
    }
}

fn cancel(m: &Mutex<Slot>, id: u32) {
    let mut s = slot(m);
    if s.live == Some(id) {
        s.live = None;
    }
    s.cancelled = Some(id);
}

impl SearchRegistry {
    /// Harita aramasını durdurur: kimlik geçersiz olur. Başka bir arama sürüyorsa dokunulmaz.
    pub fn cancel_chart_search(&self, search_id: u32) {
        cancel(&self.chart, search_id);
    }

    /// Olay aramasını durdurur.
    pub fn cancel_event_search(&self, search_id: u32) {
        cancel(&self.event, search_id);
    }
}

fn invalid(field: &str) -> CommandError {
    CommandError::new("invalid_search_input", &[("field", field.to_owned())])
}

fn stale() -> CommandError {
    CommandError::new("search_stale", &[])
}

/// Sonraki sayfanın imleci: taramanın geldiği yer ve o ana dek biriken sayılar. Arayüz olduğu
/// gibi geri gönderir.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
pub struct ChartSearchCursor {
    /// Bu kimlikten büyük haritalar taranacak.
    pub after_chart_id: u32,
    pub scanned: u32,
    pub skipped_time_unknown: u32,
    pub failed: u32,
    /// Arama başlarken kapsamdaki harita sayısı.
    pub total: u32,
    /// Aramanın başladığı an (tarihi verilmeyen `DashaAt` koşulları için); sayfalar arasında
    /// değişmez.
    pub now_jd_ut: f64,
}

/// Eşleşen bir harita.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct ChartHit {
    pub client_id: u32,
    pub chart_id: u32,
    pub client_name: String,
    /// Boşsa arayüz harita türünün adını gösterir.
    pub chart_label: String,
    pub matched: Vec<MatchFact>,
}

/// Bir sayfanın sonucu. Sayılar aramanın başından bu sayfanın sonuna dek birikmiş değerlerdir;
/// `hits` yalnızca bu sayfanın eşleşmeleridir.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct ChartSearchPage {
    pub search_id: u32,
    pub hits: Vec<ChartHit>,
    pub scanned: u32,
    pub total: u32,
    /// Sorgu lagna istediği ve doğum saati bilinmediği için değerlendirilemeyen haritalar.
    pub skipped_time_unknown: u32,
    /// Hesaplanamayan haritalar (ör. hesap aralığının dışındaki tarih).
    pub failed: u32,
    pub done: bool,
    /// Sonraki sayfanın imleci; `done` ise `null`.
    pub next: Option<ChartSearchCursor>,
    pub method: String,
}

fn jd_of(now: Timestamp) -> f64 {
    now.as_millisecond() as f64 / 86_400_000.0 + UNIX_EPOCH_JD
}

/// Bir dilimin motor işi: her harita için hafif yol ve değerlendirme.
/// Dönen üçlü: (eşleşmeler, saati bilinmediği için atlananlar, hesaplanamayanlar).
pub fn evaluate_rows(
    se: &mut SwissEph,
    query: &ChartQuery,
    rows: Vec<SearchRow>,
    now_jd_ut: f64,
) -> (Vec<ChartHit>, u32, u32) {
    let (mut hits, mut unknown, mut failed) = (Vec::new(), 0, 0);
    for row in rows {
        let outcome = jyotish_core::search_facts(se, &row.input)
            .and_then(|facts| jyotish_core::evaluate(query, &facts, now_jd_ut));
        match outcome {
            Ok(Outcome::Match(matched)) => hits.push(ChartHit {
                client_id: row.client_id,
                chart_id: row.chart_id,
                client_name: row.client_name,
                chart_label: row.chart_label,
                matched,
            }),
            Ok(Outcome::NoMatch) => {}
            Ok(Outcome::TimeUnknown) => unknown += 1,
            // Hesaplanamayan harita aramayı durdurmaz; yalnızca sayılır.
            Err(_) => failed += 1,
        }
    }
    (hits, unknown, failed)
}

/// `chart_search` gövdesi; `now` testte sabitlenir. Bloklar ve tek bir sayfa hesaplar. Hatayla
/// biten aramanın kaydı silinir (yerine başlamış başka bir aramaya dokunulmaz).
pub fn chart_search_blocking(
    engine: &Engine,
    store: &Mutex<Store>,
    registry: &SearchRegistry,
    query: ChartQuery,
    search_id: u32,
    cursor: Option<ChartSearchCursor>,
    now: Timestamp,
) -> Result<ChartSearchPage, CommandError> {
    let page = chart_search_page(engine, store, registry, query, search_id, cursor, now);
    if page.is_err() {
        end(&registry.chart, search_id);
    }
    page
}

fn chart_search_page(
    engine: &Engine,
    store: &Mutex<Store>,
    registry: &SearchRegistry,
    query: ChartQuery,
    search_id: u32,
    cursor: Option<ChartSearchCursor>,
    now: Timestamp,
) -> Result<ChartSearchPage, CommandError> {
    // Sorgu, veritabanından ve motordan önce denetlenir; hata yalnızca alan adıdır.
    query.validate()?;
    let state = match cursor {
        Some(c) => {
            if !is_live(&registry.chart, search_id) {
                return Err(stale());
            }
            c
        }
        None => {
            let total = lock(store)?.search_scope_count(&query.scope)?;
            begin(&registry.chart, search_id)?;
            ChartSearchCursor {
                after_chart_id: 0,
                scanned: 0,
                skipped_time_unknown: 0,
                failed: 0,
                total,
                now_jd_ut: jd_of(now),
            }
        }
    };
    // Store kilidi yalnızca okuma boyunca tutulur; hesap sırasında kayıt işleri beklemez.
    let rows = lock(store)?.search_rows(&query.scope, state.after_chart_id, SEARCH_PAGE_CHARTS)?;
    let done = rows.len() < SEARCH_PAGE_CHARTS as usize;
    let after_chart_id = rows.last().map_or(state.after_chart_id, |r| r.chart_id);
    // İmleç arayüzden geri gelir: sayaçlar taşmaz.
    let scanned = state.scanned.saturating_add(rows.len() as u32);

    let query = Arc::new(query);
    let (mut hits, mut unknown, mut failed) = (Vec::new(), 0, 0);
    let mut rows = rows.into_iter();
    loop {
        let slice: Vec<SearchRow> = rows.by_ref().take(SEARCH_SLICE_CHARTS).collect();
        if slice.is_empty() {
            break;
        }
        // Dilim sınırı: durdurulan aramanın sonraki dilimi hesaplanmaz.
        if !is_live(&registry.chart, search_id) {
            return Err(stale());
        }
        let (q, now_jd_ut) = (Arc::clone(&query), state.now_jd_ut);
        let (h, u, f) =
            engine.call(move |se: &mut SwissEph| evaluate_rows(se, &q, slice, now_jd_ut))?;
        hits.extend(h);
        unknown += u;
        failed += f;
    }
    let skipped_time_unknown = state.skipped_time_unknown.saturating_add(unknown);
    let failed = state.failed.saturating_add(failed);
    if done {
        end(&registry.chart, search_id);
    }
    Ok(ChartSearchPage {
        search_id,
        hits,
        scanned,
        total: state.total,
        skipped_time_unknown,
        failed,
        done,
        next: (!done).then_some(ChartSearchCursor {
            after_chart_id,
            scanned,
            skipped_time_unknown,
            failed,
            total: state.total,
            now_jd_ut: state.now_jd_ut,
        }),
        method: SEARCH_METHOD.to_owned(),
    })
}

/// Harita araması, bir sayfa (en çok 500 harita): sorgunun koşullarını sağlayan kayıtlı haritalar.
/// İlk çağrıda `cursor` boştur ve `search_id` yeni bir aramayı başlatır (süren arama varsa
/// geçersiz olur); dönen `next` bir sonraki çağrıya olduğu gibi verilir. Her harita kendi kayıtlı
/// anı, yeri ve hesap ayarıyla hesaplanır. Sorgu ve sonuçlar loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn chart_search(
    engine: State<'_, Arc<Engine>>,
    store: State<'_, StoreState>,
    registry: State<'_, Arc<SearchRegistry>>,
    query: ChartQuery,
    search_id: u32,
    cursor: Option<ChartSearchCursor>,
) -> Result<ChartSearchPage, CommandError> {
    let (engine, store, registry) = (
        Arc::clone(&engine),
        Arc::clone(&store),
        Arc::clone(&registry),
    );
    blocking(move || {
        chart_search_blocking(
            &engine,
            &store,
            &registry,
            query,
            search_id,
            cursor,
            Timestamp::now(),
        )
    })
    .await
}

/// Harita aramasını durdurur: süren sayfa dilim sınırında biter ve sonraki istek `search_stale`
/// döner. Kimlik süren aramaya ait değilse hiçbir şey yapmaz.
#[tauri::command]
#[specta::specta]
pub fn chart_search_cancel(
    registry: State<'_, Arc<SearchRegistry>>,
    search_id: u32,
) -> Result<(), CommandError> {
    registry.cancel_chart_search(search_id);
    Ok(())
}

/// "Sonuçları etiketle"nin girdisi.
#[derive(Debug, Clone, PartialEq, Deserialize, Type)]
pub struct TagChartsRequest {
    pub chart_ids: Vec<u32>,
    pub tag: String,
}

/// Etiketi verilen haritaların hepsine ekler (öteki etiketleri durur). Etiketlenen harita
/// sayısını döner; artık var olmayan haritalar atlanır. Arayüz onay penceresinden sonra çağırır.
#[tauri::command]
#[specta::specta]
pub async fn tag_charts(
    store: State<'_, StoreState>,
    request: TagChartsRequest,
) -> Result<u32, CommandError> {
    with_store(&store, move |s| {
        s.tag_charts(&request.chart_ids, &request.tag)
    })
    .await
}

/// "Aramayı sakla"nın girdisi.
#[derive(Debug, Clone, PartialEq, Deserialize, Type)]
pub struct SaveSearchRequest {
    pub name: String,
    pub query: ChartQuery,
}

#[tauri::command]
#[specta::specta]
pub async fn list_saved_searches(
    store: State<'_, StoreState>,
) -> Result<Vec<SavedSearch>, CommandError> {
    with_store(&store, |s| s.saved_searches()).await
}

/// Aramayı adıyla saklar; aynı adlı kayıt varsa koşulları değişir. Yalnızca koşullar saklanır.
#[tauri::command]
#[specta::specta]
pub async fn save_search(
    store: State<'_, StoreState>,
    request: SaveSearchRequest,
) -> Result<SavedSearch, CommandError> {
    with_store(&store, move |s| {
        s.save_search(&request.name, &request.query)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_saved_search(
    store: State<'_, StoreState>,
    search_id: u32,
) -> Result<(), CommandError> {
    with_store(&store, move |s| s.delete_saved_search(search_id)).await
}

/// Olay aramasının girdisi. `from_date` ve `to_date` yerin takvim günleridir (`YYYY-MM-DD`, ikisi
/// de dahil); `from_date` boşsa bugün, `to_date` boşsa başlangıçtan bir yıl sonrası. `place`
/// yalnızca günlerin sınırını ve saatlerin gösterileceği dilimi belirler. `natal` açık haritanın
/// modül girdisidir (doğum boylamına geliş ve doğum Ay'ına göre ev değiştirme için); `settings`
/// açık haritanın, harita yoksa genel varsayılanın hesap ayarıdır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct EventSearchSpec {
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub place: MuhurtaPlace,
    pub kinds: Vec<SkyEventKind>,
    pub grahas: Vec<Graha>,
    pub settings: CalcSettings,
    pub natal: Option<ChartInput>,
}

/// Bir olay ve o andaki UTC ofseti (saat, yerin diliminde gösterilir).
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct SkyEventView {
    pub event: SkyEvent,
    pub offset_secs: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct EventSearchView {
    /// Zaman sırasıyla; en çok `MAX_SKY_EVENTS`.
    pub events: Vec<SkyEventView>,
    /// Olay sayısı sınırı aştı: liste sınırda kesildi ve aralığın sonuna dek taranmadı.
    pub truncated: bool,
    /// Taranan aralık (kesildiyse `scanned_to_jd_ut` aralığın sonundan öncedir) ve uçlarındaki
    /// ofset.
    pub from_jd_ut: f64,
    pub to_jd_ut: f64,
    pub scanned_to_jd_ut: f64,
    pub from_offset_secs: i32,
    pub to_offset_secs: i32,
    pub method: String,
}

/// Muhurta yardımcılarının alan adlı hatasını aramanın koduna çevirir; alan adı sabit bir
/// kümeden seçilir (değer taşınmaz).
fn field_error(e: CommandError) -> CommandError {
    let field = match e.params.get("field").map(String::as_str) {
        Some("place") => "place",
        Some("from_date") => "from_date",
        Some("to_date") => "to_date",
        _ => "range",
    };
    invalid(field)
}

/// `event_search` gövdesi; `now` testte sabitlenir. Bloklar; dilimleri sırayla yürütür ve her
/// dilimden sonra motor iş parçacığını bırakır. Hatayla biten aramanın kaydı silinir.
pub fn event_search_blocking(
    engine: &Engine,
    registry: &SearchRegistry,
    spec: EventSearchSpec,
    search_id: u32,
    now: Timestamp,
) -> Result<EventSearchView, CommandError> {
    let view = event_search_run(engine, registry, spec, search_id, now);
    if view.is_err() {
        end(&registry.event, search_id);
    }
    view
}

/// Bitiş günü dahildir: aralık, sonraki takvim gününün başında biter. Gün 23 ya da 25 saat
/// sürebilir (yaz saati geçişi); bu yüzden günün başına 24 saat eklenmez.
fn day_end_jd(date: &str, place: &MuhurtaPlace) -> Result<f64, CommandError> {
    let day: atlas::Date = date.trim().parse().map_err(|_| invalid("to_date"))?;
    let next = day.tomorrow().map_err(|_| invalid("to_date"))?;
    day_start_jd(&next.to_string(), place, "to_date").map_err(field_error)
}

fn event_search_run(
    engine: &Engine,
    registry: &SearchRegistry,
    spec: EventSearchSpec,
    search_id: u32,
    now: Timestamp,
) -> Result<EventSearchView, CommandError> {
    // Yer, tarihlerden önce denetlenir: geçersiz yer hangi tarihle sorulursa sorulsun
    // `place`'tir.
    offset_at(&spec.place, J2000_JD).map_err(field_error)?;
    let from_date = match &spec.from_date {
        Some(d) => d.clone(),
        None => today(&spec.place, now).map_err(field_error)?,
    };
    let from_jd_ut = day_start_jd(&from_date, &spec.place, "from_date").map_err(field_error)?;
    let to_jd_ut = match &spec.to_date {
        Some(d) => day_end_jd(d, &spec.place)?,
        None => from_jd_ut + DEFAULT_EVENT_DAYS,
    };
    let core = EventSpec {
        from_jd_ut,
        to_jd_ut,
        kinds: spec.kinds.clone(),
        grahas: spec.grahas.clone(),
    };
    // Tür, graha, aralık ve "açık harita gerekir" denetimi motordan önce.
    core.validate(spec.natal.is_some())?;
    let from_offset_secs = offset_at(&spec.place, from_jd_ut).map_err(field_error)?;
    let to_offset_secs = offset_at(&spec.place, to_jd_ut).map_err(field_error)?;

    begin(&registry.event, search_id)?;
    let natal: Option<Arc<Positions>> = match spec.natal.clone() {
        Some(input) => Some(Arc::new(engine.call(move |se: &mut SwissEph| {
            jyotish_core::compute_positions(se, &input)
        })??)),
        None => None,
    };
    let (core, settings) = (Arc::new(core), Arc::new(spec.settings.clone()));
    let mut events: Vec<SkyEvent> = Vec::new();
    let mut truncated = false;
    let mut scanned_to_jd_ut = from_jd_ut;
    for chunk in jyotish_core::event_chunks(&core) {
        // Dilim sınırı: durdurulan ya da yerine yenisi başlatılan arama burada biter.
        if !is_live(&registry.event, search_id) {
            return Err(stale());
        }
        let (c, s, n) = (Arc::clone(&core), Arc::clone(&settings), natal.clone());
        let found = engine.call(move |se: &mut SwissEph| {
            jyotish_core::event_search_chunk(se, &s, &c, n.as_deref(), chunk)
        })??;
        events.extend(found);
        scanned_to_jd_ut = chunk.1;
        if events.len() > MAX_SKY_EVENTS {
            events.truncate(MAX_SKY_EVENTS);
            truncated = true;
            // Kesilen listenin son olayından sonrası listede yoktur (tam o ana düşen bir başka
            // olay da kesilmiş olabilir).
            scanned_to_jd_ut = events.last().map_or(chunk.1, |e| e.jd_ut);
            break;
        }
    }
    if !is_live(&registry.event, search_id) {
        return Err(stale());
    }
    end(&registry.event, search_id);
    let events = events
        .into_iter()
        .map(|event| {
            Ok(SkyEventView {
                offset_secs: offset_at(&spec.place, event.jd_ut).map_err(field_error)?,
                event,
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;
    Ok(EventSearchView {
        events,
        truncated,
        from_jd_ut,
        to_jd_ut,
        scanned_to_jd_ut,
        from_offset_secs,
        to_offset_secs,
        method: EVENT_METHOD.to_owned(),
    })
}

/// Olay araması: tarih aralığındaki gök olayları, zaman sırasıyla. `search_id` aramanın
/// kimliğidir; `event_search_cancel` ile durdurulur. Açık haritanın doğum boylamları yalnızca
/// bellekte kullanılır. Yer ve girdi loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn event_search(
    engine: State<'_, Arc<Engine>>,
    registry: State<'_, Arc<SearchRegistry>>,
    spec: EventSearchSpec,
    search_id: u32,
) -> Result<EventSearchView, CommandError> {
    let (engine, registry) = (Arc::clone(&engine), Arc::clone(&registry));
    blocking(move || event_search_blocking(&engine, &registry, spec, search_id, Timestamp::now()))
        .await
}

/// Olay aramasını durdurur: süren arama dilim sınırında `search_stale` ile biter.
#[tauri::command]
#[specta::specta]
pub fn event_search_cancel(
    registry: State<'_, Arc<SearchRegistry>>,
    search_id: u32,
) -> Result<(), CommandError> {
    registry.cancel_event_search(search_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_search_id_is_live_until_it_is_cancelled_replaced_or_finished() {
        let r = SearchRegistry::default();
        assert!(!is_live(&r.chart, 1));
        begin(&r.chart, 1).unwrap();
        assert!(is_live(&r.chart, 1));
        // Başka bir kimliğin durdurulması süren aramaya dokunmaz.
        r.cancel_chart_search(9);
        assert!(is_live(&r.chart, 1));
        // Yeni arama eskisini geçersiz kılar.
        begin(&r.chart, 2).unwrap();
        assert!(!is_live(&r.chart, 1) && is_live(&r.chart, 2));
        r.cancel_chart_search(2);
        assert!(!is_live(&r.chart, 2));
        // İki tür birbirinden bağımsızdır.
        begin(&r.chart, 7).unwrap();
        begin(&r.event, 7).unwrap();
        r.cancel_event_search(7);
        assert!(is_live(&r.chart, 7) && !is_live(&r.event, 7));
        end(&r.chart, 7);
        assert!(!is_live(&r.chart, 7));
    }

    #[test]
    fn a_search_cancelled_before_it_registers_never_starts() {
        // Arama 3'ün ilk isteği store kilidini beklerken arayüz onu durdurur ve arama 4'ü
        // başlatır; 4 önce kaydolur. Geç kaydolmak isteyen 3 başlayamaz ve 4 sürer.
        let r = SearchRegistry::default();
        r.cancel_chart_search(3);
        begin(&r.chart, 4).unwrap();
        assert_eq!(begin(&r.chart, 3).unwrap_err().code, "search_stale");
        assert!(is_live(&r.chart, 4));
        // Hatayla biten eski aramanın temizliği yenisine dokunmaz.
        end(&r.chart, 3);
        assert!(is_live(&r.chart, 4));
        // Olay aramasında da aynıdır.
        r.cancel_event_search(5);
        assert_eq!(begin(&r.event, 5).unwrap_err().code, "search_stale");
        assert!(begin(&r.event, 6).is_ok());
    }

    #[test]
    fn errors_name_the_field_and_carry_nothing_else() {
        let e = invalid("place");
        assert_eq!(e.code, "invalid_search_input");
        assert_eq!(e.params.len(), 1);
        assert_eq!(e.params["field"], "place");
        assert!(stale().params.is_empty());
        // Muhurta yardımcılarının hatası aramanın koduna, sabit bir alan adıyla çevrilir.
        let from = |field: &str| {
            field_error(CommandError::new(
                "invalid_muhurta_input",
                &[("field", field.to_owned())],
            ))
        };
        for (given, want) in [
            ("place", "place"),
            ("from_date", "from_date"),
            ("to_date", "to_date"),
            ("range", "range"),
            ("41.0138, 28.9497", "range"),
        ] {
            let e = from(given);
            assert_eq!(
                (e.code.as_str(), e.params["field"].as_str()),
                ("invalid_search_input", want)
            );
        }
        // Alanı olmayan hata da yalnızca alan adı taşır.
        assert_eq!(field_error(stale()).params["field"], "range");
    }

    #[test]
    fn the_search_moment_is_the_julian_day_of_now() {
        let now: Timestamp = "2000-01-01T12:00:00Z".parse().unwrap();
        assert_eq!(jd_of(now), 2_451_545.0);
    }
}
