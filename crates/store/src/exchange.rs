//! Sürümlü JSON içe/dışa aktarma (spec §5 "İçe ve dışa aktarma"). Kayıtlar UUID ile
//! birleştirilir: var olan UUID atlanır, üzerine yazılmaz. Notlar yalnızca istenirse
//! dışa aktarılır (varsayılan kapalı; notlar hassas veridir).

use jyotish_core::JD_UT_RANGE;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::birth::{parse_birth, resolve_parsed, ParsedBirth};
use crate::charts::{clean_name, insert_chart, insert_client, ChartRow};
use crate::db::format_ts;
use crate::model::{BirthData, ChartKind, Gender, ResolvedRecord, TzChoice};
use crate::settings::{load_calc_value, SettingsLoad};
use crate::view_state::{chart_view_json, parse_dasha_path};
use crate::{Store, StoreError};

pub const EXPORT_FORMAT: &str = "jyotish-desk";
pub const EXPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ExportOptions {
    /// "Notları dahil et"; varsayılan kapalı.
    pub include_notes: bool,
    /// Boşsa tüm danışanlar.
    pub client_ids: Option<Vec<u32>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportFile {
    pub format: String,
    pub schema_version: u32,
    pub exported_at: String,
    pub clients: Vec<ExportClient>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportClient {
    pub uuid: String,
    pub name: String,
    pub gender: Option<Gender>,
    pub created_at: String,
    pub updated_at: String,
    pub charts: Vec<ExportChart>,
    /// Notlar dahil edilmediyse `null`.
    pub notes: Option<Vec<ExportNote>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportChart {
    pub uuid: String,
    pub label: String,
    pub kind: ChartKind,
    pub birth: BirthData,
    pub resolved: ResolvedRecord,
    /// Saklanan JSON aynen (daha yeni sürümün alanları korunur).
    pub calc_settings: Value,
    /// Haritanın görünüm kısmı (`dasha_expanded_path`); düzenler kullanıcınındır, aktarılmaz.
    pub view_state: Value,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportNote {
    pub uuid: String,
    pub chart_uuid: Option<String>,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum ImportWarningCode {
    /// Haritanın ayarı okunamadı; o anki genel varsayılan yazıldı.
    CalcSettingsReset,
    /// JHD: saat dilimi ofseti ile boylam 60°'den fazla uyuşmuyor; işaret kuralı şüpheli.
    JhdOffsetLongitudeMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ImportWarning {
    /// Harita UUID'si ya da JHD dosya adı.
    pub item: String,
    pub code: ImportWarningCode,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ImportReport {
    pub clients_added: u32,
    pub charts_added: u32,
    pub notes_added: u32,
    /// Zaten var olan (UUID ya da JHD eşleşmesi) ve atlanan kayıtlar.
    pub skipped: u32,
    pub warnings: Vec<ImportWarning>,
}

fn id_by_uuid(conn: &Connection, table: &str, uuid: &str) -> Result<Option<u32>, StoreError> {
    let sql = format!("SELECT id FROM {table} WHERE uuid = ?1");
    Ok(conn
        .query_row(&sql, params![uuid], |r| r.get(0))
        .optional()?)
}

/// UUID'yi denetler ve kanonik biçime (küçük harf, tireli) çevirir; birleştirme bu biçimle
/// yapılır.
fn check_uuid(uuid: &str) -> Result<String, StoreError> {
    Uuid::parse_str(uuid)
        .map(|u| u.to_string())
        .map_err(|_| StoreError::ImportRecord {
            item: uuid.to_owned(),
            field: "uuid",
        })
}

/// İçe alınan çözülmüş anı denetler. Güncel tzdb ile yeniden çözülen an dosyadakiyle aynıysa
/// yeni çözüm yazılır. Farklıysa yalnızca IANA dilimli ve başka bir tzdb sürümüyle
/// kaydedilmiş harita dosyadaki anı korur (karar açılışta kullanıcınındır); o durumda `utc`
/// okunabilir, `jd_ut` aynı anı göstermeli (≤ 1 sn) ve hesap aralığında olmalıdır. Diğer her
/// uyuşmazlık `None` verir: doğum verisiyle uyuşmayan bir anla harita sessizce açılmaz.
fn import_resolved(p: &ParsedBirth, r: &ResolvedRecord) -> Option<ResolvedRecord> {
    let stored: atlas::Timestamp = r.utc.parse().ok()?;
    if let Ok(fresh) = resolve_parsed(p) {
        if fresh.utc.parse::<atlas::Timestamp>().ok() == Some(stored) {
            return Some(fresh);
        }
    }
    let other_tzdb =
        matches!(p.canonical.tz, TzChoice::Iana(_)) && r.tzdb_version != atlas::tzdb_version();
    let jd = stored.as_millisecond() as f64 / 86_400_000.0 + 2_440_587.5;
    if !other_tzdb || !JD_UT_RANGE.contains(&r.jd_ut) || (r.jd_ut - jd).abs() * 86_400.0 > 1.0 {
        return None;
    }
    Some(ResolvedRecord {
        utc: stored.to_string(),
        ..r.clone()
    })
}

/// Geçerli bir `format_ts` zaman damgası değilse `now`.
fn ts_or(ts: &str, now: &str) -> String {
    match ts.parse::<atlas::Timestamp>() {
        Ok(t) => format_ts(t),
        Err(_) => now.to_owned(),
    }
}

impl Store {
    /// Okunabilir (girintili) JSON; danışanlar ve haritalar kimlik sırasında.
    pub fn export_json(&mut self, opts: &ExportOptions) -> Result<String, StoreError> {
        let exported_at = self.now();
        let ids: Vec<u32> = match &opts.client_ids {
            Some(ids) => {
                for id in ids {
                    self.client(*id)?;
                }
                ids.clone()
            }
            None => {
                let mut stmt = self.conn.prepare("SELECT id FROM clients ORDER BY id")?;
                let rows = stmt.query_map([], |r| r.get(0))?;
                rows.collect::<Result<Vec<u32>, _>>()?
            }
        };
        let mut clients = Vec::with_capacity(ids.len());
        for id in ids {
            let c = self.client(id)?;
            let mut charts = self.charts_of(id)?;
            charts.sort_by_key(|ch| ch.id);
            let mut out_charts = Vec::with_capacity(charts.len());
            for ch in &charts {
                let (calc, view): (String, String) = self.conn.query_row(
                    "SELECT calc_settings_json, view_state_json FROM charts WHERE id = ?1",
                    params![ch.id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )?;
                out_charts.push(ExportChart {
                    uuid: ch.uuid.clone(),
                    label: ch.label.clone(),
                    kind: ch.kind,
                    birth: ch.birth.clone(),
                    resolved: ch.resolved.clone(),
                    calc_settings: serde_json::from_str(&calc).unwrap_or(Value::Null),
                    view_state: serde_json::from_str(&view).unwrap_or(Value::Null),
                    tags: ch.tags.clone(),
                    created_at: ch.created_at.clone(),
                    updated_at: ch.updated_at.clone(),
                });
            }
            let notes = if opts.include_notes {
                let notes = self.all_notes(id)?;
                Some(
                    notes
                        .into_iter()
                        .map(|n| ExportNote {
                            uuid: n.uuid,
                            chart_uuid: n.chart_id.and_then(|cid| {
                                charts
                                    .iter()
                                    .find(|ch| ch.id == cid)
                                    .map(|ch| ch.uuid.clone())
                            }),
                            body: n.body,
                            created_at: n.created_at,
                            updated_at: n.updated_at,
                        })
                        .collect(),
                )
            } else {
                None
            };
            clients.push(ExportClient {
                uuid: c.uuid,
                name: c.name,
                gender: c.gender,
                created_at: c.created_at,
                updated_at: c.updated_at,
                charts: out_charts,
                notes,
            });
        }
        Ok(serde_json::to_string_pretty(&ExportFile {
            format: EXPORT_FORMAT.into(),
            schema_version: EXPORT_SCHEMA_VERSION,
            exported_at,
            clients,
        })?)
    }

    /// Hepsi ya da hiçbiri: geçersiz bir kayıt tüm içe aktarmayı geri alır ve kaydın
    /// UUID'siyle `ImportRecord` döner. Çözülmüş an doğum verisine karşı denetlenir
    /// (`import_resolved`); okunamayan ayar o anki genel varsayılanla değiştirilir. Haritasız
    /// danışan oluşmaz.
    pub fn import_json(&mut self, text: &str) -> Result<ImportReport, StoreError> {
        let value: Value =
            serde_json::from_str(text).map_err(|e| StoreError::ImportFormat(e.to_string()))?;
        if value.get("format").and_then(Value::as_str) != Some(EXPORT_FORMAT) {
            return Err(StoreError::ImportFormat("format".into()));
        }
        let version = value
            .get("schema_version")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .ok_or_else(|| StoreError::ImportFormat("schema_version".into()))?;
        if version == 0 || version > EXPORT_SCHEMA_VERSION {
            return Err(StoreError::ImportVersion {
                found: version,
                supported: EXPORT_SCHEMA_VERSION,
            });
        }
        let file: ExportFile =
            serde_json::from_value(value).map_err(|e| StoreError::ImportFormat(e.to_string()))?;
        let defaults = self.app_prefs()?.calc_settings;
        let now = self.now();
        let mut report = ImportReport::default();
        let tx = self.conn.transaction()?;
        for c in &file.clients {
            let client_uuid = check_uuid(&c.uuid)?;
            let client_id = match id_by_uuid(&tx, "clients", &client_uuid)? {
                Some(id) => {
                    report.skipped += 1;
                    id
                }
                None => {
                    let name =
                        clean_name(&c.name, "name").map_err(|_| StoreError::ImportRecord {
                            item: client_uuid.clone(),
                            field: "name",
                        })?;
                    report.clients_added += 1;
                    insert_client(
                        &tx,
                        &client_uuid,
                        &name,
                        c.gender,
                        &ts_or(&c.created_at, &now),
                        &ts_or(&c.updated_at, &now),
                    )?
                }
            };
            let mut first_new_chart = None;
            for ch in &c.charts {
                let chart_uuid = check_uuid(&ch.uuid)?;
                if id_by_uuid(&tx, "charts", &chart_uuid)?.is_some() {
                    report.skipped += 1;
                    continue;
                }
                let record = |field| StoreError::ImportRecord {
                    item: chart_uuid.clone(),
                    field,
                };
                let parsed = parse_birth(&ch.birth).map_err(|e| match e {
                    StoreError::InvalidInput(f) => record(f),
                    _ => record("birth"),
                })?;
                let label = clean_name(&ch.label, "label").map_err(|_| record("label"))?;
                let resolved =
                    import_resolved(&parsed, &ch.resolved).ok_or_else(|| record("resolved"))?;
                let settings = match load_calc_value(ch.calc_settings.clone(), &defaults) {
                    SettingsLoad::Reset { settings, .. } => {
                        report.warnings.push(ImportWarning {
                            item: chart_uuid.clone(),
                            code: ImportWarningCode::CalcSettingsReset,
                        });
                        settings
                    }
                    other => other.settings().clone(),
                };
                let path = parse_dasha_path(ch.view_state.get("dasha_expanded_path"));
                let chart_id = insert_chart(
                    &tx,
                    &ChartRow {
                        uuid: &chart_uuid,
                        client_id,
                        label: &label,
                        kind: ch.kind,
                        birth: &parsed.canonical,
                        resolved: &resolved,
                        calc_json: &serde_json::to_string(&settings)?,
                        view_json: &chart_view_json(&path),
                        created_at: &ts_or(&ch.created_at, &now),
                        updated_at: &ts_or(&ch.updated_at, &now),
                    },
                )?;
                for t in crate::notes::normalize_tags(&ch.tags).map_err(|_| record("tags"))? {
                    crate::notes::attach_tag(&tx, chart_id, &t)?;
                }
                first_new_chart.get_or_insert(chart_id);
                report.charts_added += 1;
            }
            if let Some(chart_id) = first_new_chart {
                tx.execute(
                    "UPDATE clients SET last_chart_id = ?1 WHERE id = ?2 AND last_chart_id IS NULL",
                    params![chart_id, client_id],
                )?;
            }
            let has_chart: bool = tx.query_row(
                "SELECT EXISTS (SELECT 1 FROM charts WHERE client_id = ?1)",
                params![client_id],
                |r| r.get(0),
            )?;
            if !has_chart {
                return Err(StoreError::ImportRecord {
                    item: client_uuid,
                    field: "charts",
                });
            }
            for n in c.notes.iter().flatten() {
                let note_uuid = check_uuid(&n.uuid)?;
                if id_by_uuid(&tx, "client_notes", &note_uuid)?.is_some() {
                    report.skipped += 1;
                    continue;
                }
                let chart_id: Option<u32> = match n.chart_uuid.as_deref().map(Uuid::parse_str) {
                    Some(Ok(u)) => tx
                        .query_row(
                            "SELECT id FROM charts WHERE uuid = ?1 AND client_id = ?2",
                            params![u.to_string(), client_id],
                            |r| r.get(0),
                        )
                        .optional()?,
                    _ => None,
                };
                tx.execute(
                    "INSERT INTO client_notes (uuid, client_id, chart_id, body, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        note_uuid,
                        client_id,
                        chart_id,
                        n.body,
                        ts_or(&n.created_at, &now),
                        ts_or(&n.updated_at, &now)
                    ],
                )?;
                report.notes_added += 1;
            }
        }
        tx.commit()?;
        Ok(report)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::testutil::new_client;
    use crate::db::testutil::store;
    use crate::open::StoreWarning;
    use crate::view_state::ViewState;
    use jyotish_core::{CalcSettings, Graha, NodeKind};
    use serde_json::json;

    fn populated() -> (Store, u32) {
        let mut s = store();
        let (c, ch) = s.create_client(&new_client("Ayşe")).unwrap();
        let settings = CalcSettings {
            node: NodeKind::True,
            ..CalcSettings::default()
        };
        s.save_calc_settings(ch.id, &settings).unwrap();
        s.set_chart_tags(ch.id, &["araştırma".into()]).unwrap();
        s.save_view_state(
            ch.id,
            &ViewState {
                dasha_expanded_path: vec![Graha::Venus],
                ..ViewState::default()
            },
        )
        .unwrap();
        s.add_note(c.id, Some(ch.id), "gizli seans notu").unwrap();
        (s, c.id)
    }

    #[test]
    fn notes_are_excluded_by_default() {
        let (mut s, _) = populated();
        let text = s.export_json(&ExportOptions::default()).unwrap();
        assert!(!text.contains("gizli seans notu"));
        let f: ExportFile = serde_json::from_str(&text).unwrap();
        assert_eq!((f.format.as_str(), f.schema_version), (EXPORT_FORMAT, 1));
        assert_eq!(f.clients[0].notes, None);
        assert_eq!(f.clients[0].charts[0].tags, ["araştırma"]);
        assert_eq!(f.clients[0].charts[0].calc_settings["node"], json!("True"));
        let with = s
            .export_json(&ExportOptions {
                include_notes: true,
                client_ids: None,
            })
            .unwrap();
        assert!(with.contains("gizli seans notu"));
    }

    #[test]
    fn round_trip_into_empty_store_is_exact_and_idempotent() {
        let (mut src, cid) = populated();
        let opts = ExportOptions {
            include_notes: true,
            client_ids: Some(vec![cid]),
        };
        let text = src.export_json(&opts).unwrap();
        let mut dst = store();
        let r = dst.import_json(&text).unwrap();
        assert_eq!(
            r,
            ImportReport {
                clients_added: 1,
                charts_added: 1,
                notes_added: 1,
                skipped: 0,
                warnings: vec![]
            }
        );
        let a = src.client(cid).unwrap();
        let b = &dst.list_clients(10).unwrap()[0];
        assert_eq!(
            (&a.uuid, &a.name, &a.created_at),
            (&b.uuid, &b.name, &b.created_at)
        );
        let ca = &src.charts_of(a.id).unwrap()[0];
        let cb = &dst.charts_of(b.id).unwrap()[0];
        assert_eq!(
            (&ca.uuid, &ca.birth, &ca.resolved, &ca.tags),
            (&cb.uuid, &cb.birth, &cb.resolved, &cb.tags)
        );
        assert_eq!(
            src.calc_settings(ca.id).unwrap(),
            dst.calc_settings(cb.id).unwrap()
        );
        assert_eq!(
            dst.view_state(cb.id).unwrap().dasha_expanded_path,
            [Graha::Venus]
        );
        assert_eq!(b.last_chart_id, Some(cb.id));
        let na = &src.notes(a.id, 10).unwrap()[0];
        let nb = &dst.notes(b.id, 10).unwrap()[0];
        assert_eq!(
            (&na.uuid, &na.body, nb.chart_id),
            (&nb.uuid, &nb.body, Some(cb.id))
        );
        // İkinci kez: hepsi atlanır, kopya oluşmaz.
        let again = dst.import_json(&text).unwrap();
        assert_eq!(
            (again.clients_added, again.charts_added, again.skipped),
            (0, 0, 3)
        );
        assert_eq!(dst.list_clients(10).unwrap().len(), 1);
        // Dışa aktarım yeniden üretilebilir (exported_at hariç).
        let mut x: Value = serde_json::from_str(&src.export_json(&opts).unwrap()).unwrap();
        let mut y: Value = serde_json::from_str(&dst.export_json(&opts).unwrap()).unwrap();
        x["exported_at"] = Value::Null;
        y["exported_at"] = Value::Null;
        assert_eq!(x, y);
    }

    #[test]
    fn invalid_files_and_records_are_rejected_atomically() {
        let mut s = store();
        assert!(matches!(
            s.import_json("nope"),
            Err(StoreError::ImportFormat(_))
        ));
        assert!(matches!(
            s.import_json(r#"{"format":"other","schema_version":1}"#),
            Err(StoreError::ImportFormat(_))
        ));
        assert!(matches!(
            s.import_json(r#"{"format":"jyotish-desk","schema_version":2,"clients":[]}"#),
            Err(StoreError::ImportVersion {
                found: 2,
                supported: 1
            })
        ));
        let (mut src, _) = populated();
        let mut v: Value =
            serde_json::from_str(&src.export_json(&ExportOptions::default()).unwrap()).unwrap();
        let second = v["clients"][0].clone();
        let mut bad = second.clone();
        bad["uuid"] = json!(Uuid::new_v4().to_string());
        bad["charts"][0]["uuid"] = json!("4b0f2a8e-0000-4000-8000-000000000001");
        bad["charts"][0]["birth"]["date"] = json!("2000-13-01");
        v["clients"] = json!([second, bad]);
        match s.import_json(&v.to_string()) {
            Err(StoreError::ImportRecord { item, field }) => {
                assert_eq!(item, "4b0f2a8e-0000-4000-8000-000000000001");
                assert_eq!(field, "birth.date");
            }
            other => panic!("{other:?}"),
        }
        // Geri alındı: ilk (geçerli) danışan da yazılmadı.
        assert!(s.list_clients(10).unwrap().is_empty());
    }

    #[test]
    fn unreadable_imported_settings_take_current_defaults_with_warning() {
        let (mut src, _) = populated();
        let mut v: Value =
            serde_json::from_str(&src.export_json(&ExportOptions::default()).unwrap()).unwrap();
        v["clients"][0]["charts"][0]["calc_settings"] = json!({"schema_version": 5});
        v["clients"][0]["charts"][0]["view_state"] = json!("garbage");
        let mut dst = store();
        let r = dst.import_json(&v.to_string()).unwrap();
        let uuid = v["clients"][0]["charts"][0]["uuid"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_eq!(
            r.warnings,
            [ImportWarning {
                item: uuid,
                code: ImportWarningCode::CalcSettingsReset
            }]
        );
        let c = &dst.list_clients(1).unwrap()[0];
        let ch = &dst.charts_of(c.id).unwrap()[0];
        assert_eq!(
            dst.calc_settings(ch.id).unwrap(),
            SettingsLoad::Current(CalcSettings::default())
        );
        assert!(dst
            .view_state(ch.id)
            .unwrap()
            .dasha_expanded_path
            .is_empty());
    }

    #[test]
    fn imported_moments_uuids_and_empty_clients_are_checked() {
        let (mut src, _) = populated();
        let exported: Value =
            serde_json::from_str(&src.export_json(&ExportOptions::default()).unwrap()).unwrap();
        let rejected = |v: &Value| match store().import_json(&v.to_string()) {
            Err(StoreError::ImportRecord { field, .. }) => field,
            other => panic!("{other:?}"),
        };
        // Doğum verisiyle uyuşmayan an (aynı tzdb sürümü): sessizce yanlış harita açılmaz.
        let mut v = exported.clone();
        v["clients"][0]["charts"][0]["resolved"]["jd_ut"] = json!(2_451_000.25);
        v["clients"][0]["charts"][0]["resolved"]["utc"] = json!("1998-07-06T18:00:00Z");
        assert_eq!(rejected(&v), "resolved");
        let mut v = exported.clone();
        v["clients"][0]["charts"][0]["resolved"]["utc"] = json!("=1+1");
        assert_eq!(rejected(&v), "resolved");
        // Başka tzdb sürümüyle kaydedilmiş IANA haritası: an korunur (karar açılışta), ama
        // `jd_ut` ile `utc` aynı anı göstermelidir.
        let mut v = exported.clone();
        v["clients"][0]["charts"][0]["resolved"]["tzdb_version"] = json!("1999a");
        v["clients"][0]["charts"][0]["resolved"]["utc"] = json!("2000-01-01T11:00:00Z");
        assert_eq!(rejected(&v), "resolved");
        v["clients"][0]["charts"][0]["resolved"]["jd_ut"] = json!(2_451_544.958_333_333_5);
        let mut dst = store();
        dst.import_json(&v.to_string()).unwrap();
        let c = &dst.list_clients(1).unwrap()[0];
        let o = dst.open_client(c.id, None).unwrap();
        assert_eq!(o.chart.resolved.utc, "2000-01-01T11:00:00Z");
        assert!(matches!(o.warnings[..], [StoreWarning::TzdbChanged { .. }]));
        // Haritasız danışan oluşmaz.
        let mut v = exported.clone();
        v["clients"][0]["charts"] = json!([]);
        assert_eq!(rejected(&v), "charts");
        // UUID yazımı (büyük harf) aynı kaydı gösterir: kopya oluşmaz.
        let mut v = exported.clone();
        for p in ["/clients/0/uuid", "/clients/0/charts/0/uuid"] {
            let up = v.pointer(p).unwrap().as_str().unwrap().to_uppercase();
            *v.pointer_mut(p).unwrap() = json!(up);
        }
        let mut dst = store();
        dst.import_json(&exported.to_string()).unwrap();
        let again = dst.import_json(&v.to_string()).unwrap();
        assert_eq!(
            (again.clients_added, again.charts_added, again.skipped),
            (0, 0, 2)
        );
    }
}
