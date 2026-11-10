//! HOOKS1-UI slice 1 (HOOKS1 GH6, AH3): the ground hazards and the arena telegraph on the board and the strip. A catalog
//! encounter played by the catalog's timeline row with both hook rules switched on; the hazards and the arena schedule
//! are test rows put on the battle's world, and every expectation is a kit query (`ground::seen_at`,
//! `intent::arena_tells`), never a shipped number. Classic marks nothing (no state key).

use crate::battle::marks::HookMarks;
use crate::battle::overlays::Layers;
use crate::battle::BattleScreen;
use crate::battle::tips::{class_names, unit_tip};
use crate::screens::{Cx, Screen};
use aegis_ui::ui::Gfx;
use emberwake_battle::grid::{V2, GRID_H, GRID_W};
use emberwake_battle::session::Session;
use aegis_ui::art::Art;
use emberwake_battle::world::World;
use serde_json::{json, Value};
use emberwake_battle::arena::{self, Hazard};
use emberwake_battle::ground::{self, Placed};
use emberwake_data::HazardDef;
use emberwake_rules::ruleset::TIMELINE;

const RULES: &str = "+ground_hazards,+arena_hazards";

fn arena_screen(ruleset: Option<&str>, rules: &str) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!("flat");
    // Decision "Battle view answers": the two groups are for testing, off by default; these tests are that testing
    cx.settings.v["battle_overlays"] = marks_on();
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), rules, false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn marks_on() -> Value {
    json!({"test_hazards": true, "test_arena": true})
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

/// Free walkable cells, left to right, top to bottom.
fn free_cells(w: &World) -> Vec<V2> {
    (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).filter(|c| w.grid.is_valid(*c) && w.grid.is_walkable(*c) && w.occupant(*c).is_none() && !w.grid.is_exit(*c)).collect()
}

fn hazard(id: &str, hidden: bool) -> HazardDef {
    let row = json!({"id": id, "name": format!("Test {id}"), "on": "enter", "damage": 5, "status": "root", "potency": 60, "affects": "foes", "trap": hidden, "hidden": hidden, "duration": 2, "tint": "#40c0ff", "glyph": "T"});
    HazardDef::from_obj(row.as_object().unwrap())
}

/// Puts three hazards down (a foe's visible one, a foe's hidden trap, a hero's hidden trap) and an arena hazard on
/// one column at blocks 2 and 4 of this round; returns the three cells and the column.
pub(super) fn mark_up(b: &mut BattleScreen) -> ([V2; 3], i32) {
    let sess = b.host.session.as_mut().unwrap();
    let round = sess.current_round;
    let w = &mut sess.w;
    let (hero, foe) = (w.allies[0], w.enemies[0]);
    let free = free_cells(w);
    let cells = [free[0], free[1], free[2]];
    for (c, (id, by, hidden)) in cells.iter().zip([("smoke", foe, false), ("wire", foe, true), ("snare", hero, true)]) {
        w.grid.ground.insert(*c, Placed { def: hazard(id, hidden), by, round, last_round: round + 1 });
    }
    let col = free[free.len() / 2].x;
    let row = json!({"id": "vents", "name": "Test Vents", "columns": [col], "blocks": [2, 4], "effect": {"damage": 9, "stagger": 10}});
    w.arena.hazards = vec![Hazard::from_obj(row.as_object().unwrap())];
    arena::round_start(w, round);
    (cells, col)
}

fn tip_text(b: &BattleScreen, id: &str) -> String {
    b.w.iter().filter(|x| x.id == id).flat_map(|x| x.tip.iter().map(|(t, _)| t.clone())).collect::<Vec<_>>().join(" | ")
}

/// GH6, AH3: the party sees the foe's visible hazard and its own hidden trap (fainter), not the foe's hidden one;
/// the struck cells are the kit's arena tells; the tile and strip hover cards name them; the state lists them.
#[test]
fn the_marks_follow_the_kit() {
    let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), RULES);
    let (cells, col) = mark_up(&mut b);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let w = &s(&b).w;
    assert!(ground::on(w) && arena::on(w), "both hook rules on");
    let viewer = w.allies[0];
    let off = HookMarks::of(s(&b), &Layers::of(&Value::Null), false);
    assert_eq!(off.json(), json!({"ground": [], "arena": []}), "not drawn by default");
    let marks = HookMarks::of(s(&b), &Layers::of(&marks_on()), false);
    let seen: Vec<V2> = marks.ground.iter().map(|g| g.cell).collect();
    let want: Vec<V2> = cells.iter().copied().filter(|c| ground::seen_at(w, viewer, *c).is_some()).collect();
    assert_eq!(seen, want);
    assert_eq!(seen, vec![cells[0], cells[2]], "the foe's hidden trap stays hidden");
    assert_eq!(marks.ground.iter().map(|g| g.hidden).collect::<Vec<_>>(), [false, true]);
    let tells = emberwake_battle::intent::arena_tells(w);
    assert_eq!(marks.arena, tells);
    assert!(!tells[0].cells.is_empty() && tells[0].cells.iter().all(|c| c.x == col));
    // the hover cards
    let tile = |c: V2| format!("tile.{}.{}", c.x, c.y);
    assert!(tip_text(&b, &tile(cells[0])).contains("Test smoke"), "{}", tip_text(&b, &tile(cells[0])));
    assert!(!tip_text(&b, &tile(cells[1])).contains("Test wire"), "the hidden foe trap has no card");
    assert!(tip_text(&b, &tile(cells[2])).contains("Hidden from the foes"));
    let struck = tells[0].cells.iter().copied().find(|c| w.occupant(*c).is_none()).expect("a free struck cell");
    assert!(tip_text(&b, &tile(struck)).contains("Test Vents: strikes here this round"), "{}", tip_text(&b, &tile(struck)));
    for blk in 1..=s(&b).def.total_blocks {
        let t = tip_text(&b, &format!("strip.block{blk}"));
        assert_eq!(t.contains(&format!("Test Vents strikes at the end of block {blk}")), tells[0].blocks.contains(&blk), "block {blk}: {t}");
    }
    // the agent link
    let st = b.state(&cx);
    assert_eq!(st["hook_marks"]["ground"].as_array().unwrap().len(), 2);
    assert_eq!(st["hook_marks"]["arena"], emberwake_battle::intent::arena_json(&tells));
}

