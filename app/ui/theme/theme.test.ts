import {
  applyTheme,
  loadThemePref,
  resolveTheme,
  saveThemePref,
  THEME_STORAGE_KEY,
  type DarkMediaQuery,
} from "./theme";

function fakeMedia(matches: boolean) {
  const listeners = new Set<() => void>();
  const media = {
    matches,
    addEventListener: (_: "change", l: () => void) => listeners.add(l),
    removeEventListener: (_: "change", l: () => void) => listeners.delete(l),
  };
  return {
    media: media as DarkMediaQuery,
    listeners,
    set(v: boolean) {
      media.matches = v;
      listeners.forEach((l) => l());
    },
  };
}

test("resolveTheme honours explicit choices and follows the system otherwise", () => {
  expect(resolveTheme("light", true)).toBe("light");
  expect(resolveTheme("dark", false)).toBe("dark");
  expect(resolveTheme("system", true)).toBe("dark");
  expect(resolveTheme("system", false)).toBe("light");
});

test("applyTheme writes data-theme on the root element", () => {
  const root = document.createElement("html");
  const { media } = fakeMedia(true);
  applyTheme("light", root, media);
  expect(root.dataset.theme).toBe("light");
  applyTheme("dark", root, media);
  expect(root.dataset.theme).toBe("dark");
});

test("system preference tracks prefers-color-scheme changes until disposed", () => {
  const root = document.createElement("html");
  const fake = fakeMedia(false);
  const dispose = applyTheme("system", root, fake.media);
  expect(root.dataset.theme).toBe("light");
  fake.set(true);
  expect(root.dataset.theme).toBe("dark");
  dispose();
  expect(fake.listeners.size).toBe(0);
  fake.set(false);
  expect(root.dataset.theme).toBe("dark");
});

test("explicit preferences do not subscribe to the media query", () => {
  const fake = fakeMedia(false);
  applyTheme("dark", document.createElement("html"), fake.media);
  expect(fake.listeners.size).toBe(0);
});

test("without matchMedia, system resolves to light", () => {
  const root = document.createElement("html");
  applyTheme("system", root, null);
  expect(root.dataset.theme).toBe("light");
});

test("theme preference round-trips through storage and defaults to system", () => {
  localStorage.clear();
  expect(loadThemePref()).toBe("system");
  saveThemePref("dark");
  expect(localStorage.getItem(THEME_STORAGE_KEY)).toBe("dark");
  expect(loadThemePref()).toBe("dark");
  localStorage.setItem(THEME_STORAGE_KEY, "sepia");
  expect(loadThemePref()).toBe("system");
});
