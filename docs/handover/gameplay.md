# Gameplay handover

Updated 2026-10-18 07:10 UTC by Gameplay session session_01HQvENsjr5dFTBZp6L6GkPU ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`. Dispatch mode: economy (user 02:35).

## Queue

Empty: OBJ1-L and ITEMUSE1-L are done. Take the next approved Gameplay task from `docs/TASKS.md` or the daily dispatch.

Open user card (Gameplay thread): "Give each hero their class's free potion in every battle?" (rec No free potion). Built with `PartyBuilder.HERO_KEEPS_CLASS_POUCH = false`. On "No free potion": ask Assets to remove the `pouch` rows from the warrior, ranger, priest and mage classes and add a decisions row. On "Free potion": set the flag true (heroes add their saved pouch after the class potion; overflow ids are kept on write-back) and add the decisions row.

## Landed on `develop` (session 01HQvENs)

OBJ1-L schema (2c9bc80) and behaviour (3abdb63): objectives, deployments, guests, verdict, `CombatSession.outcome`; also fixed `_restore_snapshot` losing `reinforcements_spawned`. ITEMUSE1-L schema (ad83b98: pouch, save v8, `use_reach`, ITEM step) and behaviour (e5653ac: ItemCommand, undo/reset, write-back skipping guests, GearModel pouch API, AI/sim `pouch=`, recordings store `pouches`). Hand-offs sent: OBJ1-U and the Gear tab pouch buttons (UIR-3) to UI/UX, OBJ1-A and ITEMUSE1-A (both landed) to Assets, OBJ1-B to Balance.

## Landed on `develop` (previous session)

WORLDMAP-ENTER (0bd96e3, Tushar's bug: click your own pin to enter the town), RECRUIT1-L schema (795aca1) and behaviour (0d6a3ae), INTENT1-L (a56dda7), MAPREAD1-L1 with location `map_pos` (bf162bb, asked by Assets and UI/UX), BOSS1-L schema (1cb608e) and behaviour (5c85e7e). Hand-offs sent for RECRUIT1-A, INTENT1-A/-U, MAPREAD1-A/-L2, BOSS1-A/-U.

## Sessions to ping

Assets & Audio session_01UywiZpBegbdrmR76smao3d, UI/UX session_01EFGiKxfL6TjsoGEDnQG5hc, Balance session_012hWXrrHGj1VxCBHAEN59mo, coordinator session_015sTvb6o2FYckV681cWHXtS (check the project's thread list if these rotated).

## Follow-ups (not task rows; ask before building unless it is a plain bug)

- OBJ1: wagon_raid has 3 raiders, so ambush puts 1 behind the party (spec acceptance assumed 2); a fled-by-exit battle now reads "⚑ BATTLE LEFT". ITEMUSE1: Gear tab buttons are UIR-3's; a recording without `pouches` replays as before.
- `tools/sim/sim_runner.gd` `side_hp_frac` counts reserved phase-wave units in enemy HP from the start (no effect on Grakka, whose phases have no wave).
- RECRUIT1: the World Forge `start_party` picker lists catalog units only; the `"'%s' can't join the party."` refusal is a defensive extra.
- Older items from the previous handover still open: ExploreRunner child-world `world_id` on return, REP1-L `op` in complete_flags ignored, POI point-marker "Flags set" note, TRAV2 chest note prints raw items.

## Notes for the next session

- Land with a detached HEAD (`git checkout --detach origin/develop`, `git merge --squash <branch>`, commit, `git push origin HEAD:develop`): checking out `develop` fails when another worktree holds it, and a following `reset --hard` then hits the task branch.
- After a schema landing, merge `origin/develop` back into the task branch at once; a later merge shows self-conflicts with the squash commit (keep the branch side).
- Two worktrees in parallel (one per task, absolute paths in the builder prompt) worked well: RECRUIT/MAPREAD in the main checkout, INTENT/BOSS in `/home/user/wt-intent`.
- The arena suite needs `SUITE_TIMEOUT=900`. A fresh container's first `godot --headless --import` can abort with "free(): invalid pointer"; run it again.
- Branches to delete at the next cleanup: `claude/obj1-l-objectives`, `claude/itemuse1-l-items`, `claude/worldmap-enter-here`, `claude/recruit1-l-recruits`, `claude/intent1-l-enemy-intent`, `claude/mapread1-l1-markers`, `claude/boss1-l-phases`.
