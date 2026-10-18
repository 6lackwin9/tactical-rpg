# Gameplay handover

Written 2026-10-18 01:40 UTC by the outgoing Gameplay session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`.

## Queue

- Empty: no approved Gameplay task is left in `docs/TASKS.md`. AUTO1-L stays `in-progress` until UI/UX lands COMBAT-UI2-L3 (the controls) against the `CombatAutoplay` API; UI/UX carries the AUTO1-L `Done:` line.
- Candidates that need Tushar first: PLAYTEST2 (idea). CAMP1 data landed through Assets & Audio.

## Landed on `develop` (this session)

TRAV2-L schema (ba79c0d), TRAV2-L behaviour (0cfe7f5), AUTO1-L logic (f09c933, API in its commit body), POI-FIX region map hint, notes and click to interact (e143307).

## Waiting on the user

- Card in the Gameplay thread: "When should autoplay stop for a hero at low HP?" First drop (recommended, built) / Every round. If he picks Every round: in `CombatAutoplay.safety_reason` drop the "above the line at round start" condition and update `_test_safety_crossing` and the decision row "AUTO1 logic readings" in `docs/decisions/combat.md`.

## Follow-ups (not yet task rows; ask before building)

- `ExploreRunner.enter` sets `world_id` to the entered location's own world, so a child world (e.g. Tidewrack) that crosses to a connected world and back ends up in the parent world (pre-existing EXP1 behaviour, surfaced by TRAV1-L).
- REP1-L: an `op` inside `complete_flags` / `sets_flags` is accepted and ignored (could be an error); World Forge Validate all status line says "All N worlds valid" without counting warnings; v1 save fixture uses the structured quest blob (the real v1 quest format is not exercised).
- Accepted lows: AI combo order ignores statuses consumed earlier in the same plan; exploit consume also runs on a fully absorbed hit; `RegionMapView._exit_tree` clears busy on a plain `remove_child`.
- The `export_to` rollback path (second rename fails) is untested: the sandbox runs as root.

- POI-FIX: point markers (ring icon, e.g. Ashlands `beacon`) only log "Flags set: ..." when walked onto; a friendlier note is a design call. In the Scene Runner the main menu's "Esc: back to the hub" hint sits half behind the runner's Menu pill (UI/UX).
- TRAV2-L: the region map chest note prints the raw items dictionary ("Opened: {...}"); cosmetic.

## Notes for the next session

- Economy mode as before. The arena suite needs `SUITE_TIMEOUT=900` in the cloud (EFFICIENCY row), otherwise it fails with exit 124 after all tests pass.
- A parallel fix in a second worktree (`git worktree add`) worked; tell its builder to use absolute paths, since one stray run hit the main checkout. After a background `git merge` check for unresolved conflicts before gating (one gate ran on a half-merged tree).
- Branches to delete at the next cleanup: `claude/gameplay-poi-fix`.
