//! Pencere araması (ileri modüller spec'i §10): süzgece uyan zaman pencereleri. Her etkin
//! koşul `timeline`'dan bir aralık dizisi ister; pencereler bu dizilerin kesişimidir.
//!
//! Arama dilim dilim yürür (`MUHURTA_CHUNK_DAYS`): her çağrı bir dilimi hesaplar ve sonraki
//! çağrının imlecini döner. Dilim sınırını aşan pencere bölünmez: sınırda açık duran pencerenin
//! başı imleçte taşınır ve pencere kapandığı dilimde verilir. Çekirdek durum tutmaz; aramayı
//! durdurmak, sonraki dilimi istememektir.

use ephemeris::{Body, Ephemeris, Precision, Zodiac};
use serde::{Deserialize, Serialize};

use crate::chart::{finite, rasi_frame, ChartWarning};
use crate::graha::Graha;
use crate::panchang::{
    karana_name, karana_no, nitya_yoga_no, paksha, tithi_no, vara_at, KaranaName, Paksha, Vara,
};
use crate::settings::CalcSettings;
use crate::timeline::{
    check_span, combust_series, graha_sign_series, karana_series, lagna_sign_at, lagna_sign_series,
    nakshatra_series, nitya_yoga_series, sun_days, sun_days_cover, tithis_of_karanas,
    vara_of_sun_days, vara_series, Interval,
};
use crate::zodiac::{nakshatra_pada, normalize, Sign};
use crate::CoreError;

use super::segments::{
    chandra_bala, day_segments, is_panchaka, is_rikta, is_vishti, tara_bala, VAIDHRITI_YOGA,
    VYATIPATA_YOGA,
};
use super::{MuhurtaFlag, NatalMoon, MUHURTA_METHOD};

/// Aramanın en uzun tarih aralığı, gün.
pub const MUHURTA_MAX_RANGE_DAYS: f64 = 366.0;
/// Bir çağrının hesapladığı dilim, gün. Motor iş parçacığı dilim aralarında bırakılır.
pub const MUHURTA_CHUNK_DAYS: f64 = 31.0;
/// "En kısa süre" süzgecinin üst sınırı, dakika (bir hafta).
pub const MUHURTA_MAX_MIN_MINUTES: u32 = 7 * 24 * 60;
/// Bundan kısa pencere verilmez (1 saniye). İki ayrı dizinin aynı ana düşen sınırları (ör. Ay'ın
/// burç girişi ile nakshatra girişi) ayrı ikiye bölmelerle bulunur ve çözünürlük kadar
/// ayrışabilir; biri kapanırken diğeri açılan iki koşul arasında oluşan bu kıymık pencere değildir.
pub const MUHURTA_MIN_WINDOW_DAYS: f64 = 1.0 / 86_400.0;

/// Süzgecin saklanabilen kısmı: koşullar. Boş küme "kısıt yok" demektir. Tarih aralığı, yer ve
/// kişi burada değildir (kayıtlı süzgeç kişi ve yer verisi taşımaz).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MuhurtaCriteria {
    pub weekdays: Vec<Vara>,
    /// 1..=30 (1–15 Shukla, 16–30 Krishna).
    pub tithis: Vec<u8>,
    /// 1..=27.
    pub nakshatras: Vec<u8>,
    pub lagna_signs: Vec<Sign>,
    /// Yalnızca gün doğumu ile batımı arası.
    pub daytime_only: bool,
    /// Pencerenin en kısa süresi, dakika; 0 kısıt yok.
    pub min_minutes: u32,
    pub exclude: Vec<MuhurtaFlag>,
    /// Tara bala elverişli (doğum Ay'ı gerekir).
    pub require_tara: bool,
    /// Chandra bala elverişli (doğum Ay'ı gerekir).
    pub require_chandra: bool,
}

impl MuhurtaCriteria {
    /// Değer aralıklarını denetler; hata alan adını taşır.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.tithis.iter().any(|t| !(1..=30).contains(t)) {
            return Err(CoreError::InvalidMuhurtaInput("tithis"));
        }
        if self.nakshatras.iter().any(|n| !(1..=27).contains(n)) {
            return Err(CoreError::InvalidMuhurtaInput("nakshatras"));
        }
        if self.min_minutes > MUHURTA_MAX_MIN_MINUTES {
            return Err(CoreError::InvalidMuhurtaInput("min_minutes"));
        }
        Ok(())
    }

    /// Kümeleri sıralar ve yinelenenleri atar (kayıt ve karşılaştırma için kanonik biçim).
    pub fn normalized(mut self) -> Self {
        fn tidy<T: Ord>(v: &mut Vec<T>) {
            v.sort();
            v.dedup();
        }
        self.weekdays.sort_by_key(|v| *v as u8);
        self.weekdays.dedup();
        tidy(&mut self.tithis);
        tidy(&mut self.nakshatras);
        tidy(&mut self.lagna_signs);
        tidy(&mut self.exclude);
        self
    }

    fn excludes(&self, flag: MuhurtaFlag) -> bool {
        self.exclude.contains(&flag)
    }
}

