//! Scene Runner (scripts/ui/runner; dev only): the launcher of every scene, opened from the main menu's Tools. The
//! combat arena takes any catalog or world encounter; Exploration (RUNNER2) plays a world from a picked location or its
//! world map without saving, a fight skipped (settled as a win at no cost) or fought (DebugFight). Reload re-reads the
//! data folder from disk.

use aegis_ui::theme::th;
use aegis_ui::canvas::{rgba, Rect};
use crate::screens::{enter_game, show_errors, Cx, GameCx, Nav, Screen};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, H, SMALL, W};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s, s_or, Dict};
use emberwake_rules::ruleset::{RuleKind, Ruleset, PLAN_BADGES, REGISTER};
use emberwake_world::{explore, new_game};
use indexmap::IndexMap;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum SAct {
    MainMenu,
    Arena,
    PickEncounter(usize),
    PickRuleset(usize),
    /// RULESET1 12: the Arena's extension toggles shown or hidden.
    ShowRules,
    /// RULESET1 3.2: one extension (an index into `extensions()`) switched on or off for the Arena's run.
    ToggleRule(usize),
    /// RULESET1 V2, V3: "Reveal enemy plans" for the Arena's run.
    Reveal,
    /// INPUT1 MR11: the Arena's "Diagonal melee" (an index into DIAGONALS).
    PickDiagonal(usize),
    ContentForge,
    WorldForge,
    Dojo,
    Gallery,
    Exploration,
    PickWorld(usize),
    PickLocation(usize),
    Reload,
    Playtests,
    /// STAGE1-C: the battle view (an index into `stage::VIEWS`), a debug setting kept out of the player's settings.
    PickView(usize),
    /// EXPLOREVIEW1: the region map's view.
    PickRegionView(usize),
    Exit,
}

/// INPUT1 MR11: the Arena's "Diagonal melee" picks (label, the host's `melee_diagonal`; None = the ruleset row's value).
const DIAGONALS: [(&str, Option<&str>); 4] = [("Ruleset", None), ("None", Some("none")), ("Maneuver", Some("maneuver")), ("All", Some("all"))];

/// The Arena's extension toggles: columns and row height.
const RULE_COLUMNS: usize = 4;
const RULE_ROW_H: i32 = 13;

/// The rules the Arena may switch one by one (RULESET1 3.2: the register's extensions, in its order).
fn extensions() -> Vec<&'static str> {
    REGISTER.iter().filter(|r| r.kind == RuleKind::Extension).map(|r| r.id).collect()
}

/// The ruleset chip's text (the Arena card's and, RULESET1-U, the Arena battle's): the base ruleset's name, the
/// switches `spec` ("-id,+id"), INPUT1 MR11's diagonal melee pick (`diagonal`, "" = the row's value: not named) and
/// whether the enemy plans are shown ("Timeline -pierce +reach, diagonal: all, plans shown").
pub(crate) fn chip_text(name: &str, spec: &str, diagonal: &str, plans_shown: bool) -> String {
    let mut t = name.to_string();
    if !spec.is_empty() {
        t += &format!(" {}", spec.replace(',', " "));
    }
    if !diagonal.is_empty() {
        t += &format!(", diagonal: {diagonal}");
    }
    if plans_shown {
        t += ", plans shown";
    }
    t
}

pub struct SceneRunner {
    encounter: usize,
    /// An index into `rulesets`: 0 is World, then the catalog's ruleset rows in file order.
    ruleset: usize,
    /// RULESET1 3.2: the extensions switched for the Arena's run (on or off whatever the picked ruleset has; one equal
    /// to the ruleset's own changes nothing).
    rules: IndexMap<&'static str, bool>,
    /// The extension toggles are shown.
    show_rules: bool,
    /// RULESET1 V2: every enemy plan is revealed (the battle's `plan_reveal`; read under the rule `plan_badges`).
    reveal: bool,
    /// INPUT1 MR11: an index into DIAGONALS (0 = the ruleset row's value).
    diagonal: usize,
    world: usize,
    /// Exploration's start: an index into `starts`.
    location: usize,
    w: Vec<Widget<SAct>>,
}

impl SceneRunner {
    pub fn new() -> Self {
        Self { encounter: 0, ruleset: 0, rules: IndexMap::new(), show_rules: false, reveal: false, diagonal: 0, world: 0, location: 0, w: Vec::new() }
    }

