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
    let t = widget(&b, "strip.title").rect;
    assert_eq!(t.x + t.w, 76, "the hero's glyph takes the title's end");
}

/// UI5, BATTLE-UI2 (contract 5): the docked foe card shows the hovered foe and stays on a foe: over nothing the first
/// foe, over a party plate still the foe (the member's round intent line is on the plate's hover).
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
    assert_eq!(widget(&b, "intel.unit").text, s(&b).w.units[foe].name, "a party plate hovered: the card stays on the foe");
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
    assert!(b.w.iter().all(|x| !x.id.ends_with(".badges") && x.id != "intel.unit"), "classic: the card is `card.unit`");
    let t = widget(&b, "strip.title").rect;
    assert_eq!(t.x + t.w, 88);
}

/// PR7-41 (PT-20): the forecast line names the blocks the hits land (never a move's block) and each hit by the kit's
/// one convention (`intent::hits_on`: per hit, at the block it lands, with guard); an unrevealed plan names its first
/// hit only; a first action on the hero with no hit reads "Heads for you".
#[test]
fn the_forecast_names_the_landing_block_and_each_hit() {
    use emberwake_battle::intent::HitOn;
    let (_cx, _g, b) = arena(Some(TIMELINE), false);
    let w = &s(&b).w;
    let mut t = s(&b).enemy_intents().into_iter().next().expect("a tell");
    let h = w.allies[0];
    t.line.target = Some(h);
    t.line.blocks = (1, 2);
    let hit = |block: i32, damage: i32, guarded: bool| HitOn { skill: "Thrust".into(), block, damage, guarded, nth: 1, fails: None };
    let nth = |h: HitOn, nth: usize| HitOn { nth, ..h };
    assert_eq!(intent_text::forecast(&t, h, &[]).as_deref(), Some("Heads for you in B1-2"));
    assert_eq!(intent_text::forecast(&t, h, &[hit(4, 23, true)]).as_deref(), Some("Targets you in B4: Thrust, about 23 (guarded)"));
    assert_eq!(intent_text::forecast(&t, h, &[hit(3, 22, false), hit(5, 22, false)]).as_deref(), Some("Targets you in B3, B5: Thrust, about 22 a hit"));
    assert_eq!(intent_text::forecast(&t, h, &[hit(3, 22, false), hit(5, 30, true)]).as_deref(), Some("Targets you in B3, B5: B3 Thrust about 22; B5 Thrust about 30 (guarded)"));
    assert_eq!(intent_text::forecast(&t, w.enemies[0], &[]), None, "not the record's target, no hit");
    // HITCALC1 (PR7-45): a hit that fails at its impact is shown as such, not as a number
    use emberwake_battle::intent::HitFail;
    let fails = |block: i32, f: HitFail| HitOn { damage: 0, fails: Some(f), ..hit(block, 0, false) };
    assert_eq!(intent_text::forecast(&t, h, &[fails(4, HitFail::Dodged)]).as_deref(), Some("Targets you in B4: Thrust, dodged"));
    assert_eq!(intent_text::forecast(&t, h, &[hit(3, 22, false), fails(5, HitFail::OutOfReach)]).as_deref(), Some("Targets you in B3, B5: B3 Thrust about 22; B5 Thrust out of reach"));
    assert_eq!(intent_text::forecast(&t, h, &[fails(4, HitFail::FlankProtected)]).as_deref(), Some("Targets you in B4: Thrust, misses (flank protected)"));
    let ally = w.allies.iter().copied().find(|&a| a != h).unwrap_or(h);
    assert_eq!(intent_text::hit_amount(&fails(4, HitFail::Intercepted { by: ally, pct: 100 }), Some(w)), format!("taken by {}", w.units[ally].name));
    let part = HitOn { damage: 14, ..fails(4, HitFail::Intercepted { by: ally, pct: 40 }) };
    assert_eq!(intent_text::hit_amount(&part, None), "about 14 (40% taken by an ally)");
    t.revealed = false;
    assert_eq!(intent_text::forecast_hits(&t, h, vec![hit(3, 22, false), nth(hit(5, 22, false), 2)]), vec![hit(3, 22, false)], "an unrevealed plan names the blow its move leads to");
    // review: action 2 hits someone else, action 3 hits the hero: the hidden later action stays hidden
    assert!(intent_text::forecast_hits(&t, h, vec![nth(hit(5, 22, false), 2)]).is_empty(), "a hit of a later action stays hidden");
    assert_eq!(intent_text::forecast_hits(&t, w.enemies[0], vec![nth(hit(2, 22, false), 0), hit(3, 22, false)]), vec![nth(hit(2, 22, false), 0)], "off-target: the first action's own hit");
    assert!(intent_text::forecast_hits(&t, w.enemies[0], vec![hit(3, 22, false)]).is_empty(), "off-target: the next action's hit stays hidden");
    t.revealed = true;
    assert_eq!(intent_text::forecast_hits(&t, h, vec![hit(3, 22, false), nth(hit(5, 22, false), 2)]).len(), 2, "revealed: every hit");
    // the hover card's line on the live round: the blocks the kit's hits land
    for r in s(&b).enemy_intents() {
        let Some(target) = r.line.target.filter(|x| w.allies.contains(x)) else { continue };
        let hits = intent_text::forecast_hits(&r, target, crate::battle::tips::hits_on_hero(s(&b), r.unit, target));
        let line = intent_text::forecast(&r, target, &hits).unwrap();
        for x in &hits {
            assert!(line.contains(&format!("B{}", x.block)) && x.block >= r.line.blocks.0, "{line} {hits:?}");
        }
    }
}

