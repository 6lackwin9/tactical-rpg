# CLEANUP: housekeeping proposals (housekeeper agent)

Newest pass on top. Candidate statuses: `proposed`, `confirmed (user <date>)`, `rejected (user <date>): <reason>`, `done`. Only the user confirms; confirmed candidates become `docs/TASKS.md` rows. Cleanup is never folded into a feature task: "after X" = run once X is merged, "superseded by X" = X deletes it anyway.

## Pass 1 (2026-10-17, main at db1ec88)

Scope: SKILL1 legacy, superseded ideas, dead code, seeds, test helpers, oversized docs and suites (read-only `git grep` scan). Overlap checked against the Touches of BAL2-L, BAL2-A, ITEM1 and CLASS1 in `docs/TASKS.md`.

### Ranked summary

| Rank | Id | What | Size | Lane | When | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | CLN-1 | Fix doc drift that misleads agents (skill cap, ArenaTestSuite, stale TASKS rows, HISTORY layout) | S | planner (docs) | now | done (docs/) 2026-10-17 |
| 2 | CLN-2 | Archive the 9 executed seeds in `data/seeds/` | S | asset-builder | now | confirmed (user 2026-10-17) |
| 3 | CLN-3 | Remove SKILL1 legacy normalisation and the `offhand` / `advanced` shims | S-M | logic-builder | after BAL2-L, after CLN-2 | confirmed (user 2026-10-17) |
| 4 | CLN-4 | Archive `.context/` (10-block design docs) | S | planner (docs) | now | done (docs/) 2026-10-17 |
| 5 | CLN-5 | Archive done specs; slim FORGE1_SKILLS and MATRIX to a shared Forge rules file | M | planner / design | before ITEM1 and CLASS1 dispatch | done 2026-10-17 |
| 6 | CLN-6 | Delete dead functions and legacy class constructors | S | logic-builder | after BAL2-L (ItemCatalog part after ITEM1) | confirmed (user 2026-10-17) |
| 7 | CLN-7 | Drop the second unit-power formula in `CharacterStats` | S | logic-builder | after BAL2-L | confirmed (user 2026-10-17) |
| 8 | CLN-8 | Move repeated test helpers into `TestHelpers` | M | logic-builder | after BAL2-L, before ITEM1 | confirmed (user 2026-10-17) |
| 9 | CLN-9 | Split `content_catalog.gd` (1384 lines) by kind | L | logic-builder | after BAL2-L, before ITEM1 and CLASS1 | confirmed (user 2026-10-17) |
| 10 | CLN-10 | Split DECISIONS.md and move superseded rows (parked item) | M | planner (docs) | user choice | done (docs/) 2026-10-17 |
| 11 | CLN-11 | Trim ROADMAP.md of decided brainstorm options and done plans | S-M | planner (docs) | now | done 2026-10-17 |
| 12 | CLN-12 | Split suites over the 450-line rule | M | logic-builder | after ITEM1 | confirmed (user 2026-10-17) |
| 13 | CLN-13 | README overview plus per-topic files (parked item) | L | planner (docs) | user choice | confirmed (user 2026-10-17) |
| 14 | CLN-14 | Remove the `ArenaTestSuite` pass-through | S | logic-builder | with CLN-8 | confirmed (user 2026-10-17) |
| 15 | CLN-15 | Map the existing goblin placeholder art to units (new, from Q1) | S | asset-builder | now, not in parallel with BAL2-A | confirmed (user 2026-10-17) |
| 16 | CLN-16 | Remove the legacy `smart_skills = false` AI path (new, from Q2) | S | logic-builder | after the BAL2 rerun | confirmed (user 2026-10-17) |

### Confirmation and sequencing (user 2026-10-17, DECISIONS "Cleanup pass 1 answers")

All candidates confirmed; TASKS rows `CLN-<n>` carry status `approved`. Lanes: docs-only work goes to design-brainstorm (the planner keeps README, CLAUDE.md and `.claude/*`), seeds and data to asset-builder, code to logic-builder.

