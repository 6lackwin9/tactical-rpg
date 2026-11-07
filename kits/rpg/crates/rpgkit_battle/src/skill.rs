//! SkillDef behaviour, SkillKinds rules and SkillCatalog queries (scripts/combat/planning/skill_def.gd,
//! skill_catalog.gd, scripts/content/skill_kinds.gd). Skills are referenced by their index in `Catalog::skills`.

use crate::grid::{V2, DIRS};
use rpgkit_data::{Catalog, Position, SkillDef};
use rpgkit_rules::ruleset::{self, Ruleset};
use rpgkit_rules::unit::{skill_in_group, BASE_AP, REACH_WEAPON_REACH};
use rpgkit_rules::{Stance, Unit};

pub type SkillId = usize;

pub const SUPPORT_REACH: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Attack,
    Defend,
    Focus,
    Move,
    Rest,
    Target,
    Status,
    Support,
    Disengage,
}

impl Kind {
    pub const ALL: [Kind; 9] = [Kind::Attack, Kind::Defend, Kind::Focus, Kind::Move, Kind::Rest, Kind::Target, Kind::Status, Kind::Support, Kind::Disengage];

    pub fn parse(s: &str) -> Kind {
        match s {
            "DEFEND" => Kind::Defend,
            "FOCUS" => Kind::Focus,
            "MOVE" => Kind::Move,
            "REST" => Kind::Rest,
            "TARGET" => Kind::Target,
            "STATUS" => Kind::Status,
            "SUPPORT" => Kind::Support,
            "DISENGAGE" => Kind::Disengage,
            _ => Kind::Attack,
        }
    }

