//! RULESET1-U slice 1 (RULESET1_INTENT UI1, UI2; the Arena chip, user 2026-10-29): the selected hero's round
//! controls on the battle screen. A catalog encounter played by the catalog's timeline row; every expectation comes
//! from the kit's planner (`round_intent_plan::preview`, `reasons`, `order`, `rest_preview`) and the intent table
//! (`round_intent::target_problem`, `would_drop`), never from shipped numbers.

use crate::battle::input::{engage_positions, plan_rows, Row, RowAct};
use crate::battle::{Action, BattleScreen, AP_PIPS, TACTICS, TACTIC_LABELS};
use crate::battle::tips::{ap_lines, inspect_tip, AP_PAID};
use aegis_ui::canvas::Rect;
use crate::intent_text;
use crate::screens::{Cx, Key, Screen};
use aegis_ui::ui::{Gfx, Widget, H, W};
use emberwake_battle::grid::{V2, GRID_H, GRID_W};
use emberwake_battle::round_intent::{self as ri, ManeuverKind, RoundIntent, RoundTarget};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::Phase;
use aegis_ui::art::Art;
use emberwake_battle::skill::Kind;
use emberwake_battle::world::World;
use serde_json::{json, Value};
use std::time::Duration;
use emberwake_rules::ruleset::{rule, RuleKind, Ruleset, PLAN_BADGES, TIMELINE};

fn arena(ruleset: Option<&str>, rules: &str, reveal: bool) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), rules, reveal);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

/// LOOK1-L1: the Stage draws the set of the battle's world, read from the manifest; a catalog encounter none.
#[test]
fn the_stage_draws_the_set_of_the_battles_world() {
    let mut cx = crate::hud::hud_tests::game_cx();
    let mut g = crate::hud::hud_tests::gfx();
    cx.settings.v["battle_view"] = json!("stage");
    // the manifest first: the battle's host shares the data from its start
    let world = emberwake_world::new_game::playable_worlds(&cx.data).first().cloned().expect("a playable world");
    std::sync::Arc::get_mut(&mut cx.app.data).expect("this test's own data").catalogs.worlds.get_mut(&world).expect("the world").manifest.insert("stage_set".into(), json!("zz_set"));
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.into()), "", false);
    assert_eq!(b.stage_set(&cx), "", "a catalog encounter");
    let root = std::env::temp_dir().join(format!("emberwake_battle_set_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    const MARK: [u8; 3] = [251, 3, 247];
    for id in emberwake_battle::grid::TERRAIN_IDS {
        let path = root.join(format!("tilesets/zz_set/{id}/v0.png"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), 64, 48);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&[MARK[0], MARK[1], MARK[2], 255].repeat(64 * 48)).unwrap();
    }
    let mut roots = vec![root.clone()];
    roots.extend(aegis_core::assets::Layers::for_game(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).roots);
    cx.art = Art::layered(aegis_core::assets::Layers::of(roots));
    let marked = |g: &Gfx| g.canvas.px.chunks_exact(4).filter(|c| c[..3] == MARK).count();
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(marked(&g), 0, "no set: today's tiles");
    b.arena_world = Some(world);
    assert_eq!(b.stage_set(&cx), "zz_set");
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(marked(&g) > 0, "the world's set draws");
    let _ = std::fs::remove_dir_all(&root);
}

fn drawn(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen) {
    b.draw(cx, g, (-500, -500), true);
}

fn widget<'a>(b: &'a BattleScreen, id: &str) -> &'a Widget<Action> {
    b.w.iter().find(|w| w.id == id).unwrap_or_else(|| panic!("no widget {id}"))
}

fn tip_has(w: &Widget<Action>, text: &str) -> bool {
    w.tip.iter().any(|t| t.0.contains(text))
}

fn hero(b: &BattleScreen) -> usize {
    b.selected.expect("a hero is selected")
}

fn w(b: &BattleScreen) -> &World {
    &b.host.session.as_ref().unwrap().w
}

/// The first valid cell no unit stands on.
fn free_tile(w: &World) -> V2 {
    (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).find(|c| w.grid.is_valid(*c) && (0..w.units.len()).all(|i| w.grid.pos(i) != Some(*c))).expect("a free tile")
}

/// A timeline Arena battle at its first planning (the tick drafts the ghosts, GH2 a), drawn.
fn planning() -> (Cx, Gfx, BattleScreen) {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), "", false);
    b.tick(&mut cx);
    drawn(&mut cx, &mut g, &mut b);
    (cx, g, b)
}

fn foes_on_field(b: &BattleScreen) -> Vec<usize> {
    w(b).enemies.iter().copied().filter(|&e| w(b).units[e].is_active() && w(b).grid.pos(e).is_some()).collect()
}

/// Puts the selected hero on a free tile next to `f` (so its melee skills reach), the battle catching up.
fn next_to(cx: &Cx, b: &mut BattleScreen, f: usize) {
    let h = hero(b);
    let fp = w(b).pos(f);
    let cell = emberwake_battle::grid::DIRS.iter().map(|d| fp + *d).find(|c| w(b).grid.is_valid(*c) && w(b).grid.is_walkable(*c) && !w(b).grid.is_exit(*c) && w(b).occupant(*c).is_none_or(|o| o == h)).expect("a free tile next to the foe");
    b.host.session.as_mut().unwrap().w.grid.positions[h] = Some(cell);
    b.after_change(cx);
}

fn round_plan(b: &mut BattleScreen) -> Value {
    b.host.handle(&json!({"cmd": "round_plan"}))["round_plan"].clone()
}

/// The intent of a skill-menu row (its first skill or item pick's; None: neutral).
fn row_intent(b: &BattleScreen, row: &Row) -> Option<String> {
    match &row.act {
        RowAct::Pick(picks) => picks.iter().find_map(|p| match p {
            rip::Pick::Skill { skill, .. } => Some(ri::skill_intent(&w(b).cat.skills[*skill]).to_string()),
            _ => None,
        }),
        _ => None,
    }
}

fn first_valid_pick(b: &BattleScreen) -> usize {
    let p = b.popup.as_ref().expect("the skill menu");
    let at = p.menu.map(|m| m.1);
    p.rows.iter().position(|r| r.valid && r.at == at && matches!(&r.act, RowAct::Pick(p) if p.iter().any(|x| x.holds_target()))).expect("a usable skill row")
}

/// IF1, IF2, IF4: a click on an enemy sets the round target and opens the skill menu at the click, titled "<hero> ->
/// <target>", with only skills an enemy takes (no Support or Guard row); a pick appends it and the menu stays open,
/// re-listed: only rows that can still be picked, of the round intent.
#[test]
fn a_target_click_opens_the_skill_menu_and_a_pick_relists_it() {
    let (mut cx, mut g, mut b) = planning();
    let (h, f) = (hero(&b), foes_on_field(&b)[0]);
    next_to(&cx, &mut b, f);
    b.run(&mut cx, Action::UnitClick(f));
    assert_eq!(w(&b).round_states[&h].target, Some(RoundTarget::Unit(f)), "the click sets the target at once: {}", b.message);
    let p = b.popup.as_ref().expect("the skill menu");
    assert_eq!((p.menu, p.title.clone()), (Some((h, RoundTarget::Unit(f))), format!("{} -> {}", w(&b).units[h].name, w(&b).units[f].name)));
    let s = b.host.session.as_ref().unwrap();
    assert_eq!(p.rows.iter().filter(|r| matches!(r.act, crate::battle::input::RowAct::Pick(_))).count(), rip::menu_options(&s.w, &s.plan, h, RoundTarget::Unit(f)).len(), "one row per kit cell");
    for row in p.rows.iter().filter(|r| r.valid) {
        assert!(row_intent(&b, row).is_none_or(|i| i == "assault" || i == "maneuver"), "an enemy takes Assault and Maneuver skills: {}", row.label);
    }
    let k = first_valid_pick(&b);
    b.run(&mut cx, Action::Choose(k));
    assert_eq!(rip::picks_of(w(&b), h).len(), 1, "{}", b.message);
    let intent = ri::intent_of(w(&b), h).map(|i| i.id().to_string());
    if let Some(p) = b.popup.as_ref() {
        // VISCLAR1-D: the rows stay and the other intents' fade (`skill_menu_tests`)
        assert!(p.rows.iter().filter(|r| r.valid).all(|r| row_intent(&b, r).is_none_or(|i| Some(i) == intent)), "only the round intent's rows can be picked");
    }
    let rp = round_plan(&mut b);
    assert_eq!(rp.as_array().unwrap().iter().find(|r| r["unit"] == json!(h)).unwrap()["picks"].as_array().unwrap().len(), 1);
    drawn(&mut cx, &mut g, &mut b);
}

