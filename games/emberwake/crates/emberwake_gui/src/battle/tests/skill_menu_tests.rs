//! VISCLAR1-D slice 2 (decision "Battle view answers, part 2"; mock `menu_icons`): the skill menu as icons: a neutral row
//! with Fill, Move and Rest, then one row per intent the ruleset lists; the other rows fade in place once an opener fixes the
//! intent; the menu's Move, its leftover Rest and note, and undo. Fixtures are the catalog's encounters under the timeline
//! ruleset (or built here); expectations come from these rules, never shipped numbers.

use crate::battle::input::{Popup, Row, RowAct};
use crate::battle::skill_menu::{menu_intents, MenuGrid, Slot, ICON};
use crate::battle::{Action, BattleScreen};
use crate::screens::{Cx, Key, Screen};
use aegis_ui::canvas::Rect;
use aegis_ui::ui::{Gfx, H, W};
use emberwake_battle::grid::V2;
use emberwake_battle::round_intent::{self as ri, RoundIntent, RoundTarget};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::Session;
use emberwake_rules::ruleset::TIMELINE;
use serde_json::json;
use std::sync::Arc;

fn arena_on(cx: &mut Cx, ruleset: Option<&str>) -> BattleScreen {
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(cx, None, &encounter, ruleset.map(String::from), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    b.tick(cx);
    b
}

fn arena() -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    let b = arena_on(&mut cx, Some(TIMELINE));
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn foe_of(b: &BattleScreen) -> usize {
    s(b).w.enemies.iter().copied().find(|&e| s(b).w.units[e].is_active() && s(b).w.has(e)).expect("a foe on the field")
}

/// A tile no unit stands on, as close to `from` as the board has.
fn free_tile_near(b: &BattleScreen, from: V2) -> V2 {
    let w = &s(b).w;
    let mut tiles: Vec<V2> = (0..emberwake_battle::grid::GRID_H).flat_map(|y| (0..emberwake_battle::grid::GRID_W).map(move |x| V2::new(x, y))).filter(|c| *c != from && w.grid.is_walkable(*c) && !w.grid.is_exit(*c) && w.occupant(*c).is_none()).collect();
    tiles.sort_by_key(|c| c.dist(from));
    tiles[0]
}

fn drawn(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen) {
    b.draw(cx, g, (-500, -500), true);
}

fn popup(b: &BattleScreen) -> &Popup {
    b.popup.as_ref().expect("the skill menu")
}

/// The first cell usable on the menu's own target.
fn own_cell(b: &BattleScreen) -> usize {
    let p = popup(b);
    let at = p.menu.unwrap().1;
    p.rows.iter().position(|r| r.valid && r.at == Some(at) && matches!(&r.act, RowAct::Pick(x) if x.iter().any(|q| q.holds_target()))).expect("a cell usable on the target")
}

fn slot_row(b: &BattleScreen, text: &str) -> usize {
    popup(b).rows.iter().position(|r| matches!(r.slot, Some(Slot::Neutral(t)) if t == text)).unwrap_or_else(|| panic!("the {text} button"))
}

fn rects(b: &BattleScreen) -> Vec<(String, Rect)> {
    b.w.iter().filter(|w| w.id.strip_prefix("popup.").is_some_and(|k| k.parse::<usize>().is_ok())).map(|w| (w.id.clone(), w.rect)).collect()
}

fn picks(b: &BattleScreen, h: usize) -> Vec<rip::Pick> {
    rip::picks_of(&s(b).w, h)
}

fn plan_of(b: &BattleScreen, h: usize) -> Vec<(usize, i32, Option<usize>, V2)> {
    crate::battle::input::plan_rows(&s(b).w, &s(b).plan, h)
}

/// The menu holds the neutral row and one row per intent the ruleset lists, in `RoundIntent::ALL`'s order, with no text on a
/// skill cell; its Fill, Move and Rest sit left to right on top, each skill and item in its own intent's row.
#[test]
fn the_rows_are_the_rulesets_intents_under_the_neutral_row() {
    let (mut cx, mut g, mut b) = arena();
    let foe = foe_of(&b);
    b.act(&mut cx, Action::UnitClick(foe));
    drawn(&mut cx, &mut g, &mut b);
    let listed = menu_intents(&s(&b).w);
    // A4.2-K1b: the intents the battle offers (the listed ones, and Field under field_intent)
    assert_eq!(listed, RoundIntent::ALL.into_iter().filter(|i| ri::listed(&s(&b).w, *i)).collect::<Vec<_>>());
    let p = popup(&b);
    let grid = p.grid.as_ref().unwrap();
    assert_eq!(grid.lines.iter().map(|l| l.1).collect::<Vec<_>>(), std::iter::once(None).chain(listed.iter().map(|i| Some(*i))).collect::<Vec<_>>());
    let x = |text: &str| p.row(b.menu_row, slot_row(&b, text));
    assert!(x("Fill").x < x("Move").x && x("Move").x < x("Rest").x && x("Fill").y == x("Rest").y, "Fill, Move, Rest on one line");
    for (k, row) in p.rows.iter().enumerate() {
        if let Some(Slot::Intent(i)) = row.slot {
            let RowAct::Pick(picks) = &row.act else { panic!("a cell is a pick") };
            let first = picks.iter().find(|q| q.holds_target()).unwrap();
            // A4.2-K1b: the kit's rows (IR21 under field_intent); a cell sits in the row its pick takes on the target
            let want = match first {
                rip::Pick::Skill { skill, .. } => {
                    let rows = ri::skill_rows(&s(&b).w, &s(&b).w.cat.skills[*skill]);
                    rows.iter().find(|r| **r == i.id()).or(rows.first()).unwrap().to_string()
                }
                rip::Pick::Item(id) => ri::item_intent(emberwake_battle::command::pouch_item(&s(&b).w, p.menu.unwrap().0, id).unwrap()).to_string(),
                _ => unreachable!(),
            };
            assert_eq!(i.id(), want, "row {k} is in its own intent's row");
            let r = p.row(b.menu_row, k);
            assert_eq!((r.w, r.h), (ICON, ICON), "a cell is its icon");
            assert!(r.y > x("Fill").y, "under the neutral row");
        }
    }
}

/// An opener fixes the intent: every widget of the menu stands where it stood, the panel with them; the other intents' cells
/// and names fade, and the same list of rows stays.
#[test]
fn nothing_shifts_when_an_intent_fixes() {
    let (mut cx, mut g, mut b) = arena();
    let (h, foe) = (b.selected.unwrap(), foe_of(&b));
    b.act(&mut cx, Action::UnitClick(foe));
    drawn(&mut cx, &mut g, &mut b);
    let before = rects(&b);
    let panel = popup(&b).rect(b.menu_row);
    let labels: Vec<String> = popup(&b).rows.iter().map(|r| r.label.clone()).collect();
    assert!(popup(&b).grid.as_ref().unwrap().fixed.is_none(), "the opener is free");
    assert!(before.iter().all(|(_, r)| r.x >= panel.x && r.x + r.w <= panel.x + panel.w && r.y >= panel.y && r.y + r.h <= panel.y + panel.h), "the cells stay in the panel");
    let k = own_cell(&b);
    b.act(&mut cx, Action::Choose(k));
    assert_eq!(picks(&b, h).len(), 1, "{}", b.message);
    drawn(&mut cx, &mut g, &mut b);
    assert_eq!(rects(&b), before, "no widget moved");
    assert_eq!(popup(&b).rect(b.menu_row), panel, "the panel did not move");
    assert_eq!(popup(&b).rows.iter().map(|r| r.label.clone()).collect::<Vec<_>>(), labels, "the same rows in the same order");
    let fixed = ri::intent_of(&s(&b).w, h).expect("the opener set the intent");
    let grid = popup(&b).grid.clone().unwrap();
    assert_eq!(grid.fixed, Some(fixed));
    for row in &popup(&b).rows {
        if let Some(Slot::Intent(i)) = row.slot {
            assert_eq!(grid.faded(Some(i)), i != fixed, "only the other rows fade");
            if i != fixed {
                assert!(!row.valid, "{} is another intent's: faded", row.label);
            }
        }
    }
    assert!(!grid.faded(None), "the neutral row never fades with an intent");
}

/// Rows of the fifth intent are one list entry: Channel listed, the menu has one more row of its own for a skill of that
/// intent, nothing overlaps at any text size, and the panel stays on the screen from the screen's far corner.
#[test]
fn a_fifth_row_holds() {
    let (mut cx, mut g, mut b) = arena();
    let (h, foe) = (b.selected.unwrap(), foe_of(&b));
    let sess = b.host.session.as_mut().unwrap();
    // A4.2-K1b: an authored round_intent is read only without field_intent (IR21 derives the rows, never Channel)
    sess.w.ruleset.rules.retain(|r| *r != emberwake_rules::ruleset::FIELD_INTENT);
    let was = menu_intents(&sess.w).len();
    let added = RoundIntent::ALL.into_iter().find(|i| !sess.w.ruleset.numbers.intents.iter().any(|x| x == i.id())).expect("an intent the ruleset does not list");
    sess.w.ruleset.numbers.intents.push(added.id().to_string());
    assert_eq!(menu_intents(&sess.w).len(), was + 1);
    // one of the hero's skills belongs to the new intent
    let sid = rip::hotkey_skills(&sess.w, h)[0];
    Arc::make_mut(&mut sess.w.cat).skills[sid].round_intent = added.id().to_string();
    b.act(&mut cx, Action::UnitClick(foe));
    for scale in [1.0f32, 1.15, 1.3] {
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
        let p = popup(&b);
        let grid = p.grid.as_ref().unwrap();
        assert_eq!(grid.lines.len(), was + 2, "the neutral row and one per listed intent");
        assert!(grid.lines.iter().any(|l| l.1 == Some(added)), "the new row is listed");
        let cell = p.rows.iter().position(|r| r.slot == Some(Slot::Intent(added))).expect("the skill sits in the new row");
        let r = p.row(b.menu_row, cell);
        assert!(r.w > 0);
        let panel = p.rect(b.menu_row);
        assert!(panel.x >= 2 && panel.y >= 2 && panel.x + panel.w <= W - 2 && panel.y + panel.h <= H - 2, "scale {scale}: on screen: {panel:?}");
        let cells: Vec<Rect> = (0..p.rows.len()).map(|k| p.row(b.menu_row, k)).collect();
        for (i, a) in cells.iter().enumerate() {
            assert!(a.x + a.w <= panel.x + panel.w && a.y + a.h <= panel.y + panel.h, "scale {scale}: cell {i} in the panel");
            for c in &cells[i + 1..] {
                assert!(a.x + a.w <= c.x || c.x + c.w <= a.x || a.y + a.h <= c.y || c.y + c.h <= a.y, "scale {scale}: {a:?} and {c:?} overlap");
            }
        }
        let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
        let audit = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
        let spill: Vec<_> = audit["spill"].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("popup."))).cloned().collect();
        assert!(spill.is_empty(), "scale {scale}: {spill:?}");
    }
}

