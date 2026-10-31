//! CAMPAIGN1-SIM (CAMPAIGN1 C5, CAMPAIGN1_TASKS 4.3; engine-only, no Godot original, no fights): `sim deadlines`, the
//! best and worst day each campaign checkpoint can be met, and its slack, on the world's full travel graph.
//!
//! The model: the main quests are the world's quests tagged `main` (or on the `main` track), in data order; each
//! starts and ends at the defeat rule's `to` hub, and its node is its first step's location (else that step's
//! encounter's). The quests run as one chain from day 1 at the hub; each act's chain ends at the last quest one of its
//! checkpoints names. Best case: travel out, win, travel back unless the quest is its act's last. Worst case: travel
//! out, then `losses` times lose, retreat to the hub (`defeat.days`), rest (`rest.days`) and travel out again; win;
//! travel back unless it is the act's last; a rest after each win. A checkpoint is met on the day its quest is won
//! (C1: the win counts, not the turn-in); slack is the deadline less the worst day.
//!
//! Every number comes from the kit's rules: legs and their days from `travel::best_path` over `travel::all_legs` (the
//! world map's path search, route gates ignored), rest and retreat days from `campaign::purse::rest_rule` /
//! `defeat_rule` (a rest counts only where `where` allows it at the hub), the day from the campaign's read key
//! `campaign.day` and the checkpoints from `campaign::checkpoint::defs`. CAMPAIGN1-TM: in a world with travel modes
//! every leg is planned in the world's default mode (its `days` on each leg, `travel::best_path_with`), the mode a new
//! game travels in; the first line names it.

use crate::args::SimArgs;
use aegis_core::json::{self, Dict};
use rpgkit_world::campaign::{self, checkpoint, purse, Campaign};
use rpgkit_world::state::GameState;
use rpgkit_world::{explore, progression, travel, Data};

/// The quest tag (and track) that puts a quest on the main chain.
pub const MAIN: &str = "main";

/// A main quest on the chain.
#[derive(Debug, Clone, PartialEq)]
pub struct Mission {
    /// The quest's id (as the data writes it) and qid.
    pub id: String,
    pub qid: String,
    /// The location qid of its node ("" when the first step names none).
    pub node: String,
    /// Days from the hub to the node and back (None when the full graph does not connect them).
    pub out: Option<i64>,
    pub back: Option<i64>,
    /// Whether it ends its act's chain (no travel back).
    pub last_of_act: bool,
    /// The days it is won on, best and worst (None once the chain cannot go on).
    pub best: Option<i64>,
    pub worst: Option<i64>,
    /// The worst case's day once it is back at the hub and rested (its act's last: once rested at the node).
    pub worst_home: Option<i64>,
}

/// One checkpoint's line.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckpointLine {
    pub id: String,
    pub act: i64,
    pub quest: String,
    pub deadline: i64,
    /// (best day, worst day); Err: why the model has no day for it.
    pub days: Result<(i64, i64), String>,
}

impl CheckpointLine {
    /// The deadline less the worst day (None without days).
    pub fn slack(&self) -> Option<i64> {
        self.days.as_ref().ok().map(|(_, worst)| self.deadline - worst)
    }
}

/// The plan of one world.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub world: String,
    pub hub: String,
    pub rest_days: i64,
    pub retreat_days: i64,
    pub losses: i64,
    /// CAMPAIGN1-TM: the default travel mode the legs are planned in (name, days per leg); None without modes.
    pub mode: Option<(String, i64)>,
    pub missions: Vec<Mission>,
    pub checkpoints: Vec<CheckpointLine>,
}

/// The worlds `sim deadlines` looks at: the given world, else every campaign world with checkpoints, in world order.
pub fn worlds(d: &Data, a: &SimArgs) -> Vec<String> {
    if a.world_set {
        return vec![a.world.clone()];
    }
    d.world_ids().into_iter().filter(|w| campaign::is_on(d, w) && !checkpoint::defs(d, w).is_empty()).collect()
}

fn is_main(q: &Dict) -> bool {
    json::s(q, "track") == MAIN || json::aget(q, "tags").iter().any(|t| t.as_str() == Some(MAIN))
}

/// The node qid a location reference names in world `w` (as the travel graph names its endpoints; "" when unknown).
fn node_of(d: &Data, gs: &GameState, r: &str) -> String {
    if r.is_empty() {
        return String::new();
    }
    let q = json::s(explore::resolve_location(d, gs, r), "_qid");
    if q.is_empty() {
        return q;
    }
    json::s_or(d.resolve(&gs.world_id, "locations", &q), "_qid", &q)
}

