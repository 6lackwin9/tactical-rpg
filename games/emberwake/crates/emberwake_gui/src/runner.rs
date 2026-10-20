//! Scene Runner (scripts/ui/runner; dev only): the launcher of every scene, opened from the main menu's Tools. The
//! combat arena takes any catalog or world encounter; Exploration (RUNNER2) plays a world from a picked location or its
//! world map without saving, a fight skipped (settled as a win at no cost) or fought (DebugFight). Reload re-reads the
//! data folder from disk.

use aegis_ui::canvas::{rgb, rgba, Rect};
use crate::screens::{enter_game, show_errors, Cx, GameCx, Nav, Screen};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, BORDER, DIM, GOLD, GOOD, H, INFO, SMALL, TEXT, W};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s, s_or, Dict};
use emberwake_world::{dialogue, explore, new_game, travel};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum SAct {
    MainMenu,
    Arena,
    PickEncounter(usize),
    ContentForge,
    WorldForge,
    Dojo,
    Gallery,
    Exploration,
    PickWorld(usize),
    PickLocation(usize),
    Reload,
    Playtests,
    Exit,
}

pub struct SceneRunner {
    encounter: usize,
    world: usize,
    /// Exploration's start: an index into `starts`.
    location: usize,
    w: Vec<Widget<SAct>>,
}

impl SceneRunner {
    pub fn new() -> Self {
        Self { encounter: 0, world: 0, location: 0, w: Vec::new() }
    }

    /// The picked world of Exploration.
    fn world_id(&self, cx: &Cx) -> Option<String> {
        let worlds = new_game::playable_worlds(&cx.data);
        worlds.get(self.world % worlds.len().max(1)).cloned()
    }

    /// Exploration's starts in `world`: (label, location qid) for the start location first, then every other location
    /// with a map, then the world map (None, at the start location).
    fn starts(d: &emberwake_world::Data, world: &str) -> Vec<(String, Option<String>)> {
        let start = explore::start_location(d, world);
        let mut out: Vec<(String, Option<String>)> = Vec::new();
        let mut others = Vec::new();
        for e in d.merged_entries(world, "locations") {
            let q = s(e, "_qid");
            if s(e, "map").is_empty() {
                continue;
            }
            let label = s_or(e, "name", &s(e, "id"));
            if q == start {
                out.push((format!("{label} (start)"), Some(q)));
            } else {
                others.push((label, Some(q)));
            }
        }
        out.extend(others);
        out.push(("World map".into(), None));
        out
    }

    /// (label, world or None for the catalog, encounter id).
    fn encounters(cx: &Cx) -> Vec<(String, Option<String>, String)> {
        let d = &cx.data;
        let mut out: Vec<(String, Option<String>, String)> = d.cat.encounters.iter().map(|(id, e)| (format!("[Catalog] {}", json::s_or(e, "name", id)), None, id.clone())).collect();
        for w in d.world_ids() {
            for e in d.own_entries(&w, "encounters") {
                out.push((format!("[{}] {}", cx.world_name(&w), s_or(e, "name", &s(e, "id"))), Some(w.clone()), s(e, "id")));
            }
        }
        out
    }

    /// Exploration's game and the arrival's events: a new game of `world` entered at `start` (a location qid), or a
    /// plain new game when None (the region map enters the start location, the world map opens over it).
    fn start_game(d: &emberwake_world::Data, world: &str, start: Option<&str>, seed: i64) -> Result<(emberwake_world::GameState, Vec<Value>), Vec<String>> {
        let mut gs = new_game::create_for_world(d, world, seed)?;
        let mut events = Vec::new();
        if let Some(q) = start.filter(|q| *q != gs.location_id) {
            let res = explore::enter(d, &mut gs, q, explore::SPAWN);
            if !json::b(&res, "ok", false) {
                return Err(json::sget(&res, "errors"));
            }
            events = json::aget(&res, "events").to_vec();
        }
        Ok((gs, events))
    }

