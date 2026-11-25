// Baskı seçenekleri (Plan 12 D5): hangi sayfalar basılır ve harita biçimi. Arayüz tercihidir
// (tema ve dil gibi `localStorage`'da durur); danışan verisi içermez. Okuma toleranslıdır:
// bozuk ya da eski kayıt varsayılana döner, bilinmeyen sayfa kimlikleri atlanır.
import { isChartStyle, type ChartStyle } from "../chart/types";

/** UI spec §7 sırası: kapak, D1+D9, dasa, graha tablosu. */
export const PRINT_PAGES = ["cover", "charts", "dasha", "grahas"] as const;
export type PrintPageId = (typeof PRINT_PAGES)[number];

export interface PrintOptions {
  /** `PRINT_PAGES` sırasında, tekrarsız, en az bir sayfa. */
  pages: PrintPageId[];
  style: ChartStyle;
}

export const PRINT_STORAGE_KEY = "jyotish-desk.print.v1";

export function defaultPrintOptions(): PrintOptions {
  return { pages: [...PRINT_PAGES], style: "north" };
}

function isPrintPageId(v: unknown): v is PrintPageId {
  return typeof v === "string" && (PRINT_PAGES as readonly string[]).includes(v);
}

export function parsePrintOptions(raw: unknown): PrintOptions {
  const fallback = defaultPrintOptions();
  if (typeof raw !== "object" || raw === null) return fallback;
  const { pages, style } = raw as { pages?: unknown; style?: unknown };
  const chosen = new Set(Array.isArray(pages) ? pages.filter(isPrintPageId) : []);
  return {
    pages: chosen.size === 0 ? fallback.pages : PRINT_PAGES.filter((p) => chosen.has(p)),
    style: isChartStyle(style) ? style : fallback.style,
  };
}

/** Sayfayı aç/kapat; son sayfa kapatılamaz (boş belge basılmaz). */
export function togglePage(pages: readonly PrintPageId[], page: PrintPageId): PrintPageId[] {
  if (pages.includes(page)) return pages.length === 1 ? [...pages] : pages.filter((p) => p !== page);
  return PRINT_PAGES.filter((p) => p === page || pages.includes(p));
}

export function loadPrintOptions(storage?: Storage): PrintOptions {
  try {
    return parsePrintOptions(JSON.parse((storage ?? window.localStorage).getItem(PRINT_STORAGE_KEY) ?? "null"));
  } catch {
    return defaultPrintOptions();
  }
}

/** Yazılamazsa (depolama kapalı ya da dolu) seçenek yalnızca bu oturumda geçerli olur. */
export function savePrintOptions(options: PrintOptions, storage?: Storage): void {
  try {
    (storage ?? window.localStorage).setItem(PRINT_STORAGE_KEY, JSON.stringify(options));
  } catch {
    // Tercih kaybı zararsızdır ve loglanmaz (C1: log izin listesi büyümez).
  }
}
