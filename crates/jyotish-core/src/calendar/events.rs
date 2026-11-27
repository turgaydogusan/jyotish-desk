//! Takvimin olayları (Plan 29 D6–D9): olay aramasının çekirdeğinden gelen olayların takvim
//! günlerine dağıtımı, takvimin olay süzgeci ve olay sayısı sınırı. Yeni astronomi yoktur:
//! olayları `search::events::event_search_chunk` bulur.

use ephemeris::{Body, Ephemeris, Precision};
use serde::{Deserialize, Serialize};

use crate::chart::rasi_frame;
use crate::graha::Graha;
use crate::positions::Positions;
use crate::search::events::{event_search_chunk, EventSpec, SkyEvent, SkyEventKind};
use crate::settings::CalcSettings;
use crate::CoreError;

use super::grid::{calendar_slices, check_place, check_spans, day_facts, DayFacts, DaySpan};

/// Takvimde gösterilebilen olay türleri, bu sırayla. Kavuşum ve doğum Ay'ından ev değişimi
/// takvimde yoktur (ikincisi Jüpiter ve Satürn'ün burç girişidir ve o satırda zaten durur).
pub const CALENDAR_KINDS: [SkyEventKind; 7] = [
    SkyEventKind::SignIngress,
    SkyEventKind::NakshatraIngress,
    SkyEventKind::StationRetrograde,
    SkyEventKind::StationDirect,
    SkyEventKind::NewMoon,
    SkyEventKind::FullMoon,
    SkyEventKind::TransitToNatal,
];
/// Yeni panelin olay türleri: girişler, dönüşler, yeni ay ve dolunay. Doğum boylamına gelişler
/// ve Ay'ın kendi girişleri süzgeçten açılır.
pub const CALENDAR_DEFAULT_KINDS: [SkyEventKind; 6] = [
    SkyEventKind::SignIngress,
    SkyEventKind::NakshatraIngress,
    SkyEventKind::StationRetrograde,
    SkyEventKind::StationDirect,
    SkyEventKind::NewMoon,
    SkyEventKind::FullMoon,
];
/// Bir ayın taşıdığı en çok olay; fazlası kesilir ve sonuç bunu söyler.
pub const CALENDAR_MAX_EVENTS: usize = 300;

/// Takvimin olay süzgeci.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CalendarFilter {
    /// `CALENDAR_KINDS`'in bir alt kümesi; boş olabilir (yalnızca gün doğumu olguları).
    pub kinds: Vec<SkyEventKind>,
    /// Ay'ın girişleri ve doğum boylamlarına gelişleri de aranır. Kapalıyken graha seçimi Ay
    /// dışındaki sekiz grahadır; yeni ay ve dolunay bu seçime bağlı değildir.
    pub moon: bool,
}

fn invalid(field: &'static str) -> CoreError {
    CoreError::InvalidSearchInput(field)
}

impl CalendarFilter {
    /// Türler tekrarsızdır ve `CALENDAR_KINDS` içindedir; hata alan adını taşır (`kinds`).
    pub fn validate(&self) -> Result<(), CoreError> {
        let distinct = self
            .kinds
            .iter()
            .enumerate()
            .all(|(i, k)| !self.kinds[..i].contains(k));
        if distinct && self.kinds.iter().all(|k| CALENDAR_KINDS.contains(k)) {
            Ok(())
        } else {
            Err(invalid("kinds"))
        }
    }

    /// Aranan grahalar, `Graha::ALL` sırasında.
    pub fn grahas(&self) -> Vec<Graha> {
        Graha::ALL
            .into_iter()
            .filter(|g| self.moon || *g != Graha::Moon)
            .collect()
    }

    /// `[from, to)` için olay aramasının girdisi; aranacak tür yoksa `None`. Açık harita
    /// isteyen tür, harita verilmediyse sessizce düşer (süzgeç panelin ayarıdır ve haritadan
    /// bağımsız saklanır).
    pub fn event_spec(&self, from_jd_ut: f64, to_jd_ut: f64, has_natal: bool) -> Option<EventSpec> {
        let kinds: Vec<SkyEventKind> = CALENDAR_KINDS
            .into_iter()
            .filter(|k| self.kinds.contains(k) && (has_natal || !k.needs_natal()))
            .collect();
        if kinds.is_empty() {
            return None;
        }
        Some(EventSpec {
            from_jd_ut,
            to_jd_ut,
            kinds,
            grahas: self.grahas(),
        })
    }
}

