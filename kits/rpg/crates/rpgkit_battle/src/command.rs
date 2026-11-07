//! BattleCommand and the commands (scripts/core/combat/battle_command.gd, scripts/combat/actions/*), plus the rules
//! they call: RiposteRules, SupportRules, HazardRules, CoverRules. RNG draws happen in Godot's order: hit crit,
//! status roll, riposte crit, per unit hit.

use crate::grid::{Grid, V2, NO_CELL};
use crate::path::{self, Cells, Hooks};
use crate::round_intent;
use crate::skill::{self, SkillId};
use crate::world::{UnitId, World};
use aegis_core::rng::GodotRng;
use rpgkit_data::ItemDef;
use rpgkit_rules::damage::{self, Hit, HitInput};
use rpgkit_rules::stats::roundi;
use rpgkit_rules::{status, Stance, Unit};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Reaction {
    pub defender: UnitId,
    pub attacker: UnitId,
    pub damage: i32,
    pub crit: bool,
    pub defeated: bool,
}

#[derive(Debug, Clone)]
pub enum CmdKind {
    Attack {
        target: Option<UnitId>,
        skill_mult: f64,
        use_offhand: bool,
        follow_tiles: i32,
        dash_tiles: i32,
        status_rounds: i32,
        min_range: i32,
        max_range: i32,
        last_damage: i32,
        last_crit: bool,
        last_target: Option<UnitId>,
    },
    Area {
        cells: Vec<V2>,
        aim: V2,
        skill_mult: f64,
        status_rounds: i32,
        last_damage: i32,
        last_hits: i32,
        crit_hits: Vec<(UnitId, bool)>,
    },
    Defend {
        /// The guard fraction (S1.2-S4: the skill's, or the ruleset default, `skill::guard_mitigation`).
        mitigation: f64,
        /// S1.2-S4: the rounds the guard lasts; 2 or more queues the `guard` status for the later rounds at the impact.
        guard_rounds: i32,
        /// AUDIT-11 (FB-85): the fraction of the queued Round 2+ `guard` status (`skill::guard_later_mitigation`).
        later_mitigation: f64,
        grant_status: String,
        status_rounds: i32,
        extra_grants: Vec<Value>,
        guard_broken: bool,
    },
    Focus {
        boost: f64,
        rounds: i32,
    },
    Rest {
        recover_ap: i32,
    },
    StatusBlock {
        status: String,
    },
    Support {
        target: UnitId,
        skill: SkillId,
        last_heal: i32,
        /// The status the skill queued ("" = none) and the statuses it removed, last execution.
        last_status: String,
        last_cleansed: Vec<String>,
    },
    Target {
        target: Option<UnitId>,
    },
    /// ItemCommand (ITEMUSE1 rules 3-5): one battle-pouch consumable on `target`, in the SupportCommand slot.
    Item {
        target: UnitId,
        item: Box<ItemDef>,
        used: bool,
        last_heal: i32,
        last_damage: i32,
        last_boost: f64,
    },
    Walk {
        path: Vec<V2>,
        expected_start: V2,
        goal: V2,
        /// The unit an Engage's approach walks toward (WalkCommand.approach_target; display only: the tells).
        approach: Option<UnitId>,
        moved_tiles: i32,
        cut_short: bool,
        /// What stopped a cut-short walk (a unit's name, "the arena edge", a terrain, "no route"; "" = nothing).
        blocked_by: String,
    },
    Disengage {
        dest: V2,
        max_tiles: i32,
        landed: bool,
        moved_tiles: i32,
    },
}

/// RULESET1_INTENT MV4: the move a Maneuver skill carries inside its own command (rule `round_intent`): to the planned
/// Engage tile before the hit (at its start), or to the planned retreat tile after the hit (at its impact).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Carry {
    #[default]
    None,
    Engage(V2),
    Retreat(V2),
}

/// INPUT1 MR12: how a command moved its unit at its start, so the battle screen can draw it (a dash with its own
/// animation, apart from a walk): a chase step (MR16, AS1) or the dash of a skill with a position (MR8 a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveKind {
    Walk,
    Chase,
    Dash,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub source: UnitId,
    pub priority: i32,
    pub block_cost: i32,
    /// RULESET1 T1: blocks booked after the impact (enemy-side units under `enemy_moves`; 0 otherwise). The action
    /// lands at its last `block_cost` block and occupies `recovery` more.
    pub recovery: i32,
    pub name: String,
    /// VISCLAR1-B: the catalog id of the skill it runs ("" for an item or a command built by hand); names are not unique.
    pub skill: String,
    pub ap_cost: i32,
    pub stance_switch_cost: i32,
    pub stance_on_start: Option<Stance>,
    pub stance_on_end: Option<Stance>,
    pub est_damage: i32,
    pub status_effect: String,
    pub status_potency: i32,
    pub status_magnitude: i32,
    pub failed: bool,
    pub disrupt_break: bool,
    pub exploit: Vec<Value>,
    pub exploit_consume: bool,
    pub broken_targets: Vec<UnitId>,
    pub exploited: Vec<(UnitId, String)>,
    pub reactions: Vec<Reaction>,
    pub log: String,
    pub kind: CmdKind,
    /// RULESET1_INTENT IN6: the skill's `intent_buff` (its grants to its own user are intent-bound).
    pub intent_buff: bool,
    /// RULESET1_INTENT MV4: the carried move (Carry::None in classic and for every other command).
    pub carry: Carry,
    /// Decision "Auto-fill keeps a flank": planned to stay on its flank or behind tile (`round_intent::keeps_flank`).
    pub keep_flank: bool,
    /// RULESET1 5, 7: the skill's stagger, windup poise, unbreakable, sure break and armour piercing (read only under
    /// the rules `poise` and `pierce`).
    pub impact: crate::poise::Impact,
    /// RULESET1 P11: the windup poise this action added at its start (taken off when it lands or is cancelled).
    pub poise_bonus: i32,
    /// S2.2-S3 (FB-71): the windup bonus pips this action added at its start under `poise_mode` pips (taken off with
    /// `poise_bonus`; 0 under `bar`).
    pub poise_bonus_pips: i32,
    /// RULESET1 P4: the units the last execution damaged, in hit order, each with whether the hit was a flank (MV3).
    pub hits: Vec<(UnitId, bool)>,
    /// RULESET1 P6-P14: what the last execution's hits did to poise (rule `poise` only; see `poise::resolve_hits`).
    pub staggers: Vec<crate::poise::StaggerHit>,
    /// HOOKS1 FM1: the skill's forced move (read only under the rule `forced_move`).
    pub forced: Option<crate::forced::Shove>,
    /// HOOKS1 5: the forced moves the last execution drove (rule `forced_move` only; see `forced::resolve`).
    pub pushes: Vec<crate::forced::Moved>,
    /// HOOKS1 GH2: the skill's or item's ground hazard (read only under the rule `ground_hazards`; see `ground`).
    pub hazard: Option<crate::ground::Place>,
    /// HOOKS1 8.2: the skill's family (a trigger's `skills` condition reads it).
    pub family: String,
    /// HOOKS1 8.1: the skill's own triggers, held while the command runs (read only under the rule `triggers`).
    pub triggers: Vec<rpgkit_data::TriggerRow>,
    /// HOOKS1 8.1: the damaging hits of the last execution, a redirected part included (rule `triggers` reads them).
    pub struck: Vec<crate::trigger::Struck>,
    /// HOOKS1 8.1 `evaded`: the locked target the last execution did not find (GD5) or that dodged it (ATTACKLOCK1).
    pub evaded: Option<UnitId>,
    /// HOOKS1 8.4: the `ally_hit` triggers the hits consult (set by the resolver before the action lands; rule
    /// `triggers` only).
    pub ally: Option<crate::trigger::AllyWatch>,
    /// HOOKS1 RD1: the skill's riders, run per unit the command landed on (read only under the rule `triggers`).
    pub riders: Vec<rpgkit_data::Rider>,
    /// HOOKS1 HC1: the HP the command pays at its start (0 = none; set only under the rule `hp_cost`).
    pub hp_cost: i32,
    /// HOOKS1 IG1: the hits skip the target's guard and P6's guard factor (set only under the rule `ignore_guard`).
    pub ignore_guard: bool,
    /// HOOKS1 IG1: the last execution's bypass left out a guard (a hit target guarded; the breakdown says so).
    pub guard_bypassed: bool,
    /// HOOKS1 UM2: the skill's meter changes, paid at the command's start after the AP and HP cost (set only under the
    /// rule `unit_meters`).
    pub meters: Vec<rpgkit_data::MeterDelta>,
    /// INPUT1 MR10: the band test counts a diagonal neighbour of the target as distance 1 (`round_intent::diagonal`).
    pub diagonal: bool,
    /// INPUT1 MR1: the position a carried Engage dashes to (None = none, or not a skill with a position).
    pub position: Option<rpgkit_data::Position>,
    /// INPUT1 MR1: how far the carried move goes (a skill with a position: its range N; else its `dash_tiles`).
    pub carry_range: i32,
    /// INPUT1 MR8: the target's tile when the plan was made (NO_CELL = unknown): a target that left it is chased.
    pub target_cell: V2,
    /// INPUT1 MR16: chase tiles of a Support command on an ally (0 = none; set only under `round_intent`).
    pub follow: i32,
    /// INPUT1 MR8 c: the target's flank was protected at the start: at its impact the command does nothing.
    pub missed: bool,
    /// INPUT1 MR12: the board event of the move the command made at its start (None = it did not move).
    pub moved: Option<MoveKind>,
    /// SIMCOUNT1: `on_start` ran. A record for the feel sim; no rule reads it.
    pub started: bool,
    /// SIMCOUNT1: the start moved the unit by a dash, of a skill with a position or a plain `dash_tiles`. A record for the
    /// feel sim; no rule reads it (`moved` stays the board event).
    pub dashed: bool,
    /// SIMCOUNT1: the hit landed from a tile only the diagonal rule puts in its band (INPUT1 MR10). A record for the feel
    /// sim; no rule reads it.
    pub diagonal_hit: bool,
    /// ATTACKLOCK1: the target and its tile as the wind-up started, after the chase or dash, a forced move carrying the
    /// tile along (`attack_lock`; None = no lock, or the target was out of reach).
    pub locked: Option<(UnitId, V2)>,
    /// ATTACKLOCK1: the target that had left the locked tile at the last execution's impact (the swing missed it).
    pub dodged: Option<UnitId>,
}

