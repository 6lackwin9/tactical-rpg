//! BattleBridge (scripts/exploration/battle_bridge.gd): the shared way out of a battle into the game state. REWARD1
//! (rewards per defeated enemy): on any outcome every enemy killed, surrendered or fled on its own pays its XP, its
//! share of the encounter purse and a roll on its unit type's loot table; a win also runs the quest hook and pays key
//! loot (its unit killed) and the objective reward. The runners apply the defeat and flee rules. Building the battle
//! itself (GameEncounter) lives with the session.

use crate::data::Data;
use crate::state::GameState;
use crate::{progression, purse, quest, rolls};
use aegis_core::json::{self, int, Dict};
use rpgkit_battle::session::{EncounterDef, Fate, Session};
use serde_json::Value;

pub const OUTCOMES: [&str; 3] = ["won", "lost", "fled"];

/// GameEncounter / WorldEncounter: the battle of encounter row `entry` resolved in `world`, with the ruleset of that
/// world (RULESET1 3.1.2: a battle plays by the world that owns the encounter; ENCOUNTER2's linked rooms too).
pub fn encounter_def(d: &Data, world: &str, entry: &Dict) -> EncounterDef {
    let mut def = EncounterDef::from_obj(entry);
    def.ruleset = d.ruleset(world);
    def
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
/// character (WorldFactory.unit_entry) and then gets its named items, passives and tuning (apply_character).
pub struct WorldRoster<'a> {
    pub d: &'a Data,
    pub world: String,
}

impl rpgkit_battle::session::Roster for WorldRoster<'_> {
    fn entry(&self, _cat: &rpgkit_data::Catalog, r: &str) -> Option<rpgkit_data::UnitDef> {
        let e = self.d.character_unit_entry(self.d.resolve(&self.world, "characters", r));
        (!e.is_empty()).then(|| rpgkit_data::UnitDef::from_obj(&e))
    }

    fn customize(&self, cat: &rpgkit_data::Catalog, r: &str, u: &mut rpgkit_rules::Unit) {
        // HOOKS1 CN3: a class whose path the world's overlay replaces is rebuilt from the world's catalog (same name,
        // deck rule and AP; the AP is kept because the session's default AP is not known here)
        if let Some(mut e) = self.entry(cat, r).filter(|e| json::get(self.d.class_overlay(&self.world, &e.class_id), "skill_path").is_object()) {
            e.name = u.name.clone();
            let (max_ap, ap) = (u.max_ap, u.ap);
            let deck = u.deck_rule.clone();
            *u = rpgkit_rules::build_unit_in(self.d.cat_in(&self.world), &e, 0, deck.as_ref());
            (u.max_ap, u.ap) = (max_ap, ap);
        }
        apply_character(self.d, cat, self.d.resolve(&self.world, "characters", r), &self.world, u);
    }

    fn tier(&self, _cat: &rpgkit_data::Catalog, r: &str) -> String {
        let e = self.d.character_unit_entry(self.d.resolve(&self.world, "characters", r));
        json::s_or(&e, "tier", "standard")
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

/// WorldFactory.apply_character: named items worn, lore passives applied, vitals re-derived when geared (derived
/// units only), then the tuning deltas.
pub fn apply_character(d: &Data, cat: &rpgkit_data::Catalog, character: &Dict, world: &str, u: &mut rpgkit_rules::Unit) {
    let mut geared = false;
    for r in json::aget(character, "named_items") {
        let item_entry = d.resolve(world, "items", &json::gstr(r));
        if item_entry.is_empty() {
            continue;
        }
        if let Some(item) = named_item(d, item_entry).filter(|i| i.is_equipment()) {
            let slot = rpgkit_rules::gear::Slot::parse(&item.slot).unwrap_or(rpgkit_rules::gear::Slot::MainHand);
            u.inventory.equip(rpgkit_rules::gear::Equipment::new(&item), slot);
            geared = true;
        }
    }
    for id in d.character_passives(character, world) {
        if let Some(p) = cat.passives.get(&id) {
            u.apply_passive(cat, p);
        }
    }
    let base_unit = d.entry("units", &json::s(character, "base_unit"));
    if geared && json::s_or(base_unit, "stat_mode", "fixed") == "derived" {
        u.sync_from_stats_and_gear(cat);
        u.hp = u.max_hp;
    }
    let tuning = json::dget(character, "tuning");
    let hp_pct = json::i(tuning, "hp_pct", 0);
    if hp_pct != 0 {
        u.max_hp = (json::roundi(u.max_hp as f64 * (1.0 + hp_pct as f64 / 100.0)) as i32).max(1);
        u.hp = u.max_hp;
    }
    let power = json::i(tuning, "power", 0) as i32;
    if power != 0 {
        u.power_override = (rpgkit_rules::damage::power_of(u, u.attack_range > 1, false) + power).max(1);
    }
    let armor = json::i(tuning, "armor", 0) as i32;
    if armor != 0 {
        u.armor_override = (rpgkit_rules::damage::armor_rating(u) + armor).max(0);
    }
    let speed = json::i(tuning, "speed", 0) as i32;
    if speed != 0 {
        u.speed = (u.speed + speed).max(1);
    }
    let ap = json::i(tuning, "ap", 0) as i32;
    if ap != 0 {
        u.max_ap = (u.max_ap + ap).max(1);
        u.ap = u.max_ap;
    }
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
/// a unit (the refs the battle spawns).
pub fn roster(d: &Data, world_id: &str, encounter_qid: &str) -> Vec<RosterFoe> {
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
        let (level, tier) = (json::i(&entry, "level", 1), json::s_or(&entry, "tier", "standard"));
        let weight = progression::kill_weight(d, level, &tier);
        out.push(RosterFoe { key: format!("{r}#{n}"), name: json::s(&entry, "name"), unit: json::s(&entry, "id"), unit_ref: r, level, tier, weight });
    }
    out
}

/// (level, tier) of the encounter's enemies, reinforcements and phase waves, seen from `world_id` (the roster).
pub fn enemy_levels(d: &Data, world_id: &str, encounter_qid: &str) -> Vec<(i64, String)> {
    roster(d, world_id, encounter_qid).into_iter().map(|f| (f.level, f.tier)).collect()
}

/// One roster enemy's fate in a fight (REWARD1 2.4).
#[derive(Debug, Clone, PartialEq)]
pub struct Foe {
    pub key: String,
    pub unit_ref: String,
    pub fate: Fate,
}

/// How a fight left each roster enemy. Built from a battle (`from_session`), or `None` in `after_fight` for a fight
/// settled without one (the default tally).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Tally {
    pub foes: Vec<Foe>,
}

