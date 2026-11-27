// Sankranti fikstürü (app/src-tauri/tests/sankranti.rs üretir): Delhi için 2026 yılının
// `sankranti_list` dönüşü ve aynı yılın Tromsø listesi (bir giriş gün doğmayan bir güne düşer).
// Ayrı dosyadır: `delhi.json` ve `fixture.ts` eşzamanlı planlarla paylaşılır.
import type { MuhurtaPlace, SankrantiListView } from "../bindings";
import raw from "./fixtures/sankranti.json?raw";

export interface SankrantiFixture {
  /** Listenin yeri: Delhi, `Asia/Kolkata`. */
  place: MuhurtaPlace;
  /** `sankranti_list(2026, place, varsayılan ayarlar)`: on iki giriş, uyarı yok. */
  list: SankrantiListView;
  /** Aynı yıl, Tromsø: anlar aynıdır; `SunriseUnavailable` uyarısı vardır. */
  list_polar: SankrantiListView;
}

const FIXTURE = JSON.parse(raw) as SankrantiFixture;

/** Her çağrıda derin kopya: testler sonucu değiştirebilir. */
export function sankrantiFixture(): SankrantiFixture {
  return structuredClone(FIXTURE);
}
