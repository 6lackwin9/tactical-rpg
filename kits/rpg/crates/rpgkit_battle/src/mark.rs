//! Battle marks (PK1-S1, docs/sessions/PK1_CORRUPTION.md 3.5 and 3.7; engine-only, no Godot original). The trigger
//! effect `mark` `{mark, value}` records an entry on a unit in battle state (`World::marks`, no save key); the battle
//! result carries each fielded member's entries (`rpgkit_world::roster::HeroEnd::marks`) for the world settle to read
//! (PK2 consumes them). A consumable's `unmark` `{mark}` removes the target's latest entry of that id; when that target
//! is a unit that started the battle on the party side and now fights everyone (a `hostile_all` rogue, `World.sides`),
//! it is also calmed: at the next round start it returns to the party side at its battle-start place among the allies
//! (`World::rejoin_party`). No data uses either yet, so every battle keeps its marks empty and logs nothing new.
//!
//! PK2-S1 (`docs/sessions/PK2_GRAFTS.md` 2, PK1 3.5): a mark whose id is a unit track row's `mark` also moves that
//! track on a unit that holds tracks (a party member, `Unit::tracks`) at once, for the rest of the battle
//! (`sync_tracks`). The in-battle stage is always the settle's formula: the battle-start stage plus the unit's current
//! marks of the row's mark id, held within the row, so a mark raises it and an `unmark` lowers it again. Each stage
//! reached grants its slotless passive now (a battle never takes a passive back), and reaching a stage with a
//! departure turns the unit on everyone (`World::set_side` `hostile_all`, PHASEFX1's mechanic). An `unmark` that takes
//! a track-turned unit below that stage calms it as on any turned ally; one that leaves it at the stage does not. The
//! saved stage moves only at the settle (`rpgkit_world::roster`). A SUPPORT skill's `unmark` is the consumable's effect
//! as a skill; `uses_per_battle` limits how often a unit uses a skill in one battle (`World::skill_uses`).

use crate::world::{UnitId, World};

/// One recorded mark: the unit, its id and value, in the order recorded.
#[derive(Debug, Clone, PartialEq)]
pub struct Mark {
    pub unit: UnitId,
    pub mark: String,
    pub value: i32,
}

/// The battle's marks, the units to calm at the next round start (with the source that calmed them) and the units on
/// the party side at the battle start (in their order then).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Marks {
    pub list: Vec<Mark>,
    calm: Vec<(UnitId, String)>,
    start_party: Vec<UnitId>,
    /// PK2-S1: the units a unit track's departure stage turned on everyone, with that track's id (in turn order).
    turned: Vec<(UnitId, String)>,
    /// PK2-S1: each (unit, track)'s stage when the battle first moved it (its battle-start stage).
    track_start: Vec<(UnitId, String, i32)>,
}

impl Marks {
    /// No marks; `start_party` the party side as the battle starts.
    pub fn new(start_party: &[UnitId]) -> Self {
        Self { list: Vec::new(), calm: Vec::new(), start_party: start_party.to_vec(), turned: Vec::new(), track_start: Vec::new() }
    }

    /// `c`'s marks (id, value) in the order recorded.
    pub fn of(&self, c: UnitId) -> Vec<(String, i32)> {
        self.list.iter().filter(|m| m.unit == c).map(|m| (m.mark.clone(), m.value)).collect()
    }

    pub fn record(&mut self, c: UnitId, mark: &str, value: i32) {
        self.list.push(Mark { unit: c, mark: mark.to_string(), value });
    }

    /// Removes `c`'s latest mark `mark` (None when it has none).
    pub fn remove_latest(&mut self, c: UnitId, mark: &str) -> Option<Mark> {
        let i = self.list.iter().rposition(|m| m.unit == c && m.mark == mark)?;
        Some(self.list.remove(i))
    }

    /// Whether `c` was on the party side when the battle started.
    pub fn started_on_party(&self, c: UnitId) -> bool {
        self.start_party.contains(&c)
    }

    /// The battle-start party side, in order.
    pub fn start_party(&self) -> &[UnitId] {
        &self.start_party
    }

    /// Whether `c` returns to the party side at the next round start.
    pub fn calming(&self, c: UnitId) -> bool {
        self.calm.iter().any(|(u, _)| *u == c)
    }

    /// PK2-S1: the unit track whose departure stage turned `c` on everyone (its latest turn; None = none did).
    pub fn turned_by(&self, c: UnitId) -> Option<&str> {
        self.turned.iter().rev().find(|(u, _)| *u == c).map(|(_, t)| t.as_str())
    }

    /// PK2-S1: `c`'s battle-start stage on track `track` (None = the battle has not moved it yet).
    pub fn track_start(&self, c: UnitId, track: &str) -> Option<i32> {
        self.track_start.iter().find(|(u, t, _)| *u == c && t == track).map(|(_, _, s)| *s)
    }

