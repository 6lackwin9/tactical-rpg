# Usage tracking

Owner: the Platform & Workflow thread (user 2026-10-18). Tushar pastes his project usage report every ~3 hours (the coordinator reminds him); the Platform thread replies with the delta only and keeps the history here for trends.

- `reports/<time>.csv`: each pasted report, one row per thread (title, tokens, output, cache writes).
- `HISTORY.md`: one row per report (totals and the heaviest threads since the previous one).
- `DELTA_<date>.md`: per-thread deltas, cost weights and cutback proposals.
- `EFFICIENCY_<date>.md`: report usage per thread session against its commits' `Timing:` lines (tasks, task type, tokens per agent kind); the residual is the thread's own use.
- `sessions.json`: thread sessions (id, title, start, end); commits map to sessions through the `Claude-Session` trailer. `pins.json`: report rows pinned to sessions when titles repeat.
- `MODE_TRIAL.md`: the standard vs economy baseline windows (2026-10-18).
- `TRIAL_SONNET.md`: the Sonnet vs Opus trial on Assets & Audio (window, tasks, per-report deltas).
- `efficiency.py`: `python3 docs/metrics/usage/efficiency.py . docs/metrics/usage/reports/<file>.csv <report time ISO>`.

On each new report: add its CSV, add the new and rotated sessions to `sessions.json`, append a `HISTORY.md` row, rerun `efficiency.py`, and reply with the delta.