/// HITCALC1 (PR7-45, FB-336): once the round is over an enemy's hover card reads this round's line ("This round:
/// <first action>") and drops the forecast of its landed hits; while planning the same tell reads "Intent:" with it.
#[test]
fn the_round_over_hover_reads_this_rounds_line() {
    use crate::battle::tips::ROUND_OVER_HEAD;
    use emberwake_battle::session::Phase;
    let (cx, _g, mut b) = arena(Some(TIMELINE), false);
    let h = b.selected.unwrap();
    let tells = s(&b).enemy_intents();
    let planned: Vec<(usize, String)> = tells.iter().map(|r| (r.unit, intent_text::first_line(&s(&b).w, &r.line))).collect();
    for (e, line) in &planned {
        let tip = unit_tip(s(&b), *e, Some(h), &tells, &s(&b).round_intents(), &[]);
        assert!(tip.iter().any(|t| t.0 == format!("Intent: {line}")), "planning: {tip:?}");
    }
    let r = b.send(&cx, json!({"cmd": "confirm"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    for _ in 0..64 {
        if s(&b).phase == Phase::Replay || s(&b).combat_over {
            break;
        }
        b.send(&cx, json!({"cmd": "step"}));
    }
    let sess = s(&b);
    assert!(sess.phase == Phase::Replay && !sess.combat_over, "the round is over: {:?}", sess.phase);
    let rounds = sess.round_intents();
    let mut seen = 0;
    for r in rounds.iter().filter(|r| sess.w.enemies.contains(&r.unit) && sess.w.units[r.unit].is_active()) {
        let tip = unit_tip(sess, r.unit, Some(h), &sess.enemy_intents(), &rounds, &[]);
        let head = format!("{ROUND_OVER_HEAD} {}", intent_text::first_line(&sess.w, &r.line));
        assert!(tip.iter().any(|t| t.0 == head), "{head}: {tip:?}");
        assert!(tip.iter().all(|t| !t.0.starts_with("Intent:") && !t.0.starts_with("Targets you") && !t.0.starts_with("Heads for you")), "{tip:?}");
        seen += 1;
    }
    assert!(seen > 0, "an enemy stands at the round's end");
}
