//! CAMPAIGN-SIM1 (ENGINE-28): the world campaign run, `campaign world=<world> [seeds=N] [side=N] [story=<enc,...>]`.
//! A world's story fights in story order, each with the party it fields (so joins and leaves come from the data), the
//! party's XP and levels carried from fight to fight by the kit's own rules (`progression`: kill XP of the defeated
//! roster at its stage level, `xp_for_level` / `level_for`; the raise spends by the class's default growth,
//! `WorldRoster::ally_levels`). Each fight counts as won for the XP (a lost story fight is retried in play), and starts
//! at full HP. Before each quest fight, `side=N` roamer fights from the world's paced patrol bindings (default N: the
//! largest `visit_cap`), round robin, a repeat win paying its binding's `repeat_xp_pct`. Per fight: the members'
//! levels on entry, won %, the won fights' median rounds, the EN7b minutes at that median and the HP left, beside the
//! same fight at the authored levels (as the feel mode fights it); then `CAMPAIGN_JSON {...}`. Report only.
//!
//! Story order (no `story=`): the fights of the world's `new_game` sequence, then each main-track quest's step fights
//! in quest order; a sequence whose effects set a flag a main quest requires plays its fights just before that quest's
//! (one that unlocks no quest plays last). The kit names no world: everything is read from the world's data.

use crate::feel::{minutes, Pace};
use crate::runner::Runner;
use aegis_core::json::{self, Dict};
use indexmap::IndexMap;
use rpgkit_world::{battle, progression, Data};
use serde_json::{json, Value};

/// The sequence whose fights open a world's story.
pub const NEW_GAME_SEQUENCE: &str = "new_game";
/// The quest track the story follows.
pub const MAIN_TRACK: &str = "main";
/// The prefix of the run's JSON line.
pub const JSON_PREFIX: &str = "CAMPAIGN_JSON ";

/// A fight of the run: its encounter, and whether it is a side fight (else a story fight); `roam` marks a story fight
/// the party walks to (a quest fight: side fights come before it), `xp_pct` the share of the XP it pays.
#[derive(Debug, Clone, PartialEq)]
pub struct Beat {
    pub id: String,
    pub side: bool,
    pub roam: bool,
    pub xp_pct: i64,
}

/// The fights of sequence `seq` (its `fight` steps' encounters, in order).
fn sequence_fights(seq: &Dict) -> Vec<String> {
    json::aget(seq, "steps").iter().map(json::dict).filter(|st| json::s(st, "kind") == "fight").map(|st| json::s(st, "encounter")).filter(|e| !e.is_empty()).collect()
}

/// The flags sequence `seq` sets (its `effects` steps' `set_flag` keys).
fn sequence_flags(seq: &Dict) -> Vec<String> {
    let steps = json::aget(seq, "steps").iter().map(json::dict);
    let effects: Vec<Dict> = steps.flat_map(|st| json::aget(st, "effects").iter().map(json::dict).cloned().collect::<Vec<_>>()).collect();
    effects.iter().filter(|e| json::s(e, "effect") == "set_flag").map(|e| json::s(e, "key")).collect()
}

/// The story fights of `world` in story order (see the module doc): (encounter, a quest fight).
pub fn story_order(d: &Data, world: &str) -> Vec<(String, bool)> {
    let seqs = d.merged_entries(world, "sequences");
    let mut out: Vec<(String, bool)> = Vec::new();
    if let Some(open) = seqs.iter().find(|s| json::s(s, "id") == NEW_GAME_SEQUENCE) {
        out.extend(sequence_fights(open).into_iter().map(|e| (e, false)));
    }
    let mut pending: Vec<&Dict> = seqs.iter().copied().filter(|s| json::s(s, "id") != NEW_GAME_SEQUENCE && !sequence_fights(s).is_empty()).collect();
    for q in d.merged_entries(world, "quests").iter().filter(|q| json::s(q, "track") == MAIN_TRACK) {
        let needs: Vec<String> = json::aget(q, "requires_flags").iter().map(|f| json::s(json::dict(f), "key")).collect();
        let (before, rest): (Vec<&Dict>, Vec<&Dict>) = pending.into_iter().partition(|s| sequence_flags(s).iter().any(|f| needs.contains(f)));
        pending = rest;
        out.extend(before.iter().flat_map(|s| sequence_fights(s)).map(|e| (e, false)));
        let steps = json::aget(q, "steps").iter().map(json::dict);
        out.extend(steps.map(|st| json::s(st, "encounter")).filter(|e| !e.is_empty()).map(|e| (e, true)));
    }
    out.extend(pending.iter().flat_map(|s| sequence_fights(s)).map(|e| (e, false)));
    let mut seen: Vec<String> = Vec::new();
    out.retain(|(e, _)| !seen.contains(e) && {
        seen.push(e.clone());
        true
    });
    out
}

