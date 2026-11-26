//! Danışan ve harita kayıtları: oluşturma (ayar kopyası, spec §4), okuma, güncelleme,
//! silme ve görünüm durumunun iki yere yazılması (bkz. `view_state`).

use jyotish_core::CalcSettings;
use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use crate::birth::resolve_birth;
use crate::compat::{clear_compat_links, settled_compat};
use crate::db::{bad_column, last_id};
use crate::model::{
    BirthData, ChartKind, ChartRecord, Client, FoldChoice, Gender, MomentWarning, NewChart,
    NewClient, Place, ResolvedRecord, TimeCertainty, TzChoice,
};
use crate::settings::{load_calc_settings, SettingsLoad};
use crate::view_state::{
    chart_view_chara_path, chart_view_compat, chart_view_json, chart_view_path,
    empty_chart_view_json, parse_panel_layout, PanelLayout, ViewState,
};
use crate::{Store, StoreError};

/// `app_state` anahtarı: kullanıcının pano düzenleri (UI spec §3.2).
pub(crate) const PANEL_LAYOUT_KEY: &str = "panel_layout";
/// Ad ve etiket uzunluk sınırı (karakter).
pub const MAX_NAME_CHARS: usize = 200;
/// `list_clients` ve aramanın sonuç üst sınırı.
pub const MAX_LIST: u32 = 500;

macro_rules! text_enum {
    ($t:ident { $($v:ident),+ $(,)? }) => {
        impl $t {
            pub(crate) fn as_str(self) -> &'static str {
                match self { $(Self::$v => stringify!($v)),+ }
            }
            pub(crate) fn parse(s: &str) -> Option<Self> {
                match s { $(stringify!($v) => Some(Self::$v),)+ _ => None }
            }
        }
    };
}
text_enum!(Gender {
    Male,
    Female,
    Other
});
text_enum!(TimeCertainty {
    Exact,
    Approx,
    Unknown
});
text_enum!(ChartKind { Natal, Event });
text_enum!(FoldChoice { Earlier, Later });

pub(crate) fn clean_name(name: &str, field: &'static str) -> Result<String, StoreError> {
    let t = name.trim();
    if t.chars().count() > MAX_NAME_CHARS || (field == "name" && t.is_empty()) {
        return Err(StoreError::InvalidInput(field));
    }
    Ok(t.to_owned())
}

const CLIENT_COLUMNS: &str =
    "id, uuid, name, gender, last_chart_id, last_opened_at, created_at, updated_at";

pub(crate) fn row_to_client(r: &Row<'_>) -> rusqlite::Result<Client> {
    let gender: Option<String> = r.get(3)?;
    Ok(Client {
        id: r.get(0)?,
        uuid: r.get(1)?,
        name: r.get(2)?,
        gender: match gender {
            Some(g) => Some(Gender::parse(&g).ok_or_else(|| bad_column(3, "gender"))?),
            None => None,
        },
        last_chart_id: r.get(4)?,
        last_opened_at: r.get(5)?,
        created_at: r.get(6)?,
        updated_at: r.get(7)?,
    })
}

const CHART_COLUMNS: &str = "id, uuid, client_id, label, kind, date, time, time_certainty, \
     place_name, country, admin1, lat, lon, geoname_id, tz_kind, tz_id, offset_override, fold, \
     birth_notes, resolved_utc, resolved_jd_ut, resolved_offset_secs, resolved_abbrev, \
     resolved_warnings_json, tzdb_version, last_opened_at, created_at, updated_at";

/// Etiketler hariç harita satırı; `tags` ayrı sorguyla doldurulur.
fn row_to_chart(r: &Row<'_>) -> rusqlite::Result<ChartRecord> {
    let kind: String = r.get(4)?;
    let certainty: String = r.get(7)?;
    let tz_kind: String = r.get(14)?;
    let tz = match tz_kind.as_str() {
        "Iana" => TzChoice::Iana(
            r.get::<_, Option<String>>(15)?
                .ok_or_else(|| bad_column(15, "tz_id"))?,
        ),
        "Fixed" => TzChoice::Fixed(
            r.get::<_, Option<i32>>(16)?
                .ok_or_else(|| bad_column(16, "offset_override"))?,
        ),
        "Lmt" => TzChoice::Lmt,
        _ => return Err(bad_column(14, "tz_kind")),
    };
    let fold: Option<String> = r.get(17)?;
    let warnings: String = r.get(23)?;
    Ok(ChartRecord {
        id: r.get(0)?,
        uuid: r.get(1)?,
        client_id: r.get(2)?,
        label: r.get(3)?,
        kind: ChartKind::parse(&kind).ok_or_else(|| bad_column(4, "kind"))?,
        birth: BirthData {
            date: r.get(5)?,
            time: r.get(6)?,
            time_certainty: TimeCertainty::parse(&certainty)
                .ok_or_else(|| bad_column(7, "time_certainty"))?,
            place: Place {
                name: r.get(8)?,
                country: r.get(9)?,
                admin1: r.get(10)?,
                lat: r.get(11)?,
                lon: r.get(12)?,
                geoname_id: r.get(13)?,
            },
            tz,
            fold: match fold {
                Some(f) => Some(FoldChoice::parse(&f).ok_or_else(|| bad_column(17, "fold"))?),
                None => None,
            },
            notes: r.get(18)?,
        },
        resolved: ResolvedRecord {
            utc: r.get(19)?,
            jd_ut: r.get(20)?,
            offset_secs: r.get(21)?,
            abbrev: r.get(22)?,
            warnings: serde_json::from_str::<Vec<MomentWarning>>(&warnings)
                .map_err(|_| bad_column(23, "resolved_warnings_json"))?,
            tzdb_version: r.get(24)?,
        },
        tags: Vec::new(),
        last_opened_at: r.get(25)?,
        created_at: r.get(26)?,
        updated_at: r.get(27)?,
    })
}

