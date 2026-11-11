# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 to PR4 rows (R1 to R4 releases, the Post-R4 queue PR4-1..PR4-9) | released or done | `main` / `develop` | moved to `docs/archive/FEEDBACK_CLOSED.md` (Build Status rows, 2026-11-10) |
| PR5-1 to PR5-6, PR5-8, PR5-16, PR5-17, PR5-20 to PR5-25 | done (FB-168 to FB-189, FB-194 to FB-196, answered) | `develop` | moved to `docs/archive/FEEDBACK_CLOSED.md` (Build Status rows, 2026-11-10) |
| PR5-7 `PK1-S3` | done (FB-190 data; FB-191 balance run, 6 findings for Design) | `develop` | -
| PR5-18 `WEAPONROSTER1` | done (FB-192; FB-193 sims and questions): class lineup, default loadouts, placeholder `warrior/2h_polearm`, FB-188 rulings; `phasefold_golden` re-recorded | `develop` | -
| PR5-26 `AUDIT-PR5-BUGS` | done (FB-197; FB-198 limit-blocker for the RANK_KEYS clause and questions): audit 1A bugs in five groups; 0 golden diffs | `develop` | -
| PR5-27 `PK1-S3-FIX` | data done (FB-200); the PR5-26 Vent bug fixed (FB-202); Vent numbers wait for Design (FB-201) | `develop` | -
| PR5-28 `R14-SPEC-CLEANUP` | items (2)-(3) done (FB-203); item (1) `spell_reach` parked (FB-204 limit-blocker) | `develop` | -
| PR5-29 `LAYER-DOWN-1` | parts (3) tools and (1) kit names done (FB-205; `rules_contract_golden` re-recorded); part (2) engine waits (FB-206) | `develop` | -
| PR5-30 `LAYER-DOWN-2` | done (FB-207): gear families and terrain types from data, Dojo presets and cast as data rows; 0 golden diffs | `develop` | -
| PR5-26b `AUDIT-PR5-FOLLOWUP` | done (FB-208): non-rank keys in rank modifiers refused, a crisis set skips that round's decay, Vent -50 / 10%; 0 golden diffs | `develop` | -
| PR5-9 `PK2-S2` | done (FB-209; `checks_golden` re-recorded: item types gain GRAFT); questions in FB-210 | `develop` | -
| PR5-10 `PK2-S3` | done (FB-211; FB-212 questions): recipes kind, NPC `recipe` service and pane, member and item effects; drill / repair / reforge refused until PR5-12 / PR5-13b; 0 golden diffs | `develop` | -
| PR5-12 `PK2-S4` | done (FB-213; `checks_golden` re-recorded: item types gain MASTERY_BOOK; FB-214 questions): weapon mastery, techniques, the family keystone, `durability.from_act` | `develop` | -
| PR5-13a `PK2-S5a` | done (FB-215; `rules_contract_golden` re-recorded: the `durability` rule; FB-216 questions): weapon instances behind `durability.from_act` and the `durability` rule | `develop` | -
| PR5-13b `PK2-S5b` | done (FB-217; FB-218 questions): durability spend, dulled at 0, sim keys `durability=` and `dulled=`; 0 golden diffs | `develop` | next: PR5-19

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-206 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-11). New items start at FB-207. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-207: PR5-30 `LAYER-DOWN-2` landed (gear families and terrain types from data; Dojo presets and cast as data)

