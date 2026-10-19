# Desktop workflow (local Windows machine)

Use this when Claude runs in a terminal on your machine (`$CLAUDE_CODE_REMOTE` empty). Shared rules: `CLAUDE.md`, skills `planner` and `planner-ops`. Differences from the cloud and the switching checklist: [README.md](README.md).

## Sessions

- **Planner**: the main terminal session (skills `planner`, `planner-ops`). The only writer of `docs/TASKS.md`, `docs/ROADMAP.md`, `docs/decisions/*` and `docs/HISTORY.md`; commits, merges and pushes.
- **Front desk**: a second local session, the user's interactive desk; the planner finds it with `ListAgents` (its display name can change). The desk is read-mostly.
- Planner to front desk: decision requests (question, recommendation and options in one message), a one-line ack on reading each relayed message plus a short done note when finished, and short summaries when a task finishes or a review has findings. Keep `docs/AGENT_MAP.md` and `docs/TASKS.md` current: the desk reads them for status. Anything needing a permission prompt, hard to reverse, or editing `CLAUDE.md` / `.claude/` is confirmed in the planner's terminal.
- Hard failures get an immediate one-line alert to the desk, without waiting for a done note: `ALERT <task>: <what failed>, <what you are doing about it>`. Triggers: an agent error, crash or usage limit; `check_project` failing twice on the same task; a merge conflict; a blocked task or stalled agent; anything that would otherwise only be logged. Routine fixes a builder resolves on its own stay in the done notes.
- Dispatch mode: the user types `mode <name>` in the front desk, which relays it; the planner acks and records it in `docs/AGENT_MAP.md` "Active now".

## Dispatch (planner skill step 2)

1. Create worktrees for parallel lanes from clean `main`: `git worktree add ../ngp-<id> -b feat/<id>-<slug>`; one worktree per concurrent builder and one read-only `main` worktree for the balance agent. `git status` must be clean first. Give each agent its worktree path, branch and spec. Each worktree builds into its own folder (`tools/fc.sh` and `check_project.ps1` set `CARGO_TARGET_DIR=<target>-<folder>`), so its first build takes a few minutes.
2. Run `tools/overlap.sh` while worktrees are active and before every merge (it lists files edited in more than one worktree).
3. Builders never commit; they leave changes in their worktree and keep `.agent/progress.md`.
4. Review fixes go to the SAME builder via `SendMessage` while it is resumable.
5. Update `docs/AGENT_MAP.md` "Active now" on every dispatch and report.

## Ship (planner skill step 3)

1. Run `check_project.ps1 -Fast -Quiet` (~30 s warm: build every crate, run every test) and `tools/fc.sh docs`; the IDE Problems tab must show 0 errors and warnings (markdownlint compliant, see `.agents/rules/engine_workflow.md` section 6). Never pipe `fc.sh docs` into `tail` before `&& git commit`.
2. If clean: commit in the worktree with the attribution trailer from the session's system reminder, fast-forward merge to `main`, push.
3. Remove the worktree and branch: `powershell -NoProfile -File tools/wt_remove.ps1 ../ngp-<id> feat/<branch>` (OneDrive marks folders read-only; plain `git worktree remove` can fail).

## Tools only on desktop

- Rust: rustup with the toolchain in `engine/rust-toolchain.toml`; build output in `C:/Users/tusha/AppData/Local/Temp/aegis-target` (`engine/.cargo/config.toml`, outside OneDrive). Godot 4.7.2 (`C:/Program Files/Godot/godot.exe` or `$GODOT`) is needed only for `tools/fc.sh parity` (the frozen reference in `legacy/`).
- Scene Runner: `run.bat` pulls the latest `main`, builds the engine (release) and opens `aegis-gui screen=runner` (`run_dev.bat` or `run.bat dev`: latest `develop` from the sibling worktree `<checkout>-dev`, created on first use; it never switches branches; it launches without pulling when the checkout has local changes or is offline); from PowerShell `Start-Process .\run.bat`. Never kill processes you did not start (other agents run tests and sims).
- IDE: rust-analyzer for the engine; a Godot language server left over from the Godot build reports false errors on `engine/parity/godot/*.gd` (they run inside `legacy/`): close any Godot editor open on the old root. Smart App Control was turned off on 2026-10-19 because it blocked freshly built engine binaries (skill `known-issues`).
- Measured costs: 3 builders plus a reviewer on one machine slowed the gate to ~10 min (`docs/EFFICIENCY.md`); parallel full-suite runs can collide on `user://` sandbox names.
