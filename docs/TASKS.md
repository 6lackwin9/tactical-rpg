# Task queue (planner-owned; agents read only their task)

Statuses: `idea` (brainstorm only), `spec` (spec written, awaiting approval), `approved`, `in-progress`, `review`, `done`. Lanes: `logic`, `asset`, `design`, `balance`. A task may start when everything in `Depends on` is `done` (schema-first tasks release their asset lane as soon as the schema slice is merged).

| Id | Title | Lane | Depends on | Status | Spec |
| --- | --- | --- | --- | --- | --- |
| B1 | Stance and rest rework: a stance-defining skill does not pay a switch; Rest enters Tactical | logic | slice A | approved | to write |
| C1 | Orb and tome off-hand types, wpn_orb, priest loses shield, caster gating on main-hand staff or wand | logic (schema), asset (items, classes) | B1 not required | approved | to write |
| D1 | weaken status (registry), intimidate family, riposte tier change (AP 1/2/3) | logic (status), asset (skills) | slice A | approved | to write |
| M2C2B | Class skill content: families, advanced families, class paths, unit migration | asset | B1, C1, D1 | approved | docs/M2C2_SKILL_DRAFT.md (to convert) |
| AI1 | AI use of disengage, sweep and cleanse-only skills | logic | M2C2B | idea | - |
| BAL1 | Balance rerun (duel, tier, levels) against 40-60% | balance | M2C2B | idea | - |
| M3 | Progression data model + save schema (S7, S13) | logic | M2C2B | idea | - |