- Now (docs, archive, seeds; no builder-file overlap): CLN-1, CLN-2, CLN-4, CLN-5, CLN-11, CLN-10 (after CLN-1), CLN-15 (before BAL2-A starts or after it merges: both edit `units.json`).
- After BAL2-L merges, before ITEM1: one logic cleanup branch in this order: CLN-14, CLN-8, CLN-6, CLN-7, CLN-3, CLN-9 (pure move last, after the content edits).
- After ITEM1: CLN-12 plus the ItemCatalog part of CLN-6. After one release with no deprecation warnings: CLN-3b. After the BAL2 rerun: CLN-16.
- Parked: CLN-13.
- Answers folded in: CLN-10 splits by system; CLN-4 gets a short integration pass before the drop; CLN-3 deprecates v1 loading and the old-key normaliser first (CLN-3b drops them). Q1 became CLN-15, Q2 became CLN-16; Q3 stays a review after ITEM1.

### CLN-1

Doc drift. Status: done for docs/ 2026-10-17.

- What and where: agent-facing facts that the code no longer has. `docs/GAME_BRIEF.md:22` "skill cap = ceil(level/2)" and `:57` "within the unit's skill cap" (SK2 removed the cap, DECISIONS rows SK2 picks / SK2-L built); `:52` riposte "L2, learned_only, guard 0.15, 2 rounds" (now skill data, DECISIONS "M2C2B content"); `:93` "Advanced families (`advanced`)" (now `grade`, HISTORY 2026-10-16 row); `docs/STRUCTURE.md:44` and `.claude/agents/logic-builder.md:22` register suites in `ArenaTestSuite.run_all_tests` (now `SuiteRegistry`, DECISIONS "R1 test split"); `.claude/skills/godot-debug-checklist/SKILL.md:42` and `.claude/skills/patch-and-seed/SKILL.md:17` cite the skill cap; `README.md:79,121,294,596` skill cap, `README.md:381` "20 test suites, 171 assertions" (34 suites now). `docs/TASKS.md:8-9` keep C1 and D1 `in-progress` although their asset parts were folded into M2C2B (done) and DECISIONS records the C1-L, C1-A1 and D1-L reviews; `docs/TASKS.md:17` still lists "open question: `evolves_from`" (resolved, DECISIONS "FORGE1 skill answers 2" and "round 4"). `docs/HISTORY.md:19-22` the Lessons section sits inside the table, so rows 23-28 render outside it.
- Change: correct the lines; set C1, D1, FORGE1 rows to `done`; move the Lessons block below the table (no row text changes, append-only kept). Leave `GAME_BRIEF.md:65` (UNARMED_VALUE 14) to BAL2-A, which changes it.
- Risk: none in code. `.claude/*` edits are the planner's with the user's consent.
- Touches: `docs/GAME_BRIEF.md`, `docs/STRUCTURE.md`, `docs/TASKS.md`, `docs/HISTORY.md`, `README.md`, `.claude/agents/logic-builder.md`, `.claude/skills/godot-debug-checklist/SKILL.md`, `.claude/skills/patch-and-seed/SKILL.md`.

### CLN-2

Archive executed seeds. Status: confirmed (user 2026-10-17).

- What and where: `data/seeds/c1a1.gd`, `m2c2b_a.gd`, `m2c2b_b.gd`, `m2c2b_c.gd`, `m2c2b_d.gd`, `m2c2b_lib.gd`, `sk2.gd`, `sk2_schema.gd`, `skill1_ranks.gd` (803 lines). Their tasks are done (TASKS rows C1, M2C2B, SK2, SKILL1-A); nothing references them except the `m2c2b_*` preloads of `m2c2b_lib.gd`.
- Why bloat: (1) they encode superseded data: `m2c2b_lib.gd:44` writes `"offhand": false`, `sk2.gd:28` writes `advanced`, `m2c2b_a.gd` re-creates pre-SK2 paths and mage `volley`, so a re-run would revert shipped data; they are the only reason the legacy normaliser of CLN-3 must stay. (2) `AssetScan.SCAN_DIRS` (`scripts/content/asset_scan.gd:11`) includes `res://data/seeds`, so every id they quote counts as a reference: `ContentCatalog.demote` refuses with "named by tests or seeds" (`content_catalog.gd:458-460`) and SAFE DELETE refuses too, for almost every shipped skill and class. (3) `parse-all` compiles them on every run. DECISIONS "C1-A1 assets" keeps seeds idempotent for re-runs after a rebase, which no longer applies once merged.
- Change: remove them from `res://` (git history keeps them) and add `data/seeds/README.md` with one line per seed (task, merge commit: `11ffe67`, `b7c4016`, `9686a36`, `7bed659`, `745962a`) plus the rule "a seed leaves this folder in the cleanup pass after its task is merged".- Risk: low. `origin_test_suite.gd:158-188` uses its own fixtures, not seed hits. BAL2-A's coming `data/seeds/bal2.gd` is unaffected. Update `docs/STRUCTURE.md:17,29`.
- Touches: `data/seeds/*` (new README), `docs/STRUCTURE.md`.

