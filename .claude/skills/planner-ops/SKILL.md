---
name: planner-ops
description: Operating rules for the planner - the dispatch-mode table (standard / economy / sprint) that sets effort and model on every Agent call, plus pointers to the environment's front-desk and alert rules. Load together with the planner skill before dispatching agents or answering a relayed message.
---

# Planner operations (both environments)

## Front desk and alerts

Environment-specific: `docs/workflow/desktop.md` (front-desk session found with `ListAgents`, `ALERT` lines) or `docs/workflow/cloud.md` (project chat, thread replies). Differences and switching: `docs/workflow/README.md`.

## Dispatch modes (user 2026-10-17)

The user switches mode with "mode standard", "mode economy" or "mode sprint" (desktop: typed in the front-desk session and relayed; cloud: typed in the project chat). Ack with the active mode on read. The mode persists until changed and applies to every later dispatch; agents already running are untouched. Pass `effort` explicitly on every Agent call per the table (this table is the explicit instruction for setting effort) and `model` where the table names one; agent frontmatter pins only the model (decision "Agent models"). Current mode: **economy** (mode trial window 15:45-18:45 UTC, switched by the Platform routine; before it **standard** from user 2026-10-18 12:42 UTC; then back to split: economy for build lanes, standard for specs, design reviews and the final change-reviewer pass; `docs/metrics/usage/MODE_TRIAL.md`; earlier: split, user 08:19 UTC usage cutbacks; cloud.md "Usage caps"; desktop: record changes in `docs/AGENT_MAP.md` Active section; cloud: this line is the record, and scheduled routines always run in economy).

| Agent | standard (default, quality baseline) | economy (fewest tokens) | sprint (least wall-clock time) |
| --- | --- | --- | --- |
| design-brainstorm | high | medium | medium |
| logic-builder | high for new systems, medium for fixes | medium; model sonnet for fixes and small tasks, opus only for new engine systems | medium |
| change-reviewer | medium | low, only at commit gates | medium |
| balance | medium (sonnet) | low (sonnet) | low (sonnet) |
| asset-builder | low (sonnet) | low (sonnet) | low (sonnet) |
| housekeeper | medium | low, only when asked | skip |

- Context hygiene (all modes, both environments; details in cloud.md "Context hygiene"): gate, sim and suite output goes to a file and only its summary or failures are read; lane work runs in repo agents; related commands are batched into one call.
- Economy rules: run lanes serially unless tasks are independent and small; resume an existing agent via SendMessage instead of spawning a fresh one; give agents exact file paths and spec sections; no design-ahead work unless the user asks.
- Sprint rules: maximum parallel lanes (desktop: separate worktrees; cloud: area threads working at the same time, one task each; no extra threads without the user's approval) (soft warning above 3 still applies: CPU contention, see `docs/EFFICIENCY.md`); design-ahead always on; `check_project.ps1 -Fast -Quiet` per task and the full check only before a milestone merge; batch review fixes into one builder pass; skip balance runs until the user asks.
- All modes keep the standing gates: user confirmation for design/gameplay decisions, change-reviewer before each commit (cloud: on the whole task branch before it lands on `develop`, verdict as a `Review:` line in the squash commit), `check_project.ps1 -Fast -Quiet` clean, IDE Problems 0 (desktop only), README/docs updated. Modes change only effort, model and parallelism. A session's own model and effort are set by the user (`/model`, `/effort`).
