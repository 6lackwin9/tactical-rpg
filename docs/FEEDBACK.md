# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5. |
| R1 path (READY R1-6..R1-8) | W2-1 done; W2-2 slice 1 (`U3.2-S3`) in review | `claude/u32s3-soft-wield` | Then W2-2 slices 2-5, W2-3, then the R1 release PR. |
| READY rows 14-20 | queued (post-R1) | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-66 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-07). New items start at FB-67. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-67: U3.2-S2 built: reach and the printed check lists wait on Design; docs delta

- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-68: Squash-landed task branches were never deleted by branch-cleanup: docs delta

- **Spec**: `docs/workflow/cloud.md` "Branch cleanup"; user 2026-11-07 (task branches must be cleaned up after landing)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `branch-cleanup.yml` deletes a branch when it heads a merged PR or merging it into `develop` changes nothing. A squash-landed branch has no PR, and once `develop` edits a file it touched, the merge re-applies old lines, so it is kept for good. The build session now closes each branch after its squash landing with one commit whose tree is `develop`'s (`git commit-tree "origin/develop^{tree}" -p origin/<branch> -p origin/develop`, pushed to the branch): no history rewritten, `develop` untouched, and the next daily run (after the 6 h age limit) deletes it. Done for this session's branches.
- **Recommendation**: `docs/workflow/cloud.md` "Landing a task on `develop`" step 8 and "Branch cleanup": after the squash landing, close the task branch with that commit; drop the "delete those by name with `also_delete`" workaround for squash-landed branches.

### FB-69: S1.2-S1 built: the status table is kit data; two questions; docs delta

- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-70: S1.2-S2/S3 built: status reads, four ruleset numbers, one named contract re-record; docs delta and four choices

- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-71: S2.2-S1/S2 built: poise pips behind `poise_mode` (default `bar`); docs delta and four questions

- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-72: BASERULES1-M2 built: sim ruleset notes and the wider LOCKED scan; one reading; docs delta

- **Spec**: `docs/READY.md` row 21; FB-40, FB-60; `BASERULES1_LIBRARY.md` section 7
- **Kind**: docs-delta
- **Status**: open
- **Finding**: (1) The sim prints the picked ruleset's notes (`Ruleset::notes`) when the command line gives `ruleset=`, or `rules=` on a run with no world fights (the feel mode's own timeline default prints none, so `sim_golden` holds): `RULES error:` lines, then `RULES warning:` lines, right under ARGS (`rpgkit_sim::args::rule_note_lines`, `SimArgs.rule_notes`); `sim help` names it (`RULE_NOTES_USAGE`). A run that may play world fights (`world=`, `enc=world:...`, campaign) with `rules=` alone prints none: those fights play each world's own row switched, so classic's notes would mislead. (2) Reading taken: no `LOCKED` validation of ruleset rows exists in `ruleset.rs`; the only one is the scan test `rpgkit_rules/tests/locked.rs`, so "recursively scan nested" was built as FB-40's scan gaps: it now recurses into subfolders and reads `pub(<scope>)`, `static`, slice-typed and multi-line declarations. `PROMOTED` still matches by name only. Tests: `rpgkit_sim/tests/ruleset_args.rs` (+1), `rpgkit_rules/tests/locked.rs` (+1, a planted nested violation). No golden moved.
- **Recommendation**: Confirm the reading of (2), or say what a ruleset-row `LOCKED` check should reject. Docs delta: `engine/docs/authoring/rule_library.md` (or the BASERULES notes): "`tools/fc.sh sim` with `ruleset=` or `rules=` prints the ruleset's notes as `RULES error:` / `RULES warning:` lines under ARGS"; `docs/STRUCTURE.md` (rpgkit_rules tests): "`tests/locked.rs`: every numeric public constant or static under `src/` (subfolders included) is in `LOCKED` or `PROMOTED`".

### FB-73: U1.2-S2 (W2-1 switch 1) built: `MIN_POWER = 1` is a named floor; one reading; docs delta

