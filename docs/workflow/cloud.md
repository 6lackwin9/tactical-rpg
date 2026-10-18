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
| Design | specs, decisions, the roadmap; each spec is approved here, then built in its area's thread; the design review cycle and its option branches ([design-review.md](design-review.md)) | `design-brainstorm`; `logic-builder` and `asset-builder` for option builds only |
| Platform & Workflow | CI, the web build, playtest tooling, agent and skill changes, cleanups | `housekeeper`, `logic-builder` for tools and CI |

Every area thread also runs `change-reviewer` on each task branch before it merges into `develop`. A task whose area is unclear goes to the thread that owns most of its `Touches`. Content changes (values, new items, enemies, abilities) follow `content-edits.md`: small tweaks land with a before/after table, new content needs one approval card.

## One task at a time, one branch per task

1. An area thread works its tasks in sequence. Each task gets its own branch (`claude/<task-id>-<slug>`, or the branch the thread was given for its first task), cut from the latest `develop`, and lands on `develop` as one squash commit (two for a split landing, "Landing a task on `develop`" step 4); the next task starts from `develop` after the previous one lands or is parked on its pushed branch. Feature branches are free: threads create, push and update them as they like. No sibling worktrees and no `tools/wt_remove.ps1`.
2. Subagents work in the thread's checkout and never commit. The thread commits after each green `tools/fc.sh` step and pushes, so a recycled container or a usage limit loses nothing. `.agent/progress.md` is still kept, but the pushed branch is the real checkpoint.
3. The thread updates only its own rows in `docs/TASKS.md` (status, branch name; it deletes the row when the task lands) and the decision rows for answers the user gave in that thread, inside its own task commit. It never edits `docs/AGENT_MAP.md` or `docs/archive/TASKS_DONE.md`: their entries go in the squash commit as `AgentMap:` and `Done:` lines and the Merge & Release thread writes both files at release time (decision "Shared logs"). Git merges replace the desktop "single writer" rule.
4. Overlap: before starting, compare the task's `Touches` with the other areas' unmerged task branches (`git branch -r`, the area threads' status) and with `develop`; sequence tasks whose `Touches` overlap, as on desktop.

## Lanes inside an area thread

The thread is the planner (lane table: skill `planner`). It writes the planner docs itself (TASKS rows, decisions given in the thread, README and docs deltas, the `AgentMap:` and `Done:` commit lines) and hands every other lane to that lane's agent, as on desktop:

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
3. A thread that does a lane's work itself has broken the lane rule; "no subagent" is valid only for a planner-docs-only task (decision records, TASKS or ROADMAP edits). The `AgentMap:` lines name each agent the thread ran.

## Landing a task on `develop` (area thread, no PR)

`develop` is the long-lived integration branch (created once from `main`; if `git ls-remote origin develop` is empty, the Merge & Release thread recreates it from `main`); `main` only moves through the batched PR below, so CI runs once per batch instead of once per task (decision "Batched merges").

1. Review: run the `change-reviewer` agent on the whole task branch (`git diff origin/develop...HEAD` plus any uncommitted change). Fix its blockers (re-run it if the fix is more than a few lines). A docs-only diff still gets the docs profile of the review.
2. Gate: `check_project.ps1 -Fast -Quiet` or `tools/fc.sh gate` (both work; ~3-4 min on a cloud container, not 32 s) and `tools/fc.sh docs`. There is no IDE Problems rule in the cloud. A docs-only diff (Markdown outside `.claude/`, no scripts, data, skills, agents or workflows) needs only `tools/fc.sh docs`; the release PR runs the full gate. Gate once (decision "Gate once per task"): while fixing review findings or test failures, run only what the change touches: `tools/fc.sh parse`, then the one suite that covers it (`tools/fc.sh arena` or `tools/fc.sh dojo`; the filter argument only narrows the output, the whole suite still runs); run the full gate once, when the task is ready to land. In the cloud this replaces the planner skill's re-gate after each post-gate change.
3. Approved means: the task row in `docs/TASKS.md` has status `approved` (set from the user's word) or the change is recorded as approved in `docs/decisions/*`. Only approved work lands on `develop`.
4. Land it: `git fetch origin develop`, merge `origin/develop` into the task branch (resolve conflicts there), re-run the full gate only if `develop` brought changes to files the task touches or to shared code they use (otherwise the release CI covers it), then `git checkout develop && git merge --squash <task-branch> && git commit` and `git push origin develop`. If the push is rejected because `develop` moved, fetch and repeat from the merge step. Landing while a release PR is open is allowed: the new commit joins that batch and CI re-runs on it.
   Split landing (speed review 2026-10-17): when other tasks depend on a large logic task (a data `-A` task on its schema, a later logic task on its rules), land it in two squash commits, each reviewed and gated with its own `Review:` and `Gate:` lines: first the schema, validation and their tests (`<ID> (schema): ...`), then the behaviour (`<ID>: ...`). The schema commit counts as done for any `Depends on` that names only the schema, so the dependent task (and the daily dispatch) can start from `develop` as soon as it lands. Each commit carries its own `Timing:`, `Blocked-by:` and `AgentMap:` lines (`TASK_LOG.md` gets one row per commit, the schema row marked `(schema)`); only the second carries the `Done:` line and deletes the TASKS row.
5. The squash commit title starts with the task id; its body says what changed and ends with the `Review:` line, for example `Review: change-reviewer clean` or `Review: change-reviewer 1 high (fixed), 2 nits accepted`, and `Gate: fc.sh gate passed` (docs-only: `Gate: fc.sh docs passed (docs-only)`). No task lands without both lines. It also carries a `Timing:` and a `Blocked-by:` line ("Task timing" below), one `AgentMap: | <agent> | <task> | <tokens> / <min> | <outcome> |` line per agent run (the `docs/AGENT_MAP.md` row), and one `Done: | <id> | <title> | <lane> | <depends on> | done | <spec> | <touches> |` line (the `docs/archive/TASKS_DONE.md` row).
6. Not approved: a small change waits on its pushed task branch and goes to the user as a question in the thread; a large one waits for the user's feedback before work starts. Rows in `spec` or `idea` status and any design question that comes up mid-task go to the user.
7. Never push to `main`. Leave the task branch on the remote; the `branch-cleanup` workflow deletes it once its work is on `develop` ("Branch cleanup" below).

## Batched release to `main` (Merge & Release thread)

1. Batch: twice a day, at 11:52 and 17:52 UTC (the second before the 18:30 dispatch), and whenever the user asks, if `develop` is ahead of `main` (decision "Twice-daily releases").
2. Check each task commit in `git log origin/main..origin/develop`: task id in the title, `Review:` and `Gate:` lines in the body, task row `approved` or a decision row. A commit that fails the check is reverted on `develop` (`git revert`, pushed) and sent back to its area thread through the coordinator. A reverted task lands again as a new squash commit of its fixed branch onto the latest `develop` (merging the old branch again would be a no-op for git). Then write the logs in one commit on `develop`, pushed before step 3 so they ship in the batch: the `AgentMap:` lines into `docs/AGENT_MAP.md` (one section per task, newest first), the `Done:` lines into `docs/archive/TASKS_DONE.md`, and the `Timing:` and `Blocked-by:` lines into `docs/metrics/TASK_LOG.md` (one row per task; release wait = landed to this commit) with a refreshed Bottlenecks section (biggest waits, the dependency chains behind them, how to decouple them). This thread is the only writer of those three files, so they never conflict.
3. Open one PR `develop` -> `main` titled `Release <date>: <task ids>`; body: one line per task with its review verdict, plus anything waiting on the user. CI (`gate.yml`) runs on this PR; `gate` is a required check on `main` (branch protection: PR required, no force push or deletion), so it always runs on the release PR, docs-only batches included.
4. CI green: merge with a **merge commit** (not squash, so `develop` and `main` keep a shared history and the next batch has no conflicts). The push to `main` runs the gate again. Web deploys are paused (decision "Web deploy paused (user 2026-10-17)"); start `web.yml` only when the user asks for a deploy.
5. CI red: find the task commit that broke it (the failing suite names its area), revert it on `develop`, push (CI re-runs on the PR), and send the failure to its area thread through the coordinator. Never fix another area's code in this thread.
6. Hotfix to `main` without `develop` only with the user's word; after any change that reaches `main` outside a release PR, merge `main` back into `develop` right away so the next batch does not conflict.
7. The `balance` agent keeps running on `origin/main`, so it measures released code only; ask for a release first when a balance run needs work still on `develop`.

## Context hygiene (token use)

Cost grows with the thread's context size times its turns (user dashboard 2026-10-17: ~98% of tokens were cache reads), so keep the thread's own context small:

- Run the gate, sims and suites with output redirected to a file and read only the summary or failures (`tail`, `grep FAIL`), never the full log.
- Hand lane work to the repo agents so file reads and build output stay in their context; the thread keeps only their reports.
- Batch related commands into one tool call, and read files by section instead of whole.
- Docs-only tasks skip the local Godot gate ("Landing a task on `develop`" step 2).

### Fresh sessions (user 2026-10-17, "Restart often")

After about 3 landed tasks, or sooner when its context gets long, an area thread stops at a task boundary (nothing uncommitted, no builder running) and hands over:

1. Write `docs/handover/<area>.md` (for example `platform.md`), replacing the previous one: queue position (next task ids), any in-flight branch and its state, open questions, cards still waiting on the user (question and options), and routines or watches it owns. Keep it short; everything else is in the repo docs.
2. Commit it straight to `develop` (`<AREA>-HANDOVER: ...`, docs-only, `Review: none (handover)` and `Gate: fc.sh docs passed (docs-only)`; no AgentMap or Done lines; the release check accepts it under decision "Fresh area sessions").
3. Send the coordinator a cross-session message: the area, the handover path and the commit.
4. The coordinator starts a fresh session for that area under a new project post, pointed at the handover, and resolves the old thread. The new thread reads the handover first and carries on.

## Task timing

Goal (user 2026-10-17): see where time goes, mainly idle time and dependence between steps; tokens are secondary. Each area thread notes timestamps as it works (`date -u +%H:%M` at each step) and puts two lines in the task's squash commit body:

- `Timing: start=<HH:MM>Z landed=<HH:MM>Z work=<min> (build <min>, review <min>, rework <min>, conflicts <min>) wait=<min> (approval <min>, dependency <min>, ci <min>, merge <min>) gates=<count> mode=<dispatch mode> agents=<agents run> tokens=<subagent total, from the agents' reports>`
- `Blocked-by: <task id, "user approval HH:MM-HH:MM", "merge of <id>", or none>`

Work is time the thread is building, reviewing, gating or fixing; wait is time the task is ready or blocked and nobody works on it, by cause. Estimates are marked `~`. The Merge & Release thread adds the release wait and compiles the lines into `docs/metrics/TASK_LOG.md` at each batch.

## Branch cleanup

Cloud sessions cannot delete remote branches (the proxy refuses ref deletes). `.github/workflows/branch-cleanup.yml` runs daily at 19:37 UTC and on request (**Actions > branch-cleanup > Run workflow**, optional dry run): it deletes every branch except `main` and `develop` that has no open PR, has had no commit for 6 hours, and is either the head of a merged PR or already contained in `develop`; branches with work not on `develop` are kept and listed in the run summary. Do not turn on the repository setting "Automatically delete head branches": the release PR's head is `develop`, so it would be deleted at every release.

## Scheduled routines

- Daily planner pass (economy mode, 18:30 UTC): pick the next `approved` task whose `Depends on` rows are `done` and whose `Touches` do not overlap another area's unmerged task branch, and hand it to its area thread, which lands it on `develop`. The routine wakes the Platform & Workflow thread (no new thread per run), which only dispatches: it sends the task to the area thread's session as a cross-session message (thread found by title in the project's thread list), or keeps it when the area is Platform & Workflow. If the area has no live thread, it asks the coordinator session to open that area thread (already approved by the user) and pass the task on. The area thread stops at any design question. If the Platform & Workflow thread is closed, recreate the routine bound to the new one. Balance and housekeeper routines are added later on request.
- Design review check: the Design thread keeps a routine bound to itself at 12:20 and 18:20 UTC, after each release, that starts a design review when one is due ([design-review.md](design-review.md)).
- Release batch: the Merge & Release thread keeps two routines bound to itself, 11:52 UTC ("Midday release batch") and 17:52 UTC ("Daily release batch", before the 18:30 dispatch).
- Routines follow the same approval rule as threads.

## Environment

- Godot 4.7.2 and PowerShell 7 come from the environment setup script (copy: `/mnt/project-files/cloud-setup/environment-setup-script.sh` in the project files); `.claude/hooks/session-start.sh` installs them if the script did not and no-ops otherwise. `github.com` must stay allowed in the environment's network policy.
- Web build: `.github/workflows/web.yml` builds and deploys `main` only on request while web deploys are paused until a playable build (user 2026-10-17; the after-release run and the 19:07 UTC fallback are off); PRs build it only when they touch the web export. When the user says "deploy", the coordinator relays it to the Platform & Workflow thread, which starts the workflow on `main` (`workflow_dispatch`) and replies with the run link. Setup and limits in [web.md](web.md).
- No interactive play. For UI work, render frames under Xvfb and attach them to the thread reply: `xvfb-run -a -s "-screen 0 1280x720x24" godot --rendering-driver opengl3 --path . <scene> --write-movie out.png --fixed-fps 10 --quit-after 30`. Playtesting feel, input and audio stays on desktop.
