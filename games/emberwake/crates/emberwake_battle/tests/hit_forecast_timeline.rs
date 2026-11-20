//! HITCALC1 (PR7-45): forecast equals result over the timeline golden's fights. The resolver's hit (`Command::land`)
//! and every forecast (`intent::hit_on`) share one hit-input builder (`command::hit_terms`), so a forecast read on the
//! round as it will run must equal what the hit then does. Each `tl_` fight and `goblin_skirmish` of the timeline
//! golden (`timeline_golden.rs`: the frozen catalog `tests/fixtures/timeline_kit.json`, the same boards, seeds and
//! party preset) is planned by the auto driver and resolved block by block with the resolver; just before each
//! damaging Attack or Area lands, every unit on the field is forecast (`intent::hit_on_adjusted`, the crit the hit then
//! rolls forced on or off), and after it lands the forecast must match: a hit forecast to land deals exactly its
//! forecast damage to that unit, a hit forecast to fail (dodge, out of reach, protected flank) fails, and no unit the
//! forecast passes over is struck. Read only on the frozen catalog; no golden is written.

use aegis_core::json;
use emberwake_battle::command::CmdKind;
use emberwake_battle::intent::{self, Adjust, HitFail};
use emberwake_battle::session::{EncounterDef, Phase, Session};
use emberwake_data::{Catalog, UnitDef};
use emberwake_rules::ruleset::{Ruleset, TIMELINE};
use indexmap::IndexMap;
use serde_json::{json, Map, Value};
use std::path::PathBuf;
use std::sync::Arc;

/// The timeline golden's fights, seeds and preset (`timeline_golden.rs`).
const ENCOUNTERS: [&str; 6] = ["tl_duel_windup", "tl_duel_reach", "tl_duel_armour", "tl_flank", "tl_skirmish", "goblin_skirmish"];
const SEEDS: [i64; 3] = [1337, 7, 42];
const PRESET: &str = "balanced";
const BOARD_KEYS: [&str; 11] = ["zone", "arena", "exits", "rows", "total_blocks", "unit_ap", "ap_regen", "deployment", "objective", "phases", "guests"];

fn frozen_catalog() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/timeline_kit.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("tests/fixtures/timeline_kit.json")).unwrap()
}

/// The fight's session as `sim::fight` starts it (no jitter, no pouch).
fn session(cat: &Arc<Catalog>, entry: &Map<String, Value>, ruleset: &Ruleset, seed: i64) -> Session {
    let mut row = json!({"id": "sim_fight", "name": "Sim", "zone": "stone_arena", "arena": "CLOSED", "total_blocks": 6, "unit_ap": 12, "ap_regen": 4});
    let obj = row.as_object_mut().unwrap();
    for k in BOARD_KEYS {
        if let Some(v) = entry.get(k) {
            obj.insert(k.to_string(), v.clone());
        }
    }
    obj.insert("party".into(), entry.get("party").cloned().unwrap_or(json!([])));
    obj.insert("enemies".into(), entry.get("enemies").cloned().unwrap_or(json!([])));
    obj.insert("max_rounds".into(), json!(json::i(entry, "max_rounds", 8)));
    obj.insert("rng_seed".into(), json!(seed));
    let def = EncounterDef::from_obj_with(obj, ruleset.clone());
    Session::start(cat.clone(), def, &IndexMap::<String, UnitDef>::new())
}

