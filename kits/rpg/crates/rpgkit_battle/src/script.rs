//! Scripted rounds (ENCOUNTER2-C section 4, docs/specs/ENCOUNTER2_C.md; engine-only, no Godot original): an encounter's
//! `script` plans both sides for up to three given rounds. A script is data that feeds the picks a player makes
//! (INPUT1's `pick`, `Session::pick_on` for either side) and the same resolver: AP, ranges, moves, Maneuver positions
//! and windups all apply, and nothing forces a hit or a roll. A locked round is confirmed at its start (the host
//! refuses planning requests); a guided round fixes the enemies' entries and hands the party's to the player as first
//! picks. An entry that cannot be planned gives one `Script` cell and its unit plays its own round (an enemy by the AI,
//! a hero by its auto plan). Inert unless the battle's ruleset has `round_intent` (classic logs one cell and plays
//! unscripted).

use crate::grid::V2;
use crate::plan::Plan;
use crate::round_intent::{ManeuverKind, Pick, RoundTarget};
use crate::round_intent_plan as rip;
use crate::session::Session;
use crate::world::UnitId;
use aegis_core::value;
use serde_json::{Map, Value};

/// SR1, 5.1: at most this many scripted rounds per encounter.
pub const MAX_SCRIPTED_ROUNDS: usize = 3;
/// 5.1: entries per round script.
pub const MAX_ENTRIES: usize = 16;
/// SN4: picks per entry.
pub const MAX_PICKS: usize = 4;
/// 5.1: a locked round's banner (characters).
pub const MAX_BANNER: usize = 40;
/// 5.1: the keys of a round script.
pub const ROUND_KEYS: [&str; 5] = ["round", "lock", "others", "banner", "entries"];
/// 5.1: the keys of an entry.
pub const ENTRY_KEYS: [&str; 4] = ["unit", "target", "picks", "fill"];
/// SR2, SR3: `others` and `fill`: the planner's plan (the default) or Rest.
pub const AUTO: &str = "auto";
pub const REST: &str = "rest";
pub const MODES: [&str; 2] = [AUTO, REST];
/// SN3: the default target.
pub const SELF_TARGET: &str = "self";
/// SN1: `party#n`, the n-th deployed hero.
pub const PARTY_INDEX: &str = "party#";
/// SN4: the pick shapes (one key each; a skill may add `kind`).
pub const PICK_KEYS: [&str; 4] = ["skill", "item", "move", "step_after"];
/// SN4: a skill pick's Maneuver kinds.
pub const KINDS: [&str; 3] = ["engage", "disengage", "utility"];
/// GH2 a: the tactic a hero with no entry (or a failed one) is drafted with.
pub const AUTO_PRESET: &str = "balanced";
/// SR8: the one cell under a ruleset without `round_intent`.
pub const CLASSIC_LINE: &str = "Scripted rounds need the round intent rules; they play unscripted.";
/// 5.1: a malformed pick list.
pub const PICKS_TEXT: &str = "Script picks are 1-4 of skill, item, rest, move, step_after.";

/// SR5: the refusal of a planning request in a locked round.
pub fn locked_text(round: i32) -> String {
    format!("Round {round} is scripted.")
}

/// SR11: the line of an entry that failed at planning time (a refusal's closing full stop dropped).
pub fn fallback_line(unit: &str, reason: &str) -> String {
    format!("Script: {unit} {}; it plays its own round.", reason.trim_end_matches('.'))
}

/// SR11, decision "ENCOUNTER2-C3 answers" (2): the line of a locked round whose plan could not be confirmed at its start
/// (it plays guided).
pub fn lock_fallback_line(round: i32, reason: &str) -> String {
    format!("Script: round {round} cannot be locked: {reason}; it plays as a guided round.")
}

/// SN3: an entry's round target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptTarget {
    Myself,
    /// A unit name (SN1).
    Unit(String),
    Cell(V2),
}

/// SN4: one pick as written (a skill by its family; the unit's own rank is used at planning time).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptPick {
    Skill { family: String, kind: Option<ManeuverKind> },
    Item(String),
    Rest,
    Move(V2),
    StepAfter(V2),
}

/// One unit's part of a round script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptEntry {
    pub unit: String,
    pub target: ScriptTarget,
    pub picks: Vec<ScriptPick>,
    /// SR3: true = the planner's fill after the picks, false = Rest.
    pub fill: bool,
}

