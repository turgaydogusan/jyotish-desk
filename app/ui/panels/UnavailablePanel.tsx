import { useTranslation } from "react-i18next";
import { isKnownPanelKind } from "../layout/model";
import "./panels.css";
import type { PanelProps } from "./types";

/**
 * Henüz uygulanmamış panel türleri ve tanınmayan türler için. Hata değildir: panel
 * düzende yerini korur ve sonraki bir sürümde içeriğiyle gelir.
 */
export function UnavailablePanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  return (
    <p className="panel-message">
      {isKnownPanelKind(panel.kind) ? t("panel.unavailable") : t("panel.unknownKind", { kind: panel.kind })}
    </p>
  );
}
