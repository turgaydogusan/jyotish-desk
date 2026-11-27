//! Olay araması (ileri modüller spec'i §12.2): bir tarih aralığındaki gök olayları, 3.7'nin
//! `timeline` yürüyücüsüyle. Olay, bir değerin değiştiği andır: burç ya da nakshatra girişi,
//! hareket yönünün dönmesi, iki boylamın farkının sıfırdan geçmesi (kavuşum, yeni ay, geçiş
//! grahasının doğum boylamına gelişi) ya da 180°'den geçmesi (dolunay).
//!
//! Arama dilim dilim yürür (`event_chunks`, `event_search_chunk`): çağıran her dilimden sonra
//! motor iş parçacığını bırakır ve durdurma isteğine bakar. Bir olay yalnızca bir dilimde
//! bulunur: dilimin içindeki değişim o dilime aittir, dilimin ilk anındaki değer başlangıçtır.
//!
//! Burçlar rasi çerçevesindedir (K5); nakshatra sidereal boylamdandır. Doğum boylamları yalnızca
//! bellekte kullanılır: sonuca ve hataya girmez.

use ephemeris::{Body, Ephemeris};
use serde::{Deserialize, Serialize};

use crate::chart::rasi_frame;
use crate::graha::Graha;
use crate::positions::Positions;
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::timeline::{
    graha_body, graha_motion_series, graha_nakshatra_series, graha_sign_series, lon_of, sidereal,
    walk, Interval, Motion, GRAHA_STEP_DAYS, MOON_STEP_DAYS, STATION_GRAHAS,
};
use crate::zodiac::{angular_distance, nakshatra_pada, normalize, Sign};
use crate::CoreError;

/// Yöntem kimliği (K7).
pub const EVENT_METHOD: &str = "event_search.v1";
/// Ay içermeyen aramanın en uzun aralığı: 120 yıl.
pub const EVENT_MAX_RANGE_DAYS: f64 = 43_830.0;
/// Ay içeren aramanın en uzun aralığı: 5 yıl.
pub const EVENT_MOON_MAX_RANGE_DAYS: f64 = 1_827.0;
/// Ay içermeyen aramanın dilimi: 5 yıl.
pub const EVENT_CHUNK_DAYS: f64 = 1_827.0;
/// Ay içeren aramanın dilimi: yaklaşık üç ay.
pub const EVENT_MOON_CHUNK_DAYS: f64 = 92.0;
/// Bir aramanın döndürdüğü en çok olay; fazlası kesilir ve sonuç bunu söyler.
pub const MAX_SKY_EVENTS: usize = 2_000;
/// Doğum Ay'ına göre ev değiştirmesi izlenen grahalar.
pub const HOUSE_CHANGE_GRAHAS: [Graha; 2] = [Graha::Jupiter, Graha::Saturn];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum SkyEventKind {
    /// Grahanın burç girişi (retro girişler dahil).
    SignIngress,
    /// Grahanın nakshatra girişi.
    NakshatraIngress,
    /// Düz hareketten retroya dönüş.
    StationRetrograde,
    /// Retrodan düz harekete dönüş.
    StationDirect,
    /// İki grahanın boylamca tam kavuşumu.
    Conjunction,
    NewMoon,
    FullMoon,
    /// Geçiş grahasının bir doğum grahasının boylamına gelişi (kendi boylamına dönüşü dahil).
    TransitToNatal,
    /// Jüpiter ya da Satürn'ün doğum Ay burcundan sayılan evinin değişmesi.
    HouseFromNatalMoon,
}

impl SkyEventKind {
    pub const ALL: [SkyEventKind; 9] = [
        SkyEventKind::SignIngress,
        SkyEventKind::NakshatraIngress,
        SkyEventKind::StationRetrograde,
        SkyEventKind::StationDirect,
        SkyEventKind::Conjunction,
        SkyEventKind::NewMoon,
        SkyEventKind::FullMoon,
        SkyEventKind::TransitToNatal,
        SkyEventKind::HouseFromNatalMoon,
    ];

    /// Olay açık bir harita ister.
    pub fn needs_natal(self) -> bool {
        matches!(self, Self::TransitToNatal | Self::HouseFromNatalMoon)
    }

    /// Olay `EventSpec.grahas` seçimini kullanır.
    pub fn uses_grahas(self) -> bool {
        !matches!(
            self,
            Self::NewMoon | Self::FullMoon | Self::HouseFromNatalMoon
        )
    }
}

/// Bir gök olayı.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SkyEvent {
    pub kind: SkyEventKind,
    pub jd_ut: f64,
    /// Olayın grahaları: girişte ve dönüşte tek graha; kavuşumda iki graha (`Graha::ALL`
    /// sırasında); yeni ay ve dolunayda Güneş ve Ay; `TransitToNatal`'da önce geçiş grahası,
    /// sonra doğum grahası.
    pub grahas: Vec<Graha>,
    /// Olayın rasi çerçevesindeki burcu (girilen burç; kavuşumun, dönüşün ve Ay'ın burcu).
    pub sign: Option<Sign>,
    /// Girilen nakshatra; yeni ay ve dolunayda Ay'ın nakshatrası.
    pub nakshatra: Option<u8>,
    /// `HouseFromNatalMoon`: doğum Ay burcundan sayılan yeni ev.
    pub house: Option<u8>,
}

