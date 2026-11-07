//! StatusRegistry + StatusEffects (scripts/combat/status/): the one status table, stacking, inflicting, round
//! roll-over. Statuses inflicted this round become active next round; DEFEND buffs (riposte) are active at once.
//! Since S1.2-S1 the rows are the frozen kit table `statuses.json` (embedded), tagged with S1's behaviour primitives.

use crate::ruleset::Numbers;
use crate::stats::roundi;
use crate::unit::{GrantUnder, Unit};
use aegis_core::rng::GodotRng;
use serde_json::Value;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stacking {
    RefreshMaxDuration,
    ExtendDuration,
    StackPotency,
    None,
}

impl Stacking {
    /// The table's `stacking` word (S1 lifecycle header): `refresh_max`, `extend_duration`, `stack_potency`, `none`.
    pub fn id(self) -> &'static str {
        match self {
            RefreshMaxDuration => "refresh_max",
            ExtendDuration => "extend_duration",
            StackPotency => "stack_potency",
            None => "none",
        }
    }

    pub fn from_id(id: &str) -> Option<Stacking> {
        [RefreshMaxDuration, ExtendDuration, StackPotency, None].into_iter().find(|s| s.id() == id)
    }
}

/// S1 (docs/sessions/S1_STATUS.md section 1): the six composable behaviour primitives a status row is tagged with.
/// Since S1.2-S2 the rules read them where a primitive carries the meaning (the round-end ticks, the block locks, a flat
/// tick's unit); a status's own behaviour (bleed's formula, root, stealth, riposte, steadfast) stays on its id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
    Modifier,
    Tick,
    BlockLock,
    Targeting,
    Reactive,
    RuleFlag,
}

/// The six primitives in S1's order.
pub const PRIMITIVES: [Primitive; 6] = [Primitive::Modifier, Primitive::Tick, Primitive::BlockLock, Primitive::Targeting, Primitive::Reactive, Primitive::RuleFlag];

impl Primitive {
    /// The table's tag word: `modifier`, `tick`, `block_lock`, `targeting`, `reactive`, `rule_flag`.
    pub fn id(self) -> &'static str {
        match self {
            Primitive::Modifier => "modifier",
            Primitive::Tick => "tick",
            Primitive::BlockLock => "block_lock",
            Primitive::Targeting => "targeting",
            Primitive::Reactive => "reactive",
            Primitive::RuleFlag => "rule_flag",
        }
    }

    pub fn from_id(id: &str) -> Option<Primitive> {
        PRIMITIVES.into_iter().find(|p| p.id() == id)
    }
}

/// I1.2-S1 (docs/sessions/I_AI.md section 1 item 5): the rating a `modifier` row changes (the table's `rating` word;
/// S1 section 1 `modifier`): outgoing attack (weaken, empower), Armor Rating (fortify, expose), Move Rate (haste, slow)
/// or situational guard (guard).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rating {
    AttackPower,
    Armor,
    MoveRate,
    Guard,
}

impl Rating {
    /// The table's `rating` word: `attack_power`, `armor`, `move_rate`, `guard`.
    pub fn id(self) -> &'static str {
        match self {
            Rating::AttackPower => "attack_power",
            Rating::Armor => "armor",
            Rating::MoveRate => "move_rate",
            Rating::Guard => "guard",
        }
    }

    pub fn from_id(id: &str) -> Option<Rating> {
        [Rating::AttackPower, Rating::Armor, Rating::MoveRate, Rating::Guard].into_iter().find(|r| r.id() == id)
    }
}

/// I1.2-S1: what a `targeting` row does to enemy targeting (the table's `targeting` word; S1 section 1 `targeting`):
/// `pull` draws enemy targeting toward the unit (taunt), `hide` makes it untargetable by single-target skills
/// (stealth).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Targeting {
    Pull,
    Hide,
}

impl Targeting {
    pub fn id(self) -> &'static str {
        match self {
            Targeting::Pull => "pull",
            Targeting::Hide => "hide",
        }
    }

    pub fn from_id(id: &str) -> Option<Targeting> {
        [Targeting::Pull, Targeting::Hide].into_iter().find(|t| t.id() == id)
    }
}

/// I1.2-S1: a `tick` row heals (a buff: regen) or damages (a debuff: bleed, poison, burn) at its tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tick {
    Heal,
    Damage,
}

