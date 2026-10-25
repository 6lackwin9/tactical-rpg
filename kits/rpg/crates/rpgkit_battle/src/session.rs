//! CombatSession + CatalogEncounter / WorldEncounter (scripts/combat/session/): one battle's round flow. Enemies are
//! drafted by the AI when a round begins; the party is drafted by a preset (or edited) and committed on confirm; then
//! blocks resolve action by action. The encounter rules run at CatalogEncounter's hook points: guests and protect,
//! objectives (rout, hold, escape, slay), deployments, the reinforcement wave, boss phases, enemy morale and start
//! buffs (encounter.rs). Rosters come from a `Roster` (catalog units, or world characters with their extras).

use crate::ai;
use crate::command::CmdKind;
use crate::encounter::{self, Morale, Objectives, Phases, LEADER_TIERS};
use crate::grid::{Grid, TerrainTable, V2};
use crate::plan::Plan;
use crate::resolver::Resolver;
use crate::metrics::BattleMetrics;
use crate::world::{AiProfile, UnitId, World};
use aegis_core::value::{self, Obj};
use rpgkit_data::{Catalog, UnitDef};
use rpgkit_rules::status;
use rpgkit_rules::{build_unit, Unit};
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

/// Which start_status entries (PACE1) are granted at round 1.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum StartGrant {
    /// None (previews, the sim default, the arena).
    #[default]
    None,
    /// Every entry (the sim's start=on).
    All,
    /// Per entry, in list order (GameEncounter: the game's flags decide).
    Only(Vec<bool>),
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
    pub guests: Vec<String>,
    pub protect: Vec<String>,
    pub deployment: String,
    pub objective: String,
    /// Boss phases and the reinforcement wave (PHASEFOLD: an `arrival` phase, first in the list).
    pub phases: Vec<Value>,
    pub start_status: Vec<Value>,
    pub start_grant: StartGrant,
    /// The whole entry (objective and morale fields are read from it).
    pub raw: Obj,
}

impl EncounterDef {
    /// ContentSchema "encounters" defaults applied to a (possibly partial) entry; a row with the retired `wave_*`
    /// fields is folded first (PHASEFOLD 1.4).
    pub fn from_obj(o: &Obj) -> Self {
        let mut folded = o.clone();
        encounter::fold_legacy_wave(&mut folded);
        let o = &folded;
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
            guests: value::strings(o, "guests"),
            protect: value::strings(o, "protect"),
            deployment: if ["standard", "ambush", "centre"].contains(&deployment.as_str()) { deployment } else { "standard".into() },
            objective: value::string(o, "objective", "rout"),
            phases: value::array(o, "phases").to_vec(),
            start_status: value::array(o, "start_status").to_vec(),
            start_grant: StartGrant::None,
            raw: o.clone(),
        }
    }
}

/// Where roster refs come from: catalog units (an `IndexMap` of extra entries, then the catalog), or world characters
/// (rpgkit_world: the base unit wearing the character, then its named items, passives and tuning).
pub trait Roster {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef>;
    /// CatalogEncounter._customize: a freshly spawned unit's extras.
    fn customize(&self, _cat: &Catalog, _r: &str, _u: &mut Unit) {}
    /// The roster tier of a ref (morale: leaders and fearless units).
    fn tier(&self, cat: &Catalog, r: &str) -> String {
        self.entry(cat, r).map(|e| e.tier).unwrap_or_default()
    }
    /// The world character a ref plays (Combatant.lore_ref; "" = none): a hero's stable key across battles.
    fn lore_ref(&self, _cat: &Catalog, _r: &str) -> String {
        String::new()
    }
}

impl Roster for IndexMap<String, UnitDef> {
    fn entry(&self, cat: &Catalog, r: &str) -> Option<UnitDef> {
        self.get(r).cloned().or_else(|| cat.units.get(r).cloned())
    }
}

/// A unit built outside the encounter (GameEncounter: a saved party member through PartyBuilder).
pub struct Prebuilt {
    pub unit: rpgkit_rules::Unit,
    pub art: String,
    /// The palette its body wears (ASSETORG2; "" = none).
    pub palette: String,
    pub profile: AiProfile,
    /// The party member id (GameEncounter.hero_key).
    pub key: String,
}

/// One unit at one point of a round (BlockSnapshot: the state the screens show).
#[derive(Debug, Clone, PartialEq)]
pub struct UnitSnap {
    pub hp: i32,
    pub max_hp: i32,
    pub ap: i32,
    pub pos: Option<V2>,
    pub stance: rpgkit_rules::Stance,
    pub guard: f64,
    pub attack_boost: f64,
    pub active: bool,
    pub withdrawn: bool,
    pub statuses: Vec<String>,
    pub pending: Vec<String>,
}