fn row_in(slot: Slot) -> Row {
    Row { label: "x".into(), detail: String::new(), valid: true, act: RowAct::Pick(vec![rip::Pick::Item("x".into())]), tip: Vec::new(), warn: false, ap_short: None, slot: Some(slot), icon: Vec::new(), at: None, count: None }
}

/// A row's place comes from its slot alone: a pouch item or a skill may sit in any intent's row, a row stays on one line
/// however long (INTENT2-U, IR19 Option A), and an empty row keeps its line.
#[test]
fn any_row_takes_a_cell_and_a_long_row_stays_on_one_line() {
    let intents: Vec<(RoundIntent, String)> = RoundIntent::ALL.into_iter().map(|i| (i, i.name().to_string())).collect();
    for (at, (i, _)) in intents.iter().enumerate() {
        let g = MenuGrid::new(&intents, intents.iter().map(|x| x.0).collect(), &[row_in(Slot::Intent(*i))], None);
        assert_eq!(g.places[0], (at as i32 + 1, 0), "the item in {}'s row", i.id());
        assert_eq!(g.depth, intents.len() as i32 + 1, "an empty row keeps its line");
    }
    let n = 12;
    let rows = vec![row_in(Slot::Intent(RoundIntent::Guard)); n];
    let g = MenuGrid::new(&intents, intents.iter().map(|x| x.0).collect(), &rows, None);
    let guard = intents.iter().position(|x| x.0 == RoundIntent::Guard).unwrap() as i32 + 1;
    assert_eq!(g.places, (0..n as i32).map(|c| (guard, c)).collect::<Vec<_>>(), "one line, a column per cell");
    assert_eq!((g.depth, g.cols), (intents.len() as i32 + 1, n as i32), "no extra line; the panel widens");
    assert!(g.lines.iter().skip(1).enumerate().all(|(k, l)| l.2 == k as i32 + 1), "every row on its own line");
}

