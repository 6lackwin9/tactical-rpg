//! Emberwake's command set on the engine's command API protocol (`aegis_host`: one JSON request in, one JSON reply
//! out, history and step counter): the battle commands. The CLI serves it over stdin / stdout (`aegis serve`), the
//! GUI's battle screen and agent link call the same `Host::handle`, so a log of the requests replays a battle exactly.
//! Also the game's own CLI commands (`cli`: `sim duel`, `battle`).
//!
//! Requests (`cmd`; `history` is the protocol's own):
//!   new {encounter, seed?}                     start a catalog encounter
//!   state                                      round, block, phase, outcome, units, grid
//!   intents                                    every scheduled action this round (enemies included)
//!   skills {unit}                              the skills the unit can plan now
//!   items {unit, target?}                      battle-pouch items it can use (on target: CombatPlan.item_options)
//!   preset {preset, unit?}                     AI-draft one ally (or every ally) with a PlanPresets tactic
//!   plan {unit, steps: [...]}                  replace an ally's steps; reports each step's problem
//!   plan_view {unit?}                          compiled timeline per ally (blocks, AP, targets, est. damage)
//!   probe {unit, step}                         would this step fit next? (valid, reason, entries it adds); no change
//!   add_step {unit, step} / undo {unit} / clear {unit}   edit an ally's steps one at a time
//!   confirm                                    lock the party plan and schedule it
//!   step                                       resolve one action (or block wrap-up)
//!   resolve                                    resolve to the end of the round
//!   advance                                    start the next round
//!   autoplay {preset?, rounds?}                draft, confirm and resolve rounds until the battle ends
//!   log {since?}                               resolution log lines
//!   history                                    the mutating requests so far (feed them back to replay)
//! Step shapes: {"mode": "engage", "skill", "target"}, {"mode": "self", "skill"}, {"mode": "ally", "skill",
//! "target"}, {"mode": "area", "skill", "cell": [x, y]}, {"mode": "goto", "cell"}, {"mode": "move", "dir": [dx, dy],
//! "tiles"}, {"mode": "withdraw"}, {"mode": "item", "item", "target"}. Units are referred to by index (as in `state`)
//! or by name.

use emberwake_battle::command;
use emberwake_battle::grid::{V2, GRID_H, GRID_W, TERRAIN_IDS};
use emberwake_battle::plan::{PlanStep, Step};
use emberwake_battle::session::{EncounterDef, Phase, Session};
use emberwake_battle::skill::{self, Kind};
use emberwake_battle::world::UnitId;
use emberwake_data::Catalog;
use indexmap::IndexMap;
use serde_json::{json, Value};
use std::sync::Arc;

pub mod cli;

/// The battle command set: the catalog and the battle in progress.
pub struct Battle {
    pub cat: Arc<Catalog>,
    pub session: Option<Session>,
}

/// The game's command host (the engine protocol over the battle command set; derefs to `Battle`).
pub type Host = aegis_host::Host<Battle>;

fn err(msg: impl Into<String>) -> Value {
    json!({"ok": false, "error": msg.into()})
}

impl aegis_host::Commands for Battle {
    fn mutating(&self, cmd: &str) -> bool {
        matches!(cmd, "new" | "preset" | "plan" | "add_step" | "undo" | "clear" | "confirm" | "step" | "resolve" | "advance" | "autoplay")
    }

    fn restarts(&self, cmd: &str) -> bool {
        cmd == "new"
    }

    fn handle(&mut self, cmd: &str, req: &Value) -> Value {
        match cmd {
            "new" => self.new_battle(req),
            _ => match self.session.as_mut() {
                None => err("no battle: send {\"cmd\": \"new\", \"encounter\": ...} first"),
                Some(s) => Self::battle_cmd(&self.cat, s, cmd, req),
            },
        }
    }
}

impl Battle {
    pub fn new(cat: Arc<Catalog>) -> Self {
        Self { cat, session: None }
    }

