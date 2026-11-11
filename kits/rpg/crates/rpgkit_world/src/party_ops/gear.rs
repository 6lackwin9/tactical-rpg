//! Gear moves on a draft (gear_model.gd equip / unequip / pouch, GEARUI1 drag and drop): a member's equipment and pouch
//! against the party stash, every move refused with its reason or applied whole. The two-handed rule stows a shield or
//! focus the new main hand can't hold (the Godot dojo's "stowed: ... is 2-Handed") and clears it when a two-handed
//! class default comes back; under `pouch_stacks` the pouch holds stack slots with packed targets (INTENT2-U). Clicks,
//! drops and a screen's stat preview share these. PR5-31 moved them from the game's party screen with no rule change.

use super::title_case;
use crate::progression;
use crate::state::{item_fits_slot, off_hand_ok, LOADOUT_SLOTS, POUCH_MAX};
use crate::{Data, UnitState};
use indexmap::IndexMap;
use rpgkit_data::{ItemDef, NO_ITEM};

/// The party stash: item id -> count.
pub type Stash = IndexMap<String, i64>;

fn take_one(stash: &mut Stash, item: &str) {
    let left = stash.get(item).copied().unwrap_or(0) - 1;
    if left <= 0 {
        stash.shift_remove(item);
    } else {
        stash.insert(item.to_string(), left);
    }
}

fn give_one(stash: &mut Stash, item: String) {
    *stash.entry(item).or_insert(0) += 1;
}

/// Wears `item` from the stash in `slot`, or (None) the first free slot it fits, else the first it fits; what the slot
/// held goes back to the stash. A two-handed main hand stows an off-hand item it can't hold.
pub fn equip(d: &Data, world: &str, work: &mut UnitState, stash: &mut Stash, item: &str, slot: Option<&str>) -> Result<(), String> {
    let def = d.cat.items.get(item).ok_or_else(|| format!("Unknown item '{item}'."))?;
    if stash.get(item).copied().unwrap_or(0) < 1 {
        return Err(format!("There is no {} in the stash.", def.name));
    }
    let slot = match slot {
        Some(s) if item_fits_slot(def, s) => s,
        Some(s) => return Err(format!("{} doesn't go in the {} slot.", def.name, title_case(s).to_lowercase())),
        None => *LOADOUT_SLOTS.iter().find(|s| item_fits_slot(def, s) && work.equipment.get(**s).is_none_or(|i| i == NO_ITEM)).or_else(|| LOADOUT_SLOTS.iter().find(|s| item_fits_slot(def, s))).ok_or_else(|| format!("{} can't be worn.", def.name))?,
    };
    work.can_wear(d, item, slot)?;
    if slot == "off_hand" {
        if let Some(main) = main_weapon(d, world, work).filter(|m| !off_hand_ok(Some(m), def)) {
            return Err(format!("{} needs both hands: the off hand can hold only a weapon with it.", main.name));
        }
    }
    // PR5-26 (audit B2): an emptied slot (`NO_ITEM`) gives nothing back
    if let Some(old) = work.equipment.insert(slot.to_string(), item.to_string()).filter(|o| o != NO_ITEM) {
        give_one(stash, old);
    }
    take_one(stash, item);
    if slot == "main_hand" && def.two_handed {
        // A shield or focus goes back to the stash (the Godot dojo's "stowed: ... is 2-Handed").
        if let Some(off) = work.equipment.get("off_hand").cloned().filter(|o| d.cat.items.get(o).is_some_and(|o| !off_hand_ok(Some(def), o))) {
            work.equipment.shift_remove("off_hand");
            give_one(stash, off);
        }
    }
    Ok(())
}

/// The weapon the main hand holds: the worn one, else the class default the build gives.
pub fn main_weapon(d: &Data, world: &str, work: &UnitState) -> Option<ItemDef> {
    match work.equipment.get("main_hand") {
        Some(id) => d.cat.items.get(id).cloned(),
        None => progression::build_as(d, work, world, false, None).and_then(|b| b.unit.inventory.get(rpgkit_rules::gear::Slot::MainHand).map(|e| e.item.clone())),
    }
}

