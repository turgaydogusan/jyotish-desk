//! Geçiş takvimi komutunun gövdesi, gerçek `Engine` ile uçtan uca (yol haritası Plan 29; ileri
//! modüller spec'i §13 "Komut katmanı"). Tek test: süreçte yalnızca bir SE örneği açılabilir.
//! Çapraz denetimler var olan iki komuta dayanır: günün olguları gün panchangıyla
//! (`day_panchang`), olaylar olay aramasıyla (`event_search`) karşılaştırılır.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use atlas::Timestamp;
use ephemeris::Engine;
use jyotish_core::calendar::{
    CalendarFilter, CALENDAR_DEFAULT_KINDS, CALENDAR_KINDS, CALENDAR_MAX_EVENTS, CALENDAR_METHOD,
};
use jyotish_core::{
    CalcSettings, ChartInput, ChartWarning, Graha, Paksha, Sign, SkyEventKind, Vara,
};
use jyotish_desk_lib::calendar_commands::{
    month_calendar_blocking, CalendarRequest, CalendarView, YearMonth,
};
use jyotish_desk_lib::commands::CommandError;
use jyotish_desk_lib::muhurta_commands::{day_panchang_blocking, MuhurtaPlace};
use jyotish_desk_lib::search_commands::{event_search_blocking, EventSearchSpec, SearchRegistry};
use jyotish_desk_lib::transit_commands::TransitAt;
use store::TzChoice;

const SECOND: f64 = 1.0 / 86_400.0;
/// Bir ayın bütçesi (ortanca; hata ayıklama derlemesi): bütün türler, Ay ve açık harita.
const MONTH_BUDGET: Duration = Duration::from_millis(150);

fn place(lat: f64, lon: f64, zone: &str) -> MuhurtaPlace {
    MuhurtaPlace {
        lat,
        lon,
        tz: TzChoice::Iana(zone.into()),
    }
}

fn delhi() -> MuhurtaPlace {
    place(28.6139, 77.209, "Asia/Kolkata")
}

fn tromso() -> MuhurtaPlace {
    place(69.6492, 18.9553, "Europe/Oslo")
}

fn ym(year: i32, month: u8) -> Option<YearMonth> {
    Some(YearMonth { year, month })
}

fn filter(kinds: &[SkyEventKind], moon: bool) -> CalendarFilter {
    CalendarFilter {
        kinds: kinds.to_vec(),
        moon,
    }
}

/// Deneme haritası (Delhi, J2000 öğlesi UT); gerçek kişi değildir.
fn natal() -> ChartInput {
    ChartInput {
        jd_ut: 2_451_545.0,
        lat: 28.6139,
        lon: 77.209,
        settings: CalcSettings::default(),
        time_known: true,
        lagna_number: None,
    }
}

fn request(month: Option<YearMonth>, place: MuhurtaPlace) -> CalendarRequest {
    CalendarRequest {
        month,
        place,
        filter: filter(&CALENDAR_DEFAULT_KINDS, false),
        settings: CalcSettings::default(),
        natal: None,
    }
}

fn field(r: Result<CalendarView, CommandError>) -> String {
    let e = r.expect_err("an input error");
    assert_eq!(e.code, "invalid_calendar_input");
    assert_eq!(e.params.len(), 1, "only the field name");
    e.params["field"].clone()
}

