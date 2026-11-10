//! BattleBridge (scripts/exploration/battle_bridge.gd): the shared way out of a battle into the game state. REWARD1
//! (rewards per defeated enemy): on any outcome every enemy killed, surrendered or fled on its own pays its XP, its
//! share of the encounter purse and a roll on its unit type's loot table; a win also runs the quest hook and pays key
//! loot (its unit killed) and the objective reward. The runners apply the defeat and flee rules. Building the battle
//! itself (GameEncounter) lives with the session.

use crate::campaign::purse as purse_mods;
use crate::data::Data;
use crate::state::{GameState, UnitState};
use crate::{progression, purse, quest, rolls, roster};
use aegis_core::json::{self, int, Dict};
use rpgkit_battle::encounter::{PartyFate, StayBehind};
use rpgkit_battle::session::{EncounterDef, Fate, Prebuilt, Session, StartGrant};
use serde_json::Value;

pub const OUTCOMES: [&str; 3] = ["won", "lost", "fled"];

/// GameEncounter / WorldEncounter: the battle of encounter row `entry` resolved in `world`, with the ruleset of that
/// world (RULESET1 3.1.2: a battle plays by the world that owns the encounter; ENCOUNTER2's linked rooms too).
pub fn encounter_def(d: &Data, world: &str, entry: &Dict) -> EncounterDef {
    EncounterDef::from_obj_with(entry, d.ruleset(world))
}

/// RULESET1 V3: the plan reveal flag of `world`: the nearest non-empty `plan_reveal_flag` of its lineage (the world,
/// then its parents), "" when none names one.
pub fn plan_reveal_flag(d: &Data, world: &str) -> String {
    d.lineage(world).iter().rev().map(|lw| json::s(d.manifest(lw), "plan_reveal_flag")).find(|f| !f.is_empty()).unwrap_or_default()
}

/// RULESET1 V3: whether a battle in `world` reveals every enemy's plan: the world names a plan reveal flag and the game
/// state holds it (a flag condition with its default value true). Read by the battle only under `plan_badges`.
pub fn plan_reveal(d: &Data, gs: &GameState, world: &str) -> bool {
    let flag = plan_reveal_flag(d, world);
    !flag.is_empty() && quest::condition_met(gs, json::dict(&serde_json::json!({"key": flag})))
}

/// `encounter_def` with the game state's reveal (V3): the battle the world runner starts.
pub fn encounter_def_in(d: &Data, gs: &GameState, world: &str, entry: &Dict) -> EncounterDef {
    let mut def = encounter_def(d, world, entry);
    def.plan_reveal = plan_reveal(d, gs, world);
    def
}

/// WorldEncounter's roster: refs are world characters resolved in `world`; each spawns as its base unit wearing the
/// character (WorldFactory.unit_entry) and then gets its named items, passives and tuning (`character_overrides`, in
/// the one builder).
pub struct WorldRoster<'a> {
    pub d: &'a Data,
    pub world: String,
    /// W.2-S3 (FB-13): the world stage the fight is played at (`world_stage`: the campaign's act, 0 without a
    /// campaign). `new` sets 0 (the host, the sim, previews); `begin_world` sets the game state's stage. A roster
    /// built with a stage above 0 must also set `allies`, or the party's guests would scale as enemies.
    pub stage: i64,
    /// W.2-S3 (FB-139): the refs fighting on the party's side (the encounter's `party` and `guests`), which never scale
    /// with the stage. `new` sets none; `begin_world` sets the encounter's.
    pub allies: Vec<String>,
}

impl<'a> WorldRoster<'a> {
    /// W.2-S1 (FB-50a): the roster of a battle played in `world` at stage 0 (the one constructor; `begin_world` builds
    /// on it and sets the stage and the allies).
    pub fn new(d: &'a Data, world: impl Into<String>) -> Self {
        Self { d, world: world.into(), stage: 0, allies: Vec::new() }
    }
}

/// W.2-S3 (FB-13): the world stage of a fight: the campaign's act in a campaign world, 0 in a world with no campaign
/// (and in the host and the sim, which have no game state: `WorldRoster::new`). Derived, never saved.
pub fn world_stage(gs: &GameState) -> i64 {
    gs.campaign.as_ref().map_or(0, |c| c.act)
}

/// W.2-S3 (FB-13, FB-139): the level a foe of base level `level` and growth `growth` (its unit's `growth_per_stage`)
/// fights at on world stage `stage`: `level + stage * growth` (E_ECONOMY), never above the top unit level
/// (`rpgkit_rules::stats::MAX_LEVEL`, or the base level when that is higher) and never below the base level. The
/// battle (`WorldRoster`'s `entry`) and the XP weight (`roster_at`, `progression::kill_weight`) both read it, so the XP
/// follows the level fought. Stage 0 or growth 0 returns the base level.
pub fn foe_level(level: i64, stage: i64, growth: i64) -> i64 {
    let top = i64::from(rpgkit_rules::stats::MAX_LEVEL).max(level);
    (level + stage.max(0) * growth.max(0)).min(top)
}

