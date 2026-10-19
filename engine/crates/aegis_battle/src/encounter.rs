//! The encounter rules a battle runs under (scripts/combat/session/): EncounterObjectives (rout, hold, escape, slay,
//! guests and protect), EncounterPhases (boss phases: hp_below and round triggers, line / wave / status / ai /
//! enrage effects) and EncounterMorale (enemy groups that break and surrender). Runtime parts only; validation stays
//! with the content tools. The session calls them at the same points CatalogEncounter does.

use crate::grid::V2;
use crate::world::{AiProfile, Role, TargetPref, UnitId, World};
use aegis_data::value::{self, Obj};
use indexmap::IndexMap;
use serde_json::Value;

pub const WON: &str = "won";
pub const LOST: &str = "lost";
pub const FLED: &str = "fled";

// ---- Objectives ----------------------------------------------------------------------------------------------

pub const RING: [V2; 2] = [V2 { x: 5, y: 4 }, V2 { x: 6, y: 4 }];
pub const MAX_GUESTS: usize = 2;

#[derive(Debug, Clone, Default)]
pub struct Objectives {
    pub objective: String,
    pub tiles: Vec<V2>,
    pub rounds: i32,
    pub unit_ref: String,
    pub count: i32,
    /// Hold rounds counted so far.
    pub held: i32,
    /// The reason of a verdict that needs one ("X got away.").
    pub note: String,
}

impl Objectives {
    pub fn create(o: &Obj) -> Self {
        let raw = value::string(o, "objective", "rout");
        let objective = if ["rout", "hold", "escape", "slay"].contains(&raw.as_str()) { raw } else { "rout".into() };
        let mut tiles: Vec<V2> = value::array(o, "objective_tiles")
            .iter()
            .filter_map(|p| p.as_array().filter(|p| p.len() == 2).map(|p| V2::new(value::to_int(&p[0], 0), value::to_int(&p[1], 0))))
            .collect();
        if tiles.is_empty() {
            tiles = RING.to_vec();
        }
        Self {
            objective,
            tiles,
            rounds: value::int(o, "objective_rounds", 3).max(1),
            unit_ref: value::string(o, "objective_unit", ""),
            count: value::int(o, "objective_count", 1).clamp(1, 4),
            held: 0,
            note: String::new(),
        }
    }

    /// Rule 3: "", won, lost or fled after a block (first match wins). Hold counting happens at a round end.
    pub fn verdict(&mut self, w: &World, heroes: &[UnitId], guests: &[UnitId], protect: &[UnitId], foes: &[UnitId], end_of_round: bool, target: Option<UnitId>) -> String {
        self.note.clear();
        let u = |c: UnitId| &w.units[c];
        let heroes_down = heroes.iter().all(|&h| !u(h).is_alive());
        let hero_active = heroes.iter().any(|&h| u(h).is_active());
        let withdrew = heroes.iter().filter(|&&h| u(h).is_alive() && u(h).is_withdrawn).count() as i32;
        if protect.iter().any(|&g| !u(g).is_alive()) {
            return LOST.into();
        }
        if heroes_down {
            return LOST.into();
        }
        if !foes.iter().any(|&e| u(e).is_active()) {
            return WON.into();
        }
        match self.objective.as_str() {
            "hold" if end_of_round => {
                let party: Vec<UnitId> = heroes.iter().chain(guests).copied().collect();
                let ours = self.on_tiles(w, &party);
                let theirs = self.on_tiles(w, foes);
                if ours && !theirs {
                    self.held += 1;
                } else if theirs && !ours {
                    self.held = (self.held - 1).max(0);
                }
                if self.held >= self.rounds {
                    return WON.into();
                }
            }
            "escape" => {
                let protect_out = protect.iter().all(|&g| u(g).is_withdrawn);
                if withdrew >= self.count && !hero_active && protect_out {
                    return WON.into();
                }
            }
            "slay" => {
                if let Some(t) = target {
                    if !u(t).is_alive() {
                        return WON.into();
                    }
                    if u(t).is_withdrawn {
                        self.note = format!("{} got away.", u(t).name);
                        return FLED.into();
                    }
                }
            }
            _ => {}
        }
        if !hero_active && withdrew > 0 {
            return FLED.into();
        }
        String::new()
    }

