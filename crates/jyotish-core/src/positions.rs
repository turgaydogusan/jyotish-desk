//! İleri modüllerin ortak girdisi (ileri modüller spec'i K2, K3): `ChartInput` komut
//! girdisidir, `Positions` ise her modülün saf hesap fonksiyonunun gördüğü, ephemeris
//! içermeyen düz konum kümesidir. El örnekleri `Positions::synthetic` ile kurulur.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{compute_chart_for, Chart};
use crate::graha::Graha;
use crate::settings::CalcSettings;
use crate::zodiac::{deg_in_sign, normalize, Sign};
use crate::CoreError;

/// Modül komutlarının ortak girdisi: `compute_chart` ile aynı alanlar ve `time_known`.
/// Doğum saati bilinmiyorsa (`time_known == false`) lagnaya bağlı her çıktı `None` döner;
/// arayüzün gizlemesine güvenilmez (K2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ChartInput {
    pub jd_ut: f64,
    /// Kuzey +.
    pub lat: f64,
    /// Doğu +.
    pub lon: f64,
    pub settings: CalcSettings,
    pub time_known: bool,
    /// Sayı prashnası (ileri modüller spec'i §11): doluysa lagna bu sayıdan gelir (1..=108,
    /// `number_lagna_lon`) ve evler o lagnadan burç evidir; grahalar anın grahalarıdır. Diğer
    /// her haritada `None`. `time_known` kuralı değişmez: yanlışsa lagnaya bağlı çıktı yine
    /// boştur (soru anı her zaman bilinir; arayüz sayı prashnasında `true` gönderir).
    pub lagna_number: Option<u8>,
}

/// Bir grahanın modüllerin kullandığı konumu.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GrahaPoint {
    pub graha: Graha,
    /// Her zodyak kipinde sidereal boylam, [0, 360). Nakshatra ve derece temelli kurallar
    /// (sphuta drishti) bunu kullanır.
    pub sidereal_lon: f64,
    /// Rasi çerçevesindeki burç (`settings.zodiac`'ı izler, K5). Burca dayalı her kural bunu
    /// kullanır; tropikal-rasi kipinde `sidereal_lon`'un burcundan farklı olabilir.
    pub rasi_sign: Sign,
    /// Rasi çerçevesinde burç içi derece, [0, 30).
    pub deg_in_sign: f64,
    /// Ekliptik enlem, derece.
    pub lat: f64,
    /// Boylam hızı, derece/gün.
    pub speed: f64,
}

/// Ephemeris görmeyen konum kümesi. `grahas` her zaman `Graha::ALL` sırasındadır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Positions {
    /// Lagnanın sidereal boylamı; doğum saati bilinmiyorsa `None`.
    pub lagna_lon: Option<f64>,
    /// Lagnanın rasi çerçevesindeki burcu; evler bundan sayılır (K4). `lagna_lon` ile
    /// birlikte dolu ya da birlikte boştur.
    pub lagna_sign: Option<Sign>,
    pub grahas: [GrahaPoint; 9],
}

impl From<&Chart> for Positions {
    /// Haritanın konumları, lagnasıyla. Doğum saati bilinmiyorsa çağıran
    /// `without_lagna` uygular (`compute_positions` bunu kendisi yapar).
    fn from(chart: &Chart) -> Self {
        Positions {
            lagna_lon: Some(chart.lagna.sidereal_lon),
            lagna_sign: Some(chart.lagna.sign),
            // `Chart::grahas` her zaman 9 elemandır ve `Graha::ALL` sırasındadır.
            grahas: std::array::from_fn(|i| {
                let g = &chart.grahas[i];
                GrahaPoint {
                    graha: g.graha,
                    sidereal_lon: g.sidereal_lon,
                    rasi_sign: g.sign,
                    deg_in_sign: g.deg_in_sign,
                    lat: g.lat,
                    speed: g.speed,
                }
            }),
        }
    }
}

