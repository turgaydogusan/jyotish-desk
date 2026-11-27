//! Uyum panelinin store yarısı (ileri modüller spec'i §9): eş haritanın modül girdisi
//! (`chart_input`), panel başlığında gösterilen kısa kimliği (`chart_brief`) ve eş bağının
//! bakımı. Eş bağı (`ViewState.compat_partner_uuid`) iki danışan arasında bir ilişki
//! bilgisidir: yalnızca var olan başka bir haritayı gösterirken saklanır, gösterdiği harita
//! silinince temizlenir ve hiçbir yerde loglanmaz.

use jyotish_core::ChartInput;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::charts::{load_chart, load_client};
use crate::model::{ChartKind, Gender, TimeCertainty};
use crate::view_state::{
    canonical_uuid, chart_view_chara_path, chart_view_compat, chart_view_json, chart_view_path,
    CompatRoles, ViewState,
};
use crate::{Store, StoreError};

/// Bir haritanın panel başlığında gösterilen kısa kimliği: danışanın adı ve cinsiyeti,
/// haritanın etiketi ve türü. Doğum verisi, not ve yer taşımaz.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ChartBrief {
    pub client_name: String,
    pub client_gender: Option<Gender>,
    /// Boşsa arayüz harita türünün yerelleştirilmiş adını gösterir.
    pub chart_label: String,
    pub chart_kind: ChartKind,
}

/// UUID'si verilen haritanın kimliği; UUID yazımı geçersizse `InvalidInput`, harita yoksa
/// `None`.
fn chart_id_by_uuid(conn: &Connection, chart_uuid: &str) -> Result<Option<u32>, StoreError> {
    let uuid = canonical_uuid(chart_uuid).ok_or(StoreError::InvalidInput("chart_uuid"))?;
    Ok(conn
        .query_row(
            "SELECT id FROM charts WHERE uuid = ?1",
            params![uuid],
            |r| r.get(0),
        )
        .optional()?)
}

/// Eş bağını saklanmadan ve okunmadan önce süzer: yalnızca var olan ve `chart_id`'den başka
/// bir haritayı gösteren bağ durur; diğer her durumda bağ yoktur ve rol varsayılandır.
pub(crate) fn settled_compat(
    conn: &Connection,
    chart_id: u32,
    partner: Option<&str>,
    roles: CompatRoles,
) -> Result<(Option<String>, CompatRoles), StoreError> {
    let Some(uuid) = partner.and_then(canonical_uuid) else {
        return Ok((None, CompatRoles::default()));
    };
    let other: Option<u32> = conn
        .query_row(
            "SELECT id FROM charts WHERE uuid = ?1 AND id <> ?2",
            params![uuid, chart_id],
            |r| r.get(0),
        )
        .optional()?;
    Ok(match other {
        Some(_) => (Some(uuid), roles),
        None => (None, CompatRoles::default()),
    })
}

/// Silinen haritaları eş olarak gösteren bağları temizler; dasha yolları durur. Silme ile aynı
/// işlemde çağrılır. `removed` kanonik UUID'lerdir.
pub(crate) fn clear_compat_links(conn: &Connection, removed: &[String]) -> Result<(), StoreError> {
    for uuid in removed {
        let rows: Vec<(u32, String)> = {
            let mut stmt = conn.prepare_cached(
                "SELECT id, view_state_json FROM charts WHERE instr(view_state_json, ?1) > 0",
            )?;
            let rows = stmt.query_map(params![uuid], |r| Ok((r.get(0)?, r.get(1)?)))?;
            rows.collect::<Result<_, _>>()?
        };
        for (id, raw) in rows {
            if chart_view_compat(&raw).0.as_deref() != Some(uuid.as_str()) {
                continue;
            }
            let cleared = ViewState {
                dasha_expanded_path: chart_view_path(&raw),
                chara_expanded_path: chart_view_chara_path(&raw),
                ..ViewState::default()
            };
            conn.execute(
                "UPDATE charts SET view_state_json = ?1 WHERE id = ?2",
                params![chart_view_json(&cleared), id],
            )?;
        }
    }
    Ok(())
}