/// A given story order (`story=`): a fight that plays in one of the world's sequences is walked to by no one (no side
/// fights before it); any other is a quest fight.
pub fn given_order(d: &Data, world: &str, ids: &[String]) -> Vec<(String, bool)> {
    let in_sequence: Vec<String> = d.merged_entries(world, "sequences").iter().flat_map(|s| sequence_fights(s)).collect();
    ids.iter().map(|e| (e.clone(), !in_sequence.contains(e))).collect()
}

/// The paced patrol bindings of `world`'s locations (any object setting a PACE1 key with an `encounters` list): per
/// encounter in first-seen order, its binding's `repeat_xp_pct` list; and the largest `visit_cap`.
pub fn side_pool(d: &Data, world: &str) -> (IndexMap<String, Vec<i64>>, i64) {
    fn walk(v: &Value, pool: &mut IndexMap<String, Vec<i64>>, cap: &mut i64) {
        match v {
            Value::Object(o) => {
                if rpgkit_world::explore::paced(o) {
                    *cap = (*cap).max(json::i(o, rpgkit_world::explore::VISIT_CAP, 0));
                    let pcts: Vec<i64> = json::aget(o, rpgkit_world::explore::REPEAT_XP_PCT).iter().map(json::to_int).collect();
                    for pair in json::aget(o, "encounters") {
                        let id = json::arr(pair).first().map(json::gstr).unwrap_or_default();
                        if !id.is_empty() {
                            pool.entry(id).or_insert_with(|| pcts.clone());
                        }
                    }
                }
                o.values().for_each(|x| walk(x, pool, cap));
            }
            Value::Array(a) => a.iter().for_each(|x| walk(x, pool, cap)),
            _ => {}
        }
    }
    let (mut pool, mut cap) = (IndexMap::new(), 0);
    for loc in d.merged_entries(world, "locations") {
        walk(&Value::Object(loc.clone()), &mut pool, &mut cap);
    }
    (pool, cap)
}

/// The run's fights: the story order (or `story`) with `side` side fights before each quest fight, drawn round robin
/// from `pool`, a repeat paying its `repeat_xp_pct` (100 the first win, 0 past the list).
pub fn beats(story: &[(String, bool)], pool: &IndexMap<String, Vec<i64>>, side: usize) -> Vec<Beat> {
    let mut out = Vec::new();
    let mut wins: IndexMap<String, usize> = IndexMap::new();
    let mut next = 0;
    for (id, roam) in story {
        if *roam && !pool.is_empty() {
            for _ in 0..side {
                let (sid, pcts) = pool.get_index(next % pool.len()).expect("a pool entry");
                next += 1;
                let w = wins.entry(sid.clone()).or_insert(0);
                let xp_pct = if *w == 0 { 100 } else { pcts.get(*w - 1).copied().map_or(if pcts.is_empty() { 100 } else { 0 }, |p| p.clamp(0, 100)) };
                *w += 1;
                out.push(Beat { id: sid.clone(), side: true, roam: false, xp_pct });
            }
        }
        out.push(Beat { id: id.clone(), side: false, roam: *roam, xp_pct: 100 });
    }
    out
}

/// One fight over the seeds at the runner's ally levels: (won %, won-fight median rounds, mean HP left %).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Read {
    pub won_pct: f64,
    pub rounds: f64,
    pub hp_pct: f64,
}

fn median(v: &mut [i32]) -> f64 {
    v.sort_unstable();
    match v.len() {
        0 => 0.0,
        n if n % 2 == 1 => f64::from(v[n / 2]),
        n => f64::from(v[n / 2 - 1] + v[n / 2]) / 2.0,
    }
}

/// The median rounds as printed ("-" when no fight was won).
fn rounds_text(x: Read) -> String {
    if x.won_pct > 0.0 {
        x.rounds.to_string()
    } else {
        "-".into()
    }
}

