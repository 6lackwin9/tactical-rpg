//! The round intent (RULESET1_INTENT 2-6, rule `round_intent`; engine-only, no Godot original): each unit's intent,
//! which persists across rounds and limits its skills; the round target (one a round, the unit's lock); the intent-bound
//! buffs that end on an intent change; Rest paid at round end; battle facing, engagement and flanks; Guard stealth; the
//! positions a Maneuver skill dashes to (INPUT1 MR1-MR6), diagonal melee (MR10) and the tile ranking (PL4) its moves and
//! after-moves use. Every function is inert unless the battle's ruleset has `round_intent` (flank bonuses also need
//! `flank`), so classic never reaches it.

use crate::command::{cover_for, dash_fits};
use crate::grid::{V2, NO_CELL, ZERO};
use crate::path::{self, Cells, Hooks};
use crate::skill::{self, Kind};
use crate::world::{UnitId, World};
use rpgkit_data::{ItemDef, Position, SkillDef};
use rpgkit_rules::ruleset::{self, MeleeDiagonal, Numbers};
use rpgkit_rules::status;

/// RULESET1_INTENT IN1: the intents a unit may hold (`channel` only when the ruleset row lists it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundIntent {
    Assault,
    Guard,
    Support,
    Maneuver,
    Channel,
}

impl RoundIntent {
    pub const ALL: [RoundIntent; 5] = [RoundIntent::Assault, RoundIntent::Guard, RoundIntent::Support, RoundIntent::Maneuver, RoundIntent::Channel];

    /// The id (`rpgkit_rules::ruleset::INTENTS`).
    pub fn id(self) -> &'static str {
        match self {
            RoundIntent::Assault => "assault",
            RoundIntent::Guard => "guard",
            RoundIntent::Support => "support",
            RoundIntent::Maneuver => "maneuver",
            RoundIntent::Channel => "channel",
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|i| i.id() == id)
    }

    /// TERMS1: the intent's term id (`intent_assault`, ...; `rpgkit_data::terms`).
    pub fn term_id(self) -> &'static str {
        use rpgkit_data::terms::*;
        match self {
            RoundIntent::Assault => INTENT_ASSAULT,
            RoundIntent::Guard => INTENT_GUARD,
            RoundIntent::Support => INTENT_SUPPORT,
            RoundIntent::Maneuver => INTENT_MANEUVER,
            RoundIntent::Channel => INTENT_CHANNEL,
        }
    }

    /// The default display name ("Assault"): the term register's default (TERMS1 TM2). The battle's own name for the
    /// intent (the game's `terms` table) is `label`.
    pub fn name(self) -> &'static str {
        rpgkit_data::terms::default_name(self.term_id())
    }

    /// TERMS1 TM3: the intent's name through `Catalog::term` on `cat` (the battle's catalog): the game's `terms` row,
    /// else `name`.
    pub fn label(self, cat: &rpgkit_data::Catalog) -> String {
        cat.term(self.term_id()).name
    }
}

/// RULESET1_INTENT TG1: the round target: the unit itself, a unit (ally or enemy) or a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundTarget {
    Myself,
    Unit(UnitId),
    Cell(V2),
}

/// RULESET1_INTENT 5.3: the kind of a Maneuver round (MV11: Engage when none is picked).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManeuverKind {
    Engage,
    Disengage,
    Utility,
}

/// A unit's round-intent state in a battle (RULESET1_TASKS 1.8). The intent and the facing persist across rounds; the
/// rest is reset at every round start.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RoundState {
    pub intent: Option<RoundIntent>,
    /// The explicit round target (None = the first targeting step's, or the lock: TG3).
    pub target: Option<RoundTarget>,
    pub maneuver: Option<ManeuverKind>,
    /// MV1: the sign vector toward the round target (ZERO = none yet).
    pub facing: V2,
    /// GD5: a move of the unit landed this round (walk, jump, dash, chase, carried move).
    pub moved: bool,
    /// RS2: Rest blocks that landed this round.
    pub rest_blocks: i32,
    /// PL1, FU1 (RULESET1-PL): the opener the player picked for this round (None = the planner's, PL3).
    pub opener: Option<Opener>,
    /// MM1: the member plans by hand (the auto planner leaves its steps alone); persists across rounds.
    pub manual: bool,
    /// MV11: `maneuver` was picked by the auto planner (Engage, else Disengage, else Utility), not by the player.
    pub maneuver_auto: bool,
    /// TG6: last round's target (the planner proposes it while it is valid).
    pub last_target: Option<RoundTarget>,
    /// INPUT1 GH1, GH6: the member's picks this round, in order (the planner's ghost fills the rest); cleared at every
    /// round start; in no trace or save.
    pub picks: Vec<Pick>,
    /// ENCOUNTER2-C SR3: a scripted entry's own ghost fill this round (Some(true) the planner's fill, Some(false) Rest),
    /// read instead of the Auto-fill setting; None = the setting. Cleared at every round start; in no trace or save.
    pub fill: Option<bool>,
}

