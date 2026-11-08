//! INTENT2-U (W2-4 slice 7b; INTENT2 section 6 "Menu", IR19 Option A): the battle's item cells under the rule
//! `field_intent` (on in the catalog's timeline row). Every intent row of the skill menu stays on one line; a pouch item
//! is an icon cell in each row it allows, after the skills, with its live copies as the corner badge; its hover card has
//! "<AP> AP, <N> blocks" and the row's variant (Field vs Assault); an area item's Field cell plans the tile command and
//! the board marks (and the hovered cell previews) its tiles; Fill and the ghost never list an item. The thrown area
//! item is built here from a catalog consumable on the hero's pouch (no shipped numbers).

use crate::battle::input::{area_preview, RowAct};
use crate::battle::overlays::overlay_of;
use crate::battle::skill_menu::Slot;
use crate::battle::{Action, BattleScreen};
use crate::screens::{Cx, Screen};
use aegis_ui::ui::Gfx;
use emberwake_battle::command;
use emberwake_battle::grid::V2;
use emberwake_battle::round_intent::{self as ri, RoundIntent, RoundTarget};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::Session;
use emberwake_rules::ruleset::{FIELD_INTENT, TIMELINE};
use serde_json::json;

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

/// A timeline arena whose selected hero carries two copies of a thrown BURST bomb (a catalog damage consumable given
/// `area` BURST and reach 3: IR21 Field, Assault), 3 tiles from a foe. Returns (cx, gfx, screen, hero, foe, item id).
fn bomb_arena(ruleset: &str) -> (Cx, Gfx, BattleScreen, usize, usize, String) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(ruleset.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    b.tick(&mut cx);
    let h = b.selected.expect("a selected hero");
    let mut item = cx.data.cat.items.values().find(|i| i.offered_in_battle() && i.aims_at_foes()).cloned().expect("a catalog damage consumable");
    item.area = "BURST".into();
    item.use_reach = 3;
    let sess = b.host.session.as_mut().unwrap();
    let w = &sess.w;
    let foe = w.enemies.iter().copied().find(|&e| w.units[e].is_active() && w.has(e)).expect("a foe");
    let fp = w.pos(foe);
    let tile = (0..emberwake_battle::grid::GRID_H)
        .flat_map(|y| (0..emberwake_battle::grid::GRID_W).map(move |x| V2::new(x, y)))
        .filter(|c| c.dist(fp) == 3 && w.grid.is_walkable(*c) && !w.grid.is_exit(*c) && w.occupant(*c).is_none_or(|u| u == h))
        .min_by_key(|c| (c.dist(w.pos(h)), c.y, c.x))
        .expect("a free tile 3 from the foe");
    sess.w.grid.positions[h] = Some(tile);
    sess.w.units[h].inventory.pouch.retain(|p| p.id != item.id);
    sess.w.units[h].inventory.pouch.push(item.clone());
    sess.w.units[h].inventory.pouch.push(item.clone());
    sess.w.add_item_skills();
    b.after_change(&cx);
    (cx, crate::hud::hud_tests::gfx(), b, h, foe, item.id)
}

/// The menu's cells of item `id`: (row index, slot, live, target).
fn item_cells(b: &BattleScreen, id: &str) -> Vec<(usize, Option<Slot>, bool, Option<RoundTarget>)> {
    let p = b.popup.as_ref().expect("the skill menu");
    p.rows.iter().enumerate().filter(|(_, r)| matches!(&r.act, RowAct::Pick(x) if matches!(&x[..], [rip::Pick::Item(i)] if i == id))).map(|(k, r)| (k, r.slot, r.valid, r.at)).collect()
}