/// One round as it was played (the round archive: CombatSession.round_actions and round_snapshots): the scheduled
/// actions (the enemies' drafts, then everyone's once the plan is confirmed; cancelled ones flagged as they resolve)
/// and a snapshot of every unit at the round's start (index 0) and after each resolved block (index = block).
#[derive(Debug, Clone)]
pub struct RoundRecord {
    pub round: i32,
    pub actions: Vec<crate::resolver::ScheduledAction>,
    pub snapshots: Vec<Vec<UnitSnap>>,
}

/// REWARD1 2.2: how a battle left an enemy-side unit. Killed (HP 0, whoever dealt the blow), surrendered (MORALE1: a
/// broken group withdraws in place), fled (withdrew on its own), standing (still there, or never came).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fate {
    Killed,
    Surrendered,
    Fled,
    Standing,
}

impl Fate {
    /// Killed, surrendered or fled: it pays its common rewards.
    pub fn defeated(self) -> bool {
        self != Fate::Standing
    }

    pub fn name(self) -> &'static str {
        match self {
            Fate::Killed => "killed",
            Fate::Surrendered => "surrendered",
            Fate::Fled => "fled",
            Fate::Standing => "standing",
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
    /// The active enemies on the grid when this round's enemies were scheduled (IntentRules: idle vs arriving).
    pub on_grid_at_schedule: Vec<UnitId>,
    /// Every round so far, the current one last (the round archive, for reviewing past rounds).
    pub history: Vec<RoundRecord>,
    /// Art key per unit (ContentFactory.art_of: the unit's own art, else its class's, else "unit").
    pub art: Vec<String>,
    /// Palette per unit (ASSETORG2; "" = none).
    pub palettes: Vec<String>,
    /// Party-side extras (also in allies): planned with Balanced, never saved.
    pub guests: Vec<UnitId>,
    /// Guests that must live.
    pub protected: Vec<UnitId>,
    pub objective: Objectives,
    pub phases: Phases,
    pub morale: Morale,
    /// Roster ref -> units spawned from it, spawn order.
    pub units_by_ref: IndexMap<String, Vec<UnitId>>,
    /// Encounter events (wave, phases, morale breaks, start buffs) as lines, oldest first.
    pub events: Vec<String>,
    /// The action log (CombatSession.log_entries): every resolved action, riposte, chase, dash and encounter event.
    pub log: Vec<crate::log::Entry>,
    /// The battle metrics following this fight (attach_metrics), fed at the session's signal points.
    pub metrics: Option<Box<BattleMetrics>>,
    /// The action the last resolving step popped: (round, block, index into `resolver.scheduled`); kept until the next
    /// one. Not in Godot; for the screens (SPRITE1-U: the actor plays its attack clip); no rule reads it.
    pub last_popped: Option<(i32, i32, usize)>,
    /// Unit -> its hero key (a party member id or a world character), when it has one.
    hero_keys: IndexMap<UnitId, String>,
    unit_keys: IndexMap<UnitId, String>,
    units_by_key: IndexMap<String, UnitId>,
    unit_tiers: IndexMap<UnitId, String>,
    authored_profiles: IndexMap<UnitId, AiProfile>,
    planning_round: i32,
    pending: VecDeque<usize>,
}

impl Session {
    /// CatalogEncounter.build + CombatSession.start: spawn, deploy, begin round 1. `units` resolves roster ids.
    pub fn start(cat: Arc<Catalog>, def: EncounterDef, units: &dyn Roster) -> Self {
        Self::start_with(cat, def, units, Vec::new())
    }

