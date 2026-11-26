//! Shadbala ve Bhava bala (ileri modüller spec'i §7). Yedi grahanın altı gücü (sthana, dig,
//! zaman gücü, cheshta, naisargika, drik), toplamı, gerekli en az değere oranı ve sırası; on
//! iki bhavanın gücü. Kaynak: BPHS'nin graha gücü ve bhava gücü bölümleri. Birim virupadır
//! (60 virupa = 1 rupa).
//!
//! Hesap fonksiyonları ephemeris görmez (K3): `shadbala` yalnızca `Positions` ve
//! `ShadbalaAstro` alır. Ek A: üçüncü güç yalnızca "zaman gücü" (`temporal`) adıyla anılır.

mod bhava;
mod cheshta;
mod sthana;
mod temporal;

use serde::{Deserialize, Serialize};

use crate::chart::ChartWarning;
use crate::drishti::{sphuta_drishti, ASPECTING_GRAHAS};
use crate::graha::Graha;
use crate::nature::{nature, Nature};
use crate::panchang::Vara;
use crate::positions::Positions;
use crate::zodiac::{angular_distance, normalize};

pub use bhava::{bhava_balas, bhava_dig_bala, bhava_midpoint, bhava_sign_kind, BhavaSignKind};
pub use cheshta::{
    cheshta_bala, cheshta_from, circular_midpoint, mean_longitudes, MeanLongitudes, MEAN_JUPITER,
    MEAN_MARS, MEAN_MERCURY, MEAN_SATURN, MEAN_SUN, MEAN_VENUS,
};
pub use sthana::{
    drekkana_bala, kendradi_bala, ojayugma_bala, rasi_lon, saptavargaja_bala, saptavargaja_points,
    sthana_bala, uccha_bala, SAPTAVARGA,
};
pub use temporal::{
    ayana_bala, ayana_declination, day_night_bala, hora_index, hora_lord, hours_from_noon,
    is_daytime, paksha_bala, paksha_elongation, temporal_bala, time_lords, tribhaga_lord,
    vara_lord, AYANA_OBLIQUITY_DEG, DAY_TRIBHAGA, HORA_CYCLE, HORA_LORD_BALA, MONTH_LORD_BALA,
    NIGHT_TRIBHAGA, WEEKDAY_LORD_BALA, YEAR_LORD_BALA,
};

/// `ShadbalaReport::method`: bu modüldeki yöntemin kimliği.
pub const SHADBALA_METHOD: &str = "shadbala.bphs_v1";
/// Shadbala hesaplanan yedi graha; `ShadbalaReport::grahas` bu sıradadır (`Graha::ALL`'un ilk
/// yedisi).
pub const SHADBALA_GRAHAS: [Graha; 7] = ASPECTING_GRAHAS;
/// 60 virupa = 1 rupa.
pub const VIRUPAS_PER_RUPA: f64 = 60.0;
/// Graha savaşına girebilen beş graha (Güneş ve Ay girmez).
pub const WAR_GRAHAS: [Graha; 5] = [
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];
/// Graha savaşı: iki grahanın boylam farkı en çok bu kadar derecedir (sınır dahil).
pub const GRAHA_WAR_ORB_DEG: f64 = 1.0;

/// Sthana bala (konum gücü) kalemleri, virupa.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SthanaBala {
    pub uccha: f64,
    pub saptavargaja: f64,
    pub ojayugma: f64,
    pub kendradi: f64,
    pub drekkana: f64,
    pub total: f64,
}

/// Zaman gücü kalemleri, virupa. `ayana` Güneş'te iki katıdır.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TemporalBala {
    pub day_night: f64,
    pub paksha: f64,
    pub tribhaga: f64,
    pub year_lord: f64,
    pub month_lord: f64,
    pub weekday_lord: f64,
    pub hora_lord: f64,
    pub ayana: f64,
    pub total: f64,
}

/// Graha savaşı işareti: `with` karşıdaki graha, `winner` kazanan. Güç aktarımı uygulanmaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GrahaWar {
    pub with: Graha,
    pub winner: Graha,
}

