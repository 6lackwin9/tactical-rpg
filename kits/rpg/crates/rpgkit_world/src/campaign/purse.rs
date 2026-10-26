//! Campaign purse, rests and retreats (CAMPAIGN1 P1-P3, R1-R4, M7; kit hook, engine-only, no Godot original): the
//! manifest's `purse`, `rest` and `defeat` blocks of a campaign world (today's rules elsewhere, X1), upkeep per day,
//! the road-fight chance of a leg (zero upkeep and the meters' band modifiers), the start-AP change at zero upkeep, a
//! rest that costs days and upkeep, and the retreat after a lost battle. The travel, Camp and wake code call these.

use super::{is_on, manifest_block, merge_events, Cause};
use crate::data::Data;
use crate::explore::{self, SPAWN};
use crate::progression;
use crate::state::GameState;
use aegis_core::json::{self, int, Dict};
use rpgkit_data::{DefeatRule, Purse, RestRule};
use serde_json::Value;

/// A road-fight chance never goes above this (travel.rs PCT_MAX).
pub const PCT_MAX: i64 = 100;
/// R1 `where`: the refusals of a rest the rule does not allow here (`rest_places`, `none`).
pub const REFUSE_NOT_REST_PLACE: &str = "Rest only at a safe place.";
pub const REFUSE_NO_REST: &str = "There is no resting here.";
/// R1 `needs_upkeep`: the refusal of a rest that costs more upkeep than the party holds.
pub const REFUSE_UPKEEP: &str = "Not enough upkeep to rest.";

/// The block `key` of world `w` when it runs a campaign, else {} (today's rules, X1).
fn block(d: &Data, w: &str, key: &str) -> Dict {
    if is_on(d, w) {
        manifest_block(d, w, key)
    } else {
        Dict::new()
    }
}

/// P1-P3: the purse of world `w` (today's names, 1 upkeep a day, x1.5 road fights and no AP change at zero upkeep
/// outside a campaign world).
pub fn purse(d: &Data, w: &str) -> Purse {
    Purse::from_obj(&block(d, w, "purse"))
}

/// R1: the rest rule of world `w` (today's Camp and free rest-place heal outside a campaign world).
pub fn rest_rule(d: &Data, w: &str) -> RestRule {
    RestRule::from_obj(&block(d, w, "rest"))
}

/// R4: the defeat rule of world `w` (today's wake outside a campaign world).
pub fn defeat_rule(d: &Data, w: &str) -> DefeatRule {
    DefeatRule::from_obj(&block(d, w, "defeat"))
}

/// P2: the upkeep a leg of `time` spends (`time x per_day`).
pub fn leg_upkeep(d: &Data, w: &str, time: i64) -> i64 {
    time.max(0) * purse(d, w).per_day as i64
}

// ---- road fights (P3, M7) ----

/// `chance` times `pct` percent, capped at PCT_MAX (travel.rs starved_chance's arithmetic).
pub fn scaled(chance: i64, pct: i64) -> i64 {
    PCT_MAX.min(chance.max(0) * pct / 100)
}

/// M7: the band modifiers of the campaign's meters that hold now on a route carrying `route_tags`, in meter then row
/// order: (road_pct, road_encounters as [encounter qid, weight] pairs). None outside a campaign world.
fn modifiers(d: &Data, gs: &GameState, route_tags: &[String]) -> Vec<(i64, Vec<Value>)> {
    let Some(c) = gs.campaign.as_ref() else { return Vec::new() };
    let mut out = Vec::new();
    for (id, def) in c.defs.iter() {
        let band = def.band(c.value(id, gs.time)).map(|b| b.id.clone()).unwrap_or_default();
        let home = json::s_or(&def.row, "_world", &gs.world_id);
        for m in json::aget(&def.row, "modifiers").iter().filter_map(Value::as_object) {
            if !json::sget(m, "bands").contains(&band) || !super::meter::tags_match(&json::sget(m, "route_tags"), route_tags) {
                continue;
            }
            let encounters = json::aget(m, "road_encounters")
                .iter()
                .map(|p| {
                    let pair = json::arr(p);
                    let r = pair.first().map(json::gstr).unwrap_or_default();
                    let qid = json::s_or(d.resolve(&home, "encounters", &r), "_qid", &r);
                    Value::Array(vec![qid.into(), pair.get(1).cloned().unwrap_or(int(0))])
                })
                .collect();
            out.push((json::i(m, "road_pct", 100), encounters));
        }
    }
    out
}

