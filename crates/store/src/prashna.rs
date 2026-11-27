//! Prashna kayıtları (ileri modüller spec'i §11 "Saklama"). Prashna haritası danışana bağlı bir
//! `Event` haritasıdır; `prashna` yan tablosunda satırı olan harita prashnadır. Yan tablo soru
//! metnini, isteğe bağlı sayıyı (sayı prashnası: lagna sayıdan gelir) ve astrologun seçtiği
//! odak evini tutar.
//!
//! Soru metni not gövdesi gibi hassastır: loglanmaz, hata iletisine girmez, arama dizinine
//! eklenmez ve dışa aktarıma yalnızca notlar dahil edilirse yazılır. Bu dosyanın ürettiği
//! hatalar yalnızca alan adı taşır.

use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::birth::resolve_birth;
use crate::charts::{clean_name, insert_chart, insert_client, ChartRow};
use crate::model::{ChartKind, ChartRecord, NewPrashna, PrashnaMeta};
use crate::view_state::empty_chart_view_json;
use crate::{Store, StoreError};

/// Soru metninin en büyük uzunluğu, karakter (spec §11).
pub const MAX_QUESTION_CHARS: usize = 2000;
/// Sayı prashnasının en büyük sayısı (`jyotish_core::PRASHNA_NUMBER_MAX` ile aynı).
pub const MAX_PRASHNA_NUMBER: u8 = jyotish_core::PRASHNA_NUMBER_MAX;
/// Danışansız prashnaların toplandığı yerleşik danışanın adı. Klasik terimdir: üç dilde aynı
/// yazılır (K11). Kullanıcı adı değiştirebilir; danışan adından değil, `app_state`'teki
/// UUID'sinden tanınır.
pub const PRASHNA_CLIENT_NAME: &str = "Prashna";
/// `app_state` anahtarı: yerleşik prashna danışanının UUID'si (JSON dizgi).
const PRASHNA_CLIENT_KEY: &str = "prashna_client_uuid";

/// Soru metnini denetler ve kırpar (baştaki ve sondaki boşluk atılır).
pub(crate) fn clean_question(question: &str) -> Result<String, StoreError> {
    let q = question.trim();
    if q.chars().count() > MAX_QUESTION_CHARS {
        return Err(StoreError::InvalidInput("prashna.question"));
    }
    Ok(q.to_owned())
}

pub(crate) fn check_number(number: Option<u8>) -> Result<(), StoreError> {
    match number {
        Some(n) if !(1..=MAX_PRASHNA_NUMBER).contains(&n) => {
            Err(StoreError::InvalidInput("prashna.number"))
        }
        _ => Ok(()),
    }
}

pub(crate) fn check_focus_house(house: Option<u8>) -> Result<(), StoreError> {
    match house {
        Some(h) if !(1..=12).contains(&h) => Err(StoreError::InvalidInput("prashna.focus_house")),
        _ => Ok(()),
    }
}

/// Yan tablo satırını yazar (yeni harita ve içe aktarma ortak kullanır).
pub(crate) fn insert_prashna(
    conn: &Connection,
    chart_id: u32,
    meta: &PrashnaMeta,
) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO prashna (chart_id, question, number, focus_house, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            chart_id,
            meta.question,
            meta.number,
            meta.focus_house,
            meta.created_at
        ],
    )?;
    Ok(())
}

/// Haritanın prashna kaydı; harita prashna değilse `None`.
pub(crate) fn load_prashna(
    conn: &Connection,
    chart_id: u32,
) -> Result<Option<PrashnaMeta>, StoreError> {
    Ok(conn
        .query_row(
            "SELECT question, number, focus_house, created_at FROM prashna WHERE chart_id = ?1",
            params![chart_id],
            |r| {
                Ok(PrashnaMeta {
                    question: r.get(0)?,
                    number: r.get(1)?,
                    focus_house: r.get(2)?,
                    created_at: r.get(3)?,
                })
            },
        )
        .optional()?)
}

