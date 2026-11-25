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
  if (!i18n.exists(key)) return t("errors.unknown", { code: error.code });
  const params = safeParams(error.params);
  // Kararlı kodlu parametreler arayüz dilinde gösterilir.
  if (error.code === "atlas_unavailable") {
    const reason = String(error.params.reason ?? "missing");
    params.reason = i18n.exists(`atlasReason.${reason}`) ? t(`atlasReason.${reason}`) : reason;
  }
  if (error.code === "file_io") {
    const detail = String(error.params.detail ?? "other");
    params.detail = i18n.exists(`errors.fileIoDetail.${detail}`) ? t(`errors.fileIoDetail.${detail}`) : detail;
  }
  if (error.code === "update_check_failed" || error.code === "update_install_failed") {
    const reason = String(error.params.reason ?? "other");
    params.reason = t(i18n.exists(`updateReason.${reason}`) ? `updateReason.${reason}` : "updateReason.other");
  }
  return t(key, params);
}
