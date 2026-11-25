// Üst çubuğun solu (UI spec §4): danışan adı, harita etiketi ve uyarı rozetleri (UI spec §5).
// Rozet amber kenarlı ve işaretlidir ama anlamı metinle taşır; açıklama ipucu olarak
// (`title`) ve ekran okuyucu için gizli metin olarak verilir.
import { useTranslation } from "react-i18next";
import { useSession } from "../session/store";

export function ChartTitle() {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <h1 className="topbar-title">{t("app.title")}</h1>;
  const label = opened.chart_record.label || t(`chart.kind.${opened.chart_record.kind}`);
  const warnings = [...opened.chart_record.resolved.warnings, ...opened.chart.meta.warnings];
  return (
    <div className="topbar-title">
      <h1 className="topbar-client">{opened.client.name}</h1>
      <span className="topbar-chart">{label}</span>
      {warnings.length > 0 && (
        <ul className="badges" aria-label={t("shell.warnings")}>
          {warnings.map((w) => (
            <li key={w} className="badge" tabIndex={0} title={t(`badgeHint.${w}`)}>
              <span className="badge-mark" aria-hidden="true">
                ▲
              </span>
              {t(`badge.${w}`)}
              <span className="visually-hidden">: {t(`badgeHint.${w}`)}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
