# UI/UX handover

Written 2026-10-18 16:52 UTC by the UI/UX session at the project wind-down (Tushar 16:28: agents finish in-flight work, then act only on his direct requests; "interactive" mode, skill `planner-ops`). Read this first, then `docs/workflow/cloud.md` and `docs/preferences/ui.md` (add a dated line whenever Tushar states or approves a UI preference). Do not start queue items on your own; wait for his request. Never touch the `engine` branch (decision "Engine branch hold"). Landed tasks no longer get a "landed" ping to Merge & Release (cloud.md step 7, f1db72d).

## Landed this session (develop)

- UIR-6 (c2de328): dialogue as a full-width bottom bar (canvas layer 1, portrait, 1-9 choices, lock reasons via `DialogueRules.lock_reason`, Esc / right click = Farewell / Cancel / Continue); shop popup (layer 31, Buy / Sell tabs, detail line); text scale. Renders `/mnt/project-files/uir6/`.
- COMBAT-UI3 (d2d1d64): layout A. Planning strip, merged enemy card, selected-hero panel, missing-actions warning (setting `ui/warn_missing_actions`), log ticker in the bottom bar. Decision row in `docs/decisions/combat.md`. Renders `/mnt/project-files/combat-ui3/`. Tushar reviewed it 17:00-17:05; follow-up COMBAT-UI3-F (ce3c381): pickers fill empty blocks, hostile HP beside the name, smaller strip labels with a kind-symbol fallback (1-block names show the symbol from Normal text up; a second smaller font tier was not tried).

No branch in flight; `claude/project-thread-durs1a` is fully squashed into develop.

## Queue (in order, start only on request)

UIR-3B (Gear tab stash stat delta, item descriptions); OBJ1-U; MORALE1-U (Gameplay API: intent records carry morale_state, morale_group; CatalogEncounter is_fearless / morale_state_of / morale_group_of / flee_ratio_of; objective_state()["morale"]; surrender = withdraw_in_place); PACE1-U (unblocked by PACE1-L a1ca665, still needs BOSS1-U: `CombatEncounter.is_enraged(c)`, intent records gain `enraged`, phase signal info has `enrage` for the second banner line, phase log entry "Enraged"; the new `empower` status makes Focus log entries (label "Empowered", combat_log_builder.gd:109) pop the empower icon: rename Focus's label or accept it); MAPLOG1-U (also reword world map notes still showing ids: "Entering <qid>." world_map_view.gd ~378, "Road fight! <encounter id>", "Flags set: {...}"); QUESTTRACK1-U; RCLOSE1-U; MAPNAV1-U (UIR-6 done); UIR-4, UIR-7; BREPORT1-U and HEAL1-U after UIR-4. Enemy-card work now goes in `scripts/ui/combat/combat_enemy_card_panel.gd` (unit_inspector_ui.gd is gone).

## Open questions (for Tushar when relevant)

- UIR-6: Esc / right click on a story end node does Continue; RCLOSE1-U may want explicit buttons only. Shop "Owned" counts stash plus worn copies. No key to switch shop tabs yet (UIR-7).
- COMBAT-UI3: the Controls tab is wider than the settings popup at Large / Extra large text (911 / 992 px vs 864), a small fix task.
- From UIR-3: Stats tab has no bar action; Hub entry still reads "Party".

## Notes

- Peers: Gameplay `session_01QCe4DugW3btbTntzfddbhN`, Platform `session_01F3J8BGWSSBbqckdqifYoJF`, Design `session_01GS8RxAznG7gTrbfCb7w5uB`, Balance `session_017JH6C5a2uwCQHAbGFH5whk`, Merge & Release `session_01KhQLgrvAgUunc9TcJKTHzK`.
- Renders: Xvfb, helper scripts outside the repo, screenshots to `/mnt/project-files/<task>/`; reply with them and "press Restart runner".
