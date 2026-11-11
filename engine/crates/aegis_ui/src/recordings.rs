//! The playtest recordings browser (PLAYTEST1-L in-game playback; a kit screen a game opens from its tools): the
//! recording toggle, the recordings of the user folder and of the repo folders the game names (badge `repo`) newest
//! first with a stale badge, a recording's sessions and notes, Play back (watch a session to its end), Take over (the
//! session replayed to a round, then played on as a new session that names its parent), Open folder and Copy to repo.
//! A note's row opens it whole with a Copy button (PLAYUI1-T, `NoteView`). The game supplies the screen a session plays
//! in (`Launch`) and its content fingerprint.
//!
//! LAYER-ENG-API (PR5-29c): the session picker was `recordings.battle` (`ID_ALIASES`: the agent link still takes the
//! old id) and its state key `battle` is `session`; the words the pane draws stay (`SESSION_LABELS`).

use crate::canvas::Rect;
use crate::playtest::TAGS;
use crate::screen::{Cx, Nav, Screen};
use crate::ui::{clipboard, tip, Gfx, Ui, Widget, BODY, H, SMALL, W};
use crate::widget_plumbing;
use aegis_host::playtest::{self, Recording, Session, Unreadable};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub const ID: &str = "recordings";
/// The settings key of the recording toggle.
pub const SETTING: &str = "record_playtests";
/// Old widget ids of this screen the agent link still takes (old, new; LAYER-ENG-API): an id that is the old one or
/// starts with it and a dot (the picker's options) is read as the new one.
pub const ID_ALIASES: [(&str, &str); 1] = [("recordings.battle", "recordings.session")];
/// The words the pane draws for sessions: the picker's header and the list's count ("3 battles"). They predate
/// LAYER-ENG-API and stay as drawn because the recordings pane's pixel hashes pin them
/// (`tests/fixtures/theme/pane_hashes.json`); renaming them is a theme re-record.
pub const SESSION_LABELS: SessionLabels = SessionLabels { header: "BATTLE", count: "battles" };
/// The keys of a session's first request that name its scene, first found wins: `scene`, then a command set's
/// `encounter` (the RPG kit's `new` request; a read alias, LAYER-ENG-API).
pub const SCENE_KEYS: [&str; 2] = ["scene", "encounter"];

/// See `SESSION_LABELS`.
pub struct SessionLabels {
    /// Above the session picker.
    pub header: &'static str,
    /// After a recording's session count in the list.
    pub count: &'static str,
}

/// `id` with an old widget id of this screen read as its new one (`ID_ALIASES`); None when it names no old id.
pub fn canonical_id(id: &str) -> Option<String> {
    ID_ALIASES.iter().find_map(|(old, new)| match id.strip_prefix(old) {
        Some("") => Some(new.to_string()),
        Some(rest) if rest.starts_with('.') => Some(format!("{new}{rest}")),
        _ => None,
    })
}

/// The scene a session plays (`SCENE_KEYS` in its first request), "?" when none names it.
pub fn session_scene(session: &Session) -> &str {
    let first = session.requests.first();
    SCENE_KEYS.iter().find_map(|k| first.and_then(|r| r[*k].as_str())).unwrap_or("?")
}

/// The game's screen for a recorded session: take-over None = watch it to its end; Some((parent, at)) = replay `at`
/// requests, then play on as a new session whose end names `parent`. None when the game cannot show it.
pub type Launch<G> = fn(&mut Cx<G>, &Session, Option<(Value, usize)>) -> Option<Box<dyn Screen<G>>>;

#[derive(Clone)]
enum Act {
    Toggle,
    Pick(usize),
    Session(usize),
    Round(usize),
    PlayBack,
    TakeOver,
    OpenFolder,
    CopyToRepo,
    Refresh,
    Back,
    /// Open note `i` of the picked recording whole.
    Note(usize),
}

struct Item {
    path: PathBuf,
    /// From a repo folder (shared), not the user folder.
    repo: bool,
    rec: Result<Recording, Unreadable>,
}

pub struct Recordings<G> {
    launch: Launch<G>,
    content: fn(&Cx<G>) -> Value,
    /// Repo folders listed beside the user folder; the first is where Copy to repo writes (none: no copying).
    repo: Vec<PathBuf>,
    items: Vec<Item>,
    pick: usize,
    /// The picked session of the picked recording.
    session_pick: usize,
    round: usize,
    pub message: String,
    w: Vec<Widget<Act>>,
}

