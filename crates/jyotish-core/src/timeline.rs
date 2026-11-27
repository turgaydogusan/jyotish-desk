//! Zaman çizelgesi çekirdeği (ileri modüller spec'i §10): bir zaman aralığı için ardışık,
//! boşluksuz `[start, end)` aralık dizileri. Gün panchangı, pencere araması (Alt proje 3.7) ve
//! olay araması (3.9) bu dizileri kullanır.
//!
//! Yöntem tek bir yürüyücüdür (`walk`): değer sabit adımlarla örneklenir; iki örnek arasında
//! değer değişmişse değişim anı ikiye bölmeyle (`TIMELINE_RESOLUTION_DAYS`) bulunur. Değer
//! v1'in panchang fonksiyonlarıyla (`tithi_no`, `karana_no`, `nakshatra_pada`, `nitya_yoga_no`,
//! `Sign::of`) aynı ephemeris sorgularından hesaplanır: bir aralığın içindeki her anda
//! `compute_panchang` aynı değeri verir. Sınır, değerin değiştiği ilk örnektir (yeni değer
//! aralığın başında geçerlidir); gerçek sınır en çok bir çözünürlük adımı öncedir.
//!
//! Dizi `[from, to]` aralığına kırpılır: ilk aralık `from`'da başlar, sonuncusu `to`'da biter.
//! Zamanlar UT Jülyen günüdür. Burç dizileri rasi çerçevesindedir (K5), nakshatra ve yoga
//! dizileri her kipte sidereal boylamdandır.

use ephemeris::{Body, Ephemeris, RiseSet, SunDisc, Zodiac};
use serde::{Deserialize, Serialize};

use crate::chart::{finite, rasi_frame};
use crate::combustion::is_combust;
use crate::graha::Graha;
use crate::panchang::{
    day_sunrise, karana_no, local_weekday, nitya_yoga_no, vara_at, Vara, RISE_STEP_DAYS,
    SUNRISE_WINDOW_DAYS,
};
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::zodiac::{nakshatra_pada, normalize, Sign};
use crate::CoreError;

/// Sınır aramasının çözünürlüğü: 0,05 saniye.
pub const TIMELINE_RESOLUTION_DAYS: f64 = 0.05 / 86_400.0;
/// Ay'a bağlı dizilerin örnekleme adımı. En kısa dilim (pada, 3°20′) ~5 saat sürer; adım
/// yalnızca tam bir çevrimden (en kısası ~27 gün) kısa olmak zorundadır.
pub const MOON_STEP_DAYS: f64 = 0.5;
/// Güneş burcu, graha burcu, graha nakshatrası, hareket yönü ve yanma dizilerinin adımı.
pub const GRAHA_STEP_DAYS: f64 = 1.0;
/// Lagna burcu dizisinin adımı (30 dakika): lagna bir günde on iki burcu dolaşır.
pub const LAGNA_STEP_DAYS: f64 = 1.0 / 48.0;
/// Vara dizisinin yavaş yolundaki adım (gün doğumu olmayan günler).
pub const VARA_STEP_DAYS: f64 = 0.25;
/// Tek bir sınır için en çok ikiye bölme adımı (1 gün / 2^21 ≈ 0,04 sn; pay bırakılmıştır).
const MAX_BISECTIONS: u32 = 40;
/// Bir örnekleme adımının içinde bulunabilecek en çok değişim (güvenlik sınırı).
const MAX_CHANGES_PER_STEP: usize = 64;

/// `[start_jd_ut, end_jd_ut)` boyunca geçerli değer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Interval<T> {
    pub value: T,
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
}

impl<T> Interval<T> {
    pub fn contains(&self, jd_ut: f64) -> bool {
        self.start_jd_ut <= jd_ut && jd_ut < self.end_jd_ut
    }

    pub fn days(&self) -> f64 {
        self.end_jd_ut - self.start_jd_ut
    }
}

/// Ay'ın nakshatrası (1..=27) ve padası (1..=4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct NakshatraPada {
    pub nakshatra: u8,
    pub pada: u8,
}

/// Grahanın boylamca hareket yönü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Motion {
    Direct,
    Retrograde,
}

/// Hareket yönü dizisi verilen grahalar: retro olabilen beş graha. Güneş ve Ay hep düzdür;
/// düğümlerin yönü olay sayılmaz (ortalama düğüm hep geri gider, gerçek düğüm birkaç günde
/// bir yön değiştirir).
pub const STATION_GRAHAS: [Graha; 5] = [
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// Güneş'in doğuşu, batışı ve sonraki doğuşu bilinen gün (gün doğumu → gün doğumu).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SunDay {
    pub sunrise_jd_ut: f64,
    pub sunset_jd_ut: f64,
    pub next_sunrise_jd_ut: f64,
    /// Gün doğumunun yerel ortalama tarihinin hafta günü (`compute_panchang` ile aynı kural).
    pub vara: Vara,
}

/// Aralık denetimi: iki uç da hesap aralığında ve `from < to`.
pub(crate) fn check_span(from_jd_ut: f64, to_jd_ut: f64) -> Result<(), CoreError> {
    for jd in [from_jd_ut, to_jd_ut] {
        // NaN ve sonsuz da aralık dışındadır.
        if !JD_UT_RANGE.contains(&jd) {
            return Err(CoreError::InvalidMuhurtaInput("range"));
        }
    }
    if from_jd_ut >= to_jd_ut {
        return Err(CoreError::InvalidMuhurtaInput("range"));
    }
    Ok(())
}

fn check_place(geo_lat: f64, geo_lon: f64) -> Result<(), CoreError> {
    if (-90.0..=90.0).contains(&geo_lat) && (-180.0..=180.0).contains(&geo_lon) {
        Ok(())
    } else {
        Err(CoreError::InvalidMuhurtaInput("place"))
    }
}

