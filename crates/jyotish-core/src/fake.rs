//! Birim testleri için sahte ephemeris. Tropikal boylamlar sabittir;
//! sidereal = tropikal − `ayanamsa`. Evler lagnadan eşit 30°'lik cusp'lardır.

use ephemeris::{
    Ayanamsa, Body, BodyPosition, EphError, Ephemeris, HouseSystem, Houses, Precision,
    SiderealMode, Zodiac,
};

use crate::zodiac::normalize;

pub(crate) struct FakeEph {
    /// (cisim, tropikal boylam, boylam hızı °/gün)
    pub positions: Vec<(Body, f64, f64)>,
    pub ayanamsa: f64,
    /// Tropikal lagna.
    pub asc: f64,
    pub precision: Precision,
    pub porphyry_fallback: bool,
    /// Son `houses` çağrısında istenen sistem.
    pub last_house_system: Option<HouseSystem>,
}

impl Default for FakeEph {
    fn default() -> Self {
        Self {
            positions: vec![
                (Body::Sun, 10.0, 0.98),
                (Body::Moon, 95.0, 13.2),
                (Body::Mercury, 20.0, 1.2),
                (Body::Venus, 40.0, 1.1),
                (Body::Mars, 200.0, 0.6),
                (Body::Jupiter, 250.0, -0.1),
                (Body::Saturn, 300.0, 0.05),
                (Body::MeanNode, 120.0, -0.053),
                (Body::TrueNode, 125.0, 0.01),
            ],
            ayanamsa: 24.0,
            asc: 70.0,
            precision: Precision::SwissEph,
            porphyry_fallback: false,
            last_house_system: None,
        }
    }
}

impl FakeEph {
    pub fn with(mut self, body: Body, lon: f64, speed: f64) -> Self {
        for p in &mut self.positions {
            if p.0 == body {
                p.1 = lon;
                p.2 = speed;
            }
        }
        self
    }

    fn in_frame(&self, tropical: f64, zodiac: Zodiac) -> f64 {
        match zodiac {
            Zodiac::Tropical => normalize(tropical),
            Zodiac::Sidereal(_) => normalize(tropical - self.ayanamsa),
        }
    }
}

impl Ephemeris for FakeEph {
    fn body(&mut self, _jd_ut: f64, body: Body, zodiac: Zodiac) -> Result<BodyPosition, EphError> {
        let &(_, lon, speed) = self
            .positions
            .iter()
            .find(|p| p.0 == body)
            .ok_or_else(|| EphError::Calc(format!("fake: no {body:?}")))?;
        Ok(BodyPosition {
            lon: self.in_frame(lon, zodiac),
            lat: 0.0,
            distance_au: 1.0,
            lon_speed: speed,
            lat_speed: 0.0,
            dist_speed: 0.0,
            precision: self.precision,
        })
    }

    fn ayanamsa(&mut self, _jd_ut: f64, _mode: SiderealMode) -> Result<Ayanamsa, EphError> {
        Ok(Ayanamsa {
            deg: self.ayanamsa,
            precision: Precision::SwissEph,
        })
    }

    fn houses(
        &mut self,
        _jd_ut: f64,
        _geo_lat: f64,
        _geo_lon: f64,
        system: HouseSystem,
        zodiac: Zodiac,
    ) -> Result<Houses, EphError> {
        self.last_house_system = Some(system);
        let asc = self.in_frame(self.asc, zodiac);
        let cusps: [f64; 12] = std::array::from_fn(|i| normalize(asc + 30.0 * i as f64));
        Ok(Houses {
            cusps,
            asc,
            mc: normalize(asc + 270.0),
            porphyry_fallback: self.porphyry_fallback,
        })
    }

    fn swe_version(&self) -> String {
        "fake-2.10".to_owned()
    }
}
