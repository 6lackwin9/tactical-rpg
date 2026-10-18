# Design handover

Written 2026-10-18 00:25 UTC by the Design session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`.

## Queue

- Empty: no design task is waiting.
- AUTO1 (battle autoplay) is approved and done as a spec: `docs/specs/AUTO1.md`, decision "AUTO1 battle autoplay (user 2026-10-18)" in `docs/decisions/combat.md`, build row AUTO1-L (logic, Gameplay) in `docs/TASKS.md`. It shares `scripts/ui/combat/*` with UIR-5 and UI/UX's combat layout mockups, so Gameplay should land after them or merge carefully; the layout decides where the dropdowns sit, the spec decides what they do.
- Ideas from CAMP1 that need Tushar before speccing: CAMP1-P (recorded New Game to ending playthrough, needs PLAYTEST2 for a clean replay), QUESTX (quests of `connects` worlds, so `iron_reaches:iron_for_embers` can finish).

## Notes for the next session

- The player AI drafting already existed (`PlanPresets` + `TacticalAI`, the Draft menu); AUTO1 is the loop and the dropdown UI around it. Check for an existing preset or AI hook before speccing new AI features.
- A short spec written directly by the thread (no design-brainstorm run) was enough for a UI-loop feature; keep brainstorm runs for new systems.