fn chart_tags(conn: &Connection, chart_id: u32) -> Result<Vec<String>, StoreError> {
    let mut stmt = conn.prepare_cached(
        "SELECT t.name FROM chart_tags ct JOIN tags t ON t.id = ct.tag_id
         WHERE ct.chart_id = ?1 ORDER BY t.name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![chart_id], |r| r.get(0))?;
    Ok(rows.collect::<Result<Vec<String>, _>>()?)
}

pub(crate) fn load_chart(conn: &Connection, id: u32) -> Result<ChartRecord, StoreError> {
    let sql = format!("SELECT {CHART_COLUMNS} FROM charts WHERE id = ?1");
    let mut chart = conn
        .query_row(&sql, params![id], row_to_chart)
        .optional()?
        .ok_or(StoreError::NotFound { what: "chart", id })?;
    chart.tags = chart_tags(conn, id)?;
    Ok(chart)
}

pub(crate) fn load_client(conn: &Connection, id: u32) -> Result<Client, StoreError> {
    let sql = format!("SELECT {CLIENT_COLUMNS} FROM clients WHERE id = ?1");
    conn.query_row(&sql, params![id], row_to_client)
        .optional()?
        .ok_or(StoreError::NotFound { what: "client", id })
}

pub(crate) fn insert_client(
    conn: &Connection,
    uuid: &str,
    name: &str,
    gender: Option<Gender>,
    created_at: &str,
    updated_at: &str,
) -> Result<u32, StoreError> {
    conn.execute(
        "INSERT INTO clients (uuid, name, gender, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            uuid,
            name,
            gender.map(Gender::as_str),
            created_at,
            updated_at
        ],
    )?;
    last_id(conn)
}

/// Bir harita satırının tüm alanları. `birth` kanonik biçimdedir (`parse_birth`).
pub(crate) struct ChartRow<'a> {
    pub uuid: &'a str,
    pub client_id: u32,
    pub label: &'a str,
    pub kind: ChartKind,
    pub birth: &'a BirthData,
    pub resolved: &'a ResolvedRecord,
    pub calc_json: &'a str,
    pub view_json: &'a str,
    pub created_at: &'a str,
    pub updated_at: &'a str,
}

fn tz_columns(tz: &TzChoice) -> (&'static str, Option<&str>, Option<i32>) {
    match tz {
        TzChoice::Iana(id) => ("Iana", Some(id.as_str()), None),
        TzChoice::Fixed(secs) => ("Fixed", None, Some(*secs)),
        TzChoice::Lmt => ("Lmt", None, None),
    }
}

pub(crate) fn insert_chart(conn: &Connection, c: &ChartRow<'_>) -> Result<u32, StoreError> {
    let b = c.birth;
    let (tz_kind, tz_id, offset) = tz_columns(&b.tz);
    let warnings = serde_json::to_string(&c.resolved.warnings)?;
    conn.execute(
        "INSERT INTO charts (uuid, client_id, label, kind, date, time, time_certainty,
            place_name, country, admin1, lat, lon, geoname_id, tz_kind, tz_id, offset_override,
            fold, birth_notes, resolved_utc, resolved_jd_ut, resolved_offset_secs,
            resolved_abbrev, resolved_warnings_json, tzdb_version, calc_settings_json,
            view_state_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17,
            ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28)",
        params![
            c.uuid,
            c.client_id,
            c.label,
            c.kind.as_str(),
            b.date,
            b.time,
            b.time_certainty.as_str(),
            b.place.name,
            b.place.country,
            b.place.admin1,
            b.place.lat,
            b.place.lon,
            b.place.geoname_id,
            tz_kind,
            tz_id,
            offset,
            b.fold.map(FoldChoice::as_str),
            b.notes,
            c.resolved.utc,
            c.resolved.jd_ut,
            c.resolved.offset_secs,
            c.resolved.abbrev,
            warnings,
            c.resolved.tzdb_version,
            c.calc_json,
            c.view_json,
            c.created_at,
            c.updated_at,
        ],
    )?;
    last_id(conn)
}