#[test]
fn the_month_calendar_agrees_with_the_day_panchang_and_the_event_search() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let engine = Engine::spawn(root.join("../../data/ephe")).expect("spawn engine");
    // 2026-11-25 10:00 UT: Delhi'de 25 Kasım 2026 Çarşamba 15:30.
    let now = Timestamp::from_second(1_795_600_800).expect("timestamp");
    let run = |r: CalendarRequest| month_calendar_blocking(&engine, r, now);

    // 1. Ay verilmezse yerin bugünkü ayıdır; günler takvim günleridir.
    let november = run(request(None, delhi())).expect("this month");
    assert_eq!(
        november.month,
        YearMonth {
            year: 2026,
            month: 11
        }
    );
    assert_eq!(november.prev, ym(2026, 10));
    assert_eq!(november.next, ym(2026, 12));
    assert_eq!(november.today, Some(25));
    assert_eq!(november.method, CALENDAR_METHOD);
    assert!(!november.truncated && !november.natal && november.warnings.is_empty());
    assert_eq!(november.days.len(), 30);
    assert_eq!(november.days[0].date, "2026-11-01");
    assert_eq!(november.days[29].date, "2026-11-30");
    // 1 Kasım 2026 Pazar'dır; hafta günleri sırayla yürür.
    for (i, d) in november.days.iter().enumerate() {
        assert_eq!(usize::from(d.day), i + 1);
        assert_eq!(d.weekday, Vara::ALL[i % 7], "{}", d.date);
        // Delhi'de yaz saati yoktur: her gün 24 saattir ve ofset +05:30'dur.
        assert!((d.span.end_jd_ut - d.span.start_jd_ut - 1.0).abs() < SECOND);
        assert_eq!(d.offset_secs, 19_800);
        // Olgular günün içindeki gün doğumunda okunur.
        assert_eq!(d.facts.sunrise_jd_ut, Some(d.facts.jd_ut));
        assert!(d.span.start_jd_ut <= d.facts.jd_ut && d.facts.jd_ut < d.span.end_jd_ut);
        assert!(d.facts.jd_ut < d.panchang_jd_ut && d.panchang_jd_ut < d.span.end_jd_ut);
        assert!(d.events.iter().all(|e| {
            let jd = e.event.jd_ut;
            d.span.start_jd_ut <= jd && jd < d.span.end_jd_ut && e.offset_secs == 19_800
        }));
    }
    assert!(november
        .days
        .windows(2)
        .all(|w| w[0].span.end_jd_ut == w[1].span.start_jd_ut));
    // İstenen ay da aynı sonucu verir; başka bir ayda "bugün" yoktur.
    assert_eq!(
        run(request(ym(2026, 11), delhi())).expect("asked month"),
        november
    );
    let december = run(request(ym(2026, 12), delhi())).expect("next month");
    assert_eq!((december.days.len(), december.today), (31, None));
    assert_eq!(december.days[0].weekday, Vara::Tuesday);

    // 2. Çapraz denetim: hücrenin tithisi ve nakshatrası, o gün doğumuyla başlayan panchang
    // gününün ilk değeridir (`day_panchang`, aynı yer ve ayar).
    for d in &november.days {
        let view = day_panchang_blocking(
            &engine,
            TransitAt::Jd(d.panchang_jd_ut),
            now,
            delhi(),
            CalcSettings::default(),
            None,
        )
        .expect("day panchang");
        let sunrise = view.day.sunrise_jd_ut.expect("a sunrise in Delhi");
        assert!((sunrise - d.facts.jd_ut).abs() < SECOND, "{}", d.date);
        assert_eq!(view.day.tithis[0].value.no, d.facts.tithi, "{}", d.date);
        assert_eq!(
            view.day.tithis[0].value.paksha, d.facts.paksha,
            "{}",
            d.date
        );
        assert_eq!(
            view.day.nakshatras[0].value.no, d.facts.nakshatra,
            "{}",
            d.date
        );
        assert_eq!(view.day.vara, d.weekday, "{}", d.date);
    }
    // Bilinen gök olayları (gökbilim almanağı): 2026 Kasım'ında yeni ay 9 Kasım'da, dolunay
    // 24 Kasım'dadır (Delhi tarihiyle de). Dolunay günü gün doğumunda Ay büyümektedir; ertesi
    // günün tithisi Krishna'dır.
    let day_of = |view: &CalendarView, kind: SkyEventKind| -> Vec<String> {
        view.days
            .iter()
            .filter(|d| d.events.iter().any(|e| e.event.kind == kind))
            .map(|d| d.date.clone())
            .collect()
    };
    assert_eq!(day_of(&november, SkyEventKind::NewMoon), ["2026-11-09"]);
    assert_eq!(day_of(&november, SkyEventKind::FullMoon), ["2026-11-24"]);
    assert_eq!(november.days[23].facts.paksha, Paksha::Shukla);
    assert_eq!(november.days[24].facts.paksha, Paksha::Krishna);
    // Güneş ayda bir kez burç değiştirir: Kasım'da sidereal Akrep'e girer.
    let sun_ingress: Vec<_> = november
        .days
        .iter()
        .flat_map(|d| &d.events)
        .filter(|e| e.event.kind == SkyEventKind::SignIngress && e.event.grahas == [Graha::Sun])
        .collect();
    assert_eq!(sun_ingress.len(), 1);
    assert_eq!(sun_ingress[0].event.sign, Some(Sign::Scorpio));
    // Varsayılan süzgeçte Ay'ın kendi girişleri yoktur.
    assert!(november.days.iter().flat_map(|d| &d.events).all(|e| {
        e.event.kind == SkyEventKind::NewMoon
            || e.event.kind == SkyEventKind::FullMoon
            || e.event.grahas[0] != Graha::Moon
    }));

    // 3. Çapraz denetim: bütün türler, Ay ve açık haritayla takvimin olayları, aynı ay için
    // olay aramasının bulduklarıdır (dilimleme olay kaybettirmez ve çoğaltmaz).
    let full_request = CalendarRequest {
        filter: filter(&CALENDAR_KINDS, true),
        natal: Some(natal()),
        ..request(ym(2026, 11), delhi())
    };
    let full = run(full_request.clone()).expect("full month");
    assert!(full.natal && !full.truncated);
    let searched = event_search_blocking(
        &engine,
        &SearchRegistry::default(),
        EventSearchSpec {
            from_date: Some("2026-11-01".into()),
            to_date: Some("2026-11-30".into()),
            place: delhi(),
            kinds: CALENDAR_KINDS.to_vec(),
            grahas: Graha::ALL.to_vec(),
            settings: CalcSettings::default(),
            natal: Some(natal()),
        },
        1,
        now,
    )
    .expect("event search");
    let in_calendar: Vec<_> = full.days.iter().flat_map(|d| &d.events).collect();
    assert_eq!(in_calendar.len(), searched.events.len());
    assert!(in_calendar.len() < CALENDAR_MAX_EVENTS / 2);
    for want in &searched.events {
        assert!(
            in_calendar.iter().any(|e| e.event.kind == want.event.kind
                && e.event.grahas == want.event.grahas
                && (e.event.jd_ut - want.event.jd_ut).abs() < SECOND),
            "{:?}",
            want.event
        );
    }
    // Günlerin olguları süzgeçten bağımsızdır; açık harita verilse de tür seçili değilse
    // doğum olayları aranmaz.
    let facts = |v: &CalendarView| v.days.iter().map(|d| d.facts).collect::<Vec<_>>();
    assert_eq!(facts(&full), facts(&november));
    let unasked = run(CalendarRequest {
        natal: Some(natal()),
        ..request(ym(2026, 11), delhi())
    })
    .expect("natal input without the natal kind");
    assert!(!unasked.natal);
    assert_eq!(unasked.days, november.days);
    // Tür seçili ama harita yok: tür sessizce düşer.
    let no_chart = run(CalendarRequest {
        natal: None,
        ..full_request.clone()
    })
    .expect("natal kind without a chart");
    assert!(!no_chart.natal);
    assert!(no_chart
        .days
        .iter()
        .flat_map(|d| &d.events)
        .all(|e| e.event.kind != SkyEventKind::TransitToNatal));
    // Olay türü seçilmemiş takvim yalnızca olguları taşır.
    let bare = run(CalendarRequest {
        filter: filter(&[], false),
        ..request(ym(2026, 11), delhi())
    })
    .expect("no kinds");
    assert!(bare.days.iter().all(|d| d.events.is_empty()));
    assert_eq!(facts(&bare), facts(&november));

    // 4. Yaz saati: Berlin'de 29 Mart 2026 23 saat, 25 Ekim 2026 25 saattir; ofset gün içinde
    // değişir ve günler yine boşluksuz ardışıktır.
    let berlin = || place(52.52, 13.405, "Europe/Berlin");
    let march = run(request(ym(2026, 3), berlin())).expect("March in Berlin");
    let hours = |v: &CalendarView, day: usize| {
        (v.days[day - 1].span.end_jd_ut - v.days[day - 1].span.start_jd_ut) * 24.0
    };
    assert!((hours(&march, 29) - 23.0).abs() < 1e-6);
    assert!((hours(&march, 28) - 24.0).abs() < 1e-6);
    assert_eq!(
        (march.days[27].offset_secs, march.days[29].offset_secs),
        (3600, 7200)
    );
    let october = run(request(ym(2026, 10), berlin())).expect("October in Berlin");
    assert!((hours(&october, 25) - 25.0).abs() < 1e-6);
    assert_eq!((march.days.len(), october.days.len()), (31, 31));
    assert!(march
        .days
        .windows(2)
        .all(|w| w[0].span.end_jd_ut == w[1].span.start_jd_ut));

    // 5. Kutup dairesi: Tromsø'da Aralık'ta Güneş doğmaz; olgular 06:00 yerel ortalama saatte
    // okunur ve gün panchangıyla aynı günü gösterir. Ocak'ta Güneş ay içinde geri döner.
    let polar = run(request(ym(2026, 12), tromso())).expect("polar night");
    assert_eq!(polar.days.len(), 31);
    for d in &polar.days {
        assert_eq!(d.facts.sunrise_jd_ut, None, "{}", d.date);
        assert!(d.span.start_jd_ut <= d.facts.jd_ut && d.facts.jd_ut < d.span.end_jd_ut);
        // 18,96° D: yerel ortalama saat UT + 1 sa 16 dk; 06:00 yerel 04:44 UT, 05:44 Oslo saati.
        let local_hours = (d.facts.jd_ut - d.span.start_jd_ut) * 24.0;
        assert!(
            (local_hours - 5.736).abs() < 0.01,
            "{}: {local_hours}",
            d.date
        );
    }
    let polar_day = day_panchang_blocking(
        &engine,
        TransitAt::Jd(polar.days[14].panchang_jd_ut),
        now,
        tromso(),
        CalcSettings::default(),
        None,
    )
    .expect("polar day panchang");
    assert!(polar_day
        .day
        .warnings
        .contains(&ChartWarning::SunriseUnavailable));
    assert!((polar_day.day.day_start_jd_ut - polar.days[14].facts.jd_ut).abs() < SECOND);
    assert_eq!(polar_day.day.tithis[0].value.no, polar.days[14].facts.tithi);
    let january = run(request(ym(2027, 1), tromso())).expect("the Sun returns");
    let risen = january
        .days
        .iter()
        .filter(|d| d.facts.sunrise_jd_ut.is_some())
        .count();
    assert!(0 < risen && risen < 31, "{risen}");
    assert_eq!(january.days[0].facts.sunrise_jd_ut, None);
    assert!(january.days[30].facts.sunrise_jd_ut.is_some());
    // Gece yarısı güneşi: Haziran'da da doğuş yoktur.
    let june = run(request(ym(2026, 6), tromso())).expect("midnight sun");
    assert!(june.days.iter().all(|d| d.facts.sunrise_jd_ut.is_none()));

    // 6. Takvimden gün atlayan saat dilimi: Samoa 30 Aralık 2011'i atladı. O gün listede yoktur
    // ve günler yine ardışıktır.
    let apia = run(request(
        ym(2011, 12),
        place(-13.8333, -171.7667, "Pacific/Apia"),
    ))
    .expect("the month with a skipped day");
    assert_eq!(apia.days.len(), 30);
    assert!(apia.days.iter().all(|d| d.date != "2011-12-30"));
    assert_eq!(
        (apia.days[28].date.as_str(), apia.days[29].date.as_str()),
        ("2011-12-29", "2011-12-31")
    );
    assert!(apia
        .days
        .windows(2)
        .all(|w| w[0].span.end_jd_ut == w[1].span.start_jd_ut));

    // 7. Yıl aralığının uçları ve ephemeris dosyalarının dışı: sonuç döner ve düşük hassasiyet
    // uyarısını taşır.
    let first = run(request(ym(1, 1), delhi())).expect("the first month");
    assert_eq!((first.prev, first.next), (None, ym(1, 2)));
    assert_eq!(first.warnings, [ChartWarning::LowPrecision]);
    let last = run(request(ym(2999, 12), delhi())).expect("the last month");
    assert_eq!((last.prev, last.next), (ym(2999, 11), None));
    assert_eq!(last.days.len(), 31);

    // 8. Girdi hataları yalnızca alan adını taşır; yer aydan önce denetlenir.
    assert_eq!(field(run(request(ym(2026, 13), delhi()))), "month");
    assert_eq!(field(run(request(ym(2026, 0), delhi()))), "month");
    assert_eq!(field(run(request(ym(0, 5), delhi()))), "month");
    assert_eq!(field(run(request(ym(3000, 1), delhi()))), "month");
    let nowhere = place(91.0, 0.0, "Asia/Kolkata");
    assert_eq!(field(run(request(ym(2026, 13), nowhere))), "place");
    let no_zone = place(28.6, 77.2, "Nowhere/Zone");
    assert_eq!(field(run(request(None, no_zone))), "place");
    for kind in [SkyEventKind::Conjunction, SkyEventKind::HouseFromNatalMoon] {
        let bad = CalendarRequest {
            filter: filter(&[kind], false),
            ..request(None, delhi())
        };
        assert_eq!(field(run(bad)), "kinds");
    }
    let stale_settings = CalendarRequest {
        settings: CalcSettings {
            schema_version: 99,
            ..CalcSettings::default()
        },
        ..request(None, delhi())
    };
    assert_eq!(
        run(stale_settings).expect_err("settings").code,
        "unsupported_schema_version"
    );

    // 9. Bütçe: en ağır ay (bütün türler, Ay, açık harita), ortanca.
    let mut runs: Vec<Duration> = (0..5)
        .map(|_| {
            let start = Instant::now();
            run(full_request.clone()).expect("timed month");
            start.elapsed()
        })
        .collect();
    runs.sort();
    eprintln!("month_calendar median: {:?}", runs[2]);
    assert!(runs[2] <= MONTH_BUDGET, "{:?}", runs[2]);
}
