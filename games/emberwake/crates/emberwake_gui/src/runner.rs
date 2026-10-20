//! Scene Runner (scripts/ui/runner; dev only): the launcher of every scene, opened from the main menu's Tools. The
//! combat arena takes any catalog or world encounter; the debug region and world maps play a world without saving and
//! settle fights with Won / Fled / Lost buttons (DebugFight). Reload re-reads the data folder from disk.

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
    DebugRegion,
    DebugWorldMap,
    PickWorld(usize),
    Reload,
    Exit,
}

pub struct SceneRunner {
    encounter: usize,
    world: usize,
    w: Vec<Widget<SAct>>,
}

impl SceneRunner {
    pub fn new() -> Self {
        Self { encounter: 0, world: 0, w: Vec::new() }
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

    fn debug_game(&mut self, cx: &mut Cx, world_map: bool) {
        let worlds = new_game::playable_worlds(&cx.data);
        let Some(w) = worlds.get(self.world % worlds.len().max(1)).cloned() else { return };
        match new_game::create_for_world(&cx.data, &w, cx.seed.unwrap_or(-1)) {
            Err(e) => show_errors(cx, e),
            Ok(gs) => {
                cx.debug = true;
                enter_game(cx, gs, -1, Vec::new());
                if world_map {
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
            SAct::DebugRegion => self.debug_game(cx, false),
            SAct::DebugWorldMap => self.debug_game(cx, true),
            SAct::PickWorld(i) => self.world = i,
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
        let scenes: [(&str, &str, &str, SAct); 8] = [
            ("runner.main_menu", "Main Menu", "The player's front door: continue, new game, load and save slots, settings and the in-game hub.", SAct::MainMenu),
            ("runner.arena", "Combat Arena", "Plan and fight a battle: a catalog encounter or any world's battle (played with that world's characters).", SAct::Arena),
            ("runner.content_forge", "Content Forge", "Read-only asset viewer: skills, items, classes, units and encounters with an engine summary.", SAct::ContentForge),
            ("runner.world_forge", "World Forge", "Read-only world viewer: regions, factions, characters, quests and encounters with their inheritance.", SAct::WorldForge),
            ("runner.dojo", "Gear & Skill Dojo", "Equip heroes and benchmark their damage against a training dummy.", SAct::Dojo),
            ("runner.gallery", "UI Gallery", "UI panes one at a time on sample data (the battle timeline first), to judge and refine them on their own.", SAct::Gallery),
            ("runner.debug_region", "Region Map (debug)", "Walk a world's region maps with Won / Fled / Lost buttons instead of real battles (nothing is saved).", SAct::DebugRegion),
            ("runner.debug_world_map", "World Map (debug)", "Travel a world's route graph: road fights with debug outcomes, the world clock.", SAct::DebugWorldMap),
        ];
        // The list scrolls when the scenes outgrow the panel (the wheel over it).
        let view = Rect::new(left.x, left.y + 14, left.w, left.h - 16);
        let content: i32 = scenes.iter().map(|s| if matches!(s.3, SAct::Arena | SAct::DebugRegion | SAct::DebugWorldMap) { 48 } else { 34 }).sum();
        let off = ui.scroll_begin("runner.scenes", view, content);
        let mut y = view.y - off;
        for (id, name, desc, act) in scenes {
            let extra = matches!(act, SAct::Arena | SAct::DebugRegion | SAct::DebugWorldMap);
            let h = if extra { 44 } else { 30 };
            let r = Rect::new(left.x + 5, y, left.w - 10, h);
            ui.p.panel(r, rgba(26, 34, 54, 255), BORDER, 2.0);
            ui.p.text(r.x as f32 + 5.0, r.y as f32 + 3.0, name, BODY, INFO);
            ui.p.text_fit(r.x as f32 + 5.0, r.y as f32 + 14.0, desc, SMALL, (r.w - 60) as f32, TEXT);
            ui.button(id, Rect::new(r.x + r.w - 50, r.y + 6, 45, 15), "Launch", act.clone(), true);
            if let SAct::Arena = act {
                let labels: Vec<String> = encounters.iter().map(|e| e.0.clone()).collect();
                ui.select("runner.arena.encounter", Rect::new(r.x + 5, r.y + 25, r.w - 60, 14), &labels, self.encounter, SAct::PickEncounter, !encounters.is_empty(), tip("Encounter", "The battle the arena launches."));
            } else if extra {
                let names: Vec<String> = worlds.iter().map(|w| cx.world_name(w)).collect();
                ui.select(&format!("{id}.world"), Rect::new(r.x + 5, r.y + 25, 160, 14), &names, self.world % worlds.len().max(1), SAct::PickWorld, worlds.len() > 1, Vec::new());
            }
            y += h + 4;
        }
        ui.scroll_end();
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 4.0, "DEVELOPER", SMALL, INFO);
        ui.button_tip("runner.reload", Rect::new(right.x + 6, right.y + 16, 70, 14), "Reload data", SAct::Reload, true, tip("Reload data", "Re-reads the data folder (catalog, worlds, maps, quests, services, XP curve)."));
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 36.0, "Parity checks (headless):", SMALL, DIM);
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 46.0, "cargo test -p emberwake_world -p emberwake_battle", SMALL, TEXT);
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 62.0, "Playtest recording: not ported yet.", SMALL, DIM);
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 78.0, &format!("Data: {}", cx.data_dir.display()), SMALL, DIM);
        let _ = GOOD;
        self.w = ui.widgets;
    }

    fn state(&self, cx: &Cx) -> Value {
        json!({"encounter": Self::encounters(cx).get(self.encounter).map(|e| e.0.clone())})
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
        for (i, (label, o)) in [("Won", "won"), ("Fled", "fled"), ("Lost", "lost")].iter().enumerate() {
            ui.button(&format!("debug_fight.{o}"), Rect::new(r.x + 10 + i as i32 * 50, r.y + r.h - 26, 46, 16), label, DFAct::Outcome(o), true);
        }
        ui.button("debug_fight.play", Rect::new(r.x + r.w - 80, r.y + r.h - 26, 70, 16), "Fight it", DFAct::Play, true);
        self.w = ui.widgets;
    }
    fn back(&mut self, _cx: &mut Cx) {}
    widget_plumbing!();
}