    /// `start` with `party` (non-empty) fighting instead of the encounter's own party roster (GameEncounter).
    pub fn start_with(cat: Arc<Catalog>, def: EncounterDef, roster: &dyn Roster, party: Vec<Prebuilt>) -> Self {
        let has = |id: &String| roster.entry(&cat, id).is_some();
        let party_ids: Vec<String> = if party.is_empty() { def.party.iter().filter(|id| has(id)).cloned().collect() } else { Vec::new() };
        let enemy_ids: Vec<String> = def.enemies.iter().filter(|id| has(id)).cloned().collect();
        let mut guest_ids: Vec<String> = Vec::new();
        for id in def.guests.iter().filter(|id| has(id)) {
            if !guest_ids.contains(id) && guest_ids.len() < encounter::MAX_GUESTS {
                guest_ids.push(id.clone());
            }
        }
        let mut phases = Phases::create(&def.phases);
        // (ref, phase id, arrival) in phase order: the arrival comes first, so its units follow the enemies (PHASEFOLD).
        let mut phase_ids: Vec<(String, String, bool)> = Vec::new();
        for p in &phases.phases {
            for r in p.wave_refs() {
                if has(&r) {
                    phase_ids.push((r, p.id.clone(), p.is_arrival()));
                }
            }
        }
        // _display_names over party + enemies + phase waves (the arrival first) + guests (guests last: names unchanged).
        let roster_order: Vec<String> = party_ids.iter().chain(&enemy_ids).cloned().chain(phase_ids.iter().map(|p| p.0.clone())).chain(guest_ids.iter().cloned()).collect();
        let mut seen: IndexMap<String, i32> = IndexMap::new();
        let names: Vec<String> = roster_order
            .iter()
            .map(|id| {
                let base = roster.entry(&cat, id).map(|e| e.name).unwrap_or_else(|| id.clone());
                let n = seen.get(&base).copied().unwrap_or(0) + 1;
                seen.insert(base.clone(), n);
                if n == 1 {
                    base
                } else {
                    format!("{base} {n}")
                }
            })
            .collect();
        let mut built: Vec<Unit> = Vec::new();
        let mut profiles = IndexMap::new();
        let mut art = Vec::new();
        let mut palettes: Vec<String> = Vec::new();
        let mut units_by_ref: IndexMap<String, Vec<UnitId>> = IndexMap::new();
        let mut hero_keys: IndexMap<UnitId, String> = IndexMap::new();
        for p in party {
            profiles.insert(built.len(), p.profile);
            hero_keys.insert(built.len(), p.key);
            art.push(p.art);
            palettes.push(p.palette);
            built.push(p.unit);
        }
        let mut spawn = |r: &str, name: &str, built: &mut Vec<Unit>| -> UnitId {
            let mut e = roster.entry(&cat, r).expect("checked");
            e.name = name.to_string();
            let mut u = build_unit(&cat, &e, def.unit_ap);
            roster.customize(&cat, r, &mut u);
            let id = built.len();
            profiles.insert(id, AiProfile::for_unit(&cat, &e));
            art.push(if !e.art.is_empty() { e.art.clone() } else { cat.classes.get(&e.class_id).map(|c| c.art.clone()).unwrap_or_else(|| "unit".into()) });
            palettes.push(e.palette.clone());
            built.push(u);
            units_by_ref.entry(r.to_string()).or_default().push(id);
            hero_keys.insert(id, roster.lore_ref(&cat, r));
            id
        };
        let n_party_ids = party_ids.len();
        for (i, r) in party_ids.iter().enumerate() {
            spawn(r, &names[i], &mut built);
        }
        let guest_slot = party_ids.len() + enemy_ids.len() + phase_ids.len();
        let mut guests = Vec::new();
        let mut protected = Vec::new();
        for (k, r) in guest_ids.iter().enumerate() {
            let g = spawn(r, &names[guest_slot + k], &mut built);
            guests.push(g);
            if def.protect.contains(r) {
                protected.push(g);
            }
        }
        let n_allies = built.len();
        let mut slot = n_party_ids;
        let mut deployed = Vec::new();
        for r in &enemy_ids {
            deployed.push(spawn(r, &names[slot], &mut built));
            slot += 1;
        }
        for (r, pid, arrival) in &phase_ids {
            let c = spawn(r, &names[slot], &mut built);
            if *arrival {
                phases.track(pid, c);
            } else {
                phases.reserve(pid, c);
            }
            slot += 1;
        }
        let count = built.len();
        let mut grid = Grid::new(def.open, def.exits.clone(), TerrainTable::from_catalog(&cat), count);
        grid.apply_zone(&def.zone);
        let mut w = World { cat: cat.clone(), units: built, grid, allies: (0..n_allies).collect(), enemies: (n_allies..count).collect(), profiles, total_blocks: def.total_blocks, hp_journal: None };
        w.add_item_skills();
        let objective = Objectives::create(&def.raw);
        // MORALE1 1.2: unit keys, tiers, authored profiles and the starting groups.
        let mut unit_keys = IndexMap::new();
        let mut units_by_key = IndexMap::new();
        let mut unit_tiers = IndexMap::new();
        let mut authored_profiles = IndexMap::new();
        let morale = if value::boolean(&def.raw, "morale", false) {
            for (r, list) in &units_by_ref {
                for (n, &c) in list.iter().enumerate() {
                    if !w.enemies.contains(&c) {
                        continue;
                    }
                    let key = format!("{r}#{n}");
                    unit_keys.insert(c, key.clone());
                    units_by_key.insert(key, c);
                    unit_tiers.insert(c, roster.tier(&cat, r));
                    if let Some(p) = w.profiles.get(&c) {
                        authored_profiles.insert(c, p.clone());
                    }
                }
            }
            let starting: Vec<String> = deployed.iter().map(|c| unit_keys.get(c).cloned().unwrap_or_default()).collect();
            let authored = value::string(&def.raw, "morale_leader", "");
            let leader_ref = if !authored.is_empty() {
                authored
            } else {
                def.enemies.iter().find(|r| LEADER_TIERS.contains(&roster.entry(&cat, r).map(|e| raw_tier(&e)).unwrap_or_default().as_str())).cloned().unwrap_or_default()
            };
            let leader_key = format!("{leader_ref}#0");
            Morale::create(&def.raw, &starting, if units_by_key.contains_key(&leader_key) { &leader_key } else { "" })
        } else {
            Morale::default()
        };
        let allies = w.allies.clone();
        let enemies = w.enemies.clone();
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
        let mut s = Session {
            def,
            w,
            resolver,
            plan: Plan::new(),
            current_round: 1,
            current_block: 1,
            phase: Phase::Planning,
            combat_over: false,
            outcome: String::new(),
            plan_confirmed: false,
            on_grid_at_schedule: Vec::new(),
            history: Vec::new(),
            art,
            palettes,
            guests,
            protected,
            objective,
            phases,
            morale,
            units_by_ref,
            events: Vec::new(),
            log: Vec::new(),
            metrics: None,
            last_popped: None,
            hero_keys,
            unit_keys,
            units_by_key,
            unit_tiers,
            authored_profiles,
            planning_round: 1,
            pending: VecDeque::new(),
        };
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
        self.prepare_round(round);
        self.plan = Plan::new();
        self.plan_confirmed = false;
        self.on_grid_at_schedule = self.w.enemies.iter().copied().filter(|&e| self.w.units[e].is_active() && self.w.has(e)).collect();
        self.schedule_enemy_ai();
        let start = self.snapshot();
        self.history.push(RoundRecord { round, actions: self.resolver.scheduled.clone(), snapshots: vec![start] });
        self.feed(|m, s| if m.live { m.round_started(s) });
    }