/// The level roster unit row `entry` (a character's unit entry, `Data::character_unit_entry`) fights at on stage
/// `stage`: `foe_level` for an enemy (tier other than `hero`), the base level for a hero. A row without `tier` reads
/// the schema default `hero` (`rpgkit_data::unit_tier`, AUDIT-01), so it never scales. Public for the sim's `stage=`
/// (R3-STAGE-SIM), which levels its world fights by the same rule.
pub fn stage_level(entry: &Dict, stage: i64) -> i64 {
    let level = json::i(entry, "level", 1);
    if rpgkit_data::unit_tier(entry) == rpgkit_data::DEFAULT_TIER {
        return level;
    }
    foe_level(level, stage, json::i(entry, "growth_per_stage", 0))
}

impl rpgkit_battle::session::Roster for WorldRoster<'_> {
    /// The character's unit entry; an enemy (not an ally, not a hero) at its stage level (`stage_level`, FB-139), an
    /// explicit `spend` topped up by the raise's extra points (`rpgkit_rules::unit::raise_level`, AUDIT-02).
    fn entry(&self, cat: &rpgkit_data::Catalog, r: &str) -> Option<rpgkit_data::UnitDef> {
        let e = self.d.character_unit_entry(self.d.resolve(&self.world, "characters", r));
        if e.is_empty() {
            return None;
        }
        let mut def = rpgkit_data::UnitDef::from_obj(&e);
        if !self.allies.iter().any(|a| a == r) {
            rpgkit_rules::unit::raise_level(cat, &mut def, stage_level(&e, self.stage) as i32);
        }
        Some(def)
    }

    /// U2.2-S2 (FB-12): the character's overrides (`character_overrides`), built from the world's catalog (HOOKS1 CN3:
    /// the class path of the world playing the battle; the battle's catalog when the world replaces no path).
    fn overrides(&self, _cat: &rpgkit_data::Catalog, r: &str) -> rpgkit_rules::Overrides {
        let mut over = character_overrides(self.d, self.d.resolve(&self.world, "characters", r), &self.world);
        over.path_cat = Some(self.d.cat_in(&self.world).clone());
        over
    }

    fn tier(&self, _cat: &rpgkit_data::Catalog, r: &str) -> String {
        let e = self.d.character_unit_entry(self.d.resolve(&self.world, "characters", r));
        rpgkit_data::unit_tier(&e)
    }

    /// WorldFactory: lore_ref = the character's qualified id, else its id.
    fn lore_ref(&self, _cat: &rpgkit_data::Catalog, r: &str) -> String {
        let c = self.d.resolve(&self.world, "characters", r);
        json::s_or(c, "_qid", &json::s(c, "id"))
    }
}

/// WorldFactory.named_item: the base item with the named item's name, story, tuning deltas and extra stat bonuses.
pub fn named_item(d: &Data, item_entry: &Dict) -> Option<rpgkit_data::ItemDef> {
    let mut base = d.entry("items", &json::s(item_entry, "base_item")).clone();
    if base.is_empty() {
        return None;
    }
    let tuning = json::dget(item_entry, "tuning");
    let name = json::s_or(item_entry, "name", &json::s(&base, "name"));
    base.insert("name".into(), Value::from(name));
    let story = json::s(item_entry, "description");
    if !story.is_empty() {
        base.insert("description".into(), Value::from(story));
    }
    for (field, key) in [("base_power", "power"), ("mitigation", "mitigation"), ("burden", "burden")] {
        let v = (json::i(&base, field, 0) + json::i(tuning, key, 0)).max(0);
        base.insert(field.into(), Value::from(v));
    }
    let mut bonuses = json::dget(&base, "stat_bonuses").clone();
    for (stat, v) in json::dget(item_entry, "stat_bonuses") {
        let sum = bonuses.get(stat).map(json::to_int).unwrap_or(0) + json::to_int(v);
        bonuses.insert(stat.clone(), Value::from(sum));
    }
    base.insert("stat_bonuses".into(), Value::Object(bonuses));
    let qid = json::s_or(item_entry, "_qid", &json::s(item_entry, "id")).replace(':', "_");
    let id = format!("{}__{}", json::s_or(&base, "id", "item"), qid);
    base.insert("id".into(), Value::from(id));
    Some(rpgkit_data::ItemDef::from_obj(&base))
}

/// WorldFactory.apply_character as data (U2.2-S2, FB-12): what world character `character` of `world` wears over its
/// base unit in the one builder (`rpgkit_rules::build_from`): its gathered passives (`Data::character_passives`, FB-28's
/// order), its named items and its tuning's five keys.
pub fn character_overrides(d: &Data, character: &Dict, world: &str) -> rpgkit_rules::Overrides {
    let named_items = json::aget(character, "named_items")
        .iter()
        .map(|r| d.resolve(world, "items", &json::gstr(r)))
        .filter(|e| !e.is_empty())
        .filter_map(|e| named_item(d, e))
        .collect();
    let tuning = json::dget(character, "tuning");
    let key = |k: &str| json::i(tuning, k, 0) as i32;
    let tuning = rpgkit_rules::Tuning { hp_pct: key("hp_pct"), power: key("power"), armor: key("armor"), initiative: key("speed"), ap: key("ap") };
    rpgkit_rules::Overrides { passives: d.character_passives(character, world), named_items, tuning, path_cat: None }
}

