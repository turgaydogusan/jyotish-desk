//! Hesap ayarlarının sürüm göçü (spec §7 "Eski `calc_settings_json`") ve genel tercihler
//! (`AppPrefs`, spec §5). Göç başarısız olursa genel varsayılan kullanılır ve uyarı verilir;
//! saklanan JSON üzerine yazılmaz (daha yeni bir sürümün verisi kaybolmasın).

use jyotish_core::{AdvancedSettings, CalcSettings, CALC_SETTINGS_SCHEMA_VERSION};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Store, StoreError};

/// Bir şema adımı: `v` sürümlü JSON'u `v + 1`'e çevirir. `schema_version` alanını
/// çağıran yazar; adım yalnızca alanları dönüştürür.
pub type ValueMigration = fn(Value) -> Result<Value, &'static str>;

/// `CalcSettings` göçleri: indeks `i`, `i + 1 → i + 2` adımıdır. Sürüm 1 → 2 `advanced`
/// alt yapısını varsayılanlarla ekler (ileri modüller spec'i K7).
pub const CALC_SETTINGS_MIGRATIONS: &[ValueMigration] = &[fill_advanced];

/// `advanced` alt yapısını tamamlar: yoksa ya da nesne değilse varsayılanların tamamı, varsa
/// yalnızca eksik anahtarlar yazılır. Var olan değerlere ve bilinmeyen anahtarlara dokunulmaz.
/// Hem sürüm 1 → 2 göç adımıdır hem de sürüm 2 kayıtlarının toleranslı okumasıdır.
fn fill_advanced(mut v: Value) -> Result<Value, &'static str> {
    let obj = v.as_object_mut().ok_or("not an object")?;
    let Ok(Value::Object(defaults)) = serde_json::to_value(AdvancedSettings::default()) else {
        return Err("advanced defaults");
    };
    match obj.get_mut("advanced").and_then(Value::as_object_mut) {
        Some(stored) => {
            for (key, value) in defaults {
                stored.entry(key).or_insert(value);
            }
        }
        None => {
            obj.insert("advanced".into(), Value::Object(defaults));
        }
    }
    Ok(v)
}

fn stored_version(value: &Value) -> Option<u64> {
    value.get("schema_version").and_then(Value::as_u64)
}

/// Ayar okumasının sonucu.
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsLoad {
    Current(CalcSettings),
    /// Eski sürümden göç edildi; çağıran yeni JSON'u saklar.
    Migrated {
        settings: CalcSettings,
        from: u32,
    },
    /// Okunamadı; `settings` genel varsayılanın kopyasıdır. `found_version` okunabildiyse
    /// saklanan sürümdür.
    Reset {
        settings: CalcSettings,
        found_version: Option<u32>,
    },
}

impl SettingsLoad {
    pub fn settings(&self) -> &CalcSettings {
        match self {
            Self::Current(s)
            | Self::Migrated { settings: s, .. }
            | Self::Reset { settings: s, .. } => s,
        }
    }
}

/// Genel sürüm göçü. `Ok((değer, göç_edildiyse_eski_sürüm))`, `Err(okunabildiyse_sürüm)`.
pub(crate) fn migrate_value<T: DeserializeOwned>(
    mut value: Value,
    current: u32,
    migrations: &[ValueMigration],
    check: impl Fn(&T) -> bool,
) -> Result<(T, Option<u32>), Option<u32>> {
    let found = value
        .get("schema_version")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .ok_or(None)?;
    if found == 0 || found > current {
        return Err(Some(found));
    }
    for step in found..current {
        let m = migrations.get((step - 1) as usize).ok_or(Some(found))?;
        value = m(value).map_err(|_| Some(found))?;
        let obj = value.as_object_mut().ok_or(Some(found))?;
        obj.insert("schema_version".into(), Value::from(step + 1));
    }
    match serde_json::from_value::<T>(value) {
        Ok(v) if check(&v) => Ok((v, (found < current).then_some(found))),
        _ => Err(Some(found)),
    }
}

