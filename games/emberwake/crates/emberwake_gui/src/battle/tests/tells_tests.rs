//! RULESET1-U slice 4 (RULESET1_INTENT UI5, RULESET1 V1, V2): the round intent in tells and threat intel, the shared
//! glyph's place, the impact badges and the full reveal on the battle screen. A catalog encounter played by the
//! catalog's timeline row; expectations come from the kit's records (`Session::enemy_intents`, `round_intents`) and the
//! text helpers, never from shipped numbers. Classic keeps its cards, intel and widgets.

use crate::battle::marks::arrows_of;
use crate::battle::overlays::Layers;
use crate::battle::{Action, BattleScreen};
use crate::battle::tips::unit_tip;
use crate::intent_text;
use crate::screens::{Cx, Screen};
use emberwake_battle::intent::Intent;
use aegis_ui::ui::{Gfx, Widget};
use emberwake_battle::command::CmdKind;
use emberwake_battle::grid::V2;
use emberwake_battle::session::Session;
use aegis_ui::art::Art;
use serde_json::{json, Value};
use emberwake_rules::ruleset::{PLAN_BADGES, TIMELINE};

fn arena(ruleset: Option<&str>, reveal: bool) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", reveal);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn widget<'a>(b: &'a BattleScreen, id: &str) -> &'a Widget<Action> {
    b.w.iter().find(|w| w.id == id).unwrap_or_else(|| panic!("no widget {id}"))
}

fn passive_name(p: &str) -> String {
    p.to_string()
}

/// UI5: each hostile card leads with the enemy's round intent and target ("Assault: Wolf; Cut B1"); a party member
/// with an intent gets the glyph on the strip (the title gives way); V1: each card has its badge row, one tip line
/// per impact badge at its block.
#[test]
fn the_cards_lead_with_the_intent_and_carry_the_badges() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
    assert!(s(&b).w.ruleset.has(PLAN_BADGES), "the timeline row shows badges");
    let h = b.selected.unwrap();
    let foe = s(&b).w.enemies.iter().copied().find(|&e| s(&b).w.units[e].is_active() && s(&b).w.has(e)).unwrap();
    let r = b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}));
    assert_eq!(r["ok"], json!(true), "{r}");
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let w = &s(&b).w;
    let tells = s(&b).enemy_intents();
    assert!(!tells.is_empty());
    let mut led = 0;
    for t in &tells {
        let card = widget(&b, &format!("foe.{}", t.unit));
        let line = intent_text::round_card_line(w, t);
        assert!(card.text.ends_with(&line), "{} vs {line}", card.text);
        if let Some(i) = t.round_intent {
            assert!(line.starts_with(i.name()), "{line}");
            led += 1;
        }
        let row = widget(&b, &format!("foe.{}.badges", t.unit));
        let marks: Vec<String> = row.tip.iter().skip(1).map(|x| x.0.clone()).collect();
        if t.impacts.is_empty() {
            assert_eq!(marks, ["Nothing lands this round"]);
        } else {
            assert_eq!(marks.len(), t.impacts.len(), "{marks:?}");
            for ((blk, _), m) in t.impacts.iter().zip(&marks) {
                assert!(m.starts_with(&format!("B{blk}:")), "{m}");
            }
        }
    }
    assert!(led > 0, "the enemies picked their intents at the round start (AI1)");
    assert_eq!(widget(&b, "strip.title").rect.w, 68, "the hero's glyph takes the title's end");
}

/// UI5: the threat intel panel shows the hovered unit of either side: over a party card it is that member, with
/// its round intent line; over nothing, the first foe.
#[test]
fn the_intel_panel_shows_either_side() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
    let h = b.selected.unwrap();
    let foe = s(&b).w.enemies.iter().copied().find(|&e| s(&b).w.units[e].is_active() && s(&b).w.has(e)).unwrap();
    b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}));
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(widget(&b, "intel.unit").text, s(&b).w.units[foe].name, "nothing hovered: the first foe");
    let card = widget(&b, &format!("party.{h}")).rect;
    b.draw(&mut cx, &mut g, (card.x + card.w / 2, card.y + card.h / 2), true);
    assert_eq!(widget(&b, "intel.unit").text, s(&b).w.units[h].name, "a party card hovered: that member");
    let rounds = s(&b).round_intents();
    let head = intent_text::round_head(&s(&b).w, rounds.iter().find(|r| r.unit == h).unwrap()).expect("the hero's intent");
    assert!(unit_tip(s(&b), h, Some(h), &[], &rounds, &[]).iter().any(|t| t.0 == head), "{head}");
    assert!(head.starts_with("Assault: "), "{head}");
}