/// IF2, IF6 b: after a pick on one enemy another enemy offers no skill: "One target per round: <target>".
#[test]
fn another_enemy_after_a_pick_offers_no_skill() {
    let (mut cx, _g, mut b) = planning();
    let foes = foes_on_field(&b);
    assert!(foes.len() >= 2, "the arena fight has two enemies on the field");
    let h = hero(&b);
    next_to(&cx, &mut b, foes[0]);
    b.run(&mut cx, Action::UnitClick(foes[0]));
    let k = first_valid_pick(&b);
    b.run(&mut cx, Action::Choose(k));
    assert!(!rip::picks_of(w(&b), h).is_empty(), "{}", b.message);
    b.run(&mut cx, Action::ClosePopup);
    b.run(&mut cx, Action::UnitClick(foes[1]));
    assert!(b.popup.as_ref().is_none_or(|p| !p.rows.iter().any(|r| matches!(r.act, RowAct::Pick(_)))), "no skill row");
    assert_eq!(b.message, format!("One target per round: {}", w(&b).units[foes[0]].name));
}

/// IF5: hovering a row previews it on the strip; leaving it restores the strip; the round plan never changes.
#[test]
fn hovering_a_row_previews_it_and_changes_nothing() {
    let (mut cx, mut g, mut b) = planning();
    let f = foes_on_field(&b)[0];
    next_to(&cx, &mut b, f);
    b.run(&mut cx, Action::UnitClick(f));
    drawn(&mut cx, &mut g, &mut b);
    let before = round_plan(&mut b);
    let k = first_valid_pick(&b);
    let r = widget(&b, &format!("popup.{k}")).rect;
    b.draw(&mut cx, &mut g, (r.x + r.w / 2, r.y + r.h / 2), true);
    assert!(b.preview.as_ref().is_some_and(|p| p.1 == k && p.2.is_some()), "the hovered row is previewed");
    assert_eq!(round_plan(&mut b), before, "a preview changes nothing");
    drawn(&mut cx, &mut g, &mut b);
    assert!(b.preview.is_none(), "leaving the row restores the strip");
    assert_eq!(round_plan(&mut b), before);
}

/// IF10: with the menu open a right click (back) or a click outside it closes it and does nothing else.
#[test]
fn closing_the_menu_does_nothing_else() {
    let (mut cx, mut g, mut b) = planning();
    let (h, f) = (hero(&b), foes_on_field(&b)[0]);
    b.run(&mut cx, Action::UnitClick(f));
    drawn(&mut cx, &mut g, &mut b);
    let p = b.popup.clone().expect("the skill menu");
    let outside = [(1, 1), (W - 2, H - 2), (1, H - 2)].into_iter().find(|&(x, y)| !p.rect(b.menu_row).contains(x, y)).unwrap();
    assert_eq!(aegis_ui::ui::hit(&b.w, outside), Some(Action::ClosePopup), "a click outside lands on the backdrop");
    let before = (round_plan(&mut b), b.selected);
    b.back(&mut cx);
    assert!(b.popup.is_none());
    assert_eq!((round_plan(&mut b), b.selected), before, "the right click only closed the menu");
    assert_eq!(b.selected, Some(h));
}

/// IF8, SL4: the intent buttons, kind and opener menus, Manual and Fill are gone; the panel names the intent and its
/// source; the Self button (and key S) opens the self menu.
#[test]
fn the_round_controls_are_gone_and_self_opens_the_self_menu() {
    let (mut cx, mut g, mut b) = planning();
    let h = hero(&b);
    assert!(b.w.iter().all(|x| !x.id.starts_with("hero.intent.") && !["hero.kind", "hero.opener", "hero.manual", "hero.fill"].contains(&x.id.as_str())), "the old controls are gone");
    let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
    if ri::intent_of(w(&b), h).is_some() {
        let tactic = TACTIC_LABELS[b.tactic_of(h)];
        assert!(recs.iter().any(|r| r.text.starts_with("Intent: ") && r.text.ends_with(&format!("from {tactic}"))), "the ghost's intent from the tactic");
    }
    b.click_id(&mut cx, "hero.self").unwrap();
    assert_eq!(b.popup.as_ref().and_then(|p| p.menu), Some((h, RoundTarget::Myself)), "{}", b.message);
    b.run(&mut cx, Action::ClosePopup);
    b.key(&mut cx, Key::Char('s'));
    assert_eq!(b.popup.as_ref().and_then(|p| p.menu), Some((h, RoundTarget::Myself)));
}

/// HK1-HK2, INPUT2: with the menu open Shift+digit picks its row; with it closed the hotkey skill goes on the round
/// target; without a target the key asks for one. A plain digit no longer picks a skill.
#[test]
fn digits_pick_the_hotkey_skills() {
    let (mut cx, mut g, mut b) = planning();
    let (h, f) = (hero(&b), foes_on_field(&b)[0]);
    next_to(&cx, &mut b, f);
    b.run(&mut cx, Action::UnitClick(f));
    let d = (1..=9).find(|d| b.popup.as_ref().unwrap().rows.iter().any(|r| r.valid && r.label.starts_with(&format!("{d} ")))).expect("a usable hotkey row");
    if d > 4 {
        b.key(&mut cx, Key::Char(char::from_digit(d as u32, 10).unwrap()));
        assert!(rip::picks_of(w(&b), h).is_empty(), "a plain digit picks nothing: {}", b.message);
    }
    b.key(&mut cx, Key::ShiftDigit(d as u8));
    assert_eq!(rip::picks_of(w(&b), h).len(), 1, "{}", b.message);
    b.run(&mut cx, Action::ClosePopup);
    b.run(&mut cx, Action::Clear);
    assert!(rip::picks_of(w(&b), h).is_empty(), "{}", b.message);
    b.key(&mut cx, Key::ShiftDigit(d as u8));
    assert_eq!(rip::picks_of(w(&b), h).len(), 1, "menu closed: the skill on the round target: {}", b.message);
    // no target and no intent: the digit of a skill that is not self-only asks for a target
    let keys = rip::hotkey_skills(w(&b), h);
    let other = keys.iter().position(|k| { let sk = &w(&b).cat.skills[*k]; sk.target_mode != "SELF" && emberwake_battle::skill::kind(sk) != Kind::Defend }).filter(|k| *k < 9).expect("a hotkey skill aimed at others");
    b.run(&mut cx, Action::Clear);
    let rs = b.host.session.as_mut().unwrap().w.round_states.entry(h).or_default();
    rs.target = None;
    rs.intent = None;
    b.after_change(&cx);
    drawn(&mut cx, &mut g, &mut b);
    b.key(&mut cx, Key::ShiftDigit(other as u8 + 1));
    assert_eq!(b.message, "Pick a target first (click a unit or a tile)");
}

/// GH2 a: at the planning start each hero without a pick shows its tactic's draft as ghost; GH2 b: a clicked target
/// is player-set, and Clear after a pick returns to the auto plan on it; the hero tactic returns it to (a).
#[test]
fn heroes_show_their_tactic_ghost_and_clear_returns_to_it() {
    let (mut cx, _g, mut b) = planning();
    for a in w(&b).allies.clone().into_iter().filter(|&a| w(&b).units[a].is_active() && w(&b).has(a)) {
        let s = b.host.session.as_ref().unwrap();
        assert!(rip::picks_of(&s.w, a).is_empty() && rip::ghost_entries(&s.w, &s.plan, a).iter().all(|g| *g), "{a}: all ghost");
        let mut w2 = s.w.clone();
        let mut p2 = rip::copy_plan(&s.plan);
        emberwake_battle::ai::plan_party_intents(&mut w2, &mut p2, &[(a, TACTICS[b.tactic_of(a)].to_string())]);
        assert_eq!(plan_rows(&s.w, &s.plan, a), plan_rows(&w2, &p2, a), "{a}: the tactic's draft");
    }
    let (h, f) = (hero(&b), foes_on_field(&b)[0]);
    next_to(&cx, &mut b, f);
    b.run(&mut cx, Action::UnitClick(f));
    assert!(b.player_target.contains(&h));
    let k = first_valid_pick(&b);
    b.run(&mut cx, Action::Choose(k));
    b.run(&mut cx, Action::ClosePopup);
    b.run(&mut cx, Action::Clear);
    let s = b.host.session.as_ref().unwrap();
    let mut w2 = s.w.clone();
    let mut p2 = rip::copy_plan(&s.plan);
    rip::plan_unit(&mut w2, &mut p2, h).unwrap();
    assert_eq!(plan_rows(&s.w, &s.plan, h), plan_rows(&w2, &p2, h), "(b): the auto plan on the clicked target");
    assert_eq!(s.w.round_states[&h].target, Some(RoundTarget::Unit(f)), "the target stays");
    b.run(&mut cx, Action::HeroMenu);
    b.run(&mut cx, Action::Choose(b.tactic_of(h)));
    assert!(!b.player_target.contains(&h), "the hero tactic: back to (a)");
}

