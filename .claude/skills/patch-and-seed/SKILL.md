---
name: patch-and-seed
description: Safe editing recipe for the Aegis project. Use when changing engine code in several places or changing game data (classes, units, items, skills, passives, encounters, terrain, maps, quests, services, world entries).
---

# Patch and seed

## Code edits

- Use the Edit tool for exact single/multi-site changes; Write for new files. Do not use sed line ranges, and do not write Rust or JSON through shell heredocs whose quoting can eat characters (backticks and `$` in `node -e "..."` / `bash -c "..."` strings are expanded by the shell).
- If many sites need the same change, a small patch script is fine (Node in the scratchpad: read the file, replace each exact old text, print `MISSING:` and exit non-zero when one is not found, write once). Verify with `git diff --stat` and `tools/fc.sh build`.
- After edits: `tools/fc.sh build`, then `tools/fc.sh test <filter>` for what you touched (skill `engine-checklist`).

## Data edits (catalog, game data, worlds)

- Shipped data changes go through the engine's writer, never by hand: `aegis content upsert | remove`, or a seed `games/emberwake/data/seeds/<task>.json` run with `aegis content apply` (left in the repo for the planner to re-run after a rebase; it is idempotent because an upsert replaces the entry with the same id). From the repo root: `cargo run -q -p emberwake_host -- content <...>`; anywhere: `tools/fc.sh content <...>`.
- A seed is `{"steps": [...]}`, one step per entry in dependency order (item families, items, skills, passives, classes, units, encounters; worlds after the catalog entries they use): `{"op": "upsert", "kind": "units", "entry": {...}}`, `{"op": "upsert", "kind": "lore", "world": "ashlands", "entry": {...}}`, `{"op": "remove", "kind": "units", "id": "..."}`; optional `"replace": "<old id>"` renames. Apply stops at the first failing step; earlier steps stay written, so fix and re-run.
- Entries are given in their stored shape, as the data files hold them: a skill is one ranked entry (base fields plus `ranks` modifiers; ids, names and tiers of the ranks are derived), an item is `family` plus `modifiers` and `tiers` (the flat numbers are derived from the family). Print the current entry first (`games/emberwake/data/catalog/<kind>.json`) and change only what the task asks.
- Kinds: `classes`, `units`, `skills`, `items`, `item_families`, `passives`, `encounters`, `terrain`, `maps`, `quests`, `services`, and with `world=<id>` every world kind (`regions`, `locations`, `factions`, `characters`, `items`, `disciplines`, `quests`, `lore`, `encounters`, `loot`, `dialogues`, `shops`, `routes`).
- The writer validates (id, name, enum options, references) and keeps a change only when it adds no blocking problem: every content rule of the catalog (the Forge's checks: skill rules, class budgets, item modifiers and tiers, encounter phases ...) and the shape rules of world entries; a world entry may name one that does not exist yet (add it next), and `content check` lists what is still unresolved. Otherwise it puts the file back and prints why. Read the error, fix the data, do not loosen a rule. Finish with `tools/fc.sh content check` (`data ok`).
- World manifests (`world.json`) go through `aegis content world <id> <json>` or the seed step `{"op": "world", "id", "fields"}` (WORLDW1), never by hand. For a new world, first add its id to game.json `worlds` (`aegis content manifest`), then the world step, then the `world=<id>` upserts. (The asset manifest is retired: art and sound are the files under the asset layers, ASSETORG2.)

## Tests after data changes

- Numbers in tests derive from rules; print real values (`aegis battle <encounter> seed=N log`, `aegis serve` requests) before changing an expectation. The golden suites read frozen fixtures, so a data edit in `games/emberwake/data/` never changes them; `tests/store.rs` checks the live `games/emberwake/data/` loads and its references resolve.
