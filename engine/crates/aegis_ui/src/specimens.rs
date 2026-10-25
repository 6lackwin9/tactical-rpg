//! The kit's widgets on one page (UIGALLERY1): every widget family of `ui.rs` in its states, on sample values, to be
//! judged on its own (a UI Gallery's "Kit widgets" entry). Game-agnostic: the gallery screen keeps a `Specimens`,
//! wraps `KitAct` in its own action and forwards drops and text changes to it. Two pages, picked by the tabs at the
//! page's foot: the widgets, and the block timeline with its segment parts, links and marks (RULESET1-E).

use crate::canvas::{rgb, Image, Rect, Rgba};
use crate::timeline::{Link, Mark, Part, PartStyle, Section, Segment, Timeline, Track};
use crate::ui::{tip, Glyph, Ui, BODY, DIM, FOE, GOLD, GOOD, INFO, SMALL, TEXT};

#[derive(Debug, Clone, PartialEq)]
pub enum KitAct {
    Press(&'static str),
    Tab(usize),
    Check,
    Step(i32),
    Pick(usize),
    Slide(usize),
    Card(usize),
    Row(usize),
    Close,
    /// The lock icon button (toggles), any other icon button (its name).
    Lock,
    Icon(&'static str),
    /// The collapsible section header.
    Fold,
    /// The page tabs at the foot.
    Page(usize),
    /// The timeline specimen: a block, a track, a segment (track, segment).
    Block(i32),
    Track(String),
    Segment(String, String),
}

pub const TABS: [&str; 3] = ["First", "Second", "Third"];
pub const PAGES: [&str; 2] = ["Widgets", "Timeline"];
pub const OPTIONS: [&str; 4] = ["Small", "Normal", "Large", "Extra large"];
pub const SLIDER_STEPS: usize = 5;

/// The page's sample values; what the last widget used did is shown at the foot of the page.
#[derive(Debug, Clone)]
pub struct Specimens {
    pub tab: usize,
    pub checked: bool,
    pub value: i32,
    pub pick: usize,
    pub slide: usize,
    pub card: usize,
    pub row: usize,
    pub text: String,
    /// The text area's sample (the kit's text editor, PLAYUI1-T).
    pub area: String,
    /// The icon buttons' lock (on: the delete icon is disabled) and whether the section is open.
    pub locked: bool,
    pub open: bool,
    /// The drag and drop specimen: the chips in its two boxes.
    pub left: Vec<String>,
    pub right: Vec<String>,
    pub last: String,
    /// The page on view (`PAGES`), and the timeline specimen's selected block and track.
    pub page: usize,
    pub block: Option<i32>,
    pub track: Option<String>,
}

impl Default for Specimens {
    fn default() -> Self {
        Self {
            tab: 0,
            checked: true,
            value: 3,
            pick: 1,
            slide: 2,
            card: 0,
            row: 1,
            text: "Type here".into(),
            area: "A text area wraps its lines and scrolls. Shift with the arrows selects, so does a drag; a double click takes a word.\nCtrl with Left or Right jumps by word; Ctrl+Z undoes, Ctrl+Y redoes.".into(),
            locked: false,
            open: true,
            left: vec!["Sword".into(), "Shield".into()],
            right: vec!["Potion".into()],
            last: "nothing yet".into(),
            page: 0,
            block: None,
            track: None,
        }
    }
}

impl Specimens {
    pub fn act(&mut self, a: KitAct) {
        self.last = match &a {
            KitAct::Press(what) => format!("pressed {what}"),
            KitAct::Tab(i) => format!("tab {}", TABS[*i % TABS.len()]),
            KitAct::Check => format!("checkbox {}", if self.checked { "off" } else { "on" }),
            KitAct::Step(d) => format!("stepper {}", (self.value + d).clamp(0, 9)),
            KitAct::Pick(i) => format!("select {}", OPTIONS[*i % OPTIONS.len()]),
            KitAct::Slide(i) => format!("slider step {i}"),
            KitAct::Card(i) => format!("card {}", i + 1),
            KitAct::Row(i) => format!("row {}", i + 1),
            KitAct::Close => "close".into(),
            KitAct::Lock => format!("lock {}", if self.locked { "off" } else { "on" }),
            KitAct::Icon(what) => format!("icon {what}"),
            KitAct::Fold => format!("section {}", if self.open { "folded" } else { "opened" }),
            KitAct::Page(i) => format!("page {}", PAGES[*i % PAGES.len()]),
            KitAct::Block(b) => format!("block {b}"),
            KitAct::Track(t) => format!("track {t}"),
            KitAct::Segment(t, s) => format!("segment {s} on {t}"),
        };
        match a {
            KitAct::Tab(i) => self.tab = i % TABS.len(),
            KitAct::Check => self.checked = !self.checked,
            KitAct::Step(d) => self.value = (self.value + d).clamp(0, 9),
            KitAct::Pick(i) => self.pick = i % OPTIONS.len(),
            KitAct::Slide(i) => self.slide = i.min(SLIDER_STEPS - 1),
            KitAct::Card(i) => self.card = i,
            KitAct::Row(i) => self.row = i,
            KitAct::Lock => self.locked = !self.locked,
            KitAct::Fold => self.open = !self.open,
            KitAct::Page(i) => self.page = i % PAGES.len(),
            KitAct::Block(b) => self.block = Some(b),
            KitAct::Track(t) => self.track = Some(t),
            KitAct::Press(_) | KitAct::Close | KitAct::Icon(_) | KitAct::Segment(..) => {}
        }
    }

    /// A chip (`chip.<name>`) onto the other box (`box.left` / `box.right`).
    pub fn can_drop(&self, from: &str, to: &str) -> bool {
        let Some(chip) = from.strip_prefix("chip.") else { return false };
        match to {
            "box.left" => self.right.iter().any(|c| c == chip),
            "box.right" => self.left.iter().any(|c| c == chip),
            _ => false,
        }
    }

    pub fn accept_drop(&mut self, from: &str, to: &str) {
        if !self.can_drop(from, to) {
            return;
        }
        let chip = from.trim_start_matches("chip.").to_string();
        let (src, dst) = if to == "box.left" { (&mut self.right, &mut self.left) } else { (&mut self.left, &mut self.right) };
        src.retain(|c| *c != chip);
        dst.push(chip.clone());
        self.last = format!("dropped {chip} on the {} box", to.trim_start_matches("box."));
    }

    pub fn text_changed(&mut self, id: &str, text: &str) {
        match id {
            "kit.text" => self.text = text.to_string(),
            "kit.area" => self.area = text.to_string(),
            _ => {}
        }
    }

    /// Draws the page on view in `r`, the page tabs (`kit.page.<i>`) and what happened last at its foot. Widget ids
    /// start with `kit.`.
    pub fn draw<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: impl Fn(KitAct) -> A) {
        let mut tx = r.x + r.w - 6;
        for (i, p) in PAGES.iter().enumerate().rev() {
            let w = (ui.p.text_w(p, BODY) + 12.0).ceil() as i32;
            tx -= w;
            ui.tab(&format!("kit.page.{i}"), Rect::new(tx, r.y + r.h - 15, w, 14), p, wrap(KitAct::Page(i)), self.page == i);
            tx -= 2;
        }
        if self.page == 1 {
            self.draw_timeline(ui, Rect::new(r.x, r.y, r.w, r.h - 18), &wrap);
        } else {
            self.draw_widgets(ui, r, &wrap);
        }
        ui.p.text_fit((r.x + 6) as f32, (r.y + r.h - 12) as f32, &format!("Last: {}", self.last), SMALL, (tx - r.x - 10) as f32, GOOD);
    }

    /// The timeline page: a sample round on two sides, every part, a whole-hatched segment, links, marks (one on a
    /// masked track), then a legend.
    fn draw_timeline<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(KitAct) -> A) {
        let x = r.x + 6;
        let w = r.w - 12;
        ui.p.text(x as f32, (r.y + 4) as f32, "Timeline: segment parts, hatching, links and marks (hover for the cards; clicks select)", SMALL, INFO);
        let y = r.y + 4 + ui.p.line_h(SMALL).ceil() as i32 + 2;
        let t = sample_timeline(self.block, self.track.clone());
        let h = (crate::timeline::RULER_H + 2 + t.content_h()).min(r.y + r.h - y - 30);
        t.draw(ui, "kit.timeline", Rect::new(x, y, w, h), |b| wrap(KitAct::Block(b)), |k| wrap(KitAct::Track(k.to_string())), |k, s| wrap(KitAct::Segment(k.to_string(), s.to_string())));
        ui.para(
            x,
            y + h + 6,
            w,
            "A windup is shaded, an impact bright with an edge where it lands, a recovery hatched; a segment can be hatched whole. A link joins one segment's end to the next one's start. A mark is a glyph in a colour with a card, inside its block; it takes no click, the text beside it makes room, and it shows on a masked track too.",
            SMALL,
            DIM,
        );
    }

