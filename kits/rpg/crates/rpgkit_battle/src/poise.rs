//! Poise, stagger and breaks (RULESET1 5 and 6, rule `poise`; engine-only, no Godot original). Every unit has a poise
//! bar (its Max Poise: the ruleset's derived `poise_stat` row, or the unit row's fixed `poise`) that refills each round.
//! An attack's `stagger` lowers it on impact, multiplied by the windup, guard, repeat, flank, passive and hook factors
//! (P6); at 0 the unit breaks: every action of it that has not landed is cancelled and its guard drops (P7). A hit into a
//! windup may roll the break chance (P13) or break at once with `sure_break` (P14); `windup_poise`, `unbreakable` and the
//! status `steadfast` hold a unit up (P11, P12). The battle keeps the bars in `World.poise`, which stays empty in classic:
//! every function here reads the rule first, so classic battles never draw, log or record anything new.
//!
//! S2.2-S1/S2 (`docs/sessions/S2_POISE.md` 1 and 4): under the ruleset's `poise_mode` pips the bar is also `poise_pips`
//! pips of `T = max(1, roundi(max_poise / poise_pips))` (`threshold`); every stagger path (a hit, a collision, the arena
//! and trigger `stagger`, the Dojo) settles through `settle`, which removes `clamp(floor(D / T), 0,
//! max_pip_loss_per_hit)` pips (`pip_loss`) and breaks at 0 pips. Under `bar` (the default) `pips` stays 0 and nothing
//! reads it: every outcome and draw is the bar's. A break keeps its consequences in both modes (P7).
//!
//! S2.2-S3 (FB-71): under pips the continuous `value` mirrors the pips (`value = pips x T`, `mirror`), so every reader
//! of the value (metrics, the sim, the host) follows the pips; a windup bonus stores its pip count on the command
//! (`Command::poise_bonus_pips`) and may sit above `poise_pips`.
//!
//! S2.2-S4 (`S2_POISE.md` 1 "Poise Break (0 Pips)", FB-49d, C10): under pips a break (the unit's pips reach 0, by any
//! cause) also queues the status `stagger` for its next round (the status table's row: `block_lock` on the row's
//! `lock_blocks`, blocks 1 and 4; an AP-debt stagger the same round start joins the same active instance), unless it holds `steadfast`
//! (`break_queues_stagger`). Under `bar` a break keeps the RULESET1 consequences only.

use crate::command::{CmdKind, Command};
use crate::resolver::{Resolver, ScheduledAction};
use crate::skill::{self, Kind};
use crate::world::{UnitId, World};
use rpgkit_data::SkillDef;
use rpgkit_rules::derived;
use rpgkit_rules::ruleset::{self, Numbers, PoiseMode};
use rpgkit_rules::pipeline::{self, Modifier, Op, Stage};
use rpgkit_rules::pool::PoolView;

/// The status that keeps a unit from breaking (P12).
pub const STEADFAST: &str = "steadfast";

/// S2.2-S4: the status a break under pips queues for the unit's next round (the status table's row).
pub const BREAK_STATUS: &str = "stagger";

/// S2.2-S4 (`S2_POISE.md` 1, step 2 of a break): under `poise_mode` pips a broken unit `u` gets `stagger` queued for its
/// next round (the row's default rounds and stacking), unless it holds `steadfast`. True when it queued. Nothing under
/// `bar`.
pub fn break_queues_stagger(n: &Numbers, u: &mut rpgkit_rules::Unit) -> bool {
    if !break_would_queue_stagger(n, u) {
        return false;
    }
    let rounds = rpgkit_rules::status::row_of(BREAK_STATUS).map_or(1, |r| r.default_rounds);
    rpgkit_rules::status::queue(u, BREAK_STATUS, rounds, 0, "");
    true
}

/// R1-BAL (FB-88): whether a break of `u` now would queue `stagger` (`break_queues_stagger` without queueing it):
/// `poise_mode` pips and `u` not `steadfast`.
pub fn break_would_queue_stagger(n: &Numbers, u: &rpgkit_rules::Unit) -> bool {
    pips_mode(n) && !u.has_status(STEADFAST)
}

