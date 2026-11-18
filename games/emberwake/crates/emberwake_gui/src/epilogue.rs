//! The epilogue screen (PR6-11, `docs/sessions/PK6_PK7_BOSSES_ENDINGS.md` sections 4-5; engine-only, no Godot
//! original): the run-over screen's finale branch. Once an end of kind `ending` completed the campaign, it replaces
//! every other screen (`runover::map_tick`): the ending's art (else the end's card) plays once as a vignette, then the
//! epilogue as pages: one per paragraph, the world's outcome lines, and the member fate cards four to a page; the last
//! page offers Load a save and Main menu. Like the run-over screen, Esc and right click do nothing and no slot takes a
//! save (`campaign::save_refusal` still refuses while the run is over). The rules are the kit's
//! (`emberwake_world::campaign::ending`); this module only shows them. Story-free: every text beyond its own labels
//! comes from the data.

use crate::hud::tone_colour;
use crate::runover::{button_w, lines, play_card, LOAD_SAVE, MAIN_MENU};
use crate::screens::{Cx, GameCx, Key, Nav, Screen};
use aegis_ui::canvas::{rgb, rgba, Rect, Rgba};
use aegis_ui::theme::th;
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, H, HEAD, SMALL, TITLE, W};
use aegis_ui::widget_plumbing;
use emberwake_world::campaign::ending::{self, Epilogue, FateCard};
use serde_json::{json, Value};

/// The epilogue's backdrop (dark, a cold cast).
const EPILOGUE_FILL: Rgba = rgb(0x0c, 0x0e, 0x14);
/// The fate cards a page holds (two rows of two).
pub const FATES_PER_PAGE: usize = 4;
/// The panel's margin to the canvas, its padding and the gap between cards.
const MARGIN: i32 = 20;
const PAD: i32 = 10;
const GAP: i32 = 6;
/// The title when the ending row is gone (the end's own text stands alone).
pub const DONE_TITLE: &str = "The campaign is complete";
/// The page headings and the button labels.
pub const OUTCOMES_HEAD: &str = "Outcomes";
pub const FATES_HEAD: &str = "Fates";
pub const BACK: &str = "Back";
pub const NEXT: &str = "Next";
/// A departed member's tag on its card.
pub const DEPARTED_TAG: &str = "Departed";

#[derive(Debug, Clone)]
pub enum EpilogueAct {
    Back,
    Next,
    LoadSave,
    MainMenu,
}

/// One page of the epilogue.
#[derive(Debug, Clone, PartialEq)]
enum Page {
    Paragraph(String),
    Outcomes,
    Fates(usize),
}

/// The finale: the ending's pages, then Load a save and Main menu.
pub struct EpilogueScreen {
    ending: String,
    epilogue: Epilogue,
    /// The vignette played as the screen opens (the ending's art, else the end's card).
    card: String,
    played: bool,
    page: usize,
    pages: Vec<Page>,
    w: Vec<Widget<EpilogueAct>>,
}

impl EpilogueScreen {
    pub fn new(cx: &Cx) -> Self {
        let over = cx.game.as_ref().and_then(|g| g.gs.campaign.as_ref()).and_then(|c| c.over.clone());
        let ending_id = over.as_ref().map(|o| o.ending.clone()).unwrap_or_default();
        let found = cx.game.as_ref().and_then(|g| ending::epilogue(&cx.data, &g.gs, &ending_id));
        let epilogue = found.unwrap_or_else(|| Epilogue {
            title: DONE_TITLE.into(),
            tone: "calm".into(),
            paragraphs: over.as_ref().map(|o| o.text.clone()).filter(|t| !t.trim().is_empty()).into_iter().collect(),
            ..Epilogue::default()
        });
        let card = if epilogue.art.is_empty() { over.as_ref().map(|o| o.card.clone()).unwrap_or_default() } else { epilogue.art.clone() };
        let mut pages: Vec<Page> = epilogue.paragraphs.iter().cloned().map(Page::Paragraph).collect();
        if !epilogue.outcomes.is_empty() {
            pages.push(Page::Outcomes);
        }
        for i in (0..epilogue.fates.len()).step_by(FATES_PER_PAGE) {
            pages.push(Page::Fates(i));
        }
        if pages.is_empty() {
            pages.push(Page::Paragraph(String::new()));
        }
        Self { ending: ending_id, epilogue, card, played: false, page: 0, pages, w: Vec::new() }
    }

    fn last(&self) -> bool {
        self.page + 1 >= self.pages.len()
    }