/// One enemy of an encounter's roster (REWARD1 2.1): its key `ref#n` (n counts that ref's earlier roster entries),
/// its ref, its base unit's id, level and tier, and its weight in the purse (`progression::kill_weight`).
#[derive(Debug, Clone, PartialEq)]
pub struct RosterFoe {
    pub key: String,
    pub unit_ref: String,
    /// The character's name.
    pub name: String,
    pub unit: String,
    pub level: i64,
    pub tier: String,
    pub weight: i64,
}

/// The encounter's roster seen from `world_id`: `enemies`, then the units of every phase's `wave` effects (phase
/// order, effect order; the arrival, the reinforcements, comes first: PHASEFOLD 3), keeping only refs that resolve to
/// a unit (the refs the battle spawns). At stage 0 (`roster_at`).
pub fn roster(d: &Data, world_id: &str, encounter_qid: &str) -> Vec<RosterFoe> {
    roster_at(d, world_id, encounter_qid, 0)
}

/// `roster` on world stage `stage` (W.2-S3, FB-13, FB-139): each foe's level is the one the battle builds it at
/// (`foe_level` of its base level and its unit's `growth_per_stage`; a hero's base level), and its purse weight follows
/// that level. `apply` reads it at the game state's `world_stage`.
pub fn roster_at(d: &Data, world_id: &str, encounter_qid: &str, stage: i64) -> Vec<RosterFoe> {
    let enc = d.resolve(world_id, "encounters", encounter_qid);
    let mut refs = json::sget(enc, "enemies");
    for phase in json::aget(enc, "phases") {
        for fx in json::aget(json::dict(phase), "effects") {
            let fx = json::dict(fx);
            if json::s(fx, "type") == "wave" {
                refs.extend(json::sget(fx, "units"));
            }
        }
    }
    let mut out: Vec<RosterFoe> = Vec::new();
    for r in refs {
        let entry = d.character_unit_entry(d.resolve(world_id, "characters", &r));
        if entry.is_empty() {
            continue;
        }
        let n = out.iter().filter(|f| f.unit_ref == r).count();
        let (level, tier) = (stage_level(&entry, stage), rpgkit_data::unit_tier(&entry));
        let weight = progression::kill_weight(d, level, &tier);
        out.push(RosterFoe { key: format!("{r}#{n}"), name: json::s(&entry, "name"), unit: json::s(&entry, "id"), unit_ref: r, level, tier, weight });
    }
    out
}

/// (level, tier) of the encounter's enemies, reinforcements and phase waves, seen from `world_id` (the roster).
pub fn enemy_levels(d: &Data, world_id: &str, encounter_qid: &str) -> Vec<(i64, String)> {
    enemy_levels_at(d, world_id, encounter_qid, 0)
}

/// `enemy_levels` on world stage `stage` (`roster_at`).
pub fn enemy_levels_at(d: &Data, world_id: &str, encounter_qid: &str, stage: i64) -> Vec<(i64, String)> {
    roster_at(d, world_id, encounter_qid, stage).into_iter().map(|f| (f.level, f.tier)).collect()
}

/// One roster enemy's fate in a fight (REWARD1 2.4).
#[derive(Debug, Clone, PartialEq)]
pub struct Foe {
    pub key: String,
    pub unit_ref: String,
    pub fate: Fate,
}

/// How a fight left each roster enemy and, for a world battle, each fielded member. Built from a battle
/// (`from_session`: the foes; `of_world_fight`: the foes and the party), or `None` in `after_fight` for a fight
/// settled without one (the default tally).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Tally {
    pub foes: Vec<Foe>,
    /// ENCOUNTER2 W1: the fielded members' ends with their S3 fates (member id, fate, HP, pouch); `apply` settles them
    /// (`roster::settle_party`) before the rewards. Empty = no party side (the party is left as it is).
    pub party: Vec<roster::HeroEnd>,
    /// The world whose encounter was fought (`roster::field_unit`'s world; "" = `fight_world`).
    pub world: String,
}

impl Tally {
    /// The session's enemies keyed `ref#n` in spawn order (REWARD1 2.1).
    pub fn from_session(s: &Session) -> Self {
        let mut foes: Vec<Foe> = Vec::new();
        for (r, fate) in s.fates() {
            let n = foes.iter().filter(|f| f.unit_ref == r).count();
            foes.push(Foe { key: format!("{r}#{n}"), unit_ref: r, fate });
        }
        Tally { foes, ..Default::default() }
    }

    /// A world battle's tally: the foes (`from_session`) and the party side (`roster::hero_ends` of the fielded
    /// `members`: member id and unloaded pouch ids, in the order they were fielded), fought in world `world`. Every
    /// screen or headless runner that plays a world fight passes this to its runner's `after_fight`.
    pub fn of_world_fight(s: &Session, members: &[(String, Vec<String>)], world: &str) -> Self {
        Tally { party: roster::hero_ends(s, members), world: world.to_string(), ..Self::from_session(s) }
    }

