//! Danışan ve harita kayıtları (spec §4 "Kalıcılık", §5 SQLite şeması). Hesaplanmış harita
//! saklanmaz; doğum verisi, çözülmüş UTC anı, hesap ayarları, görünüm durumu ve notlar
//! saklanır. Hesap yapmaz: `jyotish-core`'dan yalnızca tipleri kullanır.

mod db;
mod error;
pub mod schema;

pub use db::{Clock, Store};
pub use error::StoreError;
pub use schema::SCHEMA_VERSION;
