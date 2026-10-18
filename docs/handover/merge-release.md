# Merge & Release handover (2026-10-18 evening)

Wind-down into interactive mode (user 2026-10-18 16:28 UTC, decision "Interactive mode"). Releases now happen only when Tushar asks (`release-now` skill); nothing is scheduled.

## State

- Last release: PR #40 (evening batch 2026-10-18, develop cd55474) into `main` with a merge commit. Previous: #39 midday (e4f9289).
- `PATCH_NOTES.md` marker: `<!-- last: c5d9544 -->`, covering everything through the evening batch. Write it per release and on request only, never per landing (decision "Patch notes per release", 16:10 UTC).
- Shared logs (AGENT_MAP, TASKS_DONE, TASK_LOG with Bottlenecks) are written through the evening batch (02785bb).

## Routines (bound to `session_01KhQLgrvAgUunc9TcJKTHzK`; disabled, not deleted)

- Midday release batch 11:52 UTC: `trig_01Jwiub946w5QAEDRw4RSYdR` (disabled 17:55).
- Daily release batch 17:52 UTC: `trig_01AKbWyrt3ieUkJGSwQCKC7R` (disabled 17:55).
- When Tushar leaves interactive mode, a new Merge & Release session re-creates both bound to itself and deletes these.

## How a release runs (cloud.md "Batched release to `main`")

1. `git fetch origin main develop`; list `origin/main..origin/develop`. Task commits need `Review:` and `Gate:` lines (handover-only docs commits may skip them).
2. One `RELEASE-LOGS` commit on develop: `AgentMap:`, `Done:`, `Timing:` and `Blocked-by:` lines into the three log files, plus a Bottlenecks section. Map areas by the `Claude-Session` trailer (session ids are in each `docs/handover/<area>.md`). Keep `type=` and `tokens=`. Some threads write `landed=` later than the real push: use the commit time instead.
3. `PATCH_NOTES.md` via `.claude/skills/patch-notes-write` (`python3 tools/patch_notes.py`; set the marker by regex; "In the game" only for things a player can see now: engine work without data stays in "Every change").
4. Pin the release: push the develop sha to `claude/release-batch-vmj4rj` and open the PR from it, so later develop pushes do not restart CI. Assign 6lackwin9, subscribe, wait for CI in the background (split gate: `checks` + 8 `arena (<system>)` legs, ~10 min).
5. A red leg names the system: revert that task commit on develop, move the release branch, tell the coordinator which area owns it.
6. Merge with a merge commit (never squash, never force-push `main`/`develop`). No web deploy unless Tushar asks (deploys paused). Never touch the `engine` branch.

## Gotchas

- `gh api .../logs` redirects to a blocked host: read job logs with `mcp__github__get_job_logs` (return_content=true).
- Develop moves while you work: on a rejected push, `git pull --rebase origin develop` and push again.
- `AGENT_MAP.md` is over 1,200 lines (docs WARN only); splitting it behind an index is still open.