impl Command {
    pub fn new(source: UnitId, priority: i32, block_cost: i32, kind: CmdKind) -> Self {
        Self {
            source,
            priority,
            block_cost,
            recovery: 0,
            name: String::new(),
            skill: String::new(),
            ap_cost: 0,
            stance_switch_cost: 0,
            stance_on_start: None,
            stance_on_end: None,
            est_damage: 0,
            status_effect: String::new(),
            status_potency: 0,
            status_magnitude: 0,
            failed: false,
            disrupt_break: false,
            exploit: Vec::new(),
            exploit_consume: false,
            broken_targets: Vec::new(),
            exploited: Vec::new(),
            reactions: Vec::new(),
            log: String::new(),
            kind,
            intent_buff: false,
            carry: Carry::None,
            keep_flank: false,
            impact: crate::poise::Impact::default(),
            poise_bonus: 0,
            poise_bonus_pips: 0,
            hits: Vec::new(),
            staggers: Vec::new(),
            forced: None,
            pushes: Vec::new(),
            hazard: None,
            family: String::new(),
            triggers: Vec::new(),
            struck: Vec::new(),
            evaded: None,
            ally: None,
            riders: Vec::new(),
            hp_cost: 0,
            ignore_guard: false,
            guard_bypassed: false,
            meters: Vec::new(),
            diagonal: false,
            position: None,
            carry_range: 0,
            target_cell: NO_CELL,
            follow: 0,
            missed: false,
            moved: None,
            started: false,
            dashed: false,
            diagonal_hit: false,
            locked: None,
            dodged: None,
        }
    }

    /// RULESET1 A2: the hit skips armour (rule `pierce` and the skill's `armor_piercing`). Public for the battle
    /// screen's forecasts (RULESET1-U slice 6).
    pub fn pierces(&self, w: &World) -> bool {
        self.impact.armor_piercing && w.ruleset.has(rpgkit_rules::ruleset::PIERCE)
    }

    pub fn initiative(&self, w: &World) -> i32 {
        self.priority * 1000 + w.units[self.source].initiative
    }

    pub fn is_defend(&self) -> bool {
        matches!(self.kind, CmdKind::Defend { .. })
    }

    /// ATTACKLOCK1: this command moves its unit by a dash as it starts (a carried Engage or a plain `dash_tiles`).
    pub(crate) fn dashes_at_start(&self) -> bool {
        matches!(self.carry, Carry::Engage(_)) || matches!(self.kind, CmdKind::Attack { dash_tiles, .. } if dash_tiles > 0)
    }

    /// ATTACKLOCK1: under `attack_lock`, an attack on a unit that has it in reach after its own chase or dash stores the
    /// unit and its tile; out of reach it takes no lock, and the band test decides at impact. `opening`: the tiles the
    /// block's dashes started from (decision "Attack lock answers": a dash on the same tick leaves the locked tile, for
    /// both sides), else the tile now. A melee band is `round_intent::melee_band`; an area aims at tiles and never locks.
    pub(crate) fn lock_tile(&mut self, w: &World, opening: Option<&[Option<V2>]>) {
        self.locked = None;
        let CmdKind::Attack { target, min_range, max_range, .. } = &self.kind else { return };
        let band = reach_band(w, self.source, *min_range, *max_range);
        if !w.ruleset.attack_lock().locks(round_intent::melee_band(band)) {
            return;
        }
        let Some(t) = effective_target(w, self.source, *target).filter(|t| in_reach(w, self.source, *t, band, self.diagonal)) else { return };
        self.locked = opening.map_or(w.grid.pos(t), |tiles| tiles[t]).map(|cell| (t, cell));
    }

    /// Runs when the command STARTS (after AP and stance): attack dash / chase, defend grants.
    pub fn on_start(&mut self, w: &mut World, rng: &mut GodotRng) -> String {
        self.started = true;
        let src = self.source;
        let diag = self.diagonal;
        match &mut self.kind {
            CmdKind::Attack { target, follow_tiles, dash_tiles, min_range, max_range, .. } => {
                let tgt = effective_target(w, src, *target);
                let band = reach_band(w, src, *min_range, *max_range);
                // RULESET1_INTENT GD5: chases and carried moves do not step toward a hidden unit
                if tgt.is_some_and(|t| round_intent::hidden(w, t)) {
                    return String::new();
                }
                if let (Carry::Engage(cell), Some(position)) = (self.carry, self.position) {
                    // Decision "Auto-fill keeps a flank": planned to stay, it hits from there while the tile still flanks
                    if self.keep_flank && cell == w.pos(src) && tgt.is_some_and(|t| round_intent::keeps_flank(w, t, cell, diag)) {
                        return String::new();
                    }
                    let (line, moved, protected) = carried_engage(w, rng, src, tgt, cell, position, self.carry_range, self.target_cell, diag);
                    self.moved = moved;
                    self.dashed = moved == Some(MoveKind::Dash);
                    if protected {
                        // INPUT1 MR8 c, MR9: a miss at its impact; the ruleset's cost decides what of its AP stays paid
                        self.missed = true;
                        let paid = self.ap_cost + self.stance_switch_cost;
                        let pays = if w.pays_ap(src) { w.ruleset.numbers.flank_protected.pays(paid) } else { 0 };
                        if w.pays_ap(src) {
                            w.units[src].ap += paid - pays;
                        }
                        let t = tgt.map_or_else(String::new, |t| w.units[t].name.clone());
                        return format!("[Flank Protected] {} finds {t}'s flank protected: {} misses ({pays} AP spent)", w.units[src].name, self.name);
                    }
                    return line;
                }
                if *dash_tiles > 0 {
                    return dash(w, rng, src, tgt, band, *dash_tiles, &mut self.dashed);
                }
                let Some(t) = tgt else { return String::new() };
                if *follow_tiles <= 0 || !w.units[t].is_active() || !w.units[src].is_active() || in_reach(w, src, t, band, diag) {
                    return String::new();
                }
                let line = chase_line(w, src, t, band, diag, *follow_tiles);
                if !line.is_empty() {
                    self.moved = Some(MoveKind::Chase);
                }
                line
            }
            // INPUT1 MR16: a Support command on an ally chases it into its reach as an attack does
            CmdKind::Support { target, skill, .. } if self.follow > 0 => {
                let (t, sk) = (*target, *skill);
                if t == src || !w.units[t].is_active() || !w.units[src].is_active() || !w.has(t) || round_intent::hidden(w, t) || support_in_reach(w, src, t, sk) {
                    return String::new();
                }
                let (lo, hi) = skill::support_reach(&w.cat.skills[sk], &w.units[src]);
                let line = chase_line(w, src, t, (lo.max(1), hi), false, self.follow);
                if !line.is_empty() {
                    self.moved = Some(MoveKind::Chase);
                }
                line
            }
            CmdKind::Defend { grant_status, status_rounds, extra_grants, .. } => {
                if (grant_status.is_empty() && extra_grants.is_empty()) || !w.units[src].is_active() {
                    return String::new();
                }
                let name = w.units[src].name.clone();
                let u = &mut w.units[src];
                if !grant_status.is_empty() {
                    if status::defend_buffs().contains(&grant_status.as_str()) {
                        status::grant_now(u, grant_status, *status_rounds, self.status_magnitude, &name);
                    } else {
                        status::queue(u, grant_status, *status_rounds, self.status_magnitude, &name);
                    }
                }
                let mut granted = status::grant_extras(u, extra_grants, true, &name);
                // RULESET1_INTENT IN6: intent-bound grants carry the unit's intent
                granted.insert(0, grant_status.clone());
                round_intent::bind(w, src, src, &granted, self.intent_buff);
                format!("[Grant] {} readies {}", name, grant_status)
            }
            _ => String::new(),
        }
    }

