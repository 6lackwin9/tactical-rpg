//! SkillDef behaviour, SkillKinds rules and SkillCatalog queries (scripts/combat/planning/skill_def.gd,
//! skill_catalog.gd, scripts/content/skill_kinds.gd). Skills are referenced by their index in `Catalog::skills`.

use crate::grid::{V2, DIRS};
use rpgkit_data::{Catalog, SkillDef};
use rpgkit_rules::unit::skill_in_group;
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

pub fn is_area(s: &SkillDef) -> bool {
    area(s) != Area::None
}

pub fn is_off_hand(s: &SkillDef) -> bool {
    s.weapon_hand.eq_ignore_ascii_case("off")
}

pub fn deals_damage(s: &SkillDef) -> bool {
    kind(s) != Kind::Attack || s.power_mult > 0.0
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

/// Distance band (min, max) at which `c` can use this attack.
pub fn reach_for(s: &SkillDef, c: &Unit) -> (i32, i32) {
    let a = area(s);
    let extra = if skill_in_group(s, "spell") && a != Area::Line && a != Area::Sweep { c.spell_reach_bonus() } else { 0 };
    if s.reach_max > 0 {
        return (s.reach_min, s.reach_max + extra);
    }
    if is_off_hand(s) || a == Area::Line || a == Area::Sweep {
        return (1, 1);
    }
    if c.attack_range > 1 {
        return (2, 3.max(c.attack_range) + extra);
    }
    (1, c.attack_range + extra)
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
    let mut best = (String::new(), 1.0);
    for row in entries {
        let Some(o) = row.as_object() else { continue };
        let status = aegis_core::value::string(o, "status", "");
        let mult = aegis_core::value::float(o, "power_mult", 1.0);
        if target.has_status(&status) && !skip.contains(&status) && mult > best.1 {
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