/// Takes off what `slot` holds; when the main hand's class default that comes back is two-handed, a shield or focus
/// in the off hand goes to the stash too (the two-handed rule).
pub fn take_off(d: &Data, world: &str, work: &mut UnitState, stash: &mut Stash, slot: &str) -> Result<(), String> {
    unequip(d, world, work, stash, slot)?;
    if slot == "main_hand" {
        let off = work.equipment.get("off_hand").and_then(|o| d.cat.items.get(o)).cloned();
        if let (Some(main), Some(off)) = (main_weapon(d, world, work), off) {
            if !off_hand_ok(Some(&main), &off) {
                unequip(d, world, work, stash, "off_hand")?;
            }
        }
    }
    Ok(())
}

/// Takes off what `slot` holds (the class default comes back). PR5-26 (audit B2): a slot left empty (`NO_ITEM`) stays
/// so; a worn item whose class default the member's `forbid_hands` refuses leaves the slot empty (`NO_ITEM`).
pub fn unequip(d: &Data, world: &str, work: &mut UnitState, stash: &mut Stash, slot: &str) -> Result<(), String> {
    match work.equipment.get(slot) {
        None => return Err(format!("The {} slot already holds the class default.", slot.replace('_', " "))),
        Some(i) if i == NO_ITEM => return Err(format!("The {} slot is already empty.", slot.replace('_', " "))),
        Some(_) => {}
    }
    // PR5-26 review F3: the default of the played world (as the settle's `stash_forbidden` reads it)
    let empty = work.forbids_default(d, world, slot);
    let item = work.equipment.shift_remove(slot).unwrap_or_default();
    if empty {
        work.equipment.insert(slot.to_string(), NO_ITEM.to_string());
    }
    give_one(stash, item);
    Ok(())
}

/// Moves the item worn in `from` to `to` (it must fit there); what `to` held moves to `from` when it fits, else to the
/// stash.
pub fn move_slot(d: &Data, world: &str, work: &mut UnitState, stash: &mut Stash, from: &str, to: &str) -> Result<(), String> {
    let item = work.equipment.get(from).cloned().filter(|i| i != NO_ITEM).ok_or("That slot holds the class default.")?;
    let def = d.cat.items.get(&item).ok_or_else(|| format!("Unknown item '{item}'."))?;
    if from == to || !item_fits_slot(def, to) {
        return Err(format!("{} doesn't go in the {} slot.", def.name, title_case(to).to_lowercase()));
    }
    // PR5-26 (audit B6): the member's `forbid_hands` refuses the item in its new slot too
    work.can_wear(d, &item, to)?;
    if to == "off_hand" {
        let mut rest = work.clone();
        rest.equipment.shift_remove(from);
        if let Some(main) = main_weapon(d, world, &rest).filter(|m| !off_hand_ok(Some(m), def)) {
            return Err(format!("{} needs both hands: the off hand can hold only a weapon with it.", main.name));
        }
    }
    work.equipment.shift_remove(from);
    // PR5-26 review F1: an emptied slot (`NO_ITEM`) gives nothing back; the slot left behind holds `NO_ITEM` when the
    // class default that would come back is one the member's `forbid_hands` refuses (as `unequip`)
    let old = work.equipment.insert(to.to_string(), item).filter(|o| o != NO_ITEM);
    match old {
        Some(old) if d.cat.items.get(&old).is_some_and(|o| item_fits_slot(o, from)) => {
            work.equipment.insert(from.to_string(), old);
        }
        Some(old) => give_one(stash, old),
        None => {}
    }
    if !work.equipment.contains_key(from) && work.forbids_default(d, world, from) {
        work.equipment.insert(from.to_string(), NO_ITEM.to_string());
    }
    Ok(())
}

/// Puts a consumable from the stash in the pouch: in place of pouch slot `at` when it is filled (that item goes back),
/// else at the end while there is room. INTENT2-U: under `pouch_stacks` (`stacks`) it assigns the item a stack slot
/// instead (`stack_put`).
pub fn pouch_put(d: &Data, work: &mut UnitState, stash: &mut Stash, item: &str, at: Option<usize>, stacks: bool) -> Result<(), String> {
    let def = d.cat.items.get(item).ok_or_else(|| format!("Unknown item '{item}'."))?;
    if def.item_type != "CONSUMABLE" {
        return Err(format!("{} can't go in the battle pouch.", def.name));
    }
    if !def.offered_in_battle() {
        return Err(format!("{} can't be used in battle.", def.name));
    }
    if stash.get(item).copied().unwrap_or(0) < 1 {
        return Err(format!("There is no {} in the stash.", def.name));
    }
    if stacks {
        return stack_put(d, work, stash, item, at);
    }
    match at.filter(|&i| i < work.pouch.len()) {
        Some(i) => {
            let old = std::mem::replace(&mut work.pouch[i], item.to_string());
            give_one(stash, old);
        }
        None if work.pouch.len() >= POUCH_MAX => return Err(format!("The pouch holds at most {POUCH_MAX} items.")),
        None => work.pouch.push(item.to_string()),
    }
    take_one(stash, item);
    Ok(())
}

