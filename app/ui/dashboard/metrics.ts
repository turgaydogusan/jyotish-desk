// Piksel ↔ ızgara dönüşümü. Pano CSS grid'i `GRID_GAP_PX` aralıkla çizer.
import { ROW_HEIGHT_PX, type Columns } from "../layout/grid";

export const GRID_GAP_PX = 8;

export interface GridMetrics {
  columns: Columns;
  colWidth: number;
  rowHeight: number;
  gap: number;
}

export function gridMetrics(width: number, columns: Columns): GridMetrics {
  const colWidth = Math.max(0, (width - (columns - 1) * GRID_GAP_PX) / columns);
  return { columns, colWidth, rowHeight: ROW_HEIGHT_PX, gap: GRID_GAP_PX };
}

/** İşaretçinin piksel yer değiştirmesini en yakın ızgara adımına yuvarlar. */
export function pxToGridDelta(dxPx: number, dyPx: number, m: GridMetrics): { dx: number; dy: number } {
  const colStep = m.colWidth + m.gap;
  const rowStep = m.rowHeight + m.gap;
  return {
    dx: m.colWidth > 0 ? Math.round(dxPx / colStep) : 0,
    dy: Math.round(dyPx / rowStep),
  };
}

/** Alt+ok / Alt+Shift+ok için yön. */
export function arrowDelta(key: string): { dx: number; dy: number } | null {
  switch (key) {
    case "ArrowLeft":
      return { dx: -1, dy: 0 };
    case "ArrowRight":
      return { dx: 1, dy: 0 };
    case "ArrowUp":
      return { dx: 0, dy: -1 };
    case "ArrowDown":
      return { dx: 0, dy: 1 };
    default:
      return null;
  }
}
