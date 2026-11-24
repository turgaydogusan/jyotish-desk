//! Atlas SQLite veritabanı: üretim (build_atlas) ve sorgu (Atlas, Task 4).

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use rusqlite::{params, Connection};

use crate::geonames::{read_admin1, read_cities, read_countries, CityRecord, GeonamesError};

pub const SCHEMA_VERSION: &str = "1";

const SCHEMA_SQL: &str = "
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE places (
    geoname_id   INTEGER PRIMARY KEY,
    name         TEXT NOT NULL,
    ascii_name   TEXT NOT NULL,
    country_code TEXT NOT NULL,
    country      TEXT NOT NULL,
    admin1       TEXT NOT NULL,
    lat          REAL NOT NULL,
    lon          REAL NOT NULL,
    timezone     TEXT NOT NULL,
    population   INTEGER NOT NULL
);
CREATE VIRTUAL TABLE places_fts USING fts5(
    name, ascii_name,
    content='places', content_rowid='geoname_id',
    tokenize='unicode61 remove_diacritics 2'
);
";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BuildStats {
    pub inserted: usize,
    pub skipped_unknown_tz: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum AtlasError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Geonames(#[from] GeonamesError),
    #[error("atlas schema version {found:?}, expected {expected}")]
    SchemaMismatch {
        found: Option<String>,
        expected: &'static str,
    },
}

/// Atlası `out` yoluna atomik olarak yazar: önce `<out>.tmp`, sonra rename.
/// Saat dilimi gömülü tzdb'de olmayan şehirler atlanır ve sayılır.
pub fn build_atlas(
    cities: &[CityRecord],
    admin1: &HashMap<String, String>,
    countries: &HashMap<String, String>,
    source_date: &str,
    out: &Path,
) -> Result<BuildStats, AtlasError> {
    let tmp = out.with_extension("sqlite.tmp");
    if tmp.exists() {
        std::fs::remove_file(&tmp)?;
    }
    let mut stats = BuildStats::default();
    {
        let mut conn = Connection::open(&tmp)?;
        conn.execute_batch(SCHEMA_SQL)?;
        let tx = conn.transaction()?;
        {
            let mut ins = tx.prepare(
                "INSERT INTO places (geoname_id, name, ascii_name, country_code, country, admin1, lat, lon, timezone, population)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            )?;
            for c in cities {
                if !crate::moment::is_known_time_zone(&c.timezone) {
                    stats.skipped_unknown_tz += 1;
                    continue;
                }
                let admin1_name = admin1
                    .get(&format!("{}.{}", c.country_code, c.admin1_code))
                    .cloned()
                    .unwrap_or_default();
                let country = countries
                    .get(&c.country_code)
                    .cloned()
                    .unwrap_or_else(|| c.country_code.clone());
                ins.execute(params![
                    c.geoname_id,
                    c.name,
                    c.ascii_name,
                    c.country_code,
                    country,
                    admin1_name,
                    c.lat,
                    c.lon,
                    c.timezone,
                    c.population
                ])?;
                stats.inserted += 1;
            }
        }
        tx.execute("INSERT INTO places_fts(places_fts) VALUES('rebuild')", [])?;
        tx.execute(
            "INSERT INTO meta (key, value) VALUES ('schema_version', ?1), ('source_date', ?2), ('row_count', ?3)",
            params![SCHEMA_VERSION, source_date, stats.inserted.to_string()],
        )?;
        tx.commit()?;
        conn.execute_batch("VACUUM;")?;
    }
    std::fs::rename(&tmp, out)?;
    Ok(stats)
}

pub fn build_atlas_from_files(
    cities: &Path,
    admin1: &Path,
    countries: &Path,
    source_date: &str,
    out: &Path,
) -> Result<BuildStats, AtlasError> {
    let cities = read_cities(BufReader::new(File::open(cities)?))?;
    let admin1 = read_admin1(BufReader::new(File::open(admin1)?))?;
    let countries = read_countries(BufReader::new(File::open(countries)?))?;
    build_atlas(&cities, &admin1, &countries, source_date, out)
}