    /// Exploration: a debug game of the picked world (nothing saved, fights settled by DebugFight), entered at the
    /// picked location, or on the world map at the start location.
    fn exploration(&mut self, cx: &mut Cx) {
        let Some(w) = self.world_id(cx) else { return };
        let starts = Self::starts(&cx.data, &w);
        let pick = starts.get(self.location.min(starts.len().saturating_sub(1))).and_then(|p| p.1.clone());
        match Self::start_game(&cx.data, &w, pick.as_deref(), cx.seed.unwrap_or(-1)) {
            Err(e) => show_errors(cx, e),
            Ok((gs, events)) => {
                cx.debug = true;
                enter_game(cx, gs, -1, Vec::new());
                crate::region::log_events(cx, &events);
                if pick.is_none() {
                    cx.nav.push(Nav::Push(Box::new(crate::worldmap::WorldMap::new())));
                }
            }
        }
    }

    fn act(&mut self, cx: &mut Cx, a: SAct) {
        match a {
            SAct::MainMenu | SAct::Exit => cx.nav.push(Nav::Pop),
            SAct::PickEncounter(i) => self.encounter = i,
            SAct::Arena => {
                let list = Self::encounters(cx);
                if let Some((_, world, id)) = list.get(self.encounter) {
                    let b = crate::battle::BattleScreen::arena(cx, world.clone(), id);
                    cx.nav.push(Nav::Push(Box::new(b)));
                }
            }
            SAct::ContentForge => cx.nav.push(Nav::Push(Box::new(crate::forge::Forge::content()))),
            SAct::WorldForge => {
                let f = crate::forge::Forge::world(cx);
                cx.nav.push(Nav::Push(Box::new(f)));
            }
            SAct::Gallery => cx.nav.push(Nav::Push(Box::new(crate::gallery::Gallery::new()))),
            SAct::Dojo => {
                let dj = crate::dojo::Dojo::new(cx);
                cx.nav.push(Nav::Push(Box::new(dj)));
            }
            SAct::Exploration => self.exploration(cx),
            SAct::PickWorld(i) => {
                self.world = i;
                self.location = 0;
            }
            SAct::PickLocation(i) => self.location = i,
            SAct::Playtests => {
                let home = cx.data_dir.parent().map(std::path::Path::to_path_buf).unwrap_or_default();
                let repo = vec![home.join("playtests/inbox"), home.join("playtests/repro")];
                let launch: aegis_ui::recordings::Launch<GameCx> = |cx, session, take| Some(Box::new(crate::battle::BattleScreen::playback(cx, session, take)));
                let r = aegis_ui::recordings::Recordings::new(cx, launch, |cx| aegis_content::fingerprint::of(&cx.data_dir), repo);
                cx.nav.push(Nav::Push(Box::new(r)));
            }
            SAct::Reload => match emberwake_world::Data::open(&cx.data_dir) {
                Ok((d, problems)) => {
                    cx.data = Arc::new(d);
                    cx.notice.push(format!("Reloaded the data from {}.", cx.data_dir.display()));
                    cx.notice.extend(problems);
                }
                Err(e) => show_errors(cx, vec![format!("Reload failed: {e:#}")]),
            },
        }
    }
}

