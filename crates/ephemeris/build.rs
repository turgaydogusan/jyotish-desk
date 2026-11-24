fn main() {
    let dir = "vendor/swisseph";
    let files = [
        "swedate.c",
        "swehouse.c",
        "swejpl.c",
        "swemmoon.c",
        "swemplan.c",
        "sweph.c",
        "swephlib.c",
        "swecl.c",
        "swehel.c",
    ];

    println!("cargo:rerun-if-changed={dir}");
    let mut build = cc::Build::new();
    build.include(dir).warnings(false).opt_level(2);
    for f in files {
        let path = format!("{dir}/{f}");
        println!("cargo:rerun-if-changed={path}");
        build.file(path);
    }
    build.compile("swe");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "linux" || target_os == "android" {
        println!("cargo:rustc-link-lib=m");
    }
}
