# Playtests (PLAYTEST1)

Shared playtest material: recordings the user made, agent repros and one-seed scenarios. The format and the rules are in `docs/specs/PLAYTEST1_L.md`; the code is in `scripts/playtest/` (logic) and `tools/replay.gd` (headless tool).

## Layout

| Folder | Who writes it | Content |
| --- | --- | --- |
| `inbox/<id>/` | the user (Scene Runner "Copy to repo", then push on any branch) | `<id>.json` (the recording) and its note images `note_<n>.jpg` |
| `repro/` | an agent, only through `tools/replay.gd --variant=<patch> --write=<out>` | `<id>.json` recordings with `origin: agent`, at most one per question |
| `scenarios/` | an agent | `<name>.json` = `{format: "playtest_scenario", version: 1, id, title, why, encounter, seed}` |

Recordings are written first to `user://playtests/<id>.json` (images in `user://playtests/<id>/`) in debug builds only; the repo folders hold copies that someone chose to share. The planner removes handled inbox folders in a cleanup pass (git history keeps them).

## One-scenario rule

One scenario file is one Scene Runner entry: one encounter with one chosen seed (picked by the agent from its seed sweep as representative or interesting). No other configuration in v1. Keep human-facing material small: the user's own recordings, at most one agent repro per question, and one scenario per question.

## Headless tool

`tools/fc.sh replay <file or id> [args]` (wraps `godot --headless --path . -s tools/replay.gd -- <file> [args]`):

- no flags: replay every battle and compare each round (`match`, `diverge`, `plan_broken`, `unverified`); stale content only warns
- `--battle=K`, `--json`
- `--variant=<patch> [--then=stop|recorded] [--write=<out>]`: replay with changed plans for one round; `DIFF` lines per round
- `--seeds=<a,b|a..b> [--fallback=stop|<preset>]`: seed sweep (max 200), a debugging aid, not a balance tool
- `--trail-report`: `FLAG` lines for UI trail moments (dead or repeated clicks, long hovers, refused confirms, errors, undo churn, open / close problems, leaks, asset fallbacks)

Exit 0 all match, 1 divergence or broken plan, 2 load or validation error.