/// Takvimin değişmeyen girdisi: yer (yalnızca gün doğumu için), hesap ayarı ve süzgeç.
#[derive(Debug, Clone, PartialEq)]
pub struct CalendarSpec {
    /// Kuzey +.
    pub lat: f64,
    /// Doğu +.
    pub lon: f64,
    pub settings: CalcSettings,
    pub filter: CalendarFilter,
}

impl CalendarSpec {
    pub fn validate(&self) -> Result<(), CoreError> {
        self.settings.validate()?;
        check_place(self.lat, self.lon)?;
        self.filter.validate()
    }
}

/// Bir takvim günü: sınırları, gün doğumu olguları ve o güne düşen olaylar (zaman sırasıyla).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CalendarDay {
    pub span: DaySpan,
    pub facts: DayFacts,
    pub events: Vec<SkyEvent>,
}

/// Bir dilimin (ya da bütün ayın) günleri.
#[derive(Debug, Clone, PartialEq)]
pub struct CalendarSlice {
    pub days: Vec<CalendarDay>,
    /// Olay sayısı sınırı aştı: sınırdan sonraki olaylar listede yoktur.
    pub truncated: bool,
    /// Güneş Moshier hassasiyetinde hesaplandı (ephemeris dosyalarının aralığının dışı).
    pub low_precision: bool,
}

/// Olayları günlere dağıtır: olay, anını içeren günündür (`[start, end)`; tam gece yarısındaki
/// olay başlayan günündür). `events` zaman sırasında olmalıdır; ilk `limit` olay dağıtılır.
/// Dönüş: gün başına olaylar ve "kesildi" bilgisi. Hiçbir güne düşmeyen olay atılır.
pub fn assign_events(
    spans: &[DaySpan],
    events: Vec<SkyEvent>,
    limit: usize,
) -> (Vec<Vec<SkyEvent>>, bool) {
    let truncated = events.len() > limit;
    let mut out = vec![Vec::new(); spans.len()];
    for event in events.into_iter().take(limit) {
        let i = spans.partition_point(|s| s.end_jd_ut <= event.jd_ut);
        if spans.get(i).is_some_and(|s| s.contains(event.jd_ut)) {
            out[i].push(event);
        }
    }
    (out, truncated)
}

/// Bir dilimin günleri: her günün gün doğumu olguları ve o güne düşen olaylar. `spans` ardışık
/// takvim günleridir (çağıran `calendar_slices` ile böler); `limit` bu dilimin taşıyabileceği
/// en çok olaydır (ayın sınırından geriye duran pay). Pay bitmişse olay aranmaz ve sonuç
/// "kesildi" işaretlidir. `natal` açık haritanın konumlarıdır (yalnızca sidereal boylamlar
/// kullanılır; sonuca ve hataya girmez).
pub fn calendar_slice<E: Ephemeris + ?Sized>(
    eph: &mut E,
    spec: &CalendarSpec,
    natal: Option<&Positions>,
    spans: &[DaySpan],
    limit: usize,
) -> Result<CalendarSlice, CoreError> {
    spec.validate()?;
    check_spans(spans)?;
    let (from, to) = (spans[0].start_jd_ut, spans[spans.len() - 1].end_jd_ut);
    // Hassasiyet Güneş'ten okunur (`compute_chart` ile aynı kural).
    let low_precision = eph
        .body(from, Body::Sun, rasi_frame(&spec.settings))?
        .precision
        == Precision::Moshier;
    let facts: Vec<DayFacts> = spans
        .iter()
        .map(|s| day_facts(eph, &spec.settings, spec.lat, spec.lon, *s))
        .collect::<Result<_, _>>()?;
    let (events, truncated) = match spec.filter.event_spec(from, to, natal.is_some()) {
        None => (vec![Vec::new(); spans.len()], false),
        Some(_) if limit == 0 => (vec![Vec::new(); spans.len()], true),
        Some(search) => {
            let found = event_search_chunk(eph, &spec.settings, &search, natal, (from, to))?;
            assign_events(spans, found, limit)
        }
    };
    let days = spans
        .iter()
        .zip(facts)
        .zip(events)
        .map(|((span, facts), events)| CalendarDay {
            span: *span,
            facts,
            events,
        })
        .collect();
    Ok(CalendarSlice {
        days,
        truncated,
        low_precision,
    })
}

