//! Efemeris tablosu komutu, gerçek `Engine` ile uçtan uca (yol haritası Plan 26, §4.9) ve arayüz
//! testlerinin fikstürü: `app/ui/test/fixtures/ephemeris.json`.
//! Güncellemek için: `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ephemeris_table`.
//! Tek test: süreçte yalnızca bir SE örneği açılabilir. Çapraz denetimler v1'in transit hesabına
//! (konumlar) ve `timeline` dizilerine (giriş ve dönüş işaretleri) dayanır; beklenen değerler
//! başka bir yazılımdan alınmaz (K9).

use std::path::PathBuf;
use std::time::{Duration, Instant};

use atlas::Timestamp;
use ephemeris::{Engine, SwissEph};
use jyotish_core::{
    graha_motion_series, graha_sign_series, CalcSettings, ChartWarning, CoreError, Graha, Motion,
    NodeKind, Sign, ZodiacMode, EPHEMERIS_METHOD, STATION_GRAHAS,
};
use jyotish_desk_lib::commands::{compute_chart_blocking, compute_transits_blocking};
use jyotish_desk_lib::muhurta_commands::MuhurtaPlace;
use jyotish_desk_lib::transit_tools_commands::ephemeris::{
    ephemeris_month_blocking, EphemerisMonthView, MonthId,
};
use serde_json::{json, Value};
use store::TzChoice;

/// Bütçe (D17): bir ayın tablosu, hata ayıklama derlemesinde.
const BUDGET: Duration = Duration::from_millis(100);

fn delhi() -> MuhurtaPlace {
    MuhurtaPlace {
        lat: 28.6139,
        lon: 77.2090,
        tz: TzChoice::Iana("Asia/Kolkata".into()),
    }
}

/// 2026-11-24T00:00:00Z.
fn now() -> Timestamp {
    Timestamp::from_second(1_795_478_400).expect("timestamp")
}

fn month(year: i32, month: u8) -> MonthId {
    MonthId { year, month }
}

fn view(
    engine: &Engine,
    m: Option<MonthId>,
    place: MuhurtaPlace,
    settings: &CalcSettings,
) -> EphemerisMonthView {
    ephemeris_month_blocking(engine, m, place, settings.clone(), now()).expect("ephemeris month")
}

/// Ayın anları: satırların anları ve son satırı kapatan an (sonraki ayın ilk satırı).
fn moments(
    engine: &Engine,
    v: &EphemerisMonthView,
    place: MuhurtaPlace,
    settings: &CalcSettings,
) -> Vec<f64> {
    let mut out: Vec<f64> = v.days.iter().map(|d| d.row.jd_ut).collect();
    let next = view(engine, v.next, place, settings);
    out.push(next.days[0].row.jd_ut);
    out
}

/// `timeline` dizisinin sınırlarının düştüğü satırlar: (satır, yeni değer).
fn rows_of<T: Copy>(series: &[jyotish_core::Interval<T>], moments: &[f64]) -> Vec<(usize, T)> {
    series
        .windows(2)
        .map(|w| {
            let t = w[1].start_jd_ut;
            let row = moments.partition_point(|m| *m <= t) - 1;
            (row, w[1].value)
        })
        .collect()
}

