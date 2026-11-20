//! STATUSVIS1 (PR7-57, `docs/specs/STATUSVIS1.md` SV1-SV15): engine-only kit reads for showing each status where it
//! acts, by its primitive (never a status id, SV4): the locked runs of a unit's round (`block_lock`), the plate column's
//! priority (section 3), the round-end ticks that join the hits in one planned loss (`tick`, SV6), a hit's number after
//! every modifier with its breakdown (`modifier`, SV7), a riposte's likely return (`reactive`, SV9) and a unit's one
//! target for the round (SV14, SV15). Display reads only: nothing here changes a battle, and every number is the one the
//! rules forecast (`intent::hit_on`, `status::tick_amount`, the counter's multiplier).

use crate::command::CmdKind;
use crate::grid::V2;
use crate::intent::{self, Adjust, HitOn};
use crate::resolver::ScheduledAction;
use crate::world::{UnitId, World};
use rpgkit_rules::damage::{self, HitInput};
use rpgkit_rules::status::{self, Primitive, Rating};

/// SV5, SV15: a run of blocks one `block_lock` status locks on a unit's round (both inclusive).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockRun {
    pub status: String,
    pub start: i32,
    pub end: i32,
}

/// SV5, SV15: the blocks `u`'s active `block_lock` statuses lock this round (each status row's `lock_blocks` up to the
/// round's blocks; a block two statuses lock goes to the first active, as the plan books it), as runs: blocks of one
/// status in a row are one run (stun: 1-2), blocks apart are runs of their own (stagger: 1 and 4). Block order.
pub fn lock_runs(w: &World, u: UnitId) -> Vec<LockRun> {
    let Some(unit) = w.units.get(u) else { return Vec::new() };
    let mut owner: Vec<(i32, &str)> = Vec::new();
    for s in &unit.st.active {
        for &b in status::blocked_blocks(s) {
            if b >= 1 && b <= w.total_blocks && !owner.iter().any(|o| o.0 == b) {
                owner.push((b, s.as_str()));
            }
        }
    }
    owner.sort_by_key(|o| o.0);
    let mut out: Vec<LockRun> = Vec::new();
    for (b, s) in owner {
        match out.last_mut() {
            Some(r) if r.status == s && r.end + 1 == b => r.end = b,
            _ => out.push(LockRun { status: s.to_string(), start: b, end: b }),
        }
    }
    out
}

/// The lock run holding block `b` (None: the block is free).
pub fn run_at(runs: &[LockRun], b: i32) -> Option<&LockRun> {
    runs.iter().find(|r| (r.start..=r.end).contains(&b))
}

/// STATUSVIS1 section 3: a status's plate priority by its primitives: 1 (high) a `reactive` or `rule_flag` row (it
/// punishes a choice, or the plate is its only home), 2 a `tick` row (its amount and stacks matter), 3 every other (it
/// shows in place: locks, modifiers, targeting); 3 for an unknown id.
pub fn plate_priority(id: &str) -> u8 {
    match status::row_of(id) {
        Some(r) if r.has_primitive(Primitive::Reactive) || r.has_primitive(Primitive::RuleFlag) => 1,
        Some(r) if r.has_primitive(Primitive::Tick) => 2,
        _ => 3,
    }
}

/// SV6: one round-end tick on a unit: its status and the HP it moves (negative a loss, positive a gain).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tick {
    pub status: String,
    pub hp: i32,
}

/// SV6: the ticks `u`'s active `tick` statuses deal at the round's end, in the resolver's order (the debuff rows in
/// table order, then the buff rows; `Resolver::apply_round_end_effects`): each its `status::tick_amount`, a loss of 0
/// left out (poison at 0), a gain within the HP missing now. None on a unit that is down.
pub fn round_ticks(w: &World, u: UnitId) -> Vec<Tick> {
    let Some(unit) = w.units.get(u).filter(|x| x.is_active()) else { return Vec::new() };
    let n = &w.ruleset.numbers;
    let mut out: Vec<Tick> = status::tick_rows(false)
        .filter(|r| unit.has_status(r.id))
        .map(|r| Tick { status: r.id.to_string(), hp: -status::tick_amount(unit, r, n) })
        .filter(|t| t.hp < 0)
        .collect();
    out.extend(status::tick_rows(true).filter(|r| unit.has_status(r.id)).map(|r| Tick { status: r.id.to_string(), hp: crate::command::effective_heal(unit, status::tick_amount(unit, r, n)) }).filter(|t| t.hp > 0));
    out
}

/// SV6: the one planned loss on a unit's HP bar: every hit its foes' live actions land on it this round (`hit_on`, in
/// block order, each with its attacker), then its round-end ticks; `loss` the hits and the damage ticks together within
/// its HP now, `gain` the healing ticks.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlannedLoss {
    pub hits: Vec<(UnitId, HitOn)>,
    pub ticks: Vec<Tick>,
    pub loss: i32,
    pub gain: i32,
}

impl PlannedLoss {
    /// The hits' HP alone.
    pub fn hit_hp(&self) -> i32 {
        self.hits.iter().map(|h| h.1.damage).sum()
    }
}

