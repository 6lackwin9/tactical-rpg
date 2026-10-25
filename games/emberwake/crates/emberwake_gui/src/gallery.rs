//! UI Gallery (UIGALLERY1, Scene Runner; decision "UIGALLERY1 answers"): the UI one element at a time, each on sample
//! data and named, so each can be judged and refined on its own. Three groups:
//! - Kit: the engine's widgets in their states (`aegis_ui::specimens`).
//! - Screens: every screen, drawn shrunk into the pane (`aegis_ui::preview`) on a sample game (a debug game of the
//!   first playable world that saving refuses); Open shows the screen itself to try it.
//! - Battle panels: each panel of the battle screen enlarged on its own (the battle records its panels' rects while it
//!   draws), and the battle timeline (TIMELINE1: every unit's track over the round's blocks, fed by a live Test Arena
//!   battle; past rounds from the round archive, a block's state from its snapshot).

use crate::battle::{action_rows, kind_color, BattleScreen};
use crate::intent_text;
use crate::screens::{Cx, Game, GameCx, Nav, Screen};
use aegis_core::json;
use aegis_ui::canvas::{rgb, Rect, Rgba};
use aegis_ui::preview;
use aegis_ui::specimens::{KitAct, Specimens};
use aegis_ui::timeline::{Section, Segment, Timeline, Track};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, BORDER, DIM, GOLD, INFO, SMALL, TEXT, TITLE, W};
use aegis_ui::widget_plumbing;
use emberwake_battle::resolver::ScheduledAction;
use emberwake_battle::session::{Phase, Session, UnitSnap};
use emberwake_host::{Battle, Host};
use serde_json::{json, Value};

