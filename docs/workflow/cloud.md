# Cloud workflow (Claude project threads)

Use this when Claude runs in a cloud container (`$CLAUDE_CODE_REMOTE` is `true`). Shared rules: `CLAUDE.md`, skills `planner` and `planner-ops`. Differences from the desktop and the switching checklist: [README.md](README.md). Decided by the user 2026-10-17 (decisions "Cloud workflow" and "Environment split" in `docs/decisions/workflow.md`).

## Sessions

- **Project chat (coordinator)**: the front desk. The user asks there; it routes each ask to the matching area thread and relays follow-ups. It does not edit the repo.
- **Area thread**: a standing thread for one focus area (table below), not for one task. It is the planner for the tasks of its area: it works them one after another, and for each one reads the task row and spec, runs the lane agents as subagents in its own checkout, one after another, commits, pushes, and squash-merges the finished task into `develop` when allowed (below). It opens no PR.
- **New threads need the user's approval.** The coordinator opens an area thread when its first task arrives. Before opening any other thread, or a second thread for an area, it asks the user in the project chat and says why the work cannot go in an existing open thread. The PC launch thread stays as it is.
- **Merge & Release thread**: a standing thread that owns the only route to `main`. It keeps the approval and ticket queue (TASKS status, decisions check, `Review:` lines) and opens one batched `develop` -> `main` PR per batch (below).
- Decision requests go to the user in the thread (a decision card when offered): question, options with one-line consequences, one recommendation. Work continues on the recommended option unless the step cannot be undone.
- Failure alerts: a thread reply that leads with what is blocked and what the thread is doing about it (replaces the desktop `ALERT` line).
- Dispatch mode: the user types `mode <name>` in the project chat; it applies to later threads. Scheduled routines always run in economy mode.

## Area threads

| Area thread | Covers | Agents it runs |
| --- | --- | --- |
| Gameplay | combat, map, battles, grid, terrain, travel, quests, saves | `logic-builder`, `asset-builder` for the task's data |
| UI/UX | screens, menus, HUD, browser rendering | `logic-builder`, `asset-builder` for UI art |
| Assets & Audio | art, icons, SFX, fonts, catalog and world data | `asset-builder` |
| Balance | sims, `docs/BALANCE_REPORT.md`, tuning | `balance`, then `design-brainstorm` for options |
| Design | specs, decisions, the roadmap; each spec is approved here, then built in its area's thread | `design-brainstorm` |
| Platform & Workflow | CI, the web build, playtest tooling, agent and skill changes, cleanups | `housekeeper`, `logic-builder` for tools and CI |

Every area thread also runs `change-reviewer` on each task branch before it merges into `develop`. A task whose area is unclear goes to the thread that owns most of its `Touches`.

## One task at a time, one branch per task

