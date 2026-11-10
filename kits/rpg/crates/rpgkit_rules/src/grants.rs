//! U4.2-S1 (docs/sessions/U4_PASSIVES.md, FB-25 to FB-27, FB-30; engine-only, no Godot original): the typed grant view
//! of the passive effect grammar. A passive row's, a worn item's or a meter band's raw `effects` read once into five
//! grant kinds: `modifier` (a target with its combine rule), `proficiency`, `grant_skill`, `grant_pool` and `trigger`.
//! The rows are not rewritten: this is a read-time view, and today's unit accumulators stay the evaluators
//! (`Unit::apply_grants`), so every number is unchanged. One raw effect gives at most two grants, in row order (a
//! `forced_move` row gives its immunity, then its tiles); an unknown type gives none.

use crate::unit::{FORESIGHT_RANGE_MAX, STAGGER_TAKEN_MULT_MAX};
use aegis_core::value::{self, to_int, to_str, Obj};
use rpgkit_data::TriggerRow;
use serde_json::Value;

/// A `resist` target's cap per status (FB-26 `sum_capped`).
const RESIST_CAP: i32 = 100;
/// A `forced_move` row's largest `bonus_tiles`.
const FORCED_TILES_MAX: i32 = 2;
/// An `unlock` row's largest `levels`.
const UNLOCK_LEVELS_MAX: i32 = 2;

/// FB-26: how a target folds a new value into what the unit holds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Combine {
    /// Added: `stat`, `ap_regen`, `status_potency`, `spell_reach`, `skill_potency` (pooled per family or group, read as
    /// 1 + sum), `forced_move` tiles, a band's `crit`.
    Sum,
    /// Added, then capped: `resist` per status.
    SumCapped(i32),
    /// Multiplied: `stagger_taken`, over the rows that match an action's length (the battle reads the held list).
    Product,
    /// The largest: `foresight` range (the first source keeps the id), `unlock` levels.
    Max,
    /// Set by any source: the `forced_move` immunities.
    OrFlag,
}

impl Combine {
    /// `v` folded into `held` (whole numbers; `OrFlag` reads non-zero as set).
    pub fn fold_i(self, held: i32, v: i32) -> i32 {
        match self {
            Combine::Sum => held + v,
            Combine::SumCapped(cap) => (held + v).min(cap),
            Combine::Product => held * v,
            Combine::Max => held.max(v),
            Combine::OrFlag => i32::from(held != 0 || v != 0),
        }
    }

    /// `v` folded into `held` (fractions).
    pub fn fold_f(self, held: f64, v: f64) -> f64 {
        match self {
            Combine::Sum => held + v,
            Combine::SumCapped(cap) => (held + v).min(cap as f64),
            Combine::Product => held * v,
            Combine::Max => held.max(v),
            Combine::OrFlag => f64::from(u8::from(held != 0.0 || v != 0.0)),
        }
    }

    /// A flag folded in.
    pub fn fold_flag(self, held: bool, v: bool) -> bool {
        self.fold_i(i32::from(held), i32::from(v)) != 0
    }
}

/// What a `skill_potency` or `unlock` row names: a skill family ("" allowed) or a skill group.
#[derive(Debug, Clone, PartialEq)]
pub enum SkillSet {
    Family(String),
    Group(String),
}

/// A `forced_move` immunity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Immunity {
    Always,
    Guarding,
}

/// What a modifier changes.
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    /// A base or derived stat under its context tag (`always` or the row's last `context` pair as `key:value`); the
    /// row's `percent` counts only on a derived stat id (DERIVE1 4.1).
    Stat { stat: String, tag: String, percent: f64 },
    Resist { status: String },
    ApRegen,
    SkillPotency(SkillSet),
    StatusPotency,
    /// FB-25: no shipped row uses it (U1 moved reach to the skill); kept for the fixture catalog until the row migration.
    SpellReach,
    /// The reveal range (capped at the read).
    Foresight,
    /// The multiplier for stagger from an action of at most `max_blocks` blocks (0 = any).
    StaggerTaken { max_blocks: i32 },
    ForcedTiles,
    ForcedImmune(Immunity),
    /// A meter band's crit chance points (HOOKS1 UM4; no passive or item takes it).
    Crit,
    /// PK1-S1 (3.1): a meter band's percent added to the healing its holder receives (band-only, as `Crit`).
    HealTaken,
}