    /// SkillKinds stance rule.
    fn stance_rule(self) -> &'static str {
        match self {
            Kind::Attack => "OFFENSIVE",
            Kind::Defend => "DEFENSIVE",
            Kind::Focus | Kind::Rest => "TACTICAL",
            Kind::Support => "KEEP_UNLESS_SET",
            _ => "KEEP",
        }
    }

    /// SkillKinds.plans_on_self: kinds cast on SELF that a player plans, plus SUPPORT.
    pub fn plans_on_self(self) -> bool {
        matches!(self, Kind::Defend | Kind::Focus | Kind::Rest | Kind::Disengage | Kind::Support)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Area {
    None,
    Plus,
    Burst,
    Line,
    Sweep,
}

pub fn kind(s: &SkillDef) -> Kind {
    Kind::parse(&s.kind)
}

pub fn area(s: &SkillDef) -> Area {
    match s.area.as_str() {
        "PLUS" => Area::Plus,
        "BURST" => Area::Burst,
        "LINE" => Area::Line,
        "SWEEP" => Area::Sweep,
        _ => Area::None,
    }
}

fn stance_index(s: &str) -> Option<Stance> {
    match s {
        "NEUTRAL" => Some(Stance::Neutral),
        "OFFENSIVE" => Some(Stance::Offensive),
        "DEFENSIVE" => Some(Stance::Defensive),
        "TACTICAL" => Some(Stance::Tactical),
        _ => None,
    }
}

/// The row's stance override (`stance` field; AUTO = none).
pub fn stance_override(s: &SkillDef) -> Option<Stance> {
    if s.stance == "AUTO" {
        None
    } else {
        stance_index(&s.stance)
    }
}

/// Flow move end stance (`ends_in_stance`; "" = none).
pub fn ends_in_stance(s: &SkillDef) -> Option<Stance> {
    stance_index(&s.ends_in_stance)
}

/// The stance this skill puts the unit in.
pub fn stance(s: &SkillDef) -> Stance {
    if let Some(o) = stance_override(s) {
        return o;
    }
    match kind(s).stance_rule() {
        "OFFENSIVE" => Stance::Offensive,
        "DEFENSIVE" => Stance::Defensive,
        "TACTICAL" => Stance::Tactical,
        _ => Stance::Neutral,
    }
}

pub fn keeps_stance(s: &SkillDef) -> bool {
    let rule = kind(s).stance_rule();
    rule == "KEEP" || (rule == "KEEP_UNLESS_SET" && stance_override(s).is_none())
}

pub fn stance_during(s: &SkillDef, current: Stance) -> Stance {
    if keeps_stance(s) {
        current
    } else {
        stance(s)
    }
}

pub fn stance_after(s: &SkillDef, current: Stance) -> Stance {
    ends_in_stance(s).unwrap_or_else(|| stance_during(s, current))
}

/// Automatic stance-switch AP: 1 when the skill changes stance (Rest and stance keepers never pay).
pub fn switch_cost(s: &SkillDef, current: Stance) -> i32 {
    if kind(s) == Kind::Rest || keeps_stance(s) {
        0
    } else if current == stance(s) {
        0
    } else {
        1
    }
}

/// AP = level (Move: per tile, 1; Rest / Target / Status: free), shifted by the unit's modifier, clamped 1..5.
pub fn ap_cost(s: &SkillDef, actor: Option<&Unit>) -> i32 {
    let k = kind(s);
    if matches!(k, Kind::Rest | Kind::Target | Kind::Status) {
        return 0;
    }
    let shift = actor.map(|a| a.ap_cost_modifier).unwrap_or(0);
    let base = if k == Kind::Move { 1 } else { s.level.clamp(1, 5) };
    (base + shift).clamp(1, 5)
}

/// RULESET1 T10: the AP of `s` under `rules`. With the extension `ap_cost` on and the row's `ap_cost` set (>= 0), an
/// authored kind costs `ap_cost` + the unit's modifier, clamped 0..BASE_AP; Move, Rest, Target and Status keep their
/// rules; otherwise (and always in classic) `ap_cost` above. Who pays at all is `World::pays_ap` (T11).
pub fn ap_cost_by(s: &SkillDef, actor: Option<&Unit>, rules: &Ruleset) -> i32 {
    let k = kind(s);
    if rules.has(ruleset::AP_COST) && s.ap_cost >= 0 && !matches!(k, Kind::Move | Kind::Rest | Kind::Target | Kind::Status) {
        let shift = actor.map(|a| a.ap_cost_modifier).unwrap_or(0);
        return (s.ap_cost + shift).clamp(0, BASE_AP);
    }
    ap_cost(s, actor)
}

/// An area attack, or a hazard placement (HOOKS1 GH3), both aimed at a tile.
pub fn is_area(s: &SkillDef) -> bool {
    area(s) != Area::None || is_placement(s)
}

/// HOOKS1 GH3: an ATTACK with no power and a `hazard` is a placement: it aims at a tile like an area opener, deals no
/// damage and rolls no status (the AI's attack lists skip it: it deals no damage and counts as an area). With the rule
/// `ground_hazards` off the battle's catalog has no skill hazards (`World::ignore_hazards_when_off`, 4.1), so this is
/// false there.
pub fn is_placement(s: &SkillDef) -> bool {
    kind(s) == Kind::Attack && s.power_mult <= 0.0 && s.hazard.is_some() && !is_item(s)
}

pub fn is_off_hand(s: &SkillDef) -> bool {
    s.weapon_hand.eq_ignore_ascii_case("off")
}

pub fn deals_damage(s: &SkillDef) -> bool {
    kind(s) != Kind::Attack || s.power_mult > 0.0
}

/// S1.2-S4 (S1 section 3, FB-48): a Guard skill's guard fraction, for every round of its guard: its own `mitigation` (0 =
/// no guard, as riposte's third rank), or for a row without the key the ruleset's `guard_common_mitigation` (1-round)
/// or `guard_multi_round_mitigation` (`guard_rounds` 2 or more). A row the content loader resolved always has the key
/// (the schema default fills it), so the ruleset numbers reach only rows built without it.
pub fn guard_mitigation(s: &SkillDef, n: &ruleset::Numbers) -> f64 {
    if s.raw.contains_key("mitigation") {
        s.mitigation
    } else if s.guard_rounds >= 2 {
        n.guard_multi_round_mitigation
    } else {
        n.guard_common_mitigation
    }
}

pub fn heals(s: &SkillDef) -> bool {
    kind(s) == Kind::Support && (s.heal_base > 0 || s.heal_mult > 0.0)
}

pub fn cleanses(s: &SkillDef) -> bool {
    kind(s) == Kind::Support && !s.cleanse_statuses.is_empty()
}

/// Debuffs this skill removes, in status table order.
pub fn cleanse_list(s: &SkillDef) -> Vec<&'static str> {
    if kind(s) != Kind::Support {
        return Vec::new();
    }
    let all = s.cleanse_statuses.iter().any(|x| x == "all");
    rpgkit_rules::status::debuffs().into_iter().filter(|d| all || s.cleanse_statuses.iter().any(|x| x == d)).collect()
}

