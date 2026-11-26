//! Zaman gücü (ileri modüller spec'i §7, "3. Zaman gücü"; Ek A: bu güç yalnızca `temporal`
//! adıyla anılır): gündüz–gece gücü, paksha bala, tribhaga bala, yıl, ay, gün ve saat lordu,
//! ayana bala. Kaynak: BPHS, graha gücü bölümü.
//!
//! Fonksiyonlar ephemeris görmez: gün doğumu, batımı ve giriş varaları `ShadbalaAstro` ile
//! verilir (K3). Zamanlar UT Jülyen günüdür; birim virupadır.

use crate::graha::Graha;
use crate::nature::{nature, Nature};
use crate::panchang::Vara;
use crate::positions::Positions;
use crate::zodiac::angular_distance;

use super::{ShadbalaAstro, TemporalBala, TimeLords, SHADBALA_GRAHAS};

/// Ekliptiğin eğimi, ayana bala için: 23°27′.
pub const AYANA_OBLIQUITY_DEG: f64 = 23.45;
/// Yıl, ay, gün ve saat lordunun aldığı değerler.
pub const YEAR_LORD_BALA: f64 = 15.0;
pub const MONTH_LORD_BALA: f64 = 30.0;
pub const WEEKDAY_LORD_BALA: f64 = 45.0;
pub const HORA_LORD_BALA: f64 = 60.0;

/// Saat lordlarının döngüsü: yörünge hızına göre yavaştan hızlıya.
pub const HORA_CYCLE: [Graha; 7] = [
    Graha::Saturn,
    Graha::Jupiter,
    Graha::Mars,
    Graha::Sun,
    Graha::Venus,
    Graha::Mercury,
    Graha::Moon,
];
/// Gündüzün üç diliminin lordları (doğuş → batış).
pub const DAY_TRIBHAGA: [Graha; 3] = [Graha::Mercury, Graha::Sun, Graha::Saturn];
/// Gecenin üç diliminin lordları (batış → sonraki doğuş).
pub const NIGHT_TRIBHAGA: [Graha; 3] = [Graha::Moon, Graha::Venus, Graha::Mars];

/// Varanın lordu: Pazar Güneş, Pazartesi Ay, Salı Mars, Çarşamba Merkür, Perşembe Jüpiter,
/// Cuma Venüs, Cumartesi Satürn.
pub fn vara_lord(vara: Vara) -> Graha {
    match vara {
        Vara::Sunday => Graha::Sun,
        Vara::Monday => Graha::Moon,
        Vara::Tuesday => Graha::Mars,
        Vara::Wednesday => Graha::Mercury,
        Vara::Thursday => Graha::Jupiter,
        Vara::Friday => Graha::Venus,
        Vara::Saturday => Graha::Saturn,
    }
}

/// Doğum anının en yakın görünür öğleye uzaklığı, saat, [0, 12]. Görünür öğle, günün gün
/// doğumu ile batımının orta noktasıdır; ardışık öğleler 24 saat arayla sayılır (12 saatten
/// büyük uzaklık 24'ten çıkarılır).
pub fn hours_from_noon(astro: &ShadbalaAstro) -> f64 {
    let noon = 0.5 * (astro.sunrise_jd_ut + astro.sunset_jd_ut);
    let m = ((astro.jd_ut - noon) * 24.0).abs().rem_euclid(24.0);
    if m > 12.0 {
        24.0 - m
    } else {
        m
    }
}

/// Gündüz–gece gücü (`day_night`). `m`: öğleye uzaklık, saat (0–12). Güneş, Jüpiter ve Venüs
/// öğlede 60, gece yarısında 0; Ay, Mars ve Satürn tersine; Merkür her zaman 60. Düğümler 0.
pub fn day_night_bala(graha: Graha, m: f64) -> f64 {
    match graha {
        Graha::Sun | Graha::Jupiter | Graha::Venus => 60.0 * (1.0 - m / 12.0),
        Graha::Moon | Graha::Mars | Graha::Saturn => 60.0 * m / 12.0,
        Graha::Mercury => 60.0,
        Graha::Rahu | Graha::Ketu => 0.0,
    }
}

/// Ay'ın Güneş'ten uzanımı, 0°–180°'ye indirgenmiş (yeni ayda 0, dolunayda 180).
pub fn paksha_elongation(positions: &Positions) -> f64 {
    angular_distance(
        positions.point(Graha::Moon).sidereal_lon,
        positions.point(Graha::Sun).sidereal_lon,
    )
}

