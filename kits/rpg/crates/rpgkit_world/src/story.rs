//! Story layer (STORYUI1 4.8, 5.2, 5.3; kit hook, engine-only, no Godot original): the saved `story` state (the
//! running sequence, the vignette queue, the seen list), vignettes as scenes (cards resolved at play time: `when`,
//! tokens, speakers), the queue (W1, W2, W4; a campaign `{vignette}` item joins it, CAMPAIGN1 T2), and story sequences
//! (Q1-Q9): steps `vignette`, `fight` (runner `story`), `effects` (the dialogue effect body), `enter` and `end`. The
//! game's story stage plays what `current` and `next_scene` give and reports back through `step_done`, `after_fight`,
//! `finish` and `vignette_done`. A world without vignettes or sequences never fills the state, so it is not saved.

use crate::data::Data;
use crate::state::{qid_ok, GameState};
use crate::{battle, campaign, dialogue, explore, progression};
use aegis_core::json::{self, Dict};
use rpgkit_data::Step;
use serde_json::Value;

/// 4.8: the keys of the saved `story` block and of its `running` part.
pub const BLOCK_KEYS: [&str; 3] = ["running", "queue", "seen"];
pub const RUNNING_KEYS: [&str; 2] = ["sequence", "step"];
/// Q5: the log line of a lost or fled story fight that is tried again.
pub const RETRY_TEXT: &str = "Try again.";
/// Q1: the refusal of a second sequence.
pub const RUNNING_TEXT: &str = "A story sequence is already running.";
/// Q5: the runner source of a story fight.
pub const FIGHT_SOURCE: &str = "story";

/// 4.8: the sequence that runs and the index of its current step.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Running {
    pub sequence: String,
    pub step: i64,
}

/// `gs.story` (4.8): written only when one of its parts is not empty.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Story {
    pub running: Option<Running>,
    /// Vignette qids waiting to play, oldest first (W1, W2).
    pub queue: Vec<String>,
    /// Vignette qids played (or left out whole, W4), in the order they were seen.
    pub seen: Vec<String>,
}

impl Story {
    pub fn is_empty(&self) -> bool {
        self.running.is_none() && self.queue.is_empty() && self.seen.is_empty()
    }

    /// The saved block: each key only when not empty.
    pub fn to_value(&self) -> Value {
        let mut out = Dict::new();
        if let Some(r) = &self.running {
            out.insert("running".into(), Value::Object(json::obj([("sequence", r.sequence.clone().into()), ("step", json::int(r.step))])));
        }
        if !self.queue.is_empty() {
            out.insert("queue".into(), json::strs(&self.queue));
        }
        if !self.seen.is_empty() {
            out.insert("seen".into(), json::strs(&self.seen));
        }
        Value::Object(out)
    }

    /// From a saved `story` value; every damage is a load problem (the save is refused, never coerced).
    pub fn from_value(raw: &Value, problems: &mut Vec<String>) -> Self {
        let mut out = Self::default();
        let Some(o) = raw.as_object() else {
            problems.push("story must be an object {running, queue, seen}.".into());
            return out;
        };
        for k in o.keys().filter(|k| !BLOCK_KEYS.contains(&k.as_str())) {
            problems.push(format!("story: unknown key '{k}'."));
        }
        if let Some(r) = o.get("running") {
            match r.as_object() {
                Some(r) if r.keys().all(|k| RUNNING_KEYS.contains(&k.as_str())) && r.get("sequence").is_some_and(Value::is_string) && r.get("step").is_some_and(json::is_int) => {
                    out.running = Some(Running { sequence: json::s(r, "sequence"), step: json::to_int(&r["step"]) });
                }
                _ => problems.push("story.running must be {sequence, step}.".into()),
            }
        }
        for (key, slot) in [("queue", &mut out.queue), ("seen", &mut out.seen)] {
            if let Some(v) = o.get(key) {
                match v.as_array().filter(|a| a.iter().all(Value::is_string)) {
                    Some(a) => *slot = a.iter().map(json::gstr).collect(),
                    None => problems.push(format!("story.{key} must be a list of text.")),
                }
            }
        }
        out
    }

