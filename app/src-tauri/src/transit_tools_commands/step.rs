//! Geçiş anının adımı (yol haritası §4.10): bir andan bir birim ileri ya da geri. Adım aritmetiği
//! burada çözülür; arayüz tarih hesabı yapmaz (C4).
//!
//! Saat adımı mutlaktır (tam 3 600 saniye). Gün, hafta, ay ve yıl adımı duvar saatiyledir: an,
//! verilen IANA dilimindeki takvim tarihine ve saatine çevrilir, takvimde yürünür ve aynı duvar
//! saati yeniden çözülür. Yaz saati geçişinden geçen bir gün adımı bu yüzden 23 ya da 25 saat
//! sürer. Varılan ayda o gün yoksa ayın son günü alınır (31 Ocak + 1 ay = 28 ya da 29 Şubat).
//! Varılan duvar saati yaz saati boşluğuna düşerse boşluk kadar ileri alınır (v1'in önerdiği
//! saat); örtüşmeye düşerse önceki an alınır (transit anının kuralı, Plan 11 D7).
//!
//! Komut saf hesaptır: motora gitmez. Hata yalnızca alan adını taşır; an ve dilim hataya girmez.

use atlas::{Timestamp, TzSpec};
use jyotish_core::JD_UT_RANGE;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::{invalid, TOOLS_YEAR_RANGE};
use crate::commands::CommandError;
use crate::transit_commands::{transit_moment, TransitAt};

const UNIX_EPOCH_JD: f64 = 2_440_587.5;
const DAY_SECS: i64 = 86_400;
/// Bir çağrının en büyük adım sayısı (ileri ya da geri).
pub const MAX_STEP_COUNT: i32 = 12;

/// Adım birimi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub enum StepUnit {
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl StepUnit {
    pub const ALL: [StepUnit; 5] = [
        StepUnit::Hour,
        StepUnit::Day,
        StepUnit::Week,
        StepUnit::Month,
        StepUnit::Year,
    ];
}

/// Adımın girdisi. `jd_ut` gösterilen transit anıdır (`TransitView.snapshot.jd_ut`); `count`
/// adım sayısıdır (eksi: geri; sıfır olamaz); `tz` duvar saatinin okunduğu IANA dilimidir
/// (tarayıcının dilimi; yol ya da adres değildir).
#[derive(Debug, Clone, PartialEq, Deserialize, Type)]
pub struct TransitStepRequest {
    pub jd_ut: f64,
    pub unit: StepUnit,
    pub count: i32,
    pub tz: String,
}

/// Varılan an; arayüz onu `TransitAt::Jd` olarak geri verir.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct TransitStepView {
    pub jd_ut: f64,
    /// Anın UTC yazımı (RFC 3339).
    pub utc: String,
}

/// Takvim tarihine `months` ay ekler; varılan ayda o gün yoksa ayın son günü alınır.
fn add_months(date: atlas::Date, months: i32) -> Result<atlas::Date, CommandError> {
    let total = i32::from(date.year()) * 12 + i32::from(date.month()) - 1 + months;
    let (year, month) = (total.div_euclid(12), total.rem_euclid(12) + 1);
    if !TOOLS_YEAR_RANGE.contains(&year) {
        return Err(invalid("range"));
    }
    let first = atlas::Date::new(
        i16::try_from(year).map_err(|_| invalid("range"))?,
        i8::try_from(month).map_err(|_| invalid("range"))?,
        1,
    )
    .map_err(|_| invalid("range"))?;
    let day = date.day().min(first.days_in_month());
    atlas::Date::new(first.year(), first.month(), day).map_err(|_| invalid("range"))
}