### CLN-3

SKILL1 legacy shims. Status: confirmed (user 2026-10-17).

- What and where: `ContentSchema._legacy_skill` (`scripts/content/content_schema.gd:369,379-392`, comment "accepted for one release"); derived properties `SkillDef.offhand` (`scripts/combat/planning/skill_def.gd:72-77`) and `SkillDef.advanced` (`:101-106`); their readers `tactical_ai.gd:168,171`, `plan_entry.gd:62`, `skill_catalog.gd:90`, `damage_calculator.gd:196`, `dojo_session.gd:458-459,513,618,621,630`, `forge_preview.gd:224`, `combatant.gd:284,299`, `class_rules_test_suite.gd:175`; tests of the removed behaviour `skill_model_test_suite.gd:147-151,160,254`; `content_test_suite.gd:115` still feeds the old `"advanced": true` key.
- Why bloat: HISTORY 2026-10-16 rows (bool `advanced` / `evolves_from`, off-hand categories) and DECISIONS "Strict hand typing" / "round 4". Shipped data is clean: `skills.json` is version 2 with no `advanced`, `offhand` or `evolves_from` keys (the 11 hits are `"grade": "advanced"`), no world file has them, `evolves_from` is already gone from code.
- Confirmed shape (user: deprecate first, drop once unused): in CLN-3 `_legacy_skill` and v1 skills loading warn on use; CLN-3b deletes both when logs show no hit. Shims and legacy-key tests go in CLN-3.
- Change: delete `_legacy_skill` (CLN-3b); readers use `weapon_hand == SkillDef.WeaponHand.OFF` (one helper such as `is_off_hand()` is fine) and `grade != SkillDef.Grade.NOVICE`; delete the legacy tests, rewrite line 115 with `grade`. Optional: drop version 1 skills-file reading (`content_catalog.gd:143` keeps v1 on the generic path) and its check `skill_ranks_test_suite.gd:196-203`; small gain, ask. Keep `WeaponHand.BOTH` and `BOTH_ENABLED` (DECISIONS "Strict hand typing": `both` is an engine follow-up).
- Risk: an old JSON with the bool keys would silently lose them; none exists after CLN-2. Covered by skill_model, content, class_rules, skill_family, ai_skills, dojo suites. Assertion count drops by the removed legacy checks only.
- When: after BAL2-L (shares `content_schema.gd`) and CLN-2; `combatant.gd` is in ITEM1's Touches, so run before ITEM1 starts or after it merges.
- Touches: `content_schema.gd`, `skill_def.gd`, `tactical_ai.gd`, `plan_entry.gd`, `skill_catalog.gd`, `damage_calculator.gd`, `dojo_session.gd`, `forge_preview.gd`, `combatant.gd`, suites skill_model, content, class_rules (optionally content_catalog, skill_ranks).

### CLN-4

Archive `.context/`. Status: done for docs/ 2026-10-17; the planner deletes the folder.

