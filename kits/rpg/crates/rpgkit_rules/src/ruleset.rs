//! Rulesets (RULESET1 3, docs/specs/RULESET1.md; engine-only, no Godot original): every ruleset is a row of the catalog
//! kind `rulesets` (its rules from the register below and its numbers; CLN-44: the ids are open, `classic` included,
//! decision "INPUT1-M rule choices and the classic ruleset"). `classic` is the default id (the legacy rules: no rule on,
//! `melee_diagonal` none); a game without its row plays the built-in classic values (`Ruleset::classic`) with a load
//! note. A world names its ruleset in its manifest (`ruleset`, "" = the nearest parent's, else classic); a battle gets
//! it from its caller (`EncounterDef.ruleset`: the world bridge, the Combat Arena, the Dojo, the sim, the host). Every
//! extension reads its flag here first, so classic battles never change. CLN-45: the legacy-vs-new behaviours the round
//! intent rule used to switch as a bundle (Rest AP timing, the chase, root stopping it, the Switch Target step, the
//! stances) are row fields with values (`BY_RULES_FIELDS`); empty, each reads as the round intent rule's choice when the
//! row lists it, else the legacy one.

use aegis_core::value::{self, Obj};
use rpgkit_data::Catalog;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

/// The default ruleset id (a world that names none, a catalog encounter, the Dojo's start).
pub const CLASSIC: &str = "classic";
pub const TIMELINE: &str = "timeline";
/// The ruleset a world without its own plays by (CLN-46: screens dim a chip at it, with no switches).
pub const DEFAULT: &str = CLASSIC;
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
/// HOOKS1 4.1: the combat hooks' rule ids (all extensions, off in classic): forced moves, ground hazards, scheduled
/// arena hazards, events and triggers (riders, held guard), unit meters, the HP cost of skills, the guard bypass, the
/// AI tactics and target preferences, and the execute match of an `exploit` entry with `hp_below`.
pub const FORCED_MOVE: &str = "forced_move";
pub const GROUND_HAZARDS: &str = "ground_hazards";
pub const ARENA_HAZARDS: &str = "arena_hazards";
pub const TRIGGERS: &str = "triggers";
pub const UNIT_METERS: &str = "unit_meters";
pub const HP_COST: &str = "hp_cost";
pub const IGNORE_GUARD: &str = "ignore_guard";
pub const AI_TACTICS: &str = "ai_tactics";
pub const EXECUTE: &str = "execute";

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

/// The rule register (RULESET1 3.2, then HOOKS1 4.1), in its order.
pub const REGISTER: [Rule; 18] = [
    Rule { id: ROUND_INTENT, kind: RuleKind::Option },
    Rule { id: ENEMY_MOVES, kind: RuleKind::Option },
    Rule { id: DECK, kind: RuleKind::Option },
    Rule { id: POISE, kind: RuleKind::Extension },
    Rule { id: AP_COST, kind: RuleKind::Extension },
    Rule { id: PIERCE, kind: RuleKind::Extension },
    Rule { id: REACH, kind: RuleKind::Extension },
    Rule { id: FLANK, kind: RuleKind::Extension },
    Rule { id: PLAN_BADGES, kind: RuleKind::Extension },
    Rule { id: FORCED_MOVE, kind: RuleKind::Extension },
    Rule { id: GROUND_HAZARDS, kind: RuleKind::Extension },
    Rule { id: ARENA_HAZARDS, kind: RuleKind::Extension },
    Rule { id: TRIGGERS, kind: RuleKind::Extension },
    Rule { id: UNIT_METERS, kind: RuleKind::Extension },
    Rule { id: HP_COST, kind: RuleKind::Extension },
    Rule { id: IGNORE_GUARD, kind: RuleKind::Extension },
    Rule { id: AI_TACTICS, kind: RuleKind::Extension },
    Rule { id: EXECUTE, kind: RuleKind::Extension },
];

/// The register entry of `id`.
pub fn rule(id: &str) -> Option<&'static Rule> {
    REGISTER.iter().find(|r| r.id == id)
}

/// The round intents a ruleset row may list (`channel` is reserved for Act 2: RULESET1_INTENT IN1).
pub const INTENTS: [&str; 5] = ["assault", "guard", "support", "maneuver", "channel"];
/// INPUT1 MR10: which melee hits count a diagonal neighbour (one step on each axis) as distance 1: none, the Maneuver
/// skills (the default of a row), or every melee attack, every skill with a position and the riposte counter. The
/// built-in classic values: none. CLN-45: read as the row sets it, whatever the rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeleeDiagonal {
    None,
    Maneuver,
    All,
}

