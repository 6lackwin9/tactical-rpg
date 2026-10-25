//! Rulesets (RULESET1 3, docs/specs/RULESET1.md; engine-only, no Godot original): `classic` (the Godot build's rules,
//! the empty rule set, no data row) and `timeline` (the row `timeline` of the catalog kind `rulesets`: its rules from
//! the register below and its numbers). A world names its ruleset in its manifest (`ruleset`, "" = the nearest parent's,
//! else classic); a battle gets it from its caller (`EncounterDef.ruleset`: the world bridge, the Combat Arena, the
//! Dojo, the sim, the host). Every timeline rule reads its flag here first, so classic battles never change.

use aegis_core::value::{self, Obj};
use rpgkit_data::Catalog;

pub const CLASSIC: &str = "classic";
pub const TIMELINE: &str = "timeline";
/// Every ruleset id, in order: the options of a world's `ruleset` (beside "" = inherit) and of a derived-stat row's
/// `rulesets` (the schema lists the same; a test keeps them in agreement).
pub const RULESETS: [&str; 2] = [CLASSIC, TIMELINE];
/// The ids a row of the `rulesets` kind may take (classic has no row).
pub const ROW_IDS: [&str; 1] = [TIMELINE];
/// The catalog kind of the ruleset rows (data/catalog/rulesets.json).
pub const KIND: &str = "rulesets";

/// Rule ids of the register (RULESET1 3.2).
pub const ROUND_INTENT: &str = "round_intent";
pub const ENEMY_MOVES: &str = "enemy_moves";
pub const DECK: &str = "deck";
pub const POISE: &str = "poise";
pub const AP_COST: &str = "ap_cost";
pub const PIERCE: &str = "pierce";
pub const REACH: &str = "reach";
pub const FLANK: &str = "flank";
pub const PLAN_BADGES: &str = "plan_badges";

/// An option is incompatible with classic (on only under its ruleset); an extension is compatible and inert without
/// data, still off in classic so classic cannot drift when data is added, and may be switched on or off one by one
/// (the Combat Arena, the sim).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    Option,
    Extension,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    pub id: &'static str,
    pub kind: RuleKind,
}

/// The rule register (RULESET1 3.2), in its order.
pub const REGISTER: [Rule; 9] = [
    Rule { id: ROUND_INTENT, kind: RuleKind::Option },
    Rule { id: ENEMY_MOVES, kind: RuleKind::Option },
    Rule { id: DECK, kind: RuleKind::Option },
    Rule { id: POISE, kind: RuleKind::Extension },
    Rule { id: AP_COST, kind: RuleKind::Extension },
    Rule { id: PIERCE, kind: RuleKind::Extension },
    Rule { id: REACH, kind: RuleKind::Extension },
    Rule { id: FLANK, kind: RuleKind::Extension },
    Rule { id: PLAN_BADGES, kind: RuleKind::Extension },
];

/// The register entry of `id`.
pub fn rule(id: &str) -> Option<&'static Rule> {
    REGISTER.iter().find(|r| r.id == id)
}

/// The round intents a ruleset row may list (`channel` is reserved for Act 2: RULESET1_INTENT IN1).
pub const INTENTS: [&str; 5] = ["assault", "guard", "support", "maneuver", "channel"];
/// A row's `intents` when it lists none.
pub const DEFAULT_INTENTS: [&str; 4] = ["assault", "guard", "support", "maneuver"];
/// The intent of commands usable in any intent (a plain move, Rest; RULESET1_INTENT IN5).
pub const NEUTRAL: &str = "neutral";

/// RULESET1_INTENT IN4: the intent a skill of `kind` has when its row sets none.
pub fn kind_intent(kind: &str) -> &'static str {
    match kind {
        "ATTACK" => "assault",
        "DEFEND" => "guard",
        "SUPPORT" | "FOCUS" => "support",
        "DISENGAGE" => "maneuver",
        _ => NEUTRAL,
    }
}

/// The intent Act 2's meter spec reserves (IN1): a skill may take it only when a ruleset row lists it.
pub const CHANNEL: &str = "channel";

