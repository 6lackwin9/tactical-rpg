//! `kit_sample`: the game's program: a window, or none with `headless=WxH` (launch keys only). Its command line is the
//! program `kit_sample_cli` (`src/bin/kit_sample_cli.rs`).

fn main() -> anyhow::Result<()> {
    kit_sample_game::run_native()
}
