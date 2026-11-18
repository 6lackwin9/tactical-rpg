//! The two screens: the main menu and the field. Both draw on the engine's old 640 x 360 grid with the engine's widgets
//! and default theme.

use crate::walk::{self, Walk};
use crate::{play, save, sound, Hello};
use aegis_core::clock::Instant;
use aegis_ui::canvas::{rgb, Rect};
use aegis_ui::pacing::FrameNeed;
use aegis_ui::screen::{Cx, Key, Nav, Screen};
use aegis_ui::ui::{Gfx, Ui, Widget, BODY, H, HEAD, W};
use aegis_ui::widget_plumbing;
use serde_json::{json, Value};

type HCx = Cx<Hello>;

// ---- the main menu ----

#[derive(Clone, Copy)]
pub enum MenuAct {
    New,
    Continue,
    Quit,
}

/// The main menu: a new walk, the saved walk, quit.
#[derive(Default)]
pub struct Menu {
    w: Vec<Widget<MenuAct>>,
    /// Why the last Continue failed.
    problem: Option<String>,
}

impl Menu {
    fn act(&mut self, cx: &mut HCx, a: MenuAct) {
        play(cx, sound::PRESS);
        match a {
            MenuAct::New => {
                cx.app.walk = Walk::default();
                cx.nav.push(Nav::Reset(Box::new(Field::default())));
            }
            MenuAct::Continue => match save::read(&cx.app.save_dir) {
                Ok(w) => {
                    cx.app.walk = w;
                    cx.nav.push(Nav::Reset(Box::new(Field::default())));
                }
                Err(e) => self.problem = Some(e.join(" ")),
            },
            MenuAct::Quit => cx.quit = true,
        }
    }
}

impl Screen<Hello> for Menu {
    fn id(&self) -> &'static str {
        "main_menu"
    }
    fn frame_need(&self, _cx: &HCx) -> FrameNeed {
        FrameNeed::Idle
    }
    fn auto_focus(&self) -> bool {
        true
    }
    fn draw(&mut self, cx: &mut HCx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let can_continue = save::exists(&cx.app.save_dir);
        let sprite = cx.art.image(SPRITE, 16, 16).cloned();
        let mut ui: Ui<MenuAct> = Ui::new(g, mouse, live);
        let (bg, text, dim, bad) = (ui.t.bg, ui.t.text, ui.t.dim, ui.t.foe);
        ui.p.fill(Rect::new(0, 0, W, H), bg);
        if let Some(img) = &sprite {
            ui.p.image(img, Rect::new(W / 2 - 24, 52, 48, 48));
        }
        let title = "Hello, Aegis";
        let tw = ui.p.text_w(title, HEAD);
        ui.p.text((W as f32 - tw) / 2.0, 112.0, title, HEAD, text);
        let sub = "The engine's sample game: walk, save, come back.";
        let sw = ui.p.text_w(sub, BODY);
        ui.p.text((W as f32 - sw) / 2.0, 132.0, sub, BODY, dim);
        let x = W / 2 - 60;
        ui.button("menu.new", Rect::new(x, 160, 120, 18), "New walk", MenuAct::New, true);
        ui.button("menu.continue", Rect::new(x, 184, 120, 18), "Continue", MenuAct::Continue, can_continue);
        ui.button("menu.quit", Rect::new(x, 208, 120, 18), "Quit", MenuAct::Quit, true);
        if let Some(p) = &self.problem {
            let pw = ui.p.text_w(p, BODY);
            ui.p.text((W as f32 - pw) / 2.0, 240.0, p, BODY, bad);
        }
        self.w = ui.widgets;
    }
    fn state(&self, cx: &HCx) -> Value {
        json!({"can_continue": save::exists(&cx.app.save_dir), "problem": self.problem})
    }
    /// Esc on the menu does nothing (Quit is a button).
    fn back(&mut self, _cx: &mut HCx) {}
    widget_plumbing!(HCx);
}

// ---- the field ----

/// The walker's picture, over the asset layers (the game's own `assets/`).
pub const SPRITE: &str = "sprites/walker.png";
/// One tile in grid units, and the field's top-left corner.
pub const TILE: i32 = 24;
pub const ORIGIN: (i32, i32) = ((W - walk::COLS as i32 * TILE) / 2, 60);
/// How long the walker slides from one tile to the next.
pub const SLIDE_MS: u128 = 120;

#[derive(Clone, Copy)]
pub enum FieldAct {
    Save,
    Menu,
}

/// The walled field the walker crosses: arrows or W A S D step, Save writes the slot, Menu (or Esc) goes back.
#[derive(Default)]
pub struct Field {
    w: Vec<Widget<FieldAct>>,
    /// Where the walker was drawn (grid units) when it last stepped, and when, while it slides.
    slide: Option<((i32, i32), Instant)>,
    /// The line under the field: the last save, or what refused it.
    status: String,
}