/// INPUT1 IF4, ET1-ET4 (`round_intent_plan::pick`): one pick of a member's round. A skill (with the Maneuver kind of
/// its menu row; None = the planner's), a pouch item, Rest, a plain move to a tile first ("Move here") or one move
/// block at the round's end ("Step here after").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pick {
    Skill { skill: usize, kind: Option<ManeuverKind> },
    Item(String),
    Rest,
    MoveHere(V2),
    StepAfter(V2),
}

impl Pick {
    /// A skill or item pick: it holds the round target and its intent (IF4); Rest and the moves are neutral.
    pub fn holds_target(&self) -> bool {
        matches!(self, Pick::Skill { .. } | Pick::Item(_))
    }
}

/// RULESET1_INTENT FU1, PL1: a round's opener: a skill (catalog index) or a pouch item (its id).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opener {
    Skill(usize),
    Item(String),
}

/// Whether the battle plays by the round intent.
pub fn on(w: &World) -> bool {
    w.ruleset.has(ruleset::ROUND_INTENT)
}

/// The unit's round intent (None with the rule off or none picked yet).
pub fn intent_of(w: &World, c: UnitId) -> Option<RoundIntent> {
    if !on(w) {
        return None;
    }
    w.round_states.get(&c).and_then(|r| r.intent)
}

/// RULESET1_INTENT IN4: a skill's intent id (its `round_intent`, else its kind's).
pub fn skill_intent(s: &SkillDef) -> &str {
    if s.round_intent.is_empty() {
        ruleset::kind_intent(&s.kind)
    } else {
        &s.round_intent
    }
}

/// RULESET1_INTENT IN5: a pouch item's intent by aim: foe-aimed Maneuver, else Support.
pub fn item_intent(item: &ItemDef) -> &'static str {
    if item.aims_at_foes() {
        "maneuver"
    } else {
        "support"
    }
}

/// RULESET1_INTENT PL2 (the intent table of section 2): "" when `target` fits `intent` for unit `c`, else the reason.
pub fn target_problem(w: &World, c: UnitId, intent: RoundIntent, target: RoundTarget) -> String {
    let unit_ok = |t: UnitId| w.units.get(t).is_some_and(|u| u.is_active()) && w.has(t);
    let foe = |t: UnitId| w.foes_of(c).contains(&t);
    let friend = |t: UnitId| t == c || w.friends_of(c).contains(&t);
    match target {
        RoundTarget::Unit(t) if !unit_ok(t) => return "That unit is not on the field.".into(),
        RoundTarget::Cell(p) if !w.grid.is_valid(p) => return "Aim inside the arena.".into(),
        _ => {}
    }
    let fits = match (intent, target) {
        (RoundIntent::Assault | RoundIntent::Maneuver, RoundTarget::Unit(t)) => foe(t),
        (RoundIntent::Assault | RoundIntent::Maneuver, RoundTarget::Cell(_)) => true,
        (RoundIntent::Assault | RoundIntent::Maneuver, RoundTarget::Myself) => false,
        (RoundIntent::Guard, RoundTarget::Myself | RoundTarget::Cell(_)) => true,
        (RoundIntent::Guard, RoundTarget::Unit(t)) => t == c,
        (RoundIntent::Support, RoundTarget::Myself) => true,
        (RoundIntent::Support, RoundTarget::Unit(t)) => friend(t),
        (RoundIntent::Support, RoundTarget::Cell(_)) => false,
        (RoundIntent::Channel, _) => true,
    };
    if fits {
        return String::new();
    }
    match intent {
        RoundIntent::Assault | RoundIntent::Maneuver => format!("{} targets an enemy or a tile.", intent.label(&w.cat)),
        RoundIntent::Guard => "Guard targets yourself or a tile.".into(),
        RoundIntent::Support => "Support targets yourself or an ally.".into(),
        RoundIntent::Channel => String::new(),
    }
}

/// `RoundTarget::Unit(c)` of the unit itself is `Myself`.
pub fn normalize(c: UnitId, t: RoundTarget) -> RoundTarget {
    match t {
        RoundTarget::Unit(u) if u == c => RoundTarget::Myself,
        other => other,
    }
}

/// RULESET1_INTENT IN2, IN6: sets `c`'s intent (only while planning; the caller checks). The first pick is not a
/// change; a change drops every intent-bound instance the unit holds and clears a round target that no longer fits.
/// Returns the log lines ("Bryn leaves Guard: Riposte ends"), or the reason it is refused.
pub fn set_intent(w: &mut World, c: UnitId, intent: RoundIntent) -> Result<Vec<String>, String> {
    if !on(w) {
        return Err("This battle has no round intents.".into());
    }
    if !w.ruleset.numbers.intents.iter().any(|i| i == intent.id()) {
        return Err(format!("{} is not an intent of this ruleset.", intent.label(&w.cat)));
    }
    if !w.units[c].is_active() {
        return Err(format!("{} is not on the field.", w.units[c].name));
    }
    let old = w.round_states.get(&c).and_then(|r| r.intent);
    let mut lines = Vec::new();
    if old.is_some_and(|o| o != intent) {
        let from = old.unwrap().label(&w.cat);
        for name in drop_bound(w, c) {
            lines.push(format!("{} leaves {from}: {name} ends", w.units[c].name));
        }
    }
    let target = w.round_states.get(&c).and_then(|r| r.target);
    let keep = target.is_some_and(|t| target_problem(w, c, intent, t).is_empty());
    let rs = w.round_states.entry(c).or_default();
    rs.intent = Some(intent);
    if !keep {
        rs.target = None;
    }
    if intent != RoundIntent::Maneuver {
        rs.maneuver = None;
    }
    // PL8: an opener picked for another intent no longer applies
    if old != Some(intent) {
        rs.opener = None;
    }
    Ok(lines)
}

