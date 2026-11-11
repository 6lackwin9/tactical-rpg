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
| PR5-19 `PK2-S6` | done (FB-219 data; FB-220 balance run; FB-221 questions): PK2 framework and test data (track, body slots, grafts, recipes, test techniques, family durability, `durability.from_act: 2`); repair / reforge rows wait (pending in the kit); 0 golden diffs | `develop` | -
| PR5-29a `LAYER-ENG-KEYS` | done (FB-222 docs delta; FB-223 the `rules=` question): scenario launch keys `scene` / `rules`, playtest `sessions`, replay `session=K`, with aliases; 0 golden diffs | `develop` | -
| PR5-29b `LAYER-ENG-FX` | done (FB-224 docs delta; FB-225 questions): portrait overlay primitives lines / point_glow / particles_rise / drift with load aliases; 0 golden diffs | `develop` | -
| PR5-29c `LAYER-ENG-API` | done (FB-226 docs delta; FB-227 left-overs and questions): engine recordings browser session nouns with an agent-link alias; `round` / `block` kept; 0 golden diffs | `develop` | -
| PR5-31 `LAYER-UP-PARTY` | done (FB-228): party edit rules in `rpgkit_world::party_ops`, `roster::toggle_active`; AP text fixed; 0 golden diffs | `develop` | -
| PR5-32 `LAYER-UP-EVENTS` | done (FB-229): typed battle events and refusals, the area-preview helper, Channel words from terms; 0 golden diffs | `develop` | -
| PR5-33 `LAYER-UP-DATA` | done (FB-230; FB-231 questions): GUI look tables and the sim config from data (seed `layerup3`, `games/emberwake/sim.json`); rank 2 / 3 skills show their look; 0 golden diffs | `develop` | -
| PR5-34 `SUNKWARD-FIX1` | partial (FB-232; FB-233 questions): sunk_ward founder and boss rows, meters, start_act 1, talon, captive, Maren's censer attack (seed `sunkfix1`); the M4 retreat phase and loot moves blocked (no M4 encounter yet); 0 golden diffs | `develop` | -
| PR5-35 `LORE1-DATA` | done (FB-234): CANON lore names (Talia, seven items), keepsake placeholder icons, explicit reach bands (seed `lore1`); 0 golden diffs | `develop` | queue empty: waiting for Design (FB-207 to FB-234 open)

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
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-211: PR5-10 `PK2-S3` landed (recipes, the NPC `recipe` service and pane)