/// Tablonun konumları v1'in transit hesabıyla, işaretleri `timeline` dizileriyle aynıdır.
/// Dönen sayı, tablodaki dönüş işaretlerinin sayısıdır.
fn cross_check(
    engine: &Engine,
    v: &EphemerisMonthView,
    place: MuhurtaPlace,
    settings: &CalcSettings,
) -> usize {
    assert_eq!(v.method, EPHEMERIS_METHOD);
    let bhavas = compute_chart_blocking(engine, 2_451_545.0, 28.6139, 77.2090, settings.clone())
        .expect("chart")
        .bhavas;
    for day in &v.days {
        let snap =
            compute_transits_blocking(engine, day.row.jd_ut, settings.clone(), bhavas.clone())
                .expect("transits");
        assert_eq!(day.row.cells.len(), 9);
        for (c, g) in day.row.cells.iter().zip(&snap.grahas) {
            let ctx = format!("{} {:?}", day.date, c.graha);
            assert_eq!(c.graha, g.graha, "{ctx}");
            assert_eq!(
                (c.lon, c.sign, c.deg_in_sign),
                (g.lon, g.sign, g.deg_in_sign),
                "{ctx}"
            );
            assert_eq!(
                (c.nakshatra, c.pada, c.retrograde),
                (g.nakshatra, g.pada, g.retrograde),
                "{ctx}"
            );
        }
    }
    let moments = moments(engine, v, place, settings);
    let (from, to) = (moments[0], *moments.last().expect("moments"));
    let mut stations = 0;
    for (index, graha) in Graha::ALL.into_iter().enumerate() {
        let s = settings.clone();
        let (signs, motions) = engine
            .call(move |se: &mut SwissEph| {
                let signs = graha_sign_series(se, &s, graha, from, to)?;
                let motions = if STATION_GRAHAS.contains(&graha) {
                    graha_motion_series(se, &s, graha, from, to)?
                } else {
                    Vec::new()
                };
                Ok::<_, CoreError>((signs, motions))
            })
            .expect("engine")
            .expect("series");
        let marked: Vec<(usize, Sign)> = v
            .days
            .iter()
            .enumerate()
            .filter_map(|(i, d)| d.row.cells[index].ingress.map(|sign| (i, sign)))
            .collect();
        assert_eq!(
            marked,
            rows_of(&signs, &moments),
            "{graha:?} ingress, {:?}",
            v.month
        );
        let turned: Vec<(usize, Motion)> = v
            .days
            .iter()
            .enumerate()
            .filter_map(|(i, d)| d.row.cells[index].station.map(|m| (i, m)))
            .collect();
        assert_eq!(
            turned,
            rows_of(&motions, &moments),
            "{graha:?} station, {:?}",
            v.month
        );
        stations += turned.len();
    }
    stations
}