pub(crate) fn load_calc_value(value: Value, fallback: &CalcSettings) -> SettingsLoad {
    // Güncel sürümlü kayıt göç adımından geçmez; eksik `advanced` anahtarları burada dolar.
    // Tamamlanan kayıt `Current` sayılır ve geri yazılmaz (her okumada aynı sonuç).
    let value = if stored_version(&value) == Some(u64::from(CALC_SETTINGS_SCHEMA_VERSION)) {
        fill_advanced(value.clone()).unwrap_or(value)
    } else {
        value
    };
    match migrate_value::<CalcSettings>(
        value,
        CALC_SETTINGS_SCHEMA_VERSION,
        CALC_SETTINGS_MIGRATIONS,
        |s| s.validate().is_ok(),
    ) {
        Ok((settings, None)) => SettingsLoad::Current(settings),
        Ok((settings, Some(from))) => SettingsLoad::Migrated { settings, from },
        Err(found_version) => SettingsLoad::Reset {
            settings: fallback.clone(),
            found_version,
        },
    }
}

/// Saklanan `calc_settings_json`'u okur, gerekirse göç ettirir.
pub fn load_calc_settings(raw: &str, fallback: &CalcSettings) -> SettingsLoad {
    match serde_json::from_str::<Value>(raw) {
        Ok(v) => load_calc_value(v, fallback),
        Err(_) => SettingsLoad::Reset {
            settings: fallback.clone(),
            found_version: None,
        },
    }
}

pub const APP_PREFS_SCHEMA_VERSION: u32 = 1;
/// Arayüz dilleri (spec §1).
pub const LOCALES: [&str; 3] = ["tr", "en", "fr"];
const APP_PREFS_KEY: &str = "app_prefs";

/// Tercih okumasının ölümcül olmayan durumları; arayüz açılış bandında gösterir (spec §7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum PrefsWarning {
    /// Kayıt JSON nesnesi değil; yerleşik varsayılanlar kullanıldı.
    Unreadable,
    /// Genel hesap varsayılanı okunamadı ya da göç edilemedi; yerleşik varsayılan kullanıldı.
    /// Saklanan JSON değiştirilmedi.
    CalcSettingsReset { found_version: Option<u32> },
    /// Tercihler bu sürümden yeni bir uygulamayla yazıldı; yerleşik varsayılanlar kullanıldı ve
    /// kayıt korunur (`set_app_prefs` üzerine yazmaz).
    TooNew { found: u32 },
}

/// `app_prefs_load` sonucu: kullanılacak tercihler ve okuma uyarıları.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PrefsLoad {
    pub prefs: AppPrefs,
    pub warnings: Vec<PrefsWarning>,
}

/// Saklanan kaydın sürümü; alan yoksa ya da sayı değilse güncel sürüm sayılır (Plan 03 kayıtları
/// her zaman damgalıdır; damgasız kayıt elle yazılmıştır).
fn stored_prefs_version(obj: &serde_json::Map<String, Value>) -> u32 {
    obj.get("schema_version")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(APP_PREFS_SCHEMA_VERSION)
}

/// Genel tercihler (spec §5 `AppPrefs`). Varsayılan görünüm, kullanıcının pano düzenidir
/// (`app_state` `panel_layout`, bkz. `view_state`); burada tutulmaz.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AppPrefs {
    pub schema_version: u32,
    pub locale: String,
    /// Yeni haritalara tam kopyası yazılan genel varsayılan (spec §4 "Ayar katmanları").
    pub calc_settings: CalcSettings,
}

impl Default for AppPrefs {
    fn default() -> Self {
        Self {
            schema_version: APP_PREFS_SCHEMA_VERSION,
            locale: "tr".into(),
            calc_settings: CalcSettings::default(),
        }
    }
}

impl Store {
    /// Toleranslı okuma, uyarılarla: kayıt yoksa varsayılanlar; tanınmayan dil "tr"; okunamayan
    /// hesap varsayılanı yerleşik varsayılanla değişir ve `CalcSettingsReset` uyarısı verir;
    /// daha yeni sürümlü kayıtta hiçbir alan okunmaz (`TooNew`).
    pub fn app_prefs_load(&self) -> Result<PrefsLoad, StoreError> {
        let mut out = PrefsLoad {
            prefs: AppPrefs::default(),
            warnings: Vec::new(),
        };
        let Some(raw) = self.app_state_get(APP_PREFS_KEY)? else {
            return Ok(out);
        };
        let Ok(Value::Object(mut obj)) = serde_json::from_str::<Value>(&raw) else {
            out.warnings.push(PrefsWarning::Unreadable);
            return Ok(out);
        };
        let found = stored_prefs_version(&obj);
        if found > APP_PREFS_SCHEMA_VERSION {
            out.warnings.push(PrefsWarning::TooNew { found });
            return Ok(out);
        }
        if let Some(Value::String(l)) = obj.get("locale") {
            if LOCALES.contains(&l.as_str()) {
                out.prefs.locale = l.clone();
            }
        }
        match obj
            .remove("calc_settings")
            .map(|v| load_calc_value(v, &CalcSettings::default()))
        {
            Some(SettingsLoad::Reset { found_version, .. }) => out
                .warnings
                .push(PrefsWarning::CalcSettingsReset { found_version }),
            Some(load) => out.prefs.calc_settings = load.settings().clone(),
            None => {}
        }
        Ok(out)
    }

