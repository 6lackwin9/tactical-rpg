//! CombatSession + CatalogEncounter (scripts/combat/session/): one battle's round flow. Enemies are drafted by the
//! AI when a round begins; the party is drafted by a preset (or edited) and committed on confirm; then blocks
//! resolve action by action. Covered so far: catalog encounters with the rout objective and standard deployment
//! (no reinforcement waves, boss phases or morale yet).

use crate::ai;
use crate::grid::{Grid, TerrainTable, V2};
use crate::plan::Plan;
use crate::resolver::Resolver;
use crate::world::{AiProfile, UnitId, World};
use aegis_data::value::{self, Obj};
use aegis_data::{Catalog, UnitDef};
use aegis_rules::status;
use aegis_rules::build_unit;
use indexmap::IndexMap;
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Planning,
    Resolving,
    Replay,
}

#[derive(Debug, Clone)]
pub struct EncounterDef {
    pub id: String,
    pub name: String,
    pub zone: String,
    pub open: bool,
    pub exits: Vec<V2>,
    pub max_rounds: i32,
    pub total_blocks: i32,
    pub ap_regen: i32,
    pub unit_ap: i32,
    pub rng_seed: i64,
    pub party: Vec<String>,
    pub enemies: Vec<String>,
    pub deployment: String,
    pub objective: String,
    /// One reinforcement wave: off the field until it enters before `wave_block` of `wave_round`.
    pub wave_units: Vec<String>,
    pub wave_round: i32,
    pub wave_block: i32,
}

impl EncounterDef {
    /// ContentSchema "encounters" defaults applied to a (possibly partial) entry.
    pub fn from_obj(o: &Obj) -> Self {
        let exits = match o.get("exits") {
            Some(Value::Array(a)) => a.iter().filter_map(|p| p.as_array()).filter(|p| p.len() == 2).map(|p| V2::new(value::to_int(&p[0], 0), value::to_int(&p[1], 0))).collect(),
            _ => vec![V2::new(11, 4)],
        };
        let deployment = value::string(o, "deployment", "standard");
        Self {
            id: value::string(o, "id", ""),
            name: value::string(o, "name", ""),
            zone: value::string(o, "zone", "stone_arena"),
            open: value::string(o, "arena", "CLOSED") == "OPEN",
            exits,
            max_rounds: value::int(o, "max_rounds", 6),
            total_blocks: value::int(o, "total_blocks", 6),
            ap_regen: value::int(o, "ap_regen", 4),
            unit_ap: value::int(o, "unit_ap", 12),
            rng_seed: o.get("rng_seed").and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64))).unwrap_or(1337),
            party: value::strings(o, "party"),
            enemies: value::strings(o, "enemies"),
            deployment: if ["standard", "ambush", "centre"].contains(&deployment.as_str()) { deployment } else { "standard".into() },
            objective: value::string(o, "objective", "rout"),
            wave_units: value::strings(o, "wave_units"),
            wave_round: value::int(o, "wave_round", 1),
            wave_block: value::int(o, "wave_block", 4),
        }
    }
}

pub struct Session {
    pub def: EncounterDef,
    pub w: World,
    pub resolver: Resolver,
    pub plan: Plan,
    pub current_round: i32,
    pub current_block: i32,
    pub phase: Phase,
    pub combat_over: bool,
    /// "" while the battle goes on, then "won", "lost" or "fled".
    pub outcome: String,
    pub plan_confirmed: bool,
    pub wave: Vec<UnitId>,
    pub reinforcements_spawned: bool,
    /// Art key per unit (ContentFactory.art_of: the unit's own art, else its class's, else "unit").
    pub art: Vec<String>,
    pending: VecDeque<usize>,
}

/// A unit built outside the encounter (GameEncounter: a saved party member through PartyBuilder).
pub struct Prebuilt {
    pub unit: aegis_rules::Unit,
    pub art: String,
    pub profile: AiProfile,
}

impl Session {
    /// CatalogEncounter.build + CombatSession.start: spawn, deploy, begin round 1. `units` resolves roster ids.
    pub fn start(cat: Arc<Catalog>, def: EncounterDef, units: &IndexMap<String, UnitDef>) -> Self {
        Self::start_with(cat, def, units, Vec::new())
    }

