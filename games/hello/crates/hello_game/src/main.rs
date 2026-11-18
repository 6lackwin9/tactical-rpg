//! `hello`: the sample game's program: a window, or none with `headless=WxH` (launch keys only). Its command line is
//! the program `hello_cli` (`src/bin/hello_cli.rs`).

fn main() -> anyhow::Result<()> {
    hello_game::run_native()
}