/// Paksha bala. `e` = `paksha_elongation`. Ay: `2 × e / 3`; diğerleri doğal sınıfına göre
/// (K6): iyicil `e / 3`, kötücül `(180 − e) / 3`. Düğümler 0.
pub fn paksha_bala(graha: Graha, positions: &Positions) -> f64 {
    let e = paksha_elongation(positions);
    match graha {
        Graha::Rahu | Graha::Ketu => 0.0,
        Graha::Moon => 2.0 * e / 3.0,
        _ => match nature(graha, positions) {
            Nature::Benefic => e / 3.0,
            Nature::Malefic => (180.0 - e) / 3.0,
        },
    }
}

/// Doğum gündüz mü: gün doğumundan (dahil) gün batımına (hariç).
pub fn is_daytime(astro: &ShadbalaAstro) -> bool {
    astro.jd_ut < astro.sunset_jd_ut
}

/// `[start, end)` aralığının `parts` eşit diliminden `t` anını içerenin sırası, 0'dan.
fn part_index(t: f64, start: f64, end: f64, parts: usize) -> usize {
    let x = (t - start) / (end - start) * parts as f64;
    // NaN ve negatifler 0'a, aralığın sonu son dilime düşer.
    (x.max(0.0) as usize).min(parts - 1)
}

/// İçinde bulunulan tribhaga diliminin lordu: gündüz Merkür, Güneş, Satürn; gece Ay, Venüs,
/// Mars.
pub fn tribhaga_lord(astro: &ShadbalaAstro) -> Graha {
    if is_daytime(astro) {
        DAY_TRIBHAGA[part_index(astro.jd_ut, astro.sunrise_jd_ut, astro.sunset_jd_ut, 3)]
    } else {
        NIGHT_TRIBHAGA[part_index(astro.jd_ut, astro.sunset_jd_ut, astro.next_sunrise_jd_ut, 3)]
    }
}

/// Doğum anını içeren saat diliminin (hora) sırası, 0..=23: gün doğumundan sonraki gün
/// doğumuna 24 eşit dilim.
pub fn hora_index(astro: &ShadbalaAstro) -> u8 {
    part_index(
        astro.jd_ut,
        astro.sunrise_jd_ut,
        astro.next_sunrise_jd_ut,
        24,
    ) as u8
}

/// `index`. saat diliminin lordu: ilk dilim gün lordunundur, sonrakiler `HORA_CYCLE`
/// sırasında ilerler.
pub fn hora_lord(weekday_lord: Graha, index: u8) -> Graha {
    let start = HORA_CYCLE
        .iter()
        .position(|g| *g == weekday_lord)
        .unwrap_or(0);
    HORA_CYCLE[(start + usize::from(index)) % 7]
}

/// Beş zaman lordu.
pub fn time_lords(astro: &ShadbalaAstro) -> TimeLords {
    let weekday = vara_lord(astro.weekday);
    TimeLords {
        year: vara_lord(astro.year_entry_weekday),
        month: vara_lord(astro.month_entry_weekday),
        weekday,
        hora: hora_lord(weekday, hora_index(astro)),
        tribhaga: tribhaga_lord(astro),
    }
}

/// Tropikal boylamın ekliptik üzerindeki dik açıklığı, derece: `asin(sin 23°27′ × sin λ)`.
pub fn ayana_declination(tropical_lon: f64) -> f64 {
    (AYANA_OBLIQUITY_DEG.to_radians().sin() * tropical_lon.to_radians().sin())
        .asin()
        .to_degrees()
}

/// Ayana bala (Güneş'te iki katı alınmamış hâli). `δ` = `ayana_declination`. Güneş, Mars,
/// Jüpiter, Venüs: `(24 + δ) / 48 × 60`; Ay, Satürn: `(24 − δ) / 48 × 60`; Merkür:
/// `(24 + |δ|) / 48 × 60`. Düğümler 0.
pub fn ayana_bala(graha: Graha, tropical_lon: f64) -> f64 {
    let d = ayana_declination(tropical_lon);
    let signed = match graha {
        Graha::Sun | Graha::Mars | Graha::Jupiter | Graha::Venus => d,
        Graha::Moon | Graha::Saturn => -d,
        Graha::Mercury => d.abs(),
        Graha::Rahu | Graha::Ketu => return 0.0,
    };
    (24.0 + signed) / 48.0 * 60.0
}