/// Section 6 "Menu": an item allowing two rows has a cell in each (Assault, Field: the kit's derived order), live on an
/// enemy in reach (Assault on it, Field on its tile); on a tile only the Field cell is live. With the copies left as
/// the badge.
#[test]
fn a_two_row_item_has_a_cell_in_each_row() {
    let (mut cx, _g, mut b, h, foe, id) = bomb_arena(TIMELINE);
    assert!(s(&b).w.ruleset.has(FIELD_INTENT), "the catalog's timeline row plays field_intent");
    let item = command::pouch_item(&s(&b).w, h, &id).unwrap().clone();
    assert_eq!(ri::item_rows(&s(&b).w, &item), ["field", "assault"], "a thrown area item: Field and Assault (IR21)");
    let fp = s(&b).w.pos(foe);
    b.act(&mut cx, Action::UnitClick(foe));
    let cells = item_cells(&b, &id);
    let mut slots: Vec<Option<Slot>> = cells.iter().map(|c| c.1).collect();
    slots.sort_by_key(|x| format!("{x:?}"));
    assert_eq!(slots, [Some(Slot::Intent(RoundIntent::Assault)), Some(Slot::Intent(RoundIntent::Field))], "{cells:?}");
    assert!(cells.iter().all(|c| c.2), "both live on an enemy in reach: {cells:?}");
    for c in &cells {
        let want = if c.1 == Some(Slot::Intent(RoundIntent::Field)) { RoundTarget::Cell(fp) } else { RoundTarget::Unit(foe) };
        assert_eq!(c.3, Some(want), "{c:?}");
        let left = s(&b).plan.items_left(&s(&b).w, h, &id, -1);
        assert_eq!(b.popup.as_ref().unwrap().rows[c.0].count, Some(left), "the live stack count on the corner");
    }
    b.popup = None;
    let tile = [V2::new(0, 1), V2::new(0, -1), V2::new(1, 0), V2::new(-1, 0)].into_iter().map(|d| fp + d).find(|t| s(&b).w.grid.is_valid(*t) && s(&b).w.occupant(*t).is_none()).expect("a free tile by the foe");
    b.act(&mut cx, Action::TileClick(tile));
    let on_tile = item_cells(&b, &id);
    let live: Vec<Option<Slot>> = on_tile.iter().filter(|c| c.2).map(|c| c.1).collect();
    assert_eq!(live, [Some(Slot::Intent(RoundIntent::Field))], "a tile: the Field cell only (IR2): {on_tile:?}");
    assert_eq!(on_tile.len(), cells.len(), "the same cells whatever the target (nothing shifts)");
}

/// IR19 Option A: every intent row on one line, and an item's cell comes after the skills of its row.
#[test]
fn every_row_is_one_line_with_items_after_the_skills() {
    let (mut cx, mut g, mut b, _h, foe, id) = bomb_arena(TIMELINE);
    b.act(&mut cx, Action::UnitClick(foe));
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let p = b.popup.as_ref().unwrap();
    let grid = p.grid.as_ref().expect("the icon menu");
    for (_, intent, line) in grid.lines.iter().filter(|l| l.1.is_some()) {
        let in_row: Vec<usize> = (0..p.rows.len()).filter(|&k| p.rows[k].slot == intent.map(Slot::Intent)).collect();
        assert!(in_row.iter().all(|&k| grid.places[k].0 == *line), "{intent:?}: one line");
        let skills = in_row.iter().filter(|&&k| matches!(&p.rows[k].act, RowAct::Pick(x) if matches!(x[..], [rip::Pick::Skill { .. }]))).map(|&k| grid.places[k].1).max();
        let items = in_row.iter().filter(|&&k| matches!(&p.rows[k].act, RowAct::Pick(x) if matches!(&x[..], [rip::Pick::Item(i)] if *i == id))).map(|&k| grid.places[k].1).min();
        if let (Some(s), Some(i)) = (skills, items) {
            assert!(i > s, "{intent:?}: the item after the skills");
        }
    }
    let rows: Vec<i32> = grid.lines.iter().filter(|l| l.1.is_some()).map(|l| l.2).collect();
    assert_eq!(rows, (1..=rows.len() as i32).collect::<Vec<_>>(), "one line per row, none wraps");
    // no block count on the cells: the only text on an item cell is its `xN` badge
    let k = item_cells(&b, &id)[0].0;
    let left = s(&b).plan.items_left(&s(&b).w, b.selected.unwrap(), &id, -1);
    assert_eq!(b.popup.as_ref().unwrap().rows[k].count, Some(left));
}

/// The kit's option behind menu row `k` (its picks on its target).
fn option_of(b: &BattleScreen, k: usize) -> rip::PickOption {
    let p = b.popup.as_ref().unwrap();
    let (h, at) = p.menu.unwrap();
    let row = &p.rows[k];
    let RowAct::Pick(picks) = &row.act else { panic!("a pick row") };
    rip::menu_options(&s(b).w, &s(b).plan, h, at).into_iter().find(|o| &o.picks == picks && Some(o.at) == row.at && o.intent.map(Slot::Intent) == row.slot).expect("the row is the kit's")
}

