# Gameplay handover

Written 2026-10-17 22:12 UTC by the outgoing Gameplay session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`.

## Queue

- Empty: no approved Gameplay logic task is left in `docs/TASKS.md`. Data tasks (S12C-A, TRAV1-A, REP1-A, all landed) belong to the Assets & Audio thread now.
- Candidates that need Tushar first: TRAV2 supplies (idea; its "parked until TRAV1 merges" condition is now met), CAMP1 story spec (design, G4), PLAYTEST2 (idea).

## Landed today on `develop` (this session)

SAVE-COMPAT (1f74b50), REP1-L schema (feee36a), REP1-L behaviour (0717392).

## Waiting on the user

- Nothing open.

## Follow-ups (not yet task rows; ask before building)

- `ExploreRunner.enter` sets `world_id` to the entered location's own world, so a child world (e.g. Tidewrack) that crosses to a connected world and back ends up in the parent world (pre-existing EXP1 behaviour, surfaced by TRAV1-L).
- REP1-L: an `op` inside `complete_flags` / `sets_flags` is accepted and ignored (could be an error); World Forge Validate all status line says "All N worlds valid" without counting warnings; v1 save fixture uses the structured quest blob (the real v1 quest format is not exercised).
- Accepted lows: AI combo order ignores statuses consumed earlier in the same plan; exploit consume also runs on a fully absorbed hit; `RegionMapView._exit_tree` clears busy on a plain `remove_child`.
- The `export_to` rollback path (second rename fails) is untested: the sandbox runs as root.

## Notes for the next session

- Economy mode: logic-builder on sonnet for fixes and tests, opus for new engine systems, medium effort; change-reviewer sonnet low. Resume the same builder between a task's commits with SendMessage; split landings worked well (schema first unblocked REP1-A within the hour).
- When merging `develop` back into the task branch after landing a split commit 1, git re-applies the squash and can duplicate hunks: check `git diff <pre-merge> HEAD` and restore the branch tree if develop brought nothing else.
- Data landing on `develop` between gate and land: re-run `tools/fc.sh arena` after merging it.
- Branches to delete at the next cleanup: `claude/gameplay-11ptle` once this handover is on develop.