/// Bir grahanın Shadbala sonucu, virupa.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GrahaShadbala {
    pub graha: Graha,
    pub sthana: SthanaBala,
    pub dig: f64,
    pub temporal: TemporalBala,
    pub cheshta: f64,
    pub naisargika: f64,
    /// Negatif olabilir.
    pub drik: f64,
    /// Altı gücün toplamı.
    pub total: f64,
    /// `total / 60`.
    pub rupas: f64,
    /// Klasik gerekli en az değer, virupa.
    pub required: f64,
    /// `total / required`.
    pub ratio: f64,
    /// Yedi graha arasında toplamın sırası: 1 en güçlü.
    pub rank: u8,
    pub war: Option<GrahaWar>,
}

/// Zaman gücünde değer alan beş lord.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TimeLords {
    pub year: Graha,
    pub month: Graha,
    pub weekday: Graha,
    pub hora: Graha,
    pub tribhaga: Graha,
}

/// Bir bhavanın gücü, virupa.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct BhavaBala {
    /// 1..=12.
    pub house: u8,
    pub lord: Graha,
    /// Bhavadhipati bala: lordun Shadbala toplamı.
    pub lord_bala: f64,
    pub dig: f64,
    /// Negatif olabilir.
    pub drishti: f64,
    pub total: f64,
    /// On iki bhava arasında toplamın sırası: 1 en güçlü.
    pub rank: u8,
}

/// `compute_shadbala` sonucu. AI yorum paneli için sözleşmedir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ShadbalaReport {
    /// `SHADBALA_GRAHAS` sırasında.
    pub grahas: [GrahaShadbala; 7],
    pub time_lords: TimeLords,
    /// 1.–12. bhava.
    pub bhavas: [BhavaBala; 12],
    pub method: String,
    pub warnings: Vec<ChartWarning>,
}

/// Saf hesabın astronomik girdisi (K3): toplayıcı ephemeris'ten üretir, el örnekleri elle
/// yazar. Zamanlar UT Jülyen günüdür; hesap yalnızca farklarını kullanır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShadbalaAstro {
    /// Doğum anı.
    pub jd_ut: f64,
    /// Doğumdan önceki son gün doğumu (doğum anı dahil).
    pub sunrise_jd_ut: f64,
    /// O gün doğumundan sonraki ilk gün batımı.
    pub sunset_jd_ut: f64,
    /// Sonraki gün doğumu (doğumdan sonra).
    pub next_sunrise_jd_ut: f64,
    /// MC'nin sidereal boylamı (astronomik açı; ev sistemi ayarından bağımsız).
    pub mc_lon: f64,
    /// Yedi grahanın tropikal jeosantrik boylamı, `SHADBALA_GRAHAS` sırasında.
    pub tropical_lons: [f64; 7],
    /// Doğum anındaki tropikal ortalama boylamlar.
    pub mean: MeanLongitudes,
    /// Doğum gününün varası (gün doğumunda değişir).
    pub weekday: Vara,
    /// Güneş'in doğumdan önceki son sidereal Koç girişinin gerçekleştiği vara.
    pub year_entry_weekday: Vara,
    /// Güneş'in bulunduğu sidereal burca girişinin gerçekleştiği vara.
    pub month_entry_weekday: Vara,
}

impl ShadbalaAstro {
    /// Değerler sonlu ve sıralı mı: doğuş ≤ doğum < sonraki doğuş, doğuş < batış < sonraki
    /// doğuş.
    pub fn is_consistent(&self) -> bool {
        let m = &self.mean;
        let finite = [
            self.jd_ut,
            self.sunrise_jd_ut,
            self.sunset_jd_ut,
            self.next_sunrise_jd_ut,
            self.mc_lon,
            m.sun,
            m.mercury,
            m.venus,
            m.mars,
            m.jupiter,
            m.saturn,
        ]
        .into_iter()
        .chain(self.tropical_lons)
        .all(f64::is_finite);
        finite
            && self.sunrise_jd_ut <= self.jd_ut
            && self.jd_ut < self.next_sunrise_jd_ut
            && self.sunrise_jd_ut < self.sunset_jd_ut
            && self.sunset_jd_ut < self.next_sunrise_jd_ut
    }
}

