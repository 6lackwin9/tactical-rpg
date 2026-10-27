# Desktop workflow (local Windows machine)

Use this when Claude runs in a terminal on your machine (`$CLAUDE_CODE_REMOTE` empty). Shared rules: `CLAUDE.md`, skills `planner` and `planner-ops`. Differences from the cloud and the switching checklist: [README.md](README.md).

## Sessions

- **Planner**: the main terminal session (skills `planner`, `planner-ops`). The only writer of `docs/TASKS.md`, `games/emberwake/docs/ROADMAP.md`, `docs/decisions/*` and `docs/HISTORY.md`; commits, merges and pushes.
- **Front desk**: a second local session, the user's interactive desk; the planner finds it with `ListAgents` (its display name can change). The desk is read-mostly.
- Planner to front desk: decision requests (question, recommendation and options in one message), a one-line ack on reading each relayed message plus a short done note when finished, and short summaries when a task finishes or a review has findings. Keep `docs/AGENT_MAP.md` and `docs/TASKS.md` current: the desk reads them for status. Anything needing a permission prompt, hard to reverse, or editing `CLAUDE.md` / `.claude/` is confirmed in the planner's terminal.
- Hard failures get an immediate one-line alert to the desk, without waiting for a done note: `ALERT <task>: <what failed>, <what you are doing about it>`. Triggers: an agent error, crash or usage limit; `check_project` failing twice on the same task; a merge conflict; a blocked task or stalled agent; anything that would otherwise only be logged. Routine fixes a builder resolves on its own stay in the done notes.
- Dispatch mode: the user types `mode <name>` in the front desk, which relays it; the planner acks and records it in `docs/AGENT_MAP.md` "Active now".

## Dispatch (planner skill step 2)

1. Create worktrees for parallel lanes from the latest `develop` (decision "Develop-first on desktop (user 2026-10-19)"): `git worktree add ../ngp-<id> -b claude/<id>-<slug> origin/develop`; one worktree per concurrent builder and one read-only `main` worktree for the balance agent. `git status` must be clean first. Give each agent its worktree path, branch and spec. Each worktree builds into its own folder (`tools/fc.sh` and `check_project.ps1` set `CARGO_TARGET_DIR=<target>-<folder>`), so its first build takes a few minutes. Copy the recorded Godot reference into it (`cp -r games/emberwake/parity/out ../<worktree>/games/emberwake/parity/`; gitignored, so a new worktree lacks it and the parity suites fail or skip). A planner-docs-only task needs no worktree: branch in the main checkout (`git checkout -b claude/<id>-<slug> origin/develop`) and return it to `main` afterwards (`run.bat` launches only from `main`).
2. Run `tools/overlap.sh` while worktrees are active and before every merge (it lists files edited in more than one worktree).
3. Builders never commit; they leave changes in their worktree and keep `.agent/progress.md`.
4. Review fixes go to the SAME builder via `SendMessage` while it is resumable.
5. Update `docs/AGENT_MAP.md` "Active now" on every dispatch and report.

## Ship (planner skill step 3)

Batched gate (user 2026-10-28, decision "Batched gate"): a reviewed task lands on the local branch `staging` (checked out in the sibling worktree `tactical-rpg-stage`) after a partial gate: build every crate with its tests, run the tests of the crates it touches, the layer and docs checks (`stage.sh <worktree> <name>`, the planner's helper beside `land.sh`). `staging` reaches `develop` in batches under one full gate (`flush.sh`: rebase onto `origin/develop`, `check_project.ps1 -Fast -Quiet`, `fc.sh docs`, push), at the latest before a release or a wind-down; a failed full gate is bisected within the batch. New task worktrees branch from `staging`. Planner docs-only commits still go straight to `develop`. Steps 1-2 below are the per-task full gate this replaces for task landings.

1. Run `check_project.ps1 -Fast -Quiet` (~30 s warm: build every crate, run every test) and `tools/fc.sh docs`; the IDE Problems tab must show 0 errors and warnings (markdownlint compliant, see `.agents/rules/engine_workflow.md` section 6). Never pipe `fc.sh docs` into `tail` before `&& git commit`.
2. If clean: commit on the task branch with the attribution trailer from the session's system reminder, then land it on `develop`: squash it onto the latest `origin/develop` (one commit titled with the task id, its body ending with `Review:` and `Gate:` lines as in `cloud.md` "Landing a task on `develop`") and push `develop`. Never commit to `main` or `engine`.
3. Remove the worktree and branch: `powershell -NoProfile -File tools/wt_remove.ps1 ../tactical-rpg-<task> claude/<task>`
   (plain `git worktree remove` can fail on read-only folders). It also deletes the worktree's build folder
   `<target-dir>-<folder name>` (the base comes from `.cargo/config.toml`; several GB each, WTCLEAN1). To sweep build
   folders left by older removals, run `tools/wt_remove.ps1 -Targets -WhatIf`, then without `-WhatIf`; it never touches
   the base folder or a live worktree's folder.

## Release (desktop Merge & Release)

While no cloud threads run, the planner session is also the Merge & Release role (`cloud.md` "Batched release to `main`"; decision "Develop-first on desktop (user 2026-10-19)"):

1. When the user asks for a release: `git fetch origin`, check that `main` is contained in `develop` (`git merge-base --is-ancestor origin/main origin/develop`; if not, a hotfix reached `main` outside a release: merge `origin/main` into `develop` first).
2. Open the release PR `develop` -> `main` with the GitHub CLI (`gh pr create --base main --head develop --title "Release <date>: <task ids>"`, body one line per task with its review verdict). CI (`gate.yml`, required check `gate`) runs on it.
3. CI green: merge with a merge commit (`gh pr merge --merge`), so `develop` and `main` keep a shared history; then fast-forward `develop` to the merge (`git push origin origin/main:develop`), so both are equal again. CI red: fix on a task branch, land it on `develop`, and the PR re-runs.
4. `engine` is frozen at the transition (952f214) and `legacy` at 7fa10da: neither moves.

## Tools only on desktop

- Checkout: `C:\dev\tactical-rpg` (outside OneDrive since 2026-10-19: OneDrive's reparse points broke Git branch switches, decision "Repository outside OneDrive"); worktrees go next to it (`../ngp-<id>`), never under OneDrive.
- Rust: rustup with the toolchain in `rust-toolchain.toml`; build output in `C:/Users/tusha/AppData/Local/Temp/aegis-target` (`.cargo/config.toml`, outside OneDrive). Godot 4.7.2 (`C:/Program Files/Godot/godot.exe` or `$GODOT`) is needed only for `tools/fc.sh parity` (the frozen reference in `legacy/`).
- Scene Runner: `run.bat` pulls the latest `main`, builds the engine (release) and opens `emberwake screen=runner` (`run_dev.bat` or `run.bat dev`: latest `develop` from the sibling worktree `<checkout>-dev`, created on first use; it never switches branches; it launches without pulling when the checkout has local changes or is offline); from PowerShell `Start-Process .\run.bat`. Never kill processes you did not start (other agents run tests and sims).
- IDE: rust-analyzer for the engine; a Godot language server left over from the Godot build reports false errors on `games/emberwake/parity/godot/*.gd` (they run inside `legacy/`): close any Godot editor open on the old root. Smart App Control was turned off on 2026-10-19 because it blocked freshly built engine binaries (skill `known-issues`).
- Measured costs: 3 builders plus a reviewer on one machine slowed the gate to ~10 min (`docs/EFFICIENCY.md`); parallel full-suite runs can collide on `user://` sandbox names.
