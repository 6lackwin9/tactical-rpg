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
| `scripts/tests/`, `scenes/dojo/dojo_assertions.gd` | headless suites | all | done |
| `tools/` | fc.sh (per-worktree logs), balance_sim.gd, parse_all.gd, overlap.sh | S15 | done |
| `docs/TASKS.md`, `docs/specs/` | task queue and per-task specs (logic/asset contracts) | process | done |
| `scripts/combat/status/support_rules.gd`, `scripts/combat/actions/support_command.gd` | heal/buff rules and command (M1) | S12 | done |
| `scripts/tests/combat_extensions_test_suite.gd` | S12-A/S4-A suite (M1) | S12, S4 | done |
| `scripts/combat/status/riposte_rules.gd`, `scripts/tests/riposte_test_suite.gd` | riposte counter rules and suite (M2a) | S12 | done |
| `scripts/core/specs/gear_budget.gd`, `scripts/tests/gear_budget_test_suite.gd` | GearBudget weights, loadout band, soft warnings; suite (M2d) | S3 | done |
| `scripts/tests/skill_family_test_suite.gd` | skill families (family/tier/basic), planner tier grouping, active loadout (M2c-2a1) | S12 | done |
| `scripts/combat/actions/disengage_command.gd`, `scripts/tests/engine_pieces_test_suite.gd` | disengage jump command; cleanse / disengage / SWEEP / unarmed suite (M2c-2a2) | S12 | done |
| `scripts/combat/status/status_registry.gd`, `scripts/tests/status_registry_test_suite.gd`, `scripts/tests/foundation_test_suite.gd` | single status table with stacking rules; suites for it and for slice A (skill potency hook, `unarmed` token, derived weapon hands) | S12 | done |
| `scripts/progression/` | persistent unit state, XP, level-up flow, spend allocation | S7 | placeholder |
| `scripts/quests/` | quest state machine, flags, rewards (grants passives/skills/items) | S8 | placeholder |
| `scripts/exploration/` | region maps, location interactions, encounter triggers | S9 | placeholder |
| `scripts/travel/` | world map, routes, travel events | S10 | placeholder |
| `scripts/dialogue/` | dialogue trees, speakers, shops, reputation | S11 | placeholder |
| `scripts/game_state/` | save/load, new game, global flags, session-wide state | S13 | placeholder |
| `scripts/ui/menus/` | main menu, party screen, journal | S7, S13 | placeholder |
| `data/progression/` | XP curves, level rewards | S7 | placeholder |
| `data/quests/` | quest definitions (lore-free templates) | S8 | placeholder |
| `data/dialogue/` | dialogue trees | S11 | placeholder |
| `assets/` | audio, fonts, extra art | S14 | placeholder |

Rules: gameplay logic in `scripts/**` as `RefCounted` rules engines; scenes only render; data in `data/**` validated through a catalog; one test suite per system in `scripts/tests/` registered in `ArenaTestSuite.run_all_tests`.