/// Aramanın tam girdisi: tarih aralığı, yer, koşullar ve (kişiye bağlı koşullar için) doğum
/// Ay'ı.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MuhurtaFilter {
    pub from_jd_ut: f64,
    pub to_jd_ut: f64,
    /// Kuzey +.
    pub lat: f64,
    /// Doğu +.
    pub lon: f64,
    pub criteria: MuhurtaCriteria,
    pub natal: Option<NatalMoon>,
}

impl MuhurtaFilter {
    pub fn validate(&self) -> Result<(), CoreError> {
        check_span(self.from_jd_ut, self.to_jd_ut)?;
        if self.to_jd_ut - self.from_jd_ut > MUHURTA_MAX_RANGE_DAYS {
            return Err(CoreError::InvalidMuhurtaInput("range"));
        }
        if !((-90.0..=90.0).contains(&self.lat) && (-180.0..=180.0).contains(&self.lon)) {
            return Err(CoreError::InvalidMuhurtaInput("place"));
        }
        self.criteria.validate()?;
        match &self.natal {
            Some(n) => n.validate(),
            None if self.criteria.require_tara || self.criteria.require_chandra => {
                Err(CoreError::InvalidMuhurtaInput("natal"))
            }
            None => Ok(()),
        }
    }
}

/// Pencerenin başındaki beş uzuv ve lagna burcu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct WindowSnapshot {
    pub vara: Vara,
    pub tithi: u8,
    pub paksha: Paksha,
    pub nakshatra: u8,
    pub pada: u8,
    pub yoga: u8,
    pub karana: u8,
    pub karana_name: KaranaName,
    pub lagna_sign: Sign,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MuhurtaWindow {
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
    pub snapshot: WindowSnapshot,
}

