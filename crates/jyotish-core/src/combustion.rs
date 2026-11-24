//! Yanma (astangata): Güneş'e orb içinde yakınlık. Spec §6, BPHS orbları.

use crate::graha::Graha;
use crate::settings::CombustionOrbs;
use crate::zodiac::angular_distance;

/// `lon` ile `sun_lon` aynı çerçevede olmalıdır (fark iki çerçevede de aynıdır).
/// Sınır dahildir (`≤ orb`). Güneş ve düğümler asla yanmaz.
pub fn is_combust(
    graha: Graha,
    lon: f64,
    sun_lon: f64,
    retrograde: bool,
    orbs: &CombustionOrbs,
) -> bool {
    let orb = match graha {
        Graha::Sun | Graha::Rahu | Graha::Ketu => return false,
        Graha::Moon => orbs.moon,
        Graha::Mars => orbs.mars,
        Graha::Mercury if retrograde => orbs.mercury_retro,
        Graha::Mercury => orbs.mercury,
        Graha::Jupiter => orbs.jupiter,
        Graha::Venus if retrograde => orbs.venus_retro,
        Graha::Venus => orbs.venus,
        Graha::Saturn => orbs.saturn,
    };
    angular_distance(lon, sun_lon) <= orb
}
#[cfg(test)]
mod tests {
    use super::*;

    fn orbs() -> CombustionOrbs {
        CombustionOrbs::default()
    }

    #[test]
    fn direct_orbs_are_inclusive() {
        assert!(is_combust(Graha::Mars, 117.0, 100.0, false, &orbs()));
        assert!(!is_combust(Graha::Mars, 117.1, 100.0, false, &orbs()));
        assert!(is_combust(Graha::Moon, 112.0, 100.0, false, &orbs()));
        assert!(is_combust(Graha::Mercury, 113.9, 100.0, false, &orbs()));
        assert!(!is_combust(Graha::Mercury, 114.1, 100.0, false, &orbs()));
        assert!(is_combust(Graha::Jupiter, 89.0, 100.0, false, &orbs()));
        assert!(is_combust(Graha::Venus, 109.9, 100.0, false, &orbs()));
        assert!(!is_combust(Graha::Venus, 110.1, 100.0, false, &orbs()));
        assert!(is_combust(Graha::Saturn, 85.0, 100.0, false, &orbs()));
        assert!(!is_combust(Graha::Saturn, 84.9, 100.0, false, &orbs()));
    }

    #[test]
    fn retrograde_mercury_and_venus_use_smaller_orbs() {
        assert!(!is_combust(Graha::Mercury, 112.1, 100.0, true, &orbs()));
        assert!(is_combust(Graha::Mercury, 111.9, 100.0, true, &orbs()));
        assert!(!is_combust(Graha::Venus, 108.1, 100.0, true, &orbs()));
        assert!(is_combust(Graha::Venus, 107.9, 100.0, true, &orbs()));
        // Diğerlerinde retro orbu değiştirmez.
        assert!(is_combust(Graha::Saturn, 114.0, 100.0, true, &orbs()));
    }

    #[test]
    fn sun_and_nodes_never_combust() {
        assert!(!is_combust(Graha::Sun, 100.0, 100.0, false, &orbs()));
        assert!(!is_combust(Graha::Rahu, 100.0, 100.0, true, &orbs()));
        assert!(!is_combust(Graha::Ketu, 100.0, 100.0, true, &orbs()));
    }

    #[test]
    fn distance_wraps_around_zero() {
        assert!(is_combust(Graha::Moon, 355.0, 5.0, false, &orbs()));
        assert!(is_combust(Graha::Moon, 5.0, 355.0, false, &orbs()));
    }
}
