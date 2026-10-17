# Gameplay handover

Written 2026-10-17 20:55 UTC by the outgoing Gameplay session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`.

## Queue (in order)

1. SAVE-COMPAT (approved; Tushar's card "Backups + save test", 20:21; decision "Save safety in testing" in `docs/decisions/workflow.md`): commit sample saves for each save version (now 1-5; TRAV1-L made it 5), built from a played state, and a gate step that loads each through `SaveGame.read` against the current catalog and worlds and fails on any error. The `run.bat` backups half is already on develop (85eae02).
2. S12C-A (approved): 3 combo families (feint, crushing_blow, opportunist), asset lane, spec `docs/specs/S12C_A.md`; the S12C-L engine is on develop, so combos do nothing in play until this lands.
3. TRAV1-A (approved): Ashlands roads and Iron Reaches south road, two world_map exit markers; asset lane on the TRAV1-L routes schema (on develop).
4. REP1-L then REP1-A (approved): reputation; REP1-L runs after TRAV1-L because both edit `world_schema` and `world_catalog`.

## Landed today on `develop` (this session)

S14-FIX, REVIEW-F1, S12T-L, S12T-A, S12C-L, TRAV1-L. The REVIEW-F3 remainder and F2 went to UI/UX; F4 and F5 to Platform.

## Waiting on the user

- Nothing open. Answered: S12T-L rule 8c "Leave as is" (AI target ranking ignores cover; S12C-L rule 6c ranking half also not built); recorded in `docs/decisions/combat.md`.

## Follow-ups (not yet task rows; ask before building)

- `ExploreRunner.enter` sets `world_id` to the entered location's own world, so a child world (e.g. Tidewrack) that crosses to a connected world and back ends up in the parent world (pre-existing EXP1 behaviour, surfaced by TRAV1-L).
- Accepted lows: AI combo order ignores statuses consumed earlier in the same plan; exploit consume also runs on a fully absorbed hit; `RegionMapView._exit_tree` clears busy on a plain `remove_child`.
- The `export_to` rollback path (second rename fails) is untested: the sandbox runs as root.

## Notes for the next session

- Economy mode: logic-builder on sonnet for fixes, opus for new engine systems, medium effort; change-reviewer low; asset-builder sonnet low. Resume the same builder between a task's commits with SendMessage.
- Have the builder stop after each spec commit so the thread commits and pushes; the thread writes README, ROADMAP, STRUCTURE, decisions readings and TASKS row removal itself.
- `godot --headless --import` fixes a stale class cache after a big develop merge; re-run the gate after merging develop if it brought code.
- Branches to delete at the next cleanup (all landed): `claude/gameplay-mhp9aq`, `claude/REVIEW-F1-save-safety`, `claude/S12T-L-terrain`, `claude/S12T-A-terrain-rows`, `claude/S12C-L-combos`, `claude/TRAV1-L-travel`, `claude/gameplay-handover`.
