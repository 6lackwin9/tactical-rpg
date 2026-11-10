//! The round intent auto planner (RULESET1_INTENT 4, 7, 8; RULESET1-PL; engine-only, no Godot original): from a
//! unit's intent, round target, Maneuver kind and optional opener it builds the round's steps through `Plan::add_step`
//! (every rule is checked there), each with a reason tag (PL6): the before-move (approach, kite, the move to a Guard
//! tile), the opener (FU1, PL3), the reserved after-move (AS3) or the Disengage jump (MV8), the follow-ups from the
//! member's order (`deck.order`, FU2-FU4) and Rest on every block left (RS1). `fill_rest` (MM3) fills the rest after steps
//! already placed. Deterministic (PL1): orders are lists, tiles are ranked by PL4 down to the cell
//! index, nothing draws a random number. Inert unless the battle's ruleset has `round_intent`.
//!
//! Picks (INPUT1-U, INPUT1 2-5 and 7): a member's round may hold the player's picks in order (a skill on the round
//! target, an item, Rest, Move here, Step here after); its plan is then the picks, each with its implied moves, and
//! the ghost after them (the same fill, or Rest with Auto-fill off), every pick checked with the menu's refusal words.
//! A member with no pick plans exactly `plan_unit`'s plan, so the sims, the AI and the goldens never see a pick.

use crate::command;
use crate::grid::{V2, GRID_H, GRID_W};
use crate::path::{self, Cells};
use crate::plan::{carries, Mode, Plan, PlanStep, SimState, Step};
use crate::round_intent::{self, Aim, ManeuverKind, Opener, RoundIntent, RoundTarget};
use crate::skill::{self, Kind, SkillId};
use crate::world::{UnitId, World};
use rpgkit_data::{Deck, SkillDef};
use std::rc::Rc;

/// PL6 reason tags.
pub const APPROACH: &str = "approach";
pub const APPROACH_OUT_OF_RANGE: &str = "approach: out of range";
pub const KITE_IN_MELEE: &str = "kite: in melee";
pub const KITE_THREATENED: &str = "kite: threatened";
pub const FLANK_TILE: &str = "flank tile";
pub const COVER_TILE: &str = "cover tile";
pub const OPENER: &str = "opener";
pub const RETREAT: &str = "retreat";
pub const REST: &str = "rest";

/// MV11: the kinds the planner tries, in order, when the player picked none.
pub const AUTO_KINDS: [ManeuverKind; 3] = [ManeuverKind::Engage, ManeuverKind::Disengage, ManeuverKind::Utility];

// ---- Inputs (RULESET1_TASKS 1.8 IntentChoice) -------------------------------------------------------------------

/// The unit's deck (FU4; RULESET1 9 D5): its catalog row's `deck`, a party member's saved one; None when empty (auto).
pub fn deck_of(w: &World, c: UnitId) -> Option<&Deck> {
    Some(&w.units[c].deck).filter(|d| !d.is_empty())
}

/// The row of skill family `fam` that `c` uses now: its highest tier the unit can use within its skill cap; None when
/// it has none.
pub fn family_skill(w: &World, c: UnitId, fam: &str) -> Option<SkillId> {
    let u = &w.units[c];
    let cap = w.skill_cap_of(c);
    let row = w.cat.family_rows(fam).into_iter().rev().find(|s| s.level <= cap && !skill::is_item(s) && u.can_use(&w.cat, s))?;
    w.book().find(&row.id)
}

/// The families of FU4's auto order before the intent filter: the deck's actives (under the deck rule the unit's
/// active slots, RULESET1 9 D2; without a deck: every family the unit can use that is not a basic, in catalog order),
/// then its meter skills (HOOKS1 UM6, rule unit_meters: in the order the battle gave them) and its worn items' skills
/// (U3.2-S4, OFFHAND1), then the basics (the
/// ruleset's `basics` in their order; a row without any: the catalog's basic skills), then (CLASS-BASICS) the unit's
/// class basics its deck rule holds.
fn auto_families(w: &World, c: UnitId) -> Vec<String> {
    let mut basics: Vec<String> = if w.ruleset.numbers.basics.is_empty() { w.cat.basic_ids().iter().map(|id| w.cat.family_of(id).to_string()).collect() } else { w.ruleset.numbers.basics.clone() };
    for b in w.units[c].deck_rule.iter().flat_map(|r| &r.basics) {
        if !basics.contains(b) {
            basics.push(b.clone());
        }
    }
    let slots = Some(w.units[c].active_families()).filter(|a| w.units[c].deck_rule.is_some() && !a.is_empty());
    let mut out: Vec<String> = match slots.or_else(|| deck_of(w, c).filter(|d| !d.active.is_empty()).map(|d| d.active.clone())) {
        Some(active) => active,
        None => {
            let mut fams: Vec<String> = Vec::new();
            for s in w.cat.skills.iter().filter(|s| !s.system && !s.basic && !skill::is_item(s)) {
                let fam = s.family_id().to_string();
                if !fams.contains(&fam) && !basics.contains(&fam) && family_skill(w, c, &fam).is_some() {
                    fams.push(fam);
                }
            }
            fams
        }
    };
    // U3.2-S4 (OFFHAND1): the worn items' skills too, open while worn
    for id in w.units[c].meter_skills.iter().chain(&w.units[c].gear_skills) {
        let fam = w.cat.family_of(id).to_string();
        if !out.contains(&fam) {
            out.push(fam);
        }
    }
    for b in basics {
        if !out.contains(&b) {
            out.push(b);
        }
    }
    out
}

/// RULESET1_INTENT FU4: `c`'s order for `intent`: the skills (the row of each family the unit uses now) of its
/// `deck.order[intent]`, else the auto order (the deck's actives of that intent in slot order, then the basics of that
/// intent); only skills of the intent the unit can use, each once. A4.2-K1b (INTENT2 IR15): the player's order may hold
/// a skill in any of its rows; the auto order holds only skills whose primary row is the intent (a second row is by
/// pick or by the player's order).
pub fn order(w: &World, c: UnitId, intent: RoundIntent) -> Vec<SkillId> {
    // PK1-S2a (PK1 4.3 item 4): a Channel round fills in the Channel order (marrow versions only)
    if crate::channel::is_channel(w, c) {
        return crate::channel::order(w, c, intent, &auto_families(w, c));
    }
    let authored = deck_of(w, c).and_then(|d| d.order.get(intent.id())).filter(|l| !l.is_empty());
    let fams: Vec<String> = match authored {
        Some(list) => list.clone(),
        None => auto_families(w, c),
    };
    let fits = |s: &SkillDef| if authored.is_some() { round_intent::skill_rows(w, s).contains(&intent.id()) } else { round_intent::skill_primary(w, s) == intent.id() };
    let mut out: Vec<SkillId> = Vec::new();
    for f in &fams {
        if let Some(sid) = family_skill(w, c, f) {
            if fits(&w.cat.skills[sid]) && !out.contains(&sid) {
                out.push(sid);
            }
        }
    }
    // PK1-S2a: a Cold Steel round leaves the marrow-only skills out (nothing is left out without the rule channel)
    if crate::channel::on(w) {
        out.retain(|&sid| crate::channel::problem(w, c, &w.cat.skills[sid]).is_none());
    }
    out
}

/// INPUT1 FL2: the fill's order after `opener`: the skills of `c`'s `deck.follow_ups[<opener family>]` (in its order;
/// only families still in the unit's deck that it can use in this battle, of `intent`), then `order(w, c, intent)`;
/// a skill in both at its first place. An item opener, no opener or no list for it: `order` alone (so every unit
/// without lists plans as before).
pub fn follow_order(w: &World, c: UnitId, intent: RoundIntent, opener: Option<&Opener>) -> Vec<SkillId> {
    let base = order(w, c, intent);
    // PK1-S2a (PK1 4.3 item 4): a Channel round follows the Channel order, not the follow-up lists
    if crate::channel::is_channel(w, c) {
        return base;
    }
    let Some(Opener::Skill(sid)) = opener else { return base };
    let Some(fam) = w.cat.skills.get(*sid).map(|s| s.family_id().to_string()) else { return base };
    let Some(list) = deck_of(w, c).and_then(|d| d.follow_ups_of(&fam)) else { return base };
    let in_deck = auto_families(w, c);
    let mut out: Vec<SkillId> = Vec::new();
    for f in list.iter().filter(|f| in_deck.contains(f)) {
        if let Some(s) = family_skill(w, c, f) {
            // A4.2-K1b (IR15): a follow-up list may hold a skill in any of its rows (PK1-S2a: not a marrow-only one)
            if round_intent::skill_rows(w, &w.cat.skills[s]).contains(&intent.id()) && !out.contains(&s) && crate::channel::problem(w, c, &w.cat.skills[s]).is_none() {
                out.push(s);
            }
        }
    }
    for s in base {
        if !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

/// PL1, FU1: sets (Some) or clears (None) the opener of `c`'s round.
pub fn set_opener(w: &mut World, c: UnitId, opener: Option<Opener>) -> Result<(), String> {
    if !round_intent::on(w) {
        return Err("This battle has no round intents.".into());
    }
    match &opener {
        Some(Opener::Skill(sid)) if *sid >= w.cat.skills.len() => return Err("Unknown skill.".into()),
        Some(Opener::Item(id)) if command::pouch_item(w, c, id).is_none() => return Err(format!("{} carries no {id}.", w.units[c].name)),
        _ => {}
    }
    w.round_states.entry(c).or_default().opener = opener;
    Ok(())
}

/// MV11: the player's Maneuver kind (Some) or the planner's pick (None: Engage, else Disengage, else Utility).
pub fn set_kind(w: &mut World, c: UnitId, kind: Option<ManeuverKind>) -> Result<(), String> {
    match kind {
        Some(k) => round_intent::set_maneuver(w, c, k),
        None => {
            if round_intent::intent_of(w, c) != Some(RoundIntent::Maneuver) {
                return Err("Engage, Disengage and Utility are Maneuver kinds.".into());
            }
            let rs = w.round_states.entry(c).or_default();
            rs.maneuver = None;
            rs.maneuver_auto = false;
            Ok(())
        }
    }
}

/// TG6: the target the planner proposes for `c` under `intent`: the round's set target, else last round's while valid,
/// else (Assault, Maneuver) the lock, else the nearest active foe (distance, then roster index); Guard and Support:
/// the unit itself. A4.2-K1b (IR5): Field, a tile: the lock's tile, else the nearest active foe's.
pub fn proposed_target(w: &World, c: UnitId, intent: RoundIntent) -> Option<RoundTarget> {
    let rs = w.round_states.get(&c);
    let valid = |t: RoundTarget| round_intent::target_problem(w, c, intent, t).is_empty();
    if let Some(t) = rs.and_then(|r| r.target).filter(|t| valid(*t)) {
        return Some(t);
    }
    if let Some(t) = rs.and_then(|r| r.last_target).filter(|t| valid(*t)) {
        return Some(t);
    }
    match intent {
        RoundIntent::Assault | RoundIntent::Maneuver => {
            if let Some(t) = w.units[c].active_target.map(RoundTarget::Unit).filter(|t| valid(*t)) {
                return Some(t);
            }
            let here = w.pos(c);
            w.foes_of(c).iter().copied().filter(|&f| valid(RoundTarget::Unit(f))).min_by_key(|&f| (w.pos(f).dist(here), f)).map(RoundTarget::Unit)
        }
        RoundIntent::Field => {
            let on_field = |f: UnitId| w.units[f].is_active() && w.has(f) && !round_intent::hidden(w, f);
            if let Some(t) = w.units[c].active_target.filter(|&t| w.foes_of(c).contains(&t) && on_field(t)) {
                return Some(RoundTarget::Cell(w.pos(t)));
            }
            let here = w.pos(c);
            w.foes_of(c).iter().copied().filter(|&f| on_field(f)).min_by_key(|&f| (w.pos(f).dist(here), f)).map(|f| RoundTarget::Cell(w.pos(f)))
        }
        _ => Some(RoundTarget::Myself),
    }
}

// ---- Entry points -----------------------------------------------------------------------------------------------

/// RULESET1_INTENT PL1-PL7: rebuilds `c`'s round from its intent, target (proposed when unset: TG6), kind and opener.
/// A Maneuver round whose kind the player did not pick tries Engage, then Disengage, then Utility (MV11) and keeps the
/// first that has an opener. Err = the intent (or kind) is not offered (PL7); `c`'s steps are then empty.
pub fn plan_unit(w: &mut World, plan: &mut Plan, c: UnitId) -> Result<(), String> {
    if !round_intent::on(w) {
        return Err("This battle has no round intents.".into());
    }
    let Some(intent) = round_intent::intent_of(w, c) else { return Err("Pick a round intent first.".into()) };
    if w.round_states.get(&c).and_then(|r| r.target).is_none() {
        if let Some(t) = proposed_target(w, c, intent) {
            round_intent::set_target(w, c, t)?;
        }
    }
    let rs = w.round_states.entry(c).or_default();
    if intent != RoundIntent::Maneuver || (rs.maneuver.is_some() && !rs.maneuver_auto) {
        return build(w, plan, c);
    }
    let mut first = None;
    for kind in AUTO_KINDS {
        let rs = w.round_states.entry(c).or_default();
        rs.maneuver = Some(kind);
        rs.maneuver_auto = true;
        match build(w, plan, c) {
            Ok(()) => return Ok(()),
            Err(e) => {
                first.get_or_insert(e);
            }
        }
    }
    let rs = w.round_states.entry(c).or_default();
    rs.maneuver = None;
    rs.maneuver_auto = false;
    plan.set_steps(c, Vec::new());
    Err(first.unwrap_or_default())
}

/// PL8: rebuilds `c`'s round, then every later member of its side in roster order that holds an intent and plans
/// automatically (approach tiles are claims; a member with picks is rebuilt from them, INPUT1 GH5). Returns `c`'s
/// result and the later members' refusals.
pub fn replan(w: &mut World, plan: &mut Plan, c: UnitId) -> (Result<(), String>, Vec<(UnitId, String)>) {
    let first = plan_unit(w, plan, c);
    (first, rebuild_later(w, plan, c))
}

/// RULESET1_TASKS 1.8, INPUT1 GH2 b: the fields of one round change (the host's `round_intent` request), each optional;
/// `kind` and `opener` set to Some(None) hand the choice back to the planner.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RoundSet {
    pub intent: Option<RoundIntent>,
    pub target: Option<RoundTarget>,
    pub kind: Option<Option<ManeuverKind>>,
    pub opener: Option<Option<Opener>>,
}

/// What `set_round` did: IN6's lines (the buffs an intent change dropped), `c`'s rebuild result and the later members'
/// refusals (PL8).
#[derive(Clone, Debug)]
pub struct RoundSetDone {
    pub lines: Vec<String>,
    pub planned: Result<(), String>,
    pub refused: Vec<(UnitId, String)>,
}

/// IN2, IN6: sets `c`'s round intent; a change of intent drops the intent-bound buffs (the lines name them) and the old
/// steps (`Session::set_round_intent` goes through here).
pub fn change_intent(w: &mut World, plan: &mut Plan, c: UnitId, intent: RoundIntent) -> Result<Vec<String>, String> {
    let before = round_intent::intent_of(w, c);
    let lines = round_intent::set_intent(w, c, intent)?;
    if before.is_some_and(|b| b != intent) {
        plan.clear(c);
    }
    Ok(lines)
}

/// RULESET1_TASKS 1.8 (the host's `round_intent`), INPUT1 GH2 b: sets each given field in order (the intent first,
/// then the target, the Maneuver kind, the opener); then the member gets its round rebuilt, and every later member's
/// (`replan`). Err = the first refused field; the world and the plan may be half changed then (the
/// caller restores them).
pub fn set_round(w: &mut World, plan: &mut Plan, c: UnitId, set: &RoundSet) -> Result<RoundSetDone, String> {
    let mut done = RoundSetDone { lines: Vec::new(), planned: Ok(()), refused: Vec::new() };
    if let Some(i) = set.intent {
        done.lines = change_intent(w, plan, c, i)?;
    }
    if let Some(t) = set.target {
        round_intent::set_target(w, c, t)?;
    }
    if let Some(k) = set.kind {
        set_kind(w, c, k)?;
    }
    if let Some(o) = &set.opener {
        set_opener(w, c, o.clone())?;
    }
    (done.planned, done.refused) = replan(w, plan, c);
    Ok(done)
}

/// INPUT1 GH2 b, IN6 timing (decision "INPUT1 answers, part 4" Q18): copies of the world and the plan as `set_round`
/// would leave them; nothing changes. The battle screen draws a round change that waits for Confirm from them, and
/// Confirm sends that change to `set_round`, so both read one source. Err = a refused field.
pub fn preview_round(w: &World, plan: &Plan, c: UnitId, set: &RoundSet) -> Result<(World, Plan), String> {
    let mut w2 = w.clone();
    let mut p2 = plan_copy(plan);
    set_round(&mut w2, &mut p2, c, set)?;
    Ok((w2, p2))
}

/// PL8, INPUT1 GH5: rebuilds every member of `c`'s side after it in roster order: a member with picks from its picks
/// (a pick that no longer fits is dropped with every pick after it: "<hero>: <skill> dropped: <reason>"), one without
/// from its intent (`plan_unit`; Auto-fill off: Rest). Members without an intent are left alone.
/// Returns the drops and the refusals.
fn rebuild_later(w: &mut World, plan: &mut Plan, c: UnitId) -> Vec<(UnitId, String)> {
    let side: Vec<UnitId> = if w.allies.contains(&c) { w.allies.clone() } else { w.enemies.clone() };
    let mut refused = Vec::new();
    for &m in side.iter().skip_while(|&&m| m != c).skip(1) {
        if !w.units[m].is_active() {
            continue;
        }
        if picks_of(w, m).is_empty() {
            if round_intent::intent_of(w, m).is_none() {
                continue;
            }
            if let Err(e) = rebuild_member(w, plan, m) {
                refused.push((m, e));
            }
            continue;
        }
        while let Err((i, why)) = build_picks(w, plan, m) {
            let picks = picks_of(w, m);
            let i = i.min(picks.len() - 1);
            refused.push((m, format!("{}: {} dropped: {why}", w.units[m].name, pick_name(w, &picks[i]))));
            set_picks(w, m, picks[..i].to_vec());
            if picks_of(w, m).is_empty() {
                let _ = rebuild_member(w, plan, m);
                break;
            }
        }
    }
    refused
}

/// MM3 "Fill the rest": PL5's fill after the last step placed by hand (follow-ups from the order on the round target,
/// from the tile the unit stands on then), then Rest on every free block.
pub fn fill_rest(w: &World, plan: &mut Plan, c: UnitId) -> Result<(), String> {
    let b = Builder::new(w, c)?;
    // FL2: the round's opener (set, else the first planned skill of the intent) chooses the follow-up list
    let opener = w.round_states.get(&c).and_then(|r| r.opener.clone()).or_else(|| {
        plan.steps_of(c).iter().filter_map(|st| st.skill).find(|&sid| sid < w.cat.skills.len() && round_intent::skill_rows(w, &w.cat.skills[sid]).contains(&b.intent.id())).map(Opener::Skill)
    });
    let cause = b.fill(plan, &Reserve::default(), &follow_order(w, c, b.intent, opener.as_ref()));
    b.rest(plan, &cause);
    Ok(())
}

/// The plan `intent` (and `kind` for Maneuver) would make for `c` now, on a copy of the world (UI2's disabled reasons,
/// the Dojo's plan per intent, UI4): its steps, or the reason it is not offered (PL7). Nothing changes.
pub fn preview(w: &World, plan: &Plan, c: UnitId, intent: RoundIntent, kind: Option<ManeuverKind>) -> Result<Vec<Step>, String> {
    let mut w2 = w.clone();
    let mut p2 = Plan::new();
    p2.sequences = plan.sequences.clone();
    p2.path_penalties = plan.path_penalties.clone();
    p2.goal_preferences = plan.goal_preferences.clone();
    if round_intent::intent_of(&w2, c) != Some(intent) {
        round_intent::set_intent(&mut w2, c, intent)?;
    }
    if let Some(k) = kind {
        round_intent::set_maneuver(&mut w2, c, k)?;
    }
    plan_unit(&mut w2, &mut p2, c)?;
    Ok(p2.steps_of(c))
}

/// RULESET1_INTENT UI2 ("Rest: 1 AP, 10 poise"): what the Rest blocks of `c`'s plan would give: the AP at round end
/// (RS2) and the poise their landing restores (RS3, rule `poise`; before the Max Poise cap). S2.2-S3 (FB-71): under
/// `poise_mode` pips the pips instead (a pip per 2 Rest blocks, before the `poise_pips` cap; `poise` 0 there).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RestGain {
    pub blocks: i32,
    pub ap: i32,
    pub poise: i32,
    /// S2.2-S3: the pips the Rest blocks restore under `poise_mode` pips (0 under `bar`).
    pub pips: i32,
}

