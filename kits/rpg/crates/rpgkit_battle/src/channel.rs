//! Channel rounds (PK1-S2a, `docs/sessions/PK1_CORRUPTION.md` 4, ACTS23 5.1 Option B, FB-167 Q1-Q3; rule `channel`,
//! engine-only, no Godot original). A unit that holds the ruleset's `channel_meter` may flip its round to Channel: the
//! round flag `RoundState::channel` (not an intent: the first skill's row stays the round's intent, FB-167 Q1 (a)),
//! cleared at every round start, flipped any time while planning by the converting toggle
//! (`round_intent_plan::set_channel`). In a Channel round the planner offers only channeled versions: the unit's
//! `channel.only` skills, its class basic's version and the versions of the deck actives it marked channeled (deck
//! `channeled`, at most `channeled_actives`), each single intent (IR21) and not `channel.none`; Move and Rest stay, items do
//! not; Rest becomes `channel_rest`. A channeled version resolves with power x `channel_power` (or `channel.power`),
//! armour piercing on attacks, `channel_cost` of the meter by AP (or `channel.cost`) paid at its start and its
//! `channel.effects` as a rider; a `channel.only` skill resolves as authored (no derived power, piercing or cost) plus
//! its own overrides. A Channel round counts as a gain of the channel meter, so its decay skips the round. Every entry
//! point reads the rule (and `round_intent`, `unit_meters`, a named meter) first, so a battle without it, or without a
//! unit holding the meter, plans and resolves as before (no draw, no line, no order change).
//!
//! PK1-S2b: the tells show Channel while the flag is set (`tell_intent`; the records in `intent`) and name a channeled
//! version with the channeled mark (`channeled_name`, the term `channeled_mark`); the AI's Channel row is `ai::channel_row`.

use crate::meter;
use crate::round_intent::{self, RoundIntent};
use crate::skill::{self, Kind, SkillId};
use crate::world::{UnitId, World};
use aegis_core::value::Obj;
use rpgkit_data::{MeterDelta, Rider, SkillDef};
use rpgkit_rules::ruleset;

/// PK1 4.3 item 2: the reason a Standard version (or an item) is greyed in a Channel round.
pub const CHANNEL_ROUND: &str = "Channel round";
/// PK1 4.3 item 1: the reason a channeled-only skill is greyed in a Standard round.
pub const ONLY_CHANNEL: &str = "Only in a Channel round";

/// Whether the battle plays Channel rounds: the rule `channel` with `round_intent` and `unit_meters` (its requirements)
/// and a `channel_meter` named.
pub fn on(w: &World) -> bool {
    w.ruleset.has(ruleset::CHANNEL) && w.ruleset.has(ruleset::ROUND_INTENT) && w.ruleset.has(ruleset::UNIT_METERS) && !w.ruleset.numbers.channel_meter.is_empty()
}

/// The channel meter's id ("" without the rule).
pub fn meter_id(w: &World) -> &str {
    if on(w) {
        &w.ruleset.numbers.channel_meter
    } else {
        ""
    }
}

/// PK1 4.2: whether `c` can Channel: the rule is on and the unit is on the field holding the channel meter (others
/// never see the toggle).
pub fn can_channel(w: &World, c: UnitId) -> bool {
    on(w) && w.units.get(c).is_some_and(|u| u.is_active()) && meter::value(w, c, &w.ruleset.numbers.channel_meter).is_some()
}

/// Whether `c`'s round is a Channel round (the round flag; false without the rule).
pub fn is_channel(w: &World, c: UnitId) -> bool {
    on(w) && w.round_states.get(&c).is_some_and(|r| r.channel)
}

/// PK1-S2b (PK1 4.3 item 6): the intent `c`'s tells show: Channel while its round flag is set (from the toggle on,
/// with or without a skill planned yet), else its round intent (`round_intent::intent_of`; None without one). The
/// round's own intent stays the first skill's row for the rules (FB-167 Q1 (a)); only the tells read this.
pub fn tell_intent(w: &World, c: UnitId) -> Option<RoundIntent> {
    if is_channel(w, c) {
        return Some(RoundIntent::Channel);
    }
    round_intent::intent_of(w, c)
}

