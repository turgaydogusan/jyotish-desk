// "Yeni prashna" formunun taslağı ve isteğe çevrilmesi (ileri modüller spec'i §11). Burada
// astrolojik hesap yoktur (C4): an ve yer Rust'ta çözülür, lagna Rust'ta kurulur. Yalnızca alan
// biçimi denetlenir (sayı 1–108 tamsayı mı, tarih ve saat yazılmış mı). Soru metni, yer ve an
// kişisel veridir: loglanmaz.
import type { AppPrefs, CommandError, HomePlace, OpenedClient, PrashnaRequest } from "../bindings";
import { chartPlace } from "../modules/muhurta";

/** Rust `PRASHNA_NUMBER_MAX` (fikstürde `prashna_ids.max_number`). */
export const PRASHNA_NUMBER_MAX = 108;
/** Rust `MAX_QUESTION_CHARS` (fikstürde `prashna_ids.max_question_chars`). */
export const MAX_QUESTION_CHARS = 2000;

/** Yerin kaynağı: bulunulan yer (tercihlerden), açık haritanın yeri ya da atlastan aranan yer. */
export const PRASHNA_PLACE_SOURCES = ["home", "chart", "search"] as const;
export type PrashnaPlaceSource = (typeof PRASHNA_PLACE_SOURCES)[number];

export type PrashnaField = "at" | "place" | "number" | "question" | "focus_house";
export const PRASHNA_FIELDS: readonly PrashnaField[] = ["at", "place", "number", "question", "focus_house"];

export interface PrashnaDraft {
  /** `now`: an Rust'ta okunur; `local`: sorunun yerinin duvar saati. */
  moment: "now" | "local";
  date: string;
  time: string;
  source: PrashnaPlaceSource;
  /** Atlastan seçilen yer (`source === "search"`). */
  searched: HomePlace | null;
  /** Yazıldığı gibi; boşsa zaman prashnasıdır. */
  number: string;
  question: string;
  /** Boş dizgi: odak evi seçilmedi. */
  focusHouse: string;
  /** Açık danışana ekle; işaretli değilse (ya da açık danışan yoksa) yerleşik "Prashna" danışanı. */
  toOpenClient: boolean;
}

export function emptyDraft(opened: OpenedClient | null): PrashnaDraft {
  return {
    moment: "now",
    date: "",
    time: "",
    // Bulunulan yer seçilmemişse haritanın yerine sessizce düşülmez: form yer ister.
    source: "home",
    searched: null,
    number: "",
    question: "",
    focusHouse: "",
    toOpenClient: opened !== null,
  };
}

/** Seçilen kaynağın yeri; yoksa `null` (form yer ister). */
export function draftPlace(draft: PrashnaDraft, prefs: AppPrefs | null, opened: OpenedClient | null): HomePlace | null {
  if (draft.source === "home") return prefs?.home_place ?? null;
  if (draft.source === "chart") return opened ? chartPlace(opened) : null;
  return draft.searched;
}

export type DraftErrors = Partial<Record<PrashnaField, string>>;

/**
 * Taslağı isteğe çevirir. Biçim hataları alan başına bir i18n anahtarıdır (`prashna.form.error.*`);
 * değerlerin anlamı (an o yerde var mı, yer çözülüyor mu) Rust'ta denetlenir.
 */
export function toRequest(
  draft: PrashnaDraft,
  prefs: AppPrefs | null,
  opened: OpenedClient | null,
): { request: PrashnaRequest; errors: null } | { request: null; errors: DraftErrors } {
  const errors: DraftErrors = {};
  const place = draftPlace(draft, prefs, opened);
  if (place === null) errors.place = "prashna.form.error.placeMissing";
  if (draft.moment === "local" && (draft.date.trim() === "" || draft.time.trim() === "")) errors.at = "prashna.form.error.atMissing";
  const typed = draft.number.trim();
  let number: number | null = null;
  if (typed !== "") {
    if (/^\d{1,3}$/.test(typed) && Number(typed) >= 1 && Number(typed) <= PRASHNA_NUMBER_MAX) number = Number(typed);
    else errors.number = "prashna.form.error.number";
  }
  if ([...draft.question.trim()].length > MAX_QUESTION_CHARS) errors.question = "prashna.form.error.question";
  if (place === null || Object.keys(errors).length > 0) return { request: null, errors };
  return {
    request: {
      client_id: draft.toOpenClient && opened ? opened.client.id : null,
      at: draft.moment === "now" ? "Now" : { Local: { date: draft.date.trim(), time: draft.time.trim() } },
      place,
      number,
      question: draft.question,
      focus_house: draft.focusHouse === "" ? null : Number(draft.focusHouse),
    },
    errors: null,
  };
}

/** Rust'ın alan adlı hatasının (`invalid_prashna_input { field }`) formdaki alanı; başka hata `null`. */
export function errorField(error: CommandError): PrashnaField | null {
  if (error.code !== "invalid_prashna_input") return null;
  const field = error.params.field;
  return (PRASHNA_FIELDS as readonly string[]).includes(field) ? (field as PrashnaField) : null;
}