/// SR1: one scripted round.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoundScript {
    pub round: i32,
    /// SR5 (true) or a guided round (SR6).
    pub lock: bool,
    /// SR2: the members with no entry Rest the whole round (false: their auto plan).
    pub others_rest: bool,
    pub banner: String,
    pub entries: Vec<ScriptEntry>,
}

/// An encounter's scripted rounds (empty = none).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Script {
    pub rounds: Vec<RoundScript>,
}

/// SN1: a unit name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitName {
    /// `ref` (n = 1) or `ref#n`: the n-th unit built from the encounter ref.
    Ref(String, usize),
    /// `party:<id>`: the hero whose hero key is `<id>`.
    Hero(String),
    /// `party#n`: the n-th deployed hero (1-4).
    HeroAt(usize),
}

impl UnitName {
    /// None: empty, or a `#` without a whole number from 1.
    pub fn parse(name: &str) -> Option<UnitName> {
        if let Some(id) = name.strip_prefix(crate::encounter::PARTY_PREFIX) {
            return (!id.is_empty()).then(|| UnitName::Hero(id.to_string()));
        }
        if let Some(n) = name.strip_prefix(PARTY_INDEX) {
            return n.parse::<usize>().ok().filter(|n| *n >= 1).map(UnitName::HeroAt);
        }
        match name.split_once('#') {
            Some((r, n)) => n.parse::<usize>().ok().filter(|n| *n >= 1 && !r.is_empty()).map(|n| UnitName::Ref(r.to_string(), n)),
            None => (!name.is_empty()).then(|| UnitName::Ref(name.to_string(), 1)),
        }
    }

    /// A party-side name (`party:<id>`, `party#n`).
    pub fn is_party(&self) -> bool {
        !matches!(self, UnitName::Ref(..))
    }
}

/// A whole number (JSON numbers may load as floats).
fn whole(v: &Value) -> Option<i64> {
    v.as_i64().or_else(|| v.as_f64().filter(|f| *f == f.floor()).map(|f| f as i64))
}

/// `[x, y]` of whole numbers.
pub fn parse_cell(v: &Value) -> Option<V2> {
    match v.as_array().map(|a| a.as_slice()) {
        Some([x, y]) => Some(V2::new(whole(x)? as i32, whole(y)? as i32)),
        _ => None,
    }
}

/// SN3: `"self"` (also absent), a unit name or `[x, y]`.
pub fn parse_target(v: Option<&Value>) -> Result<ScriptTarget, String> {
    match v {
        None => Ok(ScriptTarget::Myself),
        Some(Value::String(s)) if s == SELF_TARGET => Ok(ScriptTarget::Myself),
        Some(Value::String(s)) if !s.is_empty() => Ok(ScriptTarget::Unit(s.clone())),
        Some(c) => parse_cell(c).map(ScriptTarget::Cell).ok_or_else(|| "A script target is \"self\", a unit or [x, y].".to_string()),
    }
}

/// SN4: one pick (`PICKS_TEXT` when the shape is none of them).
pub fn parse_pick(v: &Value) -> Result<ScriptPick, String> {
    let bad = || PICKS_TEXT.to_string();
    if v.as_str() == Some(REST) {
        return Ok(ScriptPick::Rest);
    }
    let o = v.as_object().ok_or_else(bad)?;
    let shapes: Vec<&str> = PICK_KEYS.iter().copied().filter(|k| o.contains_key(*k)).collect();
    let [shape] = shapes.as_slice() else { return Err(bad()) };
    let extra = o.keys().any(|k| k != shape && !(*shape == "skill" && k == "kind"));
    if extra {
        return Err(bad());
    }
    let text = |k: &str| o.get(k).and_then(Value::as_str).filter(|s| !s.is_empty()).map(str::to_string);
    match *shape {
        "skill" => {
            let family = text("skill").ok_or_else(bad)?;
            let kind = match o.get("kind") {
                None => None,
                Some(k) => Some(match k.as_str() {
                    Some("engage") => ManeuverKind::Engage,
                    Some("disengage") => ManeuverKind::Disengage,
                    Some("utility") => ManeuverKind::Utility,
                    _ => return Err(bad()),
                }),
            };
            Ok(ScriptPick::Skill { family, kind })
        }
        "item" => text("item").map(ScriptPick::Item).ok_or_else(bad),
        "move" => o.get("move").and_then(parse_cell).map(ScriptPick::Move).ok_or_else(bad),
        _ => o.get("step_after").and_then(parse_cell).map(ScriptPick::StepAfter).ok_or_else(bad),
    }
}