impl Target {
    /// FB-26: the target's combine rule.
    pub fn combine(&self) -> Combine {
        match self {
            Target::Resist { .. } => Combine::SumCapped(RESIST_CAP),
            Target::Foresight => Combine::Max,
            Target::StaggerTaken { .. } => Combine::Product,
            Target::ForcedImmune(_) => Combine::OrFlag,
            Target::Stat { .. }
            | Target::ApRegen
            | Target::SkillPotency(_)
            | Target::StatusPotency
            | Target::SpellReach
            | Target::ForcedTiles
            | Target::Crit
            | Target::HealTaken => Combine::Sum,
        }
    }
}

/// A modifier grant: its target and value (a whole number for the whole-number targets, held as f64 without loss).
#[derive(Debug, Clone, PartialEq)]
pub struct Modifier {
    pub target: Target,
    pub value: f64,
}

/// A `grant_skill` grant: a skill learned (`skill`), or a family's or group's unlock level lowered (`unlock`, levels
/// 0-2, the largest counts).
#[derive(Debug, Clone, PartialEq)]
pub enum SkillGrant {
    Learn(String),
    Unlock { set: SkillSet, levels: i32 },
}

impl SkillGrant {
    /// FB-26: an unlock keeps the largest level.
    pub const UNLOCK_COMBINE: Combine = Combine::Max;
}

/// One grant of the typed view (U4 `modifier`, `proficiency`, `grant_skill`, `grant_pool`, `trigger`).
#[derive(Debug, Clone, PartialEq)]
pub enum Grant {
    Modifier(Modifier),
    /// `levels` more proficiency in gear family `gear`.
    Proficiency { gear: String, levels: i32 },
    GrantSkill(SkillGrant),
    /// FB-26 (`meter`): the unit meter the battle spawns, its start (-1 = the row's) and floor (None = the row's min).
    GrantPool { meter: String, start: i32, floor: Option<i32> },
    /// A reactive hook; its order among the unit's triggers is its limit key (`source#index`).
    Trigger(TriggerRow),
}

impl Grant {
    /// Whether worn gear applies it: no unlock, foresight, stagger_taken or meter from an item (gear effects are rebuilt
    /// on every sync; RULESET1 P6, HOOKS1 UM1); U3.2-S4: no proficiency either (training is the class's and the drills';
    /// a worn item's `skill` and `stat` are rebuilt with the rest).
    pub fn gear_ok(&self) -> bool {
        !matches!(
            self,
            Grant::GrantSkill(SkillGrant::Unlock { .. })
                | Grant::GrantPool { .. }
                | Grant::Proficiency { .. }
                | Grant::Modifier(Modifier { target: Target::Foresight | Target::StaggerTaken { .. }, .. })
        )
    }

    /// HOOKS1 UM4: whether a meter band applies it as a passive grant (stat, resist, stagger_taken, skill_potency,
    /// status_potency); the band's crit is read apart (`band_crit`).
    pub fn band_ok(&self) -> bool {
        matches!(
            self,
            Grant::Modifier(Modifier { target: Target::Stat { .. } | Target::Resist { .. } | Target::StaggerTaken { .. } | Target::SkillPotency(_) | Target::StatusPotency, .. })
        )
    }

    /// A band's crit points.
    pub fn band_crit(&self) -> Option<f64> {
        match self {
            Grant::Modifier(Modifier { target: Target::Crit, value }) => Some(*value),
            _ => None,
        }
    }

    /// PK1-S1 (3.1): a band's `heal_taken` percent.
    pub fn band_heal_taken(&self) -> Option<f64> {
        match self {
            Grant::Modifier(Modifier { target: Target::HealTaken, value }) => Some(*value),
            _ => None,
        }
    }