- **Spec**: `docs/READY.md` W2-1 (1); FB-45; `docs/sessions/U1_STATS.md` feasibility item 2
- **Kind**: docs-delta
- **Status**: open
- **Finding**: The attack power's floor at rounding point one was already 1 in the damage pipeline (FB-45: any power below 1 counts as 1 before the multipliers). It is now the named constant `rpgkit_rules::pipeline::MIN_POWER = 1`, used by `DAMAGE`, and listed in `LOCKED` ("attack power floor at rounding point one"). Reading taken on "schema min": the unit field `power` keeps `min: 0`, since 0 means "derive it" (weapon power + unit power); a schema minimum of 1 would break that default, so the floor is stated in code (FB-45's second option) and the schema is unchanged. The other authored power fields keep `min: 0` for the same reason: `items.base_power` (0 = not offensive: armour, shields; the Dojo tests `base_power > 0`) and `skills.power_mult` (0 = a debuff-only attack); `rulesets.flank_power_mult` already has min 1. No shipped row authors a power below 1. Named golden re-record: `rules_contract_golden` only, +1 line (`locked pipeline.rs::MIN_POWER damage`); rules, timeline, sim, checks, journey, store, rng_hash and PHASEFOLD byte-identical. Test: `rpgkit_rules/tests/damage_pipeline.rs` (+1).
- **Recommendation**: Confirm the reading (or ask for a schema note on `power`, which would be a schema text change). Docs delta: HISTORY: "2026-11-07 U1.2-S2: `rules_contract_golden` re-recorded on purpose (+1 locked line `pipeline.rs::MIN_POWER`): the attack power floor at rounding point one is the named constant `MIN_POWER = 1`; no behaviour change"; GAME_BRIEF damage: "attack power is floored at `MIN_POWER` (1) at rounding point one; a unit `power` of 0 means derived".

### FB-74: U2.2-S3 (W2-1 switch 2) built: classes are stored as shares; one UI question; docs delta

- **Spec**: `docs/sessions/U2_CLASS.md` feasibility items 1 and 6; FB-44, FB-46
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Engine: `aegis_content` field type `float_stats` (a stat -> number map at full float precision). Kit schema: classes gain `base_shares` and `growth_shares` (`float_stats`, 0..1); a share row stores no `base_stats` / `growth`, which stay stored fields for integer rows and the computed view on share rows (`rpgkit_content` `class_storage`). `auto_spend` takes the growth share vector (unit build, the party Auto button, `progression::auto_for`), compared in billionths so whole-point rows spend exactly as before. Data: seed `u22s3` re-saved the 7 classes as shares (stat / 62, growth / 5); stats at levels 1-10 and every spend identical, `content check` unchanged, a second apply byte-identical. The store test that clones `warrior` now turns its copy back into an integer row. No golden moved. Tests: `aegis_content/tests/float_stats.rs` (3), `rpgkit_rules/tests/auto_spend.rs` (4), `emberwake_rules/tests/unit_budget.rs` and `rpgkit_content/tests/class_shares_checks.rs` updated.
- **Recommendation**: Decide (UI): on a share row the Forge / editor shows empty `base_stats` / `growth`; should it show the computed view [Build default: leave for a UI row]? Docs delta: `engine/README.md` `aegis_content` row: "field type `float_stats`: a stat -> number map at full float precision"; `docs/STRUCTURE.md`: `stats.rs` `auto_spend` reads the class growth shares, tests `aegis_content/tests/float_stats.rs`, `rpgkit_rules/tests/auto_spend.rs`; `kits/rpg/README.md`: class `base_shares` / `growth_shares` are schema fields, a share row stores no `base_stats` / `growth`, `auto_spend` weighs `growth_shares`; GAME_BRIEF stats: "classes are stored as shares since U2.2-S3; the default spend follows the growth shares"; HISTORY: "2026-11-07 U2.2-S3: class rows moved from integer `base_stats` / `growth` to stored `base_shares` / `growth_shares` (seed `u22s3`); stats identical at levels 1-10".

### FB-75: U1.2-S3a (W2-1 switch 3a) built: crit reads PERC; five named re-records; a balance note