/// UI2: the Rest gain of `c`'s planned round (its compiled Rest entries; the Dojo's planner view uses it too).
pub fn rest_preview(w: &World, plan: &Plan, c: UnitId) -> RestGain {
    let blocks = plan.entries_for(w, c).iter().filter(|e| skill::kind(&w.cat.skills[e.skill]) == Kind::Rest).map(|e| e.end_block(w) - e.start_block + 1).sum();
    rest_of(w, blocks)
}

/// The Rest gain of `blocks` Rest blocks under the battle's ruleset.
pub fn rest_of(w: &World, blocks: i32) -> RestGain {
    let n = &w.ruleset.numbers;
    let on = w.ruleset.has(rpgkit_rules::ruleset::POISE);
    // S2.2-S3 (FB-71): by pips under poise_mode pips (from a round with no Rest landed, below the cap)
    let pips_mode = crate::poise::pips_mode(n);
    let poise = if on && !pips_mode { n.rest_poise_per_block * blocks } else { 0 };
    let pips = if on { crate::poise::rest_pip_gain(n, 0, 0, blocks) } else { 0 };
    RestGain { blocks, ap: round_intent::rest_gain(n, blocks), poise, pips }
}

/// PL6: the reason tag of each of `c`'s compiled entries (aligned with `Plan::entries_for`): a move entry a step
/// implies takes the step's move tag, every other entry its action tag; a carried move adds "flank tile" (Engage,
/// Utility) or "retreat" (Disengage). Lost blocks and steps placed by hand: "".
pub fn reasons(w: &World, plan: &Plan, c: UnitId) -> Vec<String> {
    let steps = plan.steps_of(c);
    plan.compile(w, c)
        .entries
        .iter()
        .map(|e| {
            let Some(step) = usize::try_from(e.step_index).ok().and_then(|i| steps.get(i)) else { return String::new() };
            let is_move = skill::kind(&w.cat.skills[e.skill]) == Kind::Move;
            if is_move && !matches!(step.mode, Mode::Move | Mode::Goto | Mode::Withdraw) && !step.move_reason.is_empty() {
                return step.move_reason.clone();
            }
            let carry = match e.carry {
                command::Carry::Engage(cell) if cell != e.pos_before => FLANK_TILE,
                command::Carry::Engage(_) => "",
                command::Carry::Retreat(_) => RETREAT,
                command::Carry::None => "",
            };
            match (step.reason.is_empty(), carry.is_empty()) {
                (_, true) => step.reason.clone(),
                (true, false) => carry.to_string(),
                (false, false) => format!("{}; {carry}", step.reason),
            }
        })
        .collect()
}

// ---- The builder ----------------------------------------------------------------------------------------------

/// A step with its reason tags (a fresh step: nothing else holds it yet).
fn tagged(mut step: Step, reason: &str, move_reason: &str) -> Step {
    if let Some(s) = Rc::get_mut(&mut step) {
        s.reason = reason.to_string();
        s.move_reason = move_reason.to_string();
    }
    step
}

/// What PL5 keeps back for after the fill: blocks at the round's end, AP, and the step that uses them.
#[derive(Default)]
struct Reserve {
    blocks: i32,
    ap: i32,
    step: Option<Step>,
}

struct Builder<'a> {
    w: &'a World,
    c: UnitId,
    intent: RoundIntent,
    target: RoundTarget,
    kind: ManeuverKind,
    order: Vec<SkillId>,
}

/// The whole auto plan for `c` (its world state holds intent, target, kind and opener).
fn build(w: &World, plan: &mut Plan, c: UnitId) -> Result<(), String> {
    plan.set_steps(c, Vec::new());
    let b = Builder::new(w, c)?;
    let opener = w.round_states.get(&c).and_then(|r| r.opener.clone());
    let opener = match b.opener(plan, opener) {
        Ok(o) => o,
        Err(e) => {
            plan.set_steps(c, Vec::new());
            return Err(e);
        }
    };
    let reserve = b.reserve(plan);
    // FL2: the opener placed (given, else PL3's) chooses the follow-up list
    let cause = b.fill(plan, &reserve, &follow_order(w, c, b.intent, Some(&opener)));
    if let Some(step) = reserve.step {
        // PL5: the after-move (or the Disengage jump) right after the fill; dropped when it no longer fits
        let _ = plan.add_step(w, c, step, false);
    }
    b.rest(plan, &cause);
    Ok(())
}

impl<'a> Builder<'a> {
    fn new(w: &'a World, c: UnitId) -> Result<Self, String> {
        if !round_intent::on(w) {
            return Err("This battle has no round intents.".into());
        }
        let Some(intent) = round_intent::intent_of(w, c) else { return Err("Pick a round intent first.".into()) };
        if intent == RoundIntent::Channel {
            return Err("Channel rounds are not planned yet.".into());
        }
        let Some(target) = w.round_states.get(&c).and_then(|r| r.target).or_else(|| proposed_target(w, c, intent)) else {
            return Err(format!("{} has no target.", intent.label(&w.cat)));
        };
        let p = round_intent::target_problem(w, c, intent, target);
        if !p.is_empty() {
            return Err(p);
        }
        Ok(Self { w, c, intent, target, kind: round_intent::maneuver_of(w, c), order: order(w, c, intent) })
    }

