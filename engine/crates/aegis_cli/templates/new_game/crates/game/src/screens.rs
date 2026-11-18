//! The screens: the main menu and the play screen. Both draw on the native grid (1280 x 720 units, `Grid::Native`)
//! with the engine's widgets and the theme in the asset layers; their words are in `assets/strings/en.json` (`tr`).
//! Replace `Play` with the game.

use crate::{play, sound, __TYPE__};
use aegis_layout::{Anchor, Insets};
use aegis_ui::painter::Grid;
use aegis_ui::pacing::FrameNeed;
use aegis_ui::screen::{Cx, Nav, Screen};
use aegis_ui::strings::tr;
use aegis_ui::ui::{Gfx, Ui, Widget};
use aegis_ui::widget_plumbing;
use serde_json::{json, Value};

type GCx = Cx<__TYPE__>;

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
                cx.nav.push(Nav::Push(Box::new(Play::default())));
            }
            MenuAct::Quit => cx.quit = true,
        }
    }
}

impl Screen<__TYPE__> for Menu {
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
        let title = "__TITLE__";
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

// ---- the play screen ----

#[derive(Clone, Copy)]
pub enum PlayAct {
    Back,
}

/// Where the game starts: a panel saying so, and Back (or Esc) to the menu.
#[derive(Default)]
pub struct Play {
    w: Vec<Widget<PlayAct>>,
}

impl Play {
    fn act(&mut self, cx: &mut GCx, a: PlayAct) {
        match a {
            PlayAct::Back => Screen::back(self, cx),
        }
    }
}

impl Screen<__TYPE__> for Play {
    fn id(&self) -> &'static str {
        "play"
    }
    fn grid(&self) -> Grid {
        Grid::Native
    }
    fn frame_need(&self, _cx: &GCx) -> FrameNeed {
        FrameNeed::Idle
    }
    fn draw(&mut self, _cx: &mut GCx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<PlayAct> = Ui::new(g, mouse, live);
        let (m, area) = (ui.m(), ui.screen());
        let (bg, text) = (ui.t.bg, ui.t.text);
        ui.p.fill(area, bg);
        let panel = area.place(Anchor::Center, m.u(240), m.u(90));
        let mut body = ui.frame_body("panel", panel).pad(Insets::all(m.u(4)));
        let back = body.cut_bottom(m.u(12));
        ui.para(body.x, body.y, body.w, &tr("play.body"), m.body, text);
        ui.button("play.back", back.place(Anchor::Right, m.u(40), back.h), &tr("play.back"), PlayAct::Back, true);
        self.w = ui.widgets;
    }
    fn state(&self, cx: &GCx) -> Value {
        json!({"started": cx.app.started})
    }
    fn back(&mut self, cx: &mut GCx) {
        play(cx, sound::BACK);
        cx.nav.push(Nav::Pop);
    }
    widget_plumbing!(GCx);
}