/// Where a take-over can start in `session`: (label, requests to replay): round 1 after the opening request, then
/// after each concluded round that did not end the session.
pub fn take_over_points(session: &Session) -> Vec<(String, usize)> {
    let mut out = vec![("Round 1".to_string(), 1)];
    for c in &session.checkpoints {
        if c["state"]["over"] != json!(true) {
            out.push((format!("After round {}", c["state"]["round"].as_i64().unwrap_or(0)), c["at"].as_u64().unwrap_or(1) as usize));
        }
    }
    out
}

/// Opens `path` in the system file browser.
pub fn open_folder(path: &Path) -> Result<(), String> {
    let program = if cfg!(windows) {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    std::process::Command::new(program).arg(path).spawn().map(|_| ()).map_err(|e| format!("Cannot open {}: {e}", path.display()))
}

impl<G> Recordings<G> {
    pub fn new(cx: &Cx<G>, launch: Launch<G>, content: fn(&Cx<G>) -> Value, repo: Vec<PathBuf>) -> Self {
        let mut s = Self { launch, content, repo, items: Vec::new(), pick: 0, session_pick: 0, round: 0, message: String::new(), w: Vec::new() };
        s.refresh(cx);
        s
    }

    fn refresh(&mut self, cx: &Cx<G>) {
        let picked = self.items.get(self.pick).map(|i| i.path.clone());
        self.items = cx.playtest.list().into_iter().map(|(path, rec)| Item { path, repo: false, rec }).collect();
        for dir in &self.repo {
            self.items.extend(playtest::list(dir).into_iter().map(|(path, rec)| Item { path, repo: true, rec }));
        }
        self.items.sort_by(|a, b| b.path.file_name().cmp(&a.path.file_name()));
        self.pick = picked.and_then(|p| self.items.iter().position(|i| i.path == p)).unwrap_or(0);
    }

    fn picked(&self) -> Option<&Recording> {
        self.items.get(self.pick).and_then(|i| i.rec.as_ref().ok())
    }

    fn session(&self) -> Option<&Session> {
        self.picked().and_then(|r| r.sessions.get(self.session_pick))
    }

    fn act(&mut self, cx: &mut Cx<G>, a: Act) {
        self.message.clear();
        match a {
            Act::Toggle => {
                let on = !cx.playtest.on;
                let content = self.content;
                let fingerprint = if on && cx.playtest.content.is_null() { content(cx) } else { Value::Null };
                cx.playtest.set_on(on, || fingerprint);
                cx.settings.v[SETTING] = json!(on);
                if let Err(e) = cx.settings.save() {
                    self.message = e;
                }
                self.refresh(cx);
            }
            Act::Pick(i) => {
                self.pick = i;
                self.session_pick = 0;
                self.round = 0;
            }
            Act::Session(i) => {
                self.session_pick = i;
                self.round = 0;
            }
            Act::Round(i) => self.round = i,
            Act::PlayBack | Act::TakeOver => {
                let (Some(rec), Some(session)) = (self.picked(), self.session()) else {
                    self.message = "Pick a recording with a session.".into();
                    return;
                };
                let session = session.clone();
                let take = match a {
                    Act::TakeOver => {
                        let at = take_over_points(&session).get(self.round).map_or(1, |p| p.1);
                        Some((json!({"recording": rec.id, "session": self.session_pick, "at": at}), at))
                    }
                    _ => None,
                };
                match (self.launch)(cx, &session, take) {
                    Some(screen) => cx.nav.push(Nav::Push(screen)),
                    None => self.message = "This session cannot be shown.".into(),
                }
            }
            Act::OpenFolder => {
                let dir = match self.items.get(self.pick) {
                    Some(i) if i.repo => i.path.parent().map(Path::to_path_buf).unwrap_or_default(),
                    _ => cx.playtest.dir.clone(),
                };
                let _ = aegis_core::vfs::dev::create_dir_all(&dir);
                if let Err(e) = open_folder(&dir) {
                    self.message = e;
                }
            }
            Act::CopyToRepo => {
                let (Some(item), Some(inbox)) = (self.items.get(self.pick), self.repo.first()) else { return };
                self.message = match playtest::copy_to(&item.path, inbox) {
                    Ok(dest) => format!("Copied to {}: push it and name the id.", dest.display()),
                    Err(e) => e,
                };
                self.refresh(cx);
            }
            Act::Refresh => self.refresh(cx),
            Act::Back => cx.nav.push(Nav::Pop),
            Act::Note(i) => {
                if let Some(n) = self.picked().and_then(|r| r.notes.get(i)) {
                    cx.nav.push(Nav::Push(Box::new(NoteView::new(n))));
                }
            }
        }
    }
}

impl<G> Screen<G> for Recordings<G> {
    fn id(&self) -> &'static str {
        ID
    }

    fn draw(&mut self, cx: &mut Cx<G>, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<Act> = Ui::new(g, mouse, live);
        let t = ui.t.clone();
        ui.p.fill(Rect::new(0, 0, W, H), t.tool_bg);
        ui.p.text(12.0, 8.0, "PLAYTESTS", 10.0, t.gold);
        ui.button("recordings.back", Rect::new(W - 40, 6, 34, 13), "Back", Act::Back, true);
        let on = cx.playtest.on;
        ui.checkbox("recordings.record", 12, 24, "Record playtests (sessions, the UI trail, F8 notes)", on, Act::Toggle, true);
        ui.button("recordings.refresh", Rect::new(W - 112, 24, 66, 13), "Refresh", Act::Refresh, true);
        let folder = cx.playtest.dir.display().to_string();
        ui.p.text_fit(12.0, 38.0, &format!("Folder: {folder}"), SMALL, (W - 24) as f32, t.dim);
        // Left: the recordings, newest first.
        let left = Rect::new(12, 50, 214, 300);
        ui.p.panel(left, t.tool_pane, t.edge, 2.0);
        let view = Rect::new(left.x + 3, left.y + 3, left.w - 6, left.h - 6);
        let off = ui.scroll_begin("recordings.list", view, self.items.len() as i32 * 16);
        for (i, item) in self.items.iter().enumerate() {
            let r = Rect::new(view.x, view.y - off + i as i32 * 16, view.w - 4, 15);
            let id = item.path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            let (label, col) = match &item.rec {
                Ok(rec) => {
                    let mut badges = Vec::new();
                    if item.repo {
                        badges.push("repo");
                    }
                    if cx.playtest.stale(rec) {
                        badges.push("stale");
                    }
                    let badge = if badges.is_empty() { String::new() } else { format!(" [{}]", badges.join(", ")) };
                    (format!("{id}  {} {}, {} notes{badge}", rec.sessions.len(), SESSION_LABELS.count, rec.notes.len()), if cx.playtest.stale(rec) { t.dim } else { t.text })
                }
                Err(_) => (format!("{id}  (unreadable)"), t.foe),
            };
            ui.row(&format!("recordings.item.{i}"), r, &label, col, Some(Act::Pick(i)), i == self.pick, Vec::new());
        }
        ui.scroll_end();
        if self.items.is_empty() {
            ui.p.text(left.x as f32 + 8.0, left.y as f32 + 8.0, if on { "No recordings yet: play to record one." } else { "No recordings. Turn recording on." }, BODY, t.dim);
        }
        // Right: the picked recording.
        let right = Rect::new(232, 50, 396, 300);
        ui.p.panel(right, t.tool_pane, t.edge, 2.0);
        let (x, mut y) = (right.x + 6, right.y + 4);
        match self.items.get(self.pick).map(|i| (&i.rec, i.repo)) {
            None => {}
            Some((Err(e), _)) => ui.p.text_fit(x as f32, y as f32, &e.reason, BODY, (right.w - 12) as f32, t.foe),
            Some((Ok(rec), repo)) => {
                let stale = cx.playtest.stale(rec);
                ui.p.text(x as f32, y as f32, &rec.id, BODY, t.info);
                ui.p.text_fit(x as f32 + 90.0, y as f32 + 1.0, &format!("{}  ·  {}  ·  {}", rec.created, rec.origin, rec.engine), SMALL, (right.w - 102) as f32, t.dim);
                y += 11;
                if stale {
                    ui.p.text(x as f32, y as f32, "Stale: made on other data, so a replay may diverge.", SMALL, t.gold);
                    y += 9;
                }
                if !rec.parent.is_null() {
                    ui.p.text_fit(x as f32, y as f32, &format!("Parent: {}", rec.parent), SMALL, (right.w - 12) as f32, t.dim);
                    y += 9;
                }
                y += 2;
                ui.p.text(x as f32, y as f32, SESSION_LABELS.header, SMALL, t.info);
                y += 9;
                let labels: Vec<String> = rec
                    .sessions
                    .iter()
                    .enumerate()
                    .map(|(i, b)| {
                        let enc = session_scene(b);
                        let child = if b.end["parent"].is_null() { "" } else { " (take-over)" };
                        format!("{}. {enc}: {}, {} rounds{child}", i + 1, b.result(), b.checkpoints.len())
                    })
                    .collect();
                self.session_pick = self.session_pick.min(labels.len().saturating_sub(1));
                ui.select("recordings.session", Rect::new(x, y, 240, 14), &labels, self.session_pick, Act::Session, !labels.is_empty(), Vec::new());
                ui.button_tip("recordings.play", Rect::new(x + 246, y, 60, 14), "Play back", Act::PlayBack, !labels.is_empty(), tip("Play back", "Replays the session to its end and shows it (watched, not recorded)."));
                y += 18;
                let points: Vec<String> = rec.sessions.get(self.session_pick).map(take_over_points).unwrap_or_default().into_iter().map(|p| p.0).collect();
                self.round = self.round.min(points.len().saturating_sub(1));
                ui.select("recordings.round", Rect::new(x, y, 120, 14), &points, self.round, Act::Round, !points.is_empty(), Vec::new());
                ui.button_tip("recordings.take_over", Rect::new(x + 126, y, 60, 14), "Take over", Act::TakeOver, !points.is_empty(), tip("Take over", "Replays the session to the chosen round, then you play on: a new session that names this one as its parent."));
                y += 20;
                ui.p.text(x as f32, y as f32, &format!("NOTES ({})", rec.notes.len()), SMALL, t.info);
                y += 9;
                let view = Rect::new(x, y, right.w - 12, (right.y + right.h) - 24 - y);
                let lines: Vec<String> = rec
                    .notes
                    .iter()
                    .map(|n| {
                        let tags: Vec<&str> = n["tags"].as_array().map(|a| a.iter().filter_map(Value::as_str).filter(|t| TAGS.contains(t)).collect()).unwrap_or_default();
                        let place = match n["round"].as_i64() {
                            Some(r) => format!("{} r{r}", n["scene"].as_str().unwrap_or("")),
                            None => n["scene"].as_str().unwrap_or("").to_string(),
                        };
                        format!("{}. [{}] {place}: {}", n["n"], tags.join(", "), n["text"].as_str().unwrap_or("").replace('\n', " "))
                    })
                    .collect();
                // One row per note, cut to one line; a click opens the note whole (PLAYUI1-T).
                let row_h = 12;
                let off = ui.scroll_begin("recordings.notes", view, lines.len() as i32 * row_h);
                for (i, line) in lines.iter().enumerate() {
                    let r = Rect::new(view.x, view.y - off + i as i32 * row_h, view.w - 5, row_h);
                    ui.row(&format!("recordings.note.{i}"), r, line, t.text, Some(Act::Note(i)), false, tip(&format!("Note {}", i + 1), "Click to read it whole and copy it."));
                }
                ui.scroll_end();
                let by = (right.y + right.h) - 19;
                ui.button_tip("recordings.open_folder", Rect::new(x, by, 70, 14), "Open folder", Act::OpenFolder, true, tip("Open folder", "Shows the recording's folder (the JSON and its note images)."));
                if !repo && !self.repo.is_empty() {
                    ui.button_tip("recordings.copy", Rect::new(x + 76, by, 70, 14), "Copy to repo", Act::CopyToRepo, true, tip("Copy to repo", "Copies the recording and its images to the repo inbox: push it on any branch and name the id."));
                }
            }
        }
        if !self.message.is_empty() {
            ui.p.text_fit(12.0, (H - 8) as f32, &self.message, SMALL, (W - 24) as f32, if self.message.starts_with("Copied") { t.good } else { t.gold });
        }
        self.w = ui.widgets;
    }

    fn resume(&mut self, cx: &mut Cx<G>) {
        self.refresh(cx);
    }

    fn state(&self, cx: &Cx<G>) -> Value {
        json!({"on": cx.playtest.on, "recordings": self.items.iter().map(|i| i.path.file_stem().map(|s| s.to_string_lossy().to_string())).collect::<Vec<_>>(),
            "pick": self.pick, "session": self.session_pick, "round": self.round, "message": self.message,
            "error": self.items.get(self.pick).and_then(|i| i.rec.as_ref().err()).map(|e| e.detail.as_str())})
    }

    widget_plumbing!(Cx<G>);
}