/// P3, M7: a leg's road-fight chance from its base chance (`base`, travel.rs leg's own or repeat chance): at zero
/// upkeep times `at_zero.road_pct` (today's x1.5 by default), then times each band modifier that holds, capped at 100
/// after each step. Outside a campaign world: travel.rs starved_chance, unchanged.
pub fn road_chance(d: &Data, gs: &GameState, route: &Dict, base: i64, starved: bool) -> i64 {
    let mut chance = if starved { scaled(base, purse(d, &gs.world_id).road_pct as i64) } else { base };
    for (pct, _) in modifiers(d, gs, &json::sget(route, "tags")) {
        chance = scaled(chance, pct);
    }
    chance
}

/// M7: the [encounter qid, weight] pairs the band modifiers that hold now add to a leg's road pool on `route` (none
/// outside a campaign world).
pub fn road_encounters(d: &Data, gs: &GameState, route: &Dict) -> Vec<Value> {
    modifiers(d, gs, &json::sget(route, "tags")).into_iter().flat_map(|(_, e)| e).collect()
}

// ---- battle start (P3) ----

/// P3: what a campaign changes at a battle's start: every party unit's AP (`start_ap`, 0 or less; the battle bridge
/// applies it, not below 0) and the log line (W6, "" when nothing changes).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BattleMods {
    pub start_ap: i64,
    pub note: String,
}

/// P3: the battle start's change: `at_zero.start_ap` when the upkeep is 0 as the battle starts; nothing otherwise and
/// outside a campaign world.
pub fn battle_mods(d: &Data, gs: &GameState) -> BattleMods {
    if gs.campaign.is_none() || gs.party.supplies > 0 {
        return BattleMods::default();
    }
    let p = purse(d, &gs.world_id);
    let ap = (p.start_ap as i64).min(0);
    if ap == 0 {
        return BattleMods::default();
    }
    BattleMods { start_ap: ap, note: format!("No {}: {ap} AP", p.upkeep.name) }
}

/// P3: a party unit's AP at the battle's start after `mods` (never below 0).
pub fn start_ap(ap: i64, mods: &BattleMods) -> i64 {
    (ap + mods.start_ap).max(0)
}

// ---- rest (R1-R3) ----

/// R1, R2: why a rest is refused now ("" = may rest): `where` (Camp on a world-map node, only at a rest place, or
/// never), nobody hurt, and too little upkeep when `needs_upkeep`. Outside a campaign world: today's Camp
/// (`progression::can_camp`).
pub fn rest_refusal(d: &Data, gs: &GameState) -> &'static str {
    if gs.campaign.is_none() {
        return progression::can_camp(d, gs);
    }
    let over = super::save_refusal(gs);
    if !over.is_empty() {
        return over;
    }
    let rule = rest_rule(d, &gs.world_id);
    let place = match rule.place.as_str() {
        "rest_places" if !progression::is_rest_place(d, gs, &gs.location_id) => REFUSE_NOT_REST_PLACE,
        "none" => REFUSE_NO_REST,
        "nodes" if crate::travel::current_node(d, gs).is_empty() => progression::REFUSE_NOT_AT_NODE,
        _ => "",
    };
    if !place.is_empty() {
        return place;
    }
    // ROSTER1's wounded members count as hurt once that lands (R2)
    if !progression::hurt(&gs.party) {
        return progression::REFUSE_RESTED;
    }
    if rule.needs_upkeep && gs.party.supplies < rule.cost as i64 {
        return REFUSE_UPKEEP;
    }
    ""
}

/// R2's body without the tick: every member back to full (ROSTER1's rest hook clears the wounded once it lands), the
/// rule's `cost` spent (never below 0) and its `days` added to the clock. Returns (spent, restored, days).
fn rest_body(d: &Data, gs: &mut GameState) -> (i64, i64, i64) {
    let rule = rest_rule(d, &gs.world_id);
    let spent = gs.party.spend_supplies(rule.cost as i64);
    let restored = progression::rest(d, &mut gs.party);
    let days = (rule.days as i64).max(0);
    gs.time += days;
    (spent, restored, days)
}

/// R2, R3: Camp on the world map (and the rest place's Camp) in a campaign world: refused as `rest_refusal` says, else
/// the rest, then the `days` tick (cause days(n, rest)) and the autosave (none once the run is over).
/// {ok, errors, spent, restored, days, events, campaign?}.
pub fn rest(d: &Data, gs: &mut GameState) -> Dict {
    let refusal = rest_refusal(d, gs);
    if !refusal.is_empty() {
        let mut out = json::fail(refusal);
        out.insert("spent".into(), int(0));
        out.insert("restored".into(), int(0));
        out.insert("events".into(), Value::Array(Vec::new()));
        return out;
    }
    let (spent, restored, days) = rest_body(d, gs);
    let ticked = if days > 0 { super::tick(d, gs, &Cause::Days { n: days, why: "rest".into() }) } else { Vec::new() };
    let mut events = vec![Value::Object(json::obj([
        ("kind", "camped".into()),
        ("location", gs.location_id.clone().into()),
        ("spent", int(spent)),
        ("restored", int(restored)),
        ("days", int(days)),
    ]))];
    if super::save_refusal(gs).is_empty() {
        events.push(Value::Object(json::obj([("kind", "autosave".into())])));
    }
    let mut out = json::ok_result(true, &[]);
    out.insert("spent".into(), int(spent));
    out.insert("restored".into(), int(restored));
    out.insert("days".into(), int(days));
    out.insert("events".into(), Value::Array(events));
    merge_events(&mut out, ticked);
    out
}

