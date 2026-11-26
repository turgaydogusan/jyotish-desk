// Kabuk düzeyinde odak taşıma: bir bölge gizlenmeden önce odak onun içindeyse kararlı bir
// hedefe geçer. Odak `body`'ye düşmez ve odaktaki alan (etiket, not) çıkışta kaydını yapar;
// kaldırılan öğe odak çıkışı olayı almaz.
import { SIDEBAR_ID } from "../clients/Sidebar";

export const MAIN_ID = "shell-main";

/** Kenar çubuğu gizlenmeden önce: odak içindeyse açma düğmesine geçer. */
export function leaveSidebar() {
  if (document.getElementById(SIDEBAR_ID)?.contains(document.activeElement)) {
    document.querySelector<HTMLElement>(`[aria-controls="${SIDEBAR_ID}"]`)?.focus();
  }
}

/** Bir modül paneli düzenden kaldırılmadan önce: odak o paneldeyse ana bölgeye geçer. */
export function leavePanel(kind: string) {
  const panel = document.activeElement?.closest<HTMLElement>(".panel");
  if (panel?.dataset.kind === kind) document.getElementById(MAIN_ID)?.focus();
}

/** Notlar paneli kapanmadan önce (`⌘⇧N`): odak paneldeyse ana bölgeye geçer. */
export function leaveNotes() {
  if (document.activeElement?.closest(".notes-panel")) document.getElementById(MAIN_ID)?.focus();
}
