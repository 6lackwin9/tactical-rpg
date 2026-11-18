//! UISPLIT1 (PR7-38): screen layout files from the command line, with no game running.
//!
//!   ui_layout check <game folder> ...           every layout and fixture of each game's asset layers parses, and every
//!                                               layout fits its fixture (exit 1 on a problem; `tools/fc.sh layouts`, in the gate)
//!   ui_layout shot <game folder> <screen> [size=1280x720] [out=<file.png>] [default=1]
//!                                               the fixture drawn through the layout (or the default) to a PNG, for review

use aegis_core::assets::Layers;
use aegis_ui::screen_layout;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let key = |k: &str| args.iter().find_map(|a| a.strip_prefix(&format!("{k}=")).map(str::to_string));
    let pos: Vec<&String> = args.iter().filter(|a| !a.contains('=')).collect();
    match pos.first().map(|s| s.as_str()) {
        Some("check") if pos.len() > 1 => {
            let mut bad = 0;
            for game in &pos[1..] {
                let layers = Layers::for_game(Path::new(game.as_str()));
                let screens = screen_layout::screens(&layers);
                let problems = screen_layout::check_layers(&layers);
                for p in &problems {
                    println!("{game}: {p}");
                }
                println!("{game}: {} screen(s) with a layout or a fixture, {} problem(s)", screens.len(), problems.len());
                bad += problems.len();
            }
            if bad == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Some("shot") if pos.len() == 3 => {
            let size = key("size").unwrap_or_else(|| "1280x720".into());
            let Some((w, h)) = size.split_once('x').and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?))).filter(|(w, h)| (640..=3840).contains(w) && (360..=2160).contains(h)) else {
                println!("size is WxH, 640x360 to 3840x2160");
                return ExitCode::from(2);
            };
            let screen = pos[2].as_str();
            let out = key("out").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(format!("{screen}.png")));
            let layers = Layers::for_game(Path::new(pos[1].as_str()));
            match screen_layout::render(&layers, screen, (w, h), key("default").as_deref() == Some("1")) {
                Ok(r) => {
                    for p in &r.problems {
                        println!("drawn with the default: {p}");
                    }
                    if let Err(e) = aegis_ui::shell::save_png(&r.canvas, &out, w as i32) {
                        println!("{e:#}");
                        return ExitCode::from(1);
                    }
                    println!("wrote {} ({} widgets)", out.display(), r.widgets.len());
                    ExitCode::SUCCESS
                }
                Err(ps) => {
                    for p in ps {
                        println!("{p}");
                    }
                    ExitCode::from(1)
                }
            }
        }
        _ => {
            println!("ui_layout check <game folder> ... | shot <game folder> <screen> [size=WxH] [out=<file.png>] [default=1]");
            ExitCode::from(2)
        }
    }
}
