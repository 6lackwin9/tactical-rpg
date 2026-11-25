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
        Self::with(ending_id, epilogue, card)
    }

    /// ENGINE-25: the screen of ending `ending_id` with its epilogue and vignette, paged.
    pub(crate) fn with(ending_id: String, epilogue: Epilogue, card: String) -> Self {
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

    /// ENGINE-25: the epilogue, the open page's index and the page count.
    pub(crate) fn epilogue(&self) -> &Epilogue {
        &self.epilogue
    }

    pub(crate) fn page_at(&self) -> (usize, usize) {
        (self.page, self.pages.len())
    }

    /// The open page's kind ("paragraph", "outcomes", "fates"), its paragraph ("" for another kind) and its fate cards
    /// (none for another kind).
    pub(crate) fn page_kind(&self) -> &'static str {
        match &self.pages[self.page] {
            Page::Paragraph(_) => "paragraph",
            Page::Outcomes => "outcomes",
            Page::Fates(_) => "fates",
        }
    }

    pub(crate) fn page_text(&self) -> String {
        match &self.pages[self.page] {
            Page::Paragraph(t) => t.clone(),
            _ => String::new(),
        }
    }

    pub(crate) fn page_fates(&self) -> &[FateCard] {
        match &self.pages[self.page] {
            Page::Fates(from) => self.fates(*from),
            _ => &[],
        }
    }

    pub(crate) fn last(&self) -> bool {
        self.page + 1 >= self.pages.len()
    }

    pub(crate) fn act(&mut self, cx: &mut Cx, a: EpilogueAct) {
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

/// The Load a save button's hover line.
pub(crate) const LOAD_SAVE_NOTE: &str = "The save slots and the autosave written before the end.";

/// The page count along the foot ("2 / 5").
pub(crate) fn page_count(page: usize, pages: usize) -> String {
    format!("{} / {pages}", page + 1)
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
        let count = page_count(self.page, self.pages.len());
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
                EpilogueAct::LoadSave => tip(b.1, LOAD_SAVE_NOTE),
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
        let page = self.page_kind();
        json!({"view": crate::epilogue_view::view(self).to_json(), "ending": self.ending, "title": self.epilogue.title, "tone": self.epilogue.tone, "card": self.card, "page": self.page, "pages": self.pages.len(), "page_kind": page,
               "outcomes": self.epilogue.outcomes, "fates": self.epilogue.fates.iter().map(|f| json!({"member": f.member, "name": f.name, "departed": f.departed, "tag": fate_tag(f), "text": f.text, "keepsake": f.keepsake})).collect::<Vec<_>>()})
    }

    /// AMBIENCE1: no place's loop under this screen.
    fn ambience(&self, _cx: &Cx) -> Option<String> {
        Some(String::new())
    }

    fn music(&self, _cx: &Cx) -> Option<String> {
        Some(String::new())
    }

    widget_plumbing!();
}

// ---- Milestone 1: Act 1 end-of-slice screen ----

/// Act 1 completion card copy.
pub const ACT1_TITLE: &str = "Act 1 Complete";
pub const ACT1_SUBTITLE: &str = "To Be Continued";
pub const ACT1_CONTINUE: &str = "Continue Exploring";
pub const ACT1_MAIN_MENU: &str = "Main Menu";
pub const ACT1_OPEN_NOTE: &str = "Act 2 systems and the road to the Slag Gate are open to explore.";
const CRUCIBLE_STAYER_FLAG: &str = "sunk_ward.crucible_stayed";

#[derive(Debug, Clone)]
pub enum ActCompleteAct {
    Continue,
    MainMenu,
}

/// Milestone 1 end-of-slice screen shown over the live `RegionMap` when Act 1 advances into Act 2.
/// Unlike `EpilogueScreen`, the campaign continues: Esc, right-click, or **Continue Exploring** pops
/// back into the live map with saving still enabled; **Main Menu** autosaves and returns to the title.
pub struct ActCompleteScreen {
    days: i64,
    members: Vec<(String, i64)>,
    stayer: String,
    departed: Vec<(String, String)>,
    w: Vec<Widget<ActCompleteAct>>,
}

