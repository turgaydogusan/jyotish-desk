//! `Store`: tek SQLite bağlantısı, şema göçü, saat ve `app_state` yardımcıları.

use std::path::Path;
use std::time::Duration;

use atlas::Timestamp;
use rusqlite::{params, Connection, OptionalExtension};

use crate::schema::{MIGRATIONS, SCHEMA_VERSION};
use crate::StoreError;

/// Zaman damgası kaynağı. Uygulamada `Timestamp::now`; testlerde sabit adımlı saat.
pub type Clock = Box<dyn FnMut() -> Timestamp + Send>;

/// Danışan veritabanı. `Connection` `Sync` değildir; paylaşmak için `Mutex` arkasına koyun.
pub struct Store {
    pub(crate) conn: Connection,
    clock: Clock,
    /// Son verilen zaman damgası (ms). `now` bundan büyük bir değer verir.
    last_ms: Option<i64>,
}

impl Store {
    /// Dosyayı açar (yoksa oluşturur) ve şemayı son sürüme göç ettirir.
    pub fn open(path: &Path) -> Result<Store, StoreError> {
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Store, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> Result<Store, StoreError> {
        conn.busy_timeout(Duration::from_secs(5))?;
        // Sürüm denetimi her şeyden önce: daha yeni bir veritabanına hiçbir şey yazılmaz
        // (WAL kipi de dosya başlığına yazılır).
        let found: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        let found = usize::try_from(found).unwrap_or(usize::MAX);
        if found > SCHEMA_VERSION {
            return Err(StoreError::DatabaseTooNew {
                found,
                supported: SCHEMA_VERSION,
            });
        }
        conn.pragma_update(None, "foreign_keys", true)?;
        // WAL: otomatik kayıt sık yazar; çökmede son işlem kaybolmaz. Bellekte "memory" döner.
        let _mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |r| r.get(0))?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        MIGRATIONS.to_latest(&mut conn)?;
        Ok(Store {
            conn,
            clock: Box::new(Timestamp::now),
            last_ms: None,
        })
    }

    /// Saat kaynağını değiştirir (testler ve içe aktarma testleri için).
    pub fn with_clock(mut self, clock: Clock) -> Store {
        self.clock = clock;
        self
    }

    /// Şimdi, `format_ts` biçiminde. Aynı `Store`'dan gelen damgalar milisaniye düzeyinde
    /// kesin artandır: aynı milisaniyedeki iki açılış ya da saat geri giderse bir önceki
    /// damganın 1 ms sonrası verilir. Böylece `last_opened_at` sırası açılış sırasıdır.
    pub fn now(&mut self) -> String {
        let mut ms = (self.clock)().as_millisecond();
        if let Some(last) = self.last_ms {
            ms = ms.max(last + 1);
        }
        self.last_ms = Some(ms);
        format_ts(Timestamp::from_millisecond(ms).unwrap_or(Timestamp::MAX))
    }

    /// Yarıda kesilmiş bir işin ardından bağlantıyı kullanılır duruma getirir: açık bir işlem
    /// varsa geri alınır. `Mutex<Store>` bir panikle zehirlendiğinde çağrılır (Plan 08 ertelenen
    /// madde 11). Bu crate'in işlemleri `rusqlite::Transaction` ile açılır ve panikte kendiliğinden
    /// geri alınır; buradaki denetim, bağlantının gerçekten işlem dışında olduğunu garanti eder.
    pub fn recover(&mut self) -> Result<(), StoreError> {
        if !self.conn.is_autocommit() {
            self.conn.execute_batch("ROLLBACK")?;
        }
        Ok(())
    }

    /// Veritabanının şema sürümü (`PRAGMA user_version`).
    pub fn schema_version(&self) -> Result<usize, StoreError> {
        let v: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))?;
        Ok(usize::try_from(v).unwrap_or(usize::MAX))
    }

    pub(crate) fn app_state_get(&self, key: &str) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT value_json FROM app_state WHERE key = ?1",
                params![key],
                |r| r.get(0),
            )
            .optional()?)
    }

    pub(crate) fn app_state_set(&self, key: &str, value_json: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO app_state (key, value_json) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
            params![key, value_json],
        )?;
        Ok(())
    }
}