/// V2: with the plan reveal every enemy's tip says so and lists its plan; VISCLAR1 AR1, AR2 (they replace slice 5's
/// tile-by-tile revealed walks): a revealed enemy whose round opens with a walk has its target arrow's move leg
/// end where the kit's walk preview ends; an unrevealed enemy's arrow goes straight (two points).
#[test]
fn the_plan_reveal_shows_every_enemy_plan() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), true);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let w = &s(&b).w;
    let tells = s(&b).enemy_intents();
    let rounds = s(&b).round_intents();
    assert!(tells.iter().all(|t| t.revealed), "V2: every enemy");
    for t in &tells {
        let head = intent_text::reveal_head(w, t, passive_name).unwrap();
        let tip = unit_tip(s(&b), t.unit, b.selected, &tells, &rounds, &[]);
        assert!(tip.iter().any(|x| x.0 == head), "{head}");
        if t.readers.is_empty() && !t.enraged {
            assert_eq!(head, "Plan revealed");
        }
        for l in &t.plan {
            assert!(tip.iter().any(|x| x.0 == intent_text::plan_line(w, l)));
        }
    }
    // AR2: each enemy whose round opens with walks, and where the kit's preview of the last of those walks ends
    // (the walks before its first command)
    let opening_walks = |sess: &Session| -> Vec<(usize, V2)> {
        let w = &sess.w;
        w.enemies
            .iter()
            .copied()
            .filter_map(|e| {
                let mut acts: Vec<&emberwake_battle::resolver::ScheduledAction> = sess.resolver.scheduled.iter().filter(|a| a.unit == e && !a.cancelled).collect();
                acts.sort_by_key(|a| a.start);
                let ends: Vec<V2> = acts
                    .iter()
                    .map_while(|a| match &a.command.kind {
                        CmdKind::Walk { path, expected_start, .. } => emberwake_battle::command::walk_preview(w, e, path, *expected_start).last().copied(),
                        _ => None,
                    })
                    .collect();
                // a walk whose command also carries a dash is left out (the dash ends the leg)
                let carried = acts.get(ends.len()).is_some_and(|a| matches!(a.command.carry, emberwake_battle::command::Carry::Engage(_)));
                ends.last().copied().filter(|end| *end != w.pos(e) && !carried).map(|end| (e, end))
            })
            .collect()
    };
    let all = |b: &BattleScreen| arrows_of(s(b), &s(b).enemy_intents(), &s(b).round_intents(), &Layers::of(&Value::Null), true);
    let walks = opening_walks(s(&b));
    assert!(!walks.is_empty(), "the enemies walk at the round start");
    let arrows = all(&b);
    let mut legs = 0;
    // (an enemy with no target draws no arrow, AR1; a leg that ends on its target tile is the arrow's end)
    for (e, end) in &walks {
        let Some(a) = arrows.iter().find(|a| a.unit == *e) else { continue };
        assert_eq!(a.points[0], w.pos(*e));
        assert_eq!(a.points[1], *end, "{a:?}");
        legs += 1;
    }
    assert!(legs > 0, "a walking enemy with a target: {arrows:?}");
    // slice 5 review, AR2: an unrevealed enemy's arrow has no move leg (no reveal, no foresight)
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), false);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let tells = s(&b).enemy_intents();
    let hidden: Vec<usize> = tells.iter().filter(|t| !t.revealed).map(|t| t.unit).collect();
    assert!(opening_walks(s(&b)).iter().any(|(e, _)| hidden.contains(e)), "an unrevealed enemy walks this round");
    let arrows = all(&b);
    assert!(arrows.iter().filter(|a| hidden.contains(&a.unit)).all(|a| a.points.len() == 2), "{arrows:?}");
}

/// The slice 2 review: a foresight reader who falls during the playback stays a reader for that round, and the
/// enemy's card still names it.
#[test]
fn a_reader_who_falls_stays_listed() {
    let (cx, _g, mut b) = arena(Some(TIMELINE), false);
    let h = b.selected.unwrap();
    {
        let sess = b.host.session.as_mut().unwrap();
        sess.w.units[h].foresight_source = "t_sight".into();
        sess.w.units[h].foresight_range = emberwake_rules::unit::FORESIGHT_RANGE_MAX;
        sess.measure_reveals();
    }
    let r = b.send(&cx, json!({"cmd": "confirm"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    b.host.session.as_mut().unwrap().w.units[h].hp = 0;
    let sess = s(&b);
    assert!(!sess.w.units[h].is_active(), "the reader is down");
    let rounds = sess.round_intents();
    let read: Vec<&Intent> = rounds.iter().filter(|r| sess.w.enemies.contains(&r.unit) && r.readers.contains(&h)).collect();
    assert!(!read.is_empty(), "the range reached a foe at scheduling");
    for r in read {
        let tip = unit_tip(sess, r.unit, None, &[], &rounds, &[]);
        let line = intent_text::read_by(&sess.w, r, passive_name);
        assert!(tip.iter().any(|x| x.0 == line) && line.contains(&sess.w.units[h].name), "{line}");
    }
}

/// Classic: the cards keep their "Next:" line, no badge row, no intel widget, no glyph space on the strip.
#[test]
fn classic_cards_and_intel_are_unchanged() {
    let (mut cx, mut g, mut b) = arena(None, false);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let w = &s(&b).w;
    assert!(s(&b).round_intents().is_empty());
    for t in s(&b).enemy_intents() {
        assert!(widget(&b, &format!("foe.{}", t.unit)).text.ends_with(&intent_text::card_line(w, &t.line)));
    }
    assert!(b.w.iter().all(|x| !x.id.ends_with(".badges") && x.id != "intel.unit"));
    assert_eq!(widget(&b, "strip.title").rect.w, 80);
}
