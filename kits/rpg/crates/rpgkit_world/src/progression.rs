//! XpCurve, LevelUp, PartyBuilder and HealRules (scripts/progression/): levels from XP, awards, confirmed spends,
//! a saved member as a battle unit, and HP between battles (HEAL1-L).

use crate::data::Data;
use crate::state::{GameState, PartyState, UnitState};
use aegis_core::json::{self, int, Dict};
use rpgkit_data::STATS;
use rpgkit_rules::gear::{Equipment, Slot};
use rpgkit_rules::stats::MAX_LEVEL;
use rpgkit_rules::{build_unit, Unit};
use serde_json::Value;

// ---- XpCurve -------------------------------------------------------------------------------------------------

pub fn xp_for_level(d: &Data, level: i64) -> i64 {
    let n = (level.clamp(1, MAX_LEVEL as i64) - 1) as usize;
    d.xp.xp_to_next.iter().take(n).sum()
}

pub fn level_for(d: &Data, total_xp: i64) -> i64 {
    let mut level = 1;
    while level < MAX_LEVEL as i64 && total_xp >= xp_for_level(d, level + 1) {
        level += 1;
    }
    level
}

pub fn xp_to_next(d: &Data, total_xp: i64) -> i64 {
    let level = level_for(d, total_xp);
    if level >= MAX_LEVEL as i64 {
        0
    } else {
        xp_for_level(d, level + 1) - total_xp
    }
}

pub fn kill_xp(d: &Data, enemy_level: i64, enemy_tier: &str, member_level: i64) -> i64 {
    let k = &d.xp;
    let tier_mult = k.tier_mult.get(enemy_tier).copied().unwrap_or(1.0);
    let delta = (1.0 + k.level_delta_step * (enemy_level - member_level) as f64).clamp(k.level_delta_min, k.level_delta_max);
    let raw = (k.base + k.per_level * enemy_level as f64) * tier_mult * delta;
    json::roundi(raw).max(1)
}

/// REWARD1 3.1: an enemy's weight in its encounter's purse, its kill XP at equal level in integer points:
/// round(100 x (base + per_level x level) x tier_mult), at least 1.
pub fn kill_weight(d: &Data, level: i64, tier: &str) -> i64 {
    let k = &d.xp;
    let tier_mult = k.tier_mult.get(tier).copied().unwrap_or(1.0);
    json::roundi(100.0 * (k.base + k.per_level * level as f64) * tier_mult).max(1)
}

/// `enemies`: (level, tier) per defeated enemy.
pub fn encounter_xp(d: &Data, enemies: &[(i64, String)], member_level: i64) -> i64 {
    enemies.iter().map(|(l, t)| kill_xp(d, *l, t, member_level)).sum()
}

// ---- LevelUp -------------------------------------------------------------------------------------------------

/// {levels_gained, old_level, new_level, unspent}.
pub fn award(d: &Data, state: &mut UnitState, xp: i64) -> Dict {
    let old = state.level(d);
    state.total_xp += xp.max(0);
    let new = state.level(d);
    json::obj([("levels_gained", int(new - old)), ("old_level", int(old)), ("new_level", int(new)), ("unspent", int(state.unspent(d)))])
}

/// Full encounter XP to every active member; member id -> award result + "xp".
pub fn award_encounter(d: &Data, party: &mut PartyState, enemies: &[(i64, String)]) -> Dict {
    let mut out = Dict::new();
    for i in party.active_indexes() {
        let m = &mut party.members[i];
        let xp = encounter_xp(d, enemies, m.level(d));
        let mut res = award(d, m, xp);
        res.insert("xp".into(), int(xp));
        out.insert(m.id.clone(), Value::Object(res));
    }
    out
}

/// Adds `draft` (stat -> points) to the spend; errors leave the state untouched.
pub fn confirm(d: &Data, state: &mut UnitState, draft: &Dict) -> Vec<String> {
    let mut errors = Vec::new();
    let mut sum = 0;
    for (k, v) in draft {
        if !STATS.contains(&k.as_str()) {
            errors.push(format!("Unknown stat '{k}'."));
        } else if !json::is_int(v) || json::to_int(v) < 0 {
            errors.push(format!("Draft {k} must be a whole number >= 0."));
        } else {
            sum += json::to_int(v);
        }
    }
    if sum > state.unspent(d) {
        errors.push(format!("Draft spends {sum}, only {} unspent.", state.unspent(d)));
    }
    if errors.is_empty() {
        for (k, v) in draft {
            *state.spend.entry(k.clone()).or_insert(0) += json::to_int(v);
        }
    }
    errors
}

