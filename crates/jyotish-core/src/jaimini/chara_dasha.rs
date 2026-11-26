//! Chara dasha, padakrama düzeni (ileri modüller spec'i §6). Burç temelli Jaimini dashası:
//! on iki burcun her biri bir dönemdir; sıra ve süre burç sayımından çıkar.
//!
//! Kaynak: Jaimini Upadesha Sutraları, dasha bölümü; 20. yüzyıl yorum geleneğinin padakrama
//! düzeni. Kural adımları:
//! 1. İlk dasha lagna burcudur. Lagnadan 9. burç düz sayan gruptaysa (`counts_forward`) dashalar
//!    zodyak sırasıyla, değilse ters sırayla ilerler.
//! 2. Bir burcun süresi, burçtan lorduna burcun kendi yönünde kapsayıcı sayımın bir eksiğidir;
//!    lord kendi burcundaysa 12 yıldır. Akrep ve Kova'nın lordu `jaimini_lord` ile seçilir.
//! 3. On iki dashadan sonra ikinci çevrim gelir: her burcun süresi `12 − ilk çevrimdeki yıl`dır
//!    ve 0 yıllık burçlar atlanır. İkinci çevrimin bir dashası, başlangıcı doğumdan sonraki
//!    120. yıldan önceyse üretilir.
//! 4. Her dönem on iki eşit alt döneme bölünür; sıra haritanın yönündedir, ilk alt dönem dasha
//!    burcundan sonraki burç, sonuncusu dasha burcunun kendisidir. En çok üç seviye.
//!
//! Doğumda ilk dasha tam süresiyle başlar. Zamanlar UT Jülyen günüdür. Yücelme ve düşüş için
//! yıl düzeltmesi uygulanmaz.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::compute_chart;
use crate::positions::{ChartInput, Positions};
use crate::zodiac::Sign;
use crate::CoreError;

use super::lords::jaimini_lord;
use super::rashi::{counts_forward, directed_count};

/// `CharaDasha::method`: bu modüldeki yöntemin kimliği.
pub const CHARA_DASHA_METHOD: &str = "chara_dasha.padakrama_v1";
/// 1 dasha, 2 alt dönem, 3 alt-alt dönem.
pub const MAX_SIGN_DASHA_LEVEL: u8 = 3;
/// `CharaDasha::periods`'un taşıdığı seviye sayısı; 3. seviye `sign_dasha_children` ile istenir.
pub const CHARA_DASHA_LEVELS: u8 = 2;
/// İkinci çevrimin bir dashası, başlangıcı doğumdan bu kadar yıl geçmeden önceyse üretilir.
pub const CHARA_DASHA_HORIZON_YEARS: u16 = 120;

/// Burç dashasının bir dönemi. Zamanlar UT Jülyen günüdür.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SignDashaPeriod {
    /// Kökten bu döneme burçlar: `[dasha]`, `[dasha, alt dönem]`, …
    pub signs: Vec<Sign>,
    /// 1..=3; `signs.len()` ile aynı.
    pub level: u8,
    /// 1 ya da 2: dönemin ait olduğu çevrim.
    pub cycle: u8,
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
    /// Alt dönemleri istenebilir (`level < MAX_SIGN_DASHA_LEVEL`).
    pub has_children: bool,
}

/// `compute_chara_dasha` sonucu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CharaDasha {
    /// Dashalar zodyak sırasıyla mı ilerliyor (lagnadan 9. burç düz sayan gruptaysa).
    pub forward: bool,
    /// İlk çevrimin yılları, lagnadan dasha sırasıyla.
    pub years: [u8; 12],
    /// İlk iki seviye, ön-sırada: her dashanın ardından on iki alt dönemi. İki çevrim de
    /// buradadır; `cycle` ayırır.
    pub periods: Vec<SignDashaPeriod>,
    pub method: String,
}

/// Bir anı içeren dönemlerin yolu: çevrim ve kökten burçlar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SignDashaPath {
    pub cycle: u8,
    pub signs: Vec<Sign>,
}

/// Dashalar zodyak sırasıyla mı ilerler: lagnadan 9. burç düz sayan gruptaysa evet.
pub fn chara_dasha_forward(lagna: Sign) -> bool {
    counts_forward(Sign::from_index(lagna.index() + 8))
}

