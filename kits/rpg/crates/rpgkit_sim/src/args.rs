//! Balance sim arguments (tools/sim/sim_args.gd, BAL2_SIM.md section 1): positional mode, level, seeds, then
//! key=value tokens in any order.

use crate::targets::{self, Targets};
use crate::SimGame;
use rpgkit_world::Data;
use indexmap::IndexMap;

/// RULESET1-G: `feel` has no Godot original; nor has CAMPAIGN1-SIM's `deadlines` (deadlines.rs, no fights). Both are in
/// the help text since CLN-39 (it was Godot parity before).
pub const MODES: [&str; 11] = ["duel", "tier", "levels", "group", "stats", "party", "role", "sweep", "campaign", "feel", "deadlines"];
pub const SPENDS: [&str; 4] = ["class", "cons", "primary", "mixed"];
pub const LOADOUTS: [&str; 3] = ["default", "newest", "role"];
pub const SCORES: [&str; 3] = ["both", "hp", "binary"];
pub const GENERATED: [&str; 5] = ["mob", "line", "wave", "elite", "boss"];
pub const PRESETS: [&str; 7] = ["balanced", "aggressive", "focus_weakest", "defensive", "ranged_kite", "economy", "withdraw"];
/// RULESET1-G2 (RULESET1_BALANCE1 3.1): the feel mode's party tactics (AUTO1) and the PlanPresets id each drafts with
/// (RULESET1_INTENT AI5 maps them to the round intent table's tactics).
pub const TACTICS: [(&str, &str); 6] =
    [("balanced", "balanced"), ("all_out", "aggressive"), ("focus_weakest", "focus_weakest"), ("guard_counter", "defensive"), ("kite", "ranged_kite"), ("economy", "economy")];
/// RULESET1-G2: the keys only the feel mode reads (any other mode refuses them).
pub const FEEL_KEYS: [&str; 4] = ["tactic", "party", "armor", "tune"];
/// SIMCOUNT1: the feel key `usage` does not list (see `FEEL_COUNTS_USAGE`); any other mode refuses it too.
pub const FEEL_COUNTS_KEY: &str = "counts";
/// SIMCOUNT1: the help of the `counts=` key, printed apart from `usage` (`sim help`, a refused feel line): `sim_golden` holds
/// `usage`.
pub const FEEL_COUNTS_USAGE: &str = "       feel only: counts=on|off (off): each block and FEEL_JSON add per side the rule counts (feel::RULE_COUNTERS), protected share, Rest blocks a round, items by id; the hook counts (feel::HOOK_COUNTERS), skills landed, intents by unit, and per encounter arena strikes, windup breaks by skill, phases fired and fights with each count";
/// WORLD-SIM1 (PR5-42): the help of the feel mode's world encounters, printed apart from `usage` (`sim help`, a refused
/// feel line): `sim_golden` holds `usage`.
pub const FEEL_WORLD_USAGE: &str = "       feel world:<world>/<id> (or enc=world:<world>/<id>, or world=<world> enc=<id>): a world encounter with its authored party and guests at their authored levels, by its world's ruleset unless ruleset= picks one (stage= grows its enemies; armor= refuses it)";
/// BASERULES1-M2 (FB-40, library section 7): the help of the ruleset notes, printed apart from `usage` (`sim help`), which
/// `sim_golden` holds.
pub const RULE_NOTES_USAGE: &str = "       with ruleset= or rules=: the picked ruleset's notes (aegis rules: requires, conflicts, untested sets, ranges) print after ARGS as RULES error: / RULES warning: lines; the run still plays";
/// PR4-1 (R3-STAGE-SIM): the help of the `stage=` key, printed apart from `usage` (`sim help`), which `sim_golden` holds.
pub const STAGE_USAGE: &str = "       stage=0.. (0): world encounters (party, sweep, campaign) at that world stage: enemies grow by their growth_per_stage as in the campaign";
/// PK2-S5b (`docs/sessions/PK2_GRAFTS.md` 7): the `dulled=` values (the default dulled penalty alone; it also locks a dulled
/// main hand's techniques, `rpgkit_battle::world::World::dulled_lock`) and the key's help, printed apart from `usage`
/// (`sim help`), which `sim_golden` holds.
pub const DULLED_MODES: [&str; 2] = ["power", "lock"];
pub const DULLED_USAGE: &str = "       dulled=power|lock (power): a dulled weapon (0 durability, rule durability) loses weapon power; lock also locks its techniques";
/// PK2-S5b: the help of the `durability=` key (the allies' seeded weapon durability), printed apart from `usage`.
pub const DURABILITY_USAGE: &str = "       durability=0-999 (none): with the rule durability (ruleset= or rules=+durability) each ally's weapons start at that durability, so they wear and dull";
/// PK2-FOLLOWUP (PR5-36, FB-218 (c), FB-220): the help of the `mastery=` key (the allies' main-hand family mastery),
/// printed apart from `usage`.
pub const MASTERY_USAGE: &str = "       mastery=0-3 (none): each ally holds that weapon mastery in its main-hand family, so that many of the family's techniques open (dulled=lock can bite)";
/// RULESET1-G2: the `armor=` range (the unit field's, less -1 = derived).
pub const ARMOR_MAX: i32 = 200;
/// RULESET1-G2: the ruleset numbers `tune=` may set (RULESET1_TASKS 1.2), by key prefix; ranges from the kit schema.
/// PR5-26 (FB-191 (f)): `channel_` (the Channel rule's `channel_power` and `channel_ai_edge`).
pub const TUNE_PREFIXES: [&str; 5] = ["stagger_", "flank_", "break_chance_", "rest_", "channel_"];
/// PR5-26 (FB-191 (f)): the `tune=` keys of the ruleset's `channel_cost` (a list in the schema): the cost by AP 1, 2 and
/// 3+, each a whole number 0-`CHANNEL_COST_MAX`.
pub const CHANNEL_COST_KEYS: [&str; 3] = ["channel_cost_1", "channel_cost_2", "channel_cost_3"];
pub const CHANNEL_COST_MAX: f64 = 100.0;
/// CAMPAIGN1-SIM: the keys only the deadlines mode reads (any other mode refuses them).
pub const DEADLINE_KEYS: [&str; 1] = ["losses"];
/// CAMPAIGN1-SIM: `losses=` (lost tries per mission in the worst case): default and range.
pub const LOSSES_DEFAULT: i64 = 1;
pub const LOSSES_MAX: i64 = 9;
pub const REF_PREFIX: &str = "world:";
/// BAL5-SIM K2: party / sweep cells default to 10 seeds (other modes keep 3).
pub const PARTY_SEEDS: usize = 10;
/// tier_foe default: each hero fights its own class.
pub const MIRROR: &str = "mirror";
pub const ROLES: [&str; 5] = ["tank", "ranged", "aoe", "support", "striker"];
pub const CAP_MIN: i32 = 1;
pub const CAP_MAX: i32 = 30;
pub const DISTANCE_MIN: i32 = 2;
pub const DISTANCE_MAX: i32 = 6;
/// Metrics default on in these modes.
pub const METRIC_MODES: [&str; 3] = ["party", "role", "sweep"];
/// Role limit tests start enemies 4 tiles away.
pub const ROLE_DISTANCE: i32 = 4;
const PARTY_MAX: usize = rpgkit_world::data::PARTY_MAX as usize;
const POUCH_MAX: usize = rpgkit_world::state::POUCH_MAX;
const MAX_LEVEL: i32 = rpgkit_rules::stats::MAX_LEVEL;

