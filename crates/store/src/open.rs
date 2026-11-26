//! `open_client`'ın store yarısı (spec §4 "Danışan açma"): harita seçimi, ayar göçü,
//! görünüm durumu, tzdb denetimi, son notlar ve "son açılan" işaretleri. Harita hesabı
//! uygulama katmanındadır.

use jyotish_core::CalcSettings;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::birth::{parse_birth, resolve_parsed};
use crate::model::{ChartRecord, Client, Note, TzChoice};
use crate::notes::RECENT_NOTES;
use crate::settings::SettingsLoad;
use crate::view_state::ViewState;
use crate::{Store, StoreError};

/// Açılışın ölümcül olmayan durumları; arayüz uyarı bandında gösterir (spec §7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum StoreWarning {
    /// Saklanan ayar okunamadı ya da göç edilemedi; genel varsayılan kullanıldı. Saklanan
    /// JSON değiştirilmedi.
    CalcSettingsReset { found_version: Option<u32> },
    /// `last_chart_id` silinmiş bir haritayı gösteriyordu; en son güncellenen açıldı.
    LastChartMissing,
    /// Gömülü tzdb değişti ve bu haritanın UTC anını değiştiriyor. Saklanan an kullanıldı;
    /// karar kullanıcınındır (`update_chart` ile kabul edilir). `current_utc` yeni tzdb'nin
    /// sonucudur; yeni tzdb ile çözülemiyorsa boştur.
    TzdbChanged {
        stored_version: String,
        current_version: String,
        current_utc: Option<String>,
    },
}

/// Açılan danışanın kayıt tarafı.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenRecord {
    pub client: Client,
    pub chart: ChartRecord,
    pub calc_settings: CalcSettings,
    pub view_state: ViewState,
    /// En yeni önce, en çok `RECENT_NOTES`.
    pub recent_notes: Vec<Note>,
    pub warnings: Vec<StoreWarning>,
}

impl Store {
    /// Danışanı açar. `chart_id` verilmezse `last_chart_id`, o yoksa ya da silinmişse en son
    /// güncellenen harita. Bozuk görünüm durumu ve ayar açılışı engellemez.
    pub fn open_client(
        &mut self,
        client_id: u32,
        chart_id: Option<u32>,
    ) -> Result<OpenRecord, StoreError> {
        let mut client = self.client(client_id)?;
        let mut warnings = Vec::new();
        let chart_id = match chart_id {
            Some(id) => {
                let owner: Option<u32> = self
                    .conn
                    .query_row(
                        "SELECT client_id FROM charts WHERE id = ?1",
                        params![id],
                        |r| r.get(0),
                    )
                    .optional()?;
                if owner != Some(client_id) {
                    return Err(StoreError::NotFound { what: "chart", id });
                }
                id
            }
            None => self.pick_chart(&client, &mut warnings)?,
        };
        let mut chart = self.chart(chart_id)?;

        let load = self.calc_settings(chart_id)?;
        let view_state = self.view_state(chart_id)?;
        let recent_notes = self.notes(client_id, RECENT_NOTES)?;

        // Ayar göçü, tzdb sürümü ve "son açılan" işaretleri tek işlemde yazılır: biri
        // başarısız olursa hiçbiri kalmaz (Plan 08 ertelenen madde 9).
        let now = self.now();
        let tx = self.conn.transaction()?;
        let calc_settings = settle_settings(&tx, chart_id, load, &mut warnings)?;
        check_tzdb(&tx, &mut chart, &mut warnings)?;
        tx.execute(
            "UPDATE clients SET last_chart_id = ?1, last_opened_at = ?2 WHERE id = ?3",
            params![chart_id, now, client_id],
        )?;
        tx.execute(
            "UPDATE charts SET last_opened_at = ?1 WHERE id = ?2",
            params![now, chart_id],
        )?;
        tx.commit()?;
        client.last_chart_id = Some(chart_id);
        client.last_opened_at = Some(now.clone());
        chart.last_opened_at = Some(now);
        Ok(OpenRecord {
            client,
            chart,
            calc_settings,
            view_state,
            recent_notes,
            warnings,
        })
    }