    /// CombatEncounter.hero_key: a party unit's stable key across battles (the member id, else the world character it
    /// plays, else its name); guests have none.
    pub fn hero_key(&self, c: UnitId) -> String {
        if self.guests.contains(&c) {
            return String::new();
        }
        match self.hero_keys.get(&c) {
            Some(k) if !k.is_empty() => k.clone(),
            _ => self.w.units[c].name.clone(),
        }
    }

    /// _append_entry: adds a log cell (the metrics read it).
    fn append(&mut self, e: crate::log::Entry) {
        self.log.push(e);
        let idx = self.log.len() - 1;
        self.feed(|m, s| m.on_log(s, idx));
    }

    /// The metrics hear the HP changes since the last call (in order), then `f`.
    fn feed(&mut self, f: impl FnOnce(&mut BattleMetrics, &Session)) {
        let Some(mut m) = self.metrics.take() else { return };
        if m.is_following() {
            let changes = self.w.hp_journal.as_mut().map(std::mem::take).unwrap_or_default();
            for (c, hp) in changes {
                m.on_health(self, c, hp);
            }
            f(&mut m, self);
        }
        self.metrics = Some(m);
    }

    /// Starts `m` following this fight from now (BattleMetrics.on_start; live = the session closes rounds itself, as
    /// the game does; otherwise the driver calls metrics_step / metrics_round / metrics_end like the sim).
    pub fn attach_metrics(&mut self, mut m: BattleMetrics, live: bool) {
        m.live = live;
        self.w.hp_journal = Some(Vec::new());
        m.on_start(self);
        self.metrics = Some(Box::new(m));
    }

    /// The sim's observer hook after each step.
    pub fn metrics_step(&mut self) {
        self.feed(|m, s| m.on_step(s));
    }

    /// The sim's observer hook after a new round began.
    pub fn metrics_round(&mut self) {
        self.feed(|m, s| m.on_round(s));
    }

    /// The fight is over (the sim's result; a live battle passes the default): the records close and stop listening.
    pub fn metrics_end(&mut self, result: crate::metrics::EndResult) {
        self.feed(|m, s| if m.live { m.finish(s) } else { m.on_end(s, result) });
        self.w.hp_journal = None;
    }

    /// Every unit now (BlockSnapshot).
    pub fn snapshot(&self) -> Vec<UnitSnap> {
        self.w
            .units
            .iter()
            .enumerate()
            .map(|(i, u)| UnitSnap {
                hp: u.hp,
                max_hp: u.max_hp,
                ap: u.ap,
                pos: self.w.grid.pos(i),
                stance: u.stance,
                guard: u.guard,
                attack_boost: u.attack_boost,
                active: u.is_active(),
                withdrawn: u.is_withdrawn,
                statuses: u.st.active.iter().cloned().collect(),
                pending: u.st.pending.iter().cloned().collect(),
            })
            .collect()
    }

    /// The archive's record of `round` (None before it starts; CombatSession.recorded_actions and snapshot_at).
    pub fn round_record(&self, round: i32) -> Option<&RoundRecord> {
        self.history.iter().find(|r| r.round == round)
    }

    /// CombatSession._refresh_archive: the current round's actions as scheduled now.
    fn refresh_archive(&mut self) {
        let actions = self.resolver.scheduled.clone();
        if let Some(r) = self.history.last_mut() {
            r.actions = actions;
        }
    }

    /// CombatSession.is_planning: the party plans this round (not confirmed, the battle goes on).
    pub fn is_planning(&self) -> bool {
        self.phase == Phase::Planning && !self.combat_over && !self.plan_confirmed
    }

