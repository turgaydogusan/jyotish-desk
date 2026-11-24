use ephemeris::*;
use std::path::PathBuf;

#[test]
fn se_ephe_path_env_var_is_rejected() {
    let ephe_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/ephe");
    std::env::set_var("SE_EPHE_PATH", "/somewhere/else");
    assert!(matches!(
        SwissEph::open(&ephe_dir),
        Err(EphError::EnvOverride)
    ));
}