/// SV6: `u`'s planned loss over `sched`, the round as it will run (`intent::planned_schedule` while planning). The hits
/// are the foe card's (`intent::foe_card`: its foes' live actions by impact, then start, then booking), so a foe's
/// plate and its card agree. Read only.
pub fn planned_loss(w: &World, sched: &[ScheduledAction], u: UnitId) -> PlannedLoss {
    let Some(unit) = w.units.get(u) else { return PlannedLoss::default() };
    let foes = w.foes_of(u);
    let mut acts: Vec<(usize, &ScheduledAction)> = sched.iter().enumerate().filter(|(_, a)| !a.cancelled && a.unit != u && foes.contains(&a.unit)).collect();
    acts.sort_by_key(|(k, a)| (a.impact, a.start, *k));
    let hits: Vec<(UnitId, HitOn)> = acts.into_iter().filter_map(|(_, a)| intent::hit_on(w, sched, a, u).map(|h| (a.unit, h))).collect();
    let ticks = round_ticks(w, u);
    let hit_hp: i32 = hits.iter().map(|h| h.1.damage).sum();
    let tick_loss: i32 = ticks.iter().filter(|t| t.hp < 0).map(|t| -t.hp).sum();
    let gain: i32 = ticks.iter().filter(|t| t.hp > 0).map(|t| t.hp).sum();
    PlannedLoss { hits, ticks, loss: (hit_hp + tick_loss).clamp(0, unit.hp.max(0)), gain }
}

/// SV7: one modifier on a hit's number: the status (or `guard` for a guard from a Defend) and the percent it moves the
/// number by (negative lowers it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitMod {
    pub source: String,
    pub pct: i32,
}

/// SV7: a hit's number after every modifier (`damage`, the forecast's) and the unmodified number (`base`): the
/// attacker's attack-power statuses (weaken, empower), the target's armour statuses (expose, fortify) and its guard
/// (the `guard` status or a Defend) taken out; `mods` each modifier with its own percent.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HitParts {
    pub damage: i32,
    pub base: i32,
    pub mods: Vec<HitMod>,
}

impl HitParts {
    /// The number moved by a modifier: -1 lowered, 1 raised, 0 the same (SV7's tint and arrow).
    pub fn sign(&self) -> i32 {
        (self.damage - self.base).signum()
    }
}

/// The percent `with` is of `without`, less 100 (0 when `without` is 0).
fn pct(with: i32, without: i32) -> i32 {
    if without <= 0 {
        return 0;
    }
    ((with as f64 / without as f64 - 1.0) * 100.0).round() as i32
}

/// SV7: the breakdown of the hit scheduled action `a` lands on `t` (`intent::hit_on`; None when it lands none). Each
/// modifier's percent compares the number with it and without it alone; `base` takes them all out. Read only.
pub fn hit_parts(w: &World, sched: &[ScheduledAction], a: &ScheduledAction, t: UnitId) -> Option<HitParts> {
    let hit = intent::hit_on(w, sched, a, t)?;
    let src = &w.units[a.unit];
    let target = &w.units[t];
    let power: Vec<(&str, f64)> = src.st.active.iter().map(|s| (s.as_str(), src.attack_power_factor_of(s))).filter(|(_, f)| *f != 1.0 && *f > 0.0).collect();
    let armour: Vec<&str> = target.st.active.iter().map(String::as_str).filter(|s| status::modifies_rating(s, Rating::Armor)).collect();
    let at = |adj: &Adjust| intent::hit_on_adjusted(w, sched, a, t, false, adj).map_or(hit.damage, |h| h.damage);
    let mut mods = Vec::new();
    for &(s, f) in &power {
        mods.push(HitMod { source: s.to_string(), pct: pct(hit.damage, at(&Adjust { power: 1.0 / f, ..Adjust::NONE })) });
    }
    for &s in &armour {
        mods.push(HitMod { source: s.to_string(), pct: pct(hit.damage, at(&Adjust { strip: &[s], ..Adjust::NONE })) });
    }
    if hit.guarded {
        mods.push(HitMod { source: status::GUARD.to_string(), pct: pct(hit.damage, at(&Adjust { guard: false, ..Adjust::NONE })) });
    }
    mods.retain(|m| m.pct != 0);
    let all = Adjust { power: power.iter().fold(1.0, |p, (_, f)| p / f), strip: &armour, guard: false };
    let base = if power.is_empty() && armour.is_empty() && !hit.guarded { hit.damage } else { at(&all) };
    Some(HitParts { damage: hit.damage, base, mods })
}

/// SV7, SV9: one hit of a unit's planned loss read for show: its breakdown and the counter it draws (`counter_on`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HitRead {
    pub parts: HitParts,
    pub counter: Option<i32>,
}