/// GH7: Auto-fill off reaches the battle and the free blocks Rest (the ghost of a hero with no pick, the blocks
/// after a pick).
#[test]
fn auto_fill_off_rests_the_free_blocks() {
    let (mut cx, _g, mut b) = planning();
    cx.settings.v["auto_fill"] = json!(false);
    b.tick(&mut cx);
    assert!(!w(&b).autofill, "the setting reached the battle");
    let rest_only = |b: &BattleScreen, a: usize, from: usize| {
        let s = b.host.session.as_ref().unwrap();
        s.plan.entries_for(&s.w, a).iter().skip(from).filter(|e| e.step_index >= 0).all(|e| emberwake_battle::skill::kind(&s.w.cat.skills[e.skill]) == Kind::Rest)
    };
    for a in w(&b).allies.clone().into_iter().filter(|&a| w(&b).units[a].is_active()) {
        assert!(rest_only(&b, a, 0), "{a}: Rest on every block");
    }
    let (h, f) = (hero(&b), foes_on_field(&b)[0]);
    next_to(&cx, &mut b, f);
    b.run(&mut cx, Action::UnitClick(f));
    let k = first_valid_pick(&b);
    b.run(&mut cx, Action::Choose(k));
    let s = b.host.session.as_ref().unwrap();
    let from = rip::ghost_from(&s.w, &s.plan, h);
    assert!(rest_only(&b, h, from), "after the pick the free blocks Rest");
}

/// GH2, IN6 (Q18 a): a clicked target whose intent change would drop an intent-bound buff waits for Confirm: the
/// buff stays, the panel warns "Changing loses ..."; Confirm makes the change.
#[test]
fn a_ghost_that_drops_a_bound_buff_waits_for_confirm() {
    let (mut cx, mut g, mut b) = planning();
    let (h, f) = (hero(&b), foes_on_field(&b)[0]);
    let r = b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "guard", "target": "self"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    let s = b.host.session.as_mut().unwrap();
    emberwake_rules::status::grant_now(&mut s.w.units[h], "riposte", 2, 0, "test");
    ri::bind(&mut s.w, h, h, &["riposte".to_string()], false);
    next_to(&cx, &mut b, f);
    b.run(&mut cx, Action::UnitClick(f));
    assert!(b.deferred.iter().any(|d| d.0 == h), "the change waits: {}", b.message);
    assert_eq!(ri::intent_of(w(&b), h), Some(RoundIntent::Guard));
    assert!(w(&b).units[h].has_status("riposte"), "the buff stays until Confirm");
    b.run(&mut cx, Action::ClosePopup);
    let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
    assert!(recs.iter().any(|r| r.text.starts_with("Changing loses")), "the panel warns");
    b.run(&mut cx, Action::Confirm);
    assert_ne!(ri::intent_of(w(&b), h), Some(RoundIntent::Guard), "{}", b.message);
    assert!(!w(&b).units[h].has_status("riposte"));
}

/// GH8: a ghost block's tip starts "Auto-filled" with its reason tag; a picked block's does not.
#[test]
fn ghost_blocks_say_auto_filled() {
    let (mut cx, mut g, mut b) = planning();
    let (h, f) = (hero(&b), foes_on_field(&b)[0]);
    next_to(&cx, &mut b, f);
    b.run(&mut cx, Action::UnitClick(f));
    let k = first_valid_pick(&b);
    b.run(&mut cx, Action::Choose(k));
    b.run(&mut cx, Action::ClosePopup);
    drawn(&mut cx, &mut g, &mut b);
    let s = b.host.session.as_ref().unwrap();
    let (entries, ghost, reasons) = (s.plan.entries_for(&s.w, h), rip::ghost_entries(&s.w, &s.plan, h), rip::reasons(&s.w, &s.plan, h));
    assert!(ghost.iter().any(|g| !*g), "a picked entry");
    for ((e, gh), why) in entries.iter().zip(&ghost).zip(&reasons).filter(|((e, _), _)| e.step_index >= 0) {
        let block = widget(&b, &format!("strip.block{}", e.start_block));
        let first = block.tip.first().map(|t| t.0.clone()).unwrap_or_default();
        assert_eq!(first.starts_with("Auto-filled"), *gh, "block {}: {first}", e.start_block);
        if *gh && !why.is_empty() {
            assert!(tip_has(block, &format!("Auto-filled: {why}")));
        }
    }
}

/// ET1: an empty tile's menu opens with Move here.
#[test]
fn an_empty_tile_offers_move_here_first() {
    let (mut cx, _g, mut b) = planning();
    let free = free_tile(w(&b));
    b.run(&mut cx, Action::TileClick(free));
    let p = b.popup.as_ref().expect("the tile menu");
    assert_eq!(p.menu, Some((hero(&b), RoundTarget::Cell(free))));
    let row = p.rows.iter().find(|r| r.slot == Some(crate::battle::skill_menu::Slot::Neutral("Move"))).expect("the neutral row's Move");
    assert_eq!(row.label, "Move here");
}

/// MR13: hovering a Maneuver Engage row marks its target's open position tiles (the kit's `positions`).
#[test]
fn hovering_an_engage_row_marks_the_position_tiles() {
    let (mut cx, mut g, mut b) = planning();
    let (h, f) = (hero(&b), foes_on_field(&b)[0]);
    b.run(&mut cx, Action::UnitClick(f));
    drawn(&mut cx, &mut g, &mut b);
    let p = b.popup.clone().expect("the skill menu");
    let k = p.rows.iter().position(|r| matches!(&r.act, RowAct::Pick(x) if matches!(x[..], [rip::Pick::Skill { kind: Some(ManeuverKind::Engage), skill }] if ri::position_of(w(&b), &w(&b).cat.skills[skill]).is_some()))).expect("a Maneuver Engage row with a position");
    let RowAct::Pick(picks) = p.rows[k].act.clone() else { unreachable!() };
    let want = engage_positions(b.host.session.as_ref().unwrap(), h, RoundTarget::Unit(f), &picks);
    let r = widget(&b, &format!("popup.{k}")).rect;
    b.draw(&mut cx, &mut g, (r.x + r.w / 2, r.y + r.h / 2), true);
    assert_eq!(b.positions_shown, want);
    assert_eq!(b.state(&cx)["overlay"]["positions"].as_array().unwrap().len(), want.len());
    drawn(&mut cx, &mut g, &mut b);
    assert!(b.positions_shown.is_empty(), "only while hovered");
}

/// TEXTFIT1: the skill menu and the hero panel fit at the four text sizes (no spill), 1280x720.
#[test]
fn the_menu_and_the_hero_panel_fit_at_every_text_size() {
    let (mut cx, mut g, mut b) = planning();
    g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
    g.view = aegis_ui::painter::View::fit(1280, 720);
    let f = foes_on_field(&b)[0];
    next_to(&cx, &mut b, f);
    b.run(&mut cx, Action::UnitClick(f));
    let k = first_valid_pick(&b);
    b.run(&mut cx, Action::Choose(k));
    b.run(&mut cx, Action::UnitClick(f));
    for scale in [0.85f32, 1.0, 1.15, 1.3] {
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
        let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
        let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
        let ours: Vec<Value> = v["spill"].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("popup.") || w.starts_with("hero."))).cloned().collect();
        assert!(ours.is_empty(), "scale {scale}: {ours:?}");
    }
}

// ---- INPUT1-U slice 3: selection and inspect (INPUT1 SL1-SL7, IS1-IS3) ----------------------------------------