1. An area thread works its tasks in sequence. Each task gets its own branch (`claude/<task-id>-<slug>`, or the branch the thread was given for its first task), cut from the latest `develop`, and lands on `develop` as one squash commit; the next task starts from `develop` after the previous one lands or is parked on its pushed branch. Feature branches are free: threads create, push and update them as they like. No sibling worktrees and no `tools/wt_remove.ps1`.
2. Subagents work in the thread's checkout and never commit. The thread commits after each green `tools/fc.sh` step and pushes, so a recycled container or a usage limit loses nothing. `.agent/progress.md` is still kept, but the pushed branch is the real checkpoint.
3. The thread updates only its own rows in `docs/TASKS.md` (status, branch name), its line in the `docs/AGENT_MAP.md` finished-runs log, and the decision rows for answers the user gave in that thread, all inside its own task commit. Git merges replace the desktop "single writer" rule.
4. Overlap: before starting, compare the task's `Touches` with the other areas' unmerged task branches (`git branch -r`, the area threads' status) and with `develop`; sequence tasks whose `Touches` overlap, as on desktop.

## Lanes inside an area thread

The thread is the planner (lane table: skill `planner`). It writes the planner docs itself (TASKS rows, decisions given in the thread, AGENT_MAP line, README and docs deltas) and hands every other lane to that lane's agent, as on desktop:

| Work in the task | Agent the thread runs |
| --- | --- |
| engine code, schema, validation, tools, tests, and fixes to them (review fixes included) | `logic-builder` |
| catalog or world data and its seed | `asset-builder` |
| a new or changed spec, design options | `design-brainstorm` |
| simulator runs and `docs/BALANCE_REPORT.md` | `balance` |
| review of the whole branch before the PR is marked ready | `change-reviewer` |
| a cleanup pass (after a goal or milestone finishes, or on request) | `housekeeper` |

1. Only these six agents (`.claude/agents/`) are dispatched; generic agent types are not used for lane work. Effort and model come from the dispatch-mode table (skill `planner-ops`).
2. Work none of them covers goes to the user as a proposal: extend an existing agent or add a new agent type. Any new or changed agent or skill needs the user's approval before it merges.
3. A thread that does a lane's work itself has broken the lane rule; "no subagent" is valid only for a planner-docs-only task (decision records, TASKS or ROADMAP edits). The AGENT_MAP line names each agent the thread ran.

## Landing a task on `develop` (area thread, no PR)

`develop` is the long-lived integration branch (created once from `main`; if `git ls-remote origin develop` is empty, the Merge & Release thread recreates it from `main`); `main` only moves through the batched PR below, so CI runs once per batch instead of once per task (decision "Batched merges").

1. Review: run the `change-reviewer` agent on the whole task branch (`git diff origin/develop...HEAD` plus any uncommitted change). Fix its blockers (re-run it if the fix is more than a few lines). A docs-only diff still gets the docs profile of the review.
2. Gate: `check_project.ps1 -Fast -Quiet` or `tools/fc.sh gate` (both work; ~3-4 min on a cloud container, not 32 s) and `tools/fc.sh docs`. There is no IDE Problems rule in the cloud.
3. Approved means: the task row in `docs/TASKS.md` has status `approved` (set from the user's word) or the change is recorded as approved in `docs/decisions/*`. Only approved work lands on `develop`.
4. Land it: `git fetch origin develop`, merge `origin/develop` into the task branch (resolve conflicts there; `docs/AGENT_MAP.md` and `docs/archive/TASKS_DONE.md` conflicts are "keep both"), re-run the gate if `develop` brought code changes since the last gate, then `git checkout develop && git merge --squash <task-branch> && git commit` and `git push origin develop`. If the push is rejected because `develop` moved, fetch and repeat from the merge step. Landing while a release PR is open is allowed: the new commit joins that batch and CI re-runs on it.
5. The squash commit title starts with the task id; its body says what changed and ends with the `Review:` line, for example `Review: change-reviewer clean` or `Review: change-reviewer 1 high (fixed), 2 nits accepted`, and `Gate: fc.sh gate passed`. No task lands without both lines.
6. Not approved: a small change waits on its pushed task branch and goes to the user as a question in the thread; a large one waits for the user's feedback before work starts. Rows in `spec` or `idea` status and any design question that comes up mid-task go to the user.
7. Never push to `main`. Keep the task branch until its batch is on `main`, then delete it.

## Batched release to `main` (Merge & Release thread)

1. Batch: once a day before the 18:30 UTC routine, and whenever the user asks, if `develop` is ahead of `main`.
2. Check each task commit in `git log origin/main..origin/develop`: task id in the title, `Review:` and `Gate:` lines in the body, task row `approved` or a decision row. A commit that fails the check is reverted on `develop` (`git revert`, pushed) and sent back to its area thread through the coordinator. A reverted task lands again as a new squash commit of its fixed branch onto the latest `develop` (merging the old branch again would be a no-op for git).
3. Open one PR `develop` -> `main` titled `Release <date>: <task ids>`; body: one line per task with its review verdict, plus anything waiting on the user. CI (`gate.yml`, `web.yml` build) runs on this PR only, and is skipped when the batch changes only Markdown and `docs/` (those passed `tools/fc.sh docs` before landing).
4. CI green: merge with a **merge commit** (not squash, so `develop` and `main` keep a shared history and the next batch has no conflicts). The push to `main` runs the gate again and deploys the web build (GitHub Pages, itch.io).
5. CI red: find the task commit that broke it (the failing suite names its area), revert it on `develop`, push (CI re-runs on the PR), and send the failure to its area thread through the coordinator. Never fix another area's code in this thread.
6. After the merge, delete the task branches of the batch. Hotfix to `main` without `develop` only with the user's word; after any change that reaches `main` outside a release PR, merge `main` back into `develop` right away so the next batch does not conflict.
7. The `balance` agent keeps running on `origin/main`, so it measures released code only; ask for a release first when a balance run needs work still on `develop`.

## Scheduled routines

- Daily planner pass (economy mode, 18:30 UTC): pick the next `approved` task whose `Depends on` rows are `done` and whose `Touches` do not overlap another area's unmerged task branch, and hand it to its area thread, which lands it on `develop`. The routine wakes the Platform & Workflow thread (no new thread per run), which only dispatches: it sends the task to the area thread's session as a cross-session message (thread found by title in the project's thread list), or keeps it when the area is Platform & Workflow. If the area has no live thread, it asks the coordinator session to open that area thread (already approved by the user) and pass the task on. The area thread stops at any design question. If the Platform & Workflow thread is closed, recreate the routine bound to the new one. Balance and housekeeper routines are added later on request.
- Release batch: the Merge & Release thread keeps its own daily routine bound to itself, before the 18:30 dispatch.
- Routines follow the same approval rule as threads.

## Environment

- Godot 4.7.2 and PowerShell 7 come from the environment setup script (copy: `/mnt/project-files/cloud-setup/environment-setup-script.sh` in the project files); `.claude/hooks/session-start.sh` installs them if the script did not and no-ops otherwise. `github.com` must stay allowed in the environment's network policy.
- Web build: `.github/workflows/web.yml` exports the `Web` preset on every PR into `main` (artifact `web-build`) and deploys `main` to GitHub Pages; setup and limits in [web.md](web.md).
- No interactive play. For UI work, render frames under Xvfb and attach them to the thread reply: `xvfb-run -a -s "-screen 0 1280x720x24" godot --rendering-driver opengl3 --path . <scene> --write-movie out.png --fixed-fps 10 --quit-after 30`. Playtesting feel, input and audio stays on desktop.
