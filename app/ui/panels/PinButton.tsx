// Ortak vurgu düğmesi (UI spec §3.4): üzerine gelince ya da odaklanınca geçici vurgu, tıklayınca
// sabitleme. İleri modül panellerindeki graha ve burç adları bununla yazılır.
import type { ReactNode } from "react";
import { useHighlight } from "../highlight/store";
import { sameTarget, type HighlightTarget } from "../highlight/target";

export function PinButton({ target, label, children }: { target: HighlightTarget; /** Kısaltma gösterilirken okunacak tam ad. */ label?: string; children: ReactNode }) {
  const highlight = useHighlight();
  return (
    <button
      type="button"
      className="link-button"
      aria-label={label}
      aria-pressed={sameTarget(highlight.pinned, target)}
      onClick={() => highlight.togglePin(target)}
      onMouseEnter={() => highlight.hover(target)}
      onMouseLeave={() => highlight.hover(null)}
      onFocus={() => highlight.hover(target)}
      onBlur={() => highlight.hover(null)}
    >
      {children}
    </button>
  );
}