/// PK1-S2b (PK1 4.3 item 6): a channeled version's name in the tells: the skill's name with the channeled mark (the term
/// `channeled_mark`'s short form, the game's `terms` row else "Channeled"): "Cleave (Channeled)".
pub fn channeled_name(cat: &rpgkit_data::Catalog, name: &str) -> String {
    format!("{name} ({})", cat.term(rpgkit_data::terms::CHANNELED_MARK).short)
}

/// The kinds with no intent of their own (Move, Rest, Target, Status): neutral in either mode.
fn neutral(s: &SkillDef) -> bool {
    matches!(skill::kind(s), Kind::Move | Kind::Rest | Kind::Target | Kind::Status)
}

/// PK1 4.2: whether `s` has a channeled version: an authored skill (not an item, not neutral) that IR21 derives one
/// intent for (`rpgkit_rules::intent::single_intent`, the content and save checks' test, whether or not the battle
/// plays `field_intent`), not `channel.none` and not `channel.only` (a channeled-only skill has no Standard version to
/// convert from). `_w` keeps the battle in the signature for the callers.
pub fn has_version(_w: &World, s: &SkillDef) -> bool {
    if skill::is_item(s) || neutral(s) || s.channel.none || s.channel.only {
        return false;
    }
    rpgkit_rules::intent::single_intent(&rpgkit_rules::intent::IntentInput::of_def(s))
}

/// Whether `s` is channeled-only (`channel.only`).
pub fn only(s: &SkillDef) -> bool {
    s.channel.only && !skill::is_item(s)
}

/// CLASS-BASICS: `c`'s class role basics (the deck rule's basics beyond the ruleset's), by family.
fn class_basics(w: &World, c: UnitId) -> Vec<String> {
    let base = &w.ruleset.numbers.basics;
    w.units[c].deck_rule.iter().flat_map(|r| r.basics.iter()).filter(|b| !base.contains(b)).cloned().collect()
}

/// PK1 4.2: the deck actives `c` marked channeled whose version opens in a Channel round: its deck's `channeled` families in
/// order that are among its actives (`Unit::active_families` under the deck rule, else the deck's `active`) and have a
/// channeled version, at most `channeled_actives` of them (PK2-S2: a unit with a body, one per graft).
pub fn marked(w: &World, c: UnitId) -> Vec<String> {
    let u = &w.units[c];
    let actives: Vec<String> = if u.deck_rule.is_some() && !u.active_families().is_empty() { u.active_families() } else { u.deck.active.clone() };
    // PK2-S2: a unit with a body marks one per graft (`Unit::channeled_limit`)
    let most = usize::try_from(u.channeled_limit(w.ruleset.numbers.channeled_actives)).unwrap_or(0);
    let mut out: Vec<String> = Vec::new();
    for fam in &u.deck.channeled {
        if out.len() >= most {
            break;
        }
        // an auto deck (no actives listed, an enemy's row): a marked family the unit uses counts
        let active = if actives.is_empty() { crate::round_intent_plan::family_skill(w, c, fam).is_some() } else { actives.contains(fam) };
        if out.contains(fam) || !active {
            continue;
        }
        if w.cat.tier_one(fam).is_some_and(|s| has_version(w, s)) {
            out.push(fam.clone());
        }
    }
    out
}

/// PK1 4.2: whether skill `s` opens for `c` in a Channel round: a neutral kind (Move, Rest), a `channel.only` skill,
/// or the channeled version of its class basic or of an active it marked channeled.
pub fn opens(w: &World, c: UnitId, s: &SkillDef) -> bool {
    // PK2-S4: a technique (and the wielded family's keystone) is standard rounds only
    if skill::is_item(s) || w.units[c].technique_skill(&s.id) {
        return false;
    }
    if neutral(s) || only(s) {
        return true;
    }
    if !has_version(w, s) {
        return false;
    }
    let fam = s.family_id();
    class_basics(w, c).iter().any(|b| b == fam) || marked(w, c).iter().any(|m| m == fam)
}

