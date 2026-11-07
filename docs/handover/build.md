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
- After a squash landing, close the task branch so `branch-cleanup` deletes it (its "content already on develop" test
  misses squash-landed branches once develop changes their files, e.g. FEEDBACK.md):
  `c=$(git commit-tree "origin/develop^{tree}" -p origin/<branch> -p origin/develop -m "Close task branch")`, then
  `git push origin "$c:refs/heads/<branch>"`. Done for the three branches above (deleted at the next daily run after 6 h).
- No separate READY.md watcher (user 2026-11-07: Design is idle); fetch and re-read READY.md at each task start.
- A fresh container needs `libasound2-dev` for the gate; the session-start hook now installs it.