/// SV6, SV7, SV9: every hit in `u`'s planned loss (`planned_loss`'s hits, one for one; on a foe the foe card's hits,
/// `intent::foe_card`), each with its breakdown and the counter it draws. Read only.
pub fn loss_reads(w: &World, sched: &[ScheduledAction], u: UnitId) -> Vec<HitRead> {
    let foes = w.foes_of(u);
    let mut acts: Vec<(usize, &ScheduledAction)> = sched.iter().enumerate().filter(|(_, a)| !a.cancelled && a.unit != u && foes.contains(&a.unit)).collect();
    acts.sort_by_key(|(k, a)| (a.impact, a.start, *k));
    // FB-370: the counter answers once per attacker per round, so only an attacker's first answered hit shows it
    let mut answered: Vec<UnitId> = Vec::new();
    acts.into_iter()
        .filter_map(|(_, a)| {
            let parts = hit_parts(w, sched, a, u)?;
            let counter = counter_on(w, sched, a, u).filter(|_| !answered.contains(&a.unit));
            if counter.is_some() {
                answered.push(a.unit);
            }
            Some(HitRead { parts, counter })
        })
        .collect()
}

/// SV9: the likely return of `defender`'s counter on `attacker` (RiposteRules.counter: the ruleset's `counter_mult` x the
/// defender's attack-power factors, no crit), when the defender holds a `reactive` status; None otherwise. It depends
/// on the attacker's armour, so it is read per hero. Read only.
pub fn counter_return(w: &World, defender: UnitId, attacker: UnitId) -> Option<i32> {
    let d = w.units.get(defender)?;
    let a = w.units.get(attacker)?;
    if !d.st.active.iter().any(|s| status::has_reactive_counter(s)) || !d.is_alive() {
        return None;
    }
    let mult = w.ruleset.numbers.counter_mult * d.weaken_factor() * d.empower_factor();
    Some(damage::compute(d, a, HitInput { skill_mult: mult, ..HitInput::default() }).final_dmg)
}

/// SV9 (FB-370: by the rule, not the skill kind): the counter the hit `a` would draw from `t` (`counter_return` on the
/// action's unit) when the riposte rule would answer it (`command::riposte_answers_at`): any hit of an Attack or an
/// Area on `t` that is not ranged nor a flank, from a tile adjacent to `t`, both read where they stand at its impact
/// (the plan's moves first; when both are planned onto one tile, `t` where it stands now). None otherwise. Once per attacker is the caller's (`loss_reads`).
pub fn counter_on(w: &World, sched: &[ScheduledAction], a: &ScheduledAction, t: UnitId) -> Option<i32> {
    intent::hit_on(w, sched, a, t)?;
    let src = &w.units[a.unit];
    let ranged = match &a.command.kind {
        CmdKind::Attack { use_offhand, .. } => src.attack_range > 1 && !use_offhand,
        CmdKind::Area { .. } => src.attack_range > 1,
        _ => return None,
    };
    let from = intent::position_at(w, sched, a.unit, a.impact);
    // both planned onto one tile (a forecast clash: one of them stops short when the round runs): `t` where it stands
    let at = Some(intent::position_at(w, sched, t, a.impact)).filter(|p| *p != from).unwrap_or_else(|| w.grid.pos(t).unwrap_or(crate::grid::NO_CELL));
    let flank = crate::round_intent::flank_bonus_from(w, a.unit, from, t);
    if !crate::command::riposte_answers_at(w, a.unit, from, t, at, ranged || flank) {
        return None;
    }
    counter_return(w, t, a.unit)
}

/// SV14, SV15: a unit's one target for the round, as its row in the full timeline view names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowTarget {
    /// A Maneuver, Rest or nothing aimed (a dim dash).
    None,
    /// A unit of the other side (a crimson frame).
    Foe(UnitId),
    /// A unit of its own side (a teal frame and arrow).
    Ally(UnitId),
    /// Itself (teal).
    Myself,
    /// A ground tile (a field spell: a 3 x 3 floor patch, the tile lit).
    Ground(V2),
}

/// SV14: the row's target and whether it is an area skill's main target (the amber ring).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowAim {
    pub target: RowTarget,
    pub area: bool,
}

/// SV14 (TG1: the first skill after the opener fixes the round's one target): the target of `u`'s first live action
/// over `sched` that aims at one, in block order: an Attack's (its lock without one), an Area's main target (`area`) or
/// its tile, a Support's or an Item's unit; moves, guards, focus, rests and locks aim at none. Read only.
pub fn row_aim(w: &World, sched: &[ScheduledAction], u: UnitId) -> RowAim {
    let side = |t: UnitId| if t == u { RowTarget::Myself } else if w.friends_of(u).contains(&t) { RowTarget::Ally(t) } else { RowTarget::Foe(t) };
    for a in intent::actions_of(u, sched) {
        let aim = match &a.command.kind {
            CmdKind::Attack { target, .. } => target.or(w.units[u].active_target).map(|t| RowAim { target: side(t), area: false }),
            CmdKind::Area { target: Some(t), .. } => Some(RowAim { target: side(*t), area: true }),
            CmdKind::Area { aim, .. } => Some(RowAim { target: RowTarget::Ground(*aim), area: true }),
            CmdKind::Support { target, .. } | CmdKind::Item { target, .. } => Some(RowAim { target: side(*target), area: false }),
            _ => None,
        };
        if let Some(aim) = aim {
            return aim;
        }
    }
    RowAim { target: RowTarget::None, area: false }
}