/// The marks draw on the ground in both views: a frame with them differs from the same frame with both layers off. With
/// the layers off the board and the strip badge lose them, the hover lines and the strip lines stay (user 2026-11-03), and
/// Alt draws them as the layers do.
#[test]
fn the_layers_switch_the_marks() {
    for view in ["flat", "stage"] {
        let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), RULES);
        cx.settings.v["battle_view"] = json!(view);
        let (cells, _) = mark_up(&mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let on = g.canvas.px.clone();
        cx.settings.v["battle_overlays"] = json!({"test_hazards": false, "test_arena": false});
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_ne!(on, g.canvas.px, "{view}: the marks draw");
        assert!(tip_text(&b, &format!("tile.{}.{}", cells[0].x, cells[0].y)).contains("Test smoke"), "the tile still tells");
        assert!((1..=s(&b).def.total_blocks).any(|blk| tip_text(&b, &format!("strip.block{blk}")).contains("Test Vents")), "the block still tells");
        let st = b.state(&cx);
        assert_eq!((&st["hook_marks"]["ground"], &st["hook_marks"]["arena"]), (&json!([]), &json!([])), "not drawn");
        cx.held.alt = true;
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(!b.state(&cx)["hook_marks"]["ground"].as_array().unwrap().is_empty(), "{view}: Alt draws them");
        let alt = g.canvas.px.clone();
        cx.held.alt = false;
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_ne!(alt, g.canvas.px, "{view}: Alt shows on the board");
    }
}

