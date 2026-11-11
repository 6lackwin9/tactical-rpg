use crate::battle::BattleScreen;
use crate::screens::Screen;
use serde_json::json;
use emberwake_rules::ruleset::{Ruleset, PLAN_BADGES, TIMELINE};

/// RULESET1-G (index 12): the Arena passes its extension switches and "Reveal enemy plans" to the battle (and
/// they survive Restart); without them the request is the classic one: no ruleset, no reveal, no state key.
#[test]
fn the_arena_plays_by_its_switches_and_reveal() {
    let cx = crate::hud::hud_tests::game_cx();
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let classic = BattleScreen::arena(&cx, None, &encounter);
    let s = classic.host.session.as_ref().expect("the battle starts");
    assert!(s.w.ruleset.is_classic() && s.w.ruleset.rules.is_empty() && !s.w.plan_reveal);
    assert!(classic.state(&cx).get("ruleset").is_none(), "a classic battle's state keeps its shape");
    let timeline = Ruleset::of(&cx.data.cat, TIMELINE).expect("the catalog's timeline row");
    let off = timeline.rules.iter().copied().find(|id| emberwake_rules::ruleset::rule(id).is_some_and(|r| r.kind == emberwake_rules::ruleset::RuleKind::Extension) && *id != PLAN_BADGES).expect("an extension besides plan_badges");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.into()), &format!("-{off},+{PLAN_BADGES}"), true);
    for start in ["first", "restarted"] {
        let s = b.host.session.as_ref().expect("the battle starts");
        assert!(!s.w.ruleset.has(off) && s.w.ruleset.has(PLAN_BADGES), "{start}: the switches apply");
        assert!(s.w.plan_reveal, "{start}: the plans are revealed");
        let st = b.state(&cx);
        assert_eq!((st["ruleset"]["id"].clone(), st["ruleset"]["plan_reveal"].clone()), (json!(TIMELINE), json!(true)), "{start}");
        // what the Arena's Restart does (without the playtest hub's save)
        b.start_battle();
    }
}

/// INPUT1 MR11: the Arena's "Diagonal melee" pick reaches the `new` request (recorded), the battle plays by it and
/// keeps it on Restart; the battle's state and chip name it; "" (Ruleset) sends nothing and the row's value plays.
#[test]
fn the_arena_diagonal_melee_pick_reaches_the_battle_and_survives_restart() {
    let cx = crate::hud::hud_tests::game_cx();
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_full(&cx, None, &encounter, Some(TIMELINE.into()), "", "all", false);
    for start in ["first", "restarted"] {
        let s = b.host.session.as_ref().expect("the battle starts");
        assert_eq!(s.w.ruleset.melee_diagonal(), emberwake_rules::ruleset::MeleeDiagonal::All, "{start}");
        let st = b.state(&cx);
        assert_eq!(st["ruleset"]["melee_diagonal"], json!("all"), "{start}");
        assert!(st["ruleset"]["chip"].as_str().unwrap_or("").contains("diagonal: all"), "{start}: {}", st["ruleset"]["chip"]);
        let history = b.host.handle(&json!({"cmd": "history"}))["history"].as_array().cloned().unwrap_or_default();
        assert!(history.iter().any(|q| q["cmd"] == json!("new") && q["melee_diagonal"] == json!("all")), "{start}: the request carries it");
        b.start_battle();
    }
    let mut row = BattleScreen::arena_full(&cx, None, &encounter, Some(TIMELINE.into()), "", "", false);
    let history = row.host.handle(&json!({"cmd": "history"}))["history"].as_array().cloned().unwrap_or_default();
    assert!(history.iter().all(|q| q.get("melee_diagonal").is_none()), "Ruleset: nothing sent");
    let timeline = Ruleset::of(&cx.data.cat, TIMELINE).expect("the catalog's timeline row");
    assert_eq!(row.host.session.as_ref().unwrap().w.ruleset.melee_diagonal(), timeline.melee_diagonal(), "the row's value");
}

/// LAYER-ENG-API (PR5-29c): a playtest note over the battle names the recording's session under `session` (the engine
/// shell's stamp key; `battle` before), with the battle's own round, block and phase; no `battle` key.
#[test]
fn a_note_over_the_battle_names_its_session_round_block_and_phase() {
    let cx = crate::hud::hud_tests::game_cx();
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let b = BattleScreen::arena(&cx, None, &encounter);
    let s = b.host.session.as_ref().expect("the battle starts");
    let ctx = b.note_context(&cx);
    assert_eq!(ctx["session"], json!(-1), "not recorded: no session slot: {ctx}");
    assert_eq!((ctx["round"].clone(), ctx["block"].clone(), ctx["phase"].clone()), (json!(s.current_round), json!(s.current_block), json!("planning")), "{ctx}");
    assert!(ctx.get("battle").is_none(), "{ctx}");
}