    /// The command's effect, at the end of its last block.
    pub fn execute(&mut self, w: &mut World, rng: &mut GodotRng) {
        self.reactions.clear();
        self.broken_targets.clear();
        self.exploited.clear();
        self.hits.clear();
        self.staggers.clear();
        self.pushes.clear();
        self.struck.clear();
        self.evaded = None;
        self.dodged = None;
        self.guard_bypassed = false;
        self.failed = false;
        let src = self.source;
        let src_name = w.units[src].name.clone();
        match self.kind.clone() {
            CmdKind::Attack { target, skill_mult, use_offhand, status_rounds, min_range, max_range, .. } => {
                self.set_attack_result(0, false, None);
                if !w.units[src].is_active() {
                    self.failed = true;
                    self.log = format!("[Action Failed] {src_name} cannot act");
                    return;
                }
                let Some(t) = effective_target(w, src, target).filter(|t| w.units[*t].is_active()) else {
                    self.failed = true;
                    self.log = format!("[Attack Miss] {src_name}: target no longer in combat");
                    return;
                };
                // RULESET1_INTENT GD5: a stealthed unit that moved this round is not found (AP stays spent)
                if round_intent::hidden(w, t) {
                    self.failed = true;
                    self.evaded = Some(t);
                    self.log = format!("[Lost] {src_name} loses {}", w.units[t].name);
                    return;
                }
                // INPUT1 MR8 c: a protected flank does nothing at its impact (no damage, stagger, status, rider, hit event,
                // riposte, exploit or flank count); the command kept its blocks
                if self.missed {
                    self.failed = true;
                    self.log = format!("[Flank Protected] {src_name}'s {} misses {}: its flank is protected", self.name, w.units[t].name);
                    return;
                }
                // ATTACKLOCK1: the target left the tile the attack committed to by a move of its own, wherever it stands
                // now (no hit, no riposte, AP stays spent; a forced move carries the lock along, `forced::shove_unit`); the
                // tile is read at impact, so a target back on it is hit. An `evaded` event, as a lost attack is
                if let Some((_, cell)) = self.locked.filter(|(l, cell)| *l == t && w.grid.pos(t) != Some(*cell)) {
                    self.failed = true;
                    self.dodged = Some(t);
                    self.evaded = Some(t);
                    self.log = format!("[Dodged] {} left {cell} before {src_name}'s {} landed", w.units[t].name, self.name);
                    return;
                }
                let band = reach_band(w, src, min_range, max_range);
                if !in_reach(w, src, t, band, self.diagonal) {
                    self.failed = true;
                    self.log = format!("[Attack Out of Range] {src_name} cannot reach {}", w.units[t].name);
                    return;
                }
                w.units[src].active_target = Some(t);
                let damaging = skill_mult > 0.0;
                // SIMCOUNT1: a hit only the diagonal rule let land (read before an intercept moves it)
                self.diagonal_hit = damaging && self.diagonal && !in_reach(w, src, t, band, false);
                // RULESET1_INTENT MV3: a flank hit (rule flank) matches `flanked`, takes the flank power and no riposte
                let flank = round_intent::flank_bonus(w, src, t);
                // HOOKS1 EX1: an `hp_below` entry matches the target's HP share before the hit (rule execute)
                let matched = if damaging { skill::exploit_match_by(&self.exploit, &w.units[t], &[], flank, &w.ruleset) } else { (String::new(), 1.0) };
                let mut taken = 0;
                let mut crit = false;
                let mut t = t;
                if damaging {
                    let ranged = w.units[src].attack_range > 1 && !use_offhand;
                    let cover = cover_between(w, src, t, ranged);
                    // HOOKS1 8.4: an intercept moves the whole hit to the friend that took it (rule triggers)
                    let (on, hit, dealt) = self.land(w, rng, src, t, ranged, use_offhand, skill_mult * matched.1 * flank_power(w, flank), cover, &[]);
                    t = on;
                    taken = dealt;
                    crit = hit.is_crit;
                    self.set_attack_result(taken, crit, Some(t));
                    self.hits.push((t, flank));
                    self.struck.push(crate::trigger::Struck { target: t, damage: taken, ranged });
                }
                let mut detail = String::new();
                if !self.status_effect.is_empty() && w.units[t].is_alive() {
                    // U1.2-S3b: the actor's KNOW term (ruleset `know_potency`; 0 = potency off)
                    let bonus = status::know_bonus(&w.units[src], w.ruleset.numbers.know_potency);
                    let ok = status::try_inflict_with(&mut w.units[t], &self.status_effect.clone(), self.status_potency, bonus, Some(rng), status_rounds, self.status_magnitude, &src_name);
                    detail = format!(" ({} {})", if ok { "inflicts" } else { "resisted" }, self.status_effect);
                }
                if !damaging {
                    self.log = format!("[Status] {src_name} -> {}{detail}", w.units[t].name);
                    return;
                }
                let ranged = w.units[src].attack_range > 1 && !use_offhand;
                if let Some(r) = riposte_counter(w, rng, src, t, ranged || flank) {
                    self.reactions.push(r);
                }
                self.after_hit(w, t, taken, &matched);
                if !matched.0.is_empty() {
                    self.exploited.push((t, matched.0.clone()));
                }
                let tu = &w.units[t];
                self.log = format!("[Attack] {src_name} struck {} for {taken}{}{detail} (HP {}/{})", tu.name, if crit { " CRIT" } else { "" }, tu.hp, tu.max_hp);
            }
            CmdKind::Area { cells, skill_mult, status_rounds, .. } => {
                let mut total = 0;
                let mut hits = 0;
                let mut crits = Vec::new();
                if !w.units[src].is_active() {
                    self.failed = true;
                    self.log = format!("[Action Failed] {src_name} cannot act");
                    return;
                }
                let mut parts = Vec::new();
                // HOOKS1 8.4: a unit the area also strikes takes no redirect (rule triggers)
                let inside: Vec<UnitId> = if self.ally.is_some() { cells.iter().filter_map(|c| w.occupant(*c)).collect() } else { Vec::new() };
                for cell in cells {
                    let Some(v) = w.occupant(cell) else { continue };
                    if v == src || skill_mult <= 0.0 {
                        continue;
                    }
                    let ranged = w.units[src].attack_range > 1;
                    let cover = cover_for(&w.grid, w.pos(src), cell, ranged);
                    let flank = round_intent::flank_bonus(w, src, v);
                    let matched = skill::exploit_match_by(&self.exploit, &w.units[v], &[], flank, &w.ruleset);
                    let (v, hit, taken) = self.land(w, rng, src, v, ranged, false, skill_mult * matched.1 * flank_power(w, flank), cover, &inside);
                    self.hits.push((v, flank));
                    self.struck.push(crate::trigger::Struck { target: v, damage: taken, ranged });
                    total += taken;
                    hits += 1;
                    if hit.is_crit {
                        crits.push((v, !w.units[v].is_alive()));
                    }
                    let mut note = String::new();
                    if !self.status_effect.is_empty() && w.units[v].is_alive() {
                        let st = self.status_effect.clone();
                        let bonus = status::know_bonus(&w.units[src], w.ruleset.numbers.know_potency);
                        let chance = status::apply_chance_with(&w.units[v], &st, self.status_potency, bonus);
                        note = if status::try_inflict_with(&mut w.units[v], &st, self.status_potency, bonus, Some(rng), status_rounds, self.status_magnitude, &src_name) {
                            format!(" [{} next round ({chance}%)]", status::display_name(&st))
                        } else {
                            format!(" [resisted {} ({chance}%)]", status::display_name(&st).to_lowercase())
                        };
                    }
                    let exploits = if matched.0.is_empty() {
                        String::new()
                    } else if matched.0 == skill::EXECUTE {
                        format!(" (execute x{:.2})", matched.1)
                    } else {
                        format!(" (exploits {})", status::display_name(&matched.0))
                    };
                    let vu = &w.units[v];
                    parts.push(format!("{} {taken}{}{}{note}{exploits}", vu.name, if hit.is_crit { " CRIT" } else { "" }, if vu.is_alive() { "" } else { " DEFEATED" }));
                    if let Some(r) = riposte_counter(w, rng, src, v, ranged || flank) {
                        self.reactions.push(r);
                    }
                    self.after_hit(w, v, taken, &matched);
                    if !matched.0.is_empty() {
                        self.exploited.push((v, matched.0.clone()));
                    }
                }
                if let CmdKind::Area { last_damage, last_hits, crit_hits, .. } = &mut self.kind {
                    *last_damage = total;
                    *last_hits = hits;
                    *crit_hits = crits;
                }
                // HOOKS1 GH3: a placement (no power, a hazard) hits nobody on purpose; its hazard follows at step 7
                self.log = if hits == 0 && skill_mult <= 0.0 && self.hazard.is_some() && crate::ground::on(w) {
                    format!("[Place] {src_name} uses {}", self.name)
                } else if hits == 0 {
                    format!("[Area Miss] {src_name}'s {} hit nobody.", self.name)
                } else {
                    format!("[Attack] {src_name}'s {} hit {hits} unit{} for {total} damage ({})", self.name, if hits == 1 { "" } else { "s" }, parts.join(", "))
                };
            }
            CmdKind::Defend { mitigation, guard_rounds, later_mitigation, guard_broken, .. } => {
                if !w.units[src].is_alive() {
                    self.failed = true;
                    return;
                }
                if guard_broken {
                    self.log = format!("[Defend] {src_name}'s guard was broken");
                    return;
                }
                // S1.2-S4 (Option 1C): the guard is held from the wind-up start (`trigger::hold_defend`); the impact
                // keeps the larger of it and the skill's own
                let u = &mut w.units[src];
                u.guard = u.guard.max(mitigation);
                self.log = if mitigation > 0.0 { format!("[Defend] {src_name} raises guard ({}% less damage)", (mitigation * 100.0).round() as i32) } else { format!("[Defend] {src_name} holds a defensive stance") };
                if queue_later_guard(&mut w.units[src], guard_rounds, later_mitigation, &src_name) {
                    self.log += &format!("; guard holds {} more round{}", guard_rounds - 1, if guard_rounds == 2 { "" } else { "s" });
                }
            }
            CmdKind::Focus { boost, rounds } => {
                if !w.units[src].is_alive() {
                    return;
                }
                w.units[src].apply_attack_boost(boost, rounds);
                self.log = format!("[Focus] {src_name} x{boost} for {rounds} rounds");
            }
            CmdKind::Rest { recover_ap } => {
                if !w.units[src].is_alive() {
                    return;
                }
                // RULESET1_INTENT RS2, CLN-45: with `rest_ap` round_end a landed Rest block pays its AP at round end
                if w.ruleset.rest_at_round_end() {
                    w.round_states.entry(src).or_default().rest_blocks += self.block_cost.max(1);
                }
                // RULESET1_INTENT RS3: each landed Rest block restores poise (rule poise; S2.2-S2: a pip per 2 under pips)
                crate::poise::restore(w, src, self.block_cost.max(1));
                let u = &mut w.units[src];
                u.ap = u.max_ap.min(u.ap + recover_ap);
                self.log = format!("[Rest] {src_name} AP {}/{}", u.ap, u.max_ap);
            }
            CmdKind::StatusBlock { status } => {
                self.log = format!("[Status] {src_name} is {status} and loses this block");
            }
            CmdKind::Support { target, skill, .. } => {
                if let CmdKind::Support { last_heal, last_status, last_cleansed, .. } = &mut self.kind {
                    (*last_heal, *last_status, *last_cleansed) = (0, String::new(), Vec::new());
                }
                if !w.units[src].is_active() {
                    self.failed = true;
                    self.log = format!("[Action Failed] {src_name} cannot act (unit is inactive).");
                    return;
                }
                if !w.units[target].is_active() {
                    self.failed = true;
                    self.log = format!("[Support Failed] {src_name}'s {} has no target left.", self.name);
                    return;
                }
                if !support_in_reach(w, src, target, skill) {
                    self.failed = true;
                    self.log = format!("[Support Failed] {src_name} cannot reach {} with {}.", w.units[target].name, self.name);
                    return;
                }
                let done = support_apply(w, src, target, skill);
                // RULESET1_INTENT IN6: intent-bound grants carry the holder's intent
                let mut granted = done.extras.clone();
                granted.insert(0, done.status.clone());
                let buff = w.cat.skills[skill].intent_buff;
                round_intent::bind(w, target, src, &granted, buff);
                let s = &w.cat.skills[skill];
                let t = &w.units[target];
                let mut parts = Vec::new();
                if skill::heals(s) {
                    parts.push(format!("healed {} (HP {}/{})", done.healed, t.hp, t.max_hp));
                }
                if skill::cleanses(s) {
                    let names: Vec<String> = done.cleansed.iter().map(|st| status::display_name(st)).collect();
                    parts.push(format!("cleansed {}", if names.is_empty() { "nothing".to_string() } else { names.join(", ") }));
                }
                let next_round = |st: &str, rounds: i32| format!("{} next round{}", status::display_name(st), if rounds <= 1 { String::new() } else { format!(" for {rounds} rounds") });
                if !done.status.is_empty() {
                    parts.push(next_round(&done.status, s.status_rounds));
                }
                for row in &s.extra_grants {
                    let st = row.get("status").and_then(Value::as_str).unwrap_or("").to_string();
                    if done.extras.contains(&st) {
                        parts.push(next_round(&st, row.get("rounds").and_then(Value::as_f64).map_or(1, |v| v as i32)));
                    }
                }
                let who = if target == src { "themselves".to_string() } else { t.name.clone() };
                self.log = format!("[Support] {src_name} used {} on {who}: {}", self.name, if parts.is_empty() { "no effect".to_string() } else { parts.join(", ") });
                if let CmdKind::Support { last_heal, last_status, last_cleansed, .. } = &mut self.kind {
                    (*last_heal, *last_status, *last_cleansed) = (done.healed, done.status, done.cleansed);
                }
            }
            CmdKind::Item { target, item, .. } => {
                let problem = item_problem(w, src, target, &item);
                if !problem.is_empty() {
                    self.failed = true;
                    self.log = format!("[Item] {} not used: {problem}.", item.name);
                    return;
                }
                let idx = w.units[src].inventory.pouch.iter().position(|p| p.id == item.id).unwrap();
                w.units[src].inventory.pouch.remove(idx);
                let mut parts = Vec::new();
                let (mut heal, mut dmg, mut boost) = (0, 0, 0.0);
                if item.heal > 0 {
                    let before = w.units[target].hp;
                    w.heal(target, item.heal);
                    let t = &w.units[target];
                    heal = t.hp - before;
                    parts.push(format!("healed {heal} (HP {}/{})", t.hp, t.max_hp));
                }
                if item.damage > 0 {
                    dmg = w.damage(target, item.damage, true);
                    let t = &w.units[target];
                    parts.push(format!("{dmg} damage (HP {}/{}){}", t.hp, t.max_hp, if t.is_alive() { "" } else { ", defeated" }));
                }
                let t = &mut w.units[target];
                if item.buff_pct > 0.0 {
                    boost = 1.0 + item.buff_pct;
                    t.apply_attack_boost(boost, item.duration);
                    parts.push(format!("+{}% attack for {} rounds", roundi(item.buff_pct * 100.0), item.duration));
                }
                let who = if target == src { "themselves".to_string() } else { t.name.clone() };
                if let CmdKind::Item { used, last_heal, last_damage, last_boost, .. } = &mut self.kind {
                    (*used, *last_heal, *last_damage, *last_boost) = (true, heal, dmg, boost);
                }
                self.log = format!("[Item] {src_name} used {} on {who}: {}", item.name, if parts.is_empty() { "no effect".into() } else { parts.join(", ") });
            }
            CmdKind::Target { target } => {
                if !w.units[src].is_alive() {
                    return;
                }
                w.units[src].active_target = target;
                self.log = match target { Some(t) => format!("[Target] {src_name} locks on {}", w.units[t].name), None => format!("[Target] {src_name} clears its target") };
            }
            CmdKind::Walk { path, expected_start, goal, .. } => {
                if !w.has(src) {
                    self.log = format!("[Move Failed] {src_name} could not execute movement (inactive or off grid).");
                    return;
                }
                let start = w.pos(src);
                let (moved, cut, blocker, notes) = walk(w, rng, src, &path, expected_start, goal);
                let end = w.grid.pos(src).unwrap_or(NO_CELL);
                self.log = if w.units[src].is_withdrawn {
                    format!("[Withdrawal] {src_name} moved from {start} and safely WITHDREW through Exit!")
                } else if moved == 0 && cut {
                    format!("[Move Cut Short] {src_name} couldn't move from {start} (blocked by {blocker}).")
                } else if cut {
                    format!("[Move Cut Short] {src_name} stopped at {end} after {moved} of {} tiles (blocked by {blocker}).", path.len())
                } else {
                    format!("[Move] {src_name} moved {moved} tile{} from {start} to {end}", if moved == 1 { "" } else { "s" })
                };
                if !w.units[src].is_withdrawn {
                    for note in notes {
                        self.log += &format!(" {note}");
                    }
                }
                if let CmdKind::Walk { moved_tiles, cut_short, blocked_by, .. } = &mut self.kind {
                    (*moved_tiles, *cut_short, *blocked_by) = (moved, cut, blocker);
                }
                if moved > 0 {
                    round_intent::note_moved(w, src);
                }
            }
            CmdKind::Disengage { dest, max_tiles, .. } => {
                if let CmdKind::Disengage { landed, moved_tiles, .. } = &mut self.kind {
                    (*landed, *moved_tiles) = (false, 0);
                }
                if !w.has(src) {
                    self.log = format!("[Disengage Failed] {src_name} cannot jump (inactive or off grid).");
                    return;
                }
                if w.units[src].has_status("root") {
                    self.log = format!("[Disengage Failed] {src_name} is rooted and can't jump.");
                    return;
                }
                let from = w.pos(src);
                let dist = from.dist(dest);
                if dist > max_tiles {
                    self.log = format!("[Disengage Failed] {src_name} can't reach {dest} ({dist} tiles, the jump covers {max_tiles}).");
                    return;
                }
                if !w.jump_unit(src, dest) {
                    self.log = format!("[Disengage Failed] {src_name} can't land on {dest}.");
                    return;
                }
                if let CmdKind::Disengage { landed, moved_tiles, .. } = &mut self.kind {
                    (*landed, *moved_tiles) = (true, dist);
                }
                round_intent::note_moved(w, src);
                self.log = format!("[Disengage] {src_name} jumped {dist} tile{} from {from} to {dest}", if dist == 1 { "" } else { "s" });
                for note in hazard_on_enter(w, rng, src, &[dest]) {
                    self.log += &format!(" {note}");
                }
            }
        }
    }

