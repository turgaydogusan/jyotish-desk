// Renkler yalnızca tokens.css'te hex olarak bulunur (spec §2.1). Bu test sonradan eklenen
// her bileşeni de kapsar.
const sources = import.meta.glob("/ui/**/*.{ts,tsx,css}", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

// tokens.css belirteçlerin kaynağı; bindings.ts üretilmiş; PrintSpike ve print.css
// M7'de baskı düzenleriyle değişecek spike dosyaları; testler beklenen değer olarak hex içerir.
const EXEMPT = [/^\/ui\/theme\/tokens\.css$/, /^\/ui\/bindings\.ts$/, /^\/ui\/PrintSpike\.tsx$/, /^\/ui\/print\.css$/, /\.test\.tsx?$/];

test("no hex colors outside theme tokens", () => {
  const files = Object.keys(sources).filter((f) => !EXEMPT.some((re) => re.test(f)));
  expect(files.length).toBeGreaterThan(0);
  const offenders = files.filter((f) => /#[0-9a-fA-F]{3,8}\b/.test(sources[f]));
  expect(offenders).toEqual([]);
});
