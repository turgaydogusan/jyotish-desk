// Güncelleme denetimi tercihi (spec §10: denetim kullanıcı onayıyla yapılır; Plan 13 D1).
// Varsayılan kapalıdır ve yalnızca kullanıcı açınca "auto" yazılır. Okunamayan ya da tanınmayan
// değer kapalı sayılır: onay kaydı yitirilirse uygulama ağa bağlanmaz. Danışan verisi içermez.
export const UPDATE_STORAGE_KEY = "jyotish-desk.updates.v1";

export function loadAutoCheck(storage?: Storage): boolean {
  try {
    return (storage ?? window.localStorage).getItem(UPDATE_STORAGE_KEY) === "auto";
  } catch {
    return false;
  }
}

/** Tercihi yazar; yazılamadıysa `false` döner (çağıran tercihi açık saymaz). Hata loglanmaz (C1). */
export function saveAutoCheck(on: boolean, storage?: Storage): boolean {
  try {
    (storage ?? window.localStorage).setItem(UPDATE_STORAGE_KEY, on ? "auto" : "off");
    return true;
  } catch {
    return false;
  }
}
