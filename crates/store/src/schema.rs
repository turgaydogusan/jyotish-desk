//! SQLite şeması ve göçler (spec §5). Sürüm `PRAGMA user_version`'da tutulur
//! (`rusqlite_migration`). Göçler yalnızca eklenir; yayımlanmış bir göç değiştirilmez.
//! Sürüm 2 (Plan 21) `prashna` yan tablosunu, sürüm 3 (Plan 22) `saved_searches` tablosunu ekler.

use rusqlite_migration::{Migrations, M};

/// Göç 1: v1 şeması. Spec §5'teki tablolara ek sütunlar karar tablosundadır
/// (`admin1`, `geoname_id`, `tz_kind`, `birth_notes`, `resolved_jd_ut`, `resolved_abbrev`,
/// `resolved_warnings_json`). `tags.name_key` etiketin kimliğidir (`notes::tag_key`): SQLite
/// `NOCASE` yalnızca ASCII katladığından teklik Rust'ta üretilen anahtardadır; `name` ilk
/// yazımı saklar.
const V1: &str = "
CREATE TABLE clients (
    id             INTEGER PRIMARY KEY,
    uuid           TEXT NOT NULL UNIQUE,
    name           TEXT NOT NULL,
    gender         TEXT CHECK (gender IN ('Male', 'Female', 'Other')),
    last_chart_id  INTEGER,
    last_opened_at TEXT,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
);
CREATE TABLE charts (
    id                     INTEGER PRIMARY KEY,
    uuid                   TEXT NOT NULL UNIQUE,
    client_id              INTEGER NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    label                  TEXT NOT NULL,
    kind                   TEXT NOT NULL CHECK (kind IN ('Natal', 'Event')),
    date                   TEXT NOT NULL,
    time                   TEXT,
    time_certainty         TEXT NOT NULL CHECK (time_certainty IN ('Exact', 'Approx', 'Unknown')),
    place_name             TEXT NOT NULL,
    country                TEXT NOT NULL,
    admin1                 TEXT NOT NULL,
    lat                    REAL NOT NULL,
    lon                    REAL NOT NULL,
    geoname_id             INTEGER,
    tz_kind                TEXT NOT NULL CHECK (tz_kind IN ('Iana', 'Fixed', 'Lmt')),
    tz_id                  TEXT,
    offset_override        INTEGER,
    fold                   TEXT CHECK (fold IN ('Earlier', 'Later')),
    birth_notes            TEXT NOT NULL,
    resolved_utc           TEXT NOT NULL,
    resolved_jd_ut         REAL NOT NULL,
    resolved_offset_secs   INTEGER NOT NULL,
    resolved_abbrev        TEXT NOT NULL,
    resolved_warnings_json TEXT NOT NULL,
    tzdb_version           TEXT NOT NULL,
    calc_settings_json     TEXT NOT NULL,
    view_state_json        TEXT NOT NULL,
    last_opened_at         TEXT,
    created_at             TEXT NOT NULL,
    updated_at             TEXT NOT NULL
);
CREATE INDEX charts_client ON charts(client_id, updated_at);
CREATE TABLE client_notes (
    id         INTEGER PRIMARY KEY,
    uuid       TEXT NOT NULL UNIQUE,
    client_id  INTEGER NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    chart_id   INTEGER REFERENCES charts(id) ON DELETE SET NULL,
    body       TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX client_notes_client ON client_notes(client_id, created_at);
CREATE TABLE tags (
    id       INTEGER PRIMARY KEY,
    name     TEXT NOT NULL,
    name_key TEXT NOT NULL UNIQUE
);
CREATE TABLE chart_tags (
    chart_id INTEGER NOT NULL REFERENCES charts(id) ON DELETE CASCADE,
    tag_id   INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (chart_id, tag_id)
) WITHOUT ROWID;
CREATE TABLE app_state (
    key        TEXT PRIMARY KEY,
    value_json TEXT NOT NULL
);
CREATE VIRTUAL TABLE clients_fts USING fts5(
    name, content='clients', content_rowid='id',
    tokenize='unicode61 remove_diacritics 2'
);
CREATE VIRTUAL TABLE notes_fts USING fts5(
    body, content='client_notes', content_rowid='id',
    tokenize='unicode61 remove_diacritics 2'
);
CREATE TRIGGER clients_ai AFTER INSERT ON clients BEGIN
    INSERT INTO clients_fts(rowid, name) VALUES (new.id, new.name);
END;
CREATE TRIGGER clients_ad AFTER DELETE ON clients BEGIN
    INSERT INTO clients_fts(clients_fts, rowid, name) VALUES ('delete', old.id, old.name);
END;
CREATE TRIGGER clients_au AFTER UPDATE OF name ON clients BEGIN
    INSERT INTO clients_fts(clients_fts, rowid, name) VALUES ('delete', old.id, old.name);
    INSERT INTO clients_fts(rowid, name) VALUES (new.id, new.name);
END;
CREATE TRIGGER notes_ai AFTER INSERT ON client_notes BEGIN
    INSERT INTO notes_fts(rowid, body) VALUES (new.id, new.body);
END;
CREATE TRIGGER notes_ad AFTER DELETE ON client_notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, body) VALUES ('delete', old.id, old.body);
END;
CREATE TRIGGER notes_au AFTER UPDATE OF body ON client_notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, body) VALUES ('delete', old.id, old.body);
    INSERT INTO notes_fts(rowid, body) VALUES (new.id, new.body);
