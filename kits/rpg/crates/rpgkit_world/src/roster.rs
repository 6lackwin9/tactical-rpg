//! The roster (ROSTER1, kit hook; engine-only, no Godot original): a party member's life across battles. How a member
//! takes the field (`field_unit`), how a battle's end settles the party (`settle_party`), how a member leaves it
//! (`depart`) and what a rest clears (`on_rest`) live here, in the world crate, so every screen and runner applies one
//! rule. ROSTER1-L slice 2 moved the build and the settle from the battle screen with no rule change; slice 3 adds the
//! world's rule rows (3.2: the act's row turns a fate into `wounded`, `captured` or `dead`; no row = HEAL1 classic,
//! L3), the wounded build (W2), departures with their named keepsake (L1, L2, KS1, KS2) and the rest that clears
//! `wounded` (W3).

use crate::data::Data;
use crate::progression;
use crate::state::{Departed, GameState, PartyState, UnitState};
use aegis_core::json;
use rpgkit_battle::encounter::PartyFate;
use rpgkit_battle::session::Session;
use rpgkit_battle::world::AiProfile;
use rpgkit_data::{RosterManifest, RosterRule, UnitDef, DEPART_CAUSES, DEPART_LIVES, MEMBER_CONDITIONS, SETTLE_CLASSIC, WOUNDED_PCT_DEFAULT};
use rpgkit_rules::Unit;

/// W1: the one condition ROSTER1 defines (the kit's `MEMBER_CONDITIONS`, shared with PORTRAITFX1's checks).
pub const WOUNDED: &str = MEMBER_CONDITIONS[0];
/// 3.2: the settle value that departs a member captured, and the one that departs it dead.
pub const CAPTURED: &str = "captured";
pub const DEAD: &str = "dead";
/// L4: `hp_after` `full`: every member who comes home is back to full.
pub const HP_FULL: &str = "full";

/// 11.1: the roster block of world `w` (the nearest manifest in the lineage with a non-empty one; {} = today's rules).
pub fn manifest(d: &Data, w: &str) -> RosterManifest {
    RosterManifest::from_obj(&crate::campaign::manifest_block(d, w, "roster"))
}

/// L6: the act the rule rows read: `campaign.act`, 0 when the world has no campaign.
pub fn act(gs: &GameState) -> i32 {
    gs.campaign.as_ref().map_or(0, |c| c.act as i32)
}

/// L6: the rule row of the played world in the current act (None = classic).
pub fn rule(d: &Data, gs: &GameState) -> Option<RosterRule> {
    manifest(d, &gs.world_id).rule(act(gs)).cloned()
}

/// W2: a wounded member's battle max HP: `max - roundi(max x pct / 100)`, at least 1.
pub fn wounded_max(max_hp: i64, pct: i32) -> i64 {
    let cut = (max_hp as f64 * pct as f64 / 100.0).round() as i64;
    (max_hp - cut).max(1)
}

/// A member as it takes the field: the battle unit, the saved pouch ids that found no battle slot, its AI profile.
pub struct Fielded {
    pub unit: Unit,
    pub pouch_unloaded: Vec<String>,
    pub profile: AiProfile,
}

/// Member `m` as it fights in world `world`, the world whose encounter it fights: its class path and deck rule there
/// (`progression::build_played`), at its stored HP (HealRules.start_hp), with its base unit's AI profile. W2: a
/// `wounded` member's max HP is cut by the played world's row's `wounded_max_hp_pct` (the default without a row) and
/// its start HP is at most that max. None when the saved member does not build.
pub fn field_unit(d: &Data, gs: &GameState, m: &UnitState, world: &str) -> Option<Fielded> {
    let built = progression::build_played(d, m, world)?;
    let mut unit = built.unit;
    let max = unit.max_hp as i64;
    let mut hp = progression::start_hp(max, m.hp_lost);
    if m.has_condition(WOUNDED) {
        let pct = rule(d, gs).map_or(WOUNDED_PCT_DEFAULT, |r| r.wounded_max_hp_pct);
        let cut = wounded_max(max, pct);
        unit.max_hp = cut as i32;
        hp = hp.min(cut);
    }
    unit.hp = hp as i32;
    let base = d.cat.units.get(&m.base_unit).cloned().unwrap_or_else(|| UnitDef::from_obj(&Default::default()));
    Some(Fielded { unit, pouch_unloaded: built.pouch_unloaded, profile: AiProfile::for_unit(&d.cat, &base) })
}

