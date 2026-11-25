pub mod client_commands;
pub mod commands;
pub mod store_commands;

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
            commands::planet_longitudes,
            commands::compute_chart,
            commands::compute_transits,
            commands::dasha_children,
            commands::print_window,
            client_commands::open_client,
            client_commands::restore_session,
            client_commands::create_client,
            client_commands::add_chart,
            client_commands::update_chart,
            client_commands::save_calc_settings,
            client_commands::save_view_state,
            store_commands::update_client,
            store_commands::delete_client,
            store_commands::delete_chart,
            store_commands::list_clients,
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
            store_commands::export_json,
            store_commands::export_csv,
            store_commands::import_json,
            store_commands::import_jhd,
        ])
}

pub fn run() {
    let builder = specta_builder();
    tauri::Builder::default()
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
            std::fs::create_dir_all(&data_dir)?;
            let db = store::Store::open(&data_dir.join(client_commands::DB_FILE_NAME))?;
            let state: client_commands::StoreState = Arc::new(Mutex::new(db));
            app.manage(state);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Jyotish Desk");
}
