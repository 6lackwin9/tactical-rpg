# Platform & Workflow handover

Written 2026-10-18 05:40 UTC by the outgoing Platform & Workflow session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`.

## Queue

- No approved Platform task is open. Ideas waiting on a housekeeper or user decision: CLN-17 (unwired Dojo-27), CLN-18 (Forge widget leftovers after TOOLS-RO).
- Landed on `develop` since the last handover (release picks them up): SPLIT-LANDING, GATE-ONCE (decision "Gate once per task"), PULL-RESTART (Restart runner pulls via run.bat), FORGE-VIEW (display-only Forge, in-game skill preview per rank), NOTE-BUTTON ("✎ Note" pill button), MODE-ECONOMY (planner-ops current mode = economy, user 2026-10-18), OPTLAUNCH1 (`run.bat opt a|b|c|<branch>`, `run_opt.bat`, pc-launch "launch option A").

## Waiting on the user

- Nothing. Tushar tests on dev (`run_dev.bat`, then Restart runner) until a stable build; main later. He will send tab-specific Forge fixes (Items, Passives, Classes, Units, Encounters, Terrain) after the Skills redesign.

## Routines and watches

- Daily dispatch routine "Daily task dispatch to area threads (economy)", `trig_01QHhPUNaSM7dBgUu2hTDFQn`, 18:30 UTC, is bound to THIS (outgoing) session. Recreate it on the new session with the same prompt (`get_trigger` shows it) and send the coordinator the new id so it can disable this one.
- No PR subscriptions.

## Notes for the next session

- Untested on Windows so far: PULL-RESTART (Restart runner -> run.bat), OPTLAUNCH1 (`run.bat opt`). Watch the first reports from the "Launch game on your PC" thread.
- "send my playtests": Scene Runner "Copy to repo" only copies into the PC folder's `playtests/inbox/`; the coordinator was asked to have the PC launch thread commit and push it on a `claude/playtests-<date>` branch, then this thread lands it on develop. Not yet exercised.
- The auto-mode safety check blocks recording a decision without the user's own words in this thread (a coordinator relay of a card tap is not enough): ask for a one-line reply first.
- In the cloud the arena suite needs `SUITE_TIMEOUT=900 bash tools/fc.sh gate` (default 290 s times out).
- Branches to delete at the next cleanup: `claude/split-landing`, `claude/gate-once`, `claude/pull-restart`, `claude/forge-viewer`, `claude/playtest-notes`, `claude/optlaunch1`.