#[derive(Debug, Clone, Copy)]
pub struct StatusRow {
    pub id: &'static str,
    /// The table's `kind`: `buff` (true) or `debuff` (false).
    pub buff: bool,
    pub resistable: bool,
    pub default_rounds: i32,
    pub stacking: Stacking,
    pub cap: i32,
    pub display: &'static str,
    /// "support" / "defend" for buffs a skill grants; "" otherwise.
    pub grant: &'static str,
    pub magnitude_min: i32,
    pub magnitude_max: i32,
    /// "pct", "flat" (armor points; poison HP per round) or "pct_max_hp" (S1 lifecycle header `magnitude_unit`).
    pub magnitude_unit: &'static str,
    /// RULESET1_INTENT IN6 (timeline, rule round_intent): an instance lasts only while the round intent it was applied
    /// under stays active (riposte, stealth). Classic never reads it.
    pub intent_bound: bool,
    /// S1.2-S2: the timeline blocks a `block_lock` row locks (stagger 1 and 4, stun 1 and 2; root locks Move, not
    /// blocks); empty on every other row.
    pub lock_blocks: &'static [i32],
    /// I1.2-S1: the rating a `modifier` row changes (`Some` on every `modifier` row, `None` elsewhere).
    pub rating: Option<Rating>,
    /// I1.2-S1: what a `targeting` row does (`Some` on every `targeting` row, `None` elsewhere).
    pub targeting: Option<Targeting>,
    /// S1 behaviour primitives (at least one), see [`Primitive`].
    pub primitives: &'static [Primitive],
}

impl StatusRow {
    /// The table's `kind` word: "buff" or "debuff".
    pub fn kind(&self) -> &'static str {
        if self.buff {
            "buff"
        } else {
            "debuff"
        }
    }

    pub fn has_primitive(&self, p: Primitive) -> bool {
        self.primitives.contains(&p)
    }
}

use Stacking::*;

/// S1.2-S1: the frozen kit status table (`statuses.json` beside this file), embedded at build time so every reader
/// works without a catalog. Games do not override it (the 18th row, `guard`, came with S1.2-S4).
const TABLE_TEXT: &str = include_str!("statuses.json");

static TABLE: LazyLock<Value> = LazyLock::new(|| serde_json::from_str(TABLE_TEXT).expect("statuses.json: valid JSON"));

/// The status rows in table order (Godot's StatusRegistry order; rows 16-17, RULESET1 P12 / intent GD5, engine-only:
/// `steadfast` (cannot be broken) and `stealth` (a Guard buff), which classic data never grants; row 18, S1.2-S4: `guard`,
/// the later rounds of a multi-round Guard skill, queued by the skill, never a grant option). Every field is
/// required; a malformed table fails on first use (the kit test `status_table` loads it).
pub static ROWS: LazyLock<Vec<StatusRow>> = LazyLock::new(|| {
    let rows = TABLE.as_array().expect("statuses.json: an array of rows");
    rows.iter().enumerate().map(|(i, r)| parse_row(i, r)).collect()
});