    fn skill(&self, sid: SkillId) -> &'a SkillDef {
        &self.w.cat.skills[sid]
    }

    fn target_unit(&self) -> Option<UnitId> {
        match self.target {
            RoundTarget::Unit(t) => Some(t),
            RoundTarget::Myself => Some(self.c),
            RoundTarget::Cell(_) => None,
        }
    }

    fn target_pos(&self, st: &SimState) -> V2 {
        match self.target {
            RoundTarget::Unit(t) => self.w.pos(t),
            RoundTarget::Myself => st.pos,
            RoundTarget::Cell(p) => p,
        }
    }

    fn target_name(&self) -> String {
        match self.target {
            RoundTarget::Unit(t) => self.w.units[t].name.clone(),
            RoundTarget::Myself => "yourself".into(),
            RoundTarget::Cell(p) => p.to_string(),
        }
    }

    /// The label of PL7's refusal: the intent, or under Maneuver the kind.
    fn label(&self) -> String {
        match (self.intent, self.kind) {
            (RoundIntent::Maneuver, ManeuverKind::Engage) => "Engage".into(),
            (RoundIntent::Maneuver, ManeuverKind::Disengage) => "Disengage".into(),
            (RoundIntent::Maneuver, ManeuverKind::Utility) => "Utility".into(),
            (i, _) => i.label(&self.w.cat),
        }
    }

    /// A Disengage round (MV8).
    fn disengage(&self) -> bool {
        self.intent == RoundIntent::Maneuver && self.kind == ManeuverKind::Disengage
    }

    fn mv(&self) -> SkillId {
        self.w.book().lowest(Kind::Move)
    }

    /// Adds `steps` in order (no AP debt: PL7); on a refusal the plan is back where it was and the reason returned.
    fn add_all(&self, plan: &mut Plan, steps: Vec<Step>) -> Result<(), String> {
        let mark = plan.steps_of(self.c).len();
        for step in steps {
            let p = plan.add_step(self.w, self.c, step, false);
            if !p.is_empty() {
                plan.truncate(self.c, mark);
                return Err(p);
            }
        }
        Ok(())
    }

    // ---- PL4 tiles ------------------------------------------------------------------------------------------------

    /// Free tiles `c` can walk to from `from` within `max_blocks` standard move blocks whose distance to `goal` lies in
    /// `lo..=hi`, with their routes.
    fn walk_tiles(&self, plan: &Plan, from: V2, goal: V2, lo: i32, hi: i32, max_blocks: usize) -> Vec<(V2, Vec<V2>)> {
        let w = self.w;
        let blocked = plan.blocked_cells(w, self.c);
        let mpb = w.units[self.c].move_per_block();
        let mut out = Vec::new();
        for y in 0..GRID_H {
            for x in 0..GRID_W {
                let cell = V2::new(x, y);
                let d = cell.dist(goal);
                if cell == from || d < lo || d > hi || cell.dist(from) > mpb * max_blocks as i32 || !w.grid.is_walkable(cell) || w.grid.is_exit(cell) || blocked.contains(&cell) {
                    continue;
                }
                let goals: Cells = [cell].into_iter().collect();
                let route = plan.route(w, self.c, from, &goals, &blocked, false);
                if !route.is_empty() && path::blocks_needed(&w.grid, from, &route, mpb) <= max_blocks {
                    out.push((cell, route));
                }
            }
        }
        out
    }

    /// PL4: the best tile of `cands` (threat, cover for Guard and ranged units, path tiles, distance by `aim`, index).
    fn best(&self, goal: V2, aim: Aim, mut cands: Vec<(V2, Vec<V2>)>) -> Option<V2> {
        let cover = self.intent == RoundIntent::Guard || self.w.units[self.c].attack_range > 1;
        round_intent::rank_tiles(self.w, self.c, goal, cover, aim, &mut cands);
        cands.first().map(|(cell, _)| *cell)
    }

    /// AS1, SP1, MV10: the standard approach tile (at most the standard move's blocks) with `goal` in `lo..=hi`.
    fn approach_tile(&self, plan: &Plan, from: V2, goal: V2, lo: i32, hi: i32) -> Option<V2> {
        self.best(goal, Aim::Toward, self.walk_tiles(plan, from, goal, lo, hi, crate::plan::MAX_MOVE_BLOCKS))
    }

    // ---- Opener (FU1, PL3, AS1, AS2, GD1, SP1, MV7-MV10) ----------------------------------------------------------

    /// Places the before-move and the opener: the given one, else the first order entry that fits (PL3); returns the
    /// opener placed.
    fn opener(&self, plan: &mut Plan, given: Option<Opener>) -> Result<Opener, String> {
        if let Some(o) = given {
            return self.try_opener(plan, &o).map(|()| o);
        }
        if self.order.is_empty() {
            return Err(format!("{} has no {} skill.", self.w.units[self.c].name, self.intent.label(&self.w.cat)));
        }
        let mut first: Option<String> = None;
        for &sid in &self.order {
            // MV8: a DISENGAGE skill (a jump) belongs to Disengage rounds, and there goes last while a Maneuver attack
            // can open
            if k_is_jump(self.skill(sid)) && (!self.disengage() || self.order.iter().any(|&o| !k_is_jump(self.skill(o)))) {
                continue;
            }
            match self.try_opener(plan, &Opener::Skill(sid)) {
                Ok(()) => return Ok(Opener::Skill(sid)),
                Err(e) => {
                    first.get_or_insert(e);
                }
            }
        }
        Err(match self.target {
            // INPUT1 MR5: every Engage that could open found the target's flank protected
            RoundTarget::Unit(t) if t != self.c && first.as_deref() == Some(round_intent::FLANK_PROTECTED) => round_intent::FLANK_PROTECTED.to_string(),
            RoundTarget::Unit(t) if t != self.c => format!("No {} skill reaches {} this round.", self.label(), self.target_name()),
            // GD1: a Guard tile the standard move cannot reach is the reason
            _ => format!("No {} skill fits this round: {}", self.label(), first.unwrap_or_default()),
        })
    }

    fn try_opener(&self, plan: &mut Plan, o: &Opener) -> Result<(), String> {
        let st = plan.compile(self.w, self.c);
        let steps = match o {
            Opener::Skill(sid) => self.skill_opener(plan, &st, *sid)?,
            Opener::Item(id) => self.item_opener(plan, &st, id)?,
        };
        self.add_all(plan, steps)
    }

    fn skill_opener(&self, plan: &Plan, st: &SimState, sid: SkillId) -> Result<Vec<Step>, String> {
        let w = self.w;
        let s = self.skill(sid);
        // PK1-S2a: the round's mode (Channel or Cold Steel) offers only its versions
        if let Some(p) = crate::channel::problem(w, self.c, s) {
            return Err(p.into());
        }
        if !round_intent::skill_rows(w, s).contains(&self.intent.id()) {
            return Err(format!("Only {} skills this round.", self.intent.label(&w.cat)));
        }
        let u = &w.units[self.c];
        let k = skill::kind(s);
        match self.intent {
            RoundIntent::Guard => {
                let mut steps = Vec::new();
                let mut at = st.pos;
                if let RoundTarget::Cell(p) = self.target {
                    if p != st.pos {
                        steps.push(tagged(PlanStep::go_to(self.mv(), p), COVER_TILE, ""));
                    }
                    at = p;
                }
                steps.push(tagged(self.stand_step(sid, at).ok_or_else(|| format!("{}: no enemy around.", s.name))?, OPENER, ""));
                Ok(steps)
            }
            RoundIntent::Support => {
                let self_only = s.target_mode == "SELF" || k == Kind::Focus;
                match self.target {
                    RoundTarget::Unit(t) if t != self.c => {
                        if self_only {
                            return Err(format!("{} acts only on its user.", s.name));
                        }
                        let step = PlanStep::ally(sid, t);
                        let band = skill::support_reach(s, u);
                        let (lo, d) = (band.0.max(1), st.pos.dist(w.pos(t)));
                        if d < lo || d > band.1 {
                            let tile = self.approach_tile(plan, st.pos, w.pos(t), lo, band.1).ok_or("No path to get within reach.")?;
                            step.cell.set(tile);
                        }
                        Ok(vec![tagged(step, OPENER, APPROACH)])
                    }
                    _ => Ok(vec![tagged(PlanStep::self_skill(sid), OPENER, "")]),
                }
            }
            RoundIntent::Assault | RoundIntent::Maneuver => self.attack_opener(plan, st, sid),
            RoundIntent::Channel => Err("Channel rounds are not planned yet.".into()),
            RoundIntent::Field => self.field_opener(plan, st, sid),
        }
    }

    /// A4.2-K1b (INTENT2 IR4, IR5, IR7): a Field opener on the round's source tile (fixed at planning; one source for
    /// all the round's blocks): a standard approach first when the source is out of the Field band (with the `reach`
    /// bonus); a shape with a direction needs the source on a neighbouring tile, never the user's own.
    fn field_opener(&self, plan: &Plan, st: &SimState, sid: SkillId) -> Result<Vec<Step>, String> {
        let w = self.w;
        let s = self.skill(sid);
        let RoundTarget::Cell(source) = self.target else { return Err(format!("{} targets a tile.", self.label())) };
        if !skill::is_area(s) {
            return Err(format!("{} needs a tile to aim at.", s.name));
        }
        let a = skill::area(s);
        let band = if skill::from_user(a) { (1, 1) } else { skill::field_reach(s, &w.units[self.c], &w.ruleset) };
        let d = st.pos.dist(source);
        let mut steps = Vec::new();
        let near = if matches!(a, skill::Area::Arc | skill::Area::Cone) { source != st.pos && (source.x - st.pos.x).abs() <= 1 && (source.y - st.pos.y).abs() <= 1 } else { d >= band.0 && d <= band.1 };
        if !near {
            let tile = self.approach_tile(plan, st.pos, source, band.0.max(1), band.1).ok_or("No path to get within reach.")?;
            steps.push(tagged(PlanStep::go_to(self.mv(), tile), APPROACH, ""));
        }
        steps.push(tagged(PlanStep::area_at(sid, source), OPENER, ""));
        Ok(steps)
    }

    fn attack_opener(&self, plan: &Plan, st: &SimState, sid: SkillId) -> Result<Vec<Step>, String> {
        let w = self.w;
        let s = self.skill(sid);
        let u = &w.units[self.c];
        let maneuver = self.intent == RoundIntent::Maneuver;
        let band = skill::reach_by(s, u, &w.ruleset);
        let goal = self.target_pos(st);
        let d = st.pos.dist(goal);
        if skill::is_area(s) {
            let mut steps = Vec::new();
            if d < band.0 || d > band.1 {
                if maneuver && self.kind == ManeuverKind::Disengage {
                    return Err(format!("Disengage needs {} within reach of {}.", self.target_name(), s.name));
                }
                let tile = self.approach_tile(plan, st.pos, goal, band.0, band.1).ok_or("No path to get within reach.")?;
                steps.push(tagged(PlanStep::go_to(self.mv(), tile), if maneuver { APPROACH_OUT_OF_RANGE } else { APPROACH }, ""));
            }
            steps.push(tagged(self.area_step(sid, goal), OPENER, ""));
            return Ok(steps);
        }
        let Some(foe) = self.target_unit().filter(|&t| t != self.c) else { return Err(format!("{} needs an enemy to aim at.", s.name)) };
        if k_is_jump(s) {
            if !self.disengage() {
                return Err(format!("{} belongs to a Disengage round.", s.name));
            }
            // MV8: a lone jump opens (and retreats) only when nothing else can
            return Ok(vec![tagged(PlanStep::engage(sid, foe), RETREAT, "")]);
        }
        if maneuver && carries(w, s) {
            // MV4-MV9: the carried move rides in the skill (out of range: a plain move first, TQ22 a)
            return Ok(vec![tagged(PlanStep::engage(sid, foe), OPENER, APPROACH_OUT_OF_RANGE)]);
        }
        let in_band = d >= band.0 && d <= band.1;
        if maneuver && self.kind == ManeuverKind::Disengage && !in_band {
            return Err(format!("Disengage needs {} within reach of {}.", self.target_name(), s.name));
        }
        // AS2: a ranged unit with its target in the target's melee band kites first, to the farthest band tile
        if !maneuver && u.attack_range > 1 && d <= round_intent::melee_reach(w, foe) {
            let cands = self.walk_tiles(plan, st.pos, goal, band.0, band.1, 1);
            let far = cands.iter().map(|(cell, _)| cell.dist(goal)).max().ok_or("No tile to kite to.")?;
            let tile = self.best(goal, Aim::Away, cands.into_iter().filter(|(cell, _)| cell.dist(goal) == far).collect()).ok_or("No tile to kite to.")?;
            return Ok(vec![tagged(PlanStep::go_to(self.mv(), tile), KITE_IN_MELEE, ""), tagged(PlanStep::engage(sid, foe), OPENER, "")]);
        }
        let step = PlanStep::engage(sid, foe);
        if !in_band {
            // AS1 (MV9 a for a Maneuver attack without a carried move): the standard approach to the PL4 tile
            let tile = self.approach_tile(plan, st.pos, goal, band.0, band.1).ok_or("No path to get within reach.")?;
            step.cell.set(tile);
        }
        Ok(vec![tagged(step, OPENER, if maneuver { APPROACH_OUT_OF_RANGE } else { APPROACH })])
    }

    /// SP2, MV10: a pouch item opens a Support round (self or ally item) or a Utility round (a foe-aimed item); items
    /// carry no move, so a standard approach into the item's reach comes first.
    fn item_opener(&self, plan: &Plan, st: &SimState, id: &str) -> Result<Vec<Step>, String> {
        let w = self.w;
        let Some(item) = command::pouch_item(w, self.c, id) else { return Err(format!("{} carries no {id}.", w.units[self.c].name)) };
        // PK1-S2a: items have no marrow version, so a Channel round has none
        if let Some(p) = crate::channel::item_problem(w, self.c) {
            return Err(p.into());
        }
        // A1.2-K3 (IR11, IR21): an item opens a round of any of its rows
        if !round_intent::item_allows(w, item, self.intent) {
            return Err(format!("{} is not a {} item.", item.name, self.intent.label(&w.cat)));
        }
        if self.intent == RoundIntent::Maneuver && self.kind != ManeuverKind::Utility {
            return Err(format!("{} opens only a Utility round.", item.name));
        }
        let mut steps = Vec::new();
        // A1.2-K3 (INTENT2 section 6, IR2): an area item's Field row walks into its Field reach of the source tile
        if let RoundTarget::Cell(source) = self.target {
            if !command::item_area_on(w, item) {
                return Err(format!("{} needs a unit to aim at.", item.name));
            }
            let reach = command::item_field_reach(item);
            if st.pos.dist(source) > reach {
                let tile = self.approach_tile(plan, st.pos, source, 1, reach.max(1)).ok_or("No path to get within reach.")?;
                steps.push(tagged(PlanStep::go_to(self.mv(), tile), APPROACH, ""));
            }
            steps.push(tagged(PlanStep::use_item_at(id, source), OPENER, ""));
            return Ok(steps);
        }
        let Some(t) = self.target_unit() else { return Err(format!("{} needs a unit to aim at.", item.name)) };
        let reach = item.battle_reach();
        if t != self.c && st.pos.dist(w.pos(t)) > reach {
            let tile = self.approach_tile(plan, st.pos, w.pos(t), 1, reach.max(1)).ok_or("No path to get within reach.")?;
            steps.push(tagged(PlanStep::go_to(self.mv(), tile), APPROACH, ""));
        }
        steps.push(tagged(PlanStep::use_item(id, t), OPENER, ""));
        Ok(steps)
    }

    // ---- Reserved after-move (AS3) and the Disengage jump (MV8) ----------------------------------------------------

    fn reserve(&self, plan: &Plan) -> Reserve {
        let w = self.w;
        let st = plan.compile(w, self.c);
        let pays = w.pays_ap(self.c);
        if self.intent == RoundIntent::Maneuver && self.kind == ManeuverKind::Disengage {
            let Some(foe) = self.target_unit() else { return Reserve::default() };
            let opened_with_jump = st.entries.iter().any(|e| e.step_index >= 0 && k_is_jump(self.skill(e.skill)));
            let Some(&jump) = self.order.iter().find(|&&sid| k_is_jump(self.skill(sid))) else { return Reserve::default() };
            if opened_with_jump {
                return Reserve::default();
            }
            let s = self.skill(jump);
            let ap = if pays { skill::ap_cost_by(s, Some(&w.units[self.c]), &w.ruleset) } else { 0 };
            return Reserve { blocks: s.blocks + w.recovery_of(self.c, s), ap, step: Some(tagged(PlanStep::engage(jump, foe), RETREAT, "")) };
        }
        // AS3: a ranged Assault unit that has not moved, on a target that threatens its end tile
        let Some(foe) = self.target_unit().filter(|&t| t != self.c && self.intent == RoundIntent::Assault) else { return Reserve::default() };
        let u = &w.units[self.c];
        if u.attack_range <= 1 || st.moved || st.rooted {
            return Reserve::default();
        }
        let Some(opener) = st.entries.iter().find(|e| e.step_index >= 0 && skill::kind(self.skill(e.skill)) == Kind::Attack) else { return Reserve::default() };
        let band = skill::reach_by(self.skill(opener.skill), u, &w.ruleset);
        let (end, goal) = (st.pos, w.pos(foe));
        let f = &w.units[foe];
        let foe_reach = skill::threat_reach(f, &w.ruleset);
        if end.dist(goal) > f.move_per_block() * 2 + foe_reach {
            return Reserve::default();
        }
        let here = end.dist(goal);
        let cands: Vec<(V2, Vec<V2>)> = self.walk_tiles(plan, end, goal, band.0, band.1, 1).into_iter().filter(|(cell, _)| cell.dist(goal) > here).collect();
        let Some((cell, route)) = ({
            let mut c2 = cands;
            round_intent::rank_tiles(w, self.c, goal, true, Aim::Away, &mut c2);
            c2.into_iter().next()
        }) else {
            return Reserve::default();
        };
        let ap = if pays { route.len() as i32 } else { 0 };
        Reserve { blocks: 1, ap, step: Some(tagged(PlanStep::after_move(self.mv(), cell), KITE_THREATENED, "")) }
    }

    // ---- Fill (FU2, FU3) and Rest (RS1) -----------------------------------------------------------------------------

    /// The last block a follow-up may use: before the reserved blocks at the round's end (lost blocks do not count).
    fn limit(&self, st: &SimState, reserve: &Reserve) -> i32 {
        limit_of(st, reserve.blocks)
    }

    /// The follow-up step for skill `sid` on the round target from the unit's tile at `st` (None: it cannot act on
    /// this kind of target).
    fn follow_step(&self, st: &SimState, sid: SkillId) -> Option<Step> {
        let s = self.skill(sid);
        let k = skill::kind(s);
        match self.intent {
            RoundIntent::Guard => self.stand_step(sid, st.pos),
            RoundIntent::Support => match self.target {
                RoundTarget::Unit(t) if t != self.c => (s.target_mode != "SELF" && k != Kind::Focus).then(|| PlanStep::ally(sid, t)),
                _ => (s.target_mode != "ALLY").then(|| PlanStep::self_skill(sid)),
            },
            RoundIntent::Assault | RoundIntent::Maneuver => {
                if skill::is_area(s) {
                    return Some(self.area_step(sid, self.target_pos(st)));
                }
                let foe = self.target_unit().filter(|&t| t != self.c)?;
                let step = PlanStep::engage(sid, foe);
                if self.intent == RoundIntent::Maneuver && carries(self.w, s) && self.kind != ManeuverKind::Disengage {
                    // MV7, INPUT1 MR6: a follow-up moves only when its tile is not a position tile
                    step.cell.set(st.pos);
                }
                Some(step)
            }
            RoundIntent::Channel => None,
            // IR4, IR10: every Field block on the round's one source tile
            RoundIntent::Field => match self.target {
                RoundTarget::Cell(p) if skill::is_area(s) => Some(PlanStep::area_at(sid, p)),
                _ => None,
            },
        }
    }

    /// A Stand (`guard`) round's step for skill `sid` from tile `at`: a self cast, or A4.2-K1b (INTENT2 IR5, IR6, IR22)
    /// for an area attack (IR21: a RING) the area anchored on the user's tile, only while an enemy it hits stands in it
    /// (None: an empty ring is not cast).
    fn stand_step(&self, sid: SkillId, at: V2) -> Option<Step> {
        let w = self.w;
        let s = self.skill(sid);
        if skill::kind(s) != Kind::Attack || !skill::is_area(s) {
            return Some(PlanStep::self_skill(sid));
        }
        let affects = skill::affects_by(s, Some(RoundIntent::Guard));
        let cells = skill::area_cells(s, at, at);
        let hits = skill::affects_unit(affects, false)
            && w.foes_of(self.c).iter().any(|&f| w.units[f].is_active() && w.has(f) && !round_intent::hidden(w, f) && w.pos(f) != at && cells.contains(&w.pos(f)));
        hits.then(|| PlanStep::area_at(sid, at))
    }

    /// An area step of an Assault or Maneuver round at `goal`: A4.2-K1b (IR17), under `field_intent` an Assault round on
    /// an enemy is the shaped Assault row (`PlanStep::area_on`), else the area at the tile (classic, the legacy rows).
    fn area_step(&self, sid: SkillId, goal: V2) -> Step {
        match self.target {
            RoundTarget::Unit(t) if self.intent == RoundIntent::Assault && t != self.c && round_intent::field_rows(self.w) => PlanStep::area_on(sid, t, goal),
            _ => PlanStep::area_at(sid, goal),
        }
    }

    /// FU2, FU3: block by block, the first entry of `order` (FL2's `follow_order`: the opener's follow-up list, then
    /// the deck order) that has the intent, fits before the reserve, is affordable without debt (and leaves the
    /// reserve's AP), acts on the target from the unit's tile and is a skill. Returns the first refused entry's cause
    /// at the block where the fill stops ("" when the blocks ran out).
    fn fill(&self, plan: &mut Plan, reserve: &Reserve, order: &[SkillId]) -> String {
        let w = self.w;
        let u = &w.units[self.c];
        let pays = w.pays_ap(self.c);
        for _ in 0..=w.total_blocks {
            let st = plan.compile(w, self.c);
            let limit = self.limit(&st, reserve);
            let free = (st.next_block..=limit).filter(|b| !st.blocked_blocks.contains_key(b)).count() as i32;
            if free == 0 {
                return String::new();
            }
            let mut cause = String::new();
            let mut placed = false;
            for (k, &sid) in order.iter().enumerate() {
                let s = self.skill(sid);
                if k_is_jump(s) {
                    continue;
                }
                // A4.2-K1b (IR10 `field_fill_once`): in a Field round the fill places each family once
                if self.intent == RoundIntent::Field && st.entries.iter().any(|e| e.step_index >= 0 && self.skill(e.skill).family_id() == s.family_id()) {
                    continue;
                }
                let mut why = String::new();
                let need = s.blocks + w.recovery_of(self.c, s);
                let ap = if pays { skill::ap_cost_by(s, Some(u), &w.ruleset) } else { 0 };
                let ap_free = st.ap_left - reserve.ap;
                // HOOKS1 HC1: a skill whose HP cost the unit's HP left does not exceed is refused (rule hp_cost)
                let hp = command::hp_cost_problem(st.hp_left, command::hp_cost_for(w, self.c, s));
                // HOOKS1 UM2: a skill whose meter cost the planned meters do not cover is refused (rule unit_meters;
                // PK1-S2a: its meter gate first, then the costs with a marrow version's channel cost)
                let gate = crate::meter::gate_problem(w, s.requires_meter.as_ref(), |m| st.meter(m));
                let meter = if gate.is_empty() { crate::meter::cost_problem(w, &crate::channel::meters_for(w, self.c, s), |m| st.meter(m)) } else { gate };
                if !fits_before(&st, need, limit) {
                    why = format!("blocks {free} < {need}");
                } else if ap > ap_free {
                    why = format!("AP {} < {ap}", ap_free.max(0));
                } else if !hp.is_empty() {
                    why = hp;
                } else if !meter.is_empty() {
                    why = meter;
                } else if let Some(step) = self.follow_step(&st, sid) {
                    // Decision "Auto-fill keeps a flank": only the fill (and the same step accepted from the ghost)
                    step.stay.set(true);
                    let before = st.entries.len();
                    let p = plan.add_step(w, self.c, tagged(step, &format!("follow-up {}", k + 1), ""), false);
                    if p.is_empty() {
                        let st2 = plan.compile(w, self.c);
                        let moved = st2.entries[before..].iter().any(|e| skill::kind(self.skill(e.skill)) == Kind::Move);
                        let spill = st2.entries[before..].iter().any(|e| e.last_block(w) > limit);
                        if moved || spill || st2.ap_left < reserve.ap {
                            let n = plan.steps_of(self.c).len();
                            plan.remove_step(self.c, n - 1);
                            why = if spill { format!("blocks {free} < {need}") } else if moved { self.reach_cause() } else { format!("AP {} < {ap}", ap_free.max(0)) };
                        } else {
                            placed = true;
                        }
                    } else if p == round_intent::FLANK_PROTECTED {
                        // PL6: the Rest cause of a follow-up whose flank is protected (INPUT1 MR5)
                        why = "flank protected".into();
                    } else {
                        why = self.reach_cause();
                    }
                } else {
                    why = self.reach_cause();
                }
                if placed {
                    break;
                }
                if cause.is_empty() {
                    cause = why;
                }
            }
            if !placed {
                return cause;
            }
        }
        String::new()
    }

    fn reach_cause(&self) -> String {
        if self.intent == RoundIntent::Maneuver { "out of range" } else { "out of reach" }.into()
    }

    /// RS1: Rest on every free block left, tagged with the fill's stop cause (PL6); PK1-S2a: in a Channel round the
    /// Channel Rest (`channel_rest`, the game's Feed).
    fn rest(&self, plan: &mut Plan, cause: &str) {
        let w = self.w;
        let rest = crate::channel::rest_skill(w, self.c);
        let tag = if cause.is_empty() { REST.to_string() } else { format!("{REST}: {cause}") };
        for _ in 0..w.total_blocks {
            if plan.compile(w, self.c).blocks_left() <= 0 {
                break;
            }
            if !plan.add_step(w, self.c, tagged(PlanStep::self_skill(rest), &tag, ""), false).is_empty() {
                break;
            }
        }
    }
}