- **Spec**: `docs/sessions/U1_STATS.md` ("crit from PERC"; parameter table); FB-07
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `crit_rate` = `CRIT_BASE + max(0, PERC - CRIT_PERC_FLOOR) x CRIT_PER_PERC` (+ the weapon proficiency bonus), the weights kept (5, floor 10, 1 per point, x1.5); `CRIT_PER_DEX` / `CRIT_DEX_FLOOR` renamed `CRIT_PER_PERC` / `CRIT_PERC_FLOOR` (still locked). Named golden re-records, each intended: `rules_golden` (crit columns only: 113 of 125 unit rows, the `crit_rate` column on 1316 of 1652 damage rows; forced crits unchanged), `timeline_golden` (fight `tl_flank` seed 42), `sim_golden` (18 of 22 runs), `rules_contract_golden` (2 renamed locked lines), PHASEFOLD (the shipped wave fights roll different crits). checks, journey, store resave and rng_hash byte-identical. Test: `rpgkit_rules/tests/crit_rate.rs` (2). Balance: shipped crit chances move with PERC (warrior 6 to 10 %, ranger 24 to 16 %, goblin brawler 10 to 7 %); no tuning change, as the spec keeps the weights.
- **Recommendation**: A balance look at crit after W2-1 (rangers lose crit, warriors gain it), with the balance lane, before or after R1 as Design prefers. Docs delta: GAME_BRIEF crit line: "Crit = 5% + 1%/PERC above 10 (`CRIT_BASE`, `CRIT_PER_PERC`, `CRIT_PERC_FLOOR`; U1.2-S3) + 1%/weapon training level above 1, x1.5"; HISTORY: "2026-11-07 U1.2-S3a: crit reads PERC instead of DEX (weights kept); re-recorded on purpose: `rules_golden` (crit columns), `timeline_golden` (`tl_flank` seed 42), `sim_golden` (18 of 22 runs), `rules_contract_golden` (2 renamed locked lines), PHASEFOLD"; PORT_NOTES damage row: "crit reads PERC since U1.2-S3 (Godot: DEX)". Still saying DEX drives crit: `docs/specs/BASERULES1.md:28,42`, `docs/specs/BASERULES1_BOUND.md:97`.

### FB-76: U1.2-S3b needs a number: the KNOW-to-potency weight (built at 0 meanwhile)

