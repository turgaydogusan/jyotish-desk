// Ortak vurgu durumu (spec §3.4): oturumluk, düzen durumunun dışında. Üzerine gelme geçici
// vurgudur; tıklama sabitler, aynı öğeye yeniden tıklama ya da Esc kaldırır.
import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { sameTarget, type HighlightTarget } from "./target";

export interface Highlight {
  /** Çizilecek vurgu: üzerine gelinen öğe, yoksa sabitlenen. */
  active: HighlightTarget | null;
  pinned: HighlightTarget | null;
  hover(target: HighlightTarget | null): void;
  togglePin(target: HighlightTarget): void;
  clear(): void;
}

const HighlightContext = createContext<Highlight | null>(null);

export function HighlightProvider({ children }: { children: ReactNode }) {
  const [hovered, setHovered] = useState<HighlightTarget | null>(null);
  const [pinned, setPinned] = useState<HighlightTarget | null>(null);

  const clear = useCallback(() => {
    setHovered(null);
    setPinned(null);
  }, []);
  const togglePin = useCallback((t: HighlightTarget) => setPinned((p) => (sameTarget(p, t) ? null : t)), []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !e.defaultPrevented) clear();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [clear]);

  const value = useMemo<Highlight>(
    () => ({ active: hovered ?? pinned, pinned, hover: setHovered, togglePin, clear }),
    [hovered, pinned, togglePin, clear],
  );
  return <HighlightContext.Provider value={value}>{children}</HighlightContext.Provider>;
}

export function useHighlight(): Highlight {
  const h = useContext(HighlightContext);
  if (h === null) throw new Error("useHighlight must be used inside <HighlightProvider>");
  return h;
}
