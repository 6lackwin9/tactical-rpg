//! R4-2 (3), C28 (battle -> X4): the step stream a battle screen plays its effects from, on the frozen timeline catalog
//! (`tests/fixtures/timeline_kit.json`, the timeline golden's) under the `timeline` ruleset. A screen steps the session
//! (`Session::step_next`) and, after each step, reads the action the step popped (`last_popped`: round, block, index
//! into `resolver.scheduled`; the actor's clip and the impact, `fx::act_kind` / `fx::style` in the GUI) and the log
//! cells it appended (the floats). Over whole fights (a `tl_` fight and the class-unit fight): every pop names the
//! current round and the block it resolved in and an action scheduled this round, never one popped before; a step pops at most one
//! action; every popped action that was not broken appends its own log cell (its actor's name); and every scheduled
//! action with an effect (attack, area, support, item) that was not cancelled (its unit stunned, its intent target gone,
//! ...) is popped once in its round of a fight that did not end mid-round, so no effect is skipped.

use emberwake_battle::command::CmdKind;
use emberwake_battle::session::{EncounterDef, Phase, Session};
use emberwake_data::{Catalog, UnitDef};
use emberwake_rules::ruleset::{Ruleset, TIMELINE};
use indexmap::IndexMap;
use serde_json::{Map, Value};
use std::path::PathBuf;
use std::sync::Arc;

const FIGHTS: [(&str, i64); 2] = [("tl_skirmish", 1337), ("goblin_skirmish", 7)];
const PRESET: &str = "balanced";

fn kit() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/timeline_kit.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("the frozen timeline catalog")).unwrap()
}

/// The encounter `id` of the frozen catalog under the timeline row, seeded `seed`, started.
fn start(raw: &Value, id: &str, seed: i64) -> Session {
    let cat = Arc::new(Catalog::from_value(raw));
    let ruleset = Ruleset::of(&cat, TIMELINE).expect("the frozen catalog's timeline row");
    let mut entry: Map<String, Value> = raw["encounters"].as_array().unwrap().iter().filter_map(Value::as_object).find(|e| e["id"] == id).unwrap_or_else(|| panic!("{id}")).clone();
    entry.insert("rng_seed".into(), seed.into());
    let def = EncounterDef::from_obj_with(&entry, ruleset);
    Session::start(cat, def, &IndexMap::<String, UnitDef>::new())
}

fn has_effect(kind: &CmdKind) -> bool {
    matches!(kind, CmdKind::Attack { .. } | CmdKind::Area { .. } | CmdKind::Support { .. } | CmdKind::Item { .. })
}

#[test]
fn every_step_pops_one_scheduled_action_and_logs_it() {
    let raw = kit();
    for (id, seed) in FIGHTS {
        let mut s = start(&raw, id, seed);
        let (mut pops, mut effects) = (0, 0);
        let cap = s.def.max_rounds.max(1);
        for _ in 0..cap {
            if s.combat_over {
                break;
            }
            s.auto_draft(PRESET);
            assert!(s.confirm_plan(), "{id}: the drafted plan confirms");
            let mut popped_this_round: Vec<usize> = Vec::new();
            let mut guard = 0;
            while s.phase != Phase::Replay && !s.combat_over {
                guard += 1;
                assert!(guard < 10_000, "{id}: the round resolves");
                let (before, log_before) = (s.last_popped, s.log.len());
                s.step_next();
                let Some((round, block, idx)) = s.last_popped.filter(|p| Some(*p) != before) else { continue };
                // the block's last pop moves the session to the next block (or to the replay at the round's end) in the
                // same step, so the pop names the block it resolved in
                let closed = block + 1 == s.current_block || s.phase == Phase::Replay || s.combat_over;
                assert!(round == s.current_round && (block == s.current_block || closed), "{id}: pop ({round}, {block}) at ({}, {})", s.current_round, s.current_block);
                assert!(idx < s.resolver.scheduled.len(), "{id}: an action scheduled this round");
                assert!(!popped_this_round.contains(&idx), "{id} round {round}: action {idx} popped twice");
                popped_this_round.push(idx);
                pops += 1;
                let a = &s.resolver.scheduled[idx];
                if !a.broken {
                    let actor = &s.w.units[a.unit].name;
                    assert!(s.log[log_before..].iter().any(|e| &e.actor == actor), "{id} round {round} block {block}: the popped action of {actor} ({}) has its log cell", a.command.name);
                }
            }
            // a fight that ends mid-round leaves the rest of the round unresolved
            if s.combat_over {
                break;
            }
            let live = |i: &usize| !s.resolver.scheduled[*i].cancelled && has_effect(&s.resolver.scheduled[*i].command.kind);
            let round_effects: Vec<usize> = (0..s.resolver.scheduled.len()).filter(live).collect();
            for i in &round_effects {
                assert!(popped_this_round.contains(i), "{id} round {}: the {} of {} was never popped", s.current_round, s.resolver.scheduled[*i].command.name, s.w.units[s.resolver.scheduled[*i].unit].name);
            }
            effects += round_effects.len();
            if s.combat_over || !s.can_advance_round() {
                break;
            }
            s.advance_round();
        }
        assert!(pops > 0 && effects > 0, "{id}: the fight popped actions with effects ({pops} pops, {effects} effects)");
    }
}
