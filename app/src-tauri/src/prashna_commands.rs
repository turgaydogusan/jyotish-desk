//! Prashna komutları (ileri modüller spec'i §11): soru anının haritasını kurma, soru metnini ve
//! odak evini değiştirme, haritanın olguları. Ortak kalıp (K1–K3): hesap motor iş parçacığında,
//! komut gövdesi `blocking` havuzunda, hata `CommandError`. "Şimdi" Rust'ta okunur; girilen an
//! sorunun yerinin duvar saatidir ve Rust'ta çözülür (C4).
//!
//! Soru metni not gövdesi gibi hassastır; yer ve an kişisel veridir: hiçbiri loglanmaz ve hata
//! parametresine girmez. Bu dosyanın ürettiği hatalar yalnızca alan adı taşır
//! (`invalid_prashna_input`): v1'in doğum verisi hataları (`unknown_time_zone { tz }`,
//! `nonexistent_local_time { local, suggestion }`, `invalid_input { field: "birth.…" }`) bu
//! yollarda üretilmez.

use std::sync::{Arc, Mutex};

use atlas::{Timestamp, TzSpec};
use ephemeris::{Engine, SwissEph};
use jyotish_core::{ChartInput, PrashnaFacts, PRASHNA_NUMBER_MAX};
use serde::Deserialize;
use specta::Type;
use store::{
    BirthData, FoldChoice, HomePlace, NewPrashna, PrashnaMeta, Store, StoreError, TimeCertainty,
    TzChoice, MAX_NAME_CHARS, MAX_QUESTION_CHARS,
};
use tauri::State;

use crate::client_commands::{blocking, lock, open_client_blocking, OpenedClient, StoreState};
use crate::commands::CommandError;

/// Prashna haritasının etiketinin başı. Klasik terimdir: üç dilde aynı yazılır (K11).
pub const PRASHNA_LABEL: &str = "Prashna";

/// Soru anı. `Local`: sorunun yerinin duvar saati (`YYYY-MM-DD`, `HH:MM` ya da `HH:MM:SS`).
#[derive(Debug, Clone, PartialEq, Deserialize, Type)]
pub enum PrashnaMoment {
    Now,
    Local { date: String, time: String },
}

/// Yeni prashnanın girdisi. `client_id` boşsa harita yerleşik "Prashna" danışanına eklenir.
/// `place` sorunun yeridir (varsayılanı bulunulan yer) ve haritanın yeri olarak saklanır.
/// `number` doluysa sayı prashnasıdır: lagna sayıdan gelir (1..=108).
#[derive(Debug, Clone, PartialEq, Deserialize, Type)]
pub struct PrashnaRequest {
    pub client_id: Option<u32>,
    pub at: PrashnaMoment,
    pub place: HomePlace,
    pub number: Option<u8>,
    pub question: String,
    pub focus_house: Option<u8>,
}

/// Soru metni ve odak evi (otomatik kayıt).
#[derive(Debug, Clone, PartialEq, Deserialize, Type)]
pub struct PrashnaEdit {
    pub question: String,
    pub focus_house: Option<u8>,
}

fn invalid(field: &str) -> CommandError {
    CommandError::new("invalid_prashna_input", &[("field", field.to_owned())])
}

/// Store hatasının prashna yollarındaki karşılığı. Alan hataları ve an çözümü hataları yalnızca
/// alan adına iner: doğum verisinin hataları tarihi, saati ve dilimin adını parametreye yazar.
/// Veritabanı hataları ve `not_found { what, id }` olduğu gibi geçer (kişi verisi taşımazlar).
fn store_error(e: StoreError) -> CommandError {
    match e {
        StoreError::InvalidInput("prashna.question") => invalid("question"),
        StoreError::InvalidInput("prashna.number") => invalid("number"),
        StoreError::InvalidInput("prashna.focus_house") => invalid("focus_house"),
        StoreError::InvalidInput("birth.place.lat" | "birth.place.lon") => invalid("place"),
        StoreError::InvalidInput(_) | StoreError::Moment(_) => invalid("at"),
        other => other.into(),
    }
}

fn tz_spec(tz: &TzChoice) -> TzSpec {
    match tz {
        TzChoice::Iana(id) => TzSpec::Iana(id.trim().to_owned()),
        TzChoice::Fixed(secs) => TzSpec::Fixed(*secs),
        TzChoice::Lmt => TzSpec::Lmt,
    }
}

