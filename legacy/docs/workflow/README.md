# Workflow by environment

The game, the lanes, the agents, the gate steps and every decision are the same in both environments. Only how sessions are organised, where work runs and how changes reach `main` differ. Read this file, then the file for the environment you are in. Nothing else in the repo needs to change when you switch.

## Which environment am I in?

| Check | Desktop | Cloud |
| --- | --- | --- |
| `echo $CLAUDE_CODE_REMOTE` | empty | `true` |
| Where you talk to Claude | a terminal on your machine | a Claude project (project chat plus standing area threads) |
| Read next | [desktop.md](desktop.md) | [cloud.md](cloud.md) |

## What differs

| Topic | Desktop | Cloud |
| --- | --- | --- |
| Planner | the main terminal session, one for everything | each area thread plans its area's tasks one at a time; the project chat (coordinator) routes asks and opens a new thread only with the user's approval |
| Front desk | a second local session, found with `ListAgents` | the project chat |
| Parallel builders | subagents in sibling worktrees (`../ngp-<id>`) | one area thread per focus area, each in its own container; the six lane agents run one after another in that thread's checkout (the thread writes only planner docs itself, see cloud.md "Lanes inside an area thread") |
| Branch | `feat/<id>-<slug>`, created by the planner | a `claude/...` branch per task from the latest `develop`; feature branches are free; the squash commit on `develop` starts with the task id |
| Commits | planner commits after review; builders never commit | the thread commits and pushes after each green step; its subagents never commit; `change-reviewer` reviews the whole task branch before it lands on `develop` and its verdict goes in the squash commit (`Review:` line) |
| Merge to `main` | planner fast-forwards locally and pushes | area threads squash tasks into `develop` (no PR); the Merge & Release thread merges one batched `develop` -> `main` PR (merge commit) after CI |
| Writer of `docs/TASKS.md`, ROADMAP, decisions, HISTORY | planner only | each area thread for its tasks' rows and the decisions you gave in that thread, inside the task's commit |
| `docs/AGENT_MAP.md` "Active now" | kept current by the planner | not used (the project's thread list is live); the finished-run log and `docs/archive/TASKS_DONE.md` are written only by the Merge & Release thread at release time, from each task commit's `AgentMap:` and `Done:` lines |
| Gate | `check_project.ps1 -Fast -Quiet` (~32 s) and IDE Problems tab 0 | `tools/fc.sh gate auto` (the systems the change touches) and `tools/fc.sh docs`; no IDE rule; CI runs every system as a parallel job once per batched release PR (`gate`, a required check on `main`) |
| Godot | `C:/Program Files/Godot/godot.exe` | `/opt/godot/4.7.2/godot` on PATH (environment setup script, repo hook as fallback) |
| Playing the game | `run.bat` Scene Runner | not possible; headless or Xvfb screenshots only |
| Failure alerts | `ALERT <task>: ...` line to the front desk | a thread reply that leads with what is blocked |
| Dispatch mode (`mode standard / economy / sprint`) | typed in the front desk, relayed | typed in the project chat; scheduled routines run in economy |
| Lost work risk | partial work stays on disk | anything not pushed is lost when a container is recycled |

## Switching environments

The repo is the handover: state lives in `docs/TASKS.md`, `docs/AGENT_MAP.md`, the decisions files and pushed branches, never only in a session.

### Desktop to cloud

1. Finish or pause each running agent; have it write `.agent/progress.md`.
2. Commit work in progress on its feature branch (a WIP commit is fine) and push every branch you want to continue.
3. In `docs/TASKS.md`, write the branch name into the Status cell of each `in-progress` row, for example `in-progress (feat/dlg1-l, commit 3 open)`.
4. Set `docs/AGENT_MAP.md` "Active now" to `No agents running (handover to cloud <date>)` and list the open branches there.
5. Commit and push these docs to `main` (docs only, gate green), then close the local sessions.
6. In the project chat, ask for the next task; each open branch goes to its area thread.

### Cloud to desktop

1. Ask the project chat to wind down: each running thread pushes its branch and writes its `docs/TASKS.md` row with the branch name and next step.
2. Land what is approved and gated on `develop` and release it to `main`; leave the rest on pushed task branches.
3. Locally: `git pull` on `main`, then `git fetch` the open branches; create a worktree per branch you continue (`git worktree add ../ngp-<id> <branch>`).
4. Start the planner session (skills `planner`, `planner-ops`, [desktop.md](desktop.md)) and the front desk; set "Active now" in `docs/AGENT_MAP.md`.
5. Continue from the pushed task branches; the desktop planner may fast-forward a branch to `main` instead of using `develop`.

Both directions keep the same rules: one branch per task, `main` always green, approved work merges without asking, unapproved work waits for the user.