- What and where: `.context/COMBAT_SYSTEM.md` (14 KB, "Fixed 10-Block Timeline", six stances), `.context/GAME_DESIGN.md` (10 KB, "10-Block Tactical Combat"), `.context/AGENTS.md` (4 KB coding standards); last edited 2026-10-10/11.
- Why bloat: HISTORY row 1 deprecated the 10-block scripted layer; the coding rules repeat `CLAUDE.md` and `.agents/rules/godot_workflow.md`. Only `README.md:604,613,621` point at GAME_DESIGN.md (as the spec of the placeholder systems S9-S11).
- Change (confirmed shape): a quick pass for anything still useful (candidates: GAME_DESIGN systems 1, 2 and 9 as input for ROADMAP P-S9..P-S11; any coding rule missing from CLAUDE.md or `.agents/rules`), integrate it into the current docs, then drop the folder (git history keeps it) with a note in `docs/archive/README.md`; repoint the README lines to ROADMAP.
- Risk: none in code. Question: does an IDE integration still read `.context/AGENTS.md`? If yes keep that one file.
- Touches: `.context/*`, `docs/archive/*`, `README.md`.

### CLN-5

Archive done specs. Status: done 2026-10-17.

- What and where: (a) specs of done tasks whose headers still say `proposed`/`approved`: `AI1.md:3`, `B1.md:3`, `C1.md:3`, `D1.md:3`, `M2C2B.md:3`, `M3.md:3`, `R1.md:3`, `SK2.md:3` (about 88 KB). (b) `FORGE1_DESIGN.md` (23 KB): skills part already archived, items superseded by FORGE1_ITEMS (HISTORY 2026-10-16 items row), classes by FORGE1_CLASSES, units section 5 restated by `FORGE1_CLASSES.md` section 8. (c) `FORGE1_SKILLS.md` (40 KB) and `FORGE1_SKILLS_MATRIX.md` (28 KB), both over the 25 KB limit (`tools/fc.sh docs`); SKILL1 is done.
- Change: mark (a) `done` and move to `docs/archive/specs/`; archive (b); for (c) copy the parts the next passes still cite into `docs/specs/FORGE_SHARED.md` (section 12 shared Forge rules, 11.3 ownership, 4.1 `weapon_hand`, matrix M origin key fields; about 10 KB, keep the section numbers so code comments such as `asset_scan.gd:5` "FORGE1 section 12.5" and `skill_def.gd:31` "FORGE1 11" still resolve) and archive the rest. Repoint `FORGE1_CLASSES.md:4,29,117,234`, `FORGE1_ITEMS.md:96,172`, `TASKS.md:17-19`.
- Leave `FORGE1_ITEMS.md` (40 KB) and `FORGE1_CLASSES.md` (26 KB) alone: ITEM1 and CLASS1 build from them; slim them the same way after those merge.
- Risk: none in code; a builder brief pointing at an old path. Do it before ITEM1 / CLASS1 are dispatched so their briefs cite stable paths.
- Touches: `docs/specs/*`, `docs/archive/*`, `docs/TASKS.md`, `docs/specs/README.md` (note that done specs move to the archive).

### CLN-6

Dead code. Status: confirmed (user 2026-10-17).

- What and where (zero callers in scripts, scenes, tests, tools; checked with `git grep -w`): `ClassArchetype.create_shaman/create_scout/create_berserker/create_brawler` (`scripts/core/specs/class_archetype.gd:81-92`; three name classes removed by HISTORY 2026-10-15 roster row and would only warn "unknown class"); `CharacterStats.get_ranged_power`, `get_tactical_power` (`character_stats.gd:152-158`, fixed 1.5 scaling, HISTORY "Fixed 1.5 Unit Power" row), `get_intent_unmask_blocks` (`:169-175`), `remove_modifiers_from_source` (`:207`, alias), `tick_modifiers` (`:210-216`), field `experience` (`:19`, M3 uses `total_xp`), header "Masteries" (`:6`, HISTORY Mastery row); private test helpers `engine_pieces_test_suite.gd:209 _backflip_steps`, `ai_skills_test_suite.gd:173 _uses_any`; `world_forge.gd:623 _current_entry`; `ItemCatalog.create_spiked_knuckles`, `create_consumable`, `all_items` (`item_catalog.gd`).
- Verify before deleting (public API that a planned UI may want): `CombatGrid.can_target`, `Combatant.has_target`, `Proficiency.duplicate_set`, `PlanStep.duplicate_step`, `WorldCatalog.entry_count`, `Equipment.get_requirements_display`, `Inventory.get_total_gear_stat_bonus`, `CombatDockHost.is_edit_mode/panel_ids`, `PartyState.member_ids`, `DictUtil.bool_of`, signals `leveled_up` / `attribute_point_allocated` (emitted, never connected).
- Kept on purpose: README roadmap keeps "Reactive intent" (`README.md:558`) as an idea; deleting the unused PERC helper does not drop the idea.
- Risk: low; `parse-all` catches missed callers.
- When: `character_stats.gd` after BAL2-L; `item_catalog.gd` after ITEM1 (the item migration may rename ids).
- Touches: `class_archetype.gd`, `character_stats.gd`, `item_catalog.gd`, `world_forge.gd`, suites engine_pieces, ai_skills.