/// Yerin denetimi ve `now` anındaki ofseti. Geçersiz koordinat, fazla uzun ad ya da
/// çözülemeyen saat dilimi yalnızca alan adıyla döner.
fn place_offset(place: &HomePlace, now: Timestamp) -> Result<i32, CommandError> {
    let p = &place.place;
    let coords_ok = (-90.0..=90.0).contains(&p.lat) && (-180.0..=180.0).contains(&p.lon);
    let names_ok = [&p.name, &p.country, &p.admin1]
        .iter()
        .all(|s| s.chars().count() <= MAX_NAME_CHARS);
    if !(coords_ok && names_ok) {
        return Err(invalid("place"));
    }
    atlas::offset_at(&tz_spec(&place.tz), p.lon, now).map_err(|_| invalid("place"))
}

fn birth_at(place: &HomePlace, date: &str, time: &str, fold: Option<FoldChoice>) -> BirthData {
    BirthData {
        date: date.to_owned(),
        time: Some(time.to_owned()),
        time_certainty: TimeCertainty::Exact,
        place: place.place.clone(),
        tz: place.tz.clone(),
        fold,
        notes: String::new(),
    }
}

/// Prashnanın doğum verisi (soru anı ve yeri) ve haritanın etiketi. Denetim sırası: yer, sayı,
/// odak evi, soru, an. `Now` anı yerin duvar saatine çevrilir; duvar saati yaz saati
/// örtüşmesine düşüyorsa `now`'ı veren taraf seçilir. Girilen duvar saati boşluğa ya da
/// örtüşmeye düşüyorsa ya da okunamıyorsa `at` alanıyla reddedilir (sessizce kaydırılmaz).
pub fn prashna_birth(
    request: &PrashnaRequest,
    now: Timestamp,
) -> Result<(BirthData, String), CommandError> {
    let offset = place_offset(&request.place, now)?;
    if request
        .number
        .is_some_and(|n| !(1..=PRASHNA_NUMBER_MAX).contains(&n))
    {
        return Err(invalid("number"));
    }
    if request.focus_house.is_some_and(|h| !(1..=12).contains(&h)) {
        return Err(invalid("focus_house"));
    }
    if request.question.trim().chars().count() > MAX_QUESTION_CHARS {
        return Err(invalid("question"));
    }
    let birth = match &request.at {
        PrashnaMoment::Now => {
            let local = Timestamp::from_second(now.as_second() + i64::from(offset))
                .map_err(|_| invalid("at"))?;
            let date = local.strftime("%Y-%m-%d").to_string();
            let time = local.strftime("%H:%M:%S").to_string();
            // Örtüşmede iki an vardır: `now`'ı veren taraf seçilir. Diğer her durumda katlama
            // seçimi gereksizdir ve kayda yazılmaz. Yerel ortalama saatte ofset saniyeye
            // yuvarlanır: çözülen an `now`'dan bir saniyeye dek ayrışabilir.
            let wanted = now.as_second();
            let hits = |fold: Option<FoldChoice>| {
                store::resolve_birth(&birth_at(&request.place, &date, &time, fold))
                    .ok()
                    .and_then(|(_, r)| r.utc.parse::<Timestamp>().ok())
                    .is_some_and(|t| (t.as_second() - wanted).abs() <= 1)
            };
            let fold = [None, Some(FoldChoice::Earlier), Some(FoldChoice::Later)]
                .into_iter()
                .find(|f| hits(*f))
                .ok_or_else(|| invalid("at"))?;
            birth_at(&request.place, &date, &time, fold)
        }
        PrashnaMoment::Local { date, time } => birth_at(&request.place, date, time, None),
    };
    let (canonical, _) = store::resolve_birth(&birth).map_err(store_error)?;
    let clock = canonical.time.as_deref().unwrap_or_default();
    let label = format!(
        "{PRASHNA_LABEL} {} {}",
        canonical.date,
        clock.get(..5).unwrap_or(clock)
    );
    Ok((canonical, label))
}

/// Prashna haritasını kaydeder; dönen çift danışanın ve haritanın kimliğidir.
pub fn create_prashna_record(
    store: &Mutex<Store>,
    request: &PrashnaRequest,
    now: Timestamp,
) -> Result<(u32, u32), CommandError> {
    let (birth, label) = prashna_birth(request, now)?;
    let input = NewPrashna {
        client_id: request.client_id,
        birth,
        number: request.number,
        question: request.question.clone(),
        focus_house: request.focus_house,
    };
    let (client_id, chart) = lock(store)?
        .create_prashna(&input, &label)
        .map_err(store_error)?;
    Ok((client_id, chart.id))
}