impl Tally {
    /// The session's enemies keyed `ref#n` in spawn order (REWARD1 2.1).
    pub fn from_session(s: &Session) -> Self {
        let mut foes: Vec<Foe> = Vec::new();
        for (r, fate) in s.fates() {
            let n = foes.iter().filter(|f| f.unit_ref == r).count();
            foes.push(Foe { key: format!("{r}#{n}"), unit_ref: r, fate });
        }
        Tally { foes }
    }

    /// REWARD1 2.5, no battle: won = every roster enemy killed; lost and fled = every one standing.
    pub fn default_for(d: &Data, world: &str, encounter_qid: &str, outcome: &str) -> Self {
        let fate = if outcome == "won" { Fate::Killed } else { Fate::Standing };
        Tally { foes: roster(d, world, encounter_qid).into_iter().map(|f| Foe { key: f.key, unit_ref: f.unit_ref, fate }).collect() }
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
/// [{loot, kill}], objective: {gold, loot}, notes}; `gold` = share + objective gold, paid in one go.
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
    let roster = roster(d, &world, encounter_qid);
    let (fates, mut notes) = tally.fates_of(&roster);
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

/// `apply` with `tally`, or with the default tally when the fight was settled without a battle (REWARD1 2.5).
pub fn settle(d: &Data, gs: &mut GameState, encounter_qid: &str, outcome: &str, tally: Option<&Tally>) -> Dict {
    match tally {
        Some(t) => apply(d, gs, encounter_qid, outcome, t),
        None => {
            let t = Tally::default_for(d, &gs.world_id.clone(), encounter_qid, outcome);
            apply(d, gs, encounter_qid, outcome, &t)
        }
    }
}

/// The keys an `after_fight` result carries from `apply`: `rewards`, `gold` when some was paid, and `campaign` (the
/// campaign events, CAMPAIGN1 W1) when there are some.
pub fn result_keys(applied: &Dict) -> Dict {
    let mut out = Dict::new();
    if let Some(r) = applied.get("rewards") {
        out.insert("rewards".into(), r.clone());
    }
    if json::i(applied, "gold", 0) > 0 {
        out.insert("gold".into(), int(json::i(applied, "gold", 0)));
    }
    if let Some(c) = applied.get("campaign") {
        out.insert("campaign".into(), c.clone());
    }
    out
}