    /// CombatSession.enemy_intents (INTENT1): the enemy tells while planning (none otherwise), with each enemy's
    /// morale state word and group (MORALE1 rule 13).
    pub fn enemy_intents(&self) -> Vec<crate::intent::Intent> {
        if !self.is_planning() {
            return Vec::new();
        }
        let enraged: Vec<UnitId> = self.w.enemies.iter().copied().filter(|&e| self.is_enraged(e)).collect();
        let mut out = crate::intent::intents(&self.w, &self.resolver.scheduled, &self.on_grid_at_schedule, &enraged);
        for r in out.iter_mut() {
            r.morale_state = self.morale_state_of(r.unit);
            r.morale_group = self.morale_group_of(r.unit);
        }
        out
    }

    /// CatalogEncounter.prepare_round (+ WorldEncounter's start buffs at round 1): round phases and phases met after
    /// the last block fire, morale breaks are announced, live AI profiles are rebuilt.
    fn prepare_round(&mut self, round: i32) {
        self.planning_round = round;
        self.phases.mark_round(round);
        while let Some(id) = self.phases.next_due() {
            self.fire_phase(&id, round, 0);
        }
        for line in self.morale.take_lines(0) {
            self.morale_entry(round, 0, &line);
            self.events.push(line);
        }
        self.apply_phase_profiles(round);
        if round == 1 {
            self.grant_start_buffs();
        }
    }

    /// PACE1 rule 7: each granted start_status entry to every party unit on the field (grant_now).
    fn grant_start_buffs(&mut self) {
        for (i, raw) in self.def.start_status.clone().iter().enumerate() {
            let Some(sb) = raw.as_object() else { continue };
            let granted = match &self.def.start_grant {
                StartGrant::None => false,
                StartGrant::All => true,
                StartGrant::Only(v) => v.get(i).copied().unwrap_or(false),
            };
            if !granted {
                continue;
            }
            let st = value::string(sb, "status", "");
            let source = format!("start:{}", self.def.id.rsplit(':').next().unwrap_or(""));
            let mut names = Vec::new();
            for &c in &self.w.allies.clone() {
                if self.w.has(c) {
                    status::grant_now(&mut self.w.units[c], &st, value::int(sb, "rounds", 2), value::int(sb, "magnitude", 0), &source);
                    names.push(self.w.units[c].name.clone());
                }
            }
            if !names.is_empty() {
                let text = value::string(sb, "text", "");
                let shown = status::label_with_magnitude(&st, status::magnitude_or_min(&st, value::int(sb, "magnitude", 0)));
                let name = self.def.name.clone();
                self.append(crate::log::Entry::event(1, 0, &name, true, &status::display_name(&st), "Start", "START", &text, &shown));
                self.events.push(if text.is_empty() { format!("{} for the party.", status::display_name(&st)) } else { text });
            }
        }
    }

    pub fn units_of_ref(&self, r: &str) -> Vec<UnitId> {
        self.units_by_ref.get(r).cloned().unwrap_or_default()
    }

    fn first_of_ref(&self, r: &str) -> Option<UnitId> {
        self.units_by_ref.get(r).and_then(|l| l.first().copied())
    }

    pub fn is_reserved(&self, c: UnitId) -> bool {
        self.phases.is_reserved(c)
    }

    /// REWARD1 2.4: every enemy-side unit in spawn order (enemies, reinforcements, phase waves) with its roster ref and
    /// how the battle left it (a wave that never came, a reserved phase wave: standing).
    pub fn fates(&self) -> Vec<(String, Fate)> {
        let mut ref_of: IndexMap<UnitId, &str> = IndexMap::new();
        for (r, list) in &self.units_by_ref {
            for &c in list {
                ref_of.insert(c, r);
            }
        }
        self.w
            .enemies
            .iter()
            .filter_map(|&c| {
                let r = ref_of.get(&c)?;
                let u = &self.w.units[c];
                let fate = if !u.is_alive() {
                    Fate::Killed
                } else if u.is_withdrawn {
                    if self.unit_keys.get(&c).is_some_and(|k| self.morale.surrendered.contains(k)) {
                        Fate::Surrendered
                    } else {
                        Fate::Fled
                    }
                } else {
                    Fate::Standing
                };
                Some((r.to_string(), fate))
            })
            .collect()
    }

