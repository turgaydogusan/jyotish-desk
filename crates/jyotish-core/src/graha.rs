//! Dokuz graha.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Graha {
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
    Rahu,
    Ketu,
}

impl Graha {
    /// Geleneksel sıra (gün sırası, sonra düğümler); `Chart::grahas` bu sıradadır.
    pub const ALL: [Graha; 9] = [
        Graha::Sun,
        Graha::Moon,
        Graha::Mars,
        Graha::Mercury,
        Graha::Jupiter,
        Graha::Venus,
        Graha::Saturn,
        Graha::Rahu,
        Graha::Ketu,
    ];

    pub fn is_node(self) -> bool {
        matches!(self, Self::Rahu | Self::Ketu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_in_traditional_order_and_nodes() {
        assert_eq!(
            Graha::ALL,
            [
                Graha::Sun,
                Graha::Moon,
                Graha::Mars,
                Graha::Mercury,
                Graha::Jupiter,
                Graha::Venus,
                Graha::Saturn,
                Graha::Rahu,
                Graha::Ketu
            ]
        );
        let nodes: Vec<_> = Graha::ALL.into_iter().filter(|g| g.is_node()).collect();
        assert_eq!(nodes, [Graha::Rahu, Graha::Ketu]);
    }
}