/// Classic (and a timeline battle without the two rules): nothing is marked whatever the flags say, and the state
/// has no `hook_marks` key.
#[test]
fn without_the_rules_nothing_is_marked() {
    for (ruleset, rules) in [(None, ""), (Some(TIMELINE), "-ground_hazards,-arena_hazards")] {
        let (mut cx, mut g, mut b) = arena_screen(ruleset, rules);
        mark_up(&mut b);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let marks = HookMarks::of(s(&b), &Layers::of(&marks_on()), true);
        assert!(marks.ground.is_empty() && marks.arena.is_empty(), "{ruleset:?} {rules}");
        assert!(b.state(&cx).get("hook_marks").is_none(), "{ruleset:?} {rules}: the state keeps its shape");
    }
}


/// HOOKS1 CN2 (HOOKS1-UI slice 2): the hover card names each unit's class as `Data::class_name` gives it in the
/// battle's world (the Arena, a catalog screen: the class row's name); a unit without a class names none.
#[test]
fn the_hover_card_names_the_class_in_the_battles_world() {
    let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), "");
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(b.class_world, "", "the Arena names classes by their rows");
    let w = &s(&b).w;
    let names = class_names(&cx.data, &b.class_world, w);
    let mut seen = 0;
    for (i, u) in w.units.iter().enumerate() {
        let class = cx.data.cat.units.get(&u.catalog_id).map(|d| d.class_id.clone()).filter(|c| cx.data.cat.classes.contains_key(c));
        match class {
            Some(c) => {
                assert_eq!(names[i], cx.data.class_name("", &c));
                assert!(unit_tip(s(&b), i, None, &[], &[], &names).iter().any(|t| t.0.starts_with(&format!("Level {} {}", u.stats.level, names[i]))));
                seen += 1;
            }
            None => assert_eq!(names[i], ""),
        }
    }
    assert!(seen > 0, "a unit with a class");
}

/// PR5-26 (audit B8; PK1-S1 3.6): a hazard's unit meter change shows in its hover lines in words ("+15 <meter name>",
/// `marks::hazard_meter_text`; the meter's id when the catalog has no row): on the ground hazard's card and on the arena
/// hazard's tile and strip lines. Test rows only (no shipped Strain carrier).
#[test]
fn a_hazards_meter_change_shows_in_its_lines() {
    let (mut cx, mut g, mut b) = arena_screen(Some(TIMELINE), RULES);
    let (cells, col) = mark_up(&mut b);
    {
        let sess = b.host.session.as_mut().unwrap();
        let round = sess.current_round;
        let w = &mut sess.w;
        let row = json!({"id": "smoke", "name": "Test smoke", "on": "enter", "damage": 5, "affects": "foes", "duration": 2, "tint": "#40c0ff", "glyph": "T", "meter": {"meter": "t_strain", "delta": 15}});
        let p = w.grid.ground.get_mut(&cells[0]).expect("the visible hazard");
        p.def = HazardDef::from_obj(row.as_object().unwrap());
        let arena_row = json!({"id": "vents", "name": "Test Vents", "columns": [col], "blocks": [2, 4], "effect": {"damage": 9, "meter": {"meter": "t_strain", "delta": -5}}});
        w.arena.hazards = vec![Hazard::from_obj(arena_row.as_object().unwrap())];
        arena::round_start(w, round);
    }
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let tile = |c: V2| format!("tile.{}.{}", c.x, c.y);
    let ground_card = tip_text(&b, &tile(cells[0]));
    assert!(ground_card.contains("On entering: 5 damage, +15 t_strain"), "{ground_card}");
    let tells = emberwake_battle::intent::arena_tells(&s(&b).w);
    let blk = tells[0].blocks[0];
    let strip = tip_text(&b, &format!("strip.block{blk}"));
    assert!(strip.contains("9 damage, -5 t_strain"), "{strip}");
}
