//! Kayıtlı muhurta süzgeçleri (ileri modüller spec'i §10): `app_state` altında `muhurta_filters`
//! anahtarında JSON. Bir kayıt yalnızca bir ad ve koşullardır (`MuhurtaCriteria`); tarih aralığı,
//! yer ve kişi kaydedilmez. Okuma toleranslıdır: okunamayan kayıt boş liste, okunamayan öğe
//! yok sayılır; eksik koşul alanları varsayılanla dolar.

use jyotish_core::MuhurtaCriteria;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::charts::MAX_NAME_CHARS;
use crate::{Store, StoreError};

pub const MUHURTA_FILTERS_SCHEMA_VERSION: u32 = 1;
/// En çok kayıtlı süzgeç.
pub const MAX_MUHURTA_FILTERS: usize = 32;
const MUHURTA_FILTERS_KEY: &str = "muhurta_filters";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SavedMuhurtaFilter {
    pub name: String,
    pub criteria: MuhurtaCriteria,
}

/// Adların karşılaştırma anahtarı: büyük/küçük harf duyarsız.
fn name_key(name: &str) -> String {
    name.trim().to_lowercase()
}

/// Kanonik biçim: ad kırpılır, koşul kümeleri sıralanır. Geçersizse alan adıyla hata.
fn canonical(filter: &SavedMuhurtaFilter) -> Result<SavedMuhurtaFilter, StoreError> {
    let name = filter.name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
        return Err(StoreError::InvalidInput("muhurta_filters.name"));
    }
    filter
        .criteria
        .validate()
        .map_err(|_| StoreError::InvalidInput("muhurta_filters.criteria"))?;
    Ok(SavedMuhurtaFilter {
        name: name.to_owned(),
        criteria: filter.criteria.clone().normalized(),
    })
}

/// Saklanan bir öğeyi okur: eksik koşul alanları varsayılanla dolar, bilinmeyenler yok sayılır.
fn read_entry(mut entry: Value) -> Option<SavedMuhurtaFilter> {
    let Ok(Value::Object(defaults)) = serde_json::to_value(MuhurtaCriteria::default()) else {
        return None;
    };
    let criteria = entry.get_mut("criteria")?.as_object_mut()?;
    for (key, value) in defaults {
        criteria.entry(key).or_insert(value);
    }
    let filter: SavedMuhurtaFilter = serde_json::from_value(entry).ok()?;
    canonical(&filter).ok()
}

fn stored_version(value: &Value) -> u32 {
    value
        .get("schema_version")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(MUHURTA_FILTERS_SCHEMA_VERSION)
}

impl Store {
    /// Kayıtlı süzgeçler, kaydedildikleri sırayla. Kayıt yoksa, okunamıyorsa ya da daha yeni bir
    /// sürümle yazılmışsa liste boştur.
    pub fn muhurta_filters(&self) -> Result<Vec<SavedMuhurtaFilter>, StoreError> {
        let Some(raw) = self.app_state_get(MUHURTA_FILTERS_KEY)? else {
            return Ok(Vec::new());
        };
        let Ok(value) = serde_json::from_str::<Value>(&raw) else {
            return Ok(Vec::new());
        };
        if stored_version(&value) > MUHURTA_FILTERS_SCHEMA_VERSION {
            return Ok(Vec::new());
        }
        let Some(Value::Array(entries)) = value.get("filters").cloned() else {
            return Ok(Vec::new());
        };
        let mut out: Vec<SavedMuhurtaFilter> = Vec::new();
        for filter in entries.into_iter().filter_map(read_entry) {
            let key = name_key(&filter.name);
            if out.len() < MAX_MUHURTA_FILTERS && out.iter().all(|f| name_key(&f.name) != key) {
                out.push(filter);
            }
        }
        Ok(out)
    }

