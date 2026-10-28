Archived 2026-10-29: the cloud wind-down state of 2026-10-18 (CLEANUP.md CLN-32); superseded by a fresh handover that a resumed area thread writes (`docs/workflow/cloud.md`, decision "Fresh area sessions"). Text below unchanged.

# Gameplay handover

Updated 2026-10-18 17:00 UTC by Gameplay session session_01QCe4DugW3btbTntzfddbhN (project wind-down, user 16:28 UTC). Read this first, then `docs/workflow/cloud.md`. Mode is now **interactive** (planner-ops "Current mode", decision "Interactive mode"): act only on Tushar's direct requests; do not start queued rows on your own. Land with `bash tools/fc.sh gate auto` (per-system gate; a new suite needs its `if _on("<system>")` line in suite_registry.gd). Landed pings to Merge & Release are off (f1db72d); patch notes are written per release.

## Nothing in flight

No open branch work: the thread branch `claude/project-thread-diteok` equals develop after d0bcc9a.

## Queue (in order, start only when Tushar asks)

1. **LEVERS1-L1** then **LEVERS1-L2** (approved 14:26, `docs/specs/LEVERS1.md`), both after BAL5-L (logic row; `origin/claude/bal5-l` had WIP from another thread at 14:06, touching skill_def, status_effects, skill_validator, suite_registry). L1/L2 share tactical_ai.gd.
2. **LEVERS1-L3** (user card 15:36, LEVERS1 3.3): AI guard rule in tactical_ai.gd (taunt DEFEND skill such as shield_wall cast in the first free block when an adjacent ally is targeted by a foe intent this round and the unit is not already taunting; reckless skips; low-HP guard unchanged); Assets then does A3. After BAL5-L; shares tactical_ai.gd with L1/L2.

## Landed on `develop` (this session)

PACE1-L schema 39daec9 + behaviour a1ca665 (enrage phase effect, or_round trigger, empower status, pace field + boss warning, start_status grants at round 1, enraged intents; party_buff was dropped by the revised spec). PURSE1-L schema 02980d3 + behaviour d0bcc9a (purse field, PurseRules roll/pay, GameState.purses, save v10, BattleBridge pays on won). Assets told PACE1-A and PURSE1-A can start; Balance told PACE1-S; UI/UX told PACE1-U hooks and the Focus "Empowered" log-label clash (their call).

## Open questions (not asked yet; raise only if relevant)

- Playtest replayer rebuilds battles without game state, so game battles with start buffs or a saved party do not replay exactly; making it carry the state would be its own task.
- PACE1: a wave unit sharing an enraged boss ref would read as enraged without empower (only matters if a boss ref is reinforced).
- PURSE1: the win count goes up before the grant, so a once purse whose grant fully fails is used up having paid nothing (very unlikely).
- Until PACE1-A lands, both Ashmaw stands show the "boss fight has no enrage phase" soft warning.

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
