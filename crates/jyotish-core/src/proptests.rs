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

    use crate::ashtakavarga::{compute_ashtakavarga, SAV_TOTAL};
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

    /// Gerçekçi boylam hızları (°/gün), `PLANETS` sırasında. Panchang bitiş araması Ay'ın
    /// Güneş'ten hızlı olmasını ister; eşit hızda harita `PanchangSearch` döner.
    const SPEEDS: [f64; 7] = [0.98, 13.2, 0.6, 1.2, -0.1, 1.1, 0.05];

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
            for ((body, lon), speed) in PLANETS.iter().zip(lons).zip(SPEEDS) {
                eph = eph.with(*body, lon, speed);
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

        #[test]
        fn chart_ashtakavarga_uses_rasi_signs(
            lons in proptest::array::uniform7(0.0f64..360.0),
            asc in 0.0f64..360.0,
            tropical in any::<bool>(),
        ) {
            let mut eph = FakeEph { asc, ..FakeEph::default() };
            for ((body, lon), speed) in PLANETS.iter().zip(lons).zip(SPEEDS) {
                eph = eph.with(*body, lon, speed);
            }
            let mut s = CalcSettings::default();
            if tropical {
                s.zodiac = ZodiacMode::TropicalRasiSiderealNakshatra;
            }
            let c = compute_chart(&mut eph, 2_451_545.0, 0.0, 0.0, &s).unwrap();
            let signs = std::array::from_fn(|i| c.grahas[i].sign);
            prop_assert_eq!(&c.ashtakavarga, &compute_ashtakavarga(signs, c.lagna.sign));
            prop_assert_eq!(c.ashtakavarga.sav.iter().map(|&v| u16::from(v)).sum::<u16>(), SAV_TOTAL);
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

/// İleri modüllerin ortak temeli (Alt proje 3.1): doğal sınıf, lordluk, Kuja dosha.
mod foundation_properties {
    use proptest::prelude::*;

    use crate::graha::Graha;
    use crate::lordship::{house_lord, house_sign, houses_owned};
    use crate::nature::{kuja_dosha, moon_is_waxing, nature, Nature, KUJA_DOSHA_HOUSES};
    use crate::positions::Positions;
    use crate::zodiac::Sign;

    pub(super) fn positions() -> impl Strategy<Value = Positions> {
        (
            proptest::option::of(0.0f64..360.0),
            proptest::array::uniform9(0.0f64..360.0),
        )
            .prop_map(|(lagna, lons)| Positions::synthetic(lagna, lons))
    }

    proptest! {
        #[test]
        fn natural_class_rules_hold(p in positions()) {
            for g in [Graha::Jupiter, Graha::Venus] {
                prop_assert_eq!(nature(g, &p), Nature::Benefic);
            }
            for g in [Graha::Sun, Graha::Mars, Graha::Saturn, Graha::Rahu, Graha::Ketu] {
                prop_assert_eq!(nature(g, &p), Nature::Malefic);
            }
            let moon = if moon_is_waxing(&p) { Nature::Benefic } else { Nature::Malefic };
            prop_assert_eq!(nature(Graha::Moon, &p), moon);
            // Merkür: aynı burçta kötücül varken iyicil yoksa kötücül, değilse iyicil.
            let sign = p.sign_of(Graha::Mercury);
            let others: Vec<Nature> = p
                .occupants(sign)
                .into_iter()
                .filter(|&g| g != Graha::Mercury)
                .map(|g| nature(g, &p))
                .collect();
            let want = if others.contains(&Nature::Malefic) && !others.contains(&Nature::Benefic) {
                Nature::Malefic
            } else {
                Nature::Benefic
            };
            prop_assert_eq!(nature(Graha::Mercury, &p), want);
        }

        #[test]
        fn houses_and_kuja_dosha_follow_the_sign_count(p in positions()) {
            let k = kuja_dosha(&p);
            prop_assert_eq!(k.from_lagna.is_some(), p.lagna_sign.is_some());
            let mars_from = |s: Sign| KUJA_DOSHA_HOUSES.contains(&p.house_from(Graha::Mars, s));
            prop_assert_eq!(k.from_moon, mars_from(p.sign_of(Graha::Moon)));
            prop_assert_eq!(k.from_venus, mars_from(p.sign_of(Graha::Venus)));
            if let Some(lagna) = p.lagna_sign {
                prop_assert_eq!(k.from_lagna, Some(mars_from(lagna)));
                for g in Graha::ALL {
                    let h = p.house_of(g).unwrap();
                    prop_assert!((1..=12).contains(&h));
                    prop_assert_eq!(house_sign(lagna, h), p.sign_of(g));
                }
                for h in 1..=12u8 {
                    prop_assert!(houses_owned(lagna, house_lord(lagna, h)).contains(&h));
                }
            } else {
                for g in Graha::ALL {
                    prop_assert_eq!(p.house_of(g), None);
                }
            }
        }
    }
}

/// Graha bakışları (Alt proje 3.1).
mod drishti_properties {
    use proptest::prelude::*;

    use super::foundation_properties::positions;
    use crate::drishti::{
        drishti_table, graha_aspect_signs, sphuta_drishti, ASPECTING_GRAHAS, FULL_DRISHTI,
    };
    use crate::graha::Graha;

    proptest! {
        #[test]
        fn sphuta_drishti_is_within_zero_and_sixty(
            g in 0usize..9,
            from in -720.0f64..720.0,
            to in -720.0f64..720.0,
        ) {
            let v = sphuta_drishti(Graha::ALL[g], from, to);
            prop_assert!(v.is_finite() && (0.0..=FULL_DRISHTI).contains(&v), "{v}");
        }

        #[test]
        fn sphuta_drishti_depends_only_on_the_angle(
            g in 0usize..7,
            from in 0.0f64..360.0,
            angle in 0.0f64..360.0,
            shift in -360.0f64..360.0,
        ) {
            // Özel bakış aralıklarının uçlarında değer sıçrar; yuvarlama hatasıyla ucun öbür
            // yanına düşmemek için 30°'nin katlarından uzak durulur.
            prop_assume!((angle / 30.0 - (angle / 30.0).round()).abs() > 1e-6);
            let graha = ASPECTING_GRAHAS[g];
            let a = sphuta_drishti(graha, from, from + angle);
            let b = sphuta_drishti(graha, from + shift, from + shift + angle);
            prop_assert!((a - b).abs() < 1e-6, "{a} vs {b}");
        }

        #[test]
        fn drishti_table_is_consistent_with_the_positions(p in positions()) {
            let t = drishti_table(&p);
            prop_assert_eq!(t.sign_aspects.len(), 13);
            for a in &t.sign_aspects {
                prop_assert!(!a.graha.is_node());
                prop_assert!(graha_aspect_signs(a.graha, p.sign_of(a.graha)).contains(&a.target));
                prop_assert_eq!(&a.occupants, &p.occupants(a.target));
            }
            for (i, row) in t.sphuta.iter().enumerate() {
                for (j, v) in row.iter().enumerate() {
                    prop_assert!((0.0..=FULL_DRISHTI).contains(v));
                    if Graha::ALL[j] == ASPECTING_GRAHAS[i] {
                        prop_assert_eq!(*v, 0.0);
                    }
                }
            }
            // Lagna bakış tablosunu etkilemez.
            prop_assert_eq!(drishti_table(&p.clone().without_lagna()), t);
        }
    }
}

/// Jaimini çekirdeği (Alt proje 3.2).
mod jaimini_properties {
    use proptest::prelude::*;

    use super::foundation_properties::positions;
    use crate::graha::Graha;
    use crate::jaimini::{
        arudha_padas, chara_karakas, co_lord_choice, co_lords, graha_rashi_drishti, jaimini_report,
        karaka_roles, karakamsha, navamsa_sign, rashi_drishti_signs, ArgalaStatus, JAIMINI_METHOD,
        KARAKA_TIE_DEG,
    };
    use crate::lordship::house_sign;
    use crate::settings::{AdvancedSettings, KarakaScheme};
    use crate::varga::is_vargottama;
    use crate::zodiac::Sign;

    fn scheme() -> impl Strategy<Value = KarakaScheme> {
        prop_oneof![Just(KarakaScheme::Eight), Just(KarakaScheme::Seven)]
    }

    proptest! {
        #[test]
        fn karaka_order_is_a_permutation_of_the_scheme_grahas(p in positions(), s in scheme()) {
            let k = chara_karakas(&p, s);
            let roles = karaka_roles(s);
            prop_assert_eq!(k.len(), roles.len());
            let mut grahas: Vec<Graha> = k.iter().map(|c| c.graha).collect();
            grahas.sort();
            prop_assert_eq!(&grahas[..], &Graha::ALL[..roles.len()]);
            prop_assert!(k.iter().all(|c| c.graha != Graha::Ketu));
            for (i, c) in k.iter().enumerate() {
                prop_assert_eq!(c.role, roles[i]);
                prop_assert!((0.0..=30.0).contains(&c.effective_deg));
                let deg = p.point(c.graha).deg_in_sign;
                let want = if c.graha == Graha::Rahu { 30.0 - deg } else { deg };
                prop_assert_eq!(c.effective_deg, want);
            }
            for w in k.windows(2) {
                prop_assert!(w[0].effective_deg >= w[1].effective_deg);
                let near = w[0].effective_deg - w[1].effective_deg <= KARAKA_TIE_DEG;
                if near {
                    prop_assert!(w[0].tie && w[1].tie);
                }
                if w[0].effective_deg == w[1].effective_deg {
                    prop_assert!(w[0].graha < w[1].graha);
                }
            }
            for (i, c) in k.iter().enumerate() {
                let near = |j: usize| (c.effective_deg - k[j].effective_deg).abs() <= KARAKA_TIE_DEG;
                let expected = (i > 0 && near(i - 1)) || (i + 1 < k.len() && near(i + 1));
                prop_assert_eq!(c.tie, expected);
            }
        }

        #[test]
        fn karakamsha_houses_follow_the_two_charts(p in positions(), g in 0usize..8) {
            let ak = Graha::ALL[g];
            let k = karakamsha(&p, ak);
            prop_assert_eq!(k.sign, navamsa_sign(p.point(ak)));
            prop_assert_eq!(k.houses_in_d9[g], 1);
            for (i, point) in p.grahas.iter().enumerate() {
                prop_assert_eq!(k.houses_in_d1[i], point.rasi_sign.house_from(k.sign));
                prop_assert_eq!(k.houses_in_d9[i], navamsa_sign(point).house_from(k.sign));
            }
            // Vargottama Atmakaraka: karakamsha kendi rasi burcudur.
            let lon = p.point(ak).rasi_sign.index() as f64 * 30.0 + p.point(ak).deg_in_sign;
            prop_assert_eq!(is_vargottama(lon), k.houses_in_d1[g] == 1);
        }

        #[test]
        fn graha_rashi_drishti_follows_the_sign_table(p in positions()) {
            let aspects = graha_rashi_drishti(&p);
            prop_assert_eq!(aspects.len(), 27);
            for (i, a) in aspects.iter().enumerate() {
                prop_assert_eq!(a.graha, Graha::ALL[i / 3]);
                prop_assert_eq!(a.target, rashi_drishti_signs(p.sign_of(a.graha))[i % 3]);
                prop_assert_eq!(&a.occupants, &p.occupants(a.target));
                // Karşılıklılık: hedef burçtaki her graha da bu grahanın burcuna bakar.
                for &other in &a.occupants {
                    prop_assert!(rashi_drishti_signs(p.sign_of(other)).contains(&p.sign_of(a.graha)));
                }
            }
        }

        #[test]
        fn co_lord_choice_is_one_of_the_two_lords(p in positions()) {
            for sign in [Sign::Scorpio, Sign::Aquarius] {
                let (planet, node) = co_lords(sign).expect("co-lorded");
                let c = co_lord_choice(sign, &p).expect("choice");
                prop_assert!(c.lord == planet || c.lord == node);
                prop_assert!((1..=6).contains(&c.step));
                let (ps, ns) = (p.sign_of(planet), p.sign_of(node));
                // 1. ölçüt: yalnızca biri burçtaysa dışarıdaki seçilir.
                if (ps == sign) != (ns == sign) {
                    prop_assert_eq!(c.step, 1);
                    prop_assert_eq!(c.lord, if ps == sign { node } else { planet });
                } else {
                    prop_assert!(c.step > 1);
                }
                // İkisi aynı burçtaysa hiçbir ölçüt ayırmaz; yücelme ve düşüş dışında.
                if ps == ns {
                    prop_assert!(c.step == 3 || c.step == 6);
                }
                if c.step == 6 {
                    prop_assert_eq!(c.lord, planet);
                }
            }
        }

        #[test]
        fn no_pada_falls_in_its_house_or_the_seventh(p in positions(), co in any::<bool>()) {
            match (p.lagna_sign, arudha_padas(&p, co)) {
                (None, padas) => prop_assert_eq!(padas, None),
                (Some(lagna), Some(padas)) => {
                    for (i, pada) in padas.iter().enumerate() {
                        let house = i as u8 + 1;
                        prop_assert_eq!(pada.house, house);
                        let from_house = pada.sign.house_from(house_sign(lagna, house));
                        prop_assert!(from_house != 1 && from_house != 7);
                        // Pada, evin lordunun burcundan lordun evden uzaklığı kadar ötededir;
                        // istisnada bundan 10. burçtur.
                        let lord_sign = p.sign_of(pada.lord);
                        let n = lord_sign.house_from(house_sign(lagna, house));
                        let computed = Sign::from_index(lord_sign.index() + usize::from(n) - 1);
                        let want = if pada.exception {
                            Sign::from_index(computed.index() + 9)
                        } else {
                            computed
                        };
                        prop_assert_eq!(pada.sign, want);
                        prop_assert_eq!(pada.exception, matches!(n, 1 | 4 | 7 | 10));
                    }
                }
                (Some(_), None) => prop_assert!(false, "padas with a lagna"),
            }
        }

        #[test]
        fn report_is_consistent(p in positions(), s in scheme(), co in any::<bool>()) {
            let advanced = AdvancedSettings { karaka_scheme: s, jaimini_co_lords: co };
            let r = jaimini_report(&p, &advanced);
            prop_assert_eq!(r.method.as_str(), JAIMINI_METHOD);
            prop_assert_eq!((r.scheme, r.co_lords), (s, co));
            prop_assert_eq!(r.karakamsha.atmakaraka, r.karakas[0].graha);
            prop_assert_eq!(r.arudhas.is_some(), p.lagna_sign.is_some());
            prop_assert!(r.warnings.is_empty());
            let first = p.lagna_sign.unwrap_or(Sign::Aries);
            for (i, row) in r.argala.iter().enumerate() {
                prop_assert_eq!(row.reference, Sign::from_index(first.index() + i));
                let mut seen = 0usize;
                for pair in &row.pairs {
                    // Aynı graha bir çiftin iki tarafında görünmez.
                    prop_assert!(pair.argala_grahas.iter().all(|g| !pair.virodha_grahas.contains(g)));
                    let (a, v) = (pair.argala_grahas.len(), pair.virodha_grahas.len());
                    let want = if a == 0 {
                        ArgalaStatus::None
                    } else if a > v {
                        ArgalaStatus::Stands
                    } else if a == v {
                        ArgalaStatus::Contested
                    } else {
                        ArgalaStatus::Obstructed
                    };
                    prop_assert_eq!(pair.status, want);
                    for &g in &pair.argala_grahas {
                        prop_assert_eq!(p.house_from(g, row.reference), pair.argala_house);
                    }
                    for &g in &pair.virodha_grahas {
                        prop_assert_eq!(p.house_from(g, row.reference), pair.virodha_house);
                    }
                    seen += a + v;
                }
                // Sekiz ev (2, 3, 4, 5, 9, 10, 11, 12) birer kez sayılır.
                let counted = Graha::ALL
                    .into_iter()
                    .filter(|&g| !matches!(p.house_from(g, row.reference), 1 | 6 | 7 | 8))
                    .count();
                prop_assert_eq!(seen, counted);
            }
            // Lagna yalnızca padaları ve argala satır sırasını etkiler.
            let q = jaimini_report(&p.clone().without_lagna(), &advanced);
            prop_assert_eq!(&q.karakas, &r.karakas);
            prop_assert_eq!(&q.karakamsha, &r.karakamsha);
            prop_assert_eq!(&q.graha_rashi_drishti, &r.graha_rashi_drishti);
            prop_assert_eq!(q.arudhas, None);
        }
    }
}

mod chara_dasha_properties {
    use proptest::prelude::*;

    use super::foundation_properties::positions;
    use crate::jaimini::{
        chara_dasha, chara_dasha_years, jaimini_lord, second_cycle_years, sign_dasha_children,
        sign_dasha_path_at, SignDashaPeriod, CHARA_DASHA_HORIZON_YEARS, CHARA_DASHA_METHOD,
    };
    use crate::zodiac::Sign;

    /// JD ~2.4e6 civarında f64 çözünürlüğü ~5e-10 gün; 1e-6 gün (~0.09 sn) bol paydır.
    const TOL_DAYS: f64 = 1e-6;

    fn birth() -> impl Strategy<Value = (f64, f64)> {
        (
            2_300_000.0f64..2_500_000.0,
            prop::sample::select(vec![360.0, 365.2422, 365.25]),
        )
    }

    /// Alt dönemler ebeveyni boşluksuz ve örtüşmesiz on iki eşit parçaya böler; sonuncusu
    /// ebeveynin burcudur ve hiçbir burç yinelenmez.
    fn check_partition(
        parent: &SignDashaPeriod,
        children: &[SignDashaPeriod],
    ) -> Result<(), TestCaseError> {
        prop_assert_eq!(children.len(), 12);
        prop_assert_eq!(children[0].start_jd_ut, parent.start_jd_ut);
        prop_assert_eq!(children[11].end_jd_ut, parent.end_jd_ut);
        for w in children.windows(2) {
            prop_assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
        }
        let span = parent.end_jd_ut - parent.start_jd_ut;
        let mut sum = 0.0;
        let mut seen = [false; 12];
        for c in children {
            prop_assert_eq!(c.level, parent.level + 1);
            prop_assert_eq!(c.cycle, parent.cycle);
            prop_assert_eq!(&c.signs[..parent.signs.len()], &parent.signs[..]);
            let d = c.end_jd_ut - c.start_jd_ut;
            prop_assert!((d - span / 12.0).abs() < TOL_DAYS, "{:?}: {}", c.signs, d);
            sum += d;
            seen[c.signs.last().expect("sign").index()] = true;
        }
        prop_assert!((sum - span).abs() < TOL_DAYS);
        prop_assert!(seen.iter().all(|s| *s));
        prop_assert_eq!(children[11].signs.last(), parent.signs.last());
        Ok(())
    }

    proptest! {
        #[test]
        fn no_dasha_without_a_lagna(p in positions(), co in any::<bool>(), (jd, year) in birth()) {
            let d = chara_dasha(&p, co, jd, year);
            prop_assert_eq!(d.is_some(), p.lagna_sign.is_some());
        }

        #[test]
        fn first_cycle_is_a_permutation_from_the_lagna(
            p in positions(),
            co in any::<bool>(),
            (jd, year) in birth(),
        ) {
            let Some(lagna) = p.lagna_sign else { return Ok(()); };
            let d = chara_dasha(&p, co, jd, year).expect("lagna");
            prop_assert_eq!(d.method.as_str(), CHARA_DASHA_METHOD);
            let dashas: Vec<&SignDashaPeriod> = d.periods.iter().filter(|q| q.level == 1).collect();
            let first: Vec<&SignDashaPeriod> =
                dashas.iter().copied().filter(|q| q.cycle == 1).collect();
            prop_assert_eq!(first.len(), 12);
            prop_assert_eq!(first[0].signs[0], lagna);
            prop_assert_eq!(first[0].start_jd_ut, jd);
            let mut seen = [false; 12];
            for (i, q) in first.iter().enumerate() {
                let sign = q.signs[0];
                seen[sign.index()] = true;
                // Sıra tek yönde birer burç ilerler.
                let step = if d.forward { i } else { 12 - i };
                prop_assert_eq!(sign, Sign::from_index(lagna.index() + step));
                // Her süre 1–12 yıldır ve burçtan lorduna sayımdan gelir.
                let years = d.years[i];
                prop_assert!((1..=12).contains(&years));
                let lord_sign = p.sign_of(jaimini_lord(sign, &p, co));
                prop_assert_eq!(years, chara_dasha_years(sign, lord_sign));
                prop_assert_eq!(years == 12, lord_sign == sign);
                let length = q.end_jd_ut - q.start_jd_ut;
                prop_assert!((length - f64::from(years) * year).abs() < TOL_DAYS);
            }
            prop_assert!(seen.iter().all(|s| *s));
            // İlk çevrim 120 yıla ulaşmaz: on iki dasha her zaman tamdır.
            let total: u16 = d.years.iter().map(|y| u16::from(*y)).sum();
            prop_assert!((12..CHARA_DASHA_HORIZON_YEARS).contains(&total), "{}", total);
        }

        #[test]
        fn dashas_are_contiguous_and_the_second_cycle_completes_twelve_years(
            p in positions(),
            co in any::<bool>(),
            (jd, year) in birth(),
        ) {
            let Some(d) = chara_dasha(&p, co, jd, year) else { return Ok(()); };
            let dashas: Vec<&SignDashaPeriod> = d.periods.iter().filter(|q| q.level == 1).collect();
            for w in dashas.windows(2) {
                prop_assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
                prop_assert!(w[0].cycle <= w[1].cycle);
            }
            let order: Vec<Sign> = dashas.iter().filter(|q| q.cycle == 1).map(|q| q.signs[0]).collect();
            let horizon = jd + f64::from(CHARA_DASHA_HORIZON_YEARS) * year;
            let mut last_index = None;
            for q in dashas.iter().filter(|q| q.cycle == 2) {
                let i = order.iter().position(|s| *s == q.signs[0]).expect("in the first cycle");
                // Aynı sıra; 0 yıllık burçlar atlanır.
                prop_assert!(last_index < Some(i));
                for skipped in &d.years[last_index.map_or(0, |l: usize| l + 1)..i] {
                    prop_assert_eq!(*skipped, 12);
                }
                last_index = Some(i);
                // İki çevrimde aynı burcun yılları toplamı 12'dir.
                let years = (q.end_jd_ut - q.start_jd_ut) / year;
                prop_assert!((years - f64::from(second_cycle_years(d.years[i]))).abs() < 1e-6);
                prop_assert!((years + f64::from(d.years[i]) - 12.0).abs() < 1e-6);
                // Başlangıcı 120. yıldan önce olan dasha üretilir.
                prop_assert!(q.start_jd_ut < horizon - TOL_DAYS);
            }
            // Üretim 120. yıla ulaşan ilk dashada durur.
            let end = dashas.last().expect("dashas").end_jd_ut;
            prop_assert!(end > horizon - TOL_DAYS);
        }

        #[test]
        fn sub_periods_partition_their_parent_at_both_levels(
            p in positions(),
            co in any::<bool>(),
            (jd, year) in birth(),
            pick in 0usize..24,
            sub in 0usize..12,
        ) {
            let Some(d) = chara_dasha(&p, co, jd, year) else { return Ok(()); };
            let dashas: Vec<&SignDashaPeriod> = d.periods.iter().filter(|q| q.level == 1).collect();
            let parent = dashas[pick % dashas.len()];
            let second = sign_dasha_children(&d, &parent.signs, parent.cycle).unwrap();
            check_partition(parent, &second)?;
            // İlk alt dönem dasha burcundan sonraki burçtur (haritanın yönünde).
            let dasha_sign = parent.signs[0];
            let step = if d.forward { 1 } else { 11 };
            prop_assert_eq!(second[0].signs[1], Sign::from_index(dasha_sign.index() + step));
            let child = &second[sub];
            let third = sign_dasha_children(&d, &child.signs, child.cycle).unwrap();
            check_partition(child, &third)?;
            prop_assert!(third.iter().all(|q| !q.has_children));
            prop_assert_eq!(third[0].signs[2], Sign::from_index(child.signs[1].index() + step));
        }

        #[test]
        fn path_at_names_the_periods_that_contain_the_moment(
            p in positions(),
            co in any::<bool>(),
            (jd, year) in birth(),
            fraction in 0.0f64..1.0,
        ) {
            let Some(d) = chara_dasha(&p, co, jd, year) else { return Ok(()); };
            let end = d.periods.iter().rfind(|q| q.level == 1).expect("dashas").end_jd_ut;
            let at = jd + (end - jd) * fraction;
            let path = sign_dasha_path_at(&d, at).unwrap().expect("inside the produced span");
            prop_assert_eq!(path.signs.len(), 3);
            for depth in 1..=3usize {
                let period = sign_dasha_children(&d, &path.signs[..depth - 1], path.cycle)
                    .unwrap()
                    .into_iter()
                    .find(|q| q.signs == path.signs[..depth])
                    .expect("on the path");
                prop_assert!(period.start_jd_ut <= at && at < period.end_jd_ut);
            }
            prop_assert_eq!(sign_dasha_path_at(&d, jd - 1.0).unwrap(), None);
            prop_assert_eq!(sign_dasha_path_at(&d, end).unwrap(), None);
        }
    }
}

mod shadbala_properties {
    use proptest::prelude::*;

    use super::foundation_properties::positions;
    use crate::graha::Graha;
    use crate::lordship::house_lord;
    use crate::nature::{nature, Nature};
    use crate::panchang::Vara;
    use crate::positions::Positions;
    use crate::shadbala::{
        ayana_bala, bhava_dig_bala, bhava_sign_kind, cheshta_from, day_night_bala, dig_bala,
        drik_bala, graha_war, hora_index, hora_lord, naisargika_bala, paksha_bala, required_bala,
        shadbala, uccha_bala, vara_lord, MeanLongitudes, ShadbalaAstro, GRAHA_WAR_ORB_DEG,
        SHADBALA_GRAHAS, SHADBALA_METHOD, WAR_GRAHAS,
    };
    use crate::zodiac::{angular_distance, Sign};

    const TOL: f64 = 1e-9;

    fn vara() -> impl Strategy<Value = Vara> {
        (0usize..7).prop_map(|i| Vara::ALL[i])
    }

    /// Tutarlı bir astronomik girdi: doğuş, gündüz uzunluğu, doğuşlar arası süre ve doğumun
    /// gün içindeki yeri.
    fn astro() -> impl Strategy<Value = ShadbalaAstro> {
        (
            (0.0f64..1.0, 0.05f64..0.95, 0.97f64..1.03, 0.0f64..0.999_999),
            0.0f64..360.0,
            proptest::array::uniform7(0.0f64..360.0),
            proptest::array::uniform6(0.0f64..360.0),
            (vara(), vara(), vara()),
        )
            .prop_map(
                |((rise, day, span, at), mc, tropical, mean, (weekday, year, month))| {
                    ShadbalaAstro {
                        jd_ut: rise + span * at,
                        sunrise_jd_ut: rise,
                        sunset_jd_ut: rise + span * day,
                        next_sunrise_jd_ut: rise + span,
                        mc_lon: mc,
                        tropical_lons: tropical,
                        mean: MeanLongitudes {
                            sun: mean[0],
                            mercury: mean[1],
                            venus: mean[2],
                            mars: mean[3],
                            jupiter: mean[4],
                            saturn: mean[5],
                        },
                        weekday,
                        year_entry_weekday: year,
                        month_entry_weekday: month,
                    }
                },
            )
    }

    fn graha7() -> impl Strategy<Value = Graha> {
        (0usize..7).prop_map(|i| SHADBALA_GRAHAS[i])
    }

    /// Tam bir graha `value` alır, diğerleri 0.
    fn one_lord(values: [f64; 7], value: f64) -> bool {
        values.iter().filter(|v| **v == value).count() == 1
            && values.iter().filter(|v| **v == 0.0).count() == 6
    }

    proptest! {
        #[test]
        fn uccha_of_opposite_points_adds_to_sixty(g in graha7(), lon in 0.0f64..360.0) {
            let (a, b) = (uccha_bala(g, lon), uccha_bala(g, lon + 180.0));
            prop_assert!((0.0..=60.0).contains(&a));
            prop_assert!((a + b - 60.0).abs() < TOL, "{} + {}", a, b);
        }

        #[test]
        fn dig_of_opposite_points_adds_to_sixty(
            g in graha7(),
            lon in 0.0f64..360.0,
            lagna in 0.0f64..360.0,
            mc in 0.0f64..360.0,
        ) {
            let (a, b) = (dig_bala(g, lon, lagna, mc), dig_bala(g, lon + 180.0, lagna, mc));
            prop_assert!((0.0..=60.0).contains(&a));
            prop_assert!((a + b - 60.0).abs() < TOL, "{} + {}", a, b);
        }

        #[test]
        fn day_and_night_strong_grahas_add_to_sixty(m in 0.0f64..=12.0) {
            for day in [Graha::Sun, Graha::Jupiter, Graha::Venus] {
                for night in [Graha::Moon, Graha::Mars, Graha::Saturn] {
                    let (a, b) = (day_night_bala(day, m), day_night_bala(night, m));
                    prop_assert!((0.0..=60.0).contains(&a) && (0.0..=60.0).contains(&b));
                    prop_assert!((a + b - 60.0).abs() < TOL);
                }
            }
            prop_assert_eq!(day_night_bala(Graha::Mercury, m), 60.0);
        }

        #[test]
        fn benefic_and_malefic_paksha_add_to_sixty(p in positions()) {
            // Jüpiter her zaman iyicil, Satürn her zaman kötücüldür.
            let (b, m) = (paksha_bala(Graha::Jupiter, &p), paksha_bala(Graha::Saturn, &p));
            prop_assert!((b + m - 60.0).abs() < TOL, "{} + {}", b, m);
            let moon = paksha_bala(Graha::Moon, &p);
            prop_assert!((0.0..=120.0).contains(&moon));
            prop_assert!((moon - 2.0 * b).abs() < TOL);
            for g in SHADBALA_GRAHAS {
                if g == Graha::Moon {
                    continue;
                }
                let want = match nature(g, &p) {
                    Nature::Benefic => b,
                    Nature::Malefic => m,
                };
                prop_assert!((paksha_bala(g, &p) - want).abs() < TOL, "{:?}", g);
            }
        }

        #[test]
        fn ayana_stays_in_range_and_mirrors_across_the_equator(lon in 0.0f64..360.0) {
            // Aynı boylamda kuzeyde güçlenen ile güneyde güçlenenin toplamı 60'tır.
            let (north, south) = (ayana_bala(Graha::Sun, lon), ayana_bala(Graha::Moon, lon));
            prop_assert!((north + south - 60.0).abs() < TOL);
            for g in SHADBALA_GRAHAS {
                let v = ayana_bala(g, lon);
                prop_assert!((0.6875 - TOL..=59.3125 + TOL).contains(&v), "{:?} {}", g, v);
            }
            prop_assert!(ayana_bala(Graha::Mercury, lon) >= 30.0 - TOL);
        }

        #[test]
        fn cheshta_stays_in_range_and_opposite_apexes_add_to_sixty(
            slow in 0.0f64..360.0,
            true_lon in 0.0f64..360.0,
            apex in 0.0f64..360.0,
        ) {
            let (a, b) = (
                cheshta_from(slow, true_lon, apex),
                cheshta_from(slow, true_lon, apex + 180.0),
            );
            prop_assert!((0.0..=60.0).contains(&a));
            prop_assert!((a + b - 60.0).abs() < 1e-6, "{} + {}", a, b);
        }

        #[test]
        fn the_twenty_fifth_hora_is_the_next_day_lord(i in 0usize..7, a in astro()) {
            let today = vara_lord(Vara::ALL[i]);
            let tomorrow = vara_lord(Vara::ALL[(i + 1) % 7]);
            prop_assert_eq!(hora_lord(today, 0), today);
            prop_assert_eq!(hora_lord(today, 24), tomorrow);
            prop_assert!(hora_index(&a) < 24);
        }

        #[test]
        fn war_is_mutual_and_only_within_the_orb(p in positions(), near in 0.0f64..2.0) {
            // Venüs'ü Mars'ın `near` derece ilerisine koy: savaş olasılığı yüksek olsun.
            let mut lons: [f64; 9] = std::array::from_fn(|i| p.grahas[i].sidereal_lon);
            lons[Graha::Venus as usize] = lons[Graha::Mars as usize] + near;
            let p = Positions::synthetic(p.lagna_lon, lons);
            for g in Graha::ALL {
                let Some(war) = graha_war(g, &p) else { continue; };
                prop_assert!(WAR_GRAHAS.contains(&g) && WAR_GRAHAS.contains(&war.with));
                prop_assert!(war.with != g);
                prop_assert!(war.winner == g || war.winner == war.with);
                let gap = angular_distance(
                    p.point(g).sidereal_lon,
                    p.point(war.with).sidereal_lon,
                );
                prop_assert!(gap <= GRAHA_WAR_ORB_DEG);
                // Karşı taraf da savaştadır (kendi en yakınıyla).
                prop_assert!(graha_war(war.with, &p).is_some());
            }
            for g in [Graha::Sun, Graha::Moon, Graha::Rahu, Graha::Ketu] {
                prop_assert_eq!(graha_war(g, &p), None);
            }
        }

        #[test]
        fn report_needs_a_lagna_and_every_item_is_in_its_range(p in positions(), a in astro()) {
            let report = shadbala(&p, &a);
            prop_assert_eq!(report.is_some(), p.lagna_sign.is_some());
            let Some(r) = report else { return Ok(()); };
            let lagna = p.lagna_sign.expect("lagna");
            prop_assert_eq!(r.method.as_str(), SHADBALA_METHOD);
            prop_assert!(r.warnings.is_empty());

            let mut ranks = [false; 7];
            for (row, g) in r.grahas.iter().zip(SHADBALA_GRAHAS) {
                prop_assert_eq!(row.graha, g);
                let s = &row.sthana;
                prop_assert!((0.0..=60.0).contains(&s.uccha));
                prop_assert!((14.0..=225.0).contains(&s.saptavargaja), "{}", s.saptavargaja);
                prop_assert!([0.0, 15.0, 30.0].contains(&s.ojayugma));
                prop_assert!([15.0, 30.0, 60.0].contains(&s.kendradi));
                prop_assert!([0.0, 15.0].contains(&s.drekkana));
                let sum = s.uccha + s.saptavargaja + s.ojayugma + s.kendradi + s.drekkana;
                prop_assert!((s.total - sum).abs() < TOL);
                prop_assert!((0.0..=60.0).contains(&row.dig));
                let t = &row.temporal;
                prop_assert!((0.0..=60.0).contains(&t.day_night));
                let paksha_max = if g == Graha::Moon { 120.0 } else { 60.0 };
                prop_assert!((0.0..=paksha_max).contains(&t.paksha));
                let ayana_max = if g == Graha::Sun { 120.0 } else { 60.0 };
                prop_assert!((0.0..=ayana_max).contains(&t.ayana));
                let sum = t.day_night + t.paksha + t.tribhaga + t.year_lord + t.month_lord
                    + t.weekday_lord + t.hora_lord + t.ayana;
                prop_assert!((t.total - sum).abs() < TOL);
                prop_assert!((0.0..=60.0).contains(&row.cheshta));
                prop_assert_eq!(row.naisargika, naisargika_bala(g));
                // Altı bakış: her biri ±15; Merkür ve Jüpiter'in tamamı en çok 60'ar.
                prop_assert!((-90.0..=210.0).contains(&row.drik), "{}", row.drik);
                prop_assert_eq!(row.drik, drik_bala(g, &p));
                let total = s.total + row.dig + t.total + row.cheshta + row.naisargika + row.drik;
                prop_assert!((row.total - total).abs() < TOL);
                prop_assert!((row.rupas * 60.0 - row.total).abs() < 1e-6);
                prop_assert_eq!(row.required, required_bala(g));
                prop_assert!((row.ratio * row.required - row.total).abs() < 1e-6);
                prop_assert_eq!(row.war, graha_war(g, &p));
                prop_assert!((1..=7).contains(&row.rank));
                ranks[usize::from(row.rank) - 1] = true;
            }
            prop_assert!(ranks.iter().all(|x| *x), "ranks are a permutation of 1..=7");
            for x in &r.grahas {
                for y in &r.grahas {
                    if x.total > y.total {
                        prop_assert!(x.rank < y.rank);
                    }
                }
            }

            // Tribhaga: Jüpiter her zaman 60; diğer altı grahadan tam biri 60, ötekiler 0.
            let pick = |f: fn(&crate::shadbala::TemporalBala) -> f64| -> [f64; 7] {
                std::array::from_fn(|i| f(&r.grahas[i].temporal))
            };
            let tribhaga = pick(|t| t.tribhaga);
            prop_assert_eq!(tribhaga[Graha::Jupiter as usize], 60.0);
            prop_assert!(r.time_lords.tribhaga != Graha::Jupiter);
            let others = tribhaga
                .iter()
                .enumerate()
                .filter(|(i, v)| *i != Graha::Jupiter as usize && **v == 60.0)
                .count();
            prop_assert_eq!(others, 1);
            prop_assert!(tribhaga.iter().all(|v| *v == 0.0 || *v == 60.0));
            // Yıl, ay, gün ve saat lordu: tam bir graha 15, 30, 45 ve 60 alır.
            prop_assert!(one_lord(pick(|t| t.year_lord), 15.0));
            prop_assert!(one_lord(pick(|t| t.month_lord), 30.0));
            prop_assert!(one_lord(pick(|t| t.weekday_lord), 45.0));
            prop_assert!(one_lord(pick(|t| t.hora_lord), 60.0));
            prop_assert_eq!(r.time_lords.weekday, vara_lord(a.weekday));
            prop_assert_eq!(r.time_lords.year, vara_lord(a.year_entry_weekday));
            prop_assert_eq!(r.time_lords.month, vara_lord(a.month_entry_weekday));

            // Bhavalar.
            let mut ranks = [false; 12];
            for (i, b) in r.bhavas.iter().enumerate() {
                let house = i as u8 + 1;
                prop_assert_eq!(b.house, house);
                prop_assert_eq!(b.lord, house_lord(lagna, house));
                prop_assert_eq!(b.lord_bala, r.grahas[b.lord as usize].total);
                prop_assert!([0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0].contains(&b.dig));
                // Yedi bakış: her biri ±15; Merkür ve Jüpiter'in tamamı en çok 60'ar.
                prop_assert!((-105.0..=225.0).contains(&b.drishti), "{}", b.drishti);
                prop_assert!((b.total - (b.lord_bala + b.dig + b.drishti)).abs() < TOL);
                prop_assert!((1..=12).contains(&b.rank));
                ranks[usize::from(b.rank) - 1] = true;
            }
            prop_assert!(ranks.iter().all(|x| *x), "bhava ranks are a permutation of 1..=12");
        }

        #[test]
        fn bhava_dig_of_opposite_houses_adds_to_sixty(
            sign in 0usize..12,
            deg in 0.0f64..30.0,
            house in 1u8..=6,
        ) {
            let kind = bhava_sign_kind(Sign::from_index(sign), deg);
            prop_assert_eq!(bhava_dig_bala(kind.powerless_house(), kind), 0.0);
            prop_assert_eq!(bhava_dig_bala(house, kind) + bhava_dig_bala(house + 6, kind), 60.0);
        }
    }
}

mod yoga_properties {
    use proptest::prelude::*;

    use super::foundation_properties::positions;
    use crate::graha::Graha;
    use crate::positions::Positions;
    use crate::yogas::{
        is_exchange, yoga_relations, yoga_rule, yogas, YogaGroup, YogaHit, YogaRelation,
        KENDRA_HOUSES, YOGA_CATALOG, YOGA_CATALOG_VERSION, YOGA_CONDITIONS, YOGA_GRAHAS,
    };

    fn ids(hits: &[YogaHit], prefix: &str) -> Vec<String> {
        hits.iter()
            .filter(|h| h.id.starts_with(prefix))
            .map(|h| h.id.clone())
            .collect()
    }

    fn has(hits: &[YogaHit], id: &str) -> bool {
        hits.iter().any(|h| h.id == id)
    }

    /// Kimliğin katalogdaki sırası.
    fn catalog_index(id: &str) -> usize {
        YOGA_CATALOG
            .iter()
            .position(|r| r.id == id)
            .unwrap_or_else(|| panic!("{id} is not in the catalog"))
    }

    proptest! {
        #[test]
        fn exactly_one_count_yoga_and_at_most_one_ashraya_yoga(p in positions()) {
            let report = yogas(&p);
            prop_assert_eq!(ids(&report.hits, "nabhasa_count.").len(), 1);
            prop_assert!(ids(&report.hits, "nabhasa_ashraya.").len() <= 1);
            // Tek burçta toplanan yedi graha (Gola) her zaman bir ashraya yogası da verir.
            if has(&report.hits, "nabhasa_count.gola") {
                prop_assert_eq!(ids(&report.hits, "nabhasa_ashraya.").len(), 1);
            }
        }

        #[test]
        fn mahapurusha_needs_a_kendra_and_a_strong_sign(p in positions()) {
            for h in yogas(&p).hits.iter().filter(|h| h.group == YogaGroup::Mahapurusha) {
                prop_assert_eq!(h.grahas.len(), 1);
                let house = p.house_of(h.grahas[0]);
                prop_assert!(house.is_some_and(|x| KENDRA_HOUSES.contains(&x)));
                prop_assert_eq!(&h.houses, &vec![house.unwrap()]);
                prop_assert!(!h.conditions.is_empty());
            }
        }

        #[test]
        fn flank_yogas_exclude_each_other_and_kemadruma(p in positions()) {
            let report = yogas(&p);
            let moon: Vec<String> = ["lunar.sunapha", "lunar.anapha", "lunar.durudhara"]
                .into_iter()
                .filter(|id| has(&report.hits, id))
                .map(str::to_owned)
                .collect();
            prop_assert!(moon.len() <= 1, "{:?}", moon);
            let sun = ["solar.vesi", "solar.vasi", "solar.ubhayachari"]
                .into_iter()
                .filter(|id| has(&report.hits, id))
                .count();
            prop_assert!(sun <= 1);
            if has(&report.hits, "lunar.kemadruma") {
                prop_assert!(moon.is_empty(), "{:?}", moon);
                // Jüpiter Ay'dan kendradaysa Kemadruma oluşamaz.
                prop_assert!(!has(&report.hits, "lunar.gajakesari"));
            }
        }

        #[test]
        fn hits_are_unique_ordered_and_well_formed(p in positions()) {
            let report = yogas(&p);
            prop_assert_eq!(report.catalog_version, YOGA_CATALOG_VERSION);
            prop_assert_eq!(
                (report.evaluated + report.skipped_time_unknown) as usize,
                YOGA_CATALOG.len()
            );
            let lagna = p.lagna_sign.is_some();
            prop_assert_eq!(report.skipped_time_unknown, if lagna { 0 } else { 17 });
            let mut seen: Vec<(&str, &[Graha])> = Vec::new();
            let mut last = 0;
            for h in &report.hits {
                // Her kimlik katalogdadır; kayıtlar katalog sırasındadır.
                let rule = yoga_rule(&h.id);
                prop_assert!(rule.is_some(), "{} is not in the catalog", h.id);
                let rule = rule.unwrap();
                prop_assert_eq!(h.group, rule.group);
                let index = catalog_index(&h.id);
                prop_assert!(index >= last);
                last = index;
                // Aynı kimlik ve aynı graha kümesi iki kez dönmez.
                let key = (h.id.as_str(), h.grahas.as_slice());
                prop_assert!(!seen.contains(&key), "{:?} twice", key);
                seen.push(key);
                // Grahalar ve evler sıralı ve yinelemesizdir; düğümler hiçbir kayıtta yoktur.
                prop_assert!(!h.grahas.is_empty());
                prop_assert!(h.grahas.windows(2).all(|w| w[0] < w[1]));
                prop_assert!(h.grahas.iter().all(|g| !g.is_node()));
                prop_assert!(h.houses.windows(2).all(|w| w[0] < w[1]));
                prop_assert!(h.houses.iter().all(|x| (1..=12).contains(x)));
                prop_assert!(h.conditions.iter().all(|c| YOGA_CONDITIONS.contains(&c.as_str())));
                // Lagnasız konumda lagna isteyen kural dönmez; her kayıt yaklaşıktır, ev taşımaz.
                prop_assert_eq!(h.approximate, !lagna);
                if !lagna {
                    prop_assert!(!rule.needs_lagna);
                    prop_assert!(h.houses.is_empty());
                    prop_assert!(!h.conditions.iter().any(|c| c == "from_lagna"));
                }
            }
        }

        #[test]
        fn relation_based_hits_carry_the_relations_of_their_pair(p in positions()) {
            let report = yogas(&p);
            for h in &report.hits {
                match h.group {
                    YogaGroup::Raja if h.id != "raja.yogakaraka" => {
                        prop_assert_eq!(h.grahas.len(), 2);
                        let relations = yoga_relations(h.grahas[0], h.grahas[1], &p);
                        prop_assert!(!relations.is_empty());
                        prop_assert_eq!(&h.relations, &relations);
                    }
                    YogaGroup::Dhana => {
                        prop_assert_eq!(&h.relations, &yoga_relations(h.grahas[0], h.grahas[1], &p));
                        prop_assert!(h.houses.iter().any(|x| [2, 11].contains(x)));
                    }
                    YogaGroup::Exchange => {
                        prop_assert!(is_exchange(h.grahas[0], h.grahas[1], &p));
                        prop_assert_eq!(&h.relations, &vec![YogaRelation::Exchange]);
                        prop_assert_eq!(h.houses.len(), 2);
                    }
                    _ => {}
                }
            }
            // Dharma-Karmadhipati kaydı varsa aynı çift kendra–trikona kaydı olarak da döner.
            for h in report.hits.iter().filter(|h| h.id == "raja.dharma_karmadhipati") {
                prop_assert!(report
                    .hits
                    .iter()
                    .any(|k| k.id == "raja.kendra_trikona" && k.grahas == h.grahas));
            }
            // Değişen her çift tam bir değişim sınıfında döner.
            if p.lagna_sign.is_some() {
                let exchanging = YOGA_GRAHAS
                    .into_iter()
                    .enumerate()
                    .flat_map(|(i, a)| YOGA_GRAHAS[i + 1..].iter().map(move |&b| (a, b)))
                    .filter(|&(a, b)| is_exchange(a, b, &p))
                    .count();
                let listed = report
                    .hits
                    .iter()
                    .filter(|h| h.group == YogaGroup::Exchange)
                    .count();
                prop_assert_eq!(listed, exchanging);
            }
        }

        #[test]
        fn dropping_the_lagna_keeps_the_lagna_free_rules(p in positions()) {
            // Lagnasız rapor, lagnalı rapordaki lagnadan bağımsız kayıtları aynı grahalarla taşır.
            let without = yogas(&p.clone().without_lagna());
            let with: Positions = p;
            for h in &yogas(&with).hits {
                let free = [YogaGroup::Solar, YogaGroup::NabhasaCount, YogaGroup::NabhasaAshraya]
                    .contains(&h.group)
                    || ["lunar.sunapha", "lunar.anapha", "lunar.durudhara", "lunar.gajakesari",
                        "lunar.chandra_mangala", "lunar.adhi"].contains(&h.id.as_str());
                if free {
                    prop_assert!(
                        without.hits.iter().any(|w| w.id == h.id && w.grahas == h.grahas),
                        "{} is missing without a lagna",
                        h.id
                    );
                }
            }
        }
    }
}

mod kuta_properties {
    use proptest::prelude::*;

    use super::foundation_properties::positions;
    use crate::graha::Graha;
    use crate::kuta::{
        kuta_class_ids, kuta_report, kuta_scores, Kuta, KutaRole, MoonDayChange, MoonPoint,
        KUTA_METHOD, KUTA_TOTAL_MAX,
    };
    use crate::nature::kuja_dosha;
    use crate::zodiac::{Sign, PADA_SPAN_DEG};

    proptest! {
        #[test]
        fn total_is_the_sum_of_eight_bounded_scores(groom in positions(), bride in positions()) {
            let r = kuta_report(&groom, &bride);
            let mut sum = 0.0;
            for (i, s) in r.scores.iter().enumerate() {
                prop_assert_eq!(s.kuta, Kuta::ALL[i]);
                prop_assert_eq!(s.max, s.kuta.max());
                prop_assert!((0.0..=s.max).contains(&s.score), "{:?}: {}", s.kuta, s.score);
                // Puanlar ½'nin katıdır.
                prop_assert_eq!((s.score * 2.0).fract(), 0.0);
                let ids = kuta_class_ids(s.kuta);
                prop_assert!(ids.contains(&s.groom_class) && ids.contains(&s.bride_class));
                sum += s.score;
            }
            prop_assert_eq!(r.total, sum);
            prop_assert!((0.0..=KUTA_TOTAL_MAX).contains(&r.total));
            // Tara'da iki yön birden uygunsuz olamaz: en az 1½.
            prop_assert!(r.scores[Kuta::Tara as usize].score >= 1.5);
            prop_assert_eq!(r.method.as_str(), KUTA_METHOD);
            prop_assert_eq!(r.moon_day_change, [MoonDayChange::default(); 2]);
        }

        #[test]
        fn nadi_bhakoot_yoni_maitri_and_tara_do_not_change_when_roles_swap(
            groom in positions(),
            bride in positions(),
        ) {
            let (a, b) = (kuta_report(&groom, &bride), kuta_report(&bride, &groom));
            for kuta in [Kuta::Nadi, Kuta::Bhakoot, Kuta::Yoni, Kuta::GrahaMaitri, Kuta::Tara] {
                let (x, y) = (&a.scores[kuta as usize], &b.scores[kuta as usize]);
                prop_assert_eq!(x.score, y.score, "{:?}", kuta);
                // Sınıflar yer değiştirir.
                prop_assert_eq!(&x.groom_class, &y.bride_class);
                prop_assert_eq!(&x.bride_class, &y.groom_class);
            }
            // İstisnalar bakışımlı koşullardır; Kuja dosha ve yerleşim yarıları yer değiştirir.
            prop_assert_eq!(a.bhakoot_exception, b.bhakoot_exception);
            prop_assert_eq!(a.nadi_exception, b.nadi_exception);
            prop_assert_eq!(a.kuja_dosha, [b.kuja_dosha[1], b.kuja_dosha[0]]);
            prop_assert_eq!(a.approximate, b.approximate);
            for i in 0..9 {
                let (x, y) = (&a.overlay[i], &b.overlay[i + 9]);
                prop_assert_eq!((x.owner, y.owner), (KutaRole::Groom, KutaRole::Bride));
                prop_assert_eq!(
                    (x.graha, x.sign, x.house_from_lagna, x.house_from_moon),
                    (y.graha, y.sign, y.house_from_lagna, y.house_from_moon)
                );
            }
        }

        #[test]
        fn the_same_moon_scores_nadi_0_yoni_4_bhakoot_7_maitri_5(
            pada in 0u32..108,
            offset_a in 0.01f64..0.99,
            offset_b in 0.01f64..0.99,
        ) {
            // Aynı nakshatranın aynı padasında iki ayrı boylam.
            let at = |offset: f64| (f64::from(pada) + offset) * PADA_SPAN_DEG;
            let (g, b) = (MoonPoint::at(at(offset_a)), MoonPoint::at(at(offset_b)));
            prop_assert_eq!((g.nakshatra, g.pada), (b.nakshatra, b.pada));
            let scores = kuta_scores(&g, &b);
            let score = |kuta: Kuta| scores[kuta as usize].score;
            prop_assert_eq!(score(Kuta::Nadi), 0.0);
            prop_assert_eq!(score(Kuta::Yoni), 4.0);
            prop_assert_eq!(score(Kuta::Bhakoot), 7.0);
            prop_assert_eq!(score(Kuta::GrahaMaitri), 5.0);
            prop_assert_eq!(score(Kuta::Tara), 3.0);
            prop_assert_eq!(score(Kuta::Gana), 6.0);
            prop_assert_eq!(score(Kuta::Varna), 1.0);
            // Bir pada tek bir burçtadır, ama 15° bir pada sınırı değildir (13°20′–16°40′
            // padasının içindedir): Yay ve Oğlak'ta aynı padadaki iki Ay ayrı vashya
            // gruplarına düşebilir. O durumda vashya 1, diğer her durumda 2'dir.
            let same_half = (g.deg_in_sign < 15.0) == (b.deg_in_sign < 15.0);
            let split_sign = matches!(g.sign, Sign::Sagittarius | Sign::Capricorn);
            let vashya = if split_sign && !same_half { 1.0 } else { 2.0 };
            prop_assert_eq!(score(Kuta::Vashya), vashya);
            prop_assert_eq!(scores.iter().map(|s| s.score).sum::<f64>(), 26.0 + vashya);
        }

        #[test]
        fn exceptions_only_come_with_a_zero_and_the_overlay_counts_whole_sign_houses(
            groom in positions(),
            bride in positions(),
        ) {
            let r = kuta_report(&groom, &bride);
            if r.bhakoot_exception {
                prop_assert_eq!(r.scores[Kuta::Bhakoot as usize].score, 0.0);
                // Aynı lord ya da iki yönlü dost: graha maitri 5'tir.
                prop_assert_eq!(r.scores[Kuta::GrahaMaitri as usize].score, 5.0);
            }
            if r.nadi_exception {
                prop_assert_eq!(r.scores[Kuta::Nadi as usize].score, 0.0);
                let (g, b) = (MoonPoint::of(&groom), MoonPoint::of(&bride));
                prop_assert!((g.nakshatra, g.pada) != (b.nakshatra, b.pada));
            }
            prop_assert_eq!(r.kuja_dosha, [kuja_dosha(&groom), kuja_dosha(&bride)]);
            prop_assert_eq!(
                r.approximate,
                groom.lagna_sign.is_none() || bride.lagna_sign.is_none()
            );
            for (i, row) in r.overlay.iter().enumerate() {
                let (own, other) = if i < 9 { (&groom, &bride) } else { (&bride, &groom) };
                prop_assert_eq!(row.graha, Graha::ALL[i % 9]);
                prop_assert_eq!(row.sign, own.sign_of(row.graha));
                prop_assert!((1..=12).contains(&row.house_from_moon));
                prop_assert_eq!(row.house_from_lagna.is_some(), other.lagna_sign.is_some());
                // Lagnadan ve Ay'dan sayım aynı burcu gösterir: fark, Ay'ın lagnadan evidir.
                if let (Some(from_lagna), Some(moon_house)) =
                    (row.house_from_lagna, other.house_of(Graha::Moon))
                {
                    prop_assert_eq!(
                        (from_lagna + 12 - moon_house) % 12 + 1,
                        row.house_from_moon
                    );
                }
            }
            // Bir tarafın Ay'ı kendi Ay'ından değil, diğerinin Ay'ından sayılır: iki Ay aynı
            // burçtaysa 1. ev.
            let same = groom.sign_of(Graha::Moon) == bride.sign_of(Graha::Moon);
            prop_assert_eq!(r.overlay[Graha::Moon as usize].house_from_moon == 1, same);
        }
    }
}

mod timeline_properties {
    use ephemeris::Body;
    use proptest::prelude::*;

    use crate::fake::FakeEph;
    use crate::graha::Graha;
    use crate::settings::CalcSettings;
    use crate::timeline::{
        graha_sign_series, karana_series, lagna_sign_series, nakshatra_series, nitya_yoga_series,
        tithi_series, vara_series, Interval,
    };

    const JD: f64 = 2_451_545.0;

    /// Gerçekçi hızlarla sahte gök: Güneş, Ay, Jüpiter, Venüs ve dönen lagna rastgele yerlerde
    /// başlar (tropikal boylamlar; hızlar °/gün).
    #[derive(Debug, Clone)]
    pub(super) struct Sky {
        sun: (f64, f64),
        moon: (f64, f64),
        asc: f64,
        jupiter: f64,
        venus: f64,
    }

    impl Sky {
        pub(super) fn eph(&self) -> FakeEph {
            FakeEph {
                asc: self.asc,
                asc_speed: 360.98,
                ..FakeEph::default()
            }
            .with(Body::Sun, self.sun.0, self.sun.1)
            .with(Body::Moon, self.moon.0, self.moon.1)
            .with(Body::Jupiter, self.jupiter, 0.08)
            .with(Body::Venus, self.venus, 1.2)
        }
    }

    pub(super) fn sky() -> impl Strategy<Value = Sky> {
        (
            (0.0f64..360.0, 0.95f64..1.02),
            (0.0f64..360.0, 11.8f64..15.4),
            0.0f64..360.0,
            0.0f64..360.0,
            0.0f64..360.0,
        )
            .prop_map(|(sun, moon, asc, jupiter, venus)| Sky {
                sun,
                moon,
                asc,
                jupiter,
                venus,
            })
    }

    fn check_partition<T: PartialEq + std::fmt::Debug>(
        series: &[Interval<T>],
        from: f64,
        to: f64,
    ) -> Result<(), TestCaseError> {
        prop_assert!(!series.is_empty());
        prop_assert_eq!(series[0].start_jd_ut, from);
        prop_assert_eq!(series[series.len() - 1].end_jd_ut, to);
        for w in series.windows(2) {
            prop_assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
            prop_assert!(w[0].value != w[1].value);
        }
        for i in series {
            prop_assert!(i.start_jd_ut < i.end_jd_ut);
        }
        Ok(())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        #[test]
        fn every_series_splits_the_span_without_gaps(
            sky in sky(),
            start in 0.0f64..400.0,
            days in 0.01f64..12.0,
        ) {
            let mut eph = sky.eph();
            let s = CalcSettings::default();
            let (from, to) = (JD + start, JD + start + days);
            check_partition(&tithi_series(&mut eph, &s, from, to).unwrap(), from, to)?;
            check_partition(&karana_series(&mut eph, &s, from, to).unwrap(), from, to)?;
            check_partition(&nakshatra_series(&mut eph, &s, from, to).unwrap(), from, to)?;
            check_partition(&nitya_yoga_series(&mut eph, &s, from, to).unwrap(), from, to)?;
            check_partition(&vara_series(&mut eph, &s, 41.0, 29.0, from, to).unwrap(), from, to)?;
            check_partition(
                &lagna_sign_series(&mut eph, &s, 41.0, 29.0, from, to).unwrap(),
                from,
                to,
            )?;
            for g in Graha::ALL {
                check_partition(&graha_sign_series(&mut eph, &s, g, from, to).unwrap(), from, to)?;
            }
        }
    }
}

mod muhurta_properties {
    use ephemeris::{Body, Ephemeris};
    use proptest::prelude::*;

    use super::timeline_properties::sky;
    use crate::chart::rasi_frame;
    use crate::combustion::is_combust;
    use crate::fake::FakeEph;
    use crate::graha::Graha;
    use crate::muhurta::day::sun_day_at;
    use crate::muhurta::search::search_page_with;
    use crate::muhurta::{
        chandra_bala, day_panchang, day_segments, is_panchaka, is_rikta, is_vishti, tara_bala,
        window_snapshot, MuhurtaCriteria, MuhurtaFilter, MuhurtaFlag, MuhurtaWindow, NatalMoon,
        MUHURTA_MIN_WINDOW_DAYS, VAIDHRITI_YOGA, VYATIPATA_YOGA,
    };
    use crate::panchang::Vara;
    use crate::settings::CalcSettings;
    use crate::shadbala::vara_lord;
    use crate::timeline::{NakshatraPada, SunDay};
    use crate::zodiac::Sign;

    const JD: f64 = 2_451_545.0;
    const SECOND: f64 = 1.0 / 86_400.0;

    fn vara() -> impl Strategy<Value = Vara> {
        (0usize..7).prop_map(|i| Vara::ALL[i])
    }

    /// Gün: doğuş, 0,2–0,8 gün süren gündüz ve 0,9–1,1 gün sonraki doğuş.
    fn sun_day() -> impl Strategy<Value = SunDay> {
        (
            2_400_000.0f64..2_500_000.0,
            0.2f64..0.8,
            0.9f64..1.1,
            vara(),
        )
            .prop_map(|(rise, daylight, length, vara)| SunDay {
                sunrise_jd_ut: rise,
                sunset_jd_ut: rise + daylight,
                next_sunrise_jd_ut: rise + length,
                vara,
            })
    }

    fn subset<T: Clone + std::fmt::Debug + 'static>(all: Vec<T>) -> impl Strategy<Value = Vec<T>> {
        let n = all.len();
        // Çoğu durumda boş küme (kısıt yok); yoksa rastgele bir alt küme.
        prop_oneof![
            3 => Just(Vec::new()),
            2 => proptest::sample::subsequence(all, 1..=n),
        ]
    }

    fn criteria() -> impl Strategy<Value = MuhurtaCriteria> {
        (
            subset(Vara::ALL.to_vec()),
            subset((1..=30u8).collect()),
            subset((1..=27u8).collect()),
            subset(Sign::ALL.to_vec()),
            any::<bool>(),
            prop_oneof![3 => Just(0u32), 1 => 1u32..180],
            subset(MuhurtaFlag::ALL.to_vec()),
            any::<bool>(),
            any::<bool>(),
        )
            .prop_map(
                |(
                    weekdays,
                    tithis,
                    nakshatras,
                    lagna_signs,
                    daytime_only,
                    min_minutes,
                    exclude,
                    require_tara,
                    require_chandra,
                )| MuhurtaCriteria {
                    weekdays,
                    tithis,
                    nakshatras,
                    lagna_signs,
                    daytime_only,
                    min_minutes,
                    exclude,
                    require_tara,
                    require_chandra,
                },
            )
    }

    /// Süzgecin `jd` anında sağlanıp sağlanmadığı, aramadan bağımsız noktasal hesapla.
    fn holds_at(eph: &mut FakeEph, s: &CalcSettings, f: &MuhurtaFilter, jd: f64) -> bool {
        let c = &f.criteria;
        let snap = window_snapshot(eph, s, jd, f.lat, f.lon).unwrap();
        let excluded = |flag| c.exclude.contains(&flag);
        if !c.weekdays.is_empty() && !c.weekdays.contains(&snap.vara) {
            return false;
        }
        if !c.tithis.is_empty() && !c.tithis.contains(&snap.tithi) {
            return false;
        }
        if !c.nakshatras.is_empty() && !c.nakshatras.contains(&snap.nakshatra) {
            return false;
        }
        if !c.lagna_signs.is_empty() && !c.lagna_signs.contains(&snap.lagna_sign) {
            return false;
        }
        let moon = NakshatraPada {
            nakshatra: snap.nakshatra,
            pada: snap.pada,
        };
        if (excluded(MuhurtaFlag::Rikta) && is_rikta(snap.tithi))
            || (excluded(MuhurtaFlag::Vishti) && is_vishti(snap.karana))
            || (excluded(MuhurtaFlag::Vyatipata) && snap.yoga == VYATIPATA_YOGA)
            || (excluded(MuhurtaFlag::Vaidhriti) && snap.yoga == VAIDHRITI_YOGA)
            || (excluded(MuhurtaFlag::Panchaka) && is_panchaka(moon))
        {
            return false;
        }
        let frame = rasi_frame(s);
        let sun = eph.body(jd, Body::Sun, frame).unwrap().lon;
        for (flag, graha, body) in [
            (MuhurtaFlag::JupiterCombust, Graha::Jupiter, Body::Jupiter),
            (MuhurtaFlag::VenusCombust, Graha::Venus, Body::Venus),
        ] {
            let p = eph.body(jd, body, frame).unwrap();
            if excluded(flag)
                && is_combust(graha, p.lon, sun, p.lon_speed < 0.0, &s.combustion_orbs)
            {
                return false;
            }
        }
        let day = sun_day_at(eph, s, jd, f.lat, f.lon).unwrap();
        if c.daytime_only && !day.is_some_and(|d| jd < d.sunset_jd_ut) {
            return false;
        }
        if let Some(d) = day {
            let seg = day_segments(&d);
            if (excluded(MuhurtaFlag::RahuSegment) && seg.rahu.contains(jd))
                || (excluded(MuhurtaFlag::Yamaganda) && seg.yamaganda.contains(jd))
                || (excluded(MuhurtaFlag::Gulika) && seg.gulika.contains(jd))
            {
                return false;
            }
        }
        if let Some(natal) = &f.natal {
            if c.require_tara && !tara_bala(natal.nakshatra, snap.nakshatra).favourable {
                return false;
            }
            let moon_sign = Sign::of(eph.body(jd, Body::Moon, frame).unwrap().lon);
            if c.require_chandra && !chandra_bala(natal.sign, moon_sign).favourable {
                return false;
            }
        }
        true
    }

    fn search_all(
        eph: &mut FakeEph,
        s: &CalcSettings,
        f: &MuhurtaFilter,
        chunk_days: f64,
    ) -> Vec<MuhurtaWindow> {
        let mut out = Vec::new();
        let mut cursor = None;
        loop {
            let page = search_page_with(eph, s, f, cursor, chunk_days).unwrap();
            out.extend(page.windows);
            match page.next {
                Some(next) => cursor = Some(next),
                None => return out,
            }
        }
    }

    proptest! {
        #[test]
        fn eight_segments_split_the_daytime_and_the_three_marked_ones_differ(day in sun_day()) {
            let s = day_segments(&day);
            prop_assert_eq!(s.eighths.len(), 8);
            prop_assert_eq!(s.eighths[0].start_jd_ut, day.sunrise_jd_ut);
            prop_assert_eq!(s.eighths[7].end_jd_ut, day.sunset_jd_ut);
            for w in s.eighths.windows(2) {
                prop_assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
                prop_assert!(w[0].start_jd_ut < w[0].end_jd_ut);
            }
            let marked = [s.rahu, s.yamaganda, s.gulika];
            for m in marked {
                prop_assert_eq!(m, s.eighths[usize::from(m.value) - 1]);
            }
            prop_assert!(marked[0].value != marked[1].value);
            prop_assert!(marked[1].value != marked[2].value);
            prop_assert!(marked[0].value != marked[2].value);
            // Abhijit gündüzün tam ortasını içerir ve on beşte biri kadar sürer.
            let noon = 0.5 * (day.sunrise_jd_ut + day.sunset_jd_ut);
            prop_assert!(s.abhijit.contains(noon));
            let daylight = day.sunset_jd_ut - day.sunrise_jd_ut;
            prop_assert!((s.abhijit.days() - daylight / 15.0).abs() < 1e-9);
            prop_assert_eq!(s.abhijit_weekday_exception, day.vara == Vara::Wednesday);
        }

        #[test]
        fn twenty_four_horas_split_the_day_and_start_with_the_day_lord(day in sun_day()) {
            let s = day_segments(&day);
            prop_assert_eq!(s.horas.len(), 24);
            prop_assert_eq!(s.horas[0].start_jd_ut, day.sunrise_jd_ut);
            prop_assert_eq!(s.horas[23].end_jd_ut, day.next_sunrise_jd_ut);
            prop_assert_eq!(s.horas[0].value, vara_lord(day.vara));
            let length = (day.next_sunrise_jd_ut - day.sunrise_jd_ut) / 24.0;
            for (k, h) in s.horas.iter().enumerate() {
                prop_assert!((h.days() - length).abs() < 1e-9);
                if k > 0 {
                    prop_assert_eq!(s.horas[k - 1].end_jd_ut, h.start_jd_ut);
                }
                // Yedi saatte bir aynı lord.
                if k >= 7 {
                    prop_assert_eq!(h.value, s.horas[k - 7].value);
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        #[test]
        fn the_day_panchang_splits_its_day_and_keeps_one_vara(
            sky in sky(),
            offset in 0.0f64..400.0,
            nakshatra in 1u8..=27,
            sign in 0usize..12,
        ) {
            let mut eph = sky.eph();
            let s = CalcSettings::default();
            let natal = NatalMoon { nakshatra, sign: Sign::ALL[sign] };
            let jd = JD + offset;
            let d = day_panchang(&mut eph, jd, 41.0, 29.0, &s, Some(natal)).unwrap();
            prop_assert!(d.day_start_jd_ut <= jd && jd < d.day_end_jd_ut);
            let (a, b) = (d.day_start_jd_ut, d.day_end_jd_ut);
            for (first, last) in [
                (d.tithis[0].start_jd_ut, d.tithis[d.tithis.len() - 1].end_jd_ut),
                (d.nakshatras[0].start_jd_ut, d.nakshatras[d.nakshatras.len() - 1].end_jd_ut),
                (d.yogas[0].start_jd_ut, d.yogas[d.yogas.len() - 1].end_jd_ut),
                (d.karanas[0].start_jd_ut, d.karanas[d.karanas.len() - 1].end_jd_ut),
            ] {
                prop_assert_eq!((first, last), (a, b));
            }
            let p = d.personal.as_ref().unwrap();
            prop_assert_eq!(p.tara[0].start_jd_ut, a);
            prop_assert_eq!(p.chandra[p.chandra.len() - 1].end_jd_ut, b);
            for f in &d.flags {
                prop_assert!(a <= f.start_jd_ut && f.start_jd_ut < f.end_jd_ut && f.end_jd_ut <= b);
            }
            let seg = d.segments.as_ref().unwrap();
            prop_assert_eq!(seg.horas[0].value, vara_lord(d.vara));
            prop_assert_eq!(seg.horas[23].end_jd_ut, b);
        }

        #[test]
        fn every_moment_of_a_window_passes_the_filter_and_gaps_do_not(
            sky in sky(),
            start in 0.0f64..300.0,
            days in 1.0f64..25.0,
            criteria in criteria(),
            nakshatra in 1u8..=27,
            sign in 0usize..12,
            samples in proptest::collection::vec(0.0f64..1.0, 3),
            // Half of the cases walk the range in short chunks: windows then cross chunk ends
            // and are carried in the cursor, and the same checks hold for them.
            chunk in prop_oneof![Just(31.0f64), 0.7f64..9.0],
        ) {
            let mut eph = sky.eph();
            let s = CalcSettings::default();
            let f = MuhurtaFilter {
                from_jd_ut: JD + start,
                to_jd_ut: JD + start + days,
                lat: 41.0,
                lon: 29.0,
                criteria,
                natal: Some(NatalMoon { nakshatra, sign: Sign::ALL[sign] }),
            };
            let windows = search_all(&mut eph, &s, &f, chunk);
            let min_days = (f64::from(f.criteria.min_minutes) / 1440.0).max(MUHURTA_MIN_WINDOW_DAYS);
            let mut previous_end = f.from_jd_ut;
            for w in &windows {
                prop_assert!(f.from_jd_ut <= w.start_jd_ut && w.end_jd_ut <= f.to_jd_ut);
                prop_assert!(w.start_jd_ut < w.end_jd_ut);
                prop_assert!(w.end_jd_ut - w.start_jd_ut >= min_days);
                prop_assert!(previous_end <= w.start_jd_ut, "ordered and disjoint");
                // Pencerenin içinden rastgele anlar (uçlardan 1 sn içeride).
                let inner = w.end_jd_ut - w.start_jd_ut - 2.0 * SECOND;
                if inner > 0.0 {
                    for x in &samples {
                        let jd = w.start_jd_ut + SECOND + inner * x;
                        prop_assert!(holds_at(&mut eph, &s, &f, jd), "inside a window at {}", jd);
                    }
                }
                // En kısa süre yokken iki pencerenin arası süzgeci sağlamaz.
                let gap = w.start_jd_ut - previous_end;
                if f.criteria.min_minutes == 0 && gap > 4.0 * SECOND {
                    for x in &samples {
                        let jd = previous_end + SECOND + (gap - 2.0 * SECOND) * x;
                        prop_assert!(!holds_at(&mut eph, &s, &f, jd), "inside a gap at {}", jd);
                    }
                }
                previous_end = w.end_jd_ut;
            }
            // The stretch after the last window (the whole range when there is none) does not
            // pass the filter either.
            let gap = f.to_jd_ut - previous_end;
            if f.criteria.min_minutes == 0 && gap > 4.0 * SECOND {
                for x in &samples {
                    let jd = previous_end + SECOND + (gap - 2.0 * SECOND) * x;
                    prop_assert!(!holds_at(&mut eph, &s, &f, jd), "after the last window at {}", jd);
                }
            }
        }

        #[test]
        fn chunk_length_does_not_change_the_windows(
            sky in sky(),
            start in 0.0f64..300.0,
            days in 1.0f64..40.0,
            criteria in criteria(),
            chunk in 0.7f64..9.0,
        ) {
            let mut eph = sky.eph();
            let s = CalcSettings::default();
            let f = MuhurtaFilter {
                from_jd_ut: JD + start,
                to_jd_ut: JD + start + days,
                lat: 41.0,
                lon: 29.0,
                criteria,
                natal: Some(NatalMoon { nakshatra: 4, sign: Sign::Taurus }),
            };
            let whole = search_all(&mut eph, &s, &f, 400.0);
            let pieces = search_all(&mut eph, &s, &f, chunk);
            prop_assert_eq!(whole.len(), pieces.len());
            for (a, b) in whole.iter().zip(&pieces) {
                // Sınırlar aynı ikiye bölmeden gelmez: çözünürlük kadar oynayabilir.
                prop_assert!((a.start_jd_ut - b.start_jd_ut).abs() < SECOND);
                prop_assert!((a.end_jd_ut - b.end_jd_ut).abs() < SECOND);
            }
        }
    }
}

mod prashna_properties {
    use ephemeris::Body;
    use proptest::prelude::*;

    use crate::chart::{compute_chart, compute_chart_with_lagna};
    use crate::fake::FakeEph;
    use crate::graha::Graha;
    use crate::houses::{whole_sign_cusps, whole_sign_house};
    use crate::lordship::{house_lord, house_sign};
    use crate::nature::{moon_is_waxing, nature};
    use crate::panchang::Paksha;
    use crate::positions::{compute_positions, ChartInput, Positions};
    use crate::prashna::{number_lagna_lon, prashna_facts, LordLink, PRASHNA_NUMBER_MAX};
    use crate::settings::{CalcSettings, CombustionOrbs, HouseSystem, ZodiacMode};
    use crate::yogas::yogas;
    use crate::zodiac::Sign;

    const JD: f64 = 2_451_545.0;
    const PLANETS: [Body; 7] = [
        Body::Sun,
        Body::Moon,
        Body::Mars,
        Body::Mercury,
        Body::Jupiter,
        Body::Venus,
        Body::Saturn,
    ];
    const SPEEDS: [f64; 7] = [0.98, 13.2, 0.6, 1.2, -0.1, 1.1, 0.05];

    fn sky(lons: [f64; 7], asc: f64, node: f64, ayanamsa: f64) -> FakeEph {
        let mut eph = FakeEph {
            asc,
            ayanamsa,
            ..FakeEph::default()
        };
        for ((body, lon), speed) in PLANETS.iter().zip(lons).zip(SPEEDS) {
            eph = eph.with(*body, lon, speed);
        }
        eph.with(Body::MeanNode, node, -0.053)
    }

    fn settings(tropical: bool, house_system: HouseSystem) -> CalcSettings {
        CalcSettings {
            zodiac: if tropical {
                ZodiacMode::TropicalRasiSiderealNakshatra
            } else {
                ZodiacMode::Sidereal
            },
            house_system,
            ..CalcSettings::default()
        }
    }

    fn house_systems() -> impl Strategy<Value = HouseSystem> {
        prop_oneof![
            Just(HouseSystem::WholeSign),
            Just(HouseSystem::Sripati),
            Just(HouseSystem::Placidus),
        ]
    }

    proptest! {
        /// Spec §11: sayı prashnasında grahaların boylamı aynı anın zaman prashnasıyla aynıdır;
        /// yalnızca lagna ve evler değişir.
        #[test]
        fn a_number_changes_only_the_lagna_and_the_houses(
            lons in proptest::array::uniform7(0.0f64..360.0),
            asc in 0.0f64..360.0,
            node in 0.0f64..360.0,
            ayanamsa in 20.0f64..26.0,
            number in 1u8..=PRASHNA_NUMBER_MAX,
            tropical in any::<bool>(),
            house_system in house_systems(),
        ) {
            let mut eph = sky(lons, asc, node, ayanamsa);
            let s = settings(tropical, house_system);
            let time = compute_chart(&mut eph, JD, 10.0, 20.0, &s).unwrap();
            let lagna = number_lagna_lon(number).unwrap();
            let c = compute_chart_with_lagna(&mut eph, JD, 10.0, 20.0, &s, lagna).unwrap();
            prop_assert_eq!(c.lagna.sidereal_lon, lagna);
            prop_assert_eq!(
                (u16::from(c.lagna.nakshatra) - 1) * 4 + u16::from(c.lagna.pada),
                u16::from(number)
            );
            prop_assert_eq!(c.lagna.sign, Sign::of(c.lagna.lon));
            if !tropical {
                prop_assert_eq!(c.lagna.sign.index(), usize::from(number - 1) / 9);
            }
            prop_assert_eq!(c.bhavas.system, HouseSystem::WholeSign);
            prop_assert_eq!(&c.bhavas.cusps, &whole_sign_cusps(c.lagna.sign));
            for (a, b) in c.grahas.iter().zip(&time.grahas) {
                prop_assert_eq!(a.graha, b.graha);
                prop_assert_eq!(a.lon, b.lon);
                prop_assert_eq!(a.sidereal_lon, b.sidereal_lon);
                prop_assert_eq!(a.speed, b.speed);
                prop_assert_eq!((a.sign, a.nakshatra, a.pada), (b.sign, b.nakshatra, b.pada));
                prop_assert_eq!((a.retrograde, a.combust, a.dignity), (b.retrograde, b.combust, b.dignity));
                prop_assert_eq!(a.house, whole_sign_house(a.sign, c.lagna.sign));
            }
            prop_assert_eq!(&c.dasha, &time.dasha);
            prop_assert_eq!(&c.panchang, &time.panchang);
            prop_assert_eq!(c.ayanamsa_deg, time.ayanamsa_deg);
            // Modül girdisi aynı haritayı görür.
            let input = ChartInput {
                jd_ut: JD,
                lat: 10.0,
                lon: 20.0,
                settings: s,
                time_known: true,
                lagna_number: Some(number),
            };
            let p = compute_positions(&mut eph, &input).unwrap();
            prop_assert_eq!(p.lagna_lon, Some(lagna));
            prop_assert_eq!(p.lagna_sign, Some(c.lagna.sign));
        }

        /// Olguların iç tutarlılığı: her graha lagnadan sayılan evindedir, kendralar yalnızca
        /// 1, 4, 7 ve 10. evlerin grahalarını taşır, iki lordun ev sayıları birbirinin
        /// tümleyenidir ve bağlar kendi tanımlarını sağlar.
        #[test]
        fn facts_are_consistent(
            lons in proptest::array::uniform9(0.0f64..360.0),
            lagna in 0.0f64..360.0,
            focus in 1u8..=12,
        ) {
            let p = Positions::synthetic(Some(lagna), lons);
            let f = prashna_facts(&p, &CombustionOrbs::default(), Some(focus)).unwrap();
            let l = f.lagna.unwrap();
            let lagna_sign = p.lagna_sign.unwrap();
            prop_assert_eq!(l.sign, lagna_sign);
            prop_assert_eq!(l.lord.graha, house_lord(lagna_sign, 1));
            prop_assert_eq!(l.lord.sign, p.sign_of(l.lord.graha));
            prop_assert_eq!(Some(l.lord.house), p.house_of(l.lord.graha));
            let in_kendras: Vec<Graha> = l
                .kendras
                .iter()
                .flat_map(|k| k.grahas.iter().map(|o| o.graha))
                .collect();
            for g in Graha::ALL {
                let house = p.house_of(g).unwrap();
                prop_assert_eq!(in_kendras.contains(&g), matches!(house, 1 | 4 | 7 | 10));
            }
            for k in &l.kendras {
                prop_assert_eq!(k.sign, house_sign(lagna_sign, k.house));
                for o in &k.grahas {
                    prop_assert_eq!(p.sign_of(o.graha), k.sign);
                    prop_assert_eq!(o.nature, nature(o.graha, &p));
                }
            }
            let fo = f.focus.unwrap();
            prop_assert_eq!(fo.sign, house_sign(lagna_sign, focus));
            prop_assert_eq!(fo.lord.graha, house_lord(lagna_sign, focus));
            let r = &fo.relation;
            let (a, b) = (r.focus_lord_from_lagna_lord, r.lagna_lord_from_focus_lord);
            // Aynı burç 1 ve 1; değilse iki sayım 14'e tamamlanır.
            prop_assert!((a == 1 && b == 1) || a + b == 14);
            prop_assert_eq!(r.same_graha, l.lord.graha == fo.lord.graha);
            if r.same_graha {
                prop_assert!(r.links.is_empty());
            }
            prop_assert_eq!(r.links.contains(&LordLink::Conjunct), !r.same_graha && a == 1);
            let one_way = [LordLink::LagnaLordAspects, LordLink::FocusLordAspects];
            let aspect_links = r
                .links
                .iter()
                .filter(|k| **k == LordLink::MutualAspect || one_way.contains(k))
                .count();
            prop_assert!(aspect_links <= 1);
            // Birlikte duran iki graha birbirinin burcuna bakmaz ve değişimde olamaz.
            if r.links.contains(&LordLink::Conjunct) {
                prop_assert_eq!(r.links.len(), 1);
            }
            let mut sorted = r.links.clone();
            sorted.sort_by_key(|k| LordLink::ALL.iter().position(|x| x == k));
            prop_assert_eq!(&sorted, &r.links);
            // Ay: ev, tithi ve paksha birbirini tutar.
            prop_assert_eq!(f.moon.house, p.house_of(Graha::Moon));
            prop_assert_eq!(f.moon.paksha == Paksha::Shukla, moon_is_waxing(&p));
            prop_assert!(!f.moon.aspected_by.contains(&Graha::Moon));
            prop_assert_eq!(&f.yogas, &yogas(&p));
            // Lagnasız haritada lagnaya bağlı hiçbir şey yoktur.
            let hidden = prashna_facts(&p.clone().without_lagna(), &CombustionOrbs::default(), Some(focus)).unwrap();
            prop_assert!(hidden.lagna.is_none() && hidden.focus.is_none());
            prop_assert_eq!(hidden.moon.house, None);
        }

        /// Spec §11: `compute_chart_with_lagna` gerçek lagna verildiğinde `compute_chart` ile
        /// aynı haritayı döner. Burç evi sisteminde ve sidereal kipte eşitlik bit bittir;
        /// tropikal-rasi kipinde lagnanın rasi boylamı `sidereal + ayanamsa` ile yeniden
        /// kurulur (yuvarlama payı) ve burç sınırına bu paydan yakın lagnalar atlanır.
        #[test]
        fn the_real_lagna_gives_the_chart_of_compute_chart(
            lons in proptest::array::uniform7(0.0f64..360.0),
            asc in 0.0f64..360.0,
            node in 0.0f64..360.0,
            ayanamsa in 20.0f64..26.0,
            tropical in any::<bool>(),
        ) {
            let mut eph = sky(lons, asc, node, ayanamsa);
            let s = settings(tropical, HouseSystem::WholeSign);
            let base = compute_chart(&mut eph, JD, 10.0, 20.0, &s).unwrap();
            let again =
                compute_chart_with_lagna(&mut eph, JD, 10.0, 20.0, &s, base.lagna.sidereal_lon)
                    .unwrap();
            if tropical {
                let d = base.lagna.deg_in_sign;
                prop_assume!(d > 1e-6 && d < 30.0 - 1e-6);
                prop_assert!((again.lagna.lon - base.lagna.lon).abs() < 1e-9);
                prop_assert_eq!(again.lagna.sidereal_lon, base.lagna.sidereal_lon);
                prop_assert_eq!(again.lagna.sign, base.lagna.sign);
                prop_assert_eq!(&again.grahas, &base.grahas);
                prop_assert_eq!(&again.bhavas, &base.bhavas);
                prop_assert_eq!(&again.ashtakavarga, &base.ashtakavarga);
                prop_assert_eq!(&again.dasha, &base.dasha);
                prop_assert_eq!(&again.panchang, &base.panchang);
                prop_assert_eq!(&again.meta, &base.meta);
            } else {
                prop_assert_eq!(&again, &base);
            }
        }
    }
}
