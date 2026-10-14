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
| `tools/` | fc.sh, balance_sim.gd | S15 | done |
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
