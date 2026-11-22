//! The fight: the catalog encounter `ENCOUNTER` as the kit's battle session (`rpgkit_battle::session::Session`) plays
//! it, every side drafted by the kit's auto plan (`Session::auto_draft`, the balance sims' and autoplay's driver) and
//! resolved a round at a time. No rule lives here: the session decides every move, hit and the outcome.

use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_world::Data;
use serde_json::{json, Value};

/// The sample's one fight (`data/catalog/encounters.json`).
pub const ENCOUNTER: &str = "ks_skirmish";
/// The stage set the board is drawn from (`assets/tilesets/ks_stage/`).
pub const STAGE_SET: &str = "ks_stage";

/// A started fight.
pub struct Fight {
    pub s: Session,
    /// Each unit's body art key (its catalog row's `art`), by unit index.
    pub art: Vec<String>,
}

impl Fight {
    /// The encounter `ENCOUNTER` of `d`, started; None when the catalog has no such row.
    pub fn start(d: &Data) -> Option<Fight> {
        let row = d.cat.encounters.get(ENCOUNTER)?;
        let def = EncounterDef::from_obj(row);
        let s = Session::start(d.cat.clone(), def, &d.cat.units);
        let art = s.w.units.iter().map(|u| d.cat.units.get(&u.catalog_id).map(|r| r.art.clone()).unwrap_or_default()).collect();
        Some(Fight { s, art })
    }

    /// Whether the fight has ended (a side routed, or the last round resolved).
    pub fn over(&self) -> bool {
        self.s.combat_over || (!self.s.is_planning() && !self.s.can_advance_round())
    }

    /// One round: every side drafted by the auto plan, confirmed and resolved, then the next round opened. False when
    /// the fight is over or the plan is refused.
    pub fn round(&mut self) -> bool {
        if self.over() {
            return false;
        }
        self.s.auto_draft(rpgkit_battle::script::AUTO_PRESET);
        if !self.s.confirm_plan() {
            return false;
        }
        self.s.resolve_round();
        self.s.advance_round();
        true
    }

    /// Rounds until the fight ends (at most the encounter's round limit); the rounds played.
    pub fn to_end(&mut self) -> i32 {
        let mut n = 0;
        while n <= self.s.def.max_rounds && self.round() {
            n += 1;
        }
        n
    }

    /// The fight as the agent and the scenario read it.
    pub fn state(&self) -> Value {
        let units: Vec<Value> = self.s.w.units.iter().map(|u| json!({"name": u.name, "hp": u.hp, "max_hp": u.max_hp, "active": u.is_active()})).collect();
        json!({"over": self.over(), "outcome": self.s.outcome, "units": units})
    }
}