END;
";

/// Göç 2: prashna yan tablosu (ileri modüller spec'i §11). Bir harita, bu tabloda satırı varsa
/// prashnadır; `charts.kind` ve CHECK kısıtı değişmez (kısıtı değiştirmek tabloyu yeniden kurmayı
/// gerektirirdi). Soru metni haritadan ayrı durur ve arama dizinine girmez. Harita silinince
/// satır da silinir.
const V2: &str = "
CREATE TABLE prashna (
    chart_id    INTEGER PRIMARY KEY REFERENCES charts(id) ON DELETE CASCADE,
    question    TEXT NOT NULL DEFAULT '',
    number      INTEGER CHECK (number IS NULL OR number BETWEEN 1 AND 108),
    focus_house INTEGER CHECK (focus_house IS NULL OR focus_house BETWEEN 1 AND 12),
    created_at  TEXT NOT NULL
);
";

/// Göç 3: kayıtlı aramalar (ileri modüller spec'i §12.1). Bir kayıt yalnızca bir ad ve sorgunun
/// JSON'udur; sonuç listesi ve sayısı saklanmaz. Adın tekliği (büyük/küçük harf ayırmadan)
/// Rust'ta sağlanır (SQLite `NOCASE` yalnızca ASCII katlar).
const V3: &str = "
CREATE TABLE saved_searches (
    id         INTEGER PRIMARY KEY,
    uuid       TEXT NOT NULL UNIQUE,
    name       TEXT NOT NULL,
    query_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
";

const MIGRATION_LIST: &[M<'static>] = &[M::up(V1), M::up(V2), M::up(V3)];

/// Bu sürümün şema sürümü (`MIGRATION_LIST.len()`).
pub const SCHEMA_VERSION: usize = 3;

pub(crate) const MIGRATIONS: Migrations<'static> = Migrations::from_slice(MIGRATION_LIST);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_valid_and_counted() {
        assert_eq!(MIGRATIONS.validate(), Ok(()));
        assert_eq!(MIGRATION_LIST.len(), SCHEMA_VERSION);
    }

    /// Sürüm 1 veritabanı (yalnızca ilk göç) açılışta sürüm 2'ye çıkar: var olan haritalar
    /// durur, hiçbiri prashna değildir ve yan tablo kısıtlarıyla gelir.
    #[test]
    fn a_version_one_database_gains_the_prashna_table() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", true).unwrap();
        Migrations::from_slice(&MIGRATION_LIST[..1])
            .to_latest(&mut conn)
            .unwrap();
        let version = |c: &rusqlite::Connection| -> i64 {
            c.query_row("PRAGMA user_version", [], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(version(&conn), 1);
        conn.execute_batch(
            "INSERT INTO clients (id, uuid, name, created_at, updated_at)
             VALUES (1, 'c', 'Deneme', 't', 't');
             INSERT INTO charts (id, uuid, client_id, label, kind, date, time_certainty,
                place_name, country, admin1, lat, lon, tz_kind, birth_notes, resolved_utc,
                resolved_jd_ut, resolved_offset_secs, resolved_abbrev, resolved_warnings_json,
                tzdb_version, calc_settings_json, view_state_json, created_at, updated_at)
             VALUES (7, 'h', 1, '', 'Event', '2000-01-01', 'Exact', '', '', '', 0, 0, 'Lmt', '',
                'u', 2451545.0, 0, '', '[]', 'v', '{}', '{}', 't', 't');",
        )
        .unwrap();
        Migrations::from_slice(&MIGRATION_LIST[..2])
            .to_latest(&mut conn)
            .unwrap();
        assert_eq!(version(&conn), 2);
        let count = |c: &rusqlite::Connection, sql: &str| -> i64 {
            c.query_row(sql, [], |r| r.get(0)).unwrap()
        };
        assert_eq!(count(&conn, "SELECT count(*) FROM charts"), 1);
        assert_eq!(count(&conn, "SELECT count(*) FROM prashna"), 0);
        // Kısıtlar: sayı 1..=108, odak evi 1..=12, harita var olmalı.
        let insert = |c: &rusqlite::Connection, values: &str| {
            c.execute(
                &format!(
                    "INSERT INTO prashna (chart_id, number, focus_house, created_at) VALUES {values}"
                ),
                [],
            )
        };
        assert!(insert(&conn, "(7, 109, NULL, 't')").is_err());
        assert!(insert(&conn, "(7, 0, NULL, 't')").is_err());
        assert!(insert(&conn, "(7, NULL, 13, 't')").is_err());
        assert!(insert(&conn, "(8, NULL, NULL, 't')").is_err());
        assert_eq!(insert(&conn, "(7, 108, 12, 't')"), Ok(1));
        assert_eq!(
            conn.query_row("SELECT question FROM prashna", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            ""
        );
        // Harita silinince satır da silinir.
        conn.execute("DELETE FROM charts WHERE id = 7", []).unwrap();
        assert_eq!(count(&conn, "SELECT count(*) FROM prashna"), 0);
    }

    /// Sürüm 2 veritabanı açılışta sürüm 3'e çıkar: haritalar ve prashna satırları durur,
    /// `saved_searches` boş gelir.
    #[test]
    fn a_version_two_database_gains_the_saved_searches_table() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", true).unwrap();
        Migrations::from_slice(&MIGRATION_LIST[..2])
            .to_latest(&mut conn)
            .unwrap();
        conn.execute_batch(
            "INSERT INTO clients (id, uuid, name, created_at, updated_at)
             VALUES (1, 'c', 'Deneme', 't', 't');
             INSERT INTO charts (id, uuid, client_id, label, kind, date, time_certainty,
                place_name, country, admin1, lat, lon, tz_kind, birth_notes, resolved_utc,
                resolved_jd_ut, resolved_offset_secs, resolved_abbrev, resolved_warnings_json,
                tzdb_version, calc_settings_json, view_state_json, created_at, updated_at)
             VALUES (7, 'h', 1, '', 'Event', '2000-01-01', 'Exact', '', '', '', 0, 0, 'Lmt', '',
                'u', 2451545.0, 0, '', '[]', 'v', '{}', '{}', 't', 't');
             INSERT INTO prashna (chart_id, number, created_at) VALUES (7, 54, 't');",
        )
        .unwrap();
        MIGRATIONS.to_latest(&mut conn).unwrap();
        let count = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).unwrap() };
        assert_eq!(count("PRAGMA user_version"), 3);
        assert_eq!(count("SELECT count(*) FROM charts"), 1);
        assert_eq!(count("SELECT count(*) FROM prashna"), 1);
        assert_eq!(count("SELECT count(*) FROM saved_searches"), 0);
        // UUID tektir; ad, sorgu ve damgalar zorunludur.
        let insert = |values: &str| {
            conn.execute(
                &format!(
                    "INSERT INTO saved_searches (uuid, name, query_json, created_at, updated_at)
                     VALUES {values}"
                ),
                [],
            )
        };
        assert_eq!(insert("('u1', 'Ad', '{}', 't', 't')"), Ok(1));
        assert!(insert("('u1', 'Başka', '{}', 't', 't')").is_err());
        assert!(insert("('u2', NULL, '{}', 't', 't')").is_err());
        assert!(insert("('u3', 'Ad', NULL, 't', 't')").is_err());
    }
}
