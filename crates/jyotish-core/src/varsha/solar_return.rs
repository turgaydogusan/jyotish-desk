//! Dönüş anı: Güneş'in doğumdaki sidereal boylamına `age` kez döndüğü an. Yıllık haritanın ve
//! mudda dasanın başlangıcıdır.
//!
//! Yöntem: ilk tahmin doğum + `age` × sidereal yıldır; Güneş hep ileri yürüdüğü için tahminin
//! iki yanındaki dar aralıkta (`RETURN_BRACKET_DAYS`) işaretli yay tek bir kez sıfırdan geçer ve
//! geçiş ikiye bölmeyle bulunur (`RETURN_RESOLUTION_DAYS`). Dönen an, Güneş'in doğum boylamına
//! varmış olduğu ilk örnektir: o anda yay sıfır ya da çok küçük bir artıdır.
//!
//! Boylam her zodyak kipinde sidereal sorulur (seçili ayanamsayla): tropikal dönüş her yıl
//! yaklaşık yirmi dakika erken gelir ve yıllık lagnayı değiştirir.

use ephemeris::{Body, Ephemeris};

use crate::chart::finite;
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::timeline::{lon_of, sidereal};
use crate::zodiac::normalize;
use crate::CoreError;

use super::MAX_VARSHA_AGE;

/// Sidereal yılın ortalama uzunluğu, gün: dönüş aramasının ilk tahmini.
pub const SIDEREAL_YEAR_DAYS: f64 = 365.256_363;
/// Tahminin iki yanında aranan pay, gün. Gerçek dönüş tahminden birkaç dakikadan çok sapmaz;
/// pay geniş tutulmuştur.
pub const RETURN_BRACKET_DAYS: f64 = 3.0;
/// Dönüş anının çözünürlüğü: 0,01 saniye. Güneş bu sürede yaklaşık 0,0004″ yürür.
pub const RETURN_RESOLUTION_DAYS: f64 = 0.01 / 86_400.0;
/// İkiye bölme sınırı (6 gün / 2^40; pay bırakılmıştır).
const MAX_BISECTIONS: u32 = 40;

/// İşaretli yay `lon − target`, [−180, 180): Güneş hedefin gerisindeyse eksi.
fn arc_from(target: f64, lon: f64) -> f64 {
    normalize(lon - target + 180.0) - 180.0
}

/// `n`. dönüşün anı; `n` denetlenmez (yılın sonu için `MAX_VARSHA_AGE + 1` de sorulur).
fn return_no<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    natal_sun_sidereal_lon: f64,
    birth_jd_ut: f64,
    n: u32,
) -> Result<f64, CoreError> {
    // NaN ve sonsuz da aralık dışındadır.
    if !JD_UT_RANGE.contains(&birth_jd_ut) {
        return Err(CoreError::InvalidJulianDay(birth_jd_ut));
    }
    let target = normalize(finite(natal_sun_sidereal_lon, "varsha")?);
    if n == 0 {
        return Ok(birth_jd_ut);
    }
    let guess = birth_jd_ut + f64::from(n) * SIDEREAL_YEAR_DAYS;
    let (mut lo, mut hi) = (guess - RETURN_BRACKET_DAYS, guess + RETURN_BRACKET_DAYS);
    if !(JD_UT_RANGE.contains(&lo) && JD_UT_RANGE.contains(&hi)) {
        return Err(CoreError::InvalidJulianDay(guess));
    }
    let zodiac = sidereal(settings);
    let arc = |eph: &mut E, jd: f64| {
        Ok::<_, CoreError>(arc_from(target, lon_of(eph, jd, Body::Sun, zodiac)?))
    };
    // Aralığın başında Güneş hedefin gerisinde, sonunda hedefte ya da ötesinde olmalıdır.
    if !(arc(eph, lo)? < 0.0 && arc(eph, hi)? >= 0.0) {
        return Err(CoreError::PanchangSearch("solar_return"));
    }
    let mut steps = 0;
    while hi - lo > RETURN_RESOLUTION_DAYS && steps < MAX_BISECTIONS {
        let mid = 0.5 * (lo + hi);
        if arc(eph, mid)? < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
        steps += 1;
    }
    Ok(hi)
}

fn check_age(age: u8) -> Result<(), CoreError> {
    if age > MAX_VARSHA_AGE {
        return Err(CoreError::InvalidDashaInput("age"));
    }
    Ok(())
}

