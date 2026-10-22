//! The kit's text editor (PLAYUI1-T): the model of a text field being edited (`Edit`: caret, selection, moves by
//! character, word and visual line, typing, deletes, paste normalising, undo grouping), its layout in wrapped visual
//! lines (`Layout`), the editing keys (`EditKey`), the scroll that keeps the caret's line in view, and the clipboard
//! (`clipboard`: the system's through `arboard`, with an in-app fallback). Engine-only (no Godot counterpart). Pure
//! functions apart from the clipboard; `ui::text_field` draws with them and the shell maps keys and clicks through
//! them. Re-exported from `ui`.

use crate::canvas::Rect;
use crate::painter::Painter;
use crate::ui::BODY;

/// Undo steps one edit keeps (PLAYUI1-T: at most 100 per edit).
pub const UNDO_MAX: usize = 100;
/// A typing or deleting run stays one undo step until a pause this long (milliseconds of `Edit::now_ms`).
pub const UNDO_PAUSE_MS: u64 = 1000;

/// The kind of change a run of keys makes (typing, deleting); a run is one undo step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Run {
    Type,
    Delete,
}

/// The text, caret and selection an undo step brings back.
#[derive(Debug, Clone, PartialEq)]
struct Snap {
    text: String,
    caret: usize,
    anchor: Option<usize>,
}

/// A text field being edited (PLAYUI1-T, the kit's text editor): its id, the text so far, the caret (a character
/// index), the selection's other end, whether Enter adds a line, and its undo history. The shell types into it and
/// hands every change to the screen (`Screen::text_changed`), which owns the value. Every operation is a pure function
/// of the edit (and of the field's `Layout` for the moves by visual line); time enters only through `now_ms`.
#[derive(Debug, Clone, PartialEq)]
pub struct Edit {
    pub id: String,
    pub text: String,
    pub caret: usize,
    pub multiline: bool,
    /// Where the selection started (Shift with a move, a mouse drag); it runs from here to the caret. None: no
    /// selection.
    pub anchor: Option<usize>,
    /// The x Up and Down keep (the sticky column), until a horizontal move, a click or a change.
    pub goal_x: Option<f32>,
    /// The time of the next change in milliseconds (the shell's clock; tests set it), for undo grouping.
    pub now_ms: u64,
    undo: Vec<Snap>,
    redo: Vec<Snap>,
    /// The run the last change belongs to and when it was made; None after a move, so the next change is a new step.
    run: Option<(Run, u64)>,
}

/// A character's class for word moves: 0 space (and line breaks), 1 a word character, 2 punctuation.
fn class(c: char) -> u8 {
    if c.is_whitespace() {
        0
    } else if c.is_alphanumeric() || c == '_' {
        1
    } else {
        2
    }
}

/// Where Ctrl+Left goes from `p`: back over spaces, then over one run of word (or punctuation) characters.
pub fn word_start(chars: &[char], p: usize) -> usize {
    let mut p = p.min(chars.len());
    while p > 0 && class(chars[p - 1]) == 0 {
        p -= 1;
    }
    if p > 0 {
        let k = class(chars[p - 1]);
        while p > 0 && class(chars[p - 1]) == k {
            p -= 1;
        }
    }
    p
}

/// Where Ctrl+Right goes from `p`: over the run of word (or punctuation) characters there, then the spaces after it
/// (the start of the next word).
pub fn word_end(chars: &[char], p: usize) -> usize {
    let n = chars.len();
    let mut p = p.min(n);
    if p < n && class(chars[p]) != 0 {
        let k = class(chars[p]);
        while p < n && class(chars[p]) == k {
            p += 1;
        }
    }
    while p < n && class(chars[p]) == 0 {
        p += 1;
    }
    p
}

impl Edit {
    /// Editing `text` with the caret at its end.
    pub fn new(id: &str, text: &str, multiline: bool) -> Self {
        Self { id: id.into(), text: text.into(), caret: text.chars().count(), multiline, anchor: None, goal_x: None, now_ms: 0, undo: Vec::new(), redo: Vec::new(), run: None }
    }

