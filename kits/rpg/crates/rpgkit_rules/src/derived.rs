//! Derived stats (DERIVE1 4.2-4.3, docs/specs/DERIVE1.md; engine-only, no Godot original): the value of a catalog
//! derived-stat row for a unit, its breakdown for tooltips, and the resist chance and draw of a check against it.
//! Values are worked out on read and never stored on a unit or a save; nothing here draws from an RNG unless a
//! consumer calls `lands` (the classic rules read no derived stat).

use crate::gear::TIER_SOURCE;
use crate::ruleset::Ruleset;
use crate::stats::roundi;
use crate::unit::Unit;
use aegis_core::rng::GodotRng;
use indexmap::IndexMap;
use rpgkit_data::{Catalog, DerivedStatDef, GEAR_MITIGATION_INPUT, LEVEL_INPUT, STATS};
use serde_json::Value;

/// The per-save roll stream of resist checks out of battle (`rolls::chance(gs, RESIST_STREAM, chance)`).
pub const RESIST_STREAM: &str = "resist";
/// The potency range of the reserved consumer shape `{"resist": "<derived id>", "potency": n}`.
pub const POTENCY_MIN: i32 = 1;
pub const POTENCY_MAX: i32 = 200;
/// Breakdown source prefix of a row's `status_modifiers` entry (`status:<status id>`).
pub const STATUS_SOURCE: &str = "status";
/// StatModifier source of item stat bonuses (`Unit::sync_from_stats_and_gear`).
const GEAR_SOURCE: &str = "gear";

/// What a formula name stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    /// One of the six base stats, effective in the unit's weapon context (`CharacterStats::stat`).
    Stat,
    /// The unit level.
    Level,
    /// RULESET1 P1: the summed `effective_mitigation` of the unit's worn gear (`Inventory::total_flat_mitigation`).
    Gear,
    /// A key of the row's `constants`.
    Constant,
    /// Another derived row (its final value).
    Derived,
    /// A name no input answers (only in unchecked catalogs): reads as 0.
    Unknown,
}

/// One formula input with the value it had.
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    pub name: String,
    pub kind: InputKind,
    pub value: f64,
}

/// One flat and percent change. `source` is the modifier source (`gear:<item id>`, `gear_tier:<item id>`,
/// `passive:<passive id>`, `passive:gear:<item id>` for item effects) or `status:<status id>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub source: String,
    pub flat: i32,
    pub percent: f64,
}

/// Which cap the value hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cap {
    Min,
    Max,
}

/// How a derived value came about (DERIVE1 4.2), for the tooltips and the Dojo.
#[derive(Debug, Clone, PartialEq)]
pub struct Breakdown {
    pub id: String,
    /// The formula as written.
    pub formula: String,
    /// Every name the formula uses, first appearance order.
    pub inputs: Vec<Input>,
    /// The formula's result before modifiers.
    pub raw: f64,
    pub parts: Vec<Part>,
    /// Σ flat and Σ percent of `parts`.
    pub flat: i32,
    pub percent: f64,
    /// `roundi((raw + flat) × (1 + percent))` before the caps.
    pub uncapped: i32,
    pub min: i32,
    pub max: i32,
    pub cap: Option<Cap>,
    pub value: i32,
}

/// The row `id` when it exists in `ruleset` (a row whose `rulesets` leaves the ruleset out reads as absent).
pub fn row<'a>(cat: &'a Catalog, ruleset: &str, id: &str) -> Option<&'a DerivedStatDef> {
    cat.derived_stats.get(id).filter(|d| d.in_ruleset(ruleset))
}

/// The value of derived stat `id` for `unit`: `clamp(roundi((formula + flat) × (1 + percent)), min, max)`.
pub fn value(cat: &Catalog, ruleset: &str, unit: &Unit, id: &str) -> Option<i32> {
    let d = row(cat, ruleset, id)?;
    Some(Eval::new(cat, unit).value_of(d))
}

