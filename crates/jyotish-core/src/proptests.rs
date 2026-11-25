//! Spec §8 property testleri. Grafiğe dayalı özellikler Task 6'da eklenir.

use proptest::prelude::*;

use crate::zodiac::{
    deg_in_sign, nakshatra_pada, normalize, Sign, NAKSHATRA_SPAN_DEG, PADA_SPAN_DEG,
};

proptest! {
    #[test]
    fn normalize_is_in_range_and_congruent(x in -1.0e6f64..1.0e6) {
        let n = normalize(x);
        prop_assert!((0.0..360.0).contains(&n), "normalize({x}) = {n}");
        let turns = (n - x) / 360.0;
        prop_assert!((turns - turns.round()).abs() < 1e-9);
    }

    #[test]
    fn nakshatra_and_pada_bracket_the_longitude(lon in 0.0f64..360.0) {
        let (n, p) = nakshatra_pada(lon);
        prop_assert!((1..=27).contains(&n));
        prop_assert!((1..=4).contains(&p));
        let start = f64::from(n - 1) * NAKSHATRA_SPAN_DEG + f64::from(p - 1) * PADA_SPAN_DEG;
        prop_assert!(
            lon >= start - 1e-9 && lon < start + PADA_SPAN_DEG + 1e-9,
            "lon {lon} -> nakshatra {n} pada {p}"
        );
    }

    #[test]
    fn sign_and_degree_reconstruct_longitude(lon in -720.0f64..720.0) {
        let d = deg_in_sign(lon);
        prop_assert!((0.0..30.0).contains(&d));
        let rebuilt = Sign::of(lon).index() as f64 * 30.0 + d;
        prop_assert!((rebuilt - normalize(lon)).abs() < 1e-9);
    }
}

mod chart_properties {
    use ephemeris::Body;
    use proptest::prelude::*;

    use crate::chart::compute_chart;
    use crate::fake::FakeEph;
    use crate::houses::house_from_cusps;
    use crate::settings::{CalcSettings, ZodiacMode};
    use crate::zodiac::{angular_distance, normalize};

    const PLANETS: [Body; 7] = [
        Body::Sun,
        Body::Moon,
        Body::Mars,
        Body::Mercury,
        Body::Jupiter,
        Body::Venus,
        Body::Saturn,
    ];

    proptest! {
        #[test]
        fn rahu_and_ketu_are_exactly_opposite(
            node in 0.0f64..360.0,
            ayan in 20.0f64..26.0,
            tropical in any::<bool>(),
        ) {
            let mut eph = FakeEph { ayanamsa: ayan, ..FakeEph::default() }
                .with(Body::MeanNode, node, -0.053);
            let mut s = CalcSettings::default();
            if tropical {
                s.zodiac = ZodiacMode::TropicalRasiSiderealNakshatra;
            }
            let c = compute_chart(&mut eph, 2_451_545.0, 0.0, 0.0, &s).unwrap();
            let (rahu, ketu) = (&c.grahas[7], &c.grahas[8]);
            prop_assert_eq!(ketu.lon, normalize(rahu.lon + 180.0));
            prop_assert!((angular_distance(rahu.lon, ketu.lon) - 180.0).abs() < 1e-9);
            prop_assert!(
                (angular_distance(rahu.sidereal_lon, ketu.sidereal_lon) - 180.0).abs() < 1e-9
            );
        }

        #[test]
        fn whole_sign_house_is_sign_offset_from_lagna(
            lons in proptest::array::uniform7(0.0f64..360.0),
            asc in 0.0f64..360.0,
            node in 0.0f64..360.0,
        ) {
            let mut eph = FakeEph { asc, ..FakeEph::default() };
            for (body, lon) in PLANETS.iter().zip(lons) {
                eph = eph.with(*body, lon, 1.0);
            }
            eph = eph.with(Body::MeanNode, node, -0.053);
            let c = compute_chart(&mut eph, 2_451_545.0, 0.0, 0.0, &CalcSettings::default())
                .unwrap();
            for p in &c.grahas {
                let expected = (p.sign.index() + 12 - c.lagna.sign.index()) % 12 + 1;
                prop_assert_eq!(usize::from(p.house), expected, "{:?}", p.graha);
                prop_assert_eq!(p.house, house_from_cusps(p.lon, &c.bhavas.cusps));
            }
        }
    }
}

mod varga_properties {
    use proptest::prelude::*;

    use crate::varga::{varga_position, Varga};
    use crate::zodiac::{normalize, Sign};

