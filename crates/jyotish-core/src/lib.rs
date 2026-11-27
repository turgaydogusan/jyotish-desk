//! Saf, yan etkisiz Jyotish hesabı. Ephemeris'e yalnızca `ephemeris::Ephemeris`
//! trait'i üzerinden erişir; I/O, Tauri ve atlas bağımlılığı yoktur.
//!
//! Kapsam: ayanamsa, grahalar, lagna, bhavalar, nakshatra, dignity, yanma (M2), 16 varga,
//! Vimshottari dasha, panchang, ashtakavarga ve transit (M3). İleri modüllerin ortak temeli
//! (Alt proje 3.1): `positions` (ortak girdi), `lordship`, `nature` ve `drishti`. Jaimini
//! çekirdeği (Alt proje 3.2) ve chara dasha (Alt proje 3.3): `jaimini`. Shadbala ve Bhava bala
//! (Alt proje 3.4): `shadbala`. Yogalar (Alt proje 3.5): `yogas`. Uyum (Alt proje 3.6): `kuta`.
//! Muhurta (Alt proje 3.7): `timeline` (aralık dizileri) ve `muhurta` (gün panchangı, arama).
//! Prashna (Alt proje 3.8): `prashna` (sayı prashnasının lagnası, soru haritasının olguları).
//! Arama (Alt proje 3.9): `search` (harita araması sorgusu ve hafif yol, olay araması).
//! Doğum anı ekleri (yol haritası Plan 23): `natal_extras` (panchang ekleri, navatara, Vimshopaka).

// Bhava tablosu ve Sudarshana chakra (yol haritası Plan 25). Ayrı blok: paralel planların
// alfabetik sıradaki eklemeleriyle aynı satırlara dokunmaz. `nakshatra_lord` köke yalnızca
// `natal_extras`'tan çıkar (tek tanım oradadır); `bhava::nakshatra_lord` aynı fonksiyonun ikinci
// yoludur ve kökten yeniden dışa aktarılmaz.
pub mod bhava;
pub use bhava::{
    bhava_point, bhava_table, compute_bhava_table, compute_sudarshana, porphyry_points,
    sandhis_from_mids, signed_arc, sudarshana, whole_sign_mids, BhavaPoint, BhavaRow, BhavaTable,
    GrahaBhava, SudarshanaChakra, SudarshanaHouse, SudarshanaRef, SudarshanaRing, BHAVA_METHOD,
    SUDARSHANA_METHOD,
};

pub mod ashtakavarga;
pub mod avastha;
mod chart;
mod combustion;
pub mod dasha;
pub mod dignity;
pub mod drishti;
mod error;
#[cfg(test)]
mod fake;
mod graha;
mod houses;
pub mod jaimini;
pub mod kuta;
pub mod lordship;
pub mod muhurta;
pub mod natal_extras;
pub mod nature;
pub mod panchang;
pub mod positions;
pub mod prashna;
#[cfg(test)]
mod proptests;
pub mod search;
pub mod settings;
pub mod shadbala;
pub mod timeline;
pub mod transit;
pub mod varga;
pub mod yogas;
pub mod zodiac;

