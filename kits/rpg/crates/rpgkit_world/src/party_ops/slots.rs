//! The classic skill slots (loadout_model.gd, SKILLUI1): a member's class slots (`MAX_CLASS_SLOTS`, class path families
//! only, Q3 A) and extra slots (`MAX_EXTRA_SLOTS`, any family), with the chosen rank per slotted family (R2, Q2). Every
//! edit of a draft is pure here: the click path, a drop (SKILLUI1 section 4 drop table), unslotting, the rank step and
//! the confirmed loadout. PR5-31 moved it from the game's party screen with no rule change.

use crate::progression;
use crate::{Data, UnitState};
use indexmap::IndexMap;
use rpgkit_rules::stats::{MAX_CLASS_SLOTS, MAX_EXTRA_SLOTS};

/// A classic Skills draft: class slots, extra slots, the chosen rank per slotted family (family -> rank).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SkillSlots {
    pub class: Vec<String>,
    pub extra: Vec<String>,
    pub ranks: IndexMap<String, i64>,
}

/// The slots of a group: `MAX_CLASS_SLOTS` for the class group, else `MAX_EXTRA_SLOTS`.
pub fn slot_cap(class_group: bool) -> usize {
    if class_group {
        MAX_CLASS_SLOTS
    } else {
        MAX_EXTRA_SLOTS
    }
}

/// SKILLUI1 Q3 A: whether family `fam` may take a slot of the class group (`class_group`: only the member's class path
/// families, `path`) or of the extra group (any family).
pub fn can_assign_to_slot(path: &[String], fam: &str, class_group: bool) -> bool {
    !class_group || path.iter().any(|p| p == fam)
}

/// SKILLUI1 R2 on the draft: no entry, the highest selectable rank; an entry `c`, the highest selectable rank <= `c`,
/// else the lowest; 0 when nothing is selectable.
pub fn draft_rank(selectable: &[i32], chosen: Option<i32>) -> i32 {
    match chosen {
        None => selectable.last().copied().unwrap_or(0),
        Some(c) => selectable.iter().rev().find(|&&r| r <= c).or(selectable.first()).copied().unwrap_or(0),
    }
}

/// SKILLUI1: the rank one `step` (-1 previous, +1 next) from the draft's rank (`draft_rank` of `chosen`) among the
/// selectable ranks; None past either end.
pub fn rank_step(selectable: &[i32], chosen: Option<i32>, step: i32) -> Option<i32> {
    let now = draft_rank(selectable, chosen);
    let at = selectable.iter().position(|&r| r == now).unwrap_or(selectable.len().saturating_sub(1)) as i32;
    selectable.get((at + step).max(0) as usize).filter(|_| at + step >= 0).copied()
}

/// The member's class path families in `world` (class slots take only these, SKILLUI1 Q3 A; the played world's path,
/// HOOKS1 CN3).
pub fn path_families(d: &Data, m: &UnitState, world: &str) -> Vec<String> {
    d.cat_in(world).classes.get(&m.class_id(d)).map(|a| rpgkit_rules::unit::families_of(&d.cat, &a.path_steps().into_iter().map(|(_, s)| s).collect::<Vec<_>>())).unwrap_or_default()
}

/// The highest rank of `family` the member has unlocked in `world` (with confirmed gear): what a newly slotted skill
/// stores (the user's Q2 answer).
pub fn top_rank(d: &Data, m: &UnitState, world: &str, family: &str) -> Option<i64> {
    progression::build_with(d, m, world, false).and_then(|b| b.unit.selectable_ranks(&d.cat, family).last().map(|&r| r as i64))
}

impl SkillSlots {
    /// The member's saved slots and ranks (`loadout_class`, `loadout_extra`, `loadout_ranks`).
    pub fn of(m: &UnitState) -> Self {
        Self { class: m.loadout_class.clone(), extra: m.loadout_extra.clone(), ranks: m.loadout_ranks.clone() }
    }

    /// The default loadout (no saved slots): the families a build of `m` in `world` slots by itself (None when it does
    /// not build). Ranks are left to the caller.
    pub fn default_of(d: &Data, m: &UnitState, world: &str) -> Option<(Vec<String>, Vec<String>)> {
        let mut work = m.clone();
        work.loadout_class.clear();
        work.loadout_extra.clear();
        progression::build_with(d, &work, world, false).map(|b| (b.unit.active_class_families.clone(), b.unit.active_extra_families.clone()))
    }

    /// Whether `fam` is in a slot of either group.
    pub fn has(&self, fam: &str) -> bool {
        self.class.iter().chain(&self.extra).any(|f| f == fam)
    }

    /// The click path's group for `fam`: Ok(true) the class group (a class path family while a class slot is free),
    /// Ok(false) the extra group (a free extra slot, also for a class path family when the class slots are full, Q3 A),
    /// else why not (the full group says how to make room).
    pub fn click_group(&self, path: &[String], fam: &str) -> Result<bool, String> {
        let on_path = path.iter().any(|p| p == fam);
        if on_path && self.class.len() < MAX_CLASS_SLOTS {
            Ok(true)
        } else if self.extra.len() < MAX_EXTRA_SLOTS {
            Ok(false)
        } else {
            let group = if on_path { "Class" } else { "Extra" };
            Err(format!("{group} slots are full: drag onto a slot to replace one, or right click a slot to take it out."))
        }
    }

