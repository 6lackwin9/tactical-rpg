//! CombatSession + CatalogEncounter / WorldEncounter (scripts/combat/session/): one battle's round flow. Enemies are
//! drafted by the AI when a round begins; the party is drafted by a preset (or edited) and committed on confirm; then
//! blocks resolve action by action. The encounter rules run at CatalogEncounter's hook points: guests and protect,
//! objectives (rout, hold, escape, slay), deployments, the reinforcement wave, boss phases, enemy morale and start
//! buffs (encounter.rs). Rosters come from a `Roster` (catalog units, or world characters with their extras).

use crate::ai;
use crate::command::CmdKind;
use crate::encounter::{self, HoldGate, Morale, Objectives, PartyFate, Phases, StayBehind, LEADER_TIERS};
use crate::grid::{Grid, TerrainTable, V2};
use crate::plan::Plan;
use crate::resolver::Resolver;
use crate::round_intent;
use crate::metrics::BattleMetrics;
use crate::world::{AiProfile, UnitId, World};
use aegis_core::value::{self, Obj};
use rpgkit_data::{Catalog, UnitDef};
use rpgkit_rules::ruleset::Ruleset;
use rpgkit_rules::status;
use rpgkit_rules::{build_unit_in, DeckRule, Unit};
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
    /// Authored terrain rows (ENCOUNTER2 R0): GRID_H strings of GRID_W glyphs; empty = the zone's rows.
    pub rows: Vec<String>,
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
    /// RULESET1 3.1.3: the rules the battle plays by, set by its caller (the world bridge, the Combat Arena, the Dojo,
    /// the sim, the host); never read from the row (classic by default).
    pub ruleset: Ruleset,
    /// RULESET1 V2, V3: every enemy's whole plan is revealed while planning (read only under `plan_badges`); set by its
    /// caller (the world bridge from the world's `plan_reveal_flag`, the Arena and Dojo toggles, the host); never read
    /// from the row (false by default).
    pub plan_reveal: bool,
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
            rows: value::strings(o, "rows"),
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
            ruleset: Ruleset::classic(),
            plan_reveal: false,
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
    /// RULESET1 5 (rule poise): poise now and Max Poise at the last refill (0 and 0 in classic or without a bar).
    pub poise: i32,
    pub max_poise: i32,
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
    /// ENCOUNTER2 section 6: the encounter's `stay_behind` block (None: none, or the objective is not escape).
    pub stay: Option<StayBehind>,
    /// ENCOUNTER2 S2: the verdict when the last round ends with none (`timeout_outcome`: fled, or lost).
    pub timeout_outcome: String,
    /// S1: how many of `w.spared` have their log cell.
    spared_logged: usize,
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
        // RULESET1 TQ23 (a): the intent each roster unit starts in (read only under round_intent)
        let mut default_intents: Vec<(UnitId, String)> = Vec::new();
        // RULESET1 9 D1-D4: under the deck rule every unit fights with its deck (classic: None, units as today)
        let deck = DeckRule::of(&def.ruleset);
        for p in party {
            profiles.insert(built.len(), p.profile);
            hero_keys.insert(built.len(), p.key);
            art.push(p.art);
            palettes.push(p.palette);
            let mut u = p.unit;
            // a party member built without the rule (its passives are its builder's: progression::build_in) gets
            // the open basics and its active slots here
            if let (Some(rule), None) = (&deck, &u.deck_rule) {
                u.deck_rule = Some(rule.clone());
                u.apply_deck(&cat);
            }
            built.push(u);
        }
        let mut spawn = |r: &str, name: &str, built: &mut Vec<Unit>| -> UnitId {
            let mut e = roster.entry(&cat, r).expect("checked");
            e.name = name.to_string();
            let mut u = build_unit_in(&cat, &e, def.unit_ap, deck.as_ref());
            // a world character's passives pass the deck rule's slot gate (D3) set above
            roster.customize(&cat, r, &mut u);
            let id = built.len();
            profiles.insert(id, AiProfile::for_unit(&cat, &e));
            art.push(if !e.art.is_empty() { e.art.clone() } else { cat.classes.get(&e.class_id).map(|c| c.art.clone()).unwrap_or_else(|| "unit".into()) });
            palettes.push(e.palette.clone());
            built.push(u);
            units_by_ref.entry(r.to_string()).or_default().push(id);
            hero_keys.insert(id, roster.lore_ref(&cat, r));
            if !e.default_intent.is_empty() {
                default_intents.push((id, e.default_intent.clone()));
            }
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
        grid.apply_board(&def.zone, &def.rows, &cat.zones);
        // ENCOUNTER2 S1, H2: the unkillable enemy-side units and the gate of a hold escape (none in classic)
        let unkillable: Vec<UnitId> = value::strings(&def.raw, "unkillable").iter().flat_map(|r| units_by_ref.get(r).cloned().unwrap_or_default()).filter(|&c| c >= n_allies).collect();
        let stay = StayBehind::read(&def.raw);
        let hold = stay.as_ref().filter(|sb| sb.is_hold()).map(|sb| HoldGate {
            tiles: sb.tiles.clone(),
            label: sb.tiles_label.clone(),
            holder: None,
            fallback_any: sb.fallback_any,
            heroes: (0..n_allies).filter(|c| !guests.contains(c)).collect(),
        });
        let mut w = World { cat: cat.clone(), units: built, grid, allies: (0..n_allies).collect(), enemies: (n_allies..count).collect(), profiles, total_blocks: def.total_blocks, hp_journal: None, ruleset: def.ruleset.clone(), round_states: IndexMap::new(), plan_reveal: def.plan_reveal, poise: IndexMap::new(), hold, unkillable, spared: Vec::new(), ground: Default::default() };
        w.add_item_skills();
        w.ignore_hazards_when_off();
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
        // RULESET1_INTENT MV1, TQ23: first facings and starting intents (nothing in classic)
        round_intent::battle_start(&mut w, &default_intents);
        let mut resolver = Resolver::new(def.total_blocks, def.rng_seed);
        for &c in allies.iter().chain(enemies.iter()) {
            resolver.register(c);
        }
        for u in w.units.iter_mut() {
            u.loadout_locked = true;
        }
        let timeout_outcome = encounter::timeout_outcome(&def.raw);
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
            stay,
            timeout_outcome,
            spared_logged: 0,
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
        // RULESET1_INTENT RS2: last round's Rest pays after ap_regen; the round's target and marks reset
        let rest = round_intent::round_start(&mut self.w);
        self.events.extend(rest);
        // RULESET1 P3: every poise bar refills before planning (rule poise)
        crate::poise::round_start(&mut self.w);
        // HOOKS1 8.1: `round_start` triggers after the refill (rule triggers; nothing in classic)
        crate::trigger::round_start(&mut self.resolver, &mut self.w);
        if crate::trigger::on(&self.w) {
            self.log_triggers(round, 1);
            self.log_ground(round, 1);
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
                poise: crate::poise::state(&self.w, i).map_or(0, |p| p.value),
                max_poise: crate::poise::state(&self.w, i).map_or(0, |p| p.max),
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

    /// RULESET1 V5, intent UI5: the round intent records of every active unit, both sides (party from its plan while
    /// planning, else from its committed actions; enemies as `enemy_intents`, with morale); empty without the rule
    /// `plan_badges` (classic) or once the battle is over.
    pub fn round_intents(&self) -> Vec<crate::intent::Intent> {
        if self.combat_over || !crate::intent::badges_on(&self.w) {
            return Vec::new();
        }
        let enraged: Vec<UnitId> = self.w.enemies.iter().copied().filter(|&e| self.is_enraged(e)).collect();
        let plan = self.is_planning().then_some(&self.plan);
        let mut out = crate::intent::round_intents(&self.w, plan, &self.resolver.scheduled, &self.on_grid_at_schedule, &enraged);
        for r in out.iter_mut().filter(|r| self.w.enemies.contains(&r.unit)) {
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
        // RULESET1_INTENT AI1 (RULESET1-AI): under round_intent every enemy picks its intent and target by the AI table
        if round_intent::on(&self.w) {
            let mut enemy_plan = Plan::new();
            let lines = ai::plan_enemy_intents(&mut self.w, &mut enemy_plan);
            self.events.extend(lines);
            enemy_plan.commit(&self.w, &mut self.resolver);
            return;
        }
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
        // RULESET1_INTENT AI5 (RULESET1-AI): under round_intent a tactic picks each member's intent and target
        if round_intent::on(&self.w) {
            let presets: Vec<(UnitId, String)> = allies.iter().map(|&a| (a, self.preset_for(a, preset))).collect();
            let lines = ai::plan_party_intents(&mut self.w, &mut self.plan, &presets);
            self.events.extend(lines);
            return;
        }
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
        if round_intent::on(&self.w) {
            let lines = ai::plan_party_intents(&mut self.w, &mut self.plan, &[(actor, preset.to_string())]);
            self.events.extend(lines);
            return;
        }
        let p = self.w.profile_of(actor).with_preset(preset);
        ai::build_steps(&self.w, &mut self.plan, actor, &p);
    }

    /// RULESET1_INTENT IN2, IN6: sets unit `c`'s round intent while the round is planned (refused otherwise). A change
    /// drops its intent-bound buffs (the lines go to `events`) and its planned steps (they were for the old intent).
    pub fn set_round_intent(&mut self, c: UnitId, intent: &str) -> Result<Vec<String>, String> {
        if self.phase != Phase::Planning || self.plan_confirmed {
            return Err("The intent changes only at a round start, while planning.".into());
        }
        let Some(i) = round_intent::RoundIntent::parse(intent) else { return Err(format!("Unknown intent '{intent}'.")) };
        let before = round_intent::intent_of(&self.w, c);
        let lines = round_intent::set_intent(&mut self.w, c, i)?;
        if before.is_some_and(|b| b != i) {
            self.plan.clear(c);
        }
        self.events.extend(lines.iter().cloned());
        Ok(lines)
    }

    /// RULESET1_INTENT TG1: sets unit `c`'s round target while planning (its intent first; the intent table decides).
    pub fn set_round_target(&mut self, c: UnitId, target: round_intent::RoundTarget) -> Result<(), String> {
        if self.phase != Phase::Planning || self.plan_confirmed {
            return Err("The target is set while planning.".into());
        }
        round_intent::set_target(&mut self.w, c, target)
    }

    /// RULESET1_INTENT MV11: sets the kind of unit `c`'s Maneuver round while planning.
    pub fn set_maneuver(&mut self, c: UnitId, kind: round_intent::ManeuverKind) -> Result<(), String> {
        if self.phase != Phase::Planning || self.plan_confirmed {
            return Err("The Maneuver kind is set while planning.".into());
        }
        round_intent::set_maneuver(&mut self.w, c, kind)
    }

    pub fn plan_is_valid(&self) -> bool {
        self.w.allies.iter().all(|&a| !self.w.units[a].is_active() || self.plan.is_valid(&self.w, a))
    }

    pub fn confirm_plan(&mut self) -> bool {
        // ENCOUNTER2 H1: a hold escape is not planned before someone holds
        if self.phase != Phase::Planning || !self.plan_is_valid() || self.needs_stay_pick() {
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
            self.log_triggers(round, block);
            self.log_ground(round, block);
            self.log_spared(round, block);
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
            // Every popped action gets its cell, also one cancelled before it resolved (as Godot logs it); RULESET1 P9:
            // one its unit's break cancelled has the break's cell instead
            if !a.broken {
                self.append(crate::log::from_action(&self.w, &a, round, block));
            }
            for r in &a.command.reactions {
                self.append(crate::log::riposte_entry(&self.w, r, round, block));
            }
            // RULESET1 P9: each break the action dealt gets its cell; the metrics count the hits (rule poise)
            for st in &a.command.staggers {
                if let Some(b) = &st.broke {
                    self.append(crate::log::break_entry(&self.w, st.target, b, round, block));
                }
            }
            if !a.command.staggers.is_empty() {
                self.feed(|m, s| m.on_staggers(s, idx));
            }
            // HOOKS1 FM9: each forced move's cell, then each collision hit's (the metrics hear its HP change with it)
            // and the break it caused (rule forced_move)
            for m in &a.command.pushes {
                self.append(crate::log::forced_entry(&self.w, m, &a.command.name, round, block));
                for h in &m.hits {
                    if let Some(j) = self.w.hp_journal.as_mut() {
                        j.extend(h.journal.iter().copied());
                    }
                    self.append(crate::log::collision_entry(&self.w, m, h, &a.command.name, round, block));
                    if let Some(b) = h.stagger.as_ref().and_then(|st| st.broke.as_ref()) {
                        self.append(crate::log::break_entry(&self.w, h.unit, b, round, block));
                    }
                }
            }
            if !a.command.pushes.is_empty() {
                self.feed(|m, s| m.on_forced(s, idx));
            }
            // HOOKS1 8.1: the triggers the action fired (rule triggers; none in classic)
            self.log_triggers(round, block);
            self.log_ground(round, block);
            self.log_spared(round, block);
        }
        if self.pending.is_empty() {
            let block = self.current_block;
            self.feed(|m, s| m.on_block_completed(s, block));
            let end_of_round = self.current_block >= self.def.total_blocks;
            if end_of_round {
                self.resolver.apply_round_end_effects(&mut self.w);
                self.log_triggers(self.current_round, block);
                self.log_ground(self.current_round, block);
            }
            self.on_block_end();
            self.log_spared(self.current_round, block);
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

    /// HOOKS1 8.1: a cell per trigger effect since the last call (rule triggers, none in classic); a trigger's forced move
    /// gets its move and collision cells (FM9), and the HP rows its damage held back reach the metrics with its cells.
    /// The caller logs the ground next (HOOKS1-GF: a moved unit's hazard hits).
    fn log_triggers(&mut self, round: i32, block: i32) {
        for f in self.resolver.triggers.take() {
            if let Some(j) = self.w.hp_journal.as_mut() {
                j.extend(f.journal.iter().copied());
            }
            for l in &f.lines {
                self.append(crate::log::trigger_entry(&self.w, &f, l, round, block));
            }
            for m in &f.moves {
                self.append(crate::log::forced_entry(&self.w, m, &f.source, round, block));
                for h in &m.hits {
                    self.append(crate::log::collision_entry(&self.w, m, h, &f.source, round, block));
                }
            }
        }
    }

    /// HOOKS1 6: a cell per ground event since the last call (placements, hazard hits, fades; rule ground_hazards, none
    /// in classic); a hit on entering hands its held-back HP rows to the metrics with its own cell.
    fn log_ground(&mut self, round: i32, block: i32) {
        for ev in self.w.ground.take() {
            if let crate::ground::Event::Hit(h) = &ev {
                if let Some(j) = self.w.hp_journal.as_mut() {
                    j.extend(h.journal.iter().copied());
                }
            }
            self.append(crate::log::ground_entry(&self.w, &ev, round, block));
            self.feed(|m, s| m.on_ground(s, &ev));
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

    /// ENCOUNTER2 S2 (was BattleBridge.outcome's last-round rule, written by each caller): the battle's verdict, else
    /// `timeout_outcome` (fled by default) once the last round is over, else "" while it can go on.
    pub fn final_outcome(&self) -> String {
        if !self.outcome.is_empty() {
            return self.outcome.clone();
        }
        if self.phase == Phase::Replay && !self.can_advance_round() {
            return self.timeout_outcome.clone();
        }
        String::new()
    }

    /// ENCOUNTER2 S3: each hero (party-side units but guests, ally order) and how the battle left it: stayed (held at
    /// a won hold escape), downed (0 HP), escaped (withdrew), else standing.
    pub fn party_fates(&self) -> Vec<(UnitId, PartyFate)> {
        self.w
            .allies
            .iter()
            .copied()
            .filter(|c| !self.guests.contains(c))
            .map(|c| {
                let u = &self.w.units[c];
                let fate = if self.objective.stayed.contains(&c) {
                    PartyFate::Stayed
                } else if !u.is_alive() {
                    PartyFate::Downed
                } else if u.is_withdrawn {
                    PartyFate::Escaped
                } else {
                    PartyFate::Standing
                };
                (c, fate)
            })
            .collect()
    }

    /// ENCOUNTER2 H1: a hold escape still waits for its holder (planning cannot be confirmed).
    pub fn needs_stay_pick(&self) -> bool {
        self.w.hold.as_ref().is_some_and(|g| g.holder.is_none())
    }

    /// ENCOUNTER2 H1: `c` holds the hold tiles (host request `stay_pick`): once, while round 1 is planned and nothing
    /// is confirmed, a hero on the field.
    pub fn stay_pick(&mut self, c: UnitId) -> Result<(), String> {
        let Some(g) = self.w.hold.as_ref() else { return Err("This battle has no tile to hold.".into()) };
        if let Some(h) = g.holder {
            return Err(format!("{} already holds {}.", self.w.units[h].name, g.label));
        }
        if self.current_round != 1 || !self.is_planning() {
            return Err("The holder is picked before round 1 is confirmed.".into());
        }
        if c >= self.w.units.len() || !g.heroes.contains(&c) || !self.w.has(c) {
            return Err(format!("Pick a hero on the field to hold {}.", g.label));
        }
        if let Some(g) = self.w.hold.as_mut() {
            g.holder = Some(c);
        }
        Ok(())
    }

    /// ENCOUNTER2 H1: the holder sims and autoplay pick (`pick_default`: the hero on the field with the most HP, the
    /// first on a tie; or the first hero on the field); None when no pick is due.
    pub fn stay_pick_default(&mut self) -> Option<UnitId> {
        if !self.needs_stay_pick() || self.current_round != 1 || !self.is_planning() {
            return None;
        }
        let g = self.w.hold.as_ref()?;
        let on_field: Vec<UnitId> = g.heroes.iter().copied().filter(|&c| self.w.has(c)).collect();
        let first = self.stay.as_ref().is_some_and(|sb| sb.pick_default == "first");
        let mut pick = on_field.first().copied()?;
        if !first {
            for &c in &on_field {
                if self.w.units[c].hp > self.w.units[pick].hp {
                    pick = c;
                }
            }
        }
        self.stay_pick(pick).ok()?;
        Some(pick)
    }

    /// ENCOUNTER2 H3: the plan line of a hold escape while its gate is shut ("" otherwise).
    pub fn gate_note(&self) -> String {
        self.w.hold.as_ref().map(|g| g.note(&self.w)).unwrap_or_default()
    }

    /// ENCOUNTER2 S1: one EVENT cell (and event line) for each unkillable unit damage first failed to down.
    fn log_spared(&mut self, round: i32, block: i32) {
        while self.spared_logged < self.w.spared.len() {
            let c = self.w.spared[self.spared_logged];
            self.spared_logged += 1;
            let name = self.w.units[c].name.clone();
            let line = format!("{name} will not fall here.");
            let ally = self.w.allies.contains(&c);
            self.append(crate::log::Entry::event(round, block, &name, ally, "Unkillable", "Unkillable", "SPARED", &line, ""));
            self.events.push(line);
        }
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