impl Store {
    /// Bir haritanın modül girdisi: çözülmüş an, konum, haritanın kendi hesap ayarı ve doğum
    /// saatinin bilinip bilinmediği. Ad, not, yer adı ve saat dilimi dönmez. Ayar göçü burada
    /// yazılmaz (okunan ayar göç ettirilmiş hâliyle kullanılır; yazım `open_client`'tadır).
    /// Harita yoksa `None`.
    pub fn chart_input(&self, chart_uuid: &str) -> Result<Option<ChartInput>, StoreError> {
        let Some(id) = chart_id_by_uuid(&self.conn, chart_uuid)? else {
            return Ok(None);
        };
        let chart = load_chart(&self.conn, id)?;
        let settings = self.calc_settings(id)?.settings().clone();
        Ok(Some(ChartInput {
            jd_ut: chart.resolved.jd_ut,
            lat: chart.birth.place.lat,
            lon: chart.birth.place.lon,
            settings,
            time_known: chart.birth.time_certainty != TimeCertainty::Unknown,
            // Sayı prashnası eş harita olarak da sayının lagnasıyla hesaplanır.
            lagna_number: crate::prashna::load_prashna(&self.conn, id)?.and_then(|p| p.number),
        }))
    }

    /// Bir haritanın kısa kimliği (uyum panelinin başlığı ve ilk rol ataması). Harita yoksa
    /// `None`.
    pub fn chart_brief(&self, chart_uuid: &str) -> Result<Option<ChartBrief>, StoreError> {
        let Some(id) = chart_id_by_uuid(&self.conn, chart_uuid)? else {
            return Ok(None);
        };
        let chart = load_chart(&self.conn, id)?;
        let client = load_client(&self.conn, chart.client_id)?;
        Ok(Some(ChartBrief {
            client_name: client.name,
            client_gender: client.gender,
            chart_label: chart.label,
            chart_kind: chart.kind,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::birth::fixtures::istanbul;
    use crate::charts::testutil::new_client;
    use crate::db::testutil::store;
    use crate::model::{ChartRecord, NewChart, NewClient};
    use crate::settings::SettingsLoad;
    use jyotish_core::{AyanamsaSystem, CalcSettings, Graha, Sign, SignDashaPath};
    use uuid::Uuid;

    /// İki danışan, birer harita: (store, A'nın haritası, B'nin haritası).
    fn pair() -> (Store, ChartRecord, ChartRecord) {
        let mut s = store();
        let (_, a) = s.create_client(&new_client("Ayşe")).unwrap();
        let (_, b) = s
            .create_client(&NewClient {
                gender: Some(Gender::Male),
                ..new_client("Mehmet")
            })
            .unwrap();
        (s, a, b)
    }

    fn linked(partner: &ChartRecord, roles: CompatRoles) -> ViewState {
        ViewState {
            dasha_expanded_path: vec![Graha::Rahu],
            chara_expanded_path: SignDashaPath {
                cycle: 2,
                signs: vec![Sign::Leo],
            },
            compat_partner_uuid: Some(partner.uuid.clone()),
            compat_roles: roles,
            ..ViewState::default()
        }
    }

    #[test]
    fn the_partner_link_is_kept_per_chart_and_only_to_an_existing_other_chart() {
        let (mut s, a, b) = pair();
        assert_eq!(s.view_state(a.id).unwrap().compat_partner_uuid, None);
        s.save_view_state(a.id, &linked(&b, CompatRoles::OpenIsBride))
            .unwrap();
        let read = s.view_state(a.id).unwrap();
        assert_eq!(read.compat_partner_uuid, Some(b.uuid.clone()));
        assert_eq!(read.compat_roles, CompatRoles::OpenIsBride);
        // Bağ tek yönlüdür ve haritanındır: eşin kendi kaydı değişmez.
        assert_eq!(s.view_state(b.id).unwrap(), ViewState::default());
        // Görünüm kaydı içerik değildir.
        assert_eq!(s.chart(a.id).unwrap().updated_at, a.updated_at);

        // Kendisini gösteren bağ ve var olmayan haritayı gösteren bağ saklanmaz; rol de
        // varsayılana döner. Dasha yolları yazılır.
        for partner in [a.uuid.clone(), Uuid::new_v4().to_string()] {
            let vs = ViewState {
                compat_partner_uuid: Some(partner),
                ..linked(&b, CompatRoles::OpenIsBride)
            };
            s.save_view_state(a.id, &vs).unwrap();
            let read = s.view_state(a.id).unwrap();
            assert_eq!(
                (read.compat_partner_uuid, read.compat_roles),
                (None, CompatRoles::OpenIsGroom)
            );
            assert_eq!(read.dasha_expanded_path, [Graha::Rahu]);
        }
        // Kanonik olmayan UUID yazımı reddedilir; hiçbir şey yazılmaz.
        s.save_view_state(a.id, &linked(&b, CompatRoles::OpenIsGroom))
            .unwrap();
        let shouting = ViewState {
            compat_partner_uuid: Some(b.uuid.to_uppercase()),
            ..ViewState::default()
        };
        assert!(matches!(
            s.save_view_state(a.id, &shouting),
            Err(StoreError::InvalidInput("view_state"))
        ));
        assert_eq!(
            s.view_state(a.id).unwrap().compat_partner_uuid,
            Some(b.uuid.clone())
        );
        // Aynı danışanın başka haritası eş olabilir.
        let second = s.add_chart(a.client_id, &new_client("Ayşe").chart).unwrap();
        s.save_view_state(a.id, &linked(&second, CompatRoles::OpenIsGroom))
            .unwrap();
        assert_eq!(
            s.view_state(a.id).unwrap().compat_partner_uuid,
            Some(second.uuid)
        );
    }

    #[test]
    fn a_stored_link_to_a_missing_chart_reads_as_no_partner() {
        let (s, a, _) = pair();
        // Elle düzenlenmiş kayıt: var olmayan bir haritayı gösteriyor.
        let ghost = Uuid::new_v4().to_string();
        let raw = format!(
            r#"{{"schema_version":1,"dasha_expanded_path":["Venus"],"compat_partner_uuid":"{ghost}","compat_roles":"OpenIsBride"}}"#
        );
        s.conn
            .execute(
                "UPDATE charts SET view_state_json = ?1 WHERE id = ?2",
                params![raw, a.id],
            )
            .unwrap();
        let read = s.view_state(a.id).unwrap();
        assert_eq!(
            (read.compat_partner_uuid, read.compat_roles),
            (None, CompatRoles::OpenIsGroom)
        );
        assert_eq!(read.dasha_expanded_path, [Graha::Venus]);
    }

    #[test]
    fn deleting_a_chart_or_a_client_clears_the_links_that_point_to_it() {
        let (mut s, a, b) = pair();
        let (_, c) = s.create_client(&new_client("Zeynep")).unwrap();
        let b2 = s
            .add_chart(b.client_id, &new_client("Mehmet").chart)
            .unwrap();
        s.save_view_state(a.id, &linked(&b2, CompatRoles::OpenIsBride))
            .unwrap();
        s.save_view_state(c.id, &linked(&b, CompatRoles::OpenIsGroom))
            .unwrap();
        s.save_view_state(b.id, &linked(&a, CompatRoles::OpenIsGroom))
            .unwrap();
        let stored = |s: &Store, id: u32| -> String {
            s.conn
                .query_row(
                    "SELECT view_state_json FROM charts WHERE id = ?1",
                    params![id],
                    |r| r.get(0),
                )
                .unwrap()
        };
        assert!(stored(&s, a.id).contains(&b2.uuid));

        // Harita silinir: onu gösteren bağ ve rol temizlenir, dasha yolları durur; başka
        // haritayı gösteren bağlara dokunulmaz.
        s.delete_chart(b2.id).unwrap();
        assert!(!stored(&s, a.id).contains(&b2.uuid));
        let read = s.view_state(a.id).unwrap();
        assert_eq!(
            (read.compat_partner_uuid, read.compat_roles),
            (None, CompatRoles::OpenIsGroom)
        );
        assert_eq!(read.dasha_expanded_path, [Graha::Rahu]);
        assert_eq!(
            read.chara_expanded_path,
            SignDashaPath {
                cycle: 2,
                signs: vec![Sign::Leo]
            }
        );
        assert_eq!(
            s.view_state(c.id).unwrap().compat_partner_uuid,
            Some(b.uuid.clone())
        );

        // Danışan silinir: bütün haritalarını gösteren bağlar temizlenir; UUID veritabanında
        // hiçbir görünüm kaydında durmaz.
        s.delete_client(b.client_id).unwrap();
        assert_eq!(s.view_state(c.id).unwrap().compat_partner_uuid, None);
        assert!(!stored(&s, c.id).contains(&b.uuid));
        let left: u32 = s
            .conn
            .query_row(
                "SELECT count(*) FROM charts WHERE instr(view_state_json, ?1) > 0",
                params![b.uuid],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(left, 0);

        // Son haritası silinemeyen danışanda (`LastChart`) hiçbir bağ temizlenmez.
        s.save_view_state(c.id, &linked(&a, CompatRoles::OpenIsGroom))
            .unwrap();
        assert!(matches!(s.delete_chart(a.id), Err(StoreError::LastChart)));
        assert_eq!(
            s.view_state(c.id).unwrap().compat_partner_uuid,
            Some(a.uuid.clone())
        );
    }

    #[test]
    fn chart_input_carries_the_moment_the_place_the_settings_and_nothing_personal() {
        let (mut s, a, _) = pair();
        let raman = CalcSettings {
            ayanamsa: AyanamsaSystem::Raman,
            ..CalcSettings::default()
        };
        s.save_calc_settings(a.id, &raman).unwrap();
        s.add_note(a.client_id, Some(a.id), "gizli seans notu")
            .unwrap();
        let input = s.chart_input(&a.uuid).unwrap().expect("chart exists");
        assert_eq!(
            input,
            ChartInput {
                jd_ut: a.resolved.jd_ut,
                lat: a.birth.place.lat,
                lon: a.birth.place.lon,
                settings: raman,
                time_known: true,
                lagna_number: None,
            }
        );
        // Dönen değer ad, not, yer adı, tarih metni ve saat dilimi taşımaz: yalnızca altı alan.
        let json = serde_json::to_value(&input).unwrap();
        let mut keys: Vec<&str> = json
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "jd_ut",
                "lagna_number",
                "lat",
                "lon",
                "settings",
                "time_known"
            ]
        );
        let text = json.to_string();
        for private in ["Ayşe", "gizli", "New Delhi", "Asia/Kolkata", "2000-01-01"] {
            assert!(!text.contains(private), "{private}");
        }
        // UUID yazımı büyük harfli de olabilir; var olmayan harita `None`, UUID olmayan metin
        // hatadır.
        assert_eq!(s.chart_input(&a.uuid.to_uppercase()).unwrap(), Some(input));
        assert_eq!(s.chart_input(&Uuid::new_v4().to_string()).unwrap(), None);
        assert!(matches!(
            s.chart_input("ayşe"),
            Err(StoreError::InvalidInput("chart_uuid"))
        ));
        // Okuma ayar göçünü yazmaz ve "son açılan" işaretlerine dokunmaz.
        assert_eq!(s.chart(a.id).unwrap().last_opened_at, None);
        assert!(matches!(
            s.calc_settings(a.id).unwrap(),
            SettingsLoad::Current(_)
        ));
    }

    #[test]
    fn chart_input_marks_an_unknown_birth_time() {
        let mut s = store();
        let mut birth = istanbul("1987-03-17", "09:00");
        birth.time = None;
        birth.time_certainty = TimeCertainty::Unknown;
        let (_, ch) = s
            .create_client(&NewClient {
                chart: NewChart {
                    birth,
                    ..new_client("A").chart
                },
                ..new_client("A")
            })
            .unwrap();
        let input = s.chart_input(&ch.uuid).unwrap().expect("chart exists");
        assert!(!input.time_known);
        assert_eq!(input.jd_ut, ch.resolved.jd_ut);
    }

    #[test]
    fn chart_brief_names_the_client_and_the_chart_only() {
        let (mut s, _, b) = pair();
        let event = s
            .add_chart(
                b.client_id,
                &NewChart {
                    label: "Düğün".into(),
                    kind: ChartKind::Event,
                    ..new_client("Mehmet").chart
                },
            )
            .unwrap();
        assert_eq!(
            s.chart_brief(&b.uuid).unwrap(),
            Some(ChartBrief {
                client_name: "Mehmet".into(),
                client_gender: Some(Gender::Male),
                chart_label: String::new(),
                chart_kind: ChartKind::Natal,
            })
        );
        assert_eq!(
            s.chart_brief(&event.uuid).unwrap(),
            Some(ChartBrief {
                client_name: "Mehmet".into(),
                client_gender: Some(Gender::Male),
                chart_label: "Düğün".into(),
                chart_kind: ChartKind::Event,
            })
        );
        assert_eq!(s.chart_brief(&Uuid::new_v4().to_string()).unwrap(), None);
        assert!(matches!(
            s.chart_brief("x"),
            Err(StoreError::InvalidInput("chart_uuid"))
        ));
    }
}
