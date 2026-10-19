# data/quests

System: S8. Status: schema and runtime built (M4-L); sample templates from M4-A. Specs: docs/specs/M4_L.md, docs/specs/M4_A.md.

Purpose: Lore-free quest templates (JSON).

`quests.json`: envelope `{"kind": "quest_templates", "version": 1, "entries": [...]}`. Each entry is `{id, name, description, objectives: [{"kind": reach_location|win_encounter|flag}], rewards: {xp, items, passives, skills, gold}, tags}` (`gold` 0..5000, soft cap 200; DLG1-L).

Edit only through `QuestCatalog.upsert` (seed scripts), never raw JSON. World quests bind a template through their `template` and `steps` fields; special rewards come from the world loot table (`data/worlds/<id>/loot.json`).
