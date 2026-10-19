---
name: known-issues
description: Known-issues table for the Aegis engine project (symptom, cause, fix). Load it as soon as anything fails (build error, test FAIL, parity divergence, tool error, flaky run, git/worktree trouble) and before debugging from scratch; add a row for every new recurring cause.
---

# Known issues

Look the failure up here first. Only debug from scratch if nothing matches, then add the new cause as a row. Fixed or obsolete rows move to `docs/archive/KNOWN_ISSUES_ARCHIVE.md`. The Godot build's table (GDScript, class cache, scenes) is frozen with it in `legacy/.claude/skills/godot-known-issues/SKILL.md`.

| Symptom | Cause | Fix |
| --- | --- | --- |
| Markdown table formatting warning (`MD060`) | Table delimiter `\|---\|---\|` lacks padding spaces | Pad with spaces: `\| --- \| --- \|` |
| Markdown heading/list blanks (`MD022`, `MD032`) | Missing blank line immediately before/after `#` heading or list | Add exactly 1 blank line above and below |
| Markdown multiple blank lines (`MD012`) | Consecutive blank lines in `.md` file | Reduce to single blank line |
| `sed` line edits corrupt blocks | Line numbers shifted | Never use sed ranges; use Edit |
| A commit lands although `tools/fc.sh docs` printed `FAIL` | Piping `fc.sh docs` into `tail` before `&& git commit` takes the exit code of `tail` | Capture the output and test it, or run `fc.sh docs` on its own first |
| An append-only table (`docs/archive/TASKS_DONE.md`, HISTORY, decisions) shrinks after a merge | A conflict was resolved by taking one side | Keep both sides' rows; compare `wc -l` with the base before committing |
| (desktop) `git worktree remove` fails with "Permission denied" | OneDrive marks folders read-only and Git for Windows cannot remove them | `powershell -NoProfile -File tools/wt_remove.ps1 ../ngp-<id> feat/<branch>`; if an entry survives, clear attributes on `.git/worktrees/<name>`, delete it, `git worktree prune` |
| (desktop) A build "Blocking waiting for file lock on build directory", or a worktree's tests run another worktree's code | Every checkout shares `.cargo/config.toml`'s target folder | `tools/fc.sh` and `check_project.ps1` give a linked worktree its own folder (`<target>-<folder name>`); raw `cargo` there needs `CARGO_TARGET_DIR` set the same way |
| (desktop) `os error 4551` "An Application Control policy has blocked this file", or Git Bash "Permission denied" starting an engine exe | Windows Smart App Control blocks unsigned freshly built binaries (it was turned off on 2026-10-19; this returns if it is turned on again) | The gates delete the blocked test exe and relink (4 tries); PowerShell `Start-Process` starts a GUI build that cmd or Git Bash cannot; the lasting fix is Smart App Control off |
| (Linux, cloud, CI) `rustup` tries to install `stable-x86_64-pc-windows-gnullvm`, or builds go to a `C:/Users/...` folder | `rust-toolchain.toml` and `.cargo/config.toml` are the desktop's | Set `RUSTUP_TOOLCHAIN=stable` and `CARGO_TARGET_DIR=<checkout>/target` (the cloud hook and `gate.yml` do) |
| The gate is green locally but CI (or a fresh checkout) fails a parity test with "missing ..." | The test needs a file in `engine/parity/out` that this machine recorded earlier but CI's `export.sh` run does not write (found 2026-10-19: `trace_parity` asserted traces exist; `hash.json` was opt-in) | Parity tests skip themselves when their recording is missing; files CI must check go in `export.gd`'s default `only` list. Reproduce with a fresh clone, `engine/parity/export.sh`, then `tools/fc.sh gate` |
| A parity suite passes instantly and prints "no traces" / "run engine/parity/export.sh" | `engine/parity/out` (gitignored) is missing in this checkout | `tools/fc.sh parity` (needs Godot 4.7.2; `engine/parity/export.sh` alone covers the data, unit, damage, save and resave suites) |
| `trace_parity`, `world_traces` or `journey` diverges after a change | The change altered a rule the frozen Godot build defines | Unintended: fix the engine. Intended by the approved task: retire the check it breaks (remove or narrow the suite or its recorded cases) and name it in the commit and HISTORY (CLAUDE.md "Godot parity"); never edit the traces |
| `data_source` fails after a data edit | It compares the loader on `legacy/data` with Godot's export of it, so a live `data/` edit cannot break it; the loader or the schema changed | Fix the loader (`aegis_world::source`); `engine/data/schema.json` must equal `parity/out/schema.json` |
| `aegis content upsert` refuses an entry with "names the unknown ..." and writes nothing | A reference (class, item, skill, unit) names no catalog entry; the writer rolls the file back | Upsert the referenced entry first (seeds list steps in dependency order: item families, items, skills, passives, classes, units, encounters) or fix the id |
| The first `aegis content` save of a file reorders its fields | The file is older than its schema (nine shipped files were); Godot's own save reorders them the same way | Expected once; review the diff, the values do not change |
| Key order changes in a JSON value after an edit in engine code | `serde_json::Map::remove` swaps the last key into the hole | Use `shift_remove` (engine docs: `aegis_world::json`) |
| A loader or writer test fails on `1` vs `1.0` | Godot's JSON.parse makes every number a float; whole floats come back as ints only where the schema or `_json_ints` says | Compare through the `aegis_world::json` helpers; keep int / float types as Godot writes them |
| (desktop) IDE shows "Could not find script for class ..." in `engine/parity/godot/*.gd` | The Godot language server analyses them against the root, which is no longer a Godot project | Stale: those scripts run inside `legacy/` (`engine/parity/*.sh` pass `--path legacy`); close any Godot editor open on the old root |
| `run.bat dev` or `run.bat opt <x>` says the folder has no engine | That branch is older than the switch (the Godot `develop`, an old option branch) | Branch it again from `main`; old Godot branches run from the `legacy` branch with its own launcher |