    /// A `proficiency` grant's (gear, levels).
    pub fn proficiency(&self) -> Option<(&str, i32)> {
        match self {
            Grant::Proficiency { gear, levels } => Some((gear, *levels)),
            _ => None,
        }
    }
}

fn modifier(target: Target, value: f64) -> Grant {
    Grant::Modifier(Modifier { target, value })
}

/// A row's `group`, else its `family`.
fn skill_set(e: &Obj) -> SkillSet {
    let group = value::string(e, "group", "");
    if group.is_empty() {
        SkillSet::Family(value::string(e, "family", ""))
    } else {
        SkillSet::Group(group)
    }
}

/// The grants of one raw effect row (none for an unknown type).
pub fn read_one(e: &Obj) -> Vec<Grant> {
    let int = |key: &str, default: i32| f64::from(value::int(e, key, default));
    match value::string(e, "type", "").as_str() {
        "stat" => {
            let mut tag = "always".to_string();
            if let Some(ctx) = value::obj(e, "context") {
                for (k, v) in ctx {
                    tag = format!("{}:{}", k, to_str(v));
                }
            }
            vec![modifier(Target::Stat { stat: value::string(e, "stat", ""), tag, percent: value::float(e, "percent", 0.0) }, int("value", 0))]
        }
        "proficiency" => vec![Grant::Proficiency { gear: value::string(e, "gear", ""), levels: value::int(e, "levels", 0) }],
        "resist" => vec![modifier(Target::Resist { status: value::string(e, "status", "") }, int("value", 0))],
        "ap_regen" => vec![modifier(Target::ApRegen, int("value", 0))],
        "skill" => vec![Grant::GrantSkill(SkillGrant::Learn(value::string(e, "skill", "")))],
        "skill_potency" => vec![modifier(Target::SkillPotency(skill_set(e)), value::float(e, "value", 0.0))],
        "status_potency" => vec![modifier(Target::StatusPotency, int("value", 0))],
        "spell_reach" => vec![modifier(Target::SpellReach, int("value", 0))],
        "unlock" => vec![Grant::GrantSkill(SkillGrant::Unlock { set: skill_set(e), levels: value::int(e, "levels", 0).clamp(0, UNLOCK_LEVELS_MAX) })],
        "foresight" => vec![modifier(Target::Foresight, f64::from(value::int(e, "range", 0).clamp(0, FORESIGHT_RANGE_MAX)))],
        "stagger_taken" => {
            vec![modifier(Target::StaggerTaken { max_blocks: value::int(e, "max_blocks", 0).max(0) }, value::float(e, "mult", 1.0).clamp(0.0, STAGGER_TAKEN_MULT_MAX))]
        }
        "forced_move" => {
            let immune = match value::string(e, "immune", "").as_str() {
                "always" => Some(Immunity::Always),
                "guarding" => Some(Immunity::Guarding),
                _ => None,
            };
            let tiles = modifier(Target::ForcedTiles, f64::from(value::int(e, "bonus_tiles", 0).clamp(0, FORCED_TILES_MAX)));
            immune.map(|i| modifier(Target::ForcedImmune(i), 1.0)).into_iter().chain([tiles]).collect()
        }
        "trigger" => vec![Grant::Trigger(TriggerRow::from_obj(e))],
        "meter" => {
            let floor = e.get("floor").filter(|v| v.is_number()).map(|v| to_int(v, 0));
            vec![Grant::GrantPool { meter: value::string(e, "meter", ""), start: value::int(e, "start", -1), floor }]
        }
        "crit" => vec![modifier(Target::Crit, value::float(e, "value", 0.0))],
        rpgkit_data::HEAL_TAKEN => vec![modifier(Target::HealTaken, int("value", 0))],
        _ => Vec::new(),
    }
}

/// The typed view of an effect list, in row order (a non-object entry gives nothing).
pub fn read(effects: &[Value]) -> Vec<Grant> {
    effects.iter().filter_map(Value::as_object).flat_map(read_one).collect()
}
