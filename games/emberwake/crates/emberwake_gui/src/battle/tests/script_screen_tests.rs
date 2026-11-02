//! ENCOUNTER2-C4 (docs/specs/ENCOUNTER2_C.md section 8, the C4 row): the cutscene screen on a scratch catalog (no
//! shipped content): a locked round refuses the planning controls, shows its banner and Skip, plays by itself and
//! starts the next round after the round-end pause; Skip sends `resolve`; a guided round shows the banner and keeps
//! planning; the party tactic reaches the script's auto heroes (decision "ENCOUNTER2-C3 answers" (1)).

use crate::battle::input::RowAct;
use crate::battle::{Action, BattleScreen, Mode, TACTICS};
use crate::screens::{Cx, Key, Screen};
use emberwake_battle::grid::V2;
use emberwake_battle::session::Phase;
use aegis_core::json;
use serde_json::{json, Value};
use std::time::Instant;
use crate::hud::hud_tests::{frame, game_cx, gfx};
use emberwake_world::Data;
use std::sync::Arc;

const BANNER: &str = "Watch them";

/// The scratch catalog: two heroes and a foe under a timeline ruleset; `t_locked` scripts round 1 locked,
/// `t_guided` the same round guided; round 2 is not scripted.
fn data() -> Data {
    let skills = json!([
        {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
        {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
        {"id": "switch_target", "name": "Switch Target", "kind": "TARGET", "level": 1, "blocks": 1, "system": true, "basic": true},
        {"id": "staggered", "name": "Staggered", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
        {"id": "stunned", "name": "Stunned", "kind": "STATUS", "level": 1, "blocks": 1, "system": true},
        {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
        {"id": "active_guard", "name": "Active Guard", "kind": "DEFEND", "round_intent": "guard", "level": 1, "blocks": 2, "ap_cost": 1, "mitigation": 0.5},
        {"id": "t_cleave", "name": "Cleave", "kind": "ATTACK", "round_intent": "assault", "level": 1, "blocks": 2, "ap_cost": 2, "power_mult": 1.5, "target_mode": "ENEMY"}
    ]);
    let unit = |id: &str, side: &str| {
        json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 4000, "speed": 12, "attack_range": 1, "skill_cap": 5, "deck": {"order": {"assault": ["t_cleave", "strike"], "guard": ["active_guard"]}}})
    };
    let ruleset = json!({"id": "timeline", "name": "Timeline", "rules": ["round_intent", "ap_cost"], "basics": ["move", "rest", "strike", "active_guard"]});
    let encounter = |id: &str, lock: bool| {
        let script = json!([{"round": 1, "lock": lock, "others": "auto", "banner": BANNER, "entries": [
            {"unit": "party:t_hero", "target": "self", "picks": [{"skill": "active_guard"}], "fill": "rest"},
            {"unit": "t_foe", "target": "self", "picks": [{"skill": "active_guard"}], "fill": "auto"}
        ]}]);
        json!({"id": id, "name": id, "party": ["t_hero", "t_mate"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 8, "ap_regen": 4, "max_rounds": 3, "script": script})
    };
    let catalog = json!({"skills": skills, "units": [unit("t_hero", "party"), unit("t_mate", "party"), unit("t_foe", "enemy")], "rulesets": [ruleset],
        "encounters": [encounter("t_locked", true), encounter("t_guided", false), json!({"id": "t_plain", "name": "t_plain", "party": ["t_hero", "t_mate"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": 8, "max_rounds": 3})]});
    Data::from_values(&catalog, &json!({}), &json!({}))
}

/// A debug game over the scratch catalog (skip animations on, so no impact holds a step).
fn script_cx() -> Cx {
    let mut cx = game_cx();
    cx.data = Arc::new(data());
    cx.settings.v["battle_view"] = json!("flat");
    cx.settings.v["skip_anim"] = json!(true);
    cx
}

fn arena(cx: &Cx, encounter: &str) -> BattleScreen {
    let b = BattleScreen::arena_with(cx, None, encounter, Some("timeline".into()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    b
}

fn tick_ready(b: &mut BattleScreen, cx: &mut Cx) {
    b.next_at = Instant::now();
    b.gap_until = None;
    b.tick(cx);
}

fn widget(b: &BattleScreen, id: &str) -> Option<Value> {
    b.widgets().into_iter().find(|w| w["id"] == id)
}

fn cmds(b: &BattleScreen) -> Vec<String> {
    b.host.history.iter().map(|r| r["cmd"].as_str().unwrap_or("").to_string()).collect()
}

fn intent_of(b: &mut BattleScreen, name: &str) -> Value {
    let plan = b.host.handle(&json!({"cmd": "round_plan"}))["round_plan"].clone();
    json::arr(&plan).iter().find(|r| r["name"] == json!(name)).map(|r| r["intent"].clone()).unwrap_or(Value::Null)
}

/// SR5 on the screen: the planning controls send nothing; the banner and Skip show, Confirm and the tactic menus do
/// not act; the agent link reads the lock.
#[test]
fn a_locked_round_refuses_planning_and_shows_its_banner_and_skip() {
    let (mut cx, mut g) = (script_cx(), gfx());
    let mut b = arena(&cx, "t_locked");
    assert!(b.locked() && !b.planning(), "the script confirmed round 1");
    frame(&mut b, &mut cx, &mut g, 1.0);
    assert_eq!(widget(&b, "battle.banner").map(|w| w["text"].clone()), Some(json!(BANNER)));
    assert!(widget(&b, "battle.skip").is_some_and(|w| w["enabled"] == json!(true)), "Skip in the Confirm slot");
    assert!(widget(&b, "strip.confirm").is_none());
    for id in ["strip.party", "strip.hero", "strip.clear"] {
        assert!(widget(&b, id).is_none_or(|w| w["enabled"] == json!(false)), "{id} is locked");
    }
    let st = b.state(&cx);
    assert_eq!(st["script"], json!({"round": 1, "lock": true, "banner": BANNER, "skip": true}));
    let sent = b.host.history.len();
    let hero = b.session().w.allies[0];
    for a in [Action::Confirm, Action::Clear, Action::Undo, Action::SelfTarget, Action::TileClick(V2::new(3, 3))] {
        b.act(&mut cx, a);
    }
    b.selected = Some(hero);
    for k in [Key::Enter, Key::Char('1'), Key::Char('s'), Key::Char('z')] {
        b.key(&mut cx, k);
    }
    assert_eq!(b.host.history.len(), sent, "no planning request goes out: {:?}", cmds(&b));
}

/// The locked round plays by itself from the first tick and, at its end, the next round starts after the
/// round-end pause (without autoplay); round 2 is not scripted, so planning is back and the script key is gone.
#[test]
fn a_locked_round_plays_by_itself_and_advances() {
    let mut cx = script_cx();
    let mut b = arena(&cx, "t_locked");
    assert!(!b.auto);
    b.tick(&mut cx);
    assert!(b.playing, "the playback starts by itself");
    for _ in 0..200 {
        if b.session().phase == Phase::Replay {
            break;
        }
        tick_ready(&mut b, &mut cx);
    }
    assert_eq!(b.session().phase, Phase::Replay);
    assert!(cmds(&b).iter().any(|c| c == "step"), "{:?}", cmds(&b));
    assert!(b.pending_next_round, "the next round is due after the round-end pause");
    tick_ready(&mut b, &mut cx);
    assert_eq!(b.session().current_round, 2);
    assert!(b.planning() && !b.locked());
    assert!(b.state(&cx).get("script").is_none());
    tick_ready(&mut b, &mut cx);
    assert!(!b.playing && b.planning(), "an unscripted round waits for the player");
}

/// Skip sends `resolve`: the round ends at once with the same log as resolving it through the host, and the next
/// round follows after the pause.
#[test]
fn skip_resolves_the_rest_of_the_locked_round() {
    let mut cx = script_cx();
    let mut b = arena(&cx, "t_locked");
    b.tick(&mut cx);
    b.act(&mut cx, Action::Skip);
    assert_eq!(cmds(&b).last().map(String::as_str), Some("resolve"));
    assert_eq!(b.session().phase, Phase::Replay);
    assert!(!b.playing && b.pending_next_round);
    let mut direct = emberwake_host::Host::new(emberwake_host::Battle::with_data(cx.data.clone()));
    assert_eq!(direct.handle(&b.host.history[0].clone())["ok"], json!(true));
    direct.handle(&json!({"cmd": "resolve"}));
    assert_eq!(direct.handle(&json!({"cmd": "log"}))["log"], b.host.handle(&json!({"cmd": "log"}))["log"], "the same round as resolving it at once");
    b.act(&mut cx, Action::Skip);
    assert_eq!(cmds(&b).iter().filter(|c| *c == "resolve").count(), 1, "nothing to skip once the round is over");
    tick_ready(&mut b, &mut cx);
    assert_eq!(b.session().current_round, 2);
}

/// SR6 on the screen: a guided round shows the banner but keeps planning (Confirm, the tactic menus, no Skip, no
/// playback of its own); the party tactic sends the tactics to the kit.
#[test]
fn a_guided_round_shows_the_banner_and_keeps_planning() {
    let (mut cx, mut g) = (script_cx(), gfx());
    let mut b = arena(&cx, "t_guided");
    assert!(!b.locked() && b.planning());
    b.tick(&mut cx);
    assert!(!b.playing, "a guided round waits for Confirm");
    frame(&mut b, &mut cx, &mut g, 1.0);
    assert_eq!(widget(&b, "battle.banner").map(|w| w["text"].clone()), Some(json!(BANNER)));
    assert!(widget(&b, "battle.skip").is_none());
    assert!(widget(&b, "strip.confirm").is_some_and(|w| w["enabled"] == json!(true)));
    assert!(widget(&b, "strip.party").is_some_and(|w| w["enabled"] == json!(true)));
    assert_eq!(b.state(&cx)["script"], json!({"round": 1, "lock": false, "banner": BANNER, "skip": false}));
    b.act(&mut cx, Action::PartyMenu);
    let row = b.popup.as_ref().and_then(|p| p.rows.iter().position(|r| matches!(r.act, RowAct::PartyTactic(3)))).expect("the Guard and Counter row");
    b.act(&mut cx, Action::Choose(row));
    let last = b.host.history.last().cloned().unwrap_or(Value::Null);
    assert_eq!((last["cmd"].clone(), last["party"].clone()), (json!("tactics"), json!(TACTICS[3])));
    b.act(&mut cx, Action::Confirm);
    assert!(b.playing, "Confirm plays the guided round");
}

/// Decision "ENCOUNTER2-C3 answers" (1): the Arena's party tactic drafts the locked round's hero with no entry
/// (Guard and Counter: Guard on itself), where the kit's default drafts by the AI table (no Guard at full HP).
#[test]
fn the_party_tactic_drafts_the_scripted_rounds_auto_heroes() {
    let cx = script_cx();
    let mut plain = arena(&cx, "t_locked");
    assert_ne!(intent_of(&mut plain, "t_mate"), json!("guard"));
    let mut b = BattleScreen::blank(&cx, Mode::Arena);
    b.encounter = "t_locked".into();
    b.arena_ruleset = Some("timeline".into());
    b.party_tactic = 3;
    b.start_battle();
    assert_eq!(b.host.history[0]["tactics"], json!({"party": TACTICS[3]}));
    assert_eq!(intent_of(&mut b, "t_mate"), json!("guard"));
    // a battle with no script, or a script under classic, starts with the request it always had
    for (enc, ruleset) in [("t_plain", Some("timeline")), ("t_locked", None)] {
        b.encounter = enc.into();
        b.arena_ruleset = ruleset.map(String::from);
        b.start_battle();
        assert!(b.host.history[0].get("tactics").is_none(), "{enc} {ruleset:?}");
    }
}
