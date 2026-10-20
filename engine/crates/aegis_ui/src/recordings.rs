//! The playtest recordings browser (PLAYTEST1-L in-game playback; a kit screen a game opens from its tools): the
//! recording toggle, the recordings of the user folder and of the repo folders the game names (badge `repo`) newest
//! first with a stale badge, a recording's battles and notes, Play back (watch a battle to its end), Take over (the
//! battle replayed to a round, then played on as a new session that names its parent), Open folder and Copy to repo.
//! The game supplies the screen a session plays in (`Launch`) and its content fingerprint.

use crate::canvas::{rgba, Rect};
use crate::playtest::TAGS;
use crate::screen::{Cx, Nav, Screen};
use crate::ui::{tip, Gfx, Ui, Widget, BODY, BORDER, DIM, FOE, GOLD, GOOD, H, INFO, SMALL, TEXT, W};
use crate::widget_plumbing;
use aegis_host::playtest::{self, Recording, Session};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub const ID: &str = "recordings";
/// The settings key of the recording toggle.
pub const SETTING: &str = "record_playtests";

/// The game's screen for a recorded session: take-over None = watch it to its end; Some((parent, at)) = replay `at`
/// requests, then play on as a new session whose end names `parent`. None when the game cannot show it.
pub type Launch<G> = fn(&mut Cx<G>, &Session, Option<(Value, usize)>) -> Option<Box<dyn Screen<G>>>;

#[derive(Clone)]
enum Act {
    Toggle,
    Pick(usize),
    Battle(usize),
    Round(usize),
    PlayBack,
    TakeOver,
    OpenFolder,
    CopyToRepo,
    Refresh,
    Back,
}

struct Item {
    path: PathBuf,
    /// From a repo folder (shared), not the user folder.
    repo: bool,
    rec: Result<Recording, String>,
}

pub struct Recordings<G> {
    launch: Launch<G>,
    content: fn(&Cx<G>) -> Value,
    /// Repo folders listed beside the user folder; the first is where Copy to repo writes (none: no copying).
    repo: Vec<PathBuf>,
    items: Vec<Item>,
    pick: usize,
    battle: usize,
    round: usize,
    pub message: String,
    w: Vec<Widget<Act>>,
}

/// Where a take-over can start in `session`: (label, requests to replay): round 1 after the opening request, then
/// after each concluded round that did not end the battle.
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
        let mut s = Self { launch, content, repo, items: Vec::new(), pick: 0, battle: 0, round: 0, message: String::new(), w: Vec::new() };
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
        self.picked().and_then(|r| r.battles.get(self.battle))
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
                self.battle = 0;
                self.round = 0;
            }
            Act::Battle(i) => {
                self.battle = i;
                self.round = 0;
            }
            Act::Round(i) => self.round = i,
            Act::PlayBack | Act::TakeOver => {
                let (Some(rec), Some(session)) = (self.picked(), self.session()) else {
                    self.message = "Pick a recording with a battle.".into();
                    return;
                };
                let session = session.clone();
                let take = match a {
                    Act::TakeOver => {
                        let at = take_over_points(&session).get(self.round).map_or(1, |p| p.1);
                        Some((json!({"recording": rec.id, "battle": self.battle, "at": at}), at))
                    }
                    _ => None,
                };
                match (self.launch)(cx, &session, take) {
                    Some(screen) => cx.nav.push(Nav::Push(screen)),
                    None => self.message = "This battle cannot be shown.".into(),
                }
            }
            Act::OpenFolder => {
                let dir = match self.items.get(self.pick) {
                    Some(i) if i.repo => i.path.parent().map(Path::to_path_buf).unwrap_or_default(),
                    _ => cx.playtest.dir.clone(),
                };
                let _ = std::fs::create_dir_all(&dir);
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
        }
    }
}

