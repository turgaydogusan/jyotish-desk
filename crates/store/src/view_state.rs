//! `ViewState` (spec §5, UI spec §3.2) ve toleranslı ayrıştırma (spec §7: bozuk ya da eski
//! görünüm durumu açılışı asla engellemez; bilinmeyen anahtarlar yok sayılır, geçersiz
//! değerde varsayılan kullanılır).
//!
//! Saklama ikiye ayrılır (UI spec §3.2 "Düzenler kullanıcıya aittir"): `panel_layout`
//! kullanıcınındır ve `app_state` `panel_layout` anahtarındadır; dasha ağacının açık dalı
//! haritanındır ve `charts.view_state_json`'dadır. IPC'de ikisi tek `ViewState`'tir.
//!
//! Açık dal dasa sistemi başına ayrı tutulur (ileri modüller spec'i §6): Vimshottari yolu
//! `dasha_expanded_path`, chara dasha yolu `chara_expanded_path` alanındadır. Şema sürümü
//! değişmez: ikinci alanı taşımayan eski kayıt Vimshottari yolu sayılır ve chara yolu boş
//! okunur.
//!
//! Uyum panelinin eş haritası da haritanındır (ileri modüller spec'i §9): `compat_partner_uuid`
//! eş haritanın UUID'si, `compat_roles` açık haritanın rolüdür. Bu bağ iki danışan arasında bir
//! ilişki bilgisidir: yalnızca var olan başka bir haritayı gösterirken saklanır, gösterdiği
//! harita silinince temizlenir (`compat`) ve loglanmaz. Şema sürümü yine değişmez: iki alanı
//! taşımayan eski kayıt eşsiz okunur.

use jyotish_core::{Graha, Sign, SignDashaPath, MAX_DASHA_LEVEL, MAX_SIGN_DASHA_LEVEL};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use uuid::Uuid;

/// Dasha yolunun en çok uzunluğu (`MAX_DASHA_LEVEL` seviye).
const MAX_PATH: usize = MAX_DASHA_LEVEL as usize;
/// Chara dasha yolunun en çok uzunluğu (`MAX_SIGN_DASHA_LEVEL` seviye).
const MAX_SIGN_PATH: usize = MAX_SIGN_DASHA_LEVEL as usize;

/// Boş chara dasha yolu: ilk çevrim, açık dal yok.
pub fn empty_chara_path() -> SignDashaPath {
    SignDashaPath {
        cycle: 1,
        signs: Vec::new(),
    }
}

/// Chara dasha yolu geçerli mi: çevrim 1 ya da 2, en çok `MAX_SIGN_DASHA_LEVEL` burç.
fn chara_path_fits(path: &SignDashaPath) -> bool {
    (1..=2).contains(&path.cycle) && path.signs.len() <= MAX_SIGN_PATH
}

/// Uyum panelinde rollerin atanışı (ileri modüller spec'i §9): Ashtakuta yönlüdür, roller
/// açık harita ile eş harita arasında paylaşılır. Eş yokken değer `OpenIsGroom`'dur.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum CompatRoles {
    /// Açık harita "1. kişi (damat rolü)", eş harita gelin rolü.
    #[default]
    OpenIsGroom,
    /// Açık harita gelin rolü, eş harita damat rolü.
    OpenIsBride,
}

/// UUID'nin kanonik yazımı (küçük harf, tireli); UUID değilse `None`.
pub fn canonical_uuid(text: &str) -> Option<String> {
    Uuid::parse_str(text).ok().map(|u| u.to_string())
}

pub const VIEW_STATE_SCHEMA_VERSION: u32 = 1;
/// Okunan en çok düzen (hazır düzenler dahil); fazlası atılır. Arayüzün `MAX_LAYOUTS`'u
/// ile aynıdır (`app/ui/layout/fixtures/layout-parity.json`).
pub const MAX_LAYOUTS: usize = 64;
/// Bir düzende okunan en çok panel; fazlası atılır. Arayüzün `MAX_PANELS`'i ile aynıdır.
pub const MAX_PANELS: usize = 48;
/// Yazılan görünüm kaydının en büyük JSON boyutu (bayt). Arayüzün ürettiği en büyük düzen
/// (64 × 48 panel) bunun onda biri bile değildir; sınır bozuk ya da kötü niyetli yazımı keser.
pub const MAX_VIEW_STATE_BYTES: usize = 1 << 20;

