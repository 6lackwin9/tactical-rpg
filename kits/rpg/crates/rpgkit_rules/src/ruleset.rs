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

use crate::status;
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

/// BASERULES1-M (docs/specs/BASERULES1_LIBRARY.md 7): a register entry and its metadata. `requires` names the rules
/// without which this one is silently inert (the content check refuses a row that lists it alone: an error);
/// `conflicts` names rules or `field=value` settings it runs with but cuts against (a warning); `bundle` the bundle it
/// belongs to (`BUNDLES`, "" = none); `system` the system it reads in; `tested` the kit tests that fight it
/// (`rpgkit_battle/tests/`, "golden" = the timeline golden). Additive: a rule is off when a row does not list it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    pub id: &'static str,
    pub kind: RuleKind,
    pub requires: &'static [&'static str],
    pub conflicts: &'static [&'static str],
    pub bundle: &'static str,
    pub system: &'static str,
    pub tested: &'static [&'static str],
    /// What it does when on, in a few words (the `aegis rules` report).
    pub does: &'static str,
}

const fn rule_of(id: &'static str, kind: RuleKind, system: &'static str, bundle: &'static str, tested: &'static [&'static str], does: &'static str) -> Rule {
    Rule { id, kind, requires: &[], conflicts: &[], bundle, system, tested, does }
}

/// The rule register (RULESET1 3.2, then HOOKS1 4.1), in its order, with its metadata (BASERULES1-M).
pub const REGISTER: [Rule; 18] = [
    Rule {
        conflicts: &["stances=on", "switch_target_step=on"],
        ..rule_of(ROUND_INTENT, RuleKind::Option, "planning", "intent_core", &["round_intent.rs", "golden"], "one intent and target a round, follow-ups, facing, flanks drawn")
    },
    rule_of(ENEMY_MOVES, RuleKind::Option, "timing", "windup", &["timeline_timing.rs"], "enemies pay no AP and book recovery blocks"),
    rule_of(DECK, RuleKind::Option, "build", "build", &["deck.rs"], "open basics, slots, slottable passives"),
    rule_of(POISE, RuleKind::Extension, "poise", "windup", &["poise.rs", "battle_talk.rs"], "max poise, stagger, breaks, rest poise"),
    rule_of(AP_COST, RuleKind::Extension, "timing", "windup", &["timeline_timing.rs"], "a skill's ap_cost replaces its level as AP"),
    rule_of(PIERCE, RuleKind::Extension, "damage", "", &["poise.rs"], "armor_piercing drops the armour term"),
    rule_of(REACH, RuleKind::Extension, "positioning", "positioning", &["reach.rs"], "melee band 1-2 by the family rule"),
    Rule { requires: &[ROUND_INTENT], ..rule_of(FLANK, RuleKind::Extension, "positioning", "positioning", &["round_intent.rs"], "flanked exploit, flank power, no riposte") },
    rule_of(PLAN_BADGES, RuleKind::Extension, "tells", "tells", &["plan_badges.rs"], "impact badges, intents in tells, foresight"),
    rule_of(FORCED_MOVE, RuleKind::Extension, "hooks", "hooks", &["forced_move.rs"], "push, pull, collision"),
    rule_of(GROUND_HAZARDS, RuleKind::Extension, "hooks", "hooks", &["ground_hazards.rs"], "skill and item hazards on tiles"),
    rule_of(ARENA_HAZARDS, RuleKind::Extension, "hooks", "hooks", &["arena_hazards.rs"], "scheduled hazards"),
    rule_of(TRIGGERS, RuleKind::Extension, "hooks", "hooks", &["triggers.rs", "riders.rs"], "events, riders, held guard, intercept"),
    rule_of(UNIT_METERS, RuleKind::Extension, "hooks", "hooks", &["meters.rs"], "meters, bands, meter costs"),
    rule_of(HP_COST, RuleKind::Extension, "hooks", "hooks", &["riders.rs"], "a skill's HP cost"),
    rule_of(IGNORE_GUARD, RuleKind::Extension, "hooks", "hooks", &["riders.rs", "poise.rs"], "a skill drops guard, keeps cover"),
    rule_of(AI_TACTICS, RuleKind::Extension, "ai", "hooks", &["tactics.rs"], "target preferences and unit tactics"),
    rule_of(EXECUTE, RuleKind::Extension, "hooks", "hooks", &["execute.rs"], "the hp_below exploit"),
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

/// ATTACKLOCK1 (decision "Attack lock at wind-up"): which attacks on a unit commit to their target's tile when their
/// wind-up starts, after any chase or dash, and miss if it has left that tile at impact: none (the built-in classic
/// values and a row that sets nothing), melee bands, or every attack on a unit. Area attacks keep their tiles fixed at
/// planning under every value. Read as the row sets it, whatever the rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttackLock {
    #[default]
    None,
    Melee,
    All,
}

impl AttackLock {
    /// The `attack_lock` values, in order.
    pub const IDS: [&'static str; 3] = ["none", "melee", "all"];

    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "none" => Some(AttackLock::None),
            "melee" => Some(AttackLock::Melee),
            "all" => Some(AttackLock::All),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            AttackLock::None => "none",
            AttackLock::Melee => "melee",
            AttackLock::All => "all",
        }
    }

    /// Whether an attack on a unit locks its tile (`melee`: it hits in a melee band).
    pub fn locks(self, melee: bool) -> bool {
        match self {
            AttackLock::None => false,
            AttackLock::Melee => melee,
            AttackLock::All => true,
        }
    }
}

