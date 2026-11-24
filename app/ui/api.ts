// bindings.ts'in ince sarmalayıcısı: IPC hataları da arayüze `{code, params}` olarak ulaşır.
// Üretilen `typedError` Tauri'nin düz string reddini (bilinmeyen komut, argüman çözme,
// ACL, yönetilmeyen state) CommandError diye işaretler ve `Error`'ları yeniden fırlatır.
import { commands, type CommandError } from "./bindings";

export type Result<T> = { status: "ok"; data: T } | { status: "error"; error: CommandError };

function isCommandError(e: unknown): e is CommandError {
  return typeof e === "object" && e !== null && typeof (e as { code?: unknown }).code === "string";
}

function ipcError(e: unknown): CommandError {
  return { code: "ipc", params: { detail: String(e) } };
}

async function wrap<T>(call: () => Promise<Result<T>>): Promise<Result<T>> {
  try {
    const r = await call();
    if (r.status === "ok" || isCommandError(r.error)) return r;
    return { status: "error", error: ipcError(r.error) };
  } catch (e) {
    return { status: "error", error: ipcError(e) };
  }
}

export const api = {
  engineStatus: () => wrap(() => commands.engineStatus()),
  planetLongitudes: (jdUt: number | null) => wrap(() => commands.planetLongitudes(jdUt)),
  printWindow: () => wrap(() => commands.printWindow()),
};
