//! Balance sim arguments (tools/sim/sim_args.gd, BAL2_SIM.md section 1): positional mode, level, seeds, then
//! key=value tokens in any order.

use crate::targets::{self, Targets};
use emberwake_world::Data;
use indexmap::IndexMap;

pub const MODES: [&str; 9] = ["duel", "tier", "levels", "group", "stats", "party", "role", "sweep", "campaign"];
pub const CLASSES: [&str; 6] = ["warrior", "ranger", "priest", "brawler", "rogue", "mage"];
pub const SPENDS: [&str; 4] = ["class", "cons", "primary", "mixed"];
pub const LOADOUTS: [&str; 3] = ["default", "newest", "role"];
pub const SCORES: [&str; 3] = ["both", "hp", "binary"];
pub const GENERATED: [&str; 5] = ["mob", "line", "wave", "elite", "boss"];
pub const PRESETS: [&str; 7] = ["balanced", "aggressive", "focus_weakest", "defensive", "ranged_kite", "economy", "withdraw"];
pub const REF_PREFIX: &str = "world:";
pub const DEFAULT_WORLD: &str = "ashlands";
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
pub const DEFAULT_COMP: [&str; 4] = ["warrior", "ranger", "priest", "mage"];
const PARTY_MAX: usize = emberwake_world::data::PARTY_MAX as usize;
const POUCH_MAX: usize = emberwake_world::state::POUCH_MAX;
const MAX_LEVEL: i32 = emberwake_rules::stats::MAX_LEVEL;

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
    /// Not in the help text: the help is printed after an argument error, and those lines are Godot parity.
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
    pub world: String,
    pub world_set: bool,
    pub comp_size: usize,
    pub lead: String,
    pub seeds_set: bool,
    pub tier_foe: String,
    pub boss_phase: bool,
    pub s2: bool,
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
            comp: strings(&DEFAULT_COMP),
            comp_set: false,
            heal: "none".into(),
            rest: Vec::new(),
            extra: IndexMap::new(),
            encounters: strings(&GENERATED),
            elevel: 0,
            elevel_set: false,
            elite_class: "warrior".into(),
            boss_class: "brawler".into(),
            roles: strings(&ROLES),
            classes: strings(&CLASSES),
            dupes: false,
            sample: 0,
            jitter: true,
            targets: targets::defaults(),
            pouch: Vec::new(),
            world: DEFAULT_WORLD.into(),
            world_set: false,
            comp_size: 4,
            lead: String::new(),
            seeds_set: false,
            tier_foe: MIRROR.into(),
            boss_phase: true,
            s2: false,
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

/// Every encounter of the catalog, in catalog order.
pub fn catalog_encounters(d: &Data) -> Vec<String> {
    d.cat.encounters.keys().cloned().collect()
}

impl SimArgs {
    /// Parses the sim's command line (mode first). Err = the message after "SIM argument error: ".
    pub fn parse(d: &Data, argv: &[String]) -> Result<Self, String> {
        let mut a = SimArgs::default();
        let mut positional: Vec<String> = Vec::new();
        let mut metrics_set = false;
        let mut distance_set = false;
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
            let Some((key, value)) = token.split_once('=') else {
                positional.push(token.to_string());
                continue;
            };
            a.key(d, key, value)?;
            metrics_set |= key == "metrics";
            distance_set |= key == "distance";
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
        } else if a.mode == "party" || a.mode == "sweep" {
            a.seed_count = PARTY_SEEDS;
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
        Ok(a)
    }

