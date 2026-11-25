import { useTranslation } from "react-i18next";
import { isLanguage, LANGUAGES, setLanguage, DEFAULT_LANGUAGE } from "../i18n";
import { isThemePref, THEME_PREFS, type ThemePref } from "../theme/theme";
import { SIDEBAR_ID } from "../clients/Sidebar";
import { ChartTitle } from "./ChartTitle";
import { LayoutPicker } from "./LayoutPicker";

export interface TopBarProps {
  themePref: ThemePref;
  onThemeChange(pref: ThemePref): void;
  printView: boolean;
  /** Açık harita yoksa verilmez (düğme gösterilmez). */
  onTogglePrint?: () => void;
  onHelp(): void;
  onNewChart(): void;
  /** Açık harita yoksa verilmez (düğme gösterilmez). */
  onEditChart?: () => void;
  sidebarOpen: boolean;
  onToggleSidebar(): void;
  onPalette(): void;
  onSettings(): void;
}

/** Üst çubuk (spec §4): solda danışan ve harita (uyarı rozetleriyle), ortada düzen seçici, sağda tema, dil ve baskı. */
export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onHelp, onNewChart, onEditChart, sidebarOpen, onToggleSidebar, onPalette, onSettings }: TopBarProps) {
  const { t, i18n } = useTranslation();
  const language = isLanguage(i18n.language) ? i18n.language : DEFAULT_LANGUAGE;
  return (
    <header className="topbar no-print">
      {!printView && (
        <button type="button" aria-expanded={sidebarOpen} aria-controls={SIDEBAR_ID} aria-keyshortcuts="Meta+Backslash" onClick={onToggleSidebar}>
          {t("clients.toggle")}
        </button>
      )}
      <ChartTitle onSettings={onSettings} />
      <div className="topbar-actions">
        <button type="button" aria-keyshortcuts="Meta+N" onClick={onNewChart}>
          {t("empty.action")}
        </button>
        {onEditChart && (
          <button type="button" aria-keyshortcuts="Meta+E" onClick={onEditChart}>
            {t("birth.edit")}
          </button>
        )}
      </div>
      <div className="topbar-center">{!printView && <LayoutPicker />}</div>
      <div className="topbar-right">
        <button type="button" aria-keyshortcuts="Meta+K" onClick={onPalette}>
          {t("palette.open")}
        </button>
        <button type="button" aria-keyshortcuts="Meta+Comma" onClick={onSettings}>
          {t("settings.open")}
        </button>
        <label>
          {t("theme.label")}
          <select value={themePref} onChange={(e) => isThemePref(e.target.value) && onThemeChange(e.target.value)}>
            {THEME_PREFS.map((p) => (
              <option key={p} value={p}>
                {t(`theme.${p}`)}
              </option>
            ))}
          </select>
        </label>
        <label>
          {t("language.label")}
          <select value={language} onChange={(e) => isLanguage(e.target.value) && void setLanguage(e.target.value)}>
            {LANGUAGES.map((l) => (
              <option key={l} value={l}>
                {t(`language.${l}`)}
              </option>
            ))}
          </select>
        </label>
        <button type="button" aria-keyshortcuts="?" onClick={onHelp}>
          {t("shortcuts.open")}
        </button>
        {onTogglePrint && (
          <button type="button" aria-keyshortcuts={printView ? undefined : "Meta+P"} onClick={onTogglePrint}>
            {printView ? t("print.back") : t("print.open")}
          </button>
        )}
      </div>
    </header>
  );
}