- **Spec**: READY Section 1 row PR5-10; `docs/sessions/PK2_GRAFTS.md` section 4 (the `PR5-10` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-212: PR5-10 readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` section 4 (@ the `PR5-10` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-213: PR5-12 `PK2-S4` landed (weapon mastery and techniques; `checks_golden` re-recorded)

- **Spec**: READY Section 1 row PR5-12; `docs/sessions/PK2_GRAFTS.md` section 5 (the `PR5-12` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-214: PR5-12 readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` section 5 (@ the `PR5-12` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-215: PR5-13a `PK2-S5a` landed (weapon instances; `rules_contract_golden` re-recorded)

- **Spec**: READY Section 1 row PR5-13a; `docs/sessions/PK2_GRAFTS.md` section 6 (the `PR5-13a` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-216: PR5-13a readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` section 6 (@ the `PR5-13a` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-217: PR5-13b `PK2-S5b` landed (durability spend, dulled, sim keys)

- **Spec**: READY Section 1 row PR5-13b; `docs/sessions/PK2_GRAFTS.md` section 7 (the `PR5-13b` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-218: PR5-13b readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` section 7 (@ the `PR5-13b` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-219: PR5-19 `PK2-S6` landed (PK2 framework and test data)

- **Spec**: READY Section 1 row PR5-19; `docs/sessions/PK2_GRAFTS.md` sections 2-7 (the `PR5-19` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-220: PR5-19 balance run (durability, dulled, Cold Steel vs marrow)

- **Spec**: READY row PR5-19 ("a balance run ... reported in FEEDBACK"); `docs/sessions/PK2_GRAFTS.md` section 7
- **Kind**: sim
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-221: PR5-19 readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` sections 2-7 (@ the `PR5-19` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-222: PR5-29a `LAYER-ENG-KEYS` landed (scenario and playtest keys; docs delta)

- **Spec**: READY Section 1 row PR5-29a; FB-206 (2a) (the `PR5-29a` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-223: PR5-29a question: `rules=` means two things on one binary

- **Spec**: READY row PR5-29a ("`ruleset` -> `rules`"); `engine/README.md` (`aegis battle ... ruleset=<id> rules=<-id,+id>`)
- **Kind**: conflict
- **Status**: open
- **Finding**: Everywhere else `rules` is the rule-adjustment spec and `ruleset` the ruleset id (`aegis battle` / `sim`, host `new {ruleset, rules}`, the GUI's own playback). As the row asked, the launch key `rules=` (scenario files, `scenario shots`, the game's launch args) now means the ruleset id, so `emberwake screen=arena rules=+durability` reads `+durability` as a ruleset id; `shots ... ruleset=timeline rules=+x` fails loudly ("both names given"). No behaviour regressed (the launch had no adjustment key before).
- **Recommendation**: Built as the row says. Options, lowest undo first: (a) keep `ruleset` as the engine launch key (a ruleset is not an RPG word; board and card games have them) and rename only `encounter` -> `scene` (flip the alias table, a small row); (b) keep `rules`; (c) another name (e.g. `rules_id`). Decide before PR5-29b / 29c add more aliases.

### FB-224: PR5-29b `LAYER-ENG-FX` landed (portrait overlay primitives; docs delta)

- **Spec**: READY Section 1 row PR5-29b; FB-206 (2b) (the `PR5-29b` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-225: PR5-29b questions

- **Spec**: READY row PR5-29b (@ the `PR5-29b` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-226: PR5-29c `LAYER-ENG-API` landed (engine recordings browser nouns; docs delta)

- **Spec**: READY Section 1 row PR5-29c; FB-206 (2c), FB-222 left-overs (the `PR5-29c` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-227: PR5-29c left-overs and questions

- **Spec**: READY row PR5-29c (@ the `PR5-29c` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-228: PR5-31 `LAYER-UP-PARTY` landed (party edit rules in the kit)

- **Spec**: READY Section 1 row PR5-31; audit `2026-11-10_0de2c60` (section 2.2 in the file; the row says 2.3) (the `PR5-31` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-229: PR5-32 `LAYER-UP-EVENTS` landed (typed battle events and refusals)

- **Spec**: READY Section 1 row PR5-32; audit `2026-11-10_0de2c60` (item 2) (the `PR5-32` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-230: PR5-33 `LAYER-UP-DATA` landed (look tables and sim config from data)

- **Spec**: READY Section 1 row PR5-33; audit `2026-11-10_0de2c60` (item 3) (the `PR5-33` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-231: PR5-33 questions

- **Spec**: READY row PR5-33 (@ the `PR5-33` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: `statuses.json` is the kit's embedded table (games do not override it), so the status look keys (`star_orbit`, `rising_embers`, ...) of this game's GUI now sit in the kit (the row names `statuses.json`; the layers check passes). Look data is a key per row; colours and speeds stay in GUI code (the lowest-undo way to keep rendering byte-identical). `sim.json` sits at the game root (neither lane writes a plain non-catalog file under `data/`). A terrain `color` is not format-checked (a bad hex draws the default grey) unless the optional check landed (see FB-230).
- **Recommendation**: Questions, built default first: (a) keep status looks in the kit table as defaults (alternative: a game-side `status_looks` map in `tuning.json` or a game data kind, a small row); (b) keep rank 2 / 3 inheriting their skill's look (alternative: rank-1 only, a small change); (c) confirm `games/emberwake/sim.json` as the home (alternatives: a catalog kind or a manifest block); (d) full look parameters as data later (a separate row).

### FB-232: PR5-34 `SUNKWARD-FIX1` landed (partial: M4 blocked)

- **Spec**: READY Section 1 row PR5-34; audit `2026-11-10_0de2c60` 3.1-3.3, user A5, A6 (the `PR5-34` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Seed `games/emberwake/data/seeds/sunkfix1.json` (19 steps, idempotent; the review re-applied it to the pre-row data and got HEAD exactly). The audit's "typos" were not in the data: the three founders' `temperament` held other characters' traits (kaelen Vane's `traitors_discipline`, maren Quell's `marrow_valve`, elyse Orlov's `vat_masters_grease`); now `castellans_penance`, `copyist_of_st_lyras` (grants `bile_phial`, so `bile_coat` works), `volatile_reaction_catalyst`. Added `sunk_ward` characters: party `vesper` (`cinder_childs_reflex`), `hollis` (`crag_hawkeye`), `dunstan` ("Slag-Jaw", `foundry_wired_spite`) per CAST1_FOUNDERS; enemies `overseer_grendel`, `guild_master_orlov` (`vat_masters_grease`, `tuning.hp_pct` 20 from CAST1_BOSSES: +20% max HP), `calcified_amalgam` (`amalgam_carapace`, `hp_pct` 60), `inquisitor_prelate_nemeth` (`marrow_infused`), `bailiff_enforcer`, `calcified_husk`. Catalog: item `orlovs_flensing_talon` (dagger, RARE, special; power +3, +0.2 potency on `throat_carver` / `orlovs_retribution`; the execute effect deferred, G7), unit `tithe_captive` (party, rogue, minion, `flee_ratio` 1.0), skill `thurible_smite` (Level 1 censer ATTACK, standard rounds; provisional 2 AP, 2 blocks, x1.3, stagger 6) on `founder_maren` (`extra_skills`, `deck.active` 3 of the timeline's 3, `order.assault`). `sunk_ward/meters.json`: `alert` visible, `doom_clock` 42 / 42 visible. `world.json`: `campaign.start_act` 1; the Vanguard blurb (2H polearms). **What `start_act: 1` turns on**: no world has a `roster` block, so `roster::rule` stays classic and **no wounded rules start** (the row's stated purpose); it does turn on the act-1 meter rules in `sunk_ward/meters.json`: `a_last` (a warning at 90 Alert), `a_breach_1` (**the run fails at Breach, 100 Alert**) and `d_zero` (at 0 days: flag `lesser_tithings`, the clock restarts at 14). **Blocked**: no M4 encounter exists in any data file (`enc_evt_04`, `loot_flensing_talon`, `loot_orlovs_retribution`), so the `orlov_withdraws` phase (hp_below 20 -> flee_ratio 1.0) and the loot moves wait for the M4 encounter rows (the phase fields exist in the encounter check). Lane note: the data row also updated two live-data tests (`ruleset_switches.rs`: a sunkfix1 units step; `ir21_intent.rs`: 148 skills, Assault +1). Gate 3105 passed, 0 failed, 0 warnings; 0 golden diffs (the journey, sim and rules goldens read frozen fixtures, so they do not play this data). Review: change-reviewer PASS (fixed: the censer text, the talon's lore name, the captive an adult debtor).
- **Recommendation**: READY: PR5-34 part (4)'s phase and loot moves need a row after the M4 encounter authoring; a `roster.rules` block (acts 1-3, wounded) for the wounded-from-mission-1 intent. ART_UI_REQUESTS: portraits for `calcified_amalgam` and `bailiff_enforcer` (Grendel is AI-06). HISTORY: none (no golden re-recorded).

### FB-233: PR5-34 questions

- **Spec**: READY row PR5-34 (@ the `PR5-34` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Provisional values not in any spec: `thurible_smite` x1.3 at 2 AP / 2 blocks (as many blocks as `strike` but stronger, on the support founder; compare Slag Slam x1.2 at 3 AP, Barbed Quarrel x0.95 at 2 AP); founders' `default_intent` (vesper maneuver, hollis and dunstan assault); `tithe_captive` `ai_role` SKIRMISHER, `defend_below` 0. Maren now fills all three timeline actives (no free slot).
- **Recommendation**: Questions, built default first: (a) the censer attack's numbers (alternative: x1.1 or 3 AP), a balance pass; (b) is the act-1 Breach run-fail and the lesser tithing from mission 1 intended (built: yes, as the meter rows were authored for act 1); (c) the founders' default intents.

### FB-234: PR5-35 `LORE1-DATA` landed (lore names; keepsake placeholders)

- **Spec**: READY Section 1 row PR5-35; CANON 3, 5.2, 5.3; FB-203 (a) (the `PR5-35` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Seed `games/emberwake/data/seeds/lore1.json` (19 steps, idempotent; the review re-applied it to the pre-row data and got HEAD exactly). `sunk_ward` `alert` warn text: Tess -> Talia. Items renamed with CANON names and descriptions that promise only what the rows do: `titan_marrow_core` Dreamer's Marrow Geode, `caustic_gall_sac` Vitriol Bezoar, `stalker_optic_nerve` Amber Tapetum, `inquisitor_marrow_valve` Brass Marrow Valve, `sulfur_poultice` Tallow-Wax Stitch Pack, `brimstone_draught` Brimstone Tallow Salve, `pitch_fire_bomb` Pitch Censer-Pot (`con_*` ashlands items untouched); the `graft_optic` / `graft_valve` / `graft_core` descriptions name the new materials. The six keepsakes already existed and resolve; each gets `icon_id` `keepsake_<id>` with **placeholder** icons `games/emberwake/assets/items/accessory/keepsake_{kaelen,vesper,maren,hollis,elyse,dunstan}.png` (copies of `kits/rpg/assets/items/accessory/amulet.png`, decision "Placeholder-first assets"). `range_band {min 1, max 2}` on `polearm` and `scourge` (the kit reads only `max`, equal to the default `REACH_WEAPON_REACH` 2; no play change). Gate passed (3105, 0 failed, 0 warnings); 0 golden diffs (none to name). Review: change-reviewer PASS.
- **Recommendation**: ART_UI_REQUESTS Inbox: the six keepsake icons are placeholders (CANON 5.2 names each keepsake). A small data row for retired names still shipped: the graft names "Optic Nerve Graft" / "Marrow Valve Graft" / "Marrow Core Graft" (`items.json`) and nine surgery recipes "Optic Nerve / Marrow Valve / Marrow Core into ..." (`recipes.json`); and a logic-lane test string `emberwake_gui/src/runover.rs` ~918 "bribes at Tess". Questions: (a) keepsake display names short ("Gouged Sigil") vs CANON's possessive ("Kaelen's Gouged Sigil") — keep or rename (one seed); (b) CANON gives the Pitch Censer-Pot a 3x3 area (data: PLUS) and the Stitch Pack a bleed / poison seal (no such effect in data) — rules or CANON to change. HISTORY: none.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.

### UN-12: CI no longer runs the full gate on develop pushes (user 2026-11-11)

- **For**: design
- **Status**: open
- **Note**: GitHub Actions stopped starting jobs at 2026-11-10 23:52 UTC ("recent account payments have failed or your spending limit needs to be increased"): the private repo used its free-plan 2,000 Actions minutes, about 1,800 of them on `gate.yml` (157 runs in November, a full 12-18 min gate on every `develop` push). The user is making the repo public and asked that `develop` pushes no longer run the full gate (pushes to `main` stay). Done in `.github/workflows/gate.yml`: `on.push.branches` is `[main]` (was `[main, develop]`), plus `workflow_dispatch` for a run on request; PRs into `main` still run it, so the release PR stays the CI check (`gate` required on `main`). Every landing already runs `tools/fc.sh gate` locally first. Release PR #62 (PR5-7..PR5-35) waits on CI; once Actions runs again it merges and the web build deploys.
- **Ask**: record the decision (e.g. "CI on release PRs and main only") in `docs/decisions/workflow.md` and adjust `docs/workflow/cloud.md` (the release section's note that each `develop` push restarted CI) and `docs/workflow/desktop.md` if they describe CI on `develop`.