/// Kullanılmayan etiketleri siler.
pub(crate) fn prune_tags(conn: &Connection) -> Result<(), StoreError> {
    conn.execute(
        "DELETE FROM tags WHERE id NOT IN (SELECT tag_id FROM chart_tags)",
        [],
    )?;
    Ok(())
}

impl Store {
    /// Danışanı ilk haritasıyla oluşturur. Harita, genel varsayılan ayarların **tam
    /// kopyasını** alır (spec §4); görünüm durumu boş dasha yoludur.
    pub fn create_client(
        &mut self,
        input: &NewClient,
    ) -> Result<(Client, ChartRecord), StoreError> {
        let name = clean_name(&input.name, "name")?;
        let label = clean_name(&input.chart.label, "label")?;
        let (birth, resolved) = resolve_birth(&input.chart.birth)?;
        let calc_json = serde_json::to_string(&self.app_prefs()?.calc_settings)?;
        let now = self.now();
        let tx = self.conn.transaction()?;
        let client_id = insert_client(
            &tx,
            &Uuid::new_v4().to_string(),
            &name,
            input.gender,
            &now,
            &now,
        )?;
        let chart_id = insert_chart(
            &tx,
            &ChartRow {
                uuid: &Uuid::new_v4().to_string(),
                client_id,
                label: &label,
                kind: input.chart.kind,
                birth: &birth,
                resolved: &resolved,
                calc_json: &calc_json,
                view_json: &empty_chart_view_json(),
                created_at: &now,
                updated_at: &now,
            },
        )?;
        tx.execute(
            "UPDATE clients SET last_chart_id = ?1 WHERE id = ?2",
            params![chart_id, client_id],
        )?;
        tx.commit()?;
        Ok((self.client(client_id)?, self.chart(chart_id)?))
    }

    /// Danışana yeni harita ekler (olay haritası, düzeltilmiş saat…); ayar kopyası kuralı aynı.
    pub fn add_chart(
        &mut self,
        client_id: u32,
        chart: &NewChart,
    ) -> Result<ChartRecord, StoreError> {
        self.client(client_id)?;
        let label = clean_name(&chart.label, "label")?;
        let (birth, resolved) = resolve_birth(&chart.birth)?;
        let calc_json = serde_json::to_string(&self.app_prefs()?.calc_settings)?;
        let now = self.now();
        // Harita ve danışanın `updated_at`'i birlikte yazılır (Plan 08 ertelenen madde 9).
        let tx = self.conn.transaction()?;
        let id = insert_chart(
            &tx,
            &ChartRow {
                uuid: &Uuid::new_v4().to_string(),
                client_id,
                label: &label,
                kind: chart.kind,
                birth: &birth,
                resolved: &resolved,
                calc_json: &calc_json,
                view_json: &empty_chart_view_json(),
                created_at: &now,
                updated_at: &now,
            },
        )?;
        tx.execute(
            "UPDATE clients SET updated_at = ?1 WHERE id = ?2",
            params![now, client_id],
        )?;
        tx.commit()?;
        self.chart(id)
    }

    pub fn client(&self, id: u32) -> Result<Client, StoreError> {
        load_client(&self.conn, id)
    }

    pub fn chart(&self, id: u32) -> Result<ChartRecord, StoreError> {
        load_chart(&self.conn, id)
    }

