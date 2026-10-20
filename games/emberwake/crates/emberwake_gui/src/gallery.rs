//! UI Gallery (UIGALLERY1, Scene Runner): UI panes one at a time, each on sample data, so they can be judged and
//! refined on their own; some may never be used by the game (they flesh out the engine's kit). Entries so far:
//! the battle timeline (TIMELINE1: every unit's track over the round's blocks on the engine's `timeline` widget, fed
//! by a live Test Arena battle; past rounds from the round archive, a block's state from its snapshot).

use crate::battle::{action_rows, kind_color};
use crate::intent_text;
use crate::screens::{Cx, GameCx, Nav, Screen};
use aegis_ui::canvas::{rgb, Rect, Rgba};
use aegis_ui::timeline::{Section, Segment, Timeline, Track};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, BORDER, DIM, GOLD, SMALL, TEXT, TITLE, W};
use aegis_ui::widget_plumbing;
use emberwake_battle::resolver::ScheduledAction;
use emberwake_battle::session::{Phase, Session, UnitSnap};
use emberwake_host::{Battle, Host};
use serde_json::{json, Value};

const ENTRIES: [(&str, &str); 1] = [("timeline", "Battle timeline")];
const ALLY: Rgba = [110, 170, 255, 255];
const FOE: Rgba = [240, 100, 92, 255];
/// The strip under the timeline: an action's details or a block's snapshot.
const DETAIL_H: i32 = 64;

#[derive(Debug, Clone)]
pub enum GAct {
    Entry(usize),
    Exit,
    Restart,
    AutoPlan,
    Confirm,
    Step,
    Advance,
    /// View the round `delta` away (the round archive).
    View(i32),
    Reveal,
    Block(i32),
    Track(String),
    Segment(String, String),
}

/// The timeline entry's battle and what is selected on it.
struct Demo {
    host: Host,
    reveal: bool,
    /// The round on view (the live one unless a past one is picked).
    viewed: i32,
    block: Option<i32>,
    track: Option<String>,
    detail: Vec<(String, Rgba)>,
}

pub struct Gallery {
    entry: usize,
    demo: Option<Demo>,
    w: Vec<Widget<GAct>>,
}

impl Default for Gallery {
    fn default() -> Self {
        Self::new()
    }
}

impl Gallery {
    pub fn new() -> Self {
        Self { entry: 0, demo: None, w: Vec::new() }
    }

    fn demo(&mut self, cx: &Cx) -> &mut Demo {
        if self.demo.is_none() {
            let mut host = Host::new(Battle::new(cx.data.cat.clone()));
            let _ = host.handle(&json!({"cmd": "new", "encounter": cx.encounter.clone()}));
            if let Some(s) = host.session.as_mut() {
                s.plan_apply_preset_all("balanced");
            }
            self.demo = Some(Demo { host, reveal: false, viewed: 1, block: None, track: None, detail: Vec::new() });
        }
        self.demo.as_mut().unwrap()
    }

    fn act(&mut self, cx: &mut Cx, a: GAct) {
        match a {
            GAct::Entry(i) => self.entry = i,
            GAct::Exit => cx.nav.push(Nav::Pop),
            GAct::Restart => self.demo = None,
            a => {
                let d = self.demo(cx);
                let Some(s) = d.host.session.as_mut() else { return };
                // The live round follows the battle (paging rounds is the one action that leaves it).
                let follow = d.viewed == s.current_round && !matches!(a, GAct::View(_));
                match a {
                    GAct::AutoPlan => s.plan_apply_preset_all("balanced"),
                    GAct::Confirm => {
                        s.confirm_plan();
                    }
                    GAct::Step => {
                        if s.phase != Phase::Replay {
                            s.step_next();
                        }
                    }
                    GAct::Advance => {
                        if s.can_advance_round() && s.phase == Phase::Replay {
                            s.advance_round();
                            s.plan_apply_preset_all("balanced");
                            d.block = None;
                        }
                    }
                    GAct::View(delta) => {
                        let target = (d.viewed + delta).clamp(1, s.current_round);
                        if target != d.viewed {
                            d.viewed = target;
                            d.block = None;
                            d.detail.clear();
                        }
                    }
                    GAct::Reveal => d.reveal = !d.reveal,
                    GAct::Block(b) => d.block = if d.block == Some(b) { None } else { Some(b) },
                    GAct::Track(t) => d.track = if d.track.as_deref() == Some(t.as_str()) { None } else { Some(t) },
                    GAct::Segment(t, seg) => {
                        d.detail = segment_detail(s, d.viewed, &t, &seg);
                        d.track = Some(t);
                        d.block = None;
                    }
                    _ => {}
                }
                if follow {
                    if let Some(s) = d.host.session.as_ref() {
                        d.viewed = s.current_round;
                    }
                }
            }
        }
    }
}

