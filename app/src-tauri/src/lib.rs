pub mod commands;

use std::sync::Arc;

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
            commands::print_window,
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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Jyotish Desk");
}
