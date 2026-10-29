//! Poise, stagger and breaks (RULESET1 5 and 6, rule `poise`; engine-only, no Godot original). Every unit has a poise
//! bar (its Max Poise: the ruleset's derived `poise_stat` row, or the unit row's fixed `poise`) that refills each round.
//! An attack's `stagger` lowers it on impact, multiplied by the windup, guard, repeat, flank, passive and hook factors
//! (P6); at 0 the unit breaks: every action of it that has not landed is cancelled and its guard drops (P7). A hit into a
//! windup may roll the break chance (P13) or break at once with `sure_break` (P14); `windup_poise`, `unbreakable` and the
//! status `steadfast` hold a unit up (P11, P12). The battle keeps the bars in `World.poise`, which stays empty in classic:
//! every function here reads the rule first, so classic battles never draw, log or record anything new.

use crate::command::{CmdKind, Command};
use crate::resolver::{Resolver, ScheduledAction};
use crate::skill::{self, Kind};
use crate::world::{UnitId, World};
use rpgkit_data::SkillDef;
use rpgkit_rules::derived;
use rpgkit_rules::ruleset;
use rpgkit_rules::stats::roundi;

/// The status that keeps a unit from breaking (P12).
pub const STEADFAST: &str = "steadfast";

/// What a skill does to poise and armour (RULESET1 tasks 1.3), carried by its command; read only under the rules
/// `poise` (stagger, windup poise, unbreakable, sure break) and `pierce` (armour piercing).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Impact {
    /// P4: poise taken from each unit the hit damages (ATTACK).
    pub stagger: i32,
    /// P11: poise the user gains while the action winds up (ATTACK, SUPPORT, FOCUS).
    pub windup_poise: i32,
    /// P12: the user's poise cannot drop below 1 while the action winds up (ATTACK, SUPPORT, FOCUS).
    pub unbreakable: bool,
    /// P14: a hit into a windup breaks the target whatever its poise (ATTACK).
    pub sure_break: bool,
    /// A2: the hit skips the armour term (ATTACK).
    pub armor_piercing: bool,
}

impl Impact {
    /// The skill's fields for the kinds that carry them (items carry none: P4).
    pub fn of(s: &SkillDef) -> Self {
        if skill::is_item(s) {
            return Self::default();
        }
        let k = skill::kind(s);
        let attack = k == Kind::Attack;
        let windup = matches!(k, Kind::Attack | Kind::Support | Kind::Focus);
        Self {
            stagger: if attack { s.stagger.max(0) } else { 0 },
            windup_poise: if windup { s.windup_poise.max(0) } else { 0 },
            unbreakable: windup && s.unbreakable,
            sure_break: attack && s.sure_break,
            armor_piercing: attack && s.armor_piercing,
        }
    }
}

/// One unit's poise in a battle.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PoiseState {
    /// Max Poise at the last refill (0 = no poise bar: never staggered, never broken; P2).
    pub max: i32,
    /// Poise now (above `max` while a windup bonus holds, P11).
    pub value: i32,
    /// Broken this round (P8: at most once a round; poise stays 0 until the next refill).
    pub broken: bool,
    /// Broken last round (P10: stagger x `stagger_repeat_mult` this round).
    pub broken_last: bool,
}

/// Whether the battle plays by poise.
pub fn on(w: &World) -> bool {
    w.ruleset.has(ruleset::POISE)
}

/// Unit `c`'s bar (None in classic, or before the first refill).
pub fn state(w: &World, c: UnitId) -> Option<&PoiseState> {
    if !on(w) {
        return None;
    }
    w.poise.get(&c)
}

/// P1, P2: `c`'s Max Poise now: the unit row's fixed `poise` (above 0), none (0), else the ruleset's `poise_stat` row
/// (0 when the ruleset names no row or the row is not in it).
pub fn max_poise(w: &World, c: UnitId) -> i32 {
    max_poise_of(&w.cat, &w.ruleset, &w.units[c])
}