fn segment_of(name: &str, start: i32, end: i32, kind: &str, target: Option<String>, key: String, extra: String) -> Segment {
    let mut t = vec![(name.to_string(), GOLD), (format!("Blocks {start}-{end}  {kind}"), DIM)];
    if let Some(target) = &target {
        t.push((format!("On {target}"), TEXT));
    }
    if !extra.is_empty() {
        t.push((extra, DIM));
    }
    let mut fill = kind_color(kind);
    fill[3] = 230;
    Segment { key, start, end, label: target.map(|t| format!("{name} > {t}")).unwrap_or_else(|| name.to_string()), fill, dashed: kind == "MOVE", tip: t }
}

fn rows(s: &Session, actions: &[ScheduledAction], u: usize) -> Vec<Segment> {
    action_rows(&s.w, actions, u).into_iter().enumerate().map(|(i, c)| segment_of(&c.2, c.0, c.1, &c.3, c.4, format!("{i}"), String::new())).collect()
}

/// The units' tracks for round `viewed`: the live round shows the heroes' plan while planning (then their committed
/// actions) and masks the enemies while planning unless revealed (their tell instead); a past round shows its
/// archived actions, unmasked.
fn timeline_of(s: &Session, viewed: i32, reveal: bool, block: Option<i32>, track: Option<String>) -> Timeline {
    let w = &s.w;
    let live = viewed == s.current_round;
    let planning = live && s.phase == Phase::Planning && !s.plan_confirmed;
    let archived: Vec<ScheduledAction> = if live { s.resolver.scheduled.clone() } else { s.round_record(viewed).map(|r| r.actions.clone()).unwrap_or_default() };
    let tells = s.enemy_intents();
    let allies = w
        .allies
        .iter()
        .map(|&u| {
            let segments = if planning && w.units[u].is_active() {
                s.plan
                    .compile(w, u)
                    .entries
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let sk = &w.cat.skills[e.skill];
                        segment_of(&sk.name, e.start_block, e.end_block(w), &sk.kind, e.target.filter(|t| *t != u).map(|t| w.units[t].name.clone()), format!("{i}"), format!("{} AP", e.total_ap()))
                    })
                    .collect()
            } else {
                rows(s, &archived, u)
            };
            Track { key: format!("u{u}"), label: w.units[u].name.clone(), colour: ALLY, segments, masked: None }
        })
        .collect();
    let enemies = w
        .enemies
        .iter()
        .filter(|&&e| archived.iter().any(|a| a.unit == e) || w.has(e))
        .map(|&e| {
            let masked = (planning && !reveal && w.units[e].is_active()).then(|| match tells.iter().find(|t| t.unit == e) {
                Some(t) => format!("Plans in secret. {}", intent_text::card_line(w, &t.line)),
                None => "Plans in secret".to_string(),
            });
            Track { key: format!("u{e}"), label: w.units[e].name.clone(), colour: FOE, segments: rows(s, &archived, e), masked }
        })
        .collect();
    Timeline {
        blocks: s.def.total_blocks.max(1),
        playhead: if live && !planning && s.phase != Phase::Replay { s.current_block } else { 0 },
        selected_block: block,
        selected_track: track,
        sections: vec![Section { title: "Allies".into(), colour: ALLY, tracks: allies }, Section { title: "Enemies".into(), colour: FOE, tracks: enemies }],
    }
}

