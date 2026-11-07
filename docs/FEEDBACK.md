# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 release | released: `main` @98ab973a (PR #55, merge commit; tree = develop @f7f5cb71) | `main` | Next: Phase R1.1. |
| R1.1-1 `AUDIT-R1b` (AUDIT-08..12) | done (FB-95) | `develop` | 3 parallel lanes + FB-94 additions; checks_golden re-recorded. |
| R1.1-2 `C02-A` | done (FB-96) | `develop` | asset + logic lanes in parallel; phasefold re-recorded. |
| R1.1-3 `R1-BAL` | done (FB-99, FB-100, FB-103) | `develop` | Phase R1.1 complete. |
| READY row 14 `A2.2-S1/S2` | done (FB-102) | `develop` | - |
| READY row 16 `A4.2-K1a` + `A3.2-S1` | built, parked: limit-blocker FB-101 (not landed) | `origin/claude/a42-k1a` @e07054b0 | Full gate and change-reviewer still to run after Design answers. |
| READY row 19 `I1.2-S1` | next (batch 1) | - | Not started. |
| READY rows 15, 17, 18, 20 (R2-W1 batch 2, R3-W1) | queued | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-99 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-100. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-100: R1-BAL (1)(A) and (2b)(A) landed: "no AP (enemy_moves)" in sim feel, "(Stagger queued)" on the break log line

- **Spec**: `docs/READY.md` R1.1-3 `R1-BAL` parts (1)(A) and (2b)(A); FB-97, FB-92, FB-88 (@c427b661, @83a1d0af)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: (1) `rpgkit_sim` feel: new `Side.unpaid_rounds` counts unit-rounds with `World::pays_ap` false; the human line prints `AP/round no AP (enemy_moves)` when every unit-round of the side is one (a mixed side keeps the number; JSON `ap_per_round` unchanged). `sim_golden` re-recorded on purpose: 6 enemy lines of run 21, AP text only. (2b) `rpgkit_battle` poise: `Break.stagger_queued` (from `break_queues_stagger` in `break_unit`); `break_line` appends ` (Stagger queued)`. `timeline_golden` re-recorded on purpose: 8 of 15 fights, 21 `[Break]` lines, suffix only. Portrait and GUI break matchers are prefix matches and still match; `tests/poise.rs:269` runs under `bar` (no suffix). Gate: build, 2426 tests passed, layers, docs `--all`. change-reviewer clean; nit: no direct unit test of the suffix (pips vs `steadfast`) or of a mixed side, only the goldens cover them.
- **Recommendation**: HISTORY: "R1-BAL (1)(A): sim feel prints 'no AP (enemy_moves)' for a side that pays no AP (closes FB-92's 0 AP/round item by rule); re-recorded on purpose: `sim_golden` (text only)". HISTORY: "R1-BAL (2b)(A): the `[Break]` log line ends with '(Stagger queued)' when the break queued stagger; re-recorded on purpose: `timeline_golden` (text only)". GAME_BRIEF poise: "Under pips the battle log's break line notes '(Stagger queued)'". STRUCTURE: `poise::Break.stagger_queued`.

### FB-101: A4.2-K1a (row 16): IR21 gives 22 dual-intent skills on shipped data, not 26; the 18 Field,Assault assumes the SWEEP split

