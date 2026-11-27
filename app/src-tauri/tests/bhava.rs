//! Bhava tablosu ve Sudarshana chakra komut gövdeleri, gerçek `Engine` ile uçtan uca (yol
//! haritası Plan 25). Tek test: süreçte yalnızca bir SE örneği açılabilir.

use std::path::PathBuf;

use ephemeris::{Engine, Ephemeris, SwissEph, Zodiac};
use jyotish_core::{
    porphyry_points, sandhis_from_mids, signed_arc, CalcSettings, ChartInput, Graha, HouseSystem,
    Sign, SudarshanaRef, ZodiacMode, BHAVA_METHOD, SUDARSHANA_METHOD,
};
use jyotish_desk_lib::bhava_commands::{compute_bhava_table_blocking, compute_sudarshana_blocking};
use jyotish_desk_lib::commands::compute_chart_for_blocking;
use serde_json::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// `tests/golden/*.json` fixture'larının anları (ad, girdi).
fn golden_inputs() -> Vec<(String, ChartInput)> {
    let dir = root().join("../../tests/golden");
    let mut out: Vec<(String, ChartInput)> = std::fs::read_dir(&dir)
        .expect("golden dir")
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .map(|p| {
            let v: Value =
                serde_json::from_str(&std::fs::read_to_string(&p).expect("read fixture"))
                    .expect("parse fixture");
            let num = |k: &str| v["input"][k].as_f64().expect("number");
            let input = ChartInput {
                jd_ut: num("jd_ut"),
                lat: num("lat"),
                lon: num("lon"),
                settings: serde_json::from_value(v["settings"].clone()).expect("settings"),
                time_known: true,
                lagna_number: None,
            };
            (v["id"].as_str().expect("id").to_owned(), input)
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn with_system(input: &ChartInput, system: HouseSystem, zodiac: ZodiacMode) -> ChartInput {
    ChartInput {
        settings: CalcSettings {
            house_system: system,
            zodiac,
            ..input.settings.clone()
        },
        ..input.clone()
    }
}

#[test]
fn bhava_commands_run_on_the_engine_thread_and_agree_with_the_chart() {
    let engine = Engine::spawn(root().join("../../data/ephe")).expect("spawn engine");
    let inputs = golden_inputs();
    assert_eq!(inputs.len(), 5);
    let systems = [
        HouseSystem::WholeSign,
        HouseSystem::Sripati,
        HouseSystem::Placidus,
    ];
    let zodiacs = [
        ZodiacMode::Sidereal,
        ZodiacMode::TropicalRasiSiderealNakshatra,
    ];
    let mut fallbacks = 0;
    let mut shifted_charts = 0;
    for (id, base) in &inputs {
        for system in systems {
            for zodiac in zodiacs {
                let input = with_system(base, system, zodiac);
                let chart = compute_chart_for_blocking(&engine, input.clone()).expect("chart");
                let table = compute_bhava_table_blocking(&engine, input.clone())
                    .expect("bhava table")
                    .expect("time known");
                let ctx = format!("{id} {system:?} {zodiac:?}");
                assert_eq!(table.method, BHAVA_METHOD, "{ctx}");
                assert_eq!(table.system, chart.bhavas.system, "{ctx}");
                assert_eq!((table.rows.len(), table.grahas.len()), (12, 9), "{ctx}");
                // Başlangıçlar haritanın bhava başlangıçlarıdır; bitiş sonrakinin başlangıcıdır;
                // yaylar çemberi tam bir kez dolanır.
                for (i, row) in table.rows.iter().enumerate() {
                    assert_eq!(usize::from(row.house), i + 1, "{ctx}");
                    assert_eq!(row.start.lon, chart.bhavas.cusps[i], "{ctx}");
                    assert_eq!(row.end, table.rows[(i + 1) % 12].start, "{ctx}");
                    assert!(row.span_deg > 0.0 && row.span_deg < 180.0, "{ctx}");
                    assert_eq!(row.mid.is_some(), system != HouseSystem::Placidus, "{ctx}");
                }
                let total: f64 = table.rows.iter().map(|r| r.span_deg).sum();
                assert!((total - 360.0).abs() < 1e-6, "{ctx}: {total}");
                // Grahanın bhavası graha tablosundaki evdir; burç evi lagna burcundan sayılır.
                for (g, c) in table.grahas.iter().zip(&chart.grahas) {
                    assert_eq!((g.graha, g.bhava), (c.graha, c.house), "{ctx}");
                    assert_eq!(g.sign_house, c.sign.house_from(chart.lagna.sign), "{ctx}");
                    assert_eq!(g.shifted, g.bhava != g.sign_house, "{ctx}");
                    let row = &table.rows[usize::from(g.bhava) - 1];
                    assert!(
                        g.from_start_deg >= 0.0 && g.from_start_deg < row.span_deg,
                        "{ctx} {:?}",
                        g.graha
                    );
                    assert_eq!(g.from_mid_deg.is_some(), row.mid.is_some(), "{ctx}");
                    if let (Some(d), Some(mid)) = (g.from_mid_deg, row.mid) {
                        assert!((d - signed_arc(mid.lon, c.lon)).abs() < 1e-9, "{ctx}");
                    }
                }
                match system {
                    HouseSystem::WholeSign => {
                        // Tam burç: hiçbir graha yer değiştirmez; orta lagnanın derecesidir.
                        assert!(table.grahas.iter().all(|g| !g.shifted), "{ctx}");
                        for row in &table.rows {
                            assert_eq!(row.start.deg_in_sign, 0.0, "{ctx}");
                            let mid = row.mid.expect("mid");
                            assert!(
                                (mid.deg_in_sign - chart.lagna.deg_in_sign).abs() < 1e-9,
                                "{ctx}"
                            );
                        }
                    }
                    HouseSystem::Sripati => {
                        // İkinci yazım: ephemeris'in Sripati sandhileri, lagna ile MC'den kurulan
                        // Porphyry ortalarının ikişer ikişer ortasıdır.
                        let (jd, lat, lon) = (input.jd_ut, input.lat, input.lon);
                        let frame = match zodiac {
                            ZodiacMode::Sidereal => {
                                Zodiac::Sidereal(input.settings.ayanamsa.sidereal_mode())
                            }
                            ZodiacMode::TropicalRasiSiderealNakshatra => Zodiac::Tropical,
                        };
                        let houses = engine
                            .call(move |se: &mut SwissEph| {
                                se.houses(jd, lat, lon, ephemeris::HouseSystem::Sripati, frame)
                            })
                            .expect("engine")
                            .expect("houses");
                        let mids = porphyry_points(houses.asc, houses.mc);
                        let sandhis = sandhis_from_mids(&mids);
                        for i in 0..12 {
                            assert!(
                                signed_arc(sandhis[i], chart.bhavas.cusps[i]).abs() < 1e-6,
                                "{ctx}: sandhi {} {} vs {}",
                                i + 1,
                                sandhis[i],
                                chart.bhavas.cusps[i]
                            );
                            let mid = table.rows[i].mid.expect("mid");
                            assert!(signed_arc(mids[i], mid.lon).abs() < 1e-9, "{ctx}");
                            // Orta, bhavanın içindedir.
                            let from_start = (mid.lon - table.rows[i].start.lon).rem_euclid(360.0);
                            assert!(from_start < table.rows[i].span_deg, "{ctx}");
                        }
                        // 1. bhavanın ortası lagnadır.
                        let first = table.rows[0].mid.expect("mid");
                        assert!(signed_arc(first.lon, chart.lagna.lon).abs() < 1e-9, "{ctx}");
                        if table.grahas.iter().any(|g| g.shifted) {
                            shifted_charts += 1;
                        }
                    }
                    HouseSystem::Placidus => {
                        assert!(
                            table.grahas.iter().all(|g| g.from_mid_deg.is_none()),
                            "{ctx}"
                        );
                        if table.porphyry_fallback {
                            fallbacks += 1;
                            assert!(id.starts_with("tromso"), "{ctx}");
                        }
                    }
                }
                // Nakshatra ve pada sidereal boylamdandır: lagnanın noktası haritanın lagnasıyla
                // aynı nakshatrayı verir (tam burç ve Sripati'de 1. bhavanın ortası lagnadır).
                if let Some(mid) = table.rows[0].mid {
                    assert_eq!(
                        (mid.nakshatra, mid.pada),
                        (chart.lagna.nakshatra, chart.lagna.pada),
                        "{ctx}"
                    );
                }

                // Sudarshana: üç halka, haritanın lagna, Ay ve Güneş burcundan.
                let chakra = compute_sudarshana_blocking(&engine, input.clone()).expect("chakra");
                assert_eq!(chakra.method, SUDARSHANA_METHOD, "{ctx}");
                assert!(!chakra.approximate, "{ctx}");
                let firsts: Vec<(SudarshanaRef, Sign)> = chakra
                    .rings
                    .iter()
                    .map(|r| (r.reference, r.houses.as_ref().expect("houses")[0].sign))
                    .collect();
                assert_eq!(
                    firsts,
                    [
                        (SudarshanaRef::Lagna, chart.lagna.sign),
                        (SudarshanaRef::Moon, chart.grahas[Graha::Moon as usize].sign),
                        (SudarshanaRef::Sun, chart.grahas[Graha::Sun as usize].sign),
                    ],
                    "{ctx}"
                );
                for ring in &chakra.rings {
                    let houses = ring.houses.as_ref().expect("houses");
                    for c in &chart.grahas {
                        let at = houses
                            .iter()
                            .find(|h| h.grahas.contains(&c.graha))
                            .expect("every graha is in a house");
                        assert_eq!(at.sign, c.sign, "{ctx}");
                        assert_eq!(at.house, c.sign.house_from(houses[0].sign), "{ctx}");
                    }
                }
            }
        }
    }
    // Tromsø (kutup dairesi) Placidus'ta Porphyry'ye düşer: iki zodyak kipinde birer kez.
    assert_eq!(fallbacks, 2);
    // Sripati'de en az bir haritada bir graha burç evinden başka bir bhavadadır.
    assert!(shifted_charts > 0);

    // Doğum saati bilinmiyor: tablo yok; chakra lagna halkası olmadan ve yaklaşık döner.
    let mut unknown = inputs[0].1.clone();
    unknown.time_known = false;
    assert_eq!(
        compute_bhava_table_blocking(&engine, unknown.clone()).expect("no table"),
        None
    );
    let chakra = compute_sudarshana_blocking(&engine, unknown.clone()).expect("chakra");
    assert!(chakra.approximate);
    assert_eq!(chakra.rings[0].houses, None);
    assert!(chakra.rings[1].houses.is_some() && chakra.rings[2].houses.is_some());

    // Sayı prashnası: ev sistemi ayarı ne olursa olsun tam burç.
    let mut numbered = with_system(&inputs[0].1, HouseSystem::Placidus, ZodiacMode::Sidereal);
    numbered.lagna_number = Some(54);
    let table = compute_bhava_table_blocking(&engine, numbered)
        .expect("bhava table")
        .expect("time known");
    assert_eq!(table.system, HouseSystem::WholeSign);
    assert!(!table.porphyry_fallback);

    // Hata eşlemesi: geçersiz konum, saat bilinmese de bildirilir.
    unknown.lat = 91.0;
    let e = compute_bhava_table_blocking(&engine, unknown.clone()).expect_err("invalid location");
    assert_eq!(e.code, "invalid_location");
    let e = compute_sudarshana_blocking(&engine, unknown).expect_err("invalid location");
    assert_eq!(e.code, "invalid_location");
}
