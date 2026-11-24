// Şimdiki gökyüzü: lagna ve doğum haritası komutları gelene kadar harita ve graha tablosu
// bu veriyi gösterir. Tek çağrı yapılır, iki panel aynı sonucu paylaşır.
import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { api } from "../api";
import type { CommandError, PlanetRow } from "../bindings";
import { unixMsToJdUt } from "../time";

export type SkyState =
  | { status: "loading" }
  | { status: "ok"; rows: PlanetRow[] }
  | { status: "error"; error: CommandError };

export interface Sky {
  state: SkyState;
  retry(): void;
}

const SkyContext = createContext<Sky | null>(null);

export function SkyProvider({ children, now = Date.now }: { children: ReactNode; now?: () => number }) {
  const nowRef = useRef(now);
  const [attempt, setAttempt] = useState(0);
  const [state, setState] = useState<SkyState>({ status: "loading" });

  useEffect(() => {
    let cancelled = false;
    setState({ status: "loading" });
    void api.planetLongitudes(unixMsToJdUt(nowRef.current())).then((r) => {
      if (cancelled) return;
      setState(r.status === "ok" ? { status: "ok", rows: r.data } : { status: "error", error: r.error });
    });
    return () => {
      cancelled = true;
    };
  }, [attempt]);

  const retry = useCallback(() => setAttempt((a) => a + 1), []);
  const value = useMemo(() => ({ state, retry }), [state, retry]);
  return <SkyContext.Provider value={value}>{children}</SkyContext.Provider>;
}

export function useSky(): Sky {
  const sky = useContext(SkyContext);
  if (sky === null) throw new Error("useSky must be used inside <SkyProvider>");
  return sky;
}