/// Güneş'in `natal_sun_sidereal_lon` boylamına doğumdan sonra `age`. kez döndüğü an (UT Jülyen
/// günü). `age` tamamlanan yıldır: 0 doğum anının kendisidir, 1 ilk doğum günü dönüşüdür.
/// `age > MAX_VARSHA_AGE` `InvalidDashaInput("age")`; an hesap aralığının dışına düşerse
/// `InvalidJulianDay`; Güneş aranan aralıkta dönmüyorsa (yalnızca bozuk ya da sahte ephemeris)
/// `PanchangSearch("solar_return")`.
pub fn solar_return_jd<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    natal_sun_sidereal_lon: f64,
    birth_jd_ut: f64,
    age: u8,
) -> Result<f64, CoreError> {
    check_age(age)?;
    return_no(
        eph,
        settings,
        natal_sun_sidereal_lon,
        birth_jd_ut,
        u32::from(age),
    )
}

/// `age` yaşının yılı: `[dönüş, sonraki dönüş)`. Yılın uzunluğu iki dönüşün arasıdır (yaklaşık
/// 365,2564 gün).
pub fn varsha_span<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    natal_sun_sidereal_lon: f64,
    birth_jd_ut: f64,
    age: u8,
) -> Result<(f64, f64), CoreError> {
    check_age(age)?;
    let n = u32::from(age);
    let start = return_no(eph, settings, natal_sun_sidereal_lon, birth_jd_ut, n)?;
    let end = return_no(eph, settings, natal_sun_sidereal_lon, birth_jd_ut, n + 1)?;
    Ok((start, end))
}

