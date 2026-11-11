//! Weapon durability in battle (PK2-S5b, `docs/sessions/PK2_GRAFTS.md` 7, ACTS23 5.2; rule `durability`, engine-only,
//! no Godot original). A hero built from member state carries the durability of the weapon instances it wields that
//! wear (`rpgkit_rules::Unit::wear`, from `Growth::wear`). Under a ruleset that lists the rule, each command started
//! (its costs paid, `resolver::count_use`) spends from the weapon it swings (`Unit::swung_slot`): a standard attack
//! skill `SPEND_ATTACK`, a technique (or the wielded family's keystone, `Unit::technique_skill`) `SPEND_TECHNIQUE`;
//! basics, support skills, items and every action of a Channel round (the channeled path pays its meter instead) spend
//! nothing. At 0 the weapon is dulled: still usable, its weapon power at `gear::DULLED_POWER_MULT`
//! (`damage::attack_power`). The sims seed wear on their catalog allies (`seed`, sim key `durability=N`) and read it
//! back (`tally`). The sims' alternative (`World::dulled_lock`, sim key `dulled=lock`) also locks the
//! techniques that swing a dulled weapon (`problem`). The spend goes back to the world through
//! `rpgkit_world::roster::HeroEnd::worn` and is taken from the instances at the settle. Without the rule (a ruleset that
//! does not list it) the session clears every unit's wear at the start (`clear_without_rule`), so nothing is spent, dulled
//! or locked and the battle runs as before.

use crate::skill::{self, Kind};
use crate::world::{UnitId, World};
use rpgkit_data::SkillDef;
use rpgkit_rules::gear::{Equipment, Slot, Wear};
use rpgkit_rules::ruleset;

/// The durability one standard attack skill spends from the weapon it swings (default 1).
pub const SPEND_ATTACK: i32 = 1;
/// The durability one technique (or the wielded family's keystone) spends (default 2).
pub const SPEND_TECHNIQUE: i32 = 2;
/// The reason a technique is greyed under `World::dulled_lock` while the main hand is dulled.
pub const DULLED: &str = "Weapon dulled";

/// Whether the battle tracks durability: its ruleset lists the rule `durability`.
pub fn on(w: &World) -> bool {
    w.ruleset.has(ruleset::DURABILITY)
}

/// At the battle start: without the rule no unit tracks wear (inert by construction).
pub fn clear_without_rule(w: &mut World) {
    if !on(w) {
        for u in &mut w.units {
            u.wear.clear();
        }
    }
}

/// Whether `s` is a basic for `c`: a basic or system skill, or a family its deck rule opens as a basic (the ruleset's
/// and its class's).
fn is_basic(w: &World, c: UnitId, s: &SkillDef) -> bool {
    let fam = s.family_id();
    s.basic || s.system || w.cat.is_basic_family(fam) || w.units[c].deck_rule.as_ref().is_some_and(|r| r.basics.iter().any(|b| b == fam))
}

/// What skill `s` started by `c` spends now: 0 without the rule, without wear, for an item, in a Channel round or for
/// a basic; else `SPEND_TECHNIQUE` for a technique (whatever its kind), `SPEND_ATTACK` for an attack skill, 0 for any
/// other kind (support).
pub fn spend_of(w: &World, c: UnitId, s: &SkillDef) -> i32 {
    if !on(w) || w.units[c].wear.is_empty() || skill::is_item(s) || crate::channel::is_channel(w, c) || is_basic(w, c, s) {
        return 0;
    }
    if w.units[c].technique_skill(&s.id) {
        SPEND_TECHNIQUE
    } else if skill::kind(s) == Kind::Attack {
        SPEND_ATTACK
    } else {
        0
    }
}

/// The command `c` started with skill `s` (its costs paid) spends from the weapon it swings (`spend_of`; an untracked
/// slot spends nothing). Returns the slot and the amount spent (None: nothing spent).
pub fn spend(w: &mut World, c: UnitId, s: &SkillDef) -> Option<(Slot, i32)> {
    let n = spend_of(w, c, s);
    if n <= 0 {
        return None;
    }
    let u = &mut w.units[c];
    let slot = u.swung_slot(skill::is_off_hand(s))?;
    let wear = u.wear.get_mut(&slot)?;
    wear.spent += n;
    Some((slot, n))
}

/// The sims' seed (sim key `durability=N`, `rpgkit_sim`): units `units` track `n` durability (0 or more) on each weapon
/// they wield (the main hand, an off-hand weapon), replacing any wear they track. Nothing without the rule.
pub fn seed(w: &mut World, units: &[UnitId], n: i32) {
    if !on(w) {
        return;
    }
    for &c in units {
        let Some(u) = w.units.get_mut(c) else { continue };
        for slot in [Slot::MainHand, Slot::OffHand] {
            if u.inventory.get(slot).is_some_and(Equipment::is_weapon) {
                u.wear.insert(slot, Wear { start: n.max(0), spent: 0 });
            }
        }
    }
}

/// The durability `units` spent this battle in all and how many of their tracked weapons are dulled now (the sims'
/// per-fight record).
pub fn tally(w: &World, units: &[UnitId]) -> (i64, i32) {
    let tracked = || units.iter().filter_map(|&c| w.units.get(c)).flat_map(|u| u.wear.values());
    (tracked().map(|wr| i64::from(wr.spent)).sum(), tracked().filter(|wr| wr.dulled()).count() as i32)
}

/// The spend of every tracked slot of `c` this battle (slot save key, amount), slot order, nothing for an unspent slot.
pub fn spent(w: &World, c: UnitId) -> Vec<(String, i64)> {
    let Some(u) = w.units.get(c) else { return Vec::new() };
    u.wear.iter().filter(|(_, wr)| wr.spent > 0).map(|(s, wr)| (s.engine_name().to_lowercase(), i64::from(wr.spent))).collect()
}

/// Why `c` may not plan `s` for its weapon's wear (None: it may): under the sims' `World::dulled_lock`, a technique
/// (or the wielded family's keystone) while the weapon it swings is dulled (`Unit::swung_slot`, as the spend). None
/// without the rule or the switch.
pub fn problem(w: &World, c: UnitId, s: &SkillDef) -> Option<&'static str> {
    if !w.dulled_lock || !on(w) {
        return None;
    }
    let u = w.units.get(c)?;
    (u.technique_skill(&s.id) && u.swung_slot(skill::is_off_hand(s)).is_some_and(|slot| u.dulled(slot))).then_some(DULLED)
}
