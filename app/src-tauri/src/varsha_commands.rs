//! Varshaphala komutları (yol haritası §4.14, Plan 27): yıllık harita, muntha ve mudda dasa;
//! yıllık haritayı olay haritası olarak kaydetme. İleri modül komutlarının kalıbı: hesap motor
//! iş parçacığında, komut gövdesi `blocking` havuzunda, hata `CommandError`. Rapor `Chart`'a
//! eklenmez ve saklanmaz (K1); kayıt var olan olay haritası yolunu kullanır (`ChartKind::Event`,
//! `Store::add_chart`), veritabanı şeması değişmez (Y6). "Şimdi" Rust'ta okunur (C4).
//!
//! Girdi doğum verisidir, yer kişisel veridir: loglanmaz. Bu dosyanın ürettiği hatalar yalnızca
//! alan adı taşır (`invalid_input`).

use std::sync::{Arc, Mutex};

use atlas::{Timestamp, TzSpec};
use ephemeris::{Engine, SwissEph};
use jyotish_core::varsha::{VarshaReport, VarshaYear};
use jyotish_core::ChartInput;
use serde::Serialize;
use specta::Type;
use store::{
    BirthData, ChartKind, ChartRecord, FoldChoice, HomePlace, NewChart, Store, TimeCertainty,
    TzChoice,
};
use tauri::State;

use crate::client_commands::{blocking, lock, StoreState};
use crate::commands::CommandError;
use crate::muhurta_commands::MuhurtaPlace;

/// Kaydedilen yıllık haritanın etiketinin başı. Klasik terimdir: üç dilde aynı yazılır (K11).
pub const VARSHA_LABEL: &str = "Varshaphala";

const UNIX_EPOCH_JD: f64 = 2_440_587.5;

/// Rapor ve yıllık haritanın yerinin yıl başındaki UTC ofseti (anlar bu ofsetle gösterilir).
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct VarshaView {
    pub report: VarshaReport,
    pub offset_secs: i32,
}

fn invalid(field: &str) -> CommandError {
    CommandError::new("invalid_input", &[("field", field.to_owned())])
}

fn tz_spec(tz: &TzChoice) -> TzSpec {
    match tz {
        TzChoice::Iana(id) => TzSpec::Iana(id.trim().to_owned()),
        TzChoice::Fixed(secs) => TzSpec::Fixed(*secs),
        TzChoice::Lmt => TzSpec::Lmt,
    }
}

/// Jülyen gününün en yakın saniyesi.
fn timestamp_of(jd_ut: f64) -> Result<Timestamp, CommandError> {
    let secs = (jd_ut - UNIX_EPOCH_JD) * 86_400.0;
    if !(secs.is_finite() && secs.abs() < 1e13) {
        return Err(invalid("varsha.moment"));
    }
    Timestamp::from_second(secs.round() as i64).map_err(|_| invalid("varsha.moment"))
}

/// Saat diliminin `at` anındaki UTC ofseti; çözülemeyen dilim yalnızca alan adıyla döner.
fn offset_at(tz: &TzChoice, lon: f64, at: Timestamp) -> Result<i32, CommandError> {
    atlas::offset_at(&tz_spec(tz), lon, at).map_err(|_| invalid("varsha.place"))
}

/// `compute_varsha` gövdesi; `now` testte sabitlenir. `age` boşsa `now` anında süren yıl
/// hesaplanır. Bloklar.
pub fn compute_varsha_blocking(
    engine: &Engine,
    input: ChartInput,
    age: Option<u8>,
    place: MuhurtaPlace,
    now: Timestamp,
) -> Result<VarshaView, CommandError> {
    let year = match age {
        Some(age) => VarshaYear::Age(age),
        None => VarshaYear::At(now.as_second() as f64 / 86_400.0 + UNIX_EPOCH_JD),
    };
    let at = (place.lat, place.lon);
    let report = engine.call(move |se: &mut SwissEph| {
        jyotish_core::varsha::compute_varsha(se, &input, year, Some(at))
    })??;
    let offset_secs = offset_at(&place.tz, place.lon, timestamp_of(report.start_jd_ut)?)?;
    Ok(VarshaView {
        report,
        offset_secs,
    })
}

/// Varshaphala: `age` yaşının (tamamlanan yıl; `null` ise şu anda süren yılın) yıllık haritası,
/// munthası ve mudda dasası. `input` açık haritanın girdisidir; `place` yıllık haritanın yeridir
/// (doğum yeri ya da bulunulan yer) ve saat dilimi anların gösterileceği ofseti verir. Doğum
/// saati bilinmiyorsa (`time_known == false`) yıllık harita ve muntha `null` döner. Girdi ve yer
/// loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn compute_varsha(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
    age: Option<u8>,
    place: MuhurtaPlace,
) -> Result<VarshaView, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_varsha_blocking(&engine, input, age, place, Timestamp::now())).await
}