impl MeleeDiagonal {
    /// The field values, in order.
    pub const IDS: [&'static str; 3] = ["none", "maneuver", "all"];

    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "none" => Some(MeleeDiagonal::None),
            "maneuver" => Some(MeleeDiagonal::Maneuver),
            "all" => Some(MeleeDiagonal::All),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            MeleeDiagonal::None => "none",
            MeleeDiagonal::Maneuver => "maneuver",
            MeleeDiagonal::All => "all",
        }
    }
}

/// INPUT1 MR9: what a Maneuver whose flank is protected pays of the AP it paid at its start: all of it, half (rounded
/// down), at most a fixed number, or nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlankCost {
    Full,
    Half,
    Fixed(i32),
    None,
}

impl FlankCost {
    /// The `flank_protected_cost` values, in order.
    pub const IDS: [&'static str; 4] = ["full", "half", "fixed", "none"];
    /// The `flank_protected_ap` default.
    pub const FIXED_AP: i32 = 1;

    /// The AP a protected flank pays of `cost` (the rest is refunded).
    pub fn pays(self, cost: i32) -> i32 {
        let cost = cost.max(0);
        match self {
            FlankCost::Full => cost,
            FlankCost::Half => cost / 2,
            FlankCost::Fixed(ap) => cost.min(ap.max(0)),
            FlankCost::None => 0,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            FlankCost::Full => "full",
            FlankCost::Half => "half",
            FlankCost::Fixed(_) => "fixed",
            FlankCost::None => "none",
        }
    }

    /// From the row's `flank_protected_cost` and `flank_protected_ap` (an unknown value reads as `full`).
    pub fn from_ids(cost: &str, ap: i32) -> Self {
        match cost {
            "half" => FlankCost::Half,
            "fixed" => FlankCost::Fixed(ap),
            "none" => FlankCost::None,
            _ => FlankCost::Full,
        }
    }
}

/// CLN-45: when a landed Rest block returns its AP: as it lands (`on_use`, the legacy rule) or at the round's end by
/// `rest_ap_per_block` up to `rest_ap_cap` (`round_end`, RULESET1_INTENT RS2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestAp {
    OnUse,
    RoundEnd,
}

impl RestAp {
    /// The `rest_ap` values, in order (beside "" = by the rules).
    pub const IDS: [&'static str; 2] = ["on_use", "round_end"];

    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "on_use" => Some(RestAp::OnUse),
            "round_end" => Some(RestAp::RoundEnd),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            RestAp::OnUse => "on_use",
            RestAp::RoundEnd => "round_end",
        }
    }
}

/// CLN-45 (AS1, INPUT1 MR16): which planned skills follow their target up to 1 tile when it moves: an attack on a unit
/// unless it comes after a move to a chosen tile, Support never (`classic`, the legacy rule), or every attack on a unit
/// and every Support skill on an ally (`every_skill`). Area skills never chase (their cells are fixed when planned).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chase {
    Classic,
    EverySkill,
}

impl Chase {
    /// The `chase` values, in order (beside "" = by the rules).
    pub const IDS: [&'static str; 2] = ["classic", "every_skill"];

    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "classic" => Some(Chase::Classic),
            "every_skill" => Some(Chase::EverySkill),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Chase::Classic => "classic",
            Chase::EverySkill => "every_skill",
        }
    }
}

/// CLN-45: the values of an on/off ruleset field (`root_stops_chase`, `switch_target_step`, `stances`; beside "" = by
/// the rules).
pub const SWITCH_IDS: [&str; 2] = ["on", "off"];
/// CLN-45: the row fields whose empty value ("" or absent) reads as the round intent rule's choice when the row lists
/// `round_intent`, else the legacy (classic) one (`Numbers::by_rules`).
pub const BY_RULES_FIELDS: [&str; 5] = ["rest_ap", "chase", "root_stops_chase", "switch_target_step", "stances"];

/// The values a field of `BY_RULES_FIELDS` may take (beside "").
pub fn by_rules_values(key: &str) -> &'static [&'static str] {
    match key {
        "rest_ap" => &RestAp::IDS,
        "chase" => &Chase::IDS,
        _ => &SWITCH_IDS,
    }
}

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
    /// INPUT1 MR10 (`melee_diagonal`; row default `maneuver`, classic `none`; CLN-45: read as it is, whatever the
    /// rules).
    pub melee_diagonal: MeleeDiagonal,
    /// INPUT1 MR9 (`flank_protected_cost`, `flank_protected_ap`; default `full`).
    pub flank_protected: FlankCost,
    /// CLN-45: when Rest returns its AP (`rest_ap`).
    pub rest_ap: RestAp,
    /// CLN-45: which skills chase (`chase`).
    pub chase: Chase,
    /// CLN-45 (INPUT1 MR16): whether root stops a chase (`root_stops_chase`).
    pub root_stops_chase: bool,
    /// CLN-45 (RULESET1_INTENT TG3): whether an attack on another unit than the locked one inserts a Switch Target step
    /// first (`switch_target_step`; off: the attack targets its unit and takes the lock).
    pub switch_target_step: bool,
    /// CLN-45 (RULESET1_INTENT IN3): whether the classic stance machinery runs (`stances`).
    pub stances: bool,
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
            melee_diagonal: MeleeDiagonal::Maneuver,
            flank_protected: FlankCost::Full,
            rest_ap: RestAp::OnUse,
            chase: Chase::Classic,
            root_stops_chase: false,
            switch_target_step: true,
            stances: true,
        }
    }
}

