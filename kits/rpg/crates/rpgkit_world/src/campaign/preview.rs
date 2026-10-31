//! Campaign previews (CAMPAIGN1 W3, CMQ8; kit hook, engine-only, no Godot original): the run-ending `end` a step would
//! fire, found by running the step on a copy of the state (the real state is never touched). The game asks before a
//! marker or dialogue fight, a trip or a rest that would end the run (No as the default); a road fight cannot be
//! refused, so it only gets the notice.

use super::{Campaign, Cause};
use crate::data::Data;
use crate::state::GameState;
use aegis_core::json;
use serde_json::Value;

/// The outcomes a battle preview tries, in this order.
pub const OUTCOMES: [&str; 3] = ["won", "lost", "fled"];

/// `{case, kind, card, text}` of the run over on `g` (None while the run goes on).
fn ending(g: &GameState, case: &str) -> Option<Value> {
    let o = g.campaign.as_ref()?.over.as_ref()?;
    Some(Value::Object(json::obj([("case", case.into()), ("kind", o.kind.clone().into()), ("card", o.card.clone().into()), ("text", o.text.clone().into())])))
}

/// Whether a preview has anything to look at (a campaign world whose run goes on).
fn live(gs: &GameState) -> bool {
    gs.campaign.as_ref().is_some_and(|c| !Campaign::is_over(c))
}

/// W3: the run overs battle `encounter` (a ref or a qid) would cause, one per outcome that ends the run (`case` the
/// outcome, in OUTCOMES order): each outcome is settled on a copy (`battle::settle`: meter inputs, rules, checkpoints,
/// triggers) and a loss then wakes or retreats (`explore::wake`: the retreat's days). [] when no outcome ends the run,
/// outside a campaign world and once the run is over.
pub fn preview_battle(d: &Data, gs: &GameState, encounter: &str) -> Vec<Value> {
    if !live(gs) {
        return Vec::new();
    }
    let qid = json::s_or(d.resolve(&gs.world_id, "encounters", encounter), "_qid", encounter);
    let mut out = Vec::new();
    for outcome in OUTCOMES {
        let mut g = gs.clone();
        crate::battle::settle(d, &mut g, &qid, outcome, None);
        if outcome == "lost" {
            crate::explore::wake(d, &mut g, "");
        }
        out.extend(ending(&g, outcome));
    }
    out
}

/// W3, CAMPAIGN1-TM: the run over the trip to `to` (a ref or a qid) would cause (`case` "days"): the trip's legs (the
/// party's travel mode's path and days, `travel::path` and `leg_days`) run on a copy, each as the trip runs it: its
/// days pass, then the days and the leg tick (meter inputs on legs, the mode's among them). No road roll and no
/// arrival: like `preview_days`, the listed case is the clock's and the meters'. [] when the run goes on, when there is
/// no known way, outside a campaign world and once the run is over.
pub fn preview_trip(d: &Data, gs: &GameState, to: &str) -> Vec<Value> {
    if !live(gs) {
        return Vec::new();
    }
    let start = crate::travel::current_node(d, gs);
    let target = json::s(crate::explore::resolve_location(d, gs, to), "_qid");
    let routes = crate::travel::path(d, gs, &start, &target);
    let mut g = gs.clone();
    let extra = crate::travel::mode_days(d, gs);
    let mode = super::purse::travel_mode(d, gs).map(|m| m.id).unwrap_or_default();
    for rq in routes {
        let route = crate::travel::route_in(d, &gs.world_id, &rq);
        let n = crate::travel::leg_days(&route, extra);
        g.time += n;
        super::tick(d, &mut g, &Cause::Days { n, why: "travel".into() });
        super::tick(d, &mut g, &Cause::Leg { route_tags: json::sget(&route, "tags"), mode: mode.clone() });
        if let Some(end) = ending(&g, "days") {
            return vec![end];
        }
    }
    Vec::new()
}

/// W3: the run over `n` days passing would cause (a trip's, a rest's or a retreat's days; `case` "days"): the clock
/// moves `n` days on a copy and the campaign ticks days(n). [] when the run goes on, outside a campaign world and once
/// the run is over.
pub fn preview_days(d: &Data, gs: &GameState, n: i64) -> Vec<Value> {
    if !live(gs) {
        return Vec::new();
    }
    let mut g = gs.clone();
    let n = n.max(0);
    g.time += n;
    super::tick(d, &mut g, &Cause::Days { n, why: "travel".into() });
    ending(&g, "days").into_iter().collect()
}
