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
use crate::state::{Departed, GameState, PartyState, RosterCaps, UnitState, LOADOUT_SLOTS};
use aegis_core::json;
use rpgkit_battle::encounter::{PartyFate, STAY_THEN};
use rpgkit_battle::session::Session;
use rpgkit_battle::world::AiProfile;
use rpgkit_data::{roster_read_key, RosterManifest, RosterRead, RosterRule, DEPART_LIVES, MEMBER_CONDITIONS, RECRUIT_LEVELS, RECRUIT_LEVEL_MAX, NO_ITEM, SETTLE_CLASSIC, WOUNDED_PCT_DEFAULT};
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
    /// AUDIT-04: the intent it starts a round-intent battle in (its character's `default_intent`, else its base unit's;
    /// "" = none): `Prebuilt::default_intent`.
    pub default_intent: String,
}

/// Member `m` as it fights in world `world`, the world whose encounter it fights: its class path and deck rule there
/// (`progression::build_played`), at its stored HP (HealRules.start_hp), with the AI profile of the row it was built from (its base unit's, as an NPC's comes from its entry). W2: a
/// `wounded` member's max HP is cut by the played world's row's `wounded_max_hp_pct` (the default without a row) and
/// its start HP is at most that max. None when the saved member does not build.
pub fn field_unit(d: &Data, gs: &GameState, m: &UnitState, world: &str) -> Option<Fielded> {
    let built = progression::build_played(d, m, world)?;
    let profile = AiProfile::for_unit(&d.cat, &built.def);
    // AUDIT-04: `build_as` laid the character's default intent over the base unit's
    let default_intent = built.def.default_intent.clone();
    let mut unit = built.unit;
    // PK2-S5b: wear is tracked only while weapon instances are active (the campaign act reached, `weapons::active`);
    // the build reads the rule and the manifest key but not the act
    if !unit.wear.is_empty() && !crate::weapons::active(d, gs) {
        unit.wear.clear();
    }
    // PR7-8 (RELIC-SIG1): the campaign act its `from_act` effects read (the build does not read the act)
    unit.set_campaign_act(&d.cat_in(world), act(gs));
    let max = unit.max_hp as i64;
    let mut hp = progression::start_hp(max, m.hp_lost);
    if m.has_condition(WOUNDED) {
        let pct = rule(d, gs).map_or(WOUNDED_PCT_DEFAULT, |r| r.wounded_max_hp_pct);
        let cut = wounded_max(max, pct);
        unit.max_hp = cut as i32;
        hp = hp.min(cut);
    }
    unit.hp = hp as i32;
    Some(Fielded { unit, pouch_unloaded: built.pouch_unloaded, profile, default_intent })
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
    /// PK1-S1 (docs/sessions/PK1_CORRUPTION.md 3.5): the marks the battle recorded on it (mark id, value), in order
    /// (`rpgkit_battle::mark`); the settle shows them (`battle::apply`'s `marks`) and PK2-S1 moves the member's unit
    /// tracks by them (`settle_tracks`).
    pub marks: Vec<(String, i64)>,
    /// PK2-S1 (FB-174 (5)): the unit track whose departure stage turned the member on everyone, when it still fights
    /// everyone at the battle end (`rpgkit_battle::mark::hostile_at_end`; "" = it does not): the settle departs it.
    pub hostile: String,
    /// PK2-S4 (`docs/sessions/PK2_GRAFTS.md` 5): the main-hand weapon families it attacked with this battle
    /// (`rpgkit_battle::world::World::weapon_attacks`): the settle's mastery use (`mastery::settle`).
    pub attacked: Vec<String>,
    /// PK2-S5b (`docs/sessions/PK2_GRAFTS.md` 7): the durability the battle spent from the weapon instances it wields
    /// (slot save key, amount; slot order, spent slots only; `rpgkit_battle::wear::spent`): the settle takes it from the
    /// wielded instances, each held at 0 or more (`weapons::wear`).
    pub worn: Vec<(String, i64)>,
    /// PR6-6 (`docs/sessions/PK4_PK5_CAMPAIGN_HUB.md` 1, B5): the highest and the last value it held on the ruleset's
    /// channel meter (`rpgkit_battle::channel::meter_id`, data-configured; 0 without one or a hero that holds none), and
    /// the commands it started in a Channel round (`World::channel_actions`). A settle without a battle reads 0 for all
    /// three; the campaign tick reads them (`campaign::HeroTally`: meter inputs and triggers with a `hero` filter).
    pub strain_peak: i64,
    pub strain_end: i64,
    pub channel_actions: i64,
}

/// The ends of session `s`'s fielded members: `members` (member id, unloaded pouch ids) in the order they were fielded,
/// paired with the battle-start party side in order (the heroes come first; guests follow and have no member). PK2-S1
/// (FB-174 (5)): the start side, not today's allies, so a hero a unit track turned on everyone (no longer an ally)
/// keeps its member.
pub fn hero_ends(s: &Session, members: &[(String, Vec<String>)]) -> Vec<HeroEnd> {
    let fates = s.party_fates();
    members
        .iter()
        .zip(s.w.marks.start_party())
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
                marks: s.w.marks.of(a).into_iter().map(|(m, v)| (m, i64::from(v))).collect(),
                hostile: rpgkit_battle::mark::hostile_at_end(&s.w, a),
                attacked: s.w.weapon_attacks.get(&a).cloned().unwrap_or_default(),
                worn: rpgkit_battle::wear::spent(&s.w, a),
                strain_peak: channel_meter_read(s, a, rpgkit_battle::meter::peak),
                strain_end: channel_meter_read(s, a, rpgkit_battle::meter::value),
                channel_actions: i64::from(s.w.channel_actions.get(&a).copied().unwrap_or(0)),
            }
        })
        .collect()
}