    /// HOOKS1 4.3 steps 1-2: one damaging hit of this attack meant for `f`. The `ally_hit` triggers (rule triggers) may
    /// move part of it or all of it to a friend (8.4: the same crit roll and factors, each through its own armour and
    /// guard; the holder never drops below 1 HP, the rest goes back to `f`). Without a watch it is today's hit: one crit
    /// draw, then the damage. Returns the unit the hit lands on (`f`, or the holder of an intercept), the hit and the
    /// damage dealt to that unit.
    #[allow(clippy::too_many_arguments)]
    fn land(&mut self, w: &mut World, rng: &mut GodotRng, src: UnitId, f: UnitId, ranged: bool, offhand: bool, mult: f64, cover: f64, skip: &[UnitId]) -> (UnitId, Hit, i32) {
        let pierce = self.pierces(w);
        let claim = match self.ally.as_mut() {
            Some(watch) => crate::trigger::ally_hit(w, watch, src, f, ranged, &self.family, skip),
            None => None,
        };
        // HOOKS1 IG1: a guard-bypassing hit leaves out the guard part of the guard-and-cover term (cover stays)
        let include_guard = !self.ignore_guard;
        let Some(r) = claim else {
            self.guard_bypassed |= self.ignore_guard && w.units[f].guard > 0.0;
            // one crit draw, then the damage (roll_hit_with's order)
            let input = HitInput { ranged, offhand, skill_mult: mult, crit_roll: Some(rng.randf()), cover, ignore_armor: pierce, include_guard, ..HitInput::default() };
            let hit = damage::compute(&w.units[src], &w.units[f], input);
            let taken = w.damage(f, hit.final_dmg, true);
            return (f, hit, taken);
        };
        let input = HitInput { ranged, offhand, skill_mult: mult, crit_roll: Some(rng.randf()), cover, ignore_armor: pierce, include_guard, ..HitInput::default() };
        let own = damage::compute(&w.units[src], &w.units[f], input);
        let h = r.holder;
        self.guard_bypassed |= self.ignore_guard && (w.units[f].guard > 0.0 || w.units[h].guard > 0.0);
        let theirs = damage::compute(&w.units[src], &w.units[h], input);
        let whole = r.pct >= 100;
        let h_part = if whole { theirs.final_dmg } else { crate::trigger::part(theirs.final_dmg, r.pct) };
        let h_take = h_part.min((w.units[h].hp - 1).max(0));
        let back = h_part - h_take;
        let h_dealt = if h_take > 0 { w.damage(h, h_take, true) } else { 0 };
        let (hname, fname) = (w.units[h].name.clone(), w.units[f].name.clone());
        let mut note = if whole { format!("{hname} takes the hit meant for {fname}: {h_dealt} damage") } else { format!("{hname} takes {}% of the hit meant for {fname}: {h_dealt} damage", r.pct) };
        if whole {
            if back > 0 {
                let b = w.damage(f, back, true);
                self.struck.push(crate::trigger::Struck { target: f, damage: b, ranged });
                note += &format!("; {b} goes back to {fname}");
            }
            if let Some(watch) = self.ally.as_mut() {
                watch.note(&r, note);
            }
            return (h, theirs, h_dealt);
        }
        if back > 0 {
            note += &format!("; {back} more goes back to {fname}");
        }
        if let Some(watch) = self.ally.as_mut() {
            watch.note(&r, note);
        }
        if h_dealt > 0 {
            self.struck.push(crate::trigger::Struck { target: h, damage: h_dealt, ranged });
        }
        let dealt = w.damage(f, crate::trigger::part(own.final_dmg, 100 - r.pct) + back, true);
        (f, own, dealt)
    }