impl ActCompleteScreen {
    pub fn new(cx: &Cx) -> Self {
        let Some(g) = cx.game.as_ref() else {
            return Self { days: 0, members: Vec::new(), stayer: String::new(), departed: Vec::new(), w: Vec::new() };
        };
        let gs = &g.gs;
        let days = gs.time.max(0);
        let members: Vec<(String, i64)> = gs
            .party
            .members
            .iter()
            .map(|m| (emberwake_world::progression::display_name(&cx.data, m), m.level(&cx.data)))
            .collect();
        let stayer_id = gs.flags.get(CRUCIBLE_STAYER_FLAG).and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
        let stayer = if stayer_id.is_empty() {
            String::new()
        } else if let Some(dep) = gs.party.departed.iter().find(|d| d.member.id == stayer_id) {
            emberwake_world::progression::display_name(&cx.data, &dep.member)
        } else if let Some(m) = gs.party.member(&stayer_id) {
            emberwake_world::progression::display_name(&cx.data, m)
        } else {
            let ch = cx.data.resolve(&gs.world_id, "characters", &stayer_id);
            aegis_core::json::s_or(ch, "name", &stayer_id)
        };
        let departed: Vec<(String, String)> = gs
            .party
            .departed
            .iter()
            .filter(|d| stayer_id.is_empty() || d.member.id != stayer_id)
            .map(|d| {
                let name = emberwake_world::progression::display_name(&cx.data, &d.member);
                let why = if !d.cause.is_empty() { d.cause.clone() } else { d.life.clone() };
                (name, why)
            })
            .collect();
        Self { days, members, stayer, departed, w: Vec::new() }
    }

    fn act(&mut self, cx: &mut Cx, a: ActCompleteAct) {
        match a {
            ActCompleteAct::Continue => cx.nav.push(Nav::Pop),
            ActCompleteAct::MainMenu => {
                cx.autosave();
                cx.end_game();
                cx.nav.push(Nav::Reset(Box::new(crate::menus::MainMenu::default())));
            }
        }
    }

    fn party_line(&self) -> String {
        if self.members.is_empty() {
            "Surviving party: none".into()
        } else {
            let list = self.members.iter().map(|(n, lv)| format!("{n} (Lv {lv})")).collect::<Vec<_>>().join(", ");
            format!("Surviving party: {list}")
        }
    }

    fn departed_line(&self) -> String {
        if self.departed.is_empty() {
            String::new()
        } else {
            let list = self
                .departed
                .iter()
                .map(|(n, why)| if why.is_empty() { n.clone() } else { format!("{n} ({why})") })
                .collect::<Vec<_>>()
                .join(", ");
            format!("Departed: {list}")
        }
    }
}

