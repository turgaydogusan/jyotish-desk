//! Kayıt yönetimi komutları (danışan, harita, not, etiket, tercihler) ve dosya içe/dışa
//! aktarma. Harita hesabı gerektirmez; gövdeler `store`'dadır.

use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use jyotish_core::ChartInput;
use serde::Serialize;
use specta::Type;
use store::{
    AppPrefs, ChartBrief, ChartRecord, Client, ExportOptions, Gender, ImportReport, Note,
    PanelLayout, PrefsLoad, SearchHit, Store, TagCount,
};
use tauri::{AppHandle, State, WebviewWindow};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::client_commands::{blocking, lock, with_store, StoreState};
use crate::commands::CommandError;

/// JHD dosyası başına hata.
#[derive(Debug, Clone, Serialize, Type)]
pub struct FileFailure {
    pub file: String,
    pub error: CommandError,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct JhdImportResult {
    pub report: ImportReport,
    pub failed: Vec<FileFailure>,
}

/// İçe aktarılan JSON dosyasının sınırı; dosya okunmadan önce boyutuyla denetlenir.
// Değişirse data.exportedOverLimit metinleri de değişir.
pub const MAX_IMPORT_BYTES: u64 = 32 * 1024 * 1024;

/// Hata parametrelerinde yalnızca dosya adı bulunur: tam yol kullanıcı adını taşır.
fn base_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn io_error(path: &Path, e: std::io::Error) -> CommandError {
    CommandError::new(
        "file_io",
        &[
            ("file", base_name(path)),
            ("detail", io_detail(e.kind()).to_owned()),
        ],
    )
}

/// `ErrorKind` → kararlı kod (D16). Metin İngilizcedir ve sürümler arasında değişebilir; arayüz
/// kodu kendi dilinde gösterir (`errors.fileIoDetail`).
fn io_detail(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NotFound => "not_found",
        ErrorKind::PermissionDenied => "permission_denied",
        ErrorKind::AlreadyExists => "already_exists",
        ErrorKind::InvalidInput => "invalid_input",
        ErrorKind::InvalidData => "invalid_data",
        ErrorKind::StorageFull => "storage_full",
        _ => "other",
    }
}

/// Geçici dosya adı için denenen ad sayısı (ad doluysa sıradaki denenir).
const TEMP_ATTEMPTS: u32 = 16;

/// Süreç içinde her yazım ayrı bir geçici ad alır: eşzamanlı iki dışa aktarma aynı dosyayı
/// paylaşmaz.
static TEMP_SEQ: AtomicU32 = AtomicU32::new(0);

/// Ad hedefin adından türemez: uzun bir hedef adı dosya adı sınırını aşmaz.
fn temp_name(n: u32) -> String {
    format!(".jyotish-desk-{}-{n}.tmp", std::process::id())
}

/// Hedefin klasöründe **yeni** bir geçici dosya açar. Var olan bir dosyanın ya da bağlantının
/// üzerine yazılmaz (`create_new`); Unix'te dosya yalnızca sahibine açıktır (dışa aktarma not
/// içerebilir).
fn create_temp(target: &Path, seq: &AtomicU32) -> std::io::Result<(PathBuf, File)> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut taken = ErrorKind::AlreadyExists.into();
    for _ in 0..TEMP_ATTEMPTS {
        let tmp = target.with_file_name(temp_name(seq.fetch_add(1, Ordering::Relaxed)));
        match options.open(&tmp) {
            Ok(f) => return Ok((tmp, f)),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => taken = e,
            Err(e) => return Err(e),
        }
    }
    Err(taken)
}

/// Aynı klasörde geçici dosyaya yazıp yerine taşır: yarıda kesilen yazım eski dosyayı bozmaz.
pub fn write_atomic(path: &Path, text: &str) -> Result<(), CommandError> {
    write_atomic_with(path, text, &TEMP_SEQ)
}

fn write_atomic_with(path: &Path, text: &str, seq: &AtomicU32) -> Result<(), CommandError> {
    // Yalnızca bu çağrının açtığı geçici dosya silinir.
    let (tmp, mut f) = create_temp(path, seq).map_err(|e| io_error(path, e))?;
    let written = (|| {
        f.write_all(text.as_bytes())?;
        f.sync_all()?;
        std::fs::rename(&tmp, path)
    })();
    written.map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        io_error(path, e)
    })?;
    sync_parent(path);
    Ok(())
}

