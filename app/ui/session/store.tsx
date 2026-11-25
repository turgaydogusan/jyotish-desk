// Açık harita (oturum). Harita Rust'ta hesaplanmış gelir (`OpenedClient`); arayüz yalnızca
// çizer (spec §4). Yeni harita açılınca görünüm deposu o haritanın dasha yoluna geçer;
// önceki haritanın bekleyen görünüm kaydı önce ona yazılır (carried rule C5).
import { createContext, useCallback, useContext, useMemo, useRef, useState, type ReactNode } from "react";
import { api } from "../api";
import type { CommandError, OpenedClient } from "../bindings";
import { storeWarningNotices, useNotices } from "../shell/notices";
import { useViewStore } from "../view/store";

export interface Session {
  opened: OpenedClient | null;
  /**
   * Kayıtlı veride liste görünümlerini (kenar çubuğu, palet) etkileyen her değişiklikte artar:
   * açılış, silme, etiket değişikliği. Listeler bu sayaca göre yeniden okunur.
   */
  revision: number;
  /**
   * Komut dönüşünü (`create_client`, `add_chart`, `update_chart`, `open_client`) açar. Açılış, açma
   * hatası ve önceki haritanın bildirimleri (`boot`, `client`, `store:*`) kalkar, yeni haritanın uyarıları gösterilir.
   */
  open(opened: OpenedClient): void;
  /**
   * `open_client`: danışanı `chartId` verilmezse son haritasıyla açar. Hata dönerse açık harita
   * değişmez. Art arda iki istekte yalnızca sonuncusunun yanıtı açılır.
   */
  openClient(clientId: number, chartId?: number | null): Promise<CommandError | null>;
  /** Açık haritayı kapatır (ör. danışan silindi): pano boş duruma döner. */
  close(): void;
  /** Açık haritanın etiketleri kaydedildi (`set_chart_tags` dönüşü). */
  setTags(chartId: number, tags: string[]): void;
  /** Liste yenilemesi gerektiren, açık haritayı değiştirmeyen kayıt (ör. başka harita silindi). */
  touch(): void;
}

const SessionContext = createContext<Session | null>(null);

/**
 * `initial` açılışta geri yüklenen haritadır (`restore_session`); görünüm deposunun ilk
 * modeli aynı haritayla kurulmuş olmalıdır (`initialViewModel(layout, { id, dashaPath })`).
 */
export function SessionProvider({ initial, children }: { initial: OpenedClient | null; children: ReactNode }) {
  const [opened, setOpened] = useState(initial);
  const [revision, setRevision] = useState(0);
  const { openChart, closeChart } = useViewStore();
  const { push, dismissWhere } = useNotices();
  const seq = useRef(0);

  const touch = useCallback(() => setRevision((r) => r + 1), []);

  const open = useCallback(
    (o: OpenedClient) => {
      seq.current += 1;
      // Eski açılış hatası ("client") da kalkar: yeni harita açıldı.
      dismissWhere((n) => n.id === "boot" || n.id === "client" || n.id.startsWith("store:"));
      for (const n of storeWarningNotices(o.warnings)) push(n);
      openChart(o.chart_record.id, o.view_state.dasha_expanded_path);
      setOpened(o);
      touch();
    },
    [openChart, dismissWhere, push, touch],
  );

  const openClient = useCallback(
    async (clientId: number, chartId: number | null = null) => {
      const mine = ++seq.current;
      const r = await api.openClient(clientId, chartId);
      if (mine !== seq.current) return null;
      if (r.status === "error") return r.error;
      open(r.data);
      return null;
    },
    [open],
  );

  const close = useCallback(() => {
    seq.current += 1;
    dismissWhere((n) => n.id.startsWith("store:"));
    closeChart();
    setOpened(null);
    touch();
  }, [closeChart, dismissWhere, touch]);

  const setTags = useCallback(
    (chartId: number, tags: string[]) => {
      setOpened((o) => (o && o.chart_record.id === chartId ? { ...o, chart_record: { ...o.chart_record, tags } } : o));
      touch();
    },
    [touch],
  );

  const value = useMemo(() => ({ opened, revision, open, openClient, close, setTags, touch }), [opened, revision, open, openClient, close, setTags, touch]);
  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
}

export function useSession(): Session {
  const s = useContext(SessionContext);
  if (s === null) throw new Error("useSession must be used inside <SessionProvider>");
  return s;
}
