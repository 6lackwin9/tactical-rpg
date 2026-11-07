# Build Session handover (cloud)

Updated 2026-11-07 by the cloud Build Session after W2-1. Read this, then run `/build-session`.

## Goal of the session

Build the `R1` path in `docs/READY.md` section 1, then open the `R1` release PR `develop -> main` (user goal, FB-65,
accepted by Design @57b5b77d): `8 -> 11 -> 12 -> 13 -> 21 -> W2-1 -> W2-2 -> W2-3`, release after `W2-3` passes the full
gate and a `sim feel` sanity check. Rows `14-20` are post-`R1`.

## Queue position

- Done (all on `develop`): Wave 1 R1 rows 8, 11, 12, 13, 21, 22; W2-1 complete: U1.2-S2 (4f84b780), U2.2-S3
  (ecf83501), U1.2-S3a crit from PERC (8bf4be58), U1.2-S3b know_potency at 0 (1bf30ee6), U1.2-S3c speed split
  (b4f11aa8), U1.2-S4 tier table, HP form held (28a5308b), U1.2-S5 data pools + inert Composure (e5201625).
- Next: W2-2 (R1-7, the Unit half): U3.2-S3 soft wield penalty + `state.rs:180` equip check and deck ownership on
  `progression::member_passives` + party-screen penalty summary in FEEDBACK; U3.2-S4 reversible item grants, OFFHAND1,
  level 2-3 bonuses, `shd_tower` / `wpn_glass_wand` notes; U3.2-S5 carry/burden; U4.2-S3 temperament; U4.2-S4 U4
  evaluators on the U1 pipeline. One builder run and commit per slice, each with its named golden re-records.
- Then W2-3 (R1-8), then the full gate + `sim feel` sanity check and the `develop -> main` release PR.

## In flight

Nothing. All task branches are squash-landed and closed for branch-cleanup.

## Open with Design (none blocks R1)

FB-67, FB-69 to FB-79 (FEEDBACK.md; FB-67/69/70/71 full text in the archive). User decisions this session: R1 keeps
today's tier HP formula as a temporary measure (FB-78; the user wants stat-based units by one rule); the KNOW potency
weight is 0 until chosen (FB-76).

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