fn lord_bala(graha: Graha, lord: Graha, value: f64) -> f64 {
    if graha == lord {
        value
    } else {
        0.0
    }
}

/// Bir grahanın zaman gücü kalemleri ve toplamı. Güneş'in ayana balası iki katıdır; Jüpiter
/// tribhaga balayı her zaman alır. Düğümlerde her kalem 0'dır.
pub fn temporal_bala(
    graha: Graha,
    positions: &Positions,
    astro: &ShadbalaAstro,
    lords: &TimeLords,
) -> TemporalBala {
    // `SHADBALA_GRAHAS`, `Graha::ALL`'un ilk yedisidir: sıra numarası aynıdır. Düğümler
    // dizide yoktur ve güçleri hesaplanmaz.
    let Some(tropical) = SHADBALA_GRAHAS
        .iter()
        .position(|g| *g == graha)
        .map(|i| astro.tropical_lons[i])
    else {
        return TemporalBala {
            day_night: 0.0,
            paksha: 0.0,
            tribhaga: 0.0,
            year_lord: 0.0,
            month_lord: 0.0,
            weekday_lord: 0.0,
            hora_lord: 0.0,
            ayana: 0.0,
            total: 0.0,
        };
    };
    let day_night = day_night_bala(graha, hours_from_noon(astro));
    let paksha = paksha_bala(graha, positions);
    let tribhaga = if graha == Graha::Jupiter || graha == lords.tribhaga {
        60.0
    } else {
        0.0
    };
    let year_lord = lord_bala(graha, lords.year, YEAR_LORD_BALA);
    let month_lord = lord_bala(graha, lords.month, MONTH_LORD_BALA);
    let weekday_lord = lord_bala(graha, lords.weekday, WEEKDAY_LORD_BALA);
    let hora_lord = lord_bala(graha, lords.hora, HORA_LORD_BALA);
    let ayana = ayana_bala(graha, tropical) * if graha == Graha::Sun { 2.0 } else { 1.0 };
    TemporalBala {
        day_night,
        paksha,
        tribhaga,
        year_lord,
        month_lord,
        weekday_lord,
        hora_lord,
        ayana,
        total: day_night
            + paksha
            + tribhaga
            + year_lord
            + month_lord
            + weekday_lord
            + hora_lord
            + ayana,
    }
}

#[cfg(test)]
mod tests {
    use super::super::MeanLongitudes;
    use super::*;
    use Graha::*;

