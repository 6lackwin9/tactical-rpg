//! UI Gallery (UIGALLERY1, Scene Runner; decision "UIGALLERY1 answers"): the UI one element at a time, each on sample
//! data and named, so each can be judged and refined on its own. Three groups:
//! - Kit: the engine's widgets in their states (`aegis_ui::specimens`), and the vignette player on the engine's sample
//!   reel (STORYUI1-E, `aegis_ui::vignette`; the preview plays on and starts over, Open plays it full screen) or any
//!   world's vignette as written, in the game's card palette (STORYUI1-U1, a reel picker over the preview).
//! - Art: Portraits (PORTRAITFX1-G 6.3): every portrait of the game and kit layers (expression frames under their
//!   character) drawn through the game's binding (`portraits::Faces`) with the Kit widgets' portrait controls, its
//!   sidecar summary, one button per cue row of `portrait_cues.json`, one box per expression-effect row of
//!   `portrait_expressions.json` (PORTRAITFX1-EX) and the expression picker; the review page of the portrait asset
//!   tasks.
//! - Screens: every screen, drawn shrunk into the pane (`aegis_ui::preview`) on a sample game (a debug game of the
//!   first playable world that saving refuses); Open shows the screen itself to try it. The Mission board entry
//!   (STORYUI1-U2) draws the board pane on a fixture board without a game (`board::BoardSample`).
//! - Battle panels: each panel of the battle screen enlarged on its own (the battle records its panels' rects while it
//!   draws), and the battle timeline (TIMELINE1: every unit's track over the round's blocks, fed by a live Test Arena
//!   battle; past rounds from the round archive, a block's state from its snapshot).

use crate::battle::BattleScreen;
use crate::portraits::{cue_of, hue_name, Faces, Who};
use crate::screens::{Cx, Game, GameCx, Key, Nav, Screen};
use crate::timeline_view;
use aegis_core::assets::Layers;
use aegis_core::json;
use aegis_ui::theme::th;
use aegis_ui::canvas::{rgb, Rect, Rgba};
use aegis_ui::portrait_fx::{Cue, Tier};
use aegis_ui::preview;
use aegis_ui::screen::PointerPhase;
use aegis_ui::specimens::{portrait_label, KitAct, Specimens, PORTRAIT_OVERLAYS, PORTRAIT_REACTIONS, PORTRAIT_SIZES, PORTRAIT_STAGE, PORTRAIT_STATES};
use aegis_ui::timeline::Timeline;
use aegis_ui::ui::{tip, Gfx, Tip, Ui, Widget, BODY, SMALL, TITLE, W};
use aegis_ui::vignette;
use aegis_ui::widget_plumbing;
use emberwake_battle::session::{Phase, Session, UnitSnap};
use emberwake_host::{Battle, Host};
use serde_json::{json, Value};

/// (id, name, group, what it shows). Battle panel ids are the battle screen's own panel names.
const ENTRIES: [(&str, &str, &str, &str); 34] = [
    ("kit", "Kit widgets", "Kit", "Every kit widget in its states; use them, the foot of the page says what happened."),
    ("layout", "Layout", "Kit", "Anchors, shares, fills, a stack and pick, all calculated from the canvas: the layout audit's subject."),
    ("frame_tuner", "Frame tuner", "Kit", "A frame's corner, tip, line and padding on a live panel; Save in a debug build."),
    ("vignette", "Vignette player", "Kit", "The engine's sample reel or a world's vignette: image, text, title and line cards with fades, typing, hold and skip."),
    ("pip_gauge", "Pip gauge", "Kit", "The engine pip gauge in its states: kept, spent and next round's gain; red, a debt, two points a pip; numbers on hover."),
    ("timeline_pane", "Timeline pane", "Kit", "The engine timeline widget alone on sample data: two sections, a masked track, parts, a ghost, the playhead; the rows scroll."),
    ("portraits", "Portraits", "Art", "Every portrait with its sidecar, its effects and the cue rows: the review page of the portrait assets."),
    ("main_menu", "Main menu", "Screens", "The player's front door."),
    ("new_game", "New game", "Screens", "World cards and the start."),
    ("settings", "Settings", "Screens", "Sound, display and gameplay tabs."),
    ("save", "Save slots", "Screens", "Saving from the game menu."),
    ("load", "Load slots", "Screens", "Loading from the main menu or the game menu."),
    ("region", "Region map", "Screens", "Walking a location: the map, the Here list, the log."),
    ("dialogue", "Dialogue", "Screens", "A conversation on a plain backdrop (the first person met)."),
    ("shop", "Shop", "Screens", "A shop counter on a plain backdrop (the first trader met)."),
    ("board", "Mission board", "Screens", "The board pane on a sample board: every section, a deadline, the picked quest's detail, Accept."),
    ("hub", "Game menu", "Screens", "The in-game menu over the map."),
    ("journal", "Journal", "Screens", "Quests and notes."),
    ("stats", "Character: Stats", "Screens", "The character popup's Stats tab."),
    ("level_up", "Character: Level up", "Screens", "The character popup's Level up tab."),
    ("skills", "Character: Skills", "Screens", "The character popup's Skills tab."),
    ("gear", "Character: Gear", "Screens", "The character popup's Gear tab."),
    ("world_map", "World map", "Screens", "Travel between locations."),
    ("results", "Battle results", "Screens", "After a fight: outcome, XP, rewards."),
    ("battle", "Battle (whole)", "Screens", "The whole battle screen, planning (the Test Arena's encounter)."),
    ("battle_talk", "Battle talk", "Screens", "A talk over the battle: the veil, the talk box with its portrait, the speaker's ring."),
    ("strip", "Plan strip", "Battle panels", "The selected hero's blocks and the round controls."),
    ("party", "Party list", "Battle panels", "The heroes' cards."),
    ("hero", "Hero inspector", "Battle panels", "The selected hero's details and plan."),
    ("grid", "Battlefield", "Battle panels", "The grid: terrain, units, intents, the plan ghost."),
    ("hostiles", "Hostile list", "Battle panels", "The enemies' cards with what each does next."),
    ("intel", "Threat intel", "Battle panels", "The hovered (or first) enemy's details and threat."),
    ("bar", "Status and log", "Battle panels", "Playback buttons, round and status, the reduced log."),
    ("timeline", "Battle timeline", "Battle panels", "Every unit's track over the round's blocks (live battle, round archive)."),
];
/// Screens a preview cannot show without a game.
const NEEDS_GAME: [&str; 13] = ["region", "dialogue", "shop", "hub", "journal", "stats", "level_up", "skills", "gear", "world_map", "results", "save", "battle_talk"];
/// Where a preview's mouse is (off every widget).
const OFF: (i32, i32) = (-10_000, -10_000);
const ALLY: Rgba = [110, 170, 255, 255];
/// The strip under the timeline: an action's details or a block's snapshot.
const DETAIL_H: i32 = 64;

