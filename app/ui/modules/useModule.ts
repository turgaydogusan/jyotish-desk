// Bir modül komutunun sonucunu panel açıkken ister (ileri modüller spec'i K1). `key` sonucun
// ait olduğu girdidir (`inputKey`): değişince eski sonuç gösterilmez ve geç gelen eski yanıt
// atılır. Hata "yeniden dene" ile yeniden istenir. Paneller yükleme evresini
// `usePanelLoadPhase(loading)` ile çizer (UI spec §5).
import { useCallback, useEffect, useRef, useState } from "react";
import type { Result } from "../api";
import type { CommandError } from "../bindings";

export interface ModuleState<T> {
  data: T | null;
  error: CommandError | null;
  loading: boolean;
  retry(): void;
}

/** `key === null`: açık harita yok, komut çağrılmaz. */
export function useModule<T>(key: string | null, fetch: () => Promise<Result<T>>): ModuleState<T> {
  const [attempt, setAttempt] = useState(0);
  const [settled, setSettled] = useState<{ key: string; attempt: number; result: Result<T> } | null>(null);
  const latest = useRef(fetch);
  latest.current = fetch;
  // Süren istek: aynı girdi ve deneme için effect yeniden koşarsa (StrictMode'un geliştirme
  // sırasındaki ikinci koşusu) komut yeniden çağrılmaz, süren yanıt beklenir.
  const inflight = useRef<{ id: string; promise: Promise<Result<T>> } | null>(null);

  useEffect(() => {
    if (key === null) return;
    const id = `${attempt}:${key}`;
    if (inflight.current?.id !== id) inflight.current = { id, promise: latest.current() };
    let live = true;
    void inflight.current.promise.then(
      (result) => {
        if (live) setSettled({ key, attempt, result });
      },
      // `api` sarmalayıcıları reddetmez; yine de reddeden bir istek sonsuz yükleme bırakmaz.
      (e: unknown) => {
        if (live) setSettled({ key, attempt, result: { status: "error", error: { code: "ipc", params: { detail: String(e) } } } });
      },
    );
    return () => {
      live = false;
    };
  }, [key, attempt]);

  const retry = useCallback(() => setAttempt((n) => n + 1), []);
  const result = settled !== null && settled.key === key && settled.attempt === attempt ? settled.result : null;
  return {
    data: result?.status === "ok" ? result.data : null,
    error: result?.status === "error" ? result.error : null,
    loading: key !== null && result === null,
    retry,
  };
}

/**
 * Yeniden okuma sürerken gösterilecek veri: `state.data` yoksa ve istek sürüyorsa aynı `scope`
 * için son okunan veri. Böylece içerik (liste, tablo, odaktaki denetim) yeni yanıt gelene dek
 * yerinde durur; hata ya da başka bir `scope` eski veriyi göstermez. `scope === null` unutur.
 */
export function useLastData<T>(state: ModuleState<T>, scope: string | null): T | null {
  const last = useRef<{ scope: string; data: T } | null>(null);
  if (scope === null || (last.current !== null && last.current.scope !== scope)) last.current = null;
  if (scope !== null && state.data !== null) last.current = { scope, data: state.data };
  if (state.data !== null) return state.data;
  return state.loading ? (last.current?.data ?? null) : null;
}
