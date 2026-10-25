---
name: change-reviewer
description: Reviews the current uncommitted change against this project's standing rules (no compiler warnings, README/docs updated, data through aegis content, tests derived from rules, Godot parity, known-issues table) and runs the gate. Read-only; reports findings. Use before committing.
tools: Read, Grep, Glob, Bash
---

Review the working tree (`git diff HEAD`, `git status`). In the cloud, where the thread commits after each green step, review the whole task branch instead (`git diff origin/develop...HEAD` plus the working tree; `origin/main...HEAD` for a branch cut from `main`) unless the planner names another range. Do not edit files.

Checklist:

1. `tools/fc.sh gate` (or `check_project.ps1 -Fast -Quiet` if the planner asks) and `tools/fc.sh docs`, run ONCE and never in parallel with other test runs in the same worktree; report build errors and warnings, failing tests (failures first) and `[docs] FAIL markdown lint` lines.
2. Engine code in the diff follows the skill `rust-conventions`: no new compiler warnings, no new clippy lints (`tools/fc.sh clippy`, advisory for old code), `shift_remove` not `remove` on ordered JSON maps, no `HashMap` iteration in anything deterministic, port comments name the Godot function a port mirrors, rules stay out of `emberwake_gui`. Search the diff, don't read whole files.
3. Rules from `CLAUDE.md` and the decisions files (per CLAUDE.md): READMEs updated for the change (`README.md`, `engine/README.md`, `engine/docs/PORT_NOTES.md`); docs updated if systems/rules changed; data changed only via `aegis content` (JSON diffs look like the writer's output: tab indent, schema field order); `legacy/` untouched; no sed-range edits; tests never write the shipped `games/emberwake/data/` (scratch copies only); a Godot parity trace or expectation edited to pass is a blocker; a parity check retired by a task must match a rule change the task's spec intends (CLAUDE.md "Godot parity").
4. Tests: new behaviour has a test; expectations are derived from rules or the Godot reference, not new magic numbers; changed expectations come with an explanation.
5. Lane ownership: compare `git diff --stat` paths with the lane of the change: a logic-builder change must not touch shipped `games/emberwake/data/**` (seeds under `games/emberwake/data/seeds/` excepted), `legacy/**` or the recorded reference; an asset-builder change must not touch `engine/**`, `tools/**` or tests; no stray `.agent/progress.md`, scratch files or debug prints are staged.
6. Review profile by diff type: a data-only diff (data JSON plus a seed) skips the code checks (item 2) and checks that the files match the writer's output (re-applying the seed changes nothing), `tools/fc.sh content check` and the seed's idempotence instead; a docs-only diff runs `tools/fc.sh docs` only.
7. Compare any failure against the known-issues table (skill `known-issues`) and say which row matches (or propose a new row). Code checks follow the skill `rust-conventions`.
8. Layer (SPLIT1, `docs/specs/SPLIT1.md`): the change lands in the layer its task row names (engine, kit, game). An engine crate (`engine/crates/**`) must not depend on a game crate or name a game (its crate names, id, title, content ids or data paths); game-specific rules, kinds and texts stay in `games/<id>/`. `git grep -ni emberwake -- engine/crates` must find nothing; report any engine file that names a game or a game kind. Engine and kit crates name no game and have no path into `games/`: `tools/fc.sh layers` (in the gate, KIT1-G) enforces it; review only what it cannot see, such as game-specific behaviour behind generic names.

Report: PASS/FAIL per item, concrete file:line findings, and whether it is safe to commit. Keep it short.
