//! Toplayıcılar (K3): temel haritayı hesaplar, konumlara indirger ve saf fonksiyonlara verir.
//! Ayar, an ve konum denetimi `compute_chart`'ınkidir. Üç rapor da lagna istemez: doğum saati
//! bilinmiyorsa (`input.time_known` yanlış) rapor yine döner ve `approximate` işaretini taşır.

use ephemeris::Ephemeris;

use crate::chart::compute_chart_for;
use crate::graha::Graha;
use crate::positions::{compute_positions, ChartInput, Positions};
use crate::settings::JD_UT_RANGE;
use crate::transit::compute_transits;
use crate::CoreError;

use super::dagdha::{panchang_extras, PanchangExtras};
use super::navatara::{navatara, Navatara};
use super::vimshopaka::{vimshopaka, VimshopakaReport};

/// `compute_panchang_extras` komutunun hesabı.
pub fn compute_panchang_extras<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<PanchangExtras, CoreError> {
    let positions = compute_positions(eph, input)?;
    Ok(panchang_extras(&positions, !input.time_known))
}

/// `compute_navatara` komutunun hesabı. `transit_jd_ut` verilirse o anın dokuz grahası haritanın
/// ayarlarıyla hesaplanır ve taraları rapora eklenir. Geçiş anı, ephemeris sorulmadan önce
/// denetlenir; hatası yalnızca alan adını taşır.
pub fn compute_navatara<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
    transit_jd_ut: Option<f64>,
) -> Result<Navatara, CoreError> {
    if transit_jd_ut.is_some_and(|jd| !JD_UT_RANGE.contains(&jd)) {
        return Err(CoreError::InvalidTransitInput("transit_jd_ut"));
    }
    let chart = compute_chart_for(eph, input)?;
    let positions = Positions::from(&chart);
    let transit: Option<(f64, Vec<(Graha, f64)>)> = match transit_jd_ut {
        Some(jd) => {
            let snapshot = compute_transits(eph, jd, &input.settings, &chart.bhavas)?;
            let lons = snapshot
                .grahas
                .iter()
                .map(|g| (g.graha, g.sidereal_lon))
                .collect();
            Some((jd, lons))
        }
        None => None,
    };
    Ok(navatara(
        &positions,
        transit.as_ref().map(|(jd, lons)| (*jd, lons.as_slice())),
        !input.time_known,
    ))
}

