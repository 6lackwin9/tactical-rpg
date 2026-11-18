//! Campaign triggers (CAMPAIGN1 T1-T3, F2; kit hook, engine-only, no Godot original): the world kind `triggers` read
//! with its defaults (PR6-6, B5: the cause `recipe` with its `recipe` / `recipe_tags` filters, a battle's `hero`
//! filter), the tick's trigger step (after the checkpoints, X4) and the pieces the game uses to play a queued
//! `{battle}` item (`fight`, `after_fight`; a `{dialogue}` item opens `DialogueRunner::open_scene`, a `{vignette}` item
//! moves to the story queue through `story::pull_pending`). Effects run through the shared effect body
//! (`dialogue::apply_effects`), so a trigger, a meter rule, a checkpoint, a sequence step and a dialogue choice apply
//! an effect the same way.

use super::{apply_effects, Cause};
use crate::data::Data;
use crate::quest;
use crate::state::GameState;
use aegis_core::json::{self, Dict};
use serde_json::Value;

/// T2: the runner source of a queued `{battle}` item's fight (it settles like a dialogue fight).
pub const FIGHT_SOURCE: &str = "trigger";

/// A trigger row (11.2) read with the schema's defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct TriggerDef {
    pub id: String,
    pub name: String,
    /// One of any, day, battle, enter, act, recipe (PR6-6).
    pub on: String,
    /// On battle: the encounter's qid ("" = any).
    pub encounter: String,
    /// On battle: won, lost, fled or any.
    pub outcome: String,
    /// On enter: the location's qid ("" = any); ACT1-HOOKS2 (FB-246 (a)): on any, the location the party must be at
    /// when the tick runs ("" = anywhere).
    pub location: String,
    /// On battle (PR6-6, B5): a fielded hero's battle end must meet it (empty = no filter).
    pub hero: super::HeroFilter,
    /// On recipe (PR6-6, B5): just this recipe id ("" = any).
    pub recipe: String,
    /// On recipe (PR6-6, B5): the recipe carries one of these tags (empty = any recipe).
    pub recipe_tags: Vec<String>,
    pub when: Value,
    pub once: bool,
    pub effects: Vec<Dict>,
    pub then: Vec<Dict>,
}

/// The qid of ref `r` of kind `kind` from world `home` ("" stays "", an unknown ref stays as written).
fn qid_of(d: &Data, home: &str, kind: &str, r: &str) -> String {
    if r.is_empty() {
        return String::new();
    }
    json::s_or(d.resolve(home, kind, r), "_qid", r)
}

impl TriggerDef {
    pub fn from_row(d: &Data, e: &Dict) -> Self {
        let home = json::s(e, "_world");
        let dicts = |key: &str| -> Vec<Dict> { json::aget(e, key).iter().filter_map(|x| x.as_object().cloned()).collect() };
        Self {
            id: json::s(e, "id"),
            name: json::s(e, "name"),
            on: json::s_or(e, "on", "any"),
            encounter: qid_of(d, &home, "encounters", &json::s(e, "encounter")),
            outcome: json::s_or(e, "outcome", "any"),
            location: qid_of(d, &home, "locations", &json::s(e, "location")),
            hero: super::HeroFilter::from_value(json::get(e, "hero")),
            recipe: json::s(e, "recipe"),
            recipe_tags: json::sget(e, "recipe_tags"),
            when: Value::Array(json::aget(e, "when").to_vec()),
            once: json::b(e, "once", true),
            effects: dicts("effects"),
            then: dicts("then"),
        }
    }

    /// T1: whether the trigger is checked after `cause` (`any` after every cause but a recipe run; `battle`, `enter`
    /// and (PR6-6) `recipe` through their filters: a battle's `hero` filter holds when one fielded hero meets it; a
    /// recipe run checks only the `recipe` triggers).
    pub fn matches(&self, cause: &Cause) -> bool {
        match (self.on.as_str(), cause) {
            // PR6-6 (review): a recipe run checks the recipe triggers only (not `any`)
            ("recipe", Cause::Recipe { recipe, tags, .. }) => (self.recipe.is_empty() || &self.recipe == recipe) && super::meter::tags_match(&self.recipe_tags, tags),
            (_, Cause::Recipe { .. }) => false,
            ("any", _) | ("day", Cause::Days { .. }) | ("act", Cause::Act(_)) => true,
            ("battle", Cause::Battle { encounter, outcome, heroes, .. }) => {
                (self.encounter.is_empty() || &self.encounter == encounter)
                    && (self.outcome == "any" || self.outcome.is_empty() || &self.outcome == outcome)
                    && (self.hero.is_empty() || heroes.iter().any(|h| self.hero.meets(h)))
            }
            ("enter", Cause::Enter { location }) => self.location.is_empty() || &self.location == location,
            _ => false,
        }
    }