/// S2.2-S2 (`docs/sessions/S2_POISE.md` 1 and 4, `poise_mode`; rule `poise`): how a hit's stagger lowers poise. `bar`
/// (the default, every shipped row): the continuous bar breaks at 0 (RULESET1 P6-P14). `pips`: the bar is
/// `poise_pips` pips of `T = max(1, roundi(max_poise / poise_pips))` each; a hit of stagger taken `D` removes
/// `clamp(floor(D / T), 0, max_pip_loss_per_hit)` pips and the unit breaks at 0 pips. Row-only (no schema entry).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PoiseMode {
    #[default]
    Bar,
    Pips,
}

impl PoiseMode {
    /// The `poise_mode` values, in order.
    pub const IDS: [&'static str; 2] = ["bar", "pips"];

    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "bar" => Some(PoiseMode::Bar),
            "pips" => Some(PoiseMode::Pips),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            PoiseMode::Bar => "bar",
            PoiseMode::Pips => "pips",
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

/// VISCLAR1-D: the default of `move_after_blocks`.
pub const MOVE_AFTER_BLOCKS: i32 = 3;

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
    /// VISCLAR1-D (`move_after_blocks`): the most move blocks of a Move after a skill; a Move as the opener keeps its two.
    pub move_after_blocks: i32,
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
    /// ATTACKLOCK1: which attacks commit to their target's tile at the wind-up (`attack_lock`; default none).
    pub attack_lock: AttackLock,
    /// S1.2-S3 (`bleed_pct`, S1 section 3): the fraction of max HP a bleed tick deals.
    pub bleed_pct: f64,
    /// S1.2-S3 (`counter_mult`, S1 section 3): riposte's counter damage multiplier.
    pub counter_mult: f64,
    /// S1.2-S3 (`debt_stagger`): the AP debt at a round start that staggers.
    pub debt_stagger: i32,
    /// S1.2-S3 (`debt_stun`): the AP debt at a round start that stuns.
    pub debt_stun: i32,
    /// S2.2-S1 (`poise_pips`, S2 section 3): the pips of a full bar under `poise_mode` pips.
    pub poise_pips: i32,
    /// S2.2-S1 (`max_pip_loss_per_hit`, S2 section 3): the most pips one hit removes under `poise_mode` pips.
    pub max_pip_loss_per_hit: i32,
    /// S2.2-S2 (`poise_mode`): how a hit lowers poise (default `bar`).
    pub poise_mode: PoiseMode,
    /// U1.2-S3b (`know_potency`, U1 "status potency from KNOW", S1 section 2 `attacker_know_potency`): the potency a
    /// point of the attacker's KNOW adds to a skill's status roll; 0 (the default) is the spec's "potency off".
    pub know_potency: f64,
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
            move_after_blocks: MOVE_AFTER_BLOCKS,
            rest_ap: RestAp::OnUse,
            chase: Chase::Classic,
            root_stops_chase: false,
            switch_target_step: true,
            stances: true,
            attack_lock: AttackLock::None,
            bleed_pct: status::BLEED_PCT,
            counter_mult: status::COUNTER_MULT,
            debt_stagger: status::DEBT_STAGGER,
            debt_stun: status::DEBT_STUN,
            poise_pips: 5,
            max_pip_loss_per_hit: 3,
            poise_mode: PoiseMode::Bar,
            know_potency: 0.0,
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
            move_after_blocks: value::int(o, "move_after_blocks", MOVE_AFTER_BLOCKS),
            flank_protected: FlankCost::from_ids(&value::string(o, "flank_protected_cost", ""), value::int(o, "flank_protected_ap", FlankCost::FIXED_AP)),
            rest_ap: RestAp::parse(&value::string(o, "rest_ap", "")).unwrap_or(d.rest_ap),
            chase: Chase::parse(&value::string(o, "chase", "")).unwrap_or(d.chase),
            root_stops_chase: switch("root_stops_chase", d.root_stops_chase),
            switch_target_step: switch("switch_target_step", d.switch_target_step),
            stances: switch("stances", d.stances),
            attack_lock: AttackLock::parse(&value::string(o, "attack_lock", "")).unwrap_or(d.attack_lock),
            bleed_pct: value::float(o, "bleed_pct", d.bleed_pct),
            counter_mult: value::float(o, "counter_mult", d.counter_mult),
            debt_stagger: value::int(o, "debt_stagger", d.debt_stagger),
            debt_stun: value::int(o, "debt_stun", d.debt_stun),
            poise_pips: value::int(o, "poise_pips", d.poise_pips),
            max_pip_loss_per_hit: value::int(o, "max_pip_loss_per_hit", d.max_pip_loss_per_hit),
            poise_mode: PoiseMode::parse(&value::string(o, "poise_mode", "")).unwrap_or(d.poise_mode),
            know_potency: value::float(o, "know_potency", d.know_potency),
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

    /// ATTACKLOCK1: which attacks commit to their target's tile at the wind-up.
    pub fn attack_lock(&self) -> AttackLock {
        self.numbers.attack_lock
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

// BASERULES1-M (docs/specs/BASERULES1_LIBRARY.md 7): the register's metadata beside it. A token names a condition on a
// ruleset: a rule id (listed), `!<rule>` (not listed), `<field>=<value>` (the field reads as the value, by the rules
// when empty) or a bare field key (set: it reads other than the schema default).

/// A row field and what it needs to be read: `needs` tokens that must hold when the field is set (else a warning, or
/// nothing when `refused`: the row checks already refuse it), `untested_with` tokens no test fights it with; `range`
/// (S1.2-S3) the bounds of a row-only number the schema does not hold (a value outside is an error); `values` (S2.2-S2)
/// the ids a row-only value field may take (empty: not one; another id is a content check error).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldRule {
    pub key: &'static str,
    pub needs: &'static [&'static str],
    pub untested_with: &'static [&'static str],
    pub refused: bool,
    pub range: Option<(f64, f64)>,
    pub values: &'static [&'static str],
}