#[derive(Debug, Clone)]
pub struct SimArgs {
    pub mode: String,
    pub level: i32,
    /// `stats all`.
    pub all_levels: bool,
    pub seed_count: usize,
    pub cap: i32,
    pub loadout: String,
    /// One of SPENDS or "all".
    pub spend: String,
    /// A preset id or "both".
    pub preset: String,
    /// 0 = default deployment.
    pub distance: i32,
    pub score: String,
    pub quick: bool,
    pub metrics: bool,
    /// REWARD1-S: party and campaign print what the defeated enemies pay (fates, XP, purse share, drops, key loot).
    pub rewards: bool,
    pub band_lo: i32,
    pub band_hi: i32,
    pub comp: Vec<String>,
    /// campaign: comp= picks one party instead of all 10 trios.
    pub comp_set: bool,
    /// campaign between-battle healing: none, full, rally or a percent of max HP.
    pub heal: String,
    /// campaign story battles (1-based) that start after a full rest.
    pub rest: Vec<i32>,
    /// campaign story battle -> encounters fought just before it.
    pub extra: IndexMap<i32, Vec<String>>,
    pub encounters: Vec<String>,
    pub elevel: i32,
    /// Catalog encounters use party level + elevel too.
    pub elevel_set: bool,
    /// R3-STAGE-SIM (PR4-1): the world stage (`world_stage`) world encounters are fought at: enemies grow by their
    /// unit's `growth_per_stage` as in the campaign (`rpgkit_world::battle::stage_level`). 0 = authored levels.
    pub stage: i32,
    /// PK2-S5b: `dulled=lock`: a dulled main hand also locks its techniques in every sim battle (`World::dulled_lock`);
    /// false (`power`, the default) keeps the dulled penalty alone.
    pub dulled_lock: bool,
    /// PK2-S5b: `durability=N`: each ally's wielded weapons (main hand, off-hand weapon) start every sim battle tracking
    /// N durability (`rpgkit_battle::wear::seed`; only under a ruleset with the rule `durability`). None = no wear (catalog
    /// allies track none).
    pub durability: Option<i32>,
    /// PK2-FOLLOWUP: `mastery=N`: each ally holds mastery N (0-3) in its main-hand weapon family at every sim battle
    /// start (`rpgkit_battle::sim::apply_mastery`), so the family's first N techniques open. None = catalog units hold
    /// no mastery.
    pub mastery: Option<i32>,
    pub elite_class: String,
    pub boss_class: String,
    pub roles: Vec<String>,
    pub classes: Vec<String>,
    pub dupes: bool,
    /// sweep dupes=on: draw this many multisets (0 = all).
    pub sample: usize,
    pub jitter: bool,
    pub targets: Targets,
    pub pouch: Vec<String>,
    /// world= (default: the game's default world).
    pub world: String,
    pub world_set: bool,
    /// KIT1-SIM: what the game told the sim (default world, story run).
    pub game: SimGame,
    pub comp_size: usize,
    pub lead: String,
    pub seeds_set: bool,
    pub tier_foe: String,
    pub boss_phase: bool,
    pub s2: bool,
    /// RULESET1 3.1.3: the fights' rules, from `ruleset=classic|timeline` (default classic) and `rules=-id,+id` (single
    /// extensions switched). A world encounter
    /// without `ruleset=` plays by its world's ruleset instead (`ruleset_for`).
    pub ruleset: rpgkit_rules::ruleset::Ruleset,
    ruleset_id: String,
    rules: String,
    /// BASERULES1-M2 (FB-40): the picked ruleset's notes (`Ruleset::notes`, errors first) as printable lines, when the
    /// command line gives `ruleset=`, or `rules=` on a run with no world fights; empty otherwise and for a clean row (the
    /// feel mode's own timeline default prints none).
    pub rule_notes: Vec<String>,
    /// INPUT1 MR11: `melee_diagonal=none|maneuver|all` over the ruleset row's value (None = the row's).
    pub melee_diagonal: Option<rpgkit_rules::ruleset::MeleeDiagonal>,
    /// ATTACKLOCK1: `attack_lock=none|melee|all` over the ruleset row's value (None = the row's).
    pub attack_lock: Option<rpgkit_rules::ruleset::AttackLock>,
    /// RULESET1-G2 (feel only): the party's AUTO1 tactic (a TACTICS name; `balanced`).
    pub tactic: String,
    /// RULESET1-G2 (feel only): the party drivers, each run as its own block (`party=auto|timed|both`; auto).
    pub drivers: Vec<crate::timed::Driver>,
    /// SIMCOUNT1 (feel only): print the rule counts (`counts=on|off`; off).
    pub counts: bool,
    /// RULESET1-G2 (feel only): unit id -> the armour it fights with in this run (`armor=<unit>:<0-200>`).
    pub armor: IndexMap<String, i32>,
    /// RULESET1-G2 (feel only): ruleset number -> its value in this run (`tune=<key>:<value>`), applied to `ruleset`.
    pub tune: IndexMap<String, f64>,
    /// The feel keys given, for the check that no other mode takes them.
    feel_keys: Vec<String>,
    /// CAMPAIGN1-SIM (deadlines only): the lost tries per mission in the worst case (`losses=0..9`; 1).
    pub losses: i64,
    /// The deadlines keys given, for the check that no other mode takes them.
    deadline_keys: Vec<String>,
}

impl Default for SimArgs {
    fn default() -> Self {
        Self {
            mode: "duel".into(),
            level: 5,
            all_levels: false,
            seed_count: 3,
            cap: 10,
            loadout: "default".into(),
            spend: "class".into(),
            preset: "balanced".into(),
            distance: 0,
            score: "both".into(),
            quick: false,
            metrics: false,
            rewards: false,
            band_lo: 30,
            band_hi: 70,
            comp: Vec::new(),
            comp_set: false,
            heal: "none".into(),
            rest: Vec::new(),
            extra: IndexMap::new(),
            encounters: strings(&GENERATED),
            elevel: 0,
            elevel_set: false,
            stage: 0,
            dulled_lock: false,
            durability: None,
            mastery: None,
            elite_class: String::new(),
            boss_class: String::new(),
            roles: strings(&ROLES),
            classes: Vec::new(),
            dupes: false,
            sample: 0,
            jitter: true,
            targets: targets::defaults(),
            pouch: Vec::new(),
            world: String::new(),
            world_set: false,
            game: SimGame::default(),
            comp_size: 4,
            lead: String::new(),
            seeds_set: false,
            tier_foe: MIRROR.into(),
            boss_phase: true,
            s2: false,
            // CLN-44: a placeholder; `parse` reads the picked row (default classic) from the catalog
            ruleset: rpgkit_rules::ruleset::Ruleset::default(),
            ruleset_id: String::new(),
            rules: String::new(),
            rule_notes: Vec::new(),
            melee_diagonal: None,
            attack_lock: None,
            tactic: "balanced".into(),
            drivers: vec![crate::timed::Driver::Auto],
            counts: false,
            armor: IndexMap::new(),
            tune: IndexMap::new(),
            feel_keys: Vec::new(),
            losses: LOSSES_DEFAULT,
            deadline_keys: Vec::new(),
        }
    }
}