/// A quest's node: its first step's location, else the location of that step's encounter.
fn quest_node(d: &Data, gs: &GameState, q: &Dict) -> String {
    let steps = json::aget(q, "steps");
    let Some(step) = steps.first().and_then(|s| s.as_object()) else { return String::new() };
    let at = node_of(d, gs, &json::s(step, "location"));
    if !at.is_empty() {
        return at;
    }
    let enc = json::s(step, "encounter");
    if enc.is_empty() {
        return String::new();
    }
    node_of(d, gs, &json::s(d.resolve(&gs.world_id, "encounters", &enc), "location"))
}

/// The campaign day at clock `time` (the read key `campaign.day` on a scratch state).
fn day_at(probe: &mut GameState, time: i64) -> i64 {
    probe.time = time;
    campaign::read_key(probe, "campaign.day").map_or(time + 1, |v| json::to_int(&v))
}

/// The plan of world `w` for `losses` lost tries per mission; Err: why the world has none (a line to print).
pub fn plan(d: &Data, w: &str, losses: i64) -> Result<Plan, String> {
    let cps = checkpoint::defs(d, w);
    if !campaign::is_on(d, w) || cps.is_empty() {
        return Err(format!("DEADLINES {w}: no campaign checkpoints"));
    }
    let gs = GameState { world_id: w.to_string(), campaign: Campaign::fresh(d, w, 0), ..Default::default() };
    let defeat = purse::defeat_rule(d, w);
    if defeat.mode != "retreat" || defeat.to.is_empty() {
        return Err(format!("DEADLINES {w}: the worst case needs a defeat rule `retreat` with a `to` hub (mode {})", defeat.mode));
    }
    let hub = node_of(d, &gs, &defeat.to);
    if hub.is_empty() {
        return Err(format!("DEADLINES {w}: the retreat hub '{}' is no location", defeat.to));
    }
    let rule = purse::rest_rule(d, w);
    let rests_at_hub = match rule.place.as_str() {
        "none" => false,
        "rest_places" => progression::is_rest_place(d, &gs, &hub),
        _ => true,
    };
    let rest_days = if rests_at_hub { (rule.days as i64).max(0) } else { 0 };
    let retreat_days = (defeat.days as i64).max(0);
    let legs = travel::all_legs(d, w);
    let nodes = travel::nodes(d, w);
    // CAMPAIGN1-TM: a fresh campaign travels in the default mode
    let mode = purse::travel_mode(d, &gs).map(|m| (m.name, (m.days as i64).max(0)));
    let extra = mode.as_ref().map_or(0, |m| m.1);
    let days = |from: &str, to: &str| -> Option<i64> {
        if from == to {
            return Some(0);
        }
        travel::best_path_with(&legs, &nodes, from, to, extra).map(|(t, _)| t)
    };

    let mains: Vec<&Dict> = d.merged_entries(w, "quests").into_iter().filter(|q| is_main(q)).collect();
    let index_of = |qid: &str| mains.iter().position(|q| json::s(q, "_qid") == qid);
    // each act's chain ends at the last main quest its checkpoints name
    let mut ends: Vec<usize> = Vec::new();
    let mut acts: Vec<i64> = cps.iter().map(|c| c.act).collect();
    acts.sort_unstable();
    acts.dedup();
    for act in &acts {
        let end = cps.iter().filter(|c| c.act == *act && !c.quest.is_empty()).filter_map(|c| index_of(&c.quest)).max();
        ends.extend(end);
    }
    let chain_len = ends.iter().max().map_or(0, |e| e + 1);

    let mut probe = gs.clone();
    let (mut best_t, mut worst_t): (Option<i64>, Option<i64>) = (Some(0), Some(0));
    let mut missions = Vec::new();
    for q in mains.iter().take(chain_len) {
        let qid = json::s(q, "_qid");
        let node = quest_node(d, &gs, q);
        let (out, back) = if node.is_empty() { (None, None) } else { (days(&hub, &node), days(&node, &hub)) };
        let last_of_act = mains.iter().position(|m| json::s(m, "_qid") == qid).is_some_and(|i| ends.contains(&i));
        let back_leg = if last_of_act { Some(0) } else { back };
        let mut m = Mission { id: json::s(q, "id"), qid, node, out, back, last_of_act, best: None, worst: None, worst_home: None };
        best_t = match (best_t, out) {
            (Some(t), Some(o)) => {
                m.best = Some(day_at(&mut probe, t + o));
                back_leg.map(|b| t + o + b)
            }
            _ => None,
        };
        worst_t = match (worst_t, out) {
            (Some(t), Some(o)) => {
                let won = t + o + losses * (retreat_days + rest_days + o);
                m.worst = Some(day_at(&mut probe, won));
                back_leg.map(|b| {
                    let home = won + b + rest_days;
                    m.worst_home = Some(day_at(&mut probe, home));
                    home
                })
            }
            _ => None,
        };
        missions.push(m);
    }

    let checkpoints = cps
        .iter()
        .map(|c| {
            let days = if c.quest.is_empty() {
                Err("not modelled (met_when)".to_string())
            } else {
                match missions.iter().find(|m| m.qid == c.quest) {
                    None => Err(format!("not modelled ({} is not a main quest)", c.quest)),
                    Some(m) => match (m.best, m.worst) {
                        (Some(b), Some(wd)) => Ok((b, wd)),
                        _ => Err(unreachable_reason(&missions, &m.qid)),
                    },
                }
            };
            CheckpointLine { id: c.id.clone(), act: c.act, quest: c.quest.clone(), deadline: c.deadline, days }
        })
        .collect();
    Ok(Plan { world: w.to_string(), hub, rest_days, retreat_days, losses, mode, missions, checkpoints })
}

