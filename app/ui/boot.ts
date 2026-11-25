// Açılış: kayıtlı düzen, tercihler ve son açılan harita tek seferde okunur (spec §4: uygulama
// kapanıp açılınca oturum geri yüklenir). Hiçbir hata açılışı engellemez; hatalar ve tercih
// uyarıları bildirim bandında gösterilir.
import { api } from "./api";
import type { CommandError, OpenedClient, PrefsLoad } from "./bindings";
import { loadLayout } from "./view/persistence";

export interface Boot {
  layout: unknown | null;
  /** Kayıtlı düzen okunamadı: hazır düzenler gösterilir, görünüm kayıtları tutulur. */
  layoutError: CommandError | null;
  /** `get_app_prefs`; okunamadıysa `null` ve `prefsError`. */
  prefs: PrefsLoad | null;
  prefsError: CommandError | null;
  opened: OpenedClient | null;
  error: CommandError | null;
}

export async function boot(): Promise<Boot> {
  const [loaded, prefs, session] = await Promise.all([loadLayout(), api.getAppPrefs(), api.restoreSession()]);
  const base = {
    layout: loaded.layout,
    layoutError: loaded.error,
    prefs: prefs.status === "ok" ? prefs.data : null,
    prefsError: prefs.status === "error" ? prefs.error : null,
  };
  if (session.status === "error") return { ...base, opened: null, error: session.error };
  return { ...base, opened: session.data, error: null };
}

export const EMPTY_BOOT: Boot = { layout: null, layoutError: null, prefs: null, prefsError: null, opened: null, error: null };
