//! Facilities (PR6-8, `docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` section 4; engine-only, no Godot original; story-free):
//! the NPC service `facility`. A character keeps the facility rows naming it (`keeper_character`); each facility is at
//! level `LEVEL_MIN` until the party buys the next level (`upgrade`: the level's `requires_flags` hold, its purse cost
//! and materials are paid, its `set_flag` effects set), saved in `GameState.facilities`. The passive hooks read the
//! effects of every level reached (`effects`): `shop_pct` lowers the keeper's shop prices (`shop::offers`),
//! `recipe_pct` the keeper's recipe purse costs (`recipe`), `rest_upkeep` a campaign rest's upkeep
//! (`campaign::purse`). With no facility rows (or none raised) every hook is inert.

use crate::data::Data;
use crate::dialogue;
use crate::quest;
use crate::state::GameState;
use crate::terms;
use aegis_core::json::{self, int, Dict};
use rpgkit_data::facility::{self as f, FacilityLevel, FACILITIES_KIND, LEVEL_MAX, LEVEL_MIN};
use serde_json::Value;

/// The refusals of an upgrade.
pub const TOP_TEXT: &str = "Already at its highest level.";
pub const NOT_YET_TEXT: &str = "Not available yet.";
pub const UNKNOWN_TEXT: &str = "Unknown facility.";

/// Every facility row of the played world (lineage merged), in row order.
pub fn facilities<'a>(d: &'a Data, gs: &GameState) -> Vec<&'a Dict> {
    d.merged_entries(&gs.world_id, FACILITIES_KIND)
}

/// The facility row `r` refers to ({} when none).
pub fn facility_of<'a>(d: &'a Data, gs: &GameState, r: &str) -> &'a Dict {
    if r.is_empty() {
        json::empty()
    } else {
        d.resolve(&gs.world_id, FACILITIES_KIND, r)
    }
}

/// The character qid that keeps facility row `fac`.
pub fn keeper_qid(d: &Data, gs: &GameState, fac: &Dict) -> String {
    dialogue::character_qid(d, gs, &json::s(fac, "keeper_character"))
}

/// The facilities character `npc_qid` keeps, in row order (none while that character is gone).
pub fn kept_by<'a>(d: &'a Data, gs: &GameState, npc_qid: &str) -> Vec<&'a Dict> {
    if npc_qid.is_empty() || dialogue::is_gone(d, gs, npc_qid) {
        return Vec::new();
    }
    facilities(d, gs).into_iter().filter(|fac| keeper_qid(d, gs, fac) == npc_qid).collect()
}

/// Facility `fqid`'s level now (`LEVEL_MIN` until raised).
pub fn level(gs: &GameState, fqid: &str) -> i64 {
    gs.facilities.get(fqid).copied().unwrap_or(LEVEL_MIN).clamp(LEVEL_MIN, LEVEL_MAX)
}

/// The highest level facility row `fac` lists (`LEVEL_MIN` for a row without levels), at most `LEVEL_MAX`.
pub fn top_level(fac: &Dict) -> i64 {
    f::levels(fac).iter().map(|l| l.level).filter(|l| (LEVEL_MIN..=LEVEL_MAX).contains(l)).max().unwrap_or(LEVEL_MIN)
}

/// Facility row `fac`'s level row `level` (None when it lists none).
pub fn level_row(fac: &Dict, level: i64) -> Option<FacilityLevel> {
    f::levels(fac).into_iter().find(|l| l.level == level)
}

/// The level row the next upgrade of facility row `fac` buys (None at its top level).
pub fn next(gs: &GameState, fac: &Dict) -> Option<FacilityLevel> {
    let now = level(gs, &json::s(fac, "_qid"));
    if now >= top_level(fac) {
        return None;
    }
    level_row(fac, now + 1)
}

/// A level's cost as lines ("40 <currency>", "<material> x2"), purse first.
pub fn cost_lines(d: &Data, gs: &GameState, lv: &FacilityLevel) -> Vec<String> {
    let mut out = Vec::new();
    if lv.scrap_cost > 0 {
        out.push(format!("{} {}", lv.scrap_cost, terms::word(d, &gs.world_id, terms::CURRENCY)));
    }
    out.extend(lv.materials.iter().map(|(id, n)| format!("{} x{n}", d.item_name(id))));
    out
}