- **Spec**: `docs/READY.md` row 16 `A4.2-K1a` + `A3.2-S1`; `docs/sessions/A4_EFFECTS.md` sections 2 and 4; FB-57
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: Built on `origin/claude/a42-k1a` @e07054b0 (not landed): `rpgkit_rules::intent` (one IR21 derive, rule-gated by `round_intent`, no caller yet), `RING`/`ARC` in `skill::Area` and `ground::Shape` (code only), 15 new tests; targeted tests on the 7 golden crates 2441 passed, 0 golden diffs. On shipped `skills.json` (129 rows) the rule gives 8 `Support, Stand` (mend, purify, greater_mend, aegis, tl_field_dressing, brimstone_cauterize, tourniquet_tonic, resilience_ward) and 14 `Field, Assault` (LINE: shockwave, arc_lance, execution_wall; PLUS: volley; BURST reach_max > 1: firebomb, inferno, glacial_burst, magnesium_flash_bomb, transfusion_pulse, bone_mill_pulverizer, pressurized_lard_spray, scalding_cataclysm, ossified_spine_volley, ossification_bloom) = 22. Shipped data has no ARC, RING or CONE: the 7 shaped melee skills are still SWEEP (sweep_kick, cyclone_kick, foundry_shatter, crushing_judgment, gilded_billhook_sunder, pneumatic_hammer_strike, tendril_flail), and nothing in their data says which become ARC. The spec's 18 = 14 + 4 SWEEP rows as ARC (the other 3 as RING -> Stand); with that split Assault 59 and Maneuver 14 also match. SWEEP derives as an unshaped attack (Assault) for now; the test asserts S,S = 8 and F,A + SWEEP = 21, with a remap hook ready. Adding RING/ARC (and CONE) to the `kits/rpg/schema.json` `area` enum moves `checks_golden` (2 enum-error cases), so the schema enum and `HAZARD_SHAPES` wait for a named schema slice; data cannot use RING/ARC yet.
- **Recommendation**: (A, recommended) Design names the split per id (4 ARC, 3 RING) in READY row 16; Build applies it in-memory in the count test (26 = 8 + 18) and lands row 16 as built, leaving the data change to W2-4 (`K2` retires SWEEP). (B) Change row 16's acceptance to the shipped-data count (22 = 8 + 14) and move the 26 check to W2-4. (C) Add the SWEEP -> ARC/RING data seed and the schema enum to row 16 (moves `checks_golden`; needs naming). Also confirm: (b) a SELF-only SUPPORT (`taunt_shout`) derives Stand only (that reading gives the 8); (c) a BURST/PLUS with reach_max <= 1 and no hazard derives Field only (none shipped). Docs delta once landed: STRUCTURE `rpgkit_rules/src/intent.rs`, tests `rpgkit_rules/tests/ir21_intent.rs`, `emberwake_rules/tests/ir21_intent.rs`, `rpgkit_battle/tests/area_shapes.rs`; kit README / PORT_NOTES "rpgkit_rules::intent (INTENT2-K1a, IR21): one derive of a skill's 1-2 allowed intents; rule-gated by `round_intent`; no caller yet (K1b switches the planner and checks)".

### FB-102: A2.2-S1 + A2.2-S2 (row 14) landed: take_damage and heal on the pipeline with traces; folded factors trace-only; 0 golden diffs

