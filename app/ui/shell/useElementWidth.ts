import { useLayoutEffect, useState, type RefObject } from "react";

/** Öğenin İÇERİK kutusunun piksel genişliği; ölçülemezse (ör. jsdom) pencere genişliği. */
export function useElementWidth(ref: RefObject<HTMLElement | null>): number {
  const [width, setWidth] = useState(() => window.innerWidth);
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const fromBox = () => {
      const cs = window.getComputedStyle(el);
      const pad = (parseFloat(cs.paddingLeft) || 0) + (parseFloat(cs.paddingRight) || 0);
      return el.clientWidth - pad;
    };
    const measure = (w?: number) => setWidth(w || fromBox() || window.innerWidth);
    measure();
    if (typeof ResizeObserver === "function") {
      const observer = new ResizeObserver((entries) => measure(entries[entries.length - 1]?.contentRect.width));
      observer.observe(el);
      return () => observer.disconnect();
    }
    const onResize = () => measure();
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, [ref]);
  return width;
}