/// Yürüyücü: `value`'nun `[from, to]` boyunca aldığı değerler, değişim anlarıyla. `step`
/// örnekleme adımıdır; bir adımın içinde değer değişip aynı değere dönerse o kısa gidiş
/// görülmez (çağıran adımı buna göre seçer).
pub(crate) fn walk<E, V, F>(
    eph: &mut E,
    from: f64,
    to: f64,
    step: f64,
    mut value: F,
) -> Result<Vec<Interval<V>>, CoreError>
where
    E: Ephemeris + ?Sized,
    V: Copy + PartialEq,
    F: FnMut(&mut E, f64) -> Result<V, CoreError>,
{
    check_span(from, to)?;
    let mut out = Vec::new();
    let mut start = from;
    let mut current = value(eph, from)?;
    let mut t = from;
    while t < to {
        let next = (t + step).min(to);
        let at_next = value(eph, next)?;
        let mut lo = t;
        let mut changes = 0;
        while current != at_next {
            changes += 1;
            if changes > MAX_CHANGES_PER_STEP {
                return Err(CoreError::PanchangSearch("timeline"));
            }
            // `lo`'da değer `current`, `hi`'de `at_hi` (başka bir değer).
            let (mut hi, mut at_hi) = (next, at_next);
            let mut steps = 0;
            while hi - lo > TIMELINE_RESOLUTION_DAYS && steps < MAX_BISECTIONS {
                let mid = 0.5 * (lo + hi);
                let at_mid = value(eph, mid)?;
                if at_mid == current {
                    lo = mid;
                } else {
                    hi = mid;
                    at_hi = at_mid;
                }
                steps += 1;
            }
            out.push(Interval {
                value: current,
                start_jd_ut: start,
                end_jd_ut: hi,
            });
            current = at_hi;
            start = hi;
            lo = hi;
        }
        t = next;
    }
    out.push(Interval {
        value: current,
        start_jd_ut: start,
        end_jd_ut: to,
    });
    Ok(out)
}

/// Değerleri `f` ile çevirir ve aynı değerli ardışık aralıkları birleştirir.
pub fn map_merge<T, U: PartialEq>(
    series: &[Interval<T>],
    mut f: impl FnMut(&T) -> U,
) -> Vec<Interval<U>> {
    let mut out: Vec<Interval<U>> = Vec::new();
    for i in series {
        let value = f(&i.value);
        match out.last_mut() {
            Some(last) if last.value == value && last.end_jd_ut == i.start_jd_ut => {
                last.end_jd_ut = i.end_jd_ut;
            }
            _ => out.push(Interval {
                value,
                start_jd_ut: i.start_jd_ut,
                end_jd_ut: i.end_jd_ut,
            }),
        }
    }
    out
}

/// `jd_ut` anını içeren aralığın değeri; an dizinin dışındaysa `None`.
pub fn value_at<T: Copy>(series: &[Interval<T>], jd_ut: f64) -> Option<T> {
    let i = series.partition_point(|x| x.end_jd_ut <= jd_ut);
    series.get(i).filter(|x| x.contains(jd_ut)).map(|x| x.value)
}

fn sidereal(settings: &CalcSettings) -> Zodiac {
    Zodiac::Sidereal(settings.ayanamsa.sidereal_mode())
}

fn lon_of<E: Ephemeris + ?Sized>(
    eph: &mut E,
    jd_ut: f64,
    body: Body,
    zodiac: Zodiac,
) -> Result<f64, CoreError> {
    finite(eph.body(jd_ut, body, zodiac)?.lon, "timeline")
}

/// Karana (1..=60) dizisi: Ay − Güneş uzanımının 6°'lik dilimleri.
pub fn karana_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<u8>>, CoreError> {
    settings.validate()?;
    let zodiac = sidereal(settings);
    walk(eph, from_jd_ut, to_jd_ut, MOON_STEP_DAYS, |eph, jd| {
        let sun = lon_of(eph, jd, Body::Sun, zodiac)?;
        let moon = lon_of(eph, jd, Body::Moon, zodiac)?;
        Ok(karana_no(normalize(moon - sun)))
    })
}

/// Karananın ait olduğu tithi (1..=30): her tithi iki karanadır.
pub fn tithi_of_karana(karana: u8) -> u8 {
    karana.div_ceil(2).clamp(1, 30)
}

/// Bir karana dizisinden tithi dizisi (ardışık iki karana birleşir). Tithi sınırları karana
/// sınırlarının bir alt kümesidir; iki dizi aynı anları paylaşır.
pub fn tithis_of_karanas(karanas: &[Interval<u8>]) -> Vec<Interval<u8>> {
    map_merge(karanas, |k| tithi_of_karana(*k))
}

/// Tithi (1..=30) dizisi.
pub fn tithi_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<u8>>, CoreError> {
    Ok(tithis_of_karanas(&karana_series(
        eph, settings, from_jd_ut, to_jd_ut,
    )?))
}

/// Ay'ın nakshatra ve pada dizisi (sidereal boylam, her zodyak kipinde).
pub fn nakshatra_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<NakshatraPada>>, CoreError> {
    settings.validate()?;
    let zodiac = sidereal(settings);
    walk(eph, from_jd_ut, to_jd_ut, MOON_STEP_DAYS, |eph, jd| {
        let (nakshatra, pada) = nakshatra_pada(lon_of(eph, jd, Body::Moon, zodiac)?);
        Ok(NakshatraPada { nakshatra, pada })
    })
}

/// Nitya yoga (1..=27) dizisi: sidereal Güneş + Ay toplamı.
pub fn nitya_yoga_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<u8>>, CoreError> {
    settings.validate()?;
    let zodiac = sidereal(settings);
    walk(eph, from_jd_ut, to_jd_ut, MOON_STEP_DAYS, |eph, jd| {
        let sun = lon_of(eph, jd, Body::Sun, zodiac)?;
        let moon = lon_of(eph, jd, Body::Moon, zodiac)?;
        Ok(nitya_yoga_no(sun, moon))
    })
}

/// Grahanın ephemeris cismi ve boylamına eklenen açı (Ketu, Rahu'nun karşısıdır).
fn graha_body(graha: Graha, settings: &CalcSettings) -> (Body, f64) {
    match graha {
        Graha::Sun => (Body::Sun, 0.0),
        Graha::Moon => (Body::Moon, 0.0),
        Graha::Mars => (Body::Mars, 0.0),
        Graha::Mercury => (Body::Mercury, 0.0),
        Graha::Jupiter => (Body::Jupiter, 0.0),
        Graha::Venus => (Body::Venus, 0.0),
        Graha::Saturn => (Body::Saturn, 0.0),
        Graha::Rahu => (settings.node.body(), 0.0),
        Graha::Ketu => (settings.node.body(), 180.0),
    }
}

fn graha_step(graha: Graha) -> f64 {
    if graha == Graha::Moon {
        MOON_STEP_DAYS
    } else {
        GRAHA_STEP_DAYS
    }
}

