//! Weapon instances (PK2-S5a, `docs/sessions/PK2_GRAFTS.md` 6, FB-167 Q7 (a); engine-only, no Godot original). From the
//! act the game's manifest names (`durability.from_act`), under a world ruleset with the rule `durability`, every owned
//! weapon (item type `WEAPON`, main hand or off hand) is an instance `{uid, item, durability, masterwork}`; armour,
//! shields, accessories and consumables stay plain ids. The catalog row stays the template, and `equipment` and `stash`
//! keep their item ids (every reader of them works as before); the instances ride beside them by `uid`: a member's
//! `weapons` {slot: instance} holds the instance each weapon slot wields, the party's `stash_weapons` [instance] the
//! instances in the stash (one per stashed copy). `sync` keeps the two agreeing: at the act line it makes every owned
//! weapon an instance at full durability (`ItemDef::max_durability`); a weapon newly owned (a shop buy, loot, a reward)
//! gets a fresh full one; a weapon gone (sold, broken down) takes its worst stashed instance with it; a weapon moved
//! between a slot and the stash keeps its instance (the best one of the item goes to a slot that needs one); the stash
//! instances keep their order. A class
//! default (no saved gear in a slot) is no owned weapon and never an instance. Every key is written only when it holds
//! an instance, so a game without `durability.from_act`, or an act before it, saves as before (no save version bump).
//!
//! PK2-S5b (`docs/sessions/PK2_GRAFTS.md` 7): a fielded member carries its wielded instances' durability into the
//! battle (`wear_of`, `Growth::wear`; `rpgkit_battle::wear` spends it), and the settle takes the battle's spend off
//! those instances, each held at 0 (`wear`, from `HeroEnd::worn`).

use crate::data::Data;
use crate::roster;
use crate::state::{GameState, PartyState, UnitState};
use aegis_core::json::{self, int, Dict};
use indexmap::IndexMap;
use serde_json::Value;
use std::collections::HashSet;

/// The member save key ({slot: instance}) and the party save key ([instance]).
pub const MEMBER_KEY: &str = "weapons";
pub const STASH_KEY: &str = "stash_weapons";
/// An instance's keys, in their saved order.
pub const KEYS: [&str; 4] = ["uid", "item", "durability", "masterwork"];
/// The prefix of the uids `sync` gives (then a number past the highest in use).
pub const UID_PREFIX: &str = "w";
/// The most digits a `w<n>` uid number is read with (`uid_number`); past it a uid is valid but never counted.
pub const UID_DIGITS: usize = 9;
/// The slot whose masterwork instance opens the weapon's own technique.
pub const MAIN_HAND: &str = "main_hand";

/// One weapon instance: its uid, its catalog item (the template), its durability now and its masterwork flag.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Weapon {
    pub uid: String,
    pub item: String,
    pub durability: i64,
    pub masterwork: bool,
}

impl Weapon {
    /// A new instance of `item` at full durability, not masterwork, with no uid yet (`sync` gives one).
    pub fn fresh(d: &Data, item: &str) -> Self {
        Self { uid: String::new(), item: item.to_string(), durability: full(d, item), masterwork: false }
    }

    pub fn to_value(&self) -> Value {
        Value::Object(json::obj([
            (KEYS[0], Value::String(self.uid.clone())),
            (KEYS[1], Value::String(self.item.clone())),
            (KEYS[2], int(self.durability)),
            (KEYS[3], Value::Bool(self.masterwork)),
        ]))
    }

    /// A saved instance (a damaged one reads as empty or negative fields, which `problems` reports).
    pub fn from_value(v: &Value) -> Self {
        let o = json::dict(v);
        let durability = o.get(KEYS[2]).map(json::normalize).filter(json::is_int).map(|v| json::to_int(&v)).unwrap_or(-1);
        Self { uid: json::s(o, KEYS[0]), item: json::s(o, KEYS[1]), durability, masterwork: json::b(o, KEYS[3], false) }
    }

