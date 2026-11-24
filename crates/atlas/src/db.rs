//! Atlas SQLite veritabanı: üretim (build_atlas) ve sorgu (Atlas, Task 4).

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use rusqlite::{params, Connection, OpenFlags, OptionalExtension, Row};

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

/// Atlas sorgu sonucu: tek bir yerleşim yeri.
#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    pub geoname_id: i64,
    pub name: String,
    pub ascii_name: String,
    pub country_code: String,
    pub country: String,
    pub admin1: String,
    pub lat: f64,
    pub lon: f64,
    pub timezone: String,
    pub population: i64,
}

const MAX_LIMIT: usize = 100;

const PLACE_COLUMNS: &str =
    "p.geoname_id, p.name, p.ascii_name, p.country_code, p.country, p.admin1, p.lat, p.lon, p.timezone, p.population";

/// Salt okunur atlas. `Connection` `Sync` değildir; paylaşmak için `Mutex` arkasına koyun.
pub struct Atlas {
    conn: Connection,
}

impl Atlas {
    /// Salt okunur açar ve şema sürümünü doğrular.
    pub fn open(path: &Path) -> Result<Atlas, AtlasError> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        let found: Option<String> = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'schema_version'",
                [],
                |r| r.get(0),
            )
            .ok();
        if found.as_deref() != Some(SCHEMA_VERSION) {
            return Err(AtlasError::SchemaMismatch {
                found,
                expected: SCHEMA_VERSION,
            });
        }
        Ok(Atlas { conn })
    }

    /// Önek araması: kelimeler AND; tam ad eşleşmesi önce, sonra nüfus, sonra id.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Place>, AtlasError> {
        let Some(fts) = fts_query(query) else {
            return Ok(Vec::new());
        };
        let limit = i64::try_from(limit.min(MAX_LIMIT)).unwrap_or(0);
        let sql = format!(
            "SELECT {PLACE_COLUMNS}
             FROM places_fts JOIN places p ON p.geoname_id = places_fts.rowid
             WHERE places_fts MATCH ?1
             ORDER BY (lower(p.name) = lower(?2) OR lower(p.ascii_name) = lower(?2)) DESC,
                      p.population DESC, p.geoname_id
             LIMIT ?3"
        );
        let mut stmt = self.conn.prepare_cached(&sql)?;
        let rows = stmt.query_map(params![fts, query.trim(), limit], row_to_place)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get(&self, geoname_id: i64) -> Result<Option<Place>, AtlasError> {
        let sql = format!("SELECT {PLACE_COLUMNS} FROM places p WHERE p.geoname_id = ?1");
        Ok(self
            .conn
            .query_row(&sql, params![geoname_id], row_to_place)
            .optional()?)
    }

    pub fn source_date(&self) -> Result<String, AtlasError> {
        Ok(self.conn.query_row(
            "SELECT value FROM meta WHERE key = 'source_date'",
            [],
            |r| r.get(0),
        )?)
    }

    pub fn row_count(&self) -> Result<i64, AtlasError> {
        let s: String =
            self.conn
                .query_row("SELECT value FROM meta WHERE key = 'row_count'", [], |r| {
                    r.get(0)
                })?;
        Ok(s.parse().unwrap_or(0))
    }
}

fn row_to_place(r: &Row<'_>) -> rusqlite::Result<Place> {
    Ok(Place {
        geoname_id: r.get(0)?,
        name: r.get(1)?,
        ascii_name: r.get(2)?,
        country_code: r.get(3)?,
        country: r.get(4)?,
        admin1: r.get(5)?,
        lat: r.get(6)?,
        lon: r.get(7)?,
        timezone: r.get(8)?,
        population: r.get(9)?,
    })
}

/// Kullanıcı girdisinden güvenli bir FTS5 sorgusu: harf/rakam dışındaki her karakter
/// ayırıcıdır; her kelime `"kelime"*` (önek), kelimeler AND ile birleşir.
pub fn fts_query(input: &str) -> Option<String> {
    let terms: Vec<String> = input
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{t}\"*"))
        .collect();
    if terms.is_empty() {
        None
    } else {
        Some(terms.join(" "))
    }
}
