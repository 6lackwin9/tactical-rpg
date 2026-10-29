---
name: task-landing
description: Planner checklist for landing a finished, reviewed task (desktop, batched gate): which docs a change must update, the task-row and decision formats, the commit message, staging and flushing, worktree cleanup and docs-merge conflicts. Use after a builder reports and its change-reviewer passes, before stage.sh; also when a flush or rebase hits docs conflicts. Run docs_needed.py first: it lists the docs the diff should touch but does not.
---

# Task landing (planner, desktop)

Order: review passed -> docs delta -> commit -> `stage.sh` -> confirm STAGED -> cleanup -> flush every 3-4 tasks.

## 1. Docs the change needs

Run from the task worktree: `$LOCALAPPDATA/Programs/Python/Python312/python.exe .claude/skills/task-landing/docs_needed.py`
(it compares the worktree with `staging`). Each line it prints is a doc to update or a note to justify skipping.
The rules it applies, so a builder's docs delta can be checked against them:

| The diff touches | Update |
| --- | --- |
| any task | `docs/TASKS.md` row: Status `done (YYYY-MM-DD: what was built, user-facing defaults)`; a sliced task stays `in-progress (... slice n done; next: ...)`; follow-ups found in review go into the row of the task that will do them |
| `engine/crates/**` public API | `engine/README.md` (the crate's row or bullet); `engine/docs/PORT_NOTES.md` (a `none` row for engine-only work, or the Godot file it ports) |
| `kits/rpg/crates/**` | `kits/rpg/README.md` (one line per hook task); PORT_NOTES for engine-only modules |
| a new source or test file | `docs/STRUCTURE.md` (the folder's row) |
| a game system a reader looks for | `games/emberwake/docs/GAME_BRIEF.md` "Where to look for X" |
| behaviour that differs from its spec | an "As built" note in the spec (never rewrite the spec text) |
| a user answer or a planner ruling on a rule | a row in the matching `games/emberwake/docs/decisions/<system>.md` (`docs/decisions/workflow.md` for process) |
| a retired parity check, a re-recorded golden, a dropped idea | `docs/HISTORY.md` (append only) |
| `games/emberwake/data/seeds/*.json` | `games/emberwake/data/seeds/README.md` |
| unit art or `tools/sprites/**` | `games/emberwake/assets/README.md` and/or the spec of the rig task |
| generated art | a prompt record (`.json`) beside each source in `art_src/` |
| a new task row | `games/emberwake/docs/MILESTONES.md` (its id or pattern), or it drops out of `milestone-status` |
| a new recurring failure | the `known-issues` skill table |

## 2. Commit

One commit per task on the task branch, message:

```
<TASK-ID>: <what it does, one line>

<what changed per layer, the behaviour, tests, docs; user-facing defaults and anything retired>

Review: change-reviewer <verdict> (<fixes applied>)
Gate: <builder's gate result>; partial gate at staging

Co-Authored-By: <the attribution line from the session's system reminder>
```

Before `git add -A`: `git status --short` must show no `target*` folder and no scratch files (2026-10-29: build
folders were committed). Docs-only planner commits go straight to `develop` from the planner checkout.

## 3. Stage, flush, clean up

- Start a task in a lane (decision "Builder efficiency"): `bash <planner dir>/lane.sh <1-3> <task>` checks out
  `claude/<task>` from `staging` in `C:/dev/tactical-rpg-lane<n>`, whose build cache stays warm. Briefs name that path
  and tell the builder to finish with `bash <planner dir>/pgate.sh` (the partial gate), not the full gate.
- `bash <planner dir>/stage.sh <worktree> <name>`: rebases onto `staging`, refuses a committed build folder, runs the
  partial gate (type check, the touched crates' tests, layers, docs), fast-forwards `staging`. Read the result first.
- Clean up ONLY after a `STAGED` line, in a separate command: `bash <planner dir>/cleanup.sh lane<n>` detaches a lane
  and deletes its task branch (the cache stays); for a one-off worktree `bash <planner dir>/cleanup.sh <name>` removes the
  worktree, its branches (`claude/<name>`, `claude/<name>-land`) and its build cache `%TEMP%/aegis-target-tactical-rpg-<name>`
  (2026-10-29: caches of removed worktrees, 3-16 GB each, filled drive C:). Never chain cleanup after `stage.sh`
  (2026-10-29: a failed stage plus a chained cleanup deleted a branch).
- A task branched from an older `staging` that was rebased since: cherry-pick only its own commit onto a fresh branch
  from `staging` (a plain rebase replays the old copies of already-landed commits).
- `bash <planner dir>/flush.sh` every 3-4 staged tasks, before a release or a wind-down: full gate on `staging`, push to
  `develop`. If `develop` moved during the gate with docs-only commits, rebase `staging` onto it and push the gated
  commits (the code is unchanged); anything else: rerun the flush.
- Do not push planner docs to `develop` while a flush runs (it makes the flush's push fail); commit locally, push after.

## 4. Docs conflicts

`python <planner dir>/resolve_docs.py <files>` merges table rows (the more advanced status wins, same-id rows merge,
other clauses are kept). Afterwards check: no conflict markers, no duplicated task rows
(`grep -o "^| <ID> |" docs/TASKS.md | sort | uniq -c`), each landed task's status reads `done`, `tools/fc.sh docs`
clean. A conflict in a code file is never resolved by the docs helper: hand it to the task's builder.

## 5. After landing

- `milestone-status` counts from `docs/TASKS.md` and `MILESTONES.md`: a done task still showing as queued means its
  status cell does not start with `done`.
- Tell the user what landed, where it is (`staging` or `develop`), and any open questions with a recommendation.