/// PK1 4.3 items 1-2, 7: why `c` may not plan `s` in its round's mode (None: it may): a Standard version (or an
/// item) in a Channel round is "Channel round", a channeled-only skill in a Standard round "Only in a Channel round"
/// (ACT1-HOOKS2: not one `standard_grant` opens). None without the rule. PK2-S4: first, with or without the rule, a technique whose grip tag does not fit the
/// main-hand weapon (`Unit::technique_problem`); a technique in a Channel round is "Channel round" (`opens`).
/// PK2-S5b: then, under the sims' `World::dulled_lock`, a technique that swings a dulled weapon (`wear::problem`).
pub fn problem(w: &World, c: UnitId, s: &SkillDef) -> Option<&'static str> {
    if let Some(p) = w.units.get(c).and_then(|u| u.technique_problem(&s.id)) {
        return Some(p);
    }
    // PK2-S5b: the sims' alternative dulled mode locks the techniques that swing a dulled weapon (`wear::problem`)
    if let Some(p) = crate::wear::problem(w, c, s) {
        return Some(p);
    }
    if !on(w) {
        return None;
    }
    if is_channel(w, c) {
        return (!opens(w, c, s)).then_some(CHANNEL_ROUND);
    }
    (only(s) && !standard_grant(w, c, s)).then_some(ONLY_CHANNEL)
}

/// ACT1-HOOKS2 (FB-244 (1)): whether `c` plays channeled-only `s` in a Standard round: the encounter granted it for
/// this battle (`Unit::battle_skill`) and the unit cannot Channel (it holds no channel meter), so the skill resolves
/// as authored and pays no meter (no version outside a Channel round).
pub fn standard_grant(w: &World, c: UnitId, s: &SkillDef) -> bool {
    w.units.get(c).is_some_and(|u| u.battle_skill(&s.id)) && !can_channel(w, c)
}

/// PK1 4.3 item 2: why `c` may not use a pouch item now (None: it may): items have no channeled version, so a Channel
/// round greys them.
pub fn item_problem(w: &World, c: UnitId) -> Option<&'static str> {
    is_channel(w, c).then_some(CHANNEL_ROUND)
}

/// PK1 4.2: what a channeled version adds to its skill: the channel meter it pays at its start, the factor on its
/// power, armour piercing (attacks), and the trigger effects of `channel.effects` (run as a rider).
#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    pub cost: i32,
    pub power: f64,
    pub pierce: bool,
    pub effects: Vec<Obj>,
}

/// The channeled version `c` would use `s` in now (None: a Standard round, no rule, an item, a neutral kind, or a skill
/// with no version). A derived version: `channel.cost` else `channel_cost` by the skill's AP (`skill::ap_cost_by`: by
/// the skill, so a unit that pays no AP still pays the meter), `channel.power` else `channel_power`, piercing on an
/// attack; a `channel.only` skill: `channel.cost` else 0, `channel.power` else 1, its own piercing.
pub fn version(w: &World, c: UnitId, s: &SkillDef) -> Option<Version> {
    if !is_channel(w, c) || skill::is_item(s) || neutral(s) {
        return None;
    }
    let n = &w.ruleset.numbers;
    if only(s) {
        return Some(Version { cost: s.channel.cost.unwrap_or(0).max(0), power: s.channel.power.unwrap_or(1.0), pierce: false, effects: s.channel.effects.clone() });
    }
    if !has_version(w, s) {
        return None;
    }
    let ap = skill::ap_cost_by(s, Some(&w.units[c]), &w.ruleset);
    let cost = s.channel.cost.unwrap_or_else(|| n.channel_cost_for(ap)).max(0);
    Some(Version { cost, power: s.channel.power.unwrap_or(n.channel_power), pierce: skill::kind(s) == Kind::Attack, effects: s.channel.effects.clone() })
}