/// Duvar saatini çözer; boşluğa düşen saat, v1'in önerdiği saate (boşluk kadar ileri) alınır.
fn resolve(date: &str, time: &str, tz: &str) -> Result<(f64, String), CommandError> {
    let local = |date: &str, time: &str| TransitAt::Local {
        date: date.to_owned(),
        time: time.to_owned(),
        tz: tz.to_owned(),
    };
    // `now` yalnızca `TransitAt::Now` için okunur; burada kullanılmaz.
    let now = Timestamp::UNIX_EPOCH;
    match transit_moment(&local(date, time), now) {
        Ok(moment) => Ok(moment),
        Err(e) if e.code == "nonexistent_local_time" => {
            let suggestion = e.params.get("suggestion").ok_or_else(|| invalid("range"))?;
            let (date, time) = suggestion.split_once('T').ok_or_else(|| invalid("range"))?;
            transit_moment(&local(date, time), now).map_err(|_| invalid("range"))
        }
        Err(e) if e.code == "unknown_time_zone" => Err(invalid("tz")),
        Err(_) => Err(invalid("range")),
    }
}

/// `transit_step` gövdesi.
pub fn transit_step_body(request: &TransitStepRequest) -> Result<TransitStepView, CommandError> {
    let TransitStepRequest {
        jd_ut,
        unit,
        count,
        tz,
    } = request;
    // NaN ve sonsuz da aralık dışındadır.
    if !JD_UT_RANGE.contains(jd_ut) {
        return Err(invalid("jd_ut"));
    }
    // `abs` kullanılmaz: `i32::MIN` taşar.
    if *count == 0 || !(-MAX_STEP_COUNT..=MAX_STEP_COUNT).contains(count) {
        return Err(invalid("count"));
    }
    let tz = tz.trim();
    if tz.is_empty() || !atlas::is_known_time_zone(tz) {
        return Err(invalid("tz"));
    }
    let secs = ((jd_ut - UNIX_EPOCH_JD) * 86_400.0).round() as i64;
    let (jd_ut, utc) = if *unit == StepUnit::Hour {
        let at = Timestamp::from_second(secs + i64::from(*count) * 3_600)
            .map_err(|_| invalid("range"))?;
        (
            UNIX_EPOCH_JD + at.as_second() as f64 / 86_400.0,
            at.to_string(),
        )
    } else {
        let at = Timestamp::from_second(secs).map_err(|_| invalid("range"))?;
        // Yer boylamı 0: transit anının duvar saati de böyle çözülür (`transit_moment`).
        let offset =
            atlas::offset_at(&TzSpec::Iana(tz.to_owned()), 0.0, at).map_err(|_| invalid("tz"))?;
        // Duvar saati, UTC'ymiş gibi yazılan bir an olarak: takvim ve saat buradan okunur.
        let wall = secs + i64::from(offset);
        let days = match unit {
            StepUnit::Day => i64::from(*count),
            StepUnit::Week => 7 * i64::from(*count),
            _ => 0,
        };
        let stepped =
            Timestamp::from_second(wall + days * DAY_SECS).map_err(|_| invalid("range"))?;
        let time = stepped.strftime("%H:%M:%S").to_string();
        let mut date: atlas::Date = stepped
            .strftime("%Y-%m-%d")
            .to_string()
            .parse()
            .map_err(|_| invalid("range"))?;
        date = match unit {
            StepUnit::Month => add_months(date, *count)?,
            StepUnit::Year => add_months(date, 12 * *count)?,
            _ => date,
        };
        if !TOOLS_YEAR_RANGE.contains(&i32::from(date.year())) {
            return Err(invalid("range"));
        }
        resolve(&date.to_string(), &time, tz)?
    };
    // Her birimin sonucu aynı yıl aralığındadır (D23); aralık dışına çıkan adım hata verir.
    let year: i32 = utc
        .parse::<Timestamp>()
        .map_err(|_| invalid("range"))?
        .strftime("%Y")
        .to_string()
        .parse()
        .map_err(|_| invalid("range"))?;
    if !TOOLS_YEAR_RANGE.contains(&year) || !JD_UT_RANGE.contains(&jd_ut) {
        return Err(invalid("range"));
    }
    Ok(TransitStepView { jd_ut, utc })
}

