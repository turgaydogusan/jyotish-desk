// Genel kısayollar yazı alanlarında tetiklenmez (ör. ad girerken Cmd+Shift+N).

/** Olay hedefi metin girilen bir alan mı: input, textarea, select ya da contenteditable. */
export function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) {
    return true;
  }
  return target.isContentEditable || target.closest("[contenteditable]:not([contenteditable='false'])") !== null;
}
