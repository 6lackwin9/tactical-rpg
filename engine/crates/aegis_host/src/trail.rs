//! Trail report (PLAYTEST1 rule 12, TrailReport): reads a recording's UI trail (every screen) and flags interactions
//! that may feel wrong even when the screen looks fine. Flags {kind, t, ctl, detail, note}; lines
//! `FLAG <kind> t=<ms> <ctl> <detail>`:
//!   dead_click         an unhandled click (or one on a control that is not interactive) with no reaction within 500 ms
//!   repeat_click       3+ clicks on one control or cell within 2 s (one flag per burst)
//!   long_hover         a hover dwell of 3000 ms or more
//!   refused_confirm    a refused plan confirm, with its reason
//!   error              an error or warning, with the last user event before it
//!   undo_churn         3+ remove / undo / clear plan edits for one unit in one round
//!   double_open        a panel / modal opens again without a close
//!   close_without_open
//!   left_open          still open at a screen change or the end (not the screen's own: opened within 1 s of it)
//!   unprompted_open    no click / button / key in the 1000 ms before it, and not at a screen start
//!   slow_open          more than 500 ms after the user event before it
//!   leak               a screen event with leaked > 0
//!   asset_fallback     every asset event
//! Each flag names the nearest earlier note within 30 s. The report never fails a run.
//!
//! Trail events: {t: ms, type, ...}; controls are `ctl` {path, cls, text, tip, interactive?}. Types: screen {path,
//! leaked}, panel / modal {ctl, open}, button {ctl}, key {key, mods}, hover {ctl, ms}, plan_edit {kind, unit, round},
//! confirm {ok, reason, round}, view, note {n}, click {button, pos, ctl (null over the game world), cell?, handled},
//! error {level, msg, where}, asset {ns, key, variant, source, screen}.

use serde_json::{json, Value};

pub const DEAD_CLICK_MS: i64 = 500;
pub const REPEAT_WINDOW_MS: i64 = 2000;
pub const REPEAT_MIN: usize = 3;
pub const LONG_HOVER_MS: i64 = 3000;
pub const CHURN_MIN: usize = 3;
pub const SCENE_START_MS: i64 = 1000;
pub const PROMPT_WINDOW_MS: i64 = 1000;
pub const SLOW_OPEN_MS: i64 = 500;
pub const NOTE_WINDOW_MS: i64 = 30000;

const REACTIONS: [&str; 6] = ["button", "panel", "modal", "plan_edit", "confirm", "view"];
const USER_EVENTS: [&str; 3] = ["click", "button", "key"];
const CHURN_KINDS: [&str; 3] = ["remove", "undo", "clear"];
pub const KINDS: [&str; 13] = ["dead_click", "repeat_click", "long_hover", "refused_confirm", "error", "undo_churn", "double_open", "close_without_open", "left_open", "unprompted_open", "slow_open", "leak", "asset_fallback"];
/// Widget kinds that react to clicks (the engine kit's); a click on anything else is on a control that is not interactive.
const INTERACTIVE: [&str; 10] = ["button", "select", "option", "slider", "slider_step", "toggle", "tab", "text", "link", "window"];

fn t_of(e: &Value) -> i64 {
    e["t"].as_i64().unwrap_or(0)
}

fn ty(e: &Value) -> &str {
    e["type"].as_str().unwrap_or("")
}

/// The control path, "cell[x,y]", or "-".
pub fn target_text(e: &Value) -> String {
    if let Some(p) = e["ctl"]["path"].as_str().filter(|p| !p.is_empty()) {
        return p.to_string();
    }
    match e["cell"].as_array() {
        Some(c) if c.len() == 2 => format!("cell[{},{}]", c[0].as_i64().unwrap_or(0), c[1].as_i64().unwrap_or(0)),
        _ => "-".into(),
    }
}

fn event_text(e: &Value) -> String {
    if ty(e) == "key" {
        return format!("key {}{}", e["mods"].as_str().unwrap_or(""), e["key"].as_str().unwrap_or(""));
    }
    format!("{} {} t={}", ty(e), target_text(e), t_of(e))
}

