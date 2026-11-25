// Kenar çubuğunun açık/kapalı durumu (Plan 10 kapsam dışı 16, Plan 12 D11). Tema ve dil gibi
// bir arayüz tercihidir; danışan verisi içermez. Okunamazsa kenar çubuğu açıktır.
export const SIDEBAR_STORAGE_KEY = "jyotish-desk.sidebar.v1";

export function loadSidebarOpen(storage?: Storage): boolean {
  try {
    return (storage ?? window.localStorage).getItem(SIDEBAR_STORAGE_KEY) !== "closed";
  } catch {
    return true;
  }
}

/** Yazılamazsa durum yalnızca bu oturumda geçerli olur; hata loglanmaz (C1). */
export function saveSidebarOpen(open: boolean, storage?: Storage): void {
  try {
    (storage ?? window.localStorage).setItem(SIDEBAR_STORAGE_KEY, open ? "open" : "closed");
  } catch {
    // Tercih kaybı zararsızdır.
  }
}