/// Takes pouch slot `i` back to the stash (under `pouch_stacks`, `stacks`: the stack slot and its copies).
pub fn pouch_take(work: &mut UnitState, stash: &mut Stash, i: usize, stacks: bool) -> Result<(), String> {
    if stacks {
        return work.take_pouch_slot(i, stash).map(|_| ());
    }
    if i >= work.pouch.len() {
        return Err("That pouch slot is empty.".into());
    }
    give_one(stash, work.pouch.remove(i));
    Ok(())
}

/// INTENT2-U ("Stacks & Auto-Fill"): gives `item` a stack slot (the kit's `assign_pouch`: packed to `min(5,
/// max_stack)`, a second slot of the same item refused), in place of stack slot `at` when that holds another item (its
/// copies go back to the stash) and moved to that place (`UnitState::move_pouch_slot`). Dropped on its own slot it is a
/// no-op. No copy moves now: the kit's auto-fill fills the slot at combat start and after a fight.
pub fn stack_put(d: &Data, work: &mut UnitState, stash: &mut Stash, item: &str, at: Option<usize>) -> Result<(), String> {
    let slots = work.pouch_slots();
    if at.is_some_and(|i| slots.get(i).is_some_and(|s| s == item)) {
        return Ok(());
    }
    if let Some(i) = at.filter(|&i| i < slots.len() && !slots.iter().any(|s| s == item)) {
        work.take_pouch_slot(i, stash)?;
    }
    work.assign_pouch(d, item).map_err(|e| named(d, item, e))?;
    // the new slot's place: not always the last (a key-less slot, copies with no packed target, sorts after it)
    if let (Some(i), Some(now)) = (at, work.pouch_slots().iter().position(|s| s == item)) {
        if i < now {
            work.move_pouch_slot(d, now, i)?;
        }
    }
    Ok(())
}

/// A kit refusal with the item's name for its id (`'con_x' already has a pouch slot.` reads `Fire Flask already ...`).
pub fn named(d: &Data, item: &str, e: String) -> String {
    e.replace(&format!("'{item}'"), &d.item_name(item))
}

/// INTENT2-U: sets pouch stack `item`'s packed count (`UnitState::pack_pouch`), the refusal naming the item.
pub fn pack(d: &Data, work: &mut UnitState, stash: &mut Stash, item: &str, n: i64) -> Result<(), String> {
    work.pack_pouch(d, item, n, stash).map_err(|e| named(d, item, e))
}

/// A drop: payload `stash.<item>`, `equip.<slot>` or `pouch.<i>` on target `slot.<slot>`, `pouch.<i>` or `stash`
/// (INTENT2-U: under `pouch_stacks`, `stacks`, a pouch index names a stack slot).
pub fn drop_move(d: &Data, world: &str, work: &mut UnitState, stash: &mut Stash, from: &str, to: &str, stacks: bool) -> Result<(), String> {
    let (kind, what) = from.split_once('.').unwrap_or((from, ""));
    let (target, at) = to.split_once('.').unwrap_or((to, ""));
    let index = |s: &str| s.parse::<usize>().map_err(|_| format!("Bad pouch slot '{s}'."));
    match (kind, target) {
        ("stash", "slot") => equip(d, world, work, stash, what, Some(at)),
        ("stash", "pouch") => pouch_put(d, work, stash, what, Some(index(at)?), stacks),
        ("equip", "stash") => take_off(d, world, work, stash, what),
        ("equip", "slot") => move_slot(d, world, work, stash, what, at),
        ("pouch", "stash") => pouch_take(work, stash, index(what)?, stacks),
        ("pouch", "pouch") if stacks => work.move_pouch_slot(d, index(what)?, index(at)?),
        ("pouch", "pouch") => {
            let (i, j) = (index(what)?, index(at)?);
            if i == j || i >= work.pouch.len() {
                return Err("Nothing to move.".into());
            }
            let item = work.pouch.remove(i);
            let j = j.min(work.pouch.len());
            work.pouch.insert(j, item);
            Ok(())
        }
        _ => Err("That can't go there.".into()),
    }
}