/// The value of `id` with every input, modifier part and cap that made it.
pub fn breakdown(cat: &Catalog, ruleset: &str, unit: &Unit, id: &str) -> Option<Breakdown> {
    let d = row(cat, ruleset, id)?;
    let mut ev = Eval::new(cat, unit);
    let inputs: Vec<Input> = d
        .formula
        .names()
        .iter()
        .map(|n| {
            let (kind, value) = ev.input(d, n);
            Input { name: n.clone(), kind, value }
        })
        .collect();
    let raw = ev.raw(d);
    let parts = modifier_parts(unit, d, ev.weapon.as_deref());
    let (flat, percent) = totals(&parts);
    let uncapped = uncapped(raw, flat, percent);
    let value = capped(uncapped, d);
    let cap = if uncapped < d.min && value == d.min {
        Some(Cap::Min)
    } else if uncapped > d.max && value == d.max {
        Some(Cap::Max)
    } else {
        None
    };
    let parts = by_item(unit, &d.id, parts);
    Some(Breakdown { id: d.id.clone(), formula: d.formula.source().to_string(), inputs, raw, parts, flat, percent, uncapped, min: d.min, max: d.max, cap, value })
}

/// The value of `id` for `unit` without the modifier parts of source kind `source` (a part whose source is `source` or
/// starts with `<source>:`; e.g. `STATUS_SOURCE`: the value without the row's status modifiers), by the same rule and
/// caps as `value`; the formula's own inputs (other rows included) stay as they are. None when the row is absent in
/// `ruleset`. PORTRAITFX1 5.2 reads it for a cue row's `drop_pct` ("shaken": a drop from the value without statuses).
pub fn value_without(cat: &Catalog, ruleset: &str, unit: &Unit, id: &str, source: &str) -> Option<i32> {
    let d = row(cat, ruleset, id)?;
    let mut ev = Eval::new(cat, unit);
    let raw = ev.raw(d);
    let kept: Vec<Part> = modifier_parts(unit, d, ev.weapon.as_deref()).into_iter().filter(|p| !(p.source == source || p.source.strip_prefix(source).is_some_and(|r| r.starts_with(':')))).collect();
    let (flat, percent) = totals(&kept);
    Some(capped(uncapped(raw, flat, percent), d))
}

/// The chance (0-100) an effect of `potency` lands on `unit` against derived stat `id`: `clamp(potency − value, 0,
/// 100)`; an absent row (not in the catalog or not in the ruleset) gives the potency itself, clamped.
pub fn resist_chance(cat: &Catalog, ruleset: &str, unit: &Unit, id: &str, potency: i32) -> i32 {
    match value(cat, ruleset, unit, id) {
        Some(v) => (potency - v).clamp(0, 100),
        None => potency.clamp(0, 100),
    }
}

/// One battle draw of a check: `randi_range(1, 100) <= chance` (the debuff roll's shape, `status::try_inflict`).
pub fn roll(rng: &mut GodotRng, chance: i32) -> bool {
    rng.randi_range(1, 100) <= chance
}

/// A battle resist check as it resolves: a fallen target draws nothing and the effect does not land; a living one
/// draws exactly once from the session RNG, whatever the chance.
pub fn lands(cat: &Catalog, ruleset: &str, unit: &Unit, check: &ResistCheck, rng: &mut GodotRng) -> bool {
    if !unit.is_alive() {
        return false;
    }
    roll(rng, resist_chance(cat, ruleset, unit, &check.resist, check.potency))
}

// ---- The same reads by a battle's ruleset (RULESET1-P: `World.ruleset`, `EncounterDef.ruleset`; the `&str` above is
// its id) ------------------------------------------------------------------------------------------------------------

/// `row` in `rules`.
pub fn row_for<'a>(cat: &'a Catalog, rules: &Ruleset, id: &str) -> Option<&'a DerivedStatDef> {
    row(cat, &rules.id, id)
}

/// `value` in `rules`.
pub fn value_for(cat: &Catalog, rules: &Ruleset, unit: &Unit, id: &str) -> Option<i32> {
    value(cat, &rules.id, unit, id)
}

/// `breakdown` in `rules`.
pub fn breakdown_for(cat: &Catalog, rules: &Ruleset, unit: &Unit, id: &str) -> Option<Breakdown> {
    breakdown(cat, &rules.id, unit, id)
}

