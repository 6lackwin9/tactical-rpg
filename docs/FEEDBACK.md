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
| `TESTCLEAN1` | done | `develop` | Old Temp folders still there: UN-06. |
| `SKILLS1` | done | `develop` | Check script follow-up is READY row 22 (`SKILLS1-FB`). |
| `KITIDS1` | done | `develop` | - |
| `BASERULES1-M` | done | `develop` | Follow-up is READY row 21 (`BASERULES1-M2`). |
| `U1.2` (`U1_STATS`) | slice 1 done | `develop` | Pipeline and pool view. |
| `U2.2` (`U2_CLASS`) | slices 1 and 2 done | `develop` b20511a1 | Share vectors (S1); one unit builder with U4.2-S2 (S2, READY row 10): FB-63. |
| `U3.2` (`U3_GEAR`) | slice 1 done | `develop` b20511a1 | `block_cost` retired; two named golden re-records: FB-61. Slice 2 (family table) not started: next session. |
| `U4.2` (`U4_PASSIVES`) | slices 1 and 2 done | `develop` b20511a1 | Typed grant view (S1, FB-62); source gather in the unit builder (S2, FB-63). |
| READY rows 11-22 | queued | - | Not started; for the next build session. |

## 2. Feedback Items (Append-Only)

Each entry uses: `### FB-nn: <short title>` with fields `Spec`, `Kind` (`feasibility` | `conflict` | `cost` | `sim` | `docs-delta`), `Status` (`open` | `answered` | `closed`), `Finding`, and `Recommendation`.

FB-01 to FB-60 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-07). New items start at FB-61. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-61: U3.2-S1 landed: docs delta and two item notes that promise removed behaviour

- **Spec**: `docs/sessions/U3_GEAR.md`; FB-37
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Gear `block_cost` is retired: gone from the schema, item rows, families, the gear budget (`budget.rs` `W_EXTRA_BLOCK` removed, `LOCKED` now 49) and the checks; `source.rs` `RETIRED_NUMBERS` makes a row that still sets it fail on load. Data moved through seed `u32s1` (22 item families, 2 items). Two intended golden re-records: `checks_golden` (only the block_cost check lines) and `rules_contract_golden` (one line, `locked budget.rs::W_EXTRA_BLOCK gear`). The `special_note` texts on `shd_tower` and `wpn_glass_wand` still describe the removed block cost. Old kit test fixtures and `timeline_kit.json` still carry `block_cost` keys; the loader accepts and ignores them (build-side tidy, no ask).
- **Recommendation**: `docs/HISTORY.md` row naming both golden re-records; GAME_BRIEF and `U3_GEAR.md` gear lines without block cost; new `special_note` text for the two items (a content row for the asset lane, or say to drop the notes).

### FB-62: U4.2-S1 landed: docs delta

- **Spec**: `docs/sessions/U4_PASSIVES.md`
- **Kind**: docs-delta
- **Status**: open
- **Finding**: New `rpgkit_rules/src/grants.rs`: `grants::read(effects)` gives `Grant`s in row order (`Modifier`, `Proficiency`, `GrantSkill`, `GrantPool`, `Trigger`); `Target::combine()` is Sum, SumCapped(100) for resist, Product for `stagger_taken`, Max for foresight and unlock, OrFlag for forced-move immunity; `Unit::apply_grants` folds them; `apply_effects`, `apply_band` and `Proficiency::with_drills` read it. No row rewritten; goldens unchanged.
- **Recommendation**: `docs/STRUCTURE.md` rpgkit_rules row: `grants.rs`; `engine/docs/PORT_NOTES.md`: the passive and gear effect readers now go through `grants`.

### FB-63: The single unit builder landed (READY row 10): docs delta and three questions

- **Spec**: `docs/sessions/U2_CLASS.md`, `U4_PASSIVES.md`; FB-12, FB-23, FB-24, FB-28
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `rpgkit_rules::build_from(cat, UnitBuild)` builds every unit (session spawn through `Roster::overrides`, which replaces `customize`; WorldRoster and SimRoster through `battle::character_overrides`, which replaces `apply_character`; heroes through `progression::build_as`; Dojo and sim stats through `UnitBuild::row`); `build_unit` and `build_unit_in` are wrappers. Static and growth units keep their old step orders inside it, so no golden moved. Passives are re-gathered per build (`progression::member_passives`); every shipped hero's list equals its new-game list. `training_dummy` is a fixed warrior with no gear (seed `u22s2`); a fixed unit with no class is a content error; class `art` defaults to `""` and an empty class `art` is still an error (rule A1). Open: (1) should an empty class `art` mean the body named like the class? (2) the equip check (`state.rs:180`) and deck ownership still read the saved passive list, not the re-gathered one; (3) hero character `tuning` and named items still do not apply (RECRUIT1 forbids them).
- **Recommendation**: `engine/docs/PORT_NOTES.md:264` and `:277` (`WorldRoster::customize` becomes `overrides`), `docs/STRUCTURE.md` rpgkit_rules row and `kits/rpg/README.md:61` (Dojo uses `build_from`, not `build_unit_in`): name `build_from`, `UnitBuild`, `Overrides`, `Growth`, `Roster::overrides`, `character_overrides`, `member_passives`; GAME_BRIEF: "`training_dummy` is a fixed warrior with no gear; a fixed unit with no class is a content error"; HISTORY rows for the FB-23 error and the FB-28 re-gather. For (2), a follow-up row that moves the equip check and deck ownership to the re-gathered list, or "none" if the saved list is meant.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-05 are closed and moved to the archive (2026-11-07). New notes start at UN-06.

### UN-06: The old test folders in Temp are still there: the delete was blocked

- **For**: user
- **Status**: open
- **Note**: UN-01 approved deleting the pre-TESTCLEAN1 `emberwake_*` and `rpgkit_*` folders (about 19,000) in `%TEMP%`, but the session's permission check blocked the delete twice, so the build session will not retry or work around it. No cleanup agent or script covers them (the housekeeper is read-only; `tools/planner/cleanup.sh` removes only lane branches, removed worktrees and their build caches). They are harmless; new test runs no longer add any.
- **Ask**: Run this in PowerShell when no gate or build is running (it keeps the `aegis-target-*` build caches and anything under two hours old), or add a permission rule for it: `Get-ChildItem $env:TEMP -Directory | Where-Object { $_.Name -match '^(emberwake|rpgkit)_' -and $_.LastWriteTime -lt (Get-Date).AddHours(-2) } | Remove-Item -Recurse -Force`
