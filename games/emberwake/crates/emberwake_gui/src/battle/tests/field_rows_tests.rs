//! A4.2-K1b UI (W2-4 slice 6b; INTENT2 IR5, IR15, IR17, IR19): the battle's Field row under the rule `field_intent`
//! (on in the catalog's timeline row): the skill menu draws a Field row; a dual Field / Assault skill has a cell in each
//! row, and the target picks the live one (an enemy: Assault, locked on it; its tile or any tile: Field); the arrows move
//! a Field round's source one tile; the planned area's impact tiles show on the board in the row's colour (the Field
//! marks fixed, the Assault shape on its target), a hovered area cell previews them; Field has its own colour. Without
//! the rule nothing changes. The dual skill is built here on the session's catalog (no shipped numbers).

use crate::battle::input::{area_preview, RowAct};
use crate::battle::overlays::overlay_of;
use crate::battle::skill_menu::{menu_intents, Slot};
use crate::battle::tips::intent_ring_color;
use crate::battle::{Action, BattleScreen};
use crate::screens::{Cx, Key, Screen};
use aegis_ui::ui::Gfx;
use emberwake_battle::grid::V2;
use emberwake_battle::round_intent::{self as ri, RoundIntent, RoundTarget};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::Session;
use emberwake_battle::skill::{self, Kind};
use emberwake_rules::ruleset::{FIELD_INTENT, TIMELINE};
use serde_json::json;
use std::sync::Arc;

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

/// A timeline arena whose selected hero's first plain attack is a thrown BURST (reach 2-3: IR21 Field, Assault), the
/// hero 3 tiles from a foe (in the BURST's band). Returns (cx, gfx, screen, hero, foe, skill).
fn field_arena() -> (Cx, Gfx, BattleScreen, usize, usize, usize) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    b.tick(&mut cx);
    let h = b.selected.expect("a selected hero");
    let sess = b.host.session.as_mut().unwrap();
    assert!(sess.w.ruleset.has(FIELD_INTENT), "the catalog's timeline row plays field_intent");
    let w = &sess.w;
    let sid = rip::hotkey_skills(w, h).into_iter().find(|&k| {
        let sk = &w.cat.skills[k];
        skill::kind(sk) == Kind::Attack && !skill::is_area(sk) && sk.position.is_none() && sk.dash_tiles == 0 && sk.hazard.is_none()
    }).expect("a plain attack on the hero's keys");
    let foe = w.enemies.iter().copied().find(|&e| w.units[e].is_active() && w.has(e)).expect("a foe");
    let fp = w.pos(foe);
    let tile = (0..emberwake_battle::grid::GRID_H)
        .flat_map(|y| (0..emberwake_battle::grid::GRID_W).map(move |x| V2::new(x, y)))
        .filter(|c| c.dist(fp) == 3 && w.grid.is_walkable(*c) && !w.grid.is_exit(*c) && w.occupant(*c).is_none_or(|u| u == h))
        .min_by_key(|c| (c.dist(w.pos(h)), c.y, c.x))
        .expect("a free tile 3 from the foe");
    {
        let sk = &mut Arc::make_mut(&mut sess.w.cat).skills[sid];
        sk.area = "BURST".into();
        sk.reach_min = 2;
        sk.reach_max = 3;
        sk.power_mult = sk.power_mult.max(0.5);
    }
    sess.w.grid.positions[h] = Some(tile);
    let rows = ri::skill_rows(&sess.w, &sess.w.cat.skills[sid]);
    assert_eq!(rows, ["field", "assault"], "a dual skill");
    (cx, crate::hud::hud_tests::gfx(), b, h, foe, sid)
}

fn skill_cells(b: &BattleScreen, sid: usize) -> Vec<(usize, Option<Slot>, bool, Option<RoundTarget>)> {
    let p = b.popup.as_ref().expect("the skill menu");
    p.rows.iter().enumerate().filter(|(_, r)| matches!(&r.act, RowAct::Pick(x) if matches!(x[..], [rip::Pick::Skill { skill, .. }] if skill == sid))).map(|(k, r)| (k, r.slot, r.valid, r.at)).collect()
}

#[test]
fn the_menu_draws_a_field_row_only_under_field_intent() {
    let (_cx, _g, mut b, ..) = field_arena();
    assert!(menu_intents(&s(&b).w).contains(&RoundIntent::Field), "field_intent: a Field row");
    assert_eq!(menu_intents(&s(&b).w).last(), Some(&RoundIntent::Field), "in RoundIntent::ALL's order: last");
    b.host.session.as_mut().unwrap().w.ruleset.rules.retain(|r| *r != FIELD_INTENT);
    assert!(!menu_intents(&s(&b).w).contains(&RoundIntent::Field), "round_intent alone: the rows the ruleset lists");
}