    /// Fires phase `id` at [round, block] (block 0 = round start): wave, status, enrage, then marked fired.
    fn fire_phase(&mut self, id: &str, round: i32, block: i32) {
        let Some(p) = self.phases.phase(id).cloned() else { return };
        // The actor: an hp_below trigger's unit, else the encounter.
        let mut actor = self.def.name.clone();
        if value::string(&p.trigger, "kind", "") == "hp_below" {
            if let Some(c) = self.first_of_ref(&value::string(&p.trigger, "unit", "")) {
                actor = self.w.units[c].name.clone();
            }
        }
        let mut parts: Vec<String> = Vec::new();
        for fx in p.effects_of("wave") {
            let units = self.phases.units_of(id);
            let dep = self.def.deployment.clone();
            self.w.place_leftovers(&units, false, &dep);
            self.form_morale_group(&units, &format!("phase:{id}"));
            let label = value::string(fx, "label", "");
            let arrived = units.iter().filter(|&&c| self.w.grid.positions[c].is_some()).count();
            parts.push(if label.is_empty() { format!("{arrived} more hostiles") } else { label });
        }
        for fx in p.effects_of("status") {
            let st = value::string(fx, "status", "");
            let mut granted = false;
            for c in self.units_of_ref(&value::string(fx, "unit", "")) {
                if self.w.has(c) && !self.is_reserved(c) {
                    status::grant_now(&mut self.w.units[c], &st, value::int(fx, "rounds", 2), value::int(fx, "magnitude", 0), &format!("phase:{id}"));
                    granted = true;
                }
            }
            if granted {
                parts.push(status::display_name(&st));
            }
        }
        for fx in p.effects_of("enrage") {
            let mut granted = false;
            for c in self.units_of_ref(&value::string(fx, "unit", "")) {
                if !self.w.has(c) || self.is_reserved(c) {
                    continue;
                }
                let (rounds, power, src) = (self.def.max_rounds + 1, value::int(fx, "power", 20), format!("phase:{id}"));
                if block > 0 {
                    status::queue(&mut self.w.units[c], "empower", rounds, power, &src);
                } else {
                    status::grant_now(&mut self.w.units[c], "empower", rounds, power, &src);
                }
                granted = true;
            }
            if granted {
                parts.push("Enraged".into());
            }
        }
        if p.effects_of("ai").next().is_some() {
            parts.push(if block > 0 { "new tactics from the next round" } else { "new tactics" }.into());
        }
        let line = p.effects_of("line").last().map(|fx| value::string(fx, "text", "")).unwrap_or_default();
        self.append(crate::log::Entry::event(round, block, &actor, false, &p.name, "Phase", "PHASE", &line, &parts.join(", ")));
        self.events.push(format!("{}: {}{}", p.name, line, if parts.is_empty() { String::new() } else { format!(" ({})", parts.join(", ")) }));
        self.phases.mark_fired(id, round, block);
    }

    /// Phase ai effects, then the morale flee bonus, on the live profiles (rebuilt from the authored ones).
    fn apply_phase_profiles(&mut self, round: i32) {
        if self.morale.on {
            for (c, p) in &self.authored_profiles {
                self.w.profiles.insert(*c, p.clone());
            }
        }
        let by_ref = self.units_by_ref.clone();
        self.phases.apply_profiles(&mut self.w.profiles, |r| by_ref.get(r).cloned().unwrap_or_default(), round);
        if !self.morale.on {
            return;
        }
        let keys: Vec<UnitId> = self.authored_profiles.keys().copied().collect();
        for c in keys {
            let base = self.w.profiles[&c].flee_ratio;
            let f = self.flee_ratio_of(c, base);
            self.w.profiles.get_mut(&c).unwrap().flee_ratio = f;
        }
    }

    pub fn morale_group_of(&self, c: UnitId) -> i32 {
        if !self.morale.on {
            return -1;
        }
        self.unit_keys.get(&c).map(|k| self.morale.group_of(k)).unwrap_or(-1)
    }

    /// The shown morale state ("" = morale off or none).
    pub fn morale_state_of(&self, c: UnitId) -> &'static str {
        let gi = self.morale_group_of(c);
        if gi < 0 {
            ""
        } else if self.is_fearless(c) {
            "fearless"
        } else {
            self.morale.group_state(gi)
        }
    }

    fn flee_ratio_of(&self, c: UnitId, base: f64) -> f64 {
        let gi = self.morale_group_of(c);
        if gi < 0 || self.is_fearless(c) {
            return base;
        }
        encounter::flee_for(base, self.morale.group_state(gi))
    }

    /// A unit's tier (minion, standard, elite, boss, hero): the roster's, else the unit's own.
    pub fn tier_of(&self, c: UnitId) -> String {
        self.unit_tiers.get(&c).cloned().unwrap_or_else(|| self.w.units[c].tier.clone())
    }

    /// Rule 5: boss or elite tier, a live profile that never flees, or the slay target.
    pub fn is_fearless(&self, c: UnitId) -> bool {
        let tier = self.tier_of(c);
        if LEADER_TIERS.contains(&tier.as_str()) {
            return true;
        }
        if let Some(p) = self.w.profiles.get(&c) {
            if p.reckless || p.flee_ratio <= 0.0 {
                return true;
            }
        }
        self.objective.objective == "slay" && self.first_of_ref(&self.objective.unit_ref) == Some(c)
    }

