// `{code, params}` → yerelleştirilmiş metin (spec §7). Arka uçtan gelen parametreler
// i18next seçeneklerini (dil, çoğul, bağlam…) değiştiremez. Parametreler loglanmaz.
import type { i18n as I18n, TFunction } from "i18next";
import type { CommandError } from "../bindings";

const RESERVED_OPTIONS = new Set(["count", "context", "lng", "lngs", "defaultValue", "ns", "replace", "returnObjects", "joinArrays", "postProcess", "interpolation", "fallbackLng"]);

function safeParams(params: Record<string, unknown>): Record<string, unknown> {
  return Object.fromEntries(Object.entries(params).filter(([k]) => !RESERVED_OPTIONS.has(k)));
}

export function errorMessage(t: TFunction, i18n: I18n, error: CommandError): string {
  const key = `errors.${error.code}`;
  return i18n.exists(key) ? t(key, safeParams(error.params)) : t("errors.unknown", { code: error.code });
}