impl Positions {
    /// Sentetik konumlar (el örnekleri ve testler): `lons`, `Graha::ALL` sırasında sidereal
    /// boylamlardır ve rasi çerçevesi sidereal sayılır. Enlem ve hız sıfırdır.
    pub fn synthetic(lagna_lon: Option<f64>, lons: [f64; 9]) -> Positions {
        let lagna_lon = lagna_lon.map(normalize);
        Positions {
            lagna_lon,
            lagna_sign: lagna_lon.map(Sign::of),
            grahas: std::array::from_fn(|i| {
                let lon = normalize(lons[i]);
                GrahaPoint {
                    graha: Graha::ALL[i],
                    sidereal_lon: lon,
                    rasi_sign: Sign::of(lon),
                    deg_in_sign: deg_in_sign(lon),
                    lat: 0.0,
                    speed: 0.0,
                }
            }),
        }
    }

    /// Lagnasız kopya: doğum saati bilinmeyen harita.
    pub fn without_lagna(mut self) -> Positions {
        self.lagna_lon = None;
        self.lagna_sign = None;
        self
    }

    pub fn point(&self, graha: Graha) -> &GrahaPoint {
        &self.grahas[graha as usize]
    }

    /// Grahanın rasi çerçevesindeki burcu.
    pub fn sign_of(&self, graha: Graha) -> Sign {
        self.point(graha).rasi_sign
    }

    /// `sign` burcundaki grahalar, `Graha::ALL` sırasında.
    pub fn occupants(&self, sign: Sign) -> Vec<Graha> {
        self.grahas
            .iter()
            .filter(|p| p.rasi_sign == sign)
            .map(|p| p.graha)
            .collect()
    }

    /// Grahanın `reference` burcundan sayılan rasi evi, 1..=12 (K4).
    pub fn house_from(&self, graha: Graha, reference: Sign) -> u8 {
        self.sign_of(graha).house_from(reference)
    }

    /// Grahanın lagnadan sayılan rasi evi; doğum saati bilinmiyorsa `None`.
    pub fn house_of(&self, graha: Graha) -> Option<u8> {
        self.lagna_sign.map(|l| self.house_from(graha, l))
    }
}

