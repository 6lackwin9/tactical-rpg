//! ENGINE-5 (ENGINE-PURITY-3 part 2): the Kit widgets' Portraits page moved from the engine to the kit
//! (`rpgkit_ui::portrait_page`) and draws pixel for pixel as before: its frame hashes at 1280 x 720 at the four text
//! sizes, idle and with the mouse over one widget of each kind, are the ones the engine's theme test recorded before
//! the theme's first edit (`fixtures/theme/portrait_hashes.json`, split unchanged from the engine's
//! `default_hashes.json`). The page sits at the fourth tab, as before, and its controls keep their ids.

use aegis_ui::canvas::{Canvas, Rect};
use aegis_ui::painter::{Fonts, View};
use aegis_ui::specimens::{KitAct, PAGES};
use aegis_ui::ui::{draw_tip, Gfx, Kit, Ui, Widget, BG};
use rpgkit_ui::portrait_page::KitWidgets;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// The Kit page's rect as the gallery gives it (on the 640 x 360 logical canvas), and its foot (left out).
const KIT: Rect = Rect { x: 112, y: 22, w: 522, h: 332 };
const FOOT_H: i32 = 18;
const TEXT_SIZES: [f32; 4] = [0.85, 1.0, 1.15, 1.3];
const HOVER_KINDS: [&str; 11] = ["button", "card", "row", "select", "dropdown", "text", "textarea", "checkbox", "section", "icon_button", "slider_step"];

fn draw_page(k: &KitWidgets, scale: f32, mouse: (i32, i32)) -> (Gfx, Vec<Widget<KitAct>>) {
    let font = std::fs::read(root().join("../../../../engine/assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
    let mut g = Gfx { canvas: Canvas::new(1280, 720), view: View::fit(1280, 720), fonts: Fonts::new(&font), kit: Kit::default() };
    g.fonts.scale = scale;
    g.canvas.clear(BG);
    let w = {
        let mut ui: Ui<KitAct> = Ui::new(&mut g, mouse, true);
        k.draw(&mut ui, KIT, |a| a);
        let w = std::mem::take(&mut ui.widgets);
        draw_tip(&mut ui.p, &w, mouse);
        w
    };
    (g, w)
}

/// FNV-1a 64 over the canvas with the page's foot blanked (the engine theme test's hash).
fn frame_hash(g: &Gfx) -> String {
    let foot = g.view.rect(Rect::new(KIT.x, KIT.y + KIT.h - FOOT_H, KIT.w, FOOT_H));
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for y in 0..g.canvas.h {
        for x in 0..g.canvas.w {
            let i = ((y * g.canvas.w + x) * 4) as usize;
            let px = if foot.contains(x, y) { [0u8; 4] } else { [g.canvas.px[i], g.canvas.px[i + 1], g.canvas.px[i + 2], g.canvas.px[i + 3]] };
            for b in px {
                h ^= b as u64;
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        }
    }
    format!("{h:016x}")
}

#[test]
fn the_portraits_page_is_the_fourth_tab_and_draws_as_before() {
    assert_eq!(KitWidgets::pages(), ["Widgets", "Timeline", "Effects", "Portraits", "Theme", "Panes", "Tiles", "Frames", "Layout"]);
    assert_eq!(KitWidgets::pages().len(), PAGES.len() + 1);
    let page = KitWidgets::page_of("Portraits").unwrap();
    let k = KitWidgets { page, ..KitWidgets::default() };
    let mut now = Map::new();
    for scale in TEXT_SIZES {
        let (g, w) = draw_page(&k, scale, (-1, -1));
        now.insert(format!("portraits@{scale}"), Value::from(frame_hash(&g)));
        for kind in HOVER_KINDS {
            let Some(wd) = w.iter().find(|wd| wd.kind == kind && !wd.id.starts_with("kit.page.") && wd.rect.w > 0 && wd.rect.h > 0) else { continue };
            let (g, _) = draw_page(&k, scale, (wd.rect.x + wd.rect.w / 2, wd.rect.y + wd.rect.h / 2));
            now.insert(format!("portraits@{scale}#{}", wd.id), Value::from(frame_hash(&g)));
        }
    }
    let file = root().join("tests/fixtures/theme/portrait_hashes.json");
    let recorded: Map<String, Value> = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    assert_eq!(recorded.len(), 16);
    let differ: Vec<&String> = recorded.keys().filter(|k| now.get(*k) != recorded.get(*k)).collect();
    assert!(differ.is_empty(), "Portraits page frames that changed: {differ:?}");
}
