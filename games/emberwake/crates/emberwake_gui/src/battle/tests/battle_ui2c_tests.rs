//! BATTLE-UI2 part 3 (PR7-40, `games/emberwake/docs/specs/SCREEN_CONTRACT_BATTLE.md` 7 and 8, FB-358): the row skill
//! menu opens on a strip block (a click on an empty or a picked block, a right click or the menu key on a ghost block,
//! whose click still accepts the ghost) and its rows carry the kit's cost and hits on the target; an open foe group
//! lists its members once in the text view; the board's ground hazards show their icon and rounds left, a push its
//! dashed arrow to a landing tile on the hazard plates' rect in both views; a planned heal shows "+n" on its plate.
//! Battles are the first catalog encounter, edited in memory (no shipped data changes); every number is the kit's.

use crate::battle::input::RowAct;
use crate::battle::marks::ground_plate;
use crate::battle::{Action, BattleScreen};
use crate::screens::{Cx, Screen};
use crate::stage::{Geom, View};
use aegis_ui::art::Art;
use aegis_ui::controls::{Input, Pad};
use aegis_ui::screen::Key;
use aegis_ui::ui::Gfx;
use emberwake_battle::grid::{V2, GRID_H, GRID_W};
use emberwake_battle::ground::Placed;
use emberwake_battle::round_intent::RoundTarget;
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::Session;
use emberwake_data::HazardDef;
use emberwake_rules::ruleset::TIMELINE;
use serde_json::{json, Value};