pub fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

/// Godot's str(Array[String]): ["a", "b"].
pub fn godot_list(v: &[&str]) -> String {
    format!("[{}]", v.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(", "))
}

fn is_int(s: &str) -> bool {
    let t = s.strip_prefix(['+', '-']).unwrap_or(s);
    !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit())
}

fn int(s: &str) -> i32 {
    s.parse().unwrap_or(0)
}

/// SimArgs._list: comma separated, trimmed, empty parts dropped.
fn list(value: &str) -> Vec<String> {
    value.split(',').map(str::trim).filter(|p| !p.is_empty()).map(String::from).collect()
}

/// `enc=world:<id>` or `world:<world>/<id>` -> (world, id) (None when not a world reference).
pub fn parse_ref(text: &str, default_world: &str) -> Option<(String, String)> {
    let rest = text.strip_prefix(REF_PREFIX)?;
    let parts: Vec<&str> = rest.split('/').filter(|p| !p.is_empty()).collect();
    match parts.as_slice() {
        [id] => Some((default_world.to_string(), id.to_string())),
        [w, id] => Some((w.to_string(), id.to_string())),
        _ => None,
    }
}

/// True when `world` sees encounter `id` (its own or inherited).
pub fn has_encounter(d: &Data, world: &str, id: &str) -> bool {
    d.world_ids().iter().any(|w| w == world) && d.has_entry(world, "encounters", id)
}

/// Every balance fight of the catalog, in catalog order (`enc=all`): ENCOUNTER2 S4 skips the stay-behind and room
/// encounters and the kit's own (`kit_` ids); `enc=<id>` still runs them.
pub fn catalog_encounters(d: &Data) -> Vec<String> {
    d.cat.encounters.iter().filter(|(id, row)| !skipped_by_default(id, row)).map(|(id, _)| id.clone()).collect()
}

/// ENCOUNTER2 S4: not a balance fight (an id starting `kit_`, a `stay_behind` block or `rooms`).
pub fn skipped_by_default(id: &str, row: &aegis_core::value::Obj) -> bool {
    let filled = |key: &str| match row.get(key) {
        Some(serde_json::Value::Object(o)) => !o.is_empty(),
        Some(serde_json::Value::Array(a)) => !a.is_empty(),
        _ => false,
    };
    id.starts_with("kit_") || filled("stay_behind") || filled("rooms")
}

/// BASERULES1-M2 (FB-40, `BASERULES1_LIBRARY.md` section 7): a ruleset's notes as the sim prints them, errors first,
/// in `Ruleset::notes` order (the same text `aegis rules` shows).
pub fn rule_note_lines(r: &rpgkit_rules::ruleset::Ruleset) -> Vec<String> {
    let n = r.notes();
    n.errors.iter().map(|e| format!("RULES error: {e}")).chain(n.warnings.iter().map(|w| format!("RULES warning: {w}"))).collect()
}

impl SimArgs {
    /// RULESET1 3.1.2-3.1.3: the ruleset of a fight. `world`: the ruleset of the world that resolves the encounter (None
    /// for a catalog or generated fight). `ruleset=` overrides it; without it a world fight plays by its world's ruleset
    /// (with the `rules=` switches) and any other fight by classic.
    pub fn ruleset_for(&self, world: Option<&rpgkit_rules::ruleset::Ruleset>) -> rpgkit_rules::ruleset::Ruleset {
        match world {
            Some(w) if self.ruleset_id.is_empty() => {
                let mut r = w.clone();
                // the switches were checked at parse; their errors depend on the ids only
                let _ = r.adjust(&self.rules);
                if let Some(m) = self.melee_diagonal {
                    r.numbers.melee_diagonal = m;
                }
                if let Some(l) = self.attack_lock {
                    r.numbers.attack_lock = l;
                }
                r
            }
            _ => self.ruleset.clone(),
        }
    }

    /// Parses the sim's command line (mode first) for a game without a default world or story run. Err = the
    /// message after "SIM argument error: ".
    pub fn parse(d: &Data, argv: &[String]) -> Result<Self, String> {
        Self::parse_for(d, argv, &SimGame::default())
    }

