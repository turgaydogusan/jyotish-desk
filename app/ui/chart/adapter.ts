// PlanetRow → ChartData. Yalnızca gösterim eşlemesi yapılır (burç = floor(lon / 30),
// burç içi derece); astrolojik hesap değildir.
import type { PlanetRow } from "../bindings";
import { isGrahaId, type ChartData } from "./types";

export function signAndDegree(lon: number): { sign: number; degInSign: number } {
  const norm = ((lon % 360) + 360) % 360;
  const sign = Math.floor(norm / 30);
  return { sign, degInSign: norm - sign * 30 };
}

/** Lagna henüz backend'den gelmediği için `ascSign` her zaman `null`. */
export function chartDataFromPlanetRows(rows: readonly PlanetRow[]): ChartData {
  const planets: ChartData["planets"] = [];
  for (const r of rows) {
    // Sonlu olmayan boylam (NaN, ±∞) gösterilemez; açıkça atlanır.
    if (r.lon === null || !Number.isFinite(r.lon) || !isGrahaId(r.body)) continue;
    const pos = signAndDegree(r.lon);
    if (!Number.isInteger(pos.sign) || pos.sign < 0 || pos.sign > 11) continue;
    planets.push({ id: r.body, ...pos, retrograde: r.retrograde });
  }
  return { ascSign: null, planets };
}