/// How a battle left one fielded member (ROSTER1 3.2's input).
#[derive(Debug, Clone, PartialEq)]
pub struct HeroEnd {
    pub member: String,
    pub hp: i64,
    pub max_hp: i64,
    pub alive: bool,
    /// The battle pouch's unused item ids.
    pub pouch: Vec<String>,
    /// The saved pouch ids that found no battle slot (`Fielded::pouch_unloaded`).
    pub unloaded: Vec<String>,
    /// ENCOUNTER2 S3's fate (None for a unit `party_fates` does not list).
    pub fate: Option<PartyFate>,
}

/// The ends of session `s`'s fielded members: `members` (member id, unloaded pouch ids) in the order they were fielded,
/// paired with the session's allies in order (the heroes come first; guests follow and have no member).
pub fn hero_ends(s: &Session, members: &[(String, Vec<String>)]) -> Vec<HeroEnd> {
    let fates = s.party_fates();
    members
        .iter()
        .zip(&s.w.allies)
        .map(|((member, unloaded), &a)| {
            let u = &s.w.units[a];
            HeroEnd {
                member: member.clone(),
                hp: u.hp as i64,
                max_hp: u.max_hp as i64,
                alive: u.is_alive(),
                pouch: u.inventory.pouch.iter().map(|p| p.id.clone()).collect(),
                unloaded: unloaded.clone(),
                fate: fates.iter().find(|(c, _)| *c == a).map(|(_, f)| *f),
            }
        })
        .collect()
}

/// The battle a settle closes (3.2's battle-level inputs).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FightEnd {
    /// `won`, `lost` or `fled`.
    pub outcome: String,
    /// The encounter fought (the departed record's `encounter`).
    pub encounter: String,
    /// The world whose encounter was fought (`field_unit`'s `world`: a wounded member's unwounded max is built there).
    pub world: String,
    /// The objective was `escape` (a won escape does not hold the field).
    pub escape: bool,
}

impl FightEnd {
    /// The end of session `s` fought in world `world` (`Session::final_outcome`, the encounter id, its objective).
    pub fn of(s: &Session, world: &str) -> Self {
        Self { outcome: s.final_outcome(), encounter: s.def.id.clone(), world: world.to_string(), escape: s.objective.objective == "escape" }
    }

    /// A fight with `outcome` in world `world` (no encounter id, not an escape).
    pub fn new(outcome: &str, world: &str) -> Self {
        Self { outcome: outcome.to_string(), world: world.to_string(), ..Default::default() }
    }

    /// 3.2: "the field is held" = won and the objective is not `escape`.
    pub fn field_held(&self) -> bool {
        self.outcome == "won" && !self.escape
    }
}

/// What a settle did beyond HP (for the results lines, ROSTER1-U): the members newly wounded and those who departed
/// (member, life).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Settled {
    pub wounded: Vec<String>,
    pub departed: Vec<(String, String)>,
}

/// 3.2: the settle value of a hero's end under row `r` in fight `f`, after `then` `none` (a stayer or stander at 0 HP
/// settles as `downed`, else as `escaped`; ENCOUNTER2-W adds `capture` and `kill`); `classic` where the table has no
/// rule key (standing or escaped on a win or a flee). Returns (value, the cause when it departs).
fn settle_value(r: &RosterRule, f: &FightEnd, e: &HeroEnd) -> (String, &'static str) {
    let fate = e.fate.unwrap_or(if e.alive { PartyFate::Standing } else { PartyFate::Downed });
    let downed = match fate {
        PartyFate::Downed => true,
        PartyFate::Stayed | PartyFate::Stood => !e.alive || e.hp <= 0,
        PartyFate::Escaped | PartyFate::Standing => false,
    };
    match (downed, f.outcome.as_str()) {
        (true, "won") if f.field_held() => (r.downed_won.clone(), "downed"),
        (true, "won" | "fled") => (r.downed_left.clone(), "downed"),
        (true, _) => (r.downed_lost.clone(), "lost_battle"),
        (false, "won" | "fled") => (SETTLE_CLASSIC.to_string(), ""),
        (false, _) => (r.standing_lost.clone(), "lost_battle"),
    }
}