fn parse_row(i: usize, r: &'static Value) -> StatusRow {
    let field = |k: &str| r.get(k).unwrap_or_else(|| panic!("statuses.json row {i}: missing `{k}`"));
    let text = |k: &str| field(k).as_str().unwrap_or_else(|| panic!("statuses.json row {i}: `{k}` is not a string"));
    let int = |k: &str| field(k).as_i64().unwrap_or_else(|| panic!("statuses.json row {i}: `{k}` is not an integer")) as i32;
    let flag = |k: &str| field(k).as_bool().unwrap_or_else(|| panic!("statuses.json row {i}: `{k}` is not a bool"));
    let buff = match text("kind") {
        "buff" => true,
        "debuff" => false,
        k => panic!("statuses.json row {i}: kind `{k}` is not buff or debuff"),
    };
    let stacking = Stacking::from_id(text("stacking")).unwrap_or_else(|| panic!("statuses.json row {i}: unknown stacking"));
    let magnitude_unit = text("magnitude_unit");
    assert!(["pct", "flat", "pct_max_hp"].contains(&magnitude_unit), "statuses.json row {i}: unknown magnitude_unit `{magnitude_unit}`");
    let tags = field("primitives").as_array().unwrap_or_else(|| panic!("statuses.json row {i}: `primitives` is not a list"));
    let primitives: Vec<Primitive> = tags.iter().map(|t| t.as_str().and_then(Primitive::from_id).unwrap_or_else(|| panic!("statuses.json row {i}: unknown primitive {t}"))).collect();
    assert!(!primitives.is_empty(), "statuses.json row {i}: at least one primitive");
    let blocks = field("lock_blocks").as_array().unwrap_or_else(|| panic!("statuses.json row {i}: `lock_blocks` is not a list"));
    let lock_blocks: Vec<i32> = blocks.iter().map(|b| b.as_i64().unwrap_or_else(|| panic!("statuses.json row {i}: lock block {b} is not an integer")) as i32).collect();
    assert!(lock_blocks.is_empty() || primitives.contains(&Primitive::BlockLock), "statuses.json row {i}: lock_blocks on a block_lock row only");
    let rating = match text("rating") {
        "" => Option::None,
        w => Some(Rating::from_id(w).unwrap_or_else(|| panic!("statuses.json row {i}: unknown rating `{w}`"))),
    };
    assert_eq!(rating.is_some(), primitives.contains(&Primitive::Modifier), "statuses.json row {i}: a rating on every modifier row and only there");
    let targeting = match text("targeting") {
        "" => Option::None,
        w => Some(Targeting::from_id(w).unwrap_or_else(|| panic!("statuses.json row {i}: unknown targeting `{w}`"))),
    };
    assert_eq!(targeting.is_some(), primitives.contains(&Primitive::Targeting), "statuses.json row {i}: a targeting word on every targeting row and only there");
    StatusRow {
        id: text("id"),
        buff,
        resistable: flag("resistable"),
        default_rounds: int("default_rounds"),
        stacking,
        cap: int("cap"),
        display: text("display"),
        grant: text("grant"),
        magnitude_min: int("magnitude_min"),
        magnitude_max: int("magnitude_max"),
        magnitude_unit,
        intent_bound: flag("intent_bound"),
        // Loaded once per process into a static table, so the lists live as long as the rows.
        lock_blocks: Box::leak(lock_blocks.into_boxed_slice()),
        rating,
        targeting,
        primitives: Box::leak(primitives.into_boxed_slice()),
    }
}

/// S1.2-S3: the default of the ruleset number `bleed_pct` (the fraction of max HP a bleed tick deals).
pub const BLEED_PCT: f64 = 0.06;
/// S1.2-S3: the default of `debt_stagger` (the AP debt at a round start that staggers).
pub const DEBT_STAGGER: i32 = 3;
/// S1.2-S3: the default of `debt_stun` (the AP debt at a round start that stuns).
pub const DEBT_STUN: i32 = 5;
/// F2.2-S1: the default of `debt_stagger_status` (the status a `debt_stagger` debt imposes).
pub const DEBT_STAGGER_STATUS: &str = "stagger";
/// F2.2-S1: the default of `debt_stun_status` (the status a `debt_stun` debt imposes).
pub const DEBT_STUN_STATUS: &str = "stun";
/// S1.2-S3: the default of `counter_mult` (riposte's counter damage multiplier; was `rpgkit_battle::command`'s const).
pub const COUNTER_MULT: f64 = 0.5;
/// S1.2-S4: the default of `guard_common_mitigation` (the guard fraction of a 1-round Guard skill that authors none).
pub const GUARD_COMMON_MITIGATION: f64 = 0.35;
/// S1.2-S4: the default of `guard_multi_round_mitigation` (the guard fraction of a multi-round Guard skill that authors
/// none; lower than the 1-round one, FB-48).
pub const GUARD_MULTI_ROUND_MITIGATION: f64 = 0.20;

/// S1.2-S4 (S1 section 1, `modifier`): the status a multi-round Guard skill queues for its later rounds; its magnitude
/// is the guard fraction in percent and it raises the unit's guard for every block of the rounds it is active in.
pub const GUARD: &str = "guard";

/// S1.2-S4: the guard fraction an active `guard` status gives `c` (its magnitude / 100), 0 without one.
pub fn guard_status_mitigation(c: &Unit) -> f64 {
    if !c.st.active.contains(GUARD) {
        return 0.0;
    }
    c.active_magnitude(GUARD) as f64 / 100.0
}

