//! The roster (ROSTER1, kit hook; engine-only, no Godot original): a party member's life across battles. How a member
//! takes the field (`field_unit`), how a battle's end settles the party (`settle_party`), how a member leaves it
//! (`depart`) and what a rest clears (`on_rest`) live here, in the world crate, so every screen and runner applies one
//! rule. ROSTER1-L slice 2 moved the build and the settle from the battle screen with no rule change; slice 3 adds the
//! world's rule rows (3.2: the act's row turns a fate into `wounded`, `captured` or `dead`; no row = HEAL1 classic,
//! L3), the wounded build (W2), departures with their named keepsake (L1, L2, KS1, KS2) and the rest that clears
//! `wounded` (W3). Slice 4 adds bunks (BK1-BK4: the cap the `recruit` effect checks, the bench), the join level (BK5),
//! the fight minimum (BK6), the departed-recruit refusal (L1), the `roster.` read keys (RK1) and the load warnings.

use crate::data::Data;
use crate::progression;
use crate::state::{Departed, GameState, PartyState, RosterCaps, UnitState};
use aegis_core::json;
use rpgkit_battle::encounter::{PartyFate, STAY_THEN};
use rpgkit_battle::session::Session;
use rpgkit_battle::world::AiProfile;
use rpgkit_data::{roster_read_key, RosterManifest, RosterRead, RosterRule, DEPART_CAUSES, DEPART_LIVES, MEMBER_CONDITIONS, RECRUIT_LEVELS, RECRUIT_LEVEL_MAX, SETTLE_CLASSIC, WOUNDED_PCT_DEFAULT};
use rpgkit_rules::Unit;
use serde_json::Value;

/// W1: the one condition ROSTER1 defines (the kit's `MEMBER_CONDITIONS`, shared with PORTRAITFX1's checks).
pub const WOUNDED: &str = MEMBER_CONDITIONS[0];
/// RK1: the state `roster.<member>` reads for a member in the party.
pub const READY: &str = "ready";
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
/// (`progression::build_played`), at its stored HP (HealRules.start_hp), with the AI profile of the row it was built from (its base unit's, as an NPC's comes from its entry). W2: a
/// `wounded` member's max HP is cut by the played world's row's `wounded_max_hp_pct` (the default without a row) and
/// its start HP is at most that max. None when the saved member does not build.
pub fn field_unit(d: &Data, gs: &GameState, m: &UnitState, world: &str) -> Option<Fielded> {
    let built = progression::build_played(d, m, world)?;
    let profile = AiProfile::for_unit(&d.cat, &built.def);
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
    Some(Fielded { unit, pouch_unloaded: built.pouch_unloaded, profile })
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
    /// ENCOUNTER2 W3: the encounter's `stay_behind.then` (`none`, `capture`, `kill`; "" = none).
    pub then: String,
}