const fn field_rule(key: &'static str, needs: &'static [&'static str]) -> FieldRule {
    FieldRule { key, needs, untested_with: &[], refused: false, range: None, values: &[] }
}

/// The row fields that need a rule or a value to be read (library 2 and 4); a field not listed is read whatever the
/// rules.
pub const FIELD_RULES: [FieldRule; 23] = [
    FieldRule { refused: true, ..field_rule("intents", &[ROUND_INTENT]) },
    field_rule("flank_protected_cost", &[ROUND_INTENT]),
    field_rule("flank_protected_ap", &[ROUND_INTENT]),
    field_rule("move_after_blocks", &[ROUND_INTENT]),
    field_rule("stagger_windup_mult", &[POISE]),
    field_rule("stagger_guard_mult", &[POISE]),
    field_rule("stagger_repeat_mult", &[POISE]),
    field_rule("flank_stagger_mult", &[POISE, FLANK]),
    field_rule("flank_power_mult", &[FLANK]),
    field_rule("break_chance_scale", &[POISE]),
    field_rule("break_chance_cap", &[POISE]),
    field_rule("rest_ap_per_block", &["rest_ap=round_end"]),
    field_rule("rest_ap_cap", &["rest_ap=round_end"]),
    field_rule("rest_poise_per_block", &[POISE]),
    FieldRule { untested_with: &["chase=classic"], ..field_rule("attack_lock", &[]) },
    // S1.2-S3 (S1 section 3; the debt ranges S2 section 3): the status numbers, row-only until a schema entry with a
    // data re-save (like move_after_blocks); read whatever the rules
    FieldRule { range: Some((0.01, 0.25)), ..field_rule("bleed_pct", &[]) },
    FieldRule { range: Some((0.10, 1.50)), ..field_rule("counter_mult", &[]) },
    FieldRule { range: Some((1.0, 10.0)), ..field_rule("debt_stagger", &[]) },
    FieldRule { range: Some((1.0, 10.0)), ..field_rule("debt_stun", &[]) },
    // S2.2-S1/S2 (S2 section 3): the poise pip numbers and the bar-or-pips switch, row-only like the status numbers; the
    // pip numbers are read only under poise_mode pips
    FieldRule { values: &PoiseMode::IDS, ..field_rule("poise_mode", &[POISE]) },
    FieldRule { range: Some((3.0, 8.0)), ..field_rule("poise_pips", &[POISE, "poise_mode=pips"]) },
    FieldRule { range: Some((1.0, 5.0)), ..field_rule("max_pip_loss_per_hit", &[POISE, "poise_mode=pips"]) },
    // U1.2-S3b (U1 parameter table "potency weight (KNOW)"): row-only like the status numbers; read whatever the rules
    FieldRule { range: Some((0.0, 3.0)), ..field_rule("know_potency", &[]) },
];

/// The metadata of row field `key`, if it has any.
pub fn field_rule_of(key: &str) -> Option<&'static FieldRule> {
    FIELD_RULES.iter().find(|f| f.key == key)
}