    /// The widgets page: two columns of named groups.
    fn draw_widgets<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: &impl Fn(KitAct) -> A) {
        let col_w = (r.w - 18) / 2;
        let (x1, x2) = (r.x + 6, r.x + 12 + col_w);
        let gap = 5;
        let head = |ui: &mut Ui<A>, x: i32, y: i32, t: &str| -> i32 {
            ui.p.text(x as f32, y as f32, t, SMALL, INFO);
            y + ui.p.line_h(SMALL).ceil() as i32 + 1
        };
        let bw = |ui: &mut Ui<A>, t: &str| (ui.p.text_w(t, BODY) + 12.0).ceil() as i32;

        // -- Left column
        let mut y = r.y + 4;
        y = head(ui, x1, y, "Buttons: normal, selected, disabled, with a hover card");
        let mut x = x1;
        for (id, label, active, enabled) in [("kit.button", "Button", false, true), ("kit.button.active", "Selected", true, true), ("kit.button.off", "Disabled", false, false)] {
            let w = bw(ui, label);
            ui.button_full(id, Rect::new(x, y, w, 14), label, wrap(KitAct::Press(label_key(id))), enabled, active, BODY, Vec::new());
            x += w + 4;
        }
        let w = bw(ui, "Hover me");
        ui.button_tip("kit.button.tip", Rect::new(x, y, w, 14), "Hover me", wrap(KitAct::Press("hover card")), true, tip("Hover card", "Every widget can carry one: a title line and its text, wrapped."));
        y += 14 + gap;
        y = head(ui, x1, y, "Tabs");
        let mut x = x1;
        for (i, t) in TABS.iter().enumerate() {
            let w = bw(ui, t);
            ui.tab(&format!("kit.tab.{i}"), Rect::new(x, y, w, 14), t, wrap(KitAct::Tab(i)), self.tab == i);
            x += w + 2;
        }
        y += 14 + gap;
        y = head(ui, x1, y, "Checkbox and stepper");
        ui.checkbox("kit.check", x1, y + 2, if self.checked { "Checked" } else { "Not checked" }, self.checked, wrap(KitAct::Check), true);
        ui.stepper("kit.stepper", Rect::new(x1 + 80, y, 130, 14), self.value as f32 / 9.0, &self.value.to_string(), wrap(KitAct::Step(-1)), wrap(KitAct::Step(1)), true);
        y += 14 + gap;
        y = head(ui, x1, y, "Dropdown (a menu) and select (a choice list)");
        let half = (col_w - 4) / 2;
        ui.dropdown("kit.dropdown", Rect::new(x1, y, half, 14), "Dropdown", wrap(KitAct::Press("dropdown")), true, tip("Dropdown", "Opens a menu of its own (the screen draws it)."));
        let options: Vec<String> = OPTIONS.iter().map(|o| o.to_string()).collect();
        ui.select("kit.select", Rect::new(x1 + half + 4, y, half, 14), &options, self.pick, |i| wrap(KitAct::Pick(i)), true, tip("Select", "Opens its choice list; arrows and Enter work too."));
        y += 14 + gap;
        y = head(ui, x1, y, "Slider");
        ui.slider("kit.slider", Rect::new(x1, y, col_w.min(160), 14), self.slide, SLIDER_STEPS, &format!("Step {} of {}", self.slide + 1, SLIDER_STEPS), |i| wrap(KitAct::Slide(i)), true);
        y += 14 + gap;
        y = head(ui, x1, y, "Text field (click to type)");
        ui.text_field("kit.text", Rect::new(x1, y, col_w.min(180), 14), &self.text, false, true);
        y += 14 + gap;
        y = head(ui, x1, y, "Text area: Up / Down, Shift selects, Ctrl+arrows by word, Ctrl+A, C, X, V, Z, Y");
        let ah = (ui.p.line_h(BODY) * 4.0).ceil() as i32 + 6;
        ui.text_field("kit.area", Rect::new(x1, y, col_w, ah), &self.area, true, true);
        y += ah + gap;
        y = head(ui, x1, y, "Frame, paragraph and close button");
        let fr = Rect::new(x1, y, col_w.min(200), (r.y + r.h - 18 - y).clamp(46, 70));
        let body_y = ui.frame(fr, "Frame", false);
        ui.close_x("kit.close", Rect::new(fr.x + fr.w - 16, fr.y + 4, 12, 12), wrap(KitAct::Close));
        ui.para(fr.x + 6, body_y, fr.w - 12, "A paragraph wraps to its width and returns where it ends, so what follows sits under it.", SMALL, TEXT);

        // -- Right column
        let mut y = r.y + 4;
        y = head(ui, x2, y, "Cards (click to select)");
        let cw = (col_w - 4) / 2;
        for i in 0..2 {
            let lines: Vec<(String, Rgba)> = vec![(format!("Card {}", i + 1), GOLD), ("Two lines of text".into(), TEXT)];
            ui.card(&format!("kit.card.{i}"), Rect::new(x2 + i as i32 * (cw + 4), y, cw, 26), &lines, Some(wrap(KitAct::Card(i))), self.card == i, true, Vec::new());
        }
        y += 26 + gap;
        y = head(ui, x2, y, "Rows: clickable, selected, info (hover card only)");
        for i in 0..2 {
            ui.row(&format!("kit.row.{i}"), Rect::new(x2, y, col_w, 13), &format!("Row {}", i + 1), if self.row == i { GOLD } else { TEXT }, Some(wrap(KitAct::Row(i))), self.row == i, Vec::new());
            y += 14;
        }
        ui.row("kit.row.info", Rect::new(x2, y, col_w, 13), "Info row", DIM, None, false, tip("Info row", "No action: hovering shows its card and an outline."));
        y += 13 + gap;
        y = head(ui, x2, y, "Drag and drop (drag a chip to the other box)");
        let bh = 34;
        for (side, chips, bx) in [("left", &self.left, x2), ("right", &self.right, x2 + cw + 4)] {
            let b = Rect::new(bx, y, cw, bh);
            ui.p.panel(b, crate::ui::CARD, crate::ui::BORDER, 2.0);
            ui.push(format!("kit.box.{side}"), "drop", format!("{side} box"), b, true, false, None, Vec::new());
            ui.drop_target(format!("box.{side}"));
            let mut cx = b.x + 3;
            for c in chips {
                let w = bw(ui, c);
                let chip = Rect::new(cx, b.y + 3, w, 12);
                ui.button_full(&format!("kit.chip.{c}"), chip, c, wrap(KitAct::Press("chip")), true, false, SMALL, Vec::new());
                ui.draggable(format!("chip.{c}"));
                cx += w + 3;
            }
        }
        y += bh + gap;
        y = head(ui, x2, y, "Icon buttons: image, drawn shapes (file missing), toggled, disabled");
        let ib = |i: i32| Rect::new(x2 + i * 19, y, 16, 16);
        let px = ui.icon_px(ib(0));
        let sample = sample_icon(px);
        ui.icon_button("kit.icon.image", ib(0), "Image icon", Some(&sample), Glyph::Letter('i'), wrap(KitAct::Icon("image")), true, false, tip("Image icon", "An image from the asset layers; a drawn shape stands in while the file is missing."));
        let (lock, lock_tip) = if self.locked { ("Unlock", "Unlock") } else { ("Lock", "Lock (protect from overwrite and delete)") };
        ui.icon_button("kit.icon.lock", ib(1), lock, None, if self.locked { Glyph::Lock } else { Glyph::Unlock }, wrap(KitAct::Lock), true, self.locked, tip(lock_tip, ""));
        let del_tip = if self.locked { tip("Delete", "Unlock it to delete it.") } else { Vec::new() };
        ui.icon_button("kit.icon.delete", ib(2), "Delete", None, Glyph::Delete, wrap(KitAct::Icon("delete")), !self.locked, false, del_tip);
        ui.icon_button("kit.icon.help", ib(3), "Help", None, Glyph::Help, wrap(KitAct::Icon("help")), true, false, tip("Help (?)", "A hover card can list the keys."));
        ui.icon_button("kit.icon.menu", ib(4), "Menu", None, Glyph::Menu, wrap(KitAct::Icon("menu")), true, false, Vec::new());
        ui.icon_button("kit.icon.letter", ib(5), "Party", None, Glyph::Letter('P'), wrap(KitAct::Icon("party")), true, false, tip("Party (P)", "A letter stands in for an icon with no shape of its own."));
        ui.icon_button("kit.icon.off", ib(6), "Disabled", Some(&sample), Glyph::Letter('i'), wrap(KitAct::Icon("disabled")), false, false, tip("Disabled", "Dimmed; takes no click."));
        y += 16 + gap;
        y = head(ui, x2, y, "Section header (click to fold)");
        let open = ui.section("kit.section", Rect::new(x2, y, col_w, 14), "Section", "2 rows", self.open, wrap(KitAct::Fold), tip("Section", "A click folds or opens it; the screen keeps the state."));
        y += 14 + 1;
        if open {
            for i in 0..2 {
                ui.row(&format!("kit.section.row.{i}"), Rect::new(x2 + 8, y, col_w - 8, 12), &format!("Content row {}", i + 1), TEXT, None, false, Vec::new());
                y += 13;
            }
        }
        y += gap - 1;
        y = head(ui, x2, y, "Scroll area (the wheel over it)");
        let view = Rect::new(x2, y, col_w, (r.y + r.h - 18 - y).max(24));
        ui.p.panel(view, crate::ui::CARD, crate::ui::BORDER, 2.0);
        let line = ui.p.line_h(SMALL).ceil() as i32 + 1;
        let off = ui.scroll_begin("kit.scroll", view, 12 * line + 4);
        for i in 0..12 {
            ui.p.text((view.x + 4) as f32, (view.y + 2 - off + i * line) as f32, &format!("Line {} of a list taller than its area", i + 1), SMALL, TEXT);
        }
        ui.scroll_end();
    }
}