/// RULESET1_INTENT TG1, TG3, PL2: sets `c`'s round target (the intent first); a unit target becomes the unit's lock.
pub fn set_target(w: &mut World, c: UnitId, target: RoundTarget) -> Result<(), String> {
    let Some(intent) = intent_of(w, c) else { return Err("Pick a round intent first.".into()) };
    let target = normalize(c, target);
    let problem = target_problem(w, c, intent, target);
    if !problem.is_empty() {
        return Err(problem);
    }
    w.round_states.entry(c).or_default().target = Some(target);
    if let RoundTarget::Unit(t) = target {
        w.units[c].active_target = Some(t);
    }
    update_facing(w, c);
    Ok(())
}

/// MV11: the kind of a Maneuver round.
pub fn set_maneuver(w: &mut World, c: UnitId, kind: ManeuverKind) -> Result<(), String> {
    if intent_of(w, c) != Some(RoundIntent::Maneuver) {
        return Err("Engage, Disengage and Utility are Maneuver kinds.".into());
    }
    let rs = w.round_states.entry(c).or_default();
    rs.maneuver = Some(kind);
    rs.maneuver_auto = false;
    Ok(())
}

/// The kind of `c`'s Maneuver round (Engage when none is picked: MV11).
pub fn maneuver_of(w: &World, c: UnitId) -> ManeuverKind {
    w.round_states.get(&c).and_then(|r| r.maneuver).unwrap_or(ManeuverKind::Engage)
}

/// IN6: tags the intent-bound instances among `statuses` just granted to `holder` by `source`'s skill: every status
/// whose row is intent-bound, and every status of an `intent_buff` skill granted to its own user. The tag is the
/// holder's intent ("" with none).
pub fn bind(w: &mut World, holder: UnitId, source: UnitId, statuses: &[String], intent_buff: bool) {
    if !on(w) {
        return;
    }
    let tag = intent_of(w, holder).map_or("", |i| i.id()).to_string();
    for st in statuses.iter().filter(|s| !s.is_empty()) {
        if status::intent_bound(st) || (intent_buff && holder == source) {
            w.units[holder].st.intents.insert(st.clone(), tag.clone());
        }
    }
}

/// IN6: removes every intent-bound instance `c` holds (active and queued); their display names, in tag order.
pub fn drop_bound(w: &mut World, c: UnitId) -> Vec<String> {
    let bound: Vec<String> = w.units[c].st.intents.keys().cloned().collect();
    let mut names = Vec::new();
    for st in bound {
        if status::remove(&mut w.units[c], &st) {
            names.push(status::display_name(&st));
        }
        w.units[c].st.intents.shift_remove(&st);
    }
    names
}

/// IN6 for the intent buttons (RULESET1_INTENT UI1, "loses Riposte"): the display names of the intent-bound instances
/// `c` holds that picking `intent` would drop (`drop_bound`'s, in tag order); empty when `intent` is the current one or
/// the unit holds none yet (the first pick is not a change: IN2). Nothing changes.
pub fn would_drop(w: &World, c: UnitId, intent: RoundIntent) -> Vec<String> {
    if intent_of(w, c).is_none_or(|old| old == intent) {
        return Vec::new();
    }
    let st = &w.units[c].st;
    st.intents.keys().filter(|id| st.has_any(id)).map(|id| status::display_name(id)).collect()
}

/// RULESET1_INTENT RS2: the AP a unit gains at round end for `blocks` landed Rest blocks.
pub fn rest_gain(n: &Numbers, blocks: i32) -> i32 {
    ((blocks.max(0) as f64 * n.rest_ap_per_block).floor() as i32).clamp(0, n.rest_ap_cap.max(0))
}

/// Round start (after the status roll-over and `ap_regen`): RS2's Rest gain for the units that pay AP, then the
/// round's target, kind, opener, picks, move and Rest count reset (the intent, facing and Manual toggle stay: IN2, MV1,
/// MM1, INPUT1 GH6).
/// Returns the gain lines. CLN-45: the Rest gain is `rest_ap` round_end's (the Rest blocks are counted only then);
/// without `round_intent` a unit's state holds only its Rest count, which goes once paid.
pub fn round_start(w: &mut World) -> Vec<String> {
    let intents = on(w);
    if !intents && !w.ruleset.rest_at_round_end() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let units: Vec<UnitId> = w.round_states.keys().copied().collect();
    for c in units {
        let blocks = w.round_states[&c].rest_blocks;
        let gain = rest_gain(&w.ruleset.numbers, blocks);
        if gain > 0 && w.pays_ap(c) && w.units[c].is_active() {
            let u = &mut w.units[c];
            u.ap = u.max_ap.min(u.ap + gain);
            lines.push(format!("[Rest] {} recovers {gain} AP from {blocks} Rest block{} (AP {}/{})", u.name, if blocks == 1 { "" } else { "s" }, u.ap, u.max_ap));
        }
        if !intents {
            w.round_states.shift_remove(&c);
            continue;
        }
        let rs = w.round_states.get_mut(&c).unwrap();
        // TG6 (RULESET1-PL): the planner proposes last round's target; the opener and the kind are picked again
        rs.last_target = rs.target.take();
        rs.maneuver = None;
        rs.maneuver_auto = false;
        rs.opener = None;
        rs.moved = false;
        rs.rest_blocks = 0;
        // INPUT1 GH6: the picks go with the target, kind and opener; the intent carries over
        rs.picks.clear();
        rs.fill = None;
    }
    lines
}

