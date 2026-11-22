//! The two screens: the main menu and the field. Both draw on the engine's old 640 x 360 grid with the engine's widgets
//! and default theme. The field is a tile layer seen through a world camera (WORLD-PRIM1), and a click on a free tile
//! walks there by the engine's grid search.

use crate::walk::{self, Walk};
use crate::{play, save, sound, Hello};
use aegis_core::clock::Instant;
use aegis_ui::canvas::{rgb, Image, Rect, Rgba};
use aegis_ui::ground::{self, GroundJob, GroundKey, GroundLayer};
use aegis_ui::pacing::FrameNeed;
use aegis_ui::screen::{Cx, Key, Nav, Screen};
use aegis_ui::tilemap::TileLayer;
use aegis_ui::ui::{Gfx, Ui, Widget, BODY, H, HEAD, W};
use aegis_ui::world_camera::WorldCamera;
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
    /// Steps left of a walk to a clicked tile (taken one per slide).
    path: Vec<(i64, i64)>,
    /// ENGINE-40: the composed ground's key, made once (it hashes every texture pixel, so never per frame).
    ground: Option<GroundKey>,
}

/// The field's tiles: 1 wall, 2 and 3 the two shades of grass (the clicks read them; ENGINE-40: only the walls are drawn
/// from them, over the composed ground).
pub fn field_layer() -> TileLayer {
    let mut layer = TileLayer::new(walk::COLS as i32, walk::ROWS as i32, TILE);
    for row in 0..walk::ROWS {
        for col in 0..walk::COLS {
            layer.set(col as i32, row as i32, if walk::blocked(col, row) { 1 } else if (col + row) % 2 == 0 { 2 } else { 3 });
        }
    }
    layer
}

/// ENGINE-40 (ENGINE-GUIDE2): the path the field's ground shows, as (column, row): a worn dirt road across the field
/// and a branch down to it. A picture only: the walk never reads it (`walk::blocked` alone decides walking).
pub fn on_path(col: i64, row: i64) -> bool {
    row == 2 || (col == 10 && (2..=8).contains(&row)) || (row == 8 && (10..=17).contains(&col))
}

/// The grass and the dirt: seamless two-tone textures the size the composer samples (`TEX_CELLS` cells of `CELL_PX`).
const GRASS: (Rgba, Rgba) = (rgb(92, 140, 84), rgb(80, 126, 74));
const DIRT: (Rgba, Rgba) = (rgb(150, 122, 86), rgb(132, 106, 74));
const TEX: i32 = ground::TEX_CELLS * ground::CELL_PX;

/// ENGINE-40: the field's ground job, handed to `f`: grass everywhere (walls included, under their tiles) and the dirt
/// path over it, the two meeting at the composer's worn, generated edge, with a map-wide tone and a few quiet patches
/// on the grass. The engine's ground composer (`aegis_ui::ground`, the same one a kit's region maps use) turns it into
/// one picture with no cell edge in it; the walls are drawn as tiles on top.
pub fn with_field_ground<R>(f: impl FnOnce(&GroundJob) -> R) -> R {
    let grass_tex: Image = ground::two_tone(TEX, TEX, GRASS.0, GRASS.1, 1);
    let dirt_tex: Image = ground::two_tone(TEX, TEX, DIRT.0, DIRT.1, 2);
    let mut cells = Vec::new();
    let mut blocked = Vec::new();
    for row in 0..walk::ROWS {
        for col in 0..walk::COLS {
            let wall = walk::blocked(col, row);
            cells.push(u8::from(!wall && on_path(col, row)));
            blocked.push(wall);
        }
    }
    let grass = GroundLayer { tex: Some(&grass_tex), tone: 0.08, patches: 0.12, ..GroundLayer::flat(0, GRASS.0) };
    let dirt = GroundLayer { tex: Some(&dirt_tex), wear: 0.12, strays: true, tone: 0.05, ..GroundLayer::flat(1, DIRT.0) };
    let layers = [grass, dirt];
    f(&GroundJob {
        seed: 7,
        cols: walk::COLS as i32,
        rows: walk::ROWS as i32,
        cell: (ground::CELL_PX, ground::CELL_PX),
        cells: &cells,
        blocked: &blocked,
        layers: &layers,
        placed: &[],
    })
}

/// The camera the field is seen through: zoom 1, the whole field in its place on the screen.
pub fn field_camera(layer: &TileLayer) -> WorldCamera {
    let (_, _, w, h) = layer.bounds();
    let mut cam = WorldCamera::new(Rect::new(ORIGIN.0, ORIGIN.1, w as i32, h as i32)).bounded(layer.bounds());
    cam.snap((w / 2.0, h / 2.0));
    cam
}

impl Field {
    /// Starts a walk to tile `to`: false (and a bump) when no free route reaches it.
    fn walk_to(&mut self, cx: &mut HCx, to: (i64, i64)) -> bool {
        match cx.app.walk.route(to) {
            Some(steps) => {
                self.path = steps;
                self.path.reverse();
                true
            }
            None => {
                play(cx, sound::BUMP);
                false
            }
        }
    }

