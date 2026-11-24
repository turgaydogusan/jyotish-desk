//! GeoNames döküm dosyalarını ayrıştırma: cities1000, admin1CodesASCII, countryInfo.
//! Biçim: sekmeyle ayrılmış, tırnaksız, UTF-8.

use std::collections::HashMap;
use std::io::BufRead;

#[derive(Debug, Clone, PartialEq)]
pub struct CityRecord {
    pub geoname_id: i64,
    pub name: String,
    pub ascii_name: String,
    pub lat: f64,
    pub lon: f64,
    pub country_code: String,
    pub admin1_code: String,
    pub population: i64,
    pub timezone: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GeonamesError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("line {line}: {reason}")]
    BadLine { line: usize, reason: String },
}

const CITY_FIELDS: usize = 19;

pub fn parse_city_line(line: &str) -> Result<CityRecord, String> {
    let f: Vec<&str> = line.split('\t').collect();
    if f.len() != CITY_FIELDS {
        return Err(format!("expected {CITY_FIELDS} fields, got {}", f.len()));
    }
    let geoname_id = f[0]
        .parse::<i64>()
        .map_err(|_| format!("bad geonameid: {:?}", f[0]))?;
    let coord = |i: usize, what: &str| {
        f[i].parse::<f64>()
            .map_err(|_| format!("bad {what}: {:?}", f[i]))
    };
    let (lat, lon) = (coord(4, "latitude")?, coord(5, "longitude")?);
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        return Err(format!("coordinates out of range: {lat}, {lon}"));
    }
    let population = if f[14].is_empty() {
        0
    } else {
        f[14]
            .parse::<i64>()
            .map_err(|_| format!("bad population: {:?}", f[14]))?
    };
    Ok(CityRecord {
        geoname_id,
        name: f[1].to_owned(),
        ascii_name: f[2].to_owned(),
        lat,
        lon,
        country_code: f[8].to_owned(),
        admin1_code: f[10].to_owned(),
        population,
        timezone: f[17].to_owned(),
    })
}

pub fn read_cities<R: BufRead>(r: R) -> Result<Vec<CityRecord>, GeonamesError> {
    let mut out = Vec::new();
    for (i, line) in r.lines().enumerate() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let rec = parse_city_line(&line).map_err(|reason| GeonamesError::BadLine {
            line: i + 1,
            reason,
        })?;
        out.push(rec);
    }
    Ok(out)
}

/// "TR.34\tIstanbul\tIstanbul\t745042" → "TR.34" ↦ "Istanbul".
pub fn read_admin1<R: BufRead>(r: R) -> Result<HashMap<String, String>, GeonamesError> {
    let mut map = HashMap::new();
    for (i, line) in r.lines().enumerate() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 2 {
            return Err(GeonamesError::BadLine {
                line: i + 1,
                reason: "expected code and name".into(),
            });
        }
        map.insert(f[0].to_owned(), f[1].to_owned());
    }
    Ok(map)
}

/// countryInfo.txt: `#` ile başlayan satırlar yorumdur; alan 0 ISO kodu, alan 4 ülke adı.
pub fn read_countries<R: BufRead>(r: R) -> Result<HashMap<String, String>, GeonamesError> {
    let mut map = HashMap::new();
    for (i, line) in r.lines().enumerate() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 5 {
            return Err(GeonamesError::BadLine {
                line: i + 1,
                reason: "expected at least 5 fields".into(),
            });
        }
        map.insert(f[0].to_owned(), f[4].to_owned());
    }
    Ok(map)
}