/// Must ship together (a bundle breaks if split: a warning) or works well (a suggestion, never a warning).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleKind {
    MustShip,
    WorksWell,
}

/// A bundle (library 5): when every `anchor` token holds, a must-ship bundle wants every `holds` token too.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bundle {
    pub id: &'static str,
    pub kind: BundleKind,
    pub anchor: &'static [&'static str],
    pub holds: &'static [&'static str],
    pub note: &'static str,
}

/// The bundles of the rules built so far (library 5; the coming options add theirs).
pub const BUNDLES: [Bundle; 7] = [
    Bundle {
        id: "legacy_core",
        kind: BundleKind::MustShip,
        anchor: &["!round_intent"],
        holds: &["stances=on", "chase=classic", "rest_ap=on_use", "switch_target_step=on", "root_stops_chase=off"],
        note: "the classic row; melee_diagonal reads as set",
    },
    Bundle {
        id: "intent_core",
        kind: BundleKind::MustShip,
        anchor: &[ROUND_INTENT],
        holds: &["rest_ap=round_end", "chase=every_skill", "root_stops_chase=on"],
        note: "the by-rules choices; stances and the Switch Target step are round_intent's conflicts",
    },
    Bundle { id: "build", kind: BundleKind::MustShip, anchor: &[DECK], holds: &["basics", "deck_slots"], note: "the row checks refuse it split" },
    Bundle { id: "windup", kind: BundleKind::WorksWell, anchor: &[], holds: &[ENEMY_MOVES, AP_COST, POISE, "poise_stat"], note: "" },
    Bundle { id: "positioning", kind: BundleKind::WorksWell, anchor: &[], holds: &[FLANK, REACH, "melee_diagonal=maneuver", "attack_lock=melee"], note: "flank needs round_intent" },
    Bundle { id: "tells", kind: BundleKind::WorksWell, anchor: &[], holds: &[PLAN_BADGES, ROUND_INTENT], note: "" },
    Bundle {
        id: "hooks",
        kind: BundleKind::WorksWell,
        anchor: &[],
        holds: &[FORCED_MOVE, GROUND_HAZARDS, ARENA_HAZARDS, TRIGGERS, UNIT_METERS, HP_COST, IGNORE_GUARD, AI_TACTICS, EXECUTE],
        note: "each alone",
    },
];

