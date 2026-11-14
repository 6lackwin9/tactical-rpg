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
| PR5-36 to PR5-46 | done (FB-235 to FB-255, answered through @8f06e0fe) | `develop` | full rows in `docs/archive/FEEDBACK_CLOSED.md` |
| PR5-47 `MAP-HOOKS1` | done (FB-256; FB-257 readings and questions): act backgrounds and layout sidecars, landmarks, focus-only painted roads with up to 3 ways, labels, badge, party ring, floating Legend; 0 golden diffs | `develop` | - |
| PR5-48 `MAP-LAYOUT-ED` | next | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-255 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-14). New items start at FB-256. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-256: PR5-47 `MAP-HOOKS1` landed

- **Spec**: READY Section 1 row PR5-47; AO-01, AO-02, AO-04 (4) (the `PR5-47` squash commit on `develop`, 2026-11-14)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: (1) `rpgkit_content::art::landmark` / `world_map_layout`; `emberwake_gui/src/region.rs` `layout_ok` allows `map/landmarks/*.png` and `backgrounds/world_map/*.png|*.json`. (2) `emberwake_gui/src/worldmap.rs`: the background is `backgrounds/world_map/<region>.png` up the current location's region chain, then `<world>.png`; the layout sidecar (`MapLayout`, same name chain) overrides `map_pos` (unclamped, `scale` default 1); landmarks drawn at 48 x scale px, foot 8 px below the ground point, pin / circle fallback; hover / focus glow; locked places 85% grey, 80% opacity. (3) Seed `seeds/sunk_ward_map_pos.json` (the 9 AO-01 `map_pos`); `games/emberwake/assets/backgrounds/world_map/sunk_ward_quarter.json` (9 landmarks at scale 1, the 10 `world_diamond.py` route paths; the preview's route keys already match the route ids); **placeholder** `backgrounds/world_map/sunk_ward_quarter.png` (a copy of `high_bastion.png`, for the Art/UI inbox: Act 1 background take 1). (4) Kit `travel::paths` / `travel_via` (FEEDBACK kit README line); roads are lantern dots on splines, focus-only (nothing at rest, open roads dimmed on hover, every way to a pick with the chosen one gold and the rest pale, hover preview and click on the map or the side panel's "Pick a way" rows; travel and the run-ending question follow the picked way, `runover.rs` `trip_ask_via`, `campaign::preview_trip_via`); game-manifest key `world_map_lantern_flicker` (off for Emberwake; reduce motion wins). (5) Cyan party ring; pulsing quest badge (top right, static under reduce motion); faint labels above marks, a full plate on hover / focus / pick, pinned to the top edge only when clipped; floating Legend (`FloatPanel`: first free corner, else centred and unlocked; active quests, board jobs, "Your party"; drags only when unlocked; minimises to a "Legend" button in place); the old route / pin legend, "You are here" and "!" are gone; no backend ids in any map text. The map viewport matches the preview (496x328 with 34 / 22 px padding, the background as cover). Tests: kit `travel_paths.rs` (7), `world_map_options.rs` (2), `travel_modes` (+1), `art.rs` (+1); GUI `region.rs` (+1), `worldmap.rs` (20, incl. review shots via `AEGIS_REVIEW_DIR`), `board.rs` badge position updated. Gate 3270 passed, 0 failed; 0 golden diffs; content check unchanged.
- **Recommendation**: GAME_BRIEF / SCREEN_ATLAS (world map): the lines above. STRUCTURE (assets): `map/landmarks/<location>.png`; `backgrounds/world_map/<name>.json` layout sidecars; seed `seeds/sunk_ward_map_pos.json`; tests `rpgkit_world/tests/travel_paths.rs`, `rpgkit_content/tests/world_map_options.rs`. HISTORY: "MAP-HOOKS1: the world map's static route / pin legend, the 'You are here' line and the '!' mark gave way to the floating Legend, the party ring and the pulsing badge (AO-01 (5))." Art/UI inbox: replace `sunk_ward_quarter.png` in place; landmark pngs at `map/landmarks/<id>.png`.

### FB-257: PR5-47 readings and questions

- **Spec**: READY row PR5-47 (@ the `PR5-47` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: (a) Locked places: the kit shows a place once it is visited or one open road from a visited place, so a cut-off place is mostly hidden rather than grey, while Art/UI's preview draws every place (locked ones grey). (b) The Legend cannot tell main quests from side jobs (no data field), so it lists quest names plainly. (c) The Legend's position and lock reset when the map reopens. (d) A region background with no region sidecar falls back to the world's sidecar (as the row says), whose positions may belong to another picture. (e) Three Act 1 places have no AO-01 position and keep their `map_pos`: `alleyway_of_cinders`, `meat_market_rooftops`, `canal_bridge` (prologue places, not route nodes, so not on the world map). (f) `travel::paths` reports supplies uncapped (the path's cost; `trip()` still reports the capped spend).
- **Recommendation**: (a) a kit fog option "show every place of the region, locked ones grey" (recommended, matches the preview) or keep the fog; (b) a quest `kind` field (main / side) if the Legend should label them; (c) keep the Legend's position and lock in settings (like `map_legend_open`); (d) tie the sidecar to the background actually found (recommended) or keep the fallback; (e)-(f) confirm.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-12 are closed and moved to the archive (2026-11-11). New notes start at UN-13.
