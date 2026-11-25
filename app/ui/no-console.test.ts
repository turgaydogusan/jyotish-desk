// Kişisel veri loglanmaz (spec §4; `CommandError.params` doğum verisi taşıyabilir): arayüz
// kodunda `console` çağrısı yoktur. Testler ve üretilen bağlamalar hariç; tema ve dil tercihi
// (Plan 04) yalnızca localStorage yazma hatasını uyarır, kişisel veri içermez.
const sources = import.meta.glob("/ui/**/*.{ts,tsx}", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

const EXEMPT = [/^\/ui\/bindings\.ts$/, /\.test\.tsx?$/];
const ALLOWED = new Set(["/ui/theme/theme.ts", "/ui/i18n/index.ts"]);

test("no console calls in UI code", () => {
  const files = Object.keys(sources).filter((f) => !EXEMPT.some((re) => re.test(f)));
  expect(files.length).toBeGreaterThan(0);
  const offenders = files.filter((f) => !ALLOWED.has(f) && /\bconsole\s*\./.test(sources[f]));
  expect(offenders).toEqual([]);
});
