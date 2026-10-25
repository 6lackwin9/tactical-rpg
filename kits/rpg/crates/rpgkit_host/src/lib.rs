//! The RPG kit's battle command set on the engine's command API protocol (`aegis_host`: one JSON request in, one JSON reply
//! out, history and step counter): the battle commands. A game's CLI serves it over stdin / stdout (`aegis serve`),
//! its battle screen and agent link call the same `Host::handle`, so a log of the requests replays a battle exactly.
//! The game's own CLI commands (`sim <mode>`, `battle`) live in the game's host crate.
//!
//! Requests (`cmd`; `history` is the protocol's own):
//!   new {encounter, seed?}                     start a catalog encounter, or a world one ("<world>:<id>", when the
//!                                              host has the world data)
//!       {ruleset?, rules?}                     RULESET1: classic or timeline (default: classic for a catalog
//!                                              encounter, the world's for a world one); rules "-id,+id" switches
//!                                              single extensions
//!       {plan_reveal?}                         RULESET1 V2: reveal every enemy plan (rule `plan_badges`; default false)
//!   state                                      round, block, phase, outcome, units, grid
//!   intents                                    every scheduled action this round (enemies included); under the rule
//!                                              `plan_badges` also `round_intents`: one record per active unit, both
//!                                              sides (round intent, target, Maneuver kind, impact badges, tell, plan)
//!   intel {unit}                               RULESET1 V5: the threat intel of one unit (its `round_intents` record;
//!                                              `plan_badges` only)
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
//!   round_intent {unit, intent?, target?, maneuver?, opener?}   RULESET1-PL (rule round_intent): set a party member's
//!                                              round intent, target ("self", a unit or [x, y]), Maneuver kind
//!                                              (engage, disengage, utility; "" = auto) and opener (a skill id,
//!                                              {"item": id}; "" = auto), then rebuild its auto plan and every later
//!                                              member's (planned, reason when the intent is not offered)
//!   manual {unit, on?}                         the member's Manual toggle (default on; off rebuilds the auto plan)
//!   fill {unit}                                Manual's "fill the rest": follow-ups and Rest after the last step
//!   round_plan                                 the planner's own state of every active party-side unit: intent,
//!                                              target, kind, opener, Manual and the reason tag of each planned entry
//!                                              (never an enemy: threat intel is `intents`' `round_intents`)
//!   log {since?}                               resolution log lines
//!   history                                    the mutating requests so far (feed them back to replay) and the
//!                                              checkpoints: the round outcome after every concluded round
//! Step shapes: {"mode": "engage", "skill", "target"}, {"mode": "self", "skill"}, {"mode": "ally", "skill",
//! "target"}, {"mode": "area", "skill", "cell": [x, y]}, {"mode": "goto", "cell"}, {"mode": "move", "dir": [dx, dy],
//! "tiles"}, {"mode": "withdraw"}, {"mode": "item", "item", "target"}. Units are referred to by index (as in `state`)
//! or by name.

use rpgkit_battle::command;
use rpgkit_battle::grid::{V2, GRID_H, GRID_W, NO_CELL, TERRAIN_IDS};
use rpgkit_battle::intent;
use rpgkit_battle::plan::{PlanStep, Step};
use rpgkit_battle::round_intent::{self, ManeuverKind, Opener, RoundTarget};
use rpgkit_battle::round_intent_plan as rip;
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_battle::skill::{self, Kind};
use rpgkit_battle::world::UnitId;
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::Ruleset;
use indexmap::IndexMap;
use serde_json::{json, Value};
use std::sync::Arc;

/// The battle command set: the catalog (and the worlds, for world encounters) and the battle in progress.
pub struct Battle {
    pub cat: Arc<Catalog>,
    pub data: Option<Arc<rpgkit_world::Data>>,
    pub session: Option<Session>,
    /// Rounds checkpointed so far (a checkpoint per concluded round, one more when the battle ends).
    checkpointed: i32,
    ended: bool,
}

/// The game's command host (the engine protocol over the battle command set; derefs to `Battle`).
pub type Host = aegis_host::Host<Battle>;

