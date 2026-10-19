//! `aegis`: the engine's command line (`aegis_cli`) with Emberwake registered at start (`cli::Emberwake`).

fn main() -> anyhow::Result<()> {
    aegis_cli::run(&emberwake_host::cli::Emberwake)
}