/// A right click over widget `id` (the mouse there, as the shell's `back`), after a frame so the widgets are known.
fn right_click(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen, id: &str, unit: usize) {
    drawn(cx, g, b);
    let r = widget(b, id).rect;
    let at = (r.x + r.w / 2, r.y + r.h / 2);
    assert!(matches!(aegis_ui::ui::hit(&b.w, at), Some(Action::UnitClick(u) | Action::Select(u)) if u == unit), "{id}'s middle is unit {unit}: {:?} {:?}", b.popup.as_ref().map(|p| p.title.clone()), b.w.iter().filter(|w| w.rect.contains(at.0, at.1)).map(|w| (w.id.clone(), w.enabled, w.action.clone())).collect::<Vec<_>>());
    b.mouse = at;
    b.back(cx);
}

/// The texts drawn inside the intel panel at the last frame.
fn intel_text(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen) -> String {
    let recs = crate::hud::hud_tests::frame(b, cx, g, 1.0);
    let r = b.panels.iter().find(|p| p.0 == "intel").expect("the intel panel").1;
    recs.iter().filter(|t| r.contains(t.x as i32 + 1, t.y as i32 + 1)).map(|t| t.text.clone()).collect::<Vec<_>>().join("\n")
}

/// SL1, SL4-SL7: with a hero selected and planning a click on an enemy targets it and a click on the hero itself (its
/// token or its card) selects nothing; with none selected a click on an enemy inspects it (no target, no menu) and
/// one on an ally selects it; outside planning the same.
#[test]
fn clicks_select_target_and_inspect() {
    let (mut cx, _g, mut b) = planning();
    let (h, f) = (hero(&b), foes_on_field(&b)[0]);
    b.run(&mut cx, Action::UnitClick(f));
    assert_eq!(w(&b).round_states[&h].target, Some(RoundTarget::Unit(f)), "SL1: the click targets");
    b.run(&mut cx, Action::ClosePopup);
    let before = round_plan(&mut b);
    b.run(&mut cx, Action::UnitClick(h));
    assert_eq!((b.selected, b.popup.is_none()), (None, true), "SL4: the hero's token selects nothing");
    assert_eq!(round_plan(&mut b), before, "and plans nothing");
    let target = w(&b).round_states[&h].target;
    b.run(&mut cx, Action::UnitClick(f));
    assert_eq!((b.inspected(), b.selected, b.popup.is_none()), (Some(f), None, true), "SL5: nothing selected: the enemy is inspected");
    assert_eq!(w(&b).round_states[&h].target, target, "no target set");
    b.run(&mut cx, Action::UnitClick(h));
    assert_eq!(b.selected, Some(h), "SL5: an ally is selected");
    b.run(&mut cx, Action::Select(h));
    assert_eq!(b.selected, None, "SL4: the selected hero's card selects nothing");
    b.run(&mut cx, Action::Select(h));
    assert_eq!(b.selected, Some(h), "SL6: a card selects");
    let state = b.state(&cx);
    assert_eq!(state["inspect"], json!(f), "the agent link's inspect");
    // SL7: outside planning
    b.run(&mut cx, Action::Confirm);
    assert!(!b.planning(), "{}", b.message);
    b.inspect = None;
    b.run(&mut cx, Action::UnitClick(f));
    assert_eq!((b.inspected(), b.selected), (Some(f), Some(h)), "SL7: playback: the enemy is inspected, the hero stays selected");
    let other = w(&b).allies.iter().copied().find(|&a| a != h);
    if let Some(a) = other {
        b.run(&mut cx, Action::UnitClick(a));
        assert_eq!(b.selected, Some(a), "SL7: playback: an ally is selected");
    }
}

/// SL2, IS1, IF10: a right click on any unit (token or card) inspects it and changes neither the selection nor the
/// plan, also on the selected hero; with the menu open it only closes the menu; elsewhere it undoes the last pick.
#[test]
fn a_right_click_on_a_unit_inspects_it() {
    let (mut cx, mut g, mut b) = planning();
    let (h, f) = (hero(&b), foes_on_field(&b)[0]);
    let before = (round_plan(&mut b), b.selected);
    right_click(&mut cx, &mut g, &mut b, &format!("party.{h}"), h);
    assert_eq!(b.inspected(), Some(h), "{}", b.message);
    assert_eq!((round_plan(&mut b), b.selected), before, "the selected hero inspected: selection and plan unchanged");
    right_click(&mut cx, &mut g, &mut b, &format!("unit.{f}"), f);
    assert_eq!(b.inspected(), Some(f), "the enemy's token");
    assert_eq!((round_plan(&mut b), b.selected), before, "no target, no select");
    right_click(&mut cx, &mut g, &mut b, &format!("foe.{f}"), f);
    assert_eq!(b.inspected(), Some(f), "the enemy's card");
    // the menu open: the right click only closes it (IF10)
    next_to(&cx, &mut b, f);
    b.run(&mut cx, Action::UnitClick(f));
    assert!(b.popup.is_some());
    b.inspect = None;
    drawn(&mut cx, &mut g, &mut b);
    let r = widget(&b, &format!("party.{h}")).rect;
    b.mouse = (r.x + r.w / 2, r.y + r.h / 2);
    b.back(&mut cx);
    assert_eq!((b.popup.is_none(), b.inspected()), (true, None), "the menu closed, nothing inspected");
    // elsewhere: undo the last pick
    b.run(&mut cx, Action::UnitClick(f));
    let k = first_valid_pick(&b);
    b.run(&mut cx, Action::Choose(k));
    b.run(&mut cx, Action::ClosePopup);
    assert_eq!(rip::picks_of(w(&b), h).len(), 1, "{}", b.message);
    drawn(&mut cx, &mut g, &mut b);
    b.mouse = (-500, -500);
    b.back(&mut cx);
    assert!(rip::picks_of(w(&b), h).is_empty(), "a right click off every unit undoes: {}", b.message);
}

/// SL3: Esc closes the menu (only), then clears the inspected unit, then selects nothing, then leaves the Arena.
#[test]
fn escape_steps_back_one_layer_at_a_time() {
    let (mut cx, mut g, mut b) = planning();
    let (h, f) = (hero(&b), foes_on_field(&b)[0]);
    right_click(&mut cx, &mut g, &mut b, &format!("foe.{f}"), f);
    b.run(&mut cx, Action::UnitClick(f));
    assert!(b.popup.is_some());
    b.escape(&mut cx);
    assert_eq!((b.popup.is_none(), b.inspected(), b.selected), (true, Some(f), Some(h)), "the menu only");
    b.escape(&mut cx);
    assert_eq!((b.inspected(), b.selected), (None, Some(h)), "then the inspected unit");
    assert!(cx.nav.is_empty());
    b.escape(&mut cx);
    assert_eq!(b.selected, None, "then the selection");
    assert!(cx.nav.is_empty(), "nothing left yet");
    b.escape(&mut cx);
    assert!(!cx.nav.is_empty(), "then the Arena leaves, as before");
}

/// IS1, IS2: the inspected enemy pins the intel panel ("Inspecting <name> (Esc clears)") over the hovered unit until
/// Esc or the round's end; an unrevealed enemy's panel names none of its plan's skills but the tell's first action,
/// and that one only while the tells show.
#[test]
fn the_inspected_enemy_shows_only_what_the_party_knows() {
    let (mut cx, mut g, mut b) = planning();
    let f = foes_on_field(&b)[0];
    right_click(&mut cx, &mut g, &mut b, &format!("foe.{f}"), f);
    let s = b.host.session.as_ref().unwrap();
    let name = s.w.units[f].name.clone();
    let tell = s.enemy_intents().into_iter().find(|r| r.unit == f).expect("the enemy's tell");
    assert!(!tell.revealed, "the arena fight reveals no plan");
    let first = intent_text::first_line(&s.w, &tell.line);
    let skills: Vec<String> = s.resolver.scheduled.iter().filter(|a| a.unit == f).map(|a| a.command.name.clone()).filter(|n| !first.contains(n.as_str())).collect();
    // another unit under the mouse: the panel stays pinned
    let other = foes_on_field(&b).into_iter().find(|&e| e != f).unwrap_or(hero(&b));
    let id = if w(&b).allies.contains(&other) { format!("party.{other}") } else { format!("foe.{other}") };
    drawn(&mut cx, &mut g, &mut b);
    let r = widget(&b, &id).rect;
    b.draw(&mut cx, &mut g, (r.x + 2, r.y + 2), true);
    assert_eq!(widget(&b, "intel.unit").text, format!("Inspecting {name} (Esc clears)"), "pinned over the hover");
    let shown = intel_text(&mut cx, &mut g, &mut b);
    for n in &skills {
        assert!(!shown.contains(n.as_str()), "the panel names {n} of an unrevealed plan:\n{shown}");
    }
    assert!(shown.contains("Intent: "), "the tell's first action while the tells show:\n{shown}");
    cx.settings.v["show_tells"] = json!(false);
    let hidden = intel_text(&mut cx, &mut g, &mut b);
    assert!(!hidden.contains("Intent: "), "tells hidden: no first action:\n{hidden}");
    for n in s_names(&b, f) {
        assert!(!hidden.contains(n.as_str()), "tells hidden: no plan skill {n}:\n{hidden}");
    }
    cx.settings.v["show_tells"] = json!(true);
    // the round's end clears it
    b.send(&cx, json!({"cmd": "resolve"}));
    assert_eq!(b.inspected(), Some(f), "kept through the playback");
    b.run(&mut cx, Action::NextRound);
    assert_eq!(b.inspected(), None, "the next round starts unpinned: {}", b.message);
}

