//! Jagannatha Hora `.jhd` içe aktarma (spec §5: yalnızca içe aktarma; içe alınan haritalar
//! o anki genel varsayılanların kopyasını alır).
//!
//! Biçim: düz metin, satır başına bir değer. Kişinin adı dosya adıdır. Bu sürümün okuduğu
//! satırlar (1'den sayılır):
//!
//! | Satır | Alan | Biçim |
//! |---|---|---|
//! | 1 | ay | tamsayı 1–12 |
//! | 2 | gün | tamsayı |
//! | 3 | yıl | tamsayı (MS) |
//! | 4 | yerel saat | `S.DDSS` (17.300000 = 17:30:00); 4 haneden sonrası saniye kesri |
//! | 5 | saat dilimi | `S.DD`, **batı pozitif** (−5.300000 = UTC+05:30) |
//! | 6 | boylam | `D.DDSS`, **batı pozitif** (−77.123240 = 77°12′32.40″ D) |
//! | 7 | enlem | `D.DDSS`, kuzey pozitif |
//! | 8 | yükseklik | okunmaz |
//! | 9+ | diğer | sayı olmayan ilk iki boş olmayan satır yer adı ve ülke; diğerleri okunmaz |
//!
//! İşaret kuralları gerçek dosyalarla doğrulanacaktır (spec §5); şüpheli dosyada
//! `JhdOffsetLongitudeMismatch` uyarısı verilir.

use std::path::Path;

use atlas::{Date, Time};
use rusqlite::{params, OptionalExtension};

use crate::exchange::{ImportReport, ImportWarning, ImportWarningCode};
use crate::model::{BirthData, ChartKind, NewChart, NewClient, Place, TimeCertainty, TzChoice};
use crate::{Store, StoreError};

/// Ofset × 15° ile boylam arasındaki en büyük kabul edilen fark (derece). Batı Çin
/// (Kaşgar, UTC+8, 76° D) ≈ 44°; tek işaret hatası Hindistan'da ≈ 160° verir.
pub const JHD_OFFSET_LON_TOLERANCE_DEG: f64 = 60.0;

/// Ayrıştırılmış JHD dosyası.
#[derive(Debug, Clone, PartialEq)]
pub struct JhdChart {
    pub name: String,
    pub birth: BirthData,
    pub warnings: Vec<ImportWarningCode>,
}

/// Dosya kümesi sonucu: başarılı olanlar `report`'ta, okunamayanlar dosya adıyla `failed`'da.
#[derive(Debug)]
pub struct JhdImport {
    pub report: ImportReport,
    pub failed: Vec<(String, StoreError)>,
}

/// UTF-8 değilse Latin-1 (Windows'ta yazılmış eski dosyalar); baştaki BOM atılır.
pub fn decode_text(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.trim_start_matches('\u{feff}').to_owned(),
        Err(_) => bytes.iter().map(|&b| char::from(b)).collect(),
    }
}

/// `[-]B.DDSS[kesir]` paketli altmışlık değer → (işaret, bütün, dakika, saniye, nanosaniye).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Packed {
    pub negative: bool,
    pub whole: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub nanos: u32,
}

impl Packed {
    /// Bütün birim cinsinden işaretli değer (derece ya da saat).
    pub fn value(self) -> f64 {
        let v = f64::from(self.whole)
            + f64::from(self.minutes) / 60.0
            + (f64::from(self.seconds) + f64::from(self.nanos) / 1e9) / 3600.0;
        if self.negative {
            -v
        } else {
            v
        }
    }
}

pub fn parse_packed(s: &str, line: usize, field: &'static str) -> Result<Packed, StoreError> {
    let err = || StoreError::Jhd { line, field };
    let s = s.trim();
    let (negative, body) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let (int, frac) = body.split_once('.').unwrap_or((body, ""));
    if int.is_empty()
        || !int.bytes().all(|b| b.is_ascii_digit())
        || !frac.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(err());
    }
    let whole: u32 = int.parse().map_err(|_| err())?;
    let padded = format!("{frac:0<4}");
    let minutes: u32 = padded[0..2].parse().map_err(|_| err())?;
    let seconds: u32 = padded[2..4].parse().map_err(|_| err())?;
    let rest = &padded[4..];
    let nanos: u32 = if rest.is_empty() {
        0
    } else {
        format!("{:0<9}", &rest[..rest.len().min(9)])
            .parse()
            .map_err(|_| err())?
    };
    if minutes >= 60 || seconds >= 60 {
        return Err(err());
    }
    Ok(Packed {
        negative,
        whole,
        minutes,
        seconds,
        nanos,
    })
}

