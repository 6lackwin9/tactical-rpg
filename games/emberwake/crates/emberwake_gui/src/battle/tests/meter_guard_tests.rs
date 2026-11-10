//! HOOKS1-UI slice 3 (HOOKS1 UM7, 8.3, HC1): unit meters on the cards, the inspect card and the strip; the held-guard
//! edge; the HP cost in the strip cards and the step popup, and its refusal in words. A catalog encounter played by the
//! catalog's timeline row with the three hook rules switched on; the meters, the held guard and the HP cost are test
//! rows put on the battle's world. Every expectation is a kit query (`meter::shown`, `trigger::held_at`,
//! `command::hp_cost_of`, `command::hp_cost_problem`), never a shipped number. Classic shows none of it.

use crate::battle::marks::{held_guard, meter_text, meters_of};
use crate::battle::overlays::Layers;
use crate::battle::BattleScreen;
use aegis_ui::canvas::Rect;
use crate::intent_text;
use crate::screens::{Cx, Screen};
use aegis_ui::ui::Gfx;
use emberwake_battle::session::Session;
use aegis_ui::art::Art;
use serde_json::{json, Value};
use emberwake_battle::meter;
use emberwake_data::UnitMeterDef;
use emberwake_rules::ruleset::TIMELINE;
use emberwake_rules::unit::{HeldGuard, HeldMeter};

const RULES: &str = "+unit_meters,+triggers,+hp_cost";
const COST: i32 = 5;

fn arena_screen(ruleset: Option<&str>, rules: &str) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), rules, false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn tip_text(b: &BattleScreen, id: &str) -> String {
    b.w.iter().filter(|x| x.id == id).flat_map(|x| x.tip.iter().map(|(t, _)| t.clone())).collect::<Vec<_>>().join(" | ")
}

fn rect(b: &BattleScreen, id: &str) -> Option<Rect> {
    b.w.iter().find(|x| x.id == id).map(|x| x.rect)
}

/// The hero holds a pips meter (0-4 from 3, in band Full), the first foe a bar meter (0-20 from 5), the foe a held
/// guard 0.5 through block 3, and the hero's attack skills (every rank) cost COST HP. Returns (hero, foe).
pub(super) fn hook_up(b: &mut BattleScreen) -> (usize, usize) {
    let h = b.selected.expect("a hero");
    let sess = b.host.session.as_mut().unwrap();
    let w = &mut sess.w;
    let f = w.enemies.iter().copied().find(|&e| w.units[e].is_active() && w.has(e)).expect("a foe");
    let known = w.units[h].known_skills.clone();
    let cat = std::sync::Arc::make_mut(&mut w.cat);
    let phial = json!({"id": "t_phial", "name": "Test Phial", "short": "Phial", "max": 4, "show": "pips", "bands": [{"id": "low", "name": "Low", "from": 0}, {"id": "full", "name": "Full", "from": 3}]});
    let heat = json!({"id": "t_heat", "name": "Test Heat", "short": "Heat", "max": 20, "show": "bar"});
    for row in [phial, heat] {
        let def = UnitMeterDef::from_obj(row.as_object().unwrap());
        cat.unit_meters.insert(def.id.clone(), def);
    }
    // every rank of a known attack: a rank 2+ row is `<head>_<rank>` with the head as its `family` (the loader's rank rule)
    for sk in cat.skills.iter_mut().filter(|k| (known.contains(&k.id) || known.contains(&k.family)) && k.kind == "ATTACK") {
        sk.hp_cost = COST;
    }
    w.units[h].passive_meters.push(HeldMeter { meter: "t_phial".into(), start: 3, floor: None });
    w.units[f].passive_meters.push(HeldMeter { meter: "t_heat".into(), start: 5, floor: None });
    meter::battle_start(w);
    w.units[f].held_guard = Some(HeldGuard { mitigation: 0.5, last_block: 3 });
    (h, f)
}

/// UM7: the cards grow a meter row, the inspect card names each meter with its band, the strip row shows the
/// selected hero's; the state lists them as `meter::shown` gives them. 8.3: the foe's held guard shows on its card
/// and inspect card and in the state.
#[test]
fn meters_and_the_held_guard_follow_the_kit() {
    let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), RULES);
    let (h, f) = hook_up(&mut b);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let w = &s(&b).w;
    let (mh, mf) = (meter::shown(w, h), meter::shown(w, f));
    assert_eq!(meters_of(w, h, &Layers::of(&Value::Null)), mh);
    assert_eq!((mh[0].show.as_str(), mf[0].show.as_str()), ("pips", "bar"));
    assert!(tip_text(&b, &format!("party.{h}")).contains(&meter_text(&mh[0])), "{}", tip_text(&b, &format!("party.{h}")));
    assert!(meter_text(&mh[0]).ends_with(&format!("({})", mh[0].band)) && !mh[0].band.is_empty());
    assert!(tip_text(&b, &format!("foe.{f}")).contains(&meter_text(&mf[0])));
    assert_eq!(rect(&b, &format!("party.{h}")).unwrap().h, 34, "the card grows a meter row");
    assert!(tip_text(&b, "strip.meters").contains(&meter_text(&mh[0])));
    let held = emberwake_battle::trigger::held_at(w, f, s(&b).current_block).expect("held");
    assert!(tip_text(&b, &format!("foe.{f}")).contains(&format!("Held guard {:.0}% through B3", held * 100.0)));
    let st = b.state(&cx);
    let units: Vec<i64> = st["meters"].as_array().unwrap().iter().map(|m| m["unit"].as_i64().unwrap()).collect();
    assert!(units.contains(&(h as i64)) && units.contains(&(f as i64)));
    assert_eq!(st["held_guards"], json!([{"unit": f, "mitigation": 0.5, "last_block": 3}]));
}