/// (id, name, group, what it shows). Battle panel ids are the battle screen's own panel names.
const ENTRIES: [(&str, &str, &str, &str); 26] = [
    ("kit", "Kit widgets", "Kit", "Every kit widget in its states; use them, the foot of the page says what happened."),
    ("main_menu", "Main menu", "Screens", "The player's front door."),
    ("new_game", "New game", "Screens", "World cards and the start."),
    ("settings", "Settings", "Screens", "Sound, display and gameplay tabs."),
    ("save", "Save slots", "Screens", "Saving from the game menu."),
    ("load", "Load slots", "Screens", "Loading from the main menu or the game menu."),
    ("region", "Region map", "Screens", "Walking a location: the map, the Here list, the log."),
    ("dialogue", "Dialogue", "Screens", "A conversation over the region map (the first person met)."),
    ("shop", "Shop", "Screens", "A shop counter over the region map (the first trader met)."),
    ("hub", "Game menu", "Screens", "The in-game menu over the map."),
    ("journal", "Journal", "Screens", "Quests and notes."),
    ("stats", "Character: Stats", "Screens", "The character popup's Stats tab."),
    ("level_up", "Character: Level up", "Screens", "The character popup's Level up tab."),
    ("skills", "Character: Skills", "Screens", "The character popup's Skills tab."),
    ("gear", "Character: Gear", "Screens", "The character popup's Gear tab."),
    ("world_map", "World map", "Screens", "Travel between locations."),
    ("results", "Battle results", "Screens", "After a fight: outcome, XP, rewards."),
    ("battle", "Battle (whole)", "Screens", "The whole battle screen, planning (the Test Arena's encounter)."),
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
const NEEDS_GAME: [&str; 12] = ["region", "dialogue", "shop", "hub", "journal", "stats", "level_up", "skills", "gear", "world_map", "results", "save"];
/// Where a preview's mouse is (off every widget).
const OFF: (i32, i32) = (-10_000, -10_000);
const ALLY: Rgba = [110, 170, 255, 255];
const FOE: Rgba = [240, 100, 92, 255];
/// The strip under the timeline: an action's details or a block's snapshot.
const DETAIL_H: i32 = 64;

#[derive(Debug, Clone)]
pub enum GAct {
    Entry(usize),
    Kit(KitAct),
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

pub struct Gallery {
    entry: usize,
    demo: Option<Demo>,
    kit: Specimens,
    shown: Option<Shown>,
    battle: Option<BattleScreen>,
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
    pub fn new() -> Self {
        Self { entry: 0, demo: None, kit: Specimens::default(), shown: None, battle: None, own_game: None, w: Vec::new() }
    }

    /// The sample game for the screen entries: a debug game (nothing saved) of the first playable world.
    fn ensure_game(&mut self, cx: &mut Cx) -> bool {
        if cx.game.is_some() {
            return true;
        }
        let worlds = emberwake_world::new_game::playable_worlds(&cx.data);
        let Some(w) = worlds.first() else { return false };
        let Ok(gs) = emberwake_world::new_game::create_for_world(&cx.data, w, 1) else { return false };
        let clean = gs.to_value();
        cx.game = Some(Game { gs, slot: -1, clean, busy: false, sample: true });
        self.own_game = Some(cx.debug);
        cx.debug = true;
        true
    }

    fn build(cx: &mut Cx, id: &str) -> Option<(Box<dyn Screen<GameCx>>, Vec<Step>)> {
        let member = cx.game.as_ref().and_then(|g| g.gs.party.members.first()).map(|m| m.id.clone()).unwrap_or_default();
        let screen: Box<dyn Screen<GameCx>> = match id {
            "main_menu" => Box::new(crate::menus::MainMenu::default()),
            "new_game" => Box::new(crate::menus::NewGameScreen::new(cx)),
            "settings" => Box::new(crate::menus::SettingsScreen::new(cx)),
            "save" => Box::new(crate::menus::Slots::new(true)),
            "load" => Box::new(crate::menus::Slots::new(false)),
            "region" => Box::new(crate::region::RegionMap::new(cx)),
            "dialogue" | "shop" => Box::new(Self::talking(cx, id == "shop")),
            "hub" => Box::new(crate::hub::Hub::default()),
            "journal" => Box::new(crate::hub::Journal::default()),
            "stats" | "level_up" | "skills" | "gear" => Box::new(crate::party::Character::new(cx, &member, id)),
            "world_map" => Box::new(crate::worldmap::WorldMap::new()),
            "results" => {
                let level = cx.game.as_ref().and_then(|g| g.gs.party.members.first()).map(|m| m.level(&cx.data)).unwrap_or(1);
                let mut xp = json::Dict::new();
                xp.insert(member.clone(), json!({"xp": 24, "old_level": level, "new_level": level, "levels_gained": 0, "unspent": 0}));
                let res = json::obj([("ok", true.into()), ("xp", Value::Object(xp)), ("advanced", json!([])), ("events", json!([])), ("gold", json!(30))]);
                Box::new(crate::hub::Results::new(&res, "won", &json::Dict::new()))
            }
            _ => return None,
        };
        let steps = if id == "shop" { vec![Step::Service("shop")] } else { Vec::new() };
        Some((screen, steps))
    }

    /// A region map in conversation with the first person here (`shop`: the first who offers a shop).
    fn talking(cx: &mut Cx, shop: bool) -> crate::region::RegionMap {
        // a region map first: a new sample game enters its start location there
        let first = crate::region::RegionMap::new(cx);
        let people: Vec<String> = match cx.game.as_ref() {
            Some(g) => {
                let loc = cx.data.resolve(&g.gs.world_id, "locations", &g.gs.location_id);
                emberwake_world::explore::markers_of(loc).iter().filter(|m| json::s(m, "kind") == "npc").map(|m| json::s(m, "character")).collect()
            }
            None => Vec::new(),
        };
        let mut found = None;
        for who in &people {
            let mut m = crate::region::RegionMap::new(cx);
            m.talk(cx, who);
            let offers = Self::services(&m.state(cx));
            if !offers.is_empty() && (!shop || offers.iter().any(|c| c == "shop")) {
                found = Some(m);
                break;
            }
        }
        // a preview's conversation never ends, so it must not hold the game busy
        cx.set_busy(false);
        found.unwrap_or(first)
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
            preview::region_in(g, target, src, |g| b.draw(cx, g, OFF, false));
            return;
        }
        if NEEDS_GAME.contains(&id) && !self.ensure_game(cx) {
            return;
        }
        if self.shown.as_ref().is_none_or(|sh| sh.id != id) {
            self.shown = Self::build(cx, id).map(|(screen, steps)| Shown { id, screen, steps, tries: 0 });
        }
        if let Some(sh) = self.shown.as_mut() {
            preview::screen_in(g, r, |g| sh.screen.draw(cx, g, OFF, false));
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
                self.entry = i;
                self.shown = None;
            }
            GAct::Kit(k) => self.kit.act(k),
            GAct::Open => {
                let id = ENTRIES[self.entry].0;
                if id == "battle" || ENTRIES[self.entry].2 == "Battle panels" {
                    let b = BattleScreen::arena(cx, None, &cx.encounter.clone());
                    cx.nav.push(Nav::Push(Box::new(b)));
                } else if id != "kit" && (!NEEDS_GAME.contains(&id) || self.ensure_game(cx)) {
                    // a fresh one, so the preview keeps its own state
                    if let Some((screen, _)) = Self::build(cx, id) {
                        cx.nav.push(Nav::Push(screen));
                    }
                }
            }
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
    Segment { key, start, end, label: target.map(|t| format!("{name} > {t}")).unwrap_or_else(|| name.to_string()), fill, dashed: kind == "MOVE", tip: t, ..Segment::default() }
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
            Track { key: format!("u{u}"), label: w.units[u].name.clone(), colour: ALLY, segments, ..Track::default() }
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
            Track { key: format!("u{e}"), label: w.units[e].name.clone(), colour: FOE, segments: rows(s, &archived, e), masked, ..Track::default() }
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
                ui.p.text(list.x as f32 + 2.0, y as f32 + 2.0, group, SMALL, INFO);
                y += head_h;
            }
            ui.row(&format!("gallery.entry.{id}"), Rect::new(list.x, y, list.w - 4, row_h), name, if i == self.entry { GOLD } else { TEXT }, Some(GAct::Entry(i)), i == self.entry, tip(name, about));
            y += row_h + 1;
        }
        ui.scroll_end();
        let pane = Rect::new(110, 20, W - 114, aegis_ui::ui::H - 24);
        ui.p.panel(pane, rgb(18, 24, 38), BORDER, 2.0);
        let (id, name, _, about) = ENTRIES[self.entry];
        if id == "kit" {
            self.kit.draw(&mut ui, Rect::new(pane.x + 2, pane.y + 2, pane.w - 4, pane.h - 4), GAct::Kit);
        } else if id != "timeline" {
            // A screen or a battle panel: its name, what it is, Open, and the preview under them.
            ui.p.text(pane.x as f32 + 6.0, pane.y as f32 + 4.0, name, TITLE, GOLD);
            let ow = (ui.p.text_w("Open", SMALL) + 12.0).ceil() as i32;
            let nw = ui.p.text_w(name, TITLE).ceil() as i32;
            ui.p.text_fit((pane.x + 14 + nw) as f32, pane.y as f32 + 5.0, about, SMALL, (pane.w - nw - ow - 28) as f32, DIM);
            ui.button_full("gallery.open", Rect::new(pane.x + pane.w - ow - 4, pane.y + 3, ow, 12), "Open", GAct::Open, true, false, SMALL, tip("Open", "Shows the screen itself to try it."));
            let top = pane.y + 4 + ui.p.line_h(TITLE).ceil() as i32 + 3;
            let shown = Rect::new(pane.x + 4, top, pane.w - 8, pane.y + pane.h - 4 - top);
            ui.p.fill(shown, rgb(6, 8, 14));
            self.w = std::mem::take(&mut ui.widgets);
            drop(ui);
            self.preview(cx, g, id, shown);
            return;
        }
        if id == "timeline" {
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
            "entry": ENTRIES[self.entry].0, "shown": self.shown.as_ref().map(|sh| sh.screen.id()), "steps_left": self.shown.as_ref().map(|sh| sh.steps.len()), "kit_last": self.kit.last, "viewed": d.map(|d| d.viewed), "round": s.map(|s| s.current_round), "block": s.map(|s| s.current_block),
            "selected_block": d.and_then(|d| d.block), "planning": s.map(|s| s.phase == Phase::Planning && !s.plan_confirmed), "rounds_archived": s.map(|s| s.history.len()),
        })
    }

    /// The sample game goes with the gallery, only while it is still the one in play (a game started or loaded from an
    /// opened screen replaces it before the gallery is left).
    fn leave(&mut self, cx: &mut Cx) {
        if let Some(debug) = self.own_game.take() {
            if cx.game.as_ref().is_some_and(|g| g.sample) {
                cx.game = None;
                cx.debug = debug;
                cx.log.clear();
            }
        }
    }

    /// Back from an opened screen: the previews are built again (a setting may have changed).
    fn resume(&mut self, _cx: &mut Cx) {
        self.shown = None;
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
}