    fn key(&mut self, d: &Data, key: &str, value: &str) -> Result<(), String> {
        let classes = godot_list(&CLASSES);
        match key {
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
                    if !CLASSES.contains(&c.as_str()) {
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
                        if !is_int(&at) || int(&at) < 1 || enc.is_empty() || !has_encounter(d, DEFAULT_WORLD, &enc) {
                            return Err(format!("extra must be <battle>:<{DEFAULT_WORLD} encounter>[,..], e.g. extra=4:ash_raiders (got '{part}')"));
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
            "elite" | "boss" => {
                if !CLASSES.contains(&value) {
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
                    self.classes = strings(&CLASSES);
                } else if CLASSES.contains(&value) {
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
                if ids.is_empty() || ids.len() > POUCH_MAX {
                    return Err(format!("pouch must list 1..{POUCH_MAX} consumable ids (got '{value}')"));
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
                if !CLASSES.contains(&value) {
                    return Err(format!("lead must be a class id of {classes} (got '{value}')"));
                }
                self.lead = value.into();
            }
            "tier_foe" => {
                if value != MIRROR && !CLASSES.contains(&value) {
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
                let comp = if self.comp_set { self.comp.join(",") } else { "all trios (warrior + 2 of ranger,priest,brawler,rogue,mage)".into() };
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
            _ => {}
        }
        if !self.pouch.is_empty() {
            t += &format!(", pouch {}", self.pouch.join(","));
        }
        t
    }
}

pub fn usage() -> Vec<String> {
    strings(&[
        "usage: tools/fc.sh sim <mode> [level] [seeds] [key=value ...]   (seed i = 11 + 12 i; level default 5, seeds 3; party / sweep 10)",
        "modes: duel [level]       class round robin, hero vs hero (bin/hp matrix, row averages)",
        "       tier [level]       hero vs its class (or tier_foe=) at minion/standard/elite/boss, 1v1 and 1v3",
        "       levels             class at L vs L+2 for L = 1, 3, 5, 7",
        "       group [level]      hero L vs 3 minions at L-2, 3 standard at L-2, 3 minions at L",
        "       stats [level|all]  no fights: HP, armor, EHP, power, range, speed, crit, active families",
        "       party [level]      comp (1..PARTY_MAX classes) vs generated mob/line/wave/elite/boss or catalog encounters",
        "       role [level]       limit tests per class: tank_hold, ranged_dive, aoe_swarm, support_attrition, striker_priority",
        "       sweep [level]      every size-class composition (size=4; lead= fixes one class) vs enc: ranking, swap delta, top-5 appearances",
        "       campaign [level]   Ashlands story fights in quest order, HP carried over (level = party level, default 5): all 10 trios, both steel choices",
        "keys:  cap=1..30 (10)  loadout=default|newest|role (default; party side)",
        "       spend=class|cons|primary|mixed|all (class; party side)  preset=<PlanPresets id>|both (balanced; every unit)",
        "       distance=default|2..6 (default; role mode 4)  score=both|hp|binary (both)  quick  metrics=on|off (on in party/role/sweep)",
        "       band=lo-hi (30-70): duel row hp averages outside it are flagged LOW/HIGH (diagnostic, not a gate)",
        "       comp=c1,c2,.. (warrior,ranger,priest,mage)  enc=<ids>|generated|all|world:<id>[,..] (generated)  elevel=+n|-n|catalog (+0)",
        "       elite=<class> (warrior)  boss=<class> (brawler)  role=<tank|ranged|aoe|support|striker|all>  class=<id|all>",
        "       dupes=on|off (off)  sample=<n> (sweep with dupes; RNG seed 7)  target=<key>:<lo>-<hi> (override a section 8 band)",
        "       heal=none|full|rally|<pct> (none; campaign: between-battle healing, full also revives, rally = HealRules 25% after a win, a downed hero gets up)  comp=.. picks one campaign party",
        "       rest=<b,b> (none): campaign story battles that start after a full rest  extra=<b>:<enc>[,..] (none): campaign extra fight(s) before story battle b (a loss there is a fall at b)",
        "       jitter=on|off (on): seeded start tiles (each unit up to 1 tile inside its half) and party order per seed",
        "       pouch=<id,id,id> (none): every party unit carries these consumables and may use them (TacticalAI use_items)",
        "       world=<id> (ashlands): bare enc= ids also find that world's encounters (story characters with their extras and tuning)",
        "       size=1..PARTY_MAX (4) lead=<class> (none): sweep compositions of that size holding the lead (size=3 lead=warrior: the 10 trios)",
        "       tier_foe=mirror|<class> (mirror): tier mode foe class  phase=on|off (on): the generated boss's sim phase  s2=on|off (off): support_attrition S2 foes (authored since BAL5-R1b: no change)",
        "       jitter PLAN FAIL on an unreachable target (no path / too far) is a skipped fight (PLAN SKIP line), not a fail",
    ])
}
