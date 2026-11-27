import type { TFunction } from "i18next";
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
  /** Yalnızca erişilebilir ada eklenen ayırt edici bilgi: aynı türden iki panel aynı düzende
   *  durduğunda bölgeler aynı adı taşımaz (ör. dasa ağacının sistemi). */
  nameDetail?: (panel: Panel, t: TFunction) => string | null;
  component: ComponentType<PanelProps>;
}