/// The factor on `s`'s power for `c` now (1 outside a channeled version).
pub fn power(w: &World, c: UnitId, s: &SkillDef) -> f64 {
    version(w, c, s).map_or(1.0, |v| v.power)
}

/// HOOKS1 UM2 with PK1-S2a: the meter changes `c` pays at `s`'s start: the skill's `meters` (rule unit_meters), then a
/// channeled version's channel cost on the channel meter (rule channel). The planners and the command read it.
pub fn meters_for(w: &World, c: UnitId, s: &SkillDef) -> Vec<MeterDelta> {
    let mut out = skill::meters_by(s, &w.ruleset).to_vec();
    if let Some(v) = version(w, c, s).filter(|v| v.cost > 0) {
        out.push(MeterDelta { meter: w.ruleset.numbers.channel_meter.clone(), delta: v.cost });
    }
    out
}

/// The rider a channeled version's `channel.effects` run as (None: none).
pub fn rider(v: &Version) -> Option<Rider> {
    (!v.effects.is_empty()).then(|| Rider { when: Obj::new(), effects: v.effects.clone() })
}

/// PK1 4.3 item 5 (`channel_rest`): the skill `c` plans on a Rest block now: in a Channel round the `channel_rest`
/// family's row the unit uses (`round_intent_plan::family_skill`; its Feed) unless the value is `normal` or the unit
/// has no such row; else the catalog's Rest.
pub fn rest_skill(w: &World, c: UnitId) -> SkillId {
    let rest = w.book().lowest(Kind::Rest);
    if !is_channel(w, c) {
        return rest;
    }
    let fam = &w.ruleset.numbers.channel_rest;
    if fam.is_empty() || fam == ruleset::CHANNEL_REST_NORMAL {
        return rest;
    }
    crate::round_intent_plan::family_skill(w, c, fam).unwrap_or(rest)
}

/// Whether `sid` is `c`'s Channel Rest this round (`rest_skill`, not the plain Rest).
pub fn is_channel_rest(w: &World, c: UnitId, sid: SkillId) -> bool {
    is_channel(w, c) && sid == rest_skill(w, c) && skill::kind(&w.cat.skills[sid]) != Kind::Rest
}

/// PK1 4.3 item 4: `c`'s Channel order for `intent`: its `channel.only` skills (in `families` order: the auto order's
/// actives and worn skills, basics), then the marked actives, then the class basic, each the row it uses now
/// (`round_intent_plan::family_skill`) with `intent` among its rows, once. The meter rows' skills (the Channel basics,
/// UM6: Feed, Vent) and the Channel Rest stay out: Vent is picked by hand, Feed fills only what the order cannot
/// ("what cannot fit is Feed", `rest_skill`).
pub fn order(w: &World, c: UnitId, intent: RoundIntent, families: &[String]) -> Vec<SkillId> {
    let rows_hold = |s: &SkillDef| round_intent::skill_rows(w, s).contains(&intent.id());
    let rest = rest_skill(w, c);
    let basics: Vec<&str> = w.units[c].meter_skills.iter().map(|id| w.cat.family_of(id)).collect();
    let mut out: Vec<SkillId> = Vec::new();
    let push = |sid: SkillId, out: &mut Vec<SkillId>| {
        if !out.contains(&sid) {
            out.push(sid);
        }
    };
    for f in families.iter().filter(|f| !basics.contains(&f.as_str())) {
        if let Some(sid) = crate::round_intent_plan::family_skill(w, c, f).filter(|&sid| sid != rest && only(&w.cat.skills[sid]) && rows_hold(&w.cat.skills[sid])) {
            push(sid, &mut out);
        }
    }
    for f in marked(w, c).iter().chain(class_basics(w, c).iter()) {
        if let Some(sid) = crate::round_intent_plan::family_skill(w, c, f).filter(|&sid| has_version(w, &w.cat.skills[sid]) && rows_hold(&w.cat.skills[sid])) {
            push(sid, &mut out);
        }
    }
    out
}