impl Numbers {
    /// CLN-45: the defaults of a row with (`round_intent` true) or without the round intent rule: the empty values of
    /// `BY_RULES_FIELDS` read as the round intent's choices (Rest pays at round end, every skill chases, root stops a
    /// chase, no Switch Target step, no stances) or the legacy ones; a raw row without `melee_diagonal` reads `maneuver`
    /// or `none` the same way (a loaded row has the schema's `maneuver`).
    pub fn by_rules(round_intent: bool) -> Self {
        let d = Self::default();
        if !round_intent {
            return d;
        }
        Self { rest_ap: RestAp::RoundEnd, chase: Chase::EverySkill, root_stops_chase: true, switch_target_step: false, stances: false, ..d }
    }

    /// From a ruleset row (missing keys take the defaults; CLN-45: the switches' by `by_rules`).
    pub fn from_row(o: &Obj) -> Self {
        let round_intent = value::strings(o, "rules").iter().any(|r| r == ROUND_INTENT);
        let d = Self::by_rules(round_intent);
        let raw_diagonal = if o.contains_key("melee_diagonal") { Self::default().melee_diagonal } else if round_intent { MeleeDiagonal::Maneuver } else { MeleeDiagonal::None };
        let switch = |k: &str, by_rules: bool| match value::string(o, k, "").as_str() {
            "on" => true,
            "off" => false,
            _ => by_rules,
        };
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
            melee_diagonal: MeleeDiagonal::parse(&value::string(o, "melee_diagonal", "")).unwrap_or(raw_diagonal),
            flank_protected: FlankCost::from_ids(&value::string(o, "flank_protected_cost", ""), value::int(o, "flank_protected_ap", FlankCost::FIXED_AP)),
            rest_ap: RestAp::parse(&value::string(o, "rest_ap", "")).unwrap_or(d.rest_ap),
            chase: Chase::parse(&value::string(o, "chase", "")).unwrap_or(d.chase),
            root_stops_chase: switch("root_stops_chase", d.root_stops_chase),
            switch_target_step: switch("switch_target_step", d.switch_target_step),
            stances: switch("stances", d.stances),
        }
    }

    /// RULESET1-G2 (`sim feel tune=`): sets one tuning number by its row key (the `stagger_*`, `flank_*`,
    /// `break_chance_*` and `rest_*` numbers); an integer number takes a whole value. The caller checks the schema's
    /// range. Err: not a tuning number, or a fraction for an integer one.
    pub fn set(&mut self, key: &str, v: f64) -> Result<(), String> {
        let whole = |v: f64| if v.fract() == 0.0 { Ok(v as i32) } else { Err(format!("{key} takes a whole number (got {v})")) };
        match key {
            "stagger_windup_mult" => self.stagger_windup_mult = v,
            "stagger_guard_mult" => self.stagger_guard_mult = v,
            "stagger_repeat_mult" => self.stagger_repeat_mult = v,
            "flank_stagger_mult" => self.flank_stagger_mult = v,
            "flank_power_mult" => self.flank_power_mult = v,
            "break_chance_scale" => self.break_chance_scale = v,
            "break_chance_cap" => self.break_chance_cap = whole(v)?,
            "rest_ap_per_block" => self.rest_ap_per_block = v,
            "rest_ap_cap" => self.rest_ap_cap = whole(v)?,
            "rest_poise_per_block" => self.rest_poise_per_block = whole(v)?,
            _ => return Err(format!("'{key}' is not a ruleset tuning number")),
        }
        Ok(())
    }
}