/// RULESET1_INTENT IN4: the `round_intent` values a skill of `kind` may set (beside "" = `kind_intent`): ATTACK assault
/// or maneuver, DEFEND guard, SUPPORT support or guard (self-only), FOCUS support, DISENGAGE maneuver, the engine kinds
/// neutral; `channel` (any authored kind) is checked on its own.
pub fn kind_intents(kind: &str) -> &'static [&'static str] {
    match kind {
        "ATTACK" => &["assault", "maneuver"],
        "DEFEND" => &["guard"],
        "SUPPORT" => &["support", "guard"],
        "FOCUS" => &["support"],
        "DISENGAGE" => &["maneuver"],
        "MOVE" | "REST" | "TARGET" | "STATUS" => &[NEUTRAL],
        _ => &[],
    }
}

/// RULESET1_INTENT MV3: the condition an attack's `exploit` may name for a flank hit (rule `flank`; not a status, so
/// classic never matches it).
pub const FLANKED: &str = "flanked";

/// The deck's slot counts (RULESET1 9 D1; all 0 when the row sets none).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DeckSlots {
    pub active: i32,
    pub passive: i32,
    pub keystone: i32,
}

/// A ruleset row's numbers (RULESET1_TASKS 1.2); `Default` is the schema defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct Numbers {
    pub intents: Vec<String>,
    /// Skill families every unit may use without a slot (RULESET1 9 D1).
    pub basics: Vec<String>,
    /// The derived-stat row that is Max Poise (RULESET1 5 P1; "" = none).
    pub poise_stat: String,
    pub stagger_windup_mult: f64,
    pub stagger_guard_mult: f64,
    pub stagger_repeat_mult: f64,
    pub flank_stagger_mult: f64,
    pub flank_power_mult: f64,
    pub break_chance_scale: f64,
    pub break_chance_cap: i32,
    pub rest_ap_per_block: f64,
    pub rest_ap_cap: i32,
    pub rest_poise_per_block: i32,
    pub deck_slots: DeckSlots,
}

impl Default for Numbers {
    fn default() -> Self {
        Self {
            intents: DEFAULT_INTENTS.iter().map(|s| s.to_string()).collect(),
            basics: Vec::new(),
            poise_stat: String::new(),
            stagger_windup_mult: 2.0,
            stagger_guard_mult: 0.5,
            stagger_repeat_mult: 0.5,
            flank_stagger_mult: 1.5,
            flank_power_mult: 1.0,
            break_chance_scale: 0.0,
            break_chance_cap: 50,
            rest_ap_per_block: 0.5,
            rest_ap_cap: 2,
            rest_poise_per_block: 5,
            deck_slots: DeckSlots::default(),
        }
    }
}

impl Numbers {
    /// From a ruleset row (missing keys take the defaults).
    pub fn from_row(o: &Obj) -> Self {
        let d = Self::default();
        let slots = value::obj(o, "deck_slots");
        let slot = |k: &str| slots.map_or(0, |s| value::int(s, k, 0));
        Self {
            intents: if o.contains_key("intents") { value::strings(o, "intents") } else { d.intents },
            basics: value::strings(o, "basics"),
            poise_stat: value::string(o, "poise_stat", ""),
            stagger_windup_mult: value::float(o, "stagger_windup_mult", d.stagger_windup_mult),
            stagger_guard_mult: value::float(o, "stagger_guard_mult", d.stagger_guard_mult),
            stagger_repeat_mult: value::float(o, "stagger_repeat_mult", d.stagger_repeat_mult),
            flank_stagger_mult: value::float(o, "flank_stagger_mult", d.flank_stagger_mult),
            flank_power_mult: value::float(o, "flank_power_mult", d.flank_power_mult),
            break_chance_scale: value::float(o, "break_chance_scale", d.break_chance_scale),
            break_chance_cap: value::int(o, "break_chance_cap", d.break_chance_cap),
            rest_ap_per_block: value::float(o, "rest_ap_per_block", d.rest_ap_per_block),
            rest_ap_cap: value::int(o, "rest_ap_cap", d.rest_ap_cap),
            rest_poise_per_block: value::int(o, "rest_poise_per_block", d.rest_poise_per_block),
            deck_slots: DeckSlots { active: slot("active"), passive: slot("passive"), keystone: slot("keystone") },
        }
    }
}

