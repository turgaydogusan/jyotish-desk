// Dasa sistemi seçimi (ileri modüller spec'i §6): dasa ağacı ve dasa şeridi panellerinin
// `system` ayarı. Ayar panelindir ve düzenle kaydedilir; tanınmayan değer Vimshottari sayılır.
import { useId } from "react";
import { useTranslation } from "react-i18next";

export const DASHA_SYSTEMS = ["vimshottari", "chara"] as const;
export type DashaSystem = (typeof DASHA_SYSTEMS)[number];

export function dashaSystem(settings: Record<string, unknown>): DashaSystem {
  return settings.system === "chara" ? "chara" : "vimshottari";
}

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
