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
| PR5-36 to PR5-44 | done (FB-235 to FB-253, answered @6edec2a1, @08ccf4f5, @1c96b539, @e4505f78) | `develop` | full rows in `docs/archive/FEEDBACK_CLOSED.md`; follow-ups queued as PR5-45, PR5-46 |
| PR5-45 `SKERRIT-CRAFT-HOOK` | done (FB-254): recipe target `stash` and effect `craft`; 0 golden diffs | `develop` | - |
| PR5-46 `ACT1-DATA3` | done (FB-255): Skerrit's craft recipes, the Ivo / Ida stake consumers, "Snapped Alum Inhaler"; 0 golden diffs | `develop` | queue empty: waiting for Design (FB-254, FB-255) |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-253 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-13). New items start at FB-254. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-254: PR5-45 `SKERRIT-CRAFT-HOOK` landed

- **Spec**: READY Section 1 row PR5-45; FB-252 (1) (the `PR5-45` squash commit on `develop`, 2026-11-13)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Recipe target `stash` (`TARGET_STASH`; `RECIPE_TARGETS` member, weapon, item, stash; `kits/rpg/schema.json` `content.recipes` target options and help) and effect `craft` {`item`, `count` 1..99, default 1} (`FX_CRAFT`, `CRAFT_COUNT_DEFAULT`). `rpgkit_world::recipe` runs a stash recipe with no pick and, after the usual `requires` and cost, adds the items to the stash and calls `weapons::sync` (the row's `weapons::grant` does not exist; `sync` mints one fresh full instance per new weapon copy from `durability.from_act` and does nothing before). Notes "{Item} added to stash." / "{Item} x{count} added to stash." (the row's one-item text "added {item_name}." was capitalised to the file's note style). The content check covers unknown or missing items, counts outside 1..99 or not whole, and target-bound effects on the wrong target. `RecipePane` needed no code change (no picker without a member, item or weapon row). Tests: `rpgkit_world/tests/craft_recipes.rs` (4), `rpgkit_content/tests/recipe_checks.rs` `a_stash_recipe_crafts_a_known_item_in_range` (the target-list error now names stash), `emberwake_gui` `recipe::tests::a_stash_recipe_runs_with_no_pick_and_refuses_an_unpaid_cost`. Gate 3234 passed, 0 failed; 0 golden diffs; existing recipe rows re-save byte-identical.
- **Recommendation**: GAME_BRIEF recipes line: "Recipe target `stash` and effect `craft {item, count}` (PR5-45): a recipe can make items straight into the stash; weapons become instances from Act 2." STRUCTURE: `rpgkit_world/tests/craft_recipes.rs`. Open: should `craft` refuse quest-type items (the row only asks for a known catalog item)?

### FB-255: PR5-46 `ACT1-DATA3` landed

- **Spec**: READY Section 1 row PR5-46; FB-252 (1, 3, 4) (the `PR5-46` squash commit on `develop`, 2026-11-13)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Seed `games/emberwake/data/seeds/act1fix3.json` (8 steps, idempotent). (1) `catalog/recipes.json`: `skerrit_bone_cestus` ("Shards into a Barbed Bone Cestus": 25 Scrap + 2 `calcified_bone_shard` -> `craft barbed_bone_cestus` x1) and `skerrit_bone_quarrel` ("Shard into a Bone Quarrel": 10 Scrap + 1 shard -> `craft bone_quarrel` x1), `npc: skerrit_bone_wright`, `target: stash`, gated on `sunk_ward.skerrit_open` (both `[PROVISIONAL]`; no field holds the tag; the names fit the writer's 40-character limit, the descriptions are Build's). (2) `ash_crawl_crossroads` site `toll_pen` hidden by `sunk_ward.ivo_lost`; `q_a1_red_pen` `closed_text` and the `ivo_lost` variant (Closed on Talia's board); `dlg_talia`: "Who is Ivo?" hidden by `ivo_lost`, new "What happened to Ivo?" (once, `talia_ivo_lost_told`) -> Talia: "Her hand closes on the rod until the notches bite. \"The Red Pen swept the pen at zero. They chalk a child and call it arithmetic. I'll remember who held the pen. Go on. Read the board twice.\""; `dlg_corvo_gate`: "What do you keep behind the gate?" hidden by `ida_lost`, new "About Ida." (sets only `corvo_settled`, the flag that opens the gate on the bribe paths; no `corvo_bribed`, no -20 Alert) -> Corvo: "He does not look up from the stone of his leg. \"They took her at the count. Nothing left to keep behind the gate. Go through. I won't stop you, and I won't say thank you.\"" (3) `alum_inhaler` "Snapped Alum Inhaler" and `q_hook_counter_still`'s reward text (the item description still says "Elyse's prototype alum inhaler"). Acceptance tests in `act1_story.rs` (Skerrit crafting before / after `skerrit_open`, the Ivo and Ida paths through the real `d_zero` firings, the Alert unchanged on "About Ida.", the rename). Gate passed; content check 0 errors; 0 golden diffs.
- **Recommendation**: HISTORY: "PR5-46: a lost Ivo can no longer be rescued (CL-04); a lost Ida opens Corvo's gate without the bribe's Alert cut." GAME_BRIEF: Skerrit crafts the cestus and quarrel from Bone Shards once his hook opens. CANON / Art: review Talia's and Corvo's new lines (Build-written; "the rod" means Talia's cipher rod, or change it to her crutch) and the two recipe names / descriptions.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-12 are closed and moved to the archive (2026-11-11). New notes start at UN-13.