    /// Rule 9: the objective line: `objective_text` when set, else the generated text. `name_of(ref)` names units.
    pub fn text_of(o: &Obj, name_of: impl Fn(&str) -> String) -> String {
        let authored = value::string(o, "objective_text", "").trim().to_string();
        if !authored.is_empty() {
            return authored;
        }
        let mut text = match value::string(o, "objective", "rout").as_str() {
            "hold" => format!("Hold the ring for {} rounds, or defeat every enemy.", value::int(o, "objective_rounds", 3)),
            "escape" => format!("Get {} out by the exit, or defeat every enemy.", value::int(o, "objective_count", 1)),
            "slay" => format!("Defeat {}.", name_of(&value::string(o, "objective_unit", ""))),
            _ => "Defeat every enemy.".to_string(),
        };
        let names: Vec<String> = value::strings(o, "protect").iter().map(|r| name_of(r)).collect();
        if !names.is_empty() {
            text += &format!(" Keep {} alive.", names.join(" and "));
        }
        text
    }

    fn on_tiles(&self, w: &World, units: &[UnitId]) -> bool {
        units.iter().any(|&c| w.units[c].is_active() && w.grid.positions[c].is_some() && self.tiles.contains(&w.pos(c)))
    }
}

// ---- Phases --------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PhaseDef {
    pub id: String,
    pub name: String,
    /// kind, unit, pct, or_round | round
    pub trigger: Obj,
    /// Parsed effects with their defaults.
    pub effects: Vec<Obj>,
}

impl PhaseDef {
    pub fn effects_of<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a Obj> + 'a {
        self.effects.iter().filter(move |fx| value::string(fx, "type", "") == kind)
    }

    pub fn wave_refs(&self) -> Vec<String> {
        self.effects_of("wave").flat_map(|fx| value::strings(fx, "units")).collect()
    }
}

/// EncounterPhases.parse: typed phases with defaults (status rounds 2, enrage power 20).
pub fn parse_phases(raw: &[Value]) -> Vec<PhaseDef> {
    raw.iter()
        .filter_map(Value::as_object)
        .map(|src| {
            let t = value::obj(src, "trigger").cloned().unwrap_or_default();
            let kind = value::string(&t, "kind", "");
            let mut trigger = Obj::new();
            trigger.insert("kind".into(), Value::from(kind.clone()));
            if kind == "hp_below" {
                trigger.insert("unit".into(), Value::from(value::string(&t, "unit", "")));
                trigger.insert("pct".into(), Value::from(value::int(&t, "pct", 0)));
                trigger.insert("or_round".into(), Value::from(value::int(&t, "or_round", 0)));
            } else if kind == "round" {
                trigger.insert("round".into(), Value::from(value::int(&t, "round", 0)));
            }
            let effects = value::array(src, "effects")
                .iter()
                .filter_map(Value::as_object)
                .map(|fx| {
                    let ty = value::string(fx, "type", "");
                    let mut out = Obj::new();
                    out.insert("type".into(), Value::from(ty.clone()));
                    match ty.as_str() {
                        "line" => {
                            out.insert("text".into(), Value::from(value::string(fx, "text", "")));
                        }
                        "wave" => {
                            out.insert("units".into(), Value::from(value::strings(fx, "units")));
                            out.insert("label".into(), Value::from(value::string(fx, "label", "")));
                        }
                        "status" => {
                            out.insert("unit".into(), Value::from(value::string(fx, "unit", "")));
                            out.insert("status".into(), Value::from(value::string(fx, "status", "")));
                            out.insert("rounds".into(), Value::from(value::int(fx, "rounds", 2)));
                            out.insert("magnitude".into(), Value::from(value::int(fx, "magnitude", 0)));
                        }
                        "ai" => {
                            out.insert("unit".into(), Value::from(value::string(fx, "unit", "")));
                            for key in ["role", "target_pref", "reckless", "flee_ratio", "defend_below", "heal_below"] {
                                if let Some(v) = fx.get(key) {
                                    out.insert(key.into(), v.clone());
                                }
                            }
                        }
                        "enrage" => {
                            out.insert("unit".into(), Value::from(value::string(fx, "unit", "")));
                            out.insert("power".into(), Value::from(value::int(fx, "power", 20)));
                        }
                        _ => {}
                    }
                    out
                })
                .collect();
            PhaseDef { id: value::string(src, "id", ""), name: value::string(src, "name", ""), trigger, effects }
        })
        .collect()
}