    fn pick_chart(
        &self,
        client: &Client,
        warnings: &mut Vec<StoreWarning>,
    ) -> Result<u32, StoreError> {
        if let Some(id) = client.last_chart_id {
            let exists: Option<u32> = self
                .conn
                .query_row(
                    "SELECT id FROM charts WHERE id = ?1 AND client_id = ?2",
                    params![id, client.id],
                    |r| r.get(0),
                )
                .optional()?;
            if exists.is_some() {
                return Ok(id);
            }
            warnings.push(StoreWarning::LastChartMissing);
        }
        self.conn
            .query_row(
                "SELECT id FROM charts WHERE client_id = ?1 ORDER BY updated_at DESC, id DESC",
                params![client.id],
                |r| r.get(0),
            )
            .optional()?
            .ok_or(StoreError::NotFound {
                what: "client_chart",
                id: client.id,
            })
    }

    /// Oturum geri yükleme: en son açılan danışan ve haritası.
    pub fn last_opened(&self) -> Result<Option<(u32, u32)>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT client_id, id FROM charts WHERE last_opened_at IS NOT NULL
                 ORDER BY last_opened_at DESC, id DESC LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?)
    }
}

/// Ayar okumasının sonucunu uygular: göç edilmiş ayar yeni JSON olarak yazılır; sıfırlanan
/// ayar uyarı verir ve saklanan JSON'a dokunulmaz.
fn settle_settings(
    conn: &Connection,
    chart_id: u32,
    load: SettingsLoad,
    warnings: &mut Vec<StoreWarning>,
) -> Result<CalcSettings, StoreError> {
    Ok(match load {
        SettingsLoad::Current(s) => s,
        SettingsLoad::Migrated { settings, .. } => {
            conn.execute(
                "UPDATE charts SET calc_settings_json = ?1 WHERE id = ?2",
                params![serde_json::to_string(&settings)?, chart_id],
            )?;
            settings
        }
        SettingsLoad::Reset {
            settings,
            found_version,
        } => {
            warnings.push(StoreWarning::CalcSettingsReset { found_version });
            settings
        }
    })
}

