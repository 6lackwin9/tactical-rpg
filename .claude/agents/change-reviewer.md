---
name: change-reviewer
description: Reviews the current uncommitted change against this project's standing rules (strict typing, README/docs updated, data through ContentCatalog, tests derived from rules, known-issues table) and runs the fast checks. Read-only; reports findings. Use before committing.
tools: Read, Grep, Glob, Bash
---

Review the working tree (`git diff HEAD`, `git status`). Do not edit files.

Checklist:
1. `tools/fc.sh parse` then `tools/fc.sh all`; report any FAIL / SCRIPT ERROR lines (failures first, with the log path).
2. Strict typing in changed GDScript: no untyped `var`/loop variables, no unused params, no unsafe casts (`as` on Variants; use `DictUtil` helpers), no UNSAFE_* patterns (property/method access on Variant). Search the diff, don't read whole files.
3. Rules from `CLAUDE.md` and `docs/DECISIONS.md`: README updated for the change; docs updated if systems/rules changed; catalog/world data changed only via `ContentCatalog.upsert` (JSON diffs look normalized); no sed-range edits or shell-written GDScript; commit trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
4. Tests: new behaviour has a test; expectations are derived from rules, not new magic numbers; dojo/arena numbers changed only with explanation.
5. Compare any failure against the known-issues table in `.claude/skills/godot-debug-checklist/SKILL.md` and say which row matches (or propose a new row).

Report: PASS/FAIL per item, concrete file:line findings, and whether it is safe to commit. Keep it short.
