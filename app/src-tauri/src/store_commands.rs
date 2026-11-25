//! Kayıt yönetimi komutları (danışan, harita, not, etiket, tercihler) ve dosya içe/dışa
//! aktarma. Harita hesabı gerektirmez; gövdeler `store`'dadır.

use std::path::Path;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use specta::Type;
use store::{
    AppPrefs, ChartRecord, Client, ExportOptions, Gender, ImportReport, Note, PanelLayout,
    SearchHit, Store, TagCount,
};
use tauri::State;

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

fn io_error(path: &Path, e: std::io::Error) -> CommandError {
    CommandError::new(
        "file_io",
        &[
            ("path", path.display().to_string()),
            ("detail", e.to_string()),
        ],
    )
}

pub fn import_jhd_files_blocking(
    store: &Mutex<Store>,
    paths: &[String],
) -> Result<JhdImportResult, CommandError> {
    let mut files = Vec::with_capacity(paths.len());
    let mut failed = Vec::new();
    for p in paths {
        let path = Path::new(p);
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| p.clone());
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
    path: &str,
    options: &ExportOptions,
) -> Result<(), CommandError> {
    let text = lock(store)?.export_json(options)?;
    std::fs::write(path, text).map_err(|e| io_error(Path::new(path), e))
}

pub fn export_csv_file_blocking(store: &Mutex<Store>, path: &str) -> Result<(), CommandError> {
    let text = lock(store)?.export_csv()?;
    std::fs::write(path, text).map_err(|e| io_error(Path::new(path), e))
}

pub fn import_json_file_blocking(
    store: &Mutex<Store>,
    path: &str,
) -> Result<ImportReport, CommandError> {
    let text = std::fs::read_to_string(path).map_err(|e| io_error(Path::new(path), e))?;
    Ok(lock(store)?.import_json(&text)?)
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

#[tauri::command]
#[specta::specta]
pub async fn list_charts(
    store: State<'_, StoreState>,
    client_id: u32,
) -> Result<Vec<ChartRecord>, CommandError> {
    with_store(&store, move |s| s.charts_of(client_id)).await
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
pub async fn get_app_prefs(store: State<'_, StoreState>) -> Result<AppPrefs, CommandError> {
    with_store(&store, |s| s.app_prefs()).await
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

#[tauri::command]
#[specta::specta]
pub async fn export_json(
    store: State<'_, StoreState>,
    path: String,
    selection: ExportOptions,
) -> Result<(), CommandError> {
    let store = Arc::clone(&store);
    blocking(move || export_json_file_blocking(&store, &path, &selection)).await
}

#[tauri::command]
#[specta::specta]
pub async fn export_csv(store: State<'_, StoreState>, path: String) -> Result<(), CommandError> {
    let store = Arc::clone(&store);
    blocking(move || export_csv_file_blocking(&store, &path)).await
}

#[tauri::command]
#[specta::specta]
pub async fn import_json(
    store: State<'_, StoreState>,
    path: String,
) -> Result<ImportReport, CommandError> {
    let store = Arc::clone(&store);
    blocking(move || import_json_file_blocking(&store, &path)).await
}

#[tauri::command]
#[specta::specta]
pub async fn import_jhd(
    store: State<'_, StoreState>,
    paths: Vec<String>,
) -> Result<JhdImportResult, CommandError> {
    let store = Arc::clone(&store);
    blocking(move || import_jhd_files_blocking(&store, &paths)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_commands_report_io_errors_per_file() {
        let store = Mutex::new(Store::open_in_memory().unwrap());
        let r = import_jhd_files_blocking(&store, &["/nonexistent/dir/a.jhd".into()]).unwrap();
        assert_eq!(r.failed.len(), 1);
        assert_eq!(r.failed[0].file, "a.jhd");
        assert_eq!(r.failed[0].error.code, "file_io");
        let e = import_json_file_blocking(&store, "/nonexistent/x.json").unwrap_err();
        assert_eq!(e.code, "file_io");
    }
}
