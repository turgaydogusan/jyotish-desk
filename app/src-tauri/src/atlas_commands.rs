//! Doğum formu komutları (M5): atlas yer araması ve kaydetmeden an önizlemesi. İnce
//! katman: arama `atlas`'ta, an çözümü `store::resolve_birth`'te; burada yalnızca sınırlar
//! ve hata eşleme var.

use std::path::Path;
use std::sync::{Arc, Mutex};

use atlas::{Atlas, AtlasError};
use serde::Serialize;
use specta::Type;
use store::{BirthData, ResolvedRecord};
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::CommandError;

/// Paketlenen atlas dosyasının kaynak yolu (`tauri.conf.json` `bundle.resources`).
pub const ATLAS_RESOURCE: &str = "atlas/atlas.sqlite";
/// Bir aramada dönen en çok yer.
pub const MAX_PLACE_HITS: u32 = 20;
/// Arama metninin en çok uzunluğu (karakter); fazlası kesilir.
pub const MAX_QUERY_CHARS: usize = 100;

/// Atlas durumu. Dosya açılamazsa uygulama yine açılır: yer araması `atlas_unavailable`
/// döner, elle koordinat girişi çalışır (spec §12 "manuel koordinat birinci sınıf").
/// Atlasın neden açılamadığı (Plan 11 D18). Yol taşımaz (C1, C2′).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtlasUnavailable {
    /// Dosya yok (paket eksik ya da geliştirme kopyasında atlas üretilmemiş).
    Missing,
    /// Dosya var ama şema beklenen değil ya da SQLite veritabanı değil.
    Schema,
    /// Okuma hatası (izin, aygıt).
    Io,
}

impl AtlasUnavailable {
    fn code(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Schema => "schema",
            Self::Io => "io",
        }
    }
}

pub struct AtlasState(Result<Mutex<Atlas>, AtlasUnavailable>);

impl AtlasState {
    pub fn open(path: &Path) -> Self {
        if !path.exists() {
            return Self(Err(AtlasUnavailable::Missing));
        }
        Self(Atlas::open(path).map(Mutex::new).map_err(|e| match e {
            AtlasError::Io(_) => AtlasUnavailable::Io,
            AtlasError::Sqlite(_) | AtlasError::SchemaMismatch { .. } | AtlasError::Geonames(_) => {
                AtlasUnavailable::Schema
            }
        }))
    }

    pub fn unavailable() -> Self {
        Self(Err(AtlasUnavailable::Missing))
    }

    pub fn is_available(&self) -> bool {
        self.0.is_ok()
    }

    fn get(&self) -> Result<&Mutex<Atlas>, CommandError> {
        self.0
            .as_ref()
            .map_err(|r| CommandError::new("atlas_unavailable", &[("reason", r.code().to_owned())]))
    }
}

/// Arama sonucu: formun dolduracağı alanlar.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct PlaceHit {
    pub geoname_id: u32,
    pub name: String,
    pub admin1: String,
    pub country: String,
    pub lat: f64,
    pub lon: f64,
    /// IANA kimliği; form saat dilimini buradan önerir.
    pub timezone: String,
}

/// Kaydetmeden çözülmüş an: kanonik doğum verisi ve `ResolvedRecord`.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct MomentPreview {
    pub birth: BirthData,
    pub resolved: ResolvedRecord,
}

impl From<AtlasError> for CommandError {
    fn from(e: AtlasError) -> Self {
        match e {
            AtlasError::Sqlite(e) => Self::new("atlas_sqlite", &[("detail", e.to_string())]),
            AtlasError::Io(e) => Self::new("atlas_io", &[("detail", e.to_string())]),
            AtlasError::Geonames(e) => Self::new("atlas_io", &[("detail", e.to_string())]),
            AtlasError::SchemaMismatch { found, expected } => Self::new(
                "atlas_schema",
                &[
                    ("found", found.unwrap_or_default()),
                    ("expected", expected.to_owned()),
                ],
            ),
        }
    }
}