/// R1-BAL (FB-88): the break forecast's note when a break of `u` would queue `stagger` ("queues Stagger next round
/// (locks B1, B4)", the blocks from the status row's `lock_blocks`); None when it would not (`bar`, `steadfast`).
pub fn break_stagger_note(n: &Numbers, u: &rpgkit_rules::Unit) -> Option<String> {
    if !break_would_queue_stagger(n, u) {
        return None;
    }
    let locks: Vec<String> = rpgkit_rules::status::blocked_blocks(BREAK_STATUS).iter().map(|b| format!("B{b}")).collect();
    Some(if locks.is_empty() { "queues Stagger next round".to_string() } else { format!("queues Stagger next round (locks {})", locks.join(", ")) })
}

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
    /// S2.2-S1: pips now under `poise_mode` pips (`poise_pips` at a refill, above it while a windup bonus holds, 0 once
    /// broken); always 0 under `bar` (`pips_now` derives them there).
    pub pips: i32,
    /// S2.2-S2: Rest blocks landed this round under `poise_mode` pips (each second one restores a pip).
    pub rest_blocks: i32,
}

/// S2.2-S2: whether `n` plays poise by pips (`poise_mode` pips).
pub fn pips_mode(n: &Numbers) -> bool {
    n.poise_mode == PoiseMode::Pips
}

/// S2.2-S2: the Pip Threshold of a bar of Max Poise `max`: `T = max(1, roundi(max / poise_pips))`.
pub fn threshold(n: &Numbers, max: i32) -> i32 {
    ((max as f64 / n.poise_pips.max(1) as f64).round() as i32).max(1)
}

/// S2.2-S2: the pips stagger taken `taken` removes on threshold `t`: `clamp(floor(D / T), 0, max_pip_loss_per_hit)`
/// (a hit under the threshold removes none).
pub fn pip_loss(n: &Numbers, taken: i32, t: i32) -> i32 {
    (taken.max(0) / t.max(1)).clamp(0, n.max_pip_loss_per_hit.max(0))
}

/// S2.2-S2 (FB-49): the temporary pips a `windup_poise` grants on threshold `t`: `roundi(windup_poise / T)`, at least 1
/// when above 0.
pub fn bonus_pips(windup_poise: i32, t: i32) -> i32 {
    if windup_poise <= 0 {
        return 0;
    }
    ((windup_poise as f64 / t.max(1) as f64).round() as i32).max(1)
}

/// S2.2-S2: the pips a `bar` mode bar shows: each started `1 / poise_pips` of Max Poise lights one (a windup bonus over the
/// max stays at all of them); none once broken or without a bar (the battle view's pip rule).
pub fn bar_pips(n: &Numbers, st: &PoiseState) -> i32 {
    if st.broken || st.max <= 0 {
        return 0;
    }
    (st.value.clamp(0, st.max) * n.poise_pips.max(1) + st.max - 1) / st.max
}

/// S2.2-S2: `st`'s pips now: the stored ones under `poise_mode` pips, else `bar_pips`.
pub fn pips_now(n: &Numbers, st: &PoiseState) -> i32 {
    if pips_mode(n) {
        st.pips
    } else {
        bar_pips(n, st)
    }
}

/// S2.2-S3 (FB-71): under `poise_mode` pips the value mirrors the pips: `value = pips x T` (`threshold` of the bar's
/// max). Nothing under `bar`.
pub fn mirror(n: &Numbers, st: &mut PoiseState) {
    if pips_mode(n) {
        st.value = st.pips.max(0) * threshold(n, st.max);
    }
}