/// Spends every unspent point by the class growth weights.
pub fn auto_for(d: &Data, state: &mut UnitState) {
    let left = state.unspent(d);
    if left <= 0 {
        return;
    }
    let growth = d.cat.classes.get(&state.class_id(d)).map(|c| c.growth.clone()).unwrap_or_default();
    for (k, v) in rpgkit_rules::stats::auto_spend(&growth, left as i32) {
        *state.spend.entry(k).or_insert(0) += v as i64;
    }
}

// ---- PartyBuilder --------------------------------------------------------------------------------------------

/// The world character a member plays ({} when lore_ref is empty or unresolved).
pub fn character_of<'a>(d: &'a Data, state: &UnitState) -> &'a Dict {
    match aegis_content::split_qid(&state.lore_ref) {
        Some((w, _)) if d.has_world(&w) => d.resolve(&w, "characters", &state.lore_ref),
        _ => json::empty(),
    }
}

/// The member's name: its character's, else the base unit's, else its id.
pub fn display_name(d: &Data, state: &UnitState) -> String {
    let c = character_of(d, state);
    let name = json::s(c, "name");
    if !c.is_empty() && !name.is_empty() {
        return name;
    }
    json::s_or(d.entry("units", &state.base_unit), "name", &state.id)
}

pub struct Built {
    pub unit: Unit,
    /// Saved pouch ids that found no battle-pouch slot.
    pub pouch_unloaded: Vec<String>,
    pub warnings: Vec<String>,
}

/// PartyBuilder.build_report with auto-spend: None when the state is invalid. World-free and classic: the base
/// catalog's class path, no deck rule (tools and tests; a screen with a world uses `build_played` or `build_with`).
pub fn build(d: &Data, state: &UnitState) -> Option<Built> {
    build_as(d, state, "", true, None)
}

/// `build` as world `world` fights under `deck` (RULESET1 9 D1-D5 when Some: the member's saved deck, else its base
/// unit's, slots its passives (a slottable passive the member owns applies only while slotted, D3, D4), the ruleset's
/// basics are open and the deck fills the active slots, D2; None: classic). HOOKS1 CN3 (decision "HOOKS1-UI slice 1
/// answers", user 2026-10-30): the class path is the world's (its `classes` overlay), as the class name is.
pub fn build_in(d: &Data, state: &UnitState, world: &str, deck: Option<&rpgkit_rules::DeckRule>) -> Option<Built> {
    build_as(d, state, world, true, deck)
}

/// The deck rule of world `world`'s battles (None in classic).
pub fn deck_rule(d: &Data, world: &str) -> Option<rpgkit_rules::DeckRule> {
    rpgkit_rules::DeckRule::of(&d.ruleset(world))
}

/// The member as it fights in world `world`: its class path there and that world's deck rule (RULESET1-U slice 2 review:
/// out-of-battle max HP, the hub roster, camp rest).
pub fn build_played(d: &Data, state: &UnitState, world: &str) -> Option<Built> {
    build_as(d, state, world, true, deck_rule(d, world).as_ref())
}

/// `build_played` with `auto_spend` false leaving unspent points unspent (the party screen's preview: confirmed and
/// drafted points only).
pub fn build_with(d: &Data, state: &UnitState, world: &str, auto_spend: bool) -> Option<Built> {
    build_as(d, state, world, auto_spend, deck_rule(d, world).as_ref())
}