    /// REWARD1 2.5, no battle: won = every roster enemy killed; lost and fled = every one standing.
    pub fn default_for(d: &Data, world: &str, encounter_qid: &str, outcome: &str) -> Self {
        let fate = if outcome == "won" { Fate::Killed } else { Fate::Standing };
        Tally { foes: roster(d, world, encounter_qid).into_iter().map(|f| Foe { key: f.key, unit_ref: f.unit_ref, fate }).collect(), ..Default::default() }
    }

    /// Each roster enemy's fate (a key missing from the tally: standing), and a note per tally foe not in the roster.
    pub fn fates_of(&self, roster: &[RosterFoe]) -> (Vec<Fate>, Vec<String>) {
        let fates = roster.iter().map(|r| self.foes.iter().find(|f| f.key == r.key).map_or(Fate::Standing, |f| f.fate)).collect();
        let notes = self.foes.iter().filter(|f| !roster.iter().any(|r| r.key == f.key)).map(|f| format!("Tally foe '{}' is not in the encounter.", f.key)).collect();
        (fates, notes)
    }
}

/// The loot table row a draw `r` (0..99) lands on: rows in order, each taking `chance` of the hundred; past every
/// row nothing drops.
pub fn loot_row(rows: &[Value], mut r: i64) -> Option<&Dict> {
    for row in rows.iter().filter_map(Value::as_object) {
        let chance = json::i(row, "chance", 0).max(0);
        if r < chance {
            return Some(row);
        }
        r -= chance;
    }
    None
}

/// Grants one world loot entry and claims it: "" or why it was not.
fn grant_loot(d: &Data, gs: &mut GameState, lqid: &str) -> String {
    let entry = d.resolve(&gs.world_id, "loot", lqid).clone();
    let granted = quest::reward_apply(d, gs, &Dict::new(), "", std::slice::from_ref(&entry));
    if !json::b(&granted, "ok", false) {
        return json::sget(&granted, "errors").join("; ");
    }
    quest::claim_loot(gs, lqid)
}

/// The world loot entry `r` as an unclaimed item reward whose flags hold: its qid, else "" (claimed or gated: skipped).
fn claimable(d: &Data, gs: &GameState, world: &str, r: &str) -> String {
    let loot = d.resolve(world, "loot", r);
    if r.is_empty() || loot.is_empty() || json::s_or(loot, "reward_kind", "item") != "item" {
        return String::new();
    }
    let lqid = json::s(loot, "_qid");
    if gs.loot_claimed.contains(&lqid) || !quest::all_hold(gs, json::get(loot, "requires_flags")) {
        return String::new();
    }
    lqid
}