/// The hover card lines of a clicked segment, for the detail strip.
fn segment_detail(s: &Session, viewed: i32, track: &str, seg: &str) -> Vec<(String, Rgba)> {
    let t = timeline_of(s, viewed, true, None, None);
    t.sections
        .iter()
        .flat_map(|x| x.tracks.iter())
        .find(|x| x.key == track)
        .and_then(|x| x.segments.iter().find(|g| g.key == seg).map(|g| (x.label.clone(), g.tip.clone())))
        .map(|(unit, tip)| std::iter::once((unit, TEXT)).chain(tip).collect())
        .unwrap_or_default()
}

/// One unit of a snapshot in one line: "Warrior  HP 280/334  AP 8  Offensive  Haste".
fn snap_line(name: &str, u: &UnitSnap) -> String {
    let mut line = format!("{name}  HP {}/{}  AP {}  {:?}", u.hp, u.max_hp, u.ap, u.stance);
    if u.guard > 0.0 {
        line += &format!("  guard {:.0}%", u.guard * 100.0);
    }
    let st: Vec<String> = u.statuses.iter().map(|x| emberwake_rules::status::display_name(x)).chain(u.pending.iter().map(|x| format!("{} next", emberwake_rules::status::display_name(x)))).collect();
    if !st.is_empty() {
        line += &format!("  {}", st.join(", "));
    }
    if u.withdrawn {
        line += "  (left)";
    } else if u.hp <= 0 {
        line += "  (down)";
    }
    line
}

