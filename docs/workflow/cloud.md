# Cloud workflow (Claude project threads)

Use this when Claude runs in a cloud container (`$CLAUDE_CODE_REMOTE` is `true`). Shared rules: `CLAUDE.md`, skills `planner` and `planner-ops`. Differences from the desktop and the switching checklist: [README.md](README.md). Decided by the user 2026-10-17 (decisions "Cloud workflow" and "Environment split" in `docs/decisions/workflow.md`).

## Sessions

- **Project chat (coordinator)**: the front desk. The user asks there; it starts one thread per task and relays follow-ups. It does not edit the repo.
- **Task thread**: the planner for one task. It reads the task row and spec, runs the lane agents (`logic-builder`, `asset-builder`, `change-reviewer`, ...) as subagents in its own checkout, one after another, commits, pushes, opens the PR and merges it when allowed (below).
- Cross-cutting planning (milestone order, a ROADMAP restructure, a cleanup pass) gets its own thread.
- Decision requests go to the user in the thread (a decision card when offered): question, options with one-line consequences, one recommendation. Work continues on the recommended option unless the step cannot be undone.
- Failure alerts: a thread reply that leads with what is blocked and what the thread is doing about it (replaces the desktop `ALERT` line).
- Dispatch mode: the user types `mode <name>` in the project chat; it applies to later threads. Scheduled routines always run in economy mode.

## One task, one thread, one branch

1. The thread works on the branch it is given (`claude/...`); one new branch per task. No sibling worktrees and no `tools/wt_remove.ps1`.
2. Subagents work in the thread's checkout and never commit. The thread commits after each green `tools/fc.sh` step and pushes, so a recycled container or a usage limit loses nothing. `.agent/progress.md` is still kept, but the pushed branch is the real checkpoint.
3. The thread updates only its own rows in `docs/TASKS.md` (status, branch name), its line in the `docs/AGENT_MAP.md` finished-runs log, and the decision rows for answers the user gave in that thread, all inside its own PR. Git merges replace the desktop "single writer" rule.
4. Overlap: before starting, compare the task's `Touches` with the open PRs (GitHub reports merge conflicts); sequence tasks whose `Touches` overlap, as on desktop.

## Pull request, approval and merge

1. Push, then open a draft PR whose title starts with the task id; body: what changed and what to check.
2. Review before marking it ready: run the `change-reviewer` agent on the whole branch (`git diff origin/main...HEAD` plus any uncommitted change). Fix its blockers (re-run it if the fix is more than a few lines), then add a `Review:` line to the PR body with the verdict, for example `Review: change-reviewer clean` or `Review: change-reviewer 1 high (fixed in <sha>), 2 nits accepted`. A docs-only diff still gets the docs profile of the review. No PR merges without a `Review:` line, including threads that ran no other subagent.
3. Gate before marking it ready: `check_project.ps1 -Fast -Quiet` or `tools/fc.sh gate` (both work; ~3-4 min on a cloud container, not 32 s) and `tools/fc.sh docs`. There is no IDE Problems rule in the cloud. The GitHub Actions workflow `.github/workflows/gate.yml` runs the same gate on every PR.
4. Approved means: the task row in `docs/TASKS.md` has status `approved` (set from the user's word) or the change is recorded as approved in `docs/decisions/*`. Approved work with a `Review:` line, a green gate and green CI is squash merged by the thread without asking.
5. Not approved: a small change waits on its own branch as a PR for the user's approval before merge; a large one waits for the user's feedback before work starts. Rows in `spec` or `idea` status and any design question that comes up mid-task go to the user.
6. Never push to `main` directly; delete the branch after merge.

## Scheduled routines

- Daily planner pass (economy mode): pick the next `approved` task whose `Depends on` rows are `done` and whose `Touches` do not overlap an open PR, start one thread for it, and stop at any design question. Balance and housekeeper routines are added later on request.
- Routines follow the same approval rule as threads.

## Environment

- Godot 4.7.2 and PowerShell 7 come from the environment setup script (copy: `/mnt/project-files/cloud-setup/environment-setup-script.sh` in the project files); `.claude/hooks/session-start.sh` installs them if the script did not and no-ops otherwise. `github.com` must stay allowed in the environment's network policy.
- Web build: `.github/workflows/web.yml` exports the `Web` preset on every PR (artifact `web-build`) and deploys `main` to GitHub Pages; setup and limits in [web.md](web.md).
- No interactive play. For UI work, render frames under Xvfb and attach them to the PR: `xvfb-run -a -s "-screen 0 1280x720x24" godot --rendering-driver opengl3 --path . <scene> --write-movie out.png --fixed-fps 10 --quit-after 30`. Playtesting feel, input and audio stays on desktop.