    /// GameState::validate's part (4.8): qualified ids, a step >= 0, no id listed twice.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if let Some(r) = &self.running {
            if !qid_ok(&r.sequence) {
                errors.push(format!("story.running.sequence '{}' must be a qualified id world:id.", r.sequence));
            }
            if r.step < 0 {
                errors.push("story.running.step must be >= 0.".into());
            }
        }
        for (key, list) in [("queue", &self.queue), ("seen", &self.seen)] {
            let mut listed: Vec<&String> = Vec::new();
            for id in list {
                if !qid_ok(id) {
                    errors.push(format!("story.{key}: '{id}' must be a qualified id world:id."));
                } else if listed.contains(&id) {
                    errors.push(format!("story.{key} lists '{id}' twice."));
                }
                listed.push(id);
            }
        }
        errors
    }
}

/// Q8: after a load, a `running` that names an unknown sequence or a step out of range is dropped; returns the load
/// warnings.
pub fn check_loaded(d: &Data, gs: &mut GameState) -> Vec<String> {
    let Some(r) = gs.story.running.clone() else { return Vec::new() };
    let seq = d.resolve(&gs.world_id, "sequences", &r.sequence);
    let problem = if seq.is_empty() {
        format!("dropped story.running: unknown sequence '{}'", r.sequence)
    } else if r.step > json::aget(seq, "steps").len() as i64 {
        format!("dropped story.running: step {} is out of range for '{}'", r.step, r.sequence)
    } else {
        return Vec::new();
    };
    gs.story.running = None;
    vec![problem]
}

// ---- vignettes (5.2) ----

/// The vignette `r` names (from the party's world), {} when unknown.
pub fn vignette<'a>(d: &'a Data, gs: &GameState, r: &str) -> &'a Dict {
    d.resolve(&gs.world_id, "vignettes", r)
}

/// W1: queues vignette `r`; false (nothing queued) for an unknown ref, a `once` vignette already seen, or one already
/// in the queue.
pub fn queue(d: &Data, gs: &mut GameState, r: &str) -> bool {
    let v = vignette(d, gs, r);
    let qid = json::s(v, "_qid");
    if v.is_empty() || gs.story.queue.contains(&qid) || (json::b(v, "once", false) && gs.story.seen.contains(&qid)) {
        return false;
    }
    gs.story.queue.push(qid);
    true
}

/// CAMPAIGN1 T2: the `{vignette}` items at the head of the campaign's queue move to the story queue (W1 applies), in
/// order, up to the first item of another kind (the game plays battles and dialogues itself). Returns how many moved.
pub fn pull_pending(d: &Data, gs: &mut GameState) -> usize {
    let mut moved = 0;
    while let Some(r) = campaign::pending_head(gs).filter(|item| item.contains_key("vignette")).map(|item| json::s(item, "vignette")) {
        campaign::pop_pending(gs);
        queue(d, gs, &r);
        moved += 1;
    }
    moved
}

/// W3: the text of a card with its tokens filled: `{flag:KEY}` the flag's value as text ("" when unset; read keys
/// answer from the campaign), `{member:KEY}` the name of the party member or world character whose id the flag holds
/// (the id itself when it names neither, "" when unset). Other braces stay as written.
pub fn fill_tokens(d: &Data, gs: &GameState, text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            out.push_str(&rest[open..]);
            return out;
        };
        let inner = &after[..close];
        match rpgkit_data::split_token(inner).filter(|(kind, _)| rpgkit_data::TOKEN_KINDS.contains(kind)) {
            Some((kind, key)) => out.push_str(&token_value(d, gs, kind, key)),
            None => out.push_str(&rest[open..open + close + 2]),
        }
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    out
}

fn token_value(d: &Data, gs: &GameState, kind: &str, key: &str) -> String {
    let value = gs.read_flag(key).map(|v| json::gstr(&v)).unwrap_or_default();
    if kind == "flag" || value.is_empty() {
        return value;
    }
    let c = d.resolve(&gs.world_id, "characters", &value);
    let cqid = json::s(c, "_qid");
    if let Some(m) = gs.party.members.iter().find(|m| m.id == value || (!cqid.is_empty() && m.lore_ref == cqid)) {
        return progression::display_name(d, m);
    }
    if c.is_empty() {
        value
    } else {
        json::s_or(c, "name", &value)
    }
}

