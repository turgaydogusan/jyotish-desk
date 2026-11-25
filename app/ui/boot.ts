// Açılış: kayıtlı düzen ve son açılan harita tek seferde okunur (spec §4: uygulama kapanıp
// açılınca oturum geri yüklenir). Hiçbir hata açılışı engellemez; geri yükleme hatası
// bildirim bandında gösterilir.
import { api } from "./api";
import type { CommandError, OpenedClient } from "./bindings";
import { loadLayout } from "./view/persistence";

export interface Boot {
  layout: unknown | null;
  opened: OpenedClient | null;
  error: CommandError | null;
}

export async function boot(): Promise<Boot> {
  const [layout, session] = await Promise.all([loadLayout(), api.restoreSession()]);
  if (session.status === "error") return { layout, opened: null, error: session.error };
  return { layout, opened: session.data, error: null };
}

export const EMPTY_BOOT: Boot = { layout: null, opened: null, error: null };