/// PR6-6 (B5): what `read` gives for unit `a` on the session's channel meter (0 without the rule, a channel meter, or
/// a gauge of it on `a`).
fn channel_meter_read(s: &Session, a: usize, read: fn(&rpgkit_battle::world::World, usize, &str) -> Option<i32>) -> i64 {
    let id = rpgkit_battle::channel::meter_id(&s.w);
    if id.is_empty() {
        return 0;
    }
    read(&s.w, a, id).map_or(0, i64::from)
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
    /// A1.2-K3: the Results lines of the pouch slots auto-fill left short (`auto_fill`).
    pub stash_short: Vec<String>,
    /// PK2-S1: the unit tracks the battle's marks moved (`settle_tracks`), in order.
    pub tracks: Vec<TrackMove>,
    /// PK2-S1: the worn items a member grant `forbid_hands` sent to the stash (member, item id).
    pub stashed: Vec<(String, String)>,
    /// PR5-26 review F2: the slots left empty (`NO_ITEM`) because the grant refuses their default (member, slot).
    pub cleared: Vec<(String, String)>,
    /// PK2-S2: the items fallen members left in the stash (member, item id; `fallen_remains`).
    pub remains: Vec<(String, String)>,
    /// PK2-S4: the weapon mastery the members who came home gained by use (`mastery::settle`), in order.
    pub mastery: Vec<crate::mastery::Gain>,
    /// PK2-FOLLOWUP (PR5-36, FB-218 (d)): the battle's wear taken off the wielded weapon instances (`weapons::wear`), in
    /// order (a weapon at 0 is dulled).
    pub wear: Vec<crate::weapons::WearMove>,
}

/// PK2-S1: a member's unit track stage moved at the settle.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackMove {
    pub member: String,
    pub track: String,
    pub from: i64,
    pub to: i64,
}

/// PK2-S1 (`docs/sessions/PK2_GRAFTS.md` 2, FB-174 (5)): the unit tracks of fielded member `e` at the settle (before
/// `then` and the rule row): its battle marks left after any `unmark` move each track whose `mark` they are (summed,
/// held within the row; `out.tracks`). Returns the departure it settles with (life, cause): a track at a stage with a
/// departure (catalog order; Stage V), else a hero still hostile at the battle end (`HeroEnd::hostile`) with its
/// track's departure. None for anyone else or a member no longer in the party.
fn settle_tracks(d: &Data, gs: &mut GameState, e: &HeroEnd, out: &mut Settled) -> Option<(String, String)> {
    let m = gs.party.member_mut(&e.member)?;
    for t in d.cat.unit_tracks.values().filter(|t| !t.mark.is_empty()) {
        let sum: i64 = e.marks.iter().filter(|(id, _)| *id == t.mark).map(|(_, v)| *v).sum();
        if sum == 0 {
            continue;
        }
        let from = m.track_stage(&t.id);
        let to = i64::from(t.clamp(from + sum));
        if to == from {
            continue;
        }
        if to == 0 {
            m.tracks.shift_remove(&t.id);
        } else {
            m.tracks.insert(t.id.clone(), to);
        }
        out.tracks.push(TrackMove { member: e.member.clone(), track: t.id.clone(), from, to });
    }
    let at_departure = d.cat.unit_tracks.values().find(|t| !t.departure_at(t.clamp(m.track_stage(&t.id))).is_empty());
    if let Some(t) = at_departure {
        return Some((t.departure_at(t.clamp(m.track_stage(&t.id))).to_string(), t.cause.clone()));
    }
    let t = d.cat.unit_tracks.get(&e.hostile)?;
    Some((t.departure_at(t.top()).to_string(), t.cause.clone()))
}

/// PK2-S1: the worn items member `id`'s member grant `forbid_hands` refuses go to the stash (a stage reached in the
/// battle; PK2 9 default 5). Returns the item ids. PR5-26 (audit B2): a slot whose default (its base unit's or class's
/// item, `UnitState::forbids_default`) the grant refuses too holds `NO_ITEM` after, so the build leaves it empty instead
/// of equipping the default again (the default is no stash item: nothing goes to the stash for it). Returns the stashed
/// item ids and (review F2) the slots so emptied.
fn stash_forbidden(d: &Data, gs: &mut GameState, id: &str) -> (Vec<String>, Vec<String>) {
    let world = gs.world_id.clone();
    let Some(m) = gs.party.member(id) else { return (Vec::new(), Vec::new()) };
    let hands = progression::forbid_hands(d, m);
    if hands == 0 {
        return (Vec::new(), Vec::new());
    }
    let forbidden = |item: &str| d.cat.items.get(item).is_some_and(|i| rpgkit_rules::unit::forbids_hands(hands, i));
    let mut moves: Vec<(String, Option<String>, bool)> = Vec::new();
    for slot in LOADOUT_SLOTS {
        let stash = match m.equipment.get(slot) {
            Some(item) if item == NO_ITEM || !forbidden(item) => continue,
            Some(item) => Some(item.clone()),
            None => None,
        };
        let empty = m.forbids_default(d, &world, slot);
        if stash.is_some() || empty {
            moves.push((slot.to_string(), stash, empty));
        }
    }
    let (mut items, mut cleared) = (Vec::new(), Vec::new());
    for (slot, stash, empty) in moves {
        let Some(m) = gs.party.member_mut(id) else { break };
        m.equipment.shift_remove(&slot);
        if empty {
            m.equipment.insert(slot.clone(), NO_ITEM.to_string());
            cleared.push(slot);
        }
        if let Some(item) = stash {
            gs.party.add_to_stash(&item, 1);
            items.push(item);
        }
    }
    (items, cleared)
}