/// SR3, SR2: `auto` (also absent) or `rest`; None for anything else.
fn mode(o: &Map<String, Value>, key: &str) -> Option<bool> {
    match o.get(key) {
        None => Some(false),
        Some(Value::String(s)) if s == AUTO => Some(false),
        Some(Value::String(s)) if s == REST => Some(true),
        _ => None,
    }
}

impl ScriptEntry {
    /// An entry as written (Err: a shape the check refuses).
    pub fn read(o: &Map<String, Value>) -> Result<ScriptEntry, String> {
        let unit = o.get("unit").and_then(Value::as_str).unwrap_or("").to_string();
        let target = parse_target(o.get("target"))?;
        let raw = o.get("picks").and_then(Value::as_array).filter(|a| (1..=MAX_PICKS).contains(&a.len())).ok_or_else(|| PICKS_TEXT.to_string())?;
        let picks = raw.iter().map(parse_pick).collect::<Result<Vec<_>, _>>()?;
        let rest = mode(o, "fill").ok_or_else(|| "Script fill is auto or rest.".to_string())?;
        Ok(ScriptEntry { unit, target, picks, fill: !rest })
    }
}

impl Script {
    /// The encounter row's `script` (lenient: what does not parse is left out; the content check reports it).
    pub fn read(raw: &Map<String, Value>) -> Script {
        let mut rounds: Vec<RoundScript> = Vec::new();
        for o in value::array(raw, "script").iter().filter_map(Value::as_object) {
            let round = value::int(o, "round", 0);
            if round < 1 || rounds.iter().any(|r| r.round == round) {
                continue;
            }
            let entries = value::array(o, "entries").iter().filter_map(Value::as_object).filter_map(|e| ScriptEntry::read(e).ok()).collect();
            rounds.push(RoundScript { round, lock: value::boolean(o, "lock", true), others_rest: mode(o, "others") == Some(true), banner: value::string(o, "banner", ""), entries });
            if rounds.len() == MAX_SCRIPTED_ROUNDS {
                break;
            }
        }
        Script { rounds }
    }

    pub fn is_empty(&self) -> bool {
        self.rounds.is_empty()
    }

    /// The script of `round` (None: it plays as today).
    pub fn round(&self, round: i32) -> Option<&RoundScript> {
        self.rounds.iter().find(|r| r.round == round)
    }
}

// ---- Planning (SR2-SR6, SR11) ----------------------------------------------------------------------------------

/// The heroes in party order (guests aside): `party#n` counts them.
fn heroes(s: &Session) -> Vec<UnitId> {
    s.w.allies.iter().copied().filter(|c| !s.guests.contains(c)).collect()
}

/// SN1: the unit a name names in this battle (None: no such unit was built).
pub fn unit_of(s: &Session, name: &str) -> Option<UnitId> {
    match UnitName::parse(name)? {
        UnitName::Ref(r, n) => s.units_of_ref(&r).get(n - 1).copied(),
        UnitName::Hero(id) => s.units_named(&format!("{}{id}", crate::encounter::PARTY_PREFIX)).first().copied(),
        UnitName::HeroAt(n) => heroes(s).get(n - 1).copied(),
    }
}

/// SR11: on the field (active, placed, not a reserved phase wave).
pub fn on_field(s: &Session, c: UnitId) -> bool {
    s.w.units[c].is_active() && s.w.has(c) && !s.is_reserved(c)
}

/// What a side's entries came to: the units now holding their entry's picks, and each failure (the unit when it was
/// built, the name shown, the reason) in entry order.
#[derive(Debug, Default)]
pub(crate) struct Planned {
    pub units: Vec<UnitId>,
    pub failures: Vec<(Option<UnitId>, String, String)>,
}

/// Whether entry `e` is on the enemy side (by its unit; a name that built nothing by its shape).
fn enemy_entry(s: &Session, e: &ScriptEntry) -> bool {
    match unit_of(s, &e.unit) {
        Some(c) => s.w.enemies.contains(&c),
        None => !UnitName::parse(&e.unit).is_some_and(|n| n.is_party()),
    }
}

