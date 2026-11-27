//! Harita aramasının store yarısı (ileri modüller spec'i §12.1): kapsamdaki haritaların modül
//! girdileri (dilim dilim), toplu etiketleme ve kayıtlı aramalar.
//!
//! Tarama harita kimliği sırasıyla yürür (`id > imleç`): her harita tam bir kez okunur; arama
//! sürerken silinen harita atlanır; eklenen harita, kimliği imleçten büyükse sırası gelince okunur.
//! Hesaplanmış olgu saklanmaz; kayıtlı arama yalnızca koşulları taşır ve dışa aktarılmaz.
//! Danışan adları, harita kimlikleri ve sorgular loglanmaz.

use jyotish_core::{ChartInput, ChartQuery, QueryScope, ScopeKind};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::charts::{clean_name, prune_tags};
use crate::db::last_id;
use crate::model::TimeCertainty;
use crate::notes::{attach_tag, normalize_tags, tag_key};
use crate::settings::load_calc_settings;
use crate::{Store, StoreError};

/// Bir dilimde okunan en çok harita.
pub const SEARCH_PAGE_CHARTS: u32 = 500;
/// Tek bir toplu etiketlemedeki en çok harita.
pub const MAX_TAG_CHARTS: usize = 10_000;
/// En çok kayıtlı arama.
pub const MAX_SAVED_SEARCHES: u32 = 64;

/// Taranan bir harita: sonuç satırının kimliği ve hesabın girdisi. Doğum verisi, yer adı ve not
/// taşımaz.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchRow {
    pub chart_id: u32,
    pub client_id: u32,
    pub client_name: String,
    /// Boşsa arayüz harita türünün yerelleştirilmiş adını gösterir.
    pub chart_label: String,
    /// Haritanın kendi hesap ayarıyla (okunamıyorsa genel varsayılanla) modül girdisi.
    pub input: ChartInput,
}

/// Kayıtlı arama: bir ad ve koşullar. Sonuç listesi ve sayısı saklanmaz.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SavedSearch {
    pub id: u32,
    pub name: String,
    pub query: ChartQuery,
    pub updated_at: String,
}

/// Kapsamın SQL parametreleri: (tür, prashna dışlansın mı, etiket süzgeci var mı, etiket
/// anahtarlarının JSON dizisi).
fn scope_params(
    scope: &QueryScope,
) -> Result<(Option<&'static str>, bool, bool, String), StoreError> {
    scope
        .validate()
        .map_err(|_| StoreError::InvalidInput("search.tags"))?;
    let kind = scope.kind.map(|k| match k {
        ScopeKind::Natal => "Natal",
        ScopeKind::Event => "Event",
    });
    let keys: Vec<String> = scope.tags.iter().map(|t| tag_key(t.trim())).collect();
    Ok((
        kind,
        scope.exclude_prashna,
        !keys.is_empty(),
        serde_json::to_string(&keys)?,
    ))
}

/// Kapsam süzgeci; `ch` haritalar, `p` prashna yan tablosudur (LEFT JOIN).
const SCOPE_WHERE: &str = "(?1 IS NULL OR ch.kind = ?1)
    AND (?2 = 0 OR p.chart_id IS NULL)
    AND (?3 = 0 OR EXISTS (
        SELECT 1 FROM chart_tags ct JOIN tags t ON t.id = ct.tag_id
        WHERE ct.chart_id = ch.id AND t.name_key IN (SELECT value FROM json_each(?4))))";

fn name_key(name: &str) -> String {
    name.to_lowercase()
}

/// Kayıtlı sorgunun listelenen hali: okunamayan ya da artık geçerli olmayan sorgu `None`.
fn listable(raw: &str) -> Option<ChartQuery> {
    serde_json::from_str::<ChartQuery>(raw)
        .ok()
        .filter(|q| q.validate().is_ok())
}