/// Why a mission has no day: the first mission up to it whose node the graph does not reach.
fn unreachable_reason(missions: &[Mission], qid: &str) -> String {
    for m in missions {
        if m.out.is_none() || (!m.last_of_act && m.back.is_none() && m.qid != qid) {
            let what = if m.node.is_empty() { "has no node".to_string() } else { format!("node {} is not reached from the hub", m.node) };
            return format!("unreachable ({} {what})", m.id);
        }
        if m.qid == qid {
            break;
        }
    }
    "unreachable".into()
}

fn opt(v: Option<i64>) -> String {
    v.map_or("-".to_string(), |n| n.to_string())
}

fn plural(n: i64) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

/// The printed lines of one plan.
pub fn plan_lines(p: &Plan) -> Vec<String> {
    let mut out = vec![format!(
        "DEADLINES {}: hub {}, rest {} day{} after a loss and a win, retreat {} day{}, {} loss{} per mission in the worst case",
        p.world,
        p.hub,
        p.rest_days,
        plural(p.rest_days),
        p.retreat_days,
        plural(p.retreat_days),
        p.losses,
        if p.losses == 1 { "" } else { "es" }
    )];
    if let Some((name, days)) = &p.mode {
        out[0].push_str(&format!(", legs in the travel mode {name} (+{days} day{} a leg)", plural(*days)));
    }
    out.push(format!("{:<20} {:<28} {:>4} {:>5} {:>5} {:>6} {:>10}", "mission", "node", "out", "back", "best", "worst", "worst_home"));
    for m in &p.missions {
        let back = if m.last_of_act { "last".to_string() } else { opt(m.back) };
        let node = if m.node.is_empty() { "-" } else { m.node.as_str() };
        out.push(format!("{:<20} {:<28} {:>4} {:>5} {:>5} {:>6} {:>10}", m.id, node, opt(m.out), back, opt(m.best), opt(m.worst), opt(m.worst_home)));
    }
    out.push(format!("{:<20} {:>3} {:<20} {:>8} {:>5} {:>6} {:>6}", "checkpoint", "act", "quest", "deadline", "best", "worst", "slack"));
    for c in &p.checkpoints {
        match &c.days {
            Ok((b, wd)) => {
                let slack = c.deadline - wd;
                let flag = if slack < 0 { "  X MISSED in the worst case" } else { "" };
                out.push(format!("{:<20} {:>3} {:<20} {:>8} {:>5} {:>6} {:>6}{flag}", c.id, c.act, c.quest, c.deadline, b, wd, slack));
            }
            Err(why) => out.push(format!("{:<20} {:>3} {:<20} {:>8} {why}", c.id, c.act, c.quest, c.deadline)),
        }
    }
    out
}

/// `sim deadlines [world=<id>] [losses=0..9]`: the plan of each world (`worlds`), or why there is none.
pub fn deadlines_mode(d: &Data, a: &SimArgs) -> Vec<String> {
    let mut out = vec![format!("ARGS mode deadlines, world {}, losses {}", if a.world_set { a.world.as_str() } else { "all" }, a.losses)];
    let ws = worlds(d, a);
    if ws.is_empty() {
        out.push("DEADLINES: no world has campaign checkpoints (nothing to plan)".into());
        return out;
    }
    for w in ws {
        match plan(d, &w, a.losses) {
            Ok(p) => out.extend(plan_lines(&p)),
            Err(line) => out.push(line),
        }
    }
    out
}
