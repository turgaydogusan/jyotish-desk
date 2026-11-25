//! Seans notları (`client_notes`, spec §4 notlar paneli) ve harita etiketleri (spec §5:
//! etiketler harita düzeyindedir).

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::charts::{prune_tags, MAX_LIST};
use crate::db::last_id;
use crate::model::Note;
use crate::{Store, StoreError};

/// `open_client`'ın döndürdüğü son not sayısı.
pub const RECENT_NOTES: u32 = 20;
/// Etiket uzunluğu sınırı (karakter).
pub const MAX_TAG_CHARS: usize = 64;
/// Not gövdesi sınırı (UTF-8 bayt). Arayüz `maxLength` ile daha önce durur
/// (`MAX_NOTE_UNITS` 16 384 UTF-16 birimi en çok 49 152 bayttır).
pub const MAX_NOTE_BYTES: usize = 65_536;

fn check_body(body: &str) -> Result<(), StoreError> {
    if body.len() > MAX_NOTE_BYTES {
        return Err(StoreError::InvalidInput("body"));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TagCount {
    pub name: String,
    pub charts: u32,
}

const NOTE_COLUMNS: &str = "id, uuid, client_id, chart_id, body, created_at, updated_at";

fn row_to_note(r: &Row<'_>) -> rusqlite::Result<Note> {
    Ok(Note {
        id: r.get(0)?,
        uuid: r.get(1)?,
        client_id: r.get(2)?,
        chart_id: r.get(3)?,
        body: r.get(4)?,
        created_at: r.get(5)?,
        updated_at: r.get(6)?,
    })
}

/// Unicode NFC: "O" + U+0308 ile hazır "Ö" aynı dizgi olur (macOS dosya adları ve bazı
/// klavyeler ayrışık biçim üretir).
pub(crate) fn nfc(text: &str) -> String {
    icu_normalizer::ComposingNormalizerBorrowed::new_nfc()
        .normalize(text)
        .into_owned()
}

/// Etiketin kimlik anahtarı (`tags.name_key`): NFC, sonra Unicode küçük harf; ayrıca Türkçe
/// dört "i" (`I`, `ı`, `İ`, `i`) tek harfe iner ("KARIYER", "KARİYER", "kariyer" aynı
/// etikettir). Aksanlar korunur ("Öfke" ile "Ofke" ayrı etiketlerdir).
pub(crate) fn tag_key(tag: &str) -> String {
    nfc(tag)
        .to_lowercase()
        .chars()
        .filter(|c| *c != '\u{307}')
        .map(|c| if c == 'ı' { 'i' } else { c })
        .collect()
}

/// Etiket listesini NFC'ye çevirir, kırpar, boşları atar, büyük/küçük harf duyarsız
/// (`tag_key`) tekilleştirir (ilk yazım korunur).
pub fn normalize_tags(tags: &[String]) -> Result<Vec<String>, StoreError> {
    let mut out: Vec<String> = Vec::new();
    let mut keys: Vec<String> = Vec::new();
    for t in tags {
        let t = nfc(t.trim());
        let key = tag_key(&t);
        if key.is_empty() {
            continue;
        }
        if t.chars().count() > MAX_TAG_CHARS {
            return Err(StoreError::InvalidInput("tags"));
        }
        if !keys.contains(&key) {
            keys.push(key);
            out.push(t);
        }
    }
    Ok(out)
}

/// Etiketi yoksa oluşturur ve haritaya bağlar. Aynı anahtarlı etiket varsa onun yazımı
/// korunur. `set_chart_tags` ve JSON içe aktarma aynı yolu kullanır.
pub(crate) fn attach_tag(conn: &Connection, chart_id: u32, tag: &str) -> Result<(), StoreError> {
    let key = tag_key(tag);
    conn.execute(
        "INSERT OR IGNORE INTO tags (name, name_key) VALUES (?1, ?2)",
        params![tag, key],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO chart_tags (chart_id, tag_id)
         SELECT ?1, id FROM tags WHERE name_key = ?2",
        params![chart_id, key],
    )?;
    Ok(())
}

impl Store {
    pub fn note(&self, id: u32) -> Result<Note, StoreError> {
        let sql = format!("SELECT {NOTE_COLUMNS} FROM client_notes WHERE id = ?1");
        self.conn
            .query_row(&sql, params![id], row_to_note)
            .optional()?
            .ok_or(StoreError::NotFound { what: "note", id })
    }

    /// Tarih damgalı not. `chart_id` verilirse haritanın danışana ait olması gerekir.
    /// Boş gövde kabul edilir (arayüz notu açar, sonra otomatik kaydeder).
    pub fn add_note(
        &mut self,
        client_id: u32,
        chart_id: Option<u32>,
        body: &str,
    ) -> Result<Note, StoreError> {
        check_body(body)?;
        self.client(client_id)?;
        if let Some(ch) = chart_id {
            if self.chart(ch)?.client_id != client_id {
                return Err(StoreError::InvalidInput("chart_id"));
            }
        }
        let now = self.now();
        self.conn.execute(
            "INSERT INTO client_notes (uuid, client_id, chart_id, body, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            params![Uuid::new_v4().to_string(), client_id, chart_id, body, now],
        )?;
        let id = last_id(&self.conn)?;
        self.note(id)
    }

    /// Otomatik kayıt; `created_at` değişmez.
    pub fn update_note(&mut self, id: u32, body: &str) -> Result<Note, StoreError> {
        check_body(body)?;
        let now = self.now();
        let n = self.conn.execute(
            "UPDATE client_notes SET body = ?1, updated_at = ?2 WHERE id = ?3",
            params![body, now, id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound { what: "note", id });
        }
        self.note(id)
    }

    pub fn delete_note(&mut self, id: u32) -> Result<(), StoreError> {
        if self
            .conn
            .execute("DELETE FROM client_notes WHERE id = ?1", params![id])?
            == 0
        {
            return Err(StoreError::NotFound { what: "note", id });
        }
        Ok(())
    }

    /// Danışanın notları, en yeni önce; en çok `MAX_LIST`.
    pub fn notes(&self, client_id: u32, limit: u32) -> Result<Vec<Note>, StoreError> {
        self.client(client_id)?;
        let sql = format!(
            "SELECT {NOTE_COLUMNS} FROM client_notes WHERE client_id = ?1
             ORDER BY created_at DESC, id DESC LIMIT ?2"
        );
        let mut stmt = self.conn.prepare_cached(&sql)?;
        let rows = stmt.query_map(params![client_id, limit.min(MAX_LIST)], row_to_note)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Danışanın tüm notları, en eski önce. Dışa aktarma içindir; `MAX_LIST` uygulanmaz.
    pub(crate) fn all_notes(&self, client_id: u32) -> Result<Vec<Note>, StoreError> {
        let sql = format!(
            "SELECT {NOTE_COLUMNS} FROM client_notes WHERE client_id = ?1
             ORDER BY created_at, id"
        );
        let mut stmt = self.conn.prepare_cached(&sql)?;
        let rows = stmt.query_map(params![client_id], row_to_note)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Haritanın etiketlerini verilen listeyle değiştirir; sıralı listeyi döndürür.
    /// Var olan etiketin yazımı korunur ("Kariyer" varken "kariyer" yeni etiket açmaz).
    pub fn set_chart_tags(
        &mut self,
        chart_id: u32,
        tags: &[String],
    ) -> Result<Vec<String>, StoreError> {
        let tags = normalize_tags(tags)?;
        self.chart(chart_id)?;
        let now = self.now();
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM chart_tags WHERE chart_id = ?1",
            params![chart_id],
        )?;
        for t in &tags {
            attach_tag(&tx, chart_id, t)?;
        }
        tx.execute(
            "UPDATE charts SET updated_at = ?1 WHERE id = ?2",
            params![now, chart_id],
        )?;
        prune_tags(&tx)?;
        tx.commit()?;
        Ok(self.chart(chart_id)?.tags)
    }

    /// Tüm etiketler ve kaç haritada kullanıldıkları (kenar çubuğu filtresi).
    pub fn tags(&self) -> Result<Vec<TagCount>, StoreError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT t.name, count(ct.chart_id) FROM tags t
             JOIN chart_tags ct ON ct.tag_id = t.id
             GROUP BY t.id ORDER BY t.name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(TagCount {
                name: r.get(0)?,
                charts: r.get(1)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::testutil::new_client;
    use crate::db::testutil::store;

    #[test]
    fn notes_are_dated_ordered_and_scoped() {
        let mut s = store();
        let (a, ch) = s.create_client(&new_client("A")).unwrap();
        let (b, other) = s.create_client(&new_client("B")).unwrap();
        let n1 = s.add_note(a.id, None, "ilk seans").unwrap();
        let n2 = s.add_note(a.id, Some(ch.id), "ikinci").unwrap();
        assert_eq!(n2.chart_id, Some(ch.id));
        assert!(n2.created_at > n1.created_at);
        let up = s.update_note(n1.id, "ilk seans, düzeltildi").unwrap();
        assert_eq!(up.created_at, n1.created_at);
        assert!(up.updated_at > n1.updated_at);
        let ids: Vec<u32> = s.notes(a.id, 10).unwrap().iter().map(|n| n.id).collect();
        assert_eq!(ids, [n2.id, n1.id]);
        assert_eq!(s.notes(a.id, 1).unwrap().len(), 1);
        assert!(s.notes(b.id, 10).unwrap().is_empty());
        assert!(matches!(
            s.add_note(a.id, Some(other.id), "x"),
            Err(StoreError::InvalidInput("chart_id"))
        ));
        s.delete_note(n2.id).unwrap();
        assert!(matches!(
            s.delete_note(n2.id),
            Err(StoreError::NotFound { what: "note", .. })
        ));
    }

    #[test]
    fn a_note_body_over_the_cap_is_invalid_input_and_nothing_is_written() {
        let mut s = store();
        let (a, _) = s.create_client(&new_client("A")).unwrap();
        let at_cap = "ş".repeat(MAX_NOTE_BYTES / 2);
        let n = s.add_note(a.id, None, &at_cap).unwrap();
        let over = format!("{at_cap}x");
        assert!(matches!(
            s.add_note(a.id, None, &over),
            Err(StoreError::InvalidInput("body"))
        ));
        assert!(matches!(
            s.update_note(n.id, &over),
            Err(StoreError::InvalidInput("body"))
        ));
        assert_eq!(s.note(n.id).unwrap().body, at_cap);
        assert_eq!(s.notes(a.id, 10).unwrap().len(), 1);
    }

    #[test]
    fn notes_survive_chart_deletion_and_go_with_client() {
        let mut s = store();
        let (a, ch) = s.create_client(&new_client("A")).unwrap();
        s.add_chart(a.id, &new_client("A").chart).unwrap();
        let n = s.add_note(a.id, Some(ch.id), "bağlı").unwrap();
        s.delete_chart(ch.id).unwrap();
        assert_eq!(s.note(n.id).unwrap().chart_id, None);
        s.delete_client(a.id).unwrap();
        assert!(matches!(s.note(n.id), Err(StoreError::NotFound { .. })));
    }

    #[test]
    fn tags_are_normalized_shared_and_pruned() {
        let mut s = store();
        let (_, a) = s.create_client(&new_client("A")).unwrap();
        let (_, b) = s.create_client(&new_client("B")).unwrap();
        let got = s
            .set_chart_tags(
                a.id,
                &[
                    " Kariyer ".into(),
                    "kariyer".into(),
                    "".into(),
                    "araştırma".into(),
                ],
            )
            .unwrap();
        assert_eq!(got, ["araştırma", "Kariyer"]);
        let got = s.set_chart_tags(b.id, &["KARIYER".into()]).unwrap();
        // Var olan yazım korunur.
        assert_eq!(got, ["Kariyer"]);
        assert_eq!(
            s.tags().unwrap(),
            [
                TagCount {
                    name: "araştırma".into(),
                    charts: 1
                },
                TagCount {
                    name: "Kariyer".into(),
                    charts: 2
                }
            ]
        );
        s.set_chart_tags(a.id, &[]).unwrap();
        assert_eq!(
            s.tags().unwrap(),
            [TagCount {
                name: "Kariyer".into(),
                charts: 1
            }]
        );
        let long = "x".repeat(MAX_TAG_CHARS + 1);
        assert!(matches!(
            s.set_chart_tags(a.id, &[long]),
            Err(StoreError::InvalidInput("tags"))
        ));
    }

    #[test]
    fn tag_identity_folds_non_ascii_case_across_charts() {
        let mut s = store();
        let (_, a) = s.create_client(&new_client("A")).unwrap();
        let (_, b) = s.create_client(&new_client("B")).unwrap();
        let got = s
            .set_chart_tags(
                a.id,
                &[
                    "Öfke".into(),
                    "ÖFKE".into(),
                    "araştırma".into(),
                    "İş".into(),
                ],
            )
            .unwrap();
        assert_eq!(got, ["araştırma", "Öfke", "İş"]);
        // Başka haritada farklı yazım yeni etiket açmaz; ilk yazım korunur.
        let got = s
            .set_chart_tags(
                b.id,
                &["öfke".into(), "ARAŞTIRMA".into(), "IŞ".into(), "iş".into()],
            )
            .unwrap();
        assert_eq!(got, ["araştırma", "Öfke", "İş"]);
        let all = s.tags().unwrap();
        assert_eq!(all.len(), 3);
        assert!(all.iter().all(|t| t.charts == 2));
        // Aksan katlanmaz: "Ofke" ayrı bir etikettir.
        s.set_chart_tags(b.id, &["Ofke".into()]).unwrap();
        assert_eq!(s.tags().unwrap().len(), 4);
        // Yalnızca birleştirici noktadan oluşan etiket boş sayılır.
        assert!(normalize_tags(&["\u{307}".into()]).unwrap().is_empty());
    }

    #[test]
    fn decomposed_and_precomposed_tags_are_one_tag() {
        let mut s = store();
        let (_, a) = s.create_client(&new_client("A")).unwrap();
        let (_, b) = s.create_client(&new_client("B")).unwrap();
        // "O" + U+0308 (ayrışık) ilk yazımdır; saklanan ad NFC'dir.
        let got = s.set_chart_tags(a.id, &["O\u{308}fke".into()]).unwrap();
        assert_eq!(got, ["Öfke"]);
        let got = s.set_chart_tags(b.id, &["öfke".into()]).unwrap();
        assert_eq!(got, ["Öfke"]);
        assert_eq!(
            s.tags().unwrap(),
            [TagCount {
                name: "Öfke".into(),
                charts: 2
            }]
        );
        // Sınır NFC'den sonra sayılır: 64 ayrışık "Ö" 128 kod noktasıdır ama 64 karakterdir.
        let long = "O\u{308}".repeat(MAX_TAG_CHARS);
        assert_eq!(
            normalize_tags(&[long]).unwrap()[0].chars().count(),
            MAX_TAG_CHARS
        );
    }
}
