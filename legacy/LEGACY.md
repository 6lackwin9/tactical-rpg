# Legacy Godot build

This folder is a frozen, complete copy of the Godot version of the game, made for the move to the Aegis engine
(engine/docs/ENGINE_SPEC.md section 8.1, transition step 1). Everything the Godot project had is here: scripts, scenes,
assets, data, specs, docs, tests, tools and the agent setup. It is a standalone Godot project.

Source commit: 7fa10da0c67bce5c20fd8a615728c41b62f90c96 Release 2026-10-18 evening (#40)

| | |
| :--- | :--- |
| Copied from | `main` at the source commit above (the last Godot release), 2026-10-19 |
| Godot | 4.7.2 stable (`config/features` 4.7, Forward Plus) |
| Branch | `legacy`: a fork of `main` at the same commit, with the Godot project at the repository root |
| Data | a frozen copy of `data/` at the source commit; the engine reads the live `data/` at the repository root |

## Running it

- Open `legacy/project.godot` in Godot 4.7.2, or from the repository root:
  `"C:\Program Files\Godot\godot.exe" --path legacy` (the game) or `... --path legacy -e` (the editor).
- Do not use `legacy/run.bat`, `run_dev.bat` or `run_opt.bat` here: they fetch and fast-forward the checkout's current
  branch and create `-dev` / `-opt` worktrees, which only makes sense on the `legacy` branch.
- Checks: `legacy/check_project.ps1 -Fast -Quiet` (2026-10-19: 417 scripts, 1090 assertions, 0 errors).
- The root Godot project skips this folder (Godot prints "Detected another project.godot at res://legacy. The folder
  will be ignored."), so the two never mix.

## What it is for

- **The parity reference.** `engine/parity/export.sh`, `trace.sh` and `journey.sh` run this copy, so the Godot
  reference the engine is checked against stays exactly what was released, whatever happens to the root files.
- **A playable reference** of the released game.
- **Redevelopment if needed.** To resume Godot work, use the `legacy` branch: it is the same project at the repository
  root, with its git history, so the old workflow (`CLAUDE.md`, `docs/workflow/`) applies unchanged. Do not develop
  in this folder: keep it identical to the source commit.

## Not in this copy

- The sprite rig work in progress on `develop` (SPRITE1-A: `tools/sprites/*.py`, `docs/specs/SPRITE1.md`,
  `data/seeds/sprite1_a.gd`) was never released to `main`. It is redone in the engine instead.
- The `.godot/` import cache (Godot rebuilds it on first open).