/// IR19: on an enemy the dual skill's Assault cell aims at it and its Field cell at its tile; on a tile only the Field
/// cell is live. The same two cells whatever the target (nothing shifts).
#[test]
fn a_dual_skill_has_a_cell_in_each_row_and_the_target_picks_the_live_one() {
    let (mut cx, _g, mut b, _h, foe, sid) = field_arena();
    let fp = s(&b).w.pos(foe);
    b.act(&mut cx, Action::UnitClick(foe));
    let cells = skill_cells(&b, sid);
    assert_eq!(cells.iter().map(|c| c.1).collect::<Vec<_>>(), [Some(Slot::Intent(RoundIntent::Assault)), Some(Slot::Intent(RoundIntent::Field))]);
    assert!(cells.iter().all(|c| c.2), "both live on an enemy in reach: {cells:?}");
    assert_eq!((cells[0].3, cells[1].3), (Some(RoundTarget::Unit(foe)), Some(RoundTarget::Cell(fp))), "Assault aims at the enemy, Field at its tile");
    b.popup = None;
    let tile = fp + V2::new(0, 1);
    let tile = if s(&b).w.grid.is_valid(tile) && s(&b).w.occupant(tile).is_none() { tile } else { fp + V2::new(0, -1) };
    b.act(&mut cx, Action::TileClick(tile));
    let on_tile = skill_cells(&b, sid);
    assert_eq!(on_tile.iter().map(|c| c.1).collect::<Vec<_>>(), cells.iter().map(|c| c.1).collect::<Vec<_>>(), "the same cells on a tile");
    assert_eq!(on_tile.iter().map(|c| c.2).collect::<Vec<_>>(), [false, true], "a tile: Field only (IR2)");
}

/// IR5, IR17: the Field cell on an enemy plans a Field round on its tile (fixed marks); the Assault cell a round locked
/// on the enemy. The board's overlay shows each area's impact tiles in its row.
#[test]
fn the_cells_plan_their_rows_and_the_board_marks_them() {
    for want in [RoundIntent::Field, RoundIntent::Assault] {
        let (mut cx, _g, mut b, h, foe, sid) = field_arena();
        let fp = s(&b).w.pos(foe);
        b.act(&mut cx, Action::UnitClick(foe));
        let k = skill_cells(&b, sid).into_iter().find(|c| c.1 == Some(Slot::Intent(want))).expect("the cell").0;
        b.act(&mut cx, Action::Choose(k));
        let w = &s(&b).w;
        assert_eq!(ri::intent_of(w, h), Some(want), "{}", b.message);
        let e = s(&b).plan.entries_for(w, h).into_iter().find(|e| e.skill == sid).expect("the area entry");
        match want {
            RoundIntent::Field => assert_eq!((e.target, e.aim, w.round_states[&h].target), (None, fp, Some(RoundTarget::Cell(fp))), "Field: the source tile"),
            _ => assert_eq!(e.target, Some(foe), "Assault: locked on the enemy"),
        }
        let o = overlay_of(s(&b), Some(h), true).expect("timeline overlays");
        assert!(o.areas.iter().any(|(cells, row)| *row == want && cells.contains(&fp)), "{want:?}: {:?}", o.areas);
    }
}

/// The hovered cell previews its impact tiles before the pick: Field's at the source, Assault's on the enemy.
#[test]
fn a_hovered_area_cell_previews_its_tiles() {
    let (mut cx, _g, mut b, h, foe, sid) = field_arena();
    let fp = s(&b).w.pos(foe);
    b.act(&mut cx, Action::UnitClick(foe));
    for (_, slot, _, at) in skill_cells(&b, sid) {
        let at = at.unwrap();
        let preview = area_preview(s(&b), h, at, &[rip::Pick::Skill { skill: sid, kind: None }]);
        let row = match slot {
            Some(Slot::Intent(i)) => i,
            _ => panic!("an intent cell"),
        };
        assert!(preview.iter().any(|(cells, r)| *r == row && cells.contains(&fp)), "{row:?}: {preview:?}");
    }
    assert!(area_preview(s(&b), h, RoundTarget::Unit(foe), &[rip::Pick::Rest]).is_empty(), "no area: no preview");
}