/// A name -> number map as a JSON object.
fn obj(m: &IndexMap<String, i64>) -> Value {
    Value::Object(m.iter().map(|(k, v)| (k.clone(), json!(v))).collect())
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

fn read(r: &mut Runner, world: &str, id: &str, seeds: &[i64]) -> Read {
    let cap = json::i(r.d.resolve(world, "encounters", id), "max_rounds", i64::from(r.args.cap)) as i32;
    r.rounds_override = cap;
    let (mut won, mut rounds, mut hp) = (0, Vec::new(), 0.0);
    for &seed in seeds {
        let res = r.fight_world(world, id, seed);
        let (left, max): (i32, i32) = (res.party_hp.values().map(|h| (*h).max(0)).sum(), res.party_max.values().sum());
        hp += 100.0 * f64::from(left) / f64::from(max.max(1));
        if res.won {
            won += 1;
            rounds.push(res.rounds);
        }
    }
    r.rounds_override = 0;
    let n = seeds.len().max(1) as f64;
    Read { won_pct: round1(100.0 * f64::from(won) / n), rounds: median(&mut rounds), hp_pct: round1(hp / n) }
}

/// The level a member of `world` fields at first: its character's unit level.
fn authored_level(d: &Data, world: &str, member: &str) -> i64 {
    json::i(&d.character_unit_entry(d.resolve(world, "characters", member)), "level", 1)
}

/// The world campaign run (`campaign world=<world>`): its ARGS line, a line per fight, the final levels and the JSON.
pub fn run(r: &mut Runner) {
    let d = r.d;
    let a = r.args;
    let world = a.world.clone();
    let story = if a.story.is_empty() { story_order(d, &world) } else { given_order(d, &world, &a.story) };
    let (pool, cap) = side_pool(d, &world);
    let side = a.side.map_or(cap.max(0) as usize, |n| n as usize);
    let plan = beats(&story, &pool, side);
    let seeds = a.seeds();
    r.print(format!("ARGS mode campaign, world {world}, seeds {}, side {side}, stage {}, story {}", seeds.len(), a.stage, if a.story.is_empty() { "derived" } else { "given" }));
    if story.is_empty() {
        r.print(format!("CAMPAIGN: world {world} has no story fights (no {NEW_GAME_SEQUENCE} sequence fights, no {MAIN_TRACK} quest fights); give story=<enc,...>"));
        return;
    }
    let stage = i64::from(a.stage);
    let mut xp: IndexMap<String, i64> = IndexMap::new();
    let mut rows: Vec<Value> = Vec::new();
    for (i, b) in plan.iter().enumerate() {
        let enc = d.resolve(&world, "encounters", &b.id).clone();
        let party: Vec<String> = json::sget(&enc, "party");
        for m in &party {
            let start = progression::xp_for_level(d, authored_level(d, &world, m));
            xp.entry(m.clone()).or_insert(start);
        }
        let levels: IndexMap<String, i64> = party.iter().map(|m| (m.clone(), progression::level_for(d, xp[m]))).collect();
        r.ally_levels = levels.clone();
        let now = read(r, &world, &b.id, &seeds);
        r.ally_levels.clear();
        let base = read(r, &world, &b.id, &seeds);
        let pace = Pace { group: "", heroes: party.len().max(1), blocks: json::i(&enc, "total_blocks", 6) as i32 };
        let mins = round1(minutes(&pace, now.rounds));
        // the XP of the won fight: every foe of the roster defeated, each member paid by its own level
        let foes: Vec<(i64, String)> = battle::roster_at(d, &world, &b.id, stage).into_iter().map(|f| (f.level, f.tier)).collect();
        let mut paid = IndexMap::new();
        for m in &party {
            let full = progression::encounter_xp(d, &foes, levels[m]);
            let got = full * b.xp_pct.clamp(0, 100) / 100;
            *xp.get_mut(m).expect("a member") += got;
            paid.insert(m.clone(), got);
        }
        let kind = if b.side { "side" } else { "story" };
        let lv = levels.iter().map(|(m, l)| format!("{m} {l}")).collect::<Vec<_>>().join(", ");
        let pct = if b.xp_pct < 100 { format!(" ({}% XP)", b.xp_pct) } else { String::new() };
        let gain = paid.values().copied().max().unwrap_or(0);
        r.print(format!(
            "FIGHT {} {} [{kind}] {lv}: won {}% (authored {}%), rounds {} (authored {}), {mins} min, HP left {}% (authored {}%), XP +{gain}{pct}",
            i + 1,
            b.id,
            now.won_pct,
            base.won_pct,
            rounds_text(now),
            rounds_text(base),
            now.hp_pct,
            base.hp_pct
        ));
        rows.push(json!({
            "n": i + 1, "id": b.id, "kind": kind, "levels": obj(&levels), "xp_pct": b.xp_pct,
            "won_pct": now.won_pct, "rounds": now.rounds, "minutes": mins, "hp_pct": now.hp_pct, "xp": obj(&paid),
            "authored": {"won_pct": base.won_pct, "rounds": base.rounds, "hp_pct": base.hp_pct},
        }));
    }
    let end: IndexMap<String, i64> = xp.iter().map(|(m, x)| (m.clone(), progression::level_for(d, *x))).collect();
    r.print(format!("END levels {}", end.iter().map(|(m, l)| format!("{m} {l}")).collect::<Vec<_>>().join(", ")));
    let out = json!({"world": world, "seeds": seeds.len(), "side": side, "stage": a.stage, "fights": rows, "end_levels": obj(&end), "end_xp": obj(&xp)});
    r.print(format!("{JSON_PREFIX}{out}"));
}
