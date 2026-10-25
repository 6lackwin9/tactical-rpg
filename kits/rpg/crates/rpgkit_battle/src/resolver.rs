//! CombatResolver (scripts/combat/arena/combat_resolver.gd): the round timeline. Commands pay AP and enter their
//! stance when they START; their effect lands at the end of their last block, highest initiative first.
//! RULESET1 T1 (engine-only): an action lands at its impact block; recovery blocks booked after it (enemy-side units
//! under `enemy_moves`) keep the unit busy until `end`. In classic `impact == end`.

use crate::command::{hazard_round_end, CmdKind, Command};
use crate::world::{UnitId, World};
use aegis_core::rng::GodotRng;
use aegis_core::sort::godot_sort;
use rpgkit_rules::status;

#[derive(Debug, Clone)]
pub struct ScheduledAction {
    pub unit: UnitId,
    pub command: Command,
    pub start: i32,
    /// The block the action lands in (its last windup block): `start + block_cost - 1` (RULESET1 T1).
    pub impact: i32,
    /// The last block it occupies: `impact + recovery` (classic: `impact`).
    pub end: i32,
    pub cancelled: bool,
}

/// A combo an action resolved (CombatResolver.combo_resolved): "exploit", "break" or "flow".
#[derive(Debug, Clone, PartialEq)]
pub struct Combo {
    pub kind: &'static str,
    pub actor: UnitId,
    pub target: UnitId,
    /// The exploited status ("" for a break or a flow).
    pub status: String,
    pub skill: String,
    pub killed: bool,
}

/// One round-end status step (last_round_end_steps): bleed, poison and burn record the HP lost, regen the HP gained.
#[derive(Debug, Clone, PartialEq)]
pub struct RoundEndStep {
    pub unit: UnitId,
    pub status: &'static str,
    pub amount: i32,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct Resolver {
    pub scheduled: Vec<ScheduledAction>,
    pub registered: Vec<UnitId>,
    pub current_round: i32,
    pub total_blocks: i32,
    pub rng: GodotRng,
    pub log: Vec<String>,
    /// Combos resolved since the session last took them (in emit order).
    pub combos: Vec<Combo>,
    /// Start notes of commands (action index, message: a chase or dash) since the session last took them.
    pub notes: Vec<(usize, String)>,
    /// The last apply_round_end_effects call's steps, and how many calls there were (a new call = new steps).
    pub round_end_steps: Vec<RoundEndStep>,
    pub round_end_calls: u32,
}

impl Resolver {
    pub fn new(total_blocks: i32, seed: i64) -> Self {
        Self { scheduled: Vec::new(), registered: Vec::new(), current_round: 1, total_blocks, rng: GodotRng::from_i64(seed), log: Vec::new(), combos: Vec::new(), notes: Vec::new(), round_end_steps: Vec::new(), round_end_calls: 0 }
    }

    pub fn register(&mut self, u: UnitId) {
        if !self.registered.contains(&u) {
            self.registered.push(u);
        }
    }

    /// Books `command` from `start`: refused when it (recovery included, RULESET1 T2) does not fit the round or overlaps
    /// another live action of the unit (T3: a unit in recovery takes no other action).
    pub fn schedule(&mut self, unit: UnitId, command: Command, start: i32) -> bool {
        self.register(unit);
        let impact = start + command.block_cost - 1;
        let end = impact + command.recovery.max(0);
        if start < 1 || end > self.total_blocks {
            return false;
        }
        if self.scheduled.iter().any(|a| a.unit == unit && !a.cancelled && !(end < a.start || start > a.end)) {
            return false;
        }
        self.scheduled.push(ScheduledAction { unit, command, start, impact, end, cancelled: false });
        true
    }

    /// Resets block guards, starts the commands that begin now, raises running guards, and returns the indices of
    /// the actions that resolve this block, highest initiative first.
    pub fn prepare_block(&mut self, w: &mut World, block: i32) -> Vec<usize> {
        for &c in &self.registered {
            w.units[c].guard = 0.0;
        }
        self.begin_actions(w, block);
        for a in &self.scheduled {
            if !a.cancelled && w.units[a.unit].is_active() && block >= a.start && block <= a.end {
                if let CmdKind::Defend { mitigation, guard_broken, .. } = &a.command.kind {
                    if !guard_broken {
                        w.units[a.unit].guard = *mitigation;
                    }
                }
            }
        }
        let mut resolving: Vec<usize> = (0..self.scheduled.len()).filter(|&i| !self.scheduled[i].cancelled && self.scheduled[i].impact == block).collect();
        let inits: Vec<i32> = (0..self.scheduled.len()).map(|i| self.scheduled[i].command.initiative(w)).collect();
        godot_sort(&mut resolving, |a, b| inits[*a] > inits[*b]);
        resolving
    }

