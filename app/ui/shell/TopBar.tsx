import { useTranslation } from "react-i18next";
import { isLanguage, LANGUAGES, setLanguage, DEFAULT_LANGUAGE } from "../i18n";
import { isThemePref, THEME_PREFS, type ThemePref } from "../theme/theme";
import { LayoutPicker } from "./LayoutPicker";

export interface TopBarProps {
  themePref: ThemePref;
  onThemeChange(pref: ThemePref): void;
  printView: boolean;
  onTogglePrint(): void;
}

/** Üst çubuk (spec §4): solda başlık, ortada düzen seçici, sağda tema, dil ve baskı. */
export function TopBar({ themePref, onThemeChange, printView, onTogglePrint }: TopBarProps) {
  const { t, i18n } = useTranslation();
  const language = isLanguage(i18n.language) ? i18n.language : DEFAULT_LANGUAGE;
  return (
    <header className="topbar no-print">
      <h1 className="topbar-title">{t("app.title")}</h1>
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
        <button type="button" onClick={onTogglePrint}>
          {printView ? t("shell.backToDesk") : t("shell.printSpike")}
        </button>
      </div>
    </header>
  );
}
