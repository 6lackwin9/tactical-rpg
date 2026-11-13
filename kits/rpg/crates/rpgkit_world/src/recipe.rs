//! Recipes (PK2-S3, `docs/sessions/PK2_GRAFTS.md` section 4; engine-only, no Godot original): the NPC service
//! `recipe`. A character offers the recipe rows that name it (`npc`) while their `requires` hold (`offered`); running
//! one (`run`) checks everything first on a copy (the target, the cost, every effect), then pays the cost and applies
//! the effects in order, or changes nothing and says why. The member effects: `install` (a graft into a body slot; an
//! occupied slot is overwritten and its graft destroyed), `remove` (the slot emptied, its graft destroyed), `track`
//! (the member's stage on a unit track, never into a departure stage), `max_hp_pct` (a permanent offset, a percent of
//! the member's max HP now), `floor_offset` (for the member's next battle; a member with a graft), `train_proficiency`
//! (one level in a family of the class role, from `to - 1` to `to`), `doctrine` (reserved: nothing yet); the item
//! effect `salvage` (the picked stash item broken down into the effect's gold plus the salvage item its rarity and its
//! theme name). PK2-S4: `drill` {xp} (mastery XP in the picked family, or the member's only masterable one, scaled by
//! its proficiency rate; `mastery::gain`, from `durability.from_act`). PK2-FOLLOWUP (PR5-36, `PK2_GRAFTS.md` 5, 7): the
//! weapon effects `repair` (the picked instance back to `ItemDef::max_durability` for its scaled cost, `repair_cost`)
//! and `reforge` (back to full; `masterwork` makes it masterwork), on a `weapon` recipe while instances are active
//! (`weapons::active`; the run syncs them first), and the member effect `forge_rite` (`UnitState::forge_rite`, for a
//! member with mastery 3 in a family).

use crate::data::Data;
use crate::dialogue;
use crate::progression;
use crate::state::{GameState, UnitState};
use crate::terms;
use crate::weapons;
use aegis_core::json::{self, Dict};
use rpgkit_data::recipe::{self as r, RecipeDef, TARGET_ITEM, TARGET_MEMBER, TARGET_WEAPON};
use serde_json::Value;

/// What a run works on: the party member (a `member` recipe), the stash item (an `item` recipe), the weapon instance's
/// uid (a `weapon` recipe, PK2-FOLLOWUP) and, for a `train_proficiency` recipe whose class role holds more than one
/// family or a `drill` for a member with more than one masterable family, the family ("" = the only one).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pick {
    pub member: String,
    pub item: String,
    pub family: String,
    pub weapon: String,
}

/// The rarity of an item row without one (the schema's default).
pub const RARITY_DEFAULT: &str = "COMMON";
/// The refusal of a recipe the character does not offer.
pub const NOT_OFFERED_TEXT: &str = "That is not offered here.";
/// The refusal of a recipe whose `requires` fail now.
pub const NOT_NOW_TEXT: &str = "That is not available now.";

/// The recipes character `npc_qid` offers now (rows naming it whose `requires` hold), in catalog order.
pub fn offered<'a>(d: &'a Data, gs: &GameState, npc_qid: &str) -> Vec<&'a RecipeDef> {
    d.cat.recipes.values().filter(|rc| offers(d, gs, npc_qid, rc) && open(d, gs, rc)).collect()
}

/// Whether row `rc` names character `npc_qid`.
fn offers(d: &Data, gs: &GameState, npc_qid: &str, rc: &RecipeDef) -> bool {
    !npc_qid.is_empty() && dialogue::character_qid(d, gs, &rc.npc) == npc_qid
}

/// Whether every one of the row's `requires` holds now.
fn open(d: &Data, gs: &GameState, rc: &RecipeDef) -> bool {
    rc.requires.iter().all(|c| dialogue::condition_holds(d, gs, json::dict(c)))
}

/// The cost as lines ("12 gold", "Iron x2"), purse first.
pub fn cost_lines(d: &Data, gs: &GameState, rc: &RecipeDef) -> Vec<String> {
    let mut out = Vec::new();
    if rc.cost.gold > 0 {
        out.push(format!("{} {}", rc.cost.gold, terms::word(d, &gs.world_id, terms::CURRENCY)));
    }
    if rc.cost.supplies > 0 {
        out.push(format!("{} {}", rc.cost.supplies, terms::word(d, &gs.world_id, terms::UPKEEP)));
    }
    out.extend(rc.cost.items.iter().map(|(id, n)| format!("{} x{n}", d.item_name(id))));
    out
}

