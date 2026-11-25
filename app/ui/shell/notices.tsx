// Modal olmayan bildirim bandı (spec §7: hesap ve kayıt hataları, açılış uyarıları).
// Bildirimler yapı olarak saklanır ve çizimde çevrilir; dil değişince metin de değişir.
// Aynı `id` ile gelen bildirim öncekinin yerini alır (ör. tekrarlayan kayıt hatası).
import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError, StoreWarning } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { TzdbActions } from "./TzdbActions";

export type Notice =
  | { id: string; type: "error"; messageKey: string; error: CommandError }
  | {
      id: string;
      type: "warning" | "info";
      messageKey: string;
      params: Record<string, string>;
      /** Bildirimle gelen kararın eylemleri; bileşen oturumdan okur (veri, işlev değil). */
      action?: "tzdb";
    };

export interface Notices {
  notices: Notice[];
  push(notice: Notice): void;
  dismiss(id: string): void;
  /** Koşula uyan bildirimleri o anki listeden kaldırır (`await` sonrası eski liste okunmaz). */
  dismissWhere(match: (notice: Notice) => boolean): void;
}

const NoticeContext = createContext<Notices | null>(null);

export function NoticeProvider({ initial = [], children }: { initial?: Notice[]; children: ReactNode }) {
  const [notices, setNotices] = useState(initial);
  const push = useCallback((n: Notice) => setNotices((all) => [...all.filter((m) => m.id !== n.id), n]), []);
  const dismiss = useCallback((id: string) => setNotices((all) => all.filter((m) => m.id !== id)), []);
  const dismissWhere = useCallback((match: (n: Notice) => boolean) => setNotices((all) => all.filter((m) => !match(m))), []);
  const value = useMemo(() => ({ notices, push, dismiss, dismissWhere }), [notices, push, dismiss, dismissWhere]);
  return <NoticeContext.Provider value={value}>{children}</NoticeContext.Provider>;
}

export function useNotices(): Notices {
  const n = useContext(NoticeContext);
  if (n === null) throw new Error("useNotices must be used inside <NoticeProvider>");
  return n;
}

/** `open_client` uyarıları (spec §7) → bildirimler. */
export function storeWarningNotices(warnings: readonly StoreWarning[]): Notice[] {
  return warnings.map((w): Notice => {
    if (w === "LastChartMissing") return { id: "store:LastChartMissing", type: "warning", messageKey: "storeWarning.LastChartMissing", params: {} };
    if ("CalcSettingsReset" in w) return { id: "store:CalcSettingsReset", type: "warning", messageKey: "storeWarning.CalcSettingsReset", params: {} };
    return {
      id: "store:TzdbChanged",
      type: "warning",
      messageKey: "storeWarning.TzdbChanged",
      params: { stored: w.TzdbChanged.stored_version, current: w.TzdbChanged.current_version, utc: w.TzdbChanged.current_utc ?? "—" },
      action: "tzdb",
    };
  });
}

export function NoticeBar() {
  const { t, i18n } = useTranslation();
  const { notices, dismiss } = useNotices();
  if (notices.length === 0) return null;
  return (
    <section className="notice-bar no-print" aria-label={t("notice.region")}>
      {notices.map((n) => (
        <div key={n.id} className={`notice notice--${n.type}`} role={n.type === "error" ? "alert" : "status"}>
          <strong className="notice-kind">{t(`notice.${n.type}`)}</strong>
          <span className="notice-text">
            {n.type === "error" ? t(n.messageKey, { message: errorMessage(t, i18n, n.error) }) : t(n.messageKey, n.params)}
          </span>
          {n.type !== "error" && n.action === "tzdb" && <TzdbActions />}
          <button type="button" onClick={() => dismiss(n.id)}>
            {t("notice.dismiss")}
          </button>
        </div>
      ))}
    </section>
  );
}
