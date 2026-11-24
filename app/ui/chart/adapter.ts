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
    if (r.lon === null || !isGrahaId(r.body)) continue;
    planets.push({ id: r.body, ...signAndDegree(r.lon), retrograde: r.retrograde });
  }
  return { ascSign: null, planets };
}
