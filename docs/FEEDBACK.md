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
| PR5-33 `LAYER-UP-DATA` | done (FB-230; FB-231 questions): GUI look tables and the sim config from data (seed `layerup3`, `games/emberwake/sim.json`); rank 2 / 3 skills show their look; 0 golden diffs | `develop` | next: PR5-34

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
- **Finding**: The row asks for generic names and byte-identical rendering; the specimen page's labels ("Veins / Eye glow / Ash / Smoke", its paragraph and the vignette sample's "drifting ash") are pinned by the theme pixel-hash fixture, so they stay. `particles_rise` names specks that fall (portrait and vignette), while `drift` rises. The old words are not added to the G3 `lore_words.txt` (the alias tables must name them; `ash` / `smoke` are also hazard ids in kit tests). The expression mark `vein` (`MARKS`, `Mark::Vein`, `expression_fx` data) is a separate vocabulary, untouched.
- **Recommendation**: Questions, built default first: (a) keep the old specimen labels (alternative: new labels with a named theme-hash re-record); (b) keep `particles_rise` (alternative: `particles_fall`, cheap now via the alias table); (c) a later row for the expression mark `vein`, or leave it.

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
- **Finding**: Kit `rpgkit_battle/src/event.rs`: `BattleEvent {line, kind}`, `EventKind` (Hit {crit, zero}, Miss, Dodged, Status {status, landed}, LostBlock, Guard {broken}, Focus, Support, Move, Break {unit}), `Resolver::events` (pushed only with a log line, in line order; `Command::cue`, `Resolver::note_event` for a start's dash or chase, the poise break), `event::since`; never serialized, so the log text, traces and sim output are byte-identical. `refusal.rs`: `Reason {HpCost, MeterGate, Text}` (Display is the old text; `refusal::text`, JSON), `command::hp_cost_reason`, `meter::gate_reason`, `Plan::add_step_why`, `SimState::error_reason`, `PickOption::reasons`; host `probe` refusal reply gains `reasons`. `round_intent_plan::area_mark` / `area_marks` / `area_preview` (one rule for the planned-area overlay and the pick preview; the preview now uses the overlay's filter, so a non-attack area skill does not preview; none ships). GUI: `battle/flow.rs::sound_of(&EventKind)`, `fx::Mods::of(&[EventKind], hp_fell)`, `fx::broke`, `intent_text` refusal wording from `Reason`s; the Channel words read the `channeled_mark` term (`tips::channeled_word`; the shipped term keeps every string). Three named sites (`[Dodged]` / `[Resisted]`, `(Marrow)` trim, the old overlay lines) were already gone; their live equivalents were done. Tests: `rpgkit_battle/tests/battle_events.rs` (5), `emberwake_gui` `battle/tests/cue_tests.rs` (2: every catalog encounter under its own ruleset and timeline, seeds 1-2, step by step, typed cues equal the old prose reading kept in test code; renamed units and a custom term keep the typed cues while prose misfires; 1,091 refusals worded identically). Gate 3086 passed, 0 failed, 0 warnings; 0 golden diffs; the inbox recordings replay as before. Review: change-reviewer PASS (fixed: a clippy lint, a neutral doc example).
- **Recommendation**: STRUCTURE (`rpgkit_battle`): `event.rs`, `refusal.rs`, `round_intent_plan::area_mark`; test `tests/battle_events.rs`; `emberwake_gui` battle test `cue_tests.rs`. Host protocol (`engine/README` or the host doc): the `probe` refusal reply has `reasons` (`refusal::Reason::to_json`). GAME_BRIEF where to look: "battle sounds, impact mods and breaks: `rpgkit_battle::event` -> `battle/flow.rs::sound_of`, `fx::Mods::of`, `fx::broke`; refusal words: `intent_text` over `refusal::Reason`". HISTORY: "2026-11-11 PR5-32: GUI reading of resolver log prose (CRIT, [Break], guard was broken, (inflicts ...), move tags) and planner refusal strings replaced by typed `event::EventKind` and `refusal::Reason`; hardcoded Marrow words replaced by `channeled_mark` term reads; 0 golden diffs." Follow-up candidates (prose still read): `rpgkit_battle/src/portrait.rs` `session_step_events` reads `[Attack]` / `[Break] <name>` (portrait cues would misfire with custom unit names; `EventKind::Break {unit}` exists, `Hit` would need its crit targets), `battle/draw.rs` log colours by tag and "Failed" / "Out of Range", the kit's `log.rs` entries, the auto-fill's reason text. A custom `channeled_mark` row without `word` falls back to the short form in sentences (terms rule TM5).

### FB-230: PR5-33 `LAYER-UP-DATA` landed (look tables and sim config from data)