fn plural(n: i32) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// Section 6 "Menu": a live item cell's card has the kit's trial cost "<AP> AP, <N> blocks; <Intent>" (here the
/// ruleset's item AP and the item's blocks, the Assault row's longer wind-up for an area item) and the row's variant
/// (Field vs Assault), as a skill's card; a refused cell has the item's own cost.
#[test]
fn an_item_card_has_its_cost_and_its_row_variant() {
    let (mut cx, _g, mut b, h, foe, id) = bomb_arena(TIMELINE);
    b.act(&mut cx, Action::UnitClick(foe));
    let w = &s(&b).w;
    let item = command::pouch_item(w, h, &id).unwrap().clone();
    for (k, slot, ..) in item_cells(&b, &id) {
        let tip: Vec<String> = b.popup.as_ref().unwrap().rows[k].tip.iter().map(|t| t.0.clone()).collect();
        let row = match slot {
            Some(Slot::Intent(i)) => i,
            _ => panic!("an intent cell"),
        };
        let o = option_of(&b, k);
        assert!(o.problem.is_none(), "live on the enemy in reach");
        let cost = format!("{} AP, {} block{}; {}", o.ap, o.blocks, plural(o.blocks), row.label(&w.cat));
        assert!(tip.contains(&cost), "{row:?}: {cost} in {tip:?}");
        assert_eq!((o.ap, o.blocks), (command::item_ap(w, h), command::item_blocks(w, &item, row == RoundIntent::Assault, w.total_blocks)), "{row:?}: no approach, the item's own cost");
        let variant = format!("{}:", row.label(&w.cat));
        assert!(tip.iter().any(|t| t.starts_with(&variant)), "{row:?}: the variant line in {tip:?}");
        if row == RoundIntent::Assault {
            assert!(tip.iter().any(|t| t.contains(&format!("x{}", w.ruleset.numbers.splash_mult))), "the Assault row's splash: {tip:?}");
        }
    }
    // a refused cell (the Assault cell on a tile): the item's own cost
    b.popup = None;
    let fp = s(&b).w.pos(foe);
    let tile = [V2::new(0, 1), V2::new(0, -1), V2::new(1, 0), V2::new(-1, 0)].into_iter().map(|d| fp + d).find(|t| s(&b).w.grid.is_valid(*t) && s(&b).w.occupant(*t).is_none()).unwrap();
    b.act(&mut cx, Action::TileClick(tile));
    let (k, ..) = item_cells(&b, &id).into_iter().find(|c| !c.2).expect("a refused cell on a tile");
    let w = &s(&b).w;
    let n = command::item_blocks(w, &item, true, w.total_blocks);
    let tip: Vec<String> = b.popup.as_ref().unwrap().rows[k].tip.iter().map(|t| t.0.clone()).collect();
    assert!(tip.contains(&format!("{} AP, {n} block{}", command::item_ap(w, h), plural(n))), "{tip:?}");
}

/// Without `field_intent`, and for an item with no area, the card has the kit's trial cost and the intent label and no
/// row variant.
#[test]
fn a_plain_item_card_has_the_kit_cost_and_no_variant() {
    for field in [true, false] {
        let (mut cx, _g, mut b, h, _foe, _id) = bomb_arena(TIMELINE);
        let potion = cx.data.cat.items.values().find(|i| i.offered_in_battle() && i.heal > 0 && !i.is_area()).cloned().expect("a catalog heal consumable");
        let sess = b.host.session.as_mut().unwrap();
        if !field {
            sess.w.ruleset.rules.retain(|r| *r != FIELD_INTENT);
        }
        sess.w.units[h].inventory.pouch.push(potion.clone());
        sess.w.add_item_skills();
        b.after_change(&cx);
        b.act(&mut cx, Action::SelfTarget);
        let cells = item_cells(&b, &potion.id);
        let (k, slot, ..) = cells.iter().copied().find(|c| c.2).unwrap_or_else(|| panic!("field {field}: a live cell on yourself: {cells:?}"));
        let o = option_of(&b, k);
        let w = &s(&b).w;
        let label = match slot {
            Some(Slot::Intent(i)) => i.label(&w.cat),
            _ => panic!("an intent cell"),
        };
        let tip: Vec<String> = b.popup.as_ref().unwrap().rows[k].tip.iter().map(|t| t.0.clone()).collect();
        assert!(tip.contains(&format!("{} AP, {} block{}; {label}", o.ap, o.blocks, plural(o.blocks))), "field {field}: {tip:?}");
        assert!(!tip.iter().any(|t| t.starts_with("Field:") || t.starts_with("Assault:")), "field {field}: no variant: {tip:?}");
    }
}

