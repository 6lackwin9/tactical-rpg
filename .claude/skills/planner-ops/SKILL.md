---
name: planner-ops
description: Operating rules for the MAIN planner session - the front-desk protocol (relayed decisions, acks, done notes), immediate failure alerts, and the dispatch-mode table (standard / economy / sprint) that sets effort and model on every Agent call. Load together with the planner skill before dispatching agents or answering a relayed message.
---

# Planner operations (main session)

## Front desk and alerts (user 2026-10-17)

The front-desk session is the user's interactive desk; find it with `ListAgents` (its display name can change). Send it decision requests (question, recommendation and options in one message), a one-line ack on reading each relayed message plus a short done note when finished, and short summaries when a task finishes or a review has findings. Keep `docs/AGENT_MAP.md` and `docs/TASKS.md` current: the desk reads them for status. Anything needing a permission prompt, hard to reverse, or editing `CLAUDE.md` / `.claude/` is still confirmed in this terminal.

Hard failures get an immediate one-line alert, without waiting for a done note: `ALERT <task>: <what failed>, <what you are doing about it>`. Triggers: an agent error, crash or usage limit; `check_project` failing twice on the same task; a merge conflict; a blocked task or stalled agent; anything that would otherwise only be logged. Routine fixes a builder resolves on its own stay in the done notes.

## Dispatch modes (user 2026-10-17)

The user switches mode with "mode standard", "mode economy" or "mode sprint" (typed in the front-desk session, relayed here). Ack with the active mode on read. The mode persists until changed and applies to every later dispatch; agents already running are untouched. Pass `effort` explicitly on every Agent call per the table (this table is the explicit instruction for setting effort) and `model` where the table names one; agent frontmatter pins only the model (decision "Agent models"). Current mode: **standard** (record changes in `docs/AGENT_MAP.md` Active section).

| Agent | standard (default, quality baseline) | economy (fewest tokens) | sprint (least wall-clock time) |
| --- | --- | --- | --- |
| design-brainstorm | high | medium | medium |
| logic-builder | high for new systems, medium for fixes | medium; model sonnet for fixes and small tasks, opus only for new engine systems | medium |
| change-reviewer | medium | low, only at commit gates | medium |
| balance | medium (sonnet) | low (sonnet) | low (sonnet) |
| asset-builder | low (sonnet) | low (sonnet) | low (sonnet) |
| housekeeper | medium | low, only when asked | skip |

- Economy rules: run lanes serially unless tasks are independent and small; resume an existing agent via SendMessage instead of spawning a fresh one; give agents exact file paths and spec sections; no design-ahead work unless the user asks.
- Sprint rules: maximum parallel lanes in separate worktrees (soft warning above 3 still applies: CPU contention, see `docs/EFFICIENCY.md`); design-ahead always on; `check_project.ps1 -Fast -Quiet` per task and the full check only before a milestone merge; batch review fixes into one builder pass; skip balance runs until the user asks.
- All modes keep the standing gates: user confirmation for design/gameplay decisions, change-reviewer before each commit, `check_project.ps1 -Fast -Quiet` clean, IDE Problems 0, README/docs updated. Modes change only effort, model and parallelism. This session's own model and effort are set by the user (`/model`, `/effort`).
