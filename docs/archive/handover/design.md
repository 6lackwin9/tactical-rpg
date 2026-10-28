Archived 2026-10-29: the cloud wind-down state of 2026-10-18 (CLEANUP.md CLN-32); superseded by a fresh handover that a resumed area thread writes (`docs/workflow/cloud.md`, decision "Fresh area sessions"). Text below unchanged.

# Design handover

Written 2026-10-18 16:30 UTC by the Design session at the project wind-down (Tushar 16:28: move to an "interactive" mode, agents act only on his requests). Read this, then `docs/workflow/cloud.md` and `docs/workflow/design-review.md`. Resume only when Tushar asks for Design work.

## State

- Nothing waits on the user from Design. Nothing is in flight.
- Option builds are paused: Design proposes, Tushar approves (decision workflow "Option builds paused"). UI picks he approves are preferences unless the idea came from him.
- Done today (all on `develop`): MORALE1 spec (rule 8 flee form accepted from MORALE1-L, c75bb11), PACE1 spec, PURSE1 spec, design reviews DR2 and DR3 (answered, copies in `docs/specs/`), COMBAT-UI3 mockups (Tushar picked A plus changes; UI/UX builds it; canvas https://claude.ai/artifact/E75abGsPLH6E6GZfBXVASi), LEVERS1 spec (`docs/specs/LEVERS1.md`, approved 14:26 all recs, plus "Smarter Shield Wall" card 15:36, e0f1ae5).

## Queue (start only on request)

- Remaining DR1 design rows in `docs/TASKS.md`: KEEP1, DIFF1, DEEDS1, RESPEC1, OFFHAND1, DESERT1, COMBO-A2.
- Next design review (DR4): rule 1 counts from 8e2530a (`docs/design/OPTION_BRANCHES.md`).
- Watch items from BAL5-R (d4ab083): line / wave / elite / boss clears 26 / 28 / 38 / 26 are under their bands with no lever in BAL5 (boss slightly too hard, not too easy). Balance owns the follow-up; Design may be asked for levers. Bodyguard (T4) returns only if LEVERS1-B still shows warrior-row won-fight allies under 40.

## Routines

- "Design review check" trig_016jzPSMrdHLPY3E6KFJzfHP (12:20 and 18:20 UTC) is DISABLED (wind-down, 16:30). Re-enable with update_trigger only if Tushar asks for scheduled reviews again.

## Rules changed today

- No "landed <id> <sha>" pings to Merge & Release (f1db72d); patch notes are written per release.
- Never touch the `engine` branch until Tushar approves (decision "Engine branch hold").
- Timing: lines carry `type=` and `tokens=`.
