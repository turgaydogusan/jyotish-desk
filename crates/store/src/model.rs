//! IPC'yi geçen kayıt tipleri (spec §5 `BirthData`). JSON alan adları snake_case, enum
//! varyantları PascalCase dizgidir. Kimlikler `u32`, zaman damgaları `format_ts` dizgisidir.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Gender {
    Male,
    Female,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum TimeCertainty {
    Exact,
    Approx,
    /// Saat bilinmiyor: `BirthData.time` boştur, hesap yerel 12:00 ile yapılır.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum ChartKind {
    Natal,
    /// Olay haritası (evlilik, kuruluş, soru anı…).
    Event,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum FoldChoice {
    Earlier,
    Later,
}

/// Saat dilimi seçimi (spec §5 `tz {Iana(id) | Fixed(secs) | Lmt}`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum TzChoice {
    Iana(String),
    /// UTC ofseti, saniye, doğuda pozitif.
    Fixed(i32),
    Lmt,
}

/// Çözüm uyarıları (atlas `MomentWarning`'in IPC karşılığı; UI spec §5 rozetleri).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum MomentWarning {
    TimeUnknownNoon,
    LmtFromLongitude,
    PreTzdbReliability,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Place {
    /// Manuel koordinatta boş olabilir.
    pub name: String,
    pub country: String,
    pub admin1: String,
    pub lat: f64,
    pub lon: f64,
    pub geoname_id: Option<u32>,
}

/// Bulunulan yer (ileri modüller spec'i §10): gün panchangının ve prashnanın varsayılan yeri.
/// Kişisel veridir: loglanmaz, hata parametresine girmez, dışa aktarıma yazılmaz. Saat dilimi
/// yerin duvar saatini göstermek içindir (doğum verisindeki `tz` ile aynı seçim).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct HomePlace {
    pub place: Place,
    pub tz: TzChoice,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct BirthData {
    /// Yerel takvim tarihi, `YYYY-MM-DD`.
    pub date: String,
    /// Yerel saat, `HH:MM:SS` (kesirli saniye olabilir). Saat bilinmiyorsa boş.
    pub time: Option<String>,
    pub time_certainty: TimeCertainty,
    pub place: Place,
    pub tz: TzChoice,
    pub fold: Option<FoldChoice>,
    /// Doğum verisinin kaynağı gibi kısa not (seans notları `Note`'tadır).
    pub notes: String,
}

/// Kayıtta saklanan çözülmüş an (spec §5 `resolved_*`, `tzdb_version`). Açılışta atlas
/// sorgusu gerekmez; hesap `jd_ut` ile yapılır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ResolvedRecord {
    /// RFC 3339 UTC (`2000-01-01T12:00:00Z`).
    pub utc: String,
    pub jd_ut: f64,
    pub offset_secs: i32,
    pub abbrev: String,
    pub warnings: Vec<MomentWarning>,
    pub tzdb_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Client {
    pub id: u32,
    pub uuid: String,
    pub name: String,
    pub gender: Option<Gender>,
    pub last_chart_id: Option<u32>,
    pub last_opened_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ChartRecord {
    pub id: u32,
    pub uuid: String,
    pub client_id: u32,
    /// Boşsa arayüz harita türünün yerelleştirilmiş adını gösterir.
    pub label: String,
    pub kind: ChartKind,
    pub birth: BirthData,
    pub resolved: ResolvedRecord,
    /// Ada göre sıralı (büyük/küçük harf duyarsız).
    pub tags: Vec<String>,
    pub last_opened_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Note {
    pub id: u32,
    pub uuid: String,
    pub client_id: u32,
    pub chart_id: Option<u32>,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct NewChart {
    pub label: String,
    pub kind: ChartKind,
    pub birth: BirthData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct NewClient {
    pub name: String,
    pub gender: Option<Gender>,
    pub chart: NewChart,
}