/// Row W2-4 "Field ground-aimed": an arrow moves a Field round's source one tile, with its picks (re-made there) and
/// with none (the round target moves); an arrow outside a Field round does nothing.
#[test]
fn the_arrows_move_a_field_source() {
    let (mut cx, _g, mut b, h, foe, sid) = field_arena();
    let fp = s(&b).w.pos(foe);
    b.act(&mut cx, Action::UnitClick(foe));
    let k = skill_cells(&b, sid).into_iter().find(|c| c.1 == Some(Slot::Intent(RoundIntent::Field))).unwrap().0;
    b.act(&mut cx, Action::Choose(k));
    b.popup = None;
    assert!(!b.arrow_focus(), "aiming a Field round: the arrows steer the source, not the keyboard focus");
    let hero = s(&b).w.pos(h);
    // the step that keeps the source in the Field band (2 to 3 + 1 from the hero)
    let band = skill::field_reach(&s(&b).w.cat.skills[sid], &s(&b).w.units[h], &s(&b).w.ruleset);
    let (key, d) = [(Key::Left, V2::new(-1, 0)), (Key::Right, V2::new(1, 0)), (Key::Up, V2::new(0, -1)), (Key::Down, V2::new(0, 1))]
        .into_iter()
        .find(|(_, d)| s(&b).w.grid.is_valid(fp + *d) && (band.0..=band.1).contains(&(fp + *d).dist(hero)))
        .expect("a step in the band");
    b.key(&mut cx, key);
    let w = &s(&b).w;
    assert_eq!(w.round_states[&h].target, Some(RoundTarget::Cell(fp + d)), "{}", b.message);
    assert!(rip::picks_of(w, h).iter().any(|p| matches!(p, rip::Pick::Skill { skill, .. } if *skill == sid)), "the pick is made again");
    let e = s(&b).plan.entries_for(w, h).into_iter().find(|e| e.skill == sid).expect("the area entry");
    assert_eq!(e.aim, fp + d, "the marks moved with the source");
    // an Assault round: the arrows leave it alone
    let (mut cx, _g, mut b, h, foe, sid) = field_arena();
    b.act(&mut cx, Action::UnitClick(foe));
    let k = skill_cells(&b, sid).into_iter().find(|c| c.1 == Some(Slot::Intent(RoundIntent::Assault))).unwrap().0;
    b.act(&mut cx, Action::Choose(k));
    assert!(b.arrow_focus(), "an Assault round: the arrows move the focus as before");
    let before = s(&b).w.round_states[&h].target;
    b.key(&mut cx, Key::Left);
    assert_eq!(s(&b).w.round_states[&h].target, before);
}

/// INTENT2-U "Field ring colour": Field has a colour of its own, from the theme.
#[test]
fn field_has_its_own_colour() {
    let field = intent_ring_color(RoundIntent::Field);
    for i in RoundIntent::ALL.into_iter().filter(|i| *i != RoundIntent::Field) {
        assert_ne!(intent_ring_color(i), field, "{i:?}");
    }
    let base = crate::battle::tips::field_color();
    assert!(field[0] >= base[0] && field[1] >= base[1] && field[2] >= base[2], "the ring brightens the theme's Field colour");
}

/// The arrows never change the intent: with no pick, a tile where no Field round plans (here: out of the Field band
/// while the hero is rooted, so no approach) is refused and the round stays as it was.
#[test]
fn an_arrow_to_a_tile_no_field_round_plans_on_is_refused() {
    let (mut cx, _g, mut b, h, _foe, sid) = field_arena();
    let w = &s(&b).w;
    let hero = w.pos(h);
    let band = skill::field_reach(&w.cat.skills[sid], &w.units[h], &w.ruleset);
    let dirs = [(Key::Left, V2::new(-1, 0)), (Key::Right, V2::new(1, 0)), (Key::Up, V2::new(0, -1)), (Key::Down, V2::new(0, 1))];
    let free = |c: V2| w.grid.is_valid(c) && w.grid.is_walkable(c) && !w.grid.is_exit(c) && w.occupant(c).is_none();
    let (tile, key) = (0..emberwake_battle::grid::GRID_H)
        .flat_map(|y| (0..emberwake_battle::grid::GRID_W).map(move |x| V2::new(x, y)))
        .filter(|c| c.dist(hero) == band.1 && free(*c))
        .find_map(|c| dirs.iter().find(|(_, d)| w.grid.is_valid(c + *d) && (c + *d).dist(hero) > band.1).map(|(k, _)| (c, *k)))
        .expect("a tile at the band's edge with a step beyond it");
    b.act(&mut cx, Action::TileClick(tile));
    b.popup = None;
    assert_eq!((ri::intent_of(&s(&b).w, h), s(&b).w.round_states[&h].target), (Some(RoundIntent::Field), Some(RoundTarget::Cell(tile))), "{}", b.message);
    assert!(rip::picks_of(&s(&b).w, h).is_empty(), "no pick: the ghost's round");
    let root = emberwake_rules::status::EffectSpec::grant("root", 2, 0, "test");
    emberwake_rules::status::apply_effect(&mut b.host.session.as_mut().unwrap().w.units[h], &root, emberwake_rules::status::Timing::Now, 0.0, None);
    b.key(&mut cx, key);
    let w = &s(&b).w;
    assert_eq!((ri::intent_of(w, h), w.round_states[&h].target), (Some(RoundIntent::Field), Some(RoundTarget::Cell(tile))), "refused: {}", b.message);
    assert!(b.message.contains("no Field round"), "{}", b.message);
}