### CLN-7

Second unit-power formula. Status: confirmed (user 2026-10-17).

- What and where: `CharacterStats.get_melee_power` (`character_stats.gd:148-150`, `weapon + round(STR x 1.5)`) is used only by tests: `character_specs_test_suite.gd:34` and Dojo-10 (`scenes/dojo/dojo_assertions.gd:113`), which compares the combatant's real power with it.
- Why bloat: `DamageCalculator` is the only formula (DECISIONS "Combat extensions", GAME_BRIEF); per-class proficiency scaling replaced the fixed 1.5 (HISTORY). Dojo-10 passes only while the default scaling equals 1.5; BAL2-A "trained fists" (DECISIONS "BAL2 role answers" 5) changes unarmed power and will break this check.
- Change: derive both expectations from `DamageCalculator.attack_power`; delete `get_melee_power`.
- Risk: low; two assertions change their source, count unchanged.
- When: after BAL2-L; tell the BAL2-A brief that Dojo-10 reads a second formula until then.
- Touches: `character_stats.gd`, `character_specs_test_suite.gd`, `scenes/dojo/dojo_assertions.gd`.

### CLN-8

Test helpers into `TestHelpers`. Status: confirmed (user 2026-10-17).

- What and where: 19 private `_report` copies in two shapes (checks dict, or `ok` + detail), e.g. `ai_skills_test_suite.gd:15`, `combat_extensions_test_suite.gd:16`, `engine_pieces_test_suite.gd:12`, `focus_gear_test_suite.gd:16`, `foundation_test_suite.gd:14`, `class_rules_test_suite.gd:14`, `content_test_suite.gd:15`, `core_rules_test_suite.gd:14`; `SkillTestKit.report` (`skill_test_kit.gd:7`) is already the generic one. Five hand-built `_unit` combatant builders with the same body (`ai_skills:114`, `combat_extensions:52`, `riposte:38`, `skill_engine:42`, `weaken:55`). Fifteen suites fork a sandbox with their own check (`ContentCatalog.fork_to(SANDBOX)` vs `SkillTestKit.fork`).
- Why bloat: DECISIONS "R1 test split" names `TestHelpers` as the home of shared helpers; `GAME_BRIEF.md:78` repeats it.
- Change: `TestHelpers.report(tag, label, checks)`, `TestHelpers.report_ok(tag, label, ok, detail)`, `TestHelpers.unit(...)`, `TestHelpers.fork(sandbox)`; fold `SkillTestKit` fork/cleanup/report into it (keep its skill fixtures `attack`, `messages`, `any_contains` or move them too). Print tags stay identical so log greps keep working.
- Risk: low-medium (many files); rule: assertion count before = after.
- When: after BAL2-L (edits class_rules, engine_pieces); before ITEM1 so ITEM1's suites use the shared helpers.
- Touches: `scripts/tests/*`.

### CLN-9

Split `content_catalog.gd`. Status: confirmed (user 2026-10-17).

- What and where: `scripts/content/content_catalog.gd` is 1384 lines: store and IO (1-270), origin (280-462), ranked skills API (464-730), references (733-806), validation hub (808-945), skill rules (946-1137), item rules (1138-1186), class, unit and loadout rules (1190-1384).
- Why: `docs/TASKS.md:21,24` must run CLASS1 and ITEM1 sequentially because of these hub files; ROADMAP "Hub-file overlap" finding 2 deferred an R2 split "if a later item or skill schema change collides", which is now the case.
- Change: pure move of the per-kind rules into `scripts/content/rules/{skill,item,class_unit}_rules.gd` (static, called by `validate_entry` / `warnings_for`); `ContentCatalog` keeps store, IO, origin, references and dispatch. Optional second step: per-kind field tables of `content_schema.gd` (502 lines). No behaviour change.
- Risk: medium (largest file, private cross-calls); fully covered by the content, skill_* and class_rules suites; assertion count equal.
- When: after BAL2-L merges, before ITEM1 / CLASS1 start. Gain: ITEM1 and CLASS1 may then run in parallel if `content_schema` / `content_factory` edits stay small (planner decides).
- Touches: `content_catalog.gd`, new `scripts/content/rules/*`, `docs/STRUCTURE.md`.

