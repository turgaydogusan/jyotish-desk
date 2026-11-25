// Pano ızgarası (spec §3.1): saf fonksiyonlar, React yok. Paneller hiçbir zaman üst üste
// binmez; taşınan ya da boyutlanan panel yerinde kalır, çakıştığı paneller aşağı itilir.
// Kullanıcının bıraktığı boşluklar korunur (otomatik yukarı sıkıştırma yalnızca dar
// pencerede yeniden akışta yapılır).
import type { Panel } from "./model";

export const GRID_COLUMNS = 12;
export const ROW_HEIGHT_PX = 32;
export type Columns = 12 | 8 | 4;

/**
 * Kırılım noktaları (pano alanının piksel genişliği):
 * - ≥ 1000 px → 12 sütun (en küçük 3 sütunluk panel ≈ 240 px),
 * - ≥ 640 px → 8 sütun,
 * - daha dar → 4 sütun (her panel tam genişlik, alt alta).
 */
export const BREAKPOINTS: ReadonlyArray<{ minWidth: number; columns: Columns }> = [
  { minWidth: 1000, columns: 12 },
  { minWidth: 640, columns: 8 },
  { minWidth: 0, columns: 4 },
];

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export function columnsForWidth(px: number): Columns {
  for (const b of BREAKPOINTS) if (px >= b.minWidth) return b.columns;
  return 4;
}

/** En küçük boyut: harita 4 × 6, diğerleri 3 × 4 (spec §3.1). */
export function minSize(kind: string): { w: number; h: number } {
  return kind === "chart" ? { w: 4, h: 6 } : { w: 3, h: 4 };
}

export function collides(a: Rect, b: Rect): boolean {
  return a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
}

export function hasOverlap(panels: readonly Rect[]): boolean {
  for (let i = 0; i < panels.length; i++)
    for (let j = i + 1; j < panels.length; j++) if (collides(panels[i], panels[j])) return true;
  return false;
}

/** Tamsayıya yuvarlar, en küçük boyutu ve ızgara sınırlarını uygular. */
export function clampPanel(p: Panel, columns: number = GRID_COLUMNS): Panel {
  const m = minSize(p.kind);
  const w = Math.max(Math.min(m.w, columns), Math.min(columns, Math.round(p.w)));
  const h = Math.max(m.h, Math.round(p.h));
  const x = Math.max(0, Math.min(columns - w, Math.round(p.x)));
  const y = Math.max(0, Math.round(p.y));
  return { ...p, x, y, w, h };
}

function byReadingOrder(a: Rect, b: Rect): number {
  return a.y - b.y || a.x - b.x;
}

/**
 * Çakışmaları aşağı iterek çözer. `pinnedId` verilirse o panel yerinden oynamaz.
 * Diğerleri okuma sırasıyla (y, sonra x) yerleştirilir; çakışan panel, çakıştığı
 * panellerin altına iner. Dönüş dizisi girişle aynı sıradadır.
 */
export function settle(panels: readonly Panel[], pinnedId: string | null = null): Panel[] {
  const pinned = pinnedId === null ? undefined : panels.find((p) => p.id === pinnedId);
  const placed: Panel[] = pinned ? [pinned] : [];
  const rest = panels.filter((p) => p !== pinned).sort(byReadingOrder);
  const out = new Map<string, Panel>(pinned ? [[pinned.id, pinned]] : []);
  for (const p of rest) {
    let q = p;
    for (;;) {
      const hits = placed.filter((r) => collides(q, r));
      if (hits.length === 0) break;
      q = { ...q, y: Math.max(...hits.map((r) => r.y + r.h)) };
    }
    placed.push(q);
    out.set(q.id, q);
  }
  return panels.map((p) => out.get(p.id)!);
}

/** Yüklenen ya da dışarıdan gelen panelleri geçerli hale getirir: sınırla, sonra çakışmaları çöz. */
export function normalizePanels(panels: readonly Panel[], columns: number = GRID_COLUMNS): Panel[] {
  return settle(panels.map((p) => clampPanel(p, columns)));
}

export function movePanel(panels: readonly Panel[], id: string, x: number, y: number, columns: number = GRID_COLUMNS): Panel[] {
  const p = panels.find((q) => q.id === id);
  if (!p) return [...panels];
  const moved = clampPanel({ ...p, x, y }, columns);
  return settle(panels.map((q) => (q.id === id ? moved : q)), id);
}

export function resizePanel(panels: readonly Panel[], id: string, w: number, h: number, columns: number = GRID_COLUMNS): Panel[] {
  const p = panels.find((q) => q.id === id);
  if (!p) return [...panels];
  // Sağ kenar ızgaradan taşmasın: genişlik önce boş sütunlarla sınırlanır.
  const resized = clampPanel({ ...p, w: Math.min(w, columns - p.x), h }, columns);
  return settle(panels.map((q) => (q.id === id ? resized : q)), id);
}

/** Her paneli olabildiğince yukarı taşır (sırayı bozmadan, üstündekinin içinden geçmeden). */
export function compact(panels: readonly Panel[]): Panel[] {
  const placed: Panel[] = [];
  const out = new Map<string, Panel>();
  for (const p of [...panels].sort(byReadingOrder)) {
    let q = p;
    while (placed.some((r) => collides(q, r))) q = { ...q, y: q.y + 1 };
    while (q.y > 0 && !placed.some((r) => collides({ ...q, y: q.y - 1 }, r))) q = { ...q, y: q.y - 1 };
    placed.push(q);
    out.set(q.id, q);
  }
  return panels.map((p) => out.get(p.id)!);
}

/**
 * Dar pencere için yeniden akış (spec §3.1): okuma sırası korunur, genişlikler
 * sütun sayısına ölçeklenir ve paneller satır satır, soldan sağa dizilir.
 * 4 sütunda her panel tam genişliktir (alt alta). Sonuç yalnızca gösterim içindir;
 * kaydedilen düzen 12 sütunluk düzendir.
 */
export function reflow(panels: readonly Panel[], columns: Columns): Panel[] {
  if (columns === GRID_COLUMNS) return normalizePanels(panels, columns);
  const ordered = [...panels].sort(byReadingOrder);
  const out = new Map<string, Panel>();
  let x = 0;
  let rowY = 0;
  let rowBottom = 0;
  for (const p of ordered) {
    const scaled = columns === 4 ? 4 : Math.round((p.w * columns) / GRID_COLUMNS);
    const c = clampPanel({ ...p, w: scaled, x: 0, y: 0 }, columns);
    if (x + c.w > columns) {
      x = 0;
      rowY = rowBottom;
    }
    const placed = { ...c, x, y: rowY };
    out.set(p.id, placed);
    x += c.w;
    rowBottom = Math.max(rowBottom, rowY + c.h);
  }
  return panels.map((p) => out.get(p.id)!);
}