/// RULESET1 3.1.3: the ruleset a `new` request picks (`ruleset`: classic or timeline; absent or "" = `default`), with
/// its `rules` ("-id,+id": single extensions switched).
fn picked_ruleset(cat: &Catalog, req: &Value, default: Ruleset) -> Result<Ruleset, String> {
    let mut r = match req.get("ruleset").and_then(Value::as_str).filter(|s| !s.is_empty()) {
        Some(id) => Ruleset::of(cat, id)?,
        None => default,
    };
    if let Some(spec) = req.get("rules").and_then(Value::as_str) {
        r.adjust(spec)?;
    }
    Ok(r)
}

fn err(msg: impl Into<String>) -> Value {
    json!({"ok": false, "error": msg.into()})
}

impl aegis_host::Commands for Battle {
    fn mutating(&self, cmd: &str) -> bool {
        matches!(cmd, "new" | "preset" | "plan" | "add_step" | "undo" | "clear" | "confirm" | "step" | "resolve" | "advance" | "autoplay" | "round_intent" | "manual" | "fill")
    }

    fn restarts(&self, cmd: &str) -> bool {
        cmd == "new"
    }

    fn checkpoint(&mut self) -> Option<Value> {
        let s = self.session.as_ref()?;
        let concluded = s.phase == Phase::Replay && s.current_round > self.checkpointed;
        let ended = s.combat_over && !self.ended;
        if !concluded && !ended {
            return None;
        }
        self.checkpointed = self.checkpointed.max(s.current_round);
        self.ended = s.combat_over;
        Some(outcome_of(s))
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
        Self { cat, data: None, session: None, checkpointed: 0, ended: false }
    }

    /// A battle host that also starts world encounters ("<world>:<id>").
    pub fn with_data(data: Arc<rpgkit_world::Data>) -> Self {
        Self { cat: data.cat.clone(), data: Some(data), session: None, checkpointed: 0, ended: false }
    }

    /// Puts a battle started outside `new` in place (a party battle of the world map); the caller restarts the
    /// host's history.
    pub fn begin(&mut self, s: Session) {
        self.session = Some(s);
        self.checkpointed = 0;
        self.ended = false;
    }

    fn new_battle(&mut self, req: &Value) -> Value {
        let Some(id) = req.get("encounter").and_then(Value::as_str) else { return err("new needs an encounter id") };
        if let Some((world, local)) = id.split_once(':') {
            return self.new_world_battle(world, local, req);
        }
        let Some(entry) = self.cat.encounters.get(id) else {
            return err(format!("unknown encounter '{id}'; known: {}", self.cat.encounters.keys().cloned().collect::<Vec<_>>().join(", ")));
        };
        let mut def = EncounterDef::from_obj(entry);
        if let Some(seed) = req.get("seed").and_then(Value::as_i64) {
            def.rng_seed = seed;
        }
        // RULESET1 3.1.2: a catalog encounter is classic unless the request picks a ruleset
        match picked_ruleset(&self.cat, req, Ruleset::classic()) {
            Ok(r) => def.ruleset = r,
            Err(e) => return err(e),
        }
        def.plan_reveal = req.get("plan_reveal").and_then(Value::as_bool).unwrap_or(false);
        let s = Session::start(self.cat.clone(), def, &IndexMap::new());
        let state = state_of(&s);
        self.begin(s);
        json!({"ok": true, "state": state})
    }

    fn new_world_battle(&mut self, world: &str, local: &str, req: &Value) -> Value {
        let Some(d) = self.data.clone() else { return err("this host has no world data: world encounters need the GUI or a data folder") };
        let entry = d.resolve(world, "encounters", local);
        if entry.is_empty() {
            return err(format!("unknown encounter '{local}' in world '{world}'"));
        }
        let mut def = rpgkit_world::battle::encounter_def(&d, world, entry);
        if let Some(seed) = req.get("seed").and_then(Value::as_i64) {
            def.rng_seed = seed;
        }
        match picked_ruleset(&d.cat, req, def.ruleset.clone()) {
            Ok(r) => def.ruleset = r,
            Err(e) => return err(e),
        }
        // RULESET1 V3: the host has no game state, so the request decides (the world runner reads the flag)
        def.plan_reveal = req.get("plan_reveal").and_then(Value::as_bool).unwrap_or(false);
        def.id = format!("{world}:{local}");
        let roster = rpgkit_world::battle::WorldRoster { d: &d, world: world.to_string() };
        let s = Session::start(d.cat.clone(), def, &roster);
        let state = state_of(&s);
        self.begin(s);
        json!({"ok": true, "state": state, "intro": aegis_core::json::s(entry, "intro")})
    }

