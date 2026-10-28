//! Campaign meters (CAMPAIGN1 M1-M8, kit hook; engine-only, no Godot original): a meter row read with its defaults
//! (`MeterDef`), a meter's saved state (`MeterState`), its value and band, the floor of an act, which rules a move
//! crosses and which inputs a cause moves. Pure: the tick in `campaign/mod.rs` applies what these return.

use aegis_core::json::{self, int, Dict};
use rpgkit_rules::bands;
use serde_json::Value;

/// One band of a meter (M2): the value's band is the last band whose `from` is at or below it.
#[derive(Debug, Clone, PartialEq)]
pub struct Band {
    pub id: String,
    pub name: String,
    pub from: i64,
    /// One of `rpgkit_data::METER_TONES`.
    pub tone: String,
}

/// One meter rule (M5): fires when a move crosses `at` in `dir`.
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub id: String,
    pub at: i64,
    /// True = `up` (old below `at`, new at or above); false = `down` (old above, new at or below).
    pub up: bool,
    /// Empty = every act.
    pub acts: Vec<i64>,
    /// Fires at most once per run (recorded in `fired`); default false, so a threshold warns each time it is crossed
    /// and a soft countdown's rule fires at every restart.
    pub once: bool,
    pub warn: String,
    pub effects: Vec<Dict>,
    pub then: Vec<Dict>,
    /// Countdowns: run again with this many days left after the effects (K4; the days tick is CAMPAIGN1-C's).
    pub restart_days: Option<i64>,
}

/// One gauge input (M3).
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    /// One of `rpgkit_data::INPUT_ON`.
    pub on: String,
    /// Battle: the encounter carries one of these tags (empty = every battle).
    pub tags: Vec<String>,
    /// Battle: one of these outcomes (empty = every outcome).
    pub outcomes: Vec<String>,
    /// Leg: the route carries one of these tags (empty = every route).
    pub route_tags: Vec<String>,
    pub delta: i64,
    /// Empty = every act.
    pub acts: Vec<i64>,
}

/// A meter row (world kind `meters`, 11.2) read with the schema's defaults, so rows that skipped the loader's
/// normalisation read the same.
#[derive(Debug, Clone, PartialEq)]
pub struct MeterDef {
    pub id: String,
    pub qid: String,
    pub name: String,
    pub countdown: bool,
    pub min: i64,
    pub max: i64,
    pub start: i64,
    pub visible: bool,
    pub bands: Vec<Band>,
    pub rules: Vec<Rule>,
    pub inputs: Vec<Input>,
    /// (from_act, min), data order.
    pub floors: Vec<(i64, i64)>,
    /// The raw row (modifiers, hud, unit and description are read by CAMPAIGN1-C and -U).
    pub row: Dict,
}

fn ints(v: &Value) -> Vec<i64> {
    json::arr(v).iter().filter(|x| json::is_int(x)).map(json::to_int).collect()
}

fn dicts(v: &Value) -> Vec<Dict> {
    json::arr(v).iter().filter_map(|x| x.as_object().cloned()).collect()
}

impl MeterDef {
    pub fn from_row(e: &Dict) -> Self {
        let countdown = json::s(e, "kind") == "countdown";
        let bands: Vec<Band> = json::aget(e, "bands")
            .iter()
            .filter_map(Value::as_object)
            .map(|b| Band { id: json::s(b, "id"), name: json::s(b, "name"), from: json::i(b, "from", 0), tone: json::s_or(b, "tone", "calm") })
            .collect();
        let at_of = |raw: &Value| -> i64 {
            match raw {
                Value::String(id) => bands.iter().find(|b| &b.id == id).map(|b| b.from).unwrap_or(i64::MIN),
                v => json::to_int_or(v, i64::MIN),
            }
        };
        let rules = json::aget(e, "rules")
            .iter()
            .filter_map(Value::as_object)
            .map(|r| Rule {
                id: json::s(r, "id"),
                at: at_of(json::get(r, "at")),
                up: json::s_or(r, "dir", if countdown { "down" } else { "up" }) == "up",
                acts: ints(json::get(r, "acts")),
                once: json::b(r, "once", false),
                warn: json::s(r, "warn"),
                effects: dicts(json::get(r, "effects")),
                then: dicts(json::get(r, "then")),
                restart_days: r.get("restart_days").filter(|x| json::is_int(x)).map(json::to_int),
            })
            .filter(|r| r.at != i64::MIN)
            .collect();
        let inputs = json::aget(e, "inputs")
            .iter()
            .filter_map(Value::as_object)
            .map(|x| Input {
                on: json::s(x, "on"),
                tags: json::sget(x, "tags"),
                outcomes: json::sget(x, "outcomes"),
                route_tags: json::sget(x, "route_tags"),
                delta: json::i(x, "delta", 0),
                acts: ints(json::get(x, "acts")),
            })
            .collect();
        let floors = json::aget(e, "floors").iter().filter_map(Value::as_object).map(|f| (json::i(f, "from_act", 0), json::i(f, "min", 0))).collect();
        Self {
            id: json::s(e, "id"),
            qid: json::s(e, "_qid"),
            name: json::s_or(e, "name", "New Meter"),
            countdown,
            min: json::i(e, "min", 0),
            max: json::i(e, "max", 100),
            start: json::i(e, "start", 0),
            visible: json::b(e, "visible", false),
            bands,
            rules,
            inputs,
            floors,
            row: e.clone(),
        }
    }