/// `from`'dan başlayan on iki burç: ileri yönde zodyak sırasıyla, geri yönde ters sırayla.
pub fn sign_sequence(from: Sign, forward: bool) -> [Sign; 12] {
    std::array::from_fn(|k| {
        let step = if forward { k } else { 12 - k };
        Sign::from_index(from.index() + step)
    })
}

/// Dasha sırası: lagna burcundan başlar, on iki burç birer kez gelir.
pub fn chara_dasha_order(lagna: Sign) -> [Sign; 12] {
    sign_sequence(lagna, chara_dasha_forward(lagna))
}

/// Yıl kuralı: `sign`'dan lordunun burcuna, `sign`'ın kendi yönünde kapsayıcı sayım eksi bir;
/// lord kendi burcundaysa 12. Sonuç 1..=12.
pub fn chara_dasha_years(sign: Sign, lord_sign: Sign) -> u8 {
    match directed_count(sign, lord_sign) - 1 {
        0 => 12,
        n => n,
    }
}

/// `sign` burcunun ilk çevrimdeki yılı. Akrep ve Kova'nın lordu `jaimini_lord` ile seçilir;
/// iki eş lord da burcun içindeyse seçilen lord burcun içindedir ve süre 12 yıldır.
pub fn chara_dasha_sign_years(sign: Sign, positions: &Positions, co_lords: bool) -> u8 {
    let lord = jaimini_lord(sign, positions, co_lords);
    chara_dasha_years(sign, positions.sign_of(lord))
}

/// İkinci çevrimdeki yıl: `12 − ilk çevrimdeki yıl` (0: burç atlanır).
pub fn second_cycle_years(first_cycle_years: u8) -> u8 {
    12 - first_cycle_years
}

/// Alt dönem sırası: `sign`'dan sonraki burçtan başlar, `sign`'ın kendisiyle biter.
pub fn sub_period_signs(sign: Sign, forward: bool) -> [Sign; 12] {
    let sequence = sign_sequence(sign, forward);
    std::array::from_fn(|k| sequence[(k + 1) % 12])
}

/// Dönemi on iki eşit alt döneme böler. Bitişler baştan hesaplanır; sonuncusu ebeveynin
/// bitişine tam eşittir.
fn subdivide(parent: &SignDashaPeriod, forward: bool) -> Vec<SignDashaPeriod> {
    let level = parent.level + 1;
    let span = parent.end_jd_ut - parent.start_jd_ut;
    let dasha_sign = *parent.signs.last().expect("a period has a sign");
    let mut from = parent.start_jd_ut;
    sub_period_signs(dasha_sign, forward)
        .into_iter()
        .enumerate()
        .map(|(k, sign)| {
            let to = if k == 11 {
                parent.end_jd_ut
            } else {
                parent.start_jd_ut + span * (k as f64 + 1.0) / 12.0
            };
            let mut signs = parent.signs.clone();
            signs.push(sign);
            let period = SignDashaPeriod {
                signs,
                level,
                cycle: parent.cycle,
                start_jd_ut: from,
                end_jd_ut: to,
                has_children: level < MAX_SIGN_DASHA_LEVEL,
            };
            from = to;
            period
        })
        .collect()
}

/// Saf hesap: konumlardan chara dasha. Lagna yoksa (doğum saati bilinmiyor) `None`.
/// `birth_jd_ut` ilk dashanın başlangıcıdır; `year_days` yıl uzunluğudur
/// (`CalcSettings::dasha_year`).
pub fn chara_dasha(
    positions: &Positions,
    co_lords: bool,
    birth_jd_ut: f64,
    year_days: f64,
) -> Option<CharaDasha> {
    let lagna = positions.lagna_sign?;
    let forward = chara_dasha_forward(lagna);
    let order = sign_sequence(lagna, forward);
    let years = order.map(|sign| chara_dasha_sign_years(sign, positions, co_lords));
    let mut periods = Vec::new();
    let mut elapsed: u16 = 0;
    for cycle in 1..=2u8 {
        for (i, &sign) in order.iter().enumerate() {
            let length = if cycle == 1 {
                years[i]
            } else {
                second_cycle_years(years[i])
            };
            if length == 0 {
                continue;
            }
            if cycle == 2 && elapsed >= CHARA_DASHA_HORIZON_YEARS {
                break;
            }
            let until = elapsed + u16::from(length);
            let dasha = SignDashaPeriod {
                signs: vec![sign],
                level: 1,
                cycle,
                start_jd_ut: birth_jd_ut + f64::from(elapsed) * year_days,
                end_jd_ut: birth_jd_ut + f64::from(until) * year_days,
                has_children: true,
            };
            let children = subdivide(&dasha, forward);
            periods.push(dasha);
            periods.extend(children);
            elapsed = until;
        }
    }
    Some(CharaDasha {
        forward,
        years,
        periods,
        method: CHARA_DASHA_METHOD.to_owned(),
    })
}