    proptest! {
        #[test]
        fn varga_degree_is_in_range(lon in -720.0f64..720.0) {
            for v in Varga::ALL {
                let (_, deg) = varga_position(v, lon);
                prop_assert!((0.0..30.0).contains(&deg), "{v:?} at {lon}: {deg}");
            }
        }

        #[test]
        fn navamsa_matches_the_continuous_formula(lon in 0.0f64..360.0) {
            // BPHS kuralı (hareketli/sabit/çift doğalı başlangıcı) Koç'tan sürekli sayımla
            // aynıdır. Parça sınırında ulp düzeyinde fark olasılığı ihmal edilebilir.
            let continuous = Sign::from_index((normalize(lon) * 9.0 / 30.0) as usize);
            prop_assert_eq!(varga_position(Varga::D9, lon).0, continuous);
        }

        #[test]
        fn bhamsa_matches_the_continuous_formula(lon in 0.0f64..360.0) {
            let continuous = Sign::from_index((normalize(lon) * 27.0 / 30.0) as usize);
            prop_assert_eq!(varga_position(Varga::D27, lon).0, continuous);
        }

        #[test]
        fn trimsamsa_never_falls_in_luminary_signs(lon in 0.0f64..360.0) {
            let (sign, _) = varga_position(Varga::D30, lon);
            prop_assert!(sign != Sign::Cancer && sign != Sign::Leo, "{sign:?}");
            let odd = Sign::of(lon).index().is_multiple_of(2);
            prop_assert_eq!(sign.index().is_multiple_of(2), odd);
        }
    }
}

mod dasha_properties {
    use proptest::prelude::*;

    use crate::dasha::{
        dasha_children, mahadashas, vimshottari, vimshottari_years, DashaPeriod, DashaSeed,
        VIMSHOTTARI_ORDER, VIMSHOTTARI_TOTAL_YEARS,
    };
    use crate::graha::Graha;

    /// JD ~2.4e6 civarında f64 çözünürlüğü ~5e-10 gün; 1e-6 gün (~0.09 sn) bol paydır.
    const TOL_DAYS: f64 = 1e-6;

    fn seed_strategy() -> impl Strategy<Value = DashaSeed> {
        (
            0.0f64..360.0,
            2_300_000.0f64..2_500_000.0,
            prop::sample::select(vec![360.0, 365.2422, 365.25]),
        )
            .prop_map(|(moon, jd, year)| DashaSeed {
                birth_jd_ut: jd,
                moon_sidereal_lon: moon,
                year_days: year,
            })
    }

    fn lord() -> impl Strategy<Value = Graha> {
        prop::sample::select(VIMSHOTTARI_ORDER.to_vec())
    }

    /// Alt dönemler ebeveyni boşluksuz ve örtüşmesiz böler; paylar yıl oranındadır.
    fn check_partition(
        start: f64,
        end: f64,
        children: &[DashaPeriod],
    ) -> Result<(), TestCaseError> {
        prop_assert_eq!(children.len(), 9);
        prop_assert_eq!(children[0].start_jd_ut, start);
        prop_assert_eq!(children[8].end_jd_ut, end);
        for w in children.windows(2) {
            prop_assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
        }
        let span = end - start;
        let mut sum = 0.0;
        for c in children {
            let d = c.end_jd_ut - c.start_jd_ut;
            prop_assert!(d > 0.0, "{:?}", c.lords);
            let lord = *c.lords.last().unwrap();
            let want = span * vimshottari_years(lord) / VIMSHOTTARI_TOTAL_YEARS;
            prop_assert!((d - want).abs() < TOL_DAYS, "{:?}: {d} vs {want}", c.lords);
            sum += d;
        }
        prop_assert!((sum - span).abs() < TOL_DAYS);
        Ok(())
    }

    proptest! {
        #[test]
        fn mahadashas_total_120_years_and_contain_birth(seed in seed_strategy()) {
            let m = mahadashas(&seed).unwrap();
            check_partition(m[0].start_jd_ut, m[8].end_jd_ut, &m)?;
            let total = m[8].end_jd_ut - m[0].start_jd_ut;
            prop_assert!((total - VIMSHOTTARI_TOTAL_YEARS * seed.year_days).abs() < TOL_DAYS);
            prop_assert!(m[0].start_jd_ut <= seed.birth_jd_ut);
            prop_assert!(seed.birth_jd_ut <= m[0].end_jd_ut + TOL_DAYS);
        }

        #[test]
        fn sub_periods_partition_their_parent(
            seed in seed_strategy(),
            path in proptest::collection::vec(lord(), 1..=4),
        ) {
            let (last, parent_path) = path.split_last().unwrap();
            let siblings = dasha_children(&seed, parent_path).unwrap();
            let parent = siblings.iter().find(|p| p.lords.last() == Some(last)).unwrap();
            let children = dasha_children(&seed, &path).unwrap();
            check_partition(parent.start_jd_ut, parent.end_jd_ut, &children)?;
            for c in &children {
                prop_assert_eq!(&c.lords[..path.len()], &path[..]);
                prop_assert_eq!(usize::from(c.level), path.len() + 1);
            }
        }
    }