/// A DISENGAGE skill (a jump: MV8).
fn k_is_jump(s: &SkillDef) -> bool {
    skill::kind(s) == Kind::Disengage
}

/// The last block a step may use with `reserved` free blocks kept back at the round's end (lost blocks do not count).
fn limit_of(st: &SimState, reserved: i32) -> i32 {
    let mut left = reserved;
    let mut b = st.total_blocks;
    while left > 0 && b > 0 {
        if !st.blocked_blocks.contains_key(&b) {
            left -= 1;
        }
        b -= 1;
    }
    b
}

/// Whether `need` blocks fit in a row of free blocks from `st.next_block` ending at `limit` or before.
fn fits_before(st: &SimState, need: i32, limit: i32) -> bool {
    (st.next_block..=limit - need + 1).any(|s| (s..s + need).all(|b| !st.blocked_blocks.contains_key(&b)))
}

// ---- Picks, the ghost fill and the AP forecast (INPUT1 2-5, 7: IF, HK, GH, ET, AP1-AP2) ----------------------------

pub use crate::round_intent::Pick;

/// PL6 reason tags of the picks (INPUT1 13): a plain move picked first, the step after.
pub const MOVE_HERE: &str = "move here";
pub const STEP_AFTER: &str = "step after";
/// PL6 (VISCLAR1-D): the tag of the Rest blocks a step after left when the block it replaced was larger, before that
/// block's name (`move_note`).
pub const MOVE_REPLACED: &str = "rest: the move replaced ";
/// VISCLAR1-D: the refusal of a skill, an item or the fill after the round's Move.
pub const ONLY_REST: &str = "Only Rest after a Move";

/// INPUT1 ET5: the round's one regular move, by what planned it (a carried move, a chase or a jump never counts).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegularMove {
    /// AS1, SP1, MV10: into the skill's band.
    Approach,
    /// AS2: a ranged unit out of a melee band first.
    Kite,
    /// GD1: to the Guard tile.
    GuardTile,
    /// ET2: "Move here".
    MoveHere,
    /// MR7: a positioned skill's target beyond its range.
    OutOfRange,
    /// AS3: the reserved kite step after the actions.
    KiteAfter,
    /// ET4: "Step here after".
    StepAfter,
}

impl RegularMove {
    /// The words of ET4's refusal ("Already moved this round (approach)").
    pub fn label(self) -> &'static str {
        match self {
            RegularMove::Approach => "approach",
            RegularMove::Kite => "kite",
            RegularMove::GuardTile => "move to the Guard tile",
            RegularMove::MoveHere => "Move here",
            RegularMove::OutOfRange => "out of range",
            RegularMove::KiteAfter => "kite after",
            RegularMove::StepAfter => "step after",
        }
    }

    /// An after-move (at the round's end) rather than a before-move.
    pub fn after(self) -> bool {
        matches!(self, RegularMove::KiteAfter | RegularMove::StepAfter)
    }
}

/// INPUT1 IF2, ET1: one row of a target's skill menu: the picks it appends (two for "Move here, then X"), the intent and
/// Maneuver kind it sets (None: neutral), the AP and blocks it adds (its implied moves included), the reason it cannot be
/// picked now (None: it can) and the intent-bound buffs its intent change would drop (IN6, "Changing loses Riposte.").
/// `ap_short`: for a row refused for its AP (IF6 d), how many AP it needs beyond what it may spend (its cost less the
/// AP free for it: the AP left before it less the AP kept for a step after; INPUT1 AP3, AP8); None otherwise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickOption {
    pub picks: Vec<Pick>,
    pub intent: Option<RoundIntent>,
    pub kind: Option<ManeuverKind>,
    pub ap: i32,
    pub blocks: i32,
    pub problem: Option<String>,
    pub ap_short: Option<i32>,
    pub drops: Vec<String>,
    /// The target the picks go on (the menu's own, or for a `menu_options` cell the one its skill or item acts on).
    pub at: RoundTarget,
}

/// A pick's refusal: its words and, for an AP refusal (IF6 d), how many AP short it is (INPUT1 AP3, AP8).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Refusal {
    text: String,
    ap_short: Option<i32>,
}

impl Refusal {
    /// IF6 d: "AP <free> < <cost>" (the free AP shown from 0), `cost - free` short.
    fn ap(free: i32, cost: i32) -> Refusal {
        Refusal { text: format!("AP {} < {cost}", free.max(0)), ap_short: Some(cost - free) }
    }
}

impl From<String> for Refusal {
    fn from(text: String) -> Refusal {
        Refusal { text, ap_short: None }
    }
}

impl From<&str> for Refusal {
    fn from(text: &str) -> Refusal {
        Refusal { text: text.to_string(), ap_short: None }
    }
}

