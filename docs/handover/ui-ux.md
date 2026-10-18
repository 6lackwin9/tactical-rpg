# UI/UX handover

Written 2026-10-18 14:42 UTC by the UI/UX session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md` and `docs/preferences/ui.md` (the UI preferences file this thread owns: add a dated line whenever Tushar states or approves a UI preference). Dispatch mode: standard until ~15:45 UTC, then economy until ~18:45 UTC (Platform, user 12:42; keep `type=` and `tokens=` on Timing lines). Land with `bash tools/fc.sh gate auto`; after landing, send Merge & Release (`cse_01KhQLgrvAgUunc9TcJKTHzK`) one line with task id and sha.

## Landed this session (develop)

- INTENT1-U follow-up (f65f0bb): enemies whose drafts a packmate broke now re-draft (schedule_enemy_ai); `idle` intent kind shows "Holding"; `arriving` only for units placed after scheduling. Balance and Gameplay told.
- UIR-3 (3878540): Character popup (`character_view.gd`, party_view.gd removed). Also fixed: questions and the Gear board were hidden under in-play popups (canvas layers popup 31, board 32, question 33).
- MAPDEV1-U (7780fe0): `debug_tools` export on RegionMapView/WorldMapView, play bars show place and supplies, chest notes name items.

No branch in flight; `claude/project-thread-7s97az` equals `origin/develop`.

## Queue (in order)

1. **UIR-6** dialogue as a full-width bottom bar with portrait (pick B, a preference), shop as a popup, text scale applied. Region map rows run one at a time.
2. **COMBAT-UI3**: Tushar picked A (Design thread 11:51, verbatim: "A but have the block use the same continuous block and color code for kind, and make confirm round just confirm with warning if some blocks are empty for any party member, it can be disabled in setting to not give missing actions warning"). 11:54: the log keeps click-to-expand; party column: compact member cards plus a selected-hero panel (Design rec, a preference; fall back to richer cards if cramped). Layout A is a preference; continuous blocks, colour code and the confirm warning are his decisions. Add the COMBAT-UI3 TASKS row as approved (superseding HOVER1-U) and a decision row citing his 11:51 message in the task commit. Mockup https://claude.ai/artifact/E75abGsPLH6E6GZfBXVASi.
3. UIR-3B (Gear tab stash stat delta, item descriptions); OBJ1-U; MORALE1-U (Gameplay's API note: intent records carry morale_state, morale_group; CatalogEncounter is_fearless / morale_state_of / morale_group_of / flee_ratio_of; objective_state()["morale"]; surrender = withdraw_in_place, results wording is ours); MAPLOG1-U (after MAPDEV1-U; also reword world map notes still showing ids: own-pin "Entering <qid>." world_map_view.gd ~378, "Road fight! <encounter id>", "Flags set: {...}"); QUESTTRACK1-U; RCLOSE1-U; MAPNAV1-U (after UIR-6); UIR-4, UIR-7; BREPORT1-U and HEAL1-U after UIR-4.

## Notes

- Open UI-review questions from UIR-3: Stats tab has no bar action; Hub entry still reads "Party" (could be "Character").
- Peers: Gameplay `session_01448UbhXvTzeBA1Brw4buLy`, Platform `cse_01F3J8BGWSSBbqckdqifYoJF` (new 12:59), Design `session_01GS8RxAznG7gTrbfCb7w5uB`, Balance `session_017JH6C5a2uwCQHAbGFH5whk`. Tell them your new id.
- Renders: Xvfb, helper scripts outside the repo, screenshots to `/mnt/project-files/<task>/`; reply with them and "press Restart runner".