/// `compute_vimshopaka` komutunun hesabı.
pub fn compute_vimshopaka<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<VimshopakaReport, CoreError> {
    let positions = compute_positions(eph, input)?;
    Ok(vimshopaka(&positions, !input.time_known))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::{CalcSettings, ZodiacMode};
    use crate::zodiac::Sign;
    use Graha::*;

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

    // FakeEph (ayanamsa 24°): sidereal Güneş 346°, Ay 71°, Mars 176°, Merkür 356°, Jüpiter 226°,
    // Venüs 16°, Satürn 276°, Rahu 96° (ortalama düğüm), Ketu 276°.

    #[test]
    fn panchang_extras_of_the_fake_sky() {
        // `yogi.rs`'in B örneği: nokta 150°20′, Uttara Phalguni (12) 2. pada, yogi Güneş; Başak
        // 0°20′, eş yogi Merkür; avayogi nakshatrası Anuradha (17), avayogi Satürn. Uzanım
        // 71° − 346° = 85° → 8. tithi: İkizler ve Başak.
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        let got = compute_panchang_extras(&mut eph, &input(s.clone(), true)).unwrap();
        assert!((got.yogi.point_lon - (150.0 + 20.0 / 60.0)).abs() < 1e-9);
        assert_eq!((got.yogi.nakshatra, got.yogi.pada), (12, 2));
        assert_eq!((got.yogi.yogi, got.yogi.co_yogi), (Sun, Mercury));
        assert_eq!(got.yogi.sign, Sign::Virgo);
        assert_eq!((got.yogi.avayogi_nakshatra, got.yogi.avayogi), (17, Saturn));
        assert_eq!(got.tithi, 8);
        assert_eq!(got.dagdha_rasis, [Sign::Gemini, Sign::Virgo]);
        assert!(!got.approximate);
        // Doğum saati bilinmiyor: aynı değerler, yaklaşık işaretiyle.
        let unknown = compute_panchang_extras(&mut eph, &input(s.clone(), false)).unwrap();
        assert!(unknown.approximate);
        assert_eq!((&unknown.yogi, unknown.tithi), (&got.yogi, got.tithi));
        // Tropikal-rasi kipi: nokta, nakshatra ve lordlar değişmez; burç rasi çerçevesindedir
        // (150°20′ + 24° = 174°20′, yine Başak; burç içi derece 24°20′ olur).
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..s
        };
        let t = compute_panchang_extras(&mut eph, &input(tropical, true)).unwrap();
        assert!((t.yogi.point_lon - got.yogi.point_lon).abs() < 1e-9);
        assert_eq!((t.yogi.nakshatra, t.yogi.yogi), (12, Sun));
        assert_eq!((t.yogi.sign, t.yogi.co_yogi), (Sign::Virgo, Mercury));
        assert!((t.yogi.deg_in_sign - (24.0 + 20.0 / 60.0)).abs() < 1e-9);
        assert_eq!((t.tithi, &t.dagdha_rasis), (8, &got.dagdha_rasis));
    }

    #[test]
    fn navatara_of_the_fake_sky_with_and_without_a_transit_moment() {
        // Ay 71°: Ardra (6: 66°40′–80°). Grahalar: Güneş 346° Uttara Bhadrapada (26); Mars 176°
        // Chitra (14); Merkür 356° Revati (27); Jüpiter 226° Anuradha (17); Venüs 16° Bharani
        // (2); Satürn ve Ketu 276° Uttara Ashadha (21); Rahu 96° Pushya (8).
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        let natal = compute_navatara(&mut eph, &input(s.clone(), true), None).unwrap();
        assert_eq!(natal.janma_nakshatra, 6);
        assert_eq!((natal.transit.as_ref(), natal.transit_jd_ut), (None, None));
        let at = |n: u8| natal.rows.iter().find(|r| r.nakshatra == n).expect("row");
        assert_eq!(at(6).grahas, [Moon]);
        // Ardra'dan Pushya'ya sayım 3 → Vipat, 1. tur.
        assert_eq!(
            (at(8).grahas.as_slice(), at(8).tara.number, at(8).cycle),
            (&[Rahu][..], 3, 1)
        );
        // Chitra'ya sayım 9 → Parama Mitra, 1. tur.
        assert_eq!(
            (at(14).grahas.as_slice(), at(14).tara.number, at(14).cycle),
            (&[Mars][..], 9, 1)
        );
        // Anuradha'ya sayım 12 → 3. tara, 2. tur.
        assert_eq!(
            (at(17).grahas.as_slice(), at(17).tara.number, at(17).cycle),
            (&[Jupiter][..], 3, 2)
        );
        // Uttara Ashadha'ya sayım 16 → 7. tara, 2. tur; iki graha.
        assert_eq!(
            (at(21).grahas.as_slice(), at(21).tara.number),
            (&[Saturn, Ketu][..], 7)
        );
        // Uttara Bhadrapada'ya sayım 21 → 3. tara, 3. tur; Revati'ye 22 → 4. tara; Bharani'ye
        // 24 → 6. tara.
        assert_eq!(
            (at(26).grahas.as_slice(), at(26).tara.number, at(26).cycle),
            (&[Sun][..], 3, 3)
        );
        assert_eq!(
            (at(27).grahas.as_slice(), at(27).tara.number),
            (&[Mercury][..], 4)
        );
        assert_eq!(
            (at(2).grahas.as_slice(), at(2).tara.number),
            (&[Venus][..], 6)
        );

        // On gün sonra: Ay 71° + 132° = 203° → Vishakha (16), 1. pada (200°–203°20′): sayım 11 →
        // Sampat (2), 2. tur. Güneş 346° + 9,8° = 355,8° → Revati (27), 3. pada (353°20′–
        // 356°40′): sayım 22 → Kshema (4), 3. tur.
        let with = compute_navatara(&mut eph, &input(s.clone(), false), Some(JD + 10.0)).unwrap();
        assert!(with.approximate);
        assert_eq!(with.transit_jd_ut, Some(JD + 10.0));
        assert_eq!(with.rows, natal.rows);
        let transit = with.transit.expect("transit");
        let order: Vec<Graha> = transit.iter().map(|t| t.graha).collect();
        assert_eq!(order, Graha::ALL);
        let moon = &transit[Moon as usize];
        assert_eq!(
            (moon.nakshatra, moon.pada, moon.tara.number, moon.cycle),
            (16, 1, 2, 2)
        );
        assert!(moon.tara.favourable);
        let sun = &transit[Sun as usize];
        assert_eq!(
            (sun.nakshatra, sun.pada, sun.tara.number, sun.cycle),
            (27, 3, 4, 3)
        );
    }

    #[test]
    fn vimshopaka_collector_scores_the_chart_positions() {
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        let positions = compute_positions(&mut eph, &input(s.clone(), true)).unwrap();
        let known = compute_vimshopaka(&mut eph, &input(s.clone(), true)).unwrap();
        assert_eq!(known, vimshopaka(&positions, false));
        // Güneş sidereal 346° (Balık 16°): D1'de Balık'ın lordu Jüpiter Akrep'tedir (226°),
        // Balık'tan 9. burç → geçici düşman; doğal dost: nötr → 10.
        assert_eq!(known.grahas[0].vargas[0].sign, Sign::Pisces);
        assert_eq!(known.grahas[0].vargas[0].points, 10.0);
        // Doğum saati bilinmiyor: rapor yine döner (lagna gerekmez), yaklaşık işaretiyle.
        let unknown = compute_vimshopaka(&mut eph, &input(s, false)).unwrap();
        assert!(unknown.approximate);
        assert_eq!(unknown.grahas, known.grahas);
    }

    #[test]
    fn collectors_check_the_input_before_and_through_the_chart() {
        let mut eph = FakeEph::default();
        let mut bad = input(CalcSettings::default(), true);
        bad.lat = 91.0;
        let location = CoreError::InvalidLocation {
            lat: 91.0,
            lon: 29.0,
        };
        assert_eq!(
            compute_panchang_extras(&mut eph, &bad),
            Err(location.clone())
        );
        assert_eq!(compute_vimshopaka(&mut eph, &bad), Err(location.clone()));
        assert_eq!(compute_navatara(&mut eph, &bad, None), Err(location));
        // Geçiş anı haritadan önce denetlenir ve hata yalnızca alan adını taşır.
        for jd in [f64::NAN, f64::INFINITY, 0.0, 1e9] {
            assert_eq!(
                compute_navatara(&mut eph, &bad, Some(jd)),
                Err(CoreError::InvalidTransitInput("transit_jd_ut"))
            );
        }
        // Sayı prashnasında grahalar anın grahalarıdır: üç rapor da sayısız haritayla aynıdır.
        let plain = input(CalcSettings::default(), true);
        let mut numbered = plain.clone();
        numbered.lagna_number = Some(30);
        assert_eq!(
            compute_panchang_extras(&mut eph, &numbered),
            compute_panchang_extras(&mut eph, &plain)
        );
        assert_eq!(
            compute_navatara(&mut eph, &numbered, None),
            compute_navatara(&mut eph, &plain, None)
        );
        assert_eq!(
            compute_vimshopaka(&mut eph, &numbered),
            compute_vimshopaka(&mut eph, &plain)
        );
    }
}
