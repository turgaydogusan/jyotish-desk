import { useTranslation } from "react-i18next";
import { isLanguage, LANGUAGES, setLanguage, DEFAULT_LANGUAGE } from "../i18n";
import { isThemePref, THEME_PREFS, type ThemePref } from "../theme/theme";
import { ChartTitle } from "./ChartTitle";
import { LayoutPicker } from "./LayoutPicker";

export interface TopBarProps {
  themePref: ThemePref;
  onThemeChange(pref: ThemePref): void;
  printView: boolean;
  onTogglePrint(): void;
  onHelp(): void;
}

/** Üst çubuk (spec §4): solda danışan ve harita (uyarı rozetleriyle), ortada düzen seçici, sağda tema, dil ve baskı. */
export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onHelp }: TopBarProps) {
  const { t, i18n } = useTranslation();
  const language = isLanguage(i18n.language) ? i18n.language : DEFAULT_LANGUAGE;
  return (
    <header className="topbar no-print">
      <ChartTitle />
      <div className="topbar-center">{!printView && <LayoutPicker />}</div>
      <div className="topbar-right">
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
        <button type="button" onClick={onTogglePrint}>
          {printView ? t("shell.backToDesk") : t("shell.printSpike")}
        </button>
      </div>
    </header>
  );
}