/// `cycle` çevriminde `path` ile seçilen dönemin alt dönemleri. `[]` o çevrimin dashalarını,
/// `[dasha]` on iki alt dönemi, `[dasha, alt dönem]` on iki 3. seviye dönemi verir. Daha uzun
/// yol, 1 ve 2 dışındaki çevrim ve o çevrimde üretilmemiş bir dasha hatadır.
pub fn sign_dasha_children(
    dasha: &CharaDasha,
    path: &[Sign],
    cycle: u8,
) -> Result<Vec<SignDashaPeriod>, CoreError> {
    if !(1..=2).contains(&cycle) {
        return Err(CoreError::InvalidDashaInput("cycle"));
    }
    if path.len() >= usize::from(MAX_SIGN_DASHA_LEVEL) {
        return Err(CoreError::InvalidDashaInput("path"));
    }
    let in_cycle = || dasha.periods.iter().filter(move |p| p.cycle == cycle);
    if path.is_empty() {
        return Ok(in_cycle().filter(|p| p.level == 1).cloned().collect());
    }
    let parent = in_cycle()
        .find(|p| p.signs == path)
        .ok_or(CoreError::InvalidDashaInput("path"))?;
    if parent.level < CHARA_DASHA_LEVELS {
        let level = parent.level + 1;
        return Ok(in_cycle()
            .filter(|p| p.level == level && p.signs.starts_with(path))
            .cloned()
            .collect());
    }
    Ok(subdivide(parent, dasha.forward))
}

/// `jd_ut` anını içeren dönemlerin yolu, üç seviye. An üretilen dönemlerin dışındaysa (doğumdan
/// önce ya da son dashadan sonra) `None`. Arayüzün "güncel dönem"i buradan gelir; arayüz tarih
/// karşılaştırması yapmaz.
pub fn sign_dasha_path_at(
    dasha: &CharaDasha,
    jd_ut: f64,
) -> Result<Option<SignDashaPath>, CoreError> {
    if !jd_ut.is_finite() {
        return Err(CoreError::InvalidDashaInput("jd_ut"));
    }
    let holds = |p: &&SignDashaPeriod| p.start_jd_ut <= jd_ut && jd_ut < p.end_jd_ut;
    let Some(second) = dasha
        .periods
        .iter()
        .filter(|p| p.level == CHARA_DASHA_LEVELS)
        .find(holds)
    else {
        return Ok(None);
    };
    // Alt dönemler ebeveyni boşluksuz böler: içeren bir 3. seviye dönem her zaman vardır.
    let third = subdivide(second, dasha.forward);
    let signs = third
        .iter()
        .find(holds)
        .map_or_else(|| second.signs.clone(), |p| p.signs.clone());
    Ok(Some(SignDashaPath {
        cycle: second.cycle,
        signs,
    }))
}

/// `compute_chara_dasha` komutunun hesabı: temel harita, konumlar, dasha. Doğum saati
/// bilinmiyorsa (`input.time_known == false`) `None`: sıra lagnaya bağlıdır. Ayar, an ve konum
/// denetimi `compute_chart`'ınkidir ve saat bilinmese de yapılır.
pub fn compute_chara_dasha<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<Option<CharaDasha>, CoreError> {
    let chart = compute_chart(eph, input.jd_ut, input.lat, input.lon, &input.settings)?;
    if !input.time_known {
        return Ok(None);
    }
    Ok(chara_dasha(
        &Positions::from(&chart),
        input.settings.advanced.jaimini_co_lords,
        input.jd_ut,
        input.settings.dasha_year,
    ))
}

/// `chara_dasha_children` komutunun hesabı. Doğum saati bilinmiyorsa boş liste.
pub fn compute_chara_dasha_children<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
    path: &[Sign],
    cycle: u8,
) -> Result<Vec<SignDashaPeriod>, CoreError> {
    match compute_chara_dasha(eph, input)? {
        Some(dasha) => sign_dasha_children(&dasha, path, cycle),
        None => Ok(Vec::new()),
    }
}

