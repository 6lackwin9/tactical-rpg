# Specs

One file per task: `docs/specs/<task-id>.md`, written by the `design-brainstorm` agent (or the planner), consumed by the `logic-builder`, `asset-builder` and `balance` agents. Specs are append-only history once a task is done (mark the status, do not delete). In the cleanup pass after a task is done, its spec moves to `docs/archive/specs/` with a first line `Archived <date>, task done/superseded` and the TASKS row points there; rules that later passes still cite are copied verbatim (same section numbers) into a shared spec such as `docs/specs/FORGE_SHARED.md`.

## Template

```markdown
# <task-id>: <title>

Status: proposed | approved | in-progress | done
Depends on: <task ids>
Lane: logic | asset | both
Touches: files or areas this task edits (e.g. `scripts/combat/status/*`, `content_schema.gd`, `data/catalog/skills.json`); the planner runs tasks in parallel only if their Touches do not overlap

## Goal
One paragraph.

## Schema contract (logic lane)
Fields, allowed values, defaults, validation rules. Schema-first: yes/no.

## Rules (logic lane)
Behaviour, formulas, edge cases, acceptance checks (rule-derived).

## Assets (asset lane)
Table of ids, numbers, links to other assets. Authoring rules used (tier defaults etc.).

## Out of scope
What not to touch.

## Open questions
Only genuine design choices for the user.
```