fn int_line<T: std::str::FromStr>(
    lines: &[&str],
    line: usize,
    field: &'static str,
) -> Result<T, StoreError> {
    lines[line - 1]
        .trim()
        .parse()
        .map_err(|_| StoreError::Jhd { line, field })
}

/// `file_name` yalnızca ad için kullanılır (uzantısız kök).
pub fn parse_jhd(file_name: &str, bytes: &[u8]) -> Result<JhdChart, StoreError> {
    let name = Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().trim().to_owned())
        .filter(|s| !s.is_empty())
        .ok_or(StoreError::Jhd {
            line: 0,
            field: "name",
        })?;
    let text = decode_text(bytes);
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < 7 {
        return Err(StoreError::Jhd {
            line: lines.len() + 1,
            field: "missing",
        });
    }
    let month: i8 = int_line(&lines, 1, "month")?;
    let day: i8 = int_line(&lines, 2, "day")?;
    let year: i16 = int_line(&lines, 3, "year")?;
    if !(1..=12).contains(&month) {
        return Err(StoreError::Jhd {
            line: 1,
            field: "month",
        });
    }
    let date = Date::new(year, month, day).map_err(|_| StoreError::Jhd {
        line: 2,
        field: "day",
    })?;

    let t = parse_packed(lines[3], 4, "time")?;
    if t.negative || t.whole >= 24 {
        return Err(StoreError::Jhd {
            line: 4,
            field: "time",
        });
    }
    // Aralıklar yukarıda denetlendi; dönüşümler kayıpsızdır.
    let time = Time::new(
        t.whole as i8,
        t.minutes as i8,
        t.seconds as i8,
        t.nanos as i32,
    )
    .map_err(|_| StoreError::Jhd {
        line: 4,
        field: "time",
    })?;

    let tz = parse_packed(lines[4], 5, "timezone")?;
    let tz_hours_west = tz.value();
    if tz_hours_west.abs() > 18.0 {
        return Err(StoreError::Jhd {
            line: 5,
            field: "timezone",
        });
    }
    let offset_secs = -((tz_hours_west * 3600.0).round() as i32);

    let lon = -parse_packed(lines[5], 6, "longitude")?.value();
    if !(-180.0..=180.0).contains(&lon) {
        return Err(StoreError::Jhd {
            line: 6,
            field: "longitude",
        });
    }
    let lat = parse_packed(lines[6], 7, "latitude")?.value();
    if !(-90.0..=90.0).contains(&lat) {
        return Err(StoreError::Jhd {
            line: 7,
            field: "latitude",
        });
    }

    let mut texts = lines
        .iter()
        .skip(8)
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && l.parse::<f64>().is_err());
    let place_name = texts.next().unwrap_or("").to_owned();
    let country = texts.next().unwrap_or("").to_owned();

    let mut warnings = Vec::new();
    let diff = (f64::from(offset_secs) / 240.0 - lon + 540.0).rem_euclid(360.0) - 180.0;
    if diff.abs() > JHD_OFFSET_LON_TOLERANCE_DEG {
        warnings.push(ImportWarningCode::JhdOffsetLongitudeMismatch);
    }

    Ok(JhdChart {
        name,
        birth: BirthData {
            date: date.to_string(),
            time: Some(time.to_string()),
            time_certainty: TimeCertainty::Exact,
            place: Place {
                name: place_name,
                country,
                admin1: String::new(),
                lat,
                lon,
                geoname_id: None,
            },
            tz: TzChoice::Fixed(offset_secs),
            fold: None,
            notes: format!("JHD: {file_name}"),
        },
        warnings,
    })
}

/// Girdi kaynaklı hata mı (dosya başına raporlanır), yoksa veritabanı hatası mı (durdurur)?
fn is_input_error(e: &StoreError) -> bool {
    matches!(
        e,
        StoreError::InvalidInput(_) | StoreError::Moment(_) | StoreError::Jhd { .. }
    )
}