/// The row list is one input: a subset keeps every place, and a row left out draws nothing and takes no hover or click.
#[test]
fn a_subset_of_rows_keeps_every_place() {
    let (mut cx, mut g, mut b) = arena();
    let foe = foe_of(&b);
    b.act(&mut cx, Action::UnitClick(foe));
    drawn(&mut cx, &mut g, &mut b);
    let full = rects(&b);
    let all = popup(&b).grid.as_ref().unwrap().shown.clone();
    let (hidden, _) = popup(&b).rows.iter().enumerate().find_map(|(k, r)| match r.slot { Some(Slot::Intent(i)) => Some((i, k)), _ => None }).expect("a cell");
    let panel = popup(&b).rect(b.menu_row);
    b.popup.as_mut().unwrap().grid.as_mut().unwrap().shown = all.into_iter().filter(|i| *i != hidden).collect();
    drawn(&mut cx, &mut g, &mut b);
    let some = rects(&b);
    let p = popup(&b).clone();
    assert_eq!(p.rect(b.menu_row), panel, "the panel keeps its size");
    let gone: Vec<usize> = (0..p.rows.len()).filter(|&k| p.rows[k].slot == Some(Slot::Intent(hidden))).collect();
    assert!(!gone.is_empty());
    for (id, r) in &full {
        let k: usize = id["popup.".len()..].parse().unwrap();
        if gone.contains(&k) {
            assert!(!some.iter().any(|x| &x.0 == id), "{id} is not drawn");
            let mid = (r.x + r.w / 2, r.y + r.h / 2);
            assert_eq!(p.row_at(b.menu_row, mid), None, "{id} takes no hover");
        } else {
            assert_eq!(some.iter().find(|x| &x.0 == id).map(|x| x.1), Some(*r), "{id} stays in its slot");
            assert_eq!(p.row_at(b.menu_row, (r.x + r.w / 2, r.y + r.h / 2)), Some(k));
        }
    }
}

