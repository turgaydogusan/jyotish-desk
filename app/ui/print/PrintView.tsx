// Baskı önizlemesi (UI spec §7, Plan 12): seçilen A4 sayfalar ekranda kâğıt olarak görünür;
// araç çubuğu (`no-print`) kâğıda çıkmaz. Yazdırma penceresini Rust açar (`print_window`);
// PDF, o pencerenin "PDF olarak kaydet" seçeneğidir (v1 spec §4). Önizleme açıkken belge
// başlığı danışan ve harita adıdır: yazdırma işi ve önerilen PDF dosya adı bunu kullanır.
import { useEffect, useId, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError } from "../bindings";
import { CHART_STYLES, isChartStyle } from "../chart/types";
import { PanelError } from "../panels/states";
import { useSession } from "../session/store";
import { MAIN_ID } from "../shell/focus";
import { buildPrintDocument } from "./model";
import { loadPrintOptions, PRINT_PAGES, savePrintOptions, togglePage, type PrintOptions } from "./options";
import { PrintDocumentView } from "./PrintDocumentView";

export interface PrintViewProps {
  /** Son yazdırma denemesinin hatası (Shell tutar; ⌘P de aynı yolu kullanır). */
  error: CommandError | null;
  onPrint(): void;
}

export function PrintView({ error, onPrint }: PrintViewProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const [options, setOptions] = useState<PrintOptions>(() => loadPrintOptions());
  const id = useId();
  const printButton = useRef<HTMLButtonElement>(null);
  const doc = useMemo(() => (opened ? buildPrintDocument(t, opened, options) : null), [t, opened, options]);
  const title = doc?.title ?? null;

  // Açılınca odak "Yazdır…" düğmesindedir (⌘P, Enter); kapanınca `body`'ye düşmez.
  useEffect(() => {
    printButton.current?.focus();
    return () => {
      if (document.activeElement === document.body) document.getElementById(MAIN_ID)?.focus();
    };
  }, []);

  useEffect(() => {
    if (title === null) return;
    const previous = document.title;
    document.title = title;
    return () => {
      document.title = previous;
    };
  }, [title]);

  if (!doc) return null;
  const update = (next: PrintOptions) => {
    savePrintOptions(next);
    setOptions(next);
  };
  const lastPage = options.pages.length === 1;
  return (
    <div className="print-view" data-print-preview="">
      <div className="print-toolbar no-print" role="group" aria-label={t("print.toolbar")}>
        <fieldset className="print-pages">
          <legend>{t("print.pages")}</legend>
          {PRINT_PAGES.map((page) => {
            const only = lastPage && options.pages[0] === page;
            return (
              <label key={page}>
                <input
                  type="checkbox"
                  checked={options.pages.includes(page)}
                  aria-disabled={only || undefined}
                  aria-describedby={only ? `${id}-last` : undefined}
                  onChange={() => update({ ...options, pages: togglePage(options.pages, page) })}
                />
                {t(`print.page.${page}`)}
              </label>
            );
          })}
          {lastPage && (
            <span id={`${id}-last`} className="print-hint">
              {t("print.lastPage")}
            </span>
          )}
        </fieldset>
        <label htmlFor={`${id}-style`}>{t("print.style")}</label>
        <select id={`${id}-style`} value={options.style} onChange={(e) => isChartStyle(e.target.value) && update({ ...options, style: e.target.value })}>
          {CHART_STYLES.map((s) => (
            <option key={s} value={s}>
              {t(`chartStyle.${s}`)}
            </option>
          ))}
        </select>
        <button ref={printButton} type="button" aria-keyshortcuts="Meta+P" onClick={onPrint}>
          {t("print.print")}
        </button>
        <p className="print-hint">{t("print.pdfHint")}</p>
        {error && (
          <PanelError
            error={error}
            onRetry={() => {
              // Yeniden deneme hata kutusunu kaldırır: odak önce "Yazdır…" düğmesine taşınır, gövdeye düşmez.
              printButton.current?.focus();
              onPrint();
            }}
          />
        )}
      </div>
      <PrintDocumentView doc={doc} />
    </div>
  );
}