/// Önek araması (`Atlas::search`). Boş sorgu boş liste; `limit` `MAX_PLACE_HITS`'e kırpılır.
pub fn search_places_blocking(
    atlas: &AtlasState,
    query: &str,
    limit: u32,
) -> Result<Vec<PlaceHit>, CommandError> {
    let atlas = atlas.get()?;
    let query: String = query.trim().chars().take(MAX_QUERY_CHARS).collect();
    let limit = limit.min(MAX_PLACE_HITS) as usize;
    if query.is_empty() || limit == 0 {
        return Ok(Vec::new());
    }
    let places = atlas
        .lock()
        .map_err(|_| CommandError::new("atlas_unavailable", &[("reason", "io".to_owned())]))?
        .search(&query, limit)?;
    Ok(places
        .into_iter()
        .filter_map(|p| {
            Some(PlaceHit {
                geoname_id: u32::try_from(p.geoname_id).ok()?,
                name: p.name,
                admin1: p.admin1,
                country: p.country,
                lat: p.lat,
                lon: p.lon,
                timezone: p.timezone,
            })
        })
        .collect())
}

/// Doğum verisini kaydetmeden çözer (DST boşluğu ve örtüşmesi `create_client` ile aynı
/// hata kodlarıyla döner). Saf ve hızlıdır; store kilidi alınmaz.
pub fn preview_moment_body(birth: &BirthData) -> Result<MomentPreview, CommandError> {
    let (birth, resolved) = store::resolve_birth(birth)?;
    Ok(MomentPreview { birth, resolved })
}

#[tauri::command]
#[specta::specta]
pub async fn search_places(
    atlas: State<'_, Arc<AtlasState>>,
    query: String,
    limit: u32,
) -> Result<Vec<PlaceHit>, CommandError> {
    let atlas = Arc::clone(&atlas);
    blocking(move || search_places_blocking(&atlas, &query, limit)).await
}