    /// `app_prefs_load` tercihleri, uyarısız (yeni harita varsayılanı için).
    pub fn app_prefs(&self) -> Result<AppPrefs, StoreError> {
        Ok(self.app_prefs_load()?.prefs)
    }

    /// Doğrular ve güncel sürümle yazar. Daha yeni sürümlü kaydın üzerine yazmaz.
    pub fn set_app_prefs(&self, prefs: &AppPrefs) -> Result<(), StoreError> {
        if !LOCALES.contains(&prefs.locale.as_str()) {
            return Err(StoreError::InvalidInput("locale"));
        }
        prefs.calc_settings.validate()?;
        if let Some(raw) = self.app_state_get(APP_PREFS_KEY)? {
            if let Ok(Value::Object(obj)) = serde_json::from_str::<Value>(&raw) {
                let found = stored_prefs_version(&obj);
                if found > APP_PREFS_SCHEMA_VERSION {
                    return Err(StoreError::PrefsTooNew { found });
                }
            }
        }
        let stored = AppPrefs {
            schema_version: APP_PREFS_SCHEMA_VERSION,
            ..prefs.clone()
        };
        self.app_state_set(APP_PREFS_KEY, &serde_json::to_string(&stored)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::testutil::store;
    use jyotish_core::{AyanamsaSystem, CoreError, KarakaScheme, NodeKind};
    use serde_json::json;

    fn raman() -> CalcSettings {
        CalcSettings {
            ayanamsa: AyanamsaSystem::Raman,
            node: NodeKind::True,
            ..CalcSettings::default()
        }
    }

    #[test]
    fn current_version_loads_as_is() {
        let raw = serde_json::to_string(&raman()).unwrap();
        assert_eq!(
            load_calc_settings(&raw, &CalcSettings::default()),
            SettingsLoad::Current(raman())
        );
    }

    #[test]
    fn unreadable_future_or_invalid_settings_reset_to_fallback() {
        let fb = raman();
        let reset = |raw: &str| load_calc_settings(raw, &fb);
        assert_eq!(
            reset("not json"),
            SettingsLoad::Reset {
                settings: fb.clone(),
                found_version: None
            }
        );
        assert_eq!(
            reset(r#"{"zodiac":"Sidereal"}"#),
            SettingsLoad::Reset {
                settings: fb.clone(),
                found_version: None
            }
        );
        let mut future = serde_json::to_value(CalcSettings::default()).unwrap();
        future["schema_version"] = json!(7);
        assert_eq!(
            reset(&future.to_string()),
            SettingsLoad::Reset {
                settings: fb.clone(),
                found_version: Some(7)
            }
        );
        let mut bad = serde_json::to_value(CalcSettings::default()).unwrap();
        bad["dasha_year"] = json!(1.0);
        assert_eq!(
            reset(&bad.to_string()),
            SettingsLoad::Reset {
                settings: fb.clone(),
                found_version: Some(2)
            }
        );
        let zero = json!({"schema_version": 0});
        assert_eq!(
            reset(&zero.to_string()),
            SettingsLoad::Reset {
                settings: fb,
                found_version: Some(0)
            }
        );
    }

    /// Sürüm 1 kaydı: `advanced` yok (Plan 03–13 böyle yazdı).
    fn v1_json(settings: &CalcSettings) -> Value {
        let mut v = serde_json::to_value(settings).unwrap();
        let obj = v.as_object_mut().unwrap();
        obj.insert("schema_version".into(), json!(1));
        obj.remove("advanced");
        v
    }

    #[test]
    fn version_1_settings_migrate_to_2_with_advanced_defaults() {
        let stored = v1_json(&raman());
        assert_eq!(
            load_calc_settings(&stored.to_string(), &CalcSettings::default()),
            SettingsLoad::Migrated {
                settings: raman(),
                from: 1
            }
        );
        assert_eq!(raman().schema_version, 2);
        assert_eq!(raman().advanced, AdvancedSettings::default());
        // Göç adımı var olan değeri ezmez, yalnızca eksikleri yazar.
        let mut partial = v1_json(&raman());
        partial["advanced"] = json!({"jaimini_co_lords": false});
        let SettingsLoad::Migrated { settings, from: 1 } =
            load_calc_settings(&partial.to_string(), &CalcSettings::default())
        else {
            panic!("expected a migration");
        };
        assert!(!settings.advanced.jaimini_co_lords);
        assert_eq!(settings.advanced.karaka_scheme, KarakaScheme::Eight);
    }

    #[test]
    fn version_2_advanced_block_is_read_tolerantly() {
        let load = |edit: &dyn Fn(&mut Value)| {
            let mut v = serde_json::to_value(raman()).unwrap();
            edit(&mut v);
            load_calc_settings(&v.to_string(), &CalcSettings::default())
        };
        // Bilinmeyen anahtar yok sayılır.
        assert_eq!(
            load(&|v| v["advanced"]["future_option"] = json!([1, 2])),
            SettingsLoad::Current(raman())
        );
        // Eksik blok, nesne olmayan blok ve eksik anahtar varsayılanla dolar.
        for edit in [
            (&|v: &mut Value| {
                v.as_object_mut().unwrap().remove("advanced");
            }) as &dyn Fn(&mut Value),
            &|v| v["advanced"] = json!(null),
            &|v| v["advanced"] = json!({}),
            &|v| v["advanced"] = json!({"karaka_scheme": "Eight"}),
        ] {
            assert_eq!(load(edit), SettingsLoad::Current(raman()));
        }
        // Yazılı değer korunur.
        let seven = CalcSettings {
            advanced: AdvancedSettings {
                karaka_scheme: KarakaScheme::Seven,
                jaimini_co_lords: false,
            },
            ..raman()
        };
        assert_eq!(
            load(&|v| v["advanced"] =
                json!({"karaka_scheme": "Seven", "jaimini_co_lords": false, "x": 1})),
            SettingsLoad::Current(seven)
        );
        // Tanınmayan seçenek değeri tolerans kapsamında değildir: ayar sıfırlanır, uyarı verilir.
        assert_eq!(
            load(&|v| v["advanced"]["karaka_scheme"] = json!("Nine")),
            SettingsLoad::Reset {
                settings: CalcSettings::default(),
                found_version: Some(2)
            }
        );
    }

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct V3 {
        schema_version: u32,
        year_days: f64,
        node: String,
    }

    fn rename_dasha_year(mut v: Value) -> Result<Value, &'static str> {
        let o = v.as_object_mut().ok_or("not an object")?;
        let y = o.remove("dasha_year").ok_or("dasha_year missing")?;
        o.insert("year_days".into(), y);
        Ok(v)
    }

    fn add_node(mut v: Value) -> Result<Value, &'static str> {
        v.as_object_mut()
            .ok_or("not an object")?
            .insert("node".into(), json!("Mean"));
        Ok(v)
    }

    #[test]
    fn migration_chain_steps_and_stamps_versions() {
        let steps: &[ValueMigration] = &[rename_dasha_year, add_node];
        let ok = |_: &V3| true;
        let from1 = json!({"schema_version": 1, "dasha_year": 360.0});
        assert_eq!(
            migrate_value::<V3>(from1, 3, steps, ok),
            Ok((
                V3 {
                    schema_version: 3,
                    year_days: 360.0,
                    node: "Mean".into()
                },
                Some(1)
            ))
        );
        let from2 = json!({"schema_version": 2, "year_days": 365.25});
        assert_eq!(
            migrate_value::<V3>(from2, 3, steps, ok).map(|(v, f)| (v.node, f)),
            Ok(("Mean".to_owned(), Some(2)))
        );
        // Adım hatası ya da eksik adım: okunan sürümle birlikte Err.
        let broken = json!({"schema_version": 1});
        assert_eq!(migrate_value::<V3>(broken, 3, steps, ok), Err(Some(1)));
        let short: &[ValueMigration] = &[rename_dasha_year];
        let from1 = json!({"schema_version": 1, "dasha_year": 360.0});
        assert_eq!(migrate_value::<V3>(from1, 3, short, ok), Err(Some(1)));
        // Denetim reddederse Err.
        let from2 = json!({"schema_version": 2, "year_days": 365.25});
        assert_eq!(
            migrate_value::<V3>(from2, 3, steps, |v: &V3| v.year_days < 300.0),
            Err(Some(2))
        );
    }

    #[test]
    fn app_prefs_default_round_trip_and_tolerance() {
        let s = store();
        assert_eq!(s.app_prefs().unwrap(), AppPrefs::default());
        let p = AppPrefs {
            schema_version: 1,
            locale: "fr".into(),
            calc_settings: raman(),
        };
        s.set_app_prefs(&p).unwrap();
        assert_eq!(s.app_prefs().unwrap(), p);

        s.app_state_set(
            APP_PREFS_KEY,
            r#"{"locale":"de","calc_settings":{"schema_version":9}}"#,
        )
        .unwrap();
        assert_eq!(s.app_prefs().unwrap(), AppPrefs::default());
        s.app_state_set(APP_PREFS_KEY, "[1,2]").unwrap();
        assert_eq!(s.app_prefs().unwrap(), AppPrefs::default());
    }

    #[test]
    fn set_app_prefs_validates() {
        let s = store();
        let bad_locale = AppPrefs {
            locale: "de".into(),
            ..AppPrefs::default()
        };
        assert!(matches!(
            s.set_app_prefs(&bad_locale),
            Err(StoreError::InvalidInput("locale"))
        ));
        let mut bad = AppPrefs::default();
        bad.calc_settings.dasha_year = 10.0;
        assert!(matches!(
            s.set_app_prefs(&bad),
            Err(StoreError::Settings(CoreError::InvalidSettings(
                "dasha_year"
            )))
        ));
        assert_eq!(s.app_prefs().unwrap(), AppPrefs::default());
    }

    #[test]
    fn app_prefs_load_reports_reset_unreadable_and_too_new() {
        let s = store();
        assert_eq!(
            s.app_prefs_load().unwrap(),
            PrefsLoad {
                prefs: AppPrefs::default(),
                warnings: vec![]
            }
        );
        // Okunamayan genel varsayılan: yerleşik varsayılan, dil korunur, uyarı.
        s.app_state_set(
            APP_PREFS_KEY,
            r#"{"schema_version":1,"locale":"en","calc_settings":{"schema_version":9}}"#,
        )
        .unwrap();
        let load = s.app_prefs_load().unwrap();
        assert_eq!(load.prefs.locale, "en");
        assert_eq!(load.prefs.calc_settings, CalcSettings::default());
        assert_eq!(
            load.warnings,
            [PrefsWarning::CalcSettingsReset {
                found_version: Some(9)
            }]
        );
        // Bozuk JSON.
        s.app_state_set(APP_PREFS_KEY, "[1,2]").unwrap();
        assert_eq!(
            s.app_prefs_load().unwrap().warnings,
            [PrefsWarning::Unreadable]
        );
        // Daha yeni uygulamanın kaydı: hiçbir alanı okunmaz.
        s.app_state_set(
            APP_PREFS_KEY,
            r#"{"schema_version":2,"locale":"fr","calc_settings":{}}"#,
        )
        .unwrap();
        let load = s.app_prefs_load().unwrap();
        assert_eq!(load.prefs, AppPrefs::default());
        assert_eq!(load.warnings, [PrefsWarning::TooNew { found: 2 }]);
        assert_eq!(s.app_prefs().unwrap(), AppPrefs::default());
    }

    #[test]
    fn set_app_prefs_refuses_to_overwrite_newer_prefs() {
        let s = store();
        let newer = r#"{"schema_version":2,"locale":"fr","calc_settings":{}}"#;
        s.app_state_set(APP_PREFS_KEY, newer).unwrap();
        assert!(matches!(
            s.set_app_prefs(&AppPrefs::default()),
            Err(StoreError::PrefsTooNew { found: 2 })
        ));
        assert_eq!(
            s.app_state_get(APP_PREFS_KEY).unwrap().as_deref(),
            Some(newer)
        );
        // Okunamayan eski kayıt ise yenisiyle değiştirilebilir.
        s.app_state_set(APP_PREFS_KEY, "[1,2]").unwrap();
        s.set_app_prefs(&AppPrefs::default()).unwrap();
        assert!(s.app_prefs_load().unwrap().warnings.is_empty());
    }
}