    /// Parses the sim's command line (mode first) for `game`: world= defaults to its default world, the campaign
    /// mode plays its story run. Err = the message after "SIM argument error: ".
    pub fn parse_for(d: &Data, argv: &[String], game: &SimGame) -> Result<Self, String> {
        // KITIDS1: the class defaults are the game's (SimGame.classes); the kit names no class
        let c = &game.classes;
        let mut a = SimArgs {
            world: game.default_world.clone(),
            game: game.clone(),
            comp: strings(c.default_comp),
            elite_class: c.elite.into(),
            boss_class: c.boss.into(),
            classes: strings(c.ids),
            ..SimArgs::default()
        };
        let mut positional: Vec<String> = Vec::new();
        let mut metrics_set = false;
        let mut distance_set = false;
        let mut enc_set = false;
        let mut world_refs: Vec<String> = Vec::new();
        // world= first: enc= ids resolve against it whatever the token order.
        for raw in argv {
            let w = raw.trim();
            if let Some(v) = w.strip_prefix("world=") {
                a.key(d, "world", v)?;
            }
        }
        for raw in argv {
            let token = raw.trim();
            if token.is_empty() || token.starts_with("world=") {
                continue;
            }
            if token == "quick" {
                a.quick = true;
                continue;
            }
            // WORLD-SIM1 (PR5-42): a positional world reference (`sim feel world:<world>/<id>`) is an `enc=` entry
            if token.starts_with(REF_PREFIX) && !token.contains('=') {
                world_refs.push(token.to_string());
                continue;
            }
            let Some((key, value)) = token.split_once('=') else {
                positional.push(token.to_string());
                continue;
            };
            a.key(d, key, value)?;
            metrics_set |= key == "metrics";
            distance_set |= key == "distance";
            enc_set |= key == "enc";
        }
        if !world_refs.is_empty() {
            // after any `enc=` list, in the order given
            let before = if enc_set { std::mem::take(&mut a.encounters) } else { Vec::new() };
            a.key(d, "enc", &world_refs.join(","))?;
            a.encounters.splice(0..0, before);
            enc_set = true;
        }
        if let Some(m) = positional.first() {
            a.mode = m.clone();
        }
        if !MODES.contains(&a.mode.as_str()) {
            return Err(format!("unknown mode '{}'", a.mode));
        }
        if let Some(l) = positional.get(1) {
            if l == "all" && a.mode == "stats" {
                a.all_levels = true;
            } else if is_int(l) && int(l) >= 1 && int(l) <= MAX_LEVEL {
                a.level = int(l);
            } else {
                return Err(format!("level must be 1..{MAX_LEVEL}{} (got '{l}')", if a.mode == "stats" { " or all" } else { "" }));
            }
        }
        if let Some(s) = positional.get(2) {
            if !is_int(s) || int(s) < 1 {
                return Err(format!("seeds must be a positive number (got '{s}')"));
            }
            a.seed_count = int(s) as usize;
            a.seeds_set = true;
        } else if (a.mode == "party" || a.mode == "sweep") && !a.seeds_set {
            // PR5-26 (FB-193 (a)): `seeds=N` holds in these modes too; only without it the party default
            a.seed_count = PARTY_SEEDS;
        } else if a.mode == "feel" && !a.seeds_set {
            a.seed_count = crate::feel::FEEL_SEEDS;
        }
        if positional.len() > 3 {
            return Err("too many positional arguments".into());
        }
        if a.comp.len() > PARTY_MAX {
            return Err(format!("party size > PARTY_MAX ({PARTY_MAX})"));
        }
        if !metrics_set {
            a.metrics = METRIC_MODES.contains(&a.mode.as_str());
        }
        if a.mode == "role" && !distance_set {
            a.distance = ROLE_DISTANCE;
        }
        if a.quick {
            a.seed_count = 1;
        }
        // BASERULES1-M2: whether the command line picked the rules (before the feel mode fills in its own default)
        let picked_ruleset = !a.ruleset_id.is_empty();
        let switched = !a.rules.is_empty();
        if a.mode == "feel" {
            a.feel_defaults(d, enc_set)?;
        } else if let Some(k) = a.feel_keys.first() {
            return Err(format!("{k}= is a feel key (mode feel only)"));
        }
        if a.mode == "deadlines" {
            if positional.len() > 1 {
                return Err("deadlines takes no level or seeds".into());
            }
        } else if let Some(k) = a.deadline_keys.first() {
            return Err(format!("{k}= is a deadlines key (mode deadlines only)"));
        }
        a.ruleset = rpgkit_rules::ruleset::Ruleset::of(&d.cat, &a.ruleset_id)?;
        a.ruleset.adjust(&a.rules)?;
        if let Some(m) = a.melee_diagonal {
            a.ruleset.numbers.melee_diagonal = m;
        }
        if let Some(l) = a.attack_lock {
            a.ruleset.numbers.attack_lock = l;
        }
        // BASERULES1-M2 (FB-40): the notes of the rules the command line picked. `ruleset=` plays every fight by that row;
        // `rules=` alone switches classic for catalog and generated fights but each world's own row for world fights
        // (`ruleset_for`), so a run that may play world fights prints none rather than another ruleset's notes.
        let world_fights = a.world_set || a.mode == "campaign" || a.encounters.iter().any(|e| parse_ref(e, &a.world).is_some());
        if picked_ruleset || (switched && !world_fights) {
            a.rule_notes = rule_note_lines(&a.ruleset);
        }
        if a.mode == "feel" {
            a.feel_checks(d)?;
        }
        Ok(a)
    }

    /// RULESET1-G (RULESET1_TASKS 3): the `feel` mode's defaults: the game's feel encounters unless `enc=` is given;
    /// every encounter must be a catalog encounter with a party or (WORLD-SIM1, PR5-42) a world encounter
    /// (`world:<world>/<id>`) with an authored `party`. Without `ruleset=` a catalog encounter plays by the timeline
    /// ruleset and a world encounter by its world's (`Data::ruleset`, as in play); the run's encounters must agree on
    /// that default, and it becomes the run's ruleset (so `preset=`, `tune=` and the header read it). `armor=` changes
    /// catalog units only, so it refuses a world encounter (its units are the world's characters).
    fn feel_defaults(&mut self, d: &Data, enc_set: bool) -> Result<(), String> {
        if !enc_set {
            if self.game.feel_encounters.is_empty() {
                return Err("feel needs enc=<catalog encounters with a party> (the game names no default)".into());
            }
            self.encounters = self.game.feel_encounters.clone();
        }
        let mut defaults: Vec<String> = Vec::new();
        for e in &self.encounters {
            let default = match parse_ref(e, &self.world) {
                Some((w, id)) => {
                    if aegis_core::json::sget(d.resolve(&w, "encounters", &id), "party").is_empty() {
                        return Err(format!("feel: '{e}' is a world encounter with no authored party"));
                    }
                    if let Some(u) = self.armor.keys().next() {
                        return Err(format!("armor: '{u}' changes a catalog unit; world encounter '{e}' fights the world's characters"));
                    }
                    d.ruleset(&w).id
                }
                None => {
                    if d.cat.encounters.get(e).is_none_or(|row| aegis_core::json::sget(row, "party").is_empty()) {
                        return Err(format!("feel: '{e}' is not a catalog encounter with a party"));
                    }
                    rpgkit_rules::ruleset::TIMELINE.to_string()
                }
            };
            if !defaults.contains(&default) {
                defaults.push(default);
            }
        }
        if self.ruleset_id.is_empty() {
            match defaults.as_slice() {
                [one] => self.ruleset_id = one.clone(),
                _ => return Err(format!("feel: the encounters play by different rulesets ({}); pick one with ruleset=", defaults.join(", "))),
            }
        }
        Ok(())
    }

    /// RULESET1-G2 (RULESET1_BALANCE1 3.1): the feel keys against the parsed ruleset and encounters: `preset=` is
    /// classic-only (one preset; only `balanced` under timeline), each `armor=` unit is in a fought roster, each
    /// `tune=` number sits in the kit schema's range and is set on the run's ruleset.
    fn feel_checks(&mut self, d: &Data) -> Result<(), String> {
        if self.preset == "both" {
            return Err("feel takes one preset (got 'both')".into());
        }
        if self.ruleset.has(rpgkit_rules::ruleset::ROUND_INTENT) && self.preset != "balanced" {
            return Err("preset is classic-only; use tactic=".into());
        }
        let fought: Vec<String> = self.encounters.iter().flat_map(|e| feel_roster(d, e)).collect();
        for id in self.armor.keys() {
            if !fought.contains(id) {
                return Err(format!("armor: '{id}' is in no fought roster ({})", self.encounters.join(",")));
            }
        }
        for (key, v) in &self.tune {
            let (lo, hi) = tune_range(key).ok_or_else(|| format!("tune: '{key}' is not a ruleset tuning number (keys {})", tune_keys().join(", ")))?;
            if *v < lo || *v > hi {
                return Err(format!("tune: {key} must be {lo}..{hi} (got {v})"));
            }
            self.ruleset.numbers.set(key, *v).map_err(|e| format!("tune: {e}"))?;
        }
        Ok(())
    }