    /// The text's length in characters.
    pub fn len(&self) -> usize {
        self.text.chars().count()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    fn chars(&self) -> Vec<char> {
        self.text.chars().collect()
    }

    fn byte_at(&self, caret: usize) -> usize {
        self.text.char_indices().nth(caret).map_or(self.text.len(), |(i, _)| i)
    }

    /// The selected characters `(start, end)`, start before end; None when nothing is selected.
    pub fn selection(&self) -> Option<(usize, usize)> {
        let a = self.anchor?.min(self.len());
        (a != self.caret).then(|| (a.min(self.caret), a.max(self.caret)))
    }

    /// The selected text ("" when nothing is selected).
    pub fn selected(&self) -> String {
        self.selection().map(|(a, b)| self.text.chars().skip(a).take(b - a).collect()).unwrap_or_default()
    }

    /// How many steps undo can go back and redo forward.
    pub fn history(&self) -> (usize, usize) {
        (self.undo.len(), self.redo.len())
    }

    /// Moves the caret to `pos`; `extend` (Shift, a mouse drag) grows the selection from where it started, otherwise
    /// the selection goes. A move ends a typing run (the next change is a new undo step).
    pub fn move_to(&mut self, pos: usize, extend: bool) {
        let pos = pos.min(self.len());
        if !extend {
            self.anchor = None;
        } else if self.anchor.is_none() {
            self.anchor = Some(self.caret);
        }
        self.caret = pos;
        self.run = None;
    }

    /// Left: one character back; without Shift a selection collapses to its start.
    pub fn left(&mut self, extend: bool) {
        self.goal_x = None;
        match (self.selection(), extend) {
            (Some((a, _)), false) => self.move_to(a, false),
            _ => self.move_to(self.caret.saturating_sub(1), extend),
        }
    }

    /// Right: one character on; without Shift a selection collapses to its end.
    pub fn right(&mut self, extend: bool) {
        self.goal_x = None;
        match (self.selection(), extend) {
            (Some((_, b)), false) => self.move_to(b, false),
            _ => self.move_to(self.caret + 1, extend),
        }
    }

    /// Ctrl+Left: to the start of the word before the caret.
    pub fn word_left(&mut self, extend: bool) {
        self.goal_x = None;
        let p = word_start(&self.chars(), self.caret);
        self.move_to(p, extend);
    }

    /// Ctrl+Right: to the start of the next word.
    pub fn word_right(&mut self, extend: bool) {
        self.goal_x = None;
        let p = word_end(&self.chars(), self.caret);
        self.move_to(p, extend);
    }

    /// Home: the start of the caret's visual line.
    pub fn home(&mut self, lay: &Layout, extend: bool) {
        self.goal_x = None;
        let l = lay.line_of(self.caret);
        self.move_to(lay.lines.get(l).map_or(0, |v| v.start), extend);
    }

    /// End: the end of the caret's visual line.
    pub fn end(&mut self, lay: &Layout, extend: bool) {
        self.goal_x = None;
        let p = if lay.lines.is_empty() { self.len() } else { lay.last_pos(lay.line_of(self.caret)) };
        self.move_to(p, extend);
    }

    /// Ctrl+Home: the start of the text.
    pub fn text_start(&mut self, extend: bool) {
        self.goal_x = None;
        self.move_to(0, extend);
    }

    /// Ctrl+End: the end of the text.
    pub fn text_end(&mut self, extend: bool) {
        self.goal_x = None;
        self.move_to(self.len(), extend);
    }

    /// Up (`by` < 0) or Down by `by` visual lines (Page Up and Down: a view's worth), keeping the column's x (sticky
    /// across vertical moves); past the first line it goes to the start, past the last to the end.
    pub fn vertical(&mut self, lay: &Layout, by: isize, extend: bool) {
        if lay.lines.is_empty() {
            return;
        }
        let x = self.goal_x.unwrap_or_else(|| lay.x_of(self.caret));
        let to = lay.line_of(self.caret) as isize + by;
        let pos = if to < 0 {
            0
        } else if to as usize >= lay.lines.len() {
            self.len()
        } else {
            lay.pos_at_x(to as usize, x)
        };
        self.move_to(pos, extend);
        self.goal_x = Some(x);
    }

    /// Ctrl+A: the whole text, the caret at its end.
    pub fn select_all(&mut self) {
        self.goal_x = None;
        self.run = None;
        self.anchor = Some(0);
        self.caret = self.len();
    }

    /// A double click: the word (or run of spaces or punctuation) at `pos`.
    pub fn select_word(&mut self, pos: usize) {
        let ch = self.chars();
        let n = ch.len();
        if n == 0 {
            return;
        }
        let mut i = pos.min(n - 1);
        if class(ch[i]) == 0 && i > 0 && class(ch[i - 1]) != 0 {
            i -= 1;
        }
        let k = class(ch[i]);
        let (mut a, mut b) = (i, i + 1);
        while a > 0 && class(ch[a - 1]) == k {
            a -= 1;
        }
        while b < n && class(ch[b]) == k {
            b += 1;
        }
        self.goal_x = None;
        self.run = None;
        self.anchor = Some(a);
        self.caret = b;
    }

    /// Replaces characters `from..to` with `s` (the caret after it, no selection). `run` is the kind of key run the
    /// change belongs to: it joins the previous undo step while that is the same run and less than UNDO_PAUSE_MS old,
    /// unless `fresh`. False when nothing changes.
    fn splice(&mut self, from: usize, to: usize, s: &str, run: Option<Run>, fresh: bool) -> bool {
        if from == to && s.is_empty() {
            return false;
        }
        let now = self.now_ms;
        let joins = !fresh && run.is_some() && self.run.is_some_and(|(r, t)| Some(r) == run && now.saturating_sub(t) < UNDO_PAUSE_MS);
        if !joins {
            self.undo.push(Snap { text: self.text.clone(), caret: self.caret, anchor: self.anchor });
            if self.undo.len() > UNDO_MAX {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
        let (a, b) = (self.byte_at(from), self.byte_at(to));
        self.text.replace_range(a..b, s);
        self.caret = from + s.chars().count();
        self.anchor = None;
        self.goal_x = None;
        self.run = run.map(|r| (r, now));
        true
    }

    /// The text as this field takes it: Windows line ends become one newline, a newline stays in a multi-line field
    /// (in a one-line field `join`, a paste, turns it into a space; typing drops it), other control characters go.
    pub fn clean(&self, s: &str, join: bool) -> String {
        let s = s.replace("\r\n", "\n").replace('\r', "\n");
        s.chars()
            .filter_map(|c| match c {
                '\n' if self.multiline => Some('\n'),
                '\n' if join => Some(' '),
                c if c.is_control() => None,
                c => Some(c),
            })
            .collect()
    }

    /// Typed text at the caret, replacing a selection; a typing run is one undo step.
    pub fn insert(&mut self, s: &str) {
        let t = self.clean(s, false);
        if t.is_empty() {
            return;
        }
        let sel = self.selection();
        let (a, b) = sel.unwrap_or((self.caret, self.caret));
        self.splice(a, b, &t, Some(Run::Type), sel.is_some());
    }

    /// A paste: the clipboard's text at the caret, replacing a selection; its own undo step.
    pub fn paste(&mut self, s: &str) {
        let t = self.clean(s, true);
        let sel = self.selection();
        if t.is_empty() && sel.is_none() {
            return;
        }
        let (a, b) = sel.unwrap_or((self.caret, self.caret));
        self.splice(a, b, &t, None, true);
    }

    /// Backspace: the selection, else the character before the caret.
    pub fn backspace(&mut self) {
        match self.selection() {
            Some((a, b)) => self.splice(a, b, "", Some(Run::Delete), true),
            None if self.caret > 0 => self.splice(self.caret - 1, self.caret, "", Some(Run::Delete), false),
            None => false,
        };
    }

    /// Delete: the selection, else the character after the caret.
    pub fn delete(&mut self) {
        match self.selection() {
            Some((a, b)) => self.splice(a, b, "", Some(Run::Delete), true),
            None if self.caret < self.len() => self.splice(self.caret, self.caret + 1, "", Some(Run::Delete), false),
            None => false,
        };
    }

    /// Ctrl+Backspace: the selection, else back to the start of the word before the caret.
    pub fn word_backspace(&mut self) {
        let (a, b) = self.selection().unwrap_or_else(|| (word_start(&self.chars(), self.caret), self.caret));
        self.splice(a, b, "", None, true);
    }

    /// Ctrl+Delete: the selection, else on to the start of the next word.
    pub fn word_delete(&mut self) {
        let (a, b) = self.selection().unwrap_or_else(|| (self.caret, word_end(&self.chars(), self.caret)));
        self.splice(a, b, "", None, true);
    }

    /// Ctrl+C: the selected text (None: nothing selected).
    pub fn copy(&self) -> Option<String> {
        self.selection().map(|_| self.selected())
    }

    /// Ctrl+X: the selected text, taken out of the field (its own undo step).
    pub fn cut(&mut self) -> Option<String> {
        let (a, b) = self.selection()?;
        let t = self.selected();
        self.splice(a, b, "", None, true);
        Some(t)
    }

    fn restore(&mut self, s: Snap) {
        self.text = s.text;
        self.caret = s.caret.min(self.len());
        self.anchor = s.anchor;
        self.goal_x = None;
        self.run = None;
    }

    /// Ctrl+Z: back one step (false when there is none).
    pub fn undo(&mut self) -> bool {
        let Some(s) = self.undo.pop() else { return false };
        self.redo.push(Snap { text: self.text.clone(), caret: self.caret, anchor: self.anchor });
        self.restore(s);
        true
    }

    /// Ctrl+Y (or Ctrl+Shift+Z): forward one undone step (false when there is none).
    pub fn redo(&mut self) -> bool {
        let Some(s) = self.redo.pop() else { return false };
        self.undo.push(Snap { text: self.text.clone(), caret: self.caret, anchor: self.anchor });
        self.restore(s);
        true
    }
}

/// A key that edits the text field being typed into (PLAYUI1-T); Ctrl and Shift change what most of them do
/// (`Edit::key`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKey {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Backspace,
    Delete,
    /// Ctrl+A.
    All,
    /// Ctrl+C.
    Copy,
    /// Ctrl+X.
    Cut,
    /// Ctrl+V.
    Paste,
    /// Ctrl+Z.
    Undo,
    /// Ctrl+Y or Ctrl+Shift+Z.
    Redo,
}

impl EditKey {
    /// An editing key by name, as the agent link sends it: `left`, `shift+left`, `ctrl+shift+end`, `backspace`,
    /// `ctrl+c`, `ctrl+z`... with whether Ctrl and Shift are held; None for any other key (Enter, Esc, Tab and Ctrl+Enter
    /// go to the screen).
    pub fn parse(name: &str) -> Option<(EditKey, bool, bool)> {
        let lower = name.to_lowercase();
        let mut parts: Vec<&str> = lower.split('+').collect();
        let key = parts.pop()?;
        let (mut ctrl, mut shift) = (false, false);
        for m in parts {
            match m {
                "ctrl" => ctrl = true,
                "shift" => shift = true,
                _ => return None,
            }
        }
        let k = match key {
            "left" => EditKey::Left,
            "right" => EditKey::Right,
            "up" => EditKey::Up,
            "down" => EditKey::Down,
            "home" => EditKey::Home,
            "end" => EditKey::End,
            "pageup" => EditKey::PageUp,
            "pagedown" => EditKey::PageDown,
            "backspace" => EditKey::Backspace,
            "delete" => EditKey::Delete,
            "a" if ctrl => EditKey::All,
            "c" if ctrl => EditKey::Copy,
            "x" if ctrl => EditKey::Cut,
            "v" if ctrl => EditKey::Paste,
            "z" if ctrl && shift => EditKey::Redo,
            "z" if ctrl => EditKey::Undo,
            "y" if ctrl => EditKey::Redo,
            _ => return None,
        };
        Some((k, ctrl, shift))
    }
}

impl Edit {
    /// Applies an editing key (Shift extends the selection with a move; Ctrl moves and deletes by word, Home and End
    /// go to the text's ends); `lay` is the field's layout and `rows` the lines it shows (Page Up and Down). `paste`
    /// gives the clipboard's text, asked only on Paste. Returns the text to put on the clipboard (Copy, Cut).
    #[allow(clippy::too_many_arguments)]
    pub fn key(&mut self, k: EditKey, ctrl: bool, shift: bool, lay: &Layout, rows: usize, paste: impl FnOnce() -> String) -> Option<String> {
        let page = rows.max(1) as isize;
        match k {
            EditKey::Left if ctrl => self.word_left(shift),
            EditKey::Left => self.left(shift),
            EditKey::Right if ctrl => self.word_right(shift),
            EditKey::Right => self.right(shift),
            EditKey::Up => self.vertical(lay, -1, shift),
            EditKey::Down => self.vertical(lay, 1, shift),
            EditKey::PageUp => self.vertical(lay, -page, shift),
            EditKey::PageDown => self.vertical(lay, page, shift),
            EditKey::Home if ctrl => self.text_start(shift),
            EditKey::Home => self.home(lay, shift),
            EditKey::End if ctrl => self.text_end(shift),
            EditKey::End => self.end(lay, shift),
            EditKey::Backspace if ctrl => self.word_backspace(),
            EditKey::Backspace => self.backspace(),
            EditKey::Delete if ctrl => self.word_delete(),
            EditKey::Delete => self.delete(),
            EditKey::All => self.select_all(),
            EditKey::Copy => return self.copy(),
            EditKey::Cut => return self.cut(),
            EditKey::Paste => self.paste(&paste()),
            EditKey::Undo => {
                self.undo();
            }
            EditKey::Redo => {
                self.redo();
            }
        }
        None
    }
}

/// One visual line of a text field (PLAYUI1-T): the characters `start..end` drawn on it and the x of every caret
/// position on it (`xs[i]` is the x before character `start + i`, so `xs.len() == end - start + 1`).
#[derive(Debug, Clone, PartialEq)]
pub struct VLine {
    pub start: usize,
    pub end: usize,
    pub xs: Vec<f32>,
}

/// A text field's text in visual lines: a multi-line field wraps each paragraph (split at newlines) at spaces to its
/// width, a word wider than a line breaks where it must; the space a line breaks at is not drawn. A one-line field is
/// one line. Always at least one line.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    pub lines: Vec<VLine>,
}

impl Layout {
    /// Lays `text` out to `max_w`; `adv` is a character's advance width (the font's; tests use a fixed one).
    pub fn new(text: &str, max_w: f32, multiline: bool, adv: &mut dyn FnMut(char) -> f32) -> Layout {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();
        let mut pre = Vec::with_capacity(n + 1);
        pre.push(0.0f32);
        for c in &chars {
            let w = if *c == '\n' { 0.0 } else { adv(*c) };
            pre.push(pre[pre.len() - 1] + w);
        }
        let width = |a: usize, b: usize| pre[b] - pre[a];
        let mut lines = Vec::new();
        let mut push = |s: usize, e: usize| lines.push(VLine { start: s, end: e, xs: (s..=e).map(|k| pre[k] - pre[s]).collect() });
        if !multiline {
            push(0, n);
            return Layout { lines };
        }
        let mut p0 = 0;
        loop {
            let p1 = chars[p0..].iter().position(|c| *c == '\n').map_or(n, |k| p0 + k);
            // Greedy at spaces: `ls` starts the line, `le` ends the last word on it.
            let (mut ls, mut le, mut ws) = (p0, None::<usize>, p0);
            loop {
                let we = (ws..p1).find(|&k| chars[k] == ' ').unwrap_or(p1);
                if let Some(e) = le.filter(|_| width(ls, we) > max_w) {
                    push(ls, e);
                    ls = e + 1;
                }
                // A word wider than the line breaks at the last character that fits (at least one per line).
                while width(ls, we) > max_w && we - ls > 1 {
                    let mut k = ls + 1;
                    while k + 1 < we && width(ls, k + 1) <= max_w {
                        k += 1;
                    }
                    push(ls, k);
                    ls = k;
                }
                le = Some(we);
                if we >= p1 {
                    break;
                }
                ws = we + 1;
            }
            push(ls, le.unwrap_or(p1));
            if p1 >= n {
                break;
            }
            p0 = p1 + 1;
        }
        Layout { lines }
    }