fn interactive(e: &Value) -> bool {
    let ctl = &e["ctl"];
    if ctl.is_null() {
        return true;
    }
    if let Some(b) = ctl["interactive"].as_bool() {
        return b;
    }
    INTERACTIVE.contains(&ctl["cls"].as_str().unwrap_or(""))
}

fn flag(flags: &mut Vec<Value>, kind: &str, e: &Value, detail: String, ctl: Option<String>) {
    flags.push(json!({"kind": kind, "t": t_of(e), "ctl": ctl.unwrap_or_else(|| target_text(e)), "detail": detail}));
}

/// {flags, counts: {kind: n}, events, dropped} of a trail and its notes.
pub fn build(trail: &[Value], dropped: u64, notes: &[Value]) -> Value {
    let mut trail: Vec<Value> = trail.to_vec();
    trail.sort_by_key(t_of);
    let mut flags = Vec::new();
    clicks(&trail, &mut flags);
    singles(&trail, &mut flags);
    churn(&trail, &mut flags);
    open_close(&trail, &mut flags);
    flags.sort_by_key(t_of);
    let mut counts = serde_json::Map::new();
    for k in KINDS {
        counts.insert(k.into(), json!(0));
    }
    for f in &mut flags {
        f["note"] = json!(nearest_note(notes, t_of(f)));
        let k = f["kind"].as_str().unwrap_or("").to_string();
        let n = counts[&k].as_i64().unwrap_or(0) + 1;
        counts.insert(k, json!(n));
    }
    json!({"flags": flags, "counts": counts, "events": trail.len(), "dropped": dropped})
}

/// The report's lines: one per flag, then the summary.
pub fn lines(report: &Value) -> Vec<String> {
    let flags = report["flags"].as_array().cloned().unwrap_or_default();
    let mut out: Vec<String> = flags
        .iter()
        .map(|f| {
            let detail = f["detail"].as_str().unwrap_or("");
            let note = f["note"].as_i64().unwrap_or(0);
            format!(
                "FLAG {} t={} {}{}{}",
                f["kind"].as_str().unwrap_or(""),
                t_of(f),
                f["ctl"].as_str().unwrap_or("-"),
                if detail.is_empty() { String::new() } else { format!(" {detail}") },
                if note > 0 { format!(" (note {note})") } else { String::new() }
            )
        })
        .collect();
    let parts: Vec<String> = KINDS.iter().filter_map(|k| report["counts"][*k].as_i64().filter(|n| *n > 0).map(|n| format!("{k} {n}"))).collect();
    out.push(format!(
        "trail report: {} event(s), {} dropped, {} flag(s){}",
        report["events"].as_i64().unwrap_or(0),
        report["dropped"].as_i64().unwrap_or(0),
        flags.len(),
        if parts.is_empty() { String::new() } else { format!(" ({})", parts.join(", ")) }
    ));
    out
}

fn clicks(trail: &[Value], flags: &mut Vec<Value>) {
    let mut bursts: Vec<(String, Vec<i64>, bool)> = Vec::new();
    for (i, e) in trail.iter().enumerate() {
        if ty(e) != "click" {
            continue;
        }
        let t = t_of(e);
        let unhandled = !e["handled"].as_bool().unwrap_or(true);
        if unhandled || !interactive(e) {
            let reacted = trail[i + 1..].iter().take_while(|n| t_of(n) - t <= DEAD_CLICK_MS).any(|n| REACTIONS.contains(&ty(n)));
            if !reacted {
                let why = if unhandled { "unhandled" } else { "non-interactive control" };
                flag(flags, "dead_click", e, format!("{} click, {why}, nothing reacted within {DEAD_CLICK_MS} ms", e["button"].as_str().unwrap_or("")), None);
            }
        }
        let target = target_text(e);
        if target == "-" {
            continue;
        }
        let slot = match bursts.iter().position(|b| b.0 == target) {
            Some(i) => i,
            None => {
                bursts.push((target.clone(), Vec::new(), false));
                bursts.len() - 1
            }
        };
        let (_, times, flagged) = &mut bursts[slot];
        if times.last().is_some_and(|last| t - last > REPEAT_WINDOW_MS) {
            times.clear();
            *flagged = false;
        }
        times.push(t);
        while t - times[0] > REPEAT_WINDOW_MS {
            times.remove(0);
        }
        if times.len() >= REPEAT_MIN && !*flagged {
            *flagged = true;
            let n = times.len();
            flag(flags, "repeat_click", e, format!("{n} clicks within {REPEAT_WINDOW_MS} ms"), None);
        }
    }
}

