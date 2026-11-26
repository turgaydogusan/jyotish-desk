// Gerçek veri fikstürü (app/src-tauri/tests/ui_fixture.rs üretir): Delhi 2000-01-01 17:30
// için `create_client` dönüşü, ilk 3. seviye dönemin alt dönemleri, "new delhi" atlas araması,
// an önizlemesi, bir DST boşluğu hatası, aynı sabit saatte transit, bakış tablosu, Jaimini raporu
// ve chara dasha. Arayüz hesap yapmadığı için testler bunu çizer.
import type {
  CharaDasha,
  CommandError,
  DashaPeriod,
  DrishtiTable,
  Graha,
  JaiminiReport,
  MomentPreview,
  OpenedClient,
  PlaceHit,
  PrefsLoad,
  Sign,
  SignDashaPath,
  SignDashaPeriod,
  TransitView,
} from "../bindings";
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
  /** `compute_jaimini`, aynı girdiyle (varsayılan ayarlar: sekizli şema, eş lordlar açık). */
  jaimini: JaiminiReport;
  /** `compute_chara_dasha`, aynı girdiyle: iki seviye dönem, iki çevrim. */
  chara_dasha: CharaDasha;
  /** `chara_dasha_children`: ilk dashanın ilk alt döneminin 3. seviyesi. */
  chara_dasha_children: { path: Sign[]; cycle: number; periods: SignDashaPeriod[] };
  /** `chara_dasha_at`, fikstürdeki transit anı için (`transit.snapshot.jd_ut`). */
  chara_dasha_at: SignDashaPath;
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