impl<G> Screen<G> for Recordings<G> {
    fn id(&self) -> &'static str {
        ID
    }

    fn draw(&mut self, cx: &mut Cx<G>, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<Act> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), rgba(12, 16, 26, 255));
        ui.p.text(12.0, 8.0, "PLAYTESTS", 10.0, GOLD);
        ui.button("recordings.back", Rect::new(W - 40, 6, 34, 13), "Back", Act::Back, true);
        let on = cx.playtest.on;
        ui.checkbox("recordings.record", 12, 24, "Record playtests (sessions, the UI trail, F8 notes)", on, Act::Toggle, true);
        ui.button("recordings.refresh", Rect::new(W - 112, 24, 66, 13), "Refresh", Act::Refresh, true);
        let folder = cx.playtest.dir.display().to_string();
        ui.p.text_fit(12.0, 38.0, &format!("Folder: {folder}"), SMALL, (W - 24) as f32, DIM);
        // Left: the recordings, newest first.
        let left = Rect::new(12, 50, 214, 300);
        ui.p.panel(left, rgba(18, 24, 38, 255), BORDER, 2.0);
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
                    (format!("{id}  {} battles, {} notes{badge}", rec.battles.len(), rec.notes.len()), if cx.playtest.stale(rec) { DIM } else { TEXT })
                }
                Err(_) => (format!("{id}  (unreadable)"), FOE),
            };
            ui.row(&format!("recordings.item.{i}"), r, &label, col, Some(Act::Pick(i)), i == self.pick, Vec::new());
        }
        ui.scroll_end();
        if self.items.is_empty() {
            ui.p.text(left.x as f32 + 8.0, left.y as f32 + 8.0, if on { "No recordings yet: play a battle." } else { "No recordings. Turn recording on." }, BODY, DIM);
        }
        // Right: the picked recording.
        let right = Rect::new(232, 50, 396, 300);
        ui.p.panel(right, rgba(18, 24, 38, 255), BORDER, 2.0);
        let (x, mut y) = (right.x + 6, right.y + 4);
        match self.items.get(self.pick).map(|i| (&i.rec, i.repo)) {
            None => {}
            Some((Err(e), _)) => ui.p.text_fit(x as f32, y as f32, e, BODY, (right.w - 12) as f32, FOE),
            Some((Ok(rec), repo)) => {
                let stale = cx.playtest.stale(rec);
                ui.p.text(x as f32, y as f32, &rec.id, BODY, INFO);
                ui.p.text_fit(x as f32 + 90.0, y as f32 + 1.0, &format!("{}  ·  {}  ·  {}", rec.created, rec.origin, rec.engine), SMALL, (right.w - 102) as f32, DIM);
                y += 11;
                if stale {
                    ui.p.text(x as f32, y as f32, "Stale: made on other data, so a replay may diverge.", SMALL, GOLD);
                    y += 9;
                }
                if !rec.parent.is_null() {
                    ui.p.text_fit(x as f32, y as f32, &format!("Parent: {}", rec.parent), SMALL, (right.w - 12) as f32, DIM);
                    y += 9;
                }
                y += 2;
                ui.p.text(x as f32, y as f32, "BATTLE", SMALL, INFO);
                y += 9;
                let labels: Vec<String> = rec
                    .battles
                    .iter()
                    .enumerate()
                    .map(|(i, b)| {
                        let enc = b.requests.first().and_then(|r| r["encounter"].as_str()).unwrap_or("?");
                        let child = if b.end["parent"].is_null() { "" } else { " (take-over)" };
                        format!("{}. {enc}: {}, {} rounds{child}", i + 1, b.result(), b.checkpoints.len())
                    })
                    .collect();
                self.battle = self.battle.min(labels.len().saturating_sub(1));
                ui.select("recordings.battle", Rect::new(x, y, 240, 14), &labels, self.battle, Act::Battle, !labels.is_empty(), Vec::new());
                ui.button_tip("recordings.play", Rect::new(x + 246, y, 60, 14), "Play back", Act::PlayBack, !labels.is_empty(), tip("Play back", "Replays the battle to its end and shows it (watched, not recorded)."));
                y += 18;
                let points: Vec<String> = rec.battles.get(self.battle).map(take_over_points).unwrap_or_default().into_iter().map(|p| p.0).collect();
                self.round = self.round.min(points.len().saturating_sub(1));
                ui.select("recordings.round", Rect::new(x, y, 120, 14), &points, self.round, Act::Round, !points.is_empty(), Vec::new());
                ui.button_tip("recordings.take_over", Rect::new(x + 126, y, 60, 14), "Take over", Act::TakeOver, !points.is_empty(), tip("Take over", "Replays the battle to the chosen round, then you play on: a new session that names this one as its parent."));
                y += 20;
                ui.p.text(x as f32, y as f32, &format!("NOTES ({})", rec.notes.len()), SMALL, INFO);
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
                let off = ui.scroll_begin("recordings.notes", view, lines.len() as i32 * 10);
                for (i, line) in lines.iter().enumerate() {
                    ui.p.text_fit(view.x as f32, (view.y - off + i as i32 * 10) as f32, line, SMALL, (view.w - 6) as f32, TEXT);
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
            ui.p.text_fit(12.0, (H - 8) as f32, &self.message, SMALL, (W - 24) as f32, if self.message.starts_with("Copied") { GOOD } else { GOLD });
        }
        self.w = ui.widgets;
    }

    fn resume(&mut self, cx: &mut Cx<G>) {
        self.refresh(cx);
    }

    fn state(&self, cx: &Cx<G>) -> Value {
        json!({"on": cx.playtest.on, "recordings": self.items.iter().map(|i| i.path.file_stem().map(|s| s.to_string_lossy().to_string())).collect::<Vec<_>>(),
            "pick": self.pick, "battle": self.battle, "round": self.round, "message": self.message})
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
