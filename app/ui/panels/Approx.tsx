import type { ReactNode } from "react";

/** Yaklaşık değer: "≈" işareti (fare için `title`) ve ekran okuyucu için bir kez okunan metin. */
export function Approx({ on, label, children }: { on: boolean; label: string; children: ReactNode }) {
  if (!on) return <>{children}</>;
  return (
    <span>
      <span aria-hidden="true" title={label}>
        ≈
      </span>
      <span className="visually-hidden">{label} </span>
      {children}
    </span>
  );
}