/// `max_poise` of unit `u` outside a battle (the Dojo): the same P1, P2 rule under `rules`.
pub fn max_poise_of(cat: &rpgkit_data::Catalog, rules: &ruleset::Ruleset, u: &rpgkit_rules::Unit) -> i32 {
    match u.poise {
        p if p > 0 => p,
        0 => 0,
        _ => derived::value_for(cat, rules, u, &rules.numbers.poise_stat).unwrap_or(0).max(0),
    }
}

/// P3: every unit's poise refills to its Max Poise (round start, before planning; units not yet on the field too, so a
/// wave arrives full); last round's break is remembered for P10.
pub fn round_start(w: &mut World) {
    if !on(w) {
        return;
    }
    for c in 0..w.units.len() {
        let max = max_poise(w, c);
        let st = w.poise.entry(c).or_default();
        st.broken_last = st.broken;
        st.broken = false;
        st.max = max;
        st.value = max;
    }
}

/// P5: the command kinds that wind up (Attack, Area, Support, Focus, Item); Defend, moves, Disengage, Target, Rest and
/// lost blocks never do.
pub fn winds_up(cmd: &Command) -> bool {
    matches!(cmd.kind, CmdKind::Attack { .. } | CmdKind::Area { .. } | CmdKind::Support { .. } | CmdKind::Focus { .. } | CmdKind::Item { .. })
}

/// P5: `a` is in windup at `block`: started, not cancelled, not yet landed, of a kind that winds up.
pub fn action_in_windup(a: &ScheduledAction, block: i32) -> bool {
    !a.cancelled && !a.landed && a.start <= block && winds_up(&a.command)
}

/// P5: unit `c` has an action in windup at `block`.
pub fn in_windup(sched: &[ScheduledAction], c: UnitId, block: i32) -> bool {
    sched.iter().any(|a| a.unit == c && action_in_windup(a, block))
}

/// P6, G2: a Defend command of `c` runs at `block` (not cancelled, its guard not broken).
pub fn guarded(sched: &[ScheduledAction], c: UnitId, block: i32) -> bool {
    sched.iter().any(|a| a.unit == c && !a.cancelled && a.start <= block && block <= a.end && matches!(a.command.kind, CmdKind::Defend { guard_broken: false, .. }))
}

/// P12: `c` cannot be broken now: it holds `steadfast`, or an `unbreakable` action of it winds up.
pub fn unbreakable(w: &World, sched: &[ScheduledAction], c: UnitId, block: i32) -> bool {
    w.units[c].has_status(STEADFAST) || sched.iter().any(|a| a.unit == c && a.command.impact.unbreakable && action_in_windup(a, block))
}

/// P6: the factors of one hit's stagger, each 1.0 when it does not apply.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Factors {
    /// `stagger_windup_mult` when the target is in windup (P5).
    pub windup: f64,
    /// `stagger_guard_mult` while a Defend of the target runs (G2).
    pub guard: f64,
    /// `stagger_repeat_mult` when the target broke last round (P10).
    pub repeat: f64,
    /// `flank_stagger_mult` on a flank hit (intent MV3; the rule `flank`).
    pub flank: f64,
    /// The product of the target's `stagger_taken` effects that match the action's blocks.
    pub passive: f64,
    /// The Act 2 hook (Flesh-Tension); 1.0 in M1.
    pub hook: f64,
}

impl Default for Factors {
    fn default() -> Self {
        Self { windup: 1.0, guard: 1.0, repeat: 1.0, flank: 1.0, passive: 1.0, hook: 1.0 }
    }
}

/// P6: stagger taken = `roundi(stagger x windup x guard x repeat x flank x passive x hook)`, floor 0.
pub fn stagger_taken(stagger: i32, f: &Factors) -> i32 {
    roundi(stagger as f64 * f.windup * f.guard * f.repeat * f.flank * f.passive * f.hook).max(0)
}