/// Klasik gerekli en az Shadbala, virupa. Düğümler 0.
pub fn required_bala(graha: Graha) -> f64 {
    match graha {
        Graha::Sun | Graha::Jupiter => 390.0,
        Graha::Moon => 360.0,
        Graha::Mars | Graha::Saturn => 300.0,
        Graha::Mercury => 420.0,
        Graha::Venus => 330.0,
        Graha::Rahu | Graha::Ketu => 0.0,
    }
}

/// Naisargika bala (doğal güç): sabit, 60'ın yedide birinin katları. Güneş 60; Ay 51,43; Venüs
/// 42,86; Jüpiter 34,29; Merkür 25,71; Mars 17,14; Satürn 8,57. Düğümler 0.
pub fn naisargika_bala(graha: Graha) -> f64 {
    let sevenths = match graha {
        Graha::Sun => 7.0,
        Graha::Moon => 6.0,
        Graha::Venus => 5.0,
        Graha::Jupiter => 4.0,
        Graha::Mercury => 3.0,
        Graha::Mars => 2.0,
        Graha::Saturn => 1.0,
        Graha::Rahu | Graha::Ketu => 0.0,
    };
    60.0 * sevenths / 7.0
}

/// Dig bala (yön gücü): güçsüz noktadan açısal uzaklık (0°–180°) / 3. Güçsüz nokta: Güneş ve
/// Mars için IC (MC + 180°), Ay ve Venüs için MC, Merkür ve Jüpiter için alçalan (lagna +
/// 180°), Satürn için lagna. Üç boylam da aynı çerçevededir (sidereal). Düğümler 0.
pub fn dig_bala(graha: Graha, lon: f64, lagna_lon: f64, mc_lon: f64) -> f64 {
    let powerless = match graha {
        Graha::Sun | Graha::Mars => mc_lon + 180.0,
        Graha::Moon | Graha::Venus => mc_lon,
        Graha::Mercury | Graha::Jupiter => lagna_lon + 180.0,
        Graha::Saturn => lagna_lon,
        Graha::Rahu | Graha::Ketu => return 0.0,
    };
    angular_distance(lon, normalize(powerless)) / 3.0
}

/// Bir noktaya gelen bakış gücü: yedi grahanın (`skip` dışında) sphuta drishti değerleriyle
/// `(Σ iyicil − Σ kötücül) / 4`, artı Merkür'ün ve Jüpiter'in bakış değerlerinin tamamı.
/// İyicil/kötücül sınıfı `nature`'dır (K6). `target_lon` sidereal boylamdır.
pub fn drik_bala_at(target_lon: f64, skip: Option<Graha>, positions: &Positions) -> f64 {
    let mut sum = 0.0;
    for graha in SHADBALA_GRAHAS {
        if Some(graha) == skip {
            continue;
        }
        let value = sphuta_drishti(graha, positions.point(graha).sidereal_lon, target_lon);
        sum += match nature(graha, positions) {
            Nature::Benefic => value,
            Nature::Malefic => -value,
        } / 4.0;
        if matches!(graha, Graha::Mercury | Graha::Jupiter) {
            sum += value;
        }
    }
    sum
}

/// Drik bala (bakış gücü): grahaya diğer altı grahadan gelen bakışlar. Negatif olabilir.
pub fn drik_bala(graha: Graha, positions: &Positions) -> f64 {
    drik_bala_at(positions.point(graha).sidereal_lon, Some(graha), positions)
}

