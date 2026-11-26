// Panel türü → bileşen (spec §3.3). Yeni panel türü yalnızca buraya eklenir.
import { DashaStripPanel } from "../dasha/DashaStripPanel";
import { DashaTreePanel } from "../dasha/DashaTreePanel";
import { isKnownPanelKind, type KnownPanelKind } from "../layout/model";
import { NotesPanel } from "../notes/NotesPanel";
import { BirthInfoPanel } from "./BirthInfoPanel";
import { ChartPanel, chartVarga } from "./ChartPanel";
import { TransitTablePanel } from "../transit/TransitTablePanel";
import { GrahaTablePanel } from "./GrahaTablePanel";
import type { PanelDefinition } from "./types";
import { UnavailablePanel } from "./UnavailablePanel";

export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
  chart: { titleKey: "panel.chart", subtitle: (p) => chartVarga(p.settings), component: ChartPanel },
  graha_table: { titleKey: "panel.graha_table", component: GrahaTablePanel },
  dasha_tree: { titleKey: "panel.dasha_tree", component: DashaTreePanel },
  dasha_strip: { titleKey: "panel.dasha_strip", component: DashaStripPanel },
  transit_table: { titleKey: "panel.transit_table", component: TransitTablePanel },
  notes: { titleKey: "panel.notes", component: NotesPanel },
  birth_info: { titleKey: "panel.birth_info", component: BirthInfoPanel },
  // İleri modül panelleri (Plan 14): içerikleri Task 6–8'de gelir.
  panchang: { titleKey: "panel.panchang", component: UnavailablePanel },
  ashtakavarga: { titleKey: "panel.ashtakavarga", component: UnavailablePanel },
  drishti: { titleKey: "panel.drishti", component: UnavailablePanel },
};

const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };

export function panelDefinition(kind: string): PanelDefinition {
  return isKnownPanelKind(kind) ? PANEL_REGISTRY[kind] : UNKNOWN;
}