/// The timeline specimen's round: 8 blocks, three tracks on one side and two on the other (one masked).
fn sample_timeline(block: Option<i32>, track: Option<String>) -> Timeline {
    let part = |start, end, style| Part { start, end, style };
    let seg = |key: &str, start, end, label: &str, fill: Rgba, parts: Vec<Part>, card: &str| Segment { key: key.into(), start, end, label: label.into(), fill, parts, tip: tip(label, card), ..Segment::default() };
    let mark = |key: &str, block, glyph: &str, colour, title: &str, body: &str| Mark { key: key.into(), block, glyph: glyph.into(), colour, tip: tip(title, body) };
    let link = |from: &str, to: &str, colour, body: &str| Link { from: from.into(), to: to.into(), colour, tip: tip("Link", body) };
    let (blue, dim) = (rgb(70, 110, 170), rgb(44, 56, 78));
    let red = rgb(160, 70, 64);
    let lead = Track {
        key: "lead".into(),
        label: "Lead".into(),
        colour: TEXT,
        segments: vec![
            seg("strike", 1, 3, "Strike", blue, vec![part(1, 2, PartStyle::Windup), part(3, 3, PartStyle::Impact)], "Windup B1-B2 (shaded), impact B3 (bright, edged)."),
            seg("follow", 4, 5, "Follow-up", blue, vec![part(4, 4, PartStyle::Windup), part(5, 5, PartStyle::Impact)], "Linked to the strike before it."),
            seg("rest", 6, 8, "Rest", dim, Vec::new(), "A plain segment: no parts."),
        ],
        links: vec![link("strike", "follow", GOLD, "Chained: the follow-up starts as the strike lands.")],
        marks: vec![mark("gain", 8, "+", GOOD, "Mark", "A glyph in a colour at one block, with its card.")],
        ..Track::default()
    };
    let scout = Track {
        key: "scout".into(),
        label: "Scout".into(),
        colour: TEXT,
        segments: vec![
            Segment { dashed: true, ..seg("move", 1, 2, "Move", rgb(60, 80, 110), Vec::new(), "Dashed: a planned move.") },
            seg("shot", 3, 6, "Shot", blue, vec![part(3, 5, PartStyle::Windup), part(6, 6, PartStyle::Impact)], "A long windup, impact at B6."),
        ],
        links: vec![link("move", "shot", INFO, "Carried: the shot follows the move.")],
        marks: vec![mark("step", 2, ">", INFO, "Step", "A mark on the move's last block.")],
        ..Track::default()
    };
    let warden = Track {
        key: "warden".into(),
        label: "Warden".into(),
        colour: TEXT,
        segments: vec![
            seg("hold", 1, 4, "Hold", rgb(60, 120, 100), Vec::new(), "A plain segment."),
            Segment { hatched: true, ..seg("dropped", 5, 8, "Dropped", rgb(60, 120, 100), Vec::new(), "Hatched whole: a span that will not happen.") },
        ],
        marks: vec![mark("break", 5, "x", FOE, "Broken", "A mark at a segment's first block: its label starts after it."), mark("note", 5, "i", INFO, "Note", "Two marks in one block sit side by side.")],
        ..Track::default()
    };
    let brute = Track {
        key: "brute".into(),
        label: "Brute".into(),
        colour: FOE,
        segments: vec![seg("smash", 1, 7, "Heavy smash", red, vec![part(1, 3, PartStyle::Windup), part(4, 4, PartStyle::Impact), part(5, 7, PartStyle::Recovery)], "Windup B1-B3, impact B4, recovery B5-B7 (hatched).")],
        marks: vec![mark("impact", 4, "!", FOE, "Impact", "A badge where something lands.")],
        ..Track::default()
    };
    let lurker = Track {
        key: "lurker".into(),
        label: "Lurker".into(),
        colour: FOE,
        segments: vec![seg("hidden", 1, 8, "Hidden", red, Vec::new(), "")],
        masked: Some("Plans in secret".into()),
        marks: vec![mark("impact", 4, "!", FOE, "Impact", "Marks show on a masked track: something lands here, the rest is hidden."), mark("guess", 7, "?", DIM, "Unknown", "")],
        ..Track::default()
    };
    Timeline {
        blocks: 8,
        playhead: 0,
        selected_block: block,
        selected_track: track,
        sections: vec![Section { title: "Allies".into(), colour: INFO, tracks: vec![lead, scout, warden] }, Section { title: "Enemies".into(), colour: FOE, tracks: vec![brute, lurker] }],
    }
}

