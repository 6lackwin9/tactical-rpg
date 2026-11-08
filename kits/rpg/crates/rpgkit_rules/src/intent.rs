//! INTENT2-K1a (A4.2, `docs/sessions/A4_EFFECTS.md` section 2, rule IR21, FB-19, FB-57(3)): the one derivation of a
//! skill's allowed round intents from its baseline data (decision "Skills hold baseline data": allowed intents are
//! derived, not authored, so the row's `round_intent` is not read). Up to 2 intents, the first the primary (IR15).
//!
//! Neutral slice: nothing calls it yet; the planner, checks and screens still read `ruleset::kind_intent` and the
//! row's `round_intent` until the row-aware planner (`A4.2-K1b`, W2-4) switches them over.
//!
//! The precedence (first match wins):
//!
//! 1. engine kinds (`MOVE`, `REST`, `TARGET`, `STATUS`): neutral;
//! 2. `DEFEND`, `FOCUS`: Stand (`guard`), a `hazard` on a DEFEND included;
//! 3. `DISENGAGE`: Maneuver;
//! 4. `SUPPORT`: Support, plus Stand when it may target its user (`target_mode` "" = self or a friend); a self-only
//!    one (`SELF`) is Stand alone (Support targets another friend), an other-only one (`ALLY`) Support alone;
//! 5. `ATTACK` with `dash_tiles` or a `position`: Maneuver;
//! 6. `ATTACK` with an IR21 area shape: `RING` Stand; `ARC`, `LINE`, `CONE` without a `hazard`, and `BURST`, `PLUS`
//!    with `reach_max > 1` and no `hazard`: Field and Assault; any other shaped one Field;
//! 7. `ATTACK` with a `hazard` and no IR21 shape: Field;
//! 8. any other `ATTACK`: Assault.
//!
//! The legacy `SWEEP` (kept for the frozen kit, A3.2-S1) is not an IR21 shape: it derives by steps 7-8 until the
//! SWEEP -> ARC/RING data migration (W2-4).

use crate::ruleset::{self, Ruleset, FIELD_INTENT, NEUTRAL};
use aegis_core::json::{self, Dict};
use rpgkit_data::SkillDef;

/// The intent ids IR21 derives (Stand keeps the id `guard`; `field` is new with INTENT2).
pub const ASSAULT: &str = "assault";
pub const STAND: &str = "guard";
pub const SUPPORT: &str = "support";
pub const MANEUVER: &str = "maneuver";
pub const FIELD: &str = "field";

/// The area shapes IR21 reads (A3.1's six; `SWEEP` is not one of them).
pub const IR21_SHAPES: [&str; 6] = ["PLUS", "BURST", "LINE", "CONE", "RING", "ARC"];

/// The baseline fields of a skill IR21 reads, from a catalog row (`of_row`) or a loaded skill (`of_def`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IntentInput {
    pub kind: String,
    /// The `area` shape ("" or "NONE" = unshaped).
    pub area: String,
    /// Whether it places a ground `hazard`.
    pub hazard: bool,
    pub dash_tiles: i32,
    /// Whether it sets a `position` (flank, behind).
    pub position: bool,
    pub reach_max: i32,
    /// "" or `ALLY_OR_SELF` (self or a friend; the writer drops the schema default), `ALLY` (another friend only) or `SELF` (its user only).
    pub target_mode: String,
}

impl IntentInput {
    /// From a (normalized or raw) catalog skill row.
    pub fn of_row(e: &Dict) -> Self {
        let hazard = e.get("hazard").is_some_and(|h| h.as_object().is_some_and(|o| !o.is_empty()));
        IntentInput {
            kind: json::s(e, "kind"),
            area: json::s(e, "area"),
            hazard,
            dash_tiles: json::i(e, "dash_tiles", 0) as i32,
            position: !json::s(e, "position").is_empty(),
            reach_max: json::i(e, "reach_max", 0) as i32,
            target_mode: json::s(e, "target_mode"),
        }
    }

    /// From a loaded skill.
    pub fn of_def(s: &SkillDef) -> Self {
        IntentInput {
            kind: s.kind.clone(),
            area: s.area.clone(),
            hazard: s.hazard.is_some(),
            dash_tiles: s.dash_tiles,
            position: s.position.is_some(),
            reach_max: s.reach_max,
            target_mode: s.target_mode.clone(),
        }
    }
}

/// IR21: the skill's allowed intents (1 or 2, the primary first; `[neutral]` for the engine kinds, empty for an
/// unknown kind).
pub fn derive(s: &IntentInput) -> Vec<&'static str> {
    match s.kind.as_str() {
        "MOVE" | "REST" | "TARGET" | "STATUS" => vec![NEUTRAL],
        "DEFEND" | "FOCUS" => vec![STAND],
        "DISENGAGE" => vec![MANEUVER],
        "SUPPORT" => match s.target_mode.as_str() {
            "SELF" => vec![STAND],
            "ALLY" => vec![SUPPORT],
            _ => vec![SUPPORT, STAND],
        },
        "ATTACK" => attack(s),
        _ => Vec::new(),
    }
}

fn attack(s: &IntentInput) -> Vec<&'static str> {
    if s.dash_tiles > 0 || s.position {
        return vec![MANEUVER];
    }
    let shape = s.area.as_str();
    if IR21_SHAPES.contains(&shape) {
        let dual = match shape {
            "RING" => return vec![STAND],
            "ARC" | "LINE" | "CONE" => !s.hazard,
            _ => s.reach_max > 1 && !s.hazard,
        };
        return if dual { vec![FIELD, ASSAULT] } else { vec![FIELD] };
    }
    if s.hazard {
        vec![FIELD]
    } else {
        vec![ASSAULT]
    }
}

/// Rule-gated (FB-106): IR21's derivation where the battle plays the Field intent (rule `field_intent`, which needs
/// `round_intent`: a row listing it alone is refused), else the legacy single intent (the row's `round_intent`, else
/// its kind's: RULESET1_INTENT IN4), so classic and a `round_intent`-only row read as today.
pub fn allowed(rules: &Ruleset, s: &IntentInput, authored: &str) -> Vec<&'static str> {
    if rules.has(FIELD_INTENT) {
        return derive(s);
    }
    let legacy = if authored.is_empty() { ruleset::kind_intent(&s.kind) } else { authored };
    [ASSAULT, STAND, SUPPORT, MANEUVER, FIELD, ruleset::CHANNEL, NEUTRAL].into_iter().filter(|i| *i == legacy).collect()
}

/// Whether the derived list allows `intent` (the `row_allows` of the row-aware checks, W2-4).
pub fn row_allows(intents: &[&str], intent: &str) -> bool {
    intents.contains(&intent)
}