#[derive(Debug, Clone, Default)]
pub struct Phases {
    pub phases: Vec<PhaseDef>,
    /// id -> (round, block), fire order.
    pub fired: IndexMap<String, (i32, i32)>,
    pub due: Vec<String>,
    pub dropped: Vec<String>,
    pub wave_units: IndexMap<String, Vec<UnitId>>,
    reserved: IndexMap<UnitId, String>,
    base_profiles: IndexMap<UnitId, AiProfile>,
}

impl Phases {
    pub fn create(raw: &[Value]) -> Self {
        Self { phases: parse_phases(raw), ..Default::default() }
    }

    pub fn phase(&self, id: &str) -> Option<&PhaseDef> {
        self.phases.iter().find(|p| p.id == id)
    }

    pub fn reserve(&mut self, id: &str, c: UnitId) {
        self.wave_units.entry(id.to_string()).or_default().push(c);
        self.reserved.insert(c, id.to_string());
    }

    pub fn is_reserved(&self, c: UnitId) -> bool {
        self.reserved.get(&c).is_some_and(|id| !self.fired.contains_key(id))
    }

    pub fn units_of(&self, id: &str) -> Vec<UnitId> {
        self.wave_units.get(id).cloned().unwrap_or_default()
    }

    /// Rule 1: hp_below phases become due (or drop when their unit is out) after a block.
    pub fn check_block_end(&mut self, w: &World, unit_of: impl Fn(&str) -> Option<UnitId>) {
        for p in &self.phases {
            if value::string(&p.trigger, "kind", "") != "hp_below" || self.fired.contains_key(&p.id) || self.dropped.contains(&p.id) || self.due.contains(&p.id) {
                continue;
            }
            match unit_of(&value::string(&p.trigger, "unit", "")) {
                Some(c) if w.units[c].is_active() => {
                    let u = &w.units[c];
                    if u.hp * 100 <= value::int(&p.trigger, "pct", 0) * u.max_hp {
                        self.due.push(p.id.clone());
                    }
                }
                _ => self.dropped.push(p.id.clone()),
            }
        }
    }

    /// Round phases due at their round start; hp_below phases at their or_round.
    pub fn mark_round(&mut self, round: i32) {
        for p in &self.phases {
            if self.fired.contains_key(&p.id) || self.dropped.contains(&p.id) || self.due.contains(&p.id) {
                continue;
            }
            let kind = value::string(&p.trigger, "kind", "");
            if (kind == "round" && value::int(&p.trigger, "round", 0) == round) || (kind == "hp_below" && value::int(&p.trigger, "or_round", 0) == round) {
                self.due.push(p.id.clone());
            }
        }
    }

    fn applies(at: (i32, i32), round: i32) -> bool {
        at.0 < round || (at.0 == round && at.1 == 0)
    }

    pub fn enraged_refs(&self, round: i32) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for (id, at) in &self.fired {
            if Self::applies(*at, round) {
                if let Some(p) = self.phase(id) {
                    for fx in p.effects_of("enrage") {
                        let u = value::string(fx, "unit", "");
                        if !out.contains(&u) {
                            out.push(u);
                        }
                    }
                }
            }
        }
        out
    }

    pub fn next_due(&self) -> Option<String> {
        self.phases.iter().find(|p| self.due.contains(&p.id)).map(|p| p.id.clone())
    }

    pub fn mark_fired(&mut self, id: &str, round: i32, block: i32) {
        self.due.retain(|d| d != id);
        self.fired.insert(id.to_string(), (round, block));
    }

    /// Rule 5: live profiles = base + the ai settings of every phase fired before `round` (or at its start).
    pub fn apply_profiles(&mut self, profiles: &mut IndexMap<UnitId, AiProfile>, units_of_ref: impl Fn(&str) -> Vec<UnitId>, round: i32) {
        let mut settings: Vec<Obj> = Vec::new();
        for (id, at) in &self.fired {
            if Self::applies(*at, round) {
                if let Some(p) = self.phase(id) {
                    settings.extend(p.effects_of("ai").cloned());
                }
            }
        }
        if settings.is_empty() && self.base_profiles.is_empty() {
            return;
        }
        for fx in &settings {
            for c in units_of_ref(&value::string(fx, "unit", "")) {
                if !self.base_profiles.contains_key(&c) {
                    if let Some(p) = profiles.get(&c) {
                        self.base_profiles.insert(c, p.clone());
                    }
                }
            }
        }
        for (c, base) in &self.base_profiles {
            profiles.insert(*c, base.clone());
        }
        for fx in &settings {
            for c in units_of_ref(&value::string(fx, "unit", "")) {
                if let Some(p) = profiles.get_mut(&c) {
                    apply_ai(p, fx);
                }
            }
        }
    }
}