    /// `start` with `party` (non-empty) fighting instead of the encounter's own party roster (GameEncounter).
    pub fn start_with(cat: Arc<Catalog>, def: EncounterDef, units: &IndexMap<String, UnitDef>, party: Vec<Prebuilt>) -> Self {
        let lookup = |id: &str| units.get(id).cloned().or_else(|| cat.units.get(id).cloned());
        let party_ids: Vec<String> = if party.is_empty() { def.party.iter().filter(|id| lookup(id).is_some()).cloned().collect() } else { Vec::new() };
        let enemy_ids: Vec<String> = def.enemies.iter().filter(|id| lookup(id).is_some()).cloned().collect();
        let wave_ids: Vec<String> = def.wave_units.iter().filter(|id| lookup(id).is_some()).cloned().collect();
        // _display_names: repeated base names get " 2", " 3" ...
        let mut seen: IndexMap<String, i32> = IndexMap::new();
        let mut names = Vec::new();
        for id in party_ids.iter().chain(enemy_ids.iter()).chain(wave_ids.iter()) {
            let base = lookup(id).map(|e| e.name).unwrap_or_else(|| id.clone());
            let n = seen.get(&base).copied().unwrap_or(0) + 1;
            seen.insert(base.clone(), n);
            names.push(if n == 1 { base } else { format!("{base} {n}") });
        }
        let mut built = Vec::new();
        let mut profiles = IndexMap::new();
        let mut art = Vec::new();
        let n_prebuilt = party.len();
        for p in party {
            profiles.insert(built.len(), p.profile);
            art.push(p.art);
            built.push(p.unit);
        }
        for (i, id) in party_ids.iter().chain(enemy_ids.iter()).chain(wave_ids.iter()).enumerate() {
            let mut e = lookup(id).unwrap();
            e.name = names[i].clone();
            profiles.insert(built.len(), AiProfile::for_unit(&cat, &e));
            built.push(build_unit(&cat, &e, def.unit_ap));
            art.push(if !e.art.is_empty() { e.art.clone() } else { cat.classes.get(&e.class_id).map(|c| c.art.clone()).unwrap_or_else(|| "unit".into()) });
        }
        let n_party = n_prebuilt + party_ids.len();
        let n_deployed = n_party + enemy_ids.len();
        let count = built.len();
        let mut grid = Grid::new(def.open, def.exits.clone(), TerrainTable::from_catalog(&cat), count);
        grid.apply_zone(&def.zone);
        let mut w = World { cat, units: built, grid, allies: (0..n_party).collect(), enemies: (n_party..count).collect(), profiles, total_blocks: def.total_blocks };
        let allies = w.allies.clone();
        let enemies = w.enemies.clone();
        let deployed: Vec<UnitId> = (n_party..n_deployed).collect();
        let wave: Vec<UnitId> = (n_deployed..count).collect();
        w.deploy(&allies, &deployed, &def.deployment);
        w.place_leftovers(&allies, true, &def.deployment);
        w.place_leftovers(&deployed, false, &def.deployment);
        let mut resolver = Resolver::new(def.total_blocks, def.rng_seed);
        for &c in allies.iter().chain(enemies.iter()) {
            resolver.register(c);
        }
        for u in w.units.iter_mut() {
            u.loadout_locked = true;
        }
        let mut s = Session { def, w, resolver, plan: Plan::new(), current_round: 1, current_block: 1, phase: Phase::Planning, combat_over: false, outcome: String::new(), plan_confirmed: false, wave, reinforcements_spawned: false, art, pending: VecDeque::new() };
        s.begin_round(1);
        s
    }

    fn begin_round(&mut self, round: i32) {
        if round > 1 {
            self.resolver.finish_round(&mut self.w);
        }
        for &c in &self.resolver.registered.clone() {
            status::roll_over(&mut self.w.units[c], self.def.ap_regen);
        }
        self.current_round = round;
        self.current_block = 1;
        self.phase = Phase::Planning;
        self.pending.clear();
        self.plan = Plan::new();
        self.plan_confirmed = false;
        self.schedule_enemy_ai();
    }

    /// Re-plans the round's enemy side (after the start positions changed, e.g. the sim's jitter).
    pub fn replan_enemies(&mut self) {
        self.resolver.scheduled.clear();
        self.plan = Plan::new();
        self.schedule_enemy_ai();
    }

    /// CombatEncounter.schedule_enemy_ai: drafts every enemy on the field into one shared plan, re-drafts broken
    /// drafts, trims what is still broken, and commits.
    fn schedule_enemy_ai(&mut self) {
        let w = &self.w;
        let mut enemy_plan = Plan::new();
        let drafted: Vec<UnitId> = w.enemies.iter().copied().filter(|e| w.units[*e].is_active() && w.has(*e)).collect();
        for &e in &drafted {
            ai::build_steps(w, &mut enemy_plan, e, &w.profile_of(e));
        }
        for _ in 0..drafted.len() {
            let mut redrafted = false;
            for &e in &drafted {
                if draft_broken(w, &enemy_plan, e) {
                    ai::build_steps(w, &mut enemy_plan, e, &w.profile_of(e));
                    redrafted = true;
                }
            }
            if !redrafted {
                break;
            }
        }
        let mut trimmed = true;
        while trimmed {
            trimmed = false;
            for &e in &drafted {
                if draft_broken(w, &enemy_plan, e) {
                    let st = enemy_plan.compile(w, e);
                    let keep = if st.error_index >= 0 { st.error_index as usize } else { 0 };
                    let steps = enemy_plan.steps_of(e);
                    enemy_plan.set_steps(e, steps[..keep].to_vec());
                    trimmed = true;
                }
            }
        }
        enemy_plan.commit(w, &mut self.resolver);
    }

