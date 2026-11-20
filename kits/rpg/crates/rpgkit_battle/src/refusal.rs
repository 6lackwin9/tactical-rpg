//! Typed planner refusals (PR5-32 `LAYER-UP-EVENTS`, audit 2026-11-10 2.2 item 2): engine-only, no Godot original.
//! The planners keep their reason text (the host's `reason`, the menu's `problem`: the same words as before); a reason
//! a screen words differently also travels in fields, so the screen never reads the kit's text back: the HP cost
//! (HOOKS1 HC1, `command::hp_cost_reason`), a meter gate (PK1-S2a, `meter::gate_reason`) and HITCALC1 (PR7-45) the kind
//! of target a menu cell needs (`TargetNeed`, `round_intent_plan::pick_options`). Any other reason is its text.

use serde_json::{json, Value};

/// One reason a plan step or a menu pick is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// HOOKS1 HC1: the HP left (`hp`) is at or below the skill's HP cost: "HP 9 <= cost 10".
    HpCost { hp: i32, cost: i32 },
    /// PK1-S2a: the meter `meter` (display name `name`) is below the gate's `min`: "Gauge 30 < 50".
    MeterGate { meter: String, name: String, have: i32, min: i32 },
    /// HITCALC1 (PR7-45): the cell needs another kind of target: "Target an ally to use this".
    NeedsTarget(TargetNeed),
    /// Any other reason, as the planner words it.
    Text(String),
}

/// HITCALC1 (PR7-45): the kind of target a skill or item cell needs (VISCLAR1-D's hint, INTENT2-U's rows), so a screen
/// names its control without reading the kit's words back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetNeed {
    /// A tile (a Field row: a tile or an enemy's tile, `or_foe_tile`).
    Tile { or_foe_tile: bool },
    Enemy,
    EnemyOrTile,
    Yourself,
    Ally,
    YourselfOrAlly,
}

impl TargetNeed {
    /// Every kind (the JSON ids' order).
    pub const ALL: [TargetNeed; 7] = [TargetNeed::Tile { or_foe_tile: false }, TargetNeed::Tile { or_foe_tile: true }, TargetNeed::Enemy, TargetNeed::EnemyOrTile, TargetNeed::Yourself, TargetNeed::Ally, TargetNeed::YourselfOrAlly];

    /// The words after "Target": "a tile", "an ally", "yourself or an ally".
    pub fn words(self) -> &'static str {
        match self {
            TargetNeed::Tile { or_foe_tile: false } => "a tile",
            TargetNeed::Tile { or_foe_tile: true } => "a tile (or an enemy's tile)",
            TargetNeed::Enemy => "an enemy",
            TargetNeed::EnemyOrTile => "an enemy or a tile",
            TargetNeed::Yourself => "yourself",
            TargetNeed::Ally => "an ally",
            TargetNeed::YourselfOrAlly => "yourself or an ally",
        }
    }

    /// The JSON id ("tile", "field_tile", "enemy", "enemy_or_tile", "self", "ally", "self_or_ally").
    pub fn id(self) -> &'static str {
        match self {
            TargetNeed::Tile { or_foe_tile: false } => "tile",
            TargetNeed::Tile { or_foe_tile: true } => "field_tile",
            TargetNeed::Enemy => "enemy",
            TargetNeed::EnemyOrTile => "enemy_or_tile",
            TargetNeed::Yourself => "self",
            TargetNeed::Ally => "ally",
            TargetNeed::YourselfOrAlly => "self_or_ally",
        }
    }

    /// `id` read back.
    pub fn parse(id: &str) -> Option<TargetNeed> {
        TargetNeed::ALL.into_iter().find(|n| n.id() == id)
    }

    /// The unit itself is a target it takes.
    pub fn takes_self(self) -> bool {
        matches!(self, TargetNeed::Yourself | TargetNeed::YourselfOrAlly)
    }

    /// An ally is a target it takes.
    pub fn takes_ally(self) -> bool {
        matches!(self, TargetNeed::Ally | TargetNeed::YourselfOrAlly)
    }
}

impl std::fmt::Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reason::HpCost { hp, cost } => write!(f, "HP {hp} <= cost {cost}"),
            Reason::MeterGate { name, have, min, .. } => write!(f, "{name} {have} < {min}"),
            Reason::NeedsTarget(n) => write!(f, "Target {} to use this", n.words()),
            Reason::Text(t) => f.write_str(t),
        }
    }
}

impl Reason {
    /// The reason as JSON fields for the command host: {"kind": "hp_cost", "hp", "cost"}, {"kind": "meter_gate",
    /// "meter", "name", "have", "min"}, {"kind": "needs_target", "need", "text"} (HITCALC1) or {"kind": "text", "text"}.
    pub fn to_json(&self) -> Value {
        match self {
            Reason::HpCost { hp, cost } => json!({"kind": "hp_cost", "hp": hp, "cost": cost}),
            Reason::MeterGate { meter, name, have, min } => json!({"kind": "meter_gate", "meter": meter, "name": name, "have": have, "min": min}),
            Reason::NeedsTarget(n) => json!({"kind": "needs_target", "need": n.id(), "text": self.to_string()}),
            Reason::Text(t) => json!({"kind": "text", "text": t}),
        }
    }

    /// `to_json` read back (None for anything else).
    pub fn from_json(v: &Value) -> Option<Reason> {
        let n = |k: &str| v[k].as_i64().map(|x| x as i32);
        let s = |k: &str| v[k].as_str().map(str::to_string);
        Some(match v["kind"].as_str()? {
            "hp_cost" => Reason::HpCost { hp: n("hp")?, cost: n("cost")? },
            "meter_gate" => Reason::MeterGate { meter: s("meter")?, name: s("name")?, have: n("have")?, min: n("min")? },
            "needs_target" => Reason::NeedsTarget(TargetNeed::parse(v["need"].as_str()?)?),
            "text" => Reason::Text(s("text")?),
            _ => return None,
        })
    }
}

/// The planner's text of `reasons`: each reason's, joined with `REASONS_JOIN` ("" = none).
pub fn text(reasons: &[Reason]) -> String {
    reasons.iter().map(Reason::to_string).collect::<Vec<_>>().join(crate::round_intent_plan::REASONS_JOIN)
}
