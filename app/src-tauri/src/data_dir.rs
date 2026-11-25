//! Veri klasörünün ve veritabanı dosyalarının izinleri (Plan 08 ertelenen madde 10).
//! Danışan verisi şifresiz SQLite'ta durur (spec §12); klasör yalnızca sahibine açıktır (0700),
//! dosyalar yalnızca sahibince okunur ve yazılır (0600). Kullanıcının seçtiği yere yazılan dışa
//! aktarma dosyaları bu kuralın dışındadır.

use std::io;
use std::path::{Path, PathBuf};

/// Klasörü oluşturur (yoksa) ve izinlerini 0700 yapar; var olan klasör de daraltılır.
pub fn prepare(dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    set_mode(dir, 0o700)
}

/// Veritabanı dosyası ve SQLite'ın yan dosyaları (`-wal`, `-shm`); olmayan dosya atlanır.
pub fn restrict_db_files(db: &Path) -> io::Result<()> {
    for path in db_files(db) {
        if path.exists() {
            set_mode(&path, 0o600)?;
        }
    }
    Ok(())
}

fn db_files(db: &Path) -> [PathBuf; 3] {
    let with = |suffix: &str| {
        let mut name = db.as_os_str().to_owned();
        name.push(suffix);
        PathBuf::from(name)
    };
    [db.to_path_buf(), with("-wal"), with("-shm")]
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> io::Result<()> {
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn mode(path: &Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn a_new_and_an_existing_data_directory_are_owner_only() {
        let tmp = tempfile::tempdir().unwrap();
        let fresh = tmp.path().join("a/b");
        prepare(&fresh).unwrap();
        assert_eq!(mode(&fresh), 0o700);

        let open = tmp.path().join("open");
        std::fs::create_dir(&open).unwrap();
        std::fs::set_permissions(&open, std::fs::Permissions::from_mode(0o755)).unwrap();
        prepare(&open).unwrap();
        assert_eq!(mode(&open), 0o700);
    }

    #[test]
    fn the_database_and_its_side_files_are_owner_read_write_and_missing_ones_are_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let db = tmp.path().join("clients.sqlite");
        let wal = tmp.path().join("clients.sqlite-wal");
        for f in [&db, &wal] {
            std::fs::write(f, b"x").unwrap();
            std::fs::set_permissions(f, std::fs::Permissions::from_mode(0o644)).unwrap();
        }
        restrict_db_files(&db).unwrap();
        assert_eq!((mode(&db), mode(&wal)), (0o600, 0o600));
        assert!(!tmp.path().join("clients.sqlite-shm").exists());
    }

    #[test]
    fn a_real_store_keeps_working_after_its_files_are_restricted() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("data");
        prepare(&dir).unwrap();
        let db = dir.join("clients.sqlite");
        let store = store::Store::open(&db).unwrap();
        restrict_db_files(&db).unwrap();
        assert_eq!(mode(&db), 0o600);
        store.set_app_prefs(&store::AppPrefs::default()).unwrap();
        assert_eq!(store.app_prefs().unwrap(), store::AppPrefs::default());
    }
}