    /// plan_apply_preset_all: every active ally drafted with `preset`, then broken drafts re-drafted.
    pub fn plan_apply_preset_all(&mut self, preset: &str) {
        if self.phase != Phase::Planning {
            return;
        }
        let allies = self.w.allies.clone();
        for &a in &allies {
            if self.w.units[a].is_active() {
                let p = self.w.profile_of(a).with_preset(preset);
                ai::build_steps(&self.w, &mut self.plan, a, &p);
            }
        }
        for _ in 0..allies.len() {
            let mut redrafted = false;
            for &a in &allies {
                if self.w.units[a].is_active() && !self.plan.is_valid(&self.w, a) {
                    let p = self.w.profile_of(a).with_preset(preset);
                    ai::build_steps(&self.w, &mut self.plan, a, &p);
                    redrafted = true;
                }
            }
            if !redrafted {
                break;
            }
        }
    }

    pub fn plan_is_valid(&self) -> bool {
        self.w.allies.iter().all(|&a| !self.w.units[a].is_active() || self.plan.is_valid(&self.w, a))
    }

    pub fn confirm_plan(&mut self) -> bool {
        if self.phase != Phase::Planning || !self.plan_is_valid() {
            return false;
        }
        self.plan.commit(&self.w, &mut self.resolver);
        self.plan_confirmed = true;
        true
    }

    /// One resolution step: one action, or a block with nothing to resolve, or the block / round wrap-up.
    pub fn step_next(&mut self) {
        if self.phase == Phase::Replay {
            return;
        }
        if self.phase == Phase::Planning {
            if !self.plan_confirmed && !self.confirm_plan() {
                return;
            }
            self.phase = Phase::Resolving;
            self.current_block = 1;
            self.pending.clear();
        }
        if self.pending.is_empty() {
            if self.on_block_start() {
                return;
            }
            self.pending = self.resolver.prepare_block(&mut self.w, self.current_block).into();
        }
        if let Some(idx) = self.pending.pop_front() {
            self.resolver.execute_single(&mut self.w, idx);
        }
        if self.pending.is_empty() {
            let end_of_round = self.current_block >= self.def.total_blocks;
            if end_of_round {
                self.resolver.apply_round_end_effects(&mut self.w);
            }
            self.outcome = self.verdict();
            let over = !self.outcome.is_empty();
            if over || end_of_round {
                self.combat_over = over;
                self.phase = Phase::Replay;
                return;
            }
            self.current_block += 1;
        }
    }

    /// EncounterObjectives.verdict (rout): every hero down = lost, every enemy out = won, else fled when no hero is
    /// on the field and one withdrew.
    fn verdict(&self) -> String {
        let w = &self.w;
        let heroes = &w.allies;
        if heroes.iter().all(|&h| !w.units[h].is_alive()) {
            return "lost".into();
        }
        if !w.enemies.iter().any(|&e| w.units[e].is_active()) {
            return "won".into();
        }
        let hero_active = heroes.iter().any(|&h| w.units[h].is_active());
        let withdrew = heroes.iter().filter(|&&h| w.units[h].is_alive() && w.units[h].is_withdrawn).count();
        if !hero_active && withdrew > 0 {
            return "fled".into();
        }
        String::new()
    }

    pub fn can_advance_round(&self) -> bool {
        self.phase == Phase::Replay && !self.combat_over && self.current_round < self.def.max_rounds
    }

    pub fn advance_round(&mut self) {
        if self.can_advance_round() {
            self.begin_round(self.current_round + 1);
        }
    }
}

fn draft_broken(w: &World, plan: &Plan, e: UnitId) -> bool {
    if plan.steps_of(e).is_empty() {
        return false;
    }
    let st = plan.compile(w, e);
    st.error_index >= 0 || st.entries.is_empty()
}

impl Session {
    /// CatalogEncounter.on_block_start: the reinforcement wave enters before its block (that step does nothing else).
    fn on_block_start(&mut self) -> bool {
        if self.wave.is_empty() || self.reinforcements_spawned {
            return false;
        }
        if self.current_round != self.def.wave_round || self.current_block != self.def.wave_block {
            return false;
        }
        self.reinforcements_spawned = true;
        let wave = self.wave.clone();
        let deployment = self.def.deployment.clone();
        self.w.place_leftovers(&wave, false, &deployment);
        true
    }
}

impl Session {
    /// plan_apply_preset: replaces one ally's steps with an AI draft for `preset`.
    pub fn plan_apply_preset(&mut self, actor: UnitId, preset: &str) {
        if self.phase != Phase::Planning || !self.w.units[actor].is_active() {
            return;
        }
        let p = self.w.profile_of(actor).with_preset(preset);
        ai::build_steps(&self.w, &mut self.plan, actor, &p);
    }
}