    /// M4: the lowest value in `act`: the `min` of the floor with the latest `from_act` at or below the act (the
    /// last such row on a tie), else the meter's `min`. Countdowns have no floor.
    pub fn floor(&self, act: i64) -> i64 {
        let mut best: Option<(i64, i64)> = None;
        for &(from, lo) in &self.floors {
            if from <= act && best.is_none_or(|(f, _)| from >= f) {
                best = Some((from, lo));
            }
        }
        best.map_or(self.min, |(_, lo)| lo.clamp(self.min, self.max))
    }

    /// M4: a gauge value clamped to floor..max.
    pub fn clamp(&self, value: i64, act: i64) -> i64 {
        value.clamp(self.floor(act), self.max)
    }

    /// M2: the band holding `value` (None when the meter has no band at or below it; `rpgkit_rules::bands`, shared with
    /// the unit meters).
    pub fn band(&self, value: i64) -> Option<&Band> {
        bands::band_index(self.bands.iter().map(|b| b.from), value).map(|i| &self.bands[i])
    }

    /// M5: the rules a move from `old` to `new` crosses in `act`, in crossing order (rising points for a rise,
    /// falling for a fall; data order on a tie, K5; `rpgkit_rules::bands`).
    pub fn crossed(&self, old: i64, new: i64, act: i64) -> Vec<&Rule> {
        let mut out: Vec<&Rule> = self
            .rules
            .iter()
            .filter(|r| r.acts.is_empty() || r.acts.contains(&act))
            .filter(|r| bands::crosses(r.at, r.up, old, new))
            .collect();
        bands::crossing_order(&mut out, old, new, |r| r.at);
        out
    }

    /// M3: the summed delta of this gauge's inputs of `on` that match (`acts`, and `pick` for the cause's own filter).
    pub fn input_delta(&self, on: &str, act: i64, pick: impl Fn(&Input) -> bool) -> i64 {
        if self.countdown {
            return 0;
        }
        self.inputs.iter().filter(|x| x.on == on && (x.acts.is_empty() || x.acts.contains(&act)) && pick(x)).map(|x| x.delta).sum()
    }
}

/// Whether two tag lists meet (an empty `wanted` matches anything).
pub fn tags_match(wanted: &[String], have: &[String]) -> bool {
    wanted.is_empty() || wanted.iter().any(|t| have.contains(t))
}

/// The keys of a saved meter state (a gauge writes `value`, a countdown `due` and `running`).
pub const STATE_KEYS: [&str; 5] = ["value", "due", "running", "visible", "label"];

/// One meter's saved state (11.5): a gauge keeps `value`; a countdown keeps `due` (a time on the travel clock) and
/// `running`; both keep `visible` (M8) and the HUD `label` ("" = the meter's name; K3).
#[derive(Debug, Clone, PartialEq)]
pub struct MeterState {
    pub value: i64,
    pub due: i64,
    pub running: bool,
    pub visible: bool,
    pub label: String,
}

impl MeterState {
    /// A new game's state: the row's `start` and `visible`; a countdown stopped.
    pub fn fresh(def: &MeterDef) -> Self {
        Self { value: def.start.clamp(def.min, def.max), due: 0, running: false, visible: def.visible, label: String::new() }
    }

    /// M1: a gauge's value; a running countdown's days left at `time` (never below 0); a stopped countdown's `max`.
    pub fn value(&self, def: &MeterDef, time: i64) -> i64 {
        if !def.countdown {
            self.value
        } else if self.running {
            (self.due - time).max(0)
        } else {
            def.max
        }
    }

    pub fn to_value(&self, def: &MeterDef) -> Value {
        let mut o = if def.countdown {
            json::obj([("due", int(self.due)), ("running", Value::Bool(self.running))])
        } else {
            json::obj([("value", int(self.value))])
        };
        o.insert("visible".into(), Value::Bool(self.visible));
        o.insert("label".into(), Value::String(self.label.clone()));
        Value::Object(o)
    }

    /// From a saved entry (missing keys take the fresh state's); problems are pushed, never coerced silently.
    pub fn from_value(def: &MeterDef, raw: &Value, problems: &mut Vec<String>) -> Self {
        let mut st = Self::fresh(def);
        let Some(o) = raw.as_object() else {
            problems.push(format!("campaign.meters.{} must be an object.", def.id));
            return st;
        };
        let mut whole = |key: &str, slot: &mut i64| {
            if let Some(v) = o.get(key) {
                if json::is_int(v) {
                    *slot = json::to_int(v);
                } else {
                    problems.push(format!("campaign.meters.{}.{key} must be a whole number.", def.id));
                }
            }
        };
        whole("value", &mut st.value);
        whole("due", &mut st.due);
        let mut flag = |key: &str, slot: &mut bool| {
            if let Some(v) = o.get(key) {
                match v.as_bool() {
                    Some(b) => *slot = b,
                    None => problems.push(format!("campaign.meters.{}.{key} must be true or false.", def.id)),
                }
            }
        };
        flag("running", &mut st.running);
        flag("visible", &mut st.visible);
        match o.get("label") {
            None => {}
            Some(Value::String(t)) => st.label = t.clone(),
            Some(_) => problems.push(format!("campaign.meters.{}.label must be text.", def.id)),
        }
        for k in o.keys().filter(|k| !STATE_KEYS.contains(&k.as_str())) {
            problems.push(format!("campaign.meters.{}: unknown key '{k}'.", def.id));
        }
        st
    }
}