/// P6: the factors of a hit of an action `blocks` long on `t` at `block` (`flank`: the hit was a flank with the rule on).
pub fn factors(w: &World, sched: &[ScheduledAction], t: UnitId, block: i32, blocks: i32, flank: bool) -> Factors {
    let n = &w.ruleset.numbers;
    let pick = |on: bool, mult: f64| if on { mult } else { 1.0 };
    Factors {
        windup: pick(in_windup(sched, t, block), n.stagger_windup_mult),
        // HOOKS1 8.3: a held guard counts as guarding (rule triggers; none in classic)
        guard: pick(crate::trigger::guarding(w, sched, t, block), n.stagger_guard_mult),
        repeat: pick(w.poise.get(&t).is_some_and(|s| s.broken_last), n.stagger_repeat_mult),
        flank: pick(flank, n.flank_stagger_mult),
        passive: w.units[t].stagger_taken.iter().filter(|e| e.matches(blocks)).map(|e| e.mult).product(),
        hook: 1.0,
    }
}

/// P13: the break chance (0-100) of `taken` stagger on `before` poise: `min(cap, floor(100 x taken / before x scale))`.
pub fn break_chance(n: &ruleset::Numbers, taken: i32, before: i32) -> i32 {
    if before <= 0 || taken <= 0 {
        return 0;
    }
    ((100.0 * taken as f64 / before as f64 * n.break_chance_scale).floor() as i32).clamp(0, n.break_chance_cap.max(0))
}

/// Why a unit broke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakCause {
    /// P7: stagger took poise to 0.
    Poise,
    /// P13: the break chance roll of a hit into a windup.
    Chance,
    /// P14: a `sure_break` hit into a windup.
    Sure,
    /// HOOKS1 FM4: a `collide_break` collision into a windup (the wall-slam break, as P14).
    Collision,
}

impl BreakCause {
    pub fn id(self) -> &'static str {
        match self {
            BreakCause::Poise => "poise",
            BreakCause::Chance => "chance",
            BreakCause::Sure => "sure",
            BreakCause::Collision => "collision",
        }
    }
}

/// A break (P7, P9): its cause and the action the log names ("" = none: `[Break] <unit> is broken`).
#[derive(Debug, Clone, PartialEq)]
pub struct Break {
    pub cause: BreakCause,
    pub skill: String,
}

/// What one hit did to poise.
#[derive(Debug, Clone, PartialEq)]
pub struct StaggerHit {
    pub target: UnitId,
    /// The skill's stagger.
    pub stagger: i32,
    pub factors: Factors,
    /// The target was in windup (P5).
    pub windup: bool,
    pub taken: i32,
    pub before: i32,
    pub after: i32,
    /// P13: (chance, roll) when the break chance was rolled.
    pub chance: Option<(i32, i32)>,
    pub broke: Option<Break>,
}

/// P9: the log line of a break.
pub fn break_line(w: &World, t: UnitId, b: &Break) -> String {
    if b.skill.is_empty() {
        format!("[Break] {} is broken", w.units[t].name)
    } else {
        format!("[Break] {}'s {} is broken", w.units[t].name, b.skill)
    }
}

/// P11: an action starting now adds its `windup_poise` to the user's bar (not to a broken unit or one without a bar);
/// the command remembers it to take it off again.
pub fn on_start(w: &mut World, c: UnitId, cmd: &mut Command) {
    if !on(w) || cmd.impact.windup_poise <= 0 {
        return;
    }
    let Some(st) = w.poise.get_mut(&c) else { return };
    if st.max <= 0 || st.broken {
        return;
    }
    st.value += cmd.impact.windup_poise;
    cmd.poise_bonus = cmd.impact.windup_poise;
}

/// P11: the bonus of an action that lands or is cancelled comes off, never below 0.
pub fn drop_bonus(w: &mut World, c: UnitId, cmd: &mut Command) {
    if cmd.poise_bonus <= 0 {
        return;
    }
    if let Some(st) = w.poise.get_mut(&c) {
        st.value = (st.value - cmd.poise_bonus).max(0);
    }
    cmd.poise_bonus = 0;
}

/// RS3: landed Rest restores `amount` poise, up to Max Poise (a bar above it stays).
pub fn restore(w: &mut World, c: UnitId, amount: i32) {
    if !on(w) || amount <= 0 {
        return;
    }
    if let Some(st) = w.poise.get_mut(&c) {
        if st.max > 0 && !st.broken && st.value < st.max {
            st.value = (st.value + amount).min(st.max);
        }
    }
}