impl Store {
    /// Her dosya bağımsızdır. Aynı ad, tarih, saat ve koordinatta harita varsa dosya
    /// atlanır (`skipped`); böylece aynı klasörü ikinci kez içe almak kopya üretmez.
    pub fn import_jhd(&mut self, files: &[(String, Vec<u8>)]) -> Result<JhdImport, StoreError> {
        let mut out = JhdImport {
            report: ImportReport::default(),
            failed: Vec::new(),
        };
        for (file_name, bytes) in files {
            let parsed = match parse_jhd(file_name, bytes) {
                Ok(p) => p,
                Err(e) => {
                    out.failed.push((file_name.clone(), e));
                    continue;
                }
            };
            let b = &parsed.birth;
            let exists: Option<u32> = self
                .conn
                .query_row(
                    "SELECT ch.id FROM charts ch JOIN clients c ON c.id = ch.client_id
                     WHERE c.name = ?1 AND ch.date = ?2 AND ch.time IS ?3
                       AND ch.lat = ?4 AND ch.lon = ?5",
                    params![parsed.name, b.date, b.time, b.place.lat, b.place.lon],
                    |r| r.get(0),
                )
                .optional()?;
            if exists.is_some() {
                out.report.skipped += 1;
                continue;
            }
            let input = NewClient {
                name: parsed.name.clone(),
                gender: None,
                chart: NewChart {
                    label: String::new(),
                    kind: ChartKind::Natal,
                    birth: parsed.birth.clone(),
                },
            };
            match self.create_client(&input) {
                Ok(_) => {
                    out.report.clients_added += 1;
                    out.report.charts_added += 1;
                    for &code in &parsed.warnings {
                        out.report.warn(ImportWarning {
                            item: file_name.clone(),
                            code,
                        });
                    }
                }
                Err(e) if is_input_error(&e) => out.failed.push((file_name.clone(), e)),
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::testutil::store;
    use crate::settings::{AppPrefs, SettingsLoad};
    use jyotish_core::{AyanamsaSystem, CalcSettings};

    /// tests/golden/new-delhi-2000-01-01.json anı, JHD biçiminde (17:30 IST).
    const DELHI: &str = "1\n1\n2000\n17.300000\n-5.300000\n-77.123240\n28.365004\n0.000000\n\
                         -5.300000\n-5.300000\n0\n105\nNew Delhi\nIndia\n1\n1013.250000\n20.000000\n2\n";

    #[test]
    fn packed_sexagesimal_values() {
        let p = parse_packed("-77.123240", 6, "longitude").unwrap();
        assert_eq!(
            p,
            Packed {
                negative: true,
                whole: 77,
                minutes: 12,
                seconds: 32,
                nanos: 400_000_000
            }
        );
        assert!((p.value() + 77.209).abs() < 1e-12);
        assert_eq!(parse_packed("5.3", 5, "timezone").unwrap().minutes, 30);
        assert_eq!(parse_packed("+0", 5, "timezone").unwrap().value(), 0.0);
        for bad in ["", "-", "1.7000", "1.0060", "a.1", "1.2x", "--1"] {
            assert!(
                matches!(
                    parse_packed(bad, 4, "time"),
                    Err(StoreError::Jhd {
                        line: 4,
                        field: "time"
                    })
                ),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn delhi_file_parses_to_the_golden_moment() {
        let c = parse_jhd("Test Kişi.jhd", DELHI.as_bytes()).unwrap();
        assert_eq!(c.name, "Test Kişi");
        assert!(c.warnings.is_empty());
        let b = &c.birth;
        assert_eq!(
            (b.date.as_str(), b.time.as_deref()),
            ("2000-01-01", Some("17:30:00"))
        );
        assert_eq!(b.tz, TzChoice::Fixed(19_800));
        assert!((b.place.lon - 77.209).abs() < 1e-9);
        assert!((b.place.lat - 28.6139).abs() < 1e-9);
        assert_eq!(
            (b.place.name.as_str(), b.place.country.as_str()),
            ("New Delhi", "India")
        );
        assert_eq!(b.notes, "JHD: Test Kişi.jhd");
        let (_, r) = crate::birth::resolve_birth(b).unwrap();
        assert_eq!(r.jd_ut, 2_451_545.0);
    }

    #[test]
    fn crlf_latin1_and_minimal_files() {
        let mut bytes =
            b"7\r\n20\r\n1969\r\n20.174000\r\n5.000000\r\n74.045100\r\n4.383500\r\n0\r\n".to_vec();
        bytes.extend_from_slice(b"Bogot\xe1\r\nColombia\r\n");
        let c = parse_jhd("a.jhd", &bytes).unwrap();
        assert_eq!(c.birth.place.name, "Bogotá");
        assert_eq!(c.birth.tz, TzChoice::Fixed(-18_000));
        // 74°04′51″ B.
        assert!((c.birth.place.lon + 74.080_833_333).abs() < 1e-6);
        assert!(c.warnings.is_empty());
        assert_eq!(c.birth.time.as_deref(), Some("20:17:40"));
        // Yalnızca 7 satır: yer adı boş.
        let c = parse_jhd("b.jhd", b"1\n1\n2000\n12.0\n0\n0\n51.3000").unwrap();
        assert_eq!(c.birth.place.name, "");
        assert_eq!(c.birth.place.lat, 51.5);
    }

    #[test]
    fn invalid_files_name_the_line() {
        let cases: [(&str, usize, &str); 7] = [
            ("1\n1\n2000\n", 4, "missing"),
            ("13\n1\n2000\n12\n0\n0\n0", 1, "month"),
            ("2\n30\n2000\n12\n0\n0\n0", 2, "day"),
            ("1\n1\n2000\n24.0\n0\n0\n0", 4, "time"),
            ("1\n1\n2000\n12\n-19\n0\n0", 5, "timezone"),
            ("1\n1\n2000\n12\n0\n-181\n0", 6, "longitude"),
            ("1\n1\n2000\n12\n0\n0\n91", 7, "latitude"),
        ];
        for (text, line, field) in cases {
            match parse_jhd("x.jhd", text.as_bytes()) {
                Err(StoreError::Jhd { line: l, field: f }) => {
                    assert_eq!((l, f), (line, field), "{text:?}")
                }
                other => panic!("{text:?}: {other:?}"),
            }
        }
        assert!(matches!(
            parse_jhd("  .jhd", DELHI.as_bytes()),
            Err(StoreError::Jhd {
                line: 0,
                field: "name"
            })
        ));
    }

    #[test]
    fn sign_mismatch_is_flagged_not_rejected() {
        // Boylam işareti ters (doğu pozitif yazılmış): Delhi Atlantik'e düşer.
        let flipped = DELHI.replace("-77.123240", "77.123240");
        let c = parse_jhd("x.jhd", flipped.as_bytes()).unwrap();
        assert_eq!(c.warnings, [ImportWarningCode::JhdOffsetLongitudeMismatch]);
        // Kaşgar (UTC+8, 75°59′ D): fark ≈ 44°, uyarı yok.
        let kashgar = "1\n1\n2000\n12\n-8\n-75.5900\n39.2800\n";
        assert!(parse_jhd("k.jhd", kashgar.as_bytes())
            .unwrap()
            .warnings
            .is_empty());
    }

    #[test]
    fn import_copies_defaults_reports_failures_and_skips_duplicates() {
        let mut s = store();
        let raman = CalcSettings {
            ayanamsa: AyanamsaSystem::Raman,
            ..CalcSettings::default()
        };
        s.set_app_prefs(&AppPrefs {
            calc_settings: raman.clone(),
            ..AppPrefs::default()
        })
        .unwrap();
        let flipped = DELHI.replace("-77.123240", "77.123240");
        let files = vec![
            ("Ayşe.jhd".to_owned(), DELHI.as_bytes().to_vec()),
            ("Bozuk.jhd".to_owned(), b"1\n1\n".to_vec()),
            ("Ters.jhd".to_owned(), flipped.into_bytes()),
        ];
        let r = s.import_jhd(&files).unwrap();
        assert_eq!(
            (
                r.report.clients_added,
                r.report.charts_added,
                r.report.skipped
            ),
            (2, 2, 0)
        );
        assert_eq!(r.failed.len(), 1);
        assert_eq!(r.failed[0].0, "Bozuk.jhd");
        assert!(matches!(
            r.failed[0].1,
            StoreError::Jhd {
                line: 3,
                field: "missing"
            }
        ));
        assert_eq!(
            r.report.warnings,
            [ImportWarning {
                item: "Ters.jhd".into(),
                code: ImportWarningCode::JhdOffsetLongitudeMismatch
            }]
        );
        let hits = s.search_clients("ayse", 10).unwrap();
        assert_eq!(hits.len(), 1);
        let ch = s.chart(hits[0].client.last_chart_id.unwrap()).unwrap();
        assert_eq!(ch.resolved.jd_ut, 2_451_545.0);
        assert_eq!(
            s.calc_settings(ch.id).unwrap(),
            SettingsLoad::Current(raman)
        );
        let again = s.import_jhd(&files[..1]).unwrap();
        assert_eq!((again.report.clients_added, again.report.skipped), (0, 1));
    }
}
