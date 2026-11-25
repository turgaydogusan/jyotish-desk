// Açılış: kayıtlı düzen ve son açılan harita tek seferde okunur (spec §4: uygulama kapanıp
// açılınca oturum geri yüklenir). Hiçbir hata açılışı engellemez; geri yükleme hatası
// bildirim bandında gösterilir.
import { api } from "./api";
import type { CommandError, OpenedClient } from "./bindings";
import { loadLayout } from "./view/persistence";

export interface Boot {
  layout: unknown | null;
  /** Kayıtlı düzen okunamadı: hazır düzenler gösterilir, görünüm kayıtları tutulur. */
  layoutError: CommandError | null;
  opened: OpenedClient | null;
  error: CommandError | null;
}

export async function boot(): Promise<Boot> {
  const [loaded, session] = await Promise.all([loadLayout(), api.restoreSession()]);
  const base = { layout: loaded.layout, layoutError: loaded.error };
  if (session.status === "error") return { ...base, opened: null, error: session.error };
  return { ...base, opened: session.data, error: null };
}

export const EMPTY_BOOT: Boot = { layout: null, layoutError: null, opened: null, error: null };
