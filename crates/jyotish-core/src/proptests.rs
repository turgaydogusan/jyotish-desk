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
