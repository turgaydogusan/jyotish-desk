// Not kayıtlarının tek sırası. Otomatik kayıtlar sırayla gider (aynı notun eski metni yenisinin
// üstüne yazılmaz) ve not listesi okunmadan önce bekleyen kayıtlar biter: panel kapanırken
// gönderilen son kayıt, yeniden açılan panelin okuduğu listede görünür.
let chain: Promise<unknown> = Promise.resolve();

export function enqueueNoteSave<T>(save: () => Promise<T>): Promise<T> {
  const next = chain.then(save, save);
  chain = next.catch(() => undefined);
  return next;
}

/** Bekleyen bütün not kayıtları bitince çözülür. */
export function notesIdle(): Promise<void> {
  return chain.then(() => undefined);
}