    /// Danışanın haritaları, en son güncellenen önce.
    pub fn charts_of(&self, client_id: u32) -> Result<Vec<ChartRecord>, StoreError> {
        self.client(client_id)?;
        let mut stmt = self.conn.prepare_cached(
            "SELECT id FROM charts WHERE client_id = ?1 ORDER BY updated_at DESC, id DESC",
        )?;
        let ids = stmt
            .query_map(params![client_id], |r| r.get::<_, u32>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        ids.into_iter().map(|id| self.chart(id)).collect()
    }

    /// Kenar çubuğu listesi: son açılan önce, hiç açılmamışlar ada göre sonda.
    pub fn list_clients(&self, limit: u32) -> Result<Vec<Client>, StoreError> {
        let sql = format!(
            "SELECT {CLIENT_COLUMNS} FROM clients
             ORDER BY last_opened_at DESC NULLS LAST, name COLLATE NOCASE, id LIMIT ?1"
        );
        let mut stmt = self.conn.prepare_cached(&sql)?;
        let rows = stmt.query_map(params![limit.min(MAX_LIST)], row_to_client)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn update_client(
        &mut self,
        id: u32,
        name: &str,
        gender: Option<Gender>,
    ) -> Result<Client, StoreError> {
        let name = clean_name(name, "name")?;
        let now = self.now();
        let n = self.conn.execute(
            "UPDATE clients SET name = ?1, gender = ?2, updated_at = ?3 WHERE id = ?4",
            params![name, gender.map(Gender::as_str), now, id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound { what: "client", id });
        }
        self.client(id)
    }

    /// "Düzenle" kipi: etiket, tür ve doğum verisi; an yeniden çözülür ve güncel tzdb
    /// sürümü yazılır. Ayarlar ve görünüm durumu değişmez.
    pub fn update_chart(
        &mut self,
        chart_id: u32,
        chart: &NewChart,
    ) -> Result<ChartRecord, StoreError> {
        let label = clean_name(&chart.label, "label")?;
        let (b, r) = resolve_birth(&chart.birth)?;
        let (tz_kind, tz_id, offset) = tz_columns(&b.tz);
        let warnings = serde_json::to_string(&r.warnings)?;
        let now = self.now();
        let n = self.conn.execute(
            "UPDATE charts SET label = ?1, kind = ?2, date = ?3, time = ?4, time_certainty = ?5,
                place_name = ?6, country = ?7, admin1 = ?8, lat = ?9, lon = ?10, geoname_id = ?11,
                tz_kind = ?12, tz_id = ?13, offset_override = ?14, fold = ?15, birth_notes = ?16,
                resolved_utc = ?17, resolved_jd_ut = ?18, resolved_offset_secs = ?19,
                resolved_abbrev = ?20, resolved_warnings_json = ?21, tzdb_version = ?22,
                updated_at = ?23
             WHERE id = ?24",
            params![
                label,
                chart.kind.as_str(),
                b.date,
                b.time,
                b.time_certainty.as_str(),
                b.place.name,
                b.place.country,
                b.place.admin1,
                b.place.lat,
                b.place.lon,
                b.place.geoname_id,
                tz_kind,
                tz_id,
                offset,
                b.fold.map(FoldChoice::as_str),
                b.notes,
                r.utc,
                r.jd_ut,
                r.offset_secs,
                r.abbrev,
                warnings,
                r.tzdb_version,
                now,
                chart_id,
            ],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound {
                what: "chart",
                id: chart_id,
            });
        }
        self.chart(chart_id)
    }

    /// tzdb farkında "Saklanan zamanı koru" (Plan 11 D5): saklanan an değişmez, kayıt güncel
    /// tzdb ile damgalanır; sonraki açılışta `TzdbChanged` çıkmaz. Kabul yolu `update_chart`'tır.
    pub fn keep_stored_moment(&mut self, chart_id: u32) -> Result<ChartRecord, StoreError> {
        let now = self.now();
        let n = self.conn.execute(
            "UPDATE charts SET tzdb_version = ?1, updated_at = ?2 WHERE id = ?3",
            params![atlas::tzdb_version(), now, chart_id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound {
                what: "chart",
                id: chart_id,
            });
        }
        self.chart(chart_id)
    }

    /// Saklanan ayarları okur; okunamazsa genel varsayılan (`SettingsLoad::Reset`).
    pub fn calc_settings(&self, chart_id: u32) -> Result<SettingsLoad, StoreError> {
        let raw: String = self
            .conn
            .query_row(
                "SELECT calc_settings_json FROM charts WHERE id = ?1",
                params![chart_id],
                |r| r.get(0),
            )
            .optional()?
            .ok_or(StoreError::NotFound {
                what: "chart",
                id: chart_id,
            })?;
        Ok(load_calc_settings(&raw, &self.app_prefs()?.calc_settings))
    }

    /// Haritanın hesap ayarı (otomatik kayıt, "varsayılana döndür"). Geçersiz ayar reddedilir.
    pub fn save_calc_settings(
        &mut self,
        chart_id: u32,
        settings: &CalcSettings,
    ) -> Result<(), StoreError> {
        settings.validate()?;
        let json = serde_json::to_string(settings)?;
        let now = self.now();
        let n = self.conn.execute(
            "UPDATE charts SET calc_settings_json = ?1, updated_at = ?2 WHERE id = ?3",
            params![json, now, chart_id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound {
                what: "chart",
                id: chart_id,
            });
        }
        Ok(())
    }

    /// Kullanıcının pano düzenleri; kayıt yoksa ya da okunamazsa `None`.
    pub fn panel_layout(&self) -> Result<Option<PanelLayout>, StoreError> {
        Ok(self
            .app_state_get(PANEL_LAYOUT_KEY)?
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .and_then(|v| parse_panel_layout(&v)))
    }

    /// Görünüm durumu: düzenler `app_state`'ten, iki dasha yolu ve eş bağı haritadan
    /// (toleranslı). Eş bağı yalnızca var olan başka bir haritayı gösteriyorsa döner.
    pub fn view_state(&self, chart_id: u32) -> Result<ViewState, StoreError> {
        let raw: String = self
            .conn
            .query_row(
                "SELECT view_state_json FROM charts WHERE id = ?1",
                params![chart_id],
                |r| r.get(0),
            )
            .optional()?
            .ok_or(StoreError::NotFound {
                what: "chart",
                id: chart_id,
            })?;
        let (partner, roles) = chart_view_compat(&raw);
        let (compat_partner_uuid, compat_roles) =
            settled_compat(&self.conn, chart_id, partner.as_deref(), roles)?;
        Ok(ViewState {
            panel_layout: self.panel_layout()?,
            dasha_expanded_path: chart_view_path(&raw),
            chara_expanded_path: chart_view_chara_path(&raw),
            compat_partner_uuid,
            compat_roles,
            ..ViewState::default()
        })
    }

    /// Otomatik kayıt (≈ 500 ms debounce arayüzde). `updated_at` değişmez: görünüm içerik
    /// değildir. `panel_layout: None` kayıtlı düzeni siler (arayüz varsayılana döner). Eş bağı
    /// yalnızca var olan başka bir haritayı gösteriyorsa yazılır; kendisini ya da silinmiş bir
    /// haritayı gösteren bağ sessizce bırakılır (silinen eş, geç gelen bir kayıtla geri gelmez).
    pub fn save_view_state(&mut self, chart_id: u32, vs: &ViewState) -> Result<(), StoreError> {
        crate::view_state::check_view_state(vs)?;
        let tx = self.conn.transaction()?;
        let (compat_partner_uuid, compat_roles) = settled_compat(
            &tx,
            chart_id,
            vs.compat_partner_uuid.as_deref(),
            vs.compat_roles,
        )?;
        let chart_part = ViewState {
            compat_partner_uuid,
            compat_roles,
            ..vs.clone()
        };
        let n = tx.execute(
            "UPDATE charts SET view_state_json = ?1 WHERE id = ?2",
            params![chart_view_json(&chart_part), chart_id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound {
                what: "chart",
                id: chart_id,
            });
        }
        match &vs.panel_layout {
            Some(pl) => {
                tx.execute(
                    "INSERT INTO app_state (key, value_json) VALUES (?1, ?2)
                     ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
                    params![PANEL_LAYOUT_KEY, serde_json::to_string(pl)?],
                )?;
            }
            None => {
                tx.execute(
                    "DELETE FROM app_state WHERE key = ?1",
                    params![PANEL_LAYOUT_KEY],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Danışanı haritaları, notları ve etiket bağlarıyla kalıcı olarak siler (onay arayüzde).
    /// Haritalarını eş olarak gösteren uyum bağları aynı işlemde temizlenir.
    pub fn delete_client(&mut self, id: u32) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;
        let removed: Vec<String> = {
            let mut stmt = tx.prepare("SELECT uuid FROM charts WHERE client_id = ?1")?;
            let rows = stmt.query_map(params![id], |r| r.get(0))?;
            rows.collect::<Result<_, _>>()?
        };
        if tx.execute("DELETE FROM clients WHERE id = ?1", params![id])? == 0 {
            return Err(StoreError::NotFound { what: "client", id });
        }
        clear_compat_links(&tx, &removed)?;
        prune_tags(&tx)?;
        tx.commit()?;
        Ok(())
    }

    /// Haritayı siler; danışanın son haritası silinemez (`LastChart`). Bağlı notlar
    /// danışanda durur (`chart_id` boşalır). Haritayı eş olarak gösteren uyum bağları aynı
    /// işlemde temizlenir.
    pub fn delete_chart(&mut self, id: u32) -> Result<(), StoreError> {
        let chart = self.chart(id)?;
        // Sayım ve silme aynı işlemde: arada eklenen/silinen harita kuralı bozamaz.
        let tx = self.conn.transaction()?;
        let count: u32 = tx.query_row(
            "SELECT count(*) FROM charts WHERE client_id = ?1",
            params![chart.client_id],
            |r| r.get(0),
        )?;
        if count <= 1 {
            return Err(StoreError::LastChart);
        }
        tx.execute("DELETE FROM charts WHERE id = ?1", params![id])?;
        clear_compat_links(&tx, &[chart.uuid])?;
        prune_tags(&tx)?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod testutil {
    use crate::birth::fixtures::delhi;
    use crate::model::{ChartKind, NewChart, NewClient};

    pub fn new_client(name: &str) -> NewClient {
        NewClient {
            name: name.into(),
            gender: None,
            chart: NewChart {
                label: String::new(),
                kind: ChartKind::Natal,
                birth: delhi(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testutil::new_client;
    use super::*;
    use crate::birth::fixtures::{delhi, istanbul};
    use crate::db::testutil::store;
    use crate::settings::AppPrefs;
    use crate::view_state::{
        empty_chara_path, Layout, Panel, MAX_LAYOUTS, MAX_PANELS, MAX_VIEW_STATE_BYTES,
    };
    use jyotish_core::{AyanamsaSystem, Graha, Sign, SignDashaPath};
    use serde_json::Map;

    #[test]
    fn create_client_copies_current_defaults() {
        let mut s = store();
        let raman = CalcSettings {
            ayanamsa: AyanamsaSystem::Raman,
            ..CalcSettings::default()
        };
        s.set_app_prefs(&AppPrefs {
            calc_settings: raman.clone(),
            ..AppPrefs::default()
        })
        .unwrap();
        let (c, ch) = s
            .create_client(&NewClient {
                gender: Some(Gender::Female),
                ..new_client("  Ayşe Yılmaz ")
            })
            .unwrap();
        assert_eq!(c.name, "Ayşe Yılmaz");
        assert_eq!(c.gender, Some(Gender::Female));
        assert_eq!(c.last_chart_id, Some(ch.id));
        assert_eq!(c.uuid.len(), 36);
        assert_eq!(ch.client_id, c.id);
        assert_eq!(ch.birth.time.as_deref(), Some("17:30:00"));
        assert_eq!(ch.resolved.jd_ut, 2_451_545.0);
        assert_eq!(
            s.calc_settings(ch.id).unwrap(),
            SettingsLoad::Current(raman.clone())
        );

        // Genel varsayılan sonradan değişir; mevcut harita sessizce değişmez.
        s.set_app_prefs(&AppPrefs::default()).unwrap();
        assert_eq!(
            s.calc_settings(ch.id).unwrap(),
            SettingsLoad::Current(raman)
        );
        let ch2 = s
            .add_chart(
                c.id,
                &NewChart {
                    label: "Evlilik".into(),
                    kind: ChartKind::Event,
                    birth: istanbul("2016-06-01", "14:00"),
                },
            )
            .unwrap();
        assert_eq!(
            s.calc_settings(ch2.id).unwrap(),
            SettingsLoad::Current(CalcSettings::default())
        );
        let ids: Vec<u32> = s.charts_of(c.id).unwrap().iter().map(|x| x.id).collect();
        assert_eq!(ids, [ch2.id, ch.id]);
    }

    #[test]
    fn invalid_input_is_rejected_before_writing() {
        let mut s = store();
        assert!(matches!(
            s.create_client(&new_client("   ")),
            Err(StoreError::InvalidInput("name"))
        ));
        let mut bad = new_client("A");
        bad.chart.birth.date = "yesterday".into();
        assert!(matches!(
            s.create_client(&bad),
            Err(StoreError::InvalidInput("birth.date"))
        ));
        let mut long = new_client("A");
        long.chart.label = "x".repeat(MAX_NAME_CHARS + 1);
        assert!(matches!(
            s.create_client(&long),
            Err(StoreError::InvalidInput("label"))
        ));
        assert!(s.list_clients(10).unwrap().is_empty());
    }

    #[test]
    fn update_chart_re_resolves_and_keeps_settings() {
        let mut s = store();
        let (_, ch) = s.create_client(&new_client("A")).unwrap();
        let settings = CalcSettings {
            dasha_year: 360.0,
            ..CalcSettings::default()
        };
        s.save_calc_settings(ch.id, &settings).unwrap();
        let up = s
            .update_chart(
                ch.id,
                &NewChart {
                    label: "Düzeltilmiş".into(),
                    kind: ChartKind::Natal,
                    birth: BirthData {
                        time: Some("17:31".into()),
                        ..delhi()
                    },
                },
            )
            .unwrap();
        assert_eq!(up.label, "Düzeltilmiş");
        assert_eq!(up.resolved.utc, "2000-01-01T12:01:00Z");
        assert!(up.updated_at > ch.updated_at);
        assert_eq!(
            s.calc_settings(ch.id).unwrap(),
            SettingsLoad::Current(settings)
        );
        assert!(matches!(
            s.update_chart(999, &new_client("A").chart),
            Err(StoreError::NotFound {
                what: "chart",
                id: 999
            })
        ));
    }

    #[test]
    fn save_calc_settings_validates() {
        let mut s = store();
        let (_, ch) = s.create_client(&new_client("A")).unwrap();
        let bad = CalcSettings {
            dasha_year: f64::NAN,
            ..CalcSettings::default()
        };
        assert!(matches!(
            s.save_calc_settings(ch.id, &bad),
            Err(StoreError::Settings(_))
        ));
    }

    #[test]
    fn view_state_splits_between_user_and_chart() {
        let mut s = store();
        let (c, a) = s.create_client(&new_client("A")).unwrap();
        let b = s.add_chart(c.id, &new_client("A").chart).unwrap();
        assert_eq!(s.view_state(a.id).unwrap(), ViewState::default());
        let layout = PanelLayout {
            active_layout_id: "builtin:research".into(),
            layouts: vec![Layout {
                id: "builtin:research".into(),
                name: "Araştırma".into(),
                builtin: true,
                panels: vec![Panel {
                    id: "d1".into(),
                    kind: "chart".into(),
                    x: 0,
                    y: 0,
                    w: 4,
                    h: 10,
                    settings: Map::new(),
                }],
            }],
        };
        let vs = ViewState {
            panel_layout: Some(layout.clone()),
            dasha_expanded_path: vec![Graha::Rahu, Graha::Saturn],
            chara_expanded_path: SignDashaPath {
                cycle: 2,
                signs: vec![Sign::Gemini, Sign::Taurus],
            },
            ..ViewState::default()
        };
        s.save_view_state(a.id, &vs).unwrap();
        assert_eq!(s.view_state(a.id).unwrap(), vs);
        // Düzen kullanıcınındır: diğer harita aynı düzeni, kendi (boş) dasha yollarını görür.
        let other = s.view_state(b.id).unwrap();
        assert_eq!(other.panel_layout, Some(layout));
        assert!(other.dasha_expanded_path.is_empty());
        assert_eq!(other.chara_expanded_path, empty_chara_path());
        // Görünüm kaydı içerik değildir.
        assert_eq!(s.chart(a.id).unwrap().updated_at, a.updated_at);
        s.save_view_state(a.id, &ViewState::default()).unwrap();
        assert_eq!(s.view_state(b.id).unwrap(), ViewState::default());
    }

    #[test]
    fn corrupt_stored_view_state_reads_as_default() {
        let mut s = store();
        let (_, ch) = s.create_client(&new_client("A")).unwrap();
        s.conn
            .execute(
                "UPDATE charts SET view_state_json = '{not json' WHERE id = ?1",
                params![ch.id],
            )
            .unwrap();
        s.app_state_set(PANEL_LAYOUT_KEY, "[[[").unwrap();
        assert_eq!(s.view_state(ch.id).unwrap(), ViewState::default());
    }

    #[test]
    fn delete_rules() {
        let mut s = store();
        let (c, a) = s.create_client(&new_client("A")).unwrap();
        assert!(matches!(s.delete_chart(a.id), Err(StoreError::LastChart)));
        let b = s.add_chart(c.id, &new_client("A").chart).unwrap();
        s.delete_chart(a.id).unwrap();
        assert!(matches!(
            s.chart(a.id),
            Err(StoreError::NotFound { what: "chart", .. })
        ));
        s.delete_client(c.id).unwrap();
        assert!(matches!(
            s.chart(b.id),
            Err(StoreError::NotFound { what: "chart", .. })
        ));
        assert!(matches!(
            s.delete_client(c.id),
            Err(StoreError::NotFound { what: "client", .. })
        ));
    }

    #[test]
    fn add_chart_writes_the_chart_and_the_client_together() {
        let mut s = store();
        let (c, _) = s.create_client(&new_client("A")).unwrap();
        s.conn
            .execute_batch(
                "CREATE TEMP TRIGGER fail_touch BEFORE UPDATE OF updated_at ON clients
                 BEGIN SELECT RAISE(ABORT, 'fail'); END;",
            )
            .unwrap();
        assert!(matches!(
            s.add_chart(c.id, &new_client("A").chart),
            Err(StoreError::Sqlite(_))
        ));
        assert_eq!(s.charts_of(c.id).unwrap().len(), 1);
    }

    #[test]
    fn list_and_update_clients() {
        let mut s = store();
        let (b, _) = s.create_client(&new_client("Burak")).unwrap();
        let (a, _) = s.create_client(&new_client("ahmet")).unwrap();
        let names: Vec<String> = s
            .list_clients(10)
            .unwrap()
            .into_iter()
            .map(|c| c.name)
            .collect();
        assert_eq!(names, ["ahmet", "Burak"]);
        s.conn
            .execute(
                "UPDATE clients SET last_opened_at = '2026-01-01T00:00:00.000Z' WHERE id = ?1",
                params![b.id],
            )
            .unwrap();
        let first = &s.list_clients(1).unwrap()[0];
        assert_eq!(first.id, b.id);
        let up = s.update_client(a.id, "Ahmet", Some(Gender::Male)).unwrap();
        assert_eq!((up.name.as_str(), up.gender), ("Ahmet", Some(Gender::Male)));
        assert!(up.updated_at > a.updated_at);
    }

    fn layouts(n: usize, panels: usize) -> PanelLayout {
        PanelLayout {
            active_layout_id: "user:0".into(),
            layouts: (0..n)
                .map(|i| Layout {
                    id: format!("user:{i}"),
                    name: format!("L{i}"),
                    builtin: false,
                    panels: (0..panels)
                        .map(|j| Panel {
                            id: format!("p{j}"),
                            kind: "notes".into(),
                            x: 0,
                            y: j as u32 * 4,
                            w: 3,
                            h: 4,
                            settings: serde_json::Map::new(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    #[test]
    fn save_view_state_rejects_what_the_parsers_would_trim() {
        let mut s = store();
        let (_, ch) = s.create_client(&new_client("A")).unwrap();
        let ok = ViewState {
            panel_layout: Some(layouts(MAX_LAYOUTS, MAX_PANELS)),
            ..ViewState::default()
        };
        s.save_view_state(ch.id, &ok).unwrap();
        for bad in [
            ViewState {
                panel_layout: Some(layouts(MAX_LAYOUTS + 1, 1)),
                ..ViewState::default()
            },
            ViewState {
                panel_layout: Some(layouts(1, MAX_PANELS + 1)),
                ..ViewState::default()
            },
            ViewState {
                schema_version: 2,
                ..ViewState::default()
            },
            ViewState {
                dasha_expanded_path: vec![jyotish_core::Graha::Sun; 6],
                ..ViewState::default()
            },
            ViewState {
                chara_expanded_path: SignDashaPath {
                    cycle: 3,
                    signs: Vec::new(),
                },
                ..ViewState::default()
            },
            ViewState {
                chara_expanded_path: SignDashaPath {
                    cycle: 1,
                    signs: vec![Sign::Aries; 4],
                },
                ..ViewState::default()
            },
        ] {
            assert!(
                matches!(
                    s.save_view_state(ch.id, &bad),
                    Err(StoreError::InvalidInput("view_state" | "panel_layout"))
                ),
                "{bad:?}"
            );
        }
        // Reddedilen kayıt öncekini değiştirmez.
        assert_eq!(s.view_state(ch.id).unwrap(), ok);
    }

    #[test]
    fn save_panel_layout_rejects_over_cap_and_oversized_layouts() {
        let mut s = store();
        assert!(matches!(
            s.save_panel_layout(Some(&layouts(MAX_LAYOUTS + 1, 1))),
            Err(StoreError::InvalidInput("panel_layout"))
        ));
        let mut huge = layouts(1, 1);
        huge.layouts[0].panels[0].settings.insert(
            "blob".into(),
            serde_json::Value::String("x".repeat(MAX_VIEW_STATE_BYTES)),
        );
        assert!(matches!(
            s.save_panel_layout(Some(&huge)),
            Err(StoreError::InvalidInput("panel_layout"))
        ));
        assert_eq!(s.panel_layout().unwrap(), None);
    }

    #[test]
    fn keep_stored_moment_stamps_the_current_tzdb_and_keeps_the_moment() {
        let mut s = store();
        let (a, ch) = s.create_client(&new_client("A")).unwrap();
        s.conn
            .execute(
                "UPDATE charts SET tzdb_version = '1999a', resolved_utc = '2000-01-01T11:00:00Z',
                 resolved_jd_ut = 2451544.9583333335 WHERE id = ?1",
                params![ch.id],
            )
            .unwrap();
        assert_eq!(s.open_client(a.id, None).unwrap().warnings.len(), 1);
        let kept = s.keep_stored_moment(ch.id).unwrap();
        assert_eq!(kept.resolved.tzdb_version, atlas::tzdb_version());
        assert_eq!(kept.resolved.utc, "2000-01-01T11:00:00Z");
        assert_eq!(kept.resolved.jd_ut, 2_451_544.958_333_333_5);
        let o = s.open_client(a.id, None).unwrap();
        assert!(o.warnings.is_empty());
        assert_eq!(o.chart.resolved.utc, "2000-01-01T11:00:00Z");
        assert!(matches!(
            s.keep_stored_moment(999),
            Err(StoreError::NotFound {
                what: "chart",
                id: 999
            })
        ));
    }

    #[test]
    fn an_update_into_a_dst_gap_changes_nothing() {
        let mut s = store();
        let (_, ch) = s.create_client(&new_client("Ayşe")).unwrap();
        let before = s.chart(ch.id).unwrap();
        // İstanbul 2014-03-31 03:00–04:00 yaşanmadı (yaz saati geçişi).
        let e = s
            .update_chart(
                ch.id,
                &NewChart {
                    label: "x".into(),
                    kind: ChartKind::Event,
                    birth: istanbul("2014-03-31", "03:30"),
                },
            )
            .unwrap_err();
        assert!(matches!(e, StoreError::Moment(_)), "{e:?}");
        assert_eq!(s.chart(ch.id).unwrap(), before);
    }
}
