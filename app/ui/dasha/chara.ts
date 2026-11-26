// Chara dasha (ileri modüller spec'i §6) için arayüz yardımcıları. Dönemler, yön ve yıllar
// Rust'tan hazır gelir (`compute_chara_dasha`, `chara_dasha_children`, `chara_dasha_at`);
// burada hesap yoktur (C4): yalnızca istek, gruplama ve adlandırma.
import type { TFunction } from "i18next";
import { api } from "../api";
import type { CharaDasha, OpenedClient, Sign, SignDashaPath, SignDashaPeriod } from "../bindings";
import { signIndex } from "../chart/adapter";
import { chartInput, inputKey } from "../modules/input";
import { useModule, type ModuleState } from "../modules/useModule";

/** Rust `MAX_SIGN_DASHA_LEVEL`. */
export const MAX_CHARA_LEVEL = 3;
/** `CharaDasha.periods`'un içerdiği en derin seviye; 3. seviye istenerek yüklenir. */
export const CHARA_PRELOADED_LEVEL = 2;

/** Dönemin ağaç anahtarı: `"<çevrim>:<burç>,<burç>…"` (bkz. `PeriodTree`). */
export const charaKey = (cycle: number, signs: readonly Sign[]) => `${cycle}:${signs.join(",")}`;

/** `path` yolunun doğrudan alt dönemleri (boş yol: o çevrimin dashaları), sırası korunur. */
export function charaChildrenOf(periods: readonly SignDashaPeriod[], cycle: number, path: readonly Sign[]): SignDashaPeriod[] {
  return periods.filter((p) => p.cycle === cycle && p.level === path.length + 1 && path.every((s, i) => p.signs[i] === s));
}

/** Dashalar (1. seviye), iki çevrim birlikte, zaman sırasıyla. */
export function charaDashas(periods: readonly SignDashaPeriod[]): SignDashaPeriod[] {
  return periods.filter((p) => p.level === 1);
}

/** `node`, açık yolun bir öneki mi (o düğüm açık mı)? */
export function isCharaOpen(open: SignDashaPath, cycle: number, signs: readonly Sign[]): boolean {
  return open.cycle === cycle && signs.length <= open.signs.length && signs.every((s, i) => open.signs[i] === s);
}

/** Dönemin adı: burç kısaltmaları; ikinci çevrim "2." önekiyle ayrılır. */
export function charaName(t: TFunction, cycle: number, signs: readonly Sign[]): string {
  const name = signs.map((s) => t(`sign.${signIndex(s)}`)).join("–");
  return cycle === 2 ? t("dasha.chara.secondCycle", { name }) : name;
}

/** İlk çevrimin burçları ve yılları, dasha sırasıyla (başlık satırı için). */
export function charaYears(dasha: CharaDasha): { sign: Sign; years: number }[] {
  return charaDashas(dasha.periods)
    .filter((p) => p.cycle === 1)
    .map((p, i) => ({ sign: p.signs[0], years: dasha.years[i] }));
}

/** `enabled === false`: sistem chara değil, komut çağrılmaz. `data === null`: saat bilinmiyor. */
export function useCharaDasha(opened: OpenedClient | null, enabled = true): ModuleState<CharaDasha | null> {
  return useModule(opened !== null && enabled ? inputKey(opened) : null, () => api.computeCharaDasha(chartInput(opened as OpenedClient)));
}

/** Bir dönemin istenerek yüklenen alt dönemleri; `path === null` ise komut çağrılmaz. */
export function useCharaChildren(opened: OpenedClient | null, cycle: number, path: readonly Sign[] | null): ModuleState<SignDashaPeriod[]> {
  const key = opened !== null && path !== null ? `${inputKey(opened)}|${charaKey(cycle, path)}` : null;
  return useModule(key, () => api.charaDashaChildren(chartInput(opened as OpenedClient), [...(path ?? [])], cycle));
}
