# Planner scripts (desktop, batched gate)

The build session's helper scripts, moved here from the planner's own folder by SKILLS1 (`docs/specs/AGENTROLES1.md`) so they have review and history. They find each other from their own folder; worktree paths are the desktop layout (`C:/dev/tactical-rpg-lane1..3`, `-stage`, `-plan`).

| Script | Does |
| --- | --- |
| `lane.sh <1-3> <task>` | Starts a task branch `claude/<task>` from `staging` in a lane worktree |
| `pgate.sh` | A builder's partial gate in its worktree: check, the touched crates' tests, layers, docs |
| `landtask.sh <n> <task> <msg file>` | Commits a lane's work and lands it (`landlane.sh`) |
| `landlane.sh <worktree> <task>` | Cherry-picks the lane's commit onto `staging`, resolves `.md` and `schema.json` conflicts, then `stage.sh` |
| `stage.sh <worktree> <name>` | Rebases onto `staging`, partial gate, fast-forwards `staging` |
| `flush.sh` | The full gate on `staging`, then pushes it to `develop`; logs gate times to `$AEGIS_PLANNER_LOG/gate_times.tsv` (default the temp dir) |
| `cleanup.sh lane<n>` | Detaches a lane and deletes its task branches, keeping the build cache |
| `heavy.sh` | Sourced: one heavy run at a time on the machine, at below-normal priority (CPU1) |
| `land.sh` | The older single-step landing, kept for reference |
| `resolve_docs.py`, `sch_merge.py` | Conflict helpers for `.md` files and the schema |

Run `landlane.sh` and `landtask.sh` from a checkout that already has `tools/planner/` landed (the main checkout or the stage worktree), not from the lane being landed: `landlane.sh` checks out `staging` inside that lane, which removes its own copy of the scripts if `staging` lacks them.
