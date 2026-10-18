# UI/UX handover

Written 2026-10-18 07:31 UTC by the UI/UX session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`. Dispatch mode is **economy** (user 02:35): logic-builder medium (sonnet), change-reviewer low.

## Landed this session (develop)

COMBAT-UI2-L2 hover card replaces the inspector (1c6fea4), COMBAT-UI2-L3 tactic dropdowns and Autoplay toggle, completing AUTO1-L (bb98f4f), ITEMUSE1-U battle pouch section on the Gear screen (069ac28; pulled forward at Gameplay's request so battle items are usable). No branch in flight.

## Waiting on the user

Mockups on the Design canvas "Map Screen Mockups" (https://claude.ai/artifact/2joZ2aDAARWS55hWnpCz2W). The coordinator holds the three pick cards and relays picks (quoting question, option and time):

- Dialogue layout: A centred popup / **B bottom bar with portrait (recommended)** / C speech panel by the speaker.
- Activity log: **A one-line ticker that expands (recommended)** / B toasts plus a Log tab / C docked panel.
- Quest tracker: A pinned card / **B card plus ! and ? map badges (recommended)** / C journal only with a Menu badge.

The picks become new build rows (dialogue folds into UIR-6). Write the TASKS rows when the answers arrive.

## Queue (in order)

1. **INTENT1-U** (spec `docs/specs/INTENT1.md` section 3; logic a56dda7: `CombatSession.enemy_intents()`, `intent_of`, `Combatant.has_foresight()`). Art from ART2-A (0218602): ui keys `intent_<kind>` for every `IntentRules.KINDS` entry (32 px red-rimmed chips) and `badge_foresight`, via `AssetRegistry.texture`. The hover card is `scripts/ui/unit_inspector_ui.gd`, owned by `combat_grid_panel.gd`.
2. **BOSS1-U** phase banner (BOSS1-L 5c85e7e: `CombatSession.phase_fired`, `CombatEncounter.phase_info`).
3. **OBJ1-U** (OBJ1-L 3abdb63, OBJ1-A bda1aa0): `EncounterObjectives.text_of(entry, name_of)` header, `held`/objective_rounds for "Held %d / %d", `tiles_of(entry)` ring tiles with tile `overlay_objective`, `CombatEncounter.guests/is_guest` for `chip_guest`, `chip_slay`, `protect` for "Must survive", `CombatSession.outcome` for the results line. The hero Tactic dropdown is already disabled for guests.
4. The picked dialogue / log / tracker builds, then UIR-1, UIR-2, UIR-3 (moves the pouch section into the Character popup Gear tab), UIR-4, UIR-6, UIR-7 (`/mnt/project-files/ui-review/UI_REVIEW1.md`).
5. Skills screen: each skill's description (`{name}`/`{power}` templates) and only the active rank.

## Notes

- Peers: Gameplay `session_01HQvENsjr5dFTBZp6L6GkPU` (queue empty after ITEMUSE1-L), Assets & Audio `session_01UywiZpBegbdrmR76smao3d`, Design `session_01Bd1NNdVo4TtRxU9hZ7BcNP`. Tell Gameplay and Assets your new id.
- Timeline row: `timeline_dock_ui.gd` control row (party and hero Tactic dropdowns, Auto, Clear, Confirm) with `row_metrics()` cached in `_m`; `CombatDockHost.setup(session, with_autoplay)` (false in replay/take-over arenas) owns `CombatAutoplay` and its 0.3 s timer.
- Layout C: `CombatHudMetrics` owns column widths, tile size and card lines; the party column takes the full height (no inspector).
- Gate in the cloud: `SUITE_TIMEOUT=580 bash tools/fc.sh gate > /tmp/claude-0/X.log 2>&1; tail -4`. Re-gate after a develop merge that touches shared combat or UI code.
- One screen per test case. Right click as back is read in `_input`.
- Renders: Xvfb `xvfb-run -a -s "-screen 0 1280x720x24" godot --rendering-driver opengl3 --path . ...`, scripts outside the repo, screenshots to `/mnt/project-files/<task>/`. After each landing, reply with screenshots and "press Restart runner".
- The designated branch `claude/ui-ux-90t4co` is reset to `origin/develop` per task (push with `--force-with-lease`; it only ever holds merged history between tasks).
