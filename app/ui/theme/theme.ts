// Tema tercihi: "system" işletim sisteminin açık/koyu ayarını izler (spec §2.1).
export type ThemePref = "light" | "dark" | "system";
export type ResolvedTheme = "light" | "dark";

export const THEME_PREFS: readonly ThemePref[] = ["light", "dark", "system"];
export const THEME_STORAGE_KEY = "jyotish-desk.theme.v1";
export const DARK_QUERY = "(prefers-color-scheme: dark)";

/** `MediaQueryList`'in kullandığımız kısmı; testlerde sahtesi verilir. */
export interface DarkMediaQuery {
  readonly matches: boolean;
  addEventListener(type: "change", listener: () => void): void;
  removeEventListener(type: "change", listener: () => void): void;
}

export function resolveTheme(pref: ThemePref, prefersDark: boolean): ResolvedTheme {
  if (pref === "system") return prefersDark ? "dark" : "light";
  return pref;
}

function defaultMedia(): DarkMediaQuery | null {
  return typeof window.matchMedia === "function" ? window.matchMedia(DARK_QUERY) : null;
}

/**
 * `root.dataset.theme` değerini yazar. `pref === "system"` iken sistem ayarı değiştikçe
 * günceller. Dönen fonksiyon dinleyiciyi kaldırır.
 */
export function applyTheme(
  pref: ThemePref,
  root: HTMLElement = document.documentElement,
  media: DarkMediaQuery | null = defaultMedia(),
): () => void {
  const update = () => {
    root.dataset.theme = resolveTheme(pref, media?.matches ?? false);
  };
  update();
  if (pref !== "system" || media === null) return () => {};
  media.addEventListener("change", update);
  return () => media.removeEventListener("change", update);
}

export function isThemePref(v: unknown): v is ThemePref {
  return typeof v === "string" && (THEME_PREFS as readonly string[]).includes(v);
}

export function loadThemePref(storage: Storage = window.localStorage): ThemePref {
  try {
    const v = storage.getItem(THEME_STORAGE_KEY);
    return isThemePref(v) ? v : "system";
  } catch {
    return "system";
  }
}

export function saveThemePref(pref: ThemePref, storage: Storage = window.localStorage): void {
  try {
    storage.setItem(THEME_STORAGE_KEY, pref);
  } catch (e) {
    console.warn("theme preference not saved", e);
  }
}