    /// The instance's problems (`at` names where it is saved).
    pub fn problems(&self, d: &Data, at: &str) -> Vec<String> {
        let mut out = Vec::new();
        if !json::id_ok(&self.uid) {
            out.push(format!("{at}: a weapon uid must be non-empty [a-z0-9_] (got '{}').", self.uid));
        }
        if !d.has("items", &self.item) {
            out.push(format!("{at}: weapon '{}' has an unknown item '{}'.", self.uid, self.item));
        }
        if self.durability < 0 {
            out.push(format!("{at}: weapon '{}' durability must be a whole number 0 or more.", self.uid));
        }
        out
    }
}

/// The saved form of a member's instances ({slot: instance}).
pub fn member_value(held: &IndexMap<String, Weapon>) -> Value {
    Value::Object(held.iter().map(|(k, w)| (k.clone(), w.to_value())).collect())
}

/// A member's saved instances (absent = none).
pub fn member_from(d: &Dict) -> IndexMap<String, Weapon> {
    json::dget(d, MEMBER_KEY).iter().map(|(k, v)| (k.clone(), Weapon::from_value(v))).collect()
}

/// The party's saved stash instances (absent = none).
pub fn stash_from(d: &Dict) -> Vec<Weapon> {
    json::aget(d, STASH_KEY).iter().map(Weapon::from_value).collect()
}

/// Whether catalog item `item` becomes an instance (`ItemDef::is_weapon`).
pub fn is_weapon(d: &Data, item: &str) -> bool {
    d.cat.items.get(item).is_some_and(rpgkit_data::ItemDef::is_weapon)
}

/// The full durability of a fresh instance of `item` (`ItemDef::max_durability`; 0 = none: it never wears).
pub fn full(d: &Data, item: &str) -> i64 {
    d.cat.items.get(item).map_or(0, |i| i64::from(i.max_durability()))
}

/// Whether owned weapons are instances now: the manifest names a `durability.from_act`, the campaign's act is at or
/// after it, and the played world's ruleset has the rule `durability`.
pub fn active(d: &Data, gs: &GameState) -> bool {
    d.from_act > 0 && i64::from(roster::act(gs)) >= d.from_act && d.ruleset(&gs.world_id).has(rpgkit_rules::ruleset::DURABILITY)
}

/// Makes the instances agree with the gear and the stash while they are `active` (otherwise nothing changes). Returns
/// whether anything changed.
pub fn sync(d: &Data, gs: &mut GameState) -> bool {
    if !active(d, gs) {
        return false;
    }
    let before = snapshot(&gs.party);
    sync_party(d, &mut gs.party);
    snapshot(&gs.party) != before
}

fn snapshot(party: &PartyState) -> (Vec<IndexMap<String, Weapon>>, Vec<Weapon>) {
    (party.members.iter().map(|m| m.weapons.clone()).collect(), party.stash_weapons.clone())
}

/// `sync`'s body (no act or rule gate): see the module text.
pub fn sync_party(d: &Data, party: &mut PartyState) {
    let mut pool: Vec<Weapon> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut needs: Vec<(usize, String)> = Vec::new();
    // a member keeps the instance of a slot that still wields its item; any other goes to the pool
    for (mi, m) in party.members.iter_mut().enumerate() {
        for (slot, w) in std::mem::take(&mut m.weapons) {
            if m.equipment.get(&slot) == Some(&w.item) && is_weapon(d, &w.item) && json::id_ok(&w.uid) && seen.insert(w.uid.clone()) {
                m.weapons.insert(slot, w);
            } else {
                pool.push(w);
            }
        }
        needs.extend(m.equipment.iter().filter(|(slot, item)| is_weapon(d, item) && !m.weapons.contains_key(*slot)).map(|(slot, _)| (mi, slot.clone())));
    }
    pool.extend(std::mem::take(&mut party.stash_weapons));
    pool.retain(|w| is_weapon(d, &w.item));
    // a uid already in use (or none) is renamed below
    for w in &mut pool {
        if !json::id_ok(&w.uid) || !seen.insert(w.uid.clone()) {
            w.uid.clear();
        }
    }
    for (mi, slot) in needs {
        let item = party.members[mi].equipment.get(&slot).cloned().unwrap_or_default();
        let w = take_best(&mut pool, &item).unwrap_or_else(|| Weapon::fresh(d, &item));
        party.members[mi].weapons.insert(slot, w);
    }
    // the stash keeps one instance per stashed copy, in their order: past the count the worst go (owned no more: sold,
    // broken down, given away), short of it fresh ones follow
    let mut fresh = Vec::new();
    for (item, n) in party.stash.iter().filter(|(item, _)| is_weapon(d, item)) {
        let held = pool.iter().filter(|w| &w.item == item).count() as i64;
        for _ in 0..(held - (*n).max(0)).max(0) {
            take_worst(&mut pool, item);
        }
        fresh.extend((held..(*n).max(0)).map(|_| Weapon::fresh(d, item)));
    }
    pool.retain(|w| party.stash.get(&w.item).is_some_and(|n| *n > 0));
    pool.extend(fresh);
    party.stash_weapons = pool;
    for m in &mut party.members {
        let order: Vec<String> = m.equipment.keys().cloned().collect();
        let at = |k: &String| order.iter().position(|s| s == k);
        m.weapons.sort_by(|a, _, b, _| at(a).cmp(&at(b)));
    }
    let mut next = all(party).iter().filter_map(|w| uid_number(&w.uid)).max().unwrap_or(0).saturating_add(1);
    let mut name = |w: &mut Weapon| {
        if w.uid.is_empty() {
            w.uid = format!("{UID_PREFIX}{next}");
            next += 1;
        }
    };
    for m in &mut party.members {
        m.weapons.values_mut().for_each(&mut name);
    }
    party.stash_weapons.iter_mut().for_each(&mut name);
}

