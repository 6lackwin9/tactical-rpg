//! The screens: the main menu (on the native grid, 1280 x 720 units, `Grid::Native`, its words in
//! `assets/strings/en.json`) and the board, on the old 640 x 360 grid the kit's battle stage is laid out on
//! (`rpgkit_ui::stage`): the stage passes over the fight's grid, with the encounter's decor and the stage set's light.

use crate::battle::{Fight, STAGE_SET};
use crate::{play, sound, KitSample};
use aegis_layout::Anchor;
use aegis_ui::painter::Grid;
use aegis_ui::pacing::FrameNeed;
use aegis_ui::screen::{Cx, Nav, Screen};
use aegis_ui::strings::tr;
use aegis_ui::canvas::Rect;
use aegis_ui::ui::{Gfx, Ui, Widget, BODY, HEAD};
use rpgkit_ui::stage::{self, Geom, Item, Pass, Sizes, View};
use aegis_ui::widget_plumbing;
use serde_json::{json, Value};

type GCx = Cx<KitSample>;

// ---- the main menu ----

#[derive(Clone, Copy)]
pub enum MenuAct {
    Start,
    Quit,
}

/// The main menu: start, quit.
#[derive(Default)]
pub struct Menu {
    w: Vec<Widget<MenuAct>>,
}

impl Menu {
    fn act(&mut self, cx: &mut GCx, a: MenuAct) {
        play(cx, sound::PRESS);
        match a {
            MenuAct::Start => {
                cx.app.started = true;
                cx.nav.push(Nav::Push(Box::new(Board::default())));
            }
            MenuAct::Quit => cx.quit = true,
        }
    }
}

impl Screen<KitSample> for Menu {
    fn id(&self) -> &'static str {
        "main_menu"
    }
    fn grid(&self) -> Grid {
        Grid::Native
    }
    fn frame_need(&self, _cx: &GCx) -> FrameNeed {
        FrameNeed::Idle
    }
    fn auto_focus(&self) -> bool {
        true
    }
    fn draw(&mut self, _cx: &mut GCx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<MenuAct> = Ui::new(g, mouse, live);
        let (m, area) = (ui.m(), ui.screen());
        let (bg, text) = (ui.t.bg, ui.t.text);
        ui.p.fill(area, bg);
        let title = "Kit Sample";
        let tw = ui.p.text_w(title, m.title);
        ui.p.text(area.x as f32 + (area.w as f32 - tw) / 2.0, area.y as f32 + area.h as f32 * 0.3, title, m.title, text);
        let mut col = area.place(Anchor::Center, m.u(80), m.u(30));
        let start = col.cut_top(m.u(12));
        col.cut_top(m.u(3));
        let quit = col.cut_top(m.u(12));
        ui.button("menu.start", start, &tr("menu.start"), MenuAct::Start, true);
        ui.button("menu.quit", quit, &tr("menu.quit"), MenuAct::Quit, true);
        self.w = ui.widgets;
    }
    fn state(&self, cx: &GCx) -> Value {
        json!({"started": cx.app.started})
    }
    /// Esc on the menu does nothing (Quit is a button).
    fn back(&mut self, _cx: &mut GCx) {}
    widget_plumbing!(GCx);
}

// ---- the board ----

#[derive(Clone, Copy)]
pub enum BoardAct {
    Round,
    ToEnd,
    Back,
}

/// The unit sizes on the stage (the kit's `StageSizes`; a game with a tuning file passes its tuning instead).
pub const SIZES: Sizes = Sizes { token_px: 40, flat_token_px: 28, foot_inset_px: 5 };

/// The fight on the kit's battle stage: Round plays one round, To the end plays them all, Back (or Esc) to the menu.
#[derive(Default)]
pub struct Board {
    w: Vec<Widget<BoardAct>>,
}

impl Board {
    fn act(&mut self, cx: &mut GCx, a: BoardAct) {
        play(cx, sound::PRESS);
        let Some(f) = cx.app.fight.as_mut() else { return };
        match a {
            BoardAct::Round => {
                f.round();
            }
            BoardAct::ToEnd => {
                f.to_end();
            }
            BoardAct::Back => Screen::back(self, cx),
        }
    }
}