impl Screen<GameCx> for SceneRunner {
    fn id(&self) -> &'static str {
        "scene_runner"
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<SAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), rgb(12, 16, 26));
        ui.p.text(12.0, 8.0, &format!("{}  ·  SCENE RUNNER", cx.manifest.title.to_uppercase()), 10.0, GOLD);
        ui.p.text(12.0, 27.0, "Launch any scene; Esc returns here.", SMALL, DIM);
        ui.button("runner.exit", Rect::new(W - 40, 6, 34, 13), "Exit", SAct::Exit, true);
        let left = Rect::new(12, 41, 383, 300);
        let right = Rect::new(402, 41, 235, 110);
        ui.p.panel(left, rgba(18, 24, 38, 255), BORDER, 2.0);
        ui.p.panel(right, rgba(18, 24, 38, 255), BORDER, 2.0);
        ui.p.text(left.x as f32 + 6.0, left.y as f32 + 4.0, "SCENES", SMALL, INFO);
        let encounters = Self::encounters(cx);
        self.encounter = self.encounter.min(encounters.len().saturating_sub(1));
        let worlds = new_game::playable_worlds(&cx.data);
        let starts = self.world_id(cx).map(|w| Self::starts(&cx.data, &w)).unwrap_or_default();
        self.location = self.location.min(starts.len().saturating_sub(1));
        let scenes: [(&str, &str, &str, SAct); 7] = [
            ("runner.main_menu", "Main Menu", "The player's front door: continue, new game, load and save slots, settings and the in-game hub.", SAct::MainMenu),
            ("runner.arena", "Combat Arena", "Plan and fight a battle: a catalog encounter or any world's battle (played with that world's characters).", SAct::Arena),
            ("runner.content_forge", "Content Forge", "Read-only asset viewer: skills, items, classes, units and encounters with an engine summary.", SAct::ContentForge),
            ("runner.world_forge", "World Forge", "Read-only world viewer: regions, factions, characters, quests and encounters with their inheritance.", SAct::WorldForge),
            ("runner.dojo", "Gear & Skill Dojo", "Equip heroes and benchmark their damage against a training dummy.", SAct::Dojo),
            ("runner.gallery", "UI Gallery", "UI panes one at a time on sample data (the battle timeline first), to judge and refine them on their own.", SAct::Gallery),
            ("runner.exploration", "Exploration", "Walk a world's regions and world map from any start (travel, dialogue, shops, quests); a fight is skipped as a win at no cost, or fought. Nothing is saved.", SAct::Exploration),
        ];
        // The list scrolls when the scenes outgrow the panel (the wheel over it). TEXTFIT1: each card is as tall as its
        // name, its wrapped description and its picker.
        let view = Rect::new(left.x, left.y + 14, left.w, left.h - 16);
        let desc_w = (left.w - 70) as f32;
        let (name_h, line_h) = (ui.p.line_h(BODY), ui.p.line_h(SMALL));
        let cards: Vec<(Vec<String>, bool, i32)> = scenes
            .iter()
            .map(|s| {
                let lines = ui.p.wrap(s.2, SMALL, desc_w);
                let extra = matches!(s.3, SAct::Arena | SAct::Exploration);
                let h = (3.0 + name_h + 1.0 + lines.len() as f32 * line_h + if extra { 16.0 } else { 0.0 } + 3.0).ceil() as i32;
                (lines, extra, h.max(30))
            })
            .collect();
        let content: i32 = cards.iter().map(|c| c.2 + 4).sum();
        let off = ui.scroll_begin("runner.scenes", view, content);
        let mut y = view.y - off;
        for ((id, name, _, act), (lines, extra, h)) in scenes.into_iter().zip(cards) {
            let r = Rect::new(left.x + 5, y, left.w - 10, h);
            ui.p.panel(r, rgba(26, 34, 54, 255), BORDER, 2.0);
            ui.p.text(r.x as f32 + 5.0, r.y as f32 + 3.0, name, BODY, INFO);
            let top = r.y as f32 + 3.0 + name_h + 1.0;
            for (i, line) in lines.iter().enumerate() {
                ui.p.text(r.x as f32 + 5.0, top + i as f32 * line_h, line, SMALL, TEXT);
            }
            let pick_y = (top + lines.len() as f32 * line_h + 1.0).round() as i32;
            ui.button(id, Rect::new(r.x + r.w - 50, r.y + 6, 45, 15), "Launch", act.clone(), true);
            if let SAct::Arena = act {
                let labels: Vec<String> = encounters.iter().map(|e| e.0.clone()).collect();
                ui.select("runner.arena.encounter", Rect::new(r.x + 5, pick_y, r.w - 60, 14), &labels, self.encounter, SAct::PickEncounter, !encounters.is_empty(), tip("Encounter", "The battle the arena launches."));
            } else if extra {
                let names: Vec<String> = worlds.iter().map(|w| cx.world_name(w)).collect();
                ui.select(&format!("{id}.world"), Rect::new(r.x + 5, pick_y, 140, 14), &names, self.world % worlds.len().max(1), SAct::PickWorld, worlds.len() > 1, tip("World", "The world to explore."));
                let labels: Vec<String> = starts.iter().map(|p| p.0.clone()).collect();
                ui.select(&format!("{id}.location"), Rect::new(r.x + 149, pick_y, r.w - 204, 14), &labels, self.location, SAct::PickLocation, labels.len() > 1, tip("Start", "Where the party starts: a location, or the world map at the start location."));
            }
            y += h + 4;
        }
        ui.scroll_end();
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 4.0, "DEVELOPER", SMALL, INFO);
        ui.button_tip("runner.reload", Rect::new(right.x + 6, right.y + 16, 70, 14), "Reload data", SAct::Reload, true, tip("Reload data", "Re-reads the data folder (catalog, worlds, maps, quests, services, XP curve)."));
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 36.0, "Parity checks (headless):", SMALL, DIM);
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 46.0, "cargo test -p emberwake_world -p emberwake_battle", SMALL, TEXT);
        ui.button_tip("runner.playtests", Rect::new(right.x + 6, right.y + 58, 70, 14), "Playtests", SAct::Playtests, true, tip("Playtests", "Recordings of played battles with their notes (F8): play back, take over at a round, copy to the repo inbox; the recording toggle."));
        ui.p.text(right.x as f32 + 82.0, right.y as f32 + 61.0, if cx.playtest.on { "Recording on" } else { "Recording off" }, SMALL, if cx.playtest.on { GOOD } else { DIM });
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 78.0, &format!("Data: {}", cx.data_dir.display()), SMALL, DIM);
        self.w = ui.widgets;
    }

    fn state(&self, cx: &Cx) -> Value {
        let world = self.world_id(cx);
        let start = world.as_deref().map(|w| Self::starts(&cx.data, w)).and_then(|st| st.get(self.location.min(st.len().saturating_sub(1))).cloned());
        json!({"encounter": Self::encounters(cx).get(self.encounter).map(|e| e.0.clone()), "world": world, "start": start.map(|p| p.1.unwrap_or_else(|| "world_map".into()))})
    }

    widget_plumbing!();
}