    /// Gün doğumu 06:00, batımı 18:00, sonraki doğuş 06:00 (JD kesirleri: 0,25 / 0,75 / 1,25).
    fn astro_at(jd_ut: f64) -> ShadbalaAstro {
        ShadbalaAstro {
            jd_ut,
            sunrise_jd_ut: 0.25,
            sunset_jd_ut: 0.75,
            next_sunrise_jd_ut: 1.25,
            mc_lon: 270.0,
            tropical_lons: [0.0; 7],
            mean: MeanLongitudes {
                sun: 0.0,
                mercury: 0.0,
                venus: 0.0,
                mars: 0.0,
                jupiter: 0.0,
                saturn: 0.0,
            },
            weekday: Vara::Sunday,
            year_entry_weekday: Vara::Monday,
            month_entry_weekday: Vara::Tuesday,
        }
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn the_nodes_get_no_temporal_strength() {
        // Rahu and Ketu have no Shadbala; like the other item functions this one returns zeros.
        let p = Positions::synthetic(Some(0.0), [10.0; 9]);
        let a = astro_at(0.5);
        let lords = time_lords(&a);
        for node in [Graha::Rahu, Graha::Ketu] {
            let t = temporal_bala(node, &p, &a, &lords);
            assert_eq!(
                [
                    t.day_night,
                    t.paksha,
                    t.tribhaga,
                    t.year_lord,
                    t.month_lord,
                    t.weekday_lord,
                    t.hora_lord,
                    t.ayana,
                    t.total
                ],
                [0.0; 9],
                "{node:?}"
            );
        }
    }

    #[test]
    fn hours_from_noon_fold_into_twelve() {
        // (an, öğleye uzaklık): öğle 0,5.
        for (jd, m) in [
            (0.5, 0.0),
            (0.25, 6.0),
            (0.75, 6.0),
            (1.0, 12.0),
            (0.625, 3.0),
            // 03:00 (sonraki sabaha karşı): öğleden 15 saat sonra; en yakın öğleye 9 saat.
            (1.125, 9.0),
            (1.249, 24.0 - 0.749 * 24.0),
        ] {
            assert!(close(hours_from_noon(&astro_at(jd)), m), "{jd}");
        }
        // Eşit olmayan gün: doğuş 0,30, batış 0,80 → öğle 0,55.
        let mut a = astro_at(0.55);
        a.sunrise_jd_ut = 0.30;
        a.sunset_jd_ut = 0.80;
        assert!(close(hours_from_noon(&a), 0.0));
    }

    #[test]
    fn day_night_strength_at_noon_midnight_and_between() {
        // Öğle: gündüz güçlüler 60, gece güçlüler 0.
        for g in [Sun, Jupiter, Venus] {
            assert_eq!(day_night_bala(g, 0.0), 60.0, "{g:?}");
            assert_eq!(day_night_bala(g, 12.0), 0.0, "{g:?}");
            assert_eq!(day_night_bala(g, 6.0), 30.0, "{g:?}");
            assert_eq!(day_night_bala(g, 3.0), 45.0, "{g:?}");
        }
        for g in [Moon, Mars, Saturn] {
            assert_eq!(day_night_bala(g, 0.0), 0.0, "{g:?}");
            assert_eq!(day_night_bala(g, 12.0), 60.0, "{g:?}");
            assert_eq!(day_night_bala(g, 6.0), 30.0, "{g:?}");
            assert_eq!(day_night_bala(g, 3.0), 15.0, "{g:?}");
        }
        for m in [0.0, 3.0, 6.0, 12.0] {
            assert_eq!(day_night_bala(Mercury, m), 60.0);
            assert_eq!(day_night_bala(Rahu, m), 0.0);
        }
    }

    /// Güneş 10°'de; Ay `moon`'da; Merkür yalnız (iyicil).
    fn sky(moon: f64) -> Positions {
        Positions::synthetic(
            Some(0.0),
            [10.0, moon, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0],
        )
    }

    #[test]
    fn paksha_strength_at_new_moon_full_moon_and_quarters() {
        // Dolunay (uzanım 180°): Ay 120, iyiciller 60, kötücüller 0.
        let full = sky(190.0);
        assert!(close(paksha_elongation(&full), 180.0));
        assert!(close(paksha_bala(Moon, &full), 120.0));
        for g in [Mercury, Jupiter, Venus] {
            assert!(close(paksha_bala(g, &full), 60.0), "{g:?}");
        }
        for g in [Sun, Mars, Saturn] {
            assert!(close(paksha_bala(g, &full), 0.0), "{g:?}");
        }
        // Yeni ay (uzanım 0°): Ay 0, iyiciller 0, kötücüller 60.
        let new = sky(10.0);
        assert!(close(paksha_bala(Moon, &new), 0.0));
        assert!(close(paksha_bala(Jupiter, &new), 0.0));
        assert!(close(paksha_bala(Saturn, &new), 60.0));
        // Büyüyen çeyrek (90°) ve küçülen çeyrek (270° → 90°): Ay 60, herkes 30.
        for moon in [100.0, 280.0] {
            let quarter = sky(moon);
            assert!(close(paksha_elongation(&quarter), 90.0));
            assert!(close(paksha_bala(Moon, &quarter), 60.0));
            assert!(close(paksha_bala(Venus, &quarter), 30.0));
            assert!(close(paksha_bala(Mars, &quarter), 30.0));
        }
        // Ara değer: uzanım 60° → Ay 40, iyicil 20, kötücül 40.
        let sixty = sky(70.0);
        assert!(close(paksha_bala(Moon, &sixty), 40.0));
        assert!(close(paksha_bala(Jupiter, &sixty), 20.0));
        assert!(close(paksha_bala(Sun, &sixty), 40.0));
        // Merkür kötücülle aynı burçta (ikisi de Aslan'da; uzanım 60°): kötücül sayılır, 40.
        // Satürn Kova'ya alınınca Merkür yalnızdır: iyicil, 20.
        let mut lons = [10.0, 70.0, 220.0, 125.0, 250.0, 40.0, 130.0, 200.0, 310.0];
        assert!(close(
            paksha_bala(Mercury, &Positions::synthetic(Some(0.0), lons)),
            40.0
        ));
        lons[6] = 300.0;
        assert!(close(
            paksha_bala(Mercury, &Positions::synthetic(Some(0.0), lons)),
            20.0
        ));
        assert_eq!(paksha_bala(Rahu, &full), 0.0);
    }

    #[test]
    fn tribhaga_lords_of_the_six_parts() {
        // Gündüz 06:00–18:00: 4'er saatlik dilimler; gece 18:00–06:00: 4'er saatlik dilimler.
        for (jd, lord) in [
            (0.25, Mercury), // 06:00
            (0.25 + 3.9 / 24.0, Mercury),
            (0.25 + 4.01 / 24.0, Sun), // 10:00'dan hemen sonra
            (0.5, Sun),
            (0.25 + 8.01 / 24.0, Saturn), // 14:00'ten hemen sonra
            (0.749, Saturn),
            (0.75, Moon), // 18:00: gece başlar
            (0.75 + 3.9 / 24.0, Moon),
            (0.75 + 4.01 / 24.0, Venus), // 22:00'den hemen sonra
            (1.0, Venus),
            (0.75 + 8.01 / 24.0, Mars), // 02:00'den hemen sonra
            (1.249, Mars),
        ] {
            let a = astro_at(jd);
            assert_eq!(tribhaga_lord(&a), lord, "{jd}");
            assert_eq!(is_daytime(&a), jd < 0.75, "{jd}");
        }
    }

    /// Elle yazılmış saat lordu tablosu (bağımsız ikinci yazım): her varanın 24 saat dilimi.
    const HAND_HORAS: [(Vara, [Graha; 24]); 7] = [
        (
            Vara::Sunday,
            [
                Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon,
                Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun,
                Venus, Mercury,
            ],
        ),
        (
            Vara::Monday,
            [
                Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun,
                Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn,
                Jupiter,
            ],
        ),
        (
            Vara::Tuesday,
            [
                Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon,
                Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun,
                Venus,
            ],
        ),
        (
            Vara::Wednesday,
            [
                Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter,
                Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon,
                Saturn,
            ],
        ),
        (
            Vara::Thursday,
            [
                Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus,
                Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter,
                Mars, Sun,
            ],
        ),
        (
            Vara::Friday,
            [
                Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn,
                Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus,
                Mercury, Moon,
            ],
        ),
        (
            Vara::Saturday,
            [
                Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun,
                Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn,
                Jupiter, Mars,
            ],
        ),
    ];

    #[test]
    fn hora_lords_match_the_hand_table() {
        for (i, (vara, row)) in HAND_HORAS.into_iter().enumerate() {
            let day_lord = vara_lord(vara);
            assert_eq!(
                row[0], day_lord,
                "{vara:?}: the first hora is the day lord's"
            );
            for (k, want) in row.into_iter().enumerate() {
                assert_eq!(hora_lord(day_lord, k as u8), want, "{vara:?} hora {k}");
            }
            // 25. dilim sonraki günün ilk dilimidir: sonraki varanın lordu.
            let next = HAND_HORAS[(i + 1) % 7].0;
            assert_eq!(hora_lord(day_lord, 24), vara_lord(next), "{vara:?}");
        }
        // Dilim sırası: 06:00'dan saat başına bir dilim.
        for (jd, index) in [
            (0.25, 0),
            (0.25 + 0.99 / 24.0, 0),
            (0.25 + 1.0 / 24.0 + 1e-9, 1),
            (0.5, 6),
            (0.75, 12),
            (1.0, 18),
            (1.249, 23),
        ] {
            assert_eq!(hora_index(&astro_at(jd)), index, "{jd}");
        }
        // Doğuşlar arası 24 saat değilse dilimler de eşit uzar: 25 saatlik günde ortası 12.
        let mut long = astro_at(0.25 + 12.5 / 24.0);
        long.next_sunrise_jd_ut = 0.25 + 25.0 / 24.0;
        assert_eq!(hora_index(&long), 12);
    }

    #[test]
    fn time_lords_come_from_the_three_weekdays_and_the_clock() {
        // Pazar, öğle (7. saat dilimi, sıra 6): Pazar tablosunda Mars; tribhaga Güneş.
        let lords = time_lords(&astro_at(0.5));
        assert_eq!(
            lords,
            TimeLords {
                year: Moon,
                month: Mars,
                weekday: Sun,
                hora: Mars,
                tribhaga: Sun
            }
        );
    }

    #[test]
    fn ayana_strength_at_the_solstices_and_equinoxes() {
        // Ekinokslarda δ = 0: herkes 30. Gündönümlerinde δ = ±23,45.
        let north = (24.0 + 23.45) / 48.0 * 60.0; // 59,3125
        let south = (24.0 - 23.45) / 48.0 * 60.0; // 0,6875
        assert!(close(north, 59.3125) && close(south, 0.6875));
        assert!(close(ayana_declination(90.0), 23.45));
        assert!(close(ayana_declination(270.0), -23.45));
        assert!(ayana_declination(0.0).abs() < 1e-12);
        for g in [Sun, Mars, Jupiter, Venus] {
            assert!(close(ayana_bala(g, 0.0), 30.0), "{g:?}");
            assert!(close(ayana_bala(g, 180.0), 30.0), "{g:?}");
            assert!(close(ayana_bala(g, 90.0), north), "{g:?}");
            assert!(close(ayana_bala(g, 270.0), south), "{g:?}");
        }
        for g in [Moon, Saturn] {
            assert!(close(ayana_bala(g, 0.0), 30.0), "{g:?}");
            assert!(close(ayana_bala(g, 90.0), south), "{g:?}");
            assert!(close(ayana_bala(g, 270.0), north), "{g:?}");
        }
        // Merkür her iki yarıda da kuzey gibi sayılır.
        assert!(close(ayana_bala(Mercury, 0.0), 30.0));
        assert!(close(ayana_bala(Mercury, 90.0), north));
        assert!(close(ayana_bala(Mercury, 270.0), north));
        // Ara değer: λ = 30° → sin δ = sin 23,45° × 0,5 = 0,198974 → δ = 11,4770°
        let d = ayana_declination(30.0);
        assert!((d - 11.4770).abs() < 1e-4, "{d}");
        assert!((ayana_bala(Sun, 30.0) - (24.0 + d) / 48.0 * 60.0).abs() < 1e-12);
        assert_eq!(ayana_bala(Ketu, 90.0), 0.0);
    }

    #[test]
    fn temporal_sums_the_eight_items_and_doubles_the_ayana_of_the_sun() {
        // Pazar öğle; Güneş tropikal 90°'de; dolunay.
        let mut a = astro_at(0.5);
        a.tropical_lons = [90.0, 270.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let p = sky(190.0);
        let lords = time_lords(&a);
        let sun = temporal_bala(Sun, &p, &a, &lords);
        // Gündüz–gece 60; paksha (kötücül, dolunay) 0; tribhaga (Güneş dilimi) 60; gün lordu
        // 45; ayana 2 × 59,3125.
        assert_eq!(
            (sun.day_night, sun.paksha, sun.tribhaga, sun.weekday_lord),
            (60.0, 0.0, 60.0, 45.0)
        );
        assert_eq!(
            (sun.year_lord, sun.month_lord, sun.hora_lord),
            (0.0, 0.0, 0.0)
        );
        assert!(close(sun.ayana, 118.625));
        assert!(close(sun.total, 60.0 + 60.0 + 45.0 + 118.625));
        // Ay: yıl lordu 15; paksha 120; gündüz–gece 0; ayana (güney, Ay için güçlü) 59,3125.
        let moon = temporal_bala(Moon, &p, &a, &lords);
        assert_eq!(
            (moon.year_lord, moon.tribhaga, moon.day_night),
            (15.0, 0.0, 0.0)
        );
        assert!(close(moon.paksha, 120.0) && close(moon.ayana, 59.3125));
        assert!(close(moon.total, 15.0 + 120.0 + 59.3125));
        // Mars: ay lordu 30 ve saat lordu 60; ayana 30; paksha 0; gündüz–gece 0.
        let mars = temporal_bala(Mars, &p, &a, &lords);
        assert_eq!((mars.month_lord, mars.hora_lord), (30.0, 60.0));
        assert!(close(mars.total, 30.0 + 60.0 + 30.0));
        // Jüpiter tribhaga balayı her zaman alır.
        let jupiter = temporal_bala(Jupiter, &p, &a, &lords);
        assert_eq!(jupiter.tribhaga, 60.0);
        assert!(close(jupiter.total, 60.0 + 60.0 + 60.0 + 30.0));
    }
}