    fn act(&mut self, cx: &mut Cx, a: EpilogueAct) {
        match a {
            EpilogueAct::Back => self.page = self.page.saturating_sub(1),
            EpilogueAct::Next => self.page = (self.page + 1).min(self.pages.len() - 1),
            EpilogueAct::LoadSave => cx.nav.push(Nav::Push(Box::new(crate::menus::Slots::new(false)))),
            EpilogueAct::MainMenu => {
                cx.end_game();
                cx.nav.push(Nav::Reset(Box::new(crate::menus::MainMenu::default())));
            }
        }
    }

    /// The fate cards of page `from`.
    fn fates(&self, from: usize) -> &[FateCard] {
        let all = &self.epilogue.fates;
        &all[from.min(all.len())..(from + FATES_PER_PAGE).min(all.len())]
    }

    /// Draws one fate card in `r`: the name, the departed tag and the stage, the template's line and the keepsake.
    fn draw_card(ui: &mut Ui<EpilogueAct>, r: Rect, f: &FateCard, rows: &CardLines) {
        let (tl, sl, bl) = (ui.p.line_h(TITLE), ui.p.line_h(SMALL), ui.p.line_h(BODY));
        let border = if f.departed { th().dim } else { th().good };
        ui.p.panel(r, th().game("epilogue_card", rgba(22, 24, 32, 255)), border, 3.0);
        let x = (r.x + PAD / 2) as f32;
        let mut y = (r.y + PAD / 2) as f32;
        ui.p.text(x, y, &rows.name, TITLE, th().text);
        y += tl;
        for l in &rows.tag {
            ui.p.text(x, y, l, SMALL, if f.departed { th().dim } else { th().good });
            y += sl;
        }
        for l in &rows.text {
            ui.p.text(x, y, l, BODY, th().text);
            y += bl;
        }
        for l in &rows.keepsake {
            ui.p.text(x, y, l, SMALL, th().gold);
            y += sl;
        }
    }
}

/// A fate card's wrapped lines.
struct CardLines {
    name: String,
    tag: Vec<String>,
    text: Vec<String>,
    keepsake: Vec<String>,
}

/// The tag line of a fate card: "Departed (cause)" and the stage's name, or the stage alone.
pub fn fate_tag(f: &FateCard) -> String {
    let stage = if !f.stage_name.is_empty() { f.stage_name.clone() } else if f.stage != 0 { format!("Stage {}", f.stage) } else { String::new() };
    let mut parts = Vec::new();
    if f.departed {
        parts.push(if f.cause.is_empty() { DEPARTED_TAG.to_string() } else { format!("{DEPARTED_TAG} ({})", f.cause) });
    }
    if !stage.is_empty() {
        parts.push(stage);
    }
    parts.join(" - ")
}

/// The keepsake line of a fate card ("" = none).
pub fn keepsake_line(f: &FateCard) -> String {
    if f.keepsake.is_empty() {
        String::new()
    } else {
        format!("Keepsake: {}", f.keepsake)
    }
}

