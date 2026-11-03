//! VISCLAR1-B (decision "Battle view answers", mock C): a timeline battle's planned blocks show the skill's icon and name
//! in near-white with a dark outline, the first-move chips are gone, and classic keeps its strip and its chips. Fixtures
//! are the catalog's encounters and the kit's kind icons; expectations come from the rules, never shipped numbers.

use crate::battle::tips::{block_label, committed_icon_chains, draw_block_label, kind_color, BlockLook, BLOCK_ICON};
use crate::battle::{Action, BattleScreen, GRID_X, GRID_Y, TILE};
use crate::screens::{Cx, Screen};
use aegis_ui::art::Art;
use aegis_ui::canvas::Rect;
use aegis_ui::ui::{Gfx, Ui};
use emberwake_battle::session::Session;
use emberwake_rules::ruleset::TIMELINE;
use serde_json::{json, Value};

fn game_art() -> Art {
    Art::layered(aegis_core::assets::Layers::for_game(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")))
}

fn arena(ruleset: Option<&str>) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = game_art();
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn labels(b: &BattleScreen, cx: &Cx) -> Vec<Value> {
    b.state(cx)["strip_labels"].as_array().cloned().unwrap_or_default()
}

/// How many pixels of logical `r` pass `hit`.
fn pixels(g: &Gfx, r: Rect, hit: impl Fn(&[u8]) -> bool) -> usize {
    let r = g.view.rect(r);
    let mut n = 0;
    for y in r.y..r.y + r.h {
        for x in r.x..r.x + r.w {
            let i = ((y * g.canvas.w + x) * 4) as usize;
            n += hit(&g.canvas.px[i..i + 4]) as usize;
        }
    }
    n
}

/// Icon and name centred together; the icon alone centred when the name does not fit; the name alone centred without
/// an icon; the icon sits in the block's height.
#[test]
fn a_block_centres_what_it_shows() {
    let r = Rect::new(100, 6, 120, 16);
    let name_w = 40.0;
    let both = block_label(r, true, name_w);
    let (icon, x) = (both.icon.unwrap(), both.name_x.unwrap());
    assert!(icon.y >= r.y && icon.y + icon.h <= r.y + r.h && icon.y - r.y == r.y + r.h - icon.y - icon.h, "{icon:?}");
    assert!(((icon.x - r.x) as f32 - (r.x + r.w) as f32 + x + name_w).abs() <= 1.0, "the pair is centred: {icon:?} {x}");
    assert!(x >= (icon.x + icon.w) as f32, "the name follows the icon");
    let tight = Rect::new(100, 6, BLOCK_ICON + 6, 16);
    let alone = block_label(tight, true, name_w);
    assert_eq!(alone.name_x, None, "the name gives way");
    assert_eq!(alone.icon.unwrap().x - tight.x, tight.x + tight.w - alone.icon.unwrap().x - BLOCK_ICON);
    let plain = block_label(r, false, name_w);
    assert_eq!(plain.icon, None);
    assert!((plain.name_x.unwrap() - r.x as f32 - (r.w as f32 - name_w) / 2.0).abs() < 0.01);
    assert_eq!(block_label(tight, false, name_w).icon, None, "no icon: the name stays, however tight");
    assert!(block_label(tight, false, name_w).name_x.is_some());
}

/// Mock C: a chosen block's icon has a light border, its name is near-white with dark pixels round it; a ghost is fainter
/// on all of it; a dim Rest has no border and no outline. Each is measured against a plain kind-coloured fill.
#[test]
fn the_looks_read_as_chosen_ghost_and_dim() {
    let mut art = game_art();
    let chain = vec!["skills/kinds/attack.png".to_string()];
    assert!(art.first_rel(&chain).is_some(), "the kit's kind icon");
    let render = |art: &mut Art, look: BlockLook| {
        let mut g = crate::hud::hud_tests::gfx();
        let value = {
            let mut ui: Ui<Action> = Ui::new(&mut g, (-1, -1), true);
            let r = Rect::new(100, 6, 120, 16);
            ui.p.fill(r, kind_color("MOVE"));
            draw_block_label(&mut ui, art, r, "Shield Wall", &chain, look)
        };
        (g, value)
    };
    let r = Rect::new(100, 6, 120, 16);
    let light = |c: &[u8]| c[0] > 225 && c[1] > 215 && c[2] > 195;
    let dark = |c: &[u8]| c[0] < 40 && c[1] < 40 && c[2] < 40;
    let (chosen, v) = render(&mut art, BlockLook::Chosen);
    assert_eq!(v["icon"], json!("skills/kinds/attack.png"));
    assert_eq!(v["shows_name"], json!(true));
    let (c_light, c_dark) = (pixels(&chosen, r, light), pixels(&chosen, r, dark));
    let (ghost, _) = render(&mut art, BlockLook::Ghost);
    assert!(pixels(&ghost, r, light) * 2 < c_light, "the ghost's border and name are not light: {c_light}");
    assert!(pixels(&ghost, r, dark) < c_dark, "its outline is fainter");
    let (dim, v) = render(&mut art, BlockLook::Dim);
    assert_eq!(v["look"], json!("dim"));
    assert_eq!(pixels(&dim, r, light), 0, "a dim block has neither");
    assert!(pixels(&dim, r, dark) * 2 < c_dark, "no outline");
    // no icon: the name alone
    let mut g = crate::hud::hud_tests::gfx();
    let v = {
        let mut ui: Ui<Action> = Ui::new(&mut g, (-1, -1), true);
        ui.p.fill(r, kind_color("MOVE"));
        draw_block_label(&mut ui, &mut art, r, "Shield Wall", &["skills/kinds/no_such_kind.png".to_string()], BlockLook::Chosen)
    };
    assert_eq!((v["icon"].clone(), v["shows_name"].clone()), (Value::Null, json!(true)));
    assert!(pixels(&g, r, light) > 0 && pixels(&g, r, dark) > 0, "near-white text with its dark outline");
}

/// A real fight: every span of the selected hero gets a label; at the start they are the planner's ghost or dim Rest; a
/// pick makes chosen ones; a block that hides its name keeps it in its tip. The block with no plan stays dark. Classic
/// draws no such label.
#[test]
fn a_fight_labels_its_spans() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    b.tick(&mut cx);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let start = labels(&b, &cx);
    assert!(!start.is_empty(), "the tactic's ghost has spans");
    assert!(start.iter().all(|l| l["look"] != json!("chosen")), "nothing is chosen yet: {start:?}");
    let h = b.selected.unwrap();
    let foe = s(&b).w.enemies.iter().copied().find(|&e| s(&b).w.units[e].is_active() && s(&b).w.has(e)).unwrap();
    let r = b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}));
    assert_eq!(r["ok"], json!(true), "{r}");
    let r = b.send(&cx, json!({"cmd": "accept_ghost", "unit": h}));
    assert_eq!(r["ok"], json!(true), "{r}");
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let picked = labels(&b, &cx);
    // a chosen span of several blocks is its kind's colour from its first block, not a shaded dark bar
    let session = s(&b);
    let long: Vec<_> = session.plan.compile(&session.w, h).entries.iter().filter(|e| e.end_block(&session.w) > e.start_block).map(|e| (e.start_block, session.w.cat.skills[e.skill].kind.clone())).collect();
    assert!(!long.is_empty(), "a span of several blocks");
    for (start, kind) in long.iter().filter(|(_, kind)| kind != "REST") {
        let w = b.w.iter().find(|x| x.id == format!("strip.block{start}")).unwrap();
        let (x, y) = (g.view.x(w.rect.x as f32 + 3.0), g.view.y(w.rect.y as f32 + 1.0));
        let i = ((y * g.canvas.w + x) * 4) as usize;
        assert_eq!(g.canvas.px[i..i + 3], kind_color(kind)[..3], "{kind} at block {start}");
    }
    assert!(picked.iter().any(|l| l["look"] == json!("chosen")), "{picked:?}");
    assert!(picked.iter().filter(|l| l["look"] != json!("dim")).all(|l| l["icon"].is_string()), "every kind has an icon: {picked:?}");
    for l in picked.iter().filter(|l| l["shows_name"] == json!(false) || l["cut"] == json!(true)) {
        let name = l["name"].as_str().unwrap();
        assert!(b.w.iter().any(|x| x.id.starts_with("strip.block") && x.tip.iter().any(|(t, _)| t == name)), "{name} in a tip");
    }
    let (_, _, mut classic) = arena(None);
    classic.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(labels(&classic, &cx).is_empty());
}