/// W3, W5: card `card` as it plays now, or None when its `when` fails: `when` dropped, `text` with its tokens filled,
/// a `line` card's `speaker` as a qid plus `speaker_name`.
pub fn card_now(d: &Data, gs: &GameState, home: &str, card: &Dict) -> Option<Dict> {
    if !json::aget(card, "when").iter().all(|c| dialogue::condition_holds(d, gs, json::dict(c))) {
        return None;
    }
    let mut out = card.clone();
    out.shift_remove("when");
    if let Some(Value::String(t)) = card.get("text") {
        out.insert("text".into(), fill_tokens(d, gs, t).into());
    }
    let speaker = json::s(card, "speaker");
    if !speaker.is_empty() {
        let c = d.resolve(home, "characters", &speaker);
        out.insert("speaker".into(), json::s_or(c, "_qid", &speaker).into());
        out.insert("speaker_name".into(), json::s_or(c, "name", &speaker).into());
    }
    Some(out)
}

/// W3: vignette `r` as it plays now: {vignette (qid), name, music, frame, skippable, once, seen (W6: Skip asks only
/// when false), cards}; {} for an unknown ref.
pub fn scene(d: &Data, gs: &GameState, r: &str) -> Dict {
    let v = vignette(d, gs, r);
    if v.is_empty() {
        return Dict::new();
    }
    let qid = json::s(v, "_qid");
    let home = json::s_or(v, "_world", &gs.world_id);
    let cards: Vec<Value> = json::aget(v, "cards").iter().filter_map(|c| card_now(d, gs, &home, json::dict(c))).map(Value::Object).collect();
    json::obj([
        ("vignette", qid.clone().into()),
        ("name", json::s(v, "name").into()),
        ("music", json::s(v, "music").into()),
        ("frame", json::s_or(v, "frame", rpgkit_data::FRAME_COLOUR).into()),
        ("skippable", Value::Bool(json::b(v, "skippable", true))),
        ("once", Value::Bool(json::b(v, "once", false))),
        ("seen", Value::Bool(gs.story.seen.contains(&qid))),
        ("cards", Value::Array(cards)),
    ])
}

fn mark_seen(gs: &mut GameState, qid: &str) {
    if !gs.story.seen.iter().any(|s| s == qid) {
        gs.story.seen.push(qid.to_string());
    }
}

/// W2, W4: the scene of the queue's oldest entry (campaign vignette items pulled in first); an entry whose every card
/// is left out is marked seen and leaves the queue, an unknown one leaves it. The entry stays queued until
/// `vignette_done`.
pub fn next_scene(d: &Data, gs: &mut GameState) -> Option<Dict> {
    pull_pending(d, gs);
    while let Some(head) = gs.story.queue.first().cloned() {
        let sc = scene(d, gs, &head);
        if !json::aget(&sc, "cards").is_empty() {
            return Some(sc);
        }
        gs.story.queue.remove(0);
        if !sc.is_empty() {
            mark_seen(gs, &head);
        }
    }
    None
}

/// W2: vignette `qid` ended or was skipped: it leaves the queue and joins `seen`.
pub fn vignette_done(gs: &mut GameState, qid: &str) {
    if let Some(at) = gs.story.queue.iter().position(|q| q == qid) {
        gs.story.queue.remove(at);
    }
    mark_seen(gs, qid);
}

// ---- sequences (5.3) ----

/// The running sequence's entry ({} when none runs or it is unknown).
fn running_sequence<'a>(d: &'a Data, gs: &GameState) -> &'a Dict {
    match &gs.story.running {
        Some(r) => d.resolve(&gs.world_id, "sequences", &r.sequence),
        None => json::empty(),
    }
}

/// The running step as written (refs unresolved); a sequence run past its last step ends as `end` `map` (4.3).
fn raw_step(d: &Data, gs: &GameState) -> Option<(Step, String)> {
    let r = gs.story.running.as_ref()?;
    let seq = running_sequence(d, gs);
    let home = json::s_or(seq, "_world", &gs.world_id);
    let steps = json::aget(seq, "steps");
    if r.step < 0 || r.step as usize >= steps.len() {
        return Some((Step::End { to: "map".into() }, home));
    }
    Step::from_obj(json::dict(&steps[r.step as usize])).map(|s| (s, home))
}