/// Every build: the member in world `world` (its class path; "" = the base catalog), auto-spent or not, under `deck`.
pub fn build_as(d: &Data, state: &UnitState, world: &str, auto_spend: bool, deck: Option<&rpgkit_rules::DeckRule>) -> Option<Built> {
    if !state.validate(d).is_empty() {
        return None;
    }
    // HOOKS1 CN3: the world may replace the class path (the base catalog when it does not)
    let cat = d.cat_in(world);
    let mut work = state.clone();
    if auto_spend {
        auto_for(d, &mut work);
    }
    let mut def = cat.units.get(&work.base_unit)?.clone();
    def.level = work.level(d) as i32;
    def.spend = work.spend.iter().map(|(k, v)| (k.clone(), *v as i32)).collect();
    if def.spend.is_empty() {
        // ContentFactory.NO_AUTO_SPEND: an empty spend must not be auto-spent by the factory either
        def.spend.insert(rpgkit_data::STATS[0].to_string(), 0);
    }
    def.name = display_name(d, &work);
    let mut u = build_unit(cat, &def, 0);
    if let Some(rule) = deck {
        // RULESET1 9 D3-D5: the member's deck replaces its base unit's; the rule gates the passives below
        if !work.deck.is_empty() {
            u.deck = work.deck.clone();
        }
        u.deck_rule = Some(rule.clone());
    }
    for id in &work.passives {
        if let Some(p) = cat.passives.get(id) {
            u.apply_passive(cat, p);
        }
    }
    for sk in &work.learned_skills {
        u.learn_skill_checked(cat, sk, false);
    }
    for (slot_name, item_id) in &work.equipment {
        let (Some(slot), Some(item)) = (Slot::parse(slot_name), cat.items.get(item_id)) else { continue };
        if item.is_equipment() {
            u.inventory.unequip(slot);
            u.inventory.equip(Equipment::new(item), slot);
        }
    }
    // The two-handed rule: a class default shield or focus is not held under a two-handed weapon (state::off_hand_ok).
    if let (Some(main), Some(off)) = (u.inventory.get(Slot::MainHand), u.inventory.get(Slot::OffHand)) {
        if !crate::state::off_hand_ok(Some(&main.item), &off.item) {
            u.inventory.unequip(Slot::OffHand);
        }
    }
    u.inventory.pouch.clear();
    let mut unloaded = Vec::new();
    for id in &work.pouch {
        if let Some(item) = cat.items.get(id).filter(|i| i.item_type == "CONSUMABLE") {
            if !u.inventory.add_to_pouch(item) {
                unloaded.push(id.clone());
            }
        }
    }
    u.sync_from_stats_and_gear(cat);
    u.hp = u.max_hp;
    let mut warnings = Vec::new();
    if !work.loadout_class.is_empty() || !work.loadout_extra.is_empty() {
        if let Err(err) = u.set_loadout(cat, work.loadout_class.clone(), work.loadout_extra.clone()) {
            warnings.push(format!("Unit '{}': stored loadout ignored ({err}).", work.id));
        }
    }
    // RULESET1 9 D1, D2: the deck (else the loadout, auto) fills the active slots
    u.apply_deck(cat);
    // SKILLUI1 R4: a hero built from member state uses the ranks chosen on the Skills tab (set after gear, passives
    // and the loadout, so every unlock is final; fixed for the battle like the loadout).
    u.rank_choice = Some(work.loadout_ranks.iter().map(|(k, v)| (k.clone(), *v as i32)).collect());
    Some(Built { unit: u, pouch_unloaded: unloaded, warnings })
}

/// SKILLUI1 (the user's answer to Q2): every slotted family without a rank entry gets its highest unlocked rank now,
/// so the slot stays there when new ranks unlock (the Skills tab calls this on slotting and on Confirm). Slotted
/// means the built unit's active families, so the class default loadout (stored empty) is pinned too.
pub fn pin_ranks(d: &Data, state: &mut UnitState, world: &str) {
    let Some(b) = build_with(d, state, world, false) else { return };
    for fam in &b.unit.active_families() {
        if !state.loadout_ranks.contains_key(fam) {
            if let Some(&r) = b.unit.selectable_ranks(&d.cat, fam).last() {
                state.loadout_ranks.insert(fam.clone(), r as i64);
            }
        }
    }
}

// ---- HealRules -----------------------------------------------------------------------------------------------

pub const RALLY_PCT: i64 = 25;
pub const CAMP_COST: i64 = 3;
pub const REFUSE_NOT_AT_NODE: &str = "Make camp on the world map.";
pub const REFUSE_RESTED: &str = "Everyone is rested.";
/// TERMS1: the upkeep's default word (`terms::UPKEEP`) with CAMP_COST: only a world without a campaign refuses a Camp
/// with it, and there the purse, so the upkeep, has its default name (the campaign rest refuses with
/// `campaign::purse::REFUSE_UPKEEP`); `tests/terms.rs` checks it against the register.
pub const REFUSE_SUPPLIES: &str = "Making camp needs 3 supplies.";

pub fn rally(max_hp: i64) -> i64 {
    (max_hp as f64 * RALLY_PCT as f64 / 100.0).ceil() as i64
}

pub fn hp_after(hp: i64, max_hp: i64, outcome: &str) -> i64 {
    match outcome {
        "won" if hp <= 0 => max_hp.min(rally(max_hp)),
        "won" => max_hp.min(hp + rally(max_hp)),
        "lost" => max_hp,
        _ => hp.clamp(1, max_hp.max(1)),
    }
}

pub fn start_hp(max_hp: i64, hp_lost: i64) -> i64 {
    (max_hp - hp_lost.max(0)).clamp(1, max_hp.max(1))
}

/// The member's max HP as it fights in world `world` (`build_played`: under its deck rule an unslotted passive does not
/// count, as in battle).
pub fn max_hp_of(d: &Data, state: &UnitState, world: &str) -> i64 {
    build_played(d, state, world).map(|b| b.unit.max_hp as i64).unwrap_or(0)
}