fn singles(trail: &[Value], flags: &mut Vec<Value>) {
    let mut last_user: Option<&Value> = None;
    for e in trail {
        match ty(e) {
            t if USER_EVENTS.contains(&t) => last_user = Some(e),
            "hover" if e["ms"].as_i64().unwrap_or(0) >= LONG_HOVER_MS => flag(flags, "long_hover", e, format!("{} ms", e["ms"].as_i64().unwrap_or(0)), None),
            "confirm" if !e["ok"].as_bool().unwrap_or(true) => flag(flags, "refused_confirm", e, format!("round {}: {}", e["round"].as_i64().unwrap_or(0), e["reason"].as_str().unwrap_or("")), None),
            "error" => {
                let after = last_user.map(|u| format!("after {}", event_text(u))).unwrap_or_else(|| "no user event before it".into());
                let detail = format!("{} {} at {} ({after})", e["level"].as_str().unwrap_or(""), e["msg"].as_str().unwrap_or(""), e["where"].as_str().unwrap_or(""));
                flag(flags, "error", e, detail, Some("-".into()));
            }
            "screen" if e["leaked"].as_i64().unwrap_or(0) > 0 => {
                let detail = format!("{} node(s) of the previous screen left (now {})", e["leaked"].as_i64().unwrap_or(0), e["path"].as_str().unwrap_or(""));
                flag(flags, "leak", e, detail, Some("-".into()));
            }
            "asset" => {
                let s = |k: &str| e[k].as_str().unwrap_or("").to_string();
                flag(flags, "asset_fallback", e, format!("{}/{}/{} {} in {}", s("ns"), s("key"), s("variant"), s("source"), s("screen")), Some("-".into()));
            }
            _ => {}
        }
    }
}