/// Cost and AP are not on the icon: the hover card has them, as the kit counts them; a row refused shows why instead.
#[test]
fn the_hover_card_has_the_cost() {
    let (mut cx, _g, mut b) = arena();
    let foe = foe_of(&b);
    b.act(&mut cx, Action::UnitClick(foe));
    let at = popup(&b).menu.unwrap().1;
    let h = popup(&b).menu.unwrap().0;
    let opts = rip::menu_options(&s(&b).w, &s(&b).plan, h, at);
    for row in popup(&b).rows.iter().filter(|r| matches!(r.slot, Some(Slot::Intent(_)))) {
        let RowAct::Pick(picks) = &row.act else { panic!() };
        let o = opts.iter().find(|o| &o.picks == picks && o.at == row.at.unwrap()).expect("the row is the kit's");
        let tip: Vec<&str> = row.tip.iter().map(|t| t.0.as_str()).collect();
        match &o.problem {
            Some(why) => assert!(tip.contains(&why.as_str()), "{tip:?}"),
            None => assert!(tip.iter().any(|t| t.contains(&format!("{} AP", o.ap)) && t.contains("block")), "{tip:?}"),
        }
    }
}

/// Every action of the old menu still goes: a skill by click and by Shift+digit, Rest, Fill and an item; the move on a tile.
#[test]
fn every_old_action_still_works() {
    // Shift+digit with the menu open picks the row with that digit
    let (mut cx, _g, mut b) = arena();
    let (h, foe) = (b.selected.unwrap(), foe_of(&b));
    b.act(&mut cx, Action::UnitClick(foe));
    let d = (1..=9u8).find(|d| popup(&b).rows.iter().any(|r| r.valid && r.label.starts_with(&format!("{d} ")))).expect("a usable hotkey row");
    b.key(&mut cx, Key::ShiftDigit(d));
    assert_eq!(picks(&b, h).len(), 1, "{}", b.message);

    // Rest
    let (mut cx, _g, mut b) = arena();
    let (h, foe) = (b.selected.unwrap(), foe_of(&b));
    b.act(&mut cx, Action::UnitClick(foe));
    let k = slot_row(&b, "Rest");
    b.act(&mut cx, Action::Choose(k));
    assert_eq!(picks(&b, h), [rip::Pick::Rest], "{}", b.message);

    // a pouch item, in its intent's row
    let (mut cx, mut g, mut b) = arena();
    let h = b.selected.unwrap();
    let item = cx.data.cat.items.values().find(|i| i.offered_in_battle() && !i.aims_at_foes()).cloned().expect("a catalog item for the battle");
    let world = &mut b.host.session.as_mut().unwrap().w;
    world.units[h].inventory.pouch.push(item.clone());
    world.add_item_skills();
    b.act(&mut cx, Action::SelfTarget);
    drawn(&mut cx, &mut g, &mut b);
    // INTENT2-U: an item with two rows has a cell in each; on yourself the live one
    let k = popup(&b).rows.iter().position(|r| r.valid && matches!(&r.act, RowAct::Pick(p) if matches!(&p[..], [rip::Pick::Item(id)] if *id == item.id))).expect("the item's cell");
    // A1.2-K3 (IR15, FB-121): an item on yourself takes Stand when its derived rows hold it (field_intent), else its row
    let rows = ri::item_rows(&b.host.session.as_ref().unwrap().w, &item);
    let want = if rows.contains(&RoundIntent::Guard.id()) { RoundIntent::Guard.id() } else { rows[0] };
    assert_eq!(popup(&b).rows[k].slot, RoundIntent::parse(want).map(Slot::Intent), "the item's row on yourself");
    assert!(popup(&b).rows[k].valid, "{}", popup(&b).rows[k].detail);
    // two items can share an icon: the cell has its copies left and the card names it
    let left = s(&b).plan.items_left(&s(&b).w, h, &item.id, -1);
    assert_eq!(popup(&b).rows[k].count, Some(left));
    assert_eq!(popup(&b).rows[k].tip[0].0, format!("Use {} ({left} left)", item.name));
    // a cell with no art of its own has a generic icon: none draws an empty frame
    let icons: Vec<Vec<String>> = popup(&b).rows.iter().filter(|r| matches!(r.slot, Some(Slot::Intent(_)))).map(|r| r.icon.clone()).collect();
    let kit_art = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../kits/rpg/assets");
    assert!(icons.iter().all(|chain| chain.last().is_some_and(|g| g.starts_with("skills/kinds/") && kit_art.join(g).is_file())), "{icons:?}");
    b.act(&mut cx, Action::Choose(k));
    assert!(picks(&b, h).iter().any(|p| matches!(p, rip::Pick::Item(id) if *id == item.id)), "{}", b.message);

    // Move here on a tile, first in the round
    let (mut cx, _g, mut b) = arena();
    let h = b.selected.unwrap();
    let tile = free_tile_near(&b, s(&b).w.pos(h));
    b.act(&mut cx, Action::TileClick(tile));
    let k = slot_row(&b, "Move");
    assert!(popup(&b).rows[k].valid, "{}", popup(&b).rows[k].detail);
    b.act(&mut cx, Action::Choose(k));
    assert_eq!(picks(&b, h), [rip::Pick::MoveHere(tile)], "{}", b.message);
}