// ---- DebugFight: settle a fight without playing it (debug games) ---------------------------------------------

#[derive(Debug, Clone)]
pub enum DFAct {
    Outcome(&'static str),
    Play,
}

pub struct DebugFight {
    fight: Dict,
    runner: String,
    w: Vec<Widget<DFAct>>,
}

impl DebugFight {
    pub fn new(fight: &Dict, runner: &str) -> Self {
        Self { fight: fight.clone(), runner: runner.to_string(), w: Vec::new() }
    }

    fn act(&mut self, cx: &mut Cx, a: DFAct) {
        cx.nav.push(Nav::Pop);
        match a {
            DFAct::Play => match crate::battle::BattleScreen::world(cx, &self.fight, &self.runner) {
                Ok(b) => cx.nav.push(Nav::Push(Box::new(b))),
                Err(e) => show_errors(cx, vec![e]),
            },
            DFAct::Outcome(outcome) => {
                let d = cx.data.clone();
                let gs = cx.gs_mut();
                let res = match self.runner.as_str() {
                    "travel" => travel::after_fight(&d, gs, &self.fight, outcome),
                    "dialogue" => dialogue::after_fight(&d, gs, &self.fight, outcome),
                    _ => explore::after_fight(&d, gs, &self.fight, outcome),
                };
                cx.set_busy(false);
                crate::region::log_events(cx, json::aget(&res, "events"));
                cx.nav.push(Nav::Push(Box::new(crate::hub::Results::new(&res, outcome, &self.fight))));
            }
        }
    }
}

impl Screen<GameCx> for DebugFight {
    fn id(&self) -> &'static str {
        "debug_fight"
    }
    fn overlay(&self) -> bool {
        true
    }
    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<DFAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), rgba(0, 0, 0, 120));
        let r = Rect::new(W / 2 - 130, H / 2 - 50, 260, 100);
        ui.frame(r, "Fight (debug)", false);
        let name = s_or(cx.data.resolve(&cx.gs().world_id, "encounters", &s(&self.fight, "encounter")), "name", "Enemies");
        ui.p.text(r.x as f32 + 10.0, r.y as f32 + 30.0, &format!("{name} ({})", s(&self.fight, "source")), BODY, TEXT);
        ui.p.text_fit(r.x as f32 + 10.0, r.y as f32 + 44.0, "Skip settles it as a win at no cost; Fight it plays the battle.", SMALL, (r.w - 20) as f32, DIM);
        // RUNNER2 (decision "Skipped fights"): Skip settles the fight as a win at no cost; Fled and Lost test those paths.
        let outcomes = [
            ("Skip", "won", "Settles as a win: every enemy's XP, the purse, quest progress and the marker's flags; no HP lost, nothing spent."),
            ("Fled", "fled", "As if the party fled: back to where the fight began."),
            ("Lost", "lost", "As if the party lost: it wakes at the last safe place."),
        ];
        for (i, (label, o, help)) in outcomes.iter().enumerate() {
            ui.button_tip(&format!("debug_fight.{o}"), Rect::new(r.x + 10 + i as i32 * 50, r.y + r.h - 26, 46, 16), label, DFAct::Outcome(o), true, tip(label, help));
        }
        ui.button("debug_fight.play", Rect::new(r.x + r.w - 80, r.y + r.h - 26, 70, 16), "Fight it", DFAct::Play, true);
        self.w = ui.widgets;
    }
    fn back(&mut self, _cx: &mut Cx) {}
    fn state(&self, _cx: &Cx) -> Value {
        json!({"encounter": s(&self.fight, "encounter"), "source": s(&self.fight, "source"), "marker": s(&self.fight, "marker")})
    }
    widget_plumbing!();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> emberwake_world::Data {
        let dir = emberwake_world::Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        emberwake_world::Data::open(&dir).expect("data").0
    }

    /// RUNNER2: the start location first, every other location with a map, the world map last; a picked location is
    /// entered, the world map start leaves the new game as it is (the region map under it enters the start location).
    #[test]
    fn exploration_starts_at_any_mapped_location_or_the_world_map() {
        let d = data();
        let starts = SceneRunner::starts(&d, "ashlands");
        let start = explore::start_location(&d, "ashlands");
        assert_eq!(starts.first().and_then(|p| p.1.clone()), Some(start.clone()));
        assert_eq!(starts.last().map(|p| p.1.clone()), Some(None), "the world map is the last start");
        let mapped = d.merged_entries("ashlands", "locations").iter().filter(|e| !s(e, "map").is_empty()).count();
        assert_eq!(starts.len(), mapped + 1);
        for (label, q) in &starts {
            let (gs, events) = SceneRunner::start_game(&d, "ashlands", q.as_deref(), 1).unwrap_or_else(|e| panic!("{label}: {e:?}"));
            match q {
                Some(q) => {
                    assert_eq!(gs.location_id, *q, "{label}");
                    assert!(gs.flags.contains_key(&format!("{}.visited", q.replace(':', "."))), "{label}: entered");
                    assert!(q == &start || events.iter().any(|e| json::s(json::dict(e), "kind") == "entered"), "{label}: the arrival is logged");
                }
                None => assert_eq!(gs.location_id, "", "{label}: a plain new game"),
            }
        }
    }
}