/// EncounterPhases.apply_ai.
pub fn apply_ai(p: &mut AiProfile, fx: &Obj) {
    if let Some(r) = fx.get("role") {
        match value::to_str(r).as_str() {
            "AGGRESSOR" => p.role = Role::Aggressor,
            "SKIRMISHER" => p.role = Role::Skirmisher,
            "SNIPER" => p.role = Role::Sniper,
            "SUPPORT" => p.role = Role::Support,
            _ => {}
        }
    }
    if let Some(t) = fx.get("target_pref") {
        p.target_pref = if value::to_str(t) == "WEAKEST" { TargetPref::Weakest } else { TargetPref::Nearest };
    }
    if let Some(v) = fx.get("reckless") {
        p.reckless = v.as_bool().unwrap_or_else(|| value::to_float(v, 0.0) != 0.0);
    }
    if let Some(v) = fx.get("flee_ratio") {
        p.flee_ratio = value::to_float(v, 0.0);
    }
    if let Some(v) = fx.get("defend_below") {
        p.defend_below = value::to_float(v, 0.0);
    }
    if let Some(v) = fx.get("heal_below") {
        p.heal_below = value::to_float(v, 0.0);
    }
}

// ---- Morale --------------------------------------------------------------------------------------------------

pub const START: i32 = 100;
pub const CRIT_HIT: i32 = 10;
pub const CRIT_KILL_EXTRA: i32 = 10;
pub const LEADER_DOWN: i32 = 40;
pub const LEADER_TIERS: [&str; 2] = ["boss", "elite"];

#[derive(Debug, Clone)]
pub struct Group {
    pub members: Vec<String>,
    pub size: i32,
    pub value: i32,
    pub broken: bool,
    pub announced: bool,
    pub line: String,
    pub source: String,
}

#[derive(Debug, Clone, Default)]
pub struct Morale {
    pub on: bool,
    pub brk: i32,
    pub text: String,
    pub groups: Vec<Group>,
    pub leader: String,
    pub leader_down: bool,
    pub surrendered: Vec<String>,
    pub down: Vec<String>,
    pub pending: Vec<(String, bool)>,
}

/// The starting groups as enemy indices: `morale_groups`, or one group of every enemy.
pub fn groups_of(o: &Obj) -> Vec<Vec<i32>> {
    let raw = value::array(o, "morale_groups");
    if raw.is_empty() {
        return vec![(0..value::strings(o, "enemies").len() as i32).collect()];
    }
    raw.iter().map(|g| g.as_array().map(|a| a.iter().map(|x| value::to_int(x, 0)).collect()).unwrap_or_default()).collect()
}

pub fn state_for(v: i32, b: i32) -> &'static str {
    let span = 100 - b;
    if v <= b {
        "broken"
    } else if v <= b + span / 3 {
        "wavering"
    } else if v <= b + (2 * span) / 3 {
        "shaken"
    } else {
        "steady"
    }
}

pub fn flee_for(base: f64, state: &str) -> f64 {
    let bonus = match state {
        "shaken" => 0.15,
        "wavering" => 0.30,
        _ => 0.0,
    };
    if bonus <= 0.0 || base <= 0.0 {
        return base;
    }
    base.max((base + bonus).min(0.60))
}

impl Morale {
    /// `starting`: keys of the starting enemies in `enemies` order.
    pub fn create(o: &Obj, starting: &[String], leader_key: &str) -> Self {
        let mut m = Morale { on: value::boolean(o, "morale", false), ..Default::default() };
        if !m.on {
            return m;
        }
        m.brk = value::int(o, "morale_break", 50).clamp(0, 90);
        m.text = value::string(o, "morale_text", "").trim().to_string();
        m.leader = leader_key.to_string();
        for idx in groups_of(o) {
            let members: Vec<String> = idx.iter().filter(|&&n| n >= 0 && (n as usize) < starting.len() && !starting[n as usize].is_empty()).map(|&n| starting[n as usize].clone()).collect();
            let size = members.len() as i32;
            m.form_group(&members, size, "start");
        }
        m
    }

    pub fn form_group(&mut self, members: &[String], size: i32, source: &str) {
        let mut keys: Vec<String> = Vec::new();
        for k in members {
            if !k.is_empty() && !keys.contains(k) {
                keys.push(k.clone());
            }
        }
        if !self.on || keys.is_empty() {
            return;
        }
        self.groups.push(Group { members: keys, size: size.max(1), value: START, broken: false, announced: false, line: String::new(), source: source.to_string() });
    }

