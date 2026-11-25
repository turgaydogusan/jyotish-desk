// Yazı girilen alanlarda yalnızca değiştiricisiz kısayollar (`?`) yazı sayılır; ⌘ kısayolları
// her yerde çalışır (not yazarken ⌘K, ⌘⇧N). Seçim kutusu, onay kutusu ve radyo düğmesi yazı
// alanı değildir: oralarda `?` da çalışır (Plan 09 incelemesi M6-2, Plan 10 incelemesi MF1).

/** Yazı girmeyen `input` türleri: bunlarda genel kısayollar çalışır. */
const NON_TEXT_INPUTS = new Set(["checkbox", "radio", "button", "submit", "reset", "range", "color", "file", "image"]);

/** Olay hedefi metin girilen bir alan mı: metin türü input, textarea ya da contenteditable. */
export function isTextEntry(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target instanceof HTMLTextAreaElement) return true;
  if (target instanceof HTMLInputElement) return !NON_TEXT_INPUTS.has(target.type);
  return target.isContentEditable || target.closest("[contenteditable]:not([contenteditable='false'])") !== null;
}
