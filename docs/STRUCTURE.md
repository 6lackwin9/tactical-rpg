# Project structure: current and reserved

Builders put code where this file says; if a path is missing from here, ask the planner before creating it. Placeholder folders contain a `README.md` that states purpose, planned files and the data they will use. Update this file when a placeholder is filled.

| Path | Purpose | System | Status |
| --- | --- | --- | --- |
| `scripts/core/specs/` | StatRules, ClassArchetype, Proficiency, Passives, CharacterStats | S2 | done |
| `scripts/core/combat/`, `scripts/core/inventory/` | Combatant, DamageCalculator, equipment, inventory | S1, S3 | done |
| `scripts/combat/{planning,actions,ai,session,status,arena}/` | planner, resolver commands, AI, sessions, statuses | S1, S4 | done |
| `scripts/content/` + `data/catalog/` | catalog store, schema, factory + JSON | S5 | done |
| `scripts/world/` + `data/worlds/` | world catalog/schema/factory + JSON | S6 | done |
| `scripts/ui/{combat,forge,dojo,world,runner,framework}/` | panels and tools | all | done |
| `scripts/tests/` (suite_registry.gd, test_helpers.gd, per-group suites), `scenes/dojo/dojo_assertions.gd` | headless suites; a suite = one file + one line in the registry (R1) | all | done |
| `scripts/tests/weaken_test_suite.gd` | weaken status, zero-power attacks, riposte data models (D1-L) | S12 | done |
| `scripts/tests/progression_test_suite.gd`, `save_test_suite.gd` | progression and save suites (M3) | S7, S13 | done |
| `scripts/tests/ai_skills_test_suite.gd` | AI use of the new skill kinds (AI1) | S4 | done |
| `scripts/tests/bal2_rules_test_suite.gd` | BAL2-L rules: PERC range, unit gear `none`, tiers, trained fists | S2, S3 | done |
| `scripts/tests/skill_unlock_test_suite.gd` | SK2 unlock rule suite (its schema default seed `sk2_schema.gd` is archived, CLN-2) | S2, S12 | done |
| `data/seeds/` | asset seeds of tasks in flight only (`ContentCatalog.upsert` scripts); `README.md` indexes executed seeds by task and merge commit; a seed leaves the folder in the cleanup pass after its task merges (files stay in git history, CLN-2) | S5 | done |
| `scripts/tests/skill_engine_test_suite.gd` | area-attack status, DEFEND taunt/riposte grants (M2C2B-L) | S12 | done |
| `scripts/tests/focus_gear_test_suite.gd` | orb/tome rules, item effects, gear bonuses (C1-L) | S3 | done |
| `tools/` | fc.sh (per-worktree logs), balance_sim.gd + `tools/sim/` (sim_args, sim_runner, sim_variants, sim_observer; preloaded), parse_all.gd, overlap.sh, md_lint.awk, wt_remove.ps1 | S15 | done |
| `docs/TASKS.md`, `docs/specs/` | task queue and per-task specs (logic/asset contracts) | process | done |
| `scripts/combat/status/support_rules.gd`, `scripts/combat/actions/support_command.gd` | heal/buff rules and command (M1) | S12 | done |
| `scripts/tests/combat_extensions_test_suite.gd` | S12-A/S4-A suite (M1) | S12, S4 | done |
| `scripts/combat/status/riposte_rules.gd`, `scripts/tests/riposte_test_suite.gd` | riposte counter rules and suite (M2a) | S12 | done |
| `scripts/core/specs/gear_budget.gd`, `scripts/tests/gear_budget_test_suite.gd` | GearBudget weights, loadout band, soft warnings; suite (M2d) | S3 | done |
| `scripts/tests/skill_family_test_suite.gd` | skill families (family/tier/basic), planner tier grouping, active loadout (M2c-2a1) | S12 | done |
| `scripts/combat/actions/disengage_command.gd`, `scripts/tests/engine_pieces_test_suite.gd` | disengage jump command; cleanse / disengage / SWEEP / unarmed suite (M2c-2a2) | S12 | done |
| `scripts/combat/status/status_registry.gd`, `scripts/tests/status_registry_test_suite.gd`, `scripts/tests/foundation_test_suite.gd` | single status table with stacking rules; suites for it and for slice A (skill potency hook, `unarmed` token, derived weapon hands) | S12 | done |
| `scripts/content/{skill_kinds,skill_ranks,skill_text,asset_scan}.gd`, `tools/safe_delete.gd` | ranked skills, kind sections, description tokens, reference scan / safe delete (SKILL1; the migration seed `skill1_ranks.gd` is archived, CLN-2) | S5 | done |
| `scripts/content/validators/` | per-kind validation rules (SkillValidator, ItemValidator, ClassUnitValidator); ContentCatalog.validate_entry / warnings_for dispatch to them (CLN-9) | S5 | done |
| `scripts/ui/forge/{skill_form_model,skill_filter,skill_ranks_panel}.gd` | Forge skill tab logic and ranks panel (SKILL1) | S5 | done |
| `scripts/tests/{skill_ranks,skill_text,skill_model,origin,skill_form_model}_test_suite.gd`, `skill_test_kit.gd` | SKILL1 suites and shared fixtures | S5 | done |
| `scripts/progression/` | XpCurve, UnitState, PartyState, LevelUp, PartyBuilder (M3) | S7 | done |
| `scripts/quests/` | quest state machine, flags, rewards (grants passives/skills/items) | S8 | placeholder |
| `scripts/exploration/` | region maps, location interactions, encounter triggers | S9 | placeholder |
| `scripts/travel/` | world map, routes, travel events | S10 | placeholder |
| `scripts/dialogue/` | dialogue trees, speakers, shops, reputation | S11 | placeholder |
| `scripts/game_state/` | GameState, SaveGame, SaveMigrations, NewGame, JsonUtil (M3) | S13 | done |
| `scripts/ui/menus/` | main menu, party screen, journal | S7, S13 | placeholder |
| `data/progression/` | xp_curve.json (M3) | S7 | done |
| `data/quests/` | quest definitions (lore-free templates) | S8 | placeholder |
| `data/dialogue/` | dialogue trees | S11 | placeholder |
| `assets/` | audio, fonts, extra art | S14 | placeholder |

Rules: gameplay logic in `scripts/**` as `RefCounted` rules engines; scenes only render; data in `data/**` validated through a catalog; one test suite per system in `scripts/tests/` registered with one `results.append` line in `SuiteRegistry.run_all` (`scripts/tests/suite_registry.gd`).

## Code conventions (kept from `.context/AGENTS.md`, CLN-4)

Strict typing and the commit gate are in `CLAUDE.md` and `.agents/rules/godot_workflow.md`; these naming and syntax rules were only in the removed `.context/AGENTS.md`:

- Names: files and folders `snake_case`; `class_name` and types `PascalCase`; functions and variables `snake_case`; constants and enum values `UPPER_SNAKE_CASE`; private helpers start with one underscore.
- Register domain scripts with `class_name`; use `preload()` only to break a circular class dependency.
- Godot 4 syntax: `super()`, `signal.connect(callable)` (no string `connect`), typed arrays `Array[T]`; integer maths with `maxi`, `mini`, `clampi` (no float round trips for stats).
- Commands follow plan then resolve: planning queues commands and never changes state; resolution runs them in the resolver's deterministic order. Game logic never depends on animation timing or the scene tree.
