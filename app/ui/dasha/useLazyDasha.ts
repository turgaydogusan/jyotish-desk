// 4. ve 5. seviye dasha dönemleri istenerek yüklenir (`dasha_children`). Önbellek harita ve
// tohuma bağlıdır: başka harita açılınca ya da doğum verisi düzenlenince (aynı kimlik, yeni
// tohum) sıfırlanır; eski anahtarın geç gelen yanıtı atılır.
// Hata girdisi "yeniden dene" ile silinir ve istek tekrarlanır.
import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api";
import type { CommandError, DashaPeriod, DashaSeed, Graha } from "../bindings";
import { MAX_DASHA_LEVEL, PRELOADED_LEVEL } from "./tree";

export type LazyEntry = { status: "ok"; periods: DashaPeriod[] } | { status: "error"; error: CommandError };

export const pathKey = (path: readonly Graha[]) => path.join(",");

/** Açık yolun istenerek yüklenen önekleri: açık 3. ve 4. seviye düğümler. */
export function lazyPaths(open: readonly Graha[]): Graha[][] {
  const out: Graha[][] = [];
  for (let n = PRELOADED_LEVEL; n <= Math.min(open.length, MAX_DASHA_LEVEL - 1); n++) out.push(open.slice(0, n));
  return out;
}

/** Önbellek anahtarı: harita kimliği ve tohum (`update_chart` kimliği korur, tohumu değiştirir). */
export const cacheKeyOf = (chartId: number, seed: DashaSeed) => `${chartId}:${seed.birth_jd_ut}:${seed.moon_sidereal_lon}:${seed.year_days}`;

export function useLazyDasha(chartId: number, seed: DashaSeed, open: readonly Graha[]) {
  const cacheKey = cacheKeyOf(chartId, seed);
  const [state, setState] = useState<{ cacheKey: string; entries: Record<string, LazyEntry> }>({ cacheKey, entries: {} });
  if (state.cacheKey !== cacheKey) setState({ cacheKey, entries: {} });
  const entries = state.cacheKey === cacheKey ? state.entries : {};
  const inflight = useRef(new Set<string>());

  const needed = lazyPaths(open).filter((p) => entries[pathKey(p)] === undefined);
  const neededKey = needed.map(pathKey).join("|");

  useEffect(() => {
    for (const p of needed) {
      const key = `${cacheKey}:${pathKey(p)}`;
      if (inflight.current.has(key)) continue;
      inflight.current.add(key);
      void api.dashaChildren(seed, p).then((r) => {
        inflight.current.delete(key);
        const entry: LazyEntry = r.status === "ok" ? { status: "ok", periods: r.data } : { status: "error", error: r.error };
        setState((s) => (s.cacheKey === cacheKey ? { cacheKey, entries: { ...s.entries, [pathKey(p)]: entry } } : s));
      });
    }
    // `needed`, `neededKey` ile; `seed`, `cacheKey` ile temsil edilir.
  }, [neededKey, cacheKey]);

  const retry = useCallback((path: readonly Graha[]) => {
    setState((s) => {
      const next = { ...s.entries };
      delete next[pathKey(path)];
      return { ...s, entries: next };
    });
  }, []);

  return { entries, retry, loading: needed.length > 0 };
}