/// SR2, SR4, SR11: plans the entries of one side (`enemies`) of `rs` in list order into `plan`, each through the same
/// pick a player makes (`Session::pick_on`), its fill set by the entry (SR3). An entry fails (and its unit keeps no
/// pick) when its unit or a unit target is not on the field, a family is not the unit's, or its picks are refused; a
/// later member's rebuild that drops a planned entry's pick fails that entry too.
pub(crate) fn plan_entries(s: &mut Session, rs: &RoundScript, enemies: bool, plan: &mut Plan) -> Planned {
    let mut out = Planned::default();
    let mut dropped: Vec<(UnitId, String)> = Vec::new();
    let side: Vec<&ScriptEntry> = rs.entries.iter().filter(|e| enemy_entry(s, e) == enemies).collect();
    for e in side {
        let Some(c) = unit_of(s, &e.unit) else {
            out.failures.push((None, e.unit.clone(), "is not in this battle".into()));
            continue;
        };
        if out.units.contains(&c) || out.failures.iter().any(|f| f.0 == Some(c)) {
            continue;
        }
        match plan_entry(s, e, c, plan) {
            Ok(later) => {
                out.units.push(c);
                dropped.extend(later);
            }
            Err(why) => out.failures.push((Some(c), s.w.units[c].name.clone(), why)),
        }
    }
    // GH5: a later member's rebuild dropped a pick of an entry planned before it: that entry failed
    let wanted: Vec<(UnitId, usize)> = out.units.iter().map(|&c| (c, rs.entries.iter().find(|e| unit_of(s, &e.unit) == Some(c)).map_or(0, |e| e.picks.len()))).collect();
    for (c, n) in wanted {
        if rip::picks_of(&s.w, c).len() == n {
            continue;
        }
        let why = dropped.iter().rev().find(|d| d.0 == c).map_or_else(String::new, |d| d.1.clone());
        let _ = rip::clear_picks(&mut s.w, plan, c);
        s.w.round_states.entry(c).or_default().fill = None;
        out.units.retain(|&u| u != c);
        out.failures.push((Some(c), s.w.units[c].name.clone(), format!("cannot plan its picks: {why}")));
    }
    out
}

/// One entry of unit `c`: its target and picks resolved, then picked with its fill (Err: SR11's reason; nothing of it
/// stays planned). Ok: the later members' dropped picks (GH5).
fn plan_entry(s: &mut Session, e: &ScriptEntry, c: UnitId, plan: &mut Plan) -> Result<Vec<(UnitId, String)>, String> {
    if !on_field(s, c) {
        return Err("is not on the field".into());
    }
    let at = match &e.target {
        ScriptTarget::Myself => RoundTarget::Myself,
        ScriptTarget::Cell(p) => RoundTarget::Cell(*p),
        ScriptTarget::Unit(t) => match unit_of(s, t).filter(|&u| on_field(s, u)) {
            Some(u) => RoundTarget::Unit(u),
            None => return Err(format!("has no target: {t} is not on the field")),
        },
    };
    let mut picks: Vec<Pick> = Vec::new();
    for p in &e.picks {
        picks.push(match p {
            ScriptPick::Skill { family, kind } => match rip::family_skill(&s.w, c, family) {
                Some(skill) => Pick::Skill { skill, kind: *kind },
                None => return Err(format!("has no skill '{family}'")),
            },
            ScriptPick::Item(id) => Pick::Item(id.clone()),
            ScriptPick::Rest => Pick::Rest,
            ScriptPick::Move(p) => Pick::MoveHere(*p),
            ScriptPick::StepAfter(p) => Pick::StepAfter(*p),
        });
    }
    s.w.round_states.entry(c).or_default().fill = Some(e.fill);
    let (done, later) = crate::session::pick_on(&mut s.w, plan, &mut s.events, c, at, &picks);
    match done {
        Ok(()) => Ok(later),
        Err(why) => {
            s.w.round_states.entry(c).or_default().fill = None;
            Err(format!("cannot plan its picks: {why}"))
        }
    }
}

/// SR2 `others: rest`: every party-side unit on the field with no entry and no failed one Rests the whole round (one
/// Rest pick with the Rest fill).
pub(crate) fn rest_others(s: &mut Session, planned: &Planned, plan: &mut Plan) {
    for c in s.w.allies.clone() {
        if !on_field(s, c) || planned.units.contains(&c) || planned.failures.iter().any(|f| f.0 == Some(c)) {
            continue;
        }
        s.w.round_states.entry(c).or_default().fill = Some(false);
        if crate::session::pick_on(&mut s.w, plan, &mut s.events, c, RoundTarget::Myself, &[Pick::Rest]).0.is_err() {
            s.w.round_states.entry(c).or_default().fill = None;
        }
    }
}

/// A replanned round starts its side again: no pick and no scripted fill on `side`.
pub(crate) fn clear_side(s: &mut Session, side: &[UnitId]) {
    for c in side {
        if let Some(rs) = s.w.round_states.get_mut(c) {
            rs.picks.clear();
            rs.fill = None;
        }
    }
}
