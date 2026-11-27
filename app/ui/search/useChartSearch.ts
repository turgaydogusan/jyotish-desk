// Harita aramasını (`chart_search`) yürüten kanca (ileri modüller spec'i §12.1). Arama sayfa
// sayfa ilerler: her yanıtın imleci bir sonraki isteğe verilir; eşleşmeler geldikçe listeye
// eklenir. Her arama yeni bir kimlik alır: eski aramanın geç gelen yanıtı ve sonraki isteği
// düşer (kimlik hem burada hem Rust'ta denetlenir). "Durdur" Rust'a bildirilir; gelmiş
// eşleşmeler ve sayılar yerinde durur. Arayüzde hesap yoktur (C4); sonuç saklanmaz (K1).
// Sorgu ve sonuçlar (adlar, sayılar, kimlikler) loglanmaz.
import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api";
import type { ChartHit, ChartQuery, ChartSearchCursor, CommandError } from "../bindings";
import { nextSearchId } from "./ids";

export type ChartSearchStatus = "idle" | "running" | "done" | "stopped" | "error";

export interface ChartSearchState {
  status: ChartSearchStatus;
  /** Listenin ait olduğu sorgu (başlatıldığı andaki kopya); arama yokken `null`. */
  query: ChartQuery | null;
  hits: ChartHit[];
  /** Taranan ve kapsamdaki harita sayısı; ilk yanıttan önce `total` bilinmez (`null`). */
  scanned: number;
  total: number | null;
  skippedTimeUnknown: number;
  failed: number;
  error: CommandError | null;
}

export interface ChartSearch extends ChartSearchState {
  start(query: ChartQuery): void;
  /** Aramayı durdurur; gelmiş eşleşmeler durur. */
  stop(): void;
  /** Sonuçları siler (form temizlenince). */
  reset(): void;
}

const IDLE: ChartSearchState = { status: "idle", query: null, hits: [], scanned: 0, total: null, skippedTimeUnknown: 0, failed: 0, error: null };

export function useChartSearch(): ChartSearch {
  const [state, setState] = useState<ChartSearchState>(IDLE);
  // Süren aramanın kimliği; `null`: süren arama yok. Yanıt yalnızca kimliği buradakiyle aynıysa
  // işlenir.
  const live = useRef<number | null>(null);

  const cancel = useCallback(() => {
    const id = live.current;
    live.current = null;
    // Rust süren sayfayı dilim sınırında bırakır; yanıtı beklenmez.
    if (id !== null) void api.chartSearchCancel(id);
  }, []);

  // Yüzey kapanırken süren arama durdurulur.
  useEffect(() => cancel, [cancel]);

  const start = useCallback(
    (query: ChartQuery) => {
      cancel();
      const id = nextSearchId();
      live.current = id;
      setState({ ...IDLE, status: "running", query });
      void (async () => {
        let cursor: ChartSearchCursor | null = null;
        for (;;) {
          const r = await api.chartSearch(query, id, cursor);
          if (live.current !== id) return;
          if (r.status === "error") {
            live.current = null;
            // Gelmiş eşleşmeler durur; hata ayrıca gösterilir.
            setState((s) => ({ ...s, status: "error", error: r.error }));
            return;
          }
          const page = r.data;
          cursor = page.next;
          // Imleci olmayan sayfa sonraki sayfayı soramaz: arama biter.
          const finished = page.done || cursor === null;
          if (finished) live.current = null;
          setState((s) => ({
            status: finished ? "done" : "running",
            query,
            hits: [...s.hits, ...page.hits],
            scanned: page.scanned,
            total: page.total,
            skippedTimeUnknown: page.skipped_time_unknown,
            failed: page.failed,
            error: null,
          }));
          if (finished) return;
        }
      })();
    },
    [cancel],
  );

  const stop = useCallback(() => {
    if (live.current === null) return;
    cancel();
    setState((s) => (s.status === "running" ? { ...s, status: "stopped" } : s));
  }, [cancel]);

  const reset = useCallback(() => {
    cancel();
    setState(IDLE);
  }, [cancel]);

  return { ...state, start, stop, reset };
}