    /// The visual line caret position `pos` is on (the last line starting at or before it).
    pub fn line_of(&self, pos: usize) -> usize {
        self.lines.iter().rposition(|l| l.start <= pos).unwrap_or(0)
    }

    /// The last caret position on line `i`: its end, or one before when the next line goes on inside a word (the
    /// position between them shows at the next line's start).
    pub fn last_pos(&self, i: usize) -> usize {
        let l = &self.lines[i];
        match self.lines.get(i + 1) {
            Some(next) if next.start == l.end && l.end > l.start => l.end - 1,
            _ => l.end,
        }
    }

    /// The x of caret position `pos` on its line.
    pub fn x_of(&self, pos: usize) -> f32 {
        let Some(l) = self.lines.get(self.line_of(pos)) else { return 0.0 };
        l.xs[pos.saturating_sub(l.start).min(l.xs.len() - 1)]
    }

    /// The caret position on line `i` nearest to `x`.
    pub fn pos_at_x(&self, i: usize, x: f32) -> usize {
        let i = i.min(self.lines.len().saturating_sub(1));
        let Some(l) = self.lines.get(i) else { return 0 };
        let last = self.last_pos(i);
        (l.start..=last).min_by(|a, b| (l.xs[a - l.start] - x).abs().total_cmp(&(l.xs[b - l.start] - x).abs())).unwrap_or(l.start)
    }