/// A playtest note read whole (PLAYUI1-T): its number, tags and where it was taken, its full text (wrapped, the
/// wheel scrolls it) and Copy, which puts the text on the clipboard. Esc, right click or Close go back.
pub struct NoteView {
    title: String,
    meta: String,
    pub text: String,
    pub message: String,
    w: Vec<Widget<NoteAct>>,
}

#[derive(Clone)]
enum NoteAct {
    Copy,
    Close,
}

/// The id of the note view screen and the prefix of its widgets.
pub const NOTE_ID: &str = "recordings.note_view";

impl NoteView {
    /// The note as a recording keeps it (`n`, `tags`, `scene`, `round`, `text`).
    pub fn new(n: &Value) -> Self {
        let tags: Vec<&str> = n["tags"].as_array().map(|a| a.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
        let mut meta = n["scene"].as_str().unwrap_or("").to_string();
        if let Some(r) = n["round"].as_i64() {
            meta = format!("{meta}, round {r}");
        }
        if !tags.is_empty() {
            meta = format!("[{}] {meta}", tags.join(", "));
        }
        Self { title: format!("Note {}", n["n"]), meta, text: n["text"].as_str().unwrap_or("").to_string(), message: String::new(), w: Vec::new() }
    }

    fn act<G>(&mut self, cx: &mut Cx<G>, a: NoteAct) {
        match a {
            NoteAct::Copy => {
                clipboard::set(&self.text);
                self.message = "Copied to the clipboard.".into();
            }
            NoteAct::Close => cx.nav.push(Nav::Pop),
        }
    }
}

impl<G> Screen<G> for NoteView {
    fn id(&self) -> &'static str {
        NOTE_ID
    }
    fn overlay(&self) -> bool {
        true
    }
    fn draw(&mut self, _cx: &mut Cx<G>, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<NoteAct> = Ui::new(g, mouse, live);
        let t = ui.t.clone();
        ui.p.fill(Rect::new(0, 0, W, H), t.scrim);
        let r = Rect::new(W / 2 - 200, 30, 400, H - 60);
        ui.frame(r, &self.title, false);
        ui.p.text_fit(r.x as f32 + 10.0, r.y as f32 + 24.0, &self.meta, SMALL, (r.w - 20) as f32, t.dim);
        let view = Rect::new(r.x + 10, r.y + 36, r.w - 20, r.h - 36 - 28);
        ui.p.panel(view, t.input, t.edge, 2.0);
        let line = ui.p.line_h(BODY).ceil() as i32 + 1;
        let mut lines = Vec::new();
        for para in self.text.split('\n') {
            let wrapped = ui.p.wrap(para, BODY, (view.w - 14) as f32);
            if wrapped.is_empty() {
                lines.push(String::new());
            }
            lines.extend(wrapped);
        }
        let off = ui.scroll_begin(&format!("{NOTE_ID}.text"), view, lines.len() as i32 * line + 6);
        for (i, l) in lines.iter().enumerate() {
            ui.p.text((view.x + 5) as f32, (view.y + 3 - off + i as i32 * line) as f32, l, BODY, t.text);
        }
        ui.scroll_end();
        let by = r.y + r.h - 22;
        if !self.message.is_empty() {
            ui.p.text_fit(r.x as f32 + 10.0, by as f32 + 3.0, &self.message, SMALL, (r.w - 140) as f32, t.good);
        }
        ui.button_tip(&format!("{NOTE_ID}.copy"), Rect::new(r.x + r.w - 124, by, 54, 15), "Copy", NoteAct::Copy, true, tip("Copy", "Puts the note's text on the clipboard."));
        ui.button(&format!("{NOTE_ID}.close"), Rect::new(r.x + r.w - 64, by, 54, 15), "Close", NoteAct::Close, true);
        self.w = ui.widgets;
    }
    fn state(&self, _cx: &Cx<G>) -> Value {
        json!({"title": self.title, "meta": self.meta, "text": self.text, "message": self.message})
    }
    widget_plumbing!(Cx<G>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_over_starts_at_round_one_and_after_each_round_that_goes_on() {
        let s = Session {
            requests: vec![json!({"cmd": "new"}); 9],
            checkpoints: vec![json!({"at": 4, "state": {"round": 1, "over": false}}), json!({"at": 9, "state": {"round": 2, "over": true}})],
            end: json!({"result": "won", "at": 9}),
        };
        assert_eq!(take_over_points(&s), vec![("Round 1".to_string(), 1), ("After round 1".to_string(), 4)]);
    }
}
