// Genel tercihler (spec §5 `AppPrefs`): açılışta okunur (`boot`), ayarlar penceresi yazar.
// Yazımlar sıralıdır; başarıdan sonra yeni tercihler geçerli olur.
import { createContext, useCallback, useContext, useMemo, useRef, useState, type ReactNode } from "react";
import { api } from "../api";
import type { AppPrefs, CommandError } from "../bindings";

export interface Prefs {
  /** `null`: açılışta okunamadı; varsayılanla ilgili eylemler kapalıdır. */
  prefs: AppPrefs | null;
  save(next: AppPrefs): Promise<CommandError | null>;
}

const PrefsContext = createContext<Prefs | null>(null);

export function PrefsProvider({ initial, children }: { initial: AppPrefs | null; children: ReactNode }) {
  const [prefs, setPrefs] = useState(initial);
  const queue = useRef<Promise<unknown>>(Promise.resolve());
  const save = useCallback((next: AppPrefs) => {
    const run = queue.current.then(async () => {
      const r = await api.setAppPrefs(next);
      if (r.status === "error") return r.error;
      setPrefs(next);
      return null;
    });
    queue.current = run;
    return run;
  }, []);
  const value = useMemo(() => ({ prefs, save }), [prefs, save]);
  return <PrefsContext.Provider value={value}>{children}</PrefsContext.Provider>;
}

export function usePrefs(): Prefs {
  const p = useContext(PrefsContext);
  if (p === null) throw new Error("usePrefs must be used inside <PrefsProvider>");
  return p;
}
