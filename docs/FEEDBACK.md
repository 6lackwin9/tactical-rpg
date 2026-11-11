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
| PR5-29 `LAYER-DOWN-1` | parts (3) tools and (1) kit names done (FB-205; `rules_contract_golden` re-recorded); part (2) engine waits (FB-206) | `develop` | next: PR5-30

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-196 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-10). New items start at FB-197. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-196: PR5-24 `STORYMUSIC1` landed (story steps and vignettes ask for music; the game's `music_keys` in `game.json`)

- **Spec**: READY Section 1 row PR5-24; ART_UI_STUDIO 3.9 request 3 (the `PR5-24` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: A vignette's `music` and a new optional `music` on every sequence step kind read the same way (`rpgkit_data` `MusicAsk`, `music_ask`): `""` keeps the music, `none` is silence, a path ending `.ogg` plays as written, any other value is a music key. The content check (`check/story.rs` `music_error`) accepts a key the game lists in `game.json` **`music_keys`** (new optional manifest key, read like the purse caps: `source::MUSIC_KEYS`, in `MANIFEST_KEYS`, `music_keys_problems` refuses a non-list, a non-id or a key twice; `View::music_keys`) or a key whose `audio/music/<key>.ogg` the asset layers hold; an unknown or non-id key is one error, a missing `.ogg` path a warning. Emberwake's `game.json` lists its 9 MUSIC1 contexts and the story keys `story_opening`, `prologue_tension`, `story_escape`, `story_goodbye` (set with `tools/fc.sh content manifest`); the kit names none of them (review M1: an earlier draft held both lists as kit constants). The game resolves a key through its music chain (`music::game_track`: region-scoped file first, then `<key>.ogg`; MUSICVAR1 variants apply); with no track the music goes on. On the story stage a step's music becomes the stage's music when the stage reaches it, before its vignettes (`StoryStage::take_step_music`); an `effects` step's music carries to the next step the stage plays (`rpgkit_world::story::step_music`: the nearest one wins); a vignette's own music still wins while it plays; a `retry` fight does not re-apply it. Tests: `rpgkit_world/tests/story_music.rs` (2), `rpgkit_content/tests/story_checks.rs` (+2: the manifest key, keys or tracks), `emberwake_gui` story test (+1, with `game.json`'s list holding every `music.rs` context). Gate 2931 passed, 0 failed, 4 ignored, 0 warnings; 0 golden diffs. Review: change-reviewer PASS (M1 layer: fixed by the manifest key; L1 a tautological test expectation: fixed).
- **Recommendation**: confirm the new `game.json` key. GAME_BRIEF (manifest keys / story music): "`game.json` `music_keys` (optional, STORYMUSIC1): the music keys a vignette's or a sequence step's `music` may name without a track: Emberwake's 9 MUSIC1 contexts and `story_opening`, `prologue_tension`, `story_escape`, `story_goodbye`; set with `aegis content manifest`; without it only a shipped `audio/music/<key>.ogg` counts. A step's music holds from that step on on the story stage (an effects step's from the next step played); a vignette's own music wins while it plays." STRUCTURE: test `kits/rpg/crates/rpgkit_world/tests/story_music.rs`. Art/UI (music README, ART_UI_STUDIO 3.9): story tracks go in `games/emberwake/assets/audio/music/story_opening.ogg` (and the other three), or a region-scoped `<region>_story_opening.ogg` (AI-03 names `assets/music/`, but the player reads `assets/audio/music/`); data wiring sets `music` on the vignette row or the step. `kits/rpg/README.md`: done. Notes (built defaults): loading a save mid-sequence starts from the menu theme plus the current step's own or carried music (music from an earlier visible step is not restored); `music` on `enter` and `end` steps has no audible effect, and on a `fight` step it is the music after the fight; the story keys have no parent context (no track: the music goes on).

### FB-197: PR5-26 `AUDIT-PR5-BUGS` landed (audit 1A fixes in five groups)

- **Spec**: READY Section 1 row PR5-26; `docs/audits/2026-11-10_0de2c60_consistency_audit.md` 1A; `FB-191` (b), (e), (f); `FB-193` (a) (the `PR5-26` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: full text in `docs/archive/FEEDBACK_CLOSED.md` (Open items, full text, 2026-11-11); the Recommendation below stands.
- **Recommendation**: `combat_hooks.md`: "round end runs the `round_end` triggers before the unit meters' decay, so a meter's crisis `at_least` its max fires first (a trigger that raises a meter now lands before decay and can count as gained); the AI Vents from 75% of the channel meter's max; its Channel row counts a marrow-only attack." GAME_BRIEF (PK2 tracks): "a member whose stage forbids two-handed items gets `none` in a slot whose two-handed default it cannot wield (the default is not stashed; reported as `cleared`); the slot stays empty until gear is equipped." Sim notes / BALANCE: the new `tune=` keys, `counts=on` counters, `seeds=` in party / sweep. Tools / cloud workflow: the `RUSTUP_TOOLCHAIN` default. `kits/rpg/README.md`: done. No HISTORY row.

### FB-198: PR5-26 clause B9 parked (per-rank `hp_cost_pct`, `uses_per_battle`) and four questions

- **Spec**: READY row PR5-26 group (4) "add `hp_cost_pct` & `uses_per_battle` to `source.rs` `RANK_KEYS`" (audit B9)
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: Not built: `kits/rpg/schema.json` marks both fields `"rank": false`, and `channel_checks.rs` (:93-95) and `skill_unmark_checks.rs` (:76-86) pin "not per rank", as PK1-S2a and PK2-S1 specify; adding them to `RANK_KEYS` alone would split the schema from the code. Today an unknown key in a rank modifier is silently dropped, which is likely what B9 saw. Questions on built defaults: (Q2) after the crisis sets Strain to 50 the decay still runs in a round without gain, so Strain ends at 40; (Q3) the AI Vent threshold (75% of the meter's max) is a kit constant; (Q4) a slot emptied as `"none"` stays empty after the track stage falls (the player equips something); (Q5) `UnitState::deck_errors` builds in the member's own world (from its `lore_ref`), as `validate` has no played world.
- **Recommendation**: B9: Option A: make both per rank (`RANK_KEYS` 26 -> 28, schema `"rank": true`, flip the two tests); Option B (recommended): keep them not per rank and have the check refuse a non-rank key in a rank modifier (no silent drop). Q2: Option A (recommended): a meter set by a `round_end` trigger skips that round's decay (Strain stays 50); Option B: keep 40. Q3: keep the constant, or a ruleset number `channel_ai_vent_pct` (a named contract re-record). Q4: keep (default), or sanitize drops the marker once the default is allowed again. Q5: keep.

### FB-199: Build process error: the PR5-24 landing reverted Design's newer docs (fixed by Design @d2a94371)

- **Spec**: Build landing procedure (cloud)
- **Kind**: suggestion
- **Status**: open
- **Finding**: The PR5-24 squash (`94cc7f86`) was built by applying `git diff origin/develop <task branch> -- engine games kits tools` onto `develop`. The task branch did not hold Design's newer commits (`c8091291`, `3fa8e999`), so that diff reverted their changes under `games/emberwake/docs/` (lore, specs) and `engine/docs/`; Design restored them in `d2a94371`. The same step would have reverted `8e0926b1` and `d2a94371` in this landing; caught before the push (the staged file list showed docs it should not touch). From this landing on, a squash applies only the task's own diff, from the branch's base sync commit to its head (`git diff <base> <head>`), and the staged file list is checked for paths outside the task. Earlier landings this session applied the same pattern while the branch held `develop` (no newer Design commits), so they reverted nothing; PR5-24 is the only one.
- **Recommendation**: known-issues gains the row (this push). Design may want to re-check `games/emberwake/docs/` and `engine/docs/` against its own last commits once (d2a94371 restored what Design found).

### FB-200: PR5-27 `PK1-S3-FIX` data landed (test hero's marrow attack, a 3-block duel, no Channel duel preset)

- **Spec**: READY Section 1 row PR5-27 parts (1)-(2) (the `PR5-27` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: full text in `docs/archive/FEEDBACK_CLOSED.md` (Open items, full text, 2026-11-11); the Recommendation below stands.
- **Recommendation**: GAME_BRIEF (PK1 test data): `inoculated_hero` marks `tl_hook_thrust` for its marrow version; `pk1_channel_duel` plays 3 blocks a round; no Dojo preset ships. No HISTORY row.

### FB-201: PR5-27 balance rerun: the AI never Vents (a PR5-26 bug), crises do fire, Vent's numbers need a call

- **Spec**: READY row PR5-27 part (3) and acceptance ("Channel rounds, crises, and Vent in 40-seed `sim feel counts=on`"); PK1 6.1
- **Kind**: sim
- **Status**: open
- **Finding**: full text in `docs/archive/FEEDBACK_CLOSED.md` (Open items, full text, 2026-11-11); the Recommendation below stands.
- **Recommendation**: (a) Vent's numbers (PK1 4.3 item 5 says -20 Strain, 15% HP): Option A (recommended): -50 Strain, `hp_cost_pct` 10 (both mechanics show; 37.5% against this elite); Option B: keep -20 / 15 and change the Vent round's shape (Vent as the opener only, the rest of the round attacks, not Feed padding; a kit row, untested); Option C: AI Vent threshold to 90% (a kit number; Vent rare). The result is knife-edge (-60 flips), so pick deliberately. (b) `channel_power` stays 1.25 until Vent is settled (1.4-1.5 is the lever for ~50% against this elite; a global rule number). (c) A Strain-free hero control (the Bite rider off) would be a fairer baseline than `rules=-channel`. (d) Flesh-Tension needs a poise attacker in the test fight to be measured.

### FB-202: PR5-26 follow-up landed: the AI Vents under `field_intent`; `counts=on` counts crises

- **Spec**: READY row PR5-26 group (5) ("an AI Vent rule at high Strain", "Strain/crisis/Vent counters to `counts=on`"); found by the PR5-27 balance run (FB-201)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: full text in `docs/archive/FEEDBACK_CLOSED.md` (Open items, full text, 2026-11-11); the Recommendation below stands.
- **Recommendation**: `combat_hooks.md` (PR5-26 line): "the AI's Vent pick takes the planner's self row (Stand under `field_intent`)". Sim notes: `counts=on` `crises`; the "channel meter triggers" label counts every channel-meter trigger row (rename or narrow it if wanted). `kits/rpg/README.md`: done.

### FB-203: PR5-28 `R14-SPEC-CLEANUP` items (2) and (3) landed (reach from a family's `range_band`, schema help)

- **Spec**: READY Section 1 row PR5-28 items (2), (3); audit 1B (the `PR5-28` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: full text in `docs/archive/FEEDBACK_CLOSED.md` (Open items, full text, 2026-11-11); the Recommendation below stands.
- **Recommendation**: `kits/rpg/README.md` (gear): "PR5-28: a family's optional `range_band {min: 1, max: 2-6}` (reach families only) sets the reach rule's melee band edge (the loader carries it as the derived item key `family_reach_max`); without it, `REACH_WEAPON_REACH = 2`." Questions: (a) give polearm and scourge `range_band {min: 1, max: 2}` by seed now (no change in play; note scourge's item band `bands.main_1h.range.max` is 1 and is not the reach); (b) `round_intent::melee_band` (attack lock, diagonal test) still treats a band past 2 as not melee: if a family ever goes above 2, should it read the unit's reach.

### FB-204: PR5-28 item (1) parked: `spell_reach` is live (`wpn_orb` +1), retiring it moves goldens

- **Spec**: READY row PR5-28 item (1) ("retire `spell_reach` ... migrating `wpn_orb` ... to `range_band`"); audit 1B, FB-25, U4 call it dead
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: Not built. `spell_reach` is live: spell skills add `gear_spell_reach.min(GEAR_REACH_CAP = 1) + passive_spell_reach` to their reach (`rpgkit_battle/src/skill.rs:302` `band()`, `Unit::spell_reach_bonus` `rpgkit_rules/src/unit.rs:1723`; shapes cast from the user's own tile get none), and `wpn_orb` (+1) is the only shipped item using it, matching its description. With `spell_reach_bonus` at 0 (a reverted experiment) `sim_golden` breaks (13 of 22 runs; its frozen catalog `timeline_kit.json` holds `wpn_orb` with `spell_reach 1`); a retirement would also move `rules_contract_golden` (line 83, `locked budget.rs::W_SPELL_REACH gear`) and `checks_golden` (lines 512 and 671: the "unknown effect type" lists name `spell_reach`; the lists live in `check/simple.rs:74, 83, 195`, not `check/hooks.rs`). `range_band` cannot stand in: it is a melee reach family's band (PR5-28 item 2) with no meaning for an off-hand focus.
- **Recommendation**: Option C (recommended): keep `spell_reach`, correct the "dead" note in the audit, FB-25 and U4, and drop item (1) from PR5-28. Option A: define a focus's reach (a new focus field, a loader that converts the old effect so the frozen catalog still reads +1): 0 sim diffs, but `checks_golden` and `rules_contract_golden` re-recorded. Option B: retire it outright: the orb loses its +1 and its description changes; `sim_golden` (13 runs), `checks_golden` (2 lines) and `rules_contract_golden` (1 line) re-recorded with HISTORY rows.

### FB-205: PR5-29 `LAYER-DOWN-1` parts (3) and (1) landed (layer check G3; kit-neutral Channel names; `rules_contract_golden` re-recorded)

- **Spec**: READY Section 1 row PR5-29 parts (1), (3); audit 2.1 (the audit has no 2.4) (the `PR5-29` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: full text in `docs/archive/FEEDBACK_CLOSED.md` (Open items, full text, 2026-11-11); the Recommendation below stands.
- **Recommendation**: HISTORY: "2026-11-11 | `rules_contract_golden` (LAYER-DOWN-1, PR5-29) | re-recorded | ruleset `marrow_actives` -> `channeled_actives` (`MARROW_ACTIVES` -> `CHANNEL_ACTIVES`); deck `marrow` -> `channeled` (Emberwake's older saves load through `DECK_KEY_ALIASES`); terms `marrow_mark` / `cold_steel_mark` -> `channeled_mark` / `standard_mark` (Emberwake's `terms.json` names them Marrow / Cold Steel); `[Spasm]` -> `[Strike]` | kit purity (audit 2.1); rename only (2 contract lines), no battle or sim golden moved". HISTORY also: the seed `pk1s3fix.json` no longer re-applies (deck `marrow` is now an unknown key). STRUCTURE: :44 `marrow_errors` -> `channeled_errors`; :40 the lore-word check covers engine and kit code including tests, any case; the `tools/fc.sh` row (G3 as above); `tools/planner/` gains `test_layers_check.py`; `emberwake_world` `pub mod save` wraps the kit's save with `DECK_KEY_ALIASES` (test `tests/save_aliases.rs`). GAME_BRIEF :82: "Cold Steel" is Emberwake's `standard_mark` term, the marrow round its `channeled_mark`. `combat_hooks.md` :326 `channeled_actives`, :328 "the game term `channeled_mark` names a channeled version (Emberwake: Marrow)". Sim notes: `channeled_actions`. `kits/rpg/README.md`: done. Question: Emberwake's battle log now reads `[Strike]` where it read `[Spasm]` (no golden held it); keep, or a game term for the tag.

### FB-206: PR5-29 part (2) (engine) not built: the overlay names are a file format; the RPG nouns need an audit

- **Spec**: READY row PR5-29 part (2) (`aegis_core/src/assets.rs` overlay names; RPG nouns in `engine/crates/*` public APIs and samples)
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: The idle overlay names (`IDLE_OVERLAYS`, `EFFECTS`: veins, eye_glow, ash, smoke) are 51 engine lines (`aegis_core/assets.rs`, `aegis_ui/{portrait_fx,specimens,vignette}.rs` and tests) and also the face-sidecar data format: 137 references across Emberwake's `assets/portraits/**/*.face.json`, the catalog cues, `timeline_kit.json` and the GUI `portraits.rs`; generic primitives need a vocabulary call (for example `lines`, `point_glow`, `particles_rise`, `drift`) plus a sidecar migration or alias. RPG nouns in engine `.rs` (lines / files): battle 95 / 18, round 269 / 50, block 211 / 27, encounter 16 / 3, ruleset 20 / 2; the public names that are file formats are `aegis_cli::scenario` `Launch.encounter` / `ruleset` (scenario JSON keys and CLI args; 2 game scenario files; `tools/fc.sh scenario shots`) and `aegis_host::playtest` `battles` (the recording format under `games/emberwake/playtests`); `round` and `block` need a per-hit audit (many are maths or UI uses).
- **Recommendation**: split part (2) into three rows: (2a) scenario and playtest keys renamed with load aliases (no golden expected); (2b) the overlay vocabulary after Design names the primitives (with a sidecar alias so the shipped portraits keep working); (2c) the remaining nouns after a per-hit audit (public API names only; maths and UI words stay).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
