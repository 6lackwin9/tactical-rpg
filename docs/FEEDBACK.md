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
| `SKILLS1-FB` (READY row 22) | done | `develop` | `tools/planner/feedback_check.py`: FB-64. |
| `U3.2-S2` (READY row 8) | next | - | Then rows 11, 12, 13, 21 for R1 (FB-65). |
| READY rows 11-21 | queued | - | Not started. |

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

### FB-64: SKILLS1-FB landed: the feedback check script, one reading of "match", and docs delta

- **Spec**: `docs/READY.md` row 22; UN-05
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `tools/planner/feedback_check.py` checks this file against READY section 4: every open `FB-nn` / `UN-nn` here has a section 4 row; each row's Disposition is `accepted`, `deferred` or `policy` and its Follow-up is not empty; every Follow-up other than `none` is a task id in the Task ID column of section 1 or 3 (a range such as `U4.2-S3..S4` also matches `U4.2-S3`, both ways); a Feedback ID such as `FB-48a` is a parse error (one row per plain id); a row for an item in neither file is a warning. `archive` moves every item with a valid row to `docs/archive/FEEDBACK_CLOSED.md` with `closed: <disposition> (@<commit>)` plus `, follow-up <ids>`, the commit being the last one that changed READY.md. Reading taken: an open item with no section 4 row is a mismatch (exit 1), as the row asks; `--pending-ok` lists it as "waiting for Design" instead, for the build session's start-up pass. Today: all 43 section 4 rows valid; FB-61 to FB-64 and UN-06 wait for Design. Tests: `tools/planner/test_feedback_check.py` (17), run by a new `plannertests` step in `tools/fc.sh tooltests` and the gate.
- **Recommendation**: `.claude/skills/build-session/SKILL.md` step 3 (and the design-session skill's archive step): run `python3 tools/planner/feedback_check.py --pending-ok`, then `archive`; `docs/STRUCTURE.md` tools row: `tools/planner/feedback_check.py`; `CLAUDE.md` / gate wording: the gate also runs the tools/planner tests.

### FB-65: R1 needs Wave 2 rows W2-1 to W2-3 in section 1; build order for R1 first

- **Spec**: `docs/READY.md` sections 1 and 3; `games/emberwake/docs/MILESTONES.md` (R1); UN-03
- **Kind**: conflict
- **Status**: open
- **Finding**: The user's goal for the current build session is to build to R1 and release `develop` to `main`. R1 (Unit & Status Foundation: U1-U4, S1-S2, C01-C05) ships Option 1C Guard and 5-pip Poise, which exist only in Wave 2: section 3 marks W2-2 as closing the R1 Unit half and W2-3 the R1 Status half, and W2-1 (the U1/U2 formula switches) comes before both. Section 3 promotes Wave 2 only "after Wave 1 Rows 7-20", but rows 14-20 (A, F, P, I, E, T, W) are R2 and R3 work, so R1 waits on them as written. Wave 1 rows R1 needs: 22 (`SKILLS1-FB`, landed), 8 (`U3.2-S2`), 11 (`S1.2-S1`), 12 (`S1.2-S2/S3`), 13 (`S2.2`), 21 (`BASERULES1-M2`). R1 also needs FB-63 (2) answered before W2-2, which rewrites the same equip check (`state.rs:180`), and W2-2 carries the party-screen penalty display, mock first (FB-34), which needs the user's look.
- **Recommendation**: (1) Approve building rows 8, 11, 12, 13, 21 in that order before rows 14-20 (rows 14-20 stay in section 1 for after R1, or move to section 3). (2) Promote W2-1, W2-2 and W2-3 into section 1 now, below row 21, with their slice order and named golden re-records, so the build session moves on without waiting a session; W2-4 to W2-6 stay held. (3) Answer FB-63 (2). (4) Say how the user reviews the W2-2 party-screen mock (screenshot in FEEDBACK, or a desktop look before W2-2 lands). (5) Confirm the R1 release is the `develop` -> `main` PR right after W2-3 lands with a clean gate, and whether a balance run or the user's look is part of the R1 ship gate.

### FB-66: The cloud session-start hook installs the ALSA headers: docs delta

- **Spec**: user approval in the build session chat (2026-11-06); known-issues row "alsa-sys"
- **Kind**: docs-delta
- **Status**: open
- **Finding**: A fresh cloud container could not run `tools/fc.sh build` or the gate: `alsa-sys` (rodio -> cpal) needs `libasound2-dev` and `pkg-config`, which only CI's `gate.yml` installed. `.claude/hooks/session-start.sh` now installs both when `pkg-config --exists alsa` fails (warns, does not stop the hook, if apt fails).
- **Recommendation**: `docs/workflow/cloud.md` "Environment" bullet: the hook also installs the ALSA headers (`libasound2-dev`, `pkg-config`); optionally the known-issues row: cloud sessions get them from the hook.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-05 are closed and moved to the archive (2026-11-07). New notes start at UN-06.

### UN-06: The old test folders in Temp are still there: the delete was blocked

- **For**: user
- **Status**: open
- **Note**: UN-01 approved deleting the pre-TESTCLEAN1 `emberwake_*` and `rpgkit_*` folders (about 19,000) in `%TEMP%`, but the session's permission check blocked the delete twice, so the build session will not retry or work around it. No cleanup agent or script covers them (the housekeeper is read-only; `tools/planner/cleanup.sh` removes only lane branches, removed worktrees and their build caches). They are harmless; new test runs no longer add any.
- **Ask**: Run this in PowerShell when no gate or build is running (it keeps the `aegis-target-*` build caches and anything under two hours old), or add a permission rule for it: `Get-ChildItem $env:TEMP -Directory | Where-Object { $_.Name -match '^(emberwake|rpgkit)_' -and $_.LastWriteTime -lt (Get-Date).AddHours(-2) } | Remove-Item -Recurse -Force`
