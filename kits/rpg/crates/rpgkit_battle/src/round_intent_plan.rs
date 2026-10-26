//! The round intent auto planner (RULESET1_INTENT 4, 7, 8; RULESET1-PL; engine-only, no Godot original): from a
//! unit's intent, round target, Maneuver kind and optional opener it builds the round's steps through `Plan::add_step`
//! (every rule is checked there), each with a reason tag (PL6): the before-move (approach, kite, the move to a Guard
//! tile), the opener (FU1, PL3), the reserved after-move (AS3) or the Disengage jump (MV8), the follow-ups from the
//! member's order (`deck.order`, FU2-FU4) and Rest on every block left (RS1). Manual mode (MM1-MM3) keeps the player's
//! steps and fills the rest on request. Deterministic (PL1): orders are lists, tiles are ranked by PL4 down to the cell
//! index, nothing draws a random number. Inert unless the battle's ruleset has `round_intent`.

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

/// The unit's deck (FU4): its catalog row's `deck` (RULESET1-D gives party members theirs); None when empty (auto).
pub fn deck_of(w: &World, c: UnitId) -> Option<&Deck> {
    w.cat.units.get(&w.units[c].catalog_id).map(|d| &d.deck).filter(|d| !d.is_empty())
}

/// The row of skill family `fam` that `c` uses now: its highest tier the unit can use within its skill cap; None when
/// it has none.
pub fn family_skill(w: &World, c: UnitId, fam: &str) -> Option<SkillId> {
    let u = &w.units[c];
    let cap = w.skill_cap_of(c);
    let row = w.cat.family_rows(fam).into_iter().rev().find(|s| s.level <= cap && !skill::is_item(s) && u.can_use(&w.cat, s))?;
    w.book().find(&row.id)
}

