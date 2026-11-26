pub mod atlas_commands;
pub mod client_commands;
pub mod commands;
pub mod data_dir;
pub mod module_commands;
pub mod store_commands;
pub mod transit_commands;
pub mod update_commands;

use std::sync::{Arc, Mutex};

use tauri::path::BaseDirectory;
use tauri::Manager;

/// Komut listesi tek yerde: hem uygulama hem bağlama testi bunu kullanır.
pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
    // rc.29 varsayılanı `Throw`; arayüz `{ status, data | error }` sonucunu bekler.
    // Serde fazları kapalı: specta-serde her `Option<T>` alanında tipi
    // `X_Serialize | X_Deserialize` olarak ikiye böler. IPC tiplerimizde faz farkı yaratan
    // serde özniteliği (default, skip_serializing_if, asimetrik rename) yok; tek tip iki
    // yön için de doğrudur.
    tauri_specta::Builder::<tauri::Wry>::new()
        .error_handling(tauri_specta::ErrorHandlingMode::Result)
        .disable_serde_phases()
        .commands(tauri_specta::collect_commands![
            commands::engine_status,
            commands::compute_chart,
            commands::compute_transits,
            transit_commands::transits_at,
            commands::dasha_children,
            commands::print_window,
            module_commands::compute_drishti,
            module_commands::compute_jaimini,
            module_commands::compute_chara_dasha,
            module_commands::chara_dasha_children,
            module_commands::chara_dasha_at,
            module_commands::compute_shadbala,
            client_commands::open_client,
            client_commands::restore_session,
            client_commands::create_client,
            client_commands::add_chart,
            client_commands::update_chart,
            client_commands::keep_stored_moment,
            client_commands::save_calc_settings,
            client_commands::save_view_state,
            store_commands::update_client,
            store_commands::delete_client,
            store_commands::delete_chart,
            store_commands::list_clients,
            store_commands::list_clients_by_tag,
            store_commands::list_charts,
            store_commands::search_clients,
            store_commands::add_note,
            store_commands::update_note,
            store_commands::delete_note,
            store_commands::list_notes,
            store_commands::set_chart_tags,
            store_commands::list_tags,
            store_commands::get_app_prefs,
            store_commands::set_app_prefs,
            store_commands::load_panel_layout,
            store_commands::save_panel_layout,
            store_commands::export_clients_json,
            store_commands::export_clients_csv,
            store_commands::import_clients_json,
            atlas_commands::search_places,
            atlas_commands::preview_moment,
            update_commands::app_info,
            update_commands::check_for_update,
            update_commands::install_update,
        ])
}

pub fn run() {
    let builder = specta_builder();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // Güncelleyici yalnızca `update_commands` üzerinden çalışır; açılışta denetim yapmaz.
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(update_commands::InstallGuard::default())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            let ephe = app.path().resolve("ephe", BaseDirectory::Resource)?;
            // Eksik veri ölümcül: setup hatası Tauri'de "Failed to setup app" paniğine yol
            // açar (paketlenmiş uygulama sessizce kapanır). Yerel hata penceresi sonra eklenecek.
            let engine = ephemeris::Engine::spawn(ephe)?;
            app.manage(Arc::new(engine));
            // Danışan veritabanı: ~/Library/Application Support/<identifier>/. Açılamazsa
            // (ör. daha yeni bir sürümün veritabanı) setup hatası, motorla aynı yol.
            let data_dir = app.path().app_data_dir()?;
            // Klasör 0700, veritabanı dosyaları 0600 (Plan 08 ertelenen madde 10).
            data_dir::prepare(&data_dir)?;
            let db_path = data_dir.join(client_commands::DB_FILE_NAME);
            let db = store::Store::open(&db_path)?;
            data_dir::restrict_db_files(&db_path)?;
            let state: client_commands::StoreState = Arc::new(Mutex::new(db));
            app.manage(state);
            // Atlas ölümcül değildir: açılamazsa yer araması `atlas_unavailable` döner, elle
            // koordinat girişi çalışır.
            let atlas_path = app
                .path()
                .resolve(atlas_commands::ATLAS_RESOURCE, BaseDirectory::Resource)?;
            app.manage(Arc::new(atlas_commands::AtlasState::open(&atlas_path)));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Jyotish Desk");
}