/// (hp, max) for a read-out in world `world`.
pub fn current_hp(d: &Data, state: &UnitState, world: &str) -> (i64, i64) {
    let max = max_hp_of(d, state, world);
    (if max > 0 { start_hp(max, state.hp_lost) } else { 0 }, max)
}

/// Whether a rest would change anything: a member has lost HP or carries `wounded` (ROSTER1 W3, CAMPAIGN1 R2; no
/// classic member is ever wounded).
pub fn hurt(party: &PartyState) -> bool {
    party.members.iter().any(|m| m.hp_lost > 0 || m.has_condition(crate::roster::WOUNDED))
}

/// Every roster member back to full; the HP restored in total (max HP as the members fight in world `world`). ROSTER1
/// W3: every rest path comes here, so `roster::on_rest` clears `wounded` here too.
pub fn rest(d: &Data, party: &mut PartyState, world: &str) -> i64 {
    crate::roster::on_rest(party);
    let mut restored = 0;
    for m in party.members.iter_mut() {
        if m.hp_lost <= 0 {
            continue;
        }
        let max = max_hp_of(d, m, world);
        if max > 0 {
            restored += max - start_hp(max, m.hp_lost);
        }
        m.hp_lost = 0;
    }
    restored
}

pub fn is_rest_place(d: &Data, gs: &GameState, location_ref: &str) -> bool {
    !location_ref.is_empty() && json::b(crate::explore::resolve_location(d, gs, location_ref), "rest", false)
}

/// HealRules.can_camp; in a campaign world the rest rule's refusal (CAMPAIGN1 R1, R2: `campaign::purse::rest_refusal`).
pub fn can_camp(d: &Data, gs: &GameState) -> &'static str {
    if gs.campaign.is_some() {
        return crate::campaign::purse::rest_refusal(d, gs);
    }
    if crate::travel::current_node(d, gs).is_empty() {
        return REFUSE_NOT_AT_NODE;
    }
    if !hurt(&gs.party) {
        return REFUSE_RESTED;
    }
    if gs.party.supplies < CAMP_COST {
        return REFUSE_SUPPLIES;
    }
    ""
}

/// {ok, errors, spent, restored, events}. In a campaign world the rest rule's rest (CAMPAIGN1 R2: `campaign::purse::rest`,
/// adding `days` and `campaign?`).
pub fn make_camp(d: &Data, gs: &mut GameState) -> Dict {
    if gs.campaign.is_some() {
        return crate::campaign::purse::rest(d, gs);
    }
    let refusal = can_camp(d, gs);
    if !refusal.is_empty() {
        let mut out = json::fail(refusal);
        out.insert("spent".into(), int(0));
        out.insert("restored".into(), int(0));
        out.insert("events".into(), Value::Array(Vec::new()));
        return out;
    }
    let spent = gs.party.spend_supplies(CAMP_COST);
    let restored = rest(d, &mut gs.party, &gs.world_id.clone());
    let events = vec![
        Value::Object(json::obj([("kind", "camped".into()), ("location", gs.location_id.clone().into()), ("spent", int(spent)), ("restored", int(restored))])),
        Value::Object(json::obj([("kind", "autosave".into())])),
    ];
    let mut out = json::ok_result(true, &[]);
    out.insert("spent".into(), int(spent));
    out.insert("restored".into(), int(restored));
    out.insert("events".into(), Value::Array(events));
    out
}

/// explore.last_rest as a qid when it still resolves, else "".
pub fn last_rest(d: &Data, gs: &GameState) -> String {
    let r = json::s(&gs.explore, "last_rest");
    if r.is_empty() {
        return String::new();
    }
    json::s(crate::explore::resolve_location(d, gs, &r), "_qid")
}

/// {mode, spent, to} without changing anything (HEAL1 rule 3.3.3).
pub fn defeat_plan(d: &Data, gs: &GameState, checkpoint: &str) -> Dict {
    let at = if checkpoint.is_empty() { crate::explore::start_location(d, &gs.world_id) } else { checkpoint.to_string() };
    if is_rest_place(d, gs, &at) {
        return json::obj([("mode", "rest_place".into()), ("spent", int(0)), ("to", checkpoint.into())]);
    }
    if gs.party.supplies >= CAMP_COST {
        return json::obj([("mode", "forced_rest".into()), ("spent", int(CAMP_COST)), ("to", checkpoint.into())]);
    }
    json::obj([("mode", "back_to_town".into()), ("spent", int(0)), ("to", last_rest(d, gs).into())])
}

pub fn settle_defeat(gs: &mut GameState, plan: &Dict) {
    gs.party.spend_supplies(json::i(plan, "spent", 0));
    for m in gs.party.members.iter_mut() {
        m.hp_lost = 0;
    }
}
