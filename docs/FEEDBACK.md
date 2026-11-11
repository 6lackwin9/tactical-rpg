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
| PR5-30 `LAYER-DOWN-2` | done (FB-207): gear families and terrain types from data, Dojo presets and cast as data rows; 0 golden diffs | `develop` | next: PR5-31

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-206 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-11). New items start at FB-207. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-207: PR5-30 `LAYER-DOWN-2` landed (gear families and terrain types from data; Dojo presets and cast as data)

- **Spec**: READY Section 1 row PR5-30; audit 2.1 (the `PR5-30` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: (1) Gear: `skill::GEAR_TYPES`, `OPTIONAL_GEAR_TYPES`, `unit::WEAPON_TYPES`, `item::FOCUS_TYPES` retired; the checks read the item family table (`listed_gear_tokens`: the schema's order first, then new families, so `checks_golden` messages are unchanged; `focus_tokens` from the `focus` rule; a family's id is its gear token, any id but `unarmed`); without a family table the schema's `weapon_type` options are the fallback; schema `weapon_type` gains `options_from: gear_types`; the whole-catalog check `item_families:required` is retired (always true once the table defines the tokens). The content writer now leaves out any `omit_default` field at its default, as the world writer does (no shipped row holds one at its default; every shipped file resaves byte for byte). (3) Terrain: `grid::terrain_ids_of` (the 17 built-in types first, new `terrain.json` ids after, so built-in indices never move), `TerrainTable` ids / glyphs, `Grid::glyph`; new terrain field `glyph` (one character, unique; a built-in type keeps its own; a glyph-less new type loads but cannot be placed); encounter rows, zones and `ground` accept the catalog's glyphs. (2) Dojo: `dojo::HEROES`, `DUMMY`, `Preset`, `PRESETS` retired (the audit's id list did not match the code; the ids in `dojo.rs` were the 16 preset items, the two Dojo heroes and the dummy); `presets_of(cat)` lists `dojo_presets.json` rows in file order; `cast_of(cat)`: heroes are units with the new field `dojo: hero` (catalog order), the dummy the first `dojo: dummy`. Seed `layerdown2.json`: the 8 old presets as rows in their order, `dojo_warrior` and `dojo_ranger` `hero`, `training_dummy` `dummy`; Emberwake's Dojo is unchanged. Tests: `emberwake_world/tests/custom_gear_family.rs` and `custom_terrain.rs` (a new family and a new terrain row validate and play without kit code), kit `terrain_rows.rs`, `dojo_presets.rs` and `weapon_types.rs` rewritten. Gate 2960 passed, 0 failed, 4 ignored, 0 warnings; 0 golden diffs. Review: change-reviewer PASS (5 low, below).
- **Recommendation**: HISTORY: "2026-11-11 | `dojo::PRESETS`, `HEROES`, `DUMMY`; `GEAR_TYPES`, `OPTIONAL_GEAR_TYPES`, `unit::WEAPON_TYPES`, `FOCUS_TYPES`; the `item_families:required` check | retired (LAYER-DOWN-2) | Dojo presets and cast are data (`dojo_presets.json`, unit field `dojo`); gear types are the family table's families | kit code names no game content (audit 2.1)". GAME_BRIEF: "Gear types are the item family table's families (a new family is a new gear type, data only). Terrain types: the 17 built-in plus any `terrain.json` row with a new id and a one-character `glyph`. Dojo presets are `dojo_presets.json` rows; the Dojo cast is the unit field `dojo` (hero, dummy)." STRUCTURE: Dojo presets and cast in `games/<id>/data/catalog/dojo_presets.json` and unit `dojo`, no kit built-ins. `kits/rpg/README.md`: done. Low follow-ups (a small row if wanted): (1) the schema's `weapon_type` options (the pre-family fallback) still list `scourge`, `censer`, `bone_saw`, `sprayer`, and `check/skill.rs` has `PRE_FAMILY_ARMOR` (plate, leather, cloth): trim to neutral tokens or retire pre-family support (a `checks_golden` question); (2) the family-name message "named after its gear token (one of ...)" now misleads (any id but `unarmed` is accepted); (3) kit tests `dojo_member_grants.rs`, `dojo_timeline.rs`, `know_potency.rs` still use Emberwake unit ids as fixture constants; (4) a catalog with no `dojo: hero` unit still panics in `DojoSession::hero` (as before); (5) `check::simple::terrain_table` is rebuilt per row check. Questions: should a game without `dojo_presets` rows get a generic preset (now: none); should the off-hand categories (shield, orb, tome) open up too (a `checks_golden` re-record).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
