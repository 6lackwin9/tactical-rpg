# scripts/quests

System: S8. Status: built (M4-L slices 1 and 2). Spec: docs/specs/M4_L.md. See docs/STRUCTURE.md and docs/ROADMAP.md.

Purpose: Quest state machine, flags, rewards granting passives/skills/items.

- `quest_rules.gd` (`QuestRules`): constants (objective kinds, reward caps, soft caps, `VARIANT_FIELDS`, link keys, loot limits, stored states), validation helpers for templates, world quest fields, variants, links, loot entries and pools, flag conditions (`condition_met`, strict type equality), the stored blob (`validate_blob`), the soft warnings `link_warnings(world_id)` / `reward_warnings(world_id)`, and `resolve(gs, entry)` (pure ordered merge of the applying variants; adds `objectives`, `rewards`, `available`, `applied_variants`; empty `turn_in` = the giver).
- `quest_catalog.gd` (`QuestCatalog`): lore-free templates in `data/quests/quests.json`; same contract as ContentCatalog (`upsert`, `remove`, `validate_entry`, `validate_all`, `warnings_for`, `warnings_all`, `save`, `use_path`; save refused when the file loaded with errors or a template carries a field outside the stored set).
- `quest_runner.gd` (`QuestRunner`): statuses (invalid, completed, blocked, ready, active, unavailable, available, locked), `resolved`, `quests_with`, `current_objective`, `start`, `abandon`, `on_location_reached`, `on_encounter_won`, `set_flag`, `refresh` (flag objectives, ready, auto_start), `offers`, `picks_allowed`, `complete(gs, ref, recipient_id, picks)` (0..`loot_picks` picks, all-or-nothing), `check` (warnings only).
- `reward_grant.gd` (`RewardGrant`): `apply(gs, rewards, recipient_id, loot)`: xp to every active member, items to the stash, gold to `party.gold` (echoed in `granted.gold`, capped at `PartyState.GOLD_MAX`), passives and skills (and picked loot) to one recipient; validates before changing anything; "already owned" is a note. `take(gs, items, gold)` (DLG1-L dialogue `take`): checks every holding first ("Not enough %s."), then removes.

Rules: gameplay logic as RefCounted rules engines, data validated through a catalog. Tests: `scripts/tests/quest_test_suite.gd` (schema), `scripts/tests/quest_runtime_test_suite.gd` (runtime and save).

`RewardGrant.claim_loot(gs, qid)` claims a loot-table entry outside a quest turn-in (exploration chests): refuses unqualified ids and second claims, keeps `loot_claimed` sorted.
