//! PR7-15 (a) (`WEBFIX2`): the heads-up panels of the battle (the top strip, the two sides, the bottom bar), the region
//! map (the HUD bar, the legend's rows, the log box) and the hub (the HUD bar; the hub menu's party box) draw the theme's
//! `hud` frame art (the style bible's F2: the thin iron frame with the oxblood top line, `crate::hud::HUD_FRAME`) instead
//! of flat panels: the frame is placed over each of them (the layout records), its top edge shows the theme's
//! `frame_line` on the real picture, and with frames off the screens draw no frame and keep their vector panels. The
//! battle is the scratch world of `transitions_tests`; the map and the hub are a debug game of the first playable world.

use super::transitions_tests::{battle, beat_cx};
use crate::hud::hud_tests::{game_cx, gfx};
use crate::hud::{HUD_FRAME, HUD_H};
use crate::screens::{Cx, GameCx, Screen};
use aegis_ui::canvas::{Canvas, Rect};
use aegis_ui::layout_audit::LayoutRec;
use aegis_ui::painter::View;
use aegis_ui::ui::{Gfx, W};
use std::path::Path;

/// The game's theme, in force on this thread.
fn game_theme() -> std::rc::Rc<aegis_ui::theme::Theme> {
    let game = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let t = aegis_ui::theme::load(&aegis_core::assets::Layers::for_game(&game)).expect("the game's theme loads");
    aegis_ui::theme::set_theme(t.clone());
    aegis_ui::theme::th()
}

/// One frame of `screen` at 1280 x 720: where it placed the HUD frame.
fn hud_frames(screen: &mut dyn Screen<GameCx>, cx: &mut Cx, g: &mut Gfx) -> Vec<Rect> {
    g.canvas = Canvas::new(1280, 720);
    g.view = View::fit(1280, 720);
    g.fonts.layout = Some(Vec::new());
    screen.draw(cx, g, (-500, -500), true);
    let recs = g.fonts.layout.take().unwrap_or_default();
    recs.into_iter().filter_map(|r| match r {
        LayoutRec::Placed { what, at } if what == HUD_FRAME => Some(at),
        _ => None,
    }).collect()
}

/// The picture's pixel at the middle of `r`'s top edge, one physical px inside it: the frame's top line.
fn top_line(g: &Gfx, r: Rect) -> [u8; 3] {
    let pr = g.view.rect(r);
    let (x, y) = (pr.x + pr.w / 2, pr.y);
    let i = ((y * g.canvas.w + x) * 4) as usize;
    [g.canvas.px[i], g.canvas.px[i + 1], g.canvas.px[i + 2]]
}

/// The theme's frame line colour.
fn line(t: &aegis_ui::theme::Theme) -> [u8; 3] {
    let l = t.role("frame_line").expect("frame_line");
    [l[0], l[1], l[2]]
}

#[test]
fn the_battle_hud_panels_draw_the_hud_frame() {
    let t = game_theme();
    assert!(t.frame(HUD_FRAME).is_some(), "the game ships the HUD frame");
    let mut cx = beat_cx(false, true);
    let mut b = battle(&mut cx, "map");
    b.tick(&mut cx);
    let mut g = gfx();
    let placed = hud_frames(&mut b, &mut cx, &mut g);
    // BATTLE-UI2 (contract 3): the side panels 184 px, the board's panel between them
    let sides = [Rect::new(2, 2, W - 4, 24), Rect::new(2, 28, 92, 290), Rect::new(W - 94, 28, 92, 290), Rect::new(98, 28, 444, 290), Rect::new(2, 322, W - 4, 36)];
    for r in sides {
        assert!(placed.contains(&r), "the battle frames {r:?}: {placed:?}");
    }
    assert_eq!(top_line(&g, sides[0]), line(&t), "the top strip shows the frame's top line");
    assert_eq!(top_line(&g, sides[1]), line(&t), "the party side shows it");
    aegis_ui::theme::set_theme(aegis_ui::theme::Theme::default());
    assert!(hud_frames(&mut b, &mut cx, &mut g).is_empty(), "frames off: the vector panels");
}

#[test]
fn the_region_map_and_hub_hud_panels_draw_the_hud_frame() {
    let t = game_theme();
    let mut cx = game_cx();
    let mut g = gfx();
    let mut map = crate::region::RegionMap::new(&mut cx);
    let placed = hud_frames(&mut map, &mut cx, &mut g);
    let bar = Rect::new(0, 0, W, HUD_H);
    assert!(placed.contains(&bar), "the HUD bar is framed: {placed:?}");
    assert_eq!(top_line(&g, bar), line(&t), "the HUD bar shows the frame's top line");
    assert!(placed.iter().any(|r| r.y > HUD_H && r.x > W / 2), "the legend's rows are framed: {placed:?}");
    assert!(placed.iter().any(|r| r.y > HUD_H && r.x < W / 2), "the log box is framed: {placed:?}");
    let mut hub = crate::hub::Hub::default();
    let placed = hud_frames(&mut hub, &mut cx, &mut g);
    assert_eq!(placed.len(), 1, "the hub menu frames its party box: {placed:?}");
    aegis_ui::theme::set_theme(aegis_ui::theme::Theme::default());
    let mut map = crate::region::RegionMap::new(&mut cx);
    assert!(hud_frames(&mut map, &mut cx, &mut g).is_empty(), "frames off: the vector panels");
    let _ = std::fs::remove_dir_all(&cx.shot_dir);
}

/// Review shots of the framed battle, region map and hub at 1280 x 720 (`AEGIS_REVIEW_DIR`).
#[test]
fn hud_frame_shots() {
    let Ok(out) = std::env::var("AEGIS_REVIEW_DIR") else { return };
    game_theme();
    let write = |g: &Gfx, name: &str| {
        let file = std::fs::File::create(Path::new(&out).join(name)).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), g.canvas.w as u32, g.canvas.h as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
    };
    let mut g = gfx();
    let mut cx = beat_cx(false, true);
    let mut b = battle(&mut cx, "map");
    b.tick(&mut cx);
    hud_frames(&mut b, &mut cx, &mut g);
    write(&g, "pr7_15_battle_hud.png");
    let mut cx = game_cx();
    cx.art = aegis_ui::art::Art::new(Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    let mut map = crate::region::RegionMap::new(&mut cx);
    hud_frames(&mut map, &mut cx, &mut g);
    write(&g, "pr7_15_region_hud.png");
    let mut hub = crate::hub::Hub::default();
    hud_frames(&mut hub, &mut cx, &mut g);
    write(&g, "pr7_15_hub_menu_hud.png");
    aegis_ui::theme::set_theme(aegis_ui::theme::Theme::default());
}