/// A level effect as a line for the pane ("" for `set_flag` and unknown effects).
pub fn effect_text(d: &Data, gs: &GameState, fx: &Dict) -> String {
    let pct = json::i(fx, "pct", 100);
    match json::s(fx, "effect").as_str() {
        f::FX_SHOP_PCT => format!("Shop prices -{}%", 100 - pct),
        f::FX_RECIPE_PCT => {
            let tags = json::sget(fx, "recipe_tags");
            let on = if tags.is_empty() { String::new() } else { format!(" ({})", tags.join(", ")) };
            format!("Work costs -{}%{on}", 100 - pct)
        }
        f::FX_REST_UPKEEP => format!("Rest {} {:+}", terms::word(d, &gs.world_id, terms::UPKEEP), json::i(fx, "delta", 0)),
        _ => String::new(),
    }
}

/// Why facility `fqid` can't be raised now (None: it can). Changes nothing.
pub fn refusal(d: &Data, gs: &GameState, fqid: &str) -> Option<String> {
    let fac = facility_of(d, gs, fqid);
    if fac.is_empty() {
        return Some(UNKNOWN_TEXT.into());
    }
    let over = crate::campaign::save_refusal(gs);
    if !over.is_empty() {
        return Some(over.into());
    }
    let Some(lv) = next(gs, fac) else { return Some(TOP_TEXT.into()) };
    if !quest::all_hold(gs, &lv.requires_flags) {
        return Some(NOT_YET_TEXT.into());
    }
    if lv.scrap_cost > gs.party.gold {
        return Some(terms::not_enough_currency(d, &gs.world_id));
    }
    for (item, n) in &lv.materials {
        if gs.party.stash.get(item).copied().unwrap_or(0) < *n {
            return Some(format!("Not enough {}.", d.item_name(item)));
        }
    }
    None
}

/// Raises facility `fqid` one level: {ok, errors, level, notes}. Refused (nothing changes) as `refusal` says; else the
/// cost is paid, the level saved and its `set_flag` effects set.
pub fn upgrade(d: &Data, gs: &mut GameState, fqid: &str) -> Dict {
    if let Some(why) = refusal(d, gs, fqid) {
        return json::fail(&why);
    }
    let fac = facility_of(d, gs, fqid).clone();
    let Some(lv) = next(gs, &fac) else { return json::fail(TOP_TEXT) };
    let qid = json::s(&fac, "_qid");
    gs.party.gold -= lv.scrap_cost;
    for (item, n) in &lv.materials {
        gs.party.remove_from_stash(item, *n);
    }
    gs.facilities.insert(qid, lv.level);
    let mut notes = vec![format!("{} is now level {}: {}.", json::s_or(&fac, "name", fqid), lv.level, lv.name)];
    for fx in lv.effects.iter().filter(|fx| json::s(fx, "effect") == f::FX_SET_FLAG) {
        let why = gs.set_flag(&json::s(fx, "key"), fx.get("value").cloned().unwrap_or(Value::Bool(true)));
        if !why.is_empty() {
            notes.push(why);
        }
    }
    let mut out = json::ok_result(true, &[]);
    out.insert("level".into(), int(lv.level));
    out.insert("notes".into(), json::strs(&notes));
    out
}

/// The effects in force: (the keeper's qid, the effect) of every level reached of every facility, in row then level
/// order.
pub fn effects(d: &Data, gs: &GameState) -> Vec<(String, Dict)> {
    let mut out = Vec::new();
    for fac in facilities(d, gs) {
        let now = level(gs, &json::s(fac, "_qid"));
        let keeper = keeper_qid(d, gs, fac);
        for lv in f::levels(fac).into_iter().filter(|l| l.level <= now) {
            out.extend(lv.effects.into_iter().map(|fx| (keeper.clone(), fx)));
        }
    }
    out
}

