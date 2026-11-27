// Gerçek veri fikstürü (app/src-tauri/tests/ui_fixture.rs üretir): Delhi 2000-01-01 17:30
// için `create_client` dönüşü, ilk 3. seviye dönemin alt dönemleri, "new delhi" atlas araması,
// an önizlemesi, bir DST boşluğu hatası, aynı sabit saatte transit, bakış tablosu, Jaimini raporu,
// chara dasha, Shadbala raporu, yoga raporu (kataloğuyla), ikinci bir danışanla uyum raporları,
// gün panchangı (iki gün), üç dilimlik bir muhurta araması ve iki prashna haritası (olgularıyla).
// Arayüz hesap yapmadığı için testler bunu çizer.
import type {
  CharaDasha,
  ChartBrief,
  ChartInput,
  CommandError,
  DashaPeriod,
  DayPanchangView,
  DrishtiTable,
  Graha,
  HomePlace,
  JaiminiReport,
  Kuta,
  KutaReport,
  LordLink,
  MomentPreview,
  MuhurtaFlag,
  MuhurtaPlace,
  MuhurtaQuery,
  MuhurtaSearchView,
  NakshatraNature,
  NatalMoon,
  OpenedClient,
  PlaceHit,
  PrashnaFacts,
  PrefsLoad,
  RisingMode,
  ShadbalaReport,
  Sign,
  SignDashaPath,
  SignDashaPeriod,
  TithiGroup,
  TransitView,
  YogaGroup,
  YogaReport,
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
  /** `chara_dasha_children`: transit anındaki alt dönemin (`chara_dasha_at` yolunun ilk iki burcu) 3. seviyesi. */
  chara_dasha_at_children: SignDashaPeriod[];
  /** `compute_shadbala`, aynı girdiyle: yedi graha, zaman lordları, on iki bhava. */
  shadbala: ShadbalaReport;
  /** `compute_yogas`, aynı girdiyle: 41 kural değerlendirilir. */
  yogas: YogaReport;
  /** `compute_yogas`, `time_known: false` ile: lagna isteyen 17 kural atlanır, kayıtlar yaklaşıktır. */
  yogas_time_unknown: YogaReport;
  /** Rust `YOGA_CATALOG`: kural kimlikleri ve grupları, katalog sırasıyla. */
  yoga_catalog: { id: string; group: YogaGroup }[];
  /** Rust `YOGA_CONDITIONS`: alt koşul kimlikleri. */
  yoga_conditions: string[];
  /**
   * Uyum: ikinci danışan ("Eş Danışan", İstanbul 1987-03-17) eş haritadır. `input` ve `brief`
   * store'un `chart_input` ve `chart_brief` dönüşleridir. `report` açık harita damat rolündeyken,
   * `report_swapped` roller değişmişken, `report_time_unknown` eşin (gelin rolünde) doğum saati
   * bilinmezken `compute_compat` dönüşüdür.
   */
  compat: {
    partner_uuid: string;
    brief: ChartBrief;
    input: ChartInput;
    report: KutaReport;
    report_swapped: KutaReport;
    report_time_unknown: KutaReport;
  };
  /** Rust `kuta_class_ids`: her kutanın alabileceği sınıf kimlikleri. */
  kuta_classes: Record<Kuta, string[]>;
  /**
   * Muhurta: `place` açık haritanın doğum yeridir (Delhi, Asia/Kolkata), `natal` açık haritanın
   * Ay'ı, `home_place` aynı yerin `AppPrefs.home_place` biçimi. `day`, fikstürün sabit saatini
   * ("şimdi") içeren günün `day_panchang` dönüşüdür; `day_polar` kutup gecesinde bir gündür
   * (Tromsø, 2050-12-21: dilimler hesaplanmaz). `pages`, `query` için `muhurta_search`'ün üç
   * diliminin dönüşüdür (67 gün; Perşembe gündüzleri, üç işaretli dilim dışlanmış).
   */
  muhurta: {
    natal: NatalMoon;
    place: MuhurtaPlace;
    home_place: HomePlace;
    day: DayPanchangView;
    day_polar: DayPanchangView;
    query: MuhurtaQuery;
    pages: MuhurtaSearchView[];
  };
  /**
   * Prashna: fikstürün sabit saatinde ("şimdi"), açık danışanın altında ve Delhi'de kurulmuş iki
   * harita. `opened` zaman prashnasının, `opened_number` 54 sayılı sayı prashnasının
   * `create_prashna` dönüşüdür (deneme sorusu; odak evi 7). `facts` zaman prashnasının,
   * `facts_number` sayı prashnasının odak evi 7 ile, `facts_no_focus` zaman prashnasının odak evi
   * seçilmeden `compute_prashna_facts` dönüşüdür.
   */
  prashna: {
    opened: OpenedClient;
    opened_number: OpenedClient;
    facts: PrashnaFacts;
    facts_no_focus: PrashnaFacts;
    facts_number: PrashnaFacts;
  };
  /** Rust `RisingMode::ALL`, `LordLink::ALL`, `PRASHNA_NUMBER_MAX` ve `MAX_QUESTION_CHARS`. */
  prashna_ids: { rising: RisingMode[]; links: LordLink[]; max_number: number; max_question_chars: number };
  /** Rust `MuhurtaFlag::ALL`, `TithiGroup::ALL`, `NakshatraNature::ALL` ve `MUHURTA_MAX_MIN_MINUTES`. */
  muhurta_ids: { flags: MuhurtaFlag[]; tithi_groups: TithiGroup[]; nakshatra_natures: NakshatraNature[]; max_min_minutes: number };
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