impl From<Refusal> for String {
    fn from(r: Refusal) -> String {
        r.text
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

/// INPUT1 AP1: a party member's AP for the round planned: `now` its AP, `spent` every planned command's AP (`picked` by
/// the player, `filled` by the ghost), `left = now - spent`, `next` the AP the round's end gives with no hit taken
/// (`ap_regen`, then the planned Rest blocks' gain, RS2, capped at `max`), `gain = next - left`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ApForecast {
    pub now: i32,
    pub spent: i32,
    pub picked: i32,
    pub filled: i32,
    pub left: i32,
    pub gain: i32,
    pub rest_gain: i32,
    pub next: i32,
    pub max: i32,
}

/// `c`'s picks this round (INPUT1 GH1).
pub fn picks_of(w: &World, c: UnitId) -> Vec<Pick> {
    w.round_states.get(&c).map(|r| r.picks.clone()).unwrap_or_default()
}

/// Sets `c`'s picks; with none left the round's opener and Maneuver kind go back to the planner's (GH2: the auto plan).
fn set_picks(w: &mut World, c: UnitId, picks: Vec<Pick>) {
    let rs = w.round_states.entry(c).or_default();
    if picks.is_empty() {
        rs.opener = None;
        rs.maneuver = None;
        rs.maneuver_auto = false;
    }
    rs.accepted.retain(|i| *i < picks.len());
    rs.picks = picks;
}

/// The name a pick goes by in messages ("Heavy Cleave", "Use Potion", "Move here").
pub fn pick_name(w: &World, p: &Pick) -> String {
    match p {
        Pick::Skill { skill, .. } => w.cat.skills[*skill].name.clone(),
        Pick::Item(id) => format!("Use {}", w.cat.items.get(id).map_or(id.as_str(), |i| i.name.as_str())),
        Pick::Rest => "Rest".into(),
        Pick::MoveHere(_) => "Move here".into(),
        Pick::StepAfter(_) => "Step here after".into(),
    }
}

/// INPUT1 HK1: `c`'s hotkey order: its actives in slot order (the deck rule's slots, else the deck's `active`, else
/// every family it can use that is not a basic, in catalog order), its meter skills, then the ruleset's basics and its
/// class basics; the row
/// of each family it uses now, only skills with an intent (not neutral), each once. The order never changes in a battle.
pub fn hotkey_skills(w: &World, c: UnitId) -> Vec<SkillId> {
    let mut out: Vec<SkillId> = Vec::new();
    for f in auto_families(w, c) {
        if let Some(sid) = family_skill(w, c, &f) {
            let s = &w.cat.skills[sid];
            if !round_intent::skill_rows(w, s).contains(&rpgkit_rules::ruleset::NEUTRAL) && !s.system && !out.contains(&sid) {
                out.push(sid);
            }
        }
    }
    out
}

/// INPUT1 HK3: a skill a hotkey aims at its user while no pick holds the round target (`target_mode` SELF, or a Defend
/// kind).
pub fn self_only(s: &SkillDef) -> bool {
    s.target_mode == "SELF" || skill::kind(s) == Kind::Defend
}

/// INPUT1 HK2, HK3: the target a hotkey pick of `skill` aims at while the skill menu is closed: the user itself for a
/// self-only skill while no pick holds the round target (TG1); else the round target, else the one the planner
/// proposes for `c`'s intent (TG6). None: no target yet ("Pick a target first").
pub fn hotkey_target(w: &World, c: UnitId, skill: SkillId) -> Option<RoundTarget> {
    let held = picks_of(w, c).iter().any(|p| p.holds_target());
    if self_only(&w.cat.skills[skill]) && !held {
        return Some(RoundTarget::Myself);
    }
    w.round_states.get(&c).and_then(|r| r.target).or_else(|| round_intent::intent_of(w, c).and_then(|i| proposed_target(w, c, i)))
}

/// A4.2-K1b (INTENT2 IR15, IR21): the intent rows of a skill or item pick, the primary first (a skill's
/// `round_intent::skill_rows`; an item's one intent).
fn pick_rows(w: &World, c: UnitId, p: &Pick) -> Vec<RoundIntent> {
    match p {
        Pick::Skill { skill, .. } => round_intent::skill_rows(w, &w.cat.skills[*skill]).into_iter().filter_map(RoundIntent::parse).collect(),
        Pick::Item(id) => command::pouch_item(w, c, id).or_else(|| w.cat.items.get(id)).map(|i| round_intent::item_rows(w, i).into_iter().filter_map(RoundIntent::parse).collect()).unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// A4.2-K1b (IR15, IR19): the row a pick aimed at `at` takes: the first of its rows the target fits (a tile the Field
/// row, an enemy the Assault row), else its primary. IR3 (b), IR15: a self target is always Stand when Stand is one of
/// its rows (a self-or-ally heal on yourself plans in Stand).
fn pick_row_at(w: &World, c: UnitId, p: &Pick, at: RoundTarget) -> Option<RoundIntent> {
    let rows = pick_rows(w, c, p);
    let at = round_intent::normalize(c, at);
    // A1.2-K3: items too (a heal on yourself is a Stand round under `field_intent`)
    if at == RoundTarget::Myself && rows.contains(&RoundIntent::Guard) && round_intent::listed(w, RoundIntent::Guard) {
        return Some(RoundIntent::Guard);
    }
    rows.iter().copied().find(|i| round_intent::listed(w, *i) && round_intent::target_problem(w, c, *i, at).is_empty()).or_else(|| rows.first().copied())
}

/// INPUT1 IF4: the intent `picks` aimed at `at` would set for `c` (its first skill or item pick's row: A4.2-K1b,
/// `pick_row_at`) when no pick holds the round yet and that intent is not the current one; None otherwise. The battle
/// sets it first (`Session::pick`, IN6's lines).
pub fn pick_intent(w: &World, c: UnitId, at: RoundTarget, picks: &[Pick]) -> Option<RoundIntent> {
    if picks_of(w, c).iter().any(Pick::holds_target) {
        return None;
    }
    let p = picks.iter().find(|p| p.holds_target())?;
    let i = pick_row_at(w, c, p, pick_target(w, c, at, p))?;
    (round_intent::intent_of(w, c) != Some(i)).then_some(i)
}

/// The round target a pick acts on when it is aimed at `at`: a Support skill or item picked on a tile (after "Move
/// here") acts on the unit itself. A4.2-K1b: a pick whose row on that tile is Field (IR5) keeps the tile.
fn pick_target(w: &World, c: UnitId, at: RoundTarget, p: &Pick) -> RoundTarget {
    let at = round_intent::normalize(c, at);
    match (pick_row_at(w, c, p, at), at) {
        (Some(RoundIntent::Support), RoundTarget::Cell(_)) => RoundTarget::Myself,
        _ => at,
    }
}

/// Whether `at` is the round target `round` (TG1): the same, a self-only skill under a tile target, or a unit's tile.
fn same_target(w: &World, round: RoundTarget, at: RoundTarget, self_cast: bool) -> bool {
    match (round, at) {
        (a, b) if a == b => true,
        (RoundTarget::Unit(u), RoundTarget::Cell(p)) => w.has(u) && w.pos(u) == p,
        (RoundTarget::Cell(_), RoundTarget::Myself) => self_cast,
        _ => false,
    }
}

fn target_label(w: &World, t: RoundTarget) -> String {
    match t {
        RoundTarget::Myself => "yourself".into(),
        RoundTarget::Unit(u) => w.units[u].name.clone(),
        RoundTarget::Cell(p) => p.to_string(),
    }
}

/// INPUT1 IF4, IF6 a-b, GH3: appends `picks` to `c`'s picks on target `at` (the first skill or item pick sets the
/// intent, the Maneuver kind and the round target) and rebuilds `c`'s plan from its picks, then every later member's
/// (PL8; the later members' dropped picks and refusals come back). Atomic: a refused pick leaves the battle as it was.
pub fn pick(w: &mut World, plan: &mut Plan, c: UnitId, at: RoundTarget, picks: &[Pick]) -> (Result<(), String>, Vec<(UnitId, String)>) {
    let (w0, seq0) = (w.clone(), plan.sequences.clone());
    match pick_inner(w, plan, c, at, picks) {
        Ok(()) => {
            let later = rebuild_later(w, plan, c);
            (Ok(()), later)
        }
        Err(e) => {
            *w = w0;
            plan.sequences = seq0;
            (Err(e.text), Vec::new())
        }
    }
}

/// `pick`'s steps: the picks set (`set_up_picks`), then `c`'s plan built from them.
fn pick_inner(w: &mut World, plan: &mut Plan, c: UnitId, at: RoundTarget, picks: &[Pick]) -> Result<(), Refusal> {
    set_up_picks(w, c, at, picks)?;
    build_picks(w, plan, c).map_err(|(_, e)| e)
}

/// IF6 a-b: checks `picks` against `c`'s round (intent, target, kind) and appends them, setting the intent, kind and
/// target the first skill or item pick brings.
fn set_up_picks(w: &mut World, c: UnitId, at: RoundTarget, picks: &[Pick]) -> Result<(), String> {
    if !round_intent::on(w) {
        return Err("This battle has no round intents.".into());
    }
    if picks.is_empty() {
        return Err("Pick something first.".into());
    }
    if !w.units[c].is_active() {
        return Err(format!("{} is not on the field.", w.units[c].name));
    }
    let mut all = picks_of(w, c);
    for p in picks {
        if let Pick::Skill { skill, .. } = p {
            if *skill >= w.cat.skills.len() {
                return Err("Unknown skill.".into());
            }
        }
        if let Pick::Item(id) = p {
            if command::pouch_item(w, c, id).is_none() {
                return Err(format!("{} carries no {id}.", w.units[c].name));
            }
        }
        let t = pick_target(w, c, at, p);
        let holder = all.iter().find(|q| q.holds_target()).cloned();
        if p.holds_target() {
            // A4.2-K1b (IR15): a pick in a held round takes the round's row when it has it, a first pick the row its
            // target fits (`pick_row_at`)
            let cur = round_intent::intent_of(w, c);
            let rows = pick_rows(w, c, p);
            let row = match &holder {
                Some(_) => cur.filter(|i| rows.contains(i)).or_else(|| rows.first().copied()),
                None => pick_row_at(w, c, p, t),
            };
            let Some(i) = row else { return Err(format!("{} is not a round skill.", pick_name(w, p))) };
            match holder {
                Some(h) => {
                    // IF6 a, b: the pick-set intent and the round target held by a pick
                    let cur = round_intent::intent_of(w, c);
                    if cur != Some(i) {
                        return Err(format!("Only {} skills this round ({} set it)", cur.map_or_else(|| "other".to_string(), |c| c.label(&w.cat)), pick_name(w, &h)));
                    }
                    // IF2: one Maneuver kind a round, as `pick_options` lists them
                    let (_, kind) = row_meta(w, c, t, std::slice::from_ref(p));
                    let cur_kind = round_intent::maneuver_of(w, c);
                    if kind.is_some_and(|k| k != cur_kind) {
                        return Err(format!("Only {} this round ({} set it)", crate::intent::maneuver_id(cur_kind), pick_name(w, &h)));
                    }
                    let round = w.round_states.get(&c).and_then(|r| r.target);
                    let self_cast = i == RoundIntent::Guard || matches!(p, Pick::Skill { skill, .. } if w.cat.skills[*skill].target_mode == "SELF");
                    if let Some(r) = round.filter(|r| !same_target(w, *r, t, self_cast)) {
                        return Err(format!("One target per round: {}", target_label(w, r)));
                    }
                }
                None => {
                    // IF4: the first skill or item pick sets the intent, the round target and the Maneuver kind
                    if round_intent::intent_of(w, c) != Some(i) {
                        round_intent::set_intent(w, c, i)?;
                    }
                    round_intent::set_target(w, c, t)?;
                    if i == RoundIntent::Maneuver {
                        let kind = match p {
                            Pick::Skill { kind: Some(k), .. } => *k,
                            Pick::Skill { skill, .. } if skill::kind(&w.cat.skills[*skill]) == Kind::Disengage => ManeuverKind::Disengage,
                            Pick::Item(_) => ManeuverKind::Utility,
                            _ if matches!(t, RoundTarget::Cell(_)) => ManeuverKind::Utility,
                            _ => ManeuverKind::Engage,
                        };
                        round_intent::set_maneuver(w, c, kind)?;
                    }
                    w.round_states.entry(c).or_default().opener = match p {
                        Pick::Skill { skill, .. } => Some(Opener::Skill(*skill)),
                        Pick::Item(id) => Some(Opener::Item(id.clone())),
                        _ => None,
                    };
                }
            }
        } else if holder.is_none() && round_intent::intent_of(w, c).is_some() && !matches!(p, Pick::StepAfter(_)) {
            // IF1: a neutral pick on a target no pick holds yet points the round there when it fits the intent
            let _ = round_intent::set_target(w, c, t);
        }
        all.push(p.clone());
    }
    set_picks(w, c, all);
    Ok(())
}

/// INPUT1 IF7: removes `c`'s last pick and rebuilds (as `pick`); with no pick left the plan is `plan_unit`'s (the game
/// then applies GH2). The round target and the intent stay.
pub fn unpick(w: &mut World, plan: &mut Plan, c: UnitId) -> (Result<(), String>, Vec<(UnitId, String)>) {
    let mut picks = picks_of(w, c);
    if picks.pop().is_none() {
        return (Err("Nothing to undo.".into()), Vec::new());
    }
    set_picks(w, c, picks);
    let first = rebuild_member(w, plan, c);
    (first, rebuild_later(w, plan, c))
}

/// INPUT2: the picks that accept `c`'s ghost (GH1) up to its compiled entry `upto` (an index of `Plan::entries_for`,
/// which must be a ghost entry; None: the whole ghost), and the round target they go on (the round's, else the first
/// target the ghost steps aim at, else self). Each ghost step becomes the pick that plans it: a skill on a unit, a tile
/// (an area) or self, an item, a Rest block, a step after (an after-move); a move before a skill is the skill's implied
/// move (no pick), a move with no skill after it in the accepted steps is "Move here" when it comes first.
pub fn ghost_picks(w: &World, plan: &Plan, c: UnitId, upto: Option<usize>) -> Result<(RoundTarget, Vec<Pick>), String> {
    if picks_of(w, c).iter().any(|p| matches!(p, Pick::StepAfter(_))) {
        return Err(ONLY_REST.into());
    }
    let steps = plan.steps_of(c);
    let ghosts = ghost_entries(w, plan, c);
    let entries = plan.entries_for(w, c);
    let last = match upto {
        Some(k) if ghosts.get(k).copied().unwrap_or(false) => usize::try_from(entries[k].step_index).map_err(|_| "No auto-filled block there.".to_string())?,
        Some(_) => return Err("No auto-filled block there.".into()),
        None => steps.len().checked_sub(1).ok_or_else(|| "Nothing auto-filled to accept.".to_string())?,
    };
    let none = picks_of(w, c).is_empty();
    let chosen: Vec<&Step> = steps.iter().take(last + 1).filter(|s| none || s.ghost.get()).collect();
    if chosen.is_empty() {
        return Err("Nothing auto-filled to accept.".into());
    }
    let maneuver = w.round_states.get(&c).and_then(|r| r.maneuver);
    let mut aim: Option<RoundTarget> = None;
    let mut picks: Vec<Pick> = Vec::new();
    for (i, st) in chosen.iter().enumerate() {
        let skill = st.skill.filter(|s| *s < w.cat.skills.len());
        let kind = skill.map(|s| skill::kind(&w.cat.skills[s]));
        let pick = match st.mode {
            Mode::Engage | Mode::Ally => skill.map(|s| {
                aim = aim.or(st.target.map(RoundTarget::Unit));
                Pick::Skill { skill: s, kind: maneuver }
            }),
            Mode::Area => skill.map(|s| {
                aim = aim.or(Some(RoundTarget::Cell(st.cell.get())));
                Pick::Skill { skill: s, kind: maneuver }
            }),
            // PK1-S2a: the Channel Rest (Feed) is a Rest pick too
            Mode::SelfCast if kind == Some(Kind::Rest) || skill.is_some_and(|s| crate::channel::is_channel_rest(w, c, s)) => Some(Pick::Rest),
            Mode::SelfCast | Mode::Withdraw if skill.is_some() && kind != Some(Kind::Move) => skill.map(|s| Pick::Skill { skill: s, kind: maneuver }),
            Mode::Item => {
                aim = aim.or(Some(st.target.map_or(RoundTarget::Myself, RoundTarget::Unit)));
                Some(Pick::Item(st.item_id.clone()))
            }
            _ if st.after => Some(Pick::StepAfter(st.cell.get())),
            // a move: the next accepted skill's implied move, else "Move here" as the round's first pick
            _ => {
                let skill_after = chosen[i + 1..].iter().any(|n| n.skill.is_some_and(|s| s < w.cat.skills.len() && skill::kind(&w.cat.skills[s]) != Kind::Move) && !n.after);
                (!skill_after && picks.is_empty() && picks_of(w, c).is_empty()).then(|| Pick::MoveHere(st.cell.get()))
            }
        };
        picks.extend(pick);
    }
    if picks.is_empty() {
        return Err("Nothing auto-filled to accept.".into());
    }
    let round = w.round_states.get(&c).and_then(|r| r.target);
    Ok((round.or(aim).unwrap_or(RoundTarget::Myself), picks))
}

/// INPUT2: accepts `c`'s ghost up to entry `upto` (None: all of it) as picks (`ghost_picks`, then `pick`: atomic, the
/// later members rebuilt); the picks added on success.
pub fn accept_ghost(w: &mut World, plan: &mut Plan, c: UnitId, upto: Option<usize>) -> (Result<usize, String>, Vec<(UnitId, String)>) {
    let (at, picks) = match ghost_picks(w, plan, c, upto) {
        Ok(x) => x,
        Err(e) => return (Err(e), Vec::new()),
    };
    // decision "Auto-fill keeps a flank": what was filled plans as it was filled, so accepting moves no one
    let from = picks_of(w, c).len();
    let accepted = w.round_states.get(&c).map(|r| r.accepted.clone()).unwrap_or_default();
    w.round_states.entry(c).or_default().accepted.extend(from..from + picks.len());
    let (r, later) = pick(w, plan, c, at, &picks);
    if r.is_err() {
        w.round_states.entry(c).or_default().accepted = accepted;
    }
    (r.map(|()| picks.len()), later)
}

/// INPUT1 IF7 Clear: removes every pick of `c` and rebuilds (as `unpick`).
pub fn clear_picks(w: &mut World, plan: &mut Plan, c: UnitId) -> (Result<(), String>, Vec<(UnitId, String)>) {
    set_picks(w, c, Vec::new());
    let first = rebuild_member(w, plan, c);
    (first, rebuild_later(w, plan, c))
}

/// PK1-S2a (PK1 4.3 item 1, user 2026-11-10): the converting Channel toggle: flips `c`'s round to Channel (`on`) or
/// back to Cold Steel, any time while planning (it never locks). Only a unit that can Channel (`channel::can_channel`)
/// has it. Entering Channel counts as an intent change (IN6: the intent-bound buffs end; the lines name them). The plan
/// converts: every Move and Rest pick stays where it is, each skill pick that has a version in the new mode stays (the
/// same skill, target and anchor, now resolving as that version), a skill or item without one is dropped, and the
/// member is rebuilt from the picks left (a pick that then no longer fits is dropped with every pick after it, as
/// `rebuild_later` does), so the fill refills the freed blocks for the new mode (marrow versions and the Channel Rest
/// in Channel; the Cold Steel fill and Rest back in Cold Steel); a member without picks is planned again from its
/// intent. Then every later member is rebuilt (PL8). Returns the lines (buffs ended, picks dropped) and the later
/// members' refusals; Err when the unit cannot Channel. Flipping to the mode it is in changes nothing.
pub fn set_channel(w: &mut World, plan: &mut Plan, c: UnitId, on: bool) -> (Result<Vec<String>, String>, Vec<(UnitId, String)>) {
    if !round_intent::on(w) {
        return (Err("This battle has no round intents.".into()), Vec::new());
    }
    if !crate::channel::can_channel(w, c) {
        return (Err(format!("{} cannot Channel.", w.units.get(c).map_or("That unit", |u| u.name.as_str()))), Vec::new());
    }
    if crate::channel::is_channel(w, c) == on {
        return (Ok(Vec::new()), Vec::new());
    }
    let mut lines = Vec::new();
    if on {
        // FB-167 Q1 (a): entering a Channel round counts as an intent change (IN6)
        for name in round_intent::drop_bound(w, c) {
            lines.push(format!("{} enters Channel: {name} ends", w.units[c].name));
        }
    }
    w.round_states.entry(c).or_default().channel = on;
    let picks = picks_of(w, c);
    let mut kept: Vec<Pick> = Vec::new();
    for p in picks {
        let gone = match &p {
            Pick::Skill { skill, .. } => crate::channel::problem(w, c, &w.cat.skills[*skill]),
            Pick::Item(_) => crate::channel::item_problem(w, c),
            _ => None,
        };
        match gone {
            Some(why) => lines.push(format!("{}: {} dropped: {why}", w.units[c].name, pick_name(w, &p))),
            None => kept.push(p),
        }
    }
    // the first skill or item pick left opens the round (its intent is the round's: IF6 a)
    let opener = kept.iter().find(|p| p.holds_target()).map(|p| match p {
        Pick::Skill { skill, .. } => Opener::Skill(*skill),
        Pick::Item(id) => Opener::Item(id.clone()),
        _ => unreachable!("holds_target is a skill or an item"),
    });
    let any = !kept.is_empty();
    set_picks(w, c, kept);
    if any {
        w.round_states.entry(c).or_default().opener = opener;
    }
    if picks_of(w, c).is_empty() {
        let _ = rebuild_member(w, plan, c);
    }
    while !picks_of(w, c).is_empty() {
        let Err((i, why)) = build_picks(w, plan, c) else { break };
        let picks = picks_of(w, c);
        let i = i.min(picks.len() - 1);
        lines.push(format!("{}: {} dropped: {why}", w.units[c].name, pick_name(w, &picks[i])));
        set_picks(w, c, picks[..i].to_vec());
        if picks_of(w, c).is_empty() {
            let _ = rebuild_member(w, plan, c);
        }
    }
    (Ok(lines), rebuild_later(w, plan, c))
}

/// INPUT1 GH7: the Auto-fill setting; rebuilds every active party member's ghost (members with picks from their picks,
/// members without from their intent, `plan_unit`, or Rest with Auto-fill off).
pub fn set_autofill(w: &mut World, plan: &mut Plan, on: bool) {
    w.autofill = on;
    if !round_intent::on(w) {
        return;
    }
    for c in w.allies.clone() {
        if !w.units[c].is_active() || (picks_of(w, c).is_empty() && round_intent::intent_of(w, c).is_none()) {
            continue;
        }
        let _ = rebuild_member(w, plan, c);
    }
}

/// `c`'s plan from its picks (GH3), or with none from its intent: `plan_unit`'s plan, Rest on every block with
/// Auto-fill off.
fn rebuild_member(w: &mut World, plan: &mut Plan, c: UnitId) -> Result<(), String> {
    if !picks_of(w, c).is_empty() {
        return build_picks(w, plan, c).map_err(|(_, e)| e.text);
    }
    if !w.autofill {
        plan.set_steps(c, Vec::new());
        rest_to(w, plan, c, REST, w.total_blocks);
        return Ok(());
    }
    if round_intent::intent_of(w, c).is_none() {
        // no intent and no pick: nothing to plan (the game drafts the tactic's plan, GH2 a)
        plan.set_steps(c, Vec::new());
        return Ok(());
    }
    plan_unit(w, plan, c)
}

/// RS1 within `limit`: Rest on every free block up to `limit`, tagged `tag` (PK1-S2a: the Channel Rest in a Channel
/// round).
fn rest_to(w: &World, plan: &mut Plan, c: UnitId, tag: &str, limit: i32) {
    let rest = crate::channel::rest_skill(w, c);
    for _ in 0..w.total_blocks {
        let st = plan.compile(w, c);
        if st.blocks_left() <= 0 {
            break;
        }
        let n = st.entries.len();
        if !plan.add_step(w, c, tagged(PlanStep::self_skill(rest), tag, ""), false).is_empty() {
            break;
        }
        if plan.compile(w, c).entries.get(n).is_some_and(|e| e.start_block > limit) {
            let k = plan.steps_of(c).len();
            plan.remove_step(c, k - 1);
            break;
        }
    }
}

/// GH1: marks `c`'s steps from index `from` on as ghost (the planner's), all but `keep` (the step after and the Rest picked
/// after it).
fn mark_ghost(plan: &Plan, c: UnitId, from: usize, keep: &[Step]) {
    for (i, s) in plan.steps_of(c).iter().enumerate() {
        s.ghost.set(i >= from && !keep.iter().any(|k| Rc::ptr_eq(k, s)));
    }
}

/// The free blocks of `st` from its next block up to `limit`.
fn free_blocks(st: &SimState, limit: i32) -> i32 {
    (st.next_block..=limit).filter(|b| !st.blocked_blocks.contains_key(b)).count() as i32
}

/// The step after, held back until the fill is built (ET4, GH4, VISCLAR1-D): its pick index, step and tile.
struct After {
    index: usize,
    step: Step,
    cell: V2,
}

/// What `place` checks a pick's steps against: the last block it may use, the AP it must leave, and the blocks
/// refusal's words for a need (IF6 c).
struct Room<'a> {
    limit: i32,
    reserve_ap: i32,
    taken: &'a dyn Fn(i32) -> String,
}

/// GH3, GH4, ET2-ET5, IF6 c-e: `c`'s plan from its picks in order (each with its implied moves), then (Auto-fill on)
/// the planner's fill from the order and AS3's reserve, Rest on the blocks left, and the step after on the round's last
/// block; the steps after the picks are ghost (GH1). Err: the first pick that does not fit and why.
fn build_picks(w: &World, plan: &mut Plan, c: UnitId) -> Result<(), (usize, Refusal)> {
    let picks = picks_of(w, c);
    plan.set_steps(c, Vec::new());
    let b = Builder::new(w, c).ok();
    let pays = w.pays_ap(c);
    let mv = w.book().lowest(Kind::Move);
    let mut after: Option<After> = None;
    let mut rests_after: Vec<usize> = Vec::new();
    let opener_at = picks.iter().position(Pick::holds_target);
    for (i, p) in picks.iter().enumerate() {
        let st = plan.compile(w, c);
        let limit = limit_of(&st, 0);
        let free = free_blocks(&st, limit);
        let has_after = after.is_some();
        let taken = |need: i32| format!("Blocks {free} < {need}");
        let room = Room { limit, reserve_ap: 0, taken: &taken };
        let ap_free = st.ap_left;
        let moved = regular_move(w, plan, c);
        let fail = |e: String| (i, Refusal::from(e));
        let refuse = |r: Refusal| (i, r);
        match p {
            Pick::StepAfter(cell) => {
                // ET4: one move block at the round's end, after a skill or item pick, while the round has no regular move
                if has_after {
                    return Err(fail("One step after a round".into()));
                }
                if !picks[..i].iter().any(Pick::holds_target) {
                    return Err(fail("Step here after follows a skill or item pick".into()));
                }
                if let Some(k) = moved {
                    return Err(fail(format!("Already moved this round ({})", k.label())));
                }
                if st.rooted {
                    return Err(fail(format!("{} is rooted this round and can't move.", w.units[c].name)));
                }
                let route = move_route(w, plan, c, st.pos, *cell).map_err(fail)?;
                let need = path::blocks_needed(&w.grid, st.pos, &route, w.units[c].move_per_block()) as i32;
                let cap = w.ruleset.numbers.move_after_blocks;
                if need > cap {
                    return Err(fail(format!("Too far: {need} move blocks > {cap}")));
                }
                if free < need {
                    return Err(fail(format!("Blocks {free} < {need} (the picks fill the round)")));
                }
                let ap = if pays { route.len() as i32 } else { 0 };
                if ap > ap_free {
                    return Err(refuse(Refusal::ap(ap_free, ap)));
                }
                after = Some(After { index: i, step: tagged(PlanStep::after_move_up_to(mv, *cell, cap), STEP_AFTER, ""), cell: *cell });
            }
            Pick::MoveHere(cell) => {
                // ET2, ET3: a plain move, the round's first pick, while no move is planned
                if moved.is_some() {
                    return Err(fail("Already moving this round".into()));
                }
                if i > 0 {
                    return Err(fail("Move here comes first in a round".into()));
                }
                if st.rooted {
                    return Err(fail(format!("{} is rooted this round and can't move.", w.units[c].name)));
                }
                let route = move_route(w, plan, c, st.pos, *cell).map_err(fail)?;
                let most = plan.max_move_tiles(w, c);
                if route.len() as i32 > most || !plan.fits_move(w, c, st.pos, &route) {
                    return Err(fail(format!("Too far: {} tiles > {most} (two move blocks)", route.len())));
                }
                let ap = if pays { route.len() as i32 } else { 0 };
                if ap > ap_free {
                    return Err(refuse(Refusal::ap(ap_free, ap)));
                }
                place(w, plan, c, vec![tagged(PlanStep::go_to(mv, *cell), MOVE_HERE, "")], true, &room).map_err(refuse)?;
            }
            // VISCLAR1-D: Rest after the Move goes after it, once it is placed
            Pick::Rest if has_after => rests_after.push(i),
            Pick::Rest => {
                if free < 1 {
                    return Err(fail(taken(1)));
                }
                let rest = crate::channel::rest_skill(w, c);
                place(w, plan, c, vec![tagged(PlanStep::self_skill(rest), &format!("pick {}", i + 1), "")], false, &room).map_err(refuse)?;
            }
            Pick::Skill { .. } | Pick::Item(_) => {
                if has_after {
                    return Err(fail(ONLY_REST.into()));
                }
                let Some(b) = b.as_ref() else { return Err(fail("Pick a round intent first.".into())) };
                let opener = opener_at == Some(i);
                let tag = if opener { OPENER.to_string() } else { format!("pick {}", i + 1) };
                // the opener brings its before-move (PL5) unless the round already moves (ET5)
                let may_move = opener && moved.is_none();
                let accepted = !opener && w.round_states.get(&c).is_some_and(|r| r.accepted.contains(&i));
                let steps = b.pick_steps(plan, &st, p, may_move, &tag, ap_free, &room, accepted).map_err(refuse)?;
                if accepted {
                    steps.iter().for_each(|s| s.stay.set(true));
                }
                place(w, plan, c, steps, may_move, &room).map_err(|e| if e.text == REACH { fail(b.reach_text(plan, &st, p)) } else { refuse(e) })?;
            }
        }
    }
    let mark = plan.steps_of(c).len();
    let mut rest_tag = REST.to_string();
    match b.as_ref().filter(|_| fills(w, c)) {
        Some(b) => {
            // GH3: the fill (FU2, FU3) and AS3's reserve (none once the round has its move: ET5). A step after does not
            // refill the round: the fill stays as it was and the step takes the place of its last move or block
            let mut reserve = b.reserve(plan);
            let held_back = reserve.step.take_if(|_| after.is_some());
            // FL2: the first skill or item pick (the opener) chooses the follow-up list; an item reads the order
            let opener = opener_at.and_then(|i| match &picks[i] {
                Pick::Skill { skill, .. } => Some(Opener::Skill(*skill)),
                _ => None,
            });
            let cause = b.fill(plan, &reserve, &follow_order(w, c, b.intent, opener.as_ref()));
            if let Some(step) = reserve.step {
                let _ = plan.add_step(w, c, step, false);
            }
            // a reserve larger than the step (a Disengage's jump) counts as the larger block it replaces
            let held_back = held_back.filter(|_| reserve.blocks > 1).and_then(|s| s.skill).map(|s| w.cat.skills[s].name.clone());
            let replaced = after.as_ref().and_then(|a| replace_last_fill(w, plan, c, mark, a.cell)).or(held_back);
            rest_tag = match replaced {
                Some(name) => format!("{MOVE_REPLACED}{name}"),
                None if cause.is_empty() => REST.to_string(),
                None => format!("{REST}: {cause}"),
            };
        }
        None => {}
    }
    // VISCLAR1-D: the Move goes in the first slot left, where the block it replaced would have started; only Rest follows
    let mut kept: Vec<Step> = Vec::new();
    if let Some(a) = &after {
        let p = plan.add_step(w, c, a.step.clone(), false);
        if !p.is_empty() {
            plan.set_steps(c, Vec::new());
            return Err((a.index, p.into()));
        }
        kept.push(a.step.clone());
        let rest = crate::channel::rest_skill(w, c);
        for &i in &rests_after {
            let step = tagged(PlanStep::self_skill(rest), &format!("pick {}", i + 1), "");
            let p = plan.add_step(w, c, step.clone(), false);
            if !p.is_empty() {
                plan.set_steps(c, Vec::new());
                return Err((i, Refusal::from("Blocks 0 < 1 (the Move and its Rest fill the round)".to_string())));
            }
            kept.push(step);
        }
    }
    rest_to(w, plan, c, &rest_tag, limit_of(&plan.compile(w, c), 0));
    mark_ghost(plan, c, mark, &kept);
    Ok(())
}

/// VISCLAR1-D (the menu's Move): makes room for a Move to `cell` by taking the planner's last steps back (those from `mark`
/// on), last first, until from the tile the steps left end on the route fits the cap, the free blocks and the AP (the removed
/// steps' AP comes back). The names of the steps removed, in round order, when spare blocks are left beyond the Move's (they
/// go to Rest); None when not.
fn replace_last_fill(w: &World, plan: &mut Plan, c: UnitId, mark: usize, cell: V2) -> Option<String> {
    let mut removed: Vec<String> = Vec::new();
    let pays = w.pays_ap(c);
    loop {
        let st = plan.compile(w, c);
        let free = free_blocks(&st, st.total_blocks);
        let steps = plan.steps_of(c);
        let need = move_route(w, plan, c, st.pos, cell).ok().map(|r| (path::blocks_needed(&w.grid, st.pos, &r, w.units[c].move_per_block()) as i32, r.len() as i32));
        let fits = need.is_some_and(|(n, tiles)| n <= w.ruleset.numbers.move_after_blocks && n <= free && (!pays || tiles <= st.ap_left));
        if fits || steps.len() <= mark {
            removed.reverse();
            let spare = free - need.map_or(0, |n| n.0);
            return (!removed.is_empty() && spare > 0).then(|| removed.join(" and "));
        }
        let last = steps.len() - 1;
        removed.extend(steps[last].skill.map(|s| w.cat.skills[s].name.clone()));
        plan.remove_step(c, last);
    }
}

/// GH3, ENCOUNTER2-C SR3: whether the ghost after `c`'s picks is the planner's fill: a scripted entry's own `fill`
/// (`RoundState::fill`), else the Auto-fill setting.
fn fills(w: &World, c: UnitId) -> bool {
    w.round_states.get(&c).and_then(|r| r.fill).unwrap_or(w.autofill)
}

/// The refusal `place` gives for an implied move a pick may not make (the caller words it: IF6 e).
const REACH: &str = "\u{0}reach";

/// Places a pick's `steps` (atomic): refused when a step does not compile, when it moves and `may_move` is false
/// (REACH: ET5), when it spills past the room's last block (IF6 c) or leaves less than its AP (IF6 d, PL7).
fn place(w: &World, plan: &mut Plan, c: UnitId, steps: Vec<Step>, may_move: bool, room: &Room) -> Result<(), Refusal> {
    let before = plan.compile(w, c);
    let mark = plan.steps_of(c).len();
    let n = before.entries.len();
    for step in steps {
        let p = plan.add_step(w, c, step, true);
        if !p.is_empty() {
            plan.truncate(c, mark);
            return Err(p.into());
        }
    }
    let st = plan.compile(w, c);
    let added = &st.entries[n.min(st.entries.len())..];
    let refuse = if !may_move && added.iter().any(|e| skill::kind(&w.cat.skills[e.skill]) == Kind::Move) {
        Some(Refusal::from(REACH))
    } else if added.iter().any(|e| e.last_block(w) > room.limit) {
        Some(Refusal::from((room.taken)(added.iter().map(|e| e.last_block(w) - e.start_block + 1).sum())))
    } else if st.ap_left < room.reserve_ap {
        // the AP free before the pick (less the step after's) against what the pick spends
        Some(Refusal::ap(before.ap_left - room.reserve_ap, before.ap_left - st.ap_left))
    } else {
        None
    };
    match refuse {
        Some(e) => {
            plan.truncate(c, mark);
            Err(e)
        }
        None => Ok(()),
    }
}

/// ET3: the route of a plain move from `from` to the empty tile `cell`: "No path", "Claimed by <ally>'s move".
fn move_route(w: &World, plan: &Plan, c: UnitId, from: V2, cell: V2) -> Result<Vec<V2>, String> {
    if !w.grid.is_valid(cell) || !w.grid.is_walkable(cell) || w.grid.is_exit(cell) {
        return Err("That tile can't be moved to.".into());
    }
    if cell == from {
        return Err("Already there.".into());
    }
    let blocked = plan.blocked_cells(w, c);
    if w.occupant(cell).is_none() && blocked.contains(&cell) {
        if let Some(&m) = w.friends_of(c).iter().find(|&&m| m != c && plan.sequences.contains_key(&m) && plan.planned_position(w, m) == cell) {
            return Err(format!("Claimed by {}'s move", w.units[m].name));
        }
    }
    let route = plan.route(w, c, from, &[cell].into_iter().collect(), &blocked, false);
    if route.is_empty() {
        return Err("No path".into());
    }
    Ok(route)
}

impl Builder<'_> {
    /// The steps of a skill or item pick from the tile at `st` (IF6 a, c, d first: intent, blocks, AP): the opener's
    /// (with its before-move when `may_move`, PL5) or a follow-up's from where the plan leaves the unit.
    #[allow(clippy::too_many_arguments)]
    fn pick_steps(&self, plan: &Plan, st: &SimState, p: &Pick, may_move: bool, tag: &str, ap_free: i32, room: &Room, accepted: bool) -> Result<Vec<Step>, Refusal> {
        let w = self.w;
        let u = &w.units[self.c];
        let pays = w.pays_ap(self.c);
        match p {
            Pick::Skill { skill: sid, .. } => {
                let s = self.skill(*sid);
                // PK1-S2a (PK1 4.3 item 2): a Channel round offers only marrow versions, a Cold Steel round none
                if let Some(p) = crate::channel::problem(w, self.c, s) {
                    return Err(p.into());
                }
                // (the Channel Rest is neutral, as Rest)
                if !round_intent::skill_rows(w, s).contains(&self.intent.id()) && !crate::channel::is_channel_rest(w, self.c, *sid) {
                    return Err(format!("Only {} skills this round", self.intent.label(&w.cat)).into());
                }
                let need = s.blocks + w.recovery_of(self.c, s);
                if !fits_before(st, need, room.limit) {
                    return Err((room.taken)(need).into());
                }
                let ap = if pays { skill::ap_cost_by(s, Some(u), &w.ruleset) } else { 0 };
                if ap > ap_free {
                    return Err(Refusal::ap(ap_free, ap));
                }
                let hp = command::hp_cost_problem(st.hp_left, command::hp_cost_for(w, self.c, s));
                if !hp.is_empty() {
                    return Err(hp.into());
                }
                // PK1-S2a: the meter gate at the pick's start (Vent: Strain below 50), then the costs
                let gate = crate::meter::gate_problem(w, s.requires_meter.as_ref(), |m| st.meter(m));
                if !gate.is_empty() {
                    return Err(gate.into());
                }
                let meter = crate::meter::cost_problem(w, &crate::channel::meters_for(w, self.c, s), |m| st.meter(m));
                if !meter.is_empty() {
                    return Err(meter.into());
                }
                if may_move {
                    // the opener's own steps, tagged as PL5's (only the opener may move)
                    return Ok(self.skill_opener(plan, st, *sid)?);
                }
                // decision "Auto-fill keeps a flank": what the ghost showed on its flank tile, accepted, needs no position
                let keeps = accepted && carries(w, s) && self.kind != ManeuverKind::Disengage && self.target_unit().is_some_and(|t| round_intent::keeps_flank(w, t, st.pos, round_intent::diagonal(w, self.c, Some(s))));
                if let Some(e) = self.reach_problem(plan, st, p).filter(|_| !keeps) {
                    return Err(e.into());
                }
                let Some(step) = self.follow_step(st, *sid) else { return Err(format!("{} can't act on {}", s.name, self.target_name()).into()) };
                Ok(vec![tagged(step, tag, "")])
            }
            Pick::Item(id) => {
                let Some(item) = command::pouch_item(w, self.c, id) else { return Err(format!("{} carries no {id}.", u.name).into()) };
                if let Some(p) = crate::channel::item_problem(w, self.c) {
                    return Err(p.into());
                }
                if !round_intent::item_allows(w, item, self.intent) {
                    return Err(format!("Only {} skills this round", self.intent.label(&w.cat)).into());
                }
                let assault_area = command::item_area_on(w, item) && !matches!(self.target, RoundTarget::Cell(_));
                let need = command::item_blocks(w, item, assault_area, w.total_blocks);
                if !fits_before(st, need, room.limit) {
                    return Err((room.taken)(need).into());
                }
                let ap = command::item_ap(w, self.c);
                if ap > ap_free {
                    return Err(Refusal::ap(ap_free, ap));
                }
                if may_move {
                    return Ok(self.item_opener(plan, st, id)?);
                }
                if let Some(e) = self.reach_problem(plan, st, p) {
                    return Err(e.into());
                }
                if let RoundTarget::Cell(source) = self.target {
                    if command::item_area_on(w, item) {
                        return Ok(vec![tagged(PlanStep::use_item_at(id, source), tag, "")]);
                    }
                }
                let Some(t) = self.target_unit() else { return Err(format!("{} needs a unit to aim at.", item.name).into()) };
                Ok(vec![tagged(PlanStep::use_item(id, t), tag, "")])
            }
            _ => Ok(Vec::new()),
        }
    }

    /// IF6 e for a pick that may not move: `reach_text` when it cannot act on the round target from the tile at `st`
    /// (a self or Guard skill always can; a skill with a position when an open position tile is in its range).
    fn reach_problem(&self, plan: &Plan, st: &SimState, p: &Pick) -> Option<String> {
        let w = self.w;
        let goal = self.target_pos(st);
        let out = match p {
            Pick::Item(id) => {
                if let RoundTarget::Cell(_) = self.target {
                    command::pouch_item(w, self.c, id).is_some_and(|i| st.pos.dist(goal) > command::item_field_reach(i))
                } else {
                    let t = self.target_unit()?;
                    t != self.c && command::pouch_item(w, self.c, id).is_some_and(|i| st.pos.dist(goal) > i.battle_reach())
                }
            }
            Pick::Skill { skill: sid, .. } => {
                let s = self.skill(*sid);
                match self.intent {
                    RoundIntent::Guard | RoundIntent::Channel => false,
                    // A4.2-K1b (IR5, IR7): the source within the Field band, a shape with a direction on a neighbour
                    RoundIntent::Field => {
                        let a = skill::area(s);
                        if matches!(a, skill::Area::Arc | skill::Area::Cone) {
                            goal == st.pos || (goal.x - st.pos.x).abs() > 1 || (goal.y - st.pos.y).abs() > 1
                        } else {
                            let band = if skill::from_user(a) { (1, 1) } else { skill::field_reach(s, &w.units[self.c], &w.ruleset) };
                            let d = st.pos.dist(goal);
                            d < band.0 || d > band.1
                        }
                    }
                    RoundIntent::Support => match self.target {
                        RoundTarget::Unit(t) if t != self.c => {
                            let band = skill::support_reach(s, &w.units[self.c]);
                            let d = st.pos.dist(goal);
                            d < band.0.max(1) || d > band.1
                        }
                        _ => false,
                    },
                    RoundIntent::Assault | RoundIntent::Maneuver => match (round_intent::dash_range(w, self.c, s), self.target) {
                        (Some(n), RoundTarget::Unit(t)) if !skill::is_area(s) && self.kind != ManeuverKind::Disengage => {
                            let blocked = plan.blocked_cells(w, self.c);
                            round_intent::position_tiles(w, self.c, s, t, st.pos, n, st.rooted, &blocked).1.is_empty()
                        }
                        _ => {
                            let band = round_intent::hit_band(w, self.c, s);
                            let d = round_intent::melee_dist(w, self.c, Some(s), st.pos, goal);
                            d < band.0 || d > band.1
                        }
                    },
                }
            }
            _ => false,
        };
        out.then(|| self.reach_text(plan, st, p))
    }

    /// IF6 e: why a pick cannot act on the target from the tile at `st` without a move: "Out of reach: 3 > 2", "Too
    /// close: 1 < 2", "Out of range: 6 > 4" (a skill with a position: the nearest open position tile against its
    /// range), "Flank protected".
    fn reach_text(&self, plan: &Plan, st: &SimState, p: &Pick) -> String {
        let w = self.w;
        let u = &w.units[self.c];
        let goal = self.target_pos(st);
        let (lo, hi, d) = match p {
            Pick::Item(id) => {
                let field = matches!(self.target, RoundTarget::Cell(_));
                let reach = command::pouch_item(w, self.c, id).map_or(1, |i| if field { command::item_field_reach(i) } else { i.battle_reach() });
                (0, reach, st.pos.dist(goal))
            }
            Pick::Skill { skill: sid, .. } => {
                let s = self.skill(*sid);
                if let (Some(n), RoundTarget::Unit(t)) = (round_intent::dash_range(w, self.c, s), self.target) {
                    let blocked = plan.blocked_cells(w, self.c);
                    let (open, _) = round_intent::position_tiles(w, self.c, s, t, st.pos, n, st.rooted, &blocked);
                    if open.protected() {
                        return round_intent::FLANK_PROTECTED.into();
                    }
                    let near = open.preferred.iter().chain(&open.fallback).map(|cell| cell.dist(st.pos)).min().unwrap_or(0);
                    return format!("Out of range: {near} > {n}");
                }
                let band = if skill::kind(s) == Kind::Support { skill::support_reach(s, u) } else { round_intent::hit_band(w, self.c, s) };
                (band.0, band.1, round_intent::melee_dist(w, self.c, Some(s), st.pos, goal))
            }
            _ => (0, 0, 0),
        };
        if d < lo {
            format!("Too close: {d} < {lo}")
        } else {
            format!("Out of reach: {d} > {hi}")
        }
    }
}

/// INPUT1 GH1: whether each of `c`'s compiled entries (aligned with `Plan::entries_for`) is ghost: the planner's after
/// the picks; with no pick every entry of the planned round (its tactic's or intent's plan); lost blocks never.
pub fn ghost_entries(w: &World, plan: &Plan, c: UnitId) -> Vec<bool> {
    ghosts_of(w, plan, c, &plan.entries_for(w, c))
}

/// `ghost_entries` of `c`'s entries already compiled.
fn ghosts_of(w: &World, plan: &Plan, c: UnitId, entries: &[crate::plan::PlanEntry]) -> Vec<bool> {
    let steps = plan.steps_of(c);
    let none = picks_of(w, c).is_empty();
    entries
        .iter()
        .map(|e| usize::try_from(e.step_index).ok().and_then(|i| steps.get(i)).is_some_and(|s| none || s.ghost.get()))
        .collect()
}

/// INPUT1 1.2: the index (in `Plan::entries_for`) of `c`'s first ghost entry (the entries' count when none is).
pub fn ghost_from(w: &World, plan: &Plan, c: UnitId) -> usize {
    let g = ghost_entries(w, plan, c);
    g.iter().position(|x| *x).unwrap_or(g.len())
}

/// INPUT1 ET5: the round's one regular move in `c`'s plan, if one is planned (a carried move, a chase or a jump never
/// counts; the first move entry's step decides).
pub fn regular_move(w: &World, plan: &Plan, c: UnitId) -> Option<RegularMove> {
    let steps = plan.steps_of(c);
    let e = plan.entries_for(w, c).into_iter().find(|e| e.step_index >= 0 && skill::kind(&w.cat.skills[e.skill]) == Kind::Move)?;
    let step = steps.get(e.step_index as usize)?;
    let tag = if matches!(step.mode, Mode::Move | Mode::Goto | Mode::Withdraw) { step.reason.as_str() } else { step.move_reason.as_str() };
    Some(match tag {
        APPROACH => RegularMove::Approach,
        APPROACH_OUT_OF_RANGE => RegularMove::OutOfRange,
        KITE_IN_MELEE => RegularMove::Kite,
        COVER_TILE => RegularMove::GuardTile,
        KITE_THREATENED => RegularMove::KiteAfter,
        MOVE_HERE => RegularMove::MoveHere,
        STEP_AFTER => RegularMove::StepAfter,
        _ if step.after => RegularMove::StepAfter,
        _ if matches!(step.mode, Mode::Engage | Mode::Ally | Mode::Area | Mode::Item) => RegularMove::Approach,
        _ => RegularMove::MoveHere,
    })
}

/// INPUT1 AP1: `c`'s AP forecast for the round planned (`status::roll_over`'s `ap_regen` and the ap_regen bonus, then
/// `round_intent::round_start`'s Rest gain of the planned Rest blocks, each capped at `max_ap`).
pub fn ap_forecast(w: &World, plan: &Plan, c: UnitId) -> ApForecast {
    let u = &w.units[c];
    let entries = plan.entries_for(w, c);
    let ghost = ghosts_of(w, plan, c, &entries);
    let (mut picked, mut filled) = (0, 0);
    for (e, g) in entries.iter().zip(&ghost) {
        if *g {
            filled += e.total_ap();
        } else {
            picked += e.total_ap();
        }
    }
    let spent = picked + filled;
    let left = u.ap - spent;
    let rest_gain = if w.pays_ap(c) && w.ruleset.rest_at_round_end() { rest_preview(w, plan, c).ap } else { 0 };
    let regen = rpgkit_rules::status::regen_ap(u, left, w.ap_regen);
    let next = u.max_ap.min(regen + rest_gain);
    ApForecast { now: u.ap, spent, picked, filled, left, gain: next - left, rest_gain, next, max: u.max_ap }
}

/// A scratch copy of `plan` for trials on a cloned world (as `preview`).
fn plan_copy(plan: &Plan) -> Plan {
    let mut p2 = Plan::new();
    p2.sequences = plan.sequences.clone();
    p2.path_penalties = plan.path_penalties.clone();
    p2.goal_preferences = plan.goal_preferences.clone();
    p2.ai_pressure = plan.ai_pressure.clone();
    p2
}

/// INPUT1 IF5: `c`'s steps and AP forecast as if `picks` were picked on `at`, on copies; nothing changes.
pub fn preview_pick(w: &World, plan: &Plan, c: UnitId, at: RoundTarget, picks: &[Pick]) -> Result<(Vec<Step>, ApForecast), String> {
    let (w2, p2) = trial(w, plan, c, at, picks).map_err(String::from)?;
    Ok((p2.steps_of(c), ap_forecast(&w2, &p2, c)))
}

/// INPUT1 IF5: copies of the world and the plan as if `picks` were picked on `at` (the battle screen's hover preview
/// draws the strip from them); nothing changes.
pub fn preview_plan(w: &World, plan: &Plan, c: UnitId, at: RoundTarget, picks: &[Pick]) -> Result<(World, Plan), String> {
    trial(w, plan, c, at, picks).map_err(String::from)
}

/// A scratch copy of `plan` (as `preview` makes): its steps, path penalties, goal preferences and AI pressure.
pub fn copy_plan(plan: &Plan) -> Plan {
    plan_copy(plan)
}

fn trial(w: &World, plan: &Plan, c: UnitId, at: RoundTarget, picks: &[Pick]) -> Result<(World, Plan), Refusal> {
    let mut w2 = w.clone();
    let mut p2 = plan_copy(plan);
    pick_inner(&mut w2, &mut p2, c, at, picks)?;
    Ok((w2, p2))
}

/// The AP and blocks of `c`'s picked (not ghost) entries.
fn picked_cost(w: &World, plan: &Plan, c: UnitId) -> (i32, i32) {
    let g = ghost_entries(w, plan, c);
    plan.entries_for(w, c).iter().zip(&g).filter(|(e, g)| !**g && e.step_index >= 0).fold((0, 0), |(ap, b), (e, _)| (ap + e.total_ap(), b + e.last_block(w) - e.start_block + 1))
}

/// INPUT1 IF2, IF3, ET1 (read-only): the rows of `c`'s menu on target `at`, in hotkey order (HK1). Before the round's
/// first skill or item pick every skill that could act on that kind of target (a foe: Assault and Maneuver attacks, a
/// Maneuver attack once per kind it offers, Engage then Disengage; an ally: Support skills; the unit itself: Guard and
/// Support self skills; a tile: Move here, "Move here, then" each Guard and Support self skill, the area skills, Step
/// here after), the pouch items of that aim, then Rest, each with its problem (None: it can be picked now); after it
/// only the rows that can still be picked. A unit other than the round target held by a pick: no row.
pub fn pick_options(w: &World, plan: &Plan, c: UnitId, at: RoundTarget) -> Vec<PickOption> {
    if !round_intent::on(w) || !w.units[c].is_active() {
        return Vec::new();
    }
    let at = round_intent::normalize(c, at);
    let held = picks_of(w, c).iter().any(Pick::holds_target);
    let round = w.round_states.get(&c).and_then(|r| r.target);
    if held && !matches!(at, RoundTarget::Cell(_)) && round.is_some_and(|r| !same_target(w, r, at, at == RoundTarget::Myself)) {
        return Vec::new();
    }
    let rows = menu_rows_on(w, c, at, held);
    let cur = round_intent::intent_of(w, c);
    let cur_kind = round_intent::maneuver_of(w, c);
    let base = picked_cost(w, plan, c);
    let mut out = Vec::new();
    for (picks, row_at) in rows {
        let (intent, kind) = row_meta(w, c, row_at, &picks);
        // IF2: after the first pick only the round intent's (and kind's) rows, and no Move here (ET2)
        let other = intent.is_some() && intent != cur;
        let other_kind = intent == Some(RoundIntent::Maneuver) && kind.is_some_and(|k| k != cur_kind);
        if held && (other || other_kind || picks.iter().any(|p| matches!(p, Pick::MoveHere(_)))) {
            continue;
        }
        let o = option_of(w, plan, c, picks, row_at, base);
        if !(held && o.problem.is_some()) {
            out.push(o);
        }
    }
    out
}

/// The picks IF2 lists for `c` on `at` by the target's kind, each with the target it goes on (`pick_options`, `menu_options`).
fn menu_rows_on(w: &World, c: UnitId, at: RoundTarget, held: bool) -> Vec<(Vec<Pick>, RoundTarget)> {
    let skills = hotkey_skills(w, c);
    // A4.2-K1b (IR15): a skill is listed under each of its rows
    let intent_is = |s: &SkillDef, i: RoundIntent| round_intent::skill_rows(w, s).contains(&i.id());
    let mut rows: Vec<(Vec<Pick>, RoundTarget)> = Vec::new();
    let skill_row = |sid: SkillId, kind: Option<ManeuverKind>| (vec![Pick::Skill { skill: sid, kind }], at);
    let item_rows = |rows: &mut Vec<(Vec<Pick>, RoundTarget)>, keep: &dyn Fn(&rpgkit_data::ItemDef) -> bool| {
        for id in command::pouch_ids(w, c) {
            if command::pouch_item(w, c, &id).is_some_and(|i| i.offered_in_battle() && keep(i)) {
                rows.push((vec![Pick::Item(id)], at));
            }
        }
    };
    match at {
        RoundTarget::Unit(t) if w.foes_of(c).contains(&t) => {
            for &sid in &skills {
                let s = &w.cat.skills[sid];
                let k = skill::kind(s);
                if intent_is(s, RoundIntent::Assault) && k == Kind::Attack {
                    rows.push(skill_row(sid, None));
                } else if intent_is(s, RoundIntent::Maneuver) && k == Kind::Disengage {
                    rows.push(skill_row(sid, Some(ManeuverKind::Disengage)));
                } else if intent_is(s, RoundIntent::Maneuver) && k == Kind::Attack {
                    rows.push(skill_row(sid, Some(ManeuverKind::Engage)));
                    if carries(w, s) && !skill::is_area(s) {
                        rows.push(skill_row(sid, Some(ManeuverKind::Disengage)));
                    }
                }
            }
            item_rows(&mut rows, &|i| i.aims_at_foes());
            // A4.2-K1b (IR19): on an enemy the Field row aims at its tile (a click on a unit means the unit; the Field
            // row is how to aim at its tile)
            if round_intent::field_rows(w) && w.has(t) {
                for &sid in &skills {
                    let s = &w.cat.skills[sid];
                    if skill::is_area(s) && intent_is(s, RoundIntent::Field) {
                        rows.push((vec![Pick::Skill { skill: sid, kind: None }], RoundTarget::Cell(w.pos(t))));
                    }
                }
                // A1.2-K3 (INTENT2 section 6): an area item's Field row on the enemy's tile
                for id in command::pouch_ids(w, c) {
                    if command::pouch_item(w, c, &id).is_some_and(|i| i.offered_in_battle() && command::item_area_on(w, i) && round_intent::item_allows(w, i, RoundIntent::Field)) {
                        rows.push((vec![Pick::Item(id)], RoundTarget::Cell(w.pos(t))));
                    }
                }
            }
        }
        RoundTarget::Unit(t) => {
            for &sid in &skills {
                let s = &w.cat.skills[sid];
                if intent_is(s, RoundIntent::Support) && s.target_mode != "SELF" && skill::kind(s) != Kind::Focus {
                    rows.push(skill_row(sid, None));
                }
            }
            item_rows(&mut rows, &|i| !i.aims_at_foes() && command::item_side_problem(w, c, i, Some(t)).is_empty());
        }
        RoundTarget::Myself => {
            for &sid in &skills {
                let s = &w.cat.skills[sid];
                if intent_is(s, RoundIntent::Guard) || (intent_is(s, RoundIntent::Support) && s.target_mode != "ALLY") {
                    rows.push(skill_row(sid, None));
                }
            }
            item_rows(&mut rows, &|i| !i.aims_at_foes() && command::item_side_problem(w, c, i, Some(c)).is_empty());
        }
        RoundTarget::Cell(p) => {
            rows.push((vec![Pick::MoveHere(p)], at));
            for &sid in &skills {
                let s = &w.cat.skills[sid];
                if intent_is(s, RoundIntent::Guard) {
                    rows.push((vec![Pick::MoveHere(p), Pick::Skill { skill: sid, kind: None }], RoundTarget::Cell(p)));
                } else if intent_is(s, RoundIntent::Support) && s.target_mode != "ALLY" {
                    rows.push((vec![Pick::MoveHere(p), Pick::Skill { skill: sid, kind: None }], RoundTarget::Myself));
                }
            }
            for &sid in &skills {
                let s = &w.cat.skills[sid];
                if skill::is_area(s) && (intent_is(s, RoundIntent::Assault) || intent_is(s, RoundIntent::Field)) {
                    rows.push(skill_row(sid, None));
                } else if skill::is_area(s) && intent_is(s, RoundIntent::Maneuver) {
                    rows.push(skill_row(sid, Some(ManeuverKind::Utility)));
                }
            }
            // A1.2-K3 (INTENT2 section 6, IR2): an area item's Field row on a tile
            item_rows(&mut rows, &|i| command::item_area_on(w, i) && round_intent::item_allows(w, i, RoundIntent::Field));
            if held {
                rows.push((vec![Pick::StepAfter(p)], at));
            }
        }
    }
    rows.push((vec![Pick::Rest], at));
    rows
}

/// The intent and Maneuver kind a row's first skill or item pick sets aimed at `at` (A4.2-K1b: its row there).
fn row_meta(w: &World, c: UnitId, at: RoundTarget, picks: &[Pick]) -> (Option<RoundIntent>, Option<ManeuverKind>) {
    let first = picks.iter().find(|p| p.holds_target());
    let intent = first.and_then(|p| pick_row_at(w, c, p, pick_target(w, c, at, p)));
    let kind = match first {
        Some(Pick::Skill { kind, .. }) if intent == Some(RoundIntent::Maneuver) => *kind,
        Some(Pick::Item(_)) if intent == Some(RoundIntent::Maneuver) => Some(ManeuverKind::Utility),
        _ => None,
    };
    (intent, kind)
}

/// One row of a menu: `picks` on `at` tried on copies (`base`: `picked_cost` before them).
fn option_of(w: &World, plan: &Plan, c: UnitId, picks: Vec<Pick>, at: RoundTarget, base: (i32, i32)) -> PickOption {
    let (intent, kind) = row_meta(w, c, at, &picks);
    let held = picks_of(w, c).iter().any(Pick::holds_target);
    let (problem, ap_short, ap, blocks) = match trial(w, plan, c, at, &picks) {
        Ok((w2, p2)) => {
            let now = picked_cost(&w2, &p2, c);
            (None, None, now.0 - base.0, now.1 - base.1)
        }
        Err(e) => (Some(e.text), e.ap_short, 0, 0),
    };
    let drops = intent.filter(|_| !held).map(|i| round_intent::would_drop(w, c, i)).unwrap_or_default();
    PickOption { picks, intent, kind, ap, blocks, problem, ap_short, drops, at }
}

/// VISCLAR1-D (decisions "Battle view answers, part 2", "Intent framework answers"): the icon menu's cells for `c`, the same
/// list in the same order whatever the target and the picks so far, so nothing shifts when an opener fixes the intent: each
/// of `c`'s skills in hotkey order (a Maneuver attack that carries a move once per kind, Engage then Disengage), its pouch
/// items, Rest, and on a tile the move there (`Pick::MoveHere` first in a round, `Pick::StepAfter` after a skill). A cell is
/// lit only when `pick_options` has it on `at`; else it carries why (`target_hint` for the wrong kind of target). Empty as
/// `pick_options` is: a unit other than the round target a pick holds.
pub fn menu_options(w: &World, plan: &Plan, c: UnitId, at: RoundTarget) -> Vec<PickOption> {
    if !round_intent::on(w) || !w.units[c].is_active() {
        return Vec::new();
    }
    let at = round_intent::normalize(c, at);
    let held = picks_of(w, c).iter().any(Pick::holds_target);
    let round = w.round_states.get(&c).and_then(|r| r.target);
    if held && !matches!(at, RoundTarget::Cell(_)) && round.is_some_and(|r| !same_target(w, r, at, at == RoundTarget::Myself)) {
        return Vec::new();
    }
    let rows = menu_rows_on(w, c, at, held);
    let base = picked_cost(w, plan, c);
    // A4.2-K1b (IR19): `want` names the intent row of a dual skill's cell (None: the row its target gives)
    let cell_in = |p: Pick, same: &dyn Fn(&Pick) -> bool, want: Option<RoundIntent>| -> PickOption {
        let fits = |r: &Vec<Pick>, t: RoundTarget| want.is_none_or(|i| row_meta(w, c, t, r).0 == Some(i));
        if let Some((r, t)) = rows.iter().find(|(r, t)| r.last().is_some_and(same) && !(held && r.len() > 1) && fits(r, *t)) {
            return option_of(w, plan, c, r.clone(), *t, base);
        }
        // no row on this target: after an opener the pick's own refusal ("Only Assault skills"), else what it needs
        let why = if held && want.is_none_or(|i| round_intent::intent_of(w, c) == Some(i)) { option_of(w, plan, c, vec![p.clone()], at, base).problem } else { None };
        let (intent, kind) = row_meta(w, c, at, std::slice::from_ref(&p));
        let intent = want.or(intent);
        let problem = why.unwrap_or_else(|| match want {
            Some(RoundIntent::Field) => "Target a tile (or an enemy's tile) to use this".to_string(),
            Some(RoundIntent::Assault) => "Target an enemy to use this".to_string(),
            // INTENT2-U: an item's Stand cell is used on yourself, its Support cell on an ally
            Some(RoundIntent::Guard) => "Target yourself to use this".to_string(),
            Some(RoundIntent::Support) => "Target an ally to use this".to_string(),
            _ => target_hint(w, c, &p),
        });
        PickOption { picks: vec![p], intent, kind, ap: 0, blocks: 0, problem: Some(problem), ap_short: None, drops: Vec::new(), at }
    };
    let cell = |p: Pick, same: &dyn Fn(&Pick) -> bool| cell_in(p, same, None);
    let mut out = Vec::new();
    for sid in hotkey_skills(w, c) {
        let s = &w.cat.skills[sid];
        let kinds: &[Option<ManeuverKind>] = match (round_intent::skill_rows(w, s).contains(&RoundIntent::Maneuver.id()), skill::kind(s)) {
            (true, Kind::Disengage) => &[Some(ManeuverKind::Disengage)],
            (true, Kind::Attack) if carries(w, s) && !skill::is_area(s) => &[Some(ManeuverKind::Engage), Some(ManeuverKind::Disengage)],
            (true, Kind::Attack) => &[Some(ManeuverKind::Engage)],
            _ => &[None],
        };
        // A4.2-K1b (IR19): a dual Field / Assault skill has a cell in each row, the same list whatever the target
        let rows_of = round_intent::skill_rows(w, s);
        let dual = round_intent::field_rows(w) && rows_of.contains(&RoundIntent::Field.id()) && rows_of.contains(&RoundIntent::Assault.id());
        for &kind in kinds {
            // the kit's row of this cell: the same skill, and for Maneuver the same side of Disengage
            let same = |r: &Pick| matches!(r, Pick::Skill { skill, kind: k } if *skill == sid && (*k == Some(ManeuverKind::Disengage)) == (kind == Some(ManeuverKind::Disengage)));
            if dual {
                for want in [RoundIntent::Assault, RoundIntent::Field] {
                    out.push(cell_in(Pick::Skill { skill: sid, kind }, &same, Some(want)));
                }
            } else {
                out.push(cell(Pick::Skill { skill: sid, kind }, &same));
            }
        }
    }
    for id in command::pouch_ids(w, c) {
        let Some(item) = command::pouch_item(w, c, &id).filter(|i| i.offered_in_battle()) else { continue };
        let same = |r: &Pick| matches!(r, Pick::Item(x) if *x == id);
        // INTENT2-U (section 6 "Menu"): under `field_intent` an item has a cell in each listed row it allows, after the
        // skills (a heal in Support and Stand, a thrown bomb in Field and Assault)
        let rows: Vec<RoundIntent> = if round_intent::field_rows(w) {
            round_intent::item_rows(w, item).into_iter().filter_map(RoundIntent::parse).filter(|i| round_intent::listed(w, *i)).collect()
        } else {
            Vec::new()
        };
        if rows.len() > 1 {
            for want in rows {
                out.push(cell_in(Pick::Item(id.clone()), &same, Some(want)));
            }
        } else {
            out.push(cell(Pick::Item(id.clone()), &same));
        }
    }
    out.push(option_of(w, plan, c, vec![Pick::Rest], at, base));
    if let RoundTarget::Cell(p) = at {
        // the fade is wider than the pick's own rule: a dash has moved the unit too, though ET5 lets a step follow it
        let mut o = option_of(w, plan, c, vec![if held { Pick::StepAfter(p) } else { Pick::MoveHere(p) }], at, base);
        if let Some(why) = move_refusal(w, plan, c).filter(|_| o.problem.is_none()) {
            o = PickOption { problem: Some(why), ap: 0, blocks: 0, ..o };
        }
        out.push(o);
    }
    out
}

/// VISCLAR1-D (decision "Intent framework answers"): what a cell greyed for the kind of target needs.
fn target_hint(w: &World, c: UnitId, p: &Pick) -> String {
    let needs = match p {
        Pick::Skill { skill, .. } => {
            let s = &w.cat.skills[*skill];
            match round_intent::skill_primary(w, s) {
                i if i == RoundIntent::Field.id() => "a tile",
                i if i == RoundIntent::Guard.id() || self_only(s) => "yourself",
                i if i == RoundIntent::Support.id() && s.target_mode == "ALLY" => "an ally",
                i if i == RoundIntent::Support.id() => "yourself or an ally",
                _ if skill::is_area(s) => "an enemy or a tile",
                _ => "an enemy",
            }
        }
        Pick::Item(id) if command::pouch_item(w, c, id).is_some_and(|i| command::item_area_on(w, i)) => "an enemy or a tile",
        Pick::Item(id) if command::pouch_item(w, c, id).is_some_and(|i| i.aims_at_foes()) => "an enemy",
        Pick::Item(_) => "yourself or an ally",
        _ => "a tile",
    };
    format!("Target {needs} to use this")
}

/// VISCLAR1-D: one tile the menu's Move can go to now, what it takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveTile {
    pub cell: V2,
    pub blocks: i32,
    pub ap: i32,
    /// The filled skills this tile takes back, in round order ("": none): a far tile can cost more than the last block.
    pub drops: String,
}

