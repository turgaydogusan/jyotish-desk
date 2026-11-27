// Onay kutusu kümesi: görünür başlık ve her değer için etiketli bir kutu. Uzun kümeler açılır
// kapanır bir bölümde durur; bölümün başlığı seçili sayıyı söyler.
import type { ReactNode } from "react";

export interface CheckSetProps<T extends string | number> {
  legend: string;
  options: readonly { value: T; label: ReactNode }[];
  selected: readonly T[];
  onToggle(value: T): void;
  /** Verilirse küme `<details>` içinde durur ve bu metin özet satırıdır. */
  fold?: string;
}

export function CheckSet<T extends string | number>({ legend, options, selected, onToggle, fold }: CheckSetProps<T>) {
  const boxes = (
    <fieldset className="search-checks">
      <legend className={fold ? "visually-hidden" : undefined}>{legend}</legend>
      {options.map((o) => (
        <label key={o.value}>
          <input type="checkbox" checked={selected.includes(o.value)} onChange={() => onToggle(o.value)} />
          {o.label}
        </label>
      ))}
    </fieldset>
  );
  if (!fold) return boxes;
  return (
    <details className="search-fold">
      <summary>{fold}</summary>
      {boxes}
    </details>
  );
}