/// P3, S2.2-S2: `st` refilled to Max Poise `max` (and to `poise_pips` pips under `poise_mode` pips; 0 under `bar`;
/// S2.2-S3: the value then mirrors them, `mirror`).
pub fn refill(n: &Numbers, st: &mut PoiseState, max: i32) {
    st.max = max;
    st.value = max;
    st.pips = if pips_mode(n) && max > 0 { n.poise_pips.max(1) } else { 0 };
    st.rest_blocks = 0;
    mirror(n, st);
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

/// U1.2 slice 1 (`FB-04`): `c`'s bar as a pool view over `World.poise` (0 to its Max Poise; the value sits above the
/// max while a windup bonus holds, P11). None as `state`.
pub fn pool(w: &World, c: UnitId) -> Option<PoolView> {
    state(w, c).map(|s| PoolView { current: s.value, min: 0, max: s.max })
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
    let n = w.ruleset.numbers.clone();
    for c in 0..w.units.len() {
        let max = max_poise(w, c);
        let st = w.poise.entry(c).or_default();
        st.broken_last = st.broken;
        st.broken = false;
        refill(&n, st, max);
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
/// It runs the stage pipeline (U1.2 slice 1): the stagger as the base, the factors as amplifiers in that order.
pub fn stagger_taken(stagger: i32, f: &Factors) -> i32 {
    pipeline::STAGGER.run(&stagger_modifiers(stagger, f)).value
}

/// The pipeline modifiers of `stagger_taken`, in its order.
pub fn stagger_modifiers(stagger: i32, f: &Factors) -> [Modifier; 7] {
    let mult = |id: &'static str, x: f64| Modifier::new(id, Stage::Mult, Op::Mul(x));
    [
        Modifier::new("stagger", Stage::Base, Op::Add(stagger as f64)),
        mult("windup", f.windup),
        mult("guard", f.guard),
        mult("repeat", f.repeat),
        mult("flank", f.flank),
        mult("passive", f.passive),
        mult("hook", f.hook),
    ]
}

/// U4.2-S4 (FB-30): `stagger_taken` keeping every step, its `passive` step the target's evaluator
/// (`Unit::stagger_taken_modifier`, the same product as `factors` reads, with its grant sources as parts).
pub fn stagger_traced(stagger: i32, f: &Factors, target: &rpgkit_rules::Unit, blocks: i32) -> (i32, pipeline::Trace) {
    let [base, windup, guard, repeat, flank, _, hook] = stagger_modifiers(stagger, f);
    let passive = target.stagger_taken_modifier(blocks);
    debug_assert_eq!(passive.op, Op::Mul(f.passive), "the evaluator's product is the factors'");
    let (out, trace) = pipeline::STAGGER.run_traced(&[base, windup, guard, repeat, flank, passive, hook]);
    (out.value, trace)
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

/// P13: the break chance (0-100) of `taken` stagger on `before` poise: `min(cap, floor(100 x taken / before x scale))`
/// (S2.2-S2: under `poise_mode` pips, pips lost on pips before).
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
    /// R1-BAL (FB-88, FB-97 (2b)(A)): the break queued `stagger` for the unit's next round (`break_queues_stagger`);
    /// the log line ends with "(Stagger queued)".
    pub stagger_queued: bool,
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
    /// S2.2-S2: the target's pips before and after the hit (`pips_now`: derived from the bar under `bar`; 0 after a
    /// break).
    pub pips_before: i32,
    pub pips_after: i32,
    /// P13: (chance, roll) when the break chance was rolled.
    pub chance: Option<(i32, i32)>,
    pub broke: Option<Break>,
}

/// P9: the log line of a break; R1-BAL (FB-97 (2b)(A)): " (Stagger queued)" when the break queued `stagger`.
pub fn break_line(w: &World, t: UnitId, b: &Break) -> String {
    let base = if b.skill.is_empty() {
        format!("[Break] {} is broken", w.units[t].name)
    } else {
        format!("[Break] {}'s {} is broken", w.units[t].name, b.skill)
    };
    if b.stagger_queued {
        format!("{base} (Stagger queued)")
    } else {
        base
    }
}

/// P11: an action starting now adds its `windup_poise` to the user's bar (not to a broken unit or one without a bar);
/// the command remembers it to take it off again.
pub fn on_start(w: &mut World, c: UnitId, cmd: &mut Command) {
    if !on(w) || cmd.impact.windup_poise <= 0 {
        return;
    }
    let n = &w.ruleset.numbers;
    let Some(st) = w.poise.get_mut(&c) else { return };
    if st.max <= 0 || st.broken {
        return;
    }
    st.value += cmd.impact.windup_poise;
    // S2.2-S2 (FB-49): under pips the bonus is temporary pips (above `poise_pips` too); S2.2-S3 (FB-71): the command
    // keeps the count for `drop_bonus` and the value mirrors the pips
    if pips_mode(n) {
        let extra = bonus_pips(cmd.impact.windup_poise, threshold(n, st.max));
        st.pips += extra;
        cmd.poise_bonus_pips = extra;
        mirror(n, st);
    }
    cmd.poise_bonus = cmd.impact.windup_poise;
}

/// P11: the bonus of an action that lands or is cancelled comes off, never below 0 (S2.2-S2: its bonus pips too, under
/// `poise_mode` pips; S2.2-S3 (FB-71): the count `on_start` stored on the command, whatever the max is now, and the
/// value mirrors the pips).
pub fn drop_bonus(w: &mut World, c: UnitId, cmd: &mut Command) {
    if cmd.poise_bonus <= 0 {
        return;
    }
    let n = &w.ruleset.numbers;
    if let Some(st) = w.poise.get_mut(&c) {
        st.value = (st.value - cmd.poise_bonus).max(0);
        if pips_mode(n) {
            st.pips = (st.pips - cmd.poise_bonus_pips).max(0);
            mirror(n, st);
        }
    }
    cmd.poise_bonus = 0;
    cmd.poise_bonus_pips = 0;
}

/// RS3: landed Rest of `blocks` blocks restores `rest_poise_per_block` poise a block, up to Max Poise (a bar above it
/// stays). S2.2-S2 (FB-49): under `poise_mode` pips each second Rest block of the round restores a pip instead, up to
/// `poise_pips` (pips above it stay; S2.2-S3: the value mirrors the pips, so `rest_poise_per_block` adds nothing there).
pub fn restore(w: &mut World, c: UnitId, blocks: i32) {
    if !on(w) || blocks <= 0 {
        return;
    }
    let n = &w.ruleset.numbers;
    let amount = n.rest_poise_per_block * blocks;
    if let Some(st) = w.poise.get_mut(&c) {
        if st.max <= 0 || st.broken {
            return;
        }
        if pips_mode(n) {
            rest_pips(n, st, blocks);
        } else if amount > 0 && st.value < st.max {
            st.value = (st.value + amount).min(st.max);
        }
    }
}

/// S2.2-S2 (FB-49): `blocks` more Rest blocks this round on `st` (not broken) under `poise_mode` pips: each second Rest
/// block of the round restores a pip, up to `poise_pips` (pips above it stay; S2.2-S3: the value mirrors the pips).
/// Nothing under `bar`.
pub fn rest_pips(n: &Numbers, st: &mut PoiseState, blocks: i32) {
    if !pips_mode(n) || blocks <= 0 || st.max <= 0 || st.broken {
        return;
    }
    st.pips += rest_pip_gain(n, st.pips, st.rest_blocks, blocks);
    st.rest_blocks += blocks;
    mirror(n, st);
}

/// S2.2-S3 (FB-71): the pips `blocks` more Rest blocks restore under `poise_mode` pips on `pips` pips with `landed` Rest
/// blocks already this round: a pip per second Rest block of the round, up to `poise_pips` (none from a count at or
/// above it); 0 under `bar`. `rest_pips` and the planner's Rest preview (`round_intent_plan::rest_of`) read it.
pub fn rest_pip_gain(n: &Numbers, pips: i32, landed: i32, blocks: i32) -> i32 {
    if !pips_mode(n) || blocks <= 0 {
        return 0;
    }
    let gain = (landed.max(0) + blocks) / 2 - landed.max(0) / 2;
    let full = n.poise_pips.max(1);
    if pips >= full {
        0
    } else {
        gain.min(full - pips)
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
        let pips = pips_mode(&w.ruleset.numbers);
        let mut hit = StaggerHit {
            target: t,
            stagger: impact.stagger,
            factors: fc.factors,
            windup: fc.windup,
            taken: fc.taken,
            before: st.value,
            after: st.value,
            pips_before: fc.pips_before,
            pips_after: fc.pips_before,
            chance: None,
            broke: None,
        };
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
            let bar = w.poise.get_mut(&t).unwrap();
            bar.value = fc.after;
            if pips {
                bar.pips = fc.pips_after;
            }
            (hit.after, hit.pips_after) = (fc.after, fc.pips_after);
            if let Some(cause) = cause {
                (hit.after, hit.pips_after) = (0, 0);
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
    let n = &w.ruleset.numbers;
    let held = !st.broken && unbreakable(w, sched, t, block);
    let terms = Terms { windup, held, sure: impact.sure_break.then_some(BreakCause::Sure), chance: true };
    let s = settle(n, &st, taken, terms);
    let (pips_before, mut pips_after) = if pips_mode(n) { (st.pips, s.pips) } else { (bar_pips(n, &st), bar_pips(n, &PoiseState { value: s.after, ..st })) };
    // AUDIT-02: a break sets the pips to 0 on top (`Forecast`'s contract)
    if matches!(s.verdict, Verdict::Breaks(_)) {
        pips_after = 0;
    }
    Forecast { factors: f, windup, taken, before: st.value, after: s.after, verdict: s.verdict, threshold: threshold(n, st.max), pips_before, pips_after }
}

/// S2.2-S2: what decides a stagger hit beside its stagger taken: the target winds up (P5), cannot be broken (P12), the
/// windup break the hit makes (P14 `sure_break`, FM4 `collide_break`) and whether it may roll the break chance (P13: an
/// attack's hit; a collision never rolls).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Terms {
    pub windup: bool,
    pub held: bool,
    pub sure: Option<BreakCause>,
    pub chance: bool,
}

/// `settle`'s result: the bar's value and pips after the hit (0 at the least; a break sets both to 0 on top) and the
/// verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settled {
    pub after: i32,
    /// The stored pips after the hit (`st.pips` unchanged under `bar` unless the hit breaks).
    pub pips: i32,
    pub verdict: Verdict,
}

/// P6-P14 and S2.2-S2, the one rule every stagger path settles by (deterministic, no roll): stagger taken `taken` on bar
/// `st`. Under `bar` the value drops by it, held at 1 by P12, and breaks at 0 (P7); a hit into a windup that leaves poise
/// may roll `break_chance(taken, before)` (P13). Under `poise_mode` pips the hit removes `pip_loss` pips: held at 1 pip
/// by `unbreakable` or `steadfast`, a break at 0 pips, the chance from pips lost on pips before; the value after is the
/// pips' mirror (`pips x T`, S2.2-S3). A `sure` break into a windup removes every pip (P14) in both modes.
pub fn settle(n: &Numbers, st: &PoiseState, taken: i32, h: Terms) -> Settled {
    if st.broken {
        return Settled { after: st.value, pips: st.pips, verdict: Verdict::AlreadyBroken };
    }
    let mut after = st.value - taken;
    if h.held {
        after = after.max(st.value.min(1));
    }
    let sure = h.sure.filter(|_| h.windup && !h.held);
    let may_roll = h.chance && h.windup && !h.held && n.break_chance_scale > 0.0;
    if !pips_mode(n) {
        let verdict = if taken > 0 && after <= 0 && !h.held {
            Verdict::Breaks(BreakCause::Poise)
        } else if let Some(c) = sure {
            Verdict::Breaks(c)
        } else if may_roll && taken > 0 && after > 0 {
            Verdict::Chance(break_chance(n, taken, st.value))
        } else {
            Verdict::Holds
        };
        return broken_to_zero(Settled { after: after.max(0), pips: st.pips, verdict });
    }
    let lost = pip_loss(n, taken, threshold(n, st.max));
    let mut pips = st.pips - lost;
    if h.held {
        pips = pips.max(st.pips.min(1));
    }
    let pips_lost = st.pips - pips.max(0);
    let verdict = if lost > 0 && pips <= 0 && !h.held {
        Verdict::Breaks(BreakCause::Poise)
    } else if let Some(c) = sure {
        Verdict::Breaks(c)
    } else if may_roll && pips_lost > 0 && pips > 0 {
        Verdict::Chance(break_chance(n, pips_lost, st.pips))
    } else {
        Verdict::Holds
    };
    // S2.2-S3 (FB-71): the value mirrors the pips (`mirror`)
    let pips = pips.max(0);
    broken_to_zero(Settled { after: pips * threshold(n, st.max), pips, verdict })
}

/// AUDIT-02: a break leaves the bar at 0 value and 0 pips (`Settled`'s contract), a `sure` break whose stagger did not
/// deplete the bar too.
fn broken_to_zero(s: Settled) -> Settled {
    match s.verdict {
        Verdict::Breaks(_) => Settled { after: 0, pips: 0, ..s },
        _ => s,
    }
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
            if pips_mode(&w.ruleset.numbers) {
                next.pips = fc.pips_after;
            }
            if matches!(fc.verdict, Verdict::Breaks(_)) {
                (next.value, next.pips, next.broken) = (0, 0, true);
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
    /// S2.2-S2: the target's Pip Threshold `T` (`threshold`; in both modes).
    pub threshold: i32,
    /// S2.2-S2: the target's pips before and after the hit (a break sets them to 0 on top): the stored pips under
    /// `poise_mode` pips, `bar_pips` of the bar before and after under `bar`.
    pub pips_before: i32,
    pub pips_after: i32,
}

/// P7: `t` breaks at `block`: poise 0 until the next refill; every action of it that has not landed is cancelled (also
/// one later in this block, its Rest blocks and its running Defend, whose guard drops at once; AP paid stays spent, an
/// action not started pays nothing); the recovery of a landed action ends. Returns the break with the action it names
/// (the one in windup, else the one running, else the next).
pub fn break_unit(res: &mut Resolver, w: &mut World, t: UnitId, block: i32, cause: BreakCause) -> Break {
    let st = w.poise.entry(t).or_default();
    st.value = 0;
    st.pips = 0;
    st.broken = true;
    // S2.2-S4: under pips the break queues `stagger` for next round (not on a `steadfast` unit)
    let n = w.ruleset.numbers.clone();
    let stagger_queued = break_queues_stagger(&n, &mut w.units[t]);
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
        a.command.poise_bonus_pips = 0;
    }
    // HOOKS1 8.3, S1.2-S4: a break ends the guard: the held guard (a Guard skill's from its wind-up, Option 1C, or a
    // trigger's) and the `guard` status instance
    crate::trigger::end_guard(w, t);
    let b = Break { cause, skill: named.map(|(_, n)| n).unwrap_or_default(), stagger_queued };
    res.log.push(break_line(w, t, &b));
    b
}
