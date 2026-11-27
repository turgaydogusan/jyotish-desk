// Olay aramasını (`event_search`) yürüten kanca (ileri modüller spec'i §12.2). Arama tek bir
// komut çağrısıdır; Rust dilim dilim yürütür ve dilim sınırlarında durdurma isteğine bakar. Her
// arama yeni bir kimlik alır: eski aramanın geç gelen yanıtı düşer. "Durdur" Rust'a bildirilir.
// Sonuç yalnızca kendi isteğiyle durur: yeni arama başlayınca silinir (liste o an DOM'dan
// kalkar; D24), hatadan sonraki arama da boş başlar. Arayüzde hesap
// yoktur (C4); sonuç saklanmaz (K1). Girdi yer ve açık haritanın doğum verisini taşır: loglanmaz.
import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api";
import type { CommandError, EventSearchSpec, EventSearchView } from "../bindings";
import { nextSearchId } from "../search/ids";

export type EventSearchStatus = "idle" | "running" | "done" | "stopped" | "error";

export interface EventSearchState {
  status: EventSearchStatus;
  view: EventSearchView | null;
  error: CommandError | null;
}

export interface EventSearch extends EventSearchState {
  start(spec: EventSearchSpec): void;
  stop(): void;
}

const IDLE: EventSearchState = { status: "idle", view: null, error: null };

/** `scope` değişince (başka harita) süren arama durdurulur ve sonuç silinir. */
export function useEventSearch(scope: string): EventSearch {
  const [state, setState] = useState<EventSearchState>(IDLE);
  const live = useRef<number | null>(null);

  const cancel = useCallback(() => {
    const id = live.current;
    live.current = null;
    if (id !== null) void api.eventSearchCancel(id);
  }, []);

  // Başka harita ya da panelin kapanması: süren arama durdurulur.
  useEffect(() => cancel, [scope, cancel]);
  const [shownScope, setShownScope] = useState(scope);
  if (shownScope !== scope) {
    setShownScope(scope);
    setState(IDLE);
  }

  const start = useCallback(
    (spec: EventSearchSpec) => {
      cancel();
      const id = nextSearchId();
      live.current = id;
      setState({ ...IDLE, status: "running" });
      void (async () => {
        const r = await api.eventSearch(spec, id);
        if (live.current !== id) return;
        live.current = null;
        if (r.status === "ok") setState({ status: "done", view: r.data, error: null });
        // Başka bir yüzeyin başlattığı arama bunu geçersiz kıldıysa durmuş sayılır.
        else if (r.error.code === "search_stale") setState({ ...IDLE, status: "stopped" });
        else setState({ status: "error", view: null, error: r.error });
      })();
    },
    [cancel],
  );

  const stop = useCallback(() => {
    if (live.current === null) return;
    cancel();
    setState({ ...IDLE, status: "stopped" });
  }, [cancel]);

  return { ...state, start, stop };
}
