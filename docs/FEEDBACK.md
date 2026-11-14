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
| PR5-36 to PR5-48 | done (FB-235 to FB-259, answered through @35c365b9) | `develop` | full rows in `docs/archive/FEEDBACK_CLOSED.md` |
| PR5-49 `MAP-POLISH1` | done (FB-260; FB-261 questions): region `map_reveal` with grey locked places, sidecar beside the drawn background, location `art`, saved Legend, Act 1 `map_pos` = sidecar, editor end anchors and pins; 0 golden diffs | `develop` | - |
| PR5-50 `KIT-HOOKS-CLEANUP` | next | - | - |
| PR5-51a to PR5-56 | queued, in order (user 2026-11-14) | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-259 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-14). New items start at FB-260. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-260: PR5-49 `MAP-POLISH1` landed

- **Spec**: READY Section 1 row PR5-49; FB-257 (a, c, d), FB-259 (a, c), AO-04, AO-05 (the `PR5-49` squash commit on `develop`, 2026-11-14)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: (1) Region field `map_reveal` (kit rule, FEEDBACK kit README line); Sunk Ward's `sunk_ward_quarter` has it on (seed `seeds/mappolish1.json`): after the Prologue the map shows every routed Act 1 place, the unreachable ones grey (85% grey, 80% opacity, no locked roads on hover); the three prologue places stay off; the world map's agent state gains `places` and `locked`. (2) `worldmap::layout_name`: the map reads only the sidecar beside the background png it drew (F7 editing edits that one); location `art` override (`art::landmark(id, art)`, used by the map, the editor and the sidecar check). (3) Settings key `world_map_legend` `{pos, locked}` next to `map_legend_open`, saved on drag end / lock toggle, restored on reopen (a partly off-map place is pulled in, a fully off-map or bad value dropped; minimised is not kept). (4) The 9 Act 1 `map_pos` equal the final `sunk_ward_quarter.json` positions (test `the_shipped_map_pos_match_the_act_one_sidecar`). (5) Engine `aegis_ui::layout_editor`: `Polyline::ends` drawn as fixed diamond anchors with a live smooth preview (`end_points`, `chain`, `preview`; `spline` moved here from the game's world map, which re-exports it), end-segment insert handles, `Marker::pin` / `PIN_PX` 14 (a marker without a picture is its pin or a dot), `marker_art`; agent state adds `ends` / `pin` only when set. Tests: engine `layout_editor.rs` (21, 6 new), kit `map_reveal.rs` (5), `world_map_fields.rs` (4), GUI `worldmap.rs` (29); `talk::tests::the_sunk_ward_hub_opens_its_panes_and_the_world_map` now expects the kiln shown grey (reveal on). Gate 3326 passed, 0 failed; 0 golden diffs.
- **Recommendation**: GAME_BRIEF (world map): "A region's `map_reveal` (Sunk Ward: on) shows every routed place of the region, the unreachable ones grey; unrouted places stay off. A location's `art` names its landmark picture. The map reads only the sidecar beside the background it drew. The Legend's place and lock are kept in settings `world_map_legend`." engine/README (`layout_editor`): "`Polyline::ends` fixed anchors with a live spline preview (`spline` lives here); a marker without a picture is its 14 px pin; `marker_art`." STRUCTURE: tests `rpgkit_world/tests/map_reveal.rs`, `rpgkit_content/tests/world_map_fields.rs`; seed `seeds/mappolish1.json`. HISTORY: "PR5-49: `art::landmark(id)` became `art::landmark(id, art)`; the world map no longer applies a lineage name's sidecar over another name's background (FB-257 (d)); `worldmap::spline` moved to `aegis_ui::layout_editor::spline`." Decision: region-wide reveal is a region field, since the flagged region is also the scope.

### FB-261: PR5-49 questions

- **Spec**: READY row PR5-49 (@ the `PR5-49` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: (a) "Locked" means no trip is possible now, so a place on an open road that the party has not reached yet (two legs past a visited place) is also drawn grey, the same rule the GUI used before. (b) The Legend's minimised state is not kept across reopens (the row asks for position and lock only).
- **Recommendation**: (a) keep (recommended), or draw open-but-unvisited places differently from shut ones (a kit flag); (b) keep minimised in `world_map_legend` too if wanted (a one-line change).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-12 are closed and moved to the archive (2026-11-11). New notes start at UN-13.