/// REWARD1 section 6: {ok, errors, xp: member id -> award, advanced, rewards}. `rewards` = {foes, defeated: {slain,
/// surrendered, fled}, gold (the purse share), drops: [{item, count, from}], key_loot: [loot qid], key_missed:
/// [{loot, kill}], objective: {gold, loot}, notes}; `gold` = share + objective gold, paid in one go. First the party
/// side (`settle_party_side`: the tally's fielded members settled by `roster::settle_party`, the stay-behind flag) and
/// its keys `party_fates`, `stay_lines`, `wounded`, `departed` (`PARTY_KEYS`).
pub fn apply(d: &Data, gs: &mut GameState, encounter_qid: &str, outcome: &str, tally: &Tally) -> Dict {
    let done = |ok: bool, errors: &[String], xp: Dict, advanced: &[String]| {
        let mut out = json::ok_result(ok, errors);
        out.insert("xp".into(), Value::Object(xp));
        out.insert("advanced".into(), json::strs(advanced));
        out
    };
    if !OUTCOMES.contains(&outcome) {
        return done(false, &[format!("Unknown battle outcome '{outcome}' (use won, lost or fled).")], Dict::new(), &[]);
    }
    let won = outcome == "won";
    let world = gs.world_id.clone();
    let enc = d.resolve(&world, "encounters", encounter_qid).clone();
    let enc_world = json::s_or(&enc, "_world", &world);
    // 1. the party side first (ROSTER1 3.2 and L5, ENCOUNTER2 W1-W4): before the XP, the quest hook and the tick
    let (party_keys, party_notes) = settle_party_side(d, gs, &enc, encounter_qid, outcome, tally);
    let roster = roster_at(d, &world, encounter_qid, world_stage(gs));
    let (fates, mut notes) = tally.fates_of(&roster);
    notes.extend(party_notes);
    let defeated: Vec<bool> = fates.iter().map(|f| f.defeated()).collect();
    // 2. XP for the defeated enemies only (none: no award at all). A won fight whose roster does not resolve from the
    // party's world (a road fight on another world's route) gives every member a 0 award, as before REWARD1 (Godot).
    let levels: Vec<(i64, String)> = roster.iter().zip(&defeated).filter(|(_, d)| **d).map(|(f, _)| (f.level, f.tier.clone())).collect();
    let xp = if levels.is_empty() && !(won && roster.is_empty()) { Dict::new() } else { progression::award_encounter(d, &mut gs.party, &levels) };
    // 3. the quest hook
    let advanced = if won { quest::on_encounter_won(d, gs, encounter_qid) } else { Vec::new() };
    // 4. the purse share
    let share = purse::share(gs, &enc, &roster, &defeated);
    // 5. loot-table rolls, roster order
    let mut drops: Vec<Value> = Vec::new();
    for (f, _) in roster.iter().zip(&defeated).filter(|(_, d)| **d) {
        let rows = json::aget(d.entry("units", &f.unit), "loot").to_vec();
        if rows.is_empty() {
            continue;
        }
        let r = rolls::range_i(gs, rolls::LOOT, 0, 99);
        if let Some(row) = loot_row(&rows, r) {
            let (item, count) = (json::s(row, "item"), json::i(row, "count", 1).max(1));
            gs.party.add_to_stash(&item, count);
            drops.push(json::obj([("item", item.into()), ("count", int(count)), ("from", f.name.clone().into())]).into());
        }
    }
    // 6. key loot: a win with every unit of its ref killed
    let (mut key_loot, mut key_missed): (Vec<String>, Vec<Value>) = (Vec::new(), Vec::new());
    for row in json::aget(&enc, "key_loot").iter().filter_map(Value::as_object) {
        let (loot, kill) = (json::s(row, "loot"), json::s(row, "kill"));
        let lqid = claimable(d, gs, &enc_world, &loot);
        if lqid.is_empty() {
            continue;
        }
        let of_ref: Vec<Fate> = roster.iter().zip(&fates).filter(|(f, _)| f.unit_ref == kill).map(|(_, x)| *x).collect();
        if won && !of_ref.is_empty() && of_ref.iter().all(|x| *x == Fate::Killed) {
            let why = grant_loot(d, gs, &lqid);
            if why.is_empty() {
                key_loot.push(lqid);
            } else {
                notes.push(format!("Key loot '{lqid}' skipped: {why}"));
            }
        } else {
            key_missed.push(json::obj([("loot", lqid.into()), ("kill", kill.into())]).into());
        }
    }
    // 7. the objective reward
    let qid = json::s(&enc, "_qid");
    let mut objective = Dict::new();
    let reward = json::dget(&enc, "objective_reward").clone();
    let once = reward.get("once").and_then(Value::as_bool).unwrap_or(true);
    let paid_before = json::aget(&gs.reward_paid, &qid).iter().any(|k| json::gstr(k) == "objective");
    if won && !reward.is_empty() && !(once && paid_before) {
        let gold = json::i(&reward, "gold", 0).clamp(0, purse::GOLD_CAP);
        let lqid = claimable(d, gs, &enc_world, &json::s(&reward, "loot"));
        let mut got = String::new();
        if !lqid.is_empty() {
            let why = grant_loot(d, gs, &lqid);
            if why.is_empty() {
                got = lqid;
            } else {
                notes.push(format!("Objective loot '{lqid}' skipped: {why}"));
            }
        }
        if once {
            let mut keys = json::aget(&gs.reward_paid, &qid).to_vec();
            keys.push("objective".into());
            gs.reward_paid.insert(qid.clone(), Value::Array(keys));
        }
        objective = json::obj([("gold", int(gold)), ("loot", got.into())]);
    }
    // 8. gold: the share and the objective's, in one go
    let gold = share + json::i(&objective, "gold", 0);
    let mut paid_gold = 0;
    if gold > 0 {
        let granted = quest::reward_apply(d, gs, &json::obj([("gold", int(gold))]), "", &[]);
        notes.extend(json::sget(&granted, "notes"));
        notes.extend(json::sget(&granted, "errors"));
        paid_gold = json::i(json::dget(&granted, "granted"), "gold", 0);
    }
    let count = |f: Fate| fates.iter().filter(|x| **x == f).count() as i64;
    let rewards = json::obj([
        ("foes", int(roster.len() as i64)),
        ("defeated", json::obj([("slain", int(count(Fate::Killed))), ("surrendered", int(count(Fate::Surrendered))), ("fled", int(count(Fate::Fled)))]).into()),
        ("gold", int(share)),
        ("drops", Value::Array(drops)),
        ("key_loot", json::strs(&key_loot)),
        ("key_missed", Value::Array(key_missed)),
        ("objective", Value::Object(objective)),
        ("notes", json::strs(&notes)),
    ]);
    let mut out = done(true, &[], xp, &advanced);
    out.insert("rewards".into(), Value::Object(rewards));
    out.extend(party_keys);
    if paid_gold > 0 {
        out.insert("gold".into(), int(paid_gold));
    }
    // CAMPAIGN1 X4: the battle tick (meter inputs by the encounter's tags and the outcome); its events only when there
    // are some (none outside a campaign world, X3)
    let cause = crate::campaign::Cause::Battle { encounter: qid, tags: json::sget(&enc, "tags"), outcome: outcome.to_string() };
    let events = crate::campaign::tick(d, gs, &cause);
    if !events.is_empty() {
        out.insert("campaign".into(), Value::Array(events));
    }
    out
}

