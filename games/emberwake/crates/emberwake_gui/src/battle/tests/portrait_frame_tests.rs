//! PR7-25 (`STYLE-ENGINE1` 2, style bible 2 F2): every portrait draws inside the one frame of the portrait widget
//! (`portraits::Faces` -> `rpgkit_ui::portrait_fx::draw_frame`): a 2-unit iron line (the theme's `edge`; 1 unit on a
//! portrait under `FRAME_THIN_BELOW`) with its top line in the theme's `frame_line` (oxblood), and no other border:
//! the border a picture paints itself (its sidecar's `border`) is covered. GUI audit on the talk box, the bunk card,
//! the Character screen, the hub's party box, the battle's panel portrait and the timeline's icons at 1280 x 720,
//! under the game's theme, with every portrait a scratch picture whose painted border is a marker colour.

use crate::hud::hud_tests::{game_cx, gfx};
use crate::portraits::{Faces, Who};
use crate::screens::{Cx, GameCx, Screen};
use aegis_core::assets::Layers;
use aegis_ui::art::Art;
use aegis_ui::canvas::{Canvas, Rect};
use aegis_ui::layout_audit::LayoutRec;
use aegis_ui::painter::View;
use rpgkit_ui::portrait_fx::{FRAME_LINE, FRAME_THIN_BELOW, PORTRAIT_FRAME};
use aegis_ui::ui::{Gfx, Ui};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// The painted border's marker colour and the picture's fill.
const MARK: [u8; 3] = [236, 196, 64];
const FILL: [u8; 3] = [150, 60, 40];
/// The scratch pictures' side and painted border (the sidecar's `size` and `border`; the shipped heroes paint 8 of 240).
const SIDE: u32 = 96;
const PAINTED: u32 = 6;

/// The game's theme, in force on this thread.
fn game_theme() -> std::rc::Rc<aegis_ui::theme::Theme> {
    let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let t = aegis_ui::theme::load(&Layers::for_game(&game)).expect("the game's theme loads");
    aegis_ui::theme::set_theme(t.clone());
    aegis_ui::theme::th()
}

/// A scratch layer holding `rel`: a `SIDE` px picture of `FILL` inside a `PAINTED` px painted border of `MARK`, with a
/// sidecar naming that border.
fn marked_layer(name: &str, rel: &str) -> PathBuf {
    let root = aegis_core::scratch::Scratch::vacant(&format!("emberwake_portrait_frame_{name}"));
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut px = Vec::new();
    for y in 0..SIDE {
        for x in 0..SIDE {
            let edge = x < PAINTED || y < PAINTED || x >= SIDE - PAINTED || y >= SIDE - PAINTED;
            let c = if edge { MARK } else { FILL };
            px.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(&path).unwrap()), SIDE, SIDE);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&px).unwrap();
    let face = json!({"size": SIDE, "border": PAINTED, "eyes": [], "blink": false, "lid": "", "glow": [], "lines": [], "mouth": [], "rim": "", "idle": []});
    std::fs::write(root.join(aegis_core::assets::face_file(rel)), face.to_string()).unwrap();
    root.until_thread_end()
}

/// Art whose every one of `whos` shows a marked picture (the first file of its plain chain).
fn marked_art(name: &str, whos: &[Who]) -> Art {
    let mut rels: Vec<String> = whos.iter().filter_map(|w| w.chain("").first().cloned()).collect();
    rels.sort();
    rels.dedup();
    Art::layered(Layers::of(rels.iter().enumerate().map(|(i, r)| marked_layer(&format!("{name}_{i}"), r)).collect()))
}

/// `g` at 1280 x 720 with the layout audit on.
fn at_720(g: &mut Gfx) {
    g.canvas = Canvas::new(1280, 720);
    g.view = View::fit(1280, 720);
    g.fonts.layout = Some(Vec::new());
}

/// The portrait frames the last draw placed.
fn placed(g: &mut Gfx) -> Vec<Rect> {
    g.fonts.layout.take().unwrap_or_default().into_iter().filter_map(|r| match r {
        LayoutRec::Placed { what, at } if what == PORTRAIT_FRAME => Some(at),
        _ => None,
    }).collect()
}

/// One frame of `screen`: the portrait frames placed.
fn draw(screen: &mut dyn Screen<GameCx>, cx: &mut Cx, g: &mut Gfx) -> Vec<Rect> {
    at_720(g);
    screen.draw(cx, g, (-500, -500), true);
    placed(g)
}

/// The rects of `screen`'s portrait widgets (kind `portrait`), square ones only (a timeline icon scrolled partly out
/// is cut to the rows' view).
fn portraits(screen: &dyn Screen<GameCx>) -> Vec<(String, Rect)> {
    let n = |w: &Value, i: usize| w["rect"][i].as_i64().unwrap_or(0) as i32;
    screen.widgets().into_iter().filter(|w| w["kind"] == "portrait").map(|w| (w["id"].as_str().unwrap_or("").to_string(), Rect::new(n(&w, 0), n(&w, 1), n(&w, 2), n(&w, 3)))).filter(|(_, r)| r.w == r.h && r.w > 0).collect()
}