impl Screen<GameCx> for EpilogueScreen {
    fn id(&self) -> &'static str {
        "epilogue"
    }

    fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<EpilogueAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("epilogue_fill", EPILOGUE_FILL));
        let tone = tone_colour(&self.epilogue.tone);
        let r = Rect::new(MARGIN, MARGIN, W - 2 * MARGIN, H - 2 * MARGIN);
        ui.p.panel(r, th().panel, tone, 4.0);
        let inner = r.w - 2 * PAD;
        let (hl, tl, bl, sl) = (ui.p.line_h(HEAD), ui.p.line_h(TITLE), ui.p.line_h(BODY), ui.p.line_h(SMALL));
        let title = lines(&mut ui, &self.epilogue.title, HEAD, inner as f32);
        let mut y = (r.y + PAD) as f32;
        for l in &title {
            ui.p.text_center((r.x + r.w / 2) as f32, y, l, HEAD, tone);
            y += hl;
        }
        y += 6.0;
        let left = (r.x + PAD) as f32;
        match self.pages[self.page].clone() {
            Page::Paragraph(text) => {
                for l in lines(&mut ui, &text, BODY, inner as f32) {
                    ui.p.text(left, y, &l, BODY, th().text);
                    y += bl;
                }
            }
            Page::Outcomes => {
                ui.p.text(left, y, OUTCOMES_HEAD, TITLE, tone);
                y += tl + 2.0;
                for o in self.epilogue.outcomes.clone() {
                    for l in lines(&mut ui, &o, BODY, (inner - 8) as f32) {
                        ui.p.text(left + 8.0, y, &l, BODY, th().text);
                        y += bl;
                    }
                    y += 3.0;
                }
            }
            Page::Fates(from) => {
                ui.p.text(left, y, FATES_HEAD, TITLE, tone);
                y += tl + 2.0;
                let cw = (inner - GAP) / 2;
                let tw = (cw - PAD) as f32;
                let cards: Vec<FateCard> = self.fates(from).to_vec();
                let rows: Vec<CardLines> = cards
                    .iter()
                    .map(|f| CardLines { name: f.name.clone(), tag: lines(&mut ui, &fate_tag(f), SMALL, tw), text: lines(&mut ui, &f.text, BODY, tw), keepsake: lines(&mut ui, &keepsake_line(f), SMALL, tw) })
                    .collect();
                let height = |c: &CardLines| (PAD as f32 + tl + (c.tag.len() + c.keepsake.len()) as f32 * sl + c.text.len() as f32 * bl).ceil() as i32;
                let mut top = y.ceil() as i32;
                for pair in (0..cards.len()).step_by(2) {
                    let h = (pair..(pair + 2).min(cards.len())).map(|i| height(&rows[i])).max().unwrap_or(0);
                    for i in pair..(pair + 2).min(cards.len()) {
                        let x = r.x + PAD + (i - pair) as i32 * (cw + GAP);
                        Self::draw_card(&mut ui, Rect::new(x, top, cw, h), &cards[i], &rows[i]);
                    }
                    top += h + GAP;
                }
            }
        }
        // the page count and the buttons, along the panel's foot
        let bh = (bl + 6.0).ceil() as i32;
        let by = r.y + r.h - PAD - bh;
        let count = format!("{} / {}", self.page + 1, self.pages.len());
        ui.p.text(left, (by + (bh - sl.ceil() as i32) / 2) as f32, &count, SMALL, th().dim);
        let mut buttons: Vec<(&str, &str, EpilogueAct, bool)> = vec![("epilogue.back", BACK, EpilogueAct::Back, self.page > 0)];
        if self.last() {
            buttons.push(("epilogue.load_save", LOAD_SAVE, EpilogueAct::LoadSave, true));
            buttons.push(("epilogue.main_menu", MAIN_MENU, EpilogueAct::MainMenu, true));
        } else {
            buttons.push(("epilogue.next", NEXT, EpilogueAct::Next, true));
        }
        let widths: Vec<i32> = buttons.iter().map(|b| button_w(&mut ui, b.1)).collect();
        let mut x = r.x + r.w - PAD - widths.iter().sum::<i32>() - GAP * (widths.len() as i32 - 1);
        for (b, bw) in buttons.into_iter().zip(widths) {
            let card = match b.2 {
                EpilogueAct::LoadSave => tip(b.1, "The save slots and the autosave written before the end."),
                _ => tip(b.1, ""),
            };
            ui.button_tip(b.0, Rect::new(x, by, bw, bh), b.1, b.2, b.3, card);
            x += bw + GAP;
        }
        self.w = ui.widgets;
    }

    /// F3: no way back into the run (Esc and right click do nothing).
    fn back(&mut self, _cx: &mut Cx) {}

    fn auto_focus(&self) -> bool {
        true
    }

    fn key(&mut self, cx: &mut Cx, k: Key) {
        match k {
            Key::Right | Key::PageDown if !self.last() => self.act(cx, EpilogueAct::Next),
            Key::Left | Key::PageUp => self.act(cx, EpilogueAct::Back),
            _ => {}
        }
    }

    fn tick(&mut self, cx: &mut Cx) {
        if !self.played {
            self.played = true;
            play_card(cx, &self.card);
        }
    }

    fn state(&self, _cx: &Cx) -> Value {
        let page = match &self.pages[self.page] {
            Page::Paragraph(_) => "paragraph",
            Page::Outcomes => "outcomes",
            Page::Fates(_) => "fates",
        };
        json!({"ending": self.ending, "title": self.epilogue.title, "tone": self.epilogue.tone, "card": self.card, "page": self.page, "pages": self.pages.len(), "page_kind": page,
               "outcomes": self.epilogue.outcomes, "fates": self.epilogue.fates.iter().map(|f| json!({"member": f.member, "name": f.name, "departed": f.departed, "tag": fate_tag(f), "text": f.text, "keepsake": f.keepsake})).collect::<Vec<_>>()})
    }

    fn music(&self, _cx: &Cx) -> Option<String> {
        Some(String::new())
    }

    widget_plumbing!();
}