/// What a recipe of row `rc` may be run on now: the party members (a `member` recipe), the stash items it can break
/// down (an `item` recipe: not quest items, each once), the weapon instances' uids (a `weapon` recipe: the members'
/// in member and slot order, then the stash's, as a sync would leave them; none while instances are not active).
pub fn targets(d: &Data, gs: &GameState, rc: &RecipeDef) -> Vec<String> {
    match rc.target.as_str() {
        TARGET_MEMBER => gs.party.members.iter().map(|m| m.id.clone()).collect(),
        TARGET_ITEM => gs.party.stash.iter().filter(|(id, n)| **n > 0 && breakable(d, id)).map(|(id, _)| id.clone()).collect(),
        TARGET_WEAPON => weapon_targets(d, gs),
        _ => Vec::new(),
    }
}

/// The uids of the party's weapon instances after a sync (empty while instances are not active).
pub fn weapon_targets(d: &Data, gs: &GameState) -> Vec<String> {
    if !weapons::active(d, gs) {
        return Vec::new();
    }
    let mut work = gs.clone();
    weapons::sync(d, &mut work);
    weapons::all(&work.party).into_iter().map(|w| w.uid.clone()).collect()
}

/// The refusal of a `weapon` recipe while weapon instances are not active (`weapons::active`).
pub const NO_WEAPONS_TEXT: &str = "Weapons do not wear yet.";

/// PK2-FOLLOWUP: what a `repair` effect `fx` costs on instance `w` beyond the recipe's own cost: (purse, the salvage
/// item and its count; "" and 0 for none). Each is its full-repair amount (`gold`; `count`, default
/// `REPAIR_COUNT_DEFAULT`) times the missing durability's fraction, rounded up (`repair_share`); the salvage item is the
/// one the weapon's theme names in `themes`, else the one its rarity names in `tiers`, else none (a basic weapon costs
/// purse alone).
pub fn repair_cost(d: &Data, fx: &Dict, w: &weapons::Weapon) -> (i64, String, i64) {
    let max = weapons::full(d, &w.item);
    let missing = (max - w.durability).max(0);
    let gold = r::repair_share(json::i(fx, "gold", 0), missing, max);
    let Some(def) = d.cat.items.get(&w.item) else { return (gold, String::new(), 0) };
    // the schema's default rarity for a row without one
    let rarity = aegis_core::value::string(&def.raw, "rarity", RARITY_DEFAULT);
    let themed = json::s(json::dget(fx, "themes"), &def.theme);
    let salvage = if !def.theme.is_empty() && !themed.is_empty() { themed } else { json::s(json::dget(fx, "tiers"), &rarity) };
    if salvage.is_empty() {
        return (gold, salvage, 0);
    }
    let count = r::repair_share(json::i(fx, "count", r::REPAIR_COUNT_DEFAULT), missing, max);
    (gold, salvage, count)
}

/// The cost lines of row `rc` run on `pick` now: `cost_lines`, then a `repair`'s share for the picked weapon, read on
/// a synced copy as the run and `targets` see the instances (a weapon equipped since the last sync shows its share).
pub fn pick_cost_lines(d: &Data, gs: &GameState, rc: &RecipeDef, pick: &Pick) -> Vec<String> {
    let mut out = cost_lines(d, gs, rc);
    if rc.target != TARGET_WEAPON || !rc.effects.iter().any(|fx| json::s(fx, "effect") == r::FX_REPAIR) {
        return out;
    }
    let mut work = gs.clone();
    weapons::sync(d, &mut work);
    let Some(w) = weapons::find(&work.party, &pick.weapon) else { return out };
    for fx in rc.effects.iter().filter(|fx| json::s(fx, "effect") == r::FX_REPAIR) {
        let (gold, salvage, count) = repair_cost(d, fx, w);
        if gold > 0 {
            out.push(format!("{gold} {}", terms::word(d, &gs.world_id, terms::CURRENCY)));
        }
        if count > 0 {
            out.push(format!("{} x{count}", d.item_name(&salvage)));
        }
    }
    out
}

/// An item a salvage may break down (known, not a quest item).
fn breakable(d: &Data, id: &str) -> bool {
    d.cat.items.get(id).is_some_and(|i| i.item_type != "QUEST")
}

