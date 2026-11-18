//! `__ID__`: the game's program: a window, or none with `headless=WxH` (launch keys only). Its command line is the
//! program `__ID___cli` (`src/bin/__ID___cli.rs`).

fn main() -> anyhow::Result<()> {
    __ID___game::run_native()
}