/// Picks on a foe of `h` that keep it on its tile: `h` stands next to the foe and takes an opener that needs no move.
fn opener_in_place(cx: &mut Cx, b: &mut BattleScreen, h: usize, foe: usize) {
    let p = s(b).w.pos(foe);
    let tile = free_tile_near(b, p);
    b.host.session.as_mut().unwrap().w.grid.positions[h] = Some(tile);
    b.after_change(cx);
    b.act(cx, Action::UnitClick(foe));
    let open = popup(b).clone();
    let k = (0..open.rows.len()).find(|&k| {
        let row = &open.rows[k];
        let RowAct::Pick(x) = &row.act else { return false };
        row.valid && row.at == Some(RoundTarget::Unit(foe)) && x.iter().any(|q| q.holds_target()) && rip::preview_plan(&s(b).w, &s(b).plan, h, RoundTarget::Unit(foe), x).is_ok_and(|(w2, p2)| rip::move_refusal(&w2, &p2, h).is_none())
    }).expect("an opener that does not move");
    b.act(cx, Action::Choose(k));
    assert!(rip::move_refusal(&s(b).w, &s(b).plan, h).is_none(), "the opener left it unmoved");
}

/// Move is valid while the unit has not moved: with a free opener and after an opener that stays; it fades when the opener
/// moves the unit, the hover card says why, and a click does nothing.
#[test]
fn move_fades_when_the_opener_moves_the_unit() {
    let (mut cx, _g, mut b) = arena();
    let (h, foe) = (b.selected.unwrap(), foe_of(&b));
    b.act(&mut cx, Action::UnitClick(foe));
    assert!(popup(&b).rows[slot_row(&b, "Move")].valid, "no opener: the unit has not moved");
    let k = slot_row(&b, "Move");
    assert!(matches!(popup(&b).rows[k].act, RowAct::MoveAim), "a unit's menu takes the tile next");
    b.run(&mut cx, Action::ClosePopup);

    // an opener that has to walk: far from every foe, the first cell usable on it is its approach
    let far = {
        let w = &s(&b).w;
        let p = w.pos(foe);
        let mut tiles: Vec<V2> = (0..emberwake_battle::grid::GRID_H).flat_map(|y| (0..emberwake_battle::grid::GRID_W).map(move |x| V2::new(x, y))).filter(|c| w.grid.is_walkable(*c) && !w.grid.is_exit(*c) && w.occupant(*c).is_none() && c.dist(p) == 3).collect();
        tiles.sort_by_key(|c| (c.x, c.y));
        tiles[0]
    };
    b.host.session.as_mut().unwrap().w.grid.positions[h] = Some(far);
    b.after_change(&cx);
    b.act(&mut cx, Action::UnitClick(foe));
    let open = popup(&b).clone();
    let k = (0..open.rows.len()).find(|&k| {
        let row = &open.rows[k];
        let RowAct::Pick(x) = &row.act else { return false };
        row.valid && row.at == Some(RoundTarget::Unit(foe)) && x.iter().any(|q| q.holds_target()) && rip::preview_plan(&s(&b).w, &s(&b).plan, h, RoundTarget::Unit(foe), x).is_ok_and(|(w2, p2)| rip::move_refusal(&w2, &p2, h).is_some())
    }).expect("an opener that walks");
    b.act(&mut cx, Action::Choose(k));
    let why = rip::move_refusal(&s(&b).w, &s(&b).plan, h).expect("the opener moved");
    assert!(why.starts_with("Already moved"), "{why}");
    let k = slot_row(&b, "Move");
    let row = popup(&b).rows[k].clone();
    assert!(!row.valid, "Move fades");
    assert!(row.tip.iter().any(|t| t.0 == why), "the card says why: {:?}", row.tip);
    b.act(&mut cx, Action::Choose(k));
    assert!(b.move_aim.is_none() && b.popup.is_some(), "a faded Move does nothing");
}