/// `resist_chance` in `rules`.
pub fn resist_chance_for(cat: &Catalog, rules: &Ruleset, unit: &Unit, id: &str, potency: i32) -> i32 {
    resist_chance(cat, &rules.id, unit, id, potency)
}

/// `lands` in `rules`.
pub fn lands_for(cat: &Catalog, rules: &Ruleset, unit: &Unit, check: &ResistCheck, rng: &mut GodotRng) -> bool {
    lands(cat, &rules.id, unit, check, rng)
}

/// The reserved consumer shape `{"resist": "<derived id>", "potency": 1-200}` (hazards, nightmares, quirks).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResistCheck {
    pub resist: String,
    pub potency: i32,
}

impl ResistCheck {
    /// The check of a value that `check_errors` passes; None otherwise.
    pub fn from_value(v: &Value) -> Option<Self> {
        let o = v.as_object()?;
        let resist = o.get("resist")?.as_str()?.to_string();
        let potency = whole(o.get("potency")?)?;
        (!resist.is_empty() && (POTENCY_MIN..=POTENCY_MAX).contains(&potency)).then_some(Self { resist, potency })
    }
}

fn whole(v: &Value) -> Option<i32> {
    v.as_f64().filter(|x| x.is_finite() && x.fract() == 0.0 && x.abs() <= i32::MAX as f64).map(|x| x as i32)
}

/// The problems of a consumer's resist check against the catalog's derived ids (exact lowercase ids); every consumer
/// validates the shape through this one function. Other keys of the object belong to the consumer and are not read.
pub fn check_errors(v: &Value, derived: &[String]) -> Vec<String> {
    let Some(o) = v.as_object() else {
        return vec![format!("Resist check must be an object {{\"resist\": <derived stat id>, \"potency\": {POTENCY_MIN}-{POTENCY_MAX}}}.")];
    };
    let mut out = Vec::new();
    match o.get("resist").map(|r| r.as_str()) {
        None | Some(Some("")) => out.push("Resist: a derived stat id is required.".to_string()),
        Some(None) => out.push("Resist must be a derived stat id (text).".to_string()),
        Some(Some(id)) if !derived.iter().any(|d| d == id) => out.push(format!("Resist: '{id}' is not a derived stat.")),
        Some(Some(_)) => {}
    }
    if !o.get("potency").and_then(whole).is_some_and(|p| (POTENCY_MIN..=POTENCY_MAX).contains(&p)) {
        out.push(format!("Potency must be a whole number {POTENCY_MIN}-{POTENCY_MAX}."));
    }
    out
}

/// One read of a unit's derived values: every row is worked out at most once (inputs naming other rows).
struct Eval<'a> {
    cat: &'a Catalog,
    unit: &'a Unit,
    weapon: Option<String>,
    done: IndexMap<String, i32>,
    busy: Vec<String>,
}

impl<'a> Eval<'a> {
    fn new(cat: &'a Catalog, unit: &'a Unit) -> Self {
        Self { cat, unit, weapon: unit.weapon_context(), done: IndexMap::new(), busy: Vec::new() }
    }

    /// A formula name: a base stat, `level`, `gear_mitigation`, a constant of the row, another row (computed whatever
    /// its rulesets), or 0.
    fn input(&mut self, d: &DerivedStatDef, name: &str) -> (InputKind, f64) {
        if STATS.contains(&name) {
            return (InputKind::Stat, self.unit.stats.stat(name, self.weapon.as_deref()) as f64);
        }
        if name == LEVEL_INPUT {
            return (InputKind::Level, self.unit.stats.level as f64);
        }
        if name == GEAR_MITIGATION_INPUT {
            return (InputKind::Gear, self.unit.inventory.total_flat_mitigation(Some(&self.unit.stats)) as f64);
        }
        if let Some(c) = d.constants.get(name) {
            return (InputKind::Constant, *c);
        }
        let cat = self.cat;
        match cat.derived_stats.get(name) {
            Some(other) => (InputKind::Derived, self.value_of(other) as f64),
            None => (InputKind::Unknown, 0.0),
        }
    }

