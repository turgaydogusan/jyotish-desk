// Muhurta aramasını (`muhurta_search`) yürüten kanca (ileri modüller spec'i §10). Arama dilim
// dilim ilerler: her yanıtın imleci bir sonraki isteğe verilir; aramayı durdurmak sonraki dilimi
// istememektir (Rust durum tutmaz). Pencereler geldikçe listeye eklenir ve yeni arama başlayana
// dek yerinde durur. Arayüzde hesap yoktur (C4); sonuç saklanmaz (K1). Sorgu yer ve doğum Ay'ı
// taşır: loglanmaz.
import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api";
import type { CalcSettings, ChartWarning, CommandError, MuhurtaPageCursor, MuhurtaQuery, MuhurtaSearchView, MuhurtaWindowView } from "../bindings";

export type SearchStatus = "idle" | "running" | "done" | "stopped" | "error";

export interface MuhurtaSearchState {
  status: SearchStatus;
  windows: MuhurtaWindowView[];
  /** Taranan dilim ve toplam; ilk yanıttan önce 0. */
  scanned: number;
  chunks: number;
  /** Taranan aralık (ilk yanıtla gelir). */
  range: Pick<MuhurtaSearchView, "from_jd_ut" | "to_jd_ut" | "from_offset_secs" | "to_offset_secs"> | null;
  warnings: ChartWarning[];
  method: string | null;
  error: CommandError | null;
}

export interface MuhurtaSearch extends MuhurtaSearchState {
  start(query: MuhurtaQuery, settings: CalcSettings): void;
  /** Sonraki dilimi istemez; gelmiş pencereler durur. */
  stop(): void;
}

const IDLE: MuhurtaSearchState = { status: "idle", windows: [], scanned: 0, chunks: 0, range: null, warnings: [], method: null, error: null };

/** `scope` değişince (başka harita) süren arama bırakılır ve sonuçlar silinir. */
export function useMuhurtaSearch(scope: string): MuhurtaSearch {
  const [state, setState] = useState<MuhurtaSearchState>(IDLE);
  // Her arama bir numara alır; eski aramanın geç gelen yanıtı ve sonraki isteği düşer.
  const run = useRef(0);

  // Başka harita ya da panelin kapanması: süren aramanın numarası geçersiz olur (effect
  // temizliğinde; çizim sırasında ref yazılmaz).
  useEffect(() => {
    return () => {
      run.current += 1;
    };
  }, [scope]);
  const [shownScope, setShownScope] = useState(scope);
  if (shownScope !== scope) {
    setShownScope(scope);
    setState(IDLE);
  }

  const start = useCallback((query: MuhurtaQuery, settings: CalcSettings) => {
    const mine = ++run.current;
    setState({ ...IDLE, status: "running" });
    void (async () => {
      let cursor: MuhurtaPageCursor | null = null;
      for (;;) {
        const r = await api.muhurtaSearch(query, settings, cursor);
        if (mine !== run.current) return;
        if (r.status === "error") {
          // Gelmiş pencereler durur; hata ayrıca gösterilir.
          setState((s) => ({ ...s, status: "error", error: r.error }));
          return;
        }
        const page = r.data;
        cursor = page.next;
        setState((s) => ({
          status: page.next === null ? "done" : "running",
          windows: [...s.windows, ...page.windows],
          scanned: page.scanned,
          chunks: page.chunks,
          range: { from_jd_ut: page.from_jd_ut, to_jd_ut: page.to_jd_ut, from_offset_secs: page.from_offset_secs, to_offset_secs: page.to_offset_secs },
          warnings: [...new Set([...s.warnings, ...page.warnings])],
          method: page.method,
          error: null,
        }));
        if (cursor === null) return;
      }
    })();
  }, []);

  const stop = useCallback(() => {
    run.current += 1;
    setState((s) => (s.status === "running" ? { ...s, status: "stopped" } : s));
  }, []);

  return { ...state, start, stop };
}