/// The families of member `m`'s class with role `role` (class proficiency order).
pub fn role_families(d: &Data, m: &UnitState, role: &str) -> Vec<String> {
    d.cat.classes.get(&m.class_id(d)).map(|c| c.proficiency_roles.iter().filter(|(_, r)| *r == role).map(|(f, _)| f.clone()).collect()).unwrap_or_default()
}

/// Why recipe `id` of character `npc_qid` can't run on `pick` now (None: it can). Changes nothing.
pub fn refusal(d: &Data, gs: &GameState, npc_qid: &str, id: &str, pick: &Pick) -> Option<String> {
    let mut work = gs.clone();
    plan(d, &mut work, npc_qid, id, pick).err()
}

/// Runs recipe `id` of character `npc_qid` on `pick`: {ok, errors, notes}. Everything is checked first; a refusal
/// changes nothing.
pub fn run(d: &Data, gs: &mut GameState, npc_qid: &str, id: &str, pick: &Pick) -> Dict {
    let mut work = gs.clone();
    match plan(d, &mut work, npc_qid, id, pick) {
        Ok(notes) => {
            *gs = work;
            let mut out = json::ok_result(true, &[]);
            out.insert("notes".into(), json::strs(&notes));
            out
        }
        Err(why) => json::fail(&why),
    }
}

/// The run on `gs` (a copy the caller keeps only on Ok): the offer, the target, the cost, then each effect. Ok(the
/// notes).
fn plan(d: &Data, gs: &mut GameState, npc_qid: &str, id: &str, pick: &Pick) -> Result<Vec<String>, String> {
    let rc = d.cat.recipes.get(id).ok_or_else(|| format!("Unknown recipe '{id}'."))?;
    if !offers(d, gs, npc_qid, rc) {
        return Err(NOT_OFFERED_TEXT.into());
    }
    if !open(d, gs, rc) {
        return Err(NOT_NOW_TEXT.into());
    }
    if let Some(system) = rc.effect_names().iter().find_map(|e| r::pending_system(e)) {
        return Err(format!("{} needs {system}, which is not available yet.", rc.name));
    }
    match rc.target.as_str() {
        TARGET_MEMBER if gs.party.member(&pick.member).is_none() => return Err("Pick a party member.".into()),
        TARGET_ITEM if !breakable(d, &pick.item) || gs.party.stash.get(&pick.item).copied().unwrap_or(0) < 1 => return Err("Pick an item from the stash.".into()),
        TARGET_WEAPON => {
            // PK2-FOLLOWUP: the instances as a sync leaves them (a GUI equip catches up here), then the picked one
            if !weapons::active(d, gs) {
                return Err(NO_WEAPONS_TEXT.into());
            }
            weapons::sync(d, gs);
            if weapons::find(&gs.party, &pick.weapon).is_none() {
                return Err("Pick a weapon.".into());
            }
        }
        _ => {}
    }
    pay(d, gs, rc, pick)?;
    let mut notes = Vec::new();
    for fx in &rc.effects {
        let name = json::s(fx, "effect");
        match name.as_str() {
            r::FX_SALVAGE => salvage(d, gs, fx, &pick.item, &mut notes)?,
            r::FX_REPAIR => repair(d, gs, fx, &pick.weapon, &mut notes)?,
            r::FX_REFORGE => reforge(d, gs, fx, &pick.weapon, &mut notes)?,
            // reserved for a later system (PK4): nothing to apply yet, on any target
            r::FX_DOCTRINE => {}
            _ => member_effect(d, gs, &name, fx, pick, &mut notes)?,
        }
    }
    notes.insert(0, format!("Done: {}.", rc.name));
    Ok(notes)
}

/// Pays the cost (refused when the party can't: the purse, the stash items, the picked item counted on top).
fn pay(d: &Data, gs: &mut GameState, rc: &RecipeDef, pick: &Pick) -> Result<(), String> {
    let w = gs.world_id.clone();
    if rc.cost.gold > gs.party.gold {
        return Err(terms::not_enough_currency(d, &w));
    }
    if rc.cost.supplies > gs.party.supplies {
        return Err(format!("Not enough {}.", terms::word(d, &w, terms::UPKEEP)));
    }
    for (item, n) in &rc.cost.items {
        let picked = i64::from(rc.target == TARGET_ITEM && *item == pick.item);
        if gs.party.stash.get(item).copied().unwrap_or(0) < n + picked {
            return Err(format!("Not enough {}.", d.item_name(item)));
        }
    }
    gs.party.gold -= rc.cost.gold;
    gs.party.supplies -= rc.cost.supplies;
    for (item, n) in &rc.cost.items {
        gs.party.remove_from_stash(item, *n);
    }
    Ok(())
}

