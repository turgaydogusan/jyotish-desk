// Hakkında penceresi: sürüm, lisans ve kaynak kod adresi (AGPL, spec §10), veri kaynaklarının
// atfı (GeoNames CC BY, spec §9) ve güncelleme (spec §10: imzalı, kullanıcı onayıyla).
// Güncelleme iki ayrı onaydır: denetim (düğme ya da açılış tercihi, varsayılan kapalı) ve
// kurulum (bulunan sürüm için düğme). Kaynak adresi seçilebilir metindir; pencere tarayıcı açmaz.
import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { AppInfo, CommandError, UpdateInfo } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { Dialog } from "../shell/Dialog";
import { loadAutoCheck, saveAutoCheck } from "../update/pref";
import "./about.css";

export const SOURCE_URL = "https://github.com/turgaydogusan/jyotish-desk";

type Phase =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "current" }
  | { kind: "available"; update: UpdateInfo }
  | { kind: "installing"; update: UpdateInfo }
  | { kind: "installed"; version: string };

export function AboutDialog({ onClose }: { onClose(): void }) {
  const { t, i18n } = useTranslation();
  const id = useId();
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [auto, setAuto] = useState(() => loadAutoCheck());
  const [prefLost, setPrefLost] = useState(false);
  const [phase, setPhase] = useState<Phase>({ kind: "idle" });
  const [error, setError] = useState<CommandError | null>(null);
  const mounted = useRef(true);
  const closeRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    mounted.current = true;
    void api.appInfo().then((r) => {
      if (mounted.current && r.status === "ok") setInfo(r.data);
    });
    return () => {
      mounted.current = false;
    };
  }, []);

  const busy = phase.kind === "checking" || phase.kind === "installing";

  const changeAuto = (next: boolean) => {
    const stored = saveAutoCheck(next);
    // Yazılamayan onay açık sayılmaz.
    setAuto(stored && next);
    setPrefLost(!stored);
  };

  const check = async () => {
    if (busy) return;
    setError(null);
    setPhase({ kind: "checking" });
    const r = await api.checkForUpdate();
    if (!mounted.current) return;
    if (r.status === "error") {
      setError(r.error);
      setPhase({ kind: "idle" });
    } else setPhase(r.data === null ? { kind: "current" } : { kind: "available", update: r.data });
  };

  const install = async (update: UpdateInfo) => {
    if (busy) return;
    setError(null);
    setPhase({ kind: "installing", update });
    const r = await api.installUpdate(update.version);
    if (!mounted.current) return;
    if (r.status === "error") {
      setError(r.error);
      setPhase({ kind: "available", update });
    } else {
      setPhase({ kind: "installed", version: update.version });
      // Kur düğmesi kalkar; odak pencerede bir düğmede durur.
      closeRef.current?.focus();
    }
  };

  const update = phase.kind === "available" || phase.kind === "installing" ? phase.update : null;
  const status =
    phase.kind === "checking"
      ? t("update.checking")
      : phase.kind === "current"
        ? t("update.current")
        : phase.kind === "installing"
          ? t("update.installing", { version: phase.update.version })
          : phase.kind === "installed"
            ? t("update.installed", { version: phase.version })
            : phase.kind === "available"
              ? phase.update.published
                ? t("update.availableOn", { version: phase.update.version, date: phase.update.published })
                : t("update.available", { version: phase.update.version })
              : "";

  return (
    <Dialog title={t("about.title")} onClose={onClose}>
      <div className="about">
        <p className="about-version">{info ? t("about.version", { version: info.version }) : t("about.versionUnknown")}</p>
        <p>{t("about.license")}</p>
        <p>
          {t("about.source")} <span className="about-source">{SOURCE_URL}</span>
        </p>
        <p className="about-hint">{t("about.swisseph")}</p>
        <p className="about-hint">{t("about.geonames")}</p>
        <fieldset className="about-updates">
          <legend>{t("update.title")}</legend>
          {info !== null && !info.updaterConfigured ? (
            <p className="about-hint">{t("update.unconfigured")}</p>
          ) : (
            <>
              <label>
                <input type="checkbox" checked={auto} aria-describedby={`${id}-privacy`} onChange={(e) => changeAuto(e.target.checked)} /> {t("update.auto")}
              </label>
              <p id={`${id}-privacy`} className="about-hint">
                {t("update.privacy")}
              </p>
              {prefLost && (
                <p className="field-error" role="alert">
                  {t("update.prefLost")}
                </p>
              )}
              <div className="dialog-actions">
                <button type="button" data-autofocus aria-disabled={busy || undefined} aria-describedby={busy ? `${id}-status` : undefined} onClick={() => void check()}>
                  {t("update.check")}
                </button>
                {update && (
                  <button type="button" aria-disabled={busy || undefined} aria-describedby={`${id}-status`} onClick={() => void install(update)}>
                    {t("update.install", { version: update.version })}
                  </button>
                )}
              </div>
              <p id={`${id}-status`} role="status" className="about-status">
                {status}
              </p>
              {update?.notes && (
                <p className="about-notes" aria-label={t("update.notes", { version: update.version })}>
                  {update.notes}
                </p>
              )}
              {error && (
                <p className="field-error" role="alert">
                  {errorMessage(t, i18n, error)}
                </p>
              )}
            </>
          )}
        </fieldset>
      </div>
      <div className="dialog-actions">
        <button ref={closeRef} type="button" onClick={onClose}>
          {t("about.close")}
        </button>
      </div>
    </Dialog>
  );
}