/// Sonraki dilimin imleci. Arayüz bunu olduğu gibi geri gönderir.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MuhurtaCursor {
    /// Hesaplanacak dilimin sırası, 0'dan.
    pub chunk: u32,
    /// Önceki dilimin sonunda açık duran pencerenin başı.
    pub open_start_jd_ut: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MuhurtaSearchPage {
    /// Bu dilimde kapanan pencereler, başlangıç sırasıyla.
    pub windows: Vec<MuhurtaWindow>,
    /// Taranan dilim sayısı (bu dilim dahil) ve toplam.
    pub scanned: u32,
    pub chunks: u32,
    /// Sonraki dilimin imleci; arama bittiyse `None`.
    pub next: Option<MuhurtaCursor>,
    /// `SunriseUnavailable`: dilimde doğuşu ya da batışı olmayan gün var ve süzgeç gündüze ya da
    /// gündüz dilimlerine bağlı. `LowPrecision`: Moshier yedeği.
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

/// Aralığın dilim sayısı.
pub fn muhurta_chunks(from_jd_ut: f64, to_jd_ut: f64) -> u32 {
    chunk_count(from_jd_ut, to_jd_ut, MUHURTA_CHUNK_DAYS)
}

fn chunk_count(from: f64, to: f64, chunk_days: f64) -> u32 {
    (((to - from) / chunk_days).ceil() as u32).max(1)
}

type Spans = Vec<(f64, f64)>;

/// Koşulu sağlayan aralıklar, birleştirilmiş.
fn spans_where<T>(series: &[Interval<T>], holds: impl Fn(&T) -> bool) -> Spans {
    let mut out: Spans = Vec::new();
    for i in series.iter().filter(|i| holds(&i.value)) {
        match out.last_mut() {
            Some(last) if last.1 == i.start_jd_ut => last.1 = i.end_jd_ut,
            _ => out.push((i.start_jd_ut, i.end_jd_ut)),
        }
    }
    out
}

/// İki sıralı, ayrık aralık listesinin kesişimi.
fn intersect(a: &Spans, b: &Spans) -> Spans {
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < a.len() && j < b.len() {
        let start = a[i].0.max(b[j].0);
        let end = a[i].1.min(b[j].1);
        if start < end {
            out.push((start, end));
        }
        if a[i].1 <= b[j].1 {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

/// `a`'dan `cut` aralıklarının çıkarılması (`cut` sıralı ve ayrık).
fn subtract(a: &Spans, cut: &Spans) -> Spans {
    let mut out = Vec::new();
    for &(mut start, end) in a {
        for &(c0, c1) in cut {
            if c1 <= start || c0 >= end {
                continue;
            }
            if c0 > start {
                out.push((start, c0));
            }
            start = start.max(c1);
        }
        if start < end {
            out.push((start, end));
        }
    }
    out
}

/// `jd_ut` anının beş uzvu ve lagna burcu (noktasal; `compute_panchang` ve `compute_chart`
/// ile aynı sorgular).
pub fn window_snapshot<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    jd_ut: f64,
    geo_lat: f64,
    geo_lon: f64,
) -> Result<WindowSnapshot, CoreError> {
    let zodiac = Zodiac::Sidereal(settings.ayanamsa.sidereal_mode());
    let sun = finite(eph.body(jd_ut, Body::Sun, zodiac)?.lon, "panchang")?;
    let moon = finite(eph.body(jd_ut, Body::Moon, zodiac)?.lon, "panchang")?;
    let elongation = normalize(moon - sun);
    let tithi = tithi_no(elongation);
    let (nakshatra, pada) = nakshatra_pada(moon);
    let karana = karana_no(elongation);
    Ok(WindowSnapshot {
        vara: vara_at(
            eph,
            jd_ut,
            geo_lat,
            geo_lon,
            settings.sunrise_def.sun_disc(),
        )?,
        tithi,
        paksha: paksha(tithi),
        nakshatra,
        pada,
        yoga: nitya_yoga_no(sun, moon),
        karana,
        karana_name: karana_name(karana),
        lagna_sign: lagna_sign_at(eph, settings, jd_ut, geo_lat, geo_lon)?,
    })
}

/// Bir dilimde süzgeci sağlayan aralıklar ve uyarılar.
fn allowed_spans<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    filter: &MuhurtaFilter,
    a: f64,
    b: f64,
) -> Result<(Spans, Vec<ChartWarning>), CoreError> {
    let c = &filter.criteria;
    let (lat, lon) = (filter.lat, filter.lon);
    let mut allowed: Spans = vec![(a, b)];
    let mut warnings = Vec::new();
    if eph.body(a, Body::Sun, rasi_frame(settings))?.precision == Precision::Moshier {
        warnings.push(ChartWarning::LowPrecision);
    }

    let segment_flags: Vec<MuhurtaFlag> = MuhurtaFlag::ALL
        .into_iter()
        .filter(|f| f.is_day_segment() && c.excludes(*f))
        .collect();
    let needs_days = c.daytime_only || !segment_flags.is_empty();
    if needs_days || !c.weekdays.is_empty() {
        let days = sun_days(eph, settings, lat, lon, a, b)?;
        let covered = sun_days_cover(&days, a, b);
        if !c.weekdays.is_empty() {
            let varas = if covered {
                vara_of_sun_days(&days, a, b)
            } else {
                vara_series(eph, settings, lat, lon, a, b)?
            };
            allowed = intersect(&allowed, &spans_where(&varas, |v| c.weekdays.contains(v)));
        }
        if needs_days && !covered {
            warnings.push(ChartWarning::SunriseUnavailable);
        }
        if c.daytime_only {
            let daylight: Spans = days
                .iter()
                .map(|d| (d.sunrise_jd_ut, d.sunset_jd_ut))
                .collect();
            allowed = intersect(&allowed, &daylight);
        }
        if !segment_flags.is_empty() {
            let mut cut: Spans = Vec::new();
            for d in &days {
                let s = day_segments(d);
                let mut of_day: Spans = segment_flags
                    .iter()
                    .map(|f| match f {
                        MuhurtaFlag::RahuSegment => s.rahu,
                        MuhurtaFlag::Yamaganda => s.yamaganda,
                        _ => s.gulika,
                    })
                    .map(|i| (i.start_jd_ut, i.end_jd_ut))
                    .collect();
                of_day.sort_by(|x, y| x.0.total_cmp(&y.0));
                cut.extend(of_day);
            }
            allowed = subtract(&allowed, &cut);
        }
    }

    let (rikta, vishti) = (
        c.excludes(MuhurtaFlag::Rikta),
        c.excludes(MuhurtaFlag::Vishti),
    );
    if !c.tithis.is_empty() || rikta || vishti {
        let karanas = karana_series(eph, settings, a, b)?;
        if vishti {
            allowed = intersect(&allowed, &spans_where(&karanas, |k| !is_vishti(*k)));
        }
        if !c.tithis.is_empty() || rikta {
            let tithis = tithis_of_karanas(&karanas);
            allowed = intersect(
                &allowed,
                &spans_where(&tithis, |t| {
                    (c.tithis.is_empty() || c.tithis.contains(t)) && !(rikta && is_rikta(*t))
                }),
            );
        }
    }

    let panchaka = c.excludes(MuhurtaFlag::Panchaka);
    if !c.nakshatras.is_empty() || panchaka || c.require_tara {
        let series = nakshatra_series(eph, settings, a, b)?;
        allowed = intersect(
            &allowed,
            &spans_where(&series, |n| {
                (c.nakshatras.is_empty() || c.nakshatras.contains(&n.nakshatra))
                    && !(panchaka && is_panchaka(*n))
                    && match (&filter.natal, c.require_tara) {
                        (Some(natal), true) => tara_bala(natal.nakshatra, n.nakshatra).favourable,
                        _ => true,
                    }
            }),
        );
    }

    let (vyatipata, vaidhriti) = (
        c.excludes(MuhurtaFlag::Vyatipata),
        c.excludes(MuhurtaFlag::Vaidhriti),
    );
    if vyatipata || vaidhriti {
        let yogas = nitya_yoga_series(eph, settings, a, b)?;
        allowed = intersect(
            &allowed,
            &spans_where(&yogas, |y| {
                !(vyatipata && *y == VYATIPATA_YOGA) && !(vaidhriti && *y == VAIDHRITI_YOGA)
            }),
        );
    }

    if let (Some(natal), true) = (&filter.natal, c.require_chandra) {
        let signs = graha_sign_series(eph, settings, Graha::Moon, a, b)?;
        allowed = intersect(
            &allowed,
            &spans_where(&signs, |s| chandra_bala(natal.sign, *s).favourable),
        );
    }

    for (flag, graha) in [
        (MuhurtaFlag::JupiterCombust, Graha::Jupiter),
        (MuhurtaFlag::VenusCombust, Graha::Venus),
    ] {
        if c.excludes(flag) {
            let series = combust_series(eph, settings, graha, a, b)?;
            allowed = intersect(&allowed, &spans_where(&series, |combust| !combust));
        }
    }

    // En pahalı dizi sona kalır: diğer koşullar hiçbir an bırakmadıysa hesaplanmaz.
    if !c.lagna_signs.is_empty() && !allowed.is_empty() {
        let series = lagna_sign_series(eph, settings, lat, lon, a, b)?;
        allowed = intersect(
            &allowed,
            &spans_where(&series, |s| c.lagna_signs.contains(s)),
        );
    }
    Ok((allowed, warnings))
}

/// Aramanın bir dilimi. `cursor` yoksa ilk dilim hesaplanır; dönen `next` bir sonraki çağrıya
/// olduğu gibi verilir. Aynı süzgeç ve imleç her zaman aynı sayfayı verir.
pub fn muhurta_search_page<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    filter: &MuhurtaFilter,
    cursor: Option<MuhurtaCursor>,
) -> Result<MuhurtaSearchPage, CoreError> {
    search_page_with(eph, settings, filter, cursor, MUHURTA_CHUNK_DAYS)
}

pub(crate) fn search_page_with<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    filter: &MuhurtaFilter,
    cursor: Option<MuhurtaCursor>,
    chunk_days: f64,
) -> Result<MuhurtaSearchPage, CoreError> {
    settings.validate()?;
    filter.validate()?;
    let (from, to) = (filter.from_jd_ut, filter.to_jd_ut);
    let chunks = chunk_count(from, to, chunk_days);
    let cursor = cursor.unwrap_or(MuhurtaCursor {
        chunk: 0,
        open_start_jd_ut: None,
    });
    if cursor.chunk >= chunks {
        return Err(CoreError::InvalidMuhurtaInput("cursor"));
    }
    let a = from + f64::from(cursor.chunk) * chunk_days;
    let last = cursor.chunk + 1 == chunks;
    let b = if last {
        to
    } else {
        from + f64::from(cursor.chunk + 1) * chunk_days
    };
    // Açık pencerenin başı aralığın içinde ve bu dilimden önce olmalıdır.
    if cursor
        .open_start_jd_ut
        .is_some_and(|s| !(s >= from && s < a))
    {
        return Err(CoreError::InvalidMuhurtaInput("cursor"));
    }

    let (allowed, warnings) = allowed_spans(eph, settings, filter, a, b)?;
    let min_days = (f64::from(filter.criteria.min_minutes) / 1440.0).max(MUHURTA_MIN_WINDOW_DAYS);
    let mut closed: Spans = Vec::new();
    let mut open = cursor.open_start_jd_ut;
    let mut carry = None;
    for (start, end) in allowed {
        let start = match open.take() {
            // Önceki dilimde açık duran pencere bu dilimin başında sürüyor.
            Some(o) if start == a => o,
            // Sürmüyor: tam dilim sınırında kapanmış.
            Some(o) => {
                closed.push((o, a));
                start
            }
            None => start,
        };
        if end == b && !last {
            carry = Some(start);
        } else {
            closed.push((start, end));
        }
    }
    if let Some(o) = open {
        closed.push((o, a));
    }

    let mut windows = Vec::new();
    for (start, end) in closed {
        if end - start >= min_days {
            windows.push(MuhurtaWindow {
                start_jd_ut: start,
                end_jd_ut: end,
                snapshot: window_snapshot(eph, settings, start, filter.lat, filter.lon)?,
            });
        }
    }
    Ok(MuhurtaSearchPage {
        windows,
        scanned: cursor.chunk + 1,
        chunks,
        next: (!last).then_some(MuhurtaCursor {
            chunk: cursor.chunk + 1,
            open_start_jd_ut: carry,
        }),
        warnings,
        method: MUHURTA_METHOD.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::panchang::compute_panchang;

    const JD: f64 = 2_451_545.0;
    const TOL: f64 = 0.2 / 86_400.0;

    fn eph() -> FakeEph {
        FakeEph {
            asc_speed: 360.0,
            ..FakeEph::default()
        }
    }

    fn filter(days: f64, criteria: MuhurtaCriteria) -> MuhurtaFilter {
        MuhurtaFilter {
            from_jd_ut: JD,
            to_jd_ut: JD + days,
            lat: 41.0,
            lon: 29.0,
            criteria,
            natal: None,
        }
    }

    /// Bütün dilimleri sırayla ister.
    fn run(eph: &mut FakeEph, f: &MuhurtaFilter, chunk_days: f64) -> Vec<MuhurtaWindow> {
        let s = CalcSettings::default();
        let mut out = Vec::new();
        let mut cursor = None;
        let mut pages = 0;
        loop {
            let page = search_page_with(eph, &s, f, cursor, chunk_days).unwrap();
            pages += 1;
            assert_eq!(page.scanned, pages);
            assert_eq!(page.method, MUHURTA_METHOD);
            out.extend(page.windows);
            match page.next {
                Some(next) => cursor = Some(next),
                None => {
                    assert_eq!(page.scanned, page.chunks);
                    return out;
                }
            }
        }
    }

    #[test]
    fn span_sets_intersect_and_subtract() {
        let a: Spans = vec![(0.0, 4.0), (6.0, 10.0)];
        let b: Spans = vec![(1.0, 2.0), (3.0, 7.0), (9.0, 12.0)];
        assert_eq!(
            intersect(&a, &b),
            [(1.0, 2.0), (3.0, 4.0), (6.0, 7.0), (9.0, 10.0)]
        );
        assert_eq!(intersect(&a, &Vec::new()), []);
        assert_eq!(subtract(&a, &b), [(0.0, 1.0), (2.0, 3.0), (7.0, 9.0)]);
        assert_eq!(subtract(&a, &vec![(-1.0, 20.0)]), []);
        assert_eq!(subtract(&a, &Vec::new()), a);
        // Uçları değen aralıklar birleşir; koşulu sağlamayan araya girerse birleşmez.
        let series = [
            Interval {
                value: 1u8,
                start_jd_ut: 0.0,
                end_jd_ut: 1.0,
            },
            Interval {
                value: 2,
                start_jd_ut: 1.0,
                end_jd_ut: 2.0,
            },
            Interval {
                value: 9,
                start_jd_ut: 2.0,
                end_jd_ut: 3.0,
            },
            Interval {
                value: 3,
                start_jd_ut: 3.0,
                end_jd_ut: 4.0,
            },
        ];
        assert_eq!(spans_where(&series, |v| *v < 5), [(0.0, 2.0), (3.0, 4.0)]);
    }

    #[test]
    fn no_condition_gives_one_window_over_the_whole_range_across_chunks() {
        let mut e = eph();
        let f = filter(100.0, MuhurtaCriteria::default());
        for chunk in [31.0, 7.0, 100.0, 250.0] {
            let w = run(&mut e, &f, chunk);
            assert_eq!(w.len(), 1, "chunk {chunk}");
            assert_eq!((w[0].start_jd_ut, w[0].end_jd_ut), (JD, JD + 100.0));
        }
        assert_eq!(muhurta_chunks(JD, JD + 100.0), 4);
        assert_eq!(muhurta_chunks(JD, JD + 31.0), 1);
        assert_eq!(muhurta_chunks(JD, JD + 366.0), 12);
        // İlk sayfa pencereyi vermez: pencere son dilimde kapanır.
        let s = CalcSettings::default();
        let first = muhurta_search_page(&mut e, &s, &f, None).unwrap();
        assert!(first.windows.is_empty());
        assert_eq!((first.scanned, first.chunks), (1, 4));
        assert_eq!(
            first.next,
            Some(MuhurtaCursor {
                chunk: 1,
                open_start_jd_ut: Some(JD)
            })
        );
    }

    #[test]
    fn a_tithi_filter_finds_the_tithi_and_the_snapshot_describes_its_start() {
        // Uzanım 85° + 12,22°/gün: 9. tithi 96°–108°; 29,46 gün sonra yeniden.
        let mut e = eph();
        let f = filter(
            40.0,
            MuhurtaCriteria {
                tithis: vec![9],
                ..MuhurtaCriteria::default()
            },
        );
        let w = run(&mut e, &f, MUHURTA_CHUNK_DAYS);
        assert_eq!(w.len(), 2);
        assert!((w[0].start_jd_ut - (JD + 11.0 / 12.22)).abs() < TOL);
        assert!((w[0].end_jd_ut - (JD + 23.0 / 12.22)).abs() < TOL);
        assert!((w[1].start_jd_ut - (JD + 371.0 / 12.22)).abs() < TOL);
        let s = CalcSettings::default();
        for win in &w {
            let snap = win.snapshot;
            assert_eq!((snap.tithi, snap.paksha), (9, Paksha::Shukla));
            // Karana 17: (17 − 2) mod 7 = 1 → Balava.
            assert_eq!((snap.karana, snap.karana_name), (17, KaranaName::Balava));
            let p = compute_panchang(&mut e, win.start_jd_ut, 41.0, 29.0, &s).unwrap();
            assert_eq!(snap.vara, p.vara);
            assert_eq!(
                (snap.nakshatra, snap.pada),
                (p.nakshatra.no, p.nakshatra.pada)
            );
            assert_eq!(snap.yoga, p.yoga.no);
            assert_eq!(
                snap.lagna_sign,
                lagna_sign_at(&mut e, &s, win.start_jd_ut, 41.0, 29.0).unwrap()
            );
        }
        // İkinci pencere iki dilimi aşar (31. gün sınırı 30,36–31,34 arasındadır); dilim
        // uzunluğu sonucu değiştirmez.
        let small = run(&mut e, &f, 3.0);
        assert_eq!(small.len(), 2);
        for (x, y) in w.iter().zip(&small) {
            assert_eq!(x, y);
        }
    }

    #[test]
    fn daytime_and_segment_exclusions_cut_the_day() {
        // Sahte gün: doğuş n + 0,67, batış n + 1,15 (gündüz 0,48 gün; sekizde bir 0,06 gün).
        let mut e = eph();
        let day_only = MuhurtaCriteria {
            daytime_only: true,
            ..MuhurtaCriteria::default()
        };
        let w = run(&mut e, &filter(3.0, day_only.clone()), 31.0);
        // JD'de gündüzün içindeyiz: [JD, +0,15), sonra üç tam gündüz… sonuncusu aralıkta kesilir.
        assert_eq!(w.len(), 4);
        assert_eq!(w[0].start_jd_ut, JD);
        assert!((w[0].end_jd_ut - (JD + 0.15)).abs() < 1e-9);
        assert!((w[1].start_jd_ut - (JD + 0.67)).abs() < 1e-9);
        assert!((w[1].end_jd_ut - (JD + 1.15)).abs() < 1e-9);
        assert_eq!(w[3].end_jd_ut, JD + 3.0);

        // JD + 0,67'de başlayan gün Pazar'dır: Rahu 8., Yamaganda 5., Gulika 7. dilim.
        let cut = MuhurtaCriteria {
            exclude: vec![
                MuhurtaFlag::Gulika,
                MuhurtaFlag::RahuSegment,
                MuhurtaFlag::Yamaganda,
            ],
            ..day_only
        };
        let f = MuhurtaFilter {
            from_jd_ut: JD + 0.67,
            to_jd_ut: JD + 1.15,
            ..filter(1.0, cut)
        };
        let w = run(&mut e, &f, 31.0);
        let rise = JD + 0.67;
        let eighth = 0.06;
        // Geriye 1.–4. dilim ve 6. dilim durur.
        assert_eq!(w.len(), 2);
        assert!((w[0].start_jd_ut - rise).abs() < 1e-9);
        assert!((w[0].end_jd_ut - (rise + 4.0 * eighth)).abs() < 1e-9);
        assert!((w[1].start_jd_ut - (rise + 5.0 * eighth)).abs() < 1e-9);
        assert!((w[1].end_jd_ut - (rise + 6.0 * eighth)).abs() < 1e-9);
        assert_eq!(w[0].snapshot.vara, Vara::Sunday);
    }

    #[test]
    fn weekday_lagna_and_minimum_length_combine() {
        // Lagna sidereal 46° + 360°/gün: Yengeç 90°–120°, her gün 44/360'ta başlar ve 2 saat
        // sürer. Pazartesi günü JD + 1,67'de başlar.
        let mut e = eph();
        let criteria = MuhurtaCriteria {
            weekdays: vec![Vara::Monday],
            lagna_signs: vec![Sign::Cancer],
            ..MuhurtaCriteria::default()
        };
        let w = run(&mut e, &filter(10.0, criteria.clone()), 31.0);
        // 10 günde iki Pazartesi: JD + 1,67 … 2,67 ve JD + 8,67 … 9,67.
        assert_eq!(w.len(), 2);
        assert!((w[0].start_jd_ut - (JD + 2.0 + 44.0 / 360.0)).abs() < TOL);
        assert!((w[0].end_jd_ut - (JD + 2.0 + 74.0 / 360.0)).abs() < TOL);
        assert!((w[1].start_jd_ut - (JD + 9.0 + 44.0 / 360.0)).abs() < TOL);
        for win in &w {
            assert_eq!(win.snapshot.vara, Vara::Monday);
            assert_eq!(win.snapshot.lagna_sign, Sign::Cancer);
        }
        // Pencere 120 dakikadır: 120 dakikalık alt sınır geçer, 121 dakikalık geçmez.
        let at_least = |min_minutes| MuhurtaCriteria {
            min_minutes,
            ..criteria.clone()
        };
        assert_eq!(run(&mut e, &filter(10.0, at_least(119)), 31.0).len(), 2);
        assert_eq!(run(&mut e, &filter(10.0, at_least(121)), 31.0).len(), 0);
    }

    #[test]
    fn boundaries_of_two_series_that_fall_on_the_same_moment_leave_no_sliver_window() {
        // Panchaka Ay 360°'ye varınca biter; aynı anda Ay Balık'tan Koç'a geçer. Doğum Ay burcu
        // Boğa'ysa Balık elverişli (11. ev), Koç elverişsizdir (12. ev): biri açılırken diğeri
        // kapanır, pencere yoktur. İki sınır ayrı dizilerden gelir (nakshatra ve burç) ve
        // çözünürlük kadar ayrışabilir; dilim uzunluğu ne olursa olsun kıymık pencere verilmez.
        let mut e = eph();
        let f = MuhurtaFilter {
            natal: Some(NatalMoon {
                nakshatra: 4,
                sign: Sign::Taurus,
            }),
            ..filter(
                40.0,
                MuhurtaCriteria {
                    exclude: vec![MuhurtaFlag::Panchaka],
                    require_chandra: true,
                    ..MuhurtaCriteria::default()
                },
            )
        };
        let whole = run(&mut e, &f, 400.0);
        for chunk in [0.7, 1.3, 2.9, 7.0, 31.0] {
            let pieces = run(&mut e, &f, chunk);
            assert_eq!(pieces.len(), whole.len(), "chunk {chunk}");
        }
        for w in &whole {
            assert!(w.end_jd_ut - w.start_jd_ut >= MUHURTA_MIN_WINDOW_DAYS);
            // Hiçbir pencere Kova ya da Balık'ta (panchaka) değildir.
            assert!(!matches!(
                Sign::of(
                    e.body(
                        0.5 * (w.start_jd_ut + w.end_jd_ut),
                        Body::Moon,
                        Zodiac::Sidereal(ephemeris::SiderealMode::Lahiri)
                    )
                    .unwrap()
                    .lon
                ),
                Sign::Aquarius | Sign::Pisces
            ));
        }
        // Sahte Ay 27,27 günde bir döner: 40 günde elverişli altı burcun dördü (Boğa, Yengeç,
        // Terazi, Akrep) en az birer kez geçilir.
        assert!(whole.len() >= 4, "{}", whole.len());
    }

    #[test]
    fn personal_conditions_need_the_natal_moon() {
        let mut e = eph();
        let s = CalcSettings::default();
        let criteria = MuhurtaCriteria {
            require_tara: true,
            require_chandra: true,
            ..MuhurtaCriteria::default()
        };
        let without = filter(30.0, criteria.clone());
        assert_eq!(
            muhurta_search_page(&mut e, &s, &without, None),
            Err(CoreError::InvalidMuhurtaInput("natal"))
        );
        let natal = NatalMoon {
            nakshatra: 4,
            sign: Sign::Taurus,
        };
        let with = MuhurtaFilter {
            natal: Some(natal),
            ..without
        };
        let w = run(&mut e, &with, 31.0);
        assert!(!w.is_empty());
        for win in &w {
            for jd in [
                win.start_jd_ut,
                0.5 * (win.start_jd_ut + win.end_jd_ut),
                win.end_jd_ut - 2.0 * TOL,
            ] {
                let snap = window_snapshot(&mut e, &s, jd, 41.0, 29.0).unwrap();
                assert!(tara_bala(natal.nakshatra, snap.nakshatra).favourable);
                let moon = graha_sign_series(&mut e, &s, Graha::Moon, jd, jd + TOL).unwrap();
                assert!(chandra_bala(natal.sign, moon[0].value).favourable);
            }
        }
    }

    #[test]
    fn excluded_flags_never_hold_inside_a_window() {
        let mut e = eph();
        let s = CalcSettings::default();
        let criteria = MuhurtaCriteria {
            exclude: vec![
                MuhurtaFlag::Rikta,
                MuhurtaFlag::Vishti,
                MuhurtaFlag::Vyatipata,
                MuhurtaFlag::Vaidhriti,
                MuhurtaFlag::Panchaka,
            ],
            ..MuhurtaCriteria::default()
        };
        let w = run(&mut e, &filter(60.0, criteria), 31.0);
        assert!(w.len() > 10);
        let mut covered = 0.0;
        for win in &w {
            covered += win.end_jd_ut - win.start_jd_ut;
            for k in 0..5 {
                let jd = win.start_jd_ut
                    + (win.end_jd_ut - win.start_jd_ut - 2.0 * TOL) * f64::from(k) / 4.0;
                let snap = window_snapshot(&mut e, &s, jd, 41.0, 29.0).unwrap();
                assert!(!is_rikta(snap.tithi), "rikta at {jd}");
                assert_ne!(snap.karana_name, KaranaName::Vishti);
                assert!(snap.yoga != VYATIPATA_YOGA && snap.yoga != VAIDHRITI_YOGA);
                assert!(!is_panchaka(crate::timeline::NakshatraPada {
                    nakshatra: snap.nakshatra,
                    pada: snap.pada
                }));
            }
        }
        for pair in w.windows(2) {
            assert!(pair[0].end_jd_ut < pair[1].start_jd_ut, "disjoint, ordered");
        }
        // Bir şey dışlandı, her şey değil.
        assert!(covered > 20.0 && covered < 55.0, "{covered}");
    }

    #[test]
    fn days_without_a_sunrise_give_no_daytime_windows_and_a_warning() {
        let mut polar = FakeEph {
            polar: true,
            ..eph()
        };
        let s = CalcSettings::default();
        let day_only = filter(
            5.0,
            MuhurtaCriteria {
                daytime_only: true,
                ..MuhurtaCriteria::default()
            },
        );
        let page = muhurta_search_page(&mut polar, &s, &day_only, None).unwrap();
        assert!(page.windows.is_empty());
        assert_eq!(page.warnings, [ChartWarning::SunriseUnavailable]);
        // Dilim dışlaması o günlerde uygulanamaz: pencere kesilmez, uyarı verilir.
        let rahu = filter(
            5.0,
            MuhurtaCriteria {
                exclude: vec![MuhurtaFlag::RahuSegment],
                ..MuhurtaCriteria::default()
            },
        );
        let page = muhurta_search_page(&mut polar, &s, &rahu, None).unwrap();
        assert_eq!(page.windows.len(), 1);
        assert_eq!(page.warnings, [ChartWarning::SunriseUnavailable]);
        // Vara koşulu gün doğumu olmadan da çalışır (06:00 yerel ortalama saat) ve uyarı vermez.
        let monday = filter(
            7.0,
            MuhurtaCriteria {
                weekdays: vec![Vara::Monday],
                ..MuhurtaCriteria::default()
            },
        );
        let page = muhurta_search_page(&mut polar, &s, &monday, None).unwrap();
        assert_eq!(page.windows.len(), 1);
        assert!(page.warnings.is_empty());
        assert!((page.windows[0].end_jd_ut - page.windows[0].start_jd_ut - 1.0).abs() < TOL);
    }

    #[test]
    fn invalid_filters_and_cursors_are_rejected_by_field() {
        let mut e = eph();
        let s = CalcSettings::default();
        let bad =
            |f: MuhurtaFilter, e: &mut FakeEph| muhurta_search_page(e, &s, &f, None).unwrap_err();
        let ok = MuhurtaCriteria::default();
        assert_eq!(
            bad(filter(366.5, ok.clone()), &mut e),
            CoreError::InvalidMuhurtaInput("range")
        );
        assert_eq!(
            bad(filter(-1.0, ok.clone()), &mut e),
            CoreError::InvalidMuhurtaInput("range")
        );
        assert_eq!(
            bad(
                MuhurtaFilter {
                    lat: 95.0,
                    ..filter(1.0, ok.clone())
                },
                &mut e
            ),
            CoreError::InvalidMuhurtaInput("place")
        );
        let with = |criteria| filter(1.0, criteria);
        assert_eq!(
            bad(
                with(MuhurtaCriteria {
                    tithis: vec![0],
                    ..ok.clone()
                }),
                &mut e
            ),
            CoreError::InvalidMuhurtaInput("tithis")
        );
        assert_eq!(
            bad(
                with(MuhurtaCriteria {
                    nakshatras: vec![28],
                    ..ok.clone()
                }),
                &mut e
            ),
            CoreError::InvalidMuhurtaInput("nakshatras")
        );
        assert_eq!(
            bad(
                with(MuhurtaCriteria {
                    min_minutes: MUHURTA_MAX_MIN_MINUTES + 1,
                    ..ok.clone()
                }),
                &mut e
            ),
            CoreError::InvalidMuhurtaInput("min_minutes")
        );
        let f = filter(100.0, ok);
        for cursor in [
            MuhurtaCursor {
                chunk: 4,
                open_start_jd_ut: None,
            },
            // Açık pencere bu dilimden sonra başlayamaz.
            MuhurtaCursor {
                chunk: 1,
                open_start_jd_ut: Some(JD + 40.0),
            },
            MuhurtaCursor {
                chunk: 1,
                open_start_jd_ut: Some(JD - 1.0),
            },
            MuhurtaCursor {
                chunk: 0,
                open_start_jd_ut: Some(JD),
            },
            MuhurtaCursor {
                chunk: 1,
                open_start_jd_ut: Some(f64::NAN),
            },
        ] {
            assert_eq!(
                muhurta_search_page(&mut e, &s, &f, Some(cursor)),
                Err(CoreError::InvalidMuhurtaInput("cursor")),
                "{cursor:?}"
            );
        }
    }

    #[test]
    fn criteria_normalize_to_a_canonical_form() {
        let c = MuhurtaCriteria {
            weekdays: vec![Vara::Friday, Vara::Sunday, Vara::Friday],
            tithis: vec![9, 2, 9],
            nakshatras: vec![27, 1],
            lagna_signs: vec![Sign::Leo, Sign::Aries, Sign::Leo],
            exclude: vec![MuhurtaFlag::Gulika, MuhurtaFlag::Rikta, MuhurtaFlag::Gulika],
            ..MuhurtaCriteria::default()
        }
        .normalized();
        assert_eq!(c.weekdays, [Vara::Sunday, Vara::Friday]);
        assert_eq!(c.tithis, [2, 9]);
        assert_eq!(c.nakshatras, [1, 27]);
        assert_eq!(c.lagna_signs, [Sign::Aries, Sign::Leo]);
        assert_eq!(c.exclude, [MuhurtaFlag::Rikta, MuhurtaFlag::Gulika]);
        assert_eq!(c.clone().normalized(), c);
    }
}
