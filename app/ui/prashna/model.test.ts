import { expect, test } from "vitest";
import { fixture } from "../test/fixture";
import { draftPlace, emptyDraft, errorField, MAX_QUESTION_CHARS, PRASHNA_NUMBER_MAX, toRequest, type PrashnaDraft } from "./model";

const home = () => fixture().muhurta.home_place;
const prefsWithHome = () => ({ ...fixture().prefs.prefs, home_place: home() });

test("the limits are the ones Rust checks", () => {
  const { prashna_ids } = fixture();
  expect(PRASHNA_NUMBER_MAX).toBe(prashna_ids.max_number);
  expect(MAX_QUESTION_CHARS).toBe(prashna_ids.max_question_chars);
});

test("the empty draft is a time prashna now, at the current place, for the open client", () => {
  const { opened } = fixture();
  expect(emptyDraft(opened)).toEqual({ moment: "now", date: "", time: "", source: "home", searched: null, number: "", question: "", focusHouse: "", toOpenClient: true });
  expect(emptyDraft(null).toOpenClient).toBe(false);
  const checked = toRequest(emptyDraft(opened), prefsWithHome(), opened);
  expect(checked.request).toEqual({ client_id: opened.client.id, at: "Now", place: home(), number: null, question: "", focus_house: null });
});

test("the place comes from the chosen source and is never silently replaced", () => {
  const { opened } = fixture();
  const draft = emptyDraft(opened);
  // Bulunulan yer seçilmemişse haritanın yerine düşülmez: form yer ister.
  expect(draftPlace(draft, fixture().prefs.prefs, opened)).toBeNull();
  expect(draftPlace(draft, null, opened)).toBeNull();
  expect(toRequest(draft, fixture().prefs.prefs, opened)).toEqual({ request: null, errors: { place: "prashna.form.error.placeMissing" } });
  const chart = { ...draft, source: "chart" as const };
  expect(draftPlace(chart, null, opened)).toEqual({ place: opened.chart_record.birth.place, tz: opened.chart_record.birth.tz });
  expect(draftPlace(chart, null, null)).toBeNull();
  const searched = { ...draft, source: "search" as const };
  expect(draftPlace(searched, prefsWithHome(), opened)).toBeNull();
  expect(draftPlace({ ...searched, searched: home() }, null, null)).toEqual(home());
});

test("a typed moment, a number, a question and a focus house go into the request as typed", () => {
  const { opened } = fixture();
  const draft: PrashnaDraft = { ...emptyDraft(opened), moment: "local", date: "2026-11-27", time: "14:05", number: " 54 ", question: "  Deneme sorusu ", focusHouse: "7", toOpenClient: false };
  expect(toRequest(draft, prefsWithHome(), opened).request).toEqual({
    client_id: null,
    at: { Local: { date: "2026-11-27", time: "14:05" } },
    place: home(),
    number: 54,
    // Kırpma Rust'ın işidir; metin yazıldığı gibi gider.
    question: "  Deneme sorusu ",
    focus_house: 7,
  });
  // Açık danışan yoksa kutu işaretli olsa da kimlik gitmez.
  expect(toRequest({ ...draft, toOpenClient: true }, prefsWithHome(), null).request?.client_id).toBeNull();
});

test("format errors name the field; the values themselves are checked in Rust", () => {
  const { opened } = fixture();
  const base = emptyDraft(opened);
  const errors = (patch: Partial<PrashnaDraft>) => toRequest({ ...base, ...patch }, prefsWithHome(), opened).errors;
  for (const number of ["0", "109", "1.5", "-3", "on", "1e2", "0108"]) expect(errors({ number })).toEqual({ number: "prashna.form.error.number" });
  for (const number of ["1", "9", "108", ""]) expect(errors({ number })).toBeNull();
  expect(errors({ moment: "local" })).toEqual({ at: "prashna.form.error.atMissing" });
  expect(errors({ moment: "local", date: "2026-11-27" })).toEqual({ at: "prashna.form.error.atMissing" });
  // "Şimdi" seçiliyken yazılı tarih ve saat yok sayılır.
  expect(toRequest({ ...base, date: "2026-11-27" }, prefsWithHome(), opened).request?.at).toBe("Now");
  // Sınır karakterle sayılır (çok baytlı harfler dahil) ve kırpılmış metne uygulanır.
  expect(errors({ question: ` ${"ş".repeat(MAX_QUESTION_CHARS)} ` })).toBeNull();
  expect(errors({ question: "ş".repeat(MAX_QUESTION_CHARS + 1) })).toEqual({ question: "prashna.form.error.question" });
  // Birden çok hata birlikte döner.
  expect(toRequest({ ...base, number: "0", moment: "local" }, fixture().prefs.prefs, opened).errors).toEqual({
    place: "prashna.form.error.placeMissing",
    at: "prashna.form.error.atMissing",
    number: "prashna.form.error.number",
  });
});

test("a field error from Rust maps to its form field; anything else is a form error", () => {
  for (const field of ["at", "place", "number", "question", "focus_house"]) {
    expect(errorField({ code: "invalid_prashna_input", params: { field } })).toBe(field);
  }
  expect(errorField({ code: "invalid_prashna_input", params: { field: "lagna" } })).toBeNull();
  expect(errorField({ code: "not_found", params: { what: "client", id: "9" } })).toBeNull();
  expect(errorField({ code: "store_sqlite", params: { detail: "x" } })).toBeNull();
});
