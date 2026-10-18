# Gameplay handover

Updated 2026-10-18 14:40 UTC by Gameplay session session_01448UbhXvTzeBA1Brw4buLy (rotation by context size, "Usage caps" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`. Dispatch mode: standard for every lane until ~15:45 UTC, then economy until ~18:45 UTC (user 12:42, `docs/metrics/usage/MODE_TRIAL.md`); keep `type=` and `tokens=` on every Timing line. Land with `bash tools/fc.sh gate auto` (per-system gate; `fc.sh systems` shows the pick; a new suite needs its `if _on("<system>")` line in suite_registry.gd). After each push to develop, send Merge & Release (session_01KhQLgrvAgUunc9TcJKTHzK) "landed <task-id> <sha>" (cloud.md "Landing a task" step 7).

## Queue (in order)

1. **PACE1-L** (approved 09:33, `docs/specs/PACE1.md`; decision "PACE1 two-act boss" in decisions/combat.md): enrage and party_buff phase effects, `or_round` trigger, `empower` status, `start_status` field (Q3 B: schema only, no reward data), `pace` field and boss warning, enraged intents. Commit 1 = schema slice. Overlaps MORALE1-L files (catalog_encounter, combat_session, schemas): merge develop first. On landing ping Assets (PACE1-A), Balance (PACE1-S) and UI/UX (PACE1-U).
2. **PURSE1-L** (approved 12:28, `docs/specs/PURSE1.md`, decision "PURSE1 encounter purse" in items-gear.md): purse field + validation, GameState.purses + save bump, PurseRules, BattleBridge.apply hook. Commit 1 = schema slice; ping Assets for PURSE1-A after it. Sequence with other save-bump tasks.
3. **LEVERS1-L1** then **LEVERS1-L2** (approved 14:26, `docs/specs/LEVERS1.md`), both after BAL5-L (check who owns BAL5-L; it is a logic row from the Balance levers spec). L1/L2 share tactical_ai.gd. Then **LEVERS1-L3** (user card 15:36, LEVERS1 3.3, row e0f1ae5): AI guard rule in tactical_ai.gd (taunt DEFEND skill such as shield_wall cast in the first free block when an adjacent ally is targeted by a foe intent this round and the unit is not already taunting; reckless skips; low-HP guard unchanged); Assets then does A3.

## Landed on `develop` (this session)

WMAP-BLANK 41d2282 (world map over the region map was zero-sized; known-issues row added), BREPORT1-L d0219fa (BattleMetrics/BattleReport, combo signal, results-model API), UIR-2 52dda97 (game menu popup over the paused map: Party, Journal, Save, Load, Settings, Quit; PLAY route; busy rules), POUCH-TIDY 8c1494c, MORALE1-L schema 6a473ad + behaviour f344531 (flee formula max(base, min(0.60, base+bonus)) accepted by Design), STATCRED1-L 3fa4cea (statuses credited to their applier; boss-phase grants to the enemy recipient). All pinged to the owning threads.

## Sessions to ping

Coordinator session_015sTvb6o2FYckV681cWHXtS, UI/UX session_01JBWtdZFJoHPjEnzP1TUwVR, Platform session_01F3J8BGWSSBbqckdqifYoJF, Assets & Audio session_01V6GmFsz2ESnDgqza17YhQt, Balance session_017JH6C5a2uwCQHAbGFH5whk, Design session_01GS8RxAznG7gTrbfCb7w5uB, Merge & Release session_01KhQLgrvAgUunc9TcJKTHzK (check the thread list if they rotated).

## Follow-ups (not task rows; ask before building unless it is a plain bug)

- From earlier handovers: REP1-L `op` inside `complete_flags` / `sets_flags` ignored; POI point-marker "Flags set" note (design call); OBJ1 wagon_raid ambush puts 1 raider behind the party; `sim_runner.gd` `side_hp_frac` counts reserved phase-wave units.
- BREPORT1 rows follow `session.allies` order; spec wants guests after heroes (told UI/UX to sort in the view or ask).
- `application/config/version` is unset, so the main menu version label reads "dev".
- StatusEffects.queue/grant_now with an empty source keep the previous applier's source (pre-existing).

## Notes for the next session

- Land with a detached HEAD (`git checkout --detach origin/develop`, `git merge --squash <branch>`, commit, `git push origin HEAD:develop`), then merge `origin/develop` back into the branch at once. Remove the task's TASKS row in the branch before the squash (the `Done:` line carries it); a split landing keeps the row until the second commit.
- A TASKS or GAME_BRIEF conflict after another thread's landing: keep both sides' live rows/lines, drop done rows.
- The Controls tab in Settings is near its width limit at text 1.15/1.30: keep `ControlsList` action labels short (a long one broke the settings fit check).
- Resume a builder with SendMessage for review fixes and stage 2 of a split landing. A container restart kills background work but keeps uncommitted files: commit them as WIP and resume the builder.
