// Üst çubuğun solu (UI spec §4): danışan adı, harita etiketi, uyarı rozetleri (UI spec §5) ve
// fark rozeti (Plan 11 D12). Rozet amber kenarlı ve işaretlidir ama anlamı metinle taşır;
// açıklama ipucu olarak (`title`) ve ekran okuyucu için gizli metin olarak verilir.
import { useTranslation } from "react-i18next";
import { useSession } from "../session/store";
import { settingsDiff } from "../settings/model";
import { usePrefs } from "../settings/prefs";

export function ChartTitle({ onSettings }: { onSettings?: () => void }) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { prefs } = usePrefs();
  if (!opened) return <h1 className="topbar-title">{t("app.title")}</h1>;
  const label = opened.chart_record.label || t(`chart.kind.${opened.chart_record.kind}`);
  const warnings = [...opened.chart_record.resolved.warnings, ...opened.chart.meta.warnings];
  const diff = prefs ? settingsDiff(opened.calc_settings, prefs.calc_settings) : [];
  const diffHint = t("settingsBadge.hint", { fields: diff.map((f) => t(`settings.field.${f}`)).join(", ") });
  return (
    <div className="topbar-title">
      <h1 className="topbar-client">{opened.client.name}</h1>
      <span className="topbar-chart">{label}</span>
      {(warnings.length > 0 || diff.length > 0) && (
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
          {diff.length > 0 && (
            <li>
              <button type="button" className="badge badge-button" title={diffHint} onClick={onSettings}>
                <span className="badge-mark" aria-hidden="true">
                  ≠
                </span>
                {t("settingsBadge.label")}
                <span className="visually-hidden">: {diffHint}</span>
              </button>
            </li>
          )}
        </ul>
      )}
    </div>
  );
}
