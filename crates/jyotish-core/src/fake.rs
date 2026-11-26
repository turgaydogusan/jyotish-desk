//! Birim testleri için sahte ephemeris. Tropikal boylamlar `epoch` anında verilir ve
//! sabit hızla doğrusal ilerler; sidereal = tropikal − `ayanamsa`. Evler lagnadan eşit
//! 30°'lik cusp'lardır. Gün doğumu/batımı her gün aynı UT saatindedir.

use ephemeris::{
    Ayanamsa, Body, BodyPosition, EphError, Ephemeris, HouseSystem, Houses, Precision, RiseSet,
    SiderealMode, SunDisc, Zodiac,
};

use crate::zodiac::normalize;

/// `FakeEph::sunrises` verildiğinde doğuş aramasının ileriye baktığı süre (26 saat).
pub(crate) const FAKE_RISE_HORIZON_DAYS: f64 = 26.0 / 24.0;

pub(crate) struct FakeEph {
    /// (cisim, tropikal boylam, boylam hızı °/gün)
    pub positions: Vec<(Body, f64, f64)>,
    pub ayanamsa: f64,
    /// Tropikal lagna (`epoch` anında).
    pub asc: f64,
    /// Lagnanın hızı, °/gün; 0 ise lagna sabittir. Zaman çizelgesi testleri 360 verir.
    pub asc_speed: f64,
    pub precision: Precision,
    /// Yalnızca bu cisimler `Moshier` hassasiyetinde döner (`precision` ne olursa olsun).
    pub moshier_bodies: Vec<Body>,
    pub porphyry_fallback: bool,
    /// Son `houses` çağrısında istenen sistem.
    pub last_house_system: Option<HouseSystem>,
    /// Sahte gün doğumu ve batımı her gün `n + offset` anındadır (n tamsayı JD; JD n + 0.5
    /// UT gece yarısıdır). 0.67 ≈ 04:05 UT, 0.15 ≈ 15:36 UT.
    pub sunrise_offset: f64,
    pub sunset_offset: f64,
    /// Güneş hiç doğup batmıyor (kutup gecesi ya da gündüzü).
    pub polar: bool,
    /// `Some` ise gün doğumları yalnızca bu anlardır (`sunrise_offset` yok sayılır) ve arama,
    /// SE'nin yüksek enlem araması gibi, başlangıçtan en çok `FAKE_RISE_HORIZON_DAYS` ileriye
    /// bakar.
    pub sunrises: Option<Vec<f64>>,
    /// Konumların tanımlandığı an; boylam `lon + hız × (jd − epoch)` olarak ilerler.
    pub epoch: f64,
    /// Son `sun_rise_set` çağrısında istenen disk tanımı.
    pub last_sun_disc: Option<SunDisc>,
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
            asc_speed: 0.0,
            precision: Precision::SwissEph,
            moshier_bodies: Vec::new(),
            porphyry_fallback: false,
            last_house_system: None,
            sunrise_offset: 0.67,
            sunset_offset: 0.15,
            polar: false,
            sunrises: None,
            epoch: 2_451_545.0,
            last_sun_disc: None,
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
    fn body(&mut self, jd_ut: f64, body: Body, zodiac: Zodiac) -> Result<BodyPosition, EphError> {
        let &(_, lon, speed) = self
            .positions
            .iter()
            .find(|p| p.0 == body)
            .ok_or_else(|| EphError::Calc(format!("fake: no {body:?}")))?;
        Ok(BodyPosition {
            lon: self.in_frame(lon + speed * (jd_ut - self.epoch), zodiac),
            lat: 0.0,
            distance_au: 1.0,
            lon_speed: speed,
            lat_speed: 0.0,
            dist_speed: 0.0,
            precision: if self.precision == Precision::Moshier
                || self.moshier_bodies.contains(&body)
            {
                Precision::Moshier
            } else {
                self.precision
            },
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
        jd_ut: f64,
        _geo_lat: f64,
        _geo_lon: f64,
        system: HouseSystem,
        zodiac: Zodiac,
    ) -> Result<Houses, EphError> {
        self.last_house_system = Some(system);
        let asc = self.in_frame(self.asc + self.asc_speed * (jd_ut - self.epoch), zodiac);
        let cusps: [f64; 12] = std::array::from_fn(|i| normalize(asc + 30.0 * i as f64));
        Ok(Houses {
            cusps,
            asc,
            mc: normalize(asc + 270.0),
            porphyry_fallback: self.porphyry_fallback,
        })
    }

    fn sun_rise_set(
        &mut self,
        after_jd_ut: f64,
        event: RiseSet,
        _geo_lat: f64,
        _geo_lon: f64,
        disc: SunDisc,
    ) -> Result<Option<f64>, EphError> {
        self.last_sun_disc = Some(disc);
        if self.polar {
            return Ok(None);
        }
        if let (RiseSet::Rise, Some(rises)) = (event, &self.sunrises) {
            return Ok(rises
                .iter()
                .copied()
                .find(|&t| t > after_jd_ut && t <= after_jd_ut + FAKE_RISE_HORIZON_DAYS));
        }
        let offset = match event {
            RiseSet::Rise => self.sunrise_offset,
            RiseSet::Set => self.sunset_offset,
        };
        // `after_jd_ut`'den kesinlikle sonraki ilk `n + offset`.
        Ok(Some((after_jd_ut - offset).floor() + 1.0 + offset))
    }

    fn swe_version(&self) -> String {
        "fake-2.10".to_owned()
    }
}