    /// The PlanPresets id the party drafts with in the feel mode (its tactic's).
    pub fn tactic_preset(&self) -> &'static str {
        TACTICS.iter().find(|(name, _)| *name == self.tactic).map_or("balanced", |(_, p)| p)
    }

    fn key(&mut self, d: &Data, key: &str, value: &str) -> Result<(), String> {
        let ids = self.game.classes.ids;
        let classes = godot_list(ids);
        if (FEEL_KEYS.contains(&key) || key == FEEL_COUNTS_KEY) && !self.feel_keys.iter().any(|k| k == key) {
            self.feel_keys.push(key.to_string());
        }
        if DEADLINE_KEYS.contains(&key) && !self.deadline_keys.iter().any(|k| k == key) {
            self.deadline_keys.push(key.to_string());
        }
        match key {
            // CAMPAIGN1-SIM: the deadlines mode's key.
            "losses" => {
                let Some(n) = value.parse::<i64>().ok().filter(|n| (0..=LOSSES_MAX).contains(n)) else {
                    return Err(format!("losses must be 0..{LOSSES_MAX} (got '{value}')"));
                };
                self.losses = n;
            }
            // RULESET1-G2: the feel mode's keys (RULESET1_BALANCE1 3.1).
            "tactic" => {
                if !TACTICS.iter().any(|(name, _)| *name == value) {
                    let names: Vec<&str> = TACTICS.iter().map(|(name, _)| *name).collect();
                    return Err(format!("tactic must be one of {} (got '{value}')", godot_list(&names)));
                }
                self.tactic = value.into();
            }
            "party" => {
                self.drivers = match value {
                    "both" => vec![crate::timed::Driver::Auto, crate::timed::Driver::Timed],
                    v => vec![crate::timed::Driver::parse(v).ok_or_else(|| format!("party must be one of {} or both (got '{value}')", godot_list(&crate::timed::DRIVERS)))?],
                };
            }
            "counts" => {
                if value != "on" && value != "off" {
                    return Err(format!("counts must be on or off (got '{value}')"));
                }
                self.counts = value == "on";
            }
            "armor" => {
                for part in list(value) {
                    let parsed = part.split_once(':').filter(|(id, n)| !id.is_empty() && is_int(n) && (0..=ARMOR_MAX).contains(&int(n)));
                    let Some((id, n)) = parsed else { return Err(format!("armor must be <unit id>:<0-{ARMOR_MAX}>, comma separated (got '{part}')")) };
                    self.armor.insert(id.to_string(), int(n));
                }
            }
            "tune" => {
                for part in list(value) {
                    let parsed = part.split_once(':').and_then(|(k, v)| v.parse::<f64>().ok().filter(|f| f.is_finite()).map(|f| (k, f)));
                    let Some((k, v)) = parsed else { return Err(format!("tune must be <ruleset number>:<value>, comma separated (got '{part}')")) };
                    self.tune.insert(k.to_string(), v);
                }
            }
            // RULESET1-G: `seeds=N` (the feel mode's spelling; the positional seeds win).
            "seeds" => {
                if !is_int(value) || int(value) < 1 {
                    return Err(format!("seeds must be a positive number (got '{value}')"));
                }
                self.seed_count = int(value) as usize;
                self.seeds_set = true;
            }
            "cap" => {
                if !is_int(value) || int(value) < CAP_MIN || int(value) > CAP_MAX {
                    return Err(format!("cap must be {CAP_MIN}..{CAP_MAX} (got '{value}')"));
                }
                self.cap = int(value);
            }
            "loadout" => {
                if !LOADOUTS.contains(&value) {
                    return Err(format!("loadout must be one of {} (got '{value}')", godot_list(&LOADOUTS)));
                }
                self.loadout = value.into();
            }
            "spend" => {
                if !SPENDS.contains(&value) && value != "all" {
                    return Err(format!("spend must be one of {} or all (got '{value}')", godot_list(&SPENDS)));
                }
                self.spend = value.into();
            }
            "preset" => {
                if !PRESETS.contains(&value) && value != "both" {
                    return Err(format!("preset must be one of {} or both (got '{value}')", godot_list(&PRESETS)));
                }
                self.preset = value.into();
            }
            "distance" => {
                if value == "default" {
                    self.distance = 0;
                } else if is_int(value) && int(value) >= DISTANCE_MIN && int(value) <= DISTANCE_MAX {
                    self.distance = int(value);
                } else {
                    return Err(format!("distance must be default or {DISTANCE_MIN}..{DISTANCE_MAX} (got '{value}')"));
                }
            }
            "score" => {
                if !SCORES.contains(&value) {
                    return Err(format!("score must be one of {} (got '{value}')", godot_list(&SCORES)));
                }
                self.score = value.into();
            }
            "metrics" => {
                if value != "on" && value != "off" {
                    return Err(format!("metrics must be on or off (got '{value}')"));
                }
                self.metrics = value == "on";
            }
            // CLN-44: the ids are open; `parse` reads the row (an id without one is refused there)
            "ruleset" => self.ruleset_id = value.into(),
            "rules" => self.rules = value.into(),
            "melee_diagonal" => {
                let ids = rpgkit_rules::ruleset::MeleeDiagonal::IDS;
                self.melee_diagonal = Some(rpgkit_rules::ruleset::MeleeDiagonal::parse(value).ok_or_else(|| format!("melee_diagonal must be one of {} (got '{value}')", godot_list(&ids)))?);
            }
            "attack_lock" => {
                let ids = rpgkit_rules::ruleset::AttackLock::IDS;
                self.attack_lock = Some(rpgkit_rules::ruleset::AttackLock::parse(value).ok_or_else(|| format!("attack_lock must be one of {} (got '{value}')", godot_list(&ids)))?);
            }
            "rewards" => {
                if value != "on" && value != "off" {
                    return Err(format!("rewards must be on or off (got '{value}')"));
                }
                self.rewards = value == "on";
            }
            "quick" => self.quick = true,
            "band" => {
                let parts: Vec<&str> = value.split('-').collect();
                let ok = parts.len() == 2 && is_int(parts[0]) && is_int(parts[1]) && int(parts[0]) >= 0 && int(parts[1]) <= 100 && int(parts[0]) <= int(parts[1]);
                if !ok {
                    return Err(format!("band must be lo-hi in percent, 0 <= lo <= hi <= 100 (got '{value}')"));
                }
                self.band_lo = int(parts[0]);
                self.band_hi = int(parts[1]);
                self.targets.insert("duel".into(), (self.band_lo as f64, self.band_hi as f64));
            }
            "target" => {
                if !targets::apply_override(&mut self.targets, value) {
                    let keys: Vec<&str> = targets::DEFAULTS.iter().map(|(k, _, _)| *k).collect();
                    return Err(format!("target must be <key>:<lo>-<hi> with a key of {} (got '{value}')", godot_list(&keys)));
                }
                if value.starts_with("duel:") {
                    self.band_lo = targets::lo(&self.targets, "duel").round() as i32;
                    self.band_hi = targets::hi(&self.targets, "duel").round() as i32;
                }
            }
            "comp" => {
                let picked = list(value);
                if picked.is_empty() {
                    return Err(format!("comp needs 1..{PARTY_MAX} class ids (got '{value}')"));
                }
                for c in &picked {
                    if !ids.contains(&c.as_str()) {
                        return Err(format!("comp: unknown class '{c}' (one of {classes})"));
                    }
                }
                self.comp = picked;
                self.comp_set = true;
            }
            "heal" => {
                let pct = value.strip_suffix('%').unwrap_or(value);
                if value == "none" || value == "full" || value == "rally" {
                    self.heal = value.into();
                } else if is_int(pct) && (0..=100).contains(&int(pct)) {
                    self.heal = int(pct).to_string();
                } else {
                    return Err(format!("heal must be none, full, rally or a percent 0..100 (got '{value}')"));
                }
            }
            "rest" => {
                let mut picked = Vec::new();
                if value != "none" {
                    for part in list(value) {
                        if !is_int(&part) || int(&part) < 1 {
                            return Err(format!("rest must be none or story battle numbers >= 1, e.g. rest=3 (got '{value}')"));
                        }
                        picked.push(int(&part));
                    }
                }
                self.rest = picked;
            }
            "extra" => {
                let mut picked: IndexMap<i32, Vec<String>> = IndexMap::new();
                if value != "none" {
                    for part in list(value) {
                        let at = part.split(':').next().unwrap_or("").to_string();
                        let enc = part.get(at.len() + 1..).unwrap_or("").to_string();
                        let world = &self.game.default_world;
                        if !is_int(&at) || int(&at) < 1 || enc.is_empty() || !has_encounter(d, world, &enc) {
                            let example = self.game.story.as_ref().map_or("4:<encounter>", |s| s.extra_example.as_str());
                            return Err(format!("extra must be <battle>:<{world} encounter>[,..], e.g. extra={example} (got '{part}')"));
                        }
                        picked.entry(int(&at)).or_default().push(enc);
                    }
                }
                self.extra = picked;
            }
            "enc" => {
                let mut picked = Vec::new();
                for e in list(value) {
                    if e == "generated" {
                        picked.extend(strings(&GENERATED));
                    } else if e == "all" {
                        picked.extend(strings(&GENERATED));
                        picked.extend(catalog_encounters(d));
                    } else if e.starts_with(REF_PREFIX) {
                        match parse_ref(&e, &self.world) {
                            Some((w, id)) if has_encounter(d, &w, &id) => picked.push(e),
                            _ => return Err(format!("enc: unknown world encounter '{e}' (world:<id> in {} or world:<world>/<id>)", self.world)),
                        }
                    } else if GENERATED.contains(&e.as_str()) {
                        picked.push(e);
                    } else if self.world_set && has_encounter(d, &self.world, &e) {
                        picked.push(format!("{REF_PREFIX}{}/{e}", self.world));
                    } else if d.cat.encounters.contains_key(&e) {
                        picked.push(e);
                    } else {
                        return Err(format!(
                            "enc: unknown encounter '{e}' (generated ids {}, a catalog encounter id, an encounter of world= {}, generated or all)",
                            godot_list(&GENERATED),
                            self.world
                        ));
                    }
                }
                if picked.is_empty() {
                    return Err("enc needs at least one encounter".into());
                }
                self.encounters = picked;
            }
            "elevel" => {
                if value == "catalog" {
                    self.elevel = 0;
                    self.elevel_set = false;
                } else if is_int(value.strip_prefix('+').unwrap_or(value)) {
                    self.elevel = int(value.strip_prefix('+').unwrap_or(value));
                    self.elevel_set = true;
                } else {
                    return Err(format!("elevel must be +n, -n or catalog (got '{value}')"));
                }
            }
            "stage" => {
                match value.parse::<i32>() {
                    Ok(n) if n >= 0 => self.stage = n,
                    _ => return Err(format!("stage must be a number >= 0 (got '{value}')")),
                }
            }
            "dulled" => {
                if !DULLED_MODES.contains(&value) {
                    return Err(format!("dulled must be power or lock (got '{value}')"));
                }
                self.dulled_lock = value == DULLED_MODES[1];
            }
            "durability" => {
                let max = rpgkit_data::DURABILITY_MAX;
                match value.parse::<i64>() {
                    Ok(n) if (0..=max).contains(&n) => self.durability = Some(n as i32),
                    _ => return Err(format!("durability must be a number 0-{max} (got '{value}')")),
                }
            }
            "mastery" => {
                let max = rpgkit_data::mastery::MASTERY_MAX;
                match value.parse::<i32>() {
                    Ok(n) if (0..=max).contains(&n) => self.mastery = Some(n),
                    _ => return Err(format!("mastery must be a number 0-{max} (got '{value}')")),
                }
            }
            "elite" | "boss" => {
                if !ids.contains(&value) {
                    return Err(format!("{key} must be a class id of {classes} (got '{value}')"));
                }
                if key == "elite" {
                    self.elite_class = value.into();
                } else {
                    self.boss_class = value.into();
                }
            }
            "role" => {
                if value == "all" {
                    self.roles = strings(&ROLES);
                } else if ROLES.contains(&value) {
                    self.roles = vec![value.into()];
                } else {
                    return Err(format!("role must be one of {} or all (got '{value}')", godot_list(&ROLES)));
                }
            }
            "class" => {
                if value == "all" {
                    self.classes = strings(ids);
                } else if ids.contains(&value) {
                    self.classes = vec![value.into()];
                } else {
                    return Err(format!("class must be one of {classes} or all (got '{value}')"));
                }
            }
            "dupes" | "jitter" | "phase" | "s2" => {
                if value != "on" && value != "off" {
                    return Err(format!("{key} must be on or off (got '{value}')"));
                }
                let on = value == "on";
                match key {
                    "dupes" => self.dupes = on,
                    "jitter" => self.jitter = on,
                    "phase" => self.boss_phase = on,
                    _ => self.s2 = on,
                }
            }
            "pouch" => {
                let ids = list(value);
                // A1.2-K3 (INTENT2-K3): the classic form (1..POUCH_MAX copies) or the stack form (at most POUCH_MAX
                // distinct ids, each at most its stack limit; the battle loads it under `pouch_stacks`)
                let mut distinct: Vec<&String> = Vec::new();
                for id in &ids {
                    if !distinct.contains(&id) {
                        distinct.push(id);
                    }
                }
                let stacked = distinct.len() <= POUCH_MAX && distinct.iter().all(|id| ids.iter().filter(|x| x == id).count() as i64 <= rpgkit_world::state::stack_limit(d, id));
                if ids.is_empty() || (ids.len() > POUCH_MAX && !stacked) {
                    return Err(format!("pouch must list 1..{POUCH_MAX} consumable ids, or {POUCH_MAX} stacks each within its stack limit (got '{value}')"));
                }
                for id in &ids {
                    if aegis_core::json::s(d.entry("items", id), "item_type") != "CONSUMABLE" {
                        return Err(format!("pouch: '{id}' is not a consumable item"));
                    }
                }
                self.pouch = ids;
            }
            "world" => {
                let ids = d.world_ids();
                if !ids.iter().any(|w| w == value) {
                    let names: Vec<&str> = ids.iter().map(String::as_str).collect();
                    return Err(format!("world must be one of {} (got '{value}')", godot_list(&names)));
                }
                self.world = value.into();
                self.world_set = true;
            }
            "size" => {
                if !is_int(value) || int(value) < 1 || int(value) as usize > PARTY_MAX {
                    return Err(format!("size must be 1..{PARTY_MAX} (got '{value}')"));
                }
                self.comp_size = int(value) as usize;
            }
            "lead" => {
                if !ids.contains(&value) {
                    return Err(format!("lead must be a class id of {classes} (got '{value}')"));
                }
                self.lead = value.into();
            }
            "tier_foe" => {
                if value != MIRROR && !ids.contains(&value) {
                    return Err(format!("tier_foe must be {MIRROR} or a class id of {classes} (got '{value}')"));
                }
                self.tier_foe = value.into();
            }
            "sample" => {
                if !is_int(value) || int(value) < 1 {
                    return Err(format!("sample must be a positive number (got '{value}')"));
                }
                self.sample = int(value) as usize;
            }
            _ => return Err(format!("unknown key '{key}'")),
        }
        Ok(())
    }

    /// Seed i = 11 + 12 i (the old sim's seeds).
    pub fn seeds(&self) -> Vec<i64> {
        (0..self.seed_count as i64).map(|i| 11 + 12 * i).collect()
    }

    /// The tier-mode foe class for hero class `hero` (its own class under `mirror`).
    pub fn tier_foe_for(&self, hero: &str) -> String {
        if self.tier_foe == MIRROR {
            hero.into()
        } else {
            self.tier_foe.clone()
        }
    }

    /// The spend variants a matchup averages over (`all` = class, cons, primary; `quick` keeps the first).
    pub fn spend_variants(&self) -> Vec<String> {
        let mut out = if self.spend == "all" { strings(&["class", "cons", "primary"]) } else { vec![self.spend.clone()] };
        if self.quick {
            out.truncate(1);
        }
        out
    }

    /// The presets a matchup averages over (`both` = balanced, aggressive).
    pub fn preset_variants(&self) -> Vec<String> {
        let mut out = if self.preset == "both" { strings(&["balanced", "aggressive"]) } else { vec![self.preset.clone()] };
        if self.quick {
            out.truncate(1);
        }
        out
    }

    pub fn show_hp(&self) -> bool {
        self.score != "binary"
    }

    pub fn show_binary(&self) -> bool {
        self.score != "hp"
    }

    /// KITIDS1: the campaign header's party text without comp=: "all trios (<lead> + 2 of <the other classes>)".
    fn trios_text(&self) -> String {
        let c = &self.game.classes;
        let others: Vec<&str> = c.ids.iter().copied().filter(|id| *id != c.trio_lead).collect();
        format!("all trios ({} + 2 of {})", c.trio_lead, others.join(","))
    }

    /// One line echoing the effective arguments (every table starts with it).
    pub fn header(&self) -> String {
        let on = |b: bool| if b { "on" } else { "off" };
        let lvl = if self.all_levels { "all".to_string() } else { self.level.to_string() };
        let mut t = format!(
            "ARGS mode {}, level {lvl}, seeds {}, cap {}, loadout {}, spend {}, preset {}, distance {}, score {}, quick {}, metrics {}, band {}-{}, jitter {}",
            self.mode,
            self.seed_count,
            self.cap,
            self.loadout,
            self.spend,
            self.preset,
            if self.distance == 0 { "default".to_string() } else { self.distance.to_string() },
            self.score,
            on(self.quick),
            on(self.metrics),
            self.band_lo,
            self.band_hi,
            on(self.jitter)
        );
        match self.mode.as_str() {
            "party" | "sweep" => {
                let elevel = if self.elevel_set { format!("{:+}", self.elevel) } else { "+0 (catalog authored)".into() };
                t += &format!(", enc {}, elevel {elevel}, elite {}, boss {}", self.encounters.join(","), self.elite_class, self.boss_class);
                t += &format!(", phase {}", on(self.boss_phase));
                if self.world_set {
                    t += &format!(", world {}", self.world);
                }
                if self.mode == "party" {
                    t += &format!(", comp {}", self.comp.join(","));
                } else {
                    let sample = if self.sample > 0 { self.sample.to_string() } else { "all".into() };
                    t += &format!(", size {}, lead {}, dupes {}, sample {sample}", self.comp_size, if self.lead.is_empty() { "none" } else { &self.lead }, on(self.dupes));
                }
            }
            "campaign" => {
                let comp = if self.comp_set { self.comp.join(",") } else { self.trios_text() };
                t += &format!(", comp {comp}, heal {}", self.heal);
                if !self.rest.is_empty() {
                    t += &format!(", rest {}", self.rest.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(","));
                }
                if !self.extra.is_empty() {
                    let parts: Vec<String> = self.extra.iter().map(|(b, l)| format!("{b}:{}", l.join(","))).collect();
                    t += &format!(", extra {}", parts.join(" "));
                }
            }
            "role" => {
                t += &format!(", role {}, class {}", self.roles.join(","), self.classes.join(","));
                if self.s2 {
                    t += ", s2 on";
                }
            }
            "tier" => t += &format!(", tier_foe {}", self.tier_foe),
            "feel" => {
                t += &format!(", enc {}, ruleset {}{}", self.encounters.join(","), self.ruleset.id, if self.rules.is_empty() { String::new() } else { format!(", rules {}", self.rules) });
                // RULESET1-G2: every feel key that is not its default
                if self.tactic != "balanced" {
                    t += &format!(", tactic {}", self.tactic);
                }
                if self.drivers != [crate::timed::Driver::Auto] {
                    t += &format!(", party {}", if self.drivers.len() > 1 { "both" } else { self.drivers[0].id() });
                }
                if self.counts {
                    t += ", counts on";
                }
                if !self.armor.is_empty() {
                    t += &format!(", armor {}", self.armor.iter().map(|(u, n)| format!("{u}:{n}")).collect::<Vec<_>>().join(","));
                }
                if !self.tune.is_empty() {
                    t += &format!(", tune {}", self.tune.iter().map(|(k, v)| format!("{k}:{v}")).collect::<Vec<_>>().join(","));
                }
            }
            _ => {}
        }
        if !self.pouch.is_empty() {
            t += &format!(", pouch {}", self.pouch.join(","));
        }
        if self.stage > 0 {
            t += &format!(", stage {}", self.stage);
        }
        if let Some(n) = self.durability {
            t += &format!(", durability {n}");
        }
        if self.dulled_lock {
            t += ", dulled lock";
        }
        if let Some(n) = self.mastery {
            t += &format!(", mastery {n}");
        }
        t
    }
}

