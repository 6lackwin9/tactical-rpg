//! The kit's widgets on one page (UIGALLERY1): every widget family of `ui.rs` in its states, on sample values, to be
//! judged on its own (a UI Gallery's "Kit widgets" entry). Game-agnostic: the gallery screen keeps a `Specimens`,
//! wraps `KitAct` in its own action and forwards drops and text changes to it.

use crate::canvas::{Rect, Rgba};
use crate::ui::{tip, Ui, BODY, DIM, GOLD, GOOD, INFO, SMALL, TEXT};

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
}

pub const TABS: [&str; 3] = ["First", "Second", "Third"];
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
    /// The drag and drop specimen: the chips in its two boxes.
    pub left: Vec<String>,
    pub right: Vec<String>,
    pub last: String,
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
            left: vec!["Sword".into(), "Shield".into()],
            right: vec!["Potion".into()],
            last: "nothing yet".into(),
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
        };
        match a {
            KitAct::Tab(i) => self.tab = i % TABS.len(),
            KitAct::Check => self.checked = !self.checked,
            KitAct::Step(d) => self.value = (self.value + d).clamp(0, 9),
            KitAct::Pick(i) => self.pick = i % OPTIONS.len(),
            KitAct::Slide(i) => self.slide = i.min(SLIDER_STEPS - 1),
            KitAct::Card(i) => self.card = i,
            KitAct::Row(i) => self.row = i,
            KitAct::Press(_) | KitAct::Close => {}
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
        if id == "kit.text" {
            self.text = text.to_string();
        }
    }

    /// Draws the page in `r`: two columns of named groups. Widget ids start with `kit.`.
    pub fn draw<A: Clone>(&self, ui: &mut Ui<A>, r: Rect, wrap: impl Fn(KitAct) -> A) {
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
        y = head(ui, x2, y, "Scroll area (the wheel over it)");
        let view = Rect::new(x2, y, col_w, (r.y + r.h - 18 - y).max(24));
        ui.p.panel(view, crate::ui::CARD, crate::ui::BORDER, 2.0);
        let line = ui.p.line_h(SMALL).ceil() as i32 + 1;
        let off = ui.scroll_begin("kit.scroll", view, 12 * line + 4);
        for i in 0..12 {
            ui.p.text((view.x + 4) as f32, (view.y + 2 - off + i * line) as f32, &format!("Line {} of a list taller than its area", i + 1), SMALL, TEXT);
        }
        ui.scroll_end();

        ui.p.text_fit(x1 as f32, (r.y + r.h - 12) as f32, &format!("Last: {}", self.last), SMALL, (r.w - 12) as f32, GOOD);
    }
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