/// The names of `f`'s planned commands this round.
fn s_names(b: &BattleScreen, f: usize) -> Vec<String> {
    b.host.session.as_ref().unwrap().resolver.scheduled.iter().filter(|a| a.unit == f).map(|a| a.command.name.clone()).collect()
}

/// IS3: an inspected party member shows its intent, its picks and plan (one line an entry, ghosts marked) and its AP
/// numbers (AP9's lines from the kit's `ap_forecast`); the selected hero stays selected.
#[test]
fn an_inspected_member_shows_its_plan_and_ap() {
    let (mut cx, mut g, mut b) = planning();
    let h = hero(&b);
    let m = w(&b).allies.iter().copied().find(|&a| a != h && w(&b).units[a].is_active()).unwrap_or(h);
    right_click(&mut cx, &mut g, &mut b, &format!("party.{m}"), m);
    assert_eq!((b.inspected(), b.selected), (Some(m), Some(h)), "IS3: the selection stays");
    drawn(&mut cx, &mut g, &mut b);
    let s = b.host.session.as_ref().unwrap();
    let lines: Vec<String> = inspect_tip(s, m, b.selected, &s.enemy_intents(), &s.round_intents(), &[], true).into_iter().map(|l| l.0).collect();
    for l in ap_lines(&rip::ap_forecast(&s.w, &s.plan, m)) {
        assert!(lines.contains(&l), "AP9 line {l:?} in {lines:?}");
    }
    let entries = s.plan.entries_for(&s.w, m);
    let ghosts = rip::ghost_entries(&s.w, &s.plan, m);
    assert!(lines.iter().any(|l| l.starts_with("Picks: ")), "{lines:?}");
    for (e, gh) in entries.iter().zip(&ghosts) {
        let name = &s.w.cat.skills[e.skill].name;
        assert!(lines.iter().any(|l| l.starts_with(&intent_text::block_text((e.start_block, e.end_block(&s.w)))) && l.contains(name.as_str()) && l.ends_with(", auto-filled") == *gh), "entry {name} at B{}: {lines:?}", e.start_block);
    }
    let rec = s.round_intents().into_iter().find(|r| r.unit == m);
    if let Some(head) = rec.as_ref().and_then(|r| intent_text::round_head(&s.w, r)) {
        assert!(lines.contains(&head), "its intent and target: {head}");
    }
    assert_eq!(widget(&b, "intel.unit").text, format!("Inspecting {} (Esc clears)", w(&b).units[m].name));
}

/// TEXTFIT1: the inspect panel's texts (an enemy's, a party member's) stay inside the panel at the four text sizes,
/// 1280x720.
#[test]
fn the_inspect_panel_fits_at_every_text_size() {
    let (mut cx, mut g, mut b) = planning();
    g.canvas = aegis_ui::canvas::Canvas::new(1280, 720);
    g.view = aegis_ui::painter::View::fit(1280, 720);
    let (h, f) = (hero(&b), foes_on_field(&b)[0]);
    for u in [f, h] {
        b.inspect = Some((u, b.session().current_round));
        for scale in [0.85f32, 1.0, 1.15, 1.3] {
            let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
            let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            let r = b.panels.iter().find(|p| p.0 == "intel").expect("the intel panel").1;
            let inside = |e: &Value| r.contains(e["at"][0].as_f64().unwrap_or(-1.0) as i32 + 1, e["at"][1].as_f64().unwrap_or(-1.0) as i32 + 1);
            let ours: Vec<Value> = v["spill"].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("intel."))).chain(v["cut"].as_array().unwrap().iter().filter(|e| inside(e))).cloned().collect();
            assert!(ours.is_empty(), "unit {u} scale {scale}: {ours:?}");
        }
    }
}

// ---- INPUT2: party keys, Shift skill keys, the ghost fill (decision "Playtest 2026-11-02") ---------------------

/// The selected hero's ghost entries (aligned with its plan entries).
fn ghosts_of(b: &BattleScreen, u: usize) -> Vec<bool> {
    let s = b.host.session.as_ref().unwrap();
    rip::ghost_entries(&s.w, &s.plan, u)
}

fn rows_of(b: &BattleScreen, u: usize) -> Vec<(usize, i32, Option<usize>, V2)> {
    let s = b.host.session.as_ref().unwrap();
    plan_rows(&s.w, &s.plan, u)
}

/// Keys 1-4 select the party's 1st to 4th member (the party cards' order) and close an open menu; a plain digit
/// picks no skill.
#[test]
fn keys_one_to_four_select_the_party_members() {
    let (mut cx, _g, mut b) = planning();
    let allies = w(&b).allies.clone();
    for (k, &a) in allies.iter().enumerate().take(4) {
        b.key(&mut cx, Key::Char(char::from_digit(k as u32 + 1, 10).unwrap()));
        if w(&b).units[a].is_active() {
            assert_eq!(b.selected, Some(a), "key {} selects the party's member {}", k + 1, k + 1);
        }
    }
    let h = allies[0];
    b.key(&mut cx, Key::Char('1'));
    let f = foes_on_field(&b)[0];
    next_to(&cx, &mut b, f);
    b.run(&mut cx, Action::UnitClick(f));
    assert!(b.popup.is_some(), "the skill menu on the foe");
    let picks = rip::picks_of(w(&b), h).len();
    b.key(&mut cx, Key::Char('1'));
    assert_eq!((b.selected, b.popup.is_none(), rip::picks_of(w(&b), h).len()), (Some(h), true, picks), "key 1 selects, closes the menu and picks nothing");
    if allies.len() < 4 {
        b.key(&mut cx, Key::Char('4'));
        assert_eq!((b.selected, b.message.clone()), (Some(h), "No party member on key 4.".to_string()));
    }
}

/// The playtest bug: while the hero's round is its ghost, a click on another party member selects that member; the
/// hero's round target and plan stay as they were.
#[test]
fn a_click_on_another_member_selects_it_while_the_ghost_shows() {
    let (mut cx, mut g, mut b) = planning();
    let h = hero(&b);
    let other = w(&b).allies.iter().copied().find(|&a| a != h && w(&b).units[a].is_active() && w(&b).has(a)).expect("a second party member on the field");
    assert!(rip::picks_of(w(&b), h).is_empty() && ghosts_of(&b, h).iter().all(|g| *g), "the hero shows its ghost");
    let (target, rows) = (w(&b).round_states.get(&h).and_then(|r| r.target), rows_of(&b, h));
    b.run(&mut cx, Action::UnitClick(other));
    assert_eq!(b.selected, Some(other), "the click selects the member: {}", b.message);
    assert!(b.popup.is_none());
    assert_eq!(w(&b).round_states.get(&h).and_then(|r| r.target), target, "not the hero's round target");
    assert_eq!(rows_of(&b, h), rows, "the hero's plan is unchanged");
    // through the drawn token too (the hit test reaches the member's unit widget)
    b.selected = Some(h);
    drawn(&mut cx, &mut g, &mut b);
    let r = widget(&b, &format!("unit.{other}")).rect;
    if let Some(a) = aegis_ui::ui::hit(&b.w, (r.x + r.w / 2, r.y + r.h / 2)) {
        b.run(&mut cx, a);
        assert_eq!(b.selected, Some(other));
    }
}