/// Battle start: every unit gets its state; MV1's first facing (party right, foes left) and a valid starting intent
/// (TQ23 `default_intent`, `defaults[unit]`; "" = none).
pub fn battle_start(w: &mut World, defaults: &[(UnitId, String)]) {
    if !on(w) {
        return;
    }
    for c in 0..w.units.len() {
        let facing = if w.enemies.contains(&c) { V2::new(-1, 0) } else { V2::new(1, 0) };
        w.round_states.entry(c).or_default().facing = facing;
    }
    for (c, id) in defaults {
        if let Some(i) = RoundIntent::parse(id).filter(|i| w.ruleset.numbers.intents.iter().any(|x| x == i.id())) {
            w.round_states.entry(*c).or_default().intent = Some(i);
        }
    }
}

/// GD5 bookkeeping: a move of `c` landed this round.
pub fn note_moved(w: &mut World, c: UnitId) {
    if on(w) {
        w.round_states.entry(c).or_default().moved = true;
    }
}

/// RULESET1_INTENT GD5: a single-target attack locked on `t` finds nothing: it holds `stealth` and moved this round.
pub fn hidden(w: &World, t: UnitId) -> bool {
    on(w) && w.units[t].has_status("stealth") && w.round_states.get(&t).is_some_and(|r| r.moved)
}

// ---- Facing, engagement, flanks (MV1-MV3) -------------------------------------------------------------------------

/// The tile `c` faces toward: its round target's (a unit's tile, a cell), else its lock's; None for a self target or
/// nothing on the field (it keeps its last facing).
pub fn facing_point(w: &World, c: UnitId) -> Option<V2> {
    let unit_at = |t: UnitId| (w.units[t].is_active() && w.has(t)).then(|| w.pos(t));
    match w.round_states.get(&c).and_then(|r| r.target) {
        Some(RoundTarget::Myself) => None,
        Some(RoundTarget::Cell(p)) => Some(p),
        Some(RoundTarget::Unit(t)) => unit_at(t),
        None => w.units[c].active_target.and_then(unit_at),
    }
}

fn sign(v: V2) -> V2 {
    V2::new(v.x.signum(), v.y.signum())
}

/// MV1: re-reads `c`'s facing (the sign vector toward its round target).
pub fn update_facing(w: &mut World, c: UnitId) {
    if !on(w) || !w.has(c) {
        return;
    }
    let here = w.pos(c);
    if let Some(p) = facing_point(w, c).filter(|p| *p != here) {
        w.round_states.entry(c).or_default().facing = sign(p - here);
    }
}

/// MV1: every unit re-reads its facing (each block start).
pub fn update_facings(w: &mut World) {
    if !on(w) {
        return;
    }
    for c in 0..w.units.len() {
        update_facing(w, c);
    }
}

/// `c`'s facing (ZERO = none).
pub fn facing(w: &World, c: UnitId) -> V2 {
    w.round_states.get(&c).map_or(ZERO, |r| r.facing)
}

/// RULESET1 K1: `c`'s melee reach (2 with a reach weapon under the rule `reach`, else 1); MV2's engaged band is 1 to it.
pub fn melee_reach(w: &World, c: UnitId) -> i32 {
    skill::melee_reach(&w.units[c], &w.ruleset)
}

/// MV2: the unit `t` is engaged with: its round target (the lock), a foe of `t` on the field within `t`'s melee band
/// (diagonals in it under `melee_diagonal` all, INPUT1 MR10).
pub fn engaged_with(w: &World, t: UnitId) -> Option<UnitId> {
    if !on(w) || !w.has(t) {
        return None;
    }
    let e = match w.round_states.get(&t).and_then(|r| r.target) {
        Some(RoundTarget::Unit(e)) => Some(e),
        Some(_) => None,
        None => w.units[t].active_target,
    }?;
    // INPUT1 MR10: under `melee_diagonal` all a diagonal neighbour is in the melee band
    let diag = w.ruleset.melee_diagonal() == MeleeDiagonal::All;
    let d = if w.has(e) && w.units[e].is_active() { band_dist(diag, w.pos(t), w.pos(e)) } else { return None };
    (w.foes_of(t).contains(&e) && d >= 1 && d <= melee_reach(w, t)).then_some(e)
}

/// Whether `cell` lies in `t`'s front: `dot(cell - t, facing(t)) > 0` (no facing: no front).
pub fn in_front(w: &World, t: UnitId, cell: V2) -> bool {
    let f = facing(w, t);
    if f == ZERO || !w.has(t) {
        return false;
    }
    let d = cell - w.pos(t);
    d.x * f.x + d.y * f.y > 0
}