    /// The sum of `c`'s current marks `mark`.
    pub fn sum_of(&self, c: UnitId, mark: &str) -> i64 {
        self.list.iter().filter(|m| m.unit == c && m.mark == mark).map(|m| i64::from(m.value)).sum()
    }
}

/// PK2-S1 (FB-174 (5)): the unit track that turned `c` on everyone when `c` still fights everyone at the battle end
/// (a rogue no `unmark` is calming); "" otherwise. The settle departs such a member (`rpgkit_world::roster`).
pub fn hostile_at_end(w: &World, c: UnitId) -> String {
    if !w.sides.is_rogue(c) || w.marks.calming(c) {
        return String::new();
    }
    w.marks.turned_by(c).unwrap_or_default().to_string()
}

/// Whether an `unmark` may reach `c` although it fights the user: `c` started the battle on the party side and is now
/// a `hostile_all` rogue (3.7: "it was an ally at the battle's start").
pub fn turned_ally(w: &World, c: UnitId) -> bool {
    w.sides.is_rogue(c) && w.marks.started_on_party(c)
}

/// PK1-S1 (3.7), PR5-25: whether an `unmark` of `mark` ("" = the use has none) used by `user` may reach `target`
/// although it fights the user: the user is on the party side and the target a turned ally (`turned_ally`). The same
/// rule for a consumable's `unmark` and a SUPPORT skill's.
pub fn unmark_reaches(w: &World, user: UnitId, mark: &str, target: UnitId) -> bool {
    !mark.is_empty() && w.allies.contains(&user) && turned_ally(w, target)
}

/// PR5-25: the SUPPORT skills with an `unmark` that `c` can use now (catalog order): known at its skill cap and not
/// used up this battle (`uses_problem`, nothing planned counted).
pub fn unmark_skills(w: &World, c: UnitId) -> Vec<crate::skill::SkillId> {
    let cap = w.skill_cap_of(c);
    w.book().available(crate::skill::Kind::Support, cap, Some(&w.units[c])).into_iter().filter(|&sid| !w.cat.skills[sid].unmark.is_empty() && uses_problem(w, c, &w.cat.skills[sid], 0).is_empty()).collect()
}

/// PR5-25 (rule round_intent): whether a Support round of `c` may take the turned ally `t` as its round target: an
/// `unmark` reaches it (`unmark_reaches`) through a skill `c` can use (`unmark_skills`) or an offered pouch item with
/// one. Any other step of that round still needs a friend (its own side check).
pub fn unmark_round_target(w: &World, c: UnitId, t: UnitId) -> bool {
    if !w.allies.contains(&c) || !turned_ally(w, t) {
        return false;
    }
    let item = w.units[c].inventory.pouch.iter().any(|i| !i.unmark.is_empty() && i.offered_in_battle());
    item || !unmark_skills(w, c).is_empty()
}

/// PR5-25 (the AI's read of `unmark`): whether an `unmark` of `mark` on `c` now would calm it: `c` is a turned ally
/// not already calming that holds a `mark` entry, and the stage its turning track (if one turned it) would be at once
/// the latest entry goes (`sync_tracks`' formula) is no departure stage. Reads only; draws nothing.
pub fn would_calm(w: &World, c: UnitId, mark: &str) -> bool {
    if mark.is_empty() || !turned_ally(w, c) || w.marks.calming(c) {
        return false;
    }
    let Some(latest) = w.marks.list.iter().rev().find(|m| m.unit == c && m.mark == mark) else { return false };
    let Some(t) = w.marks.turned_by(c).and_then(|id| w.cat.unit_tracks.get(id)) else { return true };
    let now = w.units[c].tracks.as_ref().and_then(|s| s.get(&t.id).copied()).unwrap_or(0);
    let after = if t.mark == mark {
        let start = w.marks.track_start(c, &t.id).unwrap_or(now);
        t.clamp(i64::from(start) + w.marks.sum_of(c, mark) - i64::from(latest.value))
    } else {
        now
    };
    t.departure_at(after).is_empty()
}

/// 3.7: an `unmark` of `mark` by `source` on `c`: its latest `mark` entry goes (PK2-S1: the unit tracks it moved move
/// back, `sync_tracks`); on a turned ally (`turned_ally`) it is also calmed at the next round start, unless a unit
/// track turned it and it is still at that track's departure stage. Returns the log part ("" = nothing to undo).
pub fn unmark(w: &mut World, c: UnitId, mark: &str, source: &str) -> String {
    if mark.is_empty() || w.marks.remove_latest(c, mark).is_none() {
        return String::new();
    }
    let name = w.units[c].name.clone();
    let mut out = format!("{name}'s {mark} undone");
    for part in sync_tracks(w, c, mark) {
        out = format!("{out}; {part}");
    }
    if turned_ally(w, c) && !w.marks.calming(c) && !at_turning_stage(w, c) {
        w.marks.calm.push((c, source.to_string()));
        out = format!("{out}; {name} calms at the next round start");
    }
    out
}