/// The bundle `id`.
pub fn bundle(id: &str) -> Option<&'static Bundle> {
    BUNDLES.iter().find(|b| b.id == id)
}

/// The rule sets kit tests fight (library 4, `[poise, flank]` left out: it is read only, and flank is inert there); a
/// row listing another set is untested together (a warning).
pub const TESTED_SETS: [&[&str]; 21] = [
    &[],
    &[ROUND_INTENT],
    &[ROUND_INTENT, FLANK],
    &[ROUND_INTENT, AP_COST],
    &[ROUND_INTENT, AP_COST, FLANK],
    &[ROUND_INTENT, DECK],
    &[ROUND_INTENT, REACH],
    &[ROUND_INTENT, PLAN_BADGES],
    &[ROUND_INTENT, POISE, PIERCE],
    &[ROUND_INTENT, ENEMY_MOVES, AP_COST],
    &[ROUND_INTENT, AP_COST, POISE, DECK],
    &[ENEMY_MOVES, AP_COST],
    &[DECK],
    &[DECK, POISE],
    &[POISE],
    &[REACH],
    &[UNIT_METERS],
    &[TRIGGERS],
    &[AI_TACTICS],
    &[GROUND_HAZARDS],
    &[
        ROUND_INTENT, ENEMY_MOVES, DECK, POISE, AP_COST, PIERCE, REACH, FLANK, PLAN_BADGES, FORCED_MOVE, GROUND_HAZARDS, ARENA_HAZARDS, TRIGGERS,
        UNIT_METERS, HP_COST, IGNORE_GUARD, AI_TACTICS, EXECUTE,
    ],
];

/// Whether the rule set `rules` is one the kit tests fight (order does not matter).
pub fn tested_set(rules: &[&str]) -> bool {
    TESTED_SETS.iter().any(|t| t.len() == rules.len() && t.iter().all(|r| rules.contains(r)))
}

/// A numeric `pub const` of `rpgkit_rules/src` that stays code (BASERULES1 4.1): its name, file, system and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Locked {
    pub name: &'static str,
    pub file: &'static str,
    pub system: &'static str,
    pub why: &'static str,
}

const fn locked(name: &'static str, file: &'static str, system: &'static str, why: &'static str) -> Locked {
    Locked { name, file, system, why }
}

