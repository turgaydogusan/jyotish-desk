// Shadbala ve Bhava bala panellerinin ortak parçaları (ileri modüller spec'i §7 "Arayüz"):
// birim ayarı, sayı biçimi ve "rapor yok" açıklaması. Değerler Rust'tan virupa olarak gelir;
// rupa gösterimi yalnızca birim çevirisidir (60 virupa = 1 rupa), astrolojik hesap değildir (C4).
import { useId, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { OpenedClient } from "../bindings";
import type { Panel } from "../layout/model";
import { timeUnknown } from "../session/birth";
import { useLayoutStore } from "../view/store";

export const BALA_UNITS = ["virupa", "rupa"] as const;
export type BalaUnit = (typeof BALA_UNITS)[number];
/** Rust `VIRUPAS_PER_RUPA`. */
export const VIRUPAS_PER_RUPA = 60;
const DASH = "—";

/** Panelin birim ayarı; tanınmayan değer ve eksik anahtar virupadır. */
export function balaUnit(settings: Record<string, unknown>): BalaUnit {
  return settings.unit === "rupa" ? "rupa" : "virupa";
}

/** İşaretli sabit ondalık: eksi işareti tipografik (−); boş ya da sonlu olmayan değer çizgi. */
export function formatFixed(value: number | null, digits: number): string {
  if (value === null || !Number.isFinite(value)) return DASH;
  const text = Math.abs(value).toFixed(digits);
  // "−0.0" yazılmaz.
  return value < 0 && Number(text) !== 0 ? `−${text}` : text;
}

/** Virupa değeri seçili birimde: virupa bir ondalık, rupa iki ondalık. */
export function formatBala(virupa: number | null, unit: BalaUnit): string {
  if (virupa === null) return DASH;
  return unit === "rupa" ? formatFixed(virupa / VIRUPAS_PER_RUPA, 2) : formatFixed(virupa, 1);
}

/**
 * Oran, iki ondalık. 1'in altındaki oran yuvarlanınca "1.00" yazılmaz (0,995 ile 1 arası "0.99"
 * olur): hücredeki `<1` işaretiyle çelişmesin. Gösterimdir; oran Rust'tan gelir (C4).
 */
export function formatRatio(ratio: number | null): string {
  const text = formatFixed(ratio, 2);
  return ratio !== null && ratio < 1 && text === "1.00" ? "0.99" : text;
}

/**
 * Tablonun yatay kaydırma kabı. Dar panelde tablo yana kayar; içinde odaklanabilir denetim
 * olmayan tablolar da klavyeyle kaydırılabilsin diye kap odak alır ve adını tablonun
 * başlığından (`labelledBy`: `caption` kimliği) okur.
 */
export function BalaScroll({ labelledBy, children }: { labelledBy: string; children: ReactNode }) {
  return (
    <div className="bala-scroll" role="group" tabIndex={0} aria-labelledby={labelledBy}>
      {children}
    </div>
  );
}

/** Birim seçicisi; değer panelin `unit` ayarıdır ve düzenle birlikte saklanır. */
export function UnitSelect({ panel }: { panel: Panel }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  return (
    <>
      <label htmlFor={id}>{t("bala.unit")}</label>
      <select id={id} value={balaUnit(panel.settings)} onChange={(e) => setPanelSettings(panel.id, { unit: e.target.value })}>
        {BALA_UNITS.map((u) => (
          <option key={u} value={u}>
            {t(`bala.units.${u}`)}
          </option>
        ))}
      </select>
    </>
  );
}

/**
 * Rust rapor döndürmedi (`null`): doğum saati bilinmiyorsa onu, değilse gün doğumunun ya da
 * batımının olmadığını söyler. İkinci durumda harita başlığındaki uyarı rozeti de görünür.
 */
export function BalaUnavailable({ opened }: { opened: OpenedClient }) {
  const { t } = useTranslation();
  return <p className="panel-notice">{t(timeUnknown(opened) ? "bala.timeUnknown" : "bala.noSunrise")}</p>;
}