pub fn row_of(status: &str) -> Option<&'static StatusRow> {
    ROWS.iter().find(|r| r.id == status)
}

pub fn cap_of(status: &str) -> i32 {
    row_of(status).map(|r| r.cap).unwrap_or(0)
}

pub fn stacking_of(status: &str) -> Stacking {
    row_of(status).map(|r| r.stacking).unwrap_or(RefreshMaxDuration)
}

/// RULESET1_INTENT IN6: whether the row of `status` is intent-bound (false for an unknown id).
pub fn intent_bound(status: &str) -> bool {
    row_of(status).is_some_and(|r| r.intent_bound)
}

pub fn display_name(status: &str) -> String {
    row_of(status).map(|r| r.display.to_string()).unwrap_or_else(|| status.to_string())
}

/// SkillText.format_magnitude: "20%", "4 armor", "3 HP" (poison), "5% max HP"; "" for no magnitude.
pub fn format_magnitude(st: &str, magnitude: i32) -> String {
    if magnitude <= 0 || !has_magnitude(st) {
        return String::new();
    }
    match magnitude_unit(st) {
        // S1.2-S2: a flat tick (poison) deals HP; a flat modifier (fortify, expose) is armor
        "flat" if row_of(st).is_some_and(|r| r.has_primitive(Primitive::Tick)) => format!("{magnitude} HP"),
        "flat" => format!("{magnitude} armor"),
        "pct_max_hp" => format!("{magnitude}% max HP"),
        _ => format!("{magnitude}%"),
    }
}

/// StatusEffects.label_with_magnitude: "Fortify 4 armor", or the name alone.
pub fn label_with_magnitude(st: &str, magnitude: i32) -> String {
    let shown = format_magnitude(st, magnitude);
    if shown.is_empty() {
        display_name(st)
    } else {
        format!("{} {shown}", display_name(st))
    }
}

/// Buff ids in table order (StatusEffects.buffs).
pub fn buffs() -> Vec<&'static str> {
    ROWS.iter().filter(|r| r.buff).map(|r| r.id).collect()
}

/// Debuff ids in table order (cleanse lists follow it).
pub fn debuffs() -> Vec<&'static str> {
    ROWS.iter().filter(|r| !r.buff).map(|r| r.id).collect()
}

pub fn defend_buffs() -> Vec<&'static str> {
    ROWS.iter().filter(|r| r.buff && r.grant == "defend").map(|r| r.id).collect()
}

/// StatusRegistry.magnitude_unit: "pct", "flat" (armor points; poison HP per round) or "pct_max_hp".
/// Read from the row's `magnitude_unit` since S1.2-S1; "pct" for an unknown id.
pub fn magnitude_unit(status: &str) -> &'static str {
    row_of(status).map(|r| r.magnitude_unit).unwrap_or("pct")
}

pub fn has_magnitude(status: &str) -> bool {
    row_of(status).is_some_and(|r| r.magnitude_max > 0)
}

pub fn magnitude_or_min(status: &str, magnitude: i32) -> i32 {
    if magnitude <= 0 && has_magnitude(status) {
        row_of(status).unwrap().magnitude_min
    } else {
        magnitude
    }
}

/// StatusRegistry.stack: (rounds, potency) of an existing application (rounds 0 = absent) plus a new one.
pub fn stack(existing_rounds: i32, existing_potency: i32, new_rounds: i32, new_potency: i32, status: &str) -> (i32, i32) {
    let rule = stacking_of(status);
    let cap = cap_of(status);
    let add = new_rounds.max(1);
    let (mut rounds, mut potency) = (add, new_potency);
    if existing_rounds > 0 {
        match rule {
            ExtendDuration => {
                rounds = existing_rounds + add;
                potency = existing_potency.max(new_potency);
            }
            StackPotency => {
                rounds = existing_rounds.max(add);
                potency = existing_potency + new_potency;
            }
            None => {
                rounds = existing_rounds;
                potency = existing_potency;
            }
            RefreshMaxDuration => {
                rounds = existing_rounds.max(add);
                potency = existing_potency.max(new_potency);
            }
        }
    }
    if cap > 0 {
        if rule == ExtendDuration {
            rounds = rounds.min(cap);
        } else {
            potency = potency.min(cap);
        }
    }
    (rounds, potency)
}