    fn set_attack_result(&mut self, dmg: i32, crit: bool, tgt: Option<UnitId>) {
        if let CmdKind::Attack { last_damage, last_crit, last_target, .. } = &mut self.kind {
            *last_damage = dmg;
            *last_crit = crit;
            *last_target = tgt;
        }
    }

    /// RULESET1_INTENT MV8: after the impact, a carried retreat moves the unit to its planned tile (re-picked when it no
    /// longer works, skipped when none does) within the skill's range (INPUT1 MR14). Classic commands carry nothing.
    pub fn after_impact(&mut self, w: &mut World, rng: &mut GodotRng) {
        let Carry::Retreat(cell) = self.carry else { return };
        let src = self.source;
        let target = match &self.kind {
            CmdKind::Attack { target, .. } => effective_target(w, src, *target),
            _ => None,
        };
        let note = carried_retreat(w, rng, src, target, self.carry_range, cell);
        if !note.is_empty() {
            self.log = if self.log.is_empty() { note } else { format!("{} {note}", self.log) };
        }
    }

    /// The unit this command acts on, when it is another unit (RULESET1_INTENT TG4): an attack's target (or lock), a
    /// support skill's or item's target other than the user. Areas aim at tiles and never fail.
    pub fn acted_on(&self, w: &World) -> Option<UnitId> {
        let t = match &self.kind {
            CmdKind::Attack { target, .. } => effective_target(w, self.source, *target),
            CmdKind::Support { target, .. } | CmdKind::Item { target, .. } => Some(*target),
            _ => None,
        };
        t.filter(|t| *t != self.source)
    }

    /// RULESET1_INTENT TG4: the command becomes Rest blocks (no AP, no stance, no move); its blocks stay booked.
    pub fn into_rest(&mut self) {
        self.kind = CmdKind::Rest { recover_ap: 0 };
        self.name = "Rest".into();
        self.ap_cost = 0;
        self.stance_switch_cost = 0;
        self.stance_on_start = None;
        self.stance_on_end = None;
        self.carry = Carry::None;
        self.priority = 0;
        self.status_effect.clear();
        self.exploit.clear();
        self.disrupt_break = false;
        self.est_damage = 0;
        self.impact = crate::poise::Impact::default();
        self.forced = None;
        self.hazard = None;
        self.triggers.clear();
        self.riders.clear();
        self.hp_cost = 0;
        self.ignore_guard = false;
        self.meters.clear();
    }

    /// After one damaging hit: stance break (Offensive / Defensive target to Tactical), then exploit consume.
    fn after_hit(&mut self, w: &mut World, t: UnitId, taken: i32, matched: &(String, f64)) {
        let tu = &mut w.units[t];
        if self.disrupt_break && taken > 0 && tu.is_active() && matches!(tu.stance, Stance::Offensive | Stance::Defensive) {
            tu.set_stance(Stance::Tactical);
            tu.guard = 0.0;
            self.broken_targets.push(t);
        }
        // HOOKS1 EX1: an execute match names no status, so nothing is consumed
        if self.exploit_consume && !matched.0.is_empty() && matched.0 != skill::EXECUTE {
            status::remove_active(&mut w.units[t], &matched.0);
        }
    }
}

/// S1.2-S4: a multi-round guard (`guard_rounds` 2 or more) queues the full-round `guard` status for its later rounds at
/// `fraction` (AUDIT-11: `skill::guard_later_mitigation`) in percent capped at the row's maximum; a fraction below the
/// row's minimum queues nothing (it is not raised to the minimum). True when it queued. The battle's Defend impact and
/// the Dojo's Guard (AUDIT-12) share it.
pub fn queue_later_guard(u: &mut Unit, guard_rounds: i32, fraction: f64, source: &str) -> bool {
    let row = status::row_of(status::GUARD);
    let pct = roundi(fraction * 100.0).min(row.map_or(0, |r| r.magnitude_max));
    if guard_rounds < 2 || !row.is_some_and(|r| pct >= r.magnitude_min) {
        return false;
    }
    status::queue(u, status::GUARD, guard_rounds - 1, pct, source);
    true
}

/// One hit with a crit roll drawn from the battle RNG (DamageCalculator.compute with the resolver's rng).
pub fn roll_hit(w: &World, rng: &mut GodotRng, a: UnitId, t: UnitId, ranged: bool, offhand: bool, skill_mult: f64, cover: f64) -> Hit {
    roll_hit_with(w, rng, a, t, ranged, offhand, skill_mult, cover, false)
}

/// `roll_hit` that may skip the armour term (RULESET1 A2: `ignore_armor`; the same single crit draw).
#[allow(clippy::too_many_arguments)]
pub fn roll_hit_with(w: &World, rng: &mut GodotRng, a: UnitId, t: UnitId, ranged: bool, offhand: bool, skill_mult: f64, cover: f64, ignore_armor: bool) -> Hit {
    let roll = rng.randf();
    damage::compute(&w.units[a], &w.units[t], HitInput { ranged, offhand, skill_mult, crit_roll: Some(roll), cover, ignore_armor, ..HitInput::default() })
}

fn effective_target(w: &World, src: UnitId, target: Option<UnitId>) -> Option<UnitId> {
    target.or(w.units[src].active_target)
}

/// The attack's reach band: the command's own, else the unit's (ranged 2..range, melee 1).
pub fn reach_band(w: &World, src: UnitId, min_range: i32, max_range: i32) -> (i32, i32) {
    let u = &w.units[src];
    (if min_range > 0 { min_range } else { u.min_attack_range() }, if max_range > 0 { max_range } else { u.attack_range })
}

/// Whether `t` is in the band from where `src` stands (INPUT1 MR10: `diag` counts a diagonal neighbour as 1).
fn in_reach(w: &World, src: UnitId, t: UnitId, band: (i32, i32), diag: bool) -> bool {
    if !w.has(src) || !w.has(t) {
        return false;
    }
    let d = round_intent::band_dist(diag, w.pos(src), w.pos(t));
    d >= band.0 && d <= band.1
}

