//! emberwake: the desktop program. The game, its launch keys and its startup are the library's (`emberwake_gui`,
//! `run_native`); the web build is the library itself as a `cdylib` (its `web` module, WEBREL1-G1).

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    emberwake_gui::run_native()
}

/// The web build starts from the library's `wasm-bindgen` start function, not from a program.
#[cfg(target_arch = "wasm32")]
fn main() {}
