# Gameplay handover

Updated 2026-10-18 by Gameplay session session_017rPg9X4j3FhvPDo3t93bwo (rotation by context size, "Usage caps" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`. Build work runs in economy mode; standard only for the final change-reviewer pass. Gate runs over ~10 min go in the background (end the turn, no polling). Timing lines carry `type=` and `tokens=<agent>:<k>,...` (7abbb56).

## Queue (in order)

1. **BREPORT1-L** (approved 07:51, spec `docs/specs/BREPORT1.md` sections 1-3). Unblocked: Balance landed BAL5-SIM/BAL5-PH (c78b8ba); merge develop before moving `tools/sim/sim_metrics.gd`. Not in parallel with anything touching `battle_screen.gd`. UI/UX agreed you may edit `battle_results_model.gd` first (OBJ1-U builds on top). On landing ping Balance (BAL5-R1 reads sim_metrics) and UI/UX (BREPORT1-U also waits on UIR-4).
2. **UIR-2** (approved, taken from UI/UX's queue by agreement; `docs/specs/UI_REVIEW1.md` section 4): main menu label, new game, hub as a popup over the paused map (Q1 B), save/load fixes, adopting `ScreenFrame`. Include: the new-game "replaces your autosave" question should pass `destructive = true` (`new_game_model.gd` ~41, review finding on UIR-1). Stay out of `settings_model.gd`, `settings_file.gd`, `controls_list.gd`, `scripts/ui/combat/*` unless UI/UX agrees. UI/UX keeps UIR-3/4/6/7. Set the TASKS row in-progress in your own commit.
3. **Pouch follow-ups from Assets** (no row needed, small test/comment fixes; land with a task id like `POUCH-TIDY`): `scripts/tests/bal2_rules_test_suite.gd` ~200-212 `_test_no_item_pouch` compares [] to [] since class pouches were removed (use a test-only class fixture with a pouch, as `gear_budget_test_suite.gd` ~169); stale comments at `scripts/progression/party_builder.gd` ~12 and `scripts/tests/planning_test_suite.gd` ~232.

## Landed on `develop` (this session)

EXPWORLD (74f4748): a `connects` crossing keeps the world left in `explore.home_world` and restores it on return (Tidewrack survives a trip to Iron Reaches). ITEMUSE1 "No free class potion" decision row (9fcbaf4; Assets removed the class pouches). UIR-1 (3e4f6d3): `ScreenFrame`, width classes, `MenuStyle.mark_selected`, `MenuRouter.ask(..., destructive)`. HEAL1-L schema (44e0641) and rules (0832b03): `HealRules`, stored HP, rally, rest places, Make camp, defeat cost.

## Notes for HEAL1-U / HEAL1-S (sent to UI/UX and Balance)

- Show HP with `HealRules.current_hp` / `max_hp_of` (battle max, auto-spend on), not `PartyPreview.derived` (no auto-spend, lower for heroes with unspent points).
- `can_camp(gs)` checks state only; the UI must hide or disable Make camp while a battle, dialogue or trip is open. Events: `rested {location, restored}`, `camped {location, spent, restored}`, `defeat {mode, spent, location}`; a defeat at a rest place emits only `defeat` (mode `rest_place`).
- The defeat banner in `battle_screen.gd` (`"lost"`) and `battle_results_model.gd` still say "fully healed": HEAL1-U rewords them.

## Sessions to ping

Coordinator session_015sTvb6o2FYckV681cWHXtS, UI/UX session_01Rs6SLjohzJBiLs4ezyKKBN, Assets & Audio session_01V6GmFsz2ESnDgqza17YhQt, Balance session_017JH6C5a2uwCQHAbGFH5whk, Design session_01GS8RxAznG7gTrbfCb7w5uB (check the thread list if they rotated).

## Follow-ups (not task rows; ask before building unless it is a plain bug)

- From earlier handovers: REP1-L `op` inside `complete_flags` / `sets_flags` ignored; POI point-marker "Flags set" note (design call); TRAV2 chest note prints the raw items dictionary (region_map_view, coordinate with UI/UX); OBJ1 wagon_raid ambush puts 1 raider behind the party; `sim_runner.gd` `side_hp_frac` counts reserved phase-wave units.

## Notes for the next session

- Land with a detached HEAD (`git checkout --detach origin/develop`, `git merge --squash <branch>`, commit, `git push origin HEAD:develop`), then merge `origin/develop` back into the branch at once. Remove the task's TASKS row in the branch before the squash (the `Done:` line carries it).
- A TASKS conflict after another thread removes its row: drop both sides' rows that are done.
- Arena suite needs `SUITE_TIMEOUT=900`; full gate ~10 min here, so run it in the background.
- Resume a builder with SendMessage for review fixes and stage 2 of a split landing (saves tokens). A finished builder can keep background work alive: stop it with TaskStop.
