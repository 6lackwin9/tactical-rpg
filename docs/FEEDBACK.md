# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 and -S4. |
| R1 path (READY R1-6..R1-8) | W2-1 done; W2-2 slices 1-2 done; slice 3 (`U3.2-S5`, proposed as covered: FB-82) in review | `claude/u32s5-carry-burden` | Then W2-2 slices 4-5, W2-3, then the R1 release PR. |
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

- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-73: U1.2-S2 (W2-1 switch 1) built: `MIN_POWER = 1` is a named floor; one reading; docs delta

- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-74: U2.2-S3 (W2-1 switch 2) built: classes are stored as shares; one UI question; docs delta

- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-75: U1.2-S3a (W2-1 switch 3a) built: crit reads PERC; five named re-records; a balance note

- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-76: U1.2-S3b needs a number: the KNOW-to-potency weight (built at 0 meanwhile)

- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-77: U1.2-S3c (W2-1 switch 3c) built: speed splits into initiative and move rate; three named re-records

- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

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

### FB-81: U3.2-S4 (W2-2 slice 2) built: reversible item grants, OFFHAND1, empty family bonus fields; values to pick

- **Spec**: `docs/sessions/U3_GEAR.md` (Level 2-3, OFFHAND1); FB-35, FB-61
- **Kind**: conflict
- **Status**: open
- **Finding**: Items may grant `skill` and `stat` (not `proficiency`: training stays the class's and drills'); gear grants are rebuilt on every sync and leave with the item (`Unit::gear_skills`, source `gear_effect`). A worn item's skill is open while worn, takes no deck or loadout slot, follows the hand rule, and shows read-only on the deck screen. Family fields `level2_bonus` (`{}`, or one of `{"crit": 1-10}` / `{"burden": 1-3}`, from proficiency 2) and `level3_passive` (an always-on passive, from 3, applied as the item's) are built and checked, all shipped empty. Seed `u32s4` rewrote the `shd_tower` and `wpn_glass_wand` notes without block cost (FB-61). Named re-record: `checks_golden` (2 `wpn_claymore` lines: the item effect list gains `skill`, `stat`); every other golden byte-identical. Tests: `item_grants.rs` (9), `family_mastery.rs` (3), a deck-screen GUI test.
- **Recommendation**: Pick values (Build proposal): Level 2 crit +2 for sword, dagger, fist, mace, bow, crossbow, scourge, bone_saw, staff, wand; crit +1 for orb, tome, cloth; burden -1 for axe, hammer, polearm, censer, sprayer, shield, plate, leather. Level 3 passives (new always-on rows): resist stagger 15 (sword, plate), status potency +5 (axe, bone_saw, scourge, sprayer), forced-move +1 tile (mace, hammer, polearm), DEX +1 (dagger, leather), spell potency +0.10 (staff, wand, orb, tome), heal potency +0.10 (censer), forced-move immunity while guarding (shield), status potency +3 (cloth), a signature skill for bow, crossbow, fist. Also: does `level2_bonus` add to or replace the built +1 crit / -1 burden per level? Should a worn item's skill keep the path-level gate (today it skips it; `min_proficiency` and `weapon_types` stay hard)? A Level-3 passive may only carry item effect types. Weapon-context only? List worn skills on the classic Skills tab? `shd_tower`'s description still says "slow to raise". Docs delta: GAME_BRIEF proficiency bullet; HISTORY: "U3.2-S4: item grants reversible, OFFHAND1, family Level 2-3 fields; `checks_golden` re-recorded (item effect list)"; STRUCTURE: the two tests; `kits/rpg/README.md`: the family fields.

### FB-82: U3.2-S5 (W2-2 slice 3): carry and burden already built; the on/off switch needs Design

- **Spec**: `docs/sessions/U3_GEAR.md` (Carry and burden, C01); FB-36 (d); FB-07
- **Kind**: conflict
- **Status**: open
- **Finding**: Built before this slice: burden and carry overload lower initiative first, move rate only past its floor (U1.2-S3c); party-wide carry (`carry.rs`: capacity 30 + sum of members' `STR x 3 + CONS`, load = stash + worn + pouches, overload 1 per 10 over). Campaign battles never apply carry (only the kit Dojo calls `carry::apply`), so carry is effectively off in play. Missing: an on/off switch; turning it on needs answers the spec lacks. Build's proposal: treat the slice as covered for R1, pending Design's choice below; `rpgkit_rules/tests/carry.rs` (4) pins the built rule. This corrects FB-36 (d) ("carry is always on today"): it never applied outside the Dojo, where it is always on and not switchable. No golden moved.
- **Recommendation**: Pick for later: (A, Build's choice) a row-only ruleset field `carry_stat` naming a derived row (formula as data, like `poise_stat`), empty = off; battles apply carry at start with the stash weight; (B) a register rule `carry`, formula in code; (C) nothing until a world wants carry. For A/B: active members only? shown on the party screen? does the Dojo follow the switch? Docs delta: GAME_BRIEF carry line as in the Finding ("today only the Dojo applies it"); STRUCTURE: test `carry.rs`; the party screen's Initiative tooltip mentions carry overload, which applies only in the Dojo; HISTORY: "U3.2-S5: covered by U1.2-S3c and `carry.rs`; switch left to Design".

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-06 are closed and moved to the archive (2026-11-07). New notes start at UN-07.