    /// The click path: `fam` into the first free slot of its group (`click_group`), storing the rank `top_rank` gives
    /// (the highest unlocked, Q2; read only when it is slotted). Ok(None): already slotted (nothing changes).
    pub fn slot(&self, path: &[String], fam: &str, top_rank: &dyn Fn(&str) -> Option<i64>) -> Result<Option<Self>, String> {
        if self.has(fam) {
            return Ok(None);
        }
        let to_class = self.click_group(path, fam)?;
        let mut s = self.clone();
        if to_class {
            s.class.push(fam.to_string());
        } else {
            s.extra.push(fam.to_string());
        }
        if let Some(r) = top_rank(fam) {
            s.ranks.insert(fam.to_string(), r);
        }
        Ok(Some(s))
    }

    /// Takes slot `i` of a group out with its rank (an empty slot: nothing changes).
    pub fn unslot(&mut self, class_group: bool, i: usize) {
        let list = if class_group { &mut self.class } else { &mut self.extra };
        if i < list.len() {
            let fam = list.remove(i);
            self.ranks.shift_remove(&fam);
        }
    }

    /// The ranks of the slotted families only (every other entry dropped).
    pub fn keep_slotted_ranks(&mut self) {
        let keep: Vec<String> = self.class.iter().chain(&self.extra).cloned().collect();
        self.ranks.retain(|f, _| keep.contains(f));
    }

    /// The draft after dropping payload `from` (`skill.<f>` from the list, `slot.<g>.<i>` from a slot) on target `to`
    /// (`slot.<g>.<i>` or `list`), or why it can't be done (SKILLUI1 section 4 drop table). `path` is the member's class
    /// path (`path_families`); `top_rank` gives a newly slotted family's rank (read only when one is slotted).
    pub fn after_drop(&self, path: &[String], from: &str, to: &str, top_rank: &dyn Fn(&str) -> Option<i64>) -> Result<Self, String> {
        let (mut class, mut extra, mut ranks) = (self.class.clone(), self.extra.clone(), self.ranks.clone());
        let slot_of = |key: &str| -> Option<(bool, usize)> {
            let rest = key.strip_prefix("slot.")?;
            let (g, i) = rest.split_once('.')?;
            Some((g == "class", i.parse().ok()?))
        };
        let fits = |fam: &str, class_group: bool| can_assign_to_slot(path, fam, class_group);
        if let Some(fam) = from.strip_prefix("skill.") {
            let (to_class, i) = slot_of(to).ok_or("Drop a skill on a slot.")?;
            // A family already slotted moves from its own slot (the slot-to-slot rules: swap or move, never a shifted
            // index), whether it was dragged from the list or from its slot.
            if let Some(j) = class.iter().position(|f| f == fam) {
                return self.after_drop(path, &format!("slot.class.{j}"), to, top_rank);
            }
            if let Some(j) = extra.iter().position(|f| f == fam) {
                return self.after_drop(path, &format!("slot.extra.{j}"), to, top_rank);
            }
            if !fits(fam, to_class) {
                return Err("Class slots take class path skills only.".into());
            }
            let list = if to_class { &mut class } else { &mut extra };
            match list.get(i).cloned() {
                Some(old) => {
                    list[i] = fam.to_string();
                    ranks.shift_remove(&old);
                }
                None => {
                    if list.len() >= slot_cap(to_class) {
                        return Err("That group is full.".into());
                    }
                    list.push(fam.to_string());
                }
            }
            // the user's Q2 answer: a newly slotted skill stores its highest unlocked rank now
            if let Some(r) = top_rank(fam) {
                ranks.insert(fam.to_string(), r);
            }
            return Ok(Self { class, extra, ranks });
        }
        let (from_class, i) = slot_of(from).ok_or("That can't go there.")?;
        let fam = (if from_class { &class } else { &extra }).get(i).cloned().ok_or("That slot is empty.")?;
        if to == "list" {
            (if from_class { &mut class } else { &mut extra }).remove(i);
            ranks.shift_remove(&fam);
            return Ok(Self { class, extra, ranks });
        }
        let (to_class, j) = slot_of(to).ok_or("That can't go there.")?;
        if (from_class, i) == (to_class, j) {
            return Err("Same slot.".into());
        }
        let other = (if to_class { &class } else { &extra }).get(j).cloned();
        if !fits(&fam, to_class) || other.as_ref().is_some_and(|o| !fits(o, from_class)) {
            return Err("Class slots take class path skills only.".into());
        }
        match other {
            Some(o) => {
                (if from_class { &mut class } else { &mut extra })[i] = o;
                (if to_class { &mut class } else { &mut extra })[j] = fam;
            }
            None => {
                (if from_class { &mut class } else { &mut extra }).remove(i);
                let list = if to_class { &mut class } else { &mut extra };
                if list.len() >= slot_cap(to_class) {
                    return Err("That group is full.".into());
                }
                list.push(fam);
            }
        }
        Ok(Self { class, extra, ranks })
    }

    /// The member with this draft as its loadout, checked and its ranks pinned (Confirm): `using_default` saves no slots
    /// (the build picks them); ranks are kept for the slotted families only (every slotted family gets an entry, Q2).
    /// Err: the loadout's problems (`UnitState::loadout_errors`).
    pub fn confirmed(&self, d: &Data, m: &UnitState, world: &str, using_default: bool) -> Result<UnitState, Vec<String>> {
        let mut work = m.clone();
        if using_default {
            work.loadout_class.clear();
            work.loadout_extra.clear();
        } else {
            work.loadout_class = self.class.clone();
            work.loadout_extra = self.extra.clone();
        }
        let slotted: Vec<String> = self.class.iter().chain(&self.extra).cloned().collect();
        work.loadout_ranks = self.ranks.iter().filter(|(f, _)| slotted.contains(f)).map(|(f, r)| (f.clone(), *r)).collect();
        let errors = work.loadout_errors(d, &format!("Unit '{}'", work.id));
        if !errors.is_empty() {
            return Err(errors);
        }
        progression::pin_ranks(d, &mut work, world);
        Ok(work)
    }
}
