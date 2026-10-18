# scripts/progression

System: S7 (party and progression data model). Status: done (M3). Tests: `scripts/tests/progression_test_suite.gd`.

Persistent party snapshot as plain data (`to_dict` / `from_dict` / `validate`, RefCounted only, no scene dependencies).

- `unit_state.gd` (`UnitState`): one member; `total_xp` (level is derived), confirmed `spend`, `learned_skills`, `passives`, `loadout`, `equipment`, `carried`, `pouch`, `hp_lost` (HP missing, 0 = full; save v9). `at_level(base_unit, level)`.
- `party_state.gd` (`PartyState`): roster (any size), `active_ids` (at most `PARTY_MAX` = 4; empty = first four), `stash`.
- `xp_curve.gd` (`XpCurve`): `level_for`, `xp_for_level`, `kill_xp`, `encounter_xp` over `data/progression/xp_curve.json` (defaults plus error list when invalid).
- `level_up.gd` (`LevelUp`): `award`, `award_encounter` (full XP to every active member), `confirm` (permanent spend), `auto`.
- `party_builder.gd` (`PartyBuilder`): `UnitState` to `Combatant` through `ContentFactory.unit` (same numbers as a catalog unit of that level), then passives, learned skills, equipment, loadout (always full HP).
- `heal_rules.gd` (`HealRules`, HEAL1-L): HP between battles. `hp_after` (won: +`RALLY_PCT` 25% of max, a downed hero gets up at that; fled: at least 1 HP), `start_hp` / `current_hp` (`clampi(max_hp - hp_lost, 1, max_hp)`), `rest`, `can_camp` / `make_camp` (`CAMP_COST` 3 supplies, world map nodes only), `on_defeat` (rest place free, else forced rest for 3 supplies, else back to `explore.last_rest`).

Rules: HP carries over between game battles (`docs/specs/HEAL1.md`; previews, sims and the Dojo build at full HP), spend is permanent, no respec. Temporary passives are runtime only and never serialised.
