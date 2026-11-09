//! R4-2 (3), C27 / X6 on a scratch world (no shipped data): a `GameState` round-trips through each of the three save
//! gates, the autosave slot (`AUTOSAVE_SLOT`), a manual slot (1..`LAST_SLOT`) and the keyed act-start save (CAMPAIGN1
//! F5, `campaign::act_save_key`): written, read back (migrate, then validate) without a warning, the restored state
//! equals the original, and writing the restored state again gives the same file byte for byte. The state carries the
//! blocks added since the last save version: a member's packed pouch targets (`UnitState.pouch_packed`, W2-4 K3) and
//! the `dialogue` roll stream of `GameRolls` (W2-6 T1.2-S2), drawn from before the save.

use aegis_core::scratch::Scratch;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{campaign, rolls, save, Data, GameState};
use serde_json::json;
use std::path::{Path, PathBuf};

const POTION: &str = "p_potion";
/// The packed target set on the potion slot (below the default `min(5, max_stack)`, so it is a saved choice).
const PACKED: i64 = 2;
/// Dialogue roll draws before the save (the stream count and position must come back).
const DRAWS: usize = 3;
const SAVED_AT: i64 = 42;

/// World `w`: a campaign from act 1, a camp, the party unit `hero` (a class-derived party unit), a stackable potion.
fn data() -> Data {
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "skill_path": {}}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}],
        "items": [{"id": POTION, "name": "Potion", "item_type": "CONSUMABLE", "consumable_type": "HEALTH_POTION", "heal": 10, "max_stack": 3}]
    });
    let worlds = json!({
        "w": {"manifest": {"id": "w", "start_location": "camp", "start_party": ["hero"], "campaign": {"start_act": 1}}, "own": {
            "locations": [{"id": "camp", "name": "Camp", "_world": "w", "_qid": "w:camp"}]
        }}
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// A started game whose hero carries a potion slot with a packed target, a stash, and `DRAWS` dialogue rolls drawn.
fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    let mut hero = UnitState::create("hero", "hero");
    hero.assign_pouch(d, POTION).expect("a consumable slot");
    hero.set_packed(d, POTION, PACKED).expect("a packed target within the stack");
    party.members.push(hero);
    party.add_to_stash(POTION, 4);
    let mut gs = GameState::create(d, "w", party);
    for _ in 0..DRAWS {
        rolls::range_i(&mut gs, rolls::DIALOGUE, 1, 20);
    }
    gs
}

/// The one file a gate wrote into `dir` (each gate writes into its own folder).
fn only_file(dir: &Path) -> PathBuf {
    let files: Vec<PathBuf> = std::fs::read_dir(dir).expect("the save folder").filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.is_file()).collect();
    assert_eq!(files.len(), 1, "one save file in {}: {files:?}", dir.display());
    files[0].clone()
}

type Write = fn(&Data, &Path, &GameState) -> Vec<String>;
type Read = fn(&Data, &Path) -> Result<save::Loaded, Vec<String>>;

fn key(gs: &GameState) -> String {
    campaign::act_save_key(1, &gs.campaign.as_ref().expect("a campaign world").run)
}

#[test]
fn a_state_round_trips_byte_for_byte_through_every_save_gate() {
    let d = data();
    let gs = state(&d);
    assert_eq!(gs.party.members[0].pouch_packed.get(POTION), Some(&PACKED), "the fixture sets a packed target");
    let saved = gs.to_value();
    assert!(saved["rolls"].to_string().contains(rolls::DIALOGUE), "the dialogue stream is in the state: {}", saved["rolls"]);
    let gates: [(&str, Write, Read); 3] = [
        ("autosave", |d, dir, gs| save::write(d, dir, gs, save::AUTOSAVE_SLOT, SAVED_AT), |d, dir| save::read(d, dir, save::AUTOSAVE_SLOT)),
        ("manual slot", |d, dir, gs| save::write(d, dir, gs, save::LAST_SLOT, SAVED_AT), |d, dir| save::read(d, dir, save::LAST_SLOT)),
        ("act-start key", |d, dir, gs| save::write_keyed(d, dir, gs, &key(gs), SAVED_AT), |d, dir| save::read_keyed(d, dir, &key(&data_state()))),
    ];
    for (gate, write, read) in gates {
        let first = Scratch::vacant(&format!("rpgkit_save_gates_{}_a", gate.replace([' ', '-'], "_")));
        let second = Scratch::vacant(&format!("rpgkit_save_gates_{}_b", gate.replace([' ', '-'], "_")));
        std::fs::create_dir_all(&*first).unwrap();
        std::fs::create_dir_all(&*second).unwrap();
        assert!(write(&d, &first, &gs).is_empty(), "{gate}: written");
        let loaded = read(&d, &first).unwrap_or_else(|e| panic!("{gate}: reads back: {e:?}"));
        assert!(loaded.warnings.is_empty(), "{gate}: no load warning: {:?}", loaded.warnings);
        assert_eq!(loaded.saved_at, SAVED_AT, "{gate}: the save time");
        assert_eq!(loaded.state, gs, "{gate}: the restored state equals the original");
        assert_eq!(loaded.state.party.members[0].pouch_packed, gs.party.members[0].pouch_packed, "{gate}: pouch_packed");
        assert_eq!(loaded.state.rolls, gs.rolls, "{gate}: the roll streams, the dialogue stream included");
        assert!(write(&d, &second, &loaded.state).is_empty(), "{gate}: the restored state writes");
        let (a, b) = (std::fs::read_to_string(only_file(&first)).unwrap(), std::fs::read_to_string(only_file(&second)).unwrap());
        assert_eq!(a, b, "{gate}: save -> load -> save is byte-identical");
    }
}

/// The fixture state again (the keyed gate reads by the run's key, which the fixture's campaign names).
fn data_state() -> GameState {
    state(&data())
}

#[test]
fn the_next_dialogue_draw_after_a_reload_continues_the_saved_stream() {
    let d = data();
    let gs = state(&d);
    let dir = Scratch::vacant("rpgkit_save_gates_stream");
    std::fs::create_dir_all(&*dir).unwrap();
    assert!(save::write(&d, &dir, &gs, 1, SAVED_AT).is_empty());
    let mut back = save::read(&d, &dir, 1).expect("reads back").state;
    let mut kept = gs.clone();
    let faces = |g: &mut GameState| (0..DRAWS).map(|_| rolls::range_i(g, rolls::DIALOGUE, 1, 1_000_000)).collect::<Vec<_>>();
    assert_eq!(faces(&mut back), faces(&mut kept), "the reloaded stream draws what the unsaved one draws next");
}