/// Bleed's tick: `bleed_pct` (ruleset number, S1.2-S3) of max HP, at least 1.
pub fn bleed_damage(c: &Unit, n: &Numbers) -> i32 {
    roundi(c.max_hp as f64 * n.bleed_pct).max(1)
}

/// S1.2-S2: the round-end amount of the `tick` row `row` on `c` (damage for a debuff, healing for a buff): bleed its
/// own formula (kept on its id, S1 section 4 item 2), a `flat` tick its magnitude (poison: may be 0), a `pct_max_hp`
/// tick that percent of max HP, at least 1 (burn, regen).
pub fn tick_amount(c: &Unit, row: &StatusRow, n: &Numbers) -> i32 {
    if row.id == "bleed" {
        return bleed_damage(c, n);
    }
    match row.magnitude_unit {
        "flat" => c.active_magnitude(row.id),
        _ => roundi(c.max_hp as f64 * c.active_magnitude(row.id) as f64 / 100.0).max(1),
    }
}

/// The `tick` rows in table order, debuffs (`buff` false: bleed, poison, burn) or buffs (regen).
pub fn tick_rows(buff: bool) -> impl Iterator<Item = &'static StatusRow> {
    ROWS.iter().filter(move |r| r.buff == buff && r.has_primitive(Primitive::Tick))
}

/// The timeline blocks `status` locks: its row's `lock_blocks` when it is a `block_lock` row (S1.2-S2), else none.
pub fn blocked_blocks(status: &str) -> &'static [i32] {
    row_of(status).filter(|r| r.has_primitive(Primitive::BlockLock)).map_or(&[], |r| r.lock_blocks)
}

/// I1.2-S1 (I_AI section 1 item 1): whether `status` is a `targeting` row that pulls enemy targeting toward its holder
/// (false for an unknown id).
pub fn has_targeting_pull(status: &str) -> bool {
    row_of(status).is_some_and(|r| r.has_primitive(Primitive::Targeting) && r.targeting == Some(Targeting::Pull))
}

/// I1.2-S1 (item 2): whether `status` is a `reactive` row (a counter while active).
pub fn has_reactive_counter(status: &str) -> bool {
    row_of(status).is_some_and(|r| r.has_primitive(Primitive::Reactive))
}

/// I1.2-S1 (item 3): whether `status` is a `tick` row that heals (`Tick::Heal`, a buff) or damages (`Tick::Damage`, a
/// debuff).
pub fn has_tick(status: &str, tick: Tick) -> bool {
    row_of(status).is_some_and(|r| r.has_primitive(Primitive::Tick) && r.buff == (tick == Tick::Heal))
}

/// I1.2-S1 (item 4): whether `status` stops its holder moving this round: a `block_lock` row that locks the Move
/// command itself (no `lock_blocks`: root) or every one of the round's `move_blocks` opening blocks (1 to
/// `move_blocks`: stun with 2 move blocks; stagger leaves block 2 open).
pub fn locks_movement(status: &str, move_blocks: usize) -> bool {
    row_of(status).is_some_and(|r| r.has_primitive(Primitive::BlockLock) && (r.lock_blocks.is_empty() || (1..=move_blocks as i32).all(|b| r.lock_blocks.contains(&b))))
}

/// I1.2-S1 (item 5): whether `status` is a `modifier` row changing `rating` (either direction; a row's `kind` says
/// which).
pub fn modifies_rating(status: &str, rating: Rating) -> bool {
    row_of(status).is_some_and(|r| r.has_primitive(Primitive::Modifier) && r.rating == Some(rating))
}

/// Whether `status` is a buff row (false for an unknown id).
pub fn is_buff(status: &str) -> bool {
    row_of(status).is_some_and(|r| r.buff)
}

/// The chance (0-100) a roll of `potency` lands `status` on `target`: potency less INT resistance and the status's own
/// resist; S1.2-S3: 100 for a row that is not `resistable` (no roll). No attacker term (hazards, grounds, triggers).
pub fn apply_chance(target: &Unit, status: &str, potency: i32) -> i32 {
    apply_chance_with(target, status, potency, 0.0)
}

