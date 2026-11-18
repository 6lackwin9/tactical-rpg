# Emberwake scenarios: notes for writing them

The format and the commands are in `engine/docs/authoring/scenarios.md`. These are facts about this game, measured 2026-11-04 (INTEG1-E1) in a real window and
2026-11-05 (HEADLESS1) with none. A run has no window by default; none here needs one. The gate runs them all (PR7-4); a baseline change for a golden shot
needs the user's approval (`engine/docs/authoring/scenarios.md` "Golden shots"). `saves/hub_start/` was made with the agent
playtest harness (`tools/fc.sh playtest`) after the prologue; make it again the same way when a save change breaks it.

## The scenarios

| File | Walks | Time, dev build |
| --- | --- | --- |
| `menu_opens` | The main menu to the new game screen and back | 0.8 s |
| `timeline_fight` | `tl_flank` on the Timeline ruleset, autoplay on screen to a win, with a clip | 13 s |
| `story_battle_hub` | A new Sunk Ward game: the opening vignette, the battle beat (TRANSITIONS1 T1), the first fight won by the link's `autoplay`, the victory beat and the results over the field (T2), then the prologue's vignettes and fights to the hub | 18 s |
| `screens_menus` | Review shots (PR7-4; golden once the user approves the baselines): main menu, new game, load, the five settings tabs, the Scene Runner | 3 s |
| `screens_hub` | Review shots from the save `saves/hub_start` (the hub right after the prologue): region map, talk box, notice board, shop, party (stats, skills, gear), world map | 25 s |
| `screens_battle` | Review shot: `tl_flank` seed 7 at its start, planning | 4 s |

The three screen scenarios also time frames (`perf`, ENGINE-2, numbers only, no budget): the main menu, the region map and
battle planning. Measured 2026-11-18 at 1280x720 in a dev build with no window: p50 4.6 ms, 29.5 ms and 20.2 ms.

A battle's `state.beat` is there only while a beat runs (`kind` `intro` or `victory`); wait for it to be absent before the fight takes input. A battle talk can open after the link's `autoplay` and holds the victory beat: Enter moves it on.

The main menu's screen id is `main_menu`.

## Fights

| How | What it does | Use it for |
| --- | --- | --- |
| `{"step": "key", "key": "a"}` | The battle's own autoplay: rounds play on screen, with their timing | Proving the fight on screen |
| `{"step": "link", "body": {"cmd": "autoplay"}}` | Resolves the whole battle in the host in one request: no playback, and the screen's message stays as it was | Reaching an outcome fast |

Wait for the end with `{"step": "wait", "until": {"path": "state.outcome", "ne": ""}, "timeout_ms": 240000}`: `state.outcome` is empty
until the fight ends. `tl_flank` on the Timeline ruleset took about 11 s on the key in a window, and 12 s with none at 15 fps, in a dev build.
A fight on one ruleset: launch with `{"screen": "arena", "scene": "tl_flank", "ruleset": "timeline"}` (`"encounter"` and `"rules"` are accepted as file load aliases; CLI launch and `shots` accept `encounter=` but refuse `rules=`). Any other `screen` with a `ruleset`, or an unknown ruleset, stops the game at launch.

## The Scene Runner's arena

| Fact | Detail |
| --- | --- |
| Screen id | `test_arena`, not `battle` |
| Ruleset | Pick it in the `runner.arena.ruleset` select. `screen=arena` with no `ruleset` opens the fight on the Classic rules, even for a `tl_` fight |
| Options | `<select id>.opt.<n>`, in list order, so an option's place moves when the data changes. A click on the select replies with the list |
| The end of a fight | `banner.leave` and `banner.restart`. No `battle_results` screen appeared |

## Comparing with a real window

Measured 2026-11-05 at 1280x720, dev build, `tl_flank` seed 7 on the Timeline ruleset.

| Compared | Window and no window |
| --- | --- |
| Main menu; fight start with `reduce_motion` | Byte for byte the same |
| Fight start with motion | 2,653 pixels within 1 px: idle clips and portraits at another moment. Two runs with no window also differ, by 1,100 |
| Fight end, 2 frames after the outcome | 9,344 pixels: the playback's last moments, shot a little apart in time |
| Fight end, 3 s later | 484 pixels: only the bodies' idle frame, faint behind the banner |
| Result | The same: `won` in 2 rounds, Vanguard HP 78/210, Spiker 210/210, all log lines, 33 requests, both checkpoints |
| Playback, key to outcome | 12.3 s against 10.9 s (about 11 % long; 12.1 s in an earlier run): timers fire on the next frame, at 15 fps |

So a shot shows the same state, not the same animation frame; to compare pixels, use `reduce_motion` and a screen at rest.

`fps=` starts at 12 because the battle's effects clock takes at most 100 wall ms a frame (`battle.rs`, `tick`): at 10 fps the frame gap is 100 ms plus overshoot, and each frame would lose the overshoot.