/// Graha savaşı: `graha` beş grahadan biriyse ve bir başkası `GRAHA_WAR_ORB_DEG` içindeyse
/// işaretlenir. Karşıdaki graha en yakın olandır (eşitlikte `WAR_GRAHAS` sırasında önce
/// gelen); kazanan ekliptik enlemi daha kuzeyde olandır (eşit enlemde `WAR_GRAHAS` sırasında
/// önce gelen).
pub fn graha_war(graha: Graha, positions: &Positions) -> Option<GrahaWar> {
    let place = |g: Graha| WAR_GRAHAS.iter().position(|w| *w == g);
    let own = place(graha)?;
    let me = positions.point(graha);
    let mut nearest: Option<(Graha, f64)> = None;
    for other in WAR_GRAHAS {
        if other == graha {
            continue;
        }
        let gap = angular_distance(me.sidereal_lon, positions.point(other).sidereal_lon);
        if gap <= GRAHA_WAR_ORB_DEG && nearest.is_none_or(|(_, best)| gap < best) {
            nearest = Some((other, gap));
        }
    }
    let (with, _) = nearest?;
    let other = positions.point(with);
    let wins = me.lat > other.lat || (me.lat == other.lat && Some(own) < place(with));
    Some(GrahaWar {
        with,
        winner: if wins { graha } else { with },
    })
}

/// Toplamların sırası: en büyük 1. Eşit toplamlarda dizide önce gelen öndedir.
pub(crate) fn ranks<const N: usize>(totals: [f64; N]) -> [u8; N] {
    let mut order: [usize; N] = std::array::from_fn(|i| i);
    // `sort_by` kararlıdır: eşitler dizi sırasını korur.
    order.sort_by(|a, b| totals[*b].total_cmp(&totals[*a]));
    let mut out = [0u8; N];
    for (rank, index) in order.into_iter().enumerate() {
        out[index] = rank as u8 + 1;
    }
    out
}