    /// Commands starting now pay AP (skill + stance switch; debt allowed; RULESET1 T11: not the units `pays_ap` spares),
    /// enter their stance and run on_start.
    fn begin_actions(&mut self, w: &mut World, block: i32) {
        for i in 0..self.scheduled.len() {
            let a = &self.scheduled[i];
            if a.cancelled || a.start != block || !w.units[a.unit].is_active() {
                continue;
            }
            let c = a.unit;
            let total = a.command.ap_cost + a.command.stance_switch_cost;
            if w.pays_ap(c) {
                w.units[c].ap -= total;
            }
            if let Some(s) = a.command.stance_on_start {
                if s != w.units[c].stance {
                    w.units[c].set_stance(s);
                }
            }
            let msg = self.scheduled[i].command.on_start(w, &mut self.rng);
            if !msg.is_empty() {
                self.notes.push((i, msg.clone()));
                self.log.push(msg);
            }
        }
    }

    pub fn execute_single(&mut self, w: &mut World, idx: usize) {
        if self.scheduled[idx].cancelled {
            return;
        }
        let unit = self.scheduled[idx].unit;
        if !w.units[unit].is_active() {
            self.scheduled[idx].cancelled = true;
            return;
        }
        self.scheduled[idx].command.execute(w, &mut self.rng);
        let cmd = &self.scheduled[idx].command;
        if !cmd.log.is_empty() {
            self.log.push(cmd.log.clone());
        }
        self.apply_combo(w, idx);
        let inactive: Vec<UnitId> = self.registered.iter().copied().filter(|c| !w.units[*c].is_active()).collect();
        for c in inactive {
            for a in self.scheduled.iter_mut() {
                if a.unit == c {
                    a.cancelled = true;
                }
            }
        }
    }

    /// Stance breaks end the broken units' running guards; a flow move then enters its end stance for free. The combos
    /// are recorded: exploits, then breaks, then the flow.
    fn apply_combo(&mut self, w: &mut World, idx: usize) {
        let at = self.scheduled[idx].impact;
        let broken = self.scheduled[idx].command.broken_targets.clone();
        let (actor, skill) = (self.scheduled[idx].unit, self.scheduled[idx].command.name.clone());
        for (victim, status) in &self.scheduled[idx].command.exploited {
            self.combos.push(Combo { kind: "exploit", actor, target: *victim, status: status.clone(), skill: skill.clone(), killed: !w.units[*victim].is_alive() });
        }
        for &b in &broken {
            self.combos.push(Combo { kind: "break", actor, target: b, status: String::new(), skill: skill.clone(), killed: !w.units[b].is_alive() });
        }
        for b in broken {
            for other in self.scheduled.iter_mut() {
                let running = other.start <= at && other.end >= at;
                if other.unit == b && running && !other.cancelled {
                    if let CmdKind::Defend { guard_broken, .. } = &mut other.command.kind {
                        *guard_broken = true;
                    }
                }
            }
        }
        let a = &self.scheduled[idx];
        if let Some(s) = a.command.stance_on_end {
            if !a.command.failed && w.units[a.unit].is_active() {
                self.combos.push(Combo { kind: "flow", actor, target: actor, status: String::new(), skill, killed: false });
                w.units[a.unit].set_stance(s);
            }
        }
    }

    /// Round end per unit in registered order: bleed, poison, burn, regen; then round-end hazards.
    pub fn apply_round_end_effects(&mut self, w: &mut World) {
        self.round_end_steps.clear();
        self.round_end_calls += 1;
        for &c in &self.registered.clone() {
            for st in ["bleed", "poison", "burn"] {
                let u = &w.units[c];
                if !u.is_active() || !u.has_status(st) {
                    continue;
                }
                let dmg = match st {
                    "bleed" => status::bleed_damage(u),
                    "poison" => status::poison_damage(u),
                    _ => status::burn_damage(u),
                };
                if st == "poison" && dmg <= 0 {
                    continue;
                }
                let before = u.hp;
                w.damage(c, dmg, true);
                self.step(w, c, st, before - w.units[c].hp);
            }
            if w.units[c].is_active() && w.units[c].has_status("regen") {
                let u = &w.units[c];
                let before = u.hp;
                let amount = status::regen_amount(u).min(u.max_hp - u.hp);
                w.heal(c, amount);
                let healed = w.units[c].hp - before;
                if healed > 0 {
                    self.step(w, c, "regen", healed);
                }
            }
        }
        let units = self.registered.clone();
        let lines = hazard_round_end(w, &mut self.rng, &units);
        self.log.extend(lines);
    }

    /// _record_round_end: the step row with the status's recorded source.
    fn step(&mut self, w: &World, c: UnitId, status: &'static str, amount: i32) {
        let source = w.units[c].st.sources.get(status).cloned().unwrap_or_default();
        self.round_end_steps.push(RoundEndStep { unit: c, status, amount, source });
    }

    pub fn finish_round(&mut self, w: &mut World) {
        for &c in &self.registered {
            if w.units[c].is_active() {
                w.units[c].tick_round_buffs();
            }
        }
        self.scheduled.clear();
        self.current_round += 1;
    }
}