/// U1.2-S3b: the attacker's potency term of a skill's status roll, `know_potency x KNOW` (S1 section 2
/// `attacker_know_potency`; the ruleset number `know_potency`, 0 = potency off, so the term is 0).
pub fn know_bonus(attacker: &Unit, know_potency: f64) -> f64 {
    if know_potency == 0.0 {
        return 0.0;
    }
    know_potency * attacker.stats.stat("KNOW", attacker.weapon_context().as_deref()) as f64
}

/// `apply_chance` with an attacker term `bonus` (`know_bonus`) added to the potency before resistance:
/// `clamp(round(potency + bonus - INT resist - status resist), 0, 100)`; 100 for a row that is not `resistable`.
pub fn apply_chance_with(target: &Unit, status: &str, potency: i32, bonus: f64) -> i32 {
    if !resistable(status) {
        return 100;
    }
    let resist = target.debuff_resistance() + target.resist_for(status) as f64;
    roundi(potency as f64 + bonus - resist).clamp(0, 100)
}

/// Whether `status` is rolled against resistance: its row's `resistable` (true for an id with no row, as before).
pub fn resistable(status: &str) -> bool {
    row_of(status).is_none_or(|r| r.resistable)
}

/// Rolls a skill-inflicted status (randi_range(1, 100) <= chance); on success it is queued for next round. S1.2-S3: a
/// row that is not `resistable` lands without a roll (no draw).
pub fn try_inflict(target: &mut Unit, status: &str, potency: i32, rng: Option<&mut GodotRng>, rounds: i32, magnitude: i32, source: &str) -> bool {
    try_inflict_with(target, status, potency, 0.0, rng, rounds, magnitude, source)
}

/// `try_inflict` with an attacker term `bonus` (`know_bonus`; U1.2-S3b) in the chance (`apply_chance_with`); a row that
/// is not `resistable` still lands without a roll.
#[allow(clippy::too_many_arguments)]
pub fn try_inflict_with(target: &mut Unit, status: &str, potency: i32, bonus: f64, rng: Option<&mut GodotRng>, rounds: i32, magnitude: i32, source: &str) -> bool {
    if !target.is_alive() {
        return false;
    }
    if !resistable(status) {
        queue(target, status, rounds, magnitude, source);
        return true;
    }
    let chance = apply_chance_with(target, status, potency, bonus);
    let roll = match rng {
        Some(r) => r.randi_range(1, 100),
        Option::None => 100,
    };
    if roll > chance {
        return false;
    }
    queue(target, status, rounds, magnitude, source);
    true
}

/// Queues `status` for the target's next round (stacking rule and cap; the newer source wins).
pub fn queue(target: &mut Unit, status: &str, rounds: i32, magnitude: i32, source: &str) {
    let magnitude = magnitude_or_min(status, magnitude);
    let st = &mut target.st;
    st.pending.insert(status.to_string());
    let existing = st.pending_rounds.get(status).copied().unwrap_or(0);
    let (r, p) = stack(existing, st.pending_magnitudes.get(status).copied().unwrap_or(0), rounds, magnitude, status);
    st.pending_rounds.insert(status.to_string(), r);
    if has_magnitude(status) {
        st.pending_magnitudes.insert(status.to_string(), p);
    }
    if !source.is_empty() {
        st.pending_sources.insert(status.to_string(), source.to_string());
    }
}

/// Makes a buff active right now, lasting `rounds` rounds counting the current one.
pub fn grant_now(target: &mut Unit, status: &str, rounds: i32, magnitude: i32, source: &str) {
    note_grant_under(target, status, rounds, magnitude_or_min(status, magnitude), source);
    apply_now(target, status, rounds, magnitude, source);
}

/// EMPOWER1: a grant landing on a status a band holds also keeps its own part, stacked by the row's rule.
fn note_grant_under(target: &mut Unit, status: &str, rounds: i32, magnitude: i32, source: &str) {
    let st = &mut target.st;
    if !st.band_held.contains_key(status) {
        return;
    }
    let old = st.band_under.get(status).cloned().unwrap_or_default();
    let (rounds, magnitude) = stack(old.rounds, old.magnitude, rounds, magnitude, status);
    st.band_under.insert(status.to_string(), GrantUnder { rounds, magnitude, source: source.to_string() });
}