/// INPUT2 (user 2026-11-02, Q3): while the hero shows only its ghost, Ctrl+click on another member targets it (SL6:
/// the skill menu "<hero> -> <ally>" with the hero's support rows and "Select <ally>" last); the hero stays selected.
#[test]
fn ctrl_click_on_a_member_opens_the_support_menu_while_the_ghost_shows() {
    let (mut cx, _g, mut b) = planning();
    let allies: Vec<usize> = w(&b).allies.iter().copied().filter(|&a| w(&b).units[a].is_active() && w(&b).has(a)).collect();
    // a member with a support row it can pick on another member (the pair found by the kit's menu rows); without one
    // in the fixture, the first member learns a support skill of the catalog it can use (in memory, a meter skill:
    // listed after its actives, HK1)
    let reaches = |b: &BattleScreen, h: usize, o: usize| {
        let s = b.host.session.as_ref().unwrap();
        rip::pick_options(&s.w, &s.plan, h, RoundTarget::Unit(o)).iter().any(|r| r.problem.is_none() && r.picks.iter().any(|p| p.holds_target()))
    };
    if !allies.iter().any(|&h| allies.iter().any(|&o| o != h && reaches(&b, h, o))) {
        let support: Vec<String> = w(&b).cat.skills.iter().filter(|s| s.kind == "SUPPORT" && !s.system && ri::skill_intent(s) == "support").map(|s| s.id.clone()).collect();
        for id in support {
            let u = &mut b.host.session.as_mut().unwrap().w.units[allies[0]];
            u.meter_skills.push(id.clone());
            u.known_skills.push(id);
            b.after_change(&cx);
            if reaches(&b, allies[0], allies[1]) {
                break;
            }
            let u = &mut b.host.session.as_mut().unwrap().w.units[allies[0]];
            u.meter_skills.pop();
            u.known_skills.pop();
        }
    }
    let pair = allies.iter().flat_map(|&h| allies.iter().map(move |&o| (h, o))).filter(|(h, o)| h != o).find(|&(h, o)| {
        let s = b.host.session.as_ref().unwrap();
        rip::pick_options(&s.w, &s.plan, h, RoundTarget::Unit(o)).iter().any(|r| r.problem.is_none() && r.picks.iter().any(|p| p.holds_target()))
    });
    let (h, o) = pair.expect("a member with a support row on another member");
    b.selected = Some(h);
    assert!(rip::picks_of(w(&b), h).is_empty(), "only the ghost");
    cx.held.ctrl = true;
    b.run(&mut cx, Action::UnitClick(o));
    {
        assert_eq!(b.selected, Some(h), "Ctrl+click keeps the hero: {}", b.message);
        let p = b.popup.as_ref().expect("the support menu");
        assert_eq!(p.menu, Some((h, RoundTarget::Unit(o))));
        assert!(p.rows.last().is_some_and(|r| matches!(r.act, RowAct::SelectAlly(x) if x == o)), "Select <ally> last");
    }
    // a plain click still selects
    cx.held.ctrl = false;
    b.run(&mut cx, Action::ClosePopup);
    b.selected = Some(h);
    if rip::picks_of(w(&b), h).iter().all(|p| !p.holds_target()) {
        b.run(&mut cx, Action::UnitClick(o));
        assert_eq!(b.selected, Some(o));
    }
}

/// Key F accepts the selected hero's whole ghost as picks: the same plan entries, none ghost; F again says there is
/// nothing auto-filled.
#[test]
fn f_fills_the_timeline_with_the_ghost() {
    let (mut cx, mut g, mut b) = planning();
    let h = hero(&b);
    let before = rows_of(&b, h);
    assert!(!before.is_empty() && ghosts_of(&b, h).iter().all(|g| *g));
    b.key(&mut cx, Key::Char('f'));
    assert!(!rip::picks_of(w(&b), h).is_empty(), "{}", b.message);
    assert!(ghosts_of(&b, h).iter().all(|g| !*g), "nothing ghost after F");
    assert_eq!(rows_of(&b, h), before, "the same plan, solid");
    b.key(&mut cx, Key::Char('f'));
    assert!(b.message.ends_with("Nothing auto-filled to accept."), "{}", b.message);
    drawn(&mut cx, &mut g, &mut b);
    assert!(b.w.iter().filter(|x| x.id.starts_with("strip.block")).all(|x| !matches!(x.action, Some(Action::GhostSlot(_)))), "no ghost block left to click");
}

/// A click on a strip block showing a ghost entry makes that command a pick (the ghost up to it accepted; the fill
/// after it stays ghost).
#[test]
fn a_click_on_a_ghost_block_picks_its_command() {
    let (mut cx, mut g, mut b) = planning();
    let h = hero(&b);
    let before = rows_of(&b, h);
    let slots: Vec<(String, usize)> = b.w.iter().filter_map(|x| match x.action {
        Some(Action::GhostSlot(k)) if x.id.starts_with("strip.block") => Some((x.id.clone(), k)),
        _ => None,
    }).collect();
    assert!(!slots.is_empty(), "the ghost blocks are clickable");
    let k = slots.iter().map(|s| s.1).max().unwrap().min(1);
    let id = slots.iter().find(|s| s.1 == k).unwrap().0.clone();
    let r = widget(&b, &id).rect;
    assert!(tip_has(widget(&b, &id), "accept the auto-fill up to here"));
    let a = aegis_ui::ui::hit(&b.w, (r.x + 2, r.y + r.h / 2)).expect("the block takes the click");
    assert_eq!(a, Action::GhostSlot(k));
    b.run(&mut cx, a);
    let ghosts = ghosts_of(&b, h);
    assert!(ghosts[..=k].iter().all(|g| !*g), "entries up to {k} are picks: {ghosts:?} {}", b.message);
    assert!(ghosts[k + 1..].iter().all(|g| *g), "the fill after it stays ghost: {ghosts:?}");
    assert_eq!(rows_of(&b, h)[..=k], before[..=k], "the picked commands keep their place");
    drawn(&mut cx, &mut g, &mut b);
    assert!(!matches!(aegis_ui::ui::hit(&b.w, (r.x + 2, r.y + r.h / 2)), Some(Action::GhostSlot(_))), "a solid block is no ghost slot");
}

/// The Help card lists the party keys, the Shift skill keys, F and Ctrl+click.
#[test]
fn the_help_keys_name_the_input2_keys() {
    let battle = crate::menus::CONTROLS.iter().find(|(g, _)| *g == "Battle").expect("the battle keys").1;
    for (what, key) in [("Select party member 1-4", "1-4"), ("Skills of the selected hero (timeline)", "Shift+1-9"), ("Accept the selected hero's auto-fill (timeline)", "F"), ("Support an ally while only the auto-fill shows (timeline)", "Ctrl+click")] {
        assert!(battle.iter().any(|r| r.0 == what && r.1 == key), "{what}: {key} in {battle:?}");
    }
}

// ---- INPUT1-AP: the AP pip gauge (INPUT1 AP3, AP5-AP10) ---------------------------------------------------------

/// The agent link's `ap_gauge` (null: hidden).
fn gauge(b: &BattleScreen, cx: &Cx) -> Value {
    b.state(cx)["ap_gauge"].clone()
}

/// AP6 and INPUT2 (style D) written out from the kit's plan and forecast: `u = ceil(max / 12)` AP a slot above 12;
/// each plan entry that costs AP is a block (`B<n>`, its slots from the running total, rounded up), free sockets
/// (`O`) follow up to the AP held, crossed slots (`X`) up to the max; a debt (`now < 0`): every slot crossed.
fn pips_by_rule(b: &BattleScreen, unit: usize) -> String {
    let f = forecast(b, unit);
    let s = b.host.session.as_ref().unwrap();
    let aps: Vec<i32> = s.plan.compile(&s.w, unit).entries.iter().map(|e| e.total_ap()).filter(|ap| *ap > 0).collect();
    let u = if f.max > AP_PIPS { (f.max + AP_PIPS - 1) / AP_PIPS } else { 1 };
    let c = |n: i32| if n <= 0 { 0 } else { (n + u - 1) / u };
    let slots = c(f.max);
    if f.now < 0 {
        return vec!["X"; slots as usize].join(" ");
    }
    let mut out: Vec<String> = Vec::new();
    let (mut sum, mut k) = (0, 0);
    for ap in aps {
        sum += ap;
        let end = c(sum).min(slots) as usize;
        if end > out.len() {
            k += 1;
        }
        while out.len() < end {
            out.push(format!("B{k}"));
        }
    }
    let held = c(f.now).min(slots) as usize;
    while (out.len() as i32) < slots {
        out.push(if out.len() < held { "O".into() } else { "X".into() });
    }
    out.join(" ")
}