    /// The caret position at a point of the text (`x` from the text's left, `y` from its top; lines `line_h` apart).
    pub fn pos_at(&self, x: f32, y: f32, line_h: f32) -> usize {
        let i = if y < 0.0 { 0 } else { (y / line_h.max(1.0)) as usize };
        self.pos_at_x(i, x)
    }
}

/// A text field's padding: the text starts this far from its left and top edges, and wraps this much short of its
/// right edge (`Ui::text_field`).
pub const FIELD_PAD_X: f32 = 5.0;
pub const FIELD_PAD_Y: f32 = 3.0;

/// The layout of a text field in `r` showing `text` in the painter's font (`text_field` draws it, the shell maps keys
/// and clicks with it).
pub fn field_layout(p: &mut Painter, text: &str, r: Rect, multiline: bool) -> Layout {
    let mut buf = [0u8; 4];
    Layout::new(text, r.w as f32 - 2.0 * FIELD_PAD_X, multiline, &mut |c| p.text_w(c.encode_utf8(&mut buf), BODY))
}

/// The id of a multi-line text field's scroll area (its offset in `Kit::scroll`).
pub fn field_scroll_id(id: &str) -> String {
    format!("{id}.scroll")
}

/// The height of a multi-line field's content: its lines and the padding above and below.
pub fn field_content_h(lines: usize, line_h: f32) -> i32 {
    (lines as f32 * line_h + 2.0 * FIELD_PAD_Y).ceil() as i32
}

/// The scroll offset that shows line `i` (lines `line_h` apart) of a field `view_h` tall whose content is
/// `content_h` tall: `off` when the line already shows, else the nearest offset that shows it.
pub fn reveal_offset(off: i32, i: usize, line_h: f32, view_h: i32, content_h: i32) -> i32 {
    let max = (content_h - view_h).max(0);
    let top = FIELD_PAD_Y + i as f32 * line_h;
    let lo = (top + line_h + FIELD_PAD_Y - view_h as f32).ceil() as i32;
    let hi = (top - FIELD_PAD_Y).floor() as i32;
    off.clamp(lo.min(hi), hi.max(lo)).clamp(0, max)
}

/// How far a one-line field in `r` slides its text left to keep caret position `caret` in view.
pub fn field_shift(lay: &Layout, caret: usize, r: Rect) -> f32 {
    (lay.x_of(caret) - (r.w as f32 - 2.0 * FIELD_PAD_X)).max(0.0)
}

/// The clipboard (PLAYUI1-T): the system's through `arboard` (the engine runs on the desktop only), with an in-app
/// copy that keeps copy and paste working inside the app when the system clipboard is unavailable. The crate's unit
/// tests never touch the system's.
pub mod clipboard {
    use std::cell::RefCell;

    thread_local! {
        static LOCAL: RefCell<String> = const { RefCell::new(String::new()) };
        /// None: not opened yet; Some(None): the system clipboard can't be opened.
        static SYSTEM: RefCell<Option<Option<arboard::Clipboard>>> = const { RefCell::new(None) };
    }

    fn system<T>(f: impl FnOnce(&mut arboard::Clipboard) -> Option<T>) -> Option<T> {
        if cfg!(test) {
            return None;
        }
        SYSTEM.with(|s| s.borrow_mut().get_or_insert_with(|| arboard::Clipboard::new().ok()).as_mut().and_then(f))
    }

    /// Copies `text` (the in-app copy always, the system's when it works).
    pub fn set(text: &str) {
        LOCAL.with(|l| *l.borrow_mut() = text.to_string());
        system(|c| c.set_text(text.to_string()).ok());
    }

    /// The text to paste: the system clipboard's ("" when it holds no text), else the in-app copy.
    pub fn get() -> String {
        let sys = system(|c| match c.get_text() {
            Ok(t) => Some(t),
            Err(arboard::Error::ContentNotAvailable) => Some(String::new()),
            Err(_) => None,
        });
        sys.unwrap_or_else(|| LOCAL.with(|l| l.borrow().clone()))
    }
}
