// Açık harita (oturum). Harita Rust'ta hesaplanmış gelir (`OpenedClient`); arayüz yalnızca
// çizer (spec §4). Yeni harita açılınca görünüm deposu o haritanın dasha yoluna geçer.
import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";
import type { OpenedClient } from "../bindings";
import { useViewStore } from "../view/store";

export interface Session {
  opened: OpenedClient | null;
  /** `create_client`/`update_chart` dönüşü: haritayı açar, görünümü ona bağlar. */
  open(opened: OpenedClient): void;
}

const SessionContext = createContext<Session | null>(null);

/**
 * `initial` açılışta geri yüklenen haritadır (`restore_session`); görünüm deposunun ilk
 * modeli aynı haritayla kurulmuş olmalıdır (`initialViewModel(layout, { id, dashaPath })`).
 */
export function SessionProvider({ initial, children }: { initial: OpenedClient | null; children: ReactNode }) {
  const [opened, setOpened] = useState(initial);
  const { openChart } = useViewStore();
  const open = useCallback(
    (o: OpenedClient) => {
      openChart(o.chart_record.id, o.view_state.dasha_expanded_path);
      setOpened(o);
    },
    [openChart],
  );
  const value = useMemo(() => ({ opened, open }), [opened, open]);
  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
}

export function useSession(): Session {
  const s = useContext(SessionContext);
  if (s === null) throw new Error("useSession must be used inside <SessionProvider>");
  return s;
}