    /// ACT1-HOOKS2 (FB-246 (a)): an `any` trigger with a `location` is checked only while the party is at that location
    /// (`GameState::location_id`, after a fight, a turn-in, a day, an entry...); every other trigger is checked anywhere
    /// (`enter` filters by the entered location in `matches`).
    pub fn at_location(&self, gs: &GameState) -> bool {
        self.on != "any" || self.location.is_empty() || gs.location_id == self.location
    }

    /// T1: every `when` condition holds now (read keys included).
    pub fn holds(&self, gs: &GameState) -> bool {
        quest::all_hold(gs, &self.when)
    }

    /// The key a `once` trigger leaves in `fired` (11.5).
    pub fn fired_key(&self) -> String {
        format!("trigger:{}", self.id)
    }
}

/// The trigger rows of world `w` (merged along the lineage), in data order.
pub fn defs(d: &Data, w: &str) -> Vec<TriggerDef> {
    d.merged_entries(w, "triggers").into_iter().map(|e| TriggerDef::from_row(d, e)).filter(|t| !t.id.is_empty()).collect()
}

/// T1, T2, the tick's trigger step after `cause`: first a repeatable trigger whose `when` stopped holding leaves
/// `armed` (it may fire again); then, in data order, each trigger checked after this cause (an `any` trigger with a
/// `location` only while the party is there, `at_location`) whose `when` holds fires
/// unless it already did (`once`: its `trigger:<id>` in `fired`; repeatable: its id in `armed`). Firing runs the
/// effects at once (the shared effect body; a `trigger_fired` event carries their notes for the log) and, unless the
/// run ended, queues the `then` items in `pending`. Stops at the first `end` (F1). Returns whether any fired (the rules
/// run again).
pub fn step(d: &Data, gs: &mut GameState, cause: &Cause, events: &mut Vec<Value>) -> bool {
    let rows = defs(d, &gs.world_id);
    let holding: Vec<String> = rows.iter().filter(|t| !t.once && t.holds(gs)).map(|t| t.id.clone()).collect();
    let Some(c) = gs.campaign.as_mut() else { return false };
    // PR6-6 (review): a recipe run re-arms nothing (the next tick does, as before recipe runs ticked)
    if !matches!(cause, Cause::Recipe { .. }) {
        c.armed.retain(|id| holding.contains(id));
    }
    let mut fired = false;
    for t in rows.iter().filter(|t| t.matches(cause)) {
        if !t.at_location(gs) {
            continue;
        }
        let Some(c) = gs.campaign.as_ref() else { break };
        if c.is_over() {
            break;
        }
        let done = if t.once { c.fired.contains(&t.fired_key()) } else { c.armed.contains(&t.id) };
        // `when` is read now: an earlier trigger's effects count
        if done || !t.holds(gs) {
            continue;
        }
        let Some(c) = gs.campaign.as_mut() else { break };
        if t.once {
            c.fired.push(t.fired_key());
            c.fired.sort();
        } else {
            c.armed.push(t.id.clone());
        }
        fired = true;
        let mut fx_events = Vec::new();
        let notes = apply_effects(d, gs, &t.effects, &mut fx_events);
        events.push(Value::Object(json::obj([
            ("event", "trigger_fired".into()),
            ("trigger", t.id.clone().into()),
            ("name", t.name.clone().into()),
            ("notes", json::strs(&notes)),
        ])));
        events.extend(fx_events);
        let Some(c) = gs.campaign.as_mut() else { break };
        if c.is_over() {
            break;
        }
        c.pending.extend(t.then.iter().cloned());
    }
    fired
}

/// T2: the world fight of the queue's head when it is a `{battle}` item: `{source: "trigger", encounter}` (the
/// encounter as a qid); None for any other head or an empty queue. The game pops the item (`campaign::pop_pending`)
/// when it starts the fight.
pub fn fight(d: &Data, gs: &GameState) -> Option<Dict> {
    let r = json::s(super::pending_head(gs)?, "battle");
    if r.is_empty() {
        return None;
    }
    let qid = qid_of(d, &gs.world_id, "encounters", &r);
    Some(json::obj([("source", FIGHT_SOURCE.into()), ("encounter", qid.into())]))
}

/// T2: after a queued battle: settled like a dialogue fight (`battle::settle`, then the wake or the retreat on a loss).
pub fn after_fight(d: &Data, gs: &mut GameState, fight: &Dict, outcome: &str, tally: Option<&crate::battle::Tally>) -> Dict {
    crate::dialogue::after_fight(d, gs, fight, outcome, tally)
}
