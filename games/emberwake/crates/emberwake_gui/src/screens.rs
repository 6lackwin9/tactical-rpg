//! The game's side of the screen stack (MenuRouter, on the engine's `aegis_ui::screen`): its half of the shared
//! context (`GameCx`: the data, the manifest, the game in play (CurrentGame), the save folder, the event log, the
//! tuning), its settings defaults and entering a game. Screens use `Cx` (the engine context over `GameCx`) and queue
//! navigation in `cx.nav`; they never call each other.

use crate::tuning::Tuning;
use aegis_content::manifest::Manifest;
use aegis_ui::tuning::TuningFile;
use emberwake_world::{save, Data, GameState};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

pub use aegis_ui::screen::{show_errors, title, Key, Screen};

/// The shared context screens get: the engine's half, and `GameCx` through Deref.
pub type Cx = aegis_ui::Cx<GameCx>;
pub type Nav = aegis_ui::Nav<GameCx>;

/// The game's own settings keys and defaults (sound, display, text size and playtest recording are the
/// engine's: `Settings::engine_defaults`). PLAYUI1-M: `map_names` (`hover` or `always`: the region map's marker names)
/// and `map_legend_open` (the region map's legend, open or collapsed to its header). STAGE1: `battle_view` (`stage` or
/// `flat`, `stage::VIEWS`; a debug setting set in the Scene Runner since STAGE1-C). STAGE1-O: `see_through`
/// (`selected` or `always`, `stage::SEE_THROUGH`: when covered units are outlined in battle). EXPLOREVIEW1:
/// `region_view` (`square` or `tilted`, `mapview::VIEWS`; a debug setting set in the Scene Runner).
pub fn default_settings() -> Value {
    // RULESET1-U slice 6: every battle overlay group at its default (`battle::OVERLAY_GROUPS`; VISCLAR1 VC6: on but
    // `all_bars`, `test_hazards` and `test_arena`). VC5: the debug key `debug_party_foresight_off` is never a default.
    let overlays: serde_json::Map<String, Value> = crate::battle::OVERLAY_GROUPS.iter().map(|g| (g.0.to_string(), Value::Bool(g.3))).collect();
    // INPUT1 GH7: "Auto-fill the rest" on by default; PB3: the playback speed 1x
    json!({"anim_speed": 1.0, "skip_anim": false, "show_tells": true, "warn_missing": true, "auto_fill": true, "map_names": "hover", "map_zoom": 1.0, "walk_step_ms": crate::mapview::WALK_STEP_MS.2, crate::mapview::TURN_BEAT_KEY: false, crate::mapview::EQUAL_SPEED_KEY: false, crate::mapview::TURN_CORNER_KEY: crate::mapview::TURN_CORNERS[0].name(), "map_legend_open": true, "battle_view": "stage", "region_view": "square", "see_through": "selected", "battle_overlays": overlays,
        "playback_speed": "1x"})
}

/// The game in play (CurrentGame): the state, the slot it came from and its copy at the last save or load.
pub struct Game {
    pub gs: GameState,
    pub slot: i64,
    pub clean: Value,
    /// A battle, conversation or shop is open (saving and leaving wait).
    pub busy: bool,
    /// The UI Gallery's sample game (never saved; the gallery drops it when it goes, and only it).
    pub sample: bool,
    /// CAMPAIGN1 W1: campaign events waiting for their notice (the map screens show them one by one, `runover`).
    pub notices: Vec<Value>,
}

impl Game {
    pub fn new(gs: GameState, slot: i64, sample: bool) -> Self {
        let clean = gs.to_value();
        Self { gs, slot, clean, busy: false, sample, notices: Vec::new() }
    }
}

/// The game's half of the shared context.
pub struct GameCx {
    pub data: Arc<Data>,
    /// game.json: the id, and the title the window and the main menu show.
    pub manifest: Manifest,
    pub game: Option<Game>,
    pub save_dir: PathBuf,
    pub tuning: TuningFile<Tuning>,
    /// The region map's event log (newest last).
    pub log: Vec<String>,
    pub encounter: String,
    pub seed: Option<i64>,
    /// A debug game (Scene Runner): fights get Skip / Fled / Lost buttons and nothing is saved.
    pub debug: bool,
    /// The Godot export folder the data was loaded from (Reload re-reads it).
    pub data_dir: PathBuf,
}

impl GameCx {
    pub fn gs(&self) -> &GameState {
        &self.game.as_ref().expect("a game is running").gs
    }

