# scripts/quests

System: S8. Status: in progress (M4-L slice 1, schema). Spec: docs/specs/M4_L.md. See docs/STRUCTURE.md and docs/ROADMAP.md.

Purpose: Quest state machine, flags, rewards granting passives/skills/items.

Built:

- `quest_rules.gd` (`QuestRules`): constants (objective kinds, reward caps, soft caps, `VARIANT_FIELDS`, link keys, loot limits, stored states) and pure validation helpers for templates, world quest fields, variants, links, loot entries and pools, flag conditions (`condition_met`), the stored blob (`validate_blob`), and the soft warnings `link_warnings(world_id)` and `reward_warnings(world_id)`.
- `quest_catalog.gd` (`QuestCatalog`): lore-free templates in `data/quests/quests.json`; same contract as ContentCatalog (`upsert`, `remove`, `validate_entry`, `validate_all`, `warnings_for`, `warnings_all`, `save`, `use_path`; save refused when the file loaded with errors).

Planned (slice 2): `quest_runner.gd` (state machine over `GameState`), `reward_grant.gd`, `QuestRules.resolve`.

Rules: gameplay logic as RefCounted rules engines, data validated through a catalog, one test suite (`scripts/tests/quest_test_suite.gd`).