/// Draws the stage passes of `f` (the kit's order, `stage::passes`): the backdrop, the composed ground with the lamps'
/// light, the front face, then the board's props, the decor and the units sorted by their feet.
pub fn draw_stage(ui: &mut Ui<BoardAct>, art: &mut aegis_ui::art::Art, f: &Fight) {
    let (s, w) = (&f.s, &f.s.w);
    let geo = Geom::new(View::Stage, &SIZES);
    let look = stage::zone_look(art, STAGE_SET, &s.def.zone);
    let zset = (STAGE_SET, &look);
    let mut items = stage::board_items(art, &w.grid);
    items.extend(stage::decor_items(art, &w.grid, zset, &s.def.decor));
    let mut bodies = Vec::new();
    for (i, u) in w.units.iter().enumerate() {
        let Some(c) = w.grid.pos(i).filter(|_| u.is_active()) else { continue };
        let r = geo.token_rect(c, 0, 0);
        bodies.push((i, c, r));
        items.push(stage::unit_item(i, geo.foot(c), r));
    }
    let order = stage::feet_order(&items);
    let no_zones = indexmap::IndexMap::new();
    ui.p.clip(Some(stage::STAGE_CLIP));
    for pass in stage::passes() {
        match pass {
            Pass::Backdrop => stage::draw_backdrop(&mut ui.p, art, &w.grid, &s.def.zone, &no_zones),
            Pass::Ground => {
                stage::draw_ground(&mut ui.p, art, &w.grid, zset, 0);
                stage::draw_floor_courses(&mut ui.p, &w.grid);
                let spots = stage::light_spots(art, STAGE_SET, &s.def.zone);
                let lamps = stage::lamps_with(art, &w.grid, zset, &s.def.decor);
                stage::draw_light(&mut ui.p, &spots, &lamps);
                stage::draw_dividers(&mut ui.p, 0);
                stage::draw_exit_gates(&mut ui.p, art, &w.grid.exits);
            }
            Pass::Face => stage::draw_face(&mut ui.p, art, &w.grid, zset),
            Pass::Feet => {
                for p in &order {
                    match p.item {
                        Item::Block(c) => stage::draw_block(&mut ui.p, art, &w.grid, zset, c),
                        Item::Prop(c) => stage::draw_prop(&mut ui.p, art, &w.grid, c),
                        Item::Decor(k) => stage::draw_decor(&mut ui.p, art, &w.grid, zset, &s.def.decor[k]),
                        Item::Unit(i) => {
                            let Some(&(_, c, r)) = bodies.iter().find(|b| b.0 == i) else { continue };
                            let foe = w.enemies.contains(&i);
                            if let Some(img) = art.body_clip_flipped(&f.art[i], "", "idle", 0, r.w, foe) {
                                ui.p.image_tinted(img, r, stage::depth_tint(c.y as f32, false, None), 255);
                            }
                        }
                        Item::Fx(_) => {}
                    }
                }
            }
            _ => {}
        }
    }
    ui.p.clip(None);
}

impl Screen<KitSample> for Board {
    fn id(&self) -> &'static str {
        "board"
    }
    fn frame_need(&self, _cx: &GCx) -> FrameNeed {
        FrameNeed::Idle
    }
    fn draw(&mut self, cx: &mut GCx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        if cx.app.fight.is_none() {
            cx.app.fight = cx.app.data.as_ref().and_then(Fight::start);
        }
        let mut ui: Ui<BoardAct> = Ui::new(g, mouse, live);
        let (bg, text, dim) = (ui.t.bg, ui.t.text, ui.t.dim);
        ui.p.fill(Rect::new(0, 0, aegis_ui::ui::W, aegis_ui::ui::H), bg);
        match cx.app.fight.as_ref() {
            Some(f) => {
                let head = if f.over() { format!("{}: {}", f.s.def.name, if f.s.outcome.is_empty() { "over" } else { f.s.outcome.as_str() }) } else { f.s.def.name.clone() };
                ui.p.text(100.0, 16.0, &head, HEAD, text);
                // who is who: the party left of the dash, the foes right, each with its HP
                let names = |ids: &[usize]| ids.iter().map(|&i| format!("{} {}", f.s.w.units[i].name, f.s.w.units[i].hp)).collect::<Vec<_>>().join(", ");
                let line = format!("{}  -  {}", names(&f.s.w.allies), names(&f.s.w.enemies));
                ui.p.text(100.0, 312.0, &line, BODY, dim);
                draw_stage(&mut ui, &mut cx.art, f);
                let over = f.over();
                ui.button("board.round", Rect::new(100, 326, 70, 18), &tr("board.round"), BoardAct::Round, !over);
                ui.button("board.to_end", Rect::new(176, 326, 70, 18), &tr("board.to_end"), BoardAct::ToEnd, !over);
            }
            None => {
                ui.p.text(100.0, 40.0, &tr("board.no_data"), BODY, dim);
            }
        }
        ui.button("board.back", Rect::new(470, 326, 70, 18), &tr("board.back"), BoardAct::Back, true);
        self.w = ui.widgets;
    }
    fn state(&self, cx: &GCx) -> Value {
        json!({"started": cx.app.started, "fight": cx.app.fight.as_ref().map(Fight::state)})
    }
    fn back(&mut self, cx: &mut GCx) {
        play(cx, sound::BACK);
        cx.app.fight = None;
        cx.nav.push(Nav::Pop);
    }
    widget_plumbing!(GCx);
}
