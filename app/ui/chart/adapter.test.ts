import { chartDataFromPlanetRows, signAndDegree } from "./adapter";

test("sign is floor(lon / 30) and degInSign the remainder", () => {
  expect(signAndDegree(0)).toEqual({ sign: 0, degInSign: 0 });
  expect(signAndDegree(256.5)).toEqual({ sign: 8, degInSign: 16.5 });
  expect(signAndDegree(359.75)).toEqual({ sign: 11, degInSign: 29.75 });
  expect(signAndDegree(360)).toEqual({ sign: 0, degInSign: 0 });
  expect(signAndDegree(-30)).toEqual({ sign: 11, degInSign: 0 });
});

test("rows map to chart planets; null longitudes and unknown bodies are skipped", () => {
  const data = chartDataFromPlanetRows([
    { body: "Sun", lon: 256.5, retrograde: false },
    { body: "Saturn", lon: 10.25, retrograde: true },
    { body: "Mars", lon: null, retrograde: false },
    { body: "Pluto", lon: 100, retrograde: false },
    { body: "Rahu", lon: 100, retrograde: true },
    { body: "Ketu", lon: 280, retrograde: true },
  ]);
  expect(data.ascSign).toBeNull();
  expect(data.planets).toEqual([
    { id: "Sun", sign: 8, degInSign: 16.5, retrograde: false },
    { id: "Saturn", sign: 0, degInSign: 10.25, retrograde: true },
    { id: "Rahu", sign: 3, degInSign: 10, retrograde: true },
    { id: "Ketu", sign: 9, degInSign: 10, retrograde: true },
  ]);
});

test("non-finite longitudes are skipped explicitly", () => {
  const data = chartDataFromPlanetRows([
    { body: "Sun", lon: Number.NaN, retrograde: false },
    { body: "Moon", lon: Number.POSITIVE_INFINITY, retrograde: false },
    { body: "Mars", lon: Number.NEGATIVE_INFINITY, retrograde: false },
    { body: "Venus", lon: 45, retrograde: false },
  ]);
  expect(data.planets).toEqual([{ id: "Venus", sign: 1, degInSign: 15, retrograde: false }]);
});