/// The rules a battle plays by: a ruleset row's rules and numbers, possibly with single
/// extensions switched on or off by the caller (`adjust`).
#[derive(Debug, Clone, PartialEq)]
pub struct Ruleset {
    /// The row id (`classic` for the built-in fallback).
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
    /// The built-in classic values: no rule on, the default numbers (no rule reads them), no diagonal melee, the legacy
    /// switches (CLN-45: Rest pays on use, the classic chase, Switch Target steps, stances on). CLN-44:
    /// only the fallback of a game without its `classic` row, and tests; battles read the row (`of`).
    pub fn classic() -> Self {
        Self { id: CLASSIC.into(), name: "Classic".into(), rules: Vec::new(), numbers: Numbers { melee_diagonal: MeleeDiagonal::None, ..Numbers::default() } }
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

    /// The ruleset `id` of the catalog ("" is classic): its row in `rulesets`. CLN-44: a missing `classic` row falls back
    /// to the built-in classic values with a load note (`fallback_note`); any other id without its row is refused.
    pub fn of(cat: &Catalog, id: &str) -> Result<Self, String> {
        let id = if id.is_empty() { CLASSIC } else { id };
        if let Some(row) = cat.rulesets.get(id) {
            return Ok(Self::from_row(row));
        }
        if id == CLASSIC {
            note_fallback(id);
            return Ok(Self::classic());
        }
        let known: Vec<&str> = cat.rulesets.keys().map(String::as_str).collect();
        Err(format!("ruleset '{id}' has no row in {KIND} (data/catalog/{KIND}.json; rows: {})", if known.is_empty() { "none".to_string() } else { known.join(", ") }))
    }

    /// The ruleset `id` of the catalog, the built-in classic values with a load note when it has no row (the content
    /// check reports a world that names a ruleset without its row).
    pub fn of_or_classic(cat: &Catalog, id: &str) -> Self {
        Self::of(cat, id).unwrap_or_else(|_| {
            note_fallback(id);
            Self::classic()
        })
    }

    /// The ruleset rows' ids, in file order (the options of a world's `ruleset` and of a row's `rulesets`).
    pub fn ids(cat: &Catalog) -> Vec<String> {
        cat.rulesets.keys().cloned().collect()
    }

    /// INPUT1 MR10: the diagonal melee in effect: the row's value (CLN-45: no longer read as `none` without
    /// `round_intent`; the classic row sets `none`).
    pub fn melee_diagonal(&self) -> MeleeDiagonal {
        self.numbers.melee_diagonal
    }

    /// CLN-45 (RULESET1_INTENT RS2): when Rest returns its AP.
    pub fn rest_ap(&self) -> RestAp {
        self.numbers.rest_ap
    }

    /// CLN-45: whether a landed Rest block pays its AP at the round's end (`rest_ap` round_end).
    pub fn rest_at_round_end(&self) -> bool {
        self.numbers.rest_ap == RestAp::RoundEnd
    }

    /// CLN-45 (AS1, INPUT1 MR16): which planned skills chase.
    pub fn chase(&self) -> Chase {
        self.numbers.chase
    }

    /// CLN-45 (INPUT1 MR16): whether root stops a chase.
    pub fn root_stops_chase(&self) -> bool {
        self.numbers.root_stops_chase
    }

    /// CLN-45 (RULESET1_INTENT TG3): whether an attack on another unit than the locked one inserts Switch Target.
    pub fn switch_target_step(&self) -> bool {
        self.numbers.switch_target_step
    }

    pub fn is_classic(&self) -> bool {
        self.id == CLASSIC
    }

    /// Whether rule `id` is on.
    pub fn has(&self, id: &str) -> bool {
        self.rules.contains(&id)
    }

    /// Whether the classic stance machinery runs (stances, the switch AP, the matchup, the `stance` override,
    /// `ends_in_stance`, `disrupt_break`; RULESET1_INTENT IN3). CLN-45: the row's `stances` (by the rules: off under
    /// `round_intent`, else on).
    pub fn stances(&self) -> bool {
        self.numbers.stances
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

/// CLN-44: the load note of ruleset `id` played as the built-in classic values for want of its row.
pub fn fallback_note(id: &str) -> String {
    let id = if id.is_empty() { CLASSIC } else { id };
    format!("ruleset '{id}' has no row in {KIND} (data/catalog/{KIND}.json): the built-in classic values apply")
}

/// Prints the load note of `id` once per process (stderr: a debug convenience, decision "Saves and data edits").
fn note_fallback(id: &str) {
    static NOTED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let first = NOTED.get_or_init(|| Mutex::new(HashSet::new())).lock().map(|mut n| n.insert(id.to_string())).unwrap_or(false);
    if first {
        eprintln!("note: {}", fallback_note(id));
    }
}

/// RULESET1 3.1.1: a world's ruleset id from the manifests of its lineage, nearest first (the world, then its parents
/// nearest first): the first non-empty `ruleset`, else classic.
pub fn world_ruleset<'a>(nearest_first: impl IntoIterator<Item = &'a Obj>) -> String {
    nearest_first.into_iter().map(|m| value::string(m, "ruleset", "")).find(|r| !r.is_empty()).unwrap_or_else(|| CLASSIC.to_string())
}