/// Q2: the step at `running.step` with its refs as qids (`vignette`, `fight`, `enter` or `end`; never `effects`, which
/// run at once); None while no sequence runs.
pub fn current(d: &Data, gs: &GameState) -> Option<Step> {
    let (step, home) = raw_step(d, gs)?;
    let qid = |kind: &str, r: &str| json::s_or(d.resolve(&home, kind, r), "_qid", r);
    Some(match step {
        Step::Vignette { vignette } => Step::Vignette { vignette: qid("vignettes", &vignette) },
        Step::Fight { encounter, on_lost, on_fled } => Step::Fight { encounter: qid("encounters", &encounter), on_lost, on_fled },
        Step::Enter { location, anchor } => Step::Enter { location: qid("locations", &location), anchor },
        other => other,
    })
}

/// Whether a sequence runs (the game shows its story stage; Q8: the game menu cannot open).
pub fn is_running(gs: &GameState) -> bool {
    gs.story.running.is_some()
}

fn next_step(gs: &mut GameState) {
    if let Some(r) = gs.story.running.as_mut() {
        r.step += 1;
    }
}

/// Q3: the `effects` steps from the running step on apply at once, in order, with the dialogue effect body (no tick:
/// the caller ticks the campaign once after).
fn run_effects(d: &Data, gs: &mut GameState, notes: &mut Vec<String>, events: &mut Vec<Value>) {
    while let Some((Step::Effects { effects }, _)) = raw_step(d, gs) {
        for fx in &effects {
            dialogue::apply_effect(d, gs, json::dict(fx), notes, events);
        }
        next_step(gs);
    }
}

/// Q1: starts sequence `r` (refused while one runs or for an unknown ref) and runs its leading `effects` steps, with
/// no campaign tick (the effect list that started it ticks after). Ok(qid) or Err(why).
pub fn start_quiet(d: &Data, gs: &mut GameState, r: &str, notes: &mut Vec<String>, events: &mut Vec<Value>) -> Result<String, String> {
    if is_running(gs) {
        return Err(RUNNING_TEXT.into());
    }
    let qid = json::s(d.resolve(&gs.world_id, "sequences", r), "_qid");
    if qid.is_empty() {
        return Err(format!("Unknown sequence '{r}'."));
    }
    gs.story.running = Some(Running { sequence: qid.clone(), step: 0 });
    run_effects(d, gs, notes, events);
    Ok(qid)
}

/// {ok, errors, notes, campaign?}: the effects' notes for the log, the campaign events when there are some.
fn story_result(ok: bool, errors: &[String], notes: Vec<String>, events: Vec<Value>) -> Dict {
    let mut out = json::ok_result(ok, errors);
    out.insert("notes".into(), json::strs(&notes));
    if !events.is_empty() {
        out.insert("campaign".into(), Value::Array(events));
    }
    out
}

/// Runs `f` (effects through the dialogue body), then the campaign tick after an effect list (CAMPAIGN1 X4).
fn with_tick(d: &Data, gs: &mut GameState, f: impl FnOnce(&mut GameState, &mut Vec<String>, &mut Vec<Value>) -> Vec<String>) -> Dict {
    let act = gs.campaign.as_ref().map(|c| c.act);
    let (mut notes, mut events) = (Vec::new(), Vec::new());
    let errors = f(gs, &mut notes, &mut events);
    if errors.is_empty() {
        events.extend(campaign::after_effects(d, gs, act));
    }
    story_result(errors.is_empty(), &errors, notes, events)
}

/// Q1, Q3: starts sequence `r` and runs its leading `effects` steps, then the campaign tick.
pub fn start(d: &Data, gs: &mut GameState, r: &str) -> Dict {
    with_tick(d, gs, |gs, notes, events| start_quiet(d, gs, r, notes, events).err().into_iter().collect())
}

/// Q4: the running `vignette` step ended or was skipped: the vignette joins `seen`, the next step, then Q3.
pub fn step_done(d: &Data, gs: &mut GameState) -> Dict {
    let Some(Step::Vignette { vignette }) = current(d, gs) else { return json::fail("The story is not playing a vignette.") };
    with_tick(d, gs, |gs, notes, events| {
        mark_seen(gs, &vignette);
        next_step(gs);
        run_effects(d, gs, notes, events);
        Vec::new()
    })
}

/// Q5: the world fight of the running `fight` step, `{source: "story", encounter}`; None at any other step.
pub fn fight(d: &Data, gs: &GameState) -> Option<Dict> {
    match current(d, gs)? {
        Step::Fight { encounter, .. } => Some(json::obj([("source", FIGHT_SOURCE.into()), ("encounter", encounter.into())])),
        _ => None,
    }
}

