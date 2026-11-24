#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// GeoNames cities1000 satırı (19 sekmeyle ayrılmış alan).
// Test yardımcısı: GeoNames alanlarını birebir yansıtır, bu yüzden parametre çok.
#[allow(clippy::too_many_arguments)]
pub fn city_line(
    id: i64,
    name: &str,
    ascii: &str,
    lat: f64,
    lon: f64,
    cc: &str,
    admin1: &str,
    pop: i64,
    tz: &str,
) -> String {
    let id = id.to_string();
    let (lat, lon, pop) = (lat.to_string(), lon.to_string(), pop.to_string());
    [
        id.as_str(),
        name,
        ascii,
        "",
        lat.as_str(),
        lon.as_str(),
        "P",
        "PPLA",
        cc,
        "",
        admin1,
        "",
        "",
        "",
        pop.as_str(),
        "",
        "10",
        tz,
        "2024-01-01",
    ]
    .join("\t")
}

pub fn write_fixture_files(dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let cities = [
        city_line(
            745044,
            "İstanbul",
            "Istanbul",
            41.01384,
            28.94966,
            "TR",
            "34",
            14_804_116,
            "Europe/Istanbul",
        ),
        city_line(
            323786,
            "Ankara",
            "Ankara",
            39.91987,
            32.85427,
            "TR",
            "68",
            3_517_182,
            "Europe/Istanbul",
        ),
        city_line(
            311046,
            "İzmir",
            "Izmir",
            38.41273,
            27.13838,
            "TR",
            "35",
            2_500_603,
            "Europe/Istanbul",
        ),
        city_line(
            298333,
            "Şanlıurfa",
            "Sanliurfa",
            37.16708,
            38.79392,
            "TR",
            "63",
            2_213_964,
            "Europe/Istanbul",
        ),
        city_line(
            2988507,
            "Paris",
            "Paris",
            48.85341,
            2.3488,
            "FR",
            "11",
            2_138_551,
            "Europe/Paris",
        ),
        city_line(
            5128581,
            "New York City",
            "New York City",
            40.71427,
            -74.00597,
            "US",
            "NY",
            8_804_190,
            "America/New_York",
        ),
        city_line(
            9_999_999,
            "Nowhere",
            "Nowhere",
            0.0,
            0.0,
            "XX",
            "00",
            10,
            "Mars/Olympus",
        ),
    ]
    .join("\n");
    let admin1 = [
        "TR.34\tIstanbul\tIstanbul\t745042",
        "TR.68\tAnkara\tAnkara\t323784",
        "TR.35\tİzmir\tIzmir\t311044",
        "TR.63\tŞanlıurfa\tSanliurfa\t298332",
        "FR.11\tÎle-de-France\tIle-de-France\t3012874",
        "US.NY\tNew York\tNew York\t5128638",
    ]
    .join("\n");
    let countries = [
        "# GeoNames countryInfo",
        "#ISO\tISO3\tISO-Numeric\tfips\tCountry\tCapital",
        "TR\tTUR\t792\tTU\tTurkey\tAnkara",
        "FR\tFRA\t250\tFR\tFrance\tParis",
        "US\tUSA\t840\tUS\tUnited States\tWashington",
    ]
    .join("\n");
    let (c, a, k) = (
        dir.join("cities1000.txt"),
        dir.join("admin1CodesASCII.txt"),
        dir.join("countryInfo.txt"),
    );
    std::fs::write(&c, cities + "\n").unwrap();
    std::fs::write(&a, admin1 + "\n").unwrap();
    std::fs::write(&k, countries + "\n").unwrap();
    (c, a, k)
}

pub fn fixture_db() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let (c, a, k) = write_fixture_files(dir.path());
    let out = dir.path().join("atlas.sqlite");
    atlas::db::build_atlas_from_files(&c, &a, &k, "2026-11-24", &out).unwrap();
    (dir, out)
}