impl FightEnd {
    /// The end of session `s` fought in world `world` (`Session::final_outcome`, the encounter id, its objective, its
    /// `stay_behind.then`).
    pub fn of(s: &Session, world: &str) -> Self {
        let then = s.stay.as_ref().map(|b| b.then.clone()).unwrap_or_default();
        Self { outcome: s.final_outcome(), encounter: s.def.id.clone(), world: world.to_string(), escape: s.objective.objective == "escape", then }
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

/// ENCOUNTER2 W3, 3.2's first row: the life `then` gives a stayer or stander (`capture`: captured, `kill`: dead) and its
/// cause (`stayed`, `stood`); None for any other hero or `then` `none`.
fn then_departs(f: &FightEnd, e: &HeroEnd) -> Option<(&'static str, &'static str)> {
    let cause = match e.fate {
        Some(PartyFate::Stayed) => "stayed",
        Some(PartyFate::Stood) => "stood",
        _ => return None,
    };
    match f.then.as_str() {
        x if x == STAY_THEN[1] => Some((CAPTURED, cause)),
        x if x == STAY_THEN[2] => Some((DEAD, cause)),
        _ => None,
    }
}

/// 3.2: the settle value of a hero's end under row `r` in fight `f`, after `then` `none` (a stayer or stander at 0 HP
/// settles as `downed`, else as `escaped`; `capture` and `kill` depart before the table, `then_departs`); `classic`
/// where the table has no rule key (standing or escaped on a win or a flee). Returns (value, the cause when it departs).
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
/// is HEAL1's `hp_after` (a loss heals, as the wake does) written back against the unwounded max (W2). Before the table
/// (L5's `then`, ENCOUNTER2 W3, every world, with or without a row): with `stay_behind.then` `capture` or `kill` each
/// stayer or stander departs captured or dead (cause `stayed` or `stood`, gear and pouch to the stash, the keepsake).
/// `battle::apply` calls it first, so every runner's `after_fight` settles the party before the rewards and
/// CAMPAIGN1's battle tick and its retreat (L5: the retreat then moves a cell that already carries the loss).
pub fn settle_party(d: &Data, gs: &mut GameState, f: &FightEnd, ends: &[HeroEnd]) -> Settled {
    let mut out = Settled::default();
    for e in ends {
        if let Some(m) = gs.party.member_mut(&e.member) {
            m.pouch = e.pouch.iter().chain(&e.unloaded).cloned().collect();
        }
    }
    let mut home: Vec<&HeroEnd> = Vec::new();
    for e in ends {
        if let Some((life, cause)) = then_departs(f, e) {
            if depart(d, gs, &e.member, life, cause, &f.encounter).is_ok() {
                out.departed.push((e.member.clone(), life.to_string()));
                continue;
            }
        }
        home.push(e);
    }
    let ends = home;
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

/// 3.2 on every runner: the ends of a fight settled without a battle (REWARD1 2.5's default tally; the debug skip, the
/// campaign preview): each active member as it would take the field in world `world` (`field_unit`), standing at its
/// stored HP on a win or a flee, downed on a loss, its saved pouch unchanged. `battle::settle` uses them only in a
/// world with a rule row, so a classic world's fight without a battle leaves the party as before.
pub fn default_ends(d: &Data, gs: &GameState, world: &str, outcome: &str) -> Vec<HeroEnd> {
    let lost = outcome == "lost";
    gs.party
        .active_members()
        .into_iter()
        .filter_map(|m| {
            let f = field_unit(d, gs, m, world)?;
            let hp = if lost { 0 } else { f.unit.hp as i64 };
            Some(HeroEnd {
                member: m.id.clone(),
                hp,
                max_hp: f.unit.max_hp as i64,
                alive: hp > 0,
                pouch: m.pouch.clone(),
                unloaded: Vec::new(),
                fate: Some(if hp > 0 { PartyFate::Standing } else { PartyFate::Downed }),
            })
        })
        .collect()
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

// ---- bunks, joining, the fight minimum, read keys (slice 4) ----

/// BK7: world `w`'s resolved roster settings (its party max and `roster` block), kept on the party (never saved).
pub fn caps(d: &Data, w: &str) -> RosterCaps {
    RosterCaps { party_max: d.party_max(w), manifest: manifest(d, w) }
}

/// BK1: the bunk cap: `bunks` (0 = the party max) plus the int flag `bunks_flag` (clamped at 0), at most `bunks_max`.
pub fn bunk_cap(gs: &GameState) -> i64 {
    let caps = &gs.party.roster;
    let flag = &caps.manifest.bunks_flag;
    let value = if flag.is_empty() { 0 } else { gs.flag(flag).filter(|v| json::is_int(v)).map_or(0, json::to_int) };
    caps.bunk_cap(value)
}

/// BK2: the bunks in use: every member (housing `bunk` or `crypt`; `crypt_only` and its crypt places come with
/// ROSTER1-P, so today every member takes one). Departed members free theirs (L2).
pub fn bunks_used(party: &PartyState) -> i64 {
    party.members.len() as i64
}

/// RK1 `roster.open`: the world's `open_flag` holds (true when it names none).
pub fn is_open(gs: &GameState) -> bool {
    let flag = &gs.party.roster.manifest.open_flag;
    flag.is_empty() || gs.flag_true(flag)
}

/// BK3: why the party takes no new member now ("" = a bunk is free): with `bunks` 0 the RECRUIT1 text, else the bunk
/// text. Pool entries and hub guests take no bunk (they are not members).
pub fn full_text(gs: &GameState) -> String {
    let (n, cap) = (bunks_used(&gs.party), bunk_cap(gs));
    if n < cap {
        String::new()
    } else if gs.party.roster.has_bunks() {
        format!("Every bunk is taken ({n} of {cap}).")
    } else {
        format!("The party is full ({n} of {cap}).")
    }
}

/// L1: whether world character `character_ref` left the party (its departed record, by member id or character qid):
/// a recruit of it is refused, as member ids are never reused.
pub fn has_departed(d: &Data, gs: &GameState, character_ref: &str) -> bool {
    let c = d.resolve(&gs.world_id, "characters", character_ref);
    if c.is_empty() {
        return false;
    }
    let (id, qid) = (json::s(c, "id"), json::s(c, "_qid"));
    gs.party.departed.iter().any(|x| x.member.id == id || (!qid.is_empty() && x.member.lore_ref == qid))
}

/// L1: the refusal of recruiting a departed character.
pub fn gone_text(name: &str) -> String {
    format!("{name} is gone.")
}

/// BK5: the total XP a recruit joins with for the `recruit` effect's `level` (absent = `highest`): `highest` keeps
/// RECRUIT1 (its own XP raised to the best active member's), `average` is the active members' mean level rounded
/// down, a whole number 1..20 is that level. `own_xp` is the recruit's XP at its base unit's level.
pub fn join_xp(d: &Data, party: &PartyState, own_xp: i64, level: &Value) -> i64 {
    let active = party.active_members();
    let level = json::normalize(level);
    if json::is_int(&level) {
        return progression::xp_for_level(d, json::to_int(&level).clamp(1, RECRUIT_LEVEL_MAX));
    }
    if level.as_str() == Some(RECRUIT_LEVELS[1]) {
        if active.is_empty() {
            return own_xp;
        }
        let sum: i64 = active.iter().map(|m| progression::level_for(d, m.total_xp)).sum();
        return progression::xp_for_level(d, sum / active.len() as i64);
    }
    active.iter().fold(own_xp, |xp, m| xp.max(m.total_xp))
}

/// BK6: the refusal of fighting world encounter `encounter_ref` with the active party ("" = it may start): the
/// encounter's `party_min` above the active members' count. World encounters only (a catalog encounter has no
/// minimum). The dialogue `fight` precheck and the battle start read it.
pub fn party_min_text(d: &Data, gs: &GameState, encounter_ref: &str) -> String {
    party_min_refusal(gs, d.resolve(&gs.world_id, "encounters", encounter_ref))
}

/// BK6 on a resolved encounter entry `e` (the battle screen reads the entry of the fight's world, `battle::fight_world`).
pub fn party_min_refusal(gs: &GameState, e: &json::Dict) -> String {
    let need = json::i(e, "party_min", 0);
    let have = gs.party.active_members().len() as i64;
    if e.is_empty() || need <= have {
        String::new()
    } else {
        format!("Take {need} into this fight ({have} in the party).")
    }
}

/// RK1: the value of `roster.` read key `key` from the party (None when `key` is not one): `roster.living` (members,
/// in bunks or crypts), `roster.bunks` (the cap), `roster.bunks_free`, `roster.pool` (0 until ROSTER1-P), `roster.open`,
/// `roster.departed.<life>` (a count), `roster.<member>` (`ready`, its departed life, or "" when unknown).
pub fn read_key(gs: &GameState, key: &str) -> Option<Value> {
    let p = &gs.party;
    Some(match roster_read_key(key)? {
        RosterRead::Living => json::int(p.members.len() as i64),
        RosterRead::Bunks => json::int(bunk_cap(gs)),
        RosterRead::BunksFree => json::int((bunk_cap(gs) - bunks_used(p)).max(0)),
        RosterRead::Pool => json::int(0),
        RosterRead::Open => Value::Bool(is_open(gs)),
        RosterRead::Departed(life) => json::int(p.departed.iter().filter(|x| x.life == life).count() as i64),
        RosterRead::Member(id) => Value::String(if p.member(id).is_some() {
            READY.to_string()
        } else {
            p.departed_of(id).map(|x| x.life.clone()).unwrap_or_default()
        }),
    })
}

/// 11.4: the roster warnings of a loaded save (never a refusal): bunk use above the cap (the world's roster block
/// changed since the save, or a save from before it). A world without a roster block warns of nothing (Godot saves
/// may hold more members than the party max). Crypt places are checked with ROSTER1-P's housing.
pub fn load_warnings(gs: &GameState) -> Vec<String> {
    let (n, cap) = (bunks_used(&gs.party), bunk_cap(gs));
    if gs.party.roster.has_block() && n > cap {
        vec![format!("{n} members for {cap} bunks: no one new joins until a bunk is free.")]
    } else {
        Vec::new()
    }
}