/// Taşımanın kendisi de diske yazılsın (Unix: klasör girdisi). Yalnızca dayanıklılıktır: dosya
/// zaten yerindedir, hata yok sayılır.
fn sync_parent(path: &Path) {
    // Öznitelik `if` ifadesine doğrudan yazılamaz (derlenmez); blok sarılır.
    #[cfg(unix)]
    {
        let dir = path.parent().filter(|d| !d.as_os_str().is_empty());
        if let Some(Ok(d)) = dir.map(File::open) {
            let _ = d.sync_all();
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}

/// `MAX_IMPORT_BYTES`'tan büyük dosya okunmaz (okurken büyüyen dosya da reddedilir). Sıradan
/// dosya olmayan yol (klasör, adlandırılmış boru, aygıt) açılmadan reddedilir: boru boyut
/// denetiminden 0 ile geçer ve açılması yazan biri gelene dek bekler. UTF-8 olmayan dosya
/// `import_format` (detail `utf8`) olur; baştaki BOM atılır.
pub fn read_bounded(path: &Path) -> Result<String, CommandError> {
    let too_large = || {
        CommandError::new(
            "import_too_large",
            &[
                ("file", base_name(path)),
                ("max_mb", (MAX_IMPORT_BYTES / (1024 * 1024)).to_string()),
            ],
        )
    };
    let meta = std::fs::metadata(path).map_err(|e| io_error(path, e))?;
    if !meta.is_file() {
        return Err(io_error(path, ErrorKind::InvalidInput.into()));
    }
    if meta.len() > MAX_IMPORT_BYTES {
        return Err(too_large());
    }
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|f| f.take(MAX_IMPORT_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|e| io_error(path, e))?;
    if bytes.len() as u64 > MAX_IMPORT_BYTES {
        return Err(too_large());
    }
    // Başındaki UTF-8 BOM (bazı düzenleyiciler ekler) atılır; UTF-8 olmayan dosya biçim hatasıdır.
    let body = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    String::from_utf8(body.to_vec())
        .map_err(|_| CommandError::new("import_format", &[("detail", "utf8".to_owned())]))
}

pub fn import_jhd_files_blocking(
    store: &Mutex<Store>,
    paths: &[String],
) -> Result<JhdImportResult, CommandError> {
    let mut files = Vec::with_capacity(paths.len());
    let mut failed = Vec::new();
    for p in paths {
        let path = Path::new(p);
        let name = base_name(path);
        match std::fs::read(path) {
            Ok(bytes) => files.push((name, bytes)),
            Err(e) => failed.push(FileFailure {
                file: name,
                error: io_error(path, e),
            }),
        }
    }
    let out = lock(store)?.import_jhd(&files)?;
    failed.extend(out.failed.into_iter().map(|(file, e)| FileFailure {
        file,
        error: e.into(),
    }));
    Ok(JhdImportResult {
        report: out.report,
        failed,
    })
}

pub fn export_json_file_blocking(
    store: &Mutex<Store>,
    path: &Path,
    options: &ExportOptions,
) -> Result<usize, CommandError> {
    let text = lock(store)?.export_json(options)?;
    write_atomic(path, &text)?;
    Ok(text.len())
}

pub fn export_csv_file_blocking(store: &Mutex<Store>, path: &Path) -> Result<usize, CommandError> {
    let text = lock(store)?.export_csv()?;
    write_atomic(path, &text)?;
    Ok(text.len())
}

/// Dışa aktarılan dosyanın adı ve boyutu; içe aktarma sınırını aşan dosya bu uygulamaya geri
/// yüklenemez (D15: dışa aktarma engellenmez, arayüz uyarır).
#[derive(Debug, Clone, Serialize, Type)]
pub struct ExportResult {
    pub file: String,
    /// `u32::MAX`'ta doyar (4 GiB).
    pub bytes: u32,
    pub over_import_limit: bool,
}

fn export_result(path: &Path, bytes: usize) -> ExportResult {
    ExportResult {
        file: base_name(path),
        bytes: u32::try_from(bytes).unwrap_or(u32::MAX),
        over_import_limit: bytes as u64 > MAX_IMPORT_BYTES,
    }
}

pub fn import_json_file_blocking(
    store: &Mutex<Store>,
    path: &Path,
) -> Result<ImportReport, CommandError> {
    let text = read_bounded(path)?;
    Ok(lock(store)?.import_json(&text)?)
}

/// Seçilen dosyanın adı ve içe aktarma özeti.
#[derive(Debug, Clone, Serialize, Type)]
pub struct ImportResult {
    pub file: String,
    pub report: ImportReport,
}

fn chosen(path: Option<FilePath>) -> Result<Option<PathBuf>, CommandError> {
    path.map(|p| {
        p.into_path()
            .map_err(|_| CommandError::new("dialog_path", &[]))
    })
    .transpose()
}

/// Kaydetme penceresi; uygulama penceresine bağlıdır (`set_parent`). `None`: kullanıcı vazgeçti.
fn save_dialog(
    app: &AppHandle,
    window: &WebviewWindow,
    filter: &str,
    ext: &str,
    name: &str,
) -> Result<Option<PathBuf>, CommandError> {
    chosen(
        app.dialog()
            .file()
            .set_parent(window)
            .add_filter(filter, &[ext])
            .set_file_name(name)
            .blocking_save_file(),
    )
}

#[tauri::command]
#[specta::specta]
pub async fn update_client(
    store: State<'_, StoreState>,
    client_id: u32,
    name: String,
    gender: Option<Gender>,
) -> Result<Client, CommandError> {
    with_store(&store, move |s| s.update_client(client_id, &name, gender)).await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_client(
    store: State<'_, StoreState>,
    client_id: u32,
) -> Result<(), CommandError> {
    with_store(&store, move |s| s.delete_client(client_id)).await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_chart(store: State<'_, StoreState>, chart_id: u32) -> Result<(), CommandError> {
    with_store(&store, move |s| s.delete_chart(chart_id)).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_clients(
    store: State<'_, StoreState>,
    limit: u32,
) -> Result<Vec<Client>, CommandError> {
    with_store(&store, move |s| s.list_clients(limit)).await
}

/// Kenar çubuğu etiket süzgeci: etiketin tam adıyla (büyük/küçük harf ve NFC duyarsız).
#[tauri::command]
#[specta::specta]
pub async fn list_clients_by_tag(
    store: State<'_, StoreState>,
    tag: String,
    limit: u32,
) -> Result<Vec<Client>, CommandError> {
    with_store(&store, move |s| s.clients_with_tag(&tag, limit)).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_charts(
    store: State<'_, StoreState>,
    client_id: u32,
) -> Result<Vec<ChartRecord>, CommandError> {
    with_store(&store, move |s| s.charts_of(client_id)).await
}

/// Bir haritanın modül girdisi (ileri modüller spec'i §9): çözülmüş an, konum, haritanın
/// hesap ayarı ve `time_known`. Ad, not ve yer adı dönmez. Harita yoksa `null` (silinmiş eş).
/// `chart_uuid` bir kayıt kimliğidir, dosya yolu ya da adres değildir (C2′).
#[tauri::command]
#[specta::specta]
pub async fn chart_input(
    store: State<'_, StoreState>,
    chart_uuid: String,
) -> Result<Option<ChartInput>, CommandError> {
    with_store(&store, move |s| s.chart_input(&chart_uuid)).await
}

/// Bir haritanın kısa kimliği: danışanın adı ve cinsiyeti, haritanın etiketi ve türü (uyum
/// panelinin başlığı ve ilk rol ataması). Harita yoksa `null`.
#[tauri::command]
#[specta::specta]
pub async fn chart_brief(
    store: State<'_, StoreState>,
    chart_uuid: String,
) -> Result<Option<ChartBrief>, CommandError> {
    with_store(&store, move |s| s.chart_brief(&chart_uuid)).await
}

#[tauri::command]
#[specta::specta]
pub async fn search_clients(
    store: State<'_, StoreState>,
    query: String,
    limit: u32,
) -> Result<Vec<SearchHit>, CommandError> {
    with_store(&store, move |s| s.search_clients(&query, limit)).await
}

#[tauri::command]
#[specta::specta]
pub async fn add_note(
    store: State<'_, StoreState>,
    client_id: u32,
    chart_id: Option<u32>,
    body: String,
) -> Result<Note, CommandError> {
    with_store(&store, move |s| s.add_note(client_id, chart_id, &body)).await
}

#[tauri::command]
#[specta::specta]
pub async fn update_note(
    store: State<'_, StoreState>,
    note_id: u32,
    body: String,
) -> Result<Note, CommandError> {
    with_store(&store, move |s| s.update_note(note_id, &body)).await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_note(store: State<'_, StoreState>, note_id: u32) -> Result<(), CommandError> {
    with_store(&store, move |s| s.delete_note(note_id)).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_notes(
    store: State<'_, StoreState>,
    client_id: u32,
    limit: u32,
) -> Result<Vec<Note>, CommandError> {
    with_store(&store, move |s| s.notes(client_id, limit)).await
}

#[tauri::command]
#[specta::specta]
pub async fn set_chart_tags(
    store: State<'_, StoreState>,
    chart_id: u32,
    tags: Vec<String>,
) -> Result<Vec<String>, CommandError> {
    with_store(&store, move |s| s.set_chart_tags(chart_id, &tags)).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_tags(store: State<'_, StoreState>) -> Result<Vec<TagCount>, CommandError> {
    with_store(&store, |s| s.tags()).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_app_prefs(store: State<'_, StoreState>) -> Result<PrefsLoad, CommandError> {
    with_store(&store, |s| s.app_prefs_load()).await
}

#[tauri::command]
#[specta::specta]
pub async fn set_app_prefs(
    store: State<'_, StoreState>,
    prefs: AppPrefs,
) -> Result<(), CommandError> {
    with_store(&store, move |s| s.set_app_prefs(&prefs)).await
}

/// Kullanıcının pano düzeni (`app_state`); kayıt yoksa ya da okunamazsa `None`.
#[tauri::command]
#[specta::specta]
pub async fn load_panel_layout(
    store: State<'_, StoreState>,
) -> Result<Option<PanelLayout>, CommandError> {
    with_store(&store, |s| s.panel_layout()).await
}

/// Açık harita olmadan düzen kaydı (otomatik kayıt); `None` kayıtlı düzeni siler.
#[tauri::command]
#[specta::specta]
pub async fn save_panel_layout(
    store: State<'_, StoreState>,
    layout: Option<PanelLayout>,
) -> Result<(), CommandError> {
    with_store(&store, move |s| s.save_panel_layout(layout.as_ref())).await
}

/// Bütün danışanlar JSON'a. Pencere Rust'ta açılır; arayüz yol vermez. Dönüş: kaydedilen
/// dosyanın adı ve boyutu (`ExportResult`), vazgeçilirse `None`. Notlar yalnızca `include_notes` ile (spec §5).
#[tauri::command]
#[specta::specta]
pub async fn export_clients_json(
    app: AppHandle,
    window: WebviewWindow,
    store: State<'_, StoreState>,
    include_notes: bool,
) -> Result<Option<ExportResult>, CommandError> {
    let store = Arc::clone(&store);
    blocking(move || {
        let Some(path) = save_dialog(&app, &window, "JSON", "json", EXPORT_JSON_NAME)? else {
            return Ok(None);
        };
        let options = ExportOptions {
            include_notes,
            client_ids: None,
        };
        let bytes = export_json_file_blocking(&store, &path, &options)?;
        Ok(Some(export_result(&path, bytes)))
    })
    .await
}

/// Bütün haritalar CSV'ye (yalnızca dışa aktarma; notlar CSV'de yoktur).
#[tauri::command]
#[specta::specta]
pub async fn export_clients_csv(
    app: AppHandle,
    window: WebviewWindow,
    store: State<'_, StoreState>,
) -> Result<Option<ExportResult>, CommandError> {
    let store = Arc::clone(&store);
    blocking(move || {
        let Some(path) = save_dialog(&app, &window, "CSV", "csv", EXPORT_CSV_NAME)? else {
            return Ok(None);
        };
        let bytes = export_csv_file_blocking(&store, &path)?;
        Ok(Some(export_result(&path, bytes)))
    })
    .await
}

/// JSON içe aktarma (hepsi ya da hiçbiri). Vazgeçilirse `None`.
#[tauri::command]
#[specta::specta]
pub async fn import_clients_json(
    app: AppHandle,
    window: WebviewWindow,
    store: State<'_, StoreState>,
) -> Result<Option<ImportResult>, CommandError> {
    let store = Arc::clone(&store);
    blocking(move || {
        let picked = app
            .dialog()
            .file()
            .set_parent(&window)
            .add_filter("JSON", &["json"])
            .blocking_pick_file();
        let Some(path) = chosen(picked)? else {
            return Ok(None);
        };
        let report = import_json_file_blocking(&store, &path)?;
        Ok(Some(ImportResult {
            file: base_name(&path),
            report,
        }))
    })
    .await
}

pub const EXPORT_JSON_NAME: &str = "jyotish-desk-clients.json";
pub const EXPORT_CSV_NAME: &str = "jyotish-desk-charts.csv";

#[cfg(test)]
mod tests {
    use super::*;

    fn populated() -> Mutex<Store> {
        let store = Store::open_in_memory().unwrap();
        let store = Mutex::new(store);
        let new = store::NewClient {
            name: "Ayşe".into(),
            gender: None,
            chart: store::NewChart {
                label: String::new(),
                kind: store::ChartKind::Natal,
                birth: store::BirthData {
                    date: "2000-01-01".into(),
                    time: Some("17:30".into()),
                    time_certainty: store::TimeCertainty::Exact,
                    place: store::Place {
                        name: "New Delhi".into(),
                        country: "India".into(),
                        admin1: "NCT".into(),
                        lat: 28.6139,
                        lon: 77.2090,
                        geoname_id: None,
                    },
                    tz: store::TzChoice::Iana("Asia/Kolkata".into()),
                    fold: None,
                    notes: String::new(),
                },
            },
        };
        let (c, _) = lock(&store).unwrap().create_client(&new).unwrap();
        lock(&store).unwrap().add_note(c.id, None, "gizli").unwrap();
        store
    }

    #[test]
    fn file_errors_name_only_the_file() {
        let store = Mutex::new(Store::open_in_memory().unwrap());
        let r = import_jhd_files_blocking(&store, &["/nonexistent/dir/a.jhd".into()]).unwrap();
        assert_eq!(r.failed.len(), 1);
        assert_eq!(r.failed[0].file, "a.jhd");
        assert_eq!(r.failed[0].error.code, "file_io");
        let e =
            import_json_file_blocking(&store, Path::new("/nonexistent/dir/x.json")).unwrap_err();
        assert_eq!(e.code, "file_io");
        assert_eq!(e.params["file"], "x.json");
        assert!(!e.params.values().any(|v| v.contains("/nonexistent")));
    }

    #[test]
    fn export_writes_atomically_and_round_trips_without_notes_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.json");
        std::fs::write(&path, "eski").unwrap();
        let src = populated();
        export_json_file_blocking(&src, &path, &ExportOptions::default()).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("Ayşe") && !text.contains("gizli"));
        // Geçici dosya kalmaz.
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
        let dst = Mutex::new(Store::open_in_memory().unwrap());
        let report = import_json_file_blocking(&dst, &path).unwrap();
        assert_eq!(
            (
                report.clients_added,
                report.charts_added,
                report.notes_added
            ),
            (1, 1, 0)
        );
        let csv = dir.path().join("out.csv");
        export_csv_file_blocking(&src, &csv).unwrap();
        assert!(std::fs::read_to_string(&csv).unwrap().contains("Ayşe"));
    }

    #[test]
    fn a_failed_write_leaves_the_old_file_and_no_temporary() {
        let dir = tempfile::tempdir().unwrap();
        // Hedef bir klasör: taşıma başarısız olur.
        let target = dir.path().join("taken");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("keep"), "x").unwrap();
        let e = write_atomic(&target, "yeni").unwrap_err();
        assert_eq!(e.code, "file_io");
        assert_eq!(e.params["file"], "taken");
        let names: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names, ["taken"]);
    }

    /// D2: pencere Rust'ta açılır; web görünümüne dosya penceresi ya da dosya sistemi izni
    /// verilmez (C2′).
    #[test]
    fn the_webview_is_granted_no_dialog_or_file_system_permission() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/capabilities");
        // Tauri `capabilities/**/*` ile gizli dosyaları da okur; yalnızca macOS'un üst veri
        // dosyaları (`.DS_Store`, `._*`) yok sayılır.
        assert!(is_capability_candidate(".extra.json"));
        assert!(is_capability_candidate(".d"));
        assert!(!is_capability_candidate(".DS_Store"));
        assert!(!is_capability_candidate("._default.json"));
        let names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| is_capability_candidate(n))
            .collect();
        assert_eq!(names, ["default.json"]);
        let v: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();
        assert_eq!(v["permissions"], serde_json::json!(["core:event:default"]));
        // Yetenek `tauri.conf.json` içinde satır içi de tanımlanamaz.
        let conf: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert!(conf["app"]["security"].get("capabilities").is_none());
        // Dosya sistemi eklentisi hiçbir kaynak dosyada kurulmaz: pencere eklentisinin
        // bağımlılığıdır, o kadar.
        // Aranan ad parçalardan kurulur: bu testin kendi metni onu içermez.
        let needle = ["tauri_plugin", "_fs"].concat();
        let src = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
        for e in std::fs::read_dir(src).unwrap() {
            let path = e.unwrap().path();
            if path.extension().is_some_and(|x| x == "rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                assert!(!text.contains(&needle), "{}", path.display());
            }
        }
    }

    fn is_capability_candidate(name: &str) -> bool {
        name != ".DS_Store" && !name.starts_with("._")
    }

    #[test]
    fn simultaneous_exports_to_one_target_all_succeed_and_leave_one_whole_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.json");
        let texts: Vec<String> = (0..8).map(|i| i.to_string().repeat(64 * 1024)).collect();
        for _ in 0..25 {
            std::thread::scope(|s| {
                let writers: Vec<_> = texts
                    .iter()
                    .map(|t| s.spawn(|| write_atomic(&path, t)))
                    .collect();
                for w in writers {
                    w.join().unwrap().unwrap();
                }
            });
        }
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(texts.contains(&text));
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn a_target_with_a_very_long_name_is_written() {
        let dir = tempfile::tempdir().unwrap();
        // 255 baytlık ad sınırına yakın: geçici dosyanın adı hedefin adından türeyemez.
        let path = dir.path().join(format!("{}.json", "a".repeat(245)));
        write_atomic(&path, "yeni").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "yeni");
    }

    #[cfg(unix)]
    #[test]
    fn an_exported_file_is_readable_by_its_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.json");
        let options = ExportOptions {
            include_notes: true,
            client_ids: None,
        };
        export_json_file_blocking(&populated(), &path, &options).unwrap();
        assert!(std::fs::read_to_string(&path).unwrap().contains("gizli"));
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn the_temporary_file_is_never_an_existing_file_or_link() {
        let dir = tempfile::tempdir().unwrap();
        let victim = dir.path().join("victim");
        std::fs::write(&victim, "dokunma").unwrap();
        let target = dir.path().join("out.json");
        // İlk iki ad dolu: biri başka bir dosyaya bağlantı, öbürü sıradan dosya.
        std::os::unix::fs::symlink(&victim, target.with_file_name(temp_name(0))).unwrap();
        std::fs::write(target.with_file_name(temp_name(1)), "başkasının").unwrap();
        let seq = AtomicU32::new(0);
        let (tmp, mut f) = create_temp(&target, &seq).unwrap();
        f.write_all(b"yeni").unwrap();
        assert_eq!(tmp, target.with_file_name(temp_name(2)));
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "dokunma");
        assert_eq!(
            std::fs::read_to_string(target.with_file_name(temp_name(1))).unwrap(),
            "başkasının"
        );
        // Bütün adlar doluysa hata döner; var olan dosyalar silinmez.
        let full = tempfile::tempdir().unwrap();
        let target = full.path().join("out.json");
        for n in 0..TEMP_ATTEMPTS {
            std::fs::write(target.with_file_name(temp_name(n)), "dolu").unwrap();
        }
        let e = write_atomic_with(&target, "yeni", &AtomicU32::new(0)).unwrap_err();
        assert_eq!(e.code, "file_io");
        assert_eq!(e.params["file"], "out.json");
        assert_eq!(
            std::fs::read_dir(full.path()).unwrap().count(),
            TEMP_ATTEMPTS as usize
        );
    }

    #[cfg(unix)]
    #[test]
    fn an_import_from_something_that_is_not_a_regular_file_is_refused_unread() {
        // Adlandırılmış boru ya da aygıt boyut denetiminden 0 ile geçer ve okuma hiç bitmeyebilir.
        let store = Mutex::new(Store::open_in_memory().unwrap());
        let e = import_json_file_blocking(&store, Path::new("/dev/null")).unwrap_err();
        assert_eq!(e.code, "file_io");
        assert_eq!(e.params["file"], "null");
        let dir = tempfile::tempdir().unwrap();
        let e = read_bounded(dir.path()).unwrap_err();
        assert_eq!(e.code, "file_io");
    }

    #[test]
    fn an_import_over_the_byte_cap_is_refused_before_reading() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.json");
        let f = File::create(&path).unwrap();
        f.set_len(MAX_IMPORT_BYTES + 1).unwrap();
        let store = Mutex::new(Store::open_in_memory().unwrap());
        let e = import_json_file_blocking(&store, &path).unwrap_err();
        assert_eq!(e.code, "import_too_large");
        assert_eq!(e.params["file"], "big.json");
        assert_eq!(e.params["max_mb"], "32");
        assert!(lock(&store).unwrap().list_clients(10).unwrap().is_empty());
    }

    #[test]
    fn an_import_that_is_not_utf8_is_a_format_error_and_a_bom_is_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let latin = dir.path().join("eski.json");
        std::fs::write(&latin, b"{\"name\": \"Ay\xfee\"}").unwrap();
        let e = read_bounded(&latin).unwrap_err();
        assert_eq!(
            (e.code.as_str(), e.params["detail"].as_str()),
            ("import_format", "utf8")
        );
        let bom = dir.path().join("bom.json");
        std::fs::write(&bom, "\u{feff}{\"a\":1}").unwrap();
        assert_eq!(read_bounded(&bom).unwrap(), "{\"a\":1}");
    }

    #[test]
    fn an_exported_backup_with_a_bom_imports() {
        let src = populated();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("yedek.json");
        let text = lock(&src)
            .unwrap()
            .export_json(&ExportOptions::default())
            .unwrap();
        std::fs::write(&path, format!("\u{feff}{text}")).unwrap();
        let dst = Mutex::new(Store::open_in_memory().unwrap());
        let report = import_json_file_blocking(&dst, &path).unwrap();
        assert_eq!(report.clients_added, 1);
    }

    #[test]
    fn file_errors_carry_a_stable_detail_code() {
        let e = io_error(Path::new("/x/a.json"), ErrorKind::PermissionDenied.into());
        assert_eq!(e.params["detail"], "permission_denied");
        let codes: Vec<_> = [
            ErrorKind::NotFound,
            ErrorKind::AlreadyExists,
            ErrorKind::InvalidInput,
            ErrorKind::InvalidData,
            ErrorKind::StorageFull,
            ErrorKind::Interrupted,
        ]
        .into_iter()
        .map(io_detail)
        .collect();
        assert_eq!(
            codes,
            [
                "not_found",
                "already_exists",
                "invalid_input",
                "invalid_data",
                "storage_full",
                "other"
            ]
        );
    }

    #[test]
    fn an_export_reports_its_size_and_whether_it_can_be_imported_again() {
        let r = export_result(Path::new("/x/yedek.json"), 1_000);
        assert_eq!(
            (r.file.as_str(), r.bytes, r.over_import_limit),
            ("yedek.json", 1_000, false)
        );
        let big = export_result(Path::new("/x/büyük.json"), MAX_IMPORT_BYTES as usize + 1);
        assert!(big.over_import_limit);
        assert_eq!(big.bytes as u64, MAX_IMPORT_BYTES + 1);
        let store = populated();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.json");
        let n = export_json_file_blocking(&store, &path, &ExportOptions::default()).unwrap();
        assert_eq!(n as u64, std::fs::metadata(&path).unwrap().len());
    }
}