- **Spec**: `docs/sessions/U1_STATS.md` ("status potency from KNOW, against resist"; parameter "potency weight (KNOW)"); `S1_STATUS.md` section 2; FB-07
- **Kind**: conflict
- **Status**: open
- **Finding**: No spec, code, data or port note gives the weight. Today `apply_chance` = `clamp(potency - INT x 2.5 - resist_for(status), 0, 100)`; status potencies run 40-90, level-1 KNOW 7-15, INT resist 15-42. Built meanwhile, neutral: a row-only ruleset number `know_potency` (per KNOW point, range 0..3, default 0 = the spec's "potency off" state) adds `know_potency x attacker KNOW` on the actor paths (single-target and area skill statuses, the Dojo); ground effects, the trigger `inflict` effect and terrain / arena hazards keep the plain potency. Only `rules_contract_golden` moves (+1 tunable line).
- **Recommendation**: Pick the weight, and Build lands it as its own named balance commit (rules, timeline, sim goldens and PHASEFOLD re-recorded): (B) KNOW x 1 (+7-15 at level 1); (C) KNOW x 2.5, mirroring INT resist (large); or 0 for R1, tuned after the balance pass (Build's recommendation, with FB-75). (A threshold like crit's is out: U1 says all linear, no thresholds.) Also: should grounds use the placer's KNOW and trigger `inflict` the holder's? Docs delta: HISTORY "2026-11-07 U1.2-S3b: `rules_contract_golden` re-recorded on purpose (+1 row-only tunable `know_potency`, default 0, 0..3)"; GAME_BRIEF status chance: "+ `know_potency` x attacker KNOW on skill and Dojo rolls"; STRUCTURE: test `rpgkit_battle/tests/know_potency.rs`.

### FB-77: U1.2-S3c (W2-1 switch 3c) built: speed splits into initiative and move rate; three named re-records

- **Spec**: `docs/sessions/U1_STATS.md` (Decided; feasibility item 4); FB-07
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Initiative = the old speed (`max(1, 8 + DEX - burden) - carry overload`, floor 1); move rate = the old curve `clamp(floor((v + 2) / 5), 1, 4)` on `8 + DEX - excess`, excess = the load past initiative's floor of 1. `Unit.speed` became `initiative` + `move_rate`; the row and tuning key `speed` is read as initiative (fixed-stat units move as before); the host JSON adds `initiative` and `move_rate`; the sim `speed` column became `init` + `move`; GUI labels say Initiative. Four burdened timeline units (`tl_vanguard`, `tl_halberdier`, `tl_enforcer`, `tl_captain`) move 4 tiles instead of 3. Named re-records: `rules_golden` (key rename; `move_per_block` on those 4), `timeline_golden` (`tl_skirmish` seed 7), `sim_golden` (stats columns, help, feel `tl_skirmish`). Test: `rpgkit_rules/tests/move_rate.rs` (4).
- **Recommendation**: Answer: (1) world tuning `speed` is now an initiative delta only: add a `move_rate` key? (2) promote the move curve, its cap 4 and the initiative weight to ruleset numbers? (3) balance: `tl_skirmish` seed 7 flips from won to capped. (4) confirm the spill reading: the excess lowers the curve's input (5 points a tile), not the tiles 1:1; at 8 + DEX = 18 the first excess point already drops a tile; GUI: check "Initiative NN" still fits the hover card and portrait lines. Docs delta: GAME_BRIEF stats: the initiative and move rate formulas above; HISTORY: "2026-11-07 U1.2-S3c: speed split into initiative and move rate (burden lowers initiative first); re-recorded on purpose: `rules_golden`, `timeline_golden`, `sim_golden`"; PORT_NOTES: "speed split into initiative + move rate (Godot: one speed)".

### FB-78: U1.2-S4 (W2-1 switch 4) built neutral; the spec HP formula is held (user 2026-11-07, temporary)

- **Spec**: `docs/sessions/U1_STATS.md` feasibility item 6; `U2_CLASS.md` feasibility item 2; FB-09, FB-11
- **Kind**: conflict
- **Status**: open
- **Finding**: Built: the per-tier factors are the kit table `rpgkit_rules/src/tiers.json` (hp, power, AI skill cap; values unchanged), read by formulas as `tier_hp` / `tier_power`; `power_scale` became `tier_power` (hits unchanged); `MINION_SKILL_CAP` moved into the table. Held: the spec's `max HP = 60 x tier_hp + stats` roughly doubles minion HP and cuts boss HP by a quarter (goblin brawler 108 to 234, warlord 543 to 392; 32 of 72 wave fights flip), because tier does not change the stat budget: every enemy has its class's full budget at its level. The user chose (2026-11-07) to keep today's `(60 + stats) x tier_hp` for R1, **as a temporary measure**. The user's intent: enemies and static units should have stats so that every unit behaves the same and gets its real HP and damage from its stats by one consistent rule, not hardcoded tier multipliers. Named re-records: `rules_golden` (snapshot keys `hp_scale` / `power_scale` renamed only), `rules_contract_golden` (-1 locked line). Tests: `tier_factors.rs` (5), `tier_checks.rs` (1), `tier_ai_cap.rs` (1), `derived_stats.rs`.
- **Recommendation**: Design: spec per-tier stat budgets (minions fewer stats, bosses more) with the 6-18 per-stat rule restated for them (FB-11: a minion budget near 25 breaks it), then land the HP formula and the budgets together after R1. Docs delta: GAME_BRIEF: tier factors in `tiers.json`, formula inputs `tier_hp` / `tier_power`; HISTORY: "2026-11-07 U1.2-S4: tier factors became the kit table `tiers.json` (values unchanged); `rules_golden` key rename and `rules_contract_golden` (-1 line) re-recorded on purpose; the `60 x tier_hp + stats` HP form held (temporary, user)"; STRUCTURE: `tiers.json` and the three tests. GAME_BRIEF `:249` still names `StatRules.TIERS` / `MINION_SKILL_CAP` (now `tiers.json`, `stats::tier_row`); the kit schema's unit `tier` help says "boss 170% / 120%" but the values are 1.5 / 1.1.

### FB-79: U1.2-S5 (W2-1 switch 5) built: Composure as a data pool, off; Shaken and the 3-stat pack need Design

- **Spec**: `docs/sessions/U1_STATS.md` (Composure acceptance test; stats read roles); FB-08
- **Kind**: conflict
- **Status**: open
- **Finding**: Generic, data-only: a `unit_meters` row takes `max_stat` (a derived row as each holder's max), `start_full`, `delta_max` (1-100, default 10 = today's clamp) and `triggers` held by every holder; bands take `from_pct`; a ruleset's row-only `unit_pools` lists the meters every unit holds (rule `unit_meters`); a REST skill may carry `meters`; band statuses may be any kit status. The `within 2` filter already existed as `when.range` on ally events (FB-08 missed it). Composure is the seeded row (`u12s5`: `composure` max, full each fight, -10 when a friend within 2 is knocked, below half `weaken` 10); off, as no ruleset lists it. Named re-record: `rules_contract_golden` (+1 line `unit_pools`); every other golden byte-identical. Not built: (1) Shaken at 0 holds no status (no `shaken` in the frozen table; an 18th is reserved for `guard`); (2) the +5 on Rest is not seeded on the shipped `rest` skill; (3) the 3-stat pack proof cannot pass: there is no stat-role mapping (`rpgkit_data::STATS` is a 6-id constant used about 50 times, and the HP, initiative, crit, resist, armour, potency, carry and unlock formulas name stats directly): an L-sized design-and-build task. Tests: `rpgkit_battle/tests/data_pool.rs` (5), `rpgkit_content/tests/unit_pool_checks.rs` (4).
- **Recommendation**: Decide: Shaken = `weaken` higher, `stagger`, or a new status after `guard`; weaken 10 for the low band; `unit_pools` party-only?; a `rest` trigger event instead of REST `meters`? Make the stat-role mapping (3-stat pack) its own task after R1 (MASTERPLAN1_ROWS already maps it to BASERULES1-PP). Note: while Shaken holds no status, a pool at 0 drops `weaken` (better than a pool at 1), and the row's description promises the Rest +5 that is not seeded; fix both with the Shaken answer (the row is inert). Docs delta: GAME_BRIEF and `kits/rpg/README.md`: the meter pool fields, `unit_pools`, `meter::pool`; HISTORY: "2026-11-07 U1.2-S5: `rules_contract_golden` +1 line (`unit_pools`); Composure seeded as an inert pool".

### FB-80: U3.2-S3 (W2-2 slice 1) built: soft wield penalty; party-screen summary; docs delta

- **Spec**: `docs/sessions/U3_GEAR.md` (FB-33, FB-34); FB-63 (2)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Any unit wears any item. Per worn item, factor = (1 - stat-deficit ramp) x (0.75 if untrained); the ramp (15 % a missing point, cap 60 %, after `level x req_discount_per_level`) now also hits armour; untrained adds `untrained_burden_add` burden (initiative first). Row-only ruleset numbers: `untrained_power_mult` 0.75 (0.25..1), `untrained_burden_add` 1 (0..4), `req_discount_per_level` 2 (0..4). Unarmed, accessories, untyped gear and classes with no proficiency list carry no penalty; skill gates stay hard. The four hard equip gates are soft (`gear::trained_for`); `wears_untrained` retired (seed `u32s3` removed it from the Purifier). FB-63 (2): `UnitState::trains` and deck ownership read `progression::member_passives`. Party screen (mock): a red factor badge on a penalised slot or stash row, and its card says "Wield penalty: weapon power and armour x0.30; untrained in dagger (x0.75, +1 burden); 5 stat points short (-60%)" (screenshots sent to the user in chat). Named re-records: `rules_golden` (`dojo_warrior` armour 13 to 12), PHASEFOLD (Mara Voss's brigandine armour 6 to 5; 39 of 72 fights, 2 outcomes), `checks_golden` (class training errors gone), `rules_contract_golden` (+3 tunable, +3 promoted, +2 locked lines). No shipped fight wears untrained gear. Tests: `rpgkit_rules/tests/wield_penalty.rs` (9); drills tests rewritten.
- **Recommendation**: Answer: a badge in the Dojo too? a content warning for untrained gear? the ranges. Then the logic lane removes the `wears_untrained` schema field. Docs delta: GAME_BRIEF proficiency bullet as above; HISTORY: "U3.2-S3: soft wield; training gates and `wears_untrained` retired; re-recorded on purpose: `rules_golden`, PHASEFOLD, `checks_golden`, `rules_contract_golden`"; PORT_NOTES: `can_equip_gear` became `gear::trained_for` (and `:153`, which still calls `can_wear` the training gate); `docs/specs/BASERULES1.md:99` still says level 0 cannot be worn; the spec's main text says +1 burden per missing point, the resolution (built) +1 when untrained; the Purifier (no encounter uses it) now wears its apron untrained (+1 burden); STRUCTURE: test `wield_penalty.rs`.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-06 are closed and moved to the archive (2026-11-07). New notes start at UN-07.