/// The skills in `before` that `after` no longer has, in round order, joined for a note (Move and Rest do not count).
fn dropped_fill(w: &World, before: &Plan, after: &Plan, c: UnitId) -> String {
    let skills = |p: &Plan| -> Vec<SkillId> {
        p.steps_of(c).iter().filter_map(|s| s.skill).filter(|s| !matches!(skill::kind(&w.cat.skills[*s]), Kind::Move | Kind::Rest)).collect()
    };
    let mut left = skills(after);
    let mut gone = Vec::new();
    for s in skills(before) {
        match left.iter().position(|x| *x == s) {
            Some(i) => {
                left.remove(i);
            }
            None => gone.push(w.cat.skills[s].name.clone()),
        }
    }
    gone.join(" and ")
}

/// VISCLAR1-D: the tiles the menu's Move reaches now under its cap and the AP, each tried as the pick (empty when
/// `move_refusal` says no): what the board marks while the Move waits for its tile.
pub fn move_tiles(w: &World, plan: &Plan, c: UnitId) -> Vec<MoveTile> {
    if move_refusal(w, plan, c).is_some() || !w.has(c) {
        return Vec::new();
    }
    let held = picks_of(w, c).iter().any(Pick::holds_target);
    let from = w.pos(c);
    let reach = w.units[c].move_per_block() * if held { w.ruleset.numbers.move_after_blocks } else { 2 };
    let base = picked_cost(w, plan, c);
    let mut out = Vec::new();
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let cell = V2::new(x, y);
            if cell == from || cell.dist(from) > reach || !w.grid.is_walkable(cell) || w.grid.is_exit(cell) || w.occupant(cell).is_some() {
                continue;
            }
            let o = option_of(w, plan, c, vec![if held { Pick::StepAfter(cell) } else { Pick::MoveHere(cell) }], RoundTarget::Cell(cell), base);
            if o.problem.is_none() {
                let drops = trial(w, plan, c, o.at, &o.picks).map_or_else(|_| String::new(), |(_, after)| dropped_fill(w, plan, &after, c));
                out.push(MoveTile { cell, blocks: o.blocks, ap: o.ap, drops });
            }
        }
    }
    out
}