/// Aramanın çekirdek girdisi.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct EventSpec {
    pub from_jd_ut: f64,
    pub to_jd_ut: f64,
    pub kinds: Vec<SkyEventKind>,
    /// Graha seçimi (`SkyEventKind::uses_grahas` olan türler için).
    pub grahas: Vec<Graha>,
}

fn invalid(field: &'static str) -> CoreError {
    CoreError::InvalidSearchInput(field)
}

fn distinct<T: PartialEq>(values: &[T]) -> bool {
    values
        .iter()
        .enumerate()
        .all(|(i, v)| !values[..i].contains(v))
}

impl EventSpec {
    fn has(&self, kind: SkyEventKind) -> bool {
        self.kinds.contains(&kind)
    }

    /// Arama Ay'ın hareketini izler: aralık sınırı 5 yıldır ve dilimler kısadır.
    pub fn involves_moon(&self) -> bool {
        self.has(SkyEventKind::NewMoon)
            || self.has(SkyEventKind::FullMoon)
            || (self.grahas.contains(&Graha::Moon)
                && self.kinds.iter().any(|k| {
                    k.uses_grahas()
                        && !matches!(
                            k,
                            SkyEventKind::StationRetrograde | SkyEventKind::StationDirect
                        )
                }))
    }

    /// Girdi denetimi; hata alan adını taşır. `has_natal`: açık haritanın konumları verildi.
    pub fn validate(&self, has_natal: bool) -> Result<(), CoreError> {
        if self.kinds.is_empty() || !distinct(&self.kinds) {
            return Err(invalid("kinds"));
        }
        if !distinct(&self.grahas) {
            return Err(invalid("grahas"));
        }
        let uses_grahas = self.kinds.iter().any(|k| k.uses_grahas());
        let pairless = self.has(SkyEventKind::Conjunction) && self.grahas.len() < 2;
        if (uses_grahas && self.grahas.is_empty()) || pairless {
            return Err(invalid("grahas"));
        }
        if !has_natal && self.kinds.iter().any(|k| k.needs_natal()) {
            return Err(invalid("natal"));
        }
        // NaN ve sonsuz da aralık dışındadır.
        let in_range =
            JD_UT_RANGE.contains(&self.from_jd_ut) && JD_UT_RANGE.contains(&self.to_jd_ut);
        let max = if self.involves_moon() {
            EVENT_MOON_MAX_RANGE_DAYS
        } else {
            EVENT_MAX_RANGE_DAYS
        };
        if !in_range || self.from_jd_ut >= self.to_jd_ut || self.to_jd_ut - self.from_jd_ut > max {
            return Err(invalid("range"));
        }
        Ok(())
    }
}

/// Aramanın dilimleri, sırayla: `[from, to)` uçları. Girdi denetlenmiş olmalıdır.
pub fn event_chunks(spec: &EventSpec) -> Vec<(f64, f64)> {
    let step = if spec.involves_moon() {
        EVENT_MOON_CHUNK_DAYS
    } else {
        EVENT_CHUNK_DAYS
    };
    let span = spec.to_jd_ut - spec.from_jd_ut;
    if !span.is_finite() || span <= 0.0 {
        return Vec::new();
    }
    let n = (span / step).ceil().max(1.0) as u32;
    (0..n)
        .map(|i| {
            let start = spec.from_jd_ut + f64::from(i) * step;
            let end = (spec.from_jd_ut + f64::from(i + 1) * step).min(spec.to_jd_ut);
            (start, end)
        })
        .filter(|(start, end)| start < end)
        .collect()
}

/// Dizinin sınırları: (an, yeni değer).
fn boundaries<T: Copy>(series: &[Interval<T>]) -> impl Iterator<Item = (f64, T)> + '_ {
    series.windows(2).map(|w| (w[1].start_jd_ut, w[1].value))
}

fn rasi_lon<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    graha: Graha,
    jd_ut: f64,
) -> Result<f64, CoreError> {
    let (body, shift) = graha_body(graha, settings);
    Ok(normalize(
        lon_of(eph, jd_ut, body, rasi_frame(settings))? + shift,
    ))
}

fn sidereal_lon<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    graha: Graha,
    jd_ut: f64,
) -> Result<f64, CoreError> {
    let (body, shift) = graha_body(graha, settings);
    Ok(normalize(
        lon_of(eph, jd_ut, body, sidereal(settings))? + shift,
    ))
}

