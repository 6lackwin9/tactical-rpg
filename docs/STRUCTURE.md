# Project structure: current and reserved

Builders put code where this file says; if a path is missing from here, ask the planner before creating it. Placeholder folders contain a `README.md` that states purpose, planned files and the data they will use. Update this file when a placeholder is filled.

| Path | Purpose | System | Status |
| --- | --- | --- | --- |
| `scripts/core/specs/` | StatRules, ClassArchetype, Proficiency, Passives, CharacterStats | S2 | done |
| `scripts/core/specs/{item_families,item_modifiers,item_text}.gd` | item family templates, modifier resolve/collapse, description tokens (ITEM1) | S3, S5 | done |
| `scripts/content/class_forge_model.gd` | ClassForgeModel: class tab view model (layout, locks, path helpers, panels, grouped pickers, unit AI visibility) (CLASS1) | S5 | done |
| `scripts/core/combat/`, `scripts/core/inventory/` | Combatant, DamageCalculator, equipment, inventory | S1, S3 | done |
| `scripts/combat/{planning,actions,ai,session,status,arena}/` | planner, resolver commands, AI, sessions, statuses | S1, S4 | done |
| `scripts/content/` + `data/catalog/` | catalog store, schema, factory + JSON | S5 | done |
| `scripts/world/` + `data/worlds/` | world catalog/schema/factory + JSON | S6 | done |
| `scripts/ui/{combat,forge,dojo,world,runner,framework}/` | panels and tools | all | done |
| `scripts/tests/` (suite_registry.gd, test_helpers.gd, per-group suites), `scenes/dojo/dojo_assertions.gd` (+ `dojo_session_assertions.gd`, `dojo_ui_assertions.gd`) | headless suites; shared fixtures of a suite family live in a `*_kit.gd` beside it (CLN-12); a suite = one file + one line in the registry (R1) | all | done |
| `scripts/tests/status_expansion_test_suite.gd` | registry magnitude ranges, the six S12S statuses, magnitude validation, status sources (S12S-L) | S12 | done |
| `scripts/tests/status_expansion_effects_test_suite.gd` | slow tiles, expose/fortify armor rating, round-end order and logs, poison stacking, granted magnitudes and sources, Forge magnitude range (S12S-L commit 2) | S12 | done |
| `scripts/tests/status_expansion_ai_test_suite.gd` | AI use of the new statuses and sim metrics regen_received / dot_dealt on a fixture fight (S12S-L commit 3) | S12, S4, S15 | done |
| `scripts/tests/weaken_test_suite.gd` | weaken status, zero-power attacks, riposte data models (D1-L) | S12 | done |
| `scripts/tests/progression_test_suite.gd`, `save_test_suite.gd` | progression and save suites (M3) | S7, S13 | done |
| `scripts/tests/ai_skills_test_suite.gd`, `ai_skills_tactics_test_suite.gd`, `ai_skills_kit.gd` | AI use of the new skill kinds (AI1) | S4 | done |
| `scripts/tests/bal2_rules_test_suite.gd` | BAL2-L rules: PERC range, unit gear `none`, tiers, trained fists | S2, S3 | done |
| `scripts/tests/bal2_sim_test_suite.gd` | balance sim args, targets, role map/loadouts, sweep counts, collector self-checks (BAL2-L 1b) | S15 | done |
| `scripts/tests/{item_families,item_modifiers}_test_suite.gd` | ITEM1 suites (TestHelpers.fork_migrated) | S3, S5 | done |
| `scripts/tests/item_tiers_test_suite.gd` | item tier criteria, bonuses, evaluation order, shipped loadouts unchanged (ITEM1 commit 2) | S3 | done |
| `scripts/tests/item_form_model_test_suite.gd` | item and family Forge model and tabs (ITEM1 commit 3) | S5 | done |
| `scripts/tests/asset_pipeline_test_suite.gd` | asset manifest, key resolution, team overlay, skill and status icons, safe remove (S14-L1) | S14 | done |
| `scripts/tests/{class_path,class_form_model}_test_suite.gd` | CLASS1 suites (path dict, converter, rules, warnings, locks, seed; model and Forge class tab) | S5 | done |
| `scripts/tests/quest_test_suite.gd`, `quest_runtime_test_suite.gd` | quest schema; quest runtime and save v2 (M4) | S8, S13 | done |
| `scripts/tests/exploration_test_suite.gd` | region map schema, bindings, markers, start_location, ref scans (EXP1) | S9 | done |
| `scripts/tests/dialogue_test_suite.gd` | dialogue trees, shops, service lines, gone-NPC lint, gold fields, ref scans (DLG1) | S11 | done |
| `scripts/tests/plan_claims_test_suite.gd` | preset plans always confirmable: approach tile claims, tie order, party redraft (PLAN1) | S1, S4 | done |
| `scripts/tests/skill_unlock_test_suite.gd` | SK2 unlock rule suite (its schema default seed `sk2_schema.gd` is archived, CLN-2) | S2, S12 | done |
| `data/seeds/` | asset seeds of tasks in flight only (`ContentCatalog.upsert` scripts); `README.md` indexes executed seeds by task and merge commit; a seed leaves the folder in the cleanup pass after its task merges (files stay in git history, CLN-2) | S5 | done |
| `scripts/tests/skill_engine_test_suite.gd` | area-attack status, DEFEND taunt/riposte grants (M2C2B-L) | S12 | done |
| `scripts/tests/focus_gear_test_suite.gd`, `focus_gear_budget_test_suite.gd` | orb/tome rules, item effects, gear bonuses (C1-L) | S3 | done |
| `tools/` | fc.sh (per-worktree logs), balance_sim.gd + `tools/sim/` (sim_args, sim_runner, sim_variants, sim_observer, sim_metrics, sim_targets, sim_roles, sim_scenarios, sim_party, sim_role_mode; preloaded), parse_all.gd, overlap.sh, md_lint.awk, wt_remove.ps1, export_placeholder.gd | S15 | done |
| `docs/TASKS.md`, `docs/specs/` | task queue and per-task specs (logic/asset contracts) | process | done |
| `docs/workflow/` | desktop vs cloud workflow: `README.md` (differences, switching), `desktop.md`, `cloud.md` | process | done |
| `assets/fonts/`, `scripts/ui/runner/web_fonts.gd` | Web-only fallback fonts for emoji and symbol glyphs (`WebFonts.install`, called by the Scene Runner); sources and licenses in `assets/fonts/README.md` | process | done |
| `export_presets.cfg`, `.github/workflows/web.yml` | Web (HTML5) export preset (single-threaded) and the workflow that builds it on every PR and deploys it to GitHub Pages from `main` (`docs/workflow/web.md`) | process | done |
| `.github/workflows/gate.yml` | GitHub Actions: fast gate (`tools/fc.sh gate`) and `tools/fc.sh docs` on every PR and push to `main` | process | done |
| `scripts/combat/status/support_rules.gd`, `scripts/combat/actions/support_command.gd` | heal/buff rules and command (M1) | S12 | done |
| `scripts/tests/combat_extensions_test_suite.gd`, `combat_ext_attacks_test_suite.gd`, `combat_ext_ai_test_suite.gd`, `combat_ext_kit.gd` | S12-A/S4-A suites (M1) | S12, S4 | done |
| `scripts/combat/status/riposte_rules.gd`, `scripts/tests/riposte_test_suite.gd` | riposte counter rules and suite (M2a) | S12 | done |
| `scripts/core/specs/gear_budget.gd`, `scripts/tests/gear_budget_test_suite.gd` | GearBudget weights, loadout band, soft warnings; suite (M2d) | S3 | done |
| `scripts/tests/skill_family_test_suite.gd` | skill families (family/tier/basic), planner tier grouping, active loadout (M2c-2a1) | S12 | done |
| `scripts/combat/actions/disengage_command.gd`, `scripts/tests/engine_pieces_test_suite.gd`, `unarmed_test_suite.gd` | disengage jump command; cleanse / disengage / SWEEP suite, unarmed suite (M2c-2a2) | S12 | done |
| `scripts/combat/status/status_registry.gd`, `scripts/tests/status_registry_test_suite.gd`, `scripts/tests/foundation_test_suite.gd` | single status table with stacking rules; suites for it and for slice A (skill potency hook, `unarmed` token, derived weapon hands) | S12 | done |
| `scripts/content/{skill_kinds,skill_ranks,skill_text,asset_scan}.gd`, `tools/safe_delete.gd` | ranked skills, kind sections, description tokens, reference scan / safe delete (SKILL1; the migration seed `skill1_ranks.gd` is archived, CLN-2) | S5 | done |
| `scripts/content/validators/` | per-kind validation rules (SkillValidator, ItemValidator, ClassUnitValidator); ContentCatalog.validate_entry / warnings_for dispatch to them (CLN-9) | S5 | done |
| `scripts/content/{skill,item,class_unit}_fields.gd` | per-kind field tables of ContentSchema (CLN-9b) | S5 | done |
| `scripts/ui/forge/{skill_form_model,skill_filter,skill_ranks_panel}.gd` | Forge skill tab logic and ranks panel (SKILL1) | S5 | done |
| `scripts/ui/forge/{item_form_model,item_tables}.gd` | Forge item and item-family tabs: ItemFormModel (sections, locks, options, limits, problems, modifier and tier tables, impact, filters) and ItemTables (widgets) (ITEM1) | S5 | done |
| `scripts/art/` | SvgRaster, UnitArt, ItemArt, TileArt, SkillArt, StatusArt (generated placeholders); AssetRegistry (key to manifest file, template, fallback; team overlay) and AssetCatalog (manifest store) (S14-L1) | S14 | done |
| `data/assets/` | manifest.json: art and sound file overrides and aliases (AssetCatalog; empty until S14-A1) | S14 | in progress |
| `scripts/tests/{skill_ranks,skill_text,skill_model,origin,skill_form_model}_test_suite.gd` | SKILL1 suites (shared fixtures now in `test_helpers.gd`, CLN-8) | S5 | done |
| `scripts/progression/` | XpCurve, UnitState, PartyState, LevelUp, PartyBuilder (M3) | S7 | done |
| `scripts/quests/` | quest_rules.gd (validation, `resolve` variants), quest_catalog.gd (templates), quest_runner.gd (status, events, offers, complete), reward_grant.gd (xp to all active, items to stash, passive/skill to one recipient, loot claims) | S8 | done |
| `scripts/exploration/` | map_rules.gd (MapRules: templates, bindings, markers, start_location), map_catalog.gd (MapCatalog); explore_runner.gd (ExploreRunner), battle_bridge.gd (BattleBridge), game_encounter.gd (GameEncounter) | S9 | done |
| `scripts/travel/` | world map, routes, travel events | S10 | placeholder |
| `scripts/dialogue/` | dialogue_rules.gd (DialogueRules), shop_rules.gd (ShopRules), service_catalog.gd (ServiceCatalog); dialogue_runner.gd (DialogueRunner); panels in DLG1-L commit 3 | S11 | in progress |
| `scripts/game_state/` | GameState, SaveGame, SaveMigrations, NewGame, JsonUtil (M3; save v2 with structured quests, `loot_claimed`, built-in 1->2 migration, M4) | S13 | done |
| `scripts/audio/` | SoundEvents (sfx keys, preset map), SoundBank (key lookup through AssetRegistry, warn once), AudioDirector (voices on the SFX/UI buses, presenter hook, bus volumes), PlaceholderSfx (generated blips for `tools/gen_placeholder_sfx.gd`) (S14-L3) | S14 | done |
| `scripts/game_state/settings_file.gd` | SettingsFile: `user://settings.cfg` key table (audio keys S14-L3; UI1-L adds its keys), never in a save slot | S14, UI1 | in progress |
| `scripts/game_state/game_rolls.gd` | GameRolls: per-save deterministic random streams (explore, travel) | S9, S10 | done |
| `scenes/overworld/`, `scripts/ui/overworld/` | debug region map (RegionMapView, EXP1), battle screen into the combat arena (BattleScreen, BATTLE1), dialogue and shop panels (DialoguePanel, ShopPanel, DLG1); later the world map | S9, S10, S11 | in progress |
| `scripts/ui/menus/` | main menu, new game, save slots, settings, hub (UI1-L); party screen, journal (UI2, UI3) | S7, S13 | in use |
| `data/progression/` | xp_curve.json (M3) | S7 | done |
| `data/quests/` | quests.json (lore-free quest templates; empty until M4-A) | S8 | in progress |
| `data/maps/` | maps.json (lore-free region map templates, MapCatalog; EXP1) | S9 | in progress |
| `data/dialogue/` | services.json: lore-free base service lines (dialogue trees are world data in `data/worlds/<id>/dialogues.json`, shops in `shops.json`) | S11 | in progress |
| `assets/art/{unit,item,tile,skill,status,ui}/`, `assets/audio/{sfx,music}/` | file overrides by folder pattern `<key>_<variant>.png/svg`, `<key>.ogg/wav` (S14) | S14 | in progress |

Rules: gameplay logic in `scripts/**` as `RefCounted` rules engines; scenes only render; data in `data/**` validated through a catalog; one test suite per system in `scripts/tests/` registered with one `results.append` line in `SuiteRegistry.run_all` (`scripts/tests/suite_registry.gd`).

## Code conventions (kept from `.context/AGENTS.md`, CLN-4)

Strict typing and the commit gate are in `CLAUDE.md` and `.agents/rules/godot_workflow.md`; these naming and syntax rules were only in the removed `.context/AGENTS.md`:

- Names: files and folders `snake_case`; `class_name` and types `PascalCase`; functions and variables `snake_case`; constants and enum values `UPPER_SNAKE_CASE`; private helpers start with one underscore.
- Register domain scripts with `class_name`; use `preload()` only to break a circular class dependency.
- Godot 4 syntax: `super()`, `signal.connect(callable)` (no string `connect`), typed arrays `Array[T]`; integer maths with `maxi`, `mini`, `clampi` (no float round trips for stats).
- Commands follow plan then resolve: planning queues commands and never changes state; resolution runs them in the resolver's deterministic order. Game logic never depends on animation timing or the scene tree.
