# UI1-L: MenuRouter, main menu, new game, save/load slots with peek and autosave, settings file, in-game hub

Status: approved (user 2026-10-17) (design-brainstorm 2026-10-17; DECISIONS progression-save "UI menus baseline answers (user 2026-10-17)" and docs/specs/UI_MENUS_RULES.md)
Depends on: M4-L (merged: save v2, QuestRunner); EXP1-L merged (it adds the world manifest field `start_location`, `NewGame` seeding and save v3 in the same hub files: `world_schema.gd`, `world_catalog.gd`, `new_game.gd`, `save_game.gd`); run before or after DLG1-L, never in parallel with it (same hubs)
Lane: logic (schema first; commit 1 releases UI1-A)
Touches: `scripts/world/world_schema.gd` (`_world_fields`: `start_party`), `scripts/world/world_catalog.gd` (`manifest_errors`, resolved manifest value), `scripts/game_state/new_game.gd` (`create_for_world`), `scripts/game_state/save_game.gd` (slots, `peek`, `newest_slot`, `autosave`), new `scripts/game_state/current_game.gd`, new or extended `scripts/game_state/settings_file.gd` (shared with S14-L3), new `scripts/ui/menus/{menu_router,main_menu_model,new_game_model,save_slots_model,settings_model,hub_model}.gd` + views `scripts/ui/menus/*_view.gd`, new `scenes/main_menu.tscn`, `scripts/ui/runner/scene_runner.gd` (one "Main Menu" entry), new `scripts/tests/menus_test_suite.gd` + one line in `scripts/tests/suite_registry.gd`, `scripts/tests/save_test_suite.gd` (slot range check), `scripts/ui/menus/README.md`

## Goal

A player can start a new game in a playable world with its fixed starting party, continue the newest save, load or save one of three manual slots, rely on an autosave slot, change settings, and reach the in-game hub (party, journal, save, settings, quit). Every screen is a headless-testable `RefCounted` model plus a thin view; models change game data only through existing rule APIs.

## Save versioning (sequencing with EXP1, DLG1, TRAV1)

UI1 needs NO save version bump: `start_party` is world data, `peek` reads the existing envelope (`format`, `version`, `saved_at`, `game_state`), and the autosave slot is a `SaveGame` API change, not a format change. Recommendation (no question): UI1 takes no bump and does not ride EXP1's v3; if a later UI task needs a saved field (for example play time), it joins the next bump in flight. `peek` must read every version from 1 to the current `SaveGame.VERSION` without migrating (rule 3).

## Schema contract (logic lane, schema first: yes)

### World manifest field `start_party` (`WorldSchema._world_fields`)

| Field | Type | Default | Validation (`manifest_errors`, exact message) |
| --- | --- | --- | --- |
| `start_party` | Array of catalog unit ids (`refs` units, `ref_filter` side party) | `[]` = the nearest ancestor's value along `extends` (same lineage rule as `start_location`), else none | each id exists: `"Start party unit '%s' is not a known unit."`; side party: `"Start party unit '%s' is not a party unit."`; at most `PartyState.PARTY_MAX`: `"A start party holds at most 4 units."`; no duplicates: `"Start party lists '%s' twice."` |

`WorldCatalog.start_party(world_id) -> Array[String]` returns the resolved list. A world is PLAYABLE when its resolved `start_party` is non-empty and its resolved `start_location` (EXP1) is non-empty.

### `NewGame.create_for_world(world_id) -> Dictionary`

`{state, errors}`; errors `"World '%s' is not playable (no start party)."` / `"World '%s' is not playable (no start location)."`; else `create(world_id, start_party(world_id))` (EXP1's seed and DLG1's gold happen inside `create`).

### `SaveGame` slot API

| Member | Rule |
| --- | --- |
| `AUTOSAVE_SLOT: int = 0`, `MANUAL_SLOTS: Array[int] = [1, 2, 3]` | constants |
| `write(state, slot)` | slot must be 0..3: `"Save slot must be 0-3."` (replaces `"Save slot must be >= 1."`; `save_test_suite` updated by rule) |
| `autosave(state) -> Array[String]` | `write(state, AUTOSAVE_SLOT)` |
| `peek(slot) -> Dictionary` | `{exists, ok, slot, saved_at, version, world_id, location_id, party: [{id, base_unit, level}], error}`; never throws; `ok` false with `error` `"unreadable"` (bad JSON or format), `"newer version"` (version above `VERSION`) |
| `newest_slot() -> int` | slot with the largest `saved_at` among `ok` slots 0..3; -1 when none |

### `CurrentGame` (static; state on the Engine singleton like the catalogs)

`state: GameState` (null = no game), `slot: int` (last slot written or loaded, -1), `clean_copy: Dictionary` (`state.to_dict()` at the last save or load), `busy: bool` (set by a battle or dialogue scene host while it runs), `start(gs)`, `end()`, `is_dirty() -> bool` (`state.to_dict() != clean_copy`), `mark_saved(slot)`.

### Settings file (`SettingsFile`, shared contract with S14-L3)

`scripts/game_state/settings_file.gd`, static, `ConfigFile` at `user://settings.cfg`, `use_path(p)` for tests; `load()`, `save() -> Error`, `get_value(key, default)`, `set_value(key, value)`; whichever of S14-L3 and UI1-L lands first creates it, the other adds keys. Never part of a save slot.