/// Cells around `target` within the band that a unit could stand on (walkable, not an exit, not blocked; INPUT1 MR10:
/// `diag` counts a diagonal neighbour as 1).
fn band_goals(w: &World, target_pos: V2, band: (i32, i32), blocked: &Cells, diag: bool) -> Cells {
    let mut goals = Cells::new();
    let hi = band.1;
    for dx in -hi..=hi {
        for dy in -hi..=hi {
            let cell = target_pos + V2::new(dx, dy);
            let d = round_intent::band_dist(diag, target_pos, cell);
            if d >= band.0 && d <= hi && w.grid.is_valid(cell) && w.grid.is_walkable(cell) && !w.grid.is_exit(cell) && !blocked.contains(&cell) {
                goals.insert(cell);
            }
        }
    }
    goals
}

fn others_blocked(w: &World, src: UnitId, active_only: bool) -> Cells {
    let mut blocked = Cells::new();
    for (u, p) in w.grid.positions.iter().enumerate() {
        if let Some(p) = p {
            if u != src && (!active_only || w.units[u].is_active()) {
                blocked.insert(*p);
            }
        }
    }
    blocked
}

/// Gap-close: route to the nearest tile in reach within `dash_tiles` cost; hazards entered roll their status.
fn dash(w: &mut World, rng: &mut GodotRng, src: UnitId, tgt: Option<UnitId>, band: (i32, i32), dash_tiles: i32, dashed: &mut bool) -> String {
    let Some(t) = tgt else { return String::new() };
    if !w.units[t].is_active() || !w.units[src].is_active() || in_reach(w, src, t, band, false) {
        return String::new();
    }
    let name = w.units[src].name.clone();
    if w.units[src].has_status("root") {
        return format!("[Dash Blocked] {name} is rooted and can't dash at {}.", w.units[t].name);
    }
    let from = w.pos(src);
    let blocked = others_blocked(w, src, true);
    let goals = band_goals(w, w.pos(t), band, &blocked, false);
    let route = path::find_path(&w.grid, from, &goals, &blocked, false, &Hooks::default());
    if route.is_empty() || !dash_fits(&w.grid, from, &route, dash_tiles) {
        let why = if route.is_empty() { "no route".to_string() } else { format!("{} tiles, move cost {}, dash covers {dash_tiles}", route.len(), path::path_cost(&w.grid, from, &route)) };
        return format!("[Dash Blocked] {name} can't reach {} ({why}).", w.units[t].name);
    }
    let mut entered = Vec::new();
    for d in route {
        if !w.move_unit(src, d) {
            break;
        }
        entered.push(w.pos(src));
        // HOOKS1 GH6: a trap stops the dash on its tile (rule ground_hazards; never in classic)
        if crate::ground::springs(w, src, w.pos(src)).is_some() {
            break;
        }
    }
    let to = w.pos(src);
    if to == from {
        return format!("[Dash Blocked] {name} couldn't move toward {}.", w.units[t].name);
    }
    round_intent::note_moved(w, src);
    *dashed = true;
    let mut line = format!("[Dash] {name} dashes at {}: {from} to {to}", w.units[t].name);
    for note in hazard_on_enter(w, rng, src, &entered) {
        line += &format!(" {note}");
    }
    line
}

pub fn dash_fits(grid: &Grid, from: V2, route: &[V2], dash_tiles: i32) -> bool {
    dash_tiles > 0 && path::blocks_needed(grid, from, route, dash_tiles) <= 1
}

/// The chase (AS1; INPUT1 MR16): `follow`, then its log line ("" when the unit did not move). With `root_stops_chase`
/// root stops it (MR16, CLN-45; the classic row keeps its chase rules).
fn chase_line(w: &mut World, src: UnitId, t: UnitId, band: (i32, i32), diag: bool, follow_tiles: i32) -> String {
    if w.ruleset.root_stops_chase() && w.units[src].has_status("root") {
        return String::new();
    }
    let from = w.pos(src);
    let too_close = round_intent::band_dist(diag, from, w.pos(t)) < band.0;
    follow(w, src, t, band, diag, follow_tiles);
    let to = w.pos(src);
    if to == from {
        return String::new();
    }
    let verb = if too_close { "backs off from" } else { "closes in on" };
    let ap = if w.pays_ap(src) { "1 AP" } else { "no AP" };
    format!("[Chase] {} {verb} {}: {} to {} ({ap})", w.units[src].name, w.units[t].name, from, to)
}

/// Steps toward a target that moved out of reach, 1 AP per tile (AP debt allowed).
fn follow(w: &mut World, src: UnitId, t: UnitId, band: (i32, i32), diag: bool, follow_tiles: i32) {
    let mut left = follow_tiles;
    while left > 0 && !in_reach(w, src, t, band, diag) {
        let blocked = others_blocked(w, src, false);
        let goals = band_goals(w, w.pos(t), band, &blocked, diag);
        let route = path::find_path(&w.grid, w.pos(src), &goals, &blocked, false, &Hooks::default());
        if route.is_empty() || !w.move_unit(src, route[0]) {
            return;
        }
        if w.pays_ap(src) {
            w.units[src].ap -= 1;
        }
        round_intent::note_moved(w, src);
        left -= 1;
    }
}

/// RULESET1_INTENT MV3: the power factor of a flank hit (`flank_power_mult`; 1.0 otherwise).
fn flank_power(w: &World, flank: bool) -> f64 {
    if flank {
        w.ruleset.numbers.flank_power_mult
    } else {
        1.0
    }
}

/// Walks `route` from the unit's tile (stops at the first refused step, or on a trap it springs: HOOKS1 GH6); the tiles
/// entered.
fn step_along(w: &mut World, src: UnitId, route: &[V2]) -> Vec<V2> {
    let mut entered = Vec::new();
    for d in route {
        if !w.move_unit(src, *d) {
            break;
        }
        entered.push(w.pos(src));
        if crate::ground::springs(w, src, w.pos(src)).is_some() {
            break;
        }
    }
    entered
}

/// INPUT1 MR8 (RULESET1_INTENT MV4, MV6): the dash of a skill with a position at its start (the wind-up's tick): the
/// position tiles are read around the target's tile now (its facing re-read, MV1) with the budget `range`, plus 1 when
/// the target left `planned` (MR16's chase, that tile at the chase's cost). The stored tile `cell` is kept while it is
/// in the first tier with a tile in the budget, else that tier's best (MR4); a unit already on a position tile stays
/// (MR6). (a) a tile in the budget: the unit dashes there (hazards on entry); (b) open tiles but none in the budget: no
/// move (`[Dash Blocked]`; the hit lands only if the target is adjacent, MR10); (c) no open tile: protected (the caller
/// logs it and the impact misses). Returns the line, the board event and whether the flank is protected.
#[allow(clippy::too_many_arguments)]
fn carried_engage(w: &mut World, rng: &mut GodotRng, src: UnitId, tgt: Option<UnitId>, cell: V2, position: rpgkit_data::Position, range: i32, planned: V2, diag: bool) -> (String, Option<MoveKind>, bool) {
    let none = (String::new(), None, false);
    let Some(t) = tgt.filter(|t| w.units[*t].is_active() && w.has(*t)) else { return none };
    if !w.units[src].is_active() || !w.has(src) {
        return none;
    }
    let name = w.units[src].name.clone();
    let from = w.pos(src);
    let tp = w.pos(t);
    let chase = planned != NO_CELL && planned != tp;
    let budget = range + i32::from(chase);
    let rooted = w.units[src].has_status("root");
    let blocked = others_blocked(w, src, true);
    let open = round_intent::positions(w, t, src, position, from, diag, &blocked);
    if open.protected() {
        return (String::new(), None, true);
    }
    // MR6: a unit already on a position tile (a follow-up) hits from there
    if cell == from && open.contains(from) {
        return none;
    }
    let in_budget = |c: V2| -> Option<Vec<V2>> {
        if c == from {
            return Some(Vec::new());
        }
        if rooted || c.dist(from) > budget {
            return None;
        }
        let goals: Cells = [c].into_iter().collect();
        let route = path::find_path(&w.grid, from, &goals, &blocked, false, &Hooks::default());
        (!route.is_empty() && dash_fits(&w.grid, from, &route, budget)).then_some(route)
    };
    let mut tiles: Vec<(V2, Vec<V2>)> = open.preferred.iter().filter_map(|&c| in_budget(c).map(|r| (c, r))).collect();
    let preferred = !tiles.is_empty();
    if !preferred {
        tiles = open.fallback.iter().filter_map(|&c| in_budget(c).map(|r| (c, r))).collect();
    }
    round_intent::rank_tiles(w, src, tp, w.units[src].attack_range > 1, round_intent::Aim::Toward, &mut tiles);
    if !preferred {
        tiles.sort_by_key(|(_, route)| route.len());
    }
    let Some((dest, route)) = round_intent::pick(cell, &tiles) else {
        if rooted {
            return (format!("[Dash Blocked] {name} is rooted and can't move on {}.", w.units[t].name), None, false);
        }
        return (format!("[Dash Blocked] {name} can't reach {}'s flank", w.units[t].name), None, false);
    };
    if dest == from {
        return none;
    }
    let cost = path::path_cost(&w.grid, from, &route);
    let entered = step_along(w, src, &route);
    let to = w.pos(src);
    if to == from {
        return (format!("[Dash Blocked] {name} couldn't move toward {}.", w.units[t].name), None, false);
    }
    round_intent::note_moved(w, src);
    let mut line = format!("[Dash] {name} dashes at {}: {from} to {to}", w.units[t].name);
    // MR16: the extra tile of a chase costs a party unit 1 AP
    if chase && cost > range {
        if w.pays_ap(src) {
            w.units[src].ap -= 1;
        }
        line += if w.pays_ap(src) { " (chase 1 AP)" } else { " (chase no AP)" };
    }
    for note in hazard_on_enter(w, rng, src, &entered) {
        line += &format!(" {note}");
    }
    (line, Some(MoveKind::Dash), false)
}

