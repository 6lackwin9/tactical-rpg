# scripts/ui/menus

System: S7, S13. Status: UI1-L built (main menu, new game, save / load slots, settings, hub); UI2-L built (party, level-up, skills loadout, gear, battle results); UI3-L built (quest journal).

Every screen is a headless `RefCounted` model plus a thin view (spec `docs/specs/UI1_L.md`):

| Screen id | Model | View |
| --- | --- | --- |
| (stack) | `menu_router.gd` (`MenuRouter`: push / pop / replace / clear_to, `ask` + `confirm` / `decline`, signals `changed`, `confirm_needed`, `notice`, `game_started`, `quit_requested`) | `main_menu_view.gd` (`MainMenuView`, host of `scenes/main_menu.tscn`: renders the top screen, the question overlay, the game scene) |
| `main` | `main_menu_model.gd` | `main_screen_view.gd` |
| `new_game` | `new_game_model.gd` | `new_game_view.gd` |
| `load`, `save` | `save_slots_model.gd` | `save_slots_view.gd` (rows in a `SlotScroll` ScrollContainer; `BackButton` fixed below it) |
| `settings` | `settings_model.gd` | `settings_view.gd` |
| `hub` | `hub_model.gd` | `hub_view.gd` |
| `party` | `party_model.gd` (+ `stats_model.gd`, `party_preview.gd`) | `party_view.gd` |
| `level_up` | `level_up_model.gd` | `level_up_view.gd` |
| `loadout` | `loadout_model.gd` | `loadout_view.gd` |
| `gear` | `gear_model.gd` (+ `party_gear_bridge.gd` for `CharacterEquipmentModal`) | `gear_view.gd` |
| `results` | `battle_results_model.gd` | `battle_results_view.gd` |
| `journal` | `journal_model.gd` (read only over `QuestRunner`) | `journal_view.gd` |

Rules: models call only rule APIs (`NewGame`, `SaveGame`, `WorldCatalog`, `CurrentGame`, `SettingsFile`) and never touch a Node; actions return `Array[String]` problems (empty = done); a new screen adds a `MenuRouter` id, a `make_model` case and a view case in `MainMenuView.render`. Party screens take `{member}` (results take `{award, advanced, outcome}`; outcome won / lost / fled sets the headline, BATTLE1-L) through `push(id, args)`; popping a screen calls `refresh()` on the one below when it has it. Tests: `scripts/tests/menus_test_suite.gd`, `scripts/tests/party_screen_test_suite.gd`, `scripts/tests/save_io_test_suite.gd`, `scripts/tests/journal_test_suite.gd`. A view may set the `MainMenuView.FOCUS_META` meta to the control the host focuses first.