### CLN-10

DECISIONS split. Status: done for docs/ 2026-10-17 (user: split by system, moved rows stay reviewable; was parked in EFFICIENCY "Doc housekeeping"; it is 35 KB now, 18 KB when parked, and ITEM1 / CLASS1 / BAL2 will add rows).

- Superseded or duplicate rows (line numbers in `docs/DECISIONS.md`): 12 commit trailer hard-coded (EFFICIENCY "Downtime pass": trailer from the session); 18 gear budget details open (25, 40, 41); 20 support identity pending (27); 24 and 32 build/milestone order (71); 28 duplicates 9; 44 level 4-5 advanced versions (55 says it supersedes); 45 slot counting open (53, 57); 52 tier selection pending (55); 55 "limited by the unit's skill cap" (79, 80); 57 basics as strike and guard families (60, 66); 61 Rest pending (67); 63 riposte AP 1/2/3 (76); 76 interim riposte level 5 (79, 82); 79, 80, 82 bool `advanced` (88); 81 "flags like `advanced`" (88); 85 `evolves_from` (88); 86 dual-wield guard as main rule (87-89); 87 per-hand bands and `offhand` (88); 92 item 11 duel gate (94 item 3).
- Change: `docs/DECISIONS.md` becomes a short index; rows move unchanged into domain files (combat, classes and skills, gear and items, content and Forge, process and balance); superseded rows go to `docs/archive/DECISIONS_SUPERSEDED.md` with a "superseded by" pointer. Add the superseded concepts HISTORY lacks: AP-cost skill unlock and skill cap, class-level active skill loadout (now on the unit), level 4-5 advanced versions, basic-skill tier variants, riposte AP 1/2/3, hard-coded commit trailer.
- Question for the user: is moving a superseded row to the archive (text unchanged, pointer added) compatible with "append-only, never delete"?
- Touches: `docs/DECISIONS.md`, new `docs/decisions/*`, `docs/archive/*`, `docs/HISTORY.md`, `CLAUDE.md` and `.claude/agents/*` pointers.

### CLN-11

ROADMAP trim. Status: done 2026-10-17.

- What and where (`docs/ROADMAP.md`, 21 KB): "Open questions for the user" (37-42, answered by DECISIONS 25-28); "Proposed designs (brainstorm pass 2026-10-15)" (44-138, options decided by DECISIONS 16-28 and built in M1-M3); "Refactor backlog" R1 (140-144, done); "Milestone plan v2" and its hub matrix (146-178, all done except M4); S2 row note "M2c-2b content next" (14, done); "Open engine items" first two bullets (192-194, AI hooks done in AI1).
- Change: move those sections verbatim to `docs/archive/ROADMAP_2026-10.md`; keep systems table, suggested build order, BAL2 plan, remaining open engine items; set the S2 note to the current state.
- Risk: none. Touches: `docs/ROADMAP.md`, `docs/archive/*`.

### CLN-12

Large suites. Status: done 2026-10-17.

- Result: suites CombatExtAttacks, CombatExtAi (helpers `CombatExtKit`), AiSkillsTactics (fixtures `AiSkillsKit`), Unarmed, FocusGearBudget; dojo split into DojoSessionAssertions, DojoUiAssertions. Same PASS lines. ItemCatalog: `create_spiked_knuckles`, `all_items` deleted; `create_consumable` kept (in use). Dojo-27 has no caller: CLN-17.
- What: suites over the 450-line rule (DECISIONS "R1 test split"): combat_extensions 795, dojo_assertions 771, ai_skills 634, engine_pieces 480, focus_gear 459.
- Change: split by feature group, one registry line each; no behaviour or number change.
- Touches: those suites, the registry.

