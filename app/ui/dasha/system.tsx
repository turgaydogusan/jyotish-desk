// Dasa panellerinin ayarları. Sistem seçimi (ileri modüller spec'i §6): dasa ağacı ve dasa
// şeridi panellerinin `system` ayarı; şeridin `level` ayarı (UI spec §3.3). Ayarlar panelindir
// ve düzenle kaydedilir; tanınmayan sistem Vimshottari, tanınmayan seviye antar sayılır.
import { useId } from "react";
import { useTranslation } from "react-i18next";

export const DASHA_SYSTEMS = ["vimshottari", "chara"] as const;
export type DashaSystem = (typeof DASHA_SYSTEMS)[number];

export function dashaSystem(settings: Record<string, unknown>): DashaSystem {
  return settings.system === "chara" ? "chara" : "vimshottari";
}

export const STRIP_LEVELS = ["maha", "antar", "pratyantar"] as const;
export type StripLevel = (typeof STRIP_LEVELS)[number];

export function stripLevel(settings: Record<string, unknown>): StripLevel {
  return (STRIP_LEVELS as readonly unknown[]).includes(settings.level) ? (settings.level as StripLevel) : "antar";
}

/** Seviyenin üst yol uzunluğu: maha için kök (0), antar için 1, pratyantar için 2. */
export const PARENT_DEPTH: Record<StripLevel, number> = { maha: 0, antar: 1, pratyantar: 2 };

/** "Sistem" seçicisi: görünür etiketli `select`; panelin araç çubuğuna konur. */
export function SystemSelect({ value, onChange }: { value: DashaSystem; onChange(system: DashaSystem): void }) {
  const { t } = useTranslation();
  const id = useId();
  return (
    <>
      <label htmlFor={`${id}-system`}>{t("dasha.system")}</label>
      <select id={`${id}-system`} value={value} onChange={(e) => onChange(dashaSystem({ system: e.target.value }))}>
        {DASHA_SYSTEMS.map((s) => (
          <option key={s} value={s}>
            {t(`dasha.systems.${s}`)}
          </option>
        ))}
      </select>
    </>
  );
}