/// Removes and returns the best instance of `item` in `pool`: a masterwork first, then the most durability, then the
/// earliest (the one a slot that needs an instance wields).
fn take_best(pool: &mut Vec<Weapon>, item: &str) -> Option<Weapon> {
    let mut best: Option<usize> = None;
    for (i, w) in pool.iter().enumerate().filter(|(_, w)| w.item == item) {
        if best.is_none_or(|b| (w.masterwork, w.durability) > (pool[b].masterwork, pool[b].durability)) {
            best = Some(i);
        }
    }
    best.map(|i| pool.remove(i))
}

/// Removes the worst instance of `item` in `pool`: not masterwork first, then the least durability, then the latest.
fn take_worst(pool: &mut Vec<Weapon>, item: &str) {
    let mut worst: Option<usize> = None;
    for (i, w) in pool.iter().enumerate().filter(|(_, w)| w.item == item) {
        if worst.is_none_or(|b| (w.masterwork, w.durability) <= (pool[b].masterwork, pool[b].durability)) {
            worst = Some(i);
        }
    }
    if let Some(i) = worst {
        pool.remove(i);
    }
}

/// The number of a uid `sync` gave (`w<n>`; None for any other uid).
/// The number of a `w<n>` uid (at most `UID_DIGITS` digits, so the next number never overflows; a longer one is still a
/// valid uid, it just never collides with a new `w<n>`).
fn uid_number(uid: &str) -> Option<u64> {
    uid.strip_prefix(UID_PREFIX).filter(|n| !n.is_empty() && n.len() <= UID_DIGITS && n.bytes().all(|b| b.is_ascii_digit())).and_then(|n| n.parse().ok())
}

/// Every instance of the party: the members' (member order, slot order), then the stash's.
pub fn all(party: &PartyState) -> Vec<&Weapon> {
    party.members.iter().flat_map(|m| m.weapons.values()).chain(&party.stash_weapons).collect()
}

/// The instance with uid `uid`, wherever it is.
pub fn find<'a>(party: &'a PartyState, uid: &str) -> Option<&'a Weapon> {
    all(party).into_iter().find(|w| w.uid == uid)
}

/// The instance member `m` wields in `slot` (None: no instance there, or one of another item than the slot holds).
pub fn wielded<'a>(m: &'a UnitState, slot: &str) -> Option<&'a Weapon> {
    m.weapons.get(slot).filter(|w| m.equipment.get(slot) == Some(&w.item))
}

/// Whether member `m`'s main hand wields a masterwork instance (its own technique opens, `Unit::masterwork`).
pub fn masterwork_main(m: &UnitState) -> bool {
    wielded(m, MAIN_HAND).is_some_and(|w| w.masterwork)
}

