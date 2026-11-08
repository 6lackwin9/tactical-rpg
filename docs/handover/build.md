# Build Session handover (cloud)

Updated 2026-11-07 by the cloud Build Session after the R1 release. Read this, then run `/build-session`.

## Goal of the session

Build the `R1` path in `docs/READY.md` section 1, then open the `R1` release PR `develop -> main` (user goal, FB-65,
accepted by Design @57b5b77d): `8 -> 11 -> 12 -> 13 -> 21 -> W2-1 -> W2-2 -> W2-3`, release after `W2-3` passes the full
gate and a `sim feel` sanity check. Rows `14-20` are post-`R1`.

## Queue position

- R1 is released: `main` @98ab973a (PR #55, 2026-11-07), tree = develop @f7f5cb71. Every R1 row (Wave 1, W2-1..W2-3,
  AUDIT-R1) is on develop. main now has develop as an ancestor (the two histories were unrelated before the release
  merge commit), so later releases are plain develop -> main merges.
- Next: nothing ready-to-build for R1. Post-R1 rows wait in READY (C02-A, R1-BAL, U2.3-TIERS, BASERULES1-PP, rows
  14-20, W2-4..W2-6) until Design promotes them; fetch READY at each start.

## In flight

Nothing. All task branches are squash-landed and closed; the release branch `claude/release-batch-vmj4rj` is merged.

## Open with Design

FB-85 to FB-92 in FEEDBACK.md (FB-87 asks Design to ratify the earlier PHASEFOLD re-records under the Authority
Matrix; FB-92 holds the sim feel numbers and the dropped `docs/metrics/` files).

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