fn px(g: &Gfx, x: i32, y: i32) -> [u8; 3] {
    let i = ((y * g.canvas.w + x) * 4) as usize;
    [g.canvas.px[i], g.canvas.px[i + 1], g.canvas.px[i + 2]]
}

/// The F2 check of the portrait at `r` (`what` names it): its frame placed at `r`; the top band in `frame_line`, the
/// sides and bottom in `edge` (each band the rule's width); no pixel of the painted border's marker inside `r`.
fn assert_f2(g: &Gfx, frames: &[Rect], r: Rect, what: &str) {
    let t = game_theme();
    let (line, edge) = (t.frame_line, t.edge);
    assert!(frames.contains(&r), "{what}: the F2 frame at {r:?} (placed {frames:?})");
    let w = if r.w.min(r.h) < FRAME_THIN_BELOW { 1 } else { FRAME_LINE };
    let (pr, inner) = (g.view.rect(r), g.view.rect(Rect::new(r.x + w, r.y + w, r.w - 2 * w, r.h - 2 * w)));
    let (mx, my) = (pr.x + pr.w / 2, pr.y + pr.h / 2);
    for y in pr.y..inner.y {
        assert_eq!(px(g, mx, y), line[..3], "{what}: the top line at y {y}");
    }
    for x in pr.x..inner.x {
        assert_eq!(px(g, x, my), edge[..3], "{what}: the left line at x {x}");
    }
    for x in inner.x + inner.w..pr.x + pr.w {
        assert_eq!(px(g, x, my), edge[..3], "{what}: the right line at x {x}");
    }
    for y in inner.y + inner.h..pr.y + pr.h {
        assert_eq!(px(g, mx, y), edge[..3], "{what}: the bottom line at y {y}");
    }
    let marks = (pr.y..pr.y + pr.h).flat_map(|y| (pr.x..pr.x + pr.w).map(move |x| (x, y))).filter(|&(x, y)| px(g, x, y) == MARK).count();
    assert_eq!(marks, 0, "{what}: no painted border shows inside the frame");
    let fills = (pr.y..pr.y + pr.h).flat_map(|y| (pr.x..pr.x + pr.w).map(move |x| (x, y))).filter(|&(x, y)| px(g, x, y) == FILL).count();
    assert!(fills > 0, "{what}: the picture shows inside the frame");
}

#[test]
fn the_character_screen_and_the_hub_party_box_frame_their_portraits() {
    game_theme();
    let mut cx = game_cx();
    cx.settings.v["reduce_motion"] = json!(true);
    let d = cx.data.clone();
    let whos: Vec<Who> = cx.gs().party.members.iter().map(|m| Who::of_member(&d, m)).collect();
    assert!(!whos.is_empty(), "the new game has a party");
    cx.art = marked_art("roster", &whos);
    let mut g = gfx();
    let member = cx.gs().party.members[0].id.clone();
    let mut c = crate::party::Character::new(&cx, &member, "stats");
    let frames = draw(&mut c, &mut cx, &mut g);
    let shown = portraits(&c);
    assert!(shown.iter().any(|(id, _)| id == "character.portrait"), "{shown:?}");
    for (id, r) in &shown {
        assert_f2(&g, &frames, *r, id);
    }
    let mut hub = crate::hub::Hub::default();
    let frames = draw(&mut hub, &mut cx, &mut g);
    let shown = portraits(&hub);
    assert!(shown.iter().any(|(id, _)| id.starts_with("hub.portrait.")), "{shown:?}");
    for (id, r) in &shown {
        assert_f2(&g, &frames, *r, id);
    }
}

#[test]
fn the_bunk_card_and_the_talk_box_frame_their_portraits() {
    game_theme();
    let (mut cx, ids) = crate::bunks::tests::hub_cx(false);
    let d = cx.data.clone();
    let gs = cx.gs().clone();
    let mut whos: Vec<Who> = gs.party.members.iter().map(|m| Who::of_member(&d, m)).collect();
    let speaker = d.merged_entries(&gs.world_id, "characters").into_iter().find(|c| aegis_core::json::s(c, "role") == "party").expect("a party character").clone();
    whos.push(Who::of_character(&d, &speaker));
    cx.art = marked_art("bunk_talk", &whos);
    let mut g = gfx();
    let mut card = crate::bunks::BunkCard::new(&ids[2]);
    let frames = draw(&mut card, &mut cx, &mut g);
    let shown = portraits(&card);
    assert!(shown.iter().any(|(id, _)| id == "bunk.portrait"), "{shown:?}");
    for (id, r) in &shown {
        assert_f2(&g, &frames, *r, id);
    }
    // the talk box (the region map's and the battle's one `draw_talk`)
    let view = aegis_core::json::dict(&json!({"speaker": aegis_core::json::s(&speaker, "_qid"), "speaker_name": aegis_core::json::s(&speaker, "name"), "node": "n", "text": "A line.", "choices": [], "end": true, "next": false, "faction": "", "faction_name": "", "band": ""})).clone();
    at_720(&mut g);
    let mut faces = Faces::default();
    let pr = {
        let mut ui: Ui<crate::talk::TalkAct> = Ui::new(&mut g, (-500, -500), true);
        faces.begin();
        let b = crate::talk::talk_layout(&mut ui.p, &view);
        crate::talk::draw_talk(&mut ui, &mut cx, &gs, &mut faces, &view, &b, false, |a| a);
        b.portrait_rect()
    };
    let frames = placed(&mut g);
    assert_f2(&g, &frames, pr, "talk box");
}