| Key | Type | Default | Clamp | Owner |
| --- | --- | --- | --- | --- |
| `audio/master`, `audio/music`, `audio/sfx`, `audio/ui` | float | 1.0, 0.8, 0.8, 0.8 | 0.0..1.0 | S14-L3 (UI1 shows them; until S14-L3 lands the values are stored only) |
| `anim/speed` | float | 1.0 | 0.25..4.0 | UI1 (read by `FxSettings`, S14-L2) |
| `anim/instant` | bool | false | - | UI1 |
| `ui/text_scale` | float | 1.0 | 0.8..1.5 | UI1 (applied to the theme's default font size) |

Wrong type: default plus one warning.

### Screen models (`scripts/ui/menus/`)

| Model | State it exposes | Actions (return `Array[String]` problems, empty = done) |
| --- | --- | --- |
| `MenuRouter` | stack of screen ids, `top()`, `has_draft(top)` | `push(id)`, `pop()` (asks via `confirm_needed` signal when the top model has a draft), `replace(id)`, `clear_to(id)` |
| `MainMenuModel` | `can_continue` (`newest_slot() >= 0`), `show_tools` (`OS.is_debug_build()`) | `continue_game()`, `quit_requested()` |
| `NewGameModel` | playable worlds `[{id, name, tagline, party: [{unit, name, class, level}]}]` | `choose(world_id)` -> `create_for_world`, `CurrentGame.start`, `autosave`, router `clear_to("hub")` |
| `SaveSlotsModel` | rows for slots 0..3 from `peek` (`autosave` label for 0, `empty`, `unreadable`) | `load(slot)` (read, show `warnings`, `CurrentGame.start`), `save(slot)` (1..3 only; confirm when it exists; refused in battle), `delete(slot)` (confirm) |
| `SettingsModel` | the key table values (draft) | `set(key, value)`, `apply()` (save file, push to `FxSettings` and `AudioDirector` when present), `revert()` |
| `HubModel` | world and location names, party summary | `open(id)` for `party` (UI2), `journal` (UI3), `save`, `settings`; `quit_to_menu()` (confirm when `CurrentGame.is_dirty()`) |

## Rules (logic lane)

1. Models only call rule APIs (`NewGame`, `SaveGame`, `WorldCatalog`, `CurrentGame`, `SettingsFile`); views render model state and forward input; no model touches a Node.
2. Continue loads `newest_slot()` (autosave included); hidden when -1.
3. `peek` reads the JSON envelope and the raw `game_state` dict without `SaveMigrations`: `world_id`, `location_id`, party member `id`, `base_unit`, level from `XpCurve` on `total_xp`. Any missing piece leaves that field empty, never an error. A file of a newer version is listed but not loadable.
4. Autosave: after a new game (rule above), after every won or fled battle result (UI2 calls `CurrentGame` then `SaveGame.autosave`), and on EXP1's `autosave` event (the exploration scene already writes the save; it writes `AUTOSAVE_SLOT`). Manual saves never write slot 0.
5. Saving is refused while a battle or dialogue scene is active: `"You can't save during a battle or a conversation."` (the scene host sets `CurrentGame.busy`).
6. Load errors keep the player on the slots screen with the error list; migration warnings show once before entering the game.
7. Quit (main menu or hub): when `CurrentGame.is_dirty()` the router asks "Quit without saving?".
8. Keyboard: every action reachable by `ui_accept`, `ui_cancel`, arrows and Tab; `ui_cancel` = `pop()`.
9. Entering the game: the router's `game_started(gs)` signal; `scenes/main_menu.tscn` opens the exploration scene when EXP1's scene exists (`scenes/overworld/region_map_debug.tscn` today) and otherwise stays on the hub screen. Project main scene stays `scene_runner.tscn`; the planner switches it.

### Commits

1. Schema (releases UI1-A): `start_party` field, validation, `WorldCatalog.start_party`, playable rule, `NewGame.create_for_world`, `SaveGame` slots / `peek` / `newest_slot` / `autosave`, `CurrentGame`, `SettingsFile` keys, suite part A.
2. Models and router, suite part B.
3. Views and `scenes/main_menu.tscn`, scene runner entry.

## Acceptance checks (rule-derived; `menus_test_suite.gd`, saves in `TestHelpers.sandbox("menus")` via `SaveGame.set_base_dir`)

1. Each `start_party` message fires on a sandbox world; a child world with `[]` inherits the parent's list.
2. `create_for_world` on a playable world gives a state that passes `validate`, members equal the resolved `start_party` (ids, base units, levels); a world without a start party returns the exact error.
3. `write(state, 4)` and `write(state, -1)` return `"Save slot must be 0-3."`; slot 0 writes and reads back equal (`GameState.equals`).
4. `peek` of a missing slot gives `exists` false; of a corrupt file `ok` false, `error` "unreadable"; of a valid v2 file and a current-version file the world, location and party levels match `read` (no migration run by `peek`, the file is unchanged afterwards).
5. `newest_slot` picks the larger `saved_at` (two sandbox files with forged `saved_at`).
6. `NewGameModel.choose` leaves `CurrentGame.state` set, slot 0 written, router top `hub`.
7. `SaveSlotsModel.save(0)` is refused; `save(2)` on an existing slot needs a confirm; while `CurrentGame.busy` it returns the rule 5 message.
8. `HubModel.quit_to_menu` asks when dirty, not after `mark_saved`.
9. `SettingsFile` round trip, clamps and defaults (one case per key); deleting any save slot leaves the settings file unchanged.
10. Router: `pop()` on a model with a draft emits `confirm_needed` and does not pop until confirmed.
11. Existing save, progression, quest and world suites pass (the only changed expectation is check 3's message).

## Out of scope

Party screen and level-up (UI2-L), battle results (UI2-L), journal (UI3-L), lore codex, named or unlimited slots, cloud saves, play time, gamepad, localisation, the theme and art pass, audio playback (S14-L3), choosing a custom party.

## Open questions

None (UI menus answers: three manual slots plus autosave with `peek`, fixed `start_party`).
