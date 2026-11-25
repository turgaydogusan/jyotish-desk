import clientsCss from "../clients/clients.css?raw";
import dashboardCss from "../dashboard/dashboard.css?raw";
import shellCss from "../shell/shell.css?raw";
import css from "./tokens.css?raw";

// Spec §2.1 tablosu, birebir.
const LIGHT: Record<string, string> = {
  "--bg": "#FAF9F5",
  "--panel": "#FFFFFF",
  "--line": "#D3D1C7",
  "--ink": "#534AB7",
  "--ink-strong": "#3C3489",
  "--select": "#EEEDFE",
  "--text": "#2C2C2A",
  "--text-2": "#5F5E5A",
  "--warn": "#BA7517",
  "--danger": "#A32D2D",
};
const DARK: Record<string, string> = {
  "--bg": "#1B1830",
  "--panel": "#26215C",
  "--line": "#3C3489",
  "--ink": "#AFA9EC",
  "--ink-strong": "#CECBF6",
  "--select": "#3C3489",
  "--text": "#EEEDFE",
  "--text-2": "#AFA9EC",
  "--warn": "#EF9F27",
  "--danger": "#F09595",
};

/** `marker`dan sonraki ilk `{ … }` bloğundaki `--ad: #hex` bildirimleri. */
function colorTokens(marker: string): Record<string, string> {
  const start = css.indexOf(marker);
  expect(start, marker).toBeGreaterThanOrEqual(0);
  const open = css.indexOf("{", start);
  const body = css.slice(open + 1, css.indexOf("}", open));
  const out: Record<string, string> = {};
  for (const m of body.matchAll(/(--[a-z0-9-]+)\s*:\s*(#[0-9A-Fa-f]{6})\s*;/g)) out[m[1]] = m[2];
  return out;
}

function luminance(hex: string): number {
  const [r, g, b] = [1, 3, 5]
    .map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)
    .map((c) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

test("light tokens match the spec table", () => {
  expect(colorTokens(":root {")).toEqual(LIGHT);
});

test("dark tokens match the spec table", () => {
  expect(colorTokens(':root[data-theme="dark"] {')).toEqual(DARK);
});

test("print forces the light palette with only ink and text", () => {
  const print = colorTokens("@media print");
  const allowed = new Set([LIGHT["--ink"], LIGHT["--text"], "#FFFFFF"]);
  expect(Object.keys(print).sort()).toEqual(Object.keys(LIGHT).sort());
  for (const [name, value] of Object.entries(print)) expect(allowed.has(value), name).toBe(true);
  expect(print["--ink"]).toBe(LIGHT["--ink"]);
  expect(print["--text"]).toBe(LIGHT["--text"]);
});

test("the paper class carries the print palette to the screen preview", () => {
  const paper = colorTokens(".paper {");
  expect(Object.keys(paper)).toHaveLength(10);
  expect(paper).toEqual(colorTokens("@media print"));
});

test("typography: system font, two weights, four sizes, tabular numbers, 2px ink focus ring", () => {
  expect(css).toContain("-apple-system");
  expect(css).toMatch(/--fw-regular:\s*400;/);
  expect(css).toMatch(/--fw-medium:\s*500;/);
  for (const px of [12, 13, 15, 18]) expect(css).toContain(`${px}px;`);
  expect(css).toContain("font-variant-numeric: tabular-nums;");
  expect(css).toMatch(/:focus-visible\s*{\s*outline:\s*2px solid var\(--ink\);/);
  const weights = [...css.matchAll(/font-weight:\s*([^;]+);/g)].map((m) => m[1].trim());
  for (const w of weights) expect(["var(--fw-regular)", "var(--fw-medium)"]).toContain(w);
});

test.each([
  ["light", LIGHT],
  ["dark", DARK],
])("%s text tokens reach WCAG AA (4.5:1) on every surface", (_, t) => {
  for (const fg of ["--text", "--text-2", "--ink-strong", "--ink", "--danger"]) {
    for (const bg of ["--bg", "--panel", "--select"]) {
      expect(contrast(t[fg], t[bg]), `${fg} on ${bg}`).toBeGreaterThanOrEqual(4.5);
    }
  }
  // Uyarı rengi metin değil rozet zeminidir (rozet metni --text); grafik öğe eşiği 3:1.
  for (const bg of ["--bg", "--panel"]) {
    expect(contrast(t["--warn"], t[bg]), `--warn on ${bg}`).toBeGreaterThanOrEqual(3);
  }
});

test("shared helper classes live in the stylesheets every screen loads", () => {
  expect(css).toMatch(/\.visually-hidden\s*\{/);
  expect(dashboardCss).not.toMatch(/\.visually-hidden\s*\{/);
  expect(shellCss).toMatch(/\.button-danger\s*\{/);
  expect(clientsCss).not.toMatch(/\.button-danger\s*\{/);
});