/// PK2-S2 (`docs/sessions/PK2_GRAFTS.md` 3, "Losing a grafted unit"): a member who just departed `dead` while the party
/// holds the field (`FightEnd::field_held`) leaves its body's item in the stash: its character's `remains`, else its
/// class's (`ClassDef::remains`; "" = none, an unknown item = none); `out.remains` names it. A captured member, a
/// unit track's departure (`lost`) and a fight the party does not hold leave nothing; its installed grafts are lost with
/// it (the body is no gear: `depart` stashes none).
fn fallen_remains(d: &Data, gs: &mut GameState, f: &FightEnd, member: &str, life: &str, out: &mut Settled) {
    if life != DEAD || !f.field_held() {
        return;
    }
    let Some(record) = gs.party.departed.iter().rev().find(|r| r.member.id == member) else { return };
    let own = json::s(d.resolve(&gs.world_id, "characters", &record.member.lore_ref), rpgkit_data::REMAINS_KEY);
    let item = if own.is_empty() { d.cat.classes.get(&record.member.class_id(d)).map(|c| c.remains.clone()).unwrap_or_default() } else { own };
    if item.is_empty() || !d.has("items", &item) {
        return;
    }
    gs.party.add_to_stash(&item, 1);
    out.remains.push((member.to_string(), item));
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
        PartyFate::Dead => return (DEAD.to_string(), "bled_out"),
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

/// A1.2-K3 (INTENT2-K3, "Stacks & Auto-Fill"): tops the pouches of `members` (party slot order) up from the stash to
/// each slot's packed target (`UnitState::packed`), under the played world's `pouch_stacks` (classic: nothing). Item by
/// item in slot order, the copies go one at a time in round-robin passes, each pass ordered by the higher packed
/// target first (descending, ties by party slot order), until the stash runs out or everyone is full. Returns a
/// Results line `Stash short: <name> <cur> of <packed> <item>` for each slot still below its target.
pub fn auto_fill(d: &Data, gs: &mut GameState, world: &str, members: &[String]) -> Vec<String> {
    if !d.ruleset(world).numbers.pouch_stacks {
        return Vec::new();
    }
    let mut items: Vec<String> = Vec::new();
    for id in members {
        for slot in gs.party.member(id).map(UnitState::pouch_slots).unwrap_or_default() {
            if !items.contains(&slot) {
                items.push(slot);
            }
        }
    }
    for item in &items {
        loop {
            let mut want: Vec<(usize, i64)> = Vec::new();
            for (i, id) in members.iter().enumerate() {
                if let Some(m) = gs.party.member(id) {
                    if m.pouch_slots().contains(item) && m.pouch_count(item) < m.packed(item) {
                        want.push((i, m.packed(item)));
                    }
                }
            }
            // stable: ties keep party slot order
            want.sort_by_key(|w| std::cmp::Reverse(w.1));
            let mut moved = false;
            for (i, packed) in want {
                if gs.party.stash.get(item).copied().unwrap_or(0) <= 0 {
                    break;
                }
                let Some(m) = gs.party.member_mut(&members[i]) else { continue };
                if m.pouch_count(item) < packed {
                    // keep the slot's place: the copy joins its stack
                    let at = m.pouch.iter().rposition(|p| p == item).map_or(m.pouch.len(), |k| k + 1);
                    m.pouch.insert(at, item.clone());
                    gs.party.remove_from_stash(item, 1);
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }
    }
    let mut lines = Vec::new();
    for id in members {
        let Some(m) = gs.party.member(id) else { continue };
        for slot in m.pouch_slots() {
            let (cur, packed) = (m.pouch_count(&slot), m.packed(&slot));
            if cur < packed {
                let name = d.cat.items.get(&slot).map_or(slot.clone(), |i| i.name.clone());
                lines.push(format!("Stash short: {} {cur} of {packed} {name}", progression::display_name(d, m)));
            }
        }
    }
    lines
}

/// A1.2-K3 (INTENT2-K3): combat start: each active member's pouch slots without a packed target take their current
/// count as one (so the fight's spent copies are refilled after it), then `auto_fill` for the active members.
pub fn auto_fill_at_start(d: &Data, gs: &mut GameState, world: &str) -> Vec<String> {
    // PR6-9: a siege room after the first starts with the pouches as the last room left them (`carry_pouch`)
    if !d.ruleset(world).numbers.pouch_stacks || crate::siege::holds_pouch(d, gs, false) {
        return Vec::new();
    }
    let ids: Vec<String> = gs.party.active_members().into_iter().map(|m| m.id.clone()).collect();
    for id in &ids {
        if let Some(m) = gs.party.member_mut(id) {
            for slot in m.pouch_slots() {
                if !m.pouch_packed.contains_key(&slot) {
                    let n = m.pouch_count(&slot).clamp(1, crate::state::stack_limit(d, &slot));
                    m.pouch_packed.insert(slot, n);
                }
            }
        }
    }
    auto_fill(d, gs, world, &ids)
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
/// PK2-S1: first of all each member's unit tracks (`settle_tracks`: the marks move the stages; a departure stage or a
/// hero still hostile at the end departs it, `lost` for the track's cause), then the worn items its `forbid_hands`
/// refuses go to the stash. PK2-S5b: before everything the battle's weapon wear comes off the wielded instances
/// (`HeroEnd::worn`, `weapons::wear`).
/// `battle::apply` calls it first, so every runner's `after_fight` settles the party before the rewards and
/// CAMPAIGN1's battle tick and its retreat (L5: the retreat then moves a cell that already carries the loss).
pub fn settle_party(d: &Data, gs: &mut GameState, f: &FightEnd, ends: &[HeroEnd]) -> Settled {
    // PK2-S5b: first the wear the battle spent comes off the wielded instances (before a departure or `forbid_hands`
    // moves them to the stash)
    let wear = crate::weapons::wear(gs, ends);
    let mut out = settle_fates(d, gs, f, ends);
    out.wear = wear;
    // ROSTER1 FZ2, FZ4 (PK3 3.1): each fielded Martyr counts the deployment; a last stand departs dead after it
    out.departed.extend(settle_fuses(d, gs, f, ends));
    // A1.2-K3 (INTENT2-K3): after every departure, the members who came home top their pouches up from the stash (the
    // fight's world), so a departing member takes nothing and only survivors are named short
    let home: Vec<String> = ends.iter().map(|e| e.member.clone()).filter(|m| !out.departed.iter().any(|(x, _)| x == m) && gs.party.member(m).is_some()).collect();
    // PR6-9: not between a siege's rooms (`carry_pouch`; the siege's `restock` tops up instead)
    if !crate::siege::holds_pouch(d, gs, true) {
        out.stash_short = auto_fill(d, gs, &f.world, &home);
    }
    // PK2-S4: the members who came home gain mastery in the families they attacked with (from `durability.from_act`)
    let came_home: Vec<&HeroEnd> = ends.iter().filter(|e| home.contains(&e.member)).collect();
    out.mastery = crate::mastery::settle(d, gs, &came_home);
    crate::talks::count_together(gs, &f.outcome, &home);
    out
}

/// `settle_party` before the auto-fill: the pouches written back, the departures and the HP and conditions.
fn settle_fates(d: &Data, gs: &mut GameState, f: &FightEnd, ends: &[HeroEnd]) -> Settled {
    let mut out = Settled::default();
    for e in ends {
        if let Some(m) = gs.party.member_mut(&e.member) {
            m.pouch = e.pouch.iter().chain(&e.unloaded).cloned().collect();
            // PK2-S2 (FB-167 Q6): the floor offset held for this battle only
            m.floor_offset = 0;
        }
    }
    let mut home: Vec<&HeroEnd> = Vec::new();
    for e in ends {
        // PK2-S1: the unit tracks first: a departure stage (or a hero still hostile at the end) departs the member
        if let Some((life, cause)) = settle_tracks(d, gs, e, &mut out) {
            if depart(d, gs, &e.member, &life, &cause, &f.encounter).is_ok() {
                out.departed.push((e.member.clone(), life));
                continue;
            }
        }
        let (items, cleared) = stash_forbidden(d, gs, &e.member);
        out.stashed.extend(items.into_iter().map(|item| (e.member.clone(), item)));
        out.cleared.extend(cleared.into_iter().map(|slot| (e.member.clone(), slot)));
        if let Some((life, cause)) = then_departs(f, e) {
            if depart(d, gs, &e.member, life, cause, &f.encounter).is_ok() {
                out.departed.push((e.member.clone(), life.to_string()));
                fallen_remains(d, gs, f, &e.member, life, &mut out);
                continue;
            }
        }
        // PK3: a party member who bled out during battle departs permanently dead with cause "bled_out" on every outcome.
        if e.fate == Some(PartyFate::Dead) {
            if depart(d, gs, &e.member, DEAD, "bled_out", &f.encounter).is_ok() {
                out.departed.push((e.member.clone(), DEAD.to_string()));
                fallen_remains(d, gs, f, &e.member, DEAD, &mut out);
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
                fallen_remains(d, gs, f, &e.member, &value, &mut out);
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
                marks: Vec::new(),
                hostile: String::new(),
                attacked: Vec::new(),
                worn: Vec::new(),
                strain_peak: 0,
                strain_end: 0,
                channel_actions: 0,
            })
        })
        .collect()
}

/// L2, RQ2 (a): member `id` leaves the party with `life` (`DEPART_LIVES`) for `cause` (`Catalog::depart_causes`) in encounter
/// `encounter` ("" outside a battle): out of `members` and `active_ids` (its bunk is free: bunk use counts members),
/// its worn and carried gear and its pouch to the stash (its state keeps them as its record), recorded in
/// `party.departed` with the day (`gs.time`); then KS1, KS2: when the act's keepsake lives hold `life` and its
/// character names a `keepsake` item, the stash gets one and the record keeps the item id. Member ids are never
/// reused, so a keepsake is paid once. KS3 (PK3 4): a procedural member (one with a recruit template) gets its class's
/// keepsake instance instead (`mint_keepsake`; the record keeps the instance id). Err (nothing changes) for an unknown
/// member, life or cause.
pub fn depart(d: &Data, gs: &mut GameState, id: &str, life: &str, cause: &str, encounter: &str) -> Result<Departed, String> {
    if !DEPART_LIVES.contains(&life) {
        return Err(format!("Unknown life '{life}' (known: {}).", DEPART_LIVES.join(", ")));
    }
    if !d.cat.depart_cause_ok(cause) {
        return Err(format!("Unknown cause '{cause}' (known: {}).", d.cat.depart_causes().join(", ")));
    }
    let Some(at) = gs.party.members.iter().position(|m| m.id == id) else {
        return Err(format!("'{id}' is not a party member."));
    };
    let lives = manifest(d, &gs.world_id).keepsake_lives(act(gs)).to_vec();
    let mut member = gs.party.members.remove(at);
    gs.party.active_ids.retain(|a| a != id);
    // PK2-S5a: its weapon instances go to the stash with its gear (the record keeps none)
    let held: Vec<crate::weapons::Weapon> = std::mem::take(&mut member.weapons).into_values().collect();
    gs.party.stash_weapons.extend(held);
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
        } else if let Some(k) = mint_keepsake(d, gs, &member) {
            // KS3 (PK3 4): a procedural member's keepsake is an instance in `party.keepsakes`, never a stash item
            keepsake = k.id.clone();
            gs.party.keepsakes.push(k);
            gs.party.sync_keepsakes();
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

/// The squad's active size (party_model.gd toggle; PR5-31 moved it from the game's party screen with no rule change):
/// member `id` joins or leaves the active members. An empty active list (every member up to `max_active` fights) is
/// first written out as those members; the last active member stays, and the list holds at most `max_active`. Err: why
/// not (the written-out list is kept).
pub fn toggle_active(party: &mut PartyState, id: &str) -> Result<(), String> {
    if party.active_ids.is_empty() {
        party.active_ids = party.members.iter().take(party.max_active as usize).map(|m| m.id.clone()).collect();
    }
    if let Some(i) = party.active_ids.iter().position(|x| *x == id) {
        if party.active_ids.len() <= 1 {
            return Err("Keep at least one active member.".into());
        }
        party.active_ids.remove(i);
    } else if party.active_ids.len() as i64 >= party.max_active {
        return Err(format!("The active party holds at most {}.", party.max_active));
    } else {
        party.active_ids.push(id.to_string());
    }
    Ok(())
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

/// BK2: the bunks in use: every member housed `bunk` or `crypt` (PK3 3.1: a `crypt_only` Living Martyr takes none,
/// `crypts_used`). Departed members free theirs (L2).
pub fn bunks_used(party: &PartyState) -> i64 {
    party.members.iter().filter(|m| m.housing() != CRYPT_ONLY).count() as i64
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
/// in bunks or crypts), `roster.bunks` (the cap), `roster.bunks_free`, `roster.pool` (the pool entries), `roster.open`,
/// `roster.departed.<life>` (a count), `roster.<member>` (`ready`, its departed life, or "" when unknown).
pub fn read_key(gs: &GameState, key: &str) -> Option<Value> {
    let p = &gs.party;
    Some(match roster_read_key(key)? {
        RosterRead::Living => json::int(p.members.len() as i64),
        RosterRead::Bunks => json::int(bunk_cap(gs)),
        RosterRead::BunksFree => json::int((bunk_cap(gs) - bunks_used(p)).max(0)),
        RosterRead::Pool => json::int(p.pool.len() as i64),
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
/// may hold more members than the party max). PK3 3.1: crypt use above the crypt places likewise.
pub fn load_warnings(gs: &GameState) -> Vec<String> {
    let (n, cap) = (bunks_used(&gs.party), bunk_cap(gs));
    let mut out = Vec::new();
    if gs.party.roster.has_block() && n > cap {
        out.push(format!("{n} members for {cap} bunks: no one new joins until a bunk is free."));
    }
    let (c, places) = (crypts_used(&gs.party), crypt_cap(gs));
    if gs.party.roster.has_block() && c > places {
        out.push(format!("{c} members for {places} crypt places: no Martyr joins until a place is free."));
    }
    out
}

// ---- the pool, recruits, housing, Living Martyrs and procedural keepsakes (ROSTER1-P/-M/-K, PK3 3-4) ----

/// PO3 (PK3 3.1): the refusal of swearing in, retiring or dismissing while the roster's `open_flag` does not hold.
pub const CLOSED_TEXT: &str = "The cell takes no one new yet.";
/// FZ4 (PK3 3.1): the life and cause of a Living Martyr retired at its spent fuse, and the cause of a last stand's
/// death (a roster choice, `rpgkit_data::DEPART_CAUSES`).
pub const RETIRED: &str = "retired";
pub const DISMISSED: &str = "dismissed";
pub const ROSTER_CAUSE: &str = "roster";
/// BK2 (PK3 3.1): the housing of a Living Martyr: a crypt place and no bunk.
pub const CRYPT_ONLY: &str = rpgkit_data::HOUSING[2];

/// PG1: the `recruits` template `r` names as seen from the played world (qid, row); None when it names none.
pub fn template(d: &Data, gs: &GameState, r: &str) -> Option<(String, rpgkit_data::RecruitDef)> {
    let row = d.resolve(&gs.world_id, rpgkit_data::RECRUITS_KIND, r);
    (!row.is_empty()).then(|| (json::s(row, "_qid"), rpgkit_data::RecruitDef::from_obj(row)))
}

/// PK3 3.2: a procedural member's portrait: its template's `art` (a file under the game's assets; "" = none, the body's
/// common portrait as for anyone else). The portrait screens read it before the character chain.
pub fn portrait_of(d: &Data, m: &UnitState) -> String {
    if m.template.is_empty() {
        return String::new();
    }
    let world = aegis_content::split_qid(&m.template).map(|(w, _)| w).unwrap_or_default();
    json::s(d.resolve(&world, rpgkit_data::RECRUITS_KIND, &m.template), "art")
}

/// FZ1: the fuse rules of member `m`'s template (None: no template, or a template without a fuse).
pub fn fuse_def(d: &Data, m: &UnitState) -> Option<rpgkit_data::FuseDef> {
    if m.template.is_empty() || m.fuse.is_none() {
        return None;
    }
    let world = aegis_content::split_qid(&m.template).map(|(w, _)| w).unwrap_or_default();
    let row = d.resolve(&world, rpgkit_data::RECRUITS_KIND, &m.template);
    rpgkit_data::RecruitDef::from_obj(row).fuse
}

/// FZ2: member `m`'s fuse value on day `day`: `deploys x per_deploy + floor((day - since) / stasis_days)`, never below
/// the saved value (nothing lowers it). 0 for a member without a fuse.
pub fn fuse_value(d: &Data, m: &UnitState, day: i64) -> i64 {
    let (Some(f), Some(def)) = (m.fuse.as_ref(), fuse_def(d, m)) else { return 0 };
    def.value(f.deploys, day - f.since).max(f.value)
}

/// FZ2: writes every member's fuse value for the game's day (`gs.time`). The settle calls it; a runner may call it
/// after days pass (`fuse_value` reads the day anyway).
pub fn refresh_fuses(d: &Data, gs: &mut GameState) {
    let day = gs.time;
    let values: Vec<(usize, i64)> = gs.party.members.iter().enumerate().filter(|(_, m)| m.fuse.is_some()).map(|(i, m)| (i, fuse_value(d, m, day))).collect();
    for (i, v) in values {
        if let Some(f) = gs.party.members[i].fuse.as_mut() {
            f.value = v;
        }
    }
}

/// FZ3, FZ4 (PK3 3.1): the passives member `m`'s fuse grants at its saved value: every tier at or below it (cumulative,
/// tier order), then the template's `last_passive` on a last stand.
pub fn fuse_passives(d: &Data, m: &UnitState) -> Vec<String> {
    let (Some(f), Some(def)) = (m.fuse.as_ref(), fuse_def(d, m)) else { return Vec::new() };
    let mut out = def.passives_at(f.value);
    if f.last_stand && !def.last_passive.is_empty() && !out.contains(&def.last_passive) {
        out.push(def.last_passive.clone());
    }
    out
}

/// FZ4: member `m`'s fuse is spent on day `day` (`value >= max`).
pub fn fuse_spent(d: &Data, m: &UnitState, day: i64) -> bool {
    fuse_def(d, m).is_some_and(|def| def.spent(fuse_value(d, m, day)))
}

/// FZ4 (PK3 3.1): the active members whose next deployment needs the choice (retire or last stand): fuse spent, no
/// last stand chosen. The deployment prompt reads it (`fuse_refusal`).
pub fn fuse_choices(d: &Data, gs: &GameState) -> Vec<String> {
    gs.party.active_members().into_iter().filter(|m| fuse_spent(d, m, gs.time) && !m.fuse.as_ref().is_some_and(|f| f.last_stand)).map(|m| m.id.clone()).collect()
}

/// FZ4: why the active party can't deploy before a choice ("" = it can): the first member `fuse_choices` names.
pub fn fuse_refusal(d: &Data, gs: &GameState) -> String {
    match fuse_choices(d, gs).first().and_then(|id| gs.party.member(id)) {
        None => String::new(),
        Some(m) => format!("{}'s fuse is spent: retire them or send them on a last stand.", progression::display_name(d, m)),
    }
}

/// FZ4 (PK3 3.1): the player sends member `id` on its last stand: it deploys with its template's `last_passive` and
/// departs dead after that battle, whatever the outcome (`settle_party`). Err when its fuse is not spent.
pub fn last_stand(d: &Data, gs: &mut GameState, id: &str) -> Result<(), String> {
    let day = gs.time;
    let m = gs.party.member(id).ok_or_else(|| format!("'{id}' is not a party member."))?;
    if !fuse_spent(d, m, day) {
        return Err(format!("{}'s fuse is not spent.", progression::display_name(d, m)));
    }
    refresh_fuses(d, gs);
    if let Some(f) = gs.party.member_mut(id).and_then(|m| m.fuse.as_mut()) {
        f.last_stand = true;
    }
    Ok(())
}

/// FZ2, FZ4: after a battle each fielded member with a fuse counts the deployment and its value is written; a member on
/// its last stand then departs dead (cause `roster`). Returns the departures (member, life).
fn settle_fuses(d: &Data, gs: &mut GameState, f: &FightEnd, ends: &[HeroEnd]) -> Vec<(String, String)> {
    let mut stands = Vec::new();
    for e in ends {
        if let Some(fuse) = gs.party.member_mut(&e.member).and_then(|m| m.fuse.as_mut()) {
            fuse.deploys += 1;
            if fuse.last_stand {
                stands.push(e.member.clone());
            }
        }
    }
    refresh_fuses(d, gs);
    let mut out = Vec::new();
    for id in stands {
        if depart(d, gs, &id, DEAD, ROSTER_CAUSE, &f.encounter).is_ok() {
            out.push((id, DEAD.to_string()));
        }
    }
    out
}

/// BK2 (PK3 3.1): the crypt places in use: members housed `crypt` (Stage IV, who keep their bunk too) or `crypt_only`
/// (Living Martyrs, no bunk).
pub fn crypts_used(party: &PartyState) -> i64 {
    party.members.iter().filter(|m| m.housing() != rpgkit_data::HOUSING[0]).count() as i64
}

/// PR7-37 (PT-03): what the crypt places hold, as a point of interest says it: "Crypt places: 1 of 2 (Name)." or "Crypt
/// places: none of 2 in use."; "" in a world with no crypt places.
pub fn crypt_words(gs: &GameState, name: &dyn Fn(&str) -> String) -> String {
    let cap = crypt_cap(gs);
    if cap <= 0 {
        return String::new();
    }
    let lying: Vec<String> = gs.party.members.iter().filter(|m| m.housing() != rpgkit_data::HOUSING[0]).map(|m| name(&m.id)).collect();
    if lying.is_empty() {
        format!("Crypt places: none of {cap} in use.")
    } else {
        format!("Crypt places: {} of {cap} ({}).", lying.len(), lying.join(", "))
    }
}

/// BK2: the crypt places (`roster.crypts`, 0 = none).
pub fn crypt_cap(gs: &GameState) -> i64 {
    i64::from(gs.party.roster.manifest.crypts.max(0))
}

/// BK2, BK3 (PK3 3.1): why a recruit housed `housing` can't join now ("" = it can): a crypt-only recruit needs a free
/// crypt place, anyone else a free bunk (`full_text`).
pub fn housing_text(gs: &GameState, housing: &str) -> String {
    if housing != CRYPT_ONLY {
        return full_text(gs);
    }
    let (n, cap) = (crypts_used(&gs.party), crypt_cap(gs));
    if n < cap {
        String::new()
    } else {
        format!("Every crypt is taken ({n} of {cap}).")
    }
}

/// PG2: every name a member, a pool entry or a departed member holds (a new recruit's name is none of them).
fn names_taken(d: &Data, gs: &GameState) -> Vec<String> {
    let p = &gs.party;
    p.members.iter().map(|m| progression::display_name(d, m)).chain(p.pool.iter().map(|x| x.name.clone())).chain(p.departed.iter().map(|x| progression::display_name(d, &x.member))).collect()
}

/// PG2: the next procedural id `recruit_<n>`, n = 1 + the highest used by a member, a pool entry or a departed member.
fn next_recruit_id(gs: &GameState) -> String {
    let p = &gs.party;
    let ids = p.members.iter().map(|m| &m.id).chain(p.pool.iter().map(|x| &x.id)).chain(p.departed.iter().map(|x| &x.member.id));
    let top = ids.filter_map(|id| id.strip_prefix(rpgkit_data::RECRUIT_ID_PREFIX)?.parse::<i64>().ok()).max().unwrap_or(0);
    format!("{}{}", rpgkit_data::RECRUIT_ID_PREFIX, top + 1)
}

/// PG2: one draw from `list` on the `recruit` stream ("" for an empty list; no draw for a single choice).
fn draw(gs: &mut GameState, list: &[String]) -> String {
    match list.len() {
        0 => String::new(),
        1 => list[0].clone(),
        n => list[crate::rolls::range_i(gs, crate::rolls::RECRUIT, 0, n as i64 - 1) as usize].clone(),
    }
}

/// PO1, PG2 (PK3 3.1): the `rescue` effect: `count` (1..RESCUE_COUNT_MAX, default 1) pool entries from the `recruits`
/// template `template`, each drawn on the seeded `recruit` stream in order: its base unit (the effect's `base_unit`
/// when set, else one of the template's), a name no member, pool entry or departed member holds, an origin and a
/// temperament; id `recruit_<n>`, rescued today. Or one entry for the named character `character` (refused when it is
/// a member, pooled or departed). Returns the entries added and the notes (a template out of free names adds fewer).
pub fn rescue(d: &Data, gs: &mut GameState, fx: &json::Dict) -> (Vec<String>, Vec<String>) {
    let (mut added, mut notes) = (Vec::new(), Vec::new());
    let base_unit = json::s(fx, "base_unit");
    let character = json::s(fx, "character");
    if !character.is_empty() {
        let c = d.resolve(&gs.world_id, "characters", &character).clone();
        if c.is_empty() {
            notes.push(format!("Unknown character '{character}'."));
            return (added, notes);
        }
        let (id, qid, name) = (json::s(&c, "id"), json::s(&c, "_qid"), json::s_or(&c, "name", &json::s(&c, "id")));
        if crate::dialogue::is_member(d, gs, &character) || gs.party.pool.iter().any(|p| p.id == id || p.character == qid) {
            notes.push(format!("{name} is already with you."));
        } else if has_departed(d, gs, &character) {
            notes.push(gone_text(&name));
        } else {
            let base = if base_unit.is_empty() { json::s(&c, "base_unit") } else { base_unit };
            gs.party.pool.push(crate::state::PoolMember { id: id.clone(), character: qid, base_unit: base, name: name.clone(), since: gs.time, ..Default::default() });
            notes.push(format!("{name} waits to join."));
            added.push(id);
        }
        return (added, notes);
    }
    let Some((qid, t)) = template(d, gs, &json::s(fx, "template")) else {
        notes.push(format!("Unknown recruit template '{}'.", json::s(fx, "template")));
        return (added, notes);
    };
    let count = json::i(fx, "count", 1).clamp(1, rpgkit_data::RESCUE_COUNT_MAX);
    for _ in 0..count {
        let base = if base_unit.is_empty() { draw(gs, &t.base_units) } else { base_unit.clone() };
        let taken = names_taken(d, gs);
        let free: Vec<String> = t.names.iter().filter(|n| !taken.contains(n)).cloned().collect();
        if base.is_empty() || free.is_empty() {
            notes.push(format!("{}: no free name is left for another recruit.", t.name));
            break;
        }
        let name = draw(gs, &free);
        let origin = draw(gs, &t.origins);
        let temperament = draw(gs, &t.temperaments);
        let id = next_recruit_id(gs);
        gs.party.pool.push(crate::state::PoolMember { id: id.clone(), character: String::new(), template: qid.clone(), base_unit: base, name: name.clone(), origin, temperament, since: gs.time });
        notes.push(format!("{name} waits to join."));
        added.push(id);
    }
    (added, notes)
}

/// PO2, PO3 (PK3 3.1): swears pool entry `pool` (an id or `first`) into the party at `level` (BK5; Null = the
/// template's `level`, else `highest`). Refused while the roster is closed (`CLOSED_TEXT`) or with no home for it
/// (`housing_text`: a free bunk, or a free crypt place for a fused template's crypt-only Martyr). A template entry
/// becomes a member with its name, origin, temperament and template; a fused template's member starts its fuse today
/// (FZ1). BK4: it deploys when `active_ids` is empty or has room. Returns the member id.
pub fn swear_in(d: &Data, gs: &mut GameState, pool: &str, level: &Value) -> Result<String, String> {
    if !is_open(gs) {
        return Err(CLOSED_TEXT.into());
    }
    let at = if pool == rpgkit_data::POOL_FIRST { (!gs.party.pool.is_empty()).then_some(0) } else { gs.party.pool.iter().position(|p| p.id == pool) };
    let Some(at) = at else {
        return Err(if pool == rpgkit_data::POOL_FIRST { "No one waits to join.".into() } else { format!("'{pool}' is not waiting to join.") });
    };
    let entry = gs.party.pool[at].clone();
    let tpl = if entry.template.is_empty() { None } else { template(d, gs, &entry.template) };
    let mut m = if entry.character.is_empty() {
        let unit_level = json::i(d.entry("units", &entry.base_unit), "level", 1);
        let mut m = UnitState::at_level(d, &entry.base_unit, unit_level, &entry.id);
        m.name = entry.name.clone();
        m.origin = entry.origin.clone();
        m.temperament = entry.temperament.clone();
        m.template = entry.template.clone();
        m
    } else {
        let mut m = crate::new_game::hero_from_character(d, &gs.world_id, &entry.character).ok_or_else(|| format!("{} can't join the party.", entry.name))?;
        if !entry.base_unit.is_empty() && entry.base_unit != m.base_unit {
            let unit_level = json::i(d.entry("units", &entry.base_unit), "level", 1);
            m.base_unit = entry.base_unit.clone();
            m.total_xp = progression::xp_for_level(d, unit_level);
        }
        m
    };
    if tpl.as_ref().is_some_and(|(_, t)| t.fuse.is_some()) {
        m.housing = CRYPT_ONLY.into();
        m.fuse = Some(crate::state::MartyrFuse { since: gs.time, ..Default::default() });
    }
    let why = housing_text(gs, m.housing());
    if !why.is_empty() {
        return Err(why);
    }
    let level = if level.is_null() { tpl.as_ref().map_or(Value::Null, |(_, t)| t.level.clone()) } else { level.clone() };
    m.total_xp = join_xp(d, &gs.party, m.total_xp, &level);
    let id = m.id.clone();
    if !gs.party.active_ids.is_empty() && (gs.party.active_ids.len() as i64) < gs.party.max_active {
        gs.party.active_ids.push(id.clone());
    }
    gs.party.pool.remove(at);
    gs.party.members.push(m);
    gs.party.sync_keepsakes();
    Ok(id)
}

/// PO3 (PK3 3.1): member `id` retires (departs `retired`, cause `roster`; gear to the stash). Refused while the roster
/// is closed. FZ4: a Martyr with a spent fuse retires this way.
pub fn retire(d: &Data, gs: &mut GameState, id: &str) -> Result<Departed, String> {
    leave(d, gs, id, RETIRED)
}

/// PO3 (PK3 3.1): member `id` is dismissed (departs `dismissed`, cause `roster`; gear to the stash). Refused while the
/// roster is closed.
pub fn dismiss(d: &Data, gs: &mut GameState, id: &str) -> Result<Departed, String> {
    leave(d, gs, id, DISMISSED)
}

fn leave(d: &Data, gs: &mut GameState, id: &str, life: &str) -> Result<Departed, String> {
    if !is_open(gs) {
        return Err(CLOSED_TEXT.into());
    }
    depart(d, gs, id, life, ROSTER_CAUSE, "")
}

/// KS3 (PK3 4): item `id` as party `party` reads it: a catalog item, else one of its keepsake instances (its template
/// row renamed, a wear-legal accessory of value 0, special, never sold).
pub fn item_def(d: &Data, party: &PartyState, id: &str) -> Option<rpgkit_data::ItemDef> {
    if let Some(def) = d.cat.items.get(id) {
        return Some(def.clone());
    }
    party.keepsakes.iter().find(|k| k.id == id).and_then(|k| k.item_def(d))
}

/// KS3 (PK3 4): a procedural member's keepsake instance on its departure: its class's template item
/// (`keepsakes.by_class`), named by `keepsakes.name`, id `keepsake@<member>`; None for a member with no template, a
/// class with no keepsake template or an instance already minted.
fn mint_keepsake(d: &Data, gs: &GameState, m: &UnitState) -> Option<crate::state::Keepsake> {
    if m.template.is_empty() {
        return None;
    }
    let roster = manifest(d, &gs.world_id);
    let item = roster.keepsakes_by_class.get(&m.class_id(d)).filter(|i| d.has("items", i))?;
    let id = format!("{}{}", rpgkit_data::KEEPSAKE_INSTANCE_PREFIX, m.id);
    if gs.party.keepsakes.iter().any(|k| k.id == id) {
        return None;
    }
    let name = roster.keepsake_display(&progression::display_name(d, m), &d.item_name(item));
    Some(crate::state::Keepsake { id, item: item.clone(), name })
}
