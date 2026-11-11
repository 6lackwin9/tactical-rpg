//! The playtest note box (PLAYTEST1 F8 notes): the shell captures the screen, then opens this box over it. The text
//! field (the kit's text editor, PLAYUI1-T: selection, clipboard, undo) types right away and fills the box; four tags
//! (mechanic, ui, ux, bug); Save (Ctrl+Enter) writes the note and its screenshot into the run's recording, Cancel
//! (Esc) drops it. The screen under it gets no input while it is open, and does not
//! tick (a game's auto-play pauses).

use crate::canvas::Rect;
use crate::playtest::TAGS;
use crate::screen::{Cx, Key, Nav, Screen};
use crate::ui::{Gfx, Ui, Widget, BODY, H, W};
use crate::widget_plumbing;
use serde_json::{json, Value};

pub const ID: &str = "note";
/// The note's text field (the shell starts editing it when the box opens).
pub const TEXT_ID: &str = "note.text";
pub const MAX_TEXT: usize = 2000;

#[derive(Clone)]
enum Act {
    Tag(usize),
    Save,
    Cancel,
}

pub struct NoteBox {
    text: String,
    tags: [bool; 4],
    /// The screen as it was when F8 was pressed: (width, height, RGBA).
    shot: Option<(usize, usize, Vec<u8>)>,
    /// What the note records about where it was taken (screen, mouse, viewport, control, the screen's context).
    stamp: Value,
    pub message: String,
    w: Vec<Widget<Act>>,
}

impl NoteBox {
    pub fn new(shot: Option<(usize, usize, Vec<u8>)>, stamp: Value) -> Self {
        Self { text: String::new(), tags: [false; 4], shot, stamp, message: String::new(), w: Vec::new() }
    }

    fn act<G>(&mut self, cx: &mut Cx<G>, a: Act) {
        match a {
            Act::Tag(i) => self.tags[i] = !self.tags[i],
            Act::Cancel => cx.nav.push(Nav::Pop),
            Act::Save => {
                let text = self.text.trim();
                if text.is_empty() {
                    self.message = "Write the note first.".into();
                    return;
                }
                let tags: Vec<&str> = TAGS.iter().zip(self.tags).filter(|(_, on)| *on).map(|(t, _)| *t).collect();
                let mut note = self.stamp.clone();
                note["text"] = json!(text.chars().take(MAX_TEXT).collect::<String>());
                note["tags"] = json!(tags);
                match cx.playtest.add_note(note, self.shot.take()) {
                    Ok(n) => {
                        cx.notice.push(format!("Note {n} saved."));
                        cx.nav.push(Nav::Pop);
                    }
                    Err(e) => self.message = e,
                }
            }
        }
    }
}

impl<G> Screen<G> for NoteBox {
    fn id(&self) -> &'static str {
        ID
    }
    fn overlay(&self) -> bool {
        true
    }
    fn draw(&mut self, _cx: &mut Cx<G>, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<Act> = Ui::new(g, mouse, live);
        // PLAYUI1-T: the text area fills the box (at least 160 tall at Normal); the tags, the hint and the buttons sit
        // under it.
        let r = Rect::new(W / 2 - 190, 24, 380, H - 48);
        ui.frame(r, "Playtest note", false);
        let foot = 66;
        ui.text_field(TEXT_ID, Rect::new(r.x + 10, r.y + 22, r.w - 20, r.h - 22 - foot), &self.text, true, true);
        let tags_y = r.y + r.h - foot + 8;
        for (i, tag) in TAGS.iter().enumerate() {
            ui.checkbox(&format!("note.tag.{tag}"), r.x + 10 + i as i32 * 62, tags_y, tag, self.tags[i], Act::Tag(i), true);
        }
        let hint = if self.message.is_empty() { "Ctrl+Enter saves, Esc cancels; Ctrl+C, X, V copy, cut, paste; Ctrl+Z undoes." } else { self.message.as_str() };
        ui.p.text_fit(r.x as f32 + 10.0, (tags_y + 18) as f32, hint, BODY, (r.w - 20) as f32, if self.message.is_empty() { ui.t.dim } else { ui.t.text });
        ui.button("note.cancel", Rect::new(r.x + r.w - 124, r.y + r.h - 22, 54, 15), "Cancel", Act::Cancel, true);
        ui.button("note.save", Rect::new(r.x + r.w - 64, r.y + r.h - 22, 54, 15), "Save", Act::Save, true);
        self.w = ui.widgets;
    }
    fn key(&mut self, cx: &mut Cx<G>, k: Key) {
        match k {
            Key::CtrlEnter => self.act(cx, Act::Save),
            Key::Esc => self.act(cx, Act::Cancel),
            _ => {}
        }
    }
    fn text_changed(&mut self, _cx: &mut Cx<G>, id: &str, text: &str) {
        if id == TEXT_ID {
            self.text = text.to_string();
        }
    }
    fn state(&self, _cx: &Cx<G>) -> Value {
        json!({"text": self.text, "tags": TAGS.iter().zip(self.tags).filter(|(_, on)| *on).map(|(t, _)| *t).collect::<Vec<_>>(), "message": self.message})
    }
    widget_plumbing!(Cx<G>);
}
