// Efemeris fikstürü (app/src-tauri/tests/ephemeris_table.rs üretir): Delhi için Kasım 2026'nın
// `ephemeris_month` dönüşü (30 satır; dört burç girişi ve iki dönüş işareti Ay dışındaki
// grahalardadır). Ayrı dosyadır: `delhi.json` ve `fixture.ts` eşzamanlı planlarla paylaşılır.
import type { EphemerisMonthView, MuhurtaPlace } from "../bindings";
import raw from "./fixtures/ephemeris.json?raw";

export interface EphemerisFixture {
  /** Tablonun yeri: Delhi, `Asia/Kolkata`. */
  place: MuhurtaPlace;
  /** `ephemeris_month({ year: 2026, month: 11 }, place, varsayılan ayarlar)`. */
  month: EphemerisMonthView;
}

const FIXTURE = JSON.parse(raw) as EphemerisFixture;

/** Her çağrıda derin kopya: testler sonucu değiştirebilir. */
export function ephemerisFixture(): EphemerisFixture {
  return structuredClone(FIXTURE);
}