    fn new_battle(&mut self, req: &Value) -> Value {
        let Some(id) = req.get("encounter").and_then(Value::as_str) else { return err("new needs an encounter id") };
        let Some(entry) = self.cat.encounters.get(id) else {
            return err(format!("unknown encounter '{id}'; known: {}", self.cat.encounters.keys().cloned().collect::<Vec<_>>().join(", ")));
        };
        let mut def = EncounterDef::from_obj(entry);
        if let Some(seed) = req.get("seed").and_then(Value::as_i64) {
            def.rng_seed = seed;
        }
        let s = Session::start(self.cat.clone(), def, &IndexMap::new());
        let state = state_of(&s);
        self.session = Some(s);
        json!({"ok": true, "state": state})
    }

    fn battle_cmd(cat: &Catalog, s: &mut Session, cmd: &str, req: &Value) -> Value {
        match cmd {
            "state" => json!({"ok": true, "state": state_of(s)}),
            "intents" => json!({"ok": true, "actions": scheduled(s)}),
            "skills" => match unit_arg(s, req, "unit") {
                Some(u) => json!({"ok": true, "skills": skills_of(cat, s, u)}),
                None => err("skills needs a unit"),
            },
            "preset" => {
                if s.phase != Phase::Planning {
                    return err("not planning");
                }
                let preset = req.get("preset").and_then(Value::as_str).unwrap_or("balanced").to_string();
                match unit_arg(s, req, "unit") {
                    Some(u) if s.w.allies.contains(&u) => s.plan_apply_preset(u, &preset),
                    Some(_) => return err("preset applies to party units only"),
                    None => s.plan_apply_preset_all(&preset),
                }
                json!({"ok": true, "plan": plan_view(s, None)})
            }
            "items" => match unit_arg(s, req, "unit") {
                Some(u) => json!({"ok": true, "items": items_of(s, u, unit_arg(s, req, "target"))}),
                None => err("items needs a unit"),
            },
            "plan" => set_plan(s, req),
            "probe" => probe(s, req),
            "add_step" => edit_steps(s, req, "add"),
            "undo" => edit_steps(s, req, "undo"),
            "clear" => edit_steps(s, req, "clear"),
            "plan_view" => json!({"ok": true, "plan": plan_view(s, unit_arg(s, req, "unit"))}),
            "confirm" => {
                if s.confirm_plan() {
                    json!({"ok": true, "actions": scheduled(s)})
                } else {
                    err("plan refused: fix the invalid steps (see plan_view)")
                }
            }
            "step" => {
                if s.phase == Phase::Planning && !s.plan_confirmed && !s.confirm_plan() {
                    return err("plan refused");
                }
                let from = s.resolver.log.len();
                s.step_next();
                json!({"ok": true, "log": s.resolver.log[from..].to_vec(), "state": state_of(s)})
            }
            "resolve" => {
                if s.phase == Phase::Planning && !s.plan_confirmed && !s.confirm_plan() {
                    return err("plan refused");
                }
                let from = s.resolver.log.len();
                while s.phase != Phase::Replay {
                    s.step_next();
                }
                json!({"ok": true, "log": s.resolver.log[from..].to_vec(), "state": state_of(s)})
            }
            "advance" => {
                if !s.can_advance_round() {
                    return err(if s.combat_over { "the battle is over" } else { "finish the round first (resolve)" });
                }
                s.advance_round();
                json!({"ok": true, "state": state_of(s)})
            }
            "autoplay" => {
                let preset = req.get("preset").and_then(Value::as_str).unwrap_or("balanced").to_string();
                let limit = req.get("rounds").and_then(Value::as_i64).unwrap_or(99);
                let from = s.resolver.log.len();
                let mut played = 0;
                while played < limit {
                    if s.phase == Phase::Replay {
                        if !s.can_advance_round() {
                            break;
                        }
                        s.advance_round();
                    }
                    if !s.plan_confirmed {
                        s.plan_apply_preset_all(&preset);
                        if !s.confirm_plan() {
                            return err("autoplay: the party plan was refused");
                        }
                    }
                    while s.phase != Phase::Replay {
                        s.step_next();
                    }
                    played += 1;
                }
                json!({"ok": true, "rounds": played, "log": s.resolver.log[from..].to_vec(), "state": state_of(s)})
            }
            "log" => {
                let since = req.get("since").and_then(Value::as_u64).unwrap_or(0) as usize;
                json!({"ok": true, "log": s.resolver.log.get(since..).unwrap_or(&[]).to_vec(), "next": s.resolver.log.len()})
            }
            _ => err(format!("unknown cmd '{cmd}'")),
        }
    }
}

