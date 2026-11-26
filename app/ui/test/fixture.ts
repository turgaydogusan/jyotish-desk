// Gerçek veri fikstürü (app/src-tauri/tests/ui_fixture.rs üretir): Delhi 2000-01-01 17:30
// için `create_client` dönüşü, ilk 3. seviye dönemin alt dönemleri, "new delhi" atlas araması,
// an önizlemesi, bir DST boşluğu hatası, aynı sabit saatte transit ve bakış tablosu. Arayüz hesap yapmadığı için testler bunu çizer.
import type { CommandError, DashaPeriod, DrishtiTable, Graha, MomentPreview, OpenedClient, PlaceHit, PrefsLoad, TransitView } from "../bindings";
import raw from "./fixtures/delhi.json?raw";

export interface UiFixture {
  opened: OpenedClient;
  dasha_children: { path: Graha[]; periods: DashaPeriod[] };
  places: PlaceHit[];
  preview: MomentPreview;
  gap_error: CommandError;
  /** `transits_at(Now)`, saat 1 795 600 800 (fikstürün sabit saati). */
  transit: TransitView;
  /** `get_app_prefs` boş store için: varsayılanlar, uyarı yok. */
  prefs: PrefsLoad;
  /** `compute_drishti`, açık haritanın girdisiyle (`time_known: true`). */
  drishti: DrishtiTable;
}

const FIXTURE = JSON.parse(raw) as UiFixture;

/** Her çağrıda derin kopya: testler sonucu değiştirebilir. */
export function fixture(): UiFixture {
  return structuredClone(FIXTURE);
}

/** Saati bilinmeyen doğum: aynı harita, `time_certainty: "Unknown"` ve öğle uyarısı. */
export function unknownTimeOpened(): OpenedClient {
  const o = fixture().opened;
  o.chart_record.birth.time = null;
  o.chart_record.birth.time_certainty = "Unknown";
  o.chart_record.resolved.warnings = ["TimeUnknownNoon"];
  return o;
}