/// Q5, Q8: after a story fight: `battle::settle` (rewards, quest progress), then by outcome: won, the next step; lost
/// or fled by the step's `on_lost` / `on_fled`: `retry` brings every member back to full HP and keeps the step (the log
/// says "Try again."), `continue` goes on, `abort` clears `running` and wakes the party on a loss (the dialogue fight's
/// loss path). The step moves before the game's autosave, so a load resumes at the next beat. {ok, errors, xp,
/// advanced, events, notes, rewards, gold?, campaign?}.
pub fn after_fight(d: &Data, gs: &mut GameState, fight: &Dict, outcome: &str, tally: Option<&battle::Tally>) -> Dict {
    let applied = battle::settle(d, gs, &json::s(fight, "encounter"), outcome, tally);
    if !json::b(&applied, "ok", false) {
        let mut out = json::ok_result(false, &json::sget(&applied, "errors"));
        out.insert("xp".into(), Value::Object(Dict::new()));
        out.insert("advanced".into(), Value::Array(Vec::new()));
        out.insert("events".into(), Value::Array(Vec::new()));
        return out;
    }
    let rule = match current(d, gs) {
        Some(Step::Fight { on_lost, on_fled, .. }) => match outcome {
            "won" => "continue".to_string(),
            "fled" => on_fled,
            _ => on_lost,
        },
        // not at a fight step (nothing runs): the dialogue fight's loss path
        _ => "abort".to_string(),
    };
    let act = gs.campaign.as_ref().map(|c| c.act);
    let (mut notes, mut fx_events, mut events) = (Vec::new(), Vec::new(), Vec::new());
    match rule.as_str() {
        "retry" => {
            progression::rest(d, &mut gs.party);
            notes.push(RETRY_TEXT.to_string());
        }
        "continue" => {
            next_step(gs);
            run_effects(d, gs, &mut notes, &mut fx_events);
            fx_events.extend(campaign::after_effects(d, gs, act));
        }
        _ => {
            gs.story.running = None;
            if outcome == "lost" {
                let woke = explore::wake(d, gs, "");
                events.extend(json::aget(&woke, "events").iter().cloned());
            }
        }
    }
    let mut out = json::ok_result(true, &json::sget(&applied, "errors"));
    out.insert("xp".into(), applied["xp"].clone());
    out.insert("advanced".into(), applied["advanced"].clone());
    out.insert("events".into(), Value::Array(events));
    out.insert("notes".into(), json::strs(&notes));
    for (k, v) in battle::result_keys(&applied) {
        out.insert(k, v);
    }
    if !fx_events.is_empty() {
        let mut all = json::aget(&out, "campaign").to_vec();
        all.extend(fx_events);
        out.insert("campaign".into(), Value::Array(all));
    }
    out
}

/// Q6, Q7: the running sequence ends at its `enter` or `end` step: `enter` places the party at the location (anchor ""
/// = the entry) and the map shows; `end` goes to `map` or `menu` (the game autosaves and ends the game in play).
/// `running` is cleared either way. {ok, errors, to ("map" or "menu"), events}.
pub fn finish(d: &Data, gs: &mut GameState) -> Dict {
    let (to, mut out) = match current(d, gs) {
        Some(Step::Enter { location, anchor }) => {
            let anchor = if anchor.is_empty() { explore::SPAWN.to_string() } else { anchor };
            ("map".to_string(), explore::enter(d, gs, &location, &anchor))
        }
        Some(Step::End { to }) => (to, json::ok_result(true, &[])),
        _ => return json::fail("The story is not at an enter or end step."),
    };
    gs.story.running = None;
    out.insert("to".into(), to.into());
    if !out.contains_key("events") {
        out.insert("events".into(), Value::Array(Vec::new()));
    }
    out
}

/// 5.4: the sequence a new game in world `w` starts with (the nearest non-empty manifest `start_sequence` along the
/// lineage), "" for none.
pub fn start_sequence(d: &Data, w: &str) -> String {
    for lw in d.lineage(w).iter().rev() {
        let s = json::s(d.manifest(lw), "start_sequence");
        if !s.is_empty() {
            return s;
        }
    }
    String::new()
}