/// The world whose encounter `encounter_ref` is fought from the played world: the played world when the encounter
/// resolves within its lineage, else the connected world that holds it (a road fight on another world's route). The
/// battle screen builds the fight there (`roster::field_unit`, the ruleset, `party_min`).
pub fn fight_world(d: &Data, gs: &GameState, encounter_ref: &str) -> String {
    let hit_world = json::s(d.resolve(&gs.world_id, "encounters", encounter_ref), "_world");
    if hit_world.is_empty() || d.lineage(&gs.world_id).contains(&hit_world) {
        gs.world_id.clone()
    } else {
        hit_world
    }
}

/// ENCOUNTER2 3.3 and ROSTER1 3.2, the party side of `apply` (step 1): with `tally.party`, `roster::settle_party` (the
/// stay-behind `then`, the act's rule row, HP, keepsakes) in the fight's world; W2: the encounter's `stay_behind.flag`
/// set to the first member who stayed or stood ("" when none) on every outcome. Returns the result keys
/// (`party_fates`: [[member, fate]] (W1); `stay_lines`: one `result_line` per stayed or stood hero (W4), named before
/// anyone departs; `wounded`: [member]; `departed`: [{member, life}] (ROSTER1-U's lines); `marks`: [{member, mark,
/// value}] (PK1-S1); each only when not empty) and
/// the notes (a flag the world refuses). The encounter (its `stay_behind`, objective and qid) is read as the fight's
/// world sees it (`tally.world`, else `fight_world`), as the battle was built: a road fight on another world's route
/// may not resolve from the played world (`played_enc`, then only the fallback).
fn settle_party_side(d: &Data, gs: &mut GameState, played_enc: &Dict, encounter_qid: &str, outcome: &str, tally: &Tally) -> (Dict, Vec<String>) {
    let (mut out, mut notes) = (Dict::new(), Vec::new());
    let world = if tally.world.is_empty() { fight_world(d, gs, encounter_qid) } else { tally.world.clone() };
    let in_world = d.resolve(&world, "encounters", encounter_qid).clone();
    let enc = if in_world.is_empty() { played_enc } else { &in_world };
    let stay = StayBehind::read(enc);
    let stayers: Vec<&roster::HeroEnd> = tally.party.iter().filter(|e| matches!(e.fate, Some(PartyFate::Stayed | PartyFate::Stood))).collect();
    let lines: Vec<String> = match &stay {
        Some(b) => stayers
            .iter()
            .map(|e| {
                let name = gs.party.member(&e.member).map_or_else(|| e.member.clone(), |m| progression::display_name(d, m));
                b.result_line.replace("{name}", &name)
            })
            .collect(),
        None => Vec::new(),
    };
    let first_stayer = stayers.first().map(|e| e.member.clone()).unwrap_or_default();
    if !tally.party.is_empty() {
        let f = roster::FightEnd {
            outcome: outcome.to_string(),
            encounter: json::s_or(enc, "_qid", encounter_qid),
            world,
            escape: json::s(enc, "objective") == "escape",
            then: stay.as_ref().map(|b| b.then.clone()).unwrap_or_default(),
        };
        let settled = roster::settle_party(d, gs, &f, &tally.party);
        let fates: Vec<Value> = tally.party.iter().filter_map(|e| e.fate.map(|x| Value::Array(vec![e.member.clone().into(), x.name().into()]))).collect();
        if !fates.is_empty() {
            out.insert("party_fates".into(), Value::Array(fates));
        }
        if !settled.wounded.is_empty() {
            out.insert("wounded".into(), json::strs(&settled.wounded));
        }
        // A1.2-K3 (INTENT2-K3): the pouch slots auto-fill left short (the Results' "Stash short" lines)
        if !settled.stash_short.is_empty() {
            out.insert("stash_short".into(), json::strs(&settled.stash_short));
        }
        if !settled.departed.is_empty() {
            let departed = settled.departed.iter().map(|(m, life)| json::obj([("member", m.clone().into()), ("life", life.clone().into())]).into()).collect();
            out.insert("departed".into(), Value::Array(departed));
        }
        // PK1-S1 (PK1_CORRUPTION 3.5): the marks the battle recorded on the members, for the results (PK2 settles them)
        let marks: Vec<Value> = tally
            .party
            .iter()
            .flat_map(|e| e.marks.iter().map(|(m, v)| json::obj([("member", e.member.clone().into()), ("mark", m.clone().into()), ("value", int(*v))]).into()))
            .collect();
        if !marks.is_empty() {
            out.insert("marks".into(), Value::Array(marks));
        }
    }
    if let Some(b) = stay.as_ref().filter(|b| !b.flag.is_empty()) {
        let err = quest::set_flag(d, gs, &b.flag, Value::String(first_stayer));
        if !err.is_empty() {
            notes.push(err);
        }
    }
    if !lines.is_empty() {
        out.insert("stay_lines".into(), json::strs(&lines));
    }
    (out, notes)
}