    pub fn gs_mut(&mut self) -> &mut GameState {
        &mut self.game.as_mut().expect("a game is running").gs
    }

    pub fn has_game(&self) -> bool {
        self.game.is_some()
    }

    pub fn busy(&self) -> bool {
        self.game.as_ref().is_some_and(|g| g.busy)
    }

    pub fn set_busy(&mut self, b: bool) {
        if let Some(g) = self.game.as_mut() {
            g.busy = b;
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.game.as_ref().is_some_and(|g| !aegis_core::json::deep_equal(&g.gs.to_value(), &g.clean))
    }

    pub fn mark_saved(&mut self, slot: i64) {
        if let Some(g) = self.game.as_mut() {
            g.slot = slot;
            g.clean = g.gs.to_value();
        }
    }

    pub fn log(&mut self, line: impl Into<String>) {
        self.log.push(line.into());
        if self.log.len() > 200 {
            self.log.remove(0);
        }
    }

    /// Writes the autosave (slot 0); problems go to the log. Nothing while the run is over (CAMPAIGN1 F3: no slot holds
    /// a run-over state).
    pub fn autosave(&mut self) {
        if self.debug {
            return;
        }
        let Some(g) = self.game.as_ref() else { return };
        if !emberwake_world::campaign::save_refusal(&g.gs).is_empty() {
            return;
        }
        let errors = save::write(&self.data, &self.save_dir, &g.gs, save::AUTOSAVE_SLOT, 0);
        if errors.is_empty() {
            self.mark_saved(save::AUTOSAVE_SLOT);
        } else {
            self.log(format!("Autosave failed: {}", errors.join("; ")));
        }
    }

    pub fn end_game(&mut self) {
        self.game = None;
        self.debug = false;
        self.log.clear();
    }

    pub fn world_name(&self, world: &str) -> String {
        aegis_core::json::s_or(self.data.manifest(world), "name", world)
    }

    pub fn location_name(&self, world: &str, loc: &str) -> String {
        if loc.is_empty() {
            return String::new();
        }
        let hit = if self.data.has_world(world) { self.data.resolve(world, "locations", loc) } else { aegis_core::json::empty() };
        aegis_core::json::s_or(hit, "name", loc)
    }
}

impl aegis_ui::Context for GameCx {
    const MORE_COMMANDS: &'static str = "battle commands work while a battle is open";

    fn note(&mut self, line: String) {
        self.log(line);
    }

    fn agent_summary(&self) -> Value {
        match self.game.as_ref() {
            Some(g) => json!({"world": g.gs.world_id, "location": g.gs.location_id, "position": [g.gs.position.0, g.gs.position.1], "gold": g.gs.party.gold,
                "supplies": g.gs.party.supplies, "time": g.gs.time, "dirty": self.is_dirty(), "busy": g.busy}),
            None => Value::Null,
        }
    }

    fn agent_game(&self) -> Value {
        match &self.game {
            Some(g) => json!({"ok": true, "state": g.gs.to_value()}),
            None => json!({"ok": false, "error": "no game in progress"}),
        }
    }
}

/// ASSETORG2 section 4: a world character's body key, palette and portrait chain (`emberwake_content::art`).
pub fn character_art(d: &Data, character: &aegis_core::json::Dict) -> (String, String, Vec<String>) {
    use emberwake_content::art;
    let unit = d.entry("units", &aegis_core::json::s(character, "base_unit"));
    let class = d.entry("classes", &aegis_core::json::s(unit, "class_id"));
    let body = art::body_of(character, unit, class);
    let palette = art::palette_of(character, unit);
    let tier = art::portrait_tier(character, unit);
    let portraits = art::portrait_chain(&tier, &aegis_core::json::s(character, "id"), &body);
    (body, palette, portraits)
}

/// Starts playing `gs` (a new game or a loaded save) and shows the region map, or the story stage while a story
/// sequence runs (STORYUI1 5.4: a new game in a world with a start sequence, a save made during one).
pub fn enter_game(cx: &mut Cx, gs: GameState, slot: i64, warnings: Vec<String>) {
    let story = emberwake_world::story::is_running(&gs);
    cx.game = Some(Game::new(gs, slot, false));
    cx.log.clear();
    if !warnings.is_empty() {
        cx.notice = warnings;
    }
    if story {
        let stage = crate::story::StoryStage::new(cx);
        cx.nav.push(Nav::Reset(Box::new(stage)));
    } else {
        let map = crate::region::RegionMap::new(cx);
        cx.nav.push(Nav::Reset(Box::new(map)));
    }
}
