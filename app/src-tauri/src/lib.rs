pub mod atlas_commands;
pub mod avastha_commands;
pub mod bhava_commands;
pub mod calendar_commands;
pub mod client_commands;
pub mod commands;
pub mod data_dir;
pub mod module_commands;
pub mod muhurta_commands;
pub mod natal_extras_commands;
pub mod prashna_commands;
pub mod search_commands;
pub mod store_commands;
pub mod transit_commands;
pub mod transit_tools_commands;
pub mod update_commands;
pub mod varsha_commands;

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
        // Nakshatra lordları tablosu (Plan 26): arayüz lordu bu sabitten okur, türetmez (C4).
        .constant("NAKSHATRA_LORDS", jyotish_core::NAKSHATRA_LORDS)
        .commands(tauri_specta::collect_commands![
            commands::engine_status,
            commands::compute_chart,
            commands::compute_transits,
            transit_commands::transits_at,
            transit_tools_commands::sankranti::sankranti_list,
            transit_tools_commands::ephemeris::ephemeris_month,
            transit_tools_commands::step::transit_step,
            bhava_commands::compute_bhava_table,
            bhava_commands::compute_sudarshana,
            commands::dasha_children,
            varsha_commands::compute_varsha,
            varsha_commands::save_varsha_chart,
            commands::print_window,
            avastha_commands::compute_avasthas,
            module_commands::compute_drishti,
            module_commands::compute_jaimini,
            module_commands::compute_chara_dasha,
            module_commands::chara_dasha_children,
            module_commands::chara_dasha_at,
            module_commands::compute_shadbala,
            module_commands::compute_yogas,
            module_commands::compute_compat,
            natal_extras_commands::compute_panchang_extras,
            natal_extras_commands::compute_navatara,
            natal_extras_commands::compute_vimshopaka,
            muhurta_commands::day_panchang,
            muhurta_commands::muhurta_search,
            calendar_commands::month_calendar,
            prashna_commands::create_prashna,
            prashna_commands::update_prashna,
            prashna_commands::compute_prashna_facts,
            search_commands::chart_search,
            search_commands::chart_search_cancel,
            search_commands::tag_charts,
            search_commands::list_saved_searches,
            search_commands::save_search,
            search_commands::delete_saved_search,
            search_commands::event_search,
            search_commands::event_search_cancel,
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
            store_commands::chart_input,
            store_commands::chart_brief,
            store_commands::search_clients,
            store_commands::add_note,
            store_commands::update_note,
            store_commands::delete_note,
            store_commands::list_notes,
            store_commands::set_chart_tags,
            store_commands::list_tags,
            store_commands::get_app_prefs,
            store_commands::set_app_prefs,
            store_commands::load_muhurta_filters,
            store_commands::save_muhurta_filters,
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
        .manage(Arc::new(search_commands::SearchRegistry::default()))
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
