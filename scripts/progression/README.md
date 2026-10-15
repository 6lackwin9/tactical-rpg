# scripts/progression

System: S7 (party and progression data model). Status: done (M3). Tests: `scripts/tests/progression_test_suite.gd`.

Persistent party snapshot as plain data (`to_dict` / `from_dict` / `validate`, RefCounted only, no scene dependencies).

- `unit_state.gd` (`UnitState`): one member; `total_xp` (level is derived), confirmed `spend`, `learned_skills`, `passives`, `loadout`, `equipment`, `carried`. `at_level(base_unit, level)`.
- `party_state.gd` (`PartyState`): roster (any size), `active_ids` (at most `PARTY_MAX` = 4; empty = first four), `stash`.
- `xp_curve.gd` (`XpCurve`): `level_for`, `xp_for_level`, `kill_xp`, `encounter_xp` over `data/progression/xp_curve.json` (defaults plus error list when invalid).
- `level_up.gd` (`LevelUp`): `award`, `award_encounter` (full XP to every active member), `confirm` (permanent spend), `auto`.
- `party_builder.gd` (`PartyBuilder`): `UnitState` to `Combatant` through `ContentFactory.unit` (same numbers as a catalog unit of that level), then passives, learned skills, equipment, loadout.

Rules: full heal after battles, spend is permanent, no respec. Temporary passives are runtime only and never serialised.