/// Bütün ay, dilimleri sırayla yürüterek (testler ve tek seferlik kullanım). Uygulama dilimleri
/// kendisi yürütür ve aralarında motor iş parçacığını bırakır.
pub fn calendar_month<E: Ephemeris + ?Sized>(
    eph: &mut E,
    spec: &CalendarSpec,
    natal: Option<&Positions>,
    spans: &[DaySpan],
) -> Result<CalendarSlice, CoreError> {
    spec.validate()?;
    check_spans(spans)?;
    let mut month = CalendarSlice {
        days: Vec::with_capacity(spans.len()),
        truncated: false,
        low_precision: false,
    };
    let mut left = CALENDAR_MAX_EVENTS;
    for (from, to) in calendar_slices(spans.len()) {
        let slice = calendar_slice(eph, spec, natal, &spans[from..to], left)?;
        left -= slice.days.iter().map(|d| d.events.len()).sum::<usize>();
        month.truncated |= slice.truncated;
        month.low_precision |= slice.low_precision;
        month.days.extend(slice.days);
    }
    Ok(month)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::search::events::event_search;
    use crate::settings::CalcSettings;

    /// 2000-01-01 00:00 UT.
    const MIDNIGHT: f64 = 2_451_544.5;
    const HALF_SECOND: f64 = 0.5 / 86_400.0;

    fn days(n: usize) -> Vec<DaySpan> {
        (0..n)
            .map(|i| DaySpan {
                start_jd_ut: MIDNIGHT + i as f64,
                end_jd_ut: MIDNIGHT + (i + 1) as f64,
            })
            .collect()
    }

    fn spec(kinds: &[SkyEventKind], moon: bool) -> CalendarSpec {
        CalendarSpec {
            lat: 28.6,
            lon: 77.2,
            settings: CalcSettings::default(),
            filter: CalendarFilter {
                kinds: kinds.to_vec(),
                moon,
            },
        }
    }

    fn month(spec: &CalendarSpec, natal: Option<&Positions>, n: usize) -> CalendarSlice {
        calendar_month(&mut FakeEph::default(), spec, natal, &days(n)).unwrap()
    }

    /// (gün sırası, tür, grahalar) üçlüleri, takvim sırasıyla.
    fn placed(slice: &CalendarSlice) -> Vec<(usize, SkyEventKind, Vec<Graha>)> {
        slice
            .days
            .iter()
            .enumerate()
            .flat_map(|(i, d)| d.events.iter().map(move |e| (i, e.kind, e.grahas.clone())))
            .collect()
    }

    /// Gerçek bir olayın (sahte gökyüzünün ilk dolunayı) verilen ana taşınmış kopyası. Olay
    /// alan alan kurulmaz: `SkyEvent` yeni bir alan kazanırsa bu test derlenmeyi sürdürür.
    fn event(jd_ut: f64) -> SkyEvent {
        let search = spec(&[SkyEventKind::FullMoon], false)
            .filter
            .event_spec(MIDNIGHT, MIDNIGHT + 31.0, false)
            .unwrap();
        let found = event_search(
            &mut FakeEph::default(),
            &CalcSettings::default(),
            &search,
            None,
        )
        .unwrap();
        SkyEvent {
            jd_ut,
            ..found[0].clone()
        }
    }

    fn field<T: std::fmt::Debug>(r: Result<T, CoreError>) -> &'static str {
        match r {
            Err(CoreError::InvalidSearchInput(f)) => f,
            other => panic!("expected an invalid search input, got {other:?}"),
        }
    }

    #[test]
    fn the_filter_takes_calendar_kinds_only_and_the_moon_is_a_separate_choice() {
        assert!(spec(&CALENDAR_KINDS, true).validate().is_ok());
        assert!(spec(&[], false).validate().is_ok());
        assert_eq!(
            field(spec(&[SkyEventKind::Conjunction], false).validate()),
            "kinds"
        );
        assert_eq!(
            field(spec(&[SkyEventKind::HouseFromNatalMoon], false).validate()),
            "kinds"
        );
        let twice = [SkyEventKind::NewMoon, SkyEventKind::NewMoon];
        assert_eq!(field(spec(&twice, false).validate()), "kinds");
        // Yer ve ayar da denetlenir.
        let mut far = spec(&[], false);
        far.lat = 95.0;
        assert_eq!(field(far.validate()), "place");
        // Varsayılan türler takvim türlerinin alt kümesidir ve açık harita istemez.
        assert!(CALENDAR_DEFAULT_KINDS
            .iter()
            .all(|k| CALENDAR_KINDS.contains(k) && !k.needs_natal()));

        // Ay kapalıyken sekiz graha; açıkken dokuzu.
        let off = spec(&CALENDAR_KINDS, false).filter;
        assert_eq!(off.grahas().len(), 8);
        assert!(!off.grahas().contains(&Graha::Moon));
        assert_eq!(spec(&[], true).filter.grahas(), Graha::ALL);

        // Arama girdisi: türler takvim sırasında; açık harita yoksa doğum türü düşer.
        let mixed = spec(
            &[
                SkyEventKind::TransitToNatal,
                SkyEventKind::FullMoon,
                SkyEventKind::SignIngress,
            ],
            false,
        )
        .filter;
        let with = mixed.event_spec(1.0, 2.0, true).unwrap();
        assert_eq!(
            with.kinds,
            [
                SkyEventKind::SignIngress,
                SkyEventKind::FullMoon,
                SkyEventKind::TransitToNatal
            ]
        );
        assert_eq!((with.from_jd_ut, with.to_jd_ut), (1.0, 2.0));
        let without = mixed.event_spec(1.0, 2.0, false).unwrap();
        assert_eq!(
            without.kinds,
            [SkyEventKind::SignIngress, SkyEventKind::FullMoon]
        );
        // Aranacak tür yoksa arama da yoktur.
        assert!(spec(&[], true).filter.event_spec(1.0, 2.0, true).is_none());
        let natal_only = spec(&[SkyEventKind::TransitToNatal], false).filter;
        assert!(natal_only.event_spec(1.0, 2.0, false).is_none());
        assert!(natal_only.event_spec(1.0, 2.0, true).is_some());
    }

    #[test]
    fn an_event_belongs_to_the_day_that_contains_its_moment() {
        let spans = days(3);
        let events = vec![
            // Aralıktan önce: atılır.
            event(MIDNIGHT - 0.25),
            // Tam gece yarısı: başlayan günündür.
            event(MIDNIGHT),
            event(MIDNIGHT + 0.5),
            event(MIDNIGHT + 1.0),
            event(MIDNIGHT + 2.999),
            // Son günün bitişi aralığın dışındadır.
            event(MIDNIGHT + 3.0),
        ];
        let (per_day, truncated) = assign_events(&spans, events.clone(), 10);
        assert!(!truncated);
        let counts: Vec<usize> = per_day.iter().map(Vec::len).collect();
        assert_eq!(counts, [2, 1, 1]);
        assert_eq!(per_day[1][0].jd_ut, MIDNIGHT + 1.0);
        // Sınır: ilk `limit` olay dağıtılır (atılanlar da sayılır) ve sonuç bunu söyler.
        let (per_day, truncated) = assign_events(&spans, events.clone(), 3);
        assert!(truncated);
        let counts: Vec<usize> = per_day.iter().map(Vec::len).collect();
        assert_eq!(counts, [2, 0, 0]);
        let (per_day, truncated) = assign_events(&spans, events, 6);
        assert!(!truncated);
        assert_eq!(per_day.iter().map(Vec::len).sum::<usize>(), 4);
        // Olay yoksa her gün boştur.
        let (per_day, truncated) = assign_events(&spans, Vec::new(), 0);
        assert!(!truncated && per_day.iter().all(Vec::is_empty) && per_day.len() == 3);
    }

    /// Sahte ephemeris'in J2000'deki (2. günün öğlesi) sidereal boylamları: Güneş 346°
    /// (0,98°/gün), Ay 71° (13,2), Mars 176° (0,6), Merkür 356° (1,2), Jüpiter 226° (−0,1),
    /// Venüs 16° (1,1), Satürn 276° (0,05). Gün sırası 0'dan başlar: gün `i`, J2000'den
    /// `i − 0,5` … `i + 0,5` gün sonrasıdır.
    #[test]
    fn sign_ingresses_and_lunations_land_on_their_days() {
        let found = month(
            &spec(
                &[
                    SkyEventKind::SignIngress,
                    SkyEventKind::NewMoon,
                    SkyEventKind::FullMoon,
                ],
                false,
            ),
            None,
            31,
        );
        assert_eq!(found.days.len(), 31);
        assert!(!found.truncated && !found.low_precision);
        // Merkür 356° → 0°: 4 / 1,2 = 3,33 gün (gün 3). Mars 176° → 180°: 4 / 0,6 = 6,67 gün
        // (gün 7). Dolunay: uzanım 85° → 180°, 95 / 12,22 = 7,77 gün (gün 8). Venüs 16° → 30°:
        // 14 / 1,1 = 12,73 gün (gün 13). Güneş 346° → 0°: 14 / 0,98 = 14,29 gün (gün 14). Yeni
        // ay: 275 / 12,22 = 22,50 gün (gün 23). Merkür 0° → 30°: 34 / 1,2 = 28,33 gün (gün 28).
        // Jüpiter, Satürn ve düğümler bu ayda burç değiştirmez; Ay kapalıdır.
        assert_eq!(
            placed(&found),
            [
                (3, SkyEventKind::SignIngress, vec![Graha::Mercury]),
                (7, SkyEventKind::SignIngress, vec![Graha::Mars]),
                (8, SkyEventKind::FullMoon, vec![Graha::Sun, Graha::Moon]),
                (13, SkyEventKind::SignIngress, vec![Graha::Venus]),
                (14, SkyEventKind::SignIngress, vec![Graha::Sun]),
                (23, SkyEventKind::NewMoon, vec![Graha::Sun, Graha::Moon]),
                (28, SkyEventKind::SignIngress, vec![Graha::Mercury]),
            ]
        );
        // Her olay kendi gününün içindedir ve günün olayları zaman sırasındadır.
        for d in &found.days {
            assert!(d.events.iter().all(|e| d.span.contains(e.jd_ut)));
            assert!(d.events.windows(2).all(|w| w[0].jd_ut <= w[1].jd_ut));
        }
        // Günlerin olguları olay seçiminden bağımsızdır.
        let bare = month(&spec(&[], false), None, 31);
        assert!(bare.days.iter().all(|d| d.events.is_empty()));
        assert!(!bare.truncated);
        let facts = |s: &CalendarSlice| s.days.iter().map(|d| d.facts).collect::<Vec<_>>();
        assert_eq!(facts(&bare), facts(&found));
    }

    #[test]
    fn the_moon_adds_its_own_ingresses_only_when_asked() {
        let kinds = [SkyEventKind::SignIngress, SkyEventKind::NakshatraIngress];
        let off = month(&spec(&kinds, false), None, 31);
        assert!(placed(&off).iter().all(|(_, _, g)| g[0] != Graha::Moon));
        let on = month(&spec(&kinds, true), None, 31);
        let moon = |kind: SkyEventKind| {
            placed(&on)
                .iter()
                .filter(|(_, k, g)| *k == kind && g[0] == Graha::Moon)
                .count()
        };
        // Ay 31 günde 409,2° yürür: 64,4°'den 473,6°'ye. Geçtiği burç sınırları 90°, 120°, …,
        // 450° (13 giriş); nakshatra sınırları 66°40′'tan 466°40′'a 13°20′ arayla (31 giriş).
        assert_eq!(moon(SkyEventKind::SignIngress), 13);
        assert_eq!(moon(SkyEventKind::NakshatraIngress), 31);
        // Öteki grahaların olayları iki sonuçta aynıdır.
        let others = |s: &CalendarSlice| {
            placed(s)
                .into_iter()
                .filter(|(_, _, g)| g[0] != Graha::Moon)
                .collect::<Vec<_>>()
        };
        assert_eq!(others(&on), others(&off));
    }

    #[test]
    fn the_sliced_month_finds_exactly_the_events_of_one_search() {
        let calendar = spec(&CALENDAR_KINDS, true);
        let natal = Positions::synthetic(
            None,
            [0.0, 100.0, 200.0, 200.0, 200.0, 200.0, 200.0, 200.0, 200.0],
        );
        let spans = days(31);
        let sliced = month(&calendar, Some(&natal), 31);
        let search = calendar
            .filter
            .event_spec(spans[0].start_jd_ut, spans[30].end_jd_ut, true)
            .unwrap();
        let whole = event_search(
            &mut FakeEph::default(),
            &calendar.settings,
            &search,
            Some(&natal),
        )
        .unwrap();
        let flat: Vec<&SkyEvent> = sliced.days.iter().flat_map(|d| &d.events).collect();
        // İkinci yazım: dilim sınırı olay kaybettirmez ve çoğaltmaz. Aynı andaki olayların sırası
        // dilime göre değişebilir; karşılaştırma tür ve graha üzerinden yapılır.
        assert_eq!(flat.len(), whole.len());
        assert!(!sliced.truncated);
        for w in &whole {
            assert!(
                flat.iter().any(|e| e.kind == w.kind
                    && e.grahas == w.grahas
                    && (e.jd_ut - w.jd_ut).abs() < HALF_SECOND),
                "{w:?}"
            );
        }
        // Güneş doğum Güneş'inin boylamına (0°) gün 14'te gelir: kendi boylamına dönüş.
        assert!(placed(&sliced).contains(&(
            14,
            SkyEventKind::TransitToNatal,
            vec![Graha::Sun, Graha::Sun]
        )));
        // Açık harita yoksa doğum türü sessizce düşer; öteki olaylar aynıdır.
        let without = month(&calendar, None, 31);
        let natal_free = |s: &CalendarSlice| {
            placed(s)
                .into_iter()
                .filter(|(_, k, _)| *k != SkyEventKind::TransitToNatal)
                .collect::<Vec<_>>()
        };
        assert!(placed(&without)
            .iter()
            .all(|(_, k, _)| *k != SkyEventKind::TransitToNatal));
        assert_eq!(natal_free(&without), natal_free(&sliced));
    }

    #[test]
    fn the_event_limit_cuts_the_list_and_says_so() {
        let calendar = spec(&[SkyEventKind::SignIngress], false);
        let spans = days(31);
        let cut = |limit: usize| {
            calendar_slice(&mut FakeEph::default(), &calendar, None, &spans, limit).unwrap()
        };
        // Beş girişin ilk ikisi: Merkür (gün 3) ve Mars (gün 7).
        let two = cut(2);
        assert!(two.truncated);
        assert_eq!(
            placed(&two).iter().map(|(i, _, _)| *i).collect::<Vec<_>>(),
            [3, 7]
        );
        // Tam sınırda kesilme yoktur.
        assert!(!cut(5).truncated);
        assert_eq!(placed(&cut(5)).len(), 5);
        // Pay bitmişse olay aranmaz; günlerin olguları yine döner.
        let none = cut(0);
        assert!(none.truncated && placed(&none).is_empty());
        assert_eq!(none.days.len(), 31);
        // Aranacak tür yoksa "kesildi" denmez.
        let bare =
            calendar_slice(&mut FakeEph::default(), &spec(&[], false), None, &spans, 0).unwrap();
        assert!(!bare.truncated);
        // Ayın sınırı gerçek bir ayın çok üstündedir: Ay açıkken bütün türler bile sığar.
        let full = month(&spec(&CALENDAR_KINDS, true), None, 31);
        assert!(!full.truncated);
        assert!(placed(&full).len() < CALENDAR_MAX_EVENTS / 2);
    }

    #[test]
    fn bad_input_is_named_by_field_and_low_precision_is_reported() {
        let calendar = spec(&[SkyEventKind::NewMoon], false);
        let run = |spec: &CalendarSpec, spans: &[DaySpan]| {
            calendar_month(&mut FakeEph::default(), spec, None, spans)
        };
        assert_eq!(field(run(&calendar, &[])), "range");
        assert_eq!(field(run(&calendar, &days(32))), "range");
        let mut far = calendar.clone();
        far.lon = 200.0;
        assert_eq!(field(run(&far, &days(3))), "place");
        let odd = spec(&[SkyEventKind::Conjunction], false);
        assert_eq!(field(run(&odd, &days(3))), "kinds");
        // Dilim de aynı denetimi yapar.
        let slice = calendar_slice(&mut FakeEph::default(), &odd, None, &days(3), 10);
        assert_eq!(field(slice), "kinds");

        let mut moshier = FakeEph {
            precision: Precision::Moshier,
            ..FakeEph::default()
        };
        let low = calendar_month(&mut moshier, &calendar, None, &days(8)).unwrap();
        assert!(low.low_precision);
        assert_eq!(low.days.len(), 8);
    }
}