/// One member effect on the picked member.
fn member_effect(d: &Data, gs: &mut GameState, name: &str, fx: &Dict, pick: &Pick, notes: &mut Vec<String>) -> Result<(), String> {
    let world = gs.world_id.clone();
    let m = gs.party.member_mut(&pick.member).ok_or("Pick a party member.")?;
    let who = progression::display_name(d, m);
    match name {
        r::FX_INSTALL => {
            let (slot, item) = (json::s(fx, "slot"), json::s(fx, "item"));
            if let Some(old) = m.install_graft(d, &slot, &item).map_err(|e| format!("{who}: {e}"))? {
                notes.push(format!("{} is destroyed.", d.item_name(&old)));
            }
            notes.push(format!("{who} takes {}.", d.item_name(&item)));
        }
        r::FX_REMOVE => {
            let slot = json::s(fx, "slot");
            let old = m.remove_graft(&slot).ok_or_else(|| format!("{who}: body slot {slot} is empty."))?;
            notes.push(format!("{} is removed and destroyed.", d.item_name(&old)));
        }
        r::FX_TRACK => track(d, m, &who, fx, notes)?,
        r::FX_MAX_HP_PCT => {
            let max = progression::max_hp_of(d, m, &world);
            if max < 1 {
                return Err(format!("{who} can't take that now."));
            }
            let delta = max_hp_delta(max, json::i(fx, "pct", 0));
            m.max_hp_offset += delta;
            notes.push(format!("{who}: max HP {delta:+} for good."));
        }
        r::FX_FLOOR_OFFSET => {
            if m.graft_count() == 0 {
                return Err(format!("{who} holds no graft."));
            }
            m.floor_offset += json::i(fx, "delta", 0);
        }
        r::FX_TRAIN => train(d, m, &who, fx, &pick.family, notes)?,
        r::FX_DRILL => {
            // PK2-S4: mastery XP in the picked family (or the member's only masterable one), scaled by its rate
            let family = crate::mastery::drill_family(d, m, &pick.family);
            if !crate::mastery::open(d, gs) {
                return Err(crate::mastery::NOT_YET_TEXT.into());
            }
            let g = crate::mastery::gain(d, gs, &pick.member, &family?, json::i(fx, "xp", 0))?;
            notes.push(crate::mastery::note(d, gs, &g));
        }
        r::FX_FORGE_RITE => {
            // PK2-FOLLOWUP (FB-214 (d)): the forge rite, for a member with mastery 3 in a family (its keystone opens)
            if m.forge_rite {
                return Err(format!("{who} has taken the forge rite already."));
            }
            let top = rpgkit_data::mastery::MASTERY_MAX;
            if !m.mastery_xp.values().any(|xp| rpgkit_data::mastery::level_of(*xp) >= top) {
                return Err(format!("{who} needs mastery {top} in a weapon family first."));
            }
            m.set_forge_rite(true);
            notes.push(format!("{who} takes the forge rite."));
        }
        other => return Err(format!("Unknown recipe effect '{other}'.")),
    }
    Ok(())
}

/// `max_hp_pct`: `pct` percent of `max`, rounded half away from zero, at least 1 either way.
pub fn max_hp_delta(max: i64, pct: i64) -> i64 {
    let raw = (max * pct) as f64 / 100.0;
    let delta = raw.round() as i64;
    if delta == 0 && pct != 0 {
        pct.signum()
    } else {
        delta
    }
}

/// `track`: the stage moves by `delta` within the row (a lowering needs a stage to lower; a raise needs room and never
/// reaches a departure stage).
fn track(d: &Data, m: &mut UnitState, who: &str, fx: &Dict, notes: &mut Vec<String>) -> Result<(), String> {
    let id = json::s(fx, "track");
    let row = d.cat.unit_tracks.get(&id).ok_or_else(|| format!("Unknown unit track '{id}'."))?;
    let (stage, delta) = (m.track_stage(&id), json::i(fx, "delta", 0));
    let top = i64::from(row.top());
    if delta < 0 && stage == 0 {
        return Err(format!("{who} has no {} to lower.", row.name));
    }
    if delta > 0 && stage >= top {
        return Err(format!("{who} is at the last stage of {}.", row.name));
    }
    let next = (stage + delta).clamp(0, top);
    if !row.departure_at(row.clamp(next)).is_empty() {
        return Err(format!("{who} would reach the last stage of {}.", row.name));
    }
    if next == 0 {
        m.tracks.shift_remove(&id);
    } else {
        m.tracks.insert(id.clone(), next);
    }
    let stage_name = row.stage(row.clamp(next)).map(|s| s.name.clone()).unwrap_or_default();
    notes.push(if next == 0 { format!("{who} is free of {}.", row.name) } else { format!("{who}: {} stage {next} ({stage_name}).", row.name) });
    Ok(())
}

