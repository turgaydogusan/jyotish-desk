// Modal pencere (WAI-ARIA dialog): açılınca ilk alana odaklanır, Tab içeride döner, Esc
// kapatır, kapanınca odak açan öğeye döner.
// İç içe açılan pencerelerde Esc ve Tab'ı yalnızca en son açılan işler. `variant="sheet"` sağdan kayan paneldir
// (doğum formu); arka plan tıklamayla kapanmaz (yarım form kaybolmasın).
import { useEffect, useId, useRef, type MouseEvent, type ReactNode } from "react";

const FOCUSABLE = 'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export interface DialogProps {
  title: string;
  onClose(): void;
  variant?: "center" | "sheet";
  children: ReactNode;
}

export function Dialog({ title, onClose, variant = "center", children }: DialogProps) {
  const ref = useRef<HTMLDivElement>(null);
  const titleId = useId();

  useEffect(() => {
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const first = ref.current?.querySelector<HTMLElement>("[data-autofocus]:not([disabled])") ?? ref.current?.querySelector<HTMLElement>(FOCUSABLE);
    (first ?? ref.current)?.focus();
    return () => previous?.focus();
  }, []);

  // Tuşlar belge düzeyinde dinlenir: odak pencereden çıksa da (odaktaki düğme kaybolur ya da
  // kilitlenir) Esc kapatır, Tab odağı içeri alır. İçerideki bir bileşen Esc'i kendisi
  // tüketmek için React olayında `stopPropagation` çağırır (ör. yer arama listesi).
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const root = ref.current;
      if (!root) return;
      // İç içe pencerede (formun üstünde onay) tuşları yalnızca en üstteki işler.
      const modals = document.querySelectorAll('[role="dialog"][aria-modal="true"]');
      if (modals[modals.length - 1] !== root) return;
      if (e.key === "Escape") {
        // IME bileşimini iptal eden Esc pencereyi kapatmaz (WebKit bunu 229 koduyla bildirir).
        if (e.isComposing || e.keyCode === 229) return;
        e.preventDefault();
        e.stopPropagation();
        onClose();
        return;
      }
      if (e.key !== "Tab") return;
      const items = [...root.querySelectorAll<HTMLElement>(FOCUSABLE)];
      const active = document.activeElement;
      // Odak pencerenin kendisinde (tabIndex=-1) ya da dışındaysa uç öğedeymiş gibi davranılır.
      const loose = active === root || !root.contains(active);
      const edge = e.shiftKey ? items[0] : items[items.length - 1];
      if (!loose && active !== edge) return;
      e.preventDefault();
      ((e.shiftKey ? items[items.length - 1] : items[0]) ?? root).focus();
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  // Arka plana tıklamak odağı pencereden çıkarmaz.
  const keepFocus = (e: MouseEvent) => {
    if (e.target === e.currentTarget) e.preventDefault();
  };

  return (
    <div className="dialog-backdrop no-print" onMouseDown={keepFocus}>
      <div ref={ref} className={`dialog dialog--${variant}`} role="dialog" aria-modal="true" aria-labelledby={titleId} tabIndex={-1}>
        <h2 id={titleId} className="dialog-title">
          {title}
        </h2>
        {children}
      </div>
    </div>
  );
}
