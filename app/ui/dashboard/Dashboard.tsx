// Pano: etkin düzenin panellerini CSS grid ile çizer. 12 sütunda paneller başlıktan
// sürüklenir, sağ alt köşeden boyutlanır, klavyeyle Alt+ok / Alt+Shift+ok ile düzenlenir.
// Daha dar pencerede (8 ya da 4 sütun) yeniden akış yalnızca gösterimdir; düzenleme kapalıdır
// ve kaydedilen 12 sütunluk düzen değişmez.
import { useTranslation } from "react-i18next";
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
  pointerId: number;
  layoutId: string;
  startX: number;
  startY: number;
  origin: Panel;
  base: Panel[];
  lastDx: number;
  lastDy: number;
}

function samePanels(a: Panel[], b: Panel[]): boolean {
  return a.length === b.length && a.every((p, i) => p.id === b[i].id && p.x === b[i].x && p.y === b[i].y && p.w === b[i].w && p.h === b[i].h);
}

export function Dashboard({ width }: { width: number }) {
  const { t } = useTranslation();
  const { active, updatePanels } = useLayoutStore();
  const columns = columnsForWidth(width);
  const editable = columns === GRID_COLUMNS;
  const metrics = gridMetrics(width, columns);

  const [preview, setPreview] = useState<Panel[] | null>(null);
  const [announcement, setAnnouncement] = useState("");
  const previewRef = useRef<Panel[] | null>(null);
  const drag = useRef<DragState | null>(null);
  const detach = useRef<(() => void) | null>(null);
  const activeIdRef = useRef(active.id);
  activeIdRef.current = active.id;

  const showPreview = (panels: Panel[] | null) => {
    previewRef.current = panels;
    setPreview(panels);
  };

  const cancelDrag = () => {
    detach.current?.();
    drag.current = null;
    showPreview(null);
  };

  useEffect(() => () => detach.current?.(), []);

  // Sürüklerken etkin düzen değişirse sürükleme iptal edilir; sonuç başka düzene yazılmaz.
  useEffect(() => {
    if (drag.current && drag.current.layoutId !== active.id) cancelDrag();
  }, [active.id]);

  const onDragStart = (e: ReactPointerEvent, panel: Panel, mode: DragMode) => {
    if (!editable || e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    detach.current?.();
    const state: DragState = {
      id: panel.id,
      mode,
      pointerId: e.pointerId,
      layoutId: active.id,
      startX: e.clientX,
      startY: e.clientY,
      origin: panel,
      base: active.panels,
      lastDx: 0,
      lastDy: 0,
    };
    drag.current = state;
    showPreview(null);

    const onMove = (ev: PointerEvent) => {
      if (ev.pointerId !== state.pointerId) return;
      // Pencere dışında bırakılan düğme: pointerup hiç gelmez, sürükleme asılı kalmasın.
      if (ev.buttons === 0) {
        cancelDrag();
        return;
      }
      const { dx, dy } = pxToGridDelta(ev.clientX - state.startX, ev.clientY - state.startY, metrics);
      if (dx === state.lastDx && dy === state.lastDy) return;
      state.lastDx = dx;
      state.lastDy = dy;
      showPreview(
        state.mode === "move"
          ? movePanel(state.base, state.id, state.origin.x + dx, state.origin.y + dy)
          : resizePanel(state.base, state.id, state.origin.w + dx, state.origin.h + dy),
      );
    };
    const onUp = (ev: PointerEvent) => {
      if (ev.pointerId !== state.pointerId) return;
      const result = previewRef.current;
      const sameLayout = state.layoutId === activeIdRef.current;
      cancelDrag();
      if (result && sameLayout && !samePanels(result, state.base)) updatePanels(result);
    };
    const onCancel = (ev: PointerEvent) => {
      if (ev.pointerId === state.pointerId) cancelDrag();
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    window.addEventListener("pointercancel", onCancel);
    window.addEventListener("blur", cancelDrag);
    detach.current = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      window.removeEventListener("pointercancel", onCancel);
      window.removeEventListener("blur", cancelDrag);
      detach.current = null;
    };
  };

  const onKeyboardEdit = (panel: Panel, mode: DragMode, dx: number, dy: number) => {
    const next =
      mode === "move"
        ? movePanel(active.panels, panel.id, panel.x + dx, panel.y + dy)
        : resizePanel(active.panels, panel.id, panel.w + dx, panel.h + dy);
    updatePanels(next);
    const moved = next.find((p) => p.id === panel.id);
    if (moved) setAnnouncement(t("panel.announce", { col: moved.x + 1, row: moved.y + 1, w: moved.w, h: moved.h }));
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
      <div className="visually-hidden" aria-live="polite" aria-atomic="true">
        {announcement}
      </div>
    </div>
  );
}
