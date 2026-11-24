// Geçici stub: Task 3'te tauri-specta çıktısıyla değiştirilir.
import { invoke } from "@tauri-apps/api/core";

export type EngineStatus = { seVersion: string };
export type PlanetRow = { body: string; lon: number; retrograde: boolean };
export type CommandError = { code: string; params: Partial<{ [key in string]: string }> };
export type Result<T, E> = { status: "ok"; data: T } | { status: "error"; error: E };

async function wrap<T>(p: Promise<T>): Promise<Result<T, CommandError>> {
  try {
    return { status: "ok", data: await p };
  } catch (e) {
    return { status: "error", error: e as CommandError };
  }
}

export const commands = {
  engineStatus: () => wrap(invoke<EngineStatus>("engine_status")),
  planetLongitudes: (jdUt: number) => wrap(invoke<PlanetRow[]>("planet_longitudes", { jdUt })),
};
