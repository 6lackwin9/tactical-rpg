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
| PR5-7, PR5-9 to PR5-13b, PR5-18, PR5-19, PR5-26 to PR5-35 | done, released to `main` in #62 (FB-190 to FB-234, answered @cf3d93b4) | `main` | moved to `docs/archive/FEEDBACK_CLOSED.md` (Build Status rows, 2026-11-11) |
| PR5-36 (parts 1, 2, 4), PR5-37, PR5-38, PR5-39 | done (FB-235 to FB-241, answered @6edec2a1) | `develop` | full rows in `docs/archive/FEEDBACK_CLOSED.md` (FB-235 to FB-241); follow-ups queued as PR5-40 to PR5-42 |
| PR5-40 `ACT1-HOOKS1` | done (FB-242; FB-243 questions): mandatory scenes refuse Back / Esc, New Game preselects `start_worlds[0]`, encounter `survivors` count, `supplies` grants, shop `discount`; 0 golden diffs | `develop` | - |
| PR5-41 `ACT1-DATA1`, PR5-42 `WORLD-SIM1` | queued, in sequence (user 2026-11-12) | - | PR5-41 next |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-241 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-12). New items start at FB-242. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-242: PR5-40 `ACT1-HOOKS1` landed

- **Spec**: READY Section 1 row PR5-40; FB-241 (a, c-d) answer; AUDIT-04, AUDIT-05, AUDIT-08 (feed8108) (the `PR5-40` squash commit on `develop`, 2026-11-12)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: (1) Kit `DialogueRunner::scene` / `must_choose` (`rpgkit_world/src/dialogue.rs`): a queued scene that is open, offers no farewell and has a reply that can actually be picked (unlocked and passing `precheck`, on the current node or one reached through `next`) refuses Back, Esc, right-click and Farewell in `emberwake_gui/src/talk.rs` `Services::back`; a scene with nothing pickable can still be left, so it never traps the player; no dialogue ids named. (2) `Source.start_worlds` / `Data.start_worlds` and `new_game::default_world`: New Game preselects the manifest's first playable start world (`sunk_ward`), else the first playable world. (3) World encounter `survivors` {`flag`, `unit`} (schema `omit_default`) sets an int flag at every settle to the guests that lived (`battle::Tally.guests`, `GuestEnd`); `supplies` in quest template `rewards` and the dialogue `give` effect (`quest::reward_apply`, capped by `supplies_max`); world shop `discount` {`requires_flags`, `mult` 0.1..1.0} (`omit_default`) prices stock and loot at ceil(price x mult), min 1, in `shop::offers` (pane, buy and trade). The content check now validates every quest template's rewards (`rewards_errors` existed but was not called). Tests: 5 kit `rpgkit_world` files (`scene_must_choose`, `start_worlds`, `guest_survivors`, `supplies_grants`, `shop_discount`), 3 kit `rpgkit_content` (`survivors_checks`, `shop_discount_checks`, `supplies_checks`), `emberwake_world/tests/act1_hooks.rs`, GUI `talk::tests::back_and_esc_leave_a_scene_with_a_reply_to_pick_open`, `menus::tests::new_game_preselects_the_manifests_first_start_world`. 0 golden diffs; no shipped data touched. (4) AUDIT-04: `PAct::ConfirmGear` calls `emberwake_world::weapons::sync` (a stashed masterwork wielded on Confirm opens its technique at once); the gear draft and `gear_preview` show the draft as wielded through the kit's `weapons::sync_party` (no state change before Confirm); Gear slots show durability and MW; the Stats tab adds one read-only line under the stat table (Mastery, Techniques, Keystone, Weapons; full rows in its hover card; the derived rows keep their places). (5) AUDIT-05: `classes_errors` runs `duplicate_skill_errors` on world `classes.<id>.skill_path` overlays (shipped data stays clean). (6) AUDIT-08: the masterwork reforge refuses any masterwork, worn or whole. Tests: `weapon_recipes::a_damaged_masterwork_is_refused_by_the_masterwork_reforge`, `duplicate_skills::a_world_class_overlay_path_holding_a_technique_it_may_master_is_refused`, GUI `party::tests::confirming_gear_syncs_a_stashed_masterwork_and_opens_its_technique_at_once`. GUI `party::tests::the_pk2_line_fits_between_the_stats_and_the_passives` (layout: no overlapping rows; proof render with `AEGIS_REVIEW_DIR`). Gate 3161 passed, 0 failed. Seed shapes for PR5-41: `enc_evt_02` `"survivors": {"flag": "sunk_ward.saved_youths", "unit": "tithe_captive"}`; `{"effect": "give", "supplies": 3}`; template `"rewards": {"supplies": 3}`; shop `"discount": {"requires_flags": [{"key": "<flag>", "value": true}], "mult": 0.8}`.
- **Recommendation**: GAME_BRIEF "Where to look": "Mandatory scenes: `DialogueRunner::must_choose` (kit `dialogue.rs`), GUI `talk.rs` `Services::back`"; "New Game default world: `new_game::default_world` (`start_worlds`)"; "Act 1 hooks: encounter `survivors`, `supplies` rewards / `give`, shop `discount` (kit `battle.rs`, `quest.rs`, `shop.rs`)"; "Character screen mastery and weapon display (AUDIT-04): `emberwake_gui/src/party.rs` `pk2_rows`; on the Stats tab one compact line `stats.pk2` under the stat table (mastery, techniques, keystone, weapon durability / MW; full rows in its hover card); on the Gear tab `instance_note` (slot durability / MW) and `wielding` (the draft and preview synced through `weapons::sync_party`); Confirm syncs weapon instances (`weapons::sync`)". HISTORY: "AUDIT-08: the masterwork reforge used to accept a worn masterwork (a costlier repair); it now refuses any masterwork, and a plain reforge or repair restores durability."; "ACT1-HOOKS1: a farewell-less scene could be dismissed with Esc and was lost for good (FB-241 a); Back is refused while a reply can be picked; option (2), re-queueing the scene, was not taken (user 2026-11-12)." STRUCTURE: no new paths (tests in existing `tests/` folders). Known-issues row (cloud): `alsa-sys` build fails when the session-start hook did not run; `apt-get install -y libasound2-dev` (already in `docs/workflow/cloud.md`, hit again 2026-11-12).

### FB-243: PR5-40 readings and questions

- **Spec**: READY row PR5-40 (3) (@ the `PR5-40` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: The specs are silent on these; built defaults: the shop discount is one `{requires_flags, mult}` object (not a list, so no stacking rule was needed) and lowers stock and loot buy prices only, not the supply price (its help text says it ignores multipliers) nor sell prices; the `survivors` flag is set on every outcome (won, lost, fled) and a guest that fled the field counts as surviving (M2's captive gets `flee_ratio` 0 in PR5-41, so this does not matter there).
- **Recommendation**: (a) keep supplies out of the discount (alternative: the discount covers supplies too); (b) keep fled guests as survivors (alternative: count only guests on the field at the end; or a content-check warning when `survivors` sits on an encounter whose guests can flee); (c) AUDIT-08's refusal only says the weapon is already a masterwork (alternative: point to the plain reforge / repair); (d) AUDIT-05 checks the overlay path against techniques and keystone only (alternative: also re-check units' `extra_skills` against the overlay path).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-12 are closed and moved to the archive (2026-11-11). New notes start at UN-13.