/// The image icon specimen: a gold diamond with a dark outline, `n` pixels square (an icon as the art would give it).
fn sample_icon(n: i32) -> Image {
    let n = n.max(4);
    let mut px = vec![0u8; (n * n * 4) as usize];
    let c = (n - 1) as f32 / 2.0;
    for y in 0..n {
        for x in 0..n {
            let d = (x as f32 - c).abs() + (y as f32 - c).abs();
            let col = if d <= c * 0.7 { [244, 198, 84, 255] } else if d <= c * 0.95 { [70, 52, 20, 255] } else { [0, 0, 0, 0] };
            px[((y * n + x) * 4) as usize..((y * n + x) * 4 + 4) as usize].copy_from_slice(&col);
        }
    }
    Image { w: n, h: n, px }
}

fn label_key(id: &str) -> &'static str {
    match id {
        "kit.button.active" => "the selected button",
        "kit.button.off" => "the disabled button",
        _ => "the button",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chip_moves_to_the_other_box_only() {
        let mut k = Specimens::default();
        assert!(k.can_drop("chip.Sword", "box.right"));
        assert!(!k.can_drop("chip.Sword", "box.left"), "already in the left box");
        assert!(!k.can_drop("Sword", "box.right"), "not a chip");
        k.accept_drop("chip.Sword", "box.right");
        assert_eq!((k.left.clone(), k.right.clone()), (vec!["Shield".to_string()], vec!["Potion".to_string(), "Sword".to_string()]));
        k.accept_drop("chip.Sword", "box.right");
        assert_eq!(k.right.len(), 2, "a refused drop changes nothing");
    }

    #[test]
    fn actions_update_the_sample_values_and_say_what_happened() {
        let mut k = Specimens::default();
        k.act(KitAct::Check);
        assert!(!k.checked);
        k.act(KitAct::Step(10));
        assert_eq!((k.value, k.last.as_str()), (9, "stepper 9"), "the stepper stops at 9");
        k.act(KitAct::Pick(3));
        assert_eq!((k.pick, k.last.as_str()), (3, "select Extra large"));
        k.act(KitAct::Slide(99));
        assert_eq!(k.slide, SLIDER_STEPS - 1);
    }
}
