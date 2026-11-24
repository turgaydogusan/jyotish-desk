//! Kullanım: build-atlas <cities1000.txt> <admin1CodesASCII.txt> <countryInfo.txt> <source-date> <out.sqlite>

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [cities, admin1, countries, date, out] = args.as_slice() else {
        eprintln!("usage: build-atlas <cities1000.txt> <admin1CodesASCII.txt> <countryInfo.txt> <source-date> <out.sqlite>");
        return ExitCode::from(2);
    };
    match atlas::db::build_atlas_from_files(
        &PathBuf::from(cities),
        &PathBuf::from(admin1),
        &PathBuf::from(countries),
        date,
        &PathBuf::from(out),
    ) {
        Ok(stats) => {
            println!(
                "inserted {} places, skipped {} with unknown time zones",
                stats.inserted, stats.skipped_unknown_tz
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("build-atlas: {e}");
            ExitCode::FAILURE
        }
    }
}