impl Store {
    /// Kapsamdaki harita sayısı (ilerleme göstergesinin paydası; arama sürerken değişebilir).
    pub fn search_scope_count(&self, scope: &QueryScope) -> Result<u32, StoreError> {
        let (kind, no_prashna, by_tag, keys) = scope_params(scope)?;
        let sql = format!(
            "SELECT count(*) FROM charts ch LEFT JOIN prashna p ON p.chart_id = ch.id
             WHERE {SCOPE_WHERE}"
        );
        Ok(self
            .conn
            .prepare_cached(&sql)?
            .query_row(params![kind, no_prashna, by_tag, keys], |r| r.get(0))?)
    }

    /// Kapsamdaki haritaların bir dilimi: kimliği `after_chart_id`'den büyük ilk `limit` harita
    /// (en çok `SEARCH_PAGE_CHARTS`), kimlik sırasıyla. İlk dilim için `after_chart_id` 0'dır.
    /// Sayı prashnasının girdisi sayıyı taşır; doğum saati bilinmeyen haritada `time_known`
    /// yanlıştır.
    pub fn search_rows(
        &self,
        scope: &QueryScope,
        after_chart_id: u32,
        limit: u32,
    ) -> Result<Vec<SearchRow>, StoreError> {
        let (kind, no_prashna, by_tag, keys) = scope_params(scope)?;
        let fallback = self.app_prefs()?.calc_settings;
        let sql = format!(
            "SELECT ch.id, ch.client_id, c.name, ch.label, ch.resolved_jd_ut, ch.lat, ch.lon,
                    ch.time_certainty, ch.calc_settings_json, p.number
             FROM charts ch
             JOIN clients c ON c.id = ch.client_id
             LEFT JOIN prashna p ON p.chart_id = ch.id
             WHERE ch.id > ?5 AND {SCOPE_WHERE}
             ORDER BY ch.id LIMIT ?6"
        );
        let mut stmt = self.conn.prepare_cached(&sql)?;
        let rows = stmt.query_map(
            params![
                kind,
                no_prashna,
                by_tag,
                keys,
                after_chart_id,
                limit.min(SEARCH_PAGE_CHARTS)
            ],
            |r| {
                let certainty: String = r.get(7)?;
                let settings: String = r.get(8)?;
                Ok(SearchRow {
                    chart_id: r.get(0)?,
                    client_id: r.get(1)?,
                    client_name: r.get(2)?,
                    chart_label: r.get(3)?,
                    input: ChartInput {
                        jd_ut: r.get(4)?,
                        lat: r.get(5)?,
                        lon: r.get(6)?,
                        settings: load_calc_settings(&settings, &fallback).settings().clone(),
                        time_known: TimeCertainty::parse(&certainty)
                            != Some(TimeCertainty::Unknown),
                        lagna_number: r.get(9)?,
                    },
                })
            },
        )?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// "Sonuçları etiketle": etiketi verilen haritaların hepsine ekler (öteki etiketler durur),
    /// tek işlemde. Artık var olmayan harita kimlikleri atlanır. Etiketlenen (var olan) harita
    /// sayısını döner. Etiket tek ve geçerli olmalıdır (`tags`); liste boş ya da
    /// `MAX_TAG_CHARTS`'tan uzun olamaz (`chart_ids`).
    pub fn tag_charts(&mut self, chart_ids: &[u32], tag: &str) -> Result<u32, StoreError> {
        let tags = normalize_tags(&[tag.to_owned()])?;
        let [tag] = tags.as_slice() else {
            return Err(StoreError::InvalidInput("tags"));
        };
        if chart_ids.is_empty() || chart_ids.len() > MAX_TAG_CHARTS {
            return Err(StoreError::InvalidInput("chart_ids"));
        }
        let now = self.now();
        let tx = self.conn.transaction()?;
        let mut tagged = 0;
        for id in chart_ids {
            let updated = tx.execute(
                "UPDATE charts SET updated_at = ?1 WHERE id = ?2",
                params![now, id],
            )?;
            if updated == 1 {
                attach_tag(&tx, *id, tag)?;
                tagged += 1;
            }
        }
        // Hiçbir haritaya bağlanmamış etiket (bütün kimlikler silinmişse) durmaz.
        prune_tags(&tx)?;
        tx.commit()?;
        Ok(tagged)
    }

    /// Kayıtlı aramalar, ada göre. Okunamayan ya da artık geçerli olmayan kayıt (ör. katalogdan
    /// çıkmış bir yoga kimliği) listede yer almaz; satırı durur.
    pub fn saved_searches(&self) -> Result<Vec<SavedSearch>, StoreError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, query_json, updated_at FROM saved_searches
             ORDER BY name COLLATE NOCASE, id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, u32>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, name, raw, updated_at) = row?;
            if let Some(query) = listable(&raw) {
                out.push(SavedSearch {
                    id,
                    name,
                    query,
                    updated_at,
                });
            }
        }
        Ok(out)
    }

    /// Aramayı adıyla saklar: aynı adlı (büyük/küçük harf ayırmadan) kayıt varsa koşulları
    /// değişir, yoksa yeni kayıt açılır. Ad boş olamaz (`search.name`); sorgu geçerli olmalıdır
    /// (`search.query`); en çok `MAX_SAVED_SEARCHES` kayıt durur (`search.count`).
    pub fn save_search(
        &mut self,
        name: &str,
        query: &ChartQuery,
    ) -> Result<SavedSearch, StoreError> {
        let name = clean_name(name, "search.name")?;
        if name.is_empty() {
            return Err(StoreError::InvalidInput("search.name"));
        }
        query
            .validate()
            .map_err(|_| StoreError::InvalidInput("search.query"))?;
        let raw = serde_json::to_string(query)?;
        let key = name_key(&name);
        let now = self.now();
        let tx = self.conn.transaction()?;
        let existing: Option<u32> = {
            let mut stmt = tx.prepare_cached("SELECT id, name FROM saved_searches")?;
            let rows = stmt.query_map([], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, String>(1)?)))?;
            let mut found = None;
            for row in rows {
                let (id, other) = row?;
                if name_key(&other) == key {
                    found = Some(id);
                }
            }
            found
        };
        let id = match existing {
            Some(id) => {
                tx.execute(
                    "UPDATE saved_searches SET name = ?1, query_json = ?2, updated_at = ?3
                     WHERE id = ?4",
                    params![name, raw, now, id],
                )?;
                id
            }
            None => {
                // Sınır listelenen kayıtları sayar: okunamayan satır listede görünmez ve
                // silinemez, yer de tutmaz.
                let count = {
                    let mut stmt = tx.prepare_cached("SELECT query_json FROM saved_searches")?;
                    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
                    let mut count = 0u32;
                    for raw in rows {
                        if listable(&raw?).is_some() {
                            count += 1;
                        }
                    }
                    count
                };
                if count >= MAX_SAVED_SEARCHES {
                    return Err(StoreError::InvalidInput("search.count"));
                }
                tx.execute(
                    "INSERT INTO saved_searches (uuid, name, query_json, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?4)",
                    params![Uuid::new_v4().to_string(), name, raw, now],
                )?;
                last_id(&tx)?
            }
        };
        tx.commit()?;
        Ok(SavedSearch {
            id,
            name,
            query: query.clone(),
            updated_at: now,
        })
    }

    /// Kayıtlı aramayı siler; yoksa `NotFound`.
    pub fn delete_saved_search(&mut self, id: u32) -> Result<(), StoreError> {
        let found: Option<u32> = self
            .conn
            .query_row(
                "SELECT id FROM saved_searches WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
            .optional()?;
        if found.is_none() {
            return Err(StoreError::NotFound {
                what: "saved_search",
                id,
            });
        }
        self.conn
            .execute("DELETE FROM saved_searches WHERE id = ?1", params![id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use jyotish_core::{
        AyanamsaSystem, CalcSettings, Condition, Graha, QueryCondition, Sign, Varga,
    };

    use super::*;
    use crate::birth::fixtures::{delhi, istanbul};
    use crate::charts::testutil::new_client;
    use crate::db::testutil::store;
    use crate::exchange::ExportOptions;
    use crate::model::{BirthData, ChartKind, NewChart, NewPrashna};
    use crate::settings::AppPrefs;

    fn all() -> QueryScope {
        QueryScope {
            tags: Vec::new(),
            kind: None,
            exclude_prashna: false,
        }
    }

    fn query() -> ChartQuery {
        ChartQuery {
            all_of: vec![QueryCondition {
                cond: Condition::GrahaInSign {
                    graha: Graha::Mars,
                    signs: vec![Sign::Aries],
                    varga: Varga::D1,
                },
                negate: false,
            }],
            scope: QueryScope::default(),
        }
    }

    fn event(label: &str, date: &str) -> NewChart {
        NewChart {
            label: label.into(),
            kind: ChartKind::Event,
            birth: istanbul(date, "14:00"),
        }
    }

    /// Bütün dilimleri yürütür; (harita kimlikleri, dilim sayısı).
    fn scan(s: &Store, scope: &QueryScope, limit: u32) -> (Vec<u32>, u32) {
        let (mut ids, mut pages, mut after) = (Vec::new(), 0, 0);
        loop {
            let rows = s.search_rows(scope, after, limit).unwrap();
            pages += 1;
            let Some(last) = rows.last() else {
                return (ids, pages);
            };
            after = last.chart_id;
            ids.extend(rows.iter().map(|r| r.chart_id));
        }
    }

    #[test]
    fn paging_reads_every_chart_exactly_once() {
        let mut s = store();
        let mut want = Vec::new();
        for i in 0..40 {
            let (c, ch) = s
                .create_client(&new_client(&format!("Danışan {i}")))
                .unwrap();
            want.push(ch.id);
            for j in 0..24 {
                let extra = s
                    .add_chart(c.id, &event(&format!("Olay {j}"), "2016-06-01"))
                    .unwrap();
                want.push(extra.id);
            }
        }
        assert_eq!(want.len(), 1_000);
        assert_eq!(s.search_scope_count(&all()).unwrap(), 1_000);
        // 500'lük dilimler: iki dolu dilim ve boş bir son dilim.
        let (ids, pages) = scan(&s, &all(), SEARCH_PAGE_CHARTS);
        assert_eq!(pages, 3);
        want.sort_unstable();
        assert_eq!(ids, want);
        // Daha küçük dilimler aynı kümeyi verir; istenen sınır 500'ü aşamaz.
        assert_eq!(scan(&s, &all(), 77).0, want);
        assert_eq!(s.search_rows(&all(), 0, 9_999).unwrap().len(), 500);
    }

    #[test]
    fn a_chart_deleted_during_the_scan_is_skipped_without_an_error() {
        let mut s = store();
        let (c, first) = s.create_client(&new_client("Ayşe")).unwrap();
        let second = s.add_chart(c.id, &event("İkinci", "2016-06-01")).unwrap();
        let third = s.add_chart(c.id, &event("Üçüncü", "2017-06-01")).unwrap();
        let page = s.search_rows(&all(), 0, 1).unwrap();
        assert_eq!(page[0].chart_id, first.id);
        // İlk dilimden sonra sıradaki harita silinir, yenisi eklenir.
        s.delete_chart(second.id).unwrap();
        let fourth = s.add_chart(c.id, &event("Dördüncü", "2018-06-01")).unwrap();
        let rest: Vec<u32> = s
            .search_rows(&all(), first.id, 500)
            .unwrap()
            .iter()
            .map(|r| r.chart_id)
            .collect();
        assert_eq!(rest, [third.id, fourth.id]);
    }

    #[test]
    fn rows_carry_the_names_and_the_charts_own_input() {
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
        let (c, natal) = s.create_client(&new_client("Ayşe Yılmaz")).unwrap();
        // Genel varsayılan sonradan değişir: harita kendi ayarıyla aranır.
        s.set_app_prefs(&AppPrefs::default()).unwrap();
        let unknown = s
            .add_chart(
                c.id,
                &NewChart {
                    label: "Saati yok".into(),
                    kind: ChartKind::Natal,
                    birth: BirthData {
                        time: None,
                        time_certainty: TimeCertainty::Unknown,
                        ..delhi()
                    },
                },
            )
            .unwrap();
        let (_, number) = s
            .create_prashna(
                &NewPrashna {
                    client_id: Some(c.id),
                    birth: delhi(),
                    number: Some(54),
                    question: "deneme sorusu".into(),
                    focus_house: None,
                },
                "Prashna 2000-01-01 17:30",
            )
            .unwrap();
        // Bozuk ayar kaydı genel varsayılana düşer; arama durmaz.
        s.conn
            .execute(
                "UPDATE charts SET calc_settings_json = 'bozuk' WHERE id = ?1",
                params![number.id],
            )
            .unwrap();
        let rows = s.search_rows(&all(), 0, 500).unwrap();
        assert_eq!(rows.len(), 3);
        let row = |id: u32| rows.iter().find(|r| r.chart_id == id).unwrap();
        let r = row(natal.id);
        assert_eq!((r.client_id, r.client_name.as_str()), (c.id, "Ayşe Yılmaz"));
        assert_eq!(r.chart_label, "");
        assert_eq!(r.input.jd_ut, 2_451_545.0);
        assert_eq!(
            (r.input.lat, r.input.lon),
            (natal.birth.place.lat, natal.birth.place.lon)
        );
        assert_eq!(r.input.settings, raman);
        assert!(r.input.time_known);
        assert_eq!(r.input.lagna_number, None);
        let r = row(unknown.id);
        assert_eq!(r.chart_label, "Saati yok");
        assert!(!r.input.time_known);
        let r = row(number.id);
        assert_eq!(r.input.lagna_number, Some(54));
        assert_eq!(r.input.settings, CalcSettings::default());
        // Satır soru metnini ve yer adını taşımaz.
        let debug = format!("{rows:?}");
        assert!(!debug.contains("deneme sorusu") && !debug.contains("New Delhi"));
    }

    #[test]
    fn the_scope_filters_by_kind_prashna_and_tags() {
        let mut s = store();
        let (c, natal) = s.create_client(&new_client("Ayşe")).unwrap();
        let wedding = s.add_chart(c.id, &event("Evlilik", "2016-06-01")).unwrap();
        let (_, prashna) = s
            .create_prashna(
                &NewPrashna {
                    client_id: None,
                    birth: delhi(),
                    number: None,
                    question: String::new(),
                    focus_house: None,
                },
                "Prashna",
            )
            .unwrap();
        s.set_chart_tags(natal.id, &["Kariyer".into(), "Araştırma".into()])
            .unwrap();
        s.set_chart_tags(wedding.id, &["Evlilik".into()]).unwrap();
        let ids = |scope: &QueryScope| scan(&s, scope, 500).0;
        assert_eq!(ids(&all()), [natal.id, wedding.id, prashna.id]);
        // Varsayılan kapsam prashna haritalarını dışlar.
        assert_eq!(ids(&QueryScope::default()), [natal.id, wedding.id]);
        assert_eq!(s.search_scope_count(&QueryScope::default()).unwrap(), 2);
        let kind = |kind| QueryScope {
            kind: Some(kind),
            ..all()
        };
        assert_eq!(ids(&kind(ScopeKind::Natal)), [natal.id]);
        // Prashna haritası `Event` türündedir.
        assert_eq!(ids(&kind(ScopeKind::Event)), [wedding.id, prashna.id]);
        // Etiketler "ya da"dır ve etiket anahtarıyla eşleşir (büyük/küçük harf, Türkçe i).
        let tags = |tags: &[&str]| QueryScope {
            tags: tags.iter().map(|t| (*t).to_owned()).collect(),
            ..all()
        };
        assert_eq!(ids(&tags(&["KARİYER"])), [natal.id]);
        assert_eq!(
            ids(&tags(&[" evlilik ", "araştırma"])),
            [natal.id, wedding.id]
        );
        assert_eq!(ids(&tags(&["yok"])), Vec::<u32>::new());
        assert_eq!(s.search_scope_count(&tags(&["evlilik"])).unwrap(), 1);
        // Geçersiz kapsam alan adıyla reddedilir.
        let err = s.search_rows(&tags(&[" "]), 0, 500).unwrap_err();
        assert!(matches!(err, StoreError::InvalidInput("search.tags")));
    }

    #[test]
    fn tagging_results_adds_one_tag_and_keeps_the_others() {
        let mut s = store();
        let (c, a) = s.create_client(&new_client("Ayşe")).unwrap();
        let b = s.add_chart(c.id, &event("Evlilik", "2016-06-01")).unwrap();
        let untouched = s.add_chart(c.id, &event("Taşınma", "2018-06-01")).unwrap();
        s.set_chart_tags(a.id, &["Kariyer".into()]).unwrap();
        // Var olmayan kimlik atlanır.
        let tagged = s.tag_charts(&[a.id, b.id, 9_999], " Mars Koç ").unwrap();
        assert_eq!(tagged, 2);
        assert_eq!(s.chart(a.id).unwrap().tags, ["Kariyer", "Mars Koç"]);
        assert_eq!(s.chart(b.id).unwrap().tags, ["Mars Koç"]);
        assert!(s.chart(untouched.id).unwrap().tags.is_empty());
        // Aynı etiket yeniden eklenince çoğalmaz; var olan yazım korunur.
        assert_eq!(s.tag_charts(&[a.id], "mars koç").unwrap(), 1);
        assert_eq!(s.chart(a.id).unwrap().tags, ["Kariyer", "Mars Koç"]);
        // Hiçbir harita yoksa etiket oluşmaz.
        assert_eq!(s.tag_charts(&[9_998], "Boşta").unwrap(), 0);
        assert!(s.tags().unwrap().iter().all(|t| t.name != "Boşta"));
        // Girdi hataları alan adıyla.
        for (ids, tag, field) in [
            (vec![a.id], "  ", "tags"),
            (vec![a.id], "bir, iki", "tags"),
            (Vec::new(), "x", "chart_ids"),
            (vec![1; MAX_TAG_CHARTS + 1], "x", "chart_ids"),
        ] {
            match s.tag_charts(&ids, tag) {
                Err(StoreError::InvalidInput(f)) => assert_eq!(f, field),
                other => panic!("expected {field}, got {other:?}"),
            }
        }
    }

    #[test]
    fn saved_searches_keep_only_a_name_and_the_conditions() {
        let mut s = store();
        assert!(s.saved_searches().unwrap().is_empty());
        let first = s.save_search("  Mars Koç ", &query()).unwrap();
        assert_eq!(first.name, "Mars Koç");
        assert_eq!(first.query, query());
        let mut other = query();
        other.all_of[0].negate = true;
        let second = s.save_search("Ay Yengeç", &other).unwrap();
        let names: Vec<String> = s
            .saved_searches()
            .unwrap()
            .into_iter()
            .map(|x| x.name)
            .collect();
        assert_eq!(names, ["Ay Yengeç", "Mars Koç"]);
        // Aynı ad (büyük/küçük harf ayırmadan) kaydın koşullarını değiştirir; kimlik durur.
        let again = s.save_search("MARS KOÇ", &other).unwrap();
        assert_eq!(again.id, first.id);
        let list = s.saved_searches().unwrap();
        assert_eq!(list.len(), 2);
        let saved = list.iter().find(|x| x.id == first.id).unwrap();
        assert_eq!((saved.name.as_str(), &saved.query), ("MARS KOÇ", &other));
        assert!(saved.updated_at > first.updated_at);
        // Saklanan JSON yalnızca sorgudur: sonuç, sayı ya da harita kimliği yoktur.
        let raw: String = s
            .conn
            .query_row(
                "SELECT query_json FROM saved_searches WHERE id = ?1",
                params![second.id],
                |r| r.get(0),
            )
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let mut keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(keys, ["all_of", "scope"]);
        // Silme ve bulunamayan kayıt.
        s.delete_saved_search(first.id).unwrap();
        assert!(matches!(
            s.delete_saved_search(first.id),
            Err(StoreError::NotFound {
                what: "saved_search",
                ..
            })
        ));
        assert_eq!(s.saved_searches().unwrap().len(), 1);
    }

    #[test]
    fn saved_searches_reject_bad_input_and_survive_bad_rows() {
        let mut s = store();
        let field = |r: Result<SavedSearch, StoreError>| match r {
            Err(StoreError::InvalidInput(f)) => f,
            other => panic!("expected an input error, got {other:?}"),
        };
        assert_eq!(field(s.save_search("   ", &query())), "search.name");
        assert_eq!(
            field(s.save_search(&"x".repeat(201), &query())),
            "search.name"
        );
        let empty = ChartQuery {
            all_of: Vec::new(),
            scope: QueryScope::default(),
        };
        assert_eq!(field(s.save_search("Boş", &empty)), "search.query");
        for i in 0..MAX_SAVED_SEARCHES {
            s.save_search(&format!("Arama {i}"), &query()).unwrap();
        }
        assert_eq!(field(s.save_search("Fazla", &query())), "search.count");
        // Dolu listede var olan ad yine güncellenir.
        s.save_search("arama 0", &query()).unwrap();
        // Okunamayan ve artık geçersiz satırlar listelenmez; öteki kayıtlar durur.
        s.conn
            .execute_batch(
                "UPDATE saved_searches SET query_json = 'bozuk' WHERE name = 'Arama 1';
                 UPDATE saved_searches SET query_json = '{\"all_of\":[],\"scope\":{\"tags\":[],\"kind\":null,\"exclude_prashna\":true}}'
                 WHERE name = 'Arama 2';",
            )
            .unwrap();
        let list = s.saved_searches().unwrap();
        assert_eq!(list.len() as u32, MAX_SAVED_SEARCHES - 2);
        assert!(list
            .iter()
            .all(|x| x.name != "Arama 1" && x.name != "Arama 2"));
        // Listelenmeyen satır yer tutmaz: sınır görünen kayıtları sayar.
        s.save_search("Yeni 1", &query()).unwrap();
        s.save_search("Yeni 2", &query()).unwrap();
        assert_eq!(field(s.save_search("Fazla", &query())), "search.count");
        assert_eq!(s.saved_searches().unwrap().len() as u32, MAX_SAVED_SEARCHES);
    }

    #[test]
    fn saved_searches_are_not_exported() {
        let mut s = store();
        s.create_client(&new_client("Ayşe")).unwrap();
        s.save_search("Gizli arama adı", &query()).unwrap();
        let json = s
            .export_json(&ExportOptions {
                include_notes: true,
                client_ids: None,
            })
            .unwrap();
        assert!(!json.contains("Gizli arama adı"));
        assert!(!json.contains("saved_search") && !json.contains("all_of"));
        let csv = s.export_csv().unwrap();
        assert!(!csv.contains("Gizli arama adı") && !csv.contains("all_of"));
    }
}