/// The Field cell plans the tile command (`PlanStep::use_item_at`): the board marks its tiles in the Field row, and the
/// hovered cell previews them before the pick.
#[test]
fn a_field_item_marks_its_tiles_and_previews_them() {
    let (mut cx, _g, mut b, h, foe, id) = bomb_arena(TIMELINE);
    let fp = s(&b).w.pos(foe);
    b.act(&mut cx, Action::UnitClick(foe));
    let (k, _, _, at) = item_cells(&b, &id).into_iter().find(|c| c.1 == Some(Slot::Intent(RoundIntent::Field))).expect("the Field cell");
    let preview = area_preview(s(&b), h, at.unwrap(), &[rip::Pick::Item(id.clone())]);
    assert!(preview.iter().any(|(cells, row)| *row == RoundIntent::Field && cells.contains(&fp)), "the hovered cell previews: {preview:?}");
    b.act(&mut cx, Action::Choose(k));
    let w = &s(&b).w;
    assert_eq!(ri::intent_of(w, h), Some(RoundIntent::Field), "{}", b.message);
    let e = s(&b).plan.entries_for(w, h).into_iter().find(|e| e.item.as_ref().is_some_and(|i| i.id == id)).expect("the item entry");
    assert_eq!((e.target, e.aim), (None, fp), "the tile command on the source tile");
    let o = overlay_of(s(&b), Some(h), true).expect("timeline overlays");
    assert!(o.areas.iter().any(|(cells, row)| *row == RoundIntent::Field && *cells == e.cells), "the board marks the item's tiles: {:?}", o.areas);
    assert!(area_preview(s(&b), h, RoundTarget::Unit(foe), &[rip::Pick::Rest]).is_empty(), "no area: no preview");
}

/// Section 6: Fill and the ghost never list an item: the auto-filled round of a hero with a pouch holds no item, and
/// Fill takes none.
#[test]
fn fill_and_the_ghost_never_list_an_item() {
    let (mut cx, _g, mut b, h, _foe, _id) = bomb_arena(TIMELINE);
    let w = &s(&b).w;
    let entries = s(&b).plan.entries_for(w, h);
    let ghosts = rip::ghost_entries(w, &s(&b).plan, h);
    assert!(ghosts.contains(&true), "the hero has a ghost round to take");
    assert!(entries.iter().zip(&ghosts).filter(|(_, g)| **g).all(|(e, _)| e.item.is_none()), "no item in the ghost");
    b.accept_ghost(&cx, None);
    assert!(!rip::picks_of(&s(&b).w, h).iter().any(|p| matches!(p, rip::Pick::Item(_))), "Fill takes no item: {}", b.message);
    b.tick(&mut cx);
}

/// Without `field_intent` (a round_intent row without the rule) an item keeps its one cell, in the row its target gives.
#[test]
fn without_field_intent_an_item_keeps_one_cell() {
    let (mut cx, _g, mut b, _h, foe, id) = bomb_arena(TIMELINE);
    b.host.session.as_mut().unwrap().w.ruleset.rules.retain(|r| *r != FIELD_INTENT);
    b.act(&mut cx, Action::UnitClick(foe));
    let cells = item_cells(&b, &id);
    assert_eq!(cells.len(), 1, "{cells:?}");
    let w = &s(&b).w;
    let rows = ri::item_rows(w, command::pouch_item(w, b.selected.unwrap(), &id).unwrap());
    assert_eq!(rows.len(), 1, "one row without the rule: {rows:?}");
    assert_eq!(cells[0].1, RoundIntent::parse(rows[0]).map(Slot::Intent), "in its one row");
}