/// RULESET1_INTENT MV8: a Disengage's carried retreat after the hit: to the stored tile while it is free and within
/// `dash_tiles` in one block, else the PL4 re-pick away from the target; skipped when none is left.
fn carried_retreat(w: &mut World, rng: &mut GodotRng, src: UnitId, tgt: Option<UnitId>, dash_tiles: i32, cell: V2) -> String {
    if !w.units[src].is_active() || !w.has(src) {
        return String::new();
    }
    let name = w.units[src].name.clone();
    if w.units[src].has_status("root") {
        return format!("[Retreat Blocked] {name} is rooted.");
    }
    let from = w.pos(src);
    let blocked = others_blocked(w, src, true);
    let tp = tgt.filter(|t| w.has(*t)).map_or(from, |t| w.pos(t));
    let mut tiles: Vec<(V2, Vec<V2>)> = round_intent::carry_tiles(w, from, dash_tiles, &blocked).into_iter().filter(|(c, _)| *c != from).collect();
    round_intent::rank_tiles(w, src, tp, w.units[src].attack_range > 1, round_intent::Aim::Away, &mut tiles);
    let Some((_, route)) = round_intent::pick(cell, &tiles) else {
        return format!("[Retreat Blocked] {name} has nowhere to fall back.");
    };
    let entered = step_along(w, src, &route);
    let to = w.pos(src);
    if to == from {
        return format!("[Retreat Blocked] {name} couldn't fall back.");
    }
    round_intent::note_moved(w, src);
    let mut line = format!("[Retreat] {name} falls back from {from} to {to}");
    for note in hazard_on_enter(w, rng, src, &entered) {
        line += &format!(" {note}");
    }
    line
}

/// RULESET1-U slice 5 (a revealed walk on the board): the tiles a planned walk of `src` passes through from
/// `expected_start` (its planned start; NO_CELL: where it stands) along the steps `planned`, as `walk` would take them
/// now: the start, then each tile it enters, stopping before a step `World::step_from` refuses, on an exit it leaves by
/// and on a trap it springs. Nothing changes and nothing is drawn at random.
pub fn walk_preview(w: &World, src: UnitId, planned: &[V2], expected_start: V2) -> Vec<V2> {
    if !w.units[src].is_active() || !w.has(src) {
        return Vec::new();
    }
    let mut at = if expected_start != NO_CELL { expected_start } else { w.pos(src) };
    let mut tiles = vec![at];
    for d in planned {
        match w.step_from(src, at, *d) {
            crate::world::StepTo::Blocked => break,
            crate::world::StepTo::Leaves => {
                if w.grid.is_valid(at + *d) {
                    tiles.push(at + *d);
                }
                break;
            }
            crate::world::StepTo::Enters(next) => {
                tiles.push(next);
                at = next;
                if crate::ground::springs(w, src, next).is_some() {
                    break;
                }
            }
        }
    }
    tiles
}

/// WalkCommand.execute: follows the planned steps (re-paths when not where the plan expected); unwalked tiles are
/// refunded as AP. Returns (tiles moved, cut short).
/// WalkCommand.execute: (tiles moved, cut short, what blocked it, hazard notes).
fn walk(w: &mut World, rng: &mut GodotRng, src: UnitId, planned: &[V2], expected_start: V2, goal: V2) -> (i32, bool, String, Vec<String>) {
    if !w.has(src) {
        return (0, true, String::new(), Vec::new());
    }
    let start = w.pos(src);
    let route: Vec<V2> = if expected_start != NO_CELL && goal != NO_CELL && start != expected_start {
        let blocked = others_blocked(w, src, false);
        let goals: Cells = [goal].into_iter().collect();
        let full = path::find_path(&w.grid, start, &goals, &blocked, w.grid.is_exit(goal), &Hooks::default());
        let budget = w.units[src].move_per_block().max(path::path_cost(&w.grid, expected_start, planned));
        path::fit_blocks(&w.grid, start, &full, budget, 1).into_iter().take(planned.len()).collect()
    } else {
        planned.to_vec()
    };
    let mut moved = 0;
    let mut cut = false;
    let mut blocked_by = String::new();
    let mut trapped = String::new();
    let mut entered = Vec::new();
    for d in route {
        let next = w.pos(src) + d;
        if !w.units[src].is_active() || !w.move_unit(src, d) {
            cut = true;
            blocked_by = describe_blocker(w, next);
            break;
        }
        moved += 1;
        if w.grid.pos(src).is_some() {
            entered.push(next);
        }
        // HOOKS1 GH6: a trap stops the walk on its tile; the rest of the path is dropped (rule ground_hazards)
        if let Some(trap) = crate::ground::springs(w, src, next) {
            trapped = format!("the {trap}");
            break;
        }
    }
    let notes = hazard_on_enter(w, rng, src, &entered);
    if w.units[src].is_withdrawn {
        return (moved, cut, blocked_by, notes);
    }
    if (moved as usize) < planned.len() {
        let pays = w.pays_ap(src);
        let u = &mut w.units[src];
        if pays {
            u.ap = u.max_ap.min(u.ap + (planned.len() as i32 - moved));
        }
        cut = true;
        if blocked_by.is_empty() {
            blocked_by = if trapped.is_empty() { "no route".into() } else { trapped };
        }
    }
    (moved, cut, blocked_by, notes)
}

/// WalkCommand._describe_blocker: the unit there, the arena edge, the terrain, else an obstacle.
fn describe_blocker(w: &World, cell: V2) -> String {
    if let Some(o) = w.occupant(cell) {
        return w.units[o].name.clone();
    }
    if !w.grid.is_valid(cell) {
        return "the arena edge".into();
    }
    if !w.grid.is_walkable(cell) {
        return terrain_name(w, cell);
    }
    // ENCOUNTER2 H3: an exit a hold escape's gate kept shut (or its holder may not take)
    if w.hold.is_some() && w.grid.is_exit(cell) {
        return "the shut gate".into();
    }
    "an obstacle".into()
}

/// CoverRules.cover_for: the target tile's cover, or a blocking neighbour toward the attacker (ranged hits only).
pub fn cover_for(grid: &Grid, attacker: V2, target: V2, ranged: bool) -> f64 {
    if !ranged || !grid.is_valid(target) {
        return 0.0;
    }
    let mut best = grid.cover_guard(target);
    let dx = (attacker.x - target.x).signum();
    let dy = (attacker.y - target.y).signum();
    let cells: Vec<V2> = if dx == 0 && dy == 0 {
        vec![]
    } else if dx == 0 || dy == 0 {
        vec![target + V2::new(dx, dy)]
    } else {
        vec![target + V2::new(dx, 0), target + V2::new(0, dy)]
    };
    for c in cells {
        if grid.is_valid(c) && !grid.is_walkable(c) {
            best = best.max(grid.cover_guard(c));
        }
    }
    best
}

pub fn cover_between(w: &World, a: UnitId, t: UnitId, ranged: bool) -> f64 {
    if !w.has(a) || !w.has(t) {
        return 0.0;
    }
    cover_for(&w.grid, w.pos(a), w.pos(t), ranged)
}

/// HazardRules.on_enter: each enter-hazard status of the entered cells rolls once (the first tile of a status wins).
pub fn hazard_on_enter(w: &mut World, rng: &mut GodotRng, u: UnitId, cells: &[V2]) -> Vec<String> {
    let mut lines = Vec::new();
    if !w.units[u].is_active() {
        return lines;
    }
    let mut seen: Vec<String> = Vec::new();
    let mut ground_seen: Vec<String> = Vec::new();
    for &cell in cells {
        if let Some(h) = w.grid.hazard_at(cell).cloned() {
            if (h.on == "enter" || h.on == "both") && !seen.contains(&h.status) {
                seen.push(h.status.clone());
                lines.extend(hazard_roll(w, rng, u, &h, cell));
            }
        }
        // HOOKS1 GH4, GH5: the tile's ground hazard after its terrain's (rule ground_hazards; none in classic)
        lines.extend(crate::ground::enter(w, rng, u, cell, &mut ground_seen));
    }
    lines
}

/// HazardRules._roll: the unit rolls the hazard's status; the note when it lands.
fn hazard_roll(w: &mut World, rng: &mut GodotRng, u: UnitId, h: &crate::grid::Hazard, cell: V2) -> Option<String> {
    if !status::try_inflict(&mut w.units[u], &h.status, h.potency, Some(rng), h.rounds, h.magnitude, "terrain") {
        return None;
    }
    Some(format!("[Hazard] {} is {} by {}", w.units[u].name, status::display_name(&h.status), terrain_name(w, cell)))
}