/// Geçiş anının adımı: `request.jd_ut` anından `count` birim sonraki (eksi: önceki) an. Saat adımı
/// mutlaktır; gün, hafta, ay ve yıl adımı `request.tz` dilimindeki duvar saatini korur. Saf
/// hesaptır, motora gitmez. An ve dilim loglanmaz.
#[tauri::command]
#[specta::specta]
pub fn transit_step(request: TransitStepRequest) -> Result<TransitStepView, CommandError> {
    transit_step_body(&request)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 3339 UTC yazımından Jülyen günü.
    fn jd(utc: &str) -> f64 {
        let t: Timestamp = utc.parse().expect("timestamp");
        UNIX_EPOCH_JD + t.as_second() as f64 / 86_400.0
    }

    fn step(utc: &str, unit: StepUnit, count: i32, tz: &str) -> Result<String, CommandError> {
        let request = TransitStepRequest {
            jd_ut: jd(utc),
            unit,
            count,
            tz: tz.to_owned(),
        };
        let view = transit_step_body(&request)?;
        // İki alan aynı andır.
        assert!((view.jd_ut - jd(&view.utc)).abs() < 1e-9, "{}", view.utc);
        Ok(view.utc)
    }

    fn ok(utc: &str, unit: StepUnit, count: i32, tz: &str) -> String {
        step(utc, unit, count, tz).expect("step")
    }

    fn field(utc: &str, unit: StepUnit, count: i32, tz: &str) -> String {
        let e = step(utc, unit, count, tz).unwrap_err();
        assert_eq!(e.code, "invalid_transit_input");
        // Hata yalnızca alan adını taşır.
        assert_eq!(e.params.len(), 1);
        e.params["field"].clone()
    }

    const BERLIN: &str = "Europe/Berlin";
    const ISTANBUL: &str = "Europe/Istanbul";

    #[test]
    fn an_hour_step_is_exactly_3600_seconds_in_any_zone() {
        assert_eq!(
            ok("2026-11-25T10:00:00Z", StepUnit::Hour, 1, ISTANBUL),
            "2026-11-25T11:00:00Z"
        );
        assert_eq!(
            ok("2026-11-25T10:00:00Z", StepUnit::Hour, -3, BERLIN),
            "2026-11-25T07:00:00Z"
        );
        // Yaz saati geçişinden geçerken de mutlaktır: Berlin 2026-03-29 01:30 CET (00:30 UT)
        // + 1 saat = 01:30 UT (duvar saati 03:30 CEST).
        assert_eq!(
            ok("2026-03-29T00:30:00Z", StepUnit::Hour, 1, BERLIN),
            "2026-03-29T01:30:00Z"
        );
    }

    #[test]
    fn day_and_week_steps_keep_the_wall_clock() {
        // İstanbul 2016'dan beri hep +03:00: bir gün tam 24 saattir. 13:45:30 → 13:45:30.
        assert_eq!(
            ok("2026-11-25T10:45:30Z", StepUnit::Day, 1, ISTANBUL),
            "2026-11-26T10:45:30Z"
        );
        assert_eq!(
            ok("2026-11-25T10:45:30Z", StepUnit::Day, -2, ISTANBUL),
            "2026-11-23T10:45:30Z"
        );
        // Berlin'de yaz saati 29 Mart 2026 Pazar günü başlar (Mart'ın son Pazar'ı).
        // 28 Mart 12:00 CET = 11:00 UT; bir gün sonrası 29 Mart 12:00 CEST = 10:00 UT: 23 saat.
        assert_eq!(
            ok("2026-03-28T11:00:00Z", StepUnit::Day, 1, BERLIN),
            "2026-03-29T10:00:00Z"
        );
        // Geri: 29 Mart 12:00 CEST → 28 Mart 12:00 CET.
        assert_eq!(
            ok("2026-03-29T10:00:00Z", StepUnit::Day, -1, BERLIN),
            "2026-03-28T11:00:00Z"
        );
        // Yaz saati 25 Ekim 2026 Pazar günü biter. 22 Ekim 12:00 CEST = 10:00 UT; bir hafta
        // sonrası 29 Ekim 12:00 CET = 11:00 UT: yedi gün ve bir saat.
        assert_eq!(
            ok("2026-10-22T10:00:00Z", StepUnit::Week, 1, BERLIN),
            "2026-10-29T11:00:00Z"
        );
        assert_eq!(
            ok("2026-11-25T10:00:00Z", StepUnit::Week, -2, "UTC"),
            "2026-11-11T10:00:00Z"
        );
    }

    #[test]
    fn month_and_year_steps_clamp_to_the_last_day_of_the_month() {
        // UTC diliminde duvar saati UT'dir: türetme yalnızca takvimdir.
        // 31 Ocak + 1 ay: Şubat 2026 28 gündür (2026 artık yıl değildir).
        assert_eq!(
            ok("2026-01-31T12:00:00Z", StepUnit::Month, 1, "UTC"),
            "2026-02-28T12:00:00Z"
        );
        // 2024 artık yıldır: 29 Şubat.
        assert_eq!(
            ok("2024-01-31T12:00:00Z", StepUnit::Month, 1, "UTC"),
            "2024-02-29T12:00:00Z"
        );
        // 31 Mart − 1 ay = 28 Şubat; 31 Mayıs + 1 ay = 30 Haziran.
        assert_eq!(
            ok("2026-03-31T12:00:00Z", StepUnit::Month, -1, "UTC"),
            "2026-02-28T12:00:00Z"
        );
        assert_eq!(
            ok("2026-05-31T12:00:00Z", StepUnit::Month, 1, "UTC"),
            "2026-06-30T12:00:00Z"
        );
        // Yıl sınırı: Aralık + 1 ay = Ocak; Ocak − 1 ay = Aralık.
        assert_eq!(
            ok("2026-12-15T12:00:00Z", StepUnit::Month, 1, "UTC"),
            "2027-01-15T12:00:00Z"
        );
        assert_eq!(
            ok("2026-01-15T12:00:00Z", StepUnit::Month, -1, "UTC"),
            "2025-12-15T12:00:00Z"
        );
        // On iki ay bir yıldır.
        assert_eq!(
            ok("2026-11-25T10:00:00Z", StepUnit::Month, 12, "UTC"),
            ok("2026-11-25T10:00:00Z", StepUnit::Year, 1, "UTC")
        );
        // 29 Şubat ± 1 yıl: 28 Şubat.
        assert_eq!(
            ok("2024-02-29T12:00:00Z", StepUnit::Year, 1, "UTC"),
            "2025-02-28T12:00:00Z"
        );
        assert_eq!(
            ok("2024-02-29T12:00:00Z", StepUnit::Year, -1, "UTC"),
            "2023-02-28T12:00:00Z"
        );
        // Ay adımı geri alınamayabilir: 31 Ocak → 28 Şubat → 28 Ocak.
        assert_eq!(
            ok("2026-02-28T12:00:00Z", StepUnit::Month, -1, "UTC"),
            "2026-01-28T12:00:00Z"
        );
        // Duvar saati dilimde korunur: Berlin 15 Ocak 12:00 CET (11:00 UT) + 6 ay = 15 Temmuz
        // 12:00 CEST (10:00 UT).
        assert_eq!(
            ok("2026-01-15T11:00:00Z", StepUnit::Month, 6, BERLIN),
            "2026-07-15T10:00:00Z"
        );
    }

    #[test]
    fn a_wall_clock_in_a_gap_moves_forward_and_one_in_an_overlap_takes_the_earlier_moment() {
        // Boşluk: Berlin'de 29 Mart 2026 02:00–03:00 yoktur. 28 Mart 02:30 CET (01:30 UT) + 1 gün
        // = 29 Mart 02:30 (yok) → boşluk kadar ileri, 03:30 CEST = 01:30 UT.
        assert_eq!(
            ok("2026-03-28T01:30:00Z", StepUnit::Day, 1, BERLIN),
            "2026-03-29T01:30:00Z"
        );
        // Ay adımıyla boşluğa düşmek: 28 Şubat 02:30 CET (01:30 UT) + 1 ay = 28 Mart 02:30 CET
        // (boşluk 29'undadır: 28 Mart olağandır) → 01:30 UT.
        assert_eq!(
            ok("2026-02-28T01:30:00Z", StepUnit::Month, 1, BERLIN),
            "2026-03-28T01:30:00Z"
        );
        // 1 Mart 02:30 CET + 28 gün (dört hafta) = 29 Mart 02:30 (yok) → 03:30 CEST = 01:30 UT.
        assert_eq!(
            ok("2026-03-01T01:30:00Z", StepUnit::Week, 4, BERLIN),
            "2026-03-29T01:30:00Z"
        );
        // Örtüşme: Berlin'de 25 Ekim 2026 02:00–03:00 iki kez yaşanır. 24 Ekim 02:30 CEST
        // (00:30 UT) + 1 gün = 25 Ekim 02:30 → önceki an (CEST), 00:30 UT.
        assert_eq!(
            ok("2026-10-24T00:30:00Z", StepUnit::Day, 1, BERLIN),
            "2026-10-25T00:30:00Z"
        );
    }

    #[test]
    fn bad_input_names_the_field_and_carries_nothing_else() {
        let at = "2026-11-25T10:00:00Z";
        assert_eq!(field(at, StepUnit::Day, 0, "UTC"), "count");
        assert_eq!(field(at, StepUnit::Day, MAX_STEP_COUNT + 1, "UTC"), "count");
        assert_eq!(
            field(at, StepUnit::Hour, -MAX_STEP_COUNT - 1, "UTC"),
            "count"
        );
        assert_eq!(field(at, StepUnit::Day, i32::MIN, "UTC"), "count");
        assert_eq!(field(at, StepUnit::Day, 1, "Mars/Olympus"), "tz");
        assert_eq!(field(at, StepUnit::Day, 1, "  "), "tz");
        assert_eq!(field(at, StepUnit::Hour, 1, ""), "tz");
        for bad in [f64::NAN, f64::INFINITY, 0.0, 1e300] {
            let request = TransitStepRequest {
                jd_ut: bad,
                unit: StepUnit::Day,
                count: 1,
                tz: "UTC".into(),
            };
            let e = transit_step_body(&request).unwrap_err();
            assert_eq!(
                (e.code.as_str(), e.params["field"].as_str()),
                ("invalid_transit_input", "jd_ut")
            );
        }
        // Yıl aralığının uçları (1–2999).
        assert_eq!(
            field("2999-06-15T12:00:00Z", StepUnit::Year, 1, "UTC"),
            "range"
        );
        assert_eq!(
            field("2999-12-15T12:00:00Z", StepUnit::Month, 1, "UTC"),
            "range"
        );
        assert_eq!(
            field("2999-12-31T12:00:00Z", StepUnit::Day, 1, "UTC"),
            "range"
        );
        assert_eq!(
            field("0001-01-01T12:00:00Z", StepUnit::Week, -1, "UTC"),
            "range"
        );
        // Saat adımı da aynı yıl aralığındadır; son ve ilk saatin kendisi geçerlidir.
        assert_eq!(
            field("2999-12-31T23:30:00Z", StepUnit::Hour, 1, "UTC"),
            "range"
        );
        assert_eq!(
            field("0001-01-01T00:30:00Z", StepUnit::Hour, -1, "UTC"),
            "range"
        );
        assert_eq!(
            ok("2999-12-31T22:30:00Z", StepUnit::Hour, 1, "UTC"),
            "2999-12-31T23:30:00Z"
        );
        assert_eq!(
            ok("0001-01-01T01:30:00Z", StepUnit::Hour, -1, "UTC"),
            "0001-01-01T00:30:00Z"
        );
        assert_eq!(
            ok("2998-06-15T12:00:00Z", StepUnit::Year, 1, "UTC"),
            "2999-06-15T12:00:00Z"
        );
        // En büyük adım sayısı geçerlidir.
        assert_eq!(
            ok(at, StepUnit::Day, MAX_STEP_COUNT, "UTC"),
            "2026-12-07T10:00:00Z"
        );
        assert_eq!(StepUnit::ALL.len(), 5);
    }
}