/// VISCLAR1-D: why the menu's Move cannot be taken now whatever the tile (None: it can): the unit has moved this round
/// (a picked move, or the opener's own walk or dash), a step after is picked, a pick other than a skill came first, the
/// picks fill the round, no AP is left after them, or it is rooted. The planner's own moves in the ghost do not count: a
/// Move replaces them.
pub fn move_refusal(w: &World, plan: &Plan, c: UnitId) -> Option<String> {
    let picks = picks_of(w, c);
    if picks.iter().any(|p| matches!(p, Pick::StepAfter(_))) {
        return Some("One step after a round".into());
    }
    let st = plan.compile(w, c);
    if !picks.is_empty() {
        if !picks.iter().any(Pick::holds_target) {
            return Some("Move here comes first in a round".into());
        }
        let steps = plan.steps_of(c);
        let entries = plan.entries_for(w, c);
        let picked = |e: &crate::plan::PlanEntry| usize::try_from(e.step_index).ok().and_then(|i| steps.get(i)).is_some_and(|s| !s.ghost.get());
        if entries.iter().any(|e| picked(e) && skill::kind(&w.cat.skills[e.skill]) == Kind::Move) {
            let label = regular_move(w, plan, c).map_or("a move", RegularMove::label);
            return Some(format!("Already moved this round ({label})"));
        }
        if entries.iter().any(|e| picked(e) && e.pos_before != e.pos_after) {
            return Some("Already moved this round (a dash)".into());
        }
        if entries.iter().all(picked) && free_blocks(&st, st.total_blocks) == 0 {
            return Some("Blocks 0 < 1 (the picks fill the round)".into());
        }
    }
    let ap_free = w.units[c].ap - picked_cost(w, plan, c).0;
    if w.pays_ap(c) && ap_free < 1 {
        return Some(format!("AP {} < 1", ap_free.max(0)));
    }
    st.rooted.then(|| format!("{} is rooted this round and can't move.", w.units[c].name))
}

/// VISCLAR1-D: the strip's note when `c`'s Move replaced a block larger than its own: "Move replaces <skill>: <n> blocks
/// left as Rest" (the Rest blocks tagged `MOVE_REPLACED`); None otherwise.
pub fn move_note(plan: &Plan, c: UnitId) -> Option<String> {
    let steps = plan.steps_of(c);
    let mut tags = steps.iter().filter_map(|s| s.reason.strip_prefix(MOVE_REPLACED));
    let name = tags.next()?;
    let n = tags.count() + 1;
    Some(format!("Move replaces {name}: {n} block{} left as Rest", if n == 1 { "" } else { "s" }))
}