- **Spec**: READY Section 1 row PR5-33; audit `2026-11-10_0de2c60` (item 3) (the `PR5-33` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: The GUI's id tables became data fields that pick keys from the GUI look library (`fx.rs` keeps the looks by neutral names): skills `fx` / `clip`; item families `fx`, `fx_far`, `attack_clips`, `area_clips`; terrain `relief`, `relief_depth`, `prop`, `color`, `face` (all schema `omit_default`; `rpgkit_data/src/looks.rs`: `FamilyLook`, `TerrainLook`, `Relief`, `Face`, `parse_color`; `TerrainTable::look`; `rpgkit_world/src/data.rs` passes the family rows to the battle catalog; `fx` / `clip` added to the skill check's `COMMON_FIELDS` so the writer keeps them); status `fx` in the kit's embedded `rpgkit_rules/src/statuses.json`. Seed `games/emberwake/data/seeds/layerup3.json` (51 upserts: 17 families, 21 skills, 13 terrain; applied with `aegis content apply`; only new keys, idempotent). Sim: `rpgkit_sim/src/config.rs` reads the new `games/emberwake/sim.json` (default world, story run, feel encounters, classes, roles, loadouts, generated rosters, role tests; embedded by `emberwake_sim`, now a thin façade). No skill / weapon / status / terrain / world id tables remain in `fx.rs`, `battle/tokens.rs`, `stage.rs` or `emberwake_sim` (single defaults left: `"unarmed"`, the `"grass"` ground fallback, the kit status `"stagger"`). A temporary 1.37M-case equivalence check (every rank-1 skill x family x kind) matched the old tables; permanent tests: `rpgkit_data/tests/looks.rs`, `rpgkit_content/tests/look_fields.rs`, `rpgkit_world/tests/family_looks.rs`, `rpgkit_sim/tests/config.rs`, `emberwake_sim/tests/config.rs` (every id in `sim.json` exists), shipped-catalog look-key and old-mapping pins and a terrain fixture tie in `emberwake_gui`. **Visible change**: rank 2 / 3 skill rows (`<id>_2`, `<id>_3`) now show their skill's look and clip (the old tables matched rank-1 ids only, so they fell back to the weapon look). Gate 3101+ passed, 0 failed, 0 warnings; 0 golden diffs. Review: change-reviewer PASS (added: the shipped look-key tests and the terrain fixture tie).
- **Recommendation**: STRUCTURE: a game-root row `games/emberwake/sim.json` (the sim description, embedded by `emberwake_sim`); `rpgkit_data/src/looks.rs`, `rpgkit_sim/src/config.rs`; line ~47 "the façade's `FEEL_ENCOUNTERS`" becomes "`sim.json` `feel_encounters`". GAME_BRIEF: lines naming removed symbols (~53 `fx::skill_style`, `support_style`; ~61 `SKILL_CLIPS`; ~77 `emberwake_sim::FEEL_ENCOUNTERS`; ~259 `emberwake_sim::CLASSES`) become "looks are data (skills `fx` / `clip`, families `fx` / `fx_far` / `attack_clips` / `area_clips`, terrain `relief` / `relief_depth` / `prop` / `color` / `face`, status `fx`) picking from the GUI look library (`fx.rs` `strike_look`, `support_look`, `blow_look`, `shot_look`, `STATUS_LOOKS`); the sim description is `games/emberwake/sim.json`". HISTORY: "2026-11-11 PR5-33: retired the GUI id tables (`skill_style`, `support_style`, `melee_style`, `shot_style`, `ARCANE_FAMILIES`, `SKILL_CLIPS`, `FIST_WEAPONS`, the terrain stage / colour / face matches, `STATUS_LOOKS` ids) and the `emberwake_sim` constants (`WORLD`, `STEEL_FLAG`, `FEEL_ENCOUNTERS`, `CLASSES`) for data (layer purity, audit item 3); rank 2 / 3 skill rows now show their skill's look; no golden re-recorded."

### FB-231: PR5-33 questions

- **Spec**: READY row PR5-33 (@ the `PR5-33` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: `statuses.json` is the kit's embedded table (games do not override it), so the status look keys (`star_orbit`, `rising_embers`, ...) of this game's GUI now sit in the kit (the row names `statuses.json`; the layers check passes). Look data is a key per row; colours and speeds stay in GUI code (the lowest-undo way to keep rendering byte-identical). `sim.json` sits at the game root (neither lane writes a plain non-catalog file under `data/`). A terrain `color` is not format-checked (a bad hex draws the default grey) unless the optional check landed (see FB-230).
- **Recommendation**: Questions, built default first: (a) keep status looks in the kit table as defaults (alternative: a game-side `status_looks` map in `tuning.json` or a game data kind, a small row); (b) keep rank 2 / 3 inheriting their skill's look (alternative: rank-1 only, a small change); (c) confirm `games/emberwake/sim.json` as the home (alternatives: a catalog kind or a manifest block); (d) full look parameters as data later (a separate row).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
