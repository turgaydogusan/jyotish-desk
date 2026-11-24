// Pano: etkin düzenin panellerini CSS grid ile çizer. 12 sütunda paneller başlıktan
// sürüklenir, sağ alt köşeden boyutlanır, klavyeyle Alt+ok / Alt+Shift+ok ile düzenlenir.
// Daha dar pencerede (8 ya da 4 sütun) yeniden akış yalnızca gösterimdir; düzenleme kapalıdır
// ve kaydedilen 12 sütunluk düzen değişmez.
import { useEffect, useRef, useState, type PointerEvent as ReactPointerEvent } from "react";
import { columnsForWidth, movePanel, reflow, resizePanel, GRID_COLUMNS } from "../layout/grid";
import type { Panel } from "../layout/model";
import { useLayoutStore } from "../layout/store";
import "./dashboard.css";
import { gridMetrics, pxToGridDelta } from "./metrics";
import { PanelFrame, type DragMode } from "./PanelFrame";

interface DragState {
  id: string;
  mode: DragMode;
  startX: number;
  startY: number;
  origin: Panel;
  base: Panel[];
}

export function Dashboard({ width }: { width: number }) {
  const { active, updatePanels } = useLayoutStore();
  const columns = columnsForWidth(width);
  const editable = columns === GRID_COLUMNS;
  const metrics = gridMetrics(width, columns);

  const [preview, setPreview] = useState<Panel[] | null>(null);
  const previewRef = useRef<Panel[] | null>(null);
  const drag = useRef<DragState | null>(null);
  const detach = useRef<(() => void) | null>(null);

  useEffect(() => () => detach.current?.(), []);

  const showPreview = (panels: Panel[] | null) => {
    previewRef.current = panels;
    setPreview(panels);
  };

  const onDragStart = (e: ReactPointerEvent, panel: Panel, mode: DragMode) => {
    if (!editable || e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    drag.current = { id: panel.id, mode, startX: e.clientX, startY: e.clientY, origin: panel, base: active.panels };

    const onMove = (ev: PointerEvent) => {
      const d = drag.current;
      if (!d) return;
      const { dx, dy } = pxToGridDelta(ev.clientX - d.startX, ev.clientY - d.startY, metrics);
      showPreview(
        d.mode === "move"
          ? movePanel(d.base, d.id, d.origin.x + dx, d.origin.y + dy)
          : resizePanel(d.base, d.id, d.origin.w + dx, d.origin.h + dy),
      );
    };
    const finish = (commit: boolean) => () => {
      const result = previewRef.current;
      detach.current?.();
      drag.current = null;
      showPreview(null);
      if (commit && result) updatePanels(result);
    };
    const onUp = finish(true);
    const onCancel = finish(false);
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    window.addEventListener("pointercancel", onCancel);
    detach.current = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      window.removeEventListener("pointercancel", onCancel);
      detach.current = null;
    };
  };

  const onKeyboardEdit = (panel: Panel, mode: DragMode, dx: number, dy: number) => {
    updatePanels(
      mode === "move"
        ? movePanel(active.panels, panel.id, panel.x + dx, panel.y + dy)
        : resizePanel(active.panels, panel.id, panel.w + dx, panel.h + dy),
    );
  };

  const panels = editable ? (preview ?? active.panels) : reflow(active.panels, columns);
  return (
    <div
      className="dashboard"
      data-columns={columns}
      style={{
        gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
        gridAutoRows: `${metrics.rowHeight}px`,
        gap: `${metrics.gap}px`,
      }}
    >
      {panels.map((p) => (
        <PanelFrame key={p.id} panel={p} editable={editable} onDragStart={onDragStart} onKeyboardEdit={onKeyboardEdit} />
      ))}
    </div>
  );
}