const BUDGET_WHY: &str = "gear budget weight of a soft check (warnings only)";
const STATS_WHY: &str = "stat and level rule; opens in the stat sessions (U1, U2)";
const CRIT_WHY: &str = "crit input; becomes a role row (BASERULES1 4)";
const FORMULA_WHY: &str = "formula value; leaves code as a value (BASERULES1 4)";
const CARRY_WHY: &str = "the carry formula; one form, no option yet";
const BOUND_WHY: &str = "a schema field's bound";

/// What stays code, and why (BASERULES1 4.1). A test scans `src` and fails on a numeric `pub const` neither here nor in
/// `PROMOTED`, so the table cannot go stale.
pub const LOCKED: [Locked; 47] = [
    locked("W_POWER", "budget.rs", "gear", BUDGET_WHY),
    locked("W_MITIGATION", "budget.rs", "gear", BUDGET_WHY),
    locked("W_STAT_POINT", "budget.rs", "gear", BUDGET_WHY),
    locked("W_BURDEN", "budget.rs", "gear", BUDGET_WHY),
    locked("W_SKILL_POTENCY", "budget.rs", "gear", BUDGET_WHY),
    locked("W_STATUS_POTENCY", "budget.rs", "gear", BUDGET_WHY),
    locked("W_SPELL_REACH", "budget.rs", "gear", BUDGET_WHY),
    locked("TARGET_BUDGET", "budget.rs", "gear", BUDGET_WHY),
    locked("TOLERANCE", "budget.rs", "gear", BUDGET_WHY),
    locked("UNARMED_VALUE", "budget.rs", "gear", "follows UNARMED_BASE_POWER"),
    locked("BASE_CAPACITY", "carry.rs", "carry", CARRY_WHY),
    locked("HEAVY_RATIO", "carry.rs", "carry", CARRY_WHY),
    locked("OVERLOAD_STEP", "carry.rs", "carry", CARRY_WHY),
    locked("DEFAULT_UNIT_SCALING", "damage.rs", "damage", "unit power scale of an untrained wielder"),
    locked("MIN_POWER", "pipeline.rs", "damage", "attack power floor at rounding point one (U1.2-S2, FB-45)"),
    locked("ARMOR_K", "damage.rs", "damage", FORMULA_WHY),
    locked("CRIT_BASE", "damage.rs", "damage", CRIT_WHY),
    locked("CRIT_PER_PERC", "damage.rs", "damage", CRIT_WHY),
    locked("CRIT_PERC_FLOOR", "damage.rs", "damage", CRIT_WHY),
    locked("CRIT_MULT", "damage.rs", "damage", FORMULA_WHY),
    locked("UNARMED_BASE_POWER", "damage.rs", "damage", "unarmed power input; becomes a role row (BASERULES1 4)"),
    locked("TRAINED_FIST_MIN_LEVEL", "damage.rs", "damage", "the fist proficiency gate"),
    locked("GUARD_CAP", "damage.rs", "damage", FORMULA_WHY),
    locked("POTENCY_MIN", "derived.rs", "stats", BOUND_WHY),
    locked("POTENCY_MAX", "derived.rs", "stats", BOUND_WHY),
    locked("CLASS_BUDGET", "stats.rs", "stats", STATS_WHY),
    locked("SPEND_BASE", "stats.rs", "stats", STATS_WHY),
    locked("FREE_PER_LEVEL", "stats.rs", "stats", STATS_WHY),
    locked("MAX_LEVEL", "stats.rs", "stats", STATS_WHY),
    locked("MAX_CLASS_SLOTS", "stats.rs", "stats", STATS_WHY),
    locked("MAX_EXTRA_SLOTS", "stats.rs", "stats", STATS_WHY),
    locked("TIER_UNLOCK_STEP", "stats.rs", "stats", STATS_WHY),
    locked("MAX_UNLOCK_REDUCTION", "stats.rs", "stats", STATS_WHY),
    locked("ADVANCED_UNLOCK_FLOOR", "stats.rs", "stats", STATS_WHY),
    locked("KNOW_UNLOCK_BASE", "stats.rs", "stats", STATS_WHY),
    locked("KNOW_UNLOCK_STEP", "stats.rs", "stats", STATS_WHY),
    locked("MINION_SKILL_CAP", "stats.rs", "stats", STATS_WHY),
    locked("HP_PER_CONS", "stats.rs", "stats", STATS_WHY),
    locked("RANGE_PERC_THRESHOLD", "stats.rs", "stats", STATS_WHY),
    locked("PROF_MAX_LEVEL", "stats.rs", "stats", STATS_WHY),
    locked("DEFAULT_SCALING", "stats.rs", "stats", STATS_WHY),
    locked("PASSIVE_ONLY_SCALING", "stats.rs", "stats", STATS_WHY),
    locked("BASE_AP", "unit.rs", "timing", "the AP pool every timing rule counts in"),
    locked("GEAR_REACH_CAP", "unit.rs", "positioning", "the most reach gear adds"),
    locked("FORESIGHT_RANGE_MAX", "unit.rs", "tells", BOUND_WHY),
    locked("STAGGER_TAKEN_MULT_MAX", "unit.rs", "poise", BOUND_WHY),
    locked("REACH_WEAPON_REACH", "unit.rs", "positioning", "the reach rule's band"),
];