/// A committed block finds its icon by the skill its command runs, not by name: with another skill of the same name
/// earlier in the catalog, which the unit knows too, planning and the committed block show the same family. A command
/// with no skill (an item) takes its kind's icon.
#[test]
fn a_committed_block_keeps_its_skill_when_names_repeat() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    b.tick(&mut cx);
    let h = b.selected.unwrap();
    let session = b.host.session.as_mut().unwrap();
    let plan = session.plan.compile(&session.w, h);
    let cat = std::sync::Arc::make_mut(&mut session.w.cat);
    // the plan's first skill that has another family before it in the catalog
    let (own, other) = plan.entries.iter().find_map(|e| {
        let fam = cat.skills[e.skill].family_id();
        cat.skills[..e.skill].iter().position(|o| o.family_id() != fam).map(|o| (e.skill, o))
    }).expect("a planned skill with another family before it");
    let (name, family, other_id) = (cat.skills[own].name.clone(), cat.skills[own].family_id().to_string(), cat.skills[other].id.clone());
    cat.skills[other].name = name.clone();
    session.w.units[h].known_skills.push(other_id);
    let r = b.send(&cx, json!({"cmd": "accept_ghost", "unit": h}));
    assert_eq!(r["ok"], json!(true), "{r}");
    let r = b.send(&cx, json!({"cmd": "confirm"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    let session = s(&b);
    let at = session.resolver.scheduled.iter().filter(|a| a.unit == h).position(|a| a.command.name == name).expect("the committed block");
    let chains = committed_icon_chains(session, h);
    assert_eq!(chains[at][0], format!("skills/{family}.png"));
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let session = b.host.session.as_mut().unwrap();
    let scheduled = session.resolver.scheduled.iter_mut().find(|a| a.unit == h).unwrap();
    scheduled.command.skill = String::new();
    let kind = crate::battle::tips::committed(session, h)[0].3.clone();
    assert_eq!(committed_icon_chains(session, h)[0], [format!("skills/kinds/{}.png", kind.to_lowercase())]);
}

/// A block with no icon (a committed item, until `kinds/item.png` exists) shows its name alone, and a name that does not
/// fit is reported cut, not cut silently; the tip has it whole. A block that hides its name, or fits it, cuts nothing.
#[test]
fn a_name_only_block_reports_a_cut() {
    let mut art = game_art();
    let long = "A Name Far Too Long For One Small Block";
    let chain = ["skills/kinds/no_such_kind.png".to_string()];
    let label = |art: &mut Art, r: Rect, name: &str, chain: &[String]| {
        let mut g = crate::hud::hud_tests::gfx();
        let mut ui: Ui<Action> = Ui::new(&mut g, (-1, -1), true);
        draw_block_label(&mut ui, art, r, name, chain, BlockLook::Chosen)
    };
    let tight = Rect::new(100, 6, 40, 16);
    let v = label(&mut art, tight, long, &chain);
    assert_eq!((v["icon"].clone(), v["shows_name"].clone(), v["cut"].clone()), (Value::Null, json!(true), json!(true)), "{v}");
    let v = label(&mut art, Rect::new(100, 6, 400, 16), long, &chain);
    assert_eq!(v["cut"], json!(false), "it fits: {v}");
    let v = label(&mut art, tight, long, &["skills/kinds/attack.png".to_string()]);
    assert_eq!((v["shows_name"].clone(), v["cut"].clone()), (json!(false), json!(false)), "the icon alone: {v}");
}

/// Decision "Battle view answers": no first-move chip under the round intent; the setting that showed it is gone, and a
/// file that still holds it is ignored. Classic has no round intent marker and keeps its chip. The intent glyphs are off
/// here, as they follow the tells too.
#[test]
fn only_classic_draws_first_move_chips() {
    for (ruleset, classic) in [(Some(TIMELINE), false), (None, true)] {
        let (mut cx, mut g, mut b) = arena(ruleset);
        cx.settings.v["battle_overlays"] = json!({"glyphs": false, "first_move": true});
        cx.settings.v["show_tells"] = json!(true);
        b.tick(&mut cx);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let tells = s(&b).enemy_intents();
        assert!(!tells.is_empty(), "tells while planning");
        let on = g.canvas.px.clone();
        cx.settings.v["show_tells"] = json!(false);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let differs = tells.iter().any(|t| {
            let at = s(&b).w.grid.pos(t.unit).unwrap();
            let chip = g.view.rect(Rect::new(GRID_X + at.x * TILE + 1, GRID_Y + at.y * TILE + 1, 10, 10));
            (chip.y..chip.y + chip.h).any(|y| (chip.x..chip.x + chip.w).any(|x| {
                let i = ((y * g.canvas.w + x) * 4) as usize;
                on[i..i + 4] != g.canvas.px[i..i + 4]
            }))
        });
        assert_eq!(differs, classic, "ruleset {ruleset:?}");
    }
    // the Stage view of a timeline battle: the tells draw nothing on the board once the glyphs are off
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    cx.settings.v["battle_view"] = json!("stage");
    cx.settings.v["battle_overlays"] = json!({"glyphs": false, "first_move": true});
    b.tick(&mut cx);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(!s(&b).enemy_intents().is_empty(), "tells while planning");
    let on = g.canvas.px.clone();
    cx.settings.v["show_tells"] = json!(false);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let board = g.view.rect(Rect::new(GRID_X, GRID_Y, emberwake_battle::grid::GRID_W * TILE, emberwake_battle::grid::GRID_H * TILE));
    for y in board.y..board.y + board.h {
        let (a, z) = (((y * g.canvas.w + board.x) * 4) as usize, ((y * g.canvas.w + board.x + board.w) * 4) as usize);
        assert!(on[a..z] == g.canvas.px[a..z], "row {y} of the board");
    }
    assert!(crate::battle::OVERLAY_GROUPS.iter().all(|g| g.0 != "first_move"));
}