/// P4, P6-P14: the stagger of the action `idx` that just landed, on each unit it damaged in hit order: stagger taken,
/// the break by poise, by `sure_break` or by the chance roll (one draw on the resolver RNG per qualifying hit, after the
/// action's own damage, status and riposte draws; an area's after all its hits), then the break itself. The records go
/// to the command's `staggers`. Nothing under classic (no rule poise).
pub fn resolve_hits(res: &mut Resolver, w: &mut World, idx: usize) {
    if !on(w) {
        return;
    }
    let block = res.scheduled[idx].impact;
    let (impact, blocks) = (res.scheduled[idx].command.impact, res.scheduled[idx].command.block_cost);
    if impact.stagger <= 0 && !impact.sure_break {
        return;
    }
    let hits = res.scheduled[idx].command.hits.clone();
    let ignore_guard = res.scheduled[idx].command.ignore_guard;
    let mut out = Vec::new();
    for (t, flank) in hits {
        let Some(st) = w.poise.get(&t).copied() else { continue };
        if !w.units[t].is_alive() || st.max <= 0 {
            continue;
        }
        let fc = forecast(w, &res.scheduled, t, st, block, &impact, blocks, flank, ignore_guard);
        let mut hit = StaggerHit { target: t, stagger: impact.stagger, factors: fc.factors, windup: fc.windup, taken: fc.taken, before: st.value, after: st.value, chance: None, broke: None };
        // P8: a unit broken this round stays at 0 and breaks no further
        if !st.broken {
            let cause = match fc.verdict {
                Verdict::Breaks(c) => Some(c),
                Verdict::Chance(chance) => {
                    let roll = res.rng.randi_range(1, 100);
                    hit.chance = Some((chance, roll));
                    (roll <= chance).then_some(BreakCause::Chance)
                }
                Verdict::Holds | Verdict::AlreadyBroken => None,
            };
            w.poise.get_mut(&t).unwrap().value = fc.after;
            hit.after = fc.after;
            if let Some(cause) = cause {
                hit.after = 0;
                hit.broke = Some(break_unit(res, w, t, block, cause));
            }
        }
        out.push(hit);
    }
    res.scheduled[idx].command.staggers = out;
}

/// What a hit's stagger would do, the P13 roll aside (`forecast`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The bar holds (poise left, no chance roll).
    Holds,
    /// P7 (poise to 0), P14 (`sure_break` into a windup): the unit breaks.
    Breaks(BreakCause),
    /// P13: a hit into a windup that leaves poise: it breaks on a roll at or under this chance (0-100).
    Chance(i32),
    /// P8: broken this round already: nothing more.
    AlreadyBroken,
}

/// P6-P14 for one hit, without the P13 roll (deterministic, nothing changes): the stagger factors of a hit landing at
/// `block` from an action `blocks` long (`flank`: a flank hit with the rule on; `ignore_guard`: HOOKS1 IG1), the stagger
/// taken, the bar after it and the verdict. `st` is the target's bar before the hit. `resolve_hits` resolves every hit
/// through it (one rule), and the battle screen forecasts a planned hit with it (RULESET1-U slice 5).
#[allow(clippy::too_many_arguments)]
pub fn forecast(w: &World, sched: &[ScheduledAction], t: UnitId, st: PoiseState, block: i32, impact: &Impact, blocks: i32, flank: bool, ignore_guard: bool) -> Forecast {
    let windup = in_windup(sched, t, block);
    let mut f = factors(w, sched, t, block, blocks, flank);
    // HOOKS1 IG1: a guard-bypassing hit staggers as if the target did not guard (set only under the rule ignore_guard)
    if ignore_guard {
        f.guard = 1.0;
    }
    let taken = stagger_taken(impact.stagger, &f);
    if st.broken {
        return Forecast { factors: f, windup, taken, before: st.value, after: st.value, verdict: Verdict::AlreadyBroken };
    }
    let held = unbreakable(w, sched, t, block);
    let mut after = st.value - taken;
    if held {
        after = after.max(st.value.min(1));
    }
    let verdict = if taken > 0 && after <= 0 && !held {
        Verdict::Breaks(BreakCause::Poise)
    } else if windup && impact.sure_break && !held {
        Verdict::Breaks(BreakCause::Sure)
    } else if windup && !held && taken > 0 && after > 0 && w.ruleset.numbers.break_chance_scale > 0.0 {
        Verdict::Chance(break_chance(&w.ruleset.numbers, taken, st.value))
    } else {
        Verdict::Holds
    };
    Forecast { factors: f, windup, taken, before: st.value, after: after.max(0), verdict }
}