    /// Listenin tamamını yazar ve saklanan (kanonik) listeyi döner. Adlar boş olamaz ve
    /// büyük/küçük harf ayırmadan tek olmalıdır. Daha yeni sürümlü kaydın üzerine yazmaz.
    pub fn save_muhurta_filters(
        &self,
        filters: &[SavedMuhurtaFilter],
    ) -> Result<Vec<SavedMuhurtaFilter>, StoreError> {
        if filters.len() > MAX_MUHURTA_FILTERS {
            return Err(StoreError::InvalidInput("muhurta_filters"));
        }
        let mut stored: Vec<SavedMuhurtaFilter> = Vec::with_capacity(filters.len());
        for filter in filters {
            let filter = canonical(filter)?;
            let key = name_key(&filter.name);
            if stored.iter().any(|f| name_key(&f.name) == key) {
                return Err(StoreError::InvalidInput("muhurta_filters.name"));
            }
            stored.push(filter);
        }
        if let Some(raw) = self.app_state_get(MUHURTA_FILTERS_KEY)? {
            if let Ok(value) = serde_json::from_str::<Value>(&raw) {
                if stored_version(&value) > MUHURTA_FILTERS_SCHEMA_VERSION {
                    return Err(StoreError::InvalidInput("muhurta_filters"));
                }
            }
        }
        let value = json!({
            "schema_version": MUHURTA_FILTERS_SCHEMA_VERSION,
            "filters": stored,
        });
        self.app_state_set(MUHURTA_FILTERS_KEY, &value.to_string())?;
        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use jyotish_core::{MuhurtaFlag, Sign, Vara};

    use super::*;
    use crate::db::testutil::store;

    fn filter(name: &str) -> SavedMuhurtaFilter {
        SavedMuhurtaFilter {
            name: name.into(),
            criteria: MuhurtaCriteria {
                weekdays: vec![Vara::Thursday, Vara::Monday],
                tithis: vec![11, 2],
                nakshatras: vec![4],
                lagna_signs: vec![Sign::Taurus],
                daytime_only: true,
                min_minutes: 45,
                exclude: vec![MuhurtaFlag::RahuSegment, MuhurtaFlag::Rikta],
                require_tara: true,
                require_chandra: false,
            },
        }
    }

    #[test]
    fn filters_round_trip_in_canonical_form_and_keep_their_order() {
        let s = store();
        assert_eq!(s.muhurta_filters().unwrap(), []);
        let saved = s
            .save_muhurta_filters(&[filter("  Yolculuk "), filter("Açılış")])
            .unwrap();
        assert_eq!(saved.len(), 2);
        assert_eq!(saved[0].name, "Yolculuk");
        assert_eq!(saved[0].criteria.weekdays, [Vara::Monday, Vara::Thursday]);
        assert_eq!(saved[0].criteria.tithis, [2, 11]);
        assert_eq!(
            saved[0].criteria.exclude,
            [MuhurtaFlag::Rikta, MuhurtaFlag::RahuSegment]
        );
        assert_eq!(s.muhurta_filters().unwrap(), saved);
        // Liste bütünüyle değişir; boş liste hepsini siler.
        let one = s.save_muhurta_filters(&[filter("Açılış")]).unwrap();
        assert_eq!(s.muhurta_filters().unwrap(), one);
        assert_eq!(s.save_muhurta_filters(&[]).unwrap(), []);
        assert_eq!(s.muhurta_filters().unwrap(), []);
    }

    #[test]
    fn a_saved_filter_holds_a_name_and_criteria_and_nothing_personal() {
        let s = store();
        s.save_muhurta_filters(&[filter("Yolculuk")]).unwrap();
        let raw = s.app_state_get(MUHURTA_FILTERS_KEY).unwrap().unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["schema_version"], 1);
        let entry = v["filters"][0].as_object().unwrap();
        assert_eq!(entry.keys().collect::<Vec<_>>(), ["criteria", "name"]);
        let mut keys: Vec<&String> = entry["criteria"].as_object().unwrap().keys().collect();
        keys.sort();
        assert_eq!(
            keys,
            [
                "daytime_only",
                "exclude",
                "lagna_signs",
                "min_minutes",
                "nakshatras",
                "require_chandra",
                "require_tara",
                "tithis",
                "weekdays"
            ]
        );
        // Yer, tarih ve kişi alanı yoktur.
        for word in ["lat", "lon", "place", "natal", "jd", "date"] {
            assert!(!raw.contains(&format!("\"{word}")), "{word} in {raw}");
        }
    }