/// From a unit's menu the Move takes the next tile clicked; Esc, or a click on anything else, ends the wait.
#[test]
fn move_from_a_units_menu_takes_the_next_tile() {
    let (mut cx, _g, mut b) = arena();
    let (h, foe) = (b.selected.unwrap(), foe_of(&b));
    opener_in_place(&mut cx, &mut b, h, foe);
    let n = picks(&b, h).len();
    let k = slot_row(&b, "Move");
    b.act(&mut cx, Action::Choose(k));
    assert!(b.popup.is_none() && b.move_aim.is_some(), "it waits for a tile: {}", b.message);
    b.escape(&mut cx);
    assert!(b.move_aim.is_none(), "Esc cancels: {}", b.message);
    assert_eq!(picks(&b, h).len(), n);

    b.act(&mut cx, Action::UnitClick(foe));
    let k = slot_row(&b, "Move");
    b.act(&mut cx, Action::Choose(k));
    assert!(b.move_aim.is_some());
    let tile = free_tile_near(&b, s(&b).w.pos(h));
    b.act(&mut cx, Action::TileClick(tile));
    assert!(b.move_aim.is_none());
    assert_eq!(picks(&b, h).last(), Some(&rip::Pick::StepAfter(tile)), "{}", b.message);
    let entries = s(&b).plan.entries_for(&s(&b).w, h);
    let ghost = rip::ghost_entries(&s(&b).w, &s(&b).plan, h);
    let last = entries.iter().zip(&ghost).rev().find(|(_, g)| !**g).map(|(e, _)| e).unwrap();
    assert_eq!((s(&b).w.cat.skills[last.skill].kind.as_str(), last.pos_after), ("MOVE", tile), "the Move is the last pick, to that tile");
}

/// The Move replaces the last filled block; a larger one leaves Rest, with the note on the strip naming it (the kit's words);
/// Undo gives the block back and the note goes. The fill comes from what the hero's order fills here, so the test looks for a
/// hero and foe whose last filled block is larger than one.
#[test]
fn a_move_over_a_larger_block_leaves_rest_a_note_and_an_undo() {
    let (mut cx, mut g, mut b) = arena();
    let foe = foe_of(&b);
    let allies = s(&b).w.allies.clone();
    let mut done = false;
    for h in allies {
        b.selected = Some(h);
        if b.host.session.as_ref().unwrap().w.units[h].is_active() {
            opener_in_place(&mut cx, &mut b, h, foe);
            b.run(&mut cx, Action::ClosePopup);
            let before = plan_of(&b, h);
            let sess = s(&b);
            let (entries, ghosts) = (sess.plan.entries_for(&sess.w, h), rip::ghost_entries(&sess.w, &sess.plan, h));
            let Some(last) = entries.iter().zip(&ghosts).rev().find(|(_, g)| **g).map(|(e, _)| e) else { continue };
            let span = last.end_block(&sess.w) - last.start_block + 1;
            if span < 2 || sess.w.cat.skills[last.skill].kind == "REST" {
                continue;
            }
            let name = sess.w.cat.skills[last.skill].name.clone();
            let tile = free_tile_near(&b, sess.w.pos(h));
            b.act(&mut cx, Action::TileClick(tile));
            let k = slot_row(&b, "Move");
            assert!(popup(&b).rows[k].valid, "{}", popup(&b).rows[k].detail);
            b.act(&mut cx, Action::Choose(k));
            b.run(&mut cx, Action::ClosePopup);
            drawn(&mut cx, &mut g, &mut b);
            let want = format!("Move replaces {name}: {} block{} left as Rest", span - 1, if span == 2 { "" } else { "s" });
            assert_eq!(rip::move_note(&s(&b).plan, h).as_deref(), Some(want.as_str()));
            let note = b.w.iter().find(|w| w.id == "strip.move_note").expect("the note on the strip");
            assert_eq!(note.text, want);
            let bar_blocks: Vec<&str> = s(&b).plan.entries_for(&s(&b).w, h).iter().filter(|e| e.start_block >= last.start_block).map(|e| s(&b).w.cat.skills[e.skill].kind.as_str()).collect::<Vec<_>>();
            assert_eq!(bar_blocks.first(), Some(&"MOVE"), "the Move in the block's first slot: {bar_blocks:?}");
            assert!(bar_blocks[1..].iter().all(|k| *k == "REST") && bar_blocks.len() as i32 == span, "Rest after it: {bar_blocks:?}");
            b.act(&mut cx, Action::Undo);
            drawn(&mut cx, &mut g, &mut b);
            assert_eq!(plan_of(&b, h), before, "Undo restores the block");
            assert!(!b.w.iter().any(|w| w.id == "strip.move_note"), "the note goes");
            done = true;
            break;
        }
    }
    assert!(done, "no hero's last filled block spans two blocks in this fight");
}

/// The new menu's text at the Normal and larger text sizes: no overlap and no cut among its own lines.
#[test]
fn the_menu_text_holds_at_every_size() {
    let (mut cx, mut g, mut b) = arena();
    g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
    g.view = aegis_ui::painter::View::fit(1280, 720);
    let foe = foe_of(&b);
    b.act(&mut cx, Action::UnitClick(foe));
    for scale in [1.0f32, 1.15, 1.3] {
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
        let panel = popup(&b).rect(b.menu_row);
        let (px, py, pw, ph) = (panel.x as f32, panel.y as f32, panel.w as f32, panel.h as f32);
        let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
        let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
        let inside = |x: f32, y: f32| x >= px && x <= px + pw && y >= py && y <= py + ph;
        for (kind, a) in [("overlap", "at_a"), ("cut", "at")] {
            let ours: Vec<_> = v[kind].as_array().map(|l| l.iter().filter(|e| inside(e[a][0].as_f64().unwrap_or(-1.0) as f32, e[a][1].as_f64().unwrap_or(-1.0) as f32)).cloned().collect()).unwrap_or_default();
            assert!(ours.is_empty(), "scale {scale}: {kind}: {ours:?}");
        }
    }
}