/// The product of the `pct` of `list`'s effects, in percent (100 = none).
fn pct_of<'a>(list: impl Iterator<Item = &'a Dict>) -> i64 {
    list.fold(100, |acc, fx| acc * json::i(fx, "pct", 100).clamp(f::PCT_RANGE.0, f::PCT_RANGE.1) / 100)
}

/// `shop_pct`: the factor in percent on the stock and loot prices of the shops character `keeper_qid` runs (100 = none).
pub fn shop_pct(d: &Data, gs: &GameState, keeper_qid: &str) -> i64 {
    if facilities(d, gs).is_empty() {
        return 100;
    }
    let all = effects(d, gs);
    pct_of(all.iter().filter(|(k, fx)| k == keeper_qid && json::s(fx, "effect") == f::FX_SHOP_PCT).map(|(_, fx)| fx))
}

/// `recipe_pct`: the factor in percent on the purse cost of a recipe character `keeper_qid` offers carrying `tags`
/// (100 = none; an effect with `recipe_tags` counts only when the recipe carries one of them).
pub fn recipe_pct(d: &Data, gs: &GameState, keeper_qid: &str, tags: &[String]) -> i64 {
    if facilities(d, gs).is_empty() {
        return 100;
    }
    let all = effects(d, gs);
    pct_of(
        all.iter()
            .filter(|(k, fx)| {
                let want = json::sget(fx, "recipe_tags");
                k == keeper_qid && json::s(fx, "effect") == f::FX_RECIPE_PCT && (want.is_empty() || want.iter().any(|t| tags.contains(t)))
            })
            .map(|(_, fx)| fx),
    )
}

/// `rest_upkeep`: the sum of the deltas in force on a campaign rest's upkeep cost.
pub fn rest_upkeep_delta(d: &Data, gs: &GameState) -> i64 {
    if facilities(d, gs).is_empty() {
        return 0;
    }
    effects(d, gs).iter().filter(|(_, fx)| json::s(fx, "effect") == f::FX_REST_UPKEEP).map(|(_, fx)| json::i(fx, "delta", 0)).sum()
}

/// `price` after the factor `pct` (percent): ceil(price x pct / 100), at least 1; the price itself at 100 or for 0.
pub fn scaled(price: i64, pct: i64) -> i64 {
    if pct >= 100 || price <= 0 {
        return price;
    }
    ((price * pct + 99) / 100).max(1)
}

/// The pane's reading of character `npc_qid`'s facilities: [{id, name, level, top, levels: [{level, name, effects:
/// [line], reached}], next: {level, name, cost: [line], effects: [line]} | null, ready, reason}] in row order.
pub fn view(d: &Data, gs: &GameState, npc_qid: &str) -> Vec<Value> {
    kept_by(d, gs, npc_qid)
        .into_iter()
        .map(|fac| {
            let qid = json::s(fac, "_qid");
            let now = level(gs, &qid);
            let lines = |lv: &FacilityLevel| -> Vec<String> { lv.effects.iter().map(|fx| effect_text(d, gs, fx)).filter(|t| !t.is_empty()).collect() };
            let levels: Vec<Value> = f::levels(fac)
                .iter()
                .map(|lv| {
                    Value::Object(json::obj([
                        ("level", int(lv.level)),
                        ("name", lv.name.clone().into()),
                        ("effects", json::strs(&lines(lv))),
                        ("reached", Value::Bool(lv.level <= now)),
                    ]))
                })
                .collect();
            let next_row = next(gs, fac).map_or(Value::Null, |lv| {
                Value::Object(json::obj([
                    ("level", int(lv.level)),
                    ("name", lv.name.clone().into()),
                    ("cost", json::strs(&cost_lines(d, gs, &lv))),
                    ("effects", json::strs(&lines(&lv))),
                ]))
            });
            let why = refusal(d, gs, &qid).unwrap_or_default();
            Value::Object(json::obj([
                ("id", qid.into()),
                ("name", json::s_or(fac, "name", &json::s(fac, "id")).into()),
                ("level", int(now)),
                ("top", int(top_level(fac))),
                ("levels", Value::Array(levels)),
                ("next", next_row),
                ("ready", Value::Bool(why.is_empty())),
                ("reason", why.into()),
            ]))
        })
        .collect()
}