/// RULESET1-G2: the unit refs an encounter fights with (party, enemies, guests and phase waves).
pub fn feel_roster(d: &Data, id: &str) -> Vec<String> {
    let Some(row) = d.cat.encounters.get(id) else { return Vec::new() };
    let mut refs: Vec<String> = ["party", "enemies", "guests"].iter().flat_map(|k| aegis_core::json::sget(row, k)).collect();
    refs.extend(rpgkit_battle::encounter::Phases::create(aegis_core::json::aget(row, "phases")).phases.iter().flat_map(|p| p.wave_refs()));
    refs
}

/// RULESET1-G2: the `tune=` keys: the kit schema's numeric ruleset fields with a TUNE_PREFIXES prefix.
pub fn tune_keys() -> Vec<String> {
    tune_fields().map(|f| aegis_core::json::s(f, "key")).chain(CHANNEL_COST_KEYS.iter().map(|k| k.to_string())).collect()
}

/// RULESET1-G2: the schema range (min, max) of the `tune=` number `key` (None: not one). PR5-26: a `channel_cost` key's
/// range is 0..`CHANNEL_COST_MAX` (the row check's).
pub fn tune_range(key: &str) -> Option<(f64, f64)> {
    if CHANNEL_COST_KEYS.contains(&key) {
        return Some((0.0, CHANNEL_COST_MAX));
    }
    let f = tune_fields().find(|f| aegis_core::json::s(f, "key") == key)?;
    Some((f.get("min")?.as_f64()?, f.get("max")?.as_f64()?))
}