/// An Arena battle of the first catalog encounter under the timeline with `rules`; `foes` copies of its first enemy
/// when set; the game's art on (the status icons); a hero selected.
fn arena(foes: Option<usize>, rules: &str, view: &str) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.settings.v["battle_view"] = json!(view);
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    if let Some(n) = foes {
        std::sync::Arc::get_mut(&mut cx.app.data).expect("the test owns the data").edit_cat(|c| {
            let row = c.encounters.get_mut(&encounter).expect("the row");
            let first = row["enemies"][0].clone();
            row.insert("enemies".into(), json!(vec![first; n]));
        });
    }
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), rules, false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    let mut g = crate::hud::hud_tests::gfx();
    b.tick(&mut cx);
    if b.selected.is_none() {
        b.selected = b.session().w.allies.first().copied();
    }
    b.draw(&mut cx, &mut g, (-500, -500), true);
    (cx, g, b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

/// The strip block widgets with their action, by block number.
fn blocks(b: &BattleScreen) -> Vec<(i32, aegis_ui::canvas::Rect, Option<Action>)> {
    b.w.iter().filter_map(|w| w.id.strip_prefix("strip.block").and_then(|n| n.parse().ok()).map(|n| (n, w.rect, w.action.clone()))).collect()
}

fn centre(r: aegis_ui::canvas::Rect) -> (i32, i32) {
    (r.x + r.w / 2, r.y + r.h / 2)
}

/// The menu opened on strip block `b`: the selected hero's, under the block, titled with the block the next pick takes.
fn assert_menu_on(cx: &mut Cx, g: &mut Gfx, sb: &mut BattleScreen, b: i32) {
    let mouse = sb.mouse;
    sb.draw(cx, g, mouse, true);
    let sb = &*sb;
    let p = sb.popup.as_ref().expect("the skill menu opens");
    let h = sb.selected.unwrap();
    assert_eq!(p.menu.map(|m| m.0), Some(h), "the selected hero's menu");
    assert_eq!(p.block.map(|x| x.0), Some(b), "on block {b}");
    let br = p.block.unwrap().1;
    let r = p.rect(sb.menu_row);
    assert!(r.y > br.y + br.h, "under the block: {r:?} {br:?}");
    // titled with the block the next pick takes (a target out of every skill's reach says so instead)
    let next = rip::next_pick_block(&s(sb).w, &s(sb).plan, h);
    let live = p.rows.iter().any(|r| r.valid && matches!(r.act, RowAct::Pick(_)));
    assert!(if live { p.title.starts_with(&format!("Block {next} vs ")) } else { p.title.starts_with("No skill reaches") }, "{}", p.title);
}

/// Contract 7, FB-358: a ghost block's click still accepts the ghost; its right click and Tab (and the pad's north
/// button) open the skill menu on it, the key again closes it; once picked, a click on a block opens the menu there.
#[test]
fn the_menu_opens_on_a_strip_block() {
    let (mut cx, mut g, mut b) = arena(None, "", "flat");
    let all = blocks(&b);
    let (gb, gr, act) = all.iter().find(|x| matches!(x.2, Some(Action::GhostSlot(_)))).cloned().expect("a ghost block under the auto-fill");
    assert!(matches!(act, Some(Action::GhostSlot(_))), "a click accepts the ghost");
    // the right click on it
    b.mouse = centre(gr);
    b.back(&mut cx);
    assert_menu_on(&mut cx, &mut g, &mut b, gb);
    assert!(b.w.iter().any(|w| w.id == "popup.panel"), "the menu draws");
    // the menu key closes it and opens it again, from the keyboard and the pad
    assert!(b.capture(&mut cx, Input::Key(Key::Tab)), "Tab is the menu key while planning");
    assert!(b.popup.is_none(), "again: it closes");
    assert!(b.capture(&mut cx, Input::Pad(Pad::North)));
    assert_menu_on(&mut cx, &mut g, &mut b, gb);
    b.popup = None;
    // with the mouse off the strip the key opens it on the block the next pick takes
    b.mouse = (-500, -500);
    assert!(b.capture(&mut cx, Input::Key(Key::Tab)));
    let next = rip::next_pick_block(&s(&b).w, &s(&b).plan, b.selected.unwrap());
    assert_menu_on(&mut cx, &mut g, &mut b, next);
    b.popup = None;
    // the whole ghost taken as picks: every planned block opens the menu on a click
    b.act(&mut cx, Action::FillGhost);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let picked: Vec<_> = blocks(&b).into_iter().filter(|x| matches!(x.2, Some(Action::StripMenu(_)))).collect();
    assert!(!picked.is_empty(), "the picked blocks open the menu: {:?}", blocks(&b));
    assert!(blocks(&b).iter().all(|x| !matches!(x.2, Some(Action::GhostSlot(_)))), "no ghost left");
    let (pb, pr, _) = picked[0].clone();
    b.mouse = centre(pr);
    b.act(&mut cx, Action::StripMenu(pb));
    assert_menu_on(&mut cx, &mut g, &mut b, pb);
    // outside planning neither opens it, and Tab moves the focus as before
    b.popup = None;
    b.host.session.as_mut().unwrap().plan_confirmed = true;
    assert!(!b.capture(&mut cx, Input::Key(Key::Tab)));
    b.strip_menu(pb);
    assert!(b.popup.is_none());
}

/// Contract 7 with CONTROLS-UI1: the menu key is whatever the player's map turns into Tab, not the raw Tab key or pad
/// north; a swap of Tab and pad north with Pad X moves the menu key with it.
#[test]
fn the_menu_key_follows_a_rebind() {
    let (mut cx, _g, mut b) = arena(None, "", "flat");
    let (mut map, _) = aegis_ui::controls::Controls::from_settings(&cx.settings.v);
    map.bind_pad(Key::Tab, Pad::West);
    cx.settings.v[aegis_ui::controls::SETTINGS_KEY] = map.to_json();
    b.mouse = (-500, -500);
    assert!(!b.capture(&mut cx, Input::Pad(Pad::North)), "pad north no longer gives Tab");
    assert!(b.capture(&mut cx, Input::Pad(Pad::West)), "Pad X now gives Tab and opens the menu");
    b.popup = None;
    assert!(b.capture(&mut cx, Input::Key(Key::Tab)), "the Tab key still gives Tab");
}

/// Contract 7: each skill row's facts are the kit option's AP and blocks and the kit's `pick_hits` on the foe; the
/// panel draws the damage of a row that lands one.
#[test]
fn the_rows_carry_the_kits_cost_and_hits() {
    let (mut cx, mut g, mut b) = arena(None, "", "flat");
    let h = b.selected.unwrap();
    let foe = s(&b).w.enemies.iter().copied().find(|&e| s(&b).w.units[e].is_active() && s(&b).w.has(e)).expect("a foe");
    b.act(&mut cx, Action::UnitClick(foe));
    let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
    let p = b.popup.clone().expect("the menu");
    let sess = s(&b);
    let opts = rip::menu_options(&sess.w, &sess.plan, h, RoundTarget::Unit(foe));
    let mut landed = 0;
    for (k, row) in p.rows.iter().enumerate() {
        let RowAct::Pick(picks) = &row.act else { continue };
        // a skill row (the Off-Hand button is a pick drawn as a button)
        if !matches!(row.slot, Some(crate::battle::skill_menu::Slot::Intent(_))) {
            continue;
        }
        let Some(f) = row.facts.as_ref() else { continue };
        let at = row.at.unwrap();
        let o = opts.iter().find(|o| o.picks == *picks && o.at == at).expect("the kit's option");
        assert_eq!((f.ap, f.blocks), (o.ap, o.blocks), "row {k}: the kit's cost");
        let want = match at {
            RoundTarget::Unit(u) if o.problem.is_none() && sess.w.enemies.contains(&u) => rip::pick_hits(&sess.w, &sess.plan, &sess.resolver.scheduled, h, at, picks, u),
            _ => None,
        };
        assert_eq!(f.hits, want, "row {k}: the kit's hits");
        if let Some(hit) = f.hits.filter(|x| x.hits > 0 && row.valid) {
            landed += 1;
            let r = p.row(b.menu_row, k);
            let text = format!("-{}", hit.damage);
            assert!(recs.iter().any(|t| t.text == text && t.y >= r.y as f32 - 1.0 && t.y < (r.y + r.h) as f32), "row {k} draws {text}");
        }
    }
    assert!(landed > 0, "a row lands a hit on the foe");
    assert!(recs.iter().any(|t| t.text == crate::battle::skill_menu::MENU_HINT), "the footer's hint");
}

/// FB-358: an open foe group is its header alone in the text view; each member is listed once, in its own row.
#[test]
fn an_open_group_lists_its_members_once() {
    let n = crate::battle_view::GROUP_FROM + 1;
    let (mut cx, mut g, mut b) = arena(Some(n), "", "flat");
    let key = b.w.iter().find_map(|w| match &w.action {
        Some(Action::FoeGroup(k)) => Some(k.clone()),
        _ => None,
    });
    let key = key.expect("a group row");
    let ids = |b: &BattleScreen| -> Vec<String> {
        b.view_json["fields"][crate::battle_view::FOES]["rows"].as_array().expect("the foe rows").iter().flat_map(|r| r["id"].as_str().unwrap_or("").split(',').map(String::from).collect::<Vec<_>>()).collect()
    };
    let closed = ids(&b);
    let mut uniq = closed.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(uniq.len(), closed.len(), "closed: each foe once");
    b.act(&mut cx, Action::FoeGroup(key.clone()));
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let open = ids(&b);
    let rows = b.view_json["fields"][crate::battle_view::FOES]["rows"].as_array().unwrap().clone();
    let header = rows.iter().find(|r| r["id"] == Value::String(key.clone())).expect("the open group's header row");
    assert_eq!(header["detail"], json!(""), "the header lists no member");
    let members: Vec<String> = open.iter().filter(|x| **x != key).cloned().collect();
    let mut once = members.clone();
    once.sort();
    once.dedup();
    assert_eq!(once.len(), members.len(), "open: each member once: {open:?}");
    let mut was = closed.clone();
    was.sort();
    assert_eq!(once, was, "the same foes either way");
}

/// Free walkable cells, left to right, top to bottom.
fn free_cells(s: &Session) -> Vec<V2> {
    let w = &s.w;
    (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).filter(|c| w.grid.is_valid(*c) && w.grid.is_walkable(*c) && w.occupant(*c).is_none() && !w.grid.is_exit(*c)).collect()
}

/// Contract 8: a ground hazard's plate shows its status icon and, right of it, its rounds left (the last round less
/// the round now, plus one), in both views.
#[test]
fn a_hazard_shows_its_icon_and_rounds_left() {
    for view in ["flat", "stage"] {
        let (mut cx, mut g, mut b) = arena(None, "+ground_hazards", view);
        cx.settings.v["battle_overlays"] = json!({"test_hazards": true});
        let sess = b.host.session.as_mut().unwrap();
        let round = sess.current_round;
        let (hero, cell) = (sess.w.allies[0], free_cells(sess)[0]);
        let row = json!({"id": "t_mire", "name": "Test Mire", "on": "enter", "damage": 5, "status": "root", "potency": 60, "affects": "foes", "duration": 3, "tint": "#40c0ff", "glyph": "M"});
        let last = round + 2;
        sess.w.grid.ground.insert(cell, Placed { def: HazardDef::from_obj(row.as_object().unwrap()), by: hero, round, last_round: last });
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
        let plate = ground_plate(&Geom::new(View::of_setting(&json!(view)), &cx.tuning.value), cell);
        let left = (last - round + 1).to_string();
        let side = (plate.h / 3).clamp(5, 8);
        let t = recs.iter().find(|t| t.text == left && t.x >= plate.x as f32 && t.x < (plate.x + plate.w) as f32 && t.y >= plate.y as f32 - 1.0 && t.y < (plate.y + plate.h) as f32).unwrap_or_else(|| panic!("{view}: the rounds left on the plate"));
        assert!(t.x >= (plate.x + 2 + side) as f32, "{view}: the number right of the icon (the icon drew): {} vs {plate:?}", t.x);
        assert!(!recs.iter().any(|t| t.text == "M" && t.x >= plate.x as f32 && t.x < (plate.x + plate.w) as f32 && t.y >= plate.y as f32 && t.y < (plate.y + plate.h) as f32), "{view}: the icon stands for the glyph");
    }
}

/// Contract 8 (and the review of part 2): a planned push draws a gold landing tile on the hazard plates' rect
/// (`ground_plate`) of its landing cell in both views, its edge dashed (gold and not gold along it).
#[test]
fn a_push_lands_on_the_plate_rect_in_both_views() {
    for view in ["flat", "stage"] {
        let (mut cx, mut g, mut b) = arena(None, "+forced_move", view);
        let (h, _) = super::push_forecast_tests::plan_push(&cx, &mut b);
        // a unit standing where the push would go steps aside to a free tile far off (the push then moves)
        for _ in 0..3 {
            let blockers: Vec<usize> = crate::battle::marks::push_forecasts(s(&b), h).into_iter().flatten().filter_map(|f| f.blocker).collect();
            let sess = b.host.session.as_mut().unwrap();
            for u in blockers {
                let far = *free_cells(sess).last().expect("a free tile");
                sess.w.grid.positions[u] = Some(far);
            }
        }
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let pushes: Vec<_> = crate::battle::marks::push_forecasts(s(&b), h).into_iter().flatten().filter(|f| !f.held && f.to != f.from).collect();
        let f = pushes.first().unwrap_or_else(|| panic!("{view}: a planned push moves its target: {:?}", crate::battle::marks::push_forecasts(s(&b), h)));
        let on = g.canvas.px.clone();
        cx.settings.v["battle_overlays"] = json!({"pushes": false});
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let off = g.canvas.px.clone();
        let land = ground_plate(&Geom::new(View::of_setting(&json!(view)), &cx.tuning.value), f.to);
        let w = g.canvas.w;
        let gold = aegis_ui::theme::th().gold;
        let px = |buf: &[u8], x: i32, y: i32| {
            let i = ((y * w + x) * 4) as usize;
            [buf[i], buf[i + 1], buf[i + 2]]
        };
        // the top edge: the pixels that changed with the layer are the theme's gold, and a dashed edge has gaps
        let changed: Vec<bool> = (land.x..land.x + land.w).map(|x| px(&on, x, land.y) != px(&off, x, land.y)).collect();
        assert!(changed.iter().filter(|c| **c).count() >= 3, "{view}: the landing tile's top edge draws on {land:?}");
        assert!(changed.iter().any(|c| !*c), "{view}: the edge is dashed");
        assert!((land.x..land.x + land.w).filter(|&x| px(&on, x, land.y) != px(&off, x, land.y)).all(|x| px(&on, x, land.y) == [gold[0], gold[1], gold[2]]), "{view}: in gold");
    }
}

/// Contract 6: a hero's planned heal on a wounded member shows "+n" on that member's plate, n the kit's `heal_on`
/// over the round as it will run.
#[test]
fn a_planned_heal_shows_plus_n_on_the_plate() {
    let (mut cx, mut g, mut b) = arena(None, "", "flat");
    let sess = b.host.session.as_mut().unwrap();
    let (h, ally) = (sess.w.allies[0], sess.w.allies[1]);
    let heal = sess.w.cat.skills.iter().enumerate().filter(|(_, k)| emberwake_battle::skill::heals(k) && k.kind == "SUPPORT" && k.round_intent == "support").min_by_key(|(_, k)| k.level).map(|(i, _)| i).expect("a heal skill in the catalog");
    let id = sess.w.cat.skills[heal].id.clone();
    let family = sess.w.cat.family_of(&id).to_string();
    sess.w.units[h].known_skills.push(id.clone());
    sess.w.units[h].active_extra_families.push(family);
    let max = sess.w.units[ally].max_hp;
    sess.w.units[ally].hp = (max / 3).max(1);
    let r = b.send(&cx, json!({"cmd": "pick", "unit": h, "target": ally, "pick": {"skill": id}}));
    assert_eq!(r["ok"], json!(true), "the heal is picked: {r}");
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let sess = s(&b);
    let want = emberwake_battle::intent::heal_on(&sess.w, &crate::battle_view::round_schedule(sess), ally);
    assert!(want > 0, "the heal restores HP");
    let label = b.w.iter().find(|w| w.id == format!("party.{ally}.heal")).expect("the heal label on the plate");
    assert_eq!(label.text, format!("+{want}"));
}

/// DIORAMA1 D3 (SCREEN_CONTRACT_BATTLE 8.1): a ground hazard whose status has a decal file (`burn`, `poison`, `slow`)
/// draws the decal instead of the tint plate (its tint no longer shows: two tints draw the same frame); a status
/// without one (`root`) keeps the tint (two tints differ), in both views. The icon and rounds left stay over either.
#[test]
fn a_hazard_decal_replaces_the_tint_and_the_tint_stays_without_one() {
    let art = |c: &mut Art, s: &str| crate::battle::marks::decal_art(c, s, V2::new(2, 3));
    let mut a = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    for s in ["burn", "poison", "slow"] {
        assert!(art(&mut a, s).is_some_and(|r| r.starts_with(&format!("props/hazards/{s}/v"))), "{s}");
    }
    assert_eq!((art(&mut a, "root"), art(&mut a, "")), (None, None));
    for view in ["flat", "stage"] {
        let shot = |status: &str, tint: &str| {
            let (mut cx, mut g, mut b) = arena(None, "+ground_hazards", view);
            cx.settings.v["battle_overlays"] = json!({"test_hazards": true});
            let sess = b.host.session.as_mut().unwrap();
            let round = sess.current_round;
            let (hero, cell) = (sess.w.allies[0], free_cells(sess)[0]);
            let row = json!({"id": "t_patch", "name": "Test Patch", "on": "enter", "damage": 5, "status": status, "potency": 60, "affects": "foes", "duration": 3, "tint": tint, "glyph": "P"});
            sess.w.grid.ground.insert(cell, Placed { def: HazardDef::from_obj(row.as_object().unwrap()), by: hero, round, last_round: round + 2 });
            crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
            assert_eq!(b.state(&cx)["hook_marks"]["ground"][0]["status"], json!(status), "{view}");
            g.canvas.px.clone()
        };
        assert!(shot("burn", "#40c0ff") == shot("burn", "#ff2020"), "{view}: the decal, not the tint");
        assert!(shot("root", "#40c0ff") != shot("root", "#ff2020"), "{view}: no decal file: the tint");
    }
}

/// PR7-64: the dev link's `debug_ground` puts a test hazard of a status on a free tile (the proof shots' decals) and
/// refuses a tile that is not walkable.
#[test]
fn the_dev_link_puts_a_ground_hazard() {
    let (cx, _g, mut b) = arena(None, "+ground_hazards", "stage");
    let cell = free_cells(s(&b))[0];
    let r = b.debug_ground(&cx, &json!({"cmd": "debug_ground", "at": [cell.x, cell.y], "status": "burn"}));
    assert_eq!(r, json!({"ok": true, "at": [cell.x, cell.y]}));
    let r = b.debug_ground(&cx, &json!({"cmd": "debug_ground", "free": 1, "status": "slow"}));
    assert_eq!(r["at"], json!([free_cells(s(&b))[1].x, free_cells(s(&b))[1].y]), "the second free tile");
    assert_eq!(s(&b).w.grid.ground.get(&cell).map(|p| p.def.status.as_str()), Some("burn"));
    assert_eq!(b.debug_ground(&cx, &json!({"cmd": "debug_ground", "at": [-1, 0], "status": "burn"}))["ok"], json!(false));
}

/// DIORAMA2 R7: a hazard's decal on the first or last column lies inside the stage's clip (the panel's gutters, past
/// the board), so its edge is not cut; and it draws there (the frame changes with it).
#[test]
fn an_edge_column_decal_is_not_clipped() {
    let (mut cx, mut g, mut b) = arena(None, "+ground_hazards", "stage");
    cx.settings.v["battle_overlays"] = json!({"test_hazards": true});
    let geo = Geom::new(View::Stage, &cx.tuning.value);
    let before = {
        crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
        g.canvas.px.clone()
    };
    let sess = b.host.session.as_mut().unwrap();
    let cell = free_cells(sess).into_iter().find(|c| c.x == 0 || c.x == GRID_W - 1).expect("a free edge tile");
    let plate = ground_plate(&geo, cell);
    let clip = crate::stage::STAGE_CLIP;
    assert!(clip.contains(plate.x - 2, plate.y) && clip.contains(plate.x + plate.w + 1, plate.y + plate.h - 1), "{plate:?} inside {clip:?} with room");
    let (by, round) = (sess.w.allies[0], sess.current_round);
    let row = json!({"id": "t_burn", "name": "Test Burn", "on": "enter", "damage": 5, "status": "burn", "potency": 60, "affects": "foes", "duration": 3, "tint": "#ff8000", "glyph": "B"});
    sess.w.grid.ground.insert(cell, Placed { def: HazardDef::from_obj(row.as_object().unwrap()), by, round, last_round: round + 2 });
    crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
    let w = g.canvas.w;
    let s = (w / 640).max(1);
    let changed = g.canvas.px.chunks(4).zip(before.chunks(4)).enumerate().filter(|(_, (a, b))| a != b).map(|(i, _)| ((i as i32) % w / s, (i as i32) / w / s)).filter(|&(x, y)| plate.contains(x, y)).count();
    assert!(changed > 0, "the decal draws on the edge tile");
}

/// DIORAMA3 R6, DIORAMA5 R6: the Show full grid setting (off by default, a Gameplay row) draws the grid lines; off,
/// none at rest or planning (the frame changes); on, stronger while planning than at rest.
#[test]
fn the_board_grid_setting_turns_the_grid_lines_off() {
    let shot = |on: bool| {
        let (mut cx, mut g, mut b) = arena(None, "", "stage");
        assert_eq!(cx.settings.v[crate::battle::BOARD_GRID], json!(false), "off by default");
        cx.settings.v[crate::battle::BOARD_GRID] = json!(on);
        crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, 1.0);
        g.canvas.px.clone()
    };
    assert!(shot(true) != shot(false), "the setting shows in the frame");
    let t = crate::tuning::Tuning::default();
    assert_eq!(t.grid_alpha, 0, "no grid at rest");
    assert!(t.grid_alpha_planning > crate::stage::FULL_GRID_ALPHA, "the full grid is stronger while planning ({})", t.grid_alpha_planning);
}