/// A world fight's start with the saved party (BattleBridge.encounter_for; `begin_world` builds the fight after it):
/// the world that resolves `encounter_ref` (`fight_world`), refused when the fight world's entry's `party_min`
/// is above the active members (ROSTER1 BK6, `roster::party_min_refusal`); then A1.2-K3 (INTENT2-K3) the combat-start
/// auto-fill of the active members' pouches (`roster::auto_fill_at_start`, under the world's `pouch_stacks`). Returns
/// the fight world and the auto-fill's "Stash short" lines.
pub fn begin_fight(d: &Data, gs: &mut GameState, encounter_ref: &str) -> Result<(String, Vec<String>), String> {
    let world = fight_world(d, gs, encounter_ref);
    let local = d.resolve(&gs.world_id, "encounters", encounter_ref);
    let entry = d.resolve(&world, "encounters", &json::s(local, "id"));
    let short = roster::party_min_refusal(gs, entry);
    if !short.is_empty() {
        return Err(short);
    }
    let lines = roster::auto_fill_at_start(d, gs, &world);
    Ok((world, lines))
}

/// A started world fight (W.2-S2, FB-50b, SILOS1 T1): what `begin_world` built for the screen or a headless runner.
pub struct WorldFight {
    /// The battle, at its start (the heroes first, in the order fielded; the start-AP change applied).
    pub session: Session,
    /// The fight's world (`fight_world`): the world whose class paths, ruleset and characters the battle uses; settle
    /// with it (`Tally::of_world_fight`).
    pub world: String,
    /// The fielded members in battle order: member id and the saved pouch ids that found no battle slot.
    pub members: Vec<(String, Vec<String>)>,
    /// The combat-start auto-fill's "Stash short" lines (A1.2-K3).
    pub stash_short: Vec<String>,
    /// CAMPAIGN1 P3, W6: the battle start's mods (read before the session; its note is logged once).
    pub mods: purse_mods::BattleMods,
    /// The encounter's intro line (as the fight's world sees it).
    pub intro: String,
    /// W.2-S3 (FB-13): the world stage the fight is played at (`world_stage`).
    pub stage: i64,
}

impl WorldFight {
    /// The fight's tally from its battle as it now stands (`Tally::of_world_fight`), for `finish_world`.
    pub fn tally(&self) -> Tally {
        Tally::of_world_fight(&self.session, &self.members, &self.world)
    }
}

/// The body art of a fielded member (`Prebuilt::art`, `Prebuilt::palette`) given the fight's world and the member: the
/// screen's callback (art is not the kit's); `begin_world` leaves both "".
pub type MemberArt<'a> = dyn FnMut(&str, &UnitState) -> (String, String) + 'a;

/// `begin_world_with` and no art (a headless runner).
pub fn begin_world(d: &Data, gs: &mut GameState, fight: &Dict) -> Result<WorldFight, String> {
    begin_world_with(d, gs, fight, &mut |_, _| (String::new(), String::new()))
}

/// W.2-S2 (FB-50b, SILOS1 T1): a world fight's start with the saved party (BattleBridge.encounter_for and
/// GameEncounter), the one start the battle screen and headless runners share. Refused when `fight`'s `encounter` does
/// not resolve from the played world or no member is active; then `begin_fight` (the fight world, BK6's `party_min`
/// refusal, the combat-start auto-fill). The battle plays by the fight world's ruleset with the game state's plan
/// reveal (`encounter_def_in`), in the fight's `zone` when it names one, granting each `start_status` entry whose
/// `requires_flags` all hold (PACE1 rule 7). Each active member is fielded by the one builder (`roster::field_unit`:
/// its class path, wounds, AI profile, default intent) wearing the art `art` gives it; the foes come from the world's
/// characters (`WorldRoster` at the game state's `world_stage`); then the start-AP change (`lower_start_ap` with
/// `campaign::purse::battle_mods`, read before the session).
pub fn begin_world_with(d: &Data, gs: &mut GameState, fight: &Dict, art: &mut MemberArt) -> Result<WorldFight, String> {
    let enc_ref = json::s(fight, "encounter");
    let hit = d.resolve(&gs.world_id, "encounters", &enc_ref);
    if hit.is_empty() {
        return Err(format!("Unknown encounter '{enc_ref}'."));
    }
    let hit_id = json::s(hit, "id");
    if gs.party.active_members().is_empty() {
        return Err("No active party member to fight.".into());
    }
    let (world, stash_short) = begin_fight(d, gs, &enc_ref)?;
    let gs: &GameState = gs;
    let entry = d.resolve(&world, "encounters", &hit_id);
    let mut def = encounter_def_in(d, gs, &world, entry);
    let zone = json::s(fight, "zone");
    if !zone.is_empty() {
        def.zone = zone;
    }
    def.id = format!("{world}:{hit_id}");
    let grants: Vec<bool> = def.start_status.iter().map(|sb| json::aget(json::dict(sb), "requires_flags").iter().all(|c| quest::condition_met(gs, json::dict(c)))).collect();
    def.start_grant = StartGrant::Only(grants);
    let stage = world_stage(gs);
    let allies = def.party.iter().chain(&def.guests).cloned().collect();
    let roster = WorldRoster { stage, allies, ..WorldRoster::new(d, world.clone()) };
    let mut party = Vec::new();
    let mut members = Vec::new();
    for m in gs.party.active_members() {
        let Some(field) = roster::field_unit(d, gs, m, &world) else { continue };
        let (art, palette) = art(&world, m);
        party.push(Prebuilt { unit: field.unit, art, palette, profile: field.profile, key: m.id.clone(), default_intent: field.default_intent });
        members.push((m.id.clone(), field.pouch_unloaded));
    }
    let mods = purse_mods::battle_mods(d, gs);
    let heroes = party.len();
    let mut session = Session::start_with(d.cat.clone(), def, &roster, party);
    lower_start_ap(&mut session, heroes, &mods);
    Ok(WorldFight { session, world, members, stash_short, mods, intro: json::s(entry, "intro"), stage })
}

