mod common;

use atlas::db::{build_atlas_from_files, BuildStats};
use atlas::geonames::{parse_city_line, read_admin1, read_countries};

#[test]
fn parses_a_city_line() {
    let line = common::city_line(
        745044,
        "İstanbul",
        "Istanbul",
        41.01384,
        28.94966,
        "TR",
        "34",
        14_804_116,
        "Europe/Istanbul",
    );
    let c = parse_city_line(&line).unwrap();
    assert_eq!(c.geoname_id, 745044);
    assert_eq!(c.name, "İstanbul");
    assert_eq!(c.ascii_name, "Istanbul");
    assert_eq!((c.lat, c.lon), (41.01384, 28.94966));
    assert_eq!(
        (c.country_code.as_str(), c.admin1_code.as_str()),
        ("TR", "34")
    );
    assert_eq!(c.population, 14_804_116);
    assert_eq!(c.timezone, "Europe/Istanbul");
}

#[test]
fn rejects_malformed_lines() {
    assert!(parse_city_line("1\tonly\tthree").is_err());
    let bad_lat = common::city_line(1, "X", "X", 91.0, 0.0, "TR", "34", 1, "Europe/Istanbul");
    assert!(parse_city_line(&bad_lat).is_err());
    let bad_id = common::city_line(1, "X", "X", 0.0, 0.0, "TR", "34", 1, "Europe/Istanbul")
        .replacen('1', "x", 1);
    assert!(parse_city_line(&bad_id).is_err());
}

#[test]
fn empty_population_is_zero() {
    let line = common::city_line(1, "X", "X", 0.0, 0.0, "TR", "34", 0, "Europe/Istanbul")
        .replace("\t0\t\t10\t", "\t\t\t10\t");
    assert_eq!(parse_city_line(&line).unwrap().population, 0);
}

#[test]
fn reads_admin1_and_countries() {
    let a = read_admin1("TR.34\tIstanbul\tIstanbul\t745042\n".as_bytes()).unwrap();
    assert_eq!(a["TR.34"], "Istanbul");
    let k = read_countries("# c\n#ISO\tx\nTR\tTUR\t792\tTU\tTurkey\tAnkara\n".as_bytes()).unwrap();
    assert_eq!(k["TR"], "Turkey");
    assert_eq!(k.len(), 1);
}

#[test]
fn builds_db_and_skips_unknown_time_zones() {
    let dir = tempfile::tempdir().unwrap();
    let (c, a, k) = common::write_fixture_files(dir.path());
    let out = dir.path().join("atlas.sqlite");
    let stats = build_atlas_from_files(&c, &a, &k, "2026-11-24", &out).unwrap();
    assert_eq!(
        stats,
        BuildStats {
            inserted: 6,
            skipped_unknown_tz: 1
        }
    );
    assert!(out.exists());
    assert!(!dir.path().join("atlas.sqlite.tmp").exists());

    let conn = rusqlite::Connection::open(&out).unwrap();
    let n: i64 = conn
        .query_row("SELECT count(*) FROM places", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 6);
    let (admin1, country): (String, String) = conn
        .query_row(
            "SELECT admin1, country FROM places WHERE geoname_id = 745044",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((admin1.as_str(), country.as_str()), ("Istanbul", "Turkey"));
    let fts: i64 = conn
        .query_row(
            "SELECT count(*) FROM places_fts WHERE places_fts MATCH '\"istanbul\"*'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(fts, 1);
    let v: String = conn
        .query_row(
            "SELECT value FROM meta WHERE key='schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(v, "1");
}

#[test]
fn rebuild_overwrites_existing_output() {
    let dir = tempfile::tempdir().unwrap();
    let (c, a, k) = common::write_fixture_files(dir.path());
    let out = dir.path().join("atlas.sqlite");
    std::fs::write(&out, b"stale").unwrap();
    build_atlas_from_files(&c, &a, &k, "2026-11-24", &out).unwrap();
    assert!(rusqlite::Connection::open(&out)
        .unwrap()
        .query_row("SELECT count(*) FROM places", [], |r| r.get::<_, i64>(0))
        .is_ok());
}
