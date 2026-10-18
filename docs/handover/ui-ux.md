# UI/UX handover

Written 2026-10-18 05:35 UTC by the UI/UX session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`. Dispatch mode is **economy** (user 02:35): logic-builder medium (sonnet), change-reviewer low.

## Landed this session (develop)

MAPVIEW1 full-screen maps (cf333ab), MAPREAD1-L2 map markers, legend, world pins and hover (73f31fa), COMBAT-UI2-L1 combat layout C plus UIR-5 leftovers (7f66209). UIR-5 is done with COMBAT-UI2-L1: delete its TASKS row in your first task commit.

## Waiting on the user

Mockups on the Design canvas "Map Screen Mockups" (https://claude.ai/artifact/2joZ2aDAARWS55hWnpCz2W), three decision cards in the old UI/UX thread (taps do not reach a new session: if the coordinator relays a pick, it quotes question, option and time):

- Dialogue layout: A centred popup / **B bottom bar with portrait (recommended)** / C speech panel by the speaker.
- Activity log: **A one-line ticker that expands (recommended)** / B toasts plus a Log tab / C docked panel.
- Quest tracker: A pinned card / **B card plus ! and ? map badges (recommended)** / C journal only with a Menu badge.

The picks become new build rows (they fold into UIR-6 for dialogue). Write the TASKS rows when the answers arrive.

## Queue (in order)

1. **COMBAT-UI2-L2** hover card replaces the inspector (spec `docs/specs/COMBAT_UI2.md` rule 5; hover 150 ms, keyboard focus, pin on click, Esc closes, never covers the hovered tile). Today the read-only inspector sits under the party column and scrolls at large text.
2. **COMBAT-UI2-L3** tactic dropdown + Autoplay toggle (AUTO1-L API in `scripts/combat/session/combat_autoplay.gd`; guests: `set_hero_tactic` refuses them, hide the row picker). Also bring `timeline_dock_ui.gd` text to the 14 px floor (still 7-9 px). Carry the AUTO1-L Done line.
3. **INTENT1-U** (INTENT1-L landed a56dda7: `CombatSession.enemy_intents()`, `intent_of`, `Combatant.has_foresight()`, README "Enemy intent tells"), **BOSS1-U** (BOSS1-L 5c85e7e: `CombatSession.phase_fired`, `CombatEncounter.phase_info`), **OBJ1-U** (OBJ1-L 3abdb63: `EncounterObjectives.text_of/tiles_of`, `CombatEncounter.guests/is_guest`, `protect`, `CombatSession.outcome`). All after L2.
4. The picked dialogue / log / tracker builds, then UIR-1..UIR-4, UIR-6, UIR-7 (`/mnt/project-files/ui-review/UI_REVIEW1.md`).
5. Skills screen: each skill's description (`{name}`/`{power}` templates) and only the active rank.

## Notes

- Peers: Gameplay `session_01HQvENsjr5dFTBZp6L6GkPU` (ITEMUSE1-L now unblocked; wants a ping when L2 lands for OBJ1-U/INTENT1-U order questions), Assets & Audio `session_018BCoHkCDtUL2gxECWiCHzk`, Design `session_01Bd1NNdVo4TtRxU9hZ7BcNP`. Tell Gameplay and Assets your new id.
- Layout C: `CombatHudMetrics` (`scripts/ui/combat/combat_hud_metrics.gd`) owns column widths, tile size and card lines; region names `top, left_top, left, center, right_top, right, bottom_left, bottom_center`. Text size is read when the arena is built.
- Maps: `RegionMapView.camera_offset/free_rect/float_panel`, `WorldMapView.layout` (backdrop and pins letterbox into the free area), `MapArt` templates for overlay/pin/badge keys (manifest files override).
- Gate in the cloud: `SUITE_TIMEOUT=580 bash tools/fc.sh gate > /tmp/claude-0/X.log 2>&1; tail -8`. Re-gate after a develop merge that touches shared combat or world code (it usually does).
- One screen per test case (many screens in one arena frame crash). Right click as back is read in `_input`.
- Renders: Xvfb `xvfb-run -a -s "-screen 0 1280x720x24" godot --rendering-driver opengl3 --path . ...`, scripts outside the repo, screenshots to `/mnt/project-files/<task>/`. After each landing, reply with screenshots and "press Restart runner".