fn churn(trail: &[Value], flags: &mut Vec<Value>) {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for e in trail.iter().filter(|e| ty(e) == "plan_edit" && CHURN_KINDS.contains(&e["kind"].as_str().unwrap_or(""))) {
        let unit = match &e["unit"] {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        let round = e["round"].as_i64().unwrap_or(0);
        let key = format!("{unit}|{round}");
        let n = match counts.iter_mut().find(|c| c.0 == key) {
            Some(c) => {
                c.1 += 1;
                c.1
            }
            None => {
                counts.push((key, 1));
                1
            }
        };
        if n == CHURN_MIN {
            flag(flags, "undo_churn", e, format!("{n} remove / undo / clear edits for {unit} in round {round}"), Some(unit));
        }
    }
}

fn open_close(trail: &[Value], flags: &mut Vec<Value>) {
    let mut open: Vec<(String, Value, bool)> = Vec::new();
    let mut screen_t = -1;
    let mut last_user: Option<&Value> = None;
    for e in trail {
        let t = t_of(e);
        if USER_EVENTS.contains(&ty(e)) {
            last_user = Some(e);
        }
        if ty(e) == "screen" {
            left_open(&open, flags, &format!("screen change to {}", e["path"].as_str().unwrap_or("")));
            open.clear();
            screen_t = t;
            continue;
        }
        if ty(e) != "panel" && ty(e) != "modal" {
            continue;
        }
        let key = target_text(e);
        if e["open"].as_bool().unwrap_or(false) {
            let at_start = screen_t >= 0 && t - screen_t <= SCENE_START_MS;
            if open.iter().any(|o| o.0 == key) {
                flag(flags, "double_open", e, format!("{} opened again without a close", ty(e)), None);
            } else if !at_start {
                match last_user.map(|u| t - t_of(u)) {
                    None => flag(flags, "unprompted_open", e, format!("{} opened with no click, button or key in the {PROMPT_WINDOW_MS} ms before", ty(e)), None),
                    Some(since) if since > PROMPT_WINDOW_MS => flag(flags, "unprompted_open", e, format!("{} opened with no click, button or key in the {PROMPT_WINDOW_MS} ms before", ty(e)), None),
                    Some(since) if since > SLOW_OPEN_MS => flag(flags, "slow_open", e, format!("{} opened {since} ms after {}", ty(e), event_text(last_user.unwrap())), None),
                    Some(_) => {}
                }
            }
            open.retain(|o| o.0 != key);
            open.push((key, e.clone(), at_start));
        } else {
            if !open.iter().any(|o| o.0 == key) {
                flag(flags, "close_without_open", e, format!("{} closed but never opened", ty(e)), None);
            }
            open.retain(|o| o.0 != key);
        }
    }
    left_open(&open, flags, "end of the recording");
}

fn left_open(open: &[(String, Value, bool)], flags: &mut Vec<Value>, at: &str) {
    for (_, e, own) in open {
        if !own {
            flag(flags, "left_open", e, format!("{} still open at the {at}", ty(e)), None);
        }
    }
}

fn nearest_note(notes: &[Value], t: i64) -> i64 {
    notes
        .iter()
        .filter(|n| t_of(n) <= t && t - t_of(n) <= NOTE_WINDOW_MS)
        .max_by_key(|n| t_of(n))
        .and_then(|n| n["n"].as_i64())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(r: &Value, k: &str) -> i64 {
        r["counts"][k].as_i64().unwrap()
    }

    /// PLAYTEST1 acceptance 12 and 13 on a synthetic trail.
    #[test]
    fn a_synthetic_trail_gives_exactly_the_expected_flags() {
        let ctl = |p: &str, cls: &str| json!({"path": p, "cls": cls});
        let trail = vec![
            json!({"t": 0, "type": "screen", "path": "arena", "leaked": 0}),
            json!({"t": 100, "type": "modal", "ctl": ctl("own_modal", "screen"), "open": true}),
            json!({"t": 2000, "type": "click", "button": "left", "pos": [5, 5], "ctl": ctl("bg", "label"), "handled": false}),
            json!({"t": 3000, "type": "click", "button": "left", "ctl": ctl("go", "button"), "handled": true}),
            json!({"t": 3050, "type": "button", "ctl": ctl("go", "button")}),
            json!({"t": 3700, "type": "panel", "ctl": ctl("slow", "screen"), "open": true}),
            json!({"t": 3800, "type": "panel", "ctl": ctl("slow", "screen"), "open": false}),
            json!({"t": 4000, "type": "click", "ctl": ctl("x", "button"), "handled": true}),
            json!({"t": 4200, "type": "click", "ctl": ctl("x", "button"), "handled": true}),
            json!({"t": 4400, "type": "click", "ctl": ctl("x", "button"), "handled": true}),
            json!({"t": 4600, "type": "click", "ctl": ctl("x", "button"), "handled": true}),
            json!({"t": 4650, "type": "button", "ctl": ctl("x", "button")}),
            json!({"t": 5000, "type": "hover", "ctl": ctl("y", "button"), "ms": 3500}),
            json!({"t": 5100, "type": "error", "level": "error", "msg": "boom", "where": "a.rs:1"}),
            json!({"t": 7000, "type": "modal", "ctl": ctl("pop", "screen"), "open": true}),
            json!({"t": 7100, "type": "modal", "ctl": ctl("pop", "screen"), "open": true}),
            json!({"t": 8000, "type": "screen", "path": "menu", "leaked": 1}),
        ];
        let notes = vec![json!({"n": 1, "t": 1500})];
        let r = build(&trail, 0, &notes);
        for (k, n) in [("dead_click", 1), ("repeat_click", 1), ("long_hover", 1), ("error", 1), ("slow_open", 1), ("unprompted_open", 1), ("double_open", 1), ("left_open", 1), ("leak", 1)] {
            assert_eq!(count(&r, k), n, "{k}: {:#}", r["flags"]);
        }
        let error = r["flags"].as_array().unwrap().iter().find(|f| f["kind"] == "error").unwrap();
        assert!(error["detail"].as_str().unwrap().contains("after button x t=4650"), "{error}");
        assert_eq!(error["note"], json!(1));
        let ls = lines(&r);
        assert!(ls.last().unwrap().starts_with("trail report: 17 event(s), 0 dropped, 9 flag(s)"), "{}", ls.last().unwrap());
    }
}