    /// The ruleset the picked encounter plays by before the toggles: the picked one, or for World the ruleset of the
    /// encounter's world (classic for a catalog encounter: RULESET1 3.1.2).
    fn base_ruleset(&self, cx: &Cx) -> Ruleset {
        match self.picked(cx) {
            Some(id) => Ruleset::of_or_classic(&cx.data.cat, &id),
            None => match Self::encounters(cx).get(self.encounter).and_then(|e| e.1.clone()) {
                Some(world) => cx.data.ruleset(&world),
                None => Ruleset::of_or_classic(&cx.data.cat, emberwake_rules::ruleset::CLASSIC),
            },
        }
    }

    /// The `rules` switches of the run ("-id,+id"): the toggles that differ from the base ruleset.
    fn rules_spec(&self, base: &Ruleset) -> String {
        self.rules.iter().filter(|(id, on)| base.has(id) != **on).map(|(id, on)| format!("{}{id}", if *on { '+' } else { '-' })).collect::<Vec<_>>().join(",")
    }

    /// The Combat Arena's rules (RULESET1-R, CLN-46): the encounter's own (its world's; classic for a catalog encounter),
    /// or a ruleset picked for the run.
    /// The picker: "World" (the encounter's world's own), then a row per ruleset of the catalog
    /// (id, name).
    fn rulesets(cx: &Cx) -> Vec<(Option<String>, String)> {
        let rows = Ruleset::ids(&cx.data.cat).into_iter().map(|id| {
            let name = Ruleset::of_or_classic(&cx.data.cat, &id).name;
            (Some(id), name)
        });
        std::iter::once((None, "World".to_string())).chain(rows).collect()
    }

    /// The ruleset id the picker names (None = World).
    fn picked(&self, cx: &Cx) -> Option<String> {
        Self::rulesets(cx).into_iter().nth(self.ruleset).and_then(|r| r.0)
    }

    /// The rules the Arena's run plays by: the base ruleset with the switches.
    fn effective_ruleset(&self, cx: &Cx) -> Ruleset {
        let mut r = self.base_ruleset(cx);
        // only extensions are listed, so the switches cannot fail
        let _ = r.adjust(&self.rules_spec(&r));
        r
    }

    /// The ruleset chip: the run's ruleset name, its switches and the reveal ("Timeline -pierce +reach, plans shown").
    fn chip(&self, cx: &Cx) -> String {
        let base = self.base_ruleset(cx);
        chip_text(&base.name, &self.rules_spec(&base), self.diagonal_pick(), self.reveal && self.effective_ruleset(cx).has(PLAN_BADGES))
    }

