//! Güncelleme (spec §10): Tauri güncelleyicisi, imzalı, GitHub Releases üzerinden. Denetim yalnızca
//! kullanıcı isteyince yapılır: bu modülde kendiliğinden çalışan hiçbir şey yoktur; arayüz
//! `check_for_update`'i kullanıcının açık isteğiyle (düğme ya da açtığı tercih) çağırır.
//!
//! Ağa giden tek istek güncelleme adresine yapılan GET'tir (`UPDATE_ENDPOINT`) ve kurulumda
//! paketin indirilmesidir; istekte sürüm, kimlik ya da danışan verisi yoktur. İndirilen paket
//! gömülü açık anahtarla doğrulanmadan kurulmaz (eklenti bunu atlamaya izin vermez) ve imzanın
//! taşıdığı sürüm, bildirilen sürümle aynı olmalıdır (`requireSignedVersion`).
//!
//! Eklentinin kendi IPC komutları (`plugin:updater|…`) yetenek dosyasında izinli değildir;
//! arayüz yalnızca buradaki üç komutu çağırır ve hiçbiri adres ya da dosya yolu almaz (C2′).

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};
use tauri_plugin_updater::{Error as UpdaterError, UpdaterExt};

use crate::commands::CommandError;

/// Güncelleme bildiriminin adresi; `tauri.conf.json` ile aynıdır (test denetler).
pub const UPDATE_ENDPOINT: &str =
    "https://github.com/turgaydogusan/jyotish-desk/releases/latest/download/latest.json";
/// Bildirim isteğinin ve paket indirmenin zaman sınırları.
const CHECK_TIMEOUT: Duration = Duration::from_secs(30);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(600);
/// Arayüze taşınan yayın notunun en büyük uzunluğu (karakter).
pub const MAX_NOTES_CHARS: usize = 4000;

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    /// Açık anahtar gömülü mü; değilse (anahtar üretilmeden yapılan derleme) denetim kapalıdır.
    pub updater_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: Option<String>,
    /// Yayın günü, `YYYY-MM-DD`.
    pub published: Option<String>,
}

/// Aynı anda tek kurulum (managed state).
#[derive(Default)]
pub struct InstallGuard(AtomicBool);

/// `plugins.updater.pubkey` boş değilse güncelleme yapılandırılmıştır.
pub fn updater_configured(plugins: &serde_json::Value) -> bool {
    plugins
        .pointer("/updater/pubkey")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|k| !k.trim().is_empty())
}

fn configured(app: &AppHandle) -> bool {
    serde_json::to_value(&app.config().plugins).is_ok_and(|v| updater_configured(&v))
}

pub fn clip_notes(notes: &str) -> String {
    match notes.char_indices().nth(MAX_NOTES_CHARS) {
        Some((i, _)) => format!("{}…", &notes[..i]),
        None => notes.to_owned(),
    }
}

/// Hatanın kararlı nedeni; arayüz bunu kendi dilinde gösterir. Eklentinin iletisi taşınmaz
/// (adres ve dosya yolu içerebilir).
pub fn update_reason(e: &UpdaterError) -> &'static str {
    match e {
        UpdaterError::Reqwest(_) | UpdaterError::Network(_) | UpdaterError::Http(_) => "network",
        UpdaterError::Minisign(_)
        | UpdaterError::Base64(_)
        | UpdaterError::SignatureUtf8(_)
        | UpdaterError::MissingSignedVersion
        | UpdaterError::SignedVersionMismatch { .. } => "signature",
        UpdaterError::ReleaseNotFound
        | UpdaterError::Serialization(_)
        | UpdaterError::Semver(_)
        | UpdaterError::UrlParse(_)
        | UpdaterError::TargetNotFound(_)
        | UpdaterError::TargetsNotFound(_) => "manifest",
        UpdaterError::Io(_)
        | UpdaterError::PreviousAppNotRestored(_)
        | UpdaterError::TempDirNotOnSameMountPoint
        | UpdaterError::TempDirNotFound
        | UpdaterError::BinaryNotFoundInArchive
        | UpdaterError::FailedToDetermineExtractPath
        | UpdaterError::AuthenticationFailed => "install",
        _ => "other",
    }
}

fn update_error(code: &str, e: &UpdaterError) -> CommandError {
    CommandError::new(code, &[("reason", update_reason(e).to_owned())])
}

async fn find_update(
    app: &AppHandle,
    code: &str,
) -> Result<Option<tauri_plugin_updater::Update>, CommandError> {
    if !configured(app) {
        return Err(CommandError::new("update_unconfigured", &[]));
    }
    let updater = app
        .updater_builder()
        .timeout(CHECK_TIMEOUT)
        .build()
        .map_err(|e| update_error(code, &e))?;
    updater.check().await.map_err(|e| update_error(code, &e))
}

#[tauri::command]
#[specta::specta]
pub async fn app_info(app: AppHandle) -> Result<AppInfo, CommandError> {
    Ok(AppInfo {
        version: app.package_info().version.to_string(),
        updater_configured: configured(&app),
    })
}