fn forecast(b: &BattleScreen, u: usize) -> rip::ApForecast {
    let s = b.host.session.as_ref().unwrap();
    rip::ap_forecast(&s.w, &s.plan, u)
}

fn set_ap(cx: &Cx, b: &mut BattleScreen, u: usize, ap: i32, max: Option<i32>) {
    let unit = &mut b.host.session.as_mut().unwrap().w.units[u];
    unit.ap = ap;
    if let Some(m) = max {
        unit.max_ap = m;
    }
    b.after_change(cx);
}

/// AP5-AP7, AP9: while planning the selected party member's gauge (`strip.ap`) reads its kit forecast by AP7's rule,
/// its hover card holds AP9's lines; with no hero selected it is hidden.
#[test]
fn the_gauge_reads_the_kit_forecast_while_planning() {
    let (mut cx, mut g, mut b) = planning();
    let h = hero(&b);
    let f = forecast(&b, h);
    let st = gauge(&b, &cx);
    assert_eq!(st["pips"], json!(pips_by_rule(&b, h)), "{f:?}");
    assert_eq!((st["unit"].clone(), st["red"].clone(), st["label"].clone()), (json!(h), json!(false), Value::Null));
    let wg = widget(&b, "strip.ap");
    assert_eq!(wg.text, pips_by_rule(&b, h));
    for l in ap_lines(&f) {
        assert!(tip_has(wg, &l), "AP9 line {l} in {:?}", wg.tip);
    }
    // a pick changes the gauge at once (AP2)
    let foe = foes_on_field(&b)[0];
    next_to(&cx, &mut b, foe);
    b.run(&mut cx, Action::UnitClick(foe));
    let k = first_valid_pick(&b);
    b.run(&mut cx, Action::Choose(k));
    b.run(&mut cx, Action::ClosePopup);
    drawn(&mut cx, &mut g, &mut b);
    assert_eq!(gauge(&b, &cx)["pips"], json!(pips_by_rule(&b, h)));
    b.selected = None;
    drawn(&mut cx, &mut g, &mut b);
    assert_eq!(gauge(&b, &cx), Value::Null, "no hero selected: hidden");
    assert!(b.w.iter().all(|x| x.id != "strip.ap"));
}

/// AP5: a classic battle draws no gauge and its state has no `ap_gauge` key.
#[test]
fn classic_draws_no_gauge() {
    let (mut cx, mut g, mut b) = arena(None, "", false);
    b.tick(&mut cx);
    drawn(&mut cx, &mut g, &mut b);
    assert!(b.w.iter().all(|x| x.id != "strip.ap"));
    assert!(b.state(&cx).get("ap_gauge").is_none());
}

/// The hero next to a foe with the skill menu open on it and a skill row it can pay at full AP; the row's index
/// and AP (the kit option's).
fn payable_row(cx: &mut Cx, b: &mut BattleScreen) -> (usize, usize, i32) {
    let (h, f) = (hero(b), foes_on_field(b)[0]);
    next_to(cx, b, f);
    b.run(cx, Action::UnitClick(f));
    let s = b.host.session.as_ref().unwrap();
    let opts = rip::pick_options(&s.w, &s.plan, h, RoundTarget::Unit(f));
    let k = opts.iter().position(|o| o.problem.is_none() && o.ap >= 1 && matches!(o.picks[..], [rip::Pick::Skill { .. }])).expect("a payable skill row with an AP cost");
    let row = b.popup.as_ref().expect("the menu").rows.iter().position(|r| matches!(&r.act, RowAct::Pick(p) if *p == opts[k].picks)).expect("the option's cell");
    (f, row, opts[k].ap)
}

/// AP3, AP8: hovering a row the hero cannot pay (its AP one short of the row's) turns the gauge red with "-1" after
/// the pips; leaving it restores the gauge.
#[test]
fn an_unpayable_row_turns_the_gauge_red_with_its_shortfall() {
    let (mut cx, mut g, mut b) = planning();
    let h = hero(&b);
    let (f, k, cost) = payable_row(&mut cx, &mut b);
    b.run(&mut cx, Action::ClosePopup);
    set_ap(&cx, &mut b, h, cost - 1, None);
    b.run(&mut cx, Action::UnitClick(f));
    let row = b.popup.as_ref().expect("the menu").rows[k].clone();
    assert!(!row.valid && row.detail.starts_with("AP "), "the row is refused for its AP: {}", row.detail);
    assert_eq!(row.ap_short, Some(1), "the kit's shortfall: the cost less the AP free");
    drawn(&mut cx, &mut g, &mut b);
    let r = widget(&b, &format!("popup.{k}")).rect;
    b.draw(&mut cx, &mut g, (r.x + r.w / 2, r.y + r.h / 2), true);
    let st = gauge(&b, &cx);
    assert_eq!((st["red"].clone(), st["label"].clone(), st["label_side"].clone()), (json!(true), json!("-1"), json!("after")), "{st}");
    assert_eq!(st["pips"], json!(pips_by_rule(&b, h)), "the alert changes colours only");
    drawn(&mut cx, &mut g, &mut b);
    let st = gauge(&b, &cx);
    assert_eq!((st["red"].clone(), st["label"].clone()), (json!(false), Value::Null), "leaving the row clears both");
}

/// AP8: a hero at -2 AP shows "-2" before its slots, every slot crossed (INPUT2).
#[test]
fn a_debt_shows_before_the_slots() {
    let (mut cx, mut g, mut b) = planning();
    let h = hero(&b);
    set_ap(&cx, &mut b, h, -2, None);
    drawn(&mut cx, &mut g, &mut b);
    let st = gauge(&b, &cx);
    let slots = st["slots"].as_i64().unwrap() as usize;
    assert_eq!((st["label"].clone(), st["label_side"].clone()), (json!("-2"), json!("before")));
    assert_eq!(st["pips"], json!(vec!["X"; slots].join(" ")));
    assert_eq!(st["pips"], json!(pips_by_rule(&b, h)));
}

/// AP6, AP9, IS3: a 14 AP hero shows 7 pips at 2 AP each; the hover card and the inspect panel say "1 pip = 2 AP".
#[test]
fn above_twelve_ap_a_pip_holds_two() {
    let (mut cx, mut g, mut b) = planning();
    let h = hero(&b);
    set_ap(&cx, &mut b, h, 14, Some(14));
    drawn(&mut cx, &mut g, &mut b);
    let st = gauge(&b, &cx);
    assert_eq!((st["slots"].clone(), st["per_pip"].clone()), (json!(7), json!(2)));
    assert_eq!(st["pips"], json!(pips_by_rule(&b, h)));
    assert!(tip_has(widget(&b, "strip.ap"), "1 pip = 2 AP"));
    let s = b.host.session.as_ref().unwrap();
    let lines: Vec<String> = inspect_tip(s, h, b.selected, &s.enemy_intents(), &s.round_intents(), &[], true).into_iter().map(|l| l.0).collect();
    assert!(lines.contains(&"1 pip = 2 AP".to_string()), "IS3: {lines:?}");
    set_ap(&cx, &mut b, h, 10, Some(10));
    assert!(!ap_lines(&forecast(&b, h)).iter().any(|l| l.starts_with("1 pip")), "at most 12 AP: no pip line");
}