    fn raw(&mut self, d: &DerivedStatDef) -> f64 {
        d.formula.eval(&mut |n| self.input(d, n).1)
    }

    fn value_of(&mut self, d: &DerivedStatDef) -> i32 {
        if let Some(v) = self.done.get(&d.id) {
            return *v;
        }
        if self.busy.contains(&d.id) {
            // a cycle (only in unchecked catalogs: the loader drops rows in one)
            return 0;
        }
        self.busy.push(d.id.clone());
        let raw = self.raw(d);
        let (flat, percent) = totals(&modifier_parts(self.unit, d, self.weapon.as_deref()));
        let v = capped(uncapped(raw, flat, percent), d);
        self.busy.pop();
        self.done.insert(d.id.clone(), v);
        v
    }
}

/// The unit's modifiers on the id (any case: `StatModifier::permanent` upper-cases), matching the weapon context,
/// in modifier order, then the row's status modifiers of active statuses in row order.
fn modifier_parts(unit: &Unit, d: &DerivedStatDef, weapon: Option<&str>) -> Vec<Part> {
    let mut parts: Vec<Part> = unit
        .stats
        .modifiers
        .iter()
        .filter(|m| m.stat.eq_ignore_ascii_case(&d.id) && m.matches(weapon))
        .map(|m| Part { source: m.source.clone(), flat: m.flat, percent: m.percent })
        .collect();
    for sm in &d.status_modifiers {
        if unit.has_status(&sm.status) {
            parts.push(Part { source: format!("{STATUS_SOURCE}:{}", sm.status), flat: sm.flat, percent: sm.percent });
        }
    }
    parts
}

fn totals(parts: &[Part]) -> (i32, f64) {
    let mut flat = 0;
    let mut percent = 0.0;
    for p in parts {
        flat += p.flat;
        percent += p.percent;
    }
    (flat, percent)
}

/// The order of `CharacterStats::effective`: flat first, then percent, then `roundi`.
fn uncapped(raw: f64, flat: i32, percent: f64) -> i32 {
    roundi((raw + flat as f64) * (1.0 + percent))
}

/// The row's caps (min first, so a row with min above max, which the checks reject, reads as max and never panics).
fn capped(v: i32, d: &DerivedStatDef) -> i32 {
    v.max(d.min).min(d.max)
}

/// Splits the summed `gear` and `gear_tier` parts by item (`gear:<item id>`, `gear_tier:<item id>`) when the worn
/// items still add up to them; otherwise the summed part stays as it is.
fn by_item(unit: &Unit, id: &str, parts: Vec<Part>) -> Vec<Part> {
    let up = id.to_uppercase();
    let split = |source: &str| -> Vec<Part> {
        unit.inventory
            .gear()
            .map(|e| {
                let flat = if source == GEAR_SOURCE {
                    e.item.stat_bonuses.iter().filter(|(k, _)| k.eq_ignore_ascii_case(id)).map(|(_, v)| *v).sum()
                } else {
                    e.tier_stat_bonuses(&unit.stats).get(&up).copied().unwrap_or(0)
                };
                Part { source: format!("{source}:{}", e.id()), flat, percent: 0.0 }
            })
            .filter(|p| p.flat != 0)
            .collect()
    };
    let mut out = Vec::new();
    for source in [GEAR_SOURCE, TIER_SOURCE] {
        let summed: Vec<&Part> = parts.iter().filter(|p| p.source == source).collect();
        if summed.is_empty() {
            continue;
        }
        let items = split(source);
        let fits = summed.iter().all(|p| p.percent == 0.0) && items.iter().map(|p| p.flat).sum::<i32>() == summed.iter().map(|p| p.flat).sum::<i32>();
        out.push((source, if fits { items } else { summed.into_iter().cloned().collect() }));
    }
    let mut result = Vec::new();
    for p in parts {
        match out.iter_mut().find(|(s, _)| *s == p.source) {
            Some((_, items)) => result.append(items),
            None => result.push(p),
        }
    }
    result
}