/// ItemCommand.skill_for: the synthetic SUPPORT skill a battle item rides on the timeline (keeps the stance, never
/// offered as a skill). Sessions add one per pouch item id to their catalog copy (Session::add_item_skills).
pub fn item_skill(item: &rpgkit_data::ItemDef) -> SkillDef {
    let o = serde_json::json!({"id": item_skill_id(&item.id), "name": item.name, "kind": "SUPPORT", "level": 1, "blocks": item.use_blocks.max(1),
        "target_mode": "ALLY_OR_SELF", "system": true, "_item": item.id});
    SkillDef::from_obj(o.as_object().unwrap())
}

pub fn item_skill_id(item_id: &str) -> String {
    format!("item_{item_id}")
}

/// True for an item's synthetic skill.
pub fn is_item(s: &SkillDef) -> bool {
    s.raw.contains_key("_item")
}

pub fn target_mode(s: &SkillDef) -> &str {
    &s.target_mode
}

pub fn targets_ally(s: &SkillDef) -> bool {
    kind(s) == Kind::Support && s.target_mode != "SELF"
}

/// Distance band at which a SUPPORT skill reaches its target.
pub fn support_reach(s: &SkillDef, c: &Unit) -> (i32, i32) {
    if s.reach_max > 0 {
        (s.reach_min, s.reach_max)
    } else {
        (0, SUPPORT_REACH.max(c.attack_range))
    }
}

/// Distance band (min, max) at which `c` can use this attack (the classic rule; a battle calls `reach_by` with its
/// ruleset).
pub fn reach_for(s: &SkillDef, c: &Unit) -> (i32, i32) {
    band(s, c, c.attack_range)
}

/// RULESET1 K2: the band of `reach_for` under `rules`. With the rule `reach`, a melee attack without its own
/// `reach_max` by a unit with a reach weapon uses (1, its melee reach); off-hand jabs, LINE and SWEEP stay (1, 1).
/// Without the rule (classic) it is `reach_for` exactly.
pub fn reach_by(s: &SkillDef, c: &Unit, rules: &Ruleset) -> (i32, i32) {
    band(s, c, melee_band_max(c, rules))
}

/// The attack band with `melee` as the far edge of a plain melee attack. INPUT1 MR4: a skill with a `position` hits from
/// the tile next to its target whatever its reach, so its band is (1, 1) in every ruleset (its `reach_max` is how far
/// it dashes: `dash_range_by`).
fn band(s: &SkillDef, c: &Unit, melee: i32) -> (i32, i32) {
    if s.position.is_some() && kind(s) == Kind::Attack && !is_area(s) {
        return (1, 1);
    }
    let a = area(s);
    let extra = if skill_in_group(s, "spell") && a != Area::Line && a != Area::Sweep { c.spell_reach_bonus() } else { 0 };
    if s.reach_max > 0 {
        return (s.reach_min, s.reach_max + extra);
    }
    if is_off_hand(s) || a == Area::Line || a == Area::Sweep {
        return (1, 1);
    }
    let (lo, hi) = plain_band(c, melee);
    (lo, hi + extra)
}