/// `create_prashna` gövdesi; `now` testte sabitlenir. Bloklar.
pub fn create_prashna_blocking(
    engine: &Engine,
    store: &Mutex<Store>,
    request: PrashnaRequest,
    now: Timestamp,
) -> Result<OpenedClient, CommandError> {
    let (client_id, chart_id) = create_prashna_record(store, &request, now)?;
    open_client_blocking(engine, store, client_id, Some(chart_id))
}

/// Soru anının haritasını kurar ve açar. An "şimdi" ya da sorunun yerinin duvar saatidir; yer
/// haritanın yeri olarak saklanır. Harita `Event` türündedir ve genel varsayılan ayarların
/// kopyasını alır. Soru metni, yer ve an loglanmaz; hatalar yalnızca alan adı taşır.
#[tauri::command]
#[specta::specta]
pub async fn create_prashna(
    engine: State<'_, Arc<Engine>>,
    store: State<'_, StoreState>,
    request: PrashnaRequest,
) -> Result<OpenedClient, CommandError> {
    let (engine, store) = (Arc::clone(&engine), Arc::clone(&store));
    blocking(move || create_prashna_blocking(&engine, &store, request, Timestamp::now())).await
}

/// `update_prashna` gövdesi.
pub fn update_prashna_body(
    store: &Mutex<Store>,
    chart_id: u32,
    edit: &PrashnaEdit,
) -> Result<PrashnaMeta, CommandError> {
    lock(store)?
        .update_prashna(chart_id, &edit.question, edit.focus_house)
        .map_err(store_error)
}

/// Açık prashnanın soru metnini ve odak evini kaydeder (otomatik kayıt). Harita yeniden
/// hesaplanmaz: ikisi de lagnayı ve grahaları etkilemez.
#[tauri::command]
#[specta::specta]
pub async fn update_prashna(
    store: State<'_, StoreState>,
    chart_id: u32,
    edit: PrashnaEdit,
) -> Result<PrashnaMeta, CommandError> {
    let store = Arc::clone(&store);
    blocking(move || update_prashna_body(&store, chart_id, &edit)).await
}

/// `compute_prashna_facts` komutunun gövdesi; bloklar.
pub fn compute_prashna_facts_blocking(
    engine: &Engine,
    input: ChartInput,
    focus_house: Option<u8>,
) -> Result<PrashnaFacts, CommandError> {
    let facts = engine.call(move |se: &mut SwissEph| {
        jyotish_core::compute_prashna_facts(se, &input, focus_house)
    })??;
    Ok(facts)
}