/// MV3: `a`'s attack on `t` is a flank: `t` is engaged with a unit other than `a`, and `a` stands outside `t`'s front
/// (beside or behind). Drawn under `round_intent`; the bonus needs `flank` too (`flank_bonus`).
pub fn is_flank(w: &World, a: UnitId, t: UnitId) -> bool {
    if !on(w) || a == t || !w.has(a) || !w.has(t) || facing(w, t) == ZERO {
        return false;
    }
    flank_at(w, a, w.pos(a), t)
}

/// MV3 from `cell`: `t` is engaged with a unit other than `a` and `cell` lies outside `t`'s front.
fn flank_at(w: &World, a: UnitId, cell: V2, t: UnitId) -> bool {
    engaged_with(w, t).is_some_and(|e| e != a) && !in_front(w, t, cell)
}

/// MV3 with the rule `flank`: no riposte from `t`, `exploit` `flanked` applies, power x `flank_power_mult`.
pub fn flank_bonus(w: &World, a: UnitId, t: UnitId) -> bool {
    w.ruleset.has(ruleset::FLANK) && is_flank(w, a, t)
}

/// RULESET1-U slice 5 (the forecasts): `flank_bonus` with `a` striking from `cell`, a planned tile (the hit after a
/// move), instead of where it stands now.
pub fn flank_bonus_from(w: &World, a: UnitId, cell: V2, t: UnitId) -> bool {
    w.ruleset.has(ruleset::FLANK) && on(w) && a != t && w.has(a) && w.has(t) && facing(w, t) != ZERO && flank_at(w, a, cell, t)
}

// ---- Tiles (MV5, MV6, PL4) -----------------------------------------------------------------------------------------

/// Where a ranked move goes: nearer the target (an approach, an Engage) or away from it (a kite step, a retreat).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aim {
    Toward,
    Away,
}

/// PL4 (1): the active foes of `c` whose next-round reach covers `cell` (their 2-block move plus their reach); HOOKS1
/// AT3 (rule ai_tactics, never in classic): plus 1 for a ground hazard on `cell` that would harm `c` and that its side
/// can see (`ground::harms`), plus 1 when an arena hazard is due on `cell` this round (AH3, rule arena_hazards too).
pub fn threat(w: &World, c: UnitId, cell: V2) -> usize {
    let foes = w
        .foes_of(c)
        .iter()
        .filter(|&&f| {
            let u = &w.units[f];
            let reach = skill::threat_reach(u, &w.ruleset);
            u.is_active() && w.has(f) && w.pos(f).dist(cell) <= u.move_per_block() * 2 + reach
        })
        .count();
    if !w.ai_tactics() {
        return foes;
    }
    foes + crate::ground::harms(w, c, cell) as usize + crate::arena::due_on(w, cell) as usize
}

/// HOOKS1 AT2 `hold_line` (rule ai_tactics): PL4's rank right after threat for a unit with the tactic: (0 next to a
/// friend on the field else 1, the walkable free neighbours of `cell`), fewer first; (0, 0) for every other unit.
pub fn hold_rank(w: &World, c: UnitId, cell: V2) -> (i32, i32) {
    if !w.has_tactic(c, crate::world::UnitTactic::HoldLine) {
        return (0, 0);
    }
    let beside = w.friends_of(c).iter().any(|&f| f != c && w.units[f].is_active() && w.has(f) && w.pos(f).dist(cell) == 1);
    let free = crate::grid::DIRS
        .iter()
        .map(|d| cell + *d)
        .filter(|&n| w.grid.is_valid(n) && w.grid.is_walkable(n) && !w.grid.is_exit(n) && w.occupant(n).is_none_or(|o| o == c))
        .count() as i32;
    (i32::from(!beside), free)
}

/// PL4: ranks candidate tiles (tile, route) for `c`'s move toward or away from `target`: fewer threats, (HOOKS1 AT2,
/// a `hold_line` unit) next to a friend then fewer free neighbours, more cover toward the target (only when `cover`),
/// fewer path tiles, nearer (Toward) or farther (Away), lower cell index.
pub fn rank_tiles(w: &World, c: UnitId, target: V2, cover: bool, aim: Aim, cands: &mut [(V2, Vec<V2>)]) {
    let key = |cell: V2, route: &Vec<V2>| {
        let cov = if cover { cover_for(&w.grid, target, cell, true) } else { 0.0 };
        let d = cell.dist(target);
        ((threat(w, c, cell), hold_rank(w, c, cell)), cov, route.len(), if aim == Aim::Away { -d } else { d }, cell.y * crate::grid::GRID_W + cell.x)
    };
    cands.sort_by(|a, b| {
        let (ka, kb) = (key(a.0, &a.1), key(b.0, &b.1));
        ka.0.cmp(&kb.0).then(kb.1.partial_cmp(&ka.1).unwrap_or(std::cmp::Ordering::Equal)).then(ka.2.cmp(&kb.2)).then(ka.3.cmp(&kb.3)).then(ka.4.cmp(&kb.4))
    });
}