/// The `meters` and `guard` layers: off, the rows, the strip row and the edge go (a frame differs), the cards keep
/// their classic height; the inspect lines stay (as the poise line does).
#[test]
fn the_layers_switch_meters_and_the_guard_edge() {
    let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), RULES);
    let (h, f) = hook_up(&mut b);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let on = g.canvas.px.clone();
    cx.settings.v["battle_overlays"] = json!({"meters": false, "guard": false});
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_ne!(on, g.canvas.px);
    assert!(rect(&b, "strip.meters").is_none());
    assert_eq!(rect(&b, &format!("party.{h}")).unwrap().h, 30);
    assert!(tip_text(&b, &format!("foe.{f}")).contains("Held guard"), "the inspect line stays");
    let st = b.state(&cx);
    assert_eq!((st["meters"].clone(), st["held_guards"].clone()), (json!([]), json!([])));
}

/// HC1: a planned step that costs HP says so on its strip card and in the step popup (its detail and its card); at
/// or below the cost the popup row is refused and reads the refusal in words.
#[test]
fn the_hp_cost_shows_on_the_strip_and_the_popup() {
    let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), RULES);
    let (h, f) = hook_up(&mut b);
    let r = b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": f}));
    assert_eq!(r["ok"], json!(true), "{r}");
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let w = &s(&b).w;
    let costly: Vec<String> = s(&b).plan.compile(w, h).entries.iter().map(|e| &w.cat.skills[e.skill]).filter(|k| emberwake_battle::command::hp_cost_of(w, k) > 0).map(|k| k.name.clone()).collect();
    assert!(!costly.is_empty(), "the plan holds a costly attack");
    let text = intent_text::hp_cost_text(COST);
    assert!((1..=s(&b).def.total_blocks).any(|blk| tip_text(&b, &format!("strip.block{blk}")).contains(&text)));
    let rows = b.options_for(h, "foe", Some(f), None);
    let row = rows.iter().find(|r| r.tip.iter().any(|t| t.0 == text)).expect("a costly step in the popup");
    assert!(!row.valid || row.detail.contains(&format!("{COST} HP")), "{}", row.detail);
    b.host.session.as_mut().unwrap().w.units[h].hp = COST;
    let rows = b.options_for(h, "foe", Some(f), None);
    let refused = rows.iter().find(|r| r.tip.iter().any(|t| t.0 == text)).expect("the costly step");
    assert!(!refused.valid);
    assert_eq!(refused.detail, intent_text::refusal_text(&emberwake_battle::command::hp_cost_problem(COST, COST)));
    assert_eq!(intent_text::refusal_text(&emberwake_battle::command::hp_cost_problem(9, 10)), "Not enough HP: 9, it costs 10");
    assert_eq!(intent_text::refusal_text("Out of reach"), "Out of reach");
}

/// Classic: no meter, no held guard, no HP cost shows and the state has no `meters` or `held_guards` key.
#[test]
fn classic_shows_none_of_it() {
    let (mut cx, mut g, mut b) = arena_screen(None, "");
    let (h, _) = hook_up(&mut b);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(meters_of(&s(&b).w, h, &Layers::of(&Value::Null)).is_empty());
    assert!(rect(&b, "strip.meters").is_none());
    let st = b.state(&cx);
    assert!(st.get("meters").is_none() && st.get("held_guards").is_none());
    assert!(!b.w.iter().any(|x| x.tip.iter().any(|t| t.0.starts_with("Held guard") || t.0 == intent_text::hp_cost_text(COST))));
}


/// The slice 3 review: a timeline battle without unit_meters, triggers and hp_cost shows none of the marks either
/// (the kit queries answer nothing without their rules), and its state has no `meters` or `held_guards` key.
#[test]
fn a_timeline_battle_without_the_rules_shows_none_of_it() {
    let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), "-unit_meters,-triggers,-hp_cost");
    let (h, f) = hook_up(&mut b);
    let r = b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": f}));
    assert_eq!(r["ok"], json!(true), "{r}");
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let w = &s(&b).w;
    assert!(!meter::on(w) && !emberwake_battle::trigger::on(w));
    assert!((0..w.units.len()).all(|i| meters_of(w, i, &Layers::of(&Value::Null)).is_empty() && held_guard(s(&b), i).is_none()));
    assert!(rect(&b, "strip.meters").is_none());
    assert_eq!(rect(&b, &format!("party.{h}")).unwrap().h, 30);
    let st = b.state(&cx);
    assert!(st.get("meters").is_none() && st.get("held_guards").is_none());
    assert!(!b.w.iter().any(|x| x.tip.iter().any(|t| t.0.starts_with("Held guard") || t.0 == intent_text::hp_cost_text(COST) || t.0.starts_with("Test Phial"))));
}