/// Saklanan tzdb sürümü güncel değilse ve saat dilimi IANA ise an yeniden çözülür.
/// Sonuç aynıysa sürüm sessizce güncellenir; farklıysa ya da çözülemezse uyarı.
fn check_tzdb(
    conn: &Connection,
    chart: &mut ChartRecord,
    warnings: &mut Vec<StoreWarning>,
) -> Result<(), StoreError> {
    let current = atlas::tzdb_version();
    if chart.resolved.tzdb_version == current || !matches!(chart.birth.tz, TzChoice::Iana(_)) {
        return Ok(());
    }
    let fresh = parse_birth(&chart.birth).and_then(|p| resolve_parsed(&p));
    match fresh {
        Ok(r) if r.utc == chart.resolved.utc => {
            // An aynı; yeni tzdb'nin kısaltması, ofseti ve uyarıları yazılır (Plan 11 D6).
            conn.execute(
                "UPDATE charts SET tzdb_version = ?1, resolved_abbrev = ?2,
                 resolved_offset_secs = ?3, resolved_warnings_json = ?4 WHERE id = ?5",
                params![
                    current,
                    r.abbrev,
                    r.offset_secs,
                    serde_json::to_string(&r.warnings)?,
                    chart.id
                ],
            )?;
            chart.resolved = r;
        }
        other => warnings.push(StoreWarning::TzdbChanged {
            stored_version: chart.resolved.tzdb_version.clone(),
            current_version: current.to_owned(),
            current_utc: other.ok().map(|r| r.utc),
        }),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::testutil::new_client;
    use crate::db::testutil::{store, stuck_store};
    use crate::settings::AppPrefs;
    use jyotish_core::{AyanamsaSystem, Graha};
    use proptest::prelude::*;

    #[test]
    fn open_returns_everything_and_marks_opened() {
        let mut s = store();
        let (c, ch) = s.create_client(&new_client("A")).unwrap();
        let note = s.add_note(c.id, Some(ch.id), "not").unwrap();
        let vs = ViewState {
            dasha_expanded_path: vec![Graha::Moon],
            ..ViewState::default()
        };
        s.save_view_state(ch.id, &vs).unwrap();
        assert_eq!(s.last_opened().unwrap(), None);
        let o = s.open_client(c.id, None).unwrap();
        assert_eq!(o.chart.id, ch.id);
        assert_eq!(o.calc_settings, CalcSettings::default());
        assert_eq!(o.view_state, vs);
        assert_eq!(o.recent_notes, [note]);
        assert!(o.warnings.is_empty());
        let opened = o.client.last_opened_at.clone().unwrap();
        assert_eq!(o.chart.last_opened_at.as_deref(), Some(opened.as_str()));
        assert_eq!(s.client(c.id).unwrap(), o.client);
        assert_eq!(s.last_opened().unwrap(), Some((c.id, ch.id)));
    }

    #[test]
    fn explicit_chart_becomes_last_and_foreign_chart_is_refused() {
        let mut s = store();
        let (a, first) = s.create_client(&new_client("A")).unwrap();
        let second = s.add_chart(a.id, &new_client("A").chart).unwrap();
        let (_, foreign) = s.create_client(&new_client("B")).unwrap();
        assert_eq!(s.open_client(a.id, None).unwrap().chart.id, first.id);
        assert_eq!(
            s.open_client(a.id, Some(second.id)).unwrap().chart.id,
            second.id
        );
        assert_eq!(s.open_client(a.id, None).unwrap().chart.id, second.id);
        assert!(matches!(
            s.open_client(a.id, Some(foreign.id)),
            Err(StoreError::NotFound { what: "chart", .. })
        ));
        assert!(matches!(
            s.open_client(999, None),
            Err(StoreError::NotFound { what: "client", .. })
        ));
    }

    #[test]
    fn deleted_last_chart_falls_back_to_most_recently_updated() {
        let mut s = store();
        let (a, first) = s.create_client(&new_client("A")).unwrap();
        let second = s.add_chart(a.id, &new_client("A").chart).unwrap();
        let third = s.add_chart(a.id, &new_client("A").chart).unwrap();
        s.save_calc_settings(second.id, &CalcSettings::default())
            .unwrap();
        s.open_client(a.id, Some(third.id)).unwrap();
        s.delete_chart(third.id).unwrap();
        let o = s.open_client(a.id, None).unwrap();
        assert_eq!(o.chart.id, second.id);
        assert_eq!(o.warnings, [StoreWarning::LastChartMissing]);
        assert_ne!(o.chart.id, first.id);
    }

    #[test]
    fn unreadable_settings_use_current_default_and_keep_stored_json() {
        let mut s = store();
        let (a, ch) = s.create_client(&new_client("A")).unwrap();
        let raman = CalcSettings {
            ayanamsa: AyanamsaSystem::Raman,
            ..CalcSettings::default()
        };
        s.set_app_prefs(&AppPrefs {
            calc_settings: raman.clone(),
            ..AppPrefs::default()
        })
        .unwrap();
        let future = r#"{"schema_version":3,"zodiac":"Draconic"}"#;
        s.conn
            .execute(
                "UPDATE charts SET calc_settings_json = ?1 WHERE id = ?2",
                params![future, ch.id],
            )
            .unwrap();
        let o = s.open_client(a.id, None).unwrap();
        assert_eq!(o.calc_settings, raman);
        assert_eq!(
            o.warnings,
            [StoreWarning::CalcSettingsReset {
                found_version: Some(3)
            }]
        );
        let stored: String = s
            .conn
            .query_row(
                "SELECT calc_settings_json FROM charts WHERE id = ?1",
                params![ch.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stored, future);
    }

    #[test]
    fn version_1_settings_migrate_on_open_and_unknown_advanced_keys_do_not_block() {
        let mut s = store();
        let (a, ch) = s.create_client(&new_client("A")).unwrap();
        let stored_json = |s: &Store| -> String {
            s.conn
                .query_row(
                    "SELECT calc_settings_json FROM charts WHERE id = ?1",
                    params![ch.id],
                    |r| r.get(0),
                )
                .unwrap()
        };
        let write = |s: &Store, json: &str| {
            s.conn
                .execute(
                    "UPDATE charts SET calc_settings_json = ?1 WHERE id = ?2",
                    params![json, ch.id],
                )
                .unwrap();
        };
        // Sürüm 1 kaydı (Plan 03–13'ün yazdığı biçim): `advanced` yok.
        let v1 = concat!(
            r#"{"schema_version":1,"zodiac":"Sidereal","ayanamsa":"Raman","node":"Mean","#,
            r#""house_system":"WholeSign","dasha_year":365.25,"combustion_orbs":{"moon":12.0,"#,
            r#""mars":17.0,"mercury":14.0,"mercury_retro":12.0,"jupiter":11.0,"venus":10.0,"#,
            r#""venus_retro":8.0,"saturn":15.0},"sunrise_def":"DiskCenterNoRefraction"}"#
        );
        write(&s, v1);
        let raman = CalcSettings {
            ayanamsa: AyanamsaSystem::Raman,
            ..CalcSettings::default()
        };
        let o = s.open_client(a.id, None).unwrap();
        assert_eq!(o.calc_settings, raman);
        assert_eq!(o.calc_settings.schema_version, 2);
        assert!(o.warnings.is_empty());
        // Göç edilen ayar sürüm 2 olarak geri yazılır.
        assert_eq!(stored_json(&s), serde_json::to_string(&raman).unwrap());

        // Sürüm 2 kaydında bilinmeyen `advanced` anahtarı: açılış sürer, uyarı yok, kayıt
        // olduğu gibi durur (daha yeni bir sürümün alanı kaybolmaz).
        let with_extra = serde_json::to_string(&raman).unwrap().replace(
            r#""jaimini_co_lords":true"#,
            r#""jaimini_co_lords":true,"future_option":3"#,
        );
        write(&s, &with_extra);
        let o = s.open_client(a.id, None).unwrap();
        assert_eq!(o.calc_settings, raman);
        assert!(o.warnings.is_empty());
        assert_eq!(stored_json(&s), with_extra);
    }

    #[test]
    fn tzdb_change_is_checked_and_reported() {
        let mut s = store();
        let (a, ch) = s.create_client(&new_client("A")).unwrap();
        // Eski sürüm, aynı sonuç: uyarı yok, sürüm güncellenir.
        s.conn
            .execute(
                "UPDATE charts SET tzdb_version = '1999a' WHERE id = ?1",
                params![ch.id],
            )
            .unwrap();
        let o = s.open_client(a.id, None).unwrap();
        assert!(o.warnings.is_empty());
        assert_eq!(o.chart.resolved.tzdb_version, atlas::tzdb_version());
        assert_eq!(
            s.chart(ch.id).unwrap().resolved.tzdb_version,
            atlas::tzdb_version()
        );
        // Eski sürüm, farklı sonuç: saklanan an kullanılır, uyarı yeni anı taşır.
        s.conn
            .execute(
                "UPDATE charts SET tzdb_version = '1999a', resolved_utc = '2000-01-01T11:00:00Z',
                 resolved_jd_ut = 2451544.9583333335 WHERE id = ?1",
                params![ch.id],
            )
            .unwrap();
        let o = s.open_client(a.id, None).unwrap();
        assert_eq!(o.chart.resolved.jd_ut, 2_451_544.958_333_333_5);
        assert_eq!(
            o.warnings,
            [StoreWarning::TzdbChanged {
                stored_version: "1999a".into(),
                current_version: atlas::tzdb_version().into(),
                current_utc: Some("2000-01-01T12:00:00Z".into()),
            }]
        );
    }

    #[test]
    fn migrated_settings_are_written_back_and_reset_is_not() {
        let mut s = store();
        let (_, ch) = s.create_client(&new_client("A")).unwrap();
        let migrated = CalcSettings {
            ayanamsa: AyanamsaSystem::Raman,
            ..CalcSettings::default()
        };
        let mut warnings = Vec::new();
        let got = settle_settings(
            &s.conn,
            ch.id,
            SettingsLoad::Migrated {
                settings: migrated.clone(),
                from: 0,
            },
            &mut warnings,
        )
        .unwrap();
        assert_eq!(got, migrated);
        assert!(warnings.is_empty());
        let stored: String = s
            .conn
            .query_row(
                "SELECT calc_settings_json FROM charts WHERE id = ?1",
                params![ch.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stored, serde_json::to_string(&migrated).unwrap());
        // Reset: uyarı verir, saklanan JSON'a dokunmaz.
        let reset = settle_settings(
            &s.conn,
            ch.id,
            SettingsLoad::Reset {
                settings: CalcSettings::default(),
                found_version: Some(9),
            },
            &mut warnings,
        )
        .unwrap();
        assert_eq!(reset, CalcSettings::default());
        assert_eq!(
            warnings,
            [StoreWarning::CalcSettingsReset {
                found_version: Some(9)
            }]
        );
        let after: String = s
            .conn
            .query_row(
                "SELECT calc_settings_json FROM charts WHERE id = ?1",
                params![ch.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(after, stored);
    }

    #[test]
    fn open_writes_are_one_transaction() {
        let mut s = store();
        let (a, ch) = s.create_client(&new_client("A")).unwrap();
        // Eski tzdb sürümü, aynı sonuç: açılış sürümü yazmak ister; son yazım başarısız olur.
        s.conn
            .execute(
                "UPDATE charts SET tzdb_version = '1999a' WHERE id = ?1",
                params![ch.id],
            )
            .unwrap();
        s.conn
            .execute_batch(
                "CREATE TEMP TRIGGER fail_open BEFORE UPDATE OF last_opened_at ON charts
                 BEGIN SELECT RAISE(ABORT, 'fail'); END;",
            )
            .unwrap();
        assert!(matches!(
            s.open_client(a.id, None),
            Err(StoreError::Sqlite(_))
        ));
        assert_eq!(s.client(a.id).unwrap().last_opened_at, None);
        assert_eq!(s.chart(ch.id).unwrap().resolved.tzdb_version, "1999a");
        assert_eq!(s.last_opened().unwrap(), None);
    }

    #[test]
    fn last_opened_follows_open_order_within_one_millisecond() {
        let mut s = stuck_store();
        let (a, _) = s.create_client(&new_client("A")).unwrap();
        let (b, _) = s.create_client(&new_client("B")).unwrap();
        s.open_client(b.id, None).unwrap();
        s.open_client(a.id, None).unwrap();
        assert_eq!(s.last_opened().unwrap().map(|(c, _)| c), Some(a.id));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]
        /// Spec §8: rastgele ya da bozuk `view_state_json` ile açılış asla hata dönmez.
        #[test]
        fn open_never_fails_on_garbage_view_state(chart_raw in ".{0,120}", layout_raw in ".{0,120}") {
            let mut s = store();
            let (a, ch) = s.create_client(&new_client("A")).unwrap();
            s.conn.execute(
                "UPDATE charts SET view_state_json = ?1 WHERE id = ?2",
                params![chart_raw, ch.id],
            ).unwrap();
            s.app_state_set("panel_layout", &layout_raw).unwrap();
            let o = s.open_client(a.id, None);
            prop_assert!(o.is_ok());
        }
    }

    #[test]
    fn silent_tzdb_refresh_rewrites_abbrev_offset_and_warnings() {
        let mut s = store();
        let (a, ch) = s.create_client(&new_client("A")).unwrap();
        let fresh = ch.resolved.clone();
        s.conn
            .execute(
                "UPDATE charts SET tzdb_version = '1999a', resolved_abbrev = 'OLD',
                 resolved_offset_secs = 1, resolved_warnings_json = '[\"PreTzdbReliability\"]'
                 WHERE id = ?1",
                params![ch.id],
            )
            .unwrap();
        let o = s.open_client(a.id, None).unwrap();
        assert!(o.warnings.is_empty());
        assert_eq!(o.chart.resolved, fresh);
        assert_eq!(s.chart(ch.id).unwrap().resolved, fresh);
    }
}
