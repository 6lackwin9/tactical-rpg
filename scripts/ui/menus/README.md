# scripts/ui/menus

System: S7, S13. Status: UI1-L built (main menu, new game, save / load slots, settings, hub); UI2-L (party screen, level-up, battle results) and UI3-L (journal) add their screens here.

Every screen is a headless `RefCounted` model plus a thin view (spec `docs/specs/UI1_L.md`):

| Screen id | Model | View |
| --- | --- | --- |
| (stack) | `menu_router.gd` (`MenuRouter`: push / pop / replace / clear_to, `ask` + `confirm` / `decline`, signals `changed`, `confirm_needed`, `notice`, `game_started`, `quit_requested`) | `main_menu_view.gd` (`MainMenuView`, host of `scenes/main_menu.tscn`: renders the top screen, the question overlay, the game scene) |
| `main` | `main_menu_model.gd` | `main_screen_view.gd` |
| `new_game` | `new_game_model.gd` | `new_game_view.gd` |
| `load`, `save` | `save_slots_model.gd` | `save_slots_view.gd` |
| `settings` | `settings_model.gd` | `settings_view.gd` |
| `hub` | `hub_model.gd` | `hub_view.gd` |

Rules: models call only rule APIs (`NewGame`, `SaveGame`, `WorldCatalog`, `CurrentGame`, `SettingsFile`) and never touch a Node; actions return `Array[String]` problems (empty = done); a new screen adds a `MenuRouter` id, a `make_model` case and a view case in `MainMenuView.render`. Tests: `scripts/tests/menus_test_suite.gd`.