fn tune_fields() -> impl Iterator<Item = &'static serde_json::Map<String, serde_json::Value>> {
    let fields = rpgkit_content::source::schema()["content"]["rulesets"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    fields.iter().filter_map(serde_json::Value::as_object).filter(|f| {
        let key = aegis_core::json::s(f, "key");
        matches!(aegis_core::json::s(f, "type").as_str(), "int" | "float") && TUNE_PREFIXES.iter().any(|p| key.starts_with(p))
    })
}

/// The help text printed after an argument error; the campaign line is the game's story run's (KIT1-SIM).
pub fn usage(game: &SimGame) -> Vec<String> {
    let campaign = game.story.as_ref().map_or(NO_STORY_USAGE, |s| s.usage.as_str());
    let world = format!("       world=<id> ({}): bare enc= ids also find that world's encounters (story characters with their extras and tuning)", game.default_world);
    // KITIDS1: the class defaults are the game's
    let c = &game.classes;
    let or_none = |s: String| if s.is_empty() { "none".to_string() } else { s };
    let fill = |l: &str| {
        l.replace("{comp}", &or_none(c.default_comp.join(",")))
            .replace("{elite}", &or_none(c.elite.into()))
            .replace("{boss}", &or_none(c.boss.into()))
            .replace("{lead}", &or_none(c.trio_lead.into()))
            .replace("{trios}", &crate::campaign::trios(c).len().to_string())
    };
    let mut out: Vec<String> = USAGE.iter().map(|l| fill(l)).collect();
    out.insert(USAGE_CAMPAIGN_AT, campaign.to_string());
    out.insert(USAGE_WORLD_AT, world);
    out
}