impl Store {
    /// Yerleşik prashna danışanının kimliği; hiç oluşturulmadıysa ya da silindiyse `None`.
    pub fn prashna_client_id(&self) -> Result<Option<u32>, StoreError> {
        let Some(json) = self.app_state_get(PRASHNA_CLIENT_KEY)? else {
            return Ok(None);
        };
        // Okunamayan kayıt "yok" sayılır: sonraki prashna yeni bir yerleşik danışan açar.
        let Ok(uuid) = serde_json::from_str::<String>(&json) else {
            return Ok(None);
        };
        Ok(self
            .conn
            .query_row(
                "SELECT id FROM clients WHERE uuid = ?1",
                params![uuid],
                |r| r.get(0),
            )
            .optional()?)
    }

    /// Prashna haritasını kurar: `Event` türünde bir harita ve yan tablo satırı, tek işlemde.
    /// `input.client_id` verilmişse o danışana, verilmemişse yerleşik "Prashna" danışanına
    /// eklenir (yoksa aynı işlemde oluşturulur). Harita, genel varsayılan ayarların tam
    /// kopyasını alır. Dönen çift danışanın kimliği ve yeni haritadır.
    pub fn create_prashna(
        &mut self,
        input: &NewPrashna,
        label: &str,
    ) -> Result<(u32, ChartRecord), StoreError> {
        check_number(input.number)?;
        check_focus_house(input.focus_house)?;
        let question = clean_question(&input.question)?;
        let label = clean_name(label, "label")?;
        if let Some(id) = input.client_id {
            self.client(id)?;
        }
        let (birth, resolved) = resolve_birth(&input.birth)?;
        let calc_json = serde_json::to_string(&self.app_prefs()?.calc_settings)?;
        let book = match input.client_id {
            Some(_) => None,
            None => self.prashna_client_id()?,
        };
        let now = self.now();
        let tx = self.conn.transaction()?;
        let (client_id, created) = match input.client_id.or(book) {
            Some(id) => (id, false),
            None => {
                let uuid = Uuid::new_v4().to_string();
                let id = insert_client(&tx, &uuid, PRASHNA_CLIENT_NAME, None, &now, &now)?;
                tx.execute(
                    "INSERT INTO app_state (key, value_json) VALUES (?1, ?2)
                     ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
                    params![PRASHNA_CLIENT_KEY, serde_json::to_string(&uuid)?],
                )?;
                (id, true)
            }
        };
        let chart_id = insert_chart(
            &tx,
            &ChartRow {
                uuid: &Uuid::new_v4().to_string(),
                client_id,
                label: &label,
                kind: ChartKind::Event,
                birth: &birth,
                resolved: &resolved,
                calc_json: &calc_json,
                view_json: &empty_chart_view_json(),
                created_at: &now,
                updated_at: &now,
            },
        )?;
        insert_prashna(
            &tx,
            chart_id,
            &PrashnaMeta {
                question,
                number: input.number,
                focus_house: input.focus_house,
                created_at: now.clone(),
            },
        )?;
        if created {
            tx.execute(
                "UPDATE clients SET last_chart_id = ?1 WHERE id = ?2",
                params![chart_id, client_id],
            )?;
        } else {
            tx.execute(
                "UPDATE clients SET updated_at = ?1 WHERE id = ?2",
                params![now, client_id],
            )?;
        }
        tx.commit()?;
        Ok((client_id, self.chart(chart_id)?))
    }

    /// Haritanın prashna kaydı; harita prashna değilse `None`.
    pub fn prashna_meta(&self, chart_id: u32) -> Result<Option<PrashnaMeta>, StoreError> {
        load_prashna(&self.conn, chart_id)
    }