/// The rules a battle plays by: classic (no rules) or a ruleset row's rules and numbers, possibly with single
/// extensions switched on or off by the caller (`adjust`).
#[derive(Debug, Clone, PartialEq)]
pub struct Ruleset {
    /// `classic` or a row id.
    pub id: String,
    pub name: String,
    /// The rules on, in register order.
    pub rules: Vec<&'static str>,
    pub numbers: Numbers,
}

impl Default for Ruleset {
    fn default() -> Self {
        Self::classic()
    }
}

impl Ruleset {
    /// The classic ruleset: no rule on, the default numbers (no rule reads them).
    pub fn classic() -> Self {
        Self { id: CLASSIC.into(), name: "Classic".into(), rules: Vec::new(), numbers: Numbers::default() }
    }

    /// From a (checked) ruleset row: the register's rules it lists, its numbers.
    pub fn from_row(o: &Obj) -> Self {
        let listed = value::strings(o, "rules");
        Self {
            id: value::string(o, "id", TIMELINE),
            name: value::string(o, "name", "Timeline"),
            rules: REGISTER.iter().map(|r| r.id).filter(|id| listed.iter().any(|l| l == id)).collect(),
            numbers: Numbers::from_row(o),
        }
    }

    /// The ruleset `id` of the catalog: "" or classic is classic; another id needs its row in `rulesets`.
    pub fn of(cat: &Catalog, id: &str) -> Result<Self, String> {
        if id.is_empty() || id == CLASSIC {
            return Ok(Self::classic());
        }
        if !RULESETS.contains(&id) {
            return Err(format!("unknown ruleset '{id}' (one of {})", RULESETS.join(", ")));
        }
        cat.rulesets.get(id).map(Self::from_row).ok_or_else(|| format!("ruleset '{id}' has no row in {KIND} (data/catalog/{KIND}.json)"))
    }

    /// The ruleset `id` of the catalog, classic when it cannot be had (the content check reports a world that names a
    /// ruleset without its row).
    pub fn of_or_classic(cat: &Catalog, id: &str) -> Self {
        Self::of(cat, id).unwrap_or_default()
    }

    pub fn is_classic(&self) -> bool {
        self.id == CLASSIC
    }

    /// Whether rule `id` is on.
    pub fn has(&self, id: &str) -> bool {
        self.rules.contains(&id)
    }

    /// Whether the classic stance machinery runs (stances, the switch AP, the matchup, the `stance` override,
    /// `ends_in_stance`, `disrupt_break`): always but under `round_intent` (RULESET1_INTENT IN3).
    pub fn stances(&self) -> bool {
        !self.has(ROUND_INTENT)
    }

    /// Switches single extensions: `spec` is a comma list of `+<id>` (on) and `-<id>` (off), e.g. "-pierce,+reach".
    /// Options belong to their ruleset and are refused, as are unknown ids.
    pub fn adjust(&mut self, spec: &str) -> Result<(), String> {
        for part in spec.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let (on, id) = match (part.strip_prefix('+'), part.strip_prefix('-')) {
                (Some(id), _) => (true, id),
                (_, Some(id)) => (false, id),
                _ => return Err(format!("rules: '{part}' must be +<rule> or -<rule>")),
            };
            let Some(r) = rule(id) else {
                return Err(format!("rules: unknown rule '{id}' (one of {})", REGISTER.iter().map(|r| r.id).collect::<Vec<_>>().join(", ")));
            };
            if r.kind == RuleKind::Option {
                return Err(format!("rules: '{id}' is an option of its ruleset; only extensions switch one by one"));
            }
            let mut set: Vec<&'static str> = self.rules.clone();
            set.retain(|x| *x != r.id);
            if on {
                set.push(r.id);
            }
            self.rules = REGISTER.iter().map(|x| x.id).filter(|x| set.contains(x)).collect();
        }
        Ok(())
    }
}

/// RULESET1 3.1.1: a world's ruleset id from the manifests of its lineage, nearest first (the world, then its parents
/// nearest first): the first non-empty `ruleset`, else classic.
pub fn world_ruleset<'a>(nearest_first: impl IntoIterator<Item = &'a Obj>) -> String {
    nearest_first.into_iter().map(|m| value::string(m, "ruleset", "")).find(|r| !r.is_empty()).unwrap_or_else(|| CLASSIC.to_string())
}
