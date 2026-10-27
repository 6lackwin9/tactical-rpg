//! CombatResolver (scripts/combat/arena/combat_resolver.gd): the round timeline. Commands pay AP and enter their
//! stance when they START; their effect lands at the end of their last block, highest initiative first.
//! RULESET1 T1 (engine-only): an action lands at its impact block; recovery blocks booked after it (enemy-side units
//! under `enemy_moves`) keep the unit busy until `end`. In classic `impact == end`.

use crate::command::{hazard_round_end, CmdKind, Command};
use crate::round_intent;
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
    /// The action has landed (executed at its impact block); RULESET1 P5: no longer in windup.
    pub landed: bool,
    /// RULESET1 P7: cancelled by its unit's break (rule poise).
    pub broken: bool,
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
    /// HOOKS1 8.1-8.4: the events and triggers state (rule `triggers`; untouched in classic).
    pub triggers: crate::trigger::State,
}

impl Resolver {
    pub fn new(total_blocks: i32, seed: i64) -> Self {
        Self { scheduled: Vec::new(), registered: Vec::new(), current_round: 1, total_blocks, rng: GodotRng::from_i64(seed), log: Vec::new(), combos: Vec::new(), notes: Vec::new(), round_end_steps: Vec::new(), round_end_calls: 0, triggers: Default::default() }
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
        self.scheduled.push(ScheduledAction { unit, command, start, impact, end, cancelled: false, landed: false, broken: false });
        true
    }

    /// Resets block guards, starts the commands that begin now, raises running guards, and returns the indices of
    /// the actions that resolve this block, highest initiative first.
    pub fn prepare_block(&mut self, w: &mut World, block: i32) -> Vec<usize> {
        for &c in &self.registered {
            w.units[c].guard = 0.0;
        }
        // RULESET1_INTENT MV1: every unit faces its round target, re-read each block
        round_intent::update_facings(w);
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
        // HOOKS1 8.3, 8.1: held guards, then the Defends' `guard_start` triggers (rule triggers; nothing in classic)
        crate::trigger::on_block_start(self, w, block);
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
            // RULESET1_INTENT TG4: a unit holding an intent whose target is gone before a command on it starts rests
            // through that command and every later one this round (no AP)
            if round_intent::intent_of(w, c).is_some() && a.command.acted_on(w).is_some_and(|t| !w.units[t].is_active()) {
                let gone = w.units[a.command.acted_on(w).unwrap()].name.clone();
                for later in self.scheduled.iter_mut().filter(|x| x.unit == c && !x.cancelled && x.start >= block) {
                    later.command.into_rest();
                }
                let msg = format!("[Rest] {} has lost {gone}: the rest of its round is Rest", w.units[c].name);
                self.notes.push((i, msg.clone()));
                self.log.push(msg);
            }
            let a = &self.scheduled[i];
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
            // RULESET1 P11: a heavy windup holds the unit up while it winds up (rule poise)
            crate::poise::on_start(w, c, &mut self.scheduled[i].command);
            // HOOKS1 GH2: a Defend leaves its hazard as it starts (rule ground_hazards; nothing in classic)
            crate::ground::on_start(self, w, i);
        }
        // HOOKS1 GH6: a trap a dash or carried move sprang cancels that unit's later Walks (`stop_moves`)
        crate::ground::settle(self, w);
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
        // HOOKS1 8.4: the `ally_hit` triggers the hits consult, and what the action's events compare with (rule triggers)
        let before = crate::trigger::before_action(self, w, idx);
        self.scheduled[idx].command.execute(w, &mut self.rng);
        // RULESET1_INTENT MV8: a carried retreat follows the hit (nothing in classic)
        self.scheduled[idx].command.after_impact(w, &mut self.rng);
        self.scheduled[idx].landed = true;
        // RULESET1 P11, P4-P14: the windup bonus comes off; the hits stagger and may break (rule poise)
        crate::poise::drop_bonus(w, unit, &mut self.scheduled[idx].command);
        // HOOKS1 FM1: who was in windup as the hit landed, read before its stagger can break them (rule forced_move)
        let windups = crate::forced::windups(self, w, idx);
        crate::poise::resolve_hits(self, w, idx);
        let cmd = &self.scheduled[idx].command;
        if !cmd.log.is_empty() {
            self.log.push(cmd.log.clone());
        }
        // HOOKS1 4.3 step 6: the forced moves follow the stagger and breaks (rule forced_move; nothing in classic)
        crate::forced::resolve(self, w, idx, &windups);
        // HOOKS1 4.3 step 7: the hazard the action leaves, then the Walks a sprung trap cancels (rule ground_hazards)
        crate::ground::after_impact(self, w, idx);
        crate::ground::settle(self, w);
        self.apply_combo(w, idx);
        // HOOKS1 4.3 step 9, 8.1: the action's events after its combo pass, then the Walks a trap on a trigger's move
        // cancels (rule triggers; nothing in classic)
        if let Some(b) = before {
            crate::trigger::after_action(self, w, idx, &b);
            crate::ground::settle(self, w);
        }
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
        crate::trigger::before_round_end(self, w);
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
        // HOOKS1 GH5, GH7: the ground hazards after the terrain's, then the expired ones fade (none in classic)
        crate::ground::round_end(self, w);
        // HOOKS1 8.1, 8.3: `hp_below` and `round_end` after the ticks; held guards end (rule triggers; none in classic)
        crate::trigger::round_end(self, w);
        crate::ground::settle(self, w);
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