/// A classic battle keeps its text menus: no grid, no icons, rows as before.
#[test]
fn classic_keeps_its_text_menu() {
    let mut cx = crate::hud::hud_tests::game_cx();
    let mut b = arena_on(&mut cx, None);
    assert!(!ri::on(&s(&b).w), "the plain arena is classic");
    let h = s(&b).w.allies[0];
    b.selected = Some(h);
    let foe = foe_of(&b);
    b.act(&mut cx, Action::UnitClick(foe));
    let p = b.popup.as_ref().expect("the attack menu");
    assert!(p.grid.is_none() && p.menu.is_none(), "a text menu");
    assert!(p.rows.iter().all(|r| r.slot.is_none() && r.icon.is_empty()));
}

/// Decision "Intent framework answers": on a foe the Guard and Support cells are grey and say what they need; a click there
/// does nothing and keeps the menu; the neutral row stays live; the unit's own menu lights them. HK3: a self-only key still
/// targets self.
#[test]
fn a_cell_lights_only_for_its_target() {
    let (mut cx, mut g, mut b) = arena();
    let (h, foe) = (b.selected.unwrap(), foe_of(&b));
    b.act(&mut cx, Action::UnitClick(foe));
    drawn(&mut cx, &mut g, &mut b);
    let p = popup(&b).clone();
    let opts = rip::pick_options(&s(&b).w, &s(&b).plan, h, RoundTarget::Unit(foe));
    let mut grey = 0;
    for (k, row) in p.rows.iter().enumerate() {
        if let (Some(Slot::Intent(i)), RowAct::Pick(picks)) = (row.slot, &row.act) {
            assert_eq!(row.valid, opts.iter().any(|o| &o.picks == picks && o.problem.is_none()), "{}: lit only with `pick_options`", row.label);
            if matches!(i, RoundIntent::Guard | RoundIntent::Support) {
                assert!(!row.valid, "{} is grey on a foe", row.label);
                assert!(row.tip.iter().any(|t| t.0.starts_with("Target yourself")), "the card says why: {:?}", row.tip);
                let r = p.row(b.menu_row, k);
                assert!(matches!(aegis_ui::ui::hit(&b.w, (r.x + r.w / 2, r.y + r.h / 2)), Some(Action::MenuGap)), "the click lands on the panel");
                b.act(&mut cx, Action::Choose(k));
                assert!(b.popup.is_some(), "the menu stays");
                grey += 1;
            }
        }
    }
    assert!(grey > 0, "the fixture's hero has a Guard or Support skill");
    assert!(picks(&b, h).is_empty(), "nothing was picked");
    for text in ["Fill", "Move", "Rest"] {
        assert!(popup(&b).rows[slot_row(&b, text)].valid, "{text} stays live");
    }
    b.act(&mut cx, Action::ClosePopup);
    b.act(&mut cx, Action::SelfTarget);
    let lit = popup(&b).rows.iter().filter(|r| r.valid && matches!(r.slot, Some(Slot::Intent(RoundIntent::Guard)))).count();
    assert!(lit > 0, "the unit's own menu lights the Guard row");

    // the self-only key from a foe's menu acts on self
    let (mut cx, _g, mut b) = arena();
    let (h, foe) = (b.selected.unwrap(), foe_of(&b));
    b.act(&mut cx, Action::UnitClick(foe));
    let keys = rip::hotkey_skills(&s(&b).w, h);
    let d = keys.iter().position(|k| rip::self_only(&s(&b).w.cat.skills[*k])).filter(|d| *d < 9).expect("a self-only hotkey");
    b.key(&mut cx, Key::ShiftDigit(d as u8 + 1));
    assert_eq!(picks(&b, h).len(), 1, "{}", b.message);
    assert_eq!(s(&b).w.round_states[&h].target, Some(RoundTarget::Myself), "HK3: on self");
}

/// A near miss in a gap or the name column keeps the menu; a click outside closes it.
#[test]
fn a_click_inside_the_panel_keeps_the_menu() {
    let (mut cx, mut g, mut b) = arena();
    let foe = foe_of(&b);
    b.act(&mut cx, Action::UnitClick(foe));
    drawn(&mut cx, &mut g, &mut b);
    let panel = popup(&b).rect(b.menu_row);
    let name_column = (panel.x + 3, panel.y + panel.h - 8);
    let action = aegis_ui::ui::hit(&b.w, name_column).expect("the panel takes the click");
    assert!(matches!(action, Action::MenuGap));
    b.act(&mut cx, action);
    assert!(b.popup.is_some());
    let outside = (panel.x + panel.w + 2, panel.y + panel.h + 2);
    assert!(matches!(aegis_ui::ui::hit(&b.w, outside), Some(Action::ClosePopup)));
}