/// The families of FU4's auto order before the intent filter: the deck's actives (without a deck: every family the
/// unit can use that is not a basic, in catalog order), then the basics (the ruleset's `basics` in their order; a row
/// without any: the catalog's basic skills).
fn auto_families(w: &World, c: UnitId) -> Vec<String> {
    let basics: Vec<String> = if w.ruleset.numbers.basics.is_empty() { w.cat.basic_ids().iter().map(|id| w.cat.family_of(id).to_string()).collect() } else { w.ruleset.numbers.basics.clone() };
    let mut out: Vec<String> = match deck_of(w, c).filter(|d| !d.active.is_empty()) {
        Some(d) => d.active.clone(),
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

/// MM1: whether `c` plans by hand.
pub fn manual(w: &World, c: UnitId) -> bool {
    w.round_states.get(&c).is_some_and(|r| r.manual)
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
/// automatically (approach tiles are claims). Returns `c`'s result and the later members' refusals.
pub fn replan(w: &mut World, plan: &mut Plan, c: UnitId) -> (Result<(), String>, Vec<(UnitId, String)>) {
    let first = plan_unit(w, plan, c);
    let side: Vec<UnitId> = if w.allies.contains(&c) { w.allies.clone() } else { w.enemies.clone() };
    let mut refused = Vec::new();
    for &m in side.iter().skip_while(|&&m| m != c).skip(1) {
        if !w.units[m].is_active() || manual(w, m) || round_intent::intent_of(w, m).is_none() {
            continue;
        }
        if let Err(e) = plan_unit(w, plan, m) {
            refused.push((m, e));
        }
    }
    (first, refused)
}

/// MM1, MM3: the Manual toggle. On keeps the steps for the player to edit; off rebuilds the auto plan (and the later
/// members', PL8).
pub fn set_manual(w: &mut World, plan: &mut Plan, c: UnitId, on: bool) -> Result<(), String> {
    if !round_intent::on(w) {
        return Err("This battle has no round intents.".into());
    }
    w.round_states.entry(c).or_default().manual = on;
    if on {
        return Ok(());
    }
    replan(w, plan, c).0
}

/// MM3 "Fill the rest": PL5's fill after the last step placed by hand (follow-ups from the order on the round target,
/// from the tile the unit stands on then), then Rest on every free block.
pub fn fill_rest(w: &World, plan: &mut Plan, c: UnitId) -> Result<(), String> {
    let b = Builder::new(w, c)?;
    let cause = b.fill(plan, &Reserve::default());
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
    if let Err(e) = b.opener(plan, opener) {
        plan.set_steps(c, Vec::new());
        return Err(e);
    }
    let reserve = b.reserve(plan);
    let cause = b.fill(plan, &reserve);
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
            return Err(format!("{} has no target.", intent.name()));
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
    fn label(&self) -> &'static str {
        match (self.intent, self.kind) {
            (RoundIntent::Maneuver, ManeuverKind::Engage) => "Engage",
            (RoundIntent::Maneuver, ManeuverKind::Disengage) => "Disengage",
            (RoundIntent::Maneuver, ManeuverKind::Utility) => "Utility",
            (i, _) => i.name(),
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

    /// Places the before-move and the opener: the given one, else the first order entry that fits (PL3).
    fn opener(&self, plan: &mut Plan, given: Option<Opener>) -> Result<(), String> {
        if let Some(o) = given {
            return self.try_opener(plan, &o);
        }
        if self.order.is_empty() {
            return Err(format!("{} has no {} skill.", self.w.units[self.c].name, self.intent.name()));
        }
        let mut first: Option<String> = None;
        for &sid in &self.order {
            // MV8: a DISENGAGE skill (a jump) belongs to Disengage rounds, and there goes last while a Maneuver attack
            // can open
            if k_is_jump(self.skill(sid)) && (!self.disengage() || self.order.iter().any(|&o| !k_is_jump(self.skill(o)))) {
                continue;
            }
            match self.try_opener(plan, &Opener::Skill(sid)) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    first.get_or_insert(e);
                }
            }
        }
        Err(match self.target {
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
            return Err(format!("Only {} skills this round.", self.intent.name()));
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
        if maneuver && carries(s) {
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
            return Err(format!("{} is not a {} item.", item.name, self.intent.name()));
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
        let mut left = reserve.blocks;
        let mut b = st.total_blocks;
        while left > 0 && b > 0 {
            if !st.blocked_blocks.contains_key(&b) {
                left -= 1;
            }
            b -= 1;
        }
        b
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
                if self.intent == RoundIntent::Maneuver && carries(s) && self.kind != ManeuverKind::Disengage {
                    // MV7: a follow-up moves only when its tile no longer meets the condition
                    step.cell.set(st.pos);
                }
                Some(step)
            }
            RoundIntent::Channel => None,
        }
    }

    /// FU2, FU3: block by block, the first order entry that has the intent, fits before the reserve, is affordable
    /// without debt (and leaves the reserve's AP), acts on the target from the unit's tile and is a skill. Returns the
    /// first refused entry's cause at the block where the fill stops ("" when the blocks ran out).
    fn fill(&self, plan: &mut Plan, reserve: &Reserve) -> String {
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
            for (k, &sid) in self.order.iter().enumerate() {
                let s = self.skill(sid);
                if k_is_jump(s) {
                    continue;
                }
                let mut why = String::new();
                let need = s.blocks + w.recovery_of(self.c, s);
                let ap = if pays { skill::ap_cost_by(s, Some(u), &w.ruleset) } else { 0 };
                let ap_free = st.ap_left - reserve.ap;
                if !fits_before(&st, need, limit) {
                    why = format!("blocks {free} < {need}");
                } else if ap > ap_free {
                    why = format!("AP {} < {ap}", ap_free.max(0));
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

/// Whether `need` blocks fit in a row of free blocks from `st.next_block` ending at `limit` or before.
fn fits_before(st: &SimState, need: i32, limit: i32) -> bool {
    (st.next_block..=limit - need + 1).any(|s| (s..s + need).all(|b| !st.blocked_blocks.contains_key(&b)))
}