/// Numeric `pub const`s promoted to data: the default of a ruleset row field (name, field).
pub const PROMOTED: [(&str, &str); 6] = [
    ("FIXED_AP", "flank_protected_ap"),
    ("MOVE_AFTER_BLOCKS", "move_after_blocks"),
    ("BLEED_PCT", "bleed_pct"),
    ("COUNTER_MULT", "counter_mult"),
    ("DEBT_STAGGER", "debt_stagger"),
    ("DEBT_STUN", "debt_stun"),
];

impl Numbers {
    /// BASERULES1-M: a row field as it reads, in its row form (a number, a value id, `on`/`off`, a comma list; deck
    /// slots `active/passive/keystone`); None for a key that is not a ruleset field.
    pub fn field(&self, key: &str) -> Option<String> {
        let switch = |on: bool| if on { "on" } else { "off" }.to_string();
        Some(match key {
            "intents" => self.intents.join(","),
            "basics" => self.basics.join(","),
            "poise_stat" => self.poise_stat.clone(),
            "stagger_windup_mult" => self.stagger_windup_mult.to_string(),
            "stagger_guard_mult" => self.stagger_guard_mult.to_string(),
            "stagger_repeat_mult" => self.stagger_repeat_mult.to_string(),
            "flank_stagger_mult" => self.flank_stagger_mult.to_string(),
            "flank_power_mult" => self.flank_power_mult.to_string(),
            "break_chance_scale" => self.break_chance_scale.to_string(),
            "break_chance_cap" => self.break_chance_cap.to_string(),
            "rest_ap_per_block" => self.rest_ap_per_block.to_string(),
            "rest_ap_cap" => self.rest_ap_cap.to_string(),
            "rest_poise_per_block" => self.rest_poise_per_block.to_string(),
            "deck_slots" => format!("{}/{}/{}", self.deck_slots.active, self.deck_slots.passive, self.deck_slots.keystone),
            "melee_diagonal" => self.melee_diagonal.id().to_string(),
            "flank_protected_cost" => self.flank_protected.id().to_string(),
            "flank_protected_ap" => match self.flank_protected {
                FlankCost::Fixed(ap) => ap,
                _ => FlankCost::FIXED_AP,
            }
            .to_string(),
            "move_after_blocks" => self.move_after_blocks.to_string(),
            "rest_ap" => self.rest_ap.id().to_string(),
            "chase" => self.chase.id().to_string(),
            "root_stops_chase" => switch(self.root_stops_chase),
            "switch_target_step" => switch(self.switch_target_step),
            "stances" => switch(self.stances),
            "attack_lock" => self.attack_lock.id().to_string(),
            "bleed_pct" => self.bleed_pct.to_string(),
            "counter_mult" => self.counter_mult.to_string(),
            "debt_stagger" => self.debt_stagger.to_string(),
            "debt_stun" => self.debt_stun.to_string(),
            "poise_pips" => self.poise_pips.to_string(),
            "max_pip_loss_per_hit" => self.max_pip_loss_per_hit.to_string(),
            "poise_mode" => self.poise_mode.id().to_string(),
            "know_potency" => self.know_potency.to_string(),
            _ => return None,
        })
    }
}