impl Field {
    fn act(&mut self, cx: &mut HCx, a: FieldAct) {
        match a {
            FieldAct::Save => {
                let problems = save::write(&cx.app.save_dir, cx.app.walk);
                if problems.is_empty() {
                    play(cx, sound::SAVED);
                    self.status = format!("Saved at {}, {} after {} steps.", cx.app.walk.x, cx.app.walk.y, cx.app.walk.steps);
                } else {
                    play(cx, sound::BUMP);
                    self.status = problems.join(" ");
                }
            }
            FieldAct::Menu => {
                play(cx, sound::BACK);
                cx.nav.push(Nav::Reset(Box::new(Menu::default())));
            }
        }
    }

    /// The walker's place in grid units now: between the tile it left and its tile while it slides.
    fn walker_at(&self, w: Walk) -> (i32, i32) {
        let to = (ORIGIN.0 + w.x as i32 * TILE, ORIGIN.1 + w.y as i32 * TILE);
        match self.slide {
            Some((from, at)) => {
                let t = (at.elapsed().as_millis().min(SLIDE_MS) as f32) / SLIDE_MS as f32;
                (from.0 + ((to.0 - from.0) as f32 * t) as i32, from.1 + ((to.1 - from.1) as f32 * t) as i32)
            }
            None => to,
        }
    }
}

impl Screen<Hello> for Field {
    fn id(&self) -> &'static str {
        "field"
    }
    fn arrow_focus(&self) -> bool {
        false
    }
    fn frame_need(&self, _cx: &HCx) -> FrameNeed {
        if self.slide.is_some() {
            FrameNeed::Full
        } else {
            FrameNeed::Idle
        }
    }
    fn tick(&mut self, _cx: &mut HCx) {
        if self.slide.is_some_and(|(_, at)| at.elapsed().as_millis() >= SLIDE_MS) {
            self.slide = None;
        }
    }
    fn draw(&mut self, cx: &mut HCx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let w = cx.app.walk;
        let sprite = cx.art.image(SPRITE, 16, 16).cloned();
        let at = self.walker_at(w);
        let mut ui: Ui<FieldAct> = Ui::new(g, mouse, live);
        let (bg, text, dim) = (ui.t.bg, ui.t.text, ui.t.dim);
        ui.p.fill(Rect::new(0, 0, W, H), bg);
        ui.p.text(ORIGIN.0 as f32, 24.0, "Walk with the arrows or W A S D.", BODY, text);
        ui.p.text(ORIGIN.0 as f32, 38.0, &format!("Steps {}   Tile {}, {}", w.steps, w.x, w.y), BODY, dim);
        for row in 0..walk::ROWS {
            for col in 0..walk::COLS {
                let r = Rect::new(ORIGIN.0 + col as i32 * TILE, ORIGIN.1 + row as i32 * TILE, TILE, TILE);
                let c = if walk::blocked(col, row) {
                    rgb(58, 62, 74)
                } else if (col + row) % 2 == 0 {
                    rgb(92, 140, 84)
                } else {
                    rgb(84, 130, 78)
                };
                ui.p.fill(r, c);
            }
        }
        match &sprite {
            Some(img) => ui.p.image(img, Rect::new(at.0, at.1, TILE, TILE)),
            None => ui.p.fill(Rect::new(at.0 + 4, at.1 + 4, TILE - 8, TILE - 8), rgb(88, 168, 232)),
        }
        let bottom = ORIGIN.1 + walk::ROWS as i32 * TILE;
        ui.p.text(ORIGIN.0 as f32, bottom as f32 + 14.0, &self.status, BODY, dim);
        let right = ORIGIN.0 + walk::COLS as i32 * TILE;
        ui.button("field.save", Rect::new(right - 124, 22, 58, 18), "Save", FieldAct::Save, true);
        ui.button("field.menu", Rect::new(right - 60, 22, 60, 18), "Menu", FieldAct::Menu, true);
        self.w = ui.widgets;
    }
    fn key(&mut self, cx: &mut HCx, k: Key) {
        let Some((dx, dy)) = k.dir() else { return };
        // From where it is drawn now, so a step pressed mid-slide carries on from there.
        let from = self.walker_at(cx.app.walk);
        if cx.app.walk.step(dx, dy) {
            self.slide = Some((from, Instant::now()));
            play(cx, sound::STEP);
        } else {
            play(cx, sound::BUMP);
        }
    }
    fn back(&mut self, cx: &mut HCx) {
        self.act(cx, FieldAct::Menu);
    }
    fn state(&self, cx: &HCx) -> Value {
        json!({"walk": cx.app.walk.to_value(), "sliding": self.slide.is_some(), "status": self.status})
    }
    widget_plumbing!(HCx);
}