#[derive(Debug, Clone)]
pub enum GAct {
    Entry(usize),
    Kit(KitAct),
    /// The vignette entry's reel (STORYUI1-U1: the sample, then every world's vignettes).
    Reel(usize),
    /// The Portraits entry (PORTRAITFX1-G).
    Portrait(PAct),
    /// Shows the previewed screen itself.
    Open,
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
    /// INPUT1 TV7: the Timeline pane entry's block and track (a click selects, again unselects).
    PaneBlock(i32),
    PaneTrack(String),
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

/// A previewed screen and the clicks still to make on it (to open a dialogue or a shop).
struct Shown {
    id: &'static str,
    screen: Box<dyn Screen<GameCx>>,
    steps: Vec<Step>,
    tries: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Step {
    /// Pick the open dialogue's choice of this service.
    Service(&'static str),
}

/// The Portraits entry's controls (PORTRAITFX1-G 6.3; ids `gallery.portraits.*`).
#[derive(Debug, Clone)]
pub enum PAct {
    /// A portrait (an index into the page's files) with an expression (0 = neutral, else its frame `n - 1`).
    Pick(usize, usize),
    /// An overlay or state of the Kit widgets' portrait page (`PORTRAIT_OVERLAYS`, `PORTRAIT_STATES`), held on.
    Toggle(&'static str),
    /// A reaction of the Kit widgets' portrait page (`PORTRAIT_REACTIONS`).
    Fire(&'static str),
    /// A step event (`PORTRAIT_EVENTS`): the data's event rows as the battle panel fires them.
    Event(&'static str),
    /// A cue row by id: an event row fires, a state row toggles.
    Cue(String),
    Size(usize),
    Reduce,
    Anchors,
    /// The expression picker (0 = neutral, else the picked portrait's frame `n - 1`).
    Expression(usize),
    /// PORTRAITFX1-EX: an expression-effect row by id: holds its expression on the portrait (again: off).
    ExprFx(String),
}

/// The folders under `portraits/`, in list order.
const PORTRAIT_FOLDERS: [&str; 5] = ["heroes", "npcs", "elites", "bosses", "common"];
/// The Portraits entry's event buttons fire the rows as this surface does, under this ruleset.
const PORTRAIT_EVENT_SURFACE: &str = "panel";

/// One portrait of the list: its file (`portraits/<folder>/<id>.png`; a character with only expression frames keeps
/// this path), the layer holding it and its expression frames (in the game's face order, `Catalog::faces`).
#[derive(Debug, Clone, PartialEq)]
struct PortraitFile {
    rel: String,
    folder: String,
    id: String,
    layer: String,
    frames: Vec<String>,
}

/// Every portrait of `layers` (a file once: the first layer holding it), by folder, with its frames (of `faces`, in
/// that order; PORTRAITFX2 FL4).
fn portrait_files(layers: &Layers, faces: &[String]) -> Vec<PortraitFile> {
    let layer_of = |rel: &str| {
        let i = layers.roots.iter().position(|r| aegis_core::vfs::is_file(r.join(rel)));
        let names = if layers.roots.len() == 3 { ["game", "kit", "engine"] } else { ["", "", ""] };
        i.map(|i| names.get(i).filter(|n| !n.is_empty()).map_or_else(|| format!("layer {}", i + 1), |n| n.to_string())).unwrap_or_default()
    };
    let mut out = Vec::new();
    for folder in PORTRAIT_FOLDERS {
        let dir = format!("portraits/{folder}");
        let mut ids: Vec<String> = layers.list(&dir).iter().filter_map(|f| f.strip_suffix(".png")).map(str::to_string).collect();
        let faced = if folder == "common" { Vec::new() } else { layers.dirs(&format!("{dir}/faces")) };
        for id in &faced {
            if !ids.contains(id) {
                ids.push(id.clone());
            }
        }
        ids.sort();
        for id in ids {
            let rel = format!("{dir}/{id}.png");
            let listed = layers.list(&format!("{dir}/faces/{id}"));
            let frames: Vec<String> = faces.iter().filter(|e| listed.contains(&format!("{e}.png"))).cloned().collect();
            let at = if layers.has(&rel) { rel.clone() } else { frames.first().map(|e| format!("{dir}/faces/{id}/{e}.png")).unwrap_or_default() };
            out.push(PortraitFile { layer: layer_of(&at), rel, folder: folder.to_string(), id, frames });
        }
    }
    out
}

/// A sidecar in one paragraph: its eyes, blink, anchors, rim and idle overlays.
fn face_summary(face: Option<&aegis_core::assets::Face>) -> String {
    let Some(f) = face else { return "No sidecar: no eyes, no overlays (base breath and rim only).".into() };
    let mut out = format!("Sidecar {} px, border {}: {} eye box{}", f.size, f.border, f.eyes.len(), if f.eyes.len() == 1 { "" } else { "es" });
    if !f.eyes.is_empty() {
        out += if f.blink { ", blinks" } else { ", no blink" };
    }
    out += &format!("; {} glow, {} line root{}", f.glow.len(), f.lines.len(), if f.lines.len() == 1 { "" } else { "s" });
    if let Some(c) = f.rim {
        out += &format!("; rim #{:02x}{:02x}{:02x}", c[0], c[1], c[2]);
    }
    let idle: Vec<String> = f.idle.iter().map(|o| format!("{} {:.2}", o.effect.replace('_', " "), o.intensity)).collect();
    out += &if idle.is_empty() { "; no idle overlay.".to_string() } else { format!("; idle: {}.", idle.join(", ")) };
    out
}

/// A cue row's hover text: its trigger and match, then its effect and numbers, colour, expression and limits.
fn cue_text(r: &emberwake_data::PortraitCue) -> String {
    let when = match r.trigger.as_str() {
        "event" => format!("on {}", r.event.replace('_', " ")),
        "status" => format!("while {}", r.status),
        "condition" => format!("while {}", r.condition),
        "hp" => format!("below {}% HP", r.below),
        "derived" if r.below > 0 => format!("{} below {}", r.stat, r.below),
        "derived" => format!("{} down {}%", r.stat, r.drop_pct),
        t => format!("while {t}"),
    };
    let mut what = format!("{} {:.2}", r.effect.replace('_', " "), r.intensity);
    if r.ms > 0 {
        what += &format!(", {} ms", r.ms);
    }
    for extra in [&r.preset, &r.expression] {
        if !extra.is_empty() {
            what += &format!(", {extra}");
        }
    }
    if !r.color.is_empty() {
        let name = hue_name(&r.color);
        what += &format!(", {}", if name.is_empty() { r.color.clone() } else { format!("{name} {}", r.color) });
    }
    let only: Vec<String> = r.surfaces.iter().chain(r.rulesets.iter()).cloned().collect();
    format!("{when}: {what}{}", if only.is_empty() { String::new() } else { format!(" (only {})", only.join(", ")) })
}

/// An expression-effect row's hover text: its tint, mark and shake.
fn expr_text(r: &emberwake_data::PortraitExpression) -> String {
    let mut parts = Vec::new();
    let colour = |c: &str| {
        let name = hue_name(c);
        if name.is_empty() { c.to_string() } else { format!("{name} {c}") }
    };
    if !r.tint.is_empty() {
        let len = if r.tint_ms > 0 { format!(", fading over {} ms", r.tint_ms) } else { String::new() };
        parts.push(format!("tint {} {:.2} over the face box{len}", colour(&r.tint), r.tint_strength));
    }
    if !r.mark.is_empty() {
        let c = if r.mark_color.is_empty() { String::new() } else { format!(" {}", colour(&r.mark_color)) };
        let pulse = if r.mark_pulse_ms > 0 { format!(", pulsing every {} ms", r.mark_pulse_ms) } else { String::new() };
        parts.push(format!("{} mark{c} at ({:.2}, {:.2}) of the {} box{pulse}", r.mark, r.mark_x, r.mark_y, r.mark_box));
    }
    if r.shake_px > 0 {
        parts.push(format!("shake {} px for {} ms", r.shake_px, r.shake_ms));
    }
    if parts.is_empty() {
        format!("{}: nothing.", r.id)
    } else {
        format!("While {} holds: {}. Reduce motion stops the shake and the pulse.", r.id, parts.join("; "))
    }
}

/// The Portraits entry (6.3): the list, the picked portrait and the controls; its portraits draw through `Faces`.
#[derive(Debug, Clone, Default)]
struct PortraitPage {
    /// The portraits of the art layers (read at the first draw).
    files: Option<Vec<PortraitFile>>,
    pick: usize,
    /// 0 = neutral, else the picked portrait's frame `n - 1` (held as a `face` cue under every cue row's priority).
    expression: usize,
    faces: Faces,
    /// The overlays and states toggled on, and the state cue rows toggled on (by id).
    on: std::collections::BTreeSet<&'static str>,
    rows_on: std::collections::BTreeSet<String>,
    /// PORTRAITFX1-EX: the expression-effect row held on (its expression over the picked frame).
    fx_on: Option<String>,
    /// `PORTRAIT_SIZES` index.
    size: usize,
    /// What the last control did (the foot line).
    last: String,
}

impl PortraitPage {
    fn picked(&self) -> Option<&PortraitFile> {
        self.files.as_ref().and_then(|f| f.get(self.pick))
    }

    /// The `Faces` key of the picked portrait (its file).
    fn key(&self) -> String {
        self.picked().map(|f| f.rel.clone()).unwrap_or_default()
    }

    /// The cues the page holds on the picked portrait: the toggles, the toggled state rows, the picked expression.
    fn pins(&self, rows: &[emberwake_data::PortraitCue]) -> Vec<Cue> {
        let mut out: Vec<Cue> = self.on.iter().filter_map(|e| Cue::parse(e, "").map(|c| c.with_key(&format!("gallery.{e}")))).collect();
        out.extend(rows.iter().filter(|r| !r.is_event() && self.rows_on.contains(&r.id)).filter_map(cue_of));
        if let Some(e) = self.picked().and_then(|f| f.frames.get(self.expression.wrapping_sub(1))) {
            out.extend(Cue::parse("face", "").map(|c| c.with_expression(e).with_priority(0).with_key("gallery.expression")));
        }
        // PORTRAITFX1-EX: the held expression-effect row's face, over the picker (its effects run from the click)
        if let Some(e) = &self.fx_on {
            out.extend(Cue::parse("face", "").map(|c| c.with_expression(e).with_priority(1).with_key("gallery.expression_fx")));
        }
        out
    }

    fn act(&mut self, cx: &mut Cx, a: PAct) {
        let key = self.key();
        self.last = match a {
            PAct::Pick(i, e) => {
                self.pick = i;
                self.expression = e;
                format!("picked {}", self.key())
            }
            PAct::Toggle(e) => {
                let on = !self.on.remove(e);
                if on {
                    self.on.insert(e);
                }
                format!("{} {}", e.replace('_', " "), if on { "on" } else { "off" })
            }
            PAct::Fire(what) => {
                let cue = if what == "flash" { Cue::parse("flash", "") } else { Cue::parse("jolt", what.trim_start_matches("jolt.")) };
                if let Some(c) = cue {
                    self.faces.fire(&key, c);
                }
                what.replace('.', " ")
            }
            PAct::Event(ev) => {
                let n = self.faces.react(cx, &key, &[ev], PORTRAIT_EVENT_SURFACE, emberwake_rules::ruleset::CLASSIC);
                let skip = cx.settings.v["skip_anim"].as_bool().unwrap_or(false);
                format!("{}: {n} row{} fired{}", ev.replace('_', " "), if n == 1 { "" } else { "s" }, if skip { " (skip animations is on)" } else { "" })
            }
            PAct::Cue(id) => match cx.data.cat.portrait_cues.iter().find(|r| r.id == id) {
                Some(r) if r.is_event() => {
                    if let Some(c) = cue_of(r) {
                        self.faces.fire(&key, c);
                    }
                    format!("{id} fired")
                }
                Some(_) => {
                    let on = !self.rows_on.remove(&id);
                    if on {
                        self.rows_on.insert(id.clone());
                    }
                    format!("{id} {}", if on { "on" } else { "off" })
                }
                None => format!("no cue row {id}"),
            },
            PAct::Size(i) => {
                self.size = i % PORTRAIT_SIZES.len();
                format!("size {} px", PORTRAIT_SIZES[self.size])
            }
            PAct::Reduce => {
                let on = !self.faces.reduce.unwrap_or(false);
                self.faces.reduce = Some(on);
                format!("reduce motion {}", if on { "on" } else { "off" })
            }
            PAct::Anchors => {
                self.faces.show_anchors = !self.faces.show_anchors;
                format!("anchors {}", if self.faces.show_anchors { "on" } else { "off" })
            }
            PAct::Expression(e) => {
                self.expression = e;
                format!("expression {}", self.picked().and_then(|f| f.frames.get(e.wrapping_sub(1))).map_or("neutral", |s| s.as_str()))
            }
            PAct::ExprFx(id) => {
                let on = self.fx_on.as_deref() != Some(id.as_str());
                self.fx_on = on.then(|| id.clone());
                format!("{id} effect {}", if on { "on" } else { "off" })
            }
        };
    }

    /// Draws the page into `r`: the list on the left, the stage with the foot line and the sidecar in the middle, the
    /// controls and the cue rows on the right.
    fn draw(&mut self, ui: &mut Ui<GAct>, cx: &mut Cx, r: Rect) {
        let files = self.files.get_or_insert_with(|| portrait_files(cx.art.layers(), &cx.data.cat.faces())).clone();
        let data = cx.data.clone();
        let rows = &data.cat.portrait_cues;
        let line = ui.p.line_h(SMALL).ceil() as i32;
        let row_h = (ui.p.line_h(BODY) + 2.0).ceil() as i32;
        ui.p.text(r.x as f32 + 4.0, r.y as f32 + 2.0, "Portraits", TITLE, th().gold);
        let nw = ui.p.text_w("Portraits", TITLE).ceil() as i32;
        let count = format!("{} portraits (game and kit layers); {} cue rows (portrait_cues.json)", files.len(), rows.len());
        ui.p.text_fit((r.x + 12 + nw) as f32, r.y as f32 + 3.0, &count, SMALL, (r.w - nw - 16) as f32, th().dim);
        let top = r.y + 4 + ui.p.line_h(TITLE).ceil() as i32 + 3;

        // -- The list: a heading per folder, a row per portrait, its frames indented under it
        let list = Rect::new(r.x + 2, top, 126, r.y + r.h - top - 2);
        let mut content = 0;
        let mut last_folder = "";
        for f in &files {
            if f.folder != last_folder {
                content += line + 2;
                last_folder = &f.folder;
            }
            content += (row_h + 1) * (1 + f.frames.len() as i32);
        }
        let off = ui.scroll_begin("gallery.portraits.list", list, content + 4);
        let mut y = list.y - off;
        let mut last_folder = "";
        for (i, f) in files.iter().enumerate() {
            if f.folder != last_folder {
                ui.p.text(list.x as f32 + 2.0, y as f32 + 1.0, &f.folder, SMALL, th().info);
                y += line + 2;
                last_folder = &f.folder;
            }
            let picked = i == self.pick;
            let about = format!("{} ({} layer){}", f.rel, f.layer, if f.frames.is_empty() { String::new() } else { format!("; faces: {}", f.frames.join(", ")) });
            ui.row(&format!("gallery.portraits.pick.{}.{}", f.folder, f.id), Rect::new(list.x, y, list.w - 4, row_h), &f.id, if picked { th().gold } else { th().text }, Some(GAct::Portrait(PAct::Pick(i, 0))), picked && self.expression == 0, tip(&f.id, &about));
            y += row_h + 1;
            for (k, e) in f.frames.iter().enumerate() {
                let on = picked && self.expression == k + 1;
                ui.row(&format!("gallery.portraits.pick.{}.{}.{e}", f.folder, f.id), Rect::new(list.x + 10, y, list.w - 14, row_h), e, if on { th().gold } else { th().dim }, Some(GAct::Portrait(PAct::Pick(i, k + 1))), on, tip(e, &format!("{} as {e}", f.id)));
                y += row_h + 1;
            }
        }
        ui.scroll_end();

        // -- The stage, the foot line, the sidecar
        let x2 = list.x + list.w + 8;
        let col_w = 150;
        let stage = Rect::new(x2, top, PORTRAIT_STAGE, PORTRAIT_STAGE);
        ui.p.panel(stage, th().game("gallery_stage", rgb(14, 15, 20)), th().edge, 2.0);
        let size = PORTRAIT_SIZES[self.size.min(PORTRAIT_SIZES.len() - 1)];
        let face_r = Rect::new(stage.x + (stage.w - size) / 2, stage.y + (stage.h - size) / 2, size, size);
        let key = self.key();
        self.faces.begin();
        let pins = self.pins(rows);
        self.faces.pin(&key, pins);
        let mut text = vec![(String::from("No portrait files in the art layers."), th().dim)];
        if let Some(f) = files.get(self.pick) {
            let who = Who::file(&f.rel);
            let look = emberwake_battle::portrait::Look { ruleset: emberwake_rules::ruleset::CLASSIC.into(), ..Default::default() };
            self.faces.draw(&mut ui.p, cx, &who, &look, PORTRAIT_EVENT_SURFACE, face_r);
            let now = self.faces.now();
            let fx = self.faces.get(&key);
            let tier = fx.and_then(|x| x.tier()).map_or("", Tier::name);
            let live = fx.map(|x| x.live(now)).unwrap_or_default();
            let device = (size as f32 * ui.p.v.s).round() as i32;
            let expression = fx.map(|x| x.expression(now).to_string()).unwrap_or_default();
            let drawn = cx.art.first_rel(&who.chain(&expression)).unwrap_or_default();
            let face = crate::portraits::sidecar(&mut cx.art, &drawn);
            let png = cx.art.size_of(&drawn).map_or(String::new(), |(w, h)| format!(", {w} x {h}"));
            text = vec![
                (format!("Tier {tier} ({device} device px). Live: {}", if live.is_empty() { "nothing".to_string() } else { live.join(", ") }), th().dim),
                (format!("{} ({} layer{png}){}", if drawn.is_empty() { f.rel.clone() } else { drawn }, f.layer, if expression.is_empty() { String::new() } else { format!(", face {expression}") }), th().text),
                (face_summary(face.as_ref()), th().text),
            ];
        }
        ui.push("gallery.portraits.face", "portrait", format!("{key}: {size} px"), stage, true, false, None, Vec::new());
        let mut fy = stage.y + stage.h + 4;
        for (t, col) in &text {
            fy = ui.para(x2, fy, col_w, t, SMALL, *col) + 3;
        }
        if !self.last.is_empty() {
            ui.para(x2, fy + 2, col_w, &format!("Last: {}", self.last), SMALL, th().info);
        }

        // -- The controls
        let x3 = x2 + col_w + 10;
        let right = r.x + r.w - 4;
        let head = |ui: &mut Ui<GAct>, y: i32, t: &str| -> i32 {
            ui.p.text(x3 as f32, y as f32, t, SMALL, th().info);
            y + line + 1
        };
        let label = |s: &str| {
            let s = s.replace(['_', '.'], " ");
            s[..1].to_uppercase() + &s[1..]
        };
        let mut yy = top;
        for (title, names) in [("Overlays (held at full strength)", &PORTRAIT_OVERLAYS[..]), ("States", &PORTRAIT_STATES[..])] {
            yy = head(ui, yy, title);
            let mut bx = x3;
            for name in names.iter().copied() {
                // an overlay keeps the label it always had (`PORTRAIT_OVERLAY_LABELS`, FB-206 (2b))
                let l = if PORTRAIT_OVERLAYS.contains(&name) { portrait_label(name) } else { label(name) };
                let w = 12 + ui.p.text_w(&l, BODY).ceil() as i32 + 4;
                if bx + w > right && bx > x3 {
                    bx = x3;
                    yy += 14;
                }
                ui.checkbox(&format!("gallery.portraits.{name}"), bx, yy + 2, &l, self.on.contains(name), GAct::Portrait(PAct::Toggle(name)), true);
                bx += w + 6;
            }
            yy += 14 + 3;
        }
        // reactions and step events: a row of buttons each
        let buttons = |ui: &mut Ui<GAct>, yy: &mut i32, items: Vec<(String, String, GAct, Tip)>| {
            let mut bx = x3;
            for (id, l, act, t) in items {
                let w = (ui.p.text_w(&l, SMALL) + 10.0).ceil() as i32;
                if bx + w > right && bx > x3 {
                    bx = x3;
                    *yy += 14;
                }
                ui.button_full(&id, Rect::new(bx, *yy, w, 12), &l, act, true, false, SMALL, t);
                bx += w + 3;
            }
            *yy += 12 + 4;
        };
        yy = head(ui, yy, "Reactions");
        let items = PORTRAIT_REACTIONS.iter().map(|w| (format!("gallery.portraits.{w}"), label(w), GAct::Portrait(PAct::Fire(w)), tip(&label(w), "The engine's reaction at full strength."))).collect();
        buttons(ui, &mut yy, items);
        yy = head(ui, yy, "Step events (the cue rows, as the battle panel)");
        let items = emberwake_data::PORTRAIT_EVENTS.iter().map(|e| (format!("gallery.portraits.event.{e}"), label(e), GAct::Portrait(PAct::Event(e)), tip(&label(e), "Fires every event row of this event, as a battle step does."))).collect();
        buttons(ui, &mut yy, items);
        yy = head(ui, yy, "Cue rows (events fire, states toggle)");
        if rows.is_empty() {
            yy = ui.para(x3, yy, right - x3, "No cue rows yet (data/catalog/portrait_cues.json, PORTRAITFX1-A).", SMALL, th().dim) + 3;
        } else {
            let mut bx = x3;
            for row in rows.iter() {
                let id = format!("gallery.portraits.cue.{}", row.id);
                let t = tip(&row.id, &cue_text(row));
                if row.is_event() {
                    let w = (ui.p.text_w(&row.id, SMALL) + 10.0).ceil() as i32;
                    if bx + w > right && bx > x3 {
                        bx = x3;
                        yy += 14;
                    }
                    ui.button_full(&id, Rect::new(bx, yy, w, 12), &row.id, GAct::Portrait(PAct::Cue(row.id.clone())), true, false, SMALL, t);
                    bx += w + 3;
                } else {
                    let w = 12 + ui.p.text_w(&row.id, BODY).ceil() as i32 + 4;
                    if bx + w > right && bx > x3 {
                        bx = x3;
                        yy += 14;
                    }
                    ui.checkbox(&id, bx, yy + 1, &row.id, self.rows_on.contains(&row.id), GAct::Portrait(PAct::Cue(row.id.clone())), true);
                    if let Some(box_) = ui.widgets.last_mut() {
                        box_.tip = t;
                    }
                    bx += w + 6;
                }
            }
            yy += 14 + 3;
        }
        // PORTRAITFX1-EX: one box per expression-effect row (a click holds its expression, a second click releases it)
        yy = head(ui, yy, "Expression effects (held as the face)");
        let fx_rows = &data.cat.portrait_expressions;
        if fx_rows.is_empty() {
            yy = ui.para(x3, yy, right - x3, "No expression-effect rows (data/catalog/portrait_expressions.json).", SMALL, th().dim) + 3;
        } else {
            let mut bx = x3;
            for row in fx_rows.iter() {
                let w = 12 + ui.p.text_w(&row.id, BODY).ceil() as i32 + 4;
                if bx + w > right && bx > x3 {
                    bx = x3;
                    yy += 14;
                }
                let id = format!("gallery.portraits.fx.{}", row.id);
                ui.checkbox(&id, bx, yy + 1, &row.id, self.fx_on.as_deref() == Some(row.id.as_str()), GAct::Portrait(PAct::ExprFx(row.id.clone())), true);
                if let Some(box_) = ui.widgets.last_mut() {
                    box_.tip = tip(&row.id, &expr_text(row));
                }
                bx += w + 6;
            }
            yy += 14 + 3;
        }
        yy = head(ui, yy, "Size, motion, anchors, face");
        let sizes: Vec<String> = PORTRAIT_SIZES.iter().map(|s| format!("{s} px")).collect();
        let sw = (ui.p.text_w("96 px", BODY) + 22.0).ceil() as i32;
        ui.select("gallery.portraits.size", Rect::new(x3, yy, sw, 14), &sizes, self.size, |i| GAct::Portrait(PAct::Size(i)), true, tip("Size", "The portrait's side in logical px; the tier follows the device px."));
        let reduce = self.faces.reduce.unwrap_or(false);
        ui.checkbox("gallery.portraits.reduce", x3 + sw + 8, yy + 2, "Reduce motion", reduce, GAct::Portrait(PAct::Reduce), true);
        yy += 14 + 4;
        ui.checkbox("gallery.portraits.anchors", x3, yy + 2, "Anchors", self.faces.show_anchors, GAct::Portrait(PAct::Anchors), true);
        let frames = files.get(self.pick).map(|f| f.frames.clone()).unwrap_or_default();
        if !frames.is_empty() {
            let options: Vec<String> = std::iter::once("neutral (portrait)".to_string()).chain(frames.iter().cloned()).collect();
            let ax = x3 + 12 + ui.p.text_w("Anchors", BODY).ceil() as i32 + 12;
            let ew = (ui.p.text_w("neutral (portrait)", BODY) + 22.0).ceil() as i32;
            ui.select("gallery.portraits.expression", Rect::new(ax, yy, ew, 14), &options, self.expression.min(frames.len()), |i| GAct::Portrait(PAct::Expression(i)), true, tip("Expression", "The face frame drawn (a face cue row of a higher priority wins)."));
        }
    }
}

pub struct Gallery {
    entry: usize,
    /// The vignette entry's reel (an index into `story::gallery_reels`).
    reel: usize,
    demo: Option<Demo>,
    kit: Specimens,
    portraits: PortraitPage,
    shown: Option<Shown>,
    battle: Option<BattleScreen>,
    /// INPUT1 TV7: the Timeline pane entry's selected block and track (the sample's to start).
    pane: (Option<i32>, Option<String>),
    /// The sample game is the gallery's (it goes when the gallery does), with the debug flag it found.
    own_game: Option<bool>,
    w: Vec<Widget<GAct>>,
}

impl Default for Gallery {
    fn default() -> Self {
        Self::new()
    }
}

impl Gallery {
    /// PR5-48: the Kit widgets entry shows its Layout page (the layout editor).
    fn on_layout_page(&self) -> bool {
        ENTRIES[self.entry].0 == "kit" && aegis_ui::specimens::PAGES.get(self.kit.page) == Some(&"Layout")
    }

    pub fn new() -> Self {
        let s = aegis_ui::timeline::sample();
        Self { entry: 0, reel: 0, demo: None, kit: Specimens::default(), portraits: PortraitPage::default(), shown: None, battle: None, pane: (s.selected_block, s.selected_track), own_game: None, w: Vec::new() }
    }

    /// The sample game for the screen entries: a debug game (nothing saved) of the first playable world.
    fn ensure_game(&mut self, cx: &mut Cx) -> bool {
        if cx.game.is_some() {
            return true;
        }
        let worlds = emberwake_world::new_game::playable_worlds(&cx.data);
        let Some(w) = worlds.first() else { return false };
        let Ok(mut gs) = emberwake_world::new_game::create_for_world(&cx.data, w, 1) else { return false };
        // STORYUI1-U1: the sample game shows the screens, not the world's start sequence (its map hands over to the
        // story stage while one runs)
        gs.story = Default::default();
        cx.game = Some(Game::new(gs, -1, true));
        self.own_game = Some(cx.debug);
        cx.debug = true;
        true
    }

    /// The vignette entry's picked reel (the sample when the pick is gone).
    fn reel(&self, cx: &Cx) -> vignette::Reel {
        crate::story::gallery_reels(&cx.data).into_iter().nth(self.reel).map(|r| r.1).unwrap_or_else(vignette::sample_reel)
    }

    fn build(&self, cx: &mut Cx, id: &str) -> Option<(Box<dyn Screen<GameCx>>, Vec<Step>)> {
        let member = cx.game.as_ref().and_then(|g| g.gs.party.members.first()).map(|m| m.id.clone()).unwrap_or_default();
        let screen: Box<dyn Screen<GameCx>> = match id {
            "layout" => Box::new(aegis_ui::layout_specimen::LayoutSpecimen::default()),
            "main_menu" => Box::new(crate::menus::MainMenu::default()),
            "new_game" => Box::new(crate::menus::NewGameScreen::new(cx)),
            "settings" => Box::new(crate::menus::SettingsScreen::new(cx)),
            "save" => Box::new(crate::menus::Slots::new(true)),
            "load" => Box::new(crate::menus::Slots::new(false)),
            "region" => Box::new(crate::region::RegionMap::new(cx)),
            "dialogue" | "shop" => Box::new(Self::talking(cx, id == "shop")),
            "battle_talk" => Box::new(Self::battle_talk(cx)),
            // STORYUI1-U2: the pane on a fixture board (no game needed)
            "board" => Box::new(crate::board::BoardSample::new()),
            "hub" => Box::new(crate::hub::Hub::default()),
            "journal" => Box::new(crate::hub::Journal::default()),
            "stats" | "level_up" | "skills" | "gear" => Box::new(crate::party::Character::new(cx, &member, id)),
            "world_map" => Box::new(crate::worldmap::WorldMap::new()),
            // opened full screen: the reel's end goes back to the gallery
            "vignette" => Box::new(crate::story::vignette_screen(cx, self.reel(cx), |cx: &mut Cx, _| cx.nav.push(Nav::Pop))),
            "results" => {
                let level = cx.game.as_ref().and_then(|g| g.gs.party.members.first()).map(|m| m.level(&cx.data)).unwrap_or(1);
                let mut xp = json::Dict::new();
                xp.insert(member.clone(), json!({"xp": 24, "old_level": level, "new_level": level, "levels_gained": 0, "unspent": 0}));
                // INTENT2-U: a sample "Stash short" line, as the kit's post-battle auto-fill writes it (`roster::auto_fill`)
                let name = cx.game.as_ref().and_then(|g| g.gs.party.members.first()).map(|m| emberwake_world::progression::display_name(&cx.data, m)).unwrap_or_default();
                let item = cx.data.cat.items.values().filter(|i| i.offered_in_battle()).map(|i| i.name.clone()).min().unwrap_or_default();
                let short = format!("Stash short: {name} 1 of 2 {item}");
                let res = json::obj([("ok", true.into()), ("xp", Value::Object(xp)), ("advanced", json!([])), ("events", json!([])), ("gold", json!(30)), ("stash_short", json!([short]))]);
                Box::new(crate::hub::Results::new(&res, "won", &json::Dict::new()))
            }
            _ => return None,
        };
        let steps = if id == "shop" { vec![Step::Service("shop")] } else { Vec::new() };
        Some((screen, steps))
    }

    /// SERVICES1: the services on a plain backdrop (`talk::ServiceScreen`, no map) in conversation with the first
    /// person here (`shop`: the first who offers a shop); none: the backdrop alone.
    fn talking(cx: &mut Cx, shop: bool) -> crate::talk::ServiceScreen {
        // a new sample game enters its start location first (as the region map does)
        let d = cx.data.clone();
        let _ = emberwake_world::explore::resume(&d, cx.gs_mut());
        let people: Vec<String> = match cx.game.as_ref() {
            Some(g) => {
                let loc = cx.data.resolve(&g.gs.world_id, "locations", &g.gs.location_id);
                emberwake_world::explore::markers_of(loc).iter().filter(|m| json::s(m, "kind") == "npc").map(|m| json::s(m, "character")).collect()
            }
            None => Vec::new(),
        };
        let mut found = None;
        for who in &people {
            let mut m = crate::talk::ServiceScreen::default();
            m.talk(cx, who);
            let offers = Self::services(&m.state(cx));
            if !offers.is_empty() && (!shop || offers.iter().any(|c| c == "shop")) {
                found = Some(m);
                break;
            }
        }
        // a preview's conversation never ends, so it must not hold the game busy
        cx.set_busy(false);
        found.unwrap_or_default()
    }

    /// ENCOUNTER2-C2: the Test Arena's battle with the first dialogue tree of the sample game's world that opens as a
    /// scene shown over it (none opens: the battle, the reason in its message bar).
    fn battle_talk(cx: &mut Cx) -> BattleScreen {
        let mut b = BattleScreen::arena(cx, None, &cx.encounter.clone());
        let world = cx.game.as_ref().map(|g| g.gs.world_id.clone()).unwrap_or_default();
        let trees: Vec<String> = cx.data.own_entries(&world, "dialogues").iter().map(|t| json::s(t, "id")).collect();
        for id in trees {
            if b.preview_talk(cx, &world, &id, "") {
                break;
            }
        }
        b
    }

    /// The services of the open dialogue's choices in a region map's state (none when no dialogue is open).
    fn services(st: &Value) -> Vec<String> {
        json::aget(json::dict(&st["dialogue"]), "choices").iter().map(|c| json::s(json::dict(c), "service")).collect()
    }

    /// One scripted click on the previewed screen per frame (its widgets exist once it has drawn).
    fn script(&mut self, cx: &mut Cx) {
        let Some(sh) = self.shown.as_mut() else { return };
        let Some(step) = sh.steps.first().copied() else { return };
        sh.tries += 1;
        if sh.tries > 60 {
            sh.steps.clear();
            return;
        }
        let open = Self::services(&sh.screen.state(cx));
        match step {
            Step::Service(svc) => {
                if let Some(k) = open.iter().position(|c| c == svc) {
                    if sh.screen.click_id(cx, &format!("choice.{k}")).is_ok() {
                        sh.steps.remove(0);
                    }
                }
            }
        }
    }

    /// The battle the panel entries show: a Test Arena battle with the Balanced plan drafted.
    fn battle(&mut self, cx: &Cx) -> &mut BattleScreen {
        self.battle.get_or_insert_with(|| {
            let mut b = BattleScreen::arena(cx, None, &cx.encounter.clone());
            if let Some(s) = b.host.session.as_mut() {
                s.plan_apply_preset_all("balanced");
            }
            b
        })
    }

    /// Draws entry `id` into `r`: a screen shrunk, or a battle panel enlarged.
    fn preview(&mut self, cx: &mut Cx, g: &mut Gfx, id: &'static str, r: Rect) {
        if id == "battle" || ENTRIES.iter().any(|e| e.0 == id && e.2 == "Battle panels") {
            let b = self.battle(cx);
            let whole = Rect::new(0, 0, W, aegis_ui::ui::H);
            let src = if id == "battle" { whole } else { b.panels.iter().find(|p| p.0 == id).map(|p| p.1).unwrap_or(whole) };
            // a panel is enlarged at most twice, centred in the box
            let k = (r.w as f32 / src.w.max(1) as f32).min(r.h as f32 / src.h.max(1) as f32).min(2.0);
            let (w, h) = ((src.w as f32 * k) as i32, (src.h as f32 * k) as i32);
            let target = Rect::new(r.x + (r.w - w) / 2, r.y + (r.h - h) / 2, w, h);
            preview::region_in(g, target, src, b.grid(), |g| b.draw(cx, g, OFF, false));
            return;
        }
        if NEEDS_GAME.contains(&id) && !self.ensure_game(cx) {
            return;
        }
        if self.shown.as_ref().is_none_or(|sh| sh.id != id) {
            self.shown = if id == "vignette" {
                // the preview's end changes nothing: `tick` starts it over
                let screen: Box<dyn Screen<GameCx>> = Box::new(crate::story::vignette_screen(cx, self.reel(cx), |_: &mut Cx, _| {}));
                Some(Shown { id, screen, steps: Vec::new(), tries: 0 })
            } else {
                self.build(cx, id).map(|(screen, steps)| Shown { id, screen, steps, tries: 0 })
            };
        }
        if let Some(sh) = self.shown.as_mut() {
            preview::screen_in(g, r, sh.screen.grid(), |g| sh.screen.draw(cx, g, OFF, false));
        }
        self.script(cx);
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
            GAct::Entry(i) => {
                if i != self.entry {
                    // LOOK1-E1: the Kit page's theme switch goes back to the layers' theme.
                    self.kit.leave_theme();
                }
                self.entry = i;
                self.shown = None;
            }
            GAct::Kit(k) => self.kit.act(k),
            GAct::Portrait(p) => self.portraits.act(cx, p),
            GAct::Reel(i) => {
                self.reel = i;
                self.shown = None;
            }
            GAct::Open => {
                let id = ENTRIES[self.entry].0;
                if id == "battle" || ENTRIES[self.entry].2 == "Battle panels" {
                    let b = BattleScreen::arena(cx, None, &cx.encounter.clone());
                    cx.nav.push(Nav::Push(Box::new(b)));
                } else if !["kit", "portraits", "pip_gauge", "timeline_pane"].contains(&id) && (!NEEDS_GAME.contains(&id) || self.ensure_game(cx)) {
                    // a fresh one, so the preview keeps its own state
                    if let Some((screen, _)) = self.build(cx, id) {
                        cx.nav.push(Nav::Push(screen));
                    }
                }
            }
            GAct::Exit => cx.nav.push(Nav::Pop),
            GAct::Restart => self.demo = None,
            GAct::PaneBlock(b) => self.pane.0 = if self.pane.0 == Some(b) { None } else { Some(b) },
            GAct::PaneTrack(k) => self.pane.1 = if self.pane.1.as_deref() == Some(k.as_str()) { None } else { Some(k) },
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
                        d.detail = segment_detail(cx, s, d.viewed, &t, &seg);
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

/// INPUT1 TV7: the Battle timeline entry's view: the battle screen's timeline view (`timeline_view::build`, the one
/// builder) of round `viewed` under the player's settings (overlay groups, tells, VC4's Party foresight off), every
/// enemy unmasked with Reveal, block and track as clicked (an enemy track too).
pub(crate) fn battle_timeline(cx: &Cx, s: &Session, viewed: i32, reveal: bool, block: Option<i32>, track: Option<String>) -> Timeline {
    let v = &cx.settings.v;
    let opts = timeline_view::Opts {
        viewed,
        reveal_all: reveal,
        foresight_off: crate::battle::party_foresight_off(v, cfg!(debug_assertions)),
        show_tells: v["show_tells"].as_bool().unwrap_or(true),
        layers: crate::battle::Layers::of(&v["battle_overlays"]),
        selected: None,
        block,
        shown: None,
        playhead: None,
    };
    Timeline { selected_track: track, ..timeline_view::build(s, &opts) }
}

/// The hover card lines of a clicked segment, for the detail strip.
fn segment_detail(cx: &Cx, s: &Session, viewed: i32, track: &str, seg: &str) -> Vec<(String, Rgba)> {
    let t = battle_timeline(cx, s, viewed, true, None, None);
    t.sections
        .iter()
        .flat_map(|x| x.tracks.iter())
        .find(|x| x.key == track)
        .and_then(|x| x.segments.iter().find(|g| g.key == seg).map(|g| (x.label.clone(), g.tip.clone())))
        .map(|(unit, tip)| std::iter::once((unit, th().text)).chain(tip).collect())
        .unwrap_or_default()
}

/// INPUT1 TV7: the Timeline pane entry (Kit): the engine timeline widget alone on its sample round
/// (`aegis_ui::timeline::sample`, no game type and no session) under its name and what it shows; a click on a block or
/// a track selects it (again: unselects), a span's click selects its track; hover cards are the widget's own.
fn draw_timeline_pane(ui: &mut Ui<GAct>, r: Rect, name: &str, about: &str, sel: &(Option<i32>, Option<String>)) {
    ui.p.text(r.x as f32 + 4.0, r.y as f32 + 2.0, name, TITLE, th().gold);
    let nw = ui.p.text_w(name, TITLE).ceil() as i32;
    ui.p.text_fit((r.x + 12 + nw) as f32, r.y as f32 + 3.0, about, SMALL, (r.w - nw - 16) as f32, th().dim);
    let top = r.y + 4 + ui.p.line_h(TITLE).ceil() as i32 + 3;
    let t = Timeline { selected_block: sel.0, selected_track: sel.1.clone(), ..aegis_ui::timeline::sample() };
    t.draw(ui, "gallery.pane", Rect::new(r.x + 4, top, r.w - 8, r.y + r.h - 4 - top), GAct::PaneBlock, |k| GAct::PaneTrack(k.to_string()), |k, _| GAct::PaneTrack(k.to_string()));
}

/// INPUT1-AP: the Pip gauge entry (Kit): the engine's pip gauge alone, each sample state (`aegis_ui::pips::samples`)
/// in a row with its name, the gauge (`gallery.pip.<k>`, its numbers on hover) and its reading, under a legend.
fn draw_pip_gauges(ui: &mut Ui<GAct>, r: Rect, name: &str, about: &str) {
    ui.p.text(r.x as f32 + 4.0, r.y as f32 + 2.0, name, TITLE, th().gold);
    let nw = ui.p.text_w(name, TITLE).ceil() as i32;
    ui.p.text_fit((r.x + 12 + nw) as f32, r.y as f32 + 3.0, about, SMALL, (r.w - nw - 16) as f32, th().dim);
    let mut y = r.y + 4 + ui.p.line_h(TITLE).ceil() as i32 + 3;
    let legend = "F full: kept.  H hollow: spent by the plan.  Hg / Sg: a ghost fill, next round's gain.  S: a slot above what is held now.";
    ui.p.text_fit(r.x as f32 + 4.0, y as f32, legend, SMALL, (r.w - 8) as f32, th().text);
    let row_h = (ui.p.line_h(BODY) + 6.0).ceil() as i32;
    y += row_h;
    let (label_w, gauge_w) = (210, 90);
    for (k, (label, gauge)) in aegis_ui::pips::samples().into_iter().enumerate() {
        ui.p.text_fit(r.x as f32 + 4.0, y as f32, label, BODY, label_w as f32 - 8.0, th().text);
        let cy = y + (ui.p.line_h(BODY) / 2.0).round() as i32;
        gauge.draw(ui, &format!("gallery.pip.{k}"), r.x + label_w, cy);
        ui.p.text_fit((r.x + label_w + gauge_w) as f32, y as f32, &gauge.reading(), SMALL, (r.w - label_w - gauge_w - 6) as f32, th().dim);
        y += row_h;
    }
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
        ui.p.fill(Rect::new(0, 0, W, aegis_ui::ui::H), th().tool_bg);
        ui.p.text(6.0, 4.0, "UI GALLERY", BODY, th().gold);
        ui.button("gallery.exit", Rect::new(W - 36, 3, 32, 12), "Exit", GAct::Exit, true);
        // The entries by group; the list scrolls (the wheel over it).
        let list = Rect::new(4, 20, 102, aegis_ui::ui::H - 24);
        let row_h = (ui.p.line_h(BODY) + 2.0).ceil() as i32;
        let head_h = (ui.p.line_h(SMALL) + 3.0).ceil() as i32;
        let groups = ENTRIES.iter().enumerate().filter(|(i, e)| *i == 0 || ENTRIES[i - 1].2 != e.2).count() as i32;
        let content = ENTRIES.len() as i32 * (row_h + 1) + groups * head_h + 4;
        let off = ui.scroll_begin("gallery.entries", list, content);
        let mut y = list.y - off;
        for (i, (id, name, group, about)) in ENTRIES.iter().enumerate() {
            if i == 0 || ENTRIES[i - 1].2 != *group {
                ui.p.text(list.x as f32 + 2.0, y as f32 + 2.0, group, SMALL, th().info);
                y += head_h;
            }
            ui.row(&format!("gallery.entry.{id}"), Rect::new(list.x, y, list.w - 4, row_h), name, if i == self.entry { th().gold } else { th().text }, Some(GAct::Entry(i)), i == self.entry, tip(name, about));
            y += row_h + 1;
        }
        ui.scroll_end();
        let pane = Rect::new(110, 20, W - 114, aegis_ui::ui::H - 24);
        ui.p.panel(pane, th().tool_pane, th().edge, 2.0);
        let (id, name, _, about) = ENTRIES[self.entry];
        if id == "kit" {
            self.kit.draw(&mut ui, Rect::new(pane.x + 2, pane.y + 2, pane.w - 4, pane.h - 4), GAct::Kit);
        } else if id == "frame_tuner" {
            self.kit.tuner.draw(&mut ui, Rect::new(pane.x + 2, pane.y + 2, pane.w - 4, pane.h - 4), &|a| GAct::Kit(KitAct::Tuner(a)));
        } else if id == "portraits" {
            self.portraits.draw(&mut ui, cx, Rect::new(pane.x + 2, pane.y + 2, pane.w - 4, pane.h - 4));
        } else if id == "pip_gauge" {
            draw_pip_gauges(&mut ui, Rect::new(pane.x + 2, pane.y + 2, pane.w - 4, pane.h - 4), name, about);
        } else if id == "timeline_pane" {
            draw_timeline_pane(&mut ui, Rect::new(pane.x + 2, pane.y + 2, pane.w - 4, pane.h - 4), name, about, &self.pane);
        } else if id != "timeline" {
            // A screen or a battle panel: its name, what it is, Open, and the preview under them.
            ui.p.text(pane.x as f32 + 6.0, pane.y as f32 + 4.0, name, TITLE, th().gold);
            let ow = (ui.p.text_w("Open", SMALL) + 12.0).ceil() as i32;
            let nw = ui.p.text_w(name, TITLE).ceil() as i32;
            ui.p.text_fit((pane.x + 14 + nw) as f32, pane.y as f32 + 5.0, about, SMALL, (pane.w - nw - ow - 28) as f32, th().dim);
            ui.button_full("gallery.open", Rect::new(pane.x + pane.w - ow - 4, pane.y + 3, ow, 12), "Open", GAct::Open, true, false, SMALL, tip("Open", "Shows the screen itself to try it."));
            let mut top = pane.y + 4 + ui.p.line_h(TITLE).ceil() as i32 + 3;
            if id == "vignette" {
                // STORYUI1-U1 (V15): the reel to play, the engine's sample or a world's vignette as written
                let labels: Vec<String> = crate::story::gallery_reels(&cx.data).into_iter().map(|r| r.0).collect();
                let h = (ui.p.line_h(BODY) + 4.0).ceil() as i32;
                let t = tip("Reel", "The engine's sample reel, or a world's vignette (every card shown, tokens as written).");
                ui.select("gallery.vignette.reel", Rect::new(pane.x + 4, top, 260, h), &labels, self.reel.min(labels.len().saturating_sub(1)), GAct::Reel, labels.len() > 1, t);
                top += h + 3;
            }
            let shown = Rect::new(pane.x + 4, top, pane.w - 8, pane.y + pane.h - 4 - top);
            ui.p.fill(shown, th().game("gallery_well", rgb(6, 8, 14)));
            self.w = std::mem::take(&mut ui.widgets);
            drop(ui);
            self.preview(cx, g, id, shown);
            return;
        }
        if id == "timeline" {
            let d = self.demo.as_ref().unwrap();
            let Some(s) = d.host.session.as_ref() else {
                ui.p.text(pane.x as f32 + 6.0, pane.y as f32 + 6.0, "No battle (the arena encounter did not start).", BODY, th().dim);
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
            ui.p.text_fit(pane.x as f32 + 21.0, pane.y as f32 + 4.0, &format!("Round {}/{}  {phase}", d.viewed, s.current_round), TITLE, 120.0, if live_round { th().text } else { th().gold });
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
            let t = battle_timeline(cx, s, d.viewed, d.reveal, d.block, d.track.clone());
            t.draw(&mut ui, "gallery.timeline", Rect::new(pane.x + 4, pane.y + 20, pane.w - 8, pane.h - 24 - DETAIL_H), GAct::Block, |k| GAct::Track(k.to_string()), |k, g| GAct::Segment(k.to_string(), g.to_string()));
            // The strip under the timeline: a selected block's snapshot, else the clicked action, else a hint.
            let strip = Rect::new(pane.x + 4, pane.y + pane.h - DETAIL_H, pane.w - 8, DETAIL_H - 4);
            ui.p.fill(Rect::new(strip.x, strip.y - 2, strip.w, 1), th().edge);
            let snap = d.block.and_then(|b| s.round_record(d.viewed).and_then(|r| r.snapshots.get(b as usize)).map(|sn| (b, sn)));
            let lines: Vec<(String, Rgba)> = match (d.block, snap) {
                (Some(_), Some((b, sn))) => std::iter::once((format!("After block {b} of round {}", d.viewed), th().gold))
                    .chain(s.w.allies.iter().chain(s.w.enemies.iter()).filter_map(|&u| sn.get(u).filter(|x| x.pos.is_some() || x.hp <= 0 || x.withdrawn).map(|x| (snap_line(&s.w.units[u].name, x), if s.w.allies.contains(&u) { ALLY } else { th().foe }))))
                    .collect(),
                (Some(b), None) => vec![(format!("Block {b} has not resolved yet."), th().dim)],
                _ if !d.detail.is_empty() => d.detail.clone(),
                _ => vec![("Click an action for its details, a resolved block for the state after it; < > page through the rounds.".to_string(), th().dim)],
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
            "entry": ENTRIES[self.entry].0, "shown": self.shown.as_ref().map(|sh| sh.screen.id()), "steps_left": self.shown.as_ref().map(|sh| sh.steps.len()), "kit_last": self.kit.last, "tuner": self.kit.tuner.state(), "viewed": d.map(|d| d.viewed), "round": s.map(|s| s.current_round), "block": s.map(|s| s.current_block),
            "selected_block": d.and_then(|d| d.block), "planning": s.map(|s| s.phase == Phase::Planning && !s.plan_confirmed), "rounds_archived": s.map(|s| s.history.len()),
            // PORTRAITFX1-G (4.11): the Portraits entry's pick and its portraits' state
            "portrait_pick": self.portraits.key(), "portrait_last": self.portraits.last, "portraits": self.portraits.faces.state(),
            // PR5-48: the Kit widgets' Layout page
            aegis_ui::layout_editor::STATE_KEY: if self.on_layout_page() { self.kit.layout.state() } else { Value::Null },
        })
    }

    /// The sample game goes with the gallery, only while it is still the one in play (a game started or loaded from an
    /// opened screen replaces it before the gallery is left).
    fn leave(&mut self, cx: &mut Cx) {
        self.kit.leave_theme();
        if let Some(debug) = self.own_game.take() {
            if cx.game.as_ref().is_some_and(|g| g.sample) {
                cx.game = None;
                cx.debug = debug;
                cx.log.clear();
            }
        }
    }

    /// The Kit widgets entry takes its Panes page's keys (hotkeys, PageUp / PageDown, the ask's Enter, y and n) and its
    /// Layout page's (PR5-48: the arrows with Shift, + and -, Ctrl+Z).
    fn key(&mut self, cx: &mut Cx, k: Key) {
        if ENTRIES[self.entry].0 == "kit" {
            self.kit.key_with(k, cx.held.shift, cx.held.ctrl);
        }
        if ENTRIES[self.entry].0 == "frame_tuner" {
            self.kit.tuner.key(k);
        }
    }

    /// PR5-48: the Kit widgets' Layout page steers with the arrows (the layout editor's nudge).
    fn arrow_focus(&self) -> bool {
        !self.on_layout_page()
    }

    /// PR5-48: a handle drag on the Kit widgets' Layout page goes to its layout editor; any other press clicks.
    fn pointer(&mut self, cx: &mut Cx, id: &str, phase: PointerPhase, pos: (i32, i32)) {
        if self.on_layout_page() && self.kit.pointer(id, phase, pos) {
            return;
        }
        if phase == PointerPhase::Down {
            self.click(cx, pos);
        }
    }

    /// PR5-48: the Layout page's agent commands (`layout_markers`, `layout`).
    fn command(&mut self, _cx: &Cx, body: &Value) -> Option<Value> {
        if ENTRIES[self.entry].0 != "kit" {
            return None;
        }
        self.kit.command(body)
    }

    /// Esc answers the Panes page's ask (No) there; anywhere else it leaves the gallery.
    fn escape(&mut self, cx: &mut Cx) {
        if !(ENTRIES[self.entry].0 == "kit" && self.kit.key(Key::Esc)) && !(ENTRIES[self.entry].0 == "frame_tuner" && self.kit.tuner.key(Key::Esc)) {
            self.back(cx);
        }
    }

    /// Back from an opened screen: the previews are built again (a setting may have changed).
    fn resume(&mut self, _cx: &mut Cx) {
        self.shown = None;
    }

    /// The vignette preview plays on (only the top screen ticks) and starts over at its end; the Kit widgets' Effects
    /// page runs its clock (FX1-E), the Portraits entry its portraits' clock (PORTRAITFX1-G).
    fn tick(&mut self, cx: &mut Cx) {
        if ENTRIES[self.entry].0 == "kit" {
            self.kit.tick();
        }
        if ENTRIES[self.entry].0 == "portraits" {
            self.portraits.faces.tick();
        }
        let Some(sh) = self.shown.as_mut().filter(|sh| sh.id == "vignette") else { return };
        sh.screen.tick(cx);
        if sh.screen.state(cx)["done"] == json!(true) {
            self.shown = None;
        }
    }

    fn can_drop(&self, _cx: &Cx, from: &str, to: &str) -> bool {
        ENTRIES[self.entry].0 == "kit" && self.kit.can_drop(from, to)
    }

    fn accept_drop(&mut self, _cx: &mut Cx, from: &str, to: &str) {
        self.kit.accept_drop(from, to);
    }

    fn text_changed(&mut self, _cx: &mut Cx, id: &str, text: &str) {
        self.kit.text_changed(id, text);
    }

    widget_plumbing!();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PR5-48 (MAP-LAYOUT-ED): the real Gallery hands the Kit widgets' Layout page its handle drags (`Screen::pointer`),
    /// its keys with Shift (a coarse nudge) and its agent commands, steers no focus with the arrows there, and shows the
    /// editor's state as `layout_markers`; on another page none of that.
    #[test]
    fn the_gallery_layout_page_drags_nudges_with_shift_and_takes_commands() {
        use crate::hud::hud_tests::{game_cx, gfx};
        use aegis_ui::layout_editor::{NUDGE, STATE_KEY};
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut gallery = Gallery::new();
        gallery.act(&mut cx, GAct::Entry(ENTRIES.iter().position(|e| e.0 == "kit").unwrap()));
        let page = aegis_ui::specimens::PAGES.iter().position(|p| *p == "Layout").expect("a Layout page");
        gallery.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(gallery.arrow_focus() && gallery.state(&cx)[STATE_KEY].is_null(), "not on the Layout page yet");
        gallery.click_id(&mut cx, &format!("kit.page.{page}")).unwrap();
        gallery.draw(&mut cx, &mut g, (-500, -500), true);
        assert!(!gallery.arrow_focus(), "the arrows nudge on the Layout page");
        let first = gallery.kit.layout.markers()[0].clone();
        let handle = format!("layout.marker.{}", first.id);
        let (r, _) = gallery.widget_rect(&handle).expect("the marker's handle");
        let at = (r.x + r.w / 2, r.y + r.h / 2);
        gallery.pointer(&mut cx, &handle, PointerPhase::Down, at);
        gallery.pointer(&mut cx, &handle, PointerPhase::Move, (at.0 + 20, at.1));
        gallery.pointer(&mut cx, &handle, PointerPhase::Up, (at.0 + 20, at.1));
        let moved = gallery.kit.layout.markers()[0].pos;
        assert!(moved[0] > first.pos[0] && (moved[1] - first.pos[1]).abs() < 1e-3, "dragged right: {:?} -> {moved:?}", first.pos);
        assert_eq!(gallery.kit.layout.undo_depth(), 1, "the drag is one undo step");
        // Shift held: the coarse nudge; Ctrl+Z: undo
        cx.held.shift = true;
        gallery.key(&mut cx, Key::Right);
        cx.held.shift = false;
        let nudged = gallery.kit.layout.markers()[0].pos;
        assert!((nudged[0] - moved[0] - NUDGE.1).abs() < 1e-3, "{moved:?} -> {nudged:?}");
        cx.held.ctrl = true;
        gallery.key(&mut cx, Key::Char('z'));
        cx.held.ctrl = false;
        assert!((gallery.kit.layout.markers()[0].pos[0] - moved[0]).abs() < 1e-3, "Ctrl+Z took the nudge back");
        // the agent link's commands and state
        let reply = gallery.command(&cx, &json!({"cmd": "layout", "id": first.id, "pos": [0.5, 0.5]})).expect("a layout op");
        assert_eq!(reply["ok"], json!(true), "{reply}");
        assert_eq!(gallery.state(&cx)[STATE_KEY]["markers"][0]["pos"], json!([0.5, 0.5]));
        assert_eq!(gallery.command(&cx, &json!({"cmd": STATE_KEY})).expect("the state")["ok"], json!(true));
        // another page: no layout commands, the arrows steer the focus again
        gallery.click_id(&mut cx, "kit.page.0").unwrap();
        assert!(gallery.command(&cx, &json!({"cmd": STATE_KEY})).is_none() && gallery.arrow_focus());
    }

    /// LOOK1-M1: the real Gallery forwards keys to the Kit widgets' Panes page: the menu column's hotkey, the tab rail's
    /// PageUp / PageDown, the ask's y and Esc (Esc answers there and leaves the gallery elsewhere); another entry takes none.
    #[test]
    fn the_gallery_forwards_keys_to_the_panes_page() {
        use crate::hud::hud_tests::{game_cx, gfx};
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut gallery = Gallery::new();
        let at = |id: &str| ENTRIES.iter().position(|e| e.0 == id).unwrap();
        gallery.act(&mut cx, GAct::Entry(at("kit")));
        let page = aegis_ui::specimens::PAGES.iter().position(|p| *p == "Panes").expect("a Panes page");
        let pane = |name: &str| aegis_ui::specimens::PANES.iter().position(|p| *p == name).unwrap_or_else(|| panic!("a pane {name}"));
        let open = |gallery: &mut Gallery, cx: &mut Cx, g: &mut Gfx, id: String| {
            gallery.draw(cx, g, (-500, -500), true);
            gallery.click_id(cx, &id).unwrap();
            gallery.draw(cx, g, (-500, -500), true);
        };
        let last = |gallery: &Gallery, cx: &Cx| gallery.state(cx)["kit_last"].as_str().unwrap_or("").to_string();
        open(&mut gallery, &mut cx, &mut g, format!("kit.page.{page}"));
        open(&mut gallery, &mut cx, &mut g, format!("kit.pane.{}", pane("Menu column")));
        gallery.key(&mut cx, Key::Char('q'));
        assert_eq!(last(&gallery, &cx), "pressed quit", "the menu column's hotkey");
        open(&mut gallery, &mut cx, &mut g, format!("kit.pane.{}", pane("Tab rail")));
        gallery.key(&mut cx, Key::PageDown);
        assert!(last(&gallery, &cx).starts_with("rail tab "), "PageDown: {}", last(&gallery, &cx));
        open(&mut gallery, &mut cx, &mut g, format!("kit.pane.{}", pane("Ask")));
        gallery.key(&mut cx, Key::Char('y'));
        assert_eq!(last(&gallery, &cx), "pressed yes");
        gallery.escape(&mut cx);
        assert_eq!(last(&gallery, &cx), "pressed no");
        assert!(cx.nav.is_empty(), "Esc answered the ask and stayed");
        open(&mut gallery, &mut cx, &mut g, format!("kit.pane.{}", pane("Backdrop")));
        gallery.escape(&mut cx);
        assert!(matches!(cx.nav.as_slice(), [Nav::Pop]), "no key to take: Esc leaves");
        cx.nav.clear();
        gallery.act(&mut cx, GAct::Entry(at("pip_gauge")));
        let before = last(&gallery, &cx);
        gallery.key(&mut cx, Key::Char('q'));
        assert_eq!(last(&gallery, &cx), before, "another entry takes no key");
    }

    /// Every battle panel entry names a panel the battle screen records, and every recorded panel has an entry.
    #[test]
    fn the_battle_panel_entries_are_the_battle_screens_panels() {
        let mut entries: Vec<&str> = ENTRIES.iter().filter(|e| e.2 == "Battle panels" && e.0 != "timeline").map(|e| e.0).collect();
        let mut panels = crate::battle::PANELS.to_vec();
        entries.sort();
        panels.sort();
        assert_eq!(entries, panels);
        let mut ids: Vec<&str> = ENTRIES.iter().map(|e| e.0).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), ENTRIES.len(), "entry ids are unique");
        assert!(NEEDS_GAME.iter().all(|n| ENTRIES.iter().any(|e| e.0 == *n)), "every game-needing id is an entry");
    }

    /// ENCOUNTER2-C2: the Battle talk entry previews the Test Arena's battle with a talk of the sample game's world open
    /// over it (the box and its backdrop drawn, the agent link's `talk` set) whenever that world has a dialogue tree.
    #[test]
    fn the_battle_talk_entry_shows_a_talk_over_the_battle() {
        use crate::hud::hud_tests::{game_cx, gfx};
        let (mut cx, mut g) = (game_cx(), gfx());
        cx.encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let world = cx.gs().world_id.clone();
        let trees = cx.data.own_entries(&world, "dialogues").len();
        let mut gallery = Gallery::new();
        let i = ENTRIES.iter().position(|e| e.0 == "battle_talk").expect("a battle talk entry");
        gallery.act(&mut cx, GAct::Entry(i));
        gallery.draw(&mut cx, &mut g, (-500, -500), true);
        let sh = gallery.shown.as_ref().expect("the preview is built");
        assert_eq!(sh.screen.id(), "test_arena");
        let st = sh.screen.state(&cx);
        if trees == 0 {
            assert!(st.get("talk").is_none());
            return;
        }
        assert!(st["talk"]["dialogue"].as_str().is_some_and(|d| !d.is_empty()), "a talk is open: {}", st["message"]);
        let ws = sh.screen.widgets();
        assert!(["dialogue", "talk.backdrop"].iter().all(|id| ws.iter().any(|w| w["id"] == *id)), "the box over the battle");
    }

    /// INPUT1-AP: the Pip gauge entry (Kit, no game needed) draws every sample state of the engine's gauge with its
    /// reading and hover card, Open does nothing, and its texts fit the pane at the four text sizes.
    #[test]
    fn the_pip_gauge_entry_shows_every_sample_state() {
        use crate::hud::hud_tests::{frame, game_cx, gfx};
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut gallery = Gallery::new();
        let i = ENTRIES.iter().position(|e| e.0 == "pip_gauge").expect("a pip gauge entry");
        assert_eq!(ENTRIES[i].2, "Kit");
        gallery.act(&mut cx, GAct::Entry(i));
        let samples = aegis_ui::pips::samples();
        for &(_, scale) in aegis_ui::screen::TEXT_SIZES.iter() {
            let recs: Vec<aegis_ui::painter::TextRec> = frame(&mut gallery, &mut cx, &mut g, scale as f32).into_iter().filter(|r| r.x >= 110.0).collect();
            let ws = gallery.widgets();
            for (k, (_, gauge)) in samples.iter().enumerate() {
                let w = ws.iter().find(|w| w["id"] == format!("gallery.pip.{k}")).unwrap_or_else(|| panic!("sample {k} at text size {scale}"));
                assert_eq!(w["text"], json!(gauge.reading()));
            }
            let v = aegis_ui::text_audit::issues(&recs, W as f32, aegis_ui::ui::H as f32, &[]);
            assert_eq!((v["counts"]["edge"].clone(), v["counts"]["overlap"].clone()), (json!(0), json!(0)), "text size {scale}: {v}");
        }
        gallery.act(&mut cx, GAct::Open);
        assert!(cx.nav.is_empty(), "nothing to open");
        assert_eq!(gallery.state(&cx)["entry"], json!("pip_gauge"));
    }

    /// STORYUI1-E: the vignette entry previews the engine's sample reel (it plays on as the gallery ticks) and Open
    /// plays it full screen.
    #[test]
    fn the_vignette_entry_plays_the_sample_reel() {
        use crate::hud::hud_tests::{game_cx, gfx};
        let (mut cx, mut g) = (game_cx(), gfx());
        let mut gallery = Gallery::new();
        let i = ENTRIES.iter().position(|e| e.0 == "vignette").expect("a vignette entry");
        assert!(!NEEDS_GAME.contains(&"vignette"), "the player needs no game");
        gallery.act(&mut cx, GAct::Entry(i));
        gallery.draw(&mut cx, &mut g, (-500, -500), true);
        let state = |gallery: &Gallery, cx: &Cx| gallery.shown.as_ref().map(|sh| sh.screen.state(cx)).expect("the preview is built");
        let st = state(&gallery, &cx);
        assert_eq!((st["reel"].clone(), st["cards"].clone()), (json!("sample"), json!(vignette::sample_reel().cards.len())));
        gallery.tick(&mut cx);
        assert!(gallery.shown.is_some(), "the preview survives a tick");
        gallery.act(&mut cx, GAct::Open);
        assert!(matches!(cx.nav.last(), Some(Nav::Push(s)) if s.id() == "vignette"), "Open pushes the player");
    }

    /// STORYUI1-U1 TU3 (V15): with no game, the vignette entry draws its reel picker at every text size, the engine's
    /// sample first; a picked world vignette plays in the preview and Open plays it full screen.
    #[test]
    fn the_vignette_pane_plays_a_worlds_vignette_without_a_game() {
        use crate::hud::hud_tests::{frame, gfx};
        use crate::story::tests::{data, story_cx};
        let (mut cx, mut g) = (story_cx(data()), gfx());
        let mut gallery = Gallery::new();
        let i = ENTRIES.iter().position(|e| e.0 == "vignette").expect("a vignette entry");
        gallery.act(&mut cx, GAct::Entry(i));
        for &(_, scale) in aegis_ui::screen::TEXT_SIZES.iter() {
            frame(&mut gallery, &mut cx, &mut g, scale as f32);
            assert!(gallery.widgets().iter().any(|w| w["id"] == "gallery.vignette.reel" && w["enabled"] == json!(true)), "the picker at text size {scale}");
        }
        let shown = |gallery: &Gallery, cx: &Cx| gallery.shown.as_ref().map(|sh| sh.screen.state(cx)).expect("the preview is built");
        assert_eq!(shown(&gallery, &cx)["reel"], json!("sample"));
        gallery.act(&mut cx, GAct::Reel(1));
        gallery.draw(&mut cx, &mut g, (-500, -500), true);
        let st = shown(&gallery, &cx);
        assert_eq!((st["reel"].clone(), st["cards"].clone()), (json!("tale:v_intro"), json!(2)));
        assert!(!cx.has_game(), "no game needed");
        gallery.act(&mut cx, GAct::Open);
        assert!(matches!(cx.nav.last(), Some(Nav::Push(s)) if s.id() == "vignette" && s.state(&cx)["reel"] == json!("tale:v_intro")), "Open plays the picked reel");
    }

    /// SERVICES1 (SILOS1 T2): the Dialogue and Shop entries show the services on their plain backdrop, no region map:
    /// a conversation with the first person here who has replies (Shop: the first whose replies offer a shop, opened
    /// by the scripted click on that reply); the preview holds no game busy. The person is found by rule.
    #[test]
    fn the_dialogue_and_shop_entries_open_on_the_plain_backdrop() {
        use crate::hud::hud_tests::{game_cx, gfx};
        let (mut cx, mut g) = (game_cx(), gfx());
        // the entry enters the start location first; so does the expectation
        let d = cx.data.clone();
        let _ = emberwake_world::explore::resume(&d, cx.gs_mut());
        let gs = cx.gs().clone();
        let loc = cx.data.resolve(&gs.world_id, "locations", &gs.location_id).clone();
        let people: Vec<String> = emberwake_world::explore::markers_of(&loc).iter().filter(|m| json::s(m, "kind") == "npc").map(|m| json::s(m, "character")).collect();
        // who offers a shop reply, opened the way the entry opens them (the state put back after)
        let mut has_shop = false;
        for who in &people {
            let mut s = crate::talk::ServiceScreen::default();
            s.talk(&mut cx, who);
            has_shop |= Gallery::services(&s.state(&cx)).iter().any(|c| c == "shop");
        }
        *cx.gs_mut() = gs;
        cx.set_busy(false);
        assert!(!people.is_empty() && has_shop, "the sample start location has someone to talk to and a shop keeper (else this test checks nothing)");
        for id in ["dialogue", "shop"] {
            let mut gallery = Gallery::new();
            let i = ENTRIES.iter().position(|e| e.0 == id).expect("the entry");
            gallery.act(&mut cx, GAct::Entry(i));
            for _ in 0..3 {
                gallery.draw(&mut cx, &mut g, (-500, -500), true);
            }
            let sh = gallery.shown.as_ref().expect("the preview is built");
            assert_eq!(sh.screen.id(), "services", "{id}: the plain-backdrop host, not a region map");
            assert!(!cx.busy(), "{id}: a preview holds no game busy");
            let st = sh.screen.state(&cx);
            let ws = sh.screen.widgets();
            if id == "dialogue" {
                assert!(st["dialogue"].is_object() && ws.iter().any(|w| w["id"] == "dialogue"), "{id}: the talk box: {st}");
            }
            if id == "shop" {
                assert!(st["shop"].is_object() && ws.iter().any(|w| w["id"] == "shop"), "{id}: the trade opened by the shop reply: {st}");
                assert_eq!(sh.steps.len(), 0, "the scripted click is done");
            }
        }
    }

    /// STORYUI1-U2: with no game, the Mission board entry previews the board pane on its fixture board (every section,
    /// a row per quest) and Open shows it full screen.
    #[test]
    fn the_board_entry_shows_the_sample_board_without_a_game() {
        use crate::hud::hud_tests::gfx;
        use crate::story::tests::{data, story_cx};
        let (mut cx, mut g) = (story_cx(data()), gfx());
        let mut gallery = Gallery::new();
        let i = ENTRIES.iter().position(|e| e.0 == "board").expect("a board entry");
        assert!(!NEEDS_GAME.contains(&"board"), "the sample needs no game");
        gallery.act(&mut cx, GAct::Entry(i));
        gallery.draw(&mut cx, &mut g, (-500, -500), true);
        let sh = gallery.shown.as_ref().expect("the preview is built");
        assert_eq!(sh.screen.id(), "board_sample");
        let (view, _) = crate::board::sample();
        let ids: Vec<String> = json::aget(&view, "sections").iter().map(|s| json::s(json::dict(s), "id")).collect();
        assert_eq!(ids, emberwake_world::board::SECTIONS.iter().map(|s| s.0.to_string()).collect::<Vec<_>>(), "every section in order");
        let ws = sh.screen.widgets();
        for r in crate::board::rows(&view) {
            assert!(ws.iter().any(|w| w["id"] == format!("board.row.{}", json::s(&r, "quest"))), "a row for {}", json::s(&r, "quest"));
        }
        assert!(!cx.has_game());
        gallery.act(&mut cx, GAct::Open);
        assert!(matches!(cx.nav.last(), Some(Nav::Push(s)) if s.id() == "board_sample"), "Open shows the pane");
    }

    /// PORTRAITFX1-G G6: the Portraits entry lists every portrait of the game and kit layers (counted here from the
    /// folders) and one control per cue row; an event row's button changes the portrait's pixels, a state row's box
    /// toggles its hold; every control of the Kit widgets' portrait page is there, at every text size.
    #[test]
    fn the_portraits_entry_lists_every_portrait_and_its_cue_rows() {
        use crate::hud::hud_tests::{frame, game_cx, gfx};
        use crate::portraits::tests::row;
        let (mut cx, mut g) = (game_cx(), gfx());
        cx.art = aegis_ui::art::Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
        // the portraits on disk: each folder's PNGs and each character with only expression frames, once across layers
        let mut on_disk = std::collections::BTreeSet::new();
        for root in &cx.art.layers().roots {
            for folder in PORTRAIT_FOLDERS {
                let dir = root.join("portraits").join(folder);
                for e in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                    let name = e.file_name().to_string_lossy().to_string();
                    if let Some(id) = name.strip_suffix(".png") {
                        on_disk.insert(format!("{folder}/{id}"));
                    }
                }
                for e in std::fs::read_dir(dir.join("faces")).into_iter().flatten().flatten().filter(|_| folder != "common") {
                    on_disk.insert(format!("{folder}/{}", e.file_name().to_string_lossy()));
                }
            }
        }
        assert!(!on_disk.is_empty(), "the shipped layers hold portraits");
        // the cue table of PORTRAITFX1_TASKS 2.3 (the asset task's rows), so the layout is audited at its full size
        let rows = vec![
            row(json!({"id": "hit_flash", "event": "hit", "effect": "flash", "intensity": 0.6, "ms": 140, "color": "#d9cbb4"})),
            row(json!({"id": "hit_jolt", "event": "hit", "effect": "jolt", "preset": "hit"})),
            row(json!({"id": "crit_flash", "event": "crit", "effect": "flash", "intensity": 0.9, "ms": 180, "color": "#e11d48"})),
            row(json!({"id": "crit_jolt", "event": "crit", "effect": "jolt", "preset": "crit"})),
            row(json!({"id": "guard_flash", "event": "guard_hit", "effect": "flash", "intensity": 0.4, "ms": 120, "color": "#fbbf24"})),
            row(json!({"id": "guard_jolt", "event": "guard_hit", "effect": "jolt", "intensity": 0.5, "preset": "hit"})),
            row(json!({"id": "heal_glow", "event": "heal", "effect": "flash", "intensity": 0.3, "ms": 420, "color": "#fbbf24"})),
            row(json!({"id": "break_flash", "event": "break", "effect": "flash", "intensity": 0.9, "ms": 120, "color": "#d9cbb4"})),
            row(json!({"id": "break_jolt", "event": "break", "effect": "jolt", "preset": "crit"})),
            row(json!({"id": "wounded", "trigger": "condition", "condition": "wounded", "effect": "desaturate", "intensity": 0.35})),
            row(json!({"id": "hp_half", "trigger": "hp", "below": 50, "effect": "desaturate", "intensity": 0.2})),
            row(json!({"id": "hp_low", "trigger": "hp", "below": 25, "effect": "desaturate", "intensity": 0.5})),
            row(json!({"id": "downed", "trigger": "down", "effect": "desaturate"})),
            row(json!({"id": "downed_still", "trigger": "down", "effect": "still"})),
            row(json!({"id": "shaken", "trigger": "derived", "stat": "composure", "drop_pct": 25, "effect": "tint", "intensity": 0.5, "color": "#06b6d4"})),
            row(json!({"id": "stealthed", "trigger": "status", "status": "stealth", "effect": "fade", "intensity": 0.6})),
        ];
        let mut cue_ids: Vec<String> = rows.iter().map(|r| format!("gallery.portraits.cue.{}", r.id)).collect();
        // PORTRAITFX1-EX: the three expression-effect rows, one box each
        let fx_rows: Vec<emberwake_data::PortraitExpression> = [
            json!({"id": "hurt", "tint": "#e11d48", "tint_strength": 0.8, "tint_ms": 240, "shake_px": 2, "shake_ms": 220}),
            json!({"id": "angry", "tint": "#e11d48", "tint_strength": 0.35, "mark": "vein", "mark_color": "#e11d48", "mark_pulse_ms": 900}),
            json!({"id": "afraid", "tint": "#06b6d4", "tint_strength": 0.4, "mark": "sweat", "mark_x": 0.12, "mark_y": 0.22}),
        ]
        .iter()
        .map(crate::portraits::tests::ex_row_of)
        .collect();
        cue_ids.extend(fx_rows.iter().map(|r| format!("gallery.portraits.fx.{}", r.id)));
        std::sync::Arc::get_mut(&mut cx.app.data).expect("the test owns the data").edit_cat(|c| {
            c.portrait_cues = rows;
            c.portrait_expressions = fx_rows;
        });
        let mut gallery = Gallery::new();
        let i = ENTRIES.iter().position(|e| e.0 == "portraits").expect("a portraits entry");
        gallery.act(&mut cx, GAct::Entry(i));
        for &(_, scale) in aegis_ui::screen::TEXT_SIZES.iter() {
            // TEXTFIT1 over the pane (the gallery's own entry list is the gallery's)
            let recs: Vec<aegis_ui::painter::TextRec> = frame(&mut gallery, &mut cx, &mut g, scale as f32).into_iter().filter(|r| r.x >= 110.0).collect();
            let boxes: Vec<(String, f32, f32, f32, f32)> = gallery
                .widgets()
                .iter()
                .filter(|w| w["id"].as_str().is_some_and(|i| i.starts_with("gallery.portraits.")))
                .map(|w| {
                    let (x, y, ww, hh) = crate::hud::hud_tests::rect(w);
                    (w["id"].as_str().unwrap_or("").to_string(), x as f32, y as f32, ww as f32, hh as f32)
                })
                .collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, aegis_ui::ui::H as f32, &boxes);
            assert_eq!(v["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "text size {scale}: {v}");
            let ids: Vec<String> = gallery.widgets().iter().map(|w| w["id"].as_str().unwrap_or("").to_string()).collect();
            let listed: std::collections::BTreeSet<String> = ids.iter().filter_map(|id| id.strip_prefix("gallery.portraits.pick.")).filter(|r| r.split('.').count() == 2).map(|r| r.replacen('.', "/", 1)).collect();
            assert_eq!(listed, on_disk, "every portrait listed once at text size {scale}");
            let controls = PORTRAIT_OVERLAYS.iter().chain(PORTRAIT_STATES.iter()).chain(PORTRAIT_REACTIONS.iter()).map(|c| format!("gallery.portraits.{c}"));
            for want in controls.chain(["size", "reduce", "anchors", "face"].iter().map(|c| format!("gallery.portraits.{c}"))).chain(cue_ids.iter().cloned()) {
                assert!(ids.contains(&want), "{want} at text size {scale}");
            }
        }
        g.fonts.scale = 1.0;
        let key = gallery.portraits.key();
        let composed = |gallery: &Gallery| gallery.portraits.faces.get(&key).and_then(|f| f.composed()).cloned().expect("the picked portrait is drawn");
        gallery.draw(&mut cx, &mut g, (-500, -500), true);
        let before = composed(&gallery);
        gallery.click_id(&mut cx, "gallery.portraits.cue.hit_flash").expect("the event row's button");
        gallery.draw(&mut cx, &mut g, (-500, -500), true);
        assert_ne!(composed(&gallery).px, before.px, "the flash changes the portrait's pixels");
        assert!(gallery.portraits.faces.get(&key).unwrap().level("flash", 0).is_some());
        gallery.click_id(&mut cx, "gallery.portraits.cue.wounded").expect("the state row's box");
        gallery.draw(&mut cx, &mut g, (-500, -500), true);
        let boxed = |gallery: &Gallery| gallery.widgets().into_iter().find(|w| w["id"] == "gallery.portraits.cue.wounded").map(|w| w["active"] == json!(true));
        assert_eq!(boxed(&gallery), Some(true), "the state row is held");
        assert!(gallery.portraits.pins(&cx.data.cat.portrait_cues).iter().any(|c| c.key == "cue.wounded" && c.effect == "desaturate"));
        gallery.click_id(&mut cx, "gallery.portraits.cue.wounded").expect("again");
        gallery.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!(boxed(&gallery), Some(false), "and released");
        assert_eq!(gallery.state(&cx)["portrait_pick"], json!(key));
        // PORTRAITFX1-EX: an expression-effect box holds its expression on the portrait; a second click releases it
        gallery.click_id(&mut cx, "gallery.portraits.fx.angry").expect("the angry box");
        gallery.draw(&mut cx, &mut g, (-500, -500), true);
        let now = gallery.portraits.faces.now();
        assert_eq!(gallery.portraits.faces.get(&key).unwrap().expression(now), "angry");
        assert!(gallery.portraits.faces.get(&key).unwrap().expression_fx(now).is_some(), "its row reaches the portrait");
        gallery.click_id(&mut cx, "gallery.portraits.fx.angry").expect("again");
        gallery.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!(gallery.portraits.faces.get(&key).unwrap().expression(gallery.portraits.faces.now()), "");
    }

    /// INPUT1 TV7: the Kit group's Timeline pane draws the engine widget alone on its sample round with no game and no
    /// battle (the widgets are the sample's own: a masked track with its marks, a link, every track's label); a block
    /// or track click selects it and again unselects; its texts read clean at the four text sizes (a cut one is
    /// whole on its hover card).
    #[test]
    fn the_timeline_pane_draws_the_engine_widget_with_no_session() {
        use crate::hud::hud_tests::{frame, gfx};
        use crate::story::tests::{data, story_cx};
        let (mut cx, mut g) = (story_cx(data()), gfx());
        let mut gallery = Gallery::new();
        let i = ENTRIES.iter().position(|e| e.0 == "timeline_pane").expect("a timeline pane entry");
        assert_eq!(ENTRIES[i].2, "Kit");
        gallery.act(&mut cx, GAct::Entry(i));
        let sample = aegis_ui::timeline::sample();
        for &(_, scale) in aegis_ui::screen::TEXT_SIZES.iter() {
            let recs: Vec<aegis_ui::painter::TextRec> = frame(&mut gallery, &mut cx, &mut g, scale as f32).into_iter().filter(|r| r.x >= 110.0).collect();
            let ws = gallery.widgets();
            let ids: Vec<String> = ws.iter().filter_map(|w| w["id"].as_str()).filter(|id| id.starts_with("gallery.pane.")).map(str::to_string).collect();
            for t in sample.sections.iter().flat_map(|s| &s.tracks).take(12) {
                assert!(ids.contains(&format!("gallery.pane.track.{}", t.key)), "{} at {scale}", t.key);
            }
            assert!(ids.contains(&"gallery.pane.track.e2.masked".to_string()) && ids.contains(&"gallery.pane.mark.e2.i0".to_string()) && ids.contains(&"gallery.pane.link.p1.strike.follow".to_string()));
            let boxes: Vec<(String, f32, f32, f32, f32)> = ws
                .iter()
                .filter(|w| w["id"].as_str().is_some_and(|i| i.starts_with("gallery.pane.")))
                .map(|w| {
                    let (x, y, ww, hh) = crate::hud::hud_tests::rect(w);
                    (w["id"].as_str().unwrap_or("").to_string(), x as f32, y as f32, ww as f32, hh as f32)
                })
                .collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, aegis_ui::ui::H as f32, &boxes);
            assert_eq!((v["counts"]["spill"].clone(), v["counts"]["edge"].clone(), v["counts"]["overlap"].clone()), (json!(0), json!(0), json!(0)), "text size {scale}: {v}");
            for c in v["cut"].as_array().unwrap() {
                let full = c["text"].as_str().unwrap();
                let (x, y) = (c["at"][0].as_f64().unwrap() as i64 + 1, c["at"][1].as_f64().unwrap() as i64 + 2);
                // the widget's own texts (the entry's line above it is the gallery's, whole on the entry's card)
                let on = |w: &&Value| {
                    let (rx, ry, rw, rh) = crate::hud::hud_tests::rect(w);
                    w["id"].as_str().is_some_and(|i| i.starts_with("gallery.pane.")) && x >= rx && x < rx + rw && y >= ry && y < ry + rh
                };
                if !ws.iter().any(|w| on(&w)) {
                    continue;
                }
                let held = ws.iter().filter(on).any(|w| w["text"] == json!(full) || w["tip"].to_string().contains(full));
                assert!(held, "text size {scale}: {full:?} cut with no whole text on hover");
            }
        }
        assert_eq!(gallery.pane, (sample.selected_block, sample.selected_track.clone()), "the sample's selection to start");
        gallery.click_id(&mut cx, "gallery.pane.block.5").unwrap();
        gallery.click_id(&mut cx, "gallery.pane.track.e1").unwrap();
        assert_eq!(gallery.pane, (Some(5), Some("e1".to_string())));
        gallery.click_id(&mut cx, "gallery.pane.block.5").unwrap();
        assert_eq!(gallery.pane.0, None, "again: unselected");
        assert!(!cx.has_game(), "no game needed");
    }

    /// TV7: the Battle panels' Battle timeline draws the battle screen's timeline view of its battle (the shared
    /// builder): a track widget for every track `battle_timeline` builds, a masked note for every masked one.
    #[test]
    fn the_battle_timeline_entry_draws_the_shared_builders_tracks() {
        use crate::hud::hud_tests::{game_cx, gfx};
        let (mut cx, mut g) = (game_cx(), gfx());
        cx.encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
        let mut gallery = Gallery::new();
        let i = ENTRIES.iter().position(|e| e.0 == "timeline").expect("the timeline entry");
        gallery.act(&mut cx, GAct::Entry(i));
        gallery.draw(&mut cx, &mut g, (-500, -500), true);
        let d = gallery.demo.as_ref().unwrap();
        let s = d.host.session.as_ref().expect("the demo battle");
        let t = battle_timeline(&cx, s, d.viewed, d.reveal, d.block, d.track.clone());
        assert_eq!(t.sections.iter().map(|x| x.title.as_str()).collect::<Vec<_>>(), ["Party", "Enemies"]);
        let ids: Vec<String> = gallery.widgets().iter().filter_map(|w| w["id"].as_str().map(str::to_string)).collect();
        for tr in t.sections.iter().flat_map(|x| &x.tracks) {
            assert!(ids.contains(&format!("gallery.timeline.track.{}", tr.key)), "{}", tr.key);
            assert_eq!(ids.contains(&format!("gallery.timeline.track.{}.masked", tr.key)), tr.masked.is_some(), "{}", tr.key);
        }
    }
}
