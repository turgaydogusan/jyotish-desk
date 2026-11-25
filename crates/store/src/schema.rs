//! SQLite şeması ve göçler (spec §5). Sürüm `PRAGMA user_version`'da tutulur
//! (`rusqlite_migration`). Göçler yalnızca eklenir; yayımlanmış bir göç değiştirilmez.

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

const MIGRATION_LIST: &[M<'static>] = &[M::up(V1)];

/// Bu sürümün şema sürümü (`MIGRATION_LIST.len()`).
pub const SCHEMA_VERSION: usize = 1;

pub(crate) const MIGRATIONS: Migrations<'static> = Migrations::from_slice(MIGRATION_LIST);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_valid_and_counted() {
        assert_eq!(MIGRATIONS.validate(), Ok(()));
        assert_eq!(MIGRATION_LIST.len(), SCHEMA_VERSION);
    }
}