fn unit_arg(s: &Session, req: &Value, key: &str) -> Option<UnitId> {
    match req.get(key)? {
        Value::Number(n) => n.as_u64().map(|i| i as usize).filter(|i| *i < s.w.units.len()),
        Value::String(name) => s.w.units.iter().position(|u| u.name.eq_ignore_ascii_case(name)),
        _ => None,
    }
}

fn cell_of(v: &Value) -> Option<V2> {
    let a = v.as_array()?;
    Some(V2::new(a.first()?.as_i64()? as i32, a.get(1)?.as_i64()? as i32))
}

fn phase_name(p: Phase) -> &'static str {
    match p {
        Phase::Planning => "planning",
        Phase::Resolving => "resolving",
        Phase::Replay => "round_over",
    }
}

fn stance_name(u: &emberwake_rules::Unit) -> &'static str {
    ["neutral", "offensive", "defensive", "tactical"][u.stance as usize]
}

/// The battle as the agent sees it: everything the GUI would draw, as data.
pub fn state_of(s: &Session) -> Value {
    let w = &s.w;
    let units: Vec<Value> = (0..w.units.len())
        .map(|i| {
            let u = &w.units[i];
            let p = w.grid.pos(i);
            json!({
                "id": i, "name": u.name, "side": if w.allies.contains(&i) { "party" } else { "enemy" },
                "hp": u.hp, "max_hp": u.max_hp, "ap": u.ap, "max_ap": u.max_ap, "speed": u.speed, "range": u.attack_range,
                "pos": p.map(|p| json!([p.x, p.y])), "stance": stance_name(u), "level": u.stats.level, "tier": u.tier,
                "statuses": u.st.active.iter().collect::<Vec<_>>(), "pending": u.st.pending.iter().collect::<Vec<_>>(),
                "alive": u.is_alive(), "withdrawn": u.is_withdrawn, "target": u.active_target,
            })
        })
        .collect();
    let rows: Vec<String> = (0..GRID_H)
        .map(|y| (0..GRID_W).map(|x| TERRAIN_IDS.iter().position(|t| *t == w.grid.terrain_id(V2::new(x, y))).map(|i| ".,:os*~WmL#T=rfpi".chars().nth(i).unwrap()).unwrap_or('.')).collect())
        .collect();
    json!({
        "encounter": s.def.id, "round": s.current_round, "max_rounds": s.def.max_rounds, "block": s.current_block,
        "blocks": s.def.total_blocks, "phase": phase_name(s.phase), "plan_confirmed": s.plan_confirmed,
        "outcome": s.outcome, "over": s.combat_over, "units": units,
        "grid": {"w": GRID_W, "h": GRID_H, "rows": rows, "exits": w.grid.exits.iter().map(|e| json!([e.x, e.y])).collect::<Vec<_>>()},
    })
}

fn scheduled(s: &Session) -> Vec<Value> {
    s.resolver
        .scheduled
        .iter()
        .map(|a| json!({"unit": a.unit, "name": s.w.units[a.unit].name, "command": a.command.name, "start": a.start, "end": a.end, "ap": a.command.ap_cost + a.command.stance_switch_cost, "cancelled": a.cancelled}))
        .collect()
}