/// Dönüş anının `home` yerindeki duvar saatiyle doğum verisi. Duvar saati yaz saati örtüşmesine
/// düşüyorsa dönüş anını veren taraf seçilir; öteki her durumda katlama seçimi yazılmaz.
fn event_birth(home: &HomePlace, at: Timestamp) -> Result<BirthData, CommandError> {
    let offset = offset_at(&home.tz, home.place.lon, at)?;
    let local = Timestamp::from_second(at.as_second() + i64::from(offset))
        .map_err(|_| invalid("varsha.moment"))?;
    let birth = |fold: Option<FoldChoice>| BirthData {
        date: local.strftime("%Y-%m-%d").to_string(),
        time: Some(local.strftime("%H:%M:%S").to_string()),
        time_certainty: TimeCertainty::Exact,
        place: home.place.clone(),
        tz: home.tz.clone(),
        fold,
        notes: String::new(),
    };
    // Yerel ortalama saatte ofset saniyeye yuvarlanır: çözülen an bir saniyeye dek ayrışabilir.
    let hits = |fold: Option<FoldChoice>| {
        store::resolve_birth(&birth(fold))
            .ok()
            .and_then(|(_, r)| r.utc.parse::<Timestamp>().ok())
            .is_some_and(|t| (t.as_second() - at.as_second()).abs() <= 1)
    };
    [None, Some(FoldChoice::Earlier), Some(FoldChoice::Later)]
        .into_iter()
        .find(|fold| hits(*fold))
        .map(birth)
        .ok_or_else(|| invalid("varsha.moment"))
}

/// `save_varsha_chart` gövdesi; bloklar. Kaynak harita, ayarları ve (istenirse) bulunulan yer
/// veritabanından okunur: arayüzden yer ya da an alınmaz.
pub fn save_varsha_chart_blocking(
    engine: &Engine,
    store: &Mutex<Store>,
    chart_id: u32,
    age: u8,
    home: bool,
) -> Result<ChartRecord, CommandError> {
    let (source, settings, number, home_place) = {
        let s = lock(store)?;
        let source = s.chart(chart_id)?;
        let settings = s.calc_settings(chart_id)?.settings().clone();
        let number = s.prashna_meta(chart_id)?.and_then(|p| p.number);
        (source, settings, number, s.app_prefs()?.home_place)
    };
    // Doğum saati bilinmiyorsa yıllık harita hesaplanmaz (K2): kaydedilecek harita yoktur.
    if source.birth.time_certainty == TimeCertainty::Unknown {
        return Err(invalid("birth.time"));
    }
    let place = if home {
        home_place.ok_or_else(|| invalid("home_place"))?
    } else {
        HomePlace {
            place: source.birth.place.clone(),
            tz: source.birth.tz.clone(),
        }
    };
    let input = ChartInput {
        jd_ut: source.resolved.jd_ut,
        lat: source.birth.place.lat,
        lon: source.birth.place.lon,
        settings: settings.clone(),
        time_known: true,
        lagna_number: number,
    };
    let at = (place.place.lat, place.place.lon);
    let report = engine.call(move |se: &mut SwissEph| {
        jyotish_core::varsha::compute_varsha(se, &input, VarshaYear::Age(age), Some(at))
    })??;
    let birth = event_birth(&place, timestamp_of(report.start_jd_ut)?)?;
    let chart = NewChart {
        label: format!("{VARSHA_LABEL} {}", birth.date),
        kind: ChartKind::Event,
        birth,
    };
    let mut s = lock(store)?;
    let saved = s.add_chart(source.client_id, &chart)?;
    // Yeni harita genel varsayılanı alır; yıllık harita kaynak haritanın ayarlarıyla
    // hesaplanmıştır: aynı ayar yazılır ki kayıt açılınca aynı harita çıksın.
    s.save_calc_settings(saved.id, &settings)?;
    Ok(s.chart(saved.id)?)
}

/// `age` yaşının yıllık haritasını kaynak haritanın danışanına olay haritası olarak kaydeder
/// (`ChartKind::Event`; etiket `Varshaphala <tarih>`). Yer doğum yeridir; `home` doğruysa
/// tercihlerdeki bulunulan yerdir. Kaynak haritanın ayarları yeni haritaya kopyalanır. Harita
/// açılmaz: dönen kayıt yalnızca bildirilir. Doğum saati bilinmeyen harita ve seçilmemiş
/// bulunulan yer `invalid_input` ile reddedilir.
#[tauri::command]
#[specta::specta]
pub async fn save_varsha_chart(
    engine: State<'_, Arc<Engine>>,
    store: State<'_, StoreState>,
    chart_id: u32,
    age: u8,
    home: bool,
) -> Result<ChartRecord, CommandError> {
    let (engine, store) = (Arc::clone(&engine), Arc::clone(&store));
    blocking(move || save_varsha_chart_blocking(&engine, &store, chart_id, age, home)).await
}