/// Yeni sürüm var mı. `None`: kurulu sürüm günceldir.
#[tauri::command]
#[specta::specta]
pub async fn check_for_update(app: AppHandle) -> Result<Option<UpdateInfo>, CommandError> {
    let update = find_update(&app, "update_check_failed").await?;
    Ok(update.map(|u| UpdateInfo {
        version: u.version.clone(),
        notes: u.body.as_deref().map(clip_notes),
        published: u.date.map(|d| d.date().to_string()),
    }))
}

/// Kullanıcının onayladığı sürümü indirir, imzasını doğrular ve kurar. Yayın bu arada
/// değiştiyse (`version` artık son sürüm değilse) hiçbir şey kurulmaz: onay o sürüme verilmiştir.
/// Yeni sürüm uygulama kapatılıp açılınca çalışır; uygulama kendini yeniden başlatmaz.
#[tauri::command]
#[specta::specta]
pub async fn install_update(
    app: AppHandle,
    guard: State<'_, InstallGuard>,
    version: String,
) -> Result<(), CommandError> {
    if guard.0.swap(true, Ordering::SeqCst) {
        return Err(CommandError::new("update_busy", &[]));
    }
    let result = install(&app, &version).await;
    guard.0.store(false, Ordering::SeqCst);
    result
}

async fn install(app: &AppHandle, version: &str) -> Result<(), CommandError> {
    let code = "update_install_failed";
    let mut update = find_update(app, code)
        .await?
        .filter(|u| u.version == version)
        .ok_or_else(|| CommandError::new("update_changed", &[]))?;
    update.timeout = Some(DOWNLOAD_TIMEOUT);
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| update_error(code, &e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn conf() -> serde_json::Value {
        serde_json::from_str(include_str!("../tauri.conf.json")).unwrap()
    }

    #[test]
    fn the_updater_is_configured_only_with_a_public_key() {
        assert!(!updater_configured(&json!({})));
        assert!(!updater_configured(&json!({ "updater": { "pubkey": "" } })));
        assert!(!updater_configured(
            &json!({ "updater": { "pubkey": "  " } })
        ));
        assert!(!updater_configured(&json!({ "updater": { "pubkey": 7 } })));
        assert!(updater_configured(
            &json!({ "updater": { "pubkey": "dW50cnVzdGVk" } })
        ));
    }

    #[test]
    fn the_shipped_configuration_checks_one_https_endpoint_and_requires_signed_versions() {
        let updater = &conf()["plugins"]["updater"];
        assert_eq!(updater["endpoints"], json!([UPDATE_ENDPOINT]));
        assert!(UPDATE_ENDPOINT.starts_with("https://github.com/"));
        // Adreste sürüm, hedef ya da başka bir değişken yoktur: istek kimseyi ayırt etmez.
        assert!(!UPDATE_ENDPOINT.contains("{{") && !UPDATE_ENDPOINT.contains('?'));
        assert_eq!(updater["requireSignedVersion"], json!(true));
        let keys: Vec<&str> = updater
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["endpoints", "pubkey", "requireSignedVersion"]);
        // Anahtar ya henüz yoktur (boş) ya da base64'tür (minisign açık anahtar dosyası).
        let key = updater["pubkey"].as_str().unwrap();
        assert!(
            key.is_empty()
                || (key.len() >= 100
                    && key
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"+/=".contains(&b))),
            "plugins.updater.pubkey is neither empty nor base64"
        );
    }

    #[test]
    fn the_webview_gets_no_updater_or_network_permission() {
        let cap: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();
        assert_eq!(cap["permissions"], json!(["core:event:default"]));
        let csp = conf()["app"]["security"]["csp"]
            .as_str()
            .unwrap()
            .to_owned();
        assert!(csp.contains("connect-src ipc: http://ipc.localhost;"));
        assert!(!csp.contains("https:"));
    }

    #[test]
    fn updater_artifacts_are_off_by_default_so_a_build_needs_no_signing_key() {
        assert!(conf()["bundle"].get("createUpdaterArtifacts").is_none());
    }

    #[test]
    fn long_release_notes_are_clipped_on_a_character_boundary() {
        assert_eq!(clip_notes("kısa"), "kısa");
        let long = "ş".repeat(MAX_NOTES_CHARS + 5);
        let clipped = clip_notes(&long);
        assert_eq!(clipped.chars().count(), MAX_NOTES_CHARS + 1);
        assert!(clipped.ends_with('…'));
    }

    #[test]
    fn updater_errors_map_to_stable_reasons_without_their_text() {
        assert_eq!(update_reason(&UpdaterError::ReleaseNotFound), "manifest");
        assert_eq!(
            update_reason(&UpdaterError::MissingSignedVersion),
            "signature"
        );
        assert_eq!(update_reason(&UpdaterError::Network("x".into())), "network");
        assert_eq!(
            update_reason(&UpdaterError::Io(std::io::Error::other("/Users/a/b"))),
            "install"
        );
        assert_eq!(update_reason(&UpdaterError::EmptyEndpoints), "other");
        let e = update_error(
            "update_check_failed",
            &UpdaterError::Io(std::io::Error::other("/Users/a/b")),
        );
        assert_eq!(e.code, "update_check_failed");
        assert_eq!(e.params.len(), 1);
        assert_eq!(e.params["reason"], "install");
    }
}