/// Tiles `c` can reach from `from` in one carried move of `dash_tiles` (MV4: `dash_fits`), with their routes; `from`
/// itself with an empty route. Blocked tiles, exits and unwalkable tiles are never ends.
pub fn carry_tiles(w: &World, from: V2, dash_tiles: i32, blocked: &Cells) -> Vec<(V2, Vec<V2>)> {
    let mut out = vec![(from, Vec::new())];
    if dash_tiles <= 0 {
        return out;
    }
    for y in 0..crate::grid::GRID_H {
        for x in 0..crate::grid::GRID_W {
            let cell = V2::new(x, y);
            if cell == from || cell.dist(from) > dash_tiles || !w.grid.is_walkable(cell) || w.grid.is_exit(cell) || blocked.contains(&cell) {
                continue;
            }
            let goals: Cells = [cell].into_iter().collect();
            let route = path::find_path(&w.grid, from, &goals, blocked, false, &Hooks::default());
            if !route.is_empty() && dash_fits(&w.grid, from, &route, dash_tiles) {
                out.push((cell, route));
            }
        }
    }
    out
}

// ---- Positions, range and diagonal melee (INPUT1 MR1-MR10, MR16) ---------------------------------------------------

/// INPUT1 MR5, PL6: the planner's refusal of an Engage whose target has no open position tile, and the Rest cause.
pub const FLANK_PROTECTED: &str = "Flank protected";

/// INPUT1 MR1: the position skill `s` dashes to in this battle (rule `round_intent`; the transition reads a Maneuver
/// attack's `dash_tiles` as `flank`); None = it hits from where it stands.
pub fn position_of(w: &World, s: &SkillDef) -> Option<Position> {
    skill::position_by(s, &w.ruleset)
}

/// INPUT1 MR1: how far `c`'s skill `s` dashes to its position (`reach_max`, else the unit's default band max; the
/// transition's `dash_tiles`); None: `s` has no position and hits from where it stands.
pub fn dash_range(w: &World, c: UnitId, s: &SkillDef) -> Option<i32> {
    skill::dash_range_by(s, &w.units[c], &w.ruleset)
}

/// INPUT1 MR4: the band `c`'s skill `s` hits in: (1, 1) for a skill with a position (it hits from the tile next to its
/// target), else its reach band (K1-K2).
pub fn hit_band(w: &World, c: UnitId, s: &SkillDef) -> (i32, i32) {
    if position_of(w, s).is_some() {
        (1, 1)
    } else {
        skill::reach_by(s, &w.units[c], &w.ruleset)
    }
}

/// INPUT1 MR10: whether the band test of `c`'s skill `s` (None: the riposte counter) counts a diagonal neighbour as
/// distance 1: under `maneuver` a Maneuver attack with a position or a melee band, under `all` every melee attack, every
/// skill with a position and the counter; never under `none` (classic) nor for a ranged band.
pub fn diagonal(w: &World, c: UnitId, s: Option<&SkillDef>) -> bool {
    let melee_attack = |s: &SkillDef| {
        let band = hit_band(w, c, s);
        skill::kind(s) == Kind::Attack && !skill::is_area(s) && band.0 <= 1 && band.1 <= 2
    };
    match w.ruleset.melee_diagonal() {
        MeleeDiagonal::None => false,
        MeleeDiagonal::Maneuver => s.is_some_and(|s| skill_intent(s) == "maneuver" && melee_attack(s)),
        MeleeDiagonal::All => s.is_none_or(melee_attack),
    }
}

/// INPUT1 MR10: the distance a band test reads from `a` to `b`: Manhattan, except that a diagonal neighbour (one step on
/// each axis) is 1 when `diag`.
pub fn band_dist(diag: bool, a: V2, b: V2) -> i32 {
    let d = b - a;
    if diag && d.x.abs() == 1 && d.y.abs() == 1 {
        1
    } else {
        a.dist(b)
    }
}

/// INPUT1 MR10 through one helper: the distance `c`'s skill `s` (None: the riposte counter) reads from `a` to `b`.
pub fn melee_dist(w: &World, c: UnitId, s: Option<&SkillDef>, a: V2, b: V2) -> i32 {
    band_dist(diagonal(w, c, s), a, b)
}

/// INPUT1 MR2-MR3: the open position tiles of a position around a target, each tier in MR4's rank order once
/// `position_tiles` ranks them (here in ring order).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Positions {
    pub preferred: Vec<V2>,
    pub fallback: Vec<V2>,
}

impl Positions {
    /// No open tile in either tier: the flank is protected (MR5, MR8 c).
    pub fn protected(&self) -> bool {
        self.preferred.is_empty() && self.fallback.is_empty()
    }

    /// Whether `cell` is a position tile of either tier (MR6).
    pub fn contains(&self, cell: V2) -> bool {
        self.preferred.contains(&cell) || self.fallback.contains(&cell)
    }
}

/// The 8 neighbour offsets around a circle (45 degrees a step).
const RING: [V2; 8] = [V2 { x: 1, y: 0 }, V2 { x: 1, y: 1 }, V2 { x: 0, y: 1 }, V2 { x: -1, y: 1 }, V2 { x: -1, y: 0 }, V2 { x: -1, y: -1 }, V2 { x: 0, y: -1 }, V2 { x: 1, y: -1 }];

