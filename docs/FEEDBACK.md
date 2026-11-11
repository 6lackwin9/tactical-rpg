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
| PR5-31 `LAYER-UP-PARTY` | done (FB-228): party edit rules in `rpgkit_world::party_ops`, `roster::toggle_active`; AP text fixed; 0 golden diffs | `develop` | next: PR5-32

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
- **Finding**: Per-hit audit of `battle`, `round`, `block` in `engine/crates/*`. Renamed: the recordings browser (`aegis_ui/src/recordings.rs`) `Act::Battle` -> `Act::Session`, field `session_pick`, widget id `recordings.battle` -> `recordings.session` (the old id and its `.opt.N` children are rewritten for agent-link commands and drags, `recordings::ID_ALIASES` / `canonical_id` in `shell.rs` `bridge`; scenario `click` / `hover` / `drag` steps go through it; a scenario `widget` wait or assert and a state-path read do not), state key `battle` -> `session` and the note stamp `battle` -> `session` (game `note_context` too; nothing reads either, so no alias; old recordings keep `battle: -1`), messages and tips reworded, doc comments in `screen`, `audio`, `depth`, `painter`, `bridge`, `notes`, `aegis_core::rng`. Kept: `round` (a turn word other genres share; "Round 1" / "After round N" pixel-pinned; the kit checkpoint `state.round`), `block` (the timeline widget's slot, UI, tiles, codecs), maths uses, the PR5-29a alias tables. No public engine name holds `battle` or `combat` besides the alias tables. Tests: `aegis_ui/tests/recordings_keys.rs` (3), a shell test for new and old ids, a game note-context test. Gate 3066 passed, 0 failed, 0 warnings; 0 golden diffs (pane and theme pixel hashes unchanged); scenario check clean. Review: change-reviewer PASS.
- **Recommendation**: `engine/README.md` ~210 "the session (`session`, was `battle`), round, block and phase"; ~221 "per session"; add "the recordings browser's session picker is `recordings.session` (old id `recordings.battle` accepted for agent-link commands only, `recordings::ID_ALIASES`); its state key is `session`; a session's label names its first request's `scene`, else `encounter` (`SCENE_KEYS`); the drawn 'BATTLE' / 'battles' stay (`SESSION_LABELS`, pane hashes)". `engine/docs/PORT_NOTES.md` 328 "the session slot (`session`, was `battle`)". Reference `docs/specs/SILOS1.md` 75 (`Act::Battle` done). HISTORY: "2026-11-11 PR5-29c LAYER-ENG-API (FB-206 2c): the engine recordings browser's `Act::Battle` / `battle` / widget `recordings.battle` / state `battle` became `Session` / `session_pick` / `recordings.session` / `session` (agent-link alias for the old id); the note stamp `battle` became `session`; drawn labels kept for the pane hashes; `round` and `block` kept; no golden re-recorded."

### FB-227: PR5-29c left-overs and questions

- **Spec**: READY row PR5-29c (@ the `PR5-29c` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Left in the engine under the 0-golden-diff rule (pixel-pinned or file formats): `recordings::SESSION_LABELS` {"BATTLE", "battles"} (a public engine const that still draws the words); `SCENE_KEYS` reads the kit's `encounter` request key (nothing writes `scene`, so the engine reads a kit wire word); the timeline sample's "Party", "Enemies", "Brute", "Raider", "Heavy smash", "Strike" (`timeline.rs` ~488-537) and the specimens' "Brute", "Strike", "Party" icon; theme role `foe` (`theme.rs` ~102, `ui::FOE`; part of the theme file format); `PipBlock.ap`; the default note stamp's `round` / `block` / `phase`; the trail analysis words (`plan_edit {unit}`, `undo_churn` "for one unit in one round"). G3's lore list should not take `battle` (the kit uses it legitimately: `rpgkit_battle`); an engine-only genre-word check with an alias allowlist would be the guard.
- **Recommendation**: Questions, built default first: (a) keep `round` (alternative: `turn` / `step` with a named pane-hash re-record and a kit checkpoint alias); (b) keep the drawn "BATTLE" / "battles" (alternatives: "SESSION" / "sessions" with a named re-record, or the game supplies the labels through a setter so the engine default is generic and the hashes hold, a small row); (c) the game supplies the session's scene name (a callback) instead of the engine reading `encounter`; (d) a follow-up row for the sample labels, theme role `foe` (with a theme-file alias) and `ap`; (e) an engine-only genre-word check; (f) FB-223 (`rules=`) still open.

### FB-228: PR5-31 `LAYER-UP-PARTY` landed (party edit rules in the kit)

- **Spec**: READY Section 1 row PR5-31; audit `2026-11-10_0de2c60` (section 2.2 in the file; the row says 2.3) (the `PR5-31` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: New `rpgkit_world::party_ops/` (game façade `emberwake_world::party_ops`): `deck.rs` (`DeckTerms`: bucket caps, auto-pinning, `channeled` / order / follow-up pruning, follow-up legality, `order_auto`; `DECK_KINDS`, `intent_names`), `slots.rs` (`SkillSlots` {class, extra, ranks}: `click_group`, `slot`, `after_drop`, `unslot`, `keep_slotted_ranks`, `confirmed`, `default_of`; `can_assign_to_slot`, `slot_cap`, `draft_rank`, `rank_step`, `path_families`, `top_rank`), `gear.rs` (equip, take off, move, the two-handed off-hand rule, pouch and stack targets, `drop_move`, `pack`); `roster::toggle_active` (the active squad size, its old quirk kept). Moved verbatim (same checks, order and refusal strings; the review diffed them); `emberwake_gui/src/party.rs` is a presenter (about 900 lines fewer), the Dojo reads `DeckTerms` from the kit (its order-auto too). AP text fixed: the party AP help ("the most this unit holds; each round refills some or all of it ...; a skill costs its level (1-5) in AP, Move 1 and Rest nothing, or the AP cost its row sets under the AP cost rule") and the forge's cost line (`ap_cost`, plus each catalog ruleset that prices the skill differently); no golden pins either. Tests: `rpgkit_world/tests/party_ops.rs` (13, scratch data). Gate 3079+ passed, 0 failed, 0 warnings; 0 golden diffs. Review: change-reviewer PASS (fixed: the Dojo's order-auto through the kit, the AP help wording).
- **Recommendation**: STRUCTURE: `kits/rpg/crates/rpgkit_world/src/party_ops/` (deck, slots, gear; test `tests/party_ops.rs`), `roster.rs` `toggle_active`; `emberwake_gui` `party.rs` a presenter over `party_ops`. GAME_BRIEF where to look: "party edit rules: `rpgkit_world::party_ops`". HISTORY: none. `kits/rpg/README.md`: done. Optional follow-ups: a `DeckTerms` confirm to match `SkillSlots::confirmed` (the deck Confirm stays inline in `party.rs`); the classic Skills tab could use `ap_cost_by` for a ruleset with `ap_cost` but no deck rule (none ships).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