/// EMPOWER1: a meter band holds `status` this round: merged with any grant like a 1-round grant (the higher magnitude
/// applies). An instance already there when the band takes hold is a grant, kept apart for `band_release`.
pub fn band_hold(target: &mut Unit, status: &str, magnitude: i32, source: &str) {
    let st = &mut target.st;
    if !st.band_held.contains_key(status) && st.active.contains(status) {
        let under = GrantUnder {
            rounds: st.rounds_left.get(status).copied().unwrap_or(0),
            magnitude: st.magnitudes.get(status).copied().unwrap_or(0),
            source: st.sources.get(status).cloned().unwrap_or_default(),
        };
        st.band_under.insert(status.to_string(), under);
    }
    st.band_held.insert(status.to_string(), magnitude);
    apply_now(target, status, 1, magnitude, source);
}

/// EMPOWER1: a band lets go of `status`: only its part goes. A queued grant stays queued and an active grant is put
/// back at its own rounds and magnitude. Does nothing when the band had not taken hold yet.
pub fn band_release(target: &mut Unit, status: &str) {
    let st = &mut target.st;
    if st.band_held.shift_remove(status).is_none() {
        return;
    }
    st.active.shift_remove(status);
    st.rounds_left.shift_remove(status);
    st.magnitudes.shift_remove(status);
    st.sources.shift_remove(status);
    if let Some(under) = st.band_under.shift_remove(status) {
        st.active.insert(status.to_string());
        st.rounds_left.insert(status.to_string(), under.rounds);
        if has_magnitude(status) {
            st.magnitudes.insert(status.to_string(), under.magnitude);
        }
        if !under.source.is_empty() {
            st.sources.insert(status.to_string(), under.source);
        }
    } else if !st.pending.contains(status) {
        st.intents.shift_remove(status);
    }
}

fn apply_now(target: &mut Unit, status: &str, rounds: i32, magnitude: i32, source: &str) {
    let magnitude = magnitude_or_min(status, magnitude);
    let st = &mut target.st;
    let (r, p) = stack(st.rounds_left.get(status).copied().unwrap_or(0), st.magnitudes.get(status).copied().unwrap_or(0), rounds, magnitude, status);
    st.active.insert(status.to_string());
    st.rounds_left.insert(status.to_string(), r);
    if has_magnitude(status) {
        st.magnitudes.insert(status.to_string(), p);
    }
    if !source.is_empty() {
        st.sources.insert(status.to_string(), source.to_string());
    }
}

/// A skill's extra_grants rows ({status, rounds, magnitude}); `defend` makes defend buffs active at once.
pub fn grant_extras(target: &mut Unit, extras: &[Value], defend: bool, source: &str) -> Vec<String> {
    let mut granted = Vec::new();
    for row in extras {
        let Some(o) = row.as_object() else { continue };
        let status = aegis_core::value::string(o, "status", "");
        if status.is_empty() {
            continue;
        }
        let rounds = aegis_core::value::int(o, "rounds", 1);
        let magnitude = aegis_core::value::int(o, "magnitude", 0);
        if defend && defend_buffs().contains(&status.as_str()) {
            grant_now(target, &status, rounds, magnitude, source);
        } else {
            queue(target, &status, rounds, magnitude, source);
        }
        granted.push(status);
    }
    granted
}

/// Removes `status` active and pending; true when anything was there.
pub fn remove(target: &mut Unit, status: &str) -> bool {
    let found = target.st.has_any(status);
    target.remove_status(status);
    found
}

/// Removes only the ACTIVE instance (a queued one stays; exploit consume).
pub fn remove_active(target: &mut Unit, status: &str) -> bool {
    target.st.band_under.shift_remove(status);
    let st = &mut target.st;
    let found = st.active.contains(status);
    st.active.shift_remove(status);
    st.rounds_left.shift_remove(status);
    st.magnitudes.shift_remove(status);
    st.sources.shift_remove(status);
    if !st.pending.contains(status) {
        st.intents.shift_remove(status);
    }
    found
}