/// `a`, `b`'nin önündeki yarım çemberde: fark [0°, 180°). Değer, fark 0°'den ve 180°'den
/// geçerken değişir.
fn ahead(a: f64, b: f64) -> bool {
    normalize(a - b) < 180.0
}

fn single(kind: SkyEventKind, jd_ut: f64, graha: Graha) -> SkyEvent {
    SkyEvent {
        kind,
        jd_ut,
        grahas: vec![graha],
        sign: None,
        nakshatra: None,
        house: None,
    }
}

fn ingresses<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    spec: &EventSpec,
    (from, to): (f64, f64),
    out: &mut Vec<SkyEvent>,
) -> Result<(), CoreError> {
    for &graha in &spec.grahas {
        if spec.has(SkyEventKind::SignIngress) {
            let series = graha_sign_series(eph, settings, graha, from, to)?;
            out.extend(boundaries(&series).map(|(jd, sign)| SkyEvent {
                sign: Some(sign),
                ..single(SkyEventKind::SignIngress, jd, graha)
            }));
        }
        if spec.has(SkyEventKind::NakshatraIngress) {
            let series = graha_nakshatra_series(eph, settings, graha, from, to)?;
            out.extend(boundaries(&series).map(|(jd, nakshatra)| SkyEvent {
                nakshatra: Some(nakshatra),
                ..single(SkyEventKind::NakshatraIngress, jd, graha)
            }));
        }
    }
    Ok(())
}

fn stations<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    spec: &EventSpec,
    (from, to): (f64, f64),
    out: &mut Vec<SkyEvent>,
) -> Result<(), CoreError> {
    let (retro, direct) = (
        spec.has(SkyEventKind::StationRetrograde),
        spec.has(SkyEventKind::StationDirect),
    );
    if !retro && !direct {
        return Ok(());
    }
    for &graha in spec.grahas.iter().filter(|g| STATION_GRAHAS.contains(g)) {
        let series = graha_motion_series(eph, settings, graha, from, to)?;
        for (jd, motion) in boundaries(&series) {
            let kind = match motion {
                Motion::Retrograde if retro => SkyEventKind::StationRetrograde,
                Motion::Direct if direct => SkyEventKind::StationDirect,
                _ => continue,
            };
            let sign = Sign::of(rasi_lon(eph, settings, graha, jd)?);
            out.push(SkyEvent {
                sign: Some(sign),
                ..single(kind, jd, graha)
            });
        }
    }
    Ok(())
}

fn conjunctions<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    spec: &EventSpec,
    (from, to): (f64, f64),
    out: &mut Vec<SkyEvent>,
) -> Result<(), CoreError> {
    if !spec.has(SkyEventKind::Conjunction) {
        return Ok(());
    }
    // `Graha::ALL` sırasında çiftler; Rahu ile Ketu hep karşı karşıyadır.
    let chosen: Vec<Graha> = Graha::ALL
        .into_iter()
        .filter(|g| spec.grahas.contains(g))
        .collect();
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for i in 0..chosen.len() {
        for j in i + 1..chosen.len() {
            if !(chosen[i].is_node() && chosen[j].is_node()) {
                pairs.push((i, j));
            }
        }
    }
    if pairs.is_empty() {
        return Ok(());
    }
    let lons = |eph: &mut E, jd: f64| -> Result<Vec<f64>, CoreError> {
        chosen
            .iter()
            .map(|g| rasi_lon(eph, settings, *g, jd))
            .collect()
    };
    let step = if chosen.contains(&Graha::Moon) {
        MOON_STEP_DAYS
    } else {
        GRAHA_STEP_DAYS
    };
    // Her çift bir bit: önde olma durumu. En çok 35 çift vardır.
    let series = walk(eph, from, to, step, |eph, jd| {
        let l = lons(eph, jd)?;
        Ok(pairs
            .iter()
            .enumerate()
            .fold(0_u64, |mask, (bit, &(i, j))| {
                mask | (u64::from(ahead(l[i], l[j])) << bit)
            }))
    })?;
    for w in series.windows(2) {
        let jd = w[1].start_jd_ut;
        let changed = w[0].value ^ w[1].value;
        let l = lons(eph, jd)?;
        for (bit, &(i, j)) in pairs.iter().enumerate() {
            // Fark 180°'den de geçer (karşıt konum); o olay sayılmaz.
            if changed & (1 << bit) != 0 && angular_distance(l[i], l[j]) < 90.0 {
                out.push(SkyEvent {
                    kind: SkyEventKind::Conjunction,
                    jd_ut: jd,
                    grahas: vec![chosen[i], chosen[j]],
                    sign: Some(Sign::of(l[i])),
                    nakshatra: None,
                    house: None,
                });
            }
        }
    }
    Ok(())
}

