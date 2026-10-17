# Platform & Workflow handover

Written 2026-10-17 20:38 UTC by the outgoing Platform & Workflow session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`.

## Queue

- No approved Platform task is open. Ideas waiting on a housekeeper or user decision: CLN-17 (unwired Dojo-27), CLN-18 (Forge widget leftovers after TOOLS-RO).
- Landed today on `develop` (release picks them up): REVIEW-F4 (Butler pinned to 15.32.0 + sha256), REVIEW-F5, TOOLS-RO + content-edits doc, PC-LAUNCH-CACHE, SIM-IMPORT, DEPLOY-AFTER-RELEASE, WEB-PAUSE, PULL-ON-LAUNCH, DEV-USERDATA, SAVE-BACKUP.

## Waiting on the user

- Nothing. The save-protection card was answered "Backups + save test" (20:21): the backups landed (SAVE-BACKUP, `run.bat`); the save test is task SAVE-COMPAT (Gameplay, approved, sent to the Gameplay thread to run after TRAV1-L).

## Routines and watches

- Daily dispatch routine "Daily task dispatch to area threads (economy)", `trig_01P3NjaHL5fcD6Jbqyrdf7KW`, 18:30 UTC, is bound to the OLD Platform session. The coordinator must rebind it to the new session (or recreate it there); its prompt is unchanged.
- No PR subscriptions.

## Notes for the next session

- Web deploys are paused until a playable build (decision "Web deploy paused"); "deploy" still means a manual `workflow_dispatch` of `web.yml` on `main`.
- `run.bat` / `run_dev.bat` and the `pc-launch` skill reach the user's PC only with the next release; nothing has run them on Windows yet, so check the first launch report from the "Launch game on your PC" thread.
- Branches to delete at the next cleanup (all landed or temporary): `claude/butler-probe` (temporary probe; not on develop, so pass it in `also_delete`), `claude/pc-launch-cache`, `claude/review-f4-pin`, `claude/review-f5`, `claude/tools-ro`, `claude/sim-import`, `claude/deploy-after-release`, `claude/web-pause`, `claude/run-pull`, `claude/dev-userdata`, `claude/save-backup`, `claude/thread-handover`.
- The auto-mode safety check blocks recording a decision without the user's own answer: get a card tap or the user's message before writing a `docs/decisions` row.
