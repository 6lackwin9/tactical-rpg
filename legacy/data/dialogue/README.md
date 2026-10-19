# data/dialogue

System: S11 (spec `docs/specs/DLG1_L.md`). Status: in progress (schema merged; lines arrive with DLG1-A).

Purpose: lore-free base service lines. Dialogue trees are world data (`data/worlds/<id>/dialogues.json`), shops too (`shops.json`).

`services.json`: envelope `{"kind": "dialogue_services", "version": 1, "entries": [{"id", "text"}]}`. Ids are exactly `greeting`, `offer`, `accept`, `decline`, `turn_in`, `shop`, `farewell` (`DialogueRules.SERVICE_IDS`); a missing id is a warning and the runtime uses `DialogueRules.DEFAULT_LINES`. Tokens: `{npc}`, `{quest}`, `{shop}`, `{gold}` only. Worlds override lines through the manifest `service_lines` (merged along the lineage, nearest wins).

Edit only through `ServiceCatalog.upsert` + `ServiceCatalog.save()` (seed scripts), never raw JSON.