/// Yazım denetimi (Plan 11 D4): ayrıştırıcıların kırpacağı kayıt reddedilir, saklanmaz.
pub fn check_panel_layout(pl: &PanelLayout) -> Result<(), crate::StoreError> {
    let fits =
        pl.layouts.len() <= MAX_LAYOUTS && pl.layouts.iter().all(|l| l.panels.len() <= MAX_PANELS);
    if !fits || serde_json::to_vec(pl)?.len() > MAX_VIEW_STATE_BYTES {
        return Err(crate::StoreError::InvalidInput("panel_layout"));
    }
    Ok(())
}

/// `save_view_state` denetimi: sürüm, iki dasha yolunun sınırları, eş haritanın UUID yazımı
/// (kanonik) ve düzen sınırları. Eşin var olup olmadığına burada bakılmaz (`Store` bakar).
pub fn check_view_state(vs: &ViewState) -> Result<(), crate::StoreError> {
    let partner_fits = match &vs.compat_partner_uuid {
        Some(uuid) => canonical_uuid(uuid).as_deref() == Some(uuid.as_str()),
        None => true,
    };
    if vs.schema_version != VIEW_STATE_SCHEMA_VERSION
        || vs.dasha_expanded_path.len() > usize::from(MAX_DASHA_LEVEL)
        || !chara_path_fits(&vs.chara_expanded_path)
        || !partner_fits
    {
        return Err(crate::StoreError::InvalidInput("view_state"));
    }
    match &vs.panel_layout {
        Some(pl) => check_panel_layout(pl),
        None => Ok(()),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewState {
    pub schema_version: u32,
    /// Kullanıcının pano düzenleri. `None`: hiç kaydedilmemiş ya da okunamadı; arayüz
    /// hazır düzenleri (`defaultLayoutState`) kullanır.
    pub panel_layout: Option<PanelLayout>,
    /// Vimshottari ağacında açık dal: kökten lord yolu, en çok `MAX_DASHA_LEVEL`.
    pub dasha_expanded_path: Vec<Graha>,
    /// Chara dasha ağacında açık dal: çevrim ve kökten burç yolu, en çok
    /// `MAX_SIGN_DASHA_LEVEL`. Boş yolun çevrimi 1'dir.
    pub chara_expanded_path: SignDashaPath,
    /// Uyum panelinin eş haritası: var olan başka bir haritanın UUID'si (kanonik yazım) ya da
    /// `None`. Harita başınadır; ilişki bilgisidir ve loglanmaz.
    pub compat_partner_uuid: Option<String>,
    /// Açık haritanın uyum rolü; eş yokken `OpenIsGroom`.
    pub compat_roles: CompatRoles,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            schema_version: VIEW_STATE_SCHEMA_VERSION,
            panel_layout: None,
            dasha_expanded_path: Vec::new(),
            chara_expanded_path: empty_chara_path(),
            compat_partner_uuid: None,
            compat_roles: CompatRoles::default(),
        }
    }
}

/// UI spec §3.2 biçimi: `{ active_layout_id, layouts: [{ id, name, builtin, panels }] }`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PanelLayout {
    /// Geçersizse boş dizgi; arayüz ilk hazır düzene döner.
    pub active_layout_id: String,
    pub layouts: Vec<Layout>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Layout {
    pub id: String,
    pub name: String,
    pub builtin: bool,
    pub panels: Vec<Panel>,
}

/// `kind` bilinmeyen bir değer de olabilir (daha yeni sürümün paneli); korunur.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Panel {
    pub id: String,
    pub kind: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    /// Panel ayarları (varga, stil, transit halkası…); anahtarlar arayüzündür. TS'te
    /// `{ [key in string]: unknown }` (serde_json `Value`'nun i64'ü specta'da yasaktır).
    #[cfg_attr(
        feature = "specta",
        specta(type = std::collections::BTreeMap<String, specta_typescript::Unknown>)
    )]
    pub settings: Map<String, Value>,
}