/// PK2-S1: whether the unit track that turned `c` (if one did) still holds it at a departure stage.
fn at_turning_stage(w: &World, c: UnitId) -> bool {
    let Some(t) = w.marks.turned_by(c).and_then(|id| w.cat.unit_tracks.get(id)) else { return false };
    let stage = w.units[c].tracks.as_ref().and_then(|s| s.get(&t.id).copied()).unwrap_or(0);
    !t.departure_at(stage).is_empty()
}

/// PK2-S1: re-reads every unit track row whose `mark` is `mark` on `c` after its marks changed, on a unit that holds
/// tracks (`Unit::tracks`; anyone else only keeps the mark). The stage is the battle-start stage plus `c`'s current
/// marks `mark`, held within the row (0..its top), the settle's formula. Each stage newly reached grants its passive now
/// (`Unit::grant_passive_now`, once); reaching a departure stage turns a party-side unit on everyone (`World::set_side`
/// `hostile_all`) and cancels a pending calm of a unit still turned. Returns the log parts (none when nothing moved).
pub fn sync_tracks(w: &mut World, c: UnitId, mark: &str) -> Vec<String> {
    if mark.is_empty() || w.units[c].tracks.is_none() {
        return Vec::new();
    }
    let cat = w.cat.clone();
    let sum = w.marks.sum_of(c, mark);
    let mut parts = Vec::new();
    for t in cat.unit_tracks.values().filter(|t| t.mark == mark) {
        let name = w.units[c].name.clone();
        let old = w.units[c].tracks.as_ref().map_or(0, |s| t.clamp(i64::from(s.get(&t.id).copied().unwrap_or(0))));
        let start = match w.marks.track_start(c, &t.id) {
            Some(s) => s,
            None => {
                w.marks.track_start.push((c, t.id.clone(), old));
                old
            }
        };
        let new = t.clamp(i64::from(start) + sum);
        if new == old {
            continue;
        }
        if let Some(tracks) = w.units[c].tracks.as_mut() {
            tracks.insert(t.id.clone(), new);
        }
        let stage = t.stage(new).map_or(String::new(), |s| s.name.clone());
        parts.push(if stage.is_empty() { format!("{name}'s {} is at stage {new}", t.name) } else { format!("{name}'s {} is at stage {new} ({stage})", t.name) });
        if new < old {
            continue;
        }
        for n in old + 1..=new {
            if let Some(p) = t.stage(n).and_then(|s| cat.passives.get(&s.passive)) {
                w.units[c].grant_passive_now(&cat, p);
            }
        }
        if t.departure_at(new).is_empty() {
            continue;
        }
        if w.allies.contains(&c) && w.set_side(c, crate::encounter::HOSTILE_ALL) {
            w.marks.turned.push((c, t.id.clone()));
            parts.push(format!("{name} turns on everyone"));
        } else if w.sides.is_rogue(c) && w.marks.calming(c) {
            w.marks.calm.retain(|(u, _)| *u != c);
            w.marks.turned.push((c, t.id.clone()));
            parts.push(format!("{name} stays turned on everyone"));
        }
    }
    parts
}

/// PK2-S1: how many times `c` used skill `skill` (a catalog id) this battle.
pub fn uses_of(w: &World, c: UnitId, skill: &str) -> i32 {
    w.skill_uses.get(&(c, skill.to_string())).copied().unwrap_or(0)
}

/// PK2-S1: why `c` cannot plan another use of skill `s` ("" = it can): its `uses_per_battle` (0 = no limit) taken by
/// the uses so far this battle and `planned` more already in its plan.
pub fn uses_problem(w: &World, c: UnitId, s: &rpgkit_data::SkillDef, planned: usize) -> String {
    if s.uses_per_battle <= 0 || uses_of(w, c, &s.id) + (planned as i32) < s.uses_per_battle {
        return String::new();
    }
    let times = if s.uses_per_battle == 1 { "once".to_string() } else { format!("{} times", s.uses_per_battle) };
    format!("{} is used {times} a battle.", s.name)
}

/// 3.7: the round starts: each unit an `unmark` calmed that still fights everyone rejoins the party side (in calm
/// order). Returns (unit, source) per unit that rejoined.
pub fn round_start(w: &mut World) -> Vec<(UnitId, String)> {
    let calm = std::mem::take(&mut w.marks.calm);
    calm.into_iter().filter(|(c, _)| w.units[*c].is_active() && w.sides.is_rogue(*c) && w.rejoin_party(*c)).collect()
}
