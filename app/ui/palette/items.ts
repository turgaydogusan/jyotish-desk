// Komut paletinin öğeleri (UI spec §4: danışan, harita, varga, düzen, panel ve komut tek arama
// kutusunda). Danışanlar Rust'ta aranır (`search_clients`); diğer öğeler arayüzün kendi
// listeleridir ve burada metinle süzülür. Süzme bir dizgi karşılaştırmasıdır, hesap değildir.
import type { ChartRecord, SearchHit, Varga } from "../bindings";
import type { Layout, ModulePanelKind } from "../layout/model";

export type PaletteCommand = "newChart" | "newPrashna" | "chartSearch" | "addChart" | "editChart" | "settings" | "print" | "toggleNotes" | "sidebar" | "help" | "about";

export type PaletteItem =
  | { kind: "client"; key: string; label: string; detail: string | null; clientId: number }
  | { kind: "chart"; key: string; label: string; detail: string; clientId: number; chartId: number }
  | { kind: "layout"; key: string; label: string; layoutId: string }
  | { kind: "varga"; key: string; label: string; varga: Varga }
  | { kind: "command"; key: string; label: string; command: PaletteCommand }
  | { kind: "panel"; key: string; label: string; panelKind: ModulePanelKind };

/** Büyük/küçük harf ve aksan duyarsız karşılaştırma anahtarı ("Navāmsa" → "navamsa"). */
export function foldText(text: string, locale: string): string {
  return text.normalize("NFD").replace(/\p{M}/gu, "").toLocaleLowerCase(locale).replace(/ı/g, "i");
}

/** Sorgunun her kelimesi öğe adında geçiyorsa eşleşir; boş sorgu her şeyle eşleşir. */
export function matches(label: string, query: string, locale: string): boolean {
  const words = foldText(query, locale).split(/\s+/).filter(Boolean);
  const hay = foldText(label, locale);
  return words.every((w) => hay.includes(w));
}

export function clientItems(hits: readonly SearchHit[], detail: (hit: SearchHit) => string | null): PaletteItem[] {
  return hits.map((hit) => ({ kind: "client", key: `client:${hit.client.id}`, label: hit.client.name, detail: detail(hit), clientId: hit.client.id }));
}

export interface LocalSources {
  charts: readonly ChartRecord[];
  chartLabel(chart: ChartRecord): string;
  layouts: readonly Layout[];
  layoutLabel(layout: Layout): string;
  /** Etkin düzende harita paneli yoksa varga öğesi gösterilmez. */
  vargas: readonly { varga: Varga; label: string }[];
  commands: readonly { command: PaletteCommand; label: string }[];
  /** Etkin düzene eklenip kaldırılabilen paneller (Plan 14 D9). */
  panels: readonly { kind: ModulePanelKind; label: string }[];
}

/** Arayüzün kendi öğeleri, sorguyla süzülmüş: komutlar, paneller, haritalar, düzenler, vargalar. */
export function localItems(src: LocalSources, query: string, locale: string): PaletteItem[] {
  const out: PaletteItem[] = [
    ...src.commands.map((c): PaletteItem => ({ kind: "command", key: `command:${c.command}`, label: c.label, command: c.command })),
    ...src.panels.map((p): PaletteItem => ({ kind: "panel", key: `panel:${p.kind}`, label: p.label, panelKind: p.kind })),
    ...src.charts.map(
      (c): PaletteItem => ({ kind: "chart", key: `chart:${c.id}`, label: src.chartLabel(c), detail: c.birth.date, clientId: c.client_id, chartId: c.id }),
    ),
    ...src.layouts.map((l): PaletteItem => ({ kind: "layout", key: `layout:${l.id}`, label: src.layoutLabel(l), layoutId: l.id })),
    ...src.vargas.map((v): PaletteItem => ({ kind: "varga", key: `varga:${v.varga}`, label: v.label, varga: v.varga })),
  ];
  return out.filter((i) => matches(i.label, query, locale));
}
