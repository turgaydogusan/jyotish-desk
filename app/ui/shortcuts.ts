// Klavye kısayollarının tek kaynağı (spec §4 "tam klavye"). Genel komutlar burada eşlenir;
// yardım penceresi (`?`) listeyi buradan çizer. Panel içi tuşlar (Alt+ok, ağaç gezintisi)
// odaktaki öğede işlenir ve burada yalnızca listelenir.
import { isTextEntry } from "./keyboard";

export type Command =
  | { id: "selectLayout"; index: number }
  | { id: "toggleNotes" }
  | { id: "newChart" }
  | { id: "newPrashna" }
  | { id: "chartSearch" }
  | { id: "editChart" }
  | { id: "settings" }
  | { id: "print" }
  | { id: "palette" }
  | { id: "sidebar" }
  | { id: "help" };

export interface ShortcutInfo {
  /** i18n anahtarı: `shortcuts.<id>`. */
  id: "palette" | "sidebar" | "selectLayout" | "toggleNotes" | "newChart" | "newPrashna" | "chartSearch" | "editChart" | "settings" | "print" | "help" | "escape" | "movePanel" | "resizePanel" | "tree";
  keys: string;
}

export const SHORTCUT_LIST: readonly ShortcutInfo[] = [
  { id: "palette", keys: "⌘K" },
  { id: "sidebar", keys: "⌘\\" },
  { id: "newChart", keys: "⌘N" },
  { id: "editChart", keys: "⌘E" },
  { id: "settings", keys: "⌘," },
  { id: "print", keys: "⌘P" },
  { id: "selectLayout", keys: "⌘1 … ⌘9" },
  { id: "toggleNotes", keys: "⌘⇧N" },
  { id: "newPrashna", keys: "⌘⇧P" },
  { id: "chartSearch", keys: "⌘⇧F" },
  { id: "help", keys: "?" },
  { id: "escape", keys: "Esc" },
  { id: "movePanel", keys: "⌥ ←↑→↓" },
  { id: "resizePanel", keys: "⌥⇧ ←↑→↓" },
  { id: "tree", keys: "↑ ↓ ← → Home End ↵" },
];

/** Açık bir modal pencere varken genel kısayollar kapalıdır (arkadaki pano değişmez). */
export function modalOpen(): boolean {
  return document.querySelector('[aria-modal="true"]') !== null;
}

/** Baskı önizlemesi açık mı (`PrintView` kökü). Önizlemede pano yoktur: düzen kısayolları kapalıdır. */
export function printPreviewOpen(): boolean {
  return document.querySelector("[data-print-preview]") !== null;
}

/**
 * Klavye olayı → genel komut. Tuş tekrarında, IME bileşiminde (WebKit'in 229 kodu dahil) ve modal açıkken `null`;
 * ⌘ kısayolları yazı alanında da çalışır, değiştiricisiz `?` yazı alanında yazıdır. Rakamlar ve ters bölü fiziksel tuştan (`code`) okunur: AZERTY'de `key`
 * rakam değildir, Türkçe Q klavyede ters bölü tek tuşla yazılmaz.
 */
export function matchShortcut(e: KeyboardEvent): Command | null {
  if (e.repeat || e.isComposing || e.keyCode === 229 || modalOpen()) return null;
  if (e.ctrlKey || e.altKey) return null;
  if (e.metaKey) {
    if (e.shiftKey) {
      const shifted = e.key.toLowerCase();
      // ⌘⇧P "Yeni prashna"dır (ileri modüller spec'i §11); ⌘P baskıdır. ⌘⇧F "Haritalarda ara"dır
      // (§12).
      if (shifted === "f") return { id: "chartSearch" };
      return shifted === "n" ? { id: "toggleNotes" } : shifted === "p" ? { id: "newPrashna" } : null;
    }
    // Önizlemenin ardındaki düzen görünmeden değişmesin (Plan 04 incelemesi M4).
    if (/^Digit[1-9]$/.test(e.code)) return printPreviewOpen() ? null : { id: "selectLayout", index: Number(e.code.slice(5)) - 1 };
    if (e.code === "Backslash" || e.key === "\\") return { id: "sidebar" };
    if (e.code === "Comma" || e.key === ",") return { id: "settings" };
    const k = e.key.toLowerCase();
    if (k === "k") return { id: "palette" };
    if (k === "n") return { id: "newChart" };
    if (k === "e") return { id: "editChart" };
    if (k === "p") return { id: "print" };
    return null;
  }
  // Değiştiricisiz tuş yazıdır: metin girişinde komut sayılmaz. ⌘ kısayolları her yerde çalışır.
  return e.key === "?" && !isTextEntry(e.target) ? { id: "help" } : null;
}
