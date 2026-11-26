// Panel türü → bileşen (spec §3.3). Yeni panel türü yalnızca buraya eklenir.
import { DashaStripPanel } from "../dasha/DashaStripPanel";
import { DashaTreePanel } from "../dasha/DashaTreePanel";
import { isKnownPanelKind, type KnownPanelKind } from "../layout/model";
import { NotesPanel } from "../notes/NotesPanel";
import { ArgalaPanel } from "./ArgalaPanel";
import { ArudhaPanel } from "./ArudhaPanel";
import { AshtakavargaPanel } from "./AshtakavargaPanel";
import { BhavaBalaPanel } from "./BhavaBalaPanel";
import { BirthInfoPanel } from "./BirthInfoPanel";
import { ChartPanel, chartVarga } from "./ChartPanel";
import { CompatPanel } from "./CompatPanel";
import { TransitTablePanel } from "../transit/TransitTablePanel";
import { DrishtiPanel } from "./DrishtiPanel";
import { GrahaTablePanel } from "./GrahaTablePanel";
import { JaiminiKarakasPanel } from "./JaiminiKarakasPanel";
import { PanchangPanel } from "./PanchangPanel";
import { ShadbalaPanel } from "./ShadbalaPanel";
import type { PanelDefinition } from "./types";
import { UnavailablePanel } from "./UnavailablePanel";
import { YogasPanel } from "./YogasPanel";

export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
  chart: { titleKey: "panel.chart", subtitle: (p) => chartVarga(p.settings), component: ChartPanel },
  graha_table: { titleKey: "panel.graha_table", component: GrahaTablePanel },
  dasha_tree: { titleKey: "panel.dasha_tree", component: DashaTreePanel },
  dasha_strip: { titleKey: "panel.dasha_strip", component: DashaStripPanel },
  transit_table: { titleKey: "panel.transit_table", component: TransitTablePanel },
  notes: { titleKey: "panel.notes", component: NotesPanel },
  birth_info: { titleKey: "panel.birth_info", component: BirthInfoPanel },
  // İleri modül panelleri (Plan 14).
  panchang: { titleKey: "panel.panchang", component: PanchangPanel },
  ashtakavarga: { titleKey: "panel.ashtakavarga", component: AshtakavargaPanel },
  drishti: { titleKey: "panel.drishti", component: DrishtiPanel },
  // Jaimini çekirdeği (Plan 15).
  jaimini_karakas: { titleKey: "panel.jaimini_karakas", component: JaiminiKarakasPanel },
  arudha: { titleKey: "panel.arudha", component: ArudhaPanel },
  argala: { titleKey: "panel.argala", component: ArgalaPanel },
  // Shadbala ve Bhava bala (Plan 17).
  shadbala: { titleKey: "panel.shadbala", component: ShadbalaPanel },
  bhava_bala: { titleKey: "panel.bhava_bala", component: BhavaBalaPanel },
  // Yogalar (Plan 18).
  yogas: { titleKey: "panel.yogas", component: YogasPanel },
  // Uyum (Plan 19).
  compat: { titleKey: "panel.compat", component: CompatPanel },
};

const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };

export function panelDefinition(kind: string): PanelDefinition {
  return isKnownPanelKind(kind) ? PANEL_REGISTRY[kind] : UNKNOWN;
}