/// The band of a plain attack (no own reach, no area, no spell bonus): ranged (2, max(3, range)), melee (1, `melee`).
fn plain_band(c: &Unit, melee: i32) -> (i32, i32) {
    if c.attack_range > 1 {
        (2, 3.max(c.attack_range))
    } else {
        (1, melee)
    }
}

/// INPUT1 MR1: the position a single-target Maneuver ATTACK dashes to under `rules` (rule `round_intent` only): its
/// `position`, else, in the transition until the data moves (INPUT1-MA), `flank` for one with `dash_tiles` above 0;
/// None = it hits from where it stands.
pub fn position_by(s: &SkillDef, rules: &Ruleset) -> Option<Position> {
    if !rules.has(ruleset::ROUND_INTENT) || kind(s) != Kind::Attack || is_area(s) || intent_id(s) != "maneuver" {
        return None;
    }
    s.position.or((s.dash_tiles > 0).then_some(Position::Flank))
}

/// INPUT1 MR1: how far a skill with a position (`position_by`) dashes to it: its `reach_max`, else the unit's default
/// band max; the transition's `dash_tiles` for one without a `position` field. None: it hits from where it stands.
pub fn dash_range_by(s: &SkillDef, c: &Unit, rules: &Ruleset) -> Option<i32> {
    position_by(s, rules)?;
    Some(match s.position {
        None => s.dash_tiles,
        Some(_) if s.reach_max > 0 => s.reach_max,
        Some(_) => plain_band(c, melee_band_max(c, rules)).1,
    })
}

/// RULESET1_INTENT IN4: a skill's intent id (its `round_intent`, else its kind's).
fn intent_id(s: &SkillDef) -> &str {
    if s.round_intent.is_empty() {
        ruleset::kind_intent(&s.kind)
    } else {
        &s.round_intent
    }
}

/// RULESET1 K6: the band of `c`'s plain attack under `rules` (`reach_by` of a skill with no reach of its own): the
/// battle screen's reach band and the hover card's "Reach 1-2".
pub fn attack_band(c: &Unit, rules: &Ruleset) -> (i32, i32) {
    plain_band(c, melee_band_max(c, rules))
}

/// RULESET1 K1: whether `c` fights with a reach weapon in this battle (the rule `reach` on, a melee unit, its main
/// weapon's family has the engine rule `reach`).
pub fn has_reach(c: &Unit, rules: &Ruleset) -> bool {
    rules.has(ruleset::REACH) && c.has_reach_weapon()
}

/// The far edge of `c`'s plain melee band: its melee reach with a reach weapon (K1), else `attack_range` as today.
fn melee_band_max(c: &Unit, rules: &Ruleset) -> i32 {
    if has_reach(c, rules) {
        REACH_WEAPON_REACH
    } else {
        c.attack_range
    }
}

/// RULESET1 K1: `c`'s melee reach, the far edge of its melee band (1 to it): 2 with a reach weapon under the rule
/// `reach`, else 1. It also bounds "engaged" (RULESET1_INTENT MV2).
pub fn melee_reach(c: &Unit, rules: &Ruleset) -> i32 {
    if has_reach(c, rules) {
        REACH_WEAPON_REACH
    } else {
        1
    }
}

/// RULESET1_INTENT PL4, AS3: how far `c` strikes from where it stands, for a threat measure (its 2-block move plus
/// this): a ranged unit's band edge (`max(3, range)`), else its melee band's far edge (at least 1; K1 widens it).
pub fn threat_reach(c: &Unit, rules: &Ruleset) -> i32 {
    if c.attack_range > 1 {
        3.max(c.attack_range)
    } else {
        melee_band_max(c, rules).max(1)
    }
}

