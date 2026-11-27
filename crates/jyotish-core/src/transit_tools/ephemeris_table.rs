//! Efemeris tablosu (yol haritası §4.9): verilen anlarda dokuz grahanın konumu, satır satır.
//!
//! Konum hesabı transit hesabıyla aynıdır (`chart::raw_grahas`): boylam, burç ve burç içi derece
//! rasi çerçevesindedir (K5); nakshatra ve pada sidereal boylamdandır. Çağıran anları verir
//! (uygulama katmanı: bir ayın günleri, yerin gece yarısında) ve son satırı kapatan bir an daha
//! ekler: her satır, bir sonraki ana dek olan değişimi iki işaretle taşır. `ingress`: graha o
//! süre içinde başka bir burca girer (girilen burç). `station`: graha o süre içinde yön
//! değiştirir (retroya ya da düze döner). İşaretler iki ardışık örneğin karşılaştırmasıdır;
//! kesin an olay aramasındadır. Aynı süre içinde gidip dönen bir değişim (durağan grahanın burç
//! sınırını iki kez geçmesi) görülmez.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{finite, rasi_frame, raw_grahas, to_sidereal, ChartWarning};
use crate::graha::Graha;
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::timeline::{Motion, STATION_GRAHAS};
use crate::zodiac::{deg_in_sign, nakshatra_pada, Sign};
use crate::CoreError;

/// Yöntem kimliği (K7).
pub const EPHEMERIS_METHOD: &str = "ephemeris_table.v1";
/// Bir tablonun en çok satırı: bir ay.
pub const EPHEMERIS_MAX_ROWS: usize = 31;
/// Ardışık iki an arasındaki en uzun süre, gün. İşaretler satır aralığının bir günü pek
/// aşmamasına dayanır (Ay iki günde bir burçtan fazla yürümez).
pub const EPHEMERIS_MAX_STEP_DAYS: f64 = 2.0;

/// Bir grahanın bir satırdaki konumu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct EphemerisCell {
    pub graha: Graha,
    /// Rasi çerçevesinde boylam, [0, 360).
    pub lon: f64,
    pub sign: Sign,
    pub deg_in_sign: f64,
    /// Boylam hızı < 0 (v1'in retro tanımı).
    pub retrograde: bool,
    /// Sidereal boylamdan.
    pub nakshatra: u8,
    pub pada: u8,
    /// Sonraki ana dek girilen burç; burç değişmiyorsa `None`.
    pub ingress: Option<Sign>,
    /// Sonraki ana dek dönülen yön; yalnızca `STATION_GRAHAS` için, yön değişmiyorsa `None`.
    pub station: Option<Motion>,
}

/// Bir anın dokuz grahası.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct EphemerisRow {
    pub jd_ut: f64,
    /// Her zaman 9 eleman, `Graha::ALL` sırasında.
    pub cells: Vec<EphemerisCell>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct EphemerisTable {
    /// Verilen anların sonuncusu dışında her an için bir satır.
    pub rows: Vec<EphemerisRow>,
    /// Yalnızca `LowPrecision` olabilir.
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

fn invalid(field: &'static str) -> CoreError {
    CoreError::InvalidTransitInput(field)
}

/// Bir anın dokuz hücresi (işaretsiz) ve Güneş ya da Ay'ın Moshier yedeğiyle hesaplanıp
/// hesaplanmadığı.
fn sample<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    jd_ut: f64,
) -> Result<(Vec<EphemerisCell>, bool), CoreError> {
    let ayanamsa = eph.ayanamsa(jd_ut, settings.ayanamsa.sidereal_mode())?;
    finite(ayanamsa.deg, "ayanamsa")?;
    let (raw, low_precision) = raw_grahas(eph, jd_ut, rasi_frame(settings), settings.node)?;
    let cells = raw
        .iter()
        .map(|r| {
            let (nakshatra, pada) =
                nakshatra_pada(to_sidereal(settings.zodiac, r.lon, ayanamsa.deg));
            EphemerisCell {
                graha: r.graha,
                lon: r.lon,
                sign: Sign::of(r.lon),
                deg_in_sign: deg_in_sign(r.lon),
                retrograde: r.speed < 0.0,
                nakshatra,
                pada,
                ingress: None,
                station: None,
            }
        })
        .collect();
    Ok((cells, low_precision))
}

