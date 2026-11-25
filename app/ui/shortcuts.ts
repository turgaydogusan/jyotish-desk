// Klavye kısayollarının tek kaynağı (spec §4 "tam klavye"). Genel komutlar burada eşlenir;
// yardım penceresi (`?`) listeyi buradan çizer. Panel içi tuşlar (Alt+ok, ağaç gezintisi)
// odaktaki öğede işlenir ve burada yalnızca listelenir.
import { isEditableTarget } from "./keyboard";

export type Command =
  | { id: "selectLayout"; index: number }
  | { id: "toggleNotes" }
  | { id: "newChart" }
  | { id: "editChart" }
  | { id: "help" };

export interface ShortcutInfo {
  /** i18n anahtarı: `shortcuts.<id>`. */
  id: "selectLayout" | "toggleNotes" | "newChart" | "editChart" | "help" | "escape" | "movePanel" | "resizePanel" | "tree";
  keys: string;
}

export const SHORTCUT_LIST: readonly ShortcutInfo[] = [
  { id: "newChart", keys: "⌘N" },
  { id: "editChart", keys: "⌘E" },
  { id: "selectLayout", keys: "⌘1 … ⌘9" },
  { id: "toggleNotes", keys: "⌘⇧N" },
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

/**
 * Klavye olayı → genel komut. Yazı alanlarında, tuş tekrarında, IME bileşiminde ve modal
 * açıkken `null`. Rakamlar fiziksel tuştan (`code`) okunur: AZERTY'de `key` rakam değildir.
 */
export function matchShortcut(e: KeyboardEvent): Command | null {
  if (e.repeat || e.isComposing || isEditableTarget(e.target) || modalOpen()) return null;
  if (e.ctrlKey || e.altKey) return null;
  if (e.metaKey) {
    if (e.shiftKey) return e.key.toLowerCase() === "n" ? { id: "toggleNotes" } : null;
    if (/^Digit[1-9]$/.test(e.code)) return { id: "selectLayout", index: Number(e.code.slice(5)) - 1 };
    if (e.key.toLowerCase() === "n") return { id: "newChart" };
    if (e.key.toLowerCase() === "e") return { id: "editChart" };
    return null;
  }
  return e.key === "?" ? { id: "help" } : null;
}