pub use ashtakavarga::{
    compute_ashtakavarga, Ashtakavarga, ASHTAKAVARGA_GRAHAS, BAV_TOTALS, SAV_TOTAL,
};
pub use chart::{
    compute_chart, compute_chart_for, compute_chart_with_lagna, Chart, ChartMeta, ChartWarning,
    ENGINE_VERSION,
};
pub use combustion::is_combust;
pub use dasha::{
    birth_dasha, dasha_children, dasha_path_at, mahadashas, vimshottari, vimshottari_years, Dasha,
    DashaPeriod, DashaSeed, CHART_DASHA_LEVELS, MAX_DASHA_LEVEL, VIMSHOTTARI_ORDER,
    VIMSHOTTARI_TOTAL_YEARS,
};
pub use dignity::{dignity, Dignity, Relation};
pub use drishti::{
    compute_drishti, drishti_table, graha_aspect_houses, graha_aspect_signs, sphuta_drishti,
    DrishtiTable, SignAspect, ASPECTING_GRAHAS, DRISHTI_METHOD, FULL_DRISHTI,
};
pub use error::CoreError;
pub use graha::{Graha, GrahaPosition};
pub use houses::{house_from_cusps, whole_sign_cusps, whole_sign_house, Bhavas, Lagna};
pub use jaimini::{
    argala_row, argala_rows, argala_status, arudha_of, arudha_padas, chara_dasha,
    chara_dasha_forward, chara_dasha_order, chara_dasha_sign_years, chara_dasha_years,
    chara_karakas, co_lord_choice, co_lords, compute_chara_dasha, compute_chara_dasha_at,
    compute_chara_dasha_children, compute_jaimini, counts_forward, directed_count,
    graha_rashi_drishti, jaimini_lord, jaimini_report, karaka_roles, karakamsha, navamsa_sign,
    rashi_drishti_matrix, rashi_drishti_signs, second_cycle_years, sign_dasha_children,
    sign_dasha_path_at, sign_kind, sign_sequence, stronger_co_lord, sub_period_signs, ArgalaPair,
    ArgalaRow, ArgalaStatus, ArudhaPada, CharaDasha, CharaKaraka, CoLordChoice, JaiminiReport,
    KarakaRole, Karakamsha, SignDashaPath, SignDashaPeriod, SignKind, ARGALA_PAIRS,
    CHARA_DASHA_HORIZON_YEARS, CHARA_DASHA_LEVELS, CHARA_DASHA_METHOD, EIGHT_KARAKA_ROLES,
    JAIMINI_METHOD, KARAKA_TIE_DEG, MAX_SIGN_DASHA_LEVEL, SEVEN_KARAKA_ROLES, THIRD_HOUSE_MALEFICS,
};
pub use kuta::{
    compute_compat, kuta_class_ids, kuta_report, kuta_scores, Kuta, KutaReport, KutaRole,
    KutaScore, MoonDayChange, MoonPoint, OverlayRow, KUTA_METHOD, KUTA_TOTAL_MAX,
};
pub use lordship::{house_lord, house_sign, houses_owned};
pub use muhurta::{
    chandra_bala, day_panchang, day_segments, muhurta_chunks, muhurta_search_page,
    nakshatra_nature, tara_bala, tithi_group, window_snapshot, ChandraBala, DayKarana,
    DayNakshatra, DayPanchang, DaySegments, DayTithi, MuhurtaCriteria, MuhurtaCursor,
    MuhurtaFilter, MuhurtaFlag, MuhurtaSearchPage, MuhurtaWindow, NakshatraNature, NatalMoon,
    PersonalFactors, TaraBala, TithiGroup, WindowSnapshot, MUHURTA_CHUNK_DAYS,
    MUHURTA_MAX_RANGE_DAYS, MUHURTA_METHOD,
};
pub use natal_extras::{
    avayogi_nakshatra, compute_navatara, compute_panchang_extras, compute_vimshopaka, dagdha_rasis,
    nakshatra_lord, navatara, navatara_rows, panchang_extras, rasi_frame_offset, rung_points,
    tara_cycle, transit_taras, varga_relation, vimshopaka, vimshopaka_score, yogi_facts,
    yogi_point, GrahaVimshopaka, Navatara, NavataraRow, PanchangExtras, TransitTara, VargaScheme,
    VimshopakaReport, VimshopakaVarga, YogiFacts, AVAYOGI_COUNT, NAVATARA_METHOD,
    PANCHANG_EXTRAS_METHOD, VIMSHOPAKA_GRAHAS, VIMSHOPAKA_MAX, VIMSHOPAKA_METHOD, YOGI_ARC_DEG,
};
pub use nature::{kuja_dosha, moon_is_waxing, nature, KujaDosha, Nature, KUJA_DOSHA_HOUSES};
pub use panchang::{
    compute_panchang, elongation, karana_name, karana_no, local_weekday, nitya_yoga_no, paksha,
    tithi_no, Karana, KaranaName, NityaYoga, Paksha, Panchang, PanchangNakshatra, Tithi, Vara,
    END_BISECTIONS, END_SEARCH_DAYS, KARANA_SPAN_DEG, SUNRISE_WINDOW_DAYS, TITHI_SPAN_DEG,
};
pub use positions::{compute_positions, ChartInput, GrahaPoint, Positions};
pub use prashna::{
    compute_prashna_facts, lord_relation, moon_facts, number_lagna_lon, prashna_facts, rising_mode,
    FocusFacts, HouseOccupants, LagnaFacts, LordFacts, LordLink, LordRelation, MoonFacts, Occupant,
    PrashnaFacts, RisingMode, PRASHNA_METHOD, PRASHNA_NUMBER_MAX,
};
pub use search::{
    date_noon_jd, evaluate, event_chunks, event_search, event_search_chunk, search_facts,
    ChartQuery, Condition, EventSpec, GrahaFlag, MatchFact, Outcome, QueryCondition, QueryScope,
    ScopeKind, SearchFacts, SkyEvent, SkyEventKind, EVENT_CHUNK_DAYS, EVENT_MAX_RANGE_DAYS,
    EVENT_METHOD, EVENT_MOON_CHUNK_DAYS, EVENT_MOON_MAX_RANGE_DAYS, HOUSE_CHANGE_GRAHAS,
    MAX_CONDITIONS, MAX_SCOPE_TAGS, MAX_SCOPE_TAG_CHARS, MAX_SEARCH_DASHA_LEVEL, MAX_SKY_EVENTS,
    MAX_YOGA_IDS, SEARCH_METHOD,
};
pub use settings::{
    AdvancedSettings, AyanamsaSystem, CalcSettings, CombustionOrbs, HouseSystem, KarakaScheme,
    NodeKind, SunriseDef, ZodiacMode, CALC_SETTINGS_SCHEMA_VERSION, JD_UT_RANGE,
};
pub use shadbala::{
    ayana_bala, ayana_declination, bhava_balas, bhava_dig_bala, bhava_midpoint, bhava_sign_kind,
    cheshta_bala, cheshta_from, circular_midpoint, compute_shadbala, compute_shadbala_astro,
    day_night_bala, dig_bala, drekkana_bala, drik_bala, drik_bala_at, graha_war, hora_index,
    hora_lord, hours_from_noon, is_daytime, kendradi_bala, mean_longitudes, naisargika_bala,
    ojayugma_bala, paksha_bala, paksha_elongation, rasi_lon, required_bala, saptavargaja_bala,
    saptavargaja_points, shadbala, sthana_bala, sun_entry_before, temporal_bala, time_lords,
    tribhaga_lord, uccha_bala, vara_lord, BhavaBala, BhavaSignKind, GrahaShadbala, GrahaWar,
    MeanLongitudes, ShadbalaAstro, ShadbalaReport, SthanaBala, TemporalBala, TimeLords,
    GRAHA_WAR_ORB_DEG, HORA_CYCLE, MAX_SUNRISE_GAP_DAYS, SAPTAVARGA, SHADBALA_GRAHAS,
    SHADBALA_METHOD, VIRUPAS_PER_RUPA, WAR_GRAHAS,
};
pub use timeline::{
    combust_series, graha_motion_series, graha_nakshatra_series, graha_sign_series, karana_series,
    lagna_sign_at, lagna_sign_series, nakshatra_series, nitya_yoga_series, sun_days, tithi_series,
    vara_series, Interval, Motion, NakshatraPada, SunDay, STATION_GRAHAS, TIMELINE_RESOLUTION_DAYS,
};
pub use transit::{compute_transits, TransitGraha, TransitSnapshot};
pub use varga::{
    all_vargas, is_vargottama, varga_chart, varga_position, Varga, VargaChart, VargaPlacement,
};
pub use yogas::{
    compute_yogas, exchange_kind, is_exchange, is_mutual_aspect, occupied_sign_count,
    yoga_relations, yoga_rule, yogas, ExchangeKind, YogaGroup, YogaHit, YogaMatch, YogaRelation,
    YogaReport, YogaRule, COUNTED_GRAHAS, DHANA_HOUSES, DUSTHANA_HOUSES, KENDRA_HOUSES,
    TRIKONA_HOUSES, UPACHAYA_HOUSES, YOGA_CATALOG, YOGA_CATALOG_VERSION, YOGA_CONDITIONS,
    YOGA_GRAHAS,
};
pub use zodiac::{angular_distance, deg_in_sign, nakshatra_pada, normalize, Sign};

// Geçiş araçları (yol haritası Plan 26). Ayrı blok, dosyanın sonunda: eşzamanlı planların
// alfabetik gruplardaki ve dosya başındaki ekleriyle aynı satırlara dokunmaz.
pub mod transit_tools;
pub use transit_tools::{
    ephemeris_table, sankranti_list, sankranti_moments, EphemerisCell, EphemerisRow,
    EphemerisTable, Sankranti, SankrantiList, EPHEMERIS_MAX_ROWS, EPHEMERIS_MAX_STEP_DAYS,
    EPHEMERIS_METHOD, NAKSHATRA_LORDS, SANKRANTI_MAX_RANGE_DAYS, SANKRANTI_METHOD,
};
