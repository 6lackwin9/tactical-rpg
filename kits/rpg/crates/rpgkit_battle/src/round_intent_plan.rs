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
/// then its meter skills (HOOKS1 UM6, rule unit_meters: in the order the battle gave them), then the basics (the
/// ruleset's `basics` in their order; a row without any: the catalog's basic skills).
fn auto_families(w: &World, c: UnitId) -> Vec<String> {
    let basics: Vec<String> = if w.ruleset.numbers.basics.is_empty() { w.cat.basic_ids().iter().map(|id| w.cat.family_of(id).to_string()).collect() } else { w.ruleset.numbers.basics.clone() };
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
    for id in &w.units[c].meter_skills {
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
/// intent); only skills of the intent the unit can use, each once.
pub fn order(w: &World, c: UnitId, intent: RoundIntent) -> Vec<SkillId> {
    let fams: Vec<String> = match deck_of(w, c).and_then(|d| d.order.get(intent.id())).filter(|l| !l.is_empty()) {
        Some(list) => list.clone(),
        None => auto_families(w, c),
    };
    let mut out: Vec<SkillId> = Vec::new();
    for f in &fams {
        if let Some(sid) = family_skill(w, c, f) {
            if round_intent::skill_intent(&w.cat.skills[sid]) == intent.id() && !out.contains(&sid) {
                out.push(sid);
            }
        }
    }
    out
}

/// INPUT1 FL2: the fill's order after `opener`: the skills of `c`'s `deck.follow_ups[<opener family>]` (in its order;
/// only families still in the unit's deck that it can use in this battle, of `intent`), then `order(w, c, intent)`;
/// a skill in both at its first place. An item opener, no opener or no list for it: `order` alone (so every unit
/// without lists plans as before).
pub fn follow_order(w: &World, c: UnitId, intent: RoundIntent, opener: Option<&Opener>) -> Vec<SkillId> {
    let base = order(w, c, intent);
    let Some(Opener::Skill(sid)) = opener else { return base };
    let Some(fam) = w.cat.skills.get(*sid).map(|s| s.family_id().to_string()) else { return base };
    let Some(list) = deck_of(w, c).and_then(|d| d.follow_ups_of(&fam)) else { return base };
    let in_deck = auto_families(w, c);
    let mut out: Vec<SkillId> = Vec::new();
    for f in list.iter().filter(|f| in_deck.contains(f)) {
        if let Some(s) = family_skill(w, c, f) {
            if round_intent::skill_intent(&w.cat.skills[s]) == intent.id() && !out.contains(&s) {
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
/// the unit itself.
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
        plan.steps_of(c).iter().filter_map(|st| st.skill).find(|&sid| sid < w.cat.skills.len() && round_intent::skill_intent(&w.cat.skills[sid]) == b.intent.id()).map(Opener::Skill)
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
/// (RS2) and the poise their landing restores (RS3, rule `poise`; before the Max Poise cap).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RestGain {
    pub blocks: i32,
    pub ap: i32,
    pub poise: i32,
}

/// UI2: the Rest gain of `c`'s planned round (its compiled Rest entries; the Dojo's planner view uses it too).
pub fn rest_preview(w: &World, plan: &Plan, c: UnitId) -> RestGain {
    let blocks = plan.entries_for(w, c).iter().filter(|e| skill::kind(&w.cat.skills[e.skill]) == Kind::Rest).map(|e| e.end_block(w) - e.start_block + 1).sum();
    rest_of(w, blocks)
}

/// The Rest gain of `blocks` Rest blocks under the battle's ruleset.
pub fn rest_of(w: &World, blocks: i32) -> RestGain {
    let n = &w.ruleset.numbers;
    let poise = if w.ruleset.has(rpgkit_rules::ruleset::POISE) { n.rest_poise_per_block * blocks } else { 0 };
    RestGain { blocks, ap: round_intent::rest_gain(n, blocks), poise }
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
        if round_intent::skill_intent(s) != self.intent.id() {
            return Err(format!("Only {} skills this round.", self.intent.label(&w.cat)));
        }
        let u = &w.units[self.c];
        let k = skill::kind(s);
        match self.intent {
            RoundIntent::Guard => {
                let mut steps = Vec::new();
                if let RoundTarget::Cell(p) = self.target {
                    if p != st.pos {
                        steps.push(tagged(PlanStep::go_to(self.mv(), p), COVER_TILE, ""));
                    }
                }
                steps.push(tagged(PlanStep::self_skill(sid), OPENER, ""));
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
        }
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
            steps.push(tagged(PlanStep::area_at(sid, goal), OPENER, ""));
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
        let of = round_intent::item_intent(item);
        if of != self.intent.id() {
            return Err(format!("{} is not a {} item.", item.name, self.intent.label(&w.cat)));
        }
        if self.intent == RoundIntent::Maneuver && self.kind != ManeuverKind::Utility {
            return Err(format!("{} opens only a Utility round.", item.name));
        }
        let Some(t) = self.target_unit() else { return Err(format!("{} needs a unit to aim at.", item.name)) };
        let mut steps = Vec::new();
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
            RoundIntent::Guard => Some(PlanStep::self_skill(sid)),
            RoundIntent::Support => match self.target {
                RoundTarget::Unit(t) if t != self.c => (s.target_mode != "SELF" && k != Kind::Focus).then(|| PlanStep::ally(sid, t)),
                _ => (s.target_mode != "ALLY").then(|| PlanStep::self_skill(sid)),
            },
            RoundIntent::Assault | RoundIntent::Maneuver => {
                if skill::is_area(s) {
                    return Some(PlanStep::area_at(sid, self.target_pos(st)));
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
                let mut why = String::new();
                let need = s.blocks + w.recovery_of(self.c, s);
                let ap = if pays { skill::ap_cost_by(s, Some(u), &w.ruleset) } else { 0 };
                let ap_free = st.ap_left - reserve.ap;
                // HOOKS1 HC1: a skill whose HP cost the unit's HP left does not exceed is refused (rule hp_cost)
                let hp = command::hp_cost_problem(st.hp_left, command::hp_cost_of(w, s));
                // HOOKS1 UM2: a skill whose meter cost the planned meters do not cover is refused (rule unit_meters)
                let meter = crate::meter::cost_problem(w, skill::meters_by(s, &w.ruleset), |m| st.meter(m));
                if !fits_before(&st, need, limit) {
                    why = format!("blocks {free} < {need}");
                } else if ap > ap_free {
                    why = format!("AP {} < {ap}", ap_free.max(0));
                } else if !hp.is_empty() {
                    why = hp;
                } else if !meter.is_empty() {
                    why = meter;
                } else if let Some(step) = self.follow_step(&st, sid) {
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

    /// RS1: Rest on every free block left, tagged with the fill's stop cause (PL6).
    fn rest(&self, plan: &mut Plan, cause: &str) {
        let w = self.w;
        let rest = w.book().lowest(Kind::Rest);
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
/// every family it can use that is not a basic, in catalog order), its meter skills, then the ruleset's basics; the row
/// of each family it uses now, only skills with an intent (not neutral), each once. The order never changes in a battle.
pub fn hotkey_skills(w: &World, c: UnitId) -> Vec<SkillId> {
    let mut out: Vec<SkillId> = Vec::new();
    for f in auto_families(w, c) {
        if let Some(sid) = family_skill(w, c, &f) {
            let s = &w.cat.skills[sid];
            if round_intent::skill_intent(s) != rpgkit_rules::ruleset::NEUTRAL && !s.system && !out.contains(&sid) {
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

/// The intent of a skill or item pick (None: neutral, or an id the ruleset does not know).
fn pick_intent_of(w: &World, c: UnitId, p: &Pick) -> Option<RoundIntent> {
    match p {
        Pick::Skill { skill, .. } => RoundIntent::parse(round_intent::skill_intent(&w.cat.skills[*skill])),
        Pick::Item(id) => command::pouch_item(w, c, id).or_else(|| w.cat.items.get(id)).and_then(|i| RoundIntent::parse(round_intent::item_intent(i))),
        _ => None,
    }
}

/// INPUT1 IF4: the intent `picks` would set for `c` (its first skill or item pick's) when no pick holds the round yet
/// and that intent is not the current one; None otherwise. The battle sets it first (`Session::pick`, IN6's lines).
pub fn pick_intent(w: &World, c: UnitId, picks: &[Pick]) -> Option<RoundIntent> {
    if picks_of(w, c).iter().any(Pick::holds_target) {
        return None;
    }
    let i = picks.iter().find(|p| p.holds_target()).and_then(|p| pick_intent_of(w, c, p))?;
    (round_intent::intent_of(w, c) != Some(i)).then_some(i)
}

/// The round target a pick acts on when it is aimed at `at`: a Support skill or item picked on a tile (after "Move
/// here") acts on the unit itself.
fn pick_target(w: &World, c: UnitId, at: RoundTarget, p: &Pick) -> RoundTarget {
    let at = round_intent::normalize(c, at);
    match (pick_intent_of(w, c, p), at) {
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
            let Some(i) = pick_intent_of(w, c, p) else { return Err(format!("{} is not a round skill.", pick_name(w, p))) };
            match holder {
                Some(h) => {
                    // IF6 a, b: the pick-set intent and the round target held by a pick
                    let cur = round_intent::intent_of(w, c);
                    if cur != Some(i) {
                        return Err(format!("Only {} skills this round ({} set it)", cur.map_or_else(|| "other".to_string(), |c| c.label(&w.cat)), pick_name(w, &h)));
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
            Mode::SelfCast if kind == Some(Kind::Rest) => Some(Pick::Rest),
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
    let (r, later) = pick(w, plan, c, at, &picks);
    (r.map(|()| picks.len()), later)
}

/// INPUT1 IF7 Clear: removes every pick of `c` and rebuilds (as `unpick`).
pub fn clear_picks(w: &mut World, plan: &mut Plan, c: UnitId) -> (Result<(), String>, Vec<(UnitId, String)>) {
    set_picks(w, c, Vec::new());
    let first = rebuild_member(w, plan, c);
    (first, rebuild_later(w, plan, c))
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

/// RS1 within `limit`: Rest on every free block up to `limit`, tagged `tag`.
fn rest_to(w: &World, plan: &mut Plan, c: UnitId, tag: &str, limit: i32) {
    let rest = w.book().lowest(Kind::Rest);
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

/// GH1: marks `c`'s steps from index `from` on as ghost (the planner's), all but `keep` (the step after, a pick).
fn mark_ghost(plan: &Plan, c: UnitId, from: usize, keep: Option<&Step>) {
    for (i, s) in plan.steps_of(c).iter().enumerate() {
        s.ghost.set(i >= from && !keep.is_some_and(|k| Rc::ptr_eq(k, s)));
    }
}

/// The free blocks of `st` from its next block up to `limit`.
fn free_blocks(st: &SimState, limit: i32) -> i32 {
    (st.next_block..=limit).filter(|b| !st.blocked_blocks.contains_key(b)).count() as i32
}

/// The last block of the round that is not lost (the step after's block).
fn last_free_block(st: &SimState) -> i32 {
    (1..=st.total_blocks).rev().find(|b| !st.blocked_blocks.contains_key(b)).unwrap_or(st.total_blocks)
}

/// The step after held back for the round's last block (ET4, GH4): its pick index, step and AP.
struct After {
    index: usize,
    step: Step,
    ap: i32,
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
    let opener_at = picks.iter().position(Pick::holds_target);
    for (i, p) in picks.iter().enumerate() {
        let st = plan.compile(w, c);
        let (reserved, reserve_ap) = after.as_ref().map_or((0, 0), |a| (1, a.ap));
        let limit = limit_of(&st, reserved);
        let free = free_blocks(&st, limit);
        let has_after = after.is_some();
        let taken = |need: i32| {
            let tail = if has_after && fits_before(&st, need, st.total_blocks) { format!(" (the step after takes B{})", last_free_block(&st)) } else { String::new() };
            format!("Blocks {free} < {need}{tail}")
        };
        let room = Room { limit, reserve_ap, taken: &taken };
        let ap_free = st.ap_left - reserve_ap;
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
                let mpb = w.units[c].move_per_block();
                if route.len() as i32 > mpb {
                    return Err(fail(format!("Too far for one move block: {} > {mpb}", route.len())));
                }
                if free < 1 {
                    return Err(fail("Blocks 0 < 1 (the picks fill the round)".into()));
                }
                let ap = if pays { route.len() as i32 } else { 0 };
                if ap > ap_free {
                    return Err(refuse(Refusal::ap(ap_free, ap)));
                }
                after = Some(After { index: i, step: tagged(PlanStep::after_move(mv, *cell), STEP_AFTER, ""), ap });
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
            Pick::Rest => {
                if free < 1 {
                    return Err(fail(taken(1)));
                }
                let rest = w.book().lowest(Kind::Rest);
                place(w, plan, c, vec![tagged(PlanStep::self_skill(rest), &format!("pick {}", i + 1), "")], false, &room).map_err(refuse)?;
            }
            Pick::Skill { .. } | Pick::Item(_) => {
                let Some(b) = b.as_ref() else { return Err(fail("Pick a round intent first.".into())) };
                let opener = opener_at == Some(i);
                let tag = if opener { OPENER.to_string() } else { format!("pick {}", i + 1) };
                // the opener brings its before-move (PL5) unless the round already moves (ET5)
                let may_move = opener && moved.is_none();
                let steps = b.pick_steps(plan, &st, p, may_move, &tag, ap_free, &room).map_err(refuse)?;
                place(w, plan, c, steps, may_move, &room).map_err(|e| if e.text == REACH { fail(b.reach_text(plan, &st, p)) } else { refuse(e) })?;
            }
        }
    }
    let mark = plan.steps_of(c).len();
    let reserved = i32::from(after.is_some());
    match b.as_ref().filter(|_| fills(w, c)) {
        Some(b) => {
            // GH3: the fill (FU2, FU3), AS3's reserve unless a step after is picked or the round has its move (ET5)
            let reserve = match &after {
                Some(a) => Reserve { blocks: 1, ap: a.ap, step: None },
                None => b.reserve(plan),
            };
            // FL2: the first skill or item pick (the opener) chooses the follow-up list; an item reads the order
            let opener = opener_at.and_then(|i| match &picks[i] {
                Pick::Skill { skill, .. } => Some(Opener::Skill(*skill)),
                _ => None,
            });
            let cause = b.fill(plan, &reserve, &follow_order(w, c, b.intent, opener.as_ref()));
            if let Some(step) = reserve.step {
                let _ = plan.add_step(w, c, step, false);
            }
            let tag = if cause.is_empty() { REST.to_string() } else { format!("{REST}: {cause}") };
            rest_to(w, plan, c, &tag, limit_of(&plan.compile(w, c), reserved));
        }
        None => rest_to(w, plan, c, REST, limit_of(&plan.compile(w, c), reserved)),
    }
    if let Some(a) = &after {
        let p = plan.add_step(w, c, a.step.clone(), false);
        if !p.is_empty() {
            plan.set_steps(c, Vec::new());
            return Err((a.index, p.into()));
        }
        rest_to(w, plan, c, REST, w.total_blocks);
    }
    mark_ghost(plan, c, mark, after.as_ref().map(|a| &a.step));
    Ok(())
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
    fn pick_steps(&self, plan: &Plan, st: &SimState, p: &Pick, may_move: bool, tag: &str, ap_free: i32, room: &Room) -> Result<Vec<Step>, Refusal> {
        let w = self.w;
        let u = &w.units[self.c];
        let pays = w.pays_ap(self.c);
        match p {
            Pick::Skill { skill: sid, .. } => {
                let s = self.skill(*sid);
                if round_intent::skill_intent(s) != self.intent.id() {
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
                let hp = command::hp_cost_problem(st.hp_left, command::hp_cost_of(w, s));
                if !hp.is_empty() {
                    return Err(hp.into());
                }
                let meter = crate::meter::cost_problem(w, skill::meters_by(s, &w.ruleset), |m| st.meter(m));
                if !meter.is_empty() {
                    return Err(meter.into());
                }
                if may_move {
                    // the opener's own steps, tagged as PL5's (only the opener may move)
                    return Ok(self.skill_opener(plan, st, *sid)?);
                }
                if let Some(e) = self.reach_problem(plan, st, p) {
                    return Err(e.into());
                }
                let Some(step) = self.follow_step(st, *sid) else { return Err(format!("{} can't act on {}", s.name, self.target_name()).into()) };
                Ok(vec![tagged(step, tag, "")])
            }
            Pick::Item(id) => {
                let Some(item) = command::pouch_item(w, self.c, id) else { return Err(format!("{} carries no {id}.", u.name).into()) };
                if round_intent::item_intent(item) != self.intent.id() {
                    return Err(format!("Only {} skills this round", self.intent.label(&w.cat)).into());
                }
                let need = item.use_blocks.max(1);
                if !fits_before(st, need, room.limit) {
                    return Err((room.taken)(need).into());
                }
                let ap = if pays { command::ITEM_AP } else { 0 };
                if ap > ap_free {
                    return Err(Refusal::ap(ap_free, ap));
                }
                if may_move {
                    return Ok(self.item_opener(plan, st, id)?);
                }
                if let Some(e) = self.reach_problem(plan, st, p) {
                    return Err(e.into());
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
                let t = self.target_unit()?;
                t != self.c && command::pouch_item(w, self.c, id).is_some_and(|i| st.pos.dist(goal) > i.battle_reach())
            }
            Pick::Skill { skill: sid, .. } => {
                let s = self.skill(*sid);
                match self.intent {
                    RoundIntent::Guard | RoundIntent::Channel => false,
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
                let reach = command::pouch_item(w, self.c, id).map_or(1, |i| i.battle_reach());
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
    let steps = plan.steps_of(c);
    let none = picks_of(w, c).is_empty();
    plan.entries_for(w, c)
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
    let ghost = ghost_entries(w, plan, c);
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
    let cur = round_intent::intent_of(w, c);
    let cur_kind = round_intent::maneuver_of(w, c);
    let skills = hotkey_skills(w, c);
    let intent_is = |s: &SkillDef, i: RoundIntent| round_intent::skill_intent(s) == i.id();
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
                if skill::is_area(s) && intent_is(s, RoundIntent::Assault) {
                    rows.push(skill_row(sid, None));
                } else if skill::is_area(s) && intent_is(s, RoundIntent::Maneuver) {
                    rows.push(skill_row(sid, Some(ManeuverKind::Utility)));
                }
            }
            if held {
                rows.push((vec![Pick::StepAfter(p)], at));
            }
        }
    }
    rows.push((vec![Pick::Rest], at));
    let base = picked_cost(w, plan, c);
    let mut out = Vec::new();
    for (picks, row_at) in rows {
        let first = picks.iter().find(|p| p.holds_target()).cloned();
        let intent = first.as_ref().and_then(|p| pick_intent_of(w, c, p));
        let kind = match first {
            Some(Pick::Skill { kind, .. }) if intent == Some(RoundIntent::Maneuver) => kind,
            Some(Pick::Item(_)) if intent == Some(RoundIntent::Maneuver) => Some(ManeuverKind::Utility),
            _ => None,
        };
        // IF2: after the first pick only the round intent's (and kind's) rows, and no Move here (ET2)
        let other = intent.is_some() && intent != cur;
        let other_kind = intent == Some(RoundIntent::Maneuver) && kind.is_some_and(|k| k != cur_kind);
        if held && (other || other_kind || picks.iter().any(|p| matches!(p, Pick::MoveHere(_)))) {
            continue;
        }
        let (problem, ap_short, ap, blocks) = match trial(w, plan, c, row_at, &picks) {
            Ok((w2, p2)) => {
                let now = picked_cost(&w2, &p2, c);
                (None, None, now.0 - base.0, now.1 - base.1)
            }
            Err(e) => (Some(e.text), e.ap_short, 0, 0),
        };
        if held && problem.is_some() {
            continue;
        }
        let drops = intent.filter(|_| !held).map(|i| round_intent::would_drop(w, c, i)).unwrap_or_default();
        out.push(PickOption { picks, intent, kind, ap, blocks, problem, ap_short, drops });
    }
    out
}
