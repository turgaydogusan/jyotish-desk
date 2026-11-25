// Panel türü → bileşen (spec §3.3). Yeni panel türü yalnızca buraya eklenir.
import { DashaTreePanel } from "../dasha/DashaTreePanel";
import { isKnownPanelKind, type KnownPanelKind } from "../layout/model";
import { NotesPanel } from "../notes/NotesPanel";
import { ChartPanel, chartVarga } from "./ChartPanel";
import { TransitTablePanel } from "../transit/TransitTablePanel";
import { GrahaTablePanel } from "./GrahaTablePanel";
import type { PanelDefinition } from "./types";
import { UnavailablePanel } from "./UnavailablePanel";

export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
  chart: { titleKey: "panel.chart", subtitle: (p) => chartVarga(p.settings), component: ChartPanel },
  graha_table: { titleKey: "panel.graha_table", component: GrahaTablePanel },
  dasha_tree: { titleKey: "panel.dasha_tree", component: DashaTreePanel },
  dasha_strip: { titleKey: "panel.dasha_strip", component: UnavailablePanel },
  transit_table: { titleKey: "panel.transit_table", component: TransitTablePanel },
  notes: { titleKey: "panel.notes", component: NotesPanel },
  birth_info: { titleKey: "panel.birth_info", component: UnavailablePanel },
};

const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };

export function panelDefinition(kind: string): PanelDefinition {
  return isKnownPanelKind(kind) ? PANEL_REGISTRY[kind] : UNKNOWN;
}
