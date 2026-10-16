# Decisions: workflow and process

Settled; do not re-litigate without asking the user. Scope: milestone planning, agents, tests, cleanup. Rows moved unchanged from `docs/DECISIONS.md` on 2026-10-17 (CLN-10), in their original order; append new rows at the end of the table. Index and global rules: `docs/DECISIONS.md`; superseded rows: `docs/decisions/archive.md`.

| Decision | Choice | Why |
| --- | --- | --- |
| M2 slice order | M2a riposte engine -> M2d gear budget guard -> M2b items/art (validated by the guard) -> M2c roster rework with updated class suite. Art stays placeholder until the base game is done; UI/UX/art/animation/sound pass comes after | User 2026-10-15 |
| Milestone split | M2c-2a1 skill families + loadout (engine/data model) -> M2c-2a2 cleanse, disengage, L-sweep, unarmed budget -> M2c-2b content (skills, paths, units) | planner 2026-10-15 |
| Milestone plan v2 picks (2026-10-16) | Approved: R1 test split first (tests only), then SK-L (M2C2B-L), AI1 and M3 in parallel (builder cap 3), M2C2B asset branch after SK-L's schema commit and C1-A1/D1-L, M4 spec after M3, BAL1 after SK/AI1/M3. Party 4 active (roster may be larger), full heal after battles, permanent spend, full XP to each active member, enemy AI uses riposte, AI avoids friendly-fire areas | User 2026-10-16 |
| R1 test split (2026-10-16) | `ArenaTestSuite` is a thin runner; the suite list lives in `SuiteRegistry.run_all` (one `results.append` line per suite); shared test helpers are `TestHelpers` (hand_ctx, run_steps, resolve_until_replay); suite files stay under ~450 lines; `run_planning_and_ai_test` was split at its 5e section into PlanningTestSuite + PlanningAiExtraTestSuite (the latter is called by the former, not registered). Natural next splits: combat_extensions_test_suite.gd (helpers lines 16-118 + three groups) and dojo_assertions.gd (per _run_* function) | R1 report |
| Reviewer model (2026-10-16) | change-reviewer stays on the inherited model; haiku used 2.4x the tokens and 3.5x the time on the same static review (more exploration) for one extra true finding; use haiku only for an optional second thorough pass | Trial 1, see docs/AGENT_MAP.md |
| Test convention (2026-10-16) | Tests must not assume a unit owns exactly the loadout slot count: derive overflow/hidden families from owned_families() minus active_class_families; find a tier by rule (e.g. the purify tier whose cleanse list contains `all`); rename fixtures use their own family, never a shipped one | M2C2B test update |
| Agent models (user 2026-10-17) | `asset-builder` and `balance` pinned to `model: sonnet` (scripted data seeds and sims); `logic-builder`, `change-reviewer`, `design-brainstorm` inherit the session model; haiku not used (trial 1: more tokens on a static review) | User 2026-10-17 |
| Cleanup pass 1 answers (user 2026-10-17) | Candidates confirmed with the planner's sequencing (docs/archive moves now; logic cleanups after BAL2-L, before ITEM1). (1) DECISIONS may be split, also by system, so brainstorming reads only the targeted system's decisions; moved rows stay available for review (archive with "superseded by" pointers counts as append-only). (2) Map the already available placeholder art (goblin keys) to units now; core systems incl. art-asset handling are the focus. (3) Deprecate the legacy `smart_skills = false` AI path and drop its tests (after the BAL2 rerun). (4) `.context/` is a legacy artifact: quick pass for anything worth integrating into the current docs, then drop it. (5) Skills v1 loading: deprecate first, drop once no usage is found; same approach for other old skill paths | User 2026-10-17 |

## Partly superseded rows (text unchanged; the newer row wins on the named part)

- "R1 test split (2026-10-16)": the `ArenaTestSuite` pass-through is removed in CLN-14 (`SuiteRegistry.run_all` is called directly).

See also balance.md "Small answers (user 2026-10-17)" for the CLN-15 placeholder-art answer.
