# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates `## 1. Build Status` and appends items to `## 2. Feedback Items` (feasibility findings, cost surprises, spec conflicts, sim results, and docs deltas for `README.md`, `STRUCTURE.md`, `GAME_BRIEF.md`, `HISTORY.md`).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| `ATTACKLOCK1` (slice 1) | built (off) | `develop` | Switch-on with the power test is `F1.2-S1` (FB-58). |
| `SIMCOUNT1` | done | `develop` | `sim feel counts=on`. |
| `VISCLAR1-D` | built | `develop` | User look deferred to after the combat silos (UN-02). |
| `TESTCLEAN1` | done | `develop` | Old Temp folders removed by Design (UN-06). |
| `SKILLS1` | done | `develop` | Check script follow-up `SKILLS1-FB` done. |
| `KITIDS1` | done | `develop` | - |
| `BASERULES1-M` | done | `develop` | Follow-up is READY row 21 (`BASERULES1-M2`). |
| `U1.2` (`U1_STATS`) | slice 1 done | `develop` | Pipeline and pool view. |
| `U2.2` (`U2_CLASS`) | slices 1 and 2 done | `develop` b20511a1 | Share vectors (S1); one unit builder with U4.2-S2 (S2, READY row 10): FB-63. |
| `U3.2` (`U3_GEAR`) | slices 1 and 2 done | `develop` | `block_cost` retired (S1); gear rules read the family table (S2, READY row 8): reach and printed check lists held, FB-67. |
| `U4.2` (`U4_PASSIVES`) | slices 1 and 2 done | `develop` b20511a1 | Typed grant view (S1, FB-62); source gather in the unit builder (S2, FB-63). |
| `SKILLS1-FB` (READY row 22) | done | `develop` 127f168 | `tools/planner/feedback_check.py`. |
| R1 path (READY R1-2..R1-8) | row 11 (`S1.2-S1`) building | `claude/s12s1-status-table` | Then 12, 13, 21, W2-1, W2-2, W2-3, then the R1 release PR. |
| READY rows 14-20 | queued (post-R1) | - | - |

## 2. Feedback Items (Append-Only)

Each entry uses: `### FB-nn: <short title>` with fields `Spec`, `Kind` (`feasibility` | `conflict` | `cost` | `sim` | `docs-delta`), `Status` (`open` | `answered` | `closed`), `Finding`, and `Recommendation`.

FB-01 to FB-66 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-07). New items start at FB-67. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-67: U3.2-S2 built: reach and the printed check lists wait on Design; docs delta

- **Spec**: `docs/sessions/U3_GEAR.md`; FB-31, FB-32, FB-36 (c), FB-37
- **Kind**: conflict
- **Status**: open
- **Finding**: Built with 0 golden diffs: `ALWAYS_RANGED_TYPES` is gone (`is_ranged_weapon` = the `always_ranged` rule or `base_range > 1`, so a data-only always-ranged family at range 1 is now ranged; no shipped item changes); `is_shield`, `off_hand_ok` and the spell group (`skill_in_group`, new resolved skill key `spell_source`) read the family's `engine_rules` (new resolved item key `family_rules`, which replaces `reach_rule`; the writer drops both); the check's membership tests (skill `weapon_types`, passive `proficiency.gear`, class proficiency, the `weapon_types` options) derive their lists from `item_families.json` (the 21 vs 18 "disagreement" is armour: gear = WEAPON, SHIELD and ARMOR families, weapon = WEAPON and SHIELD). Rows the loader did not resolve (the frozen golden catalog `timeline_kit.json`, hand-built test rows) fall back to `rpgkit_data::LEGACY_FAMILY_RULES` (eight family ids). Held, as the row cannot be met without moving a golden or gameplay: (1) **reach**: the family table has no `range_band`; the nearest field, `bands.<hand>.range.max`, is the allowed `base_range` band, which gives polearm 2 but scourge 1, so reading it would cut scourge reach from 2 to 1; `REACH_WEAPON_REACH` 2 stays. (2) **printed lists**: the check messages in `checks_golden` print `GEAR_TYPES` / `OPTIONAL_GEAR_TYPES` / `OFF_CATEGORIES` in an order the table cannot give (the table puts crossbow after cloth), and derived from the same table the family-level checks (gear-token name, off-hand ids, completeness) would always pass; the schema enum `items.weapon_type` also lists the tokens in the code order. Those lists stay code. (3) Resolved-row shape changed as above. (4) `OFF_CATEGORIES` and `FOCUS_TYPES` stay code too (printed in "Off-hand families are shield, orb, tome"; `FOCUS_TYPES` is read only by the no-family-table legacy check), so a new off-hand category still needs code (FB-31). Side effects on other catalogs, none on ours: with a family table, a skill, class or passive naming an optional gear type (polearm to sprayer) that has no family is now rejected (before, `OPTIONAL_GEAR_TYPES` let it through); the editor's `weapon_types` picker lists tokens in table order.
- **Recommendation**: (1) Reach, choose: (A) a new family field `range_band {min, max}` on polearm and scourge (kit schema slice, then a seed), the loader putting its max on the item with fallback 2; or (B) keep `REACH_WEAPON_REACH` as the value of the `reach` rule and drop "range_band.max replaces it" from FB-36 (c). Build recommends (B): no gameplay change and no new field. (2) Printed lists (and `OFF_CATEGORIES`), choose: (A) derive and re-record `checks_golden` in a follow-up, rewording or dropping the family-level checks; (B) make the kit schema's `items.weapon_type` options the one ordered list (a new family is then a schema edit, data only); or (C) leave them as code. Build recommends (C) for R1 and (B) later. (3) Confirm the resolved-row keys. Docs delta: PORT_NOTES "U3.2-S2: `is_ranged_weapon`, `is_shield`, `off_hand_ok` and the spell group read the family's `engine_rules` (resolved `family_rules`, skill `spell_source`); unresolved rows use `rpgkit_data::LEGACY_FAMILY_RULES`"; `kits/rpg/README.md`: the loader resolves `family_rules` and `spell_source`, the check derives gear and weapon tokens (`check::skill::gear_tokens`, `weapon_tokens`); GAME_BRIEF gear: "a new gear family using existing engine rules works in the rules from data alone; the printed check lists, reach 2 and fist are still code"; stale lines to fix: `engine/docs/PORT_NOTES.md:265` and `games/emberwake/docs/GAME_BRIEF.md:72` (the `reach_rule` item key), GAME_BRIEF `:207` ("Spell = ATTACK skill gated on staff or wand", "Orb and tome are off-hand only": both now family rules); `docs/STRUCTURE.md:43`: tests `rpgkit_content/tests/family_rules.rs`, `emberwake_content/tests/family_table.rs` and the updated `reach_rule.rs`; HISTORY: "U3.2-S2: `ALWAYS_RANGED_TYPES` retired (an always-ranged family at range 1 is now ranged; no shipped item changed); resolved `reach_rule` replaced by `family_rules`".