/// `train_proficiency`: one level in a family of the member's class role `role`, from `to - 1` to `to` (the picked
/// family, or the role's only family).
fn train(d: &Data, m: &mut UnitState, who: &str, fx: &Dict, family: &str, notes: &mut Vec<String>) -> Result<(), String> {
    let (role, to) = (json::s(fx, "role"), json::i(fx, "to", 0));
    let fams = role_families(d, m, &role);
    let fam = match (family.is_empty(), fams.len()) {
        (_, 0) => return Err(format!("{who}'s class has no {role} family.")),
        (true, 1) => fams[0].clone(),
        (true, _) => return Err(format!("Pick a {role} family ({}).", fams.join(", "))),
        (false, _) if fams.iter().any(|f| f == family) => family.to_string(),
        (false, _) => return Err(format!("{family} is not a {role} family of {who}'s class.")),
    };
    let class = d.cat.classes.get(&m.class_id(d)).ok_or_else(|| format!("{who} has no class."))?;
    let level = i64::from(m.proficiency(d, class).level_of(&fam));
    if level != to - 1 {
        return Err(format!("{who} is at {fam} level {level}; this training takes level {} to {to}.", to - 1));
    }
    *m.trained.entry(fam.clone()).or_insert(0) += 1;
    notes.push(format!("{who}: {fam} level {to}."));
    Ok(())
}

/// PK2-FOLLOWUP: `repair`: the picked weapon instance pays its share (`repair_cost`; refused when the party can't, a
/// weapon that never wears, or one already whole) and goes back to full durability.
fn repair(d: &Data, gs: &mut GameState, fx: &Dict, uid: &str, notes: &mut Vec<String>) -> Result<(), String> {
    let w = weapons::find(&gs.party, uid).cloned().ok_or("Pick a weapon.")?;
    let (name, max) = (d.item_name(&w.item), weapons::full(d, &w.item));
    if max <= 0 {
        return Err(format!("{name} never wears."));
    }
    if w.durability >= max {
        return Err(format!("{name} needs no repair."));
    }
    let (gold, salvage, count) = repair_cost(d, fx, &w);
    if gold > gs.party.gold {
        return Err(terms::not_enough_currency(d, &gs.world_id));
    }
    if count > 0 && !gs.party.remove_from_stash(&salvage, count) {
        return Err(format!("Not enough {}.", d.item_name(&salvage)));
    }
    gs.party.gold -= gold;
    if let Some(w) = weapons::find_mut(&mut gs.party, uid) {
        w.durability = max;
    }
    notes.push(format!("{name} is repaired ({max}/{max})."));
    Ok(())
}

/// PK2-FOLLOWUP: `reforge`: the picked weapon instance goes back to full durability and, with `masterwork`, becomes
/// masterwork (its own technique opens in the main hand); refused when that changes nothing. AUDIT-08 (PR5-40): a
/// masterwork reforge of a weapon that is already masterwork is refused even when it is worn (a plain `reforge` or a
/// `repair` restores it), so the masterwork recipe is never a costlier repair; ACT1-HOOKS2 (FB-243 (c)): the refusal
/// says so.
fn reforge(d: &Data, gs: &mut GameState, fx: &Dict, uid: &str, notes: &mut Vec<String>) -> Result<(), String> {
    let w = weapons::find_mut(&mut gs.party, uid).ok_or("Pick a weapon.")?;
    let (name, max) = (d.item_name(&w.item), weapons::full(d, &w.item));
    if json::b(fx, "masterwork", false) && w.masterwork {
        return Err(format!("{name} is already a masterwork. Use a plain reforge or a repair to restore it."));
    }
    let make = json::b(fx, "masterwork", false) && !w.masterwork;
    if !make && w.durability >= max {
        return Err(format!("{name} needs no reforging."));
    }
    w.durability = w.durability.max(max);
    w.masterwork |= make;
    notes.push(if make { format!("{name} is reforged into a masterwork.") } else { format!("{name} is reforged ({max}/{max}).") });
    Ok(())
}