#[test]
fn ephemeris_command_runs_on_the_engine_thread_and_maps_errors() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let engine = Engine::spawn(root.join("../../data/ephe")).expect("spawn engine");
    let settings = CalcSettings::default();

    let started = Instant::now();
    let november = view(&engine, Some(month(2026, 11)), delhi(), &settings);
    let spent = started.elapsed();
    eprintln!("ephemeris_month(2026-11): {spent:?}");
    assert!(spent <= BUDGET, "ephemeris_month took {spent:?}");

    // Kasım 30 gündür; her satır yerin gece yarısındadır (Delhi +05:30: 18:30 UT).
    assert_eq!(november.month, month(2026, 11));
    assert_eq!(
        (november.prev, november.next),
        (Some(month(2026, 10)), Some(month(2026, 12)))
    );
    assert_eq!(november.days.len(), 30);
    assert!(november.warnings.is_empty());
    for (i, day) in november.days.iter().enumerate() {
        assert_eq!(day.date, format!("2026-11-{:02}", i + 1));
        assert_eq!(day.offset_secs, 19_800);
        let local = day.row.jd_ut + 0.5 + f64::from(day.offset_secs) / 86_400.0;
        assert!(
            (local - local.round()).abs() < 1e-9,
            "{}: {local}",
            day.date
        );
    }
    // Güneş ayda bir burç değiştirir; Ay 30 günde 13 ya da 14 burca girer; Güneş ve Ay retro
    // olmaz. Bunlar tablonun kendi içinde tutarlılığıdır; kesin denetim aşağıdadır.
    let ingresses = |g: usize| {
        november
            .days
            .iter()
            .filter(|d| d.row.cells[g].ingress.is_some())
            .count()
    };
    assert_eq!(ingresses(0), 1);
    assert!((13..=14).contains(&ingresses(1)), "{}", ingresses(1));
    assert!(november
        .days
        .iter()
        .all(|d| !d.row.cells[0].retrograde && !d.row.cells[1].retrograde));

    // Konumlar ve işaretler, üç ay ve iki zodyak kipi; gerçek düğümle bir ay daha.
    let tropical = CalcSettings {
        zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
        ..settings.clone()
    };
    let true_node = CalcSettings {
        node: NodeKind::True,
        ..settings.clone()
    };
    let mut stations = 0;
    for m in [month(2026, 11), month(2026, 12), month(2027, 1)] {
        for s in [&settings, &tropical] {
            let v = view(&engine, Some(m), delhi(), s);
            stations += cross_check(&engine, &v, delhi(), s);
        }
    }
    cross_check(
        &engine,
        &view(&engine, Some(month(2026, 11)), delhi(), &true_node),
        delhi(),
        &true_node,
    );
    // Üç ayda en az bir dönüş vardır (iki kipte iki kez sayılır).
    assert!(stations >= 2, "{stations}");

    // Ay verilmezse yerin bugünkü ayıdır. 2026-11-30 19:00 UT Delhi'de 1 Aralık 00:30'dur.
    assert_eq!(
        view(&engine, None, delhi(), &settings).month,
        month(2026, 11)
    );
    let month_end = Timestamp::from_second(1_796_065_200).expect("timestamp");
    let december = ephemeris_month_blocking(&engine, None, delhi(), settings.clone(), month_end)
        .expect("month");
    assert_eq!(december.month, month(2026, 12));
    assert_eq!(
        (december.days.len(), december.next),
        (31, Some(month(2027, 1)))
    );
    // Şubat: 2024 artık yıldır.
    assert_eq!(
        view(&engine, Some(month(2024, 2)), delhi(), &settings)
            .days
            .len(),
        29
    );

    // Yaz saati gece yarısında başlayan yer: São Paulo'da 2017-10-15 00:00 yoktur (saat 00:00 -03
    // iken 01:00 -02'ye atlar). O günün satırı günün ilk geçerli anıdır: 03:00 UT = 01:00 yerel.
    // Beklenen anlar python3 zoneinfo ile türetildi (America/Sao_Paulo, her gün için ilk geçerli
    // yerel an): 14 Ekim 03:00 UT, 15 Ekim 03:00 UT (01:00 -02), 16 Ekim 02:00 UT (00:00 -02).
    // Aralar: 14->15 24 saat, 15->16 23 saat (indeks 13 ve 14).
    let sao_paulo = MuhurtaPlace {
        lat: -23.5475,
        lon: -46.6361,
        tz: TzChoice::Iana("America/Sao_Paulo".into()),
    };
    let october = view(&engine, Some(month(2017, 10)), sao_paulo, &settings);
    assert_eq!(october.days.len(), 31);
    let fract_hours = |jd: f64| (jd + 0.5).fract() * 24.0;
    assert_eq!(october.days[14].date, "2017-10-15");
    assert_eq!(october.days[14].offset_secs, -7200);
    assert!((fract_hours(october.days[13].row.jd_ut) - 3.0).abs() < 1e-6);
    assert!((fract_hours(october.days[14].row.jd_ut) - 3.0).abs() < 1e-6);
    assert!((fract_hours(october.days[15].row.jd_ut) - 2.0).abs() < 1e-6);
    let gaps: Vec<f64> = october
        .days
        .windows(2)
        .map(|w| (w[1].row.jd_ut - w[0].row.jd_ut) * 24.0)
        .collect();
    assert!((gaps[14] - 23.0).abs() < 1e-6, "{}", gaps[14]);
    assert!(gaps
        .iter()
        .enumerate()
        .all(|(i, g)| i == 14 || (g - 24.0).abs() < 1e-6));

    // Yerde hiç var olmayan gün: Pacific/Apia 2011-12-30 atlandı (29 Aralık 23:59 -10'dan 31
    // Aralık 00:00 +14'e). Ay hata vermez, 30 satır döner ve 30 Aralık yoktur. İlk 31 Aralık
    // anı 2011-12-30 10:00 UT (python3 zoneinfo). Satırlar arası 29->31: 24 saat.
    let apia = MuhurtaPlace {
        lat: -13.83,
        lon: -171.76,
        tz: TzChoice::Iana("Pacific/Apia".into()),
    };
    let samoa = view(&engine, Some(month(2011, 12)), apia.clone(), &settings);
    assert_eq!(samoa.days.len(), 30);
    assert!(samoa.days.iter().all(|d| d.date != "2011-12-30"));
    let dates: Vec<&str> = samoa.days.iter().map(|d| d.date.as_str()).collect();
    assert_eq!(dates[28], "2011-12-29");
    assert_eq!(dates[29], "2011-12-31");
    assert_eq!(samoa.days[29].offset_secs, 14 * 3600);
    assert!((fract_hours(samoa.days[29].row.jd_ut) - 10.0).abs() < 1e-6);
    let skip_gap = (samoa.days[29].row.jd_ut - samoa.days[28].row.jd_ut) * 24.0;
    assert!((skip_gap - 24.0).abs() < 1e-6, "{skip_gap}");
    // İşaretler atlanan günün üstünden ardışık satırları karşılaştırır.
    cross_check(&engine, &samoa, apia, &settings);
    // Geçişten önceki ve sonraki günlerin ofseti: −03:00 ve −02:00. Geçiş anının kendisi (15
    // Ekim'in satırı) sınırdadır; ofseti burada sabitlenmez.
    assert_eq!(
        (october.days[13].offset_secs, october.days[15].offset_secs),
        (-10_800, -7_200)
    );
    assert_eq!(october.days[14].date, "2017-10-15");

    // Ephemeris dosyalarının dışındaki yıl: yedek ephemeris uyarısı.
    let far = view(&engine, Some(month(2500, 1)), delhi(), &settings);
    assert_eq!(far.warnings, [ChartWarning::LowPrecision]);

    // Hatalar yalnızca alan adını taşır; yer hataya girmez.
    let bad_place = MuhurtaPlace {
        lon: 200.0,
        ..delhi()
    };
    let e = ephemeris_month_blocking(
        &engine,
        Some(month(2026, 11)),
        bad_place,
        settings.clone(),
        now(),
    )
    .unwrap_err();
    assert_eq!(e.code, "invalid_transit_input");
    assert_eq!(e.params.len(), 1);
    assert_eq!(e.params["field"], "place");
    for bad in [
        month(0, 1),
        month(3000, 1),
        month(2026, 0),
        month(2026, 13),
        month(i32::MIN, 255),
    ] {
        let e = ephemeris_month_blocking(&engine, Some(bad), delhi(), settings.clone(), now())
            .unwrap_err();
        assert_eq!(e.code, "invalid_transit_input", "{bad:?}");
        assert_eq!(e.params.len(), 1);
        assert_eq!(e.params["field"], "month");
    }
    let old = CalcSettings {
        schema_version: 1,
        ..settings.clone()
    };
    let e =
        ephemeris_month_blocking(&engine, Some(month(2026, 11)), delhi(), old, now()).unwrap_err();
    assert_eq!(e.code, "unsupported_schema_version");

    // Fikstür: Delhi, Kasım 2026.
    let fresh = json!({ "place": delhi(), "month": november });
    let path = root.join("../ui/test/fixtures/ephemeris.json");
    if std::env::var_os("UPDATE_FIXTURES").is_some() {
        let text = serde_json::to_string(&fresh).expect("serialize fixture") + "\n";
        std::fs::write(&path, text).expect("write fixture");
        return;
    }
    let committed: Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null);
    assert!(
        approx_eq(&committed, &fresh),
        "app/ui/test/fixtures/ephemeris.json is stale; run: UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ephemeris_table"
    );
}

/// Yapısal eşitlik; sayılar göreli 1e-9 toleransla, diğer değerler birebir (`ui_fixture.rs`
/// ile aynı kural).
fn approx_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(x), Some(y)) => (x - y).abs() <= 1e-9 * x.abs().max(1.0),
            _ => x == y,
        },
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| approx_eq(x, y))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| approx_eq(v, w)))
        }
        _ => a == b,
    }
}
