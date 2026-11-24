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
