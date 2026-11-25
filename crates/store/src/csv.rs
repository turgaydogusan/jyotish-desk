//! CSV dışa aktarma (spec §5: yalnızca dışa aktarma). Harita başına bir satır; notlar
//! hiçbir zaman CSV'ye yazılmaz. RFC 4180: virgül ayırıcı, CRLF satır sonu, gerekirse çift
//! tırnak. Excel'in UTF-8'i tanıması için BOM ile başlar.

use rusqlite::params;

use crate::model::{ChartRecord, Client, TzChoice};
use crate::{Store, StoreError};

pub const CSV_COLUMNS: [&str; 24] = [
    "client_uuid",
    "client_name",
    "gender",
    "chart_uuid",
    "label",
    "kind",
    "date",
    "time",
    "time_certainty",
    "place_name",
    "country",
    "admin1",
    "lat",
    "lon",
    "tz_kind",
    "tz_id",
    "offset_override",
    "fold",
    "resolved_utc",
    "resolved_offset_secs",
    "tzdb_version",
    "tags",
    "birth_notes",
    "calc_settings_json",
];

/// Tek alan. Kullanıcı metni `=`, `+`, `-`, `@`, sekme ya da CR ile başlıyorsa başına `'`
/// eklenir (tablolama programlarında formül enjeksiyonu). Sayı alanları bu yoldan geçmez.
fn text(s: &str) -> String {
    let guarded = match s.chars().next() {
        Some('=' | '+' | '-' | '@' | '\t' | '\r') => format!("'{s}"),
        _ => s.to_owned(),
    };
    quote(&guarded)
}

fn quote(s: &str) -> String {
    if s.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}

fn row(c: &Client, ch: &ChartRecord, calc_json: &str) -> String {
    let b = &ch.birth;
    let (tz_kind, tz_id, offset) = match &b.tz {
        TzChoice::Iana(id) => ("Iana", id.clone(), String::new()),
        TzChoice::Fixed(secs) => ("Fixed", String::new(), secs.to_string()),
        TzChoice::Lmt => ("Lmt", String::new(), String::new()),
    };
    let fields = [
        quote(&c.uuid),
        text(&c.name),
        c.gender.map(|g| g.as_str()).unwrap_or("").to_owned(),
        quote(&ch.uuid),
        text(&ch.label),
        ch.kind.as_str().to_owned(),
        quote(&b.date),
        quote(b.time.as_deref().unwrap_or("")),
        b.time_certainty.as_str().to_owned(),
        text(&b.place.name),
        text(&b.place.country),
        text(&b.place.admin1),
        b.place.lat.to_string(),
        b.place.lon.to_string(),
        tz_kind.to_owned(),
        text(&tz_id),
        offset,
        b.fold.map(|f| f.as_str()).unwrap_or("").to_owned(),
        text(&ch.resolved.utc),
        ch.resolved.offset_secs.to_string(),
        text(&ch.resolved.tzdb_version),
        text(&ch.tags.join("; ")),
        text(&b.notes),
        quote(calc_json),
    ];
    fields.join(",")
}

impl Store {
    /// Tüm haritalar; danışan ve harita kimliği sırasında.
    pub fn export_csv(&self) -> Result<String, StoreError> {
        let mut out = String::from("\u{feff}");
        out.push_str(&CSV_COLUMNS.join(","));
        out.push_str("\r\n");
        let mut stmt = self.conn.prepare(
            "SELECT ch.id, ch.client_id, ch.calc_settings_json FROM charts ch
             ORDER BY ch.client_id, ch.id",
        )?;
        let rows = stmt
            .query_map(params![], |r| {
                Ok((
                    r.get::<_, u32>(0)?,
                    r.get::<_, u32>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (chart_id, client_id, calc) in rows {
            let c = self.client(client_id)?;
            let ch = self.chart(chart_id)?;
            out.push_str(&row(&c, &ch, &calc));
            out.push_str("\r\n");
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::birth::fixtures::delhi;
    use crate::charts::testutil::new_client;
    use crate::db::testutil::store;
    use crate::model::{BirthData, NewClient};

    #[test]
    fn quoting_and_formula_guard() {
        assert_eq!(quote("plain"), "plain");
        assert_eq!(quote("a,b"), "\"a,b\"");
        assert_eq!(quote("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(quote("two\nlines"), "\"two\nlines\"");
        assert_eq!(text("=HYPERLINK(\"x\")"), "\"'=HYPERLINK(\"\"x\"\")\"");
        assert_eq!(text("-5"), "'-5");
        assert_eq!(text("Ayşe"), "Ayşe");
    }

    #[test]
    fn one_row_per_chart_without_notes() {
        let mut s = store();
        let (c, ch) = s
            .create_client(&NewClient {
                chart: crate::model::NewChart {
                    birth: BirthData {
                        notes: "doğum belgesi, 1. sayfa".into(),
                        ..delhi()
                    },
                    ..new_client("x").chart
                },
                ..new_client("Yılmaz, Ayşe")
            })
            .unwrap();
        s.set_chart_tags(ch.id, &["b".into(), "a".into()]).unwrap();
        s.add_note(c.id, None, "GİZLİ").unwrap();
        let csv = s.export_csv().unwrap();
        assert!(csv.starts_with("\u{feff}client_uuid,client_name,"));
        assert!(!csv.contains("GİZLİ"));
        let lines: Vec<&str> = csv.trim_end_matches("\r\n").split("\r\n").collect();
        assert_eq!(lines.len(), 2);
        let expected_start = format!(
            "{},\"Yılmaz, Ayşe\",,{},,Natal,2000-01-01,17:30:00,Exact,New Delhi,India,NCT,28.6139,77.209,Iana,Asia/Kolkata,,,2000-01-01T12:00:00Z,19800,{},a; b,\"doğum belgesi, 1. sayfa\",\"{{\"\"schema_version\"\":1,",
            c.uuid,
            ch.uuid,
            atlas::tzdb_version()
        );
        assert!(lines[1].starts_with(&expected_start), "{}", lines[1]);
    }
}