#[tauri::command]
#[specta::specta]
pub fn preview_moment(birth: BirthData) -> Result<MomentPreview, CommandError> {
    preview_moment_body(&birth)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use store::{Place, TimeCertainty, TzChoice};

    fn shipped_atlas() -> AtlasState {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/atlas/atlas.sqlite");
        let state = AtlasState::open(&p);
        assert!(state.is_available(), "data/atlas/atlas.sqlite must open");
        state
    }

    fn birth(date: &str, time: Option<&str>, tz: TzChoice) -> BirthData {
        BirthData {
            date: date.into(),
            time: time.map(Into::into),
            time_certainty: if time.is_some() {
                TimeCertainty::Exact
            } else {
                TimeCertainty::Unknown
            },
            place: Place {
                name: "New Delhi".into(),
                country: "India".into(),
                admin1: "NCT".into(),
                lat: 28.6139,
                lon: 77.2090,
                geoname_id: Some(1_261_481),
            },
            tz,
            fold: None,
            notes: String::new(),
        }
    }

    #[test]
    fn search_finds_new_delhi_with_its_time_zone_and_clamps_the_limit() {
        let atlas = shipped_atlas();
        let hits = search_places_blocking(&atlas, "  new delhi ", 5).unwrap();
        let first = hits.first().expect("a hit");
        assert_eq!(first.name, "New Delhi");
        assert_eq!(first.timezone, "Asia/Kolkata");
        assert!((first.lat - 28.6).abs() < 0.1 && (first.lon - 77.2).abs() < 0.1);
        assert!(hits.len() <= 5);
        let many = search_places_blocking(&atlas, "san", 500).unwrap();
        assert_eq!(many.len(), MAX_PLACE_HITS as usize);
    }

    #[test]
    fn empty_query_and_zero_limit_return_nothing_and_long_queries_are_cut() {
        let atlas = shipped_atlas();
        assert!(search_places_blocking(&atlas, "   ", 8).unwrap().is_empty());
        assert!(search_places_blocking(&atlas, "delhi", 0)
            .unwrap()
            .is_empty());
        let long = "a".repeat(10_000);
        assert!(search_places_blocking(&atlas, &long, 8).unwrap().is_empty());
    }

    #[test]
    fn missing_atlas_is_reported_not_fatal() {
        let state = AtlasState::open(Path::new("/nonexistent/atlas.sqlite"));
        assert!(!state.is_available());
        let e = search_places_blocking(&state, "delhi", 8).unwrap_err();
        assert_eq!(e.code, "atlas_unavailable");
        assert!(!AtlasState::unavailable().is_available());
    }

    #[test]
    fn preview_resolves_without_a_store() {
        let p = preview_moment_body(&birth(
            "2000-01-01",
            Some("17:30"),
            TzChoice::Iana("Asia/Kolkata".into()),
        ))
        .unwrap();
        assert_eq!(p.resolved.jd_ut, 2_451_545.0);
        assert_eq!(p.resolved.offset_secs, 19_800);
        assert_eq!(p.resolved.abbrev, "IST");
        assert_eq!(p.birth.time.as_deref(), Some("17:30:00"));
        let noon =
            preview_moment_body(&birth("2000-01-01", None, TzChoice::Fixed(19_800))).unwrap();
        assert_eq!(
            noon.resolved.warnings,
            [store::MomentWarning::TimeUnknownNoon]
        );
    }

    #[test]
    fn preview_reports_dst_gap_and_overlap_with_store_error_codes() {
        let mut gap = birth(
            "2014-03-31",
            Some("03:30"),
            TzChoice::Iana("Europe/Istanbul".into()),
        );
        gap.place.lon = 28.94966;
        let e = preview_moment_body(&gap).unwrap_err();
        assert_eq!(e.code, "nonexistent_local_time");
        assert_eq!(e.params["suggestion"], "2014-03-31T04:30:00");
        let mut overlap = birth(
            "2014-10-26",
            Some("03:30"),
            TzChoice::Iana("Europe/Istanbul".into()),
        );
        overlap.place.lon = 28.94966;
        let e = preview_moment_body(&overlap).unwrap_err();
        assert_eq!(e.code, "ambiguous_local_time");
        overlap.fold = Some(store::FoldChoice::Later);
        assert_eq!(
            preview_moment_body(&overlap).unwrap().resolved.offset_secs,
            7_200
        );
        let mut bad = birth("2000-13-01", Some("10:00"), TzChoice::Lmt);
        assert_eq!(
            preview_moment_body(&bad).unwrap_err().params["field"],
            "birth.date"
        );
        bad.date = "2000-01-01".into();
        bad.place.lat = 91.0;
        assert_eq!(
            preview_moment_body(&bad).unwrap_err().params["field"],
            "birth.place.lat"
        );
    }

    #[test]
    fn the_atlas_is_bundled_at_the_resource_path() {
        let conf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(conf).unwrap()).unwrap();
        assert_eq!(
            v["bundle"]["resources"]["../../data/atlas/atlas.sqlite"],
            ATLAS_RESOURCE
        );
    }

    #[test]
    fn atlas_error_codes_are_stable() {
        let e = CommandError::from(AtlasError::SchemaMismatch {
            found: Some("0".into()),
            expected: "1",
        });
        assert_eq!(
            (e.code.as_str(), e.params["found"].as_str()),
            ("atlas_schema", "0")
        );
        let e = CommandError::from(AtlasError::Io(std::io::Error::other("x")));
        assert_eq!(e.code, "atlas_io");
    }

    #[test]
    fn unavailable_atlas_reports_why_without_a_path() {
        let dir = tempfile::tempdir().unwrap();
        let missing = AtlasState::open(&dir.path().join("atlas.sqlite"));
        let e = search_places_blocking(&missing, "delhi", 5).unwrap_err();
        assert_eq!(
            (e.code.as_str(), e.params["reason"].as_str()),
            ("atlas_unavailable", "missing")
        );
        assert_eq!(e.params.len(), 1);
        let junk = dir.path().join("junk.sqlite");
        std::fs::write(&junk, b"not a database").unwrap();
        let e = search_places_blocking(&AtlasState::open(&junk), "delhi", 5).unwrap_err();
        assert!(
            ["schema", "io"].contains(&e.params["reason"].as_str()),
            "{:?}",
            e.params
        );
        assert_eq!(
            search_places_blocking(&AtlasState::unavailable(), "delhi", 5)
                .unwrap_err()
                .params["reason"],
            "missing"
        );
    }
}