/// The status an AP debt of `debt` at a round start imposes: `debt_stun_status` from `debt_stun`, `debt_stagger_status`
/// from `debt_stagger` (ruleset numbers, S1.2-S3, F2.2-S1), else none.
pub fn debt_status(debt: i32, n: &Numbers) -> &str {
    if debt >= n.debt_stun {
        &n.debt_stun_status
    } else if debt >= n.debt_stagger {
        &n.debt_stagger_status
    } else {
        ""
    }
}

/// The AP a round start gives `c` from `ap` (its AP at the round's end): `ap_regen` plus its bonus, capped at its
/// max (`ap_regen` below 0: back to full). `roll_over` applies it; the AP forecast (INPUT1 AP1) reads it.
pub fn regen_ap(c: &Unit, ap: i32, ap_regen: i32) -> i32 {
    if ap_regen < 0 {
        c.max_ap
    } else {
        c.max_ap.min(ap + ap_regen + c.ap_regen_bonus)
    }
}

/// Round start: last round's statuses expire (multi-round ones count down), queued ones and AP-debt penalties take
/// effect (`n`: the ruleset's debt numbers), then AP recovers (`regen_ap`). Returns the statuses now active.
pub fn roll_over(c: &mut Unit, ap_regen: i32, n: &Numbers) -> Vec<String> {
    let st = &mut c.st;
    let carried: Vec<(String, i32)> = st.rounds_left.iter().filter_map(|(k, v)| (v - 1 > 0).then(|| (k.clone(), v - 1))).collect();
    let old_magnitudes = st.magnitudes.clone();
    let old_sources = st.sources.clone();
    st.active.clear();
    st.rounds_left.clear();
    st.magnitudes.clear();
    st.sources.clear();
    for (s, left) in carried {
        st.active.insert(s.clone());
        st.rounds_left.insert(s.clone(), left);
        if let Some(m) = old_magnitudes.get(&s) {
            st.magnitudes.insert(s.clone(), *m);
        }
        if let Some(src) = old_sources.get(&s) {
            st.sources.insert(s.clone(), src.clone());
        }
    }
    // EMPOWER1: a grant beneath a band counts down with the instance
    st.band_under.retain(|_, u| {
        u.rounds -= 1;
        u.rounds > 0
    });
    let pending: Vec<String> = st.pending.iter().cloned().collect();
    for s in pending {
        let queued = st.pending_rounds.get(&s).copied().unwrap_or(1);
        if st.band_held.contains_key(&s) {
            let (magnitude, source) = (st.pending_magnitudes.get(&s).copied().unwrap_or(0), st.pending_sources.get(&s).cloned().unwrap_or_default());
            let old = st.band_under.get(&s).cloned().unwrap_or_default();
            let (rounds, magnitude) = stack(old.rounds, old.magnitude, queued, magnitude, &s);
            st.band_under.insert(s.clone(), GrantUnder { rounds, magnitude, source });
        }
        if has_magnitude(&s) {
            let (_, p) = stack(st.rounds_left.get(&s).copied().unwrap_or(0), st.magnitudes.get(&s).copied().unwrap_or(0), queued, st.pending_magnitudes.get(&s).copied().unwrap_or(0), &s);
            st.magnitudes.insert(s.clone(), p);
        }
        st.active.insert(s.clone());
        let left = st.rounds_left.get(&s).copied().unwrap_or(0).max(queued);
        st.rounds_left.insert(s.clone(), left);
        if let Some(src) = st.pending_sources.get(&s).cloned() {
            st.sources.insert(s.clone(), src);
        }
    }
    st.pending.clear();
    st.pending_rounds.clear();
    st.pending_magnitudes.clear();
    st.pending_sources.clear();
    // IN6: an intent tag lives as long as its instance (classic never sets one)
    let (active, intents) = (&st.active, &mut st.intents);
    intents.retain(|k, _| active.contains(k));
    c.riposted.clear();
    // F2.2-S1 (FB-58b): the debt status goes straight into `st.active` for the coming round, with the round count of
    // a 1-round instance (an instance already there keeps its longer count)
    let penalty = debt_status((-c.ap).max(0), n);
    if !penalty.is_empty() {
        c.st.active.insert(penalty.to_string());
        c.st.rounds_left.entry(penalty.to_string()).or_insert(1);
    }
    c.ap = regen_ap(c, c.ap, ap_regen);
    c.st.active.iter().cloned().collect()
}
