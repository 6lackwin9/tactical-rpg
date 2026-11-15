//! The off hand in battle (GEAR-REWORK1, PR5-51a pass 2, `docs/sessions/U3_GEAR_REWORK.md` 2-4; engine-only, no Godot
//! original). Every rule here reads the off-hand item's family fields (`ItemDef::slot_rule`, `off_hand_skill`,
//! `empower_next`), never a family or skill id, so a catalog without them (every row from before the rework) plays as
//! before. The battle's Off-Hand button shows `action`: a `1h_light` weapon's off-hand jab (reaching as the weapon
//! does, `skill::light_off_reach`), else the family's off-hand skill (held as a worn item's skill, `Unit::gear_skills`);
//! an ammunition family's skill is its Reload, open only at 0 (`reload_problem`, read through `channel::problem` by
//! every planner and the AI's order). Under the rule `unit_meters` the wearer of an ammunition item holds its meter
//! (`meter::battle_start`); each ranged main-hand weapon attack started while the meter covers the cost spends it and
//! hits harder, and the Reload refills it (`meter::off_hand_ammo`). PR5-51a pass 3 (U3.3 2 / 4): an ammunition rule's
//! `throw_skill`, granted to a `1h_light` main-hand weapon beside the item (`gear::light_throw`), is open only while the
//! meter covers the cost (`throw_problem`, through `channel::problem`; at the start `meter::unpaid`), spends it at its
//! start and is not empowered.

use crate::skill::{self, Kind, SkillId};
use crate::world::{UnitId, World};
use rpgkit_data::{EmpowerNext, SkillDef};
use rpgkit_rules::gear::{OffHandUse, Slot};

/// Why a Reload is closed while its meter still holds ammunition.
pub const RELOAD_NOT_EMPTY: &str = "Reload only at 0 ammo";
/// Why a Reload is closed without its meter (the rule `unit_meters` off, or no gauge).
pub const RELOAD_NO_METER: &str = "No ammo meter to reload";
/// Why a Throw is closed while its meter holds less than the cost (the pip is what unlocks it).
pub const THROW_NO_AMMO: &str = "Throw needs ammo";
/// Why a Throw is closed without its meter (the rule `unit_meters` off, or no gauge).
pub const THROW_NO_METER: &str = "No ammo meter to throw";
/// Why a Throw is closed without a `1h_light` main-hand weapon (known some other way).
pub const THROW_NO_LIGHT: &str = "Throw needs a light main-hand weapon";

/// The Off-Hand button of unit `c`: the off-hand item, the skill the button picks, its use, the ammunition meter's
/// value and max (an ammunition item under the rule) and why it cannot be picked now (None: it can, as far as the
/// unit goes; the menu adds the target, block and AP checks).
#[derive(Debug, Clone, PartialEq)]
pub struct OffHandAction {
    pub item: String,
    pub skill: SkillId,
    pub used: OffHandUse,
    pub ammo: Option<(i32, i32)>,
    pub problem: Option<String>,
}

/// `c`'s Off-Hand button (None: an empty off hand, or an off-hand item from before the rework: the button stays hidden):
/// the unit's (`Unit::off_hand_button`), with the round's mode check (`channel::problem`, a Reload above 0 included).
pub fn action(w: &World, c: UnitId) -> Option<OffHandAction> {
    let u = w.units.get(c)?;
    let b = u.off_hand_button(&w.cat)?;
    let sid = w.book().find(&b.skill)?;
    let off = u.inventory.get(Slot::OffHand)?;
    let ammo = off.item.empower_next.as_ref().and_then(|e| crate::meter::pool(w, c, &e.meter)).map(|p| (p.current, p.max));
    let problem = b.problem.or_else(|| crate::channel::problem(w, c, &w.cat.skills[sid]).map(String::from));
    Some(OffHandAction { item: b.item, skill: sid, used: b.used, ammo, problem })
}

/// Why `c` may not start its Reload `s` now (None: it may, or `s` is not its Reload): the skill is the off-hand
/// ammunition item's `off_hand_skill`, open only while its meter is at 0 (closed without the meter).
pub fn reload_problem(w: &World, c: UnitId, s: &SkillDef) -> Option<&'static str> {
    let off = w.units.get(c)?.inventory.get(Slot::OffHand)?;
    let e = off.item.empower_next.as_ref().filter(|_| off.item.off_hand_skill == s.id)?;
    match crate::meter::value(w, c, &e.meter) {
        None => Some(RELOAD_NO_METER),
        Some(v) if v > 0 => Some(RELOAD_NOT_EMPTY),
        Some(_) => None,
    }
}

/// The ammunition rule whose Throw skill `s` is (`EmpowerNext::throw_skill` of `c`'s off-hand item; None: `s` is not one).
pub fn throw_rule<'a>(w: &'a World, c: UnitId, s: &str) -> Option<&'a EmpowerNext> {
    let off = w.units.get(c)?.inventory.get(Slot::OffHand)?;
    off.item.empower_next.as_ref().filter(|e| !e.throw_skill.is_empty() && e.throw_skill == s)
}

/// Why `c` may not start its Throw `s` now (None: it may, or `s` is not its Throw): the main hand holds no `1h_light`
/// weapon, the unit holds no meter, or the meter is below the cost.
pub fn throw_problem(w: &World, c: UnitId, s: &SkillDef) -> Option<&'static str> {
    let e = throw_rule(w, c, &s.id)?;
    if !w.units[c].inventory.main_weapon().is_some_and(|m| rpgkit_rules::gear::light_reach(&m.item).is_some()) {
        return Some(THROW_NO_LIGHT);
    }
    match crate::meter::value(w, c, &e.meter) {
        None => Some(THROW_NO_METER),
        Some(v) if v < e.cost => Some(THROW_NO_AMMO),
        Some(_) => None,
    }
}

/// Whether skill `s` started by `c` is a shot its off-hand ammunition empowers: a damaging attack (not an item, not an
/// off-hand one, not the ammunition's own Throw) swung with a ranged main-hand weapon.
pub fn empowers(w: &World, c: UnitId, s: &SkillDef) -> bool {
    throw_rule(w, c, &s.id).is_none()
        && skill::kind(s) == Kind::Attack && !skill::is_item(s) && !skill::is_off_hand(s) && skill::deals_damage(s) && w.units[c].inventory.main_weapon().is_some_and(|m| m.is_ranged_weapon())
}