/// PK2-S5b (`docs/sessions/PK2_GRAFTS.md` 7): slot -> the durability of each weapon instance member `m` wields whose
/// item wears (`full` more than 0), under world `world` whose ruleset lists the rule `durability` with a manifest
/// `durability.from_act` (`Growth::wear`; empty otherwise, so the unit tracks nothing). A slot keyed by a name the gear
/// does not know is skipped.
pub fn wear_of(d: &Data, m: &UnitState, world: &str) -> IndexMap<rpgkit_rules::gear::Slot, i32> {
    if d.from_act <= 0 || !d.ruleset(world).has(rpgkit_rules::ruleset::DURABILITY) {
        return IndexMap::new();
    }
    m.weapons
        .keys()
        .filter_map(|slot| {
            let w = wielded(m, slot)?;
            let s = rpgkit_rules::gear::Slot::parse(slot)?;
            (full(d, &w.item) > 0).then(|| (s, w.durability.clamp(0, i64::from(i32::MAX)) as i32))
        })
        .collect()
}

/// PK2-S5b: the battle's wear (`HeroEnd::worn`) comes off the instances each fielded member still in the party wields
/// in those slots, each held at 0 or more. Nothing to take, nothing changes.
pub fn wear(gs: &mut GameState, ends: &[roster::HeroEnd]) {
    for e in ends.iter().filter(|e| !e.worn.is_empty()) {
        let Some(m) = gs.party.members.iter_mut().find(|m| m.id == e.member) else { continue };
        for (slot, n) in &e.worn {
            if wielded(m, slot).is_none() {
                continue;
            }
            if let Some(w) = m.weapons.get_mut(slot) {
                w.durability = (w.durability - (*n).max(0)).max(0);
            }
        }
    }
}

/// Equipment by uid: member `member` wields the stashed instance `uid` in `slot`, which must hold the same item; the
/// slot's instance goes to the stash in its place (the item counts do not change). Err (nothing changes) otherwise. It
/// syncs first, so a slot equipped since the last sync holds its instance.
pub fn wield(d: &Data, gs: &mut GameState, member: &str, slot: &str, uid: &str) -> Result<(), String> {
    sync(d, gs);
    let party = &mut gs.party;
    let at = party.stash_weapons.iter().position(|w| w.uid == uid).ok_or_else(|| format!("No weapon '{uid}' in the stash."))?;
    let m = party.members.iter_mut().find(|m| m.id == member).ok_or_else(|| format!("'{member}' is not a party member."))?;
    let item = party.stash_weapons[at].item.clone();
    if m.equipment.get(slot) != Some(&item) || !m.weapons.contains_key(slot) {
        return Err(format!("The {} slot does not hold a weapon of the same item.", slot.replace('_', " ")));
    }
    let incoming = party.stash_weapons.remove(at);
    if let Some(out) = m.weapons.insert(slot.to_string(), incoming) {
        party.stash_weapons.insert(at, out);
    }
    Ok(())
}

/// Drops the saved instances whose item the data no longer has as a weapon (load warnings); `save::decode` runs it
/// before `sync` and the validation, whether instances are active or not.
pub fn sanitize(d: &Data, party: &mut PartyState) -> Vec<String> {
    let mut warnings = Vec::new();
    for m in &mut party.members {
        let id = m.id.clone();
        m.weapons.retain(|slot, w| {
            let keep = is_weapon(d, &w.item);
            if !keep {
                warnings.push(format!("Unit '{id}': weapon '{}' in {slot} ({}) is no weapon in the data any more; dropped.", w.uid, w.item));
            }
            keep
        });
    }
    party.stash_weapons.retain(|w| {
        let keep = is_weapon(d, &w.item);
        if !keep {
            warnings.push(format!("Stash weapon '{}' ({}) is no weapon in the data any more; dropped.", w.uid, w.item));
        }
        keep
    });
    warnings
}

/// The party-wide instance problems: each stash instance's, and a uid used twice (members and stash together).
pub fn party_problems(d: &Data, party: &PartyState) -> Vec<String> {
    let mut out: Vec<String> = party.stash_weapons.iter().flat_map(|w| w.problems(d, "Stash")).collect();
    let mut seen: HashSet<&str> = HashSet::new();
    for w in all(party) {
        if !w.uid.is_empty() && !seen.insert(w.uid.as_str()) {
            out.push(format!("Weapon uid '{}' is used twice.", w.uid));
        }
    }
    out
}