/// One planned hit for `forecast_hits`: its target, impact block, the skill's impact, the action's blocks, whether it
/// flanks (with the rule on) and whether it bypasses guard.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlannedHit {
    pub target: UnitId,
    pub block: i32,
    pub impact: Impact,
    pub blocks: i32,
    pub flank: bool,
    pub ignore_guard: bool,
}

/// RULESET1-U slice 6 (the battle screen's forecasts): `forecast` of each planned hit in its order, each on the bar the
/// earlier hits of the sequence leave (poise after it; a break holds the bar broken at 0 for the rest of the round: P7,
/// P8). A target without a bar (no rule poise, or Max Poise 0) gets None. Nothing changes and nothing is rolled.
pub fn forecast_hits(w: &World, sched: &[ScheduledAction], hits: &[PlannedHit]) -> Vec<Option<Forecast>> {
    let mut bars: Vec<(UnitId, PoiseState)> = Vec::new();
    hits.iter()
        .map(|h| {
            let st = match bars.iter().find(|b| b.0 == h.target) {
                Some(b) => b.1,
                None => *state(w, h.target).filter(|st| st.max > 0)?,
            };
            let fc = forecast(w, sched, h.target, st, h.block, &h.impact, h.blocks, h.flank, h.ignore_guard);
            let mut next = st;
            next.value = fc.after;
            if matches!(fc.verdict, Verdict::Breaks(_)) {
                (next.value, next.broken) = (0, true);
            }
            match bars.iter_mut().find(|b| b.0 == h.target) {
                Some(b) => b.1 = next,
                None => bars.push((h.target, next)),
            }
            Some(fc)
        })
        .collect()
}

/// One hit's forecast (`forecast`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Forecast {
    pub factors: Factors,
    /// The target winds up at the hit's block (P5).
    pub windup: bool,
    pub taken: i32,
    pub before: i32,
    /// Poise after the hit (0 at the least; a break sets it to 0 on top).
    pub after: i32,
    pub verdict: Verdict,
}

/// P7: `t` breaks at `block`: poise 0 until the next refill; every action of it that has not landed is cancelled (also
/// one later in this block, its Rest blocks and its running Defend, whose guard drops at once; AP paid stays spent, an
/// action not started pays nothing); the recovery of a landed action ends. Returns the break with the action it names
/// (the one in windup, else the one running, else the next).
pub fn break_unit(res: &mut Resolver, w: &mut World, t: UnitId, block: i32, cause: BreakCause) -> Break {
    let st = w.poise.entry(t).or_default();
    st.value = 0;
    st.broken = true;
    let mut named: Option<(u8, String)> = None;
    for a in res.scheduled.iter_mut().filter(|a| a.unit == t && !a.cancelled) {
        if a.landed {
            // a Defend that landed earlier in this block still runs: its guard ends; a recovery ends
            if let CmdKind::Defend { guard_broken, .. } = &mut a.command.kind {
                if a.end >= block {
                    *guard_broken = true;
                }
            }
            a.end = a.end.min(block.max(a.impact));
            continue;
        }
        let rank = if action_in_windup(a, block) {
            0
        } else if a.start <= block {
            1
        } else {
            2
        };
        if named.as_ref().is_none_or(|(r, _)| rank < *r) && !matches!(a.command.kind, CmdKind::Rest { .. }) {
            named = Some((rank, a.command.name.clone()));
        }
        a.cancelled = true;
        a.broken = true;
        a.command.poise_bonus = 0;
    }
    w.units[t].guard = 0.0;
    // HOOKS1 8.3: a break ends a held guard (rule triggers; none in classic)
    w.units[t].held_guard = None;
    let b = Break { cause, skill: named.map(|(_, n)| n).unwrap_or_default() };
    res.log.push(break_line(w, t, &b));
    b
}