/// 3.2: settles the fielded members after the fight `f`. Every outcome: the unused pouch items go back to the saved
/// pouch, then the ids that found no slot (ITEMUSE1 rule 7, GameEncounter.write_back_pouches). Then (L5) the played
/// world's rule row in the current act (L6) turns each fate into `classic`, `wounded` (W1: held once), `captured` or
/// `dead` (`depart`, which pays the keepsake). HP of the members who come home: with no row (L3, classic) won and fled
/// write HP back against the battle max (HealRules.hp_after: a downed hero rallies on a win, keeps 1 HP on a flee) and
/// a lost battle leaves HP to the runner's defeat rule; with a row (L4) `full` brings everyone back to full and `carry`
/// is HEAL1's `hp_after` (a loss heals, as the wake does) written back against the unwounded max (W2). Call it before
/// the runner's `after_fight` (L5: the settle comes before CAMPAIGN1's battle tick and its retreat, which then moves a
/// cell that already carries the loss).
pub fn settle_party(d: &Data, gs: &mut GameState, f: &FightEnd, ends: &[HeroEnd]) -> Settled {
    let mut out = Settled::default();
    for e in ends {
        if let Some(m) = gs.party.member_mut(&e.member) {
            m.pouch = e.pouch.iter().chain(&e.unloaded).cloned().collect();
        }
    }
    let won_or_fled = f.outcome == "won" || f.outcome == "fled";
    let Some(row) = rule(d, gs) else {
        if won_or_fled {
            for e in ends {
                if let Some(m) = gs.party.member_mut(&e.member) {
                    let hp = progression::hp_after(if e.alive { e.hp.min(e.max_hp) } else { 0 }, e.max_hp, &f.outcome);
                    m.hp_lost = (e.max_hp - hp).max(0);
                }
            }
        }
        return out;
    };
    for e in ends {
        let Some(m) = gs.party.member(&e.member) else { continue };
        // the unwounded max HP: a member wounded when it took the field fought at the cut max (W2)
        let full_max = if m.has_condition(WOUNDED) { progression::max_hp_of(d, m, &f.world).max(e.max_hp) } else { e.max_hp };
        let (value, cause) = settle_value(&row, f, e);
        if value == CAPTURED || value == DEAD {
            if depart(d, gs, &e.member, &value, cause, &f.encounter).is_ok() {
                out.departed.push((e.member.clone(), value));
            }
            continue;
        }
        let Some(m) = gs.party.member_mut(&e.member) else { continue };
        if value == WOUNDED && m.add_condition(WOUNDED) {
            out.wounded.push(e.member.clone());
        }
        m.hp_lost = if row.hp_after == HP_FULL {
            0
        } else {
            let outcome = if won_or_fled { f.outcome.as_str() } else { "lost" };
            let hp = progression::hp_after(if e.alive { e.hp.min(e.max_hp) } else { 0 }, e.max_hp, outcome);
            (full_max - hp).max(0)
        };
    }
    out
}

/// L2, RQ2 (a): member `id` leaves the party with `life` (`DEPART_LIVES`) for `cause` (`DEPART_CAUSES`) in encounter
/// `encounter` ("" outside a battle): out of `members` and `active_ids` (its bunk is free: bunk use counts members),
/// its worn and carried gear and its pouch to the stash (its state keeps them as its record), recorded in
/// `party.departed` with the day (`gs.time`); then KS1, KS2: when the act's keepsake lives hold `life` and its
/// character names a `keepsake` item, the stash gets one and the record keeps the item id. Member ids are never
/// reused, so a keepsake is paid once. Err (nothing changes) for an unknown member, life or cause.
pub fn depart(d: &Data, gs: &mut GameState, id: &str, life: &str, cause: &str, encounter: &str) -> Result<Departed, String> {
    if !DEPART_LIVES.contains(&life) {
        return Err(format!("Unknown life '{life}' (known: {}).", DEPART_LIVES.join(", ")));
    }
    if !DEPART_CAUSES.contains(&cause) {
        return Err(format!("Unknown cause '{cause}' (known: {}).", DEPART_CAUSES.join(", ")));
    }
    let Some(at) = gs.party.members.iter().position(|m| m.id == id) else {
        return Err(format!("'{id}' is not a party member."));
    };
    let lives = manifest(d, &gs.world_id).keepsake_lives(act(gs)).to_vec();
    let member = gs.party.members.remove(at);
    gs.party.active_ids.retain(|a| a != id);
    let gear: Vec<String> = member.equipment.values().chain(&member.carried).chain(&member.pouch).cloned().collect();
    for item in gear {
        if d.has("items", &item) {
            gs.party.add_to_stash(&item, 1);
        }
    }
    let mut keepsake = String::new();
    if lives.iter().any(|l| l == life) {
        let item = json::s(d.resolve(&gs.world_id, "characters", &member.lore_ref), "keepsake");
        if !item.is_empty() && d.has("items", &item) {
            gs.party.add_to_stash(&item, 1);
            keepsake = item;
        }
    }
    let record = Departed { member, life: life.into(), cause: cause.into(), encounter: encounter.into(), day: gs.time, keepsake };
    gs.party.departed.push(record.clone());
    Ok(record)
}

/// W3: a rest clears `wounded` on every member (`progression::rest` calls this, so every rest path does: rest places,
/// Camp, CAMPAIGN1 R2's rest and its `rest` effect). The defeat wake and the retreat are not rests.
pub fn on_rest(party: &mut PartyState) {
    for m in party.members.iter_mut() {
        m.conditions.retain(|c| c != WOUNDED);
    }
}
