//! GOLDEN-BISECT1 (ENGINE-14): named test-only switches. A builder keeps a rule change that moves a golden behind
//! `switches::on("<name>")` instead of a temporary edit; `tools/fc.sh golden-bisect <golden> <name>` then records the
//! golden with the switch off and on and lists what moves. A switch is on when its name is in the comma list of the
//! environment variable `AEGIS_SWITCHES`, and only in a debug build (tests, dev runs): a release build never reads it,
//! so a switch can never change a shipped game. The list is read once per process.

use std::collections::BTreeSet;
use std::sync::OnceLock;

/// The environment variable that names the switches on (`a,b`).
pub const ENV: &str = "AEGIS_SWITCHES";

/// The switch names in a comma list (trimmed, empty names dropped).
pub fn parse(list: &str) -> BTreeSet<String> {
    list.split(',').map(str::trim).filter(|s| !s.is_empty()).map(String::from).collect()
}

/// Whether switch `name` is on (see the module).
pub fn on(name: &str) -> bool {
    static ON: OnceLock<BTreeSet<String>> = OnceLock::new();
    cfg!(debug_assertions) && ON.get_or_init(|| std::env::var(ENV).map(|v| parse(&v)).unwrap_or_default()).contains(name)
}