fn non_empty_str(v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) if !s.trim().is_empty() => Some(s.clone()),
        _ => None,
    }
}

/// Tamsayı konum/boyut: `3` ve kesirsiz `3.0`/`1e1` kabul edilir (arayüz `Number.isInteger`
/// ile aynı), aralık `min..=u32::MAX`.
fn int(v: Option<&Value>, min: u32) -> Option<u32> {
    let v = v?;
    let n = match v.as_u64() {
        Some(n) => n,
        None => {
            let f = v.as_f64()?;
            if f.fract() != 0.0 || !(0.0..=f64::from(u32::MAX)).contains(&f) {
                return None;
            }
            f as u64
        }
    };
    u32::try_from(n).ok().filter(|n| *n >= min)
}

fn parse_panel(v: &Value) -> Option<Panel> {
    let o = v.as_object()?;
    Some(Panel {
        id: non_empty_str(o.get("id"))?,
        kind: non_empty_str(o.get("kind"))?,
        x: int(o.get("x"), 0)?,
        y: int(o.get("y"), 0)?,
        w: int(o.get("w"), 1)?,
        h: int(o.get("h"), 1)?,
        settings: match o.get("settings") {
            Some(Value::Object(m)) => m.clone(),
            _ => Map::new(),
        },
    })
}

fn parse_layout(v: &Value) -> Option<Layout> {
    let o = v.as_object()?;
    let mut panels: Vec<Panel> = Vec::new();
    if let Some(Value::Array(items)) = o.get("panels") {
        for p in items.iter().filter_map(parse_panel) {
            if panels.len() == MAX_PANELS {
                break;
            }
            if !panels.iter().any(|q| q.id == p.id) {
                panels.push(p);
            }
        }
    }
    Some(Layout {
        id: non_empty_str(o.get("id"))?,
        name: non_empty_str(o.get("name"))?,
        builtin: matches!(o.get("builtin"), Some(Value::Bool(true))),
        panels,
    })
}

/// Toleranslı düzen ayrıştırma (UI `parseLayoutState` kuralları; hazır düzenleri yeniden
/// eklemek arayüzün işidir). Nesne değilse `None`.
pub fn parse_panel_layout(v: &Value) -> Option<PanelLayout> {
    let o = v.as_object()?;
    let mut layouts: Vec<Layout> = Vec::new();
    if let Some(Value::Array(items)) = o.get("layouts") {
        for l in items.iter().filter_map(parse_layout) {
            if layouts.len() == MAX_LAYOUTS {
                break;
            }
            if !layouts.iter().any(|m| m.id == l.id) {
                layouts.push(l);
            }
        }
    }
    Some(PanelLayout {
        active_layout_id: non_empty_str(o.get("active_layout_id")).unwrap_or_default(),
        layouts,
    })
}

