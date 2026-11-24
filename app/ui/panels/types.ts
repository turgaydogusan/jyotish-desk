import type { ComponentType } from "react";
import type { Panel } from "../layout/model";

export interface PanelProps {
  panel: Panel;
}

export interface PanelDefinition {
  /** Panel başlığının i18n anahtarı. */
  titleKey: string;
  /** Başlığa eklenen kısa bilgi (ör. harita için varga). */
  subtitle?: (panel: Panel) => string | null;
  component: ComponentType<PanelProps>;
}