- **Spec**: `docs/READY.md` row 14; `docs/sessions/A2_RESOLUTION.md` A2.2-S1/S2; FB-43, FB-56, FB-84 (@a2854544)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `pipeline::TAKE_GUARDED` (floor 1) and `pipeline::PASS` (exact integer, no floor); `damage::take_damage(_traced)` (amount, then guard Reduce; min 1 only when the guard applies, bit-identical to the old expression) and `damage::heal_gain(_traced)` (cap at max_hp after the run, an integer step: the pipeline has no ceiling stage); `Unit::take_damage` and `Unit::heal` delegate, signatures unchanged. `heal_amount` was already on `pipeline::HEAL` since U4.2-S4 (verified by test, code unchanged). S2: `damage::compute_traced_folded` names the factors folded into `skill_mult` (`fold:skill|exploit|flank|splash|counter|weaken|empower`) as trace-only parts of the `skill` step; op and f64 order unchanged. S2 is API only: every hit resolved in battle uses the untraced `damage::compute`, so no caller produces folded parts yet. No `splash` factor exists in kit or game code today (`FOLD_SPLASH` is defined, nothing produces it). FB-84: there is no hit-time `add_pct` term; damage potency is still folded at planning; status rolls (`status::apply_chance_with`) are outside the pipeline with no trace. Tests: `rpgkit_rules/tests/take_damage_pipeline.rs` (5). Gate: workspace tests after rebase on R1-BAL (rebased on 83a1d0af: build ok, 2431 passed, 0 failed, 0 golden diffs). change-reviewer clean; nits: doc comments on the six other `FOLD_*` consts, `take_damage_modifiers` doc wording.
- **Recommendation**: kit README / PORT_NOTES (`rpgkit_rules::pipeline`): "A2.2-S1: `pipeline::TAKE_GUARDED` (floor 1) and `pipeline::PASS` (exact integer); `Unit::take_damage` -> `damage::take_damage(_traced)`, `Unit::heal` -> `damage::heal_gain(_traced)` (cap after the run). A2.2-S2: `damage::compute_traced_folded` names the factors folded into `skill_mult` (skill, exploit, flank, splash, counter, weaken, empower: wider than the spec's flank/splash/counter) as trace-only parts; op and order unchanged." STRUCTURE: test `kits/rpg/crates/rpgkit_rules/tests/take_damage_pipeline.rs`. No golden re-recorded, no HISTORY row. Questions (suggestions, not blocking): (a) a pipeline ceiling/clamp step for the heal cap and the status-chance clamp, or keep integer steps after the run; (b) wire a traced display (Dojo / forecast) to `compute_traced_folded` now or in a later A2.2 slice; (c) is `splash` meant to be a new factor (A1/area work)? Recommended: a later slice traces status chance with a small separate struct, not in R1.

### FB-103: R1-BAL H6 tuning landed: timeline feel 79.5 % over 40 seeds via seed r1bal

- **Spec**: `docs/READY.md` R1.1-3 `R1-BAL` part (3); FB-98 Option A and C (@7d67fbe9)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Seed `games/emberwake/data/seeds/r1bal.json` (5 steps, idempotent): `units.json` `tl_enforcer` armor 35 -> 15, `tl_captain` level 1 -> 2, `tl_footman` armor 25 -> 40; `encounters.json` `tl_skirmish` max_rounds 10 -> 12; `rulesets.json` `timeline` `stagger_windup_mult` 2.0 -> 1.0; `know_potency` stays 0.0 (Option C). Data diff is exactly those 5 values; content check ok; `ruleset_switches` test replays `r1bal.json` after `s22s3.json`. `tools/fc.sh sim feel 5 ruleset=timeline seeds=40`: windup 65 %, reach 75 %, armour 65 %, flank 100 %, skirmish 92.5 % (37 won, 1 lost, 2 capped), overall 79.5 %: matches FB-98. No golden moved: the R1-BAL row names `rules_golden`, `timeline_golden`, `sim_golden`, `phasefold_golden`, `rules_contract_golden`, but the first three read frozen fixtures (FB-96: frozen by design until W2-4), phasefold does not play the `tl_` fights, and the contract golden records metadata, not values. Gate: workspace tests 2426 passed, 0 failed; content check ok; 0 golden diffs. Review: planner self-review only (data-only seed, values verbatim from FB-98); no change-reviewer run (desktop time box before the cloud handover).
- **Recommendation**: HISTORY: "R1-BAL H6: timeline tuning (`tl_enforcer` armor 15, `tl_captain` level 2, `tl_footman` armor 40, `tl_skirmish` max_rounds 12, timeline `stagger_windup_mult` 1.0); timeline feel 79.5 % over 40 seeds (65 / 75 / 65 / 100 / 92.5); no golden moved (frozen fixtures)". GAME_BRIEF timeline: `stagger_windup_mult` 1.0. BALANCE_REPORT (Design already added the R1-BAL section): add the measured result line above. With FB-100 and FB-103, R1.1-3 `R1-BAL` is complete, so Phase R1.1 is done.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-10 are closed and moved to the archive (2026-11-08). New notes start at UN-11.
