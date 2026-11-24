// Panel durumları (spec §5): 100 ms altında gösterge yok, 100 ms – 1 s arası başlıkta satır
// içi gösterge, 1 s üstünde panel iskeleti. Hatalar panel içinde, kurtarma eylemiyle.
import { createContext, useContext, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError } from "../bindings";

export type LoadPhase = "none" | "inline" | "skeleton";
export const INLINE_AFTER_MS = 100;
export const SKELETON_AFTER_MS = 1000;

/** Panel çerçevesi (pano) bu bağlamla başlıktaki satır içi göstergeyi öğrenir. */
export const PanelStatusContext = createContext<(phase: LoadPhase) => void>(() => {});

export function usePanelLoadPhase(loading: boolean): LoadPhase {
  const [phase, setPhase] = useState<LoadPhase>("none");
  useEffect(() => {
    if (!loading) {
      setPhase("none");
      return;
    }
    const inline = setTimeout(() => setPhase("inline"), INLINE_AFTER_MS);
    const skeleton = setTimeout(() => setPhase("skeleton"), SKELETON_AFTER_MS);
    return () => {
      clearTimeout(inline);
      clearTimeout(skeleton);
    };
  }, [loading]);
  const effective: LoadPhase = loading ? phase : "none";
  const report = useContext(PanelStatusContext);
  useEffect(() => {
    report(effective);
  }, [effective, report]);
  return effective;
}

export function PanelSkeleton() {
  const { t } = useTranslation();
  return (
    <div className="panel-skeleton" aria-busy="true" aria-label={t("panel.loading")}>
      <div className="panel-skeleton-bar" />
      <div className="panel-skeleton-bar" />
      <div className="panel-skeleton-bar" />
    </div>
  );
}

// Arka uçtan gelen parametreler i18next seçeneklerini (dil, çoğul, bağlam…) değiştirememeli.
const RESERVED_OPTIONS = new Set(["count", "context", "lng", "lngs", "defaultValue", "ns", "replace", "returnObjects", "joinArrays", "postProcess", "interpolation", "fallbackLng"]);

function safeParams(params: Record<string, unknown>): Record<string, unknown> {
  return Object.fromEntries(Object.entries(params).filter(([k]) => !RESERVED_OPTIONS.has(k)));
}

export function PanelError({ error, onRetry }: { error: CommandError; onRetry: () => void }) {
  const { t, i18n } = useTranslation();
  const key = `errors.${error.code}`;
  const message = i18n.exists(key) ? t(key, safeParams(error.params)) : t("errors.unknown", { code: error.code });
  return (
    <div className="panel-error" role="alert">
      <p>{message}</p>
      <button type="button" onClick={onRetry}>
        {t("panel.retry")}
      </button>
    </div>
  );
}
