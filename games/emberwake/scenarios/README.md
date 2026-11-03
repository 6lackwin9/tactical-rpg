# Emberwake scenarios: notes for writing them

The format and the commands are in `engine/docs/authoring/scenarios.md`. These are facts about this game, measured 2026-11-04 (INTEG1-E1).

## Fights

| How | What it does | Use it for |
| --- | --- | --- |
| `{"step": "key", "key": "a"}` | The battle's own autoplay: rounds play on screen, with their timing | Proving the fight on screen |
| `{"step": "link", "body": {"cmd": "autoplay"}}` | Resolves the whole battle in the host in one request: no playback, and the screen's message stays as it was | Reaching an outcome fast |

Wait for the end with `{"step": "wait", "until": {"path": "state.outcome", "ne": ""}, "timeout_ms": 240000}`: `state.outcome` is empty
until the fight ends. `tl_flank` on the Timeline ruleset took about 11 s on the key, in a dev build.

## The Scene Runner's arena

| Fact | Detail |
| --- | --- |
| Screen id | `test_arena`, not `battle` |
| Ruleset | Pick it in the `runner.arena.ruleset` select. `screen=arena` opens the fight on the Classic rules, even for a `tl_` fight |
| Options | `<select id>.opt.<n>`, in list order, so an option's place moves when the data changes. A click on the select replies with the list |
| The end of a fight | `banner.leave` and `banner.restart`. No `battle_results` screen appeared |