/// ITEMUSE1 rule 2: one row per distinct offered pouch item with copies left this round; with a target, only the items
/// that may be aimed at it (CombatPlan.item_options).
fn items_of(s: &Session, u: UnitId, target: Option<UnitId>) -> Vec<Value> {
    let mut out = Vec::new();
    for id in command::pouch_ids(&s.w, u) {
        let con = command::pouch_item(&s.w, u, &id).unwrap();
        let left = s.plan.items_left(&s.w, u, &id, -1);
        if left <= 0 || !con.offered_in_battle() || target.is_some_and(|t| !command::item_side_problem(&s.w, u, con, Some(t)).is_empty()) {
            continue;
        }
        out.push(json!({"item": id, "name": con.name, "left": left, "heal": con.heal, "damage": con.damage, "buff_pct": con.buff_pct, "rounds": con.duration,
            "reach": con.battle_reach(), "blocks": con.use_blocks.max(1), "ap": command::ITEM_AP, "at_foes": con.aims_at_foes(), "description": con.raw.get("description")}));
    }
    out
}

fn skills_of(cat: &Catalog, s: &Session, u: UnitId) -> Vec<Value> {
    let w = &s.w;
    let cap = w.skill_cap_of(u);
    let mut out = Vec::new();
    for k in Kind::ALL {
        for sid in w.book().available(k, cap, Some(&w.units[u])) {
            let sk = &cat.skills[sid];
            if sk.system && !matches!(k, Kind::Move | Kind::Rest) {
                continue;
            }
            let reach = if k == Kind::Support { skill::support_reach(sk, &w.units[u]) } else { skill::reach_for(sk, &w.units[u]) };
            out.push(json!({"skill": sk.id, "name": sk.name, "kind": sk.kind, "ap": skill::ap_cost(sk, Some(&w.units[u])), "blocks": sk.blocks, "reach": [reach.0, reach.1], "area": sk.area, "description": sk.raw.get("description")}));
        }
    }
    out
}

fn step_from(s: &Session, v: &Value) -> Result<Step, String> {
    let mode = v.get("mode").and_then(Value::as_str).unwrap_or("");
    let book = s.w.book();
    let skill_id = || -> Result<usize, String> {
        let id = v.get("skill").and_then(Value::as_str).ok_or("step needs a skill")?;
        book.find(id).ok_or(format!("unknown skill '{id}'"))
    };
    let target = || unit_arg(s, v, "target").ok_or_else(|| "step needs a target unit".to_string());
    let cell = || v.get("cell").and_then(cell_of).ok_or_else(|| "step needs a cell [x, y]".to_string());
    Ok(match mode {
        "engage" => PlanStep::engage(skill_id()?, target()?),
        "self" => PlanStep::self_skill(skill_id()?),
        "ally" => PlanStep::ally(skill_id()?, target()?),
        "area" => PlanStep::area_at(skill_id()?, cell()?),
        "goto" => PlanStep::go_to(book.lowest(Kind::Move), cell()?),
        "move" => {
            let d = v.get("dir").and_then(cell_of).ok_or("move needs dir [dx, dy]")?;
            PlanStep::mv(book.lowest(Kind::Move), d, v.get("tiles").and_then(Value::as_i64).unwrap_or(1) as i32)
        }
        "withdraw" => PlanStep::withdraw(),
        "item" => PlanStep::use_item(v.get("item").and_then(Value::as_str).ok_or("item step needs an item id")?, target()?),
        _ => return Err(format!("unknown step mode '{mode}'")),
    })
}

fn set_plan(s: &mut Session, req: &Value) -> Value {
    if s.phase != Phase::Planning || s.plan_confirmed {
        return err("not planning");
    }
    let Some(u) = unit_arg(s, req, "unit").filter(|u| s.w.allies.contains(u)) else { return err("plan needs a party unit") };
    let steps = req.get("steps").and_then(Value::as_array).cloned().unwrap_or_default();
    s.plan.set_steps(u, Vec::new());
    let mut results = Vec::new();
    for v in &steps {
        let r = match step_from(s, v) {
            Err(e) => e,
            Ok(step) => s.plan.add_step(&s.w, u, step, true),
        };
        results.push(if r.is_empty() { json!({"ok": true}) } else { json!({"ok": false, "error": r}) });
    }
    json!({"ok": true, "steps": results, "plan": plan_view(s, Some(u))})
}