    /// One step: a slide from where the walker is drawn now, with its sound; false on a bump.
    fn step(&mut self, cx: &mut HCx, dx: i64, dy: i64) -> bool {
        let from = self.walker_at(cx.app.walk);
        if cx.app.walk.step(dx, dy) {
            self.slide = Some((from, Instant::now()));
            play(cx, sound::STEP);
            true
        } else {
            play(cx, sound::BUMP);
            false
        }
    }

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
        if self.slide.is_some() || !self.path.is_empty() {
            FrameNeed::Full
        } else {
            FrameNeed::Idle
        }
    }
    fn tick(&mut self, cx: &mut HCx) {
        if self.slide.is_some_and(|(_, at)| at.elapsed().as_millis() >= SLIDE_MS) {
            self.slide = None;
        }
        if self.slide.is_none() {
            if let Some((dx, dy)) = self.path.pop() {
                if !self.step(cx, dx, dy) {
                    self.path.clear();
                }
            }
        }
    }
    fn draw(&mut self, cx: &mut HCx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let w = cx.app.walk;
        let sprite = cx.art.image(SPRITE, 16, 16).cloned();
        let at = self.walker_at(w);
        let mut ui: Ui<FieldAct> = Ui::new(g, mouse, live);
        let (bg, text, dim) = (ui.t.bg, ui.t.text, ui.t.dim);
        ui.p.fill(Rect::new(0, 0, W, H), bg);
        ui.p.text(ORIGIN.0 as f32, 24.0, "Walk with the arrows or W A S D, or click a tile.", BODY, text);
        ui.p.text(ORIGIN.0 as f32, 38.0, &format!("Steps {}   Tile {}, {}", w.steps, w.x, w.y), BODY, dim);
        let layer = field_layer();
        let key = self.ground.get_or_insert_with(|| with_field_ground(|job| GroundKey::of("hello_field", job))).clone();
        let (_, _, fw, fh) = layer.bounds();
        ground::draw(&mut ui.p, key, Rect::new(ORIGIN.0, ORIGIN.1, fw as i32, fh as i32), || with_field_ground(ground::compose));
        layer.draw(&field_camera(&layer), |_, _, v, r| {
            if v == 1 {
                ui.p.fill(r, rgb(58, 62, 74));
            }
        });
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
        // A key takes over from a walk to a clicked tile; from where it is drawn now, so a step pressed mid-slide
        // carries on from there.
        self.path.clear();
        self.step(cx, dx, dy);
    }
    fn back(&mut self, cx: &mut HCx) {
        self.act(cx, FieldAct::Menu);
    }
    fn state(&self, cx: &HCx) -> Value {
        json!({"walk": cx.app.walk.to_value(), "sliding": self.slide.is_some(), "status": self.status, "path_left": self.path.len()})
    }
    /// The link's `walk_to {x, y}`: what a click on that tile does.
    fn command(&mut self, cx: &HCx, body: &Value) -> Option<Value> {
        if body["cmd"] != json!("walk_to") {
            return None;
        }
        let (x, y) = (body["x"].as_i64().unwrap_or(-1), body["y"].as_i64().unwrap_or(-1));
        Some(match cx.app.walk.route((x, y)) {
            Some(steps) => {
                self.path = steps.into_iter().rev().collect();
                json!({"ok": true, "steps": self.path.len()})
            }
            None => json!({"ok": false, "error": format!("no free route to ({x}, {y})")}),
        })
    }
    fn widgets(&self) -> Vec<Value> {
        self.w.iter().map(|w| w.info()).collect()
    }
    fn draw_tip(&self, p: &mut aegis_ui::painter::Painter, mouse: (i32, i32)) {
        aegis_ui::ui::draw_tip(p, &self.w, mouse)
    }
    /// A widget's click, else a click on a free tile of the field walks there.
    fn click(&mut self, cx: &mut HCx, pos: (i32, i32)) -> bool {
        if let Some(a) = aegis_ui::ui::hit(&self.w, pos) {
            self.act(cx, a);
            return true;
        }
        let layer = field_layer();
        match layer.cell_at(field_camera(&layer).to_world((pos.0 as f32, pos.1 as f32))) {
            Some((x, y)) => self.walk_to(cx, (i64::from(x), i64::from(y))),
            None => false,
        }
    }
    fn click_id(&mut self, cx: &mut HCx, id: &str) -> Result<(), String> {
        let a = aegis_ui::ui::by_id(&self.w, id)?;
        self.act(cx, a);
        Ok(())
    }
    fn widget_rect(&self, id: &str) -> Option<(Rect, Vec<String>)> {
        self.w.iter().find(|w| w.id == id).map(|w| (w.rect, w.tip.iter().map(|t| t.0.clone()).collect()))
    }
}
