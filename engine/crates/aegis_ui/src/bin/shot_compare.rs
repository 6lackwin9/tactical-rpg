//! Compares two PNG shots with `aegis_ui::shot_diff` (HEADLESS1), for tools and proofs: `shot_compare <a.png> <b.png> [s=1]`,
//! where `s` is the physical px per grid unit the tolerance comes from. Prints the report. Exit 0 they match, 1 they do not,
//! 2 a shot could not be read or compared.

use aegis_ui::art::load_png;
use aegis_ui::shot_diff::shot_diff;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(a), Some(b)) = (args.first(), args.get(1)) else {
        println!("usage: shot_compare <a.png> <b.png> [s=1]");
        return ExitCode::from(2);
    };
    let s = args.get(2).and_then(|t| t.strip_prefix("s=")).map_or(Some(1.0), |t| t.parse::<f32>().ok());
    let (Some(ia), Some(ib), Some(s)) = (load_png(Path::new(a)), load_png(Path::new(b)), s) else {
        println!("shot_compare: a shot is not a PNG, or s is not a number");
        return ExitCode::from(2);
    };
    match shot_diff(&ia, &ib, s) {
        Ok(report) => {
            println!("{report}");
            ExitCode::from(u8::from(!report.passed()))
        }
        Err(e) => {
            println!("{e}");
            ExitCode::from(2)
        }
    }
}