/// MR2: the number of 45-degree steps (0-4) between the offsets `a` and `b` (both neighbour offsets).
fn ring_steps(a: V2, b: V2) -> usize {
    let at = |v: V2| RING.iter().position(|r| *r == v).unwrap_or(0);
    let d = at(a).abs_diff(at(b));
    d.min(8 - d)
}

/// INPUT1 MR2-MR3: the open tiles of `position` around `t` (on its tile now, facing as MV1 reads it; a target with no
/// facing faces from its tile toward `from`, where `attacker` stands or will start). `behind` prefers the neighbour
/// straight behind (k 0), `flank` the two beside it (k 2); the fallback of both is the two diagonals between (k 1); the
/// front (k 3, 4) is never a position. A tile is open on the grid, walkable, not an exit, with no unit but `attacker`
/// on it and not in `blocked` (other units and, while planning, the earlier members' claims); without diagonal melee
/// for the skill (`diag` false, MR10) a diagonal neighbour of `t` is never a position.
pub fn positions(w: &World, t: UnitId, attacker: UnitId, position: Position, from: V2, diag: bool, blocked: &Cells) -> Positions {
    let mut out = Positions::default();
    if !w.has(t) {
        return out;
    }
    let tp = w.pos(t);
    let mut f = facing(w, t);
    if f == ZERO {
        f = sign(from - tp);
    }
    if f == ZERO {
        f = V2::new(1, 0);
    }
    let back = V2::new(-f.x, -f.y);
    for off in RING {
        let n = tp + off;
        let open = w.grid.is_valid(n) && w.grid.is_walkable(n) && !w.grid.is_exit(n) && w.occupant(n).is_none_or(|o| o == attacker) && !blocked.contains(&n);
        if !open || (!diag && off.x != 0 && off.y != 0) {
            continue;
        }
        match (ring_steps(off, back), position) {
            (0, Position::Behind) | (2, Position::Flank) => out.preferred.push(n),
            (1, _) => out.fallback.push(n),
            _ => {}
        }
    }
    out
}

/// INPUT1 MR4: the open position tiles of `c`'s skill `s` on `t` that `c` reaches from `from` within `budget` move cost
/// in one block (`dash_fits`; `from` itself at cost 0; only `from` when `rooted`), with their routes: the preferred
/// tier in range ranked by PL4, else the fallback tier in range ranked by fewer path tiles, then PL4. Also returns every
/// open position tile (range aside).
#[allow(clippy::too_many_arguments)]
pub fn position_tiles(w: &World, c: UnitId, s: &SkillDef, t: UnitId, from: V2, budget: i32, rooted: bool, blocked: &Cells) -> (Positions, Vec<(V2, Vec<V2>)>) {
    let Some(position) = position_of(w, s) else { return (Positions::default(), Vec::new()) };
    let open = positions(w, t, c, position, from, diag_of(w, c, s), blocked);
    let reach = |cell: V2| -> Option<Vec<V2>> {
        if cell == from {
            return Some(Vec::new());
        }
        if rooted || cell.dist(from) > budget {
            return None;
        }
        let goals: Cells = [cell].into_iter().collect();
        let route = path::find_path(&w.grid, from, &goals, blocked, false, &Hooks::default());
        (!route.is_empty() && dash_fits(&w.grid, from, &route, budget)).then_some(route)
    };
    let tp = w.pos(t);
    let mut tiles: Vec<(V2, Vec<V2>)> = open.preferred.iter().filter_map(|&cell| reach(cell).map(|r| (cell, r))).collect();
    if !tiles.is_empty() {
        rank_tiles(w, c, tp, w.units[c].attack_range > 1, Aim::Toward, &mut tiles);
        return (open, tiles);
    }
    tiles = open.fallback.iter().filter_map(|&cell| reach(cell).map(|r| (cell, r))).collect();
    rank_tiles(w, c, tp, w.units[c].attack_range > 1, Aim::Toward, &mut tiles);
    // "nearest": fewer path tiles first (a stable sort keeps PL4's order inside each length)
    tiles.sort_by_key(|(_, route)| route.len());
    (open, tiles)
}

/// MR10 for a positioned skill: its hit from a diagonal neighbour counts (`diagonal`).
fn diag_of(w: &World, c: UnitId, s: &SkillDef) -> bool {
    diagonal(w, c, Some(s))
}

/// INPUT1 MR5: `attacker`'s skill `s` on `t` finds no open position tile (`from`: where it stands or will start).
pub fn flank_protected(w: &World, t: UnitId, attacker: UnitId, s: &SkillDef, from: V2, blocked: &Cells) -> bool {
    position_of(w, s).is_some_and(|p| positions(w, t, attacker, p, from, diag_of(w, attacker, s), blocked).protected())
}

/// INPUT1 MR14 (AI table row 4): `c`'s Maneuver attack `s` reaches `t` from `from`: with a position, an open position
/// tile within its range (MR4); without one, `t` in its band (MR10).
pub fn engage_reach(w: &World, c: UnitId, s: &SkillDef, t: UnitId, from: V2, blocked: &Cells) -> bool {
    match dash_range(w, c, s) {
        Some(n) => !position_tiles(w, c, s, t, from, n, w.units[c].has_status("root"), blocked).1.is_empty(),
        None => {
            let band = hit_band(w, c, s);
            let d = melee_dist(w, c, Some(s), from, w.pos(t));
            d >= band.0 && d <= band.1
        }
    }
}