/// Tiles an area attack aimed at `cell` hits, cast from `from`.
pub fn area_cells(s: &SkillDef, from: V2, cell: V2) -> Vec<V2> {
    match area(s) {
        Area::Plus => vec![cell, cell + V2::new(1, 0), cell + V2::new(-1, 0), cell + V2::new(0, 1), cell + V2::new(0, -1)],
        Area::Burst => {
            let mut out = Vec::new();
            for dx in -1..=1 {
                for dy in -1..=1 {
                    out.push(cell + V2::new(dx, dy));
                }
            }
            out
        }
        Area::Line => {
            let dir = cell - from;
            (1..4).map(|i| from + dir * i).collect()
        }
        Area::Sweep => {
            let q = V2::new((cell.x - from.x).signum(), (cell.y - from.y).signum());
            vec![from + V2::new(q.x, 0), from + q, from + V2::new(0, q.y)]
        }
        Area::None => vec![cell],
    }
}

/// The best exploit entry whose status is ACTIVE on `target`, skipping `skip`: (status, mult); ("", 1.0) when none.
pub fn exploit_match(entries: &[serde_json::Value], target: &Unit, skip: &[String]) -> (String, f64) {
    exploit_match_when(entries, target, skip, false)
}

/// `exploit_match` where the condition `flanked` (RULESET1_INTENT MV3) also matches when `flanked` is true (a flank
/// hit under the rule `flank`); classic always passes false.
pub fn exploit_match_when(entries: &[serde_json::Value], target: &Unit, skip: &[String], flanked: bool) -> (String, f64) {
    exploit_match_with(entries, target, skip, flanked, false)
}

/// HOOKS1 EX1: the label an `exploit` entry `{hp_below, power_mult}` matches under (not a status: a consuming skill
/// removes nothing for it, the planner never marks it used).
pub const EXECUTE: &str = ruleset::EXECUTE;

/// HOOKS1 EX1: the `exploit` key of an execute entry.
pub const EXPLOIT_HP_BELOW: &str = "hp_below";

/// HOOKS1 EX1: the skill's execute entry, `(hp_below, power_mult)` (validation allows one).
pub fn execute_entry(entries: &[serde_json::Value]) -> Option<(i64, f64)> {
    entries.iter().filter_map(|r| r.as_object()).find(|o| o.contains_key(EXPLOIT_HP_BELOW)).map(|o| (aegis_core::value::int(o, EXPLOIT_HP_BELOW, 0) as i64, aegis_core::value::float(o, "power_mult", 1.0)))
}

/// `exploit_match_when` under the battle's rules: with the rule `execute` an entry `{hp_below, power_mult}` also matches
/// when `target`'s HP share is at or below `hp_below` (read before the hit's damage); the best entry wins.
pub fn exploit_match_by(entries: &[serde_json::Value], target: &Unit, skip: &[String], flanked: bool, rules: &Ruleset) -> (String, f64) {
    exploit_match_with(entries, target, skip, flanked, rules.has(ruleset::EXECUTE))
}

/// The best entry: a status ACTIVE on `target` (or `flanked`), or (`execute`) an `hp_below` entry the target's HP share
/// meets; an `hp_below` entry never matches without `execute` (it names no status).
fn exploit_match_with(entries: &[serde_json::Value], target: &Unit, skip: &[String], flanked: bool, execute: bool) -> (String, f64) {
    let mut best = (String::new(), 1.0);
    for row in entries {
        let Some(o) = row.as_object() else { continue };
        let mult = aegis_core::value::float(o, "power_mult", 1.0);
        if o.contains_key(EXPLOIT_HP_BELOW) {
            let pct = aegis_core::value::float(o, EXPLOIT_HP_BELOW, 0.0);
            let share = if target.max_hp > 0 { target.hp as f64 * 100.0 / target.max_hp as f64 } else { 100.0 };
            if execute && share <= pct && mult > best.1 {
                best = (EXECUTE.to_string(), mult);
            }
            continue;
        }
        let status = aegis_core::value::string(o, "status", "");
        let mult = aegis_core::value::float(o, "power_mult", 1.0);
        let present = target.has_status(&status) || (flanked && status == ruleset::FLANKED);
        if present && !skip.contains(&status) && mult > best.1 {
            best = (status, mult);
        }
    }
    best
}

