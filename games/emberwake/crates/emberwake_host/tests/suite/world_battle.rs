//! WORLD-SIM1 (PR5-42, FB-239 (a)): `aegis battle world:<world>/<id>` on the shipped Act 1 world. Every encounter of
//! the world with an authored `party` starts (`cli::world_session`) with that party then its guests as the world's
//! characters (each at its base unit's level, read from the data; a Founder in its character's `default_intent`), by the
//! world's ruleset unless `ruleset=` picks one; the binary autoplays one to its outcome and refuses an unknown one. The
//! shipped data is only read.

use emberwake_battle::round_intent;
use emberwake_host::cli::world_session;
use emberwake_world::Data;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The Act 1 world (`CAST1_ENCOUNTERS.md`).
const WORLD: &str = "sunk_ward";

fn data_dir() -> PathBuf {
    Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")
}

fn data() -> Data {
    Data::open(&data_dir()).expect("data").0
}

/// The world's own encounters with an authored party.
fn authored(d: &Data) -> Vec<serde_json::Map<String, serde_json::Value>> {
    d.own_entries(WORLD, "encounters").iter().filter(|e| !aegis_core::json::sget(e, "party").is_empty()).cloned().collect()
}

#[test]
fn every_authored_world_fight_starts_with_its_authored_party_levels_and_intents() {
    let d = data();
    let fights = authored(&d);
    assert!(!fights.is_empty());
    for e in &fights {
        let id = aegis_core::json::s(e, "id");
        let s = world_session(&d, &format!("world:{WORLD}/{id}"), Some(11), "", "").unwrap_or_else(|err| panic!("{id}: {err}"));
        assert_eq!(s.def.ruleset.id, d.ruleset(WORLD).id, "{id}: the world's ruleset");
        assert_eq!(s.def.rng_seed, 11);
        let refs: Vec<String> = aegis_core::json::sget(e, "party").into_iter().chain(aegis_core::json::sget(e, "guests")).collect();
        assert_eq!(s.w.allies.len(), refs.len(), "{id}: party then guests");
        for (&c, r) in s.w.allies.iter().zip(&refs) {
            let character = d.resolve(WORLD, "characters", r);
            let base = d.entry("units", &aegis_core::json::s(character, "base_unit"));
            assert_eq!(i64::from(s.w.units[c].stats.level), aegis_core::json::i(base, "level", 1), "{id} {r}: the authored level");
            let intent = aegis_core::json::s(character, "default_intent");
            if !intent.is_empty() && round_intent::on(&s.w) {
                assert_eq!(round_intent::intent_of(&s.w, c).map(|i| i.id().to_string()), Some(intent), "{id} {r}: its default intent");
            }
        }
    }
    // ruleset= picks the rules over the world's
    let id = aegis_core::json::s(&fights[0], "id");
    let s = world_session(&d, &format!("world:{WORLD}/{id}"), None, "classic", "").expect("classic");
    assert_eq!(s.def.ruleset.id, "classic");
    assert!(world_session(&d, &format!("world:{WORLD}/no_such_fight"), None, "", "").is_err());
}

#[test]
fn aegis_battle_autoplays_a_world_encounter() {
    let d = data();
    let id = aegis_core::json::s(&authored(&d)[0], "id");
    let run = |reference: &str| Command::new(env!("CARGO_BIN_EXE_aegis")).args(["battle", reference, "seed=11"]).arg(format!("data={}", data_dir().display())).output().expect("runs");
    let out = run(&format!("world:{WORLD}/{id}"));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{text}\n{}", String::from_utf8_lossy(&out.stderr));
    assert!(text.lines().any(|l| l.starts_with("outcome: ")), "{text}");
    let name = aegis_core::json::s(&authored(&d)[0], "name");
    assert!(text.starts_with(&format!("{name} (seed 11)")), "{text}");
    assert!(!run(&format!("world:{WORLD}/no_such_fight")).status.success());
}
