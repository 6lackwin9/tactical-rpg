//! Weapon mastery (PK2-S4, `docs/sessions/PK2_GRAFTS.md` section 5; engine-only, no Godot original): a member's own
//! XP per main-hand weapon family (`UnitState::mastery_xp`, a bar toward levels 1-3; `rpgkit_data::mastery`). Only a
//! family the member may master gains it (`masterable`: a main-hand WEAPON family of the item family table at
//! proficiency 2 or more), scaled by the proficiency rate (x1 at 2, x1.5 at 3), and only from the act the manifest's
//! `durability.from_act` names (`open`; no key = never, so a game or an act without it gains nothing). The sources: use
//! (each battle the member attacked with the family, `settle`), books (`read_book`, item type `MASTERY_BOOK`) and drills
//! (the recipe effect `drill`, `drill`). The XP stops at the top level's total; a gain past it is refused (a book, a
//! drill) or skipped (use).

use crate::data::Data;
use crate::progression;
use crate::roster::{self, HeroEnd};
use crate::state::{GameState, UnitState};
use aegis_core::json::{self, Dict};
use rpgkit_data::mastery as m;

/// One member's mastery gain (for the results and the notes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gain {
    pub member: String,
    pub family: String,
    pub from: i64,
    pub to: i64,
}

impl Gain {
    /// The mastery level after the gain (0-3).
    pub fn level(&self) -> i32 {
        m::level_of(self.to)
    }

    /// Whether the gain reached a new level.
    pub fn level_up(&self) -> bool {
        m::level_of(self.to) > m::level_of(self.from)
    }
}

/// The total XP the top level needs (where the bar stops).
pub fn xp_cap() -> i64 {
    m::MASTERY_LEVEL_XP[m::MASTERY_LEVEL_XP.len() - 1]
}

/// Whether mastery is gained now: the manifest names a `durability.from_act` and the campaign's act is at or after it.
pub fn open(d: &Data, gs: &GameState) -> bool {
    d.from_act > 0 && i64::from(roster::act(gs)) >= d.from_act
}

/// The refusal while mastery is not gained yet.
pub const NOT_YET_TEXT: &str = "Weapon mastery is not available yet.";

/// The proficiency level member `mem` has in gear type `family` (its class's, its drills' and its trained levels).
pub fn proficiency(d: &Data, mem: &UnitState, family: &str) -> i32 {
    d.cat.classes.get(&mem.class_id(d)).map_or(0, |c| mem.proficiency(d, c).level_of(family))
}

/// Whether `family` is a main-hand weapon family of the item family table (never armour, shields or off-hand foci).
pub fn main_hand_family(d: &Data, family: &str) -> bool {
    let row = d.entry("item_families", family);
    !row.is_empty() && m::main_hand_family(row)
}

/// Whether member `mem` may master `family`: a main-hand weapon family it has at proficiency 2 or more.
pub fn masterable(d: &Data, mem: &UnitState, family: &str) -> bool {
    main_hand_family(d, family) && proficiency(d, mem, family) >= m::MASTERY_PROF_MIN
}

/// The families member `mem` may master, in the item family table's order.
pub fn masterable_families(d: &Data, mem: &UnitState) -> Vec<String> {
    d.content.get("item_families").map(|t| t.keys().filter(|f| masterable(d, mem, f)).cloned().collect()).unwrap_or_default()
}

/// Adds `base` XP (scaled by the proficiency rate) to member `member`'s mastery in `family`. Err (nothing changes) while
/// mastery is closed, for a member not in the party, a family it may not master, or a bar already full.
pub fn gain(d: &Data, gs: &mut GameState, member: &str, family: &str, base: i64) -> Result<Gain, String> {
    if !open(d, gs) {
        return Err(NOT_YET_TEXT.into());
    }
    let mem = gs.party.member(member).ok_or("Pick a party member.")?;
    let who = progression::display_name(d, mem);
    if !masterable(d, mem, family) {
        return Err(format!("{who} can't master {family} (a main-hand weapon family at proficiency {}+).", m::MASTERY_PROF_MIN));
    }
    let add = m::scaled(base, proficiency(d, mem, family));
    let from = mem.mastery_xp.get(family).copied().unwrap_or(0).max(0);
    if from >= xp_cap() {
        return Err(format!("{who} has mastered {family} already."));
    }
    let to = (from + add).min(xp_cap());
    let mem = gs.party.member_mut(member).ok_or("Pick a party member.")?;
    mem.mastery_xp.insert(family.to_string(), to);
    Ok(Gain { member: member.to_string(), family: family.to_string(), from, to })
}

/// A gain's note: "Ana: sword mastery +10 (level 1)." (+ " New level." on a level-up).
pub fn note(d: &Data, gs: &GameState, g: &Gain) -> String {
    let who = gs.party.member(&g.member).map_or_else(|| g.member.clone(), |mem| progression::display_name(d, mem));
    let up = if g.level_up() { " New level." } else { "" };
    format!("{who}: {} mastery +{} (level {}).{up}", g.family, g.to - g.from, g.level())
}

/// The settle's use: each fielded member still in the party gains the use XP in every family it attacked with this
/// battle (`HeroEnd::attacked`) that it may master; a full bar or a closed act gains nothing. In order.
pub fn settle(d: &Data, gs: &mut GameState, ends: &[&HeroEnd]) -> Vec<Gain> {
    if !open(d, gs) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for e in ends {
        for family in &e.attacked {
            if let Ok(g) = gain(d, gs, &e.member, family, m::USE_XP) {
                out.push(g);
            }
        }
    }
    out
}

/// Member `member` reads a `MASTERY_BOOK` from the stash: its family gains the book's XP and one copy is used. {ok,
/// errors, notes}; a refusal changes nothing.
pub fn read_book(d: &Data, gs: &mut GameState, member: &str, item: &str) -> Dict {
    let Some(book) = d.cat.items.get(item).and_then(|i| i.book.clone()) else { return json::fail(&format!("{} is not a mastery book.", d.item_name(item))) };
    if gs.party.stash.get(item).copied().unwrap_or(0) < 1 {
        return json::fail(&format!("Not enough {}.", d.item_name(item)));
    }
    let mut work = gs.clone();
    match gain(d, &mut work, member, &book.family, book.xp) {
        Ok(g) => {
            work.party.remove_from_stash(item, 1);
            let line = note(d, &work, &g);
            *gs = work;
            let mut out = json::ok_result(true, &[]);
            out.insert("notes".into(), json::strs(&[line]));
            out
        }
        Err(why) => json::fail(&why),
    }
}

/// The family a `drill` works on for member `mem`: `picked` when given (it must be one it may master), else its only
/// masterable family.
pub fn drill_family(d: &Data, mem: &UnitState, picked: &str) -> Result<String, String> {
    let who = progression::display_name(d, mem);
    let fams = masterable_families(d, mem);
    match (picked.is_empty(), fams.len()) {
        (_, 0) => Err(format!("{who} has no weapon family to master.")),
        (true, 1) => Ok(fams[0].clone()),
        (true, _) => Err(format!("Pick a weapon family ({}).", fams.join(", "))),
        (false, _) if fams.iter().any(|f| f == picked) => Ok(picked.to_string()),
        (false, _) => Err(format!("{who} can't master {picked}.")),
    }
}