    #[test]
    fn invalid_lists_are_refused_by_field_name() {
        let s = store();
        let bad = |filters: &[SavedMuhurtaFilter], field: &'static str| {
            let e = s.save_muhurta_filters(filters).unwrap_err();
            assert!(
                matches!(e, StoreError::InvalidInput(f) if f == field),
                "{e}"
            );
        };
        bad(&[filter("   ")], "muhurta_filters.name");
        bad(
            &[filter(&"x".repeat(MAX_NAME_CHARS + 1))],
            "muhurta_filters.name",
        );
        // Adlar büyük/küçük harf ayırmadan tek olmalıdır.
        bad(
            &[filter("Yolculuk"), filter("yolculuk ")],
            "muhurta_filters.name",
        );
        let mut tithi = filter("a");
        tithi.criteria.tithis = vec![31];
        bad(&[tithi], "muhurta_filters.criteria");
        let mut nakshatra = filter("a");
        nakshatra.criteria.nakshatras = vec![0];
        bad(&[nakshatra], "muhurta_filters.criteria");
        let many: Vec<_> = (0..=MAX_MUHURTA_FILTERS)
            .map(|i| filter(&format!("f{i}")))
            .collect();
        bad(&many, "muhurta_filters");
        assert_eq!(s.muhurta_filters().unwrap(), []);
        assert_eq!(
            s.save_muhurta_filters(&many[..MAX_MUHURTA_FILTERS])
                .unwrap()
                .len(),
            MAX_MUHURTA_FILTERS
        );
    }

    #[test]
    fn reading_is_tolerant_and_a_newer_record_is_not_overwritten() {
        let s = store();
        for raw in [
            "not json",
            "[1,2]",
            r#"{"filters":7}"#,
            r#"{"schema_version":1}"#,
        ] {
            s.app_state_set(MUHURTA_FILTERS_KEY, raw).unwrap();
            assert_eq!(s.muhurta_filters().unwrap(), [], "{raw}");
        }
        // Eksik koşul alanları varsayılanla dolar; bilinmeyen alanlar yok sayılır; bozuk, adsız
        // ve yinelenen öğeler atılır.
        s.app_state_set(
            MUHURTA_FILTERS_KEY,
            r#"{"schema_version":1,"filters":[
                {"name":"Kısa","criteria":{"tithis":[5,5,2],"future":true}},
                {"name":"kısa","criteria":{}},
                {"name":"","criteria":{}},
                {"name":"Bozuk","criteria":{"tithis":[99]}},
                {"name":"Koşulsuz"},
                7
            ]}"#,
        )
        .unwrap();
        let read = s.muhurta_filters().unwrap();
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].name, "Kısa");
        assert_eq!(
            read[0].criteria,
            MuhurtaCriteria {
                tithis: vec![2, 5],
                ..MuhurtaCriteria::default()
            }
        );
        // Okunamayan eski kaydın üzerine yazılabilir.
        s.app_state_set(MUHURTA_FILTERS_KEY, "[1,2]").unwrap();
        s.save_muhurta_filters(&[filter("Yeni")]).unwrap();
        // Daha yeni sürümün kaydı boş okunur ve üzerine yazılmaz.
        let newer = r#"{"schema_version":2,"filters":[{"name":"Yarın","criteria":{}}]}"#;
        s.app_state_set(MUHURTA_FILTERS_KEY, newer).unwrap();
        assert_eq!(s.muhurta_filters().unwrap(), []);
        assert!(matches!(
            s.save_muhurta_filters(&[filter("Yeni")]),
            Err(StoreError::InvalidInput("muhurta_filters"))
        ));
        assert_eq!(
            s.app_state_get(MUHURTA_FILTERS_KEY).unwrap().as_deref(),
            Some(newer)
        );
    }
}