    /// INPUT1 MR11: the run's `melee_diagonal` ("" = the ruleset row's value).
    fn diagonal_pick(&self) -> &'static str {
        DIAGONALS[self.diagonal.min(DIAGONALS.len() - 1)].1.unwrap_or("")
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
    /// plain new game when None (the region map enters the start location, the world map opens over it). STORYUI1-U1
    /// (planner ruling): a location other than the start drops the world's running start sequence, so the tester lands
    /// where they asked; the start location and the world map keep it (the story stage plays it).
    fn start_game(d: &emberwake_world::Data, world: &str, start: Option<&str>, seed: i64) -> Result<(emberwake_world::GameState, Vec<Value>), Vec<String>> {
        let mut gs = new_game::create_for_world(d, world, seed)?;
        if start.is_some_and(|q| q != explore::start_location(d, world)) {
            gs.story.running = None;
        }
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
            SAct::PickRuleset(i) => self.ruleset = i.min(Self::rulesets(cx).len() - 1),
            SAct::ShowRules => self.show_rules = !self.show_rules,
            SAct::ToggleRule(i) => {
                if let Some(id) = extensions().get(i).copied() {
                    let on = self.effective_ruleset(cx).has(id);
                    self.rules.insert(id, !on);
                }
            }
            SAct::Reveal => self.reveal = !self.reveal,
            SAct::PickDiagonal(i) => self.diagonal = i.min(DIAGONALS.len() - 1),
            SAct::Arena => {
                let list = Self::encounters(cx);
                if let Some((_, world, id)) = list.get(self.encounter) {
                    let ruleset = self.picked(cx);
                    let rules = self.rules_spec(&self.base_ruleset(cx));
                    // a reveal ticked under another ruleset stays off where plan_badges is not on (classic stays byte-identical)
                    let reveal = self.reveal && self.effective_ruleset(cx).has(PLAN_BADGES);
                    let b = crate::battle::BattleScreen::arena_full(cx, world.clone(), id, ruleset, &rules, self.diagonal_pick(), reveal);
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
            SAct::PickView(i) => {
                if let Some((v, _)) = crate::stage::VIEWS.get(i) {
                    cx.settings.v["battle_view"] = serde_json::json!(v);
                    if let Err(e) = cx.settings.save() {
                        show_errors(cx, vec![format!("Settings could not be saved: {e}")]);
                    }
                }
            }
            SAct::PickRegionView(i) => {
                if let Some((v, _)) = crate::mapview::VIEWS.get(i) {
                    cx.settings.v["region_view"] = serde_json::json!(v);
                    if let Err(e) = cx.settings.save() {
                        show_errors(cx, vec![format!("Settings could not be saved: {e}")]);
                    }
                }
            }
            SAct::Playtests => {
                let home = cx.data_dir.parent().map(std::path::Path::to_path_buf).unwrap_or_default();
                let repo = vec![home.join("playtests/inbox"), home.join("playtests/repro")];
                let launch: aegis_ui::recordings::Launch<GameCx> = |cx, session, take| Some(Box::new(crate::battle::BattleScreen::playback(cx, session, take)));
                let r = aegis_ui::recordings::Recordings::new(cx, launch, |cx| aegis_content::fingerprint::of(&cx.data_dir), repo);
                cx.nav.push(Nav::Push(Box::new(r)));
            }
            SAct::Reload => {
                match emberwake_world::Data::open(&cx.data_dir) {
                    Ok((d, problems)) => {
                        cx.data = Arc::new(d);
                        cx.notice.push(format!("Reloaded the data from {}.", cx.data_dir.display()));
                        cx.notice.extend(problems);
                    }
                    Err(e) => show_errors(cx, vec![format!("Reload failed: {e:#}")]),
                }
                // REGIONGROUND1-L: an edited art file shows after this (`Art` never re-reads a file it holds).
                cx.art = aegis_ui::art::Art::layered(cx.art.layers().clone());
                // LOOK1-E1: the theme files too (a problem keeps the theme in force and shows a notice).
                aegis_ui::theme::reload(cx);
            }
        }
    }
}

impl Screen<GameCx> for SceneRunner {
    fn id(&self) -> &'static str {
        "scene_runner"
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<SAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().tool_bg);
        ui.p.text(12.0, 8.0, &format!("{}  ·  SCENE RUNNER", cx.manifest.title.to_uppercase()), 10.0, th().gold);
        ui.p.text(12.0, 27.0, "Launch any scene; Esc returns here.", SMALL, th().dim);
        ui.button("runner.exit", Rect::new(W - 40, 6, 34, 13), "Exit", SAct::Exit, true);
        let left = Rect::new(12, 41, 383, 300);
        let right = Rect::new(402, 41, 235, 128);
        ui.p.panel(left, th().tool_pane, th().edge, 2.0);
        ui.p.panel(right, th().tool_pane, th().edge, 2.0);
        ui.p.text(left.x as f32 + 6.0, left.y as f32 + 4.0, "SCENES", SMALL, th().info);
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
        // RULESET1 12: the Arena's second row (reveal, chip, the toggles' button) and, shown, its extension toggles
        let exts = extensions();
        let rule_rows = exts.len().div_ceil(RULE_COLUMNS) as i32;
        let arena_h = 32 + if self.show_rules { rule_rows * RULE_ROW_H + 2 } else { 0 };
        let cards: Vec<(Vec<String>, bool, i32)> = scenes
            .iter()
            .map(|s| {
                let lines = ui.p.wrap(s.2, SMALL, desc_w);
                let extra = matches!(s.3, SAct::Arena | SAct::Exploration);
                let extra_h = match s.3 {
                    SAct::Arena => arena_h,
                    SAct::Exploration => 16,
                    _ => 0,
                };
                let h = (3.0 + name_h + 1.0 + lines.len() as f32 * line_h + extra_h as f32 + 3.0).ceil() as i32;
                (lines, extra, h.max(30))
            })
            .collect();
        let effective = self.effective_ruleset(cx);
        let chip = self.chip(cx);
        let content: i32 = cards.iter().map(|c| c.2 + 4).sum();
        let off = ui.scroll_begin("runner.scenes", view, content);
        let mut y = view.y - off;
        for ((id, name, _, act), (lines, extra, h)) in scenes.into_iter().zip(cards) {
            let r = Rect::new(left.x + 5, y, left.w - 10, h);
            ui.p.panel(r, th().game("slot_filled", rgba(26, 34, 54, 255)), th().edge, 2.0);
            ui.p.text(r.x as f32 + 5.0, r.y as f32 + 3.0, name, BODY, th().info);
            let top = r.y as f32 + 3.0 + name_h + 1.0;
            for (i, line) in lines.iter().enumerate() {
                ui.p.text(r.x as f32 + 5.0, top + i as f32 * line_h, line, SMALL, th().text);
            }
            let pick_y = (top + lines.len() as f32 * line_h + 1.0).round() as i32;
            ui.button(id, Rect::new(r.x + r.w - 50, r.y + 6, 45, 15), "Launch", act.clone(), true);
            if let SAct::Arena = act {
                let labels: Vec<String> = encounters.iter().map(|e| e.0.clone()).collect();
                ui.select("runner.arena.encounter", Rect::new(r.x + 5, pick_y, r.w - 214, 14), &labels, self.encounter, SAct::PickEncounter, !encounters.is_empty(), tip("Encounter", "The battle the arena launches."));
                // INPUT1 MR11: Diagonal melee to compare (Ruleset: the row's value)
                let diags: Vec<String> = DIAGONALS.iter().map(|x| x.0.to_string()).collect();
                let diag_tip = tip("Diagonal melee", "Ruleset: the row's value. None: no diagonal hits. Maneuver: Maneuver skills hit diagonally. All: every melee attack and the riposte do.");
                ui.select("runner.arena.diagonal", Rect::new(r.x + r.w - 205, pick_y, 66, 14), &diags, self.diagonal, SAct::PickDiagonal, true, diag_tip);
                let rules: Vec<String> = Self::rulesets(cx).into_iter().map(|x| x.1).collect();
                ui.select("runner.arena.ruleset", Rect::new(r.x + r.w - 135, pick_y, 80, 14), &rules, self.ruleset, SAct::PickRuleset, true, tip("Rules", "World: the encounter's world's ruleset (classic for a catalog encounter). Any other row of the catalog's rulesets: play it by those rules."));
                // RULESET1 12: "Reveal enemy plans" (V2: under the rule plan_badges), the ruleset chip, the toggles
                let row_y = pick_y + 17;
                ui.checkbox("runner.arena.reveal", r.x + 5, row_y + 2, "Reveal enemy plans", self.reveal, SAct::Reveal, effective.has(PLAN_BADGES));
                let chip_r = Rect::new(r.x + 110, row_y, r.w - 204, 14);
                ui.p.panel(chip_r, th().game("chip", rgba(30, 44, 70, 255)), th().edge, 2.0);
                ui.p.text_fit(chip_r.x as f32 + 4.0, chip_r.y as f32 + 2.0, &chip, SMALL, (chip_r.w - 8) as f32, if effective.id == emberwake_rules::ruleset::DEFAULT && self.rules_spec(&self.base_ruleset(cx)).is_empty() { th().dim } else { th().gold });
                let chip_tip = format!("Rules on: {}.", if effective.rules.is_empty() { "none (classic)".to_string() } else { effective.rules.join(", ") });
                ui.push("runner.arena.chip", "label", chip.clone(), chip_r, true, false, None, tip(&format!("Ruleset: {chip}"), &chip_tip));
                ui.button_full("runner.arena.rules", Rect::new(r.x + r.w - 90, row_y, 85, 14), "Extensions", SAct::ShowRules, true, self.show_rules, BODY, tip("Extensions", "Switch single extensions of the ruleset on or off for this run (RULESET1 3.2); options belong to their ruleset."));
                if self.show_rules {
                    let col_w = (r.w - 10) / RULE_COLUMNS as i32;
                    for (i, id) in exts.iter().enumerate() {
                        let (col, row) = ((i % RULE_COLUMNS) as i32, (i / RULE_COLUMNS) as i32);
                        ui.checkbox(&format!("runner.arena.rule.{id}"), r.x + 5 + col * col_w, row_y + 18 + row * RULE_ROW_H, &id.replace('_', " "), effective.has(id), SAct::ToggleRule(i), true);
                    }
                }
            } else if extra {
                let names: Vec<String> = worlds.iter().map(|w| cx.world_name(w)).collect();
                ui.select(&format!("{id}.world"), Rect::new(r.x + 5, pick_y, 140, 14), &names, self.world % worlds.len().max(1), SAct::PickWorld, worlds.len() > 1, tip("World", "The world to explore."));
                let labels: Vec<String> = starts.iter().map(|p| p.0.clone()).collect();
                ui.select(&format!("{id}.location"), Rect::new(r.x + 149, pick_y, r.w - 204, 14), &labels, self.location, SAct::PickLocation, labels.len() > 1, tip("Start", "Where the party starts: a location, or the world map at the start location."));
            }
            y += h + 4;
        }
        ui.scroll_end();
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 4.0, "DEVELOPER", SMALL, th().info);
        ui.button_tip("runner.reload", Rect::new(right.x + 6, right.y + 16, 70, 14), "Reload data", SAct::Reload, true, tip("Reload data", "Re-reads the data folder (catalog, worlds, maps, quests, services, XP curve) and the art files."));
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 36.0, "Parity checks (headless):", SMALL, th().dim);
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 46.0, "cargo test -p emberwake_world -p emberwake_battle", SMALL, th().text);
        ui.button_tip("runner.playtests", Rect::new(right.x + 6, right.y + 58, 70, 14), "Playtests", SAct::Playtests, true, tip("Playtests", "Recordings of played battles with their notes (F8): play back, take over at a round, copy to the repo inbox; the recording toggle."));
        ui.p.text(right.x as f32 + 82.0, right.y as f32 + 61.0, if cx.playtest.on { "Recording on" } else { "Recording off" }, SMALL, if cx.playtest.on { th().good } else { th().dim });
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 78.0, &format!("Data: {}", cx.data_dir.display()), SMALL, th().dim);
        // STAGE1-C: the battle view for testing (Stage ships; Flat is the board seen from above)
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 96.0, "Battle view", SMALL, th().text);
        let views: Vec<String> = crate::stage::VIEWS.iter().map(|v| v.1.to_string()).collect();
        let at = crate::stage::VIEWS.iter().position(|v| cx.settings.v["battle_view"].as_str() == Some(v.0)).unwrap_or(0);
        ui.select("runner.battle_view", Rect::new(right.x + 82, right.y + 93, 90, 14), &views, at, SAct::PickView, true, tip("Battle view (debug)", crate::stage::VIEW_TIP));
        // EXPLOREVIEW1: the region map's view for testing (Square ships until the review picks one)
        ui.p.text(right.x as f32 + 6.0, right.y as f32 + 114.0, "Region view", SMALL, th().text);
        let views: Vec<String> = crate::mapview::VIEWS.iter().map(|v| v.1.to_string()).collect();
        let at = crate::mapview::VIEWS.iter().position(|v| v.0 == crate::mapview::View::of_setting(&cx.settings.v["region_view"]).name()).unwrap_or(0);
        ui.select("runner.region_view", Rect::new(right.x + 82, right.y + 111, 90, 14), &views, at, SAct::PickRegionView, true, tip("Region view (debug)", crate::mapview::VIEW_TIP));
        self.w = ui.widgets;
    }

    fn state(&self, cx: &Cx) -> Value {
        let world = self.world_id(cx);
        let start = world.as_deref().map(|w| Self::starts(&cx.data, w)).and_then(|st| st.get(self.location.min(st.len().saturating_sub(1))).cloned());
        let base = self.base_ruleset(cx);
        json!({"encounter": Self::encounters(cx).get(self.encounter).map(|e| e.0.clone()), "ruleset": Self::rulesets(cx).get(self.ruleset).map(|r| r.1.clone()), "world": world, "start": start.map(|p| p.1.unwrap_or_else(|| "world_map".into())),
            "rules": self.rules_spec(&base), "rules_on": self.effective_ruleset(cx).rules, "reveal": self.reveal, "melee_diagonal": self.diagonal_pick(), "chip": self.chip(cx)})
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
                // no battle: the default tally (a skip = every enemy killed, REWARD1 2.5)
                let res = crate::battle::after_fight(&d, gs, &self.runner, &self.fight, outcome, None);
                cx.set_busy(false);
                crate::battle::log_settled(cx, &self.runner, &res);
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
        ui.p.fill(Rect::new(0, 0, W, H), th().scrim);
        let r = Rect::new(W / 2 - 130, H / 2 - 50, 260, 100);
        ui.frame(r, "Fight (debug)", false);
        let name = s_or(cx.data.resolve(&cx.gs().world_id, "encounters", &s(&self.fight, "encounter")), "name", "Enemies");
        ui.p.text(r.x as f32 + 10.0, r.y as f32 + 30.0, &format!("{name} ({})", s(&self.fight, "source")), BODY, th().text);
        ui.p.text_fit(r.x as f32 + 10.0, r.y as f32 + 44.0, "Skip settles it as a win at no cost; Fight it plays the battle.", SMALL, (r.w - 20) as f32, th().dim);
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

    /// The picker index of the catalog's ruleset row `id` (0 is World).
    fn row_of(cx: &Cx, id: &str) -> usize {
        1 + Ruleset::ids(&cx.data.cat).iter().position(|r| r == id).expect("the ruleset row")
    }

    /// Adds a scratch ruleset row `plain`: the classic row's numbers, no rules, `rest_ap: on_use`.
    fn with_third_row(cx: &mut Cx) {
        let d = std::sync::Arc::make_mut(&mut cx.data);
        let mut row = d.cat.rulesets.get("classic").cloned().expect("the classic row");
        row.insert("id".into(), json!("plain"));
        row.insert("name".into(), json!("Plain"));
        row.insert("rest_ap".into(), json!("on_use"));
        std::sync::Arc::make_mut(&mut d.cat).rulesets.insert("plain".into(), row);
    }

    /// CLN-46: the Arena's picker lists World and one entry per catalog ruleset row (a third row appears), picking it
    /// plays by it; the chip is dim at the default ruleset with no switches and lit for another row.
    #[test]
    fn the_arena_ruleset_picker_lists_the_catalog_rows() {
        let mut cx = crate::hud::hud_tests::game_cx();
        with_third_row(&mut cx);
        let mut r = SceneRunner::new();
        let rows = SceneRunner::rulesets(&cx);
        assert_eq!(rows.len(), 1 + Ruleset::ids(&cx.data.cat).len());
        assert!(rows.iter().any(|x| x.1 == "Plain") && rows[0].1 == "World");
        let at = row_of(&cx, "plain");
        r.act(&mut cx, SAct::PickRuleset(at));
        assert_eq!(r.base_ruleset(&cx).id, "plain");
        assert_eq!(r.state(&cx)["ruleset"], json!("Plain"));
        r.act(&mut cx, SAct::PickRuleset(99));
        assert_eq!(r.ruleset, rows.len() - 1, "clamped to the last row");
    }

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

    /// STORYUI1-U1 (planner ruling): in a world with a start sequence, Exploration at a location other than the start
    /// drops the running sequence (the tester lands there, on the map); the start location and the world map keep it.
    #[test]
    fn exploration_away_from_the_start_drops_the_start_sequence() {
        let d = crate::story::tests::data();
        let start = explore::start_location(&d, "tale");
        let other = d.merged_entries("tale", "locations").iter().map(|e| s(e, "_qid")).find(|q| *q != start).expect("a second location");
        let running = |pick: Option<&str>| SceneRunner::start_game(&d, "tale", pick, 1).expect("a game").0.story.running.is_some();
        assert!(running(Some(&start)), "the start location keeps the start sequence");
        assert!(running(None), "the world map start keeps it");
        let (gs, _) = SceneRunner::start_game(&d, "tale", Some(&other), 1).expect("a game");
        assert!(gs.story.running.is_none(), "another location drops it");
        assert_eq!(gs.location_id, other, "the party stands where the tester asked");
    }

    /// RULESET1-G (index 12): the Arena's toggles. A catalog encounter under World is classic, nothing switched; under
    /// Timeline an extension of the row toggled off is the run's "-id" switch, one toggled back is no switch; the
    /// extension toggles show one checkbox per register extension; "Reveal enemy plans" needs the rule plan_badges;
    /// the chip names the ruleset and its switches.
    #[test]
    fn the_arena_toggles_switch_single_extensions_and_reveal_the_plans() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut r = SceneRunner::new();
        let first_catalog = SceneRunner::encounters(&cx).iter().position(|e| e.1.is_none()).expect("a catalog encounter");
        r.act(&mut cx, SAct::PickEncounter(first_catalog));
        assert!(r.effective_ruleset(&cx).is_classic() && r.rules_spec(&r.base_ruleset(&cx)).is_empty(), "World on a catalog encounter: classic");
        let at = row_of(&cx, "timeline");
        r.act(&mut cx, SAct::PickRuleset(at));
        let base = r.base_ruleset(&cx);
        assert!(!base.is_classic(), "the catalog has the timeline row");
        let exts = extensions();
        let (i, id) = exts.iter().enumerate().find(|(_, id)| base.has(id)).map(|(i, id)| (i, *id)).expect("the timeline row has an extension");
        r.act(&mut cx, SAct::ToggleRule(i));
        assert_eq!(r.rules_spec(&base), format!("-{id}"));
        assert!(!r.effective_ruleset(&cx).has(id));
        assert!(r.chip(&cx).ends_with(&format!("-{id}")) && r.chip(&cx).starts_with(&base.name), "{}", r.chip(&cx));
        r.act(&mut cx, SAct::ToggleRule(i));
        assert!(r.rules_spec(&base).is_empty(), "toggled back: no switch");
        r.act(&mut cx, SAct::ShowRules);
        let mut g = crate::hud::hud_tests::gfx();
        r.draw(&mut cx, &mut g, (-500, -500), true);
        assert_eq!(r.w.iter().filter(|w| w.id.starts_with("runner.arena.rule.")).count(), exts.len(), "one toggle per extension");
        let reveal = r.w.iter().find(|w| w.id == "runner.arena.reveal").expect("the reveal checkbox");
        assert_eq!(reveal.enabled, r.effective_ruleset(&cx).has(PLAN_BADGES), "the reveal needs plan_badges");
        r.act(&mut cx, SAct::Reveal);
        assert_eq!(r.state(&cx)["reveal"], json!(true));
    }

    /// INPUT1 MR11: the Arena's "Diagonal melee" select (Ruleset, None, Maneuver, All; default Ruleset: nothing sent)
    /// names its pick in the state and the chip; the battle it launches carries it (`arena_full`, tested in battle.rs).
    #[test]
    fn the_arena_diagonal_melee_select() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut r = SceneRunner::new();
        assert_eq!(r.state(&cx)["melee_diagonal"], json!(""), "Ruleset by default");
        let at = row_of(&cx, "timeline");
        r.act(&mut cx, SAct::PickRuleset(at));
        let plain = r.chip(&cx);
        assert!(!plain.contains("diagonal"), "{plain}");
        r.act(&mut cx, SAct::PickDiagonal(3));
        assert_eq!(r.state(&cx)["melee_diagonal"], json!("all"));
        assert_eq!(r.chip(&cx), format!("{plain}, diagonal: all"));
        let mut g = crate::hud::hud_tests::gfx();
        r.draw(&mut cx, &mut g, (-500, -500), true);
        let sel = r.w.iter().find(|w| w.id == "runner.arena.diagonal").expect("the select");
        assert!(sel.enabled && sel.text.contains("All"), "{}", sel.text);
        r.act(&mut cx, SAct::PickDiagonal(0));
        assert_eq!(r.chip(&cx), plain, "back to the row's value");
    }

    /// EXPLOREVIEW1 check 9: the Developer panel's select shows and saves `region_view`, and offers both views.
    #[test]
    fn the_region_view_select_saves_the_setting() {
        let dir = aegis_core::scratch::Scratch::vacant("emberwake_runner_view");
        std::fs::create_dir_all(&dir).expect("a scratch folder");
        let path = dir.join("settings.json");
        let mut cx = crate::hud::hud_tests::game_cx();
        cx.settings = aegis_ui::screen::Settings::load(path.clone(), crate::screens::default_settings());
        let mut g = crate::hud::hud_tests::gfx();
        let mut r = SceneRunner::new();
        crate::hud::hud_tests::frame(&mut r, &mut cx, &mut g, 1.0);
        assert!(r.widgets().iter().any(|w| w["id"] == "runner.region_view"), "the select");
        assert_eq!(cx.settings.v["region_view"], json!("square"), "the default");
        r.act(&mut cx, SAct::PickRegionView(1));
        assert_eq!(cx.settings.v["region_view"], json!(crate::mapview::VIEWS[1].0));
        let saved: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("saved")).expect("json");
        assert_eq!(saved["region_view"], json!("tilted"), "saved to the settings");
        crate::hud::hud_tests::frame(&mut r, &mut cx, &mut g, 1.0);
        let shown = r.widgets().into_iter().find(|w| w["id"] == "runner.region_view").expect("the select");
        assert_eq!(shown["text"], json!("Tilted"), "the select shows the pick: {shown}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
