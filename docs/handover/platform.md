# Platform & Workflow handover (2026-10-18 13:00 UTC)

Rotated by context size (278k). Read cloud.md first; everything below is what is in flight.

## Routines bound to the old session (recreate on the new one, then delete these)

- Daily dispatch 18:30 UTC: `trig_01G4hERhAcmLSpcpZgzrsQ8y` (respects the building slots, cloud.md "Usage caps").
- Mode switch 15:45 UTC: `trig_01BDg2cGEakGSdBb7XYWFWY7` (send_later). At 15:45: planner-ops "Current mode" to economy, fill the standard row of `docs/metrics/usage/MODE_TRIAL.md`, land it, tell each area thread "mode economy until 18:45 UTC", and send_later 18:45 to switch back to split (economy for build lanes, standard for specs, design reviews and the final review).

## Usage tracking (user 2026-10-18; repo home `docs/metrics/usage/`, README there)

- On each pasted report: CSV into `reports/`, sessions.json and pins.json, HISTORY row, `efficiency.py` (time without Z), DELTA section, copy to `/mnt/project-files/metrics/`, reply with the delta only (top 3 threads and why), cutbacks need Tushar's typed OK.
- `get_session <id>` returns exact per-session usage (`external_metadata.usage`: cache reads, writes, output, cost). Use it for the trials instead of guessing rows by title.
- Sonnet trial: Assets & Audio `cse_01V6GmFsz2ESnDgqza17YhQt`, Sonnet since 12:15; `TRIAL_SONNET.md` (the thread sends "landed <id> <sha> model=<m>").
- Mode baseline: standard 12:42-15:45, economy 15:45-18:45; `MODE_TRIAL.md`; report with the delta after 18:45.

## Queue

- CLN-17, CLN-18: approved (user 12:14), cleanup lane; wait for the rotating building slot (coordinator assigns).
- Arena trim, proposed in `docs/EFFICIENCY.md`: Playtest 89 s, CombatUiFramework 86 s, Bal2Rules 81 s, Planning 71 s are 42% of the arena; ask Tushar before starting.
- Forge tab fixes: waiting on Tushar's list.

## Landed this session

RUNNER-WINDOW 5ddcea3, PATCH-NOTES bd9bff1 (Merge & Release writes PATCH_NOTES.md; landing threads send it "landed <id> <sha>"), PREFS-FILES 462d2f2 and PREFS-ASSETS-SPLIT d80d9ed (`docs/preferences/`), FC-TIMEOUT 182f9bb, TRIAL-SONNET e1bedb0, GATE-SPLIT bd932b2 (8 systems, `fc.sh gate auto`, CI matrix; release pinning step in cloud.md). Release PR #39 still runs the old single CI job; the next batch uses the matrix, so watch its first run.

## Sessions

Coordinator `session_015sTvb6o2FYckV681cWHXtS`; Merge & Release `cse_01KhQLgrvAgUunc9TcJKTHzK`; Gameplay `cse_01448UbhXvTzeBA1Brw4buLy`; UI/UX `cse_01JBWtdZFJoHPjEnzP1TUwVR`; Balance `cse_017JH6C5a2uwCQHAbGFH5whk`; Design `cse_01GS8RxAznG7gTrbfCb7w5uB`; Assets & Audio `cse_01V6GmFsz2ESnDgqza17YhQt`.