pub fn is_stance_break(s: &SkillDef) -> bool {
    s.disrupt == "STANCE_BREAK"
}

pub fn neighbours(c: V2) -> [V2; 4] {
    [c + DIRS[0], c + DIRS[1], c + DIRS[2], c + DIRS[3]]
}

/// SkillCatalog: queries over the catalog skills in file order.
pub struct Book<'a> {
    pub cat: &'a Catalog,
}

impl<'a> Book<'a> {
    pub fn get(&self, id: SkillId) -> &'a SkillDef {
        &self.cat.skills[id]
    }

    pub fn find(&self, skill_id: &str) -> Option<SkillId> {
        self.cat.skills.iter().position(|s| s.id == skill_id)
    }

    pub fn lowest(&self, k: Kind) -> SkillId {
        self.cat.skills.iter().position(|s| kind(s) == k).expect("catalog has a skill of every engine kind")
    }

    /// Skills of `k` the unit can use with level <= cap, catalog order.
    pub fn available(&self, k: Kind, cap: i32, c: Option<&Unit>) -> Vec<SkillId> {
        self.cat
            .skills
            .iter()
            .enumerate()
            .filter(|(_, s)| kind(s) == k && !is_item(s) && s.level <= cap && c.is_none_or(|u| u.can_use(self.cat, s)))
            .map(|(i, _)| i)
            .collect()
    }

    /// Single-target attacks (no areas; the off-hand jab only for ranged units).
    pub fn single_attacks(&self, c: &Unit, cap: i32) -> Vec<SkillId> {
        self.available(Kind::Attack, cap, Some(c)).into_iter().filter(|&i| !(is_area(self.get(i)) || (is_off_hand(self.get(i)) && c.attack_range <= 1))).collect()
    }

    pub fn area_attacks(&self, cap: i32, c: Option<&Unit>) -> Vec<SkillId> {
        self.available(Kind::Attack, cap, c).into_iter().filter(|&i| is_area(self.get(i))).collect()
    }

    pub fn cleanse_only(&self, cap: i32, c: Option<&Unit>) -> Vec<SkillId> {
        self.available(Kind::Support, cap, c).into_iter().filter(|&i| cleanses(self.get(i)) && !heals(self.get(i))).collect()
    }

    pub fn granting(&self, k: Kind, status: &str, cap: i32, c: Option<&Unit>) -> Vec<SkillId> {
        self.available(k, cap, c).into_iter().filter(|&i| self.get(i).grant_status == status && !heals(self.get(i))).collect()
    }

    pub fn ally_buffs(&self, except: &str, cap: i32, c: Option<&Unit>) -> Vec<SkillId> {
        self.available(Kind::Support, cap, c)
            .into_iter()
            .filter(|&i| {
                let s = self.get(i);
                !s.grant_status.is_empty() && s.grant_status != except && !heals(s)
            })
            .collect()
    }

    pub fn debuff_attacks(&self, cap: i32, c: Option<&Unit>) -> Vec<SkillId> {
        self.available(Kind::Attack, cap, c)
            .into_iter()
            .filter(|&i| {
                let s = self.get(i);
                !deals_damage(s) && !is_area(s) && !s.status_effect.is_empty()
            })
            .collect()
    }
}

/// HOOKS1 UM2: the meter changes skill `s` pays at its command's start (none without the rule unit_meters; items pay
/// none).
pub fn meters_by<'a>(s: &'a SkillDef, rs: &Ruleset) -> &'a [rpgkit_data::MeterDelta] {
    if rs.has(ruleset::UNIT_METERS) && !is_item(s) {
        &s.meters
    } else {
        &[]
    }
}