/// Saf hesap: konumlardan ve astronomik girdiden Shadbala ve Bhava bala. Lagna yoksa (doğum
/// saati bilinmiyor) ya da `astro` tutarsızsa `None`. `warnings` boştur; toplayıcı doldurur.
pub fn shadbala(positions: &Positions, astro: &ShadbalaAstro) -> Option<ShadbalaReport> {
    let lagna_lon = positions.lagna_lon?;
    if !astro.is_consistent() {
        return None;
    }
    let lords = time_lords(astro);
    let mut rows = Vec::with_capacity(7);
    for graha in SHADBALA_GRAHAS {
        let sthana = sthana_bala(graha, positions)?;
        let dig = dig_bala(
            graha,
            positions.point(graha).sidereal_lon,
            lagna_lon,
            astro.mc_lon,
        );
        let temporal = temporal_bala(graha, positions, astro, &lords);
        let cheshta = cheshta_bala(graha, positions, astro);
        let naisargika = naisargika_bala(graha);
        let drik = drik_bala(graha, positions);
        let total = sthana.total + dig + temporal.total + cheshta + naisargika + drik;
        let required = required_bala(graha);
        rows.push(GrahaShadbala {
            graha,
            sthana,
            dig,
            temporal,
            cheshta,
            naisargika,
            drik,
            total,
            rupas: total / VIRUPAS_PER_RUPA,
            required,
            ratio: total / required,
            rank: 0,
            war: graha_war(graha, positions),
        });
    }
    let order = ranks(std::array::from_fn::<f64, 7, _>(|i| rows[i].total));
    for (row, rank) in rows.iter_mut().zip(order) {
        row.rank = rank;
    }
    let grahas: [GrahaShadbala; 7] = rows.try_into().ok()?;
    let bhavas = bhava_balas(positions, &grahas)?;
    Some(ShadbalaReport {
        grahas,
        time_lords: lords,
        bhavas,
        method: SHADBALA_METHOD.to_owned(),
        warnings: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    const BASE: [f64; 9] = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];

    fn sky(lons: [f64; 9]) -> Positions {
        Positions::synthetic(Some(0.0), lons)
    }

    fn astro() -> ShadbalaAstro {
        ShadbalaAstro {
            jd_ut: 0.5,
            sunrise_jd_ut: 0.25,
            sunset_jd_ut: 0.75,
            next_sunrise_jd_ut: 1.25,
            mc_lon: 270.0,
            tropical_lons: [34.0, 124.0, 244.0, 89.0, 274.0, 64.0, 324.0],
            mean: MeanLongitudes {
                sun: 30.0,
                mercury: 200.0,
                venus: 100.0,
                mars: 240.0,
                jupiter: 270.0,
                saturn: 320.0,
            },
            weekday: Vara::Sunday,
            year_entry_weekday: Vara::Thursday,
            month_entry_weekday: Vara::Friday,
        }
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn dig_is_zero_at_the_powerless_point_and_sixty_opposite() {
        // Lagna 0°, MC 270°: IC 90°, alçalan 180°.
        for (g, powerless) in [
            (Sun, 90.0),
            (Mars, 90.0),
            (Moon, 270.0),
            (Venus, 270.0),
            (Mercury, 180.0),
            (Jupiter, 180.0),
            (Saturn, 0.0),
        ] {
            assert!(close(dig_bala(g, powerless, 0.0, 270.0), 0.0), "{g:?}");
            assert!(
                close(dig_bala(g, powerless + 180.0, 0.0, 270.0), 60.0),
                "{g:?}"
            );
            assert!(
                close(dig_bala(g, powerless + 90.0, 0.0, 270.0), 30.0),
                "{g:?}"
            );
            assert!(
                close(dig_bala(g, powerless - 45.0, 0.0, 270.0), 15.0),
                "{g:?}"
            );
        }
        // MC lagnadan 90° geride olmak zorunda değildir: Güneş MC'de (250°) 60 alır.
        assert!(close(dig_bala(Sun, 250.0, 0.0, 250.0), 60.0));
        assert!(close(dig_bala(Moon, 70.0, 0.0, 250.0), 60.0));
        assert_eq!(dig_bala(Rahu, 10.0, 0.0, 270.0), 0.0);
    }

    #[test]
    fn naisargika_is_fixed_and_required_values_are_the_classical_ones() {
        let rounded: Vec<f64> = SHADBALA_GRAHAS
            .into_iter()
            .map(|g| (naisargika_bala(g) * 100.0).round() / 100.0)
            .collect();
        // Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn.
        assert_eq!(rounded, [60.0, 51.43, 17.14, 25.71, 34.29, 42.86, 8.57]);
        let required: Vec<f64> = SHADBALA_GRAHAS.into_iter().map(required_bala).collect();
        assert_eq!(required, [390.0, 360.0, 300.0, 420.0, 390.0, 330.0, 300.0]);
        assert_eq!((naisargika_bala(Rahu), required_bala(Ketu)), (0.0, 0.0));
    }

    #[test]
    fn drik_adds_a_quarter_of_each_aspect_and_all_of_mercury_and_jupiter() {
        // Hedef 0°. Yalnızca Jüpiter bakar: 180° geride (tam bakış 60); diğerleri hedefin 10°
        // gerisinde (açı 10° → 0) durur.
        let mut lons = [350.0; 9];
        lons[Jupiter as usize] = 180.0;
        let p = sky(lons);
        // İyicil: 60 / 4 = 15; Jüpiter'in tamamı: +60 → 75.
        assert!(close(drik_bala_at(0.0, None, &p), 75.0));
        // Yalnızca Satürn bakar (açı 180° → 60): kötücül, −15.
        let mut lons = [350.0; 9];
        lons[Saturn as usize] = 180.0;
        assert!(close(drik_bala_at(0.0, None, &sky(lons)), -15.0));
        // Yalnızca Venüs bakar: +15. Yalnızca Güneş: −15.
        let mut lons = [350.0; 9];
        lons[Venus as usize] = 180.0;
        assert!(close(drik_bala_at(0.0, None, &sky(lons)), 15.0));
        let mut lons = [350.0; 9];
        lons[Sun as usize] = 180.0;
        assert!(close(drik_bala_at(0.0, None, &sky(lons)), -15.0));
        // Yalnız Merkür (iyicil): 15 + 60 = 75. Merkür Satürn'le aynı burçtaysa kötücüldür:
        // Merkür −15 + 60, Satürn −15 → 30.
        let mut lons = [350.0; 9];
        lons[Mercury as usize] = 180.0;
        assert!(close(drik_bala_at(0.0, None, &sky(lons)), 75.0));
        lons[Saturn as usize] = 180.0;
        assert!(close(drik_bala_at(0.0, None, &sky(lons)), 30.0));
        // Büyüyen Ay iyicil (+15), küçülen Ay kötücül (−15). Güneş 350°'de.
        let mut lons = [350.0; 9];
        lons[Moon as usize] = 120.0; // uzanım 130°: büyüyor; açı 240° → (300 − 240) / 2 = 30
        assert!(close(drik_bala_at(0.0, None, &sky(lons)), 7.5));
        lons[Moon as usize] = 240.0; // uzanım 250°: küçülüyor; açı 120° → 30
        assert!(close(drik_bala_at(0.0, None, &sky(lons)), -7.5));
        // Grahanın kendisi sayılmaz: Jüpiter'e yalnızca kendisi "bakıyorsa" 0.
        let mut lons = [170.0; 9];
        lons[Jupiter as usize] = 180.0;
        assert!(close(drik_bala(Jupiter, &sky(lons)), 0.0));
    }

    #[test]
    fn graha_war_marks_both_sides_and_the_northern_one_wins() {
        // Mars 100°, Venüs 100,6°: savaş. Enlem: Mars +1,0, Venüs +0,5 → Mars kazanır.
        let mut lons = BASE;
        lons[Mars as usize] = 100.0;
        lons[Venus as usize] = 100.6;
        let mut p = sky(lons);
        p.grahas[Mars as usize].lat = 1.0;
        p.grahas[Venus as usize].lat = 0.5;
        let war = Some(GrahaWar {
            with: Venus,
            winner: Mars,
        });
        assert_eq!(graha_war(Mars, &p), war);
        assert_eq!(
            graha_war(Venus, &p),
            Some(GrahaWar {
                with: Mars,
                winner: Mars
            })
        );
        // Güneyde olan kaybeder.
        p.grahas[Mars as usize].lat = -1.0;
        assert_eq!(graha_war(Mars, &p).map(|w| w.winner), Some(Venus));
        // Eşit enlem: `WAR_GRAHAS` sırasında önce gelen (Mars).
        p.grahas[Mars as usize].lat = 0.5;
        assert_eq!(graha_war(Venus, &p).map(|w| w.winner), Some(Mars));
        // Sınır dahil: tam 1° savaş, 1°'den büyük değil.
        let mut lons = BASE;
        lons[Mars as usize] = 100.0;
        lons[Venus as usize] = 101.0;
        assert!(graha_war(Mars, &sky(lons)).is_some());
        lons[Venus as usize] = 101.001;
        assert_eq!(graha_war(Mars, &sky(lons)), None);
        // 0°'nin iki yanı: 359,7° ile 0,2°.
        let mut lons = BASE;
        lons[Jupiter as usize] = 359.7;
        lons[Saturn as usize] = 0.2;
        assert_eq!(graha_war(Saturn, &sky(lons)).map(|w| w.with), Some(Jupiter));
        // Güneş, Ay ve düğümler savaşa girmez, yakın olsalar da.
        let mut lons = BASE;
        lons[Sun as usize] = 100.0;
        lons[Moon as usize] = 100.1;
        lons[Rahu as usize] = 100.2;
        lons[Mars as usize] = 100.3;
        let p = sky(lons);
        for g in [Sun, Moon, Rahu, Ketu, Mars] {
            assert_eq!(graha_war(g, &p), None, "{g:?}");
        }
        // Üç graha 1° içinde: karşıdaki en yakın olandır.
        let mut lons = BASE;
        lons[Mars as usize] = 100.0;
        lons[Mercury as usize] = 100.9;
        lons[Jupiter as usize] = 100.5;
        let p = sky(lons);
        assert_eq!(graha_war(Mars, &p).map(|w| w.with), Some(Jupiter));
        assert_eq!(graha_war(Mercury, &p).map(|w| w.with), Some(Jupiter));
        assert_eq!(graha_war(Jupiter, &p).map(|w| w.with), Some(Mercury));
    }

    #[test]
    fn ranks_order_by_total_and_keep_array_order_on_ties() {
        assert_eq!(ranks([10.0, 30.0, 20.0]), [3, 1, 2]);
        assert_eq!(ranks([5.0, 5.0, 7.0, 5.0]), [2, 3, 1, 4]);
        assert_eq!(ranks([-1.0, -3.0]), [1, 2]);
    }

    #[test]
    fn report_sums_the_six_strengths_and_ranks_grahas_and_bhavas() {
        let p = sky(BASE);
        let a = astro();
        let r = shadbala(&p, &a).expect("lagna and astro");
        assert_eq!(r.method, SHADBALA_METHOD);
        assert!(r.warnings.is_empty());
        assert_eq!(r.time_lords, time_lords(&a));
        let mut seen = [false; 7];
        for (row, g) in r.grahas.iter().zip(SHADBALA_GRAHAS) {
            assert_eq!(row.graha, g);
            let sum = row.sthana.total
                + row.dig
                + row.temporal.total
                + row.cheshta
                + row.naisargika
                + row.drik;
            assert!(close(row.total, sum), "{g:?}");
            assert!(close(row.rupas, row.total / 60.0), "{g:?}");
            assert_eq!(row.required, required_bala(g));
            assert!(close(row.ratio, row.total / row.required), "{g:?}");
            assert_eq!(row.sthana, sthana_bala(g, &p).expect("lagna"));
            assert_eq!(row.war, None);
            seen[usize::from(row.rank) - 1] = true;
        }
        assert!(seen.iter().all(|s| *s), "ranks are a permutation of 1..=7");
        // Sıra toplamı izler.
        for a in &r.grahas {
            for b in &r.grahas {
                if a.total > b.total {
                    assert!(a.rank < b.rank, "{:?} before {:?}", a.graha, b.graha);
                }
            }
        }
        // Bhavalar: lordun toplamı + dig + drishti; sıra 1..=12.
        let mut seen = [false; 12];
        for (i, b) in r.bhavas.iter().enumerate() {
            assert_eq!(usize::from(b.house), i + 1);
            assert_eq!(b.lord_bala, r.grahas[b.lord as usize].total);
            assert!(close(b.total, b.lord_bala + b.dig + b.drishti));
            seen[usize::from(b.rank) - 1] = true;
        }
        assert!(seen.iter().all(|s| *s), "bhava ranks are a permutation");
        // Lagna Koç 0°: 1. bhava dört ayaklı (güçsüz bhava 4; 90° uzakta) → 30; 8. bhava Akrep
        // (güçsüz bhava 1; 210° → kısa yay 150°) → 50.
        assert_eq!(r.bhavas[0].dig, 30.0);
        assert_eq!(r.bhavas[7].dig, 50.0);
    }

    #[test]
    fn no_report_without_a_lagna_or_with_inconsistent_times() {
        let p = sky(BASE);
        assert_eq!(shadbala(&p.clone().without_lagna(), &astro()), None);
        // Doğum, sonraki gün doğumundan sonra.
        let late = ShadbalaAstro {
            jd_ut: 1.3,
            ..astro()
        };
        assert!(!late.is_consistent());
        assert_eq!(shadbala(&p, &late), None);
        // Batış doğuştan önce.
        let no_day = ShadbalaAstro {
            sunset_jd_ut: 0.2,
            ..astro()
        };
        assert_eq!(shadbala(&p, &no_day), None);
        let nan = ShadbalaAstro {
            mc_lon: f64::NAN,
            ..astro()
        };
        assert_eq!(shadbala(&p, &nan), None);
        assert!(astro().is_consistent());
        // Doğum tam gün doğumunda: tutarlı.
        let at_rise = ShadbalaAstro {
            jd_ut: 0.25,
            ..astro()
        };
        assert!(at_rise.is_consistent());
    }
}
