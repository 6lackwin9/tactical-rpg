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
| PR5-13b `PK2-S5b` | done (FB-217; FB-218 questions): durability spend, dulled at 0, sim keys `durability=` and `dulled=`; 0 golden diffs | `develop` | -
| PR5-19 `PK2-S6` | done (FB-219 data; FB-220 balance run; FB-221 questions): PK2 framework and test data (track, body slots, grafts, recipes, test techniques, family durability, `durability.from_act: 2`); repair / reforge rows wait (pending in the kit); 0 golden diffs | `develop` | next: PR5-29a

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
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

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
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-218: PR5-13b readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` section 7 (@ the `PR5-13b` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Readings: a technique costs 2 whatever its kind (support included), the keystone counts as a technique; "support" is any kind other than attack; spend keeps counting past 0 in battle, the settle clamps; lock greys at planning, a technique planned before the weapon dulled earlier that round still resolves; a hero with authored `tuning.power` ignores weapon power, so dulling does not touch it; no settle result key or save key for wear; `masterwork_main` is not act-gated (a masterwork exists only on an instance, made from the act line on). **Number placement**: `DULLED_POWER_MULT` lives in `rpgkit_data` (re-exported as `gear::DULLED_POWER_MULT`), and the spend costs are `rpgkit_battle::wear` constants, so `rpgkit_rules`' `LOCKED` table (printed in `rules_contract_golden`) does not list them and the row's "0 golden diffs" holds; the `dulled` switch is a sim key, not a ruleset number, for the same reason (same pattern as PK2-S4's `MASTERY_LEVEL_XP`). `dulled=lock` bites nothing in sims today: catalog sim allies have no mastery, so no techniques. Repair / reforge stay pending: the effects are near-trivial now (a uid pick, `weapons::full`, `masterwork = true` behind `requires`), the gap is the cost model.
- **Recommendation**: Questions, built default first: (a) support techniques cost 2 (alternative 0); (b) move the spend costs and the 0.5 factor to data or `LOCKED` before PK2-S6 balance (a named `rules_contract_golden` re-record and HISTORY row) or keep them as code constants; (c) a sim key `mastery=N` (allies' main-hand mastery) so PR5-19's balance run can test `dulled=lock` (a small kit row); (d) a settle result key for wear for the UI; (e) repair's cost model (section 7): a cost per missing durability point or band, the weapon's tier to fine salvage and theme to faction salvage (reuse `salvage`'s `tiers` / `themes`), basic weapons Scrap-only; a schema field for the scaled cost; then a row wiring `repair` / `reforge` and target `weapon` (FB-211, FB-216).

### FB-219: PR5-19 `PK2-S6` landed (PK2 framework and test data)

- **Spec**: READY Section 1 row PR5-19; `docs/sessions/PK2_GRAFTS.md` sections 2-7 (the `PR5-19` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Seed `games/emberwake/data/seeds/pk2s6.json` (72 steps, idempotent; the review re-applied it to the pre-row data and got HEAD byte for byte) and `aegis content manifest` for `game.json` `durability.from_act: 2`. Rows: unit track `calcification` (`unit_tracks.json`: mark `calcify`, cause `calcified`, stages I-V with CANON names, V departs `lost`; passives `calcify_1` composure -5 and +5% resist on the 9 resistable debuffs, `calcify_4` `forbid_hands` 2 and `pierce_melee`); `body_slots.json` (`cranial` class, `forearms` / `thoracic` / `visceral` monster); family `graft`; GRAFT items `graft_optic` (T1), `graft_valve` (T2), `graft_core` (T3), `graft_litany` (priest class piece), `graft_purge` (priest Inoculated keystone, T3), Strain floors [5,15,30,50]; test slot materials, `rock_salt`, `black_iron_ingot`, salvage `salvage_fine` / `salvage_inquisitor`, `theme: inquisitor` on `prelates_blade` and `quells_bone_saws`; skills `marrow_rend`, `marrow_litany`, `purge_marrow` (SUPPORT `unmark calcify`, once per battle) and six test techniques `tq_*` (one per class family: polearm, crossbow, fist, dagger, censer, sprayer; family `techniques`); main-hand weapon families `durability` 40 (orb, tome, shield none); the `timeline` ruleset lists the rule `durability`; NPCs `sabina_kress` (Chirurgeon Sabina, Slab) and `berek_dorn` (Berek "Cinder-Lung" Dorn, Hearth) in `sunk_ward` (CANON's locked hub cast; the spec's "Aline" / "Bram" are stale; no placement, dialogue or portrait yet); 22 recipes (11 surgeries, 4 excisions, Rock Salt flush, bone-scraping, 3 trainings, drill, salvage), every one behind `pk2.slab_open` / `pk2.hearth_open`, Tier III behind `pk2.apex_open`, so Act 1 offers none. Not applied: `seeds/pk2s6_after.json` (`hearth_repair`, `hearth_reforge`; the kit refuses them as pending). One real rule effect: with body slots shipped, a party member with no graft marks no channeled active (PK2-S2 rule, FB-209); no shipped party member has channeled marks (only the catalog test unit `inoculated_hero`, built without growth, keeps the ruleset's `channeled_actives`). Live-data tests updated (derived): `ruleset_switches.rs` (the PK2-S6 step), `ir21_intent.rs` (147 skills, 29 dual; each new skill's intent checked), `passive_grants.rs` (the raw reader learns `forbid_hands` / `pierce_melee`), `unit_builder.rs` (the oracle mirrors `tracks` and `graft_count`). Gate 3049 passed, 0 failed, 0 warnings; 0 golden diffs. Review: change-reviewer PASS (fixed: the Apex gate, a description typo, CANON-tone test material names, a stale `wear.rs` comment).
- **Recommendation**: HISTORY / decision: "PR5-19 (PK2-S6): with body slots shipped, a member with no graft marks no channeled active (PK2-S2 rule, FB-209); catalog units built without growth keep the ruleset's `channeled_actives`." STRUCTURE: catalog `unit_tracks.json`, `body_slots.json`, `recipes.json`; seed `pk2s6.json`. GAME_BRIEF: the Slab and Hearth recipe list, family durability 40, `durability.from_act: 2`. **Spec**: PK2_GRAFTS should say Sabina / Berek (CANON) for Aline / Bram.

### FB-220: PR5-19 balance run (durability, dulled, Cold Steel vs marrow)

- **Spec**: READY row PR5-19 ("a balance run ... reported in FEEDBACK"); `docs/sessions/PK2_GRAFTS.md` section 7
- **Kind**: sim
- **Status**: open
- **Finding**: `aegis sim party <L> 200 ruleset=timeline [durability=N] [dulled=lock]` (L 1, 3, 5, 7; 200 seeds), `sim duel 3 10`, `sim feel 2 seeds=200 enc=pk1_channel_duel counts=on [rules=-channel]`. Clear % L1 line / wave / elite / boss: base 46 / 45 / 31 / 8; `durability=` 40, 25, 20, 15 identical to base; 10 nearly (boss 6); 5: 42 / 43 / 27 / 6; 1: 35 / 33 / 19 / 4; 0 (dulled from the start): 18 / 16 / 8 / 1. L5 line / wave / elite: base 29 / 20 / 32, 5: 23 / 17 / 25, 0: 16 / 11 / 18. Duels move within noise. `dulled=lock` equals `power` everywhere (catalog sim allies have no techniques). Channel duel: 32.5% won (median 6 rounds), 2.97 channeled actions, 1.0 Vent and 0.92 crises per fight; `rules=-channel` 0% won (Cold Steel alone cannot win it). Findings: (1) 40 durability is never reached in one fight (the spend per weapon per fight is about 10-15), so a weapon dulls in about 3-4 fights, which only campaign carry-over shows and the sim cannot run; (2) dulled x0.5 more than halves clear rates when it bites early (L1); from L3 the base set is already failing, independent of PK2; (3) lock cannot be balance-tested with sim allies; (4) `FightRes::worn` / `dulled` are recorded, not printed; `feel` does not apply the durability seed.
- **Recommendation**: Design levers: decide the intended pace (3-4 fights to dulled at family 40) before touching the spend costs or family values; consider `DULLED_POWER_MULT` 0.7-0.75 if dulled should be a nuisance, not a lockout; Channel duel still below 40-60 (levers from FB-191 / FB-200: `channel_power` 1.5 gave 45%, 1.75 gave 60%; Vent numbers). Small kit row candidates: print `worn` / `dulled` in sim output, apply `durability=` in `feel`, a sim `mastery=N` so lock can bite (FB-218 (c)).

### FB-221: PR5-19 readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` sections 2-7 (@ the `PR5-19` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Calcification stages II, III and V have no passive: the grammar cannot express II (+2 armour rating, -1 tile per move block), III (fear / morale immunity, no such statuses; -15% ranged damage beyond 3 tiles); V is the `lost` departure only. The recipe `requires` grammar has no act condition, so Slab / Hearth rows wait on `pk2.slab_open` / `pk2.hearth_open` / `pk2.apex_open` / `pk2.rite_<class>` / `pk2.bone_scraping` / `pk2.minor_quest` flags that nothing sets yet. Provisional numbers: surgery gold T1 40, T2 80, T3 150, class piece 40; excision 30; flush 20 + 1 Rock Salt; bone-scraping 60 + 2 bone shards; training 40 / 80 + 1 black iron / 120 + 1 black iron; drill 15 (xp 25); salvage 5 (UNCOMMON+ gives fine, `inquisitor` gives Inquisitor Steel); doctrine axes `flesh_broker` / `preservationist` +1; skill numbers (marrow_rend x2.0 / 35 Strain, marrow_litany x1.3 / 20, techniques x1.6 / 2 AP). Names: the graft and recipe texts reuse the existing core names (Stalker Optic Nerve, Inquisitor Marrow Valve, Titan Marrow Core) that CANON retires (PR5-35 renames the items; the graft texts should follow); "Black Iron Ingot" is not in CANON's material list. Techniques are 1 per family (spec: 3).
- **Recommendation**: Questions, built default first: (a) the stage II / III effects: kit grants (a kit row: armour rating, move-block tiles, a status immunity, ranged falloff) or dropped; (b) the flags' setters (quests / story data, PK8) or a recipe act condition (a kit row); (c) the provisional numbers above; (d) the NPCs' hub placement, dialogue and portraits (an Art/UI and story row); (e) PR5-35 to also rename the graft and recipe texts; (f) `train_minor` needs the pane's family picker (FB-212 (d)) once its flag opens.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
