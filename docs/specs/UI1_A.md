# UI1-A: start party for the playable world

Status: spec (design-brainstorm 2026-10-17; DECISIONS progression-save "UI menus baseline answers (user 2026-10-17)"; awaiting approval)
Depends on: UI1-L commit 1 (`start_party` field and validation); EXP1-A merged (sets `start_location` on ashlands through the same `update_manifest` call path)
Lane: asset
Touches: `data/worlds/ashlands/world.json` (through `WorldCatalog.update_manifest`), seed `data/seeds/ui1_start_party.gd`, `data/seeds/README.md`

## Goal

Make the first campaign world playable from New Game with a fixed four-member party built from the shipped party units (DECISIONS world-campaign "Playable scope": one small campaign first; balance "BAL2 role answers" (7): no new priest or brawler party units, so the four are the existing party units of distinct classes).

## Schema contract

None (UI1-L field).

## Assets

| World | Change | Value |
| --- | --- | --- |
| `ashlands` | `WorldCatalog.update_manifest("ashlands", {"start_party": [...]})` | `["warrior", "ranger", "rogue", "mage"]` |
| `tidewrack` | none (extends ashlands, inherits the list and EXP1's `start_location`, so it is playable too) | - |
| `iron_reaches` | none (no `extends`, no start party: not playable) | - |

Order is the party order (first member = slot 1 of the active four). `dojo_warrior` and `dojo_ranger` are tool units and are not used.

## Acceptance (rule-derived)

1. `WorldCatalog.manifest_errors("ashlands")` and `manifest_errors("tidewrack")` are empty; `WorldCatalog.validate_all()` (or the world equivalent) has no new entry.
2. `WorldCatalog.start_party("tidewrack")` equals the ashlands list (inheritance).
3. `NewGame.create_for_world("ashlands")` returns a valid state with four members at their authored levels; `create_for_world("iron_reaches")` returns the "no start party" error.
4. Seed listed in `data/seeds/README.md`.

## Out of scope

Named world characters as party members (UnitState `lore_ref` stays empty; Mara Voss and Tev Quill are quest NPCs with death variants), new party units, iron_reaches as a start world.

## Open questions

None.