impl Screen<GameCx> for Gallery {
    fn id(&self) -> &'static str {
        "ui_gallery"
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        self.demo(cx);
        let mut ui: Ui<GAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, aegis_ui::ui::H), rgb(12, 16, 26));
        ui.p.text(6.0, 4.0, "UI GALLERY", BODY, GOLD);
        ui.button("gallery.exit", Rect::new(W - 36, 3, 32, 12), "Exit", GAct::Exit, true);
        let mut y = 22;
        for (i, (id, name)) in ENTRIES.iter().enumerate() {
            ui.card(&format!("gallery.entry.{id}"), Rect::new(4, y, 100, 16), &[(name.to_string(), if i == self.entry { GOLD } else { TEXT })], Some(GAct::Entry(i)), i == self.entry, true, tip(name, "Show this pane."));
            y += 18;
        }
        let pane = Rect::new(110, 20, W - 114, aegis_ui::ui::H - 24);
        ui.p.panel(pane, rgb(18, 24, 38), BORDER, 2.0);
        if ENTRIES[self.entry].0 == "timeline" {
            let d = self.demo.as_ref().unwrap();
            let Some(s) = d.host.session.as_ref() else {
                ui.p.text(pane.x as f32 + 6.0, pane.y as f32 + 6.0, "No battle (the arena encounter did not start).", BODY, DIM);
                self.w = ui.widgets;
                return;
            };
            let live_round = d.viewed == s.current_round;
            let planning = s.phase == Phase::Planning && !s.plan_confirmed;
            let phase = if !live_round {
                "archive".to_string()
            } else if s.combat_over {
                format!("battle over: {}", s.outcome)
            } else if planning {
                "planning".into()
            } else if s.phase == Phase::Replay {
                "resolved".into()
            } else {
                format!("resolving block {}", s.current_block)
            };
            ui.button_full("gallery.timeline.prev", Rect::new(pane.x + 4, pane.y + 3, 14, 12), "<", GAct::View(-1), d.viewed > 1, false, SMALL, tip("Previous round", "View the round before (the round archive)."));
            ui.p.text_fit(pane.x as f32 + 21.0, pane.y as f32 + 4.0, &format!("Round {}/{}  {phase}", d.viewed, s.current_round), TITLE, 120.0, if live_round { TEXT } else { GOLD });
            ui.button_full("gallery.timeline.next", Rect::new(pane.x + 144, pane.y + 3, 14, 12), ">", GAct::View(1), d.viewed < s.current_round, false, SMALL, tip("Next round", "View the round after (back to the live one)."));
            let mut x = pane.x + 164;
            let buttons = [
                ("autoplan", "Auto plan", GAct::AutoPlan, live_round && planning, "Fill every hero's round with the Balanced tactic."),
                ("confirm", "Confirm", GAct::Confirm, live_round && planning, "Confirm the round (the enemies' plans were drafted first)."),
                ("step", "Step", GAct::Step, live_round && !planning && s.phase != Phase::Replay, "Resolve the next action: the playhead moves."),
                ("advance", "Next round", GAct::Advance, live_round && s.phase == Phase::Replay && s.can_advance_round(), "Start the next round."),
                ("reveal", if d.reveal { "Hide enemies" } else { "Reveal enemies" }, GAct::Reveal, true, "Show the enemies' plans while planning (as foresight would)."),
                ("restart", "Restart", GAct::Restart, true, "A new battle."),
            ];
            for (id, label, act, enabled, help) in buttons {
                let w = ui.p.text_w(label, SMALL).ceil() as i32 + 10;
                ui.button_full(&format!("gallery.timeline.{id}"), Rect::new(x, pane.y + 3, w, 12), label, act, enabled, false, SMALL, tip(label, help));
                x += w + 2;
            }
            let t = timeline_of(s, d.viewed, d.reveal, d.block, d.track.clone());
            t.draw(&mut ui, "gallery.timeline", Rect::new(pane.x + 4, pane.y + 20, pane.w - 8, pane.h - 24 - DETAIL_H), GAct::Block, |k| GAct::Track(k.to_string()), |k, g| GAct::Segment(k.to_string(), g.to_string()));
            // The strip under the timeline: a selected block's snapshot, else the clicked action, else a hint.
            let strip = Rect::new(pane.x + 4, pane.y + pane.h - DETAIL_H, pane.w - 8, DETAIL_H - 4);
            ui.p.fill(Rect::new(strip.x, strip.y - 2, strip.w, 1), BORDER);
            let snap = d.block.and_then(|b| s.round_record(d.viewed).and_then(|r| r.snapshots.get(b as usize)).map(|sn| (b, sn)));
            let lines: Vec<(String, Rgba)> = match (d.block, snap) {
                (Some(_), Some((b, sn))) => std::iter::once((format!("After block {b} of round {}", d.viewed), GOLD))
                    .chain(s.w.allies.iter().chain(s.w.enemies.iter()).filter_map(|&u| sn.get(u).filter(|x| x.pos.is_some() || x.hp <= 0 || x.withdrawn).map(|x| (snap_line(&s.w.units[u].name, x), if s.w.allies.contains(&u) { ALLY } else { FOE }))))
                    .collect(),
                (Some(b), None) => vec![(format!("Block {b} has not resolved yet."), DIM)],
                _ if !d.detail.is_empty() => d.detail.clone(),
                _ => vec![("Click an action for its details, a resolved block for the state after it; < > page through the rounds.".to_string(), DIM)],
            };
            // Snapshot lines go in two columns.
            let (cols, col_w) = if lines.len() > 6 { (2, (strip.w - 8) / 2) } else { (1, strip.w - 8) };
            for (i, (text, col)) in lines.iter().enumerate() {
                let (cx_, row) = if cols == 2 && i > 0 { (((i - 1) % 2) as i32, ((i - 1) / 2 + 1) as i32) } else { (0, i as i32) };
                let yy = strip.y as f32 + row as f32 * 8.5;
                if yy > (strip.y + strip.h) as f32 - 7.0 {
                    break;
                }
                ui.p.text_fit((strip.x + 2 + cx_ * col_w) as f32, yy, text, SMALL, col_w as f32 - 4.0, *col);
            }
        }
        self.w = ui.widgets;
    }

    fn state(&self, _cx: &Cx) -> Value {
        let d = self.demo.as_ref();
        let s = d.and_then(|d| d.host.session.as_ref());
        json!({
            "entry": ENTRIES[self.entry].0, "viewed": d.map(|d| d.viewed), "round": s.map(|s| s.current_round), "block": s.map(|s| s.current_block),
            "selected_block": d.and_then(|d| d.block), "planning": s.map(|s| s.phase == Phase::Planning && !s.plan_confirmed), "rounds_archived": s.map(|s| s.history.len()),
        })
    }

    widget_plumbing!();
}
