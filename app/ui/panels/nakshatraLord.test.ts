import { expect, test } from "vitest";
import { NAKSHATRA_LORDS } from "../bindings";
import { fixture } from "../test/fixture";
import { nakshatraLord } from "./nakshatraLord";

test("the lord is read from the Rust table: nine lords, three times, in the Vimshottari order", () => {
  expect(NAKSHATRA_LORDS).toHaveLength(27);
  expect(NAKSHATRA_LORDS.slice(0, 9)).toEqual(["Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury"]);
  for (let n = 1; n <= 27; n++) expect(nakshatraLord(n)).toBe(NAKSHATRA_LORDS[(n - 1) % 9]);
  expect([nakshatraLord(1), nakshatraLord(4), nakshatraLord(27)]).toEqual(["Ketu", "Moon", "Mercury"]);
});

test("a number outside 1..27 has no lord", () => {
  for (const n of [0, 28, -1, 1.5, Number.NaN, null]) expect(nakshatraLord(n)).toBeNull();
});

test("the table agrees with the lords Rust wrote into the fixture", () => {
  // Navatara satırları ve bhava noktaları lordu Rust'tan taşır: aynı tablo, iki ayrı yoldan.
  for (const row of fixture().natal_extras.navatara.rows) expect(nakshatraLord(row.nakshatra)).toBe(row.lord);
  for (const row of fixture().bhava.table.rows) expect(nakshatraLord(row.start.nakshatra)).toBe(row.start.nakshatra_lord);
});
