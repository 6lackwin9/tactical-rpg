//! HOOKS1-UI slice 4 (HOOKS1 5): the selected hero's push and pull forecasts. A catalog encounter played by the
//! catalog's timeline row with forced_move switched on; the hero's attack skills get a test push 2 in the battle's
//! catalog. Every expectation is the kit's `forced::forecast`; the kit test `forced_move.rs` checks that a forecast is
//! the move the hit makes.

use crate::battle::marks::{push_forecasts, push_json, push_text};
use crate::battle::overlays::strike_tile;
use crate::battle::BattleScreen;
use crate::screens::{Cx, Screen};
use aegis_ui::ui::Gfx;
use emberwake_battle::command::CmdKind;
use emberwake_battle::session::Session;
use aegis_ui::art::Art;
use serde_json::{json, Value};
use emberwake_rules::ruleset::TIMELINE;

fn arena_screen(rules: &str) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), rules, false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn tip_text(b: &BattleScreen, id: &str) -> String {
    b.w.iter().filter(|x| x.id == id).flat_map(|x| x.tip.iter().map(|(t, _)| t.clone())).collect::<Vec<_>>().join(" | ")
}

/// The hero's attack skills (every rank) push 2 (a test row on the battle's catalog), the hero plans Assault on its first foe.
pub(super) fn plan_push(cx: &Cx, b: &mut BattleScreen) -> (usize, usize) {
    let h = b.selected.expect("a hero");
    let sess = b.host.session.as_mut().unwrap();
    let w = &mut sess.w;
    let f = w.enemies.iter().copied().find(|&e| w.units[e].is_active() && w.has(e)).expect("a foe");
    let known = w.units[h].known_skills.clone();
    let push = emberwake_data::ForcedMove::from_obj(json!({"kind": "push", "tiles": 2, "collide_mult": 0.5}).as_object().unwrap());
    // every rank of a known attack: a rank 2+ row is `<head>_<rank>` with the head as its `family` (the loader's rank rule)
    for sk in std::sync::Arc::make_mut(&mut w.cat).skills.iter_mut().filter(|k| (known.contains(&k.id) || known.contains(&k.family)) && k.kind == "ATTACK") {
        sk.forced_move = push.clone();
    }
    let r = b.send(cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": f}));
    assert_eq!(r["ok"], json!(true), "{r}");
    (h, f)
}

/// The forecasts are the kit's for each planned entry; each one shows on its strip card and in the state; the board
/// draws them (a frame differs with the layer off, and then no card line and no state entry stays).
#[test]
fn the_push_forecasts_follow_the_kit() {
    let (mut cx, mut g, mut b) = arena_screen("+forced_move");
    let (h, f) = plan_push(&cx, &mut b);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let sess = s(&b);
    let w = &sess.w;
    let per_entry = push_forecasts(sess, h);
    let entries = sess.plan.compile(w, h).entries;
    assert_eq!(per_entry.len(), entries.len());
    for (e, got) in entries.iter().zip(&per_entry) {
        let cmd = e.build_command(w, h);
        let targets: Vec<usize> = match &cmd.kind {
            CmdKind::Attack { target, .. } => target.iter().copied().collect(),
            CmdKind::Area { cells, .. } => cells.iter().filter_map(|c| w.occupant(*c)).collect(),
            _ => Vec::new(),
        };
        assert_eq!(*got, emberwake_battle::forced::forecast(w, &sess.resolver.scheduled, h, strike_tile(e), &cmd, e.end_block(w), &targets));
    }
    let all: Vec<&emberwake_battle::forced::Forecast> = per_entry.iter().flatten().collect();
    assert!(all.iter().any(|x| x.unit == f), "the planned attack on the foe forecasts its push");
    for x in &all {
        let line = push_text(w, x).0;
        assert!((1..=sess.def.total_blocks).any(|blk| tip_text(&b, &format!("strip.block{blk}")).contains(&line)), "{line}");
    }
    let st = b.state(&cx);
    assert_eq!(st["push_forecasts"], Value::Array(all.iter().map(|x| push_json(x)).collect()));
    let on = g.canvas.px.clone();
    cx.settings.v["battle_overlays"] = json!({"pushes": false});
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_ne!(on, g.canvas.px, "the arrows draw");
    assert!(!(1..=s(&b).def.total_blocks).any(|blk| tip_text(&b, &format!("strip.block{blk}")).contains("Pushes ")));
    assert_eq!(b.state(&cx)["push_forecasts"], json!([]));
}

/// Without the rule nothing is forecast and the state has no `push_forecasts` key.
#[test]
fn without_the_rule_nothing_is_forecast() {
    let (mut cx, mut g, mut b) = arena_screen("-forced_move");
    let (h, _) = plan_push(&cx, &mut b);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(push_forecasts(s(&b), h).is_empty());
    assert!(b.state(&cx).get("push_forecasts").is_none());
}