/// Toplayıcı: temel haritayı hesaplar ve konumlara indirger. `input.time_known` yanlışsa
/// lagna boştur; sayı prashnasında (`input.lagna_number`) lagna sayıdan gelir
/// (`compute_chart_for`). Ayar, an ve konum denetimi `compute_chart`'ınkidir.
pub fn compute_positions<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<Positions, CoreError> {
    let chart = compute_chart_for(eph, input)?;
    let positions = Positions::from(&chart);
    Ok(if input.time_known {
        positions
    } else {
        positions.without_lagna()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::compute_chart;
    use crate::fake::FakeEph;
    use crate::settings::ZodiacMode;

    const JD: f64 = 2_451_545.0;

    fn input(settings: CalcSettings, time_known: bool) -> ChartInput {
        ChartInput {
            jd_ut: JD,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known,
            lagna_number: None,
        }
    }

    #[test]
    fn synthetic_positions_are_in_graha_order_with_sidereal_signs() {
        let p = Positions::synthetic(
            Some(370.0),
            [10.0, 95.0, 200.0, 20.0, 250.0, 40.0, 300.0, 120.0, -60.0],
        );
        assert_eq!(p.lagna_lon, Some(10.0));
        assert_eq!(p.lagna_sign, Some(Sign::Aries));
        for (i, g) in Graha::ALL.into_iter().enumerate() {
            assert_eq!(p.grahas[i].graha, g);
            assert_eq!(p.point(g).graha, g);
        }
        assert_eq!(p.sign_of(Graha::Moon), Sign::Cancer);
        assert_eq!(p.point(Graha::Moon).deg_in_sign, 5.0);
        assert_eq!(p.point(Graha::Ketu).sidereal_lon, 300.0);
        assert_eq!(p.occupants(Sign::Aries), [Graha::Sun, Graha::Mercury]);
        assert_eq!(p.occupants(Sign::Aquarius), [Graha::Saturn, Graha::Ketu]);
        assert!(p.occupants(Sign::Gemini).is_empty());
        assert_eq!(p.house_of(Graha::Moon), Some(4));
        assert_eq!(p.house_from(Graha::Sun, Sign::Cancer), 10);
        let q = p.clone().without_lagna();
        assert_eq!((q.lagna_lon, q.lagna_sign), (None, None));
        assert_eq!(q.house_of(Graha::Moon), None);
        assert_eq!(q.grahas, p.grahas);
    }

    #[test]
    fn positions_follow_the_chart_and_hide_the_lagna_when_time_is_unknown() {
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        let chart = compute_chart(&mut eph, JD, 41.0, 29.0, &s).unwrap();
        let p = compute_positions(&mut eph, &input(s.clone(), true)).unwrap();
        assert_eq!(p, Positions::from(&chart));
        assert_eq!(p.lagna_lon, Some(chart.lagna.sidereal_lon));
        assert_eq!(p.lagna_sign, Some(chart.lagna.sign));
        for (point, g) in p.grahas.iter().zip(&chart.grahas) {
            assert_eq!(
                (point.graha, point.sidereal_lon, point.rasi_sign),
                (g.graha, g.sidereal_lon, g.sign)
            );
            assert_eq!(
                (point.deg_in_sign, point.lat, point.speed),
                (g.deg_in_sign, g.lat, g.speed)
            );
        }
        let unknown = compute_positions(&mut eph, &input(s, false)).unwrap();
        assert_eq!((unknown.lagna_lon, unknown.lagna_sign), (None, None));
        assert_eq!(unknown.grahas, p.grahas);
    }

    #[test]
    fn tropical_rasi_mode_keeps_sidereal_longitudes_and_tropical_signs() {
        // FakeEph: Güneş tropikal 10° (Koç), ayanamsa 24° → sidereal 346° (Balık).
        let mut eph = FakeEph::default();
        let s = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        let p = compute_positions(&mut eph, &input(s, true)).unwrap();
        let sun = p.point(Graha::Sun);
        assert_eq!(sun.rasi_sign, Sign::Aries);
        assert!((sun.deg_in_sign - 10.0).abs() < 1e-9);
        assert!((sun.sidereal_lon - 346.0).abs() < 1e-9);
        // Lagna tropikal 70° (İkizler); sidereal boylamı 46°.
        assert_eq!(p.lagna_sign, Some(Sign::Gemini));
        assert!((p.lagna_lon.unwrap() - 46.0).abs() < 1e-9);
    }

    #[test]
    fn a_lagna_number_moves_the_lagna_and_keeps_the_grahas() {
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        let plain = compute_positions(&mut eph, &input(s.clone(), true)).unwrap();
        // 30. sayı: 98°20′ sidereal, Yengeç.
        let mut numbered = input(s, true);
        numbered.lagna_number = Some(30);
        let p = compute_positions(&mut eph, &numbered).unwrap();
        assert!((p.lagna_lon.unwrap() - 5900.0 / 60.0).abs() < 1e-12);
        assert_eq!(p.lagna_sign, Some(Sign::Cancer));
        assert_eq!(p.grahas, plain.grahas);
        // FakeEph'te Ay sidereal 71° (İkizler): Yengeç lagnasından 12. ev.
        assert_eq!(p.house_of(Graha::Moon), Some(12));
        // `time_known` kuralı değişmez.
        numbered.time_known = false;
        let hidden = compute_positions(&mut eph, &numbered).unwrap();
        assert_eq!((hidden.lagna_lon, hidden.lagna_sign), (None, None));
        numbered.lagna_number = Some(0);
        assert_eq!(
            compute_positions(&mut eph, &numbered),
            Err(CoreError::InvalidPrashnaInput("number"))
        );
    }

    #[test]
    fn collector_reports_the_chart_errors() {
        let mut eph = FakeEph::default();
        let mut bad = input(CalcSettings::default(), true);
        bad.lat = 91.0;
        assert_eq!(
            compute_positions(&mut eph, &bad),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0
            })
        );
        let old = CalcSettings {
            schema_version: 1,
            ..CalcSettings::default()
        };
        assert_eq!(
            compute_positions(&mut eph, &input(old, true)),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 1,
                supported: 2
            })
        );
    }
}