#[test]
fn the_battle_panel_and_timeline_portraits_are_framed() {
    game_theme();
    let (mut cx, mut g, mut b) = super::portrait_timeline_tests::battle("f2");
    let n = b.session().w.units.len();
    let whos: Vec<Who> = (0..n).map(|u| b.unit_portrait(&cx, u).0).collect();
    cx.art = marked_art("battle", &whos);
    cx.settings.v["reduce_motion"] = json!(true);
    let frames = draw(&mut b, &mut cx, &mut g);
    let shown = portraits(&b);
    assert!(shown.iter().any(|(id, _)| id == "hero.portrait"), "the panel portrait: {shown:?}");
    for (id, r) in &shown {
        assert_f2(&g, &frames, *r, id);
    }
    let icons: Vec<Rect> = b.widgets().into_iter().filter(|w| w["id"].as_str().is_some_and(|i| i.starts_with("timeline.icon."))).map(|w| {
        let n = |i: usize| w["rect"][i].as_i64().unwrap_or(0) as i32;
        Rect::new(n(0), n(1), n(2), n(3))
    }).filter(|r| r.w == r.h && r.w > 0).collect();
    assert!(!icons.is_empty(), "the timeline view shows its icon squares");
    for r in icons {
        assert_f2(&g, &frames, r, "timeline icon");
    }
}

/// Writes `g`'s canvas to `<dir>/<name>.png` and the portrait at `r` (with a 4-unit margin) at 4x to
/// `<dir>/<name>_zoom.png`.
fn write_shot(dir: &Path, g: &Gfx, name: &str, r: Rect) {
    let save = |file: &str, w: i32, h: i32, px: &[u8]| {
        let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(dir.join(file)).unwrap()), w as u32, h as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(px).unwrap();
    };
    save(&format!("{name}.png"), g.canvas.w, g.canvas.h, &g.canvas.px);
    let pr = g.view.rect(Rect::new(r.x - 4, r.y - 4, r.w + 8, r.h + 8));
    let (k, w, h) = (4, pr.w, pr.h);
    let mut px = Vec::with_capacity((w * h * k * k * 4) as usize);
    for y in 0..h * k {
        for x in 0..w * k {
            let (sx, sy) = ((pr.x + x / k).clamp(0, g.canvas.w - 1), (pr.y + y / k).clamp(0, g.canvas.h - 1));
            let i = ((sy * g.canvas.w + sx) * 4) as usize;
            px.extend_from_slice(&g.canvas.px[i..i + 4]);
        }
    }
    save(&format!("{name}_zoom.png"), w * k, h * k, &px);
}

/// Review shots (not a check; only with `AEGIS_REVIEW_DIR`): the shipped portraits in their F2 frame at 1280 x 720 on
/// the Character screen (`f2_party_card`), the bunk card's talk box (`f2_talk_box`) and the hub's party box
/// (`f2_hub_party`), each with a 4x zoom of one portrait.
#[test]
fn review_shots_f2_portraits() {
    let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") else { return };
    let dir = PathBuf::from(out);
    std::fs::create_dir_all(&dir).unwrap();
    game_theme();
    let (mut cx, ids) = crate::bunks::tests::hub_cx(false);
    let mut g = gfx();
    let mut c = crate::party::Character::new(&cx, &ids[0], "stats");
    draw(&mut c, &mut cx, &mut g);
    let r = portraits(&c).into_iter().find(|(id, _)| id == "character.portrait").expect("the portrait").1;
    write_shot(&dir, &g, "f2_party_card", r);
    let mut map = crate::region::RegionMap::new(&mut cx);
    draw(&mut map, &mut cx, &mut g);
    let mut card = crate::bunks::BunkCard::new(&ids[0]);
    at_720(&mut g);
    card.draw(&mut cx, &mut g, (-500, -500), true);
    let r = portraits(&card).into_iter().find(|(id, _)| id == "bunk.portrait").expect("the portrait").1;
    write_shot(&dir, &g, "f2_talk_box", r);
    let mut hub = crate::hub::Hub::default();
    draw(&mut hub, &mut cx, &mut g);
    if let Some((_, r)) = portraits(&hub).into_iter().find(|(id, _)| id.starts_with("hub.portrait.")) {
        write_shot(&dir, &g, "f2_hub_party", r);
    }
}
