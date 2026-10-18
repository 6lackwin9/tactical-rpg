# UI/UX handover

Written 2026-10-18 02:40 UTC by the UI/UX session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`.

## Landed this session (develop)

REVIEW-F2, REVIEW-F3, COMBAT-LAYOUT, LOAD-SLOTS-UI, SETTINGS-UI, DISPLAY1, MENU-POLISH, SETTINGS-POPUP, BATTLE-FLOW (608d45a), SOUND-PREVIEW (bf53760), CONTROLS-HELP with the volume fix and Mute (3a2ede5).

## Queue (in order)

1. **Full-screen maps** (Tushar 02:23, pulled forward): the region map fills the screen, the map is larger than the view and scrolls (camera follows the party, clamps at edges, a smaller map is centred). The world map is full screen too. Missing art falls back to today's placeholders. Agreed with Assets & Audio (session_01EXCnXuYorGRpDEDtGgx3FL): TILE 22 -> 32 (1x pixel art, nearest filter, scale the view by the screen scale on high-DPI). Overlay keys exactly as MAPREAD1 2.1 (`overlay_npc`, `overlay_chest`, `overlay_lore`, `overlay_exit`, `overlay_encounter`, `overlay_roamer`, `overlay_poi`, plus optional open-chest and locked-exit keys); 32x32 NPC tokens. World backdrop ui key `ashlands_world_map` (1920x1080, letterboxed), pins 24x24 per kind placed by location field `map_pos` = [x, y] normalized 0..1 (Gameplay adds the field with MAPREAD1-L1; fall back to today's layout without it). Assets adds four 48x30 templates in `data/maps` and switches the locations onto them only after this view lands: ping them when it does. The view today has a fixed 24x16 canvas. Merge develop first: Gameplay's WORLDMAP-ENTER (0bd96e3) changed `world_map_view.gd` `travel_to`, and MUSIC1 added hooks in `region_map_view._refresh` / `world_map_view._refresh`.
2. **MAPREAD1-L2** with or right after the maps, once Gameplay's MAPREAD1-L1 lands (Gameplay pings).
3. **Mockups first, then build** (Tushar likes options plus an opinion; use the Artifact Design canvas like "Combat Screen Layouts" https://claude.ai/artifact/73p4skj1uDKHxm9XApz2hC): dialogue choices / selection options UI, an activity log, an active-quest tracker or journal (Tushar 02:23).
4. **Skills screen** (Tushar 02:06): show each skill's description (data has `description`, some with `{name}`/`{power}` templates), and only the one active version of a skill instead of every level. Find which screen (party view or the skill list) and mock up if it is more than a small fix.
5. **COMBAT-UI2-L1** (layout C + UIR-5 leftovers), **L2** (hover card), **L3** (tactic dropdown + Autoplay toggle) per `docs/specs/COMBAT_UI2.md`. Size hero cards for 1 to 3 heroes (the party starts as the warrior alone and recruits two). L3 builds on Gameplay's AUTO1-L API in f09c933 (`scripts/combat/session/combat_autoplay.gd`: `CombatAutoplay.new(session, CurrentGame.state())` after `session.start()`, `tick()` on the auto-play timer, Space/A -> `request_pause()`/`toggle()`, `tactic_entries()` dropdowns, `set_party_tactic` clears overrides, `set_hero_tactic`/`row_tactic`, "custom" after an edit, banner text from `paused(reason, by_player)` and `waiting(reason)`). Carry the AUTO1-L Done line in the L3 landing. `timeline_dock_ui.gd` stays Gameplay-owned; UI/UX builds the dropdown against the API.
6. **INTENT1-U** (after Gameplay's INTENT1-L and COMBAT-UI2-L2), **BOSS1-U**, **OBJ1-U** (after their -L rows). Gameplay (cse_01JracEuqBiTxnrgMZMMFE5s) pings when unblocked.
7. **UIR-1..UIR-7** from `/mnt/project-files/ui-review/UI_REVIEW1.md` (UIR-5 folded into COMBAT-UI2-L1). Small open questions: version label on the main menu, "(default)" marker on gear lines.

## Notes

- Gate in the cloud: `SUITE_TIMEOUT=580 bash tools/fc.sh gate > /tmp/claude-0/X.log 2>&1; tail -8`. Gate once: only affected suites while fixing, full gate before landing, again only if a develop merge touched the task's files.
- One screen per test case: many screens in one arena frame overflow Godot's message queue (signal 11 / exit 134).
- Right click as back must be read in `_input` (controls swallow it before `_unhandled_input`).
- Bus volumes: `AudioDirector.apply_saved()` is static and needs no director; `audio/mute` mutes Master.
- In-game Settings: Esc or the ☰ Menu button; `game_view` pauses (previous `process_mode` restored) and `BattleScreen.set_keys_blocked` is OR'ed with the results popup. Back to hub lives in the in-game Settings footer.
- Renders: Xvfb `xvfb-run -a -s "-screen 0 1280x720x24" godot --rendering-driver opengl3 --path . ...`, scripts outside the repo, screenshots to `/mnt/project-files/<task>/`.
- After each landing, reply in the thread with screenshots and "press Restart runner" (Tushar tests develop through run_dev.bat).
