import { expect, test } from "vitest";
import type { CalcSettings } from "../bindings";
import { fixture } from "../test/fixture";
import { ayanamsaKey, parseOrb, sameValue, settingsDiff } from "./model";

const base = (): CalcSettings => fixture().opened.calc_settings;

test("equal settings have no difference, key order does not matter", () => {
  const a = base();
  const b = { ...base(), combustion_orbs: Object.fromEntries(Object.entries(base().combustion_orbs).reverse()) } as CalcSettings;
  expect(settingsDiff(a, b)).toEqual([]);
  expect(sameValue({ x: [1, { y: 2 }] }, { x: [1, { y: 2 }] })).toBe(true);
  expect(sameValue({ x: 1 }, { x: 1, y: undefined })).toBe(false);
});

test("differences are listed in field order", () => {
  const b: CalcSettings = { ...base(), node: "True", ayanamsa: "Raman", combustion_orbs: { ...base().combustion_orbs, moon: 1 } };
  expect(settingsDiff(base(), b)).toEqual(["ayanamsa", "node", "combustion_orbs"]);
});

test("orb text accepts a decimal comma and rejects junk or out of range", () => {
  expect(parseOrb("8")).toBe(8);
  expect(parseOrb(" 8,5 ")).toBe(8.5);
  expect(parseOrb("30")).toBe(30);
  for (const bad of ["", "-1", "31", "8.", "abc", "1e2"]) expect(parseOrb(bad), bad).toBeNull();
});

test("custom ayanamsa is recognised", () => {
  expect(ayanamsaKey("Lahiri")).toBe("Lahiri");
  expect(ayanamsaKey({ Custom: { t0_jd: 2451545, deg: 23.85 } })).toBe("Custom");
});