/// `salvage`: the picked stash item breaks down into the effect's gold, the salvage item its rarity names in `tiers`
/// and the one its theme names in `themes` (one each); an item that would give nothing is refused.
fn salvage(d: &Data, gs: &mut GameState, fx: &Dict, item: &str, notes: &mut Vec<String>) -> Result<(), String> {
    let def = d.cat.items.get(item).ok_or_else(|| format!("Unknown item '{item}'."))?;
    let gold = json::i(fx, "gold", 0).max(0);
    let mut gives: Vec<String> = Vec::new();
    // the schema's default rarity for a row without one
    let rarity = aegis_core::value::string(&def.raw, "rarity", RARITY_DEFAULT);
    for (key, k) in [("tiers", rarity.as_str()), ("themes", def.theme.as_str())] {
        let out = json::s(json::dget(fx, key), k);
        if !k.is_empty() && !out.is_empty() {
            gives.push(out);
        }
    }
    if gold == 0 && gives.is_empty() {
        return Err(format!("{} gives nothing here.", def.name));
    }
    if !gs.party.remove_from_stash(item, 1) {
        return Err(format!("Not enough {}.", def.name));
    }
    let mut got = Vec::new();
    if gold > 0 {
        let added = gs.party.add_gold(gold);
        got.push(format!("{added} {}", terms::word(d, &gs.world_id, terms::CURRENCY)));
    }
    for id in gives {
        gs.party.add_to_stash(&id, 1);
        got.push(d.item_name(&id));
    }
    notes.push(format!("{} breaks down into {}.", def.name, got.join(", ")));
    Ok(())
}

/// PK2-FOLLOWUP (FB-212 (d)): the families row `rc` lets the pick name for its member `pick.member`, in order and
/// each once: a `train_proficiency`'s class role families and a `drill`'s masterable families, each only when it has
/// more than one (with one, the run takes it unnamed). Empty: the row needs no family pick.
pub fn families(d: &Data, gs: &GameState, rc: &RecipeDef, pick: &Pick) -> Vec<String> {
    let Some(m) = gs.party.member(&pick.member).filter(|_| rc.target == TARGET_MEMBER) else { return Vec::new() };
    let mut out: Vec<String> = Vec::new();
    for fx in &rc.effects {
        let fams = match json::s(fx, "effect").as_str() {
            r::FX_TRAIN => role_families(d, m, &json::s(fx, "role")),
            r::FX_DRILL => crate::mastery::masterable_families(d, m),
            _ => continue,
        };
        if fams.len() > 1 {
            out.extend(fams.into_iter().filter(|f| !out.contains(f)).collect::<Vec<_>>());
        }
    }
    out
}

/// The pick row `rc` runs on: `pick` with its family kept only when the row offers that family (`families`), so one
/// family picker serves every row of a pane (a row with a single family takes it unnamed).
pub fn pick_for(d: &Data, gs: &GameState, rc: &RecipeDef, pick: &Pick) -> Pick {
    let keep = !pick.family.is_empty() && families(d, gs, rc, pick).contains(&pick.family);
    Pick { family: if keep { pick.family.clone() } else { String::new() }, ..pick.clone() }
}

/// The pane's reading of character `npc_qid`'s recipes for `pick`: [{id, name, description, target, cost: [line],
/// targets: [id], families: [id], ready, reason}] in catalog order (`reason` "" when ready; each row read with
/// `pick_for`; `cost` with a `repair`'s share for the picked weapon, `pick_cost_lines`).
pub fn view(d: &Data, gs: &GameState, npc_qid: &str, pick: &Pick) -> Vec<Value> {
    offered(d, gs, npc_qid)
        .into_iter()
        .map(|rc| {
            let pick = &pick_for(d, gs, rc, pick);
            let why = refusal(d, gs, npc_qid, &rc.id, pick).unwrap_or_default();
            Value::Object(json::obj([
                ("id", rc.id.clone().into()),
                ("name", rc.name.clone().into()),
                ("description", rc.description.clone().into()),
                ("target", rc.target.clone().into()),
                ("cost", json::strs(&pick_cost_lines(d, gs, rc, pick))),
                ("targets", json::strs(&targets(d, gs, rc))),
                ("families", json::strs(&families(d, gs, rc, pick))),
                ("ready", Value::Bool(why.is_empty())),
                ("reason", why.into()),
            ]))
        })
        .collect()
}
