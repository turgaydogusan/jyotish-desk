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