- **Spec**: READY Section 1 row PR5-30; audit 2.1 (the `PR5-30` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-208: PR5-26b `AUDIT-PR5-FOLLOWUP` landed (rank-modifier keys, crisis decay, Vent numbers)

- **Spec**: READY Section 1 row PR5-26b; `FB-198` B9 Option B and Q2 Option A; `FB-201` (a) Option A (the `PR5-26b` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-209: PR5-9 `PK2-S2` landed (body slots, grafts, the floor table, the keystone graft, remains; `checks_golden` re-recorded)

- **Spec**: READY Section 1 row PR5-9; `docs/sessions/PK2_GRAFTS.md` section 3; `FB-167` Q6 (the `PR5-9` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-210: PR5-9 readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` section 3 (@ the `PR5-9` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: (1) the channeled-active limit is the graft count for a unit with a body, bounded by the deck's 3 actives (a deck without a deck rule could mark more); a member with 0 grafts in a catalog with body slots marks 0, and **once PR5-19 ships body slots, members' saved channeled marks beyond their graft count are cut at load with a warning** (spec-intended: "PK2 replaces it with one per installed graft"; the balance lane should know); (2) item type spelled `GRAFT` (the uppercase convention; the row says `graft`); (3) every member that dies with the field held leaves remains, grafted or not ("a fallen body yields one organ"); (4) "or a lost one where the fallen are recovered under the roster rules" is not built: a lost fight settles through the downed rules with no recovered body today (PK3 bleed-out / revive, PR5-11); (5) Tier III's +1 track step on install, the excision's max HP cost and all costs are PR5-10's recipes (install and remove are free state changes); (6) several class slots are allowed (each opens with the class rite); (7) a non-keystone graft's skill takes an active deck slot under the deck rule; (8) `floors` is allowed on any passive's meter effect, not only on `body_effects`; (9) **remains' type**: `remains` (class and character) accepts GRAFT items only, while PK2 speaks of a "core" (an Apex core, a class core); if a core is a material that a surgery turns into a graft, both filters should take that material instead.
- **Recommendation**: confirm (1)-(8) as built; for (9) name what `remains` holds (the graft itself, recommended by the built filter, or the surgery's core material) so PR5-19's data matches.

### FB-211: PR5-10 `PK2-S3` landed (recipes, the NPC `recipe` service and pane)

- **Spec**: READY Section 1 row PR5-10; `docs/sessions/PK2_GRAFTS.md` section 4 (the `PR5-10` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-212: PR5-10 readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` section 4 (@ the `PR5-10` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Readings: pending effects are refused rather than no-ops (no data can author an effect that silently does nothing); target `item` added for salvage; item `theme` added; "tier" is the item's rarity and Scrap is the purse gold; the surgery's core is a cost item (nothing ties the installed graft to a cost item; data lists both); `doctrine` {axis, delta} applies nothing; the pane has no family picker, so a role holding several families (minor) cannot be trained from it yet (the kit refuses with "Pick a minor family (...)"); recipes whose `requires` fail are not shown.
- **Recommendation**: Questions, built default first: (a) a Tier III surgery (`track +1`) on a member one stage before departure is refused (alternative: allowed, the member departs at the next settle); bone-scraping (`track -1`) at stage 0 is refused; (b) `doctrine` records nothing until PK4 (alternative: a party key now); (c) training recipes whose role the member's class lacks are listed and refused with a reason (alternative: hidden); (d) a family picker in the pane before PR5-19 ships minor training (a small game row).

### FB-213: PR5-12 `PK2-S4` landed (weapon mastery and techniques; `checks_golden` re-recorded)

- **Spec**: READY Section 1 row PR5-12; `docs/sessions/PK2_GRAFTS.md` section 5 (the `PR5-12` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-214: PR5-12 readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` section 5 (@ the `PR5-12` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Shipped data now gives one new content warning: `classes:tl_drill` (the timeline drill class, outside the PR5-18 lineup) has 4 masterable families (sword, mace, dagger, polearm); inert while mastery is closed, and trimming it risks the timeline and sim goldens, so no data was touched. Readings: the keystone opens at mastery 3 plus the forge rite while any weapon of the family is wielded (the spec says Bram forges it "into a masterwork weapon of that family"; the row says "mastery 3 plus the forge-rite flag"); nothing sets `forge_rite` yet (only `UnitState::set_forge_rite`); the keystone is fixed at build from the saved main hand while techniques follow a mid-battle swap; a technique skill the class also owns through its path or deck is not refused by a check (it would be standard-rounds-only and grip-greyed); no game GUI reads a book yet (`mastery::read_book` is kit-only).
- **Recommendation**: Questions, built default first: (a) `[PROVISIONAL]` kit constants not in the spec: XP 100 / 300 / 600, use XP 10, book default 25 (alternative: a ruleset or `tuning.json` field before balance); (b) `tl_drill`: accept the warning (alternatives: exempt timeline classes, or trim it in a data row with the golden re-record named); (c) keystone on any family weapon (alternative: only a masterwork, once PR5-13a lands); (d) the forge-rite source: a recipe effect `forge_rite` in PR5-13b or a quest flag in PR5-19; (e) a content check refusing a technique that is also a path or deck skill (a small kit row); (f) a book-reading action in the inventory GUI (a small game row).

### FB-215: PR5-13a `PK2-S5a` landed (weapon instances; `rules_contract_golden` re-recorded)

- **Spec**: READY Section 1 row PR5-13a; `docs/sessions/PK2_GRAFTS.md` section 6 (the `PR5-13a` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Kit rule `durability` (extension, system `gear`, requires nothing; the battle reads nothing of it until PR5-13b, a test plays a fight the same with or without it). `rpgkit_world/src/weapons.rs`: `Weapon` {uid, item, durability, masterwork}; `active` = the manifest's `durability.from_act` AND the campaign act at or after it AND the played world's ruleset lists `durability` (otherwise nothing changes; shipped data has neither, so saves are byte-identical). Every owned item of type WEAPON (main and off hand, foci included; shields, armour and class defaults stay plain) gets an instance; `equipment` and `stash` keep their item ids and instances ride beside them: member save key `weapons` {slot: instance}, party `stash_weapons` [instance], each written only when not empty (no save bump). `weapons::sync` at the act line (`set_act`), on load (after `sanitize`), on every save write (a copy), at the fight start, after a battle's settle (loot) and after a shop buy, sell or trade: new instances at full durability, a slot keeps its instance while it holds the item else takes the best free one (masterwork, then durability), a sale drops the worst stashed one; uids `w<n>` past the highest in use. `weapons::wield` (equipment by uid; syncs first); `roster::depart` stashes the member's instances; `Growth::masterwork` from the main hand's instance (opens the weapon's own technique, PR5-12). Item family and item field `durability` 0-999 (`omit_default`, WEAPON rows; resolved `family_durability`, out of the stored form; `ItemDef::max_durability`, 0 = never wears). Validation: a uid is an id used once, a known item, durability 0 or more. Tests: `rpgkit_world/tests/weapon_instances.rs` (9, incl. the act-line conversion and an Act 1 save loading unchanged), `rpgkit_battle/tests/weapon_instances.rs` (2), `rpgkit_content/tests/weapon_durability.rs` (2). **`rules_contract_golden` re-recorded** (named by the row): the `rules` values list gains `durability` and one line `rule durability extension system=gear bundle=- requires=- conflicts=-` (2 lines, no other change); 0 journey diffs, no other golden moved. Review: change-reviewer PASS (fixed: a `w<n>` number read with at most 9 digits so the next uid never overflows, `wield` syncs first).
- **Recommendation**: HISTORY: "2026-11-11 PR5-13a (PK2-S5a): `rules_contract_golden` re-recorded; the `rules` values list gains `durability` and a `rule durability` line (2 lines, no other change); intended." STRUCTURE: `rpgkit_world/src/weapons.rs` and the three test files. PORT_NOTES (save keys): member `weapons`, party `stash_weapons`; data field family/item `durability`, resolved key `family_durability`. GAME_BRIEF / `combat_hooks.md`: weapon instances under `durability.from_act` and the `durability` rule; class defaults are not instances. **READY**: PR5-19 needs, besides `durability.from_act: 2`, the timeline ruleset row listing `durability` and family `durability` values, or instances stay off and never wear. `kits/rpg/README.md`: done.

### FB-216: PR5-13a readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` section 6 (@ the `PR5-13a` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Readings: the spec says equipment and the stash refer to instances "by uid"; the build keeps item ids there and stores instances beside them by slot and in `stash_weapons` (every reader, GUI, build, sim and shop works unchanged; additive keys); max durability (section 7's "per family, overridable per item") lands here to define "full", with no kit default; without a UI choice the best copy fills a slot and the worst leaves the stash; uids are reused after an instance is gone (unique among live ones, no saved counter; PR5-13b must not track spend by uid across a sale); GUI equips, quest gifts and salvage catch up at the next fight start, save or load; a new game starting at or past `from_act` gets instances at its first fight, save or shop, not at `set_act`; weapons in a member's `carried` list are not instances; no content warning yet for a ruleset listing `durability` without a manifest `from_act`. `repair` / `reforge` (and target `weapon`) stay in `RECIPE_PENDING`: wiring is feasible now (`Pick` gains a uid, `weapons::find`), but repair's cost scales with missing durability, PR5-13b's spend model.
- **Recommendation**: Questions, built default first: (a) class-default weapons never wear (alternative: they become owned instances in Act 2, changing the can't-unequip model); (b) off-hand foci (orb, tome) are instances as item type WEAPON (alternative: main hand only, a one-line change); (c) instances need both the manifest key and the ruleset rule (alternative: the manifest key alone); (d) READY PR5-13b should name `repair` / `reforge` (FB-211).

### FB-217: PR5-13b `PK2-S5b` landed (durability spend, dulled, sim keys)

- **Spec**: READY Section 1 row PR5-13b; `docs/sessions/PK2_GRAFTS.md` section 7 (the `PR5-13b` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Under the rule `durability` with `durability.from_act` reached, a member's wielded instances that wear (max durability above 0) ride into battle (`weapons::wear_of` → `Growth::wear` → `Unit::wear` {slot: `gear::Wear` {start, spent}}; `roster::field_unit` clears it before the act; a session without the rule clears all wear, `wear::clear_without_rule`). `rpgkit_battle/src/wear.rs`, from `resolver::count_use` once a command's costs are paid, charges the weapon the skill swings (`Unit::swung_slot`: the off hand for an off-hand skill, else the ready weapon): a standard attack skill `SPEND_ATTACK` 1, a technique or the family keystone `SPEND_TECHNIQUE` 2; basics (basic or system skills, the deck rule's basics), support skills, items and every Channel-round action 0. At 0 the weapon is **dulled**: weapon power x`DULLED_POWER_MULT` 0.5 (`damage::attack_power`, `AttackPower::dulled`), still usable; the Dojo breakdown shows "Dulled x0.50". `HeroEnd::worn` (`wear::spent`); `settle_party` takes the spend off the instances first (before a departure or `forbid_hands` moves them), held at 0. Sim keys `dulled=power|lock` (`World::dulled_lock`: lock greys a technique whose swung weapon is dulled, "Weapon dulled") and `durability=0-999` (seeds every ally's wielded weapons at each fight start, `wear::seed`, under the rule only); the ARGS header names them only when set, help lines print apart from `usage`; `FightRes::{worn, dulled}` recorded (not printed). Tests: `rpgkit_battle/tests/durability.rs` (9), `rpgkit_world/tests/weapon_instances.rs` (+4), `rpgkit_sim/tests/dulled.rs` (3). Gate 3049 passed, 0 failed, 0 warnings; **0 golden diffs**. Review: change-reviewer PASS (fixed: the sim `durability=` seed so the alternative can be run, the field-time act gate, lock follows the swung weapon, off-hand / settle-order / Dojo tests).
- **Recommendation**: STRUCTURE: `rpgkit_battle/src/wear.rs`, the two new test files. GAME_BRIEF / `combat_hooks.md`: "Durability (PK2-S5b): a standard attack skill wears the swung weapon by 1 and a technique by 2; basics, support skills, items and Channel-round actions cost nothing. At 0 the weapon is dulled (weapon power x0.5, still usable). The settle takes the spend off the instances, held at 0." BALANCE / sim docs: keys `durability=N` and `dulled=power|lock`. HISTORY: none (no golden re-recorded). `kits/rpg/README.md`: done.

### FB-218: PR5-13b readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` section 7 (@ the `PR5-13b` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Readings: a technique costs 2 whatever its kind (support included), the keystone counts as a technique; "support" is any kind other than attack; spend keeps counting past 0 in battle, the settle clamps; lock greys at planning, a technique planned before the weapon dulled earlier that round still resolves; a hero with authored `tuning.power` ignores weapon power, so dulling does not touch it; no settle result key or save key for wear; `masterwork_main` is not act-gated (a masterwork exists only on an instance, made from the act line on). **Number placement**: `DULLED_POWER_MULT` lives in `rpgkit_data` (re-exported as `gear::DULLED_POWER_MULT`), and the spend costs are `rpgkit_battle::wear` constants, so `rpgkit_rules`' `LOCKED` table (printed in `rules_contract_golden`) does not list them and the row's "0 golden diffs" holds; the `dulled` switch is a sim key, not a ruleset number, for the same reason (same pattern as PK2-S4's `MASTERY_LEVEL_XP`). `dulled=lock` bites nothing in sims today: catalog sim allies have no mastery, so no techniques. Repair / reforge stay pending: the effects are near-trivial now (a uid pick, `weapons::full`, `masterwork = true` behind `requires`), the gap is the cost model.
- **Recommendation**: Questions, built default first: (a) support techniques cost 2 (alternative 0); (b) move the spend costs and the 0.5 factor to data or `LOCKED` before PK2-S6 balance (a named `rules_contract_golden` re-record and HISTORY row) or keep them as code constants; (c) a sim key `mastery=N` (allies' main-hand mastery) so PR5-19's balance run can test `dulled=lock` (a small kit row); (d) a settle result key for wear for the UI; (e) repair's cost model (section 7): a cost per missing durability point or band, the weapon's tier to fine salvage and theme to faction salvage (reuse `salvage`'s `tiers` / `themes`), basic weapons Scrap-only; a schema field for the scaled cost; then a row wiring `repair` / `reforge` and target `weapon` (FB-211, FB-216).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