/// 11.3 `rest` (a dialogue choice or a trigger, R3): the rest without its own tick; the days it passed are owed to
/// the campaign and run as cause days(n, rest) at the start of the tick after the effect list. Returns the log note.
/// Inert outside a campaign world; the dialogue's precheck refuses it as `rest_refusal` says.
pub fn rest_effect(d: &Data, gs: &mut GameState) -> String {
    if gs.campaign.as_ref().is_none_or(super::Campaign::is_over) {
        return String::new();
    }
    let (spent, _, days) = rest_body(d, gs);
    if let Some(c) = gs.campaign.as_mut() {
        c.owed_days += days;
    }
    let p = purse(d, &gs.world_id);
    let mut note = "The party rests: everyone is back to full".to_string();
    if days > 0 {
        note.push_str(&format!(", {days} day{}", if days == 1 { "" } else { "s" }));
    }
    if spent > 0 {
        note.push_str(&format!(", {spent} {}", p.upkeep.short));
    }
    note.push('.');
    note
}

// ---- defeat (R4) ----

/// R4: whether a lost battle retreats in the party's world (a campaign world whose `defeat` is `retreat`).
pub fn retreats(d: &Data, gs: &GameState) -> bool {
    gs.campaign.is_some() && defeat_rule(d, &gs.world_id).mode == "retreat"
}

/// R4: the `retreat_failed` event when the retreat's `to` cannot be entered (unknown, or its map or spawn is missing):
/// `explore::wake` then wakes the classic way and reports this first (the log names why). None when it can.
pub fn retreat_problem(d: &Data, gs: &GameState) -> Option<Value> {
    let to = defeat_rule(d, &gs.world_id).to;
    let problem = explore::entry_problem(d, gs, &to, SPAWN);
    if problem.is_empty() {
        return None;
    }
    let text = format!("The retreat to '{to}' failed: {problem} The party wakes instead.");
    Some(Value::Object(json::obj([("kind", "retreat_failed".into()), ("to", to.into()), ("text", text.into())])))
}

/// R4: the retreat after a lost battle (all three `after_fight` paths, through `explore::wake`, which checks
/// `retreat_problem` first): the cell moves to the rule's `to` (no heal unless `to` is a rest place with `on_enter`;
/// ROSTER1 applies the loss once it lands; the encounter stays undone), `days x per_day` upkeep is spent
/// (CMQ4), `days` pass (cause days(n, retreat)) and the autosave follows (none once the run is over). Events as
/// `wake`'s, the `defeat` event with mode `retreat`; `campaign` holds the tick's events when there are some.
pub fn retreat(d: &Data, gs: &mut GameState) -> Dict {
    let rule = defeat_rule(d, &gs.world_id);
    let to = json::s(explore::resolve_location(d, gs, &rule.to), "_qid");
    let problem = explore::entry_problem(d, gs, &to, SPAWN);
    if !problem.is_empty() {
        return json::fail(&problem);
    }
    let mut moved = explore::enter(d, gs, &to, SPAWN);
    if !json::b(&moved, "ok", false) {
        return moved;
    }
    let days = (rule.days as i64).max(0);
    let spent = gs.party.spend_supplies(days * purse(d, &gs.world_id).per_day as i64);
    gs.time += days;
    let ticked = if days > 0 { super::tick(d, gs, &Cause::Days { n: days, why: "retreat".into() }) } else { Vec::new() };
    let mut events: Vec<Value> = json::aget(&moved, "events").iter().filter(|e| json::s(json::dict(e), "kind") != "autosave").cloned().collect();
    events.push(Value::Object(json::obj([
        ("kind", "defeat".into()),
        ("mode", "retreat".into()),
        ("spent", int(spent)),
        ("days", int(days)),
        ("location", gs.location_id.clone().into()),
    ])));
    if super::save_refusal(gs).is_empty() {
        events.push(Value::Object(json::obj([("kind", "autosave".into())])));
    }
    moved.insert("events".into(), Value::Array(events));
    merge_events(&mut moved, ticked);
    moved
}