    pub fn has_group(&self, source: &str) -> bool {
        self.groups.iter().any(|g| g.source == source)
    }

    pub fn group_of(&self, key: &str) -> i32 {
        self.groups.iter().position(|g| g.members.iter().any(|m| m == key)).map(|i| i as i32).unwrap_or(-1)
    }

    pub fn group_state(&self, i: i32) -> &'static str {
        if i < 0 || i as usize >= self.groups.len() {
            return "";
        }
        let g = &self.groups[i as usize];
        if g.broken {
            "broken"
        } else {
            state_for(g.value, self.brk)
        }
    }

    pub fn record_crit(&mut self, key: &str, killed: bool) {
        if self.on && self.group_of(key) >= 0 {
            self.pending.push((key.to_string(), killed));
        }
    }

    /// Rules 6 and 7 at a block end; returns the units that surrender now.
    pub fn block_end(&mut self, w: &World, unit_of: impl Fn(&str) -> Option<UnitId>, fearless: impl Fn(UnitId) -> bool, reserved: impl Fn(UnitId) -> bool) -> Vec<UnitId> {
        let mut out = Vec::new();
        if !self.on {
            return out;
        }
        let mut deltas = vec![0; self.groups.len()];
        let mut crit_killed: Vec<String> = Vec::new();
        for (key, killed) in std::mem::take(&mut self.pending) {
            let gi = self.group_of(&key);
            if gi < 0 {
                continue;
            }
            if killed {
                crit_killed.push(key);
            } else {
                deltas[gi as usize] -= CRIT_HIT;
            }
        }
        let mut leader_fell = false;
        for gi in 0..self.groups.len() {
            let size = self.groups[gi].size.max(1);
            for key in self.groups[gi].members.clone() {
                if self.down.contains(&key) || self.surrendered.contains(&key) {
                    continue;
                }
                let Some(c) = unit_of(&key) else { continue };
                if w.units[c].is_active() || reserved(c) {
                    continue;
                }
                self.down.push(key.clone());
                deltas[gi] -= (100 + size - 1) / size + if crit_killed.contains(&key) { CRIT_KILL_EXTRA } else { 0 };
                if key == self.leader && !self.leader_down {
                    leader_fell = true;
                }
            }
        }
        if leader_fell {
            self.leader_down = true;
            for d in deltas.iter_mut() {
                *d -= LEADER_DOWN;
            }
        }
        let mut first_break = !self.groups.iter().any(|g| g.broken);
        let leader_group = if self.leader.is_empty() { -1 } else { self.group_of(&self.leader) };
        for gi in 0..self.groups.len() {
            if self.groups[gi].broken {
                continue;
            }
            self.groups[gi].value = (self.groups[gi].value + deltas[gi]).clamp(0, START);
            if self.groups[gi].value > self.brk {
                continue;
            }
            self.groups[gi].broken = true;
            let mut names = Vec::new();
            for key in self.groups[gi].members.clone() {
                let Some(c) = unit_of(&key) else { continue };
                if !w.units[c].is_active() || reserved(c) || fearless(c) {
                    continue;
                }
                self.surrendered.push(key);
                names.push(w.units[c].name.clone());
                out.push(c);
            }
            if names.is_empty() {
                self.groups[gi].announced = true;
            } else {
                let leader_cause = leader_fell && gi as i32 == leader_group;
                self.groups[gi].line = if first_break && !self.text.is_empty() {
                    self.text.clone()
                } else if leader_cause {
                    format!("{} falls. The rest throw down their arms.", unit_of(&self.leader).map(|c| w.units[c].name.clone()).unwrap_or_else(|| self.leader.clone()))
                } else if self.groups.len() == 1 {
                    "The rest throw down their arms.".into()
                } else {
                    format!("{} throw down their arms.", names.join(" and "))
                };
            }
            first_break = false;
        }
        out
    }

    /// Break lines not logged yet (at most `limit`; 0 = all).
    pub fn take_lines(&mut self, limit: usize) -> Vec<String> {
        let mut out = Vec::new();
        for g in self.groups.iter_mut() {
            if g.announced || g.line.is_empty() {
                continue;
            }
            if limit > 0 && out.len() >= limit {
                break;
            }
            g.announced = true;
            out.push(g.line.clone());
        }
        out
    }
}