    fn battle_cmd(cat: &Catalog, s: &mut Session, cmd: &str, req: &Value) -> Value {
        match cmd {
            "state" => json!({"ok": true, "state": state_of(s)}),
            "intents" => {
                let mut reply = json!({"ok": true, "actions": scheduled(s)});
                // RULESET1 V5: both sides' round intents (classic keeps the reply's shape)
                if intent::badges_on(&s.w) {
                    reply["round_intents"] = Value::Array(s.round_intents().iter().map(|r| intel_json(s, r)).collect());
                }
                reply
            }
            "intel" => {
                if !intent::badges_on(&s.w) {
                    return err("intel needs the rule plan_badges");
                }
                let Some(u) = unit_arg(s, req, "unit") else { return err("intel needs a unit") };
                match s.round_intents().iter().find(|r| r.unit == u) {
                    Some(r) => json!({"ok": true, "intel": intel_json(s, r)}),
                    None => err(format!("{} is not on the field", s.w.units[u].name)),
                }
            }
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
            "round_intent" => round_intent_req(s, req),
            "manual" => manual_req(s, req),
            "fill" => fill_req(s, req),
            "round_plan" => json!({"ok": true, "round_plan": round_plan(s)}),
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

fn stance_name(u: &rpgkit_rules::Unit) -> &'static str {
    ["neutral", "offensive", "defensive", "tactical"][u.stance as usize]
}

/// The checkpoint after a request that concluded a round (or ended the battle): what a replay compares (PLAYTEST1 round
/// outcome): round, over, result (won, lost, fled, or timeout at the round limit; "" while it goes on), the log
/// length and every unit's HP, AP, tile, stance, statuses (active and pending), alive and withdrawn.
pub fn outcome_of(s: &Session) -> Value {
    let state = state_of(s);
    let keep = ["id", "hp", "ap", "pos", "stance", "statuses", "pending", "alive", "withdrawn"];
    let units: Vec<Value> = state["units"]
        .as_array()
        .map(|a| a.iter().map(|u| Value::Object(keep.iter().map(|k| (k.to_string(), u[*k].clone())).collect())).collect())
        .unwrap_or_default();
    let result = if !s.outcome.is_empty() { s.outcome.clone() } else if s.combat_over { "timeout".into() } else { String::new() };
    json!({"round": s.current_round, "over": s.combat_over, "result": result, "log_count": s.resolver.log.len(), "units": units})
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

/// RULESET1 V5, intent UI5: one unit's round intent record as JSON: {unit, name, side ("party" or "enemy"), intent
/// ("" = none), maneuver, target {kind ("", "self", "unit", "cell"), unit?, cell?}, impacts [[block, kind]], tell (the
/// first line), revealed, readers, plan (revealed lines)}; a line is {kind, target?, cell?, start, impact, recovery,
/// skill}.
fn intel_json(s: &Session, r: &intent::Intent) -> Value {
    let w = &s.w;
    let line = |l: &intent::Line| {
        let mut v = json!({"kind": l.kind, "start": l.blocks.0, "impact": l.blocks.1, "recovery": l.recovery, "skill": l.skill});
        if let Some(t) = l.target {
            v["target"] = json!(t);
        }
        if l.cell != NO_CELL {
            v["cell"] = json!([l.cell.x, l.cell.y]);
        }
        v
    };
    let target = match r.round_target {
        None => json!({"kind": ""}),
        Some(RoundTarget::Myself) => json!({"kind": "self", "unit": r.unit}),
        Some(RoundTarget::Unit(u)) => json!({"kind": "unit", "unit": u, "name": w.units[u].name}),
        Some(RoundTarget::Cell(p)) => json!({"kind": "cell", "cell": [p.x, p.y]}),
    };
    json!({
        "unit": r.unit,
        "name": w.units[r.unit].name,
        "side": if w.allies.contains(&r.unit) { "party" } else { "enemy" },
        "intent": r.round_intent.map_or("", |i| i.id()),
        "maneuver": r.maneuver.map_or("", intent::maneuver_id),
        "target": target,
        "impacts": r.impacts.iter().map(|(b, k)| json!([b, k])).collect::<Vec<_>>(),
        "tell": line(&r.line),
        "revealed": r.revealed,
        "readers": r.readers,
        "plan": r.plan.iter().map(line).collect::<Vec<_>>(),
    })
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
            out.push(json!({"skill": sk.id, "name": sk.name, "kind": sk.kind, "ap": if w.pays_ap(u) { skill::ap_cost_by(sk, Some(&w.units[u]), &w.ruleset) } else { 0 }, "blocks": sk.blocks, "reach": [reach.0, reach.1], "area": sk.area, "description": sk.raw.get("description")}));
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

// ---- RULESET1-PL: round intent requests (rule round_intent) ------------------------------------------------------

/// RULESET1_TASKS 1.8: a round target: "self", a unit (index or name) or a cell [x, y].
fn round_target_of(s: &Session, v: &Value) -> Option<RoundTarget> {
    match v {
        Value::String(t) if t == "self" => Some(RoundTarget::Myself),
        Value::Array(_) => cell_of(v).map(RoundTarget::Cell),
        Value::Number(n) => n.as_u64().map(|i| i as usize).filter(|i| *i < s.w.units.len()).map(RoundTarget::Unit),
        Value::String(name) => s.w.units.iter().position(|u| u.name.eq_ignore_ascii_case(name)).map(RoundTarget::Unit),
        _ => None,
    }
}

/// An opener: a skill id, {"skill": id} or {"item": id}; null or "" = the planner's (PL3).
fn opener_of(s: &Session, v: &Value) -> Result<Option<Opener>, String> {
    let skill = |id: &str| s.w.book().find(id).map(|i| Some(Opener::Skill(i))).ok_or_else(|| format!("unknown skill '{id}'"));
    match v {
        Value::Null => Ok(None),
        Value::String(id) if id.is_empty() => Ok(None),
        Value::String(id) => skill(id),
        Value::Object(o) => match (o.get("skill").and_then(Value::as_str), o.get("item").and_then(Value::as_str)) {
            (Some(id), _) => skill(id),
            (None, Some(id)) => Ok(Some(Opener::Item(id.to_string()))),
            _ => Err("opener needs a skill or an item".into()),
        },
        _ => Err("opener: a skill id, {\"skill\": id} or {\"item\": id}".into()),
    }
}

/// A Maneuver kind: engage, disengage, utility; "" or "auto" = the planner's (MV11).
fn kind_of(v: &Value) -> Result<Option<ManeuverKind>, String> {
    match v.as_str().unwrap_or("") {
        "" | "auto" => Ok(None),
        "engage" => Ok(Some(ManeuverKind::Engage)),
        "disengage" => Ok(Some(ManeuverKind::Disengage)),
        "utility" => Ok(Some(ManeuverKind::Utility)),
        other => Err(format!("unknown maneuver '{other}' (engage, disengage, utility)")),
    }
}

/// The party unit of a round intent request while the round is planned.
fn planning_member(s: &Session, req: &Value, cmd: &str) -> Result<UnitId, Value> {
    if s.phase != Phase::Planning || s.plan_confirmed {
        return Err(err("not planning"));
    }
    unit_arg(s, req, "unit").filter(|u| s.w.allies.contains(u)).ok_or_else(|| err(format!("{cmd} needs a party unit")))
}

/// The reply after a round intent change: whether the member's auto plan was built (else PL7's reason), the later
/// members the rebuild refused (PL8), the party plan and the party's planner state (`round_plan`).
fn intent_reply(s: &Session, result: Result<(), String>, refused: Vec<(UnitId, String)>) -> Value {
    let refused: Vec<Value> = refused.into_iter().map(|(u, r)| json!({"unit": u, "name": s.w.units[u].name, "reason": r})).collect();
    json!({"ok": true, "planned": result.is_ok(), "reason": result.err(), "refused": refused, "plan": plan_view(s, None), "round_plan": round_plan(s)})
}

/// RULESET1_TASKS 1.8 `round_intent {unit, intent, target, maneuver, opener}`: each given field is set in order (the
/// intent first: a change drops intent-bound buffs and the old steps), then a member that plans automatically gets its
/// plan rebuilt, and every later member's (PL8). Atomic: on any refused field the battle is left exactly as it was
/// (the protocol records only successful requests, so a replay must never see half a request).
fn round_intent_req(s: &mut Session, req: &Value) -> Value {
    let u = match planning_member(s, req, "round_intent") {
        Ok(u) => u,
        Err(e) => return e,
    };
    let (w0, steps0, events0) = (s.w.clone(), s.plan.sequences.clone(), s.events.len());
    if let Err(e) = set_round_fields(s, u, req) {
        s.w = w0;
        s.plan.sequences = steps0;
        s.events.truncate(events0);
        return err(e);
    }
    if rip::manual(&s.w, u) {
        return intent_reply(s, Ok(()), Vec::new());
    }
    let (result, refused) = rip::replan(&mut s.w, &mut s.plan, u);
    intent_reply(s, result, refused)
}

/// The fields of a `round_intent` request, in order; the first refusal stops (the caller restores the battle).
fn set_round_fields(s: &mut Session, u: UnitId, req: &Value) -> Result<(), String> {
    if let Some(i) = req.get("intent").and_then(Value::as_str) {
        s.set_round_intent(u, i)?;
    }
    if let Some(v) = req.get("target") {
        let t = round_target_of(s, v).ok_or("target: \"self\", a unit or a cell [x, y]")?;
        s.set_round_target(u, t)?;
    }
    if let Some(v) = req.get("maneuver") {
        kind_of(v).and_then(|k| rip::set_kind(&mut s.w, u, k))?;
    }
    if let Some(v) = req.get("opener") {
        opener_of(s, v).and_then(|o| rip::set_opener(&mut s.w, u, o))?;
    }
    Ok(())
}

/// MM1, MM3 `manual {unit, on}`: the Manual toggle (default on); off rebuilds the auto plan.
fn manual_req(s: &mut Session, req: &Value) -> Value {
    let u = match planning_member(s, req, "manual") {
        Ok(u) => u,
        Err(e) => return e,
    };
    let on = req.get("on").and_then(Value::as_bool).unwrap_or(true);
    match rip::set_manual(&mut s.w, &mut s.plan, u, on) {
        Err(e) if on => err(e),
        result => intent_reply(s, result, Vec::new()),
    }
}

/// MM3 `fill {unit}`: follow-ups and Rest after the member's last step.
fn fill_req(s: &mut Session, req: &Value) -> Value {
    let u = match planning_member(s, req, "fill") {
        Ok(u) => u,
        Err(e) => return e,
    };
    match rip::fill_rest(&s.w, &mut s.plan, u) {
        Err(e) => err(e),
        Ok(()) => intent_reply(s, Ok(()), Vec::new()),
    }
}

/// RULESET1_TASKS 1.8 `round_plan`: the planner's own state of every active party-side unit (round intent, target,
/// Maneuver kind, opener, Manual toggle and the reason tag of each entry it has planned, PL6); never an enemy (its
/// opener and target are not shown: the threat intel view is `intents`' `round_intents`, reveals respected); empty
/// without the rule round_intent.
fn round_plan(s: &Session) -> Vec<Value> {
    let w = &s.w;
    if !round_intent::on(w) {
        return Vec::new();
    }
    w.allies
        .iter()
        .copied()
        .filter(|&c| w.units[c].is_active())
        .map(|c| {
            let rs = w.round_states.get(&c).cloned().unwrap_or_default();
            let target = match rs.target {
                Some(RoundTarget::Myself) => json!("self"),
                Some(RoundTarget::Unit(t)) => json!(t),
                Some(RoundTarget::Cell(p)) => json!([p.x, p.y]),
                None => Value::Null,
            };
            let opener = match &rs.opener {
                Some(Opener::Skill(i)) => json!(w.cat.skills[*i].id),
                Some(Opener::Item(id)) => json!({"item": id}),
                None => Value::Null,
            };
            json!({"unit": c, "name": w.units[c].name, "side": "party",
                "intent": rs.intent.map(|i| i.id()), "target": target, "maneuver": rs.maneuver.map(intent::maneuver_id), "maneuver_auto": rs.maneuver_auto,
                "opener": opener, "manual": rs.manual, "reasons": rip::reasons(w, &s.plan, c)})
        })
        .collect()
}
