// Hesap ayarları penceresi (spec §4 "Ayar katmanları"). Açık harita varsa onun ayarı düzenlenir
// ve harita Rust'ta yeniden hesaplanır (`save_calc_settings`); yoksa yeni haritaların genel
// varsayılanı (`AppPrefs.calc_settings`). Kayıt sıralı ve debounce'ludur, kapanırken ve sayfa
// gizlenirken boşaltılır (C5, C17). Varsayılandan farklı alanlar metinle işaretlenir (C9).
import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CalcSettings, CommandError } from "../bindings";
import { DEFAULT_LANGUAGE, isLanguage } from "../i18n";
import { errorMessage } from "../i18n/errors";
import { useSession } from "../session/store";
import { Dialog } from "../shell/Dialog";
import { useNotices } from "../shell/notices";
import { SAVE_DEBOUNCE_MS } from "../view/store";
import {
  ayanamsaKey,
  DASHA_YEARS,
  HOUSE_SYSTEMS,
  NODES,
  ORB_KEYS,
  parseOrb,
  PRESET_AYANAMSAS,
  sameValue,
  SUNRISE_DEFS,
  ZODIACS,
  type OrbKey,
  type SettingsField,
} from "./model";
import { usePrefs } from "./prefs";
import { createSaver } from "./saves";
import "./settings.css";

type Status = "idle" | "saving" | "saved" | "madeDefault";

function orbTexts(s: CalcSettings | null): Record<OrbKey, string> {
  return Object.fromEntries(ORB_KEYS.map((k) => [k, s ? String(s.combustion_orbs[k] ?? "") : ""])) as Record<OrbKey, string>;
}

