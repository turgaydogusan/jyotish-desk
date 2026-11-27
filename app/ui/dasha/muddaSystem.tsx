// Dasa ağacının sistem seçimi: Vimshottari, chara dasha ve mudda dasa (yol haritası §4.14).
// Mudda dasa yalnızca ağaçta vardır: dasa şeridi transit anına bağlıdır ve `system.tsx`'teki iki
// sistemi sunmayı sürdürür. Ayar panelindir ve düzenle kaydedilir; tanınmayan değer Vimshottari
// sayılır.
import type { TFunction } from "i18next";
import { useId } from "react";
import { useTranslation } from "react-i18next";
import { DASHA_SYSTEMS, dashaSystem, type DashaSystem } from "./system";

export const TREE_SYSTEMS = [...DASHA_SYSTEMS, "mudda"] as const;
export type TreeSystem = DashaSystem | "mudda";

export function treeSystem(settings: Record<string, unknown>): TreeSystem {
  return settings.system === "mudda" ? "mudda" : dashaSystem(settings);
}

/** Sistemin görünen adı. Mudda dasanın adı `varsha` bloğundadır. */
export function treeSystemName(system: TreeSystem, t: TFunction): string {
  return system === "mudda" ? t("varsha.mudda.system") : t(`dasha.systems.${system}`);
}

/** "Sistem" seçicisi: görünür etiketli `select`; dasa ağacının araç çubuğuna konur. */
export function TreeSystemSelect({ value, onChange }: { value: TreeSystem; onChange(system: TreeSystem): void }) {
  const { t } = useTranslation();
  const id = useId();
  return (
    <>
      <label htmlFor={`${id}-system`}>{t("dasha.system")}</label>
      <select id={`${id}-system`} value={value} onChange={(e) => onChange(treeSystem({ system: e.target.value }))}>
        {TREE_SYSTEMS.map((s) => (
          <option key={s} value={s}>
            {treeSystemName(s, t)}
          </option>
        ))}
      </select>
    </>
  );
}
