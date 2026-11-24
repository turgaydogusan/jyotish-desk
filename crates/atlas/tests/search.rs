mod common;

use atlas::db::fts_query;
use atlas::{Atlas, AtlasError};

fn ids(atlas: &Atlas, q: &str) -> Vec<i64> {
    atlas
        .search(q, 10)
        .unwrap()
        .iter()
        .map(|p| p.geoname_id)
        .collect()
}

#[test]
fn finds_istanbul_in_all_spellings() {
    let (_dir, path) = common::fixture_db();
    let atlas = Atlas::open(&path).unwrap();
    for q in ["istanbul", "İstanbul", "ISTANBUL", "ista", "  istanbul  "] {
        assert_eq!(ids(&atlas, q).first(), Some(&745044), "query {q:?}");
    }
    let p = &atlas.search("istanbul", 1).unwrap()[0];
    assert_eq!(p.name, "İstanbul");
    assert_eq!(p.admin1, "Istanbul");
    assert_eq!(p.country, "Turkey");
    assert_eq!(p.country_code, "TR");
    assert_eq!(p.timezone, "Europe/Istanbul");
    assert_eq!((p.lat, p.lon), (41.01384, 28.94966));
}

#[test]
fn turkish_letters_match_ascii_and_native() {
    let (_dir, path) = common::fixture_db();
    let atlas = Atlas::open(&path).unwrap();
    assert_eq!(ids(&atlas, "izmir"), vec![311046]);
    assert_eq!(ids(&atlas, "İzmir"), vec![311046]);
    assert_eq!(ids(&atlas, "sanliurfa"), vec![298333]);
    assert_eq!(ids(&atlas, "Şanlıurfa"), vec![298333]);
}

#[test]
fn multi_word_queries_are_and_prefix() {
    let (_dir, path) = common::fixture_db();
    let atlas = Atlas::open(&path).unwrap();
    assert_eq!(ids(&atlas, "new york"), vec![5128581]);
    assert_eq!(ids(&atlas, "york new"), vec![5128581]);
    assert_eq!(ids(&atlas, "new par"), Vec::<i64>::new());
}

#[test]
fn ranks_by_population_and_respects_limit() {
    let (_dir, path) = common::fixture_db();
    let atlas = Atlas::open(&path).unwrap();
    assert_eq!(ids(&atlas, "i"), vec![745044, 311046]);
    assert_eq!(atlas.search("i", 1).unwrap().len(), 1);
}

#[test]
fn hostile_or_empty_input_is_safe() {
    let (_dir, path) = common::fixture_db();
    let atlas = Atlas::open(&path).unwrap();
    for q in ["", "   ", "*", "\"", "\"foo\" OR", "NEAR(", "a:b", "-- ;"] {
        assert!(atlas.search(q, 10).is_ok(), "query {q:?}");
    }
    assert!(atlas.search("", 10).unwrap().is_empty());
}

#[test]
fn fts_query_builds_quoted_prefix_terms() {
    assert_eq!(fts_query("new york").as_deref(), Some("\"new\"* \"york\"*"));
    assert_eq!(
        fts_query("Saint-Denis").as_deref(),
        Some("\"Saint\"* \"Denis\"*")
    );
    assert_eq!(fts_query("\"\"*"), None);
    assert_eq!(fts_query("Şanlı"), Some("\"Şanlı\"*".to_owned()));
}

#[test]
fn get_by_id_and_meta() {
    let (_dir, path) = common::fixture_db();
    let atlas = Atlas::open(&path).unwrap();
    assert_eq!(atlas.get(323786).unwrap().unwrap().name, "Ankara");
    assert!(atlas.get(1).unwrap().is_none());
    assert_eq!(atlas.source_date().unwrap(), "2026-11-24");
    assert_eq!(atlas.row_count().unwrap(), 6);
}

#[test]
fn open_rejects_missing_and_foreign_files() {
    let dir = tempfile::tempdir().unwrap();
    assert!(Atlas::open(&dir.path().join("nope.sqlite")).is_err());
    let foreign = dir.path().join("foreign.sqlite");
    rusqlite::Connection::open(&foreign)
        .unwrap()
        .execute_batch("CREATE TABLE t(x);")
        .unwrap();
    assert!(matches!(
        Atlas::open(&foreign),
        Err(AtlasError::SchemaMismatch { found: None, .. })
    ));
}