/// `moments` anlarının efemeris tablosu. Anlar artan sıradadır; sonuncusu yalnızca son satırın
/// işaretlerini kapatır ve satır olmaz: `n + 1` an `n` satır verir (1 ≤ n ≤ `EPHEMERIS_MAX_ROWS`).
/// Girdi hatası yalnızca alan adını taşır (`moments`).
pub fn ephemeris_table<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    moments: &[f64],
) -> Result<EphemerisTable, CoreError> {
    settings.validate()?;
    let count_ok = (2..=EPHEMERIS_MAX_ROWS + 1).contains(&moments.len());
    // NaN ve sonsuz da aralık dışındadır.
    let in_range = moments.iter().all(|jd| JD_UT_RANGE.contains(jd));
    let ordered = moments
        .windows(2)
        .all(|w| w[0] < w[1] && w[1] - w[0] <= EPHEMERIS_MAX_STEP_DAYS);
    if !(count_ok && in_range && ordered) {
        return Err(invalid("moments"));
    }
    let mut low_precision = false;
    let mut samples = Vec::with_capacity(moments.len());
    for &jd_ut in moments {
        let (cells, low) = sample(eph, settings, jd_ut)?;
        low_precision |= low;
        samples.push(cells);
    }
    let rows = moments[..moments.len() - 1]
        .iter()
        .enumerate()
        .map(|(i, &jd_ut)| {
            let cells = samples[i]
                .iter()
                .zip(&samples[i + 1])
                .map(|(now, next)| EphemerisCell {
                    ingress: (next.sign != now.sign).then_some(next.sign),
                    station: (STATION_GRAHAS.contains(&now.graha)
                        && next.retrograde != now.retrograde)
                        .then_some(if next.retrograde {
                            Motion::Retrograde
                        } else {
                            Motion::Direct
                        }),
                    ..now.clone()
                })
                .collect();
            EphemerisRow { jd_ut, cells }
        })
        .collect();
    Ok(EphemerisTable {
        rows,
        warnings: if low_precision {
            vec![ChartWarning::LowPrecision]
        } else {
            Vec::new()
        },
        method: EPHEMERIS_METHOD.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use ephemeris::{
        Ayanamsa, Body, BodyPosition, EphError, HouseSystem, Houses, Precision, RiseSet,
        SiderealMode, SunDisc, Zodiac,
    };

    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::ZodiacMode;
    use crate::transit::compute_transits;
    use crate::zodiac::normalize;
    use crate::{compute_chart, Bhavas};

    const JD: f64 = 2_451_545.0;

    /// `JD`'den başlayan `rows` günlük tablo: `rows + 1` an.
    fn days(rows: usize) -> Vec<f64> {
        (0..=rows).map(|i| JD + i as f64).collect()
    }

    fn table(settings: &CalcSettings, rows: usize) -> EphemerisTable {
        ephemeris_table(&mut FakeEph::default(), settings, &days(rows)).unwrap()
    }

    fn cell(row: &EphemerisRow, graha: Graha) -> &EphemerisCell {
        row.cells.iter().find(|c| c.graha == graha).unwrap()
    }

    fn natal_bhavas(settings: &CalcSettings) -> Bhavas {
        compute_chart(&mut FakeEph::default(), JD, 41.0, 29.0, settings)
            .unwrap()
            .bhavas
    }

    #[test]
    fn a_row_is_the_nine_grahas_of_its_moment() {
        // Sahte gök (sidereal = tropikal − 24°): Güneş `JD` anında 346°, günde 0,98°; Ay 71°,
        // günde 13,2°; Jüpiter 226°, günde −0,1° (retro).
        let t = table(&CalcSettings::default(), 10);
        assert_eq!(t.method, "ephemeris_table.v1");
        assert!(t.warnings.is_empty());
        assert_eq!(t.rows.len(), 10);
        for (i, row) in t.rows.iter().enumerate() {
            assert_eq!(row.jd_ut, JD + i as f64);
            let grahas: Vec<Graha> = row.cells.iter().map(|c| c.graha).collect();
            assert_eq!(grahas, Graha::ALL);
        }
        // Satır 0: Güneş 346° = Balık 16°; Ay 71° = İkizler 11°, 71 / 13,3333 = 5,3 → 6.
        // nakshatra (Ardra), içinde 71 − 66,667 = 4,333°, 4,333 / 3,3333 = 1,3 → 2. pada.
        let r0 = &t.rows[0];
        let sun = cell(r0, Graha::Sun);
        assert_eq!(sun.sign, Sign::Pisces);
        assert!((sun.lon - 346.0).abs() < 1e-9 && (sun.deg_in_sign - 16.0).abs() < 1e-9);
        let moon = cell(r0, Graha::Moon);
        assert_eq!((moon.sign, moon.nakshatra, moon.pada), (Sign::Gemini, 6, 2));
        assert!((moon.deg_in_sign - 11.0).abs() < 1e-9);
        // Satır 3: Ay 71 + 39,6 = 110,6° = Yengeç 20,6°; 110,6 / 13,3333 = 8,3 → 9. nakshatra
        // (Ashlesha), içinde 110,6 − 106,667 = 3,933°, 3,933 / 3,3333 = 1,18 → 2. pada.
        let moon = cell(&t.rows[3], Graha::Moon);
        assert_eq!((moon.sign, moon.nakshatra, moon.pada), (Sign::Cancer, 9, 2));
        assert!((moon.deg_in_sign - 20.6).abs() < 1e-9);
        // Retro: yalnızca hızı eksi olanlar (Jüpiter ve ortalama düğümler).
        let retro: Vec<Graha> = r0
            .cells
            .iter()
            .filter(|c| c.retrograde)
            .map(|c| c.graha)
            .collect();
        assert_eq!(retro, [Graha::Jupiter, Graha::Rahu, Graha::Ketu]);
        // Ketu, Rahu'nun tam karşısındadır.
        let (rahu, ketu) = (cell(r0, Graha::Rahu), cell(r0, Graha::Ketu));
        assert!((ketu.lon - normalize(rahu.lon + 180.0)).abs() < 1e-9);
    }

    #[test]
    fn rows_agree_with_the_transit_snapshot_of_the_same_moment() {
        // İkinci yazım: v1'in transit hesabı aynı anda aynı konumu verir, iki zodyak kipinde.
        for zodiac in [
            ZodiacMode::Sidereal,
            ZodiacMode::TropicalRasiSiderealNakshatra,
        ] {
            let s = CalcSettings {
                zodiac,
                ..CalcSettings::default()
            };
            let bhavas = natal_bhavas(&s);
            for row in table(&s, 31).rows {
                let snap =
                    compute_transits(&mut FakeEph::default(), row.jd_ut, &s, &bhavas).unwrap();
                for (c, g) in row.cells.iter().zip(&snap.grahas) {
                    let ctx = format!("{:?} {zodiac:?} {}", c.graha, row.jd_ut);
                    assert_eq!(c.graha, g.graha, "{ctx}");
                    assert_eq!(
                        (c.lon, c.sign, c.deg_in_sign),
                        (g.lon, g.sign, g.deg_in_sign)
                    );
                    assert_eq!((c.nakshatra, c.pada), (g.nakshatra, g.pada), "{ctx}");
                    assert_eq!(c.retrograde, g.retrograde, "{ctx}");
                }
            }
        }
    }

    #[test]
    fn the_tropical_rasi_mode_reads_signs_from_the_tropical_longitude() {
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        // Güneş tropikal 10° = Koç 10°; sidereal 346°: 346 / 13,3333 = 25,95 → 26. nakshatra
        // (Uttara Bhadrapada), içinde 346 − 333,333 = 12,667°, 12,667 / 3,3333 = 3,8 → 4. pada.
        let sun = table(&tropical, 1).rows[0].cells[0].clone();
        assert_eq!((sun.sign, sun.nakshatra, sun.pada), (Sign::Aries, 26, 4));
        assert!((sun.lon - 10.0).abs() < 1e-9);
        // Sidereal kipte aynı an: Balık 16°, aynı nakshatra.
        let sun = table(&CalcSettings::default(), 1).rows[0].cells[0].clone();
        assert_eq!((sun.sign, sun.nakshatra, sun.pada), (Sign::Pisces, 26, 4));
    }

    #[test]
    fn the_ingress_mark_names_the_sign_entered_before_the_next_moment() {
        let t = table(&CalcSettings::default(), 31);
        // Güneş: 346 + 0,98 g = 360 → g = 14,29: 14. satırla 15. an arasında Koç'a girer.
        for (i, row) in t.rows.iter().enumerate() {
            let expected = (i == 14).then_some(Sign::Aries);
            assert_eq!(cell(row, Graha::Sun).ingress, expected, "row {i}");
        }
        // Ay: kuraldan, satır satır. g. günde boylam 71 + 13,2 g; burç floor(boylam / 30) mod 12.
        // İlk girişler: 90° g = 1,44 (satır 1, Yengeç); 120° g = 3,71 (satır 3, Aslan);
        // 150° g = 5,98 (satır 5, Başak); 180° g = 8,26 (satır 8, Terazi).
        let sign_at = |day: usize| Sign::of(normalize(71.0 + 13.2 * day as f64));
        let mut marked = Vec::new();
        for (i, row) in t.rows.iter().enumerate() {
            let expected = (sign_at(i) != sign_at(i + 1)).then_some(sign_at(i + 1));
            assert_eq!(cell(row, Graha::Moon).ingress, expected, "row {i}");
            if expected.is_some() {
                marked.push(i);
            }
        }
        assert_eq!(&marked[..4], [1, 3, 5, 8]);
        assert_eq!(cell(&t.rows[1], Graha::Moon).ingress, Some(Sign::Cancer));
        // Son satırın işareti kapatan andan okunur: 31 satır, 32 an.
        assert_eq!(t.rows.len(), 31);
        // Retro graha geriye girer: Jüpiter 226° − 0,1 g; 210°'ye 160 günde varır (bu ayda yok).
        assert!(t
            .rows
            .iter()
            .all(|r| cell(r, Graha::Jupiter).ingress.is_none()));
    }

    /// Mars'ın hızı `turn` anından sonra eksi olan sahte gök: dönüş işaretinin el örneği.
    struct Turning {
        inner: FakeEph,
        turn: f64,
    }

    impl Ephemeris for Turning {
        fn body(
            &mut self,
            jd_ut: f64,
            body: Body,
            zodiac: Zodiac,
        ) -> Result<BodyPosition, EphError> {
            let mut p = self.inner.body(jd_ut, body, zodiac)?;
            if body == Body::Mars && jd_ut > self.turn {
                p.lon_speed = -p.lon_speed.abs();
            }
            if body == Body::Jupiter && jd_ut > self.turn + 3.0 {
                p.lon_speed = p.lon_speed.abs();
            }
            Ok(p)
        }

        fn ayanamsa(&mut self, jd_ut: f64, mode: SiderealMode) -> Result<Ayanamsa, EphError> {
            self.inner.ayanamsa(jd_ut, mode)
        }

        fn houses(
            &mut self,
            jd_ut: f64,
            geo_lat: f64,
            geo_lon: f64,
            system: HouseSystem,
            zodiac: Zodiac,
        ) -> Result<Houses, EphError> {
            self.inner.houses(jd_ut, geo_lat, geo_lon, system, zodiac)
        }

        fn sun_rise_set(
            &mut self,
            after_jd_ut: f64,
            event: RiseSet,
            geo_lat: f64,
            geo_lon: f64,
            disc: SunDisc,
        ) -> Result<Option<f64>, EphError> {
            self.inner
                .sun_rise_set(after_jd_ut, event, geo_lat, geo_lon, disc)
        }

        fn swe_version(&self) -> String {
            self.inner.swe_version()
        }
    }

    #[test]
    fn the_station_mark_names_the_direction_turned_to_before_the_next_moment() {
        // Mars 4,5. günde retroya döner: 4. satır (4. gün) düz, 5. an retro → 4. satır
        // "retroya döner". Jüpiter 7,5. günde düze döner: 7. satır retro, 8. an düz → 7. satır
        // "düze döner". Düğümlerin yönü işaretlenmez.
        let mut eph = Turning {
            inner: FakeEph::default(),
            turn: JD + 4.5,
        };
        let t = ephemeris_table(&mut eph, &CalcSettings::default(), &days(12)).unwrap();
        for (i, row) in t.rows.iter().enumerate() {
            let mars = cell(row, Graha::Mars);
            assert_eq!(mars.retrograde, i >= 5, "row {i}");
            assert_eq!(
                mars.station,
                (i == 4).then_some(Motion::Retrograde),
                "row {i}"
            );
            let jupiter = cell(row, Graha::Jupiter);
            assert_eq!(jupiter.retrograde, i <= 7, "row {i}");
            assert_eq!(
                jupiter.station,
                (i == 7).then_some(Motion::Direct),
                "row {i}"
            );
            for graha in [Graha::Sun, Graha::Moon, Graha::Rahu, Graha::Ketu] {
                assert_eq!(cell(row, graha).station, None, "row {i} {graha:?}");
            }
        }
    }

    #[test]
    fn the_fallback_ephemeris_is_reported() {
        let mut eph = FakeEph {
            precision: Precision::Moshier,
            ..FakeEph::default()
        };
        let t = ephemeris_table(&mut eph, &CalcSettings::default(), &days(3)).unwrap();
        assert_eq!(t.warnings, [ChartWarning::LowPrecision]);
    }

    #[test]
    fn bad_moments_name_the_field() {
        let s = CalcSettings::default();
        let mut eph = FakeEph::default();
        let bad = Err(CoreError::InvalidTransitInput("moments"));
        let cases: Vec<Vec<f64>> = vec![
            vec![],
            vec![JD],
            days(EPHEMERIS_MAX_ROWS + 1),
            vec![JD, JD],
            vec![JD + 1.0, JD],
            vec![JD, JD + 1.0, JD + 0.5],
            vec![JD, f64::NAN],
            vec![JD, f64::INFINITY],
            vec![1.0, 2.0],
            vec![JD, JD + EPHEMERIS_MAX_STEP_DAYS + 0.01],
        ];
        for moments in cases {
            assert_eq!(ephemeris_table(&mut eph, &s, &moments), bad, "{moments:?}");
        }
        // En uzun tablo: 31 satır, 32 an.
        assert_eq!(table(&s, EPHEMERIS_MAX_ROWS).rows.len(), EPHEMERIS_MAX_ROWS);
        let old = CalcSettings {
            schema_version: 1,
            ..CalcSettings::default()
        };
        assert_eq!(
            ephemeris_table(&mut eph, &old, &days(2)),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 1,
                supported: 2
            })
        );
    }
}