/// Sabit genişlikli UTC zaman damgası, milisaniye: `2026-11-25T10:00:00.000Z`.
/// Sabit genişlik sayesinde metin sıralaması zaman sıralamasıdır.
pub(crate) fn format_ts(ts: Timestamp) -> String {
    ts.strftime("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Son eklenen satırın kimliği. IPC kimlikleri `u32`'dir (specta `i64`'ü reddeder).
pub(crate) fn last_id(conn: &Connection) -> Result<u32, StoreError> {
    u32::try_from(conn.last_insert_rowid()).map_err(|_| StoreError::InvalidInput("id"))
}

/// Metin sütunundaki geçersiz değer için rusqlite dönüşüm hatası.
pub(crate) fn bad_column(idx: usize, what: &'static str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(idx, rusqlite::types::Type::Text, what.into())
}

#[cfg(test)]
pub(crate) mod testutil {
    use super::*;

    /// 2025-11-24T16:00:00Z'den başlayıp her çağrıda bir saniye ilerleyen saat.
    pub fn step_clock() -> Clock {
        let mut n = 0_i64;
        Box::new(move || {
            n += 1;
            Timestamp::from_second(1_764_000_000 + n).expect("valid timestamp")
        })
    }

    pub fn store() -> Store {
        Store::open_in_memory()
            .expect("open in memory")
            .with_clock(step_clock())
    }

    /// Hiç ilerlemeyen saat (2025-11-24T16:00:00Z): aynı milisaniyedeki işlemler için.
    pub fn stuck_store() -> Store {
        Store::open_in_memory()
            .expect("open in memory")
            .with_clock(Box::new(|| {
                Timestamp::from_second(1_764_000_000).expect("valid timestamp")
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::testutil::{store, stuck_store};
    use super::*;

    #[test]
    fn timestamps_strictly_increase_when_the_clock_does_not() {
        let mut s = stuck_store();
        assert_eq!(s.now(), "2025-11-24T16:00:00.000Z");
        assert_eq!(s.now(), "2025-11-24T16:00:00.001Z");
        assert_eq!(s.now(), "2025-11-24T16:00:00.002Z");
    }

    #[test]
    fn recover_rolls_back_an_open_transaction_and_is_a_no_op_otherwise() {
        let mut s = store();
        s.recover().unwrap();
        s.conn
            .execute_batch("BEGIN; INSERT INTO app_state (key, value_json) VALUES ('half', '1');")
            .unwrap();
        assert!(!s.conn.is_autocommit());
        s.recover().unwrap();
        assert!(s.conn.is_autocommit());
        assert_eq!(s.app_state_get("half").unwrap(), None);
    }

    #[test]
    fn fresh_database_has_schema_and_foreign_keys() {
        let s = store();
        assert_eq!(s.schema_version().unwrap(), SCHEMA_VERSION);
        let fk: i64 = s
            .conn
            .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fk, 1);
        let tables: i64 = s
            .conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name IN
                 ('clients', 'charts', 'client_notes', 'tags', 'chart_tags', 'app_state',
                  'clients_fts', 'notes_fts', 'prashna')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(tables, 9);
    }

    #[test]
    fn reopening_a_file_keeps_data_and_version() {
        let dir = std::env::temp_dir().join(format!("jd-store-reopen-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("db.sqlite");
        {
            let s = Store::open(&path).unwrap();
            s.conn
                .execute(
                    "INSERT INTO app_state (key, value_json) VALUES ('k', '1')",
                    [],
                )
                .unwrap();
        }
        let s = Store::open(&path).unwrap();
        assert_eq!(s.schema_version().unwrap(), SCHEMA_VERSION);
        let v: String = s
            .conn
            .query_row(
                "SELECT value_json FROM app_state WHERE key = 'k'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(v, "1");
        drop(s);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn newer_database_is_refused_and_left_untouched() {
        let dir = std::env::temp_dir().join(format!("jd-store-newer-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("db.sqlite");
        {
            let c = Connection::open(&path).unwrap();
            c.pragma_update(None, "user_version", 99).unwrap();
        }
        match Store::open(&path) {
            Err(StoreError::DatabaseTooNew { found, supported }) => {
                assert_eq!((found, supported), (99, SCHEMA_VERSION));
            }
            other => panic!("expected DatabaseTooNew, got {:?}", other.err()),
        }
        let c = Connection::open(&path).unwrap();
        let v: i64 = c
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, 99);
        let tables: i64 = c
            .query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get(0))
            .unwrap();
        assert_eq!(tables, 0);
        let mode: String = c
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        assert_eq!(mode, "delete");
        drop(c);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn app_state_upserts() {
        let s = store();
        assert_eq!(s.app_state_get("a").unwrap(), None);
        s.app_state_set("a", "1").unwrap();
        s.app_state_set("a", "2").unwrap();
        assert_eq!(s.app_state_get("a").unwrap().as_deref(), Some("2"));
    }

    #[test]
    fn timestamps_are_fixed_width_utc_millis() {
        let mut s = store();
        assert_eq!(s.now(), "2025-11-24T16:00:01.000Z");
        assert_eq!(s.now(), "2025-11-24T16:00:02.000Z");
        let t = Timestamp::from_millisecond(1_764_000_000_123).unwrap();
        assert_eq!(format_ts(t), "2025-11-24T16:00:00.123Z");
    }
}
