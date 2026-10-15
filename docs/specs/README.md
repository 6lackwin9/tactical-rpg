# Specs

One file per task: `docs/specs/<task-id>.md`, written by the `design-brainstorm` agent (or the planner), consumed by the `logic-builder`, `asset-builder` and `balance` agents. Specs are append-only history once a task is done (mark the status, do not delete).

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