    proptest! {
        // Her durumda 90 ebeveyn × 819 dönem taranır; debug derlemede süreyi sınırlı tut.
        #![proptest_config(ProptestConfig::with_cases(32))]
        #[test]
        fn chart_dasha_matches_dasha_children(seed in seed_strategy()) {
            let d = vimshottari(seed).unwrap();
            prop_assert_eq!(d.periods.len(), 9 + 81 + 729);
            for p in d.periods.iter().filter(|p| p.level < 3) {
                let kids: Vec<DashaPeriod> = d
                    .periods
                    .iter()
                    .filter(|c| {
                        c.level == p.level + 1 && c.lords[..p.lords.len()] == p.lords[..]
                    })
                    .cloned()
                    .collect();
                prop_assert_eq!(kids, dasha_children(&seed, &p.lords).unwrap());
            }
        }
    }
}

mod panchang_properties {
    use ephemeris::Body;
    use proptest::prelude::*;

    use crate::fake::FakeEph;
    use crate::panchang::{
        compute_panchang, elongation, karana_no, nitya_yoga_no, tithi_no, END_SEARCH_DAYS,
    };
    use crate::settings::CalcSettings;
    use crate::zodiac::{nakshatra_pada, normalize};

    const JD: f64 = 2_451_545.0;
    const ONE_SECOND: f64 = 1.0 / 86_400.0;
    /// FakeEph varsayılan ayanamsası.
    const AYANAMSA: f64 = 24.0;

    proptest! {
        #[test]
        fn end_times_close_the_current_element(
            sun in 0.0f64..360.0,
            moon in 0.0f64..360.0,
            sun_speed in 0.95f64..1.02,
            moon_speed in 11.8f64..15.4,
        ) {
            let mut eph = FakeEph::default()
                .with(Body::Sun, sun, sun_speed)
                .with(Body::Moon, moon, moon_speed);
            let p = compute_panchang(&mut eph, JD, 41.0, 29.0, &CalcSettings::default()).unwrap();
            let ends = [
                p.tithi.ends_jd_ut,
                p.nakshatra.ends_jd_ut,
                p.yoga.ends_jd_ut,
                p.karana.ends_jd_ut,
            ];
            for end in ends {
                prop_assert!((JD..=JD + END_SEARCH_DAYS).contains(&end), "{end}");
            }
            prop_assert!(p.karana.ends_jd_ut <= p.tithi.ends_jd_ut + ONE_SECOND);
            // Bitişten 1 sn sonra her öğe bir sonraki numaradadır (sahte hareket doğrusal).
            let after = |end: f64| {
                let dt = end + ONE_SECOND - JD;
                let s = normalize(sun - AYANAMSA + sun_speed * dt);
                let m = normalize(moon - AYANAMSA + moon_speed * dt);
                (s, m)
            };
            let (s, m) = after(p.tithi.ends_jd_ut);
            prop_assert_eq!(tithi_no(elongation(s, m)), p.tithi.no % 30 + 1);
            let (_, m) = after(p.nakshatra.ends_jd_ut);
            prop_assert_eq!(nakshatra_pada(m).0, p.nakshatra.no % 27 + 1);
            let (s, m) = after(p.yoga.ends_jd_ut);
            prop_assert_eq!(nitya_yoga_no(s, m), p.yoga.no % 27 + 1);
            let (s, m) = after(p.karana.ends_jd_ut);
            prop_assert_eq!(karana_no(elongation(s, m)), p.karana.no % 60 + 1);
        }
    }
}

mod ashtakavarga_properties {
    use proptest::prelude::*;

    use crate::ashtakavarga::{compute_ashtakavarga, BAV_TOTALS, SAV_TOTAL};
    use crate::zodiac::Sign;

    proptest! {
        #[test]
        fn bav_and_sav_totals_are_fixed(
            idx in proptest::array::uniform7(0usize..12),
            lagna in 0usize..12,
        ) {
            let a = compute_ashtakavarga(idx.map(Sign::from_index), Sign::from_index(lagna));
            for (row, total) in a.bav.iter().zip(BAV_TOTALS) {
                prop_assert_eq!(row.iter().map(|&v| u16::from(v)).sum::<u16>(), u16::from(total));
                // Bir burca en çok sekiz referans bindu verir.
                prop_assert!(row.iter().all(|&v| v <= 8));
            }
            prop_assert_eq!(a.sav.iter().map(|&v| u16::from(v)).sum::<u16>(), SAV_TOTAL);
            for s in 0..12 {
                prop_assert_eq!(a.sav[s], a.bav.iter().map(|row| row[s]).sum::<u8>());
            }
        }
    }
}
