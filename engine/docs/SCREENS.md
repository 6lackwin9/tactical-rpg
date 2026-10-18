# Screens to replicate

Goal (user, 2026-10-19): replicate every active Godot screen with its placeholder logic, function over form, so the
engine has a working reference for each screen type before the transition. Look and feel come after
(ENGINE_SPEC section 11).

Reference shots: `engine/docs/screens/` (1280x720, captured from the Godot build by
`engine/parity/godot/screen_tour.gd` on a sandbox save folder; re-run it after Godot UI changes).

## Player screens

| # | Screen | Godot model / view | Systems behind it | Engine status |
| :--- | :--- | :--- | :--- | :--- |
| 00 | Main menu | `ui/menus/main_menu_model.gd`, `main_screen_view.gd` | router, save peek | to do |
| 01 | New game (world cards) | `new_game_model.gd`, `new_game_view.gd` | NewGame, WorldCatalog, PartyBuilder | to do |
| 02, 17 | Load / save slots | `save_slots_model.gd`, `save_slots_view.gd` | SaveGame, migrations | to do |
| 03-06 | Settings (Sound, Display, Gameplay, Controls) | `settings_model.gd`, `settings_view.gd`, `display_settings.gd`, `controls_list.gd` | SettingsFile | to do |
| 07 | Region map (exploration) | `ui/overworld/region_map_view.gd` | ExploreRunner, MapRules, MapCatalog, encounters | to do |
| 08 | Dialogue (bottom dock) | `ui/overworld/dialogue_panel.gd` | DialogueRunner, DialogueRules, RepRules, quests | to do |
| 09 | Shop | `ui/overworld/shop_panel.gd` | ShopRules, ServiceCatalog | to do |
| 10 | World map (travel) | `ui/overworld/world_map_view.gd` | TravelRunner, TravelRules, routes, road fights | to do |
| 11 | Game menu (hub) | `hub_model.gd`, `hub_view.gd` | router | to do |
| 12-15 | Character: Stats, Level up, Skills, Gear | `party_model.gd`, `character_view.gd`, `stats_model.gd`, `level_up_*`, `loadout_*`, `gear_*` | PartyState, UnitState, LevelUp, XpCurve, gear model | to do |
| 16 | Quest journal | `journal_model.gd`, `journal_view.gd` | QuestRunner | to do |
| 18 | Battle results | `battle_results_model.gd`, `battle_results_view.gd` | BattleBridge, XP award, purse, battle report | to do |
| 23 | Battle | `ui/combat/*`, `scenes/combat_arena.gd` | combat core | done (aegis-gui), parity-checked |

## Dev tools (dev-only, never in the real game)

| # | Screen | Godot | Engine status |
| :--- | :--- | :--- | :--- |
| 19 | Content Forge (catalog viewer) | `ui/forge/*` | to do |
| 20 | World Forge (world viewer) | `ui/world/*` | to do |
| 21 | Gear & Skill Dojo | `ui/dojo/*` | to do |
| 22 | Scene Runner | `ui/runner/*` | to do (becomes the engine's launcher) |

## Order

1. Data: read `data/` directly (item families, world catalogs with inheritance), checked against Godot exports.
2. Game state: GameState, NewGame, SaveGame (Godot save files load unchanged), party and progression rules.
3. Game loop rules: exploration, travel, dialogue, shop, quests, rest and camp, world battles, results; checked with
   scripted action sequences replayed in both engines (state compared after every action).
4. Screens on a shared widget kit (frame, list, tabs, scroll, slider, checkbox, dropdown, popup), menus first, then the
   play screens, then the dev tools.