    /// Soru metnini ve odak evini değiştirir (otomatik kayıt). Sayı ve an değişmez: sayı
    /// haritanın lagnasıdır, an doğum formundan düzenlenir. Harita prashna değilse `NotFound`.
    pub fn update_prashna(
        &mut self,
        chart_id: u32,
        question: &str,
        focus_house: Option<u8>,
    ) -> Result<PrashnaMeta, StoreError> {
        check_focus_house(focus_house)?;
        let question = clean_question(question)?;
        let changed = self.conn.execute(
            "UPDATE prashna SET question = ?1, focus_house = ?2 WHERE chart_id = ?3",
            params![question, focus_house, chart_id],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound {
                what: "prashna",
                id: chart_id,
            });
        }
        load_prashna(&self.conn, chart_id)?.ok_or(StoreError::NotFound {
            what: "prashna",
            id: chart_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::birth::fixtures::{delhi, istanbul};
    use crate::charts::testutil::new_client;
    use crate::db::testutil::store;
    use crate::model::{NewChart, TimeCertainty};
    use crate::{ExportOptions, SCHEMA_VERSION};

    const QUESTION: &str = "Kayıp anahtar bulunacak mı?";

    fn new_prashna(client_id: Option<u32>) -> NewPrashna {
        NewPrashna {
            client_id,
            birth: delhi(),
            number: None,
            question: format!("  {QUESTION}\n"),
            focus_house: Some(4),
        }
    }

    #[test]
    fn a_prashna_is_an_event_chart_with_a_side_row() {
        let mut s = store();
        let (client, natal) = s.create_client(&new_client("Ayşe")).unwrap();
        let (owner, chart) = s
            .create_prashna(&new_prashna(Some(client.id)), "Prashna 2000-01-01 17:30")
            .unwrap();
        assert_eq!(owner, client.id);
        assert_eq!(chart.client_id, client.id);
        assert_eq!(chart.kind, ChartKind::Event);
        assert_eq!(chart.label, "Prashna 2000-01-01 17:30");
        assert_eq!(chart.birth.time_certainty, TimeCertainty::Exact);
        let meta = s.prashna_meta(chart.id).unwrap().expect("a prashna row");
        // Soru kırpılır; sayı yoksa zaman prashnasıdır.
        assert_eq!(meta.question, QUESTION);
        assert_eq!((meta.number, meta.focus_house), (None, Some(4)));
        assert_eq!(meta.created_at, chart.created_at);
        // Doğum haritası prashna değildir.
        assert_eq!(s.prashna_meta(natal.id).unwrap(), None);
        // Harita genel varsayılan ayarların kopyasını alır ve danışanın haritaları arasındadır.
        assert_eq!(s.charts_of(client.id).unwrap().len(), 2);
        assert!(s.client(client.id).unwrap().updated_at >= chart.created_at);
        // Şema sürümü 2'dir ve `kind` değişmemiştir.
        assert_eq!((s.schema_version().unwrap(), SCHEMA_VERSION), (2, 2));
    }

    #[test]
    fn without_a_client_prashnas_gather_under_one_built_in_client() {
        let mut s = store();
        assert_eq!(s.prashna_client_id().unwrap(), None);
        let (book, first) = s.create_prashna(&new_prashna(None), "Prashna 1").unwrap();
        let client = s.client(book).unwrap();
        assert_eq!(client.name, PRASHNA_CLIENT_NAME);
        assert_eq!(client.last_chart_id, Some(first.id));
        assert_eq!(s.prashna_client_id().unwrap(), Some(book));
        // İkinci prashna aynı danışana eklenir; danışan ada göre değil kayda göre bulunur.
        s.update_client(book, "Sorular", None).unwrap();
        let (again, second) = s.create_prashna(&new_prashna(None), "Prashna 2").unwrap();
        assert_eq!(again, book);
        assert_eq!(second.client_id, book);
        assert_eq!(s.list_clients(10).unwrap().len(), 1);
        // "Prashna" adlı başka bir danışan yerleşik danışan sayılmaz.
        let (other, _) = s.create_client(&new_client(PRASHNA_CLIENT_NAME)).unwrap();
        assert_ne!(other.id, book);
        assert_eq!(s.prashna_client_id().unwrap(), Some(book));
        // Yerleşik danışan silinirse sonraki prashna yenisini açar.
        s.delete_client(book).unwrap();
        assert_eq!(s.prashna_client_id().unwrap(), None);
        let (fresh, _) = s.create_prashna(&new_prashna(None), "Prashna 3").unwrap();
        assert_ne!(fresh, book);
        assert_eq!(s.client(fresh).unwrap().name, PRASHNA_CLIENT_NAME);
        // Okunamayan kayıt da "yok"tur.
        s.app_state_set(PRASHNA_CLIENT_KEY, "{").unwrap();
        assert_eq!(s.prashna_client_id().unwrap(), None);
    }

    #[test]
    fn question_number_and_focus_house_round_trip_through_open_client() {
        let mut s = store();
        let (client, _) = s.create_client(&new_client("Ayşe")).unwrap();
        let input = NewPrashna {
            number: Some(54),
            ..new_prashna(Some(client.id))
        };
        let (_, chart) = s.create_prashna(&input, "Prashna").unwrap();
        // Kur → başka haritayı aç → yeniden aç.
        let natal = s.charts_of(client.id).unwrap()[1].id;
        assert_eq!(s.open_client(client.id, Some(natal)).unwrap().prashna, None);
        let opened = s.open_client(client.id, Some(chart.id)).unwrap();
        let meta = opened.prashna.expect("prashna meta");
        assert_eq!(meta.question, QUESTION);
        assert_eq!((meta.number, meta.focus_house), (Some(54), Some(4)));
        // Modül girdisi sayıyı taşır: eş harita olarak seçilen sayı prashnası da sayının
        // lagnasıyla hesaplanır.
        let module = s.chart_input(&chart.uuid).unwrap().unwrap();
        assert_eq!(module.lagna_number, Some(54));
        assert!(module.time_known);
        let plain = s
            .chart_input(&s.chart(natal).unwrap().uuid)
            .unwrap()
            .unwrap();
        assert_eq!(plain.lagna_number, None);
    }

    #[test]
    fn update_changes_the_question_and_the_focus_house_only() {
        let mut s = store();
        let (client, natal) = s.create_client(&new_client("Ayşe")).unwrap();
        let input = NewPrashna {
            number: Some(9),
            ..new_prashna(Some(client.id))
        };
        let (_, chart) = s.create_prashna(&input, "Prashna").unwrap();
        let before = s.prashna_meta(chart.id).unwrap().unwrap();
        let after = s.update_prashna(chart.id, " Yeni soru ", None).unwrap();
        assert_eq!(after.question, "Yeni soru");
        assert_eq!(after.focus_house, None);
        assert_eq!(
            (after.number, &after.created_at),
            (Some(9), &before.created_at)
        );
        assert_eq!(s.prashna_meta(chart.id).unwrap(), Some(after));
        // Boş soru geçerlidir.
        assert_eq!(
            s.update_prashna(chart.id, "", Some(12)).unwrap().question,
            ""
        );
        // Prashna olmayan harita ve olmayan harita.
        for id in [natal.id, 9999] {
            assert!(matches!(
                s.update_prashna(id, "x", None),
                Err(StoreError::NotFound { what: "prashna", id: got }) if got == id
            ));
        }
        // Doğum formundan düzenleme prashna kaydını korur.
        let edited = NewChart {
            label: "Düzeltilmiş".into(),
            kind: ChartKind::Event,
            birth: istanbul("2024-06-01", "14:30:00"),
        };
        s.update_chart(chart.id, &edited).unwrap();
        assert_eq!(s.prashna_meta(chart.id).unwrap().unwrap().number, Some(9));
    }

    #[test]
    fn invalid_input_names_the_field_and_writes_nothing() {
        let mut s = store();
        let (client, _) = s.create_client(&new_client("Ayşe")).unwrap();
        let long = "ş".repeat(MAX_QUESTION_CHARS + 1);
        let cases: [(NewPrashna, &str); 5] = [
            (
                NewPrashna {
                    number: Some(0),
                    ..new_prashna(Some(client.id))
                },
                "prashna.number",
            ),
            (
                NewPrashna {
                    number: Some(109),
                    ..new_prashna(Some(client.id))
                },
                "prashna.number",
            ),
            (
                NewPrashna {
                    focus_house: Some(0),
                    ..new_prashna(Some(client.id))
                },
                "prashna.focus_house",
            ),
            (
                NewPrashna {
                    focus_house: Some(13),
                    ..new_prashna(Some(client.id))
                },
                "prashna.focus_house",
            ),
            (
                NewPrashna {
                    question: long.clone(),
                    ..new_prashna(Some(client.id))
                },
                "prashna.question",
            ),
        ];
        for (input, field) in cases {
            match s.create_prashna(&input, "Prashna") {
                Err(StoreError::InvalidInput(f)) => assert_eq!(f, field),
                other => panic!("{field}: {other:?}"),
            }
        }
        // Tam sınırdaki soru (çok baytlı karakterlerle) geçerlidir.
        let edge = NewPrashna {
            question: "ş".repeat(MAX_QUESTION_CHARS),
            ..new_prashna(Some(client.id))
        };
        let (_, chart) = s.create_prashna(&edge, "Prashna").unwrap();
        assert!(matches!(
            s.update_prashna(chart.id, &long, None),
            Err(StoreError::InvalidInput("prashna.question"))
        ));
        assert!(matches!(
            s.update_prashna(chart.id, "x", Some(13)),
            Err(StoreError::InvalidInput("prashna.focus_house"))
        ));
        // Olmayan danışan: hiçbir şey yazılmaz, yerleşik danışan da oluşmaz.
        assert!(matches!(
            s.create_prashna(&new_prashna(Some(9999)), "Prashna"),
            Err(StoreError::NotFound { what: "client", .. })
        ));
        let rows: u32 = s
            .conn
            .query_row("SELECT count(*) FROM prashna", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 1);
        assert_eq!(s.prashna_client_id().unwrap(), None);
    }

    #[test]
    fn deleting_the_chart_or_the_client_deletes_the_prashna_row() {
        let mut s = store();
        let (client, _) = s.create_client(&new_client("Ayşe")).unwrap();
        let (_, a) = s
            .create_prashna(&new_prashna(Some(client.id)), "A")
            .unwrap();
        let (_, b) = s
            .create_prashna(&new_prashna(Some(client.id)), "B")
            .unwrap();
        let rows = |s: &Store| -> u32 {
            s.conn
                .query_row("SELECT count(*) FROM prashna", [], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(rows(&s), 2);
        s.delete_chart(a.id).unwrap();
        assert_eq!(rows(&s), 1);
        assert_eq!(s.prashna_meta(b.id).unwrap().unwrap().question, QUESTION);
        s.delete_client(client.id).unwrap();
        assert_eq!(rows(&s), 0);
    }

    #[test]
    fn the_question_is_not_searchable_and_is_exported_only_with_the_notes() {
        let mut s = store();
        let (client, _) = s.create_client(&new_client("Ayşe")).unwrap();
        let input = NewPrashna {
            number: Some(108),
            ..new_prashna(Some(client.id))
        };
        let (_, chart) = s.create_prashna(&input, "Prashna").unwrap();
        // Arama dizini soru metnini görmez.
        for word in ["anahtar", "Kayıp", "bulunacak"] {
            assert!(s.search_clients(word, 10).unwrap().is_empty(), "{word}");
        }
        // Varsayılan dışa aktarım: sayı ve odak evi var, soru yok.
        let plain = s.export_json(&ExportOptions::default()).unwrap();
        assert!(!plain.contains("anahtar"));
        let value: serde_json::Value = serde_json::from_str(&plain).unwrap();
        let exported = value["clients"][0]["charts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["uuid"] == chart.uuid.as_str())
            .unwrap();
        assert_eq!(exported["prashna"]["number"], 108);
        assert_eq!(exported["prashna"]["focus_house"], 4);
        assert!(exported["prashna"]["question"].is_null());
        // Prashna olmayan haritada alan hiç yazılmaz.
        let natal = value["clients"][0]["charts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["uuid"] != chart.uuid.as_str())
            .unwrap();
        assert!(natal.get("prashna").is_none());
        // "Notları dahil et" işaretliyse soru yazılır.
        let with_notes = s
            .export_json(&ExportOptions {
                include_notes: true,
                client_ids: None,
            })
            .unwrap();
        assert!(with_notes.contains(QUESTION));
        // CSV dışa aktarım soru metnini hiç taşımaz.
        assert!(!s.export_csv().unwrap().contains("anahtar"));
    }

    #[test]
    fn import_restores_the_prashna_row_and_rejects_bad_values() {
        let mut s = store();
        let (client, _) = s.create_client(&new_client("Ayşe")).unwrap();
        let input = NewPrashna {
            number: Some(10),
            ..new_prashna(Some(client.id))
        };
        let (_, chart) = s.create_prashna(&input, "Prashna").unwrap();
        let with_notes = s
            .export_json(&ExportOptions {
                include_notes: true,
                client_ids: None,
            })
            .unwrap();
        let plain = s.export_json(&ExportOptions::default()).unwrap();

        let mut t = store();
        let report = t.import_json(&with_notes).unwrap();
        assert_eq!((report.clients_added, report.charts_added), (1, 2));
        let id: u32 = t
            .conn
            .query_row(
                "SELECT id FROM charts WHERE uuid = ?1",
                params![chart.uuid],
                |r| r.get(0),
            )
            .unwrap();
        let meta = t.prashna_meta(id).unwrap().expect("imported prashna");
        assert_eq!(meta.question, QUESTION);
        assert_eq!((meta.number, meta.focus_house), (Some(10), Some(4)));
        // İçe alınan yerleşik danışan kaydı taşınmaz: tercih ve durum içe aktarılmaz.
        assert_eq!(t.prashna_client_id().unwrap(), None);

        // Sorusuz dosya: satır boş soruyla kurulur, harita yine prashnadır.
        let mut u = store();
        u.import_json(&plain).unwrap();
        let id: u32 = u
            .conn
            .query_row(
                "SELECT id FROM charts WHERE uuid = ?1",
                params![chart.uuid],
                |r| r.get(0),
            )
            .unwrap();
        let meta = u.prashna_meta(id).unwrap().unwrap();
        assert_eq!((meta.question.as_str(), meta.number), ("", Some(10)));

        // Geçersiz değerler kaydın UUID'siyle ve alan adıyla reddedilir; hiçbir şey yazılmaz.
        // `u8`'e sığmayan, negatif, kesirli ya da sayı olmayan değer de dosya biçimi hatası
        // değil kayıt hatasıdır: ileti değeri yinelemez.
        for (from, to) in [
            ("\"number\": 10", "\"number\": 109"),
            ("\"number\": 10", "\"number\": 0"),
            ("\"number\": 10", "\"number\": 300"),
            ("\"number\": 10", "\"number\": -1"),
            ("\"number\": 10", "\"number\": 10.5"),
            ("\"number\": 10", "\"number\": \"gizli metin\""),
            ("\"focus_house\": 4", "\"focus_house\": 13"),
            ("\"focus_house\": 4", "\"focus_house\": 70000"),
            ("\"focus_house\": 4", "\"focus_house\": [4]"),
        ] {
            assert!(plain.contains(from), "{from}");
            let mut v = store();
            match v.import_json(&plain.replace(from, to)) {
                Err(e @ StoreError::ImportRecord { .. }) => {
                    assert!(!e.to_string().contains("gizli"), "{to}");
                    let StoreError::ImportRecord { item, field } = e else {
                        unreachable!()
                    };
                    assert_eq!((item.as_str(), field), (chart.uuid.as_str(), "prashna"));
                }
                other => panic!("{to}: {other:?}"),
            }
            assert!(v.list_clients(10).unwrap().is_empty());
        }
        // Sayısı ve odak evi hiç yazılmamış kayıt geçerlidir (zaman prashnası, ev seçilmemiş).
        let bare = plain
            .replace("\"number\": 10,", "")
            .replace("\"focus_house\": 4,", "");
        assert!(!bare.contains("\"number\"") && !bare.contains("\"focus_house\""));
        let mut w = store();
        w.import_json(&bare).unwrap();
        let id: u32 = w
            .conn
            .query_row(
                "SELECT id FROM charts WHERE uuid = ?1",
                params![chart.uuid],
                |r| r.get(0),
            )
            .unwrap();
        let meta = w.prashna_meta(id).unwrap().unwrap();
        assert_eq!((meta.number, meta.focus_house), (None, None));
    }

    /// Projenin ilk şema göçü: sürüm 1 dosyası `Store::open` ile sürüm 2'ye çıkar ve içindeki
    /// danışan, harita, not ve arama dizini olduğu gibi durur.
    #[test]
    fn a_version_one_file_is_upgraded_in_place_and_keeps_its_records() {
        let dir = std::env::temp_dir().join(format!("jd-store-upgrade-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("db.sqlite");
        let (client, chart, note) = {
            let mut s = Store::open(&path).unwrap();
            let (client, chart) = s.create_client(&new_client("Deneme Kaydı")).unwrap();
            let note = s
                .add_note(client.id, Some(chart.id), "göçten önceki not")
                .unwrap();
            (client, chart, note)
        };
        // Sürüm 2 yalnızca `prashna` tablosunu ekler: tabloyu atıp sürümü 1'e çekmek dosyayı
        // sürüm 1'in bıraktığı duruma getirir.
        {
            let c = Connection::open(&path).unwrap();
            c.execute_batch("DROP TABLE prashna; PRAGMA user_version = 1;")
                .unwrap();
            let tables: u32 = c
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE name = 'prashna'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(tables, 0);
        }
        let mut s = Store::open(&path).unwrap();
        assert_eq!(s.schema_version().unwrap(), 2);
        assert_eq!(s.client(client.id).unwrap(), client);
        let kept = s.chart(chart.id).unwrap();
        assert_eq!(kept, chart);
        assert_eq!(kept.resolved.jd_ut, chart.resolved.jd_ut);
        assert_eq!(s.notes(client.id, 10).unwrap(), vec![note]);
        assert_eq!(s.search_clients("Deneme", 10).unwrap().len(), 1);
        assert_eq!(s.search_clients("göçten", 10).unwrap().len(), 1);
        // Eski haritalar prashna değildir; yeni tablo çalışır.
        assert_eq!(s.prashna_meta(chart.id).unwrap(), None);
        assert_eq!(s.open_client(client.id, None).unwrap().prashna, None);
        let (_, asked) = s
            .create_prashna(&new_prashna(Some(client.id)), "Prashna")
            .unwrap();
        assert_eq!(
            s.prashna_meta(asked.id).unwrap().unwrap().question,
            QUESTION
        );
        // Yeniden açılış bir şey değiştirmez.
        drop(s);
        let s = Store::open(&path).unwrap();
        assert_eq!(s.schema_version().unwrap(), 2);
        assert_eq!(s.charts_of(client.id).unwrap().len(), 2);
        drop(s);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