impl Screen<GameCx> for ActCompleteScreen {
    fn id(&self) -> &'static str {
        "act_complete"
    }

    fn draw(&mut self, _cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<ActCompleteAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("epilogue_fill", EPILOGUE_FILL));
        let r = Rect::new(MARGIN, MARGIN, W - 2 * MARGIN, H - 2 * MARGIN);
        ui.p.panel(r, th().panel, th().gold, 4.0);
        let inner = r.w - 2 * PAD;
        let (hl, tl, bl, sl) = (ui.p.line_h(HEAD), ui.p.line_h(TITLE), ui.p.line_h(BODY), ui.p.line_h(SMALL));
        let center_x = (r.x + r.w / 2) as f32;
        let left = (r.x + PAD) as f32;
        let mut y = (r.y + PAD) as f32;

        for l in lines(&mut ui, ACT1_TITLE, HEAD, inner as f32) {
            ui.p.text_center(center_x, y, &l, HEAD, th().gold);
            y += hl;
        }
        for l in lines(&mut ui, ACT1_SUBTITLE, TITLE, inner as f32) {
            ui.p.text_center(center_x, y, &l, TITLE, th().dim);
            y += tl;
        }
        y += 8.0;

        let days_text = format!("Days elapsed: {} (Day {})", self.days, self.days + 1);
        let party_text = self.party_line();
        let stayer_text = if self.stayer.is_empty() { String::new() } else { format!("Held the winch: {} stayed behind at the Crucible Watchpost.", self.stayer) };
        let dep_text = self.departed_line();

        let card_pad = PAD / 2;
        let card_w = inner;
        let tw = (card_w - 2 * card_pad) as f32;
        let days_lines = lines(&mut ui, &days_text, BODY, tw);
        let party_lines = lines(&mut ui, &party_text, BODY, tw);
        let stayer_lines = lines(&mut ui, &stayer_text, BODY, tw);
        let dep_lines = lines(&mut ui, &dep_text, SMALL, tw);
        let card_h = (2 * card_pad) as f32
            + (days_lines.len() + party_lines.len() + stayer_lines.len()) as f32 * bl
            + dep_lines.len() as f32 * sl
            + if !party_lines.is_empty() { 4.0 } else { 0.0 }
            + if !stayer_lines.is_empty() { 4.0 } else { 0.0 }
            + if !dep_lines.is_empty() { 4.0 } else { 0.0 };
        let card_rect = Rect::new(r.x + PAD, y.ceil() as i32, card_w, card_h.ceil() as i32);
        ui.p.panel(card_rect, th().game("epilogue_card", rgba(22, 24, 32, 255)), th().dim, 3.0);

        let cx_text = (card_rect.x + card_pad) as f32;
        let mut cy = (card_rect.y + card_pad) as f32;
        for l in &days_lines {
            ui.p.text(cx_text, cy, l, BODY, th().text);
            cy += bl;
        }
        if !party_lines.is_empty() {
            cy += 4.0;
            for l in &party_lines {
                ui.p.text(cx_text, cy, l, BODY, th().good);
                cy += bl;
            }
        }
        if !stayer_lines.is_empty() {
            cy += 4.0;
            for l in &stayer_lines {
                ui.p.text(cx_text, cy, l, BODY, th().gold);
                cy += bl;
            }
        }
        if !dep_lines.is_empty() {
            cy += 4.0;
            for l in &dep_lines {
                ui.p.text(cx_text, cy, l, SMALL, th().dim);
                cy += sl;
            }
        }

        let mut ny = (card_rect.y + card_rect.h) as f32 + 10.0;
        for l in lines(&mut ui, ACT1_OPEN_NOTE, BODY, inner as f32) {
            ui.p.text(left, ny, &l, BODY, th().text);
            ny += bl;
        }

        let bh = (bl + 6.0).ceil() as i32;
        let by = r.y + r.h - PAD - bh;
        let buttons = [
            ("act_complete.continue", ACT1_CONTINUE, ActCompleteAct::Continue, "Return to the live map and keep playing."),
            ("act_complete.main_menu", ACT1_MAIN_MENU, ActCompleteAct::MainMenu, "Autosave and return to the main menu."),
        ];
        let widths: Vec<i32> = buttons.iter().map(|b| button_w(&mut ui, b.1)).collect();
        let mut x = r.x + r.w - PAD - widths.iter().sum::<i32>() - GAP * (widths.len() as i32 - 1);
        for (b, bw) in buttons.into_iter().zip(widths) {
            ui.button_tip(b.0, Rect::new(x, by, bw, bh), b.1, b.2, true, tip(b.1, b.3));
            x += bw + GAP;
        }
        self.w = ui.widgets;
    }

    fn back(&mut self, cx: &mut Cx) {
        self.act(cx, ActCompleteAct::Continue);
    }

    fn auto_focus(&self) -> bool {
        true
    }

    fn state(&self, _cx: &Cx) -> Value {
        json!({
            "title": ACT1_TITLE,
            "subtitle": ACT1_SUBTITLE,
            "days": self.days,
            "members": self.members.iter().map(|(n, lv)| json!({"name": n, "level": lv})).collect::<Vec<_>>(),
            "stayer": self.stayer,
            "departed": self.departed.iter().map(|(n, why)| json!({"name": n, "cause": why})).collect::<Vec<_>>(),
            "note": ACT1_OPEN_NOTE,
        })
    }

    fn ambience(&self, _cx: &Cx) -> Option<String> {
        Some(String::new())
    }

    widget_plumbing!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::hud_tests::{frame, gfx, rect};
    use crate::screens::Game;
    use crate::story::tests::story_cx;
    use aegis_core::json as ajson;
    use aegis_ui::painter::TextRec;
    use aegis_ui::screen::TEXT_SIZES;
    use emberwake_world::{campaign, new_game, state::Departed, Data};

    fn act_complete_cx() -> Cx {
        let catalog = json!({
            "classes": [{"id": "fighter", "name": "Fighter"}],
            "units": [
                {"id": "hero", "name": "Ash", "side": "party", "class_id": "fighter", "level": 5},
                {"id": "mate", "name": "Kaelen", "side": "party", "class_id": "fighter", "level": 4},
                {"id": "stayer", "name": "Bram", "side": "party", "class_id": "fighter", "level": 4},
                {"id": "fallen", "name": "Orla", "side": "party", "class_id": "fighter", "level": 3}
            ]
        });
        let worlds = json!({
            "w": {"manifest": {"id": "w", "name": "W", "start_location": "hub", "start_party": ["hero", "mate"], "campaign": {"start_act": 1}}, "own": {
                "locations": [{"id": "hub", "name": "Hub", "_world": "w", "_qid": "w:hub"}]
            }}
        });
        let d = Data::from_values(&catalog, &worlds, &json!({}));
        let mut cx = story_cx(d);
        let mut gs = new_game::create_for_world(&cx.data, "w", 1).expect("new game");
        gs.time = 11;
        gs.campaign.as_mut().unwrap().act = 2;
        let stayer_m = emberwake_world::state::UnitState::at_level(&cx.data, "stayer", 4, "stayer_u");
        let fallen_m = emberwake_world::state::UnitState::at_level(&cx.data, "fallen", 3, "fallen_u");
        gs.flags.insert(CRUCIBLE_STAYER_FLAG.into(), json!("stayer_u"));
        gs.party.departed.push(Departed { member: stayer_m, life: "captured".into(), cause: "crucible".into(), encounter: String::new(), day: 11, keepsake: String::new() });
        gs.party.departed.push(Departed { member: fallen_m, life: "dead".into(), cause: "combat".into(), encounter: String::new(), day: 8, keepsake: String::new() });
        cx.game = Some(Game::new(gs, -1, false));
        cx
    }

    #[test]
    fn act_complete_renders_clean_at_every_text_size_and_keeps_saving_open() {
        let mut cx = act_complete_cx();
        let mut screen = ActCompleteScreen::new(&cx);
        let st = screen.state(&cx);
        assert_eq!(st["title"], json!(ACT1_TITLE));
        assert_eq!(st["subtitle"], json!(ACT1_SUBTITLE));
        assert_eq!(st["days"], json!(11));
        assert_eq!(st["stayer"], json!("Bram"));
        assert_eq!(st["departed"], json!([{"name": "Orla", "cause": "combat"}]));
        assert_eq!(campaign::save_refusal(cx.gs()), "", "saving stays enabled on ActCompleteScreen");

        let mut g = gfx();
        for (size, scale) in TEXT_SIZES {
            let recs: Vec<TextRec> = frame(&mut screen, &mut cx, &mut g, scale as f32);
            let boxes: Vec<(String, f32, f32, f32, f32)> = screen
                .widgets()
                .iter()
                .map(|w| {
                    let (x, y, ww, hh) = rect(w);
                    (ajson::s(ajson::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
                })
                .collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            assert_eq!(v["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "act_complete at {size}: {v}");
        }
    }

    #[test]
    fn act_complete_continue_pops_and_main_menu_ends_game() {
        let mut cx = act_complete_cx();
        let mut g = gfx();
        let mut screen = ActCompleteScreen::new(&cx);
        frame(&mut screen, &mut cx, &mut g, 1.0);

        // Esc / back pops back into live RegionMap.
        screen.back(&mut cx);
        assert!(matches!(cx.nav.pop(), Some(Nav::Pop)), "Esc pops back to RegionMap");
        assert!(cx.has_game(), "game stays live after Esc");

        // Continue Exploring button pops back into live RegionMap.
        screen.click_id(&mut cx, "act_complete.continue").expect("Continue Exploring");
        assert!(matches!(cx.nav.pop(), Some(Nav::Pop)), "Continue Exploring pops back to RegionMap");
        assert!(cx.has_game(), "game stays live after Continue Exploring");
        assert_eq!(campaign::save_refusal(cx.gs()), "", "saving stays enabled");

        // Main Menu button autosaves, ends the game, and resets to MainMenu.
        screen.click_id(&mut cx, "act_complete.main_menu").expect("Main Menu");
        match cx.nav.pop() {
            Some(Nav::Reset(s)) => assert_eq!(s.id(), "main_menu"),
            other => panic!("expected Reset(main_menu), got {:?}", other.map(|_| "nav")),
        }
        assert!(!cx.has_game(), "Main Menu ends the active game");
    }
}