export function SettingsDialog({ onClose, debounceMs = SAVE_DEBOUNCE_MS }: { onClose(): void; debounceMs?: number }) {
  const { t, i18n } = useTranslation();
  const { opened, updateSettings } = useSession();
  const { prefs, save } = usePrefs();
  const { push } = useNotices();
  const id = useId();
  const [chartId] = useState(() => opened?.chart_record.id ?? null);
  const [draft, setDraft] = useState<CalcSettings | null>(() => opened?.calc_settings ?? prefs?.calc_settings ?? null);
  const [orbs, setOrbs] = useState(() => orbTexts(draft));
  const [error, setError] = useState<CommandError | null>(null);
  const [status, setStatus] = useState<Status>("idle");
  const mounted = useRef(true);
  const latest = useRef({ prefs, language: i18n.language, updateSettings, push, save });
  latest.current = { prefs, language: i18n.language, updateSettings, push, save };
  const locale = () => (isLanguage(latest.current.language) ? latest.current.language : DEFAULT_LANGUAGE);

  const [saver] = useState(() =>
    createSaver<CalcSettings>(
      async (s) => {
        if (chartId !== null) {
          const r = await api.saveCalcSettings(chartId, s);
          if (r.status === "error") return r.error;
          latest.current.updateSettings(chartId, s, r.data);
          return null;
        }
        const base = latest.current.prefs;
        return base ? latest.current.save({ ...base, locale: locale(), calc_settings: s }) : null;
      },
      {
        debounceMs,
        onError: (e) => {
          if (mounted.current) {
            setError(e);
            setStatus("idle");
          } else latest.current.push({ id: "settings", type: "error", messageKey: "notice.settingsSaveFailed", error: e });
        },
        onSaved: () => {
          if (!mounted.current) return;
          setError(null);
          setStatus("saved");
        },
      },
    ),
  );

  useEffect(() => {
    mounted.current = true;
    const hidden = () => document.visibilityState === "hidden" && saver.flush();
    const pagehide = () => saver.flush();
    document.addEventListener("visibilitychange", hidden);
    window.addEventListener("pagehide", pagehide);
    return () => {
      mounted.current = false;
      document.removeEventListener("visibilitychange", hidden);
      window.removeEventListener("pagehide", pagehide);
      saver.flush();
    };
  }, [saver]);

  const change = (next: CalcSettings) => {
    setDraft(next);
    setStatus("saving");
    saver.schedule(next);
  };

  const defaults = prefs?.calc_settings ?? null;
  const differs = (f: SettingsField) => chartId !== null && defaults !== null && draft !== null && !sameValue(draft[f], defaults[f]);
  const anyDiff = draft !== null && defaults !== null && chartId !== null && !sameValue({ ...draft, schema_version: 0 }, { ...defaults, schema_version: 0 });
  const mark = (f: SettingsField) =>
    differs(f) && (
      <span className="settings-differs">
        <span aria-hidden="true">≠ </span>
        {t("settings.differs")}
      </span>
    );

  const row = (f: SettingsField, control: ReactNode) => (
    <div className={`settings-row${differs(f) ? " settings-row--differs" : ""}`}>
      <label htmlFor={`${id}-${f}`}>{t(`settings.field.${f}`)}</label>
      {control}
      {mark(f)}
    </div>
  );

  const resetToDefault = () => {
    if (!defaults) return;
    setOrbs(orbTexts(defaults));
    change(defaults);
    saver.flush();
  };

  const makeDefault = async () => {
    if (!prefs || !draft) return;
    saver.flush();
    const e = await save({ ...prefs, locale: locale(), calc_settings: draft });
    if (!mounted.current) return;
    if (e) setError(e);
    else {
      setError(null);
      setStatus("madeDefault");
    }
  };

  const editOrb = (k: OrbKey, text: string) => {
    const next = { ...orbs, [k]: text };
    setOrbs(next);
    if (!draft) return;
    const values = ORB_KEYS.map((key) => parseOrb(next[key]));
    if (values.some((v) => v === null)) return;
    change({ ...draft, combustion_orbs: Object.fromEntries(ORB_KEYS.map((key, i) => [key, values[i]])) as CalcSettings["combustion_orbs"] });
  };

  const label = opened ? opened.chart_record.label || t(`chart.kind.${opened.chart_record.kind}`) : "";
  return (
    <Dialog title={t("settings.title")} onClose={onClose}>
      <p className="settings-scope">{chartId !== null && opened ? t("settings.chartScope", { name: opened.client.name, chart: label }) : t("settings.defaultsScope")}</p>
      {draft === null ? (
        <p>{t("settings.noPrefs")}</p>
      ) : (
        <div className="settings-form">
          {row(
            "zodiac",
            <select id={`${id}-zodiac`} value={draft.zodiac} data-autofocus onChange={(e) => change({ ...draft, zodiac: e.target.value as CalcSettings["zodiac"] })}>
              {ZODIACS.map((z) => (
                <option key={z} value={z}>
                  {t(`settings.zodiac.${z}`)}
                </option>
              ))}
            </select>,
          )}
          {row(
            "ayanamsa",
            ayanamsaKey(draft.ayanamsa) === "Custom" ? (
              <>
                <select id={`${id}-ayanamsa`} value="Custom" disabled aria-describedby={`${id}-custom`}>
                  <option value="Custom">{t("settings.ayanamsa.Custom")}</option>
                </select>
                <span id={`${id}-custom`} className="settings-hint">
                  {t("settings.customLocked")}
                </span>
              </>
            ) : (
              <select id={`${id}-ayanamsa`} value={ayanamsaKey(draft.ayanamsa)} onChange={(e) => change({ ...draft, ayanamsa: e.target.value as CalcSettings["ayanamsa"] })}>
                {PRESET_AYANAMSAS.map((a) => (
                  <option key={a} value={a}>
                    {t(`settings.ayanamsa.${a}`)}
                  </option>
                ))}
              </select>
            ),
          )}
          {row(
            "node",
            <select id={`${id}-node`} value={draft.node} onChange={(e) => change({ ...draft, node: e.target.value as CalcSettings["node"] })}>
              {NODES.map((n) => (
                <option key={n} value={n}>
                  {t(`settings.node.${n}`)}
                </option>
              ))}
            </select>,
          )}
          {row(
            "house_system",
            <select id={`${id}-house_system`} value={draft.house_system} onChange={(e) => change({ ...draft, house_system: e.target.value as CalcSettings["house_system"] })}>
              {HOUSE_SYSTEMS.map((h) => (
                <option key={h} value={h}>
                  {t(`settings.house.${h}`)}
                </option>
              ))}
            </select>,
          )}
          {row(
            "dasha_year",
            <select id={`${id}-dasha_year`} value={String(draft.dasha_year)} onChange={(e) => change({ ...draft, dasha_year: Number(e.target.value) })}>
              {[...new Set<number>([...DASHA_YEARS, draft.dasha_year ?? 365.25])].map((d) => (
                <option key={d} value={String(d)}>
                  {t("settings.dashaYearOption", { days: d })}
                </option>
              ))}
            </select>,
          )}
          {row(
            "sunrise_def",
            <select id={`${id}-sunrise_def`} value={draft.sunrise_def} onChange={(e) => change({ ...draft, sunrise_def: e.target.value as CalcSettings["sunrise_def"] })}>
              {SUNRISE_DEFS.map((s) => (
                <option key={s} value={s}>
                  {t(`settings.sunrise.${s}`)}
                </option>
              ))}
            </select>,
          )}
          <fieldset className={`settings-orbs${differs("combustion_orbs") ? " settings-row--differs" : ""}`}>
            <legend>
              {t("settings.field.combustion_orbs")} {mark("combustion_orbs")}
            </legend>
            {ORB_KEYS.map((k) => {
              const invalid = parseOrb(orbs[k]) === null;
              return (
                <div key={k} className="settings-orb">
                  <label htmlFor={`${id}-orb-${k}`}>{t(`settings.orb.${k}`)}</label>
                  <input
                    id={`${id}-orb-${k}`}
                    inputMode="decimal"
                    value={orbs[k]}
                    aria-invalid={invalid || undefined}
                    aria-describedby={invalid ? `${id}-orb-${k}-error` : undefined}
                    onChange={(e) => editOrb(k, e.target.value)}
                  />
                  {invalid && (
                    <span id={`${id}-orb-${k}-error`} className="field-error">
                      {t("settings.orbInvalid")}
                    </span>
                  )}
                </div>
              );
            })}
          </fieldset>
        </div>
      )}
      {error && (
        <p className="field-error" role="alert">
          {errorMessage(t, i18n, error)}
        </p>
      )}
      <p role="status" className="settings-status">
        {status === "idle" ? "" : t(`settings.status.${status}`)}
      </p>
      <div className="dialog-actions">
        {chartId !== null && (
          <>
            <button type="button" disabled={!anyDiff} onClick={resetToDefault}>
              {t("settings.resetToDefault")}
            </button>
            <button type="button" disabled={!anyDiff || prefs === null} onClick={() => void makeDefault()}>
              {t("settings.makeDefault")}
            </button>
          </>
        )}
        <button type="button" onClick={onClose}>
          {t("settings.close")}
        </button>
      </div>
    </Dialog>
  );
}