fn plan_view(s: &Session, only: Option<UnitId>) -> Value {
    let w = &s.w;
    let mut out = serde_json::Map::new();
    for &a in &w.allies {
        if only.is_some_and(|o| o != a) || !w.units[a].is_active() {
            continue;
        }
        let st = s.plan.compile(w, a);
        let entries: Vec<Value> = st
            .entries
            .iter()
            .map(|e| {
                json!({"skill": w.cat.skills[e.skill].id, "name": w.cat.skills[e.skill].name, "start": e.start_block, "end": e.end_block(w), "ap": e.total_ap(),
                    "target": e.target.map(|t| w.units[t].name.clone()), "to": [e.pos_after.x, e.pos_after.y], "est_damage": e.est_damage, "est_heal": e.est_heal})
            })
            .collect();
        out.insert(
            w.units[a].name.clone(),
            json!({"unit": a, "valid": st.error_index < 0, "error": if st.error_index >= 0 { Value::String(st.error_text.clone()) } else { Value::Null }, "ap_left": st.ap_left, "free_blocks": st.blocks_left(), "entries": entries}),
        );
    }
    Value::Object(out)
}

/// The step as the next input of `unit`, without changing the plan: {valid, reason, ap, blocks, entries}.
fn probe(s: &mut Session, req: &Value) -> Value {
    let Some(u) = unit_arg(s, req, "unit").filter(|u| s.w.allies.contains(u)) else { return err("probe needs a party unit") };
    let step = match req.get("step").map(|v| step_from(s, v)) {
        Some(Ok(st)) => st,
        Some(Err(e)) => return err(e),
        None => return err("probe needs a step"),
    };
    let had_entry = s.plan.sequences.contains_key(&u);
    let before = s.plan.steps_of(u);
    let base = s.plan.compile(&s.w, u).entries.len();
    let problem = s.plan.add_step(&s.w, u, step, true);
    let result = if problem.is_empty() {
        let st = s.plan.compile(&s.w, u);
        let added: Vec<Value> = st.entries[base..]
            .iter()
            .map(|e| json!({"skill": s.w.cat.skills[e.skill].id, "start": e.start_block, "end": e.end_block(&s.w), "ap": e.total_ap(), "est_damage": e.est_damage, "est_heal": e.est_heal, "to": [e.pos_after.x, e.pos_after.y]}))
            .collect();
        let ap: i32 = st.entries[base..].iter().map(|e| e.total_ap()).sum();
        json!({"ok": true, "valid": true, "ap": ap, "ap_left": st.ap_left, "entries": added})
    } else {
        json!({"ok": true, "valid": false, "reason": problem})
    };
    if had_entry {
        s.plan.set_steps(u, before);
    } else {
        s.plan.clear(u);
    }
    result
}

fn edit_steps(s: &mut Session, req: &Value, op: &str) -> Value {
    if s.phase != Phase::Planning || s.plan_confirmed {
        return err("not planning");
    }
    let Some(u) = unit_arg(s, req, "unit").filter(|u| s.w.allies.contains(u)) else { return err("needs a party unit") };
    match op {
        "add" => {
            let step = match req.get("step").map(|v| step_from(s, v)) {
                Some(Ok(st)) => st,
                Some(Err(e)) => return err(e),
                None => return err("add_step needs a step"),
            };
            let problem = s.plan.add_step(&s.w, u, step, true);
            if !problem.is_empty() {
                return err(problem);
            }
        }
        "undo" => {
            let n = s.plan.steps_of(u).len();
            if n == 0 {
                return err("nothing to undo");
            }
            s.plan.remove_step(u, n - 1);
        }
        _ => s.plan.set_steps(u, Vec::new()),
    }
    json!({"ok": true, "plan": plan_view(s, Some(u))})
}
