# Task queue (planner-owned; agents read only their task)

Statuses: `idea` (brainstorm only), `spec` (spec written, awaiting approval), `approved`, `in-progress`, `review`, `done`. Lanes: `logic`, `asset`, `design`, `balance`. A task may start when everything in `Depends on` is `done` (schema-first tasks release their asset lane as soon as the schema slice is merged).

| Id | Title | Lane | Depends on | Status | Spec |
| --- | --- | --- | --- | --- | --- |
| B1 | Stance and rest rework: a stance-defining skill does not pay a switch; Rest enters Tactical | logic | slice A | approved | to write |
| C1 | Orb and tome off-hand types, wpn_orb, priest loses shield, caster gating on main-hand staff or wand (C1-L schema/effects hook, C1-A1 gating + proficiency + wpn_tome retype, C1-A2 wpn_orb + loadouts, needs C1-L) | logic (schema), asset (items, classes) | B1 not required | spec | docs/specs/C1.md |
| D1 | weaken status (registry), intimidate family, riposte tier change (AP 1/2/3; D1-L logic, D1-A assets needs D1-L) | logic (status), asset (skills) | slice A | spec | docs/specs/D1.md |
| M2C2B | Class skill content: families, advanced families, class paths, unit migration (phase A independent; hex/mage rows need C1-A1; warrior riposte/intimidate rows need D1-A; four advanced skills need M2C2B-L) | asset | B1, C1, D1 | spec | docs/specs/M2C2B.md |
| M2C2B-L | Engine gaps for advanced skills: X1 status on area attacks, X2 DEFEND skill may grant taunt | logic | slice A | spec | docs/specs/M2C2B.md (section Engine gaps) |
| AI1 | AI use of disengage, sweep and cleanse-only skills | logic | M2C2B | idea | - |
| BAL1 | Balance rerun (duel, tier, levels) against 40-60% | balance | M2C2B | idea | - |
| M3 | Progression data model + save schema (S7, S13) | logic | M2C2B | idea | - |