/// Grahanın rasi çerçevesindeki burç dizisi; sınırlar burç girişleridir (retro girişler dahil).
/// `Graha::Moon` Ay burcu, `Graha::Sun` Güneş burcu dizisidir.
pub fn graha_sign_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    graha: Graha,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<Sign>>, CoreError> {
    settings.validate()?;
    let frame = rasi_frame(settings);
    let (body, shift) = graha_body(graha, settings);
    walk(eph, from_jd_ut, to_jd_ut, graha_step(graha), |eph, jd| {
        Ok(Sign::of(lon_of(eph, jd, body, frame)? + shift))
    })
}

/// Grahanın nakshatra (1..=27) dizisi (sidereal boylam); sınırlar nakshatra girişleridir.
pub fn graha_nakshatra_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    graha: Graha,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<u8>>, CoreError> {
    settings.validate()?;
    let zodiac = sidereal(settings);
    let (body, shift) = graha_body(graha, settings);
    walk(eph, from_jd_ut, to_jd_ut, graha_step(graha), |eph, jd| {
        Ok(nakshatra_pada(lon_of(eph, jd, body, zodiac)? + shift).0)
    })
}

/// Grahanın hareket yönü dizisi; sınırlar retro ve düz dönüş anlarıdır (boylam hızının sıfır
/// geçişi). Yalnızca `STATION_GRAHAS` için; diğerlerinde `InvalidMuhurtaInput("graha")`.
/// Retro tanımı v1 ile aynıdır: hız < 0.
pub fn graha_motion_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    graha: Graha,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<Motion>>, CoreError> {
    settings.validate()?;
    if !STATION_GRAHAS.contains(&graha) {
        return Err(CoreError::InvalidMuhurtaInput("graha"));
    }
    let frame = rasi_frame(settings);
    let (body, _) = graha_body(graha, settings);
    walk(eph, from_jd_ut, to_jd_ut, GRAHA_STEP_DAYS, |eph, jd| {
        let speed = finite(eph.body(jd, body, frame)?.lon_speed, "timeline")?;
        Ok(if speed < 0.0 {
            Motion::Retrograde
        } else {
            Motion::Direct
        })
    })
}

/// Grahanın yanık olup olmadığının dizisi (v1 yanma kuralı: `is_combust`, haritanın orblarıyla;
/// retro Merkür ve Venüs'ün orbu küçüktür, dönüş anında dizi değişebilir).
pub fn combust_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    graha: Graha,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<bool>>, CoreError> {
    settings.validate()?;
    let frame = rasi_frame(settings);
    let (body, shift) = graha_body(graha, settings);
    walk(eph, from_jd_ut, to_jd_ut, GRAHA_STEP_DAYS, |eph, jd| {
        let p = eph.body(jd, body, frame)?;
        let lon = finite(p.lon, "timeline")? + shift;
        let speed = finite(p.lon_speed, "timeline")?;
        let sun = lon_of(eph, jd, Body::Sun, frame)?;
        Ok(is_combust(
            graha,
            lon,
            sun,
            speed < 0.0,
            &settings.combustion_orbs,
        ))
    })
}

/// `jd_ut` anında yükselen burç (rasi çerçevesinde; `compute_chart`'ın lagnasıyla aynı sorgu).
pub fn lagna_sign_at<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    jd_ut: f64,
    geo_lat: f64,
    geo_lon: f64,
) -> Result<Sign, CoreError> {
    let houses = eph.houses(
        jd_ut,
        geo_lat,
        geo_lon,
        settings.house_system.ephemeris_system(),
        rasi_frame(settings),
    )?;
    Ok(Sign::of(finite(houses.asc, "lagna")?))
}

/// Lagna burcu dizisi: yükselen burcun değiştiği anlar.
pub fn lagna_sign_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    geo_lat: f64,
    geo_lon: f64,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<Sign>>, CoreError> {
    settings.validate()?;
    check_place(geo_lat, geo_lon)?;
    walk(eph, from_jd_ut, to_jd_ut, LAGNA_STEP_DAYS, |eph, jd| {
        lagna_sign_at(eph, settings, jd, geo_lat, geo_lon)
    })
}

fn rise_set<E: Ephemeris + ?Sized>(
    eph: &mut E,
    after: f64,
    event: RiseSet,
    geo_lat: f64,
    geo_lon: f64,
    disc: SunDisc,
) -> Result<Option<f64>, CoreError> {
    match eph.sun_rise_set(after, event, geo_lat, geo_lon, disc)? {
        Some(t) if !t.is_finite() => Err(CoreError::NonFiniteResult("sunrise")),
        Some(t) if t > after => Ok(Some(t)),
        _ => Ok(None),
    }
}

/// `[from, to]` ile kesişen, doğuşu, batışı ve sonraki doğuşu bilinen günler; sırayla. Ardışık
/// iki doğuş arası `SUNRISE_WINDOW_DAYS`'ten uzunsa ya da arada batış yoksa (kutup gecesi ya
/// da gündüzü) o gün listede yer almaz. İlk gün `from`'dan önce başlayabilir, sonuncusu
/// `to`'dan sonra bitebilir.
pub fn sun_days<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    geo_lat: f64,
    geo_lon: f64,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<SunDay>, CoreError> {
    settings.validate()?;
    check_span(from_jd_ut, to_jd_ut)?;
    check_place(geo_lat, geo_lon)?;
    let disc = settings.sunrise_def.sun_disc();
    let mut out = Vec::new();
    let mut rise = day_sunrise(eph, from_jd_ut, geo_lat, geo_lon, disc)?;
    // Bilinen doğuş yokken ileriye doğru aramanın başladığı an.
    let mut cursor = from_jd_ut;
    loop {
        let Some(r) = rise else {
            if cursor >= to_jd_ut {
                break;
            }
            // SE'nin yüksek enlem araması ~1 gün ileriye bakar: doğuş yoksa bir gün atlanır.
            rise = rise_set(eph, cursor, RiseSet::Rise, geo_lat, geo_lon, disc)?;
            cursor += 1.0;
            continue;
        };
        if r >= to_jd_ut {
            break;
        }
        let from = r + RISE_STEP_DAYS;
        match rise_set(eph, from, RiseSet::Rise, geo_lat, geo_lon, disc)? {
            Some(n) if n - r <= SUNRISE_WINDOW_DAYS => {
                let set = rise_set(eph, r, RiseSet::Set, geo_lat, geo_lon, disc)?;
                if let Some(s) = set.filter(|s| *s < n) {
                    out.push(SunDay {
                        sunrise_jd_ut: r,
                        sunset_jd_ut: s,
                        next_sunrise_jd_ut: n,
                        vara: local_weekday(r, geo_lon),
                    });
                }
                rise = Some(n);
            }
            // Sonraki doğuş uzakta: `r`'de başlayan gün atlanır.
            Some(n) => rise = Some(n),
            None => {
                rise = None;
                cursor = r + 1.0;
            }
        }
    }
    Ok(out)
}