/// Soru haritasının olguları: lagna ve lordu, odak evinin lordu ve iki lordun bağı, Ay,
/// kendralardaki grahalar ve yoga listesi. `input` açık haritanın girdisidir (sayı
/// prashnasında `lagna_number` doludur); `focus_house` astrologun seçtiği evdir. Sonuç
/// saklanmaz.
#[tauri::command]
#[specta::specta]
pub async fn compute_prashna_facts(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
    focus_house: Option<u8>,
) -> Result<PrashnaFacts, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_prashna_facts_blocking(&engine, input, focus_house)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use store::Place;

    const QUESTION: &str = "Kayıp anahtar bulunacak mı?";

    fn place(name: &str, lat: f64, lon: f64, tz: TzChoice) -> HomePlace {
        HomePlace {
            place: Place {
                name: name.into(),
                country: String::new(),
                admin1: String::new(),
                lat,
                lon,
                geoname_id: None,
            },
            tz,
        }
    }

    fn berlin() -> HomePlace {
        place(
            "Berlin",
            52.52,
            13.405,
            TzChoice::Iana("Europe/Berlin".into()),
        )
    }

    fn request(at: PrashnaMoment, place: HomePlace) -> PrashnaRequest {
        PrashnaRequest {
            client_id: None,
            at,
            place,
            number: None,
            question: QUESTION.into(),
            focus_house: Some(4),
        }
    }

    fn ts(text: &str) -> Timestamp {
        text.parse().expect("timestamp")
    }

    fn local(date: &str, time: &str) -> PrashnaMoment {
        PrashnaMoment::Local {
            date: date.into(),
            time: time.into(),
        }
    }

    fn utc_of(birth: &BirthData) -> String {
        store::resolve_birth(birth).expect("resolves").1.utc
    }

    #[test]
    fn now_becomes_the_wall_clock_of_the_place() {
        // 2026-11-27 12:34:56 UTC; Berlin kışın UTC+1.
        let now = ts("2026-11-27T12:34:56Z");
        let (birth, label) = prashna_birth(&request(PrashnaMoment::Now, berlin()), now).unwrap();
        assert_eq!(birth.date, "2026-11-27");
        assert_eq!(birth.time.as_deref(), Some("13:34:56"));
        assert_eq!(birth.time_certainty, TimeCertainty::Exact);
        assert_eq!(birth.fold, None);
        assert_eq!(birth.place.name, "Berlin");
        assert_eq!(label, "Prashna 2026-11-27 13:34");
        assert_eq!(utc_of(&birth), "2026-11-27T12:34:56Z");
        // Yerel ortalama saat: boylam 77,209° → +5 sa 08 dk 50 sn.
        let lmt = place("", 28.6139, 77.209, TzChoice::Lmt);
        let (birth, label) = prashna_birth(&request(PrashnaMoment::Now, lmt), now).unwrap();
        assert_eq!(birth.time.as_deref(), Some("17:43:46"));
        assert_eq!(label, "Prashna 2026-11-27 17:43");
        // Ofset saniyeye yuvarlanır (18 530,16 sn): çözülen an bir saniyenin içindedir.
        let resolved: Timestamp = utc_of(&birth).parse().unwrap();
        assert!((resolved.as_second() - now.as_second()).abs() <= 1);
    }

    #[test]
    fn now_inside_an_overlap_keeps_the_instant() {
        // Berlin 2026-10-25: 03:00 CEST'te saat 02:00 CET'e alınır; 02:30 iki kez yaşanır.
        for (utc, fold) in [
            ("2026-10-25T00:30:00Z", FoldChoice::Earlier),
            ("2026-10-25T01:30:00Z", FoldChoice::Later),
        ] {
            let (birth, _) =
                prashna_birth(&request(PrashnaMoment::Now, berlin()), ts(utc)).unwrap();
            assert_eq!(birth.time.as_deref(), Some("02:30:00"), "{utc}");
            assert_eq!(birth.fold, Some(fold), "{utc}");
            assert_eq!(utc_of(&birth), utc);
        }
    }

    #[test]
    fn a_typed_moment_is_the_wall_clock_of_the_place() {
        let now = ts("2026-11-27T12:00:00Z");
        let (birth, label) =
            prashna_birth(&request(local("2026-06-01", "14:05"), berlin()), now).unwrap();
        assert_eq!(
            (birth.date.as_str(), birth.time.as_deref()),
            ("2026-06-01", Some("14:05:00"))
        );
        assert_eq!(label, "Prashna 2026-06-01 14:05");
        // Yazın Berlin UTC+2.
        assert_eq!(utc_of(&birth), "2026-06-01T12:05:00Z");
    }

    #[test]
    fn errors_name_the_field_and_never_carry_the_place_the_moment_or_the_question() {
        let now = ts("2026-11-27T12:00:00Z");
        let field = |r: &PrashnaRequest| {
            let e = prashna_birth(r, now).unwrap_err();
            assert_eq!(e.code, "invalid_prashna_input");
            // Yalnızca alan adı: yer, an ve soru hata parametresine girmez.
            assert_eq!(e.params.len(), 1);
            let text = format!("{e:?}");
            for private in [
                "Berlin",
                "Europe",
                "2026-03-29",
                "02:30",
                "anahtar",
                "Mars/",
            ] {
                assert!(!text.contains(private), "{private}");
            }
            e.params["field"].clone()
        };
        // Yer: koordinat, bilinmeyen dilim, aralık dışı sabit ofset, fazla uzun ad.
        let mut bad = request(PrashnaMoment::Now, berlin());
        bad.place.place.lat = 91.0;
        assert_eq!(field(&bad), "place");
        bad.place.place.lat = f64::NAN;
        assert_eq!(field(&bad), "place");
        let zone = place(
            "Berlin",
            52.52,
            13.405,
            TzChoice::Iana("Mars/Olympus".into()),
        );
        assert_eq!(field(&request(PrashnaMoment::Now, zone)), "place");
        let offset = place("Berlin", 52.52, 13.405, TzChoice::Fixed(100 * 3600));
        assert_eq!(field(&request(PrashnaMoment::Now, offset)), "place");
        let mut long = request(PrashnaMoment::Now, berlin());
        long.place.place.name = "x".repeat(MAX_NAME_CHARS + 1);
        assert_eq!(field(&long), "place");
        // Yer her şeyden önce denetlenir: an da sayı da geçersizken alan `place`'tir.
        let mut all = request(local("bozuk", "25:99"), berlin());
        all.place.place.lon = 181.0;
        all.number = Some(0);
        assert_eq!(field(&all), "place");
        // Sayı, odak evi ve soru andan önce denetlenir.
        let mut r = request(local("bozuk", "25:99"), berlin());
        r.number = Some(109);
        assert_eq!(field(&r), "number");
        r.number = Some(0);
        assert_eq!(field(&r), "number");
        r.number = Some(108);
        r.focus_house = Some(13);
        assert_eq!(field(&r), "focus_house");
        r.focus_house = None;
        r.question = "ş".repeat(MAX_QUESTION_CHARS + 1);
        assert_eq!(field(&r), "question");
        r.question = QUESTION.into();
        // An: bozuk tarih ya da saat; yaz saati boşluğu (Berlin 2026-03-29 02:30 yoktur);
        // örtüşme (2026-10-25 02:30 iki kez yaşanır); hesap aralığının dışı.
        assert_eq!(field(&r), "at");
        for (date, time) in [
            ("2026-13-01", "10:00"),
            ("2026-03-29", "02:30"),
            ("2026-10-25", "02:30"),
            ("9999-01-01", "10:00"),
            ("", ""),
        ] {
            assert_eq!(field(&request(local(date, time), berlin())), "at", "{date}");
        }
    }

    #[test]
    fn the_record_is_written_under_the_client_or_the_built_in_one() {
        let store = Mutex::new(Store::open_in_memory().expect("store"));
        let now = ts("2026-11-27T12:34:56Z");
        let mut r = request(PrashnaMoment::Now, berlin());
        r.number = Some(54);
        let (book, chart_id) = create_prashna_record(&store, &r, now).unwrap();
        {
            let s = store.lock().unwrap();
            assert_eq!(s.prashna_client_id().unwrap(), Some(book));
            let chart = s.chart(chart_id).unwrap();
            assert_eq!(chart.label, "Prashna 2026-11-27 13:34");
            assert_eq!(chart.resolved.utc, "2026-11-27T12:34:56Z");
            let meta = s.prashna_meta(chart_id).unwrap().unwrap();
            assert_eq!(meta.question, QUESTION);
            assert_eq!((meta.number, meta.focus_house), (Some(54), Some(4)));
        }
        // Seçili danışan verilirse ona eklenir.
        r.client_id = Some(book);
        r.at = local("2026-06-01", "14:05");
        let (owner, second) = create_prashna_record(&store, &r, now).unwrap();
        assert_eq!(owner, book);
        assert_ne!(second, chart_id);
        // Olmayan danışan `not_found`'dur; kimlikten başka bir şey taşımaz.
        r.client_id = Some(9999);
        let e = create_prashna_record(&store, &r, now).unwrap_err();
        assert_eq!(e.code, "not_found");
        assert_eq!(e.params["what"], "client");
        // Düzenleme: soru ve odak evi; alan hataları yalnızca alan adı taşır.
        let edit = PrashnaEdit {
            question: "Yeni soru".into(),
            focus_house: None,
        };
        let meta = update_prashna_body(&store, chart_id, &edit).unwrap();
        assert_eq!(
            (meta.question.as_str(), meta.focus_house, meta.number),
            ("Yeni soru", None, Some(54))
        );
        for (edit, field) in [
            (
                PrashnaEdit {
                    question: "ş".repeat(MAX_QUESTION_CHARS + 1),
                    focus_house: None,
                },
                "question",
            ),
            (
                PrashnaEdit {
                    question: String::new(),
                    focus_house: Some(13),
                },
                "focus_house",
            ),
        ] {
            let e = update_prashna_body(&store, chart_id, &edit).unwrap_err();
            assert_eq!(e.code, "invalid_prashna_input");
            assert_eq!(e.params.len(), 1);
            assert_eq!(e.params["field"], field);
        }
        let e = update_prashna_body(&store, 9999, &edit_of("x")).unwrap_err();
        assert_eq!(e.code, "not_found");
        assert_eq!(e.params["what"], "prashna");
    }

    fn edit_of(question: &str) -> PrashnaEdit {
        PrashnaEdit {
            question: question.into(),
            focus_house: None,
        }
    }
}
