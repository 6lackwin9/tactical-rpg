# Task queue (planner-owned; agents read only their task)

Statuses: `idea` (brainstorm only), `spec` (spec written, awaiting approval), `approved`, `in-progress`, `review`, `done`. Lanes: `logic`, `asset`, `design`, `balance`. A task may start when everything in `Depends on` is `done` (schema-first tasks release their asset lane as soon as the schema slice is merged).

| Id | Title | Lane | Depends on | Status | Spec | Touches |
| --- | --- | --- | --- | --- | --- | --- |
| B1 | Stance and rest rework: a stance-defining skill does not pay a switch; Rest enters Tactical | logic | slice A | done | docs/specs/B1.md | skill_def, stance tests (done) |
| C1 | Orb and tome off-hand types, wpn_orb, priest loses shield, caster gating on main-hand staff or wand (C1-L schema/effects hook, C1-A1 gating + proficiency + wpn_tome retype, C1-A2 wpn_orb + loadouts, needs C1-L) | logic (schema), asset (items, classes) | B1 not required | in-progress (C1-L, C1-A1 dispatched 2026-10-16) | docs/specs/C1.md | items, passives, gear_budget, combatant, content_schema/catalog (item part); items.json |
| D1 | weaken status (registry), intimidate family, riposte tier change (AP 1/2/3; D1-L logic, D1-A assets needs D1-L) | logic (status), asset (skills) | slice A | in-progress (D1-L dispatched 2026-10-16) | docs/specs/D1.md | status_registry/effects, skill_def, content_schema/catalog (skill part), combatant, plan_entry; skills.json |
| M2C2B | Class skill content: families, advanced families, class paths, unit migration (phase A independent; hex/mage rows need C1-A1; warrior riposte/intimidate rows need D1-A; four advanced skills need M2C2B-L) | asset | B1, C1, D1 | spec | docs/specs/M2C2B.md | data/catalog skills/classes/units (asset lane only) |
| M2C2B-L | Engine gaps for advanced skills: X1 status on area attacks, X2 DEFEND skill may grant taunt | logic | slice A | spec | docs/specs/M2C2B.md (section Engine gaps) | area_attack_command, defend_command, skill_def, content_schema; |
| AI1 | AI use of disengage, sweep and cleanse-only skills | logic | M2C2B | idea | - | see spec |
| BAL1 | Balance rerun (duel, tier, levels) against 40-60% | balance | M2C2B | idea | - | see spec |
| M3 | Progression data model + save schema (S7, S13) | logic | M2C2B | idea | - | see spec |