    fn form_morale_group(&mut self, units: &[UnitId], source: &str) {
        if !self.morale.on || self.morale.has_group(source) {
            return;
        }
        let mut keys = Vec::new();
        let mut size = 0;
        for &c in units {
            let Some(k) = self.unit_keys.get(&c) else { continue };
            keys.push(k.clone());
            if !self.is_reserved(c) {
                size += 1;
            }
        }
        self.morale.form_group(&keys, size, source);
    }

    /// PACE1 3.6: enraged in the planning round.
    pub fn is_enraged(&self, c: UnitId) -> bool {
        self.phases.enraged_refs(self.planning_round).iter().any(|r| self.units_of_ref(r).contains(&c))
    }

    /// Re-plans the round's enemy side (after the start positions changed, e.g. the sim's jitter).
    pub fn replan_enemies(&mut self) {
        self.resolver.scheduled.clear();
        self.plan = Plan::new();
        self.schedule_enemy_ai();
        self.refresh_archive();
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

    /// OBJ1 rule 2: guests always draft with Balanced.
    fn preset_for(&self, c: UnitId, preset: &str) -> String {
        if self.guests.contains(&c) {
            "balanced".into()
        } else {
            preset.to_string()
        }
    }

    /// plan_apply_preset_all: every active ally drafted with `preset`, then broken drafts re-drafted.
    pub fn plan_apply_preset_all(&mut self, preset: &str) {
        if self.phase != Phase::Planning {
            return;
        }
        let allies = self.w.allies.clone();
        for &a in &allies {
            if self.w.units[a].is_active() {
                let p = self.w.profile_of(a).with_preset(&self.preset_for(a, preset));
                ai::build_steps(&self.w, &mut self.plan, a, &p);
            }
        }
        for _ in 0..allies.len() {
            let mut redrafted = false;
            for &a in &allies {
                if self.w.units[a].is_active() && !self.plan.is_valid(&self.w, a) {
                    let p = self.w.profile_of(a).with_preset(&self.preset_for(a, preset));
                    ai::build_steps(&self.w, &mut self.plan, a, &p);
                    redrafted = true;
                }
            }
            if !redrafted {
                break;
            }
        }
    }

    /// plan_apply_preset: replaces one ally's steps with an AI draft for `preset`.
    pub fn plan_apply_preset(&mut self, actor: UnitId, preset: &str) {
        if self.phase != Phase::Planning || !self.w.units[actor].is_active() {
            return;
        }
        let p = self.w.profile_of(actor).with_preset(preset);
        ai::build_steps(&self.w, &mut self.plan, actor, &p);
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
        self.refresh_archive();
        true
    }

    /// One resolution step: one action, an encounter event (phase, morale break, wave), or the block / round wrap-up.
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
            // A chase or dash as an attack started gets its own cell.
            let (round, block) = (self.current_round, self.current_block);
            for (i, note) in std::mem::take(&mut self.resolver.notes) {
                if let Some(e) = crate::log::move_note_entry(&self.w, &self.resolver.scheduled[i], round, block, &note) {
                    self.append(e);
                }
            }
        }
        if let Some(idx) = self.pending.pop_front() {
            self.last_popped = Some((self.current_round, self.current_block, idx));
            self.resolver.execute_single(&mut self.w, idx);
            for k in std::mem::take(&mut self.resolver.combos) {
                self.feed(|m, s| m.on_combo(s, &k));
            }
            self.on_action_resolved(idx);
            let (round, block) = (self.current_round, self.current_block);
            let a = self.resolver.scheduled[idx].clone();
            // Every popped action gets its cell, also one cancelled before it resolved (as Godot logs it).
            self.append(crate::log::from_action(&self.w, &a, round, block));
            for r in &a.command.reactions {
                self.append(crate::log::riposte_entry(&self.w, r, round, block));
            }
        }
        if self.pending.is_empty() {
            let block = self.current_block;
            self.feed(|m, s| m.on_block_completed(s, block));
            let end_of_round = self.current_block >= self.def.total_blocks;
            if end_of_round {
                self.resolver.apply_round_end_effects(&mut self.w);
            }
            self.on_block_end();
            // The archive: the actions as they resolved and the state after this block.
            self.refresh_archive();
            let snap = self.snapshot();
            if let Some(r) = self.history.last_mut() {
                r.snapshots.push(snap);
            }
            self.outcome = self.verdict(end_of_round);
            let over = !self.outcome.is_empty();
            if over || end_of_round {
                self.combat_over = over;
                self.phase = Phase::Replay;
                self.feed(|m, s| {
                    m.on_round_closed(s);
                    if m.live {
                        m.round_concluded(s);
                    }
                });
                return;
            }
            self.current_block += 1;
        }
    }

    /// CatalogEncounter.on_block_start: a due phase fires first, then one morale break line, then the arrival (the
    /// reinforcement wave) enters; each takes the step (the block resolves on the next one).
    fn on_block_start(&mut self) -> bool {
        let (round, block) = (self.current_round, self.current_block);
        if let Some(id) = self.phases.next_due() {
            self.fire_phase(&id, round, block);
            return true;
        }
        let broke = self.morale.take_lines(1);
        if let Some(line) = broke.into_iter().next() {
            self.morale_entry(round, block, &line);
            self.events.push(line);
            return true;
        }
        let Some(id) = self.phases.arrival_due(round, block) else { return false };
        self.fire_arrival(&id, round, block);
        true
    }

    /// PHASEFOLD rule 4: the arrival's units take the free tiles nearest the middle of the enemy half, form the morale
    /// group "wave" and get the Wave cell and event line (actor = the phase name); no other effect, banner or pause.
    fn fire_arrival(&mut self, id: &str, round: i32, block: i32) {
        let Some(p) = self.phases.phase(id).cloned() else { return };
        let wave = self.phases.units_of(id);
        let deployment = self.def.deployment.clone();
        self.w.place_leftovers(&wave, false, &deployment);
        self.form_morale_group(&wave, "wave");
        let names: Vec<String> = wave.iter().filter(|&&c| self.w.grid.positions[c].is_some()).map(|&c| self.w.units[c].name.clone()).collect();
        let label = p.effects_of("wave").next().map(|fx| value::string(fx, "label", "")).unwrap_or_default();
        self.append(crate::log::Entry::event(round, block, &p.name, false, &label, "Wave", "ARRIVED", &format!("{} join the battle.", names.join(", ")), &format!("{} more hostiles", names.len())));
        self.events.push(format!("{label}: {} join the battle.", names.join(", ")));
        self.phases.mark_fired(id, round, block);
    }

    /// MORALE1 rule 12: a break's EVENT cell (actor = the leader, else the encounter).
    fn morale_entry(&mut self, round: i32, block: i32, line: &str) {
        let actor = self.units_by_key.get(&self.morale.leader).map_or_else(|| self.def.name.clone(), |&c| self.w.units[c].name.clone());
        self.append(crate::log::Entry::event(round, block, &actor, false, "Morale", "Morale", "MORALE", line, ""));
    }

    /// MORALE1 rule 6: the crits an action dealt (attacks, area attacks, then provoked ripostes).
    fn on_action_resolved(&mut self, idx: usize) {
        if !self.morale.on {
            return;
        }
        let cmd = &self.resolver.scheduled[idx].command;
        let mut hits: Vec<(UnitId, bool)> = Vec::new();
        match &cmd.kind {
            CmdKind::Attack { last_crit: true, last_target: Some(t), .. } => hits.push((*t, !self.w.units[*t].is_alive())),
            CmdKind::Area { crit_hits, .. } => hits.extend(crit_hits.iter().copied()),
            _ => {}
        }
        for r in &cmd.reactions {
            if r.crit {
                hits.push((r.attacker, r.defeated));
            }
        }
        for (c, killed) in hits {
            if let Some(k) = self.unit_keys.get(&c).cloned() {
                self.morale.record_crit(&k, killed);
            }
        }
    }

    /// Phase checks, then morale losses / breaks / surrenders (surrendering units withdraw at their tile).
    fn on_block_end(&mut self) {
        let by_ref = self.units_by_ref.clone();
        self.phases.check_block_end(&self.w, |r| by_ref.get(r).and_then(|l| l.first().copied()));
        if !self.morale.on {
            return;
        }
        let n = self.w.units.len();
        let fearless: Vec<bool> = (0..n).map(|c| self.is_fearless(c)).collect();
        let reserved: Vec<bool> = (0..n).map(|c| self.is_reserved(c)).collect();
        let by_key = self.units_by_key.clone();
        let surrendering = self.morale.block_end(&self.w, |k| by_key.get(k).copied(), |c| fearless[c], |c| reserved[c]);
        for c in surrendering {
            self.w.withdraw_in_place(c);
        }
    }

    /// OBJ1 rule 3 over the heroes, guests, protected guests and every enemy that is not reserved.
    fn verdict(&mut self, end_of_round: bool) -> String {
        let heroes: Vec<UnitId> = self.w.allies.iter().copied().filter(|a| !self.guests.contains(a)).collect();
        let foes: Vec<UnitId> = self.w.enemies.iter().copied().filter(|&e| !self.is_reserved(e)).collect();
        let target = if self.objective.objective == "slay" { self.first_of_ref(&self.objective.unit_ref) } else { None };
        let (guests, protected) = (self.guests.clone(), self.protected.clone());
        self.objective.verdict(&self.w, &heroes, &guests, &protected, &foes, end_of_round, target)
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

/// A unit entry's own tier as written (morale's leader rule reads the raw field; "" when the entry has none).
fn raw_tier(e: &UnitDef) -> String {
    e.tier.clone()
}

fn draft_broken(w: &World, plan: &Plan, e: UnitId) -> bool {
    if plan.steps_of(e).is_empty() {
        return false;
    }
    let st = plan.compile(w, e);
    st.error_index >= 0 || st.entries.is_empty()
}
