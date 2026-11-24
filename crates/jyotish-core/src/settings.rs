//! Hesap ayarları (spec §4 "Ayar katmanları", §5 `CalcSettings`, §6 varsayılanlar).
//! Sürümlüdür; harita oluşturulurken tam kopyası haritaya yazılır.

use ephemeris::{Body, SiderealMode};
use serde::{Deserialize, Serialize};

use crate::CoreError;

pub const CALC_SETTINGS_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum ZodiacMode {
    /// Burç, nakshatra ve evler sidereal boylamdan.
    Sidereal,
    /// Burç, burç içi derece ve evler tropikal boylamdan; nakshatra ve pada sidereal boylamdan.
    TropicalRasiSiderealNakshatra,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum AyanamsaSystem {
    Lahiri,
    Raman,
    /// Krishnamurti (KP).
    Kp,
    /// True Chitrapaksha (Spica 180°).
    TrueChitra,
    /// `t0_jd` (UT) anında ayanamsa `deg` derecedir; SE presesyonla ilerletir.
    Custom {
        t0_jd: f64,
        deg: f64,
    },
}

impl AyanamsaSystem {
    pub fn sidereal_mode(self) -> SiderealMode {
        match self {
            Self::Lahiri => SiderealMode::Lahiri,
            Self::Raman => SiderealMode::Raman,
            Self::Kp => SiderealMode::Krishnamurti,
            Self::TrueChitra => SiderealMode::TrueChitra,
            Self::Custom { t0_jd, deg } => SiderealMode::Custom {
                t0_jd_ut: t0_jd,
                ayan_t0_deg: deg,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum NodeKind {
    Mean,
    True,
}

impl NodeKind {
    pub fn body(self) -> Body {
        match self {
            Self::Mean => Body::MeanNode,
            Self::True => Body::TrueNode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum HouseSystem {
    WholeSign,
    /// Bhava chalit: Porphyry madhyaları, sandhi'ler bhava başlangıcı.
    Sripati,
    /// KP.
    Placidus,
}

impl HouseSystem {
    pub fn ephemeris_system(self) -> ephemeris::HouseSystem {
        match self {
            Self::WholeSign => ephemeris::HouseSystem::WholeSign,
            Self::Sripati => ephemeris::HouseSystem::Sripati,
            Self::Placidus => ephemeris::HouseSystem::Placidus,
        }
    }
}

/// Yanma orbları, derece (BPHS).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CombustionOrbs {
    pub moon: f64,
    pub mars: f64,
    pub mercury: f64,
    pub mercury_retro: f64,
    pub jupiter: f64,
    pub venus: f64,
    pub venus_retro: f64,
    pub saturn: f64,
}

impl Default for CombustionOrbs {
    fn default() -> Self {
        Self {
            moon: 12.0,
            mars: 17.0,
            mercury: 14.0,
            mercury_retro: 12.0,
            jupiter: 11.0,
            venus: 10.0,
            venus_retro: 8.0,
            saturn: 15.0,
        }
    }
}

impl CombustionOrbs {
    fn all(&self) -> [f64; 8] {
        [
            self.moon,
            self.mars,
            self.mercury,
            self.mercury_retro,
            self.jupiter,
            self.venus,
            self.venus_retro,
            self.saturn,
        ]
    }
}

/// Gün doğumu tanımı. Bu planda kullanılmaz (M3 panchang); ayar olarak saklanır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum SunriseDef {
    /// Disk merkezi, kırılmasız (`SE_BIT_HINDU_RISING`).
    DiskCenterNoRefraction,
    /// Üst kenar, atmosfer kırılmasıyla (batı konvansiyonu).
    UpperLimbRefracted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CalcSettings {
    pub schema_version: u32,
    pub zodiac: ZodiacMode,
    pub ayanamsa: AyanamsaSystem,
    pub node: NodeKind,
    pub house_system: HouseSystem,
    /// Vimshottari yıl uzunluğu, gün. Bu planda kullanılmaz (M3 dasha).
    pub dasha_year: f64,
    pub combustion_orbs: CombustionOrbs,
    pub sunrise_def: SunriseDef,
}

impl Default for CalcSettings {
    fn default() -> Self {
        Self {
            schema_version: CALC_SETTINGS_SCHEMA_VERSION,
            zodiac: ZodiacMode::Sidereal,
            ayanamsa: AyanamsaSystem::Lahiri,
            node: NodeKind::Mean,
            house_system: HouseSystem::WholeSign,
            dasha_year: 365.25,
            combustion_orbs: CombustionOrbs::default(),
            sunrise_def: SunriseDef::DiskCenterNoRefraction,
        }
    }
}

impl CalcSettings {
    /// Sürüm ve değer aralıklarını denetler. Sürüm göçü `store` planında yapılır;
    /// burada yalnızca desteklenen sürüm kabul edilir.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.schema_version != CALC_SETTINGS_SCHEMA_VERSION {
            return Err(CoreError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: CALC_SETTINGS_SCHEMA_VERSION,
            });
        }
        if !(self.dasha_year.is_finite() && self.dasha_year > 0.0) {
            return Err(CoreError::InvalidSettings("dasha_year"));
        }
        // NaN ve sonsuz aralık dışında kalır.
        if self
            .combustion_orbs
            .all()
            .iter()
            .any(|o| !(0.0..=30.0).contains(o))
        {
            return Err(CoreError::InvalidSettings("combustion_orbs"));
        }
        if let AyanamsaSystem::Custom { t0_jd, deg } = self.ayanamsa {
            if !(t0_jd.is_finite() && deg.is_finite()) {
                return Err(CoreError::InvalidSettings("ayanamsa"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // `super::*`, `Body`, `SiderealMode` ve `CoreError`'ı da getirir.
    use super::*;

    const DEFAULT_JSON: &str = concat!(
        r#"{"schema_version":1,"zodiac":"Sidereal","ayanamsa":"Lahiri","node":"Mean","#,
        r#""house_system":"WholeSign","dasha_year":365.25,"combustion_orbs":{"moon":12.0,"#,
        r#""mars":17.0,"mercury":14.0,"mercury_retro":12.0,"jupiter":11.0,"venus":10.0,"#,
        r#""venus_retro":8.0,"saturn":15.0},"sunrise_def":"DiskCenterNoRefraction"}"#
    );

    #[test]
    fn default_matches_spec_defaults() {
        let s = CalcSettings::default();
        assert_eq!(s.schema_version, 1);
        assert_eq!(s.zodiac, ZodiacMode::Sidereal);
        assert_eq!(s.ayanamsa, AyanamsaSystem::Lahiri);
        assert_eq!(s.node, NodeKind::Mean);
        assert_eq!(s.house_system, HouseSystem::WholeSign);
        assert_eq!(s.dasha_year, 365.25);
        assert_eq!(s.sunrise_def, SunriseDef::DiskCenterNoRefraction);
        let o = s.combustion_orbs;
        assert_eq!(
            [
                o.moon,
                o.mars,
                o.mercury,
                o.mercury_retro,
                o.jupiter,
                o.venus,
                o.venus_retro,
                o.saturn
            ],
            [12.0, 17.0, 14.0, 12.0, 11.0, 10.0, 8.0, 15.0]
        );
    }

    #[test]
    fn default_json_is_stable() {
        let json = serde_json::to_string(&CalcSettings::default()).unwrap();
        assert_eq!(json, DEFAULT_JSON);
        let back: CalcSettings = serde_json::from_str(DEFAULT_JSON).unwrap();
        assert_eq!(back, CalcSettings::default());
    }

    #[test]
    fn custom_ayanamsa_round_trips() {
        let a = AyanamsaSystem::Custom {
            t0_jd: 2_451_545.0,
            deg: 23.5,
        };
        let json = serde_json::to_string(&a).unwrap();
        assert_eq!(json, r#"{"Custom":{"t0_jd":2451545.0,"deg":23.5}}"#);
        assert_eq!(serde_json::from_str::<AyanamsaSystem>(&json).unwrap(), a);
    }

    #[test]
    fn unknown_variant_is_rejected() {
        let bad = DEFAULT_JSON.replace(r#""Lahiri""#, r#""Fagan""#);
        assert!(serde_json::from_str::<CalcSettings>(&bad).is_err());
    }

    #[test]
    fn sidereal_mode_mapping() {
        assert_eq!(AyanamsaSystem::Lahiri.sidereal_mode(), SiderealMode::Lahiri);
        assert_eq!(AyanamsaSystem::Raman.sidereal_mode(), SiderealMode::Raman);
        assert_eq!(
            AyanamsaSystem::Kp.sidereal_mode(),
            SiderealMode::Krishnamurti
        );
        assert_eq!(
            AyanamsaSystem::TrueChitra.sidereal_mode(),
            SiderealMode::TrueChitra
        );
        assert_eq!(
            AyanamsaSystem::Custom {
                t0_jd: 2_451_545.0,
                deg: 23.5
            }
            .sidereal_mode(),
            SiderealMode::Custom {
                t0_jd_ut: 2_451_545.0,
                ayan_t0_deg: 23.5
            }
        );
    }

    #[test]
    fn node_body_mapping() {
        assert_eq!(NodeKind::Mean.body(), Body::MeanNode);
        assert_eq!(NodeKind::True.body(), Body::TrueNode);
    }

    #[test]
    fn house_system_mapping() {
        use ephemeris::HouseSystem as E;
        assert_eq!(HouseSystem::WholeSign.ephemeris_system(), E::WholeSign);
        assert_eq!(HouseSystem::Sripati.ephemeris_system(), E::Sripati);
        assert_eq!(HouseSystem::Placidus.ephemeris_system(), E::Placidus);
    }

    #[test]
    fn validate_accepts_defaults() {
        assert_eq!(CalcSettings::default().validate(), Ok(()));
    }

    #[test]
    fn validate_rejects_bad_values() {
        let s = CalcSettings {
            schema_version: 2,
            ..CalcSettings::default()
        };
        assert_eq!(
            s.validate(),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 2,
                supported: 1
            })
        );

        let s = CalcSettings {
            dasha_year: 0.0,
            ..CalcSettings::default()
        };
        assert_eq!(s.validate(), Err(CoreError::InvalidSettings("dasha_year")));

        let mut s = CalcSettings::default();
        s.combustion_orbs.venus_retro = f64::NAN;
        assert_eq!(
            s.validate(),
            Err(CoreError::InvalidSettings("combustion_orbs"))
        );

        let mut s = CalcSettings::default();
        s.combustion_orbs.mars = 31.0;
        assert_eq!(
            s.validate(),
            Err(CoreError::InvalidSettings("combustion_orbs"))
        );

        let s = CalcSettings {
            ayanamsa: AyanamsaSystem::Custom {
                t0_jd: 2_451_545.0,
                deg: f64::INFINITY,
            },
            ..CalcSettings::default()
        };
        assert_eq!(s.validate(), Err(CoreError::InvalidSettings("ayanamsa")));
    }
}