/// `days` listesi `[from, to]` aralığını boşluksuz örtüyor mu (her günün sonraki doğuşu bir
/// sonraki günün doğuşu, ilk doğuş `from`'dan önce, son gün `to`'dan sonra bitiyor)?
pub fn sun_days_cover(days: &[SunDay], from_jd_ut: f64, to_jd_ut: f64) -> bool {
    let (Some(first), Some(last)) = (days.first(), days.last()) else {
        return false;
    };
    first.sunrise_jd_ut <= from_jd_ut
        && last.next_sunrise_jd_ut >= to_jd_ut
        && days
            .windows(2)
            .all(|w| w[0].next_sunrise_jd_ut == w[1].sunrise_jd_ut)
}

/// Vara dizisi: gün doğumundan gün doğumuna. Değer her anda `compute_panchang`'ın varasıdır
/// (gün doğumu yoksa gün 06:00 yerel ortalama saatte başlar). Günler aralığı boşluksuz
/// örtüyorsa dizi doğuş anlarından kurulur; değilse (kutup gecesi ya da gündüzü) v1 kuralı
/// adım adım yürünür.
pub fn vara_series<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    geo_lat: f64,
    geo_lon: f64,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<Interval<Vara>>, CoreError> {
    let days = sun_days(eph, settings, geo_lat, geo_lon, from_jd_ut, to_jd_ut)?;
    if sun_days_cover(&days, from_jd_ut, to_jd_ut) {
        return Ok(vara_of_sun_days(&days, from_jd_ut, to_jd_ut));
    }
    let disc = settings.sunrise_def.sun_disc();
    walk(eph, from_jd_ut, to_jd_ut, VARA_STEP_DAYS, |eph, jd| {
        vara_at(eph, jd, geo_lat, geo_lon, disc)
    })
}

/// Aralığı boşluksuz örten günlerden vara dizisi, `[from, to]`'ya kırpılmış.
pub fn vara_of_sun_days(days: &[SunDay], from_jd_ut: f64, to_jd_ut: f64) -> Vec<Interval<Vara>> {
    days.iter()
        .map(|d| Interval {
            value: d.vara,
            start_jd_ut: d.sunrise_jd_ut.max(from_jd_ut),
            end_jd_ut: d.next_sunrise_jd_ut.min(to_jd_ut),
        })
        .filter(|i| i.start_jd_ut < i.end_jd_ut)
        .collect()
}

#[cfg(test)]
mod tests {
    use ephemeris::{
        Ayanamsa, BodyPosition, EphError, HouseSystem, Houses, Precision, SiderealMode,
    };

    use super::*;
    use crate::fake::FakeEph;
    use crate::panchang::{compute_panchang, karana_name, tithi_no, KaranaName};
    use crate::settings::{NodeKind, ZodiacMode};

    const JD: f64 = 2_451_545.0;
    /// Bir sınırın analitik değerine izin verilen uzaklık: 0,2 sn.
    const TOL: f64 = 0.2 / 86_400.0;

    fn settings() -> CalcSettings {
        CalcSettings::default()
    }