/// `at_jd_ut` anında süren yılın yaşı: dönüşü `at_jd_ut`'den sonra olmayan en büyük yaş. Doğumdan
/// önceki an 0, `MAX_VARSHA_AGE`. dönüşten sonraki her an `MAX_VARSHA_AGE` verir.
pub fn completed_years_at<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    natal_sun_sidereal_lon: f64,
    birth_jd_ut: f64,
    at_jd_ut: f64,
) -> Result<u8, CoreError> {
    if !at_jd_ut.is_finite() {
        return Err(CoreError::InvalidJulianDay(at_jd_ut));
    }
    let returned = |eph: &mut E, age: u8| {
        solar_return_jd(eph, settings, natal_sun_sidereal_lon, birth_jd_ut, age)
            .map(|jd| jd <= at_jd_ut)
    };
    // Doğum anı da burada denetlenir.
    if !returned(eph, 0)? {
        return Ok(0);
    }
    let years = ((at_jd_ut - birth_jd_ut) / SIDEREAL_YEAR_DAYS).floor();
    let mut age = years.clamp(0.0, f64::from(MAX_VARSHA_AGE)) as u8;
    // Tahmin en çok bir yıl sapar: gerçek dönüşlerle düzeltilir.
    while age < MAX_VARSHA_AGE && returned(eph, age + 1)? {
        age += 1;
    }
    while age > 0 && !returned(eph, age)? {
        age -= 1;
    }
    Ok(age)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::ZodiacMode;
    use crate::zodiac::angular_distance;

    const JD: f64 = 2_451_545.0;
    /// Bir yay saniyesi, derece.
    const ARCSEC: f64 = 1.0 / 3600.0;
    /// Sahte Güneş'in hızı: tam bir sidereal yılda 360°.
    const SUN_SPEED: f64 = 360.0 / SIDEREAL_YEAR_DAYS;

    fn eph() -> FakeEph {
        FakeEph::default().with(Body::Sun, 10.0, SUN_SPEED)
    }

    fn settings(zodiac: ZodiacMode) -> CalcSettings {
        CalcSettings {
            zodiac,
            ..CalcSettings::default()
        }
    }

    /// Sahte Güneş'in `jd` anındaki sidereal boylamı (tropikal 10°, ayanamsa 24°).
    fn sun_at(eph: &mut FakeEph, jd: f64) -> f64 {
        lon_of(eph, jd, Body::Sun, sidereal(&CalcSettings::default())).unwrap()
    }

    #[test]
    fn the_return_brings_the_sidereal_sun_back_within_one_arcsecond() {
        // Doğumda sidereal Güneş 10° − 24° = 346°. Her dönüş tam bir sidereal yıl sonradır.
        for zodiac in [
            ZodiacMode::Sidereal,
            ZodiacMode::TropicalRasiSiderealNakshatra,
        ] {
            let mut eph = eph();
            let natal = sun_at(&mut eph, JD);
            assert!((natal - 346.0).abs() < 1e-9, "{natal}");
            for age in [1_u8, 2, 37, 120] {
                let jd = solar_return_jd(&mut eph, &settings(zodiac), natal, JD, age).unwrap();
                let back = sun_at(&mut eph, jd);
                assert!(
                    angular_distance(back, natal) < ARCSEC,
                    "{zodiac:?} age {age}: {back} vs {natal}"
                );
                // Güneş doğum boylamına varmıştır: yay eksi değildir.
                assert!(arc_from(natal, back) >= 0.0, "{zodiac:?} age {age}");
                let want = JD + f64::from(age) * SIDEREAL_YEAR_DAYS;
                assert!((jd - want).abs() < 1e-6, "{zodiac:?} age {age}: {jd}");
            }
        }
    }

    #[test]
    fn age_zero_is_the_birth_moment_and_the_span_runs_to_the_next_return() {
        let mut eph = eph();
        let natal = sun_at(&mut eph, JD);
        let s = CalcSettings::default();
        assert_eq!(solar_return_jd(&mut eph, &s, natal, JD, 0).unwrap(), JD);
        let (start, end) = varsha_span(&mut eph, &s, natal, JD, 0).unwrap();
        assert_eq!(start, JD);
        assert!((end - start - SIDEREAL_YEAR_DAYS).abs() < 1e-6, "{end}");
        // Ardışık yıllar boşluksuzdur: bir yılın sonu sonrakinin başıdır.
        let (next_start, next_end) = varsha_span(&mut eph, &s, natal, JD, 1).unwrap();
        assert_eq!(next_start, end);
        assert!(next_end > next_start);
        // En büyük yaşın yılı da kapanır (121. dönüş yalnızca yılın sonu olarak sorulur).
        let (last_start, last_end) = varsha_span(&mut eph, &s, natal, JD, MAX_VARSHA_AGE).unwrap();
        assert!((last_end - last_start - SIDEREAL_YEAR_DAYS).abs() < 1e-6);
    }

    #[test]
    fn a_sun_off_the_mean_year_is_found_inside_the_bracket_and_a_stray_one_is_an_error() {
        // Hız 0,9856°/gün: yıl 365,2597 gün; 120 yılda tahminden 0,4 gün sapar (pay 3 gün).
        let mut slow = FakeEph::default().with(Body::Sun, 10.0, 0.9856);
        let natal = sun_at(&mut slow, JD);
        let s = CalcSettings::default();
        let jd = solar_return_jd(&mut slow, &s, natal, JD, 120).unwrap();
        assert!((jd - (JD + 120.0 * 360.0 / 0.9856)).abs() < 1e-6, "{jd}");
        // Hız 0,98°/gün: yıl 367,35 gün; 30 yılda sapma 62 gün, dönüş aralığın dışındadır.
        let mut stray = FakeEph::default();
        let natal = sun_at(&mut stray, JD);
        assert_eq!(
            solar_return_jd(&mut stray, &s, natal, JD, 30),
            Err(CoreError::PanchangSearch("solar_return"))
        );
    }

    #[test]
    fn age_moment_and_longitude_are_validated() {
        let mut eph = eph();
        let s = CalcSettings::default();
        assert_eq!(
            solar_return_jd(&mut eph, &s, 346.0, JD, MAX_VARSHA_AGE + 1),
            Err(CoreError::InvalidDashaInput("age"))
        );
        assert_eq!(
            varsha_span(&mut eph, &s, 346.0, JD, MAX_VARSHA_AGE + 1),
            Err(CoreError::InvalidDashaInput("age"))
        );
        assert!(matches!(
            solar_return_jd(&mut eph, &s, 346.0, f64::NAN, 1),
            Err(CoreError::InvalidJulianDay(_))
        ));
        assert_eq!(
            solar_return_jd(&mut eph, &s, f64::INFINITY, JD, 1),
            Err(CoreError::NonFiniteResult("varsha"))
        );
        // Hesap aralığının sonuna yakın doğum: dönüş aralığın dışına düşer.
        let late = *JD_UT_RANGE.end() - 100.0;
        assert!(matches!(
            solar_return_jd(&mut eph, &s, 346.0, late, 1),
            Err(CoreError::InvalidJulianDay(_))
        ));
        assert!(matches!(
            completed_years_at(&mut eph, &s, 346.0, JD, f64::NAN),
            Err(CoreError::InvalidJulianDay(_))
        ));
    }

    #[test]
    fn completed_years_follow_the_returns() {
        let mut eph = eph();
        let natal = sun_at(&mut eph, JD);
        let s = CalcSettings::default();
        let fifth = solar_return_jd(&mut eph, &s, natal, JD, 5).unwrap();
        let mut age_at = |at: f64| completed_years_at(&mut eph, &s, natal, JD, at).unwrap();
        assert_eq!(age_at(JD - 1.0), 0, "before birth");
        assert_eq!(age_at(JD), 0);
        assert_eq!(age_at(fifth - 1e-3), 4);
        assert_eq!(age_at(fifth), 5, "the return moment opens the year");
        assert_eq!(age_at(fifth + 200.0), 5);
        assert_eq!(age_at(JD + 300.0 * SIDEREAL_YEAR_DAYS), MAX_VARSHA_AGE);
    }
}