/// BASERULES1-M: what the register's metadata says of a rule set: errors (a rule that would be silently inert; the
/// content check refuses the row) and warnings (a set field nothing reads, a conflict, a split bundle, an untested set).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuleNotes {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Ruleset {
    /// Whether the metadata token `token` holds (see `FIELD_RULES`); an unknown token never holds.
    pub fn holds(&self, token: &str) -> bool {
        if let Some(t) = token.strip_prefix('!') {
            return !self.holds(t);
        }
        if let Some((key, v)) = token.split_once('=') {
            return self.numbers.field(key).is_some_and(|x| x == v);
        }
        if rule(token).is_some() {
            return self.has(token);
        }
        match (self.numbers.field(token), Numbers::default().field(token)) {
            (Some(x), Some(d)) => x != d,
            _ => false,
        }
    }

    /// The register metadata's notes on this ruleset (the content check's, and an `adjust`ed set's for the tools).
    /// Additive: a rule not listed and a field at its default raise nothing.
    pub fn notes(&self) -> RuleNotes {
        let mut n = RuleNotes::default();
        for r in REGISTER.iter().filter(|r| self.has(r.id)) {
            for req in r.requires.iter().filter(|t| !self.holds(t)) {
                n.errors.push(format!("{} needs {req} in Rules: without it the rule does nothing.", r.id));
            }
            for c in r.conflicts.iter().filter(|t| self.holds(t)) {
                n.warnings.push(format!("{} conflicts with {c}: it runs, but the two cut against each other.", r.id));
            }
        }
        for f in FIELD_RULES.iter().filter(|f| self.holds(f.key)) {
            let unmet: Vec<&str> = f.needs.iter().copied().filter(|t| !self.holds(t)).collect();
            if !unmet.is_empty() && !f.refused {
                n.warnings.push(format!("{} is set but read only with {}.", f.key, unmet.join(" and ")));
            }
            for u in f.untested_with.iter().filter(|t| self.holds(t)) {
                n.warnings.push(format!("{} {} is untested with {u}.", f.key, self.numbers.field(f.key).unwrap_or_default()));
            }
            // S1.2-S3: a row-only number outside its range
            if let (Some((lo, hi)), Some(v)) = (f.range, self.numbers.field(f.key).and_then(|v| v.parse::<f64>().ok())) {
                if v < lo || v > hi {
                    n.errors.push(format!("{} {v} is outside {lo}..{hi}.", f.key));
                }
            }
        }
        // S1.2-S3: the AP debt rule tests stun first, so a stagger threshold above the stun one never staggers
        let (stagger, stun) = (self.numbers.debt_stagger, self.numbers.debt_stun);
        if stagger > stun {
            n.errors.push(format!("debt_stagger {stagger} is above debt_stun {stun}: a debt stuns first, so it never staggers."));
        }
        for b in BUNDLES.iter().filter(|b| b.kind == BundleKind::MustShip && b.anchor.iter().all(|t| self.holds(t))) {
            for t in b.holds.iter().filter(|t| !self.holds(t)) {
                n.warnings.push(format!("Bundle {} ships together: it needs {t}.", b.id));
            }
        }
        if !tested_set(&self.rules) {
            n.warnings.push(format!("Rules untested together: {} (no kit test fights this set; aegis rules lists the tested sets).", self.rules.join(", ")));
        }
        n
    }
}