    fn assert_partition<T: PartialEq + std::fmt::Debug>(s: &[Interval<T>], from: f64, to: f64) {
        assert!(!s.is_empty());
        assert_eq!(s[0].start_jd_ut, from);
        assert_eq!(s[s.len() - 1].end_jd_ut, to);
        for w in s.windows(2) {
            assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut, "no gap, no overlap");
            assert_ne!(w[0].value, w[1].value, "adjacent values differ");
        }
        assert!(s.iter().all(|i| i.start_jd_ut < i.end_jd_ut));
    }

    #[test]
    fn walker_finds_every_change_and_keeps_the_span() {
        // Değer günün tam sayı kısmıdır: sınırlar tam günlerdedir; adım 2,5 gün (bir adımda
        // iki ya da üç değişim).
        let mut eph = FakeEph::default();
        let s = walk(&mut eph, JD + 0.3, JD + 7.6, 2.5, |_, jd| {
            Ok((jd - JD).floor() as i64)
        })
        .unwrap();
        assert_partition(&s, JD + 0.3, JD + 7.6);
        assert_eq!(
            s.iter().map(|i| i.value).collect::<Vec<_>>(),
            (0..=7).collect::<Vec<_>>()
        );
        for (k, i) in s.iter().enumerate().skip(1) {
            let boundary = JD + k as f64;
            // Sınır, yeni değerin geçerli olduğu ilk örnektir: gerçek sınırdan önce değildir.
            assert!(
                i.start_jd_ut >= boundary && i.start_jd_ut - boundary <= TIMELINE_RESOLUTION_DAYS
            );
        }
        // Değişmeyen değer tek aralıktır.
        let flat = walk(&mut eph, JD, JD + 3.0, 1.0, |_, _| Ok(7u8)).unwrap();
        assert_eq!(
            flat,
            [Interval {
                value: 7,
                start_jd_ut: JD,
                end_jd_ut: JD + 3.0
            }]
        );
    }

    #[test]
    fn walker_rejects_bad_spans_and_reports_value_errors() {
        let mut eph = FakeEph::default();
        let ok = |_: &mut FakeEph, _: f64| Ok(0u8);
        assert_eq!(
            walk(&mut eph, JD, JD, 1.0, ok),
            Err(CoreError::InvalidMuhurtaInput("range"))
        );
        assert_eq!(
            walk(&mut eph, JD + 1.0, JD, 1.0, ok),
            Err(CoreError::InvalidMuhurtaInput("range"))
        );
        assert!(matches!(
            walk(&mut eph, f64::NAN, JD, 1.0, ok),
            Err(CoreError::InvalidMuhurtaInput("range"))
        ));
        assert_eq!(
            walk(&mut eph, 1e9, 1e9 + 1.0, 1.0, ok),
            Err(CoreError::InvalidMuhurtaInput("range"))
        );
        let failing = walk(&mut eph, JD, JD + 1.0, 1.0, |_, jd| {
            if jd > JD {
                Err(CoreError::NonFiniteResult("timeline"))
            } else {
                Ok(0u8)
            }
        });
        assert_eq!(failing, Err(CoreError::NonFiniteResult("timeline")));
    }

    #[test]
    fn map_merge_and_value_at() {
        let s = [
            Interval {
                value: 1u8,
                start_jd_ut: 0.0,
                end_jd_ut: 1.0,
            },
            Interval {
                value: 2,
                start_jd_ut: 1.0,
                end_jd_ut: 2.0,
            },
            Interval {
                value: 3,
                start_jd_ut: 2.0,
                end_jd_ut: 3.0,
            },
        ];
        let merged = map_merge(&s, |v| *v >= 2);
        assert_eq!(merged.len(), 2);
        assert_eq!((merged[1].start_jd_ut, merged[1].end_jd_ut), (1.0, 3.0));
        assert_eq!(value_at(&s, 0.0), Some(1));
        assert_eq!(value_at(&s, 1.0), Some(2));
        assert_eq!(value_at(&s, 2.999), Some(3));
        // Uçlar yarı açıktır; dizinin dışı boştur.
        assert_eq!(value_at(&s, 3.0), None);
        assert_eq!(value_at(&s, -0.1), None);
        assert_eq!(value_at::<u8>(&[], 1.0), None);
    }

    #[test]
    fn karana_and_tithi_series_follow_the_linear_elongation() {
        // Sahte sidereal: Güneş 346° + 0,98°/gün, Ay 71° + 13,2°/gün; uzanım 85° + 12,22°/gün.
        let mut eph = FakeEph::default();
        let (from, to) = (JD, JD + 3.0);
        let karanas = karana_series(&mut eph, &settings(), from, to).unwrap();
        assert_partition(&karanas, from, to);
        // 85° → 15. karana (Vishti); 90°'de 16., sonra her 6°'de bir.
        assert_eq!(karanas[0].value, 15);
        assert_eq!(karana_name(karanas[0].value), KaranaName::Vishti);
        for (k, i) in karanas.iter().enumerate().skip(1) {
            let boundary = JD + (90.0 + 6.0 * (k as f64 - 1.0) - 85.0) / 12.22;
            assert!((i.start_jd_ut - boundary).abs() < TOL, "karana {k}");
            assert_eq!(i.value, 15 + k as u8);
        }
        // 3 günde uzanım 36,66° ilerler: 85° → 121,66°; 90, 96, …, 120: altı sınır.
        assert_eq!(karanas.len(), 7);

        let tithis = tithi_series(&mut eph, &settings(), from, to).unwrap();
        assert_partition(&tithis, from, to);
        // 85° → 8. tithi; 96°, 108°, 120°'de değişir.
        assert_eq!(
            tithis.iter().map(|i| i.value).collect::<Vec<_>>(),
            [8, 9, 10, 11]
        );
        for (k, i) in tithis.iter().enumerate().skip(1) {
            let boundary = JD + (96.0 + 12.0 * (k as f64 - 1.0) - 85.0) / 12.22;
            assert!((i.start_jd_ut - boundary).abs() < TOL, "tithi {k}");
        }
        // Her tithi sınırı bir karana sınırıdır (aynı an, bit bit).
        for t in &tithis[1..] {
            assert!(karanas.iter().any(|k| k.start_jd_ut == t.start_jd_ut));
        }
        for k in 1..=60u8 {
            assert_eq!(
                tithi_of_karana(k),
                tithi_no((f64::from(k) - 0.5) * 6.0),
                "karana {k}"
            );
        }
    }

    #[test]
    fn nakshatra_and_yoga_series_follow_the_linear_motion() {
        let mut eph = FakeEph::default();
        let (from, to) = (JD, JD + 2.0);
        let n = nakshatra_series(&mut eph, &settings(), from, to).unwrap();
        assert_partition(&n, from, to);
        // Ay 71°: Ardra (6.) 2. pada; padalar 73°20′, 76°40′, 80° (Punarvasu), … 'de değişir.
        assert_eq!(
            n[0].value,
            NakshatraPada {
                nakshatra: 6,
                pada: 2
            }
        );
        let expected = [(6, 3), (6, 4), (7, 1), (7, 2), (7, 3), (7, 4), (8, 1)];
        for (k, (nak, pada)) in expected.into_iter().enumerate() {
            let i = &n[k + 1];
            assert_eq!((i.value.nakshatra, i.value.pada), (nak, pada));
            let boundary = JD + (220.0 / 3.0 + 10.0 / 3.0 * k as f64 - 71.0) / 13.2;
            assert!((i.start_jd_ut - boundary).abs() < TOL, "pada {k}");
        }
        // 2 günde 26,4°: 71° → 97,4°; son sınır 96°40′ (Pushya 2. pada).
        assert_eq!(n.len(), 9);

        let y = nitya_yoga_series(&mut eph, &settings(), from, to).unwrap();
        assert_partition(&y, from, to);
        // Toplam 57° + 14,18°/gün: 5. yoga; 66°40′ ve 80°'de değişir.
        assert_eq!(y.iter().map(|i| i.value).collect::<Vec<_>>(), [5, 6, 7]);
        assert!((y[1].start_jd_ut - (JD + (200.0 / 3.0 - 57.0) / 14.18)).abs() < TOL);
        assert!((y[2].start_jd_ut - (JD + (80.0 - 57.0) / 14.18)).abs() < TOL);
    }

    #[test]
    fn series_agree_with_the_birth_panchang_at_sampled_moments() {
        // Çapraz denetim (ileri modüller spec'i §10 "Test"): aralıkların içinden örneklenen
        // anlarda `compute_panchang` aynı değerleri verir; aralığın sonu, `compute_panchang`'ın
        // bitiş anıyla 1 saniye içinde örtüşür.
        let s = settings();
        let (from, to) = (JD - 5.0, JD + 5.0);
        let mut eph = FakeEph::default();
        let tithis = tithi_series(&mut eph, &s, from, to).unwrap();
        let karanas = karana_series(&mut eph, &s, from, to).unwrap();
        let nakshatras = nakshatra_series(&mut eph, &s, from, to).unwrap();
        let yogas = nitya_yoga_series(&mut eph, &s, from, to).unwrap();
        let varas = vara_series(&mut eph, &s, 41.0, 29.0, from, to).unwrap();
        assert_partition(&varas, from, to);
        let second = 1.0 / 86_400.0;
        let mut checked = 0;
        for k in 0..200 {
            let jd = from + 0.013 + 0.0497 * f64::from(k);
            let p = compute_panchang(&mut eph, jd, 41.0, 29.0, &s).unwrap();
            let (Some(t), Some(ka), Some(n), Some(y), Some(v)) = (
                tithis.iter().find(|i| i.contains(jd)),
                karanas.iter().find(|i| i.contains(jd)),
                nakshatras.iter().find(|i| i.contains(jd)),
                yogas.iter().find(|i| i.contains(jd)),
                varas.iter().find(|i| i.contains(jd)),
            ) else {
                panic!("sample {k} outside the series");
            };
            assert_eq!(t.value, p.tithi.no, "tithi at {jd}");
            assert_eq!(ka.value, p.karana.no, "karana at {jd}");
            assert_eq!(
                (n.value.nakshatra, n.value.pada),
                (p.nakshatra.no, p.nakshatra.pada)
            );
            assert_eq!(y.value, p.yoga.no, "yoga at {jd}");
            assert_eq!(v.value, p.vara, "vara at {jd}");
            // Kırpılmamış uçlar bitiş anıyla örtüşür.
            if t.end_jd_ut < to {
                assert!((t.end_jd_ut - p.tithi.ends_jd_ut).abs() <= second);
                checked += 1;
            }
            if ka.end_jd_ut < to {
                assert!((ka.end_jd_ut - p.karana.ends_jd_ut).abs() <= second);
            }
            if y.end_jd_ut < to {
                assert!((y.end_jd_ut - p.yoga.ends_jd_ut).abs() <= second);
            }
            // `compute_panchang` nakshatranın bitişini verir: son padanın sonu.
            let nak_end = nakshatras
                .iter()
                .rfind(|i| i.value.nakshatra == n.value.nakshatra)
                .map(|i| i.end_jd_ut)
                .unwrap();
            if nak_end < to {
                assert!((nak_end - p.nakshatra.ends_jd_ut).abs() <= second);
            }
        }
        assert!(checked > 150, "{checked}");
    }

    #[test]
    fn graha_sign_series_follow_the_rasi_frame() {
        let (from, to) = (JD, JD + 40.0);
        // Sidereal kip: Güneş 346° + 0,98°/gün; 360°'de (14,29. gün) Koç'a, 30°'de Boğa'ya.
        let mut eph = FakeEph::default();
        let sun = graha_sign_series(&mut eph, &settings(), Graha::Sun, from, to).unwrap();
        assert_partition(&sun, from, to);
        assert_eq!(
            sun.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Sign::Pisces, Sign::Aries]
        );
        assert!((sun[1].start_jd_ut - (JD + 14.0 / 0.98)).abs() < TOL);
        // Tropikal rasi kipinde burç tropikal boylamdandır: Güneş 10° (Koç), 30°'de Boğa'ya.
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..settings()
        };
        let sun = graha_sign_series(&mut eph, &tropical, Graha::Sun, from, to).unwrap();
        assert_eq!(
            sun.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Sign::Aries, Sign::Taurus]
        );
        assert!((sun[1].start_jd_ut - (JD + 20.0 / 0.98)).abs() < TOL);
        // Nakshatra iki kipte de sidereal'dir.
        let a = graha_nakshatra_series(&mut eph, &settings(), Graha::Moon, from, to).unwrap();
        let b = graha_nakshatra_series(&mut eph, &tropical, Graha::Moon, from, to).unwrap();
        assert_eq!(a, b);
        assert_partition(&a, from, to);
        // Ay 71° + 13,2°/gün: 40 günde 528° → 39 nakshatra sınırı ve bir fazlası kadar aralık.
        assert_eq!(a.len(), 40);
        assert_eq!(a[0].value, 6);
        assert_eq!(a[1].value, 7);

        // Geri giden graha burca geriden girer: Jüpiter sidereal 226° − 0,1°/gün; 210°'de
        // (160. gün) Akrep'ten Terazi'ye.
        let jup = graha_sign_series(&mut eph, &settings(), Graha::Jupiter, JD, JD + 200.0).unwrap();
        assert_eq!(
            jup.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Sign::Scorpio, Sign::Libra]
        );
        assert!((jup[1].start_jd_ut - (JD + 160.0)).abs() < TOL);

        // Ketu, Rahu'nun karşısıdır; düğüm türü ayardan gelir. Ortalama düğüm sidereal 96° −
        // 0,053°/gün: Rahu 90°'de (113,2. gün) Yengeç'ten İkizler'e, Ketu aynı anda Oğlak'tan
        // Yay'a.
        let rahu = graha_sign_series(&mut eph, &settings(), Graha::Rahu, JD, JD + 150.0).unwrap();
        let ketu = graha_sign_series(&mut eph, &settings(), Graha::Ketu, JD, JD + 150.0).unwrap();
        assert_eq!(
            rahu.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Sign::Cancer, Sign::Gemini]
        );
        assert_eq!(
            ketu.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Sign::Capricorn, Sign::Sagittarius]
        );
        assert_eq!(rahu[1].start_jd_ut, ketu[1].start_jd_ut);
        assert!((rahu[1].start_jd_ut - (JD + 6.0 / 0.053)).abs() < TOL);
        // Gerçek düğüm ayarı başka cismi sorar (sahte: 101° sidereal, ileri gider).
        let true_node = CalcSettings {
            node: NodeKind::True,
            ..settings()
        };
        let rahu = graha_sign_series(&mut eph, &true_node, Graha::Rahu, JD, JD + 150.0).unwrap();
        assert_eq!(rahu.len(), 1);
        assert_eq!(rahu[0].value, Sign::Cancer);
    }

    /// Boylamı `lon0 + hız × t + genlik × sin(2π t / dönem)` olan tek cisimli sahte ephemeris
    /// (retro dönemleri olan bir graha); Güneş sabit hızla ilerler.
    struct WobbleEph {
        lon0: f64,
        speed: f64,
        amplitude: f64,
        period: f64,
        sun0: f64,
    }

    impl WobbleEph {
        fn phase(&self, jd: f64) -> f64 {
            std::f64::consts::TAU * (jd - JD) / self.period
        }
    }

    impl Ephemeris for WobbleEph {
        fn body(&mut self, jd: f64, body: Body, _: Zodiac) -> Result<BodyPosition, EphError> {
            let (lon, lon_speed) = if body == Body::Sun {
                (self.sun0 + (jd - JD), 1.0)
            } else {
                (
                    self.lon0 + self.speed * (jd - JD) + self.amplitude * self.phase(jd).sin(),
                    self.speed
                        + self.amplitude * std::f64::consts::TAU / self.period
                            * self.phase(jd).cos(),
                )
            };
            Ok(BodyPosition {
                lon: normalize(lon),
                lat: 0.0,
                distance_au: 1.0,
                lon_speed,
                lat_speed: 0.0,
                dist_speed: 0.0,
                precision: Precision::SwissEph,
            })
        }

        fn ayanamsa(&mut self, _: f64, _: SiderealMode) -> Result<Ayanamsa, EphError> {
            Ok(Ayanamsa {
                deg: 0.0,
                precision: Precision::SwissEph,
            })
        }

        fn houses(
            &mut self,
            _: f64,
            _: f64,
            _: f64,
            _: HouseSystem,
            _: Zodiac,
        ) -> Result<Houses, EphError> {
            Err(EphError::Calc("wobble: no houses".into()))
        }

        fn sun_rise_set(
            &mut self,
            _: f64,
            _: RiseSet,
            _: f64,
            _: f64,
            _: SunDisc,
        ) -> Result<Option<f64>, EphError> {
            Ok(None)
        }

        fn swe_version(&self) -> String {
            "wobble".into()
        }
    }

    #[test]
    fn motion_series_finds_the_stations_where_the_speed_changes_sign() {
        // Hız 0,5 + 1,0 × cos(2π t / 100): cos = −0,5 olduğunda sıfır; t = 100/3 ve 200/3.
        let mut eph = WobbleEph {
            lon0: 100.0,
            speed: 0.5,
            amplitude: 100.0 / std::f64::consts::TAU,
            period: 100.0,
            sun0: 0.0,
        };
        let s = settings();
        let m = graha_motion_series(&mut eph, &s, Graha::Mars, JD, JD + 100.0).unwrap();
        assert_partition(&m, JD, JD + 100.0);
        assert_eq!(
            m.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Motion::Direct, Motion::Retrograde, Motion::Direct]
        );
        assert!((m[1].start_jd_ut - (JD + 100.0 / 3.0)).abs() < TOL);
        assert!((m[2].start_jd_ut - (JD + 200.0 / 3.0)).abs() < TOL);
        // Dönüş anında hız işaret değiştirir.
        for station in [m[1].start_jd_ut, m[2].start_jd_ut] {
            let before = eph
                .body(station - 0.01, Body::Mars, Zodiac::Tropical)
                .unwrap();
            let after = eph
                .body(station + 0.01, Body::Mars, Zodiac::Tropical)
                .unwrap();
            assert!(before.lon_speed * after.lon_speed < 0.0);
        }
        // Retro graha burç sınırını üç kez geçer: 100° → 130,4° (retro dönüşü) → 119,6° (düz
        // dönüş) → 148,5°; 120° sınırı ileri, geri ve yeniden ileri geçilir.
        let signs = graha_sign_series(&mut eph, &s, Graha::Mars, JD, JD + 99.0).unwrap();
        assert_eq!(
            signs.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Sign::Cancer, Sign::Leo, Sign::Cancer, Sign::Leo]
        );
        for i in &signs[1..] {
            let lon = eph
                .body(i.start_jd_ut, Body::Mars, Zodiac::Tropical)
                .unwrap()
                .lon;
            // Giriş anında boylam sınırın 1″ içindedir.
            assert!((lon - 120.0).abs() < 1.0 / 3600.0, "{lon}");
        }
        // Yalnızca retro olabilen beş graha.
        for g in [Graha::Sun, Graha::Moon, Graha::Rahu, Graha::Ketu] {
            assert_eq!(
                graha_motion_series(&mut eph, &s, g, JD, JD + 1.0),
                Err(CoreError::InvalidMuhurtaInput("graha"))
            );
        }
    }

    #[test]
    fn combust_series_uses_the_chart_orbs_and_the_retrograde_orb() {
        // Güneş 0° + 1°/gün; Venüs 20° sabit (hız 0, düz): fark 20 − t. Venüs'ün orbu 10°:
        // t = 10'da yanar, t = 30'da (fark −10°) çıkar.
        let mut eph = WobbleEph {
            lon0: 20.0,
            speed: 0.0,
            amplitude: 0.0,
            period: 100.0,
            sun0: 0.0,
        };
        let s = settings();
        let c = combust_series(&mut eph, &s, Graha::Venus, JD, JD + 40.0).unwrap();
        assert_partition(&c, JD, JD + 40.0);
        assert_eq!(
            c.iter().map(|i| i.value).collect::<Vec<_>>(),
            [false, true, false]
        );
        assert!((c[1].start_jd_ut - (JD + 10.0)).abs() < TOL);
        assert!((c[2].start_jd_ut - (JD + 30.0)).abs() < TOL);
        // Geri giden Venüs'ün orbu 8°'dir: hız −0,001°/gün; fark 20 − 1,001 t.
        let mut retro = WobbleEph {
            lon0: 20.0,
            speed: -0.001,
            amplitude: 0.0,
            period: 100.0,
            sun0: 0.0,
        };
        let c = combust_series(&mut retro, &s, Graha::Venus, JD, JD + 40.0).unwrap();
        assert!((c[1].start_jd_ut - (JD + 12.0 / 1.001)).abs() < TOL);
        assert!((c[2].start_jd_ut - (JD + 28.0 / 1.001)).abs() < TOL);
        // Güneş ve düğümler yanmaz.
        let sun = combust_series(&mut eph, &s, Graha::Sun, JD, JD + 40.0).unwrap();
        assert_eq!(sun.len(), 1);
        assert!(!sun[0].value);
    }

    #[test]
    fn lagna_series_turns_through_twelve_signs_a_day() {
        // Sahte lagna tropikal 70° (sidereal 46°, Boğa), günde 360°: her 2 saatte bir burç.
        let mut eph = FakeEph {
            asc_speed: 360.0,
            ..FakeEph::default()
        };
        let s = settings();
        let l = lagna_sign_series(&mut eph, &s, 41.0, 29.0, JD, JD + 1.0).unwrap();
        assert_partition(&l, JD, JD + 1.0);
        assert_eq!(l.len(), 13);
        assert_eq!(l[0].value, Sign::Taurus);
        assert_eq!(l[12].value, Sign::Taurus);
        for (k, i) in l.iter().enumerate().skip(1) {
            assert_eq!(i.value, Sign::from_index(1 + k));
            // İlk sınır 60°'de: 14° / 360 gün; sonrakiler 30°'de bir.
            let boundary = JD + (14.0 + 30.0 * (k as f64 - 1.0)) / 360.0;
            assert!((i.start_jd_ut - boundary).abs() < TOL, "lagna {k}");
            assert_eq!(
                lagna_sign_at(&mut eph, &s, i.start_jd_ut, 41.0, 29.0).unwrap(),
                i.value
            );
        }
        // Tropikal rasi kipinde lagna burcu tropikal boylamdandır (70°: İkizler).
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..settings()
        };
        let l = lagna_sign_series(&mut eph, &tropical, 41.0, 29.0, JD, JD + 0.05).unwrap();
        assert_eq!(l[0].value, Sign::Gemini);
        assert_eq!(
            lagna_sign_series(&mut eph, &s, 91.0, 29.0, JD, JD + 1.0),
            Err(CoreError::InvalidMuhurtaInput("place"))
        );
    }

    #[test]
    fn sun_days_and_the_vara_series_run_from_sunrise_to_sunrise() {
        // Sahte gün doğumu her gün n + 0,67, batımı n + 0,15.
        let mut eph = FakeEph::default();
        let s = settings();
        let (from, to) = (JD, JD + 3.0);
        let days = sun_days(&mut eph, &s, 41.0, 29.0, from, to).unwrap();
        assert_eq!(days.len(), 4);
        assert!(sun_days_cover(&days, from, to));
        for (k, d) in days.iter().enumerate() {
            let rise = 2_451_544.67 + k as f64;
            assert!((d.sunrise_jd_ut - rise).abs() < 1e-9);
            assert!((d.sunset_jd_ut - (rise + 0.48)).abs() < 1e-9);
            assert!((d.next_sunrise_jd_ut - (rise + 1.0)).abs() < 1e-9);
        }
        // 2451544,67 (1 Ocak 2000, yerel ortalama) Cumartesi.
        assert_eq!(
            days.iter().map(|d| d.vara).collect::<Vec<_>>(),
            [Vara::Saturday, Vara::Sunday, Vara::Monday, Vara::Tuesday]
        );
        let varas = vara_series(&mut eph, &s, 41.0, 29.0, from, to).unwrap();
        assert_partition(&varas, from, to);
        assert_eq!(varas.len(), 4);
        assert!((varas[1].start_jd_ut - 2_451_545.67).abs() < 1e-9);
        assert_eq!(varas, vara_of_sun_days(&days, from, to));
        // Aralık tam bir doğuşta biterse sonraki gün listeye girmez.
        let exact = sun_days(&mut eph, &s, 41.0, 29.0, JD, days[1].next_sunrise_jd_ut).unwrap();
        assert_eq!(exact.len(), 2);
    }

    #[test]
    fn without_a_sunrise_the_vara_changes_at_six_local_mean_time() {
        let mut eph = FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        let s = settings();
        let (from, to) = (JD, JD + 2.0);
        assert!(sun_days(&mut eph, &s, 41.0, 29.0, from, to)
            .unwrap()
            .is_empty());
        let varas = vara_series(&mut eph, &s, 41.0, 29.0, from, to).unwrap();
        assert_partition(&varas, from, to);
        // Boylam 29°: 06:00 yerel ortalama saat = 04:04 UT = n + 0,6694.
        assert_eq!(
            varas.iter().map(|i| i.value).collect::<Vec<_>>(),
            [Vara::Saturday, Vara::Sunday, Vara::Monday]
        );
        let six = 2_451_545.75 - 29.0 / 360.0;
        assert!((varas[1].start_jd_ut - six).abs() < TOL);
        assert!((varas[2].start_jd_ut - (six + 1.0)).abs() < TOL);
        for i in &varas {
            let mid = 0.5 * (i.start_jd_ut + i.end_jd_ut);
            assert_eq!(
                compute_panchang(&mut eph, mid, 41.0, 29.0, &s)
                    .unwrap()
                    .vara,
                i.value
            );
        }
    }

    #[test]
    fn a_gap_between_sunrises_leaves_the_day_out_and_falls_back_to_the_slow_path() {
        // Doğuşlar: gün 0, 1, 2 ve sonra 5 (üç günlük boşluk: kutup gecesi). Batışlar her gün
        // n + 0,15'tedir; sahte ephemeris doğuş listesi verilince 26 saat ileriye bakar.
        let rises = vec![JD - 0.3, JD + 0.7, JD + 1.7, JD + 4.7, JD + 5.7];
        let mut eph = FakeEph {
            sunrises: Some(rises),
            ..FakeEph::default()
        };
        let s = settings();
        let (from, to) = (JD, JD + 5.5);
        let days = sun_days(&mut eph, &s, 41.0, 29.0, from, to).unwrap();
        // [−0,3; 0,7), [0,7; 1,7) ve boşluktan sonra [4,7; 5,7): 1,7'de başlayan günün sonraki
        // doğuşu yoktur.
        assert_eq!(
            days.iter()
                .map(|d| ((d.sunrise_jd_ut - JD) * 10.0).round() as i64)
                .collect::<Vec<_>>(),
            [-3, 7, 47]
        );
        assert!(!sun_days_cover(&days, from, to));
        let varas = vara_series(&mut eph, &s, 41.0, 29.0, from, to).unwrap();
        assert_partition(&varas, from, to);
        for k in 0..110 {
            let jd = from + 0.01 + 0.05 * f64::from(k);
            let v = varas.iter().find(|i| i.contains(jd)).unwrap().value;
            assert_eq!(
                v,
                compute_panchang(&mut eph, jd, 41.0, 29.0, &s).unwrap().vara,
                "vara at +{}",
                jd - JD
            );
        }
    }
}
