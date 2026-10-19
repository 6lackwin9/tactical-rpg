//! World battles step for step: every world encounter (and the synthetic variants that cover escape, centre
//! deployment, morale, custom hold tiles and start buffs) as Godot's WorldEncounter plays it, recorded by
//! `games/emberwake/parity/trace.sh mode=world preset=<p>` into parity/out/traces_world_<p>.json. The Rust session runs the
//! same roster (world characters with named items, passives and tuning), guests, objectives, phases and morale; units
//! (HP, AP, tile, stance, statuses, withdrawn) are compared after every step, the schedule every round, then the outcome.

use emberwake_battle::session::{EncounterDef, Phase, Session, StartGrant};
use aegis_core::json;
use emberwake_world::battle::WorldRoster;
use emberwake_world::Data;
use serde_json::{json, Value};
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity/out")
}

fn units_state(s: &Session) -> Value {
    let w = &s.w;
    let mut out = Vec::new();
    for &c in w.allies.iter().chain(w.enemies.iter()) {
        let u = &w.units[c];
        let p = w.grid.position(c);
        let mut active: Vec<&String> = u.st.active.iter().collect();
        let mut pending: Vec<&String> = u.st.pending.iter().collect();
        active.sort();
        pending.sort();
        out.push(json!([u.name, u.hp, u.ap, p.x, p.y, u.stance as i32, active, pending, u.is_withdrawn]));
    }
    Value::Array(out)
}

fn phase_int(p: Phase) -> i64 {
    match p {
        Phase::Planning => 0,
        Phase::Resolving => 1,
        Phase::Replay => 2,
    }
}

/// The Rust trace of one Godot fight record.
fn run(d: &Data, f: &serde_json::Map<String, Value>) -> Value {
    let world = json::s(f, "world");
    let id = json::s(f, "encounter");
    let mut entry = d.resolve(&world, "encounters", &id).clone();
    for (k, v) in json::dget(f, "override") {
        entry.insert(k.clone(), v.clone());
    }
    let mut def = EncounterDef::from_obj(&entry);
    def.id = format!("{world}:{id}");
    def.rng_seed = json::i(f, "seed", 1);
    if json::b(f, "start_buffs", false) {
        def.start_grant = StartGrant::All;
    }
    let roster = WorldRoster { d, world: world.clone() };
    let mut s = Session::start(d.cat.clone(), def, &roster);
    let preset = json::s_or(f, "preset", "balanced");
    let start = units_state(&s);
    let mut rounds = Vec::new();
    for _ in 0..s.def.max_rounds {
        s.plan_apply_preset_all(&preset);
        if !s.confirm_plan() {
            break;
        }
        let actions: Vec<Value> = s.resolver.scheduled.iter().map(|a| json!([s.w.units[a.unit].name, a.command.name, a.start, a.end, a.command.ap_cost, a.command.stance_switch_cost])).collect();
        let mut steps = Vec::new();
        for _ in 0..600 {
            if s.phase == Phase::Replay {
                break;
            }
            s.step_next();
            steps.push(json!({"block": s.current_block, "phase": phase_int(s.phase), "units": units_state(&s)}));
        }
        rounds.push(json!({"round": s.current_round, "actions": actions, "steps": steps}));
        if s.combat_over || !s.can_advance_round() {
            break;
        }
        s.advance_round();
    }
    json!({"start": start, "rounds": rounds, "outcome": s.outcome})
}

fn first_divergence(g: &Value, r: &Value) -> Option<String> {
    if g["start"] != r["start"] {
        return Some(format!("start:\n  godot {}\n  rust  {}", g["start"], r["start"]));
    }
    let (gr, rr) = (g["rounds"].as_array().unwrap(), r["rounds"].as_array().unwrap());
    for (i, (a, b)) in gr.iter().zip(rr.iter()).enumerate() {
        if a["actions"] != b["actions"] {
            return Some(format!("round {} actions:\n  godot {}\n  rust  {}", i + 1, a["actions"], b["actions"]));
        }
        let (gs, rs) = (a["steps"].as_array().unwrap(), b["steps"].as_array().unwrap());
        for (j, (x, y)) in gs.iter().zip(rs.iter()).enumerate() {
            if x != y {
                let prev = if j > 0 { gs[j - 1].to_string() } else { "-".into() };
                return Some(format!("round {} step {}:\n  before {}\n  godot  {}\n  rust   {}", i + 1, j + 1, prev, x, y));
            }
        }
        if gs.len() != rs.len() {
            return Some(format!("round {} step count: godot {} rust {}", i + 1, gs.len(), rs.len()));
        }
    }
    if gr.len() != rr.len() {
        return Some(format!("round count: godot {} rust {}", gr.len(), rr.len()));
    }
    if g["result"]["outcome"] != r["outcome"] {
        return Some(format!("outcome: godot {} rust {}", g["result"]["outcome"], r["outcome"]));
    }
    None
}

#[test]
fn world_battles_replay_godot_step_for_step() {
    let dir = out_dir();
    let mut files: Vec<String> = std::fs::read_dir(&dir).expect("parity/out").filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).filter(|n| n.starts_with("traces_world_")).collect();
    files.sort();
    if files.is_empty() {
        eprintln!("no traces_world_*.json: run games/emberwake/parity/trace.sh mode=world");
        return;
    }
    let d = Data::load(&dir).expect("parity exports");
    let mut failures = Vec::new();
    let mut total = 0;
    for name in files {
        let t: Value = serde_json::from_str(&std::fs::read_to_string(dir.join(&name)).unwrap()).unwrap();
        for f in t["fights"].as_array().unwrap() {
            let fo = f.as_object().unwrap();
            total += 1;
            let r = run(&d, fo);
            if let Some(diff) = first_divergence(f, &r) {
                failures.push(format!("{name}: {}:{} {} seed {}: {diff}", json::s(fo, "world"), json::s(fo, "encounter"), json::get(fo, "override"), json::i(fo, "seed", 0)));
            }
        }
    }
    eprintln!("world fights checked: {total}, diverged: {}", failures.len());
    assert!(failures.is_empty(), "{} of {total} diverge; first:\n{}", failures.len(), failures.iter().take(3).cloned().collect::<Vec<_>>().join("\n"));
}