/// Lord yolu; herhangi bir öğe geçersizse ya da yol `MAX_DASHA_LEVEL`'den uzunsa boş yol.
pub fn parse_dasha_path(v: Option<&Value>) -> Vec<Graha> {
    let Some(Value::Array(items)) = v else {
        return Vec::new();
    };
    if items.len() > MAX_PATH {
        return Vec::new();
    }
    items
        .iter()
        .map(|g| serde_json::from_value::<Graha>(g.clone()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_default()
}

/// Chara dasha yolu: `{"cycle": 1 | 2, "signs": [burç, …]}`. Nesne değilse, çevrim ya da bir
/// burç geçersizse ya da yol `MAX_SIGN_DASHA_LEVEL`'den uzunsa boş yol. Alan hiç yoksa (bu
/// alandan önce yazılmış kayıt) yine boş yol.
pub fn parse_chara_path(v: Option<&Value>) -> SignDashaPath {
    let parsed = (|| {
        let o = v?.as_object()?;
        let cycle = u8::try_from(o.get("cycle")?.as_u64()?).ok()?;
        let signs = o
            .get("signs")?
            .as_array()?
            .iter()
            .map(|s| serde_json::from_value::<Sign>(s.clone()).ok())
            .collect::<Option<Vec<_>>>()?;
        Some(SignDashaPath { cycle, signs })
    })();
    parsed
        .filter(chara_path_fits)
        .unwrap_or_else(empty_chara_path)
}

/// Eş bağı: `compat_partner_uuid` geçerli bir UUID ise kanonik yazımı ve `compat_roles`;
/// UUID yoksa ya da geçersizse bağ yoktur ve rol varsayılandır. Tanınmayan rol değeri
/// varsayılana düşer, bağ durur.
pub fn parse_compat(
    partner: Option<&Value>,
    roles: Option<&Value>,
) -> (Option<String>, CompatRoles) {
    let Some(uuid) = partner.and_then(Value::as_str).and_then(canonical_uuid) else {
        return (None, CompatRoles::default());
    };
    let roles = roles
        .and_then(|r| serde_json::from_value::<CompatRoles>(r.clone()).ok())
        .unwrap_or_default();
    (Some(uuid), roles)
}

/// Tam `ViewState` JSON'unu toleranslı okur (içe aktarma ve testler). Hiçbir girdide
/// hata ya da panik yoktur.
pub fn parse_view_state(raw: &str) -> ViewState {
    let Ok(v) = serde_json::from_str::<Value>(raw) else {
        return ViewState::default();
    };
    let (compat_partner_uuid, compat_roles) =
        parse_compat(v.get("compat_partner_uuid"), v.get("compat_roles"));
    ViewState {
        schema_version: VIEW_STATE_SCHEMA_VERSION,
        panel_layout: v.get("panel_layout").and_then(parse_panel_layout),
        dasha_expanded_path: parse_dasha_path(v.get("dasha_expanded_path")),
        chara_expanded_path: parse_chara_path(v.get("chara_expanded_path")),
        compat_partner_uuid,
        compat_roles,
    }
}

/// Haritaya düşen kısım: iki dasha yolu ve eş bağı (`panel_layout` kullanıcınındır, yazılmaz).
/// `{"schema_version":1,"dasha_expanded_path":[…],"chara_expanded_path":{…},
/// "compat_partner_uuid":null,"compat_roles":"OpenIsGroom"}`. Eş yoksa rol varsayılan yazılır.
pub fn chart_view_json(vs: &ViewState) -> String {
    let roles = match vs.compat_partner_uuid {
        Some(_) => vs.compat_roles,
        None => CompatRoles::default(),
    };
    json!({
        "schema_version": VIEW_STATE_SCHEMA_VERSION,
        "dasha_expanded_path": vs.dasha_expanded_path,
        "chara_expanded_path": vs.chara_expanded_path,
        "compat_partner_uuid": vs.compat_partner_uuid,
        "compat_roles": roles,
    })
    .to_string()
}

/// Yeni haritanın görünüm kaydı: iki yol da boş, eş yok.
pub fn empty_chart_view_json() -> String {
    chart_view_json(&ViewState::default())
}

/// `charts.view_state_json`'dan lord yolu; bozuk JSON boş yol verir.
pub fn chart_view_path(raw: &str) -> Vec<Graha> {
    serde_json::from_str::<Value>(raw)
        .map(|v| parse_dasha_path(v.get("dasha_expanded_path")))
        .unwrap_or_default()
}

/// `charts.view_state_json`'dan chara dasha yolu; bozuk JSON ya da eksik alan boş yol verir.
pub fn chart_view_chara_path(raw: &str) -> SignDashaPath {
    serde_json::from_str::<Value>(raw)
        .map(|v| parse_chara_path(v.get("chara_expanded_path")))
        .unwrap_or_else(|_| empty_chara_path())
}

/// `charts.view_state_json`'dan eş bağı; bozuk JSON ya da eksik alan bağsız okunur. Eşin var
/// olup olmadığına burada bakılmaz.
pub fn chart_view_compat(raw: &str) -> (Option<String>, CompatRoles) {
    serde_json::from_str::<Value>(raw)
        .map(|v| parse_compat(v.get("compat_partner_uuid"), v.get("compat_roles")))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const PARTNER: &str = "4b0f2a8e-0000-4000-8000-00000000000a";

    fn sample() -> ViewState {
        let mut settings = Map::new();
        settings.insert("varga".into(), json!("D9"));
        settings.insert("transit_ring".into(), json!(true));
        ViewState {
            schema_version: 1,
            panel_layout: Some(PanelLayout {
                active_layout_id: "user:1".into(),
                layouts: vec![Layout {
                    id: "user:1".into(),
                    name: "Benim".into(),
                    builtin: false,
                    panels: vec![Panel {
                        id: "d9".into(),
                        kind: "chart".into(),
                        x: 6,
                        y: 0,
                        w: 6,
                        h: 12,
                        settings,
                    }],
                }],
            }),
            dasha_expanded_path: vec![Graha::Rahu, Graha::Jupiter],
            chara_expanded_path: SignDashaPath {
                cycle: 2,
                signs: vec![Sign::Aries, Sign::Taurus],
            },
            compat_partner_uuid: Some(PARTNER.into()),
            compat_roles: CompatRoles::OpenIsBride,
        }
    }

    #[test]
    fn valid_state_round_trips() {
        let vs = sample();
        let json = serde_json::to_string(&vs).unwrap();
        assert_eq!(parse_view_state(&json), vs);
    }

    #[test]
    fn broken_input_gives_defaults() {
        for raw in ["", "null", "[]", "42", "{", r#"{"panel_layout":7}"#] {
            assert_eq!(parse_view_state(raw), ViewState::default(), "{raw:?}");
        }
    }

    #[test]
    fn invalid_parts_are_dropped_and_unknown_kept() {
        let raw = json!({
            "schema_version": 99,
            "future_key": {"a": 1},
            "dasha_expanded_path": ["Rahu", "Pluto"],
            "panel_layout": {
                "active_layout_id": 5,
                "layouts": [
                    {"id": "a", "name": "A", "builtin": "yes", "panels": [
                        {"id": "p", "kind": "astro_clock", "x": 0, "y": 0, "w": 3, "h": 4, "settings": 1},
                        {"id": "p", "kind": "chart", "x": 0, "y": 0, "w": 3, "h": 4},
                        {"id": "q", "kind": "chart", "x": -1, "y": 0, "w": 3, "h": 4},
                        {"id": "r", "kind": "chart", "x": 0, "y": 0, "w": 0, "h": 4},
                        {"id": "s", "kind": "chart", "x": 1.5, "y": 0, "w": 3, "h": 4}
                    ]},
                    {"id": "a", "name": "duplicate"},
                    {"id": "", "name": "no id"},
                    "garbage"
                ]
            }
        })
        .to_string();
        let vs = parse_view_state(&raw);
        assert_eq!(vs.schema_version, 1);
        assert!(vs.dasha_expanded_path.is_empty());
        assert_eq!(vs.chara_expanded_path, empty_chara_path());
        assert_eq!(vs.compat_partner_uuid, None);
        let pl = vs.panel_layout.unwrap();
        assert_eq!(pl.active_layout_id, "");
        assert_eq!(pl.layouts.len(), 1);
        let l = &pl.layouts[0];
        assert_eq!((l.name.as_str(), l.builtin), ("A", false));
        assert_eq!(l.panels.len(), 1);
        assert_eq!(l.panels[0].kind, "astro_clock");
        assert!(l.panels[0].settings.is_empty());
    }

    /// Arayüzle ortak fikstür: iki ayrıştırıcı aynı düzenleri ve panelleri tutar.
    const PARITY: &str = include_str!("../../../app/ui/layout/fixtures/layout-parity.json");

    #[test]
    fn parity_fixture_matches_the_ui_parser() {
        let fixture: Value = serde_json::from_str(PARITY).unwrap();
        assert_eq!(fixture["max_layouts"], json!(MAX_LAYOUTS));
        assert_eq!(fixture["max_panels"], json!(MAX_PANELS));
        for case in fixture["cases"].as_array().unwrap() {
            let name = case["name"].as_str().unwrap();
            let pl = parse_panel_layout(&case["input"]).unwrap();
            let got: Vec<Value> = pl
                .layouts
                .iter()
                .map(|l| json!({"id": l.id, "panels": l.panels.iter().map(|p| &p.id).collect::<Vec<_>>()}))
                .collect();
            assert_eq!(Value::Array(got), case["layouts"], "{name}");
        }
    }

    #[test]
    fn layout_and_panel_counts_are_capped() {
        let panels: Vec<Value> = (0..MAX_PANELS + 10)
            .map(
                |i| json!({"id": format!("p{i}"), "kind": "chart", "x": 0, "y": i, "w": 3, "h": 4}),
            )
            .collect();
        let layouts: Vec<Value> = (0..MAX_LAYOUTS + 6)
            .map(|i| json!({"id": format!("user:{i}"), "name": "L", "panels": panels}))
            .collect();
        let pl = parse_panel_layout(&json!({ "layouts": layouts })).unwrap();
        assert_eq!(pl.layouts.len(), MAX_LAYOUTS);
        assert!(pl.layouts.iter().all(|l| l.panels.len() == MAX_PANELS));
        assert_eq!(
            pl.layouts[MAX_LAYOUTS - 1].id,
            format!("user:{}", MAX_LAYOUTS - 1)
        );
    }

    #[test]
    fn dasha_path_longer_than_five_levels_is_dropped() {
        let six = json!(["Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn"]);
        assert!(parse_dasha_path(Some(&six)).is_empty());
        let five = json!(["Sun", "Moon", "Mars", "Rahu", "Jupiter"]);
        assert_eq!(parse_dasha_path(Some(&five)).len(), 5);
    }

    #[test]
    fn chart_view_keeps_the_two_dasha_paths_and_the_partner_link() {
        let chara = SignDashaPath {
            cycle: 2,
            signs: vec![Sign::Leo],
        };
        let vs = ViewState {
            dasha_expanded_path: vec![Graha::Venus],
            chara_expanded_path: chara.clone(),
            compat_partner_uuid: Some(PARTNER.into()),
            compat_roles: CompatRoles::OpenIsBride,
            ..sample()
        };
        // Düzen kullanıcınındır: haritanın kaydına yazılmaz.
        let raw = chart_view_json(&vs);
        assert_eq!(
            raw,
            format!(
                r#"{{"chara_expanded_path":{{"cycle":2,"signs":["Leo"]}},"compat_partner_uuid":"{PARTNER}","compat_roles":"OpenIsBride","dasha_expanded_path":["Venus"],"schema_version":1}}"#
            )
        );
        assert_eq!(chart_view_path(&raw), [Graha::Venus]);
        assert_eq!(chart_view_chara_path(&raw), chara);
        assert_eq!(
            chart_view_compat(&raw),
            (Some(PARTNER.to_owned()), CompatRoles::OpenIsBride)
        );
        assert!(chart_view_path("{broken").is_empty());
        assert_eq!(chart_view_chara_path("{broken"), empty_chara_path());
        assert_eq!(
            chart_view_compat("{broken"),
            (None, CompatRoles::OpenIsGroom)
        );
        assert_eq!(
            empty_chart_view_json(),
            r#"{"chara_expanded_path":{"cycle":1,"signs":[]},"compat_partner_uuid":null,"compat_roles":"OpenIsGroom","dasha_expanded_path":[],"schema_version":1}"#
        );
        // Eş yokken rol saklanmaz: varsayılan yazılır.
        let no_partner = ViewState {
            compat_partner_uuid: None,
            ..vs
        };
        assert_eq!(
            chart_view_compat(&chart_view_json(&no_partner)),
            (None, CompatRoles::OpenIsGroom)
        );
    }

    #[test]
    fn the_partner_link_parses_tolerantly() {
        let read = |partner: Value, roles: Value| parse_compat(Some(&partner), Some(&roles));
        // Büyük harfli UUID kanonik yazıma çevrilir.
        assert_eq!(
            read(json!(PARTNER.to_uppercase()), json!("OpenIsBride")),
            (Some(PARTNER.to_owned()), CompatRoles::OpenIsBride)
        );
        // Tanınmayan rol varsayılana düşer; bağ durur.
        assert_eq!(
            read(json!(PARTNER), json!("Both")),
            (Some(PARTNER.to_owned()), CompatRoles::OpenIsGroom)
        );
        // UUID olmayan değer: bağ yok, rol varsayılan.
        for bad in [
            json!(null),
            json!(7),
            json!(""),
            json!("ayşe"),
            json!([PARTNER]),
        ] {
            assert_eq!(
                read(bad.clone(), json!("OpenIsBride")),
                (None, CompatRoles::OpenIsGroom),
                "{bad}"
            );
        }
        assert_eq!(parse_compat(None, None), (None, CompatRoles::OpenIsGroom));
        // Bu alanlardan önce yazılmış kayıt eşsiz okunur.
        let old = r#"{"dasha_expanded_path":["Rahu"],"schema_version":1}"#;
        assert_eq!(chart_view_compat(old), (None, CompatRoles::OpenIsGroom));
        let vs = parse_view_state(old);
        assert_eq!(
            (vs.compat_partner_uuid, vs.compat_roles),
            (None, CompatRoles::OpenIsGroom)
        );
    }

    #[test]
    fn a_view_state_with_a_partner_that_is_not_a_canonical_uuid_is_rejected() {
        assert!(check_view_state(&sample()).is_ok());
        for bad in [
            "",
            "ayşe",
            &PARTNER.to_uppercase(),
            "4b0f2a8e00004000800000000000000a",
        ] {
            let vs = ViewState {
                compat_partner_uuid: Some(bad.to_owned()),
                ..sample()
            };
            assert!(
                matches!(
                    check_view_state(&vs),
                    Err(crate::StoreError::InvalidInput("view_state"))
                ),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_record_written_before_the_chara_path_reads_as_the_vimshottari_path() {
        // Plan 16 öncesinin kaydı: yalnızca Vimshottari yolu vardır.
        let old = r#"{"dasha_expanded_path":["Rahu","Saturn"],"schema_version":1}"#;
        assert_eq!(chart_view_path(old), [Graha::Rahu, Graha::Saturn]);
        assert_eq!(chart_view_chara_path(old), empty_chara_path());
        let vs = parse_view_state(old);
        assert_eq!(vs.dasha_expanded_path, [Graha::Rahu, Graha::Saturn]);
        assert_eq!(vs.chara_expanded_path, empty_chara_path());
    }

    #[test]
    fn invalid_chara_paths_are_dropped_whole() {
        let ok = json!({"cycle": 2, "signs": ["Aries", "Taurus", "Gemini"]});
        assert_eq!(
            parse_chara_path(Some(&ok)),
            SignDashaPath {
                cycle: 2,
                signs: vec![Sign::Aries, Sign::Taurus, Sign::Gemini]
            }
        );
        for bad in [
            json!(null),
            json!(["Aries"]),
            json!({"signs": ["Aries"]}),
            json!({"cycle": 1}),
            json!({"cycle": 0, "signs": ["Aries"]}),
            json!({"cycle": 3, "signs": ["Aries"]}),
            json!({"cycle": 1.5, "signs": ["Aries"]}),
            json!({"cycle": "1", "signs": ["Aries"]}),
            json!({"cycle": 1, "signs": "Aries"}),
            json!({"cycle": 1, "signs": ["Aries", "Ophiuchus"]}),
            json!({"cycle": 1, "signs": ["Aries", "Taurus", "Gemini", "Cancer"]}),
        ] {
            assert_eq!(parse_chara_path(Some(&bad)), empty_chara_path(), "{bad}");
        }
        assert_eq!(parse_chara_path(None), empty_chara_path());
    }

    fn arb_json() -> impl Strategy<Value = Value> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::from),
            any::<i64>().prop_map(Value::from),
            any::<f64>().prop_map(|f| json!(f)),
            "[a-z_]{0,8}".prop_map(Value::from),
            Just(json!("Rahu")),
            Just(json!("chart")),
        ];
        leaf.prop_recursive(4, 48, 6, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..6).prop_map(Value::Array),
                prop::collection::btree_map(
                    prop_oneof![
                        Just("panel_layout".to_owned()),
                        Just("layouts".to_owned()),
                        Just("panels".to_owned()),
                        Just("dasha_expanded_path".to_owned()),
                        Just("chara_expanded_path".to_owned()),
                        Just("compat_partner_uuid".to_owned()),
                        Just("compat_roles".to_owned()),
                        Just("cycle".to_owned()),
                        Just("signs".to_owned()),
                        Just("id".to_owned()),
                        Just("name".to_owned()),
                        Just("kind".to_owned()),
                        Just("x".to_owned()),
                        Just("w".to_owned()),
                        "[a-z]{1,4}",
                    ],
                    inner,
                    0..6
                )
                .prop_map(|m| Value::Object(m.into_iter().collect())),
            ]
        })
    }

    fn arb_graha() -> impl Strategy<Value = Graha> {
        prop::sample::select(Graha::ALL.to_vec())
    }

    fn arb_panel() -> impl Strategy<Value = Panel> {
        (
            "[a-z0-9]{1,6}",
            "[a-z_]{1,10}",
            0u32..12,
            0u32..40,
            1u32..12,
            1u32..20,
            any::<bool>(),
        )
            .prop_map(|(id, kind, x, y, w, h, ring)| {
                let mut settings = Map::new();
                settings.insert("transit_ring".into(), Value::Bool(ring));
                Panel {
                    id,
                    kind,
                    x,
                    y,
                    w,
                    h,
                    settings,
                }
            })
    }

    fn arb_view_state() -> impl Strategy<Value = ViewState> {
        let layout = (
            "[a-z:0-9]{1,8}",
            "[A-Za-zçğış ]{0,6}[a-z]",
            any::<bool>(),
            prop::collection::vec(arb_panel(), 0..5),
        )
            .prop_map(|(id, name, builtin, panels)| {
                let mut uniq: Vec<Panel> = Vec::new();
                for p in panels {
                    if !uniq.iter().any(|q| q.id == p.id) {
                        uniq.push(p);
                    }
                }
                Layout {
                    id,
                    name,
                    builtin,
                    panels: uniq,
                }
            });
        (
            prop::option::of(("[a-z:0-9]{1,8}", prop::collection::vec(layout, 0..4))),
            prop::collection::vec(arb_graha(), 0..=MAX_PATH),
            1u8..=2,
            prop::collection::vec(prop::sample::select(Sign::ALL.to_vec()), 0..=MAX_SIGN_PATH),
            prop::option::of((any::<u128>(), any::<bool>())),
        )
            .prop_map(|(pl, path, cycle, signs, compat)| ViewState {
                schema_version: VIEW_STATE_SCHEMA_VERSION,
                panel_layout: pl.map(|(active, ls)| {
                    let mut uniq: Vec<Layout> = Vec::new();
                    for l in ls {
                        if !uniq.iter().any(|m| m.id == l.id) {
                            uniq.push(l);
                        }
                    }
                    PanelLayout {
                        active_layout_id: active,
                        layouts: uniq,
                    }
                }),
                dasha_expanded_path: path,
                chara_expanded_path: SignDashaPath { cycle, signs },
                compat_partner_uuid: compat.map(|(bits, _)| Uuid::from_u128(bits).to_string()),
                compat_roles: match compat {
                    Some((_, true)) => CompatRoles::OpenIsBride,
                    _ => CompatRoles::OpenIsGroom,
                },
            })
    }

    proptest! {
        #[test]
        fn any_string_parses_without_panic(raw in ".{0,200}") {
            let vs = parse_view_state(&raw);
            prop_assert_eq!(vs.schema_version, VIEW_STATE_SCHEMA_VERSION);
        }

        #[test]
        fn any_json_parses_within_bounds(v in arb_json()) {
            let vs = parse_view_state(&v.to_string());
            prop_assert!(vs.dasha_expanded_path.len() <= MAX_PATH);
            prop_assert!(chara_path_fits(&vs.chara_expanded_path));
            let chart_part = ViewState {
                panel_layout: None,
                ..vs.clone()
            };
            prop_assert!(check_view_state(&chart_part).is_ok());
            if vs.compat_partner_uuid.is_none() {
                prop_assert_eq!(vs.compat_roles, CompatRoles::OpenIsGroom);
            }
            if let Some(pl) = &vs.panel_layout {
                for l in &pl.layouts {
                    prop_assert!(!l.id.trim().is_empty());
                    for p in &l.panels {
                        prop_assert!(p.w >= 1 && p.h >= 1);
                    }
                }
            }
        }

        #[test]
        fn generated_states_round_trip(vs in arb_view_state()) {
            let raw = serde_json::to_string(&vs).unwrap();
            prop_assert_eq!(parse_view_state(&raw), vs);
        }
    }
}
