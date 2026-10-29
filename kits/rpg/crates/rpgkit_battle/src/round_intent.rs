//! The round intent (RULESET1_INTENT 2-6, rule `round_intent`; engine-only, no Godot original): each unit's intent,
//! which persists across rounds and limits its skills; the round target (one a round, the unit's lock); the intent-bound
//! buffs that end on an intent change; Rest paid at round end; battle facing, engagement and flanks; Guard stealth; the
//! carried move of a Maneuver skill and the tile ranking (PL4) its moves and after-moves use. Every function is inert
//! unless the battle's ruleset has `round_intent` (flank bonuses also need `flank`), so classic never reaches it.

use crate::command::{cover_for, dash_fits};
use crate::grid::{V2, NO_CELL, ZERO};
use crate::path::{self, Cells, Hooks};
use crate::skill;
use crate::world::{UnitId, World};
use rpgkit_data::{ItemDef, SkillDef};
use rpgkit_rules::ruleset::{self, Numbers};
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

    /// The display name ("Assault").
    pub fn name(self) -> &'static str {
        match self {
            RoundIntent::Assault => "Assault",
            RoundIntent::Guard => "Guard",
            RoundIntent::Support => "Support",
            RoundIntent::Maneuver => "Maneuver",
            RoundIntent::Channel => "Channel",
        }
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
        RoundIntent::Assault | RoundIntent::Maneuver => format!("{} targets an enemy or a tile.", intent.name()),
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
        return Err(format!("{} is not an intent of this ruleset.", intent.name()));
    }
    if !w.units[c].is_active() {
        return Err(format!("{} is not on the field.", w.units[c].name));
    }
    let old = w.round_states.get(&c).and_then(|r| r.intent);
    let mut lines = Vec::new();
    if old.is_some_and(|o| o != intent) {
        let from = old.unwrap().name();
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
/// round's target, kind, opener, move and Rest count reset (the intent, facing and Manual toggle stay: IN2, MV1, MM1).
/// Returns the gain lines.
pub fn round_start(w: &mut World) -> Vec<String> {
    if !on(w) {
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
        let rs = w.round_states.get_mut(&c).unwrap();
        // TG6 (RULESET1-PL): the planner proposes last round's target; the opener and the kind are picked again
        rs.last_target = rs.target.take();
        rs.maneuver = None;
        rs.maneuver_auto = false;
        rs.opener = None;
        rs.moved = false;
        rs.rest_blocks = 0;
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

/// MV2: the unit `t` is engaged with: its round target (the lock), a foe of `t` on the field within `t`'s melee band.
pub fn engaged_with(w: &World, t: UnitId) -> Option<UnitId> {
    if !on(w) || !w.has(t) {
        return None;
    }
    let e = match w.round_states.get(&t).and_then(|r| r.target) {
        Some(RoundTarget::Unit(e)) => Some(e),
        Some(_) => None,
        None => w.units[t].active_target,
    }?;
    let d = if w.has(e) && w.units[e].is_active() { w.dist(t, e) } else { return None };
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

/// MV5, MV7: the Engage tiles of skill `s` against `t` from `from`: carried-move ends with `t` in the skill's band
/// and outside `t`'s front, ranked by PL4 (best first).
pub fn engage_tiles(w: &World, c: UnitId, s: &SkillDef, t: UnitId, from: V2, blocked: &Cells) -> Vec<(V2, Vec<V2>)> {
    let band = skill::reach_by(s, &w.units[c], &w.ruleset);
    let tp = w.pos(t);
    let mut out: Vec<(V2, Vec<V2>)> = carry_tiles(w, from, s.dash_tiles, blocked)
        .into_iter()
        .filter(|(cell, _)| {
            let d = cell.dist(tp);
            d >= band.0 && d <= band.1 && !in_front(w, t, *cell)
        })
        .collect();
    rank_tiles(w, c, tp, w.units[c].attack_range > 1, Aim::Toward, &mut out);
    out
}

/// MV8: the retreat tiles of skill `s` from `from` after its hit on `t`: carried-move ends other than `from`, ranked by
/// PL4 away from `t` (best first).
pub fn retreat_tiles(w: &World, c: UnitId, s: &SkillDef, t: UnitId, from: V2, blocked: &Cells) -> Vec<(V2, Vec<V2>)> {
    let tp = if w.has(t) { w.pos(t) } else { from };
    let mut out: Vec<(V2, Vec<V2>)> = carry_tiles(w, from, s.dash_tiles, blocked).into_iter().filter(|(cell, _)| *cell != from).collect();
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