/// One fight: (hits compared, failures forecast and seen, mismatches).
fn check_fight(s: &mut Session, label: &str) -> (usize, usize, Vec<String>) {
    let (mut compared, mut failed, mut bad) = (0, 0, Vec::new());
    for _ in 0..s.def.max_rounds {
        s.auto_draft(PRESET);
        if !s.confirm_plan() {
            break;
        }
        for b in 1..=s.w.total_blocks {
            let resolving = s.resolver.prepare_block(&mut s.w, b);
            for i in resolving {
                let a = s.resolver.scheduled[i].clone();
                let damaging = match &a.command.kind {
                    CmdKind::Attack { skill_mult, .. } | CmdKind::Area { skill_mult, .. } => *skill_mult > 0.0,
                    _ => false,
                };
                let live = damaging && !a.cancelled && s.w.units[a.unit].is_active();
                // every unit on the field, forecast both ways (the crit is rolled at the hit)
                let fc: Vec<(usize, Option<(intent::HitOn, intent::HitOn)>)> = if live {
                    let sched = &s.resolver.scheduled;
                    (0..s.w.units.len())
                        .filter(|&u| u != a.unit && s.w.has(u))
                        .map(|u| (u, intent::hit_on_adjusted(&s.w, sched, &a, u, false, &Adjust::NONE).zip(intent::hit_on_adjusted(&s.w, sched, &a, u, true, &Adjust::NONE))))
                        .collect()
                } else {
                    Vec::new()
                };
                s.resolver.execute_single(&mut s.w, i);
                if !live || s.resolver.scheduled[i].cancelled {
                    continue;
                }
                let cmd = &s.resolver.scheduled[i].command;
                let crit_on = |u: usize| match &cmd.kind {
                    CmdKind::Attack { last_crit, .. } => *last_crit,
                    CmdKind::Area { crit_hits, .. } => crit_hits.iter().any(|(v, _)| *v == u),
                    _ => false,
                };
                let at = format!("{label} round {} block {b} {} {}", s.current_round, s.w.units[a.unit].name, cmd.name);
                for (u, f) in &fc {
                    let struck: Vec<i32> = cmd.struck.iter().filter(|x| x.target == *u).map(|x| x.damage).collect();
                    match f {
                        None if !struck.is_empty() => bad.push(format!("{at}: {} struck for {struck:?}, forecast none", s.w.units[*u].name)),
                        None => {}
                        Some((plain, crit)) => {
                            let h = if crit_on(*u) { crit } else { plain };
                            match h.fails {
                                Some(HitFail::Dodged | HitFail::OutOfReach | HitFail::FlankProtected) => {
                                    failed += 1;
                                    if !struck.is_empty() {
                                        bad.push(format!("{at}: forecast {:?} on {}, struck {struck:?} ({})", h.fails, s.w.units[*u].name, cmd.log));
                                    }
                                }
                                // an intercept splits the hit by the holder's HP (not forecast): only its kind is checked
                                Some(HitFail::Intercepted { .. }) => {}
                                None => {
                                    compared += 1;
                                    if struck != vec![h.damage] {
                                        bad.push(format!("{at}: {} forecast {} (crit {}), struck {struck:?} ({})", s.w.units[*u].name, h.damage, crit_on(*u), cmd.log));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let side_up = |side: &[usize]| side.iter().any(|&c| s.w.units[c].is_active());
        if !side_up(&s.w.allies) || !side_up(&s.w.enemies) {
            break;
        }
        s.phase = Phase::Replay;
        if !s.can_advance_round() {
            break;
        }
        s.advance_round();
    }
    (compared, failed, bad)
}

/// HITCALC1: every hit of the timeline fights lands as forecast (same cover, tiles, guard and stances), and a hit
/// forecast to fail at its impact fails.
#[test]
fn every_hit_lands_as_forecast_in_the_timeline_fights() {
    emberwake_battle::plan::set_escape_plan1(false);
    let raw = frozen_catalog();
    let cat = Arc::new(Catalog::from_value(&raw));
    let ruleset = Ruleset::of(&cat, TIMELINE).expect("the frozen catalog's timeline row");
    let rows = json::aget(json::dict(&raw), "encounters");
    let (mut compared, mut failed, mut bad) = (0, 0, Vec::new());
    for id in ENCOUNTERS {
        let entry = rows.iter().filter_map(Value::as_object).find(|e| json::s(e, "id") == id).unwrap_or_else(|| panic!("encounter {id}"));
        for seed in SEEDS {
            let mut s = session(&cat, entry, &ruleset, seed);
            let (c, f, b) = check_fight(&mut s, &format!("{id} seed {seed}"));
            compared += c;
            failed += f;
            bad.extend(b);
        }
    }
    eprintln!("hits compared: {compared}, forecast failures: {failed}, mismatches: {}", bad.len());
    assert!(compared > 50, "the fights land hits ({compared})");
    assert!(failed > 0, "the fights hold hits that fail at their impact ({failed})");
    assert!(bad.is_empty(), "{} of {compared} hits differ from their forecast:\n{}", bad.len(), bad.iter().take(25).cloned().collect::<Vec<_>>().join("\n"));
}