/// MV8: the retreat tiles of skill `s` from `from` after its hit on `t`: carried-move ends other than `from` within the
/// skill's range (INPUT1 MR14: `dash_range`, else its `dash_tiles`), ranked by PL4 away from `t` (best first).
pub fn retreat_tiles(w: &World, c: UnitId, s: &SkillDef, t: UnitId, from: V2, blocked: &Cells) -> Vec<(V2, Vec<V2>)> {
    let tp = if w.has(t) { w.pos(t) } else { from };
    let n = dash_range(w, c, s).unwrap_or(s.dash_tiles);
    let mut out: Vec<(V2, Vec<V2>)> = carry_tiles(w, from, n, blocked).into_iter().filter(|(cell, _)| *cell != from).collect();
    rank_tiles(w, c, tp, w.units[c].attack_range > 1, Aim::Away, &mut out);
    out
}

/// The tile a stored carried move should end on now: `stored` when it is still among `tiles` (free, reachable, meeting
/// the condition), else the best of `tiles` (MV6's re-pick); None when there is none.
pub fn pick(stored: V2, tiles: &[(V2, Vec<V2>)]) -> Option<(V2, Vec<V2>)> {
    if stored != NO_CELL {
        if let Some(t) = tiles.iter().find(|(cell, _)| *cell == stored) {
            return Some(t.clone());
        }
    }
    tiles.first().cloned()
}

// ---- The round summary (RULESET1 12; RULESET1-U slice 5) -----------------------------------------------------------

/// RULESET1 12, the battle screen's round summary: what the round's landed actions did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RoundSummary {
    /// Breaks by stagger (P7, P13, P14), and how many of them came in a windup.
    pub breaks: i32,
    pub windup_breaks: i32,
    /// Stagger hits that landed on a unit in windup (the "of" of "windups broken of hit").
    pub windup_hits: i32,
    /// Hits that were flanks (MV3).
    pub flanks: i32,
    /// Party-side commands after each member's first one (follow-ups: FU2; moves, Rest and lock changes not counted).
    pub follow_ups: i32,
    /// The AP the party's landed Rest blocks give at round end (RS2).
    pub rest_ap: i32,
}

/// `RoundSummary` of the round's actions `sched` (read before the next round start resets the Rest counts).
pub fn round_summary(w: &World, sched: &[crate::resolver::ScheduledAction]) -> RoundSummary {
    use crate::command::CmdKind;
    let mut out = RoundSummary::default();
    let mut acted: Vec<(UnitId, i32)> = Vec::new();
    for a in sched.iter().filter(|a| a.landed) {
        for st in &a.command.staggers {
            out.windup_hits += st.windup as i32;
            if st.broke.is_some() {
                out.breaks += 1;
                out.windup_breaks += st.windup as i32;
            }
        }
        out.flanks += a.command.hits.iter().filter(|(_, flank)| *flank).count() as i32;
        let counts = !matches!(a.command.kind, CmdKind::Walk { .. } | CmdKind::Rest { .. } | CmdKind::Target { .. } | CmdKind::StatusBlock { .. });
        if counts && w.allies.contains(&a.unit) {
            match acted.iter_mut().find(|x| x.0 == a.unit) {
                Some(x) => x.1 += 1,
                None => acted.push((a.unit, 1)),
            }
        }
    }
    out.follow_ups = acted.iter().map(|x| (x.1 - 1).max(0)).sum();
    out.rest_ap = w.allies.iter().map(|c| rest_gain(&w.ruleset.numbers, w.round_states.get(c).map_or(0, |r| r.rest_blocks))).sum();
    out
}

/// RULESET1 12: a round summary in words: "1 break (1 of 2 hits into a wind-up), 1 flank, 3 follow-ups, Rest +1 AP";
/// "no breaks" leads when nothing broke (the battle screen's round-over message and the log's `[Round]` line).
pub fn summary_text(sum: &RoundSummary) -> String {
    let plural = |n: i32, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
    let mut lead = if sum.breaks == 0 { "no breaks".to_string() } else { plural(sum.breaks, "break", "breaks") };
    if sum.windup_hits > 0 {
        lead += &format!(" ({} of {} into a wind-up)", sum.windup_breaks, plural(sum.windup_hits, "hit", "hits"));
    }
    let mut parts = vec![lead];
    if sum.flanks > 0 {
        parts.push(plural(sum.flanks, "flank", "flanks"));
    }
    if sum.follow_ups > 0 {
        parts.push(plural(sum.follow_ups, "follow-up", "follow-ups"));
    }
    if sum.rest_ap > 0 {
        parts.push(format!("Rest +{} AP", sum.rest_ap));
    }
    parts.join(", ")
}

/// The log's line closing a round under the round intent (decision "RULESET1-U slice 5 answers"): `[Round] <round>:
/// <summary_text>`.
pub fn summary_line(round: i32, sum: &RoundSummary) -> String {
    format!("[Round] {round}: {}", summary_text(sum))
}
