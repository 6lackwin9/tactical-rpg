---
name: milestone-status
description: Task completion per milestone (M0, M1, M2, M2b) with a breakdown by category (system, lane, layer) and status (done, in progress, queued, not started), plus what is in progress and ready to start. Use when the user asks for a milestone update, progress by milestone, how far a milestone is, or what is left in a milestone.
---

# Milestone status

1. Run the counter from the repo root of an up-to-date checkout (desktop: the planner checkout after
   `git fetch` and a detached checkout of `origin/develop`; cloud: the area thread's checkout after a pull):
   `python .claude/skills/milestone-status/progress.py` (desktop: the Python 3.12 full path,
   `$LOCALAPPDATA/Programs/Python/Python312/python.exe`). Options: `--milestone M1` for one milestone, `--all` to add
   the tasks outside the slice milestones, `--json` for raw rows.
2. It reads `games/emberwake/docs/MILESTONES.md` (which tasks belong to which milestone), `docs/TASKS.md` and
   `docs/archive/TASKS_DONE.md`. Statuses map to four states: `done` (also covered or folded), `in-progress` or
   `review` = in progress, `approved` = queued, `spec`, `idea` or `proposed` = not started; obsolete or dropped rows are
   left out. "Ready to start" lists queued tasks whose dependencies are all done.
3. Reply with, per milestone: the headline (done of total, percent, bar), the system table, the lane and layer lines,
   then in progress and ready to start. Lead with the milestone that is moving (the one with tasks in progress), keep
   the others to their headline unless the user asked for all. Add one plain line on what blocks the milestone if the
   ready list is empty while tasks are queued.
4. A task that matches no milestone counts as outside the slice. When new task rows are added (planner), add their
   system or id to the right row of `MILESTONES.md` in the same commit, or they drop out of the report.
5. Read-only: the skill changes nothing. Do not guess counts; when the script fails (a moved file, a new table header),
   fix the script or say so.
