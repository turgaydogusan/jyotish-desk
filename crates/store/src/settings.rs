//! Hesap ayarlarının sürüm göçü (spec §7 "Eski `calc_settings_json`") ve genel tercihler
//! (`AppPrefs`, spec §5). Göç başarısız olursa genel varsayılan kullanılır ve uyarı verilir;
//! saklanan JSON üzerine yazılmaz (daha yeni bir sürümün verisi kaybolmasın).

use jyotish_core::{CalcSettings, CALC_SETTINGS_SCHEMA_VERSION};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Store, StoreError};

/// Bir şema adımı: `v` sürümlü JSON'u `v + 1`'e çevirir. `schema_version` alanını
/// çağıran yazar; adım yalnızca alanları dönüştürür.
pub type ValueMigration = fn(Value) -> Result<Value, &'static str>;

/// `CalcSettings` göçleri: indeks `i`, `i + 1 → i + 2` adımıdır. v1 tek sürümdür; liste
/// boştur. v2 geldiğinde `CALC_SETTINGS_SCHEMA_VERSION` 2 olur ve buraya bir adım eklenir.
pub const CALC_SETTINGS_MIGRATIONS: &[ValueMigration] = &[];

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
    /// Toleranslı okuma: kayıt yoksa ya da bozuksa varsayılanlar; tanınmayan dil "tr";
    /// okunamayan ayar spec §6 varsayılanı.
    pub fn app_prefs(&self) -> Result<AppPrefs, StoreError> {
        let mut prefs = AppPrefs::default();
        let Some(raw) = self.app_state_get(APP_PREFS_KEY)? else {
            return Ok(prefs);
        };
        let Ok(Value::Object(mut obj)) = serde_json::from_str::<Value>(&raw) else {
            return Ok(prefs);
        };
        if let Some(Value::String(l)) = obj.get("locale") {
            if LOCALES.contains(&l.as_str()) {
                prefs.locale = l.clone();
            }
        }
        if let Some(v) = obj.remove("calc_settings") {
            prefs.calc_settings = load_calc_value(v, &CalcSettings::default())
                .settings()
                .clone();
        }
        Ok(prefs)
    }

    /// "Bunu varsayılan yap" ve dil seçimi. Mevcut haritalar değişmez (spec §4).
    pub fn set_app_prefs(&self, prefs: &AppPrefs) -> Result<(), StoreError> {
        if !LOCALES.contains(&prefs.locale.as_str()) {
            return Err(StoreError::InvalidInput("locale"));
        }
        prefs.calc_settings.validate()?;
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
    use jyotish_core::{AyanamsaSystem, CoreError, NodeKind};
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
                found_version: Some(1)
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
}
