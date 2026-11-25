//! Kullanıcının pano düzeni, açık harita olmadan (UI spec §3.2: düzenler kullanıcınındır).
//! `save_view_state` bir harita kimliği ister; ilk açılışta ve danışan açık değilken
//! arayüz düzeni bununla kaydeder (Plan 08 ertelenen madde 2).

use rusqlite::params;

use crate::charts::PANEL_LAYOUT_KEY;
use crate::view_state::PanelLayout;
use crate::{Store, StoreError};

impl Store {
    /// Yalnızca düzeni yazar; `None` kayıtlı düzeni siler (arayüz hazır düzenlere döner).
    /// Okuma `Store::panel_layout` iledir (toleranslı).
    pub fn save_panel_layout(&mut self, layout: Option<&PanelLayout>) -> Result<(), StoreError> {
        match layout {
            Some(pl) => self.app_state_set(PANEL_LAYOUT_KEY, &serde_json::to_string(pl)?),
            None => {
                self.conn.execute(
                    "DELETE FROM app_state WHERE key = ?1",
                    params![PANEL_LAYOUT_KEY],
                )?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Map;

    use crate::db::testutil::store;
    use crate::view_state::{Layout, Panel, PanelLayout};

    fn layout(id: &str) -> PanelLayout {
        PanelLayout {
            active_layout_id: id.into(),
            layouts: vec![Layout {
                id: id.into(),
                name: "Seans".into(),
                builtin: false,
                panels: vec![Panel {
                    id: "d1".into(),
                    kind: "chart".into(),
                    x: 0,
                    y: 0,
                    w: 6,
                    h: 12,
                    settings: Map::new(),
                }],
            }],
        }
    }

    #[test]
    fn layout_is_saved_without_a_chart_and_replaced_then_cleared() {
        let mut s = store();
        assert_eq!(s.panel_layout().unwrap(), None);
        s.save_panel_layout(Some(&layout("user:1"))).unwrap();
        assert_eq!(s.panel_layout().unwrap(), Some(layout("user:1")));
        s.save_panel_layout(Some(&layout("user:2"))).unwrap();
        assert_eq!(s.panel_layout().unwrap(), Some(layout("user:2")));
        s.save_panel_layout(None).unwrap();
        assert_eq!(s.panel_layout().unwrap(), None);
        s.save_panel_layout(None).unwrap();
    }
}