/// `chara_dasha_at` komutunun hesabı: `jd_ut` anının dönem yolu. Doğum saati bilinmiyorsa ya da
/// an üretilen dönemlerin dışındaysa `None`.
pub fn compute_chara_dasha_at<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
    jd_ut: f64,
) -> Result<Option<SignDashaPath>, CoreError> {
    match compute_chara_dasha(eph, input)? {
        Some(dasha) => sign_dasha_path_at(&dasha, jd_ut),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::{AdvancedSettings, CalcSettings, KarakaScheme};
    use Sign::*;

    const JD: f64 = 2_451_545.0;
    const YEAR: f64 = 365.25;

    /// Elle yazılmış yıl tablosu (bağımsız ikinci yazım, spec §6 "Çapraz denetim", §13): satır
    /// dasha burcu, sütun lordun bulunduğu burç (Koç…Balık). 12 × 12 = 144 hücre. Düz sayan
    /// burçlarda (Koç, Boğa, İkizler, Terazi, Akrep, Yay) lord bir sonraki burçtaysa 1, bir
    /// önceki burçtaysa 11 yıl; ters sayanlarda tersi; lord kendi burcundaysa 12.
    #[rustfmt::skip]
    const HAND_YEARS: [(Sign, [u8; 12]); 12] = [
        //             Koç Boğ İki Yen Asl Baş Ter Akr Yay Oğl Kov Bal
        (Aries,       [12,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11]),
        (Taurus,      [11, 12,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10]),
        (Gemini,      [10, 11, 12,  1,  2,  3,  4,  5,  6,  7,  8,  9]),
        (Cancer,      [ 3,  2,  1, 12, 11, 10,  9,  8,  7,  6,  5,  4]),
        (Leo,         [ 4,  3,  2,  1, 12, 11, 10,  9,  8,  7,  6,  5]),
        (Virgo,       [ 5,  4,  3,  2,  1, 12, 11, 10,  9,  8,  7,  6]),
        (Libra,       [ 6,  7,  8,  9, 10, 11, 12,  1,  2,  3,  4,  5]),
        (Scorpio,     [ 5,  6,  7,  8,  9, 10, 11, 12,  1,  2,  3,  4]),
        (Sagittarius, [ 4,  5,  6,  7,  8,  9, 10, 11, 12,  1,  2,  3]),
        (Capricorn,   [ 9,  8,  7,  6,  5,  4,  3,  2,  1, 12, 11, 10]),
        (Aquarius,    [10,  9,  8,  7,  6,  5,  4,  3,  2,  1, 12, 11]),
        (Pisces,      [11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1, 12]),
    ];

    /// Tablodan yıl: satırı ve sütunu arayarak (formül kullanmadan).
    fn hand_years(sign: Sign, lord_sign: Sign) -> u8 {
        let row = HAND_YEARS
            .iter()
            .find(|(s, _)| *s == sign)
            .expect("twelve rows");
        row.1[Sign::ALL
            .iter()
            .position(|s| *s == lord_sign)
            .expect("twelve columns")]
    }

    /// Lagna Koç 5°; Güneş Aslan, Ay İkizler, Mars Boğa, Merkür Başak, Jüpiter Balık, Venüs Koç,
    /// Satürn Oğlak, Rahu Yengeç, Ketu Oğlak. Yıllar (el hesabı, `forward-…` el örneğiyle aynı
    /// harita): 1, 11, 3, 1, 12, 12, 6, 2, 3, 12, 1, 12; toplam 76.
    fn aries_chart() -> Positions {
        Positions::synthetic(
            Some(5.0),
            [125.0, 70.0, 40.0, 160.0, 340.0, 15.0, 280.0, 100.0, 280.0],
        )
    }

    fn first_level(dasha: &CharaDasha) -> Vec<&SignDashaPeriod> {
        dasha.periods.iter().filter(|p| p.level == 1).collect()
    }

    fn last_signs(periods: &[SignDashaPeriod]) -> Vec<Sign> {
        periods.iter().map(|p| *p.signs.last().unwrap()).collect()
    }

    #[test]
    fn direction_follows_the_ninth_sign_from_the_lagna() {
        // 9. burç düz sayan gruptaysa ileri: Koç → Yay, Aslan → Koç, Başak → Boğa,
        // Terazi → İkizler, Kova → Terazi, Balık → Akrep.
        let forward: Vec<Sign> = Sign::ALL
            .into_iter()
            .filter(|&s| chara_dasha_forward(s))
            .collect();
        assert_eq!(forward, [Aries, Leo, Virgo, Libra, Aquarius, Pisces]);
        assert_eq!(
            chara_dasha_order(Aries),
            [
                Aries,
                Taurus,
                Gemini,
                Cancer,
                Leo,
                Virgo,
                Libra,
                Scorpio,
                Sagittarius,
                Capricorn,
                Aquarius,
                Pisces
            ]
        );
        // Boğa'dan 9. burç Oğlak (ters sayan): geri.
        assert_eq!(
            chara_dasha_order(Taurus),
            [
                Taurus,
                Aries,
                Pisces,
                Aquarius,
                Capricorn,
                Sagittarius,
                Scorpio,
                Libra,
                Virgo,
                Leo,
                Cancer,
                Gemini
            ]
        );
    }

    #[test]
    fn years_match_the_hand_table_in_every_cell() {
        for sign in Sign::ALL {
            for lord_sign in Sign::ALL {
                assert_eq!(
                    chara_dasha_years(sign, lord_sign),
                    hand_years(sign, lord_sign),
                    "{sign:?} with its lord in {lord_sign:?}"
                );
            }
        }
        // Tablonun kendi tutarlılığı: her satır 1..=12'nin permütasyonudur ve köşegen 12'dir.
        for (i, (sign, row)) in HAND_YEARS.iter().enumerate() {
            assert_eq!(*sign, Sign::ALL[i]);
            let mut sorted = *row;
            sorted.sort_unstable();
            assert_eq!(sorted, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], "{sign:?}");
            assert_eq!(row[i], 12, "{sign:?}");
        }
    }

    #[test]
    fn years_use_the_jaimini_lord_of_scorpio_and_aquarius() {
        let p = aries_chart();
        // Akrep: Mars Boğa'da yalnız, Ketu Oğlak'ta Satürn'le: Ketu. Akrep'ten Oğlak'a 3: 2 yıl.
        assert_eq!(chara_dasha_sign_years(Scorpio, &p, true), 2);
        // Eş lordlar devre dışı: Mars Boğa'da, Akrep'ten 7.: 6 yıl.
        assert_eq!(chara_dasha_sign_years(Scorpio, &p, false), 6);
        // Kova: Satürn Oğlak'ta Ketu'yla (1), Rahu Yengeç'te yalnız (0): Satürn; geri 2: 1 yıl.
        assert_eq!(chara_dasha_sign_years(Aquarius, &p, true), 1);
        assert_eq!(chara_dasha_sign_years(Aquarius, &p, false), 1);
        // İki eş lord da Akrep'te: 12 yıl. Yalnızca Mars Akrep'teyse lord dışarıdaki Ketu'dur.
        let both = Positions::synthetic(
            Some(5.0),
            [125.0, 70.0, 215.0, 160.0, 340.0, 15.0, 280.0, 45.0, 225.0],
        );
        assert_eq!(chara_dasha_sign_years(Scorpio, &both, true), 12);
        let mars_only = Positions::synthetic(
            Some(5.0),
            [125.0, 70.0, 215.0, 160.0, 340.0, 15.0, 280.0, 75.0, 255.0],
        );
        // Ketu Yay'da: Akrep'ten 2.: 1 yıl. Ayar devre dışıyken Mars kendi burcunda: 12 yıl.
        assert_eq!(chara_dasha_sign_years(Scorpio, &mars_only, true), 1);
        assert_eq!(chara_dasha_sign_years(Scorpio, &mars_only, false), 12);
    }

    #[test]
    fn first_cycle_starts_at_birth_with_the_lagna_sign_and_runs_without_gaps() {
        let d = chara_dasha(&aries_chart(), true, JD, YEAR).expect("lagna");
        assert_eq!(d.method, CHARA_DASHA_METHOD);
        assert!(d.forward);
        assert_eq!(d.years, [1, 11, 3, 1, 12, 12, 6, 2, 3, 12, 1, 12]);
        let dashas = first_level(&d);
        let first: Vec<&&SignDashaPeriod> = dashas.iter().filter(|p| p.cycle == 1).collect();
        assert_eq!(first.len(), 12);
        assert_eq!(first[0].signs, [Aries]);
        assert_eq!(first[0].start_jd_ut, JD);
        // Koç 1 yıl, Boğa 11 yıl: Boğa doğumdan 1 yıl sonra başlar, 12 yıl sonra biter.
        assert_eq!(first[0].end_jd_ut, JD + YEAR);
        assert_eq!(first[1].signs, [Taurus]);
        assert_eq!(first[1].end_jd_ut, JD + 12.0 * YEAR);
        // İlk çevrim 76 yıl sürer.
        assert_eq!(first[11].signs, [Pisces]);
        assert_eq!(first[11].end_jd_ut, JD + 76.0 * YEAR);
        for w in dashas.windows(2) {
            assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
        }
        assert!(d
            .periods
            .iter()
            .all(|p| p.has_children && p.level <= 2 && usize::from(p.level) == p.signs.len()));
    }

    #[test]
    fn second_cycle_skips_zero_year_signs_and_stops_at_the_horizon() {
        let d = chara_dasha(&aries_chart(), true, JD, YEAR).expect("lagna");
        let second: Vec<(Sign, f64)> = first_level(&d)
            .into_iter()
            .filter(|p| p.cycle == 2)
            .map(|p| (p.signs[0], (p.end_jd_ut - p.start_jd_ut) / YEAR))
            .collect();
        // 12 − yıl: Koç 11 (76→87), Boğa 1 (→88), İkizler 9 (→97), Yengeç 11 (→108); Aslan ve
        // Başak 0 yıl, atlanır; Terazi 6 (→114); Akrep 10 (114→124): 114 < 120, üretilir.
        // Yay 124. yılda başlardı: üretilmez.
        assert_eq!(
            second,
            [
                (Aries, 11.0),
                (Taurus, 1.0),
                (Gemini, 9.0),
                (Cancer, 11.0),
                (Libra, 6.0),
                (Scorpio, 10.0)
            ]
        );
        let last = d.periods.iter().rfind(|p| p.level == 1).expect("dashas");
        assert_eq!(last.end_jd_ut, JD + 124.0 * YEAR);
    }

    #[test]
    fn a_dasha_starting_exactly_at_the_horizon_is_not_produced() {
        // `backward-…` el örneğinin haritası: yıllar 5, 7, 3, 8, 9, 12, 1, 12, 4, 11, 12, 11
        // (toplam 95); ikinci çevrim Boğa 7 (→102), Koç 5 (→107), Balık 9 (→116), Kova 4 (→120).
        // Oğlak tam 120. yılda başlardı: üretilmez.
        let p = Positions::synthetic(
            Some(40.0),
            [155.0, 100.0, 220.0, 50.0, 250.0, 190.0, 5.0, 70.0, 250.0],
        );
        let d = chara_dasha(&p, true, JD, YEAR).expect("lagna");
        assert!(!d.forward);
        assert_eq!(d.years, [5, 7, 3, 8, 9, 12, 1, 12, 4, 11, 12, 11]);
        let second: Vec<Sign> = first_level(&d)
            .into_iter()
            .filter(|p| p.cycle == 2)
            .map(|p| p.signs[0])
            .collect();
        assert_eq!(second, [Taurus, Aries, Pisces, Aquarius]);
        let last = d.periods.iter().rfind(|p| p.level == 1).expect("dashas");
        assert_eq!(last.end_jd_ut, JD + 120.0 * YEAR);
    }

    #[test]
    fn sub_periods_follow_the_chart_direction_and_end_with_the_dasha_sign() {
        assert_eq!(
            sub_period_signs(Aries, true),
            [
                Taurus,
                Gemini,
                Cancer,
                Leo,
                Virgo,
                Libra,
                Scorpio,
                Sagittarius,
                Capricorn,
                Aquarius,
                Pisces,
                Aries
            ]
        );
        assert_eq!(
            sub_period_signs(Taurus, false),
            [
                Aries,
                Pisces,
                Aquarius,
                Capricorn,
                Sagittarius,
                Scorpio,
                Libra,
                Virgo,
                Leo,
                Cancer,
                Gemini,
                Taurus
            ]
        );
        let d = chara_dasha(&aries_chart(), true, JD, YEAR).expect("lagna");
        // Ön-sıra: dasha, ardından on iki alt dönemi.
        assert_eq!(d.periods[0].signs, [Aries]);
        assert_eq!(d.periods[1].signs, [Aries, Taurus]);
        assert_eq!(d.periods[12].signs, [Aries, Aries]);
        assert_eq!(d.periods[13].signs, [Taurus]);
        // Boğa dashası 11 yıl: her alt dönem 11/12 yıl; sonuncusu ebeveynin bitişine tam eşit.
        let taurus = &d.periods[13];
        let subs = &d.periods[14..26];
        assert_eq!(last_signs(subs), sub_period_signs(Taurus, true));
        assert_eq!(subs[0].start_jd_ut, taurus.start_jd_ut);
        assert_eq!(subs[11].end_jd_ut, taurus.end_jd_ut);
        for (k, s) in subs.iter().enumerate() {
            assert_eq!((s.level, s.cycle), (2, 1));
            let want = taurus.start_jd_ut + 11.0 * YEAR * (k as f64 + 1.0) / 12.0;
            assert!((s.end_jd_ut - want).abs() < 1e-6, "{k}");
        }
        for w in subs.windows(2) {
            assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
        }
    }

    #[test]
    fn children_are_served_per_cycle_down_to_the_third_level() {
        let d = chara_dasha(&aries_chart(), true, JD, YEAR).expect("lagna");
        let roots = sign_dasha_children(&d, &[], 1).unwrap();
        assert_eq!(last_signs(&roots), chara_dasha_order(Aries));
        let second = sign_dasha_children(&d, &[], 2).unwrap();
        assert_eq!(
            last_signs(&second),
            [Aries, Taurus, Gemini, Cancer, Libra, Scorpio]
        );
        // 2. seviye haritadaki dönemlerdir.
        let subs = sign_dasha_children(&d, &[Taurus], 1).unwrap();
        assert_eq!(subs[..], d.periods[14..26]);
        // 3. seviye istenince bölünür: aynı yön, son dönem ebeveyn burcu.
        let third = sign_dasha_children(&d, &[Taurus, Leo], 1).unwrap();
        assert_eq!(last_signs(&third), sub_period_signs(Leo, true));
        let parent = &d.periods[14 + 2];
        assert_eq!(parent.signs, [Taurus, Leo]);
        assert_eq!(third[0].start_jd_ut, parent.start_jd_ut);
        assert_eq!(third[11].end_jd_ut, parent.end_jd_ut);
        assert!(third
            .iter()
            .all(|p| p.level == 3 && !p.has_children && p.cycle == 1 && p.signs.len() == 3));
        // İkinci çevrimin dönemi kendi çevrimiyle istenir: Koç'un iki çevrimi ayrı dönemlerdir.
        let again = sign_dasha_children(&d, &[Aries], 2).unwrap();
        assert_eq!(again[0].cycle, 2);
        assert_eq!(again[0].start_jd_ut, JD + 76.0 * YEAR);
    }

    #[test]
    fn children_reject_bad_cycles_and_paths() {
        let d = chara_dasha(&aries_chart(), true, JD, YEAR).expect("lagna");
        for cycle in [0, 3] {
            assert_eq!(
                sign_dasha_children(&d, &[], cycle),
                Err(CoreError::InvalidDashaInput("cycle"))
            );
        }
        // Üç burçluk yolun alt dönemi yoktur (en çok üç seviye).
        assert_eq!(
            sign_dasha_children(&d, &[Aries, Taurus, Gemini], 1),
            Err(CoreError::InvalidDashaInput("path"))
        );
        // Aslan ikinci çevrimde atlanır, Yay üretilmez.
        for sign in [Leo, Sagittarius] {
            assert_eq!(
                sign_dasha_children(&d, &[sign], 2),
                Err(CoreError::InvalidDashaInput("path"))
            );
        }
    }

    #[test]
    fn path_at_follows_the_containing_periods() {
        let d = chara_dasha(&aries_chart(), true, JD, YEAR).expect("lagna");
        // Doğum anı: Koç dashası, ilk alt dönem Boğa, onun ilk alt dönemi İkizler.
        assert_eq!(
            sign_dasha_path_at(&d, JD).unwrap(),
            Some(SignDashaPath {
                cycle: 1,
                signs: vec![Aries, Taurus, Gemini]
            })
        );
        // 80 yıl sonra: ikinci çevrimin Koç dashası (76→87).
        let at = sign_dasha_path_at(&d, JD + 80.0 * YEAR)
            .unwrap()
            .expect("inside");
        assert_eq!((at.cycle, at.signs[0], at.signs.len()), (2, Aries, 3));
        for depth in 1..=3usize {
            let period = sign_dasha_children(&d, &at.signs[..depth - 1], at.cycle)
                .unwrap()
                .into_iter()
                .find(|p| p.signs == at.signs[..depth])
                .expect("on the path");
            let jd = JD + 80.0 * YEAR;
            assert!(
                period.start_jd_ut <= jd && jd < period.end_jd_ut,
                "level {depth}"
            );
        }
        // Dönemlerin dışı: doğumdan önce ve son dashanın bitişinde (124. yıl) yol yoktur.
        assert_eq!(sign_dasha_path_at(&d, JD - 1.0).unwrap(), None);
        assert_eq!(sign_dasha_path_at(&d, JD + 124.0 * YEAR).unwrap(), None);
        assert_eq!(
            sign_dasha_path_at(&d, f64::NAN),
            Err(CoreError::InvalidDashaInput("jd_ut"))
        );
    }

    #[test]
    fn no_lagna_means_no_dasha() {
        let p = aries_chart().without_lagna();
        assert_eq!(chara_dasha(&p, true, JD, YEAR), None);
    }

    fn input(settings: CalcSettings, time_known: bool) -> ChartInput {
        ChartInput {
            jd_ut: JD,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known,
        }
    }

    #[test]
    fn collectors_follow_the_chart_the_settings_and_the_birth_time() {
        let mut eph = FakeEph::default();
        let settings = CalcSettings {
            dasha_year: 360.0,
            advanced: AdvancedSettings {
                karaka_scheme: KarakaScheme::Eight,
                jaimini_co_lords: false,
            },
            ..CalcSettings::default()
        };
        let chart = compute_chart(&mut eph, JD, 41.0, 29.0, &settings).unwrap();
        let want = chara_dasha(&Positions::from(&chart), false, JD, 360.0).expect("lagna");
        let known = input(settings.clone(), true);
        let got = compute_chara_dasha(&mut eph, &known)
            .unwrap()
            .expect("time known");
        assert_eq!(got, want);
        assert_eq!(got.periods[0].signs, [chart.lagna.sign]);
        assert_eq!(
            got.periods[0].end_jd_ut,
            JD + f64::from(got.years[0]) * 360.0
        );
        let first = got.periods[0].signs[0];
        assert_eq!(
            compute_chara_dasha_children(&mut eph, &known, &[first], 1).unwrap(),
            sign_dasha_children(&want, &[first], 1).unwrap()
        );
        assert_eq!(
            compute_chara_dasha_at(&mut eph, &known, JD + 400.0).unwrap(),
            sign_dasha_path_at(&want, JD + 400.0).unwrap()
        );

        // Doğum saati bilinmiyor: dasha yok, alt dönem listesi boş, yol yok (K2).
        let unknown = input(settings, false);
        assert_eq!(compute_chara_dasha(&mut eph, &unknown).unwrap(), None);
        assert_eq!(
            compute_chara_dasha_children(&mut eph, &unknown, &[first], 1).unwrap(),
            []
        );
        assert_eq!(
            compute_chara_dasha_at(&mut eph, &unknown, JD).unwrap(),
            None
        );
    }

    #[test]
    fn collectors_report_the_chart_errors_even_without_a_birth_time() {
        let mut eph = FakeEph::default();
        for time_known in [true, false] {
            let mut bad = input(CalcSettings::default(), time_known);
            bad.lat = 91.0;
            let err = Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0,
            });
            assert_eq!(compute_chara_dasha(&mut eph, &bad), err);
            assert_eq!(
                compute_chara_dasha_children(&mut eph, &bad, &[], 1),
                Err(CoreError::InvalidLocation {
                    lat: 91.0,
                    lon: 29.0
                })
            );
            assert_eq!(
                compute_chara_dasha_at(&mut eph, &bad, JD),
                Err(CoreError::InvalidLocation {
                    lat: 91.0,
                    lon: 29.0
                })
            );
        }
        let ok = input(CalcSettings::default(), true);
        assert_eq!(
            compute_chara_dasha_children(&mut eph, &ok, &[], 3),
            Err(CoreError::InvalidDashaInput("cycle"))
        );
    }
}
