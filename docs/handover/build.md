# Build Session handover (cloud)

Written 2026-11-07 by the cloud Build Session (rotated at about 246k context, cloud.md "Fresh sessions"). Read this, then
run `/build-session`.

## Goal of the session

Build the `R1` path in `docs/READY.md` section 1, then open the `R1` release PR `develop -> main` (user goal, FB-65,
accepted by Design @57b5b77d): `8 -> 11 -> 12 -> 13 -> 21 -> W2-1 -> W2-2 -> W2-3`, release after `W2-3` passes the full
gate and a `sim feel` sanity check. Rows `14-20` are post-`R1`.

## Queue position

- Done this session: `SKILLS1-FB` (127f168), cloud hook ALSA headers (d4094eaf), `U3.2-S2` row 8 (c212bf4c).
- Next: row 11 `S1.2-S1` (`docs/sessions/S1_STATUS.md` section 4 item 2, FB-48 in the archive): the 17 status rows
  as a frozen kit table behind `cap_of`, `stacking_of`, `display_name`, `row_of`, plus behaviour primitive tags; 0 golden
  diffs. Logic-builder run (several files).
- Then 12 (`rules_contract_golden` re-record), 13, 21, W2-1..W2-3 (each golden-moving switch its own named commit, logged
  in FEEDBACK for HISTORY). Row 12 and post-R1 row 17 both promote the debt numbers: 17 reuses 12's.

## In flight

Nothing. No open task branch with unlanded work (`claude/skills1-fb-feedback-check`, `claude/hook-alsa`,
`claude/u32s2-family-table` are all squashed onto `develop`).

## Open with Design

- FB-67 (row 8 held parts): reach stays `REACH_WEAPON_REACH` (no `range_band` field) and the printed check lists stay
  code until Design answers. Not an R1 blocker. Build recommends reach option B, lists option C.

## Working notes

- Land every task as one squash commit on `develop` (user 2026-11-07); `wip` and `merge develop` commits stay on the
  task branch.
- `python3 tools/planner/feedback_check.py --pending-ok`, then `archive` when READY section 4 answers items; fix the
  "FB-.. are closed" lines in FEEDBACK by hand.
- The user asked for a watcher on `docs/READY.md`: a background loop that fetches `origin/develop` every 5 min and
  exits (waking the session) when the READY.md blob changes; re-arm it after each wake (2 h limit per run).
- A fresh container needs `libasound2-dev` for the gate; the session-start hook now installs it.
