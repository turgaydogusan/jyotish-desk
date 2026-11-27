// Transit anı (UI spec §3.3, §3.4; Plan 11 D7, D8). An oturumluktur: kaydedilmez, her açılışta ve
// her yeni haritada "şimdi"dir. Transitler yalnızca bir panel istediğinde okunur (`useTransit`);
// konumlar, "şimdi" ve dasa yolu Rust'tan gelir (C4). Eski ana ait geç yanıt düşürülür.
import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { api } from "../api";
import type { CommandError, TransitAt, TransitView } from "../bindings";
import { useSession } from "../session/store";

export interface Transit {
  at: TransitAt;
  /** Bu haritanın son yanıtı (yeni an yüklenirken eskisi gösterilir); hata varsa `null`. */
  view: TransitView | null;
  error: CommandError | null;
  loading: boolean;
  /**
   * "Şimdi" her seçildiğinde ve her yeniden denemede artar. Anı kendisi okuyan bileşenler (gün
   * panchangı) bunu anahtarlarına katar: an "şimdi" olarak dursa da yeniden okurlar.
   */
  epoch: number;
  setAt(at: TransitAt): void;
  retry(): void;
}

interface Context extends Transit {
  register(): () => void;
}

const TransitContext = createContext<Context | null>(null);

interface Reply {
  key: string;
  chartId: number;
  view: TransitView | null;
  error: CommandError | null;
}

export function TransitProvider({ children }: { children: ReactNode }) {
  const { opened } = useSession();
  const [at, setAtState] = useState<TransitAt>("Now");
  const [demand, setDemand] = useState(0);
  const [nonce, setNonce] = useState(0);
  const [reply, setReply] = useState<Reply | null>(null);
  const chartId = opened?.chart_record.id ?? null;
  // Yeni harita: an "şimdi"ye döner (çizim sırasında; eski an bir kez bile istenmez).
  const [shownChart, setShownChart] = useState(chartId);
  if (shownChart !== chartId) {
    setShownChart(chartId);
    setAtState("Now");
  }

  const key = opened && demand > 0 ? JSON.stringify([chartId, at, nonce, opened.chart.jd_ut, opened.calc_settings]) : null;
  useEffect(() => {
    if (key === null || opened === null) return;
    let live = true;
    const id = opened.chart_record.id;
    void api.transitsAt(at, opened.calc_settings, opened.chart.bhavas, opened.chart.dasha.seed).then((r) => {
      if (!live) return;
      setReply(r.status === "ok" ? { key, chartId: id, view: r.data, error: null } : { key, chartId: id, view: null, error: r.error });
    });
    return () => {
      live = false;
    };
    // `key` açık haritayı, anı, ayarı ve yeniden denemeyi kapsar.
  }, [key]);

  const register = useCallback(() => {
    setDemand((d) => d + 1);
    return () => setDemand((d) => d - 1);
  }, []);
  // "Şimdi" her seçildiğinde yeniden okunur: an zaten "şimdi" olsa da Rust'un saati ilerlemiştir.
  const setAt = useCallback((next: TransitAt) => {
    setAtState(next);
    if (next === "Now") setNonce((n) => n + 1);
  }, []);
  const retry = useCallback(() => setNonce((n) => n + 1), []);

  const mine = reply !== null && reply.chartId === chartId ? reply : null;
  const value = useMemo<Context>(
    () => ({
      at,
      view: mine?.view ?? null,
      error: mine && mine.key === key ? mine.error : null,
      loading: key !== null && mine?.key !== key,
      epoch: nonce,
      setAt,
      retry,
      register,
    }),
    [at, mine, key, nonce, setAt, retry, register],
  );
  return <TransitContext.Provider value={value}>{children}</TransitContext.Provider>;
}

function useContextValue(): Context {
  const c = useContext(TransitContext);
  if (c === null) throw new Error("useTransit must be used inside <TransitProvider>");
  return c;
}

/** Transitleri gösteren bileşenler: çizildiği sürece transitler okunur. */
export function useTransit(): Transit {
  const c = useContextValue();
  const { register } = c;
  useEffect(() => register(), [register]);
  return c;
}

/** Yalnızca anı değiştiren ya da okuyan bileşenler: kendi başına okuma başlatmaz. */
export function useTransitControl(): Transit {
  return useContextValue();
}

/** Tarayıcının IANA dilimi; bilinmiyorsa UTC. */
export function browserTimeZone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
}

/** UTC dizgisi → dilimdeki duvar saati (`YYYY-MM-DD`, `HH:MM`); biçimlemedir, hesap değil. */
export function localParts(utc: string, zone: string): { date: string; time: string } | null {
  const d = new Date(utc);
  if (Number.isNaN(d.getTime())) return null;
  const parts = Object.fromEntries(
    new Intl.DateTimeFormat("en-CA", { timeZone: zone, year: "numeric", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", hourCycle: "h23" })
      .formatToParts(d)
      .map((p) => [p.type, p.value]),
  );
  return { date: `${parts.year}-${parts.month}-${parts.day}`, time: `${parts.hour}:${parts.minute}` };
}

export function formatMoment(utc: string, language: string, zone: string): string {
  const d = new Date(utc);
  if (Number.isNaN(d.getTime())) return "—";
  return new Intl.DateTimeFormat(language, { dateStyle: "medium", timeStyle: "short", timeZone: zone }).format(d);
}
