//! Battle marks (PK1-S1, docs/sessions/PK1_CORRUPTION.md 3.5 and 3.7; engine-only, no Godot original). The trigger
//! effect `mark` `{mark, value}` records an entry on a unit in battle state (`World::marks`, no save key); the battle
//! result carries each fielded member's entries (`rpgkit_world::roster::HeroEnd::marks`) for the world settle to read
//! (PK2 consumes them). A consumable's `unmark` `{mark}` removes the target's latest entry of that id; when that target
//! is a unit that started the battle on the party side and now fights everyone (a `hostile_all` rogue, `World.sides`),
//! it is also calmed: at the next round start it returns to the party side at its battle-start place among the allies
//! (`World::rejoin_party`). No data uses either yet, so every battle keeps its marks empty and logs nothing new.

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
}

impl Marks {
    /// No marks; `start_party` the party side as the battle starts.
    pub fn new(start_party: &[UnitId]) -> Self {
        Self { list: Vec::new(), calm: Vec::new(), start_party: start_party.to_vec() }
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
}

/// Whether an `unmark` may reach `c` although it fights the user: `c` started the battle on the party side and is now
/// a `hostile_all` rogue (3.7: "it was an ally at the battle's start").
pub fn turned_ally(w: &World, c: UnitId) -> bool {
    w.sides.is_rogue(c) && w.marks.started_on_party(c)
}

/// 3.7: an `unmark` of `mark` by `source` on `c`: its latest `mark` entry goes; on a turned ally (`turned_ally`) it is
/// also calmed at the next round start. Returns the log part ("" = nothing to undo).
pub fn unmark(w: &mut World, c: UnitId, mark: &str, source: &str) -> String {
    if mark.is_empty() || w.marks.remove_latest(c, mark).is_none() {
        return String::new();
    }
    let name = w.units[c].name.clone();
    if turned_ally(w, c) && !w.marks.calming(c) {
        w.marks.calm.push((c, source.to_string()));
        return format!("{name}'s {mark} undone; {name} calms at the next round start");
    }
    format!("{name}'s {mark} undone")
}

/// 3.7: the round starts: each unit an `unmark` calmed that still fights everyone rejoins the party side (in calm
/// order). Returns (unit, source) per unit that rejoined.
pub fn round_start(w: &mut World) -> Vec<(UnitId, String)> {
    let calm = std::mem::take(&mut w.marks.calm);
    calm.into_iter().filter(|(c, _)| w.units[*c].is_active() && w.sides.is_rogue(*c) && w.rejoin_party(*c)).collect()
}