/// CAMPAIGN1 P3: the battle bridge's start-AP change: each of the first `heroes` units (the party members; guests and
/// enemies keep theirs) has `mods.start_ap` AP more in round 1 (never below 0, `campaign::purse::start_ap`), and the
/// round's start snapshot shows it. Nothing when the mods change nothing.
pub fn lower_start_ap(s: &mut Session, heroes: usize, mods: &purse_mods::BattleMods) {
    if mods.start_ap == 0 {
        return;
    }
    for u in s.w.units.iter_mut().take(heroes) {
        u.ap = purse_mods::start_ap(u.ap as i64, mods) as i32;
    }
    let start = s.snapshot();
    if let Some(first) = s.history.last_mut().and_then(|r| r.snapshots.first_mut()) {
        *first = start;
    }
}

/// `apply` with `tally`, or with the default tally when the fight was settled without a battle (REWARD1 2.5). ROSTER1
/// 3.2 on every runner: in a world with a rule row (`roster::rule`) the default tally also carries the party side
/// (`roster::default_ends` in `fight_world`), so a fight resolved without a battle still applies the row; a classic
/// world's default tally has none (the party is left as before).
pub fn settle(d: &Data, gs: &mut GameState, encounter_qid: &str, outcome: &str, tally: Option<&Tally>) -> Dict {
    match tally {
        Some(t) => apply(d, gs, encounter_qid, outcome, t),
        None => {
            let mut t = Tally::default_for(d, &gs.world_id.clone(), encounter_qid, outcome);
            if roster::rule(d, gs).is_some() {
                t.world = fight_world(d, gs, encounter_qid);
                t.party = roster::default_ends(d, gs, &t.world, outcome);
            }
            apply(d, gs, encounter_qid, outcome, &t)
        }
    }
}

/// W.2-S1 (FB-50a): the settle of a world fight by the runner that started it (BattleBridge's after_fight by the
/// fight's source): `travel`, `dialogue`, `story` (STORYUI1 Q5: the story sequence moves on, retries or aborts),
/// `trigger` (CAMPAIGN1 T2: a queued battle, settled like a dialogue fight), else `explore`. Each runner settles the
/// party side first, then the rewards, then its clock tick (`apply`). The battle screen and the debug fight popup both
/// settle through it.
pub fn finish_world(d: &Data, gs: &mut GameState, runner: &str, fight: &Dict, outcome: &str, tally: Option<&Tally>) -> Dict {
    match runner {
        "travel" => crate::travel::after_fight(d, gs, fight, outcome, tally),
        "dialogue" => crate::dialogue::after_fight(d, gs, fight, outcome, tally),
        crate::story::FIGHT_SOURCE => crate::story::after_fight(d, gs, fight, outcome, tally),
        crate::campaign::trigger::FIGHT_SOURCE => crate::campaign::trigger::after_fight(d, gs, fight, outcome, tally),
        _ => crate::explore::after_fight(d, gs, fight, outcome, tally),
    }
}

/// The party side's result keys (`settle_party_side`), carried by every runner's `after_fight`.
/// PK1-S1: `marks` [{member, mark, value}], the battle's marks on the fielded members.
pub const PARTY_KEYS: [&str; 6] = ["party_fates", "stay_lines", "wounded", "departed", "stash_short", "marks"];

/// The keys an `after_fight` result carries from `apply`: `rewards`, `gold` when some was paid, `campaign` (the
/// campaign events, CAMPAIGN1 W1) when there are some, and the party side's `party_fates`, `stay_lines`, `wounded`
/// and `departed` when `apply` set them (ENCOUNTER2 W1, W4; ROSTER1 3.2).
pub fn result_keys(applied: &Dict) -> Dict {
    let mut out = Dict::new();
    if let Some(r) = applied.get("rewards") {
        out.insert("rewards".into(), r.clone());
    }
    for key in PARTY_KEYS {
        if let Some(v) = applied.get(key) {
            out.insert(key.into(), v.clone());
        }
    }
    if json::i(applied, "gold", 0) > 0 {
        out.insert("gold".into(), int(json::i(applied, "gold", 0)));
    }
    if let Some(c) = applied.get("campaign") {
        out.insert("campaign".into(), c.clone());
    }
    out
}