/// The usage `campaign` line of a game without a story run.
const NO_STORY_USAGE: &str = "       campaign [level]   the story fights of the game's story run (this game has none)";
/// Where usage() inserts the campaign mode line and the world= key line into USAGE.
const USAGE_CAMPAIGN_AT: usize = 9;
const USAGE_WORLD_AT: usize = 23;

const USAGE: [&str; 30] = [
    "usage: tools/fc.sh sim <mode> [level] [seeds] [key=value ...]   (seed i = 11 + 12 i; level default 5, seeds 3; party / sweep 10)",
    "modes: duel [level]       class round robin, hero vs hero (bin/hp matrix, row averages)",
    "       tier [level]       hero vs its class (or tier_foe=) at minion/standard/elite/boss, 1v1 and 1v3",
    "       levels             class at L vs L+2 for L = 1, 3, 5, 7",
    "       group [level]      hero L vs 3 minions at L-2, 3 standard at L-2, 3 minions at L",
    "       stats [level|all]  no fights: HP, armor, EHP, power, range, initiative, move rate, crit, active families",
    "       party [level]      comp (1..PARTY_MAX classes) vs generated mob/line/wave/elite/boss or catalog encounters",
    "       role [level]       limit tests per class: tank_hold, ranged_dive, aoe_swarm, support_attrition, striker_priority",
    "       sweep [level]      every size-class composition (size=4; lead= fixes one class) vs enc: ranking, swap delta, top-5 appearances",
    "       feel [level]       AI-vs-AI catalog encounters with their authored party (enc=, default the game's feel encounters; timeline; seeds 10): outcomes, poise, hits, AP, FEEL_JSON",
    "       deadlines          no fights: best and worst day each campaign checkpoint can be met, and its slack (world=)",
    "keys:  cap=1..30 (10)  loadout=default|newest|role (default; party side)",
    "       spend=class|cons|primary|mixed|all (class; party side)  preset=<PlanPresets id>|both (balanced; every unit)",
    "       distance=default|2..6 (default; role mode 4)  score=both|hp|binary (both)  quick  metrics=on|off (on in party/role/sweep)",
    "       band=lo-hi (30-70): duel row hp averages outside it are flagged LOW/HIGH (diagnostic, not a gate)",
    "       comp=c1,c2,.. ({comp})  enc=<ids>|generated|all|world:<id>[,..] (generated)  elevel=+n|-n|catalog (+0)",
    "       elite=<class> ({elite})  boss=<class> ({boss})  role=<tank|ranged|aoe|support|striker|all>  class=<id|all>",
    "       dupes=on|off (off)  sample=<n> (sweep with dupes; RNG seed 7)  target=<key>:<lo>-<hi> (override a section 8 band)",
    "       heal=none|full|rally|<pct> (none; campaign: between-battle healing, full also revives, rally = HealRules 25% after a win, a downed hero gets up)  comp=.. picks one campaign party",
    "       rest=<b,b> (none): campaign story battles that start after a full rest  extra=<b>:<enc>[,..] (none): campaign extra fight(s) before story battle b (a loss there is a fall at b)",
    "       jitter=on|off (on): seeded start tiles (each unit up to 1 tile inside its half) and party order per seed",
    "       pouch=<id,id,id> (none): every party unit carries these consumables and may use them (TacticalAI use_items)",
    "       size=1..PARTY_MAX (4) lead=<class> (none): sweep compositions of that size holding the lead (size=3 lead={lead}: the {trios} trios)",
    "       tier_foe=mirror|<class> (mirror): tier mode foe class  phase=on|off (on): the generated boss's sim phase  s2=on|off (off): support_attrition S2 foes (authored since BAL5-R1b: no change)",
    "       jitter PLAN FAIL on an unreachable target (no path / too far) is a skipped fight (PLAN SKIP line), not a fail",
    "       ruleset=classic|timeline (classic; a world encounter: its world's)  rules=+id,-id (none): switch single extensions  melee_diagonal=none|maneuver|all (the row's)",
    "       rewards=on|off (off): party and campaign print what the defeated enemies pay (fates, XP, purse share, drops, key loot)",
    "       feel only: seeds=<n> (10)  tactic=balanced|all_out|focus_weakest|guard_counter|kite|economy (balanced)  party=auto|timed|both (auto)",
    "       feel only: armor=<unit>:<0-200>[,..] (none)  tune=<ruleset number>:<value>[,..] (none; stagger_, flank_, break_chance_, rest_ numbers)",
    "       deadlines only: losses=0..9 (1): lost tries per mission in the worst case",
];