/// AP10: in playback the gauge shows the live AP full and what the hero paid this round hollow (one pip an AP); a
/// payment turns its pips hollow over 150 ms (the frame it shows starts the change), at once with skip animations;
/// the next planning reads the forecast again.
#[test]
fn playback_drains_the_paid_pips() {
    let (mut cx, mut g, mut b) = planning();
    let h = hero(&b);
    let start = w(&b).units[h].ap;
    b.send(&cx, json!({"cmd": "resolve"}));
    assert_eq!(b.session().phase, Phase::Replay, "{}", b.message);
    drawn(&mut cx, &mut g, &mut b);
    let live = w(&b).units[h].ap;
    let max = w(&b).units[h].max_ap;
    let paid: i32 = b.session().resolver.scheduled.iter().filter(|a| a.unit == h && !a.cancelled).map(|a| a.command.ap_cost + a.command.stance_switch_cost).sum();
    let st = gauge(&b, &cx);
    assert_eq!(st["pips"], json!(aegis_ui::pips::PipGauge::live(start, live, AP_PAID, max, AP_PIPS).reading()));
    let read = st["pips"].as_str().unwrap().to_string();
    let count = |code: &str| read.split(' ').filter(|p| *p == code).count() as i32;
    if max <= AP_PIPS && live >= 0 && live <= start {
        assert_eq!((count("O"), count("B1")), (live, start - live), "sockets hold the live AP, one block what was paid: {read}");
        if paid > 0 && live == start - paid {
            assert_eq!(count("B1"), paid, "a command paying {paid} AP fills {paid} slots of the paid block");
        }
    }
    // a payment shown mid-playback: the frame that sees it starts the change, 150 ms later it is done
    let ap = w(&b).units[h].ap;
    let (pay_from, pay_to) = if ap >= 2 { (ap, ap - 2) } else { (ap, ap) };
    set_ap(&cx, &mut b, h, pay_to, None);
    drawn(&mut cx, &mut g, &mut b);
    if pay_from != pay_to {
        assert_eq!(gauge(&b, &cx)["pips"], json!(read), "the change starts from the shown pips");
        let a = b.ap_anim.as_mut().unwrap();
        a.at = a.at.checked_sub(Duration::from_millis(200)).unwrap_or(a.at);
        drawn(&mut cx, &mut g, &mut b);
    }
    let done = aegis_ui::pips::PipGauge::live(start, pay_to, AP_PAID, max, AP_PIPS).reading();
    assert_eq!(gauge(&b, &cx)["pips"], json!(done), "after 150 ms");
    // skip animations: at once
    cx.settings.v["skip_anim"] = json!(true);
    set_ap(&cx, &mut b, h, pay_to.max(1) - 1, None);
    drawn(&mut cx, &mut g, &mut b);
    assert_eq!(gauge(&b, &cx)["pips"], json!(aegis_ui::pips::PipGauge::live(start, pay_to.max(1) - 1, AP_PAID, max, AP_PIPS).reading()), "skip animations: at once");
    // the next planning: the forecast again
    b.run(&mut cx, Action::NextRound);
    b.tick(&mut cx);
    drawn(&mut cx, &mut g, &mut b);
    if b.planning() && w(&b).units[h].is_active() {
        assert_eq!(gauge(&b, &cx)["pips"], json!(pips_by_rule(&b, h)));
    }
}

/// A frame at text scale `scale` with the mouse at `mouse` (the text records kept).
fn frame_at(b: &mut BattleScreen, cx: &mut Cx, g: &mut Gfx, scale: f32, mouse: (i32, i32)) -> Vec<aegis_ui::painter::TextRec> {
    g.fonts.scale = scale;
    g.fonts.audit = Some(Vec::new());
    b.draw(cx, g, mouse, true);
    g.fonts.audit.take().unwrap_or_default()
}

/// AP6, TEXTFIT1: at 640x360 and the four text sizes the gauge sits inside the strip, after the last block and
/// before the tactic menus, with its label (the shortfall's and the debt's), and no strip text spills or is cut
/// (the blocks' span labels cut by design).
#[test]
fn the_strip_with_the_gauge_fits_at_every_text_size() {
    let (mut cx, mut g, mut b) = planning();
    let h = hero(&b);
    let (f, k, cost) = payable_row(&mut cx, &mut b);
    b.run(&mut cx, Action::ClosePopup);
    let full = w(&b).units[h].ap;
    for (state, ap) in [("plain", full), ("red", cost - 1), ("debt", -2)] {
        set_ap(&cx, &mut b, h, ap, None);
        b.popup = None;
        if state == "red" {
            b.run(&mut cx, Action::UnitClick(f));
        }
        for (_, scale) in aegis_ui::screen::TEXT_SIZES {
            let scale = scale as f32;
            frame_at(&mut b, &mut cx, &mut g, scale, (-500, -500));
            let mouse = match (state, b.w.iter().find(|x| x.id == format!("popup.{k}"))) {
                ("red", Some(r)) => (r.rect.x + r.rect.w / 2, r.rect.y + r.rect.h / 2),
                _ => (-500, -500),
            };
            let recs = frame_at(&mut b, &mut cx, &mut g, scale, mouse);
            let st = gauge(&b, &cx);
            assert_eq!(st["label"].is_null(), state == "plain", "{state} scale {scale}: {st}");
            let ga = widget(&b, "strip.ap").rect;
            let strip = Rect::new(2, 2, W - 4, 24);
            assert!(ga.x >= strip.x && ga.y >= strip.y && ga.x + ga.w <= strip.x + strip.w && ga.y + ga.h <= strip.y + strip.h, "{state} scale {scale}: the gauge {ga:?} in the strip");
            let last = (1..=b.session().def.total_blocks).filter_map(|n| b.w.iter().find(|x| x.id == format!("strip.block{n}"))).map(|x| x.rect.x + x.rect.w).max().unwrap();
            let party = widget(&b, "strip.party").rect;
            assert!(ga.x >= last && ga.x + ga.w <= party.x, "{state} scale {scale}: blocks end {last}, gauge {ga:?}, party menu {party:?}");
            let confirm = widget(&b, "strip.confirm").rect;
            assert!(confirm.x + confirm.w <= W - 4, "{state} scale {scale}: the controls stay on the strip");
            let boxes: Vec<(String, f32, f32, f32, f32)> = b.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            let widget_at = |x: f32, y: f32| boxes.iter().filter(|b| x >= b.1 && y >= b.2 && x < b.1 + b.3 && y < b.2 + b.4).min_by(|a, b| (a.3 * a.4).total_cmp(&(b.3 * b.4))).map(|b| b.0.clone());
            let on_strip = |e: &Value| {
                let (x, y) = (e["at"][0].as_f64().unwrap_or(-1.0) as f32 + 0.5, e["at"][1].as_f64().unwrap_or(-1.0) as f32 + 0.5);
                y < 26.0 && widget_at(x, y).is_none_or(|id| !id.starts_with("strip.block"))
            };
            // the open menu's own texts (it may open over the strip) are the menu's audit
            let menu = b.popup.as_ref().map(|p| p.rect(b.menu_row));
            let off_menu = |e: &Value| menu.is_none_or(|m| !m.contains(e["at"][0].as_f64().unwrap_or(-1.0) as i32 + 1, e["at"][1].as_f64().unwrap_or(-1.0) as i32 + 1));
            let ours: Vec<Value> = v["spill"].as_array().unwrap().iter().filter(|e| e["widget"].as_str().is_some_and(|w| w.starts_with("strip."))).chain(v["cut"].as_array().unwrap().iter().filter(|e| on_strip(e))).filter(|e| off_menu(e)).cloned().collect();
            assert!(ours.is_empty(), "{state} scale {scale}: {ours:?}");
        }
    }
}

/// User 2026-10-29: an Arena fight beyond classic shows the Scene Runner card's ruleset chip (same text: the
/// ruleset's name, its switches, the reveal); a classic fight draws neither the chip nor any intent control and its
/// state keeps its shape.
#[test]
fn the_arena_chip_shows_beyond_classic_and_classic_is_unchanged() {
    let timeline = Ruleset::of(&crate::hud::hud_tests::game_cx().data.cat, TIMELINE).expect("the catalog's timeline row");
    let off = timeline.rules.iter().copied().find(|id| rule(id).is_some_and(|r| r.kind == RuleKind::Extension) && *id != PLAN_BADGES).expect("an extension besides plan_badges");
    let spec = format!("-{off}");
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE), &spec, true);
    drawn(&mut cx, &mut g, &mut b);
    let s = b.host.session.as_ref().unwrap();
    let want = crate::runner::chip_text(&timeline.name, &spec, "", s.w.ruleset.has(PLAN_BADGES));
    assert_eq!(widget(&b, "bar.chip").text, want);
    assert_eq!(b.state(&cx)["ruleset"]["chip"], json!(want));
    assert!(want.contains(&format!("-{off}")));
    let (mut cx, mut g, mut b) = arena(None, "", false);
    drawn(&mut cx, &mut g, &mut b);
    assert!(w(&b).ruleset.is_classic());
    assert!(b.w.iter().all(|x| !["hero.self", "bar.chip"].contains(&x.id.as_str())), "classic draws no round control");
    let st = b.state(&cx);
    assert!(st.get("picks").is_none() && st.get("ruleset").is_none(), "a classic battle's state keeps its shape");
}
