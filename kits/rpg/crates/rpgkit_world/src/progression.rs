//! XpCurve, LevelUp, PartyBuilder and HealRules (scripts/progression/): levels from XP, awards, confirmed spends,
//! a saved member as a battle unit, and HP between battles (HEAL1-L).

use crate::data::Data;
use crate::state::{GameState, PartyState, UnitState};
use aegis_core::json::{self, int, Dict};
use rpgkit_data::{UnitDef, NO_ITEM, STATS};
use rpgkit_rules::gear::Slot;
use rpgkit_rules::stats::MAX_LEVEL;
use rpgkit_rules::{build_from, Growth, Overrides, Unit, UnitBuild};
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
    // U2.2-S3 (FB-46): the class growth shares (an integer row's growth / 5), not level 2's rounded gain; R4-1 slice 2
    // (FB-144): read through the base unit's role template (`spend_growth`; a party member never has one)
    let growth = match d.cat.units.get(&state.base_unit) {
        Some(def) => rpgkit_rules::unit::spend_growth(&d.cat, def),
        None => d.cat.classes.get(&state.class_id(d)).map(|c| c.unit_shares().growth).unwrap_or_default(),
    };
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
    /// The row it was built from (its base unit at its level and spend; the AI profile reads it).
    pub def: UnitDef,
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
    build_unchecked(d, state, world, auto_spend, deck)
}

/// `build_as` without the validation first (PR5-26: `UnitState::deck_errors` builds the member's automatic deck through
/// it, so validating there would recurse); the caller answers for the state.
pub(crate) fn build_unchecked(d: &Data, state: &UnitState, world: &str, auto_spend: bool, deck: Option<&rpgkit_rules::DeckRule>) -> Option<Built> {
    // HOOKS1 CN3: the world may replace the class path (the base catalog when it does not)
    let cat = d.cat_in(world);
    let mut work = state.clone();
    if auto_spend {
        auto_for(d, &mut work);
    }
    let mut def = cat.units.get(&work.base_unit)?.clone();
    // PR5-26 (audit B2): a slot the member holds empty (`NO_ITEM`, `roster::stash_forbidden`) takes no default either
    for (slot, _) in work.equipment.iter().filter(|(_, id)| *id == NO_ITEM) {
        def.slots.insert(slot.clone(), NO_ITEM.to_string());
    }
    def.level = work.level(d) as i32;
    def.spend = work.spend.iter().map(|(k, v)| (k.clone(), *v as i32)).collect();
    if def.spend.is_empty() {
        // ContentFactory.NO_AUTO_SPEND: an empty spend must not be auto-spent by the factory either
        def.spend.insert(rpgkit_data::STATS[0].to_string(), 0);
    }
    def.name = display_name(d, &work);
    // U4.2-S3 (FB-29): the character's AI role and default intent over the base unit's (the fielded hero's AI profile)
    let character = character_of(d, &work);
    for (key, field) in [("ai_role", &mut def.ai_role), ("default_intent", &mut def.default_intent)] {
        let v = json::s(character, key);
        if !v.is_empty() {
            *field = v;
        }
    }
    let growth = Growth {
        deck: work.deck.clone(),
        learned_skills: work.learned_skills.clone(),
        equipment: work.equipment.iter().filter_map(|(slot, id)| Some((Slot::parse(slot)?, cat.items.get(id)?.clone()))).collect(),
        pouch: work.pouch.iter().filter_map(|id| cat.items.get(id).cloned()).collect(),
        loadout: (!work.loadout_class.is_empty() || !work.loadout_extra.is_empty()).then(|| (work.loadout_class.clone(), work.loadout_extra.clone())),
        ranks: work.loadout_ranks.iter().map(|(k, v)| (k.clone(), *v as i32)).collect(),
        pouch_stacks: d.ruleset(world).numbers.pouch_stacks,
        // PK2-S1: the member's tracks (a battle mark advances them) and its permanent max HP offset
        max_hp_offset: work.max_hp_offset.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        tracks: work.tracks.iter().map(|(k, v)| (k.clone(), (*v).clamp(0, i64::from(i32::MAX)) as i32)).collect(),
        // PK2-S2: the grafts in its body slots (catalog slot order) and its floor offset for this battle
        body: work.body_items(d),
        floor_offset: work.floor_offset.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        // PK2-S3: the proficiency levels it trained
        trained: work.trained.iter().map(|(k, v)| (k.clone(), (*v).clamp(0, i64::from(rpgkit_rules::stats::PROF_MAX_LEVEL)) as i32)).collect(),
        // PK2-S4: its mastery levels (the techniques of the wielded family) and its forge rite; PK2-S5a: a masterwork
        // main-hand instance opens the weapon's own technique
        mastery: work.mastery_xp.iter().map(|(k, v)| (k.clone(), rpgkit_data::mastery::level_of(*v))).filter(|(_, l)| *l > 0).collect(),
        masterwork: crate::weapons::masterwork_main(&work),
        forge_rite: work.forge_rite,
        // PK2-S5b: the durability of its wielded instances that wear (none without the rule durability)
        wear: crate::weapons::wear_of(d, &work, world),
    };
    let over = Overrides { passives: member_passives(d, &work), ..Overrides::default() };
    // U3.2-S3: the gear is read under the world's ruleset's wield terms
    let wield = d.ruleset(world).numbers.wield();
    let b = build_from(cat, UnitBuild { row: &def, default_ap: 0, deck, wield, over, growth: Some(growth) });
    let warnings = b.loadout_error.map(|err| format!("Unit '{}': stored loadout ignored ({err}).", work.id)).into_iter().collect();
    Some(Built { unit: b.unit, pouch_unloaded: b.pouch_unloaded, warnings, def })
}

/// U2.2-S2 (FB-28): a member's passives, re-gathered at each build: its character's sources now (character, faction,
/// disciplines, named items, in that order, resolved in the character's own world), then the other ids it owns (quest
/// grants, earned ones) in saved order, each once. A new game saves the sources as they were (`new_game`), so the two
/// agree until the data changes; then a grant added to a source reaches the existing save. PK2-S1: then its unit
/// tracks' stage passives (`track_passives`), slotless.
pub fn member_passives(d: &Data, state: &UnitState) -> Vec<String> {
    let world = aegis_content::split_qid(&state.lore_ref).map(|(w, _)| w).unwrap_or_default();
    let mut out = d.character_passives(character_of(d, state), &world);
    for id in state.passives.iter().chain(&track_passives(d, state)) {
        if !out.contains(id) {
            out.push(id.clone());
        }
    }
    out
}

/// PK2-S1 (`docs/sessions/PK2_GRAFTS.md` 2): the passives the member's unit tracks grant: per track row (catalog order)
/// the member is on, every stage's passive up to its stage (`UnitTrackDef::passives_to`).
pub fn track_passives(d: &Data, state: &UnitState) -> Vec<String> {
    d.cat.unit_tracks.values().flat_map(|t| t.passives_to(t.clamp(state.track_stage(&t.id)))).collect()
}

/// PK2-S1: the member grant `forbid_hands` its passives give (`member_passives`; 0 = none).
pub fn forbid_hands(d: &Data, state: &UnitState) -> i32 {
    member_passives(d, state)
        .iter()
        .filter_map(|id| d.cat.passives.get(id))
        .flat_map(|p| rpgkit_rules::grants::read(&p.effects))
        .filter_map(|g| match g {
            rpgkit_rules::grants::Grant::ForbidHands { hands } => Some(hands),
            _ => None,
        })
        .max()
        .unwrap_or(0)
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