### CLN-13

README split. Status: confirmed (user 2026-10-17), still parked until the next large README update (EFFICIENCY item 4).

- What and where: `README.md` 88 KB, 674 lines; "Codebase Architecture & Directory Breakdown" 20 KB repeats STRUCTURE and GAME_BRIEF; "Headless Testing" 8.8 KB of per-milestone notes; "Roadmap & Features Yet to be Implemented" repeats ROADMAP.
- Change: overview README (run, layout pointer, feature list) plus `docs/features/<system>.md`; replace the repeated parts with links. Agents do not read the README, so the value is safer planner edits.
- Touches: `README.md`, new `docs/features/*`.

### CLN-14

`ArenaTestSuite` pass-through. Status: confirmed (user 2026-10-17).

- What and where: `scripts/tests/arena_test_suite.gd` (15 lines) only forwards to `SuiteRegistry.run_all`, and its suite list (8-11) is stale; the only caller is `scenes/test_arena.gd:15`.
- Change: call `SuiteRegistry.run_all` from `test_arena.gd`, delete the file, update `GAME_BRIEF.md:78`, `STRUCTURE.md:13,44`, `logic-builder.md:22`.
- Risk: very low. Touches: `arena_test_suite.gd`, `scenes/test_arena.gd`, docs above.

### CLN-15

Goblin placeholder art. Status: confirmed (user 2026-10-17, new from Q1).

- What and where: `UnitArt.GOBLIN_CLASSES` (`unit_art.gd:17-18`) draws archer, berserker, shaman, skirmisher with goblin skin (plus scout), but every unit has `art: ""`. Unit `art` is a schema enum over `CLASS_KEYS` (`content_schema.gd:251`), read via `ContentFactory.art_of` (`catalog_encounter.gd:141`); world characters inherit it (`world_factory.gd:19-21`): data only.
- Change (asset lane, seed `data/seeds/cln15_art.gd` through `ContentCatalog.upsert`): `goblin_archer` to archer, `goblin_shaman` to shaman, `goblin_skirmisher` to skirmisher, `goblin_berserker` and `ambush_berserker` to berserker, `fleeing_scout` to scout. `goblin_brawler` has no goblin key: berserker (goblin skin) or keep brawler; the builder asks the planner.
- Risk: low; no test pins a unit's art. Not beside BAL2-A (both edit `units.json`).
- Touches: `data/catalog/units.json`, `data/seeds/cln15_art.gd` (archived after merge per CLN-2).

### CLN-16

Legacy AI path. Status: done 2026-10-17.

- What and where: `AiProfile.smart_skills` (`ai_profile.gd`, 2 uses) and its off branches in `tactical_ai.gd` (5 uses); `ai_skills_test_suite.gd` (7 uses) covers both paths; BAL2-L adds a sim switch for it (`tools/sim/*`).
- Change: remove the `false` path, the field, its tests and the sim switch; fix GAME_BRIEF "AI skill use".
- When: after the BAL2 rerun. Risk: low (ai_skills suite).
- Touches: `ai_profile.gd`, `tactical_ai.gd`, `ai_skills_test_suite.gd`, `tools/sim/*`, `docs/GAME_BRIEF.md` (planner).

### Questions (not proposed for removal)

Q1 and Q2 answered (now CLN-15, CLN-16); Q3 waits for ITEM1.

- Q1 (unused goblin art keys) answered: map them now (CLN-15). Q2 (`smart_skills = false`) answered: remove after the BAL2 rerun (CLN-16).
- Q3 `ItemCatalog` named shorthands (34 uses in `dojo_loadouts.gd`) map old names to item ids (`create_iron_claymore` returns `wpn_claymore`); review after ITEM1's migration. Answered (user 2026-10-17): fold the review into the CLN-6 ItemCatalog part (ITEM1 is done).

### Checked and kept

`WeaponHand.BOTH` / `BOTH_ENABLED` (planned), `StatRules.default_unlock` (live rule), `AiProfile.skill_cap` (SK2; dummy sets 1), slot constants (unit loadout), flat-row `upsert` (live API), unused passive templates (M4 sources), plain dual wield `equipment.gd:80` (superseded by ITEM1).