/// Terrain.display_name of the tile's terrain (the catalog's name, else Floor).
pub fn terrain_name(w: &World, cell: V2) -> String {
    w.cat.terrain.get(w.grid.terrain_id(cell)).map(|r| aegis_core::value::string(r, "name", "Floor")).unwrap_or_else(|| "Floor".into())
}

/// HazardRules.at_round_end: every active unit on a round-end hazard tile rolls it, in `units` order.
pub fn hazard_round_end(w: &mut World, rng: &mut GodotRng, units: &[UnitId]) -> Vec<String> {
    let mut lines = Vec::new();
    for &u in units {
        if !w.units[u].is_active() || !w.has(u) {
            continue;
        }
        let cell = w.pos(u);
        let Some(h) = w.grid.hazard_at(cell).cloned() else { continue };
        if h.on == "round_end" || h.on == "both" {
            lines.extend(hazard_roll(w, rng, u, &h, cell));
        }
    }
    lines
}

/// RiposteRules.counter: a riposting defender answers an adjacent melee hit it survived, once per attacker per round
/// (diagonally adjacent too under `melee_diagonal` all, INPUT1 MR10).
pub fn riposte_counter(w: &mut World, rng: &mut GodotRng, attacker: UnitId, defender: UnitId, ranged: bool) -> Option<Reaction> {
    if ranged || attacker == defender {
        return None;
    }
    let d = &w.units[defender];
    if !d.has_status("riposte") || !d.is_alive() || !w.units[attacker].is_active() || d.riposted.contains(&attacker) {
        return None;
    }
    // INPUT1 MR10, GD3: under `melee_diagonal` all the counter answers a diagonal melee hit too
    if !w.has(attacker) || !w.has(defender) || round_intent::melee_dist(w, defender, None, w.pos(defender), w.pos(attacker)) != 1 {
        return None;
    }
    w.units[defender].riposted.push(attacker);
    let du = &w.units[defender];
    // S1.2-S3: the counter multiplier is the ruleset number `counter_mult`
    let mult = w.ruleset.numbers.counter_mult * du.weaken_factor() * du.empower_factor();
    let hit = roll_hit(w, rng, defender, attacker, false, false, mult, 0.0);
    let taken = w.damage(attacker, hit.final_dmg, true);
    Some(Reaction { defender, attacker, damage: taken, crit: hit.is_crit, defeated: !w.units[attacker].is_alive() })
}

// ---- SupportRules ---------------------------------------------------------------------------------------------

pub fn heal_amount(w: &World, caster: UnitId, skill_id: SkillId) -> i32 {
    let s = &w.cat.skills[skill_id];
    if !skill::heals(s) {
        return 0;
    }
    // U4.2-S4 (FB-30): the heal pipeline (base, then the caster's potency; one rounding, floor 0)
    rpgkit_rules::damage::heal(&w.units[caster], s)
}

pub fn effective_heal(target: &Unit, amount: i32) -> i32 {
    if !target.is_alive() {
        return 0;
    }
    amount.clamp(0, target.max_hp - target.hp)
}

pub fn cleansable(target: &Unit, s: &rpgkit_data::SkillDef) -> Vec<&'static str> {
    skill::cleanse_list(s).into_iter().filter(|st| target.st.has_any(st)).collect()
}

/// HOOKS1 HC1: the HP skill `s` costs (0 without the rule hp_cost; items pay none).
pub fn hp_cost_of(w: &World, s: &rpgkit_data::SkillDef) -> i32 {
    if w.ruleset.has(rpgkit_rules::ruleset::HP_COST) && !skill::is_item(s) {
        s.hp_cost.max(0)
    } else {
        0
    }
}

/// HOOKS1 HC1: why a unit at `hp` cannot use a skill costing `cost` HP ("" = it can): HP at or below the cost.
pub fn hp_cost_problem(hp: i32, cost: i32) -> String {
    if cost > 0 && hp <= cost {
        format!("HP {hp} <= cost {cost}")
    } else {
        String::new()
    }
}

/// HOOKS1 IG1: skill `s`'s hits skip the guard (the rule ignore_guard and the ATTACK flag `ignore_guard`).
pub fn ignores_guard(w: &World, s: &rpgkit_data::SkillDef) -> bool {
    s.ignore_guard && w.ruleset.has(rpgkit_rules::ruleset::IGNORE_GUARD) && !skill::is_item(s) && skill::kind(s) == skill::Kind::Attack
}

/// "" when `target` is legal for `skill` cast by `caster` (distance aside).
pub fn target_problem(w: &World, caster: UnitId, skill_id: SkillId, target: Option<UnitId>) -> String {
    let s = &w.cat.skills[skill_id];
    let Some(t) = target.filter(|t| w.units[*t].is_active()) else { return "Choose a friendly unit.".into() };
    if t == caster {
        return if s.target_mode == "ALLY" { "only on another ally".into() } else { String::new() };
    }
    if s.target_mode == "SELF" {
        return "self only".into();
    }
    if !w.friends_of(caster).contains(&t) {
        return "not an ally".into();
    }
    String::new()
}

// ---- ItemCommand shared rules (planner, AI, command) --------------------------------------------------------

/// ITEMUSE1 rule 3 (A1.2-S1): the AP an item use costs `c`: the ruleset's `item_ap` (AP debt allowed); nothing for a
/// unit that pays no AP (RULESET1 T11).
pub fn item_ap(w: &World, c: UnitId) -> i32 {
    if w.pays_ap(c) {
        w.ruleset.numbers.item_ap
    } else {
        0
    }
}

/// ItemCommand.pouch_item: the first consumable with `item_id` in the unit's battle pouch.
pub fn pouch_item<'a>(w: &'a World, c: UnitId, item_id: &str) -> Option<&'a ItemDef> {
    w.units[c].inventory.pouch.iter().find(|p| p.id == item_id)
}

pub fn pouch_count(w: &World, c: UnitId, item_id: &str) -> i32 {
    w.units[c].inventory.pouch.iter().filter(|p| p.id == item_id).count() as i32
}

/// Distinct pouch item ids in slot order.
pub fn pouch_ids(w: &World, c: UnitId) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for p in &w.units[c].inventory.pouch {
        if !out.contains(&p.id) {
            out.push(p.id.clone());
        }
    }
    out
}

/// ItemCommand.side_problem: "" when `user` may aim `item` at `target` by side (and reach 0 = self only).
pub fn item_side_problem(w: &World, user: UnitId, item: &ItemDef, target: Option<UnitId>) -> String {
    let Some(target) = target else { return format!("Pick a target for {}.", item.name) };
    let foe = w.foes_of(user).contains(&target);
    if item.aims_at_foes() {
        if !foe {
            return format!("{} is thrown at a foe.", item.name);
        }
    } else if foe {
        return format!("{} is used on yourself or an ally.", item.name);
    }
    if item.battle_reach() == 0 && target != user {
        return format!("{} is used on yourself only.", item.name);
    }
    String::new()
}

/// ItemCommand.problem: why the item can't be used now ("" = it can).
fn item_problem(w: &World, src: UnitId, target: UnitId, item: &ItemDef) -> String {
    if !w.units[src].is_active() {
        return format!("{} can't act", w.units[src].name);
    }
    if !w.units[target].is_active() {
        return format!("{} is down", w.units[target].name);
    }
    let in_reach = target == src || (w.has(src) && w.has(target) && w.dist(src, target) <= item.battle_reach());
    if !in_reach {
        return format!("{} is out of reach", w.units[target].name);
    }
    if pouch_item(w, src, &item.id).is_none() {
        return format!("none left in {}'s pouch", w.units[src].name);
    }
    String::new()
}

fn support_in_reach(w: &World, src: UnitId, target: UnitId, skill_id: SkillId) -> bool {
    if target == src {
        return true;
    }
    if !w.has(src) || !w.has(target) {
        return false;
    }
    let (lo, hi) = skill::support_reach(&w.cat.skills[skill_id], &w.units[src]);
    let d = w.dist(src, target);
    d >= lo && d <= hi
}

/// What SupportRules.apply did: HP healed, the status queued ("" = none), the statuses removed, the extras granted.
struct Supported {
    healed: i32,
    status: String,
    cleansed: Vec<String>,
    extras: Vec<String>,
}

/// SupportRules.apply: cleanse, heal, grant, extra grants.
fn support_apply(w: &mut World, caster: UnitId, target: UnitId, skill_id: SkillId) -> Supported {
    let s = w.cat.skills[skill_id].clone();
    let mut cleansed = Vec::new();
    for st in skill::cleanse_list(&s) {
        if status::remove(&mut w.units[target], st) {
            cleansed.push(st.to_string());
        }
    }
    let mut healed = 0;
    if skill::heals(&s) {
        let amount = heal_amount(w, caster, skill_id);
        let before = w.units[target].hp;
        w.heal(target, amount);
        healed = w.units[target].hp - before;
    }
    let name = w.units[caster].name.clone();
    if !s.grant_status.is_empty() {
        status::queue(&mut w.units[target], &s.grant_status, s.status_rounds, s.status_magnitude, &name);
    }
    let extras = status::grant_extras(&mut w.units[target], &s.extra_grants, false, &name);
    Supported { healed, status: s.grant_status.clone(), cleansed, extras }
}
