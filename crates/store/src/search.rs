//! `Cmd+K` danışan araması (spec §4): isim (FTS5), etiket (önek) ve not (FTS5). Sorgu
//! dili atlasla aynıdır: harf/rakam dışı her karakter ayırıcı, her kelime önek, AND.

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::charts::{row_to_client, MAX_LIST};
use crate::model::Client;
use crate::notes::tag_key;
use crate::{Store, StoreError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum SearchMatch {
    Name,
    Tag,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SearchHit {
    pub client: Client,
    pub matched: SearchMatch,
    /// Not eşleşmesinde notun eşleşen bölümü (işaretsiz düz metin); diğerlerinde boş.
    pub snippet: Option<String>,
}

/// `LIKE` önek kalıbı: `%`, `_` ve `\` kaçışlanır.
fn like_prefix(q: &str) -> String {
    let mut out = String::with_capacity(q.len() + 1);
    for c in q.chars() {
        if matches!(c, '%' | '_' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('%');
    out
}

impl Store {
    /// Danışan başına tek sonuç, öncelik isim → etiket → not; her grupta son açılan önce
    /// (not grubu dahil; alaka yalnızca danışanın hangi notunun gösterileceğini seçer).
    /// `limit` danışan sayısıdır ve her grup en çok `limit` ayrı danışan getirir.
    /// Boş ya da yalnızca ayırıcıdan oluşan sorgu boş liste döner (arayüz son açılanları
    /// `list_clients` ile gösterir).
    pub fn search_clients(&self, query: &str, limit: u32) -> Result<Vec<SearchHit>, StoreError> {
        let Some(fts) = atlas::db::fts_query(query) else {
            return Ok(Vec::new());
        };
        let limit = limit.min(MAX_LIST);
        let mut hits: Vec<SearchHit> = Vec::new();
        let push = |hits: &mut Vec<SearchHit>, client: Client, matched, snippet| {
            if hits.len() < limit as usize && !hits.iter().any(|h| h.client.id == client.id) {
                hits.push(SearchHit {
                    client,
                    matched,
                    snippet,
                });
            }
        };

        let mut stmt = self.conn.prepare_cached(
            "SELECT c.id, c.uuid, c.name, c.gender, c.last_chart_id, c.last_opened_at,
                    c.created_at, c.updated_at
             FROM clients_fts JOIN clients c ON c.id = clients_fts.rowid
             WHERE clients_fts MATCH ?1
             ORDER BY c.last_opened_at DESC NULLS LAST, c.name COLLATE NOCASE, c.id
             LIMIT ?2",
        )?;
        for c in stmt.query_map(params![fts, limit], row_to_client)? {
            push(&mut hits, c?, SearchMatch::Name, None);
        }

        let mut stmt = self.conn.prepare_cached(
            "SELECT DISTINCT c.id, c.uuid, c.name, c.gender, c.last_chart_id, c.last_opened_at,
                    c.created_at, c.updated_at
             FROM tags t
             JOIN chart_tags ct ON ct.tag_id = t.id
             JOIN charts ch ON ch.id = ct.chart_id
             JOIN clients c ON c.id = ch.client_id
             WHERE t.name_key LIKE ?1 ESCAPE '\\'
             ORDER BY c.last_opened_at DESC NULLS LAST, c.name COLLATE NOCASE, c.id
             LIMIT ?2",
        )?;
        for c in stmt.query_map(
            params![like_prefix(&tag_key(query.trim())), limit],
            row_to_client,
        )? {
            push(&mut hits, c?, SearchMatch::Tag, None);
        }

        // Danışan başına tek satır: en alakalı notun parçası (eşitlikte en yeni not). `LIMIT`
        // danışan sayar; yoksa tek danışanın çok sayıda notu öbür danışanları dışarıda bırakırdı.
        // `snippet` ve `rank` yalnızca FTS taramasının kendi satırında hesaplanabilir; bu yüzden
        // eşleşmeler önce somutlaştırılır, pencere işlevi onun üzerinde çalışır.
        let mut stmt = self.conn.prepare_cached(
            "WITH matched AS MATERIALIZED (
                 SELECT n.client_id AS client_id, n.id AS note_id,
                        snippet(notes_fts, 0, '', '', '…', 10) AS snip,
                        notes_fts.rank AS score
                 FROM notes_fts JOIN client_notes n ON n.id = notes_fts.rowid
                 WHERE notes_fts MATCH ?1
             ), best AS (
                 SELECT client_id, snip,
                        row_number() OVER (
                            PARTITION BY client_id ORDER BY score, note_id DESC
                        ) AS pos
                 FROM matched
             )
             SELECT c.id, c.uuid, c.name, c.gender, c.last_chart_id, c.last_opened_at,
                    c.created_at, c.updated_at, best.snip
             FROM best JOIN clients c ON c.id = best.client_id
             WHERE best.pos = 1
             ORDER BY c.last_opened_at DESC NULLS LAST, c.name COLLATE NOCASE, c.id
             LIMIT ?2",
        )?;
        let notes = stmt.query_map(params![fts, limit], |r| {
            Ok((row_to_client(r)?, r.get::<_, String>(8)?))
        })?;
        for row in notes {
            let (client, snippet) = row?;
            push(&mut hits, client, SearchMatch::Note, Some(snippet));
        }
        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::testutil::new_client;
    use crate::db::testutil::store;

    #[test]
    fn name_search_is_prefix_and_diacritic_insensitive() {
        let mut s = store();
        let (ayse, _) = s.create_client(&new_client("Ayşe Çelik")).unwrap();
        s.create_client(&new_client("Mehmet Öz")).unwrap();
        for q in ["ayse", "AYŞE", "cel", "Ayşe Çe", "  ay  "] {
            let hits = s.search_clients(q, 10).unwrap();
            assert_eq!(hits.len(), 1, "{q}");
            assert_eq!(hits[0].client.id, ayse.id);
            assert_eq!(hits[0].matched, SearchMatch::Name);
        }
        assert!(s.search_clients("xyz", 10).unwrap().is_empty());
        assert!(s.search_clients(" *\"( ", 10).unwrap().is_empty());
    }

    #[test]
    fn renamed_and_deleted_clients_update_the_index() {
        let mut s = store();
        let (c, _) = s.create_client(&new_client("Zeynep")).unwrap();
        s.update_client(c.id, "Yasemin", None).unwrap();
        assert!(s.search_clients("zeynep", 10).unwrap().is_empty());
        assert_eq!(s.search_clients("yasemin", 10).unwrap().len(), 1);
        let n = s.add_note(c.id, None, "Satürn dönüşü konuşuldu").unwrap();
        assert_eq!(s.search_clients("saturn", 10).unwrap().len(), 1);
        s.update_note(n.id, "Jüpiter").unwrap();
        assert!(s.search_clients("saturn", 10).unwrap().is_empty());
        s.delete_client(c.id).unwrap();
        assert!(s.search_clients("yasemin", 10).unwrap().is_empty());
        assert!(s.search_clients("jupiter", 10).unwrap().is_empty());
    }

    #[test]
    fn tag_and_note_hits_rank_after_names_one_per_client() {
        let mut s = store();
        let (kara, _) = s.create_client(&new_client("Kartal")).unwrap();
        let (b, bch) = s.create_client(&new_client("Bora")).unwrap();
        let (c, _) = s.create_client(&new_client("Cem")).unwrap();
        s.set_chart_tags(bch.id, &["kartal_100%".into()]).unwrap();
        s.add_note(c.id, None, "Kartal burcu değil; kartal gibi bakış")
            .unwrap();
        s.add_note(kara.id, None, "kartal").unwrap();
        let hits = s.search_clients("kart", 10).unwrap();
        let got: Vec<(u32, SearchMatch)> = hits.iter().map(|h| (h.client.id, h.matched)).collect();
        assert_eq!(
            got,
            [
                (kara.id, SearchMatch::Name),
                (b.id, SearchMatch::Tag),
                (c.id, SearchMatch::Note)
            ]
        );
        assert!(hits[2].snippet.as_deref().unwrap().contains("Kartal"));
        assert_eq!(s.search_clients("kart", 2).unwrap().len(), 2);
        // LIKE kaçışı: '%' joker değildir.
        assert!(s
            .search_clients("k%", 10)
            .unwrap()
            .iter()
            .all(|h| h.matched != SearchMatch::Tag));
        // Etiket öneki ASCII dışı harflerde de büyük/küçük harf duyarsızdır.
        s.set_chart_tags(bch.id, &["Öfke".into(), "ARAŞTIRMA".into()])
            .unwrap();
        for q in ["ÖFK", "öf", "araştır"] {
            let hits = s.search_clients(q, 10).unwrap();
            let got: Vec<(u32, SearchMatch)> =
                hits.iter().map(|h| (h.client.id, h.matched)).collect();
            assert_eq!(got, [(b.id, SearchMatch::Tag)], "{q}");
        }
    }

    #[test]
    fn note_hits_are_one_per_client_and_follow_last_opened() {
        let mut s = store();
        let (kartal, _) = s.create_client(&new_client("Kartal")).unwrap();
        let (bora, _) = s.create_client(&new_client("Bora")).unwrap();
        let (cem, _) = s.create_client(&new_client("Cem")).unwrap();
        let (deniz, _) = s.create_client(&new_client("Deniz")).unwrap();
        // İsimle bulunan danışanın çok sayıda eşleşen notu öbürlerinin yerini tutmamalı.
        for _ in 0..4 {
            s.add_note(kartal.id, None, "kartal").unwrap();
        }
        s.add_note(bora.id, None, "Seansta kartal burcu üzerine konuşuldu")
            .unwrap();
        s.add_note(
            cem.id,
            None,
            "Uzun bir seans notu: önce aile, sonra iş, en sonda kartal konusu açıldı",
        )
        .unwrap();
        s.add_note(cem.id, None, "kartal kartal").unwrap();
        s.add_note(cem.id, None, "ilgisiz not").unwrap();
        s.add_note(
            deniz.id,
            None,
            "Bu notta aranan sözcük yalnızca bir kez ve sonda geçer: kartal",
        )
        .unwrap();
        for (id, ts) in [
            (bora.id, "2026-01-01T00:00:00.000Z"),
            (deniz.id, "2026-02-01T00:00:00.000Z"),
        ] {
            s.conn
                .execute(
                    "UPDATE clients SET last_opened_at = ?2 WHERE id = ?1",
                    params![id, ts],
                )
                .unwrap();
        }
        let ids = |hits: &[SearchHit]| -> Vec<(u32, SearchMatch)> {
            hits.iter().map(|h| (h.client.id, h.matched)).collect()
        };
        // Not grubu da son açılana göre sıralanır (alaka sırası değil); hiç açılmamış sonda.
        let all = s.search_clients("kartal", 10).unwrap();
        assert_eq!(
            ids(&all),
            [
                (kartal.id, SearchMatch::Name),
                (deniz.id, SearchMatch::Note),
                (bora.id, SearchMatch::Note),
                (cem.id, SearchMatch::Note)
            ]
        );
        // Danışanın birden çok notu eşleşirse parça en alakalı nottan gelir.
        assert_eq!(all[3].snippet.as_deref(), Some("kartal kartal"));
        assert_eq!(all[0].snippet, None);
        // Sınır danışan sayısıdır, not satırı sayısı değil.
        let top = s.search_clients("kartal", 3).unwrap();
        assert_eq!(ids(&top), ids(&all[..3]));
        assert_eq!(ids(&s.search_clients("kartal", 2).unwrap()), ids(&all[..2]));
    }

    #[test]
    fn query_syntax_is_inert_and_limit_is_bounded() {
        let mut s = store();
        let (ayse, _) = s.create_client(&new_client("Ayşe Çelik")).unwrap();
        let (near, _) = s.create_client(&new_client("Near Ali Bora")).unwrap();
        let ids = |q: &str| -> Vec<u32> {
            s.search_clients(q, 10)
                .unwrap()
                .iter()
                .map(|h| h.client.id)
                .collect()
        };
        // FTS5 işleçleri ve tırnaklar ayırıcıdır: önek, olumsuzlama, sütun süzgeci yok.
        for q in [
            "ayse*", "-ayse", "^ayse", "\"ayse", "ayse\"", "(ayse)", "ayse:",
        ] {
            assert_eq!(ids(q), [ayse.id], "{q}");
        }
        // İşleç sözcükleri sıradan kelimedir (hepsi AND ile aranır).
        for q in ["name:ayse", "ayse OR xyz", "ayse AND", "ayse NOT xyz"] {
            assert!(ids(q).is_empty(), "{q}");
        }
        for q in ["NEAR(a b)", "a\"b", "near + bora"] {
            assert_eq!(ids(q), [near.id], "{q}");
        }
        for q in ["", "\"\"", "\"", "*", "-", "()", ":"] {
            assert!(ids(q).is_empty(), "{q}");
        }
        assert!(s.search_clients("a", 0).unwrap().is_empty());
        assert_eq!(s.search_clients("a", 1).unwrap().len(), 1);
        assert_eq!(s.search_clients("a", u32::MAX).unwrap().len(), 2);
    }
}
