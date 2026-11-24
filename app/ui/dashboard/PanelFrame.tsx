import { useId, useState, type KeyboardEvent, type PointerEvent } from "react";
import { useTranslation } from "react-i18next";
import type { Panel } from "../layout/model";
import { panelDefinition } from "../panels/registry";
import { PanelStatusContext, type LoadPhase } from "../panels/states";
import { arrowDelta } from "./metrics";

const KEY_SHORTCUTS = ["Alt", "Alt+Shift"]
  .flatMap((m) => ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].map((k) => `${m}+${k}`))
  .join(" ");

export type DragMode = "move" | "resize";

export interface PanelFrameProps {
  panel: Panel;
  editable: boolean;
  onDragStart(e: PointerEvent, panel: Panel, mode: DragMode): void;
  onKeyboardEdit(panel: Panel, mode: DragMode, dx: number, dy: number): void;
}

/** Panel çerçevesi: başlık (sürükleme tutamacı), satır içi yükleme göstergesi, gövde, boyut tutamacı. */
export function PanelFrame({ panel, editable, onDragStart, onKeyboardEdit }: PanelFrameProps) {
  const { t } = useTranslation();
  const hintId = useId();
  const [phase, setPhase] = useState<LoadPhase>("none");
  const def = panelDefinition(panel.kind);
  const Body = def.component;
  const title = t(def.titleKey);
  const subtitle = def.subtitle?.(panel) ?? null;

  const onKeyDown = (e: KeyboardEvent<HTMLElement>) => {
    // Yalnızca panelin kendisi odaktayken; panel içindeki düğme ve alanlar etkilenmez.
    if (e.target !== e.currentTarget || !e.altKey || !editable) return;
    const step = arrowDelta(e.key);
    if (!step) return;
    e.preventDefault();
    onKeyboardEdit(panel, e.shiftKey ? "resize" : "move", step.dx, step.dy);
  };

  return (
    <section
      className="panel"
      data-panel-id={panel.id}
      data-kind={panel.kind}
      data-x={panel.x}
      data-y={panel.y}
      data-w={panel.w}
      data-h={panel.h}
      data-editable={editable}
      tabIndex={0}
      aria-label={subtitle ? `${title} ${subtitle}` : title}
      aria-keyshortcuts={editable ? KEY_SHORTCUTS : undefined}
      aria-describedby={editable ? hintId : undefined}
      style={{ gridColumn: `${panel.x + 1} / span ${panel.w}`, gridRow: `${panel.y + 1} / span ${panel.h}` }}
      onKeyDown={onKeyDown}
    >
      {editable && (
        <span id={hintId} className="visually-hidden">
          {t("panel.moveHint")}
        </span>
      )}
      <header
        className="panel-header"
        data-testid={`panel-header-${panel.id}`}
        title={editable ? t("panel.moveHint") : undefined}
        onPointerDown={(e) => {
          // Sürükleme başlangıcı varsayılan odaklanmayı engeller; paneli elle odakla.
          e.currentTarget.parentElement?.focus({ preventScroll: true });
          onDragStart(e, panel, "move");
        }}
      >
        <span>{title}</span>
        {subtitle && <span className="panel-subtitle">{subtitle}</span>}
        {phase === "inline" && (
          <span className="panel-status" role="status">
            {t("panel.loading")}…
          </span>
        )}
      </header>
      <div className="panel-body">
        <PanelStatusContext.Provider value={setPhase}>
          <Body panel={panel} />
        </PanelStatusContext.Provider>
      </div>
      {editable && (
        <div
          className="panel-resize"
          data-testid={`panel-resize-${panel.id}`}
          aria-hidden="true"
          onPointerDown={(e) => onDragStart(e, panel, "resize")}
        />
      )}
    </section>
  );
}