fn lunations<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    spec: &EventSpec,
    (from, to): (f64, f64),
    out: &mut Vec<SkyEvent>,
) -> Result<(), CoreError> {
    let (new, full) = (
        spec.has(SkyEventKind::NewMoon),
        spec.has(SkyEventKind::FullMoon),
    );
    if !new && !full {
        return Ok(());
    }
    let zodiac = sidereal(settings);
    let series = walk(eph, from, to, MOON_STEP_DAYS, |eph, jd| {
        let sun = lon_of(eph, jd, Body::Sun, zodiac)?;
        let moon = lon_of(eph, jd, Body::Moon, zodiac)?;
        Ok(ahead(moon, sun))
    })?;
    for (jd, waxing) in boundaries(&series) {
        // Uzanım 0°'den geçince Ay büyümeye başlar (yeni ay); 180°'den geçince küçülmeye.
        let kind = match waxing {
            true if new => SkyEventKind::NewMoon,
            false if full => SkyEventKind::FullMoon,
            _ => continue,
        };
        let sign = Sign::of(rasi_lon(eph, settings, Graha::Moon, jd)?);
        let nakshatra = nakshatra_pada(sidereal_lon(eph, settings, Graha::Moon, jd)?).0;
        out.push(SkyEvent {
            kind,
            jd_ut: jd,
            grahas: vec![Graha::Sun, Graha::Moon],
            sign: Some(sign),
            nakshatra: Some(nakshatra),
            house: None,
        });
    }
    Ok(())
}

fn natal_events<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    spec: &EventSpec,
    natal: &Positions,
    (from, to): (f64, f64),
    out: &mut Vec<SkyEvent>,
) -> Result<(), CoreError> {
    if spec.has(SkyEventKind::TransitToNatal) {
        for &graha in &spec.grahas {
            let step = if graha == Graha::Moon {
                MOON_STEP_DAYS
            } else {
                GRAHA_STEP_DAYS
            };
            // Dokuz doğum boylamı dokuz bit: geçiş grahası her birinin önünde mi.
            let series = walk(eph, from, to, step, |eph, jd| {
                let lon = sidereal_lon(eph, settings, graha, jd)?;
                Ok(natal
                    .grahas
                    .iter()
                    .enumerate()
                    .fold(0_u16, |mask, (bit, p)| {
                        mask | (u16::from(ahead(lon, p.sidereal_lon)) << bit)
                    }))
            })?;
            for w in series.windows(2) {
                let jd = w[1].start_jd_ut;
                let changed = w[0].value ^ w[1].value;
                let lon = sidereal_lon(eph, settings, graha, jd)?;
                for (bit, p) in natal.grahas.iter().enumerate() {
                    if changed & (1 << bit) != 0 && angular_distance(lon, p.sidereal_lon) < 90.0 {
                        out.push(SkyEvent {
                            kind: SkyEventKind::TransitToNatal,
                            jd_ut: jd,
                            grahas: vec![graha, p.graha],
                            sign: Some(p.rasi_sign),
                            nakshatra: None,
                            house: None,
                        });
                    }
                }
            }
        }
    }
    if spec.has(SkyEventKind::HouseFromNatalMoon) {
        let moon_sign = natal.sign_of(Graha::Moon);
        for graha in HOUSE_CHANGE_GRAHAS {
            let series = graha_sign_series(eph, settings, graha, from, to)?;
            out.extend(boundaries(&series).map(|(jd, sign)| SkyEvent {
                sign: Some(sign),
                house: Some(sign.house_from(moon_sign)),
                ..single(SkyEventKind::HouseFromNatalMoon, jd, graha)
            }));
        }
    }
    Ok(())
}

/// Bir dilimin olayları, zaman sırasıyla. `chunk` `event_chunks`'in verdiği bir dilimdir.
/// `natal` açık haritanın konumlarıdır (yalnızca sidereal boylamlar ve Ay'ın rasi burcu
/// kullanılır).
pub fn event_search_chunk<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    spec: &EventSpec,
    natal: Option<&Positions>,
    chunk: (f64, f64),
) -> Result<Vec<SkyEvent>, CoreError> {
    settings.validate()?;
    spec.validate(natal.is_some())?;
    if !(spec.from_jd_ut <= chunk.0 && chunk.0 < chunk.1 && chunk.1 <= spec.to_jd_ut) {
        return Err(invalid("range"));
    }
    let mut out = Vec::new();
    ingresses(eph, settings, spec, chunk, &mut out)?;
    stations(eph, settings, spec, chunk, &mut out)?;
    conjunctions(eph, settings, spec, chunk, &mut out)?;
    lunations(eph, settings, spec, chunk, &mut out)?;
    if let Some(natal) = natal {
        natal_events(eph, settings, spec, natal, chunk, &mut out)?;
    }
    // Kararlı sıralama: aynı andaki olaylar tür sırasında durur.
    out.sort_by(|a, b| a.jd_ut.total_cmp(&b.jd_ut));
    Ok(out)
}