### FB-68: Squash-landed task branches were never deleted by branch-cleanup: docs delta

- **Spec**: `docs/workflow/cloud.md` "Branch cleanup"; user 2026-11-07 (task branches must be cleaned up after landing)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `branch-cleanup.yml` deletes a branch when it heads a merged PR or merging it into `develop` changes nothing. Squash-landed branches have no PR, and once `develop` later edits a file the branch touched (here `docs/FEEDBACK.md`, which every archive pass edits), the merge re-applies old lines, so the branch is kept for good: two of this session's three branches would have stayed. The build session now closes each branch after its squash landing with one commit whose tree is `develop`'s (`git commit-tree "origin/develop^{tree}" -p origin/<branch> -p origin/develop`, pushed to the branch): no history rewritten, `develop` untouched, and the next daily run (after the 6 h age limit) deletes it. Done for `claude/skills1-fb-feedback-check`, `claude/hook-alsa`, `claude/u32s2-family-table`.
- **Recommendation**: `docs/workflow/cloud.md` "Landing a task on `develop`" step 8 and "Branch cleanup": after the squash landing, close the task branch with that commit; drop the "delete those by name with `also_delete`" workaround for squash-landed branches.

### FB-69: S1.2-S1 built: the status table is kit data; two questions; docs delta

- **Spec**: `docs/sessions/S1_STATUS.md`; FB-48 (a)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: The Rust const `status::ROWS: [StatusRow; 17]` is replaced by the frozen kit table `kits/rpg/crates/rpgkit_rules/src/statuses.json` (17 rows, old order), compiled in with `include_str!` and read as `ROWS: LazyLock<Vec<StatusRow>>`; every reader (`row_of`, `cap_of`, `stacking_of`, `display_name`, `intent_bound`, `magnitude_unit`, `has_magnitude`, `magnitude_or_min`, `buffs`, `debuffs`, `defend_buffs`) keeps its signature and values. Each row carries the S1 lifecycle header (`id`, `display`, `kind`, `resistable`, `default_rounds`, `stacking`, `cap`, `magnitude_min`/`max`, `magnitude_unit`, `intent_bound`) plus today's `grant`, and one behaviour primitive tag (modifier: weaken, haste, slow, expose, fortify, empower; tick: bleed, regen, poison, burn; block_lock: stagger, stun, root; targeting: taunt, stealth; reactive: riposte; rule_flag: steadfast); the tags are unread until row 12. No golden moved; no kit schema entry (the table is compiled-in kit data, not loader content). Test `rpgkit_rules/tests/status_table.rs` (4). Open: (1) home of the file: `rpgkit_rules/src/statuses.json` (beside its reader) or `kits/rpg/statuses.json` (beside `schema.json`, embedded the same way); (2) `grant` (support / defend / "") is not in S1's header but the content check and `defend_buffs` read it: add it to the header, or replace it with a primitive or intent field in row 12?
- **Recommendation**: Answer (1) and (2) (Build defaults: keep the file where it is; keep `grant` until row 12 says otherwise). Docs delta: `docs/STRUCTURE.md` rpgkit_rules row: "`status.rs` (status readers and stacking; rows from the compiled-in kit table `statuses.json`, S1 header fields and `Primitive` tags), test `tests/status_table.rs`"; GAME_BRIEF "where to look": status rows in `kits/rpg/crates/rpgkit_rules/src/statuses.json`; `kits/rpg/README.md`: the status table is kit data, compiled in, not overridden by games; PORT_NOTES `status.rs`: since S1.2-S1 the rows are data (`statuses.json`), same order and values; HISTORY: "2026-11-07 S1.2-S1: the const `status::ROWS` became the frozen kit table `statuses.json` (S1 statuses as data over six primitives; no golden moved)". Process note: `tools/fc.sh test <filter>` matches test names, not test files (`cargo test --workspace --test <file>` runs one file); worth a line in `docs/EFFICIENCY.md` repeat findings.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-06 are closed and moved to the archive (2026-11-07). New notes start at UN-07.
