# Design handover

Written 2026-10-17 23:45 UTC by the Design session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`.

## Queue

- Empty: no design task is waiting. CAMP1 is approved and done as a spec; its build rows (CAMP1-A1, CAMP1-A2 for Assets & Audio, CAMP1-B for Balance) are in `docs/TASKS.md`.
- Ideas from CAMP1 that need Tushar before speccing: CAMP1-P (recorded New Game to ending playthrough, needs PLAYTEST2 for a clean replay), QUESTX (quests of `connects` worlds, so `iron_reaches:iron_for_embers` can finish).

## Landed today on `develop`

- CAMP1 (spec): `docs/specs/CAMP1.md` "The Long Dry".
- CAMP1 approval: decision "CAMP1 story (user 2026-10-17)" in `docs/decisions/world-campaign.md` (all four recommendations).

## Notes for the next session

- design-brainstorm on the default model took about 135k tokens and 9 minutes for a 143-line spec; a brief that names the spec files and data folders to read kept it from crawling code.
- The spec's findings F1-F3 (cross-world quests, map-less locations hide NPCs, flag-then-arrive order) are worth checking before any future world or quest spec.
