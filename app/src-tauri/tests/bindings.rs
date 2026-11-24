//! `app/ui/bindings.ts` dosyasının komut tanımlarıyla aynı olduğunu doğrular.
//! Güncellemek için: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings`

use std::path::PathBuf;

#[test]
fn bindings_ts_is_up_to_date() {
    let committed = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ui/bindings.ts");
    let fresh = std::env::temp_dir().join(format!("jd-bindings-{}.ts", std::process::id()));
    jyotish_desk_lib::specta_builder()
        .export(specta_typescript::Typescript::default(), &fresh)
        .expect("export bindings");
    let fresh_text = std::fs::read_to_string(&fresh).expect("read fresh");
    let _ = std::fs::remove_file(&fresh);
    // Her IPC tipi tek bir TS tipidir: `X_Serialize | X_Deserialize` ayrımı yok.
    assert!(
        !fresh_text.contains("_Serialize") && !fresh_text.contains("_Deserialize"),
        "bindings split a type into serde phases; keep disable_serde_phases() in specta_builder"
    );
    if std::env::var_os("UPDATE_BINDINGS").is_some() {
        std::fs::write(&committed, &fresh_text).expect("write bindings.ts");
        return;
    }
    let committed_text = std::fs::read_to_string(&committed).unwrap_or_default();
    assert!(
        committed_text == fresh_text,
        "app/ui/bindings.ts is stale; run: UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings"
    );
}