/// While the Move waits for a tile the board marks what it reaches and the strip says what the tile under the mouse takes;
/// Confirm ends the wait, so the next round's first Esc is not spent on it.
#[test]
fn move_waiting_for_a_tile_marks_its_reach_and_ends_with_confirm() {
    let (mut cx, mut g, mut b) = arena();
    let (h, foe) = (b.selected.unwrap(), foe_of(&b));
    opener_in_place(&mut cx, &mut b, h, foe);
    let k = slot_row(&b, "Move");
    b.act(&mut cx, Action::Choose(k));
    drawn(&mut cx, &mut g, &mut b);
    let tiles = rip::move_tiles(&s(&b).w, &s(&b).plan, h);
    assert!(!tiles.is_empty());
    assert_eq!(b.positions_shown, tiles.iter().map(|t| (t.cell, true)).collect::<Vec<_>>(), "the board marks them");
    assert!(tiles.iter().all(|t| t.blocks >= 1 && t.blocks <= s(&b).w.ruleset.numbers.move_after_blocks && t.ap >= 1), "{tiles:?}");
    let t = tiles[0].clone();
    let r = b.w.iter().find(|w| w.id == format!("tile.{}.{}", t.cell.x, t.cell.y)).unwrap().rect;
    b.draw(&mut cx, &mut g, (r.x + r.w / 2, r.y + r.h / 2), true);
    b.draw(&mut cx, &mut g, (r.x + r.w / 2, r.y + r.h / 2), true);
    let note = b.w.iter().find(|w| w.id == "strip.move_note").expect("the tile's cost");
    let drops = if t.drops.is_empty() { String::new() } else { format!(", drops {}", t.drops) };
    assert_eq!(note.text, format!("Move here: {} block{}, {} AP{drops}", t.blocks, if t.blocks == 1 { "" } else { "s" }, t.ap));

    assert!(b.move_aim.is_some());
    b.act(&mut cx, Action::Confirm);
    assert!(b.move_aim.is_none() && b.move_tiles.is_empty(), "Confirm ends the wait");
}

/// User 2026-11-05: the menu opens beside the clicked unit, inside the battle view: never over the strip or off the screen,
/// flipping sides near an edge: both views, 1280x720 and 1920x1080, the three text sizes, a unit at each corner of the board.
#[test]
fn the_menu_opens_beside_the_unit_at_every_corner() {
    use crate::stage::{Geom, View};
    use emberwake_battle::grid::{GRID_H, GRID_W};
    let (mut cx, mut g, mut b) = arena();
    let foe = foe_of(&b);
    let view_rect = Rect::new(crate::battle::GRID_X - 2, crate::battle::GRID_Y - 2, GRID_W * crate::battle::TILE + 4, GRID_H * crate::battle::TILE + 4);
    for battle_view in ["flat", "stage"] {
        cx.settings.v["battle_view"] = json!(battle_view);
        let geo = Geom::new(View::of_setting(&cx.settings.v["battle_view"]), &cx.tuning.value);
        for (w_px, h_px) in [(1280, 720), (1920, 1080)] {
            g.canvas = aegis_ui::canvas::Canvas::new(w_px, h_px);
            g.view = aegis_ui::painter::View::fit(w_px as u32, h_px as u32);
            for corner in [V2::new(0, 0), V2::new(GRID_W - 1, 0), V2::new(0, GRID_H - 1), V2::new(GRID_W - 1, GRID_H - 1), V2::new(GRID_W / 2, GRID_H / 2)] {
                b.host.session.as_mut().unwrap().w.grid.positions[foe] = Some(corner);
                b.after_change(&cx);
                b.act(&mut cx, Action::ClosePopup);
                b.act(&mut cx, Action::UnitClick(foe));
                for scale in [1.0f32, 1.15, 1.3] {
                    crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
                    let at = format!("{battle_view} {w_px}x{h_px} {corner} scale {scale}");
                    let panel = popup(&b).rect(b.menu_row);
                    let tile = geo.cell_rect(corner);
                    assert!(panel.x >= view_rect.x && panel.x + panel.w <= view_rect.x + view_rect.w, "{at}: in the view across: {panel:?}");
                    assert!(panel.y >= view_rect.y && panel.y + panel.h <= view_rect.y + view_rect.h, "{at}: in the view down: {panel:?}");
                    assert!(panel.y >= 26, "{at}: clear of the strip: {panel:?}");
                    assert!(panel.x >= tile.x + tile.w || panel.x + panel.w <= tile.x, "{at}: beside the tile {tile:?}, not over it: {panel:?}");
                    if corner.x == GRID_W - 1 {
                        assert!(panel.x + panel.w <= tile.x, "{at}: flipped to the left");
                    }
                }
            }
        }
    }
}