/// Aramanın tamamı, dilimleri sırayla yürüterek (testler ve tek seferlik kullanım). Uygulama
/// dilimleri kendisi yürütür ve aralarında motor iş parçacığını bırakır.
pub fn event_search<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    spec: &EventSpec,
    natal: Option<&Positions>,
) -> Result<Vec<SkyEvent>, CoreError> {
    settings.validate()?;
    spec.validate(natal.is_some())?;
    let mut out = Vec::new();
    for chunk in event_chunks(spec) {
        out.extend(event_search_chunk(eph, settings, spec, natal, chunk)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use ephemeris::{
        Ayanamsa, BodyPosition, EphError, HouseSystem, Houses, RiseSet, SiderealMode, SunDisc,
        Zodiac,
    };

    use super::*;
    use crate::fake::FakeEph;

    const JD: f64 = 2_451_545.0;
    /// Yarım saniye, gün olarak: sınır çözünürlüğünün (0,05 sn) on katı.
    const HALF_SECOND: f64 = 0.5 / 86_400.0;
    const ARCSECOND: f64 = 1.0 / 3600.0;

    fn spec(days: f64, kinds: &[SkyEventKind], grahas: &[Graha]) -> EventSpec {
        EventSpec {
            from_jd_ut: JD,
            to_jd_ut: JD + days,
            kinds: kinds.to_vec(),
            grahas: grahas.to_vec(),
        }
    }

    fn search(spec: &EventSpec, natal: Option<&Positions>) -> Vec<SkyEvent> {
        event_search(
            &mut FakeEph::default(),
            &CalcSettings::default(),
            spec,
            natal,
        )
        .unwrap()
    }

    fn field(r: Result<(), CoreError>) -> &'static str {
        match r {
            Err(CoreError::InvalidSearchInput(f)) => f,
            other => panic!("expected an invalid search input, got {other:?}"),
        }
    }

    /// Sahte ephemeris'in sidereal başlangıç boylamları (tropikal − 24°): Güneş 346°
    /// (0,98°/gün), Ay 71° (13,2°/gün), Jüpiter 226° (−0,1°/gün), Satürn 276° (0,05°/gün).
    #[test]
    fn sign_and_nakshatra_ingresses_include_retrograde_entries() {
        let s = spec(
            200.0,
            &[SkyEventKind::SignIngress],
            &[Graha::Sun, Graha::Jupiter],
        );
        let found = search(&s, None);
        // Güneş Koç'a 14 / 0,98 gün sonra girer; sonra her 30 / 0,98 günde bir burç.
        let sun: Vec<&SkyEvent> = found.iter().filter(|e| e.grahas == [Graha::Sun]).collect();
        assert_eq!(sun.len(), 7);
        assert!((sun[0].jd_ut - (JD + 14.0 / 0.98)).abs() < HALF_SECOND);
        assert_eq!(sun[0].sign, Some(Sign::Aries));
        assert_eq!(sun[1].sign, Some(Sign::Taurus));
        // Jüpiter Akrep 16°'den geri gider: 160 gün sonra Terazi'ye girer.
        let jupiter: Vec<&SkyEvent> = found
            .iter()
            .filter(|e| e.grahas == [Graha::Jupiter])
            .collect();
        assert_eq!(jupiter.len(), 1);
        assert!((jupiter[0].jd_ut - (JD + 160.0)).abs() < HALF_SECOND);
        assert_eq!(jupiter[0].sign, Some(Sign::Libra));
        // Sıra zaman sırasıdır.
        assert!(found.windows(2).all(|w| w[0].jd_ut <= w[1].jd_ut));

        // Ay 71°: 6. nakshatranın (66°40′–80°) içindedir; 7. nakshatraya 9 / 13,2 gün sonra
        // girer.
        let s = spec(2.0, &[SkyEventKind::NakshatraIngress], &[Graha::Moon]);
        let found = search(&s, None);
        assert_eq!(found.len(), 2);
        assert!((found[0].jd_ut - (JD + 9.0 / 13.2)).abs() < HALF_SECOND);
        assert_eq!((found[0].nakshatra, found[0].sign), (Some(7), None));
        assert_eq!(found[1].nakshatra, Some(8));
    }

    #[test]
    fn new_and_full_moons_are_at_zero_and_half_a_turn_within_an_arcsecond() {
        // Uzanım 85° ve günde 12,22° büyür: dolunay (180 − 85) / 12,22, yeni ay
        // (360 − 85) / 12,22 gün sonra.
        let s = spec(40.0, &[SkyEventKind::NewMoon, SkyEventKind::FullMoon], &[]);
        let found = search(&s, None);
        let kinds: Vec<SkyEventKind> = found.iter().map(|e| e.kind).collect();
        assert_eq!(
            kinds,
            [
                SkyEventKind::FullMoon,
                SkyEventKind::NewMoon,
                SkyEventKind::FullMoon
            ]
        );
        assert!((found[0].jd_ut - (JD + 95.0 / 12.22)).abs() < HALF_SECOND);
        assert!((found[1].jd_ut - (JD + 275.0 / 12.22)).abs() < HALF_SECOND);
        let mut eph = FakeEph::default();
        let settings = CalcSettings::default();
        for e in &found {
            let sun = sidereal_lon(&mut eph, &settings, Graha::Sun, e.jd_ut).unwrap();
            let moon = sidereal_lon(&mut eph, &settings, Graha::Moon, e.jd_ut).unwrap();
            let want = if e.kind == SkyEventKind::NewMoon {
                0.0
            } else {
                180.0
            };
            let off = angular_distance(normalize(moon - sun), want);
            assert!(off <= ARCSECOND, "{:?}: {off}", e.kind);
            assert_eq!(e.grahas, [Graha::Sun, Graha::Moon]);
            assert_eq!(e.sign, Some(Sign::of(moon)));
            assert_eq!(e.nakshatra, Some(nakshatra_pada(moon).0));
        }
        // Yalnızca istenen tür döner.
        let only_new = search(&spec(40.0, &[SkyEventKind::NewMoon], &[]), None);
        assert_eq!(only_new.len(), 1);
    }

    #[test]
    fn conjunctions_are_exact_within_an_arcsecond_and_oppositions_are_not_reported() {
        let s = spec(
            40.0,
            &[SkyEventKind::Conjunction],
            &[Graha::Moon, Graha::Sun, Graha::Rahu, Graha::Ketu],
        );
        let found = search(&s, None);
        let mut eph = FakeEph::default();
        let settings = CalcSettings::default();
        assert!(!found.is_empty());
        for e in &found {
            assert_eq!(e.kind, SkyEventKind::Conjunction);
            assert_eq!(e.grahas.len(), 2);
            // Rahu ile Ketu çifti hiç aranmaz.
            assert_ne!(e.grahas, [Graha::Rahu, Graha::Ketu]);
            let a = sidereal_lon(&mut eph, &settings, e.grahas[0], e.jd_ut).unwrap();
            let b = sidereal_lon(&mut eph, &settings, e.grahas[1], e.jd_ut).unwrap();
            assert!(angular_distance(a, b) <= ARCSECOND, "{:?}", e.grahas);
            assert_eq!(e.sign, Some(Sign::of(a)));
        }
        // Güneş–Ay kavuşumu yeni ay anıdır; çift `Graha::ALL` sırasındadır.
        let sun_moon: Vec<&SkyEvent> = found
            .iter()
            .filter(|e| e.grahas == [Graha::Sun, Graha::Moon])
            .collect();
        assert_eq!(sun_moon.len(), 1);
        assert!((sun_moon[0].jd_ut - (JD + 275.0 / 12.22)).abs() < HALF_SECOND);
        // Ay 40 günde düğümlerin her biriyle en az bir kez kavuşur.
        for node in [Graha::Rahu, Graha::Ketu] {
            assert!(found.iter().any(|e| e.grahas == [Graha::Moon, node]));
        }
    }

    /// Mars'ın boylamı sinüsle salınan sahte ephemeris: hız 50. ve 150. günlerde sıfırdan geçer.
    struct Wobble(FakeEph);

    impl Ephemeris for Wobble {
        fn body(
            &mut self,
            jd_ut: f64,
            body: Body,
            zodiac: Zodiac,
        ) -> Result<BodyPosition, EphError> {
            let mut p = self.0.body(jd_ut, body, zodiac)?;
            if body == Body::Mars {
                let phase = std::f64::consts::TAU * (jd_ut - JD) / 200.0;
                p.lon = normalize(100.0 + 20.0 * phase.sin());
                p.lon_speed = 20.0 * std::f64::consts::TAU / 200.0 * phase.cos();
            }
            Ok(p)
        }
        fn ayanamsa(&mut self, jd_ut: f64, mode: SiderealMode) -> Result<Ayanamsa, EphError> {
            self.0.ayanamsa(jd_ut, mode)
        }
        fn houses(
            &mut self,
            jd_ut: f64,
            geo_lat: f64,
            geo_lon: f64,
            system: HouseSystem,
            zodiac: Zodiac,
        ) -> Result<Houses, EphError> {
            self.0.houses(jd_ut, geo_lat, geo_lon, system, zodiac)
        }
        fn sun_rise_set(
            &mut self,
            after_jd_ut: f64,
            event: RiseSet,
            geo_lat: f64,
            geo_lon: f64,
            disc: SunDisc,
        ) -> Result<Option<f64>, EphError> {
            self.0
                .sun_rise_set(after_jd_ut, event, geo_lat, geo_lon, disc)
        }
        fn swe_version(&self) -> String {
            self.0.swe_version()
        }
    }

    #[test]
    fn stations_are_the_zero_crossings_of_the_speed() {
        let both = [SkyEventKind::StationRetrograde, SkyEventKind::StationDirect];
        // Güneş, Ay ve düğümler dönüş vermez; seçimde olmaları hata değildir.
        let s = spec(200.0, &both, &[Graha::Sun, Graha::Mars, Graha::Rahu]);
        let mut eph = Wobble(FakeEph::default());
        let settings = CalcSettings::default();
        let found = event_search(&mut eph, &settings, &s, None).unwrap();
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].kind, SkyEventKind::StationRetrograde);
        assert!((found[0].jd_ut - (JD + 50.0)).abs() < 1e-3);
        assert_eq!(found[1].kind, SkyEventKind::StationDirect);
        assert!((found[1].jd_ut - (JD + 150.0)).abs() < 1e-3);
        for e in &found {
            assert_eq!(e.grahas, [Graha::Mars]);
        }
        // 50. günde boylam 120°'dir: Aslan'ın başı; 150. günde 80°: İkizler.
        assert_eq!(found[1].sign, Some(Sign::Gemini));
        let only = spec(200.0, &[SkyEventKind::StationDirect], &[Graha::Mars]);
        let found = event_search(&mut eph, &settings, &only, None).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, SkyEventKind::StationDirect);
    }

    #[test]
    fn transits_to_natal_longitudes_and_house_changes_from_the_natal_moon() {
        // Doğum: Güneş 0° (Koç), Ay 100° (Yengeç), diğerleri 200°.
        let natal = Positions::synthetic(
            None,
            [0.0, 100.0, 200.0, 200.0, 200.0, 200.0, 200.0, 200.0, 200.0],
        );
        let s = spec(400.0, &[SkyEventKind::TransitToNatal], &[Graha::Sun]);
        let found = search(&s, Some(&natal));
        // Güneş 346°'den 0°'ye 14 / 0,98 gün sonra gelir (kendi boylamına dönüş); sonra Ay'ın
        // boylamına (100°) 114 / 0,98, diğer yedisinin boylamına (200°) 214 / 0,98 gün sonra;
        // ikinci dönüş 374 / 0,98 gün sonradır.
        let at = |days: f64| JD + days / 0.98;
        assert_eq!(found.len(), 10);
        assert_eq!(found[0].grahas, [Graha::Sun, Graha::Sun]);
        assert!((found[0].jd_ut - at(14.0)).abs() < HALF_SECOND);
        assert_eq!(found[0].sign, Some(Sign::Aries));
        assert_eq!(found[1].grahas, [Graha::Sun, Graha::Moon]);
        assert!((found[1].jd_ut - at(114.0)).abs() < HALF_SECOND);
        assert_eq!(found[1].sign, Some(Sign::Cancer));
        assert!(found[2..9]
            .iter()
            .all(|e| (e.jd_ut - at(214.0)).abs() < HALF_SECOND && e.grahas[0] == Graha::Sun));
        assert_eq!(found[9].grahas, [Graha::Sun, Graha::Sun]);
        assert!((found[9].jd_ut - at(374.0)).abs() < HALF_SECOND);

        // Jüpiter Akrep 16°'den günde 0,1° geri gider: 160 gün sonra Terazi'ye (Yengeç'ten
        // 4. burç), 460 gün sonra Başak'a (3. burç) girer. Satürn Oğlak 6°'den günde 0,05°
        // ilerler: 480 gün sonra Kova'ya girer (8. burç). Seçili graha listesi bu türü etkilemez.
        let s = spec(500.0, &[SkyEventKind::HouseFromNatalMoon], &[]);
        let found = search(&s, Some(&natal));
        let got: Vec<(Graha, Option<Sign>, Option<u8>)> = found
            .iter()
            .map(|e| (e.grahas[0], e.sign, e.house))
            .collect();
        assert_eq!(
            got,
            [
                (Graha::Jupiter, Some(Sign::Libra), Some(4)),
                (Graha::Jupiter, Some(Sign::Virgo), Some(3)),
                (Graha::Saturn, Some(Sign::Aquarius), Some(8)),
            ]
        );
        for (e, days) in found.iter().zip([160.0, 460.0, 480.0]) {
            assert!((e.jd_ut - (JD + days)).abs() < HALF_SECOND);
        }
    }

    #[test]
    fn the_spec_is_checked_by_field_name() {
        let ingress = [SkyEventKind::SignIngress];
        let ok = spec(30.0, &ingress, &[Graha::Sun]);
        assert_eq!(ok.validate(false), Ok(()));
        assert_eq!(
            field(spec(30.0, &[], &[Graha::Sun]).validate(false)),
            "kinds"
        );
        assert_eq!(
            field(spec(30.0, &[SkyEventKind::NewMoon, SkyEventKind::NewMoon], &[]).validate(false)),
            "kinds"
        );
        assert_eq!(field(spec(30.0, &ingress, &[]).validate(false)), "grahas");
        assert_eq!(
            field(spec(30.0, &ingress, &[Graha::Sun, Graha::Sun]).validate(false)),
            "grahas"
        );
        assert_eq!(
            field(spec(30.0, &[SkyEventKind::Conjunction], &[Graha::Sun]).validate(false)),
            "grahas"
        );
        // Açık harita isteyen türler haritasız reddedilir.
        for kind in [
            SkyEventKind::TransitToNatal,
            SkyEventKind::HouseFromNatalMoon,
        ] {
            let s = spec(30.0, &[kind], &[Graha::Saturn]);
            assert_eq!(field(s.validate(false)), "natal");
            assert_eq!(s.validate(true), Ok(()));
        }
        // Aralık: sıra, hesap aralığı ve uzunluk (Ay'sız 120 yıl, Ay'lı 5 yıl).
        assert_eq!(
            field(spec(0.0, &ingress, &[Graha::Sun]).validate(false)),
            "range"
        );
        assert_eq!(
            field(spec(-1.0, &ingress, &[Graha::Sun]).validate(false)),
            "range"
        );
        assert_eq!(
            field(spec(f64::NAN, &ingress, &[Graha::Sun]).validate(false)),
            "range"
        );
        let long = spec(EVENT_MAX_RANGE_DAYS, &ingress, &[Graha::Saturn]);
        assert_eq!(long.validate(false), Ok(()));
        assert_eq!(
            field(spec(EVENT_MAX_RANGE_DAYS + 1.0, &ingress, &[Graha::Saturn]).validate(false)),
            "range"
        );
        let moon = spec(EVENT_MOON_MAX_RANGE_DAYS, &ingress, &[Graha::Moon]);
        assert!(moon.involves_moon());
        assert_eq!(moon.validate(false), Ok(()));
        for s in [
            spec(EVENT_MOON_MAX_RANGE_DAYS + 1.0, &ingress, &[Graha::Moon]),
            spec(
                EVENT_MOON_MAX_RANGE_DAYS + 1.0,
                &[SkyEventKind::FullMoon],
                &[],
            ),
        ] {
            assert_eq!(field(s.validate(false)), "range");
        }
        // Ay yalnızca dönüş aramasında seçiliyse arama Ay'ı izlemez.
        let stations = spec(
            EVENT_MAX_RANGE_DAYS,
            &[SkyEventKind::StationRetrograde],
            &[Graha::Moon, Graha::Mars],
        );
        assert!(!stations.involves_moon());
        assert_eq!(stations.validate(false), Ok(()));
        // Dilim aramanın aralığında olmalıdır; hata değer taşımaz.
        let err = event_search_chunk(
            &mut FakeEph::default(),
            &CalcSettings::default(),
            &ok,
            None,
            (JD - 1.0, JD + 1.0),
        )
        .unwrap_err();
        assert_eq!(err.to_string(), "invalid search input: range");
    }

    #[test]
    fn chunks_tile_the_range_and_find_each_event_exactly_once() {
        let slow = spec(4_000.0, &[SkyEventKind::SignIngress], &[Graha::Saturn]);
        let chunks = event_chunks(&slow);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], (JD, JD + EVENT_CHUNK_DAYS));
        assert_eq!(chunks[2].1, JD + 4_000.0);
        assert!(chunks.windows(2).all(|w| w[0].1 == w[1].0));
        let moon = spec(
            200.0,
            &[
                SkyEventKind::NakshatraIngress,
                SkyEventKind::NewMoon,
                SkyEventKind::FullMoon,
                SkyEventKind::Conjunction,
            ],
            &[Graha::Moon, Graha::Sun],
        );
        let chunks = event_chunks(&moon);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], (JD, JD + EVENT_MOON_CHUNK_DAYS));
        // Dilim dilim arama, tek dilimlik aramayla aynı olayları verir.
        let mut eph = FakeEph::default();
        let settings = CalcSettings::default();
        let whole = event_search_chunk(&mut eph, &settings, &moon, None, (JD, JD + 200.0)).unwrap();
        let tiled = event_search(&mut eph, &settings, &moon, None).unwrap();
        assert_eq!(tiled.len(), whole.len());
        // 200 günde 200 × 13,2 / 13°20′ = 198 nakshatra girişi, 6 ya da 7 yeni ay.
        assert!(whole.len() > 200);
        // Aynı andaki olayların (yeni ay ve Güneş–Ay kavuşumu) sırası çözünürlük içinde
        // değişebilir: türler ayrı ayrı karşılaştırılır.
        for kind in SkyEventKind::ALL {
            let of = |events: &[SkyEvent]| -> Vec<SkyEvent> {
                events.iter().filter(|e| e.kind == kind).cloned().collect()
            };
            let (a, b) = (of(&tiled), of(&whole));
            assert_eq!(a.len(), b.len(), "{kind:?}");
            for (x, y) in a.iter().zip(&b) {
                assert_eq!((&x.grahas, x.nakshatra), (&y.grahas, y.nakshatra));
                assert!((x.jd_ut - y.jd_ut).abs() < HALF_SECOND);
            }
        }
        assert!(tiled.windows(2).all(|w| w[0].jd_ut <= w[1].jd_ut));
    }
}
